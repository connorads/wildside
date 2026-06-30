use super::*;

/// Archer fire interval indexed by enemy_level (HP 1/2/3).
/// From Archer.asm `arrow_accuracy`: $1E=30, $14=20, $0A=10 frames.
pub(crate) const ARCHER_FIRE_TIMER: [u16; 3] = [0x1E, 0x14, 0x0A];

pub(crate) const ARCHER_KILL_BOUNCE_Y_VELOCITY_LOC_3414A: i32 = -0x0004_0000;

/// Arrow velocity table from Archer.asm `byte_3437A`.
/// 28 entries: (x_direction_sign, sprite_id, x_vel_byte, y_vel_byte).
/// Velocity = signed_byte * 0x1000 (ext.w → swap → asr.l #4).
pub(crate) const ARROW_ANGLE_TABLE: [(i8, i8); 28] = [
    (0x20, 0),   // 0: right
    (0x20, -8),  // 1: right + slight up
    (0x1C, -16), // 2
    (0x18, -24), // 3
    (0x10, -28), // 4
    (8, -32),    // 5
    (0, -32),    // 6: straight up
    (0, -32),    // 7: straight up (mirrored)
    (-8, -32),   // 8
    (-16, -28),  // 9
    (-24, -24),  // 10
    (-28, -16),  // 11
    (-32, -8),   // 12
    (-32, 0),    // 13: left
    (-32, 0),    // 14: left (dup)
    (-32, 8),    // 15
    (-28, 16),   // 16
    (-24, 24),   // 17
    (-16, 28),   // 18
    (-8, 32),    // 19
    (0, 32),     // 20: straight down
    (0, 32),     // 21: straight down (mirrored)
    (8, 32),     // 22
    (16, 28),    // 23
    (24, 24),    // 24
    (28, 16),    // 25
    (32, 8),     // 26
    (32, 0),     // 27: right (wrap)
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ArcherShootingTableLoc3419A {
    Horizontal,
    Vertical,
}

#[allow(non_snake_case)]
pub(crate) fn archer_opening_angle_loc_3419A(
    table: ArcherShootingTableLoc3419A,
    facing: Direction,
) -> usize {
    // ASM Archer.asm loc_3419A: load the first table word as the default-left
    // opening angle; if `x_direction(a3)` is non-zero (flipped/right), replace
    // it with the second word before advancing to the speed/animation setup.
    let (left_angle, right_angle) = match table {
        ArcherShootingTableLoc3419A::Horizontal => (0x0D, 0x00),
        ArcherShootingTableLoc3419A::Vertical => (0x0A, 0x03),
    };

    match facing {
        Direction::Left => left_angle,
        Direction::Right => right_angle,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct ArcherShotSelectionLoc340C0 {
    pub(crate) initial_angle: usize,
    pub(crate) facing: Direction,
    pub(crate) uses_vertical_table: bool,
}

#[allow(non_snake_case)]
pub(crate) fn archer_shot_selection_loc_340C0(target_angle: usize) -> ArcherShotSelectionLoc340C0 {
    // ASM Archer.asm loc_3408C/loc_340C0: start on Shooting_Vertical with
    // x_direction clear, then `cmpi #6`, `cmpi #$D`, and `cmpi #$14` divide the
    // target angle into vertical-right, vertical-left, horizontal-left, and
    // horizontal-right opening poses.  The following table reader then converts
    // the pose table's fixed opening angle, not the exact target angle; later
    // arrow logic re-aims.
    let (facing, table) = match target_angle {
        0..=5 => (Direction::Right, ArcherShootingTableLoc3419A::Vertical),
        6..=13 => (Direction::Left, ArcherShootingTableLoc3419A::Vertical),
        14..=20 => (Direction::Left, ArcherShootingTableLoc3419A::Horizontal),
        _ => (Direction::Right, ArcherShootingTableLoc3419A::Horizontal),
    };

    ArcherShotSelectionLoc340C0 {
        initial_angle: archer_opening_angle_loc_3419A(table, facing),
        facing,
        uses_vertical_table: table == ArcherShootingTableLoc3419A::Vertical,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct ArcherArrowSetupLoc341AA {
    pub(crate) x_velocity: i32,
    pub(crate) y_velocity: i32,
    pub(crate) held_arrow_facing: Direction,
}

#[allow(non_snake_case)]
pub(crate) fn archer_arrow_setup_loc_341AA(
    opening_angle: usize,
    archer_facing: Direction,
) -> ArcherArrowSetupLoc341AA {
    // ASM Archer.asm loc_341AA: after skipping the second angle word, run
    // Enemy07_ArcherArrow_AngleToSpeedAndAnim for the held-arrow GfxObject,
    // then copy `x_direction(a3)` to `$16(a1)` and `not.b` it.
    let (x_velocity, y_velocity) = arrow_velocity_from_angle(opening_angle);
    let held_arrow_facing = match archer_facing {
        Direction::Left => Direction::Right,
        Direction::Right => Direction::Left,
    };

    ArcherArrowSetupLoc341AA {
        x_velocity,
        y_velocity,
        held_arrow_facing,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ArcherShootingEntryLoc341BE {
    RestartCycle,
    PoseFrame {
        frame_timer: u16,
    },
    FireArrow {
        hold_frame_timer: u16,
        held_arrow_reset_sprite_offset: u16,
        arrow_count_increment: u16,
        held_arrow_position_nudge_px: i16,
    },
}

#[allow(non_snake_case)]
pub(crate) fn archer_shooting_entry_loc_341BE(
    table_frame_delay: i16,
) -> ArcherShootingEntryLoc341BE {
    // ASM Archer.asm loc_341BE: read the next shooting-table delay word into
    // d3. Negative restarts the archer cycle, non-zero routes to the pose-frame
    // path, and zero is the arrow-fire sentinel with fixed held-arrow effects.
    match table_frame_delay {
        delay if delay < 0 => ArcherShootingEntryLoc341BE::RestartCycle,
        delay if delay > 0 => ArcherShootingEntryLoc341BE::PoseFrame {
            frame_timer: delay.unsigned_abs(),
        },
        _ => ArcherShootingEntryLoc341BE::FireArrow {
            hold_frame_timer: 0x14,
            held_arrow_reset_sprite_offset: 0,
            arrow_count_increment: 1,
            held_arrow_position_nudge_px: 3,
        },
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct ArcherArrowDeleteLoc34336 {
    pub(crate) number_of_arrows: u16,
    pub(crate) deletes_current_object: bool,
}

#[allow(non_snake_case)]
pub(crate) fn archer_arrow_delete_loc_34336(number_of_arrows: u16) -> ArcherArrowDeleteLoc34336 {
    // ASM Archer.asm loc_34336: `subq.w #1,(Number_of_Arrows).w`, then jump to
    // j_Delete_CurrentObject. Native stores the same live-arrow count as the
    // projectile Vec length.
    ArcherArrowDeleteLoc34336 {
        number_of_arrows: number_of_arrows.wrapping_sub(1),
        deletes_current_object: true,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct ArcherPoseFrameLoc34222 {
    pub(crate) held_arrow_sprite_offset: u16,
    pub(crate) x_offset_px: i16,
    pub(crate) y_offset_px: i16,
}

#[allow(non_snake_case)]
pub(crate) fn archer_pose_frame_loc_34222(
    held_arrow_sprite_offset: u16,
    table_x_offset_px: i16,
    table_y_offset_px: i16,
    archer_facing: Direction,
) -> ArcherPoseFrameLoc34222 {
    // ASM Archer.asm loc_34222: consume the pose sprite plus d6/d7 offsets from
    // the shooting table, then negate only d6 when `x_direction(a3)` is set.
    let x_offset_px = match archer_facing {
        Direction::Left => table_x_offset_px,
        Direction::Right => table_x_offset_px.wrapping_neg(),
    };

    ArcherPoseFrameLoc34222 {
        held_arrow_sprite_offset,
        x_offset_px,
        y_offset_px: table_y_offset_px,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct ArcherHeldArrowPositionLoc34234 {
    pub(crate) x_pos_word: u16,
    pub(crate) y_pos_word: u16,
}

#[allow(non_snake_case)]
pub(crate) fn archer_held_arrow_position_loc_34234(
    archer_x_pos: u32,
    archer_y_pos: u32,
    x_offset_px: i16,
    y_offset_px: i16,
) -> ArcherHeldArrowPositionLoc34234 {
    // ASM Archer.asm loc_34234: copy only the integer word of x_pos/y_pos into
    // the held-arrow object, then add d6/d7 with 16-bit word arithmetic.
    ArcherHeldArrowPositionLoc34234 {
        x_pos_word: fp16_integer_word(archer_x_pos).wrapping_add_signed(x_offset_px),
        y_pos_word: fp16_integer_word(archer_y_pos).wrapping_add_signed(y_offset_px),
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ArcherDeathStatusRouteLoc3414A {
    WriteRespawnMarker(ArcherRespawnMarkerLoc34188),
    SubtractSpawnCounter { index: usize, decrement: u16 },
    DeleteOnly,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct ArcherRespawnMarkerLoc34188 {
    pub(crate) index: usize,
    pub(crate) word_value: u16,
}

#[allow(non_snake_case)]
pub(crate) fn archer_respawn_marker_loc_34188(
    object_status_slot: u8,
) -> ArcherRespawnMarkerLoc34188 {
    // ASM Archer.asm loc_34188: positive `$42(a5)` is doubled as an
    // EnemyStatus_Table word offset, then `$2168` is written as the respawn
    // marker before falling into the delete tail.
    ArcherRespawnMarkerLoc34188 {
        index: usize::from(object_status_slot),
        word_value: 0x2168,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct ArcherDeathSequenceLoc3414A {
    pub(crate) has_kid_collision: bool,
    pub(crate) kid_y_velocity: i32,
    pub(crate) status_route: ArcherDeathStatusRouteLoc3414A,
    pub(crate) deletes_current_object: bool,
}

#[allow(non_snake_case)]
pub(crate) fn archer_death_sequence_loc_3414A(
    object_status_slot: u8,
) -> ArcherDeathSequenceLoc3414A {
    // ASM Archer.asm loc_3414A: set `has_kid_collision(a3)`, play the death
    // animation, force Kid's y velocity to `$FFFC0000`, then route `$42(a5)`
    // before ASM Archer.asm loc_34196 deletes the current object.
    let status_route = if object_status_slot & 0x80 == 0 {
        ArcherDeathStatusRouteLoc3414A::WriteRespawnMarker(archer_respawn_marker_loc_34188(
            object_status_slot,
        ))
    } else if object_status_slot & 0x40 != 0 {
        ArcherDeathStatusRouteLoc3414A::SubtractSpawnCounter {
            index: usize::from(object_status_slot & 0x3F),
            decrement: 0x0400,
        }
    } else {
        ArcherDeathStatusRouteLoc3414A::DeleteOnly
    };

    ArcherDeathSequenceLoc3414A {
        has_kid_collision: true,
        kid_y_velocity: ARCHER_KILL_BOUNCE_Y_VELOCITY_LOC_3414A,
        status_route,
        deletes_current_object: true,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct ArcherAngleLoc34458 {
    pub(crate) angle: usize,
    pub(crate) quadrant_selector: u8,
}

/// Compute arrow angle index (0..27) from archer to player.
/// Faithfully mirrors Archer.asm `loc_34458`: quadrant dispatch + unk_34404 ratio lookup.
pub(crate) fn archer_angle_to_player(
    archer_x: u32,
    archer_y: u32,
    player_x: u32,
    player_y: u32,
) -> usize {
    archer_angle_loc_34458(archer_x, archer_y, player_x, player_y).angle
}

pub(crate) fn archer_angle_loc_34458(
    archer_x: u32,
    archer_y: u32,
    player_x: u32,
    player_y: u32,
) -> ArcherAngleLoc34458 {
    let dx = (player_x >> 16) as i32 - (archer_x >> 16) as i32;
    // Archer aims 16px above player centre (subi.w #$10,d7 in ASM).
    let dy = ((player_y >> 16) as i32 - 0x10) - (archer_y >> 16) as i32;

    // Ratio lookup table from unk_34404 (84 entries). Maps (abs_dy*8/abs_dx) to sub-angle 0-5.
    const RATIO_TABLE: [u8; 84] = [
        0, 1, 1, 2, 2, 2, 3, 3, 3, 3, 3, 3, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 5, 5, 5, 5, 5, 5,
        5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5,
        5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5,
    ];

    if dx == 0 {
        // ORIGINAL_QUIRK: ASM loc_34530 sets d7=0 then tests d7 (not d4/dy), so
        // bmi never fires and this always returns angle 21 (straight down).
        return ArcherAngleLoc34458 {
            angle: 0x15,
            quadrant_selector: 3,
        };
    }

    let dx_is_right = dx.is_positive();
    let dy_is_down = !dy.is_negative();
    let quadrant_selector = match (dx_is_right, dy_is_down) {
        (true, false) => 0,
        (false, false) => 1,
        (false, true) => 2,
        (true, true) => 3,
    };

    let abs_dx = dx.unsigned_abs();
    let abs_dy = dy.unsigned_abs();
    let ratio = (abs_dy * 8) / abs_dx;

    // ASM loc_34546: ratio >= $3F → nearly vertical, return 6 (up) or 21 (down).
    if ratio >= 0x3F {
        return ArcherAngleLoc34458 {
            angle: match dy.signum() {
                -1 => 6,
                _ => 0x15,
            },
            // ASM Archer.asm loc_3452C: overflow paths keep the d5 quadrant
            // selector and return it in d7 alongside the vertical angle in d6.
            quadrant_selector,
        };
    }

    let sub = RATIO_TABLE[ratio as usize] as i32;

    let angle = match (dx_is_right, dy_is_down) {
        // ASM Archer.asm loc_344C6: right+up keeps the ratio-table sub-angle.
        (true, false) => sub as usize,
        // ASM Archer.asm loc_34492: left+up negates both deltas, looks up the
        // slope sub-angle, then returns `$D - sub`.
        (false, false) => (0x0D - sub) as usize,
        // ASM Archer.asm loc_344E6: left+down adds `$E` to the ratio-table sub-angle.
        (false, true) => (0x0E + sub) as usize,
        // ASM Archer.asm loc_3450A: right+down subtracts the ratio-table sub-angle from `$1B`.
        (true, true) => (0x1B - sub) as usize,
    };

    // ASM Archer.asm loc_3452C: ratio paths return the computed d5 quadrant
    // selector in d7. Native callers only need the d6 angle today, but keeping
    // the selector explicit documents the ROM contract and protects future ports.
    ArcherAngleLoc34458 {
        angle,
        quadrant_selector,
    }
}

/// Archer arrow spawn X offset (Archer.asm Shooting_Horizontal/Vertical 3rd frame: $FFF8 = -8).
const ARCHER_ARROW_X_OFFSET: i32 = -8;
/// Archer arrow spawn Y offset for horizontal shooting (Archer.asm: $FFEF = -17).
const ARCHER_ARROW_Y_OFFSET_HORIZONTAL: i32 = -17;
/// Archer arrow spawn Y offset for vertical shooting (Archer.asm: $FFE8 = -24).
const ARCHER_ARROW_Y_OFFSET_VERTICAL: i32 = -24;

/// Adjust arrow angle by ±1 toward target on a 28-value circular ring.
/// ASM loc_34308/loc_34320: if shortest arc ≤ 14 steps, move direct; otherwise wrap.
pub(crate) fn archer_adjust_angle(current: u8, target: u8) -> u8 {
    if current == target {
        return current;
    }
    let delta = target as i16 - current as i16; // -27..27
    let threshold: i16 = match target.cmp(&current) {
        std::cmp::Ordering::Greater => 14,
        std::cmp::Ordering::Less => -14,
        std::cmp::Ordering::Equal => unreachable!("equal angles returned above"),
    };
    if delta >= threshold {
        // Decrement path (ASM loc_34320)
        if current == 0 { 27 } else { current - 1 }
    } else {
        // Increment path (ASM loc_34308 fall-through)
        if current >= 27 { 0 } else { current + 1 }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct ArcherReaimLoc3432E {
    pub(crate) current_angle: u8,
    pub(crate) x_velocity: i32,
    pub(crate) y_velocity: i32,
}

#[allow(non_snake_case)]
pub(crate) fn archer_reaim_loc_3432E(current_angle: u8, target_angle: u8) -> ArcherReaimLoc3432E {
    // ASM Archer.asm loc_3432E: after loc_34308/loc_34320 adjust d2 by one
    // circular angle step, copy d2 to d6 and rerun
    // Enemy07_ArcherArrow_AngleToSpeedAndAnim for the arrow object.
    let current_angle = archer_adjust_angle(current_angle, target_angle);
    let (x_velocity, y_velocity) = arrow_velocity_from_angle(usize::from(current_angle));
    ArcherReaimLoc3432E {
        current_angle,
        x_velocity,
        y_velocity,
    }
}

/// Convert an angle index (0..27) to (x_velocity, y_velocity) via ARROW_ANGLE_TABLE.
/// ASM Enemy07_ArcherArrow_AngleToSpeedAndAnim: byte * 0x1000.
pub(crate) fn arrow_velocity_from_angle(angle: usize) -> (i32, i32) {
    let (x_byte, y_byte) = ARROW_ANGLE_TABLE[angle.min(ARROW_ANGLE_TABLE.len() - 1)];
    ((x_byte as i32) * 0x1000, (y_byte as i32) * 0x1000)
}

fn archer_live_facing_loc_3408c(
    enemy_x: u32,
    enemy_y: u32,
    player_x: Option<u32>,
    player_y: Option<u32>,
) -> Option<Direction> {
    let (Some(px), Some(py)) = (player_x, player_y) else {
        return None;
    };

    // ASM Archer.asm loc_3408C temporarily subtracts `$1E` from y_pos(a3)
    // before calling loc_34458, then uses loc_340C0's sector branch to set
    // x_direction for the whole shooting table cycle.
    let aim_origin_y = enemy_y.wrapping_sub(0x001E_0000);
    let target_angle = archer_angle_to_player(enemy_x, aim_origin_y, px, py);
    Some(archer_shot_selection_loc_340C0(target_angle).facing)
}

/// Step Archer: stationary enemy that faces player and fires arrows at intervals.
/// ASM: arrows persist until terrain/object collision, re-aiming every `arrow_accuracy`
/// frames by ±1 angle step toward the player (loc_342D6 loop).
pub(crate) fn step_archer(
    enemy: &mut ActiveEnemy,
    projectiles: &mut Vec<EnemyProjectile>,
    player_x_pos: Option<u32>,
    player_y_pos: Option<u32>,
    played_sound_effects: Option<&mut Vec<QueuedSoundEffect>>,
) {
    if let Some(dir) =
        archer_live_facing_loc_3408c(enemy.x_pos, enemy.y_pos, player_x_pos, player_y_pos)
    {
        enemy.facing = dir;
    }

    // Count down fire timer.
    if enemy.wait_timer > 0 {
        enemy.wait_timer -= 1;
        return;
    }

    // Reset timer for the next shooting-table cycle. `arrow_accuracy` below is
    // only the fired arrow's re-aim cadence, not the archer's shot cadence.
    let level = enemy.hit_points.saturating_sub(1).min(2) as usize;
    enemy.wait_timer = ARCHER_SHOOTING_CYCLE_DELAY_FRAMES_LOC_341BE;

    // Need player position to compute angle.
    let (Some(px), Some(py)) = (player_x_pos, player_y_pos) else {
        return;
    };

    // ASM loc_34248: `cmpi.w #2,(Number_of_Arrows).w; bge.w loc_3408C`.
    // Native represents `Number_of_Arrows` as live ArcherArrow projectiles,
    // not the whole shared projectile pool.
    let archer_arrow_count = projectiles
        .iter()
        .filter(|projectile| projectile.kind == EnemyProjectileKind::ArcherArrow)
        .count() as u16;
    if archer_arrow_count >= 2 {
        return;
    }
    if projectiles.len() >= MAX_PROJECTILES {
        return;
    }

    // Compute initial angle and velocity.
    let aim_origin_y = enemy.y_pos.wrapping_sub(0x001E_0000);
    let target_angle = archer_angle_to_player(enemy.x_pos, aim_origin_y, px, py);
    let shot_selection = archer_shot_selection_loc_340C0(target_angle);
    enemy.facing = shot_selection.facing;
    let angle = shot_selection.initial_angle;
    let arrow_setup = archer_arrow_setup_loc_341AA(angle, enemy.facing);
    let reaim = ARCHER_FIRE_TIMER[level];

    // Spawn offset from Archer.asm 3rd animation frame.
    // Vertical shooting table: Y=-24. Horizontal table: Y=-17.
    // X offset negated when facing right (ASM loc_34216).
    let y_offset = if shot_selection.uses_vertical_table {
        ARCHER_ARROW_Y_OFFSET_VERTICAL
    } else {
        ARCHER_ARROW_Y_OFFSET_HORIZONTAL
    };
    let x_offset = if enemy.facing == Direction::Right {
        -ARCHER_ARROW_X_OFFSET
    } else {
        ARCHER_ARROW_X_OFFSET
    };
    let (hold_frame_timer, held_arrow_reset_sprite_offset, arrow_count_increment) =
        match archer_shooting_entry_loc_341BE(0) {
            ArcherShootingEntryLoc341BE::FireArrow {
                hold_frame_timer,
                held_arrow_reset_sprite_offset,
                arrow_count_increment,
                held_arrow_position_nudge_px: _,
            } => (
                hold_frame_timer,
                held_arrow_reset_sprite_offset,
                arrow_count_increment,
            ),
            ArcherShootingEntryLoc341BE::RestartCycle
            | ArcherShootingEntryLoc341BE::PoseFrame { .. } => {
                unreachable!("ASM Archer.asm loc_341BE zero delay is the fire sentinel")
            }
        };
    debug_assert_eq!(hold_frame_timer, 0x14);
    debug_assert_eq!(arrow_count_increment, 1);

    // Native collapses the visible held-arrow pose table into the final spawn
    // boundary, but the same ASM loc_34222/loc_34234 word-offset contract still
    // guards the fire-frame position used below.
    let pose_frame = archer_pose_frame_loc_34222(
        held_arrow_reset_sprite_offset,
        ARCHER_ARROW_X_OFFSET as i16,
        y_offset as i16,
        enemy.facing,
    );
    debug_assert_eq!(pose_frame.held_arrow_sprite_offset, 0);
    debug_assert_eq!(i32::from(pose_frame.x_offset_px), x_offset);
    debug_assert_eq!(i32::from(pose_frame.y_offset_px), y_offset);
    let held_arrow_position = archer_held_arrow_position_loc_34234(
        enemy.x_pos,
        enemy.y_pos,
        pose_frame.x_offset_px,
        pose_frame.y_offset_px,
    );
    let spawn_x = (enemy.x_pos as i64 + ((x_offset as i64) << 16)) as u32;
    let spawn_y = (enemy.y_pos as i64 + ((y_offset as i64) << 16)) as u32;
    debug_assert_eq!(held_arrow_position.x_pos_word, fp16_integer_word(spawn_x));
    debug_assert_eq!(held_arrow_position.y_pos_word, fp16_integer_word(spawn_y));

    projectiles.push(EnemyProjectile {
        x_pos: spawn_x,
        y_pos: spawn_y,
        x_velocity: arrow_setup.x_velocity,
        y_velocity: arrow_setup.y_velocity,
        // ASM arrows live until terrain collision, not a fixed timer.
        // Use large lifetime as fallback; terrain_kill handles normal removal.
        lifetime: u16::MAX,
        friendly: false,
        gravity: 0,
        homing_accel: 0,
        speed_cap: 0,
        wall_bounce: false,
        lion_projectile_bounce_empty_checks_loc_3e2c0: 0,
        reaim_interval: reaim,
        reaim_countdown: reaim,
        current_angle: angle as u8,
        terrain_kill: true,
        drips_spawn_crawler_on_splash: false,
        owner: None,
        kind: EnemyProjectileKind::ArcherArrow,
        palette_variant: 0,
        shooter_tracks_layout: false,
        shooter_layout_col: 0,
        shooter_layout_row: 0,
        shooter_state_word: 0,
        shooter_skip_projectile_step_frame: None,
        animation_elapsed_frames: 0,
        boss_head_boomerang_direction_byte_d0: 0,
        drips_surface_crawler_state: DripsSurfaceCrawlerState::HorizontalLoc33650,
        drips_surface_crawler_x_direction_flipped: false,
        ufo_beam_child_x_direction: 0,
        ufo_beam_child_loop_timer_loc_3554a: 0,
        drips_child_byte_44: 0,
        drips_child_byte_45: 0,
        tar_monster_collision_response_budget: 0,
        dragon_flying_descent_fire_state: DragonFlyingDescentFireState::None,
        dragon_flying_descent_fire_counter: 0,
        boss_death_burst_state: BossDeathBurstState::None,
        collision_type: 0,
        boss_eye_sprite_mapping_label: "",
        boss_eye_pop_effect_x_direction: 0,
    });
    debug_assert_eq!(
        projectiles
            .iter()
            .filter(|projectile| projectile.kind == EnemyProjectileKind::ArcherArrow)
            .count() as u16,
        archer_arrow_count.wrapping_add(arrow_count_increment)
    );
    if let Some(sound_effects) = played_sound_effects {
        // ASM Archer.asm loc_34266: `Enemy07_ArcherArrow_Init` plays
        // `sfx_Archer_shoots` before loading the arrow object.
        sound_effects.push(QueuedSoundEffect::ArcherShoots);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::enemies::test_fixtures::*;

    #[test]
    fn archer_kill_bounce_loc_3414a_uses_negative_rom_velocity() {
        assert_eq!(
            ARCHER_KILL_BOUNCE_Y_VELOCITY_LOC_3414A, -0x0004_0000,
            "ASM Archer.asm loc_3414A writes `$FFFC0000` to Kid y_vel"
        );
        assert_eq!(
            archer_death_sequence_loc_3414A(0x80).kid_y_velocity,
            -0x0004_0000,
            "ASM Archer.asm loc_3414A preserves the upward death-bounce sign"
        );
    }

    #[test]
    fn archer_byte_3437a_velocity_table_is_exact() {
        assert_eq!(
            ARROW_ANGLE_TABLE,
            [
                (0x20, 0),
                (0x20, -8),
                (0x1C, -16),
                (0x18, -24),
                (0x10, -28),
                (8, -32),
                (0, -32),
                (0, -32),
                (-8, -32),
                (-16, -28),
                (-24, -24),
                (-28, -16),
                (-32, -8),
                (-32, 0),
                (-32, 0),
                (-32, 8),
                (-28, 16),
                (-24, 24),
                (-16, 28),
                (-8, 32),
                (0, 32),
                (0, 32),
                (8, 32),
                (16, 28),
                (24, 24),
                (28, 16),
                (32, 8),
                (32, 0),
            ],
            "ASM Archer.asm byte_3437A stores signed x/y velocity bytes for all 28 arrow angles"
        );
    }

    #[test]
    fn archer_shot_selection_loc_340c0_clamps_out_of_range_to_right_horizontal_pose() {
        assert_eq!(
            archer_shot_selection_loc_340C0(usize::MAX),
            ArcherShotSelectionLoc340C0 {
                initial_angle: 0,
                facing: Direction::Right,
                uses_vertical_table: false,
            },
            "ASM Archer.asm loc_340C0 only receives 0..27 from loc_34458; native out-of-range callers stay in the right-horizontal catch-all"
        );
    }

    #[test]
    fn archer_angle_loc_34458_pins_ratio_edges_and_quadrants() {
        assert_eq!(
            archer_angle_loc_34458(100 << 16, 100 << 16, 36 << 16, 92 << 16),
            ArcherAngleLoc34458 {
                angle: 0x0B,
                quadrant_selector: 1,
            },
            "ASM Archer.asm loc_34492 computes `0x0D - sub` for the left/up ratio path"
        );
        assert_eq!(
            archer_angle_loc_34458(100 << 16, 100 << 16, 164 << 16, 140 << 16),
            ArcherAngleLoc34458 {
                angle: 0x19,
                quadrant_selector: 3,
            },
            "ASM Archer.asm loc_3450A computes `0x1B - sub` for the right/down ratio path"
        );
        assert_eq!(
            archer_angle_loc_34458(100 << 16, 100 << 16, 101 << 16, 22 << 16),
            ArcherAngleLoc34458 {
                angle: 6,
                quadrant_selector: 0,
            },
            "ASM Archer.asm loc_34546 keeps the up-vertical overflow angle when d4 is negative"
        );
        assert_eq!(
            archer_angle_loc_34458(100 << 16, 100 << 16, 101 << 16, 179 << 16),
            ArcherAngleLoc34458 {
                angle: 0x15,
                quadrant_selector: 3,
            },
            "ASM Archer.asm loc_34546 switches to down-vertical overflow when d4 is non-negative"
        );
    }

    #[test]
    fn archer_adjust_angle_keeps_half_ring_boundary_on_decrement_path() {
        assert_eq!(
            archer_adjust_angle(5, 19),
            4,
            "ASM Archer.asm loc_34308 uses `bge loc_34320`, so +14 takes the decrement/wrap-shortcut path"
        );
        assert_eq!(
            archer_adjust_angle(19, 5),
            18,
            "ASM Archer.asm loc_34320 also handles the exact -14 half-ring boundary by decrementing"
        );
    }

    #[test]
    fn archer_arrow_velocity_from_angle_clamps_to_last_rom_entry() {
        assert_eq!(
            arrow_velocity_from_angle(usize::MAX),
            (32 * 0x1000, 0),
            "ASM Archer.asm byte_3437A has 28 entries; native clamps defensive out-of-range angles to the duplicated final right-facing entry"
        );
    }

    #[test]
    fn archer_live_step_decrements_wait_timer_without_firing() {
        let mut archer = ActiveEnemy {
            family: EnemyFamily::Archer,
            wait_timer: 2,
            facing: Direction::Left,
            hit_points: 1,
            x_pos: 100 << 16,
            y_pos: 100 << 16,
            ..mini_hopping_skull_test_enemy()
        };
        let mut projectiles = Vec::new();

        step_archer(
            &mut archer,
            &mut projectiles,
            Some(200 << 16),
            Some(100 << 16),
            None,
        );

        assert_eq!(
            archer.wait_timer, 1,
            "ASM Archer.asm loc_34248 DBF countdown is modelled as a one-frame decrement before returning"
        );
        assert!(
            projectiles.is_empty(),
            "ASM Archer.asm loc_34248 should not reach the fire sentinel while the countdown remains non-zero"
        );
    }

    #[test]
    fn archer_live_step_countdown_uses_shot_sector_facing_for_same_x_player() {
        let mut archer = ActiveEnemy {
            family: EnemyFamily::Archer,
            wait_timer: 2,
            facing: Direction::Left,
            hit_points: 1,
            x_pos: 100 << 16,
            y_pos: 100 << 16,
            ..mini_hopping_skull_test_enemy()
        };
        let mut projectiles = Vec::new();

        step_archer(
            &mut archer,
            &mut projectiles,
            Some(100 << 16),
            Some(146 << 16),
            None,
        );

        assert_eq!(
            archer.facing,
            Direction::Right,
            "ASM Archer.asm loc_34458 returns angle 21 when dx is zero; loc_340C0 sets x_direction for the right-horizontal shooting table during countdown"
        );
        assert_eq!(archer.wait_timer, 1);
        assert!(
            projectiles.is_empty(),
            "ASM Archer.asm loc_34248 still only decrements the active shooting-table countdown"
        );
    }

    #[test]
    fn archer_live_step_uses_vertical_table_spawn_offset() {
        let mut archer = ActiveEnemy {
            family: EnemyFamily::Archer,
            wait_timer: 0,
            facing: Direction::Left,
            hit_points: 1,
            x_pos: 100 << 16,
            y_pos: 100 << 16,
            ..mini_hopping_skull_test_enemy()
        };
        let mut projectiles = Vec::new();

        step_archer(
            &mut archer,
            &mut projectiles,
            Some(36 << 16),
            Some(62 << 16),
            None,
        );

        assert_eq!(projectiles.len(), 1);
        assert_eq!(
            (projectiles[0].x_pos >> 16, projectiles[0].y_pos >> 16),
            (92, 76),
            "ASM Archer.asm Shooting_Vertical third fire frame uses x offset `$FFF8` and y offset `$FFE8`"
        );
        assert_eq!(
            projectiles[0].current_angle, 0x0A,
            "ASM Archer.asm loc_3419A selects the left-facing vertical opening angle"
        );
    }

    #[test]
    fn archer_live_step_queues_shoot_sound_when_arrow_spawns() {
        let archer = ActiveEnemy {
            family: EnemyFamily::Archer,
            wait_timer: 0,
            facing: Direction::Left,
            hit_points: 1,
            x_pos: 100 << 16,
            y_pos: 100 << 16,
            ..mini_hopping_skull_test_enemy()
        };
        let mut enemies = [archer];
        let mut rng = RomRandomState::default();
        let mut drilled_blocks = Vec::new();
        let mut projectiles = Vec::new();
        let mut sound_effects = Vec::new();

        step_active_enemies_with_rng(
            &mut enemies,
            None,
            Some(200 << 16),
            Some(100 << 16),
            None,
            None,
            None,
            None,
            None,
            PlayerForm::TheKid,
            0,
            false,
            Some(&mut projectiles),
            &mut rng,
            &mut drilled_blocks,
            Some(&mut sound_effects),
        );

        assert_eq!(
            projectiles.len(),
            1,
            "ASM Archer.asm loc_34266 should load the arrow object when the shooting timer expires",
        );
        assert_eq!(
            projectiles[0].kind,
            EnemyProjectileKind::ArcherArrow,
            "ASM Archer.asm loc_34248/loc_34266 should spawn an Archer arrow before playing the shot cue",
        );
        assert_eq!(
            sound_effects,
            vec![QueuedSoundEffect::ArcherShoots],
            "ASM Archer.asm loc_34266 should play `sfx_Archer_shoots` before loading the arrow object",
        );
        assert_eq!(sound_effects[0].sound_id(), 0x59);
    }
}
