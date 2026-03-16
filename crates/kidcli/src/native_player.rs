use crate::scenarios::GameplayCheckpoint;
use anyhow::{Result, anyhow, bail};
use font8x8::{BASIC_FONTS, UnicodeFonts};
use kid_core::{
    FrameInput, GameMode, LevelId, NativeObjectKind, NativeObjectSnapshot, NativeRuntime,
    NativeTraceFrame, PlayerForm, physics::Direction, terrain::BlockGrid,
};
use kid_data::{AssetManifest, LevelHeader};
use kid_render::{NativeLevelVisuals, Palette, TileRef, TileSheet, load_level_visuals};
use sdl2::audio::AudioQueue;
use sdl2::controller::{Button, GameController};
use sdl2::event::Event;
use sdl2::keyboard::{Keycode, Scancode};
use sdl2::pixels::PixelFormatEnum;
use sdl2::rect::Rect;
use sdl2::render::WindowCanvas;
use sdl2::video::FullscreenType;
use serde::Deserialize;
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};

const FRAME_WIDTH: u32 = 320;
const FRAME_HEIGHT: u32 = 224;
const FRAME_TIME: Duration = Duration::from_micros(16_667);
#[cfg(test)]
const PLAYER_COLOUR: [u8; 3] = [92, 214, 104];
const ENEMY_COLOUR: [u8; 3] = [220, 90, 88];
const ENEMY_FLASH_COLOUR: [u8; 3] = [255, 255, 255];

/// Per-family enemy rect colour so different types are visually distinguishable.
fn enemy_family_colour(object_type: u8) -> [u8; 3] {
    match object_type {
        0x00 => [255, 80, 40],   // FireDemon: orange-red
        0x03 => [160, 160, 180], // Robot: steel grey
        0x04 => [180, 140, 80],  // Armadillo: tan
        0x05 => [100, 60, 120],  // TarMonster: dark purple
        0x06 => [60, 180, 220],  // Sphere: cyan
        0x07 => [120, 160, 60],  // Archer: olive
        0x08 => [40, 100, 180],  // Orca: deep blue
        0x09 => [220, 120, 60],  // Crab: orange
        0x0A => [100, 120, 100], // Tank: military green
        0x0E => [200, 200, 220], // Cloud: pale grey
        0x0F => [140, 200, 140], // UFO: light green
        0x10 => [180, 160, 120], // Goat: brown
        0x11 => [80, 40, 100],   // Ninja: dark purple
        0x12 => [200, 160, 60],  // Lion: gold
        0x13 => [160, 80, 80],   // Scorpion: dark red
        0x14 => [180, 100, 200], // SpinningTwins: magenta
        0x16 => [100, 180, 180], // Drips: teal
        0x17 => [180, 140, 140], // Hand: flesh
        0x18 => [160, 200, 240], // Tornado: light blue
        0x19 => [255, 120, 40],  // Fireball: bright orange
        0x1A => [140, 100, 60],  // Driller: brown
        0x1B => [120, 120, 140], // EmoRock: grey-blue
        0x1D => [200, 180, 160], // BigHoppingSkull: bone
        0x20 => [255, 60, 60],   // HeadyMetal: bright red (boss)
        0x22 => [255, 100, 60],  // Shishkaboss: boss orange
        0x23 => [255, 80, 120],  // BoomerangBosses: boss pink
        0x24 => [255, 120, 80],  // BagelBrothers: boss coral
        _ => ENEMY_COLOUR,
    }
}
#[cfg(test)]
const PLATFORM_COLOUR: [u8; 3] = [232, 176, 64];
#[cfg(test)]
const GUIDE_COLOUR: [u8; 3] = [62, 84, 108];
const SKY_COLOUR: [u8; 3] = [28, 42, 56];
const PALETTE_LINE_COLOUR_COUNT: usize = 15;

