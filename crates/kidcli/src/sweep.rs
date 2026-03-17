use super::*;
use cache::{OracleCacheKey, load_cached_oracle, store_oracle_cache};
use diff::{annotate_diff, frame_mae, generate_diff_composite};
use native_player::render_gameplay_frame_evidence_for_level;
use oracle::run_jgenesis_capture_dynamic;
use scenarios::{DynamicCheckpoint, DynamicScenario, idle_level_scenario};
use sha1::{Digest, Sha1};

/// Advisory fields — diverge due to architectural differences, not native bugs:
/// - number_objects: native doesn't model system objects (palette rotation,
///   weather, lava geysers, fade-in) as separate object slots
/// - number_hitpoints: oracle is accidentally paused (Start press carry-over
///   from title card transition), so the oracle player doesn't take damage
///   from enemies/murder wall during idle while the native does
const ADVISORY_FIELDS: &[&str] = &["number_objects", "number_hitpoints"];

/// Result tier controls how much data is collected per level.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, clap::ValueEnum)]
#[serde(rename_all = "snake_case")]
pub(crate) enum SweepTier {
    /// State-only: NativeTraceFrame comparison, no frame capture.
    State,
    /// State + checkpoint frame capture + MAE comparison.
    Frames,
    /// Full diagnosis: multi-frame capture + drift detection.
    Full,
}

/// Per-level sweep result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct LevelSweepResult {
    pub(crate) level_id: u8,
    pub(crate) slug: String,
    pub(crate) tier: SweepTier,
    pub(crate) passed: bool,
    pub(crate) state_diffs: Vec<StateDiff>,
    pub(crate) mae: Option<f32>,
    pub(crate) diff_images: Vec<String>,
    pub(crate) error: Option<String>,
}

/// Field-level state divergence.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct StateDiff {
    pub(crate) checkpoint: String,
    pub(crate) field: String,
    pub(crate) native_value: String,
    pub(crate) oracle_value: String,
}

/// Top-level sweep report.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct SweepReport {
    pub(crate) timestamp: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) kid_core_hash: Option<String>,
    pub(crate) levels: Vec<LevelSweepResult>,
    pub(crate) passed: usize,
    pub(crate) failed: usize,
    pub(crate) errors: usize,
}

/// Options controlling sweep behaviour beyond tier and level selection.
#[derive(Debug, Clone)]
pub(crate) struct SweepOptions {
    /// Only populate the oracle cache — skip native runs and comparison.
    pub(crate) warm_cache_only: bool,
    /// Stop the sweep after this many failures (0 = no limit).
    pub(crate) max_failures: usize,
}

impl Default for SweepOptions {
    fn default() -> Self {
        Self {
            warm_cache_only: false,
            max_failures: 0,
        }
    }
}

/// Kid Chameleon level IDs are BCD-encoded: 0x00-0x09, 0x10-0x19, …, 0x60-0x68.
/// Returns true if `id` is a valid BCD level ID.
fn is_valid_level_id(id: u8) -> bool {
    let lo = id & 0x0F;
    let hi = id >> 4;
    lo <= 9 && hi <= 6 && id <= 0x68
}

/// Parse a level range spec like "0-104", "0,5,10", or "0-10".
/// Invalid BCD level IDs (0x0A-0x0F, 0x1A-0x1F, etc.) are filtered out.
pub(crate) fn parse_level_range(spec: &str) -> Result<Vec<u8>> {
    let mut levels = Vec::new();
    for part in spec.split(',') {
        let part = part.trim();
        if let Some((start, end)) = part.split_once('-') {
            let start: u8 = start.trim().parse().context("invalid level range start")?;
            let end: u8 = end.trim().parse().context("invalid level range end")?;
            if end < start {
                bail!("level range end {end} < start {start}");
            }
            levels.extend((start..=end).filter(|&id| is_valid_level_id(id)));
        } else {
            let level: u8 = part.parse().context("invalid level id")?;
            if is_valid_level_id(level) {
                levels.push(level);
            }
        }
    }
    levels.sort_unstable();
    levels.dedup();
    Ok(levels)
}

