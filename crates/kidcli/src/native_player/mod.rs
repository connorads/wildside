use crate::recording::{
    RecordingBundleArtifacts, build_recording, recording_start_from_options,
    save_recording_with_artifacts,
};
use crate::scenarios::{DynamicScenario, DynamicStartPosition, GameplayCheckpoint};
use anyhow::{Result, anyhow, bail};
use font8x8::{BASIC_FONTS, UnicodeFonts};
use kid_core::{
    ActiveProjectileKind, ActiveProjectileSnapshot, DiamondPowerSpriteSnapshot, FrameInput,
    GameMode, LevelId, NativeObjectKind, NativeObjectSnapshot, NativeRuntime, NativeTraceFrame,
    PlayerForm, physics::Direction, terrain::BlockGrid,
};
use kid_data::{AssetManifest, LevelHeader, resource_level_id, resource_level_record};
use kid_md::GemsAudioEngine;
use kid_render::{MdColour, NativeLevelVisuals, Palette, TileRef, TileSheet, load_level_visuals};
use sdl2::audio::AudioQueue;
use sdl2::controller::{Button, GameController};
use sdl2::event::Event;
use sdl2::keyboard::{Keycode, Scancode};
use sdl2::pixels::PixelFormatEnum;
use sdl2::rect::Rect;
use sdl2::render::WindowCanvas;
use sdl2::video::FullscreenType;
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use std::path::PathBuf;
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};

const FRAME_WIDTH: u32 = 320;
const FRAME_HEIGHT: u32 = 224;
const FRAME_TIME: Duration = Duration::from_micros(16_667);
const LEVEL_CARD_HOLD_FRAMES: usize = 60;
const AUDIO_SAMPLES_PER_FRAME: usize = 735;
const AUDIO_QUEUE_TARGET_FRAMES: usize = 1;
const SDL_AUDIO_CALLBACK_SAMPLES: u16 = 512;
#[cfg(test)]
const PLAYER_COLOUR: [u8; 3] = [92, 214, 104];
const ENEMY_COLOUR: [u8; 3] = [220, 90, 88];
const ENEMY_FLASH_COLOUR: [u8; 3] = [255, 255, 255];

/// Degraded-asset fallback colour for an enemy rect when its ROM sprite
/// art is not loaded. The previous per-family hand-picked palette
/// (orange-red Fire Demon, steel-grey Robot, military-green Tank, etc.)
/// was a hand-invented "visually distinguishable" set with no ROM
/// citation — the original game's per-family colours come from each
/// family's `Pal_*` block (e.g. `Pal_A2382` for Heady Metal). Until
/// those palettes flow through `level_visuals.enemies.families`, render
/// a single consistent placeholder so missing-asset states do not claim
/// per-family hues that the ROM does not specify.
fn enemy_family_colour(_object_type: u8) -> [u8; 3] {
    ENEMY_COLOUR
}
#[cfg(test)]
const PLATFORM_COLOUR: [u8; 3] = [232, 176, 64];
#[cfg(test)]
const GUIDE_COLOUR: [u8; 3] = [62, 84, 108];
const SKY_COLOUR: [u8; 3] = [28, 42, 56];
const PALETTE_LINE_COLOUR_COUNT: usize = 15;

#[derive(Debug, Clone)]
pub struct NativePlayOptions {
    pub fullscreen: bool,
    pub scale: Option<u32>,
    pub debug_overlay: bool,
    pub warp_level: Option<u8>,
    pub settled: bool,
    pub record_to: Option<PathBuf>,
    pub record_frames: Option<usize>,
    pub record_notes: Option<String>,
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
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Scene {
    background: [u8; 3],
    commands: Vec<DrawCommand>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum DrawCommand {
    Rect {
        x: i32,
        y: i32,
        width: i32,
        height: i32,
        colour: [u8; 3],
    },
    ClearPriorityRect {
        x: i32,
        y: i32,
        width: i32,
        height: i32,
    },
    Text {
        x: i32,
        y: i32,
        text: String,
        colour: [u8; 3],
        scale: i32,
    },
    #[allow(dead_code)]
    IndexedTile {
        x: i32,
        y: i32,
        tile_sheet: Arc<TileSheet>,
        tile: TileRef,
        palette: Arc<Palette>,
    },
    #[allow(dead_code)]
    IndexedTileRow {
        x: i32,
        y: i32,
        tile_sheet: Arc<TileSheet>,
        tile: TileRef,
        palette: Arc<Palette>,
        row: u8,
    },
    #[allow(dead_code)]
    IndexedTileSlice {
        x: i32,
        y: i32,
        tile_sheet: Arc<TileSheet>,
        tile: TileRef,
        palette: Arc<Palette>,
        row: u8,
        start_column: u8,
        width: u8,
    },
    #[allow(dead_code)]
    Sprite {
        x: i32,
        y: i32,
        tile_sheet: Arc<TileSheet>,
        palette: Arc<Palette>,
        tiles: Vec<SpriteTile>,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct SpriteTile {
    offset_x: i32,
    offset_y: i32,
    tile: TileRef,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct NativeSpriteTileDiagnostic {
    pub(crate) offset_x: i32,
    pub(crate) offset_y: i32,
    pub(crate) tile_index: u16,
    pub(crate) palette_line: u8,
    pub(crate) priority: bool,
    pub(crate) flip_x: bool,
    pub(crate) flip_y: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct NativeEolScoreHardwareSpriteDiagnostic {
    pub(crate) command_index: usize,
    pub(crate) chunk_index: usize,
    pub(crate) x: i32,
    pub(crate) y: i32,
    pub(crate) width: u16,
    pub(crate) height: u16,
    pub(crate) first_tile_index: u16,
    pub(crate) min_x: i32,
    pub(crate) min_y: i32,
    pub(crate) max_x: i32,
    pub(crate) max_y: i32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct NativeEolScoreSpriteDiagnostic {
    pub(crate) command_index: usize,
    pub(crate) x: i32,
    pub(crate) y: i32,
    pub(crate) min_x: i32,
    pub(crate) min_y: i32,
    pub(crate) max_x: i32,
    pub(crate) max_y: i32,
    pub(crate) tile_count: usize,
    pub(crate) tiles: Vec<NativeSpriteTileDiagnostic>,
    #[serde(default)]
    pub(crate) hardware_sprites: Vec<NativeEolScoreHardwareSpriteDiagnostic>,
}

#[derive(Debug, Clone)]
struct NativeSessionAssets {
    level_visuals: NativeLevelVisuals,
    block_grid: BlockGrid,
}

#[derive(Debug, Clone)]
struct ResolvedSceneTile {
    tile_sheet: Arc<TileSheet>,
    tile: TileRef,
    palette: Arc<Palette>,
}

#[derive(Debug, Clone, Copy, Deserialize)]
struct GeneratedLevelLayoutMetadata {
    data_bytes: usize,
    width_blocks: u16,
    height_blocks: u16,
}

const FOREGROUND_TILE_VRAM_BASE: u16 = 0x00BC;
const BLOCK_TILE_VRAM_BASE: u16 = 0x0220;
const HUD_TILE_VRAM_BASE: u16 = 0x06BA;
const HUD_DIAMOND_TILE_VRAM_BASE: u16 = 0x06EA;

mod hud;
mod render;
mod scene;
mod sprites;

use hud::*;
use render::*;
use scene::*;
use sprites::*;

pub(crate) use render::sample_scene_pixel_provenance;
pub(crate) use render::{render_frame_on_blank_canvas, render_scene, render_scene_overlay};
pub(crate) use scene::{
    BackgroundPixelSample, LayerFilter, build_scene, build_scene_filtered,
    eol_score_sprite_diagnostics, render_trace_with_vdp_latched_state,
    sample_background_pixel_loc_c070,
};
pub(crate) use sprites::render_dynamic_gameplay_frame_evidence;
pub(crate) use sprites::render_gameplay_frame_evidence;
pub(crate) use sprites::render_gameplay_frame_evidence_for_level;

pub(crate) const NATIVE_FRAME_WIDTH: u32 = FRAME_WIDTH;
pub(crate) const NATIVE_FRAME_HEIGHT: u32 = FRAME_HEIGHT;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct FinalScenePixelDiagnostic {
    pub(crate) x: usize,
    pub(crate) y: usize,
    pub(crate) final_rgb: [u8; 3],
    pub(crate) final_high_priority: bool,
    pub(crate) writes: Vec<FinalScenePixelWrite>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct FinalScenePixelWrite {
    pub(crate) command_index: usize,
    pub(crate) kind: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) origin_x: Option<i32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) origin_y: Option<i32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) width: Option<i32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) height: Option<i32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) tile_index: Option<u16>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) palette_line: Option<u8>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) palette_index: Option<u8>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) source_x: Option<u8>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) source_y: Option<u8>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) rgb: Option<[u8; 3]>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) priority: Option<bool>,
    pub(crate) prior_high_priority: bool,
    pub(crate) accepted: bool,
}

