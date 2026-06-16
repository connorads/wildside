use super::*;
use crate::capture::{AUDIO_FRAME_SAMPLES, read_wav_f32, write_wav_with_channels};
use crate::diff::{annotate_diff, generate_diff_composite};
use crate::native_player::{
    BackgroundPixelSample, FinalScenePixelDiagnostic, LayerFilter, NATIVE_FRAME_HEIGHT,
    NATIVE_FRAME_WIDTH, NativeEolScoreSpriteDiagnostic, NativePlayOptions, build_scene,
    build_scene_filtered, eol_score_sprite_diagnostics, load_session_assets, render_scene,
    render_scene_overlay, render_trace_with_vdp_latched_state, render_vdp_trace_overlay,
    sample_background_pixel_loc_c070, sample_scene_pixel_provenance,
};
use crate::oracle::{OracleFrame, OracleManifest, OracleState, run_jgenesis_capture_dynamic};
use crate::scenarios::{DynamicCheckpoint, DynamicScenario};
use crate::verify::{compare_rgb_frames, load_png_rgb};
use image::{ColorType, ImageFormat};
use kid_core::{FrameInput, GameMode, LevelId, NativeRuntime, NativeTraceFrame, PlayerForm};
#[cfg(test)]
use kid_core::{
    BlockBounceTrace, BlockBounceTraceKind, IslandShoreDmaUploadLoc1202A, NativeObjectKind,
    NativeObjectSnapshot,
};
use kid_render::NativeLevelVisuals;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

const RECORDING_CHECKPOINT_INTERVAL: usize = 60;
const RECORDING_LEVEL_CARD_HOLD_FRAMES: usize = 60;
const RECORDING_BUNDLE_VERSION: u32 = 4;

