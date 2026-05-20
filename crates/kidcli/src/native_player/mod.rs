use crate::recording::{
    RecordingBundleArtifacts, build_recording, recording_start_from_options,
    save_recording_with_artifacts,
};
use crate::scenarios::GameplayCheckpoint;
use anyhow::{Result, anyhow, bail};
use font8x8::{BASIC_FONTS, UnicodeFonts};
use kid_core::{
    ActiveProjectileKind, ActiveProjectileSnapshot, FrameInput, GameMode, LevelId,
    NativeObjectKind, NativeObjectSnapshot, NativeRuntime, NativeTraceFrame, PlayerForm,
    physics::Direction, terrain::BlockGrid,
};
use kid_data::{
    AssetManifest, DecodedLevelAssets, LevelHeader, resource_level_id, resource_level_record,
};
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
use serde::Deserialize;
use std::path::PathBuf;
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};

const FRAME_WIDTH: u32 = 320;
const FRAME_HEIGHT: u32 = 224;
const FRAME_TIME: Duration = Duration::from_micros(16_667);
const LEVEL_CARD_HOLD_FRAMES: usize = 60;
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

pub(crate) use render::render_scene;
pub(crate) use scene::{LayerFilter, build_scene, build_scene_filtered};
pub(crate) use sprites::render_dynamic_gameplay_frame_evidence;
pub(crate) use sprites::render_gameplay_frame_evidence;
pub(crate) use sprites::render_gameplay_frame_evidence_for_level;

pub(crate) const NATIVE_FRAME_WIDTH: u32 = FRAME_WIDTH;
pub(crate) const NATIVE_FRAME_HEIGHT: u32 = FRAME_HEIGHT;

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
                samples: Some(1024),
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
    let mut in_game_bgm_playing = false; // triggers BGM on first InGame entry
    let mut fade_alpha: u8 = 0; // 0=no fade, 255=fully black
    let mut fading_in = false; // true: fade from black to clear
    let mut level_card_frames_remaining: usize = 0;
    let mut recorded_script = Vec::new();
    let mut recorded_trace = Vec::new();
    let mut recorded_audio = Vec::new();
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
                    let next = runtime.next_level_id();
                    runtime.warp_to_level(next);
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
                    let prev = runtime.prev_level_id();
                    runtime.warp_to_level(prev);
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
            level_card_frames_remaining = 0;
        } else {
            prev_trace = trace.clone();
            if level_card_frames_remaining == 0 {
                let buttons = keyboard_buttons(&events) | controller_buttons(controller.as_ref());
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

            // Show level card when the engine first enters gameplay
            // (title → StartingLevel maps to GameMode::InGame in trace)
            let entered_gameplay = !matches!(prev_trace.game_mode, GameMode::InGame)
                && matches!(trace.game_mode, GameMode::InGame)
                && options.warp_level.is_none()
                && !options.settled;
            if entered_gameplay {
                level_card_frames_remaining = LEVEL_CARD_HOLD_FRAMES;
            }
        }

        let should_capture_audio = options.record_to.is_some();
        let should_step_audio = should_capture_audio || audio_queue.is_some();
        if should_step_audio {
            let maybe_samples = if let Some(ref mut gems) = gems_engine {
                detect_gems_events(&prev_trace, &trace, gems);
                gems.step_frame();
                Some(gems.generate_samples(735))
            } else {
                Some(sfx_engine.generate(735))
            };
            if let Some(samples) = maybe_samples {
                if should_capture_audio {
                    recorded_audio.extend_from_slice(&samples);
                }
                if let Some(ref aq) = audio_queue
                    && aq.size() < 8192
                {
                    let _ = aq.queue_audio(&samples);
                }
            }
        }

        // Trigger BGM when entering InGame for the first time or on level change
        let level_changed = trace.current_level.0 != current_level_id;
        let entered_in_game = matches!(trace.game_mode, GameMode::InGame | GameMode::DemoPlay)
            && !in_game_bgm_playing;
        if level_changed || entered_in_game {
            let new_level = trace.current_level.0;
            if level_changed {
                // Reload presentation assets only — engine already set up the new level
                // (advance_level calls reload_block_grid_for_level internally)
                match load_native_session_assets(manifest, new_level) {
                    Ok(new_assets) => {
                        session_assets = new_assets;
                        current_level_id = new_level;
                    }
                    Err(error) => {
                        eprintln!("failed to load visuals for level 0x{new_level:02X}: {error:#}");
                    }
                }
                fade_alpha = 255;
                fading_in = true;
                level_card_frames_remaining = LEVEL_CARD_HOLD_FRAMES;
            }
            if let Some(ref mut gems) = gems_engine {
                let level_theme =
                    resolved_level_header(manifest, new_level).map(|header| header.level_theme);
                if let Some(bgm) = level_theme.and_then(theme_to_bgm) {
                    gems.play_sound(bgm);
                }
            }
            if matches!(trace.game_mode, GameMode::InGame | GameMode::DemoPlay) {
                in_game_bgm_playing = true;
            }
        }
        if !matches!(trace.game_mode, GameMode::InGame | GameMode::DemoPlay) {
            in_game_bgm_playing = false;
        }

        let render_trace = if level_card_frames_remaining > 0 {
            let mut overlay_trace = trace.clone();
            overlay_trace.game_mode = GameMode::LevelTransition;
            overlay_trace.game_mode_value = GameMode::LevelTransition.as_disassembly_value();
            overlay_trace
        } else {
            trace.clone()
        };

        if let Some(vdp) = runtime.vdp_state() {
            // Non-gameplay mode with VDP state: render directly using the
            // proven software VDP renderer for pixel-accurate output.
            let snapshot = kid_md::VdpSnapshot {
                registers: vdp.registers,
                vram: vdp.vram.clone(),
                cram: vdp.cram.clone(),
                vsram: vdp.vsram.clone(),
            };
            kid_md::render_vdp_into(&snapshot, &mut frame);
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

        if dump_recording_and_exit {
            break 'running;
        }

        let elapsed = frame_started.elapsed();
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
        final_frame_rgb,
        trace,
        audio_channels,
    )?;

    Ok(())
}