/// Run a sweep across the specified levels.
pub(crate) fn run_sweep(
    repo_root: &Path,
    rom_path: &Path,
    manifest: &AssetManifest,
    levels: &[u8],
    tier: SweepTier,
) -> Result<SweepReport> {
    run_sweep_with_options(repo_root, rom_path, manifest, levels, tier, &SweepOptions::default())
}

/// Run a sweep with additional options (warm-cache-only, fast-fail).
pub(crate) fn run_sweep_with_options(
    repo_root: &Path,
    rom_path: &Path,
    manifest: &AssetManifest,
    levels: &[u8],
    tier: SweepTier,
    options: &SweepOptions,
) -> Result<SweepReport> {
    let rom_bytes = fs::read(rom_path)
        .with_context(|| format!("failed to read ROM from {}", rom_path.display()))?;

    let env = ensure_oracles(repo_root)?;

    // Warm-cache-only: populate oracle cache in parallel, skip native runs.
    if options.warm_cache_only {
        return warm_oracle_cache(&rom_bytes, rom_path, repo_root, &env, levels);
    }

    let timestamp = chrono_timestamp();
    let run_dir = repo_root.join(".cache/sweep-runs").join(&timestamp);
    fs::create_dir_all(&run_dir)
        .with_context(|| format!("failed to create {}", run_dir.display()))?;

    // Symlink latest → this run
    let latest_link = repo_root.join(".cache/sweep-runs/latest");
    let _ = fs::remove_file(&latest_link);
    #[cfg(unix)]
    std::os::unix::fs::symlink(&run_dir, &latest_link).ok();

    let mut results = Vec::with_capacity(levels.len());
    let mut failure_count: usize = 0;

    for &level_id in levels {
        println!("sweep level {level_id:#04x}...");
        let result = sweep_single_level(
            repo_root, rom_path, &rom_bytes, manifest, &env, &run_dir, level_id, tier,
        );
        match result {
            Ok(r) => {
                let status = if r.passed { "PASS" } else { "FAIL" };
                let mae_str = r
                    .mae
                    .map(|v| format!(" MAE={v:.1}"))
                    .unwrap_or_default();
                let blocking_diffs = r.state_diffs.iter().filter(|d| !ADVISORY_FIELDS.contains(&d.field.as_str())).count();
                let advisory_diffs = r.state_diffs.len() - blocking_diffs;
                let diff_count = match (blocking_diffs, advisory_diffs) {
                    (0, 0) => String::new(),
                    (0, a) => format!(" ({a} advisory diffs)"),
                    (b, 0) => format!(" ({b} state diffs)"),
                    (b, a) => format!(" ({b} state diffs, {a} advisory)"),
                };
                println!("  {status}{mae_str}{diff_count}");
                if !r.passed {
                    failure_count += 1;
                }
                results.push(r);
            }
            Err(e) => {
                println!("  ERROR: {e:#}");
                failure_count += 1;
                results.push(LevelSweepResult {
                    level_id,
                    slug: format!("idle_level_{level_id:02x}"),
                    tier,
                    passed: false,
                    state_diffs: Vec::new(),
                    mae: None,
                    diff_images: Vec::new(),
                    error: Some(format!("{e:#}")),
                });
            }
        }

        // Fast-fail: stop after max_failures
        if options.max_failures > 0 && failure_count >= options.max_failures {
            println!(
                "\nfast-fail: stopping after {failure_count} failure(s) (--max-failures {})",
                options.max_failures
            );
            break;
        }
    }

    let passed = results.iter().filter(|r| r.passed).count();
    let failed = results.iter().filter(|r| !r.passed && r.error.is_none()).count();
    let errors = results.iter().filter(|r| r.error.is_some()).count();

    let source_hash = kid_core_source_hash(repo_root).ok();
    let report = SweepReport {
        timestamp,
        kid_core_hash: source_hash,
        levels: results,
        passed,
        failed,
        errors,
    };

    // Write report JSON
    let report_path = run_dir.join("report.json");
    fs::write(&report_path, serde_json::to_vec_pretty(&report)?)
        .with_context(|| format!("failed to write {}", report_path.display()))?;

    println!(
        "\nsweep complete: {passed} passed, {failed} failed, {errors} errors"
    );
    println!("report: {}", report_path.display());

    Ok(report)
}

