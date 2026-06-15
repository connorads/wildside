use super::*;

const HEADYMETAL_WOBBLE_STAGE_1: u16 = 0x015E;
const HEADYMETAL_WOBBLE_STAGE_2: u16 = 0x00FA;
const HEADYMETAL_WOBBLE_STAGE_3: u16 = 0x0096;
const HEADYMETAL_WOBBLE_STAGE_4: u16 = 0x0064;
const FRIENDLY_PROJECTILE_COLOUR: [u8; 3] = [100, 255, 100];
const ENEMY_PROJECTILE_COLOUR: [u8; 3] = [255, 80, 40];
const SHOOTER_BLOCK_PROJECTILE_COLOUR: [u8; 3] = [255, 216, 72];
const FIRE_TRAIL_PLACEHOLDER_COLOUR: [u8; 3] = [255, 120, 32];
const FIRE_DEMON_DEATH_TRAIL_NATIVE_LIFETIME: u16 = 45;
const DRAGON_WALKING_BREATH_LIFETIME: u16 = 68;
const ROBOT_SHOT_IMPACT_LIFETIME_LOC_3A762: u16 = 3 + 4 + 8;
const VDP_SPRITE_SCREEN_BIAS_LOC_13C0: i32 = 0x80;
const MURDER_WALL_FALLBACK_BASE_COLOUR: [u8; 3] = [190, 20, 8];
const MURDER_WALL_FALLBACK_SHADOW_COLOUR: [u8; 3] = [65, 10, 10];
const MURDER_WALL_FALLBACK_HIGHLIGHT_COLOUR: [u8; 3] = [255, 160, 70];
const FIRE_DEMON_OBJECT_TYPE: u8 = 0x00;
const ROBOT_OBJECT_TYPE: u8 = 0x03;
const FLYING_DRAGON_OBJECT_TYPE: u8 = 0x0C;
const WALKING_DRAGON_OBJECT_TYPE: u8 = 0x0D;
const DRIPS_OBJECT_TYPE: u8 = 0x16;
const TAR_MONSTER_OBJECT_TYPE: u8 = 0x05;
const PRIZE_GENERIC_LIFETIME_LOW_BYTE: u8 = 0x20;
const PRIZE_BLOCK_SPAWN_FLASH_TOTAL_FRAMES_LOC_102B8: u8 = 25;
// ASM loc_D734: after loc_D702 allocates the EOL row workers, the ROM runs
// eight one-frame palette fade steps before the score rows are visible in the
// reduced oracle bundle. The trace clock keeps loc_D468 object timing; the
// renderer applies this final visual fade phase.
pub(super) const EOL_SCORE_ROWS_RENDER_FADE_IN_FRAMES_LOC_D734_ORACLE: usize = 8;
// ASM loc_D734 falls through after the fade, then loc_C5A2/loc_C48C workers
// update before sub_13C0 latches the sprite table for the *next* displayed
// frame. The reduced `issue-175-eol-flag-touch-level12` oracle bundle exposes
// the visible frame through `eol_pre_frame_vdp_pixel_samples_loc_c070`, which
// is 15 worker ticks ahead of the trace's post-frame row counter.
const EOL_SCORE_ROWS_RENDER_LATCH_LEAD_FRAMES_LOC_C070_ORACLE: usize = 15;
const CONTINUE_SCREEN_INITIAL_HOLD_FRAMES_LOC_D31E: usize = 0x1C;
const PAUSE_MENU_TEXT_X_LOC_69D2: i32 = 0x78;
const PAUSE_MENU_TEXT_TOP_Y_RESTART_TEXT: i32 = 0x48;
const PAUSE_MENU_TEXT_BOTTOM_Y_RESTART_TEXT: i32 = 0x58;
const PAUSE_MENU_CURSOR_X_LOC_6A84: i32 = 0x68;
const PRIZE_BLOCK_SPAWN_FLASH_FRAMES_STRU_102DE: [(u8, u16); 4] =
    [(4, 0x02CF), (6, 0x02D8), (10, 0x02E1), (4, 0x02EA)];
const KID_PROJECTILE_HIT_SPARK_FRAMES_STRU_1F40: [(u8, u16); 4] =
    [(1, 0x02CF), (1, 0x02D8), (1, 0x02E1), (1, 0x02EA)];
const BLOCK_DEBRIS_TILE_ATTR_SUB_2A4C: u16 = 0x8228;
const BLOCK_DEBRIS_BASE_X_SUB_292E: [i16; 5] = [0, 0, 8, 8, 4];
const BLOCK_DEBRIS_BASE_Y_SUB_292E: [i16; 5] = [0, 8, 8, 0, 4];
const BLOCK_DEBRIS_VEL_D6_2_SUB_292E: [(i16, i16); 5] =
    [(-1, -4), (-1, -3), (1, -3), (1, -4), (1, -5)];
const BLOCK_DEBRIS_VEL_D6_0_SUB_292E: [(i16, i16); 5] =
    [(-1, -2), (-2, 0), (1, 0), (1, -1), (-1, -2)];
const BLOCK_DEBRIS_VEL_D6_1_SUB_292E: [(i16, i16); 5] =
    [(-5, -3), (-4, -3), (-3, -3), (-2, -3), (-1, -3)];
const BLOCK_DEBRIS_VEL_D6_DEFAULT_SUB_292E: [(i16, i16); 5] =
    [(5, -3), (4, -3), (3, -3), (2, -3), (1, -3)];
const LAVA_GEYSER_TOP_TEMPLATE_UNK_14624: [kid_core::LavaGeyserSpriteTemplateLoc14BE8; 2] = [
    kid_core::LavaGeyserSpriteTemplateLoc14BE8 {
        x_offset_px: 0x0001,
        y_step_px: 0x0008,
        tile_base_word: 0xA33F,
        size_word: 0x0C00,
    },
    kid_core::LavaGeyserSpriteTemplateLoc14BE8 {
        x_offset_px: 0x0000,
        y_step_px: 0x0020,
        tile_base_word: 0xA399,
        size_word: 0x0F00,
    },
];
const LAVA_GEYSER_MIDDLE_TEMPLATE_UNK_1465A: [kid_core::LavaGeyserSpriteTemplateLoc14BE8; 1] =
    [kid_core::LavaGeyserSpriteTemplateLoc14BE8 {
        x_offset_px: 0x0001,
        y_step_px: 0x0010,
        tile_base_word: 0xA36E,
        size_word: 0x0D00,
    }];
const LAVA_GEYSER_BOTTOM_TEMPLATE_UNK_14650: [kid_core::LavaGeyserSpriteTemplateLoc14BE8; 1] =
    [kid_core::LavaGeyserSpriteTemplateLoc14BE8 {
        x_offset_px: 0x0002,
        y_step_px: 0x0010,
        tile_base_word: 0xA3B1,
        size_word: 0x0D00,
    }];

const FIRE_DEMON_DEATH_TRAIL_FRAMES: [kid_render::SpriteFrame; 6] = [
    fire_demon_trail_frame(0x0B0, 0x07, 0x00, 0x0E, 0x08),
    fire_demon_trail_frame(0x0B2, 0x07, 0x00, 0x0D, 0x07),
    fire_demon_trail_frame(0x0B4, 0x06, 0x00, 0x0C, 0x06),
    fire_demon_trail_frame(0x0B6, 0x07, 0x00, 0x0E, 0x08),
    fire_demon_trail_frame(0x0B8, 0x07, 0x00, 0x0D, 0x08),
    fire_demon_trail_frame(0x0BA, 0x06, 0x00, 0x0C, 0x06),
];

const fn fire_demon_trail_frame(
    tile_base_index: u16,
    origin_x: i16,
    origin_y: i16,
    width_px: u16,
    height_px: u16,
) -> kid_render::SpriteFrame {
    kid_render::SpriteFrame {
        origin_x,
        origin_y,
        width_px,
        height_px,
        tile_base_index,
        tile_sheet: kid_render::TileSheet { tiles: Vec::new() },
    }
}

fn drips_drop_child_frame(
    visuals: &kid_render::AnimatedSpriteVisual,
) -> Option<kid_render::SpriteFrame> {
    let tile_sheet = visuals.frames.first()?.tile_sheet.clone();
    Some(kid_render::SpriteFrame {
        // ASM Drips.asm loc_33424 writes `LnkTo_unk_C7B26` to the falling
        // child. `unk_C7B26` is `sprite_frame_vram $082,$02,$09,$05,$0A`.
        origin_x: 0x02,
        origin_y: 0x09,
        width_px: 0x05,
        height_px: 0x0A,
        tile_base_index: 0x082,
        tile_sheet,
    })
}

fn tar_monster_glob_child_frame(
    visuals: &kid_render::AnimatedSpriteVisual,
) -> Option<kid_render::SpriteFrame> {
    let tile_sheet = visuals.frames.first()?.tile_sheet.clone();
    Some(kid_render::SpriteFrame {
        // ASM Tar_Monster.asm loc_3247E writes `LnkTo_unk_C781E` to the straight
        // glob. `unk_C781E` is `sprite_frame_vram $0BC,$04,$04,$1E,$08` (decoded
        // from kid.bin @ $C781E). This is the dark tar comet, not a red dot.
        origin_x: 0x04,
        origin_y: 0x04,
        width_px: 0x1E,
        height_px: 0x08,
        tile_base_index: 0x0BC,
        tile_sheet,
    })
}

fn tar_monster_burst_child_frame(
    visuals: &kid_render::AnimatedSpriteVisual,
) -> Option<kid_render::SpriteFrame> {
    let tile_sheet = visuals.frames.first()?.tile_sheet.clone();
    Some(kid_render::SpriteFrame {
        // ASM Tar_Monster.asm loc_325E6 writes `LnkTo_unk_C782E` to each
        // retaliation fragment. `unk_C782E` is `sprite_frame_vram
        // $0C1,$03,$03,$06,$06` (decoded from kid.bin @ $C782E) — a small dark
        // tar blob.
        origin_x: 0x03,
        origin_y: 0x03,
        width_px: 0x06,
        height_px: 0x06,
        tile_base_index: 0x0C1,
        tile_sheet,
    })
}

fn tank_bullet_child_frame(
    visuals: &kid_render::AnimatedSpriteVisual,
    remaining_frames: u16,
) -> Option<kid_render::SpriteFrame> {
    let tile_sheet = visuals.frames.first()?.tile_sheet.clone();
    let elapsed_frames = u16::MAX.saturating_sub(remaining_frames);
    let (tile_base_index, origin_x, width_px) = if elapsed_frames % 10 < 6 {
        // ASM Tank_Shooting.asm `stru_3B526`: first frame `unk_C7EE8`.
        (0x038, 0x09, 0x10)
    } else {
        // ASM Tank_Shooting.asm `stru_3B526`: second frame `unk_C7EF0`.
        (0x03A, 0x09, 0x0F)
    };
    Some(kid_render::SpriteFrame {
        origin_x,
        origin_y: 0x00,
        width_px,
        height_px: 0x05,
        tile_base_index,
        tile_sheet,
    })
}

fn robot_shot_child_frame(
    visuals: &kid_render::AnimatedSpriteVisual,
    remaining_frames: u16,
    animation_elapsed_frames: u16,
    impact: bool,
) -> Option<kid_render::SpriteFrame> {
    let tile_sheet = visuals.frames.first()?.tile_sheet.clone();
    let (tile_base_index, origin_x, origin_y, width_px, height_px) = if impact {
        let elapsed_frames = ROBOT_SHOT_IMPACT_LIFETIME_LOC_3A762.saturating_sub(remaining_frames);
        if elapsed_frames < 3 {
            // ASM Robot.asm `stru_3A762`: first impact frame `LnkTo_unk_C8260`.
            (0x000, 0x08, 0x00, 0x10, 0x10)
        } else if elapsed_frames < 7 {
            // ASM Robot.asm `stru_3A762`: second impact frame `LnkTo_unk_C8268`.
            (0x004, 0x0A, 0x00, 0x14, 0x14)
        } else {
            // ASM Robot.asm `stru_3A762`: final impact frame `LnkTo_unk_C8270`.
            (0x00D, 0x0B, 0x00, 0x16, 0x16)
        }
    } else {
        match (animation_elapsed_frames / 3) % 3 {
            // ASM Robot.asm `stru_3A754`: first flying frame `LnkTo_unk_C8278`.
            0 => (0x016, 0x03, 0x00, 0x1F, 0x06),
            // ASM Robot.asm `stru_3A754`: second flying frame `LnkTo_unk_C8280`.
            1 => (0x01A, 0x03, 0x00, 0x1F, 0x06),
            // ASM Robot.asm `stru_3A754`: third flying frame `LnkTo_unk_C8288`.
            _ => (0x01E, 0x03, 0x00, 0x17, 0x06),
        }
    };
    Some(kid_render::SpriteFrame {
        origin_x,
        origin_y,
        width_px,
        height_px,
        tile_base_index,
        tile_sheet,
    })
}

fn dragon_walking_breath_child_frame(
    visuals: &kid_render::AnimatedSpriteVisual,
    remaining_frames: u16,
) -> Option<kid_render::SpriteFrame> {
    let tile_sheet = visuals.frames.first()?.tile_sheet.clone();
    let elapsed_frames = DRAGON_WALKING_BREATH_LIFETIME.saturating_sub(remaining_frames);
    let (tile_base_index, origin_x, origin_y, width_px, height_px) = if elapsed_frames < 8 {
        // ASM Dragon.asm `stru_3CCC8`: first frame `LnkTo_unk_C7576`.
        (0x028, 0x0B, 0x00, 0x1B, 0x13)
    } else if ((elapsed_frames - 8) / 10).is_multiple_of(2) {
        // ASM Dragon.asm `stru_3CCC8`: alternating frame `LnkTo_unk_C757E`.
        (0x034, 0x0A, 0x00, 0x20, 0x15)
    } else {
        // ASM Dragon.asm `stru_3CCC8`: alternating frame `LnkTo_unk_C7586`.
        (0x040, 0x0A, 0x00, 0x20, 0x16)
    };
    Some(kid_render::SpriteFrame {
        origin_x,
        origin_y,
        width_px,
        height_px,
        tile_base_index,
        tile_sheet,
    })
}

fn dragon_flying_descent_fire_child_frame(
    visuals: &kid_render::AnimatedSpriteVisual,
    projectile: &ActiveProjectileSnapshot,
) -> Option<kid_render::SpriteFrame> {
    let tile_sheet = visuals.frames.first()?.tile_sheet.clone();
    let (tile_base_index, origin_x, origin_y, width_px, height_px) =
        match projectile.dragon_flying_descent_fire_state {
            // ASM Dragon_Flying.asm loc_34BAC/loc_34C26 installs
            // LnkTo_unk_C75CE for the first DBF loop.
            DragonFlyingDescentFireState::FirstAnimationLoc34C26 => (0x0EC, 0x05, 0x05, 0x13, 0x18),
            // ASM Dragon_Flying.asm loc_34C3A switches to LnkTo_unk_C75D6 for
            // the travelling flame after the startup counter underflows.
            DragonFlyingDescentFireState::TravelAnimationLoc34C3A
            | DragonFlyingDescentFireState::None => (0x0F5, 0x06, 0x07, 0x1B, 0x1F),
            // ASM Dragon_Flying.asm loc_34C4A uses LnkTo_unk_C75DE for the
            // first cleanup hibernation loop before switching to LnkTo_unk_C75E6.
            DragonFlyingDescentFireState::CleanupFirstLoc34C4A => (0x105, 0x0D, 0x05, 0x1F, 0x1B),
            DragonFlyingDescentFireState::CleanupSecondLoc34C4A => (0x115, 0x0F, 0x05, 0x20, 0x10),
        };
    Some(kid_render::SpriteFrame {
        origin_x,
        origin_y,
        width_px,
        height_px,
        tile_base_index,
        tile_sheet,
    })
}

fn dragon_walking_breath_child_facing_loc_3d02a(
    trace: &NativeTraceFrame,
    projectile: &ActiveProjectileSnapshot,
) -> Direction {
    trace
        .active_objects
        .iter()
        .find_map(|object| {
            if !matches!(
                object.kind,
                NativeObjectKind::Enemy {
                    object_type: WALKING_DRAGON_OBJECT_TYPE,
                }
            ) || object.hit_points == Some(0)
            {
                return None;
            }

            let dragon_y_px = object.y_pos >> 16;
            let breath_y_px = projectile.y_pos >> 16;
            if dragon_y_px.abs_diff(breath_y_px) > 1 {
                return None;
            }

            let dragon_x_px = object.x_pos >> 16;
            let breath_x_px = projectile.x_pos >> 16;
            match breath_x_px as i32 - dragon_x_px as i32 {
                // ASM Dragon.asm loc_3D02A copies parent x_direction into the
                // breath child, then loc_3D070 adds -$1C or +$1C to x_pos.
                // Dragon art is stored facing left, so native renders the
                // default left child unflipped and the right child flipped.
                -28 => Some(Direction::Right),
                28 => Some(Direction::Left),
                _ => None,
            }
        })
        .unwrap_or(Direction::Right)
}

fn fireball_trail_child_frame(
    visuals: &kid_render::AnimatedSpriteVisual,
    remaining_frames: u16,
) -> Option<kid_render::SpriteFrame> {
    let tile_sheet = visuals.frames.first()?.tile_sheet.clone();
    let elapsed_frames = 0x0096_u16.saturating_sub(remaining_frames);
    let (tile_base_index, origin_x, origin_y, width_px, height_px) = match (elapsed_frames / 6) % 3
    {
        // ASM Fireball.asm `stru_330C2`: `LnkTo_unk_C7996`.
        0 => (0x070, 0x04, 0x05, 0x0C, 0x0D),
        // ASM Fireball.asm `stru_330C2`: `LnkTo_unk_C799E`.
        1 => (0x074, 0x04, 0x05, 0x0B, 0x0C),
        // ASM Fireball.asm `stru_330C2`: `LnkTo_unk_C79A6`.
        _ => (0x078, 0x04, 0x05, 0x0B, 0x0B),
    };
    Some(kid_render::SpriteFrame {
        origin_x,
        origin_y,
        width_px,
        height_px,
        tile_base_index,
        tile_sheet,
    })
}

fn drips_surface_crawler_frame(
    visuals: &kid_render::AnimatedSpriteVisual,
    remaining_frames: u16,
    animation_elapsed_frames: u16,
) -> Option<kid_render::SpriteFrame> {
    let tile_sheet = visuals.frames.first()?.tile_sheet.clone();
    let (tile_base_index, origin_x, origin_y, width_px, height_px) = if animation_elapsed_frames > 0
    {
        let elapsed = 31_u16.saturating_sub(animation_elapsed_frames);
        match elapsed / 10 {
            // ASM Drips.asm loc_335D8 initialises `stru_33FA6` before the child
            // becomes mobile/hostile.
            0 => (0x0BD, 0x08, 0x00, 0x0F, 0x04), // LnkTo_unk_C7BAE
            1 => (0x0BF, 0x05, 0x00, 0x0F, 0x08), // LnkTo_unk_C7BB6
            _ => (0x0C1, 0x07, 0x00, 0x0E, 0x0C), // LnkTo_unk_C7BBE
        }
    } else {
        let elapsed = u16::MAX.wrapping_sub(remaining_frames);
        // ASM Drips.asm loc_33630 installs `stru_33F14` for the ordinary
        // horizontal crawler, not the tiny `off_33520` pool-surface table.
        drips_surface_crawler_stru_33f14_frame(elapsed)
    };
    Some(kid_render::SpriteFrame {
        origin_x,
        origin_y,
        width_px,
        height_px,
        tile_base_index,
        tile_sheet,
    })
}

fn drips_surface_crawler_stru_33f14_frame(elapsed_frames: u16) -> (u16, i16, i16, u16, u16) {
    const C7B46: (u16, i16, i16, u16, u16) = (0x08D, 0x07, 0x00, 0x0F, 0x0F);
    const C7B3E: (u16, i16, i16, u16, u16) = (0x089, 0x07, 0x00, 0x0D, 0x0F);
    const C7B36: (u16, i16, i16, u16, u16) = (0x085, 0x07, 0x00, 0x0F, 0x10);
    const C7BE6: (u16, i16, i16, u16, u16) = (0x0DC, 0x07, 0x00, 0x0D, 0x0E);
    const STRU_33F14: [((u16, i16, i16, u16, u16), u16); 15] = [
        (C7B46, 0x0C),
        (C7B3E, 0x0C),
        (C7B36, 0x0C),
        (C7B46, 0x0C),
        (C7B3E, 0x0C),
        (C7B36, 0x0C),
        (C7B46, 0x0C),
        (C7B3E, 0x0C),
        (C7B36, 0x0C),
        (C7B46, 0x0C),
        (C7BE6, 0x08),
        (C7B36, 0x0C),
        (C7B46, 0x0C),
        (C7BE6, 0x08),
        (C7B36, 0x0C),
    ];

    let total_frames = STRU_33F14
        .iter()
        .map(|(_, duration)| *duration)
        .sum::<u16>();
    let mut frame_offset = elapsed_frames % total_frames;
    for (frame, duration) in STRU_33F14 {
        if frame_offset < duration {
            return frame;
        }
        frame_offset -= duration;
    }
    C7B46
}

