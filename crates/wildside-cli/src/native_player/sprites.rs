use super::*;
use crate::scenarios::DynamicCheckpoint;
use wildside_render::{Palette, SpriteFrame};

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

const BOSS_HEAD_DEATH_ANIMATION_FRAMES_LOC_39EBE: u16 = 120 + (10 * 17);

fn boss_head_death_frame_loc_39ebe(remaining: u16) -> usize {
    // ASM loc_39EBE plays eight sweep scripts, then loc_39F7A loops four
    // one-tick spin scripts ten times before j_Delete_CurrentObject.
    const INTRO: &[(u16, usize)] = &[
        // stru_39FEE
        (6, 4),
        (6, 3),
        (6, 2),
        (6, 1),
        (6, 0),
        // stru_3A004
        (6, 1),
        (6, 2),
        (6, 3),
        (6, 4),
        // stru_3A016
        (4, 3),
        (4, 2),
        (4, 1),
        (4, 0),
        // stru_3A028
        (4, 1),
        (4, 2),
        (4, 3),
        (4, 4),
        // stru_3A03A
        (2, 4),
        (2, 3),
        (2, 2),
        (2, 1),
        (2, 0),
        // stru_3A050
        (2, 1),
        (2, 2),
        (2, 3),
        (2, 4),
        // stru_3A062
        (2, 3),
        (2, 2),
        (2, 1),
        (2, 0),
        // stru_3A074
        (2, 1),
        (2, 2),
        (2, 3),
        (2, 4),
    ];
    const SPIN: &[(u16, usize)] = &[
        // stru_3A086
        (1, 4),
        (1, 3),
        (1, 2),
        (1, 1),
        (1, 0),
        // stru_3A09C
        (1, 1),
        (1, 2),
        (1, 3),
        (1, 4),
        // stru_3A0AE
        (1, 3),
        (1, 2),
        (1, 1),
        (1, 0),
        // stru_3A0C0
        (1, 1),
        (1, 2),
        (1, 3),
        (1, 4),
    ];

    let mut elapsed =
        usize::from(BOSS_HEAD_DEATH_ANIMATION_FRAMES_LOC_39EBE.saturating_sub(remaining));
    for &(duration, frame) in INTRO {
        let duration = usize::from(duration);
        if elapsed < duration {
            return frame;
        }
        elapsed -= duration;
    }

    let spin_total = SPIN
        .iter()
        .map(|(duration, _)| usize::from(*duration))
        .sum::<usize>();
    let mut spin_elapsed = elapsed % spin_total;
    for &(duration, frame) in SPIN {
        let duration = usize::from(duration);
        if spin_elapsed < duration {
            return frame;
        }
        spin_elapsed -= duration;
    }
    4
}

fn boss_head_death_facing_loc_39ebe(facing: Direction, remaining: u16) -> Direction {
    let elapsed = usize::from(BOSS_HEAD_DEATH_ANIMATION_FRAMES_LOC_39EBE.saturating_sub(remaining));
    // ASM loc_39EE2/loc_39F7A toggles x_direction with `eori.b #$FF,x_direction(a3)`
    // between the death animation scripts; no extra toggle happens at the loop tail.
    let toggled = match elapsed {
        0..30 => false,
        30..70 => true,
        70..96 => false,
        96..112 => true,
        112..120 => false,
        _ => {
            let spin_elapsed = (elapsed - 120) % 17;
            (5..13).contains(&spin_elapsed)
        }
    };
    if toggled {
        opposite_direction(facing)
    } else {
        facing
    }
}

fn opposite_direction(facing: Direction) -> Direction {
    match facing {
        Direction::Right => Direction::Left,
        Direction::Left => Direction::Right,
    }
}

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

fn build_player_sprite_tiles_with_flip(
    frame: &wildside_render::SpriteFrame,
    flip_x: bool,
    priority_override: Option<bool>,
) -> Vec<SpriteTile> {
    let mut tiles = build_sprite_tiles_with_flip(frame, flip_x, 0);
    if let Some(priority) = priority_override {
        for tile in &mut tiles {
            tile.tile.priority = priority;
        }
    }
    tiles
}

pub(super) fn align_sprite_dimension(px: u16) -> u16 {
    px.div_ceil(8) * 8
}

pub(super) fn sprite_draw_position(
    frame: &wildside_render::SpriteFrame,
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
    frame: &wildside_render::SpriteFrame,
    facing: Direction,
    palette_line: u8,
) -> Vec<SpriteTile> {
    build_sprite_tiles_with_flip(frame, matches!(facing, Direction::Left), palette_line)
}

fn build_sprite_tiles_with_flip(
    frame: &wildside_render::SpriteFrame,
    flip_x: bool,
    palette_line: u8,
) -> Vec<SpriteTile> {
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
    player_visuals: &'a wildside_render::PlayerFormVisuals,
) -> &'a wildside_render::SpriteFrame {
    let on_ground = trace.kid_on_ground.unwrap_or(true);
    let x_velocity = trace.kid_x_velocity.unwrap_or(0);
    let y_velocity = trace.kid_y_velocity.unwrap_or(0);

    if trace.ledge_snap_cooldown > 0 && !player_visuals.pull_up.is_empty() {
        // ASM loc_ABF0 (kid.asm:13449) starts `stru_8BB4`, then
        // `Hibernate_UntilAnimFinished` holds seven rollover frames for three
        // VBlanks each before returning to MoveID_Standingstill.
        let total = player_visuals.pull_up.len() * 3;
        let elapsed = total.saturating_sub(usize::from(trace.ledge_snap_cooldown));
        let frame_index = (elapsed / 3).min(player_visuals.pull_up.len() - 1);
        return &player_visuals.pull_up[frame_index];
    }

    // ASM sub_B270 loc_B350/loc_B380 (kid.asm:14220-14248): the airborne
    // attack-down counter (`red_stealth_airborne_slash_counter`, 0→$7F)
    // selects one of four downward-slash phases via `counter >> 5`. Native
    // wired the counter through the runtime step in commit `bcfb47f6`; this
    // branch now consumes it so the visual sword actually points down during
    // a fast-fall hold instead of falling through to the generic jump-fall
    // pose. Takes precedence over the ground swing timer because the airborne
    // path is what the user reported broken (#153).
    if !on_ground
        && trace.red_stealth_airborne_slash_counter > 0
        && !player_visuals.slash_air.is_empty()
    {
        let phase = usize::from(trace.red_stealth_airborne_slash_counter >> 5);
        let frame_index = phase.min(player_visuals.slash_air.len() - 1);
        return &player_visuals.slash_air[frame_index];
    }

    if trace.red_stealth_sword_timer > 0 {
        let slash_frames = if on_ground {
            &player_visuals.slash_ground
        } else {
            &player_visuals.slash_air
        };
        if !slash_frames.is_empty() {
            // ASM loc_76B0 initialises `stru_8B3C` for the grounded sword and
            // sub_B270 indexes `unk_B40DE..unk_B4870` for the air slash.
            let elapsed = usize::from(12u8.saturating_sub(trace.red_stealth_sword_timer));
            let frame_index = (elapsed / 3).min(slash_frames.len() - 1);
            return &slash_frames[frame_index];
        }
    }

    let iron_knight_attach_hibernate = trace.current_helmet_id
        == PlayerForm::IronKnight.helmet_id()
        && trace.iron_knight_climbing
        && trace.iron_knight_climb_pose_accum == 0
        && y_velocity == -0x4000;
    if trace.current_helmet_id == PlayerForm::IronKnight.helmet_id()
        && trace.iron_knight_climbing
        && !iron_knight_attach_hibernate
        && !player_visuals.climb.is_empty()
    {
        // ASM sub_A3BC (kid.asm:12710-12715) stores d3, buckets `d3 >> 2`
        // through `unk_A3F0`, then writes `off_A420` into addroffset_sprite.
        // Preserve that sprite write directly; a stationary cling is not idle.
        //
        // ASM loc_A276 seeds d3=0/y_vel=$FFFFC000 and hibernates through
        // loc_A28A before sub_A3BC has written an off_A420 climb sprite, so
        // the exact attach snapshot must fall through to the ordinary pose.
        let frame_index = match trace.iron_knight_climb_pose_accum >> 2 {
            0..=15 => 0,
            16..=31 => 1,
            _ => 2,
        }
        .min(player_visuals.climb.len() - 1);
        return &player_visuals.climb[frame_index];
    }

    if on_ground {
        if trace.current_helmet_id == PlayerForm::Berzerker.helmet_id()
            && (trace.berzerker_charging || trace.berzerker_charge_pose)
            && !player_visuals.charge.is_empty()
        {
            // ASM loc_9564 writes off_9688 (`berzerker_run_*`) immediately
            // after the charge accumulator carries. That sprite write survives
            // loc_9326 clearing Berzerker_charging after wall contact, so the
            // latched pose must also win over idle when native velocity stops.
            let frame_index =
                usize::from(trace.walk_visual_accumulator >> 8) % player_visuals.charge.len();
            &player_visuals.charge[frame_index]
        } else if trace.current_helmet_id == PlayerForm::Berzerker.helmet_id()
            && trace.kid_on_slope != 0
        {
            // ASM loc_A13A / off_A190: slope animation table selection first
            // canonicalises x_direction through $FFFFFA26. SlopeUp keeps the
            // live facing byte; SlopeDown XORs it with $FF before loc_A16C
            // chooses the second six-frame table half.
            let facing = trace.kid_facing.unwrap_or(Direction::Right);
            let canonical_reverse = match (trace.kid_on_slope < 0, facing) {
                (false, Direction::Right) | (true, Direction::Left) => false,
                (false, Direction::Left) | (true, Direction::Right) => true,
            };
            let slope_frames = match canonical_reverse {
                false if !player_visuals.slope_walk_left.is_empty() => {
                    &player_visuals.slope_walk_left
                }
                true if !player_visuals.slope_walk.is_empty() => &player_visuals.slope_walk,
                _ => &player_visuals.walk,
            };
            // ASM loc_A13A writes the slope frame through `$FFFFF8F0 >> 8` for
            // every form. Native used to approximate non-Juggernaut slopes
            // from frame time, which made downhill/reverse motion animate out
            // of phase with the ROM accumulator.
            let frame_index = usize::from(trace.walk_visual_accumulator >> 8) % slope_frames.len();
            &slope_frames[frame_index]
        } else if trace.current_helmet_id == PlayerForm::Berzerker.helmet_id()
            && trace.berzerker_walk_pose
            && !player_visuals.walk.is_empty()
        {
            // ASM loc_9580 -> loc_95DA -> loc_9600 writes Berzerker's
            // ordinary off_967C walk sprite before later wall collision can
            // leave the sampled post-frame velocity at zero, but loc_A13A's
            // slope sprite write happens later and therefore wins above.
            let frame_index =
                usize::from(trace.walk_visual_accumulator >> 8) % player_visuals.walk.len();
            &player_visuals.walk[frame_index]
        } else if trace.kid_on_slope != 0 {
            // ASM loc_A13A / off_A190: non-Berzerker slope animation table
            // selection follows the same `$FFFFF8F0 >> 8` phase and happens
            // before the standing-still draw state, so wedged forms do not
            // flicker between idle and slope art when x_vel reaches zero.
            let facing = trace.kid_facing.unwrap_or(Direction::Right);
            let canonical_reverse = match (trace.kid_on_slope < 0, facing) {
                (false, Direction::Right) | (true, Direction::Left) => false,
                (false, Direction::Left) | (true, Direction::Right) => true,
            };
            let slope_frames = match canonical_reverse {
                false if !player_visuals.slope_walk_left.is_empty() => {
                    &player_visuals.slope_walk_left
                }
                true if !player_visuals.slope_walk.is_empty() => &player_visuals.slope_walk,
                _ => &player_visuals.walk,
            };
            let frame_index = usize::from(trace.walk_visual_accumulator >> 8) % slope_frames.len();
            &slope_frames[frame_index]
        } else if x_velocity == 0 {
            &player_visuals.idle
        } else {
            // ASM loc_95DA -> loc_9600/off_9618 indexes ordinary walking
            // frames from `$FFFFF8F0 >> 8` for Kid and helmet forms. Native
            // used to drive most forms from wall-clock frame time, which put
            // Red Stealth's held-right slope approach visibly out of phase.
            let frame_index =
                usize::from(trace.walk_visual_accumulator >> 8) % player_visuals.walk.len();
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
    player_visuals: &'a wildside_render::PlayerVisuals,
) -> &'a wildside_render::PlayerFormVisuals {
    player_visuals
        .forms
        .get(&trace.current_helmet_id)
        .or_else(|| player_visuals.forms.get(&0))
        .expect("Kid visuals should always be loaded")
}

