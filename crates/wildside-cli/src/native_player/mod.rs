use crate::inspect::target_enemy_contact_from_frame;
use crate::recording::{
    RecordingBundleArtifacts, RecordingRouteTarget, build_recording,
    recording_route_target_score_commands, recording_start_from_options,
    recording_tas_frontier_command, save_recording_with_artifacts,
};
use crate::scenarios::{
    DynamicCheckpoint, DynamicScenario, DynamicStartPosition, GameplayCheckpoint,
};
use anyhow::{Result, anyhow, bail};
use sdl2::audio::AudioQueue;
use sdl2::controller::{Button, GameController};
use sdl2::event::Event;
use sdl2::keyboard::{Keycode, Scancode};
use sdl2::pixels::PixelFormatEnum;
use sdl2::rect::Rect;
use sdl2::render::WindowCanvas;
use sdl2::video::FullscreenType;
use std::collections::VecDeque;
use std::path::PathBuf;
use std::thread;
use std::time::{Duration, Instant};
use wildside_core::{
    FrameInput, GameMode, LevelId, NativeRuntime, NativeSoundCommand, NativeTraceFrame, PlayerForm,
    terrain::BlockGrid,
};
use wildside_data::{
    AssetManifest, AssetSource, FsAssetSource, LevelHeader, resource_level_id,
    resource_level_record,
};
use wildside_md::GemsAudioEngine;
use wildside_render::{NativeLevelVisuals, load_level_visuals_with_source};
// Symbols the bin build no longer touches now the rasteriser lives in
// wildside-frame; this module's raster tests still reach them via `super::*`.
#[cfg(test)]
use std::collections::BTreeSet;
#[cfg(test)]
use wildside_core::{
    ActiveProjectileKind, ActiveProjectileSnapshot, BlockBounceTraceKind,
    DiamondPowerSpriteSnapshot, DragonFlyingDescentFireState, NativeContactPhase,
    NativeObjectSnapshot,
};
// Test fixtures load visuals through the legacy default-fs path.
#[cfg(test)]
use wildside_render::load_level_visuals;

const FRAME_TIME: Duration = Duration::from_micros(16_667);
const LEVEL_CARD_HOLD_FRAMES: usize = 60;
const AUDIO_SAMPLE_RATE: u32 = 44_100;
const AUDIO_FRAME_RATE_HZ: u32 = 60;
const AUDIO_SAMPLES_PER_FRAME: usize = 735;
const AUDIO_QUEUE_TARGET_FRAMES: usize = 2;
const AUDIO_SOFTWARE_BACKLOG_MAX_FRAMES: usize = 60;
const SDL_AUDIO_CALLBACK_SAMPLES: u16 = 512;
#[cfg(test)]
const PLAYER_COLOUR: [u8; 3] = [92, 214, 104];
#[cfg(test)]
const PLATFORM_COLOUR: [u8; 3] = [232, 176, 64];
#[cfg(test)]
const GUIDE_COLOUR: [u8; 3] = [62, 84, 108];

#[derive(Debug, Clone)]
pub struct NativePlayOptions {
    pub fullscreen: bool,
    pub scale: Option<u32>,
    pub debug_overlay: bool,
    pub warp_level: Option<u8>,
    pub settled: bool,
    pub record_to: Option<PathBuf>,
    pub record_frames: Option<usize>,
    pub record_stop_on_target_enemy_contact: bool,
    pub record_contact_tail_frames: usize,
    pub record_notes: Option<String>,
    pub record_route_target: Option<RecordingRouteTarget>,
    pub record_frontier_kind: Option<String>,
    pub scripted_route: Option<NativeScriptedRoute>,
}

#[derive(Debug, Clone)]
pub struct NativeScriptedRoute {
    pub script: Vec<u8>,
    pub delay_recording_input: bool,
    pub start_position: Option<DynamicStartPosition>,
    pub start_velocity: Option<crate::scenarios::DynamicStartVelocity>,
    pub camera_position: Option<DynamicStartPosition>,
    pub playtest_form: Option<u16>,
    pub initial_diamond_count: Option<u16>,
    pub initial_lives: Option<u16>,
}

impl From<DynamicScenario> for NativeScriptedRoute {
    fn from(scenario: DynamicScenario) -> Self {
        Self {
            script: scenario.script,
            delay_recording_input: scenario.settle_oracle_warp,
            start_position: scenario.start_position,
            start_velocity: scenario.start_velocity,
            camera_position: scenario.camera_position,
            playtest_form: scenario.playtest_form,
            initial_diamond_count: scenario.initial_diamond_count,
            initial_lives: scenario.initial_lives,
        }
    }
}

#[derive(Debug, Clone)]
struct NativeSessionAssets {
    level_visuals: NativeLevelVisuals,
    block_grid: BlockGrid,
}

// The SDL-free rasteriser (scene/render/sprites/hud + its DTOs, diagnostics,
// frame geometry, VRAM bases, and rendering colours) now lives in
// `wildside-frame`. Glob-re-export its public API so the native player, its
// sibling consumers, and the raster tests keep resolving the same
// `crate::native_player::X` paths unchanged.
pub(crate) use wildside_frame::*;

pub(crate) const NATIVE_FRAME_WIDTH: u32 = FRAME_WIDTH;
pub(crate) const NATIVE_FRAME_HEIGHT: u32 = FRAME_HEIGHT;

/// Render the native engine's own VDP reconstruction into `rgb` (320×224×3).
///
/// `VdpState` (wildside-core) and `VdpSnapshot` (wildside-md) carry identical fields; we
/// copy lazily on render rather than coupling wildside-core capture to wildside-md.
/// `render_vdp_into` hard-codes 320×224, matching `NATIVE_FRAME_WIDTH/HEIGHT`.
pub(crate) fn render_native_vdp_rgb(vdp: &wildside_core::vdp::VdpState, rgb: &mut [u8]) {
    let snapshot = wildside_md::VdpSnapshot {
        registers: vdp.registers,
        vram: vdp.vram.clone(),
        cram: vdp.cram.clone(),
        vsram: vdp.vsram.clone(),
    };
    wildside_md::render_vdp_into(&snapshot, rgb);
}

pub(crate) fn load_session_assets(
    manifest: &AssetManifest,
    level_id: u8,
) -> Result<(NativeLevelVisuals, BlockGrid)> {
    let assets = load_native_session_assets(manifest, level_id)?;
    Ok((assets.level_visuals, assets.block_grid))
}

// Frame-evidence helpers stay CLI-side: they bind the SDL-free rasteriser
// (`build_scene`/`render_scene`) to the asset loader (`load_native_session_assets`)
// and the scenario checkpoint types, which `wildside-frame` does not depend on.
pub(crate) fn render_gameplay_frame_evidence(
    manifest: &AssetManifest,
    trace: &[NativeTraceFrame],
    checkpoints: &[GameplayCheckpoint],
) -> Result<Vec<(String, usize, usize, usize, Vec<u8>)>> {
    render_gameplay_frame_evidence_for_level(manifest, trace, checkpoints, 0x00)
}

pub(crate) fn render_dynamic_gameplay_frame_evidence(
    manifest: &AssetManifest,
    trace: &[NativeTraceFrame],
    checkpoints: &[DynamicCheckpoint],
) -> Result<Vec<(String, usize, usize, usize, Vec<u8>)>> {
    render_dynamic_gameplay_frame_evidence_for_level(manifest, trace, checkpoints, 0x00)
}

pub(crate) fn render_gameplay_frame_evidence_for_level(
    manifest: &AssetManifest,
    trace: &[NativeTraceFrame],
    checkpoints: &[GameplayCheckpoint],
    level_id: u8,
) -> Result<Vec<(String, usize, usize, usize, Vec<u8>)>> {
    let session_assets = load_native_session_assets(manifest, level_id)?;
    checkpoints
        .iter()
        .map(|checkpoint| {
            let frame = trace
                .iter()
                .find(|frame| frame.frame_index == checkpoint.frame_index)
                .or_else(|| trace.get(checkpoint.frame_index))
                .ok_or_else(|| {
                    anyhow!(
                        "missing native gameplay frame evidence {}@{}",
                        checkpoint.label,
                        checkpoint.frame_index
                    )
                })?;
            let scene = build_scene(
                frame,
                false,
                Some(&session_assets.level_visuals),
                Some(&session_assets.block_grid),
            );
            let mut rgb = vec![0; (FRAME_WIDTH * FRAME_HEIGHT * 3) as usize];
            render_scene(&scene, &mut rgb);
            Ok((
                checkpoint.label.to_string(),
                checkpoint.frame_index,
                FRAME_WIDTH as usize,
                FRAME_HEIGHT as usize,
                rgb,
            ))
        })
        .collect()
}

pub(crate) fn render_dynamic_gameplay_frame_evidence_for_level(
    manifest: &AssetManifest,
    trace: &[NativeTraceFrame],
    checkpoints: &[DynamicCheckpoint],
    level_id: u8,
) -> Result<Vec<(String, usize, usize, usize, Vec<u8>)>> {
    let session_assets = load_native_session_assets(manifest, level_id)?;
    checkpoints
        .iter()
        .map(|checkpoint| {
            let frame = trace
                .iter()
                .find(|frame| frame.frame_index == checkpoint.frame_index)
                .or_else(|| trace.get(checkpoint.frame_index))
                .ok_or_else(|| {
                    anyhow!(
                        "missing native gameplay frame evidence {}@{}",
                        checkpoint.label,
                        checkpoint.frame_index
                    )
                })?;
            let scene = build_scene(
                frame,
                false,
                Some(&session_assets.level_visuals),
                Some(&session_assets.block_grid),
            );
            let mut rgb = vec![0; (FRAME_WIDTH * FRAME_HEIGHT * 3) as usize];
            render_scene(&scene, &mut rgb);
            Ok((
                checkpoint.label.clone(),
                checkpoint.frame_index,
                FRAME_WIDTH as usize,
                FRAME_HEIGHT as usize,
                rgb,
            ))
        })
        .collect()
}

pub fn run_native_player(manifest: &AssetManifest, options: &NativePlayOptions) -> Result<()> {
    let initial_level_id = options.warp_level.unwrap_or(0x00);
    let mut session_assets = load_native_session_assets(manifest, initial_level_id)?;
    let sdl = sdl2::init().map_err(|error| anyhow!("failed to init SDL: {error}"))?;
    let video = sdl
        .video()
        .map_err(|error| anyhow!("failed to init SDL video: {error}"))?;
    let game_controller = sdl
        .game_controller()
        .map_err(|error| anyhow!("failed to init SDL game controller subsystem: {error}"))?;

    let usable_bounds = video
        .display_usable_bounds(0)
        .unwrap_or_else(|_| Rect::new(0, 0, 1280, 720));
    let initial_scale = display_scale(options.scale, usable_bounds)?;
    let window = video
        .window(
            "Kid Chameleon Native",
            FRAME_WIDTH * initial_scale,
            FRAME_HEIGHT * initial_scale,
        )
        .position_centered()
        .resizable()
        .build()
        .map_err(|error| anyhow!("failed to create SDL window: {error}"))?;
    let mut canvas = window
        .into_canvas()
        .present_vsync()
        .build()
        .map_err(|error| anyhow!("failed to create SDL canvas: {error}"))?;
    canvas
        .set_logical_size(FRAME_WIDTH, FRAME_HEIGHT)
        .map_err(|error| anyhow!("failed to set SDL logical size: {error}"))?;
    canvas
        .set_integer_scale(true)
        .map_err(|error| anyhow!("failed to enable SDL integer scaling: {error}"))?;

    let mut fullscreen = options.fullscreen;
    apply_window_mode(&mut canvas, &video, fullscreen, options.scale)?;

    let texture_creator = canvas.texture_creator();
    let mut texture = texture_creator
        .create_texture_streaming(PixelFormatEnum::RGB24, FRAME_WIDTH, FRAME_HEIGHT)
        .map_err(|error| anyhow!("failed to create SDL texture: {error}"))?;
    let mut events = sdl
        .event_pump()
        .map_err(|error| anyhow!("failed to create SDL event pump: {error}"))?;
    let mut controller = open_first_controller(&game_controller)?;

    let mut gems_engine = try_load_gems_engine(manifest);
    let has_gems = gems_engine.is_some();
    // Stereo if GEMS is available, mono otherwise
    let audio_channels: u8 = if has_gems { 2 } else { 1 };
    let audio_queue: Option<AudioQueue<f32>> = match sdl.audio() {
        Ok(audio) => {
            let desired = sdl2::audio::AudioSpecDesired {
                freq: Some(AUDIO_SAMPLE_RATE as i32),
                channels: Some(audio_channels),
                samples: Some(SDL_AUDIO_CALLBACK_SAMPLES),
            };
            match audio.open_queue::<f32, _>(None, &desired) {
                Ok(queue) => Some(queue),
                Err(e) => {
                    eprintln!("Audio: failed to open queue: {e}");
                    eprintln!("  Check SDL2 audio drivers: SDL_AUDIODRIVER env var");
                    None
                }
            }
        }
        Err(e) => {
            eprintln!("Audio: SDL2 audio subsystem unavailable: {e}");
            eprintln!("  Ensure SDL2 is built with audio support");
            None
        }
    };
    match (&audio_queue, &gems_engine) {
        (Some(_), Some(_)) => {
            eprintln!("Audio: GEMS engine active (stereo, 44100 Hz)");
        }
        (Some(_), None) => eprintln!("Audio: SFX fallback (GEMS data unavailable)"),
        (None, _) => eprintln!("Audio: disabled — no sound will play"),
    }
    let audio_output_sample_rate = audio_queue
        .as_ref()
        .and_then(|queue| u32::try_from(queue.spec().freq).ok())
        .filter(|sample_rate| *sample_rate > 0)
        .unwrap_or(AUDIO_SAMPLE_RATE);
    if audio_output_sample_rate != AUDIO_SAMPLE_RATE {
        eprintln!(
            "Audio: SDL device opened at {audio_output_sample_rate} Hz; generating live audio at device rate"
        );
    }
    let mut audio_sample_clock = AudioSampleClock::new(audio_output_sample_rate);
    let mut sfx_engine = SfxEngine::new(audio_output_sample_rate);

    let mut runtime = initialise_runtime(manifest, &session_assets, options);
    let mut trace = runtime.trace_frame();
    #[allow(unused_assignments)]
    let mut prev_trace = trace.clone();
    let mut debug_overlay_visible = options.debug_overlay;
    let mut frame = vec![0; (FRAME_WIDTH * FRAME_HEIGHT * 3) as usize];
    let mut current_level_id: u8 = trace.current_level.0;
    let mut active_bgm_cue: Option<NativeBgmCue> = None;
    let mut pending_bgm_start: Option<u8> = None;
    let mut fade_alpha: u8 = 0; // 0=no fade, 255=fully black
    let mut fading_in = false; // true: fade from black to clear
    let mut level_card_frames_remaining: usize = 0;
    let mut debug_level_card_fallback_frames_remaining: usize = 0;
    let mut suppress_level_card_for_debug_warp = false;
    let mut wait_for_live_input_release = false;
    let mut recorded_script = Vec::new();
    let mut recorded_trace = Vec::new();
    let mut target_contact_stop_tail_remaining: Option<usize> = None;
    let mut target_contact_stop: Option<TargetEnemyContactStop> = None;
    let mut recorded_audio = Vec::new();
    let mut recorded_audio_events = Vec::new();
    let mut audio_diagnostics = NativeAudioDiagnostics::new(
        has_gems,
        audio_queue.is_some(),
        audio_channels,
        audio_output_sample_rate,
    );
    let mut final_frame_rgb = options.record_to.as_ref().map(|_| frame.clone());
    let mut audio_queue_priming = audio_queue.is_some();
    let mut live_audio_backlog = LiveAudioBacklog::new(AUDIO_SOFTWARE_BACKLOG_MAX_FRAMES);
    let audio_recording_sample_rate = if audio_queue.is_some() {
        audio_output_sample_rate
    } else {
        AUDIO_SAMPLE_RATE
    };
    if let Some(ref aq) = audio_queue {
        aq.pause();
        audio_diagnostics.record_queue_prime();
    }

    if options.record_to.is_some() {
        eprintln!("Recording enabled: press F10 to save the current bundle and exit.");
    }

    if let Some(vdp) = runtime.vdp_state() {
        render_native_vdp_rgb(vdp, &mut frame);
        render_vdp_trace_overlay(&trace, &session_assets.level_visuals, &mut frame);
    } else {
        let scene = build_scene(
            &trace,
            debug_overlay_visible,
            Some(&session_assets.level_visuals),
            runtime.block_grid(),
        );
        render_scene(&scene, &mut frame);
    }
    texture
        .update(None, &frame, (FRAME_WIDTH as usize) * 3)
        .map_err(|error| anyhow!("failed to update SDL texture: {error}"))?;
    canvas.clear();
    canvas
        .copy(&texture, None, None)
        .map_err(|error| anyhow!("failed to copy SDL texture: {error}"))?;
    canvas.present();

    'running: loop {
        let frame_started = Instant::now();
        let mut toggle_fullscreen = false;
        let mut reset_runtime = false;
        let mut dump_recording_and_exit = false;

        for event in events.poll_iter() {
            match event {
                Event::Quit { .. } => break 'running,
                Event::ControllerDeviceAdded { .. } => {
                    if controller.is_none() {
                        controller = open_first_controller(&game_controller)?;
                    }
                }
                Event::ControllerDeviceRemoved { .. } => {
                    controller = controller.filter(sdl2::controller::GameController::attached);
                    if controller.is_none() {
                        controller = open_first_controller(&game_controller)?;
                    }
                }
                Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    repeat: false,
                    ..
                } => break 'running,
                Event::KeyDown {
                    keycode: Some(Keycode::F11),
                    repeat: false,
                    ..
                } => toggle_fullscreen = true,
                Event::KeyDown {
                    keycode: Some(Keycode::Tab),
                    repeat: false,
                    ..
                } => debug_overlay_visible = !debug_overlay_visible,
                Event::KeyDown {
                    keycode: Some(Keycode::F10),
                    repeat: false,
                    ..
                } => {
                    if options.record_to.is_some() {
                        dump_recording_and_exit = true;
                    }
                }
                Event::KeyDown {
                    keycode: Some(Keycode::R),
                    repeat: false,
                    ..
                } => {
                    if options.record_to.is_some() {
                        bail!(
                            "play-native recording does not support reset/warp debug controls (R/N/P)"
                        );
                    }
                    reset_runtime = true;
                    wait_for_live_input_release = true;
                }
                // Debug: N = next level, P = previous level
                Event::KeyDown {
                    keycode: Some(Keycode::N),
                    repeat: false,
                    ..
                } => {
                    if options.record_to.is_some() {
                        bail!(
                            "play-native recording does not support reset/warp debug controls (R/N/P)"
                        );
                    }
                    if let Some((next, next_assets)) = load_debug_warp_session_assets(
                        manifest,
                        trace.current_level,
                        DebugWarpDirection::Next,
                    ) {
                        let presentation = enter_debug_hotkey_level(
                            manifest,
                            &mut runtime,
                            &mut trace,
                            &mut session_assets,
                            next,
                            next_assets,
                        );
                        current_level_id = next.0;
                        level_card_frames_remaining = presentation.level_card_frames_remaining;
                        debug_level_card_fallback_frames_remaining =
                            presentation.debug_level_card_fallback_frames_remaining;
                        fade_alpha = presentation.fade_alpha;
                        fading_in = presentation.fading_in;
                        suppress_level_card_for_debug_warp =
                            presentation.suppress_level_card_for_debug_warp;
                        wait_for_live_input_release = presentation.wait_for_live_input_release;
                    } else {
                        eprintln!("no playable next level with generated visuals and block layout");
                    }
                }
                Event::KeyDown {
                    keycode: Some(Keycode::P),
                    repeat: false,
                    ..
                } => {
                    if options.record_to.is_some() {
                        bail!(
                            "play-native recording does not support reset/warp debug controls (R/N/P)"
                        );
                    }
                    if let Some((prev, prev_assets)) = load_debug_warp_session_assets(
                        manifest,
                        trace.current_level,
                        DebugWarpDirection::Previous,
                    ) {
                        let presentation = enter_debug_hotkey_level(
                            manifest,
                            &mut runtime,
                            &mut trace,
                            &mut session_assets,
                            prev,
                            prev_assets,
                        );
                        current_level_id = prev.0;
                        level_card_frames_remaining = presentation.level_card_frames_remaining;
                        debug_level_card_fallback_frames_remaining =
                            presentation.debug_level_card_fallback_frames_remaining;
                        fade_alpha = presentation.fade_alpha;
                        fading_in = presentation.fading_in;
                        suppress_level_card_for_debug_warp =
                            presentation.suppress_level_card_for_debug_warp;
                        wait_for_live_input_release = presentation.wait_for_live_input_release;
                    } else {
                        eprintln!(
                            "no playable previous level with generated visuals and block layout"
                        );
                    }
                }
                _ => {}
            }
        }

        if dump_recording_and_exit {
            break 'running;
        }

        if toggle_fullscreen {
            fullscreen = !fullscreen;
            apply_window_mode(&mut canvas, &video, fullscreen, options.scale)?;
        }

        if reset_runtime {
            session_assets = load_native_session_assets(manifest, initial_level_id)?;
            runtime = initialise_runtime(manifest, &session_assets, options);
            trace = runtime.trace_frame();
            current_level_id = trace.current_level.0;
            active_bgm_cue = None;
            level_card_frames_remaining = 0;
            debug_level_card_fallback_frames_remaining = 0;
            suppress_level_card_for_debug_warp = false;
        } else {
            prev_trace = trace.clone();
            let level_card_hold_active = native_level_card_hold_active(
                level_card_frames_remaining,
                debug_level_card_fallback_frames_remaining,
            );
            let should_step_during_hold = should_step_runtime_during_level_card_hold(
                level_card_frames_remaining,
                debug_level_card_fallback_frames_remaining,
            );
            if !level_card_hold_active || should_step_during_hold {
                let buttons = if should_step_during_hold {
                    0
                } else {
                    let live_buttons =
                        keyboard_buttons(&events) | controller_buttons(controller.as_ref());
                    let live_buttons = live_buttons_after_debug_release(
                        live_buttons,
                        &mut wait_for_live_input_release,
                    );
                    native_frame_buttons(
                        options
                            .scripted_route
                            .as_ref()
                            .map(|route| (route.script.as_slice(), route.delay_recording_input)),
                        recorded_trace.len(),
                        live_buttons,
                    )
                };
                trace = runtime.step_frame(FrameInput::from_mask(buttons));
                if options.record_to.is_some() {
                    recorded_script.push(buttons);
                    recorded_trace.push(trace.clone());
                    if update_target_contact_auto_stop(
                        options,
                        &trace,
                        recorded_trace.len().saturating_sub(1),
                        &mut target_contact_stop_tail_remaining,
                        &mut target_contact_stop,
                    ) {
                        dump_recording_and_exit = true;
                    }
                    if recording_frame_limit_reached(options, recorded_trace.len()) {
                        dump_recording_and_exit = true;
                    }
                }
            }

            // Detect game events by comparing frames
            if !has_gems {
                detect_sfx_events(&prev_trace, &trace, &mut sfx_engine);
            }

            // Death sequence: fade to black (the original cuts to black instantly,
            // but a brief fade feels closer than the previous aggressive shake/flash).
            if trace.number_lives < prev_trace.number_lives {
                fade_alpha = 80;
                fading_in = false;
            }
            // Respawn: fade back in when death windup ends
            if prev_trace.death_windup_elapsed > 0
                && trace.death_windup_elapsed == 0
                && fade_alpha > 0
            {
                fading_in = true;
            }

            if should_start_native_level_card_overlay(
                &prev_trace,
                &trace,
                options.warp_level,
                options.settled,
            ) {
                level_card_frames_remaining = LEVEL_CARD_HOLD_FRAMES;
            }
        }

        // Trigger BGM before stepping audio so title/level transitions do not
        // emit one more frame of stale music.
        let level_changed = trace.current_level.0 != current_level_id;
        if level_changed {
            let new_level = trace.current_level.0;
            match load_level_change_session_assets(manifest, LevelId(new_level)) {
                Ok((resolved_level, new_assets)) => {
                    apply_live_level_change_session_assets(
                        &mut runtime,
                        &mut trace,
                        &mut session_assets,
                        LevelId(new_level),
                        resolved_level,
                        new_assets,
                    );
                    current_level_id = resolved_level.0;
                }
                Err(error) => {
                    eprintln!(
                        "failed to load visuals for level 0x{new_level:02X} or any later playable level: {error:#}"
                    );
                }
            }
            fade_alpha = 255;
            fading_in = true;
            if should_start_native_level_card_overlay_for_level_change(
                &trace,
                options.warp_level,
                options.settled,
                suppress_level_card_for_debug_warp,
            ) {
                level_card_frames_remaining = LEVEL_CARD_HOLD_FRAMES;
            }
            suppress_level_card_for_debug_warp = false;
        }
        let mut queued_bgm_start_this_frame = false;
        let mut suppress_sfx_this_frame = false;
        let (next_bgm_cue, bgm_command) =
            native_bgm_state_after_trace(manifest, &trace, active_bgm_cue);
        if let Some(command) = bgm_command
            && options.record_to.is_some()
        {
            recorded_audio_events.push(format_native_bgm_event(
                recorded_trace.len().saturating_sub(1),
                &trace,
                active_bgm_cue,
                next_bgm_cue,
                command,
            ));
        }
        if let Some(command) = bgm_command {
            if let Some(ref mut gems) = gems_engine {
                pending_bgm_start = apply_native_bgm_command(gems, command);
                queued_bgm_start_this_frame = pending_bgm_start.is_some();
            } else if native_bgm_command_flushes_queued_audio(command) {
                sfx_engine.clear();
            }
            if native_bgm_command_flushes_queued_audio(command)
                && let Some(ref aq) = audio_queue
            {
                aq.pause();
                aq.clear();
                live_audio_backlog.clear();
                audio_queue_priming = true;
                audio_diagnostics.record_queue_clear();
                audio_diagnostics.record_queue_prime();
            }
            suppress_sfx_this_frame =
                native_bgm_command_suppresses_same_frame_sfx(command, &trace, active_bgm_cue);
        }
        active_bgm_cue = next_bgm_cue;

        let should_capture_audio = options.record_to.is_some();
        let should_step_audio = should_capture_audio || audio_queue.is_some();
        if should_step_audio {
            if let Some(ref aq) = audio_queue {
                queue_live_audio_backlog(
                    aq,
                    &mut live_audio_backlog,
                    &mut audio_diagnostics,
                    &mut audio_queue_priming,
                );
            }
            let audio_generation = native_audio_generation_plan(
                audio_queue.is_some(),
                audio_output_sample_rate,
                &mut audio_sample_clock,
            );
            let maybe_samples = if let Some(ref mut gems) = gems_engine {
                if !queued_bgm_start_this_frame && let Some(bgm) = pending_bgm_start.take() {
                    gems.start_sequence(bgm);
                }
                if !suppress_sfx_this_frame {
                    detect_gems_events(&prev_trace, &trace, gems);
                }
                gems.step_frame();
                if audio_generation.sample_rate == AUDIO_SAMPLE_RATE {
                    Some(gems.generate_samples(audio_generation.samples_per_frame))
                } else {
                    Some(gems.generate_samples_at_rate(
                        audio_generation.sample_rate,
                        audio_generation.samples_per_frame,
                    ))
                }
            } else {
                Some(sfx_engine.generate(audio_generation.samples_per_frame))
            };
            if let Some(samples) = maybe_samples {
                audio_diagnostics.record_generated_frame(samples.len());
                if should_capture_audio {
                    recorded_audio.extend_from_slice(&samples);
                }
                if let Some(ref aq) = audio_queue {
                    if live_audio_backlog.push_frame(samples) {
                        audio_diagnostics.record_backlog_frame(live_audio_backlog.pending_frames());
                    } else {
                        audio_diagnostics
                            .record_backlog_full_frame(live_audio_backlog.pending_frames());
                    }
                    queue_live_audio_backlog(
                        aq,
                        &mut live_audio_backlog,
                        &mut audio_diagnostics,
                        &mut audio_queue_priming,
                    );
                }
            }
        }

        let render_debug_level_card_fallback = debug_level_card_fallback_frames_remaining > 0;
        let base_render_trace = if native_level_card_hold_active(
            level_card_frames_remaining,
            debug_level_card_fallback_frames_remaining,
        ) {
            let mut overlay_trace = trace.clone();
            overlay_trace.game_mode = GameMode::LevelTransition;
            overlay_trace.game_mode_value = GameMode::LevelTransition.as_disassembly_value();
            overlay_trace
        } else {
            trace.clone()
        };
        let render_trace =
            render_trace_with_vdp_latched_state(&base_render_trace, Some(&prev_trace));

        if render_debug_level_card_fallback {
            let scene = build_debug_level_card_fallback_scene(&render_trace);
            render_scene(&scene, &mut frame);
        } else if let Some(vdp) = runtime.vdp_state() {
            // Non-gameplay mode with VDP state: render directly using the
            // proven software VDP renderer for pixel-accurate output.
            render_native_vdp_rgb(vdp, &mut frame);
            render_vdp_trace_overlay(&render_trace, &session_assets.level_visuals, &mut frame);
        } else {
            let scene = build_scene(
                &render_trace,
                debug_overlay_visible,
                Some(&session_assets.level_visuals),
                runtime.block_grid(),
            );
            render_scene(&scene, &mut frame);
        }

        // Apply fade overlay: darken all pixels by fade_alpha/255
        if fade_alpha > 0 {
            let inv = 255 - u16::from(fade_alpha);
            for pixel in &mut frame {
                *pixel = (u16::from(*pixel) * inv / 255) as u8;
            }
            if fading_in {
                fade_alpha = fade_alpha.saturating_sub(8);
            } else if fade_alpha < 220 {
                // Ramp up to near-black during death sequence
                fade_alpha = fade_alpha.saturating_add(3);
            }
        }

        // Diamond Power golden tint: add warm overlay when active
        if trace.diamond_power_active {
            let pulse = ((trace.frame_index % 16) as u8).min(8);
            let tint_r: u8 = 30 + pulse * 3;
            let tint_g: u8 = 20 + pulse * 2;
            for pixel in frame.chunks_exact_mut(3) {
                pixel[0] = pixel[0].saturating_add(tint_r);
                pixel[1] = pixel[1].saturating_add(tint_g);
            }
        }

        texture
            .update(None, &frame, (FRAME_WIDTH as usize) * 3)
            .map_err(|error| anyhow!("failed to update SDL texture: {error}"))?;
        if options.record_to.is_some() {
            final_frame_rgb = Some(frame.clone());
        }
        canvas.clear();
        canvas
            .copy(&texture, None, None)
            .map_err(|error| anyhow!("failed to copy SDL texture: {error}"))?;
        canvas.present();

        level_card_frames_remaining = level_card_frames_remaining.saturating_sub(1);
        debug_level_card_fallback_frames_remaining =
            debug_level_card_fallback_frames_remaining.saturating_sub(1);

        if dump_recording_and_exit {
            break 'running;
        }

        let elapsed = frame_started.elapsed();
        audio_diagnostics.record_frame_timing(elapsed, FRAME_TIME);
        if let Some(remaining) = FRAME_TIME.checked_sub(elapsed) {
            thread::sleep(remaining);
        }
    }

    save_native_recording_bundle(
        manifest,
        options,
        recorded_script,
        recorded_trace,
        recorded_audio,
        recorded_audio_events,
        audio_diagnostics.into_lines(),
        final_frame_rgb,
        trace,
        target_contact_stop.as_ref(),
        audio_channels,
        audio_recording_sample_rate,
    )?;

    Ok(())
}

pub(crate) fn render_vdp_trace_overlay(
    trace: &NativeTraceFrame,
    level_visuals: &NativeLevelVisuals,
    frame: &mut [u8],
) {
    // ASM Load_IntroSequence4 (`sub_1B988`) sets `Camera_Y_pos=4` and allocates
    // a single object running `loc_1C7A0` with `IntroText4` — the holographic
    // background plus the story-text writer. It never allocates a player/Kid
    // GfxObjectSlot: the per-glyph kid montage poses belong to
    // `Load_IntroSequence3` (`unk_1AF24`/`loc_1AEEC`) and `Load_IntroSequence5`
    // (`loc_1BCD0`), not IntroSequence4. The previous overlay composited a
    // normal Kid sprite here from a stale carried `kid_gfx` pose
    // (kid_x/kid_y ~ $60/$D0), which drew a spurious Kid at the bottom-left of
    // the story screen (#210). No player overlay belongs on IntroSequence4.
    if trace.boot_phase.as_deref() != Some("starting_level")
        || trace.game_mode != GameMode::LevelTransition
    {
        return;
    }

    // ASM Load_TitleCard has two independent pieces: `Load_TitleCard` builds
    // the static VDP shell, then `loc_1AB26` / `loc_1AC70` animate
    // Title_Letter gfx workers over it. The VDP path owns the shell; this
    // overlay keeps the ROM-derived letter workers visible without restoring
    // the old invented static level-card text.
    let overlay = build_title_card_worker_overlay_scene(trace, Some(level_visuals));
    if !overlay.commands.is_empty() {
        render_scene_overlay(&overlay, frame);
    }
}

fn recording_frame_limit_reached(options: &NativePlayOptions, recorded_frames: usize) -> bool {
    options
        .record_frames
        .is_some_and(|limit| recorded_frames >= limit)
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct TargetEnemyContactStop {
    frame_index: usize,
    object_type: u8,
    result: String,
    gate: String,
    score_px: i64,
}

fn update_target_contact_auto_stop(
    options: &NativePlayOptions,
    trace: &NativeTraceFrame,
    replay_frame_index: usize,
    tail_remaining: &mut Option<usize>,
    detected_contact: &mut Option<TargetEnemyContactStop>,
) -> bool {
    if let Some(remaining) = tail_remaining {
        if *remaining == 0 {
            return true;
        }
        *remaining -= 1;
        return false;
    }

    let Some(target) = &options.record_route_target else {
        return false;
    };
    if !options.record_stop_on_target_enemy_contact {
        return false;
    }
    let Some(contact) = target
        .enemies
        .iter()
        .filter_map(|enemy| {
            target_enemy_contact_from_frame(std::slice::from_ref(trace), *enemy, 0)
                .map(|contact| (*enemy, contact))
        })
        .find(|(_, contact)| contact.result != "none")
    else {
        return false;
    };
    let (object_type, contact) = contact;
    detected_contact.get_or_insert_with(|| TargetEnemyContactStop {
        frame_index: replay_frame_index,
        object_type,
        result: contact.result.clone(),
        gate: contact.gate.clone(),
        score_px: contact.score_px,
    });
    let tail = options.record_contact_tail_frames;
    eprintln!(
        "recording target enemy contact detected: enemy=0x{object_type:02X} frame={replay_frame_index} result={} gate={} score_px={} tail_frames={}",
        contact.result, contact.gate, contact.score_px, tail
    );
    if tail == 0 {
        true
    } else {
        *tail_remaining = Some(tail - 1);
        false
    }
}

fn append_target_enemy_contact_checkpoint(
    scenario: &mut DynamicScenario,
    contact: &TargetEnemyContactStop,
) {
    if contact.frame_index >= scenario.frame_count {
        return;
    }
    let label = format!(
        "target_enemy_contact_{:02X}_{}",
        contact.object_type, contact.result
    );
    if scenario.checkpoints.iter().any(|checkpoint| {
        checkpoint.label == label && checkpoint.frame_index == contact.frame_index
    }) {
        return;
    }
    scenario.checkpoints.push(DynamicCheckpoint {
        label,
        frame_index: contact.frame_index,
    });
    scenario
        .checkpoints
        .sort_by_key(|checkpoint| checkpoint.frame_index);
}

fn native_frame_buttons(
    scripted_input: Option<(&[u8], bool)>,
    frame_index: usize,
    live_buttons: u8,
) -> u8 {
    scripted_input.map_or(live_buttons, |(script, delay_recording_input)| {
        let input_index = if delay_recording_input {
            frame_index.checked_sub(1)
        } else {
            Some(frame_index)
        };
        input_index
            .and_then(|index| script.get(index))
            .copied()
            .unwrap_or_default()
    })
}

fn live_buttons_after_debug_release(live_buttons: u8, waiting_for_release: &mut bool) -> u8 {
    if !*waiting_for_release {
        return live_buttons;
    }
    if live_buttons == 0 {
        *waiting_for_release = false;
    }
    0
}

fn native_level_card_hold_active(
    level_card_frames_remaining: usize,
    debug_level_card_fallback_frames_remaining: usize,
) -> bool {
    level_card_frames_remaining > 0 || debug_level_card_fallback_frames_remaining > 0
}

fn should_step_runtime_during_level_card_hold(
    level_card_frames_remaining: usize,
    debug_level_card_fallback_frames_remaining: usize,
) -> bool {
    let _ = debug_level_card_fallback_frames_remaining;
    level_card_frames_remaining > 0
}

#[cfg(test)]
fn debug_level_card_fallback_frames(manifest: &AssetManifest, level_id: LevelId) -> usize {
    let _ = (manifest, level_id);
    0
}

fn debug_hotkey_level_card_hold_frames() -> usize {
    // `N`/`P` are native-only debug hotkeys, not ROM input. The core still
    // enters ASM `loc_6E2`/`Load_TitleCard`; play-native must not add a native
    // hold in front of that path, or debug skips show a duplicate level intro.
    0
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct DebugHotkeyLevelPresentation {
    level_card_frames_remaining: usize,
    debug_level_card_fallback_frames_remaining: usize,
    fade_alpha: u8,
    fading_in: bool,
    suppress_level_card_for_debug_warp: bool,
    wait_for_live_input_release: bool,
}

fn enter_debug_hotkey_level(
    manifest: &AssetManifest,
    runtime: &mut NativeRuntime,
    trace: &mut NativeTraceFrame,
    session_assets: &mut NativeSessionAssets,
    level_id: LevelId,
    next_assets: NativeSessionAssets,
) -> DebugHotkeyLevelPresentation {
    *session_assets = next_assets;
    runtime.set_block_grid(session_assets.block_grid.clone());
    *trace = runtime.enter_level_debug_title_card(level_id);
    let has_renderable_title_card = level_has_generated_title_card(manifest, level_id);

    debug_hotkey_level_presentation(has_renderable_title_card)
}

fn apply_live_level_change_session_assets(
    runtime: &mut NativeRuntime,
    trace: &mut NativeTraceFrame,
    session_assets: &mut NativeSessionAssets,
    requested_level: LevelId,
    resolved_level: LevelId,
    next_assets: NativeSessionAssets,
) {
    *session_assets = next_assets;
    runtime.set_level_entry_block_grid(session_assets.block_grid.clone());
    if trace.game_mode == GameMode::LevelTransition {
        // ASM loc_D7F4/loc_74E0 re-enters loc_6E2/Load_TitleCard for the
        // target Current_LevelID. Rebuild the VDP shell after the asset/grid
        // swap so direct level changes do not render the previous level's
        // title-card background with target-level assets.
        *trace = runtime.enter_level_debug_title_card(resolved_level);
    } else if requested_level != resolved_level {
        *trace = runtime.enter_level_debug_title_card(resolved_level);
    }
}

fn debug_hotkey_level_presentation(
    has_renderable_title_card: bool,
) -> DebugHotkeyLevelPresentation {
    let hold_frames = debug_hotkey_level_card_hold_frames();
    let _ = has_renderable_title_card;
    DebugHotkeyLevelPresentation {
        level_card_frames_remaining: hold_frames,
        debug_level_card_fallback_frames_remaining: 0,
        fade_alpha: 0,
        fading_in: false,
        suppress_level_card_for_debug_warp: false,
        wait_for_live_input_release: true,
    }
}

fn level_has_generated_title_card(manifest: &AssetManifest, level_id: LevelId) -> bool {
    resource_level_record(manifest, level_id.0)
        .or_else(|| {
            manifest
                .levels
                .iter()
                .find(|level| level.level_id == level_id.0)
        })
        .and_then(|level| level.title_card.as_ref())
        .is_some()
}

fn build_debug_level_card_fallback_scene(trace: &NativeTraceFrame) -> Scene {
    let level_name = trace
        .current_level_name
        .clone()
        .unwrap_or_else(|| format!("LEVEL {:02X}", trace.current_level.0));
    let title = format!("LEVEL {:02X}", trace.current_level.0);
    let title_x = centred_text_x(&title, 2);
    let name_x = centred_text_x(&level_name, 1);

    Scene {
        background: [0, 0, 0],
        commands: vec![
            DrawCommand::Text {
                x: title_x,
                y: 82,
                text: title,
                colour: [210, 224, 255],
                scale: 2,
            },
            DrawCommand::Text {
                x: name_x,
                y: 116,
                text: level_name,
                colour: [255, 255, 255],
                scale: 1,
            },
        ],
        pause_palette_darken: false,
    }
}

fn centred_text_x(text: &str, scale: i32) -> i32 {
    let glyph_width = 8 * scale;
    let width = text.chars().count() as i32 * glyph_width;
    ((FRAME_WIDTH as i32 - width) / 2).max(0)
}

fn should_start_native_level_card_overlay(
    _previous_trace: &NativeTraceFrame,
    _trace: &NativeTraceFrame,
    _warp_level: Option<u8>,
    _settled: bool,
) -> bool {
    // The old native placeholder card could not distinguish real
    // `Load_TitleCard` from hidden death/continue bootstrap windows. The ROM
    // path is now rendered by VDP state plus the `loc_1AB26` title workers, so
    // generic mode changes must not re-enter a fake LevelTransition overlay.
    false
}

fn should_start_native_level_card_overlay_for_level_change(
    _trace: &NativeTraceFrame,
    _warp_level: Option<u8>,
    _settled: bool,
    _debug_hotkey_warp: bool,
) -> bool {
    // ASM Load_TitleCard (`loc_6E2` -> `loc_1AB26`) is the only real level-card
    // renderer. Level changes, debug warps, and death/continue restarts should
    // expose the ROM/current core state instead of a native placeholder.
    false
}

fn save_native_recording_bundle(
    manifest: &AssetManifest,
    options: &NativePlayOptions,
    mut recorded_script: Vec<u8>,
    mut recorded_trace: Vec<NativeTraceFrame>,
    recorded_audio: Vec<f32>,
    recorded_audio_events: Vec<String>,
    audio_diagnostics: Vec<String>,
    final_frame_rgb: Option<Vec<u8>>,
    current_trace: NativeTraceFrame,
    target_contact_stop: Option<&TargetEnemyContactStop>,
    audio_channels: u8,
    audio_sample_rate: u32,
) -> Result<()> {
    let Some(record_to) = &options.record_to else {
        return Ok(());
    };

    if recorded_trace.is_empty() {
        recorded_script.push(0);
        recorded_trace.push(current_trace);
    }
    if let Some(route) = &options.scripted_route {
        recorded_script = route
            .script
            .iter()
            .copied()
            .take(recorded_trace.len())
            .collect();
    }
    let recorded_audio_events = if options.scripted_route.is_some() {
        native_audio_events_for_trace(manifest, &recorded_trace)
    } else {
        recorded_audio_events
    };

    let slug = record_to
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| {
            anyhow!(
                "recording path has no valid file name: {}",
                record_to.display()
            )
        })?;
    let mut recording = build_recording(
        manifest,
        slug,
        recording_start_from_options(options),
        recorded_script,
        recorded_trace,
    )?;
    recording.manifest.record_command = native_recording_command(slug, options);
    recording.manifest.tas_frontier_command = recording_tas_frontier_command(
        slug,
        options.record_frontier_kind.as_deref().unwrap_or("enemy"),
    );
    recording
        .manifest
        .route_target
        .clone_from(&options.record_route_target);
    if let Some(contact) = target_contact_stop {
        append_target_enemy_contact_checkpoint(&mut recording.manifest.scenario, contact);
    }
    if let Some(route) = &options.scripted_route {
        recording.manifest.scenario.start_position = route.start_position;
        recording.manifest.scenario.start_velocity = route.start_velocity;
        recording.manifest.scenario.camera_position = route.camera_position;
        recording.manifest.scenario.playtest_form = route.playtest_form;
        recording.manifest.scenario.initial_diamond_count = route.initial_diamond_count;
        recording.manifest.scenario.initial_lives = route.initial_lives;
    }
    let artifacts = RecordingBundleArtifacts {
        user_notes: options.record_notes.clone(),
        final_frame_rgb,
        audio_samples: Some(recorded_audio),
        audio_sample_rate,
        audio_channels: u16::from(audio_channels),
        audio_events: recorded_audio_events,
        audio_diagnostics,
    };
    save_recording_with_artifacts(record_to, &recording, &artifacts)?;
    println!(
        "recording saved to {}. verify with: cargo run -p wildside-cli -- verify-recording --slug {}",
        record_to.display(),
        slug
    );
    for command in native_recording_follow_up_commands(slug, options) {
        println!("{command}");
    }
    Ok(())
}

fn native_recording_follow_up_commands(slug: &str, options: &NativePlayOptions) -> Vec<String> {
    let mut commands = vec![recording_tas_frontier_command(
        slug,
        options.record_frontier_kind.as_deref().unwrap_or("enemy"),
    )];
    if let Some(target) = &options.record_route_target {
        commands.extend(recording_route_target_score_commands(slug, target));
    }
    commands
}

fn native_recording_command(slug: &str, options: &NativePlayOptions) -> String {
    use std::fmt::Write;

    let mut command = String::from("cargo run --release -p wildside-cli -- play-native");
    if let Some(level) = options.warp_level {
        let _ = write!(command, " --warp-level {level:02X}");
    }
    if options.settled {
        command.push_str(" --settled");
    }
    let _ = write!(command, " --record {slug}");
    if let Some(frame_limit) = options.record_frames {
        let _ = write!(command, " --record-frames {frame_limit}");
    }
    if options.record_stop_on_target_enemy_contact {
        command.push_str(" --record-stop-on-target-enemy-contact");
        if options.record_contact_tail_frames != 30 {
            let _ = write!(
                command,
                " --record-contact-tail-frames {}",
                options.record_contact_tail_frames
            );
        }
    }
    if let Some(target) = &options.record_route_target {
        append_record_route_target_args(&mut command, target);
    }
    if let Some(frontier_kind) = &options.record_frontier_kind {
        let _ = write!(command, " --record-frontier-kind {frontier_kind}");
    }
    command
}

fn append_record_route_target_args(command: &mut String, target: &RecordingRouteTarget) {
    use std::fmt::Write;

    if let Some(label) = &target.label {
        let _ = write!(command, " --record-target-label {label}");
    }
    if let Some(level) = target.level {
        let _ = write!(command, " --record-target-level {level:02X}");
    }
    for point in &target.points {
        let _ = write!(
            command,
            " --record-target-point {},{}",
            point.x_px, point.y_px
        );
    }
    for cell in &target.cells {
        let _ = write!(command, " --record-target-cell {},{}", cell.col, cell.row);
    }
    for cell in &target.cell_hitboxes {
        let _ = write!(
            command,
            " --record-target-cell-hitbox {},{}",
            cell.col, cell.row
        );
    }
    for enemy in &target.enemies {
        let _ = write!(command, " --record-target-enemy {enemy:02X}");
    }
}

fn load_native_session_assets(
    manifest: &AssetManifest,
    level_id: u8,
) -> Result<NativeSessionAssets> {
    // Native conduit: build the bundle's fs source once and inject it into the
    // render read path. A wasm host swaps this for a fetched-bundle source.
    let source = FsAssetSource::new(manifest.output_root.clone());
    load_native_session_assets_with(manifest, level_id, |manifest, level_id| {
        load_level_visuals_with_source(manifest, level_id, &source)
    })
    .map_err(|error| {
        anyhow!(
            "{error:#}. play-native needs extracted level visuals and block layout; run `mise run extract-assets` if they are missing"
        )
    })
}

#[cfg(test)]
pub(crate) fn validate_native_session_assets(manifest: &AssetManifest, level_id: u8) -> Result<()> {
    load_native_session_assets(manifest, level_id).map(|_| ())
}

fn load_native_session_assets_with<F>(
    manifest: &AssetManifest,
    level_id: u8,
    mut visuals_loader: F,
) -> Result<NativeSessionAssets>
where
    F: FnMut(&AssetManifest, u8) -> Result<NativeLevelVisuals>,
{
    let level_visuals = visuals_loader(manifest, level_id)?;
    let block_grid = try_load_block_grid(manifest, level_id)?;
    Ok(NativeSessionAssets {
        level_visuals,
        block_grid,
    })
}

fn initialise_runtime(
    manifest: &AssetManifest,
    session_assets: &NativeSessionAssets,
    options: &NativePlayOptions,
) -> NativeRuntime {
    // Native conduit: one shared fs source feeds both runtime constructors. A
    // wasm host injects a fetched-bundle source here instead.
    let source: std::sync::Arc<dyn AssetSource> =
        std::sync::Arc::new(FsAssetSource::new(manifest.output_root.clone()));
    let mut runtime = if options.settled && options.warp_level.is_some() {
        NativeRuntime::with_generated_level_layout_with_source(manifest.clone(), source)
    } else {
        let mut runtime = NativeRuntime::new_with_source(manifest.clone(), source);
        runtime.set_block_grid(session_assets.block_grid.clone());
        runtime
    };
    if let Some(level_id) = options.warp_level {
        if options.settled {
            // ASM loc_1AE2E/loc_74E0 reaches settled recording warps through
            // the same level-skip shell that `verify-recording` replays. Keep
            // live `play-native --settled --record` bundles on that path so
            // fresh repros are replayable at frame 0 instead of starting from
            // the synthetic `enter_level_playtest` shortcut.
            let level_id = LevelId(level_id);
            runtime.boot_to_level(level_id);
            runtime.settle_recording_oracle_warp_live_window(level_id);
        } else {
            runtime.warp_to_level(LevelId(level_id));
        }
    } else if options.settled {
        runtime.enter_level_playtest(LevelId(0));
    }
    apply_scripted_route_probe_state(&mut runtime, options.scripted_route.as_ref())
        .expect("invalid scripted play-native route");
    runtime
}

fn apply_scripted_route_probe_state(
    runtime: &mut NativeRuntime,
    scripted_route: Option<&NativeScriptedRoute>,
) -> Result<()> {
    let Some(route) = scripted_route else {
        return Ok(());
    };
    if let Some(position) = route.start_position {
        runtime.set_player_probe_position(position.x, position.y);
    }
    if let Some(velocity) = route.start_velocity {
        runtime.set_player_probe_velocity(velocity.x, velocity.y);
    }
    if let Some(position) = route.camera_position {
        runtime.set_camera_probe_position(position.x, position.y);
    }
    if let Some(helmet_id) = route.playtest_form {
        let form = PlayerForm::from_helmet_id(helmet_id)
            .ok_or_else(|| anyhow!("invalid playtest_form helmet id {helmet_id}"))?;
        runtime.set_playtest_form(form);
    }
    if let Some(count) = route.initial_diamond_count {
        runtime.set_probe_diamond_count(count);
    }
    if let Some(count) = route.initial_lives {
        runtime.set_probe_lives_count(count);
    }
    Ok(())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DebugWarpDirection {
    Next,
    Previous,
}

fn load_debug_warp_session_assets(
    manifest: &AssetManifest,
    current_level: LevelId,
    direction: DebugWarpDirection,
) -> Option<(LevelId, NativeSessionAssets)> {
    load_debug_warp_session_assets_with(
        manifest,
        current_level,
        direction,
        load_native_session_assets,
    )
}

fn load_level_change_session_assets(
    manifest: &AssetManifest,
    target_level: LevelId,
) -> Result<(LevelId, NativeSessionAssets)> {
    load_level_change_session_assets_with(manifest, target_level, load_native_session_assets)
}

fn load_level_change_session_assets_with<F>(
    manifest: &AssetManifest,
    target_level: LevelId,
    mut load_session_assets: F,
) -> Result<(LevelId, NativeSessionAssets)>
where
    F: FnMut(&AssetManifest, u8) -> Result<NativeSessionAssets>,
{
    match load_session_assets(manifest, target_level.0) {
        Ok(assets) => return Ok((target_level, assets)),
        Err(first_error) => {
            eprintln!(
                "skipping level-change target 0x{:02X}: {first_error:#}",
                target_level.0
            );
        }
    }

    for candidate in debug_warp_candidate_ids(manifest, target_level, DebugWarpDirection::Next) {
        match load_session_assets(manifest, candidate.0) {
            Ok(assets) => return Ok((candidate, assets)),
            Err(error) => {
                eprintln!(
                    "skipping level-change target 0x{:02X}: {error:#}",
                    candidate.0
                );
            }
        }
    }

    Err(anyhow!(
        "no playable level with generated visuals and block layout after 0x{:02X}",
        target_level.0
    ))
}

fn load_debug_warp_session_assets_with<F>(
    manifest: &AssetManifest,
    current_level: LevelId,
    direction: DebugWarpDirection,
    mut load_session_assets: F,
) -> Option<(LevelId, NativeSessionAssets)>
where
    F: FnMut(&AssetManifest, u8) -> Result<NativeSessionAssets>,
{
    for candidate in debug_warp_candidate_ids(manifest, current_level, direction) {
        match load_session_assets(manifest, candidate.0) {
            Ok(assets) => return Some((candidate, assets)),
            Err(error) => {
                eprintln!(
                    "skipping debug warp target 0x{:02X}: {error:#}",
                    candidate.0
                );
            }
        }
    }
    None
}

fn debug_warp_candidate_ids(
    manifest: &AssetManifest,
    current_level: LevelId,
    direction: DebugWarpDirection,
) -> Vec<LevelId> {
    let mut level_ids = if manifest.map_order.is_empty() {
        manifest
            .levels
            .iter()
            .map(|level| LevelId(level.level_id))
            .collect::<Vec<_>>()
    } else {
        manifest
            .map_order
            .iter()
            .enumerate()
            .filter_map(|(logical_level_id, &resource_level_id)| {
                if resource_level_id == u8::MAX {
                    None
                } else {
                    u8::try_from(logical_level_id).ok().map(LevelId)
                }
            })
            .collect::<Vec<_>>()
    };

    level_ids.sort_unstable_by_key(|level| level.0);
    level_ids.dedup();
    if level_ids.is_empty() {
        return Vec::new();
    }

    let start = match level_ids.iter().position(|&level| level == current_level) {
        Some(pos) => pos,
        None => match direction {
            DebugWarpDirection::Next => level_ids
                .iter()
                .position(|&level| level.0 > current_level.0)
                .unwrap_or(0),
            DebugWarpDirection::Previous => level_ids
                .iter()
                .rposition(|&level| level.0 < current_level.0)
                .unwrap_or(level_ids.len() - 1),
        },
    };

    (1..=level_ids.len())
        .map(|offset| match direction {
            DebugWarpDirection::Next => level_ids[(start + offset) % level_ids.len()],
            DebugWarpDirection::Previous => {
                level_ids[(start + level_ids.len() - (offset % level_ids.len())) % level_ids.len()]
            }
        })
        .collect()
}

fn display_scale(requested: Option<u32>, usable_bounds: Rect) -> Result<u32> {
    match requested {
        Some(scale @ 1..=4) => Ok(scale),
        Some(scale) => bail!("unsupported native display scale {scale}; use 1-4"),
        None => {
            let width_scale = (usable_bounds.width() / FRAME_WIDTH).max(1);
            let height_scale = (usable_bounds.height() / FRAME_HEIGHT).max(1);
            Ok(width_scale.min(height_scale).clamp(1, 4))
        }
    }
}

fn apply_window_mode(
    canvas: &mut WindowCanvas,
    video: &sdl2::VideoSubsystem,
    fullscreen: bool,
    scale: Option<u32>,
) -> Result<()> {
    if fullscreen {
        canvas
            .window_mut()
            .set_fullscreen(FullscreenType::Desktop)
            .map_err(|error| anyhow!("failed to enable fullscreen: {error}"))?;
        return Ok(());
    }

    canvas
        .window_mut()
        .set_fullscreen(FullscreenType::Off)
        .map_err(|error| anyhow!("failed to disable fullscreen: {error}"))?;
    let usable_bounds = video
        .display_usable_bounds(0)
        .unwrap_or_else(|_| Rect::new(0, 0, 1280, 720));
    let scale = display_scale(scale, usable_bounds)?;
    canvas
        .window_mut()
        .set_size(FRAME_WIDTH * scale, FRAME_HEIGHT * scale)
        .map_err(|error| anyhow!("failed to resize SDL window: {error}"))?;
    canvas.window_mut().set_position(
        sdl2::video::WindowPos::Centered,
        sdl2::video::WindowPos::Centered,
    );
    Ok(())
}

fn keyboard_buttons(events: &sdl2::EventPump) -> u8 {
    let keyboard = events.keyboard_state();
    let bindings = [
        (&[Keycode::Up, Keycode::W][..], 1 << 0),
        (&[Keycode::Down, Keycode::S][..], 1 << 1),
        (&[Keycode::Left, Keycode::A][..], 1 << 2),
        (&[Keycode::Right, Keycode::D][..], 1 << 3),
        (&[Keycode::Z, Keycode::Space][..], 1 << 4),
        (&[Keycode::X][..], 1 << 5),
        (&[Keycode::C, Keycode::LShift, Keycode::RShift][..], 1 << 6),
        (&[Keycode::Return][..], 1 << 7),
    ];
    let mut buttons = 0u8;
    for (keycodes, mask) in bindings {
        if keycodes.iter().copied().any(|keycode| {
            Scancode::from_keycode(keycode)
                .is_some_and(|scancode| keyboard.is_scancode_pressed(scancode))
        }) {
            buttons |= mask;
        }
    }
    buttons
}

fn controller_buttons(controller: Option<&GameController>) -> u8 {
    let Some(controller) = controller else {
        return 0;
    };

    let mut buttons = 0u8;
    if controller.button(Button::DPadUp) {
        buttons |= 1 << 0;
    }
    if controller.button(Button::DPadDown) {
        buttons |= 1 << 1;
    }
    if controller.button(Button::DPadLeft) {
        buttons |= 1 << 2;
    }
    if controller.button(Button::DPadRight) {
        buttons |= 1 << 3;
    }
    if controller.button(Button::A) {
        buttons |= 1 << 4;
    }
    if controller.button(Button::B) {
        buttons |= 1 << 5;
    }
    if controller.button(Button::X) {
        buttons |= 1 << 6;
    }
    if controller.button(Button::Start) {
        buttons |= 1 << 7;
    }
    buttons
}

fn open_first_controller(
    game_controller: &sdl2::GameControllerSubsystem,
) -> Result<Option<GameController>> {
    let joystick_count = game_controller
        .num_joysticks()
        .map_err(|error| anyhow!("failed to enumerate SDL controllers: {error}"))?;
    for index in 0..joystick_count {
        if !game_controller.is_game_controller(index) {
            continue;
        }
        let controller = game_controller
            .open(index)
            .map_err(|error| anyhow!("failed to open SDL controller {index}: {error}"))?;
        return Ok(Some(controller));
    }
    Ok(None)
}

// ---------------------------------------------------------------------------
// Simple SFX synthesiser: generates square-wave tones for game events.
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy)]
struct SfxTone {
    frequency: f32,
    remaining_samples: u32,
    phase: f32,
    volume: f32,
}

/// Try to load a GEMS audio engine from extracted sound data in the disassembly.
fn try_load_gems_engine(manifest: &AssetManifest) -> Option<GemsAudioEngine> {
    let sound_dir = gems_sound_dir(manifest);
    let driver = std::fs::read(sound_dir.join("GEMS_Sounddriver.bin"))
        .map_err(|e| eprintln!("GEMS: failed to read driver: {e}"))
        .ok()?;
    let patches = std::fs::read(sound_dir.join("instruments.bin"))
        .map_err(|e| eprintln!("GEMS: failed to read patches: {e}"))
        .ok()?;
    let envelopes = std::fs::read(sound_dir.join("envelopes.bin"))
        .map_err(|e| eprintln!("GEMS: failed to read envelopes: {e}"))
        .ok()?;
    let sequences = std::fs::read(sound_dir.join("sequences.bin"))
        .map_err(|e| eprintln!("GEMS: failed to read sequences: {e}"))
        .ok()?;
    let samples = std::fs::read(sound_dir.join("samples.bin"))
        .map_err(|e| eprintln!("GEMS: failed to read samples: {e}"))
        .ok()?;

    let mut engine = GemsAudioEngine::new(&driver, &patches, &envelopes, &sequences, &samples);
    engine.init();
    Some(engine)
}

fn gems_sound_dir(manifest: &AssetManifest) -> PathBuf {
    manifest.output_root.join("sound")
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct AudioSampleClock {
    sample_rate: u32,
    remainder: u32,
}

impl AudioSampleClock {
    fn new(sample_rate: u32) -> Self {
        Self {
            sample_rate: sample_rate.max(1),
            remainder: 0,
        }
    }

    fn next_frame_samples(&mut self) -> usize {
        let total = self.remainder + self.sample_rate;
        let samples = total / AUDIO_FRAME_RATE_HZ;
        self.remainder = total % AUDIO_FRAME_RATE_HZ;
        usize::try_from(samples).unwrap_or(usize::MAX)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct NativeAudioGenerationPlan {
    sample_rate: u32,
    samples_per_frame: usize,
}

fn native_audio_generation_plan(
    sdl_queue_present: bool,
    audio_output_sample_rate: u32,
    audio_sample_clock: &mut AudioSampleClock,
) -> NativeAudioGenerationPlan {
    if sdl_queue_present {
        NativeAudioGenerationPlan {
            sample_rate: audio_output_sample_rate.max(1),
            samples_per_frame: audio_sample_clock.next_frame_samples(),
        }
    } else {
        NativeAudioGenerationPlan {
            sample_rate: AUDIO_SAMPLE_RATE,
            samples_per_frame: AUDIO_SAMPLES_PER_FRAME,
        }
    }
}

fn should_queue_audio_frame(queued_bytes: u32, generated_sample_count: usize) -> bool {
    let queued_bytes = usize::try_from(queued_bytes).unwrap_or(usize::MAX);
    // SDL_GetQueuedAudioSize reports bytes, while `generate_samples` returns
    // interleaved f32 samples. Keep only a small live queue so gameplay SFX
    // detected from the current trace frame are not heard multiple frames late.
    // Two generated frames still cover SDL's 512-sample callback without the
    // three-frame SFX latency that made pickup/block sounds feel detached.
    let target_bytes = live_audio_queue_target_bytes(generated_sample_count);
    queued_bytes < target_bytes
}

fn queue_live_audio_backlog(
    audio_queue: &AudioQueue<f32>,
    backlog: &mut LiveAudioBacklog,
    diagnostics: &mut NativeAudioDiagnostics,
    audio_queue_priming: &mut bool,
) {
    while let Some(sample_count) = backlog.front_sample_count() {
        let queued_bytes = audio_queue.size();
        if !should_queue_audio_frame(queued_bytes, sample_count) {
            diagnostics.record_queue_observation(queued_bytes);
            break;
        }
        let Some(samples) = backlog.pop_front() else {
            break;
        };
        if let Ok(()) = audio_queue.queue_audio(&samples) {
            diagnostics.record_queued_frame(samples.len(), queued_bytes);
            if *audio_queue_priming
                && should_resume_audio_queue_after_queue(queued_bytes, samples.len())
            {
                audio_queue.resume();
                *audio_queue_priming = false;
                diagnostics.record_queue_resume();
            }
        } else {
            backlog.push_front(samples);
            diagnostics.record_queue_error(queued_bytes);
            break;
        }
    }
    diagnostics.record_backlog_depth(backlog.pending_frames());
}

fn should_resume_audio_queue_after_queue(
    queued_bytes_before_queue: u32,
    generated_sample_count: usize,
) -> bool {
    let queued_bytes_before_queue =
        usize::try_from(queued_bytes_before_queue).unwrap_or(usize::MAX);
    let queued_bytes_after_queue = queued_bytes_before_queue
        .saturating_add(generated_audio_sample_bytes(generated_sample_count));
    queued_bytes_after_queue >= live_audio_queue_target_bytes(generated_sample_count)
}

fn live_audio_queue_target_bytes(generated_sample_count: usize) -> usize {
    generated_audio_sample_bytes(generated_sample_count).saturating_mul(AUDIO_QUEUE_TARGET_FRAMES)
}

fn generated_audio_sample_bytes(generated_sample_count: usize) -> usize {
    generated_sample_count.saturating_mul(std::mem::size_of::<f32>())
}

#[derive(Debug, Clone, PartialEq)]
struct LiveAudioBacklog {
    frames: VecDeque<Vec<f32>>,
    max_frames: usize,
}

impl LiveAudioBacklog {
    fn new(max_frames: usize) -> Self {
        Self {
            frames: VecDeque::new(),
            max_frames,
        }
    }

    fn pending_frames(&self) -> usize {
        self.frames.len()
    }

    fn front_sample_count(&self) -> Option<usize> {
        self.frames.front().map(Vec::len)
    }

    fn push_frame(&mut self, samples: Vec<f32>) -> bool {
        if self.frames.len() >= self.max_frames {
            return false;
        }
        self.frames.push_back(samples);
        true
    }

    fn push_front(&mut self, samples: Vec<f32>) {
        self.frames.push_front(samples);
    }

    fn pop_front(&mut self) -> Option<Vec<f32>> {
        self.frames.pop_front()
    }

    fn clear(&mut self) {
        self.frames.clear();
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct NativeAudioDiagnostics {
    gems_loaded: bool,
    sdl_queue_present: bool,
    audio_channels: u8,
    output_sample_rate: u32,
    generated_frames: usize,
    generated_samples: usize,
    queued_frames: usize,
    queued_samples: usize,
    skipped_full_frames: usize,
    backlog_frames: usize,
    backlog_full_frames: usize,
    max_backlog_frames: usize,
    queue_errors: usize,
    queue_prime_events: usize,
    queue_resume_events: usize,
    queue_clear_events: usize,
    queue_observations: usize,
    min_queue_bytes_before_queue: Option<u32>,
    max_queue_bytes_before_queue: Option<u32>,
    frame_timing_observations: usize,
    max_frame_micros: u128,
    over_budget_frames: usize,
    total_over_budget_micros: u128,
}

impl NativeAudioDiagnostics {
    fn new(
        gems_loaded: bool,
        sdl_queue_present: bool,
        audio_channels: u8,
        output_sample_rate: u32,
    ) -> Self {
        Self {
            gems_loaded,
            sdl_queue_present,
            audio_channels,
            output_sample_rate,
            generated_frames: 0,
            generated_samples: 0,
            queued_frames: 0,
            queued_samples: 0,
            skipped_full_frames: 0,
            backlog_frames: 0,
            backlog_full_frames: 0,
            max_backlog_frames: 0,
            queue_errors: 0,
            queue_prime_events: 0,
            queue_resume_events: 0,
            queue_clear_events: 0,
            queue_observations: 0,
            min_queue_bytes_before_queue: None,
            max_queue_bytes_before_queue: None,
            frame_timing_observations: 0,
            max_frame_micros: 0,
            over_budget_frames: 0,
            total_over_budget_micros: 0,
        }
    }

    fn record_generated_frame(&mut self, sample_count: usize) {
        self.generated_frames += 1;
        self.generated_samples += sample_count;
    }

    fn record_queue_observation(&mut self, queued_bytes: u32) {
        self.queue_observations += 1;
        self.min_queue_bytes_before_queue = Some(
            self.min_queue_bytes_before_queue
                .map_or(queued_bytes, |current| current.min(queued_bytes)),
        );
        self.max_queue_bytes_before_queue = Some(
            self.max_queue_bytes_before_queue
                .map_or(queued_bytes, |current| current.max(queued_bytes)),
        );
    }

    fn record_queued_frame(&mut self, sample_count: usize, queued_bytes: u32) {
        self.record_queue_observation(queued_bytes);
        self.queued_frames += 1;
        self.queued_samples += sample_count;
    }

    fn record_backlog_frame(&mut self, backlog_frames: usize) {
        self.backlog_frames += 1;
        self.record_backlog_depth(backlog_frames);
    }

    fn record_backlog_full_frame(&mut self, backlog_frames: usize) {
        self.backlog_full_frames += 1;
        self.record_backlog_depth(backlog_frames);
    }

    fn record_backlog_depth(&mut self, backlog_frames: usize) {
        self.max_backlog_frames = self.max_backlog_frames.max(backlog_frames);
    }

    fn record_queue_error(&mut self, queued_bytes: u32) {
        self.record_queue_observation(queued_bytes);
        self.queue_errors += 1;
    }

    fn record_queue_prime(&mut self) {
        self.queue_prime_events += 1;
    }

    fn record_queue_resume(&mut self) {
        self.queue_resume_events += 1;
    }

    fn record_queue_clear(&mut self) {
        self.queue_clear_events += 1;
    }

    fn record_frame_timing(&mut self, elapsed: Duration, budget: Duration) {
        self.frame_timing_observations += 1;
        self.max_frame_micros = self.max_frame_micros.max(elapsed.as_micros());
        if elapsed > budget {
            self.over_budget_frames += 1;
            self.total_over_budget_micros += elapsed.saturating_sub(budget).as_micros();
        }
    }

    fn into_lines(self) -> Vec<String> {
        vec![
            format!("gems_loaded={}", self.gems_loaded),
            format!("sdl_queue_present={}", self.sdl_queue_present),
            format!("audio_channels={}", self.audio_channels),
            format!("output_sample_rate={}", self.output_sample_rate),
            format!("generated_frames={}", self.generated_frames),
            format!("generated_samples={}", self.generated_samples),
            format!("queued_frames={}", self.queued_frames),
            format!("queued_samples={}", self.queued_samples),
            format!("skipped_full_frames={}", self.skipped_full_frames),
            format!("backlog_frames={}", self.backlog_frames),
            format!("backlog_full_frames={}", self.backlog_full_frames),
            format!("max_backlog_frames={}", self.max_backlog_frames),
            format!("queue_errors={}", self.queue_errors),
            format!("queue_prime_events={}", self.queue_prime_events),
            format!("queue_resume_events={}", self.queue_resume_events),
            format!("queue_clear_events={}", self.queue_clear_events),
            format!("queue_observations={}", self.queue_observations),
            format!(
                "min_queue_bytes_before_queue={}",
                self.min_queue_bytes_before_queue
                    .map_or_else(|| String::from("none"), |value| value.to_string())
            ),
            format!(
                "max_queue_bytes_before_queue={}",
                self.max_queue_bytes_before_queue
                    .map_or_else(|| String::from("none"), |value| value.to_string())
            ),
            format!(
                "frame_timing_observations={}",
                self.frame_timing_observations
            ),
            format!("max_frame_micros={}", self.max_frame_micros),
            format!("over_budget_frames={}", self.over_budget_frames),
            format!("total_over_budget_micros={}", self.total_over_budget_micros),
        ]
    }
}

/// GEMS sound IDs for Kid Chameleon (from constants.asm).
#[allow(dead_code)]
mod gems_sfx {
    // General
    pub const SCORE_COUNTER: u8 = 0x0C;
    pub const BLOCK_HIT: u8 = 0x0D;
    pub const CLOCK_PRIZE: u8 = 0x0E;
    pub const DESTROY_BLOCK: u8 = 0x11;
    pub const METAL_BLOCK: u8 = 0x12;
    pub const REVEAL_HIDDEN: u8 = 0x12;
    pub const EVANESCENT: u8 = 0x14;
    pub const SHIFTING: u8 = 0x15;
    pub const JUMP: u8 = 0x17;
    pub const LAVA_GEYSER: u8 = 0x1E;
    pub const TANK_DRIVING: u8 = 0x1F;
    pub const ELEVATOR: u8 = 0x27;
    pub const DIAMOND_POWER_AVAILABLE: u8 = 0x28;
    pub const ANKH_PRIZE: u8 = 0x29;
    pub const REPLENISH_HEALTH: u8 = 0x2A;
    pub const COIN_PRIZE: u8 = 0x2C;
    pub const DIAMOND_PRIZE: u8 = 0x2D;
    pub const NAVIGATE_JINGLE: u8 = 0x2D;
    pub const TANK_MOUTH_OPEN: u8 = 0x3C;
    pub const MINI_HOPPING_SKULL_SCREAM: u8 = 0x3F;
    pub const JUMP_ON_ENEMY: u8 = 0x4C;
    pub const VOICE_NO_TIME: u8 = 0x51;
    pub const TAR_MONSTER_DISAPPEARS: u8 = 0x53;
    pub const TAR_MONSTER_APPEARS: u8 = 0x54;
    pub const RUBBER_BLOCK: u8 = 0x56;
    pub const DRIPS_DRIPPING: u8 = 0x58;
    pub const TAR_MONSTER_SHOOTS: u8 = 0x5E;
    pub const VOICE_DIE: u8 = 0x60;
    pub const VOICE_OUCH: u8 = 0x61;
    pub const VOICE_DIAMOND: u8 = 0x62;
    pub const KID_PULLUP: u8 = 0x64;
    pub const VOICE_BUMMER: u8 = 0x68;
    pub const VOICE_OUCH_1: u8 = 0x6B;
    pub const VOICE_OUCH_2: u8 = 0x6C;
    pub const VOICE_OUCH_3: u8 = 0x6D;
    pub const TELEPORT: u8 = 0x76;
    pub const THUNDERSTORM: u8 = 0x77;
    // Helmet transforms
    pub const IRON_KNIGHT_TRANSFORM: u8 = 0x1A;
    pub const MANIAXE_TRANSFORM: u8 = 0x1B;
    pub const RED_STEALTH_TRANSFORM: u8 = 0x2E;
    pub const BERZERKER_TRANSFORM: u8 = 0x2F;
    pub const SKYCUTTER_TRANSFORM: u8 = 0x30;
    pub const EYECLOPS_TRANSFORM: u8 = 0x31;
    pub const CYCLONE_TRANSFORM: u8 = 0x32;
    pub const MICROMAX_TRANSFORM: u8 = 0x33;
    pub const JUGGERNAUT_TRANSFORM: u8 = 0x34;
    // Helmet abilities
    pub const MANIAXE_THROW: u8 = 0x20;
    pub const RED_STEALTH_ATTACK_DOWN: u8 = 0x13;
    pub const RED_STEALTH_SWORD: u8 = 0x23;
    pub const RED_STEALTH_HITTING_TARGET: u8 = 0x5A;
    pub const EYECLOPS_BEAM: u8 = 0x35;
    pub const SKYCUTTER_FLIPBOARD: u8 = 0x17;
    pub const MICROMAX_STICK: u8 = 0x22;
}

/// GEMS BGM IDs (from constants.asm).
#[allow(dead_code)]
mod gems_bgm {
    pub const MOUNTAIN: u8 = 0;
    pub const DESERT: u8 = 1;
    pub const SKY: u8 = 2;
    pub const ICE: u8 = 3;
    pub const HILLS: u8 = 4;
    pub const ENDING: u8 = 5;
    pub const FOREST: u8 = 6;
    pub const CAVE: u8 = 7;
    pub const ISLAND: u8 = 8;
    pub const MURDERWALL: u8 = 9;
    pub const SWAMP: u8 = 0x0A;
    pub const CITY: u8 = 0x65;
    pub const SCORE_BOARD: u8 = 0x74;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum NativeBgmCue {
    TitleShell,
    Options,
    Level(LevelId),
    CoreQueued(u8),
    ScoreBoard,
    Ending,
    StoppedForTransition,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum NativeBgmPlaybackCommand {
    Start(u8),
    StopMusic,
    StopMusicThenStart(u8),
}

fn native_bgm_cue_for_trace(trace: &NativeTraceFrame) -> Option<NativeBgmCue> {
    match (trace.game_mode, trace.played_bgm_ids.last().copied()) {
        // ASM Score_Board loc_C238 (kid.asm:15812): core queues the exact
        // `bgm_Score_Board` command once the EOL path reaches the BGM frame.
        (GameMode::InGame | GameMode::DemoPlay, Some(gems_bgm::SCORE_BOARD))
            if trace.level_complete =>
        {
            return Some(NativeBgmCue::ScoreBoard);
        }
        // ASM End_Credits loc_D824 (kid.asm:16924): core queues
        // `bgm_Ending` after the `$28` StopMusic pause.
        (GameMode::EndingSequence, Some(gems_bgm::ENDING)) => {
            return Some(NativeBgmCue::Ending);
        }
        // ASM Load_InGame loc_11FD2/loc_12004 is now emitted by wildside-core into
        // the trace. Prefer that exact command over re-deriving level music
        // from manifest headers, while keeping the scoreboard override below.
        (GameMode::InGame | GameMode::DemoPlay, Some(bgm)) if !trace.level_complete => {
            return Some(NativeBgmCue::CoreQueued(bgm));
        }
        _ => {}
    }

    match trace.game_mode {
        // ASM Load_IntroSequence1 / Load_TitleScreen both queue `bgm_Ice` via
        // `j_PlaySound`; later intro/title-shell modes inherit that cue. Native
        // may enter those modes directly through manual Start/demo paths, so
        // keep them recoverable instead of requiring the cold first sequence.
        GameMode::IntroSequence1
        | GameMode::IntroSequence2
        | GameMode::IntroSequence3
        | GameMode::IntroSequence4
        | GameMode::IntroSequence5
        | GameMode::IntroSequence6
        | GameMode::TitleScreen => Some(NativeBgmCue::TitleShell),
        // ASM Load_OptionMenu (kid.asm:31570-31572) stops the active shell
        // music, then queues bgm_City before drawing the options screen.
        GameMode::OptionsMenu => Some(NativeBgmCue::Options),
        // ASM Score_Board (kid.asm:15838): after the flagpole transition
        // sleeps, the EOL sequence queues bgm_Score_Board before loc_D55E.
        GameMode::InGame | GameMode::DemoPlay if trace.level_complete => {
            Some(NativeBgmCue::ScoreBoard)
        }
        GameMode::InGame | GameMode::DemoPlay => Some(NativeBgmCue::Level(trace.current_level)),
        _ => None,
    }
}

fn native_bgm_state_after_trace(
    manifest: &AssetManifest,
    trace: &NativeTraceFrame,
    active_bgm_cue: Option<NativeBgmCue>,
) -> (Option<NativeBgmCue>, Option<NativeBgmPlaybackCommand>) {
    let Some(cue) = native_bgm_cue_for_trace(trace) else {
        if active_bgm_cue == Some(NativeBgmCue::StoppedForTransition) {
            return (active_bgm_cue, None);
        }
        if trace.game_mode == GameMode::SegaScreen && active_bgm_cue.is_some() {
            // ASM Load_SegaScreen (kid.asm:25444-25445) runs StopMusic before
            // loading the Sega logo, so the following Load_IntroSequence1
            // must be able to queue bgm_Ice again.
            return (None, Some(NativeBgmPlaybackCommand::StopMusic));
        }
        if matches!(
            active_bgm_cue,
            Some(
                NativeBgmCue::TitleShell
                    | NativeBgmCue::Level(_)
                    | NativeBgmCue::CoreQueued(_)
                    | NativeBgmCue::ScoreBoard
            )
        ) && trace.game_mode == GameMode::LevelTransition
        {
            // ASM loc_6E2 dispatches mode 8 through LoadGameModeData; the
            // subsequent Load_InGame path (kid.asm:23623-23629) resets GEMS
            // with StopMusic before loc_12004 queues the level BGM again.
            return (
                Some(NativeBgmCue::StoppedForTransition),
                Some(NativeBgmPlaybackCommand::StopMusic),
            );
        }
        if active_bgm_cue.is_none()
            && trace.game_mode == GameMode::LevelTransition
            && (trace.level_complete
                || trace.level_complete_elapsed_frames > 0
                || trace.level_complete_score_counter_drained)
        {
            // If the active cue tracker has already been cleared by an earlier
            // shell frame, the GEMS driver can still be releasing the final
            // loc_D7F4 score-counter SFX. StopMusic is harmless with no BGM and
            // mirrors the ROM hard reset before Load_InGame.
            return (
                Some(NativeBgmCue::StoppedForTransition),
                Some(NativeBgmPlaybackCommand::StopMusic),
            );
        }
        return (active_bgm_cue, None);
    };
    if Some(cue) == active_bgm_cue {
        return (active_bgm_cue, None);
    }
    let Some(bgm) = bgm_for_native_cue(manifest, cue) else {
        return (active_bgm_cue, None);
    };
    let command = if active_bgm_cue.is_none()
        && matches!(cue, NativeBgmCue::Level(_) | NativeBgmCue::CoreQueued(_))
    {
        // ASM Load_InGame (kid.asm:23623) runs StopMusic even when
        // native's cue tracker has no previous BGM. This hard reset is the
        // incoming-level off switch for stale looping GEMS sequences (#339).
        NativeBgmPlaybackCommand::StopMusicThenStart(bgm)
    } else if active_bgm_cue.is_some_and(|active| active != NativeBgmCue::StoppedForTransition)
        || (cue == NativeBgmCue::TitleShell && trace.game_mode == GameMode::IntroSequence1)
    {
        NativeBgmPlaybackCommand::StopMusicThenStart(bgm)
    } else {
        NativeBgmPlaybackCommand::Start(bgm)
    };
    (Some(cue), Some(command))
}

pub(crate) fn native_audio_events_for_trace(
    manifest: &AssetManifest,
    trace_frames: &[NativeTraceFrame],
) -> Vec<String> {
    native_audio_events_for_trace_with_initial_bgm(manifest, trace_frames, None)
}

fn native_audio_events_for_trace_with_initial_bgm(
    manifest: &AssetManifest,
    trace_frames: &[NativeTraceFrame],
    initial_active_bgm_cue: Option<NativeBgmCue>,
) -> Vec<String> {
    let mut active_bgm_cue = initial_active_bgm_cue;
    let mut events = Vec::new();
    let mut prev_trace: Option<&NativeTraceFrame> = None;
    for (frame_index, trace) in trace_frames.iter().enumerate() {
        let active_before = active_bgm_cue;
        let (next_bgm_cue, bgm_command) =
            native_bgm_state_after_trace(manifest, trace, active_bgm_cue);
        if let Some(command) = bgm_command {
            events.push(format_native_bgm_event(
                frame_index,
                trace,
                active_before,
                next_bgm_cue,
                command,
            ));
        }
        active_bgm_cue = next_bgm_cue;
        if let Some(prev) = prev_trace {
            let suppress_sfx_this_frame = bgm_command.is_some_and(|command| {
                native_bgm_command_suppresses_same_frame_sfx(command, trace, active_before)
            });
            if !suppress_sfx_this_frame {
                for command in gems_event_audio_commands(prev, trace) {
                    match command {
                        NativeAudioTraceCommand::Sound(command) => {
                            events.push(format_native_sound_event(frame_index, trace, command));
                        }
                        NativeAudioTraceCommand::MusicSpeed(speed) => {
                            events.push(format_native_music_speed_event(frame_index, trace, speed));
                        }
                    }
                }
            }
        }
        prev_trace = Some(trace);
    }
    events
}

pub(crate) fn native_audio_artifacts_for_trace(
    manifest: &AssetManifest,
    trace_frames: &[NativeTraceFrame],
) -> RecordingBundleArtifacts {
    native_audio_artifacts_for_trace_with_initial_bgm(manifest, trace_frames, None)
}

pub(crate) fn native_audio_artifacts_for_settled_trace(
    manifest: &AssetManifest,
    trace_frames: &[NativeTraceFrame],
) -> RecordingBundleArtifacts {
    let initial_active_bgm_cue = trace_frames.first().and_then(native_bgm_cue_for_trace);
    native_audio_artifacts_for_trace_with_initial_bgm(
        manifest,
        trace_frames,
        initial_active_bgm_cue,
    )
}

fn native_audio_artifacts_for_trace_with_initial_bgm(
    manifest: &AssetManifest,
    trace_frames: &[NativeTraceFrame],
    initial_active_bgm_cue: Option<NativeBgmCue>,
) -> RecordingBundleArtifacts {
    let audio_events = native_audio_events_for_trace_with_initial_bgm(
        manifest,
        trace_frames,
        initial_active_bgm_cue,
    );
    let Some(mut gems) = try_load_gems_engine(manifest) else {
        return RecordingBundleArtifacts {
            audio_events,
            ..RecordingBundleArtifacts::default()
        };
    };

    let audio_samples = render_native_audio_samples_for_trace(
        manifest,
        trace_frames,
        initial_active_bgm_cue,
        &mut gems,
    );

    RecordingBundleArtifacts {
        audio_samples: Some(audio_samples),
        audio_channels: 2,
        audio_events,
        ..RecordingBundleArtifacts::default()
    }
}

fn render_native_audio_samples_for_trace(
    manifest: &AssetManifest,
    trace_frames: &[NativeTraceFrame],
    initial_active_bgm_cue: Option<NativeBgmCue>,
    audio: &mut impl NativeAudioSink,
) -> Vec<f32> {
    if let Some(bgm) = settled_audio_initial_bgm_id(manifest, initial_active_bgm_cue) {
        // Settled dynamic audio probes start after the ROM has already loaded
        // level music. Suppress duplicate event-log commands, but prime the
        // standalone GEMS renderer so block/pickup SFX are mixed over the same
        // active BGM instead of silence until the first SFX.
        audio.start_sequence(bgm);
    }
    let mut active_bgm_cue = initial_active_bgm_cue;
    let mut pending_bgm_start: Option<u8> = None;
    let mut prev_trace: Option<&NativeTraceFrame> = None;
    let mut audio_samples = Vec::with_capacity(trace_frames.len() * AUDIO_SAMPLES_PER_FRAME * 2);
    for trace in trace_frames {
        let (next_bgm_cue, bgm_command) =
            native_bgm_state_after_trace(manifest, trace, active_bgm_cue);
        let mut queued_bgm_start_this_frame = false;
        let mut suppress_sfx_this_frame = false;
        if let Some(command) = bgm_command {
            pending_bgm_start = apply_native_bgm_command(audio, command);
            queued_bgm_start_this_frame = pending_bgm_start.is_some();
            suppress_sfx_this_frame =
                native_bgm_command_suppresses_same_frame_sfx(command, trace, active_bgm_cue);
        }
        active_bgm_cue = next_bgm_cue;

        if !queued_bgm_start_this_frame && let Some(bgm) = pending_bgm_start.take() {
            audio.start_sequence(bgm);
        }
        if !suppress_sfx_this_frame && let Some(prev) = prev_trace {
            for command in gems_event_audio_commands(prev, trace) {
                match command {
                    NativeAudioTraceCommand::Sound(command) => {
                        apply_native_sound_command(audio, command);
                    }
                    NativeAudioTraceCommand::MusicSpeed(speed) => {
                        audio.set_music_speed(speed);
                    }
                }
            }
        }
        audio.step_frame();
        audio_samples.extend(audio.generate_samples(AUDIO_SAMPLES_PER_FRAME));
        prev_trace = Some(trace);
    }

    audio_samples
}

fn settled_audio_initial_bgm_id(
    manifest: &AssetManifest,
    initial_active_bgm_cue: Option<NativeBgmCue>,
) -> Option<u8> {
    initial_active_bgm_cue.and_then(|cue| bgm_for_native_cue(manifest, cue))
}

trait NativeAudioSink {
    fn start_sequence(&mut self, sound_id: u8);
    fn stop_sequence(&mut self, sound_id: u8);
    fn stop_music(&mut self);
    fn set_music_speed(&mut self, speed: u8);
    fn step_frame(&mut self);
    fn generate_samples(&mut self, sample_count: usize) -> Vec<f32>;
}

impl NativeAudioSink for GemsAudioEngine {
    fn start_sequence(&mut self, sound_id: u8) {
        GemsAudioEngine::start_sequence(self, sound_id);
    }

    fn stop_sequence(&mut self, sound_id: u8) {
        GemsAudioEngine::stop_sequence(self, sound_id);
    }

    fn stop_music(&mut self) {
        GemsAudioEngine::stop_music(self);
    }

    fn set_music_speed(&mut self, speed: u8) {
        GemsAudioEngine::set_music_speed(self, speed);
    }

    fn step_frame(&mut self) {
        GemsAudioEngine::step_frame(self);
    }

    fn generate_samples(&mut self, sample_count: usize) -> Vec<f32> {
        GemsAudioEngine::generate_samples(self, sample_count)
    }
}

fn apply_native_bgm_command(
    audio: &mut impl NativeAudioSink,
    command: NativeBgmPlaybackCommand,
) -> Option<u8> {
    match command {
        NativeBgmPlaybackCommand::Start(bgm) => {
            audio.start_sequence(bgm);
            None
        }
        NativeBgmPlaybackCommand::StopMusic => {
            audio.stop_music();
            None
        }
        NativeBgmPlaybackCommand::StopMusicThenStart(bgm) => {
            // ASM StopMusic (kid.asm:49727) reloads/resets the GEMS driver
            // and queues command `$0B` with the sound data pointers. Native
            // playback must use that ROM path before changing music contexts;
            // `$16` stop-all leaves the standalone driver with stale title
            // sequence state and can starve gameplay SFX channels.
            audio.stop_music();
            Some(bgm)
        }
    }
}

fn native_bgm_command_flushes_queued_audio(command: NativeBgmPlaybackCommand) -> bool {
    // ASM StopMusic (kid.asm:49727) resets the Z80/GEMS driver immediately.
    // play-native queues audio ahead of playback through SDL, so mirror that
    // hard reset by dropping already-generated samples from the host queue.
    matches!(
        command,
        NativeBgmPlaybackCommand::StopMusic | NativeBgmPlaybackCommand::StopMusicThenStart(_)
    )
}

fn native_bgm_command_suppresses_same_frame_sfx(
    command: NativeBgmPlaybackCommand,
    trace: &NativeTraceFrame,
    active_before: Option<NativeBgmCue>,
) -> bool {
    if !native_bgm_command_flushes_queued_audio(command) {
        return false;
    }
    // ASM loc_D7F4 (kid.asm:16906) replays sfx_Score_counter before Teleport,
    // then Load_InGame (kid.asm:23623-23629) runs StopMusic before the next
    // level's BGM. Native condenses that handoff into one LevelTransition trace
    // frame, so any same-frame SFX must be treated as pre-reset driver state.
    if trace.game_mode == GameMode::LevelTransition {
        return true;
    }
    // ASM Load_InGame runs j_StopMusic before loc_11FD2/loc_12004
    // queues level music. A cold native cue means this frame is also the first
    // visible level-load boundary, so stale enemy SFX deltas from the previous
    // route must not be replayed after the reset.
    matches!(command, NativeBgmPlaybackCommand::StopMusicThenStart(_))
        && active_before.is_none()
        && matches!(trace.game_mode, GameMode::InGame | GameMode::DemoPlay)
}

fn apply_native_sound_command(audio: &mut impl NativeAudioSink, command: NativeSoundCommand) {
    match command.command_byte {
        NativeSoundCommand::STOP_SEQUENCE_COMMAND_BYTE => audio.stop_sequence(command.sound_id),
        _ => audio.start_sequence(command.sound_id),
    }
}

fn format_native_bgm_event(
    frame_index: usize,
    trace: &NativeTraceFrame,
    active_before: Option<NativeBgmCue>,
    active_after: Option<NativeBgmCue>,
    command: NativeBgmPlaybackCommand,
) -> String {
    format!(
        "frame={frame_index} mode={:?} level={:02X} active_before={active_before:?} active_after={active_after:?} command={command:?}",
        trace.game_mode, trace.current_level.0
    )
}

fn format_native_sound_event(
    frame_index: usize,
    trace: &NativeTraceFrame,
    command: NativeSoundCommand,
) -> String {
    let command_name = command.command_name();
    let sound_id = command.sound_id;
    format!(
        "frame={frame_index} mode={:?} level={:02X} command={command_name}({sound_id})",
        trace.game_mode, trace.current_level.0
    )
}

fn format_native_music_speed_event(
    frame_index: usize,
    trace: &NativeTraceFrame,
    speed: u8,
) -> String {
    format!(
        "frame={frame_index} mode={:?} level={:02X} command=SetMusicSpeed({speed})",
        trace.game_mode, trace.current_level.0
    )
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum NativeAudioTraceCommand {
    Sound(NativeSoundCommand),
    MusicSpeed(u8),
}

fn gems_event_audio_commands(
    prev: &NativeTraceFrame,
    curr: &NativeTraceFrame,
) -> Vec<NativeAudioTraceCommand> {
    let sound_commands = gems_event_sound_commands(prev, curr);
    let music_speed_updates = gems_event_music_speed_updates(prev, curr);
    let (voice_bummer_commands, pre_speed_sound_commands): (Vec<_>, Vec<_>) = sound_commands
        .into_iter()
        .partition(|command| command.sound_id == gems_sfx::VOICE_BUMMER);

    pre_speed_sound_commands
        .into_iter()
        .map(NativeAudioTraceCommand::Sound)
        .chain(
            music_speed_updates
                .into_iter()
                .map(NativeAudioTraceCommand::MusicSpeed),
        )
        .chain(
            voice_bummer_commands
                .into_iter()
                .map(NativeAudioTraceCommand::Sound),
        )
        .collect()
}

fn bgm_for_native_cue(manifest: &AssetManifest, cue: NativeBgmCue) -> Option<u8> {
    match cue {
        // ASM Load_TitleScreen queues bgm_Ice through j_PlaySound before loading title art.
        NativeBgmCue::TitleShell => Some(gems_bgm::ICE),
        NativeBgmCue::Options => Some(gems_bgm::CITY),
        NativeBgmCue::Level(level) => {
            resolved_level_header(manifest, level.0).and_then(level_start_bgm)
        }
        NativeBgmCue::CoreQueued(bgm) => Some(bgm),
        NativeBgmCue::ScoreBoard => Some(gems_bgm::SCORE_BOARD),
        NativeBgmCue::Ending => Some(gems_bgm::ENDING),
        NativeBgmCue::StoppedForTransition => None,
    }
}

fn queued_gems_sound_ids_since<'a>(
    prev: &NativeTraceFrame,
    curr: &'a NativeTraceFrame,
) -> &'a [u8] {
    if curr.played_sound_effect_ids.len() >= prev.played_sound_effect_ids.len() {
        &curr.played_sound_effect_ids[prev.played_sound_effect_ids.len()..]
    } else {
        &curr.played_sound_effect_ids
    }
}

fn queued_gems_sound_commands_since(
    prev: &NativeTraceFrame,
    curr: &NativeTraceFrame,
) -> Vec<NativeSoundCommand> {
    if curr.played_sound_commands.is_empty() {
        return queued_gems_sound_ids_since(prev, curr)
            .iter()
            .copied()
            .map(NativeSoundCommand::start_sequence)
            .collect();
    }

    if curr.played_sound_commands.len() >= prev.played_sound_commands.len() {
        curr.played_sound_commands[prev.played_sound_commands.len()..].to_vec()
    } else {
        curr.played_sound_commands.clone()
    }
}

fn queued_gems_music_speed_updates_since<'a>(
    prev: &NativeTraceFrame,
    curr: &'a NativeTraceFrame,
) -> &'a [u8] {
    if curr.played_music_speed_updates.len() >= prev.played_music_speed_updates.len() {
        &curr.played_music_speed_updates[prev.played_music_speed_updates.len()..]
    } else {
        &curr.played_music_speed_updates
    }
}

fn gems_event_music_speed_updates(prev: &NativeTraceFrame, curr: &NativeTraceFrame) -> Vec<u8> {
    queued_gems_music_speed_updates_since(prev, curr).to_vec()
}

fn is_ouch_sound_id(sound_id: u8) -> bool {
    matches!(
        sound_id,
        gems_sfx::VOICE_OUCH
            | gems_sfx::VOICE_OUCH_1
            | gems_sfx::VOICE_OUCH_2
            | gems_sfx::VOICE_OUCH_3
    )
}

fn is_damage_resolution_voice_sound_id(sound_id: u8) -> bool {
    // ASM loc_B73C/Death (kid.asm:14661,14714): once the core trace carries
    // either the selected ouch take or the direct death voice, the legacy
    // HP-drop fallback must stay silent.
    is_ouch_sound_id(sound_id) || sound_id == gems_sfx::VOICE_DIE
}

fn is_explicit_block_sound_id(sound_id: u8) -> bool {
    matches!(
        sound_id,
        gems_sfx::DESTROY_BLOCK | gems_sfx::METAL_BLOCK | gems_sfx::RUBBER_BLOCK
    )
}

fn is_transform_sound_id(sound_id: u8) -> bool {
    matches!(
        sound_id,
        gems_sfx::IRON_KNIGHT_TRANSFORM
            | gems_sfx::MANIAXE_TRANSFORM
            | gems_sfx::RED_STEALTH_TRANSFORM
            | gems_sfx::BERZERKER_TRANSFORM
            | gems_sfx::SKYCUTTER_TRANSFORM
            | gems_sfx::EYECLOPS_TRANSFORM
            | gems_sfx::CYCLONE_TRANSFORM
            | gems_sfx::MICROMAX_TRANSFORM
            | gems_sfx::JUGGERNAUT_TRANSFORM
    )
}

fn transform_sound_for_helmet_id_loc_7cd0(helmet_id: u16) -> Option<u8> {
    // ASM loc_7CD0 (kid.asm:8426): `unk_7ED8[Current_Helmet]` maps helmet ids
    // to transform SFX. Keep this fallback for older traces; current core
    // frames queue the same id explicitly.
    match helmet_id {
        1 => Some(gems_sfx::SKYCUTTER_TRANSFORM),
        2 => Some(gems_sfx::CYCLONE_TRANSFORM),
        3 => Some(gems_sfx::RED_STEALTH_TRANSFORM),
        4 => Some(gems_sfx::EYECLOPS_TRANSFORM),
        5 => Some(gems_sfx::JUGGERNAUT_TRANSFORM),
        6 => Some(gems_sfx::IRON_KNIGHT_TRANSFORM),
        7 => Some(gems_sfx::BERZERKER_TRANSFORM),
        8 => Some(gems_sfx::MANIAXE_TRANSFORM),
        9 => Some(gems_sfx::MICROMAX_TRANSFORM),
        _ => None,
    }
}

fn play_fallback_transform_sfx(sfx: &mut SfxEngine) {
    sfx.play(330.0, 90, 0.16);
    sfx.play(494.0, 110, 0.17);
    sfx.play(740.0, 150, 0.18);
}

fn play_fallback_jump_on_enemy_sfx(sfx: &mut SfxEngine) {
    // ASM Jump_On_Enemy loc_B6DA queues sfx_Jump_on_enemy=$4C. In fallback
    // mode keep it brighter and shorter than the ordinary jump cue so stomps
    // do not read as a slow rubber thud.
    sfx.play(784.0, 34, 0.13);
    sfx.play(1175.0, 48, 0.11);
}

fn play_fallback_rubber_block_sfx(sfx: &mut SfxEngine) {
    // ASM loc_10DA4 queues sfx_Rubber_block=$56 before the skin-swap worker.
    // The no-GEMS fallback is intentionally elastic rather than bass-heavy;
    // the previous 294/196 Hz pair sounded like a slowed generic thud.
    sfx.play(392.0, 36, 0.10);
    sfx.play(784.0, 58, 0.12);
    sfx.play(523.0, 82, 0.08);
}

fn play_fallback_queued_sfx_id(sfx: &mut SfxEngine, sound_id: u8) {
    match sound_id {
        gems_sfx::SCORE_COUNTER => {
            sfx.play(784.0, 45, 0.1);
            sfx.play(1047.0, 55, 0.1);
        }
        gems_sfx::BLOCK_HIT => {
            sfx.play(1100.0, 30, 0.08);
            sfx.play(550.0, 40, 0.06);
        }
        gems_sfx::CLOCK_PRIZE => {
            sfx.play(698.0, 60, 0.12);
            sfx.play(880.0, 80, 0.12);
        }
        gems_sfx::DESTROY_BLOCK => {
            sfx.play(700.0, 35, 0.08);
            sfx.play(350.0, 50, 0.07);
        }
        gems_sfx::METAL_BLOCK => {
            sfx.play(740.0, 45, 0.09);
            sfx.play(370.0, 55, 0.07);
        }
        gems_sfx::EVANESCENT | gems_sfx::SHIFTING | gems_sfx::ELEVATOR => {
            sfx.play(440.0, 45, 0.08);
            sfx.play(660.0, 65, 0.08);
        }
        gems_sfx::JUMP => sfx.play(440.0, 40, 0.1),
        gems_sfx::DIAMOND_POWER_AVAILABLE => {
            sfx.play(523.0, 60, 0.18);
            sfx.play(784.0, 60, 0.18);
            sfx.play(1047.0, 120, 0.2);
        }
        gems_sfx::ANKH_PRIZE
        | gems_sfx::REPLENISH_HEALTH
        | gems_sfx::COIN_PRIZE
        | gems_sfx::DIAMOND_PRIZE => sfx.play(880.0, 60, 0.15),
        gems_sfx::JUMP_ON_ENEMY => play_fallback_jump_on_enemy_sfx(sfx),
        gems_sfx::VOICE_NO_TIME
        | gems_sfx::VOICE_DIE
        | gems_sfx::VOICE_OUCH
        | gems_sfx::VOICE_BUMMER
        | gems_sfx::VOICE_OUCH_1
        | gems_sfx::VOICE_OUCH_2
        | gems_sfx::VOICE_OUCH_3 => sfx.play(220.0, 150, 0.2),
        gems_sfx::VOICE_DIAMOND => sfx.play(1047.0, 90, 0.14),
        gems_sfx::KID_PULLUP => sfx.play(392.0, 45, 0.09),
        gems_sfx::RUBBER_BLOCK => play_fallback_rubber_block_sfx(sfx),
        gems_sfx::DRIPS_DRIPPING => sfx.play(196.0, 90, 0.05),
        gems_sfx::TELEPORT => {
            sfx.play(330.0, 80, 0.1);
            sfx.play(440.0, 80, 0.1);
            sfx.play(550.0, 120, 0.1);
        }
        gems_sfx::RED_STEALTH_ATTACK_DOWN => {
            sfx.play(988.0, 55, 0.18);
            sfx.play(494.0, 90, 0.16);
        }
        gems_sfx::RED_STEALTH_SWORD => {
            sfx.play(1320.0, 35, 0.17);
            sfx.play(880.0, 55, 0.14);
        }
        gems_sfx::RED_STEALTH_HITTING_TARGET => {
            sfx.play(1568.0, 45, 0.22);
            sfx.play(784.0, 95, 0.18);
        }
        gems_sfx::MANIAXE_THROW | gems_sfx::MICROMAX_STICK | gems_sfx::EYECLOPS_BEAM => {
            sfx.play(659.0, 45, 0.09);
        }
        id if is_transform_sound_id(id) => play_fallback_transform_sfx(sfx),
        _ => {}
    }
}

#[cfg(test)]
fn gems_event_sound_ids(prev: &NativeTraceFrame, curr: &NativeTraceFrame) -> Vec<u8> {
    gems_event_sound_commands(prev, curr)
        .into_iter()
        .map(|command| command.sound_id)
        .collect()
}

fn gems_event_sound_commands(
    prev: &NativeTraceFrame,
    curr: &NativeTraceFrame,
) -> Vec<NativeSoundCommand> {
    let queued_sound_ids = queued_gems_sound_ids_since(prev, curr);
    let mut sound_commands = queued_gems_sound_commands_since(prev, curr);
    let gameplay_sfx_fallback_active =
        matches!(curr.game_mode, GameMode::InGame | GameMode::DemoPlay);

    // Sound test: play the selected track when A/C pressed
    if curr.sound_test_play_pending
        && let Some(track) = curr.sound_test_track
    {
        sound_commands.push(NativeSoundCommand::start_sequence(track));
    }

    // Jump
    if gameplay_sfx_fallback_active
        && !queued_sound_ids.contains(&gems_sfx::JUMP)
        && prev.kid_on_ground == Some(true)
        && curr.kid_on_ground == Some(false)
        && curr.kid_y_velocity.is_some_and(|v| v < 0)
    {
        sound_commands.push(NativeSoundCommand::start_sequence(gems_sfx::JUMP));
    }

    // Stomp enemy is now driven by the core trace. ASM Jump_On_Enemy
    // (kid.asm:14607) plays `sfx_Jump_on_enemy` before any enemy HP/score
    // consequence, and many real stomps do not award score.

    // Diamond collected: the cue is now driven authoritatively by the core
    // trace, not by the visible count. ASM PrizeDiamondCollected_Init
    // (kid.asm:21103) plays `sfx_Diamond_prize` ($2D) on the *contact* frame,
    // which the core queues via `queue_prize_diamond_award`. The visible
    // `diamond_count` only rises ~33 frames later, after the fly-to-counter
    // loop (kid.asm:21149-21162), so the previous `diamond_count > prev`
    // heuristic emitted the cue that late and is what #216 heard as the diamond
    // sound lagging. The contact cue arrives through `queued_sound_ids` above,
    // so no count-based fallback is needed.

    // Damage taken (HP decreased). ASM loc_B73C/loc_B750/loc_B764/loc_B778
    // can queue enemy-family-specific ouch takes; keep the heuristic only for
    // paths that have not yet promoted their exact core SFX into the trace.
    if !queued_sound_ids
        .iter()
        .copied()
        .any(is_damage_resolution_voice_sound_id)
        && curr.number_hitpoints < prev.number_hitpoints
        && prev.number_hitpoints > 0
    {
        sound_commands.push(NativeSoundCommand::start_sequence(gems_sfx::VOICE_OUCH));
    }

    // Death (lives decreased)
    if curr.number_lives < prev.number_lives {
        sound_commands.push(NativeSoundCommand::start_sequence(gems_sfx::VOICE_DIE));
    }

    // Form change (helmet ID changed). ASM loc_7CD0 skips the unk_7ED8 cue in
    // Demo_Mode_flag, otherwise indexes this table by Current_Helmet whose
    // order differs from the extracted art order.
    if curr.game_mode != GameMode::DemoPlay
        && curr.current_helmet_id != prev.current_helmet_id
        && let Some(id) = transform_sound_for_helmet_id_loc_7cd0(curr.current_helmet_id)
        && !queued_sound_ids.contains(&id)
    {
        sound_commands.push(NativeSoundCommand::start_sequence(id));
    }

    // Diamond Power activation
    if curr.diamond_power_active && !prev.diamond_power_active {
        sound_commands.push(NativeSoundCommand::start_sequence(
            gems_sfx::DIAMOND_POWER_AVAILABLE,
        ));
    }

    // ASM loc_D752/loc_D770: Score_Board BGM and EOL rows appear before the
    // `sfx_Score_counter` cue; only start it when the ROM counter drain begins.
    if curr.level_complete
        && curr.level_complete_score_counter_elapsed_frames > 0
        && prev.level_complete_score_counter_elapsed_frames == 0
    {
        sound_commands.push(NativeSoundCommand::start_sequence(gems_sfx::SCORE_COUNTER));
    }

    // ASM loc_D7F4 (kid.asm:16906-16908): when the counter finishes draining,
    // the ROM issues GEMS `$12` (cmdstopseq, the call labelled `j_PlaySound2`
    // in the disassembly) to *stop* sequence `$0C` (`sfx_Score_counter`) before
    // hibernating and routing to Teleport/End_Credits. loc_D752 starts the tick
    // with `$10` (cmdstartseq); loc_D7F4 stops it with `$12`, cleanly silencing
    // the tick once the tally lands instead of leaving it ringing (#216
    // "level-complete sound does not finish cleanly").
    if curr.level_complete
        && curr.level_complete_score_counter_drained
        && !prev.level_complete_score_counter_drained
    {
        sound_commands.push(NativeSoundCommand::stop_sequence(gems_sfx::SCORE_COUNTER));
    }

    // Block headbutt. ASM loc_10DA4 queues rubber blocks as
    // `sfx_Rubber_block` and loc_110D0 queues metal/springy blocks as
    // `sfx_Metal_block`; when an exact block cue is present for this frame,
    // do not layer the generic `sfx_Block_hit` brick crunch over it.
    if curr.blocks_hit_this_frame > 0
        && !queued_sound_ids
            .iter()
            .copied()
            .any(is_explicit_block_sound_id)
    {
        sound_commands.push(NativeSoundCommand::start_sequence(gems_sfx::BLOCK_HIT));
    }

    // Block destroyed (new destroy effects appeared). ASM ice blocks queue
    // sfx_Prize_block=$11 from the core trace on the same frame as sub_10F9C's
    // destroy effect, so do not replay the historical delta heuristic too.
    if !queued_sound_ids.contains(&gems_sfx::DESTROY_BLOCK)
        && curr.block_destroy_effects.len() > prev.block_destroy_effects.len()
    {
        sound_commands.push(NativeSoundCommand::start_sequence(gems_sfx::DESTROY_BLOCK));
    }

    // Lava geyser erupted (new geysers appeared)
    if curr.active_geysers.len() > prev.active_geysers.len() {
        sound_commands.push(NativeSoundCommand::start_sequence(gems_sfx::LAVA_GEYSER));
    }

    // Game complete — ending music
    if curr.game_complete && !prev.game_complete {
        sound_commands.push(NativeSoundCommand::start_sequence(gems_bgm::ENDING));
    }

    // #342: the clock-prize loop is no longer inferred from a timer jump here.
    // ASM PrizeClock_Init loc_10792 starts sfx_Clock_prize when the falling
    // clock object spawns and loc_107E0 stops it when that object is deleted;
    // the core now queues both Start/StopSequence commands, which arrive via
    // `queued_gems_sound_commands_since`. A timer-jump start fired at collection
    // would re-trigger the loop after the core's stop, re-leaking it.

    sound_commands
}

/// Map level theme ID to GEMS BGM ID.
fn level_start_bgm(header: &LevelHeader) -> Option<u8> {
    // ASM loc_11FD2 (kid.asm:24183-24186): after Load_InGame's shared setup,
    // non-zero MurderWall_flag bypasses loc_12004's foreground-theme table and
    // queues bgm_Murderwall directly.
    if header.murder_wall_flags != 0 {
        Some(gems_bgm::MURDERWALL)
    } else {
        theme_to_bgm(header.level_theme)
    }
}

fn theme_to_bgm(theme: u8) -> Option<u8> {
    // ASM loc_12004 indexes `unk_1201E` by Foreground_theme, then queues the
    // resulting byte through j_PlaySound. These are sound IDs, not theme indexes.
    match theme {
        0 => Some(gems_bgm::MOUNTAIN),
        1 => Some(gems_bgm::SKY),
        2 => Some(gems_bgm::ICE),
        3 => Some(gems_bgm::HILLS),
        4 => Some(gems_bgm::ISLAND),
        5 => Some(gems_bgm::DESERT),
        6 => Some(gems_bgm::SWAMP),
        7 => Some(gems_bgm::MOUNTAIN),
        8 => Some(gems_bgm::CAVE),
        9 => Some(gems_bgm::FOREST),
        10 => Some(gems_bgm::CITY),
        _ => None,
    }
}

/// Detect game events and trigger GEMS sounds.
fn detect_gems_events(
    prev: &NativeTraceFrame,
    curr: &NativeTraceFrame,
    gems: &mut GemsAudioEngine,
) {
    for command in gems_event_audio_commands(prev, curr) {
        match command {
            NativeAudioTraceCommand::Sound(command) => apply_native_sound_command(gems, command),
            NativeAudioTraceCommand::MusicSpeed(speed) => gems.set_music_speed(speed),
        }
    }
}

struct SfxEngine {
    active_tones: Vec<SfxTone>,
    sample_rate: f32,
}

impl SfxEngine {
    fn new(sample_rate: u32) -> Self {
        Self {
            active_tones: Vec::new(),
            sample_rate: sample_rate.max(1) as f32,
        }
    }

    fn play(&mut self, frequency: f32, duration_ms: u32, volume: f32) {
        self.active_tones.push(SfxTone {
            frequency,
            remaining_samples: (self.sample_rate * duration_ms as f32 / 1000.0) as u32,
            phase: 0.0,
            volume,
        });
    }

    fn clear(&mut self) {
        self.active_tones.clear();
    }

    fn generate(&mut self, sample_count: usize) -> Vec<f32> {
        let mut samples = vec![0.0f32; sample_count];
        self.active_tones.retain_mut(|tone| {
            let phase_inc = tone.frequency / self.sample_rate;
            for sample in &mut samples {
                if tone.remaining_samples == 0 {
                    return false;
                }
                // Square wave with volume envelope
                let value = if tone.phase < 0.5 {
                    tone.volume
                } else {
                    -tone.volume
                };
                *sample += value;
                tone.phase = (tone.phase + phase_inc) % 1.0;
                tone.remaining_samples -= 1;
            }
            tone.remaining_samples > 0
        });
        // Clamp
        for s in &mut samples {
            *s = s.clamp(-1.0, 1.0);
        }
        samples
    }
}

fn detect_sfx_events(prev: &NativeTraceFrame, curr: &NativeTraceFrame, sfx: &mut SfxEngine) {
    let queued_sound_ids = queued_gems_sound_ids_since(prev, curr);
    let gameplay_sfx_fallback_active =
        matches!(curr.game_mode, GameMode::InGame | GameMode::DemoPlay);

    for sound_id in queued_sound_ids {
        play_fallback_queued_sfx_id(sfx, *sound_id);
    }

    // Jump (left ground with upward velocity)
    if gameplay_sfx_fallback_active
        && !queued_sound_ids.contains(&gems_sfx::JUMP)
        && prev.kid_on_ground == Some(true)
        && curr.kid_on_ground == Some(false)
        && curr.kid_y_velocity.is_some_and(|v| v < 0)
    {
        sfx.play(440.0, 40, 0.1);
    }
    // ASM PrizeDiamondCollected_Init (kid.asm:21103) plays the diamond cue on
    // contact. The delayed visible count increment is intentionally silent.
    // Enemy killed (score jumped by 1000+)
    if curr.score >= prev.score + 1000 && curr.diamond_count == prev.diamond_count {
        sfx.play(523.0, 80, 0.12);
        sfx.play(659.0, 80, 0.12);
    }
    // Took damage (HP decreased)
    if !queued_sound_ids
        .iter()
        .copied()
        .any(is_damage_resolution_voice_sound_id)
        && curr.number_hitpoints < prev.number_hitpoints
        && prev.number_hitpoints > 0
    {
        sfx.play(220.0, 150, 0.2);
    }
    // Form change (helmet changed)
    if curr.current_helmet_id != prev.current_helmet_id
        && curr.game_mode == prev.game_mode
        && !curr.game_over
        && !queued_sound_ids.iter().copied().any(is_transform_sound_id)
    {
        play_fallback_transform_sfx(sfx);
    }
    // Level complete
    if curr.level_complete && !prev.level_complete {
        sfx.play(523.0, 100, 0.15);
        sfx.play(659.0, 100, 0.15);
        sfx.play(784.0, 200, 0.15);
    }
    // Game over
    if curr.game_over && !prev.game_over {
        sfx.play(220.0, 200, 0.2);
        sfx.play(165.0, 300, 0.2);
    }
    // Life lost (lives decreased but not game over)
    if curr.number_lives < prev.number_lives && !curr.game_over {
        sfx.play(330.0, 120, 0.15);
        sfx.play(262.0, 180, 0.15);
    }
    // (Stomp SFX moved to direct trace flag check above)
    // Pause toggle
    if curr.paused && !prev.paused {
        sfx.play(440.0, 30, 0.08);
    }
    if !curr.paused && prev.paused {
        sfx.play(440.0, 30, 0.08);
    }
    // Level transition (new level loaded)
    if curr.current_level != prev.current_level && !curr.game_over && !curr.game_complete {
        sfx.play(330.0, 80, 0.1);
        sfx.play(440.0, 80, 0.1);
        sfx.play(550.0, 120, 0.1);
    }
    // Stomp bounce (direct trace flag)
    if curr.stomped_this_frame && !queued_sound_ids.contains(&gems_sfx::JUMP_ON_ENEMY) {
        play_fallback_jump_on_enemy_sfx(sfx);
    }
    // Block hit SFX (metallic clang for block headbutts). ASM loc_10DA4 and
    // loc_110D0 already supplied exact rubber/metal block cues for this frame.
    if curr.blocks_hit_this_frame > 0
        && !queued_sound_ids
            .iter()
            .copied()
            .any(is_explicit_block_sound_id)
    {
        sfx.play(1100.0, 30, 0.08); // metallic ping
        sfx.play(550.0, 40, 0.06); // lower thud
    }
    // Timer warning tick (under 30s, tick every second change)
    if curr.level_timer_seconds <= 30
        && curr.level_timer_seconds > 0
        && curr.level_timer_seconds < prev.level_timer_seconds
    {
        sfx.play(1200.0, 20, 0.08);
    }
    // Diamond Power activation (diamonds dropped by 20+)
    if prev.diamond_count >= 20
        && curr.diamond_count + 20 <= prev.diamond_count
        && curr.invulnerable_frames > 0
    {
        sfx.play(523.0, 60, 0.18); // C5
        sfx.play(784.0, 60, 0.18); // G5
        sfx.play(1047.0, 120, 0.2); // C6
    }
    // Timer bonus (clock pickup — seconds jumped up)
    if !queued_sound_ids.contains(&gems_sfx::CLOCK_PRIZE)
        && curr.level_timer_seconds > prev.level_timer_seconds + 1
    {
        sfx.play(698.0, 60, 0.12); // F5
        sfx.play(880.0, 80, 0.12); // A5
    }
    // Murder wall active (low rumble tone while wall is present)
    if curr.murder_wall_x.is_some() && prev.murder_wall_x.is_none() {
        sfx.play(80.0, 500, 0.08);
    }
    // Game complete / victory
    if curr.game_complete && !prev.game_complete {
        sfx.play(523.0, 150, 0.15); // C5
        sfx.play(659.0, 150, 0.15); // E5
        sfx.play(784.0, 150, 0.15); // G5
        sfx.play(1047.0, 300, 0.2); // C6
    }
}

fn try_load_block_grid(manifest: &AssetManifest, level_id: u8) -> Result<BlockGrid> {
    // Fixture-first: pre-dumped `level-layout/<id>.bin` is the fast, exact path
    // for every level the dumper reached. No change/regression for those.
    if let Some(grid) = try_load_generated_level_layout(manifest, level_id)? {
        return Ok(grid);
    }

    // Fixture absent (the unreachable bonus levels, e.g. logical 0x5A): fall back
    // to the engine's native terrain decoder via the same fixture-first-then-
    // decoder path the engine uses internally (`load_block_grid_for_level`),
    // instead of bailing. The render visuals for these levels exist in the
    // bundle; only the layout fixture was missing. Closes the "failed to load
    // visuals" half of #246.
    let runtime = NativeRuntime::new(manifest.clone());
    if let Some(grid) = runtime.block_grid_for_level(LevelId(level_id)) {
        return Ok(grid);
    }

    let resource_level = resource_level_id(manifest, level_id);
    bail!(
        "missing generated level layout for logical level 0x{level_id:02X} (resource level 0x{resource_level:02X}) and native terrain decode failed; run `mise run extract-assets` before using play-native debug warp"
    )
}

fn resolved_level_header(manifest: &AssetManifest, level_id: u8) -> Option<&LevelHeader> {
    resource_level_record(manifest, level_id)
        .and_then(|level| level.header_metadata.as_ref())
        .or_else(|| {
            manifest
                .levels
                .iter()
                .find(|level| level.level_id == level_id)
                .and_then(|level| level.header_metadata.as_ref())
        })
}

fn try_load_generated_level_layout(
    manifest: &AssetManifest,
    level_id: u8,
) -> Result<Option<BlockGrid>> {
    let layout_dir = manifest.output_root.join("level-layout");
    let logical_layout_path = layout_dir.join(format!("{level_id:02X}.bin"));
    let logical_metadata_path = layout_dir.join(format!("{level_id:02X}.json"));
    let logical_block_list_path = layout_dir.join(format!("{level_id:02X}.blocklist.bin"));
    let (layout_path, metadata_path, block_list_path) =
        if logical_layout_path.is_file() && logical_metadata_path.is_file() {
            (
                logical_layout_path,
                logical_metadata_path,
                logical_block_list_path,
            )
        } else {
            // ASM Load_Level_Data resolves Current_LevelID through
            // MapOrder_Index before decoding the BLOCK resource. Prefer
            // generated logical Level_Layout specialisations when present,
            // then fall back to the mapped resource layout for aliased levels.
            let resource_level = resource_level_id(manifest, level_id);
            (
                layout_dir.join(format!("{resource_level:02X}.bin")),
                layout_dir.join(format!("{resource_level:02X}.json")),
                layout_dir.join(format!("{resource_level:02X}.blocklist.bin")),
            )
        };
    if !layout_path.is_file() || !metadata_path.is_file() {
        return Ok(None);
    }
    if !block_list_path.is_file() {
        bail!(
            "generated Level_Layout {} is missing matching block list {}; play-native refuses partial generated layouts because Prize_Init loc_10342 would otherwise fall back to Diamond fixture mode",
            layout_path.display(),
            block_list_path.display(),
        );
    }

    let runtime = NativeRuntime::new(manifest.clone());
    runtime.block_grid_for_level(LevelId(level_id)).map_or_else(
        || {
            bail!(
                "failed to load generated Level_Layout through native runtime for logical level 0x{level_id:02X}; checked {} and {}",
                layout_path.display(),
                metadata_path.display()
            )
        },
        |grid| Ok(Some(grid)),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scenarios::{GameplayScenario, prize_block_scenario};
    use std::fs;
    use std::path::Path;
    use std::sync::Arc;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use tempfile::TempDir;
    use wildside_core::BlockBounceTrace;
    use wildside_core::{
        EOL_HOLOGRAM_VISIBLE_FRAME_LOC_C1D0_ORACLE, LevelId, NativeObjectKind,
        physics::Direction,
        terrain::{BlockType, Cell, CollisionKind},
    };
    use wildside_data::{
        AssetManifest, BackgroundLayoutAlias, DecodedAssetFile, DecodedLevelAssets,
        GENERATED_BUNDLE_VERSION, LevelHeader, LevelRecord, LevelResources, load_manifest,
    };
    use wildside_render::{
        AnimatedSpriteVisual, BackgroundChunk, BackgroundPlacement, BackgroundTile,
        BackgroundVisuals, BlockVisualMapping, ChunkedBackgroundVisuals, EnemyVisuals,
        ForegroundLayout, HudIcon, HudVisuals, IslandShoreVisualsLoc1202A, LevelVisuals, MdColour,
        NativeLevelVisuals, Palette, PaletteSet, PlatformVisuals, PlayerFormVisuals,
        PlayerTransformStep, PlayerVisuals, SpecialEffectUnknownVisuals, SpriteFrame, Tile,
        TileQuad, TileRef, TileSheet,
    };

    fn sample_trace(mode: GameMode) -> NativeTraceFrame {
        NativeTraceFrame {
            frame_index: 720,
            game_mode: mode,
            game_mode_value: mode.as_disassembly_value(),
            time_frames_loc_3038a: 0,
            start_press_count: 6,
            boot_phase: None,
            boot_phase_elapsed: None,
            title_screen_ready: false,
            current_player: 0,
            current_level: LevelId(0),
            player_1_level_id: LevelId(0),
            current_level_name: Some(String::from("Blue Lake Woods 1")),
            player_start_x: Some(0x0750),
            player_start_y: Some(0x00E0),
            flag_x: Some(0x0750),
            flag_y: Some(0x0300),
            level_enemy_object_count: Some(22),
            active_enemy_object_count: 1,
            active_platform_object_count: 1,
            number_lives: 3,
            number_hitpoints: 2,
            extra_hitpoint_slots: 0,
            current_helmet_id: 0,
            red_stealth_sword_timer: 0,
            red_stealth_airborne_slash_counter: 0,
            cyclone_flying: false,
            paused: false,
            pause_menu_option: 0,
            kid_x_pos: Some(0x0020_0000),
            kid_y_pos: Some(0x010F_0000),
            kid_x_velocity: Some(0),
            kid_y_velocity: Some(0),
            character_movement: None,
            kid_facing: Some(Direction::Right),
            kid_on_ground: Some(true),
            walk_visual_accumulator: 0,
            iron_knight_climbing: false,
            iron_knight_climb_pose_accum: 0,
            hud_diamond_frame_offset: None,
            hud_life_frame_offset: None,
            berzerker_charging: false,
            berzerker_charge_pose: false,
            berzerker_walk_pose: false,
            ledge_snap_cooldown: 0,
            kid_on_slope: 0,
            active_objects: Vec::new(),
            active_enemy_details: Vec::new(),
            camera_x_pos: 0x0010_0000,
            camera_y_pos: 0x0008_0000,
            number_objects: 3,
            number_gfx_objects: 4,
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
            sound_test_track: None,
            played_sound_effect_ids: Vec::new(),
            played_sound_commands: Vec::new(),
            played_bgm_ids: Vec::new(),
            played_music_speed_updates: Vec::new(),
            teleport_warp_colour: 0,
            level_entry_fade_progress_sub_73d0: None,
            two_player: false,
            options_two_player_two_controllers: false,
            options_controls_scheme: 0,
            options_speed_button_fast: false,
            number_continues: 3,
            invulnerable_frames: 0,
            kid_is_invulnerable: false,
            just_received_damage: false,
            diamond_power_active: false,
            diamond_count: 0,
            score: 0,
            level_timer_seconds: 180,
            level_timer_sub_frames: 0,
            level_damage_taken: false,
            level_prize_collected: false,
            level_speed_bonus_awarded: false,
            level_path_bonus_amount: 0,
            murder_wall_x: None,
            active_projectiles: Vec::new(),
            active_projectile_details: Vec::new(),
            diamond_power_sprites: Vec::new(),
            eyeclops_hard_beam_object: None,
            active_geysers: Vec::new(),
            lava_geyser_script_templates_loc_147ba: Vec::new(),
            island_shore_cycle_active: false,
            island_shore_cycle_state_loc_1202a: None,
            island_shore_vram_frames_loc_1202a: None,
            island_shore_dma_uploads_loc_1202a: Vec::new(),
            lava_geyser_art_uploads_loc_14c6a: Vec::new(),
            rotate_palette_lava_water_state_loc_1223c: None,
            lava_water_palette_words_loc_1223c: None,
            lava_geyser_palette_words_loc_14b2a: None,
            storm_pattern_upload_loc_14d32: None,
            storm_scroll_transfer_loc_30110: None,
            storm_shared_art_upload_loc_14d8a: None,
            special_effect_unknown_art_upload_loc_14df6: None,
            special_effect_unknown_plane_fill_loc_14e42: None,
            storm_falling_slots_sub_61ca: Vec::new(),
            storm_impact_slots_sub_61ca: Vec::new(),
            storm_sprites_sub_61ca: Vec::new(),
            block_bounces: Vec::new(),
            block_bounce_details: Vec::new(),
            block_destroy_effects: Vec::new(),
            block_debris_pieces: Vec::new(),
            prize_block_spawn_flashes: Vec::new(),
            kid_projectile_hit_sparks: Vec::new(),
            floating_items: Vec::new(),
            clock_prize_animation_swaps_loc_10796: Vec::new(),
            continue_prize_animation_swaps_loc_1080a: Vec::new(),
            enemy_death_effects: Vec::new(),
            heady_metal_death_splits: Vec::new(),
            heady_metal_body_pieces_sub_37abc: Vec::new(),
            stomped_this_frame: false,
            death_windup_elapsed: 0,
            form_change_flash_frames: 0,
            blocks_hit_this_frame: 0,
            sound_test_play_pending: false,
            gameplay_debug_fields: std::collections::BTreeMap::new(),
            physics_trace: None,
        }
    }

    fn has_rect(scene: &Scene, x: i32, y: i32, width: i32, height: i32, colour: [u8; 3]) -> bool {
        has_rect_command(&scene.commands, x, y, width, height, colour)
    }

    fn boss_eye_object_slot_snapshot_loc_3931a(
        x_pos: u32,
        y_pos: u32,
        mapping_index_loc_37abc: u8,
        sprite_order_word_a_loc_393c8: u16,
    ) -> NativeObjectSnapshot {
        let mut debug_fields = std::collections::BTreeMap::new();
        debug_fields.insert(
            "boss_eye_sprite_mapping_index_loc_37abc".to_owned(),
            mapping_index_loc_37abc.to_string(),
        );
        debug_fields.insert(
            "boss_eye_sprite_order_word_a_loc_393c8".to_owned(),
            format!("{sprite_order_word_a_loc_393c8:#06x}"),
        );

        NativeObjectSnapshot {
            kind: NativeObjectKind::Enemy { object_type: 0x21 },
            x_pos,
            y_pos,
            layout_index: None,
            x_velocity: Some(0),
            y_velocity: Some(0),
            facing: Some(Direction::Right),
            on_ground: None,
            hit_points: None,
            damage_flash: false,
            palette_variant: 0,
            stationary: false,
            contact_phase: None,
            shared_walker_state: None,
            shared_walker_patrol_timer: None,
            shared_walker_patrol_short_next: None,
            walk_phase: None,
            wait_timer: None,
            debug_fields,
        }
    }

    fn has_rect_command(
        commands: &[DrawCommand],
        x: i32,
        y: i32,
        width: i32,
        height: i32,
        colour: [u8; 3],
    ) -> bool {
        commands.iter().any(|command| {
            matches!(
                command,
                DrawCommand::Rect {
                    x: cx,
                    y: cy,
                    width: cw,
                    height: ch,
                    colour: cc,
                } if *cx == x && *cy == y && *cw == width && *ch == height && *cc == colour
            )
        })
    }

    fn contains_text(scene: &Scene, needle: &str) -> bool {
        scene.commands.iter().any(
            |command| matches!(command, DrawCommand::Text { text, .. } if text.contains(needle)),
        )
    }

    fn sprite_tiles(command: &DrawCommand) -> &[SpriteTile] {
        match command {
            DrawCommand::Sprite { tiles, .. } => tiles,
            other => panic!("expected sprite draw command, got {other:?}"),
        }
    }

    fn sprite_palette(command: &DrawCommand) -> &Palette {
        match command {
            DrawCommand::Sprite { palette, .. } => palette,
            other => panic!("expected sprite draw command, got {other:?}"),
        }
    }

    #[test]
    fn generic_enemy_projectile_uses_legacy_dot_render() {
        let mut trace = sample_trace(GameMode::InGame);
        trace.active_projectiles = vec![(0x0030_0000, 0x0020_0000, false)];

        let scene = build_ingame_scene(&trace, false, None);

        assert!(has_rect(&scene, 30, 22, 4, 4, [255, 80, 40]));
    }

    #[test]
    fn lion_projectile_uses_stru_3df34_sprite_frames() {
        let mut visuals = sample_level_visuals();
        let lion_visuals = visuals
            .enemies
            .families
            .get_mut(&0x12)
            .expect("sample Lion visuals");
        lion_visuals.palette = Palette {
            colours: vec![
                colour(0, 0, 0),
                colour(10, 20, 30),
                colour(20, 30, 40),
                colour(30, 40, 50),
                colour(40, 50, 60),
                colour(50, 60, 70),
                colour(60, 70, 80),
                colour(220, 0, 0),
                colour(180, 0, 0),
                colour(140, 0, 0),
                colour(100, 0, 0),
                colour(0, 220, 0),
                colour(36, 182, 0),
                colour(146, 255, 73),
                colour(0, 255, 146),
            ],
        };
        lion_visuals.frames[10].tile_sheet = blank_tile_sheet(8);
        for tile in &mut lion_visuals.frames[10].tile_sheet.tiles[4..8] {
            tile.pixels = [12; 64];
        }
        let mut trace = sample_trace(GameMode::InGame);
        trace.active_projectile_details = vec![ActiveProjectileSnapshot {
            x_pos: 0x0030_0000,
            y_pos: 0x0020_0000,
            x_velocity: 0x0006_0000,
            y_velocity: 0,
            friendly: false,
            remaining_frames: 0,
            animation_elapsed_frames: 6,
            kind: ActiveProjectileKind::LionProjectile,
            palette_variant: 0,
            boss_eye_sprite_mapping_index: 0,
            boss_eye_sprite_order_word_a: 0,
            dragon_flying_descent_fire_state: DragonFlyingDescentFireState::None,
            dragon_flying_descent_fire_counter: 0,
        }];

        let scene = build_scene(&trace, false, Some(&visuals), None);

        let projectile_sprite = scene.commands.iter().find_map(|command| match command {
            DrawCommand::Sprite { tiles, .. }
                if tiles
                    .first()
                    .is_some_and(|tile| tile.tile.tile_index == 0x004) =>
            {
                Some(tiles)
            }
            _ => None,
        });
        let tiles = projectile_sprite.expect(
            "ASM Lion.asm loc_3E12A / `stru_3DF34` should render the second C7E28 projectile frame",
        );
        assert_eq!(
            tiles[0].tile.tile_index, 0x004,
            "ASM Lion projectile frame C7E28 uses sprite_frame_vram tile base `$004`"
        );
        assert_eq!(
            tiles[0].tile.palette_line, 0,
            "ASM Lion.asm loc_3E12A sets hardware palette_line(a3) to 2, but native's extracted Lion palette is already local-slot indexed"
        );
        let mut rgb = vec![0; (NATIVE_FRAME_WIDTH * NATIVE_FRAME_HEIGHT * 3) as usize];
        render_scene(&scene, &mut rgb);
        assert_eq!(
            pixel_at(&rgb, 26, 24),
            [0, 220, 0],
            "ASM Lion.asm loc_3E12A / `stru_3DF34` projectile pixels should use the packed local Lion palette slots, not disappear behind the scene"
        );
        assert!(
            !has_rect(&scene, 30, 22, 4, 4, [255, 80, 40]),
            "Lion projectiles should not collapse to the generic enemy projectile dot"
        );
    }

    #[test]
    fn shooter_block_projectile_uses_visible_one_tile_render_loc_ef62() {
        let mut trace = sample_trace(GameMode::InGame);
        trace.active_projectile_details = vec![ActiveProjectileSnapshot {
            x_pos: 0x0030_0000,
            y_pos: 0x0020_0000,
            x_velocity: 4 << 16,
            y_velocity: 0,
            friendly: false,
            remaining_frames: 120,
            animation_elapsed_frames: 0,
            kind: ActiveProjectileKind::ShooterBlock,
            palette_variant: 0,
            boss_eye_sprite_mapping_index: 0,
            boss_eye_sprite_order_word_a: 0,
            dragon_flying_descent_fire_state: DragonFlyingDescentFireState::None,
            dragon_flying_descent_fire_counter: 0,
        }];

        let scene = build_ingame_scene(&trace, false, None);

        let shooter_sprite = scene.commands.iter().find_map(|command| match command {
            DrawCommand::Sprite { x, y, tiles, .. } if *x == 32 && *y == 20 => Some(tiles),
            _ => None,
        });
        let tiles =
            shooter_sprite.expect("ASM Process_ShooterObjects_MakeSprite should render ROM tile");
        assert_eq!(
            tiles[0].tile.tile_index, 0x076,
            "ASM loc_EE52 feeds tile word `$8A96`; after block VRAM base `$220`, native should draw block tile `$76`"
        );
        assert!(
            tiles[0].tile.flip_x,
            "ASM loc_EE52 tile word `$8A96` has the horizontal flip bit set for right-moving shooter bullets"
        );
        assert!(
            !has_rect(&scene, 30, 22, 4, 4, [255, 80, 40]),
            "typed shooter bullets should not use the generic projectile path"
        );
    }

    #[test]
    fn maniaxe_axe_projectile_uses_stru_8b86_player_form_sprite() {
        let mut visuals = sample_level_visuals();
        let mut maniaxe_visuals = visuals
            .player
            .forms
            .get(&0)
            .expect("Kid fallback visuals")
            .clone();
        maniaxe_visuals.axe = vec![
            sample_player_frame(90),
            sample_player_frame(91),
            sample_player_frame(92),
            sample_player_frame(93),
        ];
        visuals
            .player
            .forms
            .insert(PlayerForm::Maniaxe.helmet_id(), maniaxe_visuals);

        let mut trace = sample_trace(GameMode::InGame);
        trace.current_helmet_id = PlayerForm::Maniaxe.helmet_id();
        trace.active_projectile_details = vec![ActiveProjectileSnapshot {
            x_pos: 0x0030_0000,
            y_pos: 0x0020_0000,
            x_velocity: 5 << 16,
            y_velocity: 0,
            friendly: true,
            remaining_frames: 0,
            animation_elapsed_frames: 0,
            kind: ActiveProjectileKind::ManiaxeAxe,
            palette_variant: 0,
            boss_eye_sprite_mapping_index: 0,
            boss_eye_sprite_order_word_a: 0,
            dragon_flying_descent_fire_state: DragonFlyingDescentFireState::None,
            dragon_flying_descent_fire_counter: 0,
        }];

        let scene = build_scene(&trace, false, Some(&visuals), None);

        assert!(
            scene.commands.iter().any(|command| {
                matches!(
                    command,
                    DrawCommand::Sprite { tile_sheet, .. }
                        if tile_sheet.tiles[0].pixels[0] == 90
                )
            }),
            "ASM ManiaxeAxe_Init -> `stru_8B86` should render the first `maniaxe_axe_1..4` frame, not a generic projectile dot"
        );
        assert!(
            !has_rect(&scene, 30, 22, 4, 4, [100, 255, 100]),
            "typed Maniaxe axes should not fall back to FRIENDLY_PROJECTILE_COLOUR"
        );
    }

    #[test]
    fn maniaxe_axe_projectile_spins_through_stru_8b86_frames() {
        let mut visuals = sample_level_visuals();
        let mut maniaxe_visuals = visuals
            .player
            .forms
            .get(&0)
            .expect("Kid fallback visuals")
            .clone();
        maniaxe_visuals.axe = vec![
            sample_player_frame(90),
            sample_player_frame(91),
            sample_player_frame(92),
            sample_player_frame(93),
        ];
        visuals
            .player
            .forms
            .insert(PlayerForm::Maniaxe.helmet_id(), maniaxe_visuals);

        let mut trace = sample_trace(GameMode::InGame);
        trace.current_helmet_id = PlayerForm::Maniaxe.helmet_id();
        trace.active_projectile_details = vec![ActiveProjectileSnapshot {
            x_pos: 0x0030_0000,
            y_pos: 0x0020_0000,
            x_velocity: 5 << 16,
            y_velocity: 0,
            friendly: true,
            remaining_frames: 0,
            animation_elapsed_frames: 4,
            kind: ActiveProjectileKind::ManiaxeAxe,
            palette_variant: 0,
            boss_eye_sprite_mapping_index: 0,
            boss_eye_sprite_order_word_a: 0,
            dragon_flying_descent_fire_state: DragonFlyingDescentFireState::None,
            dragon_flying_descent_fire_counter: 0,
        }];

        let scene = build_scene(&trace, false, Some(&visuals), None);

        assert!(
            scene.commands.iter().any(|command| {
                matches!(
                    command,
                    DrawCommand::Sprite { tile_sheet, .. }
                        if tile_sheet.tiles[0].pixels[0] == 92
                )
            }),
            "ASM loc_8A52/stru_8B86 should use the exported animation clock to spin past the first Maniaxe axe frame"
        );
    }

    #[test]
    fn live_maniaxe_throw_renders_spinning_stru_8b86_frames() {
        let Some(manifest) = cached_generated_manifest() else {
            return;
        };
        let mut runtime = NativeRuntime::with_generated_level_layout(manifest);
        runtime.enter_level_playtest(LevelId(0x00));
        runtime.set_playtest_form(PlayerForm::Maniaxe);

        let mut visuals = sample_level_visuals();
        let mut maniaxe_visuals = visuals
            .player
            .forms
            .get(&0)
            .expect("Kid fallback visuals")
            .clone();
        maniaxe_visuals.axe = vec![
            sample_player_frame(90),
            sample_player_frame(91),
            sample_player_frame(92),
            sample_player_frame(93),
        ];
        visuals
            .player
            .forms
            .insert(PlayerForm::Maniaxe.helmet_id(), maniaxe_visuals);

        let mut first_seen_signature = None;
        let mut later_seen_signature = None;
        for frame in 0..8 {
            let trace = runtime.step_frame(FrameInput {
                c: frame == 0,
                ..FrameInput::default()
            });
            let Some(projectile) = trace.active_projectile_details.first() else {
                continue;
            };
            assert_eq!(
                projectile.kind,
                ActiveProjectileKind::ManiaxeAxe,
                "ASM ManiaxeAxe_Init installs `stru_8B86`; live traces must keep the thrown axe typed for native-player rendering"
            );
            let scene = build_scene(&trace, false, Some(&visuals), runtime.block_grid());
            let Some(signature) = scene.commands.iter().find_map(|command| match command {
                DrawCommand::Sprite { tile_sheet, .. }
                    if tile_sheet
                        .tiles
                        .first()
                        .is_some_and(|tile| matches!(tile.pixels[0], 90..=93)) =>
                {
                    Some(tile_sheet.tiles[0].pixels[0])
                }
                _ => None,
            }) else {
                panic!(
                    "ASM loc_8A52/stru_8B86 live Maniaxe axe should render a sprite frame, not the generic projectile rect"
                );
            };
            first_seen_signature.get_or_insert(signature);
            if Some(signature) != first_seen_signature {
                later_seen_signature = Some(signature);
                break;
            }
        }

        assert_eq!(
            first_seen_signature,
            Some(90),
            "live Maniaxe axe render should start on `maniaxe_axe_1` from ASM `stru_8B86`"
        );
        assert_eq!(
            later_seen_signature,
            Some(91),
            "ASM loc_8A52/stru_8B86 should advance the live thrown axe to the next render frame instead of staying visually static"
        );
    }

    #[test]
    fn boss_eye_attack_uses_mapping_index_marker_sub_37abc() {
        let mut trace = sample_trace(GameMode::InGame);
        trace.active_projectile_details = vec![ActiveProjectileSnapshot {
            x_pos: 0x0030_0000,
            y_pos: 0x0020_0000,
            x_velocity: 0,
            y_velocity: 0,
            friendly: false,
            remaining_frames: 120,
            animation_elapsed_frames: 0,
            kind: ActiveProjectileKind::BossEyeAttack,
            palette_variant: 0,
            boss_eye_sprite_mapping_index: 5,
            boss_eye_sprite_order_word_a: 0,
            dragon_flying_descent_fire_state: DragonFlyingDescentFireState::None,
            dragon_flying_descent_fire_counter: 0,
        }];

        let scene = build_ingame_scene(&trace, false, None);

        assert!(
            has_rect(&scene, 24, 16, 16, 16, [120, 200, 255]),
            "ASM sub_37ABC selects one of the 16x16 `unk_C8488..unk_C84C0` boss-eye frames"
        );
        assert!(
            has_rect(&scene, 26, 26, 4, 4, [255, 255, 255]),
            "mapping index 5 (`LnkTo_unk_C84B0`) should visibly select the lower-left boss-eye frame marker"
        );
        assert!(
            !has_rect(&scene, 30, 22, 4, 4, [255, 80, 40]),
            "HeadyMetal boss-eye attacks should not collapse to the generic enemy projectile dot"
        );
    }

    #[test]
    fn boss_eye_satellite_uses_orbit_marker_sub_393c8_39420() {
        let mut trace = sample_trace(GameMode::InGame);
        trace.active_projectile_details = vec![ActiveProjectileSnapshot {
            x_pos: 0x0030_0000,
            y_pos: 0x0020_0000,
            x_velocity: 0,
            y_velocity: 0,
            friendly: false,
            remaining_frames: 0,
            animation_elapsed_frames: 0,
            kind: ActiveProjectileKind::BossEyeSatellite,
            palette_variant: 0,
            boss_eye_sprite_mapping_index: 3,
            boss_eye_sprite_order_word_a: 0,
            dragon_flying_descent_fire_state: DragonFlyingDescentFireState::None,
            dragon_flying_descent_fire_counter: 0,
        }];

        let scene = build_ingame_scene(&trace, false, None);

        assert!(
            has_rect(&scene, 24, 16, 16, 16, [120, 200, 255]),
            "ASM sub_393C8/sub_39420 orbit satellites use the same 16x16 boss-eye sprite family refreshed by sub_37ABC"
        );
        assert!(
            has_rect(&scene, 25, 22, 4, 4, [255, 255, 255]),
            "mapping index 3 (`LnkTo_unk_C84A0`) should visibly select the left boss-eye marker"
        );
        assert!(
            !has_rect(&scene, 30, 22, 4, 4, [255, 80, 40]),
            "waiting boss-eye satellites should not collapse to the generic projectile dot"
        );
    }

    #[test]
    fn boss_eye_object_slot_uses_orbit_marker_loc_3931a_37abc() {
        let mut trace = sample_trace(GameMode::InGame);
        trace.active_projectile_details.clear();
        let mut debug_fields = std::collections::BTreeMap::new();
        debug_fields.insert(
            "boss_eye_sprite_mapping_index_loc_37abc".to_owned(),
            "3".to_owned(),
        );
        trace.active_objects = vec![NativeObjectSnapshot {
            kind: NativeObjectKind::Enemy { object_type: 0x21 },
            x_pos: 0x0030_0000,
            y_pos: 0x0020_0000,
            layout_index: None,
            x_velocity: Some(0),
            y_velocity: Some(0),
            facing: Some(Direction::Right),
            on_ground: None,
            hit_points: None,
            damage_flash: false,
            palette_variant: 0,
            stationary: false,
            contact_phase: None,
            shared_walker_state: None,
            shared_walker_patrol_timer: None,
            shared_walker_patrol_short_next: None,
            walk_phase: None,
            wait_timer: None,
            debug_fields,
        }];

        let scene = build_ingame_scene(&trace, false, None);

        assert!(
            has_rect(&scene, 24, 16, 16, 16, [120, 200, 255]),
            "ASM loc_3931A `$21` waiting boss-eye object slots should render through the same sub_37ABC 16x16 marker as the old projectile bridge"
        );
        assert!(
            has_rect(&scene, 25, 22, 4, 4, [255, 255, 255]),
            "mapping index 3 from the `$21` object debug field should visibly select the left boss-eye marker"
        );
    }

    fn glyph_worker_trace(mode: GameMode) -> NativeTraceFrame {
        // #349: the core surfaces each post-death Title_Letter worker through
        // `active_objects` as `Enemy{object_type:0}` with a `glyph` debug field
        // (see `death_respawn_exposes_load_title_card_vdp_shell_loc_b8de`).
        let mut trace = sample_trace(mode);
        trace.active_objects.clear();
        let mut debug_fields = std::collections::BTreeMap::new();
        debug_fields.insert("glyph".to_owned(), "0x61".to_owned()); // 'a' -> 'A'
        debug_fields.insert("gfx_slot_index".to_owned(), "0".to_owned());
        trace.active_objects = vec![NativeObjectSnapshot {
            kind: NativeObjectKind::Enemy { object_type: 0 },
            x_pos: 0x0080_0000,
            y_pos: 0x0070_0000,
            layout_index: None,
            x_velocity: Some(0),
            y_velocity: Some(0),
            facing: Some(Direction::Right),
            on_ground: None,
            hit_points: None,
            damage_flash: false,
            palette_variant: 0,
            stationary: false,
            contact_phase: None,
            shared_walker_state: None,
            shared_walker_patrol_timer: None,
            shared_walker_patrol_short_next: None,
            walk_phase: None,
            wait_timer: None,
            debug_fields,
        }];
        trace
    }

    #[test]
    fn death_respawn_title_card_renders_level_name_glyphs_loc_b8de() {
        // #349: ASM lose_life (kid.asm:14763) -> Game_Mode 8 -> Load_TitleCard
        // allocates the same Title_Letter glyph workers as a normal entry, so the
        // post-death card MUST draw the level name. The death-respawn window is
        // traced as InGame during the `respawn_live_handoff` hand-off
        // (trace.rs:9177) and as LevelTransition during the `Respawning` settle —
        // both must render the live glyph workers, not just the transition path.
        let transition = build_scene(
            &glyph_worker_trace(GameMode::LevelTransition),
            false,
            None,
            None,
        );
        assert!(
            contains_text(&transition, "A"),
            "the LevelTransition post-death card must draw the level-name glyph workers"
        );

        let ingame = build_ingame_scene(&glyph_worker_trace(GameMode::InGame), false, None);
        assert!(
            contains_text(&ingame, "A"),
            "the InGame respawn hand-off frame must ALSO draw the level-name glyph workers (the post-death card shows the scene but dropped the name)"
        );
    }

    #[test]
    fn death_respawn_vdp_path_overlays_real_title_name_workers_loc_b8de() {
        let Some(manifest) = cached_generated_manifest() else {
            return;
        };
        let (visuals, _) = load_session_assets(&manifest, 0x00).expect("Level 0x00 visuals");
        let mut runtime = NativeRuntime::with_generated_level_layout(manifest);
        let script = GameplayScenario::DeathRespawnLevel1.script();
        let mut previous_trace = runtime.trace_frame();

        for frame_index in 0..=1_919 {
            let buttons = script.get(frame_index).copied().unwrap_or_default();
            let trace = runtime.step_frame(FrameInput::from_mask(buttons));
            let render_trace = render_trace_with_vdp_latched_state(&trace, Some(&previous_trace));
            previous_trace = trace.clone();

            let has_on_screen_title_worker = render_trace.active_objects.iter().any(|object| {
                matches!(object.kind, NativeObjectKind::Enemy { object_type: 0 })
                    && object.debug_fields.contains_key("glyph")
                    && {
                        let x = (object.x_pos as i32) >> 16;
                        let y = (object.y_pos as i32) >> 16;
                        x > -16 && x < FRAME_WIDTH as i32 && y > -16 && y < FRAME_HEIGHT as i32
                    }
            });
            if frame_index < 1_791 || !has_on_screen_title_worker {
                continue;
            }

            let Some(vdp) = runtime.vdp_state() else {
                continue;
            };
            let mut base = vec![0; (FRAME_WIDTH * FRAME_HEIGHT * 3) as usize];
            render_native_vdp_rgb(vdp, &mut base);
            let mut overlaid = base.clone();
            render_vdp_trace_overlay(&render_trace, &visuals, &mut overlaid);
            let changed_pixels = base
                .chunks_exact(3)
                .zip(overlaid.chunks_exact(3))
                .filter(|(before, after)| before != after)
                .count();
            assert!(
                changed_pixels > 0,
                "ASM lose_life -> loc_B8DE -> Load_TitleCard should composite loc_1AB26/loc_1AC70 title workers over the VDP shell on real death_respawn_level1 frame {frame_index}; mode={:?} boot_phase={:?} objects={:?}",
                render_trace.game_mode,
                render_trace.boot_phase,
                render_trace.active_objects,
            );
            return;
        }

        panic!(
            "death_respawn_level1 should expose an on-screen loc_1AB26/loc_1AC70 title worker on a VDP respawn frame"
        );
    }

    #[test]
    fn boss_eye_object_slots_render_in_sub_ff6_sprite_order() {
        let mut trace = sample_trace(GameMode::InGame);
        trace.active_projectile_details.clear();
        trace.active_objects = vec![
            boss_eye_object_slot_snapshot_loc_3931a(0x0030_0000, 0x0020_0000, 5, 0xFFFF),
            boss_eye_object_slot_snapshot_loc_3931a(0x0030_0000, 0x0020_0000, 3, 1),
        ];

        let scene = build_ingame_scene(&trace, false, None);
        let white_highlights = scene
            .commands
            .iter()
            .filter_map(|command| match command {
                DrawCommand::Rect {
                    x,
                    y,
                    width: 4,
                    height: 4,
                    colour: [255, 255, 255],
                } => Some((*x, *y)),
                _ => None,
            })
            .collect::<Vec<_>>();

        assert_eq!(
            white_highlights,
            vec![(25, 22), (26, 26)],
            "ASM sub_FF6 reinserts `$21` waiting boss-eye objects in ascending unsigned `$A(a3)` order"
        );
    }

    #[test]
    fn boss_eye_satellites_render_in_sub_ff6_sprite_order() {
        let mut trace = sample_trace(GameMode::InGame);
        trace.active_projectile_details = vec![
            ActiveProjectileSnapshot {
                x_pos: 0x0030_0000,
                y_pos: 0x0020_0000,
                x_velocity: 0,
                y_velocity: 0,
                friendly: false,
                remaining_frames: 0,
                animation_elapsed_frames: 0,
                kind: ActiveProjectileKind::BossEyeSatellite,
                palette_variant: 0,
                boss_eye_sprite_mapping_index: 5,
                boss_eye_sprite_order_word_a: 0xFFFF,
                dragon_flying_descent_fire_state: DragonFlyingDescentFireState::None,
                dragon_flying_descent_fire_counter: 0,
            },
            ActiveProjectileSnapshot {
                x_pos: 0x0030_0000,
                y_pos: 0x0020_0000,
                x_velocity: 0,
                y_velocity: 0,
                friendly: false,
                remaining_frames: 0,
                animation_elapsed_frames: 0,
                kind: ActiveProjectileKind::BossEyeSatellite,
                palette_variant: 0,
                boss_eye_sprite_mapping_index: 3,
                boss_eye_sprite_order_word_a: 1,
                dragon_flying_descent_fire_state: DragonFlyingDescentFireState::None,
                dragon_flying_descent_fire_counter: 0,
            },
        ];

        let scene = build_ingame_scene(&trace, false, None);
        let white_highlights = scene
            .commands
            .iter()
            .filter_map(|command| match command {
                DrawCommand::Rect {
                    x,
                    y,
                    width: 4,
                    height: 4,
                    colour: [255, 255, 255],
                } => Some((*x, *y)),
                _ => None,
            })
            .collect::<Vec<_>>();

        assert_eq!(
            white_highlights,
            vec![(25, 22), (26, 26)],
            "ASM sub_FF6 reinserts gfx objects in ascending unsigned `$A(a3)` order; the `$FFFF` boss eye must render after the `$0001` eye"
        );
    }

    #[test]
    #[ignore = "requires generated .cache assets; run mise run extract-assets first"]
    fn highwater_shooter_block_projectile_resolves_real_rom_spike_tile_loc_ef62() {
        let Some(manifest) = cached_generated_manifest() else {
            return;
        };
        let visuals = load_level_visuals(&manifest, 0x02).expect("Highwater Pass visuals");
        let mut trace = sample_trace(GameMode::InGame);
        trace.current_level = LevelId(0x02);
        trace.active_projectile_details = vec![ActiveProjectileSnapshot {
            x_pos: 0x0030_0000,
            y_pos: 0x0020_0000,
            x_velocity: 4 << 16,
            y_velocity: 0,
            friendly: false,
            remaining_frames: 120,
            animation_elapsed_frames: 0,
            kind: ActiveProjectileKind::ShooterBlock,
            palette_variant: 0,
            boss_eye_sprite_mapping_index: 0,
            boss_eye_sprite_order_word_a: 0,
            dragon_flying_descent_fire_state: DragonFlyingDescentFireState::None,
            dragon_flying_descent_fire_counter: 0,
        }];

        let scene = build_scene(&trace, false, Some(&visuals), None);

        let shooter_sprite = scene.commands.iter().find_map(|command| match command {
            DrawCommand::Sprite {
                x,
                y,
                tile_sheet,
                tiles,
                ..
            } if *x == 32 && *y == 20 => Some((tile_sheet, tiles)),
            _ => None,
        });
        let (tile_sheet, tiles) =
            shooter_sprite.expect("ASM Process_ShooterObjects loc_EF62 should draw a sprite");
        let tile = tiles[0].tile;
        assert_eq!(
            tile.tile_index, 0x076,
            "ASM loc_EE52 writes `$8A96`, i.e. block VRAM tile `$296` minus base `$220`"
        );
        assert!(tile.flip_x, "ASM `$8A96` sets the horizontal flip bit");
        let pixels = &tile_sheet.tiles[usize::from(tile.tile_index)].pixels;
        assert!(
            pixels.contains(&0) && pixels.iter().any(|pixel| *pixel != 0),
            "generated ArtComp_992E4_Blocks tile `$76` should be a transparent spike tile, not a filled fallback square"
        );
        assert!(
            !has_rect(&scene, 28, 20, 8, 8, [255, 216, 72]),
            "real Highwater assets must resolve the ROM shooter tile instead of the square fallback"
        );
    }

    #[test]
    #[ignore = "requires generated .cache assets; run mise run extract-assets first"]
    fn highwater_foreground_palette_rotates_lava_water_cram_words_loc_1223c() {
        let Some(manifest) = cached_generated_manifest() else {
            return;
        };
        let visuals = load_level_visuals(&manifest, 0x02).expect("Highwater Pass visuals");
        assert_eq!(visuals.level.foreground_theme_id, 7);

        let mut trace = sample_trace(GameMode::InGame);
        trace.current_level = LevelId(0x02);
        trace.frame_index = 0;
        let palette0 = foreground_palette_for_scene(&trace, &visuals);
        assert_eq!(
            &palette0.colours[6..10],
            &[
                MdColour::from_word(0x0E62),
                MdColour::from_word(0x0EC4),
                MdColour::from_word(0x0E20),
                MdColour::from_word(0x0E40),
            ],
            "ASM RotatePalette_Lava_Water loc_1223C starts from Palette_Buffer+$E/$10/$12/$14; direct Highwater level-skip recordings reach the first visible native frame two ROM rotations later"
        );

        trace.frame_index = 5;
        let palette5 = foreground_palette_for_scene(&trace, &visuals);
        assert_eq!(
            &palette5.colours[6..10],
            &[
                MdColour::from_word(0x0E40),
                MdColour::from_word(0x0E62),
                MdColour::from_word(0x0EC4),
                MdColour::from_word(0x0E20),
            ],
            "ASM loc_1223A hibernates 5 Mountain frames before the first loc_1223C rotation"
        );

        trace.lava_water_palette_words_loc_1223c = Some([0x000E, 0x0010, 0x0012, 0x0014]);
        trace.frame_index = 999;
        let live_palette = foreground_palette_for_scene(&trace, &visuals);
        assert_eq!(
            &live_palette.colours[6..10],
            &[
                MdColour::from_word(0x000E),
                MdColour::from_word(0x0010),
                MdColour::from_word(0x0012),
                MdColour::from_word(0x0014),
            ],
            "ASM loc_1223C writes Palette_Buffer+$E/$10/$12/$14; renderer should prefer typed live trace words over route-specific frame-index inference"
        );
    }

    #[test]
    fn level_entry_fade_adds_full_scene_palette_fade_sub_73d0() {
        let visuals = sample_level_visuals();
        let mut trace = sample_trace(GameMode::InGame);
        trace.teleport_warp_colour = 0;
        trace.level_entry_fade_progress_sub_73d0 = Some(0x0000);

        let scene = build_scene(&trace, false, Some(&visuals), None);

        assert!(
            matches!(
                scene.commands.last(),
                Some(DrawCommand::PaletteFadeSub73D0 {
                    target_word: 0,
                    fade_progress: 0
                })
            ),
            "ASM sub_73D0 drives sub_14C0 over the whole CRAM palette; native-player must fade sprites and HUD, not only terrain palettes"
        );
        assert!(
            !scene
                .commands
                .iter()
                .any(|command| matches!(command, DrawCommand::Tint { .. })),
            "the rough teleport tint should not stack on top of the real sub_73D0 palette fade"
        );
    }

    #[test]
    #[ignore = "requires generated .cache assets; run mise run extract-assets first"]
    fn highwater_priority_foreground_masks_low_priority_sprites_loc_3034a() {
        let Some(manifest) = cached_generated_manifest() else {
            return;
        };
        let session_assets =
            load_native_session_assets(&manifest, 0x02).expect("Highwater Pass session assets");
        assert_eq!(
            session_assets.block_grid.raw_word_at(26, 77) & 0x8100,
            0x0100,
            "generated Highwater Level_Layout should preserve the ROM foreground priority bit near the lower waterfall-front mask"
        );

        let mut trace = sample_trace(GameMode::InGame);
        trace.current_level = LevelId(0x02);
        trace.camera_x_pos = (24 * 16) << 16;
        trace.camera_y_pos = (72 * 16) << 16;

        let mut scene = build_scene(
            &trace,
            false,
            Some(&session_assets.level_visuals),
            Some(&session_assets.block_grid),
        );
        let sample_pixels = (0usize..80)
            .flat_map(|x| (0usize..192).map(move |y| (x, y)))
            .collect();
        let priority_pixel = sample_scene_pixel_provenance(&scene, &sample_pixels)
            .into_iter()
            .find(|diagnostic| diagnostic.final_high_priority)
            .expect(
                "Highwater priority foreground should contribute visible pixels in this viewport",
            );

        let sprite_colour = [220, 20, 20];
        scene.commands.push(DrawCommand::Sprite {
            x: priority_pixel.x as i32,
            y: priority_pixel.y as i32,
            tile_sheet: Arc::new(TileSheet {
                tiles: vec![make_tile(&[(0, 0, 1)])],
            }),
            palette: Arc::new(Palette {
                colours: vec![colour(220, 20, 20)],
            }),
            tiles: vec![SpriteTile {
                offset_x: 0,
                offset_y: 0,
                tile: TileRef {
                    tile_index: 0,
                    palette_line: 0,
                    priority: false,
                    flip_x: false,
                    flip_y: false,
                },
            }],
        });
        let sample = sample_scene_pixel_provenance(
            &scene,
            &[(priority_pixel.x, priority_pixel.y)].into_iter().collect(),
        )
        .into_iter()
        .next()
        .expect("priority sample");

        assert!(sample.final_high_priority);
        assert!(
            sample.writes.iter().any(|write| {
                write.kind == "sprite_tile" && write.rgb == Some(sprite_colour) && !write.accepted
            }),
            "ASM sub_3034A/sub_302F6 priority terrain must mask ordinary low-priority sprite tiles in live Highwater art, not only synthetic layouts"
        );
    }

    #[test]
    #[ignore = "requires generated .cache assets; run mise run extract-assets first"]
    fn highwater_live_trace_shooter_projectile_renders_real_spike_tile_loc_ef62() {
        let Some(manifest) = cached_generated_manifest() else {
            return;
        };
        let session_assets =
            load_native_session_assets(&manifest, 0x02).expect("Highwater Pass session assets");
        let mut runtime = NativeRuntime::new(manifest);
        runtime.set_block_grid(session_assets.block_grid.clone());
        runtime.enter_level_playtest(LevelId(0x02));
        runtime.set_playtest_form(PlayerForm::Juggernaut);
        runtime.set_player_probe_position(11 * 16 - 24, 97 * 16 + 8);

        let trace = runtime.step_frame(FrameInput::default());

        assert!(
            trace
                .active_projectile_details
                .iter()
                .any(|projectile| projectile.kind == ActiveProjectileKind::ShooterBlock),
            "ASM loc_EAE4 -> loc_EC76 should expose live Highwater shooter bullets through active_projectile_details, not only legacy active_projectiles"
        );
        let scene = build_scene(
            &trace,
            false,
            Some(&session_assets.level_visuals),
            runtime.block_grid(),
        );
        let shooter_sprite = scene.commands.iter().find_map(|command| match command {
            DrawCommand::Sprite { tiles, .. }
                if tiles
                    .iter()
                    .any(|tile| tile.tile.tile_index == 0x076 && tile.tile.flip_x) =>
            {
                Some(tiles)
            }
            _ => None,
        });
        assert!(
            shooter_sprite.is_some(),
            "ASM Process_ShooterObjects loc_EE52/loc_EF62 should render live `$EA1E` shots as flipped block-art tile `$76`, not a square fallback"
        );
        assert!(
            !scene.commands.iter().any(|command| matches!(
                command,
                DrawCommand::Rect {
                    width: 8,
                    height: 8,
                    colour: [255, 216, 72],
                    ..
                }
            )),
            "live typed shooter bullets should not take the fallback square renderer"
        );
    }

    #[test]
    #[ignore = "requires generated .cache assets; run mise run extract-assets first"]
    fn highwater_juggernaut_shooter_clears_visible_ice_barrier_loc_f02a() {
        let Some(manifest) = cached_generated_manifest() else {
            return;
        };
        let session_assets =
            load_native_session_assets(&manifest, 0x02).expect("Highwater Pass session assets");
        let mut runtime = NativeRuntime::new(manifest);
        runtime.set_block_grid(session_assets.block_grid.clone());
        runtime.enter_level_playtest(LevelId(0x02));
        runtime.set_playtest_form(PlayerForm::Juggernaut);
        runtime.set_player_probe_position(11 * 16 - 24, 97 * 16 + 8);

        let mut trace = runtime.step_frame(FrameInput::default());
        for _ in 0..13 {
            trace = runtime.step_frame(FrameInput::default());
        }

        let barrier_col = 15;
        let barrier_row = 97;
        assert_eq!(
            runtime
                .block_grid()
                .expect("block grid")
                .cell_at(barrier_col, barrier_row),
            Cell::from_be_bytes(0, 0),
            "ASM loc_EE2A -> loc_F02A should erase Highwater Pass 1's first ice barrier cell"
        );

        let barrier_x = barrier_col as i32 * 16 - ((trace.camera_x_pos >> 16) as i32);
        let barrier_y = barrier_row as i32 * 16 - ((trace.camera_y_pos >> 16) as i32);
        let scene = build_scene(
            &trace,
            false,
            Some(&session_assets.level_visuals),
            runtime.block_grid(),
        );

        assert!(
            topmost_indexed_tile_at_with_sheet_len(&scene, barrier_x, barrier_y, 0x200).is_none(),
            "ASM loc_F02A/sub_10F9C erases the block override; native must not keep drawing the cleared ice barrier as block art"
        );
    }

    #[test]
    #[ignore = "requires generated .cache assets; run mise run extract-assets first"]
    fn highwater_berzerker_live_charge_from_spawn_renders_spikes_and_clears_barrier_loc_ef62_f02a()
    {
        let Some(manifest) = cached_generated_manifest() else {
            return;
        };
        let session_assets =
            load_native_session_assets(&manifest, 0x02).expect("Highwater Pass session assets");
        let mut runtime = NativeRuntime::new(manifest);
        runtime.set_block_grid(session_assets.block_grid.clone());
        runtime.enter_level_playtest(LevelId(0x02));
        runtime.set_playtest_form(PlayerForm::Berzerker);

        let mut saw_spike_sprite = false;
        let mut saw_fallback_square = false;
        for _ in 0..120 {
            let trace = runtime.step_frame(FrameInput {
                right: true,
                ..FrameInput::default()
            });
            let scene = build_scene(
                &trace,
                false,
                Some(&session_assets.level_visuals),
                runtime.block_grid(),
            );
            saw_spike_sprite |= scene.commands.iter().any(|command| match command {
                DrawCommand::Sprite { tiles, .. } => tiles
                    .iter()
                    .any(|tile| tile.tile.tile_index == 0x076 && tile.tile.flip_x),
                _ => false,
            });
            saw_fallback_square |= scene.commands.iter().any(|command| {
                matches!(
                    command,
                    DrawCommand::Rect {
                        width: 8,
                        height: 8,
                        colour: [255, 216, 72],
                        ..
                    }
                )
            });
        }

        assert!(
            saw_spike_sprite,
            "ASM Process_ShooterObjects loc_EE52/loc_EF62 should render the live Berzerker-route Highwater shot as the flipped ROM spike tile"
        );
        assert!(
            !saw_fallback_square,
            "live Highwater shooter shots should not fall back to the yellow square placeholder"
        );
        for row in 97..=103 {
            assert_eq!(
                runtime.block_grid().expect("block grid").cell_at(15, row),
                Cell::from_be_bytes(0, 0),
                "ASM loc_EE2A -> loc_F02A should erase Highwater Pass 1 ice barrier row {row} on the live Berzerker route"
            );
        }
    }

    #[test]
    fn tank_bullet_uses_rom_child_sprite_frame_loc_3b730_stru_3b526() {
        let visuals = sample_level_visuals();
        let mut trace = sample_trace(GameMode::InGame);
        trace.active_projectile_details = vec![ActiveProjectileSnapshot {
            x_pos: 0x0030_0000,
            y_pos: 0x0020_0000,
            x_velocity: 0x0003_0000,
            y_velocity: 0,
            friendly: false,
            remaining_frames: u16::MAX,
            animation_elapsed_frames: 0,
            kind: ActiveProjectileKind::TankBullet,
            palette_variant: 0,
            boss_eye_sprite_mapping_index: 0,
            boss_eye_sprite_order_word_a: 0,
            dragon_flying_descent_fire_state: DragonFlyingDescentFireState::None,
            dragon_flying_descent_fire_counter: 0,
        }];

        let scene = build_scene(&trace, false, Some(&visuals), None);

        let bullet_sprite = scene.commands.iter().find_map(|command| match command {
            DrawCommand::Sprite { x, y, tiles, .. } if *x == 23 && *y == 19 => Some(tiles),
            _ => None,
        });
        let tiles = bullet_sprite
            .expect("ASM Tank_Shooting.asm loc_3B730 should render a ROM bullet child");
        assert_eq!(
            tiles[0].tile.tile_index, 0x038,
            "ASM `stru_3B526` first frame uses `unk_C7EE8: sprite_frame_vram $038,$09,$00,$10,$05`",
        );
        assert!(
            !has_rect(&scene, 30, 22, 4, 4, [255, 80, 40]),
            "RockTank bullets should not collapse to the generic enemy projectile dot",
        );
    }

    #[test]
    fn cloud_lightning_uses_rom_bolt_sprite_frame_loc_35df0_unk_c7866() {
        let mut visuals = sample_level_visuals();
        let cloud_visuals = visuals
            .enemies
            .families
            .get_mut(&0x0E)
            .expect("sample Cloud visuals");
        let mut tile_sheet = blank_tile_sheet(8);
        for tile in &mut tile_sheet.tiles[0..6] {
            tile.pixels = [2; 64];
        }
        cloud_visuals.frames[0].tile_sheet = tile_sheet;

        let mut trace = sample_trace(GameMode::InGame);
        trace.active_projectile_details = vec![ActiveProjectileSnapshot {
            x_pos: 0x0030_0000,
            y_pos: 0x0020_0000,
            x_velocity: 0x0004_0000,
            y_velocity: 0,
            friendly: false,
            remaining_frames: 0,
            animation_elapsed_frames: 0,
            kind: ActiveProjectileKind::CloudLightning,
            palette_variant: 0,
            boss_eye_sprite_mapping_index: 0,
            boss_eye_sprite_order_word_a: 0,
            dragon_flying_descent_fire_state: DragonFlyingDescentFireState::None,
            dragon_flying_descent_fire_counter: 0,
        }];

        let scene = build_scene(&trace, false, Some(&visuals), None);

        let lightning_sprite = scene.commands.iter().find_map(|command| match command {
            DrawCommand::Sprite { x, y, tiles, .. }
                if tiles.len() == 6
                    && tiles
                        .first()
                        .is_some_and(|tile| tile.tile.tile_index == 0x000) =>
            {
                Some((*x, *y, tiles))
            }
            _ => None,
        });
        let (draw_x, draw_y, tiles) = lightning_sprite
            .expect("ASM Cloud.asm loc_35DF0 should render the horizontal lightning bolt sprite");
        assert_eq!(
            tiles[0].tile.tile_index, 0x000,
            "ASM `unk_C7866` uses `sprite_frame_vram $000,$00,$04,$30,$07` for horizontal Cloud bolts",
        );
        let mut rgb = vec![0; (NATIVE_FRAME_WIDTH * NATIVE_FRAME_HEIGHT * 3) as usize];
        render_scene(&scene, &mut rgb);
        assert_eq!(
            pixel_at(&rgb, draw_x as u32, draw_y as u32),
            [190, 220, 250],
            "ASM Cloud.asm loc_35DF0 bolt art should render from the Cloud enemy palette",
        );
        assert!(
            !scene.commands.iter().any(|command| matches!(
                command,
                DrawCommand::Rect { width: 4, height: 4, colour, .. } if *colour == [255, 80, 40]
            )),
            "Cloud lightning should not collapse to the generic red enemy projectile dot",
        );
    }

    #[test]
    fn tar_monster_glob_uses_rom_child_sprite_frame_loc_3247e_unk_c781e() {
        // Backlog #243: the Highwater Tar Monster ($05) shoots a dark tar glob,
        // not a red dot. ASM Tar_Monster.asm loc_3247E writes `LnkTo_unk_C781E`
        // = `sprite_frame_vram $0BC,$04,$04,$1E,$08` (decoded from kid.bin).
        let visuals = sample_level_visuals();
        let mut trace = sample_trace(GameMode::InGame);
        trace.active_projectile_details = vec![ActiveProjectileSnapshot {
            x_pos: 0x0030_0000,
            y_pos: 0x0020_0000,
            x_velocity: -0x0004_0000,
            y_velocity: 0,
            friendly: false,
            remaining_frames: 120,
            animation_elapsed_frames: 0,
            kind: ActiveProjectileKind::TarMonsterGlob,
            palette_variant: 0,
            boss_eye_sprite_mapping_index: 0,
            boss_eye_sprite_order_word_a: 0,
            dragon_flying_descent_fire_state: DragonFlyingDescentFireState::None,
            dragon_flying_descent_fire_counter: 0,
        }];

        let scene = build_scene(&trace, false, Some(&visuals), None);

        let glob_tiles = scene.commands.iter().find_map(|command| match command {
            DrawCommand::Sprite { tiles, .. }
                if tiles.iter().any(|tile| tile.tile.tile_index == 0x0BC) =>
            {
                Some(tiles)
            }
            _ => None,
        });
        assert!(
            glob_tiles.is_some(),
            "ASM Tar_Monster.asm loc_3247E should render the glob from ROM tile `unk_C781E` ($0BC)",
        );
        assert!(
            !scene.commands.iter().any(|command| matches!(
                command,
                DrawCommand::Rect { width: 4, height: 4, colour, .. } if *colour == [255, 80, 40]
            )),
            "Tar Monster globs must not fall back to the generic red enemy projectile dot",
        );
    }

    #[test]
    fn tar_monster_burst_child_uses_rom_child_sprite_frame_loc_325e6_unk_c782e() {
        // Backlog #243: retaliation fragments use `LnkTo_unk_C782E`
        // = `sprite_frame_vram $0C1,$03,$03,$06,$06` (ASM Tar_Monster.asm loc_325E6).
        let visuals = sample_level_visuals();
        let mut trace = sample_trace(GameMode::InGame);
        trace.active_projectile_details = vec![ActiveProjectileSnapshot {
            x_pos: 0x0030_0000,
            y_pos: 0x0020_0000,
            x_velocity: 0x0001_0000,
            y_velocity: -0x0001_0000,
            friendly: false,
            remaining_frames: 60,
            animation_elapsed_frames: 0,
            kind: ActiveProjectileKind::TarMonsterBurstChild,
            palette_variant: 0,
            boss_eye_sprite_mapping_index: 0,
            boss_eye_sprite_order_word_a: 0,
            dragon_flying_descent_fire_state: DragonFlyingDescentFireState::None,
            dragon_flying_descent_fire_counter: 0,
        }];

        let scene = build_scene(&trace, false, Some(&visuals), None);

        assert!(
            scene.commands.iter().any(|command| matches!(
                command,
                DrawCommand::Sprite { tiles, .. }
                    if tiles.iter().any(|tile| tile.tile.tile_index == 0x0C1)
            )),
            "ASM Tar_Monster.asm loc_325E6 should render burst fragments from ROM tile `unk_C782E` ($0C1)",
        );
        assert!(
            !scene.commands.iter().any(|command| matches!(
                command,
                DrawCommand::Rect { width: 4, height: 4, colour, .. } if *colour == [255, 80, 40]
            )),
            "Tar Monster burst fragments must not fall back to the generic red projectile dot",
        );
    }

    #[test]
    #[ignore = "requires generated .cache assets; run mise run extract-assets first"]
    fn blue_lake_woods_2_live_tank_bullet_renders_rom_child_sprite_loc_3b730() {
        let Some(manifest) = cached_generated_manifest() else {
            return;
        };
        let session_assets =
            load_native_session_assets(&manifest, 0x01).expect("Blue Lake Woods 2 session assets");
        let mut runtime = NativeRuntime::with_generated_level_layout(manifest);
        runtime.boot_to_level(LevelId(0x01));
        runtime.settle_recording_oracle_warp_live_window(LevelId(0x01));
        runtime.set_player_probe_position(1544, 383);
        runtime.set_camera_probe_position(1400, 240);

        let mut bullet_scene = None;
        for _ in 0..100 {
            let trace = runtime.step_frame(FrameInput::default());
            if let Some(projectile_y_pos) = trace
                .active_projectile_details
                .iter()
                .find(|projectile| {
                    projectile.kind == ActiveProjectileKind::TankBullet
                        && projectile.x_velocity != 0
                        && !projectile.friendly
                })
                .map(|projectile| projectile.y_pos)
            {
                let mut render_trace = trace;
                // ASM Tank_Shooting.asm loc_3B730 supplies the live projectile
                // state. Shift only the render viewport so this remains a
                // sprite-art assertion even when the public settled probe route
                // activates a lower off-screen tank.
                render_trace.camera_y_pos = projectile_y_pos.saturating_sub(120 << 16);
                bullet_scene = Some(build_scene(
                    &render_trace,
                    false,
                    Some(&session_assets.level_visuals),
                    runtime.block_grid(),
                ));
                break;
            }
        }

        let scene = bullet_scene
            .expect("ASM Tank_Shooting.asm loc_3C50E -> loc_3B730 should emit a live TankBullet");
        let (bullet_command_index, bullet_x, bullet_y, bullet_tiles) = scene
            .commands
            .iter()
            .enumerate()
            .find_map(|(index, command)| match command {
                DrawCommand::Sprite { x, y, tiles, .. }
                    if tiles.len() == 2
                        && matches!(tiles[0].tile.tile_index, 0x038 | 0x03A) =>
                {
                    Some((index, *x, *y, tiles))
                }
                _ => None,
            })
            .expect(
                "ASM Tank_Shooting.asm `stru_3B526` should render live `$0B` RockTank bullets with ROM child sprite frames",
            );

        let sample_pixels = bullet_tiles
            .iter()
            .flat_map(|tile| {
                let x0 = bullet_x + tile.offset_x;
                let y0 = bullet_y + tile.offset_y;
                (0..8).flat_map(move |dy| (0..8).map(move |dx| (x0 + dx, y0 + dy)))
            })
            .filter_map(|(x, y)| {
                (x >= 0 && y >= 0 && x < FRAME_WIDTH as i32 && y < FRAME_HEIGHT as i32)
                    .then_some((x as usize, y as usize))
            })
            .collect::<BTreeSet<_>>();
        let bullet_samples = sample_scene_pixel_provenance(&scene, &sample_pixels);
        let visible_bullet_pixels = bullet_samples
            .into_iter()
            .filter(|sample| {
                sample.writes.iter().any(|write| {
                    write.command_index == bullet_command_index
                        && write.accepted
                        && write.kind == "sprite_tile"
                        && matches!(write.tile_index, Some(0x038..=0x03B))
                        && write.rgb.is_some()
                })
            })
            .collect::<Vec<_>>();
        assert!(
            !visible_bullet_pixels.is_empty(),
            "ASM Tank_Shooting.asm `stru_3B526` bullet pixels should come from visible ROM sprite tiles, not a solid placeholder blob"
        );
        assert!(
            visible_bullet_pixels.iter().all(|sample| {
                sample.writes.iter().any(|write| {
                    write.command_index == bullet_command_index
                        && write.accepted
                        && write.rgb.is_some_and(|rgb| rgb != [100, 255, 100])
                })
            }),
            "hostile TankBullet pixels must not render with the friendly green projectile placeholder colour"
        );
        assert!(
            !scene.commands.iter().any(|command| matches!(
                command,
                DrawCommand::Rect {
                    width: 8,
                    height: 5,
                    colour,
                    ..
                } if *colour == [255, 80, 40]
            )),
            "live `$0B` RockTank bullets should not fall back to the generic enemy projectile dot"
        );
    }

    #[test]
    fn robot_shot_impact_uses_stru_3a762_sprite_frames_loc_3ac92() {
        let visuals = sample_level_visuals();
        let cases = [(15, 0x000, 24, 8), (12, 0x004, 18, 4), (8, 0x00D, 19, 2)];

        for (remaining_frames, tile_index, expected_x, expected_y) in cases {
            let mut trace = sample_trace(GameMode::InGame);
            trace.active_projectile_details = vec![ActiveProjectileSnapshot {
                x_pos: 0x0030_0000,
                y_pos: 0x0020_0000,
                x_velocity: 0,
                y_velocity: 0,
                friendly: false,
                remaining_frames,
                animation_elapsed_frames: 0,
                kind: ActiveProjectileKind::RobotShotImpact,
                palette_variant: 0,
                boss_eye_sprite_mapping_index: 0,
                boss_eye_sprite_order_word_a: 0,
                dragon_flying_descent_fire_state: DragonFlyingDescentFireState::None,
                dragon_flying_descent_fire_counter: 0,
            }];

            let scene = build_scene(&trace, false, Some(&visuals), None);

            let impact_sprite = scene.commands.iter().find_map(|command| match command {
                DrawCommand::Sprite { x, y, tiles, .. } if *x == expected_x && *y == expected_y => {
                    Some(tiles)
                }
                _ => None,
            });
            let tiles = impact_sprite
                .expect("ASM Robot.asm loc_3AC92 should render `stru_3A762` impact sprite frames");
            assert_eq!(
                tiles[0].tile.tile_index, tile_index,
                "ASM `stru_3A762` should select the impact frame for remaining={remaining_frames}",
            );
        }
    }

    #[test]
    fn robot_shot_uses_visible_rom_flying_sprite_loc_3ac0c_stru_3a754() {
        let visuals = sample_level_visuals();
        let cases = [
            (0, 0x016, 3, 18),
            (3, 0x01A, 3, 18),
            (6, 0x01E, 11, 18),
            (9, 0x016, 3, 18),
        ];

        for (animation_elapsed_frames, tile_index, expected_x, expected_y) in cases {
            let mut trace = sample_trace(GameMode::InGame);
            trace.active_projectile_details = vec![ActiveProjectileSnapshot {
                x_pos: 0x0030_0000,
                y_pos: 0x0020_0000,
                x_velocity: -(5 << 16),
                y_velocity: 0,
                friendly: false,
                remaining_frames: 60,
                animation_elapsed_frames,
                kind: ActiveProjectileKind::RobotShot,
                palette_variant: 0,
                boss_eye_sprite_mapping_index: 0,
                boss_eye_sprite_order_word_a: 0,
                dragon_flying_descent_fire_state: DragonFlyingDescentFireState::None,
                dragon_flying_descent_fire_counter: 0,
            }];

            let scene = build_scene(&trace, false, Some(&visuals), None);

            let shot_sprite = scene.commands.iter().find_map(|command| match command {
                DrawCommand::Sprite { x, y, tiles, .. } if *x == expected_x && *y == expected_y => {
                    Some(tiles)
                }
                _ => None,
            });
            let tiles = shot_sprite
                .expect("ASM Robot.asm loc_3AC0C should render the shot as `stru_3A754`");
            assert_eq!(
                tiles[0].tile.tile_index, tile_index,
                "ASM `stru_3A754` should select the flying-shot frame for elapsed={animation_elapsed_frames}",
            );
            assert!(
                !has_rect(&scene, 30, 22, 4, 4, [255, 80, 40]),
                "Robot shots should not collapse to the generic enemy projectile dot",
            );
        }
    }

    #[test]
    fn fire_demon_death_trail_falls_back_to_placeholder_without_rom_visuals() {
        // Removing the Fire Demon visuals forces the degraded-asset path.
        // The previous flame-footprint approximation invented per-phase rect
        // geometry; the simplified fallback emits a single placeholder so
        // missing-asset states stay visible without claiming ROM shapes.
        let mut visuals = sample_level_visuals();
        visuals.enemies.families.remove(&0x00);
        let mut trace = sample_trace(GameMode::InGame);
        trace.active_projectiles = vec![(0x0030_0000, 0x0020_0000, false)];
        trace.active_projectile_details = vec![ActiveProjectileSnapshot {
            x_pos: 0x0030_0000,
            y_pos: 0x0020_0000,
            x_velocity: 0,
            y_velocity: 0,
            friendly: false,
            remaining_frames: 45,
            animation_elapsed_frames: 0,
            kind: ActiveProjectileKind::FireDemonDeathTrail,
            palette_variant: 0,
            boss_eye_sprite_mapping_index: 0,
            boss_eye_sprite_order_word_a: 0,
            dragon_flying_descent_fire_state: DragonFlyingDescentFireState::None,
            dragon_flying_descent_fire_counter: 0,
        }];

        let scene = build_scene(&trace, false, Some(&visuals), None);

        // Trace at world (48, 32); sample_trace camera at (16, 8), so the
        // projectile lands at screen (32, 24). The fallback placeholder draws
        // a 4×5 rect anchored at (sx-2, sy-3) = (30, 21).
        assert!(
            has_rect(&scene, 30, 21, 4, 5, [255, 120, 32]),
            "missing-asset fallback should emit a single warm-orange placeholder rect at the trail position",
        );
        assert!(
            !has_rect(&scene, 30, 22, 4, 4, [255, 80, 40]),
            "fire trail should never collapse to the generic projectile dot",
        );
    }

    #[test]
    fn fire_demon_death_trail_uses_rom_sprite_frame_loc_3a63c_stru_3a622() {
        let visuals = sample_level_visuals();
        let mut trace = sample_trace(GameMode::InGame);
        trace.active_projectile_details = vec![ActiveProjectileSnapshot {
            x_pos: 0x0030_0000,
            y_pos: 0x0020_0000,
            x_velocity: 0,
            y_velocity: 0,
            friendly: false,
            remaining_frames: 45,
            animation_elapsed_frames: 0,
            kind: ActiveProjectileKind::FireDemonDeathTrail,
            palette_variant: 0,
            boss_eye_sprite_mapping_index: 0,
            boss_eye_sprite_order_word_a: 0,
            dragon_flying_descent_fire_state: DragonFlyingDescentFireState::None,
            dragon_flying_descent_fire_counter: 0,
        }];

        let scene = build_scene(&trace, false, Some(&visuals), None);

        let trail_sprite = scene.commands.iter().find_map(|command| match command {
            DrawCommand::Sprite { x, y, tiles, .. } if *x == 25 && *y == 16 => Some(tiles),
            _ => None,
        });
        let tiles = trail_sprite.expect("ASM loc_3A63C should render the trail as a ROM sprite");
        assert_eq!(
            tiles[0].tile.tile_index, 0x0B0,
            "ASM stru_3A622 first frame uses `unk_C7776: sprite_frame_vram $0B0,$07,$00,$0E,$08`",
        );
        assert!(
            !has_rect(&scene, 31, 20, 2, 2, [255, 120, 32]),
            "the rectangle approximation should disappear once the ROM sprite frame is wired",
        );
    }

    #[test]
    fn dragon_walking_breath_uses_rom_child_sprite_frame_loc_3d02a_stru_3ccc8() {
        let visuals = sample_level_visuals();
        let mut trace = sample_trace(GameMode::InGame);
        trace.active_projectile_details = vec![ActiveProjectileSnapshot {
            x_pos: 0x0030_0000,
            y_pos: 0x0020_0000,
            x_velocity: 0,
            y_velocity: 0,
            friendly: false,
            remaining_frames: 68,
            animation_elapsed_frames: 0,
            kind: ActiveProjectileKind::DragonWalkingBreath,
            palette_variant: 0,
            boss_eye_sprite_mapping_index: 0,
            boss_eye_sprite_order_word_a: 0,
            dragon_flying_descent_fire_state: DragonFlyingDescentFireState::None,
            dragon_flying_descent_fire_counter: 0,
        }];

        let scene = build_scene(&trace, false, Some(&visuals), None);

        let breath_sprite = scene.commands.iter().find_map(|command| match command {
            DrawCommand::Sprite { x, y, tiles, .. } if *x == 21 && *y == 5 => Some(tiles),
            _ => None,
        });
        let tiles = breath_sprite
            .expect("ASM Dragon.asm loc_3D02A should render breath as a ROM sprite child");
        assert_eq!(
            tiles[0].tile.tile_index, 0x028,
            "ASM Dragon.asm `stru_3CCC8` first frame uses `unk_C7576: sprite_frame_vram $028,$0B,$00,$1B,$13`",
        );
        assert!(
            !has_rect(&scene, 30, 22, 4, 4, [255, 80, 40]),
            "walking-dragon breath should not render as the generic enemy projectile dot",
        );
    }

    #[test]
    fn dragon_flying_descent_fire_uses_rom_child_sprite_frames_loc_34bac() {
        let visuals = sample_level_visuals();
        let cases = [
            (
                DragonFlyingDescentFireState::FirstAnimationLoc34C26,
                0x0EC,
                27,
                5,
                "unk_C75CE",
            ),
            (
                DragonFlyingDescentFireState::TravelAnimationLoc34C3A,
                0x0F5,
                26,
                0,
                "unk_C75D6",
            ),
            (
                DragonFlyingDescentFireState::CleanupFirstLoc34C4A,
                0x105,
                19,
                2,
                "unk_C75DE",
            ),
            (
                DragonFlyingDescentFireState::CleanupSecondLoc34C4A,
                0x115,
                17,
                13,
                "unk_C75E6",
            ),
        ];

        for (state, tile_index, expected_x, expected_y, label) in cases {
            let mut trace = sample_trace(GameMode::InGame);
            trace.active_projectile_details = vec![ActiveProjectileSnapshot {
                x_pos: 0x0030_0000,
                y_pos: 0x0020_0000,
                x_velocity: 1 << 16,
                y_velocity: 3 << 16,
                friendly: false,
                remaining_frames: 0,
                animation_elapsed_frames: 0,
                kind: ActiveProjectileKind::DragonFlyingDescentFire,
                palette_variant: 0,
                boss_eye_sprite_mapping_index: 0,
                boss_eye_sprite_order_word_a: 0,
                dragon_flying_descent_fire_state: state,
                dragon_flying_descent_fire_counter: 0,
            }];

            let scene = build_scene(&trace, false, Some(&visuals), None);

            let fire_sprite = scene.commands.iter().find_map(|command| match command {
                DrawCommand::Sprite { x, y, tiles, .. }
                    if tiles
                        .first()
                        .is_some_and(|tile| tile.tile.tile_index == tile_index) =>
                {
                    Some((*x, *y, tiles))
                }
                _ => None,
            });
            let (x, y, tiles) = fire_sprite
                .expect("ASM Dragon_Flying.asm loc_34BAC should render descent fire as ROM child sprite frames");
            assert_eq!(
                (x, y),
                (expected_x, expected_y),
                "ASM Dragon_Flying.asm {label}: descent fire ROM frame should keep the expected origin",
            );
            assert_eq!(
                tiles[0].tile.tile_index, tile_index,
                "ASM Dragon_Flying.asm {label}: descent fire should select the ROM frame descriptor",
            );
            assert!(
                !has_rect(&scene, 29, 21, 6, 6, [255, 110, 30]),
                "Flying Dragon descent fire should not collapse to the placeholder rect",
            );
        }
    }

    #[test]
    fn dragon_walking_breath_copies_parent_x_direction_loc_3d02a() {
        fn trace_with_dragon_and_breath(dragon_x_px: u32) -> NativeTraceFrame {
            let mut trace = sample_trace(GameMode::InGame);
            trace.active_objects = vec![NativeObjectSnapshot {
                kind: NativeObjectKind::Enemy { object_type: 0x0D },
                x_pos: dragon_x_px << 16,
                y_pos: 0x0020_0000,
                layout_index: Some(0),
                x_velocity: Some(0),
                y_velocity: Some(0),
                facing: Some(Direction::Left),
                on_ground: Some(true),
                hit_points: Some(1),
                damage_flash: false,
                palette_variant: 0,
                stationary: false,
                contact_phase: None,
                shared_walker_state: None,
                shared_walker_patrol_timer: None,
                shared_walker_patrol_short_next: None,
                walk_phase: None,
                wait_timer: None,
                debug_fields: std::collections::BTreeMap::new(),
            }];
            trace.active_projectile_details = vec![ActiveProjectileSnapshot {
                x_pos: 0x0030_0000,
                y_pos: 0x0020_0000,
                x_velocity: 0,
                y_velocity: 0,
                friendly: false,
                remaining_frames: 68,
                animation_elapsed_frames: 0,
                kind: ActiveProjectileKind::DragonWalkingBreath,
                palette_variant: 0,
                boss_eye_sprite_mapping_index: 0,
                boss_eye_sprite_order_word_a: 0,
                dragon_flying_descent_fire_state: DragonFlyingDescentFireState::None,
                dragon_flying_descent_fire_counter: 0,
            }];
            trace
        }

        let visuals = sample_level_visuals();
        let left_breath = build_scene(
            &trace_with_dragon_and_breath(0x0030 + 0x001C),
            false,
            Some(&visuals),
            None,
        );
        let right_breath = build_scene(
            &trace_with_dragon_and_breath(0x0030 - 0x001C),
            false,
            Some(&visuals),
            None,
        );

        let left_sprite = left_breath
            .commands
            .iter()
            .find(|command| {
                matches!(
                    command,
                    DrawCommand::Sprite { tiles, .. } if tiles[0].tile.tile_index == 0x028
                )
            })
            .expect("ASM Dragon.asm loc_3D02A left breath child should draw the ROM frame");
        let right_sprite = right_breath
            .commands
            .iter()
            .find(|command| {
                matches!(
                    command,
                    DrawCommand::Sprite { tiles, .. } if tiles[0].tile.tile_index == 0x028
                )
            })
            .expect("ASM Dragon.asm loc_3D02A right breath child should draw the ROM frame");

        assert!(
            !sprite_tiles(left_sprite)[0].tile.flip_x,
            "ASM loc_3D02A copies clear x_direction for the -$1C child; left-facing dragon art remains unflipped",
        );
        assert!(
            sprite_tiles(right_sprite)[0].tile.flip_x,
            "ASM loc_3D02A copies flipped x_direction for the +$1C child; native must mirror the breath sprite",
        );
    }

    #[test]
    fn drips_drop_uses_rom_child_sprite_frame_loc_33424() {
        let mut visuals = sample_level_visuals();
        let variant_palette = Palette {
            colours: vec![
                colour(0, 0, 0),
                colour(5, 10, 15),
                colour(10, 20, 30),
                colour(15, 30, 45),
                colour(20, 40, 60),
                colour(25, 50, 75),
                colour(30, 60, 90),
                colour(35, 70, 105),
                colour(40, 80, 120),
            ],
        };
        visuals
            .enemies
            .families
            .get_mut(&0x16)
            .expect("sample visuals include Drips")
            .extra_palettes
            .push(variant_palette.clone());
        let mut trace = sample_trace(GameMode::InGame);
        trace.active_projectile_details = vec![ActiveProjectileSnapshot {
            x_pos: 0x0030_0000,
            y_pos: 0x0020_0000,
            x_velocity: 0,
            y_velocity: 0,
            friendly: false,
            remaining_frames: 200,
            animation_elapsed_frames: 0,
            kind: ActiveProjectileKind::DripsDrop,
            palette_variant: 1,
            boss_eye_sprite_mapping_index: 0,
            boss_eye_sprite_order_word_a: 0,
            dragon_flying_descent_fire_state: DragonFlyingDescentFireState::None,
            dragon_flying_descent_fire_counter: 0,
        }];

        let scene = build_scene(&trace, false, Some(&visuals), None);

        let drop_sprite = scene.commands.iter().find(|command| match command {
            DrawCommand::Sprite { x, y, .. } => *x == 30 && *y == 23,
            _ => false,
        });
        let drop_sprite =
            drop_sprite.expect("ASM Drips.asm loc_33424 should render the falling child");
        let tiles = sprite_tiles(drop_sprite);
        assert_eq!(
            tiles[0].tile.tile_index, 0x082,
            "ASM `unk_C7B26: sprite_frame_vram $082,$02,$09,$05,$0A` is the falling-drop child frame",
        );
        assert_eq!(
            sprite_palette(drop_sprite),
            &variant_palette,
            "ASM Drips.asm loc_33424 reuses the Drip child object, whose palette line was set by loc_32146 from the parent enemy slot"
        );
        assert!(
            !has_rect(&scene, 30, 22, 4, 5, [120, 200, 90]),
            "the green rectangle approximation should disappear once the ROM child sprite is wired",
        );
    }

    #[test]
    fn drips_surface_crawler_uses_loc_335d8_moving_frames() {
        let visuals = sample_level_visuals();
        let mut trace = sample_trace(GameMode::InGame);
        trace.active_projectile_details = vec![ActiveProjectileSnapshot {
            x_pos: 0x0030_0000,
            y_pos: 0x0020_0000,
            x_velocity: 0,
            y_velocity: 0,
            friendly: false,
            remaining_frames: u16::MAX,
            animation_elapsed_frames: 0,
            kind: ActiveProjectileKind::DripsSurfaceCrawler,
            palette_variant: 0,
            boss_eye_sprite_mapping_index: 0,
            boss_eye_sprite_order_word_a: 0,
            dragon_flying_descent_fire_state: DragonFlyingDescentFireState::None,
            dragon_flying_descent_fire_counter: 0,
        }];

        let scene = build_scene(&trace, false, Some(&visuals), None);

        let crawler_sprite = scene.commands.iter().find_map(|command| match command {
            DrawCommand::Sprite { tiles, .. } if tiles[0].tile.tile_index == 0x08D => Some(tiles),
            _ => None,
        });
        let tiles =
            crawler_sprite.expect("ASM Drips.asm loc_335D8 should render moving crawler frames");
        assert_eq!(
            tiles[0].tile.tile_index, 0x08D,
            "ASM `stru_33F14` starts with `unk_C7B46: sprite_frame_vram $08D,$07,$00,$0F,$0F`, not the tiny off_33520 pool frame",
        );
        assert!(
            scene.commands.iter().all(|command| {
                !matches!(command, DrawCommand::Sprite { tiles, .. } if tiles[0].tile.tile_index == 0x0B9)
            }),
            "moving surface crawlers should not reuse the tiny off_33520 pool/surface frame",
        );
    }

    #[test]
    fn drips_surface_crawler_startup_uses_stru_33fa6_frames() {
        let visuals = sample_level_visuals();
        let mut trace = sample_trace(GameMode::InGame);
        trace.active_projectile_details = vec![ActiveProjectileSnapshot {
            x_pos: 0x0030_0000,
            y_pos: 0x0020_0000,
            x_velocity: 0,
            y_velocity: 0,
            friendly: false,
            remaining_frames: 200,
            animation_elapsed_frames: 31,
            kind: ActiveProjectileKind::DripsSurfaceCrawler,
            palette_variant: 0,
            boss_eye_sprite_mapping_index: 0,
            boss_eye_sprite_order_word_a: 0,
            dragon_flying_descent_fire_state: DragonFlyingDescentFireState::None,
            dragon_flying_descent_fire_counter: 0,
        }];

        let scene = build_scene(&trace, false, Some(&visuals), None);

        let crawler_sprite = scene.commands.iter().find_map(|command| match command {
            DrawCommand::Sprite { tiles, .. } if tiles[0].tile.tile_index == 0x0BD => Some(tiles),
            _ => None,
        });
        let tiles =
            crawler_sprite.expect("ASM Drips.asm loc_335D8 should render stru_33FA6 startup");
        assert_eq!(
            tiles[0].tile.tile_index, 0x0BD,
            "ASM `stru_33FA6` starts with `unk_C7BAE: sprite_frame_vram $0BD,$08,$00,$0F,$04`",
        );
    }

    #[test]
    fn drips_surface_crawler_finish_uses_stru_33f06_not_tar_blob() {
        let visuals = sample_level_visuals();
        let mut trace = sample_trace(GameMode::InGame);
        trace.active_projectile_details = vec![ActiveProjectileSnapshot {
            x_pos: 0x0030_0000,
            y_pos: 0x0020_0000,
            x_velocity: 0,
            y_velocity: 0,
            friendly: false,
            remaining_frames: 60,
            animation_elapsed_frames: 0,
            kind: ActiveProjectileKind::DripsSurfaceCrawlerFinish,
            palette_variant: 0,
            boss_eye_sprite_mapping_index: 0,
            boss_eye_sprite_order_word_a: 0,
            dragon_flying_descent_fire_state: DragonFlyingDescentFireState::None,
            dragon_flying_descent_fire_counter: 0,
        }];

        let scene = build_scene(&trace, false, Some(&visuals), None);

        let finish_sprite = scene.commands.iter().find_map(|command| match command {
            DrawCommand::Sprite { tiles, .. } if tiles[0].tile.tile_index == 0x0C5 => Some(tiles),
            _ => None,
        });
        let tiles = finish_sprite.expect(
            "ASM Drips.asm loc_338F8/loc_3392C should render `stru_33F06` on the original crawler",
        );
        assert_eq!(
            tiles[0].tile.tile_index, 0x0C5,
            "ASM `stru_33F06` starts with `unk_C7BC6: sprite_frame_vram $0C5,$07,$07,$10,$10`",
        );
        assert!(
            scene.commands.iter().all(|command| {
                !matches!(command, DrawCommand::Sprite { tiles, .. } if tiles[0].tile.tile_index == 0x0C1)
            }),
            "the original crawler finish must not reuse Tar_Monster.asm loc_325E6's tiny `$0C1` burst fragment",
        );
    }

    #[test]
    fn drips_parent_controller_is_not_rendered_as_floating_slime() {
        let visuals = sample_level_visuals();
        let drips_palette = visuals
            .enemies
            .families
            .get(&0x16)
            .expect("sample visuals include Drips")
            .palette
            .clone();
        let mut trace = sample_trace(GameMode::InGame);
        trace.active_objects = vec![NativeObjectSnapshot {
            kind: NativeObjectKind::Enemy { object_type: 0x16 },
            x_pos: 0x0030_0000,
            y_pos: 0x0020_0000,
            layout_index: Some(0),
            x_velocity: Some(0),
            y_velocity: Some(-0x0001_0000),
            facing: Some(Direction::Right),
            on_ground: Some(false),
            hit_points: Some(1),
            damage_flash: false,
            palette_variant: 0,
            stationary: false,
            contact_phase: None,
            shared_walker_state: None,
            shared_walker_patrol_timer: None,
            shared_walker_patrol_short_next: None,
            walk_phase: None,
            wait_timer: None,
            debug_fields: std::collections::BTreeMap::new(),
        }];

        let scene = build_scene(&trace, false, Some(&visuals), None);

        assert!(
            scene.commands.iter().all(|command| {
                !matches!(
                    command,
                    DrawCommand::Sprite { palette, tiles, .. }
                        if **palette == drips_palette
                            && tiles
                                .iter()
                                .any(|tile| matches!(tile.tile.tile_index, 0x000 | 0x004 | 0x008))
                )
            }),
            "ASM Drips.asm keeps the parent/controller gfx object hidden via `$13`; native should not render it as a floating slime",
        );
    }

    #[test]
    fn level_complete_overlay_uses_eol_sprite_workers_loc_d55e() {
        let visuals = sample_level_visuals();
        let mut trace = sample_trace(GameMode::InGame);
        trace.level_complete = true;
        trace.level_timer_seconds = 180;
        trace.level_damage_taken = false;
        trace.level_prize_collected = false;
        trace.level_path_bonus_amount = 3_000;
        trace.level_speed_bonus_awarded = true;
        trace.level_complete_score_rows_elapsed_frames = 9;
        trace.score = 12_340;

        let scene = build_scene(&trace, false, Some(&visuals), None);

        assert!(
            has_sprite_signature(&scene, 128),
            "ASM loc_D55E should render Spr_EOL_Time instead of typed TIME text after the loc_D734 visible fade phase",
        );
        assert!(
            has_sprite_signature(&scene, 121),
            "ASM loc_D55E should render Spr_EOL_Bonus label sprites",
        );
        assert!(
            has_sprite_signature(&scene, 120),
            "ASM loc_D4CE should render the Spr_EOL_Continues row before score and bonus rows",
        );
        assert!(
            has_sprite_signature(&scene, 131),
            "ASM Object_EOL_Number should render digit sprite workers",
        );
        assert!(!contains_text(&scene, "TIME BONUS"));
        assert!(!contains_text(&scene, "SCORE:"));
    }

    #[test]
    fn level_complete_draws_hologram_scroll_b_after_flag_hibernate() {
        let visuals = sample_level_visuals();
        let mut trace = sample_trace(GameMode::InGame);
        trace.level_complete = true;
        trace.level_complete_elapsed_frames =
            wildside_core::EOL_HOLOGRAM_VISIBLE_FRAME_LOC_C1D0_ORACLE;

        let scene = build_scene(&trace, false, Some(&visuals), None);

        assert_eq!(scene.background, [0, 0, 0]);
        assert!(
            scene.commands.iter().any(|command| matches!(
                command,
                DrawCommand::Rect {
                    x: 0,
                    y: 32,
                    width,
                    height: 2,
                    colour: [255, 112, 0],
                } if *width == NATIVE_FRAME_WIDTH as i32
            )),
            "ASM loc_C186 should draw the orange EOL hologram room after the flag hibernate"
        );
    }

    #[test]
    fn level_complete_hologram_uses_mapeni_cc0e_tiles_when_loaded_and_keeps_scroll_a() {
        let mut visuals = sample_level_visuals();
        let mut tile_sheet = blank_tile_sheet(2);
        tile_sheet.tiles[1].pixels = [1; 64];
        visuals.eol_hologram = Some(wildside_render::EolHologramVisuals {
            palette: Palette {
                colours: vec![colour(255, 112, 0)],
            },
            tile_sheet,
            rows: vec![vec![TileRef::from_attr_word(0x0001)]],
        });
        let mut trace = sample_trace(GameMode::InGame);
        trace.level_complete = true;
        trace.level_complete_elapsed_frames =
            wildside_core::EOL_HOLOGRAM_VISIBLE_FRAME_LOC_C1D0_ORACLE;

        let scene = build_scene(&trace, false, Some(&visuals), None);

        assert!(
            scene.commands.iter().any(|command| matches!(
                command,
                DrawCommand::IndexedTile { x: 0, y: 0, tile, .. } if tile.tile_index == 1
            )),
            "ASM loc_C186 should draw decoded MapEni_CC0E tile words when EOL hologram art is loaded"
        );
        assert!(
            scene.commands.iter().any(|command| matches!(
                command,
                DrawCommand::IndexedTile { tile, .. } if tile.tile_index == 12
            )),
            "ASM loc_C186 writes Scroll B only; existing Scroll A terrain must stay visible over the hologram"
        );
    }

    #[test]
    fn level_complete_hologram_uses_loc_d734_dimmed_palette_after_score_fade() {
        let mut visuals = sample_level_visuals();
        let mut tile_sheet = blank_tile_sheet(3);
        tile_sheet.tiles[1].pixels = [1; 64];
        tile_sheet.tiles[2].pixels = [2; 64];
        visuals.eol_hologram = Some(wildside_render::EolHologramVisuals {
            palette: Palette {
                colours: vec![
                    wildside_render::MdColour::from_word(0x0866),
                    wildside_render::MdColour::from_word(0x00E0),
                ],
            },
            tile_sheet,
            rows: vec![vec![
                TileRef::from_attr_word(0x0001),
                TileRef::from_attr_word(0x0002),
            ]],
        });
        let mut trace = sample_trace(GameMode::InGame);
        trace.level_complete = true;
        trace.level_complete_elapsed_frames =
            wildside_core::EOL_HOLOGRAM_VISIBLE_FRAME_LOC_C1D0_ORACLE;
        trace.level_complete_score_rows_elapsed_frames =
            super::scene::EOL_SCORE_ROWS_RENDER_FADE_IN_FRAMES_LOC_D734_ORACLE;

        let scene = build_scene(&trace, false, Some(&visuals), None);
        let dimmed_palette_word = scene.commands.iter().find_map(|command| match command {
            DrawCommand::IndexedTile { palette, tile, .. } if tile.tile_index == 1 => {
                palette.colours.first().map(|colour| colour.raw_word)
            }
            _ => None,
        });
        let bright_grid_word = scene.commands.iter().find_map(|command| match command {
            DrawCommand::IndexedTile { palette, tile, .. } if tile.tile_index == 2 => {
                palette.colours.get(1).map(|colour| colour.raw_word)
            }
            _ => None,
        });

        assert_eq!(
            dimmed_palette_word,
            Some(0x0433),
            "ASM loc_D734/sub_14C0 should keep the EOL hologram on the oracle-visible dim palette during the score counter"
        );
        assert_eq!(
            bright_grid_word,
            Some(0x00E0),
            "oracle frames show the Pal_D00C green grid entry remains bright while the loc_D734-dimmed score-room entries fade"
        );
    }

    #[test]
    fn level_complete_hologram_suppresses_scroll_a_background_tiles() {
        let mut visuals = sample_level_visuals();
        let mut hologram_tiles = blank_tile_sheet(2);
        hologram_tiles.tiles[1].pixels = [1; 64];
        visuals.eol_hologram = Some(wildside_render::EolHologramVisuals {
            palette: Palette {
                colours: vec![colour(255, 112, 0)],
            },
            tile_sheet: hologram_tiles,
            rows: vec![vec![TileRef::from_attr_word(0x0001)]],
        });
        let mut trace = sample_trace(GameMode::InGame);
        trace.level_complete = true;
        trace.level_complete_elapsed_frames =
            wildside_core::EOL_HOLOGRAM_VISIBLE_FRAME_LOC_C1D0_ORACLE;
        trace.camera_x_pos = 0;
        trace.camera_y_pos = 0;

        let scene = build_scene(&trace, false, Some(&visuals), None);

        assert!(
            !scene.commands.iter().any(|command| matches!(
                command,
                DrawCommand::IndexedTile { x: 0, y: 0, tile_sheet, tile, .. }
                    if tile_sheet.tiles.len() == visuals.background_tiles.tiles.len()
                        && tile.tile_index == 0
            )),
            "ASM loc_C186 installs the fixed EOL room; native should not redraw the ordinary level background over the score-room Scroll B image"
        );
    }

    #[test]
    fn level_complete_replays_sub_c254_foreground_reveal_writes() {
        let mut visuals = sample_level_visuals();
        let mut tile_sheet = blank_tile_sheet(0x0420);
        tile_sheet.tiles[0x0417].pixels = [1; 64];
        visuals.eol_hologram = Some(wildside_render::EolHologramVisuals {
            palette: Palette {
                colours: vec![colour(255, 112, 0)],
            },
            tile_sheet,
            rows: vec![vec![TileRef::from_attr_word(0x0001)]],
        });
        let mut grid = BlockGrid::from_level_layout(&[0; 4 * 4 * 2], 4, 4);
        grid.set_cell_at(
            1,
            0,
            wildside_core::terrain::Cell::from_be_bytes(0xE1, 0x00),
        );
        let mut trace = sample_trace(GameMode::InGame);
        trace.level_complete = true;
        trace.level_complete_elapsed_frames =
            wildside_core::EOL_HOLOGRAM_VISIBLE_FRAME_LOC_C1D0_ORACLE
                + wildside_core::EOL_FOREGROUND_REVEAL_VISIBLE_DELAY_FRAMES_LOC_C21E_ORACLE
                + 1;
        trace.camera_x_pos = 0;
        trace.camera_y_pos = 0;

        let scene = build_scene(&trace, false, Some(&visuals), Some(&grid));

        assert!(
            scene.commands.iter().any(|command| matches!(
                command,
                DrawCommand::IndexedTile { x: 16, y: 0, tile, .. } if tile.tile_index == 0x0417
            )),
            "ASM loc_C21E/sub_C254 should overlay the first revealed block cell over Scroll A"
        );
    }

    #[test]
    fn level_complete_reveal_redraws_hologram_on_scroll_b_tile_grid() {
        let mut visuals = sample_level_visuals();
        let mut tile_sheet = blank_tile_sheet(0x0420);
        tile_sheet.tiles[1].pixels = [1; 64];
        visuals.eol_hologram = Some(wildside_render::EolHologramVisuals {
            palette: Palette {
                colours: vec![colour(255, 112, 0)],
            },
            tile_sheet,
            rows: vec![
                vec![TileRef::from_attr_word(0x0001); 2],
                vec![TileRef::from_attr_word(0x0001); 2],
            ],
        });
        let mut grid = BlockGrid::from_level_layout(&[0; 4 * 4 * 2], 4, 4);
        grid.set_cell_at(
            0,
            0,
            wildside_core::terrain::Cell::from_be_bytes(0xE1, 0x00),
        );
        let mut trace = sample_trace(GameMode::InGame);
        trace.level_complete = true;
        trace.level_complete_elapsed_frames =
            wildside_core::EOL_HOLOGRAM_VISIBLE_FRAME_LOC_C1D0_ORACLE
                + wildside_core::EOL_FOREGROUND_REVEAL_VISIBLE_DELAY_FRAMES_LOC_C21E_ORACLE
                + 1;
        trace.camera_x_pos = 3 << 16;
        trace.camera_y_pos = 1 << 16;

        let scene = build_scene(&trace, false, Some(&visuals), Some(&grid));
        let screen_aligned_hologram_tile_count = scene
            .commands
            .iter()
            .filter(|command| {
                matches!(
                    command,
                    DrawCommand::IndexedTileSlice { x: 0, tile, .. } if tile.tile_index == 1
                )
            })
            .count();

        assert!(
            screen_aligned_hologram_tile_count > 0,
            "ASM loc_C186 keeps the hologram on Scroll B screen tile coordinates; loc_C21E/sub_C254 reveal redraws must not inherit Scroll A's camera sub-tile offset"
        );
    }

    #[test]
    fn level_complete_reveal_keeps_hologram_redraw_above_clear_loc_c21e() {
        let mut visuals = sample_level_visuals();
        let mut tile_sheet = blank_tile_sheet(0x0420);
        tile_sheet.tiles[1].pixels = [1; 64];
        tile_sheet.tiles[0x0417].pixels = [2; 64];
        visuals.eol_hologram = Some(wildside_render::EolHologramVisuals {
            palette: Palette {
                colours: vec![colour(255, 112, 0), colour(144, 152, 184)],
            },
            tile_sheet,
            rows: vec![vec![TileRef::from_attr_word(0x0001); 2]],
        });
        let mut grid = BlockGrid::from_level_layout(&[0; 4 * 4 * 2], 4, 4);
        grid.set_cell_at(
            0,
            0,
            wildside_core::terrain::Cell::from_be_bytes(0xE1, 0x00),
        );
        let mut trace = sample_trace(GameMode::InGame);
        trace.level_complete = true;
        trace.level_complete_elapsed_frames =
            wildside_core::EOL_HOLOGRAM_VISIBLE_FRAME_LOC_C1D0_ORACLE
                + wildside_core::EOL_FOREGROUND_REVEAL_VISIBLE_DELAY_FRAMES_LOC_C21E_ORACLE
                + 1;
        trace.camera_x_pos = 0;
        trace.camera_y_pos = 0;

        let scene = build_scene(&trace, false, Some(&visuals), Some(&grid));
        let clear_index = scene
            .commands
            .iter()
            .position(|command| {
                matches!(
                    command,
                    DrawCommand::Rect {
                        x: 0,
                        y: 0,
                        width: 16,
                        height: 16,
                        colour: [0, 0, 0],
                    }
                )
            })
            .expect("EOL foreground clear");
        let hologram_redraw_index = scene
            .commands
            .iter()
            .enumerate()
            .skip(clear_index + 1)
            .find_map(|(index, command)| {
                matches!(
                    command,
                    DrawCommand::IndexedTileSlice { x: 0, y: 0, tile, .. } if tile.tile_index == 1
                )
                .then_some(index)
            })
            .expect("EOL hologram redraw");
        let foreground_index = scene
            .commands
            .iter()
            .enumerate()
            .skip(hologram_redraw_index + 1)
            .find_map(|(index, command)| {
                matches!(
                    command,
                    DrawCommand::IndexedTile { x: 0, y: 0, tile, .. } if tile.tile_index == 0x0417
                )
                .then_some(index)
            })
            .expect("EOL foreground write");

        assert!(
            clear_index < hologram_redraw_index && hologram_redraw_index < foreground_index,
            "ASM loc_C186 Scroll B redraw must sit above the clear and below loc_C21E/sub_C254 foreground tiles"
        );
    }

    #[test]
    fn level_complete_reveal_clips_hologram_redraw_to_scroll_a_cell_loc_c21e() {
        let mut visuals = sample_level_visuals();
        let mut tile_sheet = blank_tile_sheet(0x0420);
        tile_sheet.tiles[1].pixels = [1; 64];
        visuals.eol_hologram = Some(wildside_render::EolHologramVisuals {
            palette: Palette {
                colours: vec![colour(180, 0, 0)],
            },
            tile_sheet,
            rows: vec![vec![TileRef::from_attr_word(0x0001); 2]; 2],
        });
        visuals.block_tiles = blank_tile_sheet(0x0230);
        for tile in &mut visuals.block_tiles.tiles {
            tile.pixels = [2; 64];
        }
        visuals.palettes.foreground = Palette {
            colours: vec![colour(0, 0, 0), colour(0, 220, 0)],
        };
        let mut grid = BlockGrid::from_level_layout(&[0; 4 * 4 * 2], 4, 4);
        grid.set_cell_at(0, 1, Cell::from_be_bytes(0xE1, 0x00));
        let mut trace = sample_trace(GameMode::InGame);
        trace.level_complete = true;
        trace.level_complete_elapsed_frames =
            wildside_core::EOL_HOLOGRAM_VISIBLE_FRAME_LOC_C1D0_ORACLE
                + wildside_core::EOL_FOREGROUND_REVEAL_VISIBLE_DELAY_FRAMES_LOC_C21E_ORACLE
                + 1;
        trace.camera_x_pos = 0;
        trace.camera_y_pos = 1 << 16;

        let scene = build_scene(&trace, false, Some(&visuals), Some(&grid));
        let mut rgb = vec![0; (NATIVE_FRAME_WIDTH * NATIVE_FRAME_HEIGHT * 3) as usize];
        render_scene(&scene, &mut rgb);

        assert_eq!(
            pixel_at(&rgb, 0, 15),
            [0, 220, 0],
            "ASM loc_C21E/sub_C254 clips each 16px foreground update cell; the Scroll B redraw must not leak into the first scanline of the next unrevealed Plane A cell"
        );
    }

    #[test]
    fn level_complete_reveal_foreground_overrides_priority_hologram_pixels_loc_c21e() {
        let mut visuals = sample_level_visuals();
        let mut tile_sheet = blank_tile_sheet(0x0420);
        tile_sheet.tiles[1].pixels = [1; 64];
        tile_sheet.tiles[0x041A].pixels = [2; 64];
        visuals.eol_hologram = Some(wildside_render::EolHologramVisuals {
            palette: Palette {
                colours: vec![colour(255, 219, 0), colour(146, 146, 182)],
            },
            tile_sheet,
            rows: vec![vec![
                TileRef {
                    tile_index: 1,
                    palette_line: 0,
                    priority: true,
                    flip_x: false,
                    flip_y: false,
                };
                2
            ]],
        });
        let mut grid = BlockGrid::from_level_layout(&[0; 4 * 4 * 2], 4, 4);
        grid.set_cell_at(0, 0, Cell::from_be_bytes(0xE1, 0x00));
        let mut trace = sample_trace(GameMode::InGame);
        trace.level_complete = true;
        trace.level_complete_elapsed_frames =
            wildside_core::EOL_HOLOGRAM_VISIBLE_FRAME_LOC_C1D0_ORACLE
                + wildside_core::EOL_FOREGROUND_REVEAL_VISIBLE_DELAY_FRAMES_LOC_C21E_ORACLE
                + 1;
        trace.camera_x_pos = 0;
        trace.camera_y_pos = 0;

        let scene = build_scene(&trace, false, Some(&visuals), Some(&grid));
        let mut rgb = vec![0; (NATIVE_FRAME_WIDTH * NATIVE_FRAME_HEIGHT * 3) as usize];
        render_scene(&scene, &mut rgb);
        let sample = pixel_at(&rgb, 8, 8);

        assert_eq!(
            sample,
            [146, 146, 182],
            "ASM loc_C21E/sub_C254 replaces Scroll A cells, so foreground reveal pixels must draw above the loc_C186 Scroll B redraw even when that redraw tile has priority"
        );
    }

    #[test]
    fn level_complete_speed_under_seconds_uses_loc_d660_threshold() {
        let visuals = sample_level_visuals();
        let mut trace = sample_trace(GameMode::InGame);
        trace.level_complete = true;
        trace.level_timer_seconds = 180;
        trace.level_damage_taken = true;
        trace.level_prize_collected = true;
        trace.level_speed_bonus_awarded = true;
        trace.level_complete_score_rows_elapsed_frames = 0x80;

        let scene = build_scene(&trace, false, Some(&visuals), None);
        assert_eq!(
            eol_number_digits_at(&scene, 97),
            vec![2, 0],
            "ASM loc_D660 should display SpeedBonus_Values[0] = 20, not the remaining timer seconds"
        );
    }

    #[test]
    fn level_complete_score_rows_use_high_priority_gfx_slots_loc_c466_c592() {
        let visuals = sample_level_visuals();
        let mut trace = sample_trace(GameMode::InGame);
        trace.level_complete = true;
        trace.level_complete_elapsed_frames = 199;
        trace.level_complete_score_rows_started = 1;
        trace.level_complete_score_rows_elapsed_frames = 0x80;
        trace.level_skip_cheat_enabled = 1;
        trace.level_speed_bonus_awarded = true;

        let scene = build_scene(&trace, false, Some(&visuals), None);
        let eol_sprites = scene
            .commands
            .iter()
            .filter_map(|command| match command {
                DrawCommand::Sprite { tiles, .. } if !tiles.is_empty() => Some(tiles),
                _ => None,
            })
            .collect::<Vec<_>>();

        assert!(!eol_sprites.is_empty(), "expected EOL score/digit sprites");
        assert!(
            eol_sprites
                .iter()
                .flat_map(|tiles| tiles.iter())
                .all(|tile| tile.tile.priority),
            "ASM Object_EOL_Number loc_C466 and Object_EOL_Text loc_C592 set priority(a*)=1 so score rows draw over foreground blocks"
        );
    }

    #[test]
    fn eol_score_sprite_diagnostics_exposes_loc_c466_c592_row_bounds() {
        let visuals = sample_level_visuals();
        let mut trace = sample_trace(GameMode::InGame);
        trace.level_complete = true;
        trace.level_complete_score_rows_started = 1;
        trace.level_complete_score_rows_elapsed_frames = 8;

        let diagnostics = eol_score_sprite_diagnostics(&trace, &visuals, None);

        let time_sprite = diagnostics
            .iter()
            .find(|sprite| {
                sprite.x == -104
                    && sprite.y == 148
                    && sprite.min_x == -104
                    && sprite.max_x == -57
                    && sprite
                        .tiles
                        .first()
                        .is_some_and(|tile| tile.tile_index == 0x0B9 && tile.priority)
            })
            .expect(
                "ASM loc_C070 displays the pre-frame EOL sprite table after loc_D4CE seeds Spr_EOL_Time and Object_EOL_Text loc_C592 marks the gfx slot high-priority; diagnostics must expose those bounds for oracle slot comparison",
            );

        assert_eq!(
            time_sprite
                .hardware_sprites
                .iter()
                .map(|sprite| (
                    sprite.x,
                    sprite.y,
                    sprite.width,
                    sprite.height,
                    sprite.first_tile_index
                ))
                .collect::<Vec<_>>(),
            vec![(-104, 148, 32, 8, 0x0B9), (-72, 148, 16, 8, 0x0BD)],
            "ASM loc_C070 displays the pre-frame EOL sprite table after loc_D4CE seeds Spr_EOL_Time and Object_EOL_Text loc_C592 marks the gfx slot high-priority; diagnostics must expose those bounds for oracle slot comparison"
        );
    }

    fn eol_number_digits_at(scene: &Scene, y: i32) -> Vec<u8> {
        scene
            .commands
            .iter()
            .filter_map(|command| match command {
                DrawCommand::Sprite {
                    x,
                    y: sprite_y,
                    tile_sheet,
                    ..
                } if *x >= 0 && *sprite_y == y => tile_sheet
                    .tiles
                    .first()
                    .and_then(|tile| tile.pixels[0].checked_sub(130)),
                _ => None,
            })
            .collect()
    }

    fn eol_expected_digits(value: u32) -> Vec<u8> {
        value
            .min(9_999_999)
            .to_string()
            .bytes()
            .map(|digit| digit - b'0')
            .collect()
    }

    #[test]
    fn level_complete_overlay_replays_loc_d770_score_counter_drain() {
        let visuals = sample_level_visuals();
        let mut trace = sample_trace(GameMode::InGame);
        trace.level_complete = true;
        trace.level_timer_seconds = 180;
        trace.level_damage_taken = true;
        trace.level_prize_collected = true;
        trace.level_complete_score_rows_elapsed_frames = 0x40;
        trace.level_complete_score_counter_elapsed_frames = 1;
        let time_bonus = wildside_core::time_bonus_loc_d56e(trace.level_timer_seconds);
        trace.level_complete_initial_score = 1_000;
        trace.score = 0;

        let scene = build_scene(&trace, false, Some(&visuals), None);

        assert_eq!(
            eol_number_digits_at(&scene, 145),
            eol_expected_digits(time_bonus - 0x28),
            "ASM loc_D7CE should draw the active EOL bonus row after subtracting the first `$28` transfer"
        );
        assert_eq!(
            eol_number_digits_at(&scene, 169),
            eol_expected_digits(1_000 + 0x28),
            "ASM loc_D17E clears live score, but loc_D77E should draw from the copied EOL score seed"
        );
    }

    #[test]
    fn level_complete_continues_row_uses_number_continues_digit_loc_c4d2() {
        let visuals = sample_level_visuals();
        let mut trace = sample_trace(GameMode::InGame);
        trace.level_complete = true;
        trace.level_complete_score_rows_elapsed_frames = 0x40;
        trace.number_continues = 3;

        let scene = build_scene(&trace, false, Some(&visuals), None);
        let continues_digit = scene
            .commands
            .iter()
            .find_map(|command| match command {
                DrawCommand::Sprite { x, y, tiles, .. }
                    if *x == 172 && *y == 193 && !tiles.is_empty() =>
                {
                    Some(tiles[0].tile.tile_index)
                }
                _ => None,
            })
            .expect("CONTINUES digit should be drawn at the final slot");

        assert_eq!(
            continues_digit, 0x0CE,
            "ASM Object_EOL_Number loc_C4D2 indexes Spr_EOL_Digits_Index with Number_Continues=3"
        );
    }

    #[test]
    fn level_complete_overlay_draws_level_skip_time_digits_loc_d47e() {
        let mut visuals = sample_level_visuals();
        let eol = visuals.eol_score.as_mut().expect("EOL visuals");
        for frame in &mut eol.digits {
            frame.origin_x = 0;
            frame.origin_y = 0;
        }
        eol.digits[1].origin_x = -3;
        let mut trace = sample_trace(GameMode::InGame);
        trace.level_complete = true;
        trace.level_skip_cheat_enabled = 1;
        trace.level_complete_score_rows_started = 1;
        trace.level_completion_time = 1;

        let scene = build_scene(&trace, false, Some(&visuals), None);
        let digits = scene
            .commands
            .iter()
            .filter_map(|command| match command {
                DrawCommand::Sprite {
                    x, y, tile_sheet, ..
                } if *y == 201 && matches!(*x, 260 | 272 | 287) => tile_sheet
                    .tiles
                    .first()
                    .and_then(|tile| tile.pixels[0].checked_sub(130))
                    .map(|digit| (*x, digit)),
                _ => None,
            })
            .collect::<Vec<_>>();

        assert_eq!(
            digits,
            vec![(287, 1), (272, 0), (260, 0)],
            "ASM loc_D47E writes gfx-object coordinates at x=$11C,$110,$104 and y=$D4 before ordinary EOL rows"
        );
    }

    #[test]
    fn level_complete_overlay_flies_rows_in_with_loc_c5a2_offsets() {
        let visuals = sample_level_visuals();
        let mut trace = sample_trace(GameMode::InGame);
        trace.level_complete = true;
        trace.level_timer_seconds = 180;
        trace.level_damage_taken = true;
        trace.level_prize_collected = true;
        trace.score = wildside_core::time_bonus_loc_d56e(trace.level_timer_seconds);

        let before_first_hibernate = build_scene(&trace, false, Some(&visuals), None);
        assert!(
            !has_sprite_signature(&before_first_hibernate, 128),
            "ASM Object_EOL_Text loc_C5A2 hibernates before the first visible TIME row update"
        );

        trace.level_complete_score_rows_elapsed_frames = 7;
        let fade_hidden = build_scene(&trace, false, Some(&visuals), None);
        assert!(
            !has_sprite_signature(&fade_hidden, 128),
            "ASM loc_D734's eight one-frame fade steps keep native EOL rows visually hidden before the loc_C5A2 row fly-in is visible",
        );

        trace.level_complete_score_rows_elapsed_frames = 8;
        let early = build_scene(&trace, false, Some(&visuals), None);
        assert!(
            early.commands.iter().any(|command| matches!(
                command,
                DrawCommand::Sprite { x, y, tile_sheet, .. }
                    if *y == 148 && *x < 0 && tile_sheet.tiles.first().is_some_and(|tile| tile.pixels[0] == 128)
            )),
            "ASM loc_D734 falls through after the eighth fade tick, so Object_EOL_Text loc_C5A2 should start EOL text rows off-screen left before their +8px/frame fly-in",
        );
        assert!(
            early.commands.iter().any(|command| matches!(
                command,
                DrawCommand::Sprite { x, y, tile_sheet, .. }
                    if *y == 145 && *x == 420 && tile_sheet.tiles.first().is_some_and(|tile| tile.pixels[0] == 133)
            )),
            "ASM loc_C070 displays the pre-frame sprite table: Object_EOL_Number loc_C48C has already advanced 14 ticks when the first native-visible row frame is rendered",
        );
        assert!(
            early.commands.iter().any(|command| matches!(
                command,
                DrawCommand::Sprite { x, y, tile_sheet, .. }
                    if *y == 193 && *x == 60 && tile_sheet.tiles.first().is_some_and(|tile| tile.pixels[0] == 133)
            )),
            "ASM loc_C070 displays the pre-frame sprite table: Object_EOL_Number loc_C48C leaves the CONTINUES number 14 ticks into its +8px fly-in on the first native-visible row frame",
        );

        trace.level_complete_score_rows_elapsed_frames = 0x2C;
        let settled = build_scene(&trace, false, Some(&visuals), None);
        assert!(
            settled.commands.iter().any(|command| matches!(
                command,
                DrawCommand::Sprite { x, y, tile_sheet, .. }
                    if *y == 148 && *x == 64 && tile_sheet.tiles.first().is_some_and(|tile| tile.pixels[0] == 128)
            )),
            "ASM loc_D4CE increments d6 before TIME/BONUS, so after loc_C5A2 the time label should rest at x=$40",
        );
        assert!(
            settled.commands.iter().any(|command| matches!(
                command,
                DrawCommand::Sprite { x, y, tile_sheet, .. }
                    if *y == 145 && *x == 244 && tile_sheet.tiles.first().is_some_and(|tile| tile.pixels[0] == 133)
            )),
            "ASM loc_C070 displays the pre-frame sprite table: Object_EOL_Number loc_C48C reaches its settled x one rendered frame earlier than the post-frame row counter suggests",
        );
    }

    fn has_guide_rect(scene: &Scene) -> bool {
        scene.commands.iter().any(
            |command| matches!(command, DrawCommand::Rect { colour, .. } if *colour == GUIDE_COLOUR),
        )
    }

    fn count_hud_hitpoint_tiles(scene: &Scene) -> usize {
        scene
            .commands
            .iter()
            .filter(|command| {
                matches!(
                    command,
                    DrawCommand::IndexedTile { y, tile_sheet, tile, .. }
                        if *y < 48
                            && tile_sheet.tiles.len() == 4
                            && tile.tile_index <= 3
                )
            })
            .count()
    }

    fn indexed_tile_at(
        scene: &Scene,
        x: i32,
        y: i32,
    ) -> Option<(&Arc<TileSheet>, TileRef, &Arc<Palette>)> {
        scene.commands.iter().find_map(|command| match command {
            DrawCommand::IndexedTile {
                x: command_x,
                y: command_y,
                tile_sheet,
                tile,
                palette,
            } if *command_x == x && *command_y == y => Some((tile_sheet, *tile, palette)),
            _ => None,
        })
    }

    fn indexed_tile_at_with_sheet_len(
        scene: &Scene,
        x: i32,
        y: i32,
        sheet_len: usize,
    ) -> Option<(&Arc<TileSheet>, TileRef, &Arc<Palette>)> {
        scene.commands.iter().find_map(|command| match command {
            DrawCommand::IndexedTile {
                x: command_x,
                y: command_y,
                tile_sheet,
                tile,
                palette,
            } if *command_x == x && *command_y == y && tile_sheet.tiles.len() == sheet_len => {
                Some((tile_sheet, *tile, palette))
            }
            _ => None,
        })
    }

    fn topmost_indexed_tile_at_with_sheet_len(
        scene: &Scene,
        x: i32,
        y: i32,
        sheet_len: usize,
    ) -> Option<(&Arc<TileSheet>, TileRef, &Arc<Palette>)> {
        scene
            .commands
            .iter()
            .rev()
            .find_map(|command| match command {
                DrawCommand::IndexedTile {
                    x: command_x,
                    y: command_y,
                    tile_sheet,
                    tile,
                    palette,
                } if *command_x == x && *command_y == y && tile_sheet.tiles.len() == sheet_len => {
                    Some((tile_sheet, *tile, palette))
                }
                _ => None,
            })
    }

    fn pixel_at(rgb: &[u8], x: u32, y: u32) -> [u8; 3] {
        let index = ((y * FRAME_WIDTH + x) * 3) as usize;
        [rgb[index], rgb[index + 1], rgb[index + 2]]
    }

    fn colour(red: u8, green: u8, blue: u8) -> MdColour {
        MdColour {
            red,
            green,
            blue,
            raw_word: 0,
        }
    }

    fn make_tile(pixels: &[(usize, usize, u8)]) -> Tile {
        let mut tile = Tile { pixels: [0; 64] };
        for (x, y, value) in pixels {
            tile.pixels[y * 8 + x] = *value;
        }
        tile
    }

    fn sample_tile_ref(tile_index: u16) -> TileRef {
        TileRef {
            tile_index,
            palette_line: 0,
            priority: false,
            flip_x: false,
            flip_y: false,
        }
    }

    fn sample_tile_quad(tile_index: u16) -> TileQuad {
        TileQuad {
            tiles: [
                sample_tile_ref(tile_index),
                sample_tile_ref(tile_index + 1),
                sample_tile_ref(tile_index + 2),
                sample_tile_ref(tile_index + 3),
            ],
        }
    }

    fn blank_tile_sheet(tile_count: usize) -> TileSheet {
        TileSheet {
            tiles: vec![Tile { pixels: [0; 64] }; tile_count],
        }
    }

    fn sample_hitpoint_tile_sheet() -> TileSheet {
        TileSheet {
            tiles: (0..4)
                .map(|index| Tile {
                    pixels: [0xD0 + index; 64],
                })
                .collect(),
        }
    }

    fn sample_pause_menu_visuals() -> wildside_render::PauseMenuVisuals {
        let mut tile_sheet = blank_tile_sheet(25);
        for (index, tile) in tile_sheet.tiles.iter_mut().enumerate() {
            tile.pixels = [0xA0 + index as u8; 64];
        }
        wildside_render::PauseMenuVisuals {
            tile_sheet,
            vram_base_tile_index: 0x0676,
            give_up_rows: vec![
                vec![TileRef::from_attr_word(0x8000); 16],
                vec![TileRef::from_attr_word(0x0001); 16],
                vec![TileRef::from_attr_word(0x0002); 16],
                vec![TileRef::from_attr_word(0x0004); 16],
                vec![TileRef::from_attr_word(0x1000); 16],
            ],
            restart_rows: vec![
                vec![TileRef::from_attr_word(0x8000); 16],
                vec![TileRef::from_attr_word(0x0001); 16],
                vec![TileRef::from_attr_word(0x0002); 16],
                vec![TileRef::from_attr_word(0x000C); 16],
                vec![TileRef::from_attr_word(0x1000); 16],
            ],
        }
    }

    fn sample_player_frame(signature: u8) -> SpriteFrame {
        let tile = Tile {
            pixels: [signature; 64],
        };
        SpriteFrame {
            origin_x: 0,
            origin_y: 0,
            width_px: 16,
            height_px: 32,
            tile_base_index: 0,
            tile_sheet: TileSheet {
                tiles: vec![
                    tile.clone(),
                    tile.clone(),
                    tile.clone(),
                    tile.clone(),
                    tile.clone(),
                    tile.clone(),
                    tile.clone(),
                    tile,
                ],
            },
        }
    }

    fn sample_object_frame(signature: u8, width_px: u16, height_px: u16) -> SpriteFrame {
        let tile_count = usize::from(width_px.div_ceil(8)) * usize::from(height_px.div_ceil(8));
        SpriteFrame {
            origin_x: i16::try_from(width_px / 2).expect("origin x"),
            origin_y: i16::try_from(height_px).expect("origin y"),
            width_px,
            height_px,
            tile_base_index: 0,
            tile_sheet: TileSheet {
                tiles: vec![
                    Tile {
                        pixels: [signature; 64],
                    };
                    tile_count
                ],
            },
        }
    }

    fn sample_object_frame_with_base(
        signature: u8,
        width_px: u16,
        height_px: u16,
        tile_base_index: u16,
    ) -> SpriteFrame {
        SpriteFrame {
            tile_base_index,
            ..sample_object_frame(signature, width_px, height_px)
        }
    }

    fn sample_eol_frame(
        signature: u8,
        width_px: u16,
        height_px: u16,
        tile_base_index: u16,
        origin_x: i16,
        origin_y: i16,
    ) -> SpriteFrame {
        SpriteFrame {
            origin_x,
            origin_y,
            ..sample_object_frame_with_base(signature, width_px, height_px, tile_base_index)
        }
    }

    fn numbered_object_frame(width_px: u16, height_px: u16) -> SpriteFrame {
        let tile_count = usize::from(width_px.div_ceil(8)) * usize::from(height_px.div_ceil(8));
        let tiles = (0..tile_count)
            .map(|index| Tile {
                pixels: [u8::try_from(index + 1).expect("tile signature"); 64],
            })
            .collect();
        SpriteFrame {
            origin_x: 0,
            origin_y: 0,
            width_px,
            height_px,
            tile_base_index: 0,
            tile_sheet: TileSheet { tiles },
        }
    }

    fn sample_player_visuals() -> PlayerVisuals {
        let mut forms = std::collections::HashMap::new();
        forms.insert(
            0,
            PlayerFormVisuals {
                palette: Palette {
                    colours: vec![colour(0, 0, 0), colour(200, 160, 120)],
                },
                idle: sample_player_frame(1),
                walk: vec![
                    sample_player_frame(2),
                    sample_player_frame(3),
                    sample_player_frame(4),
                    sample_player_frame(5),
                    sample_player_frame(6),
                    sample_player_frame(7),
                ],
                flight_spin: Vec::new(),
                charge: Vec::new(),
                climb: Vec::new(),
                axe: Vec::new(),
                slope_walk: vec![
                    sample_player_frame(25),
                    sample_player_frame(26),
                    sample_player_frame(27),
                    sample_player_frame(28),
                    sample_player_frame(29),
                    sample_player_frame(30),
                ],
                slope_walk_left: vec![
                    sample_player_frame(31),
                    sample_player_frame(32),
                    sample_player_frame(33),
                    sample_player_frame(34),
                    sample_player_frame(35),
                    sample_player_frame(36),
                ],
                jump_rise: sample_player_frame(8),
                jump_fall: sample_player_frame(9),
                death: Some(sample_player_frame(47)),
                pull_up: vec![
                    sample_player_frame(40),
                    sample_player_frame(41),
                    sample_player_frame(42),
                    sample_player_frame(43),
                    sample_player_frame(44),
                    sample_player_frame(45),
                    sample_player_frame(46),
                ],
                slash_ground: Vec::new(),
                slash_air: Vec::new(),
                transform_palette: None,
                transform_steps: Vec::new(),
            },
        );
        forms.insert(
            PlayerForm::IronKnight.helmet_id(),
            PlayerFormVisuals {
                palette: Palette {
                    colours: vec![colour(0, 0, 0), colour(120, 180, 220)],
                },
                idle: sample_player_frame(10),
                walk: vec![
                    sample_player_frame(11),
                    sample_player_frame(12),
                    sample_player_frame(13),
                    sample_player_frame(14),
                    sample_player_frame(15),
                    sample_player_frame(16),
                ],
                flight_spin: Vec::new(),
                charge: Vec::new(),
                climb: vec![
                    sample_player_frame(50),
                    sample_player_frame(51),
                    sample_player_frame(52),
                ],
                axe: Vec::new(),
                slope_walk: Vec::new(),
                slope_walk_left: Vec::new(),
                jump_rise: sample_player_frame(17),
                jump_fall: sample_player_frame(18),
                death: None,
                pull_up: Vec::new(),
                slash_ground: Vec::new(),
                slash_air: Vec::new(),
                transform_palette: Some(Palette {
                    colours: vec![colour(0, 0, 0), colour(255, 230, 160)],
                }),
                transform_steps: vec![
                    PlayerTransformStep {
                        duration_frames: 4,
                        frame: Some(sample_player_frame(19)),
                    },
                    PlayerTransformStep {
                        duration_frames: 4,
                        frame: None,
                    },
                    PlayerTransformStep {
                        duration_frames: 4,
                        frame: Some(sample_player_frame(20)),
                    },
                ],
            },
        );
        forms.insert(
            PlayerForm::RedStealth.helmet_id(),
            PlayerFormVisuals {
                palette: Palette {
                    colours: vec![colour(0, 0, 0), colour(255, 100, 100)],
                },
                idle: sample_player_frame(21),
                walk: vec![sample_player_frame(22)],
                flight_spin: Vec::new(),
                charge: Vec::new(),
                climb: Vec::new(),
                axe: Vec::new(),
                slope_walk: vec![
                    sample_player_frame(38),
                    sample_player_frame(39),
                    sample_player_frame(39),
                    sample_player_frame(38),
                    sample_player_frame(39),
                    sample_player_frame(39),
                ],
                slope_walk_left: vec![
                    sample_player_frame(40),
                    sample_player_frame(41),
                    sample_player_frame(41),
                    sample_player_frame(40),
                    sample_player_frame(41),
                    sample_player_frame(41),
                ],
                jump_rise: sample_player_frame(23),
                jump_fall: sample_player_frame(24),
                death: None,
                pull_up: Vec::new(),
                slash_ground: vec![
                    sample_player_frame(30),
                    sample_player_frame(31),
                    sample_player_frame(32),
                    sample_player_frame(33),
                ],
                slash_air: vec![
                    sample_player_frame(34),
                    sample_player_frame(35),
                    sample_player_frame(36),
                    sample_player_frame(37),
                ],
                transform_palette: None,
                transform_steps: Vec::new(),
            },
        );
        forms.insert(
            PlayerForm::Cyclone.helmet_id(),
            PlayerFormVisuals {
                palette: Palette {
                    colours: vec![colour(0, 0, 0), colour(120, 220, 180)],
                },
                idle: sample_player_frame(48),
                walk: vec![sample_player_frame(49)],
                flight_spin: vec![
                    sample_player_frame(53),
                    sample_player_frame(54),
                    sample_player_frame(55),
                ],
                charge: Vec::new(),
                climb: Vec::new(),
                axe: Vec::new(),
                slope_walk: Vec::new(),
                slope_walk_left: Vec::new(),
                jump_rise: sample_player_frame(50),
                jump_fall: sample_player_frame(51),
                death: None,
                pull_up: Vec::new(),
                slash_ground: Vec::new(),
                slash_air: Vec::new(),
                transform_palette: Some(Palette {
                    colours: vec![colour(0, 0, 0), colour(20, 20, 20)],
                }),
                transform_steps: vec![
                    PlayerTransformStep {
                        duration_frames: 16,
                        frame: Some(sample_player_frame(2)),
                    },
                    PlayerTransformStep {
                        duration_frames: 4,
                        frame: Some(sample_player_frame(52)),
                    },
                ],
            },
        );

        PlayerVisuals { forms }
    }

    #[test]
    fn berzerker_charge_frame_uses_walk_visual_accumulator_loc_9564() {
        let visuals = PlayerFormVisuals {
            palette: Palette {
                colours: vec![colour(0, 0, 0), colour(200, 160, 120)],
            },
            idle: sample_player_frame(1),
            walk: vec![
                sample_player_frame(10),
                sample_player_frame(11),
                sample_player_frame(12),
                sample_player_frame(13),
                sample_player_frame(14),
                sample_player_frame(15),
            ],
            flight_spin: Vec::new(),
            charge: vec![
                sample_player_frame(40),
                sample_player_frame(41),
                sample_player_frame(42),
                sample_player_frame(43),
                sample_player_frame(44),
                sample_player_frame(45),
            ],
            climb: Vec::new(),
            axe: Vec::new(),
            slope_walk: Vec::new(),
            slope_walk_left: Vec::new(),
            jump_rise: sample_player_frame(20),
            jump_fall: sample_player_frame(21),
            death: None,
            pull_up: Vec::new(),
            slash_ground: Vec::new(),
            slash_air: Vec::new(),
            transform_palette: None,
            transform_steps: Vec::new(),
        };
        let mut trace = sample_trace(GameMode::InGame);
        trace.current_helmet_id = PlayerForm::Berzerker.helmet_id();
        trace.kid_x_velocity = Some(0x0002_0000);
        trace.kid_on_ground = Some(true);
        trace.berzerker_charging = true;
        trace.walk_visual_accumulator = 0x0308;

        let frame = sprites::select_player_frame(&trace, &visuals);

        assert_eq!(
            frame.tile_sheet.tiles[0].pixels[0], 43,
            "ASM sub_942A loc_9564: off_9688 index is `$FFFFF8F0 >> 8`, so $0308 selects phase 3"
        );
    }

    #[test]
    fn berzerker_charge_pose_latch_selects_head_forward_after_flag_clear_loc_9326() {
        let visuals = PlayerFormVisuals {
            palette: Palette {
                colours: vec![colour(0, 0, 0), colour(200, 160, 120)],
            },
            idle: sample_player_frame(1),
            walk: vec![
                sample_player_frame(10),
                sample_player_frame(11),
                sample_player_frame(12),
                sample_player_frame(13),
                sample_player_frame(14),
                sample_player_frame(15),
            ],
            flight_spin: Vec::new(),
            charge: vec![
                sample_player_frame(40),
                sample_player_frame(41),
                sample_player_frame(42),
                sample_player_frame(43),
                sample_player_frame(44),
                sample_player_frame(45),
            ],
            climb: Vec::new(),
            axe: Vec::new(),
            slope_walk: Vec::new(),
            slope_walk_left: Vec::new(),
            jump_rise: sample_player_frame(20),
            jump_fall: sample_player_frame(21),
            death: None,
            pull_up: Vec::new(),
            slash_ground: Vec::new(),
            slash_air: Vec::new(),
            transform_palette: None,
            transform_steps: Vec::new(),
        };
        let mut trace = sample_trace(GameMode::InGame);
        trace.current_helmet_id = PlayerForm::Berzerker.helmet_id();
        trace.kid_x_velocity = Some(0x0002_0000);
        trace.kid_on_ground = Some(true);
        trace.berzerker_charging = false;
        trace.berzerker_charge_pose = true;
        trace.walk_visual_accumulator = 0x0308;

        let frame = sprites::select_player_frame(&trace, &visuals);

        assert_eq!(
            frame.tile_sheet.tiles[0].pixels[0], 43,
            "ASM loc_9564 writes off_9688 before loc_9326 can clear Berzerker_charging"
        );
    }

    #[test]
    fn berzerker_charge_pose_latch_overrides_idle_after_wall_stop_loc_9564_9326() {
        let visuals = PlayerFormVisuals {
            palette: Palette {
                colours: vec![colour(0, 0, 0), colour(200, 160, 120)],
            },
            idle: sample_player_frame(1),
            walk: vec![
                sample_player_frame(10),
                sample_player_frame(11),
                sample_player_frame(12),
                sample_player_frame(13),
                sample_player_frame(14),
                sample_player_frame(15),
            ],
            flight_spin: Vec::new(),
            charge: vec![
                sample_player_frame(40),
                sample_player_frame(41),
                sample_player_frame(42),
                sample_player_frame(43),
                sample_player_frame(44),
                sample_player_frame(45),
            ],
            climb: Vec::new(),
            axe: Vec::new(),
            slope_walk: Vec::new(),
            slope_walk_left: Vec::new(),
            jump_rise: sample_player_frame(20),
            jump_fall: sample_player_frame(21),
            death: None,
            pull_up: Vec::new(),
            slash_ground: Vec::new(),
            slash_air: Vec::new(),
            transform_palette: None,
            transform_steps: Vec::new(),
        };
        let mut trace = sample_trace(GameMode::InGame);
        trace.current_helmet_id = PlayerForm::Berzerker.helmet_id();
        trace.kid_x_velocity = Some(0);
        trace.kid_on_ground = Some(true);
        trace.berzerker_charging = false;
        trace.berzerker_charge_pose = true;
        trace.walk_visual_accumulator = 0x0308;

        let frame = sprites::select_player_frame(&trace, &visuals);

        assert_eq!(
            frame.tile_sheet.tiles[0].pixels[0], 43,
            "ASM loc_9564 writes the off_9688 head-forward sprite before loc_9326 clears Berzerker_charging; a same-frame wall stop must not fall back to idle"
        );
    }

    #[test]
    fn berzerker_regular_walk_pose_latch_overrides_idle_after_wall_stop_loc_9580_9600() {
        let visuals = PlayerFormVisuals {
            palette: Palette {
                colours: vec![colour(0, 0, 0), colour(200, 160, 120)],
            },
            idle: sample_player_frame(1),
            walk: vec![
                sample_player_frame(10),
                sample_player_frame(11),
                sample_player_frame(12),
                sample_player_frame(13),
                sample_player_frame(14),
                sample_player_frame(15),
            ],
            flight_spin: Vec::new(),
            charge: vec![
                sample_player_frame(40),
                sample_player_frame(41),
                sample_player_frame(42),
                sample_player_frame(43),
                sample_player_frame(44),
                sample_player_frame(45),
            ],
            climb: Vec::new(),
            axe: Vec::new(),
            slope_walk: Vec::new(),
            slope_walk_left: Vec::new(),
            jump_rise: sample_player_frame(20),
            jump_fall: sample_player_frame(21),
            death: None,
            pull_up: Vec::new(),
            slash_ground: Vec::new(),
            slash_air: Vec::new(),
            transform_palette: None,
            transform_steps: Vec::new(),
        };
        let mut trace = sample_trace(GameMode::InGame);
        trace.current_helmet_id = PlayerForm::Berzerker.helmet_id();
        trace.kid_x_velocity = Some(0);
        trace.kid_on_ground = Some(true);
        trace.berzerker_charging = false;
        trace.berzerker_charge_pose = false;
        trace.berzerker_walk_pose = true;
        trace.walk_visual_accumulator = 0x0308;

        let frame = sprites::select_player_frame(&trace, &visuals);

        assert_eq!(
            frame.tile_sheet.tiles[0].pixels[0], 13,
            "ASM loc_9580 -> loc_95DA -> loc_9600 writes off_967C using `$FFFFF8F0 >> 8`; a same-frame wall stop must not fall back to idle"
        );
    }

    #[test]
    fn berzerker_charge_pose_overrides_slope_walk_loc_9564() {
        let visuals = PlayerFormVisuals {
            palette: Palette {
                colours: vec![colour(0, 0, 0), colour(200, 160, 120)],
            },
            idle: sample_player_frame(1),
            walk: vec![
                sample_player_frame(10),
                sample_player_frame(11),
                sample_player_frame(12),
                sample_player_frame(13),
                sample_player_frame(14),
                sample_player_frame(15),
            ],
            flight_spin: Vec::new(),
            charge: vec![
                sample_player_frame(40),
                sample_player_frame(41),
                sample_player_frame(42),
                sample_player_frame(43),
                sample_player_frame(44),
                sample_player_frame(45),
            ],
            climb: Vec::new(),
            axe: Vec::new(),
            slope_walk: vec![sample_player_frame(70)],
            slope_walk_left: vec![sample_player_frame(71)],
            jump_rise: sample_player_frame(20),
            jump_fall: sample_player_frame(21),
            death: None,
            pull_up: Vec::new(),
            slash_ground: Vec::new(),
            slash_air: Vec::new(),
            transform_palette: None,
            transform_steps: Vec::new(),
        };
        let mut trace = sample_trace(GameMode::InGame);
        trace.current_helmet_id = PlayerForm::Berzerker.helmet_id();
        trace.kid_x_velocity = Some(0x0002_0000);
        trace.kid_on_ground = Some(true);
        trace.kid_on_slope = 1;
        trace.berzerker_charging = true;
        trace.walk_visual_accumulator = 0x0308;

        let frame = sprites::select_player_frame(&trace, &visuals);

        assert_eq!(
            frame.tile_sheet.tiles[0].pixels[0], 43,
            "ASM loc_9564 writes off_9688 immediately after the Berzerker charge accumulator carries; slope walking must not mask the head-forward charge pose"
        );
    }

    #[test]
    fn berzerker_regular_walk_pose_yields_to_slope_walk_loc_a13a() {
        let visuals = PlayerFormVisuals {
            palette: Palette {
                colours: vec![colour(0, 0, 0), colour(200, 160, 120)],
            },
            idle: sample_player_frame(1),
            walk: vec![
                sample_player_frame(10),
                sample_player_frame(11),
                sample_player_frame(12),
                sample_player_frame(13),
                sample_player_frame(14),
                sample_player_frame(15),
            ],
            flight_spin: Vec::new(),
            charge: vec![
                sample_player_frame(40),
                sample_player_frame(41),
                sample_player_frame(42),
                sample_player_frame(43),
                sample_player_frame(44),
                sample_player_frame(45),
            ],
            climb: Vec::new(),
            axe: Vec::new(),
            slope_walk: vec![sample_player_frame(70), sample_player_frame(71)],
            slope_walk_left: vec![sample_player_frame(80), sample_player_frame(81)],
            jump_rise: sample_player_frame(20),
            jump_fall: sample_player_frame(21),
            death: None,
            pull_up: Vec::new(),
            slash_ground: Vec::new(),
            slash_air: Vec::new(),
            transform_palette: None,
            transform_steps: Vec::new(),
        };
        let mut trace = sample_trace(GameMode::InGame);
        trace.current_helmet_id = PlayerForm::Berzerker.helmet_id();
        trace.kid_x_velocity = Some(0x0000_8000);
        trace.kid_on_ground = Some(true);
        trace.kid_facing = Some(Direction::Right);
        trace.kid_on_slope = 1;
        trace.berzerker_charging = false;
        trace.berzerker_charge_pose = false;
        trace.berzerker_walk_pose = true;
        trace.walk_visual_accumulator = 0x0100;

        let frame = sprites::select_player_frame(&trace, &visuals);

        assert_eq!(
            frame.tile_sheet.tiles[0].pixels[0], 81,
            "ASM loc_A13A writes the slope off_A190 sprite after loc_9600's ordinary Berzerker walk sprite"
        );
    }

    #[test]
    fn berzerker_slope_phase_uses_off_a1e8_six_frame_bank() {
        let visuals = PlayerFormVisuals {
            palette: Palette {
                colours: vec![colour(0, 0, 0), colour(200, 160, 120)],
            },
            idle: sample_player_frame(1),
            walk: vec![sample_player_frame(10), sample_player_frame(11)],
            flight_spin: Vec::new(),
            charge: Vec::new(),
            climb: Vec::new(),
            axe: Vec::new(),
            slope_walk: vec![
                sample_player_frame(70),
                sample_player_frame(71),
                sample_player_frame(72),
                sample_player_frame(73),
                sample_player_frame(74),
                sample_player_frame(75),
            ],
            slope_walk_left: vec![
                sample_player_frame(80),
                sample_player_frame(81),
                sample_player_frame(82),
                sample_player_frame(83),
                sample_player_frame(84),
                sample_player_frame(85),
            ],
            jump_rise: sample_player_frame(20),
            jump_fall: sample_player_frame(21),
            death: None,
            pull_up: Vec::new(),
            slash_ground: Vec::new(),
            slash_air: Vec::new(),
            transform_palette: None,
            transform_steps: Vec::new(),
        };
        let mut trace = sample_trace(GameMode::InGame);
        trace.current_helmet_id = PlayerForm::Berzerker.helmet_id();
        trace.kid_x_velocity = Some(0x0000_8000);
        trace.kid_on_ground = Some(true);
        trace.kid_facing = Some(Direction::Right);
        trace.kid_on_slope = 1;
        trace.walk_visual_accumulator = 0x0200;

        let frame = sprites::select_player_frame(&trace, &visuals);

        assert_eq!(
            frame.tile_sheet.tiles[0].pixels[0], 82,
            "ASM loc_A13A/off_A190 selects off_A1E8 for Berzerker helmet id 7; phase 2 is LnkTo_unk_B14F2, not Red Stealth's off_A230 two-art repeat"
        );
    }

    #[test]
    fn juggernaut_walk_frame_uses_walk_visual_accumulator_loc_94b4() {
        let visuals = PlayerFormVisuals {
            palette: Palette {
                colours: vec![colour(0, 0, 0), colour(200, 160, 120)],
            },
            idle: sample_player_frame(1),
            walk: vec![
                sample_player_frame(10),
                sample_player_frame(11),
                sample_player_frame(12),
                sample_player_frame(13),
                sample_player_frame(14),
                sample_player_frame(15),
            ],
            flight_spin: Vec::new(),
            charge: Vec::new(),
            climb: Vec::new(),
            axe: Vec::new(),
            slope_walk: Vec::new(),
            slope_walk_left: Vec::new(),
            jump_rise: sample_player_frame(20),
            jump_fall: sample_player_frame(21),
            death: None,
            pull_up: Vec::new(),
            slash_ground: Vec::new(),
            slash_air: Vec::new(),
            transform_palette: None,
            transform_steps: Vec::new(),
        };
        let mut trace = sample_trace(GameMode::InGame);
        trace.current_helmet_id = PlayerForm::Juggernaut.helmet_id();
        trace.frame_index = 0;
        trace.kid_x_velocity = Some(0x0002_0000);
        trace.kid_on_ground = Some(true);
        trace.walk_visual_accumulator = 0x0308;

        let frame = sprites::select_player_frame(&trace, &visuals);

        assert_eq!(
            frame.tile_sheet.tiles[0].pixels[0], 13,
            "ASM loc_94B4/loc_97E8: Juggernaut walk/support mappings use `$FFFFF8F0 >> 8`, so $0308 selects phase 3"
        );
    }

    #[test]
    fn red_stealth_walk_frame_uses_walk_visual_accumulator_loc_9600() {
        let visuals = PlayerFormVisuals {
            palette: Palette {
                colours: vec![colour(0, 0, 0), colour(200, 160, 120)],
            },
            idle: sample_player_frame(1),
            walk: vec![
                sample_player_frame(30),
                sample_player_frame(31),
                sample_player_frame(32),
                sample_player_frame(33),
                sample_player_frame(34),
                sample_player_frame(35),
            ],
            flight_spin: Vec::new(),
            charge: Vec::new(),
            climb: Vec::new(),
            axe: Vec::new(),
            slope_walk: Vec::new(),
            slope_walk_left: Vec::new(),
            jump_rise: sample_player_frame(40),
            jump_fall: sample_player_frame(41),
            death: None,
            pull_up: Vec::new(),
            slash_ground: Vec::new(),
            slash_air: Vec::new(),
            transform_palette: None,
            transform_steps: Vec::new(),
        };
        let mut trace = sample_trace(GameMode::InGame);
        trace.current_helmet_id = PlayerForm::RedStealth.helmet_id();
        trace.frame_index = 0;
        trace.kid_x_velocity = Some(0x0002_0000);
        trace.kid_on_ground = Some(true);
        trace.walk_visual_accumulator = 0x0308;

        let frame = sprites::select_player_frame(&trace, &visuals);

        assert_eq!(
            frame.tile_sheet.tiles[0].pixels[0], 33,
            "ASM loc_95DA -> loc_9600/off_9658: Red Stealth ordinary walk frames are indexed by `$FFFFF8F0 >> 8`; $0308 selects phase 3 even when frame_index is 0"
        );
    }

    #[test]
    fn iron_knight_climb_pose_uses_sub_a3bc_sprite_over_idle() {
        let visuals = PlayerFormVisuals {
            palette: Palette {
                colours: vec![colour(0, 0, 0), colour(200, 160, 120)],
            },
            idle: sample_player_frame(1),
            walk: vec![sample_player_frame(10), sample_player_frame(11)],
            flight_spin: Vec::new(),
            charge: Vec::new(),
            climb: vec![
                sample_player_frame(60),
                sample_player_frame(61),
                sample_player_frame(62),
            ],
            axe: Vec::new(),
            slope_walk: Vec::new(),
            slope_walk_left: Vec::new(),
            jump_rise: sample_player_frame(20),
            jump_fall: sample_player_frame(21),
            death: None,
            pull_up: Vec::new(),
            slash_ground: Vec::new(),
            slash_air: Vec::new(),
            transform_palette: None,
            transform_steps: Vec::new(),
        };
        let mut trace = sample_trace(GameMode::InGame);
        trace.current_helmet_id = PlayerForm::IronKnight.helmet_id();
        trace.kid_x_velocity = Some(0);
        trace.kid_y_velocity = Some(0);
        trace.kid_on_ground = Some(false);
        trace.iron_knight_climbing = true;
        trace.iron_knight_climb_pose_accum = 0x0080;

        let frame = sprites::select_player_frame(&trace, &visuals);

        assert_eq!(
            frame.tile_sheet.tiles[0].pixels[0], 62,
            "ASM sub_A3BC/off_A420: d3=$80 buckets through `unk_A3F0` to the top Iron Knight climbing sprite, even when velocity is zero"
        );
    }

    #[test]
    fn juggernaut_slope_fallback_keeps_head_forward_phase_loc_94b4() {
        let visuals = PlayerFormVisuals {
            palette: Palette {
                colours: vec![colour(0, 0, 0), colour(200, 160, 120)],
            },
            idle: sample_player_frame(1),
            walk: vec![
                sample_player_frame(10),
                sample_player_frame(11),
                sample_player_frame(12),
                sample_player_frame(13),
                sample_player_frame(14),
                sample_player_frame(15),
            ],
            flight_spin: Vec::new(),
            charge: Vec::new(),
            climb: Vec::new(),
            axe: Vec::new(),
            slope_walk: Vec::new(),
            slope_walk_left: Vec::new(),
            jump_rise: sample_player_frame(20),
            jump_fall: sample_player_frame(21),
            death: None,
            pull_up: Vec::new(),
            slash_ground: Vec::new(),
            slash_air: Vec::new(),
            transform_palette: None,
            transform_steps: Vec::new(),
        };
        let mut trace = sample_trace(GameMode::InGame);
        trace.current_helmet_id = PlayerForm::Juggernaut.helmet_id();
        trace.frame_index = 0;
        trace.kid_x_velocity = Some(0x0002_0000);
        trace.kid_on_ground = Some(true);
        trace.kid_on_slope = 1;
        trace.walk_visual_accumulator = 0x0308;

        let frame = sprites::select_player_frame(&trace, &visuals);

        assert_eq!(
            frame.tile_sheet.tiles[0].pixels[0], 13,
            "ASM loc_94B4/loc_97E8: when Juggernaut falls back to normal walk art on slopes, the head-forward frame still comes from `$FFFFF8F0 >> 8`"
        );
    }

    fn sample_enemy_visuals() -> EnemyVisuals {
        use std::collections::HashMap;
        let mut families = HashMap::new();
        families.insert(
            0x00,
            AnimatedSpriteVisual {
                palette: Palette {
                    colours: vec![colour(0, 0, 0), colour(255, 120, 32)],
                },
                extra_palettes: vec![],
                frames: vec![
                    sample_object_frame_with_base(1, 0x1A, 0x20, 0x000),
                    sample_object_frame_with_base(2, 0x1A, 0x20, 0x010),
                    sample_object_frame_with_base(3, 0x1A, 0x20, 0x020),
                    sample_object_frame_with_base(4, 0x1E, 0x20, 0x030),
                    sample_object_frame_with_base(3, 0x1A, 0x20, 0x020),
                    sample_object_frame_with_base(2, 0x1A, 0x20, 0x010),
                    sample_object_frame_with_base(5, 0x1E, 0x20, 0x050),
                    sample_object_frame_with_base(6, 0x1E, 0x20, 0x060),
                    sample_object_frame_with_base(7, 0x1E, 0x1F, 0x070),
                    sample_object_frame_with_base(8, 0x1E, 0x20, 0x080),
                    sample_object_frame_with_base(9, 0x20, 0x20, 0x090),
                    sample_object_frame_with_base(10, 0x1E, 0x1F, 0x0A0),
                ],
                frame_duration: 6,
            },
        );
        families.insert(
            0x01,
            AnimatedSpriteVisual {
                palette: Palette {
                    colours: vec![colour(0, 0, 0), colour(120, 220, 255)],
                },
                extra_palettes: vec![],
                frames: vec![
                    sample_object_frame_with_base(21, 0x14, 0x20, 0x02F),
                    sample_object_frame_with_base(22, 0x14, 0x20, 0x03B),
                    sample_object_frame_with_base(23, 0x14, 0x20, 0x047),
                    sample_object_frame_with_base(24, 0x14, 0x20, 0x053),
                    sample_object_frame_with_base(25, 0x14, 0x20, 0x05F),
                    sample_object_frame_with_base(26, 0x14, 0x20, 0x023),
                    sample_object_frame_with_base(27, 0x1D, 0x20, 0x000),
                    sample_object_frame_with_base(28, 0x16, 0x18, 0x010),
                    sample_object_frame_with_base(29, 0x11, 0x12, 0x019),
                    sample_object_frame_with_base(30, 0x07, 0x07, 0x022),
                ],
                frame_duration: 6,
            },
        );
        families.insert(
            0x03,
            AnimatedSpriteVisual {
                palette: Palette {
                    colours: vec![colour(0, 0, 0), colour(180, 200, 230)],
                },
                extra_palettes: vec![],
                frames: vec![
                    sample_object_frame_with_base(91, 0x1D, 0x14, 0x039),
                    sample_object_frame_with_base(92, 0x12, 0x15, 0x045),
                    sample_object_frame_with_base(93, 0x1C, 0x15, 0x04E),
                    sample_object_frame_with_base(94, 0x1D, 0x14, 0x05A),
                    sample_object_frame_with_base(95, 0x15, 0x15, 0x066),
                    sample_object_frame_with_base(96, 0x1C, 0x15, 0x06F),
                    sample_object_frame_with_base(97, 0x1E, 0x20, 0x0BB),
                    sample_object_frame_with_base(98, 0x20, 0x20, 0x0CB),
                ],
                frame_duration: 9,
            },
        );
        families.insert(
            0x04,
            AnimatedSpriteVisual {
                palette: Palette {
                    colours: vec![colour(0, 0, 0), colour(180, 130, 90)],
                },
                extra_palettes: vec![],
                frames: vec![
                    sample_object_frame_with_base(41, 0x27, 0x17, 0x0A5),
                    sample_object_frame_with_base(42, 0x28, 0x18, 0x0B4),
                    sample_object_frame_with_base(43, 0x28, 0x18, 0x0C3),
                    sample_object_frame_with_base(44, 0x28, 0x17, 0x0D2),
                    sample_object_frame_with_base(45, 0x28, 0x18, 0x0E1),
                    sample_object_frame_with_base(46, 0x28, 0x18, 0x0F0),
                    sample_object_frame_with_base(47, 0x28, 0x14, 0x078),
                    sample_object_frame_with_base(48, 0x28, 0x14, 0x087),
                    sample_object_frame_with_base(49, 0x27, 0x18, 0x096),
                ],
                frame_duration: 11,
            },
        );
        families.insert(
            0x05,
            AnimatedSpriteVisual {
                palette: Palette {
                    colours: vec![colour(0, 0, 0), colour(80, 80, 80)],
                },
                extra_palettes: vec![],
                frames: vec![
                    sample_object_frame_with_base(59, 0x1B, 0x20, 0x0AC),
                    sample_object_frame_with_base(60, 0x1B, 0x20, 0x044),
                    sample_object_frame_with_base(61, 0x1F, 0x1F, 0x074),
                    sample_object_frame_with_base(62, 0x1F, 0x1F, 0x084),
                    sample_object_frame_with_base(63, 0x1F, 0x18, 0x094),
                    sample_object_frame_with_base(64, 0x20, 0x10, 0x0A0),
                    sample_object_frame_with_base(65, 0x20, 0x07, 0x0A8),
                ],
                frame_duration: 3,
            },
        );
        families.insert(
            0x1C,
            AnimatedSpriteVisual {
                palette: Palette {
                    colours: vec![colour(0, 0, 0), colour(255, 220, 160)],
                },
                extra_palettes: vec![],
                frames: vec![
                    sample_object_frame(31, 16, 17),
                    sample_object_frame(32, 16, 18),
                    sample_object_frame(33, 13, 15),
                    sample_object_frame(34, 11, 14),
                    sample_object_frame(35, 13, 14),
                    sample_object_frame(36, 14, 11),
                    sample_object_frame(37, 12, 15),
                    sample_object_frame(38, 16, 11),
                ],
                frame_duration: 9,
            },
        );
        families.insert(
            0x06,
            AnimatedSpriteVisual {
                palette: Palette {
                    colours: vec![colour(0, 0, 0), colour(180, 160, 220)],
                },
                extra_palettes: vec![],
                frames: vec![
                    sample_object_frame_with_base(151, 0x10, 0x10, 0x012),
                    sample_object_frame_with_base(152, 0x10, 0x10, 0x016),
                    sample_object_frame_with_base(153, 0x10, 0x10, 0x01A),
                    sample_object_frame_with_base(154, 0x10, 0x10, 0x006),
                    sample_object_frame_with_base(155, 0x10, 0x10, 0x00A),
                    sample_object_frame_with_base(156, 0x10, 0x10, 0x00E),
                ],
                frame_duration: 9,
            },
        );
        families.insert(
            0x07,
            AnimatedSpriteVisual {
                palette: Palette {
                    colours: vec![colour(0, 0, 0), colour(210, 210, 210)],
                },
                extra_palettes: vec![],
                frames: vec![
                    sample_object_frame_with_base(181, 0x1D, 0x1F, 0x030),
                    sample_object_frame_with_base(182, 0x1B, 0x20, 0x040),
                    sample_object_frame_with_base(183, 0x1B, 0x20, 0x050),
                    sample_object_frame_with_base(184, 0x1B, 0x1F, 0x060),
                    sample_object_frame_with_base(185, 0x1B, 0x20, 0x070),
                    sample_object_frame_with_base(186, 0x1B, 0x20, 0x080),
                    sample_object_frame_with_base(187, 0x1E, 0x20, 0x060),
                    sample_object_frame_with_base(188, 0x1B, 0x1E, 0x0A9),
                    sample_object_frame_with_base(189, 0x1C, 0x1E, 0x080),
                    sample_object_frame_with_base(190, 0x1F, 0x1E, 0x090),
                ],
                frame_duration: 8,
            },
        );
        families.insert(
            0x08,
            AnimatedSpriteVisual {
                palette: Palette {
                    colours: vec![colour(0, 0, 0), colour(70, 90, 120)],
                },
                extra_palettes: vec![],
                frames: vec![
                    sample_object_frame_with_base(66, 0x1C, 0x25, 0x05C),
                    sample_object_frame_with_base(67, 0x1C, 0x26, 0x070),
                    sample_object_frame_with_base(68, 0x1D, 0x26, 0x084),
                    sample_object_frame_with_base(69, 0x1E, 0x25, 0x098),
                    sample_object_frame_with_base(70, 0x1C, 0x26, 0x0AC),
                    sample_object_frame_with_base(71, 0x1C, 0x26, 0x0C0),
                    sample_object_frame_with_base(72, 0x1C, 0x1D, 0x024),
                    sample_object_frame_with_base(73, 0x1C, 0x26, 0x0C0),
                    sample_object_frame_with_base(74, 0x1D, 0x1A, 0x000),
                    sample_object_frame_with_base(75, 0x1B, 0x13, 0x010),
                    sample_object_frame_with_base(76, 0x20, 0x0C, 0x01C),
                ],
                frame_duration: 9,
            },
        );
        families.insert(
            0x09,
            AnimatedSpriteVisual {
                palette: Palette {
                    colours: vec![colour(0, 0, 0), colour(180, 140, 100)],
                },
                extra_palettes: vec![],
                frames: (211..=219)
                    .map(|signature| sample_object_frame(signature, 16, 16))
                    .collect(),
                frame_duration: 8,
            },
        );
        families.insert(
            0x10,
            AnimatedSpriteVisual {
                palette: Palette {
                    colours: vec![colour(0, 0, 0), colour(180, 160, 120)],
                },
                extra_palettes: vec![],
                frames: vec![
                    sample_object_frame(111, 16, 16),
                    sample_object_frame(112, 16, 16),
                    sample_object_frame(113, 16, 16),
                    sample_object_frame(114, 16, 16),
                    sample_object_frame(115, 16, 16),
                    sample_object_frame(116, 16, 16),
                    sample_object_frame(117, 16, 16),
                    sample_object_frame(118, 16, 16),
                    sample_object_frame(119, 16, 16),
                ],
                frame_duration: 6,
            },
        );
        families.insert(
            0x11,
            AnimatedSpriteVisual {
                palette: Palette {
                    colours: vec![colour(0, 0, 0), colour(180, 180, 210)],
                },
                extra_palettes: vec![],
                frames: (221..=229)
                    .map(|signature| sample_object_frame(signature, 16, 16))
                    .collect(),
                frame_duration: 8,
            },
        );
        families.insert(
            0x13,
            AnimatedSpriteVisual {
                palette: Palette {
                    colours: vec![colour(0, 0, 0), colour(190, 150, 120)],
                },
                extra_palettes: vec![],
                frames: vec![
                    sample_object_frame_with_base(131, 0x30, 0x1F, 0x04C),
                    sample_object_frame_with_base(132, 0x2E, 0x20, 0x064),
                    sample_object_frame_with_base(133, 0x30, 0x20, 0x07C),
                    sample_object_frame_with_base(134, 0x30, 0x1F, 0x094),
                    sample_object_frame_with_base(135, 0x2F, 0x20, 0x0AC),
                    sample_object_frame_with_base(136, 0x2F, 0x20, 0x0C4),
                    sample_object_frame_with_base(137, 0x30, 0x20, 0x034),
                    sample_object_frame_with_base(138, 0x30, 0x1F, 0x000),
                    sample_object_frame_with_base(139, 0x31, 0x1B, 0x018),
                ],
                frame_duration: 6,
            },
        );
        families.insert(
            0x12,
            AnimatedSpriteVisual {
                palette: Palette {
                    colours: vec![colour(0, 0, 0), colour(210, 160, 90)],
                },
                extra_palettes: vec![],
                frames: vec![
                    sample_object_frame_with_base(50, 0x2C, 0x2D, 0x16C),
                    sample_object_frame_with_base(51, 0x2C, 0x2E, 0x190),
                    sample_object_frame_with_base(52, 0x2B, 0x2E, 0x1B4),
                    sample_object_frame_with_base(53, 0x2D, 0x2D, 0x1D8),
                    sample_object_frame_with_base(54, 0x28, 0x2E, 0x1FC),
                    sample_object_frame_with_base(55, 0x2B, 0x2E, 0x21A),
                    sample_object_frame_with_base(56, 0x2E, 0x2E, 0x00C),
                    sample_object_frame_with_base(57, 0x31, 0x30, 0x030),
                    sample_object_frame_with_base(58, 0x30, 0x30, 0x05A),
                    sample_object_frame_with_base(59, 0x0C, 0x0C, 0x000),
                    sample_object_frame_with_base(60, 0x0C, 0x0C, 0x004),
                    sample_object_frame_with_base(61, 0x0A, 0x0B, 0x008),
                ],
                frame_duration: 7,
            },
        );
        families.insert(
            0x0C,
            AnimatedSpriteVisual {
                palette: Palette {
                    colours: vec![colour(0, 0, 0), colour(210, 120, 80)],
                },
                extra_palettes: vec![],
                frames: vec![
                    sample_object_frame(141, 24, 24),
                    sample_object_frame(142, 24, 24),
                    sample_object_frame(143, 24, 24),
                    sample_object_frame(144, 24, 24),
                    sample_object_frame(145, 24, 24),
                    sample_object_frame(146, 24, 24),
                    sample_object_frame(147, 24, 24),
                    sample_object_frame(148, 24, 24),
                    sample_object_frame(149, 24, 24),
                ],
                frame_duration: 7,
            },
        );
        families.insert(
            0x0D,
            AnimatedSpriteVisual {
                palette: Palette {
                    colours: vec![colour(0, 0, 0), colour(210, 120, 80)],
                },
                extra_palettes: vec![],
                frames: vec![
                    sample_object_frame_with_base(161, 0x1F, 0x26, 0x060),
                    sample_object_frame_with_base(162, 0x1F, 0x27, 0x074),
                    sample_object_frame_with_base(163, 0x20, 0x28, 0x088),
                    sample_object_frame_with_base(164, 0x20, 0x27, 0x09C),
                    sample_object_frame_with_base(165, 0x20, 0x28, 0x0B0),
                    sample_object_frame_with_base(166, 0x1F, 0x27, 0x0C4),
                    sample_object_frame_with_base(167, 0x20, 0x20, 0x000),
                    sample_object_frame_with_base(168, 0x20, 0x28, 0x0D8),
                    sample_object_frame_with_base(169, 0x20, 0x17, 0x010),
                    sample_object_frame_with_base(170, 0x20, 0x17, 0x01C),
                    sample_object_frame_with_base(171, 0x20, 0x27, 0x04C),
                ],
                frame_duration: 9,
            },
        );
        families.insert(
            0x0E,
            AnimatedSpriteVisual {
                palette: Palette {
                    colours: vec![colour(0, 0, 0), colour(190, 220, 250)],
                },
                extra_palettes: vec![],
                frames: vec![
                    sample_object_frame_with_base(251, 0x1F, 0x1F, 0x095),
                    sample_object_frame_with_base(252, 0x20, 0x1E, 0x0A5),
                    sample_object_frame_with_base(253, 0x20, 0x1E, 0x0B5),
                    sample_object_frame_with_base(254, 0x20, 0x1D, 0x0C5),
                    sample_object_frame_with_base(253, 0x20, 0x1E, 0x0B5),
                    sample_object_frame_with_base(252, 0x20, 0x1E, 0x0A5),
                    sample_object_frame_with_base(255, 0x20, 0x1E, 0x065),
                    sample_object_frame_with_base(200, 0x20, 0x20, 0x075),
                    sample_object_frame_with_base(201, 0x1D, 0x1D, 0x085),
                ],
                frame_duration: 11,
            },
        );
        families.insert(
            0x0F,
            AnimatedSpriteVisual {
                palette: Palette {
                    colours: vec![colour(0, 0, 0), colour(180, 200, 230)],
                },
                extra_palettes: vec![],
                frames: vec![
                    sample_object_frame_with_base(214, 0x20, 0x1D, 0x15A),
                    sample_object_frame_with_base(215, 0x20, 0x1E, 0x16A),
                    sample_object_frame_with_base(216, 0x20, 0x1F, 0x17A),
                ],
                frame_duration: 5,
            },
        );
        families.insert(
            0x19,
            AnimatedSpriteVisual {
                palette: Palette {
                    colours: vec![colour(0, 0, 0), colour(255, 130, 70)],
                },
                extra_palettes: vec![],
                frames: vec![
                    sample_object_frame(171, 24, 24),
                    sample_object_frame(172, 24, 24),
                    sample_object_frame(173, 24, 24),
                    sample_object_frame(174, 24, 24),
                    sample_object_frame(175, 16, 16),
                    sample_object_frame(176, 16, 16),
                ],
                frame_duration: 6,
            },
        );
        families.insert(
            0x18,
            AnimatedSpriteVisual {
                palette: Palette {
                    colours: vec![colour(0, 0, 0), colour(180, 210, 230)],
                },
                extra_palettes: vec![],
                frames: vec![
                    sample_object_frame_with_base(231, 0x1D, 0x28, 0x028),
                    sample_object_frame_with_base(232, 0x1C, 0x27, 0x03C),
                    sample_object_frame_with_base(233, 0x1C, 0x28, 0x050),
                    sample_object_frame_with_base(234, 0x1D, 0x26, 0x000),
                    sample_object_frame_with_base(235, 0x1B, 0x24, 0x014),
                ],
                frame_duration: 5,
            },
        );
        families.insert(
            0x1B,
            AnimatedSpriteVisual {
                palette: Palette {
                    colours: vec![colour(0, 0, 0), colour(200, 120, 90)],
                },
                extra_palettes: vec![],
                frames: vec![
                    sample_object_frame_with_base(241, 0x1D, 0x20, 0x010),
                    sample_object_frame_with_base(242, 0x1F, 0x1C, 0x040),
                    sample_object_frame_with_base(243, 0x1F, 0x20, 0x050),
                    sample_object_frame_with_base(244, 0x2A, 0x27, 0x060),
                    sample_object_frame_with_base(245, 0x24, 0x24, 0x07E),
                    sample_object_frame_with_base(246, 0x2B, 0x26, 0x097),
                    sample_object_frame_with_base(247, 0x20, 0x20, 0x0B5),
                    sample_object_frame_with_base(248, 0x1C, 0x06, 0x0C5),
                ],
                frame_duration: 20,
            },
        );
        families.insert(
            0x14,
            AnimatedSpriteVisual {
                palette: Palette {
                    colours: vec![colour(0, 0, 0), colour(200, 190, 150)],
                },
                extra_palettes: vec![],
                frames: vec![
                    sample_object_frame_with_base(202, 0x40, 0x1F, 0x0A2),
                    sample_object_frame_with_base(203, 0x34, 0x1F, 0x0C2),
                    sample_object_frame_with_base(204, 0x1E, 0x1F, 0x0DE),
                    sample_object_frame_with_base(205, 0x3D, 0x1F, 0x0EE),
                    sample_object_frame_with_base(206, 0x1C, 0x20, 0x012),
                    sample_object_frame_with_base(207, 0x1F, 0x1E, 0x022),
                    sample_object_frame_with_base(208, 0x20, 0x12, 0x032),
                ],
                frame_duration: 6,
            },
        );
        families.insert(
            0x1D,
            AnimatedSpriteVisual {
                palette: Palette {
                    colours: vec![colour(0, 0, 0), colour(230, 230, 210)],
                },
                extra_palettes: vec![],
                frames: vec![
                    sample_object_frame_with_base(209, 0x19, 0x20, 0x040),
                    sample_object_frame_with_base(210, 0x1B, 0x1D, 0x000),
                    sample_object_frame_with_base(211, 0x1C, 0x1D, 0x090),
                    sample_object_frame_with_base(212, 0x1E, 0x1D, 0x0A0),
                    sample_object_frame_with_base(213, 0x21, 0x10, 0x0B0),
                ],
                frame_duration: 80,
            },
        );
        families.insert(
            0x20,
            AnimatedSpriteVisual {
                palette: Palette {
                    colours: vec![colour(0, 0, 0), colour(255, 80, 80)],
                },
                extra_palettes: vec![],
                frames: vec![sample_object_frame(61, 25, 32)],
                frame_duration: 10,
            },
        );
        families.insert(
            0x21,
            AnimatedSpriteVisual {
                palette: Palette {
                    colours: vec![colour(0, 0, 0), colour(255, 220, 120)],
                },
                extra_palettes: vec![],
                frames: vec![sample_object_frame(71, 16, 16)],
                frame_duration: 6,
            },
        );
        families.insert(
            0x22,
            AnimatedSpriteVisual {
                palette: Palette {
                    colours: vec![colour(0, 0, 0), colour(200, 80, 200)],
                },
                extra_palettes: vec![],
                frames: vec![
                    sample_object_frame_with_base(221, 0x40, 0x40, 0x0A4),
                    sample_object_frame_with_base(222, 0x3B, 0x40, 0x0E4),
                    sample_object_frame_with_base(223, 0x3C, 0x40, 0x124),
                    sample_object_frame_with_base(224, 0x3F, 0x40, 0x164),
                    sample_object_frame_with_base(225, 0x3C, 0x4F, 0x1A4),
                ],
                frame_duration: 12,
            },
        );
        let tank_palette = Palette {
            colours: vec![colour(0, 0, 0), colour(140, 188, 120)],
        };
        families.insert(
            0x0A,
            AnimatedSpriteVisual {
                palette: tank_palette.clone(),
                extra_palettes: vec![],
                frames: vec![
                    sample_object_frame_with_base(81, 0x1E, 0x08, 0x02C),
                    sample_object_frame_with_base(82, 0x1E, 0x08, 0x030),
                    sample_object_frame_with_base(83, 0x1E, 0x08, 0x034),
                    sample_object_frame_with_base(84, 0x1F, 0x1A, 0x000),
                    sample_object_frame_with_base(85, 0x1F, 0x19, 0x010),
                    sample_object_frame_with_base(86, 0x20, 0x14, 0x06C),
                ],
                frame_duration: 6,
            },
        );
        families.insert(
            0x0B,
            AnimatedSpriteVisual {
                palette: tank_palette,
                extra_palettes: vec![],
                frames: vec![
                    sample_object_frame_with_base(81, 0x1E, 0x08, 0x02C),
                    sample_object_frame_with_base(82, 0x1E, 0x08, 0x030),
                    sample_object_frame_with_base(83, 0x1E, 0x08, 0x034),
                    sample_object_frame_with_base(84, 0x1F, 0x1A, 0x000),
                    sample_object_frame_with_base(85, 0x1F, 0x19, 0x010),
                    sample_object_frame_with_base(87, 0x20, 0x14, 0x03C),
                ],
                frame_duration: 6,
            },
        );
        families.insert(
            0x16,
            AnimatedSpriteVisual {
                palette: Palette {
                    colours: vec![colour(0, 0, 0), colour(120, 200, 90)],
                },
                extra_palettes: vec![],
                frames: vec![
                    sample_object_frame_with_base(121, 0x0F, 0x0E, 0x000),
                    sample_object_frame_with_base(122, 0x0E, 0x0F, 0x004),
                    sample_object_frame_with_base(123, 0x0F, 0x0F, 0x008),
                    sample_object_frame_with_base(124, 0x10, 0x10, 0x0C5),
                    sample_object_frame_with_base(125, 0x17, 0x16, 0x0C9),
                    sample_object_frame_with_base(126, 0x18, 0x18, 0x0D2),
                ],
                frame_duration: 8,
            },
        );
        families.insert(
            0x17,
            AnimatedSpriteVisual {
                palette: Palette {
                    colours: vec![colour(0, 0, 0), colour(180, 170, 150)],
                },
                extra_palettes: vec![],
                frames: vec![
                    sample_object_frame_with_base(141, 0x20, 0x12, 0x03F),
                    sample_object_frame_with_base(142, 0x28, 0x0C, 0x04B),
                    sample_object_frame_with_base(143, 0x28, 0x0D, 0x055),
                    sample_object_frame_with_base(144, 0x26, 0x08, 0x05F),
                    sample_object_frame_with_base(145, 0x24, 0x07, 0x064),
                    sample_object_frame_with_base(146, 0x26, 0x04, 0x069),
                    sample_object_frame_with_base(147, 0x26, 0x03, 0x06E),
                    sample_object_frame_with_base(148, 0x1F, 0x01, 0x073),
                ],
                frame_duration: 1,
            },
        );
        families.insert(
            0x1A,
            AnimatedSpriteVisual {
                palette: Palette {
                    colours: vec![colour(0, 0, 0), colour(190, 190, 170)],
                },
                extra_palettes: vec![],
                frames: vec![
                    sample_object_frame_with_base(151, 0x10, 0x0F, 0x028),
                    sample_object_frame_with_base(152, 0x10, 0x0E, 0x02C),
                    sample_object_frame_with_base(153, 0x10, 0x0E, 0x024),
                    sample_object_frame_with_base(154, 0x10, 0x0E, 0x01C),
                    sample_object_frame_with_base(155, 0x10, 0x0B, 0x020),
                ],
                frame_duration: 9,
            },
        );
        EnemyVisuals { families }
    }

    fn sample_floating_prize_visuals() -> wildside_render::FloatingPrizeVisuals {
        use std::collections::HashMap;
        let diamond_palette = Palette {
            colours: vec![colour(0, 0, 0), colour(100, 200, 255)],
        };
        let helmet_palette = Palette {
            colours: vec![colour(0, 0, 0), colour(200, 200, 255)],
        };
        let mut helmets = HashMap::new();
        for (content_id, tile_pattern, width_px, height_px) in [
            (0x02, 91, 0x11, 0x0E),
            (0x03, 92, 0x17, 0x0F),
            (0x04, 93, 0x15, 0x11),
            (0x05, 94, 0x15, 0x10),
            (0x06, 95, 0x12, 0x14),
            (0x07, 96, 0x16, 0x15),
            (0x08, 97, 0x13, 0x14),
            (0x09, 98, 0x10, 0x12),
            (0x0A, 99, 0x13, 0x12),
        ] {
            helmets.insert(
                content_id,
                AnimatedSpriteVisual {
                    palette: helmet_palette.clone(),
                    extra_palettes: vec![],
                    frames: vec![sample_object_frame(tile_pattern, width_px, height_px)],
                    frame_duration: 1,
                },
            );
        }
        let mut icons = HashMap::new();
        for (content_id, tile_pattern) in [(0x0B, 101), (0x0C, 102), (0x0D, 103)] {
            icons.insert(
                content_id,
                AnimatedSpriteVisual {
                    palette: Palette {
                        colours: vec![colour(0, 0, 0), colour(255, 255, 255)],
                    },
                    extra_palettes: vec![],
                    frames: vec![sample_object_frame(tile_pattern, 0x10, 0x10)],
                    frame_duration: 1,
                },
            );
        }
        wildside_render::FloatingPrizeVisuals {
            diamond: AnimatedSpriteVisual {
                palette: diamond_palette,
                extra_palettes: vec![],
                frames: (0..4)
                    .map(|frame| SpriteFrame {
                        origin_x: 0x06,
                        origin_y: -2,
                        width_px: 0x10,
                        height_px: 0x0C,
                        tile_base_index: 8,
                        tile_sheet: TileSheet {
                            tiles: vec![
                                Tile {
                                    pixels: [16 + frame as u8; 64],
                                };
                                16
                            ],
                        },
                    })
                    .collect(),
                frame_duration: 3,
            },
            helmets,
            icons,
        }
    }

    fn sample_platform_visuals() -> PlatformVisuals {
        PlatformVisuals {
            palette: Palette {
                colours: vec![colour(0, 0, 0), colour(232, 176, 64)],
            },
            horizontal: sample_object_frame(41, 72, 16),
            vertical: sample_object_frame(51, 32, 32),
        }
    }

    fn sample_eol_score_visuals() -> wildside_render::EolScoreVisuals {
        wildside_render::EolScoreVisuals {
            palette: Palette {
                colours: vec![colour(0, 0, 0), colour(220, 220, 255)],
            },
            bonus: sample_eol_frame(121, 0x40, 0x08, 0x000, 0, 0),
            continues: sample_eol_frame(120, 0x6C, 0x08, 0x008, 0, 0),
            game: sample_eol_frame(140, 0x5D, 0x10, 0x031, 0, 0),
            no_hit: sample_eol_frame(122, 0x57, 0x08, 0x065, 0, 0),
            no_prize: sample_eol_frame(123, 0x6F, 0x08, 0x070, 0, 0),
            over: sample_eol_frame(141, 0x60, 0x10, 0x049, 0, 0),
            path: sample_eol_frame(124, 0x35, 0x08, 0x080, 0, 0),
            score: sample_eol_frame(125, 0x3D, 0x08, 0x091, 0, 0),
            seconds: sample_eol_frame(126, 0x5F, 0x08, 0x099, 0, 0),
            speed: sample_eol_frame(127, 0x3F, 0x08, 0x0AD, 0, 0),
            time: sample_eol_frame(128, 0x2D, 0x08, 0x0B9, 0, 0),
            under: sample_eol_frame(129, 0x40, 0x08, 0x0C3, 0, 0),
            digits: (0..10)
                .map(|digit| {
                    sample_eol_frame(
                        130 + digit,
                        0x0C,
                        0x0B,
                        0x0CB + digit as u16,
                        if digit == 1 { -3 } else { 0 },
                        0,
                    )
                })
                .collect(),
        }
    }

    fn sample_background_visuals() -> BackgroundVisuals {
        BackgroundVisuals::Chunked(ChunkedBackgroundVisuals {
            placements: vec![BackgroundPlacement {
                chunk_id: 0,
                x_tiles: 0,
                y_tiles: 0,
            }],
            chunks: vec![BackgroundChunk {
                width_tiles: 2,
                height_tiles: 1,
                tiles: vec![
                    BackgroundTile {
                        tile_index: 0,
                        flip_x: false,
                    },
                    BackgroundTile {
                        tile_index: 1,
                        flip_x: true,
                    },
                ],
            }],
        })
    }

    fn sample_level_visuals_with_background(background: [u8; 3]) -> NativeLevelVisuals {
        let foreground = (0..16)
            .map(|skin| sample_tile_quad(0x00BC + (skin as u16 * 4)))
            .collect();
        let mut blocks = Vec::with_capacity(81);
        for skin in 0..81 {
            let base = 0x0220 + (skin as u16 * 4);
            blocks.push(sample_tile_quad(base));
        }
        let foreground_layout_bytes = sample_foreground_layout_bytes();
        let foreground_layout =
            ForegroundLayout::parse(&foreground_layout_bytes, 1, 1).expect("foreground layout");
        let mut block_tiles = blank_tile_sheet(0x200);
        block_tiles.tiles[usize::from(0x025A - BLOCK_TILE_VRAM_BASE)].pixels = [237; 64];
        NativeLevelVisuals {
            foreground_tiles: blank_tile_sheet(0x80),
            background_tiles: blank_tile_sheet(0x20),
            island_shore_tiles_loc_1202a: None,
            block_tiles,
            palettes: PaletteSet {
                foreground: Palette {
                    colours: vec![colour(12, 34, 56)],
                },
                background: Palette {
                    colours: vec![MdColour {
                        red: background[0],
                        green: background[1],
                        blue: background[2],
                        raw_word: 0,
                    }],
                },
            },
            background: sample_background_visuals(),
            level: LevelVisuals {
                level_id: 0x00,
                foreground_theme_id: 0x08,
                background_theme_id: 0x08,
                murder_wall_flags: 0,
                background_scroll: None,
                foreground_layout,
                foreground_layout_bytes,
                block_layout_bytes: Vec::new(),
                background_layout_bytes: Vec::new(),
                background_layout_alias: None,
                block_mapping: BlockVisualMapping::new(foreground, blocks)
                    .expect("sample block mapping"),
            },
            hud: HudVisuals {
                digits: blank_tile_sheet(12),
                digit_base_tile_index: 0x06BA,
                diamond_tiles: blank_tile_sheet(16),
                diamond_base_tile_index: 0x06EA,
                hitpoint_sheet: sample_hitpoint_tile_sheet(),
                icons: vec![
                    HudIcon::new("ankh", (0x01A1, 0x008D), TileRef::from_attr_word(0x8231)),
                    HudIcon::new("colon", (0x0090, 0x0090), TileRef::from_attr_word(0x86C4)),
                    HudIcon::new("diamond", (0x01A0, 0x00A2), TileRef::from_attr_word(0x86F2)),
                ],
                hitpoint_tiles: [
                    sample_tile_ref(0x022D),
                    sample_tile_ref(0x022E),
                    sample_tile_ref(0x022F),
                ],
            },
            player: sample_player_visuals(),
            juggernaut_skull: None,
            enemies: sample_enemy_visuals(),
            floating_prizes: sample_floating_prize_visuals(),
            flag: sample_flag_visuals(),
            murder_wall: None,
            lava: None,
            weather: None,
            special_effect_unknown: None,
            platforms: sample_platform_visuals(),
            eol_score: Some(sample_eol_score_visuals()),
            eol_hologram: None,
            pause_menu: None,
            title_card_letters: None,
        }
    }

    fn sample_level_visuals() -> NativeLevelVisuals {
        sample_level_visuals_with_background([33, 66, 99])
    }

    fn cached_generated_manifest() -> Option<AssetManifest> {
        let repo_root = if let Ok(override_root) = std::env::var("KID_REPO_ROOT")
            && !override_root.is_empty()
        {
            PathBuf::from(override_root)
        } else {
            let crate_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
            crate_dir
                .parent()
                .and_then(Path::parent)
                .expect("wildside-cli crate should live under <repo>/crates/wildside-cli")
                .to_path_buf()
        };
        let manifest_path = repo_root
            .join(".cache/generated/28b904000b2863b6760531807760b571f1a5fc1d/asset-manifest.json");
        if !manifest_path.is_file() {
            eprintln!(
                "skipping cache-backed native-player test: generated asset manifest missing at {}",
                manifest_path.display()
            );
            return None;
        }
        Some(load_manifest(&manifest_path).expect("load cached generated asset manifest"))
    }

    fn render_vdp_frame_for_test(
        runtime: &NativeRuntime,
        trace: &NativeTraceFrame,
        level_visuals: &NativeLevelVisuals,
    ) -> Vec<u8> {
        let vdp = runtime
            .vdp_state()
            .expect("test route should expose a VDP shell");
        let snapshot = wildside_md::VdpSnapshot {
            registers: vdp.registers,
            vram: vdp.vram.clone(),
            cram: vdp.cram.clone(),
            vsram: vdp.vsram.clone(),
        };
        let mut frame = vec![0; (FRAME_WIDTH * FRAME_HEIGHT * 3) as usize];
        wildside_md::render_vdp_into(&snapshot, &mut frame);
        render_vdp_trace_overlay(trace, level_visuals, &mut frame);
        frame
    }

    #[test]
    fn render_native_vdp_rgb_produces_non_blank_buffer() {
        let Some(manifest) = cached_generated_manifest() else {
            return;
        };
        // Step the Sega/title sequence (default input) until the engine exposes a
        // VDP shell (vdp_state Some on non-gameplay screens, lib.rs:2491) that
        // renders visible pixels — the Sega logo fades in, so early frames can be
        // legitimately black.
        let mut runtime = NativeRuntime::new(manifest);
        let mut rgb = vec![0u8; (FRAME_WIDTH * FRAME_HEIGHT * 3) as usize];
        let mut non_blank = false;
        for _ in 0..400 {
            runtime.step_frame(FrameInput::default());
            if let Some(vdp) = runtime.vdp_state() {
                let vdp = vdp.clone();
                render_native_vdp_rgb(&vdp, &mut rgb);
                if rgb.iter().any(|&byte| byte != 0) {
                    non_blank = true;
                    break;
                }
            }
        }
        assert!(
            non_blank,
            "a Sega/title VDP frame should render non-black pixels",
        );
    }

    #[test]
    fn diamond_enemy_animates_while_stationary_stru_3aca8() {
        let mut trace = sample_trace(GameMode::InGame);
        let visuals = sample_level_visuals();
        let diamond_visuals = visuals
            .enemies
            .families
            .get(&0x01)
            .expect("sample Diamond visuals");
        let object = NativeObjectSnapshot {
            kind: NativeObjectKind::Enemy { object_type: 0x01 },
            x_pos: 0x0040_0000,
            y_pos: 0x0030_0000,
            layout_index: Some(0),
            x_velocity: Some(0),
            y_velocity: Some(0),
            facing: Some(Direction::Right),
            on_ground: None,
            hit_points: Some(2),
            damage_flash: false,
            palette_variant: 0,
            stationary: true,
            contact_phase: None,
            shared_walker_state: None,
            shared_walker_patrol_timer: None,
            shared_walker_patrol_short_next: None,
            walk_phase: None,
            wait_timer: None,
            debug_fields: std::collections::BTreeMap::new(),
        };

        trace.frame_index = 0;
        let first = sprites::select_enemy_frame(&trace, diamond_visuals, &object);
        trace.frame_index = 6;
        let second = sprites::select_enemy_frame(&trace, diamond_visuals, &object);

        assert_eq!(
            first.tile_base_index, 0x02F,
            "ASM Diamond.asm Enemy01_Diamond_Init starts base animation `stru_3ACA8` on C80B8 / tile base `$02F`"
        );
        assert_eq!(
            second.tile_base_index, 0x03B,
            "ASM Diamond.asm `stru_3ACA8` advances after 6 ticks to C80C0 / tile base `$03B`"
        );
        assert_ne!(
            first.tile_sheet.tiles[0].pixels, second.tile_sheet.tiles[0].pixels,
            "ASM Diamond.asm `stru_3ACA8` advances every 6 ticks even though the low-HP Diamond uses the shared ledge-walker path"
        );

        trace.frame_index = 120;
        let mut later_layout = object.clone();
        later_layout.layout_index = Some(8);
        let later = sprites::select_enemy_frame(&trace, diamond_visuals, &later_layout);
        assert_eq!(
            later.tile_base_index, 0x02F,
            "ASM Diamond.asm `stru_3ACA8` loops over the five live frames only; layout-index desync shows C8180 on issue289 level 0D frame 120 instead of the oracle's first crystal frame"
        );
    }

    #[test]
    fn live_tar_sphere_emorock_hand_hold_body_frames_not_death_tail() {
        // ASM Tar_Monster.asm off_32290 / Sphere.asm stru_36174 / Emo_Rock.asm
        // loc_3621A / Hand.asm loc_327F2/loc_328E0/loc_32906: a LIVE enemy
        // must hold its body/phase frame, never free-run the extracted sheet's
        // DEATH-tail frames (#243/#333/#361 render free-run).
        // Before the select_live_* helpers, the generic free-run cycled the whole
        // sheet off trace.frame_index, exposing the submerge/disappear/shatter
        // death frames on a live enemy.
        let visuals = sample_level_visuals();
        let mut trace = sample_trace(GameMode::InGame);
        let base = |object_type: u8| NativeObjectSnapshot {
            kind: NativeObjectKind::Enemy { object_type },
            x_pos: 0x0040_0000,
            y_pos: 0x0010_0000,
            layout_index: Some(0),
            x_velocity: Some(0),
            y_velocity: Some(0),
            facing: Some(Direction::Right),
            on_ground: Some(false),
            hit_points: Some(1),
            damage_flash: false,
            palette_variant: 0,
            stationary: false,
            contact_phase: None,
            shared_walker_state: None,
            shared_walker_patrol_timer: None,
            shared_walker_patrol_short_next: None,
            walk_phase: None,
            wait_timer: None,
            debug_fields: std::collections::BTreeMap::new(),
        };

        // Tar Monster 0x05: hidden phase holds frame 0 (C77A6, sig 59); visible
        // phase holds frame 1 (C77D6, sig 60). Never the 61..=65 submerge tail.
        // frame_index 12 makes the pre-fix free-run pick frame (12/3)%7 = 4
        // (sig 63, a death frame), so these phase assertions fail without the helper.
        trace.frame_index = 12;
        let tar = visuals.enemies.families.get(&0x05).expect("tar visuals");
        let mut tar_hidden = base(0x05);
        tar_hidden.contact_phase = Some(NativeContactPhase::TarMonsterHidden);
        assert_eq!(
            sprites::select_enemy_frame(&trace, tar, &tar_hidden)
                .tile_sheet
                .tiles[0]
                .pixels[0],
            59,
            "ASM Tar_Monster.asm stru_3274C: hidden tar holds the underground body frame, not the submerge death tail"
        );
        let tar_visible = base(0x05); // contact_phase None => Visible/Attacking
        assert_eq!(
            sprites::select_enemy_frame(&trace, tar, &tar_visible)
                .tile_sheet
                .tiles[0]
                .pixels[0],
            60,
            "ASM Tar_Monster.asm stru_3272E: visible tar holds the emerged body frame C77D6"
        );

        // Sphere 0x06: live spin uses only frames 0,1,2 (C8120/C8128/C8130, sigs
        // 151/152/153). Never the 154..=156 stru_36182 disappear tail.
        // frame_index 27 makes the pre-fix free-run pick frame (27/9)%6 = 3
        // (sig 154, a death frame), so this assertion fails without the helper.
        trace.frame_index = 27;
        let sphere = visuals.enemies.families.get(&0x06).expect("sphere visuals");
        let sphere_sig = sprites::select_enemy_frame(&trace, sphere, &base(0x06))
            .tile_sheet
            .tiles[0]
            .pixels[0];
        assert!(
            matches!(sphere_sig, 151..=153),
            "ASM Sphere.asm stru_36174: live sphere spins frames 0-2 only, not the stru_36182 disappear tail (got sig {sphere_sig})"
        );

        // EmoRock 0x1B: live rock holds a body frame 0,1,2 (C7F40/C7F58/C7F60,
        // sigs 241/242/243). Never the 244..=248 stru_36468 shatter tail the
        // free-run was cycling. frame_index 60 makes the pre-fix free-run pick
        // frame (60/20)%8 = 3 (sig 244, a shatter frame), so this fails without
        // the helper.
        trace.frame_index = 60;
        let emo = visuals
            .enemies
            .families
            .get(&0x1B)
            .expect("emo rock visuals");
        let emo_sig = sprites::select_enemy_frame(&trace, emo, &base(0x1B))
            .tile_sheet
            .tiles[0]
            .pixels[0];
        assert!(
            matches!(emo_sig, 241..=243),
            "ASM Emo_Rock.asm loc_3621A/loc_362A4: live emo rock shows a body frame, not the stru_36468 shatter tail (got sig {emo_sig})"
        );

        // Hand 0x17: walking installs `stru_32B6A` (frame 0), jump attack
        // installs `stru_32B56` (frames 1/2, frame 1 at script entry), and grab
        // installs `stru_32B50` (frame 3). None may free-run into the `loc_32AA2`
        // death tail frames 4..7 while hit_points is nonzero. frame_index 6 makes
        // the pre-fix free-run pick frame (6/1)%8 = 6, a death sliver.
        trace.frame_index = 6;
        let hand = visuals.enemies.families.get(&0x17).expect("hand visuals");
        let hand_walking = base(0x17);
        assert_eq!(
            sprites::select_enemy_frame(&trace, hand, &hand_walking)
                .tile_sheet
                .tiles[0]
                .pixels[0],
            141,
            "ASM Hand.asm loc_327F2 installs `stru_32B6A`; live walking Hand holds the body frame, not the death tail"
        );
        let mut hand_jumping = base(0x17);
        hand_jumping.contact_phase = Some(NativeContactPhase::HandJumping);
        assert_eq!(
            sprites::select_enemy_frame(&trace, hand, &hand_jumping)
                .tile_sheet
                .tiles[0]
                .pixels[0],
            142,
            "ASM Hand.asm loc_328E0 installs `stru_32B56`; live jumping Hand starts on the attack frame, not the death tail"
        );
        let mut hand_grabbing = base(0x17);
        hand_grabbing.contact_phase = Some(NativeContactPhase::HandGrabbing);
        assert_eq!(
            sprites::select_enemy_frame(&trace, hand, &hand_grabbing)
                .tile_sheet
                .tiles[0]
                .pixels[0],
            144,
            "ASM Hand.asm loc_32906 installs `stru_32B50`; live grabbing Hand holds the grab frame"
        );
    }

    fn sample_foreground_layout_bytes() -> Vec<u8> {
        let mut bytes = vec![0; 20 * 14];
        bytes[0] = 0;
        bytes[1] = 3;
        bytes[20] = 5;
        bytes
    }

    fn sample_flag_visuals() -> wildside_render::FlagVisuals {
        let palette = Palette {
            colours: vec![colour(12, 34, 56)],
        };
        wildside_render::FlagVisuals {
            palette: palette.clone(),
            pole_bottom: wildside_render::SpriteFrame {
                origin_x: 8,
                origin_y: 16,
                width_px: 16,
                height_px: 16,
                tile_base_index: 0,
                tile_sheet: blank_tile_sheet(4),
            },
            waving: (0..6)
                .map(|frame| wildside_render::SpriteFrame {
                    origin_x: 6,
                    origin_y: 0,
                    width_px: 40,
                    height_px: 32,
                    tile_base_index: frame * 20,
                    tile_sheet: blank_tile_sheet(120),
                })
                .collect(),
            frame_duration: 6,
        }
    }

    fn build_ingame_scene(
        trace: &NativeTraceFrame,
        debug_overlay_visible: bool,
        block_grid: Option<&BlockGrid>,
    ) -> Scene {
        let visuals = sample_level_visuals();
        build_scene(trace, debug_overlay_visible, Some(&visuals), block_grid)
    }

    fn sample_enemy_object(object_type: u8, x_pos: u32, y_pos: u32) -> NativeObjectSnapshot {
        NativeObjectSnapshot {
            kind: NativeObjectKind::Enemy { object_type },
            x_pos,
            y_pos,
            layout_index: Some(0),
            x_velocity: Some(0),
            y_velocity: Some(0),
            facing: Some(Direction::Right),
            on_ground: None,
            hit_points: Some(1),
            damage_flash: false,
            palette_variant: 0,
            stationary: false,
            contact_phase: None,
            shared_walker_state: None,
            shared_walker_patrol_timer: None,
            shared_walker_patrol_short_next: None,
            walk_phase: None,
            wait_timer: None,
            debug_fields: std::collections::BTreeMap::new(),
        }
    }

    #[test]
    fn build_scene_falls_back_to_enemy_rect_for_empty_loaded_family_issue322() {
        let mut visuals = sample_level_visuals();
        visuals.enemies.families.insert(
            0x0C,
            AnimatedSpriteVisual {
                palette: Palette {
                    colours: vec![colour(0, 0, 0), colour(200, 30, 10)],
                },
                extra_palettes: vec![],
                frames: Vec::new(),
                frame_duration: 6,
            },
        );
        let mut trace = sample_trace(GameMode::InGame);
        trace.active_objects = vec![sample_enemy_object(0x0C, 0x0040_0000, 0x0030_0000)];

        let scene = build_scene(&trace, false, Some(&visuals), None);

        assert!(
            scene.commands.iter().any(|command| {
                matches!(
                    command,
                    DrawCommand::Rect {
                        width: 16,
                        height: 16,
                        colour,
                        ..
                    } if *colour == enemy_family_colour(0x0C)
                )
            }),
            "#322: a loaded but empty enemy family must use the total rectangle fallback instead of indexing frames[0]"
        );
    }

    #[test]
    fn build_scene_skips_empty_effect_enemy_family_issue322() {
        let mut visuals = sample_level_visuals();
        visuals.enemies.families.insert(
            0x21,
            AnimatedSpriteVisual {
                palette: Palette {
                    colours: vec![colour(0, 0, 0), colour(160, 160, 160)],
                },
                extra_palettes: vec![],
                frames: Vec::new(),
                frame_duration: 6,
            },
        );
        let mut trace = sample_trace(GameMode::InGame);
        trace.flag_x = None;
        trace.flag_y = None;
        trace.heady_metal_death_splits = vec![wildside_core::HeadyMetalDeathSplitTrace {
            current_x_velocity_delta: -0x0002_6D18,
            left_worker_x_velocity_delta: 0x0002_6D18,
            centre_worker_y_velocity_delta: 0x0001_3880,
            centre_worker_sprite_label: Some("LnkTo_unk_C8458-Data_Index".to_owned()),
            right_worker_x_velocity_delta: -0x0002_6D18,
            far_right_worker_x_velocity_delta: 0x0002_6D18,
            cleanup_worker_slot_offsets: [0xFB72, 0xFB76, 0xFB7A, 0xFBAE, 0xFBB2],
            pieces: vec![wildside_core::HeadyMetalDeathPieceTrace {
                slot_offset: 0xFB72,
                x_pos: 100 << 16,
                y_pos: 120 << 16,
                x_velocity: -0x0002_6D18,
                y_velocity: 0,
            }],
        }];

        let scene = build_scene(&trace, false, Some(&visuals), None);

        assert!(
            scene.commands.iter().all(|command| {
                !matches!(
                    command,
                    DrawCommand::Sprite { x, y, .. } if *x == 100 && *y == 120
                )
            }),
            "#322: effect-only enemy art with no frames should be skipped instead of indexing frames[0]"
        );
    }

    #[test]
    fn under_skull_mountain3_present_flying_dragon_renders_rom_sprite_issue322() {
        let Some(manifest) = cached_generated_manifest() else {
            return;
        };
        let session_assets = load_native_session_assets(&manifest, 0x06)
            .expect("Level 0x06 generated native-player assets");
        let dragon_visuals = session_assets
            .level_visuals
            .enemies
            .families
            .get(&0x0C)
            .expect("Under Skull Mountain 3 should load Flying Dragon visuals");
        assert!(
            !dragon_visuals.frames.is_empty(),
            "ASM Dragon_Flying.asm Enemy0C_Dragon_flying_Init must have decoded sprite frames in generated Level 0x06 assets"
        );
        let mut trace = sample_trace(GameMode::InGame);
        trace.current_level = LevelId(0x06);
        trace.current_level_name = Some(String::from("Under Skull Mountain 3"));
        trace.camera_x_pos = 384 << 16;
        trace.camera_y_pos = 160 << 16;
        trace.active_objects = vec![NativeObjectSnapshot {
            kind: NativeObjectKind::Enemy { object_type: 0x0C },
            x_pos: 504 << 16,
            y_pos: 239 << 16,
            layout_index: Some(0),
            x_velocity: Some(-0x0002_0000),
            y_velocity: Some(-0x0003_0000),
            facing: Some(Direction::Left),
            on_ground: Some(false),
            hit_points: Some(1),
            damage_flash: false,
            palette_variant: 0,
            stationary: false,
            contact_phase: None,
            shared_walker_state: None,
            shared_walker_patrol_timer: None,
            shared_walker_patrol_short_next: None,
            walk_phase: None,
            wait_timer: None,
            debug_fields: std::collections::BTreeMap::new(),
        }];

        let scene = build_scene(
            &trace,
            false,
            Some(&session_assets.level_visuals),
            Some(&session_assets.block_grid),
        );

        assert!(
            scene.commands.iter().any(|command| {
                matches!(
                    command,
                    DrawCommand::Sprite { x, y, tiles, .. }
                        if *x >= 80 && *x <= 140 && *y >= 40 && *y <= 100 && !tiles.is_empty()
                )
            }),
            "ASM Dragon_Flying.asm loc_348CE/stru_34600: a present Level 0x06 Flying Dragon must render a ROM sprite, not disappear or fall back to an empty-frame path"
        );
        assert!(
            scene.commands.iter().all(|command| {
                !matches!(
                    command,
                    DrawCommand::Rect {
                        x,
                        y,
                        width: 16,
                        height: 16,
                        ..
                    } if *x == 120 && *y == 79
                )
            }),
            "Level 0x06 Flying Dragon should use decoded Dragon_Flying.asm frames instead of the #322 empty-art rectangle fallback"
        );
    }

    struct ManifestFixture {
        _root: TempDir,
        manifest: AssetManifest,
    }

    impl ManifestFixture {
        fn new() -> anyhow::Result<Self> {
            let root = TempDir::new()?;
            let decoded_path = "decoded/level/00/block.bin";
            let legacy_path = "level-layout/00.bin";
            let legacy_metadata_path = "level-layout/00.json";
            let width_cells = 20usize;
            let height_cells = 14usize;
            let mut decoded_bytes = vec![0u8; width_cells * height_cells * 2];
            decoded_bytes[0] = 0xE1;
            decoded_bytes[1] = 0x05;
            decoded_bytes[2] = 0x60;
            decoded_bytes[3] = 0x07;
            let mut legacy_bytes = vec![0u8; width_cells * height_cells * 2];
            legacy_bytes[0] = 0x70;
            legacy_bytes[1] = 0x33;
            write_file(root.path(), decoded_path, &decoded_bytes)?;
            write_file(root.path(), legacy_path, &legacy_bytes)?;
            write_file(
                root.path(),
                legacy_metadata_path,
                br#"{"data_bytes":4,"width_blocks":20,"height_blocks":14}"#,
            )?;
            write_file(root.path(), "level-layout/00.blocklist.bin", &[0x00, 0x00])?;

            Ok(Self {
                manifest: AssetManifest {
                    generated_bundle_version: GENERATED_BUNDLE_VERSION,
                    rom_sha1: String::from("fixture"),
                    upstream_commit: String::from("fixture"),
                    source_root: root.path().to_path_buf(),
                    output_root: root.path().to_path_buf(),
                    map_order: Vec::new(),
                    levels: vec![LevelRecord {
                        level_id: 0x00,
                        title_text_id: Some(0x4A),
                        display_name: Some(String::from("Blue Lake Woods 1")),
                        title_card: None,
                        resource_paths: Vec::new(),
                        resources: LevelResources::default(),
                        header_metadata: Some(LevelHeader {
                            width_screens: 1,
                            height_screens: 1,
                            height_flag_bits: 0,
                            level_theme: 0x08,
                            murder_wall_flags: 0,
                            background_theme: 0x08,
                            weather_flags: 0,
                            player_start_x: 0x0750,
                            player_start_y: 0x00E0,
                            flag_x: 0x0750,
                            flag_y: 0x0300,
                        }),
                        enemy_layout_metadata: None,
                        bgscroll_metadata: None,
                        platform_metadata: None,
                        background_layout_alias: None,
                    }],
                    constants: Vec::new(),
                    ram_variables: Vec::new(),
                    enemy_sources: Vec::new(),
                    theme_assets: Vec::new(),
                    scene_assets: Vec::new(),
                    animation_assets: Vec::new(),
                    sound_assets: Vec::new(),
                    ingame_assets: Vec::new(),
                    decoded_levels: vec![DecodedLevelAssets {
                        contract_version: 3,
                        level_id: 0x00,
                        foreground: decoded_asset("decoded/level/00/foreground.bin"),
                        block: decoded_asset(decoded_path),
                        background: decoded_asset("decoded/level/00/background.bin"),
                    }],
                },
                _root: root,
            })
        }

        fn write_dummy_gems_assets(&self) -> anyhow::Result<()> {
            for (relative_path, len) in [
                ("sound/GEMS_Sounddriver.bin", 64usize),
                ("sound/instruments.bin", 16usize),
                ("sound/envelopes.bin", 16usize),
                ("sound/sequences.bin", 16usize),
                ("sound/samples.bin", 16usize),
            ] {
                write_file(&self.manifest.output_root, relative_path, &vec![0; len])?;
            }
            Ok(())
        }
    }

    fn decoded_asset(relative_path: &str) -> DecodedAssetFile {
        DecodedAssetFile {
            relative_path: relative_path.to_string(),
            preview_png_path: format!("{relative_path}.png"),
            size_bytes: 0,
            sha1: String::from("fixture"),
        }
    }

    fn write_file(root: &Path, relative_path: &str, bytes: &[u8]) -> anyhow::Result<()> {
        let path = root.join(relative_path);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(path, bytes)?;
        Ok(())
    }

    #[test]
    fn render_indexed_tiles_applies_palette_and_transparency() {
        let tile_sheet = Arc::new(TileSheet {
            tiles: vec![make_tile(&[(0, 0, 1), (1, 0, 2), (2, 0, 0), (3, 0, 3)])],
        });
        let palette = Arc::new(Palette {
            colours: (0..15)
                .map(|value| colour(value, value.saturating_add(1), value.saturating_add(2)))
                .chain([
                    colour(110, 120, 130),
                    colour(140, 150, 160),
                    colour(170, 180, 190),
                ])
                .collect(),
        });
        let scene = Scene {
            background: [1, 2, 3],
            commands: vec![DrawCommand::IndexedTile {
                x: 12,
                y: 7,
                tile_sheet,
                tile: TileRef {
                    tile_index: 0,
                    palette_line: 1,
                    priority: false,
                    flip_x: false,
                    flip_y: false,
                },
                palette,
            }],
            pause_palette_darken: false,
        };
        let mut rgb = vec![0; (FRAME_WIDTH * FRAME_HEIGHT * 3) as usize];

        render_scene(&scene, &mut rgb);

        assert_eq!(pixel_at(&rgb, 12, 7), [110, 120, 130]);
        assert_eq!(pixel_at(&rgb, 13, 7), [140, 150, 160]);
        assert_eq!(pixel_at(&rgb, 14, 7), [1, 2, 3]);
        assert_eq!(pixel_at(&rgb, 15, 7), [170, 180, 190]);
    }

    #[test]
    fn render_tile_command_honours_flip_and_camera_clipping() {
        let tile_sheet = Arc::new(TileSheet {
            tiles: vec![make_tile(&[(0, 0, 1), (7, 7, 2)])],
        });
        let palette = Arc::new(Palette {
            colours: vec![colour(200, 10, 10), colour(10, 200, 10)],
        });
        let scene = Scene {
            background: [3, 4, 5],
            commands: vec![DrawCommand::IndexedTile {
                x: FRAME_WIDTH as i32 - 1,
                y: FRAME_HEIGHT as i32 - 1,
                tile_sheet,
                tile: TileRef {
                    tile_index: 0,
                    palette_line: 0,
                    priority: false,
                    flip_x: true,
                    flip_y: true,
                },
                palette,
            }],
            pause_palette_darken: false,
        };
        let mut rgb = vec![0; (FRAME_WIDTH * FRAME_HEIGHT * 3) as usize];

        render_scene(&scene, &mut rgb);

        assert_eq!(
            pixel_at(&rgb, FRAME_WIDTH - 1, FRAME_HEIGHT - 1),
            [10, 200, 10]
        );
        assert!(!rgb.chunks_exact(3).any(|pixel| pixel == [200, 10, 10]));
    }

    #[test]
    fn render_background_palette_uses_backdrop_slot_zero_without_shifting_visible_colours() {
        let tile_sheet = Arc::new(TileSheet {
            tiles: vec![make_tile(&[(0, 0, 1), (1, 0, 7)])],
        });
        let palette = Arc::new(Palette {
            colours: vec![
                colour(1, 2, 3),
                colour(10, 20, 30),
                colour(40, 50, 60),
                colour(70, 80, 90),
                colour(100, 110, 120),
                colour(130, 140, 150),
                colour(160, 170, 180),
                colour(190, 200, 210),
            ],
        });
        let scene = Scene {
            background: [1, 2, 3],
            commands: vec![DrawCommand::IndexedTile {
                x: 0,
                y: 0,
                tile_sheet,
                tile: TileRef {
                    tile_index: 0,
                    palette_line: 0,
                    priority: false,
                    flip_x: false,
                    flip_y: false,
                },
                palette,
            }],
            pause_palette_darken: false,
        };
        let mut rgb = vec![0; (FRAME_WIDTH * FRAME_HEIGHT * 3) as usize];

        render_scene(&scene, &mut rgb);

        assert_eq!(pixel_at(&rgb, 0, 0), [10, 20, 30]);
        assert_eq!(pixel_at(&rgb, 1, 0), [190, 200, 210]);
    }

    #[test]
    fn render_background_palette_supports_plane_line_offsets() {
        let tile_sheet = Arc::new(TileSheet {
            tiles: vec![make_tile(&[(0, 0, 8), (1, 0, 15)])],
        });
        let palette = Arc::new(Palette {
            colours: vec![
                colour(1, 2, 3),
                colour(10, 20, 30),
                colour(40, 50, 60),
                colour(70, 80, 90),
                colour(100, 110, 120),
                colour(130, 140, 150),
                colour(160, 170, 180),
                colour(190, 200, 210),
            ],
        });
        let scene = Scene {
            background: [1, 2, 3],
            commands: vec![DrawCommand::IndexedTile {
                x: 0,
                y: 0,
                tile_sheet,
                tile: TileRef {
                    tile_index: 0,
                    palette_line: 1,
                    priority: false,
                    flip_x: false,
                    flip_y: false,
                },
                palette,
            }],
            pause_palette_darken: false,
        };
        let mut rgb = vec![0; (FRAME_WIDTH * FRAME_HEIGHT * 3) as usize];

        render_scene(&scene, &mut rgb);

        assert_eq!(pixel_at(&rgb, 0, 0), [1, 2, 3]);
        assert_eq!(pixel_at(&rgb, 1, 0), [190, 200, 210]);
    }

    #[test]
    fn render_full_16_colour_sprite_palette_keeps_rom_slot_numbers() {
        // ASM loc_1AB40 copies `Pal_19C48` as all 16 words of the title-card
        // letter palette. Pixel index 2 must resolve to palette slot 2; shifting
        // it down to slot 1 makes the Blue Lake Woods title glyphs black.
        let tile_sheet = Arc::new(TileSheet {
            tiles: vec![make_tile(&[(0, 0, 0), (1, 0, 1), (2, 0, 2)])],
        });
        let palette = Arc::new(Palette {
            colours: vec![
                colour(20, 20, 60),
                colour(0, 0, 0),
                colour(64, 96, 192),
                colour(80, 120, 220),
                colour(0, 0, 0),
                colour(0, 0, 0),
                colour(0, 0, 0),
                colour(0, 0, 0),
                colour(0, 0, 0),
                colour(0, 0, 0),
                colour(0, 0, 0),
                colour(0, 0, 0),
                colour(0, 0, 0),
                colour(0, 0, 0),
                colour(0, 0, 0),
                colour(0, 0, 0),
            ],
        });
        let scene = Scene {
            background: [1, 2, 3],
            commands: vec![DrawCommand::IndexedTile {
                x: 0,
                y: 0,
                tile_sheet,
                tile: TileRef {
                    tile_index: 0,
                    palette_line: 0,
                    priority: false,
                    flip_x: false,
                    flip_y: false,
                },
                palette,
            }],
            pause_palette_darken: false,
        };
        let mut rgb = vec![0; (FRAME_WIDTH * FRAME_HEIGHT * 3) as usize];

        render_scene(&scene, &mut rgb);

        assert_eq!(pixel_at(&rgb, 0, 0), [1, 2, 3]);
        assert_eq!(pixel_at(&rgb, 1, 0), [0, 0, 0]);
        assert_eq!(pixel_at(&rgb, 2, 0), [64, 96, 192]);
    }

    #[test]
    fn render_sprite_command_honours_mapping_priority_and_transparency() {
        let tile_sheet = Arc::new(TileSheet {
            tiles: vec![
                make_tile(&[(0, 0, 1)]),
                make_tile(&[(0, 0, 2)]),
                make_tile(&[(0, 0, 4), (1, 0, 4)]),
            ],
        });
        let palette = Arc::new(Palette {
            colours: vec![
                colour(12, 34, 56),
                colour(78, 90, 12),
                colour(140, 150, 160),
                colour(210, 220, 230),
            ],
        });
        let scene = Scene {
            background: [9, 8, 7],
            commands: vec![
                DrawCommand::Sprite {
                    x: 20,
                    y: 30,
                    tile_sheet: tile_sheet.clone(),
                    palette: palette.clone(),
                    tiles: vec![
                        SpriteTile {
                            offset_x: 0,
                            offset_y: 0,
                            tile: TileRef {
                                tile_index: 1,
                                palette_line: 0,
                                priority: true,
                                flip_x: false,
                                flip_y: false,
                            },
                        },
                        SpriteTile {
                            offset_x: 0,
                            offset_y: 0,
                            tile: TileRef {
                                tile_index: 0,
                                palette_line: 0,
                                priority: false,
                                flip_x: false,
                                flip_y: false,
                            },
                        },
                    ],
                },
                DrawCommand::IndexedTile {
                    x: 20,
                    y: 30,
                    tile_sheet,
                    tile: TileRef {
                        tile_index: 2,
                        palette_line: 0,
                        priority: false,
                        flip_x: false,
                        flip_y: false,
                    },
                    palette,
                },
            ],
            pause_palette_darken: false,
        };
        let mut rgb = vec![0; (FRAME_WIDTH * FRAME_HEIGHT * 3) as usize];

        render_scene(&scene, &mut rgb);

        assert_eq!(pixel_at(&rgb, 20, 30), [78, 90, 12]);
        assert_eq!(pixel_at(&rgb, 21, 30), [210, 220, 230]);
        assert!(!rgb.chunks_exact(3).any(|pixel| pixel == [12, 34, 56]));
    }

    #[test]
    fn render_priority_tiles_preserve_scene_order_for_legacy_commands() {
        let tile_sheet = Arc::new(TileSheet {
            tiles: vec![make_tile(&[(0, 0, 1), (1, 0, 1)]), make_tile(&[(1, 0, 2)])],
        });
        let palette = Arc::new(Palette {
            colours: vec![colour(200, 10, 10), colour(20, 220, 30)],
        });
        let scene = Scene {
            background: [9, 8, 7],
            commands: vec![
                DrawCommand::IndexedTile {
                    x: 40,
                    y: 50,
                    tile_sheet: tile_sheet.clone(),
                    tile: TileRef {
                        tile_index: 0,
                        palette_line: 0,
                        priority: true,
                        flip_x: false,
                        flip_y: false,
                    },
                    palette: palette.clone(),
                },
                DrawCommand::Rect {
                    x: 40,
                    y: 50,
                    width: 2,
                    height: 1,
                    colour: [60, 70, 80],
                },
                DrawCommand::IndexedTile {
                    x: 40,
                    y: 50,
                    tile_sheet,
                    tile: TileRef {
                        tile_index: 1,
                        palette_line: 0,
                        priority: false,
                        flip_x: false,
                        flip_y: false,
                    },
                    palette,
                },
            ],
            pause_palette_darken: false,
        };
        let mut rgb = vec![0; (FRAME_WIDTH * FRAME_HEIGHT * 3) as usize];

        render_scene(&scene, &mut rgb);

        assert_eq!(pixel_at(&rgb, 40, 50), [60, 70, 80]);
        assert_eq!(pixel_at(&rgb, 41, 50), [20, 220, 30]);
    }

    #[test]
    fn build_scene_uses_loaded_level_visuals_for_ingame() {
        let trace = sample_trace(GameMode::InGame);
        let visuals = sample_level_visuals_with_background([12, 34, 56]);

        let scene = build_scene(&trace, false, Some(&visuals), None);

        assert_eq!(scene.background, [12, 34, 56]);
    }

    #[test]
    fn build_scene_draws_prize_block_wobble_sprite_on_source_frame() {
        // ASM sub_1007A/loc_10200 writes `$6Fxx` to Level_Layout while the
        // first source-position frame still displays the copied block art
        // before the later hidden deleted-tile wobble frames.
        let mut trace = sample_trace(GameMode::InGame);
        trace.camera_x_pos = 0;
        trace.camera_y_pos = 0;
        trace.block_bounces = vec![(1, 0, 31, 0)];
        trace.block_bounce_details = vec![BlockBounceTrace {
            cell_x: 1,
            cell_y: 0,
            x_offset_px: 0,
            remaining: 31,
            y_offset_px: 0,
            kind: BlockBounceTraceKind::Prize,
        }];
        let visuals = sample_level_visuals();
        let mut data = vec![0u8; 20 * 14 * 2];
        data[2] = 0x6F;
        data[3] = 0x00;
        let grid = BlockGrid::from_level_layout(&data, 20, 14);

        let scene = build_scene(&trace, false, Some(&visuals), Some(&grid));

        let Some(copied_wobble_tiles) = scene.commands.iter().find_map(|command| match command {
            DrawCommand::Sprite {
                x: 16,
                y: 0,
                tile_sheet,
                tiles,
                ..
            } if tile_sheet.tiles.len() == 0x200 => Some(tiles),
            _ => None,
        }) else {
            panic!(
                "source-frame sub_1007A block_bounces entry should draw the copied prize-block art from `$220/$229`, not vanish after the temporary `$6Fxx` cell write"
            );
        };
        assert_eq!(
            copied_wobble_tiles.first().map(|tile| tile.tile.tile_index),
            Some(0x008),
            "source-frame sub_1007A block_bounces entry should draw the copied `$E102` prize source quad even though the live cell is `$6F00`",
        );
        assert!(
            !scene.commands.iter().any(|command| matches!(
                command,
                DrawCommand::Sprite {
                    x: 16,
                    y: 0,
                    tile_sheet,
                    tiles,
                    ..
                } if tile_sheet.tiles.len() == 0x200
                    && tiles.first().is_some_and(|tile| tile.tile.tile_index == 0x00D)
            )),
            "ASM sub_1007A draws the copied `$E102` art; native must not reinterpret `$6F00` as hidden `$E101` art"
        );
        assert!(
            !scene.commands.iter().any(|command| matches!(
                command,
                DrawCommand::Sprite {
                    x: 16,
                    y: 0,
                    tile_sheet,
                    tiles,
                    ..
                } if tile_sheet.tiles.len() == 0x200
                    && tiles.first().is_some_and(|tile| tile.tile.tile_index == 0x015)
            )),
            "ASM sub_1007A draws the copied block art, so native must not fall back to the immutable foreground layout skin while `$6Fxx` is live"
        );
        assert!(
            !scene.commands.iter().any(|command| matches!(
                command,
                DrawCommand::Sprite {
                    x: 16,
                    y: 0,
                    tile_sheet,
                    tiles,
                    ..
                } if tile_sheet.tiles.len() == 0x200
                    && tiles.first().is_some_and(|tile| tile.tile.tile_index == 0x1C5)
            )),
            "ASM sub_1007A writes `$6Fxx` into Level_Layout only as collision state; native must not use skin `$6F` as the wobble art"
        );
        assert!(
            !scene.commands.iter().any(|command| matches!(
                command,
                DrawCommand::Sprite {
                    x: 16,
                    y: 0,
                    tile_sheet,
                    tiles,
                    ..
                } if tile_sheet.tiles.len() == 0x200
                    && tiles.first().is_some_and(|tile| tile.tile.tile_index == 0x009)
            )),
            "ASM `$220/$229` are temporary VRAM slots for copied block art; native must not render them as absolute skin-zero sheet indices"
        );
        assert!(
            indexed_tile_at_with_sheet_len(&scene, 16, 0, 0x200).is_none(),
            "the deleted `$6Fxx` phase must not resolve through Block_Mappings as a stray block tile",
        );
        assert!(
            indexed_tile_at_with_sheet_len(&scene, 16, 0, 0x80).is_none(),
            "the source frame keeps Scroll A terrain suppressed so the copied prize block remains visible"
        );
    }

    #[test]
    fn prize_block_wobble_draws_shifted_copied_tile_sub_1007a_issue324() {
        // ASM sub_1007A keeps the copied GfxObject visible while loc_100DA
        // moves y_pos during the hibernated wobble. The temporary `$6Fxx` cell
        // is collision state only; shifted frames still draw the copied block.
        let mut trace = sample_trace(GameMode::InGame);
        trace.camera_x_pos = 0;
        trace.camera_y_pos = 0;
        trace.block_bounces = vec![(1, 0, 28, -6)];
        trace.block_bounce_details = vec![BlockBounceTrace {
            cell_x: 1,
            cell_y: 0,
            x_offset_px: 0,
            remaining: 28,
            y_offset_px: -6,
            kind: BlockBounceTraceKind::Prize,
        }];
        let mut visuals = sample_level_visuals();
        let terrain_colour = colour(12, 34, 56);
        let block_colour = colour(220, 240, 96);
        visuals.palettes.foreground.colours = vec![terrain_colour, block_colour];
        for tile in &mut visuals.foreground_tiles.tiles[8..12] {
            tile.pixels = [1; 64];
        }
        for tile in &mut visuals.block_tiles.tiles[0x011..=0x014] {
            tile.pixels = [2; 64];
        }
        let data = vec![0u8; 20 * 14 * 2];
        let mut grid = BlockGrid::from_level_layout(&data, 20, 14);
        grid.set_cell_at(1, 0, Cell::from_be_bytes(0x01, 0x02));
        let priority_cell = Cell::from_be_bytes(0x01, 0x02);
        grid.set_raw_word_at(1, 0, 0x0102);

        let scene = build_scene_filtered(
            &trace,
            false,
            Some(&visuals),
            Some(&grid),
            LayerFilter {
                background: false,
                terrain: true,
                block_overrides: true,
                platforms: false,
                enemies: false,
                player: false,
                effects: false,
                eol_score: false,
                hud: false,
            },
        );
        let Some(copied_wobble_tiles) = scene.commands.iter().find_map(|command| match command {
            DrawCommand::Sprite {
                x: 16,
                y: -6,
                tile_sheet,
                tiles,
                ..
            } if tile_sheet.tiles.len() == 0x200 => Some(tiles),
            _ => None,
        }) else {
            panic!("ASM sub_1007A shifted wobble should draw the copied prize-block art");
        };
        assert_eq!(
            copied_wobble_tiles.first().map(|tile| tile.tile.tile_index),
            Some(0x008),
            "ASM sub_1007A shifted wobble should preserve the copied `$E102` prize skin"
        );
        assert_eq!(grid.cell_at(1, 0), priority_cell);
    }

    #[test]
    fn build_scene_draws_live_prize_block_wobble_from_block_skin_not_terrain() {
        // ASM sub_1007A starts by drawing the copied prize-block gfx object
        // while the source cell can still be `$E102`. The renderer must use
        // that live block cell skin, not the immutable foreground layout skin,
        // or forest/tree terrain briefly appears while the prize block shakes.
        let mut trace = sample_trace(GameMode::InGame);
        trace.camera_x_pos = 0;
        trace.camera_y_pos = 0;
        trace.block_bounces = vec![(1, 0, 31, 0)];
        trace.block_bounce_details = vec![BlockBounceTrace {
            cell_x: 1,
            cell_y: 0,
            x_offset_px: 0,
            remaining: 31,
            y_offset_px: 0,
            kind: BlockBounceTraceKind::Prize,
        }];
        let visuals = sample_level_visuals();
        let mut data = vec![0u8; 20 * 14 * 2];
        data[2] = 0xE1;
        data[3] = 0x02;
        let grid = BlockGrid::from_level_layout(&data, 20, 14);

        let scene = build_scene(&trace, false, Some(&visuals), Some(&grid));

        let Some(copied_wobble_tiles) = scene.commands.iter().find_map(|command| match command {
            DrawCommand::Sprite {
                x: 16,
                y: 0,
                tile_sheet,
                tiles,
                ..
            } if tile_sheet.tiles.len() == 0x200 => Some(tiles),
            _ => None,
        }) else {
            panic!("live sub_1007A prize cell should draw copied prize-block art");
        };
        assert_eq!(
            copied_wobble_tiles.first().map(|tile| tile.tile.tile_index),
            Some(0x008),
            "ASM sub_1007A should copy the `$E102` prize block skin byte unchanged, not decrement it to the hidden `$E101` art"
        );
        assert!(
            indexed_tile_at_with_sheet_len(&scene, 16, 0, 0x80).is_none(),
            "ASM sub_302F6/sub_3034A still render the live `$E102` Level_Layout cell from Block_Mappings; the immutable foreground only reappears after sub_1007A rewrites the cell to `$6Fxx`"
        );
    }

    #[test]
    fn prize_block_wobble_reuses_base_copy_when_second_slot_is_neighbour_art_sub_1007a() {
        // ASM sub_1007A/stru_101B6 alternates sprite mappings at VRAM
        // `$220/$229`, but those are live copied slots. Native generated block
        // sheets are static; a visible `+9` tile range can be neighbouring
        // block art, not the second copied VRAM slot.
        let mut trace = sample_trace(GameMode::InGame);
        trace.camera_x_pos = 0;
        trace.camera_y_pos = 0;
        trace.block_bounces = vec![(1, 0, 31, 0)];
        trace.block_bounce_details = vec![BlockBounceTrace {
            cell_x: 1,
            cell_y: 0,
            x_offset_px: 0,
            remaining: 31,
            y_offset_px: 0,
            kind: BlockBounceTraceKind::Prize,
        }];
        let mut visuals = sample_level_visuals();
        for tile in &mut visuals.block_tiles.tiles[0x008..=0x00B] {
            tile.pixels = [3; 64];
        }
        for tile in &mut visuals.block_tiles.tiles[0x011..=0x014] {
            tile.pixels = [7; 64];
        }
        let mut data = vec![0u8; 20 * 14 * 2];
        data[2] = 0x6F;
        data[3] = 0x00;
        let grid = BlockGrid::from_level_layout(&data, 20, 14);

        let scene = build_scene(&trace, false, Some(&visuals), Some(&grid));

        let Some(copied_wobble_tiles) = scene.commands.iter().find_map(|command| match command {
            DrawCommand::Sprite {
                x: 16,
                y: 0,
                tile_sheet,
                tiles,
                ..
            } if tile_sheet.tiles.len() == 0x200 => Some(tiles),
            _ => None,
        }) else {
            panic!("sub_1007A prize wobble should draw copied block art");
        };
        assert_eq!(
            copied_wobble_tiles.first().map(|tile| tile.tile.tile_index),
            Some(0x008),
            "native must not treat visible neighbouring `+9` block-sheet tiles as ASM `$229` copied VRAM art"
        );
        assert!(
            !copied_wobble_tiles
                .iter()
                .any(|tile| (0x011..=0x014).contains(&tile.tile.tile_index)),
            "ASM `$229` is a live copied VRAM slot; mismatched generated `+9` tiles are unrelated block art"
        );
    }

    #[test]
    fn live_prize_block_scenario_renders_sub_1007a_wobble_sprite_not_terrain() {
        let Some(manifest) = cached_generated_manifest() else {
            return;
        };
        let scenario = prize_block_scenario(0x00);
        let mut runtime = NativeRuntime::with_generated_level_layout(manifest.clone());
        runtime.enter_level_playtest(LevelId(0x00));
        let (visuals, fallback_grid) =
            load_session_assets(&manifest, 0x00).expect("Level 0x00 visuals");

        let mut previous_trace = runtime.trace_frame();
        for frame_index in 0..=scenario.frame_count {
            let buttons = scenario.script.get(frame_index).copied().unwrap_or(0);
            let trace = runtime.step_frame(FrameInput::from_mask(buttons));
            let render_trace = render_trace_with_vdp_latched_state(&trace, Some(&previous_trace));
            let Some(bounce) = trace
                .block_bounce_details
                .iter()
                .find(|bounce| bounce.kind == BlockBounceTraceKind::Prize && bounce.remaining > 0)
            else {
                previous_trace = trace;
                continue;
            };
            let Some(&(col, row, _, y_offset)) =
                render_trace
                    .block_bounces
                    .iter()
                    .find(|(col, row, remaining, _)| {
                        *col == bounce.cell_x && *row == bounce.cell_y && *remaining > 0
                    })
            else {
                previous_trace = trace;
                continue;
            };
            let camera_x_px = (render_trace.camera_x_pos >> 16) as i32;
            let camera_y_px = (render_trace.camera_y_pos >> 16) as i32;
            let draw_x = i32::from(col) * 16 - camera_x_px;
            let draw_y = i32::from(row) * 16 - camera_y_px + i32::from(y_offset);
            let block_grid = runtime.block_grid().unwrap_or(&fallback_grid);
            let raw_word = block_grid.raw_word_at(col, row);
            let cell = block_grid.cell_at(col, row);
            let expected_skin = if raw_word & 0xFF00 == 0x6F00 {
                0x02
            } else if cell.has_block {
                cell.skin
            } else {
                (raw_word & 0x00FF) as u8
            };
            let expected_resolves = visuals
                .level
                .block_mapping
                .resolve(expected_skin, Some(BlockType::Prize as u8))
                .is_some();
            let scene = build_scene(&render_trace, false, Some(&visuals), Some(block_grid));

            assert!(
                scene.commands.iter().any(|command| matches!(
                    command,
                    DrawCommand::Sprite {
                        x,
                        y,
                        tile_sheet,
                        ..
                    } if *x == draw_x
                        && *y == draw_y
                        && tile_sheet.tiles.len() == visuals.block_tiles.tiles.len()
                )),
                "ASM sub_1007A should draw the live prize-block wobble sprite at cell ({col},{row}) on frame {frame_index}, not terrain or a stale block fallback; expected ({draw_x},{draw_y}); raw={raw_word:#06X} cell={cell:?} copied_source_skin={expected_skin:#04X} resolves={expected_resolves}"
            );
            assert!(
                indexed_tile_at_with_sheet_len(&scene, draw_x, draw_y - i32::from(y_offset), 0x80)
                    .is_none(),
                "ASM sub_1007A owns the active prize wobble cell; immutable foreground terrain must not leak through while the block is bouncing"
            );
            return;
        }

        panic!(
            "prize_block_scenario should exercise an ASM sub_1007A prize-block wobble in the render-latched trace"
        );
    }

    #[test]
    fn cold_boot_demo_prize_wobble_draws_at_live_camera_latch_sub_1007a() {
        let Some(manifest) = cached_generated_manifest() else {
            return;
        };
        let trace = wildside_core::run_script_with_warp(manifest.clone(), &[0], 3_445, None);
        let current = &trace[3_444];
        let previous = &trace[3_443];
        assert_eq!(current.current_level, LevelId(0x1B));
        assert_eq!(current.block_bounces, vec![(4, 36, 9, 0)]);
        let render_trace = render_trace_with_vdp_latched_state(current, Some(previous));
        let (visuals, grid) =
            load_session_assets(&manifest, current.current_level.0).expect("Level 0x1B visuals");
        let scene = build_scene(&render_trace, false, Some(&visuals), Some(&grid));

        assert!(
            scene
                .commands
                .iter()
                .any(|command| matches!(command, DrawCommand::Sprite { x: 64, y: 87, .. })),
            "ASM sub_1007A prize wobble at level cell (4,36) should draw over the prior Camera_Y_pos latch, not vanish while the `$6Fxx` cell is live"
        );
        let sample_pixels = (64usize..80)
            .flat_map(|x| (87usize..103).map(move |y| (x, y)))
            .collect();
        let visible_wobble_pixels = sample_scene_pixel_provenance(&scene, &sample_pixels)
            .into_iter()
            .filter(|sample| {
                sample
                    .writes
                    .iter()
                    .any(|write| write.accepted && write.kind == "sprite_tile")
            })
            .count();
        assert!(
            visible_wobble_pixels > 0,
            "the visible sub_1007A prize wobble sprite should write non-background pixels"
        );
    }

    #[test]
    fn build_scene_draws_shifting_block_rise_worker_loc_113f4() {
        let mut trace = sample_trace(GameMode::InGame);
        trace.camera_x_pos = 0;
        trace.camera_y_pos = 0;
        trace.block_bounces = vec![(1, 0, 7, 14)];
        trace.block_bounce_details = vec![BlockBounceTrace {
            cell_x: 1,
            cell_y: 0,
            x_offset_px: 0,
            remaining: 7,
            y_offset_px: 14,
            kind: BlockBounceTraceKind::Shifting,
        }];
        let data = vec![0u8; 20 * 14 * 2];
        let mut grid = BlockGrid::from_level_layout(&data, 20, 14);
        grid.set_cell_at(1, 0, Cell::from_be_bytes(0x6F, 0x00));
        grid.set_cell_at(1, 1, Cell::from_be_bytes(0xE7, 0x00));
        let visuals = sample_level_visuals();

        let scene = build_scene(&trace, false, Some(&visuals), Some(&grid));

        let Some(shifting_tiles) = scene.commands.iter().find_map(|command| match command {
            DrawCommand::Sprite {
                x: 16,
                y: 14,
                tile_sheet,
                tiles,
                ..
            } if tile_sheet.tiles.len() == 0x200 => Some(tiles),
            _ => None,
        }) else {
            panic!(
                "ASM sub_113C0/loc_113F4 should render `LnkTo_unk_E0EBE` at the bobbed y-position while the reserved cell is `$6F00`"
            );
        };
        assert_eq!(
            shifting_tiles.first().map(|tile| tile.tile.tile_index),
            Some(0x0276 - BLOCK_TILE_VRAM_BASE),
            "ASM loc_113F4 uses fixed `LnkTo_unk_E0EBE` block art at VRAM `$276`, not the temporary `$6F00` cell skin or the prize wobble `$220/$229` slots"
        );
        assert!(
            indexed_tile_at_with_sheet_len(&scene, 16, 14, 0x200).is_none(),
            "the shifting rise worker should render as a sprite object, not as a static block-grid tile snapped to the destination cell"
        );
    }

    #[test]
    fn build_scene_draws_hidden_reveal_worker_over_solid_placeholder_loc_e096() {
        // ASM sub_DFB0 loc_E096 writes `$60` into Level_Layout, then
        // loc_E2E2/loc_E2F0 bob the reveal object for eight frames before
        // sub_11530 commits the final block word. The renderer must therefore
        // draw a typed HiddenReveal worker even when the live cell has no
        // block flag.
        let mut trace = sample_trace(GameMode::InGame);
        trace.camera_x_pos = 0;
        trace.camera_y_pos = 0;
        trace.block_bounces = vec![(1, 0, 8, -2)];
        trace.block_bounce_details = vec![
            BlockBounceTrace {
                cell_x: 1,
                cell_y: 0,
                x_offset_px: 0,
                remaining: 8,
                y_offset_px: -2,
                kind: BlockBounceTraceKind::HiddenReveal,
            },
            BlockBounceTrace {
                cell_x: 1,
                cell_y: 0,
                x_offset_px: 8,
                remaining: 8,
                y_offset_px: 6,
                kind: BlockBounceTraceKind::HiddenReveal,
            },
        ];
        let data = vec![0u8; 20 * 14 * 2];
        let mut grid = BlockGrid::from_level_layout(&data, 20, 14);
        grid.set_cell_at(1, 0, Cell::from_be_bytes(0x60, 0x0B));

        let scene = build_ingame_scene(&trace, false, Some(&grid));

        assert!(
            indexed_tile_at_with_sheet_len(&scene, 16, -2, 0x200).is_some(),
            "ASM loc_E096/loc_E2E2 should render the hidden-block reveal object at the bobbed y-position while the cell is the `$60xx` placeholder"
        );
        assert!(
            indexed_tile_at_with_sheet_len(&scene, 24, 6, 0x200).is_some(),
            "ASM loc_E0C2 should render split hidden-block reveal workers with their quarter-tile x/y offsets"
        );
        assert!(
            indexed_tile_at_with_sheet_len(&scene, 16, 0, 0x200).is_none(),
            "ASM loc_E2E2 moves the reveal object before hibernating; native must not leave the block art snapped to the static cell origin"
        );
    }

    #[test]
    fn erased_forest_block_renders_restored_loc_11542_skin() {
        let mut trace = sample_trace(GameMode::InGame);
        trace.camera_x_pos = 0;
        trace.camera_y_pos = 0;
        let data = vec![0u8; 20 * 14 * 2];
        let mut grid = BlockGrid::from_level_layout(&data, 20, 14);
        grid.set_cell_at(1, 0, Cell::from_be_bytes(0x60, 0x05));

        let scene = build_ingame_scene(&trace, false, Some(&grid));

        let Some((_, tile, _)) = topmost_indexed_tile_at_with_sheet_len(&scene, 16, 0, 0x80) else {
            panic!("expected restored forest terrain after loc_11542 block erase");
        };
        assert_eq!(
            tile.tile_index, 20,
            "ASM EraseBlockFromLevelLayout loc_11542 keeps the restored forest skin byte, so rendering must not fall back to the original layout skin and leave a visible gap"
        );
        assert_ne!(
            tile.tile_index, 12,
            "the original sample layout skin at this cell would be tile 12; erased block rendering must use the restored live skin instead"
        );
    }

    #[test]
    fn erased_forest_block_renders_restored_empty_collision_skin_loc_11542() {
        let mut trace = sample_trace(GameMode::InGame);
        trace.camera_x_pos = 0;
        trace.camera_y_pos = 0;
        let data = vec![0u8; 20 * 14 * 2];
        let mut grid = BlockGrid::from_level_layout(&data, 20, 14);
        grid.set_cell_at(1, 0, Cell::from_be_bytes(0x00, 0x05));

        let scene = build_ingame_scene(&trace, false, Some(&grid));

        let Some((_, tile, _)) = topmost_indexed_tile_at_with_sheet_len(&scene, 16, 0, 0x80) else {
            panic!("expected restored forest leaf skin even when loc_11542 leaves collision empty");
        };
        assert_eq!(
            tile.tile_index, 20,
            "ASM EraseBlockFromLevelLayout loc_11542 preserves the skin byte independently of collision, so empty-collision forest leaves must not fall back to the original layout skin"
        );
    }

    #[test]
    fn build_scene_renders_level_transition_title_workers_at_their_fly_in_positions() {
        // ASM Load_TitleCard `loc_1AB26` / `loc_1AC70` (kid.asm:28750-28986)
        // builds Title_Letter workers that glide from a random off-screen
        // entry pose toward the title-card destination. The engine projects
        // each worker into `active_objects` with `Enemy{object_type:0}` and a
        // `glyph` debug field; the LevelTransition renderer must draw each
        // visible glyph at the worker's interpolated screen position rather
        // than the old static "Blue Lake Woods 1" placeholder text.
        let mut trace = sample_trace(GameMode::LevelTransition);
        let make_worker = |x_pos: u32, y_pos: u32, slot: usize, glyph: u8| NativeObjectSnapshot {
            kind: NativeObjectKind::Enemy { object_type: 0 },
            x_pos,
            y_pos,
            layout_index: None,
            x_velocity: None,
            y_velocity: None,
            facing: None,
            on_ground: None,
            hit_points: None,
            damage_flash: false,
            palette_variant: 0,
            stationary: true,
            contact_phase: None,
            shared_walker_state: None,
            shared_walker_patrol_timer: None,
            shared_walker_patrol_short_next: None,
            walk_phase: None,
            wait_timer: None,
            debug_fields: std::collections::BTreeMap::from([
                (String::from("gfx_slot_index"), slot.to_string()),
                (String::from("glyph"), format!("{glyph:#04x}")),
            ]),
        };
        // Mid-flight 'h' worker still off-screen left at screen x = -16,
        // settled 'i' worker at the target (200, 80).
        trace.active_objects = vec![
            make_worker(0xFFF0_0000, 0x0050_0000, 0, b'h'),
            make_worker(0x00C8_0000, 0x0050_0000, 1, b'i'),
        ];

        let visuals = sample_level_visuals();
        let scene = build_scene(&trace, false, Some(&visuals), None);

        let has_glyph_at = |needle: &str, x: i32, y: i32| {
            scene.commands.iter().any(|command| {
                matches!(
                    command,
                    DrawCommand::Text { x: cx, y: cy, text, .. }
                        if *cx == x && *cy == y && text == needle
                )
            })
        };
        assert!(
            has_glyph_at("H", -16, 80),
            "off-screen-left 'h' worker should still be drawn at its interpolated negative screen X",
        );
        assert!(
            has_glyph_at("I", 200, 80),
            "settled 'i' worker should render at the script-side target (200, 80)",
        );
        assert!(
            !contains_text(&scene, "Blue Lake Woods 1"),
            "static placeholder level name must not appear once title workers are visible",
        );
    }

    #[test]
    fn build_scene_keeps_plain_level_transition_blank_without_title_workers() {
        // Death/respawn transitions reach LevelTransition without seeding any
        // Title_Letter workers (see wildside_core::trace `PlayerLifecycle::Dying`
        // and `Respawning` branches mapping to GameMode::LevelTransition).
        // The ROM does not draw the native placeholder "Blue Lake Woods" card
        // there; only `Load_TitleCard`/`loc_1AB26` owns title-card visuals.
        let trace = sample_trace(GameMode::LevelTransition);
        assert!(trace.active_objects.is_empty());
        let visuals = sample_level_visuals();
        let scene = build_scene(&trace, false, Some(&visuals), None);

        assert!(
            !contains_text(&scene, "Blue Lake Woods 1"),
            "plain LevelTransition windows should not invent a static level loader",
        );
    }

    #[test]
    fn build_scene_keeps_title_card_bootstrap_blank_until_title_workers_surface() {
        // ASM `loc_6E2` dispatches title selection through `Load_TitleCard`,
        // then `loc_1AB26` seeds the animated Title_Letter workers. Native can
        // trace the early shell as LevelTransition before those workers are
        // projected; do not render the old invented static "Blue Lake Woods 1"
        // card for that ROM title-card bootstrap window.
        let mut trace = sample_trace(GameMode::LevelTransition);
        trace.boot_phase = Some(String::from("starting_level"));
        assert!(trace.active_objects.is_empty());
        let visuals = sample_level_visuals();

        let scene = build_scene(&trace, false, Some(&visuals), None);

        assert!(
            !contains_text(&scene, "Blue Lake Woods 1"),
            "Load_TitleCard bootstrap should not use the static transition fallback before loc_1AB26 workers are visible",
        );
    }

    #[test]
    fn build_scene_keeps_level_skip_hold_from_using_static_card() {
        // ASM loc_74E0 level-skip cheat enters the hold/resume animation path,
        // not Load_TitleCard. When no visible objects are projected in the
        // blank preload window, keep the ROM-style blank transition rather than
        // restoring the old fake "Blue Lake Woods" card.
        let mut trace = sample_trace(GameMode::LevelTransition);
        trace.boot_phase = Some(String::from("level_skip_hold"));
        assert!(trace.active_objects.is_empty());
        let visuals = sample_level_visuals();

        let scene = build_scene(&trace, false, Some(&visuals), None);

        assert!(
            !contains_text(&scene, "Blue Lake Woods 1"),
            "level-skip hold should not use the static transition fallback",
        );
    }

    #[test]
    fn vdp_title_card_bootstrap_composites_title_letter_workers() {
        // ASM Load_TitleCard draws the background through VDP, while
        // loc_1AB26/loc_1AC70 keep the animated Title_Letter workers in gfx
        // object slots. Native's VDP path must overlay those workers rather
        // than replacing the whole frame with the static shell.
        let mut trace = sample_trace(GameMode::LevelTransition);
        trace.boot_phase = Some(String::from("starting_level"));
        trace.active_objects = vec![NativeObjectSnapshot {
            kind: NativeObjectKind::Enemy { object_type: 0 },
            x_pos: 0x0010_0000,
            y_pos: 0x0010_0000,
            layout_index: None,
            x_velocity: None,
            y_velocity: None,
            facing: None,
            on_ground: None,
            hit_points: None,
            damage_flash: false,
            palette_variant: 0,
            stationary: true,
            contact_phase: None,
            shared_walker_state: None,
            shared_walker_patrol_timer: None,
            shared_walker_patrol_short_next: None,
            walk_phase: None,
            wait_timer: None,
            debug_fields: std::collections::BTreeMap::from([(
                String::from("glyph"),
                String::from("0x68"),
            )]),
        }];
        let visuals = sample_level_visuals();
        let mut frame = vec![12; (FRAME_WIDTH * FRAME_HEIGHT * 3) as usize];

        render_vdp_trace_overlay(&trace, &visuals, &mut frame);

        assert!(
            frame.chunks_exact(3).any(|pixel| pixel != [12, 12, 12]),
            "VDP title-card shell should retain the animated title-letter overlay",
        );
    }

    #[test]
    fn vdp_intro_sequence_4_does_not_overlay_player_sprite_loc_1b988() {
        // ASM Load_IntroSequence4 (`sub_1B988`, kid.asm:30273) sets
        // `Camera_Y_pos=4` and allocates a single object running `loc_1C7A0`
        // with `IntroText4` — the holographic background plus story-text
        // writer. It allocates NO player/Kid GfxObjectSlot. The carried
        // `kid_gfx` pose in the trace (kid_x/kid_y ~ $60/$D0) is stale leftover
        // state from `Load_IntroSequence3`'s `unk_1AF24`/`loc_1AEEC` montage,
        // not an IntroSequence4 object. Compositing it drew a spurious normal
        // Kid sprite at the bottom-left of the story screen (#210). The overlay
        // must leave a non-black IntroSequence4 shell untouched.
        let mut trace = sample_trace(GameMode::IntroSequence4);
        trace.kid_x_pos = Some(0x0060_0000);
        trace.kid_y_pos = Some(0x00D0_0000);
        trace.kid_facing = Some(Direction::Left);
        trace.kid_on_ground = Some(true);
        let visuals = sample_level_visuals();
        let mut frame = vec![40; (FRAME_WIDTH * FRAME_HEIGHT * 3) as usize];

        render_vdp_trace_overlay(&trace, &visuals, &mut frame);

        assert!(
            frame.chunks_exact(3).all(|pixel| pixel == [40, 40, 40]),
            "IntroSequence4 story shell must stay free of any carried Kid pose overlay",
        );
    }

    #[test]
    fn vdp_title_card_overlay_keeps_blue_lake_letter_pixels_on_rom_palette_slots() {
        // ASM loc_1AB40 copies `Pal_19C48` as a complete 16-word palette line
        // into CRAM line 1. Native stores that standalone line at palette line
        // 0 for rendering, so a glyph pixel with value 2 must stay on ROM slot
        // 2; shifting visible pixels down makes the Blue Lake Woods title text
        // read as black against the title-card background.
        let mut trace = sample_trace(GameMode::LevelTransition);
        trace.boot_phase = Some(String::from("starting_level"));
        trace.active_objects = vec![NativeObjectSnapshot {
            kind: NativeObjectKind::Enemy { object_type: 0 },
            x_pos: 0x0010_0000,
            y_pos: 0x0010_0000,
            layout_index: None,
            x_velocity: None,
            y_velocity: None,
            facing: None,
            on_ground: None,
            hit_points: None,
            damage_flash: false,
            palette_variant: 0,
            stationary: true,
            contact_phase: None,
            shared_walker_state: None,
            shared_walker_patrol_timer: None,
            shared_walker_patrol_short_next: None,
            walk_phase: None,
            wait_timer: None,
            debug_fields: std::collections::BTreeMap::from([(
                String::from("glyph"),
                String::from("0x68"),
            )]),
        }];
        let mut visuals = sample_level_visuals();
        let mut tile_sheet = blank_tile_sheet(0x20);
        tile_sheet.tiles[0x1C] = make_tile(&[(0, 0, 2)]);
        visuals.title_card_letters = Some(wildside_render::TitleCardLettersVisuals {
            tile_sheet,
            palette: Palette {
                colours: vec![
                    colour(20, 20, 60),
                    colour(0, 0, 0),
                    colour(64, 96, 192),
                    colour(80, 120, 220),
                    colour(0, 0, 0),
                    colour(0, 0, 0),
                    colour(0, 0, 0),
                    colour(0, 0, 0),
                    colour(0, 0, 0),
                    colour(0, 0, 0),
                    colour(0, 0, 0),
                    colour(0, 0, 0),
                    colour(0, 0, 0),
                    colour(0, 0, 0),
                    colour(0, 0, 0),
                    colour(0, 0, 0),
                ],
            },
        });
        let mut frame = vec![12; (FRAME_WIDTH * FRAME_HEIGHT * 3) as usize];

        render_vdp_trace_overlay(&trace, &visuals, &mut frame);

        assert_eq!(pixel_at(&frame, 16, 16), [64, 96, 192]);
    }

    #[test]
    fn build_scene_clamps_floating_item_alpha_before_colour_scaling() {
        let mut trace = sample_trace(GameMode::InGame);
        trace.floating_items = vec![(48, 64, u16::from(u8::MAX), 0x01)];
        let visuals = sample_level_visuals();

        let scene = build_scene(&trace, false, Some(&visuals), None);

        assert!(scene.commands.iter().any(|command| {
            matches!(
                command,
                DrawCommand::Rect { colour, .. } if *colour == [255, 255, 100]
            )
        }));
    }

    #[test]
    fn build_scene_draws_10000_points_worker_star_from_hud_rom_tiles() {
        let mut trace = sample_trace(GameMode::InGame);
        // ASM Prize10000Points_Init loc_10B00 spawns the first worker when
        // d3=0; generic prize lifetime low byte is 0x20 on that first visible
        // frame, so the renderer can recover the local effect age from it.
        trace.floating_items = vec![(48, 64, 0x20, 0x01)];
        let visuals = sample_level_visuals();

        let scene = build_scene(&trace, false, Some(&visuals), None);

        assert!(
            scene.commands.iter().any(|command| {
                matches!(
                    command,
                    DrawCommand::Sprite { tile_sheet, tiles, .. }
                        if tile_sheet.tiles.len() == visuals.hud.digits.tiles.len()
                            && tiles.iter().any(|tile| tile.tile.tile_index == 1)
                )
            }),
            "ASM loc_10B00/loc_10B3A should draw the d3=0 10000-point worker with unk_E1056 over HUD number tile $6BB"
        );
        assert!(
            !scene.commands.iter().any(|command| {
                matches!(
                    command,
                    DrawCommand::Rect { colour, .. } if *colour == [255, 255, 100]
                )
            }),
            "10000-point prize should not fall back to the placeholder square while its ROM workers are visible"
        );
    }

    #[test]
    fn build_scene_draws_ten_diamonds_workers_from_hud_diamond_rom_tiles() {
        let mut trace = sample_trace(GameMode::InGame);
        // ASM Prize10Diamonds_Init loc_108EE uses byte_108AC to spawn the
        // first four diamond workers on local frames 0..3. Remaining low byte
        // 0x1D corresponds to local age 3 from the generic 0x0320 lifetime.
        trace.floating_items = vec![(48, 64, 0x1D, 0x0E)];
        let visuals = sample_level_visuals();

        let scene = build_scene(&trace, false, Some(&visuals), None);

        let diamond_worker_count = scene
            .commands
            .iter()
            .filter(|command| {
                matches!(
                    command,
                    DrawCommand::Sprite { tile_sheet, tiles, .. }
                        if tile_sheet.tiles.len() == visuals.hud.diamond_tiles.tiles.len()
                            && tiles.iter().any(|tile| tile.tile.tile_index == 3)
                )
            })
            .count();
        assert_eq!(
            diamond_worker_count, 4,
            "ASM byte_108AC frames 0..3 should draw four ten-diamond workers using unk_E1046 over HUD diamond tile $6ED"
        );
        assert!(
            !scene.commands.iter().any(|command| {
                matches!(
                    command,
                    DrawCommand::Rect { colour, .. } if *colour == [100, 200, 255]
                )
            }),
            "10-diamond prize should not fall back to the placeholder square while its ROM workers are visible"
        );
    }

    #[test]
    fn build_scene_draws_circle_of_doom_sprites_from_hud_diamond_rom_tiles() {
        let mut trace = sample_trace(GameMode::InGame);
        // ASM DiamondPower_Main_CircleOfDoom loc_3F880 emits `word_4020E`
        // sprite attrs. `$86F2` is a 16x16 HUD diamond tile with priority set.
        trace.diamond_power_sprites = vec![DiamondPowerSpriteSnapshot {
            screen_x: 0x00A0,
            screen_y: 0x0090,
            size_word: 0x0500,
            pattern_word: 0x86F2,
        }];
        let visuals = sample_level_visuals();

        let scene = build_scene(&trace, false, Some(&visuals), None);

        assert!(
            scene.commands.iter().any(|command| {
                matches!(
                    command,
                    DrawCommand::Sprite { tile_sheet, .. }
                        if tile_sheet.tiles.len() == visuals.hud.diamond_tiles.tiles.len()
                            && tile_sheet.tiles.iter().any(|tile| tile.pixels == [17; 64])
                )
            }),
            "ASM loc_3F880/word_4020E should render Circle of Doom from HUD diamond VRAM art"
        );
    }

    #[test]
    fn build_scene_draws_floating_diamond_with_rom_sprite_art() {
        let mut trace = sample_trace(GameMode::InGame);
        trace.floating_items = vec![(48, 64, 0x01F4, 0x00)];
        let visuals = sample_level_visuals();

        let scene = build_scene(&trace, false, Some(&visuals), None);

        assert!(
            scene.commands.iter().any(|command| {
                matches!(
                    command,
                    DrawCommand::Sprite { tile_sheet, .. }
                        if tile_sheet.tiles.len() == visuals.hud.diamond_tiles.tiles.len()
                            && tile_sheet.tiles.iter().any(|tile| tile.pixels == [16; 64])
                )
            }),
            "ASM stru_10D6E/unk_E0E66 should render the floating prize from HUD diamond VRAM art"
        );
        assert!(
            !scene.commands.iter().any(|command| {
                matches!(
                    command,
                    DrawCommand::Rect { colour, .. } if *colour == [100, 200, 255]
                )
            }),
            "floating diamond should not fall back to the old placeholder rects"
        );
    }

    #[test]
    fn build_scene_keeps_floating_diamond_visible_before_loc_103b2_blink_threshold() {
        let mut trace = sample_trace(GameMode::InGame);
        trace.camera_y_pos = 1500 << 16;
        // ASM PrizeDiamond_Init loc_10396 decrements the full d3 word from
        // $01F4. A low byte of $82 is still visible when the real countdown is
        // $0182; native must not treat it as the late $0082 blink phase.
        trace.floating_items = vec![(120, 1665, 0x0182, 0x00)];
        let visuals = sample_level_visuals();

        let scene = build_scene(&trace, false, Some(&visuals), None);

        assert!(
            scene.commands.iter().any(|command| {
                matches!(
                    command,
                    DrawCommand::Sprite { tile_sheet, .. }
                        if tile_sheet.tiles.len() == visuals.hud.diamond_tiles.tiles.len()
                            && tile_sheet.tiles.iter().any(|tile| tile.pixels == [18; 64])
                )
            }),
            "ASM loc_103B2 should not switch to the blinking `stru_10D80` until the full d3 word reaches $00C8"
        );
    }

    #[test]
    fn build_scene_animates_floating_diamond_from_local_loc_10396_countdown() {
        let mut trace = sample_trace(GameMode::InGame);
        trace.frame_index = 150;
        // ASM PrizeDiamond_Init calls j_Init_Animation on `stru_10D6E` when
        // the object is created; global Time_Frames does not choose the
        // diamond frame. At remaining d3=$01F4 the local animation is still on
        // the first `unk_E0E66` frame even though frame_index/3 would select
        // fixture frame 2.
        trace.floating_items = vec![(48, 64, 0x01F4, 0x00)];
        let visuals = sample_level_visuals();

        let scene = build_scene(&trace, false, Some(&visuals), None);

        assert!(
            scene.commands.iter().any(|command| {
                matches!(
                    command,
                    DrawCommand::Sprite { tile_sheet, .. }
                        if tile_sheet.tiles.iter().any(|tile| tile.pixels == [16; 64])
                )
            }),
            "ASM loc_10396 should drive `stru_10D6E` from the prize object's local countdown"
        );
    }

    #[test]
    fn build_scene_hides_floating_diamond_on_asm_blink_null_frames() {
        let mut trace = sample_trace(GameMode::InGame);
        trace.frame_index = 510;
        trace.camera_y_pos = 1500 << 16;
        // Oracle WRAM for issue-215 Highwater frame 510 has the same diamond
        // object alive at x=$78/y=$681, but `addroffset_sprite(a3)=0`.
        // ASM PrizeDiamond_Init loc_103B2 has already switched to `stru_10D80`;
        // at remaining d3=$0082 that script is on a null frame, so
        // Make_SpriteFromGfxObject returns without emitting a sprite.
        trace.floating_items = vec![(120, 1665, 0x82, 0x00)];
        let visuals = sample_level_visuals();

        let scene = build_scene(&trace, false, Some(&visuals), None);

        assert!(
            !scene.commands.iter().any(|command| {
                matches!(
                    command,
                    DrawCommand::Sprite { x, y, tile_sheet, tiles, .. }
                        if *x == 114
                            && *y == 151
                            && tile_sheet.tiles.len() == visuals.hud.diamond_tiles.tiles.len()
                            && tiles.iter().any(|tile| tile.tile.tile_index == 8)
                )
            }),
            "ASM stru_10D80 null frames should hide the late-life floating diamond sprite"
        );
    }

    #[test]
    fn build_scene_draws_floating_helmets_with_rom_mask_art() {
        for (content_id, tile_pattern, asm_label) in [
            (0x02, 91, "PrizeHelmetSkycutter_Init/stru_106DC/unk_A96C4"),
            (0x03, 92, "PrizeHelmetCyclone_Init/stru_106E8/unk_BACEC"),
            (0x04, 93, "PrizeHelmetRedStealth_Init/stru_106F4/unk_B3FB8"),
            (0x05, 94, "PrizeHelmetEyeclops_Init/stru_106EE/unk_B7668"),
            (0x06, 95, "PrizeHelmetJuggernaut_Init/stru_106D6/unk_C1A5C"),
            (0x07, 96, "PrizeHelmetIronKnight_Init/stru_106FA/unk_C2324"),
            (0x08, 97, "PrizeHelmetBerzerker_Init/stru_106E2/unk_AFF56"),
            (0x09, 98, "PrizeHelmetManiaxe_Init/stru_106D0/unk_A5AB6"),
            (0x0A, 99, "PrizeHelmetMicromax_Init/stru_10700/unk_ABA68"),
        ] {
            let mut trace = sample_trace(GameMode::InGame);
            trace.floating_items = vec![(48, 64, u16::from(u8::MAX), content_id)];
            let visuals = sample_level_visuals();

            let scene = build_scene(&trace, false, Some(&visuals), None);

            assert!(
                scene.commands.iter().any(|command| {
                    matches!(
                        command,
                        DrawCommand::Sprite { tile_sheet, .. }
                            if tile_sheet.tiles.iter().any(|tile| tile.pixels == [tile_pattern; 64])
                    )
                }),
                "ASM {asm_label} should render floating helmet content id {content_id:#04X} from ROM mask art"
            );
            assert!(
                !scene.commands.iter().any(|command| {
                    matches!(
                        command,
                        DrawCommand::Rect { colour, .. } if *colour == [220, 220, 255]
                    )
                }),
                "floating helmet content id {content_id:#04X} should not fall back to placeholder rects"
            );
        }
    }

    #[test]
    fn build_scene_draws_floating_special_icons_with_rom_art() {
        for (content_id, tile_pattern, fallback_colour, asm_label) in [
            (
                0x0B,
                101,
                [255, 255, 200],
                "PrizeAnkh_Init/stru_10C08/unk_E0FE6",
            ),
            (
                0x0C,
                102,
                [200, 255, 200],
                "PrizeClock_Init/stru_10C18/unk_E0FCE",
            ),
            (
                0x0D,
                103,
                [255, 255, 255],
                "PrizeContinue_Init/stru_10C28/unk_E0FD6",
            ),
        ] {
            let mut trace = sample_trace(GameMode::InGame);
            trace.floating_items = vec![(48, 64, u16::from(u8::MAX), content_id)];
            let visuals = sample_level_visuals();

            let scene = build_scene(&trace, false, Some(&visuals), None);

            assert!(
                scene.commands.iter().any(|command| {
                    matches!(
                        command,
                        DrawCommand::Sprite { tile_sheet, .. }
                            if tile_sheet.tiles.iter().any(|tile| tile.pixels == [tile_pattern; 64])
                    )
                }),
                "ASM {asm_label} should render floating prize content id {content_id:#04X} from ROM icon art"
            );
            assert!(
                !scene.commands.iter().any(|command| {
                    matches!(
                        command,
                        DrawCommand::Rect { colour, .. } if *colour == fallback_colour
                    )
                }),
                "floating prize content id {content_id:#04X} should not fall back to placeholder rects"
            );
        }
    }

    #[test]
    fn floating_item_renderer_uses_asm_graphics_spec_for_sprite_backed_prizes() {
        // ASM PrizeDiamond_Init selects `stru_10D6E`; ASM sub_10CB0 gives
        // every helmet palette line 3 before PrizeHelmet* picks
        // `stru_106D0..stru_10700`; PrizeAnkh/Clock/Continue select
        // `stru_10C08`, `stru_10C18`, and `stru_10C28`.
        assert_eq!(
            floating_item_graphics_spec(0x00),
            Some(wildside_core::FloatingPrizeGraphicsSpec {
                object_meta: 0x0064,
                palette_line: 0,
                animation_script_addr: 0x0001_0D6E,
            })
        );
        assert_eq!(
            floating_item_graphics_spec(0x02),
            Some(wildside_core::FloatingPrizeGraphicsSpec {
                object_meta: 0x006C,
                palette_line: 3,
                animation_script_addr: 0x0001_06DC,
            })
        );
        assert_eq!(
            floating_item_graphics_spec(0x0B),
            Some(wildside_core::FloatingPrizeGraphicsSpec {
                object_meta: 0x0090,
                palette_line: 0,
                animation_script_addr: 0x0001_0C08,
            })
        );
        assert_eq!(
            floating_item_graphics_spec(0x0C),
            Some(wildside_core::FloatingPrizeGraphicsSpec {
                object_meta: 0x0094,
                palette_line: 0,
                animation_script_addr: 0x0001_0C18,
            })
        );
        assert_eq!(
            floating_item_graphics_spec(0x0D),
            Some(wildside_core::FloatingPrizeGraphicsSpec {
                object_meta: 0x0098,
                palette_line: 0,
                animation_script_addr: 0x0001_0C28,
            })
        );
    }

    #[test]
    fn build_game_scene_emits_background_tiles_instead_of_sky_rectangles() {
        let mut trace = sample_trace(GameMode::InGame);
        trace.camera_x_pos = 0;
        trace.camera_y_pos = 0;
        let visuals = sample_level_visuals();

        let scene = build_scene(&trace, false, Some(&visuals), None);

        let Some((tile_sheet, tile, palette)) = indexed_tile_at(&scene, 0, 0) else {
            panic!("expected a background indexed tile at the origin");
        };
        assert_eq!(tile_sheet.tiles.len(), 0x20);
        assert_eq!(palette.colours[0], colour(33, 66, 99));
        assert_eq!(tile.tile_index, 0);
        assert!(!has_guide_rect(&scene));
        assert!(!scene.commands.iter().any(|command| {
            matches!(
                command,
                DrawCommand::Rect {
                    x: 0,
                    y: 0,
                    width,
                    height,
                    ..
                } if *width == FRAME_WIDTH as i32 && *height == FRAME_HEIGHT as i32
            )
        }));
    }

    #[test]
    fn island_background_consumes_shore_vram_frames_loc_1202a() {
        let mut trace = sample_trace(GameMode::InGame);
        trace.camera_x_pos = 0;
        trace.camera_y_pos = 0;
        trace.island_shore_vram_frames_loc_1202a = Some((1, 2, 5));
        let mut visuals = sample_level_visuals();
        visuals.level.background_theme_id = 4;
        visuals.palettes.background = Palette {
            colours: (0..8)
                .map(|value| colour(value * 20, value * 20 + 1, value * 20 + 2))
                .collect(),
        };
        visuals.background_tiles = blank_tile_sheet(0x47);
        visuals.background_tiles.tiles[0x30].pixels = [7; 64];
        visuals.background_tiles.tiles[0x36].pixels = [7; 64];
        visuals.background_tiles.tiles[0x3B].pixels = [7; 64];
        visuals.background = BackgroundVisuals::Chunked(ChunkedBackgroundVisuals {
            placements: vec![BackgroundPlacement {
                chunk_id: 0,
                x_tiles: 0,
                y_tiles: 0,
            }],
            chunks: vec![BackgroundChunk {
                width_tiles: 3,
                height_tiles: 1,
                tiles: vec![
                    BackgroundTile {
                        tile_index: 0x30,
                        flip_x: false,
                    },
                    BackgroundTile {
                        tile_index: 0x36,
                        flip_x: false,
                    },
                    BackgroundTile {
                        tile_index: 0x3B,
                        flip_x: false,
                    },
                ],
            }],
        });

        let mut shore_frames = [
            blank_tile_sheet(23),
            blank_tile_sheet(23),
            blank_tile_sheet(23),
            blank_tile_sheet(23),
        ];
        shore_frames[1].tiles[0].pixels = [2; 64];
        shore_frames[2].tiles[6].pixels = [3; 64];
        shore_frames[1].tiles[11].pixels = [4; 64];
        visuals.island_shore_tiles_loc_1202a = Some(IslandShoreVisualsLoc1202A {
            frames: shore_frames,
        });

        let primary = super::scene::sample_background_pixel_loc_c070(&trace, &visuals, 0, 0)
            .expect("primary shore band pixel");
        let mid = super::scene::sample_background_pixel_loc_c070(&trace, &visuals, 8, 0)
            .expect("middle shore band pixel");
        let lower = super::scene::sample_background_pixel_loc_c070(&trace, &visuals, 16, 0)
            .expect("lower shore band pixel");

        assert_eq!(
            primary.rgb,
            [40, 41, 42],
            "ASM loc_12098 streams `off_1220E[1]+2` into VRAM `$F600`; Scroll B tile `$7B0` is relative background tile `$30`"
        );
        assert_eq!(
            mid.rgb,
            [60, 61, 62],
            "ASM loc_12110 streams `off_1220E[2]+$C2` into VRAM `$F6C0`; native must replace the middle `$A0`-byte shore band"
        );
        assert_eq!(
            lower.rgb,
            [80, 81, 82],
            "ASM loc_12188 uses the six-entry `off_1220E` mirror table, so phase 5 reuses shore frame 2 for the lower `$180`-byte band"
        );
    }

    #[test]
    fn background_pixel_sampler_reports_last_drawn_tile_provenance_loc_c070() {
        let mut trace = sample_trace(GameMode::InGame);
        trace.camera_x_pos = 0;
        trace.camera_y_pos = 0;
        let mut visuals = sample_level_visuals();
        visuals.level.background_theme_id = 9;
        visuals.background_tiles = TileSheet {
            tiles: vec![
                make_tile(&[(3, 4, 1)]),
                make_tile(&[(5, 4, 2)]),
                make_tile(&[(3, 4, 1)]),
            ],
        };
        visuals.palettes.background = Palette {
            colours: vec![colour(10, 20, 30), colour(40, 50, 60)],
        };
        visuals.background = BackgroundVisuals::Chunked(ChunkedBackgroundVisuals {
            placements: vec![
                BackgroundPlacement {
                    chunk_id: 0,
                    x_tiles: 0,
                    y_tiles: 0,
                },
                BackgroundPlacement {
                    chunk_id: 1,
                    x_tiles: 0,
                    y_tiles: 0,
                },
            ],
            chunks: vec![
                BackgroundChunk {
                    width_tiles: 1,
                    height_tiles: 1,
                    tiles: vec![BackgroundTile {
                        tile_index: 0,
                        flip_x: false,
                    }],
                },
                BackgroundChunk {
                    width_tiles: 1,
                    height_tiles: 1,
                    tiles: vec![BackgroundTile {
                        tile_index: 1,
                        flip_x: true,
                    }],
                },
            ],
        });

        let sample = super::scene::sample_background_pixel_loc_c070(&trace, &visuals, 2, 4)
            .expect("flipped second placement should draw an opaque pixel");

        assert_eq!(
            sample.tile_index, 1,
            "later Scroll B writes win just like renderer command order; this keeps loc_C070 pixel provenance aligned with visible output"
        );
        assert_eq!(sample.palette_index, 2);
        assert_eq!(sample.source_x, 5);
        assert_eq!(sample.source_y, 4);
        assert_eq!(sample.source_row_pixels, [0, 0, 0, 0, 0, 2, 0, 0]);
        assert_eq!(sample.rgb, [40, 50, 60]);
    }

    #[test]
    fn chunked_plane_b_overlay_samples_hscrolled_vdp_column_loc_3050a() {
        let mut trace = sample_trace(GameMode::InGame);
        trace.camera_x_pos = 292 << 16;
        trace.camera_y_pos = 224 << 16;

        let mut visuals = sample_level_visuals();
        visuals.level.background_theme_id = 8;
        let mut layer_speeds = vec![0x10; 8];
        layer_speeds[7] = 0x20;
        visuals.level.background_scroll = Some(wildside_data::BgscrollProfile {
            layer_speeds,
            has_terminator: true,
        });
        visuals.palettes.background = Palette {
            colours: (0..32)
                .map(|slot| colour(slot * 3, slot * 3 + 1, slot * 3 + 2))
                .collect(),
        };
        visuals.background_tiles = blank_tile_sheet(40);
        visuals.background_tiles.tiles[8].pixels = [2; 64];
        visuals.background_tiles.tiles[13].pixels = [3; 64];
        visuals.background = BackgroundVisuals::Chunked(ChunkedBackgroundVisuals {
            placements: vec![BackgroundPlacement {
                chunk_id: 0,
                x_tiles: 0,
                y_tiles: 0,
            }],
            chunks: vec![BackgroundChunk {
                width_tiles: 40,
                height_tiles: 8,
                tiles: (0..8)
                    .flat_map(|_| {
                        (0..40).map(|column| BackgroundTile {
                            tile_index: column as u16,
                            flip_x: false,
                        })
                    })
                    .collect(),
            }],
        });

        let diagnostic =
            super::scene::background_scroll_pixel_diagnostic_loc_3050a(&trace, &visuals, 61, 0)
                .expect("chunked Plane B diagnostic");
        assert_eq!(diagnostic.scroll_x_px_loc_3050a, 36);
        assert_eq!(diagnostic.destination_column_loc_30744, 4);
        assert_eq!(diagnostic.sample_column, 12);
        assert_eq!(diagnostic.source_column_start_loc_30744, 5);
        assert_eq!(diagnostic.fresh_window_offset_loc_30744, Some(8));
        assert_eq!(diagnostic.sample_source_column_loc_30744, Some(13));

        let sample = super::scene::sample_background_pixel_loc_c070(&trace, &visuals, 61, 0)
            .expect("hscrolled Plane B sample");
        assert_eq!(
            sample.tile_index, 13,
            "ASM loc_3050A applies H-scroll after sub_30744/sub_307D8 write Plane-B cells, so screen x=61 samples VDP column 12 and source column 13, not screen column 7"
        );
        assert_eq!(sample.background_row_index, Some(7));
        assert_eq!(sample.background_column, Some(13));
        assert_eq!(sample.source_x, 1);
    }

    #[test]
    fn chunked_plane_b_overlay_renders_cleared_chunk_zero_loc_12848_307d8() {
        let mut trace = sample_trace(GameMode::InGame);
        trace.camera_x_pos = 32 << 16;
        trace.camera_y_pos = 0;

        let mut visuals = sample_level_visuals();
        visuals.level.background_theme_id = 8;
        visuals.level.background_scroll = Some(wildside_data::BgscrollProfile {
            layer_speeds: vec![0x20; 16],
            has_terminator: true,
        });
        visuals.palettes.background = Palette {
            colours: (0..32)
                .map(|slot| colour(10 + slot, 40 + slot, 90 + slot))
                .collect(),
        };
        visuals.background_tiles = blank_tile_sheet(1);
        visuals.background_tiles.tiles[0].pixels = [3; 64];
        visuals.background = BackgroundVisuals::Chunked(ChunkedBackgroundVisuals {
            placements: Vec::new(),
            chunks: Vec::new(),
        });

        let sample = super::scene::sample_background_pixel_loc_c070(&trace, &visuals, 12, 0)
            .expect("cleared Plane-B row should render chunk zero");

        assert_eq!(
            sample.tile_index, 0,
            "ASM Load_BackgroundChunk loc_12848 leaves sparse `$FFFF87B2` bytes as chunk id 0, and sub_307D8 maps them through `unk_309B8` as `$2780` rather than transparent pixels"
        );
        assert_eq!(sample.palette_line, 1);
        assert_eq!(sample.rgb, [27, 57, 107]);
        assert_eq!(sample.background_row_index, Some(0));
    }

    #[test]
    fn chunked_source_buffer_applies_shared_layout_offsets_loc_12824_12848() {
        let mut visuals = sample_level_visuals();
        visuals.level.background_theme_id = 8;
        visuals.level.foreground_layout_bytes = vec![0; 80 * 14];
        visuals.level.foreground_layout =
            ForegroundLayout::parse(&visuals.level.foreground_layout_bytes, 4, 1)
                .expect("wide foreground layout");
        visuals.level.background_layout_alias = Some(BackgroundLayoutAlias {
            alias_label: "BackgroundLayout_0E".to_string(),
            source_label: "BackgroundLayout_0C".to_string(),
            source_background_path: Some("background/0C.bin".to_string()),
            x_offset_tiles: 0x13,
            y_offset_tiles: 0,
        });

        let background = ChunkedBackgroundVisuals {
            placements: vec![BackgroundPlacement {
                chunk_id: 0,
                x_tiles: 0x13 + 49,
                y_tiles: 18,
            }],
            chunks: vec![BackgroundChunk {
                width_tiles: 8,
                height_tiles: 1,
                tiles: (1..=8)
                    .map(|tile_index| BackgroundTile {
                        tile_index,
                        flip_x: false,
                    })
                    .collect(),
            }],
        };

        let direct =
            super::scene::ChunkedBackgroundSourceLoc12848::from_background_direct_loc_12830(
                &visuals,
                &background,
            )
            .expect("direct chunked source buffer");
        let source =
            super::scene::ChunkedBackgroundSourceLoc12848::from_background_alias_loc_12824(
                &visuals,
                &background,
            )
            .expect("alias-applied chunked source buffer");

        let row_tiles: Vec<u16> = (49..=56)
            .map(|column| {
                source
                    .tile_index_at(18, column)
                    .expect("source-buffer column should be in bounds")
            })
            .collect();
        assert_eq!(
            row_tiles,
            vec![1, 2, 3, 4, 5, 6, 7, 8],
            "ASM Load_BackgroundMappings loc_12824 subtracts the shared BackgroundLayout_0E x offset `$13` before Load_BackgroundChunk loc_12848 writes `$FFFF87B2`; this matches the issue #289 oracle row where columns 49..56 contain chunk bytes 01..08"
        );
        assert_eq!(
            source.tile_index_at(18, 68),
            Some(0),
            "the alias offset belongs to the loc_12848 source-buffer model, not the current sub_307D8 render sampling path"
        );
        assert_eq!(
            (direct.tile_index_at(18, 68), direct.tile_index_at(18, 69)),
            (Some(1), Some(2)),
            "the direct loc_12830 source keeps the placement's unadjusted columns, so the current renderer remains behaviourally unchanged until the persisted Plane-B history is modelled"
        );
    }

    #[test]
    fn chunked_timed_plane_b_window_uses_preload_time_loc_2ffe8_307d8() {
        let mut trace = sample_trace(GameMode::InGame);
        trace.frame_index = 60;
        trace.time_frames_loc_3038a = 174;
        trace.camera_x_pos = 0;
        trace.camera_y_pos = 0;

        let mut visuals = sample_level_visuals();
        visuals.level.background_theme_id = 8;
        visuals.level.background_scroll = Some(wildside_data::BgscrollProfile {
            layer_speeds: vec![0x08; 16],
            has_terminator: true,
        });
        visuals.palettes.background = Palette {
            colours: (0..32)
                .map(|slot| colour(10 + slot, 40 + slot, 90 + slot))
                .collect(),
        };
        visuals.background_tiles = blank_tile_sheet(1);
        visuals.background_tiles.tiles[0].pixels = [3; 64];
        visuals.background = BackgroundVisuals::Chunked(ChunkedBackgroundVisuals {
            placements: Vec::new(),
            chunks: Vec::new(),
        });

        let included = super::scene::sample_background_pixel_loc_c070(&trace, &visuals, 309, 0)
            .expect("preload-time Plane-B window should include VDP column 44");
        assert_eq!(included.tile_index, 0);
        assert_eq!(included.palette_line, 1);

        let row =
            super::scene::chunked_plane_b_row_words_for_trace_loc_30744(&trace, &visuals, 0, 0)
                .expect("chunked Plane-B row history");
        assert_eq!(
            row[45],
            Some((0, 0)),
            "ASM BackgroundScroll_ComputeShiftData loc_30452, not loc_3050A display scroll, feeds sub_30744; for `$08` row Time_Frames 114..174 with camera x=0, VDP column 45 is outside both the loc_2FFE8 preload span and the loc_30452 tile-crossing writes"
        );
        assert_eq!(
            row[48],
            Some((0, 1)),
            "ASM loc_30452/sub_304B4 crosses `$08` tile column 8 at Time_Frames 128 and writes the far-edge column 48 through sub_30744/loc_307B2"
        );
    }

    #[test]
    fn chunked_settled_row_history_replays_loc_30452_columns_into_alias_source_loc_30744() {
        let mut trace = sample_trace(GameMode::InGame);
        trace.frame_index = 60;
        trace.time_frames_loc_3038a = 174;
        trace.level_skip_cheat_enabled = 1;
        trace.camera_x_pos = 292 << 16;
        trace.camera_y_pos = 0;

        let mut visuals = sample_level_visuals();
        visuals.level.background_theme_id = 8;
        visuals.level.foreground_layout_bytes = vec![0; 80 * 14];
        visuals.level.foreground_layout =
            ForegroundLayout::parse(&visuals.level.foreground_layout_bytes, 4, 1)
                .expect("wide foreground layout");
        visuals.level.background_scroll = Some(wildside_data::BgscrollProfile {
            layer_speeds: vec![0x08; 32],
            has_terminator: true,
        });
        visuals.level.background_layout_alias = Some(BackgroundLayoutAlias {
            alias_label: "BackgroundLayout_0E".to_string(),
            source_label: "BackgroundLayout_0C".to_string(),
            source_background_path: Some("background/0C.bin".to_string()),
            x_offset_tiles: 0x13,
            y_offset_tiles: 0,
        });
        visuals.background = BackgroundVisuals::Chunked(ChunkedBackgroundVisuals {
            placements: vec![BackgroundPlacement {
                chunk_id: 0,
                x_tiles: 0x13 + 49,
                y_tiles: 18,
            }],
            chunks: vec![BackgroundChunk {
                width_tiles: 8,
                height_tiles: 1,
                tiles: (1..=8)
                    .map(|tile_index| BackgroundTile {
                        tile_index,
                        flip_x: false,
                    })
                    .collect(),
            }],
        });

        let row =
            super::scene::chunked_plane_b_row_words_for_trace_loc_30744(&trace, &visuals, 292, 18)
                .expect("settled chunked Plane-B row history");
        let cloud_tail: Vec<u16> = (49..=55)
            .map(|column| row[column].expect("column should be populated").0)
            .collect();
        assert_eq!(
            cloud_tail,
            vec![1, 2, 3, 4, 5, 6, 7],
            "ASM loc_74E0's hidden settled recording window starts 60 live frames before scenario frame 0; loc_30452/sub_304B4 then feeds sub_30744 with `$08` source/VDP columns 49..55 by Time_Frames 174, consuming the loc_12824 alias-adjusted `$87B2` source row"
        );

        let debug =
            super::scene::chunked_plane_b_row_debug_for_trace_loc_30744(&trace, &visuals, 292, 18)
                .expect("settled chunked Plane-B row debug");
        assert!(
            debug.contains("upload_data=d11<-s12w005D"),
            "ASM sub_307D8 reads the packed loc_30452 Horiz_Scroll_Data word for row-upload placement; #289 diagnostics must keep it visible beside the display-derived upload: {debug}",
        );
    }

    #[test]
    fn chunked_settled_row_history_seeds_loc_2ffe8_persisted_plane_b_row() {
        let mut trace = sample_trace(GameMode::InGame);
        trace.frame_index = 174;
        trace.time_frames_loc_3038a = 174;
        trace.level_skip_cheat_enabled = 1;
        trace.camera_x_pos = 292 << 16;
        trace.camera_y_pos = 547 << 16;
        trace.player_start_x = Some(448);
        trace.player_start_y = Some(703);

        let mut visuals = sample_level_visuals();
        visuals.level.background_theme_id = 8;
        visuals.level.foreground_layout_bytes = vec![0; 80 * 56];
        visuals.level.foreground_layout =
            ForegroundLayout::parse(&visuals.level.foreground_layout_bytes, 4, 4)
                .expect("issue #289-shaped foreground layout");
        let mut layer_speeds = vec![0xFF; 64];
        layer_speeds[8] = 0x08;
        layer_speeds[18] = 0x08;
        layer_speeds[23] = 0x00;
        layer_speeds[29] = 0x20;
        layer_speeds[40] = 0x08;
        visuals.level.background_scroll = Some(wildside_data::BgscrollProfile {
            layer_speeds,
            has_terminator: true,
        });
        visuals.level.background_layout_alias = Some(BackgroundLayoutAlias {
            alias_label: "BackgroundLayout_0E".to_string(),
            source_label: "BackgroundLayout_0C".to_string(),
            source_background_path: Some("background/0C.bin".to_string()),
            x_offset_tiles: 0x13,
            y_offset_tiles: 0,
        });
        visuals.background = BackgroundVisuals::Chunked(ChunkedBackgroundVisuals {
            placements: vec![
                BackgroundPlacement {
                    chunk_id: 0,
                    x_tiles: 0x13 + 49,
                    y_tiles: 18,
                },
                BackgroundPlacement {
                    chunk_id: 1,
                    x_tiles: 0x13 + 4,
                    y_tiles: 29,
                },
                BackgroundPlacement {
                    chunk_id: 1,
                    x_tiles: 0x13 + 40,
                    y_tiles: 29,
                },
                BackgroundPlacement {
                    chunk_id: 2,
                    x_tiles: 0x13 + 62,
                    y_tiles: 23,
                },
                BackgroundPlacement {
                    chunk_id: 3,
                    x_tiles: 0x13 + 45,
                    y_tiles: 40,
                },
                BackgroundPlacement {
                    chunk_id: 3,
                    x_tiles: 0x13 + 51,
                    y_tiles: 40,
                },
            ],
            chunks: vec![
                BackgroundChunk {
                    width_tiles: 8,
                    height_tiles: 1,
                    tiles: (1..=8)
                        .map(|tile_index| BackgroundTile {
                            tile_index,
                            flip_x: false,
                        })
                        .collect(),
                },
                BackgroundChunk {
                    width_tiles: 6,
                    height_tiles: 1,
                    tiles: [0x19, 0x15, 0x16, 0x17, 0x18, 0x39]
                        .into_iter()
                        .map(|tile_index| BackgroundTile {
                            tile_index,
                            flip_x: false,
                        })
                        .collect(),
                },
                BackgroundChunk {
                    width_tiles: 2,
                    height_tiles: 1,
                    tiles: [0x12, 0x11]
                        .into_iter()
                        .map(|tile_index| BackgroundTile {
                            tile_index,
                            flip_x: true,
                        })
                        .collect(),
                },
                BackgroundChunk {
                    width_tiles: 6,
                    height_tiles: 1,
                    tiles: [0x45, 0x46, 0x41, 0x42, 0x43, 0x44]
                        .into_iter()
                        .map(|tile_index| BackgroundTile {
                            tile_index,
                            flip_x: false,
                        })
                        .collect(),
                },
            ],
        });

        let row =
            super::scene::chunked_plane_b_row_words_for_trace_loc_30744(&trace, &visuals, 292, 18)
                .expect("settled chunked Plane-B row history");
        assert!(
            row[0..5].iter().all(|cell| *cell == Some((0, 0))),
            "ASM loc_2FFE8/sub_307D8 leaves B18 columns 0..4 as the old Plane-B `$0780` fill"
        );
        let zero_span: Vec<_> = row[5..49].to_vec();
        assert_eq!(
            zero_span,
            vec![Some((0, 1)); 44],
            "ASM loc_2FFE8/sub_307D8 seeds B18 columns 5..48 with `$2780` chunk-zero cells"
        );
        let cloud_tail: Vec<u16> = (49..=55)
            .map(|column| row[column].expect("cloud tail should be populated").0)
            .collect();
        assert_eq!(
            cloud_tail,
            vec![1, 2, 3, 4, 5, 6, 7],
            "ASM sub_30744 then accumulates the loc_12824 alias-source cloud tail after the persisted loc_2FFE8 row seed"
        );

        let ocean_row =
            super::scene::chunked_plane_b_row_words_for_trace_loc_30744(&trace, &visuals, 292, 29)
                .expect("settled chunked Plane-B ocean row history");
        assert_eq!(
            &ocean_row[0..4],
            &[Some((0, 0)); 4],
            "ASM loc_2FFE8 leaves B29 columns 0..3 on the old Plane-B `$0780` fill"
        );
        assert_eq!(
            &ocean_row[4..10],
            &[
                Some((0x19, 1)),
                Some((0x15, 1)),
                Some((0x16, 1)),
                Some((0x17, 1)),
                Some((0x18, 1)),
                Some((0x39, 1)),
            ],
            "ASM loc_2FFE8/sub_307D8 seeds B29 from the settled Camera_X column, not the pre-adjusted player-start grid column"
        );
        assert_eq!(
            ocean_row[45],
            Some((0x39, 1)),
            "ASM Load_InGame loc_11F48 first preloads Plane B at the player-start camera grid; the later settled B29 upload leaves the old far-edge cell from that grid pass intact"
        );

        let flipped_row = super::scene::chunked_plane_b_row_vdp_words_for_trace_loc_30744(
            &trace, &visuals, 292, 23,
        )
        .expect("settled chunked Plane-B flipped row history");
        assert_eq!(
            &flipped_row[62..64],
            &[Some(0x2F92), Some(0x2F91)],
            "ASM unk_309B8 maps high-bit chunk bytes from loc_12848 into the VDP H-flip bit for B23"
        );

        let wrapped_row = super::scene::chunked_plane_b_row_vdp_words_for_trace_loc_30744(
            &trace, &visuals, 292, 8,
        )
        .expect("settled chunked Plane-B wrapped row history");
        assert_eq!(
            &wrapped_row[45..56],
            &[
                Some(0x27C5),
                Some(0x27C6),
                Some(0x27C1),
                Some(0x27C2),
                Some(0x27C3),
                Some(0x27C4),
                Some(0x27C5),
                Some(0x27C6),
                Some(0x27C1),
                Some(0x27C2),
                Some(0x27C3),
            ],
            "ASM sub_30744 starts live column writes at Camera_Y_pos>>5; with camera chunk 17, Plane-B row 8 is fed by source row 40, not source row 8"
        );
    }

    #[test]
    #[ignore = "requires generated .cache assets; run mise run extract-assets first"]
    fn under_skull_mountain3_cave_background_chunk_renders_loc_12830() {
        let Some(manifest) = cached_generated_manifest() else {
            return;
        };
        let visuals = load_level_visuals(&manifest, 0x06).expect("Under Skull Mountain 3 visuals");
        assert_eq!(visuals.level.background_theme_id, 0x08);

        let mut trace = sample_trace(GameMode::InGame);
        trace.current_level = LevelId(0x06);
        trace.frame_index = 60;
        trace.camera_x_pos = 0;
        trace.camera_y_pos = 0x000C_0000;

        let sample = (0..FRAME_HEIGHT as i32)
            .flat_map(|y| (0..FRAME_WIDTH as i32).map(move |x| (x, y)))
            .find_map(|(x, y)| {
                super::scene::sample_background_pixel_loc_c070(&trace, &visuals, x, y)
                    .filter(|sample| sample.rgb != [0, 0, 0])
                    .map(|sample| (x, y, sample))
            })
            .expect("ASM Load_BackgroundMappings loc_12830 places visible cave chunks");

        assert_eq!(
            sample.2.palette_line, 1,
            "ASM sub_307D8 indexes `unk_309B8`, whose chunked background words are `$2780 + tile_id`"
        );
        assert_eq!(
            sample.2.rgb,
            [0x24, 0x00, 0x00],
            "ASM loc_12830 copies cave chunk bytes into Plane B and sub_307D8 renders them through palette_bg/cave; MD colour `$0002` is the dark red silhouette behind level 0x06 terrain"
        );
    }

    #[test]
    #[ignore = "requires generated .cache assets; run mise run extract-assets first"]
    fn under_skull_mountain3_scroll_a_keeps_issue289_edge_sliver_blank_sub_30194() {
        let Some(manifest) = cached_generated_manifest() else {
            return;
        };
        let visuals = load_level_visuals(&manifest, 0x06).expect("Under Skull Mountain 3 visuals");

        let mut trace = sample_trace(GameMode::InGame);
        trace.current_level = LevelId(0x06);
        trace.frame_index = 60;
        trace.camera_x_pos = 0;
        trace.camera_y_pos = 0x000C_0000;

        let grid =
            try_load_block_grid(&manifest, 0x06).expect("Under Skull Mountain 3 Level_Layout");
        let scene = build_scene(&trace, false, Some(&visuals), Some(&grid));
        let mut rgb = vec![0; (FRAME_WIDTH * FRAME_HEIGHT * 3) as usize];
        render_scene(&scene, &mut rgb);

        assert_eq!(
            pixel_at(&rgb, 276, 151),
            [0, 0, 0],
            "oracle frame issue-289-level06-idle-current frame 60 keeps this Scroll A edge pixel blank after ASM sub_30194/sub_302F6 uploads the visible column"
        );
    }

    #[test]
    fn forest_background_palette_keeps_line_zero_and_swapped_line_one_loc_128ba_129ae() {
        let mut visuals = sample_level_visuals();
        visuals.level.background_theme_id = 9;
        visuals.palettes.foreground = Palette {
            colours: (1..=15)
                .map(|slot| colour(slot * 10, slot * 10 + 1, slot * 10 + 2))
                .collect(),
        };
        visuals.palettes.background = Palette {
            colours: (0..8)
                .map(|slot| colour(100 + slot * 10, 101 + slot * 10, 102 + slot * 10))
                .collect(),
        };
        visuals.background_tiles = TileSheet {
            tiles: vec![make_tile(&[(0, 0, 8), (1, 0, 9)])],
        };
        visuals.background = BackgroundVisuals::Chunked(ChunkedBackgroundVisuals {
            placements: vec![BackgroundPlacement {
                chunk_id: 0,
                x_tiles: 0,
                y_tiles: 0,
            }],
            chunks: vec![BackgroundChunk {
                width_tiles: 1,
                height_tiles: 1,
                tiles: vec![BackgroundTile {
                    tile_index: 0,
                    flip_x: false,
                }],
            }],
        });
        let mut trace = sample_trace(GameMode::InGame);
        trace.camera_x_pos = 0;
        trace.camera_y_pos = 0;

        let line_zero = super::scene::sample_background_pixel_loc_c070(&trace, &visuals, 0, 0)
            .expect("line-zero forest pixel should use foreground CRAM");
        assert_eq!(
            line_zero.rgb,
            [80, 81, 82],
            "ASM loc_128BA uses base `$0780`; palette-0 Forest background pixels read foreground CRAM line 0"
        );

        if let BackgroundVisuals::Chunked(background) = &mut visuals.background {
            background.chunks[0].tiles[0] = BackgroundTile {
                tile_index: 0,
                flip_x: false,
            };
        }
        visuals.background =
            BackgroundVisuals::Layered(wildside_render::LayeredBackgroundVisuals {
                effect_start_row_selector: None,
                rows: vec![wildside_render::LayeredBackgroundRow {
                    selector: 0,
                    tiles: vec![TileRef {
                        tile_index: 0,
                        palette_line: 1,
                        priority: false,
                        flip_x: false,
                        flip_y: false,
                    }],
                }],
            });

        let line_one = super::scene::sample_background_pixel_loc_c070(&trace, &visuals, 0, 0)
            .expect("line-one forest pixel should use swapped background CRAM");
        assert_eq!(
            line_one.rgb,
            [120, 121, 122],
            "ASM sub_129AE swaps line-1 slot 8 with slot 10 after loading the Forest background palette"
        );
    }

    #[test]
    fn mountain_background_palette_keeps_followup_line_zero_loc_12910_11e16() {
        let mut visuals = sample_level_visuals();
        visuals.level.background_theme_id = 7;
        visuals.palettes.foreground = Palette {
            colours: (1..=15)
                .map(|slot| colour(slot * 10, slot * 10 + 1, slot * 10 + 2))
                .collect(),
        };
        visuals.palettes.background = Palette {
            colours: (0..8)
                .map(|slot| colour(100 + slot * 10, 101 + slot * 10, 102 + slot * 10))
                .collect(),
        };
        visuals.background_tiles = TileSheet {
            tiles: vec![make_tile(&[(0, 0, 13), (1, 0, 12)])],
        };
        visuals.background =
            BackgroundVisuals::Layered(wildside_render::LayeredBackgroundVisuals {
                effect_start_row_selector: None,
                rows: vec![wildside_render::LayeredBackgroundRow {
                    selector: 0,
                    tiles: vec![TileRef {
                        tile_index: 0,
                        palette_line: 0,
                        priority: false,
                        flip_x: false,
                        flip_y: false,
                    }],
                }],
            });
        let mut trace = sample_trace(GameMode::InGame);
        trace.camera_x_pos = 0;
        trace.camera_y_pos = 0;

        let followup_line_zero =
            super::scene::sample_background_pixel_loc_c070(&trace, &visuals, 0, 0)
                .expect("Mountain follow-up plane should read foreground CRAM line 0");
        assert_eq!(
            followup_line_zero.rgb,
            [130, 131, 132],
            "ASM loc_12910 clear-weather d0=$07E5 leaves palette-line 0 intact for CaveAlt follow-up art"
        );

        if let BackgroundVisuals::Layered(background) = &mut visuals.background {
            background.rows[0].tiles[0].palette_line = 1;
        }
        let main_line_one = super::scene::sample_background_pixel_loc_c070(&trace, &visuals, 1, 0)
            .expect("Mountain main planes should read copied background CRAM line 1");
        assert_eq!(
            main_line_one.rgb,
            [140, 141, 142],
            "ASM loc_11E16 copies background colours 1..7 into CRAM line-1 slots 9..15 for the $2780 Mountain planes"
        );
    }

    #[test]
    fn hills_background_palette_reads_foreground_foliage_plane_loc_1295e() {
        // #350: Hills (theme 3) is a two-line layered background like Mountain.
        // ASM loc_1295E decodes planes 0/1 with base `$2780` (CRAM line 1) and
        // the lower foliage plane 2 with base `$07C0` (CRAM line 0), so the
        // foliage plane must read the foreground line. Before the theme-3 branch
        // in `background_palette_for_scene` the line-0 foliage indexed the
        // background sky palette and rendered blue/purple below the tree canopy.
        let mut visuals = sample_level_visuals();
        visuals.level.background_theme_id = 3;
        visuals.palettes.foreground = Palette {
            colours: (1..=15)
                .map(|slot| colour(slot * 10, slot * 10 + 1, slot * 10 + 2))
                .collect(),
        };
        visuals.palettes.background = Palette {
            colours: (0..8)
                .map(|slot| colour(100 + slot * 10, 101 + slot * 10, 102 + slot * 10))
                .collect(),
        };
        visuals.background_tiles = TileSheet {
            tiles: vec![make_tile(&[(0, 0, 13), (1, 0, 12)])],
        };
        visuals.background =
            BackgroundVisuals::Layered(wildside_render::LayeredBackgroundVisuals {
                effect_start_row_selector: None,
                rows: vec![wildside_render::LayeredBackgroundRow {
                    selector: 0,
                    tiles: vec![TileRef {
                        tile_index: 0,
                        palette_line: 0,
                        priority: false,
                        flip_x: false,
                        flip_y: false,
                    }],
                }],
            });
        let mut trace = sample_trace(GameMode::InGame);
        trace.camera_x_pos = 0;
        trace.camera_y_pos = 0;

        let foliage_line_zero =
            super::scene::sample_background_pixel_loc_c070(&trace, &visuals, 0, 0)
                .expect("Hill foliage plane 2 should read foreground CRAM line 0");
        assert_eq!(
            foliage_line_zero.rgb,
            [130, 131, 132],
            "ASM loc_1295E decodes the Hill foliage plane with `$07C0`, so palette-line 0 reads foreground CRAM line 0"
        );

        if let BackgroundVisuals::Layered(background) = &mut visuals.background {
            background.rows[0].tiles[0].palette_line = 1;
        }
        let main_line_one = super::scene::sample_background_pixel_loc_c070(&trace, &visuals, 1, 0)
            .expect("Hill main planes 0/1 should read copied background CRAM line 1");
        assert_eq!(
            main_line_one.rgb,
            [140, 141, 142],
            "ASM loc_1295E decodes Hill planes 0/1 with `$2780`; loc_11E16 copies background colours 1..7 into CRAM line-1"
        );
    }

    #[test]
    fn build_game_scene_draws_murder_wall_from_sub_5e02_sprite_tiles() {
        let mut trace = sample_trace(GameMode::InGame);
        trace.frame_index = 10;
        trace.camera_x_pos = 0;
        trace.camera_y_pos = 0;
        trace.murder_wall_x = Some(0);
        let mut visuals = sample_level_visuals();
        visuals.level.murder_wall_flags = 2;
        visuals.murder_wall = Some(wildside_render::MurderWallVisuals {
            palette: Palette {
                colours: (0..8).map(|value| colour(value * 16, 0, 0)).collect(),
            },
            tile_sheet: TileSheet {
                tiles: vec![Tile { pixels: [8; 64] }; 0x90],
            },
            vram_base_tile_index: 0x02FB,
        });

        let scene = build_scene_filtered(
            &trace,
            false,
            Some(&visuals),
            None,
            LayerFilter {
                background: false,
                terrain: false,
                block_overrides: false,
                platforms: false,
                enemies: false,
                player: false,
                effects: true,
                eol_score: false,
                hud: false,
            },
        );

        let wall_sprites = scene
            .commands
            .iter()
            .filter(|command| {
                matches!(
                    command,
                    DrawCommand::Sprite { tile_sheet, tiles, .. }
                        if tile_sheet.tiles.len() == 0x90 && tiles.len() == 9
                )
            })
            .count();
        assert_eq!(
            wall_sprites, 24,
            "ASM sub_5E02 should emit 24 murder-wall hardware sprites from ROM tile art"
        );
        assert!(
            !scene.commands.iter().any(|command| {
                matches!(command, DrawCommand::Rect { colour, .. } if *colour == [200, 30, 10])
            }),
            "loaded Murder_wall.bin art should replace the native-player placeholder rect"
        );
    }

    #[test]
    fn murder_wall_warning_strip_clears_priority_before_hud_loc_80c_bee8() {
        let mut trace = sample_trace(GameMode::InGame);
        trace.frame_index = 10;
        trace.camera_x_pos = 0;
        trace.camera_y_pos = 0;
        trace.level_timer_seconds = 179;
        trace.murder_wall_x = Some(16 << 16);
        let mut visuals = sample_level_visuals();
        visuals.level.murder_wall_flags = 2;
        visuals.murder_wall = Some(wildside_render::MurderWallVisuals {
            palette: Palette {
                colours: (0..48)
                    .map(|index| {
                        if index == 0x11 {
                            colour(220, 20, 20)
                        } else {
                            colour(0, 0, 0)
                        }
                    })
                    .collect(),
            },
            tile_sheet: TileSheet {
                tiles: vec![Tile { pixels: [1; 64] }; 0x90],
            },
            vram_base_tile_index: 0x02FB,
        });

        let scene = build_scene_filtered(
            &trace,
            false,
            Some(&visuals),
            None,
            LayerFilter {
                background: false,
                terrain: false,
                block_overrides: false,
                platforms: false,
                enemies: false,
                player: false,
                effects: true,
                eol_score: false,
                hud: true,
            },
        );
        let wall_index = scene
            .commands
            .iter()
            .position(|command| {
                matches!(
                    command,
                    DrawCommand::Sprite { tile_sheet, tiles, .. }
                        if tile_sheet.tiles.len() == 0x90
                            && tiles.iter().any(|tile| tile.tile.priority)
                )
            })
            .expect("murder-wall strip command");
        let clear_index = scene
            .commands
            .iter()
            .position(|command| {
                matches!(
                    command,
                    DrawCommand::ClearPriorityRect { x: 0, y: 0, width, height }
                        if *width == FRAME_WIDTH as i32 && *height >= 48
                )
            })
            .expect("HUD priority clear");
        let hud_index = scene
            .commands
            .iter()
            .position(|command| {
                matches!(
                    command,
                    DrawCommand::IndexedTile { tile_sheet, .. }
                        if tile_sheet.tiles.len() == 12
                )
            })
            .expect("HUD timer digit");

        assert!(
            wall_index < clear_index && clear_index < hud_index,
            "ASM Mode_Level loc_80C runs sub_5E02 after Make_SpriteAttr_HUD, but native-player must clear its local priority mask before the fixed HUD overlay so the loc_BEE8 timer remains visible"
        );
    }

    #[test]
    fn build_game_scene_murder_wall_fallback_is_visible_warning_strip() {
        let mut trace = sample_trace(GameMode::InGame);
        trace.frame_index = 240;
        trace.camera_x_pos = 0;
        trace.camera_y_pos = 0;
        trace.murder_wall_x = Some(0);
        let mut visuals = sample_level_visuals();
        visuals.level.murder_wall_flags = 2;
        visuals.murder_wall = None;

        let scene = build_scene_filtered(
            &trace,
            false,
            Some(&visuals),
            None,
            LayerFilter {
                background: false,
                terrain: false,
                block_overrides: false,
                platforms: false,
                enemies: false,
                player: false,
                effects: true,
                eol_score: false,
                hud: false,
            },
        );

        let fallback_rects = scene
            .commands
            .iter()
            .filter_map(|command| match command {
                DrawCommand::Rect { colour, .. }
                    if matches!(*colour, [190, 20, 8] | [65, 10, 10] | [255, 160, 70]) =>
                {
                    Some(*colour)
                }
                _ => None,
            })
            .collect::<Vec<_>>();

        assert!(
            fallback_rects.len() > 8,
            "ASM sub_5E02 fallback should be a visible warning/drill strip, not the old single red slab"
        );
        assert!(
            fallback_rects.contains(&[65, 10, 10]) && fallback_rects.contains(&[255, 160, 70]),
            "fallback should include contrast bands so the death wall is visible during play-native validation"
        );
    }

    #[test]
    #[ignore = "requires generated .cache assets; run mise run extract-assets first"]
    fn level08_emits_murder_wall_effects_from_generated_assets_sub_5e02() {
        let Some(manifest) = cached_generated_manifest() else {
            return;
        };
        let (visuals, block_grid) =
            load_session_assets(&manifest, 0x08).expect("Level 0x08 generated visuals");
        let scenario = crate::scenarios::idle_level_scenario(0x08, Some(400));
        let trace_frames = wildside_core::run_script_with_warp(
            manifest,
            &scenario.script,
            scenario.frame_count,
            scenario.warp_to_level,
        );

        let (trace_index, trace, render) = trace_frames
            .iter()
            .enumerate()
            .filter(|(_, frame)| frame.game_mode == GameMode::InGame)
            .find_map(|(trace_index, frame)| {
                let wall_x_fp = frame.murder_wall_x?;
                let render = wildside_core::compile_murder_wall_warning_sprites_sub_5E02(
                    wildside_core::MurderWallWarningInputSub5E02 {
                        murder_wall_enabled: true,
                        murder_wall_x_px: (wall_x_fp >> 16) as i16,
                        murder_wall_reversed: visuals.level.murder_wall_flags & 1 != 0,
                        kid_hitbox_left_px: 0,
                        kid_hitbox_right_px: 0,
                        camera_x_px: (frame.camera_x_pos >> 16) as i16,
                        camera_y_px: (frame.camera_y_pos >> 16) as u16,
                        animation_timer: 2,
                        animation_frame: ((frame.frame_index / 5) & 3) as u8,
                        starting_number_sprites: 0,
                    },
                );
                (!render.sprites.is_empty()).then_some((trace_index, frame, render))
            })
            .expect(
                "ASM sub_5E02 should emit visible wall sprites before Level 0x08 exits gameplay",
            );
        let scene = build_scene_filtered(
            trace,
            false,
            Some(&visuals),
            Some(&block_grid),
            LayerFilter {
                background: false,
                terrain: false,
                block_overrides: false,
                platforms: false,
                enemies: false,
                player: false,
                effects: true,
                eol_score: false,
                hud: false,
            },
        );

        let visible_wall_effect = scene.commands.iter().any(|command| {
            matches!(
                command,
                DrawCommand::Sprite { tiles, .. } if tiles.len() == 9
            ) || matches!(command, DrawCommand::Rect { colour, .. } if *colour == [200, 30, 10])
        });
        assert!(
            visible_wall_effect,
            "ASM sub_5E02 should render the Level 0x08 murder-wall strip before it crushes Kid; murder_wall_tiles={:?} emitted_sprites={} scene_commands={} trace_index={}",
            visuals
                .murder_wall
                .as_ref()
                .map(|murder_wall| murder_wall.tile_sheet.tiles.len()),
            render.sprites.len(),
            scene.commands.len(),
            trace_index
        );
    }

    #[test]
    fn build_game_scene_draws_lava_geyser_from_loc_14be8_sprite_tiles() {
        let mut trace = sample_trace(GameMode::InGame);
        trace.camera_x_pos = 0;
        trace.camera_y_pos = 0;
        trace.active_geysers = vec![(0x0080, 0x0040, 0x0030, u16::MAX, u8::MAX)];
        let mut visuals = sample_level_visuals();
        visuals.lava = Some(wildside_render::LavaVisuals {
            palette: Palette {
                colours: (0..8).map(|value| colour(value * 16, 0, 0)).collect(),
            },
            tile_sheet: TileSheet {
                tiles: vec![Tile { pixels: [8; 64] }; 0xC0],
            },
            vram_base_tile_index: 0x02FB,
        });

        let scene = build_scene_filtered(
            &trace,
            false,
            Some(&visuals),
            None,
            LayerFilter {
                background: false,
                terrain: false,
                block_overrides: false,
                platforms: false,
                enemies: false,
                player: false,
                effects: true,
                eol_score: false,
                hud: false,
            },
        );

        let lava_sprites = scene
            .commands
            .iter()
            .filter(|command| {
                matches!(
                    command,
                    DrawCommand::Sprite { tile_sheet, .. } if tile_sheet.tiles.len() == 0xC0
                )
            })
            .count();
        assert_eq!(
            lava_sprites, 6,
            "ASM loc_14874/loc_148F8 should render the top, three middle, and bottom loc_14BE8 full-height sprite rows"
        );
        assert!(
            !scene.commands.iter().any(|command| {
                matches!(command, DrawCommand::Rect { colour, .. } if *colour == [255, 120, 20])
            }),
            "loaded Lava.bin art should replace the native-player placeholder column"
        );
    }

    #[test]
    fn build_game_scene_culls_lava_geyser_full_height_rows_loc_14874() {
        let mut trace = sample_trace(GameMode::InGame);
        trace.camera_x_pos = 0x0100_0000;
        trace.camera_y_pos = 0;
        trace.active_geysers = vec![(0x00DF, 0x0030, 0x0030, u16::MAX, u8::MAX)];
        let mut visuals = sample_level_visuals();
        visuals.lava = Some(wildside_render::LavaVisuals {
            palette: Palette {
                colours: (0..8).map(|value| colour(value * 16, 0, 0)).collect(),
            },
            tile_sheet: TileSheet {
                tiles: vec![Tile { pixels: [8; 64] }; 0xC0],
            },
            vram_base_tile_index: 0x02FB,
        });

        let scene = build_scene_filtered(
            &trace,
            false,
            Some(&visuals),
            None,
            LayerFilter {
                background: false,
                terrain: false,
                block_overrides: false,
                platforms: false,
                enemies: false,
                player: false,
                effects: true,
                eol_score: false,
                hud: false,
            },
        );

        assert!(
            !scene.commands.iter().any(|command| {
                matches!(
                    command,
                    DrawCommand::Sprite { tile_sheet, .. } if tile_sheet.tiles.len() == 0xC0
                ) || matches!(command, DrawCommand::Rect { colour, .. } if *colour == [255, 120, 20])
            }),
            "ASM loc_14874 should cull off-window full-height rows without falling back to the legacy lava column"
        );
    }

    #[test]
    fn build_game_scene_uses_lava_geyser_top_alt_template_dword_14636() {
        let mut trace = sample_trace(GameMode::InGame);
        trace.camera_x_pos = 0;
        trace.camera_y_pos = 0;
        trace.active_geysers = vec![(0x0080, 0x0040, 0x0030, 1, 1)];
        let mut visuals = sample_level_visuals();
        visuals.lava = Some(wildside_render::LavaVisuals {
            palette: Palette {
                colours: (0..8).map(|value| colour(value * 16, 0, 0)).collect(),
            },
            tile_sheet: TileSheet {
                tiles: vec![Tile { pixels: [8; 64] }; 0xC0],
            },
            vram_base_tile_index: 0x02FB,
        });

        let scene = build_scene_filtered(
            &trace,
            false,
            Some(&visuals),
            None,
            LayerFilter {
                background: false,
                terrain: false,
                block_overrides: false,
                platforms: false,
                enemies: false,
                player: false,
                effects: true,
                eol_score: false,
                hud: false,
            },
        );

        let lava_tile_indices: Vec<u16> = scene
            .commands
            .iter()
            .flat_map(|command| match command {
                DrawCommand::Sprite {
                    tile_sheet, tiles, ..
                } if tile_sheet.tiles.len() == 0xC0 => {
                    tiles.iter().map(|tile| tile.tile.tile_index).collect()
                }
                _ => Vec::new(),
            })
            .collect();
        assert!(
            lava_tile_indices.contains(&0x00AF),
            "ASM loc_148CE should select dword_14636 when `$4C(a5)` is non-zero; loc_14BE8 then adds the incremented sprite count to tile `$3A9`"
        );
    }

    #[test]
    fn build_game_scene_draws_lava_geyser_script_template_loc_147ba() {
        let mut trace = sample_trace(GameMode::InGame);
        trace.camera_x_pos = 0;
        trace.camera_y_pos = 0;
        trace.active_geysers = vec![(0x0080, 0x0030, 0x0010, u16::MAX, u8::MAX)];
        trace.lava_geyser_script_templates_loc_147ba =
            vec![wildside_core::LavaGeyserScriptTemplateTraceLoc147BA {
                x_px: 0x0080,
                base_y_px: 0x0040,
                source_label: "unk_14602".to_owned(),
            }];
        let mut visuals = sample_level_visuals();
        visuals.lava = Some(wildside_render::LavaVisuals {
            palette: Palette {
                colours: (0..8).map(|value| colour(value * 16, 0, 0)).collect(),
            },
            tile_sheet: TileSheet {
                tiles: vec![Tile { pixels: [8; 64] }; 0xC0],
            },
            vram_base_tile_index: 0x02FB,
        });

        let scene = build_scene_filtered(
            &trace,
            false,
            Some(&visuals),
            None,
            LayerFilter {
                background: false,
                terrain: false,
                block_overrides: false,
                platforms: false,
                enemies: false,
                player: false,
                effects: true,
                eol_score: false,
                hud: false,
            },
        );

        let lava_sprites = scene
            .commands
            .iter()
            .filter(|command| {
                matches!(
                    command,
                    DrawCommand::Sprite { tile_sheet, .. } if tile_sheet.tiles.len() == 0xC0
                )
            })
            .count();
        assert_eq!(
            lava_sprites, 4,
            "ASM loc_147BA should draw the exact four-entry `unk_14602` script template and skip the legacy height-column fallback for that geyser"
        );
    }

    #[test]
    fn build_game_scene_culls_lava_geyser_script_template_loc_147d6() {
        let mut trace = sample_trace(GameMode::InGame);
        trace.camera_x_pos = 0x0100_0000;
        trace.camera_y_pos = 0;
        trace.active_geysers = vec![(0x00DF, 0x0030, 0x0010, u16::MAX, u8::MAX)];
        trace.lava_geyser_script_templates_loc_147ba =
            vec![wildside_core::LavaGeyserScriptTemplateTraceLoc147BA {
                x_px: 0x00DF,
                base_y_px: 0x0040,
                source_label: "unk_14602".to_owned(),
            }];
        let mut visuals = sample_level_visuals();
        visuals.lava = Some(wildside_render::LavaVisuals {
            palette: Palette {
                colours: (0..8).map(|value| colour(value * 16, 0, 0)).collect(),
            },
            tile_sheet: TileSheet {
                tiles: vec![Tile { pixels: [8; 64] }; 0xC0],
            },
            vram_base_tile_index: 0x02FB,
        });

        let scene = build_scene_filtered(
            &trace,
            false,
            Some(&visuals),
            None,
            LayerFilter {
                background: false,
                terrain: false,
                block_overrides: false,
                platforms: false,
                enemies: false,
                player: false,
                effects: true,
                eol_score: false,
                hud: false,
            },
        );

        assert!(
            !scene.commands.iter().any(|command| {
                matches!(
                    command,
                    DrawCommand::Sprite { tile_sheet, .. } if tile_sheet.tiles.len() == 0xC0
                ) || matches!(command, DrawCommand::Rect { colour, .. } if *colour == [255, 120, 20])
            }),
            "ASM loc_147D6 should cull off-window loc_147BA script rows without falling back to the legacy lava column"
        );
    }

    #[test]
    fn build_game_scene_uses_live_lava_geyser_palette_words_loc_14b2a() {
        let mut trace = sample_trace(GameMode::InGame);
        trace.camera_x_pos = 0;
        trace.camera_y_pos = 0;
        trace.active_geysers = vec![(0x0080, 0x0040, 0x0010, u16::MAX, u8::MAX)];
        trace.lava_geyser_palette_words_loc_14b2a = Some([
            0x0CCC, 0x0222, 0x0444, 0x0666, 0x0004, 0x0008, 0x000C, 0x002E,
        ]);
        let mut visuals = sample_level_visuals();
        visuals.lava = Some(wildside_render::LavaVisuals {
            palette: Palette {
                colours: (0..8)
                    .map(|value| MdColour::from_word(0x0100 + value))
                    .collect(),
            },
            tile_sheet: TileSheet {
                tiles: vec![Tile { pixels: [8; 64] }; 0xC0],
            },
            vram_base_tile_index: 0x02FB,
        });

        let scene = build_scene_filtered(
            &trace,
            false,
            Some(&visuals),
            None,
            LayerFilter {
                background: false,
                terrain: false,
                block_overrides: false,
                platforms: false,
                enemies: false,
                player: false,
                effects: true,
                eol_score: false,
                hud: false,
            },
        );

        let lava_sprite = scene
            .commands
            .iter()
            .find(|command| {
                matches!(
                    command,
                    DrawCommand::Sprite { tile_sheet, .. } if tile_sheet.tiles.len() == 0xC0
                )
            })
            .expect("ASM loc_14BE8 lava geyser sprite");
        let raw_words: Vec<u16> = sprite_palette(lava_sprite)
            .colours
            .iter()
            .take(8)
            .map(|colour| colour.raw_word)
            .collect();
        assert_eq!(
            raw_words,
            vec![
                0x0CCC, 0x0222, 0x0444, 0x0666, 0x0004, 0x0008, 0x000C, 0x002E,
            ],
            "ASM loc_14BE8 draws Lava.bin sprite art using the live Palette_Buffer+$20 line seeded by loc_14C6A and rotated by loc_14B2A"
        );
    }

    #[test]
    fn build_game_scene_draws_storm_weather_sprites_from_sub_61ca() {
        let mut trace = sample_trace(GameMode::InGame);
        trace.storm_sprites_sub_61ca = vec![(0x0088, 0x0501, 0xA3CD, 0x019F)];
        let mut visuals = sample_level_visuals();
        visuals.weather = Some(wildside_render::WeatherVisuals {
            palette: Palette {
                colours: (0..8)
                    .map(|value| MdColour::from_word(0x0600 + value))
                    .collect(),
            },
            tile_sheet: TileSheet {
                tiles: vec![Tile { pixels: [8; 64] }; 0x60],
            },
            vram_base_tile_index: 0x0380,
            shared_art: None,
        });

        let scene = build_scene_filtered(
            &trace,
            false,
            Some(&visuals),
            None,
            LayerFilter {
                background: false,
                terrain: false,
                block_overrides: false,
                platforms: false,
                enemies: false,
                player: false,
                effects: true,
                eol_score: false,
                hud: false,
            },
        );

        let weather_sprite = scene
            .commands
            .iter()
            .find(|command| {
                matches!(
                    command,
                    DrawCommand::Sprite { tile_sheet, .. } if tile_sheet.tiles.len() == 0x60
                )
            })
            .expect("ASM sub_61CA loc_63C4/loc_6462 storm sprite");
        match weather_sprite {
            DrawCommand::Sprite { x, y, tiles, .. } => {
                assert_eq!((*x, *y), (0x011F, 0x0008));
                assert_eq!(tiles.len(), 1);
                assert_eq!(tiles[0].tile.tile_index, 0x004D);
                assert_eq!(tiles[0].tile.palette_line, 1);
                assert!(
                    tiles[0].tile.priority,
                    "ASM sub_61CA loc_63C4 writes priority tile word $A3CD"
                );
            }
            _ => unreachable!("weather sprite command matched Sprite above"),
        }
    }

    #[test]
    fn build_game_scene_uses_loc_14d8a_shared_weather_art_for_storm_sprites() {
        let mut trace = sample_trace(GameMode::InGame);
        trace.storm_sprites_sub_61ca = vec![(0x0088, 0x0501, 0xA3CD, 0x019F)];
        trace.storm_shared_art_upload_loc_14d8a =
            Some(wildside_core::StormSharedArtRuntimeUploadLoc14D8A {
                source_asset_label: String::from("ArtComp_9A7D2"),
                vram_word: 0x76A0,
            });
        let mut visuals = sample_level_visuals();
        visuals.weather = Some(wildside_render::WeatherVisuals {
            palette: Palette {
                colours: (0..8)
                    .map(|value| MdColour::from_word(0x0600 + value))
                    .collect(),
            },
            tile_sheet: TileSheet {
                tiles: vec![Tile { pixels: [8; 64] }; 0x60],
            },
            vram_base_tile_index: 0x0380,
            shared_art: Some(wildside_render::StormSharedWeatherArtVisuals {
                tile_sheet: TileSheet {
                    tiles: vec![Tile { pixels: [9; 64] }; 0x20],
                },
                vram_base_tile_index: 0x03B5,
            }),
        });

        let scene = build_scene_filtered(
            &trace,
            false,
            Some(&visuals),
            None,
            LayerFilter {
                background: false,
                terrain: false,
                block_overrides: false,
                platforms: false,
                enemies: false,
                player: false,
                effects: true,
                eol_score: false,
                hud: false,
            },
        );

        let weather_sprite = scene
            .commands
            .iter()
            .find(|command| match command {
                DrawCommand::Sprite { tile_sheet, .. } => {
                    tile_sheet.tiles.len() == 0x20
                        && tile_sheet
                            .tiles
                            .first()
                            .is_some_and(|tile| tile.pixels == [9; 64])
                }
                _ => false,
            })
            .expect("ASM Init_SpecialEffect_Storm loc_14D8A shared art for sub_61CA sprite");
        match weather_sprite {
            DrawCommand::Sprite { x, y, tiles, .. } => {
                assert_eq!((*x, *y), (0x011F, 0x0008));
                assert_eq!(tiles.len(), 1);
                assert_eq!(
                    tiles[0].tile.tile_index, 0x0018,
                    "ASM loc_14D8A uploads ArtComp_9A7D2 to tile $3B5, so sub_61CA tile $3CD resolves to shared local tile $18"
                );
                assert_eq!(tiles[0].tile.palette_line, 1);
                assert!(
                    tiles[0].tile.priority,
                    "ASM sub_61CA loc_63C4 writes priority tile word $A3CD"
                );
            }
            _ => unreachable!("weather sprite command matched Sprite above"),
        }
    }

    #[test]
    fn build_game_scene_gates_storm_pattern_plane_on_loc_30110_transfer() {
        let mut trace = sample_trace(GameMode::InGame);
        trace.storm_pattern_upload_loc_14d32 =
            Some(wildside_core::StormPatternRuntimeUploadLoc14D32 {
                source_asset_label: String::from("unk_9B65C"),
                vram_tile_addend: 0x0380_0380,
                skipped_header_longwords: 1,
                vdp_command: 0x4000_0003,
                written_longword_count: 4 * 8 * 8 * 4,
            });
        let mut visuals = sample_level_visuals();
        visuals.weather = Some(wildside_render::WeatherVisuals {
            palette: Palette {
                colours: (0..8)
                    .map(|value| MdColour::from_word(0x0600 + value))
                    .collect(),
            },
            tile_sheet: TileSheet {
                tiles: vec![Tile { pixels: [8; 64] }; 0x60],
            },
            vram_base_tile_index: 0x0380,
            shared_art: None,
        });

        let scene = build_scene_filtered(
            &trace,
            false,
            Some(&visuals),
            None,
            LayerFilter {
                background: true,
                terrain: false,
                block_overrides: false,
                platforms: false,
                enemies: false,
                player: false,
                effects: false,
                eol_score: false,
                hud: false,
            },
        );

        let storm_tiles = scene
            .commands
            .iter()
            .filter(|command| {
                matches!(
                    command,
                    DrawCommand::IndexedTile { tile_sheet, .. } if tile_sheet.tiles.len() == 0x60
                )
            })
            .count();
        assert_eq!(
            storm_tiles, 0,
            "ASM Transfer_ScrollDataToVRAM loc_300FE keeps the normal plane-B path active until `$FFFFFAD6` reaches zero and branches to loc_30110"
        );
    }

    #[test]
    fn build_game_scene_draws_storm_pattern_plane_loc_30110() {
        let mut trace = sample_trace(GameMode::InGame);
        trace.storm_pattern_upload_loc_14d32 =
            Some(wildside_core::StormPatternRuntimeUploadLoc14D32 {
                source_asset_label: String::from("unk_9B65C"),
                vram_tile_addend: 0x0380_0380,
                skipped_header_longwords: 1,
                vdp_command: 0x4000_0003,
                written_longword_count: 4 * 8 * 8 * 4,
            });
        trace.storm_scroll_transfer_loc_30110 =
            Some(wildside_core::TransferScrollDataStormLoc30110 {
                plane_address_word: 0x8403,
                h_scroll_word_pairs: 0x00E0,
                first_h_scroll_word: -0x0010,
                second_h_scroll_word: 0,
                first_vsram_word: 0x0008,
                second_vsram_word: 0,
                weather_mode_byte: 4,
                next_storm_scroll_offset: -8,
            });
        let mut visuals = sample_level_visuals();
        visuals.weather = Some(wildside_render::WeatherVisuals {
            palette: Palette {
                colours: (0..8)
                    .map(|value| MdColour::from_word(0x0600 + value))
                    .collect(),
            },
            tile_sheet: TileSheet {
                tiles: vec![Tile { pixels: [8; 64] }; 0x60],
            },
            vram_base_tile_index: 0x0380,
            shared_art: None,
        });

        let scene = build_scene_filtered(
            &trace,
            false,
            Some(&visuals),
            None,
            LayerFilter {
                background: true,
                terrain: false,
                block_overrides: false,
                platforms: false,
                enemies: false,
                player: false,
                effects: false,
                eol_score: false,
                hud: false,
            },
        );

        let storm_tiles: Vec<_> = scene
            .commands
            .iter()
            .filter_map(|command| match command {
                DrawCommand::IndexedTile {
                    x,
                    y,
                    tile_sheet,
                    tile,
                    ..
                } if tile_sheet.tiles.len() == 0x60 => Some((*x, *y, *tile)),
                _ => None,
            })
            .collect();
        assert_eq!(
            storm_tiles.len(),
            (FRAME_WIDTH.div_ceil(8) as usize + 1) * (FRAME_HEIGHT.div_ceil(8) as usize + 1),
            "ASM Transfer_ScrollDataToVRAM loc_30110 displays enough Scroll-B pattern data to cover the visible plane"
        );
        assert_eq!(storm_tiles[0].0, 0);
        assert_eq!(storm_tiles[0].1, 0);
        assert_eq!(storm_tiles[0].2.tile_index, 0);
        assert_eq!(storm_tiles[0].2.palette_line, 0);
        assert!(!storm_tiles[0].2.priority);
        assert_eq!(
            storm_tiles[8].2.tile_index, 0,
            "ASM loc_14D32 repeats the 8-word pattern group across Scroll B"
        );
        let row_1_col_0 = FRAME_WIDTH.div_ceil(8) as usize + 1;
        assert_eq!(
            storm_tiles[row_1_col_0].2.tile_index, 8,
            "ASM unk_9B65C second post-header group starts at source tile word $0008"
        );
    }

    #[test]
    fn build_game_scene_scrolls_storm_pattern_plane_from_loc_30110_offset() {
        let mut trace = sample_trace(GameMode::InGame);
        trace.storm_pattern_upload_loc_14d32 =
            Some(wildside_core::StormPatternRuntimeUploadLoc14D32 {
                source_asset_label: String::from("unk_9B65C"),
                vram_tile_addend: 0x0380_0380,
                skipped_header_longwords: 1,
                vdp_command: 0x4000_0003,
                written_longword_count: 4 * 8 * 8 * 4,
            });
        trace.storm_scroll_transfer_loc_30110 =
            Some(wildside_core::TransferScrollDataStormLoc30110 {
                plane_address_word: 0x8403,
                h_scroll_word_pairs: 0x00E0,
                first_h_scroll_word: -0x0010,
                second_h_scroll_word: -8,
                first_vsram_word: 0x0008,
                second_vsram_word: -8,
                weather_mode_byte: 4,
                next_storm_scroll_offset: -16,
            });
        let mut visuals = sample_level_visuals();
        visuals.weather = Some(wildside_render::WeatherVisuals {
            palette: Palette {
                colours: (0..8)
                    .map(|value| MdColour::from_word(0x0600 + value))
                    .collect(),
            },
            tile_sheet: TileSheet {
                tiles: vec![Tile { pixels: [8; 64] }; 0x60],
            },
            vram_base_tile_index: 0x0380,
            shared_art: None,
        });

        let scene = build_scene_filtered(
            &trace,
            false,
            Some(&visuals),
            None,
            LayerFilter {
                background: true,
                terrain: false,
                block_overrides: false,
                platforms: false,
                enemies: false,
                player: false,
                effects: false,
                eol_score: false,
                hud: false,
            },
        );

        let first_storm_tile = scene
            .commands
            .iter()
            .find_map(|command| match command {
                DrawCommand::IndexedTile {
                    x,
                    y,
                    tile_sheet,
                    tile,
                    ..
                } if tile_sheet.tiles.len() == 0x60 => Some((*x, *y, *tile)),
                _ => None,
            })
            .expect("storm transfer should draw Scroll-B pattern tiles");
        assert_eq!(
            (first_storm_tile.0, first_storm_tile.1),
            (0, 0),
            "ASM loc_30110 uses tile-aligned `$FFFFFAD8` steps, so a -8 offset keeps the first visible tile aligned to the viewport"
        );
        assert_eq!(
            first_storm_tile.2.tile_index, 0x0022,
            "ASM loc_30110 Scroll-B offset -8 samples the previous 8x8 pattern row/column; unk_9B65C row 7 col 7 is tile `$22`"
        );
    }

    #[test]
    fn build_game_scene_preserves_hail_storm_pattern_attr_bits_loc_14d32() {
        let mut trace = sample_trace(GameMode::InGame);
        trace.storm_pattern_upload_loc_14d32 =
            Some(wildside_core::StormPatternRuntimeUploadLoc14D32 {
                source_asset_label: String::from("unk_9B6E0"),
                vram_tile_addend: 0x0380_0380,
                skipped_header_longwords: 1,
                vdp_command: 0x4000_0003,
                written_longword_count: 4 * 8 * 8 * 4,
            });
        trace.storm_scroll_transfer_loc_30110 =
            Some(wildside_core::TransferScrollDataStormLoc30110 {
                plane_address_word: 0x8403,
                h_scroll_word_pairs: 0x00E0,
                first_h_scroll_word: -0x0010,
                second_h_scroll_word: 0,
                first_vsram_word: 0x0008,
                second_vsram_word: 0,
                weather_mode_byte: 4,
                next_storm_scroll_offset: -8,
            });
        let mut visuals = sample_level_visuals();
        visuals.weather = Some(wildside_render::WeatherVisuals {
            palette: Palette {
                colours: (0..8)
                    .map(|value| MdColour::from_word(0x0600 + value))
                    .collect(),
            },
            tile_sheet: TileSheet {
                tiles: vec![Tile { pixels: [8; 64] }; 0x60],
            },
            vram_base_tile_index: 0x0380,
            shared_art: None,
        });

        let scene = build_scene_filtered(
            &trace,
            false,
            Some(&visuals),
            None,
            LayerFilter {
                background: true,
                terrain: false,
                block_overrides: false,
                platforms: false,
                enemies: false,
                player: false,
                effects: false,
                eol_score: false,
                hud: false,
            },
        );

        let storm_tiles: Vec<_> = scene
            .commands
            .iter()
            .filter_map(|command| match command {
                DrawCommand::IndexedTile {
                    tile_sheet, tile, ..
                } if tile_sheet.tiles.len() == 0x60 => Some(*tile),
                _ => None,
            })
            .collect();
        let visible_cols = FRAME_WIDTH.div_ceil(8) as usize + 1;
        let attr_tile = storm_tiles[6 * visible_cols + 1];
        assert_eq!(
            attr_tile.tile_index, 0x0013,
            "ASM unk_9B6E0 word $1813 keeps tile id $13 after loc_14D32 adds VRAM base $0380"
        );
        assert!(attr_tile.flip_x, "ASM hail word $1813 sets H-flip");
        assert!(attr_tile.flip_y, "ASM hail word $1813 sets V-flip");
        assert_eq!(attr_tile.palette_line, 0);
        assert!(!attr_tile.priority);
    }

    #[test]
    fn build_game_scene_draws_unknown_foreground_plane_loc_14e42() {
        let mut trace = sample_trace(GameMode::InGame);
        trace.camera_x_pos = 0;
        trace.camera_y_pos = 0;
        trace.special_effect_unknown_plane_fill_loc_14e42 = Some(
            wildside_core::SpecialEffectUnknownPlaneFillRuntimeLoc14E42 {
                vdp_command: 0x4000_0003,
                outer_pass_count: 8,
                group_count_per_pass: 4,
                inner_repeat_count: 16,
                written_longword_count: 8 * 4 * 16 * 2,
                foreground_plane_register: 0x8218,
                background_plane_register: 0x8400,
            },
        );
        let mut visuals = sample_level_visuals();
        visuals.special_effect_unknown = Some(SpecialEffectUnknownVisuals {
            tile_sheet: TileSheet {
                tiles: vec![Tile { pixels: [2; 64] }; 16],
            },
            vram_base_tile_index: 0x0380,
        });

        let scene = build_scene_filtered(
            &trace,
            false,
            Some(&visuals),
            None,
            LayerFilter {
                background: false,
                terrain: true,
                block_overrides: false,
                platforms: false,
                enemies: false,
                player: false,
                effects: false,
                eol_score: false,
                hud: false,
            },
        );

        let unknown_tiles = scene
            .commands
            .iter()
            .filter(|command| {
                matches!(
                    command,
                    DrawCommand::IndexedTile { tile_sheet, .. } if tile_sheet.tiles.len() == 16
                )
            })
            .collect::<Vec<_>>();
        assert_eq!(
            unknown_tiles.len(),
            41 * 29,
            "ASM loc_14E42 fills a 64x32 foreground plane; native draws one 8px margin around the 320x224 viewport"
        );
        match unknown_tiles[0] {
            DrawCommand::IndexedTile { x, y, tile, .. } => {
                assert_eq!((*x, *y), (0, 0));
                assert_eq!(
                    tile.tile_index, 0,
                    "ASM loc_14E46 first word `$8380` is absolute VRAM tile `$380`, local tile 0"
                );
                assert_eq!(tile.palette_line, 0);
                assert!(
                    tile.priority,
                    "ASM loc_14E46 writes priority tile word `$8380`"
                );
            }
            _ => unreachable!("unknown plane command matched IndexedTile above"),
        }
        match unknown_tiles[41] {
            DrawCommand::IndexedTile { x, y, tile, .. } => {
                assert_eq!((*x, *y), (0, 8));
                assert_eq!(
                    tile.tile_index, 4,
                    "ASM loc_14E44 adds `$0004_0004` before the second plane row"
                );
            }
            _ => unreachable!("unknown plane command matched IndexedTile above"),
        }
    }

    #[test]
    fn build_game_scene_keeps_no_scroll_background_layers_screen_fixed() {
        let mut trace = sample_trace(GameMode::InGame);
        trace.camera_x_pos = 0x0020_0000;
        trace.camera_y_pos = 0x0010_0000;
        let mut visuals = sample_level_visuals();
        visuals.level.background_scroll = Some(wildside_data::BgscrollProfile {
            layer_speeds: vec![0x10; 8],
            has_terminator: true,
        });

        let scene = build_scene(&trace, false, Some(&visuals), None);

        assert!(indexed_tile_at(&scene, 0, 0).is_some());
    }

    #[test]
    fn level_complete_handoff_keeps_flag_touch_background_for_first_frame_loc_c070() {
        let mut trace = sample_trace(GameMode::InGame);
        trace.camera_x_pos = 0;
        trace.camera_y_pos = 0;
        let visuals = sample_level_visuals();
        let filter = LayerFilter::solo("background");

        let gameplay_scene = build_scene_filtered(&trace, false, Some(&visuals), None, filter);
        assert!(
            gameplay_scene.commands.iter().any(|command| {
                matches!(
                    command,
                    DrawCommand::IndexedTile { .. } | DrawCommand::IndexedTileRow { .. }
                )
            }),
            "ordinary gameplay should render Scroll B level background tiles"
        );

        trace.level_complete = true;
        trace.level_complete_elapsed_frames = 0;
        let handoff_scene = build_scene_filtered(&trace, false, Some(&visuals), None, filter);
        assert!(
            handoff_scene.commands.iter().any(|command| {
                matches!(
                    command,
                    DrawCommand::IndexedTile { .. } | DrawCommand::IndexedTileRow { .. }
                )
            }),
            "ASM loc_C070 begins the flagpole hand-off, but oracle pixels at native elapsed=0 still show the previous Scroll B frame"
        );

        trace.level_complete_elapsed_frames = 1;
        let pre_hologram_scene = build_scene_filtered(&trace, false, Some(&visuals), None, filter);
        assert!(
            !pre_hologram_scene.commands.iter().any(|command| {
                matches!(
                    command,
                    DrawCommand::IndexedTile { .. } | DrawCommand::IndexedTileRow { .. }
                )
            }),
            "later loc_C070/loc_C0CC hand-off frames should not keep repeating ordinary Scroll B before loc_C186 draws the EOL room"
        );
    }

    #[test]
    fn loc_c070_first_render_frame_uses_previous_vdp_latch() {
        let mut previous = sample_trace(GameMode::InGame);
        previous.camera_x_pos = 0x055D_0000;
        previous.camera_y_pos = 0x0173_0000;
        previous.frame_index = 719;
        previous.level_timer_seconds = 179;
        previous.active_objects = vec![NativeObjectSnapshot {
            kind: NativeObjectKind::Player,
            x_pos: 0x0558_0000,
            y_pos: 0x0140_0000,
            layout_index: None,
            x_velocity: Some(0),
            y_velocity: Some(0),
            facing: Some(Direction::Right),
            on_ground: Some(true),
            hit_points: None,
            damage_flash: false,
            palette_variant: 0,
            stationary: false,
            contact_phase: None,
            shared_walker_state: None,
            shared_walker_patrol_timer: None,
            shared_walker_patrol_short_next: None,
            walk_phase: None,
            wait_timer: None,
            debug_fields: std::collections::BTreeMap::new(),
        }];
        let mut trace = previous.clone();
        trace.camera_x_pos = 0x0563_0000;
        trace.camera_y_pos = 0x0171_0000;
        trace.frame_index = 720;
        trace.level_timer_seconds = 178;
        trace.active_objects.clear();
        trace.level_complete = true;
        trace.level_complete_elapsed_frames = 0;

        let render_trace = render_trace_with_vdp_latched_state(&trace, Some(&previous));

        assert_eq!(
            (
                render_trace.camera_x_pos,
                render_trace.camera_y_pos,
                render_trace.frame_index,
                render_trace.level_timer_seconds,
                render_trace.active_objects.len()
            ),
            (
                previous.camera_x_pos,
                previous.camera_y_pos,
                previous.frame_index,
                previous.level_timer_seconds,
                previous.active_objects.len()
            ),
            "ASM loc_C070 switches to LevelComplete after the frame's VDP transfer, so the first visible hand-off frame must keep the previous scroll, sprite, and HUD latches"
        );

        trace.level_complete_elapsed_frames = 1;
        let later_render_trace = render_trace_with_vdp_latched_state(&trace, Some(&previous));
        assert_eq!(
            (
                later_render_trace.camera_x_pos,
                later_render_trace.camera_y_pos
            ),
            (trace.camera_x_pos, trace.camera_y_pos),
            "only the first loc_C070 hand-off frame inherits the previous scroll latch"
        );
    }

    #[test]
    fn gameplay_render_uses_previous_scroll_latch_loc_300ba() {
        let mut previous = sample_trace(GameMode::InGame);
        previous.current_level = LevelId(0x02);
        previous.camera_x_pos = 0x0010_0000;
        previous.camera_y_pos = 0x05BC_0000;
        previous.kid_x_pos = Some(0x0040_0000);
        previous.kid_y_pos = Some(0x0660_0000);

        let mut trace = previous.clone();
        trace.camera_x_pos = 0x0011_0000;
        trace.camera_y_pos = 0x05BD_0000;
        trace.kid_x_pos = Some(0x0042_0000);
        trace.kid_y_pos = Some(0x0664_0000);

        let render_trace = render_trace_with_vdp_latched_state(&trace, Some(&previous));

        assert_eq!(
            (render_trace.camera_x_pos, render_trace.camera_y_pos),
            (previous.camera_x_pos, previous.camera_y_pos),
            "ASM Transfer_ScrollDataToVRAM loc_300BA writes VDP scroll before the post-update gameplay trace state is visible"
        );
        assert_eq!(
            (render_trace.kid_x_pos, render_trace.kid_y_pos),
            (previous.kid_x_pos, previous.kid_y_pos),
            "ASM Mode_Level transfers Kid's previous sprite state before Execute_Objects advances the post-update gameplay trace"
        );
    }

    #[test]
    fn gameplay_render_uses_previous_gfxobject_sprite_latch_mode_level_loc_901() {
        let mut previous = sample_trace(GameMode::InGame);
        previous.current_level = LevelId(0x06);
        previous.camera_x_pos = 0x0010_0000;
        previous.camera_y_pos = 0x0040_0000;
        previous.active_objects = vec![
            NativeObjectSnapshot {
                kind: NativeObjectKind::Player,
                x_pos: 0x0020_0000,
                y_pos: 0x0080_0000,
                layout_index: None,
                x_velocity: Some(0),
                y_velocity: Some(0),
                facing: Some(Direction::Right),
                on_ground: Some(true),
                hit_points: None,
                damage_flash: false,
                palette_variant: 0,
                stationary: false,
                contact_phase: None,
                shared_walker_state: None,
                shared_walker_patrol_timer: None,
                shared_walker_patrol_short_next: None,
                walk_phase: None,
                wait_timer: None,
                debug_fields: std::collections::BTreeMap::new(),
            },
            NativeObjectSnapshot {
                kind: NativeObjectKind::Enemy { object_type: 0x0C },
                x_pos: 0x0044_0000,
                y_pos: 0x006D_0000,
                layout_index: Some(0),
                x_velocity: Some(-0x0002_0000),
                y_velocity: Some(-0x0003_0000),
                facing: Some(Direction::Left),
                on_ground: Some(false),
                hit_points: Some(1),
                damage_flash: false,
                palette_variant: 0,
                stationary: false,
                contact_phase: None,
                shared_walker_state: None,
                shared_walker_patrol_timer: None,
                shared_walker_patrol_short_next: None,
                walk_phase: None,
                wait_timer: None,
                debug_fields: std::collections::BTreeMap::new(),
            },
        ];
        previous.active_projectiles = vec![(0x004C_0000, 0x00A1_0000, false)];

        let mut trace = previous.clone();
        trace.frame_index = previous.frame_index + 1;
        trace.kid_x_pos = Some(0x0021_0000);
        trace.active_objects[0].x_pos = 0x0021_0000;
        trace.active_objects[1].x_pos = 0x0042_0000;
        trace.active_objects[1].y_pos = 0x006B_0000;
        trace.active_projectiles = vec![(0x004B_0000, 0x00A4_0000, false)];

        let render_trace = render_trace_with_vdp_latched_state(&trace, Some(&previous));
        let rendered_player = render_trace
            .active_objects
            .iter()
            .find(|object| object.kind == NativeObjectKind::Player)
            .expect("previous Kid sprite source should be retained");
        let rendered_dragon = render_trace
            .active_objects
            .iter()
            .find(|object| matches!(object.kind, NativeObjectKind::Enemy { object_type: 0x0C }))
            .expect("previous Dragon sprite source should be retained");

        assert_eq!(
            rendered_player.x_pos, previous.active_objects[0].x_pos,
            "ASM Mode_Level loc_901 transfers Sprite_Table and Kid art before Execute_Objects/GfxObjects_MoveAndAnimate, so Kid pixels display the previous trace"
        );
        assert_eq!(
            rendered_dragon.x_pos, previous.active_objects[1].x_pos,
            "ASM Mode_Level loc_901 transfers Sprite_Table before Execute_Objects/GfxObjects_MoveAndAnimate, so enemy GfxObject pixels display the previous trace"
        );
        assert_eq!(
            render_trace.active_projectiles, previous.active_projectiles,
            "ASM Mode_Level loc_901 presents the previous non-Kid sprite table for projectile workers as well"
        );
    }

    #[test]
    fn gameplay_render_uses_previous_hitpoint_hud_latch_loc_bdc0() {
        let mut previous = sample_trace(GameMode::InGame);
        previous.current_level = LevelId(0x00);
        previous.number_hitpoints = 5;
        previous.kid_x_pos = Some(0x00A5_0000);

        let mut trace = previous.clone();
        trace.number_hitpoints = 4;
        trace.kid_x_pos = Some(0x00A6_0000);

        let render_trace = render_trace_with_vdp_latched_state(&trace, Some(&previous));

        assert_eq!(
            render_trace.number_hitpoints, previous.number_hitpoints,
            "ASM Mode_Level calls Make_SpriteAttr_HUD/sub_BDC0 before Check_for_recent_damge loc_B700, so a same-frame hit still renders the previous HP pips"
        );
        assert_eq!(
            render_trace.kid_x_pos, previous.kid_x_pos,
            "ASM Mode_Level loc_901 transfers Kid's visible sprite state before the post-update gameplay trace"
        );

        let mut next = trace.clone();
        next.kid_x_pos = Some(0x00A7_0000);
        let next_render_trace = render_trace_with_vdp_latched_state(&next, Some(&trace));

        assert_eq!(
            next_render_trace.number_hitpoints, trace.number_hitpoints,
            "the loc_BDC0 HUD latch is a one-frame ordering quirk, not a persistent stale HP display"
        );
    }

    #[test]
    fn gameplay_render_uses_previous_hud_diamond_page_latch_loc_45ee() {
        let mut previous = sample_trace(GameMode::DemoPlay);
        previous.current_level = LevelId(0x00);
        previous.hud_diamond_frame_offset = Some(4);
        previous.kid_x_pos = Some(0x004A_0000);

        let mut trace = previous.clone();
        trace.hud_diamond_frame_offset = Some(6);
        trace.kid_x_pos = Some(0x004B_0000);

        let render_trace = render_trace_with_vdp_latched_state(&trace, Some(&previous));

        assert_eq!(
            render_trace.hud_diamond_frame_offset, previous.hud_diamond_frame_offset,
            "ASM sub_44DC loc_45EE advances the diamond DMA page before Make_SpriteAttr_HUD/sub_BDC0's next visible symbol latch, so render keeps the previous HUD diamond page"
        );
        assert_eq!(
            render_trace.kid_x_pos, previous.kid_x_pos,
            "ASM Mode_Level loc_901 transfers Kid's visible sprite state before the post-update gameplay trace"
        );

        let mut next = trace.clone();
        next.hud_diamond_frame_offset = Some(0);
        let next_render_trace = render_trace_with_vdp_latched_state(&next, Some(&trace));

        assert_eq!(
            next_render_trace.hud_diamond_frame_offset, trace.hud_diamond_frame_offset,
            "the loc_45EE HUD diamond page latch advances one trace at a time"
        );
    }

    #[test]
    fn transform_hibernate_render_uses_prior_hud_diamond_page_loc_45ee() {
        let mut previous = sample_trace(GameMode::DemoPlay);
        previous.current_level = LevelId(0x00);
        previous.hud_diamond_frame_offset = Some(6);
        previous.form_change_flash_frames = 72;

        let mut trace = previous.clone();
        trace.hud_diamond_frame_offset = Some(0);
        trace.form_change_flash_frames = 71;

        let render_trace = render_trace_with_vdp_latched_state(&trace, Some(&previous));

        assert_eq!(
            render_trace.hud_diamond_frame_offset,
            Some(4),
            "ASM Kid_Transform loc_7CD0 hibernates gameplay while Mode_Level loc_80C still reaches sub_44DC; oracle frames keep the visible HUD diamond one unk_461C page behind the post-update previous trace"
        );
    }

    #[test]
    fn first_transform_frame_latches_previous_player_visual_state_loc_7c40() {
        let mut previous = sample_trace(GameMode::DemoPlay);
        previous.current_level = LevelId(0x00);
        previous.form_change_flash_frames = 0;
        previous.current_helmet_id = PlayerForm::TheKid.helmet_id();
        previous.active_objects = vec![NativeObjectSnapshot {
            kind: NativeObjectKind::Player,
            x_pos: 0x004A_0000,
            y_pos: 0x0290_0000,
            layout_index: None,
            x_velocity: None,
            y_velocity: None,
            facing: None,
            on_ground: None,
            hit_points: None,
            damage_flash: false,
            palette_variant: 0,
            stationary: false,
            contact_phase: None,
            shared_walker_state: None,
            shared_walker_patrol_timer: None,
            shared_walker_patrol_short_next: None,
            walk_phase: None,
            wait_timer: None,
            debug_fields: std::collections::BTreeMap::new(),
        }];

        let mut trace = previous.clone();
        trace.current_helmet_id = PlayerForm::Cyclone.helmet_id();
        trace.form_change_flash_frames = 73;
        trace.active_objects[0].x_pos = 0x0048_0000;

        let render_trace = render_trace_with_vdp_latched_state(&trace, Some(&previous));

        assert_eq!(
            render_trace.current_helmet_id,
            PlayerForm::TheKid.helmet_id(),
            "ASM Kid_Transform loc_7C40 starts after sprite attributes are built, so the entry frame still renders the previous form"
        );
        assert_eq!(
            render_trace.form_change_flash_frames, 0,
            "ASM Kid_Transform loc_7C40 entry should not expose the forward transform table until the next visible hibernate"
        );
        assert_eq!(
            render_trace.active_objects[0].x_pos, previous.active_objects[0].x_pos,
            "the entry frame also keeps Kid's previous GFX-object position"
        );

        let mut next = trace.clone();
        next.form_change_flash_frames = 72;
        let next_render_trace = render_trace_with_vdp_latched_state(&next, Some(&trace));
        assert_eq!(
            (
                next_render_trace.current_helmet_id,
                next_render_trace.form_change_flash_frames,
                next_render_trace.active_objects[0].x_pos
            ),
            (
                next.current_helmet_id,
                next.form_change_flash_frames,
                next.active_objects[0].x_pos
            ),
            "only the first Kid_Transform entry frame keeps the previous player visual state"
        );
    }

    #[test]
    fn transform_hibernate_suppresses_hp_pips_loc_bdc0() {
        let mut trace = sample_trace(GameMode::DemoPlay);
        trace.number_hitpoints = 3;
        trace.current_helmet_id = PlayerForm::Cyclone.helmet_id();
        let visuals = sample_level_visuals();
        let transform_frames = visuals
            .player
            .forms
            .get(&PlayerForm::Cyclone.helmet_id())
            .expect("sample Cyclone visuals")
            .transform_steps
            .iter()
            .map(|step| step.duration_frames)
            .sum::<usize>() as u8;

        trace.form_change_flash_frames = transform_frames;
        let first_hibernate_scene = build_scene(&trace, false, Some(&visuals), None);

        assert!(
            first_hibernate_scene.commands.iter().any(|command| {
                matches!(
                    command,
                    DrawCommand::IndexedTile { tile_sheet, .. }
                        if tile_sheet.as_ref() == &visuals.hud.hitpoint_sheet
                )
            }),
            "ASM Mode_Level transfers the previous Sprite_Table before Make_SpriteAttr_HUD loc_BDC0 skips HP pips, so the first hibernated Cyclone frame still shows them"
        );

        trace.form_change_flash_frames = transform_frames - 1;
        let scene = build_scene(&trace, false, Some(&visuals), None);

        assert!(
            !scene.commands.iter().any(|command| {
                matches!(
                    command,
                    DrawCommand::IndexedTile { tile_sheet, .. }
                        if tile_sheet.as_ref() == &visuals.hud.hitpoint_sheet
                )
            }),
            "ASM Make_SpriteAttr_HUD loc_BDC0 branches to loc_BE82 while Currently_transforming is set, skipping the HP pip sprites"
        );
        assert!(
            scene.commands.iter().any(|command| {
                matches!(
                    command,
                    DrawCommand::IndexedTile { tile_sheet, .. }
                        if tile_sheet.tiles.len() == visuals.hud.digits.tiles.len()
                )
            }),
            "ASM Make_SpriteAttr_HUD loc_BE82 continues through the timer/lives/diamond HUD paths after skipping HP pips"
        );
    }

    #[test]
    fn forest_parallax_uses_asm_scroll_divisor() {
        let trace = sample_trace(GameMode::InGame);
        let camera_x_px = 307;

        assert_eq!(
            super::scene::background_scroll_x_px(&trace, camera_x_px, 0x30),
            76
        );
        assert_eq!(
            super::scene::background_scroll_x_px(&trace, camera_x_px, 0x50),
            153
        );
    }

    #[test]
    fn layered_background_rows_wrap_like_plane_b_loc_11b7a() {
        assert_eq!(
            super::scene::wrap_layered_background_x_loc_11b7a(-520, 512),
            -8,
            "ASM loc_11B7A backs Plane B row pointers; native must wrap the 64-tile forest row instead of letting it disappear offscreen",
        );
        assert_eq!(
            super::scene::wrap_layered_background_x_loc_11b7a(-512, 512),
            0,
            "one full row of parallax scroll should return to the first background tile",
        );
        assert_eq!(
            super::scene::wrap_layered_background_x_loc_11b7a(400, 512),
            400,
            "ASM loc_11B7A wraps at the 64-tile Plane B row, not at the 320px viewport edge",
        );
        assert_eq!(
            super::scene::wrap_layered_background_x_loc_11b7a(520, 512),
            8,
            "tiles that pass the full 512px row should wrap back into the visible Plane B range",
        );
        assert_eq!(
            super::scene::wrap_layered_background_y_loc_11b7a(-400, 392),
            -8,
            "ASM loc_11B7A Plane B rows wrap vertically too; Highwater's deep Camera_Y_pos must not scroll all Mountain rows out of view",
        );
        assert_eq!(
            super::scene::wrap_layered_background_y_loc_11b7a(392, 392),
            0,
            "one full Plane B row-table height should return to the first composed background row",
        );
    }

    #[test]
    #[ignore = "requires generated .cache assets; run mise run extract-assets first"]
    fn highwater_layered_background_rows_wrap_vertically_like_plane_b_loc_11b7a() {
        let Some(manifest) = cached_generated_manifest() else {
            return;
        };
        let visuals = load_level_visuals(&manifest, 0x02).expect("Highwater Pass visuals");
        let mut trace = sample_trace(GameMode::InGame);
        trace.current_level = LevelId(0x02);
        trace.frame_index = 72;
        trace.camera_x_pos = 0;
        trace.camera_y_pos = 0x05DC_0000;
        trace.level_skip_cheat_enabled = 1;

        let sample = super::scene::sample_background_pixel_loc_c070(&trace, &visuals, 102, 80);

        assert_eq!(
            sample.map(|sample| sample.rgb),
            Some([0x6D, 0x49, 0xB6]),
            "ASM loc_11B7A/BackgroundScroll_ComputeShiftLayers keeps Mountain Plane B rows visible behind Highwater foreground holes, and loc_12910/loc_11E16 resolve palette-0 slot 11 through the Mountain background CRAM band"
        );
    }

    #[test]
    fn layered_background_wraps_moving_plane_b_rows_loc_3050a() {
        let mut visuals = sample_level_visuals();
        visuals.level.background_theme_id = 9;
        visuals.level.background_scroll = Some(wildside_data::BgscrollProfile {
            layer_speeds: vec![0x00, 0x08, 0x10, 0x30],
            has_terminator: true,
        });

        assert!(
            !super::scene::layered_background_row_wraps_loc_3050a(&visuals, 0),
            "ASM loc_3050A keeps `$00` rows on the finite path; the captured seam comes from moving nonzero rows"
        );
        assert!(
            super::scene::layered_background_row_wraps_loc_3050a(&visuals, 1),
            "ASM loc_3050A feeds `$08` rows into the horizontally wrapping Plane B nametable; this is not Hills-only"
        );
        assert!(
            !super::scene::layered_background_row_wraps_loc_3050a(&visuals, 2),
            "ASM loc_3050A no-scroll `$10` rows stay static and do not need native row repetition"
        );
        assert!(
            super::scene::layered_background_row_wraps_loc_3050a(&visuals, 3),
            "ASM loc_3050A camera-parallax rows also repeat so long-travel rows do not expose the backdrop"
        );
    }

    #[test]
    fn hills_auto_scroll_rows_wrap_plane_b_loc_3050a() {
        let mut visuals = sample_level_visuals();
        visuals.level.background_theme_id = 3;
        visuals.level.background_scroll = Some(wildside_data::BgscrollProfile {
            layer_speeds: vec![0x00, 0x08, 0x10],
            has_terminator: true,
        });

        assert!(
            !super::scene::layered_background_row_wraps_loc_3050a(&visuals, 0),
            "this sample keeps Hills `$00` rows on an explicit finite path; nonzero `$08` rows are the captured Level 0x08 seam"
        );
        assert!(
            super::scene::layered_background_row_wraps_loc_3050a(&visuals, 1),
            "ASM loc_3050A feeds Hills `$08` rows into the horizontally wrapping Plane B row, avoiding backdrop exposure in Level 0x08"
        );
        assert!(
            !super::scene::layered_background_row_wraps_loc_3050a(&visuals, 2),
            "ASM loc_3050A no-scroll `$10` rows do not need native row repetition"
        );
    }

    #[test]
    #[ignore = "requires generated .cache assets; run mise run extract-assets first"]
    fn level08_hills_auto_scroll_wrap_fills_frame151_sky_hole_loc_3050a() {
        let Some(manifest) = cached_generated_manifest() else {
            return;
        };
        let visuals = load_level_visuals(&manifest, 0x08).expect("Hills of the Warrior 1 visuals");
        let scenario = crate::scenarios::idle_level_scenario(0x08, Some(170));
        let frames = wildside_core::run_script_with_warp(
            manifest,
            &scenario.script,
            scenario.frame_count,
            scenario.warp_to_level,
        );
        let trace = frames.get(151).expect("frame 151");

        let sample = super::scene::sample_background_pixel_loc_c070(trace, &visuals, 96, 32)
            .expect("ASM loc_3050A Hills `$08` row should wrap through Plane B at frame 151");

        assert_eq!(
            sample.rgb,
            [0x24, 0x49, 0xFF],
            "oracle frame 151 shows the wrapped Hills sky pixel here; exposing Palette_Buffer slot 0 would leave the dark-green backdrop visible"
        );
    }

    #[test]
    fn layered_background_wrap_stops_during_flagpole_handoff_loc_c070() {
        let mut visuals = sample_level_visuals();
        visuals.level.background_scroll = Some(wildside_data::BgscrollProfile {
            layer_speeds: vec![0x30],
            has_terminator: true,
        });
        let mut trace = sample_trace(GameMode::InGame);

        assert!(
            super::scene::layered_background_row_wraps_for_trace_loc_3050a(&trace, &visuals, 0),
            "ordinary gameplay should keep camera-parallax row repetition for long forest levels"
        );

        trace.level_complete = true;
        trace.level_complete_elapsed_frames = 0;
        assert!(
            super::scene::layered_background_row_wraps_for_trace_loc_3050a(&trace, &visuals, 0),
            "oracle flag-touch pixels still show wrapped forest Scroll B rows at elapsed=0"
        );

        trace.level_complete_elapsed_frames = 1;
        assert!(
            !super::scene::layered_background_row_wraps_for_trace_loc_3050a(&trace, &visuals, 0),
            "later pre-hologram flagpole frames stop repeating forest Scroll B rows"
        );
    }

    #[test]
    fn level_complete_handoff_keeps_flag_sprites_until_cleanup_loc_c070_c0b6() {
        let mut trace = sample_trace(GameMode::InGame);
        trace.camera_x_pos = 0x0730_0000;
        trace.camera_y_pos = 0x02D0_0000;
        trace.level_complete = true;

        for elapsed in [0, 1, 35, 58] {
            trace.level_complete_elapsed_frames = elapsed;
            let scene = build_ingame_scene(&trace, false, None);

            assert!(
                scene.commands.iter().any(|command| {
                    matches!(
                        command,
                        DrawCommand::Sprite { x: 26, y: 16, tile_sheet, .. }
                            if tile_sheet.tiles.len() == 120
                    )
                }),
                "ASM loc_C070/loc_C0B6 keeps stale flag gfx visible until the oracle pre-frame sprite cleanup boundary; waving flag missing at elapsed={elapsed}"
            );
            assert!(
                scene.commands.iter().any(|command| {
                    matches!(
                        command,
                        DrawCommand::Sprite { x: 24, y: 48, tile_sheet, .. }
                            if tile_sheet.tiles.len() == 4
                    )
                }),
                "ASM Flagpole_Boss pole-bottom sprite remains visible until the loc_C0B6 oracle pre-frame sprite cleanup boundary; elapsed={elapsed}"
            );
        }

        trace.level_complete_elapsed_frames =
            wildside_core::TOUCH_FLAGPOLE_GFX_CLEANUP_VISIBLE_FRAME_LOC_C0B6_ORACLE - 1;
        let scene = build_ingame_scene(&trace, false, None);
        assert!(
            !scene.commands.iter().any(|command| {
                matches!(
                    command,
                    DrawCommand::Sprite { x: 24, y: 48, tile_sheet, .. }
                        if tile_sheet.tiles.len() == 4
                )
            }),
            "ASM loc_C0B6 deletes non-Kid gfx slots from the pre-frame rendered sprite table before the trace count drops"
        );
    }

    #[test]
    fn highwater_floor_contact_keeps_flag_base_until_cleanup_loc_c070_c0b6() {
        let Some(manifest) = cached_generated_manifest() else {
            return;
        };
        let (visuals, block_grid) =
            load_session_assets(&manifest, 0x02).expect("Highwater Pass visuals");
        let mut scenario = crate::scenarios::highwater_flag_floor_contact_audit_scenario();
        scenario.frame_count = 80;

        let trace = crate::scenarios::run_native_dynamic_scenario(
            manifest,
            &scenario,
            crate::scenarios::NativeDynamicRunOptions::default(),
        )
        .expect("native Highwater flag floor-contact route");

        let touch_index = trace
            .iter()
            .position(|frame| frame.level_complete)
            .expect("floor-contact route should touch the flag");
        assert_eq!(
            touch_index, 15,
            "ASM flag floor-contact route should enter CharacterCollision_TouchFlagpole loc_C070 after walking into the real Highwater flag"
        );

        for route_frame_index in [15, 16, 73] {
            let frame = trace.get(route_frame_index).expect("route frame");
            assert_eq!(
                frame.level_complete_elapsed_frames,
                route_frame_index - touch_index,
                "loc_C070 elapsed clock should stay tied to the floor-contact flag touch"
            );
            let scene = build_scene(frame, false, Some(&visuals), Some(&block_grid));
            assert!(
                scene.commands.iter().any(|command| {
                    matches!(
                        command,
                        DrawCommand::Sprite { tile_sheet, tiles, .. }
                            if tile_sheet.tiles.len() == 120
                                && tiles.iter().all(|tile| tile.tile.priority)
                    )
                }),
                "ASM Flagpole_Boss LnkTo_unk_E0FFE keeps the high-priority waving flag visible until loc_C0B6 cleanup; route_frame={route_frame_index}"
            );
            assert!(
                scene.commands.iter().any(|command| {
                    matches!(
                        command,
                        DrawCommand::Sprite { tile_sheet, tiles, .. }
                            if tile_sheet.tiles.len() == 4
                                && tiles.iter().all(|tile| tile.tile.priority)
                    )
                }),
                "ASM Flagpole_Boss LnkTo_unk_E0FDE keeps high-priority Flag_bottom.bin visible until loc_C0B6 cleanup; route_frame={route_frame_index}"
            );
        }

        let cleanup_frame = trace.get(74).expect("cleanup frame");
        assert_eq!(
            cleanup_frame.level_complete_elapsed_frames,
            wildside_core::TOUCH_FLAGPOLE_GFX_CLEANUP_VISIBLE_FRAME_LOC_C0B6_ORACLE - 1,
            "render cleanup happens on the pre-frame sprite-table boundary before the WRAM gfx count drops"
        );
        let scene = build_scene(cleanup_frame, false, Some(&visuals), Some(&block_grid));
        assert!(
            !scene.commands.iter().any(|command| {
                matches!(
                    command,
                    DrawCommand::Sprite { tile_sheet, .. } if tile_sheet.tiles.len() == 4
                )
            }),
            "ASM loc_C0B6 deletes non-Kid flag gfx slots at the oracle-rendered cleanup boundary"
        );
    }

    #[test]
    fn flagpole_sprites_keep_rom_priority_over_foreground_loc_bfd6() {
        let mut trace = sample_trace(GameMode::InGame);
        trace.camera_x_pos = 0x0748_0000;
        trace.camera_y_pos = 0x02C0_0000;
        trace.level_complete = true;
        trace.level_complete_elapsed_frames = 1;

        let mut visuals = sample_level_visuals();
        visuals.foreground_tiles.tiles[0].pixels = [1; 64];
        visuals.palettes.foreground = Palette {
            colours: vec![colour(20, 30, 40), colour(180, 190, 200)],
        };
        for tile in &mut visuals.flag.pole_bottom.tile_sheet.tiles {
            tile.pixels = [1; 64];
        }
        visuals.flag.palette = Palette {
            colours: vec![colour(230, 40, 20)],
        };

        let mut priority_layout = [0; 5 * 2];
        priority_layout[4 * 2] = 0x01;
        let priority_grid = BlockGrid::from_level_layout(&priority_layout, 1, 5);
        let scene = build_scene(&trace, false, Some(&visuals), Some(&priority_grid));
        let sample =
            sample_scene_pixel_provenance(&scene, &[(0usize, 64usize)].into_iter().collect())
                .into_iter()
                .next()
                .expect("flag base sample");

        assert!(
            sample.final_high_priority,
            "ASM Flagpole_Boss loc_BFD6 sets `priority(a3)=1` on both flag gfx objects"
        );
        assert!(
            sample.writes.iter().any(|write| write.kind == "sprite_tile"
                && write.rgb == Some([230, 40, 20])
                && write.priority == Some(true)
                && write.accepted),
            "ASM Flagpole_Boss loc_BFD6/loc_C00A priority keeps Flag_bottom.bin visible over high-priority foreground cells"
        );
    }

    #[test]
    fn level_complete_handoff_keeps_kid_above_priority_terrain_for_first_frame_loc_c070() {
        let mut trace = sample_trace(GameMode::InGame);
        trace.level_complete = true;
        trace.level_complete_elapsed_frames = 0;
        trace.active_objects = vec![NativeObjectSnapshot {
            kind: NativeObjectKind::Player,
            x_pos: trace.kid_x_pos.expect("sample trace should expose Kid x"),
            y_pos: trace.kid_y_pos.expect("sample trace should expose Kid y"),
            layout_index: None,
            x_velocity: trace.kid_x_velocity,
            y_velocity: trace.kid_y_velocity,
            facing: trace.kid_facing,
            on_ground: trace.kid_on_ground,
            hit_points: None,
            damage_flash: false,
            palette_variant: 0,
            stationary: true,
            contact_phase: None,
            shared_walker_state: None,
            shared_walker_patrol_timer: None,
            shared_walker_patrol_short_next: None,
            walk_phase: None,
            wait_timer: None,
            debug_fields: std::collections::BTreeMap::new(),
        }];
        let visuals = sample_level_visuals();
        let filter = LayerFilter::solo("player");

        let handoff_scene = build_scene_filtered(&trace, false, Some(&visuals), None, filter);
        let player_sprite = handoff_scene
            .commands
            .iter()
            .find(|command| matches!(command, DrawCommand::Sprite { .. }))
            .expect("player layer should render Kid during the loc_C070 hand-off");
        assert!(
            sprite_tiles(player_sprite)
                .iter()
                .all(|tile| tile.tile.priority),
            "ASM CharacterCollision_TouchFlagpole loc_C070 preserves Kid on the flag-touch frame; native must not let high-priority foreground mask him"
        );

        trace.level_complete_elapsed_frames = 1;
        let later_scene = build_scene_filtered(&trace, false, Some(&visuals), None, filter);
        let later_player_sprite = later_scene
            .commands
            .iter()
            .find(|command| matches!(command, DrawCommand::Sprite { .. }))
            .expect("player layer should still render Kid before the EOL hologram is visible");
        assert!(
            sprite_tiles(later_player_sprite)
                .iter()
                .all(|tile| !tile.tile.priority),
            "only the preserved elapsed=0 loc_C070 hand-off frame needs priority override"
        );
    }

    #[test]
    fn forest_ripple_starts_from_asm_screen_line_window() {
        assert_eq!(
            super::scene::forest_water_ripple_offset_for_screen_y_sub_30598(0, 135, 0x11, 0),
            None
        );
        assert_eq!(
            super::scene::forest_water_ripple_offset_for_screen_y_sub_30598(0, 136, 0x11, 0),
            Some(0)
        );
        assert_eq!(
            super::scene::forest_water_ripple_offset_for_screen_y_sub_30598(0, 140, 0x11, 0),
            Some(1)
        );
    }

    #[test]
    fn forest_ripple_scrolls_partial_top_window_like_asm() {
        assert_eq!(
            super::scene::forest_water_ripple_offset_for_screen_y_sub_30598(0, 0, 0x00, 28),
            Some(2)
        );
        assert_eq!(
            super::scene::forest_water_ripple_offset_for_screen_y_sub_30598(0, 57, 0x00, 28),
            None
        );
    }

    #[test]
    fn forest_ripple_uses_rom_time_frames_for_settled_warp_loc_305e6() {
        let mut trace = sample_trace(GameMode::InGame);
        trace.frame_index = 44;
        trace.level_skip_cheat_enabled = 1;

        assert_eq!(
            super::scene::forest_water_ripple_time_frames_for_trace_loc_305e6(&trace),
            96,
            "ASM loc_305E6 uses global Time_Frames; settled warp recordings keep the ROM level-skip visual phase instead of the rebased trace clock"
        );
        assert_eq!(
            super::scene::forest_water_ripple_offset_for_screen_y_sub_30598(96, 125, 0x1B, 369),
            Some(6),
            "issue-175 oracle hB=$160 at y=125 requires the loc_30598 ripple byte from the ROM Time_Frames phase"
        );
    }

    #[test]
    fn auto_scroll_rows_use_rom_time_frames_for_settled_warp_loc_3050a() {
        let mut trace = sample_trace(GameMode::InGame);
        trace.frame_index = 60;
        trace.level_skip_cheat_enabled = 1;

        assert_eq!(
            super::scene::background_scroll_x_px(&trace, 0, 0x08),
            28,
            "ASM BackgroundScroll_ComputeShiftData loc_30452 feeds `$08` auto-scroll rows from global Time_Frames; settled warp recordings keep the ROM level-skip visual phase"
        );
        assert_eq!(
            super::scene::background_scroll_x_px(&trace, 0, 0x00),
            56,
            "ASM BackgroundScroll_ComputeShiftData loc_30452 uses the same global Time_Frames basis for `$00` rows"
        );

        trace.level_skip_cheat_enabled = 0;
        assert_eq!(
            super::scene::background_scroll_x_px(&trace, 0, 0x08),
            15,
            "non-settled gameplay keeps the trace-local frame clock"
        );
    }

    #[test]
    fn auto_scroll_rows_prefer_typed_time_frames_loc_3038a() {
        let mut trace = sample_trace(GameMode::InGame);
        trace.frame_index = 60;
        trace.level_skip_cheat_enabled = 1;
        trace.time_frames_loc_3038a = 174;

        assert_eq!(
            super::scene::background_scroll_x_px(&trace, 0, 0x08),
            43,
            "ASM loc_3038A owns Time_Frames; #289 settled-warp render diagnostics must not derive loc_3050A auto-scroll timing from the hidden runtime frame_index"
        );
        assert_eq!(
            super::scene::forest_water_ripple_time_frames_for_trace_loc_305e6(&trace),
            174,
            "ASM loc_305E6 uses the same typed Time_Frames latch as BackgroundScroll_ComputeShiftData"
        );
    }

    #[test]
    fn chunked_plane_b_display_uses_horiz_scroll_data_loc_3050a() {
        let mut trace = sample_trace(GameMode::InGame);
        trace.frame_index = 60;
        trace.time_frames_loc_3038a = 174;
        let camera_x_px = 292;

        assert_eq!(
            super::scene::background_display_scroll_x_px_loc_3050a(&trace, camera_x_px, 0x08),
            122,
            "ASM loc_3050A copies `Horiz_Scroll_Data+4` into the H-scroll table; issue #289 frame 60 oracle VDP sample y=10 reports rendered hB=`FF86`, so auto-scroll display is latched one frame behind the current WRAM hbuf `FF85`"
        );
        assert_eq!(
            super::scene::background_scroll_x_px(&trace, camera_x_px, 0x08),
            43,
            "sub_30744 row-history modelling keeps the older `$08` upload phase separate from the loc_3050A display H-scroll"
        );
        assert_eq!(
            super::scene::background_display_scroll_x_px_loc_3050a(&trace, camera_x_px, 0x00),
            209,
            "ASM BackgroundScroll_ComputeShiftData loc_30452 adds Camera_X>>3 to Time_Frames for `$00` rows before loc_3050A negates the previous VDP table phase into rendered hbuf `FF2F`"
        );

        trace.time_frames_loc_3038a = 185;
        assert_eq!(
            super::scene::background_display_scroll_x_px_loc_3050a(&trace, camera_x_px, 0x00),
            220,
            "issue #289 frame 71 oracle VDP sample y=42 reports rendered hB=`FF24`; `$00` rows therefore use Time_Frames-1 for display while the current WRAM hbuf has already advanced"
        );
    }

    #[test]
    fn build_game_scene_emits_level_layout_terrain_tiles() {
        let mut trace = sample_trace(GameMode::InGame);
        trace.camera_x_pos = 0;
        trace.camera_y_pos = 0;
        let grid = BlockGrid::from_level_layout(&[0x00, 0x00, 0x00, 0x03, 0x00, 0x05], 3, 1);

        let scene = build_ingame_scene(&trace, false, Some(&grid));

        let Some((tile_sheet, tile, _)) = indexed_tile_at_with_sheet_len(&scene, 0, 0, 0x80) else {
            panic!("expected a foreground tile at the origin");
        };
        assert_eq!(tile_sheet.tiles.len(), 0x80);
        assert_eq!(tile.tile_index, 0);

        let Some((_, tile, _)) = indexed_tile_at_with_sheet_len(&scene, 16, 0, 0x80) else {
            panic!("expected a second foreground tile in the opening row");
        };
        assert_eq!(tile.tile_index, 12);

        let Some((_, tile, _)) = indexed_tile_at_with_sheet_len(&scene, 0, 16, 0x80) else {
            panic!("expected a terrain tile on the second row");
        };
        assert_eq!(tile.tile_index, 20);
    }

    #[test]
    fn build_game_scene_emits_block_family_tiles_from_skin_and_block_type() {
        let mut trace = sample_trace(GameMode::InGame);
        trace.camera_x_pos = 0;
        trace.camera_y_pos = 0;
        let data = [
            0x60, 0x00, 0xE0, 0x01, 0xE1, 0x02, 0xE2, 0x03, 0xE5, 0x04, 0xE6, 0x05, 0xE7, 0x06,
        ];
        let grid = BlockGrid::from_level_layout(&data, 7, 1);

        let scene = build_ingame_scene(&trace, false, Some(&grid));

        let expected_tiles = [
            (16, 0, 0x200, 4),
            (32, 0, 0x200, 8),
            (48, 0, 0x200, 12),
            (64, 0, 0x200, 16),
            (80, 0, 0x200, 20),
            (96, 0, 0x200, 24),
        ];
        for (x, y, expected_sheet_len, expected_tile_index) in expected_tiles {
            let Some((tile_sheet, tile, _)) =
                topmost_indexed_tile_at_with_sheet_len(&scene, x, y, expected_sheet_len)
            else {
                panic!("expected indexed block override at {x},{y}");
            };
            assert_eq!(tile_sheet.tiles.len(), expected_sheet_len);
            assert_eq!(tile.tile_index, expected_tile_index);
        }
        assert!(topmost_indexed_tile_at_with_sheet_len(&scene, 0, 0, 0x200).is_none());
    }

    #[test]
    fn foreground_layout_terrain_skips_static_live_block_cells_loc_302f6() {
        let mut trace = sample_trace(GameMode::InGame);
        trace.camera_x_pos = 0;
        trace.camera_y_pos = 0;
        // ASM sub_302F6/sub_3034A reads one Level_Layout word per Scroll-A
        // cell. Negative `$E7xx` block cells branch to Block_Mappings, so
        // transparent pixels in the block art expose Scroll A backdrop, not
        // the separate foreground byte layout. Erased `$6Fxx`/non-block cells
        // are covered by the prize-wobble tests above.
        let grid = BlockGrid::from_level_layout(&[0x00, 0x00, 0xE7, 0x01, 0x00, 0x05], 3, 1);

        let scene = build_ingame_scene(&trace, false, Some(&grid));

        assert!(
            indexed_tile_at_with_sheet_len(&scene, 16, 0, 0x80).is_none(),
            "ASM sub_302F6/sub_3034A route negative live block cells through Block_Mappings instead of drawing immutable foreground terrain underneath"
        );
        let Some((tile_sheet, tile, _)) =
            topmost_indexed_tile_at_with_sheet_len(&scene, 16, 0, 0x200)
        else {
            panic!("expected shifting-block art at the live block cell");
        };
        assert_eq!(tile_sheet.tiles.len(), 0x200);
        assert_eq!(tile.tile_index, 4);
    }

    #[test]
    fn build_game_scene_emits_hud_art_without_text_placeholders() {
        let mut trace = sample_trace(GameMode::InGame);
        trace.number_hitpoints = 1;
        trace.number_lives = 3;
        trace.diamond_count = 7;

        let scene = build_ingame_scene(&trace, false, None);
        let hud_digits = scene
            .commands
            .iter()
            .filter(|command| {
                matches!(
                    command,
                    DrawCommand::IndexedTile { y, tile_sheet, tile, .. }
                        if *y < 48
                            && tile_sheet.tiles.len() == 12
                            && tile.tile_index <= 11
                )
            })
            .count();
        let hud_hitpoints = scene
            .commands
            .iter()
            .filter(|command| {
                matches!(
                    command,
                    DrawCommand::IndexedTile { y, tile_sheet, .. }
                        if *y < 48 && tile_sheet.tiles.len() == 4
                )
            })
            .count();

        assert!(hud_digits >= 2, "expected digit tiles in the HUD");
        assert!(hud_hitpoints >= 1, "expected hitpoint art tiles in the HUD");
        assert!(
            scene.commands.iter().any(|command| {
                matches!(
                    command,
                    DrawCommand::Sprite { y, tile_sheet, tiles, .. }
                        if *y < 48
                            && tile_sheet.tiles.len() >= 16
                            && tiles.len() == 4
                            && tiles.first().is_some_and(|tile| tile.tile.tile_index == 8)
                )
            }),
            "expected the HUD diamond icon as the ASM 2x2 sprite"
        );
        assert!(!contains_text(&scene, "x3"));
        assert!(!contains_text(&scene, "D:7"));
    }

    #[test]
    fn build_effects_scene_renders_headymetal_wobble_as_sprites() {
        let mut trace = sample_trace(GameMode::InGame);
        trace.flag_x = None;
        trace.flag_y = None;
        trace.enemy_death_effects = vec![(
            100 << 16,
            120 << 16,
            wildside_core::HEADYMETAL_DEATH_EFFECT_FRAMES,
        )];
        let visuals = sample_level_visuals();

        let scene = build_scene_filtered(
            &trace,
            false,
            Some(&visuals),
            None,
            LayerFilter::solo("effects"),
        );

        let sprite_count = scene
            .commands
            .iter()
            .filter(|command| matches!(command, DrawCommand::Sprite { .. }))
            .count();
        assert_eq!(sprite_count, 3, "boss body + two eye sprites should render");
        assert!(
            !scene.commands.iter().any(|command| matches!(
                command,
                DrawCommand::Rect {
                    width: 3,
                    height: 3,
                    ..
                }
            )),
            "HeadyMetal wobble should not fall back to the generic burst rectangles"
        );
    }

    #[test]
    fn build_effects_scene_renders_kid_projectile_hit_spark_loc_1f08() {
        let mut trace = sample_trace(GameMode::InGame);
        trace.camera_x_pos = 80 << 16;
        trace.camera_y_pos = 48 << 16;
        trace.flag_x = None;
        trace.flag_y = None;
        trace.kid_projectile_hit_sparks = vec![(100, 72, 4)];
        let visuals = sample_level_visuals();

        let scene = build_scene_filtered(
            &trace,
            false,
            Some(&visuals),
            None,
            LayerFilter::solo("effects"),
        );

        assert!(
            scene.commands.iter().any(|command| {
                matches!(
                    command,
                    DrawCommand::Sprite { x, y, tiles, .. }
                        if *x == 8 && *y == 12 && tiles.len() == 9
                ) || matches!(
                    command,
                    DrawCommand::Rect {
                        x,
                        y,
                        width: 6,
                        height: 6,
                        colour: [255, 240, 96],
                    } if *x == 17 && *y == 21
                )
            }),
            "ASM loc_1F08/stru_1F40 should render the pass-0 hit spark at the helper `$44/$46` coordinates"
        );
    }

    #[test]
    fn build_effects_scene_renders_headymetal_split_worker_positions_loc_378d6() {
        let mut trace = sample_trace(GameMode::InGame);
        trace.camera_x_pos = 0;
        trace.camera_y_pos = 0;
        trace.flag_x = None;
        trace.flag_y = None;
        trace.enemy_death_effects = Vec::new();
        trace.heady_metal_death_splits = vec![wildside_core::HeadyMetalDeathSplitTrace {
            current_x_velocity_delta: -0x0002_6D18,
            left_worker_x_velocity_delta: 0x0002_6D18,
            centre_worker_y_velocity_delta: 0x0001_3880,
            centre_worker_sprite_label: Some("LnkTo_unk_C8458-Data_Index".to_owned()),
            right_worker_x_velocity_delta: -0x0002_6D18,
            far_right_worker_x_velocity_delta: 0x0002_6D18,
            cleanup_worker_slot_offsets: [0xFB72, 0xFB76, 0xFB7A, 0xFBAE, 0xFBB2],
            pieces: vec![
                wildside_core::HeadyMetalDeathPieceTrace {
                    slot_offset: 0xFB72,
                    x_pos: 100 << 16,
                    y_pos: 120 << 16,
                    x_velocity: -0x0002_6D18,
                    y_velocity: 0,
                },
                wildside_core::HeadyMetalDeathPieceTrace {
                    slot_offset: 0xFB76,
                    x_pos: 120 << 16,
                    y_pos: 124 << 16,
                    x_velocity: 0x0002_6D18,
                    y_velocity: 0,
                },
                wildside_core::HeadyMetalDeathPieceTrace {
                    slot_offset: 0xFB7A,
                    x_pos: 140 << 16,
                    y_pos: 128 << 16,
                    x_velocity: 0,
                    y_velocity: 0x0001_3880,
                },
                wildside_core::HeadyMetalDeathPieceTrace {
                    slot_offset: 0xFBAE,
                    x_pos: 160 << 16,
                    y_pos: 132 << 16,
                    x_velocity: -0x0002_6D18,
                    y_velocity: 0,
                },
                wildside_core::HeadyMetalDeathPieceTrace {
                    slot_offset: 0xFBB2,
                    x_pos: 180 << 16,
                    y_pos: 136 << 16,
                    x_velocity: 0x0002_6D18,
                    y_velocity: 0,
                },
            ],
        }];
        let visuals = sample_level_visuals();

        let scene = build_scene_filtered(
            &trace,
            false,
            Some(&visuals),
            None,
            LayerFilter::solo("effects"),
        );

        assert_eq!(
            sprite_positions(&scene, 71),
            vec![(92, 120), (112, 124), (132, 128), (152, 132), (172, 136)],
            "ASM sub_378C8 loc_378D6 mutates the five cached HeadyMetal workers; the renderer should draw those traced worker positions instead of collapsing back to the old three-sprite wobble"
        );
    }

    #[test]
    fn build_effects_scene_steps_headymetal_wobble_at_each_timer_boundary() {
        let visuals = sample_level_visuals();
        let cases = [
            // ASM Enemy20_HeadyMetal_Init seeds $40(a1) = $01C2 and the later
            // death routine steps the wobble magnitude at the remaining-frame
            // thresholds $015E, $00FA, $0096, and $0064. The quiver sizes come
            // from sub_37708/sub_3787C/sub_378A2: $30D40, $493E0, $61A80,
            // $7A120, and $927C0, truncated to whole pixels for the native
            // renderer's sprite offsets.
            (0x01C2, (88, 123), vec![(77, 102), (107, 102)]),
            (0x015E, (88, 124), vec![(76, 102), (108, 102)]),
            (0x00FA, (88, 126), vec![(74, 102), (110, 102)]),
            (0x0096, (88, 127), vec![(73, 102), (111, 102)]),
            (0x0064, (88, 129), vec![(71, 102), (113, 102)]),
        ];

        for (remaining, expected_body, expected_eyes) in cases {
            let mut trace = sample_trace(GameMode::InGame);
            trace.frame_index = 0;
            trace.camera_x_pos = 0;
            trace.camera_y_pos = 0;
            trace.flag_x = None;
            trace.flag_y = None;
            trace.enemy_death_effects = vec![(100 << 16, 120 << 16, remaining)];

            let scene = build_scene_filtered(
                &trace,
                false,
                Some(&visuals),
                None,
                LayerFilter::solo("effects"),
            );

            assert_eq!(
                sprite_position(&scene, 61),
                Some(expected_body),
                "body wobble should match remaining timer 0x{remaining:04X}",
            );
            assert_eq!(
                sprite_positions(&scene, 71),
                expected_eyes,
                "eye wobble should match remaining timer 0x{remaining:04X}",
            );
        }
    }

    #[test]
    fn render_effects_scene_draws_headymetal_wobble_pixels_at_each_timer_boundary() {
        let mut visuals = sample_level_visuals();
        visuals.enemies.families.insert(
            0x20,
            AnimatedSpriteVisual {
                palette: Palette {
                    colours: vec![colour(0, 0, 0), colour(255, 80, 80)],
                },
                extra_palettes: vec![],
                frames: vec![sample_object_frame(2, 25, 32)],
                frame_duration: 10,
            },
        );
        visuals.enemies.families.insert(
            0x21,
            AnimatedSpriteVisual {
                palette: Palette {
                    colours: vec![colour(0, 0, 0), colour(255, 220, 120)],
                },
                extra_palettes: vec![],
                frames: vec![sample_object_frame(2, 16, 16)],
                frame_duration: 6,
            },
        );
        let background = [33, 66, 99];
        let cases = [
            // ASM sub_37708/sub_3787C/sub_378A2 changes wobble magnitude as the
            // HeadyMetal death timer crosses $015E/$00FA/$0096/$0064.
            (0x01C2, (88, 123), vec![(77, 102), (107, 102)]),
            (0x015E, (88, 124), vec![(76, 102), (108, 102)]),
            (0x00FA, (88, 126), vec![(74, 102), (110, 102)]),
            (0x0096, (88, 127), vec![(73, 102), (111, 102)]),
            (0x0064, (88, 129), vec![(71, 102), (113, 102)]),
        ];

        for (remaining, expected_body, expected_eyes) in cases {
            let mut trace = sample_trace(GameMode::InGame);
            trace.frame_index = 0;
            trace.camera_x_pos = 0;
            trace.camera_y_pos = 0;
            trace.flag_x = None;
            trace.flag_y = None;
            trace.enemy_death_effects = vec![(100 << 16, 120 << 16, remaining)];

            let scene = build_scene_filtered(
                &trace,
                false,
                Some(&visuals),
                None,
                LayerFilter::solo("effects"),
            );
            let mut rgb = vec![0; (FRAME_WIDTH * FRAME_HEIGHT * 3) as usize];
            render_scene(&scene, &mut rgb);
            let body_sample = (expected_body.0 + 10, expected_body.1 + 20);

            assert_eq!(
                pixel_at(&rgb, body_sample.0 as u32, body_sample.1 as u32),
                [255, 80, 80],
                "body pixel should render at the ASM-backed wobble origin for remaining timer 0x{remaining:04X}",
            );
            for &(eye_x, eye_y) in &expected_eyes {
                assert_eq!(
                    pixel_at(&rgb, eye_x as u32, eye_y as u32),
                    [255, 220, 120],
                    "eye pixel should render at the ASM-backed wobble origin for remaining timer 0x{remaining:04X}",
                );
            }
            assert_eq!(pixel_at(&rgb, 0, 0), background);

            let body_pixels = rgb
                .chunks_exact(3)
                .filter(|pixel| *pixel == [255, 80, 80].as_slice())
                .count();
            let eye_pixels = rgb
                .chunks_exact(3)
                .filter(|pixel| *pixel == [255, 220, 120].as_slice())
                .count();
            assert_eq!(
                body_pixels,
                32 * 32,
                "body sprite footprint should stay intact at remaining timer 0x{remaining:04X}",
            );
            assert_eq!(
                eye_pixels,
                16 * 16 * 2,
                "eye sprite footprints should stay intact at remaining timer 0x{remaining:04X}",
            );
        }
    }

    #[test]
    fn render_effects_scene_flips_headymetal_wobble_phase_after_two_frames() {
        let mut visuals = sample_level_visuals();
        visuals.enemies.families.insert(
            0x20,
            AnimatedSpriteVisual {
                palette: Palette {
                    colours: vec![colour(0, 0, 0), colour(255, 80, 80)],
                },
                extra_palettes: vec![],
                frames: vec![sample_object_frame(2, 25, 32)],
                frame_duration: 10,
            },
        );
        visuals.enemies.families.insert(
            0x21,
            AnimatedSpriteVisual {
                palette: Palette {
                    colours: vec![colour(0, 0, 0), colour(255, 220, 120)],
                },
                extra_palettes: vec![],
                frames: vec![sample_object_frame(2, 16, 16)],
                frame_duration: 6,
            },
        );

        let render_rgb = |frame_index| {
            let mut trace = sample_trace(GameMode::InGame);
            trace.frame_index = frame_index;
            trace.camera_x_pos = 0;
            trace.camera_y_pos = 0;
            trace.flag_x = None;
            trace.flag_y = None;
            trace.enemy_death_effects = vec![(
                100 << 16,
                120 << 16,
                wildside_core::HEADYMETAL_DEATH_EFFECT_FRAMES,
            )];
            let scene = build_scene_filtered(
                &trace,
                false,
                Some(&visuals),
                None,
                LayerFilter::solo("effects"),
            );
            let mut rgb = vec![0; (FRAME_WIDTH * FRAME_HEIGHT * 3) as usize];
            render_scene(&scene, &mut rgb);
            rgb
        };

        let rgb_frame_0 = render_rgb(0);
        let rgb_frame_2 = render_rgb(2);

        assert_eq!(pixel_at(&rgb_frame_0, 98, 143), [255, 80, 80]);
        assert_eq!(pixel_at(&rgb_frame_0, 77, 102), [255, 220, 120]);
        assert_eq!(pixel_at(&rgb_frame_0, 107, 102), [255, 220, 120]);
        assert_eq!(pixel_at(&rgb_frame_2, 98, 137), [255, 80, 80]);
        assert_eq!(pixel_at(&rgb_frame_2, 83, 102), [255, 220, 120]);
        assert_eq!(pixel_at(&rgb_frame_2, 101, 102), [255, 220, 120]);
        assert_eq!(pixel_at(&rgb_frame_0, 98, 102), [33, 66, 99]);
        assert_eq!(pixel_at(&rgb_frame_0, 101, 102), [33, 66, 99]);
        assert_eq!(pixel_at(&rgb_frame_2, 77, 102), [33, 66, 99]);
        assert_eq!(pixel_at(&rgb_frame_2, 122, 102), [33, 66, 99]);
    }

    #[test]
    fn native_scene_builder_uses_asset_backed_commands_for_visible_level1_families() {
        let mut trace = sample_trace(GameMode::InGame);
        trace.camera_x_pos = 0;
        trace.camera_y_pos = 0;
        trace.number_lives = 2;
        trace.diamond_count = 4;
        trace.active_objects = vec![
            NativeObjectSnapshot {
                kind: NativeObjectKind::Platform {
                    kind: 0x00,
                    subtype: 0x00,
                },
                x_pos: 0x0030_0000,
                y_pos: 0x0018_0000,
                layout_index: None,
                x_velocity: None,
                y_velocity: None,
                facing: None,
                on_ground: None,
                hit_points: None,
                damage_flash: false,
                palette_variant: 0,
                stationary: false,
                contact_phase: None,
                shared_walker_state: None,
                shared_walker_patrol_timer: None,
                shared_walker_patrol_short_next: None,
                walk_phase: None,
                wait_timer: None,
                debug_fields: std::collections::BTreeMap::new(),
            },
            NativeObjectSnapshot {
                kind: NativeObjectKind::Enemy { object_type: 0x01 },
                x_pos: 0x0040_0000,
                y_pos: 0x0010_0000,
                layout_index: None,
                x_velocity: None,
                y_velocity: None,
                facing: None,
                on_ground: None,
                hit_points: None,
                damage_flash: false,
                palette_variant: 0,
                stationary: false,
                contact_phase: None,
                shared_walker_state: None,
                shared_walker_patrol_timer: None,
                shared_walker_patrol_short_next: None,
                walk_phase: None,
                wait_timer: None,
                debug_fields: std::collections::BTreeMap::new(),
            },
            NativeObjectSnapshot {
                kind: NativeObjectKind::Enemy { object_type: 0x1C },
                x_pos: 0x0050_0000,
                y_pos: 0x0010_0000,
                layout_index: None,
                x_velocity: None,
                y_velocity: None,
                facing: None,
                on_ground: None,
                hit_points: None,
                damage_flash: false,
                palette_variant: 0,
                stationary: false,
                contact_phase: None,
                shared_walker_state: None,
                shared_walker_patrol_timer: None,
                shared_walker_patrol_short_next: None,
                walk_phase: None,
                wait_timer: None,
                debug_fields: std::collections::BTreeMap::new(),
            },
            NativeObjectSnapshot {
                kind: NativeObjectKind::Player,
                x_pos: 0x0010_0000,
                y_pos: 0x0010_0000,
                layout_index: None,
                x_velocity: None,
                y_velocity: None,
                facing: None,
                on_ground: None,
                hit_points: None,
                damage_flash: false,
                palette_variant: 0,
                stationary: false,
                contact_phase: None,
                shared_walker_state: None,
                shared_walker_patrol_timer: None,
                shared_walker_patrol_short_next: None,
                walk_phase: None,
                wait_timer: None,
                debug_fields: std::collections::BTreeMap::new(),
            },
        ];
        let grid = BlockGrid::from_level_layout(&[0x60, 0x00, 0xE1, 0x02], 2, 1);

        let scene = build_ingame_scene(&trace, false, Some(&grid));

        assert!(
            indexed_tile_at(&scene, 0, 0).is_some(),
            "missing background art"
        );
        assert!(
            indexed_tile_at(&scene, 16, 0).is_some(),
            "missing terrain/block art"
        );
        assert!(
            scene.commands.iter().any(|command| {
                matches!(
                    command,
                    DrawCommand::Sprite { y, tile_sheet, tiles, .. }
                        if *y < 48
                            && tile_sheet.tiles.len() >= 16
                            && tiles.len() == 4
                            && tiles.first().is_some_and(|tile| tile.tile.tile_index == 8)
                )
            }),
            "missing diamond HUD art"
        );
        assert!(has_asset_backed_command(&scene, 41), "missing platform art");
        assert!(has_sprite_signature(&scene, 21), "missing collectible art");
        assert!(
            has_sprite_signature(&scene, 31) || has_sprite_signature(&scene, 32),
            "missing enemy art"
        );
        assert!(has_sprite_signature(&scene, 1), "missing player art");
        assert!(!has_guide_rect(&scene));
        assert!(!contains_text(&scene, "D:4"));
        assert!(!scene.commands.iter().any(|command| {
            matches!(command, DrawCommand::Rect { colour, .. } if *colour == PLAYER_COLOUR || *colour == ENEMY_COLOUR || *colour == ENEMY_FLASH_COLOUR || *colour == PLATFORM_COLOUR)
        }));
    }

    #[test]
    fn game_over_with_remaining_continues_does_not_render_invented_panel_loc_d25e() {
        let visuals = sample_level_visuals();
        let mut trace = sample_trace(GameMode::InGame);
        trace.game_over = true;
        trace.number_continues = 3;
        trace.game_over_elapsed_frames = 12;

        let scene = build_scene(&trace, false, Some(&visuals), None);

        assert!(
            !contains_text(&scene, "GAME OVER"),
            "ASM loc_D25E branches to Continue_Screen when Number_Continues is non-zero; native must not show the old text-panel GAME OVER interstitial"
        );
    }

    #[test]
    fn continue_countdown_uses_rom_eol_continues_sprites_loc_d25e() {
        let visuals = sample_level_visuals();
        let mut trace = sample_trace(GameMode::InGame);
        trace.game_over = true;
        trace.number_continues = 3;
        trace.continue_countdown = Some(291);
        trace.continue_countdown_elapsed_frames = 8;

        let scene = build_scene(&trace, false, Some(&visuals), None);

        let has_continues_label = scene.commands.iter().any(|command| {
            matches!(
                command,
                DrawCommand::Sprite { tiles, .. }
                    if tiles.first().is_some_and(|tile| tile.tile.tile_index == 0x008)
            )
        });
        let has_continue_count_digit = scene.commands.iter().any(|command| {
            matches!(
                command,
                DrawCommand::Sprite { tiles, .. }
                    if tiles.first().is_some_and(|tile| tile.tile.tile_index == 0x0CE)
            )
        });

        assert!(
            has_continues_label,
            "ASM Continue_Screen should draw Spr_EOL_Continues from EOL score art"
        );
        assert!(
            has_continue_count_digit,
            "ASM Continue_Screen Object_EOL_Number should draw the Number_Continues digit"
        );
        assert!(
            !contains_text(&scene, "CONTINUE"),
            "native should not draw the invented CONTINUE? / CONTINUES LEFT text overlay when ROM EOL assets are loaded"
        );
    }

    #[test]
    fn pause_menu_uses_rom_positioned_restart_text_loc_69d2() {
        let mut trace = sample_trace(GameMode::InGame);
        trace.paused = true;
        trace.number_lives = 3;

        let scene = build_ingame_scene(&trace, false, None);

        assert!(contains_text(&scene, "CONTINUE"));
        assert!(contains_text(&scene, "RESTART ROUND"));
        assert!(
            !contains_text(&scene, "PAUSED"),
            "ASM loc_69D2/restart_text writes the pause menu mappings; native should not render the old centred PAUSED panel"
        );
        assert!(
            !contains_text(&scene, "Press Start to resume"),
            "ASM loc_6A84 only writes cursor locations; native should not add an invented instruction hint"
        );
    }

    #[test]
    fn paused_scene_draws_pause_menu_tilemap_loc_69d2() {
        let mut visuals = sample_level_visuals();
        visuals.pause_menu = Some(sample_pause_menu_visuals());
        let mut trace = sample_trace(GameMode::InGame);
        trace.paused = true;
        trace.number_lives = 3;

        let scene = build_scene(&trace, false, Some(&visuals), None);

        let pause_tiles = scene
            .commands
            .iter()
            .filter_map(|command| match command {
                DrawCommand::IndexedTile {
                    x,
                    y,
                    tile_sheet,
                    tile,
                    ..
                } if tile_sheet.tiles.len() == 25
                    && (0x60..0xE0).contains(x)
                    && (0x40..0x68).contains(y) =>
                {
                    Some((*x, *y, *tile))
                }
                _ => None,
            })
            .collect::<Vec<_>>();

        assert_eq!(
            pause_tiles.len(),
            80,
            "ASM loc_69D2 writes a 16x5 tile pause menu block"
        );
        assert_eq!(
            pause_tiles[0],
            (
                0x60,
                0x40,
                TileRef {
                    tile_index: 0,
                    palette_line: 0,
                    priority: true,
                    flip_x: false,
                    flip_y: false,
                },
            ),
            "ASM loc_69D2 places the top-left MapUnc_PauseMenu tile at Camera_X+$60, Camera_Y+$40"
        );
        assert_eq!(
            pause_tiles[3 * 16 + 2].2.tile_index,
            0x0C,
            "more than one life selects MapUnc_PauseMenu_Restart at restart_text"
        );
        assert!(
            !contains_text(&scene, "RESTART ROUND"),
            "when ArtUnc_PauseMenu is available, native should use the ROM tilemap instead of overlay text"
        );
    }

    #[test]
    fn paused_scene_draws_give_up_tilemap_on_last_life_loc_69d2() {
        let mut visuals = sample_level_visuals();
        visuals.pause_menu = Some(sample_pause_menu_visuals());
        let mut trace = sample_trace(GameMode::InGame);
        trace.paused = true;
        trace.number_lives = 1;

        let scene = build_scene(&trace, false, Some(&visuals), None);

        let give_up_marker = scene.commands.iter().find_map(|command| match command {
            DrawCommand::IndexedTile {
                x,
                y,
                tile_sheet,
                tile,
                ..
            } if tile_sheet.tiles.len() == 25 && *x == 0x70 && *y == 0x58 => Some(*tile),
            _ => None,
        });

        assert_eq!(
            give_up_marker.map(|tile| tile.tile_index),
            Some(0x04),
            "last life selects MapUnc_PauseMenu_GiveUp at restart_text"
        );
        assert!(
            !contains_text(&scene, "GIVE UP"),
            "when ArtUnc_PauseMenu is available, native should use the ROM tilemap instead of overlay text"
        );
        assert!(!contains_text(&scene, "RESTART ROUND"));
    }

    #[test]
    fn paused_scene_moves_tile_cursor_from_pause_option_loc_6b0e() {
        let mut visuals = sample_level_visuals();
        visuals.pause_menu = Some(sample_pause_menu_visuals());
        let mut trace = sample_trace(GameMode::InGame);
        trace.paused = true;
        trace.pause_menu_option = 1;

        let scene = build_scene(&trace, false, Some(&visuals), None);
        let pause_tile_at = |x, y| {
            scene.commands.iter().find_map(|command| match command {
                DrawCommand::IndexedTile {
                    x: command_x,
                    y: command_y,
                    tile_sheet,
                    tile,
                    ..
                } if tile_sheet.tiles.len() == 25 && *command_x == x && *command_y == y => {
                    Some(*tile)
                }
                _ => None,
            })
        };

        assert_eq!(
            pause_tile_at(0x68, 0x48).map(|tile| tile.tile_index),
            Some(0x02),
            "ASM loc_6B0E writes blank tile $8678 to the upper cursor slot when moving down"
        );
        assert_eq!(
            pause_tile_at(0x68, 0x58).map(|tile| tile.tile_index),
            Some(0x06),
            "ASM loc_6B0E writes cursor tile $867C to the lower cursor slot when Pause_Option=1"
        );
        assert!(
            !contains_text(&scene, ">"),
            "when ArtUnc_PauseMenu is available, native should move the tile cursor rather than overlaying a text cursor"
        );
    }

    #[test]
    fn paused_ingame_scene_requests_palette_darken_loc_6c70() {
        let mut trace = sample_trace(GameMode::InGame);
        trace.paused = true;

        let scene = build_ingame_scene(&trace, false, None);

        assert!(
            scene.pause_palette_darken,
            "ASM sub_6874 -> Pause_DarkenPalette loc_6C70 darkens Palette_Buffer before rendering paused gameplay"
        );
    }

    #[test]
    fn paused_scene_hides_sprites_using_pause_menu_vram_tiles_sub_6874() {
        let tile_sheet = Arc::new(TileSheet {
            tiles: vec![make_tile(&[(0, 0, 1)])],
        });
        let palette = Arc::new(Palette {
            colours: vec![colour(255, 255, 255)],
        });
        let sprite = |x, tile_index| DrawCommand::Sprite {
            x,
            y: 16,
            tile_sheet: Arc::clone(&tile_sheet),
            palette: Arc::clone(&palette),
            tiles: vec![SpriteTile {
                offset_x: 0,
                offset_y: 0,
                tile: sample_tile_ref(tile_index),
            }],
        };
        let mut commands = vec![
            sprite(0, 0x0675),
            sprite(1, 0x0676),
            sprite(2, 0x8676),
            sprite(3, 0x068E),
            sprite(4, 0x068F),
            text(8, 8, "CONTINUE", [255, 255, 255], 1),
        ];

        hide_pause_menu_overwritten_sprite_commands_sub_6874(&mut commands);

        let remaining_sprite_x = commands
            .iter()
            .filter_map(|command| match command {
                DrawCommand::Sprite { x, .. } => Some(*x),
                _ => None,
            })
            .collect::<Vec<_>>();
        assert_eq!(
            remaining_sprite_x,
            vec![0, 4],
            "ASM loc_68AC masks the first sprite pattern word with `$7FF` and hides only `$676..=$68E` before the pause-menu art upload"
        );
        assert!(
            contains_text(
                &Scene {
                    background: [0, 0, 0],
                    commands,
                    pause_palette_darken: true,
                },
                "CONTINUE"
            ),
            "pause text is written after ASM sub_6874's sprite-table hide step and must remain visible"
        );
    }

    #[test]
    fn pause_menu_uses_give_up_text_on_last_life_restart_text() {
        let mut trace = sample_trace(GameMode::InGame);
        trace.paused = true;
        trace.number_lives = 1;

        let scene = build_ingame_scene(&trace, false, None);

        assert!(contains_text(&scene, "CONTINUE"));
        assert!(contains_text(&scene, "GIVE UP"));
        assert!(!contains_text(&scene, "RESTART ROUND"));
    }

    #[test]
    fn continue_countdown_holds_before_slide_in_loc_d31e() {
        let visuals = sample_level_visuals();
        let mut trace = sample_trace(GameMode::InGame);
        trace.game_over = true;
        trace.number_continues = 3;
        trace.continue_countdown = Some(319);

        trace.continue_countdown_elapsed_frames = 0x1B;
        let held_scene = build_scene(&trace, false, Some(&visuals), None);
        let held_x_positions = held_scene
            .commands
            .iter()
            .filter_map(|command| match command {
                DrawCommand::Sprite { x, tiles, .. }
                    if tiles
                        .first()
                        .is_some_and(|tile| tile.tile.tile_index == 0x008) =>
                {
                    Some(*x)
                }
                _ => None,
            })
            .collect::<Vec<_>>();
        assert!(
            held_x_positions.contains(&-0x78),
            "ASM Continue_Screen loc_D31E hibernates `$1C` frames before loc_D350 starts moving the CONTINUES label"
        );

        trace.continue_countdown_elapsed_frames = 0x1C + 8;
        let moving_scene = build_scene(&trace, false, Some(&visuals), None);
        let moving_x_positions = moving_scene
            .commands
            .iter()
            .filter_map(|command| match command {
                DrawCommand::Sprite { x, tiles, .. }
                    if tiles
                        .first()
                        .is_some_and(|tile| tile.tile.tile_index == 0x008) =>
                {
                    Some(*x)
                }
                _ => None,
            })
            .collect::<Vec<_>>();
        assert!(
            moving_x_positions.contains(&-0x38),
            "ASM Object_EOL_Text loc_C5A2 applies `$6C=8` only after the initial Continue_Screen hold"
        );

        trace.continue_countdown_elapsed_frames = 0x1C + 0x1C;
        let settled_text_scene = build_scene(&trace, false, Some(&visuals), None);
        let settled_text_x_positions = settled_text_scene
            .commands
            .iter()
            .filter_map(|command| match command {
                DrawCommand::Sprite { x, tiles, .. }
                    if tiles
                        .first()
                        .is_some_and(|tile| tile.tile.tile_index == 0x008) =>
                {
                    Some(*x)
                }
                _ => None,
            })
            .collect::<Vec<_>>();
        assert!(
            settled_text_x_positions.contains(&0x68),
            "ASM Continue_Screen sets `$46=$1C`; the CONTINUES label must finish at x=$68 instead of freezing half off-screen"
        );

        trace.continue_countdown_elapsed_frames = 0x1C + 0x1D;
        let settled_number_scene = build_scene(&trace, false, Some(&visuals), None);
        let settled_count_x_positions = settled_number_scene
            .commands
            .iter()
            .filter_map(|command| match command {
                DrawCommand::Sprite { x, tiles, .. }
                    if tiles
                        .first()
                        .is_some_and(|tile| tile.tile.tile_index == 0x0CE) =>
                {
                    Some(*x)
                }
                _ => None,
            })
            .collect::<Vec<_>>();
        assert!(
            settled_count_x_positions.contains(&0x9C),
            "ASM Continue_Screen sets Object_EOL_Number `$A=$1C`; the visible count digit must slide to centre instead of remaining off the right edge"
        );
    }

    #[test]
    fn no_continue_game_over_uses_rom_game_over_sprites_loc_d25e() {
        let visuals = sample_level_visuals();
        let mut trace = sample_trace(GameMode::InGame);
        trace.game_over = true;
        trace.number_continues = 0;
        trace.game_over_elapsed_frames = 8;

        let scene = build_scene(&trace, false, Some(&visuals), None);

        let has_game = scene.commands.iter().any(|command| {
            matches!(
                command,
                DrawCommand::Sprite { tiles, .. }
                    if tiles.first().is_some_and(|tile| tile.tile.tile_index == 0x031)
            )
        });
        let has_over = scene.commands.iter().any(|command| {
            matches!(
                command,
                DrawCommand::Sprite { tiles, .. }
                    if tiles.first().is_some_and(|tile| tile.tile.tile_index == 0x049)
            )
        });

        assert!(
            has_game && has_over,
            "ASM loc_D25E no-continues path should draw Spr_EOL_GAME and Spr_EOL_OVER instead of native text"
        );
    }

    #[test]
    fn resolve_ingame_vram_tile_maps_hud_symbols_and_diamond_tiles() {
        let visuals = sample_level_visuals();

        let colon = resolve_ingame_vram_tile(&visuals, TileRef::from_attr_word(0x86C4))
            .expect("colon tile");
        assert_eq!(colon.tile_sheet.tiles.len(), 12);
        assert_eq!(colon.tile.tile_index, 10);

        let diamond = resolve_ingame_vram_tile(&visuals, TileRef::from_attr_word(0x86F2))
            .expect("diamond tile");
        assert_eq!(diamond.tile_sheet.tiles.len(), 16);
        assert_eq!(diamond.tile.tile_index, 8);
    }

    #[test]
    fn block_destroy_effect_uses_sub_292e_rom_debris_tiles() {
        let visuals = sample_level_visuals();
        let mut trace = sample_trace(GameMode::InGame);
        trace.block_destroy_effects = vec![(0x0030, 0x0020, 2, 2)];

        let scene = build_scene(&trace, false, Some(&visuals), None);

        let expected_positions = [(31, 20), (31, 29), (41, 29), (41, 20), (37, 23)];
        for (x, y) in expected_positions {
            let (_, tile, _) =
                indexed_tile_at_with_sheet_len(&scene, x, y, 0x200).expect("debris tile");
            assert_eq!(
                tile.tile_index, 8,
                "ASM sub_2A4C emits VRAM tile $8228, i.e. block-tile sheet index 8"
            );
            assert!(tile.priority);
        }
        assert!(
            !has_rect(&scene, 37, 29, 6, 6, [180, 150, 110]),
            "block debris should use ASM sub_292E/sub_2A4C tile sprites when level art is loaded"
        );
    }

    #[test]
    fn falling_block_debris_uses_sub_2a4c_rom_tile_after_destroy_handoff() {
        let visuals = sample_level_visuals();
        let mut trace = sample_trace(GameMode::InGame);
        trace.block_debris_pieces = vec![(0x003F, 0x002C)];

        let scene = build_scene(&trace, false, Some(&visuals), None);

        let (_, tile, _) = indexed_tile_at_with_sheet_len(&scene, 47, 36, 0x200)
            .expect("falling debris tile from ASM sub_2A4C");
        assert_eq!(
            tile.tile_index, 8,
            "ASM sub_2A4C emits persistent falling shards with VRAM tile $8228, i.e. block sheet index 8"
        );
    }

    #[test]
    fn prize_block_spawn_flash_uses_loc_102b8_stru_102de_tiles_above_block() {
        let visuals = sample_level_visuals();
        let mut trace = sample_trace(GameMode::InGame);
        // ASM sub_1007A places the loc_102B8 flash at cell_x*16+8,
        // cell_y*16-8. With sample camera (16,8), world (56,24) draws a
        // 24x24 frame at screen top-left (28,4).
        trace.prize_block_spawn_flashes = vec![(0x0038, 0x0018, 25)];

        let scene = build_scene(&trace, false, Some(&visuals), None);

        let flash = scene.commands.iter().find_map(|command| match command {
            DrawCommand::Sprite {
                x,
                y,
                tile_sheet,
                tiles,
                ..
            } if *x == 28 && *y == 4 && tile_sheet.tiles.len() == 0x200 => Some(tiles),
            _ => None,
        });
        let tiles = flash.expect("ASM loc_102B8 spawn flash should render as a block-art sprite");
        assert_eq!(
            tiles[0].tile.tile_index, 0x00AF,
            "ASM stru_102DE starts at VRAM tile $2CF, i.e. block sheet index $AF",
        );
    }

    #[test]
    fn title_scene_stays_text_only() {
        let scene = build_scene(&sample_trace(GameMode::TitleScreen), true, None, None);
        assert!(contains_text(&scene, "KID CHAMELEON"));
        assert!(!scene.commands.iter().any(|command| {
            matches!(command, DrawCommand::Rect { colour, .. } if *colour == PLAYER_COLOUR || *colour == ENEMY_COLOUR || *colour == PLATFORM_COLOUR)
        }));
    }

    #[test]
    fn title_menu_fallback_shows_rom_menu_rows_when_cursor_is_active_loc_1bb52() {
        let mut trace = sample_trace(GameMode::TitleScreen);
        trace.options_cursor = Some(1);

        let scene = build_scene(&trace, true, None, None);

        assert!(
            contains_text(&scene, "1 PLAYER"),
            "ASM Obj_TitleMenu loc_1BB52 decodes the ROM menu tilemap instead of leaving the pre-menu press-start prompt"
        );
        assert!(contains_text(&scene, "2 PLAYERS"));
        assert!(contains_text(&scene, "OPTIONS"));
        assert!(!contains_text(&scene, "PRESS START"));
    }

    #[test]
    fn options_menu_fallback_uses_rom_option_copy_loc_1ce3a() {
        let mut trace = sample_trace(GameMode::OptionsMenu);
        trace.options_cursor = Some(2);
        trace.options_two_player_two_controllers = true;
        trace.options_controls_scheme = 4;
        trace.options_speed_button_fast = true;

        let scene = build_scene(&trace, true, None, None);

        assert!(contains_text(&scene, "2 PLAYERS"));
        assert!(contains_text(&scene, "TWO CONTROLLERS"));
        assert!(contains_text(&scene, "CONTROLS"));
        assert!(contains_text(&scene, "MODE"));
        assert!(contains_text(&scene, "JUMP"));
        assert!(contains_text(&scene, "SPECIAL"));
        assert!(contains_text(&scene, "SPEED BUTTON"));
        assert!(contains_text(&scene, "PUSH FOR SLOW"));
        assert!(contains_text(&scene, "EXIT"));
        assert!(!contains_text(&scene, "2 PLAYERS:"));
        assert!(!contains_text(&scene, "Up/Down select"));
    }

    #[test]
    fn options_menu_fallback_keeps_rom_rows_inside_frame_loc_1ce3a() {
        let mut trace = sample_trace(GameMode::OptionsMenu);
        trace.options_cursor = Some(1);
        trace.options_two_player_two_controllers = true;
        trace.options_controls_scheme = 5;
        trace.options_speed_button_fast = true;

        let scene = build_scene(&trace, true, None, None);

        for command in scene.commands {
            let DrawCommand::Text {
                x, y, text, scale, ..
            } = command
            else {
                continue;
            };
            let width = i32::try_from(text.len()).expect("text length fits") * 8 * scale;
            let height = 8 * scale;
            assert!(
                x >= 0
                    && y >= 0
                    && x + width <= FRAME_WIDTH as i32
                    && y + height <= FRAME_HEIGHT as i32,
                "ASM DrawOptText1..4 loc_1CE3A/loc_1CEB2/loc_1CF78/loc_1CFFE option row `{text}` should fit inside the native fallback frame"
            );
        }
    }

    #[test]
    fn pause_menu_cursor_uses_live_pause_option_loc_6b0e() {
        let mut trace = sample_trace(GameMode::InGame);
        trace.paused = true;
        trace.pause_menu_option = 1;
        let visuals = sample_level_visuals();

        let scene = build_scene(&trace, false, Some(&visuals), None);

        let cursor_rows: Vec<i32> = scene
            .commands
            .iter()
            .filter_map(|command| match command {
                DrawCommand::Text { x, y, text, .. } if *x == 0x68 && text == ">" => Some(*y),
                _ => None,
            })
            .collect();

        assert_eq!(
            cursor_rows,
            vec![0x58],
            "ASM Game_Paused_Loop loc_6B0E stores Pause_Option=1 for Restart Round / Give Up, so the pause cursor should render on the lower row"
        );
    }

    #[test]
    fn try_load_block_grid_prefers_generated_level_layout_dump() {
        let fixture = ManifestFixture::new().expect("manifest fixture");

        let grid = try_load_block_grid(&fixture.manifest, 0x00).expect("block grid");

        assert_eq!(grid.width_cells, 20);
        assert_eq!(grid.height_cells, 14);
        assert_eq!(grid.cell_at(0, 0).collision, CollisionKind::Spikes);
        assert_eq!(grid.cell_at(0, 0).skin, 0x33);
        assert_eq!(grid.cell_at(1, 0).collision, CollisionKind::Empty);
        assert_eq!(grid.cell_at(1, 0).skin, 0x00);
    }

    #[test]
    fn try_load_block_grid_rejects_partial_generated_level_layout_without_block_list() {
        let fixture = ManifestFixture::new().expect("manifest fixture");
        fs::remove_file(fixture._root.path().join("level-layout/00.blocklist.bin"))
            .expect("remove block list sidecar");

        let error = try_load_block_grid(&fixture.manifest, 0x00)
            .expect_err("generated Level_Layout without block list must fail closed");

        assert!(
            error.to_string().contains("missing matching block list"),
            "play-native must reject partial generated layouts; otherwise live helmet prizes can fall back to Diamond fixture mode: {error:#}"
        );
    }

    #[test]
    fn debug_title_card_entry_preserves_preloaded_collision_when_sidecar_probe_fails() {
        let fixture = ManifestFixture::new().expect("manifest fixture");
        let preloaded_grid = try_load_block_grid(&fixture.manifest, 0x00)
            .expect("complete generated layout should preload");
        fs::remove_file(fixture._root.path().join("level-layout/00.blocklist.bin"))
            .expect("remove block list sidecar after preloading");

        let mut runtime = NativeRuntime::new(fixture.manifest.clone());
        runtime.set_block_grid(preloaded_grid);

        let trace = runtime.enter_level_debug_title_card(LevelId(0));

        assert_eq!(trace.current_level, LevelId(0));
        assert!(
            runtime.block_grid().is_some(),
            "ASM loc_74E0 debug level changes enter loc_6E2/Load_TitleCard after play-native has already loaded target collision; a failed sidecar probe must not erase that grid and make Kid fall through the level"
        );
    }

    #[test]
    fn try_load_block_grid_rejects_missing_generated_level_layout() {
        let fixture = ManifestFixture::new().expect("manifest fixture");
        fs::remove_file(fixture._root.path().join("level-layout/00.bin")).expect("remove layout");
        fs::remove_file(fixture._root.path().join("level-layout/00.json"))
            .expect("remove layout metadata");

        let error = try_load_block_grid(&fixture.manifest, 0x00)
            .expect_err("missing generated Level_Layout must fail closed");

        assert!(
            error.to_string().contains("missing generated level layout"),
            "play-native must not reinterpret raw BLOCK resources as collision cells: {error:#}"
        );
    }

    #[test]
    #[ignore = "requires generated .cache assets; run mise run extract-assets first"]
    fn try_load_block_grid_decodes_bonus_level_without_fixture_loc_11c34() {
        // #246: the unreachable bonus levels (logical 0x5A/0x5D/0x5E/0x63) have no
        // pre-dumped `level-layout/<id>.bin` fixture, so the player's fixture-only
        // loader used to bail ("failed to load visuals"). With the engine decoder
        // fallback wired into `try_load_block_grid`, the native player now decodes
        // the collision terrain natively (ASM Load_Level loc_11C34 chain) and
        // returns a real, non-empty floor for these levels.
        let Some(manifest) = cached_generated_manifest() else {
            return;
        };

        // Sanity: this level genuinely has no fixture, so the fallback is exercised.
        let layout_dir = manifest.output_root.join("level-layout");
        assert!(
            !layout_dir.join("5A.bin").is_file(),
            "test premise broken: bonus level 0x5A now has a fixture; pick another fixture-less bonus level"
        );

        let grid =
            try_load_block_grid(&manifest, 0x5A).expect("bonus level 0x5A decodes via fallback");

        assert!(
            grid.width_cells > 0 && grid.height_cells > 0,
            "decoded bonus grid must have real dimensions"
        );
        let solid_cells = (0..grid.height_cells)
            .flat_map(|y| (0..grid.width_cells).map(move |x| (x, y)))
            .filter(|&(x, y)| grid.cell_at(x, y).collision != CollisionKind::Empty)
            .count();
        assert!(
            solid_cells > 0,
            "decoded bonus grid must contain a real floor (non-empty collision cells), not an empty void"
        );
    }

    #[test]
    fn highwater_pass2_real_grid_contains_loc_11430_e919_touch_shooter_cells() {
        let Some(manifest) = cached_generated_manifest() else {
            return;
        };

        let (_visuals, grid) = load_session_assets(&manifest, 0x03)
            .expect("level 0x03 generated layout should decode");
        let e919_cells: Vec<(u16, u16)> = (0..grid.height_cells)
            .flat_map(|row| (0..grid.width_cells).map(move |col| (col, row)))
            .filter(|&(col, row)| grid.cell_at(col, row) == Cell::from_be_bytes(0xE9, 0x19))
            .collect();
        assert_eq!(
            e919_cells,
            vec![(20, 9), (23, 9)],
            "ASM loc_11430/sub_114F4 restores Highwater Pass 2 touch-shooter cells as $E919"
        );
    }

    #[test]
    fn try_load_block_grid_rejects_mapped_decoded_layout_without_generated_dump() {
        let mut fixture = ManifestFixture::new().expect("manifest fixture");
        fs::remove_file(fixture._root.path().join("level-layout/00.bin")).expect("remove layout");
        fs::remove_file(fixture._root.path().join("level-layout/00.json"))
            .expect("remove layout metadata");

        write_file(
            fixture._root.path(),
            "decoded/level/27/block.bin",
            &[0x70, 0x11, 0x60, 0x22],
        )
        .expect("mapped decoded block");
        fixture.manifest.map_order = (0..=0x1B).collect();
        fixture.manifest.map_order[0x1B] = 0x27;
        fixture.manifest.levels.push(LevelRecord {
            level_id: 0x1B,
            title_text_id: Some(0x39),
            display_name: Some(String::from("Skydragon Castle 1")),
            title_card: None,
            resource_paths: Vec::new(),
            resources: LevelResources::default(),
            header_metadata: None,
            enemy_layout_metadata: None,
            bgscroll_metadata: None,
            platform_metadata: None,
            background_layout_alias: None,
        });
        fixture.manifest.levels.push(LevelRecord {
            level_id: 0x27,
            title_text_id: Some(0x27),
            display_name: Some(String::from("Caves of Ice")),
            title_card: None,
            resource_paths: Vec::new(),
            resources: LevelResources::default(),
            header_metadata: Some(LevelHeader {
                width_screens: 1,
                height_screens: 1,
                height_flag_bits: 0,
                level_theme: 0x09,
                murder_wall_flags: 0,
                background_theme: 0x09,
                weather_flags: 0,
                player_start_x: 0x0020,
                player_start_y: 0x0100,
                flag_x: 0x0C50,
                flag_y: 0x00B0,
            }),
            enemy_layout_metadata: None,
            bgscroll_metadata: None,
            platform_metadata: None,
            background_layout_alias: None,
        });
        fixture.manifest.decoded_levels.push(DecodedLevelAssets {
            contract_version: 3,
            level_id: 0x27,
            foreground: decoded_asset("decoded/level/27/foreground.bin"),
            block: decoded_asset("decoded/level/27/block.bin"),
            background: decoded_asset("decoded/level/27/background.bin"),
        });

        let error = try_load_block_grid(&fixture.manifest, 0x1B)
            .expect_err("mapped raw BLOCK resource is not a generated Level_Layout dump");

        assert!(
            error
                .to_string()
                .contains("logical level 0x1B (resource level 0x27)"),
            "map-order levels without generated dumps should fail closed with logical/resource context: {error:#}"
        );
    }

    #[test]
    fn try_load_block_grid_prefers_logical_level_layout_over_mapped_resource_layout() {
        let mut fixture = ManifestFixture::new().expect("manifest fixture");

        write_file(
            fixture._root.path(),
            "decoded/level/27/block.bin",
            &[0xE1, 0x44, 0x60, 0x55],
        )
        .expect("mapped decoded block");
        write_file(
            fixture._root.path(),
            "level-layout/1B.bin",
            &[0x70, 0x66, 0x60, 0x77],
        )
        .expect("logical layout");
        write_file(
            fixture._root.path(),
            "level-layout/1B.json",
            br#"{"data_bytes":4,"width_blocks":20,"height_blocks":14}"#,
        )
        .expect("logical layout metadata");
        write_file(
            fixture._root.path(),
            "level-layout/1B.blocklist.bin",
            &[0x00, 0x00],
        )
        .expect("logical block list");
        fixture.manifest.map_order = (0..=0x1B).collect();
        fixture.manifest.map_order[0x1B] = 0x27;
        fixture.manifest.levels.push(LevelRecord {
            level_id: 0x1B,
            title_text_id: Some(0x39),
            display_name: Some(String::from("Skydragon Castle 1")),
            title_card: None,
            resource_paths: Vec::new(),
            resources: LevelResources::default(),
            header_metadata: None,
            enemy_layout_metadata: None,
            bgscroll_metadata: None,
            platform_metadata: None,
            background_layout_alias: None,
        });
        fixture.manifest.levels.push(LevelRecord {
            level_id: 0x27,
            title_text_id: Some(0x27),
            display_name: Some(String::from("Caves of Ice")),
            title_card: None,
            resource_paths: Vec::new(),
            resources: LevelResources::default(),
            header_metadata: Some(LevelHeader {
                width_screens: 1,
                height_screens: 1,
                height_flag_bits: 0,
                level_theme: 0x09,
                murder_wall_flags: 0,
                background_theme: 0x09,
                weather_flags: 0,
                player_start_x: 0x0020,
                player_start_y: 0x0100,
                flag_x: 0x0C50,
                flag_y: 0x00B0,
            }),
            enemy_layout_metadata: None,
            bgscroll_metadata: None,
            platform_metadata: None,
            background_layout_alias: None,
        });
        fixture.manifest.decoded_levels.push(DecodedLevelAssets {
            contract_version: 3,
            level_id: 0x27,
            foreground: decoded_asset("decoded/level/27/foreground.bin"),
            block: decoded_asset("decoded/level/27/block.bin"),
            background: decoded_asset("decoded/level/27/background.bin"),
        });

        let grid = try_load_block_grid(&fixture.manifest, 0x1B).expect("logical block grid");

        assert_eq!(grid.width_cells, 20);
        assert_eq!(grid.height_cells, 14);
        assert_eq!(grid.cell_at(0, 0).collision, CollisionKind::Spikes);
        assert_eq!(grid.cell_at(0, 0).skin, 0x66);
        assert_eq!(grid.cell_at(1, 0).collision, CollisionKind::Solid);
        assert_eq!(grid.cell_at(1, 0).skin, 0x77);
    }

    #[test]
    fn resolved_level_header_prefers_resource_level_header_metadata() {
        let mut fixture = ManifestFixture::new().expect("manifest fixture");
        fixture.manifest.map_order = (0..=0x1B).collect();
        fixture.manifest.map_order[0x1B] = 0x27;
        fixture.manifest.levels.push(LevelRecord {
            level_id: 0x1B,
            title_text_id: Some(0x39),
            display_name: Some(String::from("Skydragon Castle 1")),
            title_card: None,
            resource_paths: Vec::new(),
            resources: LevelResources::default(),
            header_metadata: None,
            enemy_layout_metadata: None,
            bgscroll_metadata: None,
            platform_metadata: None,
            background_layout_alias: None,
        });
        fixture.manifest.levels.push(LevelRecord {
            level_id: 0x27,
            title_text_id: Some(0x27),
            display_name: Some(String::from("Caves of Ice")),
            title_card: None,
            resource_paths: Vec::new(),
            resources: LevelResources::default(),
            header_metadata: Some(LevelHeader {
                width_screens: 1,
                height_screens: 1,
                height_flag_bits: 0,
                level_theme: 0x09,
                murder_wall_flags: 0,
                background_theme: 0x04,
                weather_flags: 0,
                player_start_x: 0x0020,
                player_start_y: 0x0100,
                flag_x: 0x0C50,
                flag_y: 0x00B0,
            }),
            enemy_layout_metadata: None,
            bgscroll_metadata: None,
            platform_metadata: None,
            background_layout_alias: None,
        });

        let header = resolved_level_header(&fixture.manifest, 0x1B).expect("resolved header");

        assert_eq!(header.level_theme, 0x09);
        assert_eq!(header.background_theme, 0x04);
    }

    #[test]
    fn run_native_player_loads_level_visuals_once_before_main_loop() {
        let fixture = ManifestFixture::new().expect("manifest fixture");
        let load_count = AtomicUsize::new(0);

        let session_assets = load_native_session_assets_with(&fixture.manifest, 0x00, |_, _| {
            load_count.fetch_add(1, Ordering::SeqCst);
            Ok(sample_level_visuals())
        })
        .expect("session assets");
        let runtime = initialise_runtime(
            &fixture.manifest,
            &session_assets,
            &NativePlayOptions {
                fullscreen: false,
                scale: None,
                debug_overlay: false,
                warp_level: None,
                settled: false,
                record_to: None,
                record_frames: None,
                record_stop_on_target_enemy_contact: false,
                record_contact_tail_frames: 0,
                record_notes: None,
                record_route_target: None,
                record_frontier_kind: None,
                scripted_route: None,
            },
        );

        assert_eq!(load_count.load(Ordering::SeqCst), 1);
        let opening_cell = runtime
            .block_grid()
            .expect("runtime block grid")
            .cell_at(0, 0);
        assert_eq!(opening_cell.collision, CollisionKind::Spikes);
        assert_eq!(opening_cell.block_type, BlockType::None);
        assert_eq!(opening_cell.skin, 0x33);
        // Runtime now starts from the boot sequence (SegaScreen), not InGame
        let trace = runtime.trace_frame();
        assert_eq!(trace.game_mode, GameMode::SegaScreen);
        assert_eq!(trace.current_level.0, 0);
    }

    #[test]
    fn initialise_runtime_settled_warp_uses_recording_oracle_path() {
        let fixture = ManifestFixture::new().expect("manifest fixture");
        let session_assets = load_native_session_assets_with(&fixture.manifest, 0x00, |_, _| {
            Ok(sample_level_visuals())
        })
        .expect("session assets");
        let runtime = initialise_runtime(
            &fixture.manifest,
            &session_assets,
            &NativePlayOptions {
                fullscreen: false,
                scale: None,
                debug_overlay: false,
                warp_level: Some(0x00),
                settled: true,
                record_to: None,
                record_frames: None,
                record_stop_on_target_enemy_contact: false,
                record_contact_tail_frames: 0,
                record_notes: None,
                record_route_target: None,
                record_frontier_kind: None,
                scripted_route: None,
            },
        );

        let trace = runtime.trace_frame();
        assert_eq!(trace.current_level.0, 0x00);
        assert!(trace.kid_y_pos.is_some());
        assert!(
            trace.start_press_count > 0,
            "settled recording startup should replay the ASM loc_1AE2E title/start shell, matching verify-recording instead of a synthetic level entry"
        );
        assert!(
            trace.frame_index > 720,
            "settled recording startup should include the live ROM boot/settle window used by verify-recording"
        );
        assert_ne!(
            trace.game_mode,
            GameMode::SegaScreen,
            "settled recording startup must advance beyond cold boot even when the tiny unit-test manifest lacks full title-card assets"
        );
    }

    #[test]
    fn native_scripted_route_keeps_dynamic_scenario_probe_metadata() {
        let route = NativeScriptedRoute::from(DynamicScenario {
            slug: "live-prize-audio".into(),
            script: vec![0, 16],
            frame_count: 2,
            checkpoints: Vec::new(),
            start_position: Some(DynamicStartPosition { x: 230, y: 271 }),
            start_velocity: Some(crate::scenarios::DynamicStartVelocity {
                x: 0,
                y: -0x0004_0000,
            }),
            camera_position: Some(DynamicStartPosition { x: 16, y: 136 }),
            playtest_form: Some(PlayerForm::RedStealth.helmet_id()),
            initial_diamond_count: Some(12),
            initial_lives: Some(1),
            warp_to_level: Some(0),
            settle_oracle_warp: true,
        });

        assert_eq!(route.script, vec![0, 16]);
        assert!(route.delay_recording_input);
        assert_eq!(
            route.start_position,
            Some(DynamicStartPosition { x: 230, y: 271 })
        );
        assert_eq!(
            route.start_velocity,
            Some(crate::scenarios::DynamicStartVelocity {
                x: 0,
                y: -0x0004_0000,
            })
        );
        assert_eq!(
            route.camera_position,
            Some(DynamicStartPosition { x: 16, y: 136 })
        );
        assert_eq!(
            route.playtest_form,
            Some(PlayerForm::RedStealth.helmet_id())
        );
        assert_eq!(route.initial_diamond_count, Some(12));
        assert_eq!(route.initial_lives, Some(1));
    }

    #[test]
    fn debug_warp_candidates_follow_logical_map_order_and_skip_invalid_entries() {
        let mut fixture = ManifestFixture::new().expect("manifest fixture");
        fixture.manifest.map_order = vec![0x4A, u8::MAX, 0x2D, 0x2E];

        assert_eq!(
            debug_warp_candidate_ids(&fixture.manifest, LevelId(0), DebugWarpDirection::Next),
            vec![LevelId(2), LevelId(3), LevelId(0)],
            "debug next-level should follow logical map order and ignore u8::MAX holes"
        );
        assert_eq!(
            debug_warp_candidate_ids(&fixture.manifest, LevelId(0), DebugWarpDirection::Previous),
            vec![LevelId(3), LevelId(2), LevelId(0)],
            "debug previous-level should wrap through playable logical levels only"
        );
    }

    #[test]
    fn debug_warp_skips_targets_without_loadable_session_assets() {
        let mut fixture = ManifestFixture::new().expect("manifest fixture");
        fixture.manifest.map_order = vec![0x4A, 0x4B, 0x2D];
        let grid = try_load_block_grid(&fixture.manifest, 0x00).expect("fixture grid");

        let (level_id, _) = load_debug_warp_session_assets_with(
            &fixture.manifest,
            LevelId(0),
            DebugWarpDirection::Next,
            |_, level_id| {
                if level_id == 1 {
                    bail!("missing generated level layout for fixture target")
                }
                Ok(NativeSessionAssets {
                    level_visuals: sample_level_visuals(),
                    block_grid: grid.clone(),
                })
            },
        )
        .expect("next loadable debug target");

        assert_eq!(
            level_id,
            LevelId(2),
            "N/P debug warp should not enter a level that cannot load visuals and collision"
        );
    }

    #[test]
    fn debug_warp_without_generated_title_card_does_not_show_debug_name_card() {
        let mut fixture = ManifestFixture::new().expect("manifest fixture");
        let mut target = fixture.manifest.levels[0].clone();
        target.level_id = 0x01;
        target.display_name = Some(String::from("Debug Target"));
        target.title_text_id = Some(0x4B);
        target.title_card = None;
        fixture.manifest.levels.push(target);
        fixture.manifest.map_order = vec![0x4A, 0x4B];

        let mut runtime = NativeRuntime::new(fixture.manifest.clone());
        runtime.set_block_grid(try_load_block_grid(&fixture.manifest, 0x00).expect("fixture grid"));
        runtime.enter_level_playtest(LevelId(0));

        let mut trace = runtime.enter_level_debug_title_card(LevelId(1));
        let mut fallback_frames = debug_level_card_fallback_frames(&fixture.manifest, LevelId(1));

        assert_eq!(
            fallback_frames, 0,
            "debug N/P targets without generated Load_TitleCard script data should not show a tool-only LEVEL card before the real level intro"
        );

        if !native_level_card_hold_active(0, fallback_frames) {
            trace = runtime.step_frame(FrameInput::default());
        }

        assert_eq!(trace.game_mode, GameMode::LevelTransition);
        assert_eq!(trace.current_level_name.as_deref(), Some("Debug Target"));

        fallback_frames = fallback_frames.saturating_sub(1);
        assert!(!native_level_card_hold_active(0, fallback_frames));
    }

    #[test]
    fn debug_warp_with_generated_title_card_uses_rom_title_shell() {
        let mut fixture = ManifestFixture::new().expect("manifest fixture");
        let mut target = fixture.manifest.levels[0].clone();
        target.level_id = 0x01;
        target.display_name = Some(String::from("ROM Card Target"));
        target.title_text_id = Some(0x4B);
        target.title_card = Some(wildside_data::LevelTitleCard {
            script_bytes: vec![b'a', 0xFF],
            line_destinations: vec![wildside_data::TitleCardDestination { x: 0x80, y: 0x70 }],
            level_number_destination: None,
            level_number_glyph_count: 0,
        });
        fixture.manifest.levels.push(target);

        assert_eq!(
            debug_level_card_fallback_frames(&fixture.manifest, LevelId(1)),
            0,
            "targets with generated Load_TitleCard script data should stay on the ROM VDP/title-worker path, not the debug fallback card"
        );
    }

    #[test]
    fn generated_title_card_check_uses_map_order_resource_record() {
        let mut fixture = ManifestFixture::new().expect("manifest fixture");
        fixture.manifest.levels[0].title_card = Some(wildside_data::LevelTitleCard {
            script_bytes: vec![b'R', 0xFF],
            line_destinations: vec![wildside_data::TitleCardDestination { x: 0x80, y: 0x70 }],
            level_number_destination: None,
            level_number_glyph_count: 0,
        });
        let mut alias = fixture.manifest.levels[0].clone();
        alias.level_id = 0x06;
        alias.display_name = Some(String::from("Aliased Level"));
        alias.title_card = None;
        fixture.manifest.levels.push(alias);
        fixture.manifest.map_order = vec![0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00];

        assert!(
            level_has_generated_title_card(&fixture.manifest, LevelId(0x06)),
            "ASM Load_TitleCard resolves logical levels through MapOrder_Index, so play-native debug skips must see resource title-card data for aliases"
        );
    }

    #[test]
    fn debug_hotkey_level_skip_uses_rom_card_without_native_hold() {
        assert_eq!(
            debug_hotkey_level_card_hold_frames(),
            0,
            "native-only N/P debug skips should rely on ASM loc_6E2/Load_TitleCard instead of prepending a native hold"
        );
        assert_eq!(
            debug_hotkey_level_presentation(true).level_card_frames_remaining,
            0,
            "renderable Load_TitleCard targets should not get a second native card before the ROM intro"
        );
        assert_eq!(
            debug_hotkey_level_presentation(true).debug_level_card_fallback_frames_remaining,
            0,
            "renderable Load_TitleCard targets should not mask the ROM intro with the native debug card"
        );
        assert_eq!(
            debug_hotkey_level_presentation(false).debug_level_card_fallback_frames_remaining,
            0,
            "targets without renderable title-card VDP should not show a native debug LEVEL card before gameplay"
        );
    }

    #[test]
    fn generated_title_card_hold_advances_runtime_with_neutral_input() {
        assert!(
            should_step_runtime_during_level_card_hold(LEVEL_CARD_HOLD_FRAMES, 0),
            "ASM Load_TitleCard loc_1AB26/loc_1AC70 workers need neutral-input frames during the play-native N/P hold",
        );
        assert!(
            !should_step_runtime_during_level_card_hold(0, LEVEL_CARD_HOLD_FRAMES),
            "legacy fallback holds are static; N/P hotkeys no longer create them for missing title-card data",
        );
        assert!(
            !should_step_runtime_during_level_card_hold(0, 0),
            "ordinary gameplay should keep using live/scripted input"
        );
    }

    #[test]
    fn debug_hotkey_level_skip_enters_rom_card_without_native_hold() {
        let mut fixture = ManifestFixture::new().expect("manifest fixture");
        let mut target = fixture.manifest.levels[0].clone();
        target.level_id = 0x01;
        target.display_name = Some(String::from("Hotkey Target"));
        target.title_text_id = Some(0x4B);
        target.title_card = Some(wildside_data::LevelTitleCard {
            script_bytes: vec![b'H', 0xFF],
            line_destinations: vec![wildside_data::TitleCardDestination { x: 0x80, y: 0x70 }],
            level_number_destination: None,
            level_number_glyph_count: 0,
        });
        fixture.manifest.levels.push(target);

        let start_assets = NativeSessionAssets {
            level_visuals: sample_level_visuals(),
            block_grid: try_load_block_grid(&fixture.manifest, 0x00).expect("start grid"),
        };
        let target_assets = NativeSessionAssets {
            level_visuals: sample_level_visuals(),
            block_grid: try_load_block_grid(&fixture.manifest, 0x00).expect("target grid"),
        };
        let mut runtime = NativeRuntime::new(fixture.manifest.clone());
        let mut session_assets = start_assets;
        runtime.set_block_grid(session_assets.block_grid.clone());
        let mut trace = runtime.enter_level_playtest(LevelId(0));

        let presentation = enter_debug_hotkey_level(
            &fixture.manifest,
            &mut runtime,
            &mut trace,
            &mut session_assets,
            LevelId(1),
            target_assets,
        );

        assert_eq!(trace.current_level, LevelId(1));
        assert_eq!(
            trace.game_mode,
            GameMode::LevelTransition,
            "ASM loc_74E0 debug skip path should re-enter Load_TitleCard instead of synthetic gameplay"
        );
        assert_eq!(trace.current_level_name.as_deref(), Some("Hotkey Target"));
        assert_eq!(
            presentation.debug_level_card_fallback_frames_remaining, 0,
            "generated Load_TitleCard data should keep native hotkeys on the ROM title-card path even if this tiny fixture lacks live VDP state"
        );
        assert_eq!(
            presentation.level_card_frames_remaining, 0,
            "debug hotkeys should not prepend a native hold before ASM loc_6E2/Load_TitleCard"
        );
        assert!(presentation.wait_for_live_input_release);
        assert!(!presentation.suppress_level_card_for_debug_warp);
        assert!(!native_level_card_hold_active(
            presentation.level_card_frames_remaining,
            presentation.debug_level_card_fallback_frames_remaining
        ));
        assert!(
            runtime.vdp_state().is_none(),
            "the tiny unit fixture lacks extracted title-card art, but the presentation choice is keyed by generated title-card data so play-native never flashes the debug fallback before the ROM intro"
        );
    }

    #[test]
    #[ignore = "requires generated .cache assets; run mise run extract-assets first"]
    fn debug_hotkey_level_skip_with_generated_assets_animates_rom_title_card() {
        let Some(manifest) = cached_generated_manifest() else {
            return;
        };

        for (start_level, target_level, route_name) in [
            (0x00, 0x01, "Blue Lake Woods 1 -> Blue Lake Woods 2"),
            (0x01, 0x02, "Blue Lake Woods 2 -> Highwater Pass 1"),
        ] {
            let start_assets = load_native_session_assets(&manifest, start_level)
                .unwrap_or_else(|_| panic!("{route_name} start session assets"));
            let target_assets = load_native_session_assets(&manifest, target_level)
                .unwrap_or_else(|_| panic!("{route_name} target session assets"));
            let mut runtime = NativeRuntime::new(manifest.clone());
            let mut session_assets = start_assets;
            runtime.set_block_grid(session_assets.block_grid.clone());
            let mut trace = runtime.enter_level_playtest(LevelId(start_level));

            let presentation = enter_debug_hotkey_level(
                &manifest,
                &mut runtime,
                &mut trace,
                &mut session_assets,
                LevelId(target_level),
                target_assets,
            );

            assert_eq!(
                trace.game_mode,
                GameMode::LevelTransition,
                "{route_name} should enter ASM loc_6E2/Load_TitleCard"
            );
            assert_eq!(
                presentation.level_card_frames_remaining, 0,
                "{route_name} should not prepend a native hold before the ROM title-card shell"
            );
            assert_eq!(
                presentation.debug_level_card_fallback_frames_remaining, 0,
                "{route_name} generated title-card target should show the actual ROM level intro, not the debug fallback"
            );
            assert!(runtime.vdp_state().is_some(), "{route_name} VDP shell");
            assert!(
                !trace.active_objects.iter().any(|object| {
                    matches!(object.kind, NativeObjectKind::Enemy { object_type: 0 })
                        && object.debug_fields.contains_key("glyph")
                }),
                "ASM loc_6E2/Load_TitleCard can expose the VDP shell before loc_1AB26 title workers surface for {route_name}",
            );

            assert!(!should_step_runtime_during_level_card_hold(
                presentation.level_card_frames_remaining,
                presentation.debug_level_card_fallback_frames_remaining
            ));
            assert!(
                runtime.vdp_state().is_some(),
                "the first visible hotkey frame should come from the ROM VDP title-card shell for {route_name}"
            );
            let mut title_workers_visible = false;
            for _ in 0..LEVEL_CARD_HOLD_FRAMES {
                let advanced = runtime.step_frame(FrameInput::default());
                title_workers_visible = advanced.active_objects.iter().any(|object| {
                    matches!(object.kind, NativeObjectKind::Enemy { object_type: 0 })
                        && object.debug_fields.contains_key("glyph")
                });
                if title_workers_visible {
                    break;
                }
            }

            assert!(
                title_workers_visible,
                "play-native N/P must keep stepping with neutral input until ASM loc_1AB26 exposes title-card workers before the hold expires for {route_name}",
            );
        }
    }

    #[test]
    #[ignore = "requires generated .cache assets; run mise run extract-assets first"]
    fn debug_hotkey_rendered_vdp_shell_uses_target_theme_after_previous_theme_loc_19abe() {
        let Some(manifest) = cached_generated_manifest() else {
            return;
        };

        for (start_level, target_level, route_name) in [
            (0x06, 0x00, "cave -> Blue Lake Woods forest"),
            (0x06, 0x02, "cave -> Highwater Pass mountain"),
            (0x02, 0x06, "Highwater Pass -> Under Skull Mountain cave"),
        ] {
            let start_assets = load_native_session_assets(&manifest, start_level)
                .unwrap_or_else(|_| panic!("{route_name} start session assets"));
            let target_assets = load_native_session_assets(&manifest, target_level)
                .unwrap_or_else(|_| panic!("{route_name} target session assets"));
            let mut runtime = NativeRuntime::new(manifest.clone());
            let mut session_assets = start_assets;
            runtime.set_block_grid(session_assets.block_grid.clone());
            let mut trace = runtime.enter_level_playtest(LevelId(start_level));

            enter_debug_hotkey_level(
                &manifest,
                &mut runtime,
                &mut trace,
                &mut session_assets,
                LevelId(target_level),
                target_assets.clone(),
            );
            let hotkey_frame =
                render_vdp_frame_for_test(&runtime, &trace, &session_assets.level_visuals);

            let mut expected_runtime = NativeRuntime::new(manifest.clone());
            let expected_trace =
                expected_runtime.enter_level_debug_title_card(LevelId(target_level));
            let expected_frame = render_vdp_frame_for_test(
                &expected_runtime,
                &expected_trace,
                &target_assets.level_visuals,
            );

            let mut stale_runtime = NativeRuntime::new(manifest.clone());
            let stale_trace = stale_runtime.enter_level_debug_title_card(LevelId(start_level));
            let stale_frame = render_vdp_frame_for_test(
                &stale_runtime,
                &stale_trace,
                &session_assets.level_visuals,
            );

            assert_eq!(
                hotkey_frame, expected_frame,
                "ASM Load_TitleCard loc_19ABE must render the target title-card shell for {route_name}; play-native must not reuse stale VDP data",
            );
            assert_ne!(
                hotkey_frame, stale_frame,
                "the hotkey-rendered frame should not match the previous title-card shell for {route_name}",
            );
        }
    }

    #[test]
    #[ignore = "requires generated .cache assets; run mise run extract-assets first"]
    fn debug_hotkey_level_skip_with_generated_assets_enters_live_spawn() {
        let Some(manifest) = cached_generated_manifest() else {
            return;
        };

        for (start_level, target_level, route_name) in [
            (0x00, 0x01, "Blue Lake Woods 1 -> Blue Lake Woods 2"),
            (0x01, 0x02, "Blue Lake Woods 2 -> Highwater Pass 1"),
            (0x02, 0x03, "Highwater Pass 1 -> Highwater Pass 2"),
            (0x03, 0x04, "Highwater Pass 2 -> Under Skull Mountain 1"),
            (
                0x04,
                0x07,
                "Under Skull Mountain 1 -> Isle of the Lion Lord",
            ),
            (0x03, 0x06, "Highwater Pass 2 -> Under Skull Mountain 3"),
            (
                0x06,
                0x07,
                "Under Skull Mountain 3 -> Isle of the Lion Lord",
            ),
            (0x00, 0x1B, "Blue Lake Woods 1 -> Skydragon Castle 1"),
            (
                0x07,
                0x08,
                "Isle of the Lion Lord -> Hills of the Warrior 1",
            ),
        ] {
            let start_assets = load_native_session_assets(&manifest, start_level)
                .unwrap_or_else(|_| panic!("{route_name} start session assets"));
            let target_assets = load_native_session_assets(&manifest, target_level)
                .unwrap_or_else(|_| panic!("{route_name} target session assets"));
            let target_visual_level = target_assets.level_visuals.level.level_id;
            let target_visual_themes = (
                target_assets.level_visuals.level.foreground_theme_id,
                target_assets.level_visuals.level.background_theme_id,
            );
            let target_block_grid_dims = (
                target_assets.block_grid.width_cells,
                target_assets.block_grid.height_cells,
            );
            let target_header = resolved_level_header(&manifest, target_level)
                .unwrap_or_else(|| panic!("{route_name} target header"));
            let target_width_px = u32::from(target_header.width_screens) * FRAME_WIDTH;
            let target_height_px = u32::from(target_header.height_screens) * (FRAME_HEIGHT - 0x10);

            let mut runtime = NativeRuntime::new(manifest.clone());
            let mut session_assets = start_assets;
            runtime.set_block_grid(session_assets.block_grid.clone());
            let mut trace = runtime.enter_level_playtest(LevelId(start_level));
            let starting_lives = trace.number_lives;

            let presentation = enter_debug_hotkey_level(
                &manifest,
                &mut runtime,
                &mut trace,
                &mut session_assets,
                LevelId(target_level),
                target_assets,
            );

            assert_eq!(
                session_assets.level_visuals.level.level_id, target_visual_level,
                "{route_name} should swap the render bundle to the target logical level immediately"
            );
            assert_eq!(
                (
                    session_assets.level_visuals.level.foreground_theme_id,
                    session_assets.level_visuals.level.background_theme_id,
                ),
                target_visual_themes,
                "{route_name} should not keep the previous level's foreground/background themes after the hotkey"
            );
            assert_eq!(
                (
                    session_assets.block_grid.width_cells,
                    session_assets.block_grid.height_cells,
                ),
                target_block_grid_dims,
                "{route_name} should pair target visuals with the target generated block grid"
            );
            assert_eq!(
                presentation.level_card_frames_remaining, 0,
                "{route_name} should let ASM loc_6E2/Load_TitleCard own the generated title-card timing"
            );
            assert_eq!(
                presentation.debug_level_card_fallback_frames_remaining, 0,
                "{route_name} should not use the native debug fallback card"
            );

            for _ in 0..360 {
                trace = runtime.step_frame(FrameInput::default());
            }
            assert_eq!(
                trace.game_mode,
                GameMode::LevelTransition,
                "{route_name} should still be waiting in ASM loc_1AE2E before Start releases the title-card hibernate worker"
            );

            trace = runtime.step_frame(FrameInput::from_mask(0x80));
            assert_eq!(
                trace.game_mode,
                GameMode::LevelTransition,
                "{route_name} should latch Start on the title-card frame before entering live gameplay"
            );

            trace = runtime.step_frame(FrameInput::default());
            let mut live_trace = (trace.game_mode == GameMode::InGame).then_some(trace.clone());
            for _ in 0..16 {
                if live_trace.is_some() {
                    break;
                }
                trace = runtime.step_frame(FrameInput::default());
                if trace.game_mode == GameMode::InGame {
                    live_trace = Some(trace.clone());
                }
            }
            let mut trace =
                live_trace.unwrap_or_else(|| panic!("{route_name} did not reach live gameplay"));
            let kid_x = trace
                .kid_x_pos
                .unwrap_or_else(|| panic!("{route_name} live spawn should expose Kid x"));
            let kid_y = trace
                .kid_y_pos
                .unwrap_or_else(|| panic!("{route_name} live spawn should expose Kid y"));
            let kid_x_px = kid_x >> 16;
            let kid_y_px = kid_y >> 16;

            assert_eq!(trace.current_level, LevelId(target_level), "{route_name}");
            assert_eq!(
                trace.number_lives, starting_lives,
                "{route_name} should not die while handing off from ASM Load_TitleCard to Load_InGame"
            );
            assert!(
                kid_x_px.abs_diff(u32::from(target_header.player_start_x)) <= 32,
                "{route_name} ASM loc_74E0 debug skip should release loc_1AE2E near PlayerStart_X {:#06x}, not the old title-card slot-2 pose; x={kid_x:#010x}",
                target_header.player_start_x
            );
            assert!(
                kid_y_px.abs_diff(u32::from(target_header.player_start_y)) <= 32,
                "{route_name} ASM loc_74E0 debug skip should release near PlayerStart_Y {:#06x}; y={kid_y:#010x}",
                target_header.player_start_y
            );
            assert!(
                kid_x_px < target_width_px,
                "{route_name} live spawn x {kid_x:#010x} should be inside target level width {target_width_px}"
            );
            assert!(
                kid_y_px < target_height_px,
                "{route_name} live spawn y {kid_y:#010x} should be inside target level height {target_height_px}"
            );
            if target_header.murder_wall_flags == 0 || target_level == 0x08 {
                let post_live_frames = if target_level == 0x08 { 60 } else { 180 };
                for frame in 0..post_live_frames {
                    trace = runtime.step_frame(FrameInput::default());
                    assert_eq!(
                        trace.number_lives,
                        starting_lives,
                        "{route_name} should not die {frame} frames after live handoff; mode={:?} boot={:?} x={:?} y={:?} vy={:?} on_ground={:?} death_windup={} murder_wall_x={:?} debug={:?}",
                        trace.game_mode,
                        trace.boot_phase,
                        trace.kid_x_pos,
                        trace.kid_y_pos,
                        trace.kid_y_velocity,
                        trace.kid_on_ground,
                        trace.death_windup_elapsed,
                        trace.murder_wall_x,
                        trace.gameplay_debug_fields,
                    );
                    if target_header.murder_wall_flags == 0
                        && let Some(kid_y) = trace.kid_y_pos
                    {
                        assert!(
                            (kid_y >> 16) < target_height_px,
                            "{route_name} should keep Kid inside the target level height {target_height_px} after live handoff; frame {frame} y={kid_y:#010x}"
                        );
                    }
                }
            }
        }
    }

    #[test]
    #[ignore = "requires generated .cache assets; run mise run extract-assets first"]
    fn debug_hotkey_level_skip_can_chain_during_title_card_window() {
        let Some(manifest) = cached_generated_manifest() else {
            return;
        };

        let start_level = 0x03;
        let mid_level = 0x06;
        let target_level = 0x07;
        let route_name = "Highwater Pass 2 -> Under Skull Mountain 3 -> Isle of the Lion Lord";
        let start_assets = load_native_session_assets(&manifest, start_level)
            .unwrap_or_else(|_| panic!("{route_name} start session assets"));
        let mid_assets = load_native_session_assets(&manifest, mid_level)
            .unwrap_or_else(|_| panic!("{route_name} mid session assets"));
        let target_assets = load_native_session_assets(&manifest, target_level)
            .unwrap_or_else(|_| panic!("{route_name} target session assets"));
        let target_visual_level = target_assets.level_visuals.level.level_id;
        let target_visual_themes = (
            target_assets.level_visuals.level.foreground_theme_id,
            target_assets.level_visuals.level.background_theme_id,
        );
        let target_block_grid_dims = (
            target_assets.block_grid.width_cells,
            target_assets.block_grid.height_cells,
        );
        let target_header = resolved_level_header(&manifest, target_level)
            .unwrap_or_else(|| panic!("{route_name} target header"));

        let mut runtime = NativeRuntime::new(manifest.clone());
        let mut session_assets = start_assets;
        runtime.set_block_grid(session_assets.block_grid.clone());
        let mut trace = runtime.enter_level_playtest(LevelId(start_level));
        let starting_lives = trace.number_lives;

        let first = enter_debug_hotkey_level(
            &manifest,
            &mut runtime,
            &mut trace,
            &mut session_assets,
            LevelId(mid_level),
            mid_assets,
        );
        assert_eq!(first.level_card_frames_remaining, 0);
        assert_eq!(trace.game_mode, GameMode::LevelTransition);

        for _ in 0..20 {
            trace = runtime.step_frame(FrameInput::default());
            assert_eq!(
                trace.game_mode,
                GameMode::LevelTransition,
                "{route_name} should still be in ASM loc_1AE2E before the chained hotkey"
            );
        }

        let second = enter_debug_hotkey_level(
            &manifest,
            &mut runtime,
            &mut trace,
            &mut session_assets,
            LevelId(target_level),
            target_assets,
        );
        assert_eq!(second.level_card_frames_remaining, 0);
        assert_eq!(trace.current_level, LevelId(target_level));
        assert_eq!(trace.game_mode, GameMode::LevelTransition);
        assert_eq!(
            session_assets.level_visuals.level.level_id, target_visual_level,
            "{route_name} chained hotkey should swap render visuals to the final target"
        );
        assert_eq!(
            (
                session_assets.level_visuals.level.foreground_theme_id,
                session_assets.level_visuals.level.background_theme_id,
            ),
            target_visual_themes,
            "{route_name} chained hotkey should not keep the intermediate level's themes"
        );
        assert_eq!(
            (
                session_assets.block_grid.width_cells,
                session_assets.block_grid.height_cells,
            ),
            target_block_grid_dims,
            "{route_name} chained hotkey should pair target visuals with the target generated block grid"
        );

        for _ in 0..360 {
            let _ = runtime.step_frame(FrameInput::default());
        }
        trace = runtime.step_frame(FrameInput::from_mask(0x80));
        assert_eq!(trace.game_mode, GameMode::LevelTransition);

        let mut live_trace = None;
        for _ in 0..16 {
            let next_trace = runtime.step_frame(FrameInput::default());
            if next_trace.game_mode == GameMode::InGame {
                live_trace = Some(next_trace);
                break;
            }
        }
        let trace =
            live_trace.unwrap_or_else(|| panic!("{route_name} did not reach live gameplay"));
        let kid_x = trace
            .kid_x_pos
            .unwrap_or_else(|| panic!("{route_name} live spawn should expose Kid x"));
        let kid_y = trace
            .kid_y_pos
            .unwrap_or_else(|| panic!("{route_name} live spawn should expose Kid y"));

        assert_eq!(trace.number_lives, starting_lives);
        assert!(
            (kid_x >> 16).abs_diff(u32::from(target_header.player_start_x)) <= 32,
            "{route_name} chained debug skip should release near PlayerStart_X {:#06x}; x={kid_x:#010x}",
            target_header.player_start_x
        );
        assert!(
            (kid_y >> 16).abs_diff(u32::from(target_header.player_start_y)) <= 32,
            "{route_name} chained debug skip should release near PlayerStart_Y {:#06x}; y={kid_y:#010x}",
            target_header.player_start_y
        );
    }

    #[test]
    #[ignore = "requires generated .cache assets; run mise run extract-assets first"]
    fn debug_hotkey_level_skip_can_chain_from_start_to_isle_title_cards() {
        let Some(manifest) = cached_generated_manifest() else {
            return;
        };

        let route_name = "Blue Lake Woods 1 -> Isle of the Lion Lord through repeated N hotkeys";
        let start_assets = load_native_session_assets(&manifest, 0x00)
            .unwrap_or_else(|_| panic!("{route_name} start session assets"));
        let mut runtime = NativeRuntime::new(manifest.clone());
        let mut session_assets = start_assets;
        runtime.set_block_grid(session_assets.block_grid.clone());
        let mut trace = runtime.enter_level_playtest(LevelId(0x00));
        let starting_lives = trace.number_lives;

        for target_level in 0x01..=0x07 {
            let target_assets = load_native_session_assets(&manifest, target_level)
                .unwrap_or_else(|_| panic!("{route_name} target 0x{target_level:02X} assets"));
            let target_visual_level = target_assets.level_visuals.level.level_id;
            let target_visual_themes = (
                target_assets.level_visuals.level.foreground_theme_id,
                target_assets.level_visuals.level.background_theme_id,
            );
            let target_block_grid_dims = (
                target_assets.block_grid.width_cells,
                target_assets.block_grid.height_cells,
            );
            let presentation = enter_debug_hotkey_level(
                &manifest,
                &mut runtime,
                &mut trace,
                &mut session_assets,
                LevelId(target_level),
                target_assets,
            );

            assert_eq!(
                presentation.level_card_frames_remaining, 0,
                "{route_name}: ASM loc_74E0 jumps through loc_6E2/Load_TitleCard without a native pre-card"
            );
            assert_eq!(
                trace.current_level,
                LevelId(target_level),
                "{route_name}: repeated hotkeys should advance the logical Current_LevelID"
            );
            assert_eq!(
                trace.game_mode,
                GameMode::LevelTransition,
                "{route_name}: repeated hotkeys should stay in the ASM Load_TitleCard shell"
            );
            assert_eq!(
                session_assets.level_visuals.level.level_id, target_visual_level,
                "{route_name}: repeated hotkey target 0x{target_level:02X} should swap render visuals"
            );
            assert_eq!(
                (
                    session_assets.level_visuals.level.foreground_theme_id,
                    session_assets.level_visuals.level.background_theme_id,
                ),
                target_visual_themes,
                "{route_name}: repeated hotkey target 0x{target_level:02X} should use target visual themes"
            );
            assert_eq!(
                (
                    session_assets.block_grid.width_cells,
                    session_assets.block_grid.height_cells,
                ),
                target_block_grid_dims,
                "{route_name}: repeated hotkey target 0x{target_level:02X} should use target generated block grid"
            );

            for _ in 0..20 {
                trace = runtime.step_frame(FrameInput::default());
                assert_eq!(
                    trace.game_mode,
                    GameMode::LevelTransition,
                    "{route_name}: loc_1AE2E should keep target 0x{target_level:02X} title-carded until Start"
                );
            }
        }

        let target_header =
            resolved_level_header(&manifest, 0x07).expect("Isle of the Lion Lord header");
        let target_height_px = runtime
            .block_grid()
            .expect("Isle block grid should remain loaded after repeated hotkeys")
            .height_cells
            * 16;

        for _ in 0..360 {
            trace = runtime.step_frame(FrameInput::default());
            assert_eq!(
                trace.game_mode,
                GameMode::LevelTransition,
                "{route_name}: final Load_TitleCard shell should wait for the later Start edge"
            );
        }

        trace = runtime.step_frame(FrameInput::from_mask(0x80));
        assert_eq!(
            trace.game_mode,
            GameMode::LevelTransition,
            "{route_name}: loc_1AE2E latches Start on the title-card frame"
        );

        let mut live_trace = None;
        for _ in 0..16 {
            let next_trace = runtime.step_frame(FrameInput::default());
            if next_trace.game_mode == GameMode::InGame {
                live_trace = Some(next_trace);
                break;
            }
        }
        let trace =
            live_trace.unwrap_or_else(|| panic!("{route_name} did not reach live gameplay"));
        let kid_x = trace
            .kid_x_pos
            .unwrap_or_else(|| panic!("{route_name} live spawn should expose Kid x"));
        let kid_y = trace
            .kid_y_pos
            .unwrap_or_else(|| panic!("{route_name} live spawn should expose Kid y"));

        assert_eq!(trace.current_level, LevelId(0x07), "{route_name}");
        assert_eq!(
            trace.number_lives, starting_lives,
            "{route_name} should not lose a life during the title-card handoff"
        );
        assert!(
            (kid_x >> 16).abs_diff(u32::from(target_header.player_start_x)) <= 32,
            "{route_name} should release near PlayerStart_X {:#06x}; x={kid_x:#010x}",
            target_header.player_start_x
        );
        assert!(
            (kid_y >> 16).abs_diff(u32::from(target_header.player_start_y)) <= 32,
            "{route_name} should release near PlayerStart_Y {:#06x}; y={kid_y:#010x}",
            target_header.player_start_y
        );
        assert!(
            (kid_y >> 16) < u32::from(target_height_px),
            "{route_name} should not fall outside the repeated-hotkey target grid; y={kid_y:#010x} height={target_height_px}"
        );
    }

    #[test]
    #[ignore = "requires generated .cache assets; run mise run extract-assets first"]
    fn debug_hotkey_previous_chain_from_cave_back_to_blue_lake_uses_target_assets_loc_6e2() {
        let Some(manifest) = cached_generated_manifest() else {
            return;
        };

        let route_name = "Under Skull Mountain 3 -> Blue Lake Woods 1 through repeated P hotkeys";
        let start_assets = load_native_session_assets(&manifest, 0x06)
            .unwrap_or_else(|_| panic!("{route_name} start assets"));
        let mut runtime = NativeRuntime::new(manifest.clone());
        let mut session_assets = start_assets;
        runtime.set_block_grid(session_assets.block_grid.clone());
        let mut trace = runtime.enter_level_playtest(LevelId(0x06));
        let starting_lives = trace.number_lives;

        for target_level in (0x00..=0x05).rev() {
            let (resolved_level, target_assets) = load_debug_warp_session_assets(
                &manifest,
                trace.current_level,
                DebugWarpDirection::Previous,
            )
            .unwrap_or_else(|| panic!("{route_name}: previous target 0x{target_level:02X} assets"));
            assert_eq!(
                resolved_level,
                LevelId(target_level),
                "{route_name}: previous hotkey should walk down logical map order"
            );
            let target_visual_level = target_assets.level_visuals.level.level_id;
            let target_visual_themes = (
                target_assets.level_visuals.level.foreground_theme_id,
                target_assets.level_visuals.level.background_theme_id,
            );
            let target_block_grid_dims = (
                target_assets.block_grid.width_cells,
                target_assets.block_grid.height_cells,
            );

            let presentation = enter_debug_hotkey_level(
                &manifest,
                &mut runtime,
                &mut trace,
                &mut session_assets,
                resolved_level,
                target_assets,
            );

            assert_eq!(
                presentation.level_card_frames_remaining, 0,
                "{route_name}: ASM loc_74E0 should re-enter loc_6E2/Load_TitleCard without a native pre-card",
            );
            assert_eq!(
                trace.current_level, resolved_level,
                "{route_name}: logical Current_LevelID should follow the previous hotkey"
            );
            assert_eq!(
                trace.game_mode,
                GameMode::LevelTransition,
                "{route_name}: previous hotkey should stay in the ROM title-card shell"
            );
            assert_eq!(
                session_assets.level_visuals.level.level_id, target_visual_level,
                "{route_name}: render visuals should swap to target level 0x{target_level:02X}"
            );
            assert_eq!(
                (
                    session_assets.level_visuals.level.foreground_theme_id,
                    session_assets.level_visuals.level.background_theme_id,
                ),
                target_visual_themes,
                "{route_name}: target level 0x{target_level:02X} should not keep stale cave themes"
            );
            assert_eq!(
                (
                    session_assets.block_grid.width_cells,
                    session_assets.block_grid.height_cells,
                ),
                target_block_grid_dims,
                "{route_name}: target level 0x{target_level:02X} should pair visuals with target collision/prize grid"
            );

            for _ in 0..20 {
                trace = runtime.step_frame(FrameInput::default());
                assert_eq!(
                    trace.game_mode,
                    GameMode::LevelTransition,
                    "{route_name}: target 0x{target_level:02X} should remain in loc_1AE2E before Start"
                );
            }
        }

        for _ in 0..360 {
            trace = runtime.step_frame(FrameInput::default());
            assert_eq!(
                trace.game_mode,
                GameMode::LevelTransition,
                "{route_name}: final Blue Lake title-card shell should wait for Start"
            );
        }

        trace = runtime.step_frame(FrameInput::from_mask(0x80));
        assert_eq!(
            trace.game_mode,
            GameMode::LevelTransition,
            "{route_name}: loc_1AE2E latches Start on the title-card frame"
        );

        let mut live_trace = None;
        for _ in 0..16 {
            let next_trace = runtime.step_frame(FrameInput::default());
            if next_trace.game_mode == GameMode::InGame {
                live_trace = Some(next_trace);
                break;
            }
        }
        let trace =
            live_trace.unwrap_or_else(|| panic!("{route_name} did not reach live gameplay"));
        assert_eq!(trace.current_level, LevelId(0x00), "{route_name}");
        assert_eq!(
            trace.number_lives, starting_lives,
            "{route_name}: repeated previous hotkeys should not kill Kid during handoff"
        );
        assert_eq!(
            (
                session_assets.level_visuals.level.foreground_theme_id,
                session_assets.level_visuals.level.background_theme_id,
            ),
            (0x09, 0x09),
            "{route_name}: live Blue Lake Woods should use forest/tree visuals, not the stale cave theme"
        );
    }

    #[test]
    fn debug_hotkey_level_skip_suppresses_debug_card_without_generated_title_card() {
        let mut fixture = ManifestFixture::new().expect("manifest fixture");
        let mut target = fixture.manifest.levels[0].clone();
        target.level_id = 0x01;
        target.display_name = Some(String::from("Hotkey Fallback"));
        target.title_text_id = Some(0x4B);
        target.title_card = None;
        fixture.manifest.levels.push(target);

        let start_assets = NativeSessionAssets {
            level_visuals: sample_level_visuals(),
            block_grid: try_load_block_grid(&fixture.manifest, 0x00).expect("start grid"),
        };
        let target_assets = NativeSessionAssets {
            level_visuals: sample_level_visuals(),
            block_grid: try_load_block_grid(&fixture.manifest, 0x00).expect("target grid"),
        };
        let mut runtime = NativeRuntime::new(fixture.manifest.clone());
        let mut session_assets = start_assets;
        runtime.set_block_grid(session_assets.block_grid.clone());
        let mut trace = runtime.enter_level_playtest(LevelId(0));

        let presentation = enter_debug_hotkey_level(
            &fixture.manifest,
            &mut runtime,
            &mut trace,
            &mut session_assets,
            LevelId(1),
            target_assets,
        );

        assert_eq!(trace.current_level, LevelId(1));
        assert_eq!(trace.game_mode, GameMode::LevelTransition);
        assert_eq!(
            presentation.level_card_frames_remaining, 0,
            "targets without generated Load_TitleCard data cannot use the ROM title-card shell"
        );
        assert_eq!(
            presentation.debug_level_card_fallback_frames_remaining, 0,
            "missing generated title-card data should not trigger a synthetic debug LEVEL card"
        );
    }

    #[test]
    fn live_level_change_skips_targets_without_loadable_session_assets() {
        let mut fixture = ManifestFixture::new().expect("manifest fixture");
        fixture.manifest.map_order = vec![0x4A, 0x4B, 0x2D];
        let grid = try_load_block_grid(&fixture.manifest, 0x00).expect("fixture grid");

        let (level_id, _) =
            load_level_change_session_assets_with(&fixture.manifest, LevelId(1), |_, level_id| {
                if level_id == 1 {
                    bail!("missing generated visuals for runtime target")
                }
                Ok(NativeSessionAssets {
                    level_visuals: sample_level_visuals(),
                    block_grid: grid.clone(),
                })
            })
            .expect("next loadable level-change target");

        assert_eq!(
            level_id,
            LevelId(2),
            "ASM loc_74E0 advances Current_LevelID directly, but play-native must keep live level changes on levels with generated visuals and collision"
        );
    }

    #[test]
    fn live_level_change_resolved_target_reenters_rom_title_card() {
        let mut fixture = ManifestFixture::new().expect("manifest fixture");
        let mut target = fixture.manifest.levels[0].clone();
        target.level_id = 0x02;
        target.display_name = Some(String::from("Resolved Live Target"));
        target.title_text_id = Some(0x2D);
        target.title_card = Some(wildside_data::LevelTitleCard {
            script_bytes: vec![b'R', 0xFF],
            line_destinations: vec![wildside_data::TitleCardDestination { x: 0x80, y: 0x70 }],
            level_number_destination: None,
            level_number_glyph_count: 0,
        });
        fixture.manifest.levels.push(target);

        let mut runtime = NativeRuntime::new(fixture.manifest.clone());
        let mut session_assets = NativeSessionAssets {
            level_visuals: sample_level_visuals(),
            block_grid: try_load_block_grid(&fixture.manifest, 0x00).expect("start grid"),
        };
        runtime.set_block_grid(session_assets.block_grid.clone());
        let mut trace = runtime.enter_level_debug_title_card(LevelId(1));
        assert_eq!(trace.current_level, LevelId(1));

        let target_assets = NativeSessionAssets {
            level_visuals: sample_level_visuals(),
            block_grid: try_load_block_grid(&fixture.manifest, 0x00).expect("target grid"),
        };

        apply_live_level_change_session_assets(
            &mut runtime,
            &mut trace,
            &mut session_assets,
            LevelId(1),
            LevelId(2),
            target_assets,
        );

        assert_eq!(trace.current_level, LevelId(2));
        assert_eq!(
            trace.game_mode,
            GameMode::LevelTransition,
            "live level-change asset fallback should not use the raw playtest warp, which exposes the target spawn before ASM loc_6E2/Load_TitleCard"
        );
        assert_eq!(
            trace.current_level_name.as_deref(),
            Some("Resolved Live Target")
        );
    }

    #[test]
    fn live_level_change_direct_target_refreshes_runtime_block_grid_and_title_card() {
        let mut fixture = ManifestFixture::new().expect("manifest fixture");
        let mut target = fixture.manifest.levels[0].clone();
        target.level_id = 0x02;
        target.display_name = Some(String::from("Direct Live Target"));
        target.title_text_id = Some(0x2D);
        target.header_metadata = Some(LevelHeader {
            width_screens: 1,
            height_screens: 1,
            height_flag_bits: 0,
            level_theme: 0x08,
            murder_wall_flags: 0,
            background_theme: 0x08,
            weather_flags: 0,
            player_start_x: 0x0020,
            player_start_y: 0x0020,
            flag_x: 0x0100,
            flag_y: 0x0020,
        });
        target.title_card = Some(wildside_data::LevelTitleCard {
            script_bytes: vec![b'D', 0xFF],
            line_destinations: vec![wildside_data::TitleCardDestination { x: 0x80, y: 0x70 }],
            level_number_destination: None,
            level_number_glyph_count: 0,
        });
        fixture.manifest.levels.push(target);

        let mut runtime = NativeRuntime::new(fixture.manifest.clone());
        let start_grid = BlockGrid::from_level_layout(&[0; 20 * 14 * 2], 20, 14);
        let mut session_assets = NativeSessionAssets {
            level_visuals: sample_level_visuals(),
            block_grid: start_grid.clone(),
        };
        runtime.set_block_grid(session_assets.block_grid.clone());
        let mut trace = runtime.enter_level_debug_title_card(LevelId(2));
        assert_eq!(trace.current_level, LevelId(2));
        let pre_asset_swap_trace = trace.clone();

        let mut target_grid = BlockGrid::from_level_layout(&[0; 20 * 14 * 2], 20, 14);
        for col in 0..4 {
            target_grid.set_cell_at(col, 4, Cell::from_be_bytes(0xE1, 0x03));
        }
        let target_assets = NativeSessionAssets {
            level_visuals: sample_level_visuals(),
            block_grid: target_grid.clone(),
        };

        apply_live_level_change_session_assets(
            &mut runtime,
            &mut trace,
            &mut session_assets,
            LevelId(2),
            LevelId(2),
            target_assets,
        );

        assert_eq!(trace.current_level, LevelId(2));
        assert_eq!(
            trace.game_mode, pre_asset_swap_trace.game_mode,
            "direct live level-change asset swaps must preserve the core-owned transition shell instead of replaying ASM loc_6E2/Load_TitleCard"
        );
        assert_eq!(
            trace.current_level_name.as_deref(),
            pre_asset_swap_trace.current_level_name.as_deref(),
            "direct live level-change asset swaps should not rebuild the trace through a second title-card entry"
        );
        assert_eq!(
            runtime.block_grid().expect("runtime grid").cell_at(0, 4),
            target_grid.cell_at(0, 4),
            "direct live level changes must refresh the collision/prize grid, or debug skips keep stale block-list contents from the previous level"
        );

        let (_, kid_y) = runtime
            .level_entry_opening_player_position()
            .expect("direct live level-change transition should keep a hidden opening slice");
        assert!(
            kid_y >= 0x0030_0000,
            "direct live level-change asset swaps should rebuild the hidden opening slice from the swapped target grid, not preserve the stale no-floor entry pose; y={kid_y:#010x}"
        );
    }

    #[test]
    #[ignore = "requires generated .cache assets; run mise run extract-assets first"]
    fn live_level_change_direct_generated_targets_rebuild_opening_slice() {
        let Some(manifest) = cached_generated_manifest() else {
            return;
        };

        for target_level in [0x06, 0x07] {
            let route_name = format!("direct live level-change target 0x{target_level:02X}");
            let start_assets = load_native_session_assets(&manifest, 0x03)
                .unwrap_or_else(|_| panic!("{route_name} start assets"));
            let target_assets = load_native_session_assets(&manifest, target_level)
                .unwrap_or_else(|_| panic!("{route_name} target assets"));
            let target_grid = target_assets.block_grid.clone();
            let target_header = resolved_level_header(&manifest, target_level)
                .unwrap_or_else(|| panic!("{route_name} header"));

            let mut runtime = NativeRuntime::new(manifest.clone());
            let mut session_assets = start_assets;
            runtime.set_block_grid(session_assets.block_grid.clone());
            let mut trace = runtime.enter_level_debug_title_card(LevelId(target_level));
            assert_eq!(trace.current_level, LevelId(target_level), "{route_name}");

            apply_live_level_change_session_assets(
                &mut runtime,
                &mut trace,
                &mut session_assets,
                LevelId(target_level),
                LevelId(target_level),
                target_assets,
            );

            let runtime_grid = runtime
                .block_grid()
                .unwrap_or_else(|| panic!("{route_name} runtime grid"));
            assert_eq!(
                runtime_grid.width_cells, target_grid.width_cells,
                "{route_name}: direct swap should preserve the generated target grid width"
            );
            assert_eq!(
                runtime_grid.height_cells, target_grid.height_cells,
                "{route_name}: direct swap should preserve the generated target grid height"
            );

            let (kid_x, kid_y) = runtime
                .level_entry_opening_player_position()
                .unwrap_or_else(|| panic!("{route_name} opening slice"));
            assert!(
                (kid_x >> 16).abs_diff(u32::from(target_header.player_start_x)) <= 32,
                "{route_name}: direct asset swap should rebuild the hidden Load_InGame slice near PlayerStart_X {:#06x}; x={kid_x:#010x}",
                target_header.player_start_x
            );
            assert!(
                (kid_y >> 16).abs_diff(u32::from(target_header.player_start_y)) <= 32,
                "{route_name}: direct asset swap should rebuild the hidden Load_InGame slice near PlayerStart_Y {:#06x}; y={kid_y:#010x}",
                target_header.player_start_y
            );
            assert_eq!(
                trace.game_mode,
                GameMode::LevelTransition,
                "{route_name}: direct asset swap should preserve the active Load_TitleCard shell"
            );
        }
    }

    #[test]
    #[ignore = "requires generated .cache assets; run mise run extract-assets first"]
    fn live_level_change_direct_generated_target_refreshes_title_card_vdp_shell_loc_6e2() {
        let Some(manifest) = cached_generated_manifest() else {
            return;
        };

        let route_name = "direct live level-change cave -> Blue Lake Woods";
        let start_level = 0x06;
        let target_level = 0x00;
        let start_assets = load_native_session_assets(&manifest, start_level)
            .unwrap_or_else(|_| panic!("{route_name} start assets"));
        let target_assets = load_native_session_assets(&manifest, target_level)
            .unwrap_or_else(|_| panic!("{route_name} target assets"));

        let mut runtime = NativeRuntime::new(manifest.clone());
        let mut session_assets = start_assets;
        runtime.set_block_grid(session_assets.block_grid.clone());
        let mut trace = runtime.enter_level_debug_title_card(LevelId(start_level));
        let stale_frame =
            render_vdp_frame_for_test(&runtime, &trace, &session_assets.level_visuals);
        trace.current_level = LevelId(target_level);

        apply_live_level_change_session_assets(
            &mut runtime,
            &mut trace,
            &mut session_assets,
            LevelId(target_level),
            LevelId(target_level),
            target_assets,
        );
        let actual_frame =
            render_vdp_frame_for_test(&runtime, &trace, &session_assets.level_visuals);

        let mut expected_runtime = NativeRuntime::new(manifest);
        let expected_trace = expected_runtime.enter_level_debug_title_card(LevelId(target_level));
        let expected_frame = render_vdp_frame_for_test(
            &expected_runtime,
            &expected_trace,
            &session_assets.level_visuals,
        );

        assert_eq!(
            actual_frame, expected_frame,
            "ASM loc_6E2/Load_TitleCard should rebuild the target title-card VDP shell on direct live level changes, not keep the previous cave shell",
        );
        assert_ne!(
            actual_frame, stale_frame,
            "{route_name} should not render the previous cave title-card frame after the target asset swap",
        );
    }

    #[test]
    #[ignore = "requires generated .cache assets; run mise run extract-assets first"]
    fn debug_hotkey_cave_back_to_blue_lake_live_background_uses_target_visuals_loc_6e2() {
        let Some(manifest) = cached_generated_manifest() else {
            return;
        };

        let route_name = "debug hotkey cave -> Blue Lake Woods live handoff";
        let start_level = 0x06;
        let target_level = 0x00;
        let start_assets = load_native_session_assets(&manifest, start_level)
            .unwrap_or_else(|_| panic!("{route_name} start assets"));
        let stale_visuals = start_assets.level_visuals.clone();
        let target_assets = load_native_session_assets(&manifest, target_level)
            .unwrap_or_else(|_| panic!("{route_name} target assets"));
        let expected_visuals = target_assets.level_visuals.clone();

        let mut runtime = NativeRuntime::new(manifest.clone());
        let mut session_assets = start_assets;
        runtime.set_block_grid(session_assets.block_grid.clone());
        let mut trace = runtime.enter_level_playtest(LevelId(start_level));

        enter_debug_hotkey_level(
            &manifest,
            &mut runtime,
            &mut trace,
            &mut session_assets,
            LevelId(target_level),
            target_assets,
        );
        assert_eq!(trace.current_level, LevelId(target_level), "{route_name}");
        assert_eq!(trace.game_mode, GameMode::LevelTransition, "{route_name}");
        assert_eq!(
            session_assets.level_visuals.level.level_id, expected_visuals.level.level_id,
            "{route_name}: active render bundle should switch to Blue Lake Woods before the ROM title-card shell"
        );

        for _ in 0..360 {
            trace = runtime.step_frame(FrameInput::default());
            assert_eq!(
                trace.game_mode,
                GameMode::LevelTransition,
                "{route_name}: Load_TitleCard should still be waiting for Start"
            );
        }
        trace = runtime.step_frame(FrameInput::from_mask(0x80));
        assert_eq!(
            trace.game_mode,
            GameMode::LevelTransition,
            "{route_name}: Start is latched before the live handoff"
        );

        let mut live_trace = None;
        for _ in 0..16 {
            let next_trace = runtime.step_frame(FrameInput::default());
            if next_trace.game_mode == GameMode::InGame {
                live_trace = Some(next_trace);
                break;
            }
        }
        let trace =
            live_trace.unwrap_or_else(|| panic!("{route_name} did not reach live gameplay"));
        assert_eq!(trace.current_level, LevelId(target_level), "{route_name}");
        assert_eq!(
            session_assets.level_visuals.level.level_id, expected_visuals.level.level_id,
            "{route_name}: live gameplay should still render with the target level bundle"
        );

        let mut target_matches = 0usize;
        let mut stale_mismatches = 0usize;
        for y in (8..216).step_by(8) {
            for x in (8..312).step_by(8) {
                let Some(actual) =
                    sample_background_pixel_loc_c070(&trace, &session_assets.level_visuals, x, y)
                else {
                    continue;
                };
                let Some(expected) =
                    sample_background_pixel_loc_c070(&trace, &expected_visuals, x, y)
                else {
                    continue;
                };
                if actual.rgb == expected.rgb {
                    target_matches += 1;
                }
                if let Some(stale) = sample_background_pixel_loc_c070(&trace, &stale_visuals, x, y)
                    && stale.rgb != expected.rgb
                    && actual.rgb != stale.rgb
                {
                    stale_mismatches += 1;
                }
            }
        }

        assert!(
            target_matches >= 8,
            "{route_name}: live background samples should match fresh Blue Lake Woods visuals, not go black or reuse a stale shell"
        );
        assert!(
            stale_mismatches >= 4,
            "{route_name}: sampled live background should differ from the previous cave background at multiple visible points"
        );
    }

    #[test]
    #[ignore = "requires generated .cache assets; run mise run extract-assets first"]
    fn debug_hotkey_cave_to_highwater_live_background_uses_target_visuals_loc_6e2() {
        let Some(manifest) = cached_generated_manifest() else {
            return;
        };

        let route_name = "debug hotkey cave -> Highwater Pass live handoff";
        let start_level = 0x06;
        let target_level = 0x02;
        let start_assets = load_native_session_assets(&manifest, start_level)
            .unwrap_or_else(|_| panic!("{route_name} start assets"));
        let target_assets = load_native_session_assets(&manifest, target_level)
            .unwrap_or_else(|_| panic!("{route_name} target assets"));
        let expected_visuals = target_assets.level_visuals.clone();

        let mut runtime = NativeRuntime::new(manifest.clone());
        let mut session_assets = start_assets;
        runtime.set_block_grid(session_assets.block_grid.clone());
        let mut trace = runtime.enter_level_playtest(LevelId(start_level));

        enter_debug_hotkey_level(
            &manifest,
            &mut runtime,
            &mut trace,
            &mut session_assets,
            LevelId(target_level),
            target_assets,
        );
        assert_eq!(trace.current_level, LevelId(target_level), "{route_name}");
        assert_eq!(trace.game_mode, GameMode::LevelTransition, "{route_name}");
        assert_eq!(
            session_assets.level_visuals.level.level_id, expected_visuals.level.level_id,
            "{route_name}: active render bundle should switch to Highwater Pass before the ROM title-card shell"
        );

        for _ in 0..360 {
            trace = runtime.step_frame(FrameInput::default());
            assert_eq!(
                trace.game_mode,
                GameMode::LevelTransition,
                "{route_name}: Load_TitleCard should still be waiting for Start"
            );
        }
        trace = runtime.step_frame(FrameInput::from_mask(0x80));
        assert_eq!(
            trace.game_mode,
            GameMode::LevelTransition,
            "{route_name}: Start is latched before the live handoff"
        );

        let mut live_trace = None;
        for _ in 0..16 {
            let next_trace = runtime.step_frame(FrameInput::default());
            if next_trace.game_mode == GameMode::InGame {
                live_trace = Some(next_trace);
                break;
            }
        }
        let trace =
            live_trace.unwrap_or_else(|| panic!("{route_name} did not reach live gameplay"));
        assert_eq!(trace.current_level, LevelId(target_level), "{route_name}");
        assert_eq!(
            session_assets.level_visuals.level.level_id, expected_visuals.level.level_id,
            "{route_name}: live gameplay should still render with the target level bundle"
        );

        let mut target_matches = 0usize;
        let mut non_black_samples = 0usize;
        let mut distinct_colours = std::collections::BTreeSet::new();
        for y in (8..216).step_by(8) {
            for x in (8..312).step_by(8) {
                let Some(actual) =
                    sample_background_pixel_loc_c070(&trace, &session_assets.level_visuals, x, y)
                else {
                    continue;
                };
                let Some(expected) =
                    sample_background_pixel_loc_c070(&trace, &expected_visuals, x, y)
                else {
                    continue;
                };
                if actual.rgb == expected.rgb {
                    target_matches += 1;
                }
                if actual.rgb != [0, 0, 0] {
                    non_black_samples += 1;
                }
                distinct_colours.insert(actual.rgb);
            }
        }

        assert!(
            target_matches >= 128,
            "{route_name}: live background samples should match fresh Highwater Pass visuals, not go black or reuse a stale shell"
        );
        assert!(
            non_black_samples >= 128 && distinct_colours.len() >= 2,
            "{route_name}: sampled live background should not collapse to a black frame; non_black_samples={non_black_samples}, distinct_colours={}",
            distinct_colours.len()
        );
    }

    #[test]
    fn native_session_block_grid_prefers_logical_layout_before_resource_fallback() {
        let mut fixture = ManifestFixture::new().expect("manifest fixture");
        fixture.manifest.map_order = vec![0x00, 0x01, 0x02, 0x03, 0x01];

        write_file(fixture._root.path(), "level-layout/04.bin", &[0x00, 0x00])
            .expect("write logical layout");
        write_file(
            fixture._root.path(),
            "level-layout/04.json",
            br#"{"data_bytes":2,"width_blocks":1,"height_blocks":1}"#,
        )
        .expect("write logical metadata");
        write_file(
            fixture._root.path(),
            "level-layout/04.blocklist.bin",
            &[0x00, 0x00],
        )
        .expect("write logical block list");
        write_file(fixture._root.path(), "level-layout/01.bin", &[0x60, 0x42])
            .expect("write resource layout");
        write_file(
            fixture._root.path(),
            "level-layout/01.json",
            br#"{"data_bytes":2,"width_blocks":1,"height_blocks":1}"#,
        )
        .expect("write resource metadata");
        write_file(
            fixture._root.path(),
            "level-layout/01.blocklist.bin",
            &[0x00, 0x00],
        )
        .expect("write resource block list");

        let grid = try_load_block_grid(&fixture.manifest, 0x04)
            .expect("logical level should load generated logical collision");
        let cell = grid.cell_at(0, 0);

        assert_eq!(
            cell.collision,
            CollisionKind::Empty,
            "generated logical Level_Layout files are parity specialisations and should win over the mapped resource fallback"
        );
    }

    #[test]
    fn native_session_block_grid_falls_back_to_map_order_resource_layout() {
        let mut fixture = ManifestFixture::new().expect("manifest fixture");
        fixture.manifest.map_order = vec![0x00, 0x01, 0x02, 0x03, 0x01];

        write_file(fixture._root.path(), "level-layout/01.bin", &[0x60, 0x42])
            .expect("write resource layout");
        write_file(
            fixture._root.path(),
            "level-layout/01.json",
            br#"{"data_bytes":2,"width_blocks":1,"height_blocks":1}"#,
        )
        .expect("write resource metadata");
        write_file(
            fixture._root.path(),
            "level-layout/01.blocklist.bin",
            &[0x00, 0x00],
        )
        .expect("write resource block list");

        let grid = try_load_block_grid(&fixture.manifest, 0x04)
            .expect("logical level should fall back to mapped resource collision");
        let cell = grid.cell_at(0, 0);

        assert_eq!(
            cell.collision,
            CollisionKind::Solid,
            "ASM Load_Level_Data resolves Current_LevelID through MapOrder_Index when no generated logical Level_Layout exists"
        );
        assert_eq!(cell.skin, 0x42);
    }

    #[test]
    fn save_native_recording_bundle_persists_scripted_route_metadata_and_sfx_events() {
        let fixture = ManifestFixture::new().expect("manifest fixture");
        let record_dir = fixture._root.path().join("recordings").join("freeze-repro");
        let trace = sample_trace(GameMode::LevelTransition);
        let mut sfx_trace = trace.clone();
        sfx_trace.played_sound_effect_ids.push(gems_sfx::BLOCK_HIT);
        let final_frame = vec![0x22; (FRAME_WIDTH * FRAME_HEIGHT * 3) as usize];

        save_native_recording_bundle(
            &fixture.manifest,
            &NativePlayOptions {
                fullscreen: false,
                scale: None,
                debug_overlay: false,
                warp_level: None,
                settled: false,
                record_to: Some(record_dir.clone()),
                record_frames: Some(900),
                record_stop_on_target_enemy_contact: true,
                record_contact_tail_frames: 12,
                record_notes: Some(String::from("stuck on level card")),
                record_route_target: Some(RecordingRouteTarget {
                    label: Some(String::from("usm3-slope")),
                    level: Some(0x06),
                    points: Vec::new(),
                    cells: vec![crate::recording::RecordingTargetCell { col: 28, row: 32 }],
                    cell_hitboxes: Vec::new(),
                    enemies: vec![0x0A],
                }),
                record_frontier_kind: Some(String::from("gameplay")),
                scripted_route: Some(NativeScriptedRoute {
                    script: vec![16, 0],
                    delay_recording_input: false,
                    start_position: Some(DynamicStartPosition { x: 230, y: 271 }),
                    start_velocity: None,
                    camera_position: Some(DynamicStartPosition { x: 16, y: 136 }),
                    playtest_form: Some(PlayerForm::RedStealth.helmet_id()),
                    initial_diamond_count: Some(12),
                    initial_lives: Some(1),
                }),
            },
            vec![16, 0],
            vec![trace.clone(), sfx_trace],
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Some(final_frame),
            trace.clone(),
            None,
            1,
            AUDIO_SAMPLE_RATE,
        )
        .expect("save freeze repro");

        let manifest_text =
            fs::read_to_string(record_dir.join("manifest.json")).expect("manifest text");
        assert!(manifest_text.contains("\"slug\": \"freeze-repro\""));

        let trace_text = fs::read_to_string(record_dir.join("trace.json")).expect("trace text");
        assert!(trace_text.contains("\"game_mode\": \"LevelTransition\""));
        assert!(trace_text.contains("\"frame_index\": 720"));

        let scenario_text =
            fs::read_to_string(record_dir.join("scenario.json")).expect("scenario text");
        assert!(scenario_text.contains("\"frame_count\": 2"));
        assert!(scenario_text.contains("\"script\": [\n    16,\n    0\n  ]"));
        assert!(scenario_text.contains("\"start_position\": {"));
        assert!(scenario_text.contains("\"x\": 230"));
        assert!(scenario_text.contains("\"y\": 271"));
        assert!(scenario_text.contains("\"camera_position\": {"));
        assert!(scenario_text.contains("\"playtest_form\": 3"));
        assert!(scenario_text.contains("\"initial_diamond_count\": 12"));
        assert!(scenario_text.contains("\"initial_lives\": 1"));
        let audio_events =
            fs::read_to_string(record_dir.join("audio-events.txt")).expect("audio events");
        assert!(audio_events.contains("frame=1"));
        assert!(audio_events.contains("StartSequence(13)"));
        assert!(manifest_text.contains("\"record_frames\": 900"));
        assert!(
            manifest_text.contains(
                "--record freeze-repro --record-frames 900 --record-stop-on-target-enemy-contact --record-contact-tail-frames 12 --record-target-label usm3-slope --record-target-level 06 --record-target-cell 28,32 --record-target-enemy 0A --record-frontier-kind gameplay"
            )
        );
        assert!(manifest_text.contains("\"route_target\""));
        assert!(manifest_text.contains("\"label\": \"usm3-slope\""));
        assert!(manifest_text.contains("\"route_target_score_commands\""));
        assert!(
            manifest_text.contains(
                "probe-record-scenario-file --scenario .cache/recordings/freeze-repro/scenario.json --target-enemy-contact 0A --summary-only"
            )
        );
        assert!(
            manifest_text.contains(
                "verify-tas --scenario .cache/recordings/freeze-repro/scenario.json --bisect --frontier-kind gameplay"
            )
        );
    }

    #[test]
    fn save_native_recording_bundle_adds_target_enemy_contact_checkpoint_loc_3b464() {
        let fixture = ManifestFixture::new().expect("manifest fixture");
        let record_dir = fixture
            ._root
            .path()
            .join("recordings")
            .join("issue271-live-rocktank");
        let trace = sample_trace(GameMode::InGame);
        let contact_stop = TargetEnemyContactStop {
            frame_index: 1,
            object_type: 0x0A,
            result: String::from("damage"),
            gate: String::from("classify"),
            score_px: 17,
        };

        save_native_recording_bundle(
            &fixture.manifest,
            &NativePlayOptions {
                fullscreen: false,
                scale: None,
                debug_overlay: false,
                warp_level: Some(0x01),
                settled: true,
                record_to: Some(record_dir.clone()),
                record_frames: None,
                record_stop_on_target_enemy_contact: true,
                record_contact_tail_frames: 30,
                record_notes: None,
                record_route_target: Some(RecordingRouteTarget {
                    label: Some(String::from("issue271-live-rocktank")),
                    level: Some(0x01),
                    points: Vec::new(),
                    cells: Vec::new(),
                    cell_hitboxes: Vec::new(),
                    enemies: vec![0x0A],
                }),
                record_frontier_kind: Some(String::from("gameplay")),
                scripted_route: None,
            },
            vec![0, 0, 0],
            vec![trace.clone(), trace.clone(), trace.clone()],
            Vec::new(),
            Vec::new(),
            Vec::new(),
            None,
            trace,
            Some(&contact_stop),
            1,
            AUDIO_SAMPLE_RATE,
        )
        .expect("save contact repro");

        let scenario_text =
            fs::read_to_string(record_dir.join("scenario.json")).expect("scenario text");
        assert!(scenario_text.contains("\"label\": \"target_enemy_contact_0A_damage\""));
        assert!(scenario_text.contains("\"frame_index\": 1"));
    }

    #[test]
    fn target_enemy_contact_checkpoint_only_dedupes_exact_label_and_frame_loc_3b464() {
        let mut scenario = DynamicScenario {
            slug: String::from("issue271-live-rocktank"),
            script: vec![0; 4],
            frame_count: 4,
            checkpoints: vec![
                DynamicCheckpoint {
                    label: String::from("target_enemy_contact_0A_damage"),
                    frame_index: 1,
                },
                DynamicCheckpoint {
                    label: String::from("target_enemy_contact_0B_damage"),
                    frame_index: 2,
                },
            ],
            start_position: None,
            start_velocity: None,
            camera_position: None,
            playtest_form: None,
            initial_diamond_count: None,
            initial_lives: None,
            warp_to_level: Some(0x01),
            settle_oracle_warp: true,
        };
        let contact = TargetEnemyContactStop {
            frame_index: 2,
            object_type: 0x0A,
            result: String::from("damage"),
            gate: String::from("classify"),
            score_px: 17,
        };

        append_target_enemy_contact_checkpoint(&mut scenario, &contact);
        append_target_enemy_contact_checkpoint(&mut scenario, &contact);

        let matching: Vec<_> = scenario
            .checkpoints
            .iter()
            .filter(|checkpoint| checkpoint.label == "target_enemy_contact_0A_damage")
            .map(|checkpoint| checkpoint.frame_index)
            .collect();
        assert_eq!(
            matching,
            vec![1, 2],
            "same label at a different frame is a distinct contact checkpoint, but the exact duplicate is ignored"
        );
        assert_eq!(
            scenario.checkpoints.len(),
            3,
            "different enemy label at the same frame must not block the requested contact checkpoint"
        );
    }

    #[test]
    fn native_recording_follow_up_commands_include_route_target_scorers() {
        let commands = native_recording_follow_up_commands(
            "issue271-live-rocktank",
            &NativePlayOptions {
                fullscreen: false,
                scale: None,
                debug_overlay: false,
                warp_level: Some(0x01),
                settled: true,
                record_to: Some(PathBuf::from(".cache/recordings/issue271-live-rocktank")),
                record_frames: Some(1800),
                record_stop_on_target_enemy_contact: false,
                record_contact_tail_frames: 30,
                record_notes: None,
                record_route_target: Some(RecordingRouteTarget {
                    label: Some(String::from("issue271-live-rocktank")),
                    level: Some(0x01),
                    points: Vec::new(),
                    cells: Vec::new(),
                    cell_hitboxes: Vec::new(),
                    enemies: vec![0x0A, 0x0B],
                }),
                record_frontier_kind: Some(String::from("gameplay")),
                scripted_route: None,
            },
        );

        assert_eq!(
            commands,
            vec![
                "cargo run -p wildside-cli -- verify-tas --scenario .cache/recordings/issue271-live-rocktank/scenario.json --bisect --frontier-kind gameplay",
                "cargo run -p wildside-cli -- probe-record-scenario-file --scenario .cache/recordings/issue271-live-rocktank/scenario.json --target-enemy-contact 0A --summary-only",
                "cargo run -p wildside-cli -- probe-record-scenario-file --scenario .cache/recordings/issue271-live-rocktank/scenario.json --target-enemy-contact 0B --summary-only",
            ]
        );
    }

    #[test]
    fn target_enemy_contact_auto_stop_records_configured_tail_loc_3b464() {
        let mut options = NativePlayOptions {
            fullscreen: false,
            scale: None,
            debug_overlay: false,
            warp_level: Some(0x01),
            settled: true,
            record_to: Some(PathBuf::from(".cache/recordings/issue271-live-rocktank")),
            record_frames: None,
            record_stop_on_target_enemy_contact: true,
            record_contact_tail_frames: 2,
            record_notes: None,
            record_route_target: Some(RecordingRouteTarget {
                label: Some(String::from("issue271-live-rocktank")),
                level: Some(0x01),
                points: Vec::new(),
                cells: Vec::new(),
                cell_hitboxes: Vec::new(),
                enemies: vec![0x0B, 0x0A],
            }),
            record_frontier_kind: Some(String::from("gameplay")),
            scripted_route: None,
        };
        let mut no_contact = sample_trace(GameMode::InGame);
        no_contact.gameplay_debug_fields.insert(
            String::from("kid_collision_order_probe"),
            String::from(
                "enemy_0A#01 gate=separated/result=none/kid=[0,0,0,0]/obj=[0,0,0,0]/overlap=-1,-1/delta=0x00000000,0x00000000",
            ),
        );
        let mut contact = sample_trace(GameMode::InGame);
        contact.gameplay_debug_fields.insert(
            String::from("kid_collision_order_probe"),
            String::from(
                "enemy_0A#01 gate=classify/result=damage/kid=[198,208,185,216]/obj=[190,215,200,216]/overlap=18,16/delta=0x00000000,0x00000000",
            ),
        );
        let mut tail_remaining = None;
        let mut detected_contact = None;

        assert!(!update_target_contact_auto_stop(
            &options,
            &no_contact,
            0,
            &mut tail_remaining,
            &mut detected_contact,
        ));
        assert_eq!(tail_remaining, None);
        assert_eq!(detected_contact, None);
        assert!(!update_target_contact_auto_stop(
            &options,
            &contact,
            1,
            &mut tail_remaining,
            &mut detected_contact,
        ));
        assert_eq!(tail_remaining, Some(1));
        assert_eq!(
            detected_contact,
            Some(TargetEnemyContactStop {
                frame_index: 1,
                object_type: 0x0A,
                result: String::from("damage"),
                gate: String::from("classify"),
                score_px: 17,
            })
        );
        assert!(!update_target_contact_auto_stop(
            &options,
            &no_contact,
            2,
            &mut tail_remaining,
            &mut detected_contact,
        ));
        assert_eq!(tail_remaining, Some(0));
        assert!(update_target_contact_auto_stop(
            &options,
            &no_contact,
            3,
            &mut tail_remaining,
            &mut detected_contact,
        ));

        options.record_contact_tail_frames = 0;
        let mut no_tail = None;
        let mut no_tail_contact = None;
        assert!(update_target_contact_auto_stop(
            &options,
            &contact,
            4,
            &mut no_tail,
            &mut no_tail_contact,
        ));
        assert_eq!(
            no_tail_contact.as_ref().map(|contact| contact.frame_index),
            Some(4)
        );
    }

    #[test]
    fn recording_frame_limit_reached_only_when_recorded_count_hits_limit() {
        let options = NativePlayOptions {
            fullscreen: false,
            scale: None,
            debug_overlay: false,
            warp_level: Some(0x01),
            settled: true,
            record_to: Some(PathBuf::from("recordings/level2-death")),
            record_frames: Some(3),
            record_stop_on_target_enemy_contact: false,
            record_contact_tail_frames: 0,
            record_notes: None,
            record_route_target: None,
            record_frontier_kind: None,
            scripted_route: None,
        };

        assert!(!recording_frame_limit_reached(&options, 2));
        assert!(recording_frame_limit_reached(&options, 3));
        assert!(recording_frame_limit_reached(&options, 4));

        let unbounded = NativePlayOptions {
            record_frames: None,
            ..options
        };
        assert!(!recording_frame_limit_reached(&unbounded, usize::MAX));
    }

    #[test]
    fn play_native_scripted_input_replaces_live_buttons_for_deterministic_audio_routes() {
        let script = [0, 0x80, 1 << 3];

        assert_eq!(native_frame_buttons(None, 1, 1 << 4), 1 << 4);
        assert_eq!(
            native_frame_buttons(Some((&script, false)), 1, 1 << 4),
            0x80
        );
        assert_eq!(
            native_frame_buttons(Some((&script, false)), 2, 1 << 4),
            1 << 3
        );
        assert_eq!(
            native_frame_buttons(Some((&script, false)), 3, 1 << 4),
            0,
            "scripted live-player routes should become idle after the supplied DynamicScenario ends instead of leaking keyboard/controller input"
        );
        assert_eq!(
            native_frame_buttons(Some((&script, true)), 0, 1 << 4),
            0,
            "recording-oracle settled routes delay the first input byte until after the frame-0 state sample"
        );
        assert_eq!(native_frame_buttons(Some((&script, true)), 2, 1 << 4), 0x80);
    }

    #[test]
    fn play_native_debug_warp_waits_for_live_input_release() {
        let mut waiting_for_release = true;

        assert_eq!(
            live_buttons_after_debug_release(1 << 4, &mut waiting_for_release),
            0,
            "debug level warps should not turn held select/jump input into gameplay SFX"
        );
        assert!(waiting_for_release);

        assert_eq!(
            live_buttons_after_debug_release((1 << 3) | (1 << 4), &mut waiting_for_release),
            0,
            "held movement after a level warp should not move Kid before input is released"
        );
        assert!(waiting_for_release);

        assert_eq!(
            live_buttons_after_debug_release(0, &mut waiting_for_release),
            0
        );
        assert!(!waiting_for_release);

        assert_eq!(
            live_buttons_after_debug_release(1 << 3, &mut waiting_for_release),
            1 << 3
        );
    }

    #[test]
    fn level_card_overlay_does_not_restart_after_title_card_bootstrap_handoff() {
        let mut previous = sample_trace(GameMode::LevelTransition);
        previous.boot_phase = Some(String::from("starting_level"));
        let current = sample_trace(GameMode::InGame);

        assert!(
            !should_start_native_level_card_overlay(&previous, &current, None, false),
            "ASM loc_1AE2E -> Load_InGame should not re-enter the native fake level-card overlay after the real Load_TitleCard shell",
        );
    }

    #[test]
    fn level_card_overlay_does_not_start_for_non_title_gameplay_entry() {
        let previous = sample_trace(GameMode::LevelTransition);
        let current = sample_trace(GameMode::InGame);

        assert!(
            !should_start_native_level_card_overlay(&previous, &current, None, false),
            "death/continue LevelTransition -> InGame handoff must not be masked by a native fake loader",
        );
        assert!(!should_start_native_level_card_overlay(
            &previous,
            &current,
            Some(0),
            false
        ));
        assert!(!should_start_native_level_card_overlay(
            &previous, &current, None, true
        ));
    }

    #[test]
    fn level_card_overlay_does_not_mask_level_skip_hold() {
        let mut trace = sample_trace(GameMode::LevelTransition);
        trace.boot_phase = Some(String::from("level_skip_hold"));

        assert!(
            !should_start_native_level_card_overlay_for_level_change(&trace, None, false, false),
            "ASM loc_74E0 level-skip hold/resume path should not be covered by the native placeholder level card",
        );
    }

    #[test]
    fn level_card_overlay_does_not_start_for_ordinary_level_change() {
        let trace = sample_trace(GameMode::LevelTransition);

        assert!(
            !should_start_native_level_card_overlay_for_level_change(&trace, None, false, false),
            "only ROM Load_TitleCard should render a level-card transition",
        );
        assert!(!should_start_native_level_card_overlay_for_level_change(
            &trace,
            Some(0),
            false,
            false
        ));
        assert!(!should_start_native_level_card_overlay_for_level_change(
            &trace, None, true, false
        ));
        assert!(!should_start_native_level_card_overlay_for_level_change(
            &trace, None, false, true
        ));
    }

    #[test]
    fn game_scene_places_native_objects_relative_to_camera() {
        let mut trace = sample_trace(GameMode::InGame);
        trace.active_objects = vec![
            NativeObjectSnapshot {
                kind: NativeObjectKind::Platform {
                    kind: 0,
                    subtype: 0,
                },
                x_pos: 0x0030_0000,
                y_pos: 0x0018_0000,
                layout_index: None,
                x_velocity: None,
                y_velocity: None,
                facing: None,
                on_ground: None,
                hit_points: None,
                damage_flash: false,
                palette_variant: 0,
                stationary: false,
                contact_phase: None,
                shared_walker_state: None,
                shared_walker_patrol_timer: None,
                shared_walker_patrol_short_next: None,
                walk_phase: None,
                wait_timer: None,
                debug_fields: std::collections::BTreeMap::new(),
            },
            NativeObjectSnapshot {
                kind: NativeObjectKind::Enemy { object_type: 0x01 },
                x_pos: 0x0028_0000,
                y_pos: 0x0014_0000,
                layout_index: None,
                x_velocity: None,
                y_velocity: None,
                facing: None,
                on_ground: None,
                hit_points: None,
                damage_flash: false,
                palette_variant: 0,
                stationary: false,
                contact_phase: None,
                shared_walker_state: None,
                shared_walker_patrol_timer: None,
                shared_walker_patrol_short_next: None,
                walk_phase: None,
                wait_timer: None,
                debug_fields: std::collections::BTreeMap::new(),
            },
            NativeObjectSnapshot {
                kind: NativeObjectKind::Player,
                x_pos: 0x0018_0000,
                y_pos: 0x0010_0000,
                layout_index: None,
                x_velocity: None,
                y_velocity: None,
                facing: None,
                on_ground: None,
                hit_points: None,
                damage_flash: false,
                palette_variant: 0,
                stationary: false,
                contact_phase: None,
                shared_walker_state: None,
                shared_walker_patrol_timer: None,
                shared_walker_patrol_short_next: None,
                walk_phase: None,
                wait_timer: None,
                debug_fields: std::collections::BTreeMap::new(),
            },
        ];
        let scene = build_ingame_scene(&trace, true, None);
        assert_eq!(sprite_position(&scene, 41), Some((-4, 16)));
        assert_eq!(sprite_position(&scene, 21), Some((10, 12)));
        assert_eq!(sprite_position(&scene, 1), Some((8, -24)));
    }

    #[test]
    fn rock_tank_renderer_emits_body_and_turret_sprites_per_stru_3b27e() {
        let mut trace = sample_trace(GameMode::InGame);
        trace.frame_index = 720;
        let visuals = sample_level_visuals();
        let object = NativeObjectSnapshot {
            kind: NativeObjectKind::Enemy { object_type: 0x0A },
            x_pos: 0x0040_0000,
            y_pos: 0x0030_0000,
            layout_index: Some(0),
            x_velocity: None,
            y_velocity: None,
            facing: Some(Direction::Right),
            on_ground: None,
            hit_points: Some(2),
            damage_flash: false,
            palette_variant: 0,
            stationary: false,
            contact_phase: None,
            shared_walker_state: None,
            shared_walker_patrol_timer: None,
            shared_walker_patrol_short_next: None,
            walk_phase: None,
            wait_timer: None,
            debug_fields: std::collections::BTreeMap::new(),
        };
        let mut commands = Vec::new();

        sprites::push_enemy_sprite(&mut commands, &object, &trace, &visuals, 0x0A);

        match commands.as_slice() {
            [
                DrawCommand::Sprite {
                    x: body_x,
                    y: body_y,
                    tiles: body_tiles,
                    ..
                },
                DrawCommand::Sprite {
                    x: turret_x,
                    y: turret_y,
                    tiles: turret_tiles,
                    ..
                },
            ] => {
                assert_eq!(
                    body_tiles[0].tile.tile_index, 0x02C,
                    "ASM Tank.asm `stru_3B270` keeps the body on the first GfxObject"
                );
                assert_eq!(
                    turret_tiles[0].tile.tile_index, 0x020,
                    "ASM Tank.asm `stru_3B27E` uses `LnkTo_unk_C7EC8` for the turret idle frame"
                );
                assert_eq!(*turret_x, *body_x);
                assert!(
                    turret_y < body_y,
                    "ASM Tank.asm loc_3B39C copies the parent X/Y into the child and then `subi.w #8,$1E(a1)`, so the turret must render above the body"
                );
                // ASM Tank.asm loc_3B372 stores the turret child at parent_y-8;
                // ASM Make_SpriteFromGfxObject loc_11EE then subtracts the
                // turret frame height. The turret therefore sits 8px above the
                // tread/body GfxObject instead of being compensated back down.
                assert_eq!(
                    *body_y - *turret_y,
                    28,
                    "RockTank turret must render from the linked child baseline, 8px above the body"
                );
            }
            other => panic!("expected body + turret sprite commands, got {other:?}"),
        }
    }

    #[test]
    fn rock_tank_renderer_uses_child_snapshot_instead_of_duplicate_synthetic_turret() {
        let mut trace = sample_trace(GameMode::InGame);
        trace.frame_index = 30;
        let visuals = sample_level_visuals();
        let make_body = |object_type| NativeObjectSnapshot {
            kind: NativeObjectKind::Enemy { object_type },
            x_pos: 0x00B3_8000,
            y_pos: 0x00E0_0000,
            layout_index: Some(1),
            x_velocity: Some(0x0000_8000),
            y_velocity: Some(0),
            facing: Some(Direction::Right),
            on_ground: Some(true),
            hit_points: Some(2),
            damage_flash: false,
            palette_variant: 0,
            stationary: false,
            contact_phase: None,
            shared_walker_state: None,
            shared_walker_patrol_timer: None,
            shared_walker_patrol_short_next: None,
            walk_phase: None,
            wait_timer: None,
            debug_fields: std::collections::BTreeMap::new(),
        };
        let body = make_body(0x0A);
        let child = NativeObjectSnapshot {
            y_pos: 0x00D8_0000,
            hit_points: None,
            on_ground: None,
            ..body.clone()
        };
        trace.active_objects = vec![body.clone(), child.clone()];
        let mut commands = Vec::new();

        sprites::push_enemy_sprite(&mut commands, &body, &trace, &visuals, 0x0A);
        sprites::push_enemy_sprite(&mut commands, &child, &trace, &visuals, 0x0A);

        assert_eq!(
            commands.len(),
            2,
            "ASM Tank.asm loc_3B372 allocates the turret as a child GfxObject; when the trace already contains that child, native rendering must not also synthesise another turret from the body"
        );
        assert_eq!(
            sprite_tiles(&commands[0])[0].tile.tile_index,
            0x034,
            "ASM `stru_3B270` body snapshot renders the RockTank body frame"
        );
        assert_eq!(
            sprite_tiles(&commands[1])[0].tile.tile_index,
            0x020,
            "ASM `stru_3B27E` child snapshot renders the RockTank turret frame"
        );
        match commands.as_slice() {
            [
                DrawCommand::Sprite {
                    x: body_x,
                    y: body_y,
                    ..
                },
                DrawCommand::Sprite {
                    x: turret_x,
                    y: turret_y,
                    ..
                },
            ] => {
                assert_eq!(
                    *turret_x, *body_x,
                    "ASM Tank.asm loc_3B372 copies the parent X into the linked turret child"
                );
                assert_eq!(
                    *body_y - *turret_y,
                    28,
                    "ASM Tank.asm loc_3B372 subtracts 8 from the child Y, and ASM Make_SpriteFromGfxObject loc_11EE still subtracts the child frame height"
                );
            }
            other => panic!("expected body + turret sprite commands, got {other:?}"),
        }

        let shooting_body = make_body(0x0B);
        assert_eq!(
            sprites::select_enemy_frame(
                &trace,
                visuals.enemies.families.get(&0x0B).expect("shooting tank"),
                &shooting_body,
            )
            .tile_base_index,
            0x034,
            "ASM Tank_Shooting.asm `stru_3B4CA` uses the same three live body frames and must not cycle into appended death frames",
        );
    }

    #[test]
    fn rock_tank_renderer_treats_zero_hp_child_snapshot_as_turret_loc_3b372() {
        let mut trace = sample_trace(GameMode::InGame);
        trace.frame_index = 30;
        let visuals = sample_level_visuals();
        let body = NativeObjectSnapshot {
            kind: NativeObjectKind::Enemy { object_type: 0x0A },
            x_pos: 0x00B3_8000,
            y_pos: 0x00E0_0000,
            layout_index: Some(1),
            x_velocity: Some(0x0000_8000),
            y_velocity: Some(0),
            facing: Some(Direction::Right),
            on_ground: Some(true),
            hit_points: Some(2),
            damage_flash: false,
            palette_variant: 0,
            stationary: false,
            contact_phase: None,
            shared_walker_state: None,
            shared_walker_patrol_timer: None,
            shared_walker_patrol_short_next: None,
            walk_phase: None,
            wait_timer: None,
            debug_fields: std::collections::BTreeMap::new(),
        };
        let child = NativeObjectSnapshot {
            y_pos: 0x00D8_0000,
            hit_points: Some(0),
            on_ground: None,
            ..body.clone()
        };
        trace.active_objects = vec![body.clone(), child.clone()];
        let mut commands = Vec::new();

        sprites::push_enemy_sprite(&mut commands, &body, &trace, &visuals, 0x0A);
        sprites::push_enemy_sprite(&mut commands, &child, &trace, &visuals, 0x0A);

        assert_eq!(
            commands.len(),
            2,
            "ASM Tank.asm loc_3B372 creates a linked zeroed GfxObject child; native must not draw a synthetic turret plus a zero-HP death body"
        );
        assert_eq!(
            sprite_tiles(&commands[0])[0].tile.tile_index,
            0x034,
            "live Tank parent still renders the body frame from stru_3B270"
        );
        assert_eq!(
            sprite_tiles(&commands[1])[0].tile.tile_index,
            0x020,
            "zero-HP Tank child should render the turret frame from stru_3B27E, not the parent death frame"
        );
    }

    #[test]
    fn live_rock_tank_animation_does_not_phase_shift_by_layout_index() {
        let mut trace = sample_trace(GameMode::InGame);
        trace.frame_index = 3;
        let visuals = sample_level_visuals();
        let object = NativeObjectSnapshot {
            kind: NativeObjectKind::Enemy { object_type: 0x0A },
            x_pos: 0x0040_0000,
            y_pos: 0x0030_0000,
            layout_index: Some(1),
            x_velocity: None,
            y_velocity: None,
            facing: Some(Direction::Right),
            on_ground: None,
            hit_points: Some(2),
            damage_flash: false,
            palette_variant: 0,
            stationary: false,
            contact_phase: None,
            shared_walker_state: None,
            shared_walker_patrol_timer: None,
            shared_walker_patrol_short_next: None,
            walk_phase: None,
            wait_timer: None,
            debug_fields: std::collections::BTreeMap::new(),
        };
        let family = visuals.enemies.families.get(&0x0A).expect("rock tank");

        assert_eq!(
            sprites::select_enemy_frame(&trace, family, &object).tile_base_index,
            0x02C,
            "ASM Tank.asm loc_3B372 calls j_Init_Animation on stru_3B270 without any layout-index phase offset"
        );

        trace.frame_index = 53;
        let mut commands = Vec::new();
        sprites::push_enemy_sprite(&mut commands, &object, &trace, &visuals, 0x0A);
        assert_eq!(
            sprite_tiles(&commands[1])[0].tile.tile_index,
            0x020,
            "ASM Tank.asm loc_3B372 initialises the child turret with stru_3B27E; layout index should not push the `$37`-tick idle frame into the mouth-open frame"
        );
    }

    #[test]
    fn enemy_sprite_damage_flash_uses_white_palette() {
        let trace = sample_trace(GameMode::InGame);
        let visuals = sample_level_visuals();
        let object = NativeObjectSnapshot {
            kind: NativeObjectKind::Enemy { object_type: 0x0B },
            x_pos: 0x0040_0000,
            y_pos: 0x0030_0000,
            layout_index: Some(0),
            x_velocity: None,
            y_velocity: None,
            facing: Some(Direction::Right),
            on_ground: None,
            hit_points: Some(2),
            damage_flash: true,
            palette_variant: 0,
            stationary: false,
            contact_phase: None,
            shared_walker_state: None,
            shared_walker_patrol_timer: None,
            shared_walker_patrol_short_next: None,
            walk_phase: None,
            wait_timer: None,
            debug_fields: std::collections::BTreeMap::new(),
        };
        let mut commands = Vec::new();

        sprites::push_enemy_sprite(&mut commands, &object, &trace, &visuals, 0x0B);

        let white = MdColour::from_word(0x0FFF);
        assert!(
            sprite_palette(&commands[0])
                .colours
                .iter()
                .all(|colour| *colour == white),
            "non-lethal enemy hits should visibly flash the sprite white instead of only changing invisible HP",
        );
    }

    #[test]
    fn heady_metal_sprite_consumes_palette_buffer_slot_5e_sub_379e8() {
        let trace = sample_trace(GameMode::InGame);
        let mut visuals = sample_level_visuals();
        visuals.enemies.families.insert(
            0x20,
            AnimatedSpriteVisual {
                palette: Palette {
                    colours: (0..16).map(MdColour::from_word).collect(),
                },
                extra_palettes: vec![],
                frames: vec![sample_object_frame(201, 32, 32)],
                frame_duration: 6,
            },
        );
        let object = NativeObjectSnapshot {
            kind: NativeObjectKind::Enemy { object_type: 0x20 },
            x_pos: 0x0040_0000,
            y_pos: 0x0030_0000,
            layout_index: Some(0),
            x_velocity: None,
            y_velocity: None,
            facing: Some(Direction::Right),
            on_ground: None,
            hit_points: Some(30),
            damage_flash: false,
            palette_variant: 0,
            stationary: false,
            contact_phase: None,
            shared_walker_state: None,
            shared_walker_patrol_timer: None,
            shared_walker_patrol_short_next: None,
            walk_phase: None,
            wait_timer: None,
            debug_fields: [(
                "heady_metal_palette_word_5e_sub_379e8".to_string(),
                "0x020a".to_string(),
            )]
            .into(),
        };
        let mut commands = Vec::new();

        sprites::push_enemy_sprite(&mut commands, &object, &trace, &visuals, 0x20);

        assert_eq!(
            sprite_palette(&commands[0]).colours[15],
            MdColour::from_word(0x020A),
            "ASM sub_379E8 writes `unk_379A8[d2]` to Palette_Buffer+$5E, so live HeadyMetal sprites must render with that slot-15 colour"
        );
    }

    #[test]
    fn heady_metal_renderer_marks_sub_37abc_body_piece_positions() {
        let mut trace = sample_trace(GameMode::InGame);
        trace.camera_x_pos = 0;
        trace.camera_y_pos = 0;
        trace.kid_x_pos = Some(0x0050_0000);
        trace.kid_y_pos = Some(0x0050_0000);
        trace.heady_metal_body_pieces_sub_37abc = vec![
            wildside_core::HeadyMetalBodyPieceTraceSub37ABC {
                slot_offset: 0x000C,
                x_pos: 0x0034,
                y_pos: 0x004A,
                sprite_mapping_index: 5,
            },
            wildside_core::HeadyMetalBodyPieceTraceSub37ABC {
                slot_offset: 0x0038,
                x_pos: 0x004C,
                y_pos: 0x009A,
                sprite_mapping_index: 0,
            },
        ];
        let mut visuals = sample_level_visuals();
        visuals.enemies.families.insert(
            0x20,
            AnimatedSpriteVisual {
                palette: Palette {
                    colours: (0..16).map(MdColour::from_word).collect(),
                },
                extra_palettes: vec![],
                frames: vec![sample_object_frame(201, 32, 32)],
                frame_duration: 6,
            },
        );
        let object = NativeObjectSnapshot {
            kind: NativeObjectKind::Enemy { object_type: 0x20 },
            x_pos: 0x0040_0000,
            y_pos: 0x00A0_0000,
            layout_index: Some(0),
            x_velocity: None,
            y_velocity: None,
            facing: Some(Direction::Right),
            on_ground: None,
            hit_points: Some(30),
            damage_flash: false,
            palette_variant: 0,
            stationary: false,
            contact_phase: None,
            shared_walker_state: None,
            shared_walker_patrol_timer: None,
            shared_walker_patrol_short_next: None,
            walk_phase: None,
            wait_timer: None,
            debug_fields: std::collections::BTreeMap::new(),
        };
        let mut commands = Vec::new();

        sprites::push_enemy_sprite(&mut commands, &object, &trace, &visuals, 0x20);

        assert!(
            has_rect_command(&commands, 44, 66, 16, 16, [120, 200, 255]),
            "ASM Enemy20_HeadyMetal_Init seeds the first `$FFFFFB7E` body piece at anchor+(-$C,-$56), and sub_37ABC refreshes it as a visible 16x16 worker"
        );
        assert!(
            has_rect_command(&commands, 68, 146, 16, 16, [120, 200, 255]),
            "ASM Enemy20_HeadyMetal_Init seeds the twelfth `$FFFFFBAA` body piece at anchor+(+$C,-6), and sub_37ABC scans all twelve workers"
        );
    }

    #[test]
    fn rock_tank_child_turret_inherits_parent_damage_flash_palette() {
        let mut trace = sample_trace(GameMode::InGame);
        trace.frame_index = 30;
        let visuals = sample_level_visuals();
        let body = NativeObjectSnapshot {
            kind: NativeObjectKind::Enemy { object_type: 0x0B },
            x_pos: 0x00B3_8000,
            y_pos: 0x00E0_0000,
            layout_index: Some(1),
            x_velocity: Some(0x0000_8000),
            y_velocity: Some(0),
            facing: Some(Direction::Right),
            on_ground: Some(true),
            hit_points: Some(2),
            damage_flash: true,
            palette_variant: 0,
            stationary: false,
            contact_phase: None,
            shared_walker_state: None,
            shared_walker_patrol_timer: None,
            shared_walker_patrol_short_next: None,
            walk_phase: None,
            wait_timer: None,
            debug_fields: std::collections::BTreeMap::new(),
        };
        let child = NativeObjectSnapshot {
            y_pos: 0x00D8_0000,
            hit_points: None,
            damage_flash: false,
            on_ground: None,
            ..body.clone()
        };
        trace.active_objects = vec![body.clone(), child.clone()];
        let mut commands = Vec::new();

        sprites::push_enemy_sprite(&mut commands, &body, &trace, &visuals, 0x0B);
        sprites::push_enemy_sprite(&mut commands, &child, &trace, &visuals, 0x0B);

        let white = MdColour::from_word(0x0FFF);
        assert!(
            sprite_palette(&commands[1])
                .colours
                .iter()
                .all(|colour| *colour == white),
            "ASM Tank_Shooting.asm loc_3B700 recoils the child turret on hurt, so the traced child sprite should share the parent's damage flash",
        );
        match commands.as_slice() {
            [
                DrawCommand::Sprite { y: body_y, .. },
                DrawCommand::Sprite { y: turret_y, .. },
            ] => assert_eq!(
                *body_y - *turret_y,
                20,
                "ASM Tank_Shooting.asm loc_3B5FA uses the same child GfxObject as the plain tank, but native must compensate the shooting turret child offset on the real traced-child path too",
            ),
            other => panic!("expected shooting tank body + child turret, got {other:?}"),
        }
    }

    #[test]
    fn rock_tank_shooting_renderer_uses_stru_3b4d8_turret_frames() {
        let mut trace = sample_trace(GameMode::InGame);
        trace.frame_index = 771;
        let visuals = sample_level_visuals();
        let object = NativeObjectSnapshot {
            kind: NativeObjectKind::Enemy { object_type: 0x0B },
            x_pos: 0x0040_0000,
            y_pos: 0x0030_0000,
            layout_index: Some(0),
            x_velocity: None,
            y_velocity: None,
            facing: Some(Direction::Right),
            on_ground: None,
            hit_points: Some(4),
            damage_flash: false,
            palette_variant: 0,
            stationary: false,
            contact_phase: None,
            shared_walker_state: None,
            shared_walker_patrol_timer: None,
            shared_walker_patrol_short_next: None,
            walk_phase: None,
            wait_timer: None,
            debug_fields: std::collections::BTreeMap::new(),
        };
        let mut commands = Vec::new();

        sprites::push_enemy_sprite(&mut commands, &object, &trace, &visuals, 0x0B);

        match commands.as_slice() {
            [
                DrawCommand::Sprite {
                    x: body_x,
                    y: body_y,
                    tiles: body_tiles,
                    ..
                },
                DrawCommand::Sprite {
                    x: turret_x,
                    y: turret_y,
                    tiles: turret_tiles,
                    ..
                },
            ] => {
                assert_eq!(body_tiles[0].tile.tile_index, 0x034);
                assert_eq!(
                    turret_tiles[0].tile.tile_index, 0x060,
                    "ASM Tank_Shooting.asm `stru_3B4D8` reaches `LnkTo_unk_C7F10` on the muzzle-open frame"
                );
                assert_eq!(
                    *turret_x, *body_x,
                    "ASM Tank_Shooting.asm loc_3B5FA copies the parent X into the linked turret child"
                );
                assert_eq!(
                    *body_y - *turret_y,
                    24,
                    "ASM Tank_Shooting.asm loc_3B5FA stores child Y at body_y-8 for top-anchored hardware sprites; native must compensate that child offset so the muzzle-open turret shares the tread baseline instead of rendering as a second stacked body"
                );
            }
            other => panic!("expected body + turret sprite commands, got {other:?}"),
        }
    }

    #[test]
    fn game_scene_renders_flag_marker_when_visible() {
        let mut trace = sample_trace(GameMode::InGame);
        trace.camera_x_pos = 0x0730_0000;
        trace.camera_y_pos = 0x02D0_0000;

        let scene = build_ingame_scene(&trace, false, None);

        assert!(scene.commands.iter().any(|command| {
            matches!(command, DrawCommand::Sprite { tile_sheet, .. } if tile_sheet.tiles.len() == 120)
        }));
        assert!(scene.commands.iter().any(|command| {
            matches!(command, DrawCommand::Sprite { tile_sheet, .. } if tile_sheet.tiles.len() == 4)
        }));
    }

    #[test]
    fn hud_toggle_removes_debug_overlay() {
        let trace = sample_trace(GameMode::InGame);
        assert!(contains_text(
            &build_ingame_scene(&trace, true, None),
            "Frame"
        ));
        assert!(!contains_text(
            &build_ingame_scene(&trace, false, None),
            "Frame"
        ));
    }

    #[test]
    fn offscreen_objects_clip_without_drawing() {
        let mut trace = sample_trace(GameMode::InGame);
        trace.active_objects = vec![NativeObjectSnapshot {
            kind: NativeObjectKind::Player,
            x_pos: 0,
            y_pos: 0,
            layout_index: None,
            x_velocity: None,
            y_velocity: None,
            facing: None,
            on_ground: None,
            hit_points: None,
            damage_flash: false,
            palette_variant: 0,
            stationary: false,
            contact_phase: None,
            shared_walker_state: None,
            shared_walker_patrol_timer: None,
            shared_walker_patrol_short_next: None,
            walk_phase: None,
            wait_timer: None,
            debug_fields: std::collections::BTreeMap::new(),
        }];
        trace.camera_x_pos = 0x0040_0000;
        trace.camera_y_pos = 0x0040_0000;
        let scene = build_ingame_scene(&trace, false, None);
        let mut rgb = vec![0; (FRAME_WIDTH * FRAME_HEIGHT * 3) as usize];
        render_scene(&scene, &mut rgb);
        assert!(
            !rgb.chunks_exact(3)
                .any(|pixel| pixel == [200, 160, 120].as_slice())
        );
    }

    #[test]
    fn foreground_layout_terrain_renders_visible_cells() {
        let mut trace = sample_trace(GameMode::InGame);
        trace.camera_x_pos = 0;
        trace.camera_y_pos = 0;

        let scene = build_ingame_scene(&trace, false, None);
        for col in 0..2 {
            assert!(
                indexed_tile_at_with_sheet_len(&scene, col * 16, 0, 0x80).is_some(),
                "missing terrain tile art at col {col}"
            );
        }
        assert!(indexed_tile_at_with_sheet_len(&scene, 320, 0, 0x80).is_none());
    }

    #[test]
    fn priority_foreground_terrain_masks_later_low_priority_sprites_loc_3034a() {
        let mut trace = sample_trace(GameMode::InGame);
        trace.camera_x_pos = 0;
        trace.camera_y_pos = 0;
        let mut visuals = sample_level_visuals();
        visuals.foreground_tiles.tiles[0].pixels = [1; 64];
        visuals.palettes.foreground = Palette {
            colours: vec![colour(20, 30, 40), colour(180, 190, 200)],
        };
        let low_tile_sheet = Arc::new(TileSheet {
            tiles: vec![make_tile(&[(0, 0, 1)])],
        });
        let low_palette = Arc::new(Palette {
            colours: vec![colour(220, 20, 20)],
        });

        let mut priority_layout = [0; 5 * 2];
        priority_layout[4 * 2] = 0x01;
        let priority_grid = BlockGrid::from_level_layout(&priority_layout, 1, 5);
        let mut priority_scene = build_scene(&trace, false, Some(&visuals), Some(&priority_grid));
        priority_scene.commands.push(DrawCommand::IndexedTile {
            x: 0,
            y: 64,
            tile_sheet: Arc::clone(&low_tile_sheet),
            tile: TileRef {
                tile_index: 0,
                palette_line: 0,
                priority: false,
                flip_x: false,
                flip_y: false,
            },
            palette: Arc::clone(&low_palette),
        });
        priority_scene.commands.push(DrawCommand::Sprite {
            x: 0,
            y: 64,
            tile_sheet: Arc::clone(&low_tile_sheet),
            palette: Arc::clone(&low_palette),
            tiles: vec![SpriteTile {
                offset_x: 0,
                offset_y: 0,
                tile: TileRef {
                    tile_index: 0,
                    palette_line: 0,
                    priority: false,
                    flip_x: false,
                    flip_y: false,
                },
            }],
        });
        let priority_sample = sample_scene_pixel_provenance(
            &priority_scene,
            &[(0usize, 64usize)].into_iter().collect(),
        )
        .into_iter()
        .next()
        .expect("priority sample");
        assert!(
            priority_sample.final_high_priority,
            "ASM sub_3034A/sub_302F6 OR `$8000` into terrain tile words when Level_Layout carries bit `$0100`"
        );
        assert!(
            priority_sample
                .writes
                .iter()
                .any(|write| write.kind == "indexed_tile"
                    && write.rgb == Some([220, 20, 20])
                    && !write.accepted),
            "later ordinary Kid sprites are low priority, so Highwater foreground/water mask cells must reject them"
        );
        assert!(
            priority_sample
                .writes
                .iter()
                .any(|write| write.kind == "sprite_tile"
                    && write.rgb == Some([220, 20, 20])
                    && !write.accepted),
            "ordinary enemy/player DrawCommand::Sprite tiles must follow the same foreground-priority mask, not just terrain-tile commands"
        );

        let low_grid = BlockGrid::from_level_layout(&[0; 5 * 2], 1, 5);
        let mut low_scene = build_scene(&trace, false, Some(&visuals), Some(&low_grid));
        low_scene.commands.push(DrawCommand::IndexedTile {
            x: 0,
            y: 64,
            tile_sheet: Arc::clone(&low_tile_sheet),
            tile: TileRef {
                tile_index: 0,
                palette_line: 0,
                priority: false,
                flip_x: false,
                flip_y: false,
            },
            palette: low_palette,
        });
        low_scene.commands.push(DrawCommand::Sprite {
            x: 0,
            y: 64,
            tile_sheet: Arc::clone(&low_tile_sheet),
            palette: Arc::new(Palette {
                colours: vec![colour(20, 220, 20)],
            }),
            tiles: vec![SpriteTile {
                offset_x: 0,
                offset_y: 0,
                tile: TileRef {
                    tile_index: 0,
                    palette_line: 0,
                    priority: false,
                    flip_x: false,
                    flip_y: false,
                },
            }],
        });
        let low_sample =
            sample_scene_pixel_provenance(&low_scene, &[(0usize, 64usize)].into_iter().collect())
                .into_iter()
                .next()
                .expect("low-priority sample");
        assert!(
            low_sample
                .writes
                .iter()
                .any(|write| write.kind == "indexed_tile"
                    && write.rgb == Some([220, 20, 20])
                    && write.accepted),
            "ordinary terrain cells stay low priority; this regression only covers the ROM `$0100` foreground mask bit"
        );
        assert!(
            low_sample
                .writes
                .iter()
                .any(|write| write.kind == "sprite_tile"
                    && write.rgb == Some([20, 220, 20])
                    && write.accepted),
            "sprite tiles should still draw normally when the foreground cell has no ROM priority bit"
        );
    }

    #[test]
    fn block_grid_only_draws_visible_block_overrides() {
        let mut trace = sample_trace(GameMode::InGame);
        // Camera at (320px, 0) — first 20 columns offscreen
        trace.camera_x_pos = 0x0140_0000;
        trace.camera_y_pos = 0;

        // 40x2 grid: bottom row all solid
        let w = 40u16;
        let h = 2u16;
        let mut data = vec![0u8; usize::from(w) * usize::from(h) * 2];
        for col in 0..40 {
            let offset = (40 + col) * 2;
            data[offset] = 0xE1;
        }
        let grid = BlockGrid::from_level_layout(&data, w, h);

        let scene = build_ingame_scene(&trace, false, Some(&grid));
        let offscreen_tiles = scene.commands.iter().filter(|cmd| {
            matches!(cmd, DrawCommand::IndexedTile { x, tile_sheet, .. }
                if tile_sheet.tiles.len() == 0x200 && *x < -8)
        });
        assert_eq!(
            offscreen_tiles.count(),
            0,
            "offscreen block overrides should not be drawn"
        );
    }

    #[test]
    fn destroyed_block_reveals_foreground_terrain() {
        let mut trace = sample_trace(GameMode::InGame);
        trace.camera_x_pos = 0;
        trace.camera_y_pos = 0;

        // 2x2 grid: one prize block at (1,0), rest empty. The sample
        // foreground layout has immutable skin 3 at (1,0); a cleared
        // runtime block-grid cell has skin 0.
        let mut data = vec![0u8; 2 * 2 * 2];
        data[2] = 0xE1; // block=true, collision=Solid, block_type=Prize
        let mut grid = BlockGrid::from_level_layout(&data, 2, 2);

        // Before destruction: the live prize cell is visible from
        // Block_Mappings and does not also reveal immutable foreground art.
        let scene = build_ingame_scene(&trace, false, Some(&grid));
        assert!(indexed_tile_at_with_sheet_len(&scene, 16, 0, 0x200).is_some());
        assert!(indexed_tile_at_with_sheet_len(&scene, 16, 0, 0x80).is_none());

        // Destroy the block override; the immutable terrain art should remain.
        grid.set_cell_at(1, 0, Cell::from_be_bytes(0, 0));
        let scene = build_ingame_scene(&trace, false, Some(&grid));
        assert!(indexed_tile_at_with_sheet_len(&scene, 16, 0, 0x200).is_none());
        let Some((_, tile, _)) = indexed_tile_at_with_sheet_len(&scene, 16, 0, 0x80) else {
            panic!("expected foreground terrain after the block override was cleared");
        };
        assert_eq!(
            tile.tile_index,
            3 * 4,
            "clearing a live block cell must reveal immutable foreground skin 3, not the cleared cell's skin 0"
        );
    }

    #[test]
    fn prize_block_lifecycle_restores_smash_family_art_after_dispatch() {
        let mut trace = sample_trace(GameMode::InGame);
        trace.camera_x_pos = 0;
        trace.camera_y_pos = 0;

        let data = [0xE0, 0x01];
        let grid = BlockGrid::from_level_layout(&data, 1, 1);

        let scene = build_ingame_scene(&trace, false, Some(&grid));

        let Some((tile_sheet, tile, _)) =
            topmost_indexed_tile_at_with_sheet_len(&scene, 0, 0, 0x200)
        else {
            panic!("expected smash-family block art at the lifecycle-restored cell");
        };
        assert_eq!(tile_sheet.tiles.len(), 0x200);
        assert_eq!(
            tile.tile_index, 4,
            "ASM sub_30194 sets d4=$FF before sub_302F6/sub_3034A, so post-prize $E001 must use skin-byte block quad 1"
        );
    }

    #[test]
    fn block_types_share_tile_mapping_when_skin_matches() {
        let visuals = sample_level_visuals();
        let tile_indices: Vec<_> = [Some(0), Some(1), Some(2), Some(6), Some(7)]
            .iter()
            .map(|block_type| {
                let resolved = visuals
                    .level
                    .block_mapping
                    .resolve(0, *block_type)
                    .expect("block mapping");
                resolve_ingame_vram_tile(&visuals, resolved.quad.tiles[0])
                    .expect("scene tile")
                    .tile
                    .tile_index
            })
            .collect();
        for tile_index in tile_indices {
            assert_eq!(
                tile_index, 0,
                "ASM sub_30194 masks block cells with d4=$FF, so block_type cannot change the Block_Mappings tile when skin is unchanged"
            );
        }
    }

    // --- US-018: Invulnerability flash ---

    fn trace_with_player(frame_index: usize, invulnerable_frames: usize) -> NativeTraceFrame {
        let mut trace = sample_trace(GameMode::InGame);
        trace.frame_index = frame_index;
        trace.invulnerable_frames = invulnerable_frames;
        trace.just_received_damage = invulnerable_frames > 0;
        trace.kid_x_velocity = Some(0);
        trace.kid_y_velocity = Some(0);
        trace.kid_facing = Some(Direction::Right);
        trace.kid_on_ground = Some(true);
        trace.active_objects = vec![NativeObjectSnapshot {
            kind: NativeObjectKind::Player,
            x_pos: 0x0018_0000,
            y_pos: 0x0010_0000,
            layout_index: None,
            x_velocity: None,
            y_velocity: None,
            facing: None,
            on_ground: None,
            hit_points: None,
            damage_flash: false,
            palette_variant: 0,
            stationary: false,
            contact_phase: None,
            shared_walker_state: None,
            shared_walker_patrol_timer: None,
            shared_walker_patrol_short_next: None,
            walk_phase: None,
            wait_timer: None,
            debug_fields: std::collections::BTreeMap::new(),
        }];
        trace
    }

    fn scene_has_player_sprite(scene: &Scene) -> bool {
        player_sprite_signature(scene).is_some()
    }

    fn player_sprite_palette(scene: &Scene) -> Option<&Palette> {
        scene.commands.iter().find_map(|command| match command {
            DrawCommand::Sprite {
                tile_sheet,
                palette,
                ..
            } => tile_sheet
                .tiles
                .first()
                .map(|tile| tile.pixels[0])
                .filter(|signature| is_player_test_signature(*signature))
                .map(|_| palette.as_ref()),
            _ => None,
        })
    }

    fn is_player_test_signature(signature: u8) -> bool {
        (1..=55).contains(&signature)
    }

    fn player_sprite_signature(scene: &Scene) -> Option<u8> {
        scene.commands.iter().find_map(|command| match command {
            DrawCommand::Sprite { tile_sheet, .. } => tile_sheet
                .tiles
                .first()
                .map(|tile| tile.pixels[0])
                .filter(|signature| is_player_test_signature(*signature)),
            _ => None,
        })
    }

    fn player_sprite_first_flip_x(scene: &Scene) -> Option<bool> {
        scene.commands.iter().find_map(|command| match command {
            DrawCommand::Sprite {
                tiles, tile_sheet, ..
            } if tile_sheet
                .tiles
                .first()
                .is_some_and(|tile| is_player_test_signature(tile.pixels[0])) =>
            {
                tiles.first().map(|tile| tile.tile.flip_x)
            }
            _ => None,
        })
    }

    #[test]
    fn player_form_uses_iron_knight_sprite_art_when_helmet_equipped() {
        let mut trace = trace_with_player(720, 0);
        trace.current_helmet_id = PlayerForm::IronKnight.helmet_id();

        let scene = build_ingame_scene(&trace, false, None);

        assert_eq!(player_sprite_signature(&scene), Some(10));
    }

    #[test]
    fn iron_knight_attach_hibernate_preserves_pre_sub_a3bc_pose_loc_a276() {
        let mut trace = trace_with_player(720, 0);
        trace.current_helmet_id = PlayerForm::IronKnight.helmet_id();
        trace.kid_on_ground = Some(false);
        trace.kid_x_velocity = Some(0);
        trace.kid_y_velocity = Some(-0x4000);
        trace.iron_knight_climbing = true;
        trace.iron_knight_climb_pose_accum = 0;

        let scene = build_ingame_scene(&trace, false, None);

        assert_eq!(
            player_sprite_signature(&scene),
            Some(17),
            "ASM loc_A276 seeds wall climb and hibernates before sub_A3BC writes off_A420, so the attach frame must not force iron_knight_climbing_3"
        );
    }

    #[test]
    fn iron_knight_resume_climb_uses_sub_a3bc_pose_after_attach_loc_a330() {
        let mut trace = trace_with_player(720, 0);
        trace.current_helmet_id = PlayerForm::IronKnight.helmet_id();
        trace.kid_on_ground = Some(false);
        trace.kid_x_velocity = Some(0);
        trace.kid_y_velocity = Some(-0x3F80);
        trace.iron_knight_climbing = true;
        trace.iron_knight_climb_pose_accum = 0;

        let scene = build_ingame_scene(&trace, false, None);

        assert_eq!(
            player_sprite_signature(&scene),
            Some(50),
            "ASM loc_A330 calls sub_A3BC on the resume loop, so a post-attach climb frame should still use the bottom off_A420 pose"
        );
    }

    #[test]
    fn player_form_change_prefers_transform_sprite_over_placeholder_flash() {
        let mut trace = trace_with_player(720, 0);
        trace.current_helmet_id = PlayerForm::IronKnight.helmet_id();
        trace.form_change_flash_frames = 12;

        let scene = build_ingame_scene(&trace, false, None);

        assert_eq!(player_sprite_signature(&scene), Some(19));
        assert!(!scene.commands.iter().any(|command| {
            matches!(
                command,
                DrawCommand::Rect {
                    colour: [255, 255, 255],
                    ..
                }
            ) || matches!(
                command,
                DrawCommand::Rect {
                    colour: [255, 255, 128],
                    ..
                }
            )
        }));
    }

    #[test]
    fn loc_7cd0_cyclone_transform_uses_kid_palette_for_old_kid_walk_frame() {
        let mut trace = trace_with_player(720, 0);
        trace.current_helmet_id = PlayerForm::Cyclone.helmet_id();
        trace.form_change_flash_frames = 20;

        let scene = build_ingame_scene(&trace, false, None);

        assert_eq!(
            player_sprite_signature(&scene),
            Some(2),
            "ASM Kid_Transform loc_7CD0: first-helmet transform starts on Kid's old-form frame before target helmet art"
        );
        assert_eq!(
            player_sprite_palette(&scene).map(|palette| palette.colours[1]),
            Some(colour(200, 160, 120)),
            "ASM loc_7CD0 switches Current_Helmet later, so Kid art must not use Cyclone's dark transform palette"
        );
    }

    #[test]
    fn red_stealth_sword_timer_selects_ground_slash_rom_frame_loc_76b0_stru_8b3c() {
        let mut trace = trace_with_player(720, 0);
        trace.current_helmet_id = PlayerForm::RedStealth.helmet_id();
        trace.red_stealth_sword_timer = 11;
        trace.kid_on_ground = Some(true);
        trace.kid_x_velocity = Some(0);

        let scene = build_ingame_scene(&trace, false, None);

        assert_eq!(
            player_sprite_signature(&scene),
            Some(30),
            "ASM loc_76B0 initialises Red Stealth sword animation `stru_8B3C`, whose first rendered frame is `unk_B3520`",
        );
    }

    #[test]
    fn grounded_red_stealth_swing_ignores_stale_airborne_slash_counter_loc_76b0_78e8() {
        let mut trace = trace_with_player(720, 0);
        trace.current_helmet_id = PlayerForm::RedStealth.helmet_id();
        trace.red_stealth_sword_timer = 11;
        trace.red_stealth_airborne_slash_counter = 0x40;
        trace.kid_on_ground = Some(true);
        trace.kid_x_velocity = Some(0);

        let scene = build_ingame_scene(&trace, false, None);

        assert_eq!(
            player_sprite_signature(&scene),
            Some(30),
            "ASM loc_76B0/stru_8B3C owns grounded sword art; sub_78E8 loc_794C clears the shared Red_Stealth_sword_swing byte on grounded/default recovery",
        );
    }

    #[test]
    fn red_stealth_sword_timer_selects_air_slash_rom_frame_sub_b270() {
        let mut trace = trace_with_player(720, 0);
        trace.current_helmet_id = PlayerForm::RedStealth.helmet_id();
        trace.red_stealth_sword_timer = 8;
        trace.kid_on_ground = Some(false);
        trace.kid_y_velocity = Some(0x0002_0000);

        let scene = build_ingame_scene(&trace, false, None);

        assert_eq!(
            player_sprite_signature(&scene),
            Some(35),
            "ASM sub_B270 buckets the Red Stealth airborne slash through `unk_B40DE..unk_B4870`",
        );
    }

    #[test]
    fn cyclone_flight_uses_spin_frame_sub_b270_loc_b2a0() {
        let mut trace = trace_with_player(720, 0);
        trace.current_helmet_id = PlayerForm::Cyclone.helmet_id();
        trace.cyclone_flying = true;
        trace.kid_on_ground = Some(false);
        trace.kid_y_velocity = Some(0x0001_0000);
        trace.walk_visual_accumulator = 0x0100;

        let scene = build_ingame_scene(&trace, false, None);

        assert_eq!(
            player_sprite_signature(&scene),
            Some(54),
            "ASM sub_B270 loc_B2A0 stores the Cyclone flight phase in `$FFFFF8F0`; phase $0100 selects `unk_BA7E0` / cyclone_spinning_2"
        );
    }

    #[test]
    fn grounded_cyclone_keeps_walk_table_when_not_flying_loc_9600() {
        let mut trace = trace_with_player(720, 0);
        trace.current_helmet_id = PlayerForm::Cyclone.helmet_id();
        trace.cyclone_flying = false;
        trace.kid_on_ground = Some(true);
        trace.kid_x_velocity = Some(0x0001_0000);
        trace.walk_visual_accumulator = 0;

        let scene = build_ingame_scene(&trace, false, None);

        assert_eq!(
            player_sprite_signature(&scene),
            Some(49),
            "ASM loc_9600/off_96D8 owns grounded Cyclone walking; the sub_B270 flight spin branch must stay gated by Cyclone_flying"
        );
    }

    #[test]
    fn red_stealth_on_slope_selects_rom_tilted_frame_off_a23c() {
        let mut trace = trace_with_player(4, 0);
        trace.current_helmet_id = PlayerForm::RedStealth.helmet_id();
        trace.kid_x_velocity = Some(0x0001_0000);
        trace.kid_on_ground = Some(true);
        trace.kid_on_slope = 0x40;

        let scene = build_ingame_scene(&trace, false, None);

        assert_eq!(
            player_sprite_signature(&scene),
            Some(40),
            "ASM loc_A13A/off_A190 selects helmet 3's first bank at off_A230, whose first entry is `unk_BA0CE`; the sample fixture maps that slot to signature 40"
        );
    }

    #[test]
    fn red_stealth_zero_velocity_slope_still_selects_loc_a13a_frame() {
        let mut trace = trace_with_player(4, 0);
        trace.current_helmet_id = PlayerForm::RedStealth.helmet_id();
        trace.kid_x_velocity = Some(0);
        trace.kid_on_ground = Some(true);
        trace.kid_on_slope = 1;
        trace.kid_facing = Some(Direction::Right);

        let scene = build_ingame_scene(&trace, false, None);

        assert_eq!(
            player_sprite_signature(&scene),
            Some(40),
            "ASM loc_A13A/off_A190 writes a slope sprite even when x_vel is zero; native must not fall back to idle on wedged Samurai slope frames"
        );
    }

    #[test]
    fn red_stealth_slope_frame_phase_uses_walk_visual_accumulator_loc_a13a() {
        let mut trace = trace_with_player(4, 0);
        trace.current_helmet_id = PlayerForm::RedStealth.helmet_id();
        trace.kid_x_velocity = Some(0);
        trace.kid_on_ground = Some(true);
        trace.kid_on_slope = 1;
        trace.kid_facing = Some(Direction::Right);
        trace.walk_visual_accumulator = 0x0100;

        assert_eq!(
            player_sprite_signature(&build_ingame_scene(&trace, false, None)),
            Some(41),
            "ASM loc_A13A/loc_A17E indexes Red Stealth slope frames with `$FFFFF8F0 >> 8`; wedged Samurai slope frames must not be driven by frame time or idle/walk state"
        );
    }

    #[test]
    fn red_stealth_slope_render_flip_accounts_for_predrawn_loc_a13a_banks() {
        let selected_flip = |slope: i8, facing: Direction| {
            let mut trace = trace_with_player(4, 0);
            trace.current_helmet_id = PlayerForm::RedStealth.helmet_id();
            trace.kid_x_velocity = Some(match facing {
                Direction::Right => 0x0001_0000,
                Direction::Left => -0x0001_0000,
            });
            trace.kid_on_ground = Some(true);
            trace.kid_on_slope = slope;
            trace.kid_facing = Some(facing);

            player_sprite_first_flip_x(&build_ingame_scene(&trace, false, None))
        };

        assert_eq!(
            selected_flip(1, Direction::Right),
            Some(false),
            "ASM loc_A13A selects Red Stealth's first slope bank, then loc_1292 leaves clear x_direction unflipped"
        );
        assert_eq!(
            selected_flip(1, Direction::Left),
            Some(true),
            "ASM loc_A13A selects Red Stealth's right-facing reverse bank before loc_1292's left-facing draw; native must mirror it"
        );
        assert_eq!(
            selected_flip(-1, Direction::Right),
            Some(false),
            "ASM loc_A13A selects Red Stealth's right-facing reverse bank on SlopeDown + clear x_direction; loc_1292 leaves it unflipped"
        );
        assert_eq!(
            selected_flip(-1, Direction::Left),
            Some(true),
            "ASM loc_A13A canonicalises SlopeDown before bank selection, but loc_1292 still mirrors set x_direction"
        );
    }

    #[test]
    fn kid_slope_frame_bank_follows_loc_a13a_fa26_canonical_direction() {
        let selected_signature = |slope: i8, facing: Direction| {
            let mut trace = trace_with_player(4, 0);
            trace.kid_x_velocity = Some(match facing {
                Direction::Right => 0x0001_0000,
                Direction::Left => -0x0001_0000,
            });
            trace.kid_on_ground = Some(true);
            trace.kid_on_slope = slope;
            trace.kid_facing = Some(facing);

            player_sprite_signature(&build_ingame_scene(&trace, false, None))
        };

        assert_eq!(
            selected_signature(1, Direction::Right),
            Some(31),
            "ASM loc_A13A: SlopeUp ($FA26=0) keeps x_direction=0, selecting the first six-entry Kid slope table (`kid_slope_1..6`) at `$FFFFF8F0 >> 8 == 0`"
        );
        assert_eq!(
            selected_signature(1, Direction::Left),
            Some(25),
            "ASM loc_A13A+loc_A16C: SlopeUp keeps x_direction=$FF, selecting the second six-entry Kid slope table (`kid_slope_7..12`) at `$FFFFF8F0 >> 8 == 0`"
        );
        assert_eq!(
            selected_signature(-1, Direction::Right),
            Some(25),
            "ASM loc_A13A: SlopeDown ($FA26=$FF) flips x_direction=0 to canonical reverse before table selection at `$FFFFF8F0 >> 8 == 0`"
        );
        assert_eq!(
            selected_signature(-1, Direction::Left),
            Some(31),
            "ASM loc_A13A: SlopeDown ($FA26=$FF) flips x_direction=$FF to canonical forward before table selection at `$FFFFF8F0 >> 8 == 0`"
        );
    }

    #[test]
    fn kid_slope_frame_phase_uses_walk_visual_accumulator_loc_a13a() {
        let mut trace = trace_with_player(4, 0);
        trace.kid_x_velocity = Some(0x0001_0000);
        trace.kid_on_ground = Some(true);
        trace.kid_on_slope = 1;
        trace.kid_facing = Some(Direction::Right);
        trace.walk_visual_accumulator = 0x0308;

        assert_eq!(
            player_sprite_signature(&build_ingame_scene(&trace, false, None)),
            Some(34),
            "ASM loc_A13A/loc_A17E indexes slope frames with `$FFFFF8F0 >> 8`; native must not drive slope poses from frame time"
        );
    }

    #[test]
    fn kid_slope_render_flip_accounts_for_predrawn_loc_a13a_banks() {
        let selected_flip = |slope: i8, facing: Direction| {
            let mut trace = trace_with_player(4, 0);
            trace.kid_x_velocity = Some(match facing {
                Direction::Right => 0x0001_0000,
                Direction::Left => -0x0001_0000,
            });
            trace.kid_on_ground = Some(true);
            trace.kid_on_slope = slope;
            trace.kid_facing = Some(facing);

            player_sprite_first_flip_x(&build_ingame_scene(&trace, false, None))
        };

        assert_eq!(
            selected_flip(1, Direction::Right),
            Some(false),
            "ASM loc_A13A selects Kid's first slope bank (`kid_slope_1..6`), then loc_1292 draws clear x_direction through the unflipped sprite path"
        );
        assert_eq!(
            selected_flip(1, Direction::Left),
            Some(true),
            "ASM loc_A13A/off_A1B8 selects Kid's second slope bank (`kid_slope_7..12`), then loc_1292 mirrors it for set x_direction"
        );
        assert_eq!(
            selected_flip(-1, Direction::Right),
            Some(false),
            "ASM loc_A13A selects the right-facing `kid_slope_7..12` bank on SlopeDown + clear x_direction; loc_1292 leaves it unflipped"
        );
        assert_eq!(
            selected_flip(-1, Direction::Left),
            Some(true),
            "ASM loc_A13A selects the left-facing `kid_slope_1..6` bank on SlopeDown + set x_direction; loc_1292 still mirrors set x_direction"
        );
    }

    fn sprite_position(scene: &Scene, signature: u8) -> Option<(i32, i32)> {
        scene.commands.iter().find_map(|command| match command {
            DrawCommand::Sprite {
                x, y, tile_sheet, ..
            } if tile_sheet
                .tiles
                .first()
                .is_some_and(|tile| tile.pixels[0] == signature) =>
            {
                Some((*x, *y))
            }
            _ => None,
        })
    }

    fn sprite_positions(scene: &Scene, signature: u8) -> Vec<(i32, i32)> {
        let mut positions = scene
            .commands
            .iter()
            .filter_map(|command| match command {
                DrawCommand::Sprite {
                    x, y, tile_sheet, ..
                } if tile_sheet
                    .tiles
                    .first()
                    .is_some_and(|tile| tile.pixels[0] == signature) =>
                {
                    Some((*x, *y))
                }
                _ => None,
            })
            .collect::<Vec<_>>();
        positions.sort_unstable();
        positions
    }

    fn has_sprite_signature(scene: &Scene, signature: u8) -> bool {
        scene.commands.iter().any(|command| match command {
            DrawCommand::Sprite {
                tile_sheet, tiles, ..
            } => {
                tiles.iter().any(|sprite_tile| {
                    tile_sheet
                        .tiles
                        .get(usize::from(sprite_tile.tile.tile_index))
                        .is_some_and(|tile| tile.pixels[0] == signature)
                }) || tile_sheet
                    .tiles
                    .first()
                    .is_some_and(|tile| tile.pixels[0] == signature)
            }
            _ => false,
        })
    }

    fn has_command_signature(command: &DrawCommand, signature: u8) -> bool {
        match command {
            DrawCommand::IndexedTile {
                tile_sheet, tile, ..
            } => tile_sheet
                .tiles
                .get(usize::from(tile.tile_index))
                .is_some_and(|resolved| resolved.pixels[0] == signature),
            DrawCommand::Sprite {
                tile_sheet, tiles, ..
            } => {
                tiles.iter().any(|sprite_tile| {
                    tile_sheet
                        .tiles
                        .get(usize::from(sprite_tile.tile.tile_index))
                        .is_some_and(|resolved| resolved.pixels[0] == signature)
                }) || tile_sheet
                    .tiles
                    .first()
                    .is_some_and(|resolved| resolved.pixels[0] == signature)
            }
            _ => false,
        }
    }

    fn has_asset_backed_command(scene: &Scene, signature: u8) -> bool {
        scene
            .commands
            .iter()
            .any(|command| has_command_signature(command, signature))
    }

    fn platform_snapshot(
        kind: u8,
        subtype: u8,
        x_px: u32,
        y_px: u32,
        debug_fields: std::collections::BTreeMap<String, String>,
    ) -> NativeObjectSnapshot {
        NativeObjectSnapshot {
            kind: NativeObjectKind::Platform { kind, subtype },
            x_pos: x_px << 16,
            y_pos: y_px << 16,
            layout_index: None,
            x_velocity: None,
            y_velocity: None,
            facing: None,
            on_ground: None,
            hit_points: None,
            damage_flash: false,
            palette_variant: 0,
            stationary: false,
            contact_phase: None,
            shared_walker_state: None,
            shared_walker_patrol_timer: None,
            shared_walker_patrol_short_next: None,
            walk_phase: None,
            wait_timer: None,
            debug_fields,
        }
    }

    #[test]
    fn build_game_scene_emits_enemy_sprite_commands_for_visible_level1_families() {
        let mut trace = sample_trace(GameMode::InGame);
        trace.camera_x_pos = 0;
        trace.camera_y_pos = 0;
        trace.active_objects = vec![
            NativeObjectSnapshot {
                kind: NativeObjectKind::Enemy { object_type: 0x01 },
                x_pos: 0x0028_0000,
                y_pos: 0x0010_0000,
                layout_index: None,
                x_velocity: None,
                y_velocity: None,
                facing: None,
                on_ground: None,
                hit_points: None,
                damage_flash: false,
                palette_variant: 0,
                stationary: false,
                contact_phase: None,
                shared_walker_state: None,
                shared_walker_patrol_timer: None,
                shared_walker_patrol_short_next: None,
                walk_phase: None,
                wait_timer: None,
                debug_fields: std::collections::BTreeMap::new(),
            },
            NativeObjectSnapshot {
                kind: NativeObjectKind::Enemy { object_type: 0x1C },
                x_pos: 0x0040_0000,
                y_pos: 0x0010_0000,
                layout_index: None,
                x_velocity: None,
                y_velocity: None,
                facing: None,
                on_ground: None,
                hit_points: None,
                damage_flash: false,
                palette_variant: 0,
                stationary: false,
                contact_phase: None,
                shared_walker_state: None,
                shared_walker_patrol_timer: None,
                shared_walker_patrol_short_next: None,
                walk_phase: None,
                wait_timer: None,
                debug_fields: std::collections::BTreeMap::new(),
            },
        ];

        let scene = build_ingame_scene(&trace, false, None);

        assert!(
            has_sprite_signature(&scene, 21),
            "expected diamond sprite art"
        );
        assert!(
            has_sprite_signature(&scene, 31) || has_sprite_signature(&scene, 32),
            "expected mini hopping skull sprite art"
        );
        assert!(!scene.commands.iter().any(|command| {
            matches!(command, DrawCommand::Rect { colour, .. } if *colour == ENEMY_COLOUR || *colour == ENEMY_FLASH_COLOUR)
        }));
    }

    #[test]
    fn mini_hopping_skull_death_snapshot_selects_loc_3f158_frames() {
        let visuals = sample_level_visuals();
        let skull_visuals = visuals.enemies.families.get(&0x1C).expect("skull visuals");
        let mut object = NativeObjectSnapshot {
            kind: NativeObjectKind::Enemy { object_type: 0x1C },
            x_pos: 0x0040_0000,
            y_pos: 0x0010_0000,
            layout_index: Some(0),
            x_velocity: Some(0),
            y_velocity: Some(0),
            facing: Some(Direction::Right),
            on_ground: Some(true),
            hit_points: Some(0),
            damage_flash: false,
            palette_variant: 0,
            stationary: true,
            contact_phase: None,
            shared_walker_state: None,
            shared_walker_patrol_timer: None,
            shared_walker_patrol_short_next: None,
            walk_phase: None,
            wait_timer: Some(0x0A + 0x1E),
            debug_fields: std::collections::BTreeMap::new(),
        };
        let trace = sample_trace(GameMode::InGame);

        assert_eq!(
            sprites::select_enemy_frame(&trace, skull_visuals, &object)
                .tile_sheet
                .tiles[0]
                .pixels[0],
            37,
            "ASM Mini_Hopping_Skull.asm loc_3F158 should render LnkTo_unk_C8040 during the first $0A ticks"
        );

        object.wait_timer = Some(0x1E);
        assert_eq!(
            sprites::select_enemy_frame(&trace, skull_visuals, &object)
                .tile_sheet
                .tiles[0]
                .pixels[0],
            38,
            "ASM Mini_Hopping_Skull.asm loc_3F158 should render LnkTo_unk_C8048 for the final $1E ticks"
        );
    }

    #[test]
    fn boss_head_death_snapshot_selects_loc_39ebe_spin_frames() {
        let visuals = sample_level_visuals();
        let boss_visuals = visuals
            .enemies
            .families
            .get(&0x22)
            .expect("boss head visuals");
        let mut object = NativeObjectSnapshot {
            kind: NativeObjectKind::Enemy { object_type: 0x22 },
            x_pos: 0x0040_0000,
            y_pos: 0x0010_0000,
            layout_index: Some(0),
            x_velocity: Some(0),
            y_velocity: Some(0),
            facing: Some(Direction::Right),
            on_ground: Some(true),
            hit_points: Some(0),
            damage_flash: false,
            palette_variant: 0,
            stationary: true,
            contact_phase: None,
            shared_walker_state: None,
            shared_walker_patrol_timer: None,
            shared_walker_patrol_short_next: None,
            walk_phase: None,
            wait_timer: Some(120 + 10 * 17),
            debug_fields: std::collections::BTreeMap::new(),
        };
        let trace = sample_trace(GameMode::InGame);

        for (remaining, signature, tile_base) in [
            (120 + 10 * 17, 225, 0x1A4),
            (120 + 10 * 17 - 6, 224, 0x164),
            (10 * 17, 225, 0x1A4),
            (10 * 17 - 1, 224, 0x164),
            (10 * 17 - 5, 222, 0x0E4),
        ] {
            object.wait_timer = Some(remaining);
            let frame = sprites::select_enemy_frame(&trace, boss_visuals, &object);
            assert_eq!(
                frame.tile_sheet.tiles[0].pixels[0], signature,
                "ASM loc_39EBE/loc_39F7A should advance boss-head death frames from the remaining wait timer"
            );
            assert_eq!(
                frame.tile_base_index, tile_base,
                "ASM loc_39EBE/loc_39F7A should use the C8620/C8618/C8610/C8608/C8600 boss-head descriptors"
            );
        }
    }

    #[test]
    fn boss_head_death_snapshot_flips_loc_39ebe_x_direction() {
        let visuals = sample_level_visuals();
        let mut object = NativeObjectSnapshot {
            kind: NativeObjectKind::Enemy { object_type: 0x22 },
            x_pos: 0x0040_0000,
            y_pos: 0x0010_0000,
            layout_index: Some(0),
            x_velocity: Some(0),
            y_velocity: Some(0),
            facing: Some(Direction::Right),
            on_ground: Some(true),
            hit_points: Some(0),
            damage_flash: false,
            palette_variant: 0,
            stationary: true,
            contact_phase: None,
            shared_walker_state: None,
            shared_walker_patrol_timer: None,
            shared_walker_patrol_short_next: None,
            walk_phase: None,
            wait_timer: Some(120 + 10 * 17),
            debug_fields: std::collections::BTreeMap::new(),
        };
        let trace = sample_trace(GameMode::InGame);
        let mut commands = Vec::new();

        let mut first_flip_x = |remaining| {
            object.wait_timer = Some(remaining);
            commands.clear();
            sprites::push_enemy_sprite(&mut commands, &object, &trace, &visuals, 0x22);
            sprite_tiles(commands.first().expect("boss head sprite"))[0]
                .tile
                .flip_x
        };

        let baseline = first_flip_x(120 + 10 * 17);
        assert!(
            baseline,
            "enemy art is stored left-facing, so a right-facing boss head renders with H-flip"
        );

        for (remaining, expected_flip_x) in [
            (120 + 10 * 17 - 30, !baseline),
            (120 + 10 * 17 - 69, !baseline),
            (120 + 10 * 17 - 70, baseline),
            (120 + 10 * 17 - 96, !baseline),
            (120 + 10 * 17 - 112, baseline),
            (10 * 17, baseline),
            (10 * 17 - 5, !baseline),
            (10 * 17 - 13, baseline),
        ] {
            assert_eq!(
                first_flip_x(remaining),
                expected_flip_x,
                "ASM loc_39EE2/loc_39F7A should render boss-head H-flip from each `eori.b #$FF,x_direction(a3)` phase"
            );
        }
    }

    #[test]
    fn goat_death_snapshot_selects_stru_3c9ea_frames() {
        let visuals = sample_level_visuals();
        let goat_visuals = visuals.enemies.families.get(&0x10).expect("goat visuals");
        let mut object = NativeObjectSnapshot {
            kind: NativeObjectKind::Enemy { object_type: 0x10 },
            x_pos: 0x0040_0000,
            y_pos: 0x0010_0000,
            layout_index: Some(0),
            x_velocity: Some(0),
            y_velocity: Some(0),
            facing: Some(Direction::Right),
            on_ground: Some(true),
            hit_points: Some(0),
            damage_flash: false,
            palette_variant: 0,
            stationary: true,
            contact_phase: None,
            shared_walker_state: None,
            shared_walker_patrol_timer: None,
            shared_walker_patrol_short_next: None,
            walk_phase: None,
            wait_timer: Some(0x1E + 0x0A + 0x37),
            debug_fields: std::collections::BTreeMap::new(),
        };
        let trace = sample_trace(GameMode::InGame);

        assert_eq!(
            sprites::select_enemy_frame(&trace, goat_visuals, &object)
                .tile_sheet
                .tiles[0]
                .pixels[0],
            117,
            "ASM Goat.asm loc_3CB64 should render LnkTo_unk_C7C56 for the first $1E ticks of stru_3C9EA"
        );

        object.wait_timer = Some(0x37 + 1);
        assert_eq!(
            sprites::select_enemy_frame(&trace, goat_visuals, &object)
                .tile_sheet
                .tiles[0]
                .pixels[0],
            118,
            "ASM Goat.asm loc_3CB64 should render LnkTo_unk_C7C8E for the middle $0A ticks of stru_3C9EA"
        );

        object.wait_timer = Some(0x37);
        assert_eq!(
            sprites::select_enemy_frame(&trace, goat_visuals, &object)
                .tile_sheet
                .tiles[0]
                .pixels[0],
            119,
            "ASM Goat.asm loc_3CB64 should render LnkTo_unk_C7C96 for the final $37 ticks of stru_3C9EA"
        );
    }

    #[test]
    fn crab_and_ninja_death_snapshots_select_waited_body_frames() {
        let visuals = sample_level_visuals();
        let trace = sample_trace(GameMode::InGame);
        let mut object = NativeObjectSnapshot {
            kind: NativeObjectKind::Enemy { object_type: 0x09 },
            x_pos: 0x0040_0000,
            y_pos: 0x0010_0000,
            layout_index: Some(0),
            x_velocity: Some(0),
            y_velocity: Some(0),
            facing: Some(Direction::Right),
            on_ground: Some(true),
            hit_points: Some(0),
            damage_flash: false,
            palette_variant: 0,
            stationary: true,
            contact_phase: None,
            shared_walker_state: None,
            shared_walker_patrol_timer: None,
            shared_walker_patrol_short_next: None,
            walk_phase: None,
            wait_timer: Some(0),
            debug_fields: std::collections::BTreeMap::new(),
        };

        let crab_visuals = visuals.enemies.families.get(&0x09).expect("crab visuals");
        for (remaining, signature) in [(0x14 + 0x0A + 0x1E, 217), (0x1E + 1, 218), (0x1E, 219)] {
            object.wait_timer = Some(remaining);
            assert_eq!(
                sprites::select_enemy_frame(&trace, crab_visuals, &object)
                    .tile_sheet
                    .tiles[0]
                    .pixels[0],
                signature,
                "ASM Crab.asm loc_3B094 should step through stru_3AF82 death frames"
            );
        }

        object.kind = NativeObjectKind::Enemy { object_type: 0x11 };
        let ninja_visuals = visuals.enemies.families.get(&0x11).expect("ninja visuals");
        for (remaining, signature) in [(0x14 + 0x0A + 0x2D, 227), (0x2D + 1, 228), (0x2D, 229)] {
            object.wait_timer = Some(remaining);
            assert_eq!(
                sprites::select_enemy_frame(&trace, ninja_visuals, &object)
                    .tile_sheet
                    .tiles[0]
                    .pixels[0],
                signature,
                "ASM Ninja.asm loc_3DB4E should step through stru_3D50A death frames"
            );
        }
    }

    #[test]
    fn tank_and_robot_death_snapshots_select_waited_body_frames() {
        let visuals = sample_level_visuals();
        let trace = sample_trace(GameMode::InGame);
        let mut object = NativeObjectSnapshot {
            kind: NativeObjectKind::Enemy { object_type: 0x0A },
            x_pos: 0x0040_0000,
            y_pos: 0x0010_0000,
            layout_index: Some(0),
            x_velocity: Some(0),
            y_velocity: Some(0),
            facing: Some(Direction::Right),
            on_ground: Some(true),
            hit_points: Some(0),
            damage_flash: false,
            palette_variant: 0,
            stationary: true,
            contact_phase: None,
            shared_walker_state: None,
            shared_walker_patrol_timer: None,
            shared_walker_patrol_short_next: None,
            walk_phase: None,
            wait_timer: Some(0),
            debug_fields: std::collections::BTreeMap::new(),
        };

        let tank_visuals = visuals.enemies.families.get(&0x0A).expect("tank visuals");
        for (remaining, signature) in [(0x10 + 0x08 + 0x3C, 84), (0x08 + 0x3C, 85), (0x3C, 86)] {
            object.wait_timer = Some(remaining);
            assert_eq!(
                sprites::select_enemy_frame(&trace, tank_visuals, &object)
                    .tile_sheet
                    .tiles[0]
                    .pixels[0],
                signature,
                "ASM Tank.asm loc_3B480 should step through stru_3B29A death frames"
            );
        }

        object.kind = NativeObjectKind::Enemy { object_type: 0x0B };
        let shooting_tank_visuals = visuals
            .enemies
            .families
            .get(&0x0B)
            .expect("shooting tank visuals");
        for (remaining, signature) in [(0x10 + 0x08 + 0x3C, 84), (0x08 + 0x3C, 85), (0x3C, 87)] {
            object.wait_timer = Some(remaining);
            assert_eq!(
                sprites::select_enemy_frame(&trace, shooting_tank_visuals, &object)
                    .tile_sheet
                    .tiles[0]
                    .pixels[0],
                signature,
                "ASM Tank_Shooting.asm loc_3B6E6 should step through stru_3B518 death frames"
            );
        }

        object.kind = NativeObjectKind::Enemy { object_type: 0x03 };
        let robot_visuals = visuals.enemies.families.get(&0x03).expect("robot visuals");
        for (remaining, signature) in [(0x09 + 0x09, 97), (0x09, 98)] {
            object.wait_timer = Some(remaining);
            assert_eq!(
                sprites::select_enemy_frame(&trace, robot_visuals, &object)
                    .tile_sheet
                    .tiles[0]
                    .pixels[0],
                signature,
                "ASM Robot.asm loc_3A9D2 should step through stru_3A74A death frames"
            );
        }
    }

    #[test]
    fn build_scene_keeps_rock_tank_death_body_visible_loc_3b480() {
        let tank_death_object = |object_type, wait_timer| NativeObjectSnapshot {
            kind: NativeObjectKind::Enemy { object_type },
            x_pos: 0x0040_0000,
            y_pos: 0x0030_0000,
            layout_index: Some(0),
            x_velocity: Some(0),
            y_velocity: Some(0),
            facing: Some(Direction::Right),
            on_ground: Some(true),
            hit_points: Some(0),
            damage_flash: false,
            palette_variant: 0,
            stationary: true,
            contact_phase: None,
            shared_walker_state: None,
            shared_walker_patrol_timer: None,
            shared_walker_patrol_short_next: None,
            walk_phase: None,
            wait_timer: Some(wait_timer),
            debug_fields: std::collections::BTreeMap::new(),
        };

        let mut trace = sample_trace(GameMode::InGame);
        trace.active_objects = vec![tank_death_object(0x0A, 0x10 + 0x08 + 0x3C)];
        let scene = build_ingame_scene(&trace, false, None);
        let tank_death_command = scene
            .commands
            .iter()
            .find(|command| {
                matches!(
                    command,
                    DrawCommand::Sprite { tile_sheet, .. }
                        if tile_sheet.tiles.first().is_some_and(|tile| tile.pixels[0] == 84)
                )
            })
            .expect("ASM Tank.asm loc_3B480 keeps stru_3B29A visible before deletion");

        match tank_death_command {
            DrawCommand::Sprite { tile_sheet, .. } => assert_eq!(
                tile_sheet.tiles[0].pixels[0], 84,
                "ASM Tank.asm loc_3B480 should render the first stru_3B29A body frame instead of disappearing on the fatal hit frame"
            ),
            other => panic!("expected Tank death sprite command, got {other:?}"),
        }

        trace.active_objects = vec![tank_death_object(0x0B, 0x3C)];
        let scene = build_ingame_scene(&trace, false, None);
        let tank_shooting_death_command = scene
            .commands
            .iter()
            .find(|command| {
                matches!(
                    command,
                    DrawCommand::Sprite { tile_sheet, .. }
                        if tile_sheet.tiles.first().is_some_and(|tile| tile.pixels[0] == 87)
                )
            })
            .expect("ASM Tank_Shooting.asm loc_3B6E6 keeps stru_3B518 visible before deletion");

        match tank_shooting_death_command {
            DrawCommand::Sprite { tile_sheet, .. } => assert_eq!(
                tile_sheet.tiles[0].pixels[0], 87,
                "ASM Tank_Shooting.asm loc_3B6E6 should render the final stru_3B518 body frame instead of disappearing on the fatal hit frame"
            ),
            other => panic!("expected Tank_Shooting death sprite command, got {other:?}"),
        }
    }

    #[test]
    fn fire_demon_diamond_and_armadillo_death_snapshots_select_rom_frames() {
        let visuals = sample_level_visuals();
        let trace = sample_trace(GameMode::InGame);
        let mut object = NativeObjectSnapshot {
            kind: NativeObjectKind::Enemy { object_type: 0x00 },
            x_pos: 0x0040_0000,
            y_pos: 0x0010_0000,
            layout_index: Some(0),
            x_velocity: Some(0),
            y_velocity: Some(0),
            facing: Some(Direction::Right),
            on_ground: None,
            hit_points: Some(0),
            damage_flash: false,
            palette_variant: 0,
            stationary: false,
            contact_phase: None,
            shared_walker_state: None,
            shared_walker_patrol_timer: None,
            shared_walker_patrol_short_next: None,
            walk_phase: None,
            wait_timer: Some(0),
            debug_fields: std::collections::BTreeMap::new(),
        };

        object.kind = NativeObjectKind::Enemy { object_type: 0x00 };
        let fire_demon_visuals = visuals
            .enemies
            .families
            .get(&0x00)
            .expect("fire demon visuals");
        for (remaining, signature) in [(21, 5), (18, 6), (15, 7), (12, 8), (8, 9), (4, 10)] {
            object.wait_timer = Some(remaining);
            assert_eq!(
                sprites::select_enemy_frame(&trace, fire_demon_visuals, &object)
                    .tile_sheet
                    .tiles[0]
                    .pixels[0],
                signature,
                "ASM Fire_Demon.asm loc_3A5A6 should advance `stru_3A378` death frames"
            );
        }

        object.kind = NativeObjectKind::Enemy { object_type: 0x01 };
        let diamond_visuals = visuals
            .enemies
            .families
            .get(&0x01)
            .expect("diamond visuals");
        for (remaining, signature) in [(22, 26), (14, 27), (8, 28), (5, 29), (2, 30)] {
            object.wait_timer = Some(remaining);
            assert_eq!(
                sprites::select_enemy_frame(&trace, diamond_visuals, &object)
                    .tile_sheet
                    .tiles[0]
                    .pixels[0],
                signature,
                "ASM Diamond.asm loc_3AE28/loc_3AEE6 should advance `stru_3ACC4` death frames"
            );
        }

        object.kind = NativeObjectKind::Enemy { object_type: 0x04 };
        let armadillo_visuals = visuals
            .enemies
            .families
            .get(&0x04)
            .expect("armadillo visuals");
        for (remaining, signature) in [
            (0x30, 47),
            (0x1A, 48),
            (0x16, 47),
            (0x12, 48),
            (0x0F, 47),
            (0x0C, 48),
            (0x0A, 47),
            (0x08, 48),
            (0x07, 47),
            (0x06, 48),
            (0x05, 47),
            (0x04, 48),
            (0x03, 49),
        ] {
            object.wait_timer = Some(remaining);
            assert_eq!(
                sprites::select_enemy_frame(&trace, armadillo_visuals, &object)
                    .tile_sheet
                    .tiles[0]
                    .pixels[0],
                signature,
                "ASM Armadillo.asm loc_3C886 should advance `stru_3C69A` death frames"
            );
        }
    }

    #[test]
    fn flying_dragon_death_snapshot_selects_loc_3492c_frames() {
        let visuals = sample_level_visuals();
        let dragon_visuals = visuals.enemies.families.get(&0x0C).expect("dragon visuals");
        let mut object = NativeObjectSnapshot {
            kind: NativeObjectKind::Enemy { object_type: 0x0C },
            x_pos: 0x0040_0000,
            y_pos: 0x0010_0000,
            layout_index: Some(0),
            x_velocity: Some(0),
            y_velocity: Some(0),
            facing: Some(Direction::Right),
            on_ground: None,
            hit_points: Some(0),
            damage_flash: false,
            palette_variant: 0,
            stationary: false,
            contact_phase: None,
            shared_walker_state: None,
            shared_walker_patrol_timer: None,
            shared_walker_patrol_short_next: None,
            walk_phase: None,
            wait_timer: Some(8),
            debug_fields: std::collections::BTreeMap::new(),
        };
        let trace = sample_trace(GameMode::InGame);

        for (remaining, signature) in [(8, 141), (6, 142), (4, 143), (2, 144)] {
            object.wait_timer = Some(remaining);
            assert_eq!(
                sprites::select_enemy_frame(&trace, dragon_visuals, &object)
                    .tile_sheet
                    .tiles[0]
                    .pixels[0],
                signature,
                "ASM Dragon_Flying.asm loc_3492C should advance `stru_345E4` every two ticks"
            );
        }
    }

    #[test]
    fn flying_dragon_live_snapshot_uses_stru_34600_sequence() {
        let visuals = sample_level_visuals();
        let dragon_visuals = visuals.enemies.families.get(&0x0C).expect("dragon visuals");
        let object = NativeObjectSnapshot {
            kind: NativeObjectKind::Enemy { object_type: 0x0C },
            x_pos: 0x0040_0000,
            y_pos: 0x0010_0000,
            layout_index: Some(0),
            x_velocity: Some(0),
            y_velocity: Some(0),
            facing: Some(Direction::Right),
            on_ground: None,
            hit_points: Some(1),
            damage_flash: false,
            palette_variant: 0,
            stationary: true,
            contact_phase: None,
            shared_walker_state: None,
            shared_walker_patrol_timer: None,
            shared_walker_patrol_short_next: None,
            walk_phase: None,
            wait_timer: None,
            debug_fields: std::collections::BTreeMap::new(),
        };
        let mut trace = sample_trace(GameMode::InGame);

        for (frame_index, signature) in [
            (0, 141),
            (21, 144),
            (28, 141),
            (84, 145),
            (91, 142),
            (112, 145),
            (119, 142),
        ] {
            trace.frame_index = frame_index;
            let frame = sprites::select_enemy_frame(&trace, dragon_visuals, &object);
            assert_eq!(
                frame.tile_sheet.tiles[0].pixels[0], signature,
                "ASM Dragon_Flying.asm stru_34600 should not use a simple five-frame loop"
            );
        }
    }

    #[test]
    fn flying_dragon_dive_snapshot_uses_off_345f6_body_frame() {
        let visuals = sample_level_visuals();
        let dragon_visuals = visuals.enemies.families.get(&0x0C).expect("dragon visuals");
        let mut object = NativeObjectSnapshot {
            kind: NativeObjectKind::Enemy { object_type: 0x0C },
            x_pos: 0x0041_0000,
            y_pos: 0x006D_0000,
            layout_index: Some(0),
            x_velocity: Some(-0x0002_0000),
            y_velocity: Some(-0x0003_0000),
            facing: Some(Direction::Left),
            on_ground: None,
            hit_points: Some(1),
            damage_flash: false,
            palette_variant: 0,
            stationary: false,
            contact_phase: None,
            shared_walker_state: None,
            shared_walker_patrol_timer: None,
            shared_walker_patrol_short_next: None,
            walk_phase: None,
            wait_timer: None,
            debug_fields: std::collections::BTreeMap::new(),
        };
        let mut trace = sample_trace(GameMode::InGame);
        trace.frame_index = 60;

        assert_eq!(
            sprites::select_enemy_frame(&trace, dragon_visuals, &object)
                .tile_sheet
                .tiles[0]
                .pixels[0],
            148,
            "ASM Dragon_Flying.asm loc_348CE should map `unk_34554` velocity -2,-3 through `off_345F6+$0006` to `LnkTo_unk_C762E`, not anonymous `unk_C7626`"
        );

        object.y_velocity = Some(0);
        assert_eq!(
            sprites::select_enemy_frame(&trace, dragon_visuals, &object)
                .tile_sheet
                .tiles[0]
                .pixels[0],
            143,
            "ASM Dragon_Flying.asm loc_348CE should map the horizontal dive band through `off_345F6+$0000` to `LnkTo_unk_C75FE`"
        );
    }

    #[test]
    fn sphere_death_snapshot_selects_enemy06_kill_frames() {
        let visuals = sample_level_visuals();
        let sphere_visuals = visuals.enemies.families.get(&0x06).expect("sphere visuals");
        let mut object = NativeObjectSnapshot {
            kind: NativeObjectKind::Enemy { object_type: 0x06 },
            x_pos: 0x0040_0000,
            y_pos: 0x0010_0000,
            layout_index: Some(0),
            x_velocity: Some(0),
            y_velocity: Some(0),
            facing: Some(Direction::Right),
            on_ground: None,
            hit_points: Some(0),
            damage_flash: false,
            palette_variant: 0,
            stationary: false,
            contact_phase: None,
            shared_walker_state: None,
            shared_walker_patrol_timer: None,
            shared_walker_patrol_short_next: None,
            walk_phase: None,
            wait_timer: Some(0x0A + 0x0A + 0x0A),
            debug_fields: std::collections::BTreeMap::new(),
        };
        let trace = sample_trace(GameMode::InGame);

        for (remaining, signature, tile_base) in
            [(0x1E, 154, 0x006), (0x14, 155, 0x00A), (0x0A, 156, 0x00E)]
        {
            object.wait_timer = Some(remaining);
            let frame = sprites::select_enemy_frame(&trace, sphere_visuals, &object);
            assert_eq!(
                frame.tile_sheet.tiles[0].pixels[0], signature,
                "ASM Sphere.asm Enemy06_Sphere_Kill should step through `stru_36182` death frames"
            );
            assert_eq!(
                frame.tile_base_index, tile_base,
                "ASM `stru_36182` should use the disappearing Sphere descriptors, not normal `stru_36174`"
            );
        }
    }

    #[test]
    fn ufo_death_snapshot_selects_loc_35174_frames() {
        let visuals = sample_level_visuals();
        let ufo_visuals = visuals.enemies.families.get(&0x0F).expect("ufo visuals");
        let mut object = NativeObjectSnapshot {
            kind: NativeObjectKind::Enemy { object_type: 0x0F },
            x_pos: 0x0040_0000,
            y_pos: 0x0010_0000,
            layout_index: Some(0),
            x_velocity: Some(0),
            y_velocity: Some(0),
            facing: Some(Direction::Right),
            on_ground: None,
            hit_points: Some(0),
            damage_flash: false,
            palette_variant: 0,
            stationary: false,
            contact_phase: None,
            shared_walker_state: None,
            shared_walker_patrol_timer: None,
            shared_walker_patrol_short_next: None,
            walk_phase: None,
            wait_timer: Some(0x15),
            debug_fields: std::collections::BTreeMap::new(),
        };
        let trace = sample_trace(GameMode::InGame);

        for (remaining, signature, tile_base) in
            [(0x15, 214, 0x15A), (0x0E, 215, 0x16A), (0x07, 216, 0x17A)]
        {
            object.wait_timer = Some(remaining);
            let frame = sprites::select_enemy_frame(&trace, ufo_visuals, &object);
            assert_eq!(
                frame.tile_sheet.tiles[0].pixels[0], signature,
                "ASM UFO.asm loc_35174/loc_35180 should advance parent `stru_34CA0` death frames"
            );
            assert_eq!(
                frame.tile_base_index, tile_base,
                "ASM UFO.asm `stru_34CA0` should reuse the three parent UFO animation frames before loc_351B6 deletes it"
            );
        }
    }

    #[test]
    fn fireball_death_snapshot_selects_enemy19_kill_frames() {
        let visuals = sample_level_visuals();
        let fireball_visuals = visuals
            .enemies
            .families
            .get(&0x19)
            .expect("fireball visuals");
        let mut object = NativeObjectSnapshot {
            kind: NativeObjectKind::Enemy { object_type: 0x19 },
            x_pos: 0x0040_0000,
            y_pos: 0x0010_0000,
            layout_index: Some(0),
            x_velocity: Some(0),
            y_velocity: Some(0),
            facing: Some(Direction::Right),
            on_ground: None,
            hit_points: Some(0),
            damage_flash: false,
            palette_variant: 0,
            stationary: false,
            contact_phase: None,
            shared_walker_state: None,
            shared_walker_patrol_timer: None,
            shared_walker_patrol_short_next: None,
            walk_phase: None,
            wait_timer: Some(8),
            debug_fields: std::collections::BTreeMap::new(),
        };
        let trace = sample_trace(GameMode::InGame);

        for (remaining, signature) in [(8, 171), (6, 172), (4, 173), (2, 174)] {
            object.wait_timer = Some(remaining);
            assert_eq!(
                sprites::select_enemy_frame(&trace, fireball_visuals, &object)
                    .tile_sheet
                    .tiles[0]
                    .pixels[0],
                signature,
                "ASM Fireball.asm Enemy19_Fireball_Kill should advance `stru_345E4` every two ticks"
            );
        }
    }

    #[test]
    fn archer_death_snapshot_selects_loc_3414a_frames() {
        let visuals = sample_level_visuals();
        let archer_visuals = visuals.enemies.families.get(&0x07).expect("archer visuals");
        let mut object = NativeObjectSnapshot {
            kind: NativeObjectKind::Enemy { object_type: 0x07 },
            x_pos: 0x0040_0000,
            y_pos: 0x0010_0000,
            layout_index: Some(0),
            x_velocity: Some(0),
            y_velocity: Some(0),
            facing: Some(Direction::Right),
            on_ground: None,
            hit_points: Some(0),
            damage_flash: false,
            palette_variant: 0,
            stationary: false,
            contact_phase: None,
            shared_walker_state: None,
            shared_walker_patrol_timer: None,
            shared_walker_patrol_short_next: None,
            walk_phase: None,
            wait_timer: Some(0x05 + 0x05 + 0x05 + 0x23),
            debug_fields: std::collections::BTreeMap::new(),
        };
        let trace = sample_trace(GameMode::InGame);

        for (remaining, signature) in [(0x32, 187), (0x2D, 188), (0x28, 189), (0x23, 190)] {
            object.wait_timer = Some(remaining);
            assert_eq!(
                sprites::select_enemy_frame(&trace, archer_visuals, &object)
                    .tile_sheet
                    .tiles[0]
                    .pixels[0],
                signature,
                "ASM Archer.asm loc_3414A should advance `stru_33FC2` death frames"
            );
        }
    }

    #[test]
    fn tornado_death_snapshot_selects_loc_359b2_frames() {
        let visuals = sample_level_visuals();
        let tornado_visuals = visuals
            .enemies
            .families
            .get(&0x18)
            .expect("tornado visuals");
        let mut object = NativeObjectSnapshot {
            kind: NativeObjectKind::Enemy { object_type: 0x18 },
            x_pos: 0x0040_0000,
            y_pos: 0x0010_0000,
            layout_index: Some(0),
            x_velocity: Some(0),
            y_velocity: Some(0),
            facing: Some(Direction::Right),
            on_ground: None,
            hit_points: Some(0),
            damage_flash: false,
            palette_variant: 0,
            stationary: false,
            contact_phase: None,
            shared_walker_state: None,
            shared_walker_patrol_timer: None,
            shared_walker_patrol_short_next: None,
            walk_phase: None,
            wait_timer: Some(0x06 + 0x03 + 0x04 + 0x04 + 0x03 + 0x06),
            debug_fields: std::collections::BTreeMap::new(),
        };
        let trace = sample_trace(GameMode::InGame);

        for (remaining, signature) in [
            (0x1A, 234),
            (0x14, 235),
            (0x11, 234),
            (0x0D, 235),
            (0x09, 234),
            (0x06, 235),
        ] {
            object.wait_timer = Some(remaining);
            assert_eq!(
                sprites::select_enemy_frame(&trace, tornado_visuals, &object)
                    .tile_sheet
                    .tiles[0]
                    .pixels[0],
                signature,
                "ASM Tornado.asm loc_359B2 should alternate `stru_35B1E` death frames"
            );
        }
    }

    #[test]
    fn walking_dragon_and_lion_death_snapshots_select_rom_frames() {
        let visuals = sample_level_visuals();
        let trace = sample_trace(GameMode::InGame);
        let mut object = NativeObjectSnapshot {
            kind: NativeObjectKind::Enemy { object_type: 0x0D },
            x_pos: 0x0040_0000,
            y_pos: 0x0010_0000,
            layout_index: Some(0),
            x_velocity: Some(0),
            y_velocity: Some(0),
            facing: Some(Direction::Right),
            on_ground: None,
            hit_points: Some(0),
            damage_flash: false,
            palette_variant: 0,
            stationary: false,
            contact_phase: None,
            shared_walker_state: None,
            shared_walker_patrol_timer: None,
            shared_walker_patrol_short_next: None,
            walk_phase: None,
            wait_timer: Some(0),
            debug_fields: std::collections::BTreeMap::new(),
        };

        let walking_dragon_visuals = visuals
            .enemies
            .families
            .get(&0x0D)
            .expect("walking dragon visuals");
        for (remaining, signature) in [
            (167, 167),
            (160, 168),
            (154, 167),
            (151, 169),
            (129, 170),
            (124, 169),
            (120, 170),
            (116, 169),
            (104, 170),
            (99, 169),
        ] {
            object.wait_timer = Some(remaining);
            assert_eq!(
                sprites::select_enemy_frame(&trace, walking_dragon_visuals, &object)
                    .tile_sheet
                    .tiles[0]
                    .pixels[0],
                signature,
                "ASM Dragon.asm loc_3CEFC should advance `stru_3CCEC` death frames"
            );
        }

        object.kind = NativeObjectKind::Enemy { object_type: 0x12 };
        let lion_visuals = visuals.enemies.families.get(&0x12).expect("lion visuals");
        for (remaining, signature) in [(30, 56), (20, 57), (10, 58)] {
            object.wait_timer = Some(remaining);
            assert_eq!(
                sprites::select_enemy_frame(&trace, lion_visuals, &object)
                    .tile_sheet
                    .tiles[0]
                    .pixels[0],
                signature,
                "ASM Lion.asm loc_3E378 should advance `stru_3DF48` death frames"
            );
        }
    }

    #[test]
    fn live_lion_uses_rom_walk_script_not_death_frames() {
        let visuals = sample_level_visuals();
        let lion_visuals = visuals.enemies.families.get(&0x12).expect("lion visuals");
        let mut trace = sample_trace(GameMode::InGame);
        let mut object = NativeObjectSnapshot {
            kind: NativeObjectKind::Enemy { object_type: 0x12 },
            x_pos: 0x01EF_0000,
            y_pos: 0x02C0_0000,
            layout_index: Some(1),
            x_velocity: Some(-0x0001_8000),
            y_velocity: Some(0),
            facing: Some(Direction::Left),
            on_ground: Some(true),
            hit_points: Some(6),
            damage_flash: false,
            palette_variant: 0,
            stationary: false,
            contact_phase: None,
            shared_walker_state: Some(10),
            shared_walker_patrol_timer: Some(0x003C),
            shared_walker_patrol_short_next: None,
            walk_phase: None,
            wait_timer: Some(0),
            debug_fields: std::collections::BTreeMap::from([(
                String::from("lion_state"),
                String::from("ResumingCharge"),
            )]),
        };

        for (frame_index, signature, tile_base) in [(60, 53, 0x1D8), (71, 55, 0x21A)] {
            trace.frame_index = frame_index;
            let frame = sprites::select_enemy_frame(&trace, lion_visuals, &object);
            assert_eq!(
                frame.tile_sheet.tiles[0].pixels[0], signature,
                "ASM Lion.asm loc_3DFF0/loc_3E0F4 should keep live HP-6 Lions in `stru_3DEDA` rather than cycling into appended death frames"
            );
            assert_eq!(
                frame.tile_base_index, tile_base,
                "ASM Lion.asm `stru_3DEDA` frame descriptor should match C7E88..C7EB0 live sprite mappings"
            );
        }

        for (walk_phase, tile_base) in [(13, 0x1B4), (24, 0x1D8)] {
            object.walk_phase = Some(walk_phase);
            trace.frame_index = 0;
            let frame = sprites::select_enemy_frame(&trace, lion_visuals, &object);
            assert_eq!(
                frame.tile_base_index, tile_base,
                "ASM Lion.asm loc_3E0F4 resumes `stru_3DEDA`; issue #289 frame checkpoints should render the phase-selected C7E98/C7EA0 live bodies"
            );
        }
    }

    #[test]
    fn live_walking_dragon_uses_rom_walk_sequence_not_death_frames() {
        let visuals = sample_level_visuals();
        let walking_dragon_visuals = visuals
            .enemies
            .families
            .get(&0x0D)
            .expect("walking dragon visuals");
        let mut trace = sample_trace(GameMode::InGame);
        let mut object = NativeObjectSnapshot {
            kind: NativeObjectKind::Enemy { object_type: 0x0D },
            x_pos: 0x0040_0000,
            y_pos: 0x0010_0000,
            layout_index: Some(0),
            x_velocity: Some(0),
            y_velocity: Some(0),
            facing: Some(Direction::Right),
            on_ground: Some(true),
            hit_points: Some(1),
            damage_flash: false,
            palette_variant: 0,
            stationary: false,
            contact_phase: None,
            shared_walker_state: Some(10),
            shared_walker_patrol_timer: Some(0x12C),
            shared_walker_patrol_short_next: Some(true),
            walk_phase: Some(0),
            wait_timer: Some(0x12C),
            debug_fields: std::collections::BTreeMap::new(),
        };

        for (frame_index, signature) in [
            (0, 161),
            (9, 162),
            (18, 163),
            (27, 164),
            (36, 165),
            (45, 166),
            (54, 161),
            (63, 162),
            (72, 163),
            (81, 164),
            (90, 168),
            (99, 166),
        ] {
            trace.frame_index = frame_index;
            assert_eq!(
                sprites::select_enemy_frame(&trace, walking_dragon_visuals, &object)
                    .tile_sheet
                    .tiles[0]
                    .pixels[0],
                signature,
                "ASM Dragon.asm loc_3CE48 `stru_3CC5E` should cycle live walking-dragon frames without using `loc_3CEFC` death/smoke frames"
            );
        }

        object.layout_index = Some(2);
        trace.frame_index = 120;
        assert_eq!(
            sprites::select_enemy_frame(&trace, walking_dragon_visuals, &object)
                .tile_sheet
                .tiles[0]
                .pixels[0],
            162,
            "ASM Dragon.asm loc_3CE48 starts `stru_3CC5E` per object with `j_Init_Animation`; native must not add a layout-index phase offset that selects the wrong Blue Lake Woods dragon frame"
        );

        object.hit_points = Some(3);
        object.layout_index = Some(0);
        for (frame_index, signature) in [(0, 161), (6, 162), (60, 168), (66, 166)] {
            trace.frame_index = frame_index;
            assert_eq!(
                sprites::select_enemy_frame(&trace, walking_dragon_visuals, &object)
                    .tile_sheet
                    .tiles[0]
                    .pixels[0],
                signature,
                "ASM Dragon.asm loc_3CDFC HP-3 `stru_3CC90` should use the same live frame sequence with six-tick timing"
            );
        }

        object.hit_points = Some(0);
        object.wait_timer = Some(167);
        trace.frame_index = 54;
        assert_eq!(
            sprites::select_enemy_frame(&trace, walking_dragon_visuals, &object)
                .tile_sheet
                .tiles[0]
                .pixels[0],
            167,
            "actual zero-HP snapshots should still use ASM Dragon.asm loc_3CEFC death frames"
        );
    }

    #[test]
    fn nonlethal_walking_dragon_stomp_uses_hurt_frame_loc_3cebe_stru_3cce6() {
        let visuals = sample_level_visuals();
        let walking_dragon_visuals = visuals
            .enemies
            .families
            .get(&0x0D)
            .expect("walking dragon visuals");
        let mut trace = sample_trace(GameMode::InGame);
        trace.frame_index = 18;
        let object = NativeObjectSnapshot {
            kind: NativeObjectKind::Enemy { object_type: 0x0D },
            x_pos: 0x0040_0000,
            y_pos: 0x0010_0000,
            layout_index: Some(0),
            x_velocity: Some(0),
            y_velocity: Some(0),
            facing: Some(Direction::Right),
            on_ground: Some(true),
            hit_points: Some(1),
            damage_flash: true,
            palette_variant: 0,
            stationary: true,
            contact_phase: None,
            shared_walker_state: Some(10),
            shared_walker_patrol_timer: Some(0x12C),
            shared_walker_patrol_short_next: Some(true),
            walk_phase: Some(0),
            wait_timer: Some(0x12C),
            debug_fields: std::collections::BTreeMap::new(),
        };

        let frame = sprites::select_enemy_frame(&trace, walking_dragon_visuals, &object);

        assert_eq!(
            frame.tile_sheet.tiles[0].pixels[0], 167,
            "ASM Dragon.asm loc_3CEBE installs `stru_3CCE6`: a nonlethal stomp should hold `unk_C755E`, not continue the walk cycle or run the full loc_3CEFC death sequence"
        );
    }

    #[test]
    fn live_walking_dragon_breath_phase_uses_parent_mouth_frame_loc_3cf90() {
        let visuals = sample_level_visuals();
        let walking_dragon_visuals = visuals
            .enemies
            .families
            .get(&0x0D)
            .expect("walking dragon visuals");
        let trace = sample_trace(GameMode::InGame);
        let object = NativeObjectSnapshot {
            kind: NativeObjectKind::Enemy { object_type: 0x0D },
            x_pos: 0x0040_0000,
            y_pos: 0x0010_0000,
            layout_index: Some(0),
            x_velocity: Some(0),
            y_velocity: Some(0),
            facing: Some(Direction::Right),
            on_ground: Some(true),
            hit_points: Some(1),
            damage_flash: false,
            palette_variant: 0,
            stationary: false,
            contact_phase: None,
            shared_walker_state: Some(10),
            shared_walker_patrol_timer: Some(0x12C),
            shared_walker_patrol_short_next: Some(true),
            walk_phase: Some(1),
            wait_timer: Some(0x01F4),
            debug_fields: std::collections::BTreeMap::new(),
        };

        let frame = sprites::select_enemy_frame(&trace, walking_dragon_visuals, &object);

        assert_eq!(
            frame.tile_base_index, 0x04C,
            "ASM Dragon.asm loc_3CF90 starts parent animation `stru_3CCC2`, whose only frame is `unk_C758E: sprite_frame_vram $04C,$10,$00,$20,$27`",
        );
    }

    #[test]
    fn build_scene_keeps_walking_dragon_visible_during_loc_3cefc_death_animation() {
        let visuals = sample_level_visuals();
        let mut trace = sample_trace(GameMode::InGame);
        trace.active_objects = vec![NativeObjectSnapshot {
            kind: NativeObjectKind::Enemy { object_type: 0x0D },
            x_pos: 0x0040_0000,
            y_pos: 0x0020_0000,
            layout_index: Some(0),
            x_velocity: Some(0),
            y_velocity: Some(0),
            facing: Some(Direction::Right),
            on_ground: Some(true),
            hit_points: Some(0),
            damage_flash: false,
            palette_variant: 0,
            stationary: true,
            contact_phase: None,
            shared_walker_state: Some(10),
            shared_walker_patrol_timer: Some(0x12C),
            shared_walker_patrol_short_next: Some(true),
            walk_phase: Some(0),
            wait_timer: Some(167),
            debug_fields: std::collections::BTreeMap::new(),
        }];

        let scene = build_scene(&trace, false, Some(&visuals), None);

        let death_sprite = scene.commands.iter().find_map(|command| match command {
            DrawCommand::Sprite { x, y, tiles, .. }
                if *x == 32 && *y == 24 && tiles[0].tile.tile_index == 0x000 =>
            {
                Some(tiles)
            }
            _ => None,
        });
        assert!(
            death_sprite.is_some(),
            "ASM Dragon.asm loc_3CEFC should keep the zero-HP walking dragon visible on `stru_3CCEC` instead of dropping the enemy draw command"
        );
    }

    #[test]
    fn tar_monster_and_orca_death_snapshots_select_rom_frames() {
        let visuals = sample_level_visuals();
        let trace = sample_trace(GameMode::InGame);
        let mut object = NativeObjectSnapshot {
            kind: NativeObjectKind::Enemy { object_type: 0x05 },
            x_pos: 0x0040_0000,
            y_pos: 0x0010_0000,
            layout_index: Some(0),
            x_velocity: Some(0),
            y_velocity: Some(0),
            facing: Some(Direction::Right),
            on_ground: None,
            hit_points: Some(0),
            damage_flash: false,
            palette_variant: 0,
            stationary: false,
            contact_phase: None,
            shared_walker_state: None,
            shared_walker_patrol_timer: None,
            shared_walker_patrol_short_next: None,
            walk_phase: None,
            wait_timer: Some(0),
            debug_fields: std::collections::BTreeMap::new(),
        };

        let tar_visuals = visuals
            .enemies
            .families
            .get(&0x05)
            .expect("tar monster visuals");
        for (remaining, signature) in [
            (63, 61),
            (54, 62),
            (45, 63),
            (42, 62),
            (39, 63),
            (30, 64),
            (27, 63),
            (24, 64),
            (15, 65),
            (12, 64),
            (9, 65),
        ] {
            object.wait_timer = Some(remaining);
            assert_eq!(
                sprites::select_enemy_frame(&trace, tar_visuals, &object)
                    .tile_sheet
                    .tiles[0]
                    .pixels[0],
                signature,
                "ASM Tar_Monster.asm loc_323E6 should advance `stru_32700` death frames"
            );
        }

        object.kind = NativeObjectKind::Enemy { object_type: 0x08 };
        let orca_visuals = visuals.enemies.families.get(&0x08).expect("orca visuals");
        for (remaining, signature) in [
            (156, 72),
            (147, 73),
            (143, 72),
            (140, 74),
            (123, 75),
            (119, 74),
            (115, 75),
            (112, 74),
            (109, 75),
            (107, 74),
            (105, 75),
            (103, 74),
            (101, 75),
            (99, 76),
        ] {
            object.wait_timer = Some(remaining);
            assert_eq!(
                sprites::select_enemy_frame(&trace, orca_visuals, &object)
                    .tile_sheet
                    .tiles[0]
                    .pixels[0],
                signature,
                "ASM Orca.asm loc_3D268/loc_3D4BC should advance `stru_3D11E` death frames"
            );
        }
    }

    #[test]
    fn more_waited_death_snapshots_select_rom_frames() {
        let visuals = sample_level_visuals();
        let trace = sample_trace(GameMode::InGame);
        let mut object = NativeObjectSnapshot {
            kind: NativeObjectKind::Enemy { object_type: 0x13 },
            x_pos: 0x0040_0000,
            y_pos: 0x0010_0000,
            layout_index: Some(0),
            x_velocity: Some(0),
            y_velocity: Some(0),
            facing: Some(Direction::Right),
            on_ground: None,
            hit_points: Some(0),
            damage_flash: false,
            palette_variant: 0,
            stationary: false,
            contact_phase: None,
            shared_walker_state: None,
            shared_walker_patrol_timer: None,
            shared_walker_patrol_short_next: None,
            walk_phase: None,
            wait_timer: Some(0),
            debug_fields: std::collections::BTreeMap::new(),
        };

        for (object_type, cases, message) in [
            (
                0x13,
                &[(0x0A + 0x14 + 0x2D, 137), (0x14 + 0x2D, 138), (0x2D, 139)][..],
                "ASM Scorpion.asm loc_3DDEE should advance `stru_3DB80` death frames",
            ),
            (
                0x16,
                &[(0x0C, 124), (0x08, 125), (0x04, 126)][..],
                "ASM Drips.asm loc_3392C should advance `stru_33F06` death frames",
            ),
            (
                0x1A,
                &[(0x0A + 0x14 + 0x28, 153), (0x14 + 0x28, 154), (0x28, 155)][..],
                "ASM Driller.asm loc_3BF1C should advance `stru_3BCD4` death frames",
            ),
        ] {
            object.kind = NativeObjectKind::Enemy { object_type };
            let enemy_visuals = visuals
                .enemies
                .families
                .get(&object_type)
                .expect("enemy visuals");

            for (remaining, signature) in cases {
                object.wait_timer = Some(*remaining);
                assert_eq!(
                    sprites::select_enemy_frame(&trace, enemy_visuals, &object)
                        .tile_sheet
                        .tiles[0]
                        .pixels[0],
                    *signature,
                    "{message}"
                );
            }
        }

        object.kind = NativeObjectKind::Enemy { object_type: 0x17 };
        let hand_visuals = visuals.enemies.families.get(&0x17).expect("hand visuals");
        for (remaining, signature) in [
            (0x42, 142),
            (0x3C, 143),
            (0x36, 142),
            (0x30, 143),
            (0x2A, 142),
            (0x24, 143),
            (0x1E, 144),
            (0x18, 145),
            (0x12, 146),
            (0x0C, 147),
            (0x06, 148),
        ] {
            object.wait_timer = Some(remaining);
            assert_eq!(
                sprites::select_enemy_frame(&trace, hand_visuals, &object)
                    .tile_sheet
                    .tiles[0]
                    .pixels[0],
                signature,
                "ASM Hand.asm loc_32AA2 should advance `stru_32B78` death frames"
            );
        }
    }

    #[test]
    fn emo_rock_death_snapshot_selects_loc_36346_frames() {
        let visuals = sample_level_visuals();
        let emo_rock_visuals = visuals
            .enemies
            .families
            .get(&0x1B)
            .expect("emo rock visuals");
        let mut object = NativeObjectSnapshot {
            kind: NativeObjectKind::Enemy { object_type: 0x1B },
            x_pos: 0x0040_0000,
            y_pos: 0x0010_0000,
            layout_index: Some(0),
            x_velocity: Some(0),
            y_velocity: Some(0),
            facing: Some(Direction::Right),
            on_ground: None,
            hit_points: Some(0),
            damage_flash: false,
            palette_variant: 0,
            stationary: false,
            contact_phase: None,
            shared_walker_state: None,
            shared_walker_patrol_timer: None,
            shared_walker_patrol_short_next: None,
            walk_phase: None,
            wait_timer: Some(6 * 5),
            debug_fields: std::collections::BTreeMap::new(),
        };
        let trace = sample_trace(GameMode::InGame);

        for (remaining, signature) in [(30, 244), (24, 245), (18, 246), (12, 247), (6, 248)] {
            object.wait_timer = Some(remaining);
            assert_eq!(
                sprites::select_enemy_frame(&trace, emo_rock_visuals, &object)
                    .tile_sheet
                    .tiles[0]
                    .pixels[0],
                signature,
                "ASM Emo_Rock.asm loc_36346 should step through `stru_36468` death frames"
            );
        }
    }

    #[test]
    fn cloud_death_snapshot_selects_loc_35c0e_frames() {
        let visuals = sample_level_visuals();
        let cloud_visuals = visuals.enemies.families.get(&0x0E).expect("cloud visuals");
        let mut object = NativeObjectSnapshot {
            kind: NativeObjectKind::Enemy { object_type: 0x0E },
            x_pos: 0x0040_0000,
            y_pos: 0x0010_0000,
            layout_index: Some(0),
            x_velocity: Some(0),
            y_velocity: Some(0),
            facing: Some(Direction::Right),
            on_ground: None,
            hit_points: Some(0),
            damage_flash: false,
            palette_variant: 0,
            stationary: false,
            contact_phase: None,
            shared_walker_state: None,
            shared_walker_patrol_timer: None,
            shared_walker_patrol_short_next: None,
            walk_phase: None,
            wait_timer: Some(7 + 7 + 7),
            debug_fields: std::collections::BTreeMap::new(),
        };
        let trace = sample_trace(GameMode::InGame);

        for (remaining, signature) in [(21, 255), (14, 200), (7, 201)] {
            object.wait_timer = Some(remaining);
            assert_eq!(
                sprites::select_enemy_frame(&trace, cloud_visuals, &object)
                    .tile_sheet
                    .tiles[0]
                    .pixels[0],
                signature,
                "ASM Cloud.asm loc_35C0E should step through `stru_35EBA` death frames"
            );
        }
    }

    #[test]
    fn spinning_twins_death_snapshot_selects_loc_3bc56_frames() {
        let visuals = sample_level_visuals();
        let twins_visuals = visuals
            .enemies
            .families
            .get(&0x14)
            .expect("spinning twins visuals");
        let mut object = NativeObjectSnapshot {
            kind: NativeObjectKind::Enemy { object_type: 0x14 },
            x_pos: 0x0040_0000,
            y_pos: 0x0010_0000,
            layout_index: Some(0),
            x_velocity: Some(0),
            y_velocity: Some(0),
            facing: Some(Direction::Right),
            on_ground: None,
            hit_points: Some(0),
            damage_flash: false,
            palette_variant: 0,
            stationary: false,
            contact_phase: None,
            shared_walker_state: None,
            shared_walker_patrol_timer: None,
            shared_walker_patrol_short_next: None,
            walk_phase: None,
            wait_timer: Some(0x19 + 0x0F + 0x14),
            debug_fields: std::collections::BTreeMap::new(),
        };
        let trace = sample_trace(GameMode::InGame);

        for (remaining, signature) in [(0x3C, 206), (0x23, 207), (0x14, 208)] {
            object.wait_timer = Some(remaining);
            assert_eq!(
                sprites::select_enemy_frame(&trace, twins_visuals, &object)
                    .tile_sheet
                    .tiles[0]
                    .pixels[0],
                signature,
                "ASM Spinning_Twins.asm loc_3BC56 should step through `stru_3B83C` death frames"
            );
        }
    }

    #[test]
    fn big_hopping_skull_death_snapshot_selects_loc_3664e_frames() {
        let visuals = sample_level_visuals();
        let skull_visuals = visuals
            .enemies
            .families
            .get(&0x1D)
            .expect("big hopping skull visuals");
        let mut object = NativeObjectSnapshot {
            kind: NativeObjectKind::Enemy { object_type: 0x1D },
            x_pos: 0x0040_0000,
            y_pos: 0x0010_0000,
            layout_index: Some(0),
            x_velocity: Some(0),
            y_velocity: Some(0),
            facing: Some(Direction::Right),
            on_ground: None,
            hit_points: Some(0),
            damage_flash: false,
            palette_variant: 0,
            stationary: false,
            contact_phase: None,
            shared_walker_state: None,
            shared_walker_patrol_timer: None,
            shared_walker_patrol_short_next: None,
            walk_phase: None,
            wait_timer: Some(0x18 + 0x0C + 0x0A + 0x64),
            debug_fields: std::collections::BTreeMap::new(),
        };
        let trace = sample_trace(GameMode::InGame);

        for (remaining, signature) in [(0x8A, 210), (0x72, 211), (0x66, 212), (0x64, 213)] {
            object.wait_timer = Some(remaining);
            assert_eq!(
                sprites::select_enemy_frame(&trace, skull_visuals, &object)
                    .tile_sheet
                    .tiles[0]
                    .pixels[0],
                signature,
                "ASM Big_Hopping_Skull.asm loc_3664E should step through `stru_367FA` death frames"
            );
        }
    }

    #[test]
    fn build_game_scene_emits_platform_and_collectible_art_commands() {
        let mut trace = sample_trace(GameMode::InGame);
        trace.camera_x_pos = 0;
        trace.camera_y_pos = 0;
        let platform_debug_fields = std::collections::BTreeMap::from([
            ("platform_width_px".to_string(), "31".to_string()),
            ("platform_height_px".to_string(), "15".to_string()),
            ("platform_render_flags".to_string(), "0".to_string()),
            ("platform_mapping_addr_word".to_string(), "148".to_string()),
        ]);
        trace.active_objects = vec![
            platform_snapshot(0x00, 0x00, 0x0010, 0x0018, platform_debug_fields),
            NativeObjectSnapshot {
                kind: NativeObjectKind::Enemy { object_type: 0x01 },
                x_pos: 0x0028_0000,
                y_pos: 0x0010_0000,
                layout_index: None,
                x_velocity: None,
                y_velocity: None,
                facing: None,
                on_ground: None,
                hit_points: None,
                damage_flash: false,
                palette_variant: 0,
                stationary: false,
                contact_phase: None,
                shared_walker_state: None,
                shared_walker_patrol_timer: None,
                shared_walker_patrol_short_next: None,
                walk_phase: None,
                wait_timer: None,
                debug_fields: std::collections::BTreeMap::new(),
            },
        ];

        let scene = build_ingame_scene(&trace, false, None);

        assert!(
            has_asset_backed_command(&scene, 41),
            "expected platform ROM art command"
        );
        assert_eq!(
            scene
                .commands
                .iter()
                .filter(|command| has_command_signature(command, 41))
                .count(),
            2,
            "ASM Make_SpritesFromPlatforms loc_244C/loc_2548 emits one 16px platform cell per width slot",
        );
        assert!(
            has_sprite_signature(&scene, 21),
            "expected collectible diamond ROM art"
        );
        assert!(!scene.commands.iter().any(|command| {
            matches!(command, DrawCommand::Rect { colour, .. } if *colour == PLATFORM_COLOUR)
        }));
    }

    #[test]
    fn build_game_scene_uses_platform_off_24ac_strip_variant() {
        let mut trace = sample_trace(GameMode::InGame);
        trace.camera_x_pos = 0;
        trace.camera_y_pos = 0;
        trace.active_objects = vec![platform_snapshot(
            0x00,
            0x02,
            0x0010,
            0x0018,
            std::collections::BTreeMap::from([
                ("platform_width_px".to_string(), "79".to_string()),
                ("platform_height_px".to_string(), "47".to_string()),
                ("platform_render_flags".to_string(), "4".to_string()),
                ("platform_mapping_addr_word".to_string(), "1710".to_string()),
            ]),
        )];

        let scene = build_ingame_scene(&trace, false, None);
        let platform_commands = scene
            .commands
            .iter()
            .filter(|command| has_command_signature(command, 51))
            .collect::<Vec<_>>();

        assert_eq!(
            platform_commands.len(),
            3,
            "ASM off_24AC byte 4 routes to loc_251C, which emits a vertical strip from height only instead of width-tiling the platform",
        );
        let first_tile_indices = platform_commands
            .iter()
            .map(|command| {
                sprite_tiles(command)
                    .first()
                    .expect("platform tile")
                    .tile
                    .tile_index
            })
            .collect::<Vec<_>>();
        assert_eq!(
            first_tile_indices,
            vec![0, 4, 8],
            "ASM loc_251C uses `$18(a2)`, then `$18(a2)+4`, then `$18(a2)+8` for top/middle/bottom platform cells",
        );
    }

    #[test]
    fn build_game_scene_uses_platform_loc_24f0_cap_middle_end_tiles() {
        let mut trace = sample_trace(GameMode::InGame);
        trace.camera_x_pos = 0;
        trace.camera_y_pos = 0;
        trace.active_objects = vec![platform_snapshot(
            0x00,
            0x01,
            0x0010,
            0x0018,
            std::collections::BTreeMap::from([
                ("platform_width_px".to_string(), "79".to_string()),
                ("platform_height_px".to_string(), "15".to_string()),
                ("platform_render_flags".to_string(), "2".to_string()),
                ("platform_mapping_addr_word".to_string(), "148".to_string()),
            ]),
        )];

        let scene = build_ingame_scene(&trace, false, None);
        let platform_commands = scene
            .commands
            .iter()
            .filter(|command| has_command_signature(command, 41))
            .collect::<Vec<_>>();

        assert_eq!(
            platform_commands.len(),
            5,
            "ASM off_24AC byte 2 routes to loc_24F0, which emits one horizontal strip sprite per platform width cell",
        );
        let first_tile_indices = platform_commands
            .iter()
            .map(|command| {
                sprite_tiles(command)
                    .first()
                    .expect("platform tile")
                    .tile
                    .tile_index
            })
            .collect::<Vec<_>>();
        assert_eq!(
            first_tile_indices,
            vec![0, 4, 4, 4, 8],
            "ASM loc_24F0 uses `$18(a2)`, then `$18(a2)+4`, then `$18(a2)+8` for first/middle/last platform cells",
        );
        assert!(
            platform_commands
                .iter()
                .all(|command| sprite_tiles(command).len() == 4),
            "ASM loc_2548 writes one 16x16 hardware sprite cell for each loc_24F0 entry",
        );
    }

    #[test]
    fn build_game_scene_selects_platform_art_from_mapping_word_loc_2548() {
        let mut trace = sample_trace(GameMode::InGame);
        trace.camera_x_pos = 0;
        trace.camera_y_pos = 0;
        trace.active_objects = vec![platform_snapshot(
            0x04,
            0x00,
            0x0010,
            0x0018,
            std::collections::BTreeMap::from([
                ("platform_width_px".to_string(), "31".to_string()),
                ("platform_height_px".to_string(), "15".to_string()),
                ("platform_render_flags".to_string(), "0".to_string()),
                ("platform_mapping_addr_word".to_string(), "1710".to_string()),
            ]),
        )];

        let scene = build_ingame_scene(&trace, false, None);

        assert!(
            has_sprite_signature(&scene, 51),
            "ASM loc_244C/loc_2548 should render from `$18(a2)`/PlatformArtAddr_Table; Initialize_Platforms loads Vertical_platform.bin at VRAM tile $06AE",
        );
        assert!(
            !has_sprite_signature(&scene, 41),
            "platform kind is not the ROM art selector; `$18(a2)` is",
        );
    }

    #[test]
    fn build_game_scene_uses_platform_025a_block_art_from_loc_2548() {
        let mut trace = sample_trace(GameMode::InGame);
        trace.camera_x_pos = 0;
        trace.camera_y_pos = 0;
        trace.active_objects = vec![platform_snapshot(
            0x03,
            0x03,
            0x0010,
            0x0018,
            std::collections::BTreeMap::from([
                ("platform_width_px".to_string(), "15".to_string()),
                ("platform_height_px".to_string(), "15".to_string()),
                ("platform_render_flags".to_string(), "6".to_string()),
                ("platform_mapping_addr_word".to_string(), "602".to_string()),
            ]),
        )];

        let scene = build_ingame_scene(&trace, false, None);

        assert!(
            has_sprite_signature(&scene, 237),
            "ASM PlatformArtAddr_Table loc_282E can store `$025A`; loc_2548 writes `$18(a2)` directly as the sprite VRAM tile, so this entry must use common block art, not Vertical_platform.bin",
        );
        assert!(
            !has_sprite_signature(&scene, 51),
            "VRAM tile `$025A` is in the block/common-art range, while Vertical_platform.bin is loaded at `$06AE`",
        );
    }

    #[test]
    fn build_game_scene_culls_platforms_outside_loc_244c_viewport() {
        let mut trace = sample_trace(GameMode::InGame);
        trace.camera_x_pos = 0x0200_0000;
        trace.camera_y_pos = 0;
        trace.active_objects = vec![platform_snapshot(
            0x00,
            0x00,
            0x0010,
            0x0018,
            std::collections::BTreeMap::from([
                ("platform_width_px".to_string(), "31".to_string()),
                ("platform_height_px".to_string(), "15".to_string()),
                ("platform_render_flags".to_string(), "0".to_string()),
                ("platform_mapping_addr_word".to_string(), "1710".to_string()),
            ]),
        )];

        let scene = build_ingame_scene(&trace, false, None);

        assert_eq!(
            scene
                .commands
                .iter()
                .filter(|command| has_command_signature(command, 41))
                .count(),
            0,
            "ASM loc_244C skips platforms whose trailing edge is still left of Camera_X_pos",
        );
    }

    #[test]
    fn build_game_scene_emits_player_sprite_tiles_for_idle_walk_jump_and_fall() {
        let mut idle = trace_with_player(0, 0);
        idle.kid_x_velocity = Some(0);
        idle.kid_y_velocity = Some(0);
        idle.kid_on_ground = Some(true);

        let mut walk = trace_with_player(0, 0);
        walk.kid_x_velocity = Some(0x0001_0000);
        walk.kid_y_velocity = Some(0);
        walk.kid_on_ground = Some(true);

        let mut jump_rise = trace_with_player(0, 0);
        jump_rise.kid_x_velocity = Some(0x0001_0000);
        jump_rise.kid_y_velocity = Some(-0x0002_0000);
        jump_rise.kid_on_ground = Some(false);

        let mut jump_fall = trace_with_player(0, 0);
        jump_fall.kid_x_velocity = Some(0x0001_0000);
        jump_fall.kid_y_velocity = Some(0x0002_0000);
        jump_fall.kid_on_ground = Some(false);

        let idle_scene = build_ingame_scene(&idle, false, None);
        let walk_scene = build_ingame_scene(&walk, false, None);
        let jump_rise_scene = build_ingame_scene(&jump_rise, false, None);
        let jump_fall_scene = build_ingame_scene(&jump_fall, false, None);

        assert_eq!(player_sprite_signature(&idle_scene), Some(1));
        assert_eq!(player_sprite_signature(&walk_scene), Some(2));
        assert_eq!(player_sprite_signature(&jump_rise_scene), Some(8));
        assert_eq!(player_sprite_signature(&jump_fall_scene), Some(9));
        assert!(!has_rect(&idle_scene, 8, 8, 16, 32, PLAYER_COLOUR));
    }

    #[test]
    fn wall_grab_pullup_cooldown_renders_stru_8bb4_rollover_frames() {
        let run_case = |remaining: u8| {
            let mut trace = trace_with_player(0, 0);
            trace.kid_x_velocity = Some(0);
            trace.kid_y_velocity = Some(0);
            trace.kid_on_ground = Some(true);
            trace.ledge_snap_cooldown = remaining;
            player_sprite_signature(&build_ingame_scene(&trace, false, None))
        };

        assert_eq!(
            run_case(21),
            Some(40),
            "ASM loc_ABF0 starts `stru_8BB4` on kid_flip_1 for the first three VBlanks",
        );
        assert_eq!(
            run_case(18),
            Some(41),
            "ASM `stru_8BB4` should advance to kid_flip_2 after three hibernated frames",
        );
        assert_eq!(
            run_case(1),
            Some(46),
            "ASM `stru_8BB4` should still show kid_flip_7 on the final cooldown frame",
        );
    }

    #[test]
    fn build_sprite_tiles_matches_original_dma_chunk_order() {
        let frame = numbered_object_frame(48, 16);

        let tiles = build_sprite_tiles(&frame, Direction::Right, 0);
        let actual = tiles
            .iter()
            .map(|tile| (tile.tile.tile_index, tile.offset_x, tile.offset_y))
            .collect::<Vec<_>>();

        assert_eq!(
            actual,
            vec![
                (0, 0, 0),
                (1, 0, 8),
                (2, 8, 0),
                (3, 8, 8),
                (4, 16, 0),
                (5, 16, 8),
                (6, 24, 0),
                (7, 24, 8),
                (8, 32, 0),
                (9, 32, 8),
                (10, 40, 0),
                (11, 40, 8),
            ]
        );
    }

    #[test]
    fn build_sprite_tiles_mirrors_chunk_layout_for_left_facing_frames() {
        let frame = numbered_object_frame(48, 16);

        let tiles = build_sprite_tiles(&frame, Direction::Left, 0);
        let actual = tiles
            .iter()
            .map(|tile| {
                (
                    tile.tile.tile_index,
                    tile.offset_x,
                    tile.offset_y,
                    tile.tile.flip_x,
                )
            })
            .collect::<Vec<_>>();

        assert_eq!(
            actual,
            vec![
                (0, 40, 0, true),
                (1, 40, 8, true),
                (2, 32, 0, true),
                (3, 32, 8, true),
                (4, 24, 0, true),
                (5, 24, 8, true),
                (6, 16, 0, true),
                (7, 16, 8, true),
                (8, 8, 0, true),
                (9, 8, 8, true),
                (10, 0, 0, true),
                (11, 0, 8, true),
            ]
        );
    }

    #[test]
    fn loc_bc34_damage_flash_keeps_player_visible_and_marks_white_phase() {
        let white_phase = trace_with_player(0, 66);
        let white_scene = build_ingame_scene(&white_phase, false, None);
        assert!(
            scene_has_player_sprite(&white_scene),
            "ASM loc_BC34 keeps Kid visible through Just_received_damage instead of blinking out"
        );
        let white = MdColour::from_word(0x0FFF);
        let player_palette = player_sprite_palette(&white_scene).expect("player sprite palette");
        assert!(
            player_palette
                .colours
                .iter()
                .take(12)
                .all(|colour| *colour == white),
            "ASM sub_80BC writes $FFF into the first twelve visible player palette slots"
        );
        assert!(
            !white_scene.commands.iter().any(|command| matches!(
                command,
                DrawCommand::Rect {
                    width: 16,
                    height: 24,
                    colour: [240, 240, 240],
                    ..
                }
            )),
            "damage flash should use the player sprite palette, not an overlay rectangle"
        );

        let normal_phase = trace_with_player(4, 62);
        let normal_scene = build_ingame_scene(&normal_phase, false, None);
        assert!(scene_has_player_sprite(&normal_scene));
        let normal_palette = player_sprite_palette(&normal_scene).expect("player sprite palette");
        assert!(
            normal_palette.colours.iter().any(|colour| *colour != white),
            "ASM sub_80D0 restores the normal form palette for the next four frames"
        );

        let no_damage = trace_with_player(4, 0);
        assert!(scene_has_player_sprite(&build_ingame_scene(
            &no_damage, false, None
        )));
    }

    #[test]
    fn loc_bc56_damage_flash_mid_cycle_matches_issue289_oracle_samples() {
        let white = MdColour::from_word(0x0FFF);
        for remaining in [33, 22] {
            let scene = build_ingame_scene(&trace_with_player(60, remaining), false, None);
            let player_palette = player_sprite_palette(&scene).expect("player sprite palette");
            assert!(
                player_palette
                    .colours
                    .iter()
                    .take(12)
                    .all(|colour| *colour == white),
                "ASM loc_BC56/sub_80BC should render the #289 damage-flash samples white when native trace has {remaining} invulnerability frames remaining"
            );
        }
    }

    #[test]
    fn loc_b7fa_death_tumble_renderer_uses_engine_position_without_extra_offset() {
        let mut trace = trace_with_player(30, 0);
        trace.death_windup_elapsed = 30;
        if let Some(player) = trace.active_objects.first_mut() {
            player.x_pos = 0x0018_0000;
            player.y_pos = 0x0040_0000;
        }

        let scene = build_ingame_scene(&trace, false, None);

        assert_eq!(
            sprite_position(&scene, 47),
            Some((8, 24)),
            "ASM Death loc_B7FA/loc_B80C uses unk_A3FF8/kid_death.bin at the engine-driven tumble position"
        );

        trace.current_helmet_id = PlayerForm::IronKnight.helmet_id();
        let helmeted_scene = build_ingame_scene(&trace, false, None);
        assert_eq!(
            sprite_position(&helmeted_scene, 47),
            Some((8, 24)),
            "ASM Death loc_B7EA runs Kid_Transform before loc_B7FA, so helmeted deaths must still render Kid's death frame during the visible tumble"
        );
    }

    // --- US-012: HUD display ---

    #[test]
    fn game_hud_shows_hp_pips_and_lives() {
        let mut trace = sample_trace(GameMode::InGame);
        trace.number_hitpoints = 2;
        trace.number_lives = 3;
        trace.diamond_count = 7;
        let scene = build_ingame_scene(&trace, false, None);

        let hud_digits = scene
            .commands
            .iter()
            .filter(|command| {
                matches!(
                    command,
                    DrawCommand::IndexedTile { y, tile_sheet, tile, .. }
                        if *y < 18
                            && tile_sheet.tiles.len() == 12
                            && tile.tile_index <= 11
                )
            })
            .count();
        let hitpoint_tiles = scene
            .commands
            .iter()
            .filter(|command| {
                matches!(
                    command,
                    DrawCommand::IndexedTile { y, tile_sheet, tile, .. }
                        if *y < 48
                            && tile_sheet.tiles.len() == 4
                            && tile.tile_index <= 3
                )
            })
            .count();

        assert!(hud_digits >= 2, "HUD should show digit art");
        assert_eq!(
            hitpoint_tiles, 2,
            "ASM Make_SpriteAttr_HUD renders The Kid's 2 HP as one size-4 two-tile pip"
        );
        assert!(!contains_text(&scene, "x3"));
        assert!(!contains_text(&scene, "D:7"));
    }

    #[test]
    fn game_hud_single_digit_counters_put_digit_before_x_slot_like_calc_display_number() {
        let mut trace = sample_trace(GameMode::InGame);
        trace.number_lives = 3;
        trace.diamond_count = 7;
        let scene = build_ingame_scene(&trace, false, None);

        let (_, lives_first, _) =
            indexed_tile_at_with_sheet_len(&scene, 0x190 - 128, 0x90 - 128, 12)
                .expect("lives first tile");
        let (_, lives_second, _) =
            indexed_tile_at_with_sheet_len(&scene, 0x198 - 128, 0x90 - 128, 12)
                .expect("lives second tile");
        let (_, diamond_first, _) =
            indexed_tile_at_with_sheet_len(&scene, 0x190 - 128, 0xA4 - 128, 12)
                .expect("diamond first tile");
        let (_, diamond_second, _) =
            indexed_tile_at_with_sheet_len(&scene, 0x198 - 128, 0xA4 - 128, 12)
                .expect("diamond second tile");

        assert_eq!(
            (lives_first.tile_index, lives_second.tile_index),
            (3, 11),
            "ASM life_display writes calc_display_number's low word to the first lives slot, then swaps to put $86C5 in the following x slot"
        );
        assert_eq!(
            (diamond_first.tile_index, diamond_second.tile_index),
            (7, 11),
            "ASM diamond_display writes calc_display_number's low word to the first diamond slot, then swaps to put $86C5 in the following x slot"
        );
    }

    #[test]
    fn game_hud_double_digit_counters_clamp_and_split_like_calc_display_number() {
        let mut trace = sample_trace(GameMode::InGame);
        trace.number_lives = 100;
        trace.diamond_count = 42;
        let scene = build_ingame_scene(&trace, false, None);

        let (_, lives_tens, _) =
            indexed_tile_at_with_sheet_len(&scene, 0x190 - 128, 0x90 - 128, 12)
                .expect("lives tens tile");
        let (_, lives_ones, _) =
            indexed_tile_at_with_sheet_len(&scene, 0x198 - 128, 0x90 - 128, 12)
                .expect("lives ones tile");
        let (_, diamond_tens, _) =
            indexed_tile_at_with_sheet_len(&scene, 0x190 - 128, 0xA4 - 128, 12)
                .expect("diamond tens tile");
        let (_, diamond_ones, _) =
            indexed_tile_at_with_sheet_len(&scene, 0x198 - 128, 0xA4 - 128, 12)
                .expect("diamond ones tile");

        assert_eq!(
            (lives_tens.tile_index, lives_ones.tile_index),
            (9, 9),
            "ASM loc_BF2A clamps 100 lives to 99 before calc_display_number"
        );
        assert_eq!(
            (diamond_tens.tile_index, diamond_ones.tile_index),
            (4, 2),
            "ASM calc_display_number splits two-digit counters through divu.w #$A"
        );
    }

    #[test]
    fn game_hud_timer_uses_digit_tiles_and_colon_slot_positions() {
        let mut trace = sample_trace(GameMode::InGame);
        trace.level_timer_seconds = 179;
        let scene = build_ingame_scene(&trace, false, None);

        let (_, minutes, _) = indexed_tile_at_with_sheet_len(&scene, 0x90 - 128, 0x90 - 128, 12)
            .expect("minutes tile");
        let (_, colon, _) =
            indexed_tile_at_with_sheet_len(&scene, 0x98 - 128, 0x90 - 128, 12).expect("colon tile");
        let (_, seconds_tens, _) =
            indexed_tile_at_with_sheet_len(&scene, 0x9D - 128, 0x90 - 128, 12)
                .expect("seconds tens tile");
        let (_, seconds_ones, _) =
            indexed_tile_at_with_sheet_len(&scene, 0xA5 - 128, 0x90 - 128, 12)
                .expect("seconds ones tile");

        assert_eq!(
            (
                minutes.tile_index,
                colon.tile_index,
                seconds_tens.tile_index,
                seconds_ones.tile_index
            ),
            (2, 10, 5, 9),
            "ASM loc_BEE8 writes timer digits from $86BA+d and sub_BD0A rewrites the fixed timer row to unk_BD8E positions $90,$98,$9D,$A5"
        );
    }

    #[test]
    fn game_hud_timer_rollover_renders_sub_bdc0_decrement_before_trace_second_advances() {
        let mut trace = sample_trace(GameMode::InGame);
        trace.level_timer_seconds = 177;
        trace.level_timer_sub_frames = 1;
        let scene = build_ingame_scene(&trace, false, None);

        let (_, minutes, _) = indexed_tile_at_with_sheet_len(&scene, 0x90 - 128, 0x90 - 128, 12)
            .expect("minutes tile");
        let (_, seconds_tens, _) =
            indexed_tile_at_with_sheet_len(&scene, 0x9D - 128, 0x90 - 128, 12)
                .expect("seconds tens tile");
        let (_, seconds_ones, _) =
            indexed_tile_at_with_sheet_len(&scene, 0xA5 - 128, 0x90 - 128, 12)
                .expect("seconds ones tile");

        assert_eq!(
            (
                minutes.tile_index,
                seconds_tens.tile_index,
                seconds_ones.tile_index
            ),
            (2, 5, 6),
            "ASM Make_SpriteAttr_HUD loc_BE82 decrements Time_SubSeconds before loc_BEE8 writes timer digit tiles, so sub-frame 1 renders the next visible second"
        );
    }

    #[test]
    fn game_hud_low_time_tracks_kid_with_loc_bd40_layout() {
        let mut trace = sample_trace(GameMode::InGame);
        trace.level_timer_seconds = 29;
        trace.kid_x_pos = Some(0x0130_0000);
        trace.kid_y_pos = Some(0x0180_0000);
        trace.camera_x_pos = 0x0100_0000;
        trace.camera_y_pos = 0x0120_0000;
        let scene = build_ingame_scene(&trace, false, None);

        assert!(
            indexed_tile_at_with_sheet_len(&scene, 0x9D - 128, 0x90 - 128, 12).is_none(),
            "ASM End_Decrease_Time_Left should route <=0:29 through sub_BD0A d7=1 instead of keeping seconds tens on the fixed HUD row"
        );
        assert!(
            indexed_tile_at_with_sheet_len(&scene, 0xA5 - 128, 0x90 - 128, 12).is_none(),
            "ASM loc_BD40 should move seconds ones away from the fixed HUD row"
        );

        let (_, seconds_tens, _) =
            indexed_tile_at_with_sheet_len(&scene, 0xA9 - 128, 0xB4 - 128, 12)
                .expect("Kid-tracking seconds tens tile");
        let (_, seconds_ones, _) =
            indexed_tile_at_with_sheet_len(&scene, 0xB1 - 128, 0xB4 - 128, 12)
                .expect("Kid-tracking seconds ones tile");

        assert_eq!(
            (seconds_tens.tile_index, seconds_ones.tile_index),
            (2, 9),
            "ASM loc_BD40 positions the low-time seconds digits at Kid-Camera screen space plus unk_BD8A offsets -7/+1"
        );
    }

    #[test]
    fn level_complete_pre_hologram_hud_timer_uses_loc_c070_latched_previous_second() {
        let mut trace = sample_trace(GameMode::InGame);
        trace.level_timer_seconds = 178;
        trace.level_complete = true;
        trace.level_complete_elapsed_frames = EOL_HOLOGRAM_VISIBLE_FRAME_LOC_C1D0_ORACLE - 10;
        let scene = build_ingame_scene(&trace, false, None);

        let (_, minutes, _) = indexed_tile_at_with_sheet_len(&scene, 0x90 - 128, 0x90 - 128, 12)
            .expect("minutes tile");
        let (_, seconds_tens, _) =
            indexed_tile_at_with_sheet_len(&scene, 0x9D - 128, 0x90 - 128, 12)
                .expect("seconds tens tile");
        let (_, seconds_ones, _) =
            indexed_tile_at_with_sheet_len(&scene, 0xA5 - 128, 0x90 - 128, 12)
                .expect("seconds ones tile");

        assert_eq!(
            (
                minutes.tile_index,
                seconds_tens.tile_index,
                seconds_ones.tile_index
            ),
            (2, 5, 9),
            "ASM CharacterCollision_TouchFlagpole loc_C070 sets $FFFFFB4B before Make_SpriteAttr_HUD's decrement path, leaving the prebuilt HUD timer on the previous second until the end-level hologram takes over"
        );
    }

    #[test]
    fn game_hud_uses_asm_static_symbol_slots() {
        let trace = sample_trace(GameMode::InGame);
        let scene = build_ingame_scene(&trace, false, None);

        assert!(
            scene.commands.iter().any(|command| {
                matches!(
                    command,
                    DrawCommand::Sprite { x, y, tile_sheet, tiles, .. }
                        if *x == 0x1A1 - 128
                            && *y == 0x8D - 128
                            && tile_sheet.tiles.iter().any(|tile| tile.pixels == [101; 64])
                            && tiles.len() == 4
                )
            }),
            "ASM HUD table unk_1427C uses a 2x2 Life_icon sprite for the ankh symbol at $1A1,$8D"
        );
        assert!(
            scene.commands.iter().any(|command| {
                matches!(
                    command,
                    DrawCommand::Sprite { x, y, tile_sheet, tiles, .. }
                        if *x == 0x1A0 - 128
                            && *y == 0xA2 - 128
                            && tile_sheet.tiles.len() >= 16
                            && tiles.len() == 4
                            && tiles.first().is_some_and(|tile| tile.tile.tile_index == 8)
                )
            }),
            "ASM HUD table unk_1427C uses a 2x2 diamond sprite from VRAM tile $86F2 at $1A0,$A2"
        );
    }

    #[test]
    fn game_hud_diamond_icon_animates_via_sub_44dc_dma_pages() {
        let mut trace = sample_trace(GameMode::InGame);
        trace.frame_index = 0;
        let frame_0 = build_ingame_scene(&trace, false, None);
        trace.frame_index = 6;
        let frame_6 = build_ingame_scene(&trace, false, None);

        let diamond_page_sample = |scene: &Scene| {
            scene.commands.iter().find_map(|command| match command {
                DrawCommand::Sprite {
                    x,
                    y,
                    tile_sheet,
                    tiles,
                    ..
                } if *x == 0x1A0 - 128
                    && *y == 0xA2 - 128
                    && tiles.first().is_some_and(|tile| tile.tile.tile_index == 8) =>
                {
                    tile_sheet.tiles.first().map(|tile| tile.pixels[0])
                }
                _ => None,
            })
        };

        assert_eq!(
            diamond_page_sample(&frame_0),
            Some(18),
            "ASM sub_44B0 seeds the diamond countdown to 1, and the rendered HUD page includes the current sub_44DC loc_45EE upload, so replay-local frame 0 displays the third `unk_461C` page"
        );
        assert_eq!(
            diamond_page_sample(&frame_6),
            Some(19),
            "ASM sub_44DC advances `unk_461C` after 6 VBlanks, so native must not keep the HUD diamond static"
        );
    }

    #[test]
    fn game_hud_diamond_icon_prefers_traced_sub_44dc_offset() {
        let mut trace = sample_trace(GameMode::InGame);
        trace.frame_index = 0;
        trace.hud_diamond_frame_offset = Some(6);
        let frame_offset_6 = build_ingame_scene(&trace, false, None);

        trace.frame_index = 60;
        let frame_index_60_same_offset = build_ingame_scene(&trace, false, None);

        let diamond_page_sample = |scene: &Scene| {
            scene.commands.iter().find_map(|command| match command {
                DrawCommand::Sprite {
                    x,
                    y,
                    tile_sheet,
                    tiles,
                    ..
                } if *x == 0x1A0 - 128
                    && *y == 0xA2 - 128
                    && tiles.first().is_some_and(|tile| tile.tile.tile_index == 8) =>
                {
                    tile_sheet.tiles.first().map(|tile| tile.pixels[0])
                }
                _ => None,
            })
        };

        assert_eq!(
            diamond_page_sample(&frame_offset_6),
            Some(16),
            "ASM sub_44DC loc_45EE stores `$FFFFFA5D=6`; trace-scene rendering presents the subsequent VBlank page, wrapping the fourth `unk_461C` DMA page back to the first"
        );
        assert_eq!(
            diamond_page_sample(&frame_index_60_same_offset),
            Some(16),
            "HUD diamond rendering should use traced sub_44DC RAM, not replay-local frame_index phase"
        );
    }

    #[test]
    fn settled_level_skip_hud_diamond_icon_uses_current_sub_44dc_offset_loc_45ee() {
        let mut trace = sample_trace(GameMode::InGame);
        trace.level_skip_cheat_enabled = 1;
        trace.level_completion_time = 1;
        trace.hud_diamond_frame_offset = Some(6);
        let frame_offset_6 = build_ingame_scene(&trace, false, None);

        let diamond_page_sample = |scene: &Scene| {
            scene.commands.iter().find_map(|command| match command {
                DrawCommand::Sprite {
                    x,
                    y,
                    tile_sheet,
                    tiles,
                    ..
                } if *x == 0x1A0 - 128
                    && *y == 0xA2 - 128
                    && tiles.first().is_some_and(|tile| tile.tile.tile_index == 8) =>
                {
                    tile_sheet.tiles.first().map(|tile| tile.pixels[0])
                }
                _ => None,
            })
        };

        assert_eq!(
            diamond_page_sample(&frame_offset_6),
            Some(19),
            "ASM sub_44DC loc_45EE stores `$FFFFFA5D=6`; settled LevelSkip_Cheat recordings render the current oracle-visible `unk_461C` page instead of the normal subsequent VBlank page"
        );
    }

    #[test]
    fn game_hud_life_icon_prefers_traced_sub_44dc_offset() {
        let mut trace = sample_trace(GameMode::InGame);
        trace.hud_life_frame_offset = Some(0);
        let scene = build_ingame_scene(&trace, false, None);

        let ankh_tile = scene.commands.iter().find_map(|command| match command {
            DrawCommand::Sprite { x, y, tiles, .. }
                if *x == 0x1A1 - 128 && *y == 0x8D - 128 && tiles.len() == 4 =>
            {
                tiles.first().map(|tile| tile.tile.tile_index)
            }
            _ => None,
        });

        assert_eq!(
            ankh_tile,
            Some(8),
            "ASM sub_44DC loc_4546 samples `$FFFFFA5E=0`; oracle frame issue-289-level06-idle-current@179 shows the normal gameplay HUD ankh presents the third Life_icon ROM page for that trace state"
        );
    }

    #[test]
    fn settled_level_skip_hud_life_icon_keeps_verified_sub_44dc_phases_loc_4546() {
        let ankh_tile = |scene: &Scene| {
            scene.commands.iter().find_map(|command| match command {
                DrawCommand::Sprite { x, y, tiles, .. }
                    if *x == 0x1A1 - 128 && *y == 0x8D - 128 && tiles.len() == 4 =>
                {
                    tiles.first().map(|tile| tile.tile.tile_index)
                }
                _ => None,
            })
        };

        let mut trace = sample_trace(GameMode::InGame);
        trace.level_skip_cheat_enabled = 1;
        trace.level_completion_time = 1;

        trace.hud_life_frame_offset = Some(4);
        assert_eq!(
            ankh_tile(&build_ingame_scene(&trace, false, None)),
            Some(0),
            "ASM sub_44DC loc_4546 stores `$FFFFFA5E=4`; oracle frame issue-289-level07-idle-current@60 keeps the settled LevelSkip_Cheat HUD ankh on the frozen page"
        );

        trace.hud_life_frame_offset = Some(6);
        assert_eq!(
            ankh_tile(&build_ingame_scene(&trace, false, None)),
            Some(4),
            "ASM sub_44DC loc_4546 stores `$FFFFFA5E=6`; oracle frame issue-289-level07-idle-current@71 renders the current `AniArt_LifeIcon` frame byte `$04`"
        );
    }

    #[test]
    fn level_complete_first_handoff_uses_latched_life_icon_page_loc_4546() {
        let ankh_tile = |scene: &Scene| {
            scene.commands.iter().find_map(|command| match command {
                DrawCommand::Sprite { x, y, tiles, .. }
                    if *x == 0x1A1 - 128 && *y == 0x8D - 128 && tiles.len() == 4 =>
                {
                    tiles.first().map(|tile| tile.tile.tile_index)
                }
                _ => None,
            })
        };

        let mut trace = sample_trace(GameMode::InGame);
        assert_eq!(
            ankh_tile(&build_ingame_scene(&trace, false, None)),
            Some(0),
            "ordinary gameplay HUD uses the first Life_icon page from ASM Init_SpriteAttr_HUD"
        );

        trace.level_complete = true;
        trace.level_complete_elapsed_frames = 0;
        assert_eq!(
            ankh_tile(&build_ingame_scene(&trace, false, None)),
            Some(4),
            "ASM loc_C070's first rendered hand-off frame uses the pre-frame sub_44DC loc_4586 life-icon page"
        );

        trace.level_complete_elapsed_frames = 60;
        assert_eq!(
            ankh_tile(&build_ingame_scene(&trace, false, None)),
            Some(0),
            "later frozen-HUD frames return to the normal Life_icon page instead of retaining the first-frame latch"
        );
    }

    #[test]
    fn game_hud_hitpoint_pips_start_at_asm_sprite_table_x_position() {
        let mut trace = sample_trace(GameMode::InGame);
        trace.number_hitpoints = 1;
        let scene = build_ingame_scene(&trace, false, None);

        let (_, half_pip, _) = indexed_tile_at_with_sheet_len(&scene, 0x90 - 128, 0xA4 - 128, 4)
            .expect("first hitpoint pip");
        let (_, grey_tail, _) = indexed_tile_at_with_sheet_len(&scene, 0x98 - 128, 0xA4 - 128, 4)
            .expect("second tile of size-4 hitpoint sprite");

        assert_eq!(
            half_pip.tile_index, 1,
            "ASM Make_SpriteAttr_HUD loc_BE34 writes 1 HP as the half-pip tile $822E at biased X $90"
        );
        assert_eq!(
            grey_tail.tile_index, 2,
            "ASM Make_SpriteAttr_HUD loc_BE34 size byte #4 also draws the following grey pip tile $822F"
        );
        assert!(
            indexed_tile_at_with_sheet_len(&scene, 0, 0xA4 - 128, 4).is_none(),
            "native must not draw the old left-edge HP pip approximation"
        );
        assert!(
            indexed_tile_at_with_sheet_len(&scene, 0xA0 - 128, 0xA4 - 128, 4).is_none(),
            "ASM exits after The Kid's single 2-HP slot; there is no second empty pip"
        );
    }

    #[test]
    fn game_hud_reflects_damage() {
        let mut trace = sample_trace(GameMode::InGame);
        trace.number_hitpoints = 1; // Took damage
        let scene = build_ingame_scene(&trace, false, None);

        let has_half_pip = scene.commands.iter().any(|command| {
            matches!(
                command,
                DrawCommand::IndexedTile { y, tile_sheet, tile, .. }
                        if *y < 48
                        && tile_sheet.tiles.len() == 4
                        && tile.tile_index == 1
            )
        });
        assert!(
            has_half_pip,
            "ASM loc_BE34 should show half-hitpoint art after The Kid takes one point of damage"
        );
    }

    #[test]
    fn game_hud_uses_player_form_hitpoint_capacity_for_helmet_state() {
        let mut skycutter = sample_trace(GameMode::InGame);
        skycutter.current_helmet_id = PlayerForm::Skycutter.helmet_id();
        skycutter.number_hitpoints = PlayerForm::Skycutter.max_hitpoints();

        let skycutter_scene = build_ingame_scene(&skycutter, false, None);

        // At full HP every HP is one coloured dot: full slots draw two 8×8 tiles
        // and the odd trailing slot (#276) draws one, so the tile count equals
        // the form's max HP (Skycutter 3, Iron Knight 5) — not max/2*2.
        assert_eq!(
            count_hud_hitpoint_tiles(&skycutter_scene),
            PlayerForm::Skycutter.max_hitpoints() as usize
        );

        let mut iron_knight = sample_trace(GameMode::InGame);
        iron_knight.current_helmet_id = PlayerForm::IronKnight.helmet_id();
        iron_knight.number_hitpoints = PlayerForm::IronKnight.max_hitpoints();

        let iron_knight_scene = build_ingame_scene(&iron_knight, false, None);

        assert_eq!(
            count_hud_hitpoint_tiles(&iron_knight_scene),
            PlayerForm::IronKnight.max_hitpoints() as usize
        );
    }

    #[test]
    fn game_hud_iron_knight_first_hit_visibly_greys_fifth_pip_issue276() {
        // #276: a 5-HP Iron Knight draws five dots (two 16×8 pips + one trailing
        // 8×8 pip at loc_BE4C), and the first hit greys that trailing pip. The
        // prior code dropped the size-$00 trailing pip, so HP 5 and HP 4 both
        // rendered four dots — the "4 -> 4" the player kept reporting.
        let mut hp5 = sample_trace(GameMode::InGame);
        hp5.current_helmet_id = PlayerForm::IronKnight.helmet_id();
        hp5.number_hitpoints = 5;
        let mut hp4 = hp5.clone();
        hp4.number_hitpoints = 4;

        let hp5_scene = build_ingame_scene(&hp5, false, None);
        let hp4_scene = build_ingame_scene(&hp4, false, None);

        // Five dots render at both 5 and 4 HP (the trailing pip is not dropped).
        assert_eq!(count_hud_hitpoint_tiles(&hp5_scene), 5);
        assert_eq!(count_hud_hitpoint_tiles(&hp4_scene), 5);

        // The trailing odd pip lives at the third slot x = $B0. At 5 HP it is
        // filled ($822D -> tile_index 0); the first hit greys it
        // ($822F -> tile_index 2).
        let (_, hp5_tail, _) =
            indexed_tile_at_with_sheet_len(&hp5_scene, 0xB0 - 128, 0xA4 - 128, 4)
                .expect("filled trailing 5th pip at 5 HP");
        assert_eq!(
            hp5_tail.tile_index, 0,
            "5 HP: trailing pip is filled ($822D)"
        );
        let (_, hp4_tail, _) =
            indexed_tile_at_with_sheet_len(&hp4_scene, 0xB0 - 128, 0xA4 - 128, 4)
                .expect("grey trailing 5th pip at 4 HP");
        assert_eq!(
            hp4_tail.tile_index, 2,
            "4 HP: the first hit greys the trailing pip ($822F) — the change is visible"
        );

        // The trailing pip is a single 8×8 (size $00) tile, so nothing renders
        // at the second-tile position $B8.
        assert!(
            indexed_tile_at_with_sheet_len(&hp5_scene, 0xB8 - 128, 0xA4 - 128, 4).is_none(),
            "trailing pip is one 8×8 tile, not a 16×8 two-tile sprite"
        );
    }

    #[test]
    fn game_hud_non_iron_later_hit_visibly_decrements_loc_be34() {
        let mut before_hit = sample_trace(GameMode::InGame);
        before_hit.current_helmet_id = PlayerForm::RedStealth.helmet_id();
        before_hit.number_hitpoints = 2;

        let mut after_hit = before_hit.clone();
        after_hit.number_hitpoints -= 1;

        let before_hit_scene = build_ingame_scene(&before_hit, false, None);
        let after_hit_scene = build_ingame_scene(&after_hit, false, None);

        let (_, before_tile, _) =
            indexed_tile_at_with_sheet_len(&before_hit_scene, 0x90 - 128, 0xA4 - 128, 4)
                .expect("pre-hit Red Stealth pip");
        let (_, hit_first, _) =
            indexed_tile_at_with_sheet_len(&after_hit_scene, 0x90 - 128, 0xA4 - 128, 4)
                .expect("post-hit Red Stealth pip");

        assert_eq!(
            before_tile.tile_index, 0,
            "ASM sub_BDC0 loc_BE20: a non-Iron form at 2 HP renders one visible full pip"
        );
        assert_eq!(
            hit_first.tile_index, 1,
            "ASM loc_B700 decrements non-Iron helmet HP 2 -> 1, and sub_BDC0 loc_BE34 renders the slot as a visible half pip"
        );
        // A 3-HP non-Iron form has an odd trailing slot (loc_BE74): with 2 (or
        // 1) of 3 HP the third dot is spent, so it draws a VISIBLE grey 8×8 pip
        // ($822F, tile_index 2) — the prior code wrongly dropped it (#276).
        let (_, before_tail, _) =
            indexed_tile_at_with_sheet_len(&before_hit_scene, 0xA0 - 128, 0xA4 - 128, 4)
                .expect("trailing grey pip for the spent 3rd HP at 2 HP");
        assert_eq!(
            before_tail.tile_index, 2,
            "ASM sub_BDC0 loc_BE74: the odd 3rd HP slot draws a grey empty pip when spent"
        );
        let (_, after_tail, _) =
            indexed_tile_at_with_sheet_len(&after_hit_scene, 0xA0 - 128, 0xA4 - 128, 4)
                .expect("trailing grey pip for the spent 3rd HP at 1 HP");
        assert_eq!(
            after_tail.tile_index, 2,
            "ASM sub_BDC0 loc_BE74: the odd 3rd HP slot stays grey at 1 HP"
        );
    }

    #[test]
    fn game_hud_includes_extra_hitpoint_slots_for_grey_empty_pips() {
        let mut trace = sample_trace(GameMode::InGame);
        trace.current_helmet_id = PlayerForm::IronKnight.helmet_id();
        trace.number_hitpoints = 4;
        trace.extra_hitpoint_slots = 2;

        let scene = build_ingame_scene(&trace, false, None);

        let (_, empty_pip, _) = indexed_tile_at_with_sheet_len(&scene, 0xB0 - 128, 0xA4 - 128, 4)
            .expect("extra empty hitpoint pip");
        let (_, empty_tail, _) = indexed_tile_at_with_sheet_len(&scene, 0xB8 - 128, 0xA4 - 128, 4)
            .expect("second tile of extra empty hitpoint sprite");

        assert_eq!(
            empty_pip.tile_index, 2,
            "ASM Make_SpriteAttr_HUD loc_BE5C adds Extra_hitpoint_slots to the helmet capacity and draws empty tile $822F for the now-empty extra slot"
        );
        assert_eq!(
            empty_tail.tile_index, 3,
            "ASM Make_SpriteAttr_HUD loc_BE5C size byte #4 draws the following grey tile for the extra empty slot"
        );
    }

    #[test]
    fn play_native_loads_gems_from_generated_bundle_output() -> anyhow::Result<()> {
        let fixture = ManifestFixture::new()?;
        let mut manifest = fixture.manifest.clone();
        manifest.source_root = fixture._root.path().join("upstream");
        manifest.output_root = fixture._root.path().join("generated");
        let source_sound_dir = manifest.source_root.join("sound");
        let output_sound_dir = manifest.output_root.join("sound");

        assert_eq!(
            gems_sound_dir(&manifest),
            output_sound_dir,
            "play-native must load GEMS from the generated bundle; upstream source_root does not contain extracted sound binaries"
        );
        assert_ne!(
            gems_sound_dir(&manifest),
            source_sound_dir,
            "using source_root/sound drops native play into the no-music fallback on real extracted bundles"
        );
        Ok(())
    }

    #[test]
    fn settled_trace_audio_primes_existing_level_bgm_without_duplicate_event() {
        let fixture = ManifestFixture::new().expect("manifest fixture");
        let mut trace = sample_trace(GameMode::InGame);
        trace.current_level = LevelId(0x00);
        let frames = vec![trace.clone(), trace];

        assert_eq!(
            settled_audio_initial_bgm_id(
                &fixture.manifest,
                Some(NativeBgmCue::Level(LevelId(0x00)))
            ),
            Some(gems_bgm::CAVE),
            "settled dynamic audio exports must prime GEMS with the already-active level BGM; otherwise the waveform stays silent until the first block/pickup SFX"
        );

        let events = native_audio_events_for_trace_with_initial_bgm(
            &fixture.manifest,
            &frames,
            Some(NativeBgmCue::Level(LevelId(0x00))),
        );

        assert!(
            events.is_empty(),
            "settled audio transcripts should not log a duplicate level-BGM start when the route begins in-game"
        );
    }

    #[test]
    fn play_native_audio_queue_cap_is_measured_in_generated_frames() {
        let generated_stereo_samples = AUDIO_SAMPLES_PER_FRAME * 2;
        let one_generated_frame_bytes = generated_audio_sample_bytes(generated_stereo_samples);
        let target_generated_frame_bytes = live_audio_queue_target_bytes(generated_stereo_samples);

        assert!(
            should_queue_audio_frame(
                u32::try_from(target_generated_frame_bytes - 1).expect("fits u32"),
                generated_stereo_samples,
            ),
            "SDL_GetQueuedAudioSize reports bytes; just under the live queue target should accept the next audio frame"
        );
        assert!(
            !should_queue_audio_frame(
                u32::try_from(target_generated_frame_bytes).expect("fits u32"),
                generated_stereo_samples,
            ),
            "play-native should cap the SDL queue at the two-frame target so pickup and block SFX do not drift far behind the visual frame"
        );

        assert!(
            !should_resume_audio_queue_after_queue(0, generated_stereo_samples),
            "live SDL playback should stay paused after only one generated frame; starting with a one-frame queue risks immediate callback starvation"
        );
        assert!(
            should_resume_audio_queue_after_queue(
                u32::try_from(one_generated_frame_bytes).expect("fits u32"),
                generated_stereo_samples,
            ),
            "live SDL playback should resume once queuing this frame reaches the same two-frame target used by the queue cap"
        );
    }

    #[test]
    fn play_native_live_audio_backlog_retains_frames_while_sdl_queue_is_full() {
        let mut backlog = LiveAudioBacklog::new(2);
        let sfx_frame = vec![0.25; AUDIO_SAMPLES_PER_FRAME * 2];
        let bgm_frame = vec![-0.25; AUDIO_SAMPLES_PER_FRAME * 2];

        assert!(backlog.push_frame(sfx_frame.clone()));
        assert!(backlog.push_frame(bgm_frame.clone()));
        assert!(!backlog.push_frame(vec![0.0; AUDIO_SAMPLES_PER_FRAME * 2]));
        assert_eq!(backlog.pending_frames(), 2);
        assert_eq!(backlog.front_sample_count(), Some(sfx_frame.len()));
        assert_eq!(
            backlog.pop_front(),
            Some(sfx_frame),
            "the first GEMS frame generated at the SDL cap must be queued later, not discarded after advancing the audio engine (#264)"
        );
        assert_eq!(
            backlog.pop_front(),
            Some(bgm_frame),
            "deferred audio frames must keep generation order so block/diamond SFX stay attached to the BGM frame they were mixed into"
        );
        assert_eq!(backlog.pop_front(), None);
    }

    #[test]
    fn play_native_live_audio_sample_clock_uses_obtained_sdl_rate() {
        let mut default_clock = AudioSampleClock::new(AUDIO_SAMPLE_RATE);
        assert_eq!(
            default_clock.next_frame_samples(),
            AUDIO_SAMPLES_PER_FRAME,
            "44.1 kHz live playback must preserve the deterministic 735-sample frame used by recorded/oracle audio"
        );

        let mut host_clock = AudioSampleClock::new(48_000);
        assert_eq!(
            (0..3)
                .map(|_| host_clock.next_frame_samples())
                .collect::<Vec<_>>(),
            vec![800, 800, 800],
            "when SDL opens a 48 kHz device, play-native must generate 48 kHz-timed samples instead of feeding 44.1 kHz audio that the host stretches slow/low"
        );

        let mut fractional_clock = AudioSampleClock::new(44_117);
        let samples: usize = (0..60).map(|_| fractional_clock.next_frame_samples()).sum();
        assert_eq!(
            samples, 44_117,
            "fractional device rates must distribute the remainder across frames without long-term drift"
        );
    }

    #[test]
    fn play_native_recording_audio_generation_uses_obtained_sdl_rate_when_queueing() {
        let mut host_clock = AudioSampleClock::new(48_000);
        let plan = native_audio_generation_plan(true, 48_000, &mut host_clock);

        assert_eq!(
            plan,
            NativeAudioGenerationPlan {
                sample_rate: 48_000,
                samples_per_frame: 800,
            },
            "recorded play-native sessions still feed SDL; generating a 44.1 kHz frame for a 48 kHz queue stretches live capture audio slow/low"
        );

        let mut deterministic_clock = AudioSampleClock::new(48_000);
        let plan = native_audio_generation_plan(false, 48_000, &mut deterministic_clock);
        assert_eq!(
            plan,
            NativeAudioGenerationPlan {
                sample_rate: AUDIO_SAMPLE_RATE,
                samples_per_frame: AUDIO_SAMPLES_PER_FRAME,
            },
            "offline recording/audio artefact generation without SDL should keep the oracle-compatible 44.1 kHz frame cadence"
        );
    }

    #[test]
    fn play_native_fallback_sfx_uses_obtained_sdl_rate() {
        let mut sfx = SfxEngine::new(48_000);
        sfx.play(440.0, 1_000, 0.1);

        assert_eq!(
            sfx.active_tones[0].remaining_samples, 48_000,
            "fallback SFX must use SDL's obtained sample rate; fixed 44.1 kHz tones play slow/low on 48 kHz devices"
        );

        let samples = sfx.generate(800);
        assert_eq!(samples.len(), 800);
        assert_eq!(
            sfx.active_tones[0].remaining_samples, 47_200,
            "a 48 kHz live frame should consume the generated 800 samples from the fallback tone"
        );
    }

    #[test]
    fn play_native_audio_diagnostics_record_generated_and_queued_frames() {
        let mut diagnostics = NativeAudioDiagnostics::new(true, true, 2, 48_000);

        diagnostics.record_generated_frame(AUDIO_SAMPLES_PER_FRAME * 2);
        diagnostics.record_backlog_frame(1);
        diagnostics.record_queued_frame(AUDIO_SAMPLES_PER_FRAME * 2, 0);
        diagnostics.record_queue_prime();
        diagnostics.record_queue_resume();
        diagnostics.record_frame_timing(Duration::from_micros(12_000), FRAME_TIME);
        diagnostics.record_generated_frame(AUDIO_SAMPLES_PER_FRAME * 2);
        diagnostics.record_backlog_frame(2);
        diagnostics.record_backlog_full_frame(2);
        diagnostics.record_queue_clear();
        diagnostics.record_queue_prime();
        diagnostics.record_frame_timing(Duration::from_micros(20_000), FRAME_TIME);

        let lines = diagnostics.into_lines();
        assert!(
            lines.contains(&String::from("gems_loaded=true"))
                && lines.contains(&String::from("sdl_queue_present=true"))
                && lines.contains(&String::from("output_sample_rate=48000"))
                && lines.contains(&String::from("generated_frames=2"))
                && lines.contains(&String::from("queued_frames=1"))
                && lines.contains(&String::from("skipped_full_frames=0"))
                && lines.contains(&String::from("backlog_frames=2"))
                && lines.contains(&String::from("backlog_full_frames=1"))
                && lines.contains(&String::from("max_backlog_frames=2"))
                && lines.contains(&String::from("queue_errors=0"))
                && lines.contains(&String::from("queue_prime_events=2"))
                && lines.contains(&String::from("queue_resume_events=1"))
                && lines.contains(&String::from("queue_clear_events=1"))
                && lines.contains(&String::from("min_queue_bytes_before_queue=0"))
                && lines.contains(&String::from("max_queue_bytes_before_queue=0"))
                && lines.contains(&String::from("frame_timing_observations=2"))
                && lines.contains(&String::from("max_frame_micros=20000"))
                && lines.contains(&String::from("over_budget_frames=1"))
                && lines.contains(&String::from("total_over_budget_micros=3333")),
            "live play-native recordings need queue and timing evidence to distinguish rendered audio from SDL playback starvation or frame-loop slowdown"
        );
    }

    #[test]
    fn play_native_title_shell_uses_load_title_screen_bgm() -> anyhow::Result<()> {
        let fixture = ManifestFixture::new()?;
        let mut title = sample_trace(GameMode::TitleScreen);
        let mut options = sample_trace(GameMode::OptionsMenu);
        let intro = sample_trace(GameMode::IntroSequence1);

        assert_eq!(
            native_bgm_cue_for_trace(&title)
                .and_then(|cue| bgm_for_native_cue(&fixture.manifest, cue)),
            Some(gems_bgm::ICE),
            "ASM Load_TitleScreen queues bgm_Ice via j_PlaySound before title/menu objects are allocated"
        );
        assert_eq!(
            native_bgm_cue_for_trace(&intro)
                .and_then(|cue| bgm_for_native_cue(&fixture.manifest, cue)),
            Some(gems_bgm::ICE),
            "ASM Load_IntroSequence1 queues bgm_Ice via j_PlaySound before intro objects are allocated"
        );
        assert_eq!(
            native_bgm_cue_for_trace(&options)
                .and_then(|cue| bgm_for_native_cue(&fixture.manifest, cue)),
            Some(gems_bgm::CITY),
            "ASM Load_OptionMenu stops the title shell and queues bgm_City"
        );

        title.game_mode = GameMode::SegaScreen;
        options.game_mode = GameMode::IntroSequence6;
        assert_eq!(native_bgm_cue_for_trace(&title), None);
        assert_eq!(
            native_bgm_cue_for_trace(&options),
            Some(NativeBgmCue::TitleShell),
            "late IntroSequence6 title/menu states should recover the shell cue when native enters them directly"
        );
        Ok(())
    }

    #[test]
    fn play_native_sega_screen_stop_music_lets_intro_restart_ice() -> anyhow::Result<()> {
        let fixture = ManifestFixture::new()?;
        let sega = sample_trace(GameMode::SegaScreen);
        let intro = sample_trace(GameMode::IntroSequence1);

        assert_eq!(
            native_bgm_state_after_trace(&fixture.manifest, &intro, None),
            (
                Some(NativeBgmCue::TitleShell),
                Some(NativeBgmPlaybackCommand::StopMusicThenStart(gems_bgm::ICE)),
            ),
            "ASM Load_IntroSequence1 (kid.asm:29068-29070) runs StopMusic even on a cold intro load before queuing bgm_Ice"
        );

        let (after_sega, sega_command) =
            native_bgm_state_after_trace(&fixture.manifest, &sega, Some(NativeBgmCue::TitleShell));
        assert_eq!(
            (after_sega, sega_command),
            (None, Some(NativeBgmPlaybackCommand::StopMusic)),
            "ASM Load_SegaScreen (kid.asm:25444-25445) calls StopMusic before the intro shell reload"
        );

        assert_eq!(
            native_bgm_state_after_trace(&fixture.manifest, &intro, after_sega),
            (
                Some(NativeBgmCue::TitleShell),
                Some(NativeBgmPlaybackCommand::StopMusicThenStart(gems_bgm::ICE)),
            ),
            "ASM Load_IntroSequence1 (kid.asm:29068-29070) stops music, then queues bgm_Ice"
        );
        Ok(())
    }

    #[test]
    fn play_native_boot_idle_queues_intro_music_on_first_intro_frame() -> anyhow::Result<()> {
        let fixture = ManifestFixture::new()?;
        let mut runtime = NativeRuntime::new(fixture.manifest.clone());
        let mut trace = Vec::new();

        for _ in 0..240 {
            trace.push(runtime.step_frame(FrameInput::default()));
        }

        let first_intro_frame = trace
            .iter()
            .position(|frame| frame.game_mode == GameMode::IntroSequence1)
            .expect("boot idle route should reach Load_IntroSequence1");
        let events = native_audio_events_for_trace(&fixture.manifest, &trace);
        assert_eq!(
            events.first().map(String::as_str),
            Some(
                "frame=173 mode=IntroSequence1 level=00 active_before=None active_after=Some(TitleShell) command=StopMusicThenStart(3)"
            ),
            "ASM Load_IntroSequence1 (kid.asm:29068-29070) is the first native boot frame that should run StopMusic then queue bgm_Ice"
        );
        assert!(
            events[0].starts_with(&format!("frame={first_intro_frame} ")),
            "intro BGM should be queued on the first observed IntroSequence1 frame, not later"
        );
        Ok(())
    }

    #[test]
    fn native_bgm_event_log_records_mode_cue_and_command() {
        let trace = sample_trace(GameMode::IntroSequence1);

        let event = format_native_bgm_event(
            173,
            &trace,
            None,
            Some(NativeBgmCue::TitleShell),
            NativeBgmPlaybackCommand::StopMusicThenStart(gems_bgm::ICE),
        );

        assert!(
            event.contains("frame=173")
                && event.contains("mode=IntroSequence1")
                && event.contains("active_after=Some(TitleShell)")
                && event.contains("command=StopMusicThenStart(3)"),
            "the recording artefact should show whether ASM Load_IntroSequence1 queued the title-shell music path"
        );
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    enum RecordingAudioAction {
        StartSequence(u8),
        StopSequence(u8),
        StopMusic,
        SetMusicSpeed(u8),
        StepFrame,
        GenerateSamples(usize),
    }

    #[derive(Default)]
    struct RecordingAudioSink {
        actions: Vec<RecordingAudioAction>,
    }

    impl NativeAudioSink for RecordingAudioSink {
        fn start_sequence(&mut self, sound_id: u8) {
            self.actions
                .push(RecordingAudioAction::StartSequence(sound_id));
        }

        fn stop_sequence(&mut self, sound_id: u8) {
            self.actions
                .push(RecordingAudioAction::StopSequence(sound_id));
        }

        fn stop_music(&mut self) {
            self.actions.push(RecordingAudioAction::StopMusic);
        }

        fn set_music_speed(&mut self, speed: u8) {
            self.actions
                .push(RecordingAudioAction::SetMusicSpeed(speed));
        }

        fn step_frame(&mut self) {
            self.actions.push(RecordingAudioAction::StepFrame);
        }

        fn generate_samples(&mut self, sample_count: usize) -> Vec<f32> {
            self.actions
                .push(RecordingAudioAction::GenerateSamples(sample_count));
            vec![0.0; sample_count * 2]
        }
    }

    #[test]
    fn scripted_recording_audio_render_delays_stopmusic_then_start_until_next_frame()
    -> anyhow::Result<()> {
        let fixture = ManifestFixture::new()?;
        let intro = sample_trace(GameMode::IntroSequence1);
        let title = sample_trace(GameMode::TitleScreen);
        let mut sink = RecordingAudioSink::default();

        let samples = render_native_audio_samples_for_trace(
            &fixture.manifest,
            &[intro, title],
            None,
            &mut sink,
        );

        assert_eq!(samples.len(), 2 * AUDIO_SAMPLES_PER_FRAME * 2);
        assert_eq!(
            sink.actions,
            vec![
                RecordingAudioAction::StopMusic,
                RecordingAudioAction::StepFrame,
                RecordingAudioAction::GenerateSamples(AUDIO_SAMPLES_PER_FRAME),
                RecordingAudioAction::StartSequence(gems_bgm::ICE),
                RecordingAudioAction::StepFrame,
                RecordingAudioAction::GenerateSamples(AUDIO_SAMPLES_PER_FRAME),
            ],
            "ASM Load_IntroSequence1 (kid.asm:29068-29070) runs StopMusic before bgm_Ice; native audio must defer the new GEMS start until the frame after the reset"
        );
        Ok(())
    }

    #[test]
    fn scripted_recording_audio_render_suppresses_same_frame_sfx_after_stopmusic()
    -> anyhow::Result<()> {
        let fixture = ManifestFixture::new()?;
        let mut draining = sample_trace(GameMode::InGame);
        draining.level_complete = true;
        draining.level_complete_score_counter_elapsed_frames = 1;
        let mut transition = sample_trace(GameMode::LevelTransition);
        transition.level_complete = true;
        transition.level_complete_elapsed_frames = 1;
        transition.level_complete_score_counter_elapsed_frames = 1;
        transition.level_complete_score_counter_drained = true;
        transition.played_sound_effect_ids = vec![gems_sfx::SCORE_COUNTER];
        let mut sink = RecordingAudioSink::default();

        render_native_audio_samples_for_trace(
            &fixture.manifest,
            &[draining, transition],
            Some(NativeBgmCue::ScoreBoard),
            &mut sink,
        );

        assert!(
            !sink.actions.contains(&RecordingAudioAction::StartSequence(
                gems_sfx::SCORE_COUNTER
            )),
            "ASM loc_D7F4 (kid.asm:16906) score-counter replay is pre-reset state when the same condensed LevelTransition frame runs StopMusic"
        );
        assert!(
            sink.actions
                .contains(&RecordingAudioAction::StartSequence(gems_bgm::SCORE_BOARD)),
            "settled EOL audio starts from the already-active score-board BGM cue before rendering frame samples"
        );
        assert!(
            sink.actions.contains(&RecordingAudioAction::StopMusic),
            "ASM Load_InGame (kid.asm:23623-23629) hard-resets GEMS during the transition shell"
        );
        Ok(())
    }

    #[test]
    fn scripted_recording_audio_render_applies_start_and_stop_sequence_commands()
    -> anyhow::Result<()> {
        let fixture = ManifestFixture::new()?;
        let prev = sample_trace(GameMode::InGame);
        let mut curr = sample_trace(GameMode::InGame);
        curr.played_sound_commands = vec![
            NativeSoundCommand::start_sequence(gems_sfx::NAVIGATE_JINGLE),
            NativeSoundCommand::stop_sequence(gems_sfx::SCORE_COUNTER),
        ];
        curr.played_sound_effect_ids = vec![gems_sfx::NAVIGATE_JINGLE, gems_sfx::SCORE_COUNTER];
        let mut sink = RecordingAudioSink::default();

        render_native_audio_samples_for_trace(
            &fixture.manifest,
            &[prev, curr],
            Some(NativeBgmCue::Level(LevelId(0))),
            &mut sink,
        );

        assert!(
            sink.actions.contains(&RecordingAudioAction::StartSequence(
                gems_sfx::NAVIGATE_JINGLE
            )),
            "ASM sub_E151C queues command $10 for ordinary SFX; recording audio must replay it as a GEMS start-sequence command"
        );
        assert!(
            sink.actions
                .contains(&RecordingAudioAction::StopSequence(gems_sfx::SCORE_COUNTER)),
            "ASM sub_E153E / PlaySound2 queues command $12 for stop-sequence SFX such as loc_D7F4's score-counter cleanup"
        );
        Ok(())
    }

    #[test]
    fn scripted_recording_audio_events_follow_trace_mode_and_sfx_transitions() -> anyhow::Result<()>
    {
        let fixture = ManifestFixture::new()?;
        let mut intro = sample_trace(GameMode::IntroSequence1);
        intro.frame_index = 173;
        let mut options = sample_trace(GameMode::OptionsMenu);
        options.frame_index = 2800;
        options.played_sound_effect_ids = vec![gems_sfx::NAVIGATE_JINGLE];
        let mut gameplay = sample_trace(GameMode::InGame);
        gameplay.frame_index = 2900;
        gameplay.played_sound_effect_ids = vec![gems_sfx::NAVIGATE_JINGLE, gems_sfx::DESTROY_BLOCK];

        let events = native_audio_events_for_trace(&fixture.manifest, &[intro, options, gameplay]);

        assert_eq!(
            events,
            vec![
                String::from(
                    "frame=0 mode=IntroSequence1 level=00 active_before=None active_after=Some(TitleShell) command=StopMusicThenStart(3)"
                ),
                String::from(
                    "frame=1 mode=OptionsMenu level=00 active_before=Some(TitleShell) active_after=Some(Options) command=StopMusicThenStart(101)"
                ),
                format!(
                    "frame=1 mode=OptionsMenu level=00 command=StartSequence({})",
                    gems_sfx::NAVIGATE_JINGLE
                ),
                String::from(
                    "frame=2 mode=InGame level=00 active_before=Some(Options) active_after=Some(Level(LevelId(0))) command=StopMusicThenStart(7)"
                ),
                format!(
                    "frame=2 mode=InGame level=00 command=StartSequence({})",
                    gems_sfx::DESTROY_BLOCK
                ),
            ],
            "scripted recording bundles should expose both BGM transitions and ASM/core queued SFX as live play-native audio events"
        );
        Ok(())
    }

    #[test]
    fn settled_audio_events_do_not_restart_active_level_bgm() -> anyhow::Result<()> {
        let fixture = ManifestFixture::new()?;
        let mut previous = sample_trace(GameMode::InGame);
        previous.current_level = LevelId(0);
        let mut gameplay = sample_trace(GameMode::InGame);
        gameplay.current_level = LevelId(0);
        gameplay.played_sound_effect_ids = vec![gems_sfx::RED_STEALTH_SWORD];

        let cold_events =
            native_audio_events_for_trace(&fixture.manifest, &[previous.clone(), gameplay.clone()]);
        let settled_events = native_audio_events_for_trace_with_initial_bgm(
            &fixture.manifest,
            &[previous, gameplay],
            Some(NativeBgmCue::Level(LevelId(0))),
        );

        assert_eq!(
            cold_events.first().map(String::as_str),
            Some(
                "frame=0 mode=InGame level=00 active_before=None active_after=Some(Level(LevelId(0))) command=StopMusicThenStart(7)"
            ),
            "cold play-native recordings must mirror ASM Load_InGame StopMusic before ASM loc_12004 queues level music"
        );
        assert!(
            settled_events
                .iter()
                .all(|event| !event.contains("command=Start(")),
            "dynamic oracle probes begin after level-settle, so native comparison audio must not restart the level BGM at scenario frame 0"
        );
        assert!(
            settled_events
                .iter()
                .any(|event| event.contains("command=StartSequence(35)")),
            "settled dynamic probes must still preserve ASM sub_868E Red Stealth sword SFX events"
        );
        Ok(())
    }

    #[test]
    fn scripted_recording_audio_artifacts_render_samples_when_gems_assets_present()
    -> anyhow::Result<()> {
        let fixture = ManifestFixture::new()?;
        fixture.write_dummy_gems_assets()?;
        let intro = sample_trace(GameMode::IntroSequence1);
        let mut title = sample_trace(GameMode::TitleScreen);
        title.played_sound_effect_ids = vec![gems_sfx::NAVIGATE_JINGLE];
        let trace = [intro, title];

        let artifacts = native_audio_artifacts_for_trace(&fixture.manifest, &trace);

        let samples = artifacts
            .audio_samples
            .as_ref()
            .expect("dummy committed-test GEMS assets should enter the rendered-audio path");
        assert_eq!(samples.len(), trace.len() * AUDIO_SAMPLES_PER_FRAME * 2);
        assert_eq!(artifacts.audio_channels, 2);
        assert_eq!(
            artifacts.audio_events,
            vec![
                String::from(
                    "frame=0 mode=IntroSequence1 level=00 active_before=None active_after=Some(TitleShell) command=StopMusicThenStart(3)"
                ),
                format!(
                    "frame=1 mode=TitleScreen level=00 command=StartSequence({})",
                    gems_sfx::NAVIGATE_JINGLE
                ),
            ],
            "recording bundles must retain ASM-derived event evidence when waveform artefacts are also rendered"
        );
        Ok(())
    }

    #[test]
    fn scripted_recording_audio_artifacts_keep_events_when_gems_assets_missing()
    -> anyhow::Result<()> {
        let fixture = ManifestFixture::new()?;
        let trace = [sample_trace(GameMode::IntroSequence1)];

        let artifacts = native_audio_artifacts_for_trace(&fixture.manifest, &trace);

        assert_eq!(
            artifacts.audio_samples, None,
            "the tiny unit manifest has no GEMS binaries, so rendered audio is verified with real extracted assets instead"
        );
        assert_eq!(artifacts.audio_channels, 0);
        assert_eq!(
            artifacts.audio_events,
            vec![String::from(
                "frame=0 mode=IntroSequence1 level=00 active_before=None active_after=Some(TitleShell) command=StopMusicThenStart(3)"
            )],
            "record-scenario-file should still persist ASM-derived cue evidence even when waveform rendering is unavailable"
        );
        Ok(())
    }

    #[test]
    fn play_native_level_bgm_cue_uses_current_level_theme() -> anyhow::Result<()> {
        let fixture = ManifestFixture::new()?;
        let trace = sample_trace(GameMode::InGame);

        assert_eq!(
            native_bgm_cue_for_trace(&trace)
                .and_then(|cue| bgm_for_native_cue(&fixture.manifest, cue)),
            Some(gems_bgm::CAVE),
            "fixture level theme 0x08 should still route through ASM loc_12004 / unk_1201E"
        );
        Ok(())
    }

    #[test]
    fn play_native_level_bgm_cue_uses_murderwall_override() -> anyhow::Result<()> {
        let mut fixture = ManifestFixture::new()?;
        let header = fixture.manifest.levels[0]
            .header_metadata
            .as_mut()
            .expect("fixture level header");
        header.level_theme = 0x09;
        header.murder_wall_flags = 0b10;
        let trace = sample_trace(GameMode::InGame);

        assert_eq!(
            native_bgm_cue_for_trace(&trace)
                .and_then(|cue| bgm_for_native_cue(&fixture.manifest, cue)),
            Some(gems_bgm::MURDERWALL),
            "ASM loc_11FD2 (kid.asm:24183-24186) checks `MurderWall_flag` before loc_12004, so a murder-wall level must play `bgm_Murderwall` instead of the foreground-theme BGM"
        );
        Ok(())
    }

    #[test]
    fn play_native_level_bgm_prefers_core_queued_loc_11fd2_id() -> anyhow::Result<()> {
        let fixture = ManifestFixture::new()?;
        let mut trace = sample_trace(GameMode::InGame);
        trace.played_bgm_ids = vec![gems_bgm::MURDERWALL];

        assert_eq!(
            native_bgm_cue_for_trace(&trace)
                .and_then(|cue| bgm_for_native_cue(&fixture.manifest, cue)),
            Some(gems_bgm::MURDERWALL),
            "when wildside-core has queued ASM loc_11FD2 into played_bgm_ids, play-native should replay that exact command instead of re-resolving the level header"
        );
        Ok(())
    }

    #[test]
    fn play_native_core_queued_level_bgm_runs_load_ingame_stopmusic_from_cold_cue()
    -> anyhow::Result<()> {
        let fixture = ManifestFixture::new()?;
        let mut trace = sample_trace(GameMode::InGame);
        trace.played_bgm_ids = vec![gems_bgm::MURDERWALL];

        assert_eq!(
            native_bgm_state_after_trace(&fixture.manifest, &trace, None),
            (
                Some(NativeBgmCue::CoreQueued(gems_bgm::MURDERWALL)),
                Some(NativeBgmPlaybackCommand::StopMusicThenStart(
                    gems_bgm::MURDERWALL
                )),
            ),
            "ASM Load_InGame (kid.asm:23623) runs StopMusic before loc_11FD2/loc_12004 queues the level BGM, even when native has no active cue to replace"
        );
        Ok(())
    }

    #[test]
    fn play_native_bgm_transition_stops_previous_sequence_before_starting_level_music()
    -> anyhow::Result<()> {
        let fixture = ManifestFixture::new()?;
        let trace = sample_trace(GameMode::InGame);

        assert_eq!(
            native_bgm_state_after_trace(&fixture.manifest, &trace, Some(NativeBgmCue::TitleShell)),
            (
                Some(NativeBgmCue::Level(LevelId(0))),
                Some(NativeBgmPlaybackCommand::StopMusicThenStart(gems_bgm::CAVE)),
            ),
            "ASM StopMusic reloads/resets GEMS before title-to-gameplay BGM replacement; `$16` stop-all leaves stale title sequence state in native playback"
        );
        Ok(())
    }

    #[test]
    fn play_native_options_bgm_replaces_title_shell_with_city() -> anyhow::Result<()> {
        let fixture = ManifestFixture::new()?;
        let trace = sample_trace(GameMode::OptionsMenu);

        assert_eq!(
            native_bgm_state_after_trace(&fixture.manifest, &trace, Some(NativeBgmCue::TitleShell)),
            (
                Some(NativeBgmCue::Options),
                Some(NativeBgmPlaybackCommand::StopMusicThenStart(gems_bgm::CITY)),
            ),
            "ASM Load_OptionMenu (kid.asm:31570-31572) runs StopMusic, then PlaySound(bgm_City)"
        );
        Ok(())
    }

    #[test]
    fn play_native_title_start_stops_shell_before_level_music() -> anyhow::Result<()> {
        let fixture = ManifestFixture::new()?;
        let transition = sample_trace(GameMode::LevelTransition);
        let level = sample_trace(GameMode::InGame);

        let (active_after_gap, gap_command) = native_bgm_state_after_trace(
            &fixture.manifest,
            &transition,
            Some(NativeBgmCue::TitleShell),
        );
        assert_eq!(
            (active_after_gap, gap_command),
            (
                Some(NativeBgmCue::StoppedForTransition),
                Some(NativeBgmPlaybackCommand::StopMusic)
            ),
            "ASM Obj_TitleMenu calls StopMusic as soon as Start leaves the title shell; native keeps a transition sentinel so the later Load_InGame BGM starts from the reset state",
        );

        assert_eq!(
            native_bgm_state_after_trace(&fixture.manifest, &level, active_after_gap),
            (
                Some(NativeBgmCue::Level(LevelId(0))),
                Some(NativeBgmPlaybackCommand::Start(gems_bgm::CAVE)),
            ),
            "after the title shell has already run ASM StopMusic, Load_InGame can queue the level BGM directly"
        );
        Ok(())
    }

    #[test]
    fn play_native_death_respawn_stops_level_bgm_before_loader_restart() -> anyhow::Result<()> {
        let fixture = ManifestFixture::new()?;
        let transition = sample_trace(GameMode::LevelTransition);
        let level = sample_trace(GameMode::InGame);

        let (active_after_gap, gap_command) = native_bgm_state_after_trace(
            &fixture.manifest,
            &transition,
            Some(NativeBgmCue::Level(LevelId(0))),
        );
        assert_eq!(
            (active_after_gap, gap_command),
            (
                Some(NativeBgmCue::StoppedForTransition),
                Some(NativeBgmPlaybackCommand::StopMusic)
            ),
            "ASM lose_life enters Teleport/loc_6E2; Load_InGame (kid.asm:23623-23629) then runs StopMusic before loc_12004 queues the reloaded level BGM"
        );

        assert_eq!(
            native_bgm_state_after_trace(&fixture.manifest, &level, active_after_gap),
            (
                Some(NativeBgmCue::Level(LevelId(0))),
                Some(NativeBgmPlaybackCommand::Start(gems_bgm::CAVE)),
            ),
            "after the death-respawn loader reset, the active level cue should restart from the foreground theme instead of continuing over the black hold"
        );
        Ok(())
    }

    #[test]
    fn play_native_eol_transition_stops_scoreboard_before_next_level_bgm() -> anyhow::Result<()> {
        let fixture = ManifestFixture::new()?;
        let transition = sample_trace(GameMode::LevelTransition);
        let level = sample_trace(GameMode::InGame);

        let (active_after_gap, gap_command) = native_bgm_state_after_trace(
            &fixture.manifest,
            &transition,
            Some(NativeBgmCue::ScoreBoard),
        );
        assert_eq!(
            (active_after_gap, gap_command),
            (
                Some(NativeBgmCue::StoppedForTransition),
                Some(NativeBgmPlaybackCommand::StopMusic)
            ),
            "ASM loc_D7F4 replays sfx_Score_counter, then Teleport enters mode $0008; Load_InGame (kid.asm:23623-23629) must run StopMusic so the counter sound cannot bleed through the next level card"
        );

        assert_eq!(
            native_bgm_state_after_trace(&fixture.manifest, &level, active_after_gap),
            (
                Some(NativeBgmCue::Level(LevelId(0))),
                Some(NativeBgmPlaybackCommand::Start(gems_bgm::CAVE)),
            ),
            "after the EOL loader reset, loc_12004 queues the next level BGM from a clean GEMS state"
        );
        Ok(())
    }

    #[test]
    fn play_native_eol_transition_stops_even_after_cue_tracker_cleared() -> anyhow::Result<()> {
        let fixture = ManifestFixture::new()?;
        let mut transition = sample_trace(GameMode::LevelTransition);
        transition.level_complete = true;
        transition.level_complete_elapsed_frames = 1;
        transition.level_complete_score_counter_drained = true;
        let level = sample_trace(GameMode::InGame);

        let (active_after_gap, gap_command) =
            native_bgm_state_after_trace(&fixture.manifest, &transition, None);
        assert_eq!(
            (active_after_gap, gap_command),
            (
                Some(NativeBgmCue::StoppedForTransition),
                Some(NativeBgmPlaybackCommand::StopMusic)
            ),
            "live play-native can reach the EOL title-card shell after the cue tracker is already clear; StopMusic must still hard-reset GEMS so the loc_D7F4 score-counter tail cannot keep releasing"
        );

        assert_eq!(
            native_bgm_state_after_trace(&fixture.manifest, &transition, active_after_gap),
            (Some(NativeBgmCue::StoppedForTransition), None),
            "the forced EOL StopMusic should be one-shot across a held LevelTransition card"
        );

        assert_eq!(
            native_bgm_state_after_trace(&fixture.manifest, &level, active_after_gap),
            (
                Some(NativeBgmCue::Level(LevelId(0))),
                Some(NativeBgmPlaybackCommand::Start(gems_bgm::CAVE)),
            ),
            "after the hard reset sentinel, the next level BGM starts cleanly without another stop-all"
        );
        Ok(())
    }

    #[test]
    fn play_native_stopmusic_flushes_host_audio_queue_for_eol_handoff() {
        assert!(
            native_bgm_command_flushes_queued_audio(NativeBgmPlaybackCommand::StopMusic),
            "ASM StopMusic resets GEMS immediately; play-native must also drop queued SDL samples so loc_D7F4's score-counter tail cannot bleed over the title-card shell"
        );
        assert!(
            native_bgm_command_flushes_queued_audio(NativeBgmPlaybackCommand::StopMusicThenStart(
                gems_bgm::CAVE
            )),
            "StopMusicThenStart includes the same immediate GEMS reset before the next bgm starts"
        );
        assert!(
            !native_bgm_command_flushes_queued_audio(NativeBgmPlaybackCommand::Start(
                gems_bgm::CAVE
            )),
            "ordinary BGM starts should not clear already-generated audio"
        );
    }

    #[test]
    fn play_native_stopmusic_clears_fallback_sfx_for_eol_handoff() {
        let mut sfx = SfxEngine::new(48_000);
        sfx.play(1_200.0, 500, 0.2);
        assert!(
            sfx.generate(800).iter().any(|sample| *sample != 0.0),
            "test setup should produce an audible fallback tone before StopMusic"
        );

        if native_bgm_command_flushes_queued_audio(NativeBgmPlaybackCommand::StopMusic) {
            sfx.clear();
        }

        assert!(
            sfx.generate(800).iter().all(|sample| *sample == 0.0),
            "ASM StopMusic (kid.asm:49727) must silence fallback tones as well as the SDL/GEMS queue so loc_D7F4's score-counter tail cannot bleed into the next title-card shell"
        );
    }

    #[test]
    fn play_native_eol_stopmusic_suppresses_same_frame_score_counter_sfx() -> anyhow::Result<()> {
        let fixture = ManifestFixture::new()?;
        let mut draining = sample_trace(GameMode::InGame);
        draining.level_complete = true;
        draining.level_complete_score_counter_elapsed_frames = 1;

        let mut transition = sample_trace(GameMode::LevelTransition);
        transition.level_complete = true;
        transition.level_complete_elapsed_frames = 1;
        transition.level_complete_score_counter_elapsed_frames = 1;
        transition.level_complete_score_counter_drained = true;

        let events = native_audio_events_for_trace_with_initial_bgm(
            &fixture.manifest,
            &[draining, transition],
            Some(NativeBgmCue::ScoreBoard),
        );

        assert!(
            events
                .iter()
                .any(|event| event.contains("command=StopMusic")),
            "ASM Load_InGame (kid.asm:23623-23629) must reset GEMS after the EOL score counter before the next loading/title-card shell"
        );
        assert!(
            !events
                .iter()
                .any(|event| event.contains("command=StartSequence(12)")),
            "native condenses ASM loc_D7F4's `sfx_Score_counter` replay and Load_InGame StopMusic into one LevelTransition trace frame; replaying $0C after StopMusic reintroduces the audible counter tail"
        );
        Ok(())
    }

    #[test]
    fn play_native_initial_level_bgm_runs_load_ingame_stopmusic() -> anyhow::Result<()> {
        let fixture = ManifestFixture::new()?;
        let trace = sample_trace(GameMode::InGame);

        assert_eq!(
            native_bgm_state_after_trace(&fixture.manifest, &trace, None),
            (
                Some(NativeBgmCue::Level(LevelId(0))),
                Some(NativeBgmPlaybackCommand::StopMusicThenStart(gems_bgm::CAVE)),
            ),
            "ASM Load_InGame (kid.asm:23623) runs StopMusic before loc_12004 queues the first level BGM; this uses the real GEMS reset, not the old stop-all command"
        );
        Ok(())
    }

    #[test]
    fn play_native_cold_load_ingame_stopmusic_suppresses_stale_tank_sfx_issue339()
    -> anyhow::Result<()> {
        let fixture = ManifestFixture::new()?;
        let previous = sample_trace(GameMode::LevelTransition);
        let mut level = sample_trace(GameMode::InGame);
        level.played_sound_commands = vec![
            NativeSoundCommand::start_sequence(gems_sfx::TANK_DRIVING),
            NativeSoundCommand::start_sequence(gems_sfx::TANK_MOUTH_OPEN),
        ];
        level.played_sound_effect_ids = vec![gems_sfx::TANK_DRIVING, gems_sfx::TANK_MOUTH_OPEN];

        let events = native_audio_events_for_trace(&fixture.manifest, &[previous, level]);

        assert!(
            events.iter().any(|event| {
                event.contains("mode=InGame")
                    && event.contains("active_before=None")
                    && event.contains("command=StopMusicThenStart")
            }),
            "ASM Load_InGame (kid.asm:23623) must reset GEMS before loc_12004 queues level music on cold native level entry"
        );
        assert!(
            !events.iter().any(|event| event.contains(&format!(
                "command=StartSequence({})",
                gems_sfx::TANK_DRIVING
            ))),
            "native condenses stale Tank.asm loc_3B40A drive state into the same cold Load_InGame trace frame; replaying $1F after StopMusic keeps the #339 drone alive"
        );
        assert!(
            !events.iter().any(|event| event.contains(&format!(
                "command=StartSequence({})",
                gems_sfx::TANK_MOUTH_OPEN
            ))),
            "same-frame stale Tank.asm loc_3B41C mouth-open $3C must be pre-reset state, not post-load playback"
        );
        Ok(())
    }

    #[test]
    fn play_native_replays_core_queued_menu_jingle_sub_1bc26() {
        let mut prev = sample_trace(GameMode::TitleScreen);
        let mut curr = sample_trace(GameMode::TitleScreen);
        prev.played_sound_effect_ids = vec![gems_sfx::NAVIGATE_JINGLE];
        curr.played_sound_effect_ids = vec![gems_sfx::NAVIGATE_JINGLE, gems_sfx::NAVIGATE_JINGLE];

        assert_eq!(
            gems_event_sound_ids(&prev, &curr),
            vec![gems_sfx::NAVIGATE_JINGLE],
            "ASM sub_1BC26 queues sfx_Navigate_jingle through j_PlaySound; play-native must replay the newly queued core id"
        );
    }

    #[test]
    fn play_native_replays_core_queued_sfx_after_trace_reset() {
        let mut prev = sample_trace(GameMode::InGame);
        let mut curr = sample_trace(GameMode::InGame);
        prev.played_sound_effect_ids = vec![
            gems_sfx::NAVIGATE_JINGLE,
            gems_sfx::VOICE_OUCH_1,
            gems_sfx::VOICE_OUCH_2,
        ];
        curr.played_sound_effect_ids =
            vec![gems_sfx::MINI_HOPPING_SKULL_SCREAM, gems_sfx::THUNDERSTORM];

        assert_eq!(
            gems_event_sound_ids(&prev, &curr),
            vec![gems_sfx::MINI_HOPPING_SKULL_SCREAM, gems_sfx::THUNDERSTORM],
            "a restarted NativeRuntime resets the cumulative core sound list; the next trace frame must treat all current ASM-backed ids as new"
        );
    }

    #[test]
    fn play_native_replays_core_queued_transform_music_speed_loc_7e26() {
        let prev = sample_trace(GameMode::InGame);
        let mut curr = sample_trace(GameMode::InGame);
        curr.played_music_speed_updates = vec![0x7E];

        assert_eq!(
            gems_event_music_speed_updates(&prev, &curr),
            vec![0x7E],
            "ASM loc_7E26 queues unk_7ECC[Foreground_theme] through sub_E1330/sub_E1338; play-native must replay the core tempo byte"
        );
    }

    #[test]
    fn play_native_replays_core_queued_transform_music_speed_after_trace_reset() {
        let mut prev = sample_trace(GameMode::InGame);
        let mut curr = sample_trace(GameMode::InGame);
        prev.played_music_speed_updates = vec![0x7E, 0x7A];
        curr.played_music_speed_updates = vec![0x76];

        assert_eq!(
            gems_event_music_speed_updates(&prev, &curr),
            vec![0x76],
            "a restarted NativeRuntime resets the cumulative transform tempo list; the next trace frame must treat all current ASM-backed speed bytes as new"
        );
    }

    #[test]
    fn play_native_replays_drips_dripping_from_core_trace_loc_3345c() {
        let prev = sample_trace(GameMode::InGame);
        let mut curr = sample_trace(GameMode::InGame);
        curr.played_sound_effect_ids = vec![gems_sfx::DRIPS_DRIPPING];

        assert_eq!(
            gems_event_sound_ids(&prev, &curr),
            vec![gems_sfx::DRIPS_DRIPPING],
            "ASM Drips.asm loc_3345C queues sfx_Drips_dripping=$58 from the core trace"
        );
    }

    #[test]
    fn play_native_replays_jump_on_enemy_from_core_trace_loc_b6da() {
        let prev = sample_trace(GameMode::InGame);
        let mut curr = sample_trace(GameMode::InGame);
        curr.played_sound_effect_ids = vec![gems_sfx::JUMP_ON_ENEMY];
        curr.stomped_this_frame = true;

        assert_eq!(
            gems_event_sound_ids(&prev, &curr),
            vec![gems_sfx::JUMP_ON_ENEMY],
            "ASM Jump_On_Enemy (kid.asm:14607) queues sfx_Jump_on_enemy=$4C from the core trace before the loc_B6DA bounce tail"
        );
    }

    #[test]
    fn play_native_replays_ankh_prize_from_core_trace_touch_ankh() {
        let prev = sample_trace(GameMode::InGame);
        let mut curr = sample_trace(GameMode::InGame);
        curr.played_sound_effect_ids = vec![gems_sfx::ANKH_PRIZE];
        curr.number_lives = prev.number_lives + 1;

        assert_eq!(
            gems_event_sound_ids(&prev, &curr),
            vec![gems_sfx::ANKH_PRIZE],
            "ASM CharacterCollision_TouchAnkh (kid.asm:14883) queues sfx_Ankh_prize=$29 from the core trace"
        );
    }

    #[test]
    fn play_native_does_not_infer_jump_on_enemy_from_score_delta() {
        let prev = sample_trace(GameMode::InGame);
        let mut curr = sample_trace(GameMode::InGame);
        curr.score = prev.score + 1000;

        assert!(
            !gems_event_sound_ids(&prev, &curr).contains(&gems_sfx::JUMP_ON_ENEMY),
            "ASM Jump_On_Enemy (kid.asm:14607) is keyed to the stomp-bounce path, not to an advisory score delta"
        );
    }

    #[test]
    fn play_native_does_not_duplicate_core_queued_jump_sfx_loc_a480() {
        let mut prev = sample_trace(GameMode::InGame);
        let mut curr = sample_trace(GameMode::InGame);
        prev.kid_on_ground = Some(true);
        curr.kid_on_ground = Some(false);
        curr.kid_y_velocity = Some(-0x40000);
        curr.played_sound_effect_ids = vec![gems_sfx::JUMP];

        assert_eq!(
            gems_event_sound_ids(&prev, &curr),
            vec![gems_sfx::JUMP],
            "ASM sub_A432 loc_A480 now queues jump SFX from the core trace, so the native-player fallback must not play a duplicate jump cue"
        );
    }

    #[test]
    fn play_native_does_not_infer_jump_sfx_during_title_card_loc_1ae2e() {
        let mut prev = sample_trace(GameMode::LevelTransition);
        let mut curr = sample_trace(GameMode::LevelTransition);
        prev.kid_on_ground = Some(true);
        curr.kid_on_ground = Some(false);
        curr.kid_y_velocity = Some(-0x40000);

        assert!(
            !gems_event_sound_ids(&prev, &curr).contains(&gems_sfx::JUMP),
            "ASM loc_1AE2E only polls Start while the title card hibernates; play-native must not infer gameplay jump SFX from stale hidden-player trace deltas"
        );
    }

    #[test]
    fn fallback_sfx_does_not_infer_jump_during_title_card_loc_1ae2e() {
        let mut prev = sample_trace(GameMode::LevelTransition);
        let mut curr = sample_trace(GameMode::LevelTransition);
        prev.kid_on_ground = Some(true);
        curr.kid_on_ground = Some(false);
        curr.kid_y_velocity = Some(-0x40000);
        let mut sfx = SfxEngine::new(AUDIO_SAMPLE_RATE);

        detect_sfx_events(&prev, &curr, &mut sfx);

        assert!(
            sfx.active_tones.is_empty(),
            "ASM loc_1AE2E title-card hibernate must suppress the non-GEMS jump-tone fallback as well as the GEMS jump cue"
        );
    }

    #[test]
    fn fallback_sfx_emits_diamond_prize_from_contact_not_count_loc_2510c() {
        let prev = sample_trace(GameMode::InGame);
        let mut contact = sample_trace(GameMode::InGame);
        contact.played_sound_effect_ids = vec![gems_sfx::DIAMOND_PRIZE];
        let mut sfx = SfxEngine::new(AUDIO_SAMPLE_RATE);

        detect_sfx_events(&prev, &contact, &mut sfx);

        assert_eq!(
            sfx.active_tones.len(),
            1,
            "ASM PrizeDiamondCollected_Init queues sfx_Diamond_prize on contact; the fallback mixer must not wait for Number_Diamonds"
        );

        let mut count_increment = sample_trace(GameMode::InGame);
        count_increment.diamond_count = prev.diamond_count + 1;
        let mut delayed_sfx = SfxEngine::new(AUDIO_SAMPLE_RATE);

        detect_sfx_events(&prev, &count_increment, &mut delayed_sfx);

        assert!(
            delayed_sfx.active_tones.is_empty(),
            "the delayed Number_Diamonds increment is not a second fallback SFX trigger"
        );
    }

    #[test]
    fn fallback_sfx_replays_clock_prize_without_timer_duplicate_loc_ff92() {
        let prev = sample_trace(GameMode::InGame);
        let mut curr = sample_trace(GameMode::InGame);
        curr.played_sound_effect_ids = vec![gems_sfx::CLOCK_PRIZE];
        curr.level_timer_seconds = prev.level_timer_seconds + 180;
        let mut sfx = SfxEngine::new(AUDIO_SAMPLE_RATE);

        detect_sfx_events(&prev, &curr, &mut sfx);

        assert_eq!(
            sfx.active_tones.len(),
            2,
            "#342: the core queues sfx_Clock_prize (loc_10792) on the falling-clock spawn; with the timer-jump heuristic removed there must be no second fallback trigger"
        );
    }

    #[test]
    fn fallback_sfx_replays_metal_block_without_generic_block_crunch_loc_110d0() {
        let prev = sample_trace(GameMode::InGame);
        let mut curr = sample_trace(GameMode::InGame);
        curr.blocks_hit_this_frame = 1;
        curr.played_sound_effect_ids = vec![gems_sfx::METAL_BLOCK];
        let mut sfx = SfxEngine::new(AUDIO_SAMPLE_RATE);

        detect_sfx_events(&prev, &curr, &mut sfx);

        assert_eq!(
            sfx.active_tones.len(),
            2,
            "ASM loc_110D0 queues sfx_Metal_block for springy blocks; fallback must play that exact cue and suppress generic sfx_Block_hit"
        );
    }

    #[test]
    fn fallback_sfx_uses_core_transform_without_legacy_duplicate_loc_7cd0() {
        let mut prev = sample_trace(GameMode::InGame);
        let mut curr = sample_trace(GameMode::InGame);
        prev.current_helmet_id = PlayerForm::TheKid.helmet_id();
        curr.current_helmet_id = PlayerForm::RedStealth.helmet_id();
        curr.played_sound_effect_ids = vec![gems_sfx::RED_STEALTH_TRANSFORM];
        let mut sfx = SfxEngine::new(AUDIO_SAMPLE_RATE);

        detect_sfx_events(&prev, &curr, &mut sfx);

        assert_eq!(
            sfx.active_tones.len(),
            3,
            "ASM loc_7CD0 queues the transform SFX in core traces; fallback must not layer the legacy helmet-id heuristic over it"
        );
    }

    #[test]
    fn fallback_sfx_gives_red_stealth_cues_distinct_audible_shapes_loc_b350_868e_7cd0() {
        let prev = sample_trace(GameMode::InGame);
        let mut curr = sample_trace(GameMode::InGame);
        curr.played_sound_effect_ids = vec![
            gems_sfx::RED_STEALTH_ATTACK_DOWN,
            gems_sfx::RED_STEALTH_SWORD,
            gems_sfx::RED_STEALTH_HITTING_TARGET,
            gems_sfx::RED_STEALTH_TRANSFORM,
        ];
        let mut sfx = SfxEngine::new(AUDIO_SAMPLE_RATE);

        detect_sfx_events(&prev, &curr, &mut sfx);

        let tones: Vec<(u32, u32, u32)> = sfx
            .active_tones
            .iter()
            .map(|tone| {
                (
                    tone.frequency.round() as u32,
                    tone.remaining_samples,
                    (tone.volume * 100.0).round() as u32,
                )
            })
            .collect();

        assert_eq!(
            tones,
            vec![
                (988, 2425, 18),
                (494, 3969, 16),
                (1320, 1543, 17),
                (880, 2425, 14),
                (1568, 1984, 22),
                (784, 4189, 18),
                (330, 3969, 16),
                (494, 4851, 17),
                (740, 6615, 18),
            ],
            "ASM loc_B350/sub_868E/loc_7CD0 queue separate Samurai attack, sword-hit, and transform cues; fallback audio should not collapse them into one weak blip"
        );
    }

    #[test]
    fn fallback_sfx_gives_rubber_block_elastic_shape_loc_10da4() {
        let prev = sample_trace(GameMode::InGame);
        let mut curr = sample_trace(GameMode::InGame);
        curr.blocks_hit_this_frame = 1;
        curr.played_sound_effect_ids = vec![gems_sfx::RUBBER_BLOCK];
        let mut sfx = SfxEngine::new(AUDIO_SAMPLE_RATE);

        detect_sfx_events(&prev, &curr, &mut sfx);

        let tones: Vec<(u32, u32, u32)> = sfx
            .active_tones
            .iter()
            .map(|tone| {
                (
                    tone.frequency.round() as u32,
                    tone.remaining_samples,
                    (tone.volume * 100.0).round() as u32,
                )
            })
            .collect();

        assert_eq!(
            tones,
            vec![(392, 1587, 10), (784, 2557, 12), (523, 3616, 8)],
            "ASM loc_10DA4 queues sfx_Rubber_block=$56; fallback should sound like a quick elastic bounce, not the old low brick-thud pair"
        );
    }

    #[test]
    fn fallback_sfx_gives_jump_on_enemy_distinct_bounce_shape_loc_b6da() {
        let prev = sample_trace(GameMode::InGame);
        let mut curr = sample_trace(GameMode::InGame);
        curr.stomped_this_frame = true;
        curr.played_sound_effect_ids = vec![gems_sfx::JUMP_ON_ENEMY];
        let mut sfx = SfxEngine::new(AUDIO_SAMPLE_RATE);

        detect_sfx_events(&prev, &curr, &mut sfx);

        let tones: Vec<(u32, u32, u32)> = sfx
            .active_tones
            .iter()
            .map(|tone| {
                (
                    tone.frequency.round() as u32,
                    tone.remaining_samples,
                    (tone.volume * 100.0).round() as u32,
                )
            })
            .collect();

        assert_eq!(
            tones,
            vec![(784, 1499, 13), (1175, 2116, 11)],
            "ASM Jump_On_Enemy loc_B6DA queues sfx_Jump_on_enemy=$4C; fallback should be brighter than the ordinary jump cue and should not add the legacy stomp heuristic duplicate"
        );
    }

    #[test]
    fn play_native_keeps_asm_specific_ouch_without_generic_duplicate() {
        let mut prev = sample_trace(GameMode::InGame);
        let mut curr = sample_trace(GameMode::InGame);
        prev.played_sound_effect_ids = vec![gems_sfx::NAVIGATE_JINGLE];
        curr.played_sound_effect_ids = vec![gems_sfx::NAVIGATE_JINGLE, gems_sfx::VOICE_OUCH_2];
        curr.number_hitpoints = prev.number_hitpoints - 1;

        assert_eq!(
            gems_event_sound_ids(&prev, &curr),
            vec![gems_sfx::VOICE_OUCH_2],
            "ASM loc_B73C/loc_B750/loc_B764 selects special VoiceOuch takes for specific enemy families, so the damage heuristic must not add the generic $61 take"
        );
    }

    #[test]
    fn play_native_does_not_infer_ouch_over_fatal_death_voice_issue_363() {
        let mut prev = sample_trace(GameMode::InGame);
        let mut curr = sample_trace(GameMode::InGame);
        prev.number_hitpoints = 1;
        curr.number_hitpoints = 0;
        curr.played_sound_effect_ids = vec![gems_sfx::VOICE_DIE];

        assert_eq!(
            gems_event_sound_ids(&prev, &curr),
            vec![gems_sfx::VOICE_DIE],
            "ASM Death (kid.asm:14714) queues sfx_Voice_die directly; the legacy HP-drop heuristic must not layer loc_B778's generic ouch over a clean fatal hit"
        );
    }

    #[test]
    fn fallback_sfx_does_not_add_ouch_over_fatal_death_voice_issue_363() {
        let mut prev = sample_trace(GameMode::InGame);
        let mut curr = sample_trace(GameMode::InGame);
        prev.number_hitpoints = 1;
        curr.number_hitpoints = 0;
        curr.played_sound_effect_ids = vec![gems_sfx::VOICE_DIE];
        let mut sfx = SfxEngine::new(AUDIO_SAMPLE_RATE);

        detect_sfx_events(&prev, &curr, &mut sfx);

        assert_eq!(
            sfx.active_tones.len(),
            1,
            "fallback audio should play only the core Death voice, not a second HP-drop ouch tone"
        );
    }

    #[test]
    fn play_native_replays_rubber_block_without_generic_block_crunch() {
        let mut prev = sample_trace(GameMode::InGame);
        let mut curr = sample_trace(GameMode::InGame);
        curr.played_sound_effect_ids = vec![gems_sfx::RUBBER_BLOCK];

        assert_eq!(
            gems_event_sound_ids(&prev, &curr),
            vec![gems_sfx::RUBBER_BLOCK],
            "ASM loc_10DA4 queues sfx_Rubber_block=$56; rubber contacts must not fall back to sfx_Block_hit=$0D brick crunch"
        );

        prev.played_sound_effect_ids = Vec::new();
        curr.blocks_hit_this_frame = 1;
        curr.played_sound_effect_ids = vec![gems_sfx::RUBBER_BLOCK];

        assert_eq!(
            gems_event_sound_ids(&prev, &curr),
            vec![gems_sfx::RUBBER_BLOCK],
            "ASM loc_10DA4's explicit rubber cue must suppress any generic block-hit counter visible on the same frame"
        );

        prev.played_sound_effect_ids = vec![gems_sfx::RUBBER_BLOCK];
        curr.blocks_hit_this_frame = 1;
        curr.played_sound_effect_ids = vec![gems_sfx::RUBBER_BLOCK];

        assert_eq!(
            gems_event_sound_ids(&prev, &curr),
            vec![gems_sfx::BLOCK_HIT],
            "the old headbutt counter still drives non-rubber block-hit paths when no new ASM-backed rubber id was queued"
        );
    }

    #[test]
    fn play_native_replays_evanescent_block_from_core_trace_loc_df68() {
        let prev = sample_trace(GameMode::InGame);
        let mut curr = sample_trace(GameMode::InGame);
        curr.played_sound_effect_ids = vec![gems_sfx::EVANESCENT];

        assert_eq!(
            gems_event_sound_ids(&prev, &curr),
            vec![gems_sfx::EVANESCENT],
            "ASM EvanescentBlock_Trigger (kid.asm:17283) queues sfx_Evanescent_block=$14 from the core trace"
        );
    }

    #[test]
    fn play_native_replays_ice_prize_block_without_destroy_delta_duplicate() {
        let prev = sample_trace(GameMode::InGame);
        let mut curr = sample_trace(GameMode::InGame);
        curr.blocks_hit_this_frame = 1;
        curr.played_sound_effect_ids = vec![gems_sfx::DESTROY_BLOCK];
        curr.block_destroy_effects.push((48, 32, 1, 2));

        assert_eq!(
            gems_event_sound_ids(&prev, &curr),
            vec![gems_sfx::DESTROY_BLOCK],
            "ASM IceBlock_SmashAndLoadBullets (kid.asm:22849) queues sfx_Prize_block=$11 from the core trace; the destroy-effect delta must not add a duplicate $11"
        );
    }

    #[test]
    fn play_native_replays_metal_block_without_generic_block_crunch() {
        let prev = sample_trace(GameMode::InGame);
        let mut curr = sample_trace(GameMode::InGame);
        curr.blocks_hit_this_frame = 1;
        curr.played_sound_effect_ids = vec![gems_sfx::METAL_BLOCK];

        assert_eq!(
            gems_event_sound_ids(&prev, &curr),
            vec![gems_sfx::METAL_BLOCK],
            "ASM loc_110D0 queues sfx_Metal_block=$12 for springy block bounces; do not layer sfx_Block_hit=$0D over it"
        );
    }

    #[test]
    fn play_native_replays_elevator_block_from_core_trace_sub_fca6() {
        let prev = sample_trace(GameMode::InGame);
        let mut curr = sample_trace(GameMode::InGame);
        curr.played_sound_effect_ids = vec![gems_sfx::ELEVATOR];

        assert_eq!(
            gems_event_sound_ids(&prev, &curr),
            vec![gems_sfx::ELEVATOR],
            "ASM sub_FCA6 (kid.asm:21021) queues sfx_Elevator_block=$27 from the core trace"
        );
    }

    #[test]
    fn play_native_replays_red_stealth_direct_sfx_ids_from_core_trace() {
        let prev = sample_trace(GameMode::InGame);
        let mut curr = sample_trace(GameMode::InGame);
        curr.played_sound_effect_ids = vec![
            gems_sfx::RED_STEALTH_ATTACK_DOWN,
            gems_sfx::RED_STEALTH_SWORD,
            gems_sfx::RED_STEALTH_HITTING_TARGET,
        ];

        assert_eq!(
            gems_event_sound_ids(&prev, &curr),
            vec![
                gems_sfx::RED_STEALTH_ATTACK_DOWN,
                gems_sfx::RED_STEALTH_SWORD,
                gems_sfx::RED_STEALTH_HITTING_TARGET,
            ],
            "ASM sub_B270 loc_B350 queues attack-down=$13; sub_868E queues sword=$23 and hit-target=$5A, so play-native must replay core trace IDs verbatim"
        );
    }

    #[test]
    fn play_native_transform_sfx_follows_unk_7ed8_helmet_order() {
        let cases = [
            (PlayerForm::Skycutter, gems_sfx::SKYCUTTER_TRANSFORM),
            (PlayerForm::Cyclone, gems_sfx::CYCLONE_TRANSFORM),
            (PlayerForm::RedStealth, gems_sfx::RED_STEALTH_TRANSFORM),
            (PlayerForm::Eyeclops, gems_sfx::EYECLOPS_TRANSFORM),
            (PlayerForm::Juggernaut, gems_sfx::JUGGERNAUT_TRANSFORM),
            (PlayerForm::IronKnight, gems_sfx::IRON_KNIGHT_TRANSFORM),
            (PlayerForm::Berzerker, gems_sfx::BERZERKER_TRANSFORM),
            (PlayerForm::Maniaxe, gems_sfx::MANIAXE_TRANSFORM),
            (PlayerForm::Micromax, gems_sfx::MICROMAX_TRANSFORM),
        ];

        for (form, expected) in cases {
            let mut prev = sample_trace(GameMode::InGame);
            let mut curr = sample_trace(GameMode::InGame);
            prev.current_helmet_id = PlayerForm::TheKid.helmet_id();
            curr.current_helmet_id = form.helmet_id();

            assert_eq!(
                gems_event_sound_ids(&prev, &curr),
                vec![expected],
                "ASM unk_7ED8 indexes transform SFX by Current_Helmet; helmet id {} should queue {expected:#04X}",
                form.helmet_id()
            );
        }
    }

    #[test]
    fn play_native_uses_core_transform_sfx_without_fallback_duplicate_loc_7cd0() {
        let mut prev = sample_trace(GameMode::InGame);
        let mut curr = sample_trace(GameMode::InGame);
        prev.current_helmet_id = PlayerForm::TheKid.helmet_id();
        curr.current_helmet_id = PlayerForm::RedStealth.helmet_id();
        curr.played_sound_effect_ids = vec![gems_sfx::RED_STEALTH_TRANSFORM];

        assert_eq!(
            gems_event_sound_ids(&prev, &curr),
            vec![gems_sfx::RED_STEALTH_TRANSFORM],
            "ASM loc_7CD0 transform SFX now arrives from the core trace; play-native must not add the legacy helmet-id fallback twice"
        );
    }

    #[test]
    fn play_native_demo_transform_skips_legacy_fallback_loc_7cd0() {
        let mut prev = sample_trace(GameMode::DemoPlay);
        let mut curr = sample_trace(GameMode::DemoPlay);
        prev.current_helmet_id = PlayerForm::TheKid.helmet_id();
        curr.current_helmet_id = PlayerForm::Cyclone.helmet_id();

        assert_eq!(
            gems_event_sound_ids(&prev, &curr),
            Vec::<u8>::new(),
            "ASM loc_7CD0 branches on Demo_Mode_flag before unk_7ED8, so legacy helmet-id fallback must stay silent in demo traces"
        );
    }

    #[test]
    fn play_native_replays_voice_bummer_from_core_trace_loc_7e40() {
        let prev = sample_trace(GameMode::InGame);
        let mut curr = sample_trace(GameMode::InGame);
        curr.played_sound_effect_ids = vec![gems_sfx::VOICE_BUMMER];

        assert_eq!(
            gems_event_sound_ids(&prev, &curr),
            vec![gems_sfx::VOICE_BUMMER],
            "ASM loc_7E40 (kid.asm:8570) queues sfx_Voice_bummer=$68 when transform cleanup leaves Current_Helmet zero"
        );
    }

    #[test]
    fn play_native_preserves_transform_back_music_speed_before_voice_bummer_loc_7e26_7e40() {
        let prev = sample_trace(GameMode::InGame);
        let mut curr = sample_trace(GameMode::InGame);
        curr.played_music_speed_updates = vec![0x7E];
        curr.played_sound_commands =
            vec![NativeSoundCommand::start_sequence(gems_sfx::VOICE_BUMMER)];
        curr.played_sound_effect_ids = vec![gems_sfx::VOICE_BUMMER];

        assert_eq!(
            gems_event_audio_commands(&prev, &curr),
            vec![
                NativeAudioTraceCommand::MusicSpeed(0x7E),
                NativeAudioTraceCommand::Sound(NativeSoundCommand::start_sequence(
                    gems_sfx::VOICE_BUMMER
                )),
            ],
            "ASM Kid_Transform loc_7E26 queues the music-speed write before loc_7E40 queues sfx_Voice_bummer; replaying the voice first leaves the sample at the wrong tempo"
        );
    }

    #[test]
    fn play_native_replays_replenish_health_from_core_trace_loc_7c76() {
        let prev = sample_trace(GameMode::InGame);
        let mut curr = sample_trace(GameMode::InGame);
        curr.played_sound_effect_ids = vec![gems_sfx::REPLENISH_HEALTH];

        assert_eq!(
            gems_event_sound_ids(&prev, &curr),
            vec![gems_sfx::REPLENISH_HEALTH],
            "ASM loc_7C76 (kid.asm:8393) queues sfx_Replenish_Health=$2A from the core trace"
        );
    }

    #[test]
    fn play_native_replays_no_time_before_death_voice() {
        let prev = sample_trace(GameMode::InGame);
        let mut curr = sample_trace(GameMode::InGame);
        curr.played_sound_effect_ids = vec![gems_sfx::VOICE_NO_TIME];
        curr.number_lives = prev.number_lives - 1;

        assert_eq!(
            gems_event_sound_ids(&prev, &curr),
            vec![gems_sfx::VOICE_NO_TIME, gems_sfx::VOICE_DIE],
            "ASM Decrease_Time_Left (kid.asm:15500) plays sfx_Voice_no_time=$51 before Character_CheckCollision death voice $60"
        );
    }

    #[test]
    fn play_native_replays_kid_pullup_from_core_trace_loc_abf0() {
        let prev = sample_trace(GameMode::InGame);
        let mut curr = sample_trace(GameMode::InGame);
        curr.played_sound_effect_ids = vec![gems_sfx::KID_PULLUP];

        assert_eq!(
            gems_event_sound_ids(&prev, &curr),
            vec![gems_sfx::KID_PULLUP],
            "ASM kid_flip loc_ABF0 (kid.asm:13464) queues sfx_The_Kid_pullup=$64 from the core trace"
        );
    }

    #[test]
    fn play_native_delays_eol_score_counter_sfx_until_loc_d770() {
        let mut prev = sample_trace(GameMode::InGame);
        let mut scoreboard_setup = sample_trace(GameMode::InGame);
        prev.level_complete = true;
        scoreboard_setup.level_complete = true;
        scoreboard_setup.level_complete_score_rows_started = 1;
        scoreboard_setup.level_complete_score_rows_elapsed_frames = 1;

        assert!(
            !gems_event_sound_ids(&prev, &scoreboard_setup).contains(&gems_sfx::SCORE_COUNTER),
            "ASM loc_D702/loc_D734/loc_D752 delays `sfx_Score_counter` until after the EOL rows and Score_Board wait"
        );

        let mut counter_started = scoreboard_setup.clone();
        counter_started.level_complete_score_counter_elapsed_frames = 1;

        assert_eq!(
            gems_event_sound_ids(&scoreboard_setup, &counter_started),
            vec![gems_sfx::SCORE_COUNTER],
            "ASM loc_D752 queues `sfx_Score_counter` immediately before loc_D770 starts draining the score rows"
        );
        assert_eq!(
            gems_event_sound_commands(&scoreboard_setup, &counter_started),
            vec![NativeSoundCommand::start_sequence(gems_sfx::SCORE_COUNTER)],
            "ASM loc_D752 reaches `j_PlaySound`, whose GEMS command byte is `$10`"
        );
    }

    #[test]
    fn play_native_stops_eol_score_counter_at_loc_d7f4() {
        // ASM loc_D7F4 (kid.asm:16906-16908): once the score-counter drain
        // completes, the ROM issues GEMS `$12` (cmdstopseq, the `j_PlaySound2`
        // call) to *stop* sequence `$0C` (`sfx_Score_counter`) before
        // hibernating and routing to Teleport — cleanly silencing the tick once
        // the tally lands (#216 "level-complete sound does not finish cleanly").
        let mut draining = sample_trace(GameMode::InGame);
        draining.level_complete = true;
        draining.level_complete_score_counter_elapsed_frames = 10;
        draining.level_complete_score_counter_drained = false;

        let mut drained = draining.clone();
        drained.level_complete_score_counter_elapsed_frames = 11;
        drained.level_complete_score_counter_drained = true;

        assert_eq!(
            gems_event_sound_ids(&draining, &drained),
            vec![gems_sfx::SCORE_COUNTER],
            "ASM loc_D7F4 targets sfx_Score_counter when the EOL counter finishes draining"
        );
        assert_eq!(
            gems_event_sound_commands(&draining, &drained),
            vec![NativeSoundCommand::stop_sequence(gems_sfx::SCORE_COUNTER)],
            "ASM loc_D7F4 reaches `j_PlaySound2`, GEMS command byte `$12` = cmdstopseq (stop sequence N)"
        );

        // The stop must fire exactly once: the next frame is still drained.
        let mut still_drained = drained.clone();
        still_drained.level_complete_score_counter_elapsed_frames = 12;
        assert!(
            !gems_event_sound_ids(&drained, &still_drained).contains(&gems_sfx::SCORE_COUNTER),
            "the loc_D7F4 stop is a single edge-triggered cue, not a per-frame repeat"
        );
    }

    #[test]
    fn scripted_recording_audio_events_label_loc_d7f4_score_counter_as_stopsequence()
    -> anyhow::Result<()> {
        let fixture = ManifestFixture::new()?;
        let mut draining = sample_trace(GameMode::InGame);
        draining.level_complete = true;
        draining.level_complete_score_counter_elapsed_frames = 10;

        let mut drained = draining.clone();
        drained.level_complete_score_counter_elapsed_frames = 11;
        drained.level_complete_score_counter_drained = true;

        let events = native_audio_events_for_trace(&fixture.manifest, &[draining, drained]);

        assert!(
            events.iter().any(|event| event.contains(&format!(
                "command=StopSequence({})",
                gems_sfx::SCORE_COUNTER
            ))),
            "recording audio events should preserve ASM loc_D7F4's j_PlaySound2 command byte `$12`, not just the reused score-counter id"
        );
        Ok(())
    }

    #[test]
    fn play_native_emits_diamond_prize_sfx_from_contact_frame_not_count_increase() {
        // ASM PrizeDiamondCollected_Init (kid.asm:21103) plays
        // `sfx_Diamond_prize` ($2D) on the contact frame; Number_Diamonds only
        // rises ~33 frames later after the fly-to-counter loop. The core now
        // queues the cue at contact, so the native player must replay it from
        // the trace and must NOT add a second, late cue when the visible count
        // finally increments (#216 diamond delay).
        let mut contact = sample_trace(GameMode::InGame);
        let mut prev = sample_trace(GameMode::InGame);
        contact.played_sound_effect_ids = vec![gems_sfx::DIAMOND_PRIZE];
        assert_eq!(
            gems_event_sound_ids(&prev, &contact),
            vec![gems_sfx::DIAMOND_PRIZE],
            "ASM PrizeDiamondCollected_Init plays sfx_Diamond_prize on the diamond-prize contact frame"
        );

        // 33 frames later the visible count rises, but the SFX already played at
        // contact; no late duplicate cue should be emitted.
        prev.diamond_count = 0;
        let mut count_increment = sample_trace(GameMode::InGame);
        count_increment.diamond_count = 1;
        assert!(
            gems_event_sound_ids(&prev, &count_increment).is_empty(),
            "the diamond cue is anchored to the ASM contact frame, not the delayed Number_Diamonds increment"
        );
    }

    #[test]
    fn play_native_replays_clock_prize_without_timer_jump_duplicate() {
        let prev = sample_trace(GameMode::InGame);
        let mut curr = sample_trace(GameMode::InGame);
        curr.played_sound_effect_ids = vec![gems_sfx::CLOCK_PRIZE];
        curr.level_timer_seconds = prev.level_timer_seconds + 180;

        assert_eq!(
            gems_event_sound_ids(&prev, &curr),
            vec![gems_sfx::CLOCK_PRIZE],
            "#342: the clock-prize loop is driven by the core trace (loc_10792 spawn start / loc_107E0 stop); the removed timer-jump heuristic must not re-add a start at collection"
        );
    }

    #[test]
    fn scripted_recording_audio_events_emit_eol_score_counter_at_loc_d770() -> anyhow::Result<()> {
        let fixture = ManifestFixture::new()?;
        let mut eol_start = sample_trace(GameMode::InGame);
        eol_start.level_complete = true;

        let mut scoreboard_setup = eol_start.clone();
        scoreboard_setup.level_complete_score_rows_started = 1;
        scoreboard_setup.level_complete_score_rows_elapsed_frames = 1;

        let mut counter_started = scoreboard_setup.clone();
        counter_started.level_complete_score_counter_elapsed_frames = 1;

        let events = native_audio_events_for_trace(
            &fixture.manifest,
            &[eol_start, scoreboard_setup, counter_started],
        );

        assert_eq!(
            events,
            vec![
                format!(
                    "frame=0 mode=InGame level=00 active_before=None active_after=Some(ScoreBoard) command=Start({})",
                    gems_bgm::SCORE_BOARD
                ),
                format!(
                    "frame=2 mode=InGame level=00 command=StartSequence({})",
                    gems_sfx::SCORE_COUNTER
                ),
            ],
            "recording audio events should expose ASM Score_Board BGM and the delayed loc_D752/loc_D770 `sfx_Score_counter` cue without playing it during row setup"
        );
        Ok(())
    }

    #[test]
    fn play_native_bgm_transition_does_not_restart_same_context() -> anyhow::Result<()> {
        let fixture = ManifestFixture::new()?;
        let trace = sample_trace(GameMode::InGame);

        assert_eq!(
            native_bgm_state_after_trace(
                &fixture.manifest,
                &trace,
                Some(NativeBgmCue::Level(LevelId(0)))
            ),
            (Some(NativeBgmCue::Level(LevelId(0))), None),
            "steady gameplay should not spam stop/start commands every frame"
        );
        Ok(())
    }

    #[test]
    fn level_complete_scoreboard_bgm_uses_score_board_cue_loc_c21e() -> anyhow::Result<()> {
        let fixture = ManifestFixture::new()?;
        let mut trace = sample_trace(GameMode::InGame);
        trace.level_complete = true;

        assert_eq!(
            native_bgm_cue_for_trace(&trace)
                .and_then(|cue| bgm_for_native_cue(&fixture.manifest, cue)),
            Some(gems_bgm::SCORE_BOARD),
            "ASM Score_Board should queue bgm_Score_Board for the EOL sequence"
        );
        Ok(())
    }

    #[test]
    fn ending_bgm_uses_core_trace_cue_loc_d824() -> anyhow::Result<()> {
        let fixture = ManifestFixture::new()?;
        let mut trace = sample_trace(GameMode::EndingSequence);
        trace.played_bgm_ids = vec![gems_bgm::ENDING];

        assert_eq!(
            native_bgm_cue_for_trace(&trace)
                .and_then(|cue| bgm_for_native_cue(&fixture.manifest, cue)),
            Some(gems_bgm::ENDING),
            "ASM End_Credits loc_D824 should drive bgm_Ending from the core trace"
        );
        Ok(())
    }

    #[test]
    fn play_native_theme_bgm_ids_follow_asm_loc_12004() {
        assert_eq!(
            (0..=10).map(theme_to_bgm).collect::<Vec<_>>(),
            vec![
                Some(gems_bgm::MOUNTAIN),
                Some(gems_bgm::SKY),
                Some(gems_bgm::ICE),
                Some(gems_bgm::HILLS),
                Some(gems_bgm::ISLAND),
                Some(gems_bgm::DESERT),
                Some(gems_bgm::SWAMP),
                Some(gems_bgm::MOUNTAIN),
                Some(gems_bgm::CAVE),
                Some(gems_bgm::FOREST),
                Some(gems_bgm::CITY),
            ],
            "ASM loc_12004 / unk_1201E queues the constants.asm BGM bytes by Foreground_theme; native play must not use display theme names as the table order",
        );
    }
}