fn drips_surface_crawler_finish_frame(
    visuals: &kid_render::AnimatedSpriteVisual,
    remaining_frames: u16,
) -> Option<kid_render::SpriteFrame> {
    let tile_sheet = visuals.frames.first()?.tile_sheet.clone();
    let elapsed = 60_u16.saturating_sub(remaining_frames);
    let (tile_base_index, origin_x, origin_y, width_px, height_px) = match elapsed / 4 {
        // ASM Drips.asm loc_338F8 installs `stru_33F06` on the original
        // crawler before loc_3392C hibernates until the animation finishes.
        0 => (0x0C5, 0x07, 0x07, 0x10, 0x10), // LnkTo_unk_C7BC6
        1 => (0x0C9, 0x0A, 0x09, 0x17, 0x16), // LnkTo_unk_C7BCE
        _ => (0x0D2, 0x0B, 0x0C, 0x18, 0x18), // LnkTo_unk_C7BD6
    };
    Some(kid_render::SpriteFrame {
        origin_x,
        origin_y,
        width_px,
        height_px,
        tile_base_index,
        tile_sheet,
    })
}

fn block_debris_velocity_sub_292e(direction: u16) -> &'static [(i16, i16); 5] {
    match direction {
        // ASM sub_292E loc_2988: d6=2 uses `unk_29FC`/`unk_2A06`.
        2 => &BLOCK_DEBRIS_VEL_D6_2_SUB_292E,
        // ASM sub_292E loc_297A: d6=0 uses `unk_2A10`/`unk_2A1A`.
        0 => &BLOCK_DEBRIS_VEL_D6_0_SUB_292E,
        // ASM sub_292E loc_296C: d6=1 uses `unk_2A38`/`unk_2A42`.
        1 => &BLOCK_DEBRIS_VEL_D6_1_SUB_292E,
        // ASM sub_292E fallthrough loc_2994: d6=3/right/default uses
        // `unk_2A24`/`unk_2A2E`.
        _ => &BLOCK_DEBRIS_VEL_D6_DEFAULT_SUB_292E,
    }
}

fn push_block_destroy_debris_sub_292e(
    commands: &mut Vec<DrawCommand>,
    level_visuals: &NativeLevelVisuals,
    x_px: u16,
    y_px: u16,
    direction: u16,
    camera_x_px: i32,
    camera_y_px: i32,
) -> bool {
    let Some(tile) = resolve_ingame_vram_tile(
        level_visuals,
        TileRef::from_attr_word(BLOCK_DEBRIS_TILE_ATTR_SUB_2A4C),
    ) else {
        return false;
    };

    let velocities = block_debris_velocity_sub_292e(direction);
    for index in 0..5 {
        // ASM `sub_292E` seeds five sprite-worker rows from `unk_29E8`/
        // `unk_29F2`, then `sub_2A4C` immediately advances by the row's
        // direction-selected velocity before writing sprite tile `$8228`.
        let x = i32::from(x_px)
            + i32::from(BLOCK_DEBRIS_BASE_X_SUB_292E[index])
            + i32::from(velocities[index].0)
            - camera_x_px;
        let y = i32::from(y_px)
            + i32::from(BLOCK_DEBRIS_BASE_Y_SUB_292E[index])
            + i32::from(velocities[index].1)
            - camera_y_px;
        push_resolved_tile(commands, x, y, tile.clone());
    }

    true
}

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
    pub eol_score: bool,
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
            eol_score: true,
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
            eol_score: false,
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
            "eol_score" => f.eol_score = true,
            "hud" => f.hud = true,
            _ => {}
        }
        f
    }
}

pub(crate) fn eol_score_sprite_diagnostics(
    trace: &NativeTraceFrame,
    level_visuals: &NativeLevelVisuals,
    block_grid: Option<&kid_core::terrain::BlockGrid>,
) -> Vec<NativeEolScoreSpriteDiagnostic> {
    // ASM Object_EOL_Number loc_C466 and Object_EOL_Text loc_C592 allocate
    // priority gfx-object slots for the score rows. This diagnostic exposes
    // native's rendered EOL score sprite commands so verifier reports can be
    // compared directly with oracle `eol_visible_gfx_slots_loc_d468`.
    let scene = build_scene_filtered(
        trace,
        false,
        Some(level_visuals),
        block_grid,
        LayerFilter::solo("eol_score"),
    );
    scene
        .commands
        .iter()
        .enumerate()
        .filter_map(|(command_index, command)| {
            let DrawCommand::Sprite { x, y, tiles, .. } = command else {
                return None;
            };
            if tiles.is_empty() {
                return None;
            }

            let min_offset_x = tiles.iter().map(|tile| tile.offset_x).min().unwrap_or(0);
            let min_offset_y = tiles.iter().map(|tile| tile.offset_y).min().unwrap_or(0);
            let max_offset_x = tiles
                .iter()
                .map(|tile| tile.offset_x.saturating_add(7))
                .max()
                .unwrap_or(0);
            let max_offset_y = tiles
                .iter()
                .map(|tile| tile.offset_y.saturating_add(7))
                .max()
                .unwrap_or(0);
            let hardware_sprites =
                eol_score_hardware_sprite_diagnostics(command_index, *x, *y, tiles);
            Some(NativeEolScoreSpriteDiagnostic {
                command_index,
                x: *x,
                y: *y,
                min_x: x.saturating_add(min_offset_x),
                min_y: y.saturating_add(min_offset_y),
                max_x: x.saturating_add(max_offset_x),
                max_y: y.saturating_add(max_offset_y),
                tile_count: tiles.len(),
                tiles: tiles
                    .iter()
                    .map(|tile| NativeSpriteTileDiagnostic {
                        offset_x: tile.offset_x,
                        offset_y: tile.offset_y,
                        tile_index: tile.tile.tile_index,
                        palette_line: tile.tile.palette_line,
                        priority: tile.tile.priority,
                        flip_x: tile.tile.flip_x,
                        flip_y: tile.tile.flip_y,
                    })
                    .collect(),
                hardware_sprites,
            })
        })
        .collect()
}

fn eol_score_hardware_sprite_diagnostics(
    command_index: usize,
    x: i32,
    y: i32,
    tiles: &[SpriteTile],
) -> Vec<NativeEolScoreHardwareSpriteDiagnostic> {
    // ASM Make_SpriteFromGfxObject loc_12B8/loc_12C0 splits each gfx frame
    // into max-32px hardware entries before sub_13C0 writes the sprite table.
    // `build_sprite_tiles` already preserves that chunk order, so regroup the
    // logical draw command to make native reports comparable with oracle
    // loc_C070 hardware sprite samples.
    let mut chunk_tiles: Vec<Vec<&SpriteTile>> = Vec::new();
    for tile in tiles {
        let chunk_x = tile.offset_x.div_euclid(32) * 32;
        let chunk_y = tile.offset_y.div_euclid(32) * 32;
        if let Some(existing) = chunk_tiles.iter_mut().find(|chunk| {
            chunk.first().is_some_and(|first| {
                first.offset_x.div_euclid(32) * 32 == chunk_x
                    && first.offset_y.div_euclid(32) * 32 == chunk_y
            })
        }) {
            existing.push(tile);
        } else {
            chunk_tiles.push(vec![tile]);
        }
    }

    chunk_tiles
        .into_iter()
        .enumerate()
        .filter_map(|(chunk_index, chunk)| {
            let first = chunk.first()?;
            let min_offset_x = chunk.iter().map(|tile| tile.offset_x).min().unwrap_or(0);
            let min_offset_y = chunk.iter().map(|tile| tile.offset_y).min().unwrap_or(0);
            let max_offset_x = chunk
                .iter()
                .map(|tile| tile.offset_x.saturating_add(7))
                .max()
                .unwrap_or(0);
            let max_offset_y = chunk
                .iter()
                .map(|tile| tile.offset_y.saturating_add(7))
                .max()
                .unwrap_or(0);
            Some(NativeEolScoreHardwareSpriteDiagnostic {
                command_index,
                chunk_index,
                x: x.saturating_add(min_offset_x),
                y: y.saturating_add(min_offset_y),
                width: u16::try_from(max_offset_x.saturating_sub(min_offset_x) + 1).unwrap_or(0),
                height: u16::try_from(max_offset_y.saturating_sub(min_offset_y) + 1).unwrap_or(0),
                first_tile_index: first.tile.tile_index,
                min_x: x.saturating_add(min_offset_x),
                min_y: y.saturating_add(min_offset_y),
                max_x: x.saturating_add(max_offset_x),
                max_y: y.saturating_add(max_offset_y),
            })
        })
        .collect()
}

fn push_fire_demon_trail(
    commands: &mut Vec<DrawCommand>,
    sx: i32,
    sy: i32,
    _remaining_frames: u16,
) {
    // Degraded-asset fallback. ASM Fire_Demon loc_3A63C cycles
    // `stru_3A622`'s six frames `unk_C7776..unk_C779E`
    // (`sprite_frame_vram $0B0..$0BA`); when the Fire Demon visual bundle
    // is present the renderer uses those tiles directly through
    // `push_fire_demon_trail_sprite`. The previous six-phase rect
    // choreography assigned hand-invented outer/core rect compositions to
    // each ROM phase even though the ASM only dictates which sprite tile
    // to swap to. Collapse to a single warm-orange placeholder rect so
    // missing-asset states stay visible without inventing per-phase
    // geometry the ROM does not specify.
    commands.push(DrawCommand::Rect {
        x: sx - 2,
        y: sy - 3,
        width: 4,
        height: 5,
        colour: FIRE_TRAIL_PLACEHOLDER_COLOUR,
    });
}

fn push_fire_demon_trail_sprite(
    commands: &mut Vec<DrawCommand>,
    level_visuals: &NativeLevelVisuals,
    sx: i32,
    sy: i32,
    remaining_frames: u16,
) -> bool {
    let Some(fire_demon_visuals) = level_visuals.enemies.families.get(&FIRE_DEMON_OBJECT_TYPE)
    else {
        return false;
    };
    let Some(source_frame) = fire_demon_visuals.frames.first() else {
        return false;
    };
    let elapsed_frames =
        FIRE_DEMON_DEATH_TRAIL_NATIVE_LIFETIME.saturating_sub(remaining_frames) as usize;
    let descriptor =
        &FIRE_DEMON_DEATH_TRAIL_FRAMES[elapsed_frames % FIRE_DEMON_DEATH_TRAIL_FRAMES.len()];
    let frame = kid_render::SpriteFrame {
        tile_sheet: source_frame.tile_sheet.clone(),
        ..descriptor.clone()
    };
    // ASM Fire_Demon loc_3A63C copies the parent Fire Demon tile base/palette
    // and initialises `stru_3A622`, whose frames are
    // `unk_C7776..unk_C779E` (`sprite_frame_vram $0B0..$0BA`).
    push_sprite_frame(commands, &frame, &fire_demon_visuals.palette, sx, sy);
    true
}

fn shooter_block_projectile_sprite_tail_loc_ef62(
    projectile: &ActiveProjectileSnapshot,
) -> (i32, i32, u16) {
    if projectile.y_velocity < 0 {
        (-4, -8, 0x8295)
    } else if projectile.x_velocity > 0 {
        (0, -4, 0x8A96)
    } else if projectile.y_velocity > 0 {
        (-4, 0, 0x9295)
    } else {
        (-8, -4, 0x8296)
    }
}

fn push_shooter_block_projectile_sprite_loc_ef62(
    commands: &mut Vec<DrawCommand>,
    level_visuals: &NativeLevelVisuals,
    projectile: &ActiveProjectileSnapshot,
    sx: i32,
    sy: i32,
) -> bool {
    let (x_offset, y_offset, tile_word) = shooter_block_projectile_sprite_tail_loc_ef62(projectile);
    let tile_ref = TileRef::from_attr_word(tile_word);
    let Some(resolved) = resolve_ingame_vram_tile(level_visuals, tile_ref) else {
        return false;
    };

    commands.push(DrawCommand::Sprite {
        x: sx + x_offset,
        y: sy + y_offset,
        tile_sheet: resolved.tile_sheet,
        palette: resolved.palette,
        tiles: vec![SpriteTile {
            offset_x: 0,
            offset_y: 0,
            tile: resolved.tile,
        }],
    });
    true
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
        LayerFilter::default(),
    )
}

pub(crate) fn render_trace_with_vdp_latched_state(
    trace: &NativeTraceFrame,
    previous_trace: Option<&NativeTraceFrame>,
) -> NativeTraceFrame {
    let mut render_trace = trace.clone();
    if matches!(trace.game_mode, GameMode::InGame | GameMode::DemoPlay)
        && trace.level_complete
        && trace.level_complete_elapsed_frames == 0
        && let Some(previous_trace) = previous_trace
    {
        // ASM loc_C070 enters the flagpole hand-off after the frame's scroll
        // transfer has already latched Camera_X_pos/Camera_Y_pos into VDP
        // h/v-scroll state, so the first visible hand-off frame uses the
        // previous scroll/sprite latch while gameplay state has already
        // switched to LevelComplete and loc_C070 is about to clear object
        // links.
        render_trace.camera_x_pos = previous_trace.camera_x_pos;
        render_trace.camera_y_pos = previous_trace.camera_y_pos;
        render_trace
            .active_objects
            .clone_from(&previous_trace.active_objects);
        render_trace.frame_index = previous_trace.frame_index;
        render_trace.current_helmet_id = previous_trace.current_helmet_id;
        render_trace.kid_facing = previous_trace.kid_facing;
        render_trace.kid_on_ground = previous_trace.kid_on_ground;
        render_trace.kid_on_slope = previous_trace.kid_on_slope;
        render_trace.kid_x_velocity = previous_trace.kid_x_velocity;
        render_trace.kid_y_velocity = previous_trace.kid_y_velocity;
        render_trace.form_change_flash_frames = previous_trace.form_change_flash_frames;
        render_trace.invulnerable_frames = previous_trace.invulnerable_frames;
        render_trace.death_windup_elapsed = previous_trace.death_windup_elapsed;
        render_trace.number_hitpoints = previous_trace.number_hitpoints;
        render_trace.extra_hitpoint_slots = previous_trace.extra_hitpoint_slots;
        render_trace.number_lives = previous_trace.number_lives;
        render_trace.diamond_count = previous_trace.diamond_count;
        render_trace.hud_diamond_frame_offset = previous_trace.hud_diamond_frame_offset;
        render_trace.level_timer_seconds = previous_trace.level_timer_seconds;
    }
    if matches!(trace.game_mode, GameMode::InGame | GameMode::DemoPlay)
        && !trace.level_complete
        && let Some(previous_trace) = previous_trace
        && previous_trace.game_mode == trace.game_mode
        && previous_trace.current_level == trace.current_level
    {
        // ASM Transfer_ScrollDataToVRAM loc_300BA writes Camera_X_pos and
        // Camera_Y_pos to VDP scroll before the next gameplay update becomes
        // visible. Trace state is post-update, so render with the previous
        // scroll latch. ASM Mode_Level also calls Transfer_SpriteAndKidToVRAM
        // before Execute_Objects/GfxObjects_MoveAndAnimate, then rebuilds the
        // next Sprite_Table at Make_SpritesFromGfxObjects near the end of the
        // frame. Non-player GfxObject sprites therefore present the previous
        // trace while current gameplay state remains authoritative.
        render_trace.camera_x_pos = previous_trace.camera_x_pos;
        render_trace.camera_y_pos = previous_trace.camera_y_pos;
        latch_previous_gfxobject_sprite_sources_loc_901(&mut render_trace, previous_trace);
        // ASM Mode_Level calls Make_SpriteAttr_HUD/sub_BDC0 before the later
        // GfxObjects_CollisionKid path can reach Check_for_recent_damge
        // loc_B700, so the visible HP pips are also a pre-update latch.
        render_trace.number_hitpoints = previous_trace.number_hitpoints;
        render_trace.extra_hitpoint_slots = previous_trace.extra_hitpoint_slots;
        // ASM sub_44DC loc_45EE updates the diamond DMA script offset before
        // the next frame's Make_SpriteAttr_HUD/sub_BDC0-built symbol becomes
        // visible, so the HUD diamond page is latched from the previous trace.
        render_trace.hud_diamond_frame_offset = previous_trace.hud_diamond_frame_offset;
        if trace.form_change_flash_frames > 0 && previous_trace.form_change_flash_frames > 0 {
            // ASM Kid_Transform loc_7CD0 hibernates the player coroutine while
            // Mode_Level still reaches loc_80C/sub_44DC. Oracle frames show
            // the visible HUD diamond page remains one `unk_461C` art slot
            // behind that post-update trace value during those repeated
            // transform hibernates.
            render_trace.hud_diamond_frame_offset = previous_trace
                .hud_diamond_frame_offset
                .map(previous_hud_diamond_frame_offset_loc_45ee);
        }
        if trace.form_change_flash_frames > 0 && previous_trace.form_change_flash_frames == 0 {
            // ASM Kid_Transform loc_7C40/loc_7CD0 starts after the frame's
            // sprite attributes have already been built. The first visible
            // transform frame therefore keeps Kid's previous visual state
            // even though the post-update trace has already entered the
            // transform coroutine and adjusted the new form's collision width.
            render_trace.current_helmet_id = previous_trace.current_helmet_id;
            render_trace.kid_facing = previous_trace.kid_facing;
            render_trace.kid_on_ground = previous_trace.kid_on_ground;
            render_trace.kid_on_slope = previous_trace.kid_on_slope;
            render_trace.kid_x_velocity = previous_trace.kid_x_velocity;
            render_trace.kid_y_velocity = previous_trace.kid_y_velocity;
            render_trace.form_change_flash_frames = previous_trace.form_change_flash_frames;
            if let Some(previous_player) = previous_trace
                .active_objects
                .iter()
                .find(|object| object.kind == NativeObjectKind::Player)
                .cloned()
                && let Some(render_player) = render_trace
                    .active_objects
                    .iter_mut()
                    .find(|object| object.kind == NativeObjectKind::Player)
            {
                *render_player = previous_player;
            }
        }
    }
    render_trace
}

fn latch_previous_gfxobject_sprite_sources_loc_901(
    render_trace: &mut NativeTraceFrame,
    previous_trace: &NativeTraceFrame,
) {
    let current_players = render_trace
        .active_objects
        .iter()
        .filter(|object| object.kind == NativeObjectKind::Player)
        .cloned();
    let previous_non_players = previous_trace
        .active_objects
        .iter()
        .filter(|object| object.kind != NativeObjectKind::Player)
        .cloned();
    render_trace.active_objects = current_players.chain(previous_non_players).collect();
    render_trace
        .active_projectiles
        .clone_from(&previous_trace.active_projectiles);
    render_trace
        .active_projectile_details
        .clone_from(&previous_trace.active_projectile_details);
}

fn previous_hud_diamond_frame_offset_loc_45ee(offset: u8) -> u8 {
    match offset {
        0 => 6,
        2 => 0,
        4 => 2,
        6 => 4,
        _ => offset.saturating_sub(2),
    }
}

pub(crate) fn build_scene_filtered(
    trace: &NativeTraceFrame,
    debug_overlay_visible: bool,
    level_visuals: Option<&NativeLevelVisuals>,
    block_grid: Option<&BlockGrid>,
    filter: LayerFilter,
) -> Scene {
    match trace.game_mode {
        GameMode::InGame | GameMode::DemoPlay => build_game_scene_filtered(
            trace,
            debug_overlay_visible,
            level_visuals.expect("Level 1 visuals are required for gameplay scenes"),
            block_grid,
            filter,
        ),
        GameMode::LevelTransition => build_transition_scene(trace, level_visuals),
        GameMode::OptionsMenu => build_options_scene(trace),
        GameMode::EndingSequence => build_ending_scene(trace),
        _ => build_status_scene(trace, debug_overlay_visible),
    }
}

pub(super) fn build_transition_scene(
    trace: &NativeTraceFrame,
    level_visuals: Option<&NativeLevelVisuals>,
) -> Scene {
    let mut commands = Vec::new();
    commands.push(DrawCommand::Rect {
        x: 0,
        y: 0,
        width: FRAME_WIDTH as i32,
        height: FRAME_HEIGHT as i32,
        colour: [0, 0, 0],
    });

    commands.extend(build_title_card_worker_overlay_scene(trace, level_visuals).commands);
    push_teleport_warp_tint_loc_bcf6(&mut commands, trace);

    Scene {
        background: [0, 0, 0],
        commands,
        pause_palette_darken: false,
    }
}

pub(super) fn build_title_card_worker_overlay_scene(
    trace: &NativeTraceFrame,
    level_visuals: Option<&NativeLevelVisuals>,
) -> Scene {
    let mut commands = Vec::new();

    // ASM Load_TitleCard `loc_1AB26` / `loc_1AC70` (kid.asm:28750-28986)
    // builds a Title_Letter object per glyph in the level's title-card script.
    // Each worker decompresses its sprite from `ArtComp_19C68_TtlCardLetters`
    // into VRAM at tile `$8120`, picks one of four random off-screen entry
    // poses via `loc_1ACB0`..`loc_1ACFE`, then glides toward the script-side
    // target through `loc_1AD2A`/`loc_1AD3A` (axis-independent step add).
    // The engine simulates this in `starting_level_title_workers`; each worker
    // is surfaced through `active_objects` with `Enemy{object_type:0}` and
    // debug fields `gfx_slot_index` + `glyph`. Render each visible glyph at
    // its current interpolated (x_pos, y_pos).
    let title_card_art = level_visuals.and_then(|lv| lv.title_card_letters.as_ref());
    for object in &trace.active_objects {
        let NativeObjectKind::Enemy { object_type: 0 } = object.kind else {
            continue;
        };
        let Some(glyph_str) = object.debug_fields.get("glyph") else {
            continue;
        };
        let glyph = parse_title_glyph(glyph_str);
        // ASM loc_1AC70 stores worker positions as 16.16 fixed-point signed
        // ints; entry poses include `$FFE0`/`$FFD0` (left of screen) and
        // `$160` (right of screen). Interpret as i32 to keep off-screen
        // start positions correct.
        let sx = (object.x_pos as i32) >> 16;
        let sy = (object.y_pos as i32) >> 16;
        if let Some(art) = title_card_art
            && push_title_card_glyph_tiles(&mut commands, art, glyph, sx, sy)
        {
            continue;
        }
        // Fallback: when the title-card letter art isn't loaded, render the
        // glyph as ASCII via the placeholder font. The per-glyph motion is
        // still ASM-faithful via the worker pose; only the letter art is
        // approximate.
        if let Some(ch) = title_glyph_to_ascii(glyph) {
            commands.push(text(sx, sy, &ch.to_string(), [255, 240, 200], 2));
        }
    }

    Scene {
        background: [0, 0, 0],
        commands,
        pause_palette_darken: false,
    }
}