pub(super) fn select_player_transform_frame<'a>(
    trace: &NativeTraceFrame,
    player_visuals: &'a wildside_render::PlayerFormVisuals,
) -> Option<&'a wildside_render::SpriteFrame> {
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

fn transform_frame_palette<'a>(
    frame: &wildside_render::SpriteFrame,
    player_visuals: &'a wildside_render::PlayerVisuals,
    form_visuals: &'a wildside_render::PlayerFormVisuals,
) -> &'a Palette {
    if let Some(kid_visuals) = player_visuals.forms.get(&0)
        && kid_transform_frame_uses_kid_palette_loc_7cd0(frame, kid_visuals)
    {
        // ASM Kid_Transform loc_7CD0 (kid.asm:8426) only switches
        // Current_Helmet after the old-form table. The first-helmet
        // transforms can therefore render Kid art while the trace already
        // reports the target helmet; keep that art on Kid's palette instead
        // of the target transform palette.
        return &kid_visuals.palette;
    }

    form_visuals
        .transform_palette
        .as_ref()
        .unwrap_or(&form_visuals.palette)
}

fn kid_transform_frame_uses_kid_palette_loc_7cd0(
    frame: &wildside_render::SpriteFrame,
    kid_visuals: &wildside_render::PlayerFormVisuals,
) -> bool {
    frame == &kid_visuals.idle
        || kid_visuals.walk.iter().any(|kid_frame| frame == kid_frame)
        || frame == &kid_visuals.jump_rise
        || frame == &kid_visuals.jump_fall
        || kid_visuals
            .death
            .as_ref()
            .is_some_and(|kid_frame| frame == kid_frame)
        || kid_visuals
            .pull_up
            .iter()
            .any(|kid_frame| frame == kid_frame)
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

pub(super) fn boss_eye_attack_highlight_offset_sub_37abc(mapping_index: u8) -> (i32, i32) {
    match mapping_index & 0x07 {
        0 => (-4, -4),
        1 => (0, -5),
        2 => (4, -4),
        3 => (-5, 0),
        4 => (5, 0),
        5 => (-4, 4),
        6 => (0, 5),
        _ => (4, 4),
    }
}

pub(super) fn push_boss_eye_attack_marker_sub_37abc(
    commands: &mut Vec<DrawCommand>,
    sx: i32,
    sy: i32,
    mapping_index: u8,
) {
    // ASM sub_37332/loc_3739C and sub_37ABC select one of
    // `unk_C8488..unk_C84C0`, all decoded as 16x16 HeadyMetal eye frames.
    // Until those boss-head tiles are available in native play, keep the ROM
    // footprint and expose the selected frame with a small indexed highlight.
    commands.push(DrawCommand::Rect {
        x: sx - 8,
        y: sy - 8,
        width: 16,
        height: 16,
        colour: [120, 200, 255],
    });
    let (dx, dy) = boss_eye_attack_highlight_offset_sub_37abc(mapping_index);
    commands.push(DrawCommand::Rect {
        x: sx + dx - 2,
        y: sy + dy - 2,
        width: 4,
        height: 4,
        colour: [255, 255, 255],
    });
}

pub(super) fn push_player_sprite_with_priority_override(
    commands: &mut Vec<DrawCommand>,
    object: &NativeObjectSnapshot,
    trace: &NativeTraceFrame,
    level_visuals: &NativeLevelVisuals,
    damage_flash_white_phase: bool,
    priority_override: Option<bool>,
) {
    if trace.death_windup_elapsed > 0
        && let Some(kid_visuals) = level_visuals.player.forms.get(&0)
        && let Some(frame) = kid_visuals.death.as_ref()
    {
        // ASM Death loc_B7EA/loc_B7FA: helmeted deaths first run
        // Kid_Transform, then loc_B7FA installs `unk_A3FF8` (`kid_death.bin`)
        // for the tumble loop. Use Kid's death art/palette even if the trace
        // still carries the pre-death form until the hidden `lose_life` shell.
        let render_facing = trace.kid_facing.unwrap_or(Direction::Right);
        let (screen_x, screen_y) = player_screen_position(object, trace);
        let (draw_x, draw_y) = sprite_draw_position(frame, screen_x, screen_y, render_facing);
        commands.push(DrawCommand::Sprite {
            x: draw_x,
            y: draw_y,
            tile_sheet: Arc::new(frame.tile_sheet.clone()),
            palette: Arc::new(player_palette_for_damage_phase(
                &kid_visuals.palette,
                damage_flash_white_phase,
            )),
            tiles: build_player_sprite_tiles_with_flip(
                frame,
                matches!(render_facing, Direction::Left),
                priority_override,
            ),
        });
        return;
    }

    let form_visuals = select_player_form_visuals(trace, &level_visuals.player);
    let render_facing = trace.kid_facing.unwrap_or(Direction::Right);
    let (screen_x, screen_y) = player_screen_position(object, trace);
    if let Some(frame) = select_player_transform_frame(trace, form_visuals) {
        let (draw_x, draw_y) = sprite_draw_position(frame, screen_x, screen_y, render_facing);
        let base_palette = transform_frame_palette(frame, &level_visuals.player, form_visuals);
        commands.push(DrawCommand::Sprite {
            x: draw_x,
            y: draw_y,
            tile_sheet: Arc::new(frame.tile_sheet.clone()),
            palette: Arc::new(player_palette_for_damage_phase(
                base_palette,
                damage_flash_white_phase,
            )),
            tiles: build_player_sprite_tiles_with_flip(
                frame,
                matches!(render_facing, Direction::Left),
                priority_override,
            ),
        });
        return;
    }

    let frame = select_player_frame(trace, form_visuals);
    let (draw_x, draw_y) = sprite_draw_position(frame, screen_x, screen_y, render_facing);
    // ASM loc_A13A/off_A190 chooses the tilted slope frame bank, but the
    // later loc_1292 draw path still flips solely from live x_direction.
    let flip_x = matches!(render_facing, Direction::Left);

    commands.push(DrawCommand::Sprite {
        x: draw_x,
        y: draw_y,
        tile_sheet: Arc::new(frame.tile_sheet.clone()),
        palette: Arc::new(player_palette_for_damage_phase(
            &form_visuals.palette,
            damage_flash_white_phase,
        )),
        tiles: build_player_sprite_tiles_with_flip(frame, flip_x, priority_override),
    });
}

fn player_palette_for_damage_phase(palette: &Palette, white_phase: bool) -> Palette {
    if !white_phase {
        return palette.clone();
    }

    let mut flash_palette = palette.clone();
    let white = MdColour::from_word(0x0FFF);
    // ASM sub_80BC writes twelve $FFF words starting at Palette_Buffer+$62:
    // player palette line visible slots 1..12. Slot 0 is omitted from native
    // player palettes, so indices 0..11 correspond to those visible slots.
    for colour in flash_palette.colours.iter_mut().take(12) {
        *colour = white;
    }
    flash_palette
}

fn enemy_palette_for_damage_flash(palette: &Palette, damage_flash: bool) -> Palette {
    if !damage_flash {
        return palette.clone();
    }

    let mut flash_palette = palette.clone();
    let white = MdColour::from_word(0x0FFF);
    // ASM enemy hurt routes initialise a short hibernate/recoil animation;
    // native's `damage_flash` trace flag is the observable rendering cue for
    // that non-lethal hit. Palette index 0 still resolves as transparent in
    // the renderer, so whitening the stored colours keeps sprite holes clear.
    for colour in &mut flash_palette.colours {
        *colour = white;
    }
    flash_palette
}

fn enemy_palette_for_heady_metal_sub_379e8(
    palette: &Palette,
    object: &NativeObjectSnapshot,
    object_type: u8,
) -> Palette {
    let mut palette = palette.clone();
    if object_type != 0x20 {
        return palette;
    }

    let Some(raw_word) = object
        .debug_fields
        .get("heady_metal_palette_word_5e_sub_379e8")
        .and_then(|word| {
            word.strip_prefix("0x")
                .and_then(|hex| u16::from_str_radix(hex, 16).ok())
                .or_else(|| word.parse::<u16>().ok())
        })
    else {
        return palette;
    };

    if let Some(slot) = palette.colours.get_mut(15) {
        // ASM sub_379E8 (kid.asm:39865): write `unk_379A8[d2]` to
        // Palette_Buffer+$5E, i.e. palette line 2 colour slot 15.
        *slot = MdColour::from_word(raw_word);
    }
    palette
}

fn select_waited_sequence_frame(remaining: u16, sequence: &[(u16, usize)]) -> Option<usize> {
    let mut rest = sequence.iter().map(|(duration, _)| *duration).sum::<u16>();
    for &(duration, frame_index) in sequence {
        if remaining > rest.saturating_sub(duration) {
            return Some(frame_index);
        }
        rest = rest.saturating_sub(duration);
    }
    None
}

fn select_live_walking_dragon_frame<'a>(
    trace: &NativeTraceFrame,
    enemy_visuals: &'a wildside_render::AnimatedSpriteVisual,
    object: &NativeObjectSnapshot,
) -> Option<&'a wildside_render::SpriteFrame> {
    if !matches!(object.kind, NativeObjectKind::Enemy { object_type: 0x0D })
        || object.hit_points == Some(0)
        || enemy_visuals.frames.len() < 8
    {
        return None;
    }

    // ASM Dragon.asm loc_3CE48/loc_3CDFC installs `stru_3CC5E` for normal
    // walking dragons and `stru_3CC90` for HP-3 dragons. The extracted native
    // sheet appends `loc_3CEFC` death/smoke frames after the live frames, so
    // live animation must follow the ROM sequence rather than cycling every
    // frame in the sheet.
    if object.walk_phase == Some(1) && enemy_visuals.frames.len() > 10 {
        // ASM Dragon.asm loc_3CF90 starts `stru_3CCC2` on the parent while
        // the child breath object (`stru_3CCC8`) is active.
        return Some(&enemy_visuals.frames[10]);
    }

    let live_sequence = [0, 1, 2, 3, 4, 5, 0, 1, 2, 3, 7, 5];
    let frame_duration = if object.hit_points == Some(3) { 6 } else { 9 };
    let phase = (trace.frame_index / frame_duration) % live_sequence.len();
    Some(&enemy_visuals.frames[live_sequence[phase]])
}

