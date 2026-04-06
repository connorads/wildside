use super::*;
use crate::scenarios::DynamicCheckpoint;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct TankTurretFrameSpec {
    duration_frames: usize,
    tile_base_index: u16,
    origin_x: i16,
    origin_y: i16,
    width_px: u16,
    height_px: u16,
}

const ROCK_TANK_TURRET_STRU_3B27E: [TankTurretFrameSpec; 5] = [
    TankTurretFrameSpec {
        duration_frames: 0x37,
        // ASM unk_C7EC8 / `stru_3B27E`
        tile_base_index: 0x020,
        origin_x: 0x0F,
        origin_y: 0x00,
        width_px: 0x1E,
        height_px: 0x14,
    },
    TankTurretFrameSpec {
        duration_frames: 0x0A,
        // ASM unk_C7F20 / `stru_3B27E`
        tile_base_index: 0x078,
        origin_x: 0x0F,
        origin_y: 0x00,
        width_px: 0x1E,
        height_px: 0x15,
    },
    TankTurretFrameSpec {
        duration_frames: 0x06,
        // ASM unk_C7F28 / `stru_3B27E`
        tile_base_index: 0x084,
        origin_x: 0x0F,
        origin_y: 0x00,
        width_px: 0x1F,
        height_px: 0x18,
    },
    TankTurretFrameSpec {
        duration_frames: 0x04,
        // ASM unk_C7F20 / `stru_3B27E`
        tile_base_index: 0x078,
        origin_x: 0x0F,
        origin_y: 0x00,
        width_px: 0x1E,
        height_px: 0x15,
    },
    TankTurretFrameSpec {
        duration_frames: 0x05,
        // ASM unk_C7EC8 / `stru_3B27E`
        tile_base_index: 0x020,
        origin_x: 0x0F,
        origin_y: 0x00,
        width_px: 0x1E,
        height_px: 0x14,
    },
];

const ROCK_TANK_SHOOTING_FAST_STRU_3B4D8: [TankTurretFrameSpec; 7] = [
    TankTurretFrameSpec {
        duration_frames: 0x37,
        // ASM unk_C7EC8 / `stru_3B4D8`
        tile_base_index: 0x020,
        origin_x: 0x0F,
        origin_y: 0x00,
        width_px: 0x1E,
        height_px: 0x14,
    },
    TankTurretFrameSpec {
        duration_frames: 0x0C,
        // ASM unk_C7F00 / `stru_3B4D8`
        tile_base_index: 0x048,
        origin_x: 0x0F,
        origin_y: 0x00,
        width_px: 0x1E,
        height_px: 0x15,
    },
    TankTurretFrameSpec {
        duration_frames: 0x06,
        // ASM unk_C7F08 / `stru_3B4D8`
        tile_base_index: 0x054,
        origin_x: 0x0F,
        origin_y: 0x00,
        width_px: 0x1F,
        height_px: 0x18,
    },
    TankTurretFrameSpec {
        duration_frames: 0x06,
        // ASM unk_C7F10 / `stru_3B4D8`
        tile_base_index: 0x060,
        origin_x: 0x0F,
        origin_y: 0x00,
        width_px: 0x1F,
        height_px: 0x18,
    },
    TankTurretFrameSpec {
        duration_frames: 0x14,
        // ASM unk_C7F08 / `stru_3B4D8`
        tile_base_index: 0x054,
        origin_x: 0x0F,
        origin_y: 0x00,
        width_px: 0x1F,
        height_px: 0x18,
    },
    TankTurretFrameSpec {
        duration_frames: 0x0C,
        // ASM unk_C7F00 / `stru_3B4D8`
        tile_base_index: 0x048,
        origin_x: 0x0F,
        origin_y: 0x00,
        width_px: 0x1E,
        height_px: 0x15,
    },
    TankTurretFrameSpec {
        duration_frames: 0x05,
        // ASM unk_C7EC8 / `stru_3B4D8`
        tile_base_index: 0x020,
        origin_x: 0x0F,
        origin_y: 0x00,
        width_px: 0x1E,
        height_px: 0x14,
    },
];

