use super::*;
use kid_core::GENERIC_ENEMY_DEATH_EFFECT_FRAMES;

const HEADYMETAL_WOBBLE_STAGE_1: u16 = 0x015E;
const HEADYMETAL_WOBBLE_STAGE_2: u16 = 0x00FA;
const HEADYMETAL_WOBBLE_STAGE_3: u16 = 0x0096;
const HEADYMETAL_WOBBLE_STAGE_4: u16 = 0x0064;
const FRIENDLY_PROJECTILE_COLOUR: [u8; 3] = [100, 255, 100];
const ENEMY_PROJECTILE_COLOUR: [u8; 3] = [255, 80, 40];
const FIRE_TRAIL_OUTER_COLOUR: [u8; 3] = [255, 120, 32];
const FIRE_TRAIL_CORE_COLOUR: [u8; 3] = [255, 224, 128];

/// Controls which layers are rendered. Used for layer-isolated diagnosis.
#[derive(Debug, Clone, Copy)]
pub(crate) struct LayerFilter {
    pub background: bool,
    pub terrain: bool,
    pub block_overrides: bool,
    pub platforms: bool,
    pub enemies: bool,
    pub player: bool,
    pub effects: bool,
    pub hud: bool,
}

impl Default for LayerFilter {
    fn default() -> Self {
        Self {
            background: true,
            terrain: true,
            block_overrides: true,
            platforms: true,
            enemies: true,
            player: true,
            effects: true,
            hud: true,
        }
    }
}

impl LayerFilter {
    /// A filter with only one layer enabled.
    pub fn solo(layer_name: &str) -> Self {
        let mut f = Self {
            background: false,
            terrain: false,
            block_overrides: false,
            platforms: false,
            enemies: false,
            player: false,
            effects: false,
            hud: false,
        };
        match layer_name {
            "background" => f.background = true,
            "terrain" => f.terrain = true,
            "block_overrides" => f.block_overrides = true,
            "platforms" => f.platforms = true,
            "enemies" => f.enemies = true,
            "player" => f.player = true,
            "effects" => f.effects = true,
            "hud" => f.hud = true,
            _ => {}
        }
        f
    }
}

fn push_fire_demon_trail(commands: &mut Vec<DrawCommand>, sx: i32, sy: i32, remaining_frames: u16) {
    let phase = usize::from(remaining_frames % 6);
    let outer_rects: &[(i32, i32, i32, i32)] = match phase {
        0 => &[(-2, -4, 4, 2), (-3, -2, 6, 3), (-2, 1, 4, 1)],
        1 => &[(-1, -5, 2, 2), (-3, -3, 6, 3), (-2, 0, 4, 2)],
        2 => &[(-2, -5, 4, 2), (-2, -3, 4, 3), (-3, 0, 6, 2)],
        3 => &[(-1, -4, 2, 2), (-3, -2, 6, 3), (-1, 1, 2, 1)],
        4 => &[(-2, -4, 4, 2), (-3, -2, 6, 2), (-2, 0, 4, 2)],
        _ => &[(-1, -5, 2, 2), (-2, -3, 4, 3), (-2, 0, 4, 1)],
    };
    for &(dx, dy, width, height) in outer_rects {
        commands.push(DrawCommand::Rect {
            x: sx + dx,
            y: sy + dy,
            width,
            height,
            colour: FIRE_TRAIL_OUTER_COLOUR,
        });
    }

    let core_rects: &[(i32, i32, i32, i32)] = match phase {
        0 | 4 => &[(-1, -2, 2, 2)],
        1 | 5 => &[(-1, -3, 2, 3)],
        _ => &[(-1, -2, 2, 3)],
    };
    for &(dx, dy, width, height) in core_rects {
        commands.push(DrawCommand::Rect {
            x: sx + dx,
            y: sy + dy,
            width,
            height,
            colour: FIRE_TRAIL_CORE_COLOUR,
        });
    }
}

pub(crate) fn build_scene(
    trace: &NativeTraceFrame,
    debug_overlay_visible: bool,
    level_visuals: Option<&NativeLevelVisuals>,
    block_grid: Option<&BlockGrid>,
) -> Scene {
    build_scene_filtered(
        trace,
        debug_overlay_visible,
        level_visuals,
        block_grid,
        &LayerFilter::default(),
    )
}

pub(crate) fn build_scene_filtered(
    trace: &NativeTraceFrame,
    debug_overlay_visible: bool,
    level_visuals: Option<&NativeLevelVisuals>,
    block_grid: Option<&BlockGrid>,
    filter: &LayerFilter,
) -> Scene {
    match trace.game_mode {
        GameMode::InGame | GameMode::DemoPlay => build_game_scene_filtered(
            trace,
            debug_overlay_visible,
            level_visuals.expect("Level 1 visuals are required for gameplay scenes"),
            block_grid,
            filter,
        ),
        GameMode::LevelTransition => build_transition_scene(trace),
        GameMode::OptionsMenu => build_options_scene(trace),
        GameMode::EndingSequence => build_ending_scene(trace),
        _ => build_status_scene(trace, debug_overlay_visible),
    }
}

pub(super) fn build_transition_scene(trace: &NativeTraceFrame) -> Scene {
    let mut commands = Vec::new();
    commands.push(DrawCommand::Rect {
        x: 0,
        y: 0,
        width: FRAME_WIDTH as i32,
        height: FRAME_HEIGHT as i32,
        colour: [0, 0, 0],
    });
    let level_name = trace.current_level_name.as_deref().unwrap_or("Unknown");
    let level_text = level_name.to_string();
    let text_w = level_text.len() as i32 * 8 * 2;
    let text_x = (FRAME_WIDTH as i32 - text_w) / 2;
    let text_y = (FRAME_HEIGHT as i32) / 2 - 20;
    commands.push(text(text_x, text_y, &level_text, [255, 255, 255], 2));

    // Level ID subtitle
    let id_text = format!("Level {:02X}", trace.current_level.0);
    let id_w = id_text.len() as i32 * 8;
    commands.push(text(
        (FRAME_WIDTH as i32 - id_w) / 2,
        text_y + 22,
        &id_text,
        [120, 120, 120],
        1,
    ));

    // Lives and score
    let info_text = format!("Lives: {}  Score: {}", trace.number_lives, trace.score);
    let info_w = info_text.len() as i32 * 8;
    commands.push(text(
        (FRAME_WIDTH as i32 - info_w) / 2,
        text_y + 40,
        &info_text,
        [100, 140, 180],
        1,
    ));
    Scene {
        background: [0, 0, 0],
        commands,
    }
}