fn sweep_single_level(
    repo_root: &Path,
    rom_path: &Path,
    rom_bytes: &[u8],
    manifest: &AssetManifest,
    env: &OracleEnvironment,
    run_dir: &Path,
    level_id: u8,
    requested_tier: SweepTier,
) -> Result<LevelSweepResult> {
    let scenario = idle_level_scenario(level_id, None);
    let level_dir = run_dir.join(format!("levels/{level_id:02x}"));
    fs::create_dir_all(&level_dir)
        .with_context(|| format!("failed to create {}", level_dir.display()))?;

    // Run native (with warp if specified)
    let native_trace = kid_core::run_script_with_warp(
        manifest.clone(),
        &scenario.script,
        scenario.frame_count,
        scenario.warp_to_level,
    );

    // Extract native state at checkpoints
    let native_states = extract_checkpoint_states(&native_trace, &scenario.checkpoints);

    // Run oracle (with caching)
    let cache_key = OracleCacheKey::from_scenario(rom_bytes, &scenario);
    let oracle = match load_cached_oracle(&cache_key, repo_root) {
        Some(cached) => cached,
        None => {
            // Write scenario file for oracle
            let scenario_file = level_dir.join("scenario.json");
            fs::write(&scenario_file, serde_json::to_vec_pretty(&scenario)?)
                .with_context(|| format!("failed to write {}", scenario_file.display()))?;

            let oracle_dir = level_dir.join("oracle");
            fs::create_dir_all(&oracle_dir)?;
            let oracle = run_jgenesis_capture_dynamic(env, rom_path, &scenario_file, &oracle_dir)?;
            let _ = store_oracle_cache(&cache_key, repo_root, &oracle, &oracle_dir);
            oracle
        }
    };

    // Detect broken oracle warps: if the oracle's game_mode is clearly invalid
    // (e.g. 0x4e73 instead of 0x0000-0x0014), the warp failed and oracle data
    // is garbage. Report as error rather than native failure.
    let oracle_warp_broken = oracle.states.iter().any(|s| s.game_mode > 0x0014);
    if oracle_warp_broken {
        return Ok(LevelSweepResult {
            level_id,
            slug: format!("idle_level_{level_id:02x}"),
            tier: requested_tier,
            passed: false,
            state_diffs: Vec::new(),
            mae: None,
            diff_images: Vec::new(),
            error: Some(format!(
                "oracle warp broken: game_mode={:#06x}",
                oracle.states.first().map(|s| s.game_mode).unwrap_or(0)
            )),
        });
    }

    // Compare states
    let state_diffs = compare_sweep_states(&native_states, &oracle.states);
    // Build set of checkpoint labels where the native player died during idle
    // but the oracle survived (due to being paused). Position diffs at these
    // checkpoints are a downstream consequence of the HP advisory.
    let death_gap_checkpoints: std::collections::HashSet<&str> = native_states
        .iter()
        .zip(oracle.states.iter())
        .filter(|(n, o)| n.number_hitpoints == 0 && o.number_hitpoints > 0)
        .map(|(n, _)| n.label.as_str())
        .collect();
    let state_passed = state_diffs.iter().all(|d| {
        ADVISORY_FIELDS.contains(&d.field.as_str())
            || (death_gap_checkpoints.contains(d.checkpoint.as_str())
                && matches!(
                    d.field.as_str(),
                    "kid_x_pos" | "kid_y_pos" | "camera_x_pos" | "camera_y_pos"
                ))
    });

    // Tier 1 (state-only): done unless escalating
    let effective_tier = if !state_passed && requested_tier == SweepTier::State {
        SweepTier::Frames // auto-escalate on failure
    } else {
        requested_tier
    };

    let mut mae_val = None;
    let mut diff_images = Vec::new();

    if effective_tier == SweepTier::Frames || effective_tier == SweepTier::Full {
        // Capture native frames at checkpoints and compare with oracle PNGs
        if let Some(best_mae) =
            compare_checkpoint_frames(manifest, &native_trace, &scenario, &oracle, &level_dir, level_id)?
        {
            mae_val = Some(best_mae);
        }

        // Collect diff image paths
        for entry in fs::read_dir(&level_dir)? {
            let entry = entry?;
            let name = entry.file_name().to_string_lossy().to_string();
            if name.starts_with("diff_") && name.ends_with(".png") {
                diff_images.push(entry.path().to_string_lossy().to_string());
            }
        }
    }

    let passed = state_passed && mae_val.map_or(true, |v| v < 32.0);

    Ok(LevelSweepResult {
        level_id,
        slug: scenario.slug,
        tier: effective_tier,
        passed,
        state_diffs,
        mae: mae_val,
        diff_images,
        error: None,
    })
}

