//! Oracle settle-shell census (#442): measures, per level, the hidden live
//! window the oracle helper's settled-warp bootstrap runs between its first
//! target-level poll and scenario frame 0. The state sweep's checkpoints
//! observe the *aftermath* of that unobserved window; native models its
//! length with a hand-tuned per-level/family table
//! (`wildside-core/src/scenario.rs`). This census replaces hand tuning with
//! measurement: it re-runs the helper's own bootstrap with
//! `--capture-settle-trace` and derives the shell length in-process from the
//! captured `target_live` rows — no helper edit, so the oracle cache key
//! (`jgenesis_helper_cache_version`) is untouched.
//!
//! Derivation semantics (helper `boot_and_warp_oracle` target_live loop):
//! each iteration captures its row BEFORE the liveness test, the pause
//! recovery branch ticks one extra frame, and the loop breaks — without
//! ticking — at the start of the iteration whose row completes the 60-frame
//! stable streak. Scenario frame 0 therefore starts from the state of the
//! LAST captured target row, after
//! `last_live_step + count(paused live rows)` emulator frames. Native's
//! `settle_target_level_live_window` steps `target_live_frames - 1` frames
//! for a window of T, so the equivalent native window is that shell length
//! plus one.

use super::*;
use crate::capture::sha1_hex;
use crate::oracle::{
    OracleEnvironment, OracleRunOptions, OracleSettleTraceRow, ensure_oracles,
    run_jgenesis_capture_dynamic_with_options,
};
use std::collections::BTreeMap;

/// Mirrors the helper's stable-liveness window (`stable_target_frames >= 60`).
const SETTLE_LIVE_STABLE_FRAMES: usize = 60;

/// Bump when the derivation in this module changes meaning, so downstream
/// consumers (the checked-in shell table, S3b/S3c of #442) can detect stale
/// artefacts mechanically. Version 2 adds the RNG-freeze load-completion
/// fields (`rng_freeze_start_step`/`load_complete_step`/`title_wait_live_rows`).
pub(crate) const SETTLE_CENSUS_DERIVATION_VERSION: u32 = 2;

/// Minimum consecutive title-wait rows with a frozen RNG offset to count as
/// the loader freeze. Real loads measure 60-73 frozen rows (0x60/0x30, run
/// 1786640409); the threshold only guards against one-off lag stalls.
const MIN_RNG_FREEZE_ROWS: usize = 8;

/// Frame-0 witness: the state the last captured settle row exposes, which is
/// exactly the state scenario frame 0 starts from. RNG offset + full-ring
/// sha1 discriminate shells that agree on length but diverge in content.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct SettleFrame0Witness {
    pub(crate) rng_offset: u16,
    pub(crate) rng_ring_sha1: String,
    pub(crate) number_objects: u16,
    pub(crate) kid_x_pos: u32,
    pub(crate) kid_y_pos: u32,
    pub(crate) camera_x_pos: u32,
    pub(crate) camera_y_pos: u32,
}

/// One level's measured settle shell. `measurable == false` rows carry a
/// concrete reason instead of silently defaulting to the 60-frame window.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct SettleCensusLevel {
    pub(crate) level_id: u8,
    pub(crate) measurable: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) unmeasurable_reason: Option<String>,
    /// `target_live` (warped levels) or `title_target_live` (level 0 path).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) phase: Option<String>,
    /// Emulator frames between the first target poll row and scenario
    /// frame 0: `last_live_step + paused_live_rows`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) shell_frames_to_frame0: Option<usize>,
    /// The equivalent native `settle_target_level_live_window` argument:
    /// `shell_frames_to_frame0 + 1` (native steps T - 1 frames).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) native_target_live_frames: Option<usize>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) first_live_step: Option<usize>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) last_live_step: Option<usize>,
    /// Rows where the pause-recovery branch fired (each adds one hidden
    /// emulator frame beyond the step count).
    pub(crate) paused_live_rows: usize,
    /// live -> not-live transitions before the final stable streak; a flap
    /// means the target level was reached, lost, and re-reached.
    pub(crate) flap_resets: usize,
    /// True when the first Start+A+C press did NOT flip `Game_Mode` to
    /// $0008 at the `post_skip_input` row, i.e. the helper retried and may
    /// have taken its direct-write fallback. The trace cannot distinguish
    /// second-press success from the fallback; both mean extra hidden shell
    /// content before the title wait. `None` on the level-0 path (no skip).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) took_debug_skip: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) frame0: Option<SettleFrame0Witness>,
    /// First title-wait step of the loader's RNG-offset freeze (#442 shell
    /// content): the loader blocks the frame loop, so controller draws stop.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) rng_freeze_start_step: Option<usize>,
    /// First title-wait step drawing RNG again after the loader freeze — the
    /// measured per-level load completion the native shell must anchor on
    /// (0x30 measures 75, 0x60 measures 62; run 1786640409).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) load_complete_step: Option<usize>,
    /// Title-wait rows the level runs live after load completion
    /// (`120 - load_complete_step` on the standard debug-warp path).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) title_wait_live_rows: Option<usize>,
}