#[derive(Debug, Clone, Copy)]
pub struct NativePlayOptions {
    pub fullscreen: bool,
    pub scale: Option<u32>,
    pub debug_overlay: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Scene {
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

pub fn run_native_player(manifest: &AssetManifest, options: NativePlayOptions) -> Result<()> {
    let mut session_assets = load_native_session_assets(manifest, 0x00)?;
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

    let audio_queue: Option<AudioQueue<f32>> = sdl
        .audio()
        .ok()
        .and_then(|audio| {
            let desired = sdl2::audio::AudioSpecDesired {
                freq: Some(44_100),
                channels: Some(1),
                samples: Some(1024),
            };
            audio.open_queue::<f32, _>(None, &desired).ok()
        });
    if let Some(ref aq) = audio_queue {
        aq.resume();
    }
    let mut sfx_engine = SfxEngine::new();

    let mut runtime = initialise_runtime(manifest, &session_assets);
    let mut trace = runtime.trace_frame();
    #[allow(unused_assignments)]
    let mut prev_trace = trace.clone();
    let mut debug_overlay_visible = options.debug_overlay;
    let mut frame = vec![0; (FRAME_WIDTH * FRAME_HEIGHT * 3) as usize];
    let mut current_level_id: u8 = 0x00;

    let scene = build_scene(
        &trace,
        debug_overlay_visible,
        Some(&session_assets.level_visuals),
        runtime.block_grid(),
    );
    render_scene(&scene, &mut frame);
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

        for event in events.poll_iter() {
            match event {
                Event::Quit { .. } => break 'running,
                Event::ControllerDeviceAdded { .. } => {
                    if controller.is_none() {
                        controller = open_first_controller(&game_controller)?;
                    }
                }
                Event::ControllerDeviceRemoved { .. } => {
                    controller = controller.filter(|gamepad| gamepad.attached());
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
                    keycode: Some(Keycode::R),
                    repeat: false,
                    ..
                } => reset_runtime = true,
                // Debug: N = next level, P = previous level
                Event::KeyDown {
                    keycode: Some(Keycode::N),
                    repeat: false,
                    ..
                } => {
                    let next = runtime.next_level_id();
                    runtime.warp_to_level(next);
                }
                Event::KeyDown {
                    keycode: Some(Keycode::P),
                    repeat: false,
                    ..
                } => {
                    let prev = runtime.prev_level_id();
                    runtime.warp_to_level(prev);
                }
                _ => {}
            }
        }

        if toggle_fullscreen {
            fullscreen = !fullscreen;
            apply_window_mode(&mut canvas, &video, fullscreen, options.scale)?;
        }

        if reset_runtime {
            session_assets = load_native_session_assets(manifest, 0x00)?;
            runtime = initialise_runtime(manifest, &session_assets);
            trace = runtime.trace_frame();
            current_level_id = 0x00;
        } else {
            let buttons = keyboard_buttons(&events) | controller_buttons(controller.as_ref());
            prev_trace = trace.clone();
            trace = runtime.step_frame(FrameInput::from_mask(buttons));

            // Detect game events by comparing frames
            detect_sfx_events(&prev_trace, &trace, &mut sfx_engine);
        }

        // Push audio samples to SDL2 queue
        if let Some(ref aq) = audio_queue {
            if aq.size() < 4096 {
                let samples = sfx_engine.generate(735); // ~1 frame at 44100 Hz
                let _ = aq.queue_audio(&samples);
            }
        }

        if trace.current_level.0 != current_level_id {
            let new_level = trace.current_level.0;
            match load_native_session_assets(manifest, new_level) {
                Ok(new_assets) => {
                    session_assets = new_assets;
                    current_level_id = new_level;
                }
                Err(error) => {
                    eprintln!(
                        "failed to load visuals for level 0x{new_level:02X}: {error:#}"
                    );
                }
            }
        }

        let scene = build_scene(
            &trace,
            debug_overlay_visible,
            Some(&session_assets.level_visuals),
            runtime.block_grid(),
        );
        render_scene(&scene, &mut frame);
        texture
            .update(None, &frame, (FRAME_WIDTH as usize) * 3)
            .map_err(|error| anyhow!("failed to update SDL texture: {error}"))?;
        canvas.clear();
        canvas
            .copy(&texture, None, None)
            .map_err(|error| anyhow!("failed to copy SDL texture: {error}"))?;
        canvas.present();

        let elapsed = frame_started.elapsed();
        if elapsed < FRAME_TIME {
            thread::sleep(FRAME_TIME - elapsed);
        }
    }

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
) -> NativeRuntime {
    let mut runtime = NativeRuntime::new(manifest.clone());
    runtime.set_block_grid(session_assets.block_grid.clone());
    runtime.enter_level_playtest(LevelId(0));
    runtime
}

fn build_scene(
    trace: &NativeTraceFrame,
    debug_overlay_visible: bool,
    level_visuals: Option<&NativeLevelVisuals>,
    block_grid: Option<&BlockGrid>,
) -> Scene {
    match trace.game_mode {
        GameMode::InGame => build_game_scene(
            trace,
            debug_overlay_visible,
            level_visuals.expect("Level 1 visuals are required for gameplay scenes"),
            block_grid,
        ),
        GameMode::LevelTransition => build_transition_scene(trace),
        _ => build_status_scene(trace, debug_overlay_visible),
    }
}

fn build_transition_scene(trace: &NativeTraceFrame) -> Scene {
    let mut commands = Vec::new();
    commands.push(DrawCommand::Rect {
        x: 0,
        y: 0,
        width: FRAME_WIDTH as i32,
        height: FRAME_HEIGHT as i32,
        colour: [0, 0, 0],
    });
    let level_text = format!(
        "{}",
        trace
            .current_level_name
            .as_deref()
            .unwrap_or(&format!("Level {:02X}", trace.current_level.0))
    );
    let text_w = level_text.len() as i32 * 8 * 2;
    let text_x = (FRAME_WIDTH as i32 - text_w) / 2;
    let text_y = (FRAME_HEIGHT as i32) / 2 - 8;
    commands.push(text(text_x, text_y, &level_text, [255, 255, 255], 2));
    Scene {
        background: [0, 0, 0],
        commands,
    }
}

fn build_status_scene(trace: &NativeTraceFrame, debug_overlay_visible: bool) -> Scene {
    let mut commands = Vec::new();

    if trace.game_complete {
        return build_victory_scene(trace);
    }

    // Background
    commands.push(DrawCommand::Rect {
        x: 0,
        y: 0,
        width: FRAME_WIDTH as i32,
        height: FRAME_HEIGHT as i32,
        colour: [8, 12, 20],
    });

    // Title
    let title = "KID CHAMELEON";
    let title_w = title.len() as i32 * 8 * 3;
    commands.push(text(
        (FRAME_WIDTH as i32 - title_w) / 2,
        40,
        title,
        [255, 220, 80],
        3,
    ));
    commands.push(text(
        (FRAME_WIDTH as i32 - 15 * 8) / 2,
        72,
        "Native Playtest",
        [140, 180, 255],
        1,
    ));

    // Blinking "PRESS START" (blink every 30 frames)
    if (trace.frame_index / 30) % 2 == 0 {
        let start_text = "PRESS START";
        let start_w = start_text.len() as i32 * 8 * 2;
        commands.push(text(
            (FRAME_WIDTH as i32 - start_w) / 2,
            120,
            start_text,
            [255, 255, 255],
            2,
        ));
    }

    // Controls help
    commands.push(text(
        20,
        160,
        "Arrows/WASD move  Z/Space jump  X special  C/Shift run",
        [140, 140, 140],
        1,
    ));
    commands.push(text(
        20,
        174,
        "R restart level  Tab debug overlay  F11 fullscreen  Esc quit",
        [140, 140, 140],
        1,
    ));
    if debug_overlay_visible {
        commands.push(text(
            30,
            160,
            &format!(
                "Camera {:08X},{:08X}",
                trace.camera_x_pos, trace.camera_y_pos
            ),
            [160, 214, 255],
            1,
        ));
    }

    Scene {
        background: [18, 24, 32],
        commands,
    }
}

fn build_victory_scene(trace: &NativeTraceFrame) -> Scene {
    let mut commands = Vec::new();

    // Dark green background
    commands.push(DrawCommand::Rect {
        x: 0,
        y: 0,
        width: FRAME_WIDTH as i32,
        height: FRAME_HEIGHT as i32,
        colour: [4, 20, 8],
    });

    let title = "CONGRATULATIONS";
    let title_w = title.len() as i32 * 8 * 2;
    commands.push(text(
        (FRAME_WIDTH as i32 - title_w) / 2,
        40,
        title,
        [255, 220, 80],
        2,
    ));

    let sub = "YOU HAVE COMPLETED";
    let sub_w = sub.len() as i32 * 8;
    commands.push(text(
        (FRAME_WIDTH as i32 - sub_w) / 2,
        72,
        sub,
        [180, 220, 180],
        1,
    ));

    let game = "KID CHAMELEON";
    let game_w = game.len() as i32 * 8 * 2;
    commands.push(text(
        (FRAME_WIDTH as i32 - game_w) / 2,
        92,
        game,
        [120, 255, 120],
        2,
    ));

    let score_line = format!("FINAL SCORE: {}", trace.score);
    let score_w = score_line.len() as i32 * 8;
    commands.push(text(
        (FRAME_WIDTH as i32 - score_w) / 2,
        130,
        &score_line,
        [255, 255, 255],
        1,
    ));

    // Blinking "PRESS START" to return to title
    if (trace.frame_index / 30) % 2 == 0 {
        let start_text = "PRESS START";
        let start_w = start_text.len() as i32 * 8 * 2;
        commands.push(text(
            (FRAME_WIDTH as i32 - start_w) / 2,
            170,
            start_text,
            [255, 255, 255],
            2,
        ));
    }

    Scene {
        background: [8, 32, 16],
        commands,
    }
}

fn resolve_ingame_vram_tile(
    level_visuals: &NativeLevelVisuals,
    tile_ref: TileRef,
) -> Option<ResolvedSceneTile> {
    let foreground_range_end =
        FOREGROUND_TILE_VRAM_BASE + level_visuals.foreground_tiles.tiles.len() as u16;
    if (FOREGROUND_TILE_VRAM_BASE..foreground_range_end).contains(&tile_ref.tile_index) {
        return Some(ResolvedSceneTile {
            tile_sheet: Arc::new(level_visuals.foreground_tiles.clone()),
            tile: TileRef {
                tile_index: tile_ref.tile_index - FOREGROUND_TILE_VRAM_BASE,
                ..tile_ref
            },
            palette: Arc::new(level_visuals.palettes.foreground.clone()),
        });
    }

    let block_range_end = BLOCK_TILE_VRAM_BASE + level_visuals.block_tiles.tiles.len() as u16;
    if (BLOCK_TILE_VRAM_BASE..block_range_end).contains(&tile_ref.tile_index) {
        return Some(ResolvedSceneTile {
            tile_sheet: Arc::new(level_visuals.block_tiles.clone()),
            tile: TileRef {
                tile_index: tile_ref.tile_index - BLOCK_TILE_VRAM_BASE,
                ..tile_ref
            },
            palette: Arc::new(level_visuals.palettes.foreground.clone()),
        });
    }

    let hud_range_end = HUD_TILE_VRAM_BASE + level_visuals.hud.digits.tiles.len() as u16;
    if (HUD_TILE_VRAM_BASE..hud_range_end).contains(&tile_ref.tile_index) {
        return Some(ResolvedSceneTile {
            tile_sheet: Arc::new(level_visuals.hud.digits.clone()),
            tile: TileRef {
                tile_index: tile_ref.tile_index - HUD_TILE_VRAM_BASE,
                ..tile_ref
            },
            palette: Arc::new(level_visuals.palettes.foreground.clone()),
        });
    }

    let hud_diamond_range_end =
        HUD_DIAMOND_TILE_VRAM_BASE + level_visuals.hud.diamond_tiles.tiles.len() as u16;
    if (HUD_DIAMOND_TILE_VRAM_BASE..hud_diamond_range_end).contains(&tile_ref.tile_index) {
        return Some(ResolvedSceneTile {
            tile_sheet: Arc::new(level_visuals.hud.diamond_tiles.clone()),
            tile: TileRef {
                tile_index: tile_ref.tile_index - HUD_DIAMOND_TILE_VRAM_BASE,
                ..tile_ref
            },
            palette: Arc::new(level_visuals.palettes.foreground.clone()),
        });
    }

    None
}

fn push_resolved_tile(commands: &mut Vec<DrawCommand>, x: i32, y: i32, tile: ResolvedSceneTile) {
    commands.push(DrawCommand::IndexedTile {
        x,
        y,
        tile_sheet: tile.tile_sheet,
        tile: tile.tile,
        palette: tile.palette,
    });
}

fn push_background_tiles(
    commands: &mut Vec<DrawCommand>,
    trace: &NativeTraceFrame,
    level_visuals: &NativeLevelVisuals,
) {
    let camera_x_px = (trace.camera_x_pos >> 16) as i32;
    let camera_y_px = (trace.camera_y_pos >> 16) as i32;
    let tile_sheet = Arc::new(level_visuals.background_tiles.clone());
    let palette = Arc::new(level_visuals.palettes.background.clone());

    match &level_visuals.background {
        kid_render::BackgroundVisuals::Chunked(background) => {
            for placement in &background.placements {
                let Some(chunk) = background.chunks.get(usize::from(placement.chunk_id)) else {
                    continue;
                };
                for row in 0..chunk.height_tiles {
                    for column in 0..chunk.width_tiles {
                        let tile_index =
                            usize::from(row) * usize::from(chunk.width_tiles) + usize::from(column);
                        let Some(background_tile) = chunk.tiles.get(tile_index) else {
                            continue;
                        };
                        let (screen_x, screen_y) = background_tile_screen_position(
                            trace,
                            level_visuals,
                            placement.x_tiles + column,
                            placement.y_tiles + row,
                            camera_x_px,
                            camera_y_px,
                        );
                        if screen_x <= -8
                            || screen_y <= -8
                            || screen_x >= FRAME_WIDTH as i32
                            || screen_y >= FRAME_HEIGHT as i32
                        {
                            continue;
                        }
                        commands.push(DrawCommand::IndexedTile {
                            x: screen_x,
                            y: screen_y,
                            tile_sheet: tile_sheet.clone(),
                            tile: TileRef {
                                tile_index: background_tile.tile_index,
                                palette_line: 0,
                                priority: false,
                                flip_x: background_tile.flip_x,
                                flip_y: false,
                            },
                            palette: palette.clone(),
                        });
                    }
                }
            }
        }
        kid_render::BackgroundVisuals::Layered(background) => {
            for (row_index, row) in background.rows.iter().enumerate() {
                for (column, tile) in row.tiles.iter().enumerate() {
                    let (screen_x, screen_y) = background_tile_screen_position(
                        trace,
                        level_visuals,
                        column as u16,
                        row_index as u16,
                        camera_x_px,
                        camera_y_px,
                    );
                    if screen_x <= -8
                        || screen_y <= -8
                        || screen_x >= FRAME_WIDTH as i32
                        || screen_y >= FRAME_HEIGHT as i32
                    {
                        continue;
                    }
                    commands.push(DrawCommand::IndexedTile {
                        x: screen_x,
                        y: screen_y,
                        tile_sheet: tile_sheet.clone(),
                        tile: *tile,
                        palette: palette.clone(),
                    });
                }
            }
        }
    }
}

fn background_tile_screen_position(
    trace: &NativeTraceFrame,
    level_visuals: &NativeLevelVisuals,
    x_tiles: u16,
    y_tiles: u16,
    camera_x_px: i32,
    camera_y_px: i32,
) -> (i32, i32) {
    let world_x_px = i32::from(x_tiles) * 8;
    let world_y_px = i32::from(y_tiles) * 8;
    let Some(profile) = level_visuals.level.background_scroll.as_ref() else {
        return (
            world_x_px.saturating_sub(camera_x_px),
            world_y_px.saturating_sub(camera_y_px),
        );
    };

    let layer_speed = profile
        .layer_speeds
        .get(usize::from(y_tiles))
        .copied()
        .unwrap_or(0x10);
    (
        world_x_px.saturating_sub(background_scroll_x_px(trace, camera_x_px, layer_speed)),
        world_y_px,
    )
}

fn background_scroll_x_px(trace: &NativeTraceFrame, camera_x_px: i32, layer_speed: u8) -> i32 {
    match layer_speed {
        0x00 => (trace.frame_index as i32 / 2).rem_euclid(512),
        0x08 => (trace.frame_index as i32 / 4).rem_euclid(512),
        0x10 => 0,
        0x18..=0x88 if (layer_speed - 0x10) % 8 == 0 => {
            let step = i32::from((layer_speed - 0x10) / 8);
            camera_x_px.saturating_mul(step) / 15
        }
        _ => camera_x_px,
    }
}

fn push_tile_quad(
    commands: &mut Vec<DrawCommand>,
    level_visuals: &NativeLevelVisuals,
    x: i32,
    y: i32,
    quad: &kid_render::TileQuad,
) {
    for (index, tile_ref) in quad.tiles.iter().copied().enumerate() {
        let tile_x = x + if index % 2 == 0 { 0 } else { 8 };
        let tile_y = y + if index < 2 { 0 } else { 8 };
        if let Some(tile) = resolve_ingame_vram_tile(level_visuals, tile_ref) {
            push_resolved_tile(commands, tile_x, tile_y, tile);
        }
    }
}

fn push_level_layout_terrain(
    commands: &mut Vec<DrawCommand>,
    trace: &NativeTraceFrame,
    level_visuals: &NativeLevelVisuals,
    block_grid: Option<&BlockGrid>,
) {
    let camera_x_px = (trace.camera_x_pos >> 16) as i32;
    let camera_y_px = (trace.camera_y_pos >> 16) as i32;
    let Some(grid) = block_grid else {
        return;
    };

    let col_start = (camera_x_px / 16).max(0) as u16;
    let col_end =
        ((camera_x_px + FRAME_WIDTH as i32 + 15) / 16).min(i32::from(grid.width_cells)) as u16;
    let row_start = (camera_y_px / 16).max(0) as u16;
    let row_end =
        ((camera_y_px + FRAME_HEIGHT as i32 + 15) / 16).min(i32::from(grid.height_cells)) as u16;

    for row in row_start..row_end {
        for col in col_start..col_end {
            let cell = grid.cell_at(col, row);
            let Some(resolved) = level_visuals.level.block_mapping.resolve(cell.skin, None) else {
                continue;
            };
            push_tile_quad(
                commands,
                level_visuals,
                i32::from(col) * 16 - camera_x_px,
                i32::from(row) * 16 - camera_y_px,
                resolved.quad,
            );
        }
    }
}

fn push_block_grid_overrides(
    commands: &mut Vec<DrawCommand>,
    trace: &NativeTraceFrame,
    level_visuals: &NativeLevelVisuals,
    block_grid: Option<&BlockGrid>,
) {
    let Some(grid) = block_grid else { return };

    let camera_x_px = (trace.camera_x_pos >> 16) as i32;
    let camera_y_px = (trace.camera_y_pos >> 16) as i32;

    // Determine visible cell range (with 1-cell margin for partially visible cells)
    let col_start = (camera_x_px / 16).max(0) as u16;
    let col_end =
        ((camera_x_px + FRAME_WIDTH as i32 + 15) / 16).min(i32::from(grid.width_cells)) as u16;
    let row_start = (camera_y_px / 16).max(0) as u16;
    let row_end =
        ((camera_y_px + FRAME_HEIGHT as i32 + 15) / 16).min(i32::from(grid.height_cells)) as u16;

    for row in row_start..row_end {
        for col in col_start..col_end {
            let cell = grid.cell_at(col, row);
            if !cell.has_block {
                continue;
            }

            let Some(resolved) = level_visuals
                .level
                .block_mapping
                .resolve(cell.skin, Some(cell.block_type as u8))
            else {
                continue;
            };
            push_tile_quad(
                commands,
                level_visuals,
                i32::from(col) * 16 - camera_x_px,
                i32::from(row) * 16 - camera_y_px,
                resolved.quad,
            );
        }
    }
}

const HUD_BG: [u8; 3] = [20, 20, 30];

fn push_hud_number(
    commands: &mut Vec<DrawCommand>,
    level_visuals: &NativeLevelVisuals,
    value: u16,
    x: i32,
    y: i32,
) {
    for (index, digit) in value.to_string().chars().enumerate() {
        let Some(digit_value) = digit.to_digit(10) else {
            continue;
        };
        let Some(tile) = resolve_ingame_vram_tile(
            level_visuals,
            TileRef {
                tile_index: level_visuals.hud.digit_base_tile_index + digit_value as u16,
                palette_line: 0,
                priority: false,
                flip_x: false,
                flip_y: false,
            },
        ) else {
            continue;
        };
        push_resolved_tile(commands, x + (index as i32 * 8), y, tile);
    }
}

fn hud_counter_width(value: u16) -> i32 {
    let digit_count = value.to_string().len() as i32;
    if digit_count == 1 {
        16
    } else {
        digit_count * 8
    }
}

fn push_hud_counter(
    commands: &mut Vec<DrawCommand>,
    level_visuals: &NativeLevelVisuals,
    value: u16,
    x: i32,
    y: i32,
) {
    push_hud_number(commands, level_visuals, value, x, y);
    if value < 10
        && let Some(tile) = resolve_ingame_vram_tile(
            level_visuals,
            TileRef {
                tile_index: level_visuals.hud.digit_base_tile_index + 11,
                palette_line: 0,
                priority: false,
                flip_x: false,
                flip_y: false,
            },
        )
    {
        push_resolved_tile(commands, x + 8, y, tile);
    }
}

fn push_hud_score(
    commands: &mut Vec<DrawCommand>,
    level_visuals: &NativeLevelVisuals,
    score: u32,
    x: i32,
    y: i32,
) {
    for (index, digit) in score.to_string().chars().enumerate() {
        let Some(digit_value) = digit.to_digit(10) else {
            continue;
        };
        let Some(tile) = resolve_ingame_vram_tile(
            level_visuals,
            TileRef {
                tile_index: level_visuals.hud.digit_base_tile_index + digit_value as u16,
                palette_line: 0,
                priority: false,
                flip_x: false,
                flip_y: false,
            },
        ) else {
            continue;
        };
        push_resolved_tile(commands, x + (index as i32 * 8), y, tile);
    }
}

fn push_hud_icon(
    commands: &mut Vec<DrawCommand>,
    level_visuals: &NativeLevelVisuals,
    icon_name: &str,
    x: i32,
    y: i32,
) {
    if let Some(icon) = level_visuals
        .hud
        .icons
        .iter()
        .find(|icon| icon.name == icon_name)
        && let Some(tile) = resolve_ingame_vram_tile(level_visuals, icon.tile)
    {
        push_resolved_tile(commands, x, y, tile);
    }
}

fn push_game_hud(
    commands: &mut Vec<DrawCommand>,
    trace: &NativeTraceFrame,
    level_visuals: &NativeLevelVisuals,
) {
    let hud_y = (FRAME_HEIGHT - 18) as i32;
    let hud_h = 18;

    commands.push(DrawCommand::Rect {
        x: 0,
        y: hud_y,
        width: FRAME_WIDTH as i32,
        height: hud_h,
        colour: HUD_BG,
    });

    let max_hp = PlayerForm::from_helmet_id(trace.current_helmet_id)
        .map(PlayerForm::max_hitpoints)
        .unwrap_or_else(|| PlayerForm::TheKid.max_hitpoints()) as i32;
    let pip_gap = 2;
    let pip_y = hud_y + 5;
    for i in 0..max_hp {
        let pip_tile = if (i as u16) < trace.number_hitpoints {
            level_visuals.hud.hitpoint_tiles[0]
        } else {
            level_visuals.hud.hitpoint_tiles[2]
        };
        if let Some(tile) = resolve_ingame_vram_tile(level_visuals, pip_tile) {
            push_resolved_tile(commands, 4 + i * (8 + pip_gap), pip_y, tile);
        }
    }

    let lives_x = 4 + max_hp * (8 + pip_gap) + 8;
    push_hud_counter(commands, level_visuals, trace.number_lives, lives_x, pip_y);
    push_hud_icon(commands, level_visuals, "ankh", lives_x + 18, pip_y);

    // Level timer: M:SS in centre of HUD
    let minutes = trace.level_timer_seconds / 60;
    let seconds = trace.level_timer_seconds % 60;
    let timer_x = (FRAME_WIDTH as i32) / 2 - 16;
    push_hud_number(commands, level_visuals, minutes, timer_x, pip_y);
    // Colon character: use digit tile index + 11 (separator glyph)
    if let Some(tile) = resolve_ingame_vram_tile(
        level_visuals,
        TileRef {
            tile_index: level_visuals.hud.digit_base_tile_index + 11,
            palette_line: 0,
            priority: false,
            flip_x: false,
            flip_y: false,
        },
    ) {
        push_resolved_tile(commands, timer_x + 8, pip_y, tile);
    }
    // Two-digit seconds: zero-pad by rendering tens then ones
    push_hud_number(commands, level_visuals, seconds / 10, timer_x + 16, pip_y);
    push_hud_number(commands, level_visuals, seconds % 10, timer_x + 24, pip_y);

    // Score display right of timer
    let score_x = timer_x + 40;
    push_hud_score(commands, level_visuals, trace.score, score_x, pip_y);

    // Helmet name if wearing one
    let form = PlayerForm::from_helmet_id(trace.current_helmet_id);
    if let Some(f) = form {
        if f != PlayerForm::TheKid {
            let name = helmet_display_name(f);
            let name_x = lives_x + 30;
            commands.push(text(name_x, pip_y, name, [180, 220, 255], 1));
        }
    }

    let diamond_icon_x = FRAME_WIDTH as i32 - 12;
    let diamond_counter_x = diamond_icon_x - 2 - hud_counter_width(trace.diamond_count);
    push_hud_counter(
        commands,
        level_visuals,
        trace.diamond_count,
        diamond_counter_x,
        pip_y,
    );
    push_hud_icon(commands, level_visuals, "diamond", diamond_icon_x, pip_y);
}

fn helmet_display_name(form: PlayerForm) -> &'static str {
    match form {
        PlayerForm::TheKid => "KID",
        PlayerForm::Skycutter => "SKY",
        PlayerForm::Cyclone => "CYC",
        PlayerForm::RedStealth => "RED",
        PlayerForm::Eyeclops => "EYE",
        PlayerForm::Juggernaut => "JUG",
        PlayerForm::IronKnight => "IRN",
        PlayerForm::Berzerker => "BRZ",
        PlayerForm::Maniaxe => "MNX",
        PlayerForm::Micromax => "MCR",
    }
}