fn parse_title_glyph(value: &str) -> u8 {
    let trimmed = value.strip_prefix("0x").unwrap_or(value);
    u8::from_str_radix(trimmed, 16).unwrap_or(0)
}

fn title_glyph_to_ascii(glyph: u8) -> Option<char> {
    // ASM `loc_1AC70` -> `off_1AC2A` (kid.asm:28859-28893) maps `glyph - $61`
    // into a 35-entry sprite table covering letters `a..z` (indices 0..$19)
    // and five wide glyphs at $7B..$7F (indices $1A..$1E), plus four marker
    // glyphs at $80..$83 (indices $1F..$22). The control bytes $00 (newline),
    // $84 (space), and $FF (end) never allocate a worker.
    match glyph {
        b'a'..=b'z' => Some((glyph - b'a' + b'A') as char),
        0x7B..=0x7F => Some('*'),
        0x80..=0x83 => Some('!'),
        _ => None,
    }
}

/// Per-glyph sprite descriptor from ASM `unk_C86E8`..`unk_C87F8`
/// (kid.asm:49238-49272). Each `off_1AC2A` entry points to a
/// `sprite_frame_vram tile_offset, ox, oy, width, height` macro; the
/// title-card letter sheet uses `tile_offset` as a byte offset into the
/// uploaded VRAM region. For native rendering the offset is the tile
/// index within the loaded letter `TileSheet`, and dimensions are the
/// sprite extent in pixels.
struct TitleCardGlyphFrame {
    tile_offset: u16,
    width_px: u16,
    height_px: u16,
}

fn title_card_glyph_frame(glyph: u8) -> Option<TitleCardGlyphFrame> {
    // ASM `unk_C86E8` onwards: most letters are 2×2 tiles (16×16 px);
    // `i` (`$69`) is 1×2 (8×16), `$83` is 1×1 (8×8), and the wide
    // glyphs `$7B`..`$80` use non-square tile counts. Hard-code the
    // table verbatim from kid.asm:49238-49272 so the tile offsets and
    // extents match the ROM exactly.
    let (tile_offset, width_px, height_px) = match glyph {
        b'a' => (0x000, 0x10, 0x10),
        b'b' => (0x004, 0x10, 0x10),
        b'c' => (0x008, 0x10, 0x10),
        b'd' => (0x00C, 0x10, 0x10),
        b'e' => (0x010, 0x10, 0x10),
        b'f' => (0x014, 0x10, 0x10),
        b'g' => (0x018, 0x10, 0x10),
        b'h' => (0x01C, 0x10, 0x10),
        b'i' => (0x020, 0x08, 0x10),
        b'j' => (0x022, 0x10, 0x10),
        b'k' => (0x026, 0x10, 0x10),
        b'l' => (0x02A, 0x10, 0x10),
        b'm' => (0x02E, 0x10, 0x10),
        b'n' => (0x032, 0x10, 0x10),
        b'o' => (0x036, 0x10, 0x10),
        b'p' => (0x03A, 0x10, 0x10),
        b'q' => (0x03E, 0x10, 0x10),
        b'r' => (0x042, 0x10, 0x10),
        b's' => (0x046, 0x10, 0x10),
        b't' => (0x04A, 0x10, 0x10),
        b'u' => (0x04E, 0x10, 0x10),
        b'v' => (0x052, 0x10, 0x10),
        b'w' => (0x056, 0x10, 0x10),
        b'x' => (0x05A, 0x10, 0x10),
        b'y' => (0x05E, 0x10, 0x10),
        b'z' => (0x062, 0x10, 0x10),
        0x7B => (0x066, 0x10, 0x20),
        0x7C => (0x06E, 0x28, 0x10),
        0x7D => (0x078, 0x20, 0x10),
        0x7E => (0x080, 0x18, 0x10),
        0x7F => (0x086, 0x18, 0x10),
        0x80 => (0x08C, 0x60, 0x10),
        0x81 => (0x0A4, 0x08, 0x18),
        0x82 => (0x0A7, 0x10, 0x18),
        0x83 => (0x0AD, 0x08, 0x08),
        _ => return None,
    };
    Some(TitleCardGlyphFrame {
        tile_offset,
        width_px,
        height_px,
    })
}

/// Emit `IndexedTile` commands for every 8×8 tile in the glyph's sprite,
/// laid out column-major as the MD sprite system does. Returns true when
/// the glyph mapping resolves AND the tile sheet contains the
/// referenced tiles. ASM `loc_1AC70` writes `move.w #$409,vram_tile`,
/// which puts the letters on palette line 1.
fn push_title_card_glyph_tiles(
    commands: &mut Vec<DrawCommand>,
    art: &kid_render::TitleCardLettersVisuals,
    glyph: u8,
    sx: i32,
    sy: i32,
) -> bool {
    let Some(frame) = title_card_glyph_frame(glyph) else {
        return false;
    };
    let cols = frame.width_px / 8;
    let rows = frame.height_px / 8;
    let palette = Arc::new(art.palette.clone());
    let tile_sheet = Arc::new(art.tile_sheet.clone());
    let mut emitted = 0u32;
    for col in 0..cols {
        for row in 0..rows {
            // ASM sprite VRAM layout is column-major: tile_offset advances by
            // (rows) when moving to the next column. Mirroring the ROM's
            // column-first tile order keeps the rendered letter pixel-for-
            // pixel aligned with `sprite_frame_vram`.
            let tile_index = frame.tile_offset + (col * rows) + row;
            if usize::from(tile_index) >= tile_sheet.tiles.len() {
                continue;
            }
            commands.push(DrawCommand::IndexedTile {
                x: sx + i32::from(col) * 8,
                y: sy + i32::from(row) * 8,
                tile_sheet: tile_sheet.clone(),
                tile: kid_render::TileRef {
                    tile_index,
                    // The loaded `Title_card_letters.bin` palette is the
                    // complete 16-colour line that ASM loc_1AB40 copies to
                    // CRAM line 1. Native stores that standalone line at
                    // Palette slot 0; `palette_line: 0` lets the 16-colour
                    // resolver keep the ROM slot numbers while avoiding a
                    // second-line offset past the loaded data.
                    palette_line: 0,
                    priority: false,
                    flip_x: false,
                    flip_y: false,
                },
                palette: palette.clone(),
            });
            emitted += 1;
        }
    }
    emitted > 0
}

pub(super) fn build_status_scene(trace: &NativeTraceFrame, debug_overlay_visible: bool) -> Scene {
    let mut commands = Vec::new();

    if trace.game_complete {
        return build_victory_scene(trace);
    }

    // Sega screen: ASM Load_SegaScreen (kid.asm:25444) decompresses
    // ArtComp_12F30_Sega into VRAM at $1780, loads Pal_12EEC (background
    // word $0000 = black) into CRAM, and writes the 4×12-tile logo at
    // nametable rows $061C/$069C/$071C/$079C. The interactive player
    // renders this through `kid_md::render_vdp_into` (native_player::run);
    // this fallback path only fires in sweep / recording / TAS-viewer
    // contexts where the VDP snapshot is unavailable. The previous blue
    // background and 4×-scaled "SEGA" text were invented placeholders —
    // the ROM never renders a typed string, and palette index 0 is black,
    // not blue.
    if trace.game_mode == GameMode::SegaScreen {
        return Scene {
            background: [0, 0, 0],
            commands,
            pause_palette_darken: false,
        };
    }

    // Intro sequences: ASM `Load_IntroSequence1..5` / `Load_IntroSequence6`
    // (kid.asm:29207-29464) build tile-based VRAM compositions for each
    // segment — none of the slogans ("The Wild Side", "Heady Metal's
    // Challenge", "Enter the Game") exist as ROM-renderable ASCII text;
    // they're 4bpp tile art uploaded via `sub_1B850` / its per-segment
    // siblings. The interactive player routes through `kid_md::render_vdp_into`
    // when `runtime.vdp_state()` is populated; this fallback only fires in
    // sweep/recording/TAS-viewer contexts where the VDP snapshot is absent.
    // The previous English slogan + "Press Start to skip" text was invented
    // to fill the screen — collapse to honest black until the tile art is
    // wired through the asset bundle, matching the Sega-screen treatment.
    if matches!(
        trace.game_mode,
        GameMode::IntroSequence1
            | GameMode::IntroSequence2
            | GameMode::IntroSequence3
            | GameMode::IntroSequence4
            | GameMode::IntroSequence5
            | GameMode::IntroSequence6
    ) {
        return Scene {
            background: [0, 0, 0],
            commands,
            pause_palette_darken: false,
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

    // Title. ASM `Obj_TitleMenu` (kid.asm:30310) renders the title screen
    // through tile art uploaded by `Load_TitleScreen` — the ROM never
    // displays typed ASCII. The interactive player routes the title through
    // `kid_md::render_vdp_into` when `runtime.vdp_state()` is populated;
    // this fallback only fires in sweep/recording/TAS-viewer contexts.
    // Keep the typed "KID CHAMELEON" as a degraded-asset readability cue —
    // the previous "Native Playtest" subtitle was a pure dev invention and
    // is dropped so the fallback doesn't claim a sub-heading the ROM never
    // shows.
    let title = "KID CHAMELEON";
    let title_w = title.len() as i32 * 8 * 3;
    commands.push(text(
        (FRAME_WIDTH as i32 - title_w) / 2,
        40,
        title,
        [255, 220, 80],
        3,
    ));

    if let Some(cursor) = trace.options_cursor {
        // ASM Obj_TitleMenu loc_1BB52 (kid.asm:30428): the ROM swaps the
        // pre-menu title prompt for the three-choice tilemap decoded from
        // `unk_2EAAE`; loc_1BC0E then moves the cursor through rows 0..2.
        let rows = ["1 PLAYER", "2 PLAYERS", "OPTIONS"];
        for (row, label) in rows.iter().enumerate() {
            let y = 100 + row as i32 * 20;
            let selected = row == usize::from(cursor.min(2));
            commands.push(text(
                116,
                y,
                label,
                if selected {
                    [255, 255, 255]
                } else {
                    [150, 150, 150]
                },
                1,
            ));
            if selected {
                commands.push(text(100, y, ">", [255, 220, 80], 1));
            }
        }
    } else if (trace.frame_index / 30).is_multiple_of(2) {
        // Blinking "PRESS START" (blink every 30 frames)
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
        pause_palette_darken: false,
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

    let cursor = usize::from(trace.options_cursor.unwrap_or(0).min(3));
    let controller_label = if trace.options_two_player_two_controllers {
        "TWO CONTROLLERS"
    } else {
        "ONE CONTROLLER"
    };
    let controls = match trace.options_controls_scheme.min(5) {
        0 => ["SPEED", "JUMP", "SPECIAL"],
        1 => ["SPEED", "SPECIAL", "JUMP"],
        2 => ["JUMP", "SPEED", "SPECIAL"],
        3 => ["JUMP", "SPECIAL", "SPEED"],
        4 => ["SPECIAL", "SPEED", "JUMP"],
        _ => ["SPECIAL", "JUMP", "SPEED"],
    };
    let speed_lines = if trace.options_speed_button_fast {
        ["FAST ACTION", "PUSH FOR SLOW"]
    } else {
        ["NORMAL ACTION", "PUSH FOR FAST"]
    };

    let row_colour = |row: usize| {
        if row == cursor {
            [255, 255, 255]
        } else {
            [120, 120, 120]
        }
    };
    let option_text = |x_cell: i32, y_cell: i32, value: &str, colour: [u8; 3]| {
        text(x_cell * 8, y_cell * 8, value, colour, 1)
    };

    // ASM DrawOptText1..4 loc_1CE3A/loc_1CEB2/loc_1CF78/loc_1CFFE
    // (kid.asm:32034-32373): DrawTextLine_Offset adds the ROM base cells
    // `(7, 10)` to each packed text offset. The fallback still uses typed
    // glyphs, but it must preserve those cell positions so the Options screen
    // is readable instead of joining whole groups into over-wide rows.
    commands.push(option_text(7, 10, "2 PLAYERS", row_colour(0)));
    commands.push(option_text(19, 10, controller_label, row_colour(0)));

    commands.push(option_text(7, 13, "CONTROLS", row_colour(1)));
    commands.push(option_text(18, 13, "MODE", row_colour(1)));
    commands.push(option_text(12, 15, "A", row_colour(1)));
    commands.push(option_text(16, 15, controls[0], row_colour(1)));
    commands.push(option_text(12, 17, "B", row_colour(1)));
    commands.push(option_text(16, 17, controls[1], row_colour(1)));
    commands.push(option_text(12, 19, "C", row_colour(1)));
    commands.push(option_text(16, 19, controls[2], row_colour(1)));

    commands.push(option_text(7, 22, "SPEED BUTTON", row_colour(2)));
    commands.push(option_text(21, 22, speed_lines[0], row_colour(2)));
    commands.push(option_text(21, 24, speed_lines[1], row_colour(2)));

    commands.push(option_text(7, 27, "EXIT", row_colour(3)));

    Scene {
        background: [8, 12, 20],
        commands,
        pause_palette_darken: false,
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

    let track_label = format!("TRACK: {track:02X}");
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
        pause_palette_darken: false,
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
        pause_palette_darken: false,
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
            pause_palette_darken: false,
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
        pause_palette_darken: false,
    }
}

pub(super) fn resolve_ingame_vram_tile(
    level_visuals: &NativeLevelVisuals,
    tile_ref: TileRef,
) -> Option<ResolvedSceneTile> {
    resolve_ingame_vram_tile_with_foreground_palette(
        level_visuals,
        &level_visuals.palettes.foreground,
        tile_ref,
    )
}

fn resolve_ingame_vram_tile_with_foreground_palette(
    level_visuals: &NativeLevelVisuals,
    foreground_palette: &Palette,
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
            palette: Arc::new(foreground_palette.clone()),
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
            palette: Arc::new(foreground_palette.clone()),
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
            palette: Arc::new(foreground_palette.clone()),
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
            palette: Arc::new(foreground_palette.clone()),
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
    let palette = Arc::new(background_palette_for_scene(trace, level_visuals));

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
                                // ASM sub_307D8 indexes `unk_309B8`, whose
                                // chunked background tile words are
                                // `$2780 + tile_id`: background art base with
                                // CRAM palette line 1.
                                palette_line: 1,
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
            let row_height_px = i32::try_from(background.rows.len())
                .ok()
                .and_then(|rows| rows.checked_mul(8));
            for (row_index, row) in background.rows.iter().enumerate() {
                let row_width_px = i32::try_from(row.tiles.len())
                    .ok()
                    .and_then(|tiles| tiles.checked_mul(8));
                let wrap_row = layered_background_row_wraps_for_trace_loc_3050a(
                    trace,
                    level_visuals,
                    row_index,
                );
                for (column, tile) in row.tiles.iter().enumerate() {
                    let (mut screen_x, screen_y) = background_tile_screen_position(
                        trace,
                        level_visuals,
                        column as u16,
                        row_index as u16,
                        camera_x_px,
                        camera_y_px,
                    );
                    let screen_y = row_height_px.map_or(screen_y, |row_height_px| {
                        wrap_layered_background_y_loc_11b7a(screen_y, row_height_px)
                    });
                    if wrap_row && let Some(row_width_px) = row_width_px {
                        screen_x = wrap_layered_background_x_loc_11b7a(screen_x, row_width_px);
                    }
                    if screen_x <= -8
                        || screen_y <= -8
                        || screen_x >= FRAME_WIDTH as i32
                        || screen_y >= FRAME_HEIGHT as i32
                    {
                        continue;
                    }
                    push_layered_background_tile(
                        commands,
                        trace,
                        background.effect_start_row_selector,
                        level_visuals.level.background_theme_id,
                        camera_y_px,
                        screen_x,
                        screen_y,
                        tile_sheet.clone(),
                        palette.clone(),
                        *tile,
                    );
                }
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct BackgroundPixelSample {
    pub(crate) screen_x: i32,
    pub(crate) screen_y: i32,
    pub(crate) tile_origin_x: i32,
    pub(crate) tile_origin_y: i32,
    pub(crate) background_row_index: Option<usize>,
    pub(crate) background_row_selector: Option<u8>,
    pub(crate) background_column: Option<usize>,
    pub(crate) tile_index: u16,
    pub(crate) palette_line: u8,
    pub(crate) palette_index: u8,
    pub(crate) source_x: u8,
    pub(crate) source_y: u8,
    pub(crate) row_offset: Option<i32>,
    pub(crate) rgb: [u8; 3],
}

pub(crate) fn sample_background_pixel_loc_c070(
    trace: &NativeTraceFrame,
    level_visuals: &NativeLevelVisuals,
    sample_x: i32,
    sample_y: i32,
) -> Option<BackgroundPixelSample> {
    let camera_x_px = (trace.camera_x_pos >> 16) as i32;
    let camera_y_px = (trace.camera_y_pos >> 16) as i32;
    let palette = background_palette_for_scene(trace, level_visuals);
    let mut sample = None;

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
                        let tile_ref = TileRef {
                            tile_index: background_tile.tile_index,
                            // ASM sub_307D8 -> `unk_309B8` emits `$2780`
                            // based tile words for chunked backgrounds, so
                            // native samples must use CRAM palette line 1 too.
                            palette_line: 1,
                            priority: false,
                            flip_x: background_tile.flip_x,
                            flip_y: false,
                        };
                        if let Some(hit) = sample_background_tile_pixel(
                            level_visuals,
                            &palette,
                            tile_ref,
                            screen_x,
                            screen_y,
                            sample_x,
                            sample_y,
                            None,
                            None,
                            screen_x,
                            screen_y,
                            Some(usize::from(placement.y_tiles + row)),
                            None,
                            Some(usize::from(placement.x_tiles + column)),
                        ) {
                            sample = Some(hit);
                        }
                    }
                }
            }
        }
        kid_render::BackgroundVisuals::Layered(background) => {
            let row_height_px = i32::try_from(background.rows.len())
                .ok()
                .and_then(|rows| rows.checked_mul(8));
            for (row_index, row) in background.rows.iter().enumerate() {
                let row_width_px = i32::try_from(row.tiles.len())
                    .ok()
                    .and_then(|tiles| tiles.checked_mul(8));
                let wrap_row = layered_background_row_wraps_for_trace_loc_3050a(
                    trace,
                    level_visuals,
                    row_index,
                );
                for (column, tile) in row.tiles.iter().enumerate() {
                    let (mut screen_x, screen_y) = background_tile_screen_position(
                        trace,
                        level_visuals,
                        column as u16,
                        row_index as u16,
                        camera_x_px,
                        camera_y_px,
                    );
                    let screen_y = row_height_px.map_or(screen_y, |row_height_px| {
                        wrap_layered_background_y_loc_11b7a(screen_y, row_height_px)
                    });
                    if wrap_row && let Some(row_width_px) = row_width_px {
                        screen_x = wrap_layered_background_x_loc_11b7a(screen_x, row_width_px);
                    }
                    if let Some(hit) = sample_layered_background_tile_pixel(
                        trace,
                        level_visuals,
                        &palette,
                        background.effect_start_row_selector,
                        camera_y_px,
                        screen_x,
                        screen_y,
                        row_index,
                        row.selector,
                        column,
                        *tile,
                        sample_x,
                        sample_y,
                    ) {
                        sample = Some(hit);
                    }
                }
            }
        }
    }

    sample
}

fn sample_layered_background_tile_pixel(
    trace: &NativeTraceFrame,
    level_visuals: &NativeLevelVisuals,
    palette: &Palette,
    effect_start_row_selector: Option<u8>,
    camera_y_px: i32,
    screen_x: i32,
    screen_y: i32,
    background_row_index: usize,
    background_row_selector: u8,
    background_column: usize,
    tile: TileRef,
    sample_x: i32,
    sample_y: i32,
) -> Option<BackgroundPixelSample> {
    let Some(effect_start_row_selector) = effect_start_row_selector else {
        return sample_background_tile_pixel(
            level_visuals,
            palette,
            tile,
            screen_x,
            screen_y,
            sample_x,
            sample_y,
            None,
            None,
            screen_x,
            screen_y,
            Some(background_row_index),
            Some(background_row_selector),
            Some(background_column),
        );
    };
    if level_visuals.level.background_theme_id != 9 {
        return sample_background_tile_pixel(
            level_visuals,
            palette,
            tile,
            screen_x,
            screen_y,
            sample_x,
            sample_y,
            None,
            None,
            screen_x,
            screen_y,
            Some(background_row_index),
            Some(background_row_selector),
            Some(background_column),
        );
    }

    for row in 0..8 {
        let offset = forest_water_ripple_offset_for_screen_y_sub_30598(
            forest_water_ripple_time_frames_for_trace_loc_305e6(trace),
            screen_y + row,
            effect_start_row_selector,
            camera_y_px,
        )
        .unwrap_or(0);
        if let Some(sample) = sample_background_tile_pixel(
            level_visuals,
            palette,
            tile,
            screen_x + offset,
            screen_y + row,
            sample_x,
            sample_y,
            Some(row as u8),
            Some(offset),
            screen_x,
            screen_y,
            Some(background_row_index),
            Some(background_row_selector),
            Some(background_column),
        ) {
            return Some(sample);
        }
    }
    None
}