/// Extract NativeTraceFrame fields at checkpoint frame indices, returning
/// OracleState-compatible structs for comparison.
fn extract_checkpoint_states(
    trace: &[NativeTraceFrame],
    checkpoints: &[DynamicCheckpoint],
) -> Vec<OracleState> {
    checkpoints
        .iter()
        .filter_map(|cp| {
            let frame = trace.get(cp.frame_index)?;
            Some(OracleState {
                label: cp.label.clone(),
                frame_index: cp.frame_index,
                game_mode: frame.game_mode_value,
                current_player: frame.current_player,
                number_hitpoints: frame.number_hitpoints,
                current_helmet_id: frame.current_helmet_id,
                current_level_id: frame.current_level.0 as u16,
                player_1_level_id: frame.player_1_level_id.0 as u16,
                camera_x_pos: frame.camera_x_pos,
                camera_y_pos: frame.camera_y_pos,
                kid_x_pos: frame.kid_x_pos.unwrap_or(0),
                kid_y_pos: frame.kid_y_pos.unwrap_or(0),
                number_objects: frame.number_objects,
                number_gfx_objects: frame.number_gfx_objects,
                active_enemies: Vec::new(),
            })
        })
        .collect()
}

/// Compare native vs oracle state at each checkpoint, returning diffs.
fn compare_sweep_states(
    native_states: &[OracleState],
    oracle_states: &[OracleState],
) -> Vec<StateDiff> {
    let mut diffs = Vec::new();

    for (native, oracle) in native_states.iter().zip(oracle_states.iter()) {
        let cp = &native.label;
        if native.game_mode != oracle.game_mode {
            diffs.push(StateDiff {
                checkpoint: cp.clone(),
                field: "game_mode".into(),
                native_value: format!("{:#06x}", native.game_mode),
                oracle_value: format!("{:#06x}", oracle.game_mode),
            });
        }
        if native.current_level_id != oracle.current_level_id {
            diffs.push(StateDiff {
                checkpoint: cp.clone(),
                field: "current_level_id".into(),
                native_value: format!("{:#06x}", native.current_level_id),
                oracle_value: format!("{:#06x}", oracle.current_level_id),
            });
        }
        // Position thresholds: 6px (0x0006_0000) accommodates the ±5px
        // PLAYER_HALF_WIDTH offset from gravity settling differences between
        // native (instant floor snap) and oracle (physics-based settling).
        let cam_x_delta = native.camera_x_pos.abs_diff(oracle.camera_x_pos);
        if cam_x_delta > 0x0006_0000 {
            diffs.push(StateDiff {
                checkpoint: cp.clone(),
                field: "camera_x_pos".into(),
                native_value: format!("{:#010x}", native.camera_x_pos),
                oracle_value: format!("{:#010x}", oracle.camera_x_pos),
            });
        }
        let cam_y_delta = native.camera_y_pos.abs_diff(oracle.camera_y_pos);
        if cam_y_delta > 0x0006_0000 {
            diffs.push(StateDiff {
                checkpoint: cp.clone(),
                field: "camera_y_pos".into(),
                native_value: format!("{:#010x}", native.camera_y_pos),
                oracle_value: format!("{:#010x}", oracle.camera_y_pos),
            });
        }
        if native.number_objects != oracle.number_objects {
            diffs.push(StateDiff {
                checkpoint: cp.clone(),
                field: "number_objects".into(),
                native_value: format!("{}", native.number_objects),
                oracle_value: format!("{}", oracle.number_objects),
            });
        }
        if native.number_hitpoints != oracle.number_hitpoints {
            diffs.push(StateDiff {
                checkpoint: cp.clone(),
                field: "number_hitpoints".into(),
                native_value: format!("{}", native.number_hitpoints),
                oracle_value: format!("{}", oracle.number_hitpoints),
            });
        }
        if native.kid_x_pos.abs_diff(oracle.kid_x_pos) > 0x0006_0000 {
            diffs.push(StateDiff {
                checkpoint: cp.clone(),
                field: "kid_x_pos".into(),
                native_value: format!("{:#010x}", native.kid_x_pos),
                oracle_value: format!("{:#010x}", oracle.kid_x_pos),
            });
        }
        if native.kid_y_pos.abs_diff(oracle.kid_y_pos) > 0x0006_0000 {
            diffs.push(StateDiff {
                checkpoint: cp.clone(),
                field: "kid_y_pos".into(),
                native_value: format!("{:#010x}", native.kid_y_pos),
                oracle_value: format!("{:#010x}", oracle.kid_y_pos),
            });
        }
    }

    diffs
}