fn select_live_rock_tank_body_frame<'a>(
    trace: &NativeTraceFrame,
    enemy_visuals: &'a wildside_render::AnimatedSpriteVisual,
    object: &NativeObjectSnapshot,
) -> Option<&'a wildside_render::SpriteFrame> {
    if !matches!(
        object.kind,
        NativeObjectKind::Enemy {
            object_type: 0x0A | 0x0B
        }
    ) || object.hit_points == Some(0)
        || enemy_visuals.frames.len() < 3
    {
        return None;
    }

    // ASM Tank.asm `stru_3B270` and Tank_Shooting.asm `stru_3B4CA` animate
    // the body through only C7ED0/C7ED8/C7EE0. Native appends death frames
    // after those three entries, so live cycling must not include them.
    let phase = (trace.frame_index / enemy_visuals.frame_duration.max(1)) % 3;
    Some(&enemy_visuals.frames[phase])
}

fn select_live_diamond_frame<'a>(
    trace: &NativeTraceFrame,
    enemy_visuals: &'a wildside_render::AnimatedSpriteVisual,
    object: &NativeObjectSnapshot,
) -> Option<&'a wildside_render::SpriteFrame> {
    if !matches!(object.kind, NativeObjectKind::Enemy { object_type: 0x01 })
        || object.hit_points == Some(0)
        || enemy_visuals.frames.len() < 5
    {
        return None;
    }

    // ASM Diamond.asm Enemy01_Diamond_Init installs `stru_3ACA8`: five
    // six-tick live frames, then dispatch selector 2 rewinds to the start.
    // The later extracted frames are hurt/death-only (`stru_3ACBE`/`stru_3ACC4`),
    // so live Diamonds must not cycle the whole sheet or desynchronise by
    // layout index.
    let phase = (trace.frame_index / 6) % 5;
    Some(&enemy_visuals.frames[phase])
}

fn select_live_flying_dragon_frame<'a>(
    trace: &NativeTraceFrame,
    enemy_visuals: &'a wildside_render::AnimatedSpriteVisual,
    object: &NativeObjectSnapshot,
) -> Option<&'a wildside_render::SpriteFrame> {
    if !matches!(object.kind, NativeObjectKind::Enemy { object_type: 0x0C })
        || object.hit_points == Some(0)
        || enemy_visuals.frames.len() < 5
    {
        return None;
    }

    if let Some(frame) = select_flying_dragon_dive_frame(enemy_visuals, object) {
        return Some(frame);
    }

    // ASM Dragon_Flying.asm loc_346DA runs `stru_34600`, whose visible
    // sequence repeats C75EE/C75F6/C75FE/C7606 before occasionally showing
    // C760E. It is not the simple five-frame loop used by the fallback path.
    const STRU_34600_FRAME_SEQUENCE: [usize; 20] =
        [0, 1, 2, 3, 0, 1, 2, 3, 0, 1, 2, 3, 4, 1, 2, 3, 4, 1, 2, 3];
    let phase =
        (trace.frame_index / enemy_visuals.frame_duration.max(1)) % STRU_34600_FRAME_SEQUENCE.len();
    Some(&enemy_visuals.frames[STRU_34600_FRAME_SEQUENCE[phase]])
}