fn sample_background_tile_pixel(
    level_visuals: &NativeLevelVisuals,
    palette: &Palette,
    tile_ref: TileRef,
    screen_x: i32,
    screen_y: i32,
    sample_x: i32,
    sample_y: i32,
    row_override: Option<u8>,
    row_offset: Option<i32>,
    tile_origin_x: i32,
    tile_origin_y: i32,
    background_row_index: Option<usize>,
    background_row_selector: Option<u8>,
    background_column: Option<usize>,
) -> Option<BackgroundPixelSample> {
    if sample_x < screen_x || sample_x >= screen_x + 8 {
        return None;
    }
    let dest_x = sample_x - screen_x;
    let dest_y = match row_override {
        Some(row) if sample_y == screen_y => i32::from(row),
        Some(_) => return None,
        None if sample_y >= screen_y && sample_y < screen_y + 8 => sample_y - screen_y,
        None => return None,
    };
    let source_x = if tile_ref.flip_x { 7 - dest_x } else { dest_x };
    let source_y = if tile_ref.flip_y { 7 - dest_y } else { dest_y };
    let tile = level_visuals
        .background_tiles
        .tiles
        .get(usize::from(tile_ref.tile_index))?;
    let palette_index = tile.pixels[(source_y * 8 + source_x) as usize];
    let rgb = resolve_palette_colour(palette, tile_ref.palette_line, palette_index)?;
    Some(BackgroundPixelSample {
        screen_x,
        screen_y: if row_override.is_some() {
            sample_y
        } else {
            screen_y
        },
        tile_origin_x,
        tile_origin_y,
        background_row_index,
        background_row_selector,
        background_column,
        tile_index: tile_ref.tile_index,
        palette_line: tile_ref.palette_line,
        palette_index,
        source_x: source_x as u8,
        source_y: source_y as u8,
        row_offset,
        rgb,
    })
}

fn push_layered_background_tile(
    commands: &mut Vec<DrawCommand>,
    trace: &NativeTraceFrame,
    effect_start_row_selector: Option<u8>,
    background_theme_id: u8,
    camera_y_px: i32,
    screen_x: i32,
    screen_y: i32,
    tile_sheet: Arc<TileSheet>,
    palette: Arc<Palette>,
    tile: TileRef,
) {
    let Some(effect_start_row_selector) = effect_start_row_selector else {
        commands.push(DrawCommand::IndexedTile {
            x: screen_x,
            y: screen_y,
            tile_sheet,
            tile,
            palette,
        });
        return;
    };
    if background_theme_id != 9 {
        commands.push(DrawCommand::IndexedTile {
            x: screen_x,
            y: screen_y,
            tile_sheet,
            tile,
            palette,
        });
        return;
    }

    let row_offsets = (0..8)
        .map(|row| {
            forest_water_ripple_offset_for_screen_y_sub_30598(
                forest_water_ripple_time_frames_for_trace_loc_305e6(trace),
                screen_y + row,
                effect_start_row_selector,
                camera_y_px,
            )
        })
        .collect::<Vec<_>>();
    if row_offsets.iter().all(Option::is_none) {
        commands.push(DrawCommand::IndexedTile {
            x: screen_x,
            y: screen_y,
            tile_sheet,
            tile,
            palette,
        });
        return;
    }

    for (row, offset) in row_offsets.into_iter().enumerate() {
        commands.push(DrawCommand::IndexedTileRow {
            x: screen_x + offset.unwrap_or(0),
            y: screen_y + row as i32,
            tile_sheet: tile_sheet.clone(),
            tile,
            palette: palette.clone(),
            row: row as u8,
        });
    }
}

pub(super) fn foreground_palette_for_scene(
    trace: &NativeTraceFrame,
    level_visuals: &NativeLevelVisuals,
) -> Palette {
    let mut palette = level_visuals.palettes.foreground.clone();
    let delay = match level_visuals.level.foreground_theme_id {
        7 => 5usize,
        8 => 8usize,
        _ => return palette,
    };
    if palette.colours.len() < 10 {
        return palette;
    }

    // ASM RotatePalette_Lava_Water loc_1223A/loc_1223C: Mountain levels
    // hibernate 5 frames (Cave 8), then rotate CRAM words
    // Palette_Buffer+$E/$10/$12/$14. Native foreground palettes omit CRAM
    // word 0, so those ROM words map to visible colour indices 6..=9. The
    // extracted level palette is two ROM rotations behind the oracle
    // settled-playtest frame 0 for Highwater's direct level-skip lineage, so
    // pre-advance by two rotations before applying the live frame cadence.
    for _ in 0..((trace.frame_index / delay + 2) % 4) {
        let saved = palette.colours[9];
        palette.colours[9] = palette.colours[8];
        palette.colours[8] = palette.colours[7];
        palette.colours[7] = palette.colours[6];
        palette.colours[6] = saved;
    }

    palette
}