/// Capture native frames at checkpoint indices and compare against oracle PNGs.
/// Returns the worst (maximum) MAE across checkpoints.
fn compare_checkpoint_frames(
    manifest: &AssetManifest,
    native_trace: &[NativeTraceFrame],
    scenario: &DynamicScenario,
    oracle: &OracleManifest,
    level_dir: &Path,
    level_id: u8,
) -> Result<Option<f32>> {
    let mut worst_mae: Option<f32> = None;

    // Convert DynamicCheckpoints to GameplayCheckpoints for the renderer.
    let gameplay_checkpoints: Vec<scenarios::GameplayCheckpoint> = scenario
        .checkpoints
        .iter()
        .map(|cp| scenarios::GameplayCheckpoint {
            label: Box::leak(cp.label.clone().into_boxed_str()),
            frame_index: cp.frame_index,
        })
        .collect();

    // Use the existing headless renderer to produce real native frames.
    // Falls back to black frames if level visuals aren't available.
    let native_frames =
        render_gameplay_frame_evidence_for_level(manifest, native_trace, &gameplay_checkpoints, level_id)
            .unwrap_or_else(|_| {
                gameplay_checkpoints
                    .iter()
                    .map(|cp| {
                        (cp.label.to_string(), cp.frame_index, 320, 224, vec![0u8; 320 * 224 * 3])
                    })
                    .collect()
            });

    for (native_evidence, oracle_frame) in native_frames.iter().zip(oracle.frames.iter()) {
        let (_label, _frame_idx, w, h, native_rgb) = native_evidence;
        let oracle_rgb = load_png_rgb(&oracle_frame.png_path)?;
        let mae = frame_mae(native_rgb, &oracle_rgb);

        worst_mae = Some(worst_mae.map_or(mae, |w: f32| w.max(mae)));

        // Generate diff composite
        let (cw, ch, mut composite) = generate_diff_composite(
            native_rgb,
            &oracle_rgb,
            *w as u32,
            *h as u32,
        );
        annotate_diff(
            &mut composite,
            cw,
            level_id,
            &oracle_frame.label,
            mae,
        );

        let diff_path = level_dir.join(format!("diff_{}.png", oracle_frame.label));
        image::save_buffer_with_format(
            &diff_path,
            &composite,
            cw,
            ch,
            image::ColorType::Rgb8,
            image::ImageFormat::Png,
        )
        .with_context(|| format!("failed to write {}", diff_path.display()))?;

        // Also save native frame for direct comparison
        let native_path = level_dir.join(format!("native_{}.png", oracle_frame.label));
        image::save_buffer_with_format(
            &native_path,
            native_rgb,
            *w as u32,
            *h as u32,
            image::ColorType::Rgb8,
            image::ImageFormat::Png,
        )
        .with_context(|| format!("failed to write {}", native_path.display()))?;
    }

    Ok(worst_mae)
}