fn select_flying_dragon_dive_frame<'a>(
    enemy_visuals: &'a wildside_render::AnimatedSpriteVisual,
    object: &NativeObjectSnapshot,
) -> Option<&'a wildside_render::SpriteFrame> {
    if enemy_visuals.frames.len() < 9 {
        return None;
    }
    let x_px = object.x_velocity? / 0x0001_0000;
    let y_px = object.y_velocity? / 0x0001_0000;
    let facing = object.facing?;

    // ASM Dragon_Flying.asm loc_348CE reads the dive script's sprite byte and
    // indexes `off_345F6`: C75FE, C7616, C761E, C762E, C7636. Native does not
    // yet expose the script pointer, so match the exact pixel/frame velocity
    // bands from `unk_34554`.
    let frame_index = match (x_px.abs(), y_px, facing) {
        (1, 1, _) | (2, 1..=3, _) => 5,                   // LnkTo_unk_C7616
        (1, 2..=4, _) => 6,                               // LnkTo_unk_C761E
        (2, 0, _) => 2,                                   // LnkTo_unk_C75FE
        (2, -3..=-1, _) | (1, -1, _) => 7,                // LnkTo_unk_C762E
        (1, -4, _) | (1, -3..=-2, Direction::Right) => 8, // LnkTo_unk_C7636
        _ => return None,
    };
    Some(&enemy_visuals.frames[frame_index])
}

fn select_live_tar_monster_frame<'a>(
    trace: &NativeTraceFrame,
    enemy_visuals: &'a wildside_render::AnimatedSpriteVisual,
    object: &NativeObjectSnapshot,
) -> Option<&'a wildside_render::SpriteFrame> {
    let _ = trace;
    if !matches!(object.kind, NativeObjectKind::Enemy { object_type: 0x05 })
        || object.hit_points == Some(0)
        || enemy_visuals.frames.len() < 2
    {
        return None;
    }
    // ASM Tar_Monster.asm off_32290 dispatches stru_3272E (emerge,
    // C77A6->C77D6) and stru_3274C (submerge, C77D6->C77A6). The extracted
    // sheet keeps only the hidden body (frame 0 = C77A6) and the fully emerged
    // body (frame 1 = C77D6); frames 2.. are the stru_32700 submerge DEATH
    // tail. A live tar monster holds one of the two body frames off its phase
    // instead of free-running the whole sheet into the death frames. The 7-frame
    // emerge/submerge intermediates are not extracted, so the transition pops
    // rather than tweens (atlas limit, not a regression). contact_phase
    // TarMonsterHidden is set whenever tar_monster_state is not Visible/Attacking
    // (enemies.rs:9399-9408).
    let frame = if matches!(
        object.contact_phase,
        Some(NativeContactPhase::TarMonsterHidden)
    ) {
        0
    } else {
        1
    };
    Some(&enemy_visuals.frames[frame])
}

fn select_live_sphere_frame<'a>(
    trace: &NativeTraceFrame,
    enemy_visuals: &'a wildside_render::AnimatedSpriteVisual,
    object: &NativeObjectSnapshot,
) -> Option<&'a wildside_render::SpriteFrame> {
    if !matches!(object.kind, NativeObjectKind::Enemy { object_type: 0x06 })
        || object.hit_points == Some(0)
        || enemy_visuals.frames.len() < 3
    {
        return None;
    }
    // ASM Sphere.asm installs stru_36174 once at Enemy06_Sphere_Init and
    // Enemy06_Sphere_Loop never re-inits, so the live sphere spins through
    // exactly three frames C8120/C8128/C8130 (frames 0,1,2) via the `dc.b 2,$D`
    // rewind. Frames 3.. are the stru_36182 disappear/death tail and must not
    // appear while alive.
    let phase = (trace.frame_index / enemy_visuals.frame_duration.max(1)) % 3;
    Some(&enemy_visuals.frames[phase])
}

fn select_live_emo_rock_frame<'a>(
    trace: &NativeTraceFrame,
    enemy_visuals: &'a wildside_render::AnimatedSpriteVisual,
    object: &NativeObjectSnapshot,
) -> Option<&'a wildside_render::SpriteFrame> {
    if !matches!(object.kind, NativeObjectKind::Enemy { object_type: 0x1B })
        || object.hit_points == Some(0)
        || enemy_visuals.frames.len() < 3
    {
        return None;
    }
    // ASM Emo_Rock.asm: loc_3621A hovers with stru_3647E (C7F48/C7F50/C7F38) and
    // loc_362A4 drops with stru_363FE (C7F40/C7F58/C7F60 = atlas frames 0/1/2);
    // loc_36346 installs the stru_36468 SHATTER death tail (atlas frames 3..7).
    // The live bug was the generic free-run cycling those shatter frames on a
    // live rock. Restrict a live Emo Rock to its body frames 0..2 so the death
    // tail can never show. TODO(fidelity): the hover frames stru_3647E are not
    // extracted into the atlas, so cycling the stru_363FE drop frames is an
    // approximation of the hover phase; extract C7F48/C7F50/C7F38 and gate the
    // drop animation on the landed phase for an exact match.
    let phase = (trace.frame_index / enemy_visuals.frame_duration.max(1)) % 3;
    Some(&enemy_visuals.frames[phase])
}