const RECORDING_NOTES_FILENAME: &str = "notes.txt";
const RECORDING_TRANSCRIPT_FILENAME: &str = "transcript.txt";
const RECORDING_FINAL_FRAME_FILENAME: &str = "final-frame.png";
const RECORDING_AUDIO_FILENAME: &str = "audio.wav";
const RECORDING_AUDIO_EVENTS_FILENAME: &str = "audio-events.txt";
const RECORDING_AUDIO_DIAGNOSTICS_FILENAME: &str = "audio-diagnostics.txt";
const NATIVE_DIAGNOSTIC_LAYERS: [&str; 9] = [
    "background",
    "terrain",
    "block_overrides",
    "platforms",
    "enemies",
    "player",
    "effects",
    "eol_score",
    "hud",
];

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct RecordingArtifactManifest {
    pub(crate) path: String,
    pub(crate) kind: String,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct RecordingBundleArtifacts {
    pub(crate) user_notes: Option<String>,
    pub(crate) final_frame_rgb: Option<Vec<u8>>,
    pub(crate) audio_samples: Option<Vec<f32>>,
    pub(crate) audio_sample_rate: u32,
    pub(crate) audio_channels: u16,
    pub(crate) audio_events: Vec<String>,
    pub(crate) audio_diagnostics: Vec<String>,
}

impl Default for RecordingBundleArtifacts {
    fn default() -> Self {
        Self {
            user_notes: None,
            final_frame_rgb: None,
            audio_samples: None,
            audio_sample_rate: 44_100,
            audio_channels: 0,
            audio_events: Vec::new(),
            audio_diagnostics: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct RecordingStart {
    pub(crate) warp_level: Option<u8>,
    pub(crate) settled: bool,
    #[serde(default)]
    pub(crate) record_frames: Option<usize>,
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
    #[serde(default, skip)]
    pub(crate) legacy_block_destroy_direction_frames: BTreeSet<usize>,
    #[serde(default, skip)]
    pub(crate) legacy_missing_played_sound_effect_ids_frames: BTreeSet<usize>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub(crate) struct RecordingVerificationReport {
    pub(crate) slug: String,
    #[serde(default)]
    pub(crate) native_replay_only: bool,
    pub(crate) trace_match: bool,
    pub(crate) oracle_state_match: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) oracle_state_error: Option<String>,
    pub(crate) oracle_frame_match: bool,
    pub(crate) replay_trace_len: usize,
    pub(crate) recorded_trace_len: usize,
    pub(crate) first_trace_drift: Option<TraceDriftReport>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub(crate) checkpoint_state_deltas: Vec<CheckpointStateDelta>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub(crate) audio_event_windows: Vec<AudioEventWindowReport>,
    pub(crate) frame_diffs: Vec<CheckpointFrameDiff>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub(crate) struct AudioEventWindowReport {
    pub(crate) frame_index: usize,
    pub(crate) command: String,
    pub(crate) window_start_frame: usize,
    pub(crate) window_end_frame: usize,
    pub(crate) rms: f32,
    pub(crate) peak: f32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct CheckpointStateDelta {
    pub(crate) label: String,
    pub(crate) frame_index: usize,
    pub(crate) field: String,
    pub(crate) native_value: String,
    pub(crate) oracle_value: String,
    pub(crate) delta: String,
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
    #[serde(default)]
    pub(crate) changed_pixel_count: usize,
    #[serde(default)]
    pub(crate) changed_bounds: Option<PixelDiffBounds>,
    #[serde(default)]
    pub(crate) top_colour_diffs: Vec<PixelColourDiff>,
    #[serde(default)]
    pub(crate) native_background_samples: Vec<NativeBackgroundPixelDiagnostic>,
    #[serde(default)]
    pub(crate) native_layer_samples: Vec<NativeLayerPixelDiagnostic>,
    #[serde(default)]
    pub(crate) native_final_scene_samples: Vec<FinalScenePixelDiagnostic>,
    #[serde(default)]
    pub(crate) native_vdp_samples: Vec<NativeVdpPixelDiagnostic>,
    #[serde(default)]
    pub(crate) native_eol_score_sprites: Vec<NativeEolScoreSpriteDiagnostic>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub(crate) oracle_gameplay_debug_fields: BTreeMap<String, String>,
    pub(crate) diff_path: PathBuf,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct PixelDiffBounds {
    pub(crate) min_x: usize,
    pub(crate) min_y: usize,
    pub(crate) max_x: usize,
    pub(crate) max_y: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct PixelColourDiff {
    pub(crate) count: usize,
    pub(crate) native_rgb: [u8; 3],
    pub(crate) oracle_rgb: [u8; 3],
    #[serde(default)]
    pub(crate) sample_pixels: Vec<PixelCoord>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct PixelDiffSummary {
    changed_pixel_count: usize,
    changed_bounds: Option<PixelDiffBounds>,
    top_colour_diffs: Vec<PixelColourDiff>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct PixelCoord {
    pub(crate) x: usize,
    pub(crate) y: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct NativeBackgroundPixelDiagnostic {
    pub(crate) x: usize,
    pub(crate) y: usize,
    pub(crate) sample: Option<NativeBackgroundPixelSample>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct NativeLayerPixelDiagnostic {
    pub(crate) x: usize,
    pub(crate) y: usize,
    pub(crate) samples: Vec<NativeLayerPixelSample>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct NativeLayerPixelSample {
    pub(crate) layer: String,
    pub(crate) rgb: [u8; 3],
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct NativeVdpPixelDiagnostic {
    pub(crate) x: usize,
    pub(crate) y: usize,
    pub(crate) layers: Vec<NativeVdpLayerPixelSample>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct NativeVdpLayerPixelSample {
    pub(crate) layer: String,
    pub(crate) priority: bool,
    pub(crate) map_base: usize,
    pub(crate) cell_x: usize,
    pub(crate) cell_y: usize,
    pub(crate) cell_index: usize,
    pub(crate) entry: u16,
    pub(crate) tile_index: usize,
    pub(crate) palette_line: usize,
    pub(crate) colour_index: u8,
    pub(crate) source_x: usize,
    pub(crate) source_y: usize,
    pub(crate) rgb: Option<[u8; 3]>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct NativeBackgroundPixelSample {
    pub(crate) screen_x: i32,
    pub(crate) screen_y: i32,
    pub(crate) tile_origin_x: i32,
    pub(crate) tile_origin_y: i32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) background_row_index: Option<usize>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) background_row_selector: Option<u8>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) background_column: Option<usize>,
    pub(crate) tile_index: u16,
    pub(crate) palette_line: u8,
    pub(crate) palette_index: u8,
    pub(crate) source_x: u8,
    pub(crate) source_y: u8,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) row_offset: Option<i32>,
    pub(crate) rgb: [u8; 3],
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
struct PixelColourDiffAccumulator {
    count: usize,
    sample_pixels: Vec<PixelCoord>,
}

pub(crate) fn recording_start_from_options(options: &NativePlayOptions) -> RecordingStart {
    RecordingStart {
        warp_level: options.warp_level,
        settled: options.settled,
        record_frames: options.record_frames,
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
                start_position: None,
                start_velocity: None,
                camera_position: None,
                playtest_form: None,
                initial_diamond_count: None,
            initial_lives: None,
                warp_to_level: start.warp_level,
                settle_oracle_warp: start.settled && start.warp_level.is_some(),
            },
        },
        trace,
        legacy_block_destroy_direction_frames: BTreeSet::new(),
        legacy_missing_played_sound_effect_ids_frames: BTreeSet::new(),
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
    let legacy_block_destroy_direction_frames = legacy_block_destroy_direction_frames(&trace)?;
    let legacy_missing_played_sound_effect_ids_frames =
        legacy_missing_played_sound_effect_ids_frames(&trace)?;
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
        legacy_block_destroy_direction_frames,
        legacy_missing_played_sound_effect_ids_frames,
    };
    normalise_loaded_recording(&mut recording)?;
    Ok(recording)
}

fn legacy_block_destroy_direction_frames(trace: &[u8]) -> Result<BTreeSet<usize>> {
    let value: serde_json::Value =
        serde_json::from_slice(trace).context("failed to inspect trace.json schema")?;
    let Some(frames) = value.as_array() else {
        return Ok(BTreeSet::new());
    };

    Ok(frames
        .iter()
        .enumerate()
        .filter_map(|(frame_index, frame)| {
            let effects = frame.get("block_destroy_effects")?.as_array()?;
            effects
                .iter()
                .any(|effect| effect.as_array().is_some_and(|fields| fields.len() == 3))
                .then_some(frame_index)
        })
        .collect())
}

fn legacy_missing_played_sound_effect_ids_frames(trace: &[u8]) -> Result<BTreeSet<usize>> {
    let value: serde_json::Value =
        serde_json::from_slice(trace).context("failed to inspect trace.json schema")?;
    let Some(frames) = value.as_array() else {
        return Ok(BTreeSet::new());
    };

    Ok(frames
        .iter()
        .enumerate()
        .filter_map(|(frame_index, frame)| {
            frame
                .as_object()
                .is_some_and(|fields| !fields.contains_key("played_sound_effect_ids"))
                .then_some(frame_index)
        })
        .collect())
}

pub(crate) fn run_recorded_trace(
    manifest: &AssetManifest,
    start: RecordingStart,
    scenario: &DynamicScenario,
) -> Result<Vec<NativeTraceFrame>> {
    let RecordingRuntimeInit { mut runtime, .. } =
        initialise_recording_runtime(manifest, start, scenario)?;

    let mut trace = runtime.trace_frame();
    let mut current_level_id = trace.current_level.0;
    let frame_count = recording_trace_frame_count(scenario);
    let mut replay = Vec::with_capacity(frame_count);
    let mut level_card_frames_remaining = 0usize;

    while replay.len() < frame_count {
        if level_card_frames_remaining == 0 {
            let buttons = recording_script_buttons_at_frame(&start, scenario, replay.len());
            let previous_trace = trace.clone();
            trace = runtime.step_frame(FrameInput::from_mask(buttons));
            let mut recorded_frame = trace.clone();
            if start.settled && start.warp_level.is_some() {
                // The oracle reaches settled warp recordings through the ROM
                // level-skip cheat and leaves `LevelSkip_Cheat` armed. Preserve
                // that reduced-scenario visual state so ASM loc_D468's
                // completion-time digit sprites can be rendered.
                recorded_frame.level_skip_cheat_enabled = 1;
            }

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

    if start.settled {
        rebase_trace_frame_indices(&mut replay);
    }

    Ok(replay)
}

fn recording_trace_frame_count(scenario: &DynamicScenario) -> usize {
    scenario.frame_count.max(scenario.script.len())
}

fn recording_script_buttons_at_frame(
    start: &RecordingStart,
    scenario: &DynamicScenario,
    frame_index: usize,
) -> u8 {
    // ASM Mode_Level (kid.asm:877-917) executes objects/player logic before
    // ReadJoypad, and settled recording oracles start from the helper's live
    // gameplay window. Keep the same one-step input feed used by
    // ScriptRunner::RecordingOracle so script byte N affects gameplay on
    // traced frame N+2 after NativeRuntime::gameplay_input consumes its held
    // latch.
    let input_index = if start.settled && start.warp_level.is_some() {
        frame_index.checked_sub(1)
    } else {
        Some(frame_index)
    };
    input_index
        .and_then(|index| scenario.script.get(index))
        .copied()
        .unwrap_or_default()
}

struct RecordingRuntimeInit {
    runtime: NativeRuntime,
    level_visuals: NativeLevelVisuals,
    pre_probe_render_trace: Option<NativeTraceFrame>,
}

fn apply_recording_probe_state(
    runtime: &mut NativeRuntime,
    scenario: &DynamicScenario,
) -> Result<()> {
    if let Some(helmet_id) = scenario.playtest_form {
        let form = PlayerForm::from_helmet_id(helmet_id)
            .ok_or_else(|| anyhow!("invalid playtest_form helmet id {helmet_id}"))?;
        runtime.set_playtest_form(form);
    }
    if let Some(count) = scenario.initial_diamond_count {
        runtime.set_probe_diamond_count(count);
    }
    if let Some(count) = scenario.initial_lives {
        runtime.set_probe_lives_count(count);
    }
    Ok(())
}

fn initialise_recording_runtime(
    manifest: &AssetManifest,
    start: RecordingStart,
    scenario: &DynamicScenario,
) -> Result<RecordingRuntimeInit> {
    let initial_level_id = start.warp_level.unwrap_or(0x00);
    let mut runtime = if start.settled && start.warp_level.is_some() {
        NativeRuntime::with_generated_level_layout(manifest.clone())
    } else {
        let (_, block_grid) = load_session_assets(manifest, initial_level_id)?;
        let mut runtime = NativeRuntime::new(manifest.clone());
        runtime.set_block_grid(block_grid);
        runtime
    };

    if let Some(level_id) = start.warp_level {
        let level_id = LevelId(level_id);
        if start.settled {
            runtime.boot_to_level(level_id);
            runtime.settle_recording_oracle_warp_live_window(level_id);
        } else {
            runtime.warp_to_level(level_id);
        }
    } else if start.settled {
        runtime.enter_level_playtest(LevelId(0));
    }

    let pre_probe_render_trace = scenario
        .start_position
        .is_some()
        .then(|| runtime.trace_frame());
    if let Some(position) = scenario.start_position {
        runtime.set_player_probe_position(position.x, position.y);
    }
    if let Some(position) = scenario.camera_position {
        runtime.set_camera_probe_position(position.x, position.y);
    }
    apply_recording_probe_state(&mut runtime, scenario)?;

    let current_level_id = runtime.trace_frame().current_level.0;
    let (level_visuals, _) = load_session_assets(manifest, current_level_id)?;
    Ok(RecordingRuntimeInit {
        runtime,
        level_visuals,
        pre_probe_render_trace,
    })
}

pub(crate) fn verify_recording(
    repo_root: &Path,
    rom_path: &Path,
    manifest: &AssetManifest,
    record_to: &Path,
    native_only: bool,
) -> Result<()> {
    let record_dir = resolve_recording_dir(record_to)?;
    let recording = load_recording(&record_dir)?;
    let replay_trace = run_recorded_trace(
        manifest,
        recording.manifest.start,
        &recording.manifest.scenario,
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

    let first_trace_drift = first_trace_drift(
        &recording.trace,
        &replay_trace,
        &recording.legacy_block_destroy_direction_frames,
        &recording.legacy_missing_played_sound_effect_ids_frames,
        recording.manifest.start.settled && recording.manifest.start.warp_level.is_some(),
    );
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

    let native_frames = render_dynamic_checkpoint_frames(
        manifest,
        &recording.manifest.start,
        &recording.manifest.scenario,
        recording_visual_frame_offset(&recording.manifest.start),
    )?;
    write_rendered_frames(&native_dir, &native_frames)?;
    write_dynamic_checkpoint_layer_frames(
        manifest,
        &recording.manifest.start,
        &recording.manifest.scenario,
        recording_visual_frame_offset(&recording.manifest.start),
        &native_dir.join("layers"),
    )?;

    let (
        oracle_state_match,
        oracle_state_error,
        oracle_frame_match,
        checkpoint_state_deltas,
        frame_diffs,
    ) = if native_only {
        (true, None, true, Vec::new(), Vec::new())
    } else {
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

        // Probe recordings may patch the camera only to put a mid-level target
        // on screen. Keep Kid/movement/form/object checks authoritative while
        // allowing those diagnostic camera coordinates to drift independently.
        let comparison_options = StateComparisonOptions {
            ignore_camera_position: recording.manifest.scenario.camera_position.is_some(),
        };
        let (oracle_state_match, oracle_state_error) =
            match compare_native_states_against_oracle_with_options(
                &record_dir,
                &native_states,
                &oracle.states,
                comparison_options,
            ) {
                Ok(()) => (true, None),
                Err(error) => (false, Some(error.to_string())),
            };
        let checkpoint_state_deltas = checkpoint_state_deltas(&native_states, &oracle.states);
        let mut frame_diffs =
            compare_recording_frames(
                &native_dir,
                &native_frames,
                &oracle.frames,
                recording.manifest.start.warp_level.unwrap_or_else(|| {
                    replay_trace.last().map_or(0, |frame| frame.current_level.0)
                }),
            )?;
        attach_native_pixel_samples(
            manifest,
            &recording.manifest.start,
            &recording.manifest.scenario,
            recording_visual_frame_offset(&recording.manifest.start),
            &mut frame_diffs,
        )?;
        attach_oracle_gameplay_debug_fields(&oracle, &mut frame_diffs);
        let oracle_frame_match = frame_diffs
            .iter()
            .all(|diff| diff.max_channel_delta == 0 && diff.mean_absolute_error == 0.0);
        (
            oracle_state_match,
            oracle_state_error,
            oracle_frame_match,
            checkpoint_state_deltas,
            frame_diffs,
        )
    };

    let audio_event_windows = recording_audio_event_windows(&record_dir)?;
    let report = RecordingVerificationReport {
        slug: recording.manifest.slug.clone(),
        native_replay_only: native_only,
        trace_match,
        oracle_state_match,
        oracle_state_error,
        oracle_frame_match,
        replay_trace_len: replay_trace.len(),
        recorded_trace_len: recording.trace.len(),
        first_trace_drift,
        checkpoint_state_deltas,
        audio_event_windows,
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
    if !native_only && !report.oracle_state_match {
        let state_error = report
            .oracle_state_error
            .as_deref()
            .unwrap_or("unknown oracle state mismatch");
        bail!(
            "recording checkpoints diverged from oracle state: {}; artefacts in {}",
            state_error,
            verify_dir.display()
        );
    }
    if !native_only && !report.oracle_frame_match {
        bail!(
            "recording checkpoint frames diverged from oracle pixels; artefacts in {}",
            verify_dir.display()
        );
    }

    if native_only {
        println!(
            "native recording replay verification passed for {}: {}",
            recording.manifest.slug,
            verify_dir.display()
        );
    } else {
        println!(
            "recording verification passed for {}: {}",
            recording.manifest.slug,
            verify_dir.display()
        );
    }
    Ok(())
}

fn recording_audio_event_windows(record_dir: &Path) -> Result<Vec<AudioEventWindowReport>> {
    let audio_path = record_dir.join(RECORDING_AUDIO_FILENAME);
    let events_path = record_dir.join(RECORDING_AUDIO_EVENTS_FILENAME);
    if !audio_path.is_file() || !events_path.is_file() {
        return Ok(Vec::new());
    }

    let samples = read_wav_f32(&audio_path)?;
    let events = fs::read_to_string(&events_path)
        .with_context(|| format!("failed to read {}", events_path.display()))?;
    Ok(events
        .lines()
        .filter_map(parse_audio_event_line)
        .map(|(frame_index, command)| audio_event_window_report(&samples, frame_index, command))
        .collect())
}

fn parse_audio_event_line(line: &str) -> Option<(usize, String)> {
    let frame_index = line
        .split_whitespace()
        .find_map(|part| part.strip_prefix("frame="))?
        .parse()
        .ok()?;
    let command = line
        .split_whitespace()
        .find_map(|part| part.strip_prefix("command="))?
        .to_owned();
    Some((frame_index, command))
}

fn audio_event_window_report(
    samples: &[f32],
    frame_index: usize,
    command: String,
) -> AudioEventWindowReport {
    const WINDOW_FRAMES: usize = 8;

    let window_start_frame = frame_index;
    let window_end_frame = frame_index.saturating_add(WINDOW_FRAMES);
    let sample_start = window_start_frame
        .saturating_mul(AUDIO_FRAME_SAMPLES)
        .saturating_mul(2)
        .min(samples.len());
    let sample_end = window_end_frame
        .saturating_mul(AUDIO_FRAME_SAMPLES)
        .saturating_mul(2)
        .min(samples.len());
    let window = &samples[sample_start..sample_end];
    let (rms, peak) = stereo_window_rms_peak(window);

    AudioEventWindowReport {
        frame_index,
        command,
        window_start_frame,
        window_end_frame,
        rms,
        peak,
    }
}

fn stereo_window_rms_peak(samples: &[f32]) -> (f32, f32) {
    if samples.is_empty() {
        return (0.0, 0.0);
    }

    let mut sum_squares = 0.0f64;
    let mut sample_pairs = 0usize;
    let mut peak = 0.0f32;
    for frame in samples.chunks_exact(2) {
        let left = frame[0];
        let right = frame[1];
        sum_squares += f64::from((left * left + right * right) * 0.5);
        peak = peak.max(left.abs()).max(right.abs());
        sample_pairs += 1;
    }
    if sample_pairs == 0 {
        return (0.0, peak);
    }

    ((sum_squares / sample_pairs as f64).sqrt() as f32, peak)
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

    (
        DynamicScenario {
            slug: format!(
                "{}_settled_recording_oracle",
                recording.manifest.scenario.slug
            ),
            frame_count: recording.manifest.scenario.frame_count,
            script: recording.manifest.scenario.script.clone(),
            checkpoints: recording.manifest.scenario.checkpoints.clone(),
            start_position: recording.manifest.scenario.start_position,
            start_velocity: recording.manifest.scenario.start_velocity,
            camera_position: recording.manifest.scenario.camera_position,
            playtest_form: recording.manifest.scenario.playtest_form,
            initial_diamond_count: recording.manifest.scenario.initial_diamond_count,
            initial_lives: recording.manifest.scenario.initial_lives,
            warp_to_level: Some(recording.manifest.start.warp_level.unwrap_or(0)),
            settle_oracle_warp: true,
        },
        0,
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

fn recording_visual_frame_offset(_start: &RecordingStart) -> usize {
    0
}

fn loc_c070_render_latch_trace<'a>(
    trace: &NativeTraceFrame,
    replay_frame_index: usize,
    pre_probe_render_trace: Option<&'a NativeTraceFrame>,
    previous_trace: &'a NativeTraceFrame,
) -> &'a NativeTraceFrame {
    if replay_frame_index == 0
        && trace.level_complete
        && trace.level_complete_elapsed_frames == 0
        && let Some(pre_probe_render_trace) = pre_probe_render_trace
    {
        // ASM CharacterCollision_TouchFlagpole loc_C070 runs after the frame's
        // VDP sprite table has already been presented. Probe recordings patch
        // Kid's live object position just before input, matching the oracle
        // helper's `apply_probe_start_position`, but that WRAM edit does not
        // rewrite the already-latched sprite table until the next frame.
        return pre_probe_render_trace;
    }
    previous_trace
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
    start: &RecordingStart,
    scenario: &DynamicScenario,
    visual_frame_offset: usize,
) -> Result<Vec<RenderedFrame>> {
    let RecordingRuntimeInit {
        mut runtime,
        mut level_visuals,
        pre_probe_render_trace,
    } = initialise_recording_runtime(manifest, *start, scenario)?;

    let mut current_level_id = runtime.trace_frame().current_level.0;
    let mut checkpoints = scenario.checkpoints.iter().peekable();
    let mut rendered_frames = Vec::with_capacity(scenario.checkpoints.len());
    let mut replay_frame_index = 0usize;
    let mut level_card_frames_remaining = 0usize;

    let frame_count = recording_trace_frame_count(scenario);
    while replay_frame_index < frame_count {
        if level_card_frames_remaining > 0 {
            level_card_frames_remaining = level_card_frames_remaining.saturating_sub(1);
            continue;
        }

        let previous_trace = runtime.trace_frame();
        let buttons = recording_script_buttons_at_frame(start, scenario, replay_frame_index);
        let mut trace = runtime.step_frame(FrameInput::from_mask(buttons));
        if start.settled && start.warp_level.is_some() {
            apply_settled_warp_oracle_visual_compatibility(&mut trace);
        }

        let entered_gameplay = !matches!(previous_trace.game_mode, GameMode::InGame)
            && matches!(trace.game_mode, GameMode::InGame)
            && start.warp_level.is_none()
            && !start.settled;
        if entered_gameplay {
            level_card_frames_remaining = RECORDING_LEVEL_CARD_HOLD_FRAMES;
        }

        let level_changed = trace.current_level.0 != current_level_id;
        if level_changed {
            current_level_id = trace.current_level.0;
            let (new_visuals, _) = load_session_assets(manifest, current_level_id)?;
            level_visuals = new_visuals;
            level_card_frames_remaining = RECORDING_LEVEL_CARD_HOLD_FRAMES;
        }

        while checkpoints
            .peek()
            .is_some_and(|checkpoint| checkpoint.frame_index == replay_frame_index)
        {
            let checkpoint = checkpoints.next().expect("checkpoint should exist");
            let render_latch = loc_c070_render_latch_trace(
                &trace,
                replay_frame_index,
                pre_probe_render_trace.as_ref(),
                &previous_trace,
            );
            let mut render_trace = render_trace_with_vdp_latched_state(&trace, Some(render_latch));
            if level_card_frames_remaining > 0 {
                render_trace.game_mode = GameMode::LevelTransition;
                render_trace.game_mode_value = GameMode::LevelTransition.as_disassembly_value();
            }
            render_trace.frame_index = render_trace.frame_index.saturating_add(visual_frame_offset);
            let rgb = render_runtime_checkpoint_frame(&runtime, &render_trace, &level_visuals);
            rendered_frames.push((
                checkpoint.label.clone(),
                checkpoint.frame_index,
                NATIVE_FRAME_WIDTH as usize,
                NATIVE_FRAME_HEIGHT as usize,
                rgb,
            ));
        }

        level_card_frames_remaining = level_card_frames_remaining.saturating_sub(1);
        replay_frame_index += 1;
    }

    if let Some(checkpoint) = checkpoints.peek() {
        bail!(
            "missing native frame {}@{}",
            checkpoint.label,
            checkpoint.frame_index
        );
    }

    Ok(rendered_frames)
}

fn apply_settled_warp_oracle_visual_compatibility(trace: &mut NativeTraceFrame) {
    // Settled oracle recordings enter through the ROM level-skip path. That
    // leaves `LevelSkip_Cheat` armed and seeds `Level_completion_time` to one
    // second before ASM loc_D468 draws the end-level digit sprites. The live
    // HUD timer still comes from the authoritative trace; adding a synthetic
    // second here makes settled in-game checkpoint PNGs one tick ahead of
    // oracle while state parity remains green.
    trace.level_skip_cheat_enabled = 1;
    trace.level_completion_time = 1;
}

fn render_runtime_checkpoint_frame(
    runtime: &NativeRuntime,
    trace: &NativeTraceFrame,
    level_visuals: &NativeLevelVisuals,
) -> Vec<u8> {
    let mut rgb = vec![0; (NATIVE_FRAME_WIDTH * NATIVE_FRAME_HEIGHT * 3) as usize];
    if let Some(vdp) = runtime.vdp_state() {
        // ASM intro/title loaders build the visible screen in VDP memory.
        // Recording verification must use the same VDP renderer as play-native,
        // otherwise intro checkpoints fall back to black trace-only placeholders.
        let snapshot = kid_md::VdpSnapshot {
            registers: vdp.registers,
            vram: vdp.vram.clone(),
            cram: vdp.cram.clone(),
            vsram: vdp.vsram.clone(),
        };
        kid_md::render_vdp_into(&snapshot, &mut rgb);
        render_vdp_trace_overlay(trace, level_visuals, &mut rgb);
    } else {
        let scene = build_scene(trace, false, Some(level_visuals), runtime.block_grid());
        render_scene(&scene, &mut rgb);
    }
    rgb
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

fn write_dynamic_checkpoint_layer_frames(
    manifest: &AssetManifest,
    start: &RecordingStart,
    scenario: &DynamicScenario,
    visual_frame_offset: usize,
    out_dir: &Path,
) -> Result<()> {
    let RecordingRuntimeInit {
        mut runtime,
        mut level_visuals,
        pre_probe_render_trace,
    } = initialise_recording_runtime(manifest, *start, scenario)?;

    let mut current_level_id = runtime.trace_frame().current_level.0;
    let mut checkpoints = scenario.checkpoints.iter().peekable();
    let mut replay_frame_index = 0usize;
    let mut level_card_frames_remaining = 0usize;

    let frame_count = recording_trace_frame_count(scenario);
    while replay_frame_index < frame_count {
        if level_card_frames_remaining > 0 {
            level_card_frames_remaining = level_card_frames_remaining.saturating_sub(1);
            continue;
        }

        let previous_trace = runtime.trace_frame();
        let buttons = recording_script_buttons_at_frame(start, scenario, replay_frame_index);
        let mut trace = runtime.step_frame(FrameInput::from_mask(buttons));
        if start.settled && start.warp_level.is_some() {
            apply_settled_warp_oracle_visual_compatibility(&mut trace);
        }

        let entered_gameplay = !matches!(previous_trace.game_mode, GameMode::InGame)
            && matches!(trace.game_mode, GameMode::InGame)
            && start.warp_level.is_none()
            && !start.settled;
        if entered_gameplay {
            level_card_frames_remaining = RECORDING_LEVEL_CARD_HOLD_FRAMES;
        }

        let level_changed = trace.current_level.0 != current_level_id;
        if level_changed {
            current_level_id = trace.current_level.0;
            let (new_visuals, _) = load_session_assets(manifest, current_level_id)?;
            level_visuals = new_visuals;
            level_card_frames_remaining = RECORDING_LEVEL_CARD_HOLD_FRAMES;
        }

        while checkpoints
            .peek()
            .is_some_and(|checkpoint| checkpoint.frame_index == replay_frame_index)
        {
            let checkpoint = checkpoints.next().expect("checkpoint should exist");
            let render_latch = loc_c070_render_latch_trace(
                &trace,
                replay_frame_index,
                pre_probe_render_trace.as_ref(),
                &previous_trace,
            );
            let mut render_trace = render_trace_with_vdp_latched_state(&trace, Some(render_latch));
            if level_card_frames_remaining > 0 {
                render_trace.game_mode = GameMode::LevelTransition;
                render_trace.game_mode_value = GameMode::LevelTransition.as_disassembly_value();
            }
            render_trace.frame_index = render_trace.frame_index.saturating_add(visual_frame_offset);
            for layer in NATIVE_DIAGNOSTIC_LAYERS {
                let scene = build_scene_filtered(
                    &render_trace,
                    false,
                    Some(&level_visuals),
                    runtime.block_grid(),
                    LayerFilter::solo(layer),
                );
                let mut rgb = vec![0; (NATIVE_FRAME_WIDTH * NATIVE_FRAME_HEIGHT * 3) as usize];
                render_scene(&scene, &mut rgb);
                let layer_dir = out_dir.join(layer);
                fs::create_dir_all(&layer_dir)
                    .with_context(|| format!("failed to create {}", layer_dir.display()))?;
                let path = layer_dir.join(format!(
                    "{:06}_{}.png",
                    checkpoint.frame_index, checkpoint.label
                ));
                image::save_buffer_with_format(
                    &path,
                    &rgb,
                    NATIVE_FRAME_WIDTH,
                    NATIVE_FRAME_HEIGHT,
                    ColorType::Rgb8,
                    ImageFormat::Png,
                )
                .with_context(|| format!("failed to write {}", path.display()))?;
            }
        }

        level_card_frames_remaining = level_card_frames_remaining.saturating_sub(1);
        replay_frame_index += 1;
    }

    if let Some(checkpoint) = checkpoints.peek() {
        bail!(
            "missing native layer frame {}@{}",
            checkpoint.label,
            checkpoint.frame_index
        );
    }

    Ok(())
}

fn attach_oracle_gameplay_debug_fields(
    oracle: &OracleManifest,
    frame_diffs: &mut [CheckpointFrameDiff],
) {
    let states_by_checkpoint = oracle
        .states
        .iter()
        .map(|state| ((state.frame_index, state.label.clone()), state))
        .collect::<BTreeMap<_, _>>();

    for diff in frame_diffs {
        if diff.changed_pixel_count == 0 {
            continue;
        }
        let key = (diff.frame_index, diff.label.clone());
        let Some(state) = states_by_checkpoint.get(&key) else {
            continue;
        };
        diff.oracle_gameplay_debug_fields = state
            .gameplay_debug_fields
            .iter()
            .filter(|(field, _)| {
                field.starts_with("eol_")
                    || field.as_str() == "kid_collision_source"
                    || field.as_str() == "kid_collision_type"
            })
            .map(|(field, value)| (field.clone(), value.clone()))
            .collect();
    }
}

fn attach_native_pixel_samples(
    manifest: &AssetManifest,
    start: &RecordingStart,
    scenario: &DynamicScenario,
    visual_frame_offset: usize,
    frame_diffs: &mut [CheckpointFrameDiff],
) -> Result<()> {
    let sample_requests = frame_diffs
        .iter()
        .filter(|diff| diff.changed_pixel_count > 0)
        .map(|diff| {
            let mut pixels = BTreeSet::new();
            for colour_diff in &diff.top_colour_diffs {
                for pixel in &colour_diff.sample_pixels {
                    pixels.insert((pixel.x, pixel.y));
                }
            }
            ((diff.frame_index, diff.label.clone()), pixels)
        })
        .collect::<BTreeMap<_, _>>();
    if sample_requests.is_empty() {
        return Ok(());
    }

    let RecordingRuntimeInit {
        mut runtime,
        mut level_visuals,
        pre_probe_render_trace,
    } = initialise_recording_runtime(manifest, *start, scenario)?;
    let mut current_level_id = runtime.trace_frame().current_level.0;
    let mut checkpoints = scenario.checkpoints.iter().peekable();
    let mut replay_frame_index = 0usize;
    let mut level_card_frames_remaining = 0usize;
    let mut background_samples_by_checkpoint: BTreeMap<
        (usize, String),
        Vec<NativeBackgroundPixelDiagnostic>,
    > = BTreeMap::new();
    let mut layer_samples_by_checkpoint: BTreeMap<
        (usize, String),
        Vec<NativeLayerPixelDiagnostic>,
    > = BTreeMap::new();
    let mut final_scene_samples_by_checkpoint: BTreeMap<
        (usize, String),
        Vec<FinalScenePixelDiagnostic>,
    > = BTreeMap::new();
    let mut vdp_samples_by_checkpoint: BTreeMap<(usize, String), Vec<NativeVdpPixelDiagnostic>> =
        BTreeMap::new();
    let mut eol_sprites_by_checkpoint: BTreeMap<
        (usize, String),
        Vec<NativeEolScoreSpriteDiagnostic>,
    > = BTreeMap::new();

    let frame_count = recording_trace_frame_count(scenario);
    while replay_frame_index < frame_count {
        if level_card_frames_remaining > 0 {
            level_card_frames_remaining = level_card_frames_remaining.saturating_sub(1);
            continue;
        }

        let previous_trace = runtime.trace_frame();
        let buttons = recording_script_buttons_at_frame(start, scenario, replay_frame_index);
        let mut trace = runtime.step_frame(FrameInput::from_mask(buttons));
        if start.settled && start.warp_level.is_some() {
            apply_settled_warp_oracle_visual_compatibility(&mut trace);
        }

        let entered_gameplay = !matches!(previous_trace.game_mode, GameMode::InGame)
            && matches!(trace.game_mode, GameMode::InGame)
            && start.warp_level.is_none()
            && !start.settled;
        if entered_gameplay {
            level_card_frames_remaining = RECORDING_LEVEL_CARD_HOLD_FRAMES;
        }

        let level_changed = trace.current_level.0 != current_level_id;
        if level_changed {
            current_level_id = trace.current_level.0;
            let (new_visuals, _) = load_session_assets(manifest, current_level_id)?;
            level_visuals = new_visuals;
            level_card_frames_remaining = RECORDING_LEVEL_CARD_HOLD_FRAMES;
        }

        while checkpoints
            .peek()
            .is_some_and(|checkpoint| checkpoint.frame_index == replay_frame_index)
        {
            let checkpoint = checkpoints.next().expect("checkpoint should exist");
            let key = (checkpoint.frame_index, checkpoint.label.clone());
            if let Some(pixels) = sample_requests.get(&key) {
                let render_latch = loc_c070_render_latch_trace(
                    &trace,
                    replay_frame_index,
                    pre_probe_render_trace.as_ref(),
                    &previous_trace,
                );
                let mut render_trace =
                    render_trace_with_vdp_latched_state(&trace, Some(render_latch));
                if level_card_frames_remaining > 0 {
                    render_trace.game_mode = GameMode::LevelTransition;
                    render_trace.game_mode_value = GameMode::LevelTransition.as_disassembly_value();
                }
                render_trace.frame_index =
                    render_trace.frame_index.saturating_add(visual_frame_offset);
                let background_samples = pixels
                    .iter()
                    .map(|(x, y)| {
                        let sample = i32::try_from(*x)
                            .ok()
                            .zip(i32::try_from(*y).ok())
                            .and_then(|(sample_x, sample_y)| {
                                sample_background_pixel_loc_c070(
                                    &render_trace,
                                    &level_visuals,
                                    sample_x,
                                    sample_y,
                                )
                            })
                            .map(NativeBackgroundPixelSample::from);
                        NativeBackgroundPixelDiagnostic {
                            x: *x,
                            y: *y,
                            sample,
                        }
                    })
                    .collect::<Vec<_>>();
                let layer_samples = sample_native_layers_for_pixels(
                    &render_trace,
                    &level_visuals,
                    runtime.block_grid(),
                    pixels,
                );
                let scene = build_scene(
                    &render_trace,
                    false,
                    Some(&level_visuals),
                    runtime.block_grid(),
                );
                let final_scene_samples = sample_scene_pixel_provenance(&scene, pixels);
                let eol_sprites = eol_score_sprite_diagnostics(
                    &render_trace,
                    &level_visuals,
                    runtime.block_grid(),
                );
                let vdp_samples = runtime
                    .vdp_state()
                    .map(|vdp| {
                        let snapshot = kid_md::VdpSnapshot {
                            registers: vdp.registers,
                            vram: vdp.vram.clone(),
                            cram: vdp.cram.clone(),
                            vsram: vdp.vsram.clone(),
                        };
                        pixels
                            .iter()
                            .map(|(x, y)| NativeVdpPixelDiagnostic::from_sample(&snapshot, *x, *y))
                            .collect::<Vec<_>>()
                    })
                    .unwrap_or_default();
                background_samples_by_checkpoint.insert(key.clone(), background_samples);
                layer_samples_by_checkpoint.insert(key.clone(), layer_samples);
                final_scene_samples_by_checkpoint.insert(key.clone(), final_scene_samples);
                vdp_samples_by_checkpoint.insert(key.clone(), vdp_samples);
                eol_sprites_by_checkpoint.insert(key, eol_sprites);
            }
        }

        level_card_frames_remaining = level_card_frames_remaining.saturating_sub(1);
        replay_frame_index += 1;
    }

    for diff in frame_diffs {
        let key = (diff.frame_index, diff.label.clone());
        diff.native_background_samples = background_samples_by_checkpoint
            .remove(&key)
            .unwrap_or_default();
        diff.native_layer_samples = layer_samples_by_checkpoint.remove(&key).unwrap_or_default();
        diff.native_final_scene_samples = final_scene_samples_by_checkpoint
            .remove(&key)
            .unwrap_or_default();
        diff.native_vdp_samples = vdp_samples_by_checkpoint.remove(&key).unwrap_or_default();
        diff.native_eol_score_sprites = eol_sprites_by_checkpoint.remove(&key).unwrap_or_default();
    }

    Ok(())
}

impl NativeVdpPixelDiagnostic {
    fn from_sample(vdp: &kid_md::VdpSnapshot, x: usize, y: usize) -> Self {
        let sample = kid_md::sample_vdp_pixel(vdp, x, y);
        Self {
            x: sample.x,
            y: sample.y,
            layers: sample
                .layers
                .into_iter()
                .map(NativeVdpLayerPixelSample::from)
                .collect(),
        }
    }
}

impl From<kid_md::VdpPixelLayerSample> for NativeVdpLayerPixelSample {
    fn from(sample: kid_md::VdpPixelLayerSample) -> Self {
        Self {
            layer: sample.layer.to_string(),
            priority: sample.priority,
            map_base: sample.map_base,
            cell_x: sample.cell_x,
            cell_y: sample.cell_y,
            cell_index: sample.cell_index,
            entry: sample.entry,
            tile_index: sample.tile_index,
            palette_line: sample.palette_line,
            colour_index: sample.colour_index,
            source_x: sample.source_x,
            source_y: sample.source_y,
            rgb: sample.rgb,
        }
    }
}

fn sample_native_layers_for_pixels(
    trace: &NativeTraceFrame,
    level_visuals: &NativeLevelVisuals,
    block_grid: Option<&kid_core::terrain::BlockGrid>,
    pixels: &BTreeSet<(usize, usize)>,
) -> Vec<NativeLayerPixelDiagnostic> {
    let mut samples = pixels
        .iter()
        .map(|(x, y)| NativeLayerPixelDiagnostic {
            x: *x,
            y: *y,
            samples: Vec::with_capacity(NATIVE_DIAGNOSTIC_LAYERS.len()),
        })
        .collect::<Vec<_>>();

    for layer in NATIVE_DIAGNOSTIC_LAYERS {
        let scene = build_scene_filtered(
            trace,
            false,
            Some(level_visuals),
            block_grid,
            LayerFilter::solo(layer),
        );
        let mut rgb = vec![0; (NATIVE_FRAME_WIDTH * NATIVE_FRAME_HEIGHT * 3) as usize];
        // Use overlay-only rendering so transparent pixels stay black instead
        // of inheriting the scene background. This is diagnostic provenance for
        // ASM loc_C070/loc_C21E strict diffs, not a standalone frame.
        render_scene_overlay(&scene, &mut rgb);
        for diagnostic in &mut samples {
            diagnostic.samples.push(NativeLayerPixelSample {
                layer: layer.to_owned(),
                rgb: sample_rgb(&rgb, diagnostic.x, diagnostic.y),
            });
        }
    }

    samples
}

fn sample_rgb(rgb: &[u8], x: usize, y: usize) -> [u8; 3] {
    let index = (y * NATIVE_FRAME_WIDTH as usize + x) * 3;
    [rgb[index], rgb[index + 1], rgb[index + 2]]
}

fn checkpoint_state_deltas(
    native_states: &[OracleState],
    oracle_states: &[OracleState],
) -> Vec<CheckpointStateDelta> {
    let mut deltas = Vec::new();
    for (native, oracle) in native_states.iter().zip(oracle_states) {
        if native.label != oracle.label || native.frame_index != oracle.frame_index {
            continue;
        }

        push_u32_state_delta(
            &mut deltas,
            native,
            "camera_x_pos",
            native.camera_x_pos,
            oracle.camera_x_pos,
        );
        push_u32_state_delta(
            &mut deltas,
            native,
            "camera_y_pos",
            native.camera_y_pos,
            oracle.camera_y_pos,
        );
        push_u32_state_delta(
            &mut deltas,
            native,
            "kid_x_pos",
            native.kid_x_pos,
            oracle.kid_x_pos,
        );
        push_u32_state_delta(
            &mut deltas,
            native,
            "kid_y_pos",
            native.kid_y_pos,
            oracle.kid_y_pos,
        );
        if let (Some(native_value), Some(oracle_value)) = (native.kid_x_vel, oracle.kid_x_vel) {
            push_i32_state_delta(&mut deltas, native, "kid_x_vel", native_value, oracle_value);
        }
        if let (Some(native_value), Some(oracle_value)) = (native.kid_y_vel, oracle.kid_y_vel) {
            push_i32_state_delta(&mut deltas, native, "kid_y_vel", native_value, oracle_value);
        }
    }
    deltas
}

fn push_u32_state_delta(
    deltas: &mut Vec<CheckpointStateDelta>,
    native: &OracleState,
    field: &str,
    native_value: u32,
    oracle_value: u32,
) {
    if native_value == oracle_value {
        return;
    }
    deltas.push(CheckpointStateDelta {
        label: native.label.clone(),
        frame_index: native.frame_index,
        field: field.to_owned(),
        native_value: format!("{native_value:08X}"),
        oracle_value: format!("{oracle_value:08X}"),
        delta: format!("{:#X}", native_value.abs_diff(oracle_value)),
    });
}

fn push_i32_state_delta(
    deltas: &mut Vec<CheckpointStateDelta>,
    native: &OracleState,
    field: &str,
    native_value: i32,
    oracle_value: i32,
) {
    if native_value == oracle_value {
        return;
    }
    deltas.push(CheckpointStateDelta {
        label: native.label.clone(),
        frame_index: native.frame_index,
        field: field.to_owned(),
        native_value: format!("{native_value:08X}"),
        oracle_value: format!("{oracle_value:08X}"),
        delta: format!("{:#X}", native_value.abs_diff(oracle_value)),
    });
}

impl From<BackgroundPixelSample> for NativeBackgroundPixelSample {
    fn from(sample: BackgroundPixelSample) -> Self {
        Self {
            screen_x: sample.screen_x,
            screen_y: sample.screen_y,
            tile_origin_x: sample.tile_origin_x,
            tile_origin_y: sample.tile_origin_y,
            background_row_index: sample.background_row_index,
            background_row_selector: sample.background_row_selector,
            background_column: sample.background_column,
            tile_index: sample.tile_index,
            palette_line: sample.palette_line,
            palette_index: sample.palette_index,
            source_x: sample.source_x,
            source_y: sample.source_y,
            row_offset: sample.row_offset,
            rgb: sample.rgb,
        }
    }
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
        let pixel_diff_summary = summarize_pixel_diffs(native_rgb, &oracle_rgb, *width, *height)?;
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
            changed_pixel_count: pixel_diff_summary.changed_pixel_count,
            changed_bounds: pixel_diff_summary.changed_bounds,
            top_colour_diffs: pixel_diff_summary.top_colour_diffs,
            native_background_samples: Vec::new(),
            native_layer_samples: Vec::new(),
            native_final_scene_samples: Vec::new(),
            native_vdp_samples: Vec::new(),
            native_eol_score_sprites: Vec::new(),
            oracle_gameplay_debug_fields: BTreeMap::new(),
            diff_path,
        });
    }
    Ok(diffs)
}

fn summarize_pixel_diffs(
    native_rgb: &[u8],
    oracle_rgb: &[u8],
    width: usize,
    height: usize,
) -> Result<PixelDiffSummary> {
    if native_rgb.len() != oracle_rgb.len() {
        bail!(
            "frame size mismatch: native {} bytes vs oracle {} bytes",
            native_rgb.len(),
            oracle_rgb.len()
        );
    }
    let expected_len = width
        .checked_mul(height)
        .and_then(|pixels| pixels.checked_mul(3))
        .ok_or_else(|| anyhow!("frame dimensions overflow: {width}x{height}"))?;
    if native_rgb.len() != expected_len {
        bail!(
            "frame dimensions {}x{} imply {} RGB bytes, got {}",
            width,
            height,
            expected_len,
            native_rgb.len()
        );
    }

    let mut changed_pixel_count = 0usize;
    let mut bounds: Option<PixelDiffBounds> = None;
    let mut colour_pairs: BTreeMap<([u8; 3], [u8; 3]), PixelColourDiffAccumulator> =
        BTreeMap::new();
    for (pixel_index, (native_pixel, oracle_pixel)) in native_rgb
        .chunks_exact(3)
        .zip(oracle_rgb.chunks_exact(3))
        .enumerate()
    {
        if native_pixel != oracle_pixel {
            changed_pixel_count += 1;
            let native_rgb = [native_pixel[0], native_pixel[1], native_pixel[2]];
            let oracle_rgb = [oracle_pixel[0], oracle_pixel[1], oracle_pixel[2]];
            let x = pixel_index % width;
            let y = pixel_index / width;
            let colour_pair = colour_pairs.entry((native_rgb, oracle_rgb)).or_default();
            colour_pair.count += 1;
            if colour_pair.sample_pixels.len() < 5 {
                colour_pair.sample_pixels.push(PixelCoord { x, y });
            }
            bounds = Some(match bounds {
                Some(existing) => PixelDiffBounds {
                    min_x: existing.min_x.min(x),
                    min_y: existing.min_y.min(y),
                    max_x: existing.max_x.max(x),
                    max_y: existing.max_y.max(y),
                },
                None => PixelDiffBounds {
                    min_x: x,
                    min_y: y,
                    max_x: x,
                    max_y: y,
                },
            });
        }
    }
    let mut top_colour_diffs = colour_pairs
        .into_iter()
        .map(|((native_rgb, oracle_rgb), accumulator)| PixelColourDiff {
            count: accumulator.count,
            native_rgb,
            oracle_rgb,
            sample_pixels: accumulator.sample_pixels,
        })
        .collect::<Vec<_>>();
    top_colour_diffs.sort_by(|left, right| {
        right
            .count
            .cmp(&left.count)
            .then_with(|| left.native_rgb.cmp(&right.native_rgb))
            .then_with(|| left.oracle_rgb.cmp(&right.oracle_rgb))
    });
    top_colour_diffs.truncate(8);

    Ok(PixelDiffSummary {
        changed_pixel_count,
        changed_bounds: bounds,
        top_colour_diffs,
    })
}

fn first_trace_drift(
    recorded: &[NativeTraceFrame],
    replay: &[NativeTraceFrame],
    legacy_block_destroy_direction_frames: &BTreeSet<usize>,
    legacy_missing_played_sound_effect_ids_frames: &BTreeSet<usize>,
    normalise_settled_level_skip_cheat: bool,
) -> Option<TraceDriftReport> {
    first_trace_drift_inner(
        recorded,
        replay,
        legacy_block_destroy_direction_frames,
        legacy_missing_played_sound_effect_ids_frames,
        normalise_settled_level_skip_cheat,
        TraceLengthMode::Exact,
    )
}

pub(crate) fn first_trace_prefix_drift(
    recording: &NativeRecording,
    replay: &[NativeTraceFrame],
) -> Option<TraceDriftReport> {
    first_trace_drift_inner(
        &recording.trace,
        replay,
        &recording.legacy_block_destroy_direction_frames,
        &recording.legacy_missing_played_sound_effect_ids_frames,
        recording.manifest.start.settled && recording.manifest.start.warp_level.is_some(),
        TraceLengthMode::ReplayPrefix,
    )
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TraceLengthMode {
    Exact,
    ReplayPrefix,
}

fn first_trace_drift_inner(
    recorded: &[NativeTraceFrame],
    replay: &[NativeTraceFrame],
    legacy_block_destroy_direction_frames: &BTreeSet<usize>,
    legacy_missing_played_sound_effect_ids_frames: &BTreeSet<usize>,
    normalise_settled_level_skip_cheat: bool,
    length_mode: TraceLengthMode,
) -> Option<TraceDriftReport> {
    for (index, (expected, actual)) in recorded.iter().zip(replay).enumerate() {
        let expected_for_match = if legacy_missing_played_sound_effect_ids_frames.contains(&index) {
            let mut normalised_expected = expected.clone();
            normalised_expected
                .played_sound_effect_ids
                .clone_from(&actual.played_sound_effect_ids);
            normalised_expected
        } else {
            expected.clone()
        };
        let matches = if legacy_block_destroy_direction_frames.contains(&index) {
            trace_frames_match_with_legacy_block_destroy_direction(
                &expected_for_match,
                actual,
                normalise_settled_level_skip_cheat,
            )
        } else {
            trace_frames_match(
                &expected_for_match,
                actual,
                normalise_settled_level_skip_cheat,
            )
        };
        if !matches {
            return Some(TraceDriftReport {
                frame_index: index,
                reason: "native trace frame mismatch".to_owned(),
            });
        }
    }
    let length_drifted = match length_mode {
        TraceLengthMode::Exact => recorded.len() != replay.len(),
        TraceLengthMode::ReplayPrefix => replay.len() > recorded.len(),
    };
    if length_drifted {
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

fn trace_frames_match(
    expected: &NativeTraceFrame,
    actual: &NativeTraceFrame,
    normalise_settled_level_skip_cheat: bool,
) -> bool {
    if expected == actual {
        return true;
    }

    let mut normalised_expected = expected.clone();
    if normalise_settled_level_skip_cheat {
        // Settled warp recordings replay through the reduced oracle level-skip
        // path. ASM loc_D468 can still observe LevelSkip_Cheat for EOL digit
        // sprites, so `run_recorded_trace` preserves that compatibility bit;
        // older/live-recorded play-native traces may omit it. The same reduced
        // path seeds `Level_completion_time` to one second for loc_D47E, while
        // older visual-validation bundles recorded the default zero.
        normalised_expected.level_skip_cheat_enabled = actual.level_skip_cheat_enabled;
        normalised_expected.level_completion_time = actual.level_completion_time;
    }

    if expected.game_mode == actual.game_mode
        && !title_screen_ready_affects_recording_match(expected.game_mode)
    {
        normalised_expected.title_screen_ready = actual.title_screen_ready;
    }

    if expected.game_mode == actual.game_mode
        && !matches!(expected.game_mode, GameMode::InGame | GameMode::DemoPlay)
    {
        normalised_expected.number_objects = actual.number_objects;
        normalised_expected.number_gfx_objects = actual.number_gfx_objects;
    }

    if expected.hud_diamond_frame_offset.is_none() {
        normalised_expected.hud_diamond_frame_offset = actual.hud_diamond_frame_offset;
    }

    if expected.active_enemy_details.is_empty() {
        // `active_enemy_details` is additive trace telemetry for semantic case
        // comparison. Legacy recordings predate it, so absence must not make a
        // replay drift when the authoritative frame state still matches.
        normalised_expected
            .active_enemy_details
            .clone_from(&actual.active_enemy_details);
    }

    if expected.block_bounce_details.is_empty() {
        // `block_bounce_details` is additive render telemetry for ASM block
        // workers. Legacy recordings only carried the compact bounce tuples, so
        // replay should not drift solely because current traces name the worker
        // kind.
        normalised_expected
            .block_bounce_details
            .clone_from(&actual.block_bounce_details);
    }

    if expected.played_bgm_ids.is_empty() {
        // Level-start BGM ids are additive audio telemetry. Legacy visual
        // recordings predate the core loc_11FD2 queue and still compare their
        // audible output separately, so missing ids alone are not gameplay drift.
        normalised_expected
            .played_bgm_ids
            .clone_from(&actual.played_bgm_ids);
    }

    if !expected.island_shore_cycle_active
        && expected.island_shore_cycle_state_loc_1202a.is_none()
        && expected.island_shore_vram_frames_loc_1202a.is_none()
        && expected.island_shore_dma_uploads_loc_1202a.is_empty()
    {
        // Island-shore cycle fields are additive diagnostics for the VDP worker
        // family rooted at ASM loc_1202A/loc_1223C. Older recordings deserialize
        // them as the default inactive state; pixel checkpoints still guard the
        // visible effect.
        normalised_expected.island_shore_cycle_active = actual.island_shore_cycle_active;
        normalised_expected.island_shore_cycle_state_loc_1202a =
            actual.island_shore_cycle_state_loc_1202a.clone();
        normalised_expected.island_shore_vram_frames_loc_1202a =
            actual.island_shore_vram_frames_loc_1202a.clone();
        normalised_expected
            .island_shore_dma_uploads_loc_1202a
            .clone_from(&actual.island_shore_dma_uploads_loc_1202a);
    }

    if expected.rotate_palette_lava_water_state_loc_1223c.is_none()
        && expected.lava_water_palette_words_loc_1223c.is_none()
    {
        // These loc_1223C fields expose additive palette-worker diagnostics.
        // Pixel checkpoints and oracle state still guard the visible effect.
        normalised_expected.rotate_palette_lava_water_state_loc_1223c =
            actual.rotate_palette_lava_water_state_loc_1223c;
        normalised_expected.lava_water_palette_words_loc_1223c =
            actual.lava_water_palette_words_loc_1223c;
    }

    &normalised_expected == actual
}

fn title_screen_ready_affects_recording_match(game_mode: GameMode) -> bool {
    matches!(game_mode, GameMode::TitleScreen | GameMode::OptionsMenu)
}

fn trace_frames_match_with_legacy_block_destroy_direction(
    expected: &NativeTraceFrame,
    actual: &NativeTraceFrame,
    normalise_settled_level_skip_cheat: bool,
) -> bool {
    if trace_frames_match(expected, actual, normalise_settled_level_skip_cheat) {
        return true;
    }
    if expected.block_destroy_effects.len() != actual.block_destroy_effects.len() {
        return false;
    }
    if expected
        .block_destroy_effects
        .iter()
        .zip(&actual.block_destroy_effects)
        .any(
            |(
                &(expected_x, expected_y, expected_remaining, _),
                &(actual_x, actual_y, actual_remaining, _),
            )| {
                (expected_x, expected_y, expected_remaining)
                    != (actual_x, actual_y, actual_remaining)
            },
        )
    {
        return false;
    }

    let mut normalised_expected = expected.clone();
    normalised_expected
        .block_destroy_effects
        .clone_from(&actual.block_destroy_effects);
    trace_frames_match(
        &normalised_expected,
        actual,
        normalise_settled_level_skip_cheat,
    )
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
    rebase_settled_trace_frame_indices(recording);
    Ok(())
}

fn rebase_settled_trace_frame_indices(recording: &mut NativeRecording) {
    if !recording.manifest.start.settled {
        return;
    }
    rebase_trace_frame_indices(&mut recording.trace);
}

fn rebase_trace_frame_indices(trace: &mut [NativeTraceFrame]) {
    let Some(first_frame_index) = trace.first().map(|frame| frame.frame_index) else {
        return;
    };
    if first_frame_index == 0 {
        return;
    }

    // Settled recordings keep script/checkpoint indices relative to the
    // recorded window, but some bundles preserve absolute runtime frame indices
    // from playtest entry. Rebase them so verification and case derivation can
    // address the trace by recording-local frame numbers.
    for frame in trace {
        frame.frame_index = frame.frame_index.saturating_sub(first_frame_index);
    }
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
        write_wav_with_channels(
            &audio_path,
            artifacts.audio_sample_rate,
            artifacts.audio_channels,
            audio_samples,
        )
        .with_context(|| format!("failed to write {}", audio_path.display()))?;
        manifest_entries.push(RecordingArtifactManifest {
            path: String::from(RECORDING_AUDIO_FILENAME),
            kind: String::from("audio"),
        });
    }

    if !artifacts.audio_events.is_empty() {
        let audio_events_path = record_to.join(RECORDING_AUDIO_EVENTS_FILENAME);
        fs::write(&audio_events_path, artifacts.audio_events.join("\n") + "\n")
            .with_context(|| format!("failed to write {}", audio_events_path.display()))?;
        manifest_entries.push(RecordingArtifactManifest {
            path: String::from(RECORDING_AUDIO_EVENTS_FILENAME),
            kind: String::from("audio-events"),
        });
    }

    if !artifacts.audio_diagnostics.is_empty() {
        let audio_diagnostics_path = record_to.join(RECORDING_AUDIO_DIAGNOSTICS_FILENAME);
        fs::write(
            &audio_diagnostics_path,
            artifacts.audio_diagnostics.join("\n") + "\n",
        )
        .with_context(|| format!("failed to write {}", audio_diagnostics_path.display()))?;
        manifest_entries.push(RecordingArtifactManifest {
            path: String::from(RECORDING_AUDIO_DIAGNOSTICS_FILENAME),
            kind: String::from("audio-diagnostics"),
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
    use std::fmt::Write;

    let mut command = String::from("cargo run --release -p kidcli -- play-native");
    if let Some(level) = start.warp_level {
        let _ = write!(command, " --warp-level {level:02X}");
    }
    if start.settled {
        command.push_str(" --settled");
    }
    let _ = write!(command, " --record {slug}");
    if let Some(frame_limit) = start.record_frames {
        let _ = write!(command, " --record-frames {frame_limit}");
    }
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

    fn cached_generated_manifest() -> Option<AssetManifest> {
        let repo_root = if let Ok(override_root) = std::env::var("KID_REPO_ROOT")
            && !override_root.is_empty()
        {
            std::path::PathBuf::from(override_root)
        } else {
            let crate_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
            crate_dir
                .parent()
                .and_then(std::path::Path::parent)
                .expect("kidcli crate should live under <repo>/crates/kidcli")
                .to_path_buf()
        };
        let manifest_path = repo_root
            .join(".cache/generated/28b904000b2863b6760531807760b571f1a5fc1d/asset-manifest.json");
        if !manifest_path.is_file() {
            eprintln!(
                "skipping cache-backed recording test: generated asset manifest missing at {}",
                manifest_path.display()
            );
            return None;
        }
        Some(kid_data::load_manifest(&manifest_path).expect("load cached generated asset manifest"))
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
            extra_hitpoint_slots: 0,
            current_helmet_id: 0,
            red_stealth_sword_timer: 0,
            red_stealth_airborne_slash_counter: 0,
            paused: false,
            pause_menu_option: 0,
            kid_x_pos: Some(0x20_0000),
            kid_y_pos: Some(0x50_0000),
            kid_x_velocity: Some(0),
            kid_y_velocity: Some(0),
            character_movement: None,
            kid_facing: Some(NativeDirection::Right),
            kid_on_ground: Some(true),
            walk_visual_accumulator: 0,
            iron_knight_climbing: false,
            iron_knight_climb_pose_accum: 0,
            hud_diamond_frame_offset: None,
            berzerker_charging: false,
            berzerker_charge_pose: false,
            berzerker_walk_pose: false,
            ledge_snap_cooldown: 0,
            kid_on_slope: 0,
            active_objects: Vec::new(),
            active_enemy_details: Vec::new(),
            camera_x_pos: 0,
            camera_y_pos: 0,
            number_objects: 0,
            number_gfx_objects: 0,
            rng_offset: None,
            rng_last_byte: None,
            level_complete: false,
            level_complete_elapsed_frames: 0,
            level_complete_score_rows_elapsed_frames: 0,
            level_complete_score_counter_elapsed_frames: 0,
            level_complete_score_counter_drained: false,
            level_complete_initial_score: 0,
            level_complete_score_rows_started: 0,
            level_completion_time: 0,
            level_skip_cheat_enabled: 0,
            game_over: false,
            game_over_elapsed_frames: 0,
            continue_countdown: None,
            continue_countdown_elapsed_frames: 0,
            game_complete: false,
            ending_elapsed: None,
            options_cursor: None,
            played_sound_effect_ids: Vec::new(),
            played_sound_commands: Vec::new(),
            played_bgm_ids: Vec::new(),
            played_music_speed_updates: Vec::new(),
            teleport_warp_colour: 0,
            sound_test_track: None,
            two_player: false,
            options_two_player_two_controllers: false,
            options_controls_scheme: 0,
            options_speed_button_fast: false,
            invulnerable_frames: 0,
            kid_is_invulnerable: false,
            just_received_damage: false,
            diamond_power_active: false,
            diamond_count: 0,
            score: 0,
            level_timer_seconds: 0,
            level_damage_taken: false,
            level_prize_collected: false,
            level_speed_bonus_awarded: false,
            level_path_bonus_amount: 0,
            murder_wall_x: None,
            active_projectiles: Vec::new(),
            active_projectile_details: Vec::new(),
            diamond_power_sprites: Vec::new(),
            active_geysers: Vec::new(),
            island_shore_cycle_active: false,
            island_shore_cycle_state_loc_1202a: None,
            island_shore_vram_frames_loc_1202a: None,
            island_shore_dma_uploads_loc_1202a: Vec::new(),
            rotate_palette_lava_water_state_loc_1223c: None,
            lava_water_palette_words_loc_1223c: None,
            block_bounces: Vec::new(),
            block_bounce_details: Vec::new(),
            block_destroy_effects: Vec::new(),
            block_debris_pieces: Vec::new(),
            prize_block_spawn_flashes: Vec::new(),
            kid_projectile_hit_sparks: Vec::new(),
            floating_items: Vec::new(),
            enemy_death_effects: Vec::new(),
            heady_metal_death_splits: Vec::new(),
            heady_metal_body_pieces_sub_37abc: Vec::new(),
            death_windup_elapsed: 0,
            form_change_flash_frames: 0,
            stomped_this_frame: false,
            blocks_hit_this_frame: 0,
            sound_test_play_pending: false,
            gameplay_debug_fields: std::collections::BTreeMap::new(),
            physics_trace: None,
        }
    }

    fn sample_oracle_state(frame_index: usize, label: &str) -> OracleState {
        OracleState {
            label: label.to_owned(),
            frame_index,
            game_mode: GameMode::InGame.as_disassembly_value(),
            boot_phase: None,
            boot_phase_elapsed: None,
            title_screen_ready: false,
            main_menu_option: None,
            current_player: 0,
            number_hitpoints: 3,
            current_helmet_id: 0,
            current_level_id: 0,
            player_1_level_id: 0,
            camera_x_pos: 0x0010_0000,
            camera_y_pos: 0x0020_0000,
            kid_x_pos: 0x0030_0000,
            kid_y_pos: 0x0040_0000,
            number_objects: 0,
            number_gfx_objects: 0,
            game_paused: false,
            active_enemies: Vec::new(),
            score: 0,
            number_lives: 3,
            number_diamonds: 0,
            diamond_power_active: false,
            murder_wall_x: None,
            kid_x_vel: Some(0x0001_0000),
            kid_y_vel: Some(0),
            character_movement: None,
            just_received_damage: Some(false),
            kid_collision_type: Some(0),
            kid_collision_source: Some(0),
            kid_x_direction: Some(0),
            invulnerable_frames: Some(0),
            currently_transforming: Some(false),
            rng_offset: None,
            rng_last_byte: None,
            level_special_effects: Some(0),
            gameplay_debug_fields: BTreeMap::new(),
            cpu_registers: None,
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
        let drift = first_trace_drift(
            &recorded,
            &replay,
            &BTreeSet::new(),
            &BTreeSet::new(),
            false,
        )
        .expect("drift");
        assert_eq!(drift.frame_index, 1);
    }

    #[test]
    fn first_trace_drift_ignores_non_shell_title_screen_ready_only() {
        let recorded = vec![sample_trace_frame(0), {
            let mut frame = sample_trace_frame(1);
            frame.game_mode = GameMode::LevelTransition;
            frame.game_mode_value = GameMode::LevelTransition.as_disassembly_value();
            frame.title_screen_ready = true;
            frame
        }];
        let replay = vec![sample_trace_frame(0), {
            let mut frame = sample_trace_frame(1);
            frame.game_mode = GameMode::LevelTransition;
            frame.game_mode_value = GameMode::LevelTransition.as_disassembly_value();
            frame
        }];

        let drift = first_trace_drift(
            &recorded,
            &replay,
            &BTreeSet::new(),
            &BTreeSet::new(),
            false,
        );

        assert_eq!(
            drift, None,
            "settled gameplay recordings should not fail native replay because a shell-only title readiness diagnostic changed while both frames are outside the title/options shell"
        );
    }

    #[test]
    fn first_trace_drift_keeps_title_screen_ready_strict_in_title_shell() {
        let mut recorded = sample_trace_frame(0);
        recorded.game_mode = GameMode::TitleScreen;
        recorded.game_mode_value = GameMode::TitleScreen.as_disassembly_value();
        recorded.title_screen_ready = true;
        let mut replay = recorded.clone();
        replay.title_screen_ready = false;

        let drift = first_trace_drift(
            &[recorded],
            &[replay],
            &BTreeSet::new(),
            &BTreeSet::new(),
            false,
        )
        .expect("drift");

        assert_eq!(drift.frame_index, 0);
    }

    #[test]
    fn first_trace_drift_ignores_non_gameplay_shell_object_counters() {
        let mut recorded = sample_trace_frame(0);
        recorded.game_mode = GameMode::IntroSequence6;
        recorded.game_mode_value = GameMode::IntroSequence6.as_disassembly_value();
        recorded.number_objects = 0;
        recorded.number_gfx_objects = 0;
        let mut replay = recorded.clone();
        replay.number_objects = 4;
        replay.number_gfx_objects = 4;

        let drift = first_trace_drift(
            &[recorded],
            &[replay],
            &BTreeSet::new(),
            &BTreeSet::new(),
            false,
        );

        assert_eq!(
            drift, None,
            "intro/title shell visual workers should not make native recording replay look stale; gameplay case semantics own InGame entities"
        );
    }

    #[test]
    fn first_trace_drift_accepts_legacy_block_destroy_entries_without_direction() {
        let mut recorded = sample_trace_frame(0);
        recorded.block_destroy_effects = vec![(144, 196, 2, 2)];
        let mut replay = sample_trace_frame(0);
        replay.block_destroy_effects = vec![(144, 196, 2, 0)];

        let drift = first_trace_drift(
            &[recorded],
            &[replay],
            &BTreeSet::from([0]),
            &BTreeSet::new(),
            false,
        );

        assert_eq!(
            drift, None,
            "legacy recordings stored ASM sub_292E debris as (x,y,remaining), so replay verification must not fail only because current traces add d6 direction"
        );
    }

    #[test]
    fn first_trace_drift_accepts_legacy_frames_missing_played_sound_effect_ids() {
        let recorded = sample_trace_frame(0);
        let mut replay = recorded.clone();
        replay.played_sound_effect_ids = vec![0x61];

        let strict_drift = first_trace_drift(
            std::slice::from_ref(&recorded),
            std::slice::from_ref(&replay),
            &BTreeSet::new(),
            &BTreeSet::new(),
            false,
        )
        .expect("strict comparison should drift");
        assert_eq!(strict_drift.frame_index, 0);

        let legacy_drift = first_trace_drift(
            &[recorded],
            &[replay],
            &BTreeSet::new(),
            &BTreeSet::from([0]),
            false,
        );

        assert_eq!(
            legacy_drift, None,
            "older visual-validation recordings predate played_sound_effect_ids, so replay verification should only normalise frames whose raw trace omitted that field"
        );
    }

    #[test]
    fn first_trace_drift_accepts_legacy_frames_missing_hud_diamond_frame_offset() {
        let recorded = sample_trace_frame(0);
        let mut replay = recorded.clone();
        replay.hud_diamond_frame_offset = Some(4);

        let drift = first_trace_drift(
            &[recorded],
            &[replay],
            &BTreeSet::new(),
            &BTreeSet::new(),
            false,
        );

        assert_eq!(
            drift, None,
            "older recording bundles predate the HUD diamond animation diagnostic, so replay verification should not treat the added trace field as gameplay drift"
        );
    }

    #[test]
    fn first_trace_drift_accepts_legacy_frames_missing_active_enemy_details() {
        let recorded = sample_trace_frame(0);
        let mut replay = recorded.clone();
        replay.active_enemy_details = vec![NativeObjectSnapshot {
            kind: NativeObjectKind::Enemy { object_type: 0x16 },
            x_pos: 0x01c8_0000,
            y_pos: 0x021f_0000,
            layout_index: Some(7),
            x_velocity: Some(0),
            y_velocity: Some(0),
            facing: Some(NativeDirection::Right),
            on_ground: Some(false),
            hit_points: Some(1),
            damage_flash: false,
            palette_variant: 0,
            stationary: false,
            shared_walker_state: None,
            shared_walker_patrol_timer: None,
            shared_walker_patrol_short_next: None,
            walk_phase: None,
            wait_timer: None,
            debug_fields: std::collections::BTreeMap::new(),
        }];

        let drift = first_trace_drift(
            &[recorded],
            &[replay],
            &BTreeSet::new(),
            &BTreeSet::new(),
            false,
        );

        assert_eq!(
            drift, None,
            "active_enemy_details is additive telemetry for hidden controller semantics; legacy recording replay must not drift when current traces populate it"
        );
    }

    #[test]
    fn first_trace_drift_accepts_legacy_frames_missing_block_bounce_details() {
        let recorded = sample_trace_frame(0);
        let mut replay = recorded.clone();
        replay.block_bounce_details = vec![BlockBounceTrace {
            cell_x: 4,
            cell_y: 36,
            remaining: 31,
            y_offset_px: 0,
            kind: BlockBounceTraceKind::Prize,
        }];

        let drift = first_trace_drift(
            &[recorded],
            &[replay],
            &BTreeSet::new(),
            &BTreeSet::new(),
            false,
        );

        assert_eq!(
            drift, None,
            "block_bounce_details is additive render telemetry for ASM block workers; legacy recording replay must not drift when compact block_bounces still match"
        );
    }

    #[test]
    fn first_trace_drift_accepts_legacy_frames_missing_island_shore_diagnostics() {
        let recorded = sample_trace_frame(0);
        let mut replay = recorded.clone();
        replay.island_shore_cycle_active = true;
        replay.island_shore_cycle_state_loc_1202a = Some((1, 2, 3, 4, 5, 6));
        replay.island_shore_vram_frames_loc_1202a = Some((7, 8, 9));
        replay.island_shore_dma_uploads_loc_1202a = vec![IslandShoreDmaUploadLoc1202A {
            frame_index: 3,
            source_offset: 0x120,
            vram_word_address: 0x6400,
            dma_length_bytes: 0x80,
        }];

        let drift = first_trace_drift(
            &[recorded],
            &[replay],
            &BTreeSet::new(),
            &BTreeSet::new(),
            false,
        );

        assert_eq!(
            drift, None,
            "loc_1202A island-shore fields are additive diagnostics; pixel checkpoints guard the visible VDP effect for legacy recordings"
        );
    }

    #[test]
    fn first_trace_drift_accepts_legacy_frames_missing_level_audio_and_lava_water_diagnostics() {
        let recorded = sample_trace_frame(0);
        let mut replay = recorded.clone();
        replay.played_bgm_ids = vec![6];
        replay.rotate_palette_lava_water_state_loc_1223c = Some((3, 8, 7));
        replay.lava_water_palette_words_loc_1223c = Some([0, 0, 0, 0]);

        let drift = first_trace_drift(
            &[recorded],
            &[replay],
            &BTreeSet::new(),
            &BTreeSet::new(),
            false,
        );

        assert_eq!(
            drift, None,
            "level-start BGM ids and loc_1223C lava/water palette fields are additive diagnostics for legacy visual-validation recordings"
        );
    }

    #[test]
    fn legacy_missing_played_sound_effect_ids_frames_detects_omitted_keys() {
        let trace = br#"[
            {"frame_index":0},
            {"frame_index":1,"played_sound_effect_ids":[]},
            {"frame_index":2}
        ]"#;

        let frames = legacy_missing_played_sound_effect_ids_frames(trace).expect("schema");

        assert_eq!(frames, BTreeSet::from([0, 2]));
    }

    #[test]
    fn first_trace_drift_accepts_settled_warp_level_skip_cheat_compatibility_bit() {
        let mut recorded = sample_trace_frame(0);
        recorded.level_skip_cheat_enabled = 0;
        recorded.level_completion_time = 0;
        let mut replay = recorded.clone();
        replay.level_skip_cheat_enabled = 1;
        replay.level_completion_time = 1;

        let strict_drift = first_trace_drift(
            &[recorded.clone()],
            &[replay.clone()],
            &BTreeSet::new(),
            &BTreeSet::new(),
            false,
        )
        .expect("strict comparison should drift");
        assert_eq!(strict_drift.frame_index, 0);

        let settled_warp_drift = first_trace_drift(
            &[recorded],
            &[replay],
            &BTreeSet::new(),
            &BTreeSet::new(),
            true,
        );
        assert_eq!(
            settled_warp_drift, None,
            "settled warp recordings replay through the reduced oracle level-skip path, so the loc_D468 compatibility bit/time seed should not invalidate otherwise-identical traces"
        );
    }

    #[test]
    fn settled_warp_oracle_visual_compatibility_keeps_live_hud_timer() {
        let mut trace = sample_trace_frame(0);
        trace.level_skip_cheat_enabled = 0;
        trace.level_completion_time = 2;
        trace.level_timer_seconds = 177;

        apply_settled_warp_oracle_visual_compatibility(&mut trace);

        assert_eq!(
            trace.level_skip_cheat_enabled, 1,
            "settled warp recordings should preserve the ROM level-skip flag sampled by ASM loc_D468"
        );
        assert_eq!(
            trace.level_completion_time, 1,
            "ASM loc_D468 sees the reduced oracle level-skip path's one-second time seed, not the native probe's recomputed timer"
        );
        assert_eq!(
            trace.level_timer_seconds, 177,
            "settled oracle level-skip frames should render the live HUD timer from the authoritative trace; stale +1 compensation made Highwater checkpoint PNGs one second ahead of oracle"
        );
    }

    #[test]
    fn build_recording_uses_script_len_for_frame_count() {
        let recording = build_recording(
            &fixture_manifest(),
            "level1-tree-bug",
            RecordingStart {
                warp_level: Some(0),
                settled: true,
                record_frames: None,
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
    fn file_recording_replay_uses_declared_frame_count_when_script_is_short() {
        let scenario = DynamicScenario {
            slug: "cold_boot_idle_demo_loop_long".into(),
            script: vec![0],
            frame_count: 9000,
            checkpoints: vec![DynamicCheckpoint {
                label: "title_ready".into(),
                frame_index: 2600,
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

        assert_eq!(
            recording_trace_frame_count(&scenario),
            9000,
            "file-backed recordings must cover their declared checkpoint window even when a compact idle script stores one zero input byte"
        );
    }

    #[test]
    fn settled_warp_recording_script_buttons_use_oracle_live_window_phase() {
        let scenario = DynamicScenario {
            slug: "highwater-shooter-audit".into(),
            script: vec![0, 1 << 3, 1 << 3],
            frame_count: 4,
            checkpoints: Vec::new(),
            start_position: None,
            start_velocity: None,
            camera_position: None,
            playtest_form: Some(7),
            initial_diamond_count: None,
            initial_lives: None,
            warp_to_level: Some(2),
            settle_oracle_warp: true,
        };
        let settled_start = RecordingStart {
            warp_level: Some(2),
            settled: true,
            record_frames: None,
        };
        let unsettled_start = RecordingStart {
            warp_level: Some(2),
            settled: false,
            record_frames: None,
        };

        let settled_buttons = (0..4)
            .map(|frame| recording_script_buttons_at_frame(&settled_start, &scenario, frame))
            .collect::<Vec<_>>();
        let unsettled_buttons = (0..4)
            .map(|frame| recording_script_buttons_at_frame(&unsettled_start, &scenario, frame))
            .collect::<Vec<_>>();

        assert_eq!(
            settled_buttons,
            vec![0, 0, 1 << 3, 1 << 3],
            "ASM Mode_Level loc_892 runs gameplay before ReadJoypad; settled recording-oracle scripts must feed the input latch one frame later"
        );
        assert_eq!(
            unsettled_buttons,
            vec![0, 1 << 3, 1 << 3, 0],
            "non-settled recordings keep the local script phase"
        );
    }

    #[test]
    fn red_stealth_walk_replay_exports_loc_95da_visual_accumulator_for_renderer() {
        let Some(manifest) = cached_generated_manifest() else {
            return;
        };
        let mut script = vec![0x00; 761];
        for input in &mut script[60..760] {
            *input = 0x08;
        }
        for start in (90..=750).step_by(60) {
            for input in &mut script[start..start + 4] {
                *input = 0x18;
            }
        }
        let scenario = DynamicScenario {
            slug: "walk_right_red_stealth_00".into(),
            script,
            frame_count: 760,
            checkpoints: Vec::new(),
            start_position: None,
            start_velocity: None,
            camera_position: None,
            playtest_form: Some(PlayerForm::RedStealth.helmet_id()),
            initial_diamond_count: None,
            initial_lives: None,
            warp_to_level: Some(0),
            settle_oracle_warp: true,
        };
        let trace = run_recorded_trace(
            &manifest,
            RecordingStart {
                warp_level: Some(0),
                settled: true,
                record_frames: None,
            },
            &scenario,
        )
        .expect("recording replay trace");
        let frame = &trace[444];

        assert_eq!(frame.current_helmet_id, PlayerForm::RedStealth.helmet_id());
        assert_eq!(frame.kid_facing, Some(NativeDirection::Right));
        assert_eq!(frame.kid_x_velocity, Some(0x0002_0000));
        assert_eq!(
            frame.walk_visual_accumulator, 0x0353,
            "ASM loc_95DA/loc_9600: Red Stealth's flat walk renderer consumes `$FFFFF8F0`; frame 444 of the #211 recording route should select phase 3 (`unk_B6546` / red_stealth_walk_4), not freeze at phase 0 and look like moonwalking"
        );
    }

    #[test]
    fn recording_probe_state_applies_initial_diamond_count_probe() {
        let scenario = DynamicScenario {
            slug: "diamond_power_wall_audit_00".into(),
            script: vec![(1 << 7) | (1 << 6), 0],
            frame_count: 2,
            checkpoints: Vec::new(),
            start_position: None,
            start_velocity: None,
            camera_position: None,
            playtest_form: Some(7),
            initial_diamond_count: Some(50),
            initial_lives: Some(1),
            warp_to_level: Some(0),
            settle_oracle_warp: true,
        };
        let manifest = fixture_manifest();
        let mut runtime = NativeRuntime::new(manifest);
        runtime.boot_to_level(LevelId(0));
        apply_recording_probe_state(&mut runtime, &scenario).expect("probe state");

        let trace = runtime.trace_frame();

        assert_eq!(
            trace.diamond_count, 50,
            "initial_diamond_count must apply to native recording as well as the oracle scenario"
        );
        assert_eq!(
            trace.number_lives, 1,
            "ASM Game_Paused_ChkStart reads Number_Lives, so recording probes must seed it like the oracle helper"
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
    fn settled_recordings_translate_to_oracle_warp_without_extra_gameplay_prefix() {
        let recording = build_recording(
            &fixture_manifest(),
            "level1-tree-bug",
            RecordingStart {
                warp_level: Some(0),
                settled: true,
                record_frames: None,
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
        assert_eq!(
            oracle_scenario.slug, "level1-tree-bug_settled_recording_oracle",
            "settled recording oracle scenarios must opt into the helper's live/unpaused target-level settle path",
        );
        assert_eq!(oracle_scenario.warp_to_level, Some(0));
        assert_eq!(oracle_scenario.frame_count, 3);
        assert_eq!(oracle_scenario.script, vec![1, 2, 3]);
        assert_eq!(oracle_scenario.checkpoints[0].label, "start");
        assert_eq!(oracle_scenario.checkpoints[0].frame_index, 0);
    }

    #[test]
    fn settled_warp_recording_rendering_uses_recorded_script_phase() {
        let start = RecordingStart {
            warp_level: Some(0x12),
            settled: true,
            record_frames: None,
        };

        assert_eq!(
            recording_visual_frame_offset(&start),
            0,
            "the oracle helper already settles to live gameplay before frame 0, so render-only animation phase must stay aligned to the recorded script"
        );
    }

    #[test]
    fn non_settled_recording_rendering_keeps_local_phase() {
        assert_eq!(
            recording_visual_frame_offset(&RecordingStart {
                warp_level: Some(0x12),
                settled: false,
                record_frames: None,
            }),
            0
        );
        assert_eq!(
            recording_visual_frame_offset(&RecordingStart {
                warp_level: None,
                settled: true,
                record_frames: None,
            }),
            0
        );
    }

    #[test]
    fn verification_report_preserves_oracle_state_error() {
        let report = RecordingVerificationReport {
            slug: "issue-206".into(),
            native_replay_only: false,
            trace_match: true,
            oracle_state_match: false,
            oracle_state_error: Some("state tail@119 diverged in object counts".into()),
            oracle_frame_match: false,
            replay_trace_len: 120,
            recorded_trace_len: 120,
            first_trace_drift: None,
            checkpoint_state_deltas: Vec::new(),
            audio_event_windows: Vec::new(),
            frame_diffs: Vec::new(),
        };

        let json = serde_json::to_value(&report).expect("report json");

        assert_eq!(
            json["oracle_state_error"], "state tail@119 diverged in object counts",
            "verify-recording reports must keep the comparator error that explains a red oracle_state_match"
        );
    }

    #[test]
    fn summarize_pixel_diffs_reports_changed_count_and_bounds() {
        let native_rgb = vec![
            0, 0, 0, 1, 1, 1, 2, 2, 2, //
            3, 3, 3, 4, 4, 4, 5, 5, 5,
        ];
        let mut oracle_rgb = native_rgb.clone();
        oracle_rgb[3..6].copy_from_slice(&[9, 9, 9]);
        oracle_rgb[12..15].copy_from_slice(&[8, 8, 8]);

        let summary = summarize_pixel_diffs(&native_rgb, &oracle_rgb, 3, 2).expect("summary");

        assert_eq!(summary.changed_pixel_count, 2);
        assert_eq!(
            summary.changed_bounds,
            Some(PixelDiffBounds {
                min_x: 1,
                min_y: 0,
                max_x: 1,
                max_y: 1,
            })
        );
        assert_eq!(
            summary.top_colour_diffs,
            vec![
                PixelColourDiff {
                    count: 1,
                    native_rgb: [1, 1, 1],
                    oracle_rgb: [9, 9, 9],
                    sample_pixels: vec![PixelCoord { x: 1, y: 0 }],
                },
                PixelColourDiff {
                    count: 1,
                    native_rgb: [4, 4, 4],
                    oracle_rgb: [8, 8, 8],
                    sample_pixels: vec![PixelCoord { x: 1, y: 1 }],
                },
            ],
            "top colour pairs make strict frame diffs actionable without external image scripts"
        );
    }

    #[test]
    fn summarize_pixel_diffs_leaves_bounds_empty_when_frames_match() {
        let rgb = vec![0, 0, 0, 1, 1, 1, 2, 2, 2];

        let summary = summarize_pixel_diffs(&rgb, &rgb, 3, 1).expect("summary");

        assert_eq!(summary.changed_pixel_count, 0);
        assert_eq!(summary.changed_bounds, None);
        assert_eq!(summary.top_colour_diffs, Vec::new());
    }

    #[test]
    fn summarize_pixel_diffs_reports_top_colour_pairs_by_frequency() {
        let native_rgb = vec![
            1, 1, 1, 1, 1, 1, 4, 4, 4, //
            1, 1, 1, 2, 2, 2, 3, 3, 3,
        ];
        let oracle_rgb = vec![
            9, 9, 9, 9, 9, 9, 8, 8, 8, //
            9, 9, 9, 7, 7, 7, 6, 6, 6,
        ];

        let summary = summarize_pixel_diffs(&native_rgb, &oracle_rgb, 3, 2).expect("summary");

        assert_eq!(
            summary.top_colour_diffs[..3],
            [
                PixelColourDiff {
                    count: 3,
                    native_rgb: [1, 1, 1],
                    oracle_rgb: [9, 9, 9],
                    sample_pixels: vec![
                        PixelCoord { x: 0, y: 0 },
                        PixelCoord { x: 1, y: 0 },
                        PixelCoord { x: 0, y: 1 },
                    ],
                },
                PixelColourDiff {
                    count: 1,
                    native_rgb: [2, 2, 2],
                    oracle_rgb: [7, 7, 7],
                    sample_pixels: vec![PixelCoord { x: 1, y: 1 }],
                },
                PixelColourDiff {
                    count: 1,
                    native_rgb: [3, 3, 3],
                    oracle_rgb: [6, 6, 6],
                    sample_pixels: vec![PixelCoord { x: 2, y: 1 }],
                },
            ],
            "highest-frequency pixel substitutions should lead the diagnostic"
        );
    }

    #[test]
    fn summarize_pixel_diffs_limits_colour_pair_sample_pixels() {
        let native_rgb = vec![1; 18];
        let oracle_rgb = vec![2; 18];

        let summary = summarize_pixel_diffs(&native_rgb, &oracle_rgb, 3, 2).expect("summary");

        assert_eq!(summary.top_colour_diffs[0].count, 6);
        assert_eq!(
            summary.top_colour_diffs[0].sample_pixels,
            vec![
                PixelCoord { x: 0, y: 0 },
                PixelCoord { x: 1, y: 0 },
                PixelCoord { x: 2, y: 0 },
                PixelCoord { x: 0, y: 1 },
                PixelCoord { x: 1, y: 1 },
            ],
            "sample pixels should identify where to inspect tile/palette provenance without bloating reports"
        );
    }

    #[test]
    fn checkpoint_state_deltas_report_pixel_sensitive_tolerated_motion_fields() {
        let native = sample_oracle_state(90, "projectile_barrier_window");
        let mut oracle = native.clone();
        oracle.camera_x_pos = 0x000F_0000;
        oracle.kid_x_pos = 0x002E_A000;
        oracle.kid_x_vel = Some(0x0001_5000);

        let deltas = checkpoint_state_deltas(&[native], &[oracle]);

        assert_eq!(
            deltas
                .iter()
                .map(|delta| delta.field.as_str())
                .collect::<Vec<_>>(),
            vec!["camera_x_pos", "kid_x_pos", "kid_x_vel"],
            "pixel-strict recording reports should expose camera/player drift even when compare_native_states_against_oracle tolerates sub-six-pixel differences"
        );
        assert_eq!(deltas[0].delta, "0x10000");
        assert_eq!(deltas[1].delta, "0x16000");
        assert_eq!(deltas[2].delta, "0x5000");
    }

    #[test]
    fn settled_recording_oracle_scenario_preserves_probe_start_position() {
        let mut recording = build_recording(
            &fixture_manifest(),
            "issue-175-eol-flag-touch",
            RecordingStart {
                warp_level: Some(0),
                settled: true,
                record_frames: None,
            },
            vec![0],
            vec![sample_trace_frame(0)],
        )
        .expect("recording");
        recording.manifest.scenario.start_position =
            Some(crate::scenarios::DynamicStartPosition { x: 3152, y: 176 });
        recording.manifest.scenario.camera_position =
            Some(crate::scenarios::DynamicStartPosition { x: 3008, y: 64 });
        recording.manifest.scenario.playtest_form = Some(7);

        let (oracle_scenario, prefix) = oracle_scenario_for_recording(&recording);

        assert_eq!(prefix, 0);
        assert_eq!(
            oracle_scenario.start_position,
            Some(crate::scenarios::DynamicStartPosition { x: 3152, y: 176 }),
            "the oracle helper must patch the same live Kid position after it settles to gameplay"
        );
        assert_eq!(
            oracle_scenario.camera_position,
            Some(crate::scenarios::DynamicStartPosition { x: 3008, y: 64 }),
            "the oracle helper must patch the same Camera_X/Y probe window after it settles to gameplay"
        );
        assert_eq!(
            oracle_scenario.playtest_form,
            Some(7),
            "the oracle helper must patch the same Current_Helmet_ID/HP probe form after it settles to gameplay"
        );
    }

    #[test]
    fn unsettled_recordings_keep_original_oracle_scenario() {
        let recording = build_recording(
            &fixture_manifest(),
            "title-start",
            RecordingStart {
                warp_level: None,
                settled: false,
                record_frames: None,
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
                record_frames: None,
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
                record_frames: None,
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
                audio_sample_rate: 44_100,
                audio_channels: 2,
                audio_events: vec![String::from(
                    "frame=0 mode=IntroSequence1 active_after=TitleShell command=StopMusicThenStart(3)",
                )],
                audio_diagnostics: vec![String::from("queue_present=true queued_frames=1")],
            },
        )
        .expect("save recording with artifacts");

        assert!(temp.path().join(RECORDING_TRANSCRIPT_FILENAME).is_file());
        assert!(temp.path().join(RECORDING_NOTES_FILENAME).is_file());
        assert!(temp.path().join(RECORDING_FINAL_FRAME_FILENAME).is_file());
        assert!(temp.path().join(RECORDING_AUDIO_FILENAME).is_file());
        assert!(temp.path().join(RECORDING_AUDIO_EVENTS_FILENAME).is_file());
        assert!(
            temp.path()
                .join(RECORDING_AUDIO_DIAGNOSTICS_FILENAME)
                .is_file()
        );

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
                RecordingArtifactManifest {
                    path: String::from(RECORDING_AUDIO_EVENTS_FILENAME),
                    kind: String::from("audio-events"),
                },
                RecordingArtifactManifest {
                    path: String::from(RECORDING_AUDIO_DIAGNOSTICS_FILENAME),
                    kind: String::from("audio-diagnostics"),
                },
            ]
        );

        let readme = fs::read_to_string(temp.path().join("README.txt")).expect("recording readme");
        assert!(readme.contains("- transcript.txt (trace-transcript)"));
        assert!(readme.contains("- notes.txt (user-notes)"));
        assert!(readme.contains("- final-frame.png (final-frame)"));
        assert!(readme.contains("- audio.wav (audio)"));
        assert!(readme.contains("- audio-events.txt (audio-events)"));
        assert!(readme.contains("- audio-diagnostics.txt (audio-diagnostics)"));
        assert!(
            fs::read_to_string(temp.path().join(RECORDING_AUDIO_EVENTS_FILENAME))
                .expect("audio events")
                .contains("StopMusicThenStart(3)")
        );
        assert!(
            fs::read_to_string(temp.path().join(RECORDING_AUDIO_DIAGNOSTICS_FILENAME))
                .expect("audio diagnostics")
                .contains("queued_frames=1")
        );

        let samples = read_wav_f32(&temp.path().join(RECORDING_AUDIO_FILENAME)).expect("wav");
        assert_eq!(samples, vec![0.25, -0.25, 0.5, -0.5]);
    }

    #[test]
    fn audio_event_line_parser_extracts_frame_and_command() {
        assert_eq!(
            parse_audio_event_line("frame=31 mode=InGame level=00 command=PlaySound(13)"),
            Some((31, String::from("PlaySound(13)"))),
            "#216 audio-event reports should preserve the ASM sound command and its frame"
        );
    }

    #[test]
    fn audio_event_window_report_measures_immediate_stereo_energy() {
        let mut samples = vec![0.0; AUDIO_FRAME_SAMPLES * 2 * 40];
        let event_sample_start = 31 * AUDIO_FRAME_SAMPLES * 2;
        for sample in &mut samples[event_sample_start..event_sample_start + AUDIO_FRAME_SAMPLES * 2]
        {
            *sample = 0.25;
        }

        let report = audio_event_window_report(&samples, 31, String::from("PlaySound(13)"));

        assert_eq!(report.frame_index, 31);
        assert_eq!(report.window_start_frame, 31);
        assert_eq!(report.window_end_frame, 39);
        assert!(
            report.rms > 0.08,
            "event-window RMS should catch audible native output near the queued block-hit frame"
        );
        assert_eq!(report.peak, 0.25);
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
                record_frames: None,
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
    fn load_recording_rebases_settled_trace_frame_indices() {
        let temp = TempDir::new().expect("temp dir");
        fs::write(
            temp.path().join("manifest.json"),
            r#"{
  "slug": "settled-frame-offset",
  "recorded_at": "123",
  "start": { "warp_level": 0, "settled": true },
  "scenario": {
    "slug": "settled-frame-offset",
    "checkpoints": [{ "label": "start", "frame_index": 0 }],
    "frame_count": 2,
    "script": [0, 0],
    "warp_to_level": 0
  }
}"#,
        )
        .expect("manifest");

        let first = sample_trace_frame(261);
        let second = sample_trace_frame(262);
        fs::write(
            temp.path().join("trace.json"),
            serde_json::to_vec(&vec![first, second]).expect("trace json"),
        )
        .expect("trace");

        let recording = load_recording(temp.path()).expect("load settled recording");

        assert_eq!(recording.trace[0].frame_index, 0);
        assert_eq!(recording.trace[1].frame_index, 1);
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