/// Deep diagnosis of a single level: run with Frames tier, print findings.
pub(crate) fn diagnose_level(
    repo_root: &Path,
    rom_path: &Path,
    manifest: &AssetManifest,
    level_id: u8,
) -> Result<()> {
    let report = run_sweep(repo_root, rom_path, manifest, &[level_id], SweepTier::Frames)?;
    let result = report
        .levels
        .first()
        .ok_or_else(|| anyhow!("no result for level {level_id:#04x}"))?;

    println!("\nLevel {level_id:#04x} diagnosis:");
    if result.passed {
        println!("  PASSED — no divergence detected");
        return Ok(());
    }

    if let Some(ref err) = result.error {
        println!("  ERROR: {err}");
        return Ok(());
    }

    for diff in &result.state_diffs {
        println!(
            "  State divergence at {}: {} native={} oracle={}",
            diff.checkpoint, diff.field, diff.native_value, diff.oracle_value
        );
    }

    if let Some(mae) = result.mae {
        let pass_fail = if mae < 32.0 { "PASS" } else { "FAIL" };
        println!("  Frame MAE: {mae:.1} ({pass_fail}, threshold: 32.0)");
    }

    for path in &result.diff_images {
        println!("  Diff image: {path}");
    }

    // Suggest investigation areas based on which fields diverged
    let diverged_fields: Vec<&str> = result.state_diffs.iter().map(|d| d.field.as_str()).collect();
    println!("\n  Suggested investigation:");
    if diverged_fields.contains(&"number_objects") {
        println!("    - Enemy spawning: check enemy layout for level {level_id:#04x}");
        println!("      → code/enemy/*.asm, kidcli inspect-enemy");
    }
    if diverged_fields.contains(&"camera_x_pos") || diverged_fields.contains(&"camera_y_pos") {
        println!("    - Camera position: check camera follow/scroll logic");
        println!("      → camera routines in kid.asm");
    }
    if diverged_fields.contains(&"kid_x_pos") || diverged_fields.contains(&"kid_y_pos") {
        println!("    - Player position: check physics/movement code");
        println!("      → kid-core physics.rs, terrain.rs");
    }
    if diverged_fields.contains(&"game_mode") {
        println!("    - Game mode: check state machine transitions");
        println!("      → NativeRuntime state machine in kid-core");
    }
    if diverged_fields.contains(&"current_level_id") {
        println!("    - Level ID: check level transition / warp logic");
    }

    Ok(())
}

/// Agent-driven parity loop: sweep → find worst → diagnose → report.
pub(crate) fn run_parity_loop(
    repo_root: &Path,
    rom_path: &Path,
    manifest: &AssetManifest,
    max_cycles: u32,
) -> Result<()> {
    let all_levels: Vec<u8> = (0..=104).collect();
    let source_hash = kid_core_source_hash(repo_root).unwrap_or_else(|_| "unknown".into());

    for cycle in 1..=max_cycles {
        println!("=== parity loop cycle {cycle}/{max_cycles} ===");
        println!("kid-core source hash: {source_hash}\n");

        // Tier 1: state-only sweep
        let report = run_sweep(
            repo_root,
            rom_path,
            manifest,
            &all_levels,
            SweepTier::State,
        )?;

        if report.failed == 0 && report.errors == 0 {
            println!("\nAll {0} levels passed!", report.passed);
            return Ok(());
        }

        // Find worst failure (most state diffs)
        let worst = report
            .levels
            .iter()
            .filter(|r| !r.passed)
            .max_by_key(|r| r.state_diffs.len());

        if let Some(worst) = worst {
            println!(
                "\nWorst failure: level {:#04x} ({} state diffs)",
                worst.level_id,
                worst.state_diffs.len()
            );
            diagnose_level(repo_root, rom_path, manifest, worst.level_id)?;
        }

        println!(
            "\nCycle {cycle} summary: {} passed, {} failed, {} errors",
            report.passed, report.failed, report.errors
        );
    }

    Ok(())
}