pub(super) fn select_enemy_frame<'a>(
    trace: &NativeTraceFrame,
    enemy_visuals: &'a wildside_render::AnimatedSpriteVisual,
    object: &NativeObjectSnapshot,
) -> &'a wildside_render::SpriteFrame {
    if object.hit_points == Some(0)
        && let NativeObjectKind::Enemy { object_type } = object.kind
        && let Some(remaining) = object.wait_timer
    {
        let death_frame = match object_type {
            // ASM Fire_Demon.asm loc_3A5A6 runs `stru_3A378`: three
            // 3-tick frames, then three 4-tick frames before deletion.
            0x00 if enemy_visuals.frames.len() >= 12 => {
                if remaining > 0x03 + 0x03 + 0x04 + 0x04 + 0x04 {
                    Some(6)
                } else if remaining > 0x03 + 0x04 + 0x04 + 0x04 {
                    Some(7)
                } else if remaining > 0x04 + 0x04 + 0x04 {
                    Some(8)
                } else if remaining > 0x04 + 0x04 {
                    Some(9)
                } else if remaining > 0x04 {
                    Some(10)
                } else {
                    Some(11)
                }
            }
            // ASM Diamond.asm loc_3AE28/loc_3AEE6 runs `stru_3ACC4`:
            // C8158 for $08, C8138 for $06, C8140/C8148 for $03 each,
            // then C8150 for $02 before the shared delete tail.
            0x01 if enemy_visuals.frames.len() >= 10 => {
                if remaining > 0x06 + 0x03 + 0x03 + 0x02 {
                    Some(5)
                } else if remaining > 0x03 + 0x03 + 0x02 {
                    Some(6)
                } else if remaining > 0x03 + 0x02 {
                    Some(7)
                } else if remaining > 0x02 {
                    Some(8)
                } else {
                    Some(9)
                }
            }
            // ASM Robot.asm loc_3A9D2 runs `stru_3A74A`: C8318 for $09,
            // then C8320 for $09 before deletion.
            0x03 if enemy_visuals.frames.len() >= 8 => {
                if remaining > 0x09 {
                    Some(6)
                } else {
                    Some(7)
                }
            }
            // ASM Sphere.asm Enemy06_Sphere_Kill runs `stru_36182`: C8108
            // for $0A, C8110 for $0A, then C8118 for $0A before deletion.
            0x06 if enemy_visuals.frames.len() >= 6 => {
                if remaining > 0x14 {
                    Some(3)
                } else if remaining > 0x0A {
                    Some(4)
                } else {
                    Some(5)
                }
            }
            // ASM Archer.asm loc_3414A runs `stru_33FC2`: C81B8 for $05,
            // C81F0 for $05, C81C8 for $05, then C81D0 for $23 before delete.
            0x07 if enemy_visuals.frames.len() >= 10 => {
                if remaining > 0x05 + 0x05 + 0x23 {
                    Some(6)
                } else if remaining > 0x05 + 0x23 {
                    Some(7)
                } else if remaining > 0x23 {
                    Some(8)
                } else {
                    Some(9)
                }
            }
            // ASM Armadillo.asm loc_3C886 runs `stru_3C69A`: C76D6/C76DE
            // flash with shrinking delays, then C76E6 for $03 before delete.
            0x04 if enemy_visuals.frames.len() >= 9 => select_waited_sequence_frame(
                remaining,
                &[
                    (0x16, 6),
                    (0x04, 7),
                    (0x04, 6),
                    (0x03, 7),
                    (0x03, 6),
                    (0x02, 7),
                    (0x02, 6),
                    (0x01, 7),
                    (0x01, 6),
                    (0x01, 7),
                    (0x01, 6),
                    (0x01, 7),
                    (0x03, 8),
                ],
            ),
            // ASM Tar_Monster.asm loc_323E6 runs `stru_32700`: five
            // shrinking submerge frames before the status/delete tail.
            0x05 if enemy_visuals.frames.len() >= 7 => select_waited_sequence_frame(
                remaining,
                &[
                    (0x09, 2),
                    (0x09, 3),
                    (0x03, 4),
                    (0x03, 3),
                    (0x09, 4),
                    (0x03, 5),
                    (0x03, 4),
                    (0x09, 5),
                    (0x03, 6),
                    (0x03, 5),
                    (0x09, 6),
                ],
            ),
            // ASM Orca.asm loc_3D268/loc_3D4BC runs `stru_3D11E`: C7656
            // and C7696 lead into shrinking C763E/C7646 flashes, then C764E.
            0x08 if enemy_visuals.frames.len() >= 11 => select_waited_sequence_frame(
                remaining,
                &[
                    (0x09, 6),
                    (0x04, 7),
                    (0x03, 6),
                    (0x11, 8),
                    (0x04, 9),
                    (0x04, 8),
                    (0x03, 9),
                    (0x03, 8),
                    (0x02, 9),
                    (0x02, 8),
                    (0x02, 9),
                    (0x02, 8),
                    (0x02, 9),
                    (0x63, 10),
                ],
            ),
            // ASM Crab.asm loc_3B094 runs `stru_3AF82`: C7C16 for $14,
            // C7C06 for $0A, then C7C0E for $1E before deletion.
            0x09 if enemy_visuals.frames.len() >= 9 => {
                if remaining > 0x0A + 0x1E {
                    Some(6)
                } else if remaining > 0x1E {
                    Some(7)
                } else {
                    Some(8)
                }
            }
            // ASM Tank.asm loc_3B480 runs `stru_3B29A`: C7EB8 for $10,
            // C7EC0 for $08, then C7F18 for $3C before deletion.
            0x0A if enemy_visuals.frames.len() >= 6 => {
                if remaining > 0x08 + 0x3C {
                    Some(3)
                } else if remaining > 0x3C {
                    Some(4)
                } else {
                    Some(5)
                }
            }
            // ASM Tank_Shooting.asm loc_3B6E6 runs `stru_3B518`: C7EB8
            // for $10, C7EC0 for $08, then C7EF8 for $3C before deletion.
            0x0B if enemy_visuals.frames.len() >= 6 => {
                if remaining > 0x08 + 0x3C {
                    Some(3)
                } else if remaining > 0x3C {
                    Some(4)
                } else {
                    Some(5)
                }
            }
            // ASM Dragon.asm loc_3CEFC runs `stru_3CCEC`: C755E/C75C6
            // lead into the C7566/C756E smoke/body loop before deletion.
            0x0D if enemy_visuals.frames.len() >= 10 => select_waited_sequence_frame(
                remaining,
                &[
                    (0x07, 6),
                    (0x06, 7),
                    (0x03, 6),
                    (0x16, 8),
                    (0x05, 9),
                    (0x04, 8),
                    (0x04, 9),
                    (0x0C, 8),
                    (0x05, 9),
                    (0x63, 8),
                ],
            ),
            // ASM Cloud.asm loc_35C0E runs `stru_35EBA`: C789E, C78A6,
            // then C78AE for seven ticks each before deletion.
            0x0E if enemy_visuals.frames.len() >= 9 => {
                if remaining > 0x0E {
                    Some(6)
                } else if remaining > 0x07 {
                    Some(7)
                } else {
                    Some(8)
                }
            }
            // ASM Goat.asm loc_3CB64 runs `stru_3C9EA`: C7C56 for $1E,
            // C7C8E for $0A, then C7C96 for $37 before deletion.
            0x10 if enemy_visuals.frames.len() >= 9 => {
                if remaining > 0x0A + 0x37 {
                    Some(6)
                } else if remaining > 0x37 {
                    Some(7)
                } else {
                    Some(8)
                }
            }
            // ASM Spinning_Twins.asm loc_3BC56 runs `stru_3B83C`:
            // C7CDE for $19, C7CE6 for $0F, then C7CEE for $14.
            0x14 if enemy_visuals.frames.len() >= 7 => {
                if remaining > 0x0F + 0x14 {
                    Some(4)
                } else if remaining > 0x14 {
                    Some(5)
                } else {
                    Some(6)
                }
            }
            // ASM Drips.asm loc_3392C runs `stru_33F06`: C7BC6/C7BCE/C7BD6
            // for four ticks each before the object deletes itself.
            0x16 if enemy_visuals.frames.len() >= 6 => {
                if remaining > 0x08 {
                    Some(3)
                } else if remaining > 0x04 {
                    Some(4)
                } else {
                    Some(5)
                }
            }
            // ASM Hand.asm loc_32AA2 runs `stru_32B78`: C79E6/C79EE repeat
            // three times, then C79F6/C79FE/C7A06/C7A0E/C7A16.
            0x17 if enemy_visuals.frames.len() >= 8 => {
                if remaining > 0x3C {
                    Some(1)
                } else if remaining > 0x36 {
                    Some(2)
                } else if remaining > 0x30 {
                    Some(1)
                } else if remaining > 0x2A {
                    Some(2)
                } else if remaining > 0x24 {
                    Some(1)
                } else if remaining > 0x1E {
                    Some(2)
                } else if remaining > 0x18 {
                    Some(3)
                } else if remaining > 0x12 {
                    Some(4)
                } else if remaining > 0x0C {
                    Some(5)
                } else if remaining > 0x06 {
                    Some(6)
                } else {
                    Some(7)
                }
            }
            // ASM Ninja.asm loc_3DB4E runs `stru_3D50A`: C7D56 for $14,
            // C7D5E for $0A, then C7D66 for $2D before deletion.
            0x11 if enemy_visuals.frames.len() >= 9 => {
                if remaining > 0x0A + 0x2D {
                    Some(6)
                } else if remaining > 0x2D {
                    Some(7)
                } else {
                    Some(8)
                }
            }
            // ASM Lion.asm loc_3E378 runs `stru_3DF48`: C7E38, C7E40,
            // then C7E48 for ten ticks each before deletion.
            0x12 if enemy_visuals.frames.len() >= 9 => {
                if remaining > 0x0A + 0x0A {
                    Some(6)
                } else if remaining > 0x0A {
                    Some(7)
                } else {
                    Some(8)
                }
            }
            // ASM Scorpion.asm loc_3DDEE runs `stru_3DB80`: C7DD6 for
            // $0A, C7DC6 for $14, then C7DCE for $2D before deletion.
            0x13 if enemy_visuals.frames.len() >= 9 => {
                if remaining > 0x14 + 0x2D {
                    Some(6)
                } else if remaining > 0x2D {
                    Some(7)
                } else {
                    Some(8)
                }
            }
            // ASM UFO.asm loc_35174/loc_35180 runs parent `stru_34CA0`:
            // C83B0, C83B8, C83C0 for 7 ticks each before loc_351B6 delete.
            0x0F if enemy_visuals.frames.len() >= 3 => {
                if remaining > 0x0E {
                    Some(0)
                } else if remaining > 0x07 {
                    Some(1)
                } else {
                    Some(2)
                }
            }
            // ASM Dragon_Flying.asm loc_3492C runs shared `stru_345E4`:
            // four 2-tick frames while the flying parent hibernates before
            // loc_349A6 deletes it.
            0x0C if enemy_visuals.frames.len() >= 4 => {
                if remaining > 0x06 {
                    Some(0)
                } else if remaining > 0x04 {
                    Some(1)
                } else if remaining > 0x02 {
                    Some(2)
                } else {
                    Some(3)
                }
            }
            // ASM Fireball.asm Enemy19_Fireball_Kill reuses `stru_345E4`:
            // four 2-tick frames while the fireball hibernates before
            // loc_32ED8 deletes it.
            0x19 if enemy_visuals.frames.len() >= 4 => {
                if remaining > 0x06 {
                    Some(0)
                } else if remaining > 0x04 {
                    Some(1)
                } else if remaining > 0x02 {
                    Some(2)
                } else {
                    Some(3)
                }
            }
            // ASM Tornado.asm loc_359B2 runs `stru_35B1E`: C783E/C7846
            // alternate across $06/$03/$04/$04/$03/$06 tick spans.
            0x18 if enemy_visuals.frames.len() >= 5 => {
                if remaining > 0x03 + 0x04 + 0x04 + 0x03 + 0x06 {
                    Some(3)
                } else if remaining > 0x04 + 0x04 + 0x03 + 0x06 {
                    Some(4)
                } else if remaining > 0x04 + 0x03 + 0x06 {
                    Some(3)
                } else if remaining > 0x03 + 0x06 {
                    Some(4)
                } else if remaining > 0x06 {
                    Some(3)
                } else {
                    Some(4)
                }
            }
            // ASM Emo_Rock.asm loc_36346 runs `stru_36468`: five 6-tick
            // frames C7F68/C7F70/C7F78/C7F80/C7F88 before deletion.
            0x1B if enemy_visuals.frames.len() >= 8 => {
                if remaining > 0x18 {
                    Some(3)
                } else if remaining > 0x12 {
                    Some(4)
                } else if remaining > 0x0C {
                    Some(5)
                } else if remaining > 0x06 {
                    Some(6)
                } else {
                    Some(7)
                }
            }
            // ASM Driller.asm loc_3BF1C runs parent `stru_3BCD4`: C80D8 for
            // $0A, C80C8 for $14, then C80D0 for $28 before deletion.
            0x1A if enemy_visuals.frames.len() >= 5 => {
                if remaining > 0x14 + 0x28 {
                    Some(2)
                } else if remaining > 0x28 {
                    Some(3)
                } else {
                    Some(4)
                }
            }
            // ASM Big_Hopping_Skull.asm loc_3664E runs `stru_367FA`:
            // C7FA0 for $18, C7FE8 for $0C, C7FF0 for $0A, then C7FF8
            // for $64 before deletion.
            0x1D if enemy_visuals.frames.len() >= 5 => {
                if remaining > 0x0C + 0x0A + 0x64 {
                    Some(1)
                } else if remaining > 0x0A + 0x64 {
                    Some(2)
                } else if remaining > 0x64 {
                    Some(3)
                } else {
                    Some(4)
                }
            }
            // ASM loc_39EBE/loc_39F7A shared boss-head death spin uses
            // C8620/C8618/C8610/C8608/C8600 sweeps, then ten fast spin loops.
            0x22 if enemy_visuals.frames.len() >= 5 => {
                Some(boss_head_death_frame_loc_39ebe(remaining))
            }
            _ => None,
        };
        if let Some(frame_index) = death_frame {
            return &enemy_visuals.frames[frame_index];
        }
    }
    if matches!(object.kind, NativeObjectKind::Enemy { object_type: 0x1C })
        && object.hit_points == Some(0)
        && let Some(remaining) = object.wait_timer
        && enemy_visuals.frames.len() >= 8
    {
        // ASM Mini_Hopping_Skull.asm loc_3F158 runs `stru_3E9AC`: C8040 for
        // $0A ticks, then C8048 for $1E ticks before deletion.
        let death_frame = if remaining > 0x1E { 6 } else { 7 };
        return &enemy_visuals.frames[death_frame];
    }
    if let Some(frame) = select_live_walking_dragon_frame(trace, enemy_visuals, object) {
        return frame;
    }
    if let Some(frame) = select_live_rock_tank_body_frame(trace, enemy_visuals, object) {
        return frame;
    }
    if let Some(frame) = select_live_diamond_frame(trace, enemy_visuals, object) {
        return frame;
    }
    if let Some(frame) = select_live_flying_dragon_frame(trace, enemy_visuals, object) {
        return frame;
    }
    if let Some(frame) = select_live_tar_monster_frame(trace, enemy_visuals, object) {
        return frame;
    }
    if let Some(frame) = select_live_sphere_frame(trace, enemy_visuals, object) {
        return frame;
    }
    if let Some(frame) = select_live_emo_rock_frame(trace, enemy_visuals, object) {
        return frame;
    }
    let keeps_base_animation = matches!(
        object.kind,
        NativeObjectKind::Enemy {
            object_type: 0x01 | 0x0C
        }
    );
    if object.stationary && !keeps_base_animation {
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
    let Some(enemy_visuals) = level_visuals.enemies.families.get(&object_type) else {
        let colour = if object.damage_flash {
            ENEMY_FLASH_COLOUR
        } else {
            enemy_family_colour(object_type)
        };
        commands.push(object_rect(object, trace, 16, 16, colour));
        return;
    };

    let (screen_x, screen_y) = player_screen_position(object, trace);
    let mut facing = object.facing.unwrap_or(Direction::Right);
    if object_type == 0x22
        && object.hit_points == Some(0)
        && let Some(remaining) = object.wait_timer
    {
        facing = boss_head_death_facing_loc_39ebe(facing, remaining);
    }
    // Enemy VRAM art is stored facing left (the ROM uses the H-flip bit to
    // show right-facing sprites). Invert the facing for rendering so that
    // Direction::Right flips the left-facing art to face right.
    let render_facing = opposite_direction(facing);

    if is_rock_tank_child_snapshot(object_type, object, trace) {
        if let Some(turret_frame) =
            rock_tank_turret_frame(object, trace, enemy_visuals, object_type)
        {
            // ASM Tank.asm loc_3B372 copies parent X/Y into the linked turret
            // GfxObject and subtracts 8 from child Y. ASM Make_SpriteFromGfxObject
            // loc_11EE still subtracts the child frame height, so do not add
            // the 8px back here; the child trace already carries the ROM offset.
            let turret_screen_y = rock_tank_turret_screen_y(object_type, screen_y);
            let (turret_x, turret_y) =
                sprite_draw_position(&turret_frame, screen_x, turret_screen_y, render_facing);
            let palette = enemy_palette_for_damage_flash(
                select_enemy_palette(enemy_visuals, object.palette_variant),
                rock_tank_damage_flash(object, trace, object_type),
            );
            commands.push(DrawCommand::Sprite {
                x: turret_x,
                y: turret_y,
                tile_sheet: Arc::new(turret_frame.tile_sheet.clone()),
                palette: Arc::new(palette),
                tiles: build_sprite_tiles(&turret_frame, render_facing, 0),
            });
        }
        return;
    }

    if enemy_visuals.frames.is_empty() {
        let colour = if object.damage_flash {
            ENEMY_FLASH_COLOUR
        } else {
            enemy_family_colour(object_type)
        };
        commands.push(object_rect(object, trace, 16, 16, colour));
        return;
    }

    let frame = select_enemy_frame(trace, enemy_visuals, object);
    let (draw_x, draw_y) = sprite_draw_position(frame, screen_x, screen_y, render_facing);
    let palette = enemy_palette_for_heady_metal_sub_379e8(
        select_enemy_palette(enemy_visuals, object.palette_variant),
        object,
        object_type,
    );
    let palette = enemy_palette_for_damage_flash(&palette, object.damage_flash);
    commands.push(DrawCommand::Sprite {
        x: draw_x,
        y: draw_y,
        tile_sheet: Arc::new(frame.tile_sheet.clone()),
        palette: Arc::new(palette),
        tiles: build_sprite_tiles(frame, render_facing, 0),
    });
    if object_type == 0x20 {
        push_heady_metal_sub_37abc_body_piece_markers(commands, object, trace);
    }

    if is_rock_tank_parent_without_child_snapshot(object_type, object, trace)
        && let Some(turret_frame) =
            rock_tank_turret_frame(object, trace, enemy_visuals, object_type)
    {
        // Fallback for traces that only expose the parent. ASM Tank.asm
        // loc_3B372 creates the real child at parent_y-8 before drawing.
        let turret_screen_y = rock_tank_turret_screen_y(object_type, screen_y - 8);
        let (turret_x, turret_y) =
            sprite_draw_position(&turret_frame, screen_x, turret_screen_y, render_facing);
        let palette = enemy_palette_for_damage_flash(
            select_enemy_palette(enemy_visuals, object.palette_variant),
            object.damage_flash,
        );
        commands.push(DrawCommand::Sprite {
            x: turret_x,
            y: turret_y,
            tile_sheet: Arc::new(turret_frame.tile_sheet.clone()),
            palette: Arc::new(palette),
            tiles: build_sprite_tiles(&turret_frame, render_facing, 0),
        });
    }
}