pub(crate) fn load_session_assets(
    manifest: &AssetManifest,
    level_id: u8,
) -> Result<(NativeLevelVisuals, BlockGrid)> {
    let assets = load_native_session_assets(manifest, level_id)?;
    Ok((assets.level_visuals, assets.block_grid))
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
                freq: Some(44_100),
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
    if let Some(ref aq) = audio_queue {
        aq.resume();
    }
    match (&audio_queue, &gems_engine) {
        (Some(_), Some(_)) => {
            eprintln!("Audio: GEMS engine active (stereo, 44100 Hz)");
        }
        (Some(_), None) => eprintln!("Audio: SFX fallback (GEMS data unavailable)"),
        (None, _) => eprintln!("Audio: disabled — no sound will play"),
    }
    let mut sfx_engine = SfxEngine::new();

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
    let mut recorded_audio = Vec::new();
    let mut recorded_audio_events = Vec::new();
    let mut audio_diagnostics =
        NativeAudioDiagnostics::new(has_gems, audio_queue.is_some(), audio_channels);
    let mut final_frame_rgb = options.record_to.as_ref().map(|_| frame.clone());

    if options.record_to.is_some() {
        eprintln!("Recording enabled: press F10 to save the current bundle and exit.");
    }

    if let Some(vdp) = runtime.vdp_state() {
        let snapshot = kid_md::VdpSnapshot {
            registers: vdp.registers,
            vram: vdp.vram.clone(),
            cram: vdp.cram.clone(),
            vsram: vdp.vsram.clone(),
        };
        kid_md::render_vdp_into(&snapshot, &mut frame);
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
            if !native_level_card_hold_active(
                level_card_frames_remaining,
                debug_level_card_fallback_frames_remaining,
            ) {
                let live_buttons =
                    keyboard_buttons(&events) | controller_buttons(controller.as_ref());
                let live_buttons = live_buttons_after_debug_release(
                    live_buttons,
                    &mut wait_for_live_input_release,
                );
                let buttons = native_frame_buttons(
                    options
                        .scripted_route
                        .as_ref()
                        .map(|route| (route.script.as_slice(), route.delay_recording_input)),
                    recorded_trace.len(),
                    live_buttons,
                );
                trace = runtime.step_frame(FrameInput::from_mask(buttons));
                if options.record_to.is_some() {
                    recorded_script.push(buttons);
                    recorded_trace.push(trace.clone());
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
                    if resolved_level.0 != new_level {
                        trace = runtime.enter_level_playtest(resolved_level);
                    }
                    session_assets = new_assets;
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
        if let Some(command) = bgm_command
            && let Some(ref mut gems) = gems_engine
        {
            pending_bgm_start = apply_native_bgm_command(gems, command);
            queued_bgm_start_this_frame = pending_bgm_start.is_some();
            if native_bgm_command_flushes_queued_audio(command)
                && let Some(ref aq) = audio_queue
            {
                aq.clear();
            }
        }
        active_bgm_cue = next_bgm_cue;

        let should_capture_audio = options.record_to.is_some();
        let should_step_audio = should_capture_audio || audio_queue.is_some();
        if should_step_audio {
            let maybe_samples = if let Some(ref mut gems) = gems_engine {
                if !queued_bgm_start_this_frame && let Some(bgm) = pending_bgm_start.take() {
                    gems.play_sound(bgm);
                }
                detect_gems_events(&prev_trace, &trace, gems);
                gems.step_frame();
                Some(gems.generate_samples(AUDIO_SAMPLES_PER_FRAME))
            } else {
                Some(sfx_engine.generate(AUDIO_SAMPLES_PER_FRAME))
            };
            if let Some(samples) = maybe_samples {
                audio_diagnostics.record_generated_frame(samples.len());
                if should_capture_audio {
                    recorded_audio.extend_from_slice(&samples);
                }
                if let Some(ref aq) = audio_queue {
                    let queued_bytes = aq.size();
                    if should_queue_audio_frame(queued_bytes, samples.len()) {
                        match aq.queue_audio(&samples) {
                            Ok(()) => {
                                audio_diagnostics.record_queued_frame(samples.len(), queued_bytes);
                            }
                            Err(_) => {
                                audio_diagnostics.record_queue_error(queued_bytes);
                            }
                        }
                    } else {
                        audio_diagnostics.record_skipped_full_frame(queued_bytes);
                    }
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
            let snapshot = kid_md::VdpSnapshot {
                registers: vdp.registers,
                vram: vdp.vram.clone(),
                cram: vdp.cram.clone(),
                vsram: vdp.vsram.clone(),
            };
            kid_md::render_vdp_into(&snapshot, &mut frame);
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
        audio_channels,
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

#[cfg(test)]
fn debug_level_card_fallback_frames(manifest: &AssetManifest, level_id: LevelId) -> usize {
    if level_has_generated_title_card(manifest, level_id) {
        0
    } else {
        LEVEL_CARD_HOLD_FRAMES
    }
}

fn debug_hotkey_level_card_hold_frames() -> usize {
    // `N`/`P` are native-only debug hotkeys, not ROM input. The core still
    // enters ASM `loc_6E2`/`Load_TitleCard`; play-native holds that generated
    // shell, or a deterministic fallback for targets without title-card data,
    // so the hotkey never appears to skip straight into gameplay.
    LEVEL_CARD_HOLD_FRAMES
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
    let has_renderable_title_card =
        level_has_generated_title_card(manifest, level_id) && runtime.vdp_state().is_some();

    debug_hotkey_level_presentation(has_renderable_title_card)
}

fn debug_hotkey_level_presentation(
    has_renderable_title_card: bool,
) -> DebugHotkeyLevelPresentation {
    let hold_frames = debug_hotkey_level_card_hold_frames();
    DebugHotkeyLevelPresentation {
        level_card_frames_remaining: if has_renderable_title_card {
            hold_frames
        } else {
            0
        },
        debug_level_card_fallback_frames_remaining: if has_renderable_title_card {
            0
        } else {
            hold_frames
        },
        fade_alpha: 0,
        fading_in: false,
        suppress_level_card_for_debug_warp: false,
        wait_for_live_input_release: true,
    }
}

fn level_has_generated_title_card(manifest: &AssetManifest, level_id: LevelId) -> bool {
    manifest
        .levels
        .iter()
        .find(|level| level.level_id == level_id.0)
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
    audio_channels: u8,
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
    if let Some(route) = &options.scripted_route {
        recording.manifest.scenario.start_position = route.start_position;
        recording.manifest.scenario.start_velocity = route.start_velocity;
        recording.manifest.scenario.camera_position = route.camera_position;
        recording.manifest.scenario.playtest_form = route.playtest_form;
        recording.manifest.scenario.initial_diamond_count = route.initial_diamond_count;
    }
    let artifacts = RecordingBundleArtifacts {
        user_notes: options.record_notes.clone(),
        final_frame_rgb,
        audio_samples: Some(recorded_audio),
        audio_channels: u16::from(audio_channels),
        audio_events: recorded_audio_events,
        audio_diagnostics,
    };
    save_recording_with_artifacts(record_to, &recording, &artifacts)?;
    println!(
        "recording saved to {}. verify with: cargo run -p kidcli -- verify-recording --slug {}",
        record_to.display(),
        slug
    );
    Ok(())
}

fn load_native_session_assets(
    manifest: &AssetManifest,
    level_id: u8,
) -> Result<NativeSessionAssets> {
    load_native_session_assets_with(manifest, level_id, load_level_visuals).map_err(|error| {
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
    let mut runtime = if options.settled && options.warp_level.is_some() {
        NativeRuntime::with_generated_level_layout(manifest.clone())
    } else {
        let mut runtime = NativeRuntime::new(manifest.clone());
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

fn game_mode_label(mode: GameMode) -> &'static str {
    match mode {
        GameMode::SegaScreen => "SegaScreen",
        GameMode::IntroSequence1 => "IntroSequence1",
        GameMode::IntroSequence2 => "IntroSequence2",
        GameMode::IntroSequence3 => "IntroSequence3",
        GameMode::IntroSequence4 => "IntroSequence4",
        GameMode::IntroSequence5 => "IntroSequence5",
        GameMode::IntroSequence6 => "IntroSequence6",
        GameMode::TitleScreen => "TitleScreen",
        GameMode::LevelTransition => "LevelTransition",
        GameMode::InGame => "InGame",
        GameMode::DemoPlay => "DemoPlay",
        GameMode::OptionsMenu => "OptionsMenu",
        GameMode::EndingSequence => "EndingSequence",
    }
}

fn form_label(helmet_id: u16) -> &'static str {
    match PlayerForm::from_helmet_id(helmet_id) {
        Some(PlayerForm::TheKid) => "TheKid",
        Some(PlayerForm::Skycutter) => "Skycutter",
        Some(PlayerForm::Cyclone) => "Cyclone",
        Some(PlayerForm::RedStealth) => "RedStealth",
        Some(PlayerForm::Eyeclops) => "Eyeclops",
        Some(PlayerForm::Juggernaut) => "Juggernaut",
        Some(PlayerForm::IronKnight) => "IronKnight",
        Some(PlayerForm::Berzerker) => "Berzerker",
        Some(PlayerForm::Maniaxe) => "Maniaxe",
        Some(PlayerForm::Micromax) => "Micromax",
        None => "Unknown",
    }
}

fn on_off(value: bool) -> &'static str {
    if value { "On" } else { "Off" }
}

// ---------------------------------------------------------------------------
// Simple SFX synthesiser: generates square-wave tones for game events.
// ---------------------------------------------------------------------------

const SFX_SAMPLE_RATE: f32 = 44_100.0;

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

fn should_queue_audio_frame(queued_bytes: u32, generated_sample_count: usize) -> bool {
    let queued_bytes = usize::try_from(queued_bytes).unwrap_or(usize::MAX);
    // SDL_GetQueuedAudioSize reports bytes, while `generate_samples` returns
    // interleaved f32 samples. Keep only a small live queue so gameplay SFX
    // detected from the current trace frame are not heard multiple frames late.
    let target_bytes = generated_sample_count
        .saturating_mul(std::mem::size_of::<f32>())
        .saturating_mul(AUDIO_QUEUE_TARGET_FRAMES);
    queued_bytes < target_bytes
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct NativeAudioDiagnostics {
    gems_loaded: bool,
    sdl_queue_present: bool,
    audio_channels: u8,
    generated_frames: usize,
    generated_samples: usize,
    queued_frames: usize,
    queued_samples: usize,
    skipped_full_frames: usize,
    queue_errors: usize,
    queue_observations: usize,
    min_queue_bytes_before_queue: Option<u32>,
    max_queue_bytes_before_queue: Option<u32>,
    frame_timing_observations: usize,
    max_frame_micros: u128,
    over_budget_frames: usize,
    total_over_budget_micros: u128,
}

impl NativeAudioDiagnostics {
    fn new(gems_loaded: bool, sdl_queue_present: bool, audio_channels: u8) -> Self {
        Self {
            gems_loaded,
            sdl_queue_present,
            audio_channels,
            generated_frames: 0,
            generated_samples: 0,
            queued_frames: 0,
            queued_samples: 0,
            skipped_full_frames: 0,
            queue_errors: 0,
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

    fn record_skipped_full_frame(&mut self, queued_bytes: u32) {
        self.record_queue_observation(queued_bytes);
        self.skipped_full_frames += 1;
    }

    fn record_queue_error(&mut self, queued_bytes: u32) {
        self.record_queue_observation(queued_bytes);
        self.queue_errors += 1;
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
            format!("generated_frames={}", self.generated_frames),
            format!("generated_samples={}", self.generated_samples),
            format!("queued_frames={}", self.queued_frames),
            format!("queued_samples={}", self.queued_samples),
            format!("skipped_full_frames={}", self.skipped_full_frames),
            format!("queue_errors={}", self.queue_errors),
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
    pub const ELEVATOR: u8 = 0x27;
    pub const DIAMOND_POWER_AVAILABLE: u8 = 0x28;
    pub const ANKH_PRIZE: u8 = 0x29;
    pub const REPLENISH_HEALTH: u8 = 0x2A;
    pub const COIN_PRIZE: u8 = 0x2C;
    pub const DIAMOND_PRIZE: u8 = 0x2D;
    pub const NAVIGATE_JINGLE: u8 = 0x2D;
    pub const MINI_HOPPING_SKULL_SCREAM: u8 = 0x3F;
    pub const JUMP_ON_ENEMY: u8 = 0x4C;
    pub const VOICE_NO_TIME: u8 = 0x51;
    pub const RUBBER_BLOCK: u8 = 0x56;
    pub const VOICE_DIE: u8 = 0x60;
    pub const VOICE_OUCH: u8 = 0x61;
    pub const VOICE_DIAMOND: u8 = 0x62;
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
    ScoreBoard,
    StoppedForTransition,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum NativeBgmPlaybackCommand {
    Start(u8),
    StopMusic,
    StopMusicThenStart(u8),
}

fn native_bgm_cue_for_trace(trace: &NativeTraceFrame) -> Option<NativeBgmCue> {
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
            Some(NativeBgmCue::TitleShell | NativeBgmCue::Level(_) | NativeBgmCue::ScoreBoard)
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
    let command = if active_bgm_cue
        .is_some_and(|active| active != NativeBgmCue::StoppedForTransition)
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
            for sound_id in gems_event_sound_ids(prev, trace) {
                events.push(format_native_sound_event(frame_index, trace, sound_id));
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
    let initial_active_bgm_cue = trace_frames
        .first()
        .and_then(|trace| native_bgm_cue_for_trace(trace));
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

    if let Some(bgm) = settled_audio_initial_bgm_id(manifest, initial_active_bgm_cue) {
        // Settled dynamic audio probes start after the ROM has already loaded
        // level music. Suppress duplicate event-log commands, but prime the
        // standalone GEMS renderer so block/pickup SFX are mixed over the same
        // active BGM instead of silence until the first SFX.
        gems.play_sound(bgm);
    }

    let mut active_bgm_cue = initial_active_bgm_cue;
    let mut pending_bgm_start: Option<u8> = None;
    let mut prev_trace: Option<&NativeTraceFrame> = None;
    let mut audio_samples = Vec::with_capacity(trace_frames.len() * AUDIO_SAMPLES_PER_FRAME * 2);
    for trace in trace_frames {
        let (next_bgm_cue, bgm_command) =
            native_bgm_state_after_trace(manifest, trace, active_bgm_cue);
        let mut queued_bgm_start_this_frame = false;
        if let Some(command) = bgm_command {
            pending_bgm_start = apply_native_bgm_command(&mut gems, command);
            queued_bgm_start_this_frame = pending_bgm_start.is_some();
        }
        active_bgm_cue = next_bgm_cue;

        if !queued_bgm_start_this_frame && let Some(bgm) = pending_bgm_start.take() {
            gems.play_sound(bgm);
        }
        if let Some(prev) = prev_trace {
            for sound_id in gems_event_sound_ids(prev, trace) {
                gems.play_sound(sound_id);
            }
        }
        gems.step_frame();
        audio_samples.extend(gems.generate_samples(AUDIO_SAMPLES_PER_FRAME));
        prev_trace = Some(trace);
    }

    RecordingBundleArtifacts {
        audio_samples: Some(audio_samples),
        audio_channels: 2,
        audio_events,
        ..RecordingBundleArtifacts::default()
    }
}

fn settled_audio_initial_bgm_id(
    manifest: &AssetManifest,
    initial_active_bgm_cue: Option<NativeBgmCue>,
) -> Option<u8> {
    initial_active_bgm_cue.and_then(|cue| bgm_for_native_cue(manifest, cue))
}

fn apply_native_bgm_command(
    gems: &mut GemsAudioEngine,
    command: NativeBgmPlaybackCommand,
) -> Option<u8> {
    match command {
        NativeBgmPlaybackCommand::Start(bgm) => {
            gems.play_sound(bgm);
            None
        }
        NativeBgmPlaybackCommand::StopMusic => {
            gems.stop_music();
            None
        }
        NativeBgmPlaybackCommand::StopMusicThenStart(bgm) => {
            // ASM StopMusic (kid.asm:49727) reloads/resets the GEMS driver
            // and queues command `$0B` with the sound data pointers. Native
            // playback must use that ROM path before changing music contexts;
            // `$16` stop-all leaves the standalone driver with stale title
            // sequence state and can starve gameplay SFX channels.
            gems.stop_music();
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

fn format_native_sound_event(frame_index: usize, trace: &NativeTraceFrame, sound_id: u8) -> String {
    format!(
        "frame={frame_index} mode={:?} level={:02X} command=PlaySound({sound_id})",
        trace.game_mode, trace.current_level.0
    )
}

fn bgm_for_native_cue(manifest: &AssetManifest, cue: NativeBgmCue) -> Option<u8> {
    match cue {
        // ASM Load_TitleScreen queues bgm_Ice through j_PlaySound before loading title art.
        NativeBgmCue::TitleShell => Some(gems_bgm::ICE),
        NativeBgmCue::Options => Some(gems_bgm::CITY),
        NativeBgmCue::Level(level) => resolved_level_header(manifest, level.0)
            .and_then(|header| theme_to_bgm(header.level_theme)),
        NativeBgmCue::ScoreBoard => Some(gems_bgm::SCORE_BOARD),
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

fn is_ouch_sound_id(sound_id: u8) -> bool {
    matches!(
        sound_id,
        gems_sfx::VOICE_OUCH
            | gems_sfx::VOICE_OUCH_1
            | gems_sfx::VOICE_OUCH_2
            | gems_sfx::VOICE_OUCH_3
    )
}

fn is_explicit_block_sound_id(sound_id: u8) -> bool {
    matches!(sound_id, gems_sfx::METAL_BLOCK | gems_sfx::RUBBER_BLOCK)
}

fn gems_event_sound_ids(prev: &NativeTraceFrame, curr: &NativeTraceFrame) -> Vec<u8> {
    let queued_sound_ids = queued_gems_sound_ids_since(prev, curr);
    let mut sound_ids = queued_sound_ids.to_vec();
    let gameplay_sfx_fallback_active =
        matches!(curr.game_mode, GameMode::InGame | GameMode::DemoPlay);

    // Sound test: play the selected track when A/C pressed
    if curr.sound_test_play_pending
        && let Some(track) = curr.sound_test_track
    {
        sound_ids.push(track);
    }

    // Jump
    if gameplay_sfx_fallback_active
        && !queued_sound_ids.contains(&gems_sfx::JUMP)
        && prev.kid_on_ground == Some(true)
        && curr.kid_on_ground == Some(false)
        && curr.kid_y_velocity.is_some_and(|v| v < 0)
    {
        sound_ids.push(gems_sfx::JUMP);
    }

    // Stomp enemy (score jumped by 1000+, not from diamond collection)
    if curr.score >= prev.score + 1000 && curr.diamond_count == prev.diamond_count {
        sound_ids.push(gems_sfx::JUMP_ON_ENEMY);
    }

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
    if !queued_sound_ids.iter().copied().any(is_ouch_sound_id)
        && curr.number_hitpoints < prev.number_hitpoints
        && prev.number_hitpoints > 0
    {
        sound_ids.push(gems_sfx::VOICE_OUCH);
    }

    // Death (lives decreased)
    if curr.number_lives < prev.number_lives {
        sound_ids.push(gems_sfx::VOICE_DIE);
    }

    // Form change (helmet ID changed). ASM unk_7ED8 indexes this table by
    // Current_Helmet, whose order differs from the extracted art order.
    if curr.current_helmet_id != prev.current_helmet_id {
        let sfx = match curr.current_helmet_id {
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
        };
        if let Some(id) = sfx {
            sound_ids.push(id);
        }
    }

    // Diamond Power activation
    if curr.diamond_power_active && !prev.diamond_power_active {
        sound_ids.push(gems_sfx::DIAMOND_POWER_AVAILABLE);
    }

    // ASM loc_D752/loc_D770: Score_Board BGM and EOL rows appear before the
    // `sfx_Score_counter` cue; only start it when the ROM counter drain begins.
    if curr.level_complete
        && curr.level_complete_score_counter_elapsed_frames > 0
        && prev.level_complete_score_counter_elapsed_frames == 0
    {
        sound_ids.push(gems_sfx::SCORE_COUNTER);
    }

    // ASM loc_D7F4 (kid.asm:16906-16908): when the counter finishes draining,
    // the ROM replays `sfx_Score_counter` ($0C) a *second* time via
    // `j_PlaySound2` before hibernating and routing to Teleport/End_Credits.
    // Native previously only emitted the drain-start cue, so the level-complete
    // tally ended without the closing chime (#216 "level-complete sound does
    // not finish cleanly").
    if curr.level_complete
        && curr.level_complete_score_counter_drained
        && !prev.level_complete_score_counter_drained
    {
        sound_ids.push(gems_sfx::SCORE_COUNTER);
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
        sound_ids.push(gems_sfx::BLOCK_HIT);
    }

    // Block destroyed (new destroy effects appeared)
    if curr.block_destroy_effects.len() > prev.block_destroy_effects.len() {
        sound_ids.push(gems_sfx::DESTROY_BLOCK);
    }

    // Lava geyser erupted (new geysers appeared)
    if curr.active_geysers.len() > prev.active_geysers.len() {
        sound_ids.push(gems_sfx::LAVA_GEYSER);
    }

    // Game complete — ending music
    if curr.game_complete && !prev.game_complete {
        sound_ids.push(gems_bgm::ENDING);
    }

    // Clock pickup (timer jumped up)
    if curr.level_timer_seconds > prev.level_timer_seconds + 5 {
        sound_ids.push(gems_sfx::CLOCK_PRIZE);
    }

    sound_ids
}

/// Map level theme ID to GEMS BGM ID.
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
    for sound_id in gems_event_sound_ids(prev, curr) {
        gems.play_sound(sound_id);
    }
}

struct SfxEngine {
    active_tones: Vec<SfxTone>,
}

impl SfxEngine {
    fn new() -> Self {
        Self {
            active_tones: Vec::new(),
        }
    }

    fn play(&mut self, frequency: f32, duration_ms: u32, volume: f32) {
        self.active_tones.push(SfxTone {
            frequency,
            remaining_samples: (SFX_SAMPLE_RATE * duration_ms as f32 / 1000.0) as u32,
            phase: 0.0,
            volume,
        });
    }

    fn generate(&mut self, sample_count: usize) -> Vec<f32> {
        let mut samples = vec![0.0f32; sample_count];
        self.active_tones.retain_mut(|tone| {
            let phase_inc = tone.frequency / SFX_SAMPLE_RATE;
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

    // Jump (left ground with upward velocity)
    if gameplay_sfx_fallback_active
        && prev.kid_on_ground == Some(true)
        && curr.kid_on_ground == Some(false)
        && curr.kid_y_velocity.is_some_and(|v| v < 0)
    {
        sfx.play(440.0, 40, 0.1);
    }
    // Diamond collected
    if curr.diamond_count > prev.diamond_count {
        sfx.play(880.0, 60, 0.15);
    }
    // Enemy killed (score jumped by 1000+)
    if curr.score >= prev.score + 1000 && curr.diamond_count == prev.diamond_count {
        sfx.play(523.0, 80, 0.12);
        sfx.play(659.0, 80, 0.12);
    }
    // Took damage (HP decreased)
    if curr.number_hitpoints < prev.number_hitpoints && prev.number_hitpoints > 0 {
        sfx.play(220.0, 150, 0.2);
    }
    // Form change (helmet changed)
    if curr.current_helmet_id != prev.current_helmet_id
        && curr.game_mode == prev.game_mode
        && !curr.game_over
    {
        sfx.play(392.0, 60, 0.12); // G4
        sfx.play(523.0, 60, 0.12); // C5
        sfx.play(659.0, 80, 0.12); // E5
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
    if curr.stomped_this_frame {
        sfx.play(587.0, 50, 0.1); // D5 bounce tone
    }
    // Rubber contact. ASM loc_10DA4 plays `sfx_Rubber_block = $56`; keep the
    // fallback distinct from the metallic block-headbutt cue below.
    if queued_sound_ids.contains(&gems_sfx::RUBBER_BLOCK) {
        sfx.play(294.0, 55, 0.08);
        sfx.play(196.0, 70, 0.06);
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
    if curr.level_timer_seconds > prev.level_timer_seconds + 1 {
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
    let (layout_path, metadata_path) =
        if logical_layout_path.is_file() && logical_metadata_path.is_file() {
            (logical_layout_path, logical_metadata_path)
        } else {
            // ASM Load_Level_Data resolves Current_LevelID through
            // MapOrder_Index before decoding the BLOCK resource. Prefer
            // generated logical Level_Layout specialisations when present,
            // then fall back to the mapped resource layout for aliased levels.
            let resource_level = resource_level_id(manifest, level_id);
            (
                layout_dir.join(format!("{resource_level:02X}.bin")),
                layout_dir.join(format!("{resource_level:02X}.json")),
            )
        };
    if !layout_path.is_file() || !metadata_path.is_file() {
        return Ok(None);
    }

    let mut data = std::fs::read(&layout_path).map_err(|error| {
        anyhow!(
            "failed to read generated Level_Layout from {}: {error}",
            layout_path.display()
        )
    })?;
    let metadata = serde_json::from_slice::<GeneratedLevelLayoutMetadata>(
        &std::fs::read(&metadata_path).map_err(|error| {
            anyhow!(
                "failed to read Level_Layout metadata from {}: {error}",
                metadata_path.display()
            )
        })?,
    )
    .map_err(|error| {
        anyhow!(
            "failed to parse Level_Layout metadata from {}: {error}",
            metadata_path.display()
        )
    })?;
    data.truncate(metadata.data_bytes.min(data.len()));
    Ok(Some(BlockGrid::from_level_layout(
        &data,
        metadata.width_blocks,
        metadata.height_blocks,
    )))
}

#[cfg(test)]
mod tests {
    use super::*;
    use kid_core::{
        EOL_HOLOGRAM_VISIBLE_FRAME_LOC_C1D0_ORACLE, LevelId, NativeObjectKind,
        physics::Direction,
        terrain::{BlockType, Cell, CollisionKind},
    };
    use kid_data::{
        AssetManifest, DecodedAssetFile, DecodedLevelAssets, GENERATED_BUNDLE_VERSION, LevelHeader,
        LevelRecord, LevelResources, load_manifest,
    };
    use kid_render::{
        AnimatedSpriteVisual, BackgroundChunk, BackgroundPlacement, BackgroundTile,
        BackgroundVisuals, BlockVisualMapping, ChunkedBackgroundVisuals, EnemyVisuals,
        ForegroundLayout, HudIcon, HudVisuals, LevelVisuals, MdColour, NativeLevelVisuals, Palette,
        PaletteSet, PlatformVisuals, PlayerFormVisuals, PlayerTransformStep, PlayerVisuals,
        SpriteFrame, Tile, TileQuad, TileRef, TileSheet,
    };
    use std::fs;
    use std::path::Path;
    use std::sync::Arc;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use tempfile::TempDir;

    fn sample_trace(mode: GameMode) -> NativeTraceFrame {
        NativeTraceFrame {
            frame_index: 720,
            game_mode: mode,
            game_mode_value: mode.as_disassembly_value(),
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
            paused: false,
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
            level_damage_taken: false,
            level_prize_collected: false,
            level_speed_bonus_awarded: false,
            level_path_bonus_amount: 0,
            murder_wall_x: None,
            active_projectiles: Vec::new(),
            active_projectile_details: Vec::new(),
            diamond_power_sprites: Vec::new(),
            active_geysers: Vec::new(),
            block_bounces: Vec::new(),
            block_destroy_effects: Vec::new(),
            block_debris_pieces: Vec::new(),
            prize_block_spawn_flashes: Vec::new(),
            floating_items: Vec::new(),
            enemy_death_effects: Vec::new(),
            stomped_this_frame: false,
            death_windup_elapsed: 0,
            form_change_flash_frames: 0,
            blocks_hit_this_frame: 0,
            sound_test_track: None,
            sound_test_play_pending: false,
            gameplay_debug_fields: std::collections::BTreeMap::new(),
            physics_trace: None,
        }
    }

    fn has_rect(scene: &Scene, x: i32, y: i32, width: i32, height: i32, colour: [u8; 3]) -> bool {
        scene.commands.iter().any(|command| {
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
        let mut runtime = NativeRuntime::new(manifest);
        runtime.set_block_grid(session_assets.block_grid.clone());
        runtime.enter_level_playtest(LevelId(0x01));
        runtime.set_player_probe_position(1544, 383);
        runtime.set_camera_probe_position(1400, 240);

        let mut bullet_scene = None;
        for _ in 0..90 {
            let trace = runtime.step_frame(FrameInput::default());
            if trace.active_projectile_details.iter().any(|projectile| {
                projectile.kind == ActiveProjectileKind::TankBullet
                    && projectile.x_velocity != 0
                    && !projectile.friendly
            }) {
                bullet_scene = Some(build_scene(
                    &trace,
                    false,
                    Some(&session_assets.level_visuals),
                    runtime.block_grid(),
                ));
                break;
            }
        }

        let scene = bullet_scene
            .expect("ASM Tank_Shooting.asm loc_3C50E -> loc_3B730 should emit a live TankBullet");
        assert!(
            scene.commands.iter().any(|command| matches!(
                command,
                DrawCommand::Sprite { tiles, .. }
                    if tiles
                        .iter()
                        .any(|tile| matches!(tile.tile.tile_index, 0x038 | 0x03A))
            )),
            "ASM Tank_Shooting.asm `stru_3B526` should render live `$0B` RockTank bullets with ROM child sprite frames"
        );
        assert!(
            !scene.commands.iter().any(|command| matches!(
                command,
                DrawCommand::Rect {
                    width: 4,
                    height: 4,
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
        let visuals = sample_level_visuals();
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
        }];

        let scene = build_scene(&trace, false, Some(&visuals), None);

        let drop_sprite = scene.commands.iter().find_map(|command| match command {
            DrawCommand::Sprite { x, y, tiles, .. } if *x == 30 && *y == 23 => Some(tiles),
            _ => None,
        });
        let tiles = drop_sprite.expect("ASM Drips.asm loc_33424 should render the falling child");
        assert_eq!(
            tiles[0].tile.tile_index, 0x082,
            "ASM `unk_C7B26: sprite_frame_vram $082,$02,$09,$05,$0A` is the falling-drop child frame",
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
        trace.level_complete_elapsed_frames = kid_core::EOL_HOLOGRAM_VISIBLE_FRAME_LOC_C1D0_ORACLE;

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
        visuals.eol_hologram = Some(kid_render::EolHologramVisuals {
            palette: Palette {
                colours: vec![colour(255, 112, 0)],
            },
            tile_sheet,
            rows: vec![vec![TileRef::from_attr_word(0x0001)]],
        });
        let mut trace = sample_trace(GameMode::InGame);
        trace.level_complete = true;
        trace.level_complete_elapsed_frames = kid_core::EOL_HOLOGRAM_VISIBLE_FRAME_LOC_C1D0_ORACLE;

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
        visuals.eol_hologram = Some(kid_render::EolHologramVisuals {
            palette: Palette {
                colours: vec![
                    kid_render::MdColour::from_word(0x0866),
                    kid_render::MdColour::from_word(0x00E0),
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
        trace.level_complete_elapsed_frames = kid_core::EOL_HOLOGRAM_VISIBLE_FRAME_LOC_C1D0_ORACLE;
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
        visuals.eol_hologram = Some(kid_render::EolHologramVisuals {
            palette: Palette {
                colours: vec![colour(255, 112, 0)],
            },
            tile_sheet: hologram_tiles,
            rows: vec![vec![TileRef::from_attr_word(0x0001)]],
        });
        let mut trace = sample_trace(GameMode::InGame);
        trace.level_complete = true;
        trace.level_complete_elapsed_frames = kid_core::EOL_HOLOGRAM_VISIBLE_FRAME_LOC_C1D0_ORACLE;
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
        visuals.eol_hologram = Some(kid_render::EolHologramVisuals {
            palette: Palette {
                colours: vec![colour(255, 112, 0)],
            },
            tile_sheet,
            rows: vec![vec![TileRef::from_attr_word(0x0001)]],
        });
        let mut grid = BlockGrid::from_level_layout(&[0; 4 * 4 * 2], 4, 4);
        grid.set_cell_at(1, 0, kid_core::terrain::Cell::from_be_bytes(0xE1, 0x00));
        let mut trace = sample_trace(GameMode::InGame);
        trace.level_complete = true;
        trace.level_complete_elapsed_frames = kid_core::EOL_HOLOGRAM_VISIBLE_FRAME_LOC_C1D0_ORACLE
            + kid_core::EOL_FOREGROUND_REVEAL_VISIBLE_DELAY_FRAMES_LOC_C21E_ORACLE
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
        visuals.eol_hologram = Some(kid_render::EolHologramVisuals {
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
        grid.set_cell_at(0, 0, kid_core::terrain::Cell::from_be_bytes(0xE1, 0x00));
        let mut trace = sample_trace(GameMode::InGame);
        trace.level_complete = true;
        trace.level_complete_elapsed_frames = kid_core::EOL_HOLOGRAM_VISIBLE_FRAME_LOC_C1D0_ORACLE
            + kid_core::EOL_FOREGROUND_REVEAL_VISIBLE_DELAY_FRAMES_LOC_C21E_ORACLE
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
        visuals.eol_hologram = Some(kid_render::EolHologramVisuals {
            palette: Palette {
                colours: vec![colour(255, 112, 0), colour(144, 152, 184)],
            },
            tile_sheet,
            rows: vec![vec![TileRef::from_attr_word(0x0001); 2]],
        });
        let mut grid = BlockGrid::from_level_layout(&[0; 4 * 4 * 2], 4, 4);
        grid.set_cell_at(0, 0, kid_core::terrain::Cell::from_be_bytes(0xE1, 0x00));
        let mut trace = sample_trace(GameMode::InGame);
        trace.level_complete = true;
        trace.level_complete_elapsed_frames = kid_core::EOL_HOLOGRAM_VISIBLE_FRAME_LOC_C1D0_ORACLE
            + kid_core::EOL_FOREGROUND_REVEAL_VISIBLE_DELAY_FRAMES_LOC_C21E_ORACLE
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
        visuals.eol_hologram = Some(kid_render::EolHologramVisuals {
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
        trace.level_complete_elapsed_frames = kid_core::EOL_HOLOGRAM_VISIBLE_FRAME_LOC_C1D0_ORACLE
            + kid_core::EOL_FOREGROUND_REVEAL_VISIBLE_DELAY_FRAMES_LOC_C21E_ORACLE
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
        visuals.eol_hologram = Some(kid_render::EolHologramVisuals {
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
        trace.level_complete_elapsed_frames = kid_core::EOL_HOLOGRAM_VISIBLE_FRAME_LOC_C1D0_ORACLE
            + kid_core::EOL_FOREGROUND_REVEAL_VISIBLE_DELAY_FRAMES_LOC_C21E_ORACLE
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
        let time_bonus = kid_core::time_bonus_loc_d56e(trace.level_timer_seconds);
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
        trace.score = kid_core::time_bonus_loc_d56e(trace.level_timer_seconds);

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
                charge: Vec::new(),
                climb: Vec::new(),
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
                charge: Vec::new(),
                climb: vec![
                    sample_player_frame(50),
                    sample_player_frame(51),
                    sample_player_frame(52),
                ],
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
                charge: Vec::new(),
                climb: Vec::new(),
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
                charge: Vec::new(),
                climb: Vec::new(),
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
            charge: vec![
                sample_player_frame(40),
                sample_player_frame(41),
                sample_player_frame(42),
                sample_player_frame(43),
                sample_player_frame(44),
                sample_player_frame(45),
            ],
            climb: Vec::new(),
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
            charge: vec![
                sample_player_frame(40),
                sample_player_frame(41),
                sample_player_frame(42),
                sample_player_frame(43),
                sample_player_frame(44),
                sample_player_frame(45),
            ],
            climb: Vec::new(),
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
            charge: vec![
                sample_player_frame(40),
                sample_player_frame(41),
                sample_player_frame(42),
                sample_player_frame(43),
                sample_player_frame(44),
                sample_player_frame(45),
            ],
            climb: Vec::new(),
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
            charge: vec![
                sample_player_frame(40),
                sample_player_frame(41),
                sample_player_frame(42),
                sample_player_frame(43),
                sample_player_frame(44),
                sample_player_frame(45),
            ],
            climb: Vec::new(),
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
            charge: vec![
                sample_player_frame(40),
                sample_player_frame(41),
                sample_player_frame(42),
                sample_player_frame(43),
                sample_player_frame(44),
                sample_player_frame(45),
            ],
            climb: Vec::new(),
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
            charge: vec![
                sample_player_frame(40),
                sample_player_frame(41),
                sample_player_frame(42),
                sample_player_frame(43),
                sample_player_frame(44),
                sample_player_frame(45),
            ],
            climb: Vec::new(),
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
            charge: Vec::new(),
            climb: Vec::new(),
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
            charge: Vec::new(),
            climb: Vec::new(),
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
            charge: Vec::new(),
            climb: Vec::new(),
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
            charge: Vec::new(),
            climb: vec![
                sample_player_frame(60),
                sample_player_frame(61),
                sample_player_frame(62),
            ],
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
            charge: Vec::new(),
            climb: Vec::new(),
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

    fn sample_floating_prize_visuals() -> kid_render::FloatingPrizeVisuals {
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
        kid_render::FloatingPrizeVisuals {
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

    fn sample_eol_score_visuals() -> kid_render::EolScoreVisuals {
        kid_render::EolScoreVisuals {
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
        NativeLevelVisuals {
            foreground_tiles: blank_tile_sheet(0x80),
            background_tiles: blank_tile_sheet(0x20),
            block_tiles: blank_tile_sheet(0x200),
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
            enemies: sample_enemy_visuals(),
            floating_prizes: sample_floating_prize_visuals(),
            flag: sample_flag_visuals(),
            murder_wall: None,
            lava: None,
            platforms: sample_platform_visuals(),
            eol_score: Some(sample_eol_score_visuals()),
            eol_hologram: None,
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
                .expect("kidcli crate should live under <repo>/crates/kidcli")
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
    }

    fn sample_foreground_layout_bytes() -> Vec<u8> {
        let mut bytes = vec![0; 20 * 14];
        bytes[0] = 0;
        bytes[1] = 3;
        bytes[20] = 5;
        bytes
    }

    fn sample_flag_visuals() -> kid_render::FlagVisuals {
        let palette = Palette {
            colours: vec![colour(12, 34, 56)],
        };
        kid_render::FlagVisuals {
            palette: palette.clone(),
            pole_bottom: kid_render::SpriteFrame {
                origin_x: 8,
                origin_y: 16,
                width_px: 16,
                height_px: 16,
                tile_base_index: 0,
                tile_sheet: blank_tile_sheet(4),
            },
            waving: (0..6)
                .map(|frame| kid_render::SpriteFrame {
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
    fn build_scene_draws_prize_block_wobble_sprite_even_after_deleted_cell_write() {
        // ASM sub_1007A/loc_10200 writes `$6Fxx` to Level_Layout while the
        // wobble worker remains live and draws `unk_E0E4E`/`unk_E0E5E` at the
        // hibernated y offset. Rendering must therefore key off `block_bounces`
        // as well as the cell's block flag, or the prize block vanishes before
        // the post-prize `$E001` smash cell appears.
        let mut trace = sample_trace(GameMode::InGame);
        trace.camera_x_pos = 0;
        trace.camera_y_pos = 0;
        trace.block_bounces = vec![(2, 2, 30, -2)];
        let visuals = sample_level_visuals();
        let data = vec![0u8; 20 * 14 * 2];
        let mut grid = BlockGrid::from_level_layout(&data, 20, 14);
        grid.set_cell_at(2, 2, Cell::from_be_bytes(0x6F, 0x01));

        let scene = build_scene(&trace, false, Some(&visuals), Some(&grid));

        assert!(
            scene.commands.iter().any(|command| matches!(
                command,
                DrawCommand::Sprite {
                    x: 32,
                    y: 30,
                    tile_sheet,
                    tiles,
                    ..
                } if tile_sheet.tiles.len() == 0x200
                    && tiles.first().is_some_and(|tile| tile.tile.tile_index == 0)
            )),
            "active sub_1007A block_bounces entry should draw the deleted `$6Fxx` prize cell as the ROM wobble sprite at the bounced y offset",
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
        // Title_Letter workers (see kid_core::trace `PlayerLifecycle::Dying`
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
        visuals.title_card_letters = Some(kid_render::TitleCardLettersVisuals {
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
            Some(kid_core::FloatingPrizeGraphicsSpec {
                object_meta: 0x0064,
                palette_line: 0,
                animation_script_addr: 0x0001_0D6E,
            })
        );
        assert_eq!(
            floating_item_graphics_spec(0x02),
            Some(kid_core::FloatingPrizeGraphicsSpec {
                object_meta: 0x006C,
                palette_line: 3,
                animation_script_addr: 0x0001_06DC,
            })
        );
        assert_eq!(
            floating_item_graphics_spec(0x0B),
            Some(kid_core::FloatingPrizeGraphicsSpec {
                object_meta: 0x0090,
                palette_line: 0,
                animation_script_addr: 0x0001_0C08,
            })
        );
        assert_eq!(
            floating_item_graphics_spec(0x0C),
            Some(kid_core::FloatingPrizeGraphicsSpec {
                object_meta: 0x0094,
                palette_line: 0,
                animation_script_addr: 0x0001_0C18,
            })
        );
        assert_eq!(
            floating_item_graphics_spec(0x0D),
            Some(kid_core::FloatingPrizeGraphicsSpec {
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
    fn background_pixel_sampler_reports_last_drawn_tile_provenance_loc_c070() {
        let mut trace = sample_trace(GameMode::InGame);
        trace.camera_x_pos = 0;
        trace.camera_y_pos = 0;
        let mut visuals = sample_level_visuals();
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
        assert_eq!(sample.rgb, [40, 50, 60]);
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
        visuals.background = BackgroundVisuals::Layered(kid_render::LayeredBackgroundVisuals {
            effect_start_row_selector: None,
            rows: vec![kid_render::LayeredBackgroundRow {
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
        visuals.background = BackgroundVisuals::Layered(kid_render::LayeredBackgroundVisuals {
            effect_start_row_selector: None,
            rows: vec![kid_render::LayeredBackgroundRow {
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
    fn build_game_scene_draws_murder_wall_from_sub_5e02_sprite_tiles() {
        let mut trace = sample_trace(GameMode::InGame);
        trace.frame_index = 10;
        trace.camera_x_pos = 0;
        trace.camera_y_pos = 0;
        trace.murder_wall_x = Some(0);
        let mut visuals = sample_level_visuals();
        visuals.level.murder_wall_flags = 2;
        visuals.murder_wall = Some(kid_render::MurderWallVisuals {
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
        let trace_frames = kid_core::run_script_with_warp(
            manifest,
            &scenario.script,
            scenario.frame_count,
            scenario.warp_to_level,
        );

        let (trace_index, trace, render) = trace_frames
            .iter()
            .enumerate()
            .filter(|(_, frame)| frame.game_mode == GameMode::InGame)
            .filter_map(|(trace_index, frame)| {
                let wall_x_fp = frame.murder_wall_x?;
                let render = kid_core::compile_murder_wall_warning_sprites_sub_5E02(
                    kid_core::MurderWallWarningInputSub5E02 {
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
            .next()
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
        trace.active_geysers = vec![(0x0080, 0x0040, 0x0030)];
        let mut visuals = sample_level_visuals();
        visuals.lava = Some(kid_render::LavaVisuals {
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
        assert!(
            lava_sprites >= 4,
            "ASM loc_14BE8 should render active lava geysers from Lava.bin sprite chunks"
        );
        assert!(
            !scene.commands.iter().any(|command| {
                matches!(command, DrawCommand::Rect { colour, .. } if *colour == [255, 120, 20])
            }),
            "loaded Lava.bin art should replace the native-player placeholder column"
        );
    }

    #[test]
    fn build_game_scene_keeps_no_scroll_background_layers_screen_fixed() {
        let mut trace = sample_trace(GameMode::InGame);
        trace.camera_x_pos = 0x0020_0000;
        trace.camera_y_pos = 0x0010_0000;
        let mut visuals = sample_level_visuals();
        visuals.level.background_scroll = Some(kid_data::BgscrollProfile {
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
            (trace.kid_x_pos, trace.kid_y_pos),
            "only the VDP scroll latch is delayed; gameplay object state stays on the current trace outside loc_C070"
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
            render_trace.kid_x_pos, trace.kid_x_pos,
            "only the HUD/VDP latch is delayed; gameplay object state stays on the current trace outside loc_C070"
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
    fn layered_background_wraps_only_camera_parallax_rows_loc_3050a() {
        let mut visuals = sample_level_visuals();
        visuals.level.background_scroll = Some(kid_data::BgscrollProfile {
            layer_speeds: vec![0x00, 0x08, 0x10, 0x30],
            has_terminator: true,
        });

        assert!(
            !super::scene::layered_background_row_wraps_loc_3050a(&visuals, 0),
            "ASM loc_3050A auto-scrolling `$00` rows stay finite during the flagpole hand-off"
        );
        assert!(
            !super::scene::layered_background_row_wraps_loc_3050a(&visuals, 1),
            "ASM loc_3050A auto-scrolling `$08` rows stay finite during the flagpole hand-off"
        );
        assert!(
            !super::scene::layered_background_row_wraps_loc_3050a(&visuals, 2),
            "ASM loc_3050A no-scroll `$10` rows do not need native row repetition"
        );
        assert!(
            super::scene::layered_background_row_wraps_loc_3050a(&visuals, 3),
            "ASM loc_3050A camera-parallax rows should repeat so long-travel forest rows do not disappear"
        );
    }

    #[test]
    fn layered_background_wrap_stops_during_flagpole_handoff_loc_c070() {
        let mut visuals = sample_level_visuals();
        visuals.level.background_scroll = Some(kid_data::BgscrollProfile {
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
    fn level_complete_handoff_keeps_flag_sprite_for_first_frame_loc_c070() {
        let mut trace = sample_trace(GameMode::InGame);
        trace.camera_x_pos = 0x0730_0000;
        trace.camera_y_pos = 0x02D0_0000;
        trace.level_complete = true;
        trace.level_complete_elapsed_frames = 0;

        let scene = build_ingame_scene(&trace, false, None);

        assert!(
            scene.commands.iter().any(|command| {
                matches!(command, DrawCommand::Sprite { tile_sheet, .. } if tile_sheet.tiles.len() == 120)
            }),
            "ASM loc_C070 has just entered the flagpole hand-off, so the oracle still shows the waving flag sprite on elapsed=0"
        );
        assert!(
            scene.commands.iter().any(|command| {
                matches!(command, DrawCommand::Sprite { tile_sheet, .. } if tile_sheet.tiles.len() == 4)
            }),
            "ASM Flagpole_Boss pole-bottom sprite remains visible on the elapsed=0 flag-touch frame"
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
            kid_core::HEADYMETAL_DEATH_EFFECT_FRAMES,
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
                kid_core::HEADYMETAL_DEATH_EFFECT_FRAMES,
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
                record_notes: None,
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
                record_notes: None,
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
    fn debug_warp_without_generated_title_card_holds_debug_name_card() {
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
            fallback_frames, LEVEL_CARD_HOLD_FRAMES,
            "debug N/P targets without generated Load_TitleCard script data need a visible tool-only name hold"
        );

        if !native_level_card_hold_active(0, fallback_frames) {
            trace = runtime.step_frame(FrameInput::default());
        }

        assert_eq!(trace.game_mode, GameMode::LevelTransition);
        assert_eq!(trace.current_level_name.as_deref(), Some("Debug Target"));
        let scene = build_debug_level_card_fallback_scene(&trace);
        assert!(contains_text(&scene, "LEVEL 01"));
        assert!(contains_text(&scene, "Debug Target"));

        fallback_frames = fallback_frames.saturating_sub(1);
        assert!(native_level_card_hold_active(0, fallback_frames));
    }

    #[test]
    fn debug_warp_with_generated_title_card_uses_rom_title_shell() {
        let mut fixture = ManifestFixture::new().expect("manifest fixture");
        let mut target = fixture.manifest.levels[0].clone();
        target.level_id = 0x01;
        target.display_name = Some(String::from("ROM Card Target"));
        target.title_text_id = Some(0x4B);
        target.title_card = Some(kid_data::LevelTitleCard {
            script_bytes: vec![b'a', 0xFF],
            line_destinations: vec![kid_data::TitleCardDestination { x: 0x80, y: 0x70 }],
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
    fn debug_hotkey_level_skip_always_holds_visible_name_card() {
        assert_eq!(
            debug_hotkey_level_card_hold_frames(),
            LEVEL_CARD_HOLD_FRAMES,
            "native-only N/P debug skips should visibly acknowledge the target level even when generated ROM title-card data exists"
        );
        assert_eq!(
            debug_hotkey_level_presentation(true).level_card_frames_remaining,
            LEVEL_CARD_HOLD_FRAMES,
            "renderable Load_TitleCard targets should hold the ROM title-card shell"
        );
        assert_eq!(
            debug_hotkey_level_presentation(true).debug_level_card_fallback_frames_remaining,
            0,
            "renderable Load_TitleCard targets should not be masked by the debug fallback"
        );
        assert_eq!(
            debug_hotkey_level_presentation(false).debug_level_card_fallback_frames_remaining,
            LEVEL_CARD_HOLD_FRAMES,
            "targets without renderable title-card VDP need the deterministic fallback"
        );
    }

    #[test]
    fn debug_hotkey_level_skip_enters_visible_card_hold_before_gameplay() {
        let mut fixture = ManifestFixture::new().expect("manifest fixture");
        let mut target = fixture.manifest.levels[0].clone();
        target.level_id = 0x01;
        target.display_name = Some(String::from("Hotkey Target"));
        target.title_text_id = Some(0x4B);
        target.title_card = Some(kid_data::LevelTitleCard {
            script_bytes: vec![b'H', 0xFF],
            line_destinations: vec![kid_data::TitleCardDestination { x: 0x80, y: 0x70 }],
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
            presentation.debug_level_card_fallback_frames_remaining, LEVEL_CARD_HOLD_FRAMES,
            "fixture metadata has generated title-card script bytes but no renderable VDP art, so the hotkey must fall back instead of rendering stale gameplay"
        );
        assert_eq!(presentation.level_card_frames_remaining, 0);
        assert!(presentation.wait_for_live_input_release);
        assert!(!presentation.suppress_level_card_for_debug_warp);
        assert!(native_level_card_hold_active(
            presentation.level_card_frames_remaining,
            presentation.debug_level_card_fallback_frames_remaining
        ));
        assert!(
            runtime.vdp_state().is_none(),
            "the tiny unit fixture lacks extracted title-card art; live extracted manifests exercise the ROM-card branch"
        );
    }

    #[test]
    fn debug_hotkey_level_skip_uses_fallback_only_without_generated_title_card() {
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
            presentation.debug_level_card_fallback_frames_remaining, LEVEL_CARD_HOLD_FRAMES,
            "fallback card is reserved for missing generated title-card data"
        );

        let render_trace = {
            let mut overlay = trace.clone();
            overlay.game_mode = GameMode::LevelTransition;
            overlay.game_mode_value = GameMode::LevelTransition.as_disassembly_value();
            overlay
        };
        let scene = build_debug_level_card_fallback_scene(&render_trace);
        assert!(contains_text(&scene, "LEVEL 01"));
        assert!(contains_text(&scene, "Hotkey Fallback"));
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
        write_file(fixture._root.path(), "level-layout/01.bin", &[0x60, 0x42])
            .expect("write resource layout");
        write_file(
            fixture._root.path(),
            "level-layout/01.json",
            br#"{"data_bytes":2,"width_blocks":1,"height_blocks":1}"#,
        )
        .expect("write resource metadata");

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
                record_notes: Some(String::from("stuck on level card")),
                scripted_route: Some(NativeScriptedRoute {
                    script: vec![16, 0],
                    delay_recording_input: false,
                    start_position: Some(DynamicStartPosition { x: 230, y: 271 }),
                    start_velocity: None,
                    camera_position: Some(DynamicStartPosition { x: 16, y: 136 }),
                    playtest_form: Some(PlayerForm::RedStealth.helmet_id()),
                    initial_diamond_count: Some(12),
                }),
            },
            vec![16, 0],
            vec![trace.clone(), sfx_trace],
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Some(final_frame),
            trace.clone(),
            1,
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
        let audio_events =
            fs::read_to_string(record_dir.join("audio-events.txt")).expect("audio events");
        assert!(audio_events.contains("frame=1"));
        assert!(audio_events.contains("PlaySound(13)"));
        assert!(manifest_text.contains("\"record_frames\": 900"));
        assert!(manifest_text.contains("--record freeze-repro --record-frames 900"));
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
            record_notes: None,
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

        // 2x2 grid: one prize block at (0,0), rest empty
        let mut data = vec![0u8; 2 * 2 * 2];
        data[0] = 0xE1; // block=true, collision=Solid, block_type=Prize
        let mut grid = BlockGrid::from_level_layout(&data, 2, 2);

        // Before destruction: prize cell visible
        let scene = build_ingame_scene(&trace, false, Some(&grid));
        assert!(indexed_tile_at_with_sheet_len(&scene, 0, 0, 0x200).is_some());
        assert!(indexed_tile_at_with_sheet_len(&scene, 0, 0, 0x80).is_some());

        // Destroy the block override; the immutable terrain art should remain.
        grid.set_cell_at(0, 0, Cell::from_be_bytes(0, 0));
        let scene = build_ingame_scene(&trace, false, Some(&grid));
        assert!(indexed_tile_at_with_sheet_len(&scene, 0, 0, 0x200).is_none());
        assert!(indexed_tile_at_with_sheet_len(&scene, 0, 0, 0x80).is_some());
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
        (1..=52).contains(&signature)
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
            DrawCommand::Sprite { tile_sheet, .. } => tile_sheet
                .tiles
                .first()
                .is_some_and(|tile| tile.pixels[0] == signature),
            _ => false,
        })
    }

    fn has_asset_backed_command(scene: &Scene, signature: u8) -> bool {
        scene.commands.iter().any(|command| match command {
            DrawCommand::IndexedTile {
                tile_sheet, tile, ..
            } => tile_sheet
                .tiles
                .get(usize::from(tile.tile_index))
                .is_some_and(|resolved| resolved.pixels[0] == signature),
            DrawCommand::Sprite { tile_sheet, .. } => tile_sheet
                .tiles
                .first()
                .is_some_and(|resolved| resolved.pixels[0] == signature),
            _ => false,
        })
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
        trace.active_objects = vec![
            NativeObjectSnapshot {
                kind: NativeObjectKind::Platform {
                    kind: 0x00,
                    subtype: 0x00,
                },
                x_pos: 0x0010_0000,
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
        assert!(
            has_sprite_signature(&scene, 21),
            "expected collectible diamond ROM art"
        );
        assert!(!scene.commands.iter().any(|command| {
            matches!(command, DrawCommand::Rect { colour, .. } if *colour == PLATFORM_COLOUR)
        }));
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

        let normal_phase = trace_with_player(4, 64);
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
            Some(19),
            "ASM sub_44DC loc_45EE stores `$FFFFFA5D=6`, selecting the fourth `unk_461C` diamond DMA page"
        );
        assert_eq!(
            diamond_page_sample(&frame_index_60_same_offset),
            Some(19),
            "HUD diamond rendering should use traced sub_44DC RAM, not replay-local frame_index phase"
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

        assert_eq!(
            count_hud_hitpoint_tiles(&skycutter_scene),
            (PlayerForm::Skycutter.max_hitpoints() / 2) as usize * 2
        );

        let mut iron_knight = sample_trace(GameMode::InGame);
        iron_knight.current_helmet_id = PlayerForm::IronKnight.helmet_id();
        iron_knight.number_hitpoints = PlayerForm::IronKnight.max_hitpoints();

        let iron_knight_scene = build_ingame_scene(&iron_knight, false, None);

        assert_eq!(
            count_hud_hitpoint_tiles(&iron_knight_scene),
            (PlayerForm::IronKnight.max_hitpoints() / 2) as usize * 2
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
        let one_generated_frame_bytes = generated_stereo_samples * std::mem::size_of::<f32>();

        assert!(
            should_queue_audio_frame(
                u32::try_from(one_generated_frame_bytes - 1).expect("fits u32"),
                generated_stereo_samples,
            ),
            "SDL_GetQueuedAudioSize reports bytes; just under one generated frame should accept the next low-latency audio frame"
        );
        assert!(
            !should_queue_audio_frame(
                u32::try_from(one_generated_frame_bytes).expect("fits u32"),
                generated_stereo_samples,
            ),
            "play-native should not keep multiple generated frames queued ahead, because pickup and block SFX then lag behind the visual frame"
        );
    }

    #[test]
    fn play_native_audio_diagnostics_record_generated_and_queued_frames() {
        let mut diagnostics = NativeAudioDiagnostics::new(true, true, 2);

        diagnostics.record_generated_frame(AUDIO_SAMPLES_PER_FRAME * 2);
        diagnostics.record_queued_frame(AUDIO_SAMPLES_PER_FRAME * 2, 0);
        diagnostics.record_frame_timing(Duration::from_micros(12_000), FRAME_TIME);
        diagnostics.record_generated_frame(AUDIO_SAMPLES_PER_FRAME * 2);
        diagnostics.record_skipped_full_frame(8192);
        diagnostics.record_frame_timing(Duration::from_micros(20_000), FRAME_TIME);

        let lines = diagnostics.into_lines();
        assert!(
            lines.contains(&String::from("gems_loaded=true"))
                && lines.contains(&String::from("sdl_queue_present=true"))
                && lines.contains(&String::from("generated_frames=2"))
                && lines.contains(&String::from("queued_frames=1"))
                && lines.contains(&String::from("skipped_full_frames=1"))
                && lines.contains(&String::from("queue_errors=0"))
                && lines.contains(&String::from("min_queue_bytes_before_queue=0"))
                && lines.contains(&String::from("max_queue_bytes_before_queue=8192"))
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
                    "frame=1 mode=OptionsMenu level=00 command=PlaySound({})",
                    gems_sfx::NAVIGATE_JINGLE
                ),
                String::from(
                    "frame=2 mode=InGame level=00 active_before=Some(Options) active_after=Some(Level(LevelId(0))) command=StopMusicThenStart(7)"
                ),
                format!(
                    "frame=2 mode=InGame level=00 command=PlaySound({})",
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
                "frame=0 mode=InGame level=00 active_before=None active_after=Some(Level(LevelId(0))) command=Start(7)"
            ),
            "cold play-native recordings should still queue ASM loc_12004 level music"
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
                .any(|event| event.contains("command=PlaySound(35)")),
            "settled dynamic probes must still preserve ASM sub_868E Red Stealth sword SFX events"
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
            (None, Some(NativeBgmPlaybackCommand::StopMusic)),
            "ASM Obj_TitleMenu calls StopMusic as soon as Start leaves the title shell, before the Blue Lake Woods level card and later Load_InGame BGM",
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
    fn play_native_initial_level_bgm_starts_without_stop_all() -> anyhow::Result<()> {
        let fixture = ManifestFixture::new()?;
        let trace = sample_trace(GameMode::InGame);

        assert_eq!(
            native_bgm_state_after_trace(&fixture.manifest, &trace, None),
            (
                Some(NativeBgmCue::Level(LevelId(0))),
                Some(NativeBgmPlaybackCommand::Start(gems_bgm::CAVE)),
            ),
            "direct settled native play has no previous music context, so queueing stop-all before the first level BGM can silence the standalone GEMS start"
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
        let mut sfx = SfxEngine::new();

        detect_sfx_events(&prev, &curr, &mut sfx);

        assert!(
            sfx.active_tones.is_empty(),
            "ASM loc_1AE2E title-card hibernate must suppress the non-GEMS jump-tone fallback as well as the GEMS jump cue"
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
    }

    #[test]
    fn play_native_replays_eol_score_counter_sfx_again_at_loc_d7f4() {
        // ASM loc_D7F4 (kid.asm:16906-16908): once the score-counter drain
        // completes, the ROM replays `sfx_Score_counter` ($0C) a second time
        // via `j_PlaySound2` before hibernating and routing to Teleport. The
        // drain-start cue alone left the level-complete jingle ending abruptly
        // (#216 "level-complete sound does not finish cleanly").
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
            "ASM loc_D7F4 replays sfx_Score_counter when the EOL counter finishes draining"
        );

        // The cue must fire exactly once: the next frame is still drained.
        let mut still_drained = drained.clone();
        still_drained.level_complete_score_counter_elapsed_frames = 12;
        assert!(
            !gems_event_sound_ids(&drained, &still_drained).contains(&gems_sfx::SCORE_COUNTER),
            "the loc_D7F4 cue is a single replay, not a per-frame repeat"
        );
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
            "ASM loc_2510C plays sfx_Diamond_prize on the diamond-prize contact frame"
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
                    "frame=2 mode=InGame level=00 command=PlaySound({})",
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