fn build_player_sprite_tiles(
    frame: &kid_render::SpriteFrame,
    facing: Direction,
) -> Vec<SpriteTile> {
    build_sprite_tiles(frame, facing, 0)
}

fn align_sprite_dimension(px: u16) -> u16 {
    px.div_ceil(8) * 8
}

fn sprite_draw_position(
    frame: &kid_render::SpriteFrame,
    screen_x: i32,
    screen_y: i32,
    facing: Direction,
) -> (i32, i32) {
    let draw_x = match facing {
        Direction::Right => screen_x - i32::from(frame.origin_x),
        Direction::Left => {
            screen_x + i32::from(frame.origin_x) - i32::from(align_sprite_dimension(frame.width_px))
        }
    };
    (draw_x, screen_y - i32::from(frame.origin_y))
}

fn build_sprite_tiles(
    frame: &kid_render::SpriteFrame,
    facing: Direction,
    palette_line: u8,
) -> Vec<SpriteTile> {
    let flip_x = matches!(facing, Direction::Left);
    let aligned_width_px = align_sprite_dimension(frame.width_px);
    let aligned_height_px = align_sprite_dimension(frame.height_px);
    let mut tiles =
        Vec::with_capacity(usize::from(aligned_width_px / 8) * usize::from(aligned_height_px / 8));
    let mut next_tile_index = frame.tile_base_index;
    let mut band_y_px = 0u16;
    let mut remaining_height_px = frame.height_px;

    while remaining_height_px >= 32 {
        append_sprite_band_tiles(
            &mut tiles,
            &mut next_tile_index,
            band_y_px,
            32,
            frame.width_px,
            aligned_width_px,
            flip_x,
            palette_line,
        );
        band_y_px += 32;
        remaining_height_px -= 32;
    }

    let tail_height_px = align_sprite_dimension(remaining_height_px);
    if tail_height_px > 0 {
        append_sprite_band_tiles(
            &mut tiles,
            &mut next_tile_index,
            band_y_px,
            tail_height_px,
            frame.width_px,
            aligned_width_px,
            flip_x,
            palette_line,
        );
    }

    tiles
}