fn push_heady_metal_sub_37abc_body_piece_markers(
    commands: &mut Vec<DrawCommand>,
    object: &NativeObjectSnapshot,
    trace: &NativeTraceFrame,
) {
    let anchor_x = (object.x_pos >> 16) as u16;
    let anchor_y = (object.y_pos >> 16) as u16;
    let kid_x = trace.kid_x_pos.map_or(anchor_x, |x| (x >> 16) as u16);
    let kid_y = trace.kid_y_pos.map_or(anchor_y, |y| (y >> 16) as u16);
    let camera_x = (trace.camera_x_pos >> 16) as i32;
    let camera_y = (trace.camera_y_pos >> 16) as i32;

    if !trace.heady_metal_body_pieces_sub_37abc.is_empty() {
        for piece in &trace.heady_metal_body_pieces_sub_37abc {
            // ASM loc_37AC6 refreshes each non-null `$FFFFFB7E..$FFFFFBAA`
            // worker through `sub_37AF0`; the trace carries the refreshed
            // mapping so rendering does not have to reconstruct it.
            push_boss_eye_attack_marker_sub_37abc(
                commands,
                i32::from(piece.x_pos) - camera_x,
                i32::from(piece.y_pos) - camera_y,
                piece.sprite_mapping_index,
            );
        }
        return;
    }

    for &(x_offset, y_offset) in &wildside_core::HEADY_METAL_SUB_37ABC_BODY_PIECE_OFFSETS_LOC_39266
    {
        // ASM Enemy20_HeadyMetal_Init (kid.asm:39266-39321) seeds the twelve
        // `$FFFFFB7E` body-piece workers with these offsets. ASM sub_37ABC
        // then refreshes each worker's mapping every `loc_3763A` loop.
        let piece_x = anchor_x.wrapping_add_signed(x_offset);
        let piece_y = anchor_y.wrapping_add_signed(y_offset);
        let mapping_index = heady_metal_sub_37abc_mapping_index(piece_x, piece_y, kid_x, kid_y);
        push_boss_eye_attack_marker_sub_37abc(
            commands,
            i32::from(piece_x) - camera_x,
            i32::from(piece_y) - camera_y,
            mapping_index,
        );
    }
}

