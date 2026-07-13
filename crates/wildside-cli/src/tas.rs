use anyhow::{Context, Result, bail};
use sha1::{Digest, Sha1};
use std::io::Read;
use std::path::Path;

use crate::oracle::OracleState;
use crate::scenarios::{
    DynamicCheckpoint, DynamicScenario, NativeDynamicRunOptions, run_native_dynamic_scenario,
};

/// Parsed BK2 (BizHawk) TAS recording.
#[derive(Debug)]
pub struct Bk2Import {
    pub slug: String,
    pub total_frames: usize,
    /// Per-frame button masks using the same encoding as `FrameInput::from_mask`:
    /// bit 0=Up, 1=Down, 2=Left, 3=Right, 4=B, 5=C, 6=A, 7=Start
    pub script: Vec<u8>,
}

/// BizHawk Genesis button names in the order they appear in the LogKey,
/// mapped to our mask bit positions.
const GENESIS_BUTTON_MAP: &[(&str, u8)] = &[
    ("Up", 0),
    ("Down", 1),
    ("Left", 2),
    ("Right", 3),
    ("A", 6),
    ("B", 4),
    ("C", 5),
    ("Start", 7),
];

/// Parse a BK2 file (BizHawk TAS format) into a `Bk2Import`.
///
/// BK2 is a ZIP archive containing:
/// - `Header.txt`: key=value metadata (must have a Genesis/MD system)
/// - `Input Log.txt`: per-frame button state lines
pub fn parse_bk2(path: &Path) -> Result<Bk2Import> {
    let file = std::fs::File::open(path)
        .with_context(|| format!("failed to open BK2 file: {}", path.display()))?;
    parse_bk2_from_reader(file, &path.display().to_string())
}

/// Parse from any `Read + Seek` source (allows testing with in-memory ZIPs).
pub fn parse_bk2_from_reader<R: Read + std::io::Seek>(
    reader: R,
    source_label: &str,
) -> Result<Bk2Import> {
    let mut archive =
        zip::ZipArchive::new(reader).with_context(|| format!("invalid ZIP: {source_label}"))?;

    // Read and validate header
    let header_text = read_zip_entry(&mut archive, "Header.txt", source_label)?;
    validate_header(&header_text, source_label)?;

    // Parse input log
    let input_text = read_zip_entry(&mut archive, "Input Log.txt", source_label)?;
    let script = parse_input_log(&input_text)?;

    let slug = Path::new(source_label)
        .file_stem()
        .map_or_else(|| "tas".into(), |s| s.to_string_lossy().into_owned())
        .replace(
            |c: char| !c.is_ascii_alphanumeric() && c != '-' && c != '_',
            "_",
        );

    Ok(Bk2Import {
        slug,
        total_frames: script.len(),
        script,
    })
}

fn read_zip_entry<R: Read + std::io::Seek>(
    archive: &mut zip::ZipArchive<R>,
    name: &str,
    source_label: &str,
) -> Result<String> {
    let mut entry = archive
        .by_name(name)
        .with_context(|| format!("{source_label} missing {name}"))?;
    let mut buf = String::new();
    entry
        .read_to_string(&mut buf)
        .with_context(|| format!("failed to read {name} from {source_label}"))?;
    Ok(buf)
}

/// Verify the BK2 header declares a Genesis/Mega Drive system.
fn validate_header(header: &str, source_label: &str) -> Result<()> {
    let system = header
        .lines()
        .find_map(|line| {
            let (key, value) = line.split_once(' ')?;
            if key == "Platform" || key == "SystemID" {
                Some(value.trim())
            } else {
                None
            }
        })
        .unwrap_or("");

    let is_genesis = ["GEN", "Genesis", "MD", "MegaDrive", "Mega Drive", "SEG"]
        .iter()
        .any(|s| system.eq_ignore_ascii_case(s));
    if !is_genesis {
        bail!("{source_label}: unsupported platform '{system}' (expected Genesis/MD)");
    }
    Ok(())
}

pub(crate) fn first_crc_mismatch_from_frame(
    native_crcs: &[u32],
    oracle_crcs: &[u32],
    start: usize,
    end: usize,
) -> Option<usize> {
    let expected_len = end.saturating_sub(start);
    let native_window = native_crcs
        .get(start..native_crcs.len().min(end))
        .unwrap_or_default();
    let oracle_window = oracle_crcs
        .get(start..oracle_crcs.len().min(end))
        .unwrap_or_default();
    crate::sweep::find_first_crc_mismatch(native_window, oracle_window)
        .map(|offset| start + offset)
        .or_else(|| {
            (native_window.len() != expected_len || oracle_window.len() != expected_len)
                .then(|| start + native_window.len().min(oracle_window.len()))
        })
}