const ROCK_TANK_SHOOTING_SLOW_STRU_3B4F6: [TankTurretFrameSpec; 8] = [
    TankTurretFrameSpec {
        duration_frames: 0x37,
        // ASM unk_C7EC8 / `stru_3B4F6`
        tile_base_index: 0x020,
        origin_x: 0x0F,
        origin_y: 0x00,
        width_px: 0x1E,
        height_px: 0x14,
    },
    TankTurretFrameSpec {
        duration_frames: 0x0C,
        // ASM unk_C7F00 / `stru_3B4F6`
        tile_base_index: 0x048,
        origin_x: 0x0F,
        origin_y: 0x00,
        width_px: 0x1E,
        height_px: 0x15,
    },
    TankTurretFrameSpec {
        duration_frames: 0x06,
        // ASM unk_C7F08 / `stru_3B4F6`
        tile_base_index: 0x054,
        origin_x: 0x0F,
        origin_y: 0x00,
        width_px: 0x1F,
        height_px: 0x18,
    },
    TankTurretFrameSpec {
        duration_frames: 0x06,
        // ASM unk_C7F10 / `stru_3B4F6`
        tile_base_index: 0x060,
        origin_x: 0x0F,
        origin_y: 0x00,
        width_px: 0x1F,
        height_px: 0x18,
    },
    TankTurretFrameSpec {
        duration_frames: 0x14,
        // ASM unk_C7F08 / `stru_3B4F6`
        tile_base_index: 0x054,
        origin_x: 0x0F,
        origin_y: 0x00,
        width_px: 0x1F,
        height_px: 0x18,
    },
    TankTurretFrameSpec {
        duration_frames: 0x0C,
        // ASM unk_C7F00 / `stru_3B4F6`
        tile_base_index: 0x048,
        origin_x: 0x0F,
        origin_y: 0x00,
        width_px: 0x1E,
        height_px: 0x15,
    },
    TankTurretFrameSpec {
        duration_frames: 0x05,
        // ASM unk_C7EC8 / `stru_3B4F6`
        tile_base_index: 0x020,
        origin_x: 0x0F,
        origin_y: 0x00,
        width_px: 0x1E,
        height_px: 0x14,
    },
    TankTurretFrameSpec {
        duration_frames: 0x74,
        // ASM unk_C7EC8 / `stru_3B4F6`
        tile_base_index: 0x020,
        origin_x: 0x0F,
        origin_y: 0x00,
        width_px: 0x1E,
        height_px: 0x14,
    },
];

pub(super) fn build_player_sprite_tiles(
    frame: &kid_render::SpriteFrame,
    facing: Direction,
) -> Vec<SpriteTile> {
    build_sprite_tiles(frame, facing, 0)
}

pub(super) fn align_sprite_dimension(px: u16) -> u16 {
    px.div_ceil(8) * 8
}

pub(super) fn sprite_draw_position(
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
    (
        draw_x,
        screen_y - i32::from(frame.height_px) + i32::from(frame.origin_y),
    )
}