fn recording_frame_limit_reached(options: &NativePlayOptions, recorded_frames: usize) -> bool {
    options
        .record_frames
        .is_some_and(|limit| recorded_frames >= limit)
}

fn save_native_recording_bundle(
    manifest: &AssetManifest,
    options: &NativePlayOptions,
    mut recorded_script: Vec<u8>,
    mut recorded_trace: Vec<NativeTraceFrame>,
    recorded_audio: Vec<f32>,
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

    let slug = record_to
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| {
            anyhow!(
                "recording path has no valid file name: {}",
                record_to.display()
            )
        })?;
    let recording = build_recording(
        manifest,
        slug,
        recording_start_from_options(options),
        recorded_script,
        recorded_trace,
    )?;
    let artifacts = RecordingBundleArtifacts {
        user_notes: options.record_notes.clone(),
        final_frame_rgb,
        audio_samples: Some(recorded_audio),
        audio_channels: u16::from(audio_channels),
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
    let mut runtime = NativeRuntime::new(manifest.clone());
    runtime.set_block_grid(session_assets.block_grid.clone());
    if let Some(level_id) = options.warp_level {
        if options.settled {
            runtime.enter_level_playtest(LevelId(level_id));
        } else {
            runtime.warp_to_level(LevelId(level_id));
        }
    } else if options.settled {
        runtime.enter_level_playtest(LevelId(0));
    }
    runtime
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
    let sound_dir = manifest.source_root.join("sound");
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

/// GEMS sound IDs for Kid Chameleon (from constants.asm).
#[allow(dead_code)]
mod gems_sfx {
    // General
    pub const SCORE_COUNTER: u8 = 0x0C;
    pub const BLOCK_HIT: u8 = 0x0D;
    pub const CLOCK_PRIZE: u8 = 0x0E;
    pub const DESTROY_BLOCK: u8 = 0x11;
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
    pub const JUMP_ON_ENEMY: u8 = 0x4C;
    pub const VOICE_NO_TIME: u8 = 0x51;
    pub const RUBBER_BLOCK: u8 = 0x56;
    pub const VOICE_DIE: u8 = 0x60;
    pub const VOICE_OUCH: u8 = 0x61;
    pub const VOICE_DIAMOND: u8 = 0x62;
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
    pub const RED_STEALTH_SWORD: u8 = 0x23;
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

/// Map level theme ID to GEMS BGM ID.
fn theme_to_bgm(theme: u8) -> Option<u8> {
    match theme {
        0 => Some(gems_bgm::FOREST),
        1 => Some(gems_bgm::ISLAND),
        2 => Some(gems_bgm::DESERT),
        3 => Some(gems_bgm::ICE),
        4 => Some(gems_bgm::CAVE),
        5 => Some(gems_bgm::SKY),
        6 => Some(gems_bgm::MOUNTAIN),
        7 => Some(gems_bgm::HILLS),
        8 => Some(gems_bgm::CITY),
        9 => Some(gems_bgm::SWAMP),
        _ => None,
    }
}

/// Detect game events and trigger GEMS sounds.
fn detect_gems_events(
    prev: &NativeTraceFrame,
    curr: &NativeTraceFrame,
    gems: &mut GemsAudioEngine,
) {
    // Sound test: play the selected track when A/C pressed
    if curr.sound_test_play_pending
        && let Some(track) = curr.sound_test_track
    {
        gems.play_sound(track);
    }

    // Jump
    if prev.kid_on_ground == Some(true)
        && curr.kid_on_ground == Some(false)
        && curr.kid_y_velocity.is_some_and(|v| v < 0)
    {
        gems.play_sound(gems_sfx::JUMP);
    }

    // Stomp enemy (score jumped by 1000+, not from diamond collection)
    if curr.score >= prev.score + 1000 && curr.diamond_count == prev.diamond_count {
        gems.play_sound(gems_sfx::JUMP_ON_ENEMY);
    }

    // Diamond collected
    if curr.diamond_count > prev.diamond_count {
        gems.play_sound(gems_sfx::DIAMOND_PRIZE);
    }

    // Damage taken (HP decreased)
    if curr.number_hitpoints < prev.number_hitpoints && prev.number_hitpoints > 0 {
        gems.play_sound(gems_sfx::VOICE_OUCH);
    }

    // Death (lives decreased)
    if curr.number_lives < prev.number_lives {
        gems.play_sound(gems_sfx::VOICE_DIE);
    }

    // Form change (helmet ID changed)
    if curr.current_helmet_id != prev.current_helmet_id {
        let sfx = match curr.current_helmet_id {
            1 => Some(gems_sfx::IRON_KNIGHT_TRANSFORM),
            2 => Some(gems_sfx::RED_STEALTH_TRANSFORM),
            3 => Some(gems_sfx::SKYCUTTER_TRANSFORM),
            4 => Some(gems_sfx::CYCLONE_TRANSFORM),
            5 => Some(gems_sfx::MANIAXE_TRANSFORM),
            6 => Some(gems_sfx::JUGGERNAUT_TRANSFORM),
            7 => Some(gems_sfx::EYECLOPS_TRANSFORM),
            8 => Some(gems_sfx::MICROMAX_TRANSFORM),
            9 => Some(gems_sfx::BERZERKER_TRANSFORM),
            _ => None,
        };
        if let Some(id) = sfx {
            gems.play_sound(id);
        }
    }

    // Diamond Power activation
    if curr.diamond_power_active && !prev.diamond_power_active {
        gems.play_sound(gems_sfx::DIAMOND_POWER_AVAILABLE);
    }

    // Level complete (score counter ticking)
    if curr.level_complete && !prev.level_complete {
        gems.play_sound(gems_sfx::SCORE_COUNTER);
    }

    // Block headbutt
    if curr.blocks_hit_this_frame > 0 {
        gems.play_sound(gems_sfx::BLOCK_HIT);
    }

    // Block destroyed (new destroy effects appeared)
    if curr.block_destroy_effects.len() > prev.block_destroy_effects.len() {
        gems.play_sound(gems_sfx::DESTROY_BLOCK);
    }

    // Lava geyser erupted (new geysers appeared)
    if curr.active_geysers.len() > prev.active_geysers.len() {
        gems.play_sound(gems_sfx::LAVA_GEYSER);
    }

    // Game complete — ending music
    if curr.game_complete && !prev.game_complete {
        gems.play_sound(gems_bgm::ENDING);
    }

    // Clock pickup (timer jumped up)
    if curr.level_timer_seconds > prev.level_timer_seconds + 5 {
        gems.play_sound(gems_sfx::CLOCK_PRIZE);
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
    // Jump (left ground with upward velocity)
    if prev.kid_on_ground == Some(true)
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
    // Block hit SFX (metallic clang for any block headbutt)
    if curr.blocks_hit_this_frame > 0 {
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
    if let Some(grid) = try_load_generated_level_layout(manifest, level_id)? {
        return Ok(grid);
    }

    let resource_level = resource_level_id(manifest, level_id);
    let header = resolved_level_header(manifest, level_id).ok_or_else(|| {
        anyhow!(
            "generated bundle is missing header metadata for logical level 0x{level_id:02X} (resource level 0x{resource_level:02X})"
        )
    })?;
    let decoded = resolved_decoded_level(manifest, level_id);
    let Some(decoded) = decoded else {
        // Layout not yet extracted for this level — return an empty grid so the
        // level still loads (no block collision). The user can extract it with:
        //   mise run dump-level-layout 2E
        eprintln!(
            "warning: no block layout for level 0x{level_id:02X} (run `mise run dump-level-layout {level_id:02X}` to extract)"
        );
        let (width, height) = level_layout_dimensions(header);
        return Ok(BlockGrid::from_level_layout(&[], width, height));
    };

    let layout_path = manifest.output_root.join(&decoded.block.relative_path);
    let data = std::fs::read(&layout_path).map_err(|error| {
        anyhow!(
            "failed to read block layout from {}: {error}",
            layout_path.display()
        )
    })?;
    let (width, height) = level_layout_dimensions(header);
    Ok(BlockGrid::from_level_layout(&data, width, height))
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

fn resolved_decoded_level(manifest: &AssetManifest, level_id: u8) -> Option<&DecodedLevelAssets> {
    let resource_level = resource_level_id(manifest, level_id);
    manifest
        .decoded_levels
        .iter()
        .find(|decoded| decoded.level_id == resource_level)
        .or_else(|| {
            manifest
                .decoded_levels
                .iter()
                .find(|decoded| decoded.level_id == level_id)
        })
}

fn try_load_generated_level_layout(
    manifest: &AssetManifest,
    level_id: u8,
) -> Result<Option<BlockGrid>> {
    let layout_dir = manifest.output_root.join("level-layout");
    let layout_path = layout_dir.join(format!("{level_id:02X}.bin"));
    let metadata_path = layout_dir.join(format!("{level_id:02X}.json"));
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

fn level_layout_dimensions(header: &LevelHeader) -> (u16, u16) {
    // Kid's 16x16 collision grid spans 20x14 cells per 320x224 screen.
    (
        u16::from(header.width_screens) * 20,
        u16::from(header.height_screens) * 14,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use kid_core::{
        LevelId, NativeObjectKind,
        physics::Direction,
        terrain::{BlockType, Cell, CollisionKind},
    };
    use kid_data::{
        AssetManifest, DecodedAssetFile, DecodedLevelAssets, GENERATED_BUNDLE_VERSION, LevelHeader,
        LevelRecord, LevelResources,
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
            kid_on_slope: 0,
            active_objects: Vec::new(),
            camera_x_pos: 0x0010_0000,
            camera_y_pos: 0x0008_0000,
            number_objects: 3,
            number_gfx_objects: 4,
            rng_offset: None,
            level_complete: false,
            game_over: false,
            continue_countdown: None,
            game_complete: false,
            ending_elapsed: None,
            options_cursor: None,
            two_player: false,
            options_two_player_two_controllers: false,
            options_controls_scheme: 0,
            options_speed_button_fast: false,
            number_continues: 3,
            invulnerable_frames: 0,
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
            active_geysers: Vec::new(),
            block_bounces: Vec::new(),
            block_destroy_effects: Vec::new(),
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

    #[test]
    fn generic_enemy_projectile_uses_legacy_dot_render() {
        let mut trace = sample_trace(GameMode::InGame);
        trace.active_projectiles = vec![(0x0030_0000, 0x0020_0000, false)];

        let scene = build_ingame_scene(&trace, false, None);

        assert!(has_rect(&scene, 30, 22, 4, 4, [255, 80, 40]));
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
            friendly: false,
            remaining_frames: 45,
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
            friendly: false,
            remaining_frames: 45,
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
    fn drips_drop_uses_rom_child_sprite_frame_loc_33424() {
        let visuals = sample_level_visuals();
        let mut trace = sample_trace(GameMode::InGame);
        trace.active_projectile_details = vec![ActiveProjectileSnapshot {
            x_pos: 0x0030_0000,
            y_pos: 0x0020_0000,
            friendly: false,
            remaining_frames: 200,
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
    fn level_complete_overlay_uses_eol_sprite_workers_loc_d55e() {
        let visuals = sample_level_visuals();
        let mut trace = sample_trace(GameMode::InGame);
        trace.level_complete = true;
        trace.level_timer_seconds = 180;
        trace.level_damage_taken = false;
        trace.level_prize_collected = false;
        trace.level_path_bonus_amount = 3_000;
        trace.level_speed_bonus_awarded = true;
        trace.score = 12_340;

        let scene = build_scene(&trace, false, Some(&visuals), None);

        assert!(
            has_sprite_signature(&scene, 128),
            "ASM loc_D55E should render Spr_EOL_Time instead of typed TIME text",
        );
        assert!(
            has_sprite_signature(&scene, 121),
            "ASM loc_D55E should render Spr_EOL_Bonus label sprites",
        );
        assert!(
            has_sprite_signature(&scene, 131),
            "ASM Object_EOL_Number should render digit sprite workers",
        );
        assert!(!contains_text(&scene, "TIME BONUS"));
        assert!(!contains_text(&scene, "SCORE:"));
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
                            && tile_sheet.tiles.len() == 0x200
                            && [13, 15].contains(&tile.tile_index)
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
                slope_walk: Vec::new(),
                slope_walk_left: Vec::new(),
                jump_rise: sample_player_frame(8),
                jump_fall: sample_player_frame(9),
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
                slope_walk: Vec::new(),
                slope_walk_left: Vec::new(),
                jump_rise: sample_player_frame(17),
                jump_fall: sample_player_frame(18),
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
                slope_walk: vec![
                    sample_player_frame(38),
                    sample_player_frame(39),
                    sample_player_frame(39),
                    sample_player_frame(38),
                    sample_player_frame(39),
                    sample_player_frame(39),
                ],
                slope_walk_left: vec![
                    sample_player_frame(38),
                    sample_player_frame(39),
                    sample_player_frame(39),
                    sample_player_frame(38),
                    sample_player_frame(39),
                    sample_player_frame(39),
                ],
                jump_rise: sample_player_frame(23),
                jump_fall: sample_player_frame(24),
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

        PlayerVisuals { forms }
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
                frames: vec![SpriteFrame {
                    origin_x: 0x0E,
                    origin_y: 0,
                    width_px: 0x1A,
                    height_px: 0x20,
                    tile_base_index: 0,
                    tile_sheet: TileSheet {
                        tiles: (0..0xC0)
                            .map(|index| Tile {
                                pixels: [u8::try_from(index & 0xFF).expect("tile signature"); 64],
                            })
                            .collect(),
                    },
                }],
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
                    sample_object_frame(21, 20, 20),
                    sample_object_frame(22, 20, 20),
                    sample_object_frame(23, 20, 20),
                ],
                frame_duration: 6,
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
                ],
                frame_duration: 9,
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
                frames: vec![SpriteFrame {
                    origin_x: 0x0C,
                    origin_y: 0,
                    width_px: 0x0F,
                    height_px: 0x0E,
                    tile_base_index: 0,
                    tile_sheet: blank_tile_sheet(0x090),
                }],
                frame_duration: 8,
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
        helmets.insert(
            0x02,
            AnimatedSpriteVisual {
                palette: helmet_palette,
                extra_palettes: vec![],
                frames: vec![sample_object_frame(91, 0x11, 0x0E)],
                frame_duration: 1,
            },
        );
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
                frames: vec![SpriteFrame {
                    origin_x: 0x06,
                    origin_y: -2,
                    width_px: 0x10,
                    height_px: 0x0C,
                    tile_base_index: 8,
                    tile_sheet: blank_tile_sheet(16),
                }],
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
            bonus: sample_object_frame_with_base(121, 0x40, 0x08, 0x000),
            no_hit: sample_object_frame_with_base(122, 0x57, 0x08, 0x065),
            no_prize: sample_object_frame_with_base(123, 0x6F, 0x08, 0x070),
            path: sample_object_frame_with_base(124, 0x35, 0x08, 0x080),
            score: sample_object_frame_with_base(125, 0x3D, 0x08, 0x091),
            seconds: sample_object_frame_with_base(126, 0x5F, 0x08, 0x099),
            speed: sample_object_frame_with_base(127, 0x3F, 0x08, 0x0AD),
            time: sample_object_frame_with_base(128, 0x2D, 0x08, 0x0B9),
            under: sample_object_frame_with_base(129, 0x40, 0x08, 0x0C3),
            digits: (0..10)
                .map(|digit| {
                    sample_object_frame_with_base(130 + digit, 0x0C, 0x0B, 0x0CB + digit as u16)
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
            title_card_letters: None,
        }
    }

    fn sample_level_visuals() -> NativeLevelVisuals {
        sample_level_visuals_with_background([33, 66, 99])
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
    fn build_scene_falls_back_to_static_text_when_level_transition_has_no_title_workers() {
        // Death/respawn transitions reach LevelTransition without seeding any
        // Title_Letter workers (see kid_core::trace `PlayerLifecycle::Dying`
        // and `Respawning` branches mapping to GameMode::LevelTransition).
        // Keep the lightweight name/ID readout for those frames so the screen
        // is not blank.
        let trace = sample_trace(GameMode::LevelTransition);
        assert!(trace.active_objects.is_empty());
        let visuals = sample_level_visuals();
        let scene = build_scene(&trace, false, Some(&visuals), None);

        assert!(
            contains_text(&scene, "Blue Lake Woods 1"),
            "without title workers the transition scene should still show the level name",
        );
    }

    #[test]
    fn build_scene_clamps_floating_item_alpha_before_colour_scaling() {
        let mut trace = sample_trace(GameMode::InGame);
        trace.floating_items = vec![(48, 64, u8::MAX, 0x01)];
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
    fn build_scene_draws_floating_diamond_with_rom_sprite_art() {
        let mut trace = sample_trace(GameMode::InGame);
        trace.floating_items = vec![(48, 64, u8::MAX, 0x00)];
        let visuals = sample_level_visuals();

        let scene = build_scene(&trace, false, Some(&visuals), None);

        assert!(
            scene.commands.iter().any(|command| {
                matches!(
                    command,
                    DrawCommand::Sprite { tile_sheet, tiles, .. }
                        if tile_sheet.tiles.len() == visuals.hud.diamond_tiles.tiles.len()
                            && tiles.iter().any(|tile| tile.tile.tile_index == 8)
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
    fn build_scene_draws_floating_helmet_with_rom_mask_art() {
        let mut trace = sample_trace(GameMode::InGame);
        trace.floating_items = vec![(48, 64, u8::MAX, 0x02)];
        let visuals = sample_level_visuals();

        let scene = build_scene(&trace, false, Some(&visuals), None);

        assert!(
            scene.commands.iter().any(|command| {
                matches!(
                    command,
                    DrawCommand::Sprite { tile_sheet, .. }
                        if tile_sheet.tiles.iter().any(|tile| tile.pixels == [91; 64])
                )
            }),
            "ASM sub_10CB0/stru_106DC should render the Skycutter floating prize from mask art"
        );
        assert!(
            !scene.commands.iter().any(|command| {
                matches!(
                    command,
                    DrawCommand::Rect { colour, .. } if *colour == [200, 200, 255]
                )
            }),
            "floating helmet should not fall back to the old placeholder square"
        );
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
            trace.floating_items = vec![(48, 64, u8::MAX, content_id)];
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
    fn forest_ripple_starts_from_row_selector_not_row_index() {
        assert_eq!(
            super::scene::layered_background_shimmer_offset(0, 4, 0x10, Some(0x11)),
            0
        );
        assert_eq!(
            super::scene::layered_background_shimmer_offset(0, 4, 0x20, Some(0x11)),
            1
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
                        if *y < 48 && tile_sheet.tiles.len() == 0x200
                )
            })
            .count();

        assert!(hud_digits >= 2, "expected digit tiles in the HUD");
        assert!(hud_hitpoints >= 2, "expected hitpoint art tiles in the HUD");
        assert!(
            scene.commands.iter().any(|command| {
                matches!(
                    command,
                    DrawCommand::IndexedTile { y, tile_sheet, tile, .. }
                        if *y < 48
                            && tile_sheet.tiles.len() == 16
                            && tile.tile_index == 8
                )
            }),
            "expected the HUD diamond icon"
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
                    DrawCommand::IndexedTile { y, tile_sheet, tile, .. }
                        if *y < 48
                            && tile_sheet.tiles.len() == 16
                            && tile.tile_index == 8
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
    fn title_scene_stays_text_only() {
        let scene = build_scene(&sample_trace(GameMode::TitleScreen), true, None, None);
        assert!(contains_text(&scene, "KID CHAMELEON"));
        assert!(!scene.commands.iter().any(|command| {
            matches!(command, DrawCommand::Rect { colour, .. } if *colour == PLAYER_COLOUR || *colour == ENEMY_COLOUR || *colour == PLATFORM_COLOUR)
        }));
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
    fn try_load_block_grid_falls_back_to_decoded_block_layout_without_level_layout_dump() {
        let fixture = ManifestFixture::new().expect("manifest fixture");
        fs::remove_file(fixture._root.path().join("level-layout/00.bin")).expect("remove layout");
        fs::remove_file(fixture._root.path().join("level-layout/00.json"))
            .expect("remove layout metadata");

        let grid = try_load_block_grid(&fixture.manifest, 0x00).expect("fallback block grid");

        assert_eq!(grid.width_cells, 20);
        assert_eq!(grid.height_cells, 14);
        assert_eq!(grid.cell_at(0, 0).block_type, BlockType::Prize);
        assert_eq!(grid.cell_at(0, 0).skin, 0x05);
        assert_eq!(grid.cell_at(1, 0).collision, CollisionKind::Solid);
        assert_eq!(grid.cell_at(1, 0).skin, 0x07);
        assert_eq!(grid.cell_at(0, 0).collision, CollisionKind::Solid);
    }

    #[test]
    fn try_load_block_grid_resolves_decoded_layout_through_map_order() {
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

        let grid = try_load_block_grid(&fixture.manifest, 0x1B).expect("mapped block grid");

        assert_eq!(grid.width_cells, 20);
        assert_eq!(grid.height_cells, 14);
        assert_eq!(grid.cell_at(0, 0).collision, CollisionKind::Spikes);
        assert_eq!(grid.cell_at(0, 0).skin, 0x11);
        assert_eq!(grid.cell_at(1, 0).collision, CollisionKind::Solid);
        assert_eq!(grid.cell_at(1, 0).skin, 0x22);
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
    fn initialise_runtime_can_enter_settled_shortcut_level() {
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
            },
        );

        let trace = runtime.trace_frame();
        assert_eq!(trace.current_level.0, 0x00);
        assert_eq!(trace.game_mode, GameMode::InGame);
        assert!(trace.kid_y_pos.is_some());
    }

    #[test]
    fn save_native_recording_bundle_persists_current_trace_when_no_frames_were_stepped() {
        let fixture = ManifestFixture::new().expect("manifest fixture");
        let record_dir = fixture._root.path().join("recordings").join("freeze-repro");
        let trace = sample_trace(GameMode::LevelTransition);
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
            },
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
        assert!(scenario_text.contains("\"frame_count\": 1"));
        assert!(scenario_text.contains("\"script\": [\n    0\n  ]"));
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
            }
            other => panic!("expected body + turret sprite commands, got {other:?}"),
        }
    }

    #[test]
    fn rock_tank_renderer_uses_child_snapshot_instead_of_duplicate_synthetic_turret() {
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
                    tiles: body_tiles, ..
                },
                DrawCommand::Sprite {
                    tiles: turret_tiles,
                    ..
                },
            ] => {
                assert_eq!(body_tiles[0].tile.tile_index, 0x034);
                assert_eq!(
                    turret_tiles[0].tile.tile_index, 0x060,
                    "ASM Tank_Shooting.asm `stru_3B4D8` reaches `LnkTo_unk_C7F10` on the muzzle-open frame"
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
        (1..=40).contains(&signature)
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

    #[test]
    fn player_form_uses_iron_knight_sprite_art_when_helmet_equipped() {
        let mut trace = trace_with_player(720, 0);
        trace.current_helmet_id = PlayerForm::IronKnight.helmet_id();

        let scene = build_ingame_scene(&trace, false, None);

        assert_eq!(player_sprite_signature(&scene), Some(10));
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
            Some(39),
            "ASM off_A23C selects `unk_B7150` for Red Stealth's second slope animation slot instead of the generic walk frame"
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
            sprite_position(&scene, 1),
            Some((8, 24)),
            "ASM loc_B7FA/loc_B80C already advances the death tumble in engine state; the renderer must not add the old parabolic y-offset"
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
                            && tile_sheet.tiles.len() == 0x200
                            && [13, 15].contains(&tile.tile_index)
                )
            })
            .count();

        assert!(hud_digits >= 2, "HUD should show digit art");
        assert!(hitpoint_tiles >= 2, "HUD should show hitpoint art");
        assert!(!contains_text(&scene, "x3"));
        assert!(!contains_text(&scene, "D:7"));
    }

    #[test]
    fn game_hud_reflects_damage() {
        let mut trace = sample_trace(GameMode::InGame);
        trace.number_hitpoints = 1; // Took damage
        let scene = build_ingame_scene(&trace, false, None);

        let has_empty_pip = scene.commands.iter().any(|command| {
            matches!(
                command,
                DrawCommand::IndexedTile { y, tile_sheet, tile, .. }
                    if *y < 48
                        && tile_sheet.tiles.len() == 0x200
                        && tile.tile_index == 15
            )
        });
        assert!(
            has_empty_pip,
            "HUD should show empty hitpoint art after damage"
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
}