/// Populate oracle cache for all requested levels in parallel, skip native comparison.
fn warm_oracle_cache(
    rom_bytes: &[u8],
    rom_path: &Path,
    repo_root: &Path,
    env: &OracleEnvironment,
    levels: &[u8],
) -> Result<SweepReport> {
    let timestamp = chrono_timestamp();
    let mut cached = 0usize;
    let mut ran = 0usize;
    let mut errored = 0usize;

    // Group levels into batches for parallel oracle runs.
    // Oracle processes are I/O-heavy (emulator), so parallelism helps.
    let batch_size = std::thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(4)
        .min(8); // cap at 8 to avoid thrashing

    for chunk in levels.chunks(batch_size) {
        let handles: Vec<_> = chunk
            .iter()
            .map(|&level_id| {
                let scenario = idle_level_scenario(level_id, None);
                let cache_key = OracleCacheKey::from_scenario(rom_bytes, &scenario);

                // Already cached?
                if load_cached_oracle(&cache_key, repo_root).is_some() {
                    return None;
                }

                let rom = rom_path.to_owned();
                let root = repo_root.to_owned();
                let env = env.clone();

                Some((level_id, scenario, cache_key, rom, root, env))
            })
            .collect();

        let mut threads = Vec::new();
        for item in handles {
            match item {
                None => cached += 1,
                Some((level_id, scenario, cache_key, rom, root, env)) => {
                    threads.push(thread::spawn(move || {
                        let tmp = root.join(format!(".cache/warm-cache-tmp/{level_id:02x}"));
                        fs::create_dir_all(&tmp).ok();

                        let scenario_file = tmp.join("scenario.json");
                        fs::write(
                            &scenario_file,
                            serde_json::to_vec_pretty(&scenario).unwrap_or_default(),
                        )
                        .ok();

                        let oracle_dir = tmp.join("oracle");
                        fs::create_dir_all(&oracle_dir).ok();

                        match run_jgenesis_capture_dynamic(&env, &rom, &scenario_file, &oracle_dir)
                        {
                            Ok(oracle) => {
                                let _ = store_oracle_cache(&cache_key, &root, &oracle, &oracle_dir);
                                println!("  warm {level_id:#04x}: cached");
                                Ok(())
                            }
                            Err(e) => {
                                println!("  warm {level_id:#04x}: ERROR {e:#}");
                                Err(e)
                            }
                        }
                    }));
                }
            }
        }

        for handle in threads {
            match handle.join() {
                Ok(Ok(())) => ran += 1,
                _ => errored += 1,
            }
        }
    }

    println!(
        "\nwarm-cache complete: {cached} already cached, {ran} newly cached, {errored} errors"
    );

    Ok(SweepReport {
        timestamp,
        kid_core_hash: None,
        levels: Vec::new(),
        passed: cached + ran,
        failed: 0,
        errors: errored,
    })
}

/// Compute a SHA-1 hash of kid-core source files.
/// Used for incremental native: if the hash hasn't changed since the last sweep,
/// native re-runs can be skipped (the game logic is deterministic).
pub(crate) fn kid_core_source_hash(repo_root: &Path) -> Result<String> {
    let src_dir = repo_root.join("crates/kid-core/src");
    let mut paths: Vec<PathBuf> = Vec::new();
    collect_rs_files(&src_dir, &mut paths)?;
    paths.sort();

    let mut hasher = Sha1::new();
    for path in &paths {
        hasher.update(path.strip_prefix(repo_root).unwrap_or(path).to_string_lossy().as_bytes());
        hasher.update(b"\0");
        hasher.update(&fs::read(path).with_context(|| format!("reading {}", path.display()))?);
        hasher.update(b"\0");
    }
    Ok(format!("{:x}", hasher.finalize()))
}

fn collect_rs_files(dir: &Path, out: &mut Vec<PathBuf>) -> Result<()> {
    if !dir.is_dir() {
        return Ok(());
    }
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            collect_rs_files(&path, out)?;
        } else if path.extension().is_some_and(|e| e == "rs") {
            out.push(path);
        }
    }
    Ok(())
}