fn rng_divergence_digest(diffs: &[crate::sweep::StateDiff]) -> Option<String> {
    let mut hasher = Sha1::new();
    let mut count = 0usize;
    for diff in diffs
        .iter()
        .filter(|diff| diff.category == crate::sweep::DivergenceCategory::Rng)
    {
        count += 1;
        for field in [
            diff.checkpoint.as_str(),
            diff.field.as_str(),
            diff.native_value.as_str(),
            diff.oracle_value.as_str(),
        ] {
            hasher.update((field.len() as u64).to_be_bytes());
            hasher.update(field.as_bytes());
        }
    }
    (count != 0).then(|| format!("{:x}", hasher.finalize()))
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct RngFrameMismatch {
    frame_index: usize,
    native_offset: Option<u16>,
    oracle_offset: Option<u16>,
    first_ring_difference: Option<(usize, u8, u8)>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct RngFrameComparison {
    start_frame: Option<usize>,
    compared_frames: usize,
    mismatch_count: usize,
    first_mismatch: Option<RngFrameMismatch>,
    mismatch_digest: Option<String>,
}

fn compare_rng_frame_streams(
    native: &[Option<wildside_core::RomRandomFrameState>],
    oracle: &[Option<wildside_core::RomRandomFrameState>],
    range_start: usize,
) -> RngFrameComparison {
    let shared_len = native.len().min(oracle.len());
    let start_frame = (range_start.min(shared_len)..shared_len)
        .find(|&index| native[index].is_some() && oracle[index].is_some());
    let Some(start) = start_frame else {
        return RngFrameComparison {
            start_frame: None,
            compared_frames: 0,
            mismatch_count: 0,
            first_mismatch: None,
            mismatch_digest: None,
        };
    };

    let mut hasher = Sha1::new();
    let mut mismatch_count = 0usize;
    let mut first_mismatch = None;
    for frame_index in start..shared_len {
        let native_state = native[frame_index].as_ref();
        let oracle_state = oracle[frame_index].as_ref();
        if native_state == oracle_state {
            continue;
        }

        mismatch_count += 1;
        hasher.update((frame_index as u64).to_be_bytes());
        update_rng_frame_digest(&mut hasher, native_state);
        update_rng_frame_digest(&mut hasher, oracle_state);
        first_mismatch.get_or_insert_with(|| RngFrameMismatch {
            frame_index,
            native_offset: native_state.map(wildside_core::RomRandomFrameState::offset),
            oracle_offset: oracle_state.map(wildside_core::RomRandomFrameState::offset),
            first_ring_difference: native_state
                .zip(oracle_state)
                .and_then(|(left, right)| left.ring().first_difference(right.ring())),
        });
    }

    RngFrameComparison {
        start_frame,
        compared_frames: shared_len - start,
        mismatch_count,
        first_mismatch,
        mismatch_digest: (mismatch_count != 0).then(|| format!("{:x}", hasher.finalize())),
    }
}

fn update_rng_frame_digest(hasher: &mut Sha1, state: Option<&wildside_core::RomRandomFrameState>) {
    match state {
        Some(state) => {
            hasher.update([1]);
            hasher.update(state.offset().to_be_bytes());
            hasher.update(state.ring().as_bytes());
        }
        None => hasher.update([0]),
    }
}

fn extract_native_rng_frame_stream(
    trace: &[wildside_core::NativeTraceFrame],
) -> Result<Vec<Option<wildside_core::RomRandomFrameState>>> {
    trace
        .iter()
        .map(|frame| {
            let Some(offset) = frame.rng_offset else {
                return Ok(None);
            };
            if !wildside_core::RomRandomFrameState::offset_is_initialised(offset) {
                return Ok(None);
            }
            let ring = frame.rng_ring.clone().with_context(|| {
                format!(
                    "native frame {} has initialised RNG offset {offset:#06x} without a complete RNG ring",
                    frame.frame_index
                )
            })?;
            Ok(wildside_core::RomRandomFrameState::from_initialised_parts(offset, ring))
        })
        .collect()
}

/// Parse the `Input Log.txt` content into per-frame u8 button masks.
///
/// Format:
/// ```text
/// [Input]
/// LogKey:#Power|Reset|#P1 Up|P1 Down|...|#P2 ...|
/// |..|UDLRABCS......|..............|
/// ```
///
/// The LogKey defines column order within each `|`-delimited section.
/// We parse it dynamically to build a mapping from character position
/// within the P1 section to our mask bit.
fn parse_input_log(input: &str) -> Result<Vec<u8>> {
    let mut lines = input.lines();

    // Skip [Input] header
    for line in lines.by_ref() {
        let trimmed = line.trim();
        if trimmed == "[Input]" {
            break;
        }
    }

    // Parse LogKey to determine P1 button positions
    let logkey_line = lines
        .by_ref()
        .find(|l| l.trim().starts_with("LogKey:"))
        .context("Input Log.txt missing LogKey line")?;
    let button_positions = parse_logkey(logkey_line)?;

    // Parse frame lines
    let mut script = Vec::new();
    for line in lines {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed == "[/Input]" {
            continue;
        }
        // Each line: |console|p1buttons|p2buttons|
        let mask = parse_frame_line(trimmed, &button_positions)?;
        script.push(mask);
    }

    Ok(script)
}

/// Mapping from character position within the P1 section to our mask bit.
struct ButtonPositions {
    /// (char_index_in_p1_section, mask_bit)
    mappings: Vec<(usize, u8)>,
}

/// Parse the LogKey line to determine which character positions in the P1
/// section correspond to which buttons.
///
/// Example LogKey:
/// `LogKey:#Power|Reset|#P1 Up|P1 Down|P1 Left|P1 Right|P1 A|P1 B|P1 C|P1 Start|P1 X|P1 Y|P1 Z|P1 Mode|#P2 Up|...`
///
/// Sections are delimited by `#` markers. The P1 section starts at `#P1`
/// and contains all `P1 xxx` entries until the next `#`.
fn parse_logkey(line: &str) -> Result<ButtonPositions> {
    let key_part = line
        .trim()
        .strip_prefix("LogKey:")
        .context("invalid LogKey line")?;

    // Split by | to get all column names
    let columns: Vec<&str> = key_part.split('|').collect();

    // Find the P1 section: entries starting with "P1 " or "#P1 "
    let mut mappings = Vec::new();

    // Track which section we're in by counting `#` markers
    // Sections: #Console | #P1 | #P2
    // We want the P1 section columns
    let mut in_p1 = false;
    let mut p1_col_index = 0;

    for col in &columns {
        let col = col.trim();
        if col.is_empty() {
            continue;
        }

        // A '#' prefix marks the start of a new section
        if let Some(name) = col.strip_prefix('#') {
            if name.starts_with("P1") {
                in_p1 = true;
                // This column itself is a P1 button (e.g. "#P1 Up")
                let button_name = name.strip_prefix("P1 ").unwrap_or(name);
                if let Some(&(_, bit)) = GENESIS_BUTTON_MAP
                    .iter()
                    .find(|(name, _)| *name == button_name)
                {
                    mappings.push((p1_col_index, bit));
                }
                p1_col_index += 1;
                continue;
            } else if in_p1 {
                // Hit a new section marker (e.g. #P2), stop
                break;
            }
            continue;
        }

        if in_p1 {
            let button_name = col.strip_prefix("P1 ").unwrap_or(col);
            if let Some(&(_, bit)) = GENESIS_BUTTON_MAP
                .iter()
                .find(|(name, _)| *name == button_name)
            {
                mappings.push((p1_col_index, bit));
            }
            p1_col_index += 1;
        }
    }

    if mappings.is_empty() {
        bail!("LogKey contains no recognised P1 Genesis buttons");
    }

    Ok(ButtonPositions { mappings })
}

/// Parse a single frame line like `|..|UDLR.BCS......|..............|`
/// into a u8 button mask.
fn parse_frame_line(line: &str, positions: &ButtonPositions) -> Result<u8> {
    // Split by | — sections: ["", console, p1, p2, ""]
    let sections: Vec<&str> = line.split('|').collect();
    // P1 is typically the third section (index 2): |console|p1|p2|
    let p1_section = sections.get(2).context("frame line missing P1 section")?;

    let p1_chars: Vec<char> = p1_section.chars().collect();
    let mut mask: u8 = 0;

    for &(char_idx, bit) in &positions.mappings {
        if let Some(&ch) = p1_chars.get(char_idx)
            && ch != '.'
        {
            mask |= 1 << bit;
        }
    }

    Ok(mask)
}

/// Convert a parsed BK2 import into a `DynamicScenario` with periodic checkpoints.
pub fn bk2_to_scenario(import: &Bk2Import, checkpoint_interval: usize) -> DynamicScenario {
    let frame_count = import.total_frames;

    let mut checkpoints = Vec::new();
    // Always checkpoint frame 0
    checkpoints.push(DynamicCheckpoint {
        label: "frame_0".into(),
        frame_index: 0,
    });

    // Periodic checkpoints
    if checkpoint_interval > 0 {
        for next in (checkpoint_interval..frame_count).step_by(checkpoint_interval) {
            checkpoints.push(DynamicCheckpoint {
                label: format!("frame_{next}"),
                frame_index: next,
            });
        }
    }

    // Final frame (if not already covered)
    let last = frame_count.saturating_sub(1);
    if checkpoints
        .last()
        .is_some_and(|checkpoint| checkpoint.frame_index < last)
    {
        checkpoints.push(DynamicCheckpoint {
            label: format!("frame_{last}"),
            frame_index: last,
        });
    }

    DynamicScenario {
        slug: import.slug.clone(),
        script: import.script.clone(),
        frame_count,
        checkpoints,
        start_position: None,
        start_velocity: None,
        camera_position: None,
        playtest_form: None,
        initial_diamond_count: None,
        initial_lives: None,
        warp_to_level: None, // TAS runs from power-on
        settle_oracle_warp: false,
    }
}

/// Verify a TAS scenario: run native + oracle, compare state at checkpoints.
///
/// Optionally slices the scenario to `[frame_start, frame_end)` for iterative
/// debugging. When `checkpoint_interval` is provided, regenerates checkpoints
/// within the slice rather than using the scenario's original checkpoints.
/// Issue #7: verify-tas is meant to be the authoritative fidelity gate, which
/// cold-boots — reaching the level by replayed input from power-on, exactly
/// like the published TAS (`warp_to_level: None`). A warped scenario is a
/// synthetic spawn whose initial conditions (RNG, enemy/Murderwall ticks,
/// camera) differ from the ROM's cold-boot arrival, so a "pass" proves
/// self-consistency, not cold-boot fidelity. The idle sweep warps legitimately
/// because it is a self-consistency check (issue #26).
///
/// Returns an advisory message (for the caller to surface) when the scenario
/// warps; `None` for cold-boot. It stays advisory rather than rejecting because
/// committed frontier regression probes legitimately auto-boot (e.g.
/// `issue305_pause_give_up_last_life_probe`, `warp_to_level: 0`), so a hard
/// reject would turn the green frontier gate red. Whether the authoritative
/// gate should hard-reject warps is issue #7 itself (needs-decision).
fn cold_boot_advisory(scenario: &DynamicScenario) -> Option<String> {
    scenario.warp_to_level.map(|level| {
        format!(
            "verify-tas: scenario warps (warp_to_level={level:#x}), so this is a \
             self-consistency run, NOT an authoritative cold-boot verification — a pass \
             does not prove cold-boot fidelity (issue #7)"
        )
    })
}

fn run_tas_oracle(
    env: &crate::oracle::OracleEnvironment,
    rom_path: &Path,
    scenario_file: &Path,
    oracle_dir: &Path,
    skip_frames_before: Option<usize>,
    capture_rng_stream: bool,
) -> Result<crate::oracle::OracleManifest> {
    crate::oracle::run_jgenesis_capture_dynamic_with_options(
        env,
        rom_path,
        scenario_file,
        oracle_dir,
        &crate::oracle::OracleRunOptions {
            skip_frames_before,
            capture_rng_stream,
            ..crate::oracle::OracleRunOptions::default()
        },
    )
}

pub fn verify_tas(
    repo_root: &std::path::Path,
    rom_path: &std::path::Path,
    manifest: &wildside_data::AssetManifest,
    scenario_path: &std::path::Path,
    frame_start: Option<usize>,
    frame_end: Option<usize>,
    checkpoint_interval: Option<usize>,
    frontier_kind: crate::sweep::FrontierKind,
    include_rng: bool,
    compare_pixels: bool,
) -> Result<()> {
    let scenario_json = std::fs::read_to_string(scenario_path)
        .with_context(|| format!("failed to read {}", scenario_path.display()))?;
    let mut scenario: DynamicScenario =
        serde_json::from_str(&scenario_json).context("invalid scenario JSON")?;

    if let Some(advisory) = cold_boot_advisory(&scenario) {
        eprintln!("warning: {advisory}");
    }

    // Slice the scenario if requested
    let start = frame_start.unwrap_or(0);
    let end = frame_end
        .unwrap_or(scenario.frame_count)
        .min(scenario.frame_count);
    if start > 0 || end < scenario.frame_count {
        scenario.script = scenario.script[..end.min(scenario.script.len())].to_vec();
        scenario.frame_count = end;
        // Filter checkpoints to the slice
        scenario
            .checkpoints
            .retain(|cp| cp.frame_index >= start && cp.frame_index < end);
    }

    // Regenerate checkpoints if interval overridden
    if let Some(interval) = checkpoint_interval {
        scenario.checkpoints = generate_checkpoints(start, end, interval);
    }

    // Ensure we have at least one checkpoint
    if scenario.checkpoints.is_empty() && end > start {
        scenario.checkpoints.push(DynamicCheckpoint {
            label: format!("frame_{}", end.saturating_sub(1)),
            frame_index: end.saturating_sub(1),
        });
    }

    println!(
        "verify-tas: {} (frames {start}..{end}, {} checkpoints)",
        scenario.slug,
        scenario.checkpoints.len()
    );
    println!("  frontier kind: {frontier_kind:?}");

    // Run native
    println!("  running native ({end} frames)...");
    let native_trace = run_native_dynamic_scenario(
        manifest.clone(),
        &scenario,
        NativeDynamicRunOptions::default(),
    )?;
    let native_rng_states = include_rng
        .then(|| extract_native_rng_frame_stream(&native_trace))
        .transpose()?
        .unwrap_or_default();

    let mut native_states =
        crate::sweep::extract_checkpoint_states(&native_trace, &scenario.checkpoints);

    // Detect level transitions in native trace and inject transition checkpoints
    let mut transitions = Vec::new();
    for window in native_trace.windows(2) {
        if window[0].current_level != window[1].current_level {
            transitions.push((
                window[0].frame_index,
                window[0].current_level,
                window[1].current_level,
            ));
        }
    }
    if !transitions.is_empty() {
        println!("  level transitions detected: {}", transitions.len());
        for (frame, from, to) in &transitions {
            println!("    frame {frame}: {from:?} -> {to:?}");
        }
        // Two-pass: merge transition checkpoints into scenario
        merge_transition_checkpoints(&mut scenario, &transitions, start, end);
        // Re-extract native states with the augmented checkpoints
        native_states =
            crate::sweep::extract_checkpoint_states(&native_trace, &scenario.checkpoints);
    }

    // Write scenario JSON for oracle (with transition checkpoints merged)
    let run_dir = repo_root.join(".cache/tas-verify").join(&scenario.slug);
    std::fs::create_dir_all(&run_dir)
        .with_context(|| format!("failed to create {}", run_dir.display()))?;
    let scenario_file = run_dir.join("scenario.json");
    std::fs::write(&scenario_file, serde_json::to_vec_pretty(&scenario)?)
        .with_context(|| format!("failed to write {}", scenario_file.display()))?;

    // Run oracle
    println!("  running oracle ({end} frames)...");
    let env = crate::oracle::ensure_oracles(repo_root)?;
    let oracle_dir = run_dir.join("oracle");
    std::fs::create_dir_all(&oracle_dir)?;
    let oracle = run_tas_oracle(
        &env,
        rom_path,
        &scenario_file,
        &oracle_dir,
        None,
        include_rng,
    )?;
    let pixel_comparison = compare_pixels
        .then(|| crate::tas_pixels::compare_tas_pixels(manifest, &scenario, &oracle, start, end))
        .transpose()?;
    if let Some(comparison) = &pixel_comparison {
        println!("  pixels: {}", comparison.console_summary());
    }

    // Per-frame player/camera progression CRC (fast divergence detection).
    // This is deliberately not a whole-state or rendered-frame checksum.
    let native_crcs: Vec<u32> = native_trace
        .iter()
        .map(wildside_core::NativeTraceFrame::authoritative_crc32)
        .collect();
    let first_crc_mismatch = if matches!(frontier_kind, crate::sweep::FrontierKind::Gameplay) {
        let mismatch = first_crc_mismatch_from_frame(&native_crcs, &oracle.frame_crcs, start, end);
        if let Some(first_mismatch) = mismatch {
            println!("  player/camera CRC divergence at frame {first_mismatch}");
        } else {
            let compared_frames = native_crcs
                .len()
                .min(oracle.frame_crcs.len())
                .saturating_sub(start);
            println!("  player/camera CRC: all {compared_frames} frames match in {start}..{end}",);
        }
        mismatch
    } else {
        None
    };
    let rng_frame_comparison = include_rng
        .then(|| compare_rng_frame_streams(&native_rng_states, &oracle.frame_rng_states, start));
    if let Some(comparison) = &rng_frame_comparison {
        match &comparison.first_mismatch {
            Some(first) => println!(
                "  RNG: {} mismatches across {} compared frames; first at frame {} (native offset {:?}, oracle offset {:?}, ring {:?})",
                comparison.mismatch_count,
                comparison.compared_frames,
                first.frame_index,
                first.native_offset,
                first.oracle_offset,
                first.first_ring_difference,
            ),
            None => println!(
                "  RNG: all {} post-Init_RNG frames match from {:?}",
                comparison.compared_frames, comparison.start_frame,
            ),
        }
    }

    // Compare
    let mut state_diffs = crate::sweep::compare_sweep_states_for_frontier_with_options(
        &native_states,
        &oracle.states,
        frontier_kind,
        crate::sweep::SweepStateComparisonOptions {
            include_rng,
            ..crate::sweep::SweepStateComparisonOptions::default()
        },
    );
    let crc_clean = matches!(frontier_kind, crate::sweep::FrontierKind::Gameplay)
        && !oracle.frame_crcs.is_empty()
        && first_crc_mismatch.is_none();
    reclassify_crc_clean_velocity_diffs(
        &mut state_diffs,
        &native_states,
        &oracle.states,
        crc_clean,
    );
    reclassify_crc_clean_lion_projectile_count_diffs(
        &mut state_diffs,
        &native_states,
        &oracle.states,
        crc_clean,
    );
    reclassify_crc_clean_pause_trace_latch_diffs(
        &mut state_diffs,
        &native_states,
        &oracle.states,
        crc_clean,
    );
    let divergence_summary = crate::sweep::summarise_state_diffs(&state_diffs);
    let enemy_frontier_report =
        matches!(frontier_kind, crate::sweep::FrontierKind::Gameplay).then(|| {
            let enemy_diffs = crate::sweep::compare_sweep_states_for_frontier_with_options(
                &native_states,
                &oracle.states,
                crate::sweep::FrontierKind::Enemy,
                crate::sweep::SweepStateComparisonOptions::default(),
            );
            let report =
                frontier_diff_report(&enemy_diffs, native_states.len().min(oracle.states.len()));
            let summary = crate::sweep::summarise_state_diffs(&enemy_diffs);
            println!(
                "  enemy frontier: {} divergences ({} authoritative)",
                enemy_diffs.len(),
                summary.authoritative,
            );
            report
        });

    if state_diffs.is_empty() {
        println!(
            "  PASS: all {} checkpoints match",
            scenario.checkpoints.len()
        );
    } else {
        println!(
            "  {} divergences ({} authoritative, {} advisory, {} oracle limitations):",
            state_diffs.len(),
            divergence_summary.authoritative,
            divergence_summary.advisory,
            divergence_summary.oracle_limitation
        );
        // Show first few divergences
        for diff in state_diffs.iter().take(20) {
            println!(
                "    [{}/{}] native={} oracle={}",
                diff.checkpoint, diff.field, diff.native_value, diff.oracle_value
            );
        }
        if state_diffs.len() > 20 {
            println!("    ... and {} more", state_diffs.len() - 20);
        }

        // Find first authoritative divergence frame
        if divergence_summary.authoritative > 0
            && let Some(first_auth) = state_diffs
                .iter()
                .find(|diff| diff.category == crate::sweep::DivergenceCategory::Authoritative)
        {
            println!(
                "  first authoritative divergence: checkpoint {}",
                first_auth.checkpoint
            );
            if let Some((native_state, oracle_state)) =
                checkpoint_pair(&first_auth.checkpoint, &native_states, &oracle.states)
            {
                println!("    native: {}", state_boot_summary(native_state));
                println!("    oracle: {}", state_boot_summary(oracle_state));
            }
        }
    }

    let first_authoritative_diff = state_diffs
        .iter()
        .find(|diff| diff.category == crate::sweep::DivergenceCategory::Authoritative)
        .map(|diff| {
            let (native_state, oracle_state) =
                checkpoint_pair(&diff.checkpoint, &native_states, &oracle.states)
                    .unwrap_or_else(|| panic!("missing checkpoint pair for {}", diff.checkpoint));
            serde_json::json!({
                "checkpoint": diff.checkpoint,
                "field": diff.field,
                "native_value": diff.native_value,
                "oracle_value": diff.oracle_value,
                "native_state": native_state,
                "oracle_state": oracle_state,
            })
        });
    let rng_divergence_digest = rng_divergence_digest(&state_diffs);
    let rng_frame_report = rng_frame_comparison.as_ref().map(|comparison| {
        let first_mismatch = comparison.first_mismatch.as_ref().map(|mismatch| {
            serde_json::json!({
                "frame": mismatch.frame_index,
                "native_offset": mismatch.native_offset,
                "oracle_offset": mismatch.oracle_offset,
                "first_ring_difference": mismatch.first_ring_difference.map(
                    |(index, native_byte, oracle_byte)| serde_json::json!({
                        "index": index,
                        "native_byte": native_byte,
                        "oracle_byte": oracle_byte,
                    })
                ),
            })
        });
        serde_json::json!({
            "frame_counts": {
                "requested": end.saturating_sub(start),
                "native": native_rng_states.len().min(end).saturating_sub(start),
                "oracle": oracle.frame_rng_states.len().min(end).saturating_sub(start),
            },
            "start_frame": comparison.start_frame,
            "compared_frames": comparison.compared_frames,
            "mismatch_count": comparison.mismatch_count,
            "mismatch_digest": comparison.mismatch_digest,
            "first_mismatch": first_mismatch,
        })
    });

    // Write summary report
    let report = serde_json::json!({
        "slug": scenario.slug,
        "frame_range": [start, end],
        "crc_frame_counts": {
            "requested": end.saturating_sub(start),
            "native": native_crcs.len().min(end).saturating_sub(start),
            "oracle": oracle.frame_crcs.len().min(end).saturating_sub(start),
        },
        "crc_projection": {
            "name": wildside_core::NativeTraceFrame::PLAYER_CAMERA_CRC_PROJECTION_NAME,
            "fields": wildside_core::NativeTraceFrame::PLAYER_CAMERA_CRC_FIELDS,
            "proves_pixels": false,
            "proves_whole_state": false,
        },
        "checkpoints": scenario.checkpoints.len(),
        "transitions": transitions.len(),
        "first_crc_mismatch": first_crc_mismatch,
        "total_diffs": state_diffs.len(),
        "divergence_summary": {
            "authoritative": divergence_summary.authoritative,
            "advisory": divergence_summary.advisory,
            "accumulation": divergence_summary.accumulation,
            "rng": divergence_summary.rng,
            "oracle_limitation": divergence_summary.oracle_limitation,
        },
        "rng_divergence_digest": rng_divergence_digest,
        "rng_frame_comparison": rng_frame_report,
        "first_authoritative_diff": first_authoritative_diff,
        "enemy_frontier": enemy_frontier_report,
        "pixel_comparison": pixel_comparison,
    });
    let report_path = run_dir.join("report.json");
    std::fs::write(&report_path, serde_json::to_vec_pretty(&report)?)?;
    println!("  report: {}", report_path.display());

    Ok(())
}

fn frontier_diff_report(
    state_diffs: &[crate::sweep::StateDiff],
    checkpoint_count: usize,
) -> serde_json::Value {
    let summary = crate::sweep::summarise_state_diffs(state_diffs);
    let first_authoritative_diff = state_diffs
        .iter()
        .find(|diff| diff.category == crate::sweep::DivergenceCategory::Authoritative)
        .map(|diff| {
            serde_json::json!({
                "checkpoint": diff.checkpoint,
                "field": diff.field,
                "native_value": diff.native_value,
                "oracle_value": diff.oracle_value,
            })
        });

    serde_json::json!({
        "checkpoints": checkpoint_count,
        "total_diffs": state_diffs.len(),
        "divergence_summary": summary,
        "first_authoritative_diff": first_authoritative_diff,
    })
}

fn checkpoint_pair<'a>(
    checkpoint: &str,
    native_states: &'a [OracleState],
    oracle_states: &'a [OracleState],
) -> Option<(&'a OracleState, &'a OracleState)> {
    let native = native_states
        .iter()
        .find(|state| state.label == checkpoint)?;
    let oracle = oracle_states
        .iter()
        .find(|state| state.label == checkpoint)?;
    Some((native, oracle))
}

fn reclassify_crc_clean_velocity_diffs(
    state_diffs: &mut [crate::sweep::StateDiff],
    native_states: &[OracleState],
    oracle_states: &[OracleState],
    crc_clean: bool,
) {
    if !crc_clean {
        return;
    }

    for diff in state_diffs {
        if !matches!(diff.field.as_str(), "kid_x_vel" | "kid_y_vel") {
            continue;
        }
        let Some((native_state, oracle_state)) =
            checkpoint_pair(&diff.checkpoint, native_states, oracle_states)
        else {
            continue;
        };
        if native_state.game_mode == oracle_state.game_mode
            && native_state.kid_x_pos == oracle_state.kid_x_pos
            && native_state.kid_y_pos == oracle_state.kid_y_pos
        {
            // CRC-clean frames prove the player/camera projection is in
            // lockstep. Treat isolated top-level velocity export skew as a
            // trace limitation so gates still block on real gameplay,
            // position, object, or CRC drift.
            diff.category = crate::sweep::DivergenceCategory::OracleLimitation;
        }
    }
}

pub(crate) fn reclassify_crc_clean_lion_projectile_count_diffs(
    state_diffs: &mut [crate::sweep::StateDiff],
    native_states: &[OracleState],
    oracle_states: &[OracleState],
    crc_clean: bool,
) {
    if !crc_clean {
        return;
    }

    for diff in state_diffs {
        if diff.field != "number_objects" {
            continue;
        }
        let Some((native_state, oracle_state)) =
            checkpoint_pair(&diff.checkpoint, native_states, oracle_states)
        else {
            continue;
        };
        let native_has_lion_projectile = native_state
            .gameplay_debug_fields
            .get("lion_active_projectiles")
            .and_then(|count| count.parse::<u16>().ok())
            .is_some_and(|count| count > 0);
        let oracle_slot_dump_has_lion_projectile = oracle_state
            .gameplay_debug_fields
            .get("oracle_object_slots_loc_e1e")
            .is_some_and(|slots| {
                slots.contains("code=0x03e196") || slots.contains("code=0x03e2ac")
            });
        let lion_parent_is_active = native_state
            .active_enemies
            .iter()
            .chain(oracle_state.active_enemies.iter())
            .any(|enemy| enemy.object_type == 0x12);
        if native_state.game_mode == oracle_state.game_mode
            && native_state.current_level_id == oracle_state.current_level_id
            && native_state.player_1_level_id == oracle_state.player_1_level_id
            && native_state.kid_x_pos == oracle_state.kid_x_pos
            && native_state.kid_y_pos == oracle_state.kid_y_pos
            && native_state.number_hitpoints == oracle_state.number_hitpoints
            && native_state.current_helmet_id == oracle_state.current_helmet_id
            && native_state.number_objects == oracle_state.number_objects.saturating_add(1)
            && (native_has_lion_projectile || lion_parent_is_active)
            && oracle_slot_dump_has_lion_projectile
        {
            // ASM Lion.asm loc_3E12A allocates the projectile worker through
            // j_Allocate_ObjectSlot. Oracle WRAM samples expose the live slot
            // at loc_3E196, or loc_3E2AC while hibernating in the bounce loop,
            // even when the exported `Number_Objects` is one lower.
            // CRC-clean frames prove the player/camera projection is aligned;
            // the live oracle slot dump separately proves this count skew.
            diff.category = crate::sweep::DivergenceCategory::OracleLimitation;
        }
    }
}

fn reclassify_crc_clean_pause_trace_latch_diffs(
    state_diffs: &mut [crate::sweep::StateDiff],
    native_states: &[OracleState],
    oracle_states: &[OracleState],
    crc_clean: bool,
) {
    if !crc_clean {
        return;
    }

    for diff in state_diffs {
        if !matches!(diff.field.as_str(), "game_paused" | "character_movement") {
            continue;
        }
        let Some((native_state, oracle_state)) =
            checkpoint_pair(&diff.checkpoint, native_states, oracle_states)
        else {
            continue;
        };
        if pause_trace_latch_export_only(native_state, oracle_state) {
            // CRC-clean pause-menu probes prove the player-visible and
            // gameplay-critical state stayed aligned.  `Game_Paused` and
            // `Character_Movement` are sampled at different shell/VBlank
            // boundaries in native vs oracle, so isolated latch skew here is a
            // trace-export limitation rather than gameplay drift.
            diff.category = crate::sweep::DivergenceCategory::OracleLimitation;
        }
    }
}

fn pause_trace_latch_export_only(native_state: &OracleState, oracle_state: &OracleState) -> bool {
    native_state.game_mode == oracle_state.game_mode
        && native_state.boot_phase == oracle_state.boot_phase
        && native_state.current_player == oracle_state.current_player
        && native_state.current_level_id == oracle_state.current_level_id
        && native_state.player_1_level_id == oracle_state.player_1_level_id
        && native_state.camera_x_pos == oracle_state.camera_x_pos
        && native_state.camera_y_pos == oracle_state.camera_y_pos
        && native_state.kid_x_pos == oracle_state.kid_x_pos
        && native_state.kid_y_pos == oracle_state.kid_y_pos
        && native_state.current_helmet_id == oracle_state.current_helmet_id
        && native_state.number_hitpoints == oracle_state.number_hitpoints
        && native_state.number_lives == oracle_state.number_lives
        && native_state.number_objects == oracle_state.number_objects
        && native_state.number_gfx_objects == oracle_state.number_gfx_objects
        && native_state.score == oracle_state.score
        && native_state.number_diamonds == oracle_state.number_diamonds
        && native_state.diamond_power_active == oracle_state.diamond_power_active
        && native_state.kid_x_direction == oracle_state.kid_x_direction
        && native_state.just_received_damage == oracle_state.just_received_damage
        && native_state.currently_transforming == oracle_state.currently_transforming
}

fn state_boot_summary(state: &OracleState) -> String {
    let boot_phase = state.boot_phase.as_deref().unwrap_or("in_game");
    let elapsed = state
        .boot_phase_elapsed
        .map_or_else(|| "-".into(), |value| value.to_string());
    let menu = state
        .main_menu_option
        .map_or_else(|| "-".into(), |value| value.to_string());
    format!(
        "mode={:#06x} phase={} elapsed={} title_ready={} menu={} kid=({:#010x},{:#010x}) objects={}/{} lives={} hp={}",
        state.game_mode,
        boot_phase,
        elapsed,
        state.title_screen_ready,
        menu,
        state.kid_x_pos,
        state.kid_y_pos,
        state.number_objects,
        state.number_gfx_objects,
        state.number_lives,
        state.number_hitpoints
    )
}

/// Merge level-transition checkpoints into the scenario's existing checkpoints.
/// Adds checkpoints at the frame just before and just after each transition.
fn merge_transition_checkpoints(
    scenario: &mut DynamicScenario,
    transitions: &[(usize, wildside_core::LevelId, wildside_core::LevelId)],
    start: usize,
    end: usize,
) {
    let existing_frames: std::collections::HashSet<usize> = scenario
        .checkpoints
        .iter()
        .map(|cp| cp.frame_index)
        .collect();

    for &(frame, from, to) in transitions {
        if frame < start || frame >= end {
            continue;
        }
        // Checkpoint just before transition (the last frame of the old level)
        if !existing_frames.contains(&frame) && frame < scenario.frame_count {
            scenario.checkpoints.push(DynamicCheckpoint {
                label: format!(
                    "transition_{:02x}_to_{:02x}_pre_frame_{frame}",
                    from.0, to.0
                ),
                frame_index: frame,
            });
        }
        // Checkpoint just after transition (the first frame of the new level)
        let after = frame + 1;
        if !existing_frames.contains(&after) && after < scenario.frame_count {
            scenario.checkpoints.push(DynamicCheckpoint {
                label: format!(
                    "transition_{:02x}_to_{:02x}_post_frame_{after}",
                    from.0, to.0
                ),
                frame_index: after,
            });
        }
    }

    // Sort checkpoints by frame index for consistent oracle comparison
    scenario.checkpoints.sort_by_key(|cp| cp.frame_index);
    scenario.checkpoints.dedup_by_key(|cp| cp.frame_index);
}

/// Result of a single verify-tas probe: either clean or divergent at a specific checkpoint.
struct ProbeResult {
    /// Frame index of the first authoritative or requested RNG divergence.
    first_blocking_frame: Option<usize>,
    /// Field name + values for the first blocking divergence.
    first_blocking_detail: Option<(String, String, String)>,
}

/// Run native + oracle for a scenario slice, returning the first blocking divergence.
/// This is the inner loop used by both `verify_tas` and `bisect_tas`.
fn probe_tas(
    repo_root: &std::path::Path,
    rom_path: &std::path::Path,
    manifest: &wildside_data::AssetManifest,
    scenario: &DynamicScenario,
    _start: usize,
    end: usize,
    checkpoints: &[DynamicCheckpoint],
    frontier_kind: crate::sweep::FrontierKind,
    include_rng: bool,
) -> Result<ProbeResult> {
    // Build a sliced scenario for this probe
    let probe_scenario = DynamicScenario {
        slug: scenario.slug.clone(),
        script: scenario.script[..end.min(scenario.script.len())].to_vec(),
        frame_count: end,
        checkpoints: checkpoints.to_vec(),
        start_position: scenario.start_position,
        start_velocity: scenario.start_velocity,
        camera_position: scenario.camera_position,
        playtest_form: scenario.playtest_form,
        initial_diamond_count: scenario.initial_diamond_count,
        initial_lives: scenario.initial_lives,
        warp_to_level: scenario.warp_to_level,
        settle_oracle_warp: scenario.settle_oracle_warp,
    };

    // Run native
    let native_trace = run_native_dynamic_scenario(
        manifest.clone(),
        &probe_scenario,
        NativeDynamicRunOptions::default(),
    )?;

    let native_states =
        crate::sweep::extract_checkpoint_states(&native_trace, &probe_scenario.checkpoints);

    // Write scenario JSON for oracle
    let run_dir = repo_root
        .join(".cache/tas-verify")
        .join(&probe_scenario.slug)
        .join("bisect");
    std::fs::create_dir_all(&run_dir)?;
    let scenario_file = run_dir.join("scenario.json");
    std::fs::write(&scenario_file, serde_json::to_vec_pretty(&probe_scenario)?)?;

    // Run oracle
    let env = crate::oracle::ensure_oracles(repo_root)?;
    let oracle_dir = run_dir.join("oracle");
    std::fs::create_dir_all(&oracle_dir)?;
    let oracle = crate::oracle::run_jgenesis_capture_dynamic(
        &env,
        rom_path,
        &scenario_file,
        &oracle_dir,
        None,
    )?;

    // Compare
    let state_diffs = crate::sweep::compare_sweep_states_for_frontier_with_options(
        &native_states,
        &oracle.states,
        frontier_kind,
        crate::sweep::SweepStateComparisonOptions {
            ignore_camera_position: probe_scenario.camera_position.is_some(),
            include_rng,
        },
    );
    let first_blocking = state_diffs.iter().find(|diff| {
        matches!(
            diff.category,
            crate::sweep::DivergenceCategory::Authoritative | crate::sweep::DivergenceCategory::Rng
        )
    });

    match first_blocking {
        Some(diff) => {
            // Parse frame index from checkpoint label "frame_NNN"
            let frame = diff
                .checkpoint
                .strip_prefix("frame_")
                .and_then(|s| s.parse::<usize>().ok())
                .unwrap_or(0);
            Ok(ProbeResult {
                first_blocking_frame: Some(frame),
                first_blocking_detail: Some((
                    diff.field.clone(),
                    diff.native_value.clone(),
                    diff.oracle_value.clone(),
                )),
            })
        }
        None => Ok(ProbeResult {
            first_blocking_frame: None,
            first_blocking_detail: None,
        }),
    }
}

/// Binary-search for the first authoritative or requested RNG divergence.
///
/// Strategy:
/// 1. Coarse scan with checkpoint-interval=50 to find the first divergent region
/// 2. Binary search within that region using per-frame checkpoints
pub fn bisect_tas(
    repo_root: &std::path::Path,
    rom_path: &std::path::Path,
    manifest: &wildside_data::AssetManifest,
    scenario_path: &std::path::Path,
    frame_start: Option<usize>,
    frame_end: Option<usize>,
    frontier_kind: crate::sweep::FrontierKind,
    include_rng: bool,
) -> Result<()> {
    let scenario_json = std::fs::read_to_string(scenario_path)
        .with_context(|| format!("failed to read {}", scenario_path.display()))?;
    let scenario: DynamicScenario =
        serde_json::from_str(&scenario_json).context("invalid scenario JSON")?;

    let start = frame_start.unwrap_or(0);
    let end = frame_end
        .unwrap_or(scenario.frame_count)
        .min(scenario.frame_count);

    println!(
        "bisect-tas: {} (frames {start}..{end}, frontier {:?})",
        scenario.slug, frontier_kind
    );

    if matches!(frontier_kind, crate::sweep::FrontierKind::Gameplay)
        && scenario.camera_position.is_none()
        && !include_rng
    {
        // Fast path: use per-frame CRC arrays if oracle supports them
        println!("  computing native player/camera CRCs ({end} frames)...");
        let crc_scenario = DynamicScenario {
            slug: scenario.slug.clone(),
            script: scenario.script[..end.min(scenario.script.len())].to_vec(),
            frame_count: end,
            checkpoints: Vec::new(),
            start_position: scenario.start_position,
            start_velocity: scenario.start_velocity,
            camera_position: scenario.camera_position,
            playtest_form: scenario.playtest_form,
            initial_diamond_count: scenario.initial_diamond_count,
            initial_lives: scenario.initial_lives,
            warp_to_level: scenario.warp_to_level,
            settle_oracle_warp: scenario.settle_oracle_warp,
        };
        let native_trace = run_native_dynamic_scenario(
            manifest.clone(),
            &crc_scenario,
            NativeDynamicRunOptions::default(),
        )?;
        let native_crcs: Vec<u32> = native_trace
            .iter()
            .map(wildside_core::NativeTraceFrame::authoritative_crc32)
            .collect();

        // Run oracle for full CRC arrays
        println!("  running oracle for player/camera CRC comparison ({end} frames)...");
        let crc_scenario = crc_oracle_scenario(crc_scenario, end);
        let run_dir = repo_root
            .join(".cache/tas-verify")
            .join(&scenario.slug)
            .join("bisect");
        std::fs::create_dir_all(&run_dir)?;
        let scenario_file = run_dir.join("scenario.json");
        std::fs::write(&scenario_file, serde_json::to_vec_pretty(&crc_scenario)?)?;
        let env = crate::oracle::ensure_oracles(repo_root)?;
        let oracle_dir = run_dir.join("oracle");
        std::fs::create_dir_all(&oracle_dir)?;
        let oracle = crate::oracle::run_jgenesis_capture_dynamic(
            &env,
            rom_path,
            &scenario_file,
            &oracle_dir,
            None,
        )?;

        if !oracle.frame_crcs.is_empty() {
            if let Some(frame_idx) =
                first_crc_mismatch_from_frame(&native_crcs, &oracle.frame_crcs, start, end)
            {
                println!();
                println!("  RESULT: first player/camera CRC divergence at frame {frame_idx}");
                if let Some(native_frame) = native_trace.get(frame_idx) {
                    println!(
                        "    native: mode={:#06x} level={:02x} kid=({:#010x},{:#010x}) vel=({:?},{:?})",
                        native_frame.game_mode_value,
                        native_frame.current_level.0,
                        native_frame.kid_x_pos.unwrap_or(0),
                        native_frame.kid_y_pos.unwrap_or(0),
                        native_frame.kid_x_velocity,
                        native_frame.kid_y_velocity,
                    );
                }
                return Ok(());
            }
            println!(
                "  PASS: all {} frames match by CRC",
                native_crcs.len().min(oracle.frame_crcs.len())
            );
            return Ok(());
        }
    } else if matches!(frontier_kind, crate::sweep::FrontierKind::Gameplay) {
        println!("  explicit probe camera present; using field-level authoritative comparison...");
    }

    // Fallback: legacy binary search if oracle doesn't provide CRCs
    println!("  oracle has no player/camera CRC data, falling back to checkpoint bisection...");

    // Phase 1: coarse scan with interval=50
    let coarse_interval = 50;
    let coarse_checkpoints = generate_checkpoints(start, end, coarse_interval);
    println!(
        "  phase 1: coarse scan ({} checkpoints, interval={coarse_interval})",
        coarse_checkpoints.len()
    );

    let coarse = probe_tas(
        repo_root,
        rom_path,
        manifest,
        &scenario,
        start,
        end,
        &coarse_checkpoints,
        frontier_kind,
        include_rng,
    )?;

    let Some(first_coarse_frame) = coarse.first_blocking_frame else {
        println!("  PASS: no blocking divergences in frames {start}..{end}");
        return Ok(());
    };

    // Find the checkpoint before the divergent one
    let prev_checkpoint = coarse_checkpoints
        .iter()
        .rev()
        .find(|cp| cp.frame_index < first_coarse_frame)
        .map_or(start, |cp| cp.frame_index);

    println!(
        "  phase 1 result: first divergence at checkpoint frame_{first_coarse_frame}, searching {prev_checkpoint}..{first_coarse_frame}"
    );

    // Phase 2: binary search within [prev_checkpoint, first_coarse_frame]
    let mut lo = prev_checkpoint;
    let mut hi = first_coarse_frame;
    let mut best_frame = first_coarse_frame;
    let mut best_detail = coarse.first_blocking_detail;

    while lo < hi {
        let mid = lo + (hi - lo) / 2;
        // We need at least 2 distinct frames to check; if mid == lo, we're done
        if mid == lo {
            break;
        }

        // Probe with a single checkpoint at mid
        let checkpoints = vec![DynamicCheckpoint {
            label: format!("frame_{mid}"),
            frame_index: mid,
        }];
        println!("  bisect: probing frame {mid} (range {lo}..{hi})");

        let result = probe_tas(
            repo_root,
            rom_path,
            manifest,
            &scenario,
            start,
            // Must run up to at least mid+1 so the checkpoint is reachable
            mid + 1,
            &checkpoints,
            frontier_kind,
            include_rng,
        )?;

        match result.first_blocking_frame {
            Some(f) => {
                // Divergence at or before mid — search lower half
                best_frame = f;
                best_detail = result.first_blocking_detail;
                hi = mid;
            }
            None => {
                // Clean at mid — search upper half
                lo = mid + 1;
            }
        }
    }

    // Final verification: check frame lo with per-frame granularity
    // (lo might be the exact divergence frame)
    if lo == hi && lo != best_frame {
        let checkpoints = vec![DynamicCheckpoint {
            label: format!("frame_{lo}"),
            frame_index: lo,
        }];
        let result = probe_tas(
            repo_root,
            rom_path,
            manifest,
            &scenario,
            start,
            lo + 1,
            &checkpoints,
            frontier_kind,
            include_rng,
        )?;
        if let Some(f) = result.first_blocking_frame {
            best_frame = f;
            best_detail = result.first_blocking_detail;
        }
    }

    println!();
    if let Some((field, native_val, oracle_val)) = best_detail {
        println!("  RESULT: first blocking divergence at frame {best_frame}");
        println!("    field: {field}");
        println!("    native: {native_val}");
        println!("    oracle: {oracle_val}");
    } else {
        println!("  RESULT: first blocking divergence at frame {best_frame}");
    }

    Ok(())
}

fn generate_checkpoints(start: usize, end: usize, interval: usize) -> Vec<DynamicCheckpoint> {
    let mut checkpoints = Vec::new();
    checkpoints.push(DynamicCheckpoint {
        label: format!("frame_{start}"),
        frame_index: start,
    });
    if interval > 0 {
        let first = start.saturating_add(interval);
        for next in (first..end).step_by(interval) {
            checkpoints.push(DynamicCheckpoint {
                label: format!("frame_{next}"),
                frame_index: next,
            });
        }
    }
    let last = end.saturating_sub(1);
    if checkpoints
        .last()
        .is_some_and(|checkpoint| checkpoint.frame_index < last)
    {
        checkpoints.push(DynamicCheckpoint {
            label: format!("frame_{last}"),
            frame_index: last,
        });
    }
    checkpoints
}

fn crc_oracle_scenario(mut scenario: DynamicScenario, end: usize) -> DynamicScenario {
    let tail_frame = end.saturating_sub(1);
    scenario.checkpoints = vec![DynamicCheckpoint {
        label: format!("frame_{tail_frame}"),
        frame_index: tail_frame,
    }];
    scenario
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{Cursor, Write as IoWrite};

    #[test]
    fn rng_divergence_digest_ratchets_every_rng_value_and_ignores_other_categories() {
        let rng_diff = crate::sweep::StateDiff {
            checkpoint: "frame_150".into(),
            field: "rng_ring".into(),
            native_value: "byte[0]=0xc4".into(),
            oracle_value: "byte[0]=0x1f".into(),
            category: crate::sweep::DivergenceCategory::Rng,
        };
        let authoritative_diff = crate::sweep::StateDiff {
            checkpoint: "frame_150".into(),
            field: "kid_x_pos".into(),
            native_value: "0x00000000".into(),
            oracle_value: "0x00010000".into(),
            category: crate::sweep::DivergenceCategory::Authoritative,
        };

        let digest = rng_divergence_digest(std::slice::from_ref(&rng_diff));
        assert_eq!(
            rng_divergence_digest(&[rng_diff.clone(), authoritative_diff]),
            digest,
            "unrelated known checkpoint debt must not perturb the RNG ratchet",
        );

        let mut changed = rng_diff;
        changed.oracle_value = "byte[0]=0x20".into();
        assert_ne!(rng_divergence_digest(&[changed]), digest);
        assert_eq!(rng_divergence_digest(&[]), None);
    }

    #[test]
    fn frontier_diff_report_exposes_controlled_enemy_divergence() {
        let diff = crate::sweep::StateDiff {
            checkpoint: "frame_1022".into(),
            field: "enemy_roster".into(),
            native_value: String::new(),
            oracle_value: "70:00480000,02400000:Left".into(),
            category: crate::sweep::DivergenceCategory::Authoritative,
        };

        let report = frontier_diff_report(std::slice::from_ref(&diff), 37_759);

        assert_eq!(report["checkpoints"], 37_759);
        assert_eq!(report["total_diffs"], 1);
        assert_eq!(report["divergence_summary"]["authoritative"], 1);
        assert_eq!(
            report["first_authoritative_diff"],
            serde_json::json!({
                "checkpoint": "frame_1022",
                "field": "enemy_roster",
                "native_value": "",
                "oracle_value": "70:00480000,02400000:Left",
            })
        );
    }

    fn rng_frame(offset: u16, byte: u8) -> Option<wildside_core::RomRandomFrameState> {
        wildside_core::RomRandomFrameState::from_initialised_parts(
            offset,
            wildside_core::RomRandomRing::from_bytes([byte; wildside_core::RomRandomRing::LENGTH]),
        )
    }

    #[test]
    fn per_frame_rng_comparison_starts_when_both_sides_are_initialised_loc_1186() {
        let native = vec![None, rng_frame(0xFFED, 0x11), rng_frame(0xFFEE, 0x22)];
        let oracle = vec![rng_frame(0xFFC9, 0x99), None, rng_frame(0xFFEE, 0x22)];

        let comparison = compare_rng_frame_streams(&native, &oracle, 0);

        assert_eq!(comparison.start_frame, Some(2));
        assert_eq!(comparison.compared_frames, 1);
        assert_eq!(comparison.mismatch_count, 0);
        assert_eq!(comparison.first_mismatch, None);
        assert_eq!(comparison.mismatch_digest, None);
    }

    #[test]
    fn per_frame_rng_comparison_ratchets_full_state_and_first_byte() {
        let native = vec![
            None,
            rng_frame(0xFFED, 0x11),
            rng_frame(0xFFEE, 0x22),
            rng_frame(0xFFEF, 0x33),
        ];
        let oracle = vec![
            None,
            rng_frame(0xFFED, 0x11),
            rng_frame(0xFFEF, 0x22),
            rng_frame(0xFFEF, 0x44),
        ];

        let comparison = compare_rng_frame_streams(&native, &oracle, 0);

        assert_eq!(comparison.start_frame, Some(1));
        assert_eq!(comparison.compared_frames, 3);
        assert_eq!(comparison.mismatch_count, 2);
        assert_eq!(
            comparison.first_mismatch,
            Some(RngFrameMismatch {
                frame_index: 2,
                native_offset: Some(0xFFEE),
                oracle_offset: Some(0xFFEF),
                first_ring_difference: None,
            }),
        );
        assert!(comparison.mismatch_digest.is_some());

        let mut moved = oracle;
        moved[3] = rng_frame(0xFFEF, 0x45);
        assert_ne!(
            compare_rng_frame_streams(&native, &moved, 0).mismatch_digest,
            comparison.mismatch_digest,
            "digest must include every byte, not only the first reported mismatch",
        );
    }

    #[test]
    fn native_rng_stream_rejects_initialised_frame_without_complete_ring() {
        let frame = wildside_core::NativeTraceFrame {
            frame_index: 150,
            rng_offset: Some(0xFFED),
            rng_ring: None,
            ..wildside_core::NativeTraceFrame::default()
        };

        let error = extract_native_rng_frame_stream(&[frame])
            .expect_err("missing typed ring must make per-frame evidence incomplete");
        assert!(error.to_string().contains("frame 150"));
        assert!(error.to_string().contains("complete RNG ring"));
    }

    /// Build a minimal BK2 ZIP in memory for testing.
    fn build_test_bk2(header: &str, input_log: &str) -> Vec<u8> {
        let buf = Vec::new();
        let cursor = Cursor::new(buf);
        let mut zip = zip::ZipWriter::new(cursor);
        let options = zip::write::SimpleFileOptions::default()
            .compression_method(zip::CompressionMethod::Stored);

        zip.start_file("Header.txt", options).unwrap();
        zip.write_all(header.as_bytes()).unwrap();

        zip.start_file("Input Log.txt", options).unwrap();
        zip.write_all(input_log.as_bytes()).unwrap();

        let cursor = zip.finish().unwrap();
        cursor.into_inner()
    }

    const GENESIS_HEADER: &str = "Platform GEN\nGameName Kid Chameleon\n";

    const STANDARD_LOGKEY: &str = concat!(
        "LogKey:#Power|Reset|",
        "#P1 Up|P1 Down|P1 Left|P1 Right|P1 A|P1 B|P1 C|P1 Start|",
        "P1 X|P1 Y|P1 Z|P1 Mode|",
        "#P2 Up|P2 Down|P2 Left|P2 Right|P2 A|P2 B|P2 C|P2 Start|",
        "P2 X|P2 Y|P2 Z|P2 Mode|"
    );

    fn standard_input_log(frames: &[&str]) -> String {
        let mut log = String::from("[Input]\n");
        log.push_str(STANDARD_LOGKEY);
        log.push('\n');
        for frame in frames {
            log.push_str(frame);
            log.push('\n');
        }
        log.push_str("[/Input]\n");
        log
    }

    fn dynamic_scenario(
        frame_count: usize,
        checkpoints: Vec<DynamicCheckpoint>,
    ) -> DynamicScenario {
        DynamicScenario {
            slug: "test".into(),
            script: vec![0; frame_count],
            frame_count,
            checkpoints,
            start_position: None,
            start_velocity: None,
            camera_position: None,
            playtest_form: None,
            initial_diamond_count: None,
            initial_lives: None,
            warp_to_level: None,
            settle_oracle_warp: false,
        }
    }

    #[test]
    fn parse_empty_input_produces_empty_script() {
        let input_log = standard_input_log(&[]);
        let bk2 = build_test_bk2(GENESIS_HEADER, &input_log);
        let import = parse_bk2_from_reader(Cursor::new(bk2), "test.bk2").expect("parse_bk2");
        assert_eq!(import.total_frames, 0);
        assert!(import.script.is_empty());
    }

    #[test]
    fn parse_idle_frame_produces_zero_mask() {
        let input_log = standard_input_log(&["|..|............|............|"]);
        let bk2 = build_test_bk2(GENESIS_HEADER, &input_log);
        let import = parse_bk2_from_reader(Cursor::new(bk2), "test.bk2").expect("parse_bk2");
        assert_eq!(import.total_frames, 1);
        assert_eq!(import.script[0], 0);
    }

    #[test]
    fn parse_start_button_maps_to_bit_7() {
        // Start is column index 7 in P1 section: U D L R A B C S
        let input_log = standard_input_log(&["|..|.......S....|............|"]);
        let bk2 = build_test_bk2(GENESIS_HEADER, &input_log);
        let import = parse_bk2_from_reader(Cursor::new(bk2), "test.bk2").expect("parse_bk2");
        assert_eq!(import.script[0], 0x80); // bit 7 = Start
    }

    #[test]
    fn parse_right_button_maps_to_bit_3() {
        // Right is column index 3: U D L R
        let input_log = standard_input_log(&["|..|...R........|............|"]);
        let bk2 = build_test_bk2(GENESIS_HEADER, &input_log);
        let import = parse_bk2_from_reader(Cursor::new(bk2), "test.bk2").expect("parse_bk2");
        assert_eq!(import.script[0], 1 << 3); // bit 3 = Right
    }

    #[test]
    fn parse_b_button_maps_to_bit_4() {
        // B is column index 5 in P1 section: U D L R A B C S
        let input_log = standard_input_log(&["|..|.....B......|............|"]);
        let bk2 = build_test_bk2(GENESIS_HEADER, &input_log);
        let import = parse_bk2_from_reader(Cursor::new(bk2), "test.bk2").expect("parse_bk2");
        assert_eq!(import.script[0], 1 << 4); // bit 4 = B (jump)
    }

    #[test]
    fn parse_c_button_maps_to_bit_5() {
        // C is column index 6 in P1 section: U D L R A B C S
        let input_log = standard_input_log(&["|..|......C.....|............|"]);
        let bk2 = build_test_bk2(GENESIS_HEADER, &input_log);
        let import = parse_bk2_from_reader(Cursor::new(bk2), "test.bk2").expect("parse_bk2");
        assert_eq!(import.script[0], 1 << 5); // bit 5 = C (run)
    }

    #[test]
    fn parse_a_button_maps_to_bit_6() {
        // A is column index 4 in P1 section: U D L R A B C S
        let input_log = standard_input_log(&["|..|....A.......|............|"]);
        let bk2 = build_test_bk2(GENESIS_HEADER, &input_log);
        let import = parse_bk2_from_reader(Cursor::new(bk2), "test.bk2").expect("parse_bk2");
        assert_eq!(import.script[0], 1 << 6); // bit 6 = A
    }

    #[test]
    fn parse_multiple_buttons_combined() {
        // Right + B + Start: U D L R A B C S
        let input_log = standard_input_log(&["|..|...R.B.S....|............|"]);
        let bk2 = build_test_bk2(GENESIS_HEADER, &input_log);
        let import = parse_bk2_from_reader(Cursor::new(bk2), "test.bk2").expect("parse_bk2");
        let expected = (1 << 3) | (1 << 4) | (1 << 7); // Right + B + Start
        assert_eq!(import.script[0], expected);
    }

    #[test]
    fn parse_multiple_frames() {
        let input_log = standard_input_log(&[
            "|..|............|............|", // idle
            "|..|.......S....|............|", // Start
            "|..|...R........|............|", // Right
        ]);
        let bk2 = build_test_bk2(GENESIS_HEADER, &input_log);
        let import = parse_bk2_from_reader(Cursor::new(bk2), "test.bk2").expect("parse_bk2");
        assert_eq!(import.total_frames, 3);
        assert_eq!(import.script[0], 0);
        assert_eq!(import.script[1], 0x80);
        assert_eq!(import.script[2], 1 << 3);
    }

    #[test]
    fn parse_dpad_buttons() {
        // Up + Down + Left
        let input_log = standard_input_log(&["|..|UDL.........|............|"]);
        let bk2 = build_test_bk2(GENESIS_HEADER, &input_log);
        let import = parse_bk2_from_reader(Cursor::new(bk2), "test.bk2").expect("parse_bk2");
        let expected = (1 << 0) | (1 << 1) | (1 << 2); // Up + Down + Left
        assert_eq!(import.script[0], expected);
    }

    #[test]
    fn reject_non_genesis_platform() {
        let header = "Platform SNES\nGameName SomeGame\n";
        let input_log = standard_input_log(&[]);
        let bk2 = build_test_bk2(header, &input_log);
        let result = parse_bk2_from_reader(Cursor::new(bk2), "test.bk2");
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("unsupported platform"), "got: {err}");
    }

    #[test]
    fn reject_malformed_zip() {
        let result = parse_bk2_from_reader(Cursor::new(vec![0, 1, 2, 3]), "bad.bk2");
        assert!(result.is_err());
    }

    #[test]
    fn slug_sanitised_from_filename() {
        let input_log = standard_input_log(&["|..|............|............|"]);
        let bk2 = build_test_bk2(GENESIS_HEADER, &input_log);
        let import = parse_bk2_from_reader(Cursor::new(bk2), "Kid Chameleon (Any%) v2.bk2")
            .expect("parse_bk2");
        assert_eq!(import.slug, "Kid_Chameleon__Any___v2");
    }

    #[test]
    fn bk2_to_scenario_creates_checkpoints_at_interval() {
        let import = Bk2Import {
            slug: "test_tas".into(),
            total_frames: 1000,
            script: vec![0; 1000],
        };
        let scenario = bk2_to_scenario(&import, 300);
        assert_eq!(scenario.slug, "test_tas");
        assert_eq!(scenario.frame_count, 1000);
        assert!(scenario.warp_to_level.is_none());
        let labels: Vec<&str> = scenario
            .checkpoints
            .iter()
            .map(|c| c.label.as_str())
            .collect();
        assert_eq!(
            labels,
            vec![
                "frame_0",
                "frame_300",
                "frame_600",
                "frame_900",
                "frame_999"
            ]
        );
    }

    #[test]
    fn cold_boot_scenario_has_no_verify_tas_advisory() {
        // The published-TAS shape (warp_to_level: None) is authoritative. #7.
        let import = Bk2Import {
            slug: "cold_boot".into(),
            total_frames: 100,
            script: vec![0; 100],
        };
        let scenario = bk2_to_scenario(&import, 300);
        assert!(scenario.warp_to_level.is_none());
        assert!(cold_boot_advisory(&scenario).is_none());
    }

    #[test]
    fn warped_scenario_emits_verify_tas_advisory() {
        // A synthetic warp is flagged non-authoritative (advisory, not an error
        // — committed warped probes run through this gate). #7.
        let import = Bk2Import {
            slug: "warped".into(),
            total_frames: 100,
            script: vec![0; 100],
        };
        let mut scenario = bk2_to_scenario(&import, 300);
        scenario.warp_to_level = Some(0x03);
        let advisory = cold_boot_advisory(&scenario)
            .expect("warped scenario must emit a non-authoritative advisory");
        assert!(
            advisory.contains("warp_to_level"),
            "message was: {advisory}"
        );
        assert!(advisory.contains("issue #7"), "message was: {advisory}");
    }

    #[test]
    fn bk2_to_scenario_small_script_has_start_and_end() {
        let import = Bk2Import {
            slug: "short".into(),
            total_frames: 50,
            script: vec![0; 50],
        };
        let scenario = bk2_to_scenario(&import, 300);
        let labels: Vec<&str> = scenario
            .checkpoints
            .iter()
            .map(|c| c.label.as_str())
            .collect();
        assert_eq!(labels, vec!["frame_0", "frame_49"]);
    }

    #[test]
    fn bk2_to_scenario_does_not_duplicate_exact_interval_tail() {
        let import = Bk2Import {
            slug: "exact".into(),
            total_frames: 301,
            script: vec![0; 301],
        };
        let scenario = bk2_to_scenario(&import, 300);
        let indices: Vec<usize> = scenario
            .checkpoints
            .iter()
            .map(|checkpoint| checkpoint.frame_index)
            .collect();

        assert_eq!(indices, vec![0, 300]);
    }

    #[test]
    fn bk2_to_scenario_keeps_checkpoints_inside_frame_count() {
        let import = Bk2Import {
            slug: "bounded".into(),
            total_frames: 600,
            script: vec![0; 600],
        };
        let scenario = bk2_to_scenario(&import, 300);
        let indices: Vec<usize> = scenario
            .checkpoints
            .iter()
            .map(|checkpoint| checkpoint.frame_index)
            .collect();

        assert_eq!(indices, vec![0, 300, 599]);
    }

    #[test]
    fn bk2_to_scenario_zero_interval_keeps_start_and_tail_only() {
        let import = Bk2Import {
            slug: "zero_interval".into(),
            total_frames: 600,
            script: vec![0; 600],
        };
        let scenario = bk2_to_scenario(&import, 0);
        let indices: Vec<usize> = scenario
            .checkpoints
            .iter()
            .map(|checkpoint| checkpoint.frame_index)
            .collect();

        assert_eq!(indices, vec![0, 599]);
    }

    #[test]
    fn merge_transition_checkpoints_inserts_pre_and_post() {
        let mut scenario = DynamicScenario {
            slug: "test".into(),
            script: vec![0; 1000],
            frame_count: 1000,
            checkpoints: vec![
                DynamicCheckpoint {
                    label: "frame_0".into(),
                    frame_index: 0,
                },
                DynamicCheckpoint {
                    label: "frame_300".into(),
                    frame_index: 300,
                },
            ],
            start_position: None,
            start_velocity: None,
            camera_position: None,
            playtest_form: None,
            initial_diamond_count: None,
            initial_lives: None,
            warp_to_level: None,
            settle_oracle_warp: false,
        };
        let transitions = vec![
            (
                500,
                wildside_core::LevelId(0x00),
                wildside_core::LevelId(0x01),
            ),
            (
                800,
                wildside_core::LevelId(0x01),
                wildside_core::LevelId(0x02),
            ),
        ];
        super::merge_transition_checkpoints(&mut scenario, &transitions, 0, 1000);

        let indices: Vec<usize> = scenario
            .checkpoints
            .iter()
            .map(|cp| cp.frame_index)
            .collect();
        // Should have: 0, 300, 500, 501, 800, 801
        assert_eq!(indices, vec![0, 300, 500, 501, 800, 801]);

        // Verify labels
        assert!(
            scenario.checkpoints[2]
                .label
                .contains("transition_00_to_01")
        );
        assert!(
            scenario.checkpoints[4]
                .label
                .contains("transition_01_to_02")
        );
    }

    #[test]
    fn merge_transition_checkpoints_gives_repeated_routes_unique_labels() {
        let mut scenario = DynamicScenario {
            slug: "repeated_transition".into(),
            script: vec![0; 1000],
            frame_count: 1000,
            checkpoints: Vec::new(),
            start_position: None,
            start_velocity: None,
            camera_position: None,
            playtest_form: None,
            initial_diamond_count: None,
            initial_lives: None,
            warp_to_level: None,
            settle_oracle_warp: false,
        };
        let transitions = vec![
            (
                100,
                wildside_core::LevelId(0x15),
                wildside_core::LevelId(0x00),
            ),
            (
                800,
                wildside_core::LevelId(0x15),
                wildside_core::LevelId(0x00),
            ),
        ];

        super::merge_transition_checkpoints(&mut scenario, &transitions, 0, 1000);

        let labels = scenario
            .checkpoints
            .iter()
            .map(|checkpoint| checkpoint.label.as_str())
            .collect::<Vec<_>>();
        assert_eq!(
            labels,
            vec![
                "transition_15_to_00_pre_frame_100",
                "transition_15_to_00_post_frame_101",
                "transition_15_to_00_pre_frame_800",
                "transition_15_to_00_post_frame_801",
            ]
        );
    }

    #[test]
    fn merge_transition_checkpoints_deduplicates_existing() {
        let mut scenario = DynamicScenario {
            slug: "test".into(),
            script: vec![0; 500],
            frame_count: 500,
            checkpoints: vec![DynamicCheckpoint {
                label: "frame_200".into(),
                frame_index: 200,
            }],
            start_position: None,
            start_velocity: None,
            camera_position: None,
            playtest_form: None,
            initial_diamond_count: None,
            initial_lives: None,
            warp_to_level: None,
            settle_oracle_warp: false,
        };
        // Transition at frame 200 — already a checkpoint
        let transitions = vec![(
            200,
            wildside_core::LevelId(0x00),
            wildside_core::LevelId(0x01),
        )];
        super::merge_transition_checkpoints(&mut scenario, &transitions, 0, 500);

        let indices: Vec<usize> = scenario
            .checkpoints
            .iter()
            .map(|cp| cp.frame_index)
            .collect();
        // frame 200 already existed, 201 is new
        assert_eq!(indices, vec![200, 201]);
    }

    #[test]
    fn merge_transition_checkpoints_ignores_transitions_outside_slice() {
        let mut scenario = dynamic_scenario(
            1000,
            vec![DynamicCheckpoint {
                label: "frame_780".into(),
                frame_index: 780,
            }],
        );
        let transitions = vec![
            (
                500,
                wildside_core::LevelId(0x00),
                wildside_core::LevelId(0x01),
            ),
            (
                800,
                wildside_core::LevelId(0x01),
                wildside_core::LevelId(0x02),
            ),
            (
                900,
                wildside_core::LevelId(0x02),
                wildside_core::LevelId(0x03),
            ),
        ];
        super::merge_transition_checkpoints(&mut scenario, &transitions, 780, 850);

        let indices: Vec<usize> = scenario
            .checkpoints
            .iter()
            .map(|cp| cp.frame_index)
            .collect();
        assert_eq!(indices, vec![780, 800, 801]);
    }

    #[test]
    fn merge_transition_checkpoints_includes_start_and_excludes_end() {
        let mut scenario = dynamic_scenario(100, Vec::new());
        let transitions = vec![
            (
                10,
                wildside_core::LevelId(0x00),
                wildside_core::LevelId(0x01),
            ),
            (
                20,
                wildside_core::LevelId(0x01),
                wildside_core::LevelId(0x02),
            ),
        ];

        super::merge_transition_checkpoints(&mut scenario, &transitions, 10, 20);

        let indices: Vec<usize> = scenario
            .checkpoints
            .iter()
            .map(|checkpoint| checkpoint.frame_index)
            .collect();
        assert_eq!(indices, vec![10, 11]);
    }

    #[test]
    fn merge_transition_checkpoints_rejects_scenario_frame_count_boundary() {
        let mut scenario = dynamic_scenario(50, Vec::new());
        let transitions = vec![
            (
                49,
                wildside_core::LevelId(0x00),
                wildside_core::LevelId(0x01),
            ),
            (
                50,
                wildside_core::LevelId(0x01),
                wildside_core::LevelId(0x02),
            ),
        ];

        super::merge_transition_checkpoints(&mut scenario, &transitions, 0, 60);

        let indices: Vec<usize> = scenario
            .checkpoints
            .iter()
            .map(|checkpoint| checkpoint.frame_index)
            .collect();
        assert_eq!(indices, vec![49]);
    }

    #[test]
    fn generate_checkpoints_covers_range() {
        let cps = super::generate_checkpoints(100, 500, 150);
        let indices: Vec<usize> = cps.iter().map(|cp| cp.frame_index).collect();
        assert_eq!(indices, vec![100, 250, 400, 499]);
    }

    #[test]
    fn generate_checkpoints_does_not_duplicate_exact_interval_tail() {
        let cps = super::generate_checkpoints(100, 401, 300);
        let indices: Vec<usize> = cps
            .iter()
            .map(|checkpoint| checkpoint.frame_index)
            .collect();

        assert_eq!(indices, vec![100, 400]);
    }

    #[test]
    fn generate_checkpoints_keeps_end_exclusive() {
        let cps = super::generate_checkpoints(100, 700, 300);
        let indices: Vec<usize> = cps
            .iter()
            .map(|checkpoint| checkpoint.frame_index)
            .collect();

        assert_eq!(indices, vec![100, 400, 699]);
    }

    #[test]
    fn generate_checkpoints_zero_interval_keeps_start_and_tail_only() {
        let cps = super::generate_checkpoints(100, 700, 0);
        let indices: Vec<usize> = cps
            .iter()
            .map(|checkpoint| checkpoint.frame_index)
            .collect();

        assert_eq!(indices, vec![100, 699]);
    }

    #[test]
    fn crc_oracle_scenario_replaces_checkpoints_with_tail_frame() {
        let scenario = dynamic_scenario(
            900,
            vec![DynamicCheckpoint {
                label: "old".into(),
                frame_index: 123,
            }],
        );

        let crc_scenario = super::crc_oracle_scenario(scenario, 450);

        assert_eq!(crc_scenario.frame_count, 900);
        assert_eq!(crc_scenario.script.len(), 900);
        assert_eq!(crc_scenario.checkpoints.len(), 1);
        assert_eq!(crc_scenario.checkpoints[0].label, "frame_449");
        assert_eq!(crc_scenario.checkpoints[0].frame_index, 449);
    }

    #[test]
    fn first_crc_mismatch_from_frame_skips_earlier_divergences() {
        let native = [11, 22, 33, 44, 55];
        let oracle = [99, 22, 33, 88, 55];

        let mismatch = super::first_crc_mismatch_from_frame(&native, &oracle, 1, 5)
            .expect("mismatch after start");

        assert_eq!(mismatch, 3);
    }

    #[test]
    fn first_crc_mismatch_from_frame_returns_none_when_tail_matches() {
        let native = [11, 22, 33, 44, 55];
        let oracle = [99, 22, 33, 44, 55];

        let mismatch = super::first_crc_mismatch_from_frame(&native, &oracle, 1, 5);

        assert_eq!(mismatch, None);
    }

    #[test]
    fn first_crc_mismatch_from_frame_rejects_truncated_oracle_tail() {
        let native = [11, 22, 33];
        let oracle = [11, 22];

        let mismatch = super::first_crc_mismatch_from_frame(&native, &oracle, 0, 3);

        assert_eq!(mismatch, Some(2));
    }

    #[test]
    fn first_crc_mismatch_from_frame_rejects_equally_truncated_streams() {
        let native = [11, 22];
        let oracle = [11, 22];

        let mismatch = super::first_crc_mismatch_from_frame(&native, &oracle, 0, 3);

        assert_eq!(mismatch, Some(2));
    }

    #[test]
    fn crc_clean_velocity_diffs_are_oracle_limitations() {
        let mut diff = crate::sweep::StateDiff {
            checkpoint: "frame_904".into(),
            field: "kid_y_vel".into(),
            native_value: "0x00000000".into(),
            oracle_value: "0x00004000".into(),
            category: crate::sweep::DivergenceCategory::Authoritative,
        };
        let native = vec![demo_state("frame_904", 0x0020_0000, 0x02de_4000)];
        let oracle = vec![demo_state("frame_904", 0x0020_0000, 0x02de_4000)];

        super::reclassify_crc_clean_velocity_diffs(
            std::slice::from_mut(&mut diff),
            &native,
            &oracle,
            true,
        );

        assert_eq!(
            diff.category,
            crate::sweep::DivergenceCategory::OracleLimitation
        );
    }

    #[test]
    fn crc_clean_live_gameplay_velocity_diffs_are_oracle_limitations() {
        let mut diff = crate::sweep::StateDiff {
            checkpoint: "frame_625".into(),
            field: "kid_y_vel".into(),
            native_value: "0x00000000".into(),
            oracle_value: "0xfffe0000".into(),
            category: crate::sweep::DivergenceCategory::Authoritative,
        };
        let mut native = demo_state("frame_625", 0x0420_1c00, 0x009f_0000);
        let mut oracle = demo_state("frame_625", 0x0420_1c00, 0x009f_0000);
        native.game_mode = 0x000c;
        native.boot_phase = Some("in_game".into());
        oracle.game_mode = 0x000c;
        oracle.boot_phase = Some("in_game".into());

        super::reclassify_crc_clean_velocity_diffs(
            std::slice::from_mut(&mut diff),
            &[native],
            &[oracle],
            true,
        );

        assert_eq!(
            diff.category,
            crate::sweep::DivergenceCategory::OracleLimitation,
            "CRC-clean #211 frame 625 has matching Kid position and pixels; the isolated top-level y-velocity skew is an oracle export limitation, not movement drift"
        );
    }

    #[test]
    fn crc_clean_position_drift_keeps_velocity_authoritative() {
        let mut diff = crate::sweep::StateDiff {
            checkpoint: "frame_904".into(),
            field: "kid_y_vel".into(),
            native_value: "0x00000000".into(),
            oracle_value: "0x00004000".into(),
            category: crate::sweep::DivergenceCategory::Authoritative,
        };
        let native = vec![demo_state("frame_904", 0x0020_0000, 0x02de_4000)];
        let oracle = vec![demo_state("frame_904", 0x0020_0000, 0x02df_4000)];

        super::reclassify_crc_clean_velocity_diffs(
            std::slice::from_mut(&mut diff),
            &native,
            &oracle,
            true,
        );

        assert_eq!(
            diff.category,
            crate::sweep::DivergenceCategory::Authoritative
        );
    }

    #[test]
    fn crc_clean_lion_projectile_count_diffs_are_oracle_limitations_loc_3e12a() {
        let mut diff = crate::sweep::StateDiff {
            checkpoint: "frame_29".into(),
            field: "number_objects".into(),
            native_value: "7".into(),
            oracle_value: "6".into(),
            category: crate::sweep::DivergenceCategory::Authoritative,
        };
        let mut native = demo_state("frame_29", 0x01c0_0000, 0x02bf_0000);
        native.game_mode = 0x000c;
        native.boot_phase = Some("in_game".into());
        native.current_level_id = 7;
        native.player_1_level_id = 7;
        native.number_objects = 7;
        native
            .gameplay_debug_fields
            .insert("lion_active_projectiles".into(), "1".into());
        let mut oracle = demo_state("frame_29", 0x01c0_0000, 0x02bf_0000);
        oracle.game_mode = 0x000c;
        oracle.boot_phase = Some("in_game".into());
        oracle.current_level_id = 7;
        oracle.player_1_level_id = 7;
        oracle.number_objects = 6;
        oracle.gameplay_debug_fields.insert(
            "oracle_object_slots_loc_e1e".into(),
            "0xffff0f70:code=0x03e196,hib=1@441,686".into(),
        );

        super::reclassify_crc_clean_lion_projectile_count_diffs(
            std::slice::from_mut(&mut diff),
            &[native],
            &[oracle],
            true,
        );

        assert_eq!(
            diff.category,
            crate::sweep::DivergenceCategory::OracleLimitation,
            "ASM Lion.asm loc_3E12A allocates a child object, but issue289 Level 0x07 CRC-clean frames expose the live loc_3E196 slot while oracle Number_Objects is one lower"
        );
    }

    #[test]
    fn crc_clean_lion_projectile_tail_count_diffs_are_oracle_limitations_loc_3e12a() {
        let mut diff = crate::sweep::StateDiff {
            checkpoint: "frame_30".into(),
            field: "number_objects".into(),
            native_value: "7".into(),
            oracle_value: "6".into(),
            category: crate::sweep::DivergenceCategory::Authoritative,
        };
        let mut native = demo_state("frame_30", 0x01c0_0000, 0x02bf_0000);
        native.game_mode = 0x000c;
        native.boot_phase = Some("in_game".into());
        native.current_level_id = 7;
        native.player_1_level_id = 7;
        native.number_objects = 7;
        native.active_enemies.push(lion_enemy());
        let mut oracle = demo_state("frame_30", 0x01c0_0000, 0x02bf_0000);
        oracle.game_mode = 0x000c;
        oracle.boot_phase = Some("in_game".into());
        oracle.current_level_id = 7;
        oracle.player_1_level_id = 7;
        oracle.number_objects = 6;
        oracle.gameplay_debug_fields.insert(
            "oracle_object_slots_loc_e1e".into(),
            "0xffff0f70:code=0x03e196,hib=1@441,686".into(),
        );

        super::reclassify_crc_clean_lion_projectile_count_diffs(
            std::slice::from_mut(&mut diff),
            &[native],
            &[oracle],
            true,
        );

        assert_eq!(
            diff.category,
            crate::sweep::DivergenceCategory::OracleLimitation,
            "ASM Lion.asm loc_3E12A child slot remains visible after native lion_active_projectiles clears; CRC-clean Level 0x07 frames should not fail on oracle Number_Objects export skew"
        );
    }

    #[test]
    fn crc_clean_lion_projectile_bounce_loop_count_diffs_are_oracle_limitations_loc_3e2a8() {
        let mut diff = crate::sweep::StateDiff {
            checkpoint: "frame_38".into(),
            field: "number_objects".into(),
            native_value: "7".into(),
            oracle_value: "6".into(),
            category: crate::sweep::DivergenceCategory::Authoritative,
        };
        let mut native = demo_state("frame_38", 0x01c0_0000, 0x02bf_0000);
        native.game_mode = 0x000c;
        native.boot_phase = Some("in_game".into());
        native.current_level_id = 7;
        native.player_1_level_id = 7;
        native.number_objects = 7;
        native.active_enemies.push(lion_enemy());
        let mut oracle = demo_state("frame_38", 0x01c0_0000, 0x02bf_0000);
        oracle.game_mode = 0x000c;
        oracle.boot_phase = Some("in_game".into());
        oracle.current_level_id = 7;
        oracle.player_1_level_id = 7;
        oracle.number_objects = 6;
        oracle.gameplay_debug_fields.insert(
            "oracle_object_slots_loc_e1e".into(),
            "0xffff0f70:code=0x03e2ac,hib=1@393,696".into(),
        );

        super::reclassify_crc_clean_lion_projectile_count_diffs(
            std::slice::from_mut(&mut diff),
            &[native],
            &[oracle],
            true,
        );

        assert_eq!(
            diff.category,
            crate::sweep::DivergenceCategory::OracleLimitation,
            "ASM Lion.asm loc_3E2A8 hibernates inside the projectile bounce loop, so oracle can expose saved return PC 0x03e2ac while Number_Objects is one lower"
        );
    }

    #[test]
    fn crc_clean_pause_latch_diffs_are_oracle_limitations_loc_6b62() {
        let mut diff = crate::sweep::StateDiff {
            checkpoint: "frame_15".into(),
            field: "character_movement".into(),
            native_value: "0x0001".into(),
            oracle_value: "0x0000".into(),
            category: crate::sweep::DivergenceCategory::Authoritative,
        };
        let mut native = demo_state("frame_15", 0x0020_0000, 0x010f_0000);
        native.game_mode = 0x000c;
        native.boot_phase = Some("in_game".into());
        native.current_level_id = 0;
        native.player_1_level_id = 0;
        native.number_lives = 1;
        native.game_paused = true;
        native.character_movement = Some(1);
        let mut oracle = native.clone();
        oracle.character_movement = Some(0);

        super::reclassify_crc_clean_pause_trace_latch_diffs(
            std::slice::from_mut(&mut diff),
            &[native],
            &[oracle],
            true,
        );

        assert_eq!(
            diff.category,
            crate::sweep::DivergenceCategory::OracleLimitation,
            "ASM Game_Paused_Loop loc_6B62/loc_6C0A can leave CRC-clean pause probes with only Character_Movement latch skew between native post-frame and oracle VBlank exports"
        );
    }

    #[test]
    fn crc_clean_pause_flag_latch_diffs_are_oracle_limitations_loc_6e2() {
        let mut diff = crate::sweep::StateDiff {
            checkpoint: "frame_2801".into(),
            field: "game_paused".into(),
            native_value: "true".into(),
            oracle_value: "false".into(),
            category: crate::sweep::DivergenceCategory::Authoritative,
        };
        let mut native = demo_state("frame_2801", 0x0020_0000, 0x010f_0000);
        native.game_mode = 0x000c;
        native.boot_phase = Some("in_game".into());
        native.current_level_id = 0;
        native.player_1_level_id = 0;
        native.game_paused = true;
        let mut oracle = native.clone();
        oracle.game_paused = false;

        super::reclassify_crc_clean_pause_trace_latch_diffs(
            std::slice::from_mut(&mut diff),
            &[native],
            &[oracle],
            true,
        );

        assert_eq!(
            diff.category,
            crate::sweep::DivergenceCategory::OracleLimitation,
            "ASM loc_6E2 pause-shell restore can be CRC-clean while the exported Game_Paused latch differs by a sampling boundary"
        );
    }

    #[test]
    fn pause_latch_reclassification_requires_matching_core_state() {
        let mut diff = crate::sweep::StateDiff {
            checkpoint: "frame_15".into(),
            field: "character_movement".into(),
            native_value: "0x0001".into(),
            oracle_value: "0x0000".into(),
            category: crate::sweep::DivergenceCategory::Authoritative,
        };
        let mut native = demo_state("frame_15", 0x0020_0000, 0x010f_0000);
        native.game_mode = 0x000c;
        native.boot_phase = Some("in_game".into());
        native.current_level_id = 0;
        native.player_1_level_id = 0;
        native.character_movement = Some(1);
        let mut oracle = native.clone();
        oracle.character_movement = Some(0);
        oracle.kid_y_pos = oracle.kid_y_pos.wrapping_add(0x0001_0000);

        super::reclassify_crc_clean_pause_trace_latch_diffs(
            std::slice::from_mut(&mut diff),
            &[native],
            &[oracle],
            true,
        );

        assert_eq!(
            diff.category,
            crate::sweep::DivergenceCategory::Authoritative,
            "real Kid/camera/state drift must still block even if a caller claims the CRC window is clean"
        );
    }

    fn lion_enemy() -> crate::oracle::OracleEnemyState {
        crate::oracle::OracleEnemyState {
            object_type: 0x12,
            x_pos: 0x01c0_0000,
            y_pos: 0x02bf_0000,
            x_velocity: 0,
            y_velocity: 0,
            facing: crate::oracle::EnemyFacing::Left,
            collision_type: None,
            collision_source: None,
            layout_index: None,
            layout_ptr: None,
            parent_object_ptr: None,
            gfx_slot_index: None,
            on_ground: None,
            current_hp: Some(2),
            debug_fields: std::collections::BTreeMap::new(),
        }
    }

    fn demo_state(label: &str, kid_x_pos: u32, kid_y_pos: u32) -> OracleState {
        OracleState {
            label: label.into(),
            frame_index: 904,
            game_mode: 0x0010,
            boot_phase: Some("demo_play".into()),
            boot_phase_elapsed: None,
            title_screen_ready: false,
            main_menu_option: None,
            current_player: 0,
            two_player: false,
            number_hitpoints: 2,
            current_helmet_id: 0,
            current_level_id: 0,
            player_1_level_id: 0,
            camera_x_pos: 0,
            camera_y_pos: 0,
            kid_x_pos,
            kid_y_pos,
            number_objects: 5,
            number_gfx_objects: 4,
            game_paused: false,
            active_enemies: Vec::new(),
            score: 0,
            number_lives: 3,
            number_diamonds: 0,
            diamond_power_active: false,
            murder_wall_x: None,
            kid_x_vel: Some(0),
            kid_y_vel: Some(0),
            character_movement: Some(0),
            just_received_damage: Some(false),
            kid_collision_type: Some(0),
            kid_collision_source: Some(0),
            kid_x_direction: Some(0),
            invulnerable_frames: Some(0),
            currently_transforming: Some(false),
            rng_offset: Some(0),
            rng_last_byte: None,
            rng_ring: None,
            level_special_effects: Some(0),
            gameplay_debug_fields: std::collections::BTreeMap::new(),
            cpu_registers: None,
        }
    }

    #[test]
    fn round_trip_mask_encoding() {
        // Verify our masks match FrameInput::from_mask expectations
        use wildside_core::FrameInput;
        let test_cases: &[(u8, &str)] = &[
            (1 << 0, "up"),
            (1 << 1, "down"),
            (1 << 2, "left"),
            (1 << 3, "right"),
            (1 << 4, "b"),
            (1 << 5, "c"),
            (1 << 6, "a"),
            (1 << 7, "start"),
        ];
        for &(mask, expected_name) in test_cases {
            let input = FrameInput::from_mask(mask);
            let active = match expected_name {
                "up" => input.up,
                "down" => input.down,
                "left" => input.left,
                "right" => input.right,
                "a" => input.a,
                "b" => input.b,
                "c" => input.c,
                "start" => input.start,
                _ => unreachable!(),
            };
            assert!(active, "mask {mask:#04x} should activate {expected_name}");
        }
    }

    #[test]
    fn checked_in_boot_slice_fixtures_deserialise() {
        let fixtures = [
            include_str!("../tests/fixtures/tas/short-demo-boot-slice.json"),
            include_str!("../tests/fixtures/tas/kc-any-percent-2024-boot-slice.json"),
            include_str!("../tests/fixtures/tas/any-percent-published-boot-slice.json"),
        ];

        for fixture in fixtures {
            let scenario: DynamicScenario =
                serde_json::from_str(fixture).expect("fixture should deserialize");
            assert_eq!(scenario.frame_count, 250);
            assert_eq!(scenario.checkpoints.len(), 6);
            assert!(scenario.warp_to_level.is_none());
            assert_eq!(scenario.checkpoints[0].frame_index, 0);
            assert_eq!(
                scenario.checkpoints.last().map(|cp| cp.frame_index),
                Some(249)
            );
        }
    }
}