fn background_palette_for_scene(
    trace: &NativeTraceFrame,
    level_visuals: &NativeLevelVisuals,
) -> Palette {
    let mut palette = level_visuals.palettes.background.clone();
    if level_visuals.level.background_theme_id == 7
        && level_visuals.palettes.foreground.colours.len() >= PALETTE_LINE_COLOUR_COUNT - 1
        && palette.colours.len() >= 8
    {
        // ASM Load_BackgroundMappings_Layered loc_12910 decodes Mountain's
        // main planes with palette-line bits from base `$2780`, while the
        // clear-weather follow-up plane uses `$07E5` and therefore reads
        // foreground CRAM line 0. loc_11E16 copies background colours 1..7
        // into line-1 slots 9..15 for the main planes without replacing the
        // foreground line used by the follow-up art.
        let original = palette.colours.clone();
        let mut combined = foreground_palette_for_scene(trace, level_visuals).colours;
        combined.truncate(PALETTE_LINE_COLOUR_COUNT - 1);
        combined.resize(
            PALETTE_LINE_COLOUR_COUNT * 2,
            kid_render::MdColour::from_word(0),
        );
        combined[PALETTE_LINE_COLOUR_COUNT + 8..PALETTE_LINE_COLOUR_COUNT + 15]
            .copy_from_slice(&original[1..8]);
        palette.colours = combined;
        return palette;
    }
    if level_visuals.level.background_theme_id == 9 && palette.colours.len() >= 8 {
        // ASM Load_BackgroundMappings_Layered loc_128BA decodes Forest tile
        // words with base `$0780`, so attr palette 0 really reads from CRAM
        // line 0.  The background palette is only copied into line 1 slots
        // 8..15, then ASM sub_129AE swaps line-1 slots 8 and 10.
        let original = palette.colours.clone();
        let mut combined = foreground_palette_for_scene(trace, level_visuals).colours;
        if combined.len() < PALETTE_LINE_COLOUR_COUNT {
            return palette;
        }
        combined.truncate(PALETTE_LINE_COLOUR_COUNT);
        combined.extend(std::iter::repeat_n(
            kid_render::MdColour::from_word(0),
            PALETTE_LINE_COLOUR_COUNT,
        ));
        combined[PALETTE_LINE_COLOUR_COUNT + 7] = original[2];
        combined[PALETTE_LINE_COLOUR_COUNT + 8] = original[1];
        combined[PALETTE_LINE_COLOUR_COUNT + 9] = kid_render::MdColour::from_word(0);
        combined[PALETTE_LINE_COLOUR_COUNT + 10..PALETTE_LINE_COLOUR_COUNT + 15]
            .copy_from_slice(&original[3..8]);
        palette.colours = combined;
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
    push_sprite_frame_with_priority(commands, frame, palette, screen_x, screen_y, None);
}

fn push_sprite_frame_with_priority(
    commands: &mut Vec<DrawCommand>,
    frame: &kid_render::SpriteFrame,
    palette: &Palette,
    screen_x: i32,
    screen_y: i32,
    priority_override: Option<bool>,
) {
    let facing = Direction::Right;
    let (draw_x, draw_y) = sprite_draw_position(frame, screen_x, screen_y, facing);
    let mut tiles = build_sprite_tiles(frame, facing, 0);
    if let Some(priority) = priority_override {
        for tile in &mut tiles {
            tile.tile.priority = priority;
        }
    }
    commands.push(DrawCommand::Sprite {
        x: draw_x,
        y: draw_y,
        tile_sheet: Arc::new(frame.tile_sheet.clone()),
        palette: Arc::new(palette.clone()),
        tiles,
    });
}

pub(super) fn push_sprite_frame_top_left(
    commands: &mut Vec<DrawCommand>,
    frame: &kid_render::SpriteFrame,
    palette: &Palette,
    x: i32,
    y: i32,
) {
    let screen_x = x + i32::from(frame.origin_x);
    let screen_y = y + i32::from(frame.height_px) - i32::from(frame.origin_y);
    push_sprite_frame(commands, frame, palette, screen_x, screen_y);
}

fn push_diamond_power_sprite_loc_3f880(
    commands: &mut Vec<DrawCommand>,
    level_visuals: &NativeLevelVisuals,
    sprite: DiamondPowerSpriteSnapshot,
) -> bool {
    let size_code = sprite.size_word >> 8;
    let width_px = ((size_code & 0x03) + 1) * 8;
    let height_px = (((size_code >> 2) & 0x03) + 1) * 8;
    let vram_tile = sprite.pattern_word & 0x07FF;
    let Some(tile_base_index) = vram_tile.checked_sub(level_visuals.hud.diamond_base_tile_index)
    else {
        return false;
    };
    if usize::from(tile_base_index) >= level_visuals.hud.diamond_tiles.tiles.len() {
        return false;
    }
    let frame = kid_render::SpriteFrame {
        origin_x: 0,
        origin_y: 0,
        width_px,
        height_px,
        tile_base_index,
        tile_sheet: level_visuals.hud.diamond_tiles.clone(),
    };
    // ASM DiamondPower_Main_CircleOfDoom loc_3F880 writes VDP sprite
    // coordinates already biased by `$7F-Camera`; native drawing uses
    // top-left screen pixels, so strip the VDP sprite bias here.
    push_sprite_frame_top_left(
        commands,
        &frame,
        &level_visuals.palettes.foreground,
        i32::from(sprite.screen_x) - VDP_SPRITE_SCREEN_BIAS_LOC_13C0,
        i32::from(sprite.screen_y) - VDP_SPRITE_SCREEN_BIAS_LOC_13C0,
    );
    true
}

fn push_murder_wall_warning_sprite_sub_5e02(
    commands: &mut Vec<DrawCommand>,
    visuals: &kid_render::MurderWallVisuals,
    sprite: kid_core::MurderWallWarningSpriteSub5E02,
) -> bool {
    let tile_ref = TileRef::from_attr_word(sprite.tile_word);
    let Some(tile_base_index) = tile_ref
        .tile_index
        .checked_sub(visuals.vram_base_tile_index)
    else {
        return false;
    };
    let frame = kid_render::SpriteFrame {
        origin_x: 0,
        origin_y: 0,
        width_px: 24,
        height_px: 24,
        tile_base_index,
        tile_sheet: visuals.tile_sheet.clone(),
    };
    let facing = if tile_ref.flip_x {
        Direction::Left
    } else {
        Direction::Right
    };
    let mut tiles = build_sprite_tiles(&frame, facing, tile_ref.palette_line);
    for tile in &mut tiles {
        tile.tile.priority = tile_ref.priority;
        tile.tile.flip_y = tile_ref.flip_y;
    }
    commands.push(DrawCommand::Sprite {
        x: i32::from(sprite.x) - 0x80,
        y: i32::from(sprite.y) - 0x80,
        tile_sheet: Arc::new(visuals.tile_sheet.clone()),
        palette: Arc::new(visuals.palette.clone()),
        tiles,
    });
    true
}

fn push_murder_wall_warning_fallback_sub_5e02(
    commands: &mut Vec<DrawCommand>,
    min_x: i32,
    max_x: i32,
) {
    let width = (max_x - min_x).max(8);
    // TODO(fidelity): decode/render exact `Murder_wall.bin` drill art from
    // ASM Murderwall loc_140D2. This fallback is intentionally conspicuous so
    // live playtests can see the active `sub_5E02` wall instead of a flat red
    // placeholder while the ArtComp/Saxman path is still unresolved.
    commands.push(DrawCommand::Rect {
        x: min_x,
        y: 0,
        width,
        height: FRAME_HEIGHT as i32,
        colour: MURDER_WALL_FALLBACK_BASE_COLOUR,
    });
    let mut y = -8;
    while y < FRAME_HEIGHT as i32 {
        commands.push(DrawCommand::Rect {
            x: min_x,
            y,
            width,
            height: 6,
            colour: MURDER_WALL_FALLBACK_SHADOW_COLOUR,
        });
        commands.push(DrawCommand::Rect {
            x: min_x + width / 3,
            y: y + 6,
            width: (width * 2 / 3).max(4),
            height: 4,
            colour: MURDER_WALL_FALLBACK_HIGHLIGHT_COLOUR,
        });
        commands.push(DrawCommand::Rect {
            x: min_x,
            y: y + 12,
            width: (width * 2 / 3).max(4),
            height: 4,
            colour: MURDER_WALL_FALLBACK_SHADOW_COLOUR,
        });
        y += 24;
    }
}

fn push_lava_geyser_chunk_loc_14be8(
    commands: &mut Vec<DrawCommand>,
    visuals: &kid_render::LavaVisuals,
    chunk: kid_core::LavaGeyserSpriteChunkLoc14BE8,
) -> bool {
    let tile_ref = TileRef::from_attr_word(chunk.tile_word);
    let Some(tile_base_index) = tile_ref
        .tile_index
        .checked_sub(visuals.vram_base_tile_index)
    else {
        return false;
    };
    let size_code = (chunk.size_word >> 8) & 0x0F;
    let width_px = ((size_code & 0x03) + 1) * 8;
    let height_px = (((size_code >> 2) & 0x03) + 1) * 8;
    let frame = kid_render::SpriteFrame {
        origin_x: 0,
        origin_y: 0,
        width_px,
        height_px,
        tile_base_index,
        tile_sheet: visuals.tile_sheet.clone(),
    };
    let mut tiles = build_sprite_tiles(&frame, Direction::Right, tile_ref.palette_line);
    for tile in &mut tiles {
        tile.tile.priority = tile_ref.priority;
        tile.tile.flip_x = tile_ref.flip_x;
        tile.tile.flip_y = tile_ref.flip_y;
    }
    commands.push(DrawCommand::Sprite {
        x: i32::from(chunk.screen_x_px) - 0x80,
        y: i32::from(chunk.screen_y_with_bias) - 0x80,
        tile_sheet: Arc::new(visuals.tile_sheet.clone()),
        palette: Arc::new(visuals.palette.clone()),
        tiles,
    });
    true
}

fn push_lava_geyser_template_loc_14be8(
    commands: &mut Vec<DrawCommand>,
    visuals: &kid_render::LavaVisuals,
    geyser_x_px: u16,
    camera_x_px: u16,
    screen_y_px: i32,
    templates: &[kid_core::LavaGeyserSpriteTemplateLoc14BE8],
) {
    let initial_screen_y_with_bias = (screen_y_px + 0x90).clamp(0, i32::from(u16::MAX)) as u16;
    for chunk in kid_core::lava_geyser_sprite_chunks_loc_14BE8(
        geyser_x_px,
        camera_x_px,
        initial_screen_y_with_bias,
        0,
        templates,
    ) {
        push_lava_geyser_chunk_loc_14be8(commands, visuals, chunk);
    }
}

fn push_lava_geyser_rom_sprites_loc_14be8(
    commands: &mut Vec<DrawCommand>,
    visuals: &kid_render::LavaVisuals,
    geyser_x_px: u16,
    geyser_top_y_px: u16,
    height_px: u16,
    camera_x_px: u16,
    camera_y_px: u16,
) {
    let top_screen_y = i32::from(geyser_top_y_px) - i32::from(camera_y_px);
    push_lava_geyser_template_loc_14be8(
        commands,
        visuals,
        geyser_x_px,
        camera_x_px,
        top_screen_y,
        &LAVA_GEYSER_TOP_TEMPLATE_UNK_14624,
    );

    let middle_top = top_screen_y + 16;
    let middle_bottom = top_screen_y + i32::from(height_px).saturating_sub(16);
    let mut row_y = middle_top;
    while row_y < middle_bottom {
        push_lava_geyser_template_loc_14be8(
            commands,
            visuals,
            geyser_x_px,
            camera_x_px,
            row_y,
            &LAVA_GEYSER_MIDDLE_TEMPLATE_UNK_1465A,
        );
        row_y += 16;
    }

    push_lava_geyser_template_loc_14be8(
        commands,
        visuals,
        geyser_x_px,
        camera_x_px,
        top_screen_y + i32::from(height_px).saturating_sub(16),
        &LAVA_GEYSER_BOTTOM_TEMPLATE_UNK_14650,
    );
}

fn push_eol_number_sprites_with_palette(
    commands: &mut Vec<DrawCommand>,
    eol: &kid_render::EolScoreVisuals,
    palette: &Palette,
    x: i32,
    y: i32,
    value: u32,
) {
    let mut remaining = value.min(9_999_999);
    let mut started = false;
    for (slot, divisor) in [1_000_000, 100_000, 10_000, 1_000, 100, 10, 1]
        .into_iter()
        .enumerate()
    {
        let digit = remaining / divisor;
        remaining %= divisor;
        if digit == 0 && !started && slot < 6 {
            continue;
        }
        started = true;
        if let Some(frame) = eol.digits.get(digit as usize) {
            // ASM Object_EOL_Number loc_C466 sets `priority(a1)=1`, and
            // loc_C4E2 spaces seven digit workers 12px apart.
            push_sprite_frame_with_priority(
                commands,
                frame,
                palette,
                x + slot as i32 * 12,
                y,
                Some(true),
            );
        }
    }
}

fn push_eol_level_skip_time_digits_with_palette_loc_d47e(
    commands: &mut Vec<DrawCommand>,
    eol: &kid_render::EolScoreVisuals,
    palette: &Palette,
    trace: &NativeTraceFrame,
) {
    if trace.level_skip_cheat_enabled == 0 || trace.level_complete_score_rows_started == 0 {
        return;
    }

    // ASM loc_D468/loc_D47E: when `LevelSkip_Cheat` is set, emit three
    // completion-time digit gfx objects at camera-relative x `$11C`, `$110`,
    // `$104`, y `$D4`, consuming ones/tens/hundreds by repeated `divu #$A`.
    // These are direct gfx-object coordinates, not pre-resolved top-lefts.
    let mut remaining = trace.level_completion_time;
    for x in [0x11C, 0x110, 0x104] {
        let digit = remaining % 10;
        remaining /= 10;
        if let Some(frame) = eol.digits.get(usize::from(digit)) {
            // ASM loc_D47E allocates the level-skip time digits through the
            // same priority=$01 EOL digit gfx-object setup as loc_C466.
            push_sprite_frame_with_priority(commands, frame, palette, x, 0xD4, Some(true));
        }
    }
}

fn eol_position_loc_c5a2(initial: i32, velocity: i32, timer: usize, elapsed: usize) -> i32 {
    // ASM Object_EOL_Text/Object_EOL_Number loc_C5A2 updates `$64/$68` by
    // `$6C/$70` until `$46` or `$A` reaches zero, then clears velocity and
    // leaves the worker anchored at its settled coordinate.
    let moving_frames = elapsed.min(timer);
    initial + velocity * i32::try_from(moving_frames).unwrap_or(i32::MAX)
}

fn eol_number_position_loc_c48c(initial: i32, velocity: i32, timer: usize, elapsed: usize) -> i32 {
    // ASM Object_EOL_Number loc_C48C draws digits from `$64/$68`, then loc_C4E2
    // advances `$64/$68` and decrements `$A`. Text workers in loc_C5A2 move
    // before drawing, so EOL numbers are visibly one worker tick behind text.
    let moving_frames = elapsed.saturating_sub(1).min(timer);
    initial + velocity * i32::try_from(moving_frames).unwrap_or(i32::MAX)
}

fn push_eol_text_loc_c5a2(
    commands: &mut Vec<DrawCommand>,
    frame: &kid_render::SpriteFrame,
    eol: &kid_render::EolScoreVisuals,
    initial_x: i32,
    y: i32,
    timer: usize,
    elapsed: usize,
) {
    push_eol_text_with_velocity_loc_c5a2(commands, frame, eol, initial_x, 8, y, timer, elapsed);
}

fn push_eol_text_with_palette_loc_c5a2(
    commands: &mut Vec<DrawCommand>,
    frame: &kid_render::SpriteFrame,
    eol: &kid_render::EolScoreVisuals,
    palette: &Palette,
    initial_x: i32,
    y: i32,
    timer: usize,
    elapsed: usize,
) {
    push_eol_text_with_velocity_and_palette_loc_c5a2(
        commands, frame, eol, palette, initial_x, 8, y, timer, elapsed,
    );
}

fn push_eol_text_with_velocity_loc_c5a2(
    commands: &mut Vec<DrawCommand>,
    frame: &kid_render::SpriteFrame,
    eol: &kid_render::EolScoreVisuals,
    initial_x: i32,
    velocity: i32,
    y: i32,
    timer: usize,
    elapsed: usize,
) {
    push_eol_text_with_velocity_and_palette_loc_c5a2(
        commands,
        frame,
        eol,
        &eol.palette,
        initial_x,
        velocity,
        y,
        timer,
        elapsed,
    );
}

fn push_eol_text_with_velocity_and_palette_loc_c5a2(
    commands: &mut Vec<DrawCommand>,
    frame: &kid_render::SpriteFrame,
    _eol: &kid_render::EolScoreVisuals,
    palette: &Palette,
    initial_x: i32,
    velocity: i32,
    y: i32,
    timer: usize,
    elapsed: usize,
) {
    let x = eol_position_loc_c5a2(initial_x, velocity, timer, elapsed);
    // ASM Object_EOL_Text loc_C5A2 writes directly to the gfx object's
    // x_pos/y_pos fields after adding the camera, so render from sprite
    // coordinates rather than treating `$64/$68` as top-lefts. loc_C592 sets
    // `priority(a3)=1`, so these rows draw above high-priority foreground.
    push_sprite_frame_with_priority(commands, frame, palette, x, y, Some(true));
}

fn push_eol_number_loc_c5a2(
    commands: &mut Vec<DrawCommand>,
    eol: &kid_render::EolScoreVisuals,
    initial_x: i32,
    velocity: i32,
    y: i32,
    timer: usize,
    elapsed: usize,
    value: u32,
) {
    push_eol_number_with_palette_loc_c5a2(
        commands,
        eol,
        &eol.palette,
        initial_x,
        velocity,
        y,
        timer,
        elapsed,
        value,
    );
}

fn push_eol_number_with_palette_loc_c5a2(
    commands: &mut Vec<DrawCommand>,
    eol: &kid_render::EolScoreVisuals,
    palette: &Palette,
    initial_x: i32,
    velocity: i32,
    y: i32,
    timer: usize,
    elapsed: usize,
    value: u32,
) {
    let x = eol_number_position_loc_c48c(initial_x, velocity, timer, elapsed);
    // ASM Object_EOL_Number loc_C4E2 writes each digit worker's gfx-object
    // x_pos/y_pos directly from `$64/$68`.
    push_eol_number_sprites_with_palette(commands, eol, palette, x, y, value);
}

fn push_eol_bonus_row_with_palette_loc_d468(
    commands: &mut Vec<DrawCommand>,
    eol: &kid_render::EolScoreVisuals,
    palette: &Palette,
    label: &kid_render::SpriteFrame,
    label_initial_x: i32,
    bonus_initial_x: i32,
    y: i32,
    number_initial_x: i32,
    timer: usize,
    elapsed: usize,
    value: u32,
) {
    push_eol_text_with_palette_loc_c5a2(
        commands,
        label,
        eol,
        palette,
        label_initial_x,
        y,
        timer,
        elapsed,
    );
    push_eol_text_with_palette_loc_c5a2(
        commands,
        &eol.bonus,
        eol,
        palette,
        bonus_initial_x,
        y,
        timer,
        elapsed,
    );
    push_eol_number_with_palette_loc_c5a2(
        commands,
        eol,
        palette,
        number_initial_x,
        -8,
        y,
        timer,
        elapsed,
        value,
    );
}

fn eol_bonus_amounts_for_trace(trace: &NativeTraceFrame) -> ([u32; 5], usize) {
    let mut amounts = [0; 5];
    let mut count = 0;
    let mut push = |amount: u32| {
        amounts[count] = amount;
        count += 1;
    };

    push(kid_core::time_bonus_loc_d56e(trace.level_timer_seconds));
    if !trace.level_damage_taken {
        push(5_000);
    }
    if !trace.level_prize_collected {
        push(5_000);
    }
    if trace.level_path_bonus_amount > 0 {
        push(trace.level_path_bonus_amount);
    }
    if trace.level_speed_bonus_awarded {
        push(10_000);
    }

    (amounts, count)
}

fn eol_score_counter_snapshot_for_trace(
    trace: &NativeTraceFrame,
) -> kid_core::EolScoreCounterSnapshotLocD770 {
    let (amounts, count) = eol_bonus_amounts_for_trace(trace);
    let total_bonus = amounts[..count].iter().copied().sum::<u32>();
    let initial_score = if trace.level_complete && trace.level_complete_initial_score > 0 {
        trace.level_complete_initial_score
    } else {
        trace.score.saturating_sub(total_bonus)
    };
    let transfer_frames = kid_core::eol_score_counter_transfer_frames_from_trace_loc_D770(
        trace.level_complete_score_counter_elapsed_frames,
    );
    kid_core::eol_score_counter_snapshot_loc_D770(initial_score, &amounts[..count], transfer_frames)
}

fn eol_score_room_palette_loc_d734(trace: &NativeTraceFrame, palette: &Palette) -> Palette {
    if trace.level_complete_score_rows_elapsed_frames
        < EOL_SCORE_ROWS_RENDER_FADE_IN_FRAMES_LOC_D734_ORACLE
    {
        return palette.clone();
    }

    // ASM loc_D734: after the score-row setup, the ROM backs up
    // `Palette_Buffer`, clears `Palette_Buffer_3`, then runs eight
    // `sub_14C0` fade steps to `$0080`. The native asset stores the extracted
    // score-room colours as one flattened Pal_D00C palette, but oracle frames
    // show the grid greens/white stay on the bright CRAM line while the room
    // wall/stone colours are the dimmed loc_D734 result.
    Palette {
        colours: palette
            .colours
            .iter()
            .map(|colour| {
                let raw_word = match colour.raw_word {
                    0x0000 | 0x0040 | 0x00E0 | 0x0EEE => colour.raw_word,
                    raw_word => kid_core::eol_title_return_faded_palette_word_loc_D734(raw_word),
                };
                kid_render::MdColour::from_word(raw_word)
            })
            .collect(),
    }
}

fn push_eol_score_overlay(
    commands: &mut Vec<DrawCommand>,
    trace: &NativeTraceFrame,
    eol: &kid_render::EolScoreVisuals,
) {
    let palette = eol_score_room_palette_loc_d734(trace, &eol.palette);
    push_eol_level_skip_time_digits_with_palette_loc_d47e(commands, eol, &palette, trace);

    if trace.level_complete_score_rows_elapsed_frames
        < EOL_SCORE_ROWS_RENDER_FADE_IN_FRAMES_LOC_D734_ORACLE
    {
        return;
    }

    // ASM loc_D4CE emits CONTINUES and SCORE first, then the bonus rows with
    // Load_Object_EOL_Text and Load_Object_EOL_Number. Native follows the ROM
    // `$64/$68/$6C` seeds directly and advances them through loc_C5A2.
    // ASM loc_D770 then drains each number object's `$60` into the score
    // display; native derives that visual state from the level-complete
    // elapsed frame while keeping progression score awarded at flag touch.
    let counter = eol_score_counter_snapshot_for_trace(trace);
    let elapsed = trace.level_complete_score_rows_elapsed_frames
        - EOL_SCORE_ROWS_RENDER_FADE_IN_FRAMES_LOC_D734_ORACLE
        + EOL_SCORE_ROWS_RENDER_LATCH_LEAD_FRAMES_LOC_C070_ORACLE;
    push_eol_text_with_palette_loc_c5a2(
        commands,
        &eol.continues,
        eol,
        &palette,
        -0xB8,
        0xCC,
        0x1C,
        elapsed,
    );
    push_eol_number_with_palette_loc_c5a2(
        commands,
        eol,
        &palette,
        -0x7C,
        8,
        0xCC,
        0x1C,
        elapsed,
        u32::from(trace.number_continues),
    );
    push_eol_text_with_palette_loc_c5a2(
        commands, &eol.score, eol, &palette, -0x68, 0xB4, 0x1C, elapsed,
    );
    push_eol_number_with_palette_loc_c5a2(
        commands,
        eol,
        &palette,
        0x1B0,
        -8,
        0xB4,
        0x1C,
        elapsed,
        counter.score,
    );

    let mut amount_index = 0usize;
    let mut y = 0x9C;
    let mut timer = 0x24usize;
    // ASM loc_D4CE subtracts `$18` from d7 and adds `$40` to d6 after the
    // SCORE row, before loading the first TIME/BONUS row.
    let mut d6 = 0x40;
    push_eol_bonus_row_with_palette_loc_d468(
        commands,
        eol,
        &palette,
        &eol.time,
        -0xA0 - d6,
        -0x68 - d6,
        y,
        0x1B0 + d6,
        timer,
        elapsed,
        counter.remaining_amounts[amount_index],
    );
    amount_index += 1;
    y -= 0x18;
    timer += 8;
    d6 += 0x40;
    if !trace.level_damage_taken {
        push_eol_bonus_row_with_palette_loc_d468(
            commands,
            eol,
            &palette,
            &eol.no_hit,
            -0xC8 - d6,
            -0x68 - d6,
            y,
            0x1B0 + d6,
            timer,
            elapsed,
            counter.remaining_amounts[amount_index],
        );
        amount_index += 1;
        y -= 0x18;
        timer += 8;
        d6 += 0x40;
    }
    if !trace.level_prize_collected {
        push_eol_bonus_row_with_palette_loc_d468(
            commands,
            eol,
            &palette,
            &eol.no_prize,
            -0xE0 - d6,
            -0x68 - d6,
            y,
            0x1B0 + d6,
            timer,
            elapsed,
            counter.remaining_amounts[amount_index],
        );
        amount_index += 1;
        y -= 0x18;
        timer += 8;
        d6 += 0x40;
    }
    if trace.level_path_bonus_amount > 0 {
        push_eol_bonus_row_with_palette_loc_d468(
            commands,
            eol,
            &palette,
            &eol.path,
            -0xA8 - d6,
            -0x68 - d6,
            y,
            0x1B0 + d6,
            timer,
            elapsed,
            counter.remaining_amounts[amount_index],
        );
        amount_index += 1;
        y -= 0x18;
        timer += 8;
        d6 += 0x40;
    }
    if trace.level_speed_bonus_awarded {
        push_eol_bonus_row_with_palette_loc_d468(
            commands,
            eol,
            &palette,
            &eol.speed,
            -0xB0 - d6,
            -0x68 - d6,
            y,
            0x1B0 + d6,
            timer,
            elapsed,
            counter.remaining_amounts[amount_index],
        );
        y -= 0x18;
        timer += 8;
        d6 += 0x40;
        push_eol_text_with_palette_loc_c5a2(
            commands,
            &eol.under,
            eol,
            &palette,
            -0xB0 - d6,
            y,
            timer,
            elapsed,
        );
        // ASM loc_D660: the "UNDER ... SECONDS" number is the threshold byte
        // loaded from SpeedBonus_Values[Current_LevelID], not remaining time.
        let speed_threshold = kid_core::speed_bonus_threshold_loc_d660(trace.current_level)
            .map_or(u32::from(trace.level_timer_seconds), u32::from);
        push_eol_number_with_palette_loc_c5a2(
            commands,
            eol,
            &palette,
            -0xA4 - d6,
            8,
            y,
            timer,
            elapsed,
            speed_threshold,
        );
        push_eol_text_with_palette_loc_c5a2(
            commands,
            &eol.seconds,
            eol,
            &palette,
            -0x4C - d6,
            y,
            timer,
            elapsed,
        );
    }
}

fn push_game_over_continue_overlay(
    commands: &mut Vec<DrawCommand>,
    trace: &NativeTraceFrame,
    eol: &kid_render::EolScoreVisuals,
) {
    if let Some(_countdown) = trace.continue_countdown {
        // ASM Continue_Screen (kid.asm:16505): allocate Spr_EOL_Continues at
        // `$64=$FF88,$68=$60,$6C=8`, then Object_EOL_Number at
        // `$64=$134,$68=$80,$6C=$FFF8` using Number_Continues. The ROM then
        // hibernates for `$1C` frames at loc_D31E before loc_D350 starts the
        // interactive countdown; keep those objects at their spawn coordinates
        // until the same hold has elapsed.
        let elapsed = trace
            .continue_countdown_elapsed_frames
            .saturating_sub(CONTINUE_SCREEN_INITIAL_HOLD_FRAMES_LOC_D31E);
        push_eol_text_loc_c5a2(commands, &eol.continues, eol, -0x78, 0x60, 8, elapsed);
        push_eol_number_loc_c5a2(
            commands,
            eol,
            0x134,
            -8,
            0x80,
            8,
            elapsed,
            u32::from(trace.number_continues),
        );
    } else if trace.number_continues == 0 {
        let elapsed = trace.game_over_elapsed_frames;
        // ASM loc_D25E (kid.asm:16472): with no continues, GAME enters from
        // x=$FF98 with +8 velocity and OVER enters from x=$158 with -8.
        // If continues remain, the ROM branches to Continue_Screen instead of
        // showing GAME OVER, so native must not render a pre-continue panel.
        push_eol_text_loc_c5a2(commands, &eol.game, eol, -0x68, 0x60, 8, elapsed);
        push_eol_text_with_velocity_loc_c5a2(commands, &eol.over, eol, 0x158, -8, 0x80, 8, elapsed);
    }
}

fn push_pause_menu_text_loc_69d2(commands: &mut Vec<DrawCommand>, trace: &NativeTraceFrame) {
    // ASM loc_69D2/restart_text writes a 16x5-tile pause menu block at
    // screen-space Camera_X+$60, Camera_Y+$40. The ROM chooses the lower row
    // text from Number_Lives: >1 means Restart Round, otherwise Give Up.
    let lower_row = if trace.number_lives > 1 {
        "RESTART ROUND"
    } else {
        "GIVE UP"
    };

    // ASM loc_6A84 seeds Pause_Option=0 and writes the upper cursor command at
    // Camera_X+$68, Camera_Y+$48; Game_Paused_Loop loc_6B0E moves it to the
    // lower row at Camera_Y+$58 for Restart Round / Give Up.
    let cursor_y = if trace.pause_menu_option == 0 {
        PAUSE_MENU_TEXT_TOP_Y_RESTART_TEXT
    } else {
        PAUSE_MENU_TEXT_BOTTOM_Y_RESTART_TEXT
    };
    commands.push(text(
        PAUSE_MENU_CURSOR_X_LOC_6A84,
        cursor_y,
        ">",
        [255, 224, 96],
        1,
    ));
    commands.push(text(
        PAUSE_MENU_TEXT_X_LOC_69D2,
        PAUSE_MENU_TEXT_TOP_Y_RESTART_TEXT,
        "CONTINUE",
        [255, 255, 255],
        1,
    ));
    commands.push(text(
        PAUSE_MENU_TEXT_X_LOC_69D2,
        PAUSE_MENU_TEXT_BOTTOM_Y_RESTART_TEXT,
        lower_row,
        [255, 255, 255],
        1,
    ));
}

// Forest water ripple table from ASM: BackgroundScroll_ForestWaterRippleData
// (kid.asm line 34310). 64 signed byte entries, max amplitude 8px.
const FOREST_RIPPLE: [i8; 64] = [
    0, 0, 0, 0, 1, 1, 1, 2, 2, 2, 3, 3, 4, 5, 6, 7, 7, 7, 8, 8, 8, 8, 7, 7, 7, 6, 5, 4, 3, 2, 2, 2,
    1, 1, 1, 0, 0, 0, 0, 1, 2, 3, 4, 4, 4, 4, 5, 6, 7, 8, 8, 8, 7, 7, 6, 6, 5, 5, 4, 4, 3, 3, 2, 1,
];
const SETTLED_WARP_FOREST_RIPPLE_TIME_FRAMES_OFFSET_LOC_305E6: usize = 52;

pub(super) fn forest_water_ripple_time_frames_for_trace_loc_305e6(
    trace: &NativeTraceFrame,
) -> usize {
    // ASM loc_305E6 uses the global `Time_Frames` counter, not the
    // recording-local frame index. Settled warp recordings replay through the
    // ROM level-skip path and keep `LevelSkip_Cheat` armed; their visible
    // forest ripple is 52 VBlanks ahead of the rebased native recording clock.
    if trace.level_skip_cheat_enabled != 0 {
        trace
            .frame_index
            .saturating_add(SETTLED_WARP_FOREST_RIPPLE_TIME_FRAMES_OFFSET_LOC_305E6)
    } else {
        trace.frame_index
    }
}

pub(super) fn forest_water_ripple_offset_for_screen_y_sub_30598(
    frame_index: usize,
    screen_y: i32,
    effect_start_row_selector: u8,
    camera_y_px: i32,
) -> Option<i32> {
    // ASM BackgroundScroll_ApplyForestWaterRipple loc_30598-loc_3060A:
    // start at `(map_header+$14)[0] * 8 - (Camera_Y_pos >> 2)`, affect at
    // most 64 visible scanlines, then add one table byte per screen line.
    let start_y = i32::from(effect_start_row_selector) * 8 - (camera_y_px >> 2);
    if !(0..FRAME_HEIGHT as i32).contains(&screen_y) {
        return None;
    }
    let visible_start = start_y.max(0);
    let table_start = if start_y < 0 { -start_y } else { 0 };
    let line_count = if start_y < 0 {
        (64 + start_y).clamp(0, 64)
    } else {
        (FRAME_HEIGHT as i32 - start_y).clamp(0, 64)
    };
    let visible_end = visible_start + line_count;
    if screen_y < visible_start || screen_y >= visible_end {
        return None;
    }

    let time_phase = (frame_index / 4) & 0x3F;
    let table_idx = (time_phase + (table_start + screen_y - visible_start) as usize) & 0x3F;
    Some(i32::from(FOREST_RIPPLE[table_idx]))
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
    if level_visuals.level.background_scroll.is_none() {
        return (
            world_x_px.saturating_sub(camera_x_px),
            world_y_px.saturating_sub(background_scroll_y_px),
        );
    }

    let layer_speed = background_layer_speed_loc_3050a(level_visuals, usize::from(y_tiles));
    (
        world_x_px.saturating_sub(background_scroll_x_px(trace, camera_x_px, layer_speed)),
        world_y_px.saturating_sub(background_scroll_y_px),
    )
}

fn background_layer_speed_loc_3050a(level_visuals: &NativeLevelVisuals, y_tiles: usize) -> u8 {
    level_visuals
        .level
        .background_scroll
        .as_ref()
        .and_then(|profile| profile.layer_speeds.get(y_tiles).copied())
        .unwrap_or(0x10)
}

pub(super) fn layered_background_row_wraps_loc_3050a(
    level_visuals: &NativeLevelVisuals,
    y_tiles: usize,
) -> bool {
    // ASM BackgroundScroll_ComputeShiftLayers loc_3050A resolves bgscroll rows
    // from the per-level profile and writes 224 scanline scroll values into
    // Plane B.  Hills `$08` rows are finite map rows inside a horizontally
    // wrapping VDP plane, so Level 0x08's early sky rows must re-enter at the
    // right edge instead of exposing the CRAM backdrop.  Auto-scrolling forest
    // rows (`$00/$08`) are still left finite here; wrapping them paints scenery
    // into the oracle's clear sky during the flagpole hand-off before loc_C0CC
    // sets no-scroll mode.
    let layer_speed = background_layer_speed_loc_3050a(level_visuals, y_tiles);
    if level_visuals.level.background_theme_id == 3 && layer_speed == 0x08 {
        return true;
    }
    matches!(layer_speed, 0x18..=0x88 if (layer_speed - 0x10).is_multiple_of(8))
}

pub(super) fn layered_background_row_wraps_for_trace_loc_3050a(
    trace: &NativeTraceFrame,
    level_visuals: &NativeLevelVisuals,
    y_tiles: usize,
) -> bool {
    // ASM CharacterCollision_TouchFlagpole loc_C070 starts a frozen flagpole
    // hand-off. Native reports `level_complete` on the same flag-touch trace
    // tick, but oracle pixels still contain the previous Scroll B frame only
    // for elapsed=0; later hand-off frames stop repeating the forest rows.
    (!trace.level_complete || trace.level_complete_elapsed_frames == 0)
        && layered_background_row_wraps_loc_3050a(level_visuals, y_tiles)
}

pub(super) fn wrap_layered_background_x_loc_11b7a(screen_x_px: i32, row_width_px: i32) -> i32 {
    if row_width_px <= 0 {
        return screen_x_px;
    }
    // ASM loc_11B7A builds Plane B row pointers and the VDP scroll plane wraps
    // horizontally. Forest rows are composed as two 32-tile planes (64 tiles);
    // without this wrap, parallax-scrolled rows that move more than one full
    // plane disappear to the left. Do not fold ordinary right-edge tiles back
    // into the viewport until they cross the whole Plane B row; the ROM still
    // leaves clear sky between x=320 and the 512px nametable wrap point.
    let mut wrapped = screen_x_px;
    while wrapped < -8 {
        wrapped += row_width_px;
    }
    while wrapped >= row_width_px {
        wrapped -= row_width_px;
    }
    wrapped
}

pub(super) fn wrap_layered_background_y_loc_11b7a(screen_y_px: i32, row_height_px: i32) -> i32 {
    if row_height_px <= 0 {
        return screen_y_px;
    }
    // ASM loc_11B7A backs Plane B with a row-pointer table and
    // BackgroundScroll_ComputeShiftLayers biases the visible rows from
    // Camera_Y_pos. Native must wrap the composed Plane B rows vertically so
    // tall Mountain levels keep background pixels behind transparent terrain.
    let mut wrapped = screen_y_px;
    while wrapped < -8 {
        wrapped += row_height_px;
    }
    while wrapped >= row_height_px {
        wrapped -= row_height_px;
    }
    wrapped
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
    foreground_palette: &Palette,
    x: i32,
    y: i32,
    quad: &kid_render::TileQuad,
) {
    for (index, tile_ref) in quad.tiles.iter().copied().enumerate() {
        let tile_x = x + if index % 2 == 0 { 0 } else { 8 };
        let tile_y = y + if index < 2 { 0 } else { 8 };
        if let Some(tile) = resolve_ingame_vram_tile_with_foreground_palette(
            level_visuals,
            foreground_palette,
            tile_ref,
        ) {
            push_resolved_tile(commands, tile_x, tile_y, tile);
        }
    }
}

fn push_tile_quad_with_priority_override(
    commands: &mut Vec<DrawCommand>,
    level_visuals: &NativeLevelVisuals,
    foreground_palette: &Palette,
    x: i32,
    y: i32,
    quad: &kid_render::TileQuad,
    priority_override: Option<bool>,
) {
    match priority_override {
        Some(priority) => {
            let mut quad = quad.clone();
            for tile in &mut quad.tiles {
                tile.priority = priority;
            }
            push_tile_quad(commands, level_visuals, foreground_palette, x, y, &quad);
        }
        None => push_tile_quad(commands, level_visuals, foreground_palette, x, y, quad),
    }
}

fn prize_block_wobble_frame_sub_1007a(
    level_visuals: &NativeLevelVisuals,
    remaining: u8,
) -> kid_render::SpriteFrame {
    let elapsed = 31_u8.saturating_sub(remaining);
    // ASM stru_101B6/stru_10296 (kid.asm:21447,21529): the prize-block
    // wobble worker alternates 16x16 sprite frames at VRAM tiles $220 and
    // $229 for three hibernated frames each while sub_1007A moves y_pos.
    let tile_base_index = if (elapsed / 3).is_multiple_of(2) {
        0x000
    } else {
        0x009
    };
    kid_render::SpriteFrame {
        origin_x: 0x00,
        origin_y: 0x10,
        width_px: 0x10,
        height_px: 0x10,
        tile_base_index,
        tile_sheet: level_visuals.block_tiles.clone(),
    }
}

fn push_prize_block_wobble_sub_1007a(
    commands: &mut Vec<DrawCommand>,
    level_visuals: &NativeLevelVisuals,
    x: i32,
    y: i32,
    remaining: u8,
) {
    let frame = prize_block_wobble_frame_sub_1007a(level_visuals, remaining);
    push_sprite_frame_top_left(commands, &frame, &level_visuals.palettes.foreground, x, y);
}

fn prize_block_spawn_flash_frame_loc_102b8(
    level_visuals: &NativeLevelVisuals,
    remaining: u8,
) -> Option<kid_render::SpriteFrame> {
    let elapsed = PRIZE_BLOCK_SPAWN_FLASH_TOTAL_FRAMES_LOC_102B8.saturating_sub(remaining);
    let mut frame_start = 0_u8;
    // ASM loc_102B8/stru_102DE (kid.asm:21578) initialises four 24x24 sprite
    // frames at block-art VRAM tiles $2CF, $2D8, $2E1, and $2EA.
    for (duration, vram_tile) in PRIZE_BLOCK_SPAWN_FLASH_FRAMES_STRU_102DE {
        let frame_end = frame_start.saturating_add(duration);
        if elapsed < frame_end || frame_end == 24 {
            let tile_base_index = vram_tile.checked_sub(BLOCK_TILE_VRAM_BASE)?;
            if usize::from(tile_base_index.saturating_add(8))
                >= level_visuals.block_tiles.tiles.len()
            {
                return None;
            }
            return Some(kid_render::SpriteFrame {
                origin_x: 0x0C,
                origin_y: 0x0C,
                width_px: 0x18,
                height_px: 0x18,
                tile_base_index,
                tile_sheet: level_visuals.block_tiles.clone(),
            });
        }
        frame_start = frame_end;
    }
    None
}

fn push_prize_block_spawn_flash_loc_102b8(
    commands: &mut Vec<DrawCommand>,
    level_visuals: &NativeLevelVisuals,
    x: i32,
    y: i32,
    remaining: u8,
) -> bool {
    let Some(frame) = prize_block_spawn_flash_frame_loc_102b8(level_visuals, remaining) else {
        return false;
    };
    push_sprite_frame(commands, &frame, &level_visuals.palettes.foreground, x, y);
    true
}

fn push_kid_projectile_hit_spark_loc_1f08(
    commands: &mut Vec<DrawCommand>,
    level_visuals: &NativeLevelVisuals,
    x: i32,
    y: i32,
    remaining: u8,
) {
    let elapsed = 4_u8.saturating_sub(remaining);
    let mut frame_start = 0_u8;
    // ASM loc_1F08/stru_1F40 (kid.asm:3716,3730): four one-frame 24x24
    // mappings from the same `$2CF/$2D8/$2E1/$2EA` block-art VRAM range used
    // by the prize-block flash family, but with palette line 3.
    for (duration, vram_tile) in KID_PROJECTILE_HIT_SPARK_FRAMES_STRU_1F40 {
        let frame_end = frame_start.saturating_add(duration);
        if elapsed < frame_end || frame_end == 4 {
            let Some(tile_base_index) = vram_tile.checked_sub(BLOCK_TILE_VRAM_BASE) else {
                break;
            };
            if usize::from(tile_base_index.saturating_add(8))
                < level_visuals.block_tiles.tiles.len()
            {
                let frame = kid_render::SpriteFrame {
                    origin_x: 0x0C,
                    origin_y: 0x0C,
                    width_px: 0x18,
                    height_px: 0x18,
                    tile_base_index,
                    tile_sheet: level_visuals.block_tiles.clone(),
                };
                push_sprite_frame(commands, &frame, &level_visuals.palettes.foreground, x, y);
                return;
            }
            break;
        }
        frame_start = frame_end;
    }

    commands.push(DrawCommand::Rect {
        x: x - 3,
        y: y - 3,
        width: 6,
        height: 6,
        colour: [255, 240, 96],
    });
}

pub(super) fn push_level_layout_terrain(
    commands: &mut Vec<DrawCommand>,
    trace: &NativeTraceFrame,
    level_visuals: &NativeLevelVisuals,
    block_grid: Option<&BlockGrid>,
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
    let foreground_palette = foreground_palette_for_scene(trace, level_visuals);

    for row in row_start..row_end {
        for col in col_start..col_end {
            let raw_word = block_grid.map(|grid| grid.raw_word_at(col, row));
            let priority_override = raw_word
                .filter(|word| word & 0x8100 == 0x0100)
                .map(|_| true);
            // ASM sub_302F6/sub_3034A draws Scroll A base terrain from
            // Level_Layout; runtime block-grid writes are collision/block
            // overlay state and cleared block cells must not erase the art.
            let skin = layout.skin_at(col, row);
            let Some(resolved) = level_visuals.level.block_mapping.resolve(skin, None) else {
                continue;
            };
            // ASM sub_302F6/sub_3034A ORs `$8000` into Scroll A tile
            // words when the raw Level_Layout terrain cell carries bit
            // `$0100`; Cell remains the collision-normalised view.
            push_tile_quad_with_priority_override(
                commands,
                level_visuals,
                &foreground_palette,
                i32::from(col) * 16 - camera_x_px,
                i32::from(row) * 16 - camera_y_px,
                resolved.quad,
                priority_override,
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
    let foreground_palette = foreground_palette_for_scene(trace, level_visuals);

    for row in row_start..row_end {
        for col in col_start..col_end {
            let cell = grid.cell_at(col, row);
            let bounce_y_offset = trace
                .block_bounces
                .iter()
                .find(|(cx, cy, _, _)| *cx == col && *cy == row)
                .map(|(_, _, remaining, y)| (*remaining, i32::from(*y)));
            if !cell.has_block && bounce_y_offset.is_none() {
                continue;
            }
            if bounce_y_offset.is_some_and(|(remaining, _)| remaining > 8) {
                continue;
            }

            let Some(resolved) = level_visuals.level.block_mapping.resolve(
                cell.skin,
                Some(if cell.has_block {
                    cell.block_type as u8
                } else {
                    // ASM sub_1007A writes `$6Fxx` while the prize-block
                    // wobble object remains live. The block flag is clear,
                    // but the worker still draws the block sprite frames
                    // (`unk_E0E4E`/`unk_E0E5E`) at the bounced position.
                    kid_core::terrain::BlockType::Prize as u8
                }),
            ) else {
                continue;
            };
            // ASM sub_1007A wobble: if the cell is in `block_bounces`, offset
            // the draw y_pos by the engine-supplied value so the block visibly
            // shakes up/down before settling.
            let y_offset = bounce_y_offset.map_or(0, |(_, y)| y);
            push_tile_quad(
                commands,
                level_visuals,
                &foreground_palette,
                i32::from(col) * 16 - camera_x_px,
                i32::from(row) * 16 - camera_y_px + y_offset,
                resolved.quad,
            );
        }
    }

    for &(col, row, remaining, y_offset) in &trace.block_bounces {
        if remaining <= 8 {
            continue;
        }
        push_prize_block_wobble_sub_1007a(
            commands,
            level_visuals,
            i32::from(col) * 16 - camera_x_px,
            i32::from(row) * 16 - camera_y_px + i32::from(y_offset),
            remaining,
        );
    }
}

pub(super) fn background_colour(level_visuals: &NativeLevelVisuals) -> [u8; 3] {
    level_visuals
        .palettes
        .background
        .colours
        .first()
        .map_or(SKY_COLOUR, |colour| [colour.red, colour.green, colour.blue])
}

fn push_eol_hologram_room(
    commands: &mut Vec<DrawCommand>,
    trace: &NativeTraceFrame,
    level_visuals: &NativeLevelVisuals,
) {
    // ASM loc_C186 uploads MapEni_CC0E to Scroll B after the flag-touch
    // hibernate. Scroll A stays live until the loc_C21E/sub_C254 reveal
    // writes replace its foreground cells.
    if let Some(hologram) = &level_visuals.eol_hologram {
        let tile_sheet = Arc::new(hologram.tile_sheet.clone());
        let palette = Arc::new(eol_score_room_palette_loc_d734(trace, &hologram.palette));
        for (row_index, row) in hologram.rows.iter().enumerate() {
            for (col_index, tile) in row.iter().copied().enumerate() {
                commands.push(DrawCommand::IndexedTile {
                    x: (col_index * 8) as i32,
                    y: (row_index * 8) as i32,
                    tile_sheet: Arc::clone(&tile_sheet),
                    tile,
                    palette: Arc::clone(&palette),
                });
            }
        }
        return;
    }

    commands.push(DrawCommand::Rect {
        x: 0,
        y: 0,
        width: FRAME_WIDTH as i32,
        height: FRAME_HEIGHT as i32,
        colour: [0, 0, 0],
    });

    let grid_colour = [255, 112, 0];
    for y in (32..190).step_by(16) {
        commands.push(DrawCommand::Rect {
            x: 0,
            y,
            width: FRAME_WIDTH as i32,
            height: 2,
            colour: grid_colour,
        });
    }
    for x in (0..FRAME_WIDTH as i32).step_by(16) {
        commands.push(DrawCommand::Rect {
            x,
            y: 32,
            width: 2,
            height: 158,
            colour: grid_colour,
        });
    }

    let stone_dark = [64, 72, 96];
    let stone_light = [144, 152, 184];
    for x in [176, 224] {
        commands.push(DrawCommand::Rect {
            x,
            y: 0,
            width: 16,
            height: 152,
            colour: stone_dark,
        });
        for y in (0..152).step_by(16) {
            commands.push(DrawCommand::Rect {
                x,
                y,
                width: 16,
                height: 2,
                colour: stone_light,
            });
            commands.push(DrawCommand::Rect {
                x,
                y,
                width: 2,
                height: 16,
                colour: stone_light,
            });
        }
    }

    commands.push(DrawCommand::Rect {
        x: 0,
        y: 152,
        width: FRAME_WIDTH as i32,
        height: 72,
        colour: [111, 70, 40],
    });
    commands.push(DrawCommand::Rect {
        x: 0,
        y: 144,
        width: FRAME_WIDTH as i32,
        height: 12,
        colour: [30, 168, 72],
    });
}

#[allow(clippy::too_many_arguments)]
fn push_eol_hologram_tile_clipped(
    commands: &mut Vec<DrawCommand>,
    hologram: &kid_render::EolHologramVisuals,
    tile_x: i32,
    tile_y: i32,
    tile_col: usize,
    tile_row: usize,
    clip_x: i32,
    clip_y: i32,
    clip_width: i32,
    clip_height: i32,
    tile_sheet: &Arc<TileSheet>,
    palette: &Arc<Palette>,
) {
    let Some(row) = hologram.rows.get(tile_row) else {
        return;
    };
    let Some(tile) = row.get(tile_col).copied() else {
        return;
    };
    let clipped_left = tile_x.max(clip_x);
    let clipped_top = tile_y.max(clip_y);
    let clipped_right = (tile_x + 8).min(clip_x + clip_width);
    let clipped_bottom = (tile_y + 8).min(clip_y + clip_height);
    if clipped_left >= clipped_right || clipped_top >= clipped_bottom {
        return;
    }

    let start_column = u8::try_from(clipped_left - tile_x).expect("clipped column fits");
    let width = u8::try_from(clipped_right - clipped_left).expect("clipped width fits");
    for screen_y in clipped_top..clipped_bottom {
        commands.push(DrawCommand::IndexedTileSlice {
            x: tile_x,
            y: tile_y,
            tile_sheet: Arc::clone(tile_sheet),
            tile,
            palette: Arc::clone(palette),
            row: u8::try_from(screen_y - tile_y).expect("clipped row fits"),
            start_column,
            width,
        });
    }
}

fn push_eol_hologram_tiles_for_screen_rect(
    commands: &mut Vec<DrawCommand>,
    hologram: &kid_render::EolHologramVisuals,
    x: i32,
    y: i32,
    width: i32,
    height: i32,
    tile_sheet: &Arc<TileSheet>,
    palette: &Arc<Palette>,
) {
    let start_col = x.div_euclid(8).max(0);
    let end_col = (x + width - 1)
        .div_euclid(8)
        .min((FRAME_WIDTH as i32 - 1) / 8);
    let start_row = y.div_euclid(8).max(0);
    let end_row = (y + height - 1)
        .div_euclid(8)
        .min((FRAME_HEIGHT as i32 - 1) / 8);
    if start_col > end_col || start_row > end_row {
        return;
    }

    for tile_row in start_row..=end_row {
        for tile_col in start_col..=end_col {
            // ASM loc_C21E/sub_C254 updates a clipped 16x16 Scroll A cell. The
            // Scroll B hologram stays on whole screen tiles underneath, but the
            // redraw must be clipped back to that foreground cell; otherwise
            // sub-tile-aligned cells leak a row/column into unrevealed Plane A.
            push_eol_hologram_tile_clipped(
                commands,
                hologram,
                tile_col * 8,
                tile_row * 8,
                usize::try_from(tile_col).expect("screen tile column fits"),
                usize::try_from(tile_row).expect("screen tile row fits"),
                x,
                y,
                width,
                height,
                tile_sheet,
                palette,
            );
        }
    }
}

fn push_eol_foreground_reveal_tiles(
    commands: &mut Vec<DrawCommand>,
    trace: &NativeTraceFrame,
    level_visuals: &NativeLevelVisuals,
    block_grid: Option<&BlockGrid>,
) {
    let (Some(hologram), Some(grid)) = (&level_visuals.eol_hologram, block_grid) else {
        return;
    };
    let reveal_elapsed = trace.level_complete_elapsed_frames.saturating_sub(
        kid_core::EOL_HOLOGRAM_VISIBLE_FRAME_LOC_C1D0_ORACLE
            + kid_core::EOL_FOREGROUND_REVEAL_VISIBLE_DELAY_FRAMES_LOC_C21E_ORACLE,
    );
    if reveal_elapsed == 0 {
        return;
    }

    let camera_x_px = (trace.camera_x_pos >> 16) as i32;
    let camera_y_px = (trace.camera_y_pos >> 16) as i32;
    let tile_sheet = Arc::new(hologram.tile_sheet.clone());
    let palette = Arc::new(eol_score_room_palette_loc_d734(trace, &hologram.palette));
    let writes = kid_core::flagpole_scoreboard_reveal_tiles_loc_C21E(
        grid,
        (trace.camera_x_pos >> 16) as u16,
        (trace.camera_y_pos >> 16) as u16,
        reveal_elapsed,
    );
    for write in &writes {
        let x = i32::from(write.col) * 16 - camera_x_px;
        let y = i32::from(write.row) * 16 - camera_y_px;
        commands.push(DrawCommand::Rect {
            x,
            y,
            width: 16,
            height: 16,
            colour: [0, 0, 0],
        });
        // ASM loc_C186 uploads the hologram as a Scroll B screen map. Redraw the
        // visible screen-space tiles under the loc_C21E/sub_C254 foreground write.
        // The foreground write is camera sub-tile aligned, but Scroll B is fixed
        // to screen tile coordinates after Background_NoScrollFlag is set at
        // loc_C0CC, so the replacement must stay on 8px screen boundaries.
        push_eol_hologram_tiles_for_screen_rect(
            commands,
            hologram,
            x,
            y,
            16,
            16,
            &tile_sheet,
            &palette,
        );
    }

    for write in &writes {
        let x = i32::from(write.col) * 16 - camera_x_px;
        let y = i32::from(write.row) * 16 - camera_y_px;
        // ASM loc_C21E/sub_C254 writes replacement Scroll A cells after the
        // loc_C186 Scroll B hologram upload. Native redraws Scroll B in a first
        // pass, then clears the local priority mask before drawing Scroll A
        // replacements so later EOL sprites can still appear above them.
        commands.push(DrawCommand::ClearPriorityRect {
            x,
            y,
            width: 16,
            height: 16,
        });
        for (index, word) in write.tile_words.iter().copied().enumerate() {
            commands.push(DrawCommand::IndexedTile {
                x: x + i32::try_from(index % 2).expect("tile column fits") * 8,
                y: y + i32::try_from(index / 2).expect("tile row fits") * 8,
                tile_sheet: Arc::clone(&tile_sheet),
                tile: TileRef {
                    palette_line: 0,
                    ..TileRef::from_attr_word(word)
                },
                palette: Arc::clone(&palette),
            });
        }
    }
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
    filter: LayerFilter,
) -> Scene {
    let eol_hologram_visible = trace.level_complete
        && trace.level_complete_elapsed_frames
            >= kid_core::EOL_HOLOGRAM_VISIBLE_FRAME_LOC_C1D0_ORACLE;
    let background = if eol_hologram_visible {
        [0, 0, 0]
    } else {
        background_colour(level_visuals)
    };
    let mut commands = Vec::new();

    if eol_hologram_visible && filter.background {
        push_eol_hologram_room(&mut commands, trace, level_visuals);
    } else if filter.background
        && (!trace.level_complete || trace.level_complete_elapsed_frames == 0)
    {
        // ASM CharacterCollision_TouchFlagpole loc_C070 starts the flagpole
        // hand-off, but the oracle frame at native `level_complete_elapsed=0`
        // still contains the previous Scroll B frame. Suppress ordinary Scroll
        // B during the later pre-hologram hibernate frames.
        push_background_tiles(&mut commands, trace, level_visuals);
    }
    if filter.terrain {
        push_level_layout_terrain(&mut commands, trace, level_visuals, block_grid);
    }
    if filter.block_overrides {
        push_block_grid_overrides(&mut commands, trace, level_visuals, block_grid);
    }
    if eol_hologram_visible && filter.terrain {
        push_eol_foreground_reveal_tiles(&mut commands, trace, level_visuals, block_grid);
    }

    if !eol_hologram_visible && filter.platforms {
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
    if !eol_hologram_visible && filter.enemies {
        for object in trace
            .active_objects
            .iter()
            .filter(|object| matches!(object.kind, kid_core::NativeObjectKind::Enemy { .. }))
        {
            if let NativeObjectKind::Enemy { object_type } = object.kind {
                if object_type == DRIPS_OBJECT_TYPE {
                    // ASM Drips.asm Enemy16_Drip_Init/loc_33488 keeps the
                    // parent/controller gfx object hidden via `$13`; native
                    // renders only the falling child and hostile surface
                    // crawler through typed projectile snapshots.
                    continue;
                }
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
    if !eol_hologram_visible && filter.player {
        // ASM `loc_BC34` damage-flash thread (kid.asm:15181-15215): during
        // the 66-frame `Just_received_damage` window, the ROM alternates
        // between the white palette (sub_80BC) for 2 frames and the normal
        // palette (sub_80D0) for 4 frames, repeating 11 times. Kid stays
        // visible the whole time.
        for object in trace
            .active_objects
            .iter()
            .filter(|object| matches!(object.kind, kid_core::NativeObjectKind::Player))
        {
            // ASM loc_B7FA/loc_B80C: the death tumble is driven by the
            // engine's `advance_death_windup_motion` (combat.rs:8801).
            // The renderer must NOT add its own offset on top — that was
            // user-reported playtest bug #148 (incorrect dying animation).
            // PLAYER_DAMAGE_INVULNERABLE_FRAMES = 66; elapsed counts up from
            // 0 to 65 across the cycle. Every six-frame slot is 2 frames of
            // white + 4 frames of normal, matching ASM `loc_BC56` (d2=1 → 2
            // hibernate frames white) and `loc_BC6E` (d2=3 → 4 hibernate
            // frames normal).
            let damage_flash_white_phase = trace.invulnerable_frames > 0
                && (66usize.saturating_sub(trace.invulnerable_frames) % 6) < 2;
            let priority_override =
                if trace.level_complete && trace.level_complete_elapsed_frames == 0 {
                    // ASM CharacterCollision_TouchFlagpole loc_C070 keeps the
                    // current Kid gfx object while the first hand-off frame still
                    // presents the old level/flag sprites. Native terrain priority
                    // can otherwise mask Kid on that preserved frame.
                    Some(true)
                } else {
                    None
                };
            push_player_sprite_with_priority_override(
                &mut commands,
                object,
                trace,
                level_visuals,
                damage_flash_white_phase,
                priority_override,
            );
        }
    }

    if !eol_hologram_visible && filter.effects {
        if (!trace.level_complete || trace.level_complete_elapsed_frames == 0)
            && let (Some(flag_x), Some(flag_y)) = (trace.flag_x, trace.flag_y)
        {
            // ASM Flagpole_Boss (kid.asm:15592) allocates exactly two
            // GfxObjects at the same anchor: `LnkTo_unk_E0FDE` (pole_bottom,
            // `sprite_frame_vram $696,$08,$10,$10,$10`) and `LnkTo_unk_E0FFE`
            // (waving flag, `sprite_frame_vram $69A,$06,$00,$28,$20`).
            // The ROM ships no third "pole shaft" sprite asset — the only
            // flag-related extracted art is `End_of_level_Flag_(6_frames).bin`
            // and `Flag_bottom.bin`. The previous 2×32 grey rect above the
            // anchor invented a tall pole extension that the ROM never draws.
            let sx = i32::from(flag_x) - (trace.camera_x_pos >> 16) as i32;
            let pole_base_y = i32::from(flag_y) - (trace.camera_y_pos >> 16) as i32;
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

        // ASM `sub_5E02` (kid.asm:5840-5965) emits the visible murder-wall
        // warning strip as 24 hardware sprites using `Murder_wall.bin`, which
        // Murderwall loc_140D2 uploads to VRAM tile `$2FB`.
        if let Some(wall_x_fp) = trace.murder_wall_x {
            // NativeTraceFrame does not yet expose `$FFFFFAC0/$FFFFFABF`;
            // this keeps the ASM sprite geometry and four ROM tile pages,
            // with the phase derived from frame time for native-player
            // presentation only.
            let animation_frame = ((trace.frame_index / 5) & 3) as u8;
            let render = kid_core::compile_murder_wall_warning_sprites_sub_5E02(
                kid_core::MurderWallWarningInputSub5E02 {
                    murder_wall_enabled: true,
                    murder_wall_x_px: (wall_x_fp >> 16) as i16,
                    murder_wall_reversed: level_visuals.level.murder_wall_flags & 1 != 0,
                    kid_hitbox_left_px: 0,
                    kid_hitbox_right_px: 0,
                    camera_x_px: (trace.camera_x_pos >> 16) as i16,
                    camera_y_px: (trace.camera_y_pos >> 16) as u16,
                    animation_timer: 2,
                    animation_frame,
                    starting_number_sprites: 0,
                },
            );
            if let Some(murder_wall_visuals) = &level_visuals.murder_wall {
                for sprite in render.sprites {
                    push_murder_wall_warning_sprite_sub_5e02(
                        &mut commands,
                        murder_wall_visuals,
                        sprite,
                    );
                }
            } else if let (Some(min_x), Some(max_x)) = (
                render
                    .sprites
                    .iter()
                    .map(|sprite| i32::from(sprite.x) - 0x80)
                    .min(),
                render
                    .sprites
                    .iter()
                    .map(|sprite| i32::from(sprite.x) - 0x80 + 24)
                    .max(),
            ) {
                push_murder_wall_warning_fallback_sub_5e02(&mut commands, min_x, max_x);
            }
        }

        // ASM `sub_14BD8 -> loc_14BE8` (kid.asm:28255-28278) emits lava
        // geyser sprite chunks from 8-byte template entries. NativeTraceFrame
        // exposes the live eruption column but not the full worker script
        // phase, so this renderer uses the ROM top/middle/bottom templates
        // (`unk_14624`, `unk_1465A`, `unk_14650`) over the active height.
        for &(gx, gy_top, height) in &trace.active_geysers {
            if let Some(lava_visuals) = &level_visuals.lava {
                push_lava_geyser_rom_sprites_loc_14be8(
                    &mut commands,
                    lava_visuals,
                    gx,
                    gy_top,
                    height,
                    (trace.camera_x_pos >> 16) as u16,
                    (trace.camera_y_pos >> 16) as u16,
                );
            } else {
                let sx = i32::from(gx) - (trace.camera_x_pos >> 16) as i32;
                let sy = i32::from(gy_top) - (trace.camera_y_pos >> 16) as i32;
                commands.push(DrawCommand::Rect {
                    x: sx - 4,
                    y: sy,
                    width: 8,
                    height: i32::from(height),
                    colour: [255, 120, 20],
                });
            }
        }

        // Projectiles: generic shots stay coloured dots; shooter-block
        // bullets keep the ROM's 1x1-sprite footprint visibly distinct; the
        // Fire Demon death trail renders ROM sprite frames via
        // `push_fire_demon_trail_sprite` (ASM Fire_Demon `loc_3A63C` cycling
        // `stru_3A622` -> `unk_C7776..unk_C779E`). Missing-asset states fall
        // back to a single placeholder rect in `push_fire_demon_trail`.
        let projectile_details: Vec<ActiveProjectileSnapshot> =
            if trace.active_projectile_details.is_empty() {
                trace
                    .active_projectiles
                    .iter()
                    .map(|&(x_pos, y_pos, friendly)| ActiveProjectileSnapshot {
                        x_pos,
                        y_pos,
                        x_velocity: 0,
                        y_velocity: 0,
                        friendly,
                        remaining_frames: 0,
                        animation_elapsed_frames: 0,
                        kind: ActiveProjectileKind::Generic,
                        boss_eye_sprite_mapping_index: 0,
                        dragon_flying_descent_fire_state: DragonFlyingDescentFireState::None,
                        dragon_flying_descent_fire_counter: 0,
                    })
                    .collect()
            } else {
                trace.active_projectile_details.clone()
            };
        for projectile in projectile_details {
            let sx = (projectile.x_pos >> 16) as i32 - (trace.camera_x_pos >> 16) as i32;
            let sy = (projectile.y_pos >> 16) as i32 - (trace.camera_y_pos >> 16) as i32;
            match projectile.kind {
                ActiveProjectileKind::Generic | ActiveProjectileKind::LionProjectile => {
                    commands.push(DrawCommand::Rect {
                        x: sx - 2,
                        y: sy - 2,
                        width: 4,
                        height: 4,
                        colour: if projectile.friendly {
                            FRIENDLY_PROJECTILE_COLOUR
                        } else {
                            ENEMY_PROJECTILE_COLOUR
                        },
                    });
                }
                ActiveProjectileKind::BossHeadArrow => {
                    // ASM loc_3A0D2 uses boss projectile sprite `stru_3A1CC`;
                    // until the exact C8510/C8518 child frame is compiled here,
                    // keep a thin horizontal footprint matching the live arrow
                    // path instead of a generic dot.
                    commands.push(DrawCommand::Rect {
                        x: sx - 5,
                        y: sy - 1,
                        width: 10,
                        height: 2,
                        colour: ENEMY_PROJECTILE_COLOUR,
                    });
                }
                ActiveProjectileKind::BossHeadBoomerang => {
                    // ASM loc_3A1EA uses boss projectile sprite `stru_3A308`;
                    // render a compact wide marker until the exact C8530 frame
                    // is compiled into native play.
                    commands.push(DrawCommand::Rect {
                        x: sx - 4,
                        y: sy - 2,
                        width: 8,
                        height: 4,
                        colour: ENEMY_PROJECTILE_COLOUR,
                    });
                }
                ActiveProjectileKind::BossSkewer => {
                    // ASM loc_384E8..loc_388D8 uses the Shishkaboss skewer
                    // child sprites; render a tall marker until those exact
                    // C85E8/C85F0/C85F8 frames are compiled into native play.
                    commands.push(DrawCommand::Rect {
                        x: sx - 2,
                        y: sy - 8,
                        width: 4,
                        height: 16,
                        colour: ENEMY_PROJECTILE_COLOUR,
                    });
                }
                ActiveProjectileKind::BossEyeAttack => {
                    sprites::push_boss_eye_attack_marker_sub_37abc(
                        &mut commands,
                        sx,
                        sy,
                        projectile.boss_eye_sprite_mapping_index,
                    );
                }
                ActiveProjectileKind::ShooterBlock => {
                    if !push_shooter_block_projectile_sprite_loc_ef62(
                        &mut commands,
                        level_visuals,
                        &projectile,
                        sx,
                        sy,
                    ) {
                        // ASM Process_ShooterObjects_MakeSprite (kid.asm:19475):
                        // shooter slots emit a hostile 1x1 sprite. If the ROM
                        // block-art tile is unavailable, retain an explicit
                        // 8x8 footprint instead of the tiny generic dot.
                        commands.push(DrawCommand::Rect {
                            x: sx - 4,
                            y: sy - 4,
                            width: 8,
                            height: 8,
                            colour: SHOOTER_BLOCK_PROJECTILE_COLOUR,
                        });
                    }
                }
                ActiveProjectileKind::TankBullet => {
                    let pushed_sprite = level_visuals
                        .enemies
                        .families
                        .get(&0x0B)
                        .or_else(|| level_visuals.enemies.families.get(&0x0A))
                        .and_then(|tank_visuals| {
                            let frame =
                                tank_bullet_child_frame(tank_visuals, projectile.remaining_frames)?;
                            let facing = if projectile.x_velocity < 0 {
                                Direction::Left
                            } else {
                                Direction::Right
                            };
                            let (draw_x, draw_y) = sprite_draw_position(&frame, sx, sy, facing);
                            commands.push(DrawCommand::Sprite {
                                x: draw_x,
                                y: draw_y,
                                tile_sheet: Arc::new(frame.tile_sheet.clone()),
                                palette: Arc::new(tank_visuals.palette.clone()),
                                tiles: build_sprite_tiles(&frame, facing, 0),
                            });
                            Some(())
                        })
                        .is_some();
                    if !pushed_sprite {
                        commands.push(DrawCommand::Rect {
                            x: sx - 4,
                            y: sy - 3,
                            width: 8,
                            height: 5,
                            colour: ENEMY_PROJECTILE_COLOUR,
                        });
                    }
                }
                ActiveProjectileKind::RobotShot | ActiveProjectileKind::RobotShotImpact => {
                    let pushed_sprite = level_visuals
                        .enemies
                        .families
                        .get(&ROBOT_OBJECT_TYPE)
                        .and_then(|robot_visuals| {
                            let impact = projectile.kind == ActiveProjectileKind::RobotShotImpact;
                            let frame = robot_shot_child_frame(
                                robot_visuals,
                                projectile.remaining_frames,
                                projectile.animation_elapsed_frames,
                                impact,
                            )?;
                            let facing = if !impact && projectile.x_velocity > 0 {
                                Direction::Right
                            } else {
                                Direction::Left
                            };
                            let (draw_x, draw_y) = sprite_draw_position(&frame, sx, sy, facing);
                            commands.push(DrawCommand::Sprite {
                                x: draw_x,
                                y: draw_y,
                                tile_sheet: Arc::new(frame.tile_sheet.clone()),
                                palette: Arc::new(robot_visuals.palette.clone()),
                                tiles: build_sprite_tiles(&frame, facing, 0),
                            });
                            Some(())
                        })
                        .is_some();
                    if !pushed_sprite {
                        commands.push(DrawCommand::Rect {
                            x: sx - 4,
                            y: sy - 3,
                            width: 8,
                            height: 6,
                            colour: ENEMY_PROJECTILE_COLOUR,
                        });
                    }
                }
                ActiveProjectileKind::FireDemonDeathTrail => {
                    if !push_fire_demon_trail_sprite(
                        &mut commands,
                        level_visuals,
                        sx,
                        sy,
                        projectile.remaining_frames,
                    ) {
                        push_fire_demon_trail(&mut commands, sx, sy, projectile.remaining_frames);
                    }
                }
                ActiveProjectileKind::FireballTrail => {
                    let pushed_sprite = level_visuals
                        .enemies
                        .families
                        .get(&0x19)
                        .and_then(|fireball_visuals| {
                            let frame = fireball_trail_child_frame(
                                fireball_visuals,
                                projectile.remaining_frames,
                            )?;
                            push_sprite_frame(
                                &mut commands,
                                &frame,
                                &fireball_visuals.palette,
                                sx,
                                sy,
                            );
                            Some(())
                        })
                        .is_some();
                    if !pushed_sprite {
                        commands.push(DrawCommand::Rect {
                            x: sx - 4,
                            y: sy - 5,
                            width: 8,
                            height: 8,
                            colour: FIRE_TRAIL_PLACEHOLDER_COLOUR,
                        });
                    }
                }
                ActiveProjectileKind::DragonWalkingBreath => {
                    let pushed_sprite = level_visuals
                        .enemies
                        .families
                        .get(&WALKING_DRAGON_OBJECT_TYPE)
                        .and_then(|dragon_visuals| {
                            let frame = dragon_walking_breath_child_frame(
                                dragon_visuals,
                                projectile.remaining_frames,
                            )?;
                            let facing =
                                dragon_walking_breath_child_facing_loc_3d02a(trace, &projectile);
                            let (draw_x, draw_y) = sprite_draw_position(&frame, sx, sy, facing);
                            commands.push(DrawCommand::Sprite {
                                x: draw_x,
                                y: draw_y,
                                tile_sheet: Arc::new(frame.tile_sheet.clone()),
                                palette: Arc::new(dragon_visuals.palette.clone()),
                                tiles: build_sprite_tiles(&frame, facing, 0),
                            });
                            Some(())
                        })
                        .is_some();
                    if !pushed_sprite {
                        commands.push(DrawCommand::Rect {
                            x: sx - 2,
                            y: sy - 3,
                            width: 4,
                            height: 5,
                            colour: [255, 90, 20],
                        });
                    }
                }
                ActiveProjectileKind::DragonFlyingDescentFire => {
                    let pushed_sprite = level_visuals
                        .enemies
                        .families
                        .get(&FLYING_DRAGON_OBJECT_TYPE)
                        .and_then(|dragon_visuals| {
                            let frame = dragon_flying_descent_fire_child_frame(
                                dragon_visuals,
                                &projectile,
                            )?;
                            push_sprite_frame(
                                &mut commands,
                                &frame,
                                &dragon_visuals.palette,
                                sx,
                                sy,
                            );
                            Some(())
                        })
                        .is_some();
                    if !pushed_sprite {
                        commands.push(DrawCommand::Rect {
                            x: sx - 3,
                            y: sy - 3,
                            width: 6,
                            height: 6,
                            colour: [255, 110, 30],
                        });
                    }
                }
                ActiveProjectileKind::DripsDrop
                | ActiveProjectileKind::DripsSurfaceCrawler
                | ActiveProjectileKind::DripsSurfaceCrawlerFinish => {
                    let pushed_sprite = level_visuals
                        .enemies
                        .families
                        .get(&DRIPS_OBJECT_TYPE)
                        .and_then(|drips_visuals| {
                            let frame = match projectile.kind {
                                ActiveProjectileKind::DripsDrop => {
                                    drips_drop_child_frame(drips_visuals)?
                                }
                                ActiveProjectileKind::DripsSurfaceCrawler => {
                                    drips_surface_crawler_frame(
                                        drips_visuals,
                                        projectile.remaining_frames,
                                        projectile.animation_elapsed_frames,
                                    )?
                                }
                                ActiveProjectileKind::DripsSurfaceCrawlerFinish => {
                                    drips_surface_crawler_finish_frame(
                                        drips_visuals,
                                        projectile.remaining_frames,
                                    )?
                                }
                                _ => return None,
                            };
                            push_sprite_frame(
                                &mut commands,
                                &frame,
                                &drips_visuals.palette,
                                sx,
                                sy,
                            );
                            Some(())
                        })
                        .is_some();
                    if !pushed_sprite {
                        commands.push(DrawCommand::Rect {
                            x: sx - 2,
                            y: sy - 2,
                            width: 4,
                            height: 5,
                            colour: [120, 200, 90],
                        });
                    }
                }
                ActiveProjectileKind::TarMonsterGlob
                | ActiveProjectileKind::TarMonsterBurstChild
                | ActiveProjectileKind::DripsSurfaceCrawlerBurst => {
                    let pushed_sprite = level_visuals
                        .enemies
                        .families
                        .get(&TAR_MONSTER_OBJECT_TYPE)
                        .and_then(|tar_visuals| {
                            let frame = match projectile.kind {
                                ActiveProjectileKind::TarMonsterGlob => {
                                    tar_monster_glob_child_frame(tar_visuals)?
                                }
                                ActiveProjectileKind::TarMonsterBurstChild => {
                                    tar_monster_burst_child_frame(tar_visuals)?
                                }
                                ActiveProjectileKind::DripsSurfaceCrawlerBurst => {
                                    tar_monster_burst_child_frame(tar_visuals)?
                                }
                                _ => return None,
                            };
                            // ASM Tar_Monster.asm loc_324D0/loc_324DC mirrors the
                            // glob when `x_direction != 0`, so face the sprite by
                            // travel direction. Burst fragments scatter both ways.
                            let facing = if projectile.x_velocity < 0 {
                                Direction::Left
                            } else {
                                Direction::Right
                            };
                            let (draw_x, draw_y) = sprite_draw_position(&frame, sx, sy, facing);
                            commands.push(DrawCommand::Sprite {
                                x: draw_x,
                                y: draw_y,
                                tile_sheet: Arc::new(frame.tile_sheet.clone()),
                                palette: Arc::new(tar_visuals.palette.clone()),
                                tiles: build_sprite_tiles(&frame, facing, 0),
                            });
                            Some(())
                        })
                        .is_some();
                    if !pushed_sprite {
                        // Asset-missing fallback: keep a dark tar footprint rather
                        // than reverting to the generic red enemy-projectile dot.
                        commands.push(DrawCommand::Rect {
                            x: sx - 3,
                            y: sy - 3,
                            width: 6,
                            height: 6,
                            colour: [40, 50, 80],
                        });
                    }
                }
            }
        }

        for sprite in &trace.diamond_power_sprites {
            let _ = push_diamond_power_sprite_loc_3f880(&mut commands, level_visuals, *sprite);
        }

        // ASM sub_1007A bounce coroutine: the block itself moves with the
        // engine-supplied y_offset (replaces the previous grey-rect
        // overlay). The level-pass `push_block_grid_overrides` is
        // responsible for drawing the actual block art at the offset
        // position; this loop is intentionally empty so the original cell
        // is not double-drawn at the un-offset position.
        for &(_cx, _cy, _remaining, _y_offset) in &trace.block_bounces {
            // Block-pass handles the offset draw; nothing to do here.
        }

        // ASM `sub_292E` (kid.asm:4881) emits five direction-keyed debris
        // workers, and `sub_2A4C` (kid.asm:5025) draws each with tile `$8228`.
        // Keep the old centre marker only as an asset-missing fallback.
        for &(x_px, y_px, _remaining, direction) in &trace.block_destroy_effects {
            let camera_x_px = (trace.camera_x_pos >> 16) as i32;
            let camera_y_px = (trace.camera_y_pos >> 16) as i32;
            if push_block_destroy_debris_sub_292e(
                &mut commands,
                level_visuals,
                x_px,
                y_px,
                direction,
                camera_x_px,
                camera_y_px,
            ) {
                continue;
            }

            let cx = i32::from(x_px) + 8 - camera_x_px;
            let cy = i32::from(y_px) + 8 - camera_y_px;
            commands.push(DrawCommand::Rect {
                x: cx - 3,
                y: cy - 3,
                width: 6,
                height: 6,
                colour: [180, 150, 110],
            });
        }

        if let Some(tile) = resolve_ingame_vram_tile(
            level_visuals,
            TileRef::from_attr_word(BLOCK_DEBRIS_TILE_ATTR_SUB_2A4C),
        ) {
            let camera_x_px = (trace.camera_x_pos >> 16) as i32;
            let camera_y_px = (trace.camera_y_pos >> 16) as i32;
            for &(x_px, y_px) in &trace.block_debris_pieces {
                // ASM sub_2A4C (kid.asm:5053) emits persistent falling debris
                // sprite entries with tile `$8228` after sub_292E seeds the
                // scratch slot.
                push_resolved_tile(
                    &mut commands,
                    i32::from(x_px) - camera_x_px,
                    i32::from(y_px) - camera_y_px,
                    tile.clone(),
                );
            }
        }

        for &(x_px, y_px, remaining) in &trace.prize_block_spawn_flashes {
            let camera_x_px = (trace.camera_x_pos >> 16) as i32;
            let camera_y_px = (trace.camera_y_pos >> 16) as i32;
            let screen_x = i32::from(x_px) - camera_x_px;
            let screen_y = i32::from(y_px) - camera_y_px;
            push_prize_block_spawn_flash_loc_102b8(
                &mut commands,
                level_visuals,
                screen_x,
                screen_y,
                remaining,
            );
        }

        for &(x_px, y_px, remaining) in &trace.kid_projectile_hit_sparks {
            let camera_x_px = (trace.camera_x_pos >> 16) as i32;
            let camera_y_px = (trace.camera_y_pos >> 16) as i32;
            let screen_x = i32::from(x_px) - camera_x_px;
            let screen_y = i32::from(y_px) - camera_y_px;
            push_kid_projectile_hit_spark_loc_1f08(
                &mut commands,
                level_visuals,
                screen_x,
                screen_y,
                remaining,
            );
        }

        // Floating prize items: small coloured shape rising upward
        for &(x_px, y_px, remaining, content_id) in &trace.floating_items {
            let fx = i32::from(x_px) - (trace.camera_x_pos >> 16) as i32;
            let fy = i32::from(y_px) - (trace.camera_y_pos >> 16) as i32;
            let content = kid_core::terrain::PrizeContent::from_id(content_id);
            if render_floating_prize_worker_effect(
                &mut commands,
                level_visuals,
                content,
                fx,
                fy,
                remaining,
            ) {
                continue;
            }
            let graphics_spec = floating_item_graphics_spec(content_id);
            if let Some(prize_visuals) = floating_prize_visuals(level_visuals, content_id) {
                if !floating_prize_sprite_visible_loc_103b2(content, remaining) {
                    continue;
                }
                let frame_index = floating_prize_frame_index_loc_10396(
                    content,
                    remaining,
                    trace.frame_index,
                    prize_visuals.frame_duration.max(1),
                    prize_visuals.frames.len().max(1),
                );
                if let Some(frame) = prize_visuals.frames.get(frame_index) {
                    let facing = Direction::Right;
                    let (draw_x, draw_y) = sprite_draw_position(frame, fx, fy, facing);
                    commands.push(DrawCommand::Sprite {
                        x: draw_x,
                        y: draw_y,
                        tile_sheet: Arc::new(frame.tile_sheet.clone()),
                        palette: Arc::new(prize_visuals.palette.clone()),
                        tiles: build_sprite_tiles(frame, facing, 0),
                    });
                    continue;
                }
            }

            let alpha = u32::from(remaining).saturating_mul(8).min(255);
            let base_colour = floating_item_colour(content, graphics_spec);
            let colour = [
                (u32::from(base_colour[0]) * alpha / 255) as u8,
                (u32::from(base_colour[1]) * alpha / 255) as u8,
                (u32::from(base_colour[2]) * alpha / 255) as u8,
            ];
            // Degraded-asset fallback only: in production, Diamond / Ankh /
            // Clock / Continue / helmet floating prizes resolve through
            // `floating_prize_visuals` to real ROM sprite frames (ASM
            // PrizeDiamond_Init `stru_10D6E`, sub_10CB0 helmet selectors,
            // `stru_10C08/10C18/10C28` icon tiles), and Points10000 /
            // TenDiamonds render through `render_floating_prize_worker_effect`
            // (ASM Prize10000Points_Init / Prize10Diamonds_Init). The
            // previous Diamond / Ankh / Clock branches drew hand-invented
            // pixel-art "diamond", "cross", and "clock face" approximations
            // with no ROM citation — those shapes never appeared in the
            // original game. Collapse the fallback to one consistent
            // filled-square indicator so missing-asset states are visible
            // without inventing per-prize artwork.
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

        if trace.heady_metal_death_splits.is_empty() {
            // Enemy death effect: only HeadyMetal spawns one. ASM
            // Enemy20_HeadyMetal_Init seeds the $01C2-frame wobble worker
            // (sub_37708/sub_3787C/sub_378A2); `Jump_On_Enemy` (kid.asm:14607)
            // plays sfx_Jump_on_enemy and deletes the GfxObject without
            // emitting any per-stomp death sprite.
            for &(ex, ey, remaining) in &trace.enemy_death_effects {
                push_heady_metal_death_wobble(
                    &mut commands,
                    trace,
                    level_visuals,
                    ex,
                    ey,
                    remaining,
                );
            }
        } else {
            push_heady_metal_death_split_pieces(&mut commands, trace, level_visuals);
        }
    }

    if !eol_hologram_visible && filter.hud {
        // ASM Mode_Level calls Make_SpriteAttr_HUD before sub_5E02 but the
        // HUD occupies the fixed top overlay in native-player. Clear native's
        // local high-priority mask so later murder-wall/weather sprites cannot
        // occlude the timer/lives/diamond HUD row.
        commands.push(DrawCommand::ClearPriorityRect {
            x: 0,
            y: 0,
            width: FRAME_WIDTH as i32,
            height: 48,
        });
        push_game_hud(&mut commands, trace, level_visuals);
    }

    // Level Complete overlay. ASM `loc_D55E` onwards (kid.asm:16670-16800)
    // emits sprite-art labels — `LnkTo_Spr_EOL_Score`,
    // `LnkTo_Spr_EOL_Time`, `LnkTo_Spr_EOL_Bonus`, `LnkTo_Spr_EOL_NoHit`,
    // `LnkTo_Spr_EOL_NoPrize`, `LnkTo_Spr_EOL_Path`, `LnkTo_Spr_EOL_Speed`
    // — plus seven digit workers from `Load_Object_EOL_Number`. Native uses
    // the trace's EOL worker elapsed fields to replay the ROM fly-in timing.
    if trace.level_complete && filter.eol_score {
        if let Some(eol_score) = &level_visuals.eol_score {
            push_eol_score_overlay(&mut commands, trace, eol_score);
            return Scene {
                background,
                commands,
                pause_palette_darken: trace.paused,
            };
        }
        let text_y = (FRAME_HEIGHT as i32) / 2 - 40;
        let mut line_y = text_y + 22;
        let time_bonus = kid_core::time_bonus_loc_d56e(trace.level_timer_seconds);
        let bonus_line = format!("TIME BONUS: {time_bonus}");
        let bonus_x = (FRAME_WIDTH as i32 - bonus_line.len() as i32 * 8) / 2;
        commands.push(text(bonus_x, line_y, &bonus_line, [200, 255, 200], 1));
        line_y += 12;
        // ASM loc_D5BC (kid.asm:16715): NO-HIT bonus awarded when
        // NoHit_Bonus_Flag clear, i.e. no damage taken during the level.
        if !trace.level_damage_taken {
            let nohit = "NO-HIT BONUS: 5000";
            let nohit_x = (FRAME_WIDTH as i32 - nohit.len() as i32 * 8) / 2;
            commands.push(text(nohit_x, line_y, nohit, [255, 255, 100], 1));
            line_y += 12;
        }
        // ASM loc_D5D2 (kid.asm:16732): NO-PRIZE bonus awarded when
        // NoPrize_Bonus_Flag clear, i.e. no prize-block was struck during
        // the level. The previous overlay omitted this 5000-point bonus
        // even though the engine awards it through `add_score(5000)`, so
        // displayed totals lagged the score by 5000 on no-prize runs.
        if !trace.level_prize_collected {
            let noprize = "NO-PRIZE BONUS: 5000";
            let noprize_x = (FRAME_WIDTH as i32 - noprize.len() as i32 * 8) / 2;
            commands.push(text(noprize_x, line_y, noprize, [255, 200, 255], 1));
            line_y += 12;
        }
        // ASM loc_D60E (kid.asm:16750): PATH BONUS = `PathBonus_Values[level]
        // × 1000`. Only awarded on non-Elsewhere levels with a non-zero
        // table entry; the renderer reflects the engine's awarded amount
        // directly rather than recomputing the per-level table.
        if trace.level_path_bonus_amount > 0 {
            let path_line = format!("PATH BONUS: {}", trace.level_path_bonus_amount);
            let path_x = (FRAME_WIDTH as i32 - path_line.len() as i32 * 8) / 2;
            commands.push(text(path_x, line_y, &path_line, [200, 200, 255], 1));
            line_y += 12;
        }
        // ASM loc_D660 (kid.asm:16775): SPEED BONUS = 10,000 points,
        // awarded only when `Level_completion_time <= SpeedBonus_Values[level]`
        // (per-level threshold table, accounting for clocks collected).
        // The previous overlay used a hand-tuned `elapsed < 60` gate that
        // did not match the engine's per-level condition; surface the
        // engine-driven `level_speed_bonus_awarded` flag instead.
        if trace.level_speed_bonus_awarded {
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
    if trace.game_over && filter.eol_score {
        if let Some(eol_score) = &level_visuals.eol_score {
            push_game_over_continue_overlay(&mut commands, trace, eol_score);
        } else if trace.number_continues == 0 && filter.hud {
            // Degraded-asset fallback only. The normal asset path above uses
            // ASM loc_D25E / Continue_Screen EOL sprite workers.
            let go_text = "GAME OVER";
            let text_w = go_text.len() as i32 * 8 * 2;
            let text_x = (FRAME_WIDTH as i32 - text_w) / 2;
            let text_y = (FRAME_HEIGHT as i32) / 2 - 8;
            commands.push(text(text_x, text_y, go_text, [255, 80, 80], 2));
        }
    }

    // Pause overlay
    if trace.paused && filter.hud {
        push_pause_menu_text_loc_69d2(&mut commands, trace);
    }

    // Level complete overlay
    if trace.level_complete && filter.hud {
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

    if debug_overlay_visible && filter.hud {
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

    push_teleport_warp_tint_loc_bcf6(&mut commands, trace);

    Scene {
        background,
        commands,
        pause_palette_darken: trace.paused,
    }
}

fn push_teleport_warp_tint_loc_bcf6(commands: &mut Vec<DrawCommand>, trace: &NativeTraceFrame) {
    if trace.teleport_warp_colour == 0 {
        return;
    }
    let colour = MdColour::from_word(trace.teleport_warp_colour);
    commands.push(DrawCommand::Tint {
        colour: [colour.red, colour.green, colour.blue],
        alpha: 96,
    });
}

fn floating_prize_sprite_visible_loc_103b2(
    content: kid_core::terrain::PrizeContent,
    remaining_frames: u16,
) -> bool {
    match content {
        kid_core::terrain::PrizeContent::Diamond if remaining_frames <= 0x00C8 => {
            // ASM PrizeDiamond_Init loc_103B2 swaps to `stru_10D80` exactly
            // when d3 reaches $00C8. That script alternates `unk_E0E66` with
            // null frame pointers every three frames, so Make_SpriteFromGfxObject
            // returns without emitting a VDP sprite on the odd slots.
            let blink_elapsed = 0x00C8u16.saturating_sub(remaining_frames);
            (blink_elapsed / 3).is_multiple_of(2)
        }
        _ => true,
    }
}

fn floating_prize_frame_index_loc_10396(
    content: kid_core::terrain::PrizeContent,
    remaining_frames: u16,
    trace_frame_index: usize,
    frame_duration: usize,
    frame_count: usize,
) -> usize {
    if frame_count == 0 {
        return 0;
    }

    match content {
        kid_core::terrain::PrizeContent::Diamond if remaining_frames > 0x00C8 => {
            // ASM PrizeDiamond_Init loc_10396 decrements d3 from $01F4 after
            // j_Init_Animation starts `stru_10D6E`; global Time_Frames does
            // not choose the local diamond frame.
            let elapsed = 0x01F4u16.saturating_sub(remaining_frames);
            usize::from(elapsed) / frame_duration % frame_count
        }
        kid_core::terrain::PrizeContent::Diamond => {
            // ASM loc_103B2 switches to `stru_10D80` at d3=$00C8:
            // visible frame, null frame, next visible frame, null frame...
            let blink_elapsed = 0x00C8u16.saturating_sub(remaining_frames);
            usize::from(blink_elapsed) / (frame_duration * 2) % frame_count
        }
        _ => trace_frame_index / frame_duration % frame_count,
    }
}

fn render_floating_prize_worker_effect(
    commands: &mut Vec<DrawCommand>,
    level_visuals: &NativeLevelVisuals,
    content: kid_core::terrain::PrizeContent,
    screen_x: i32,
    screen_y: i32,
    remaining_frames: u16,
) -> bool {
    let remaining_low_byte = remaining_frames as u8;
    if remaining_low_byte > PRIZE_GENERIC_LIFETIME_LOW_BYTE {
        return false;
    }
    let effect_age = PRIZE_GENERIC_LIFETIME_LOW_BYTE.wrapping_sub(remaining_low_byte);
    match content {
        kid_core::terrain::PrizeContent::TenDiamonds => render_ten_diamonds_worker_effect(
            commands,
            level_visuals,
            screen_x,
            screen_y,
            effect_age,
        ),
        kid_core::terrain::PrizeContent::Points10000 => render_10000_points_worker_effect(
            commands,
            level_visuals,
            screen_x,
            screen_y,
            effect_age,
        ),
        _ => false,
    }
}

fn render_ten_diamonds_worker_effect(
    commands: &mut Vec<DrawCommand>,
    level_visuals: &NativeLevelVisuals,
    screen_x: i32,
    screen_y: i32,
    effect_age: u8,
) -> bool {
    // ASM Prize10Diamonds_Init loc_108EE uses byte_108AC to spawn ten
    // workers on frames 0..3 and 24..29. Each worker uses unk_E1046:
    // sprite_frame_vram $6ED,$02,$FE,$08,$08 over the HUD diamond DMA page.
    const SPAWNS: &[(u8, i32, i32)] = &[
        (0, -20, -36),
        (1, -20, -27),
        (2, -20, -18),
        (3, -20, -9),
        (24, 6, -36),
        (25, -4, -28),
        (26, 16, -28),
        (27, -4, -16),
        (28, 16, -16),
        (29, 6, -8),
    ];

    let frame = kid_render::SpriteFrame {
        origin_x: 0x02,
        origin_y: -2,
        width_px: 0x08,
        height_px: 0x08,
        tile_base_index: 0x06ED - level_visuals.hud.diamond_base_tile_index,
        tile_sheet: level_visuals.hud.diamond_tiles.clone(),
    };
    let mut drew_worker = false;
    for &(spawn_frame, x_velocity_word, y_velocity_word) in SPAWNS {
        if effect_age < spawn_frame {
            continue;
        }
        let worker_age = i32::from(effect_age - spawn_frame);
        let worker_x = screen_x + ((x_velocity_word * worker_age) >> 3);
        let worker_y = screen_y + ((y_velocity_word * worker_age) >> 3);
        push_worker_sprite(
            commands,
            &frame,
            &level_visuals.palettes.foreground,
            worker_x,
            worker_y,
        );
        drew_worker = true;
    }
    drew_worker
}

fn render_10000_points_worker_effect(
    commands: &mut Vec<DrawCommand>,
    level_visuals: &NativeLevelVisuals,
    screen_x: i32,
    screen_y: i32,
    effect_age: u8,
) -> bool {
    // ASM Prize10000Points_Init loc_10B00 uses unk_10A7A to spawn five workers
    // on frames 0,5,10,15,20. Frame 0 uses unk_E1056 (tile $6BB); later
    // workers use unk_E104E (tile $6BA), both 8x8 over HUD number tiles.
    const SPAWNS: &[u8] = &[0, 5, 10, 15, 20];
    let mut drew_worker = false;
    for &spawn_frame in SPAWNS {
        if effect_age < spawn_frame {
            continue;
        }
        let worker_age = i32::from(effect_age - spawn_frame);
        let tile_base_index = if spawn_frame == 0 { 1 } else { 0 };
        let frame = kid_render::SpriteFrame {
            origin_x: 0x04,
            origin_y: -4,
            width_px: 0x08,
            height_px: 0x08,
            tile_base_index,
            tile_sheet: level_visuals.hud.digits.clone(),
        };
        let worker_x = screen_x - 6 * worker_age.saturating_sub(0x58);
        push_worker_sprite(
            commands,
            &frame,
            &level_visuals.palettes.foreground,
            worker_x,
            screen_y,
        );
        drew_worker = true;
    }
    drew_worker
}

fn push_worker_sprite(
    commands: &mut Vec<DrawCommand>,
    frame: &kid_render::SpriteFrame,
    palette: &Palette,
    screen_x: i32,
    screen_y: i32,
) {
    let (draw_x, draw_y) = sprite_draw_position(frame, screen_x, screen_y, Direction::Right);
    commands.push(DrawCommand::Sprite {
        x: draw_x,
        y: draw_y,
        tile_sheet: Arc::new(frame.tile_sheet.clone()),
        palette: Arc::new(palette.clone()),
        tiles: build_sprite_tiles(frame, Direction::Right, 0),
    });
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

fn push_heady_metal_death_split_pieces(
    commands: &mut Vec<DrawCommand>,
    trace: &NativeTraceFrame,
    level_visuals: &NativeLevelVisuals,
) {
    for split in &trace.heady_metal_death_splits {
        for piece in &split.pieces {
            // ASM sub_378C8 loc_378D6 mutates the five cached HeadyMetal
            // workers at $FB72/$FB76/$FB7A/$FBAE/$FBB2; loc_37922 later
            // deletes the same slots. Core trace positions are already the
            // post-impulse worker positions, so render them directly.
            let screen_x = (piece.x_pos >> 16) as i32 - (trace.camera_x_pos >> 16) as i32;
            let screen_y = (piece.y_pos >> 16) as i32 - (trace.camera_y_pos >> 16) as i32;
            push_effect_enemy_sprite(
                commands,
                level_visuals,
                0x21,
                screen_x,
                screen_y,
                Direction::Right,
                false,
            );
        }
    }
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

pub(super) fn floating_item_graphics_spec(
    content_id: u8,
) -> Option<kid_core::FloatingPrizeGraphicsSpec> {
    kid_core::floating_prize_graphics_spec(kid_core::terrain::PrizeContent::from_id(content_id))
}

fn floating_prize_visuals(
    level_visuals: &NativeLevelVisuals,
    content_id: u8,
) -> Option<&kid_render::AnimatedSpriteVisual> {
    match kid_core::terrain::PrizeContent::from_id(content_id) {
        // ASM PrizeDiamond_Init: `stru_10D6E` selects `unk_E0E66`, a VRAM
        // sprite frame over the HUD diamond tiles.
        kid_core::terrain::PrizeContent::Diamond => Some(&level_visuals.floating_prizes.diamond),
        // ASM sub_10CB0 followed by PrizeHelmet*_Init selects the mask frame
        // and helmet palette for content ids 2..A.
        kid_core::terrain::PrizeContent::Skycutter
        | kid_core::terrain::PrizeContent::Cyclone
        | kid_core::terrain::PrizeContent::RedStealth
        | kid_core::terrain::PrizeContent::Eyeclops
        | kid_core::terrain::PrizeContent::Juggernaut
        | kid_core::terrain::PrizeContent::IronKnight
        | kid_core::terrain::PrizeContent::Berzerker
        | kid_core::terrain::PrizeContent::Maniaxe
        | kid_core::terrain::PrizeContent::Micromax => {
            level_visuals.floating_prizes.helmets.get(&content_id)
        }
        // ASM PrizeAnkh_Init/PrizeClock_Init/PrizeContinue_Init select
        // `stru_10C08`, `stru_10C18`, and `stru_10C28` over ROM icon tiles.
        kid_core::terrain::PrizeContent::Ankh
        | kid_core::terrain::PrizeContent::Clock
        | kid_core::terrain::PrizeContent::Continue => {
            level_visuals.floating_prizes.icons.get(&content_id)
        }
        _ => None,
    }
}

/// Returns a distinct RGB colour for each prize content type.
fn floating_item_colour(
    content: kid_core::terrain::PrizeContent,
    graphics_spec: Option<kid_core::FloatingPrizeGraphicsSpec>,
) -> [u8; 3] {
    match (content, graphics_spec) {
        // ASM PrizeDiamond_Init: palette line 0, `stru_10D6E`.
        (kid_core::terrain::PrizeContent::Diamond, Some(_)) => [100, 200, 255],
        // ASM sub_10CB0: all helmet prizes use palette line 3; use the
        // object metadata to keep placeholder helmets visually distinct until
        // the real sprite art is wired.
        (_, Some(spec)) if spec.palette_line == 3 => helmet_placeholder_colour(spec.object_meta),
        (kid_core::terrain::PrizeContent::Points10000, _) => [255, 255, 100],
        (kid_core::terrain::PrizeContent::Ankh, _) => [255, 255, 200],
        (kid_core::terrain::PrizeContent::Clock, _) => [200, 255, 200],
        (kid_core::terrain::PrizeContent::Continue, _) => [255, 255, 255],
        (kid_core::terrain::PrizeContent::TenDiamonds, _) => [100, 200, 255],
        _ => [255, 255, 100],
    }
}

fn helmet_placeholder_colour(_object_meta: u16) -> [u8; 3] {
    // ASM sub_10CB0 places every helmet prize on palette line 3 with the
    // helmet's own `Pal_*` block (e.g. `Pal_DD7A1` for Berzerker). The
    // previous nine hand-picked RGB triples — "Skycutter pale blue",
    // "Cyclone mint", "RedStealth pink", etc. — were guessed palette
    // colours with no ROM citation. Until each helmet's ROM palette
    // flows through `level_visuals.floating_prizes.helmets`, render a
    // single consistent placeholder so missing-asset states do not
    // claim per-helmet hues that the ROM does not specify.
    [220, 220, 255]
}