fn append_sprite_band_tiles(
    tiles: &mut Vec<SpriteTile>,
    next_tile_index: &mut u16,
    band_y_px: u16,
    band_height_px: u16,
    frame_width_px: u16,
    aligned_width_px: u16,
    flip_x: bool,
    palette_line: u8,
) {
    let mut remaining_width_px = frame_width_px;
    let mut chunk_x_px = 0u16;

    while remaining_width_px >= 32 {
        append_sprite_chunk_tiles(
            tiles,
            next_tile_index,
            chunk_x_px,
            band_y_px,
            32,
            band_height_px,
            aligned_width_px,
            flip_x,
            palette_line,
        );
        chunk_x_px += 32;
        remaining_width_px -= 32;
    }

    let tail_width_px = align_sprite_dimension(remaining_width_px);
    if tail_width_px > 0 {
        append_sprite_chunk_tiles(
            tiles,
            next_tile_index,
            chunk_x_px,
            band_y_px,
            tail_width_px,
            band_height_px,
            aligned_width_px,
            flip_x,
            palette_line,
        );
    }
}

fn append_sprite_chunk_tiles(
    tiles: &mut Vec<SpriteTile>,
    next_tile_index: &mut u16,
    chunk_x_px: u16,
    chunk_y_px: u16,
    chunk_width_px: u16,
    chunk_height_px: u16,
    aligned_width_px: u16,
    flip_x: bool,
    palette_line: u8,
) {
    let columns = chunk_width_px / 8;
    let rows = chunk_height_px / 8;

    for row in 0..rows {
        for column in 0..columns {
            let unflipped_x_px = chunk_x_px + column * 8;
            let offset_x = if flip_x {
                i32::from(aligned_width_px - unflipped_x_px - 8)
            } else {
                i32::from(unflipped_x_px)
            };
            tiles.push(SpriteTile {
                offset_x,
                offset_y: i32::from(chunk_y_px + row * 8),
                tile: TileRef {
                    tile_index: *next_tile_index,
                    palette_line,
                    priority: false,
                    flip_x,
                    flip_y: false,
                },
            });
            *next_tile_index += 1;
        }
    }
}