/// #442 shell content: locate the loader freeze inside the title-wait rows as
/// the longest run of consecutive rows with an identical RNG offset. Returns
/// `(freeze_start_step, load_complete_step, title_wait_live_rows)` only when
/// the freeze both spans `MIN_RNG_FREEZE_ROWS` and observably ends inside the
/// title wait.
fn derive_title_wait_rng_freeze(trace: &[OracleSettleTraceRow]) -> Option<(usize, usize, usize)> {
    let rows: Vec<&OracleSettleTraceRow> = trace
        .iter()
        .filter(|row| row.phase == "title_wait")
        .collect();
    let mut best: Option<(usize, usize)> = None; // (start_index, len)
    let mut run_start = 0usize;
    for index in 1..rows.len() {
        if rows[index].rng_offset != rows[run_start].rng_offset {
            let run_len = index - run_start;
            if best.is_none_or(|(_, best_len)| run_len > best_len) {
                best = Some((run_start, run_len));
            }
            run_start = index;
        }
    }
    // A run still open at the last row never resumed: the load did not
    // observably complete inside the title wait, so it cannot become `best`.
    let (start_index, run_len) = best?;
    if run_len < MIN_RNG_FREEZE_ROWS {
        return None;
    }
    let resume_index = start_index + run_len;
    Some((
        rows[start_index].step,
        rows[resume_index].step,
        rows.len() - resume_index,
    ))
}

/// Census artefact written to `.cache/audit/oracle-settle-census.json`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct OracleSettleCensus {
    /// `jgenesis_helper_cache_version()` at measurement time: any helper edit
    /// changes this, marking the census stale.
    pub(crate) helper_cache_version: String,
    pub(crate) rom_sha1: String,
    pub(crate) derivation_version: u32,
    pub(crate) census_run_id: String,
    pub(crate) levels: Vec<SettleCensusLevel>,
}

fn row_is_live(row: &OracleSettleTraceRow, target_level: u8) -> bool {
    // Mirror of the helper's `target_level_is_live`: the row is captured at
    // the same instant the predicate is evaluated, so the same fields decide.
    row.game_mode == 0x000C
        && row.current_level_id == u16::from(target_level)
        && !row.paused
        && row.kid_x_pos != 0
        && row.number_objects != 0
}

fn row_takes_pause_branch(row: &OracleSettleTraceRow, target_level: u8) -> bool {
    row.game_mode == 0x000C && row.current_level_id == u16::from(target_level) && row.paused
}

fn unmeasurable(
    level_id: u8,
    phase: Option<&str>,
    took_debug_skip: Option<bool>,
    reason: String,
) -> SettleCensusLevel {
    SettleCensusLevel {
        level_id,
        measurable: false,
        unmeasurable_reason: Some(reason),
        phase: phase.map(str::to_owned),
        shell_frames_to_frame0: None,
        native_target_live_frames: None,
        first_live_step: None,
        last_live_step: None,
        paused_live_rows: 0,
        flap_resets: 0,
        took_debug_skip,
        frame0: None,
        rng_freeze_start_step: None,
        load_complete_step: None,
        title_wait_live_rows: None,
    }
}

/// Pure derivation core: settle-trace rows -> one census row.
pub(crate) fn derive_settle_census_level(
    level_id: u8,
    trace: &[OracleSettleTraceRow],
) -> SettleCensusLevel {
    let took_debug_skip = trace
        .iter()
        .find(|row| row.phase == "post_skip_input")
        .map(|row| row.game_mode != 0x0008);

    let mut phase = "target_live";
    let mut rows: Vec<&OracleSettleTraceRow> =
        trace.iter().filter(|row| row.phase == phase).collect();
    if rows.is_empty() {
        phase = "title_target_live";
        rows = trace.iter().filter(|row| row.phase == phase).collect();
    }
    if rows.is_empty() {
        return unmeasurable(
            level_id,
            None,
            took_debug_skip,
            "no target_live/title_target_live rows captured (settled-warp shell missing)".into(),
        );
    }

    let last = rows[rows.len() - 1];
    let liveness: Vec<bool> = rows.iter().map(|row| row_is_live(row, level_id)).collect();
    let trailing_live_streak = liveness.iter().rev().take_while(|&&live| live).count();
    if trailing_live_streak < SETTLE_LIVE_STABLE_FRAMES {
        return unmeasurable(
            level_id,
            Some(phase),
            took_debug_skip,
            format!(
                "settle loop never stabilised: trailing live streak {} < {} over {} rows \
                 (last row: game_mode={:#06x} level={:#06x} paused={} objects={})",
                trailing_live_streak,
                SETTLE_LIVE_STABLE_FRAMES,
                rows.len(),
                last.game_mode,
                last.current_level_id,
                last.paused,
                last.number_objects
            ),
        );
    }

    let paused_live_rows = rows
        .iter()
        .filter(|row| row_takes_pause_branch(row, level_id))
        .count();
    let flap_resets = liveness
        .windows(2)
        .filter(|pair| pair[0] && !pair[1])
        .count();
    let first_live_step = liveness
        .iter()
        .position(|&live| live)
        .map(|index| rows[index].step);
    let shell_frames_to_frame0 = last.step + paused_live_rows;
    let rng_freeze = derive_title_wait_rng_freeze(trace);

    SettleCensusLevel {
        level_id,
        measurable: true,
        unmeasurable_reason: None,
        phase: Some(phase.to_owned()),
        shell_frames_to_frame0: Some(shell_frames_to_frame0),
        native_target_live_frames: Some(shell_frames_to_frame0 + 1),
        first_live_step,
        last_live_step: Some(last.step),
        paused_live_rows,
        flap_resets,
        took_debug_skip,
        frame0: Some(SettleFrame0Witness {
            rng_offset: last.rng_offset,
            rng_ring_sha1: sha1_hex(&last.rng_ram),
            number_objects: last.number_objects,
            kid_x_pos: last.kid_x_pos,
            kid_y_pos: last.kid_y_pos,
            camera_x_pos: last.camera_x_pos,
            camera_y_pos: last.camera_y_pos,
        }),
        rng_freeze_start_step: rng_freeze.map(|(start, _, _)| start),
        load_complete_step: rng_freeze.map(|(_, resume, _)| resume),
        title_wait_live_rows: rng_freeze.map(|(_, _, live_rows)| live_rows),
    }
}