fn heady_metal_sub_37abc_mapping_index(piece_x: u16, piece_y: u16, kid_x: u16, kid_y: u16) -> u8 {
    if kid_y > piece_y.saturating_add(4) {
        if kid_x < piece_x.saturating_sub(4) {
            3
        } else if kid_x > piece_x.saturating_add(4) {
            5
        } else {
            4
        }
    } else if kid_y < piece_y.saturating_sub(4) {
        if kid_x < piece_x.saturating_sub(4) {
            1
        } else if kid_x > piece_x.saturating_add(4) {
            7
        } else {
            0
        }
    } else if kid_x < piece_x.saturating_sub(8) {
        2
    } else {
        6
    }
}

fn is_rock_tank_type(object_type: u8) -> bool {
    matches!(object_type, 0x0A | 0x0B)
}

fn is_rock_tank_child_snapshot(
    object_type: u8,
    object: &NativeObjectSnapshot,
    trace: &NativeTraceFrame,
) -> bool {
    // ASM Tank.asm loc_3B372 / Tank_Shooting.asm loc_3B5FA allocate the
    // linked turret as a zeroed GfxObject. Native traces may expose that child
    // as either `None` or `Some(0)` HP depending on the source path; require a
    // live same-layout parent so true zero-HP death parents still render their
    // death animation.
    is_rock_tank_type(object_type)
        && matches!(object.hit_points, None | Some(0))
        && trace.active_objects.iter().any(|candidate| {
            matches!(candidate.kind, NativeObjectKind::Enemy { object_type: candidate_type } if candidate_type == object_type)
                && candidate.layout_index == object.layout_index
                && candidate.hit_points.is_some_and(|hit_points| hit_points > 0)
        })
}

fn rock_tank_damage_flash(
    object: &NativeObjectSnapshot,
    trace: &NativeTraceFrame,
    object_type: u8,
) -> bool {
    if object.damage_flash {
        return true;
    }
    if !is_rock_tank_child_snapshot(object_type, object, trace) {
        return false;
    }

    trace.active_objects.iter().any(|candidate| {
        matches!(candidate.kind, NativeObjectKind::Enemy { object_type: candidate_type } if candidate_type == object_type)
            && candidate.layout_index == object.layout_index
            && candidate.hit_points.is_some()
            && candidate.damage_flash
    })
}

fn is_rock_tank_parent_without_child_snapshot(
    object_type: u8,
    object: &NativeObjectSnapshot,
    trace: &NativeTraceFrame,
) -> bool {
    if !is_rock_tank_type(object_type)
        || object.hit_points.is_none()
        || is_rock_tank_child_snapshot(object_type, object, trace)
    {
        return false;
    }
    !trace.active_objects.iter().any(|candidate| {
        matches!(candidate.kind, NativeObjectKind::Enemy { object_type: candidate_type } if candidate_type == object_type)
            && candidate.layout_index == object.layout_index
            && is_rock_tank_child_snapshot(object_type, candidate, trace)
    })
}

fn rock_tank_turret_screen_y(object_type: u8, child_screen_y: i32) -> i32 {
    if object_type == 0x0B {
        // ASM Tank_Shooting.asm loc_3B5FA copies the same `y_pos-8` child
        // setup as the plain tank, but the shooting turret frames are
        // top-anchored muzzle art. Compensate the child offset so those frames
        // share the tread baseline instead of stacking a second body above it.
        child_screen_y + 8
    } else {
        child_screen_y
    }
}

fn rock_tank_turret_frame(
    object: &NativeObjectSnapshot,
    trace: &NativeTraceFrame,
    enemy_visuals: &wildside_render::AnimatedSpriteVisual,
    object_type: u8,
) -> Option<wildside_render::SpriteFrame> {
    let hit_points = object.hit_points.or_else(|| {
        trace
            .active_objects
            .iter()
            .find(|candidate| {
                matches!(candidate.kind, NativeObjectKind::Enemy { object_type: candidate_type } if candidate_type == object_type)
                    && candidate.layout_index == object.layout_index
                    && candidate.hit_points.is_some()
            })
            .and_then(|candidate| candidate.hit_points)
    });
    let specs = match object_type {
        // ASM Tank.asm loc_3B372 + `stru_3B27E`: plain RockTank allocates a
        // second GfxObject at `$3A(a5)` for the turret and keeps it 8px above
        // the body while the body animation uses the first object.
        0x0A => &ROCK_TANK_TURRET_STRU_3B27E[..],
        // ASM Tank_Shooting.asm loc_3B5FA/loc_3B682: HP 4 uses `stru_3B4D8`,
        // HP 2-3 use `stru_3B4F6`, again on the child GfxObject at `$3A(a5)`.
        0x0B if hit_points == Some(4) => &ROCK_TANK_SHOOTING_FAST_STRU_3B4D8[..],
        0x0B => &ROCK_TANK_SHOOTING_SLOW_STRU_3B4F6[..],
        _ => return None,
    };
    let spec = scripted_tank_turret_frame(trace, specs)?;
    let tile_sheet = enemy_visuals.frames.first()?.tile_sheet.clone();
    Some(wildside_render::SpriteFrame {
        origin_x: spec.origin_x,
        origin_y: spec.origin_y,
        width_px: spec.width_px,
        height_px: spec.height_px,
        tile_base_index: spec.tile_base_index,
        tile_sheet,
    })
}

fn scripted_tank_turret_frame(
    trace: &NativeTraceFrame,
    specs: &[TankTurretFrameSpec],
) -> Option<TankTurretFrameSpec> {
    let cycle_frames = specs.iter().map(|spec| spec.duration_frames).sum::<usize>();
    if cycle_frames == 0 {
        return None;
    }
    let mut phase = trace.frame_index % cycle_frames;
    for spec in specs {
        if phase < spec.duration_frames {
            return Some(*spec);
        }
        phase -= spec.duration_frames;
    }
    specs.last().copied()
}

