use super::*;

/// Hand detection ranges by HP tier (Hand.asm unk_3276A table).
/// HP 1 = 64px, HP 2 = 80px, HP 3+ = 100px.
pub(crate) const HAND_DETECT_RANGE: [i32; 3] = [0x40, 0x50, 0x64];

/// Hand walk velocities by HP tier (Hand.asm unk_3276A table).
pub(crate) const HAND_WALK_SPEED: [i32; 3] = [0x3000, 0x4800, 0x6000];

/// Hand jump-attack gravity (Hand.asm line 130: $4000 per frame).
pub(crate) const HAND_GRAVITY: i32 = 0x4000;

#[allow(non_snake_case)]
pub(crate) fn hand_jump_seed_loc_328a6(hit_points: u8) -> i16 {
    // ASM Hand.asm loc_328A6: `move.w $46(a5),d5` loads the table's
    // HP-derived attack-speed seed: HP 1 => 1, HP 2 => 2, HP 3+ => 3.
    i16::from(hit_points.clamp(1, 3))
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub(crate) struct HandJumpComponentsLoc328AA {
    pub(crate) y_velocity: i32,
    pub(crate) x_velocity_word: i16,
}

fn hand_jump_units_from_random_loc_328aa(
    speed_seed: i16,
    random_byte: u8,
    kid_position_word: u16,
) -> i16 {
    let random_component = i16::from((random_byte ^ kid_position_word.to_le_bytes()[0]) & 3);
    speed_seed.wrapping_add(random_component)
}

#[allow(non_snake_case)]
pub(crate) fn hand_jump_components_loc_328aa(
    speed_seed: i16,
    x_direction_flag: bool,
    random_y_byte: u8,
    kid_y_position_word: u16,
    random_x_byte: u8,
    kid_x_position_word: u16,
) -> HandJumpComponentsLoc328AA {
    // ASM Hand.asm loc_328AA: each axis uses `(random_byte ^ KidPosLo) & 3`,
    // adds d5 from loc_328A6, stores negative Y, then negates X only when
    // raw x_direction is clear before falling through to the X store.
    let y_units =
        hand_jump_units_from_random_loc_328aa(speed_seed, random_y_byte, kid_y_position_word);
    let x_units =
        hand_jump_units_from_random_loc_328aa(speed_seed, random_x_byte, kid_x_position_word);
    let x_velocity_word = if x_direction_flag {
        x_units
    } else {
        x_units.wrapping_neg()
    };

    HandJumpComponentsLoc328AA {
        y_velocity: i32::from(y_units.wrapping_neg()) << 16,
        x_velocity_word,
    }
}

#[allow(non_snake_case)]
pub(crate) fn hand_x_velocity_loc_328e0(x_velocity_word: i16) -> i32 {
    // ASM Hand.asm loc_328E0: `swap d7; move.l d7,x_vel(a3)` promotes the
    // signed X word prepared by loc_328AA into the high word of 16.16 velocity.
    i32::from(x_velocity_word) << 16
}

#[allow(non_snake_case)]
pub(crate) fn hand_jump_loop_gravity_loc_328f0(y_velocity: i32) -> i32 {
    // ASM Hand.asm loc_328F0: after hibernation/range/collision checks,
    // `addi.l #$4000,y_vel(a3)` applies uncapped long-word gravity.
    y_velocity.wrapping_add(HAND_GRAVITY)
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub(crate) enum HandGroundCollisionBehaviorLoc3299C {
    Reverse,
    KidBelowContact,
    Destroy,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub(crate) struct HandGroundCollisionDispatchLoc3299C {
    pub(crate) collision_type: u16,
    pub(crate) behavior: Option<HandGroundCollisionBehaviorLoc3299C>,
}

#[allow(non_snake_case)]
pub(crate) fn hand_ground_collision_dispatch_loc_3299c(
    collision_type: i16,
) -> HandGroundCollisionDispatchLoc3299C {
    // ASM Hand.asm loc_3299C: negative collision types branch to the shared
    // destroy path before the clear; zero returns; positive IDs clear
    // `collision_type(a3)` before dispatching through `off_329B6`.
    if collision_type < 0 {
        return HandGroundCollisionDispatchLoc3299C {
            collision_type: collision_type as u16,
            behavior: Some(HandGroundCollisionBehaviorLoc3299C::Destroy),
        };
    }

    if collision_type == 0 {
        return HandGroundCollisionDispatchLoc3299C {
            collision_type: 0,
            behavior: None,
        };
    }

    let behavior = match collision_type as u16 {
        0x0004 | 0x0008 | 0x0014 | 0x0018 => Some(HandGroundCollisionBehaviorLoc3299C::Reverse),
        0x000C | 0x0010 | 0x001C | 0x002C => Some(HandGroundCollisionBehaviorLoc3299C::Destroy),
        0x0020 | 0x0024 | 0x0028 => Some(HandGroundCollisionBehaviorLoc3299C::KidBelowContact),
        _ => None,
    };

    HandGroundCollisionDispatchLoc3299C {
        collision_type: 0,
        behavior,
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub(crate) enum HandJumpCollisionBehaviorLoc329F8 {
    ClearXVelocity,
    ResetToPatrol,
    ClearYVelocity,
    BounceUpLeft,
    BounceUpRight,
    ReturnOnly,
    TryGrabKid,
    Destroy,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub(crate) struct HandJumpCollisionDispatchLoc329F8 {
    pub(crate) collision_type: u16,
    pub(crate) behavior: Option<HandJumpCollisionBehaviorLoc329F8>,
}

#[allow(non_snake_case)]
pub(crate) fn hand_jump_collision_dispatch_loc_329f8(
    collision_type: i16,
) -> HandJumpCollisionDispatchLoc329F8 {
    // ASM Hand.asm loc_329F8: negative collision types branch to the shared
    // destroy path, zero returns, and positive IDs clear `collision_type(a3)`
    // before dispatching through `off_32A12[(collision_type - 4)]`.
    if collision_type < 0 {
        return HandJumpCollisionDispatchLoc329F8 {
            collision_type: collision_type as u16,
            behavior: Some(HandJumpCollisionBehaviorLoc329F8::Destroy),
        };
    }

    if collision_type == 0 {
        return HandJumpCollisionDispatchLoc329F8 {
            collision_type: 0,
            behavior: None,
        };
    }

    let behavior = match collision_type as u16 {
        0x0004 | 0x0008 => Some(HandJumpCollisionBehaviorLoc329F8::ClearXVelocity),
        0x000C => Some(HandJumpCollisionBehaviorLoc329F8::ResetToPatrol),
        0x0010 => Some(HandJumpCollisionBehaviorLoc329F8::ClearYVelocity),
        0x0014 => Some(HandJumpCollisionBehaviorLoc329F8::BounceUpLeft),
        0x0018 => Some(HandJumpCollisionBehaviorLoc329F8::BounceUpRight),
        0x001C | 0x002C => Some(HandJumpCollisionBehaviorLoc329F8::ReturnOnly),
        0x0020 | 0x0024 | 0x0028 => Some(HandJumpCollisionBehaviorLoc329F8::TryGrabKid),
        _ => None,
    };

    HandJumpCollisionDispatchLoc329F8 {
        collision_type: 0,
        behavior,
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub(crate) enum HandKidContactRouteLoc32A86 {
    ReturnOnly,
    StartGrab,
}

#[allow(non_snake_case)]
pub(crate) fn hand_kid_contact_gate_loc_32a86(
    character_movement_is_crawling: bool,
    current_helmet_is_juggernaut: bool,
    kid_grabbed_by_hand: bool,
) -> HandKidContactRouteLoc32A86 {
    // ASM Hand.asm loc_32A86: airborne Kid-contact IDs return while Kid is
    // crawling, while Juggernaut is equipped, or while KidGrabbedByHand is
    // already set; only the all-clear path branches into grab setup.
    if character_movement_is_crawling || current_helmet_is_juggernaut || kid_grabbed_by_hand {
        HandKidContactRouteLoc32A86::ReturnOnly
    } else {
        HandKidContactRouteLoc32A86::StartGrab
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub(crate) struct HandRespawnMarkerLoc32AD8 {
    pub(crate) index: usize,
    pub(crate) word_value: u16,
}

#[allow(non_snake_case)]
pub(crate) fn hand_respawn_marker_loc_32ad8(object_status_slot: u8) -> HandRespawnMarkerLoc32AD8 {
    // ASM Hand.asm loc_32AD8: positive `$42(a5)` bytes are doubled as
    // EnemyStatus_Table word offsets, then `$2168` is written as the
    // respawn marker before the delete jump.
    HandRespawnMarkerLoc32AD8 {
        index: usize::from(object_status_slot),
        word_value: 0x2168,
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub(crate) enum HandDeathStatusRouteLoc32AA2 {
    WriteRespawnMarker(HandRespawnMarkerLoc32AD8),
    SubtractSpawnCounter { index: usize, decrement: u16 },
    DeleteOnly,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub(crate) struct HandDeleteTailLoc32AE6 {
    pub(crate) deletes_current_object: bool,
}

#[allow(non_snake_case)]
pub(crate) fn hand_delete_tail_loc_32ae6() -> HandDeleteTailLoc32AE6 {
    // ASM Hand.asm loc_32AE6: every death-status route converges on a direct
    // jump to `j_Delete_CurrentObject`.
    HandDeleteTailLoc32AE6 {
        deletes_current_object: true,
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) struct HandTerrainScanLoc32AFE {
    pub(crate) row_index: i16,
    pub(crate) start_col: i16,
    pub(crate) end_col: i16,
    pub(crate) sample_count: u16,
    pub(crate) first_collision_4000: u16,
    pub(crate) last_collision_4000: u16,
    pub(crate) any_collision_4000: bool,
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn hand_terrain_scan_loc_32afe(
    x_pos_word: u16,
    y_pos_word: u16,
    sampled_row_words: &[u16],
) -> HandTerrainScanLoc32AFE {
    // ASM Hand.asm loc_32AFE samples the map row at `(y + 1) >> 4`, scans
    // columns `(x - $E) >> 4` through `(x + $E) >> 4`, masks each word with
    // `$4000`, and returns the first/last masks while `tst.w d4` reports
    // whether any sampled word carried that bit. ASM Hand.asm loc_32B32 is
    // the per-cell loop body: `move.w (a4)+,d6; andi.w #$4000,d6`.
    // ASM Hand.asm loc_32B3E is the inclusive `dbf d7,loc_32B32`
    // back-edge; after the loop, `-2(a4)` is the last scanned word.
    let row_index = asr4_word(y_pos_word.wrapping_add(1));
    let start_col = asr4_word(x_pos_word.wrapping_sub(0x000E));
    let end_col = asr4_word(x_pos_word.wrapping_add(0x000E));
    let span = end_col.saturating_sub(start_col).saturating_add(1);
    let sample_count = u16::try_from(span).unwrap_or(0);
    let count = usize::from(sample_count);
    let first_collision_4000 = sampled_row_words.first().copied().unwrap_or(0) & 0x4000;
    let last_collision_4000 = sampled_row_words
        .get(count.saturating_sub(1))
        .copied()
        .unwrap_or(0)
        & 0x4000;
    let any_collision_4000 = sampled_row_words
        .iter()
        .take(count)
        .any(|word| word & 0x4000 != 0);

    HandTerrainScanLoc32AFE {
        row_index,
        start_col,
        end_col,
        sample_count,
        first_collision_4000,
        last_collision_4000,
        any_collision_4000,
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub(crate) struct HandDeathSequenceLoc32AA2 {
    pub(crate) has_kid_collision: bool,
    pub(crate) is_moved: bool,
    pub(crate) animation_label: &'static str,
    pub(crate) hibernate_until_animation_finished: bool,
    pub(crate) status_route: HandDeathStatusRouteLoc32AA2,
    pub(crate) deletes_current_object: bool,
}

#[allow(non_snake_case)]
pub(crate) fn hand_death_sequence_loc_32aa2(object_status_slot: u8) -> HandDeathSequenceLoc32AA2 {
    // ASM Hand.asm loc_32AA2: death starts by enabling Kid collision,
    // stopping movement, playing `stru_32B78` to completion, then routing
    // `$42(a5)` through the enemy status-table update before delete.
    let status_route = if object_status_slot & 0x80 == 0 {
        HandDeathStatusRouteLoc32AA2::WriteRespawnMarker(hand_respawn_marker_loc_32ad8(
            object_status_slot,
        ))
    } else if object_status_slot & 0x40 != 0 {
        HandDeathStatusRouteLoc32AA2::SubtractSpawnCounter {
            index: usize::from(object_status_slot & 0x3F),
            decrement: 0x0400,
        }
    } else {
        HandDeathStatusRouteLoc32AA2::DeleteOnly
    };

    HandDeathSequenceLoc32AA2 {
        has_kid_collision: true,
        is_moved: false,
        animation_label: "stru_32B78",
        hibernate_until_animation_finished: true,
        status_route,
        deletes_current_object: hand_delete_tail_loc_32ae6().deletes_current_object,
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub(crate) struct HandJumpCollisionVelocityLoc32A3E {
    pub(crate) x_velocity: i32,
    pub(crate) y_velocity: i32,
}

#[allow(non_snake_case)]
pub(crate) fn hand_jump_collision_clear_y_velocity_loc_32a3e(
    x_velocity: i32,
    _y_velocity: i32,
) -> HandJumpCollisionVelocityLoc32A3E {
    // ASM Hand.asm loc_32A3E: `clr.l y_vel(a3)` clears only the vertical
    // velocity when the airborne collision table receives contact `$10`.
    HandJumpCollisionVelocityLoc32A3E {
        x_velocity,
        y_velocity: 0,
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub(crate) struct HandJumpCollisionVelocityLoc32A44 {
    pub(crate) x_velocity: i32,
    pub(crate) y_velocity: i32,
}

#[allow(non_snake_case)]
pub(crate) fn hand_jump_collision_clear_x_velocity_loc_32a44(
    _x_velocity: i32,
    y_velocity: i32,
) -> HandJumpCollisionVelocityLoc32A44 {
    // ASM Hand.asm loc_32A44: `clr.l x_vel(a3)` clears only the horizontal
    // velocity when the airborne collision table receives contact `$04`/`$08`.
    HandJumpCollisionVelocityLoc32A44 {
        x_velocity: 0,
        y_velocity,
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub(crate) struct HandJumpCollisionResetLoc32A4A {
    pub(crate) x_velocity: i32,
    pub(crate) y_velocity: i32,
    pub(crate) initial_motion: HandInitialMotionLoc327F2,
}

#[allow(non_snake_case)]
pub(crate) fn hand_jump_collision_reset_loc_32a4a(
    hit_points: u8,
    x_direction_flag: bool,
) -> HandJumpCollisionResetLoc32A4A {
    // ASM Hand.asm loc_32A4A: clear both velocity longs, drop the collision
    // caller return address with `addq.w #4,sp`, then branch into loc_327E4.
    HandJumpCollisionResetLoc32A4A {
        x_velocity: 0,
        y_velocity: 0,
        initial_motion: hand_initial_motion_loc_327f2(hit_points, x_direction_flag),
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub(crate) struct HandJumpCollisionBounceLoc32A5A {
    pub(crate) x_pos: i32,
    pub(crate) y_pos: i32,
    pub(crate) x_velocity: i32,
    pub(crate) y_velocity: i32,
}

#[allow(non_snake_case)]
pub(crate) fn hand_jump_collision_bounce_up_left_loc_32a5a(
    x_pos: i32,
    y_pos: i32,
    x_velocity: i32,
    y_velocity: i32,
) -> HandJumpCollisionBounceLoc32A5A {
    // ASM Hand.asm loc_32A5A: call shared loc_350F2 to roll back by the
    // current velocity, then store `$FFFE8000` into both velocity longs.
    let restored = ufo_restore_position_loc_350f2(x_pos, y_pos, x_velocity, y_velocity);
    HandJumpCollisionBounceLoc32A5A {
        x_pos: restored.x_pos,
        y_pos: restored.y_pos,
        x_velocity: -0x0001_8000,
        y_velocity: -0x0001_8000,
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub(crate) struct HandJumpCollisionBounceLoc32A70 {
    pub(crate) x_pos: i32,
    pub(crate) y_pos: i32,
    pub(crate) x_velocity: i32,
    pub(crate) y_velocity: i32,
}

#[allow(non_snake_case)]
pub(crate) fn hand_jump_collision_bounce_up_right_loc_32a70(
    x_pos: i32,
    y_pos: i32,
    x_velocity: i32,
    y_velocity: i32,
) -> HandJumpCollisionBounceLoc32A70 {
    // ASM Hand.asm loc_32A70: call shared loc_350F2 to roll back by the
    // current velocity, then store `$18000` X and `$FFFE8000` Y velocity.
    let restored = ufo_restore_position_loc_350f2(x_pos, y_pos, x_velocity, y_velocity);
    HandJumpCollisionBounceLoc32A70 {
        x_pos: restored.x_pos,
        y_pos: restored.y_pos,
        x_velocity: 0x0001_8000,
        y_velocity: -0x0001_8000,
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub(crate) struct HandGroundCollisionReverseLoc329E2 {
    pub(crate) x_velocity: i32,
    pub(crate) x_direction_flag: bool,
}

#[allow(non_snake_case)]
pub(crate) fn hand_ground_collision_reverse_loc_329e2(
    x_velocity: i32,
    x_direction_flag: bool,
) -> HandGroundCollisionReverseLoc329E2 {
    // ASM Hand.asm loc_329E2: `neg.l x_vel(a3)` reverses the full 32-bit
    // fixed-point velocity, then `not.b x_direction(a3)` flips the raw flag.
    HandGroundCollisionReverseLoc329E2 {
        x_velocity: x_velocity.wrapping_neg(),
        x_direction_flag: !x_direction_flag,
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub(crate) struct HandKidBelowContactLoc329EC {
    pub(crate) kid_collision_type: u16,
    pub(crate) kid_object_meta: u16,
}

#[allow(non_snake_case)]
pub(crate) fn hand_kid_below_contact_loc_329ec() -> HandKidBelowContactLoc329EC {
    // ASM Hand.asm loc_329EC: `a2` is the Kid GfxObject pointer captured by
    // Enemy17_Hand_Init; side contact from a grounded hand writes `$28` to
    // `collision_type(a2)` and clears `object_meta(a2)`.
    HandKidBelowContactLoc329EC {
        kid_collision_type: 0x0028,
        kid_object_meta: 0,
    }
}

/// Hand grab duration: 10 frames (Hand.asm loc_32906: moveq #$A,d3).
pub(crate) const HAND_GRAB_DURATION: u16 = 10;
const HAND_GRAB_TIMER_REMAINING_MASK: u16 = 0x00FF;
const HAND_GRAB_TIMER_LAST_DIRECTION_INIT: u16 = 0x8000;
const HAND_GRAB_TIMER_LAST_DIRECTION_RIGHT: u16 = 0x4000;

fn hand_kid_direction_byte_loc_3292e(facing: Direction) -> bool {
    // ASM Hand.asm loc_3292E stores Kid `$16` in d2. Native only needs the
    // zero/non-zero distinction used by the compare before decrementing d3.
    matches!(facing, Direction::Right)
}

fn hand_grab_timer_remaining(wait_timer: u16) -> u16 {
    wait_timer & HAND_GRAB_TIMER_REMAINING_MASK
}

fn hand_grab_timer_with_direction(remaining: u16, facing: Direction) -> u16 {
    let direction_bit = if hand_kid_direction_byte_loc_3292e(facing) {
        HAND_GRAB_TIMER_LAST_DIRECTION_RIGHT
    } else {
        0
    };
    HAND_GRAB_TIMER_LAST_DIRECTION_INIT
        .wrapping_add(direction_bit)
        .wrapping_add(remaining & HAND_GRAB_TIMER_REMAINING_MASK)
}

fn hand_grab_timer_step_loc_3292e(wait_timer: u16, player_facing: Option<Direction>) -> u16 {
    let remaining = hand_grab_timer_remaining(wait_timer);
    let Some(facing) = player_facing else {
        return remaining.saturating_sub(1);
    };
    let current_direction = hand_kid_direction_byte_loc_3292e(facing);
    let had_previous = wait_timer & HAND_GRAB_TIMER_LAST_DIRECTION_INIT != 0;
    let previous_direction = wait_timer & HAND_GRAB_TIMER_LAST_DIRECTION_RIGHT != 0;
    let next_remaining = if had_previous && previous_direction == current_direction {
        remaining
    } else {
        remaining.saturating_sub(1)
    };
    hand_grab_timer_with_direction(next_remaining, facing)
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub(crate) struct HandGrabSnapLoc3294C {
    pub(crate) x_pos: u32,
    pub(crate) y_pos: u32,
}

#[allow(non_snake_case)]
pub(crate) fn hand_grab_snap_loc_3294c(
    kid_x_pos: u32,
    kid_y_pos: u32,
    x_direction_flag: bool,
) -> HandGrabSnapLoc3294C {
    // ASM Hand.asm loc_3294C: loc_3292E prepares Kid X+2, subtracts 4 only
    // when raw x_direction is set, then stores high position words and clears
    // both fractional halves. Y always snaps to Kid Y-4.
    let kid_x_word = (kid_x_pos >> 16) as u16;
    let x_word = if x_direction_flag {
        kid_x_word.wrapping_sub(2)
    } else {
        kid_x_word.wrapping_add(2)
    };
    let y_word = ((kid_y_pos >> 16) as u16).wrapping_sub(4);

    HandGrabSnapLoc3294C {
        x_pos: u32::from(x_word) << 16,
        y_pos: u32::from(y_word) << 16,
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub(crate) struct HandReleaseLoc32970 {
    pub(crate) kid_grabbed_by_hand: bool,
    pub(crate) hand_state: HandState,
    pub(crate) on_ground: bool,
    pub(crate) jump_speed_seed: i16,
    pub(crate) collision_probe_half_width_px: i16,
    pub(crate) collision_probe_height_px: i16,
}

#[allow(non_snake_case)]
pub(crate) fn hand_release_loc_32970() -> HandReleaseLoc32970 {
    // ASM Hand.asm loc_32970: clear KidGrabbedByHand, probe a 9x14 collision
    // window, restore level-collision/moved flags, load `d5 = 2`, then branch
    // to loc_328AA for a new jump setup instead of resuming patrol.
    HandReleaseLoc32970 {
        kid_grabbed_by_hand: false,
        hand_state: HandState::Jumping,
        on_ground: false,
        jump_speed_seed: 2,
        collision_probe_half_width_px: 0x0E,
        collision_probe_height_px: 9,
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub(crate) struct HandInitLoc327B6 {
    pub(crate) status_flag_13: bool,
    pub(crate) is_moved: bool,
    pub(crate) x_direction_flag: bool,
    pub(crate) motion_flag_19: bool,
    pub(crate) has_level_collision: bool,
    pub(crate) priority: u8,
    pub(crate) object_meta: u16,
}

#[allow(non_snake_case)]
pub(crate) fn hand_init_loc_327b6() -> HandInitLoc327B6 {
    // ASM Hand.asm loc_327B6: `st $13/is_moved/has_level_collision`,
    // `sf x_direction/$19`, low priority, and object_meta `$17`.
    HandInitLoc327B6 {
        status_flag_13: true,
        is_moved: true,
        x_direction_flag: false,
        motion_flag_19: false,
        has_level_collision: true,
        priority: 0,
        object_meta: 0x0017,
    }
}

#[allow(non_snake_case)]
pub(crate) fn hand_velocity_candidate_loc_327e4(hit_points: u8, x_direction_flag: bool) -> i32 {
    // ASM Hand.asm loc_327E4: load the negative `$48(a5)` table velocity,
    // then negate it only when raw x_direction is non-zero.
    let tier = usize::from(hit_points.saturating_sub(1)).min(2);
    let table_velocity = -HAND_WALK_SPEED[tier];
    if x_direction_flag {
        table_velocity.saturating_neg()
    } else {
        table_velocity
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub(crate) struct HandInitialMotionLoc327F2 {
    pub(crate) x_velocity: i32,
    pub(crate) animation_label: &'static str,
    pub(crate) has_kid_collision: bool,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) enum HandPatrolDecision {
    ContinuePatrol,
    JumpAttack,
    ReverseDirection,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub(crate) struct HandReversalLoc32840 {
    pub(crate) x_velocity: i32,
    pub(crate) x_direction_flag: bool,
}

#[allow(non_snake_case)]
pub(crate) fn hand_reversal_loc_32840(
    x_velocity: i32,
    x_direction_flag: bool,
) -> HandReversalLoc32840 {
    // ASM Hand.asm loc_32840: `neg.l d5; move.l d5,x_vel(a3);
    // not.b x_direction(a3)` reverses patrol velocity and flips the raw flag.
    HandReversalLoc32840 {
        x_velocity: x_velocity.wrapping_neg(),
        x_direction_flag: !x_direction_flag,
    }
}

#[allow(non_snake_case)]
pub(crate) fn hand_left_detection_loc_32886(
    relative_x_px: i16,
    vertical_delta_minus_16_px: i16,
    detect_range_px: i16,
) -> bool {
    // ASM Hand.asm loc_32886: clear x_direction means the Hand faces left.
    // d6 must be negative, not below `-$44(a5)`, and d7 must be >= d6.
    if relative_x_px >= 0 {
        return false;
    }

    let negative_range = detect_range_px.wrapping_neg();
    if relative_x_px < negative_range {
        return false;
    }

    vertical_delta_minus_16_px >= relative_x_px
}

fn hand_right_detection_loc_3284c(
    relative_x_px: i16,
    vertical_delta_minus_16_px: i16,
    detect_range_px: i16,
) -> bool {
    // ASM Hand.asm loc_3284C: set x_direction means the Hand faces right.
    // d6 must be non-negative, within `$44(a5)`, and after `neg.w d7`, the
    // vertical lead from KidY-16 must not be greater than the horizontal lead.
    if relative_x_px < 0 {
        return false;
    }

    if relative_x_px > detect_range_px {
        return false;
    }

    vertical_delta_minus_16_px.wrapping_neg() <= relative_x_px
}

#[allow(non_snake_case)]
pub(crate) fn hand_detection_reject_loc_328a2() -> i16 {
    // ASM Hand.asm loc_328A2: shared reject return from range/cone checks,
    // `moveq #0,d7; rts`.
    0
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn hand_patrol_decision_loc_32804(
    x_velocity: i32,
    probe_d6: i16,
    probe_d7: i16,
) -> HandPatrolDecision {
    // ASM Hand.asm loc_32804: after the shared patrol probe, the loop treats
    // d6/d7 as opaque probe words. Native only models the branch decision here.
    if x_velocity < 0 {
        if probe_d6 != 0 {
            HandPatrolDecision::ContinuePatrol
        } else if probe_d7 != 0 {
            // ASM Hand.asm loc_32836: negative x_vel reaches the reversal
            // branch when d6 is clear and d7 remains non-zero.
            HandPatrolDecision::ReverseDirection
        } else {
            HandPatrolDecision::JumpAttack
        }
    } else if probe_d7 != 0 {
        HandPatrolDecision::ContinuePatrol
    } else if probe_d6 != 0 {
        // ASM Hand.asm loc_32832: positive x_vel reaches the reversal branch
        // when d7 is clear and d6 remains non-zero.
        HandPatrolDecision::ReverseDirection
    } else {
        HandPatrolDecision::JumpAttack
    }
}

fn hand_x_direction_flag(facing: Direction) -> bool {
    matches!(facing, Direction::Right)
}

fn hand_facing_from_x_direction_flag(x_direction_flag: bool) -> Direction {
    if x_direction_flag {
        Direction::Right
    } else {
        Direction::Left
    }
}

fn reverse_hand_loc_32840(enemy: &mut ActiveEnemy) {
    let reversal = hand_reversal_loc_32840(enemy.x_velocity, hand_x_direction_flag(enemy.facing));
    enemy.x_velocity = reversal.x_velocity;
    enemy.facing = hand_facing_from_x_direction_flag(reversal.x_direction_flag);
}

/// Step a Hand enemy through its walk/detect/jump state machine.
/// Walking: paces with edge/wall detection. Detects player in 45-degree cone.
/// Jumping: randomised arc toward player with gravity, lands and resumes walking.
/// ASM: Hand.asm — full state machine with cone detection, random jump, gravity cap.
pub(crate) fn step_hand(
    enemy: &mut ActiveEnemy,
    block_grid: Option<&terrain::BlockGrid>,
    player_x_pos: Option<u32>,
    player_y_pos: Option<u32>,
    player_facing: Option<Direction>,
) {
    let tier = (enemy.hit_points.saturating_sub(1) as usize).min(2);

    match enemy.hand_state {
        HandState::Walking => {
            // Walk like a pacer with Hand-specific speed
            let walk_speed = HAND_WALK_SPEED[tier];
            if let Some(grid) = block_grid {
                enemy.x_velocity = horizontal_velocity(enemy.facing, walk_speed);
                enemy.y_velocity = 0;
                enemy.x_pos = (enemy.x_pos as i64 + i64::from(enemy.x_velocity)) as u32;

                let cx = (enemy.x_pos >> 16) as i32;
                let cy = (enemy.y_pos >> 16) as i32;
                let moving_right = enemy.x_velocity >= 0;

                let wall = terrain::probe_wall(
                    grid,
                    cx,
                    cy,
                    ENEMY_PROBE_HALF_WIDTH,
                    ENEMY_PROBE_HEIGHT,
                    moving_right,
                );
                if wall.hit {
                    let wall_fp = (wall.wall_x_px as u32) << 16;
                    let edge_fp = (ENEMY_PROBE_HALF_WIDTH as u32) << 16;
                    if moving_right {
                        let blocked_x = wall_fp.saturating_sub(edge_fp);
                        enemy.x_pos = enemy.x_pos.min(blocked_x);
                    } else {
                        let blocked_x = wall_fp.saturating_add(edge_fp);
                        enemy.x_pos = enemy.x_pos.max(blocked_x);
                    }
                    reverse_hand_loc_32840(enemy);
                    return;
                }

                let look_ahead_x = if moving_right {
                    cx + ENEMY_PROBE_HALF_WIDTH
                } else {
                    cx - ENEMY_PROBE_HALF_WIDTH
                };
                if !terrain::probe_floor(grid, look_ahead_x, cy).on_ground {
                    reverse_hand_loc_32840(enemy);
                }
            }

            // Player detection: forward cone from ASM Hand.asm loc_3284C/loc_32886.
            let (Some(px), Some(py)) = (player_x_pos, player_y_pos) else {
                return;
            };
            let hand_x_word = fixed_position_integer_word(enemy.x_pos);
            let hand_y_word = fixed_position_integer_word(enemy.y_pos);
            let kid_x_word = fixed_position_integer_word(px);
            let kid_y_word = fixed_position_integer_word(py);
            let relative_x_px =
                i16::from_be_bytes(kid_x_word.wrapping_sub(hand_x_word).to_be_bytes());
            let vertical_delta_minus_16_px = i16::from_be_bytes(
                kid_y_word
                    .wrapping_sub(0x0010)
                    .wrapping_sub(hand_y_word)
                    .to_be_bytes(),
            );

            let detection_d7 = if vertical_delta_minus_16_px >= 0 {
                hand_detection_reject_loc_328a2()
            } else {
                let detect_range = HAND_DETECT_RANGE[tier] as i16;
                let in_cone = match enemy.facing {
                    Direction::Right => hand_right_detection_loc_3284c(
                        relative_x_px,
                        vertical_delta_minus_16_px,
                        detect_range,
                    ),
                    Direction::Left => hand_left_detection_loc_32886(
                        relative_x_px,
                        vertical_delta_minus_16_px,
                        detect_range,
                    ),
                };
                if in_cone {
                    1
                } else {
                    hand_detection_reject_loc_328a2()
                }
            };

            if detection_d7 != 0 {
                // Jump toward player with randomised velocity (Hand.asm lines 128-147)
                enemy.hand_state = HandState::Jumping;
                enemy.on_ground = false;
                let speed_seed = hand_jump_seed_loc_328a6(enemy.hit_points);
                let jump = hand_jump_components_loc_328aa(
                    speed_seed,
                    hand_x_direction_flag(enemy.facing),
                    u32::from(enemy.wait_timer).wrapping_mul(7).to_le_bytes()[0],
                    fixed_position_integer_word(py),
                    u32::from(enemy.wait_timer)
                        .wrapping_mul(13)
                        .wrapping_add(5)
                        .to_le_bytes()[0],
                    fixed_position_integer_word(px),
                );
                enemy.x_velocity = hand_x_velocity_loc_328e0(jump.x_velocity_word);
                enemy.y_velocity = jump.y_velocity;
            }
        }
        HandState::Jumping => {
            enemy.y_velocity = hand_jump_loop_gravity_loc_328f0(enemy.y_velocity);

            // Move
            enemy.x_pos = (enemy.x_pos as i64 + i64::from(enemy.x_velocity)) as u32;
            enemy.y_pos = (enemy.y_pos as i64 + i64::from(enemy.y_velocity)) as u32;

            let Some(grid) = block_grid else {
                return;
            };
            let cx = (enemy.x_pos >> 16) as i32;
            let cy = (enemy.y_pos >> 16) as i32;

            // Wall reversal
            let moving_right = enemy.x_velocity >= 0;
            let wall = terrain::probe_wall(
                grid,
                cx,
                cy,
                ENEMY_PROBE_HALF_WIDTH,
                ENEMY_PROBE_HEIGHT,
                moving_right,
            );
            if wall.hit {
                enemy.x_velocity = 0;
            }

            // Floor landing: check for grab, then return to walking
            if enemy.y_velocity > 0 {
                let floor = terrain::probe_floor(grid, cx, cy);
                if floor.on_ground {
                    let floor_y_fp = (floor.floor_y_px as u32) << 16;
                    if enemy.y_pos >= floor_y_fp {
                        enemy.y_pos = floor_y_fp;
                        enemy.x_velocity = 0;
                        enemy.y_velocity = 0;
                        enemy.on_ground = true;

                        // On landing, check if player is within grab range
                        // (Hand.asm loc_3284C — same range check as jump trigger)
                        let grab = if let (Some(px), Some(py)) = (player_x_pos, player_y_pos) {
                            let dx = ((px as i64 - enemy.x_pos as i64) >> 16).abs();
                            let dy = ((enemy.y_pos as i64 - py as i64) >> 16).abs();
                            dx <= i64::from(HAND_DETECT_RANGE[tier]) && dy <= 16
                        } else {
                            false
                        };

                        if grab {
                            // Enter grab: follow player for 10 frames (Hand.asm loc_32906)
                            enemy.hand_state = HandState::Grabbing;
                            enemy.wait_timer = HAND_GRAB_DURATION;
                        } else {
                            enemy.hand_state = HandState::Walking;
                        }
                    }
                }
            }
        }
        HandState::Grabbing => {
            // Hand follows player position for HAND_GRAB_DURATION frames.
            // Hand.asm loc_3294C: hand snaps to Kid's integer X±2, Y-4 each frame.
            if let (Some(px), Some(py)) = (player_x_pos, player_y_pos) {
                let snap = hand_grab_snap_loc_3294c(px, py, hand_x_direction_flag(enemy.facing));
                enemy.x_pos = snap.x_pos;
                enemy.y_pos = snap.y_pos;
            }
            enemy.x_velocity = 0;
            enemy.y_velocity = 0;

            if hand_grab_timer_remaining(enemy.wait_timer) > 0 {
                enemy.wait_timer = hand_grab_timer_step_loc_3292e(enemy.wait_timer, player_facing);
            } else {
                let release = hand_release_loc_32970();
                if let Some(grid) = block_grid {
                    // ASM Hand.asm loc_32970: `d6 = 9`, `d7 = $E`, then
                    // shared loc_3639E blocks the release jump by branching to loc_32AA2.
                    let clear = emo_rock_clearance_probe_loc_3639e(
                        grid,
                        i32::from(fixed_position_integer_word(enemy.x_pos)),
                        i32::from(fixed_position_integer_word(enemy.y_pos)),
                        release.collision_probe_height_px as u16,
                        release.collision_probe_half_width_px as u16,
                    );
                    if !clear {
                        enemy.hit_points = 0;
                        enemy.death_animation_active = true;
                        enemy.wait_timer = HAND_DEATH_ANIMATION_FRAMES;
                        return;
                    }
                }
                enemy.hand_state = release.hand_state;
                enemy.on_ground = release.on_ground;
                if let (Some(px), Some(py)) = (player_x_pos, player_y_pos) {
                    let jump = hand_jump_components_loc_328aa(
                        release.jump_speed_seed,
                        hand_x_direction_flag(enemy.facing),
                        u32::from(enemy.wait_timer).wrapping_mul(7).to_le_bytes()[0],
                        fixed_position_integer_word(py),
                        u32::from(enemy.wait_timer)
                            .wrapping_mul(13)
                            .wrapping_add(5)
                            .to_le_bytes()[0],
                        fixed_position_integer_word(px),
                    );
                    enemy.x_velocity = hand_x_velocity_loc_328e0(jump.x_velocity_word);
                    enemy.y_velocity = jump.y_velocity;
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hand_init_matches_loc_327b6_flags_and_default_direction() {
        let init = hand_init_loc_327b6();

        assert_eq!(
            init,
            HandInitLoc327B6 {
                status_flag_13: true,
                is_moved: true,
                x_direction_flag: false,
                motion_flag_19: false,
                has_level_collision: true,
                priority: 0,
                object_meta: 0x0017,
            },
            "ASM Hand.asm loc_327B6 sets the active/moved/collision flags, clears raw x_direction/$19, and tags object_meta `$17`"
        );
        assert_eq!(
            spawn_facing(EnemyFamily::Hand, 100 << 16, 120 << 16),
            Direction::Left,
            "ASM Hand.asm loc_327B6 clears x_direction; the following hand range check treats clear as the left-facing branch"
        );
    }

    #[test]
    fn hand_velocity_candidate_matches_loc_327e4_direction_gate() {
        assert_eq!(
            hand_velocity_candidate_loc_327e4(1, false),
            -0x0000_3000,
            "ASM Hand.asm loc_327E4 leaves the negative tier-1 table velocity unchanged when raw x_direction is clear"
        );
        assert_eq!(
            hand_velocity_candidate_loc_327e4(2, false),
            -0x0000_4800,
            "ASM Hand.asm loc_327E4 reads the second HP tier from `$48(a5)` before testing x_direction"
        );
        assert_eq!(
            hand_velocity_candidate_loc_327e4(3, true),
            0x0000_6000,
            "ASM Hand.asm loc_327E4 negates the tier-3 table velocity only when raw x_direction is set"
        );
    }

    #[test]
    fn hand_initial_motion_matches_loc_327f2_store_and_collision_clear() {
        assert_eq!(
            hand_initial_motion_loc_327f2(1, false),
            HandInitialMotionLoc327F2 {
                x_velocity: -0x0000_3000,
                animation_label: "stru_32B6A",
                has_kid_collision: false,
            },
            "ASM Hand.asm loc_327F2 stores the prepared d7 velocity, starts `stru_32B6A`, and clears has_kid_collision"
        );
        assert_eq!(
            hand_initial_motion_loc_327f2(2, true).x_velocity,
            0x0000_4800,
            "ASM Hand.asm loc_327F2 preserves the direction-gated d7 value when writing x_vel"
        );
        assert_eq!(
            spawn_x_velocity(EnemyFamily::Hand, 0),
            -0x0000_3000,
            "native Hand spawn x_velocity should expose the ASM Hand.asm loc_327F2 initial store for tier 1"
        );
        assert_eq!(
            spawn_x_velocity(EnemyFamily::Hand, 1),
            -0x0000_4800,
            "native Hand spawn x_velocity should expose the ASM Hand.asm loc_327F2 initial store for tier 2"
        );
        assert_eq!(
            spawn_x_velocity(EnemyFamily::Hand, 2),
            -0x0000_6000,
            "native Hand spawn x_velocity should expose the ASM Hand.asm loc_327F2 initial store for tier 3"
        );
    }

    #[test]
    fn hand_jump_components_loc_328aa_uses_eor_low_position_byte() {
        let components = hand_jump_components_loc_328aa(2, true, 0x01, 0x0001, 0x02, 0x0002);

        assert_eq!(
            components,
            HandJumpComponentsLoc328AA {
                y_velocity: -0x0002_0000,
                x_velocity_word: 2,
            },
            "ASM Hand.asm loc_328AA uses `eor.b d6,d7; andi.l #3,d7`, so equal random/Kid low bytes contribute zero rather than OR-ing bits"
        );
    }

    #[test]
    fn hand_grab_timer_loc_3292e_packs_direction_bits_and_counter() {
        let first_left = hand_grab_timer_step_loc_3292e(HAND_GRAB_DURATION, Some(Direction::Left));
        assert_eq!(
            first_left, 0x8009,
            "ASM Hand.asm loc_3292E starts with d2 clear, samples Kid x_direction, and decrements d3 on the first changed sample"
        );

        let held_left = hand_grab_timer_step_loc_3292e(first_left, Some(Direction::Left));
        assert_eq!(
            held_left, 0x8009,
            "ASM Hand.asm loc_3292E loops without decrementing while the stored Kid direction byte is unchanged"
        );

        let turned_right = hand_grab_timer_step_loc_3292e(held_left, Some(Direction::Right));
        assert_eq!(
            turned_right, 0xC008,
            "ASM Hand.asm loc_3292E stores the new nonzero Kid x_direction and decrements the remaining grab count"
        );

        assert_eq!(
            hand_grab_timer_step_loc_3292e(turned_right, None),
            0x0007,
            "native no-player-facing path preserves only d3's remaining low byte before decrementing"
        );
    }

    #[test]
    fn hand_delete_tail_loc_32ae6_deletes_current_object() {
        assert_eq!(
            hand_delete_tail_loc_32ae6(),
            HandDeleteTailLoc32AE6 {
                deletes_current_object: true,
            },
            "ASM Hand.asm loc_32AE6 jumps directly to `j_Delete_CurrentObject` after every death-status path"
        );
        assert!(
            hand_death_sequence_loc_32aa2(0x80).deletes_current_object,
            "ASM Hand.asm loc_32AE6 is also reached by the bit-7 delete-only status route"
        );
    }

    #[test]
    fn hand_init_y_snap_loc_32aea_aligns_spawn_word_after_blank_rows() {
        assert_eq!(
            hand_init_y_snap_loc_32aea(0x0048),
            0x004F,
            "ASM Hand.asm loc_32AEA subtracts 8, masks to a 16px boundary, then adds `$F`"
        );
        assert_eq!(
            hand_init_y_snap_loc_32aea(0x0040),
            0x003F,
            "ASM Hand.asm loc_32AEA can snap to the previous tile row when y is already aligned"
        );
        assert_eq!(
            spawn_y_pos_sub_36FF4(EnemyFamily::Hand, 0x0040, 1),
            0x004F_0000,
            "native Hand spawn y should apply sub_36FF4 before ASM Hand.asm loc_32AEA"
        );
    }

    #[test]
    fn hand_terrain_scan_loc_32afe_returns_edge_masks_and_any_hit() {
        let scan = hand_terrain_scan_loc_32afe(0x0040, 0x004F, &[0x4000, 0x0000]);
        assert_eq!(
            scan,
            HandTerrainScanLoc32AFE {
                row_index: 5,
                start_col: 3,
                end_col: 4,
                sample_count: 2,
                first_collision_4000: 0x4000,
                last_collision_4000: 0,
                any_collision_4000: true,
            },
            "ASM Hand.asm loc_32AFE scans `(x-$E)..(x+$E)` on row `(y+1)>>4` and returns d6=first, d7=last"
        );
        assert_eq!(
            hand_patrol_decision_loc_32804(
                0x3000,
                i16::try_from(scan.first_collision_4000).expect("mask fits in signed word"),
                i16::try_from(scan.last_collision_4000).expect("mask fits in signed word"),
            ),
            HandPatrolDecision::ReverseDirection,
            "ASM Hand.asm loc_32AFE feeds d6/d7 into the positive-x patrol edge branch"
        );
    }

    #[test]
    fn hand_terrain_scan_loc_32afe_reports_no_support_for_death_path() {
        let scan = hand_terrain_scan_loc_32afe(0x0040, 0x004F, &[0x0000, 0x0000]);
        assert!(
            !scan.any_collision_4000,
            "ASM Hand.asm loc_32AFE leaves d4 zero when no sampled word has `$4000`, so the following `beq.w loc_32AA2` death path is taken"
        );
    }
}