pub(super) fn build_sprite_tiles(
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

pub(super) fn append_sprite_band_tiles(
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

pub(super) fn append_sprite_chunk_tiles(
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

    for column in 0..columns {
        for row in 0..rows {
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

pub(super) fn select_player_frame<'a>(
    trace: &NativeTraceFrame,
    player_visuals: &'a kid_render::PlayerFormVisuals,
) -> &'a kid_render::SpriteFrame {
    let on_ground = trace.kid_on_ground.unwrap_or(true);
    let x_velocity = trace.kid_x_velocity.unwrap_or(0);
    let y_velocity = trace.kid_y_velocity.unwrap_or(0);

    if on_ground {
        if x_velocity == 0 {
            &player_visuals.idle
        } else if trace.kid_on_slope != 0 {
            // ASM stru_8B6A: use pre-drawn tilted slope walk frames.
            // The left-facing art is drawn leaning left (for uphill-right),
            // the right-facing art leans right (for uphill-left). Select
            // based on facing — the renderer applies H-flip separately.
            let facing = trace.kid_facing.unwrap_or(Direction::Right);
            let slope_frames = match facing {
                Direction::Right if !player_visuals.slope_walk_left.is_empty() => {
                    &player_visuals.slope_walk_left
                }
                Direction::Left if !player_visuals.slope_walk.is_empty() => {
                    &player_visuals.slope_walk
                }
                _ => &player_visuals.walk,
            };
            let frame_index = (trace.frame_index / 4) % slope_frames.len();
            &slope_frames[frame_index]
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

pub(super) fn select_player_form_visuals<'a>(
    trace: &NativeTraceFrame,
    player_visuals: &'a kid_render::PlayerVisuals,
) -> &'a kid_render::PlayerFormVisuals {
    player_visuals
        .forms
        .get(&trace.current_helmet_id)
        .or_else(|| player_visuals.forms.get(&0))
        .expect("Kid visuals should always be loaded")
}

pub(super) fn select_player_transform_frame<'a>(
    trace: &NativeTraceFrame,
    player_visuals: &'a kid_render::PlayerFormVisuals,
) -> Option<&'a kid_render::SpriteFrame> {
    if trace.form_change_flash_frames == 0 || player_visuals.transform_steps.is_empty() {
        return None;
    }

    let total_duration = player_visuals
        .transform_steps
        .iter()
        .map(|step| step.duration_frames)
        .sum::<usize>();
    let mut elapsed = total_duration.saturating_sub(usize::from(trace.form_change_flash_frames));
    for step in &player_visuals.transform_steps {
        if elapsed < step.duration_frames {
            return step.frame.as_ref();
        }
        elapsed -= step.duration_frames;
    }

    player_visuals
        .transform_steps
        .iter()
        .rev()
        .find_map(|step| step.frame.as_ref())
}

pub(super) fn player_screen_position(
    object: &NativeObjectSnapshot,
    trace: &NativeTraceFrame,
) -> (i32, i32) {
    (
        (object.x_pos >> 16) as i32 - (trace.camera_x_pos >> 16) as i32,
        (object.y_pos >> 16) as i32 - (trace.camera_y_pos >> 16) as i32,
    )
}

pub(super) fn push_player_sprite(
    commands: &mut Vec<DrawCommand>,
    object: &NativeObjectSnapshot,
    trace: &NativeTraceFrame,
    level_visuals: &NativeLevelVisuals,
) {
    let form_visuals = select_player_form_visuals(trace, &level_visuals.player);
    let facing = trace.kid_facing.unwrap_or(Direction::Right);
    let (screen_x, screen_y) = player_screen_position(object, trace);
    if let Some(frame) = select_player_transform_frame(trace, form_visuals) {
        let (draw_x, draw_y) = sprite_draw_position(frame, screen_x, screen_y, facing);
        commands.push(DrawCommand::Sprite {
            x: draw_x,
            y: draw_y,
            tile_sheet: Arc::new(frame.tile_sheet.clone()),
            palette: Arc::new(
                form_visuals
                    .transform_palette
                    .clone()
                    .unwrap_or_else(|| form_visuals.palette.clone()),
            ),
            tiles: build_player_sprite_tiles(frame, facing),
        });
        return;
    }

    let frame = select_player_frame(trace, form_visuals);
    let (draw_x, draw_y) = sprite_draw_position(frame, screen_x, screen_y, facing);

    commands.push(DrawCommand::Sprite {
        x: draw_x,
        y: draw_y,
        tile_sheet: Arc::new(frame.tile_sheet.clone()),
        palette: Arc::new(form_visuals.palette.clone()),
        tiles: build_player_sprite_tiles(frame, facing),
    });
}

pub(super) fn select_enemy_frame<'a>(
    trace: &NativeTraceFrame,
    enemy_visuals: &'a kid_render::AnimatedSpriteVisual,
    object: &NativeObjectSnapshot,
) -> &'a kid_render::SpriteFrame {
    if object.stationary {
        return &enemy_visuals.frames[0];
    }
    // Offset animation by layout_index so enemies of the same type don't
    // animate in lockstep. ASM uses per-object animation counters at
    // $36(a3)/$38(a3); this is a visual approximation.
    let offset = object.layout_index.unwrap_or(0);
    let frame_index = ((trace.frame_index + offset * 3) / enemy_visuals.frame_duration.max(1))
        % enemy_visuals.frames.len();
    &enemy_visuals.frames[frame_index]
}