fn select_enemy_palette(
    visuals: &wildside_render::AnimatedSpriteVisual,
    variant: u8,
) -> &wildside_render::Palette {
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
    let variant = platform_snapshot_u8(object, "platform_render_flags")
        .and_then(PlatformRenderVariantLoc244C::from_render_flags);
    let art = platform_art_for_mapping_word_loc_2548(object, level_visuals).unwrap_or_else(|| {
        let frame = match variant {
            Some(PlatformRenderVariantLoc244C::Loc251C) => level_visuals.platforms.vertical.clone(),
            _ => match kind {
                0x00 => level_visuals.platforms.horizontal.clone(),
                _ => level_visuals.platforms.vertical.clone(),
            },
        };
        PlatformArtSourceLoc2548 {
            frame,
            palette: level_visuals.platforms.palette.clone(),
        }
    });
    let frame = &art.frame;
    let (screen_x, screen_y) = player_screen_position(object, trace);
    let width_px = platform_snapshot_dimension_px(object, "platform_width_px");
    let height_px = platform_snapshot_dimension_px(object, "platform_height_px");
    if !platform_visible_loc_244C(screen_x, screen_y, width_px, height_px) {
        return;
    }
    let width_cells = platform_snapshot_dimension_cells(object, "platform_width_px");
    let height_cells = platform_snapshot_dimension_cells(object, "platform_height_px");
    let tile_sheet = Arc::new(frame.tile_sheet.clone());
    let palette = Arc::new(art.palette);
    for cell in platform_sprite_cells_loc_24AC(variant, width_cells, height_cells) {
        commands.push(DrawCommand::Sprite {
            x: screen_x - i32::from(frame.origin_x) + i32::from(cell.column) * 16,
            y: screen_y - i32::from(frame.height_px)
                + i32::from(frame.origin_y)
                + cell.y_bias
                + i32::from(cell.row) * 16,
            tile_sheet: Arc::clone(&tile_sheet),
            palette: Arc::clone(&palette),
            tiles: platform_cell_tiles_loc_2548(frame, cell.mapping_tile_offset),
        });
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PlatformRenderVariantLoc244C {
    Loc24CC,
    Loc24F0,
    Loc251C,
    Loc24D4,
    Loc24C0,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct PlatformSpriteCellLoc24AC {
    column: u16,
    row: u16,
    y_bias: i32,
    mapping_tile_offset: u16,
}

impl PlatformRenderVariantLoc244C {
    fn from_render_flags(render_flags: u8) -> Option<Self> {
        // ASM loc_244C (kid.asm:4357-4364): retail clears bit 7 of `$1F(a2)`,
        // doubles the remaining dispatch byte, then indexes the long-pointer
        // `off_24AC` table. Valid shipped entries are even bytes 0, 2, 4, 6, 8.
        match render_flags & 0x7F {
            0 => Some(Self::Loc24CC),
            2 => Some(Self::Loc24F0),
            4 => Some(Self::Loc251C),
            6 => Some(Self::Loc24D4),
            8 => Some(Self::Loc24C0),
            _ => None,
        }
    }
}

#[allow(non_snake_case)]
fn platform_sprite_cells_loc_24AC(
    variant: Option<PlatformRenderVariantLoc244C>,
    width_cells: u16,
    height_cells: u16,
) -> Vec<PlatformSpriteCellLoc24AC> {
    match variant {
        Some(PlatformRenderVariantLoc244C::Loc24CC) => (0..width_cells)
            .map(|column| platform_cell(column, 0, -2, 0))
            .collect(),
        Some(PlatformRenderVariantLoc244C::Loc24F0) => {
            platform_horizontal_strip_cells_loc_24F0(width_cells)
        }
        Some(PlatformRenderVariantLoc244C::Loc251C) => {
            platform_vertical_strip_cells_loc_251C(height_cells)
        }
        Some(PlatformRenderVariantLoc244C::Loc24D4) | None => (0..height_cells)
            .flat_map(|row| (0..width_cells).map(move |column| platform_cell(column, row, 0, 0)))
            .collect(),
        Some(PlatformRenderVariantLoc244C::Loc24C0) => vec![platform_cell(0, 0, 0, 0)],
    }
}

fn platform_cell(
    column: u16,
    row: u16,
    y_bias: i32,
    mapping_tile_offset: u16,
) -> PlatformSpriteCellLoc24AC {
    PlatformSpriteCellLoc24AC {
        column,
        row,
        y_bias,
        mapping_tile_offset,
    }
}

#[allow(non_snake_case)]
fn platform_horizontal_strip_cells_loc_24F0(width_cells: u16) -> Vec<PlatformSpriteCellLoc24AC> {
    if width_cells == 0 {
        return Vec::new();
    }

    (0..width_cells)
        .map(|column| {
            // ASM loc_24F0 advances `a4` by 4 VRAM tiles after the first
            // sprite and again before the last sprite, selecting cap/middle
            // cells from the platform art block selected by `$18(a2)`.
            let mapping_tile_offset = match column {
                0 => 0,
                last if last == width_cells - 1 => 8,
                _ => 4,
            };
            platform_cell(column, 0, 0, mapping_tile_offset)
        })
        .collect()
}

#[allow(non_snake_case)]
fn platform_vertical_strip_cells_loc_251C(height_cells: u16) -> Vec<PlatformSpriteCellLoc24AC> {
    if height_cells == 0 {
        return Vec::new();
    }

    (0..height_cells)
        .map(|row| {
            // ASM loc_251C mirrors loc_24F0 on the y axis: top, middle, and
            // bottom platform cells use `a4`, `a4+4`, and `a4+8`.
            let mapping_tile_offset = match row {
                0 => 0,
                last if last == height_cells - 1 => 8,
                _ => 4,
            };
            platform_cell(0, row, 0, mapping_tile_offset)
        })
        .collect()
}

#[allow(non_snake_case)]
fn platform_cell_tiles_loc_2548(frame: &SpriteFrame, mapping_tile_offset: u16) -> Vec<SpriteTile> {
    let first_tile = frame.tile_base_index.saturating_add(mapping_tile_offset);
    vec![
        SpriteTile {
            offset_x: 0,
            offset_y: 0,
            tile: TileRef {
                tile_index: first_tile,
                palette_line: 0,
                priority: false,
                flip_x: false,
                flip_y: false,
            },
        },
        SpriteTile {
            offset_x: 0,
            offset_y: 8,
            tile: TileRef {
                tile_index: first_tile.saturating_add(1),
                palette_line: 0,
                priority: false,
                flip_x: false,
                flip_y: false,
            },
        },
        SpriteTile {
            offset_x: 8,
            offset_y: 0,
            tile: TileRef {
                tile_index: first_tile.saturating_add(2),
                palette_line: 0,
                priority: false,
                flip_x: false,
                flip_y: false,
            },
        },
        SpriteTile {
            offset_x: 8,
            offset_y: 8,
            tile: TileRef {
                tile_index: first_tile.saturating_add(3),
                palette_line: 0,
                priority: false,
                flip_x: false,
                flip_y: false,
            },
        },
    ]
}

#[allow(non_snake_case)]
fn platform_visible_loc_244C(screen_x: i32, screen_y: i32, width_px: u16, height_px: u16) -> bool {
    // ASM loc_244C (kid.asm:4333-4349): cull when the platform origin is beyond
    // the 320x224 viewport, or when the trailing edge is still negative.
    screen_x < 0x0140
        && screen_x + i32::from(width_px) >= 0
        && screen_y < 0x00E0
        && screen_y + i32::from(height_px) >= 0
}

fn platform_snapshot_dimension_px(object: &NativeObjectSnapshot, key: &str) -> u16 {
    object
        .debug_fields
        .get(key)
        .and_then(|value| value.parse::<u16>().ok())
        .unwrap_or(15)
}

fn platform_snapshot_dimension_cells(object: &NativeObjectSnapshot, key: &str) -> u16 {
    object
        .debug_fields
        .get(key)
        .and_then(|value| value.parse::<u16>().ok())
        .map(|pixels| pixels.saturating_add(1).div_ceil(16))
        .filter(|cells| *cells > 0)
        .unwrap_or(1)
}

fn platform_snapshot_u8(object: &NativeObjectSnapshot, key: &str) -> Option<u8> {
    object
        .debug_fields
        .get(key)
        .and_then(|value| value.parse::<u8>().ok())
}

#[allow(non_snake_case)]
struct PlatformArtSourceLoc2548 {
    frame: SpriteFrame,
    palette: Palette,
}

const PLATFORM_BLOCK_ART_VRAM_BASE_LOC_E0E4E: u16 = 0x0220;

#[allow(non_snake_case)]
fn platform_art_for_mapping_word_loc_2548(
    object: &NativeObjectSnapshot,
    level_visuals: &NativeLevelVisuals,
) -> Option<PlatformArtSourceLoc2548> {
    // ASM loc_244C/loc_2548 (kid.asm:4352,4452): `$18(a2)` is copied into the
    // sprite mapping word. `PlatformArtAddr_Table` at loc_2822 stores VRAM tile
    // indices; Initialize_Platforms loads Horizontal_platform.bin at byte
    // `$1280` (tile `$94`) and Vertical_platform.bin at byte `$D5C0`
    // (tile `$6AE`). `$25A` falls in the common block-art range starting at
    // `unk_E0E4E` / VRAM tile `$220`.
    match object
        .debug_fields
        .get("platform_mapping_addr_word")
        .and_then(|value| value.parse::<u16>().ok())?
    {
        0x0094 => Some(PlatformArtSourceLoc2548 {
            frame: level_visuals.platforms.horizontal.clone(),
            palette: level_visuals.platforms.palette.clone(),
        }),
        0x06AE => Some(PlatformArtSourceLoc2548 {
            frame: level_visuals.platforms.vertical.clone(),
            palette: level_visuals.platforms.palette.clone(),
        }),
        0x025A => Some(PlatformArtSourceLoc2548 {
            frame: SpriteFrame {
                origin_x: 0,
                origin_y: 0x10,
                width_px: 0x10,
                height_px: 0x10,
                tile_base_index: 0x025A - PLATFORM_BLOCK_ART_VRAM_BASE_LOC_E0E4E,
                tile_sheet: level_visuals.block_tiles.clone(),
            },
            palette: level_visuals.palettes.foreground.clone(),
        }),
        _ => None,
    }
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

#[cfg(test)]
mod tests {
    use super::*;

    fn synthetic_player_frame() -> wildside_render::SpriteFrame {
        wildside_render::SpriteFrame {
            origin_x: 8,
            origin_y: 0,
            width_px: 16,
            height_px: 16,
            tile_base_index: 0,
            tile_sheet: wildside_render::TileSheet {
                tiles: vec![
                    wildside_render::Tile { pixels: [0; 64] },
                    wildside_render::Tile { pixels: [0; 64] },
                    wildside_render::Tile { pixels: [0; 64] },
                    wildside_render::Tile { pixels: [0; 64] },
                ],
            },
        }
    }

    #[test]
    fn player_sprite_priority_stays_low_except_flagpole_override_loc_c070() {
        let frame = synthetic_player_frame();

        let ordinary_tiles = build_player_sprite_tiles_with_flip(&frame, false, None);
        assert!(
            ordinary_tiles.iter().all(|tile| !tile.tile.priority),
            "ASM player sprite descriptors do not encode a global high-priority bit; normal Kid priority stays low"
        );

        let flagpole_tiles = build_player_sprite_tiles_with_flip(&frame, false, Some(true));
        assert!(
            flagpole_tiles.iter().all(|tile| tile.tile.priority),
            "ASM CharacterCollision_TouchFlagpole loc_C070 is the narrow first-frame priority override"
        );
    }
}