fn chrono_timestamp() -> String {
    use std::time::SystemTime;
    let since_epoch = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap_or_default();
    format!("{}", since_epoch.as_secs())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_level_range_single() {
        assert_eq!(parse_level_range("5").unwrap(), vec![5]);
    }

    #[test]
    fn parse_level_range_range() {
        assert_eq!(parse_level_range("0-4").unwrap(), vec![0, 1, 2, 3, 4]);
    }

    #[test]
    fn parse_level_range_csv() {
        assert_eq!(parse_level_range("0,5,9").unwrap(), vec![0, 5, 9]);
    }

    #[test]
    fn parse_level_range_mixed() {
        // 0x10-0x12 are valid BCD; 0x0A-0x0F are filtered out from the range
        assert_eq!(
            parse_level_range("0-2,5,16-18").unwrap(),
            vec![0, 1, 2, 5, 16, 17, 18]
        );
    }

    #[test]
    fn parse_level_range_filters_invalid_bcd() {
        // Decimal 10-15 = 0x0A-0x0F, all invalid BCD
        assert_eq!(parse_level_range("8-18").unwrap(), vec![8, 9, 16, 17, 18]);
    }

    #[test]
    fn parse_level_range_deduplicates() {
        assert_eq!(parse_level_range("1,1,2,2").unwrap(), vec![1, 2]);
    }

    #[test]
    fn parse_level_range_reversed_errors() {
        assert!(parse_level_range("5-3").is_err());
    }

    #[test]
    fn compare_sweep_states_detects_game_mode_diff() {
        let native = vec![OracleState {
            label: "settle".into(),
            frame_index: 780,
            game_mode: 0x0010,
            current_player: 0,
            number_hitpoints: 3,
            current_helmet_id: 0,
            current_level_id: 0,
            player_1_level_id: 0,
            camera_x_pos: 0,
            camera_y_pos: 0,
            kid_x_pos: 0,
            kid_y_pos: 0,
            number_objects: 0,
            number_gfx_objects: 0,
            active_enemies: Vec::new(),
        }];
        let mut oracle = native.clone();
        oracle[0].game_mode = 0x0020;

        let diffs = compare_sweep_states(&native, &oracle);
        assert_eq!(diffs.len(), 1);
        assert_eq!(diffs[0].field, "game_mode");
    }

    #[test]
    fn compare_sweep_states_tolerates_small_camera_drift() {
        let native = vec![OracleState {
            label: "settle".into(),
            frame_index: 780,
            game_mode: 0x0010,
            current_player: 0,
            number_hitpoints: 3,
            current_helmet_id: 0,
            current_level_id: 0,
            player_1_level_id: 0,
            camera_x_pos: 0x0100_0000,
            camera_y_pos: 0,
            kid_x_pos: 0,
            kid_y_pos: 0,
            number_objects: 5,
            number_gfx_objects: 3,
            active_enemies: Vec::new(),
        }];
        let mut oracle = native.clone();
        // Small drift within tolerance (0x0004_0000)
        oracle[0].camera_x_pos = 0x0100_2000;

        let diffs = compare_sweep_states(&native, &oracle);
        assert!(diffs.is_empty());
    }

    #[test]
    fn sweep_options_default_is_no_limits() {
        let opts = SweepOptions::default();
        assert!(!opts.warm_cache_only);
        assert_eq!(opts.max_failures, 0);
    }

    #[test]
    fn kid_core_source_hash_is_deterministic() {
        // Both calls with the same source should produce the same hash.
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .parent()
            .unwrap()
            .to_owned();
        let h1 = kid_core_source_hash(&root).unwrap();
        let h2 = kid_core_source_hash(&root).unwrap();
        assert_eq!(h1, h2);
        assert_eq!(h1.len(), 40); // SHA-1 hex
    }

    #[test]
    fn compare_sweep_states_detects_large_camera_drift() {
        let native = vec![OracleState {
            label: "settle".into(),
            frame_index: 780,
            game_mode: 0x0010,
            current_player: 0,
            number_hitpoints: 3,
            current_helmet_id: 0,
            current_level_id: 0,
            player_1_level_id: 0,
            camera_x_pos: 0x0100_0000,
            camera_y_pos: 0,
            kid_x_pos: 0,
            kid_y_pos: 0,
            number_objects: 5,
            number_gfx_objects: 3,
            active_enemies: Vec::new(),
        }];
        let mut oracle = native.clone();
        oracle[0].camera_x_pos = 0x0110_0000;

        let diffs = compare_sweep_states(&native, &oracle);
        assert_eq!(diffs.len(), 1);
        assert_eq!(diffs[0].field, "camera_x_pos");
    }
}