fn select_player_frame<'a>(
    trace: &NativeTraceFrame,
    player_visuals: &'a kid_render::PlayerVisuals,
) -> &'a kid_render::SpriteFrame {
    let on_ground = trace.kid_on_ground.unwrap_or(true);
    let x_velocity = trace.kid_x_velocity.unwrap_or(0);
    let y_velocity = trace.kid_y_velocity.unwrap_or(0);

    if on_ground {
        if x_velocity == 0 {
            &player_visuals.idle
        } else {
            let frame_index = (trace.frame_index / 4) % player_visuals.walk.len();
            &player_visuals.walk[frame_index]
        }
    } else if y_velocity < 0 {
        &player_visuals.jump_rise
    } else {
        &player_visuals.jump_fall
    }
}

fn player_screen_position(object: &NativeObjectSnapshot, trace: &NativeTraceFrame) -> (i32, i32) {
    (
        (object.x_pos >> 16) as i32 - (trace.camera_x_pos >> 16) as i32,
        (object.y_pos >> 16) as i32 - (trace.camera_y_pos >> 16) as i32,
    )
}

fn push_player_sprite(
    commands: &mut Vec<DrawCommand>,
    object: &NativeObjectSnapshot,
    trace: &NativeTraceFrame,
    level_visuals: &NativeLevelVisuals,
) {
    let frame = select_player_frame(trace, &level_visuals.player);
    let facing = trace.kid_facing.unwrap_or(Direction::Right);
    let (screen_x, screen_y) = player_screen_position(object, trace);
    let (draw_x, draw_y) = sprite_draw_position(frame, screen_x, screen_y, facing);

    commands.push(DrawCommand::Sprite {
        x: draw_x,
        y: draw_y,
        tile_sheet: Arc::new(frame.tile_sheet.clone()),
        palette: Arc::new(level_visuals.player.palette.clone()),
        tiles: build_player_sprite_tiles(frame, facing),
    });
}

fn select_enemy_frame<'a>(
    trace: &NativeTraceFrame,
    enemy_visuals: &'a kid_render::AnimatedSpriteVisual,
) -> &'a kid_render::SpriteFrame {
    let frame_index =
        (trace.frame_index / enemy_visuals.frame_duration.max(1)) % enemy_visuals.frames.len();
    &enemy_visuals.frames[frame_index]
}

fn push_enemy_sprite(
    commands: &mut Vec<DrawCommand>,
    object: &NativeObjectSnapshot,
    trace: &NativeTraceFrame,
    level_visuals: &NativeLevelVisuals,
    object_type: u8,
) {
    let enemy_visuals = match object_type {
        0x01 => &level_visuals.enemies.diamond,
        0x0C | 0x0D => &level_visuals.enemies.dragon,
        0x1C => &level_visuals.enemies.mini_hopping_skull,
        _ => {
            let colour = if object.damage_flash {
                ENEMY_FLASH_COLOUR
            } else {
                enemy_family_colour(object_type)
            };
            commands.push(object_rect(object, trace, 16, 16, colour));
            return;
        }
    };

    let frame = select_enemy_frame(trace, enemy_visuals);
    let (screen_x, screen_y) = player_screen_position(object, trace);
    let facing = object.facing.unwrap_or(Direction::Right);
    let (draw_x, draw_y) = sprite_draw_position(frame, screen_x, screen_y, facing);
    commands.push(DrawCommand::Sprite {
        x: draw_x,
        y: draw_y,
        tile_sheet: Arc::new(frame.tile_sheet.clone()),
        palette: Arc::new(enemy_visuals.palette.clone()),
        tiles: build_sprite_tiles(frame, facing, 0),
    });
}

fn push_platform_tiles(
    commands: &mut Vec<DrawCommand>,
    object: &NativeObjectSnapshot,
    trace: &NativeTraceFrame,
    level_visuals: &NativeLevelVisuals,
    kind: u8,
) {
    let frame = match kind {
        0x00 => &level_visuals.platforms.horizontal,
        _ => &level_visuals.platforms.vertical,
    };
    let (screen_x, screen_y) = player_screen_position(object, trace);
    commands.push(DrawCommand::Sprite {
        x: screen_x - i32::from(frame.origin_x),
        y: screen_y - i32::from(frame.origin_y),
        tile_sheet: Arc::new(frame.tile_sheet.clone()),
        palette: Arc::new(level_visuals.platforms.palette.clone()),
        tiles: build_sprite_tiles(frame, Direction::Right, 0),
    });
}