pub(super) fn build_status_scene(trace: &NativeTraceFrame, debug_overlay_visible: bool) -> Scene {
    let mut commands = Vec::new();

    if trace.game_complete {
        return build_victory_scene(trace);
    }

    // Sega screen — white text on blue
    if trace.game_mode == GameMode::SegaScreen {
        commands.push(DrawCommand::Rect {
            x: 0,
            y: 0,
            width: FRAME_WIDTH as i32,
            height: FRAME_HEIGHT as i32,
            colour: [0, 0, 128],
        });
        let sega = "SEGA";
        let sega_w = sega.len() as i32 * 8 * 4;
        commands.push(text(
            (FRAME_WIDTH as i32 - sega_w) / 2,
            80,
            sega,
            [255, 255, 255],
            4,
        ));
        return Scene {
            background: [0, 0, 128],
            commands,
        };
    }

    // Intro sequences — distinct visuals per segment
    if matches!(
        trace.game_mode,
        GameMode::IntroSequence1
            | GameMode::IntroSequence2
            | GameMode::IntroSequence3
            | GameMode::IntroSequence4
            | GameMode::IntroSequence5
            | GameMode::IntroSequence6
    ) {
        commands.push(DrawCommand::Rect {
            x: 0,
            y: 0,
            width: FRAME_WIDTH as i32,
            height: FRAME_HEIGHT as i32,
            colour: [0, 0, 0],
        });
        let (line1, line2, colour) = match trace.game_mode {
            GameMode::IntroSequence1 => (
                "The Wild Side",
                "A strange arcade machine appeared...",
                [200, 160, 80],
            ),
            GameMode::IntroSequence2 => (
                "Heady Metal's Challenge",
                "Kids go in... but they don't come out.",
                [180, 80, 80],
            ),
            _ => (
                "Enter the Game",
                "One kid decides to play.",
                [120, 180, 255],
            ),
        };
        let w1 = line1.len() as i32 * 8 * 2;
        commands.push(text((FRAME_WIDTH as i32 - w1) / 2, 70, line1, colour, 2));
        let w2 = line2.len() as i32 * 8;
        commands.push(text(
            (FRAME_WIDTH as i32 - w2) / 2,
            110,
            line2,
            [160, 160, 160],
            1,
        ));
        commands.push(text(
            (FRAME_WIDTH as i32 - 18 * 8) / 2,
            180,
            "Press Start to skip",
            [80, 80, 80],
            1,
        ));
        return Scene {
            background: [0, 0, 0],
            commands,
        };
    }

    // Title screen
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
    if (trace.frame_index / 30).is_multiple_of(2) {
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
        "R restart level  F10 dump record  Tab debug  F11 fullscreen  Esc quit",
        [140, 140, 140],
        1,
    ));
    commands.push(text(
        20,
        188,
        "B options menu  N/P warp level",
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

pub(super) fn build_options_scene(trace: &NativeTraceFrame) -> Scene {
    if let Some(track) = trace.sound_test_track {
        return build_sound_test_scene(track);
    }

    let mut commands = Vec::new();

    commands.push(DrawCommand::Rect {
        x: 0,
        y: 0,
        width: FRAME_WIDTH as i32,
        height: FRAME_HEIGHT as i32,
        colour: [8, 12, 20],
    });

    let title = "OPTIONS";
    let title_w = title.len() as i32 * 8 * 2;
    commands.push(text(
        (FRAME_WIDTH as i32 - title_w) / 2,
        40,
        title,
        [255, 220, 80],
        2,
    ));

    let cursor = trace.options_cursor.unwrap_or(0) as i32;
    let two_p_label = if trace.options_two_player_two_controllers {
        "2 PLAYERS: TWO CONTROLLERS"
    } else {
        "2 PLAYERS: ONE CONTROLLER"
    };
    let controls_label = match trace.options_controls_scheme {
        0 => "CONTROLS: A SPD  B JMP  C SPC",
        1 => "CONTROLS: A JMP  B SPD  C SPC",
        2 => "CONTROLS: A SPD  B SPC  C JMP",
        3 => "CONTROLS: A SPC  B SPD  C JMP",
        4 => "CONTROLS: A JMP  B SPC  C SPD",
        _ => "CONTROLS: A SPC  B JMP  C SPD",
    };
    let speed_label = if trace.options_speed_button_fast {
        "SPEED BUTTON: FAST ACTION"
    } else {
        "SPEED BUTTON: NORMAL ACTION"
    };
    let items = [two_p_label, controls_label, speed_label, "EXIT"];
    for (i, item) in items.iter().enumerate() {
        let y = 72 + i as i32 * 24;
        let colour = if i as i32 == cursor {
            [255, 255, 255]
        } else {
            [120, 120, 120]
        };
        commands.push(text(44, y, item, colour, 1));
        if i as i32 == cursor {
            commands.push(text(28, y, ">", [255, 220, 80], 1));
        }
    }

    commands.push(text(
        24,
        184,
        "Up/Down select  Left/Right change  Start/B back",
        [100, 100, 100],
        1,
    ));

    Scene {
        background: [8, 12, 20],
        commands,
    }
}

fn build_sound_test_scene(track: u8) -> Scene {
    let mut commands = Vec::new();

    commands.push(DrawCommand::Rect {
        x: 0,
        y: 0,
        width: FRAME_WIDTH as i32,
        height: FRAME_HEIGHT as i32,
        colour: [8, 12, 20],
    });

    let title = "SOUND TEST";
    let title_w = title.len() as i32 * 8 * 2;
    commands.push(text(
        (FRAME_WIDTH as i32 - title_w) / 2,
        40,
        title,
        [255, 220, 80],
        2,
    ));

    let track_label = format!("TRACK: {:02X}", track);
    let label_w = track_label.len() as i32 * 8 * 2;
    commands.push(text(
        (FRAME_WIDTH as i32 - label_w) / 2,
        100,
        &track_label,
        [255, 255, 255],
        2,
    ));

    commands.push(text(
        44,
        172,
        "Left/Right +1  Up/Down +16  A/C play  B back",
        [100, 100, 100],
        1,
    ));

    Scene {
        background: [8, 12, 20],
        commands,
    }
}

pub(super) fn build_victory_scene(trace: &NativeTraceFrame) -> Scene {
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
    if (trace.frame_index / 30).is_multiple_of(2) {
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

/// Credits text screens matching ASM EndText1 (kid.asm:32804-32932).
const CREDITS_SCREENS: &[&[&str]] = &[
    &[
        "CONGRATULATIONS",
        "",
        "YOU HAVE FREED THEM",
        "FROM THE EVIL BOSS",
        "",
        "YOU WILL BE REMEMBERED",
        "AS THE FIRST HOLOGRAPHIC",
        "GAME CHAMPION ...",
        "THE KID CHAMELEON.",
    ],
    &[
        "AND NOW ...",
        "",
        "THE PEOPLE WHO GAVE",
        "UP THEIR LIVES, WIVES",
        "AND SANITY TO MAKE THIS",
        "GAME A REALITY ...",
        "",
        "THE SEGA TECHNICAL",
        "INSTITUTE TEAM.",
    ],
    &[
        "... GAME DESIGN ..",
        "",
        "HOYT NG",
        "BILL DUNN",
        "RICK MACARAEG",
        "GRAEME BAYLESS",
    ],
    &[
        "... SOFTWARE ...",
        "",
        "BC TCHIU LE",
        "MARK CERNY",
        "STEVE WOITA",
        "BILL WILLIS",
    ],
    &[
        "... ART ...",
        "",
        "ALAN ACKERMAN",
        "BRENDA ROSS",
        "CRAIG STITT",
        "PAUL MICA",
        "JUDY TOTOYA",
    ],
    &[
        "SOUND BY",
        "NU ROMANTIC PRODUCTIONS",
        "",
        "SPECIAL THANKS TO ...",
        "SCOTT CHANDLER",
        "HUGH BOWEN",
        "HAVEN CARTER",
        "AND THE TEST GROUP",
    ],
    &["THE END", "", "THANK YOU FOR PLAYING!"],
];

/// ASM timing constants for the ending sequence phases.
const ENDING_PHASE1_END: usize = 40; // pause
const ENDING_PHASE2_END: usize = ENDING_PHASE1_END + 200; // music delay
const ENDING_PHASE3_END: usize = ENDING_PHASE2_END + 256; // fade to black
/// Credits screens start after the fade.
const CREDITS_START: usize = ENDING_PHASE3_END;
/// Each credits screen: 112 frames scroll + 200 frames display = 312 frames.
const CREDITS_PER_SCREEN: usize = 312;

pub(super) fn build_ending_scene(trace: &NativeTraceFrame) -> Scene {
    let mut commands = Vec::new();
    let elapsed = trace.ending_elapsed.unwrap_or(0);

    // Phase 1-3: fade to black (black screen, the level has already been cleared)
    if elapsed < CREDITS_START {
        // Calculate fade intensity: fully bright at start, fading to black
        let fade_progress = if elapsed >= ENDING_PHASE2_END {
            let fade_elapsed = elapsed - ENDING_PHASE2_END;
            (fade_elapsed * 255 / 256).min(255) as u8
        } else {
            0
        };
        let brightness = 255u8.saturating_sub(fade_progress);
        commands.push(DrawCommand::Rect {
            x: 0,
            y: 0,
            width: FRAME_WIDTH as i32,
            height: FRAME_HEIGHT as i32,
            colour: [0, 0, brightness / 10], // dark blue fading to black
        });
        return Scene {
            background: [0, 0, 0],
            commands,
        };
    }

    // Credits phase: determine which screen to show
    let credits_elapsed = elapsed - CREDITS_START;
    let screen_index = credits_elapsed / CREDITS_PER_SCREEN;

    // Black background
    commands.push(DrawCommand::Rect {
        x: 0,
        y: 0,
        width: FRAME_WIDTH as i32,
        height: FRAME_HEIGHT as i32,
        colour: [4, 8, 20],
    });

    if screen_index < CREDITS_SCREENS.len() {
        let screen = CREDITS_SCREENS[screen_index];
        let within_screen = credits_elapsed % CREDITS_PER_SCREEN;

        // Fade in during scroll phase, full brightness during display
        let alpha = if within_screen < 112 {
            ((within_screen as f32 / 112.0) * 255.0) as u8
        } else {
            255
        };

        let start_y = 40i32;
        let line_height = 18i32;
        for (i, line) in screen.iter().enumerate() {
            if line.is_empty() {
                continue;
            }
            let y = start_y + i as i32 * line_height;
            let line_w = line.len() as i32 * 8;
            let colour = if i == 0 && screen_index < 2 {
                // Title lines in gold
                [
                    alpha,
                    (alpha as u16 * 220 / 255) as u8,
                    (alpha as u16 * 80 / 255) as u8,
                ]
            } else {
                [alpha, alpha, alpha]
            };
            commands.push(text((FRAME_WIDTH as i32 - line_w) / 2, y, line, colour, 1));
        }
    }

    Scene {
        background: [4, 8, 20],
        commands,
    }
}

pub(super) fn resolve_ingame_vram_tile(
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

pub(super) fn push_resolved_tile(
    commands: &mut Vec<DrawCommand>,
    x: i32,
    y: i32,
    tile: ResolvedSceneTile,
) {
    commands.push(DrawCommand::IndexedTile {
        x,
        y,
        tile_sheet: tile.tile_sheet,
        tile: tile.tile,
        palette: tile.palette,
    });
}

pub(super) fn push_background_tiles(
    commands: &mut Vec<DrawCommand>,
    trace: &NativeTraceFrame,
    level_visuals: &NativeLevelVisuals,
) {
    let camera_x_px = (trace.camera_x_pos >> 16) as i32;
    let camera_y_px = (trace.camera_y_pos >> 16) as i32;
    let tile_sheet = Arc::new(level_visuals.background_tiles.clone());
    let palette = Arc::new(background_palette_for_scene(level_visuals));

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
                let shimmer_offset = layered_background_shimmer_offset(
                    trace.frame_index,
                    row_index,
                    row.selector,
                    background.effect_start_row_selector,
                );
                for (column, tile) in row.tiles.iter().enumerate() {
                    let (screen_x, screen_y) = background_tile_screen_position(
                        trace,
                        level_visuals,
                        column as u16,
                        row_index as u16,
                        camera_x_px,
                        camera_y_px,
                    );
                    let screen_x = screen_x + shimmer_offset;
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

fn background_palette_for_scene(level_visuals: &NativeLevelVisuals) -> Palette {
    let mut palette = level_visuals.palettes.background.clone();
    if level_visuals.level.background_theme_id == 9 && palette.colours.len() >= 8 {
        // ASM sub_129AE swaps CRAM slot 8 with slot 10 for forest backgrounds
        // after loading the 8-colour background palette into line 1.
        let original = palette.colours.clone();
        palette.colours[0] = original[2];
        palette.colours[1] = original[1];
        palette.colours[2] = kid_render::MdColour::from_word(0);
        palette.colours[3..8].copy_from_slice(&original[3..8]);
    }
    palette
}

fn push_sprite_frame(
    commands: &mut Vec<DrawCommand>,
    frame: &kid_render::SpriteFrame,
    palette: &Palette,
    screen_x: i32,
    screen_y: i32,
) {
    let facing = Direction::Right;
    let (draw_x, draw_y) = sprite_draw_position(frame, screen_x, screen_y, facing);
    commands.push(DrawCommand::Sprite {
        x: draw_x,
        y: draw_y,
        tile_sheet: Arc::new(frame.tile_sheet.clone()),
        palette: Arc::new(palette.clone()),
        tiles: build_sprite_tiles(frame, facing, 0),
    });
}

// Forest water ripple table from ASM: BackgroundScroll_ForestWaterRippleData
// (kid.asm line 34310). 64 signed byte entries, max amplitude 8px.
const FOREST_RIPPLE: [i8; 64] = [
    0, 0, 0, 0, 1, 1, 1, 2, 2, 2, 3, 3, 4, 5, 6, 7, 7, 7, 8, 8, 8, 8, 7, 7, 7, 6, 5, 4, 3, 2, 2, 2,
    1, 1, 1, 0, 0, 0, 0, 1, 2, 3, 4, 4, 4, 4, 5, 6, 7, 8, 8, 8, 7, 7, 6, 6, 5, 5, 4, 4, 3, 3, 2, 1,
];

pub(super) fn layered_background_shimmer_offset(
    frame_index: usize,
    row_index: usize,
    row_selector: u8,
    effect_start_row_selector: Option<u8>,
) -> i32 {
    if effect_start_row_selector.is_none_or(|start| row_selector < start) {
        return 0;
    }
    let time_phase = (frame_index / 4) & 0x3F;
    let table_idx = (time_phase + row_index) & 0x3F;
    i32::from(FOREST_RIPPLE[table_idx])
}

pub(super) fn background_tile_screen_position(
    trace: &NativeTraceFrame,
    level_visuals: &NativeLevelVisuals,
    x_tiles: u16,
    y_tiles: u16,
    camera_x_px: i32,
    camera_y_px: i32,
) -> (i32, i32) {
    let world_x_px = i32::from(x_tiles) * 8;
    let world_y_px = i32::from(y_tiles) * 8;
    // Plane B vertical scroll uses Camera_Y_pos >> 2 in the original game.
    let background_scroll_y_px = camera_y_px >> 2;
    let Some(profile) = level_visuals.level.background_scroll.as_ref() else {
        return (
            world_x_px.saturating_sub(camera_x_px),
            world_y_px.saturating_sub(background_scroll_y_px),
        );
    };

    let layer_speed = profile
        .layer_speeds
        .get(usize::from(y_tiles))
        .copied()
        .unwrap_or(0x10);
    (
        world_x_px.saturating_sub(background_scroll_x_px(trace, camera_x_px, layer_speed)),
        world_y_px.saturating_sub(background_scroll_y_px),
    )
}

pub(super) fn background_scroll_x_px(
    trace: &NativeTraceFrame,
    camera_x_px: i32,
    layer_speed: u8,
) -> i32 {
    match layer_speed {
        0x00 => (trace.frame_index as i32 / 2).rem_euclid(512),
        0x08 => (trace.frame_index as i32 / 4).rem_euclid(512),
        0x10 => 0,
        0x18..=0x88 if (layer_speed - 0x10).is_multiple_of(8) => {
            let step = i32::from((layer_speed - 0x10) / 8);
            camera_x_px.saturating_mul(step) / 16
        }
        _ => camera_x_px,
    }
}

pub(super) fn push_tile_quad(
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

pub(super) fn push_level_layout_terrain(
    commands: &mut Vec<DrawCommand>,
    trace: &NativeTraceFrame,
    level_visuals: &NativeLevelVisuals,
) {
    let camera_x_px = (trace.camera_x_pos >> 16) as i32;
    let camera_y_px = (trace.camera_y_pos >> 16) as i32;
    let layout = &level_visuals.level.foreground_layout;

    let col_start = (camera_x_px / 16).max(0) as u16;
    let col_end =
        ((camera_x_px + FRAME_WIDTH as i32 + 15) / 16).min(i32::from(layout.width_cells)) as u16;
    let row_start = (camera_y_px / 16).max(0) as u16;
    let row_end =
        ((camera_y_px + FRAME_HEIGHT as i32 + 15) / 16).min(i32::from(layout.height_cells)) as u16;

    for row in row_start..row_end {
        for col in col_start..col_end {
            let skin = layout.skin_at(col, row);
            let Some(resolved) = level_visuals.level.block_mapping.resolve(skin, None) else {
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

pub(super) fn push_block_grid_overrides(
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

pub(super) fn background_colour(level_visuals: &NativeLevelVisuals) -> [u8; 3] {
    level_visuals
        .palettes
        .background
        .colours
        .first()
        .map(|colour| [colour.red, colour.green, colour.blue])
        .unwrap_or(SKY_COLOUR)
}

pub(super) fn object_rect(
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

pub(super) fn screen_coord(world: u32, camera: u32) -> i32 {
    let world = i64::from(world);
    let camera = i64::from(camera);
    ((world - camera) >> 16) as i32
}

fn build_game_scene_filtered(
    trace: &NativeTraceFrame,
    debug_overlay_visible: bool,
    level_visuals: &NativeLevelVisuals,
    block_grid: Option<&BlockGrid>,
    filter: &LayerFilter,
) -> Scene {
    let background = background_colour(level_visuals);
    let mut commands = Vec::new();

    if filter.background {
        push_background_tiles(&mut commands, trace, level_visuals);
    }
    if filter.terrain {
        push_level_layout_terrain(&mut commands, trace, level_visuals);
    }
    if filter.block_overrides {
        push_block_grid_overrides(&mut commands, trace, level_visuals, block_grid);
    }

    if filter.platforms {
        for object in trace
            .active_objects
            .iter()
            .filter(|object| matches!(object.kind, kid_core::NativeObjectKind::Platform { .. }))
        {
            if let NativeObjectKind::Platform { kind, .. } = object.kind {
                push_platform_tiles(&mut commands, object, trace, level_visuals, kind);
            }
        }
    }
    if filter.enemies {
        for object in trace
            .active_objects
            .iter()
            .filter(|object| matches!(object.kind, kid_core::NativeObjectKind::Enemy { .. }))
        {
            if let NativeObjectKind::Enemy { object_type } = object.kind {
                push_enemy_sprite(&mut commands, object, trace, level_visuals, object_type);
                // Debug: show enemy HP above each enemy
                if debug_overlay_visible && let Some(hp) = object.hit_points {
                    let ex = (object.x_pos >> 16) as i32 - (trace.camera_x_pos >> 16) as i32;
                    let ey = (object.y_pos >> 16) as i32 - (trace.camera_y_pos >> 16) as i32 - 12;
                    commands.push(text(ex - 4, ey, &format!("{hp}"), [255, 255, 100], 1));
                }
            }
        }
    }
    if filter.player {
        // Player: blink every 4 frames during invulnerability
        let player_visible =
            trace.invulnerable_frames == 0 || (trace.frame_index / 4).is_multiple_of(2);
        if player_visible {
            for object in trace
                .active_objects
                .iter()
                .filter(|object| matches!(object.kind, kid_core::NativeObjectKind::Player))
            {
                if trace.death_windup_elapsed > 0 {
                    // ASM Kid_Killed: ~30f freeze, then tumble upward and fall off screen.
                    // Apply a visual Y offset to simulate the tumble animation.
                    let elapsed = trace.death_windup_elapsed;
                    let freeze_frames = 30u16;
                    if elapsed > freeze_frames {
                        let tumble = elapsed - freeze_frames;
                        // Parabolic arc: start at -6px/frame upward, accelerate downward
                        let y_offset =
                            -6i32 * i32::from(tumble) + i32::from(tumble) * i32::from(tumble) / 3;
                        let mut adjusted = object.clone();
                        adjusted.y_pos = (adjusted.y_pos as i64 + ((y_offset as i64) << 16))
                            .clamp(0, i64::from(u32::MAX))
                            as u32;
                        push_player_sprite(&mut commands, &adjusted, trace, level_visuals);
                    } else {
                        push_player_sprite(&mut commands, object, trace, level_visuals);
                    }
                } else {
                    push_player_sprite(&mut commands, object, trace, level_visuals);
                }
            }
        }
    }

    if filter.effects {
        if !trace.level_complete
            && let (Some(flag_x), Some(flag_y)) = (trace.flag_x, trace.flag_y)
        {
            let sx = i32::from(flag_x) - (trace.camera_x_pos >> 16) as i32;
            let pole_base_y = i32::from(flag_y) - (trace.camera_y_pos >> 16) as i32;
            let pole_top = pole_base_y - 32;
            commands.push(DrawCommand::Rect {
                x: sx - 1,
                y: pole_top,
                width: 2,
                height: 32,
                colour: [210, 210, 220],
            });
            let frame_index = (trace.frame_index / level_visuals.flag.frame_duration)
                % level_visuals.flag.waving.len().max(1);
            if let Some(frame) = level_visuals.flag.waving.get(frame_index) {
                push_sprite_frame(
                    &mut commands,
                    frame,
                    &level_visuals.flag.palette,
                    sx,
                    pole_base_y,
                );
            }
            push_sprite_frame(
                &mut commands,
                &level_visuals.flag.pole_bottom,
                &level_visuals.flag.palette,
                sx,
                pole_base_y,
            );
        }

        // Murder wall: layered red wall advancing from the side
        if let Some(wall_x_fp) = trace.murder_wall_x {
            let wall_x_screen = (wall_x_fp >> 16) as i32 - (trace.camera_x_pos >> 16) as i32;
            // Gradient: dark core, lighter leading edge
            commands.push(DrawCommand::Rect {
                x: wall_x_screen - 64,
                y: 0,
                width: 40,
                height: FRAME_HEIGHT as i32,
                colour: [120, 0, 0],
            });
            commands.push(DrawCommand::Rect {
                x: wall_x_screen - 24,
                y: 0,
                width: 16,
                height: FRAME_HEIGHT as i32,
                colour: [200, 30, 10],
            });
            commands.push(DrawCommand::Rect {
                x: wall_x_screen - 8,
                y: 0,
                width: 8,
                height: FRAME_HEIGHT as i32,
                colour: [255, 80, 20],
            });
        }

        // Lava geysers: orange/red eruption columns
        for &(gx, gy_top, height) in &trace.active_geysers {
            let sx = i32::from(gx) - (trace.camera_x_pos >> 16) as i32;
            let sy = i32::from(gy_top) - (trace.camera_y_pos >> 16) as i32;
            // Core column
            commands.push(DrawCommand::Rect {
                x: sx - 4,
                y: sy,
                width: 8,
                height: i32::from(height),
                colour: [255, 120, 20],
            });
            // Hot centre
            commands.push(DrawCommand::Rect {
                x: sx - 2,
                y: sy,
                width: 4,
                height: i32::from(height),
                colour: [255, 200, 60],
            });
        }

        // Projectiles: generic shots stay coloured dots, while the Fire Demon
        // death trail gets its own flame-like footprint. ASM Fire_Demon
        // `loc_3A63C` cycles `stru_3A622`'s six frames; the native player uses
        // a small six-phase rect approximation until those ROM tiles are wired
        // into the scene renderer.
        let projectile_details: Vec<ActiveProjectileSnapshot> =
            if trace.active_projectile_details.is_empty() {
                trace
                    .active_projectiles
                    .iter()
                    .map(|&(x_pos, y_pos, friendly)| ActiveProjectileSnapshot {
                        x_pos,
                        y_pos,
                        friendly,
                        remaining_frames: 0,
                        kind: ActiveProjectileKind::Generic,
                    })
                    .collect()
            } else {
                trace.active_projectile_details.clone()
            };
        for projectile in projectile_details {
            let sx = (projectile.x_pos >> 16) as i32 - (trace.camera_x_pos >> 16) as i32;
            let sy = (projectile.y_pos >> 16) as i32 - (trace.camera_y_pos >> 16) as i32;
            match projectile.kind {
                ActiveProjectileKind::Generic => commands.push(DrawCommand::Rect {
                    x: sx - 2,
                    y: sy - 2,
                    width: 4,
                    height: 4,
                    colour: if projectile.friendly {
                        FRIENDLY_PROJECTILE_COLOUR
                    } else {
                        ENEMY_PROJECTILE_COLOUR
                    },
                }),
                ActiveProjectileKind::FireDemonDeathTrail => {
                    push_fire_demon_trail(&mut commands, sx, sy, projectile.remaining_frames);
                }
            }
        }

        // Metal block bounce animation: block shifts up then back
        for &(cx, cy, remaining) in &trace.block_bounces {
            let bx = i32::from(cx) * 16 - (trace.camera_x_pos >> 16) as i32;
            let bounce_offset = if remaining > 4 {
                -i32::from(8 - remaining) // rise phase
            } else {
                -i32::from(remaining) // fall phase
            };
            let by = i32::from(cy) * 16 - (trace.camera_y_pos >> 16) as i32 + bounce_offset;
            commands.push(DrawCommand::Rect {
                x: bx,
                y: by,
                width: 16,
                height: 16,
                colour: [160, 160, 180],
            });
        }

        // Block destruction fragments: 4 small pieces moving outward
        for &(x_px, y_px, remaining) in &trace.block_destroy_effects {
            let cx = i32::from(x_px) + 8 - (trace.camera_x_pos >> 16) as i32;
            let cy = i32::from(y_px) + 8 - (trace.camera_y_pos >> 16) as i32;
            let spread = i32::from(8 - remaining) * 2;
            let alpha_colour = [200u8.saturating_sub(remaining * 15), 160, 120];
            for &(dx, dy) in &[(-1, -1), (1, -1), (-1, 1), (1, 1)] {
                commands.push(DrawCommand::Rect {
                    x: cx + dx * spread - 2,
                    y: cy + dy * spread - 2,
                    width: 4,
                    height: 4,
                    colour: alpha_colour,
                });
            }
        }

        // Floating prize items: small coloured shape rising upward
        for &(x_px, y_px, remaining, content_id) in &trace.floating_items {
            let fx = i32::from(x_px) - (trace.camera_x_pos >> 16) as i32;
            let fy = i32::from(y_px) - (trace.camera_y_pos >> 16) as i32;
            let alpha = u32::from(remaining).saturating_mul(8).min(255);
            let base_colour = floating_item_colour(content_id);
            let colour = [
                (u32::from(base_colour[0]) * alpha / 255) as u8,
                (u32::from(base_colour[1]) * alpha / 255) as u8,
                (u32::from(base_colour[2]) * alpha / 255) as u8,
            ];
            // Draw content-type-specific pixel-art icon
            // TODO(fidelity): load real floating item sprites from ROM art
            match content_id {
                // Diamond: rotated square (diamond shape)
                0 => {
                    commands.push(DrawCommand::Rect {
                        x: fx - 1,
                        y: fy - 4,
                        width: 2,
                        height: 1,
                        colour,
                    });
                    commands.push(DrawCommand::Rect {
                        x: fx - 3,
                        y: fy - 3,
                        width: 6,
                        height: 4,
                        colour,
                    });
                    commands.push(DrawCommand::Rect {
                        x: fx - 1,
                        y: fy + 1,
                        width: 2,
                        height: 1,
                        colour,
                    });
                }
                // Ankh: cross shape
                1 => {
                    commands.push(DrawCommand::Rect {
                        x: fx - 1,
                        y: fy - 4,
                        width: 2,
                        height: 8,
                        colour,
                    });
                    commands.push(DrawCommand::Rect {
                        x: fx - 3,
                        y: fy - 2,
                        width: 6,
                        height: 2,
                        colour,
                    });
                }
                // Clock: square with dot centre
                2 => {
                    commands.push(DrawCommand::Rect {
                        x: fx - 3,
                        y: fy - 3,
                        width: 6,
                        height: 1,
                        colour,
                    });
                    commands.push(DrawCommand::Rect {
                        x: fx - 3,
                        y: fy + 2,
                        width: 6,
                        height: 1,
                        colour,
                    });
                    commands.push(DrawCommand::Rect {
                        x: fx - 3,
                        y: fy - 2,
                        width: 1,
                        height: 4,
                        colour,
                    });
                    commands.push(DrawCommand::Rect {
                        x: fx + 2,
                        y: fy - 2,
                        width: 1,
                        height: 4,
                        colour,
                    });
                    commands.push(DrawCommand::Rect {
                        x: fx,
                        y: fy - 1,
                        width: 1,
                        height: 2,
                        colour,
                    });
                }
                // Helmets and other items: filled square with highlight
                _ => {
                    commands.push(DrawCommand::Rect {
                        x: fx - 3,
                        y: fy - 3,
                        width: 6,
                        height: 6,
                        colour,
                    });
                    commands.push(DrawCommand::Rect {
                        x: fx - 2,
                        y: fy - 2,
                        width: 2,
                        height: 1,
                        colour: [255, 255, 255],
                    });
                }
            }
        }

        // Enemy death effect: quiver → flatten approximation.
        // HeadyMetal uses a long wobble countdown from the original boss-death routine.
        // Generic enemies: shake horizontally then collapse to a flat pile.
        for &(ex, ey, remaining) in &trace.enemy_death_effects {
            if remaining > GENERIC_ENEMY_DEATH_EFFECT_FRAMES {
                push_heady_metal_death_wobble(
                    &mut commands,
                    trace,
                    level_visuals,
                    ex,
                    ey,
                    remaining,
                );
            } else {
                let sx = (ex >> 16) as i32 - (trace.camera_x_pos >> 16) as i32;
                let sy = (ey >> 16) as i32 - (trace.camera_y_pos >> 16) as i32;
                let total = GENERIC_ENEMY_DEATH_EFFECT_FRAMES;
                let elapsed = total - remaining;
                let quiver_end = total / 2;
                if elapsed < quiver_end {
                    // Quiver phase: shake the corpse rect horizontally
                    let shake = if elapsed.is_multiple_of(2) { 2i32 } else { -2 };
                    let brightness = 180u8.saturating_sub((elapsed as u8) * 10);
                    commands.push(DrawCommand::Rect {
                        x: sx + shake - 6,
                        y: sy - 12,
                        width: 12,
                        height: 16,
                        colour: [brightness, brightness / 2, brightness / 3],
                    });
                } else {
                    // Flatten phase: squash into a flat pile
                    let flatten_progress = elapsed - quiver_end;
                    let height = 16i32.saturating_sub(i32::from(flatten_progress) * 2).max(2);
                    let width = 12 + i32::from(flatten_progress);
                    let brightness =
                        120u8.saturating_sub(((elapsed - quiver_end) as u8).saturating_mul(12));
                    commands.push(DrawCommand::Rect {
                        x: sx - width / 2,
                        y: sy - height,
                        width,
                        height,
                        colour: [brightness, brightness / 3, brightness / 4],
                    });
                }
            }
        }
    }

    if filter.hud {
        push_game_hud(&mut commands, trace, level_visuals);
    }

    // Level Complete overlay
    if trace.level_complete {
        let lc_text = "LEVEL COMPLETE";
        let text_w = lc_text.len() as i32 * 8 * 2;
        let text_x = (FRAME_WIDTH as i32 - text_w) / 2;
        let text_y = (FRAME_HEIGHT as i32) / 2 - 40;
        let panel_w = text_w + 16;
        push_panel(
            &mut commands,
            text_x - 8,
            text_y - 4,
            panel_w,
            88,
            [0, 40, 0],
            [80, 255, 80],
        );
        commands.push(text(text_x, text_y, lc_text, [80, 255, 80], 2));
        let mut line_y = text_y + 22;
        let time_bonus = u32::from(trace.level_timer_seconds) * 100;
        let bonus_line = format!("TIME BONUS: {time_bonus}");
        let bonus_x = (FRAME_WIDTH as i32 - bonus_line.len() as i32 * 8) / 2;
        commands.push(text(bonus_x, line_y, &bonus_line, [200, 255, 200], 1));
        line_y += 12;
        if !trace.level_damage_taken {
            let nohit = "NO-HIT BONUS: 5000";
            let nohit_x = (FRAME_WIDTH as i32 - nohit.len() as i32 * 8) / 2;
            commands.push(text(nohit_x, line_y, nohit, [255, 255, 100], 1));
            line_y += 12;
        }
        let elapsed = 180u16.saturating_sub(trace.level_timer_seconds);
        if elapsed < 60 {
            let speed = "SPEED BONUS: 10000";
            let speed_x = (FRAME_WIDTH as i32 - speed.len() as i32 * 8) / 2;
            commands.push(text(speed_x, line_y, speed, [100, 200, 255], 1));
            line_y += 12;
        }
        let score_line = format!("SCORE: {}", trace.score);
        let score_x = (FRAME_WIDTH as i32 - score_line.len() as i32 * 8) / 2;
        commands.push(text(score_x, line_y, &score_line, [255, 255, 255], 1));
    }

    // Game Over / Continue overlay
    if trace.game_over {
        if let Some(countdown) = trace.continue_countdown {
            // "CONTINUE?" countdown screen — shows remaining time and continues
            let title = "CONTINUE?";
            let title_w = title.len() as i32 * 8 * 2;
            let title_x = (FRAME_WIDTH as i32 - title_w) / 2;
            let title_y = (FRAME_HEIGHT as i32) / 2 - 24;
            let seconds_left = (countdown / 60).min(9);
            let timer_line = format!("{}", seconds_left);
            let timer_w = timer_line.len() as i32 * 8 * 3;
            let timer_x = (FRAME_WIDTH as i32 - timer_w) / 2;
            let count_line = format!("{} CONTINUES LEFT", trace.number_continues);
            let count_w = count_line.len() as i32 * 8;
            let count_x = (FRAME_WIDTH as i32 - count_w) / 2;
            let panel_w = title_w.max(count_w + 16) + 16;
            let panel_x = (FRAME_WIDTH as i32 - panel_w) / 2;
            push_panel(
                &mut commands,
                panel_x,
                title_y - 4,
                panel_w,
                56,
                [0, 40, 80],
                [80, 180, 255],
            );
            commands.push(text(title_x, title_y, title, [80, 180, 255], 2));
            // Blinking countdown number
            if (countdown / 15) % 2 == 0 {
                commands.push(text(timer_x, title_y + 20, &timer_line, [255, 255, 80], 3));
            }
            commands.push(text(count_x, title_y + 44, &count_line, [200, 220, 255], 1));
        } else if trace.number_continues > 0 {
            // "GAME OVER" display before continue screen appears
            let go_text = "GAME OVER";
            let text_w = go_text.len() as i32 * 8 * 2;
            let text_x = (FRAME_WIDTH as i32 - text_w) / 2;
            let text_y = (FRAME_HEIGHT as i32) / 2 - 8;
            push_panel(
                &mut commands,
                text_x - 8,
                text_y - 4,
                text_w + 16,
                24,
                [80, 0, 0],
                [255, 80, 80],
            );
            commands.push(text(text_x, text_y, go_text, [255, 80, 80], 2));
        } else {
            // No continues — final "GAME OVER"
            let go_text = "GAME OVER";
            let text_w = go_text.len() as i32 * 8 * 2;
            let text_x = (FRAME_WIDTH as i32 - text_w) / 2;
            let text_y = (FRAME_HEIGHT as i32) / 2 - 8;
            push_panel(
                &mut commands,
                text_x - 8,
                text_y - 4,
                text_w + 16,
                24,
                [80, 0, 0],
                [255, 80, 80],
            );
            commands.push(text(text_x, text_y, go_text, [255, 80, 80], 2));
        }
    }

    // Pause overlay
    if trace.paused {
        let pause_text = "PAUSED";
        let text_w = pause_text.len() as i32 * 8 * 2;
        let text_x = (FRAME_WIDTH as i32 - text_w) / 2;
        let text_y = (FRAME_HEIGHT as i32) / 2 - 16;
        push_panel(
            &mut commands,
            text_x - 12,
            text_y - 6,
            text_w + 24,
            44,
            [0, 0, 0],
            [255, 255, 255],
        );
        commands.push(text(text_x, text_y, pause_text, [255, 255, 255], 2));
        let hint = "Press Start to resume";
        let hint_w = hint.len() as i32 * 8;
        commands.push(text(
            (FRAME_WIDTH as i32 - hint_w) / 2,
            text_y + 24,
            hint,
            [140, 140, 140],
            1,
        ));
    }

    // Level complete overlay
    if trace.level_complete {
        let complete_text = "LEVEL CLEAR!";
        let text_w = complete_text.len() as i32 * 8 * 2;
        let text_x = (FRAME_WIDTH as i32 - text_w) / 2;
        let text_y = (FRAME_HEIGHT as i32) / 2 - 32;
        push_panel(
            &mut commands,
            text_x - 16,
            text_y - 8,
            text_w + 32,
            80,
            [0, 20, 40],
            [80, 200, 255],
        );
        commands.push(text(text_x, text_y, complete_text, [80, 255, 120], 2));
        let time_text = format!(
            "TIME BONUS: {}s x 100 = {}",
            trace.level_timer_seconds,
            u32::from(trace.level_timer_seconds) * 100
        );
        commands.push(text(
            text_x - 8,
            text_y + 22,
            &time_text,
            [200, 200, 160],
            1,
        ));
        let score_text = format!("TOTAL SCORE: {}", trace.score);
        let score_w = score_text.len() as i32 * 8;
        commands.push(text(
            (FRAME_WIDTH as i32 - score_w) / 2,
            text_y + 38,
            &score_text,
            [255, 255, 200],
            1,
        ));
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

        // Collision boxes: player (green), enemies (red), platforms (blue)
        let cam_x = (trace.camera_x_pos >> 16) as i32;
        let cam_y = (trace.camera_y_pos >> 16) as i32;
        for object in &trace.active_objects {
            let ox = (object.x_pos >> 16) as i32 - cam_x;
            let oy = (object.y_pos >> 16) as i32 - cam_y;
            match object.kind {
                NativeObjectKind::Player => {
                    // Player hitbox: PLAYER_HALF_WIDTH=5, PLAYER_HEIGHT=31
                    push_collision_box(&mut commands, ox - 5, oy - 31, 10, 31, [0, 255, 0]);
                }
                NativeObjectKind::Enemy { .. } => {
                    // Enemy contact box: 8x16 centred on position
                    push_collision_box(&mut commands, ox - 4, oy - 8, 8, 16, [255, 60, 60]);
                }
                NativeObjectKind::Platform { .. } => {
                    // Platform: 32x8 centred on position
                    push_collision_box(&mut commands, ox - 16, oy - 4, 32, 8, [60, 120, 255]);
                }
            }
        }
    }

    Scene {
        background,
        commands,
    }
}

fn push_heady_metal_death_wobble(
    commands: &mut Vec<DrawCommand>,
    trace: &NativeTraceFrame,
    level_visuals: &NativeLevelVisuals,
    x_pos: u32,
    y_pos: u32,
    remaining: u16,
) {
    let screen_x = (x_pos >> 16) as i32 - (trace.camera_x_pos >> 16) as i32;
    let screen_y = (y_pos >> 16) as i32 - (trace.camera_y_pos >> 16) as i32;
    // ASM sub_37708/sub_3787C/sub_378A2 applies wobble velocities
    // -$30D40/$30D40, -$493E0/$493E0, -$61A80/$61A80, -$7A120/$7A120,
    // and -$927C0/$927C0 as the countdown crosses $015E/$00FA/$0096/$0064.
    // Use those stage magnitudes directly instead of the earlier hand-tuned
    // 2..6px approximation so the render path follows the ROM's quiver tiers.
    let amplitude = match remaining {
        r if r > HEADYMETAL_WOBBLE_STAGE_1 => 0x30D40 >> 16,
        r if r > HEADYMETAL_WOBBLE_STAGE_2 => 0x493E0 >> 16,
        r if r > HEADYMETAL_WOBBLE_STAGE_3 => 0x61A80 >> 16,
        r if r > HEADYMETAL_WOBBLE_STAGE_4 => 0x7A120 >> 16,
        _ => 0x927C0 >> 16,
    };
    let phase = if (trace.frame_index & 0x2) == 0 {
        -1
    } else {
        1
    };
    let horizontal = amplitude * phase;
    let vertical = amplitude * -phase;

    push_effect_enemy_sprite(
        commands,
        level_visuals,
        0x20,
        screen_x,
        screen_y + vertical,
        Direction::Right,
        true,
    );
    push_effect_enemy_sprite(
        commands,
        level_visuals,
        0x21,
        screen_x - 12 + horizontal,
        screen_y - 18,
        Direction::Right,
        false,
    );
    push_effect_enemy_sprite(
        commands,
        level_visuals,
        0x21,
        screen_x + 12 - horizontal,
        screen_y - 18,
        Direction::Left,
        false,
    );
}

fn push_effect_enemy_sprite(
    commands: &mut Vec<DrawCommand>,
    level_visuals: &NativeLevelVisuals,
    object_type: u8,
    screen_x: i32,
    screen_y: i32,
    facing: Direction,
    _stationary: bool,
) {
    let Some(enemy_visuals) = level_visuals.enemies.families.get(&object_type) else {
        return;
    };
    let frame = &enemy_visuals.frames[0];
    let (draw_x, draw_y) = sprite_draw_position(frame, screen_x, screen_y, facing);
    commands.push(DrawCommand::Sprite {
        x: draw_x,
        y: draw_y,
        tile_sheet: Arc::new(frame.tile_sheet.clone()),
        palette: Arc::new(enemy_visuals.palette.clone()),
        tiles: build_sprite_tiles(frame, facing, 0),
    });
}

/// Draw a 1px outline rectangle for collision box visualisation.
fn push_collision_box(
    commands: &mut Vec<DrawCommand>,
    x: i32,
    y: i32,
    w: i32,
    h: i32,
    colour: [u8; 3],
) {
    // Top edge
    commands.push(DrawCommand::Rect {
        x,
        y,
        width: w,
        height: 1,
        colour,
    });
    // Bottom edge
    commands.push(DrawCommand::Rect {
        x,
        y: y + h - 1,
        width: w,
        height: 1,
        colour,
    });
    // Left edge
    commands.push(DrawCommand::Rect {
        x,
        y,
        width: 1,
        height: h,
        colour,
    });
    // Right edge
    commands.push(DrawCommand::Rect {
        x: x + w - 1,
        y,
        width: 1,
        height: h,
        colour,
    });
}

/// Returns a distinct RGB colour for each prize content type.
fn floating_item_colour(content_id: u8) -> [u8; 3] {
    match content_id {
        0x00 | 0x0E => [100, 200, 255], // Diamond / TenDiamonds — cyan
        0x01 => [255, 255, 100],        // Points10000 — gold
        0x02 => [200, 200, 255],        // Skycutter — light blue
        0x03 => [180, 255, 180],        // Cyclone — green
        0x04 => [255, 100, 100],        // RedStealth — red
        0x05 => [255, 180, 50],         // Eyeclops — orange
        0x06 => [160, 160, 160],        // Juggernaut — grey
        0x07 => [220, 220, 255],        // IronKnight — silver-blue
        0x08 => [255, 100, 50],         // Berzerker — red-orange
        0x09 => [200, 150, 100],        // Maniaxe — brown
        0x0A => [255, 200, 255],        // Micromax — pink
        0x0B => [255, 255, 200],        // Ankh — pale gold
        0x0C => [200, 255, 200],        // Clock — pale green
        0x0D => [255, 255, 255],        // Continue — white
        _ => [255, 255, 100],           // fallback — gold
    }
}