/// The exact scenario the state-tier sweep runs, truncated to a single frame:
/// the settle shell precedes scenario frame 0, so measuring it needs no idle
/// tail. Deliberately NOT routed through the oracle cache — the census runs
/// with `--capture-settle-trace` into its own per-level out dirs.
fn settle_census_scenario(level_id: u8) -> DynamicScenario {
    let mut scenario = idle_level_scenario(level_id, None);
    scenario.settle_oracle_warp = true;
    scenario.slug = format!("settle_census_{level_id:02x}");
    scenario.frame_count = 1;
    scenario.script = vec![0u8; 2];
    scenario.checkpoints = vec![DynamicCheckpoint {
        label: "frame_0".into(),
        frame_index: 0,
    }];
    scenario
}

/// #442 shell-content instrumentation: persist the helper's full settle rows
/// (RNG ring, kid kinematics, object-slot dump per step) beside the level's
/// scenario so per-step shell analysis reads measured data. The census itself
/// only keeps derived lengths; this dump is the raw evidence.
pub(crate) fn write_settle_trace_dump(
    level_dir: &Path,
    rows: &[OracleSettleTraceRow],
) -> Result<PathBuf> {
    let path = level_dir.join("settle-trace.json");
    fs::write(&path, serde_json::to_vec_pretty(rows)?)
        .with_context(|| format!("failed to write {}", path.display()))?;
    Ok(path)
}

fn measure_level(
    env: &OracleEnvironment,
    rom_path: &Path,
    run_dir: &Path,
    level_id: u8,
    dump_trace: bool,
) -> SettleCensusLevel {
    let level_dir = run_dir.join(format!("levels/{level_id:02x}"));
    let result = (|| -> Result<SettleCensusLevel> {
        fs::create_dir_all(&level_dir)
            .with_context(|| format!("failed to create {}", level_dir.display()))?;
        let scenario = settle_census_scenario(level_id);
        let scenario_file = level_dir.join("scenario.json");
        fs::write(&scenario_file, serde_json::to_vec_pretty(&scenario)?)
            .with_context(|| format!("failed to write {}", scenario_file.display()))?;
        let options = OracleRunOptions {
            capture_settle_trace: true,
            ..OracleRunOptions::default()
        };
        let manifest = run_jgenesis_capture_dynamic_with_options(
            env,
            rom_path,
            &scenario_file,
            &level_dir,
            &options,
        )?;
        if dump_trace {
            write_settle_trace_dump(&level_dir, &manifest.settle_trace)?;
        }
        Ok(derive_settle_census_level(level_id, &manifest.settle_trace))
    })();
    result.unwrap_or_else(|error| {
        unmeasurable(
            level_id,
            None,
            None,
            format!("oracle run failed: {error:#}"),
        )
    })
}

/// Serialise a census as the checked-in shell table
/// (`crates/wildside-cli/data/oracle-settle-shell.yaml`). The staleness unit
/// test pins `helper_cache_version` to the live helper, so a helper edit goes
/// red in the gate until the table is re-measured.
pub(crate) fn write_shell_table(census: &OracleSettleCensus, path: &Path) -> Result<()> {
    let header = "\
# Oracle settle-shell table (#442 S3b): the MEASURED hidden live window per
# level, plus the frame-0 RNG/state witness the oracle exposes at scenario
# start. Reference data for the shell-content investigation — the measured
# swap into the native resolver is stopped until native reproduces the
# shell's RNG content (archive #442).
# Regenerate after any oracle-helper edit:
#   cargo run --release -p wildside-cli -- oracle-settle-census \\
#     --shell-table crates/wildside-cli/data/oracle-settle-shell.yaml
";
    let body = serde_yaml::to_string(census).context("failed to serialise shell table")?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("failed to create {}", parent.display()))?;
    }
    fs::write(path, format!("{header}{body}"))
        .with_context(|| format!("failed to write {}", path.display()))?;
    println!("wrote {}", path.display());
    Ok(())
}

