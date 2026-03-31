use super::*;
use crate::capture::write_wav_with_channels;
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
const RECORDING_BUNDLE_VERSION: u32 = 4;

const RECORDING_NOTES_FILENAME: &str = "notes.txt";
const RECORDING_TRANSCRIPT_FILENAME: &str = "transcript.txt";
const RECORDING_FINAL_FRAME_FILENAME: &str = "final-frame.png";
const RECORDING_AUDIO_FILENAME: &str = "audio.wav";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct RecordingArtifactManifest {
    pub(crate) path: String,
    pub(crate) kind: String,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub(crate) struct RecordingBundleArtifacts {
    pub(crate) user_notes: Option<String>,
    pub(crate) final_frame_rgb: Option<Vec<u8>>,
    pub(crate) audio_samples: Option<Vec<f32>>,
    pub(crate) audio_channels: u16,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct RecordingStart {
    pub(crate) warp_level: Option<u8>,
    pub(crate) settled: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct NativeRecordingManifest {
    #[serde(default)]
    pub(crate) bundle_version: u32,
    pub(crate) slug: String,
    pub(crate) recorded_at: String,
    pub(crate) start: RecordingStart,
    #[serde(default)]
    pub(crate) rom_sha1: String,
    #[serde(default)]
    pub(crate) script_sha1: String,
    #[serde(default)]
    pub(crate) trace_sha1: String,
    #[serde(default)]
    pub(crate) host_os: String,
    #[serde(default)]
    pub(crate) host_arch: String,
    #[serde(default)]
    pub(crate) git_commit: Option<String>,
    #[serde(default)]
    pub(crate) git_dirty: Option<bool>,
    #[serde(default)]
    pub(crate) record_command: String,
    #[serde(default)]
    pub(crate) verify_command: String,
    #[serde(default)]
    pub(crate) artifacts: Vec<RecordingArtifactManifest>,
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
    manifest: &AssetManifest,
    slug: &str,
    start: RecordingStart,
    script: Vec<u8>,
    trace: Vec<NativeTraceFrame>,
) -> Result<NativeRecording> {
    let frame_count = script.len();
    let git_status = current_git_status();
    let verify_command = format!("cargo run -p kidcli -- verify-recording --slug {slug}");
    let record_command = recording_command(slug, start);
    let script_sha1 = sha1_hex(&script);
    let trace_sha1 = sha1_hex(&serde_json::to_vec(&trace)?);

    Ok(NativeRecording {
        manifest: NativeRecordingManifest {
            bundle_version: RECORDING_BUNDLE_VERSION,
            slug: slug.to_owned(),
            recorded_at: unix_timestamp(),
            start,
            rom_sha1: manifest.rom_sha1.clone(),
            script_sha1,
            trace_sha1,
            host_os: std::env::consts::OS.to_owned(),
            host_arch: std::env::consts::ARCH.to_owned(),
            git_commit: git_status.as_ref().and_then(|status| status.commit.clone()),
            git_dirty: git_status.as_ref().map(|status| status.dirty),
            record_command,
            verify_command,
            artifacts: Vec::new(),
            scenario: DynamicScenario {
                slug: slug.to_owned(),
                checkpoints: recording_checkpoints(frame_count),
                frame_count,
                script,
                warp_to_level: start.warp_level,
            },
        },
        trace,
    })
}

pub(crate) fn save_recording_with_artifacts(
    record_to: &Path,
    recording: &NativeRecording,
    artifacts: &RecordingBundleArtifacts,
) -> Result<()> {
    if record_to.exists() {
        fs::remove_dir_all(record_to)
            .with_context(|| format!("failed to remove {}", record_to.display()))?;
    }
    fs::create_dir_all(record_to)
        .with_context(|| format!("failed to create {}", record_to.display()))?;
    let mut manifest = recording.manifest.clone();
    manifest.artifacts = write_recording_artifacts(record_to, recording, artifacts)?;

    fs::write(
        record_to.join("manifest.json"),
        serde_json::to_vec_pretty(&manifest)?,
    )
    .with_context(|| {
        format!(
            "failed to write {}",
            record_to.join("manifest.json").display()
        )
    })?;
    fs::write(
        record_to.join("scenario.json"),
        serde_json::to_vec_pretty(&manifest.scenario)?,
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
    fs::write(
        record_to.join("README.txt"),
        recording_bundle_summary(&manifest, recording.trace.len()),
    )
    .with_context(|| format!("failed to write {}", record_to.join("README.txt").display()))?;
    Ok(())
}

pub(crate) fn load_recording(record_to: &Path) -> Result<NativeRecording> {
    let record_dir = resolve_recording_dir(record_to)?;
    load_recording_from_dir(&record_dir)
}

pub(crate) fn resolve_recording_bundle_input(
    repo_root: &Path,
    slug: Option<&str>,
    bundle: Option<&Path>,
) -> Result<PathBuf> {
    match (slug, bundle) {
        (Some(_), Some(_)) => {
            bail!("pass either --slug <slug> or --bundle <path>, not both")
        }
        (None, None) => {
            bail!("verify-recording needs either --slug <slug> or --bundle <path>")
        }
        (Some(slug), None) => Ok(repo_root
            .join(".cache/recordings")
            .join(validate_recording_slug(slug)?)),
        (None, Some(bundle)) => resolve_recording_dir(bundle),
    }
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
    let mut recording = NativeRecording {
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
    };
    normalise_loaded_recording(&mut recording)?;
    Ok(recording)
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

            // Show level card when the engine first enters gameplay
            let entered_gameplay = !matches!(previous_trace.game_mode, GameMode::InGame)
                && matches!(trace.game_mode, GameMode::InGame)
                && start.warp_level.is_none()
                && !start.settled;
            if entered_gameplay {
                level_card_frames_remaining = RECORDING_LEVEL_CARD_HOLD_FRAMES;
            }

            let level_changed = recorded_frame.current_level.0 != current_level_id;
            if level_changed {
                current_level_id = recorded_frame.current_level.0;
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

pub(crate) fn oracle_scenario_for_recording(
    recording: &NativeRecording,
) -> (DynamicScenario, usize) {
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

fn normalise_loaded_recording(recording: &mut NativeRecording) -> Result<()> {
    if recording.manifest.bundle_version == 0 {
        recording.manifest.bundle_version = 1;
    }
    if recording.manifest.script_sha1.is_empty() {
        recording.manifest.script_sha1 = sha1_hex(&recording.manifest.scenario.script);
    }
    if recording.manifest.trace_sha1.is_empty() {
        recording.manifest.trace_sha1 = sha1_hex(&serde_json::to_vec(&recording.trace)?);
    }
    if recording.manifest.host_os.is_empty() {
        recording.manifest.host_os = String::from("unknown");
    }
    if recording.manifest.host_arch.is_empty() {
        recording.manifest.host_arch = String::from("unknown");
    }
    if recording.manifest.record_command.is_empty() {
        recording.manifest.record_command =
            recording_command(&recording.manifest.slug, recording.manifest.start);
    }
    if recording.manifest.verify_command.is_empty() {
        recording.manifest.verify_command = format!(
            "cargo run -p kidcli -- verify-recording --slug {}",
            recording.manifest.slug
        );
    }
    Ok(())
}

fn write_recording_artifacts(
    record_to: &Path,
    recording: &NativeRecording,
    artifacts: &RecordingBundleArtifacts,
) -> Result<Vec<RecordingArtifactManifest>> {
    let mut manifest_entries = Vec::new();

    let transcript_path = record_to.join(RECORDING_TRANSCRIPT_FILENAME);
    fs::write(&transcript_path, recording_transcript(&recording.trace))
        .with_context(|| format!("failed to write {}", transcript_path.display()))?;
    manifest_entries.push(RecordingArtifactManifest {
        path: String::from(RECORDING_TRANSCRIPT_FILENAME),
        kind: String::from("trace-transcript"),
    });

    if let Some(notes) = &artifacts.user_notes {
        let notes_path = record_to.join(RECORDING_NOTES_FILENAME);
        fs::write(&notes_path, notes)
            .with_context(|| format!("failed to write {}", notes_path.display()))?;
        manifest_entries.push(RecordingArtifactManifest {
            path: String::from(RECORDING_NOTES_FILENAME),
            kind: String::from("user-notes"),
        });
    }

    if let Some(final_frame_rgb) = &artifacts.final_frame_rgb {
        let frame_path = record_to.join(RECORDING_FINAL_FRAME_FILENAME);
        image::save_buffer_with_format(
            &frame_path,
            final_frame_rgb,
            NATIVE_FRAME_WIDTH,
            NATIVE_FRAME_HEIGHT,
            ColorType::Rgb8,
            ImageFormat::Png,
        )
        .with_context(|| format!("failed to write {}", frame_path.display()))?;
        manifest_entries.push(RecordingArtifactManifest {
            path: String::from(RECORDING_FINAL_FRAME_FILENAME),
            kind: String::from("final-frame"),
        });
    }

    if let Some(audio_samples) = &artifacts.audio_samples {
        let audio_path = record_to.join(RECORDING_AUDIO_FILENAME);
        write_wav_with_channels(&audio_path, 44_100, artifacts.audio_channels, audio_samples)
            .with_context(|| format!("failed to write {}", audio_path.display()))?;
        manifest_entries.push(RecordingArtifactManifest {
            path: String::from(RECORDING_AUDIO_FILENAME),
            kind: String::from("audio"),
        });
    }

    Ok(manifest_entries)
}

fn recording_transcript(trace: &[NativeTraceFrame]) -> String {
    let mut lines = vec![
        String::from("Kid Chameleon native recording transcript"),
        format!("frames: {}", trace.len()),
    ];
    if trace.is_empty() {
        lines.push(String::from("events: none"));
        return lines.join("\n");
    }

    let start = &trace[0];
    lines.push(format!(
        "start: frame {:06} mode={:?} level={} lives={} hp={} helmet={} paused={} kid={}",
        start.frame_index,
        start.game_mode,
        format_level(start),
        start.number_lives,
        start.number_hitpoints,
        start.current_helmet_id,
        start.paused,
        format_kid_position(start),
    ));
    lines.push(String::from("events:"));

    let mut event_count = 0usize;
    for (previous, frame) in trace.iter().zip(trace.iter().skip(1)) {
        if previous.game_mode != frame.game_mode {
            lines.push(format!(
                "frame {:06}: game_mode {:?} -> {:?}",
                frame.frame_index, previous.game_mode, frame.game_mode
            ));
            event_count += 1;
        }
        if previous.current_level != frame.current_level {
            lines.push(format!(
                "frame {:06}: level {} -> {}",
                frame.frame_index,
                format_level(previous),
                format_level(frame),
            ));
            event_count += 1;
        }
        if previous.paused != frame.paused {
            lines.push(format!(
                "frame {:06}: paused {} -> {}",
                frame.frame_index, previous.paused, frame.paused
            ));
            event_count += 1;
        }
        if previous.number_lives != frame.number_lives {
            lines.push(format!(
                "frame {:06}: lives {} -> {}",
                frame.frame_index, previous.number_lives, frame.number_lives
            ));
            event_count += 1;
        }
        if previous.number_hitpoints != frame.number_hitpoints {
            lines.push(format!(
                "frame {:06}: hp {} -> {}",
                frame.frame_index, previous.number_hitpoints, frame.number_hitpoints
            ));
            event_count += 1;
        }
        if previous.current_helmet_id != frame.current_helmet_id {
            lines.push(format!(
                "frame {:06}: helmet {} -> {}",
                frame.frame_index, previous.current_helmet_id, frame.current_helmet_id
            ));
            event_count += 1;
        }
        if !previous.just_received_damage && frame.just_received_damage {
            lines.push(format!(
                "frame {:06}: damage latched at kid={}",
                frame.frame_index,
                format_kid_position(frame),
            ));
            event_count += 1;
        }
        if !previous.level_complete && frame.level_complete {
            lines.push(format!(
                "frame {:06}: level_complete on {}",
                frame.frame_index,
                format_level(frame),
            ));
            event_count += 1;
        }
        if !previous.game_over && frame.game_over {
            lines.push(format!(
                "frame {:06}: game_over lives={} continues={} hp={}",
                frame.frame_index,
                frame.number_lives,
                frame.number_continues,
                frame.number_hitpoints
            ));
            event_count += 1;
        }
        if previous.continue_countdown != frame.continue_countdown
            && frame.continue_countdown.is_some()
        {
            lines.push(format!(
                "frame {:06}: continue_countdown -> {}",
                frame.frame_index,
                frame.continue_countdown.unwrap_or_default()
            ));
            event_count += 1;
        }
    }

    if event_count == 0 {
        lines.push(String::from("none"));
    }

    lines.join("\n")
}

fn format_level(frame: &NativeTraceFrame) -> String {
    match frame.current_level_name.as_deref() {
        Some(name) => format!("{:02X} ({name})", frame.current_level.0),
        None => format!("{:02X}", frame.current_level.0),
    }
}

fn format_kid_position(frame: &NativeTraceFrame) -> String {
    match (frame.kid_x_pos, frame.kid_y_pos) {
        (Some(x), Some(y)) => format!("({},{})", x >> 16, y >> 16),
        _ => String::from("(hidden)"),
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct GitStatus {
    commit: Option<String>,
    dirty: bool,
}

fn current_git_status() -> Option<GitStatus> {
    let commit = run_git(["rev-parse", "HEAD"]);
    let dirty_output = run_git(["status", "--short"]);
    if commit.is_none() && dirty_output.is_none() {
        return None;
    }
    Some(GitStatus {
        commit,
        dirty: dirty_output.is_some_and(|output| !output.is_empty()),
    })
}

fn run_git<const N: usize>(args: [&str; N]) -> Option<String> {
    let output = std::process::Command::new("git").args(args).output().ok()?;
    if !output.status.success() {
        return None;
    }
    let stdout = String::from_utf8(output.stdout).ok()?;
    let trimmed = stdout.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_owned())
    }
}

fn recording_command(slug: &str, start: RecordingStart) -> String {
    let mut command = String::from("cargo run --release -p kidcli -- play-native");
    if let Some(level) = start.warp_level {
        command.push_str(&format!(" --warp-level {level:02X}"));
    }
    if start.settled {
        command.push_str(" --settled");
    }
    command.push_str(&format!(" --record {slug}"));
    command
}

fn recording_bundle_summary(manifest: &NativeRecordingManifest, trace_len: usize) -> String {
    let mut lines = vec![
        format!(
            "Kid Chameleon native recording bundle v{}",
            manifest.bundle_version
        ),
        format!("slug: {}", manifest.slug),
        format!("recorded_at_unix: {}", manifest.recorded_at),
        format!("rom_sha1: {}", manifest.rom_sha1),
        format!("frame_count: {}", manifest.scenario.frame_count),
        format!("trace_len: {}", trace_len),
        format!("script_sha1: {}", manifest.script_sha1),
        format!("trace_sha1: {}", manifest.trace_sha1),
        format!("host: {}/{}", manifest.host_os, manifest.host_arch),
    ];
    if let Some(commit) = &manifest.git_commit {
        lines.push(format!("git_commit: {commit}"));
    }
    if let Some(dirty) = manifest.git_dirty {
        lines.push(format!("git_dirty: {dirty}"));
    }
    lines.push(format!("start_warp_level: {:?}", manifest.start.warp_level));
    lines.push(format!("start_settled: {}", manifest.start.settled));
    lines.push(String::new());
    lines.push(String::from("record_command:"));
    lines.push(manifest.record_command.clone());
    lines.push(String::new());
    lines.push(String::from("verify_command:"));
    lines.push(manifest.verify_command.clone());
    lines.push(String::new());
    lines.push(String::from("bundle_files:"));
    lines.push(String::from("- manifest.json"));
    lines.push(String::from("- scenario.json"));
    lines.push(String::from("- trace.json"));
    lines.push(String::from("- README.txt"));
    for artifact in &manifest.artifacts {
        lines.push(format!("- {} ({})", artifact.path, artifact.kind));
    }
    lines.join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::capture::read_wav_f32;
    use tempfile::TempDir;

    fn fixture_manifest() -> AssetManifest {
        AssetManifest {
            generated_bundle_version: 8,
            rom_sha1: String::from("fixture-rom"),
            upstream_commit: String::from("upstream"),
            source_root: PathBuf::from("/tmp/source"),
            output_root: PathBuf::from("/tmp/output"),
            map_order: Vec::new(),
            levels: Vec::new(),
            constants: Vec::new(),
            ram_variables: Vec::new(),
            enemy_sources: Vec::new(),
            theme_assets: Vec::new(),
            scene_assets: Vec::new(),
            animation_assets: Vec::new(),
            sound_assets: Vec::new(),
            ingame_assets: Vec::new(),
            decoded_levels: Vec::new(),
        }
    }

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
            character_movement: None,
            kid_facing: Some(NativeDirection::Right),
            kid_on_ground: Some(true),
            kid_on_slope: 0,
            active_objects: Vec::new(),
            camera_x_pos: 0,
            camera_y_pos: 0,
            number_objects: 0,
            number_gfx_objects: 0,
            rng_offset: None,
            level_complete: false,
            game_over: false,
            continue_countdown: None,
            game_complete: false,
            ending_elapsed: None,
            options_cursor: None,
            sound_test_track: None,
            two_player: false,
            options_two_player_two_controllers: false,
            options_controls_scheme: 0,
            options_speed_button_fast: false,
            invulnerable_frames: 0,
            just_received_damage: false,
            diamond_power_active: false,
            diamond_count: 0,
            score: 0,
            level_timer_seconds: 0,
            level_damage_taken: false,
            murder_wall_x: None,
            active_projectiles: Vec::new(),
            active_projectile_details: Vec::new(),
            active_geysers: Vec::new(),
            block_bounces: Vec::new(),
            block_destroy_effects: Vec::new(),
            floating_items: Vec::new(),
            enemy_death_effects: Vec::new(),
            death_windup_elapsed: 0,
            form_change_flash_frames: 0,
            stomped_this_frame: false,
            blocks_hit_this_frame: 0,
            sound_test_play_pending: false,
            gameplay_debug_fields: std::collections::BTreeMap::new(),
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
            &fixture_manifest(),
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
        )
        .expect("recording");
        assert_eq!(recording.manifest.scenario.frame_count, 3);
        assert_eq!(recording.manifest.scenario.warp_to_level, Some(0));
        assert_eq!(recording.manifest.rom_sha1, "fixture-rom");
        assert_eq!(
            recording.manifest.verify_command,
            "cargo run -p kidcli -- verify-recording --slug level1-tree-bug"
        );
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
            &fixture_manifest(),
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
        )
        .expect("recording");

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
            &fixture_manifest(),
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
        )
        .expect("recording");

        let (oracle_scenario, prefix) = oracle_scenario_for_recording(&recording);

        assert_eq!(prefix, 0);
        assert_eq!(oracle_scenario, recording.manifest.scenario);
    }

    #[test]
    fn save_recording_writes_human_readable_bundle_summary() {
        let temp = TempDir::new().expect("temp dir");
        let recording = build_recording(
            &fixture_manifest(),
            "level1-tree-bug",
            RecordingStart {
                warp_level: Some(0x01),
                settled: true,
            },
            vec![1, 2, 3],
            vec![
                sample_trace_frame(0),
                sample_trace_frame(1),
                sample_trace_frame(2),
            ],
        )
        .expect("recording");

        save_recording_with_artifacts(
            temp.path(),
            &recording,
            &RecordingBundleArtifacts::default(),
        )
        .expect("save recording");

        let readme = fs::read_to_string(temp.path().join("README.txt")).expect("recording readme");
        assert!(readme.contains("Kid Chameleon native recording bundle v4"));
        assert!(readme.contains("rom_sha1: fixture-rom"));
        assert!(readme.contains("--warp-level 01 --settled --record level1-tree-bug"));
        assert!(readme.contains("cargo run -p kidcli -- verify-recording --slug level1-tree-bug"));
        assert!(readme.contains("- transcript.txt (trace-transcript)"));
    }

    #[test]
    fn save_recording_writes_optional_artifacts() {
        let temp = TempDir::new().expect("temp dir");
        let recording = build_recording(
            &fixture_manifest(),
            "level1-tree-bug",
            RecordingStart {
                warp_level: Some(0x01),
                settled: true,
            },
            vec![1, 2, 3],
            vec![
                sample_trace_frame(0),
                sample_trace_frame(1),
                sample_trace_frame(2),
            ],
        )
        .expect("recording");

        save_recording_with_artifacts(
            temp.path(),
            &recording,
            &RecordingBundleArtifacts {
                user_notes: Some(String::from("tree clip on the second bounce")),
                final_frame_rgb: Some(vec![
                    0;
                    (NATIVE_FRAME_WIDTH as usize)
                        * (NATIVE_FRAME_HEIGHT as usize)
                        * 3
                ]),
                audio_samples: Some(vec![0.25, -0.25, 0.5, -0.5]),
                audio_channels: 2,
            },
        )
        .expect("save recording with artifacts");

        assert!(temp.path().join(RECORDING_TRANSCRIPT_FILENAME).is_file());
        assert!(temp.path().join(RECORDING_NOTES_FILENAME).is_file());
        assert!(temp.path().join(RECORDING_FINAL_FRAME_FILENAME).is_file());
        assert!(temp.path().join(RECORDING_AUDIO_FILENAME).is_file());

        let manifest: NativeRecordingManifest = serde_json::from_slice(
            &fs::read(temp.path().join("manifest.json")).expect("manifest json"),
        )
        .expect("manifest");
        assert_eq!(manifest.bundle_version, 4);
        assert_eq!(
            manifest.artifacts,
            vec![
                RecordingArtifactManifest {
                    path: String::from(RECORDING_TRANSCRIPT_FILENAME),
                    kind: String::from("trace-transcript"),
                },
                RecordingArtifactManifest {
                    path: String::from(RECORDING_NOTES_FILENAME),
                    kind: String::from("user-notes"),
                },
                RecordingArtifactManifest {
                    path: String::from(RECORDING_FINAL_FRAME_FILENAME),
                    kind: String::from("final-frame"),
                },
                RecordingArtifactManifest {
                    path: String::from(RECORDING_AUDIO_FILENAME),
                    kind: String::from("audio"),
                },
            ]
        );

        let readme = fs::read_to_string(temp.path().join("README.txt")).expect("recording readme");
        assert!(readme.contains("- transcript.txt (trace-transcript)"));
        assert!(readme.contains("- notes.txt (user-notes)"));
        assert!(readme.contains("- final-frame.png (final-frame)"));
        assert!(readme.contains("- audio.wav (audio)"));

        let samples = read_wav_f32(&temp.path().join(RECORDING_AUDIO_FILENAME)).expect("wav");
        assert_eq!(samples, vec![0.25, -0.25, 0.5, -0.5]);
    }

    #[test]
    fn save_recording_writes_trace_transcript() {
        let temp = TempDir::new().expect("temp dir");
        let mut start = sample_trace_frame(0);
        start.current_level_name = Some(String::from("Blue Lake Woods 1"));
        let mut damaged = sample_trace_frame(1);
        damaged.current_level_name = Some(String::from("Blue Lake Woods 1"));
        damaged.number_hitpoints = 1;
        damaged.just_received_damage = true;
        let mut game_over = sample_trace_frame(2);
        game_over.current_level_name = Some(String::from("Blue Lake Woods 1"));
        game_over.number_lives = 2;
        game_over.game_over = true;
        game_over.continue_countdown = Some(600);
        let recording = build_recording(
            &fixture_manifest(),
            "level1-tree-bug",
            RecordingStart {
                warp_level: Some(0x01),
                settled: true,
            },
            vec![1, 2, 3],
            vec![start, damaged, game_over],
        )
        .expect("recording");

        save_recording_with_artifacts(
            temp.path(),
            &recording,
            &RecordingBundleArtifacts::default(),
        )
        .expect("save recording");

        let transcript = fs::read_to_string(temp.path().join(RECORDING_TRANSCRIPT_FILENAME))
            .expect("transcript");
        assert!(transcript.contains("Kid Chameleon native recording transcript"));
        assert!(
            transcript.contains("start: frame 000000 mode=InGame level=00 (Blue Lake Woods 1)")
        );
        assert!(transcript.contains("frame 000001: hp 2 -> 1"));
        assert!(transcript.contains("frame 000001: damage latched at kid=(32,80)"));
        assert!(transcript.contains("frame 000002: lives 3 -> 2"));
        assert!(transcript.contains("frame 000002: game_over lives=2 continues=3 hp=2"));
        assert!(transcript.contains("frame 000002: continue_countdown -> 600"));
    }

    #[test]
    fn load_recording_backfills_legacy_metadata_fields() {
        let temp = TempDir::new().expect("temp dir");
        fs::write(
            temp.path().join("manifest.json"),
            r#"{
  "slug": "legacy-level2-entry",
  "recorded_at": "123",
  "start": { "warp_level": 1, "settled": true },
  "scenario": {
    "slug": "legacy-level2-entry",
    "checkpoints": [{ "label": "start", "frame_index": 0 }],
    "frame_count": 1,
    "script": [0],
    "warp_to_level": 1
  }
}"#,
        )
        .expect("legacy manifest");
        fs::write(
            temp.path().join("trace.json"),
            serde_json::to_vec(&vec![sample_trace_frame(0)]).expect("trace json"),
        )
        .expect("trace");

        let recording = load_recording(temp.path()).expect("load legacy recording");

        assert_eq!(recording.manifest.bundle_version, 1);
        assert_eq!(recording.manifest.host_os, "unknown");
        assert_eq!(recording.manifest.host_arch, "unknown");
        assert_eq!(
            recording.manifest.record_command,
            "cargo run --release -p kidcli -- play-native --warp-level 01 --settled --record legacy-level2-entry"
        );
        assert_eq!(
            recording.manifest.verify_command,
            "cargo run -p kidcli -- verify-recording --slug legacy-level2-entry"
        );
        assert!(!recording.manifest.script_sha1.is_empty());
        assert!(!recording.manifest.trace_sha1.is_empty());
        assert!(recording.manifest.artifacts.is_empty());
    }

    #[test]
    fn load_recording_accepts_legacy_trace_frames_missing_newer_fields() {
        let temp = TempDir::new().expect("temp dir");
        fs::write(
            temp.path().join("manifest.json"),
            r#"{
  "slug": "legacy-trace-schema",
  "recorded_at": "123",
  "start": { "warp_level": 1, "settled": true },
  "scenario": {
    "slug": "legacy-trace-schema",
    "checkpoints": [{ "label": "start", "frame_index": 0 }],
    "frame_count": 1,
    "script": [0],
    "warp_to_level": 1
  }
}"#,
        )
        .expect("legacy manifest");
        fs::write(
            temp.path().join("trace.json"),
            br#"[
  {
    "frame_index": 0,
    "game_mode": "InGame",
    "game_mode_value": 12,
    "start_press_count": 0,
    "boot_phase": null,
    "boot_phase_elapsed": null,
    "title_screen_ready": false,
    "current_level": 0,
    "player_1_level_id": 0,
    "current_level_name": null,
    "player_start_x": null,
    "player_start_y": null,
    "flag_x": null,
    "flag_y": null,
    "level_enemy_object_count": null,
    "active_enemy_object_count": 0,
    "active_platform_object_count": 0,
    "number_lives": 3,
    "number_hitpoints": 2,
    "current_helmet_id": 0,
    "kid_x_pos": 2097152,
    "kid_y_pos": 5242880,
    "kid_x_velocity": 0,
    "kid_y_velocity": 0,
    "kid_facing": "Right",
    "kid_on_ground": true,
    "active_objects": [],
    "camera_x_pos": 0,
    "camera_y_pos": 0,
    "number_objects": 0,
    "number_gfx_objects": 0,
    "level_complete": false,
    "game_over": false,
    "game_complete": false,
    "two_player": false,
    "options_two_player_two_controllers": false,
    "options_controls_scheme": 0,
    "options_speed_button_fast": false,
    "invulnerable_frames": 0,
    "diamond_power_active": false,
    "diamond_count": 0,
    "score": 0,
    "level_timer_seconds": 0,
    "level_damage_taken": false,
    "murder_wall_x": null,
    "active_projectiles": [],
    "active_geysers": [],
    "block_bounces": [],
    "block_destroy_effects": [],
    "floating_items": [],
    "enemy_death_effects": [],
    "form_change_flash_frames": 0,
    "stomped_this_frame": false,
    "blocks_hit_this_frame": 0,
    "sound_test_play_pending": false
  }
]"#,
        )
        .expect("legacy trace");

        let recording = load_recording(temp.path()).expect("load legacy trace");
        let frame = &recording.trace[0];

        assert_eq!(frame.current_player, 0);
        assert_eq!(frame.number_continues, 0);
        assert!(!frame.paused);
        assert_eq!(frame.character_movement, None);
        assert_eq!(frame.kid_on_slope, 0);
        assert!(!frame.just_received_damage);
        assert_eq!(frame.death_windup_elapsed, 0);
        assert!(frame.gameplay_debug_fields.is_empty());
        assert_eq!(frame.physics_trace, None);
    }

    #[test]
    fn resolve_recording_bundle_input_uses_local_slug_cache() {
        let temp = TempDir::new().expect("temp dir");
        let resolved = resolve_recording_bundle_input(temp.path(), Some("level1-tree-break"), None)
            .expect("resolved slug");
        assert_eq!(
            resolved,
            temp.path()
                .join(".cache/recordings")
                .join("level1-tree-break")
        );
    }

    #[test]
    fn resolve_recording_bundle_input_accepts_bundle_path() {
        let temp = TempDir::new().expect("temp dir");
        let bundle = temp.path().join("copied-bundle");
        fs::create_dir_all(&bundle).expect("bundle dir");
        fs::write(bundle.join("manifest.json"), b"{}").expect("manifest");
        fs::write(bundle.join("trace.json"), b"[]").expect("trace");

        let resolved = resolve_recording_bundle_input(temp.path(), None, Some(bundle.as_path()))
            .expect("resolved bundle");
        assert_eq!(resolved, bundle);
    }

    #[test]
    fn resolve_recording_bundle_input_rejects_missing_selector() {
        let temp = TempDir::new().expect("temp dir");
        let error =
            resolve_recording_bundle_input(temp.path(), None, None).expect_err("missing selector");
        assert!(
            error
                .to_string()
                .contains("needs either --slug <slug> or --bundle <path>")
        );
    }

    #[test]
    fn resolve_recording_bundle_input_rejects_both_slug_and_bundle() {
        let temp = TempDir::new().expect("temp dir");
        let bundle = temp.path().join("copied-bundle");
        let error = resolve_recording_bundle_input(
            temp.path(),
            Some("level1-tree-break"),
            Some(bundle.as_path()),
        )
        .expect_err("both selector types");
        assert!(
            error
                .to_string()
                .contains("pass either --slug <slug> or --bundle <path>, not both")
        );
    }
}
