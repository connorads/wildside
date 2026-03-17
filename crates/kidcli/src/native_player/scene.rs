use super::*;

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
        GameMode::InGame => build_game_scene_filtered(
            trace,
            debug_overlay_visible,
            level_visuals.expect("Level 1 visuals are required for gameplay scenes"),
            block_grid,
            filter,
        ),
        GameMode::LevelTransition => build_transition_scene(trace),
        GameMode::OptionsMenu => build_options_scene(trace),
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
    let level_text = format!("{}", level_name);
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
        GameMode::IntroSequence1 | GameMode::IntroSequence2 | GameMode::IntroSequence3
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
    let two_p_label = if trace.two_player {
        "2 PLAYERS: ON"
    } else {
        "2 PLAYERS: OFF"
    };
    let items = [two_p_label, "SOUND TEST", "EXIT"];
    for (i, item) in items.iter().enumerate() {
        let y = 80 + i as i32 * 24;
        let colour = if i as i32 == cursor {
            [255, 255, 255]
        } else {
            [120, 120, 120]
        };
        commands.push(text(100, y, item, colour, 1));
        if i as i32 == cursor {
            commands.push(text(80, y, ">", [255, 220, 80], 1));
        }
    }

    commands.push(text(
        60,
        172,
        "Up/Down select  A/C toggle  Start/B back",
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

pub(super) fn background_scroll_x_px(
    trace: &NativeTraceFrame,
    camera_x_px: i32,
    layer_speed: u8,
) -> i32 {
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
        push_level_layout_terrain(&mut commands, trace, level_visuals, block_grid);
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
                if debug_overlay_visible {
                    if let Some(hp) = object.hit_points {
                        let ex = (object.x_pos >> 16) as i32 - (trace.camera_x_pos >> 16) as i32;
                        let ey =
                            (object.y_pos >> 16) as i32 - (trace.camera_y_pos >> 16) as i32 - 12;
                        commands.push(text(ex - 4, ey, &format!("{hp}"), [255, 255, 100], 1));
                    }
                }
            }
        }
    }
    if filter.player {
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
    }

    if filter.effects {
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

        // Projectiles: coloured dots (green=friendly, red=enemy)
        for &(px, py, friendly) in &trace.active_projectiles {
            let sx = (px >> 16) as i32 - (trace.camera_x_pos >> 16) as i32;
            let sy = (py >> 16) as i32 - (trace.camera_y_pos >> 16) as i32;
            let colour = if friendly {
                [100, 255, 100]
            } else {
                [255, 80, 40]
            };
            commands.push(DrawCommand::Rect {
                x: sx - 2,
                y: sy - 2,
                width: 4,
                height: 4,
                colour,
            });
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
        for &(x_px, y_px, remaining) in &trace.floating_items {
            let fx = i32::from(x_px) - (trace.camera_x_pos >> 16) as i32;
            let fy = i32::from(y_px) - (trace.camera_y_pos >> 16) as i32;
            let brightness = u8::try_from(u16::from(remaining) * 12).unwrap_or(255);
            commands.push(DrawCommand::Rect {
                x: fx - 3,
                y: fy - 3,
                width: 6,
                height: 6,
                colour: [brightness, brightness, 50],
            });
        }

        // Enemy death particle burst: expanding ring of particles
        for &(ex, ey, remaining) in &trace.enemy_death_effects {
            let sx = (ex >> 16) as i32 - (trace.camera_x_pos >> 16) as i32;
            let sy = (ey >> 16) as i32 - (trace.camera_y_pos >> 16) as i32;
            let spread = i32::from(10 - remaining) * 2;
            let brightness = u8::try_from(u16::from(remaining) * 25).unwrap_or(255);
            let colour = [255, brightness, brightness.saturating_sub(50)];
            for angle in 0..8 {
                let (dx, dy) = match angle {
                    0 => (spread, 0),
                    1 => (spread * 7 / 10, spread * 7 / 10),
                    2 => (0, spread),
                    3 => (-spread * 7 / 10, spread * 7 / 10),
                    4 => (-spread, 0),
                    5 => (-spread * 7 / 10, -spread * 7 / 10),
                    6 => (0, -spread),
                    _ => (spread * 7 / 10, -spread * 7 / 10),
                };
                commands.push(DrawCommand::Rect {
                    x: sx + dx - 1,
                    y: sy + dy - 1,
                    width: 3,
                    height: 3,
                    colour,
                });
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
        let text_y = (FRAME_HEIGHT as i32) / 2 - 24;
        let panel_w = text_w + 16;
        push_panel(
            &mut commands,
            text_x - 8,
            text_y - 4,
            panel_w,
            56,
            [0, 40, 0],
            [80, 255, 80],
        );
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
            push_panel(
                &mut commands,
                panel_x,
                title_y - 4,
                panel_w,
                40,
                [0, 40, 80],
                [80, 180, 255],
            );
            commands.push(text(title_x, title_y, title, [80, 180, 255], 2));
            commands.push(text(count_x, title_y + 24, &count_line, [200, 220, 255], 1));
        } else {
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

    // Game over overlay
    if trace.game_over {
        let overlay_text = if trace.number_continues > 0 {
            "CONTINUE?"
        } else {
            "GAME OVER"
        };
        let text_w = overlay_text.len() as i32 * 8 * 2;
        let text_x = (FRAME_WIDTH as i32 - text_w) / 2;
        let text_y = (FRAME_HEIGHT as i32) / 2 - 16;
        push_panel(
            &mut commands,
            text_x - 12,
            text_y - 8,
            text_w + 24,
            48,
            [40, 0, 0],
            [255, 80, 80],
        );
        commands.push(text(text_x, text_y, overlay_text, [255, 80, 80], 2));
        let score_text = format!("SCORE: {}", trace.score);
        let score_w = score_text.len() as i32 * 8;
        commands.push(text(
            (FRAME_WIDTH as i32 - score_w) / 2,
            text_y + 22,
            &score_text,
            [200, 200, 200],
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