/// Imperative shell: run the helper per level (parallel, cache-bypassing),
/// derive rows in-process, and write the census artefact. `from_json` skips
/// the oracle runs and reloads a previous census artefact (e.g. to reformat
/// it as the checked-in shell table).
pub(crate) fn run_oracle_settle_census(
    repo_root: &Path,
    rom_path: &Path,
    levels: &[u8],
    output: Option<&Path>,
    from_json: Option<&Path>,
    shell_table: Option<&Path>,
    dump_trace: bool,
) -> Result<()> {
    if let Some(json_path) = from_json {
        let census: OracleSettleCensus = serde_json::from_str(
            &fs::read_to_string(json_path)
                .with_context(|| format!("failed to read {}", json_path.display()))?,
        )
        .with_context(|| format!("failed to parse {}", json_path.display()))?;
        if let Some(table_path) = shell_table {
            write_shell_table(&census, table_path)?;
        }
        return Ok(());
    }
    let rom_bytes = fs::read(rom_path)
        .with_context(|| format!("failed to read ROM from {}", rom_path.display()))?;
    let env = ensure_oracles(repo_root)?;

    let census_run_id = chrono_timestamp();
    let run_dir = repo_root
        .join(".cache/audit/oracle-settle-census-runs")
        .join(&census_run_id);
    fs::create_dir_all(&run_dir)
        .with_context(|| format!("failed to create {}", run_dir.display()))?;

    let mut rows: Vec<SettleCensusLevel> = levels
        .par_iter()
        .map(|&level_id| {
            println!("settle census level {level_id:#04x}...");
            measure_level(&env, rom_path, &run_dir, level_id, dump_trace)
        })
        .collect();
    rows.sort_by_key(|row| row.level_id);

    let census = OracleSettleCensus {
        helper_cache_version: jgenesis_helper_cache_version().to_owned(),
        rom_sha1: sha1_hex(&rom_bytes),
        derivation_version: SETTLE_CENSUS_DERIVATION_VERSION,
        census_run_id,
        levels: rows,
    };

    println!("\nlevel  shell  native_T  first_live  flaps  paused  debug_skip");
    let mut histogram: BTreeMap<usize, usize> = BTreeMap::new();
    for row in &census.levels {
        match (row.shell_frames_to_frame0, row.native_target_live_frames) {
            (Some(shell), Some(native)) => {
                *histogram.entry(native).or_default() += 1;
                println!(
                    "{:#04x}   {:>5}  {:>8}  {:>10}  {:>5}  {:>6}  {}",
                    row.level_id,
                    shell,
                    native,
                    row.first_live_step
                        .map_or_else(|| "-".into(), |step| step.to_string()),
                    row.flap_resets,
                    row.paused_live_rows,
                    row.took_debug_skip
                        .map_or_else(|| "-".into(), |took| took.to_string()),
                );
            }
            _ => println!(
                "{:#04x}   UNMEASURABLE: {}",
                row.level_id,
                row.unmeasurable_reason.as_deref().unwrap_or("unknown")
            ),
        }
    }
    println!("\nnative_target_live_frames histogram:");
    for (frames, count) in &histogram {
        println!("  {frames:>4}: {count} level(s)");
    }
    let unmeasurable_count = census.levels.iter().filter(|row| !row.measurable).count();
    if unmeasurable_count > 0 {
        println!("unmeasurable levels: {unmeasurable_count}");
    }

    let output_path = output.map_or_else(
        || repo_root.join(".cache/audit/oracle-settle-census.json"),
        Path::to_path_buf,
    );
    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("failed to create {}", parent.display()))?;
    }
    fs::write(&output_path, serde_json::to_vec_pretty(&census)?)
        .with_context(|| format!("failed to write {}", output_path.display()))?;
    println!("\nwrote {}", output_path.display());
    if let Some(table_path) = shell_table {
        write_shell_table(&census, table_path)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// #442 S3b staleness guard: the checked-in shell table records which
    /// oracle-helper build measured it. Any helper edit changes
    /// `jgenesis_helper_cache_version()`, so this goes red until the table is
    /// re-measured — the census can never silently describe a stale shell.
    #[test]
    fn checked_in_shell_table_matches_the_live_helper_and_derivation() {
        let table = include_str!("../../data/oracle-settle-shell.yaml");
        let census: OracleSettleCensus =
            serde_yaml::from_str(table).expect("shell table must parse as the census schema");
        assert_eq!(
            census.helper_cache_version,
            jgenesis_helper_cache_version(),
            "oracle helper changed since the shell table was measured — re-run \
             `cargo run --release -p wildside-cli -- oracle-settle-census \
             --shell-table crates/wildside-cli/data/oracle-settle-shell.yaml`"
        );
        assert_eq!(census.derivation_version, SETTLE_CENSUS_DERIVATION_VERSION);
        assert_eq!(census.levels.len(), 69, "one row per valid BCD level id");
        assert!(
            census.levels.iter().all(|level| level.measurable),
            "the 2026-08-01 census measured every level; a new unmeasurable row needs investigation"
        );
    }

    fn row(phase: &'static str, step: usize, live: bool, paused: bool) -> OracleSettleTraceRow {
        OracleSettleTraceRow {
            phase: phase.into(),
            step,
            game_mode: 0x000C,
            current_level_id: if live || paused { 0x0050 } else { 0x0008 },
            rng_offset: 0xFFCB,
            rng_last_byte: 0x32,
            rng_ram: vec![0xAA; 55],
            paused,
            camera_x_pos: 0x0100_0000,
            camera_y_pos: 0x0200_0000,
            kid_x_pos: if live || paused { 0x0080_0000 } else { 0 },
            kid_y_pos: 0x0090_0000,
            kid_x_vel: 0,
            kid_y_vel: 0,
            number_objects: if live || paused { 7 } else { 0 },
            object_slots_loc_e1e: String::new(),
            drips_object_slots_loc_33488: String::new(),
            background_scroll_loc_3050a: String::new(),
            background_row_offsets_loc_30744: String::new(),
            background_source_window_loc_30744: String::new(),
            background_source_absolute_loc_30744: String::new(),
            background_source_rows_loc_12848: String::new(),
            background_plane_b_rows_loc_307d8: String::new(),
            background_plane_b_spans_loc_307d8: String::new(),
        }
    }

    fn skip_row(game_mode: u16) -> OracleSettleTraceRow {
        let mut skip = row("post_skip_input", 0, false, false);
        skip.game_mode = game_mode;
        skip
    }

    fn cached_generated_manifest() -> Option<wildside_data::AssetManifest> {
        let repo_root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .and_then(Path::parent)
            .expect("wildside-cli crate should live under <repo>/crates/wildside-cli")
            .to_path_buf();
        let manifest_path = repo_root
            .join(".cache/generated/28b904000b2863b6760531807760b571f1a5fc1d/asset-manifest.json");
        let bytes = fs::read(manifest_path).ok()?;
        serde_json::from_slice(&bytes).ok()
    }

    fn settled_warp_frame0_witness_mismatches(levels: &[u8]) -> Vec<String> {
        use wildside_core::{FrameInput, LevelId, NativeRuntime};
        let table = include_str!("../../data/oracle-settle-shell.yaml");
        let census: OracleSettleCensus =
            serde_yaml::from_str(table).expect("shell table must parse as the census schema");
        let Some(manifest) = cached_generated_manifest() else {
            return Vec::new();
        };
        let _ = FrameInput::default();
        let mut mismatches = Vec::new();
        for level in &census.levels {
            if !levels.contains(&level.level_id) {
                continue;
            }
            let (Some(_load_complete), Some(frame0)) =
                (level.load_complete_step, level.frame0.as_ref())
            else {
                continue;
            };
            let mut runtime = NativeRuntime::with_generated_level_layout(manifest.clone());
            runtime.set_recorded_title_card_handoff_delay(true);
            runtime.boot_to_level_for_recording_oracle(LevelId(level.level_id));
            runtime.settle_recording_oracle_warp_live_window(LevelId(level.level_id));
            let frame = runtime.trace_frame();
            let native_offset = frame.rng_offset.expect("native trace exposes rng_offset");
            let native_ring_sha1 = frame
                .rng_ring
                .as_ref()
                .map(|ring| sha1_hex(ring.as_bytes()))
                .expect("native trace exposes the rng ring");
            let mut level_diffs = Vec::new();
            if native_offset != frame0.rng_offset {
                level_diffs.push(format!(
                    "rng_offset native {:#06x} vs oracle {:#06x}",
                    native_offset, frame0.rng_offset
                ));
            }
            if native_ring_sha1 != frame0.rng_ring_sha1 {
                level_diffs.push("rng_ring content differs".to_owned());
            }
            if frame.number_objects != frame0.number_objects {
                level_diffs.push(format!(
                    "number_objects native {} vs oracle {}",
                    frame.number_objects, frame0.number_objects
                ));
            }
            if !level_diffs.is_empty() {
                mismatches.push(format!(
                    "level {:#04x}: {}",
                    level.level_id,
                    level_diffs.join("; ")
                ));
            }
        }
        mismatches
    }

    /// #442 S3c gate on the probe levels: the settled-warp shell reproduces
    /// the oracle's measured frame-0 RNG (offset + full 55-byte ring) from
    /// `oracle-settle-shell.yaml` (census v2, run 1786640409) on all five
    /// probe levels. The only allowed residue is the #422 UFO beam-child
    /// undercount on 0x30/0x48 (one missing beam child at frame 0); any RNG
    /// divergence or a new count residue is a timing-model regression.
    #[test]
    fn gameplay_regression_settled_warp_frame0_witnesses_probe_levels() {
        let mismatches = settled_warp_frame0_witness_mismatches(&[0x30, 0x48, 0x50, 0x60, 0x61]);
        let unexplained: Vec<&String> = mismatches
            .iter()
            .filter(|mismatch| {
                !(mismatch.starts_with("level 0x30: number_objects native 5 vs oracle 6")
                    || mismatch.starts_with("level 0x48: number_objects native 13 vs oracle 14"))
            })
            .collect();
        assert!(
            unexplained.is_empty(),
            "settled-warp shell diverges from measured frame-0 witnesses beyond the \
             documented #422 beam-child residue:\n{unexplained:#?}"
        );
    }

    /// Full-coverage variant of the witness gate: every level with a measured
    /// load completion except the non-comparable leftover warps 0x38/0x41/0x47
    /// (#425/#433: the oracle plays garbage RAM there, so their witnesses
    /// compare nothing real). Slow (65 native boots); run by hand during #442
    /// model sessions, not in the per-commit gate. Residues as of the boundary
    /// swap are enemy-content divergences owned by open rows (see archive
    /// #442, 2026-08-14).
    #[test]
    #[ignore = "slow full-coverage witness sweep; run explicitly during #442 sessions"]
    fn settled_warp_frame0_witnesses_all_levels() {
        let all: Vec<u8> = (0..=0x68u8)
            .filter(|level| !matches!(level, 0x38 | 0x41 | 0x47))
            .collect();
        let mismatches = settled_warp_frame0_witness_mismatches(&all);
        assert!(
            mismatches.is_empty(),
            "settled-warp shell diverges from measured frame-0 witnesses:\n{}",
            mismatches.join("\n")
        );
    }

    /// ASM Mode_Level's live shell advances the murder wall every frame
    /// (loc_7316/loc_7396 via the sub_7196-allocated wall worker), including
    /// the hidden debug-warp shell frames behind the title window. Oracle
    /// sweep run 1786644354 pins Level 0x08's wall at 234240 on the
    /// level_settle checkpoint; the sweep's sampling sits one step after the
    /// 60th scenario frame, so this harness steps 61. Without hidden-shell
    /// wall ticks native reaches only the visible window's ticks from the
    /// -$30px seed (23px short).
    #[test]
    fn gameplay_regression_murder_wall_ages_through_hidden_shell_loc_7396() {
        use wildside_core::{FrameInput, LevelId, NativeRuntime};
        let Some(manifest) = cached_generated_manifest() else {
            return;
        };
        let mut runtime = NativeRuntime::with_generated_level_layout(manifest);
        runtime.set_recorded_title_card_handoff_delay(true);
        runtime.boot_to_level_for_recording_oracle(LevelId(0x08));
        runtime.settle_recording_oracle_warp_live_window(LevelId(0x08));
        let mut frame = runtime.trace_frame();
        for _ in 0..61 {
            frame = runtime.step_frame(FrameInput::default());
        }
        assert_eq!(
            frame.murder_wall_x,
            Some(234240),
            "level 0x08 murder wall at the sweep's level_settle sampling must match oracle run 1786644354"
        );
    }

    /// #419/#457: the 0x60 Goat li=3 must survive its shell journey — walk
    /// the y=192 floor at full cadence, fall off the ledge (Lion.asm
    /// loc_3E3B2 walkers have no cliff turn), land through loc_3E732 facing
    /// Right, and park against the wall via the grounded loc_3E406 hold
    /// (`$42(a3)` re-armed to $FF88, the crossing step never committed) —
    /// reaching frame 0 alive at the oracle's exact x=0x02478000, fraction
    /// included (`$42`=$FF88 constant from frame 0 in the oracle probe). The
    /// old spurious cliff-turn oscillation halved its walk speed and landed
    /// it in the hurt tile at x=588, dead.
    #[test]
    fn gameplay_regression_goat_survives_shell_ledge_fall_loc_3e3b2() {
        use wildside_core::{LevelId, NativeRuntime};
        let Some(manifest) = cached_generated_manifest() else {
            return;
        };
        let mut runtime = NativeRuntime::with_generated_level_layout(manifest);
        runtime.set_recorded_title_card_handoff_delay(true);
        runtime.boot_to_level_for_recording_oracle(LevelId(0x60));
        runtime.settle_recording_oracle_warp_live_window(LevelId(0x60));
        let frame = runtime.trace_frame();
        let goat3 = frame
            .active_objects
            .iter()
            .find(|object| object.layout_index == Some(3))
            .expect("Goat layout #3 must be in the frame-0 roster");
        assert_eq!(
            goat3.hit_points,
            Some(4),
            "Goat li=3 must reach frame 0 alive (oracle hp 4)"
        );
        // ASM Lion.asm loc_3E406 (#457): the grounded wall hold never commits
        // the crossing step, so the pre-boundary fraction survives — 583.5
        // exactly, not a 583.0 pixel snap.
        assert_eq!(
            goat3.x_pos, 0x0247_8000,
            "Goat li=3 must hold at the oracle's exact wall-adjacent x"
        );
        // ASM Manage_EnemyLoading loc_3691E (#419): a freshly allocated enemy
        // runs its family Init on the NEXT object pass (the init ends in the
        // walker hibernate), so the first walk step lands one pass after the
        // roster appearance. Oracle run 1786640409: Goat li=1 appears at
        // spawn x=280 on title-wait row 63, steps first on the following
        // tick, and its LAST settle row (target_live 59 — the state scenario
        // frame 0 starts from, this harness's surface) reads 0x01C78000.
        let goat1 = frame
            .active_objects
            .iter()
            .find(|object| object.layout_index == Some(1))
            .expect("Goat layout #1 must be in the frame-0 roster");
        assert_eq!(
            goat1.x_pos, 0x01C7_8000,
            "Goat li=1 must reach the pre-frame-0 surface without the extra \
             spawn-frame walk step"
        );
    }

    /// #420: the 0x55 Driller shell journey. Oracle settle trace (census run
    /// 1786662123): li=3 spawns at (120,175), walks left at 1px/frame while
    /// its left drill child (sub_3BF72 loc_3BFD4 re-pin at body-$18) eats the
    /// six `$E001` prize blocks of row 10 right-to-left (Driller.asm
    /// loc_3BE78 consuming the child's colid_leftwall each 16 body pixels);
    /// Kid's floor vanishes, he bounces on the spiky drill child, and li=3
    /// parks at x=12 in the loc_3BF4E retract after the live-51 body stomp.
    /// li=1 (312,95) terrain-dies at spawn; li=2 patrols 364..420.
    ///
    /// Native currently reproduces the whole chain one Kid-stomp phase frame
    /// late: li=3 parks at 13 (oracle 12), li=2 reads 403 (oracle 405, two
    /// 1px wall-turn debts), Kid bobs at 0x908000 (oracle 0x8FC000). The
    /// ROM's `$2C` stomp is latched by the PREVIOUS frame's
    /// GfxObjects_CollisionKid pass and consumed by loc_3BE78 BEFORE the
    /// walk step; native classifies the stomp inside the same frame's Kid
    /// pass, after the walker committed its step. Re-derive by moving the
    /// native stomp latch to the frame boundary (kid.asm:3378 CollisionKid
    /// order) and re-measuring the live-33/51 contact rows against census
    /// run 1786662123 before re-enabling.
    #[test]
    #[ignore = "#420 residue: Kid-stomp latch runs a phase frame late; see doc comment"]
    fn gameplay_regression_driller_shell_drills_kid_floor_loc_3be78_level_0x55() {
        use wildside_core::{LevelId, NativeRuntime};
        let Some(manifest) = cached_generated_manifest() else {
            return;
        };
        let mut runtime = NativeRuntime::with_generated_level_layout(manifest);
        runtime.set_recorded_title_card_handoff_delay(true);
        runtime.boot_to_level_for_recording_oracle(LevelId(0x55));
        runtime.settle_recording_oracle_warp_live_window(LevelId(0x55));
        let frame = runtime.trace_frame();
        let li3 = frame
            .active_objects
            .iter()
            .find(|object| object.layout_index == Some(3))
            .expect("Driller layout #3 must be in the frame-0 roster");
        assert_eq!(
            li3.x_pos, 0x000C_0000,
            "oracle live-59: li=3 sits at x=12 in the retract hibernate after \
             drilling its corridor and being stomped at live 51"
        );
        assert_eq!(
            li3.y_pos, 0x00B0_0000,
            "li=3 walks the y=176 row for the whole shell"
        );
        assert_eq!(
            li3.hit_points,
            Some(2),
            "ASM Driller.asm loc_3BF02: exactly ONE colid_kidabove stomp \
             decrements `$44(a3)` from 3 before frame 0"
        );
        let li2 = frame
            .active_objects
            .iter()
            .find(|object| object.layout_index == Some(2))
            .expect("Driller layout #2 must be in the frame-0 roster");
        assert_eq!(
            li2.x_pos, 0x0195_0000,
            "oracle live-59: li=2 patrols the 364..420 ledge and reads x=405 \
             at frame 0"
        );
        assert_eq!(
            frame.kid_y_pos,
            Some(0x008F_C000),
            "oracle live-59: Kid fell through the drilled floor and is \
             bobbing on the drill-child top at frame 0"
        );
    }

    /// #420: the 0x68 Driller shell journey. Oracle settle trace (census run
    /// 1786662123): li=3/li=5 walk left at 0.5px/frame (hp 2), wall-turn at
    /// x=12 against the level edge, and read exactly x=59.5 at frame 0;
    /// li=1 spawns at (40,64), falls, lands on the y=80 row, turns at x=28,
    /// and walks right to x=72.0 — its right drill child then smashes row-4
    /// cols 6/7/8 on visible frames 2/34/66 (oracle-probe 2026-08-14).
    ///
    /// Native currently reads li=3/li=5 at 60.5 (+1px: two spawn-entry phase
    /// frames — the ROM's Enemy1A_Driller_Init spends an extra
    /// Hibernate_Object_1Frame plus the loc_3C0D2 entry hibernate before the
    /// first walk step) and li=1 at 73.5 (+1.5px: three landing-phase
    /// frames in the loc_3C288 falling walker hand-off). Re-derive both
    /// against census run 1786662123's title_wait rows (li spawn at title
    /// 60/67, first steps and landing rows) before re-enabling.
    #[test]
    #[ignore = "#420 residue: Driller spawn/landing phase frames; see doc comment"]
    fn gameplay_regression_driller_shell_walkers_survive_loc_3c0fa_level_0x68() {
        use wildside_core::{LevelId, NativeRuntime};
        let Some(manifest) = cached_generated_manifest() else {
            return;
        };
        let mut runtime = NativeRuntime::with_generated_level_layout(manifest);
        runtime.set_recorded_title_card_handoff_delay(true);
        runtime.boot_to_level_for_recording_oracle(LevelId(0x68));
        runtime.settle_recording_oracle_warp_live_window(LevelId(0x68));
        let frame = runtime.trace_frame();
        for (layout_index, expected_x, expected_y, label) in [
            (
                1usize,
                0x0048_0000u32,
                0x0050_0000u32,
                "li=1 fell to y=80 and walks right",
            ),
            (
                3,
                0x003B_8000,
                0x0080_0000,
                "li=3 wall-turned at x=12 and reads 59.5",
            ),
            (
                5,
                0x003B_8000,
                0x00A0_0000,
                "li=5 wall-turned at x=12 and reads 59.5",
            ),
        ] {
            let driller = frame
                .active_objects
                .iter()
                .find(|object| object.layout_index == Some(layout_index))
                .unwrap_or_else(|| panic!("Driller li={layout_index} must be in the roster"));
            assert_eq!(
                (driller.x_pos, driller.y_pos),
                (expected_x, expected_y),
                "oracle live-59 witness: {label}"
            );
        }
    }

    /// #442 shell-content instrumentation: `--dump-trace` must persist the
    /// helper's FULL settle rows (RNG ring, kid kinematics, object-slot dump)
    /// so per-step shell analysis reads measured data instead of re-deriving
    /// it from the collapsed census lengths.
    #[test]
    fn dump_trace_persists_full_settle_rows_losslessly() {
        let dir = tempfile::tempdir().expect("tempdir");
        let rows = vec![
            row("title_wait", 68, false, false),
            row("target_live", 0, true, false),
        ];
        let path = write_settle_trace_dump(dir.path(), &rows).expect("dump must write");
        assert_eq!(path, dir.path().join("settle-trace.json"));
        let reloaded: Vec<OracleSettleTraceRow> =
            serde_json::from_str(&fs::read_to_string(&path).expect("dump must be readable"))
                .expect("dump must parse as the settle-trace schema");
        assert_eq!(
            serde_json::to_value(&reloaded).unwrap(),
            serde_json::to_value(&rows).unwrap(),
            "dump must round-trip every captured field"
        );
    }

    fn title_row(step: usize, rng_offset: u16) -> OracleSettleTraceRow {
        let mut title = row("title_wait", step, false, false);
        title.rng_offset = rng_offset;
        title
    }

    fn stable_target_rows() -> Vec<OracleSettleTraceRow> {
        (0..60)
            .map(|step| row("target_live", step, true, false))
            .collect()
    }

    /// #442 shell content: the per-level load duration is visible as the
    /// longest frozen run of the RNG offset inside the 120-frame title wait —
    /// the loader blocks the frame loop, so no controller draws land.
    /// `load_complete_step` is the first step drawing again (0x30 measures 75,
    /// 0x60 measures 62; run 1786640409).
    #[test]
    fn rng_freeze_derives_the_per_level_load_completion() {
        let mut trace = vec![title_row(0, 0xFFF6), title_row(1, 0xFFFA)];
        trace.extend((2..=74).map(|step| title_row(step, 0xFFFE)));
        trace.extend((75..120).map(|step| title_row(step, 0xFF00 + step as u16)));
        trace.extend(stable_target_rows());
        let derived = derive_settle_census_level(0x50, &trace);
        assert!(derived.measurable, "{:?}", derived.unmeasurable_reason);
        assert_eq!(derived.rng_freeze_start_step, Some(2));
        assert_eq!(derived.load_complete_step, Some(75));
        assert_eq!(derived.title_wait_live_rows, Some(45));
    }

    /// A freeze that never resumes inside the title wait means the load did
    /// not observably complete there; the census must say so rather than
    /// invent a boundary.
    #[test]
    fn rng_freeze_without_resume_reports_no_load_completion() {
        let mut trace = vec![title_row(0, 0xFFF6)];
        trace.extend((1..120).map(|step| title_row(step, 0xFFFE)));
        trace.extend(stable_target_rows());
        let derived = derive_settle_census_level(0x50, &trace);
        assert!(derived.measurable);
        assert_eq!(derived.rng_freeze_start_step, None);
        assert_eq!(derived.load_complete_step, None);
        assert_eq!(derived.title_wait_live_rows, None);
    }

    /// Continuously drawing title-wait rows (no load freeze) carry no
    /// load-completion fields; the target_live derivation is unaffected.
    #[test]
    fn moving_rng_title_wait_carries_no_load_completion() {
        let mut trace: Vec<_> = (0..120)
            .map(|step| title_row(step, 0xFF00 + step as u16))
            .collect();
        trace.extend(stable_target_rows());
        let derived = derive_settle_census_level(0x50, &trace);
        assert!(derived.measurable);
        assert_eq!(derived.load_complete_step, None);
        assert_eq!(derived.shell_frames_to_frame0, Some(59));
    }

    /// Helper loop: 60 immediately-live rows break at step 59 without a tick,
    /// so the shell is 59 frames and native's window argument is 60.
    #[test]
    fn immediate_stability_measures_the_helper_minimum_window() {
        let trace: Vec<_> = (0..60)
            .map(|step| row("target_live", step, true, false))
            .collect();
        let derived = derive_settle_census_level(0x50, &trace);
        assert!(derived.measurable, "{:?}", derived.unmeasurable_reason);
        assert_eq!(derived.shell_frames_to_frame0, Some(59));
        assert_eq!(derived.native_target_live_frames, Some(60));
        assert_eq!(derived.first_live_step, Some(0));
        assert_eq!(derived.flap_resets, 0);
        assert_eq!(derived.paused_live_rows, 0);
        assert_eq!(derived.took_debug_skip, None);
        let frame0 = derived.frame0.expect("frame0 witness");
        assert_eq!(frame0.rng_offset, 0xFFCB);
        assert_eq!(frame0.number_objects, 7);
        assert_eq!(frame0.rng_ring_sha1, sha1_hex(&[0xAA; 55]));
    }

    /// A paused live row fires the helper's pause-recovery branch, which ticks
    /// one extra emulator frame beyond the step counter; a live->dead flap
    /// resets the stable streak. Both must count into the shell length.
    #[test]
    fn pause_recovery_and_flaps_extend_the_shell() {
        let mut trace = vec![skip_row(0x0008)];
        // Steps 0..3 live, step 4 paused (pause branch), step 5 dead (flap),
        // steps 6..65 the final stable 60-row streak.
        trace.extend((0..4).map(|step| row("target_live", step, true, false)));
        trace.push(row("target_live", 4, false, true));
        trace.push(row("target_live", 5, false, false));
        trace.extend((6..66).map(|step| row("target_live", step, true, false)));
        let derived = derive_settle_census_level(0x50, &trace);
        assert!(derived.measurable, "{:?}", derived.unmeasurable_reason);
        assert_eq!(derived.last_live_step, Some(65));
        assert_eq!(derived.paused_live_rows, 1);
        assert_eq!(derived.shell_frames_to_frame0, Some(66));
        assert_eq!(derived.native_target_live_frames, Some(67));
        assert_eq!(derived.first_live_step, Some(0));
        assert_eq!(derived.flap_resets, 1);
        assert_eq!(derived.took_debug_skip, Some(false));
    }

    /// Levels whose settle loop exhausts without a 60-frame stable streak
    /// (the 0x41 shape) must be reported unmeasurable, never defaulted.
    #[test]
    fn unstable_settle_loop_reports_unmeasurable() {
        let trace: Vec<_> = (0..900)
            .map(|step| row("target_live", step, step % 30 != 0, false))
            .collect();
        let derived = derive_settle_census_level(0x41, &trace);
        assert!(!derived.measurable);
        assert!(
            derived
                .unmeasurable_reason
                .as_deref()
                .is_some_and(|reason| reason.contains("never stabilised")),
            "{:?}",
            derived.unmeasurable_reason
        );
        assert_eq!(derived.shell_frames_to_frame0, None);
    }

    #[test]
    fn missing_target_rows_report_unmeasurable() {
        let trace = vec![skip_row(0x0008), row("title_wait", 0, false, false)];
        let derived = derive_settle_census_level(0x30, &trace);
        assert!(!derived.measurable);
        assert!(
            derived
                .unmeasurable_reason
                .as_deref()
                .is_some_and(|reason| reason.contains("shell missing")),
        );
    }

    /// The level-0 bootstrap has no skip press; its rows use the
    /// `title_target_live` phase and `took_debug_skip` stays unknown.
    #[test]
    fn level_zero_uses_the_title_phase_without_skip_evidence() {
        let mut trace: Vec<_> = (0..72)
            .map(|step| {
                let mut title_row = row("title_target_live", step, step >= 12, false);
                title_row.current_level_id = if step >= 12 { 0x0000 } else { 0x0008 };
                title_row.kid_x_pos = if step >= 12 { 0x0080_0000 } else { 0 };
                title_row.number_objects = if step >= 12 { 7 } else { 0 };
                title_row
            })
            .collect();
        for title_row in &mut trace {
            if title_row.step < 12 {
                title_row.game_mode = 0x0008;
            }
        }
        let derived = derive_settle_census_level(0x00, &trace);
        assert!(derived.measurable, "{:?}", derived.unmeasurable_reason);
        assert_eq!(derived.phase.as_deref(), Some("title_target_live"));
        assert_eq!(derived.first_live_step, Some(12));
        assert_eq!(derived.shell_frames_to_frame0, Some(71));
        assert_eq!(derived.took_debug_skip, None);
    }

    /// A first skip press that fails to flip Game_Mode to $0008 marks the
    /// level as having taken the retry/fallback path.
    #[test]
    fn failed_first_skip_press_is_recorded() {
        let mut trace = vec![skip_row(0x000C)];
        trace.extend((0..60).map(|step| row("target_live", step, true, false)));
        let derived = derive_settle_census_level(0x50, &trace);
        assert_eq!(derived.took_debug_skip, Some(true));
    }
}