pub(crate) fn render_gameplay_frame_evidence(
    manifest: &AssetManifest,
    trace: &[NativeTraceFrame],
    checkpoints: &[GameplayCheckpoint],
) -> Result<Vec<(String, usize, usize, usize, Vec<u8>)>> {
    let session_assets = load_native_session_assets(manifest, 0x00)?;
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

fn build_game_scene(
    trace: &NativeTraceFrame,
    debug_overlay_visible: bool,
    level_visuals: &NativeLevelVisuals,
    block_grid: Option<&BlockGrid>,
) -> Scene {
    let background = background_colour(level_visuals);
    let mut commands = Vec::new();

    push_background_tiles(&mut commands, trace, level_visuals);
    push_level_layout_terrain(&mut commands, trace, level_visuals, block_grid);
    push_block_grid_overrides(&mut commands, trace, level_visuals, block_grid);

    for object in trace
        .active_objects
        .iter()
        .filter(|object| matches!(object.kind, kid_core::NativeObjectKind::Platform { .. }))
    {
        if let NativeObjectKind::Platform { kind, .. } = object.kind {
            push_platform_tiles(&mut commands, object, trace, level_visuals, kind);
        }
    }
    for object in trace
        .active_objects
        .iter()
        .filter(|object| matches!(object.kind, kid_core::NativeObjectKind::Enemy { .. }))
    {
        if let NativeObjectKind::Enemy { object_type } = object.kind {
            push_enemy_sprite(&mut commands, object, trace, level_visuals, object_type);
        }
    }
    // Player: blink every 4 frames during invulnerability
    let player_visible = trace.invulnerable_frames == 0 || (trace.frame_index / 4) % 2 == 0;
    if player_visible {
        for object in trace
            .active_objects
            .iter()
            .filter(|object| matches!(object.kind, kid_core::NativeObjectKind::Player))
        {
            push_player_sprite(&mut commands, object, trace, level_visuals);
        }
    }

    // Murder wall: red wall advancing from the side
    if let Some(wall_x_fp) = trace.murder_wall_x {
        let wall_x_screen = (wall_x_fp >> 16) as i32 - (trace.camera_x_pos >> 16) as i32;
        let wall_colour: [u8; 3] = [180, 20, 20];
        commands.push(DrawCommand::Rect {
            x: wall_x_screen - 48,
            y: 0,
            width: 48,
            height: FRAME_HEIGHT as i32,
            colour: wall_colour,
        });
    }

    // Projectiles: small yellow dots
    for &(px, py) in &trace.active_projectiles {
        let sx = (px >> 16) as i32 - (trace.camera_x_pos >> 16) as i32;
        let sy = (py >> 16) as i32 - (trace.camera_y_pos >> 16) as i32;
        commands.push(DrawCommand::Rect {
            x: sx - 2,
            y: sy - 2,
            width: 4,
            height: 4,
            colour: [255, 200, 0],
        });
    }

    push_game_hud(&mut commands, trace, level_visuals);

    // Level Complete overlay
    if trace.level_complete {
        let lc_text = "LEVEL COMPLETE";
        let text_w = lc_text.len() as i32 * 8 * 2;
        let text_x = (FRAME_WIDTH as i32 - text_w) / 2;
        let text_y = (FRAME_HEIGHT as i32) / 2 - 24;
        let panel_w = text_w + 16;
        push_panel(&mut commands, text_x - 8, text_y - 4, panel_w, 56, [0, 40, 0], [80, 255, 80]);
        commands.push(text(text_x, text_y, lc_text, [80, 255, 80], 2));
        let time_bonus = u32::from(trace.level_timer_seconds) * 100;
        let bonus_line = format!("TIME BONUS: {}", time_bonus);
        let bonus_x = (FRAME_WIDTH as i32 - bonus_line.len() as i32 * 8) / 2;
        commands.push(text(bonus_x, text_y + 22, &bonus_line, [200, 255, 200], 1));
        let score_line = format!("SCORE: {}", trace.score);
        let score_x = (FRAME_WIDTH as i32 - score_line.len() as i32 * 8) / 2;
        commands.push(text(score_x, text_y + 34, &score_line, [255, 255, 255], 1));
    }

    // Game Over / Continue overlay
    if trace.game_over {
        if trace.number_continues > 0 {
            let title = "CONTINUE?";
            let title_w = title.len() as i32 * 8 * 2;
            let title_x = (FRAME_WIDTH as i32 - title_w) / 2;
            let title_y = (FRAME_HEIGHT as i32) / 2 - 16;
            let count_line = format!("{} REMAINING", trace.number_continues);
            let count_w = count_line.len() as i32 * 8;
            let count_x = (FRAME_WIDTH as i32 - count_w) / 2;
            let panel_w = title_w.max(count_w + 16) + 16;
            let panel_x = (FRAME_WIDTH as i32 - panel_w) / 2;
            push_panel(&mut commands, panel_x, title_y - 4, panel_w, 40, [0, 40, 80], [80, 180, 255]);
            commands.push(text(title_x, title_y, title, [80, 180, 255], 2));
            commands.push(text(count_x, title_y + 24, &count_line, [200, 220, 255], 1));
        } else {
            let go_text = "GAME OVER";
            let text_w = go_text.len() as i32 * 8 * 2;
            let text_x = (FRAME_WIDTH as i32 - text_w) / 2;
            let text_y = (FRAME_HEIGHT as i32) / 2 - 8;
            push_panel(&mut commands, text_x - 8, text_y - 4, text_w + 16, 24, [80, 0, 0], [255, 80, 80]);
            commands.push(text(text_x, text_y, go_text, [255, 80, 80], 2));
        }
    }

    // Pause overlay
    if trace.paused {
        let pause_text = "PAUSED";
        let text_w = pause_text.len() as i32 * 8 * 2;
        let text_x = (FRAME_WIDTH as i32 - text_w) / 2;
        let text_y = (FRAME_HEIGHT as i32) / 2 - 8;
        push_panel(&mut commands, text_x - 8, text_y - 4, text_w + 16, 24, [0, 0, 0], [255, 255, 255]);
        commands.push(text(text_x, text_y, pause_text, [255, 255, 255], 2));
    }

    if debug_overlay_visible {
        push_panel(&mut commands, 8, 8, 304, 56, [12, 18, 24], [160, 214, 255]);
        commands.push(text(
            16,
            16,
            &format!(
                "{}  Frame {}  Pause {}",
                game_mode_label(trace.game_mode),
                trace.frame_index,
                on_off(trace.paused)
            ),
            [255, 240, 196],
            1,
        ));
        commands.push(text(
            16,
            28,
            &format!(
                "Level {:02X} {}  Lives {}  HP {}  {}",
                trace.current_level.0,
                trace.current_level_name.as_deref().unwrap_or(""),
                trace.number_lives,
                trace.number_hitpoints,
                form_label(trace.current_helmet_id)
            ),
            [220, 220, 220],
            1,
        ));
        commands.push(text(
            16,
            40,
            &format!(
                "Score {}  Diamonds {}  Timer {}s  Projectiles {}",
                trace.score,
                trace.diamond_count,
                trace.level_timer_seconds,
                trace.active_projectiles.len()
            ),
            [220, 220, 190],
            1,
        ));
        commands.push(text(
            16,
            52,
            &format!(
                "Camera {:08X},{:08X}  Objects {}/{}",
                trace.camera_x_pos,
                trace.camera_y_pos,
                trace.number_objects,
                trace.number_gfx_objects
            ),
            [190, 190, 190],
            1,
        ));
    }

    Scene {
        background,
        commands,
    }
}

fn background_colour(level_visuals: &NativeLevelVisuals) -> [u8; 3] {
    level_visuals
        .palettes
        .background
        .colours
        .first()
        .map(|colour| [colour.red, colour.green, colour.blue])
        .unwrap_or(SKY_COLOUR)
}

fn object_rect(
    object: &NativeObjectSnapshot,
    trace: &NativeTraceFrame,
    width: i32,
    height: i32,
    colour: [u8; 3],
) -> DrawCommand {
    DrawCommand::Rect {
        x: screen_coord(object.x_pos, trace.camera_x_pos),
        y: screen_coord(object.y_pos, trace.camera_y_pos),
        width,
        height,
        colour,
    }
}

fn screen_coord(world: u32, camera: u32) -> i32 {
    let world = i64::from(world);
    let camera = i64::from(camera);
    ((world - camera) >> 16) as i32
}

fn render_scene(scene: &Scene, rgb: &mut [u8]) {
    clear_frame(rgb, scene.background);
    let mut high_priority_pixels = vec![false; (FRAME_WIDTH * FRAME_HEIGHT) as usize];
    for command in &scene.commands {
        match command {
            DrawCommand::Rect {
                x,
                y,
                width,
                height,
                colour,
            } => fill_rect(
                rgb,
                &mut high_priority_pixels,
                *x,
                *y,
                *width,
                *height,
                *colour,
            ),
            DrawCommand::Text {
                x,
                y,
                text,
                colour,
                scale,
            } => draw_text(
                rgb,
                &mut high_priority_pixels,
                *x,
                *y,
                text,
                *colour,
                *scale,
            ),
            DrawCommand::IndexedTile {
                x,
                y,
                tile_sheet,
                tile,
                palette,
            } => draw_indexed_tile(
                rgb,
                &mut high_priority_pixels,
                *x,
                *y,
                tile_sheet,
                *tile,
                palette,
            ),
            DrawCommand::Sprite {
                x,
                y,
                tile_sheet,
                palette,
                tiles,
            } => draw_sprite(
                rgb,
                &mut high_priority_pixels,
                *x,
                *y,
                tile_sheet,
                palette,
                tiles,
            ),
        }
    }
}

fn clear_frame(rgb: &mut [u8], colour: [u8; 3]) {
    for chunk in rgb.chunks_exact_mut(3) {
        chunk.copy_from_slice(&colour);
    }
}

fn push_panel(
    commands: &mut Vec<DrawCommand>,
    x: i32,
    y: i32,
    width: i32,
    height: i32,
    fill: [u8; 3],
    border: [u8; 3],
) {
    commands.push(DrawCommand::Rect {
        x,
        y,
        width,
        height,
        colour: fill,
    });
    commands.push(DrawCommand::Rect {
        x,
        y,
        width,
        height: 2,
        colour: border,
    });
    commands.push(DrawCommand::Rect {
        x,
        y: y + height - 2,
        width,
        height: 2,
        colour: border,
    });
    commands.push(DrawCommand::Rect {
        x,
        y,
        width: 2,
        height,
        colour: border,
    });
    commands.push(DrawCommand::Rect {
        x: x + width - 2,
        y,
        width: 2,
        height,
        colour: border,
    });
}

fn text(x: i32, y: i32, value: &str, colour: [u8; 3], scale: i32) -> DrawCommand {
    DrawCommand::Text {
        x,
        y,
        text: value.to_string(),
        colour,
        scale,
    }
}

fn draw_text(
    rgb: &mut [u8],
    high_priority_pixels: &mut [bool],
    x: i32,
    y: i32,
    text: &str,
    colour: [u8; 3],
    scale: i32,
) {
    let mut cursor_x = x;
    let mut cursor_y = y;
    for ch in text.chars() {
        if ch == '\n' {
            cursor_x = x;
            cursor_y += 10 * scale;
            continue;
        }
        draw_glyph(
            rgb,
            high_priority_pixels,
            cursor_x,
            cursor_y,
            ch,
            colour,
            scale,
        );
        cursor_x += 8 * scale;
    }
}

fn draw_glyph(
    rgb: &mut [u8],
    high_priority_pixels: &mut [bool],
    x: i32,
    y: i32,
    ch: char,
    colour: [u8; 3],
    scale: i32,
) {
    let Some(bitmap) = BASIC_FONTS.get(ch) else {
        return;
    };
    for (row, bits) in bitmap.iter().enumerate() {
        for column in 0..8 {
            if bits & (1 << column) == 0 {
                continue;
            }
            fill_rect(
                rgb,
                high_priority_pixels,
                x + (column * scale),
                y + ((row as i32) * scale),
                scale,
                scale,
                colour,
            );
        }
    }
}

fn fill_rect(
    rgb: &mut [u8],
    high_priority_pixels: &mut [bool],
    x: i32,
    y: i32,
    width: i32,
    height: i32,
    colour: [u8; 3],
) {
    for py in y.max(0)..(y + height).min(FRAME_HEIGHT as i32) {
        for px in x.max(0)..(x + width).min(FRAME_WIDTH as i32) {
            set_pixel(rgb, high_priority_pixels, px, py, colour, None);
        }
    }
}

fn draw_sprite(
    rgb: &mut [u8],
    high_priority_pixels: &mut [bool],
    x: i32,
    y: i32,
    tile_sheet: &TileSheet,
    palette: &Palette,
    tiles: &[SpriteTile],
) {
    for tile in tiles {
        draw_indexed_tile(
            rgb,
            high_priority_pixels,
            x + tile.offset_x,
            y + tile.offset_y,
            tile_sheet,
            tile.tile,
            palette,
        );
    }
}

fn draw_indexed_tile(
    rgb: &mut [u8],
    high_priority_pixels: &mut [bool],
    x: i32,
    y: i32,
    tile_sheet: &TileSheet,
    tile_ref: TileRef,
    palette: &Palette,
) {
    let Some(tile) = tile_sheet.tiles.get(usize::from(tile_ref.tile_index)) else {
        return;
    };

    for dest_y in 0..8 {
        for dest_x in 0..8 {
            let source_x = if tile_ref.flip_x { 7 - dest_x } else { dest_x };
            let source_y = if tile_ref.flip_y { 7 - dest_y } else { dest_y };
            let palette_index = tile.pixels[source_y * 8 + source_x];
            let Some(colour) =
                resolve_palette_colour(palette, tile_ref.palette_line, palette_index)
            else {
                continue;
            };
            set_pixel(
                rgb,
                high_priority_pixels,
                x + dest_x as i32,
                y + dest_y as i32,
                colour,
                Some(tile_ref.priority),
            );
        }
    }
}

fn resolve_palette_colour(
    palette: &Palette,
    selected_palette_line: u8,
    palette_index: u8,
) -> Option<[u8; 3]> {
    if palette_index == 0 {
        return None;
    }

    let colour = if palette.colours.len() == 8 && selected_palette_line == 0 {
        palette.colours.get(usize::from(palette_index))
    } else {
        let palette_offset =
            usize::from(selected_palette_line).saturating_mul(PALETTE_LINE_COLOUR_COUNT);
        palette
            .colours
            .get(palette_offset + usize::from(palette_index - 1))
    }?;

    Some([colour.red, colour.green, colour.blue])
}

fn set_pixel(
    rgb: &mut [u8],
    high_priority_pixels: &mut [bool],
    x: i32,
    y: i32,
    colour: [u8; 3],
    priority: Option<bool>,
) {
    if !(0..FRAME_WIDTH as i32).contains(&x) || !(0..FRAME_HEIGHT as i32).contains(&y) {
        return;
    }
    let pixel_index = (y as u32 * FRAME_WIDTH + x as u32) as usize;
    if matches!(priority, Some(false)) && high_priority_pixels[pixel_index] {
        return;
    }
    let rgb_index = pixel_index * 3;
    rgb[rgb_index..rgb_index + 3].copy_from_slice(&colour);
    high_priority_pixels[pixel_index] = matches!(priority, Some(true));
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
        GameMode::TitleScreen => "TitleScreen",
        GameMode::LevelTransition => "LevelTransition",
        GameMode::InGame => "InGame",
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
            for sample in samples.iter_mut() {
                if tone.remaining_samples == 0 {
                    return false;
                }
                // Square wave with volume envelope
                let value = if tone.phase < 0.5 { tone.volume } else { -tone.volume };
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
    // Stomp bounce (landed on enemy — on_ground went false with upward velocity and enemy killed same frame)
    if curr.kid_on_ground == Some(false)
        && prev.kid_on_ground == Some(true)
        && curr.kid_y_velocity.is_some_and(|v| v < 0)
        && curr.score > prev.score
        && curr.diamond_count == prev.diamond_count
    {
        sfx.play(587.0, 50, 0.1); // D5 bounce tone
    }
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
        ForegroundLayout, HudIcon, HudVisuals, NativeLevelVisuals, LevelVisuals, MdColour, Palette,
        PaletteSet, PlatformVisuals, PlayerVisuals, SpriteFrame, Tile, TileQuad, TileRef,
        TileSheet,
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
            paused: false,
            kid_x_pos: Some(0x0020_0000),
            kid_y_pos: Some(0x010F_0000),
            kid_x_velocity: Some(0),
            kid_y_velocity: Some(0),
            kid_facing: Some(Direction::Right),
            kid_on_ground: Some(true),
            active_objects: Vec::new(),
            camera_x_pos: 0x0010_0000,
            camera_y_pos: 0x0008_0000,
            number_objects: 3,
            number_gfx_objects: 4,
            level_complete: false,
            game_over: false,
            game_complete: false,
            number_continues: 3,
            invulnerable_frames: 0,
            diamond_count: 0,
            score: 0,
            level_timer_seconds: 180,
            murder_wall_x: None,
            active_projectiles: Vec::new(),
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
                        if *y >= (FRAME_HEIGHT as i32 - 18)
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
        PlayerVisuals {
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
            jump_rise: sample_player_frame(8),
            jump_fall: sample_player_frame(9),
        }
    }

    fn sample_enemy_visuals() -> EnemyVisuals {
        EnemyVisuals {
            diamond: AnimatedSpriteVisual {
                palette: Palette {
                    colours: vec![colour(0, 0, 0), colour(120, 220, 255)],
                },
                frames: vec![
                    sample_object_frame(21, 20, 20),
                    sample_object_frame(22, 20, 20),
                    sample_object_frame(23, 20, 20),
                ],
                frame_duration: 6,
            },
            dragon: AnimatedSpriteVisual {
                palette: Palette {
                    colours: vec![colour(0, 0, 0), colour(255, 140, 96)],
                },
                frames: vec![
                    sample_object_frame(24, 46, 30),
                    sample_object_frame(25, 48, 25),
                    sample_object_frame(26, 47, 26),
                    sample_object_frame(27, 46, 25),
                    sample_object_frame(28, 46, 30),
                ],
                frame_duration: 7,
            },
            mini_hopping_skull: AnimatedSpriteVisual {
                palette: Palette {
                    colours: vec![colour(0, 0, 0), colour(255, 220, 160)],
                },
                frames: vec![
                    sample_object_frame(31, 16, 17),
                    sample_object_frame(32, 16, 18),
                ],
                frame_duration: 9,
            },
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
        blocks.push(sample_tile_quad(0x0220));
        for group in 0..5 {
            for skin in 0..16 {
                let base = 0x0224 + ((group * 16 + skin) as u16 * 4);
                blocks.push(sample_tile_quad(base));
            }
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
            platforms: sample_platform_visuals(),
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
                    decoded_level1: Some(DecodedLevelAssets {
                        contract_version: 2,
                        level_id: 0x00,
                        foreground: decoded_asset("decoded/level/00/foreground.bin"),
                        block: decoded_asset(decoded_path),
                        background: decoded_asset("decoded/level/00/background.bin"),
                    }),
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

        let Some((_, tile, _)) = indexed_tile_at_with_sheet_len(&scene, 32, 0, 0x80) else {
            panic!("expected a third terrain tile in the opening row");
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
            (16, 0, 0x200, 8),
            (32, 0, 0x200, 76),
            (48, 0, 0x200, 144),
            (64, 0, 0x200, 212),
            (80, 0, 0x200, 280),
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
        assert!(topmost_indexed_tile_at_with_sheet_len(&scene, 96, 0, 0x200).is_some());
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
                        if *y >= (FRAME_HEIGHT as i32 - 18)
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
                        if *y >= (FRAME_HEIGHT as i32 - 18) && tile_sheet.tiles.len() == 0x200
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
                        if *y >= (FRAME_HEIGHT as i32 - 18)
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
                        if *y >= (FRAME_HEIGHT as i32 - 18)
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
    fn run_native_player_loads_level_visuals_once_before_main_loop() {
        let fixture = ManifestFixture::new().expect("manifest fixture");
        let load_count = AtomicUsize::new(0);

        let session_assets = load_native_session_assets_with(&fixture.manifest, 0x00, |_, _| {
            load_count.fetch_add(1, Ordering::SeqCst);
            Ok(sample_level_visuals())
        })
        .expect("session assets");
        let runtime = initialise_runtime(&fixture.manifest, &session_assets);

        assert_eq!(load_count.load(Ordering::SeqCst), 1);
        let opening_cell = runtime
            .block_grid()
            .expect("runtime block grid")
            .cell_at(0, 0);
        assert_eq!(opening_cell.collision, CollisionKind::Spikes);
        assert_eq!(opening_cell.block_type, BlockType::None);
        assert_eq!(opening_cell.skin, 0x33);
        let trace = runtime.trace_frame();
        assert_eq!(trace.game_mode, GameMode::InGame);
        assert_eq!(trace.current_level.0, 0);
        assert_eq!(
            trace.current_level_name.as_deref(),
            Some("Blue Lake Woods 1")
        );
        assert_eq!(trace.kid_on_ground, Some(true));
        assert!(
            trace
                .active_objects
                .iter()
                .any(|object| matches!(object.kind, NativeObjectKind::Player)),
            "native playtest should spawn with the player visible"
        );
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
            },
        ];
        let scene = build_ingame_scene(&trace, true, None);
        assert_eq!(sprite_position(&scene, 41), Some((-4, 0)));
        assert_eq!(sprite_position(&scene, 21), Some((14, -8)));
        assert_eq!(sprite_position(&scene, 1), Some((8, 8)));
    }

    #[test]
    fn hud_toggle_removes_control_overlay() {
        let trace = sample_trace(GameMode::InGame);
        assert!(contains_text(
            &build_ingame_scene(&trace, true, None),
            "Controls Arrows"
        ));
        assert!(!contains_text(
            &build_ingame_scene(&trace, false, None),
            "Controls Arrows"
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
    fn level_layout_terrain_renders_visible_cells() {
        let mut trace = sample_trace(GameMode::InGame);
        trace.camera_x_pos = 0;
        trace.camera_y_pos = 0;
        let grid = BlockGrid::from_level_layout(&[0x00, 0x01, 0x00, 0x02, 0x00, 0x03], 3, 1);

        let scene = build_ingame_scene(&trace, false, Some(&grid));
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
            let offset = (1 * 40 + col) * 2;
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
    fn destroyed_block_renders_as_empty() {
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

        // Destroy the block (set to empty)
        grid.set_cell_at(0, 0, Cell::from_be_bytes(0, 0));
        let scene = build_ingame_scene(&trace, false, Some(&grid));
        assert!(indexed_tile_at_with_sheet_len(&scene, 0, 0, 0x200).is_none());
    }

    #[test]
    fn block_types_have_distinct_tile_mappings() {
        let visuals = sample_level_visuals();
        let tile_indices: Vec<_> = [Some(0), Some(1), Some(2), Some(6)]
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
        for i in 0..tile_indices.len() {
            for j in (i + 1)..tile_indices.len() {
                assert_ne!(
                    tile_indices[i], tile_indices[j],
                    "block mappings at indices {i} and {j} share the same tile"
                );
            }
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
        }];
        trace
    }

    fn scene_has_player_sprite(scene: &Scene) -> bool {
        scene
            .commands
            .iter()
            .any(|command| matches!(command, DrawCommand::Sprite { .. }))
    }

    fn player_sprite_signature(scene: &Scene) -> Option<u8> {
        scene.commands.iter().find_map(|command| match command {
            DrawCommand::Sprite { tile_sheet, .. } => {
                tile_sheet.tiles.first().map(|tile| tile.pixels[0])
            }
            _ => None,
        })
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
                (1, 8, 0),
                (2, 16, 0),
                (3, 24, 0),
                (4, 0, 8),
                (5, 8, 8),
                (6, 16, 8),
                (7, 24, 8),
                (8, 32, 0),
                (9, 40, 0),
                (10, 32, 8),
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
                (1, 32, 0, true),
                (2, 24, 0, true),
                (3, 16, 0, true),
                (4, 40, 8, true),
                (5, 32, 8, true),
                (6, 24, 8, true),
                (7, 16, 8, true),
                (8, 8, 0, true),
                (9, 0, 0, true),
                (10, 8, 8, true),
                (11, 0, 8, true),
            ]
        );
    }

    #[test]
    fn player_invulnerability_blink_preserves_existing_visibility_rule() {
        let trace = trace_with_player(0, 100);
        assert!(scene_has_player_sprite(&build_ingame_scene(
            &trace, false, None
        )));

        let trace = trace_with_player(4, 100);
        assert!(!scene_has_player_sprite(&build_ingame_scene(
            &trace, false, None
        )));

        let trace = trace_with_player(8, 100);
        assert!(scene_has_player_sprite(&build_ingame_scene(
            &trace, false, None
        )));

        let trace = trace_with_player(12, 100);
        assert!(!scene_has_player_sprite(&build_ingame_scene(
            &trace, false, None
        )));

        let trace = trace_with_player(4, 0);
        assert!(scene_has_player_sprite(&build_ingame_scene(
            &trace, false, None
        )));
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
                        if *y >= (FRAME_HEIGHT as i32 - 18)
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
                        if *y >= (FRAME_HEIGHT as i32 - 18)
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
                    if *y >= (FRAME_HEIGHT as i32 - 18)
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

fn try_load_block_grid(manifest: &AssetManifest, level_id: u8) -> Result<BlockGrid> {
    if let Some(grid) = try_load_generated_level_layout(manifest, level_id)? {
        return Ok(grid);
    }

    let level = manifest
        .levels
        .iter()
        .find(|level| level.level_id == level_id)
        .ok_or_else(|| anyhow!("generated bundle does not contain level 0x{level_id:02X}"))?;
    let header = level.header_metadata.as_ref().ok_or_else(|| {
        anyhow!("generated bundle is missing header metadata for level 0x{level_id:02X}")
    })?;
    let decoded = manifest
        .decoded_level1
        .as_ref()
        .ok_or_else(|| anyhow!("generated bundle is missing decoded Level 1 assets"))?;
    if decoded.level_id != level_id {
        bail!(
            "decoded Level 1 block layout points at level 0x{:02X}, expected 0x{level_id:02X}",
            decoded.level_id
        );
    }

    let layout_path = manifest.output_root.join(&decoded.block.relative_path);
    let data = std::fs::read(&layout_path).map_err(|error| {
        anyhow!(
            "failed to read Level 1 block layout from {}: {error}",
            layout_path.display()
        )
    })?;
    let (width, height) = level_layout_dimensions(header);
    Ok(BlockGrid::from_level_layout(&data, width, height))
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
