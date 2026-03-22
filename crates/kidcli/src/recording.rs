use super::*;
use crate::diff::{annotate_diff, generate_diff_composite};
use crate::native_player::{
    NATIVE_FRAME_HEIGHT, NATIVE_FRAME_WIDTH, NativePlayOptions, build_scene, load_session_assets,
    render_scene,
};
use crate::oracle::{OracleFrame, OracleManifest, run_jgenesis_capture_dynamic};
use crate::scenarios::{DynamicCheckpoint, DynamicScenario};
use crate::verify::{compare_rgb_frames, load_png_rgb};
use image::{ColorType, ImageFormat};
use kid_core::{FrameInput, GameMode, LevelId, NativeRuntime};
use serde::{Deserialize, Serialize};

const RECORDING_CHECKPOINT_INTERVAL: usize = 60;
const RECORDING_LEVEL_CARD_HOLD_FRAMES: usize = 60;
const RECORDING_SETTLED_ORACLE_PREFIX_FRAMES: usize = 60;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct RecordingStart {
    pub(crate) warp_level: Option<u8>,
    pub(crate) settled: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct NativeRecordingManifest {
    pub(crate) slug: String,
    pub(crate) recorded_at: String,
    pub(crate) start: RecordingStart,
    pub(crate) scenario: DynamicScenario,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct NativeRecording {
    pub(crate) manifest: NativeRecordingManifest,
    pub(crate) trace: Vec<NativeTraceFrame>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub(crate) struct RecordingVerificationReport {
    pub(crate) slug: String,
    pub(crate) trace_match: bool,
    pub(crate) oracle_state_match: bool,
    pub(crate) oracle_frame_match: bool,
    pub(crate) replay_trace_len: usize,
    pub(crate) recorded_trace_len: usize,
    pub(crate) first_trace_drift: Option<TraceDriftReport>,
    pub(crate) frame_diffs: Vec<CheckpointFrameDiff>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct TraceDriftReport {
    pub(crate) frame_index: usize,
    pub(crate) reason: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub(crate) struct CheckpointFrameDiff {
    pub(crate) label: String,
    pub(crate) frame_index: usize,
    pub(crate) mean_absolute_error: f32,
    pub(crate) max_channel_delta: u8,
    pub(crate) diff_path: PathBuf,
}

pub(crate) fn recording_start_from_options(options: &NativePlayOptions) -> RecordingStart {
    RecordingStart {
        warp_level: options.warp_level,
        settled: options.settled,
    }
}

pub(crate) fn validate_recording_slug(slug: &str) -> Result<&str> {
    if slug.is_empty() {
        bail!("recording slug must not be empty");
    }
    if slug
        .bytes()
        .all(|byte| byte.is_ascii_alphanumeric() || byte == b'-' || byte == b'_' || byte == b'.')
    {
        Ok(slug)
    } else {
        bail!("recording slug must use only ASCII letters, digits, '.', '-', or '_'")
    }
}

pub(crate) fn build_recording(
    slug: &str,
    start: RecordingStart,
    script: Vec<u8>,
    trace: Vec<NativeTraceFrame>,
) -> NativeRecording {
    let frame_count = script.len();
    NativeRecording {
        manifest: NativeRecordingManifest {
            slug: slug.to_owned(),
            recorded_at: unix_timestamp(),
            start,
            scenario: DynamicScenario {
                slug: slug.to_owned(),
                checkpoints: recording_checkpoints(frame_count),
                frame_count,
                script,
                warp_to_level: start.warp_level,
            },
        },
        trace,
    }
}

pub(crate) fn save_recording(record_to: &Path, recording: &NativeRecording) -> Result<()> {
    if record_to.exists() {
        fs::remove_dir_all(record_to)
            .with_context(|| format!("failed to remove {}", record_to.display()))?;
    }
    fs::create_dir_all(record_to)
        .with_context(|| format!("failed to create {}", record_to.display()))?;
    fs::write(
        record_to.join("manifest.json"),
        serde_json::to_vec_pretty(&recording.manifest)?,
    )
    .with_context(|| {
        format!(
            "failed to write {}",
            record_to.join("manifest.json").display()
        )
    })?;
    fs::write(
        record_to.join("scenario.json"),
        serde_json::to_vec_pretty(&recording.manifest.scenario)?,
    )
    .with_context(|| {
        format!(
            "failed to write {}",
            record_to.join("scenario.json").display()
        )
    })?;
    fs::write(
        record_to.join("trace.json"),
        serde_json::to_vec_pretty(&recording.trace)?,
    )
    .with_context(|| format!("failed to write {}", record_to.join("trace.json").display()))?;
    Ok(())
}

pub(crate) fn load_recording(record_to: &Path) -> Result<NativeRecording> {
    let record_dir = resolve_recording_dir(record_to)?;
    load_recording_from_dir(&record_dir)
}

fn load_recording_from_dir(record_dir: &Path) -> Result<NativeRecording> {
    let manifest = fs::read(record_dir.join("manifest.json")).with_context(|| {
        format!(
            "failed to read {}",
            record_dir.join("manifest.json").display()
        )
    })?;
    let trace = fs::read(record_dir.join("trace.json"))
        .with_context(|| format!("failed to read {}", record_dir.join("trace.json").display()))?;
    Ok(NativeRecording {
        manifest: serde_json::from_slice(&manifest).with_context(|| {
            format!(
                "failed to parse {}",
                record_dir.join("manifest.json").display()
            )
        })?,
        trace: serde_json::from_slice(&trace).with_context(|| {
            format!(
                "failed to parse {}",
                record_dir.join("trace.json").display()
            )
        })?,
    })
}

pub(crate) fn run_recorded_trace(
    manifest: &AssetManifest,
    start: RecordingStart,
    script: &[u8],
) -> Result<Vec<NativeTraceFrame>> {
    let initial_level_id = start.warp_level.unwrap_or(0x00);
    let (_, block_grid) = load_session_assets(manifest, initial_level_id)?;
    let mut runtime = NativeRuntime::new(manifest.clone());
    runtime.set_block_grid(block_grid);
    if let Some(level_id) = start.warp_level {
        if start.settled {
            runtime.enter_level_playtest(LevelId(level_id));
        } else {
            runtime.warp_to_level(LevelId(level_id));
        }
    } else if start.settled {
        runtime.enter_level_playtest(LevelId(0));
    }

    let mut trace = runtime.trace_frame();
    let mut current_level_id = trace.current_level.0;
    let mut replay = Vec::with_capacity(script.len());
    let mut level_card_frames_remaining = 0usize;
    let mut script_index = 0usize;

    while script_index < script.len() {
        if level_card_frames_remaining == 0 {
            let buttons = script[script_index];
            script_index += 1;
            let previous_trace = trace.clone();
            trace = runtime.step_frame(FrameInput::from_mask(buttons));
            let recorded_frame = trace.clone();

            let title_start_bootstrapped = start.warp_level.is_none()
                && !start.settled
                && previous_trace.start_press_count < 6
                && trace.start_press_count >= 6
                && matches!(trace.game_mode, GameMode::InGame);
            if title_start_bootstrapped {
                runtime.enter_level_playtest(LevelId(trace.current_level.0));
                trace = runtime.trace_frame();
                level_card_frames_remaining = RECORDING_LEVEL_CARD_HOLD_FRAMES;
            }

            let level_changed = recorded_frame.current_level.0 != current_level_id;
            if level_changed {
                let new_level = recorded_frame.current_level.0;
                runtime.enter_level_playtest(LevelId(new_level));
                trace = runtime.trace_frame();
                current_level_id = new_level;
                level_card_frames_remaining = RECORDING_LEVEL_CARD_HOLD_FRAMES;
            }

            replay.push(recorded_frame);
        }

        level_card_frames_remaining = level_card_frames_remaining.saturating_sub(1);
    }

    Ok(replay)
}

pub(crate) fn verify_recording(
    repo_root: &Path,
    rom_path: &Path,
    manifest: &AssetManifest,
    record_to: &Path,
) -> Result<()> {
    let record_dir = resolve_recording_dir(record_to)?;
    let recording = load_recording(&record_dir)?;
    let replay_trace = run_recorded_trace(
        manifest,
        recording.manifest.start,
        &recording.manifest.scenario.script,
    )?;

    let verify_dir = record_dir.join("verify");
    let native_dir = verify_dir.join("native");
    let oracle_dir = verify_dir.join("oracle");
    if verify_dir.exists() {
        fs::remove_dir_all(&verify_dir)
            .with_context(|| format!("failed to remove {}", verify_dir.display()))?;
    }
    fs::create_dir_all(&native_dir)
        .with_context(|| format!("failed to create {}", native_dir.display()))?;
    fs::create_dir_all(&oracle_dir)
        .with_context(|| format!("failed to create {}", oracle_dir.display()))?;

    fs::write(
        native_dir.join("trace.json"),
        serde_json::to_vec_pretty(&replay_trace)?,
    )
    .with_context(|| {
        format!(
            "failed to write {}",
            native_dir.join("trace.json").display()
        )
    })?;

    let first_trace_drift = first_trace_drift(&recording.trace, &replay_trace);
    let trace_match = first_trace_drift.is_none();

    let native_states =
        capture_dynamic_checkpoint_states(&replay_trace, &recording.manifest.scenario)?;
    fs::write(
        native_dir.join("state.json"),
        serde_json::to_vec_pretty(&native_states)?,
    )
    .with_context(|| {
        format!(
            "failed to write {}",
            native_dir.join("state.json").display()
        )
    })?;

    let native_frames =
        render_dynamic_checkpoint_frames(manifest, &replay_trace, &recording.manifest.scenario)?;
    write_rendered_frames(&native_dir, &native_frames)?;

    let env = ensure_oracles(repo_root)?;
    let (oracle_scenario, oracle_frame_prefix) = oracle_scenario_for_recording(&recording);
    let oracle_scenario_path = oracle_dir.join("scenario.json");
    fs::write(
        &oracle_scenario_path,
        serde_json::to_vec_pretty(&oracle_scenario)?,
    )
    .with_context(|| format!("failed to write {}", oracle_scenario_path.display()))?;
    let oracle =
        run_jgenesis_capture_dynamic(&env, rom_path, &oracle_scenario_path, &oracle_dir, None)?;
    let oracle = normalise_oracle_manifest_frame_indices(oracle, oracle_frame_prefix);
    fs::write(
        oracle_dir.join("manifest.json"),
        serde_json::to_vec_pretty(&oracle)?,
    )
    .with_context(|| {
        format!(
            "failed to write {}",
            oracle_dir.join("manifest.json").display()
        )
    })?;

    let oracle_state_match =
        compare_native_states_against_oracle(&record_dir, &native_states, &oracle.states).is_ok();
    let frame_diffs = compare_recording_frames(
        &native_dir,
        &native_frames,
        &oracle.frames,
        recording.manifest.start.warp_level.unwrap_or_else(|| {
            replay_trace
                .last()
                .map(|frame| frame.current_level.0)
                .unwrap_or(0)
        }),
    )?;
    let oracle_frame_match = frame_diffs
        .iter()
        .all(|diff| diff.max_channel_delta == 0 && diff.mean_absolute_error == 0.0);

    let report = RecordingVerificationReport {
        slug: recording.manifest.slug.clone(),
        trace_match,
        oracle_state_match,
        oracle_frame_match,
        replay_trace_len: replay_trace.len(),
        recorded_trace_len: recording.trace.len(),
        first_trace_drift,
        frame_diffs,
    };
    fs::write(
        verify_dir.join("report.json"),
        serde_json::to_vec_pretty(&report)?,
    )
    .with_context(|| {
        format!(
            "failed to write {}",
            verify_dir.join("report.json").display()
        )
    })?;

    if !report.trace_match {
        let drift = report
            .first_trace_drift
            .as_ref()
            .expect("trace drift should exist when trace_match is false");
        bail!(
            "recording replay drifted at frame {} ({}) ; artefacts in {}",
            drift.frame_index,
            drift.reason,
            verify_dir.display()
        );
    }
    if !report.oracle_state_match {
        bail!(
            "recording checkpoints diverged from oracle state; artefacts in {}",
            verify_dir.display()
        );
    }
    if !report.oracle_frame_match {
        bail!(
            "recording checkpoint frames diverged from oracle pixels; artefacts in {}",
            verify_dir.display()
        );
    }

    println!(
        "recording verification passed for {}: {}",
        recording.manifest.slug,
        verify_dir.display()
    );
    Ok(())
}

fn resolve_recording_dir(record_to: &Path) -> Result<PathBuf> {
    if record_to.join("manifest.json").is_file() && record_to.join("trace.json").is_file() {
        return Ok(record_to.to_path_buf());
    }
    if !record_to.is_dir() {
        return Ok(record_to.to_path_buf());
    }

    let mut nested_candidates = fs::read_dir(record_to)
        .with_context(|| format!("failed to read {}", record_to.display()))?
        .filter_map(|entry| entry.ok().map(|entry| entry.path()))
        .filter(|path| path.is_dir())
        .filter(|path| path.join("manifest.json").is_file() && path.join("trace.json").is_file())
        .collect::<Vec<_>>();
    nested_candidates.sort();

    match nested_candidates.as_slice() {
        [nested] => Ok(nested.clone()),
        [] => Ok(record_to.to_path_buf()),
        _ => bail!(
            "found multiple nested recordings under {}; expected one",
            record_to.display()
        ),
    }
}

fn recording_checkpoints(frame_count: usize) -> Vec<DynamicCheckpoint> {
    if frame_count == 0 {
        return Vec::new();
    }
    let mut checkpoints = Vec::new();
    checkpoints.push(DynamicCheckpoint {
        label: "start".to_owned(),
        frame_index: 0,
    });
    let mut frame_index = RECORDING_CHECKPOINT_INTERVAL;
    while frame_index < frame_count {
        checkpoints.push(DynamicCheckpoint {
            label: format!("frame_{frame_index:06}"),
            frame_index,
        });
        frame_index += RECORDING_CHECKPOINT_INTERVAL;
    }
    let final_frame = frame_count.saturating_sub(1);
    if checkpoints
        .last()
        .is_none_or(|checkpoint| checkpoint.frame_index != final_frame)
    {
        checkpoints.push(DynamicCheckpoint {
            label: "end".to_owned(),
            frame_index: final_frame,
        });
    }
    checkpoints
}

fn oracle_scenario_for_recording(recording: &NativeRecording) -> (DynamicScenario, usize) {
    if !recording.manifest.start.settled {
        return (recording.manifest.scenario.clone(), 0);
    }

    let mut script = vec![0; RECORDING_SETTLED_ORACLE_PREFIX_FRAMES];
    script.extend_from_slice(&recording.manifest.scenario.script);
    let checkpoints = recording
        .manifest
        .scenario
        .checkpoints
        .iter()
        .map(|checkpoint| DynamicCheckpoint {
            label: checkpoint.label.clone(),
            frame_index: checkpoint.frame_index + RECORDING_SETTLED_ORACLE_PREFIX_FRAMES,
        })
        .collect::<Vec<_>>();

    (
        DynamicScenario {
            slug: format!("{}-oracle", recording.manifest.scenario.slug),
            frame_count: script.len(),
            script,
            checkpoints,
            warp_to_level: Some(recording.manifest.start.warp_level.unwrap_or(0)),
        },
        RECORDING_SETTLED_ORACLE_PREFIX_FRAMES,
    )
}

fn normalise_oracle_manifest_frame_indices(
    mut oracle: OracleManifest,
    prefix_frames: usize,
) -> OracleManifest {
    if prefix_frames == 0 {
        return oracle;
    }

    for frame in &mut oracle.frames {
        frame.frame_index = frame.frame_index.saturating_sub(prefix_frames);
    }
    for state in &mut oracle.states {
        state.frame_index = state.frame_index.saturating_sub(prefix_frames);
    }

    oracle
}

fn capture_dynamic_checkpoint_states(
    trace: &[NativeTraceFrame],
    scenario: &DynamicScenario,
) -> Result<Vec<OracleState>> {
    scenario
        .checkpoints
        .iter()
        .map(|checkpoint| {
            let frame =
                native_trace_at_checkpoint(trace, &checkpoint.label, checkpoint.frame_index)
                    .ok_or_else(|| {
                        anyhow!(
                            "missing native checkpoint {}@{}",
                            checkpoint.label,
                            checkpoint.frame_index
                        )
                    })?;
            Ok(native_trace_to_oracle_state(
                &checkpoint.label,
                checkpoint.frame_index,
                frame,
            ))
        })
        .collect()
}

type RenderedFrame = (String, usize, usize, usize, Vec<u8>);

fn render_dynamic_checkpoint_frames(
    manifest: &AssetManifest,
    trace: &[NativeTraceFrame],
    scenario: &DynamicScenario,
) -> Result<Vec<RenderedFrame>> {
    scenario
        .checkpoints
        .iter()
        .map(|checkpoint| {
            let frame =
                native_trace_at_checkpoint(trace, &checkpoint.label, checkpoint.frame_index)
                    .ok_or_else(|| {
                        anyhow!(
                            "missing native frame {}@{}",
                            checkpoint.label,
                            checkpoint.frame_index
                        )
                    })?;
            let (level_visuals, block_grid) = load_session_assets(manifest, frame.current_level.0)?;
            let scene = build_scene(frame, false, Some(&level_visuals), Some(&block_grid));
            let mut rgb = vec![0; (NATIVE_FRAME_WIDTH * NATIVE_FRAME_HEIGHT * 3) as usize];
            render_scene(&scene, &mut rgb);
            Ok((
                checkpoint.label.clone(),
                checkpoint.frame_index,
                NATIVE_FRAME_WIDTH as usize,
                NATIVE_FRAME_HEIGHT as usize,
                rgb,
            ))
        })
        .collect()
}

fn write_rendered_frames(out_dir: &Path, frames: &[RenderedFrame]) -> Result<()> {
    for (label, frame_index, width, height, rgb) in frames {
        let path = out_dir.join(format!("{frame_index:06}_{label}.png"));
        image::save_buffer_with_format(
            &path,
            rgb,
            *width as u32,
            *height as u32,
            ColorType::Rgb8,
            ImageFormat::Png,
        )
        .with_context(|| format!("failed to write {}", path.display()))?;
    }
    Ok(())
}

fn compare_recording_frames(
    native_dir: &Path,
    native_frames: &[RenderedFrame],
    oracle_frames: &[OracleFrame],
    level_id: u8,
) -> Result<Vec<CheckpointFrameDiff>> {
    let mut diffs = Vec::with_capacity(native_frames.len());
    for (label, frame_index, width, height, native_rgb) in native_frames {
        let oracle_frame = oracle_frames
            .iter()
            .find(|frame| frame.label == *label && frame.frame_index == *frame_index)
            .ok_or_else(|| anyhow!("missing oracle frame {label}@{frame_index}"))?;
        let oracle_rgb = load_png_rgb(&oracle_frame.png_path)?;
        let frame_diff = compare_rgb_frames(native_rgb, &oracle_rgb)?;
        let (diff_width, diff_height, mut diff_rgb) =
            generate_diff_composite(native_rgb, &oracle_rgb, *width as u32, *height as u32);
        annotate_diff(
            &mut diff_rgb,
            diff_width,
            level_id,
            label,
            frame_diff.mean_absolute_error,
        );
        let diff_path = native_dir.join(format!("{frame_index:06}_{label}_diff.png"));
        image::save_buffer_with_format(
            &diff_path,
            &diff_rgb,
            diff_width,
            diff_height,
            ColorType::Rgb8,
            ImageFormat::Png,
        )
        .with_context(|| format!("failed to write {}", diff_path.display()))?;
        diffs.push(CheckpointFrameDiff {
            label: label.clone(),
            frame_index: *frame_index,
            mean_absolute_error: frame_diff.mean_absolute_error,
            max_channel_delta: frame_diff.max_channel_delta,
            diff_path,
        });
    }
    Ok(diffs)
}

fn first_trace_drift(
    recorded: &[NativeTraceFrame],
    replay: &[NativeTraceFrame],
) -> Option<TraceDriftReport> {
    for (index, (expected, actual)) in recorded.iter().zip(replay).enumerate() {
        if expected != actual {
            return Some(TraceDriftReport {
                frame_index: index,
                reason: "native trace frame mismatch".to_owned(),
            });
        }
    }
    if recorded.len() != replay.len() {
        return Some(TraceDriftReport {
            frame_index: recorded.len().min(replay.len()),
            reason: format!(
                "trace length mismatch (recorded {}, replay {})",
                recorded.len(),
                replay.len()
            ),
        });
    }
    None
}

fn unix_timestamp() -> String {
    use std::time::SystemTime;
    let since_epoch = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap_or_default();
    format!("{}", since_epoch.as_secs())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn sample_trace_frame(frame_index: usize) -> NativeTraceFrame {
        NativeTraceFrame {
            frame_index,
            game_mode: GameMode::InGame,
            game_mode_value: GameMode::InGame.as_disassembly_value(),
            start_press_count: 0,
            boot_phase: None,
            boot_phase_elapsed: None,
            title_screen_ready: false,
            current_player: 0,
            current_level: LevelId(0),
            player_1_level_id: LevelId(0),
            current_level_name: None,
            player_start_x: None,
            player_start_y: None,
            flag_x: None,
            flag_y: None,
            level_enemy_object_count: None,
            active_enemy_object_count: 0,
            active_platform_object_count: 0,
            number_lives: 3,
            number_continues: 3,
            number_hitpoints: 2,
            current_helmet_id: 0,
            paused: false,
            kid_x_pos: Some(0x20_0000),
            kid_y_pos: Some(0x50_0000),
            kid_x_velocity: Some(0),
            kid_y_velocity: Some(0),
            kid_facing: Some(NativeDirection::Right),
            kid_on_ground: Some(true),
            active_objects: Vec::new(),
            camera_x_pos: 0,
            camera_y_pos: 0,
            number_objects: 0,
            number_gfx_objects: 0,
            level_complete: false,
            game_over: false,
            continue_countdown: None,
            game_complete: false,
            ending_elapsed: None,
            options_cursor: None,
            sound_test_track: None,
            two_player: false,
            invulnerable_frames: 0,
            diamond_power_active: false,
            diamond_count: 0,
            score: 0,
            level_timer_seconds: 0,
            level_damage_taken: false,
            murder_wall_x: None,
            active_projectiles: Vec::new(),
            active_geysers: Vec::new(),
            block_bounces: Vec::new(),
            block_destroy_effects: Vec::new(),
            floating_items: Vec::new(),
            enemy_death_effects: Vec::new(),
            form_change_flash_frames: 0,
            stomped_this_frame: false,
            blocks_hit_this_frame: 0,
            sound_test_play_pending: false,
            physics_trace: None,
        }
    }

    #[test]
    fn recording_checkpoints_include_start_interval_and_end() {
        let checkpoints = recording_checkpoints(125);
        assert_eq!(checkpoints[0].label, "start");
        assert_eq!(checkpoints[0].frame_index, 0);
        assert_eq!(checkpoints[1].frame_index, 60);
        assert_eq!(checkpoints[2].frame_index, 120);
        assert_eq!(checkpoints[3].label, "end");
        assert_eq!(checkpoints[3].frame_index, 124);
    }

    #[test]
    fn first_trace_drift_reports_first_changed_frame() {
        let recorded = vec![sample_trace_frame(0), sample_trace_frame(1)];
        let mut replay = recorded.clone();
        replay[1].number_hitpoints = 1;
        let drift = first_trace_drift(&recorded, &replay).expect("drift");
        assert_eq!(drift.frame_index, 1);
    }

    #[test]
    fn build_recording_uses_script_len_for_frame_count() {
        let recording = build_recording(
            "level1-tree-bug",
            RecordingStart {
                warp_level: Some(0),
                settled: true,
            },
            vec![0, 1, 2],
            vec![
                sample_trace_frame(0),
                sample_trace_frame(1),
                sample_trace_frame(2),
            ],
        );
        assert_eq!(recording.manifest.scenario.frame_count, 3);
        assert_eq!(recording.manifest.scenario.warp_to_level, Some(0));
    }

    #[test]
    fn resolve_recording_dir_accepts_single_nested_copy() {
        let temp = TempDir::new().expect("temp dir");
        let outer = temp.path().join("level1-tree-break");
        let nested = outer.join("level1-tree-break");
        fs::create_dir_all(&nested).expect("nested dir");
        fs::write(nested.join("manifest.json"), b"{}").expect("manifest");
        fs::write(nested.join("trace.json"), b"[]").expect("trace");

        let resolved = resolve_recording_dir(&outer).expect("resolved dir");
        assert_eq!(resolved, nested);
    }

    #[test]
    fn settled_recordings_translate_to_oracle_warp_with_prefix() {
        let recording = build_recording(
            "level1-tree-bug",
            RecordingStart {
                warp_level: Some(0),
                settled: true,
            },
            vec![1, 2, 3],
            vec![
                sample_trace_frame(0),
                sample_trace_frame(1),
                sample_trace_frame(2),
            ],
        );

        let (oracle_scenario, prefix) = oracle_scenario_for_recording(&recording);

        assert_eq!(prefix, RECORDING_SETTLED_ORACLE_PREFIX_FRAMES);
        assert_eq!(oracle_scenario.warp_to_level, Some(0));
        assert_eq!(oracle_scenario.frame_count, 63);
        assert_eq!(oracle_scenario.script.len(), 63);
        assert!(
            oracle_scenario.script[..prefix]
                .iter()
                .all(|input| *input == 0)
        );
        assert_eq!(&oracle_scenario.script[prefix..], &[1, 2, 3]);
        assert_eq!(oracle_scenario.checkpoints[0].label, "start");
        assert_eq!(oracle_scenario.checkpoints[0].frame_index, prefix);
    }

    #[test]
    fn unsettled_recordings_keep_original_oracle_scenario() {
        let recording = build_recording(
            "title-start",
            RecordingStart {
                warp_level: None,
                settled: false,
            },
            vec![1, 2, 3],
            vec![
                sample_trace_frame(0),
                sample_trace_frame(1),
                sample_trace_frame(2),
            ],
        );

        let (oracle_scenario, prefix) = oracle_scenario_for_recording(&recording);

        assert_eq!(prefix, 0);
        assert_eq!(oracle_scenario, recording.manifest.scenario);
    }
}