pub(super) fn push_enemy_sprite(
    commands: &mut Vec<DrawCommand>,
    object: &NativeObjectSnapshot,
    trace: &NativeTraceFrame,
    level_visuals: &NativeLevelVisuals,
    object_type: u8,
) {
    let enemy_visuals = match level_visuals.enemies.families.get(&object_type) {
        Some(vis) => vis,
        None => {
            let colour = if object.damage_flash {
                ENEMY_FLASH_COLOUR
            } else {
                enemy_family_colour(object_type)
            };
            commands.push(object_rect(object, trace, 16, 16, colour));
            return;
        }
    };

    let frame = select_enemy_frame(trace, enemy_visuals, object);
    let (screen_x, screen_y) = player_screen_position(object, trace);
    let facing = object.facing.unwrap_or(Direction::Right);
    // Enemy VRAM art is stored facing left (the ROM uses the H-flip bit to
    // show right-facing sprites). Invert the facing for rendering so that
    // Direction::Right flips the left-facing art to face right.
    let render_facing = match facing {
        Direction::Right => Direction::Left,
        Direction::Left => Direction::Right,
    };
    let (draw_x, draw_y) = sprite_draw_position(frame, screen_x, screen_y, render_facing);
    commands.push(DrawCommand::Sprite {
        x: draw_x,
        y: draw_y,
        tile_sheet: Arc::new(frame.tile_sheet.clone()),
        palette: Arc::new(select_enemy_palette(enemy_visuals, object.palette_variant).clone()),
        tiles: build_sprite_tiles(frame, render_facing, 0),
    });

    if let Some(turret_frame) = rock_tank_turret_frame(object, trace, enemy_visuals, object_type) {
        let turret_screen_y = screen_y - 8;
        let (turret_x, turret_y) =
            sprite_draw_position(&turret_frame, screen_x, turret_screen_y, render_facing);
        commands.push(DrawCommand::Sprite {
            x: turret_x,
            y: turret_y,
            tile_sheet: Arc::new(turret_frame.tile_sheet.clone()),
            palette: Arc::new(select_enemy_palette(enemy_visuals, object.palette_variant).clone()),
            tiles: build_sprite_tiles(&turret_frame, render_facing, 0),
        });
    }
}

fn rock_tank_turret_frame(
    object: &NativeObjectSnapshot,
    trace: &NativeTraceFrame,
    enemy_visuals: &kid_render::AnimatedSpriteVisual,
    object_type: u8,
) -> Option<kid_render::SpriteFrame> {
    let specs = match object_type {
        // ASM Tank.asm loc_3B372 + `stru_3B27E`: plain RockTank allocates a
        // second GfxObject at `$3A(a5)` for the turret and keeps it 8px above
        // the body while the body animation uses the first object.
        0x0A => &ROCK_TANK_TURRET_STRU_3B27E[..],
        // ASM Tank_Shooting.asm loc_3B5FA/loc_3B682: HP 4 uses `stru_3B4D8`,
        // HP 2-3 use `stru_3B4F6`, again on the child GfxObject at `$3A(a5)`.
        0x0B if object.hit_points == Some(4) => &ROCK_TANK_SHOOTING_FAST_STRU_3B4D8[..],
        0x0B => &ROCK_TANK_SHOOTING_SLOW_STRU_3B4F6[..],
        _ => return None,
    };
    let spec = scripted_tank_turret_frame(object, trace, specs)?;
    let tile_sheet = enemy_visuals.frames.first()?.tile_sheet.clone();
    Some(kid_render::SpriteFrame {
        origin_x: spec.origin_x,
        origin_y: spec.origin_y,
        width_px: spec.width_px,
        height_px: spec.height_px,
        tile_base_index: spec.tile_base_index,
        tile_sheet,
    })
}

fn scripted_tank_turret_frame(
    object: &NativeObjectSnapshot,
    trace: &NativeTraceFrame,
    specs: &[TankTurretFrameSpec],
) -> Option<TankTurretFrameSpec> {
    let cycle_frames = specs.iter().map(|spec| spec.duration_frames).sum::<usize>();
    if cycle_frames == 0 {
        return None;
    }
    let phase_seed = trace
        .frame_index
        .wrapping_add(object.layout_index.unwrap_or(0).wrapping_mul(3));
    let mut phase = phase_seed % cycle_frames;
    for spec in specs {
        if phase < spec.duration_frames {
            return Some(*spec);
        }
        phase -= spec.duration_frames;
    }
    specs.last().copied()
}

fn select_enemy_palette(
    visuals: &kid_render::AnimatedSpriteVisual,
    variant: u8,
) -> &kid_render::Palette {
    if variant > 0
        && let Some(pal) = visuals.extra_palettes.get(usize::from(variant) - 1)
    {
        return pal;
    }
    &visuals.palette
}

pub(super) fn push_platform_tiles(
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
        y: screen_y - i32::from(frame.height_px) + i32::from(frame.origin_y),
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
