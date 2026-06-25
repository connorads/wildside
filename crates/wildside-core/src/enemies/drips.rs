use super::*;

// --- Drips constants (Drips.asm) ---
// ASM Drips is a multi-object enemy: ceiling attachment, forming animation,
// animation-driven drop, splash, and surface-crawl spawning.
// Native approximation: periodic projectile drop with ASM-matched timing; when
// the falling child splashes on terrain, it can allocate a hostile
// DripsSurfaceCrawler projectile through the ASM loc_335D8 spawn path.
// Random dormant delay: Drips.asm loc_3340E uses `(RNG >> 2)`,
// then XORs the low byte with Kid's integer X low byte.
// Drop child starts with zero vertical speed, then gains $2000/frame gravity
// until it hits the floor. ASM loc_33424 clears `$2A(a1)` before the fall and
// loc_3345C adds `#$2000` every frame.
// Full surface-crawler wall-following turn scripts remain modelled as pure
// helpers below; the live projectile carrier currently implements the spawn,
// hostile collision, cap, and initial horizontal crawl.
/// ASM loc_33424 clears the drop child's y-velocity before the fall begins.
pub(crate) const DRIPS_DROP_INITIAL_Y_SPEED: i32 = 0;
/// Gravity accumulation: ASM loc_3345C `addi.l #$2000,$2A(a1)`.
pub(crate) const DRIPS_DROP_GRAVITY: i32 = 0x2000;
/// Sentinel lifetime for the falling drop. ASM loc_3345C has no short timer:
/// terrain/contact ends the drop before the splash/reset sequence.
pub(crate) const DRIPS_DROP_LIFETIME: u16 = u16::MAX;
/// Native-only sentinel for the parent/controller while its falling child is
/// active. ASM Drips.asm loc_33424/loc_3345C does not branch back to
/// loc_3340E until the child reaches loc_33488.
pub(crate) const DRIPS_DROP_IN_FLIGHT_TIMER: u16 = u16::MAX;

/// Native-only two-frame bridge after loc_333F6 ceiling contact. Oracle #260
/// shows the Level 01 right Drips reaches loc_33424 two trace frames after the
/// native ceiling snap; delaying the loc_3340E seed keeps the RNG draw aligned
/// with the ROM coroutine resume point.
pub(crate) const DRIPS_INITIAL_SEED_DELAY_2_TIMER_LOC_3340E: u16 = u16::MAX - 2;
const DRIPS_INITIAL_SEED_DELAY_1_TIMER_LOC_3340E: u16 = u16::MAX - 3;
/// ASM Drips.asm loc_33424 initialises `stru_33F90` and waits until the
/// animation finishes before exposing the falling child. The script durations
/// are `$A,$A,8,6,1`, but ASM GfxObject_Animate loc_1158/sub_1194 consumes the
/// frame that loads each entry and the final duration-1 entry finishes
/// immediately when the terminator is visible.
pub(crate) const DRIPS_PRE_DROP_ANIMATION_FRAMES_LOC_33424: u16 =
    0x0A + 0x0A + 0x08 + 0x06 + 0x01 - 4;
const DRIPS_PRE_DROP_ANIMATION_FLAG_LOC_33424: u16 = 0x8000;
/// ASM Drips.asm loc_33488 initialises `stru_33332` and hibernates until the
/// linked splash child finishes before `off_33520` advances. Oracle issue-260
/// frame 305 native splash entry stays dormant until frame 340 like the ROM,
/// so native holds this coroutine across
/// the linked child animation before advancing the table cursor.
pub(crate) const DRIPS_SPLASH_ANIMATION_FRAMES_LOC_33488: u16 = 39;
const DRIPS_SPLASH_ANIMATION_FLAG_LOC_33488: u16 = 0x4000;

fn drips_pre_drop_animation_timer_loc_33424(frames_remaining: u16) -> u16 {
    DRIPS_PRE_DROP_ANIMATION_FLAG_LOC_33424 | frames_remaining
}

fn drips_pre_drop_animation_remaining_loc_33424(wait_timer: u16) -> Option<u16> {
    if wait_timer & DRIPS_PRE_DROP_ANIMATION_FLAG_LOC_33424 == 0 {
        None
    } else {
        Some(wait_timer & !DRIPS_PRE_DROP_ANIMATION_FLAG_LOC_33424)
    }
}

pub(crate) fn drips_splash_animation_timer_loc_33488(frames_remaining: u16) -> u16 {
    DRIPS_SPLASH_ANIMATION_FLAG_LOC_33488 | frames_remaining
}

pub(crate) fn drips_splash_animation_remaining_loc_33488(wait_timer: u16) -> Option<u16> {
    if wait_timer & DRIPS_SPLASH_ANIMATION_FLAG_LOC_33488 == 0 {
        None
    } else {
        Some(wait_timer & !DRIPS_SPLASH_ANIMATION_FLAG_LOC_33488)
    }
}
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) const DRIPS_COLID_CEILING: u16 = colid::CEILING;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) struct DripsCeilingGateLoc333F6 {
    pub(crate) waiting_for_ceiling: bool,
    pub(crate) collision_type: u16,
    pub(crate) y_velocity: i32,
}

#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn drips_ceiling_gate_loc_333_f6(
    collision_type: u16,
    y_velocity: i32,
) -> DripsCeilingGateLoc333F6 {
    // ASM Drips.asm loc_333F6: loop until `collision_type(a3)` is
    // `colid_ceiling`, then clear `collision_type(a3)` and `y_vel(a3)`.
    if collision_type == DRIPS_COLID_CEILING {
        DripsCeilingGateLoc333F6 {
            waiting_for_ceiling: false,
            collision_type: 0,
            y_velocity: 0,
        }
    } else {
        DripsCeilingGateLoc333F6 {
            waiting_for_ceiling: true,
            collision_type,
            y_velocity,
        }
    }
}

pub(crate) fn drips_wait_timer_loc_3340e(random_byte: u8, kid_x_word: u16) -> u16 {
    // ASM Drips.asm loc_3340E: `andi.w #$FF,d7; asr.w #2,d7; move.w d7,d0`,
    // then `move.w x_pos(a4),d7; eor.b d7,d0`.
    let shifted = u16::from(random_byte) >> 2;
    let shifted_bytes = shifted.to_be_bytes();
    let kid_x_low_byte = kid_x_word.to_be_bytes()[1];
    u16::from_be_bytes([shifted_bytes[0], shifted_bytes[1] ^ kid_x_low_byte])
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct DripsSplashResetLoc33488 {
    pub(crate) drop_is_moved: bool,
    pub(crate) drop_status_flag: bool,
    pub(crate) drop_collision_type: u16,
    pub(crate) drop_y_pos: u32,
    pub(crate) surface_child_status_flag: bool,
    pub(crate) surface_child_x_pos: u32,
    pub(crate) surface_child_y_pos: u32,
}

pub(crate) fn drips_splash_reset_loc_33488(
    drop_x_pos: u32,
    drop_y_pos: u32,
) -> DripsSplashResetLoc33488 {
    // ASM Drips.asm loc_33488: clear `$14(a1)` and `collision_type(a1)`,
    // add `$A` to the integer Y word, then copy the drop position into `a2`.
    let adjusted_drop_y_pos = drop_y_pos.wrapping_add(0x000A_0000);
    DripsSplashResetLoc33488 {
        drop_is_moved: false,
        drop_status_flag: false,
        drop_collision_type: 0,
        drop_y_pos: adjusted_drop_y_pos,
        surface_child_status_flag: true,
        surface_child_x_pos: drop_x_pos,
        surface_child_y_pos: adjusted_drop_y_pos,
    }
}

pub(crate) fn drips_enter_splash_animation_loc_33488(
    enemy: &mut ActiveEnemy,
    drop_x_pos: u32,
    drop_y_pos: u32,
) {
    let splash_reset = drips_splash_reset_loc_33488(drop_x_pos, drop_y_pos);
    enemy.drips_pending_splash_x_pos = splash_reset.surface_child_x_pos;
    enemy.drips_pending_splash_y_pos = splash_reset.surface_child_y_pos;
    // ASM Drips.asm loc_33488 initialises `stru_33332`, hibernates until it
    // finishes, and only then advances `d3/a0` through `off_33520`.
    enemy.wait_timer =
        drips_splash_animation_timer_loc_33488(DRIPS_SPLASH_ANIMATION_FRAMES_LOC_33488);
}

#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn drips_surface_table_reset_loc_33506() -> (usize, u16) {
    // ASM Drips.asm loc_33506: when `off_33520` reaches its negative sentinel,
    // reload `a0` from the start of the table and clear `d7` before the surface
    // child sprite offset is written.
    (0, 0)
}

#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn drips_surface_sprite_write_loc_3350e(selected_sprite_offset: u16) -> u16 {
    // ASM Drips.asm loc_3350E: write the selected `d7` word to
    // `addroffset_sprite(a2)` for the surface child before returning to the
    // dormant delay loop.
    selected_sprite_offset
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) struct DripsIdleLoopLoc33516 {
    pub(crate) hibernates_one_frame: bool,
    pub(crate) checks_in_range: bool,
    pub(crate) loops_forever: bool,
}

#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn drips_idle_loop_loc_33516() -> DripsIdleLoopLoc33516 {
    // ASM Drips.asm loc_33516: call one-frame hibernate, run
    // Object_CheckInRange, then branch back to the same idle loop.
    DripsIdleLoopLoc33516 {
        hibernates_one_frame: true,
        checks_in_range: true,
        loops_forever: true,
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct DripsVelocityDeltasLoc33526 {
    pub(crate) x_delta: i32,
    pub(crate) y_delta: i32,
}

pub(crate) fn drips_velocity_deltas_loc_33526(
    child_byte_44: u8,
    child_byte_45: u8,
    x_velocity: i32,
    y_velocity: i32,
) -> DripsVelocityDeltasLoc33526 {
    // ASM Drips.asm loc_33526: clear d7/d6, only copy `x_vel(a3)` and
    // `y_vel(a3)` when `$44(a5)` is non-zero, then double both deltas when
    // `$45(a5)` is also non-zero.
    if child_byte_44 == 0 {
        DripsVelocityDeltasLoc33526 {
            x_delta: 0,
            y_delta: 0,
        }
    } else if child_byte_45 == 0 {
        DripsVelocityDeltasLoc33526 {
            x_delta: x_velocity,
            y_delta: y_velocity,
        }
    } else {
        DripsVelocityDeltasLoc33526 {
            x_delta: x_velocity.wrapping_mul(2),
            y_delta: y_velocity.wrapping_mul(2),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct DripsVelocityApplyLoc33546 {
    pub(crate) x_velocity: i32,
    pub(crate) y_velocity: i32,
}

pub(crate) fn drips_velocity_apply_loc_33546(
    x_velocity: i32,
    y_velocity: i32,
    x_delta: i32,
    y_delta: i32,
) -> DripsVelocityApplyLoc33546 {
    // ASM Drips.asm loc_33546: add d7/d6 back into the object's x/y
    // velocities as 32-bit values, preserving 68000 wrapping semantics.
    DripsVelocityApplyLoc33546 {
        x_velocity: x_velocity.wrapping_add(x_delta),
        y_velocity: y_velocity.wrapping_add(y_delta),
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) struct DripsCollisionDispatchLoc33550 {
    pub(crate) collision_type: u16,
    pub(crate) contact_flag: u16,
}

#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn drips_collision_dispatch_loc_33550(
    collision_type: u16,
    berzerker_charging: bool,
) -> DripsCollisionDispatchLoc33550 {
    // ASM Drips.asm loc_33550: zero collision returns d5=0 unchanged; negative
    // collision clears the word and returns d5=1. Positive collision clears the
    // word before dispatching collision IDs from `$1C` upward.
    if collision_type == 0 {
        return drips_collision_no_contact_result_loc_33584(collision_type);
    }
    if (collision_type & 0x8000) != 0 {
        return drips_collision_contact_result_loc_33576();
    }

    let adjusted = collision_type.wrapping_sub(0x001C);
    if (adjusted & 0x8000) != 0 {
        return drips_collision_no_contact_result_loc_33584(0);
    }

    let contact_flag = match collision_type {
        0x001C => drips_collision_table_first_contact_loc_3356e(),
        0x002C => drips_collision_contact_result_loc_33576().contact_flag,
        0x0020 | 0x0024 if berzerker_charging => {
            drips_collision_contact_result_loc_33576().contact_flag
        }
        _ => drips_collision_no_contact_result_loc_33584(0).contact_flag,
    };
    DripsCollisionDispatchLoc33550 {
        collision_type: 0,
        contact_flag,
    }
}

#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn drips_collision_table_first_contact_loc_3356e() -> u16 {
    // ASM Drips.asm loc_3356E: the first dispatch-table branch is the
    // `$1C` collision slot and resolves to a contact result.
    1
}

#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn drips_collision_contact_result_loc_33576() -> DripsCollisionDispatchLoc33550 {
    // ASM Drips.asm loc_33576: clear `collision_type(a3)`, set d5 to 1,
    // and return to the caller.
    DripsCollisionDispatchLoc33550 {
        collision_type: 0,
        contact_flag: 1,
    }
}

#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn drips_collision_no_contact_result_loc_33584(
    collision_type: u16,
) -> DripsCollisionDispatchLoc33550 {
    // ASM Drips.asm loc_33584: move 0 into d5 and return; callers have
    // already decided whether `collision_type(a3)` should be preserved or clear.
    DripsCollisionDispatchLoc33550 {
        collision_type,
        contact_flag: 0,
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum DripsRightBoundsNextLoc33588 {
    ContactLoc335D4,
    CheckLeftLoc335AA,
    CheckVerticalLoc335B0,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum DripsLeftBoundsNextLoc335AA {
    ContactLoc335D4,
    CheckVerticalLoc335B0,
}

pub(crate) fn drips_right_bounds_gate_loc_33588(
    has_level_collision_byte_19: bool,
    x_pos_word: i16,
    x_velocity: i32,
    level_width_pixels: i16,
) -> DripsRightBoundsNextLoc33588 {
    // ASM Drips.asm loc_33588: `$19(a3)` short-circuits to d5=1. Non-negative
    // `x_vel(a3)` probes `x_pos(a3)+8` against `Level_width_pixels`; negative
    // X velocity falls through to the left-edge gate at loc_335AA.
    if has_level_collision_byte_19 {
        return DripsRightBoundsNextLoc33588::ContactLoc335D4;
    }
    if x_velocity < 0 {
        return DripsRightBoundsNextLoc33588::CheckLeftLoc335AA;
    }

    if x_pos_word.wrapping_add(8) >= level_width_pixels {
        DripsRightBoundsNextLoc33588::ContactLoc335D4
    } else {
        DripsRightBoundsNextLoc33588::CheckVerticalLoc335B0
    }
}

pub(crate) fn drips_left_bounds_gate_loc_335aa(x_pos_word: i16) -> DripsLeftBoundsNextLoc335AA {
    // ASM Drips.asm loc_335AA: subtract 8 from `x_pos(a3)` and branch to
    // d5=1 when the signed word result is negative; otherwise continue to the
    // vertical gate at loc_335B0.
    if x_pos_word.wrapping_sub(8) < 0 {
        DripsLeftBoundsNextLoc335AA::ContactLoc335D4
    } else {
        DripsLeftBoundsNextLoc335AA::CheckVerticalLoc335B0
    }
}

pub(crate) fn drips_vertical_bounds_contact_flag_loc_335b0(
    y_pos_word: i16,
    y_velocity: i32,
    level_height_pixels: i16,
) -> u16 {
    // ASM Drips.asm loc_335B0: non-negative `y_vel(a3)` probes `y_pos(a3)+8`
    // against `Level_height_pixels`; negative Y velocity probes `y_pos(a3)-8`
    // and treats a signed-negative word as d5=1.
    let out_of_bounds = if y_velocity < 0 {
        y_pos_word.wrapping_sub(8) < 0
    } else {
        y_pos_word.wrapping_add(8) >= level_height_pixels
    };
    if out_of_bounds {
        drips_bounds_contact_result_loc_335d4()
    } else {
        drips_bounds_no_contact_result_loc_335d0()
    }
}

pub(crate) fn drips_bounds_no_contact_result_loc_335d0() -> u16 {
    // ASM Drips.asm loc_335D0: the in-bounds return tail moves 0 into d5.
    0
}

pub(crate) fn drips_bounds_contact_result_loc_335d4() -> u16 {
    // ASM Drips.asm loc_335D4: the out-of-bounds return tail moves 1 into d5.
    1
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct DripsSurfaceCrawlerInitLoc335D8 {
    pub(crate) status_flag_13: bool,
    pub(crate) priority: u8,
    pub(crate) object_meta: u16,
    pub(crate) x_pos_word: u16,
    pub(crate) y_pos_word: u16,
    pub(crate) child_byte_44: u8,
    pub(crate) child_byte_45: u8,
    pub(crate) x_direction: u8,
    pub(crate) x_velocity: i32,
    pub(crate) startup_animation: &'static str,
}

pub(crate) fn drips_surface_crawler_init_loc_335d8(
    queued_x_word: u16,
    queued_y_word: u16,
    parent_child_byte_44: u8,
    parent_child_byte_45: u8,
    random_byte: u8,
) -> DripsSurfaceCrawlerInitLoc335D8 {
    // ASM Drips.asm loc_335D8: allocate a visible low-priority object_meta `$16`
    // child at queued `$44/$46`, copy the parent drip state bytes, seed x_vel
    // with `$3000`, then `bclr #2,d7` decides whether to flip x_direction and
    // negate the velocity before installing `stru_33FA6`.
    let random_bit_2_was_set = (random_byte & 0x04) != 0;
    DripsSurfaceCrawlerInitLoc335D8 {
        status_flag_13: true,
        priority: 0,
        object_meta: 0x0016,
        x_pos_word: queued_x_word,
        y_pos_word: queued_y_word,
        child_byte_44: parent_child_byte_44,
        child_byte_45: parent_child_byte_45,
        x_direction: if random_bit_2_was_set { 0xFF } else { 0x00 },
        x_velocity: if random_bit_2_was_set {
            -0x0000_3000
        } else {
            0x0000_3000
        },
        startup_animation: "stru_33FA6",
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) struct DripsSurfaceCrawlerAnimationTailLoc33630 {
    pub(crate) first_animation: &'static str,
    pub(crate) second_animation: &'static str,
    pub(crate) is_moved_set: bool,
    pub(crate) deltas: DripsVelocityDeltasLoc33526,
}

#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn drips_surface_crawler_animation_tail_loc_33630(
    child_byte_44: u8,
    child_byte_45: u8,
    x_velocity: i32,
    y_velocity: i32,
) -> DripsSurfaceCrawlerAnimationTailLoc33630 {
    // ASM Drips.asm loc_33630: run `stru_33FA6`, wait, run `stru_33F14`,
    // set `is_moved`, and then apply Drips velocity deltas via loc_33526.
    DripsSurfaceCrawlerAnimationTailLoc33630 {
        first_animation: "stru_33FA6",
        second_animation: "stru_33F14",
        is_moved_set: true,
        deltas: drips_velocity_deltas_loc_33526(
            child_byte_44,
            child_byte_45,
            x_velocity,
            y_velocity,
        ),
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[allow(dead_code)]
pub(crate) enum DripsSurfaceCrawlerMovementPathLoc33650 {
    Loc338F8,
    Loc338EA,
    Loc336AA,
    Loc3393C,
    Loc33A40,
    Loc33A2C,
    Loc33982,
    Loc33650,
    Loc339DC,
    Loc33996,
    Loc33A18,
    Loc339C8,
}

#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn drips_surface_crawler_movement_tail_loc_33650(
    collision_type: u16,
    berzerker_charging: bool,
    probe_331d2: u16,
    bounds_contact: bool,
    is_x_direction_left: bool,
    tile_flags_331f6_d5: u16,
    tile_flags_331f6_d6: u16,
) -> DripsSurfaceCrawlerMovementPathLoc33650 {
    // ASM Drips.asm loc_33650: after one frame, collision and boundary probes,
    // clear-contact guards route either to loc_338F8/loc_338EA; then branch to
    // `loc_336AA` for left-facing crawlers, otherwise interpret `loc_331F6`
    // returns into loc_3393C / loc_33A40 / loc_33650 / loc_33A2C / loc_33982.
    if drips_collision_dispatch_loc_33550(collision_type, berzerker_charging).contact_flag != 0 {
        return DripsSurfaceCrawlerMovementPathLoc33650::Loc338F8;
    }
    if probe_331d2 != 0x6000 {
        return DripsSurfaceCrawlerMovementPathLoc33650::Loc338F8;
    }
    if bounds_contact {
        return DripsSurfaceCrawlerMovementPathLoc33650::Loc338EA;
    }
    if is_x_direction_left {
        return DripsSurfaceCrawlerMovementPathLoc33650::Loc336AA;
    }
    if tile_flags_331f6_d6 == 0x6000 {
        return DripsSurfaceCrawlerMovementPathLoc33650::Loc3393C;
    }
    if tile_flags_331f6_d6 == 0x5000 {
        return DripsSurfaceCrawlerMovementPathLoc33650::Loc33A40;
    }
    if tile_flags_331f6_d5 == 0x6000 {
        return DripsSurfaceCrawlerMovementPathLoc33650::Loc33650;
    }
    if tile_flags_331f6_d5 == 0x4000 {
        return DripsSurfaceCrawlerMovementPathLoc33650::Loc33A2C;
    }
    DripsSurfaceCrawlerMovementPathLoc33650::Loc33982
}

#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn drips_surface_crawler_movement_tail_loc_336aa(
    tile_flags_331f6_d5: u16,
    tile_flags_331f6_d6: u16,
) -> DripsSurfaceCrawlerMovementPathLoc33650 {
    // ASM Drips.asm loc_336AA: interpret `loc_331F6` returns after
    // right-facing branch of loc_33650.
    if tile_flags_331f6_d6 == 0x4000 {
        return DripsSurfaceCrawlerMovementPathLoc33650::Loc339DC;
    }
    if tile_flags_331f6_d6 == 0x6000 {
        return DripsSurfaceCrawlerMovementPathLoc33650::Loc33996;
    }
    if tile_flags_331f6_d5 == 0x6000 {
        return DripsSurfaceCrawlerMovementPathLoc33650::Loc33650;
    }
    if tile_flags_331f6_d5 == 0x5000 {
        return DripsSurfaceCrawlerMovementPathLoc33650::Loc33A18;
    }
    DripsSurfaceCrawlerMovementPathLoc33650::Loc339C8
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[allow(dead_code)]
pub(crate) enum DripsSurfaceCrawlerMovementPathLoc3371A {
    Loc339B4,
    Loc336D0,
    Loc339AA,
}

#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn drips_surface_crawler_movement_tail_loc_3371a(
    tile_flags_331f6_d5: u16,
    tile_flags_331f6_d6: u16,
) -> DripsSurfaceCrawlerMovementPathLoc3371A {
    // ASM Drips.asm loc_3371A: right-facing branch from loc_336D0.
    // `$331F6` returns `d5` and `d6`, where `$6000` routes to the
    // collision branch targets and `Loc336D0` loops after `beq.s loc_336D0`.
    if tile_flags_331f6_d5 == 0x6000 {
        DripsSurfaceCrawlerMovementPathLoc3371A::Loc339B4
    } else if tile_flags_331f6_d6 == 0x6000 {
        DripsSurfaceCrawlerMovementPathLoc3371A::Loc336D0
    } else {
        DripsSurfaceCrawlerMovementPathLoc3371A::Loc339AA
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[allow(dead_code)]
pub(crate) enum DripsSurfaceCrawlerMovementPathLoc336D0 {
    Loc338F8,
    Loc3396E,
    Loc336D0,
    Loc339B4,
    Loc339AA,
    Loc33950,
}

#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn drips_surface_crawler_movement_tail_loc_336d0(
    has_collision: bool,
    probe_is_solid: bool,
    bounds_contact: bool,
    is_x_direction_left: bool,
    tile_flags_331f6_d5: u16,
    tile_flags_331f6_d6: u16,
) -> DripsSurfaceCrawlerMovementPathLoc336D0 {
    // ASM Drips.asm loc_336D0: common movement-tick gate for crawl children.
    // This routine first runs the shared loc_33550 -> loc_331D2 -> loc_33588 path;
    // failures fall through to `loc_338F8`.
    if has_collision || !probe_is_solid || bounds_contact {
        return DripsSurfaceCrawlerMovementPathLoc336D0::Loc338F8;
    }

    // `x_direction(a3)==0` stays on this routine and uses `-6` for the probe;
    // non-zero is the `loc_3371A` right-facing branch.
    if !is_x_direction_left {
        return match drips_surface_crawler_movement_tail_loc_3371a(
            tile_flags_331f6_d5,
            tile_flags_331f6_d6,
        ) {
            DripsSurfaceCrawlerMovementPathLoc3371A::Loc339B4 => {
                DripsSurfaceCrawlerMovementPathLoc336D0::Loc339B4
            }
            DripsSurfaceCrawlerMovementPathLoc3371A::Loc336D0 => {
                DripsSurfaceCrawlerMovementPathLoc336D0::Loc336D0
            }
            DripsSurfaceCrawlerMovementPathLoc3371A::Loc339AA => {
                DripsSurfaceCrawlerMovementPathLoc336D0::Loc339AA
            }
        };
    }

    if tile_flags_331f6_d5 == 0x6000 {
        DripsSurfaceCrawlerMovementPathLoc336D0::Loc336D0
    } else if tile_flags_331f6_d6 == 0x6000 {
        DripsSurfaceCrawlerMovementPathLoc336D0::Loc3396E
    } else {
        DripsSurfaceCrawlerMovementPathLoc336D0::Loc33950
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[allow(dead_code)]
pub(crate) enum DripsSurfaceCrawlerMovementPathLoc33730 {
    Loc338F8,
    Loc33964,
    Loc33730,
    Loc339FA,
    Loc339D2,
    Loc33946,
    Loc33A72,
    Loc339A0,
}

#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn drips_surface_crawler_movement_tail_loc_33730(
    has_collision: bool,
    probe_is_solid: bool,
    bounds_contact: bool,
    is_falling: bool,
    tile_flags_33286_d5: u16,
    tile_flags_33286_d6: u16,
) -> DripsSurfaceCrawlerMovementPathLoc33730 {
    // ASM Drips.asm loc_33730: vertical crawl-turn routine.
    // Shared guards in this label match loc_336D0 style and return `loc_338F8` on
    // any early collision or boundary failure.
    if has_collision || !probe_is_solid || bounds_contact {
        return DripsSurfaceCrawlerMovementPathLoc33730::Loc338F8;
    }

    // `y_vel(a3)` positive/zero takes the `loc_33782` negative-branch.
    if is_falling {
        if tile_flags_33286_d6 == 0x6000 {
            return DripsSurfaceCrawlerMovementPathLoc33730::Loc33964;
        }
        if tile_flags_33286_d5 == 0x6000 {
            return DripsSurfaceCrawlerMovementPathLoc33730::Loc33730;
        }
        if tile_flags_33286_d5 == 0x5000 {
            return DripsSurfaceCrawlerMovementPathLoc33730::Loc339FA;
        }
        return DripsSurfaceCrawlerMovementPathLoc33730::Loc339D2;
    }

    if tile_flags_33286_d6 == 0x6000 {
        DripsSurfaceCrawlerMovementPathLoc33730::Loc33946
    } else if tile_flags_33286_d5 == 0x5000 {
        DripsSurfaceCrawlerMovementPathLoc33730::Loc33A72
    } else if tile_flags_33286_d5 == 0x6000 {
        DripsSurfaceCrawlerMovementPathLoc33730::Loc33730
    } else {
        DripsSurfaceCrawlerMovementPathLoc33730::Loc339A0
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[allow(dead_code)]
pub(crate) enum DripsSurfaceCrawlerMovementPathLoc337A0 {
    GuardExit,
    NonNegativeYVelocityBranch,
    SolidD5Turn,
    Loop,
    SolidD6Corner,
    FallbackTurn,
}

#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn drips_surface_crawler_movement_tail_loc_337a0(
    has_collision: bool,
    probe_is_solid: bool,
    bounds_contact: bool,
    y_velocity_is_negative: bool,
    tile_flags_33286_d5: u16,
    tile_flags_33286_d6: u16,
) -> DripsSurfaceCrawlerMovementPathLoc337A0 {
    // ASM Drips.asm loc_337A0: left-side vertical turn routine. The shared
    // collision, right-probe, and bounds guards all take the animation-exit
    // path; non-negative y velocity branches to the next label before this
    // routine interprets `loc_33286` tile flags.
    if has_collision || !probe_is_solid || bounds_contact {
        return DripsSurfaceCrawlerMovementPathLoc337A0::GuardExit;
    }
    if !y_velocity_is_negative {
        return DripsSurfaceCrawlerMovementPathLoc337A0::NonNegativeYVelocityBranch;
    }
    if tile_flags_33286_d5 == 0x6000 {
        return DripsSurfaceCrawlerMovementPathLoc337A0::SolidD5Turn;
    }
    if tile_flags_33286_d6 == 0x6000 {
        return DripsSurfaceCrawlerMovementPathLoc337A0::Loop;
    }
    if tile_flags_33286_d6 == 0x4000 {
        return DripsSurfaceCrawlerMovementPathLoc337A0::SolidD6Corner;
    }
    DripsSurfaceCrawlerMovementPathLoc337A0::FallbackTurn
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[allow(dead_code)]
pub(crate) enum DripsSurfaceCrawlerMovementPathLoc337F2 {
    SolidD5Turn,
    SolidD5Corner,
    LoopToEntry,
    FallbackTurn,
}

#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn drips_surface_crawler_movement_tail_loc_337f2(
    tile_flags_33286_d5: u16,
    tile_flags_33286_d6: u16,
) -> DripsSurfaceCrawlerMovementPathLoc337F2 {
    // ASM Drips.asm loc_337F2: non-negative y-velocity continuation from the
    // left-side vertical turn. The routine checks `loc_33286` d5 before d6, so
    // a d5 solid flag wins over the entry-loop d6 flag.
    if tile_flags_33286_d5 == 0x6000 {
        return DripsSurfaceCrawlerMovementPathLoc337F2::SolidD5Turn;
    }
    if tile_flags_33286_d5 == 0x4000 {
        return DripsSurfaceCrawlerMovementPathLoc337F2::SolidD5Corner;
    }
    if tile_flags_33286_d6 == 0x6000 {
        return DripsSurfaceCrawlerMovementPathLoc337F2::LoopToEntry;
    }
    DripsSurfaceCrawlerMovementPathLoc337F2::FallbackTurn
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) struct DripsSurfaceCrawlerIdleLoopLoc33866 {
    pub(crate) hibernates_one_frame: bool,
    pub(crate) loops_forever: bool,
}

#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn drips_surface_crawler_idle_loop_loc_33866() -> DripsSurfaceCrawlerIdleLoopLoc33866 {
    // ASM Drips.asm loc_33866: hibernate the surface crawler for one frame,
    // then branch back to loc_33866 unconditionally.
    DripsSurfaceCrawlerIdleLoopLoc33866 {
        hibernates_one_frame: true,
        loops_forever: true,
    }
}

#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn drips_surface_crawler_idle_loop_loc_33a7c() -> DripsSurfaceCrawlerIdleLoopLoc33866 {
    // ASM Drips.asm loc_33A7C: call `j_Hibernate_Object_1Frame`, then
    // `bra.s loc_33A7C`; this is the parked surface-crawler loop.
    DripsSurfaceCrawlerIdleLoopLoc33866 {
        hibernates_one_frame: true,
        loops_forever: true,
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) enum DripsSurfaceCrawlerMovementPathLoc33878 {
    DiagonalLoop,
    SolidTurn,
    FallbackTurn,
}

#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn drips_surface_crawler_movement_tail_loc_33878(
    tile_flags_3330e_d7: u16,
) -> DripsSurfaceCrawlerMovementPathLoc33878 {
    // ASM Drips.asm loc_33878: seed `d6=-5`, call loc_3330E, loop to loc_33810
    // on `$4000`, route `$6000` to loc_33A36, otherwise fall through loc_33A0E.
    if tile_flags_3330e_d7 == 0x4000 {
        return DripsSurfaceCrawlerMovementPathLoc33878::DiagonalLoop;
    }
    if tile_flags_3330e_d7 == 0x6000 {
        return DripsSurfaceCrawlerMovementPathLoc33878::SolidTurn;
    }
    DripsSurfaceCrawlerMovementPathLoc33878::FallbackTurn
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[allow(dead_code)]
pub(crate) enum DripsSurfaceCrawlerMovementPathLoc33810 {
    GuardExit,
    AlignmentLoop,
    NegativeCornerLoop,
    NegativeSolidTurn,
    NegativeFallbackTurn,
    NonNegativeCornerLoop,
    NonNegativeSolidTurn,
    NonNegativeFallbackTurn,
}

#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn drips_surface_crawler_movement_tail_loc_33810(
    has_collision: bool,
    bounds_contact: bool,
    x_pos_word: u16,
    y_pos_word: u16,
    y_velocity_is_negative: bool,
    tile_flags_3330e_d7: u16,
) -> DripsSurfaceCrawlerMovementPathLoc33810 {
    // ASM Drips.asm loc_33810: diagonal surface crawl waits until the low
    // nibbles of x/y integer position sum to `$F`, then interprets the
    // `loc_3330E` d7 probe with different branch targets for negative and
    // non-negative y velocity.
    if has_collision || bounds_contact {
        return DripsSurfaceCrawlerMovementPathLoc33810::GuardExit;
    }
    if (x_pos_word & 0x000F) + (y_pos_word & 0x000F) != 0x000F {
        return DripsSurfaceCrawlerMovementPathLoc33810::AlignmentLoop;
    }
    if y_velocity_is_negative {
        if tile_flags_3330e_d7 == 0x4000 {
            return DripsSurfaceCrawlerMovementPathLoc33810::NegativeCornerLoop;
        }
        if tile_flags_3330e_d7 == 0x6000 {
            return DripsSurfaceCrawlerMovementPathLoc33810::NegativeSolidTurn;
        }
        return DripsSurfaceCrawlerMovementPathLoc33810::NegativeFallbackTurn;
    }
    match drips_surface_crawler_movement_tail_loc_33878(tile_flags_3330e_d7) {
        DripsSurfaceCrawlerMovementPathLoc33878::DiagonalLoop => {
            DripsSurfaceCrawlerMovementPathLoc33810::NonNegativeCornerLoop
        }
        DripsSurfaceCrawlerMovementPathLoc33878::SolidTurn => {
            DripsSurfaceCrawlerMovementPathLoc33810::NonNegativeSolidTurn
        }
        DripsSurfaceCrawlerMovementPathLoc33878::FallbackTurn => {
            DripsSurfaceCrawlerMovementPathLoc33810::NonNegativeFallbackTurn
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) enum DripsSurfaceCrawlerMovementPathLoc33892 {
    GuardExit,
    NegativeYBranch,
    DiagonalLoop,
    SolidTurn,
    FallbackTurn,
}

#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn drips_surface_crawler_movement_tail_loc_33892(
    has_collision: bool,
    bounds_contact: bool,
    y_velocity_is_negative: bool,
    tile_flags_3330e_d7: u16,
) -> DripsSurfaceCrawlerMovementPathLoc33892 {
    // ASM Drips.asm loc_33892: after hibernating, loc_33550/loc_33588 guard
    // exits go to loc_338F8. Negative y velocity branches to loc_338CE;
    // otherwise loc_3330E loops on d7 `$5000`, turns on `$6000`, and falls
    // through loc_339F0 for non-matching flags.
    if has_collision || bounds_contact {
        return DripsSurfaceCrawlerMovementPathLoc33892::GuardExit;
    }
    if y_velocity_is_negative {
        return DripsSurfaceCrawlerMovementPathLoc33892::NegativeYBranch;
    }
    if tile_flags_3330e_d7 == 0x5000 {
        return DripsSurfaceCrawlerMovementPathLoc33892::DiagonalLoop;
    }
    if tile_flags_3330e_d7 == 0x6000 {
        return DripsSurfaceCrawlerMovementPathLoc33892::SolidTurn;
    }
    DripsSurfaceCrawlerMovementPathLoc33892::FallbackTurn
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) struct DripsSurfaceCrawlerDeleteTailLoc3392C {
    pub(crate) is_moved: bool,
    pub(crate) waits_for_animation_to_finish: bool,
    pub(crate) current_drips_delta: i16,
    pub(crate) deletes_current_object: bool,
}

#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn drips_surface_crawler_delete_tail_loc_3392c() -> DripsSurfaceCrawlerDeleteTailLoc3392C
{
    // ASM Drips.asm loc_3392C: clear `is_moved(a3)`, wait for the current
    // animation to finish, decrement `$FFFFFB42`, then delete this crawler.
    DripsSurfaceCrawlerDeleteTailLoc3392C {
        is_moved: false,
        waits_for_animation_to_finish: true,
        current_drips_delta: -1,
        deletes_current_object: true,
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum DripsSurfaceCrawlerTurnFieldLoc33AF6 {
    XPos,
    YPos,
    Unknown(u8),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum DripsSurfaceCrawlerTurnAnimationLoc33AF6 {
    Stru33E10,
    Stru33E1E,
    Stru33E2C,
    Stru33E3A,
    Stru33E80,
    Stru33EB2,
    Stru33F14,
    Stru33F52,
    Stru33E48,
    Stru33DCA,
    Stru33E56,
    Stru33E64,
    Stru33E72,
    Stru33EE6,
    Stru33DA0,
    Stru33EF6,
    Stru33DF4,
    Stru33E02,
    Stru33DAE,
    Stru33DBC,
    Stru33DD8,
    Stru33DE6,
    Unknown(u8),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum DripsSurfaceCrawlerTurnNextLoc33AF6 {
    Loc33650,
    Loc336D0,
    Loc33730,
    Loc337A0,
    Loc33A7C,
    Loc33810,
    Loc33892,
    Unknown(u8),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct DripsSurfaceCrawlerTurnScriptLoc33AF6 {
    pub(crate) snap_field: DripsSurfaceCrawlerTurnFieldLoc33AF6,
    pub(crate) snap_offset: i16,
    pub(crate) snap_mask: u16,
    pub(crate) flip_before_first_animation: bool,
    pub(crate) first_x_delta: i16,
    pub(crate) first_y_delta: i16,
    pub(crate) first_animation: DripsSurfaceCrawlerTurnAnimationLoc33AF6,
    pub(crate) flip_after_first_animation: bool,
    pub(crate) second_x_delta: i16,
    pub(crate) second_y_delta: i16,
    pub(crate) x_velocity_delta: i32,
    pub(crate) y_velocity_delta: i32,
    pub(crate) second_animation: DripsSurfaceCrawlerTurnAnimationLoc33AF6,
    pub(crate) next: DripsSurfaceCrawlerTurnNextLoc33AF6,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct DripsSurfaceCrawlerFirstAnimationLoc33B30 {
    pub(crate) x_delta: i16,
    pub(crate) y_delta: i16,
    pub(crate) animation: DripsSurfaceCrawlerTurnAnimationLoc33AF6,
    pub(crate) waits_for_animation_to_finish: bool,
    pub(crate) flip_after_animation: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct DripsSurfaceCrawlerSecondAnimationLoc33B5E {
    pub(crate) x_delta: i16,
    pub(crate) y_delta: i16,
    pub(crate) x_velocity_delta: i32,
    pub(crate) y_velocity_delta: i32,
    pub(crate) animation: DripsSurfaceCrawlerTurnAnimationLoc33AF6,
    pub(crate) applies_child_velocity_deltas: bool,
    pub(crate) next: DripsSurfaceCrawlerTurnNextLoc33AF6,
}

fn drips_turn_script_i8(byte: u8) -> i16 {
    i16::from(i8::from_ne_bytes([byte]))
}

fn drips_turn_script_i16(hi: u8, lo: u8) -> i32 {
    i32::from(i16::from_be_bytes([hi, lo]))
}

fn drips_turn_script_field_loc_33af6(byte: u8) -> DripsSurfaceCrawlerTurnFieldLoc33AF6 {
    match byte {
        0x1A => DripsSurfaceCrawlerTurnFieldLoc33AF6::XPos,
        0x1E => DripsSurfaceCrawlerTurnFieldLoc33AF6::YPos,
        _ => DripsSurfaceCrawlerTurnFieldLoc33AF6::Unknown(byte),
    }
}

fn drips_turn_script_animation_loc_33af6(
    byte_offset: u8,
) -> DripsSurfaceCrawlerTurnAnimationLoc33AF6 {
    match byte_offset {
        0x00 => DripsSurfaceCrawlerTurnAnimationLoc33AF6::Stru33E10,
        0x04 => DripsSurfaceCrawlerTurnAnimationLoc33AF6::Stru33E1E,
        0x08 => DripsSurfaceCrawlerTurnAnimationLoc33AF6::Stru33E2C,
        0x0C => DripsSurfaceCrawlerTurnAnimationLoc33AF6::Stru33E3A,
        0x10 => DripsSurfaceCrawlerTurnAnimationLoc33AF6::Stru33E80,
        0x14 => DripsSurfaceCrawlerTurnAnimationLoc33AF6::Stru33EB2,
        0x18 => DripsSurfaceCrawlerTurnAnimationLoc33AF6::Stru33F14,
        0x1C => DripsSurfaceCrawlerTurnAnimationLoc33AF6::Stru33F52,
        0x20 => DripsSurfaceCrawlerTurnAnimationLoc33AF6::Stru33E48,
        0x48 => DripsSurfaceCrawlerTurnAnimationLoc33AF6::Stru33DCA,
        0x24 => DripsSurfaceCrawlerTurnAnimationLoc33AF6::Stru33E56,
        0x28 => DripsSurfaceCrawlerTurnAnimationLoc33AF6::Stru33E64,
        0x2C => DripsSurfaceCrawlerTurnAnimationLoc33AF6::Stru33E72,
        0x30 => DripsSurfaceCrawlerTurnAnimationLoc33AF6::Stru33EE6,
        0x34 => DripsSurfaceCrawlerTurnAnimationLoc33AF6::Stru33DA0,
        0x54 => DripsSurfaceCrawlerTurnAnimationLoc33AF6::Stru33EF6,
        0x38 => DripsSurfaceCrawlerTurnAnimationLoc33AF6::Stru33DF4,
        0x3C => DripsSurfaceCrawlerTurnAnimationLoc33AF6::Stru33E02,
        0x40 => DripsSurfaceCrawlerTurnAnimationLoc33AF6::Stru33DAE,
        0x44 => DripsSurfaceCrawlerTurnAnimationLoc33AF6::Stru33DBC,
        0x4C => DripsSurfaceCrawlerTurnAnimationLoc33AF6::Stru33DD8,
        0x50 => DripsSurfaceCrawlerTurnAnimationLoc33AF6::Stru33DE6,
        _ => DripsSurfaceCrawlerTurnAnimationLoc33AF6::Unknown(byte_offset),
    }
}

fn drips_turn_script_next_loc_33af6(byte_offset: u8) -> DripsSurfaceCrawlerTurnNextLoc33AF6 {
    match byte_offset {
        0x00 => DripsSurfaceCrawlerTurnNextLoc33AF6::Loc33650,
        0x04 => DripsSurfaceCrawlerTurnNextLoc33AF6::Loc336D0,
        0x08 => DripsSurfaceCrawlerTurnNextLoc33AF6::Loc33730,
        0x0C => DripsSurfaceCrawlerTurnNextLoc33AF6::Loc337A0,
        0x10 => DripsSurfaceCrawlerTurnNextLoc33AF6::Loc33A7C,
        0x14 => DripsSurfaceCrawlerTurnNextLoc33AF6::Loc33810,
        0x18 => DripsSurfaceCrawlerTurnNextLoc33AF6::Loc33892,
        _ => DripsSurfaceCrawlerTurnNextLoc33AF6::Unknown(byte_offset),
    }
}

pub(crate) fn drips_surface_crawler_first_animation_loc_33b30(
    table: [u8; 4],
) -> DripsSurfaceCrawlerFirstAnimationLoc33B30 {
    // ASM Drips.asm loc_33B30: sign-extend the two byte deltas into x/y
    // position words, initialise the selected animation, hibernate until it
    // finishes, then optionally `not.b x_direction(a3)`.
    DripsSurfaceCrawlerFirstAnimationLoc33B30 {
        x_delta: drips_turn_script_i8(table[0]),
        y_delta: drips_turn_script_i8(table[1]),
        animation: drips_turn_script_animation_loc_33af6(table[2]),
        waits_for_animation_to_finish: true,
        flip_after_animation: table[3] != 0,
    }
}

pub(crate) fn drips_surface_crawler_second_animation_loc_33b5e(
    table: [u8; 8],
) -> DripsSurfaceCrawlerSecondAnimationLoc33B5E {
    // ASM Drips.asm loc_33B5E: sign-extend the second byte x/y deltas, add the
    // signed word velocity deltas, initialise animation, run `loc_33526`, then
    // jump through `off_33ADA`.
    DripsSurfaceCrawlerSecondAnimationLoc33B5E {
        x_delta: drips_turn_script_i8(table[0]),
        y_delta: drips_turn_script_i8(table[1]),
        x_velocity_delta: drips_turn_script_i16(table[2], table[3]),
        y_velocity_delta: drips_turn_script_i16(table[4], table[5]),
        animation: drips_turn_script_animation_loc_33af6(table[6]),
        applies_child_velocity_deltas: true,
        next: drips_turn_script_next_loc_33af6(table[7]),
    }
}

fn drips_surface_crawler_turn_script_loc_33af6(
    table: [u8; 16],
) -> DripsSurfaceCrawlerTurnScriptLoc33AF6 {
    // ASM Drips.asm loc_33AF6: zero velocity/fractional position, snap the
    // selected x/y word through `andi.w #$FFF0`, run one animation, then apply
    // the second movement/velocity tuple before jumping through `off_33ADA`.
    let first_animation =
        drips_surface_crawler_first_animation_loc_33b30([table[4], table[5], table[6], table[7]]);
    let second_animation = drips_surface_crawler_second_animation_loc_33b5e([
        table[8], table[9], table[10], table[11], table[12], table[13], table[14], table[15],
    ]);
    DripsSurfaceCrawlerTurnScriptLoc33AF6 {
        snap_field: drips_turn_script_field_loc_33af6(table[0]),
        snap_offset: drips_turn_script_i8(table[1]) + drips_turn_script_i8(table[2]),
        snap_mask: 0xFFF0,
        flip_before_first_animation: table[3] != 0,
        first_x_delta: first_animation.x_delta,
        first_y_delta: first_animation.y_delta,
        first_animation: first_animation.animation,
        flip_after_first_animation: first_animation.flip_after_animation,
        second_x_delta: second_animation.x_delta,
        second_y_delta: second_animation.y_delta,
        x_velocity_delta: second_animation.x_velocity_delta,
        y_velocity_delta: second_animation.y_velocity_delta,
        second_animation: second_animation.animation,
        next: second_animation.next,
    }
}

pub(crate) fn drips_surface_crawler_turn_script_loc_3395a() -> DripsSurfaceCrawlerTurnScriptLoc33AF6
{
    // ASM Drips.asm loc_3395A: select `unk_33C00`, then branch to the shared
    // loc_33AF6 turn-script interpreter.
    drips_surface_crawler_turn_script_loc_33af6([
        0x1E, 0x08, 0x01, 0x00, 0x01, 0x00, 0x08, 0x00, 0x05, 0x00, 0x30, 0x00, 0x00, 0x00, 0x1C,
        0x04,
    ])
}

pub(crate) fn drips_surface_crawler_turn_script_loc_33950() -> DripsSurfaceCrawlerTurnScriptLoc33AF6
{
    // ASM Drips.asm loc_33950: select `unk_33BC0`, then branch to the shared
    // loc_33AF6 turn-script interpreter.
    drips_surface_crawler_turn_script_loc_33af6([
        0x1A, 0x00, 0x00, 0xFF, 0x00, 0x00, 0x0C, 0x00, 0xFF, 0xFB, 0x00, 0x00, 0xD0, 0x00, 0x14,
        0x0C,
    ])
}

pub(crate) fn drips_surface_crawler_turn_script_loc_33946() -> DripsSurfaceCrawlerTurnScriptLoc33AF6
{
    // ASM Drips.asm loc_33946: select `unk_33C30`, then branch to the shared
    // loc_33AF6 turn-script interpreter.
    drips_surface_crawler_turn_script_loc_33af6([
        0x1E, 0x08, 0x00, 0xFF, 0x00, 0x00, 0x28, 0xFF, 0x05, 0x00, 0x30, 0x00, 0x00, 0x00, 0x18,
        0x00,
    ])
}

pub(crate) fn drips_surface_crawler_turn_script_loc_33964() -> DripsSurfaceCrawlerTurnScriptLoc33AF6
{
    // ASM Drips.asm loc_33964: select `unk_33C70`, then branch to the shared
    // loc_33AF6 turn-script interpreter.
    drips_surface_crawler_turn_script_loc_33af6([
        0x1E, 0x00, 0x00, 0xFF, 0x00, 0x01, 0x24, 0x00, 0x05, 0x00, 0x30, 0x00, 0x00, 0x00, 0x1C,
        0x04,
    ])
}

pub(crate) fn drips_surface_crawler_turn_script_loc_3396e() -> DripsSurfaceCrawlerTurnScriptLoc33AF6
{
    // ASM Drips.asm loc_3396E: select `unk_33C40`, then branch to the shared
    // loc_33AF6 turn-script interpreter.
    drips_surface_crawler_turn_script_loc_33af6([
        0x1A, 0x00, 0x00, 0xFF, 0x01, 0x00, 0x20, 0xFF, 0x00, 0x05, 0x00, 0x00, 0x30, 0x00, 0x10,
        0x08,
    ])
}

pub(crate) fn drips_surface_crawler_turn_script_loc_33978() -> DripsSurfaceCrawlerTurnScriptLoc33AF6
{
    // ASM Drips.asm loc_33978: select `unk_33BD0`, then branch to the shared
    // loc_33AF6 turn-script interpreter.
    drips_surface_crawler_turn_script_loc_33af6([
        0x1E, 0x00, 0x00, 0x00, 0x01, 0x00, 0x04, 0xFF, 0x05, 0x00, 0x30, 0x00, 0x00, 0x00, 0x18,
        0x00,
    ])
}

pub(crate) fn drips_surface_crawler_turn_script_loc_3398c() -> DripsSurfaceCrawlerTurnScriptLoc33AF6
{
    // ASM Drips.asm loc_3398C: select `unk_33C90`, then branch to the shared
    // loc_33AF6 turn-script interpreter.
    drips_surface_crawler_turn_script_loc_33af6([
        0x1E, 0x08, 0x01, 0xFF, 0x00, 0xFF, 0x28, 0xFF, 0xFB, 0x00, 0xD0, 0x00, 0x00, 0x00, 0x18,
        0x00,
    ])
}

pub(crate) fn drips_surface_crawler_turn_script_loc_339a0() -> DripsSurfaceCrawlerTurnScriptLoc33AF6
{
    // ASM Drips.asm loc_339A0: select `unk_33BB0`, then branch to the shared
    // loc_33AF6 turn-script interpreter.
    drips_surface_crawler_turn_script_loc_33af6([
        0x1E, 0x08, 0x01, 0x00, 0xFF, 0x00, 0x08, 0x00, 0xFB, 0x00, 0xD0, 0x00, 0x00, 0x00, 0x1C,
        0x04,
    ])
}

pub(crate) fn drips_surface_crawler_turn_script_loc_339aa() -> DripsSurfaceCrawlerTurnScriptLoc33AF6
{
    // ASM Drips.asm loc_339AA: select `unk_33C10`, then branch to the shared
    // loc_33AF6 turn-script interpreter.
    drips_surface_crawler_turn_script_loc_33af6([
        0x1A, 0x08, 0x00, 0xFF, 0x00, 0x00, 0x0C, 0x00, 0x01, 0xFB, 0x00, 0x00, 0xD0, 0x00, 0x14,
        0x08,
    ])
}

pub(crate) fn drips_surface_crawler_turn_script_loc_339b4() -> DripsSurfaceCrawlerTurnScriptLoc33AF6
{
    // ASM Drips.asm loc_339B4: select `unk_33C60`, then branch to the shared
    // loc_33AF6 turn-script interpreter.
    drips_surface_crawler_turn_script_loc_33af6([
        0x1A, 0x08, 0x00, 0xFF, 0xFF, 0x00, 0x20, 0xFF, 0x00, 0x05, 0x00, 0x00, 0x30, 0x00, 0x10,
        0x0C,
    ])
}

pub(crate) fn drips_surface_crawler_turn_script_loc_339be() -> DripsSurfaceCrawlerTurnScriptLoc33AF6
{
    // ASM Drips.asm loc_339BE: select `unk_33C50`, then branch to the shared
    // loc_33AF6 turn-script interpreter.
    drips_surface_crawler_turn_script_loc_33af6([
        0x1E, 0x00, 0x00, 0xFF, 0x00, 0x01, 0x24, 0x00, 0xFB, 0x00, 0xD0, 0x00, 0x00, 0x00, 0x1C,
        0x04,
    ])
}

pub(crate) fn drips_surface_crawler_turn_script_loc_339c8() -> DripsSurfaceCrawlerTurnScriptLoc33AF6
{
    // ASM Drips.asm loc_339C8: select `unk_33BA0`, then branch to the shared
    // loc_33AF6 turn-script interpreter.
    drips_surface_crawler_turn_script_loc_33af6([
        0x1A, 0x08, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01, 0x05, 0x00, 0x00, 0x30, 0x00, 0x10,
        0x08,
    ])
}

pub(crate) fn drips_surface_crawler_turn_script_loc_33982() -> DripsSurfaceCrawlerTurnScriptLoc33AF6
{
    // ASM Drips.asm loc_33982: select `unk_33BF0`, then branch to the shared
    // loc_33AF6 turn-script interpreter.
    drips_surface_crawler_turn_script_loc_33af6([
        0x1A, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xFF, 0x05, 0x00, 0x00, 0x30, 0x00, 0x10,
        0x0C,
    ])
}

pub(crate) fn drips_surface_crawler_turn_script_loc_3393c() -> DripsSurfaceCrawlerTurnScriptLoc33AF6
{
    // ASM Drips.asm loc_3393C: select `unk_33C80`, then branch to the shared
    // loc_33AF6 turn-script interpreter.
    drips_surface_crawler_turn_script_loc_33af6([
        0x1A, 0x00, 0x00, 0x00, 0x01, 0x00, 0x2C, 0xFF, 0x00, 0xFB, 0x00, 0x00, 0xD0, 0x00, 0x14,
        0x08,
    ])
}

pub(crate) fn drips_surface_crawler_turn_script_loc_33a2c() -> DripsSurfaceCrawlerTurnScriptLoc33AF6
{
    // ASM Drips.asm loc_33A2C: select `unk_33CE0`, then branch to the shared
    // loc_33AF6 turn-script interpreter.
    drips_surface_crawler_turn_script_loc_33af6([
        0x1A, 0x00, 0x00, 0xFF, 0xFE, 0x01, 0x3C, 0x00, 0xFE, 0x03, 0xE0, 0x00, 0x20, 0x00, 0x54,
        0x14,
    ])
}

pub(crate) fn drips_surface_crawler_turn_script_loc_33996() -> DripsSurfaceCrawlerTurnScriptLoc33AF6
{
    // ASM Drips.asm loc_33996: select `unk_33C20`, then branch to the shared
    // loc_33AF6 turn-script interpreter.
    drips_surface_crawler_turn_script_loc_33af6([
        0x1A, 0x08, 0xFF, 0x00, 0x00, 0x00, 0x2C, 0xFF, 0x00, 0xFB, 0x00, 0x00, 0xD0, 0x00, 0x14,
        0x0C,
    ])
}

pub(crate) fn drips_surface_crawler_turn_script_loc_339dc() -> DripsSurfaceCrawlerTurnScriptLoc33AF6
{
    // ASM Drips.asm loc_339DC: select `unk_33CA0`, then branch to the shared
    // loc_33AF6 turn-script interpreter.
    drips_surface_crawler_turn_script_loc_33af6([
        0x1A, 0x08, 0xFF, 0x00, 0x00, 0x00, 0x34, 0x00, 0x04, 0xFD, 0x20, 0x00, 0xE0, 0x00, 0x30,
        0x14,
    ])
}

pub(crate) fn drips_surface_crawler_turn_script_loc_339d2() -> DripsSurfaceCrawlerTurnScriptLoc33AF6
{
    // ASM Drips.asm loc_339D2: select `unk_33BE0`, then branch to the shared
    // loc_33AF6 turn-script interpreter.
    drips_surface_crawler_turn_script_loc_33af6([
        0x1E, 0x00, 0x00, 0x00, 0xFF, 0x00, 0x04, 0xFF, 0xFB, 0x00, 0xD0, 0x00, 0x00, 0x00, 0x18,
        0x00,
    ])
}

pub(crate) fn drips_surface_crawler_turn_script_loc_33a40() -> DripsSurfaceCrawlerTurnScriptLoc33AF6
{
    // ASM Drips.asm loc_33A40: select `unk_33D10`, then branch to the shared
    // loc_33AF6 turn-script interpreter.
    drips_surface_crawler_turn_script_loc_33af6([
        0x1A, 0x00, 0x00, 0x00, 0x01, 0x00, 0x34, 0x00, 0xFC, 0xFD, 0xE0, 0x00, 0xE0, 0x00, 0x30,
        0x18,
    ])
}

pub(crate) fn drips_surface_crawler_turn_script_loc_33a18() -> DripsSurfaceCrawlerTurnScriptLoc33AF6
{
    // ASM Drips.asm loc_33A18: select `unk_33CC0`, then branch to the shared
    // loc_33AF6 turn-script interpreter.
    drips_surface_crawler_turn_script_loc_33af6([
        0x1A, 0x08, 0x00, 0xFF, 0x02, 0x01, 0x3C, 0x00, 0x03, 0x04, 0x20, 0x00, 0x20, 0x00, 0x54,
        0x18,
    ])
}

pub(crate) fn drips_surface_crawler_turn_script_loc_33a22() -> DripsSurfaceCrawlerTurnScriptLoc33AF6
{
    // ASM Drips.asm loc_33A22: select `unk_33CD0`, then branch to the shared
    // loc_33AF6 turn-script interpreter.
    drips_surface_crawler_turn_script_loc_33af6([
        0x1E, 0x08, 0x00, 0x00, 0x05, 0x00, 0x40, 0xFF, 0x05, 0x00, 0x30, 0x00, 0x00, 0x00, 0x18,
        0x00,
    ])
}

pub(crate) fn drips_surface_crawler_turn_script_loc_33a36() -> DripsSurfaceCrawlerTurnScriptLoc33AF6
{
    // ASM Drips.asm loc_33A36: select `unk_33D00`, then branch to the shared
    // loc_33AF6 turn-script interpreter.
    drips_surface_crawler_turn_script_loc_33af6([
        0x1E, 0x08, 0x01, 0x00, 0xFC, 0xFF, 0x40, 0xFF, 0xFD, 0x00, 0xD0, 0x00, 0x00, 0x00, 0x18,
        0x00,
    ])
}

pub(crate) fn drips_surface_crawler_turn_script_loc_339e6() -> DripsSurfaceCrawlerTurnScriptLoc33AF6
{
    // ASM Drips.asm loc_339E6: select `unk_33CB0`, then branch to the shared
    // loc_33AF6 turn-script interpreter.
    drips_surface_crawler_turn_script_loc_33af6([
        0x1E, 0x00, 0x00, 0x00, 0x03, 0x01, 0x38, 0x00, 0x03, 0xFF, 0x30, 0x00, 0x00, 0x00, 0x18,
        0x00,
    ])
}

pub(crate) fn drips_surface_crawler_turn_script_loc_339f0() -> DripsSurfaceCrawlerTurnScriptLoc33AF6
{
    // ASM Drips.asm loc_339F0: select `unk_33D70`, then branch to the shared
    // loc_33AF6 turn-script interpreter.
    drips_surface_crawler_turn_script_loc_33af6([
        0x1A, 0x08, 0x00, 0x00, 0x01, 0x03, 0x50, 0xFF, 0x00, 0x02, 0x00, 0x00, 0x30, 0x00, 0x10,
        0x08,
    ])
}

pub(crate) fn drips_surface_crawler_turn_script_loc_33a0e() -> DripsSurfaceCrawlerTurnScriptLoc33AF6
{
    // ASM Drips.asm loc_33A0E: select `unk_33D90`, then branch to the shared
    // loc_33AF6 turn-script interpreter.
    drips_surface_crawler_turn_script_loc_33af6([
        0x1A, 0x00, 0x00, 0x00, 0xFF, 0x03, 0x50, 0x0F, 0x00, 0x02, 0x00, 0x00, 0x30, 0x00, 0x10,
        0x0C,
    ])
}

pub(crate) fn drips_surface_crawler_turn_script_loc_33a72() -> DripsSurfaceCrawlerTurnScriptLoc33AF6
{
    // ASM Drips.asm loc_33A72: select `unk_33D60`, then branch to the shared
    // loc_33AF6 turn-script interpreter.
    drips_surface_crawler_turn_script_loc_33af6([
        0x1E, 0x08, 0x00, 0xFF, 0xFF, 0xFF, 0x48, 0x00, 0x03, 0x04, 0x20, 0x00, 0x20, 0x00, 0x54,
        0x18,
    ])
}

pub(crate) fn drips_surface_crawler_turn_script_loc_339fa() -> DripsSurfaceCrawlerTurnScriptLoc33AF6
{
    // ASM Drips.asm loc_339FA: select `unk_33D80`, then branch to the shared
    // loc_33AF6 turn-script interpreter.
    drips_surface_crawler_turn_script_loc_33af6([
        0x1E, 0x00, 0x00, 0xFF, 0x00, 0x00, 0x4C, 0x00, 0x00, 0x01, 0xE0, 0x00, 0xE0, 0x00, 0x30,
        0x18,
    ])
}

pub(crate) fn drips_surface_crawler_turn_script_loc_33a04() -> DripsSurfaceCrawlerTurnScriptLoc33AF6
{
    // ASM Drips.asm loc_33A04: select `unk_33CF0`, then branch to the shared
    // loc_33AF6 turn-script interpreter.
    drips_surface_crawler_turn_script_loc_33af6([
        0x1E, 0x00, 0x00, 0xFF, 0x00, 0x00, 0x4C, 0x00, 0x00, 0x00, 0x20, 0x00, 0xE0, 0x00, 0x30,
        0x14,
    ])
}

pub(crate) fn drips_surface_crawler_turn_script_loc_33a4a() -> DripsSurfaceCrawlerTurnScriptLoc33AF6
{
    // ASM Drips.asm loc_33A4A: select `unk_33D20`, then branch to the shared
    // loc_33AF6 turn-script interpreter.
    drips_surface_crawler_turn_script_loc_33af6([
        0x1E, 0x00, 0x00, 0x00, 0xFE, 0x01, 0x38, 0x00, 0xFE, 0xFF, 0xD0, 0x00, 0x00, 0x00, 0x18,
        0x00,
    ])
}

pub(crate) fn drips_surface_crawler_turn_script_loc_33a54() -> DripsSurfaceCrawlerTurnScriptLoc33AF6
{
    // ASM Drips.asm loc_33A54: select `unk_33D30`, then branch to the shared
    // loc_33AF6 turn-script interpreter.
    drips_surface_crawler_turn_script_loc_33af6([
        0x1A, 0x08, 0x00, 0x00, 0xFF, 0xFC, 0x44, 0xFF, 0x00, 0xFE, 0x00, 0x00, 0xD0, 0x00, 0x14,
        0x0C,
    ])
}

pub(crate) fn drips_surface_crawler_turn_script_loc_33a5e() -> DripsSurfaceCrawlerTurnScriptLoc33AF6
{
    // ASM Drips.asm loc_33A5E: select `unk_33D40`, then branch to the shared
    // loc_33AF6 turn-script interpreter.
    drips_surface_crawler_turn_script_loc_33af6([
        0x1E, 0x08, 0x01, 0xFF, 0x00, 0xFF, 0x48, 0x00, 0xFE, 0x02, 0xE0, 0x00, 0x20, 0x00, 0x54,
        0x14,
    ])
}

pub(crate) fn drips_surface_crawler_turn_script_loc_33a68() -> DripsSurfaceCrawlerTurnScriptLoc33AF6
{
    // ASM Drips.asm loc_33A68: select `unk_33D50`, then branch to the shared
    // loc_33AF6 turn-script interpreter.
    drips_surface_crawler_turn_script_loc_33af6([
        0x1A, 0x00, 0x00, 0x00, 0x01, 0xFD, 0x44, 0xFF, 0x00, 0x00, 0x00, 0x00, 0xD0, 0x00, 0x14,
        0x08,
    ])
}

/// Half-width used by the Drips ceiling probe.
/// ASM Enemy16_Drip_Init writes `object_meta=$16`; CollisionSize_Index entry
/// `$16` resolves to `unk_314D0: dc.w -$5,$A,-$1F,$1F`, so the live collision
/// box spans five pixels left and right of x_pos.
const DRIPS_CEILING_PROBE_HALF_WIDTH: i32 = 5;
/// The rising slime's GfxObjects_Collision box top sits 2px above y_pos.
/// Derived from the loc_1A6C ceiling writeback `y = contact_tile_top - y_off +
/// $10`: the issue-260 oracle shows the Level 01 Drips attached at y=530
/// under the row-32 block cells (tile top 512, cells `$E102/$EC2D` carry the
/// `$4000` collision bit), so `y_off = -2`. The contact row is the BLOCK row
/// itself, not the rock row above it — the old `0x1F` reach skipped the block
/// row entirely and only matched y=530 because row-31's bottom edge equals
/// row-32's top edge (512).
const DRIPS_CEILING_BOX_TOP_OFFSET_LOC_1A6C: i32 = 2;
/// Vertical offset from the contact tile TOP when GfxObjects_Collision
/// writes back the attached y_pos. ASM loc_1A6C: `y = tile_top - y_off +
/// $10` with `y_off = -2` gives `tile_top + $12` (Level 01: 512 + 18 = 530).
const DRIPS_CEILING_SNAP_TOP_OFFSET: i32 = 0x12;

pub(crate) fn drips_child_state_bytes_loc_3333e(enemy_level: u8) -> (u8, u8) {
    // ASM Enemy16_Drip_Init loc_3333E: `move.w 2(a0),d7; add.w d7,d7`
    // indexes `unk_33338` and copies two bytes into `$44(a5)`/`$45(a5)`.
    match enemy_level.min(2) {
        0 => (0x00, 0x00),
        1 => (0xFF, 0x00),
        _ => (0xFF, 0xFF),
    }
}

pub(crate) fn step_drips(
    enemy: &mut ActiveEnemy,
    projectiles: &mut Vec<EnemyProjectile>,
    block_grid: Option<&terrain::BlockGrid>,
) {
    // ASM Enemy16_Drip_Init -> loc_333F6 (Drips.asm:35-64): the slime spawns
    // with `y_vel=-$10000` and hibernates one frame at a time until the
    // engine reports `collision_type(a3) == colid_ceiling`. On collision the
    // ROM clears collision_type and y_vel, then enters the wait/drop cycle.
    if !enemy.drips_attached {
        if let Some(grid) = block_grid {
            // Apply the -1px/frame rise, then test the box-top row the way
            // GfxObjects_Collision's tile loop does: contact fires on the
            // frame the box top (`y - 2`) enters a `$4000`-collision row
            // (spikes skipped per loc_17C6). The ROM therefore rises ~14
            // frames from the Level 01 spawn (543 -> 529) before loc_1A6C
            // snaps the slime to 530 under the row-32 block cells.
            let new_y_fp = enemy
                .y_pos
                .wrapping_add(DRIPS_CEILING_ATTACH_RISE_VELOCITY as u32);
            enemy.y_pos = new_y_fp;
            let center_x_px = (enemy.x_pos >> 16) as i32;
            let box_top_px = ((enemy.y_pos >> 16) as i32) - DRIPS_CEILING_BOX_TOP_OFFSET_LOC_1A6C;
            if box_top_px < 0 {
                // ASM GfxObjects_Collision_TopBoundary (loc_1B30): a negative
                // box top clamps the object so its box top rests at 0 and
                // reports colid_ceiling.
                enemy.y_pos = (DRIPS_CEILING_BOX_TOP_OFFSET_LOC_1A6C as u32) << 16;
                enemy.y_velocity = 0;
                enemy.drips_attached = true;
                enemy.wait_timer = DRIPS_INITIAL_SEED_DELAY_2_TIMER_LOC_3340E;
                return;
            }
            let contact_row = terrain::pixel_to_cell(box_top_px).max(0) as u16;
            let left_col =
                terrain::pixel_to_cell(center_x_px - DRIPS_CEILING_PROBE_HALF_WIDTH).max(0) as u16;
            let right_col =
                terrain::pixel_to_cell(center_x_px + DRIPS_CEILING_PROBE_HALF_WIDTH).max(0) as u16;
            let contact = (left_col..=right_col).any(|col| {
                matches!(
                    grid.cell_at(col, contact_row).collision,
                    terrain::CollisionKind::Solid
                        | terrain::CollisionKind::SlopeUp
                        | terrain::CollisionKind::SlopeDown
                )
            });
            if contact {
                // ASM loc_333F6 exits only after GfxObjects_Collision has
                // reported colid_ceiling. loc_1A6C then writes back
                // `y = contact_tile_top - y_off + $10` = tile_top + $12.
                let contact_tile_top = i32::from(contact_row) * 16;
                enemy.y_pos = ((contact_tile_top + DRIPS_CEILING_SNAP_TOP_OFFSET) as u32) << 16;
                enemy.y_velocity = 0;
                enemy.drips_attached = true;
                // ASM loc_3340E draws the RNG byte at the object's own
                // Execute_Objects slot. Same-frame attaches must consume RNG
                // in descending layout order (Allocate_ObjectSlot LIFO), so
                // the draw is deferred to `seed_pending_drips_initial_waits`.
                enemy.wait_timer = DRIPS_INITIAL_SEED_DELAY_2_TIMER_LOC_3340E;
            }
            return;
        }
        // No block grid (e.g., unit tests without terrain): act as if already
        // attached so the wait/drop cycle still runs.
        enemy.drips_attached = true;
        enemy.y_velocity = 0;
    }

    if enemy.wait_timer == DRIPS_DROP_IN_FLIGHT_TIMER {
        return;
    }
    if enemy.wait_timer == DRIPS_INITIAL_SEED_DELAY_2_TIMER_LOC_3340E {
        enemy.wait_timer = DRIPS_INITIAL_SEED_DELAY_1_TIMER_LOC_3340E;
        return;
    }
    if enemy.wait_timer == DRIPS_INITIAL_SEED_DELAY_1_TIMER_LOC_3340E {
        enemy.wait_timer = DRIPS_INITIAL_SEED_PENDING_TIMER_LOC_3340E;
        return;
    }
    if enemy.wait_timer == DRIPS_INITIAL_SEED_PENDING_TIMER_LOC_3340E {
        // The deferred loc_3340E seed pass runs later in the same frame; a
        // direct caller without that pass must not misread the sentinel as a
        // splash/pre-drop timer.
        return;
    }
    if let Some(frames_remaining) = drips_splash_animation_remaining_loc_33488(enemy.wait_timer) {
        if frames_remaining > 0 {
            enemy.wait_timer = drips_splash_animation_timer_loc_33488(frames_remaining - 1);
        }
        return;
    }
    if let Some(frames_remaining) = drips_pre_drop_animation_remaining_loc_33424(enemy.wait_timer) {
        if frames_remaining > 1 {
            enemy.wait_timer = drips_pre_drop_animation_timer_loc_33424(frames_remaining - 1);
            return;
        }
    } else if enemy.wait_timer > 0 {
        enemy.wait_timer -= 1;
        return;
    } else {
        // ASM Drips.asm loc_33424 waits through `stru_33F90` before setting
        // `$13/$14/$3C(a1)` on the falling child.
        enemy.wait_timer =
            drips_pre_drop_animation_timer_loc_33424(DRIPS_PRE_DROP_ANIMATION_FRAMES_LOC_33424);
        return;
    }
    // ASM Drips.asm loc_33424/loc_3345C: the ROM reuses the preallocated
    // falling child object, starts it from rest, then accelerates downward
    // under `$2000` gravity until it reaches the floor.
    let (drips_child_byte_44, drips_child_byte_45) =
        drips_child_state_bytes_loc_3333e(enemy.enemy_level);
    projectiles.push(EnemyProjectile {
        kind: EnemyProjectileKind::DripsDrop,
        x_pos: enemy.x_pos,
        y_pos: enemy.y_pos,
        x_velocity: 0,
        y_velocity: DRIPS_DROP_INITIAL_Y_SPEED,
        lifetime: DRIPS_DROP_LIFETIME,
        gravity: DRIPS_DROP_GRAVITY,
        friendly: false,
        homing_accel: 0,
        speed_cap: 0,
        wall_bounce: false,
        terrain_kill: true,
        drips_spawn_crawler_on_splash: false,
        drips_child_byte_44,
        drips_child_byte_45,
        palette_variant: enemy.palette_variant,
        owner: Some(EnemyProjectileOwner {
            segment_index: enemy.segment_index,
            layout_index: enemy.layout_index,
            token: 0,
        }),
        ..Default::default()
    });
    // ASM loc_3350E branches back to loc_3340E only after the child reaches
    // loc_33488; keep the parent dormant until `step_projectiles` observes
    // the terrain splash and reseeds the timer.
    enemy.wait_timer = DRIPS_DROP_IN_FLIGHT_TIMER;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::enemies::test_fixtures::*;

    #[test]
    fn drips_ceiling_gate_matches_loc_333f6() {
        assert_eq!(
            drips_ceiling_gate_loc_333_f6(0, -0x0001_0000),
            DripsCeilingGateLoc333F6 {
                waiting_for_ceiling: true,
                collision_type: 0,
                y_velocity: -0x0001_0000,
            },
            "ASM Drips.asm loc_333F6 branches back while `collision_type(a3)` is not `colid_ceiling`"
        );
        assert_eq!(
            drips_ceiling_gate_loc_333_f6(DRIPS_COLID_CEILING, -0x0001_0000),
            DripsCeilingGateLoc333F6 {
                waiting_for_ceiling: false,
                collision_type: 0,
                y_velocity: 0,
            },
            "ASM Drips.asm loc_333F6 clears `collision_type(a3)` and `y_vel(a3)` after ceiling contact"
        );
    }

    #[test]
    fn drips_wait_timer_matches_loc_3340e_rng_and_kid_x_mix() {
        assert_eq!(
            drips_wait_timer_loc_3340e(0xFC, 0x01C0),
            0x00FF,
            "ASM Drips.asm loc_3340E shifts the RNG byte right by two, then XORs only d0's low byte with Kid X"
        );

        // ASM Enemy16_Drip_Init runs the ceiling-attach loop (loc_333F6) BEFORE
        // seeding the dormant delay at loc_3340E. Native `spawn_wait_timer`
        // therefore returns 0 for Drips so the rise phase in `step_drips`
        // executes immediately; the RNG-driven wait is seeded once the slime
        // attaches to the ceiling. The post-attach seed itself is exercised
        // separately via `drips_wait_timer_with_rng_loc_3340e`.
        let mut rng = RomRandomState::default();
        assert_eq!(
            spawn_wait_timer(EnemyFamily::Drips, 0, 0x0123_0000, &mut rng),
            0,
            "ASM Drips.asm loc_3340E only seeds the dormant timer after loc_333F6 ceiling attach"
        );
        let mut rng = RomRandomState::default();
        assert_eq!(
            drips_wait_timer_with_rng_loc_3340e(&mut rng, 0x0123_0000),
            0x0013,
            "ASM Drips.asm loc_3340E mixes the first ROM RNG byte `$C0` with Addr_GfxObject_Kid's integer X low byte `$23`; callers must pass the ROM-visible Kid gfx-object coordinate"
        );
        assert_eq!(
            drips_wait_timer_loc_3340e(0xD5, 0x0168),
            0x005D,
            "ASM Drips.asm loc_3340E reads the visible Kid gfx X; oracle #260's right Drips seed uses the `$68` low byte derived from frame-local physics x `$58` plus 16px"
        );
        assert_eq!(
            DRIPS_PRE_DROP_ANIMATION_FRAMES_LOC_33424, 31,
            "ASM GfxObject_Animate loc_1158/sub_1194 counts the frame that loads each `stru_33F90` entry, and the final duration-1 entry finishes immediately when the terminator is visible"
        );
    }

    #[test]
    fn drips_child_state_bytes_match_init_table_loc_3333e() {
        assert_eq!(
            drips_child_state_bytes_loc_3333e(0),
            (0x00, 0x00),
            "ASM Enemy16_Drip_Init loc_3333E entry 0 of `unk_33338` is `$0000`"
        );
        assert_eq!(
            drips_child_state_bytes_loc_3333e(1),
            (0xFF, 0x00),
            "ASM Enemy16_Drip_Init loc_3333E entry 1 of `unk_33338` is `$FF00`"
        );
        assert_eq!(
            drips_child_state_bytes_loc_3333e(2),
            (0xFF, 0xFF),
            "ASM Enemy16_Drip_Init loc_3333E entry 2 of `unk_33338` is `$FFFF`"
        );
        assert_eq!(
            drips_child_state_bytes_loc_3333e(3),
            (0xFF, 0xFF),
            "Native clamps unexpected Drips layout variants to the final ASM table entry"
        );
    }

    #[test]
    fn drips_drop_carries_parent_state_bytes_loc_3333e() {
        let mut enemy = ActiveEnemy {
            family: EnemyFamily::Drips,
            enemy_level: 2,
            palette_variant: 2,
            drips_attached: true,
            wait_timer: 0,
            ..mini_hopping_skull_test_enemy()
        };
        let mut projectiles = Vec::new();

        for _ in 0..=DRIPS_PRE_DROP_ANIMATION_FRAMES_LOC_33424 {
            step_drips(&mut enemy, &mut projectiles, None);
        }

        assert_eq!(projectiles.len(), 1);
        assert_eq!(
            projectiles[0].drips_child_byte_44, 0xFF,
            "ASM Enemy16_Drip_Init loc_3333E copies `$44(a5)` from `unk_33338`; loc_335D8 later forwards it to the crawler"
        );
        assert_eq!(
            projectiles[0].drips_child_byte_45, 0xFF,
            "ASM Enemy16_Drip_Init loc_3333E copies `$45(a5)` from `unk_33338`; loc_33526 uses it for the doubled startup delta"
        );
        assert_eq!(
            projectiles[0].palette_variant, 2,
            "ASM Drips.asm loc_33424 exposes the preallocated child object; its object_meta `$16` inherits the Drip art slot/palette selected by loc_32146"
        );
    }

    #[test]
    fn drips_splash_reset_matches_loc_33488_child_writeback() {
        assert_eq!(
            drips_splash_reset_loc_33488(0x0123_4000, 0x00F6_8000),
            DripsSplashResetLoc33488 {
                drop_is_moved: false,
                drop_status_flag: false,
                drop_collision_type: 0,
                drop_y_pos: 0x0100_8000,
                surface_child_status_flag: true,
                surface_child_x_pos: 0x0123_4000,
                surface_child_y_pos: 0x0100_8000,
            },
            "ASM Drips.asm loc_33488 clears the falling child, adds 10px to its Y word, and copies the adjusted x/y to the surface child"
        );
    }

    #[test]
    fn drips_splash_animation_hibernates_before_table_advance_loc_33488() {
        let mut enemy = ActiveEnemy {
            family: EnemyFamily::Drips,
            wait_timer: DRIPS_DROP_IN_FLIGHT_TIMER,
            drips_attached: true,
            ..mini_hopping_skull_test_enemy()
        };
        drips_enter_splash_animation_loc_33488(&mut enemy, 0x0123_0000, 0x00F6_0000);

        assert_eq!(
            enemy.drips_pending_splash_x_pos, 0x0123_0000,
            "ASM Drips.asm loc_33488 preserves the falling child x word for the surface child before off_33520 is consumed"
        );
        assert_eq!(
            enemy.drips_pending_splash_y_pos, 0x0100_0000,
            "ASM Drips.asm loc_33488 adds 10px to the falling child y word before the surface child copy"
        );

        let mut projectiles = Vec::new();
        for expected_remaining in (0..DRIPS_SPLASH_ANIMATION_FRAMES_LOC_33488).rev() {
            step_drips(&mut enemy, &mut projectiles, None);
            assert_eq!(
                drips_splash_animation_remaining_loc_33488(enemy.wait_timer),
                Some(expected_remaining),
                "ASM Drips.asm loc_33488 hibernates through stru_33332 before off_33520; issue-260 oracle frames 305..340 show the splash child finishing before the table entry advances"
            );
            assert!(
                projectiles.is_empty(),
                "loc_33488 must not start the next loc_33424 drop before the splash hibernate finishes"
            );
        }

        step_drips(&mut enemy, &mut projectiles, None);

        assert_eq!(
            drips_splash_animation_remaining_loc_33488(enemy.wait_timer),
            Some(0),
            "ASM Drips.asm loc_33488 resumes after the final splash hibernate frame"
        );
    }

    #[test]
    fn drips_surface_table_reset_matches_loc_33506() {
        assert_eq!(
            drips_surface_table_reset_loc_33506(),
            (0, 0),
            "ASM Drips.asm loc_33506 reloads `a0` with `off_33520` and clears d7 before the surface child sprite write"
        );
    }

    #[test]
    fn drips_surface_sprite_write_matches_loc_3350e() {
        assert_eq!(
            drips_surface_sprite_write_loc_3350e(0x1234),
            0x1234,
            "ASM Drips.asm loc_3350E stores d7 unchanged into `addroffset_sprite(a2)`"
        );
        assert_eq!(
            drips_surface_sprite_write_loc_3350e(0),
            0,
            "ASM Drips.asm loc_3350E also writes the zero sprite word produced by the table reset path"
        );
    }

    #[test]
    fn drips_idle_loop_matches_loc_33516() {
        assert_eq!(
            drips_idle_loop_loc_33516(),
            DripsIdleLoopLoc33516 {
                hibernates_one_frame: true,
                checks_in_range: true,
                loops_forever: true,
            },
            "ASM Drips.asm loc_33516 hibernates for one frame, checks object range, then branches back to itself"
        );
    }

    #[test]
    fn drips_velocity_deltas_match_loc_33526_child_state_gates() {
        assert_eq!(
            drips_velocity_deltas_loc_33526(0, 0xFF, 0x0000_3000, -0x0000_2000),
            DripsVelocityDeltasLoc33526 {
                x_delta: 0,
                y_delta: 0,
            },
            "ASM Drips.asm loc_33526 leaves d7/d6 clear when `$44(a5)` is zero, even if `$45(a5)` is set"
        );
        assert_eq!(
            drips_velocity_deltas_loc_33526(1, 0, 0x0000_3000, -0x0000_2000),
            DripsVelocityDeltasLoc33526 {
                x_delta: 0x0000_3000,
                y_delta: -0x0000_2000,
            },
            "ASM Drips.asm loc_33526 copies x/y velocity once when `$44(a5)` is set and `$45(a5)` is clear"
        );
        assert_eq!(
            drips_velocity_deltas_loc_33526(1, 1, 0x0000_3000, -0x0000_2000),
            DripsVelocityDeltasLoc33526 {
                x_delta: 0x0000_6000,
                y_delta: -0x0000_4000,
            },
            "ASM Drips.asm loc_33526 doubles d7/d6 with `add.l` when both child state bytes are set"
        );
    }

    #[test]
    fn drips_velocity_apply_matches_loc_33546_wrapping_adds() {
        assert_eq!(
            drips_velocity_apply_loc_33546(0x0000_3000, -0x0000_2000, 0x0000_6000, -0x0000_4000,),
            DripsVelocityApplyLoc33546 {
                x_velocity: 0x0000_9000,
                y_velocity: -0x0000_6000,
            },
            "ASM Drips.asm loc_33546 adds d7/d6 directly into x_vel/y_vel"
        );
        assert_eq!(
            drips_velocity_apply_loc_33546(i32::MAX, i32::MIN, 1, -1),
            DripsVelocityApplyLoc33546 {
                x_velocity: i32::MIN,
                y_velocity: i32::MAX,
            },
            "ASM Drips.asm loc_33546 uses 32-bit `add.l`, so overflow wraps instead of saturating"
        );
    }

    #[test]
    fn drips_collision_dispatch_matches_loc_33550_entry_paths() {
        assert_eq!(
            drips_collision_dispatch_loc_33550(0, true),
            DripsCollisionDispatchLoc33550 {
                collision_type: 0,
                contact_flag: 0,
            },
            "ASM Drips.asm loc_33550 returns d5=0 for zero collision without needing the dispatch table"
        );
        assert_eq!(
            drips_collision_dispatch_loc_33550(0xFFFF, false),
            DripsCollisionDispatchLoc33550 {
                collision_type: 0,
                contact_flag: 1,
            },
            "ASM Drips.asm loc_33550 treats a negative collision word as contact and clears `collision_type(a3)`"
        );
        assert_eq!(
            drips_collision_dispatch_loc_33550(0x001A, false),
            DripsCollisionDispatchLoc33550 {
                collision_type: 0,
                contact_flag: 0,
            },
            "ASM Drips.asm loc_33550 clears positive collision words before rejecting values below `$1C`"
        );
        assert_eq!(
            drips_collision_dispatch_loc_33550(0x001C, false).contact_flag,
            1,
            "ASM Drips.asm loc_33550 maps `$1C` to d5=1 contact"
        );
        assert_eq!(
            drips_collision_dispatch_loc_33550(0x0020, false).contact_flag,
            0,
            "ASM Drips.asm loc_33550 maps `$20` through the Berzerker-charging gate"
        );
        assert_eq!(
            drips_collision_dispatch_loc_33550(0x0024, true).contact_flag,
            1,
            "ASM Drips.asm loc_33550 accepts `$24` when `Berzerker_charging` is set"
        );
        assert_eq!(
            drips_collision_dispatch_loc_33550(0x0028, true).contact_flag,
            0,
            "ASM Drips.asm loc_33550 dispatches `$28` to the no-contact return"
        );
        assert_eq!(
            drips_collision_dispatch_loc_33550(0x002C, false).contact_flag,
            1,
            "ASM Drips.asm loc_33550 maps `$2C` to d5=1 contact"
        );
    }

    #[test]
    fn drips_collision_table_first_contact_matches_loc_3356e() {
        assert_eq!(
            drips_collision_table_first_contact_loc_3356e(),
            1,
            "ASM Drips.asm loc_3356E dispatches the `$1C` collision slot to contact"
        );
        assert_eq!(
            drips_collision_dispatch_loc_33550(0x001C, false),
            DripsCollisionDispatchLoc33550 {
                collision_type: 0,
                contact_flag: 1,
            },
            "ASM Drips.asm loc_3356E is reached from loc_33550 after clearing `collision_type(a3)`"
        );
    }

    #[test]
    fn drips_collision_contact_result_matches_loc_33576() {
        assert_eq!(
            drips_collision_contact_result_loc_33576(),
            DripsCollisionDispatchLoc33550 {
                collision_type: 0,
                contact_flag: 1,
            },
            "ASM Drips.asm loc_33576 clears `collision_type(a3)`, moves 1 into d5, and returns"
        );
        assert_eq!(
            drips_collision_dispatch_loc_33550(0xFFFF, false),
            DripsCollisionDispatchLoc33550 {
                collision_type: 0,
                contact_flag: 1,
            },
            "ASM Drips.asm loc_33576 is also the negative collision word contact path"
        );
    }

    #[test]
    fn drips_collision_no_contact_result_matches_loc_33584() {
        assert_eq!(
            drips_collision_no_contact_result_loc_33584(0),
            DripsCollisionDispatchLoc33550 {
                collision_type: 0,
                contact_flag: 0,
            },
            "ASM Drips.asm loc_33584 moves 0 into d5 and returns for no collision"
        );
        assert_eq!(
            drips_collision_dispatch_loc_33550(0x0028, true),
            DripsCollisionDispatchLoc33550 {
                collision_type: 0,
                contact_flag: 0,
            },
            "ASM Drips.asm loc_33584 is the `$28` dispatch-table no-contact slot after the collision word is cleared"
        );
    }

    #[test]
    fn drips_right_bounds_gate_matches_loc_33588() {
        assert_eq!(
            drips_right_bounds_gate_loc_33588(true, 0x0020, 0x0000_3000, 0x0100),
            DripsRightBoundsNextLoc33588::ContactLoc335D4,
            "ASM Drips.asm loc_33588 sends non-zero `$19(a3)` directly to d5=1"
        );
        assert_eq!(
            drips_right_bounds_gate_loc_33588(false, 0x0020, -0x0000_3000, 0x0100),
            DripsRightBoundsNextLoc33588::CheckLeftLoc335AA,
            "ASM Drips.asm loc_33588 branches to loc_335AA when `x_vel(a3)` is negative"
        );
        assert_eq!(
            drips_right_bounds_gate_loc_33588(false, 0x00F8, 0x0000_3000, 0x0100),
            DripsRightBoundsNextLoc33588::ContactLoc335D4,
            "ASM Drips.asm loc_33588 treats `x_pos(a3)+8 >= Level_width_pixels` as contact"
        );
        assert_eq!(
            drips_right_bounds_gate_loc_33588(false, 0x00F7, 0x0000_3000, 0x0100),
            DripsRightBoundsNextLoc33588::CheckVerticalLoc335B0,
            "ASM Drips.asm loc_33588 continues to loc_335B0 while the right probe is still inside the level"
        );
    }

    #[test]
    fn drips_left_bounds_gate_matches_loc_335aa() {
        assert_eq!(
            drips_left_bounds_gate_loc_335aa(0x0007),
            DripsLeftBoundsNextLoc335AA::ContactLoc335D4,
            "ASM Drips.asm loc_335AA treats `x_pos(a3)-8` becoming negative as d5=1"
        );
        assert_eq!(
            drips_left_bounds_gate_loc_335aa(0x0008),
            DripsLeftBoundsNextLoc335AA::CheckVerticalLoc335B0,
            "ASM Drips.asm loc_335AA falls through to loc_335B0 when `x_pos(a3)-8` is zero"
        );
        assert_eq!(
            drips_left_bounds_gate_loc_335aa(0x0020),
            DripsLeftBoundsNextLoc335AA::CheckVerticalLoc335B0,
            "ASM Drips.asm loc_335AA continues to the vertical gate while the left probe is inside the level"
        );
    }

    #[test]
    fn drips_vertical_bounds_flag_matches_loc_335b0() {
        assert_eq!(
            drips_vertical_bounds_contact_flag_loc_335b0(0x00F8, 0x0000_3000, 0x0100),
            1,
            "ASM Drips.asm loc_335B0 treats `y_pos(a3)+8 >= Level_height_pixels` as d5=1"
        );
        assert_eq!(
            drips_vertical_bounds_contact_flag_loc_335b0(0x00F7, 0x0000_3000, 0x0100),
            0,
            "ASM Drips.asm loc_335B0 returns d5=0 while the downward probe remains inside the level"
        );
        assert_eq!(
            drips_vertical_bounds_contact_flag_loc_335b0(0x0007, -0x0000_3000, 0x0100),
            1,
            "ASM Drips.asm loc_335B0 branches through loc_335CA when negative y velocity makes `y_pos(a3)-8` negative"
        );
        assert_eq!(
            drips_vertical_bounds_contact_flag_loc_335b0(0x0008, -0x0000_3000, 0x0100),
            0,
            "ASM Drips.asm loc_335B0 returns d5=0 when the upward probe lands exactly on the top edge"
        );
    }

    #[test]
    fn drips_bounds_no_contact_result_matches_loc_335d0() {
        assert_eq!(
            drips_bounds_no_contact_result_loc_335d0(),
            0,
            "ASM Drips.asm loc_335D0 moves 0 into d5 for the in-bounds return tail"
        );
        assert_eq!(
            drips_vertical_bounds_contact_flag_loc_335b0(0x0008, -0x0000_3000, 0x0100),
            drips_bounds_no_contact_result_loc_335d0(),
            "ASM Drips.asm loc_335D0 is reached after the upward probe remains in bounds"
        );
    }

    #[test]
    fn drips_bounds_contact_result_matches_loc_335d4() {
        assert_eq!(
            drips_bounds_contact_result_loc_335d4(),
            1,
            "ASM Drips.asm loc_335D4 moves 1 into d5 for the out-of-bounds return tail"
        );
        assert_eq!(
            drips_vertical_bounds_contact_flag_loc_335b0(0x00F8, 0x0000_3000, 0x0100),
            drips_bounds_contact_result_loc_335d4(),
            "ASM Drips.asm loc_335D4 is reached when the downward probe reaches Level_height_pixels"
        );
        assert_eq!(
            drips_vertical_bounds_contact_flag_loc_335b0(0x0007, -0x0000_3000, 0x0100),
            drips_bounds_contact_result_loc_335d4(),
            "ASM Drips.asm loc_335D4 is reached when the upward probe becomes signed-negative"
        );
    }

    #[test]
    fn drips_surface_crawler_init_matches_loc_335d8() {
        assert_eq!(
            drips_surface_crawler_init_loc_335d8(0x0124, 0x00F0, 0x01, 0x00, 0x00),
            DripsSurfaceCrawlerInitLoc335D8 {
                status_flag_13: true,
                priority: 0,
                object_meta: 0x0016,
                x_pos_word: 0x0124,
                y_pos_word: 0x00F0,
                child_byte_44: 0x01,
                child_byte_45: 0x00,
                x_direction: 0x00,
                x_velocity: 0x0000_3000,
                startup_animation: "stru_33FA6",
            },
            "ASM Drips.asm loc_335D8 keeps default x_direction and positive `$3000` x_vel when random bit 2 is clear"
        );
        assert_eq!(
            drips_surface_crawler_init_loc_335d8(0x0124, 0x00F0, 0x02, 0x03, 0x04),
            DripsSurfaceCrawlerInitLoc335D8 {
                status_flag_13: true,
                priority: 0,
                object_meta: 0x0016,
                x_pos_word: 0x0124,
                y_pos_word: 0x00F0,
                child_byte_44: 0x02,
                child_byte_45: 0x03,
                x_direction: 0xFF,
                x_velocity: -0x0000_3000,
                startup_animation: "stru_33FA6",
            },
            "ASM Drips.asm loc_335D8 copies parent `$44/$45`, flips x_direction, and negates x_vel when `bclr #2,d7` finds the bit set"
        );
    }

    #[test]
    fn drips_surface_crawler_animation_tail_matches_loc_33630() {
        assert_eq!(
            drips_surface_crawler_animation_tail_loc_33630(1, 0, 0x0000_3000, -0x0000_2000),
            DripsSurfaceCrawlerAnimationTailLoc33630 {
                first_animation: "stru_33FA6",
                second_animation: "stru_33F14",
                is_moved_set: true,
                deltas: DripsVelocityDeltasLoc33526 {
                    x_delta: 0x0000_3000,
                    y_delta: -0x0000_2000,
                },
            },
            "ASM Drips.asm loc_33630 runs both animations, sets `is_moved`, and tails through loc_33526"
        );
        assert_eq!(
            drips_surface_crawler_animation_tail_loc_33630(1, 1, 0x0000_3000, -0x0000_2000),
            DripsSurfaceCrawlerAnimationTailLoc33630 {
                first_animation: "stru_33FA6",
                second_animation: "stru_33F14",
                is_moved_set: true,
                deltas: DripsVelocityDeltasLoc33526 {
                    x_delta: 0x0000_6000,
                    y_delta: -0x0000_4000,
                },
            },
            "ASM Drips.asm loc_33630 tails through loc_33526 with double-delta child gating"
        );
    }

    #[test]
    fn drips_surface_crawler_movement_tail_matches_loc_33650() {
        assert_eq!(
            drips_surface_crawler_movement_tail_loc_33650(0xFFFF, false, 0x6000, false, true, 0, 0,),
            DripsSurfaceCrawlerMovementPathLoc33650::Loc338F8,
            "ASM Drips.asm loc_33650 routes collision contact through loc_33550 to `loc_338F8`"
        );
        assert_eq!(
            drips_surface_crawler_movement_tail_loc_33650(0, false, 0x5000, false, false, 0, 0,),
            DripsSurfaceCrawlerMovementPathLoc33650::Loc338F8,
            "ASM Drips.asm loc_33650 routes `loc_331D2` mismatch through `loc_338F8`"
        );
        assert_eq!(
            drips_surface_crawler_movement_tail_loc_33650(0, false, 0x6000, true, false, 0, 0,),
            DripsSurfaceCrawlerMovementPathLoc33650::Loc338EA,
            "ASM Drips.asm loc_33650 routes `loc_33588` contact through `loc_338EA`"
        );
        assert_eq!(
            drips_surface_crawler_movement_tail_loc_33650(
                0, false, 0x6000, false, true, 0x5000, 0x6000,
            ),
            DripsSurfaceCrawlerMovementPathLoc33650::Loc336AA,
            "ASM Drips.asm loc_33650 branches to loc_336AA when `x_direction` is left (`beq` path)"
        );
        assert_eq!(
            drips_surface_crawler_movement_tail_loc_33650(
                0, false, 0x6000, false, false, 0, 0x6000,
            ),
            DripsSurfaceCrawlerMovementPathLoc33650::Loc3393C,
            "ASM Drips.asm loc_33650 takes `loc_331F6`-d6 `$6000` into `loc_3393C`"
        );
        assert_eq!(
            drips_surface_crawler_movement_tail_loc_33650(
                0, false, 0x6000, false, false, 0, 0x5000,
            ),
            DripsSurfaceCrawlerMovementPathLoc33650::Loc33A40,
            "ASM Drips.asm loc_33650 takes `loc_331F6`-d6 `$5000` into `loc_33A40`"
        );
        assert_eq!(
            drips_surface_crawler_movement_tail_loc_33650(
                0, false, 0x6000, false, false, 0x6000, 0x0000,
            ),
            DripsSurfaceCrawlerMovementPathLoc33650::Loc33650,
            "ASM Drips.asm loc_33650 loops to itself when `loc_331F6`-d5 `$6000`"
        );
        assert_eq!(
            drips_surface_crawler_movement_tail_loc_33650(
                0, false, 0x6000, false, false, 0x4000, 0x0000,
            ),
            DripsSurfaceCrawlerMovementPathLoc33650::Loc33A2C,
            "ASM Drips.asm loc_33650 takes `loc_331F6`-d5 `$4000` into `loc_33A2C`"
        );
        assert_eq!(
            drips_surface_crawler_movement_tail_loc_33650(
                0, false, 0x6000, false, false, 0x1111, 0x0000,
            ),
            DripsSurfaceCrawlerMovementPathLoc33650::Loc33982,
            "ASM Drips.asm loc_33650 falls through to `loc_33982` for non-matching tile flags"
        );
    }

    #[test]
    fn drips_surface_crawler_movement_tail_matches_loc_336aa() {
        assert_eq!(
            drips_surface_crawler_movement_tail_loc_336aa(0x1111, 0x4000),
            DripsSurfaceCrawlerMovementPathLoc33650::Loc339DC,
            "ASM Drips.asm loc_336AA routes `loc_331F6`-d6 `$4000` into `loc_339DC`"
        );
        assert_eq!(
            drips_surface_crawler_movement_tail_loc_336aa(0x1111, 0x6000),
            DripsSurfaceCrawlerMovementPathLoc33650::Loc33996,
            "ASM Drips.asm loc_336AA routes `loc_331F6`-d6 `$6000` into `loc_33996`"
        );
        assert_eq!(
            drips_surface_crawler_movement_tail_loc_336aa(0x6000, 0x1111),
            DripsSurfaceCrawlerMovementPathLoc33650::Loc33650,
            "ASM Drips.asm loc_336AA loops to `loc_33650` when `loc_331F6`-d5 `$6000`"
        );
        assert_eq!(
            drips_surface_crawler_movement_tail_loc_336aa(0x5000, 0x1111),
            DripsSurfaceCrawlerMovementPathLoc33650::Loc33A18,
            "ASM Drips.asm loc_336AA routes `loc_331F6`-d5 `$5000` into `loc_33A18`"
        );
        assert_eq!(
            drips_surface_crawler_movement_tail_loc_336aa(0x1111, 0x1111),
            DripsSurfaceCrawlerMovementPathLoc33650::Loc339C8,
            "ASM Drips.asm loc_336AA falls through to `loc_339C8` for non-matching tile flags"
        );
    }

    #[test]
    fn drips_surface_crawler_movement_tail_matches_loc_3371a() {
        assert_eq!(
            drips_surface_crawler_movement_tail_loc_3371a(0x6000, 0x1111),
            DripsSurfaceCrawlerMovementPathLoc3371A::Loc339B4,
            "ASM Drips.asm loc_3371A sends `d5=$6000` to `loc_339B4`"
        );
        assert_eq!(
            drips_surface_crawler_movement_tail_loc_3371a(0x1111, 0x6000),
            DripsSurfaceCrawlerMovementPathLoc3371A::Loc336D0,
            "ASM Drips.asm loc_3371A sends `d6=$6000` to loop `loc_336D0`"
        );
        assert_eq!(
            drips_surface_crawler_movement_tail_loc_3371a(0x1111, 0x1111),
            DripsSurfaceCrawlerMovementPathLoc3371A::Loc339AA,
            "ASM Drips.asm loc_3371A falls through to `loc_339AA` for non-matching tile flags"
        );
    }

    #[test]
    fn drips_surface_crawler_movement_tail_matches_loc_336d0() {
        assert_eq!(
            drips_surface_crawler_movement_tail_loc_336d0(true, true, false, true, 0x1111, 0x1111),
            DripsSurfaceCrawlerMovementPathLoc336D0::Loc338F8,
            "ASM Drips.asm loc_336D0 goes to `loc_338F8` when loc_33550 reports contact"
        );
        assert_eq!(
            drips_surface_crawler_movement_tail_loc_336d0(
                false, false, false, true, 0x1111, 0x1111,
            ),
            DripsSurfaceCrawlerMovementPathLoc336D0::Loc338F8,
            "ASM Drips.asm loc_336D0 goes to `loc_338F8` when the `loc_331D2` probe is not `$6000`"
        );
        assert_eq!(
            drips_surface_crawler_movement_tail_loc_336d0(false, true, true, true, 0x1111, 0x1111,),
            DripsSurfaceCrawlerMovementPathLoc336D0::Loc338F8,
            "ASM Drips.asm loc_336D0 goes to `loc_338F8` when `loc_33588` reports level boundary contact"
        );
        assert_eq!(
            drips_surface_crawler_movement_tail_loc_336d0(false, true, false, true, 0x6000, 0x1111,),
            DripsSurfaceCrawlerMovementPathLoc336D0::Loc336D0,
            "ASM Drips.asm loc_336D0 loops to itself when `loc_331F6`-d5 is `$6000`"
        );
        assert_eq!(
            drips_surface_crawler_movement_tail_loc_336d0(false, true, false, true, 0x1111, 0x6000,),
            DripsSurfaceCrawlerMovementPathLoc336D0::Loc3396E,
            "ASM Drips.asm loc_336D0 routes `loc_331F6`-d6 `$6000` to `loc_3396E`"
        );
        assert_eq!(
            drips_surface_crawler_movement_tail_loc_336d0(false, true, false, true, 0x1111, 0x1111,),
            DripsSurfaceCrawlerMovementPathLoc336D0::Loc33950,
            "ASM Drips.asm loc_336D0 falls through to `loc_33950` when no tile flags match"
        );
        assert_eq!(
            drips_surface_crawler_movement_tail_loc_336d0(
                false, true, false, false, 0x6000, 0x1111,
            ),
            DripsSurfaceCrawlerMovementPathLoc336D0::Loc339B4,
            "ASM Drips.asm loc_336D0 paths to `loc_3371A`-derived `loc_339B4` on the right-facing x_direction branch"
        );
        assert_eq!(
            drips_surface_crawler_movement_tail_loc_336d0(
                false, true, false, false, 0x1111, 0x6000,
            ),
            DripsSurfaceCrawlerMovementPathLoc336D0::Loc336D0,
            "ASM Drips.asm loc_336D0 paths to `loc_3371A`-derived `loc_336D0` on the right-facing x_direction branch"
        );
        assert_eq!(
            drips_surface_crawler_movement_tail_loc_336d0(
                false, true, false, false, 0x1111, 0x1111,
            ),
            DripsSurfaceCrawlerMovementPathLoc336D0::Loc339AA,
            "ASM Drips.asm loc_336D0 paths to `loc_3371A`-derived `loc_339AA` on the right-facing x_direction branch"
        );
    }

    #[test]
    fn drips_surface_crawler_movement_tail_matches_loc_33730() {
        assert_eq!(
            drips_surface_crawler_movement_tail_loc_33730(true, true, false, true, 0x1111, 0x1111),
            DripsSurfaceCrawlerMovementPathLoc33730::Loc338F8,
            "ASM Drips.asm loc_33730 goes to `loc_338F8` when loc_33550 reports contact"
        );
        assert_eq!(
            drips_surface_crawler_movement_tail_loc_33730(
                false, false, false, true, 0x1111, 0x1111,
            ),
            DripsSurfaceCrawlerMovementPathLoc33730::Loc338F8,
            "ASM Drips.asm loc_33730 goes to `loc_338F8` when the `loc_331D2` probe is not `$6000`"
        );
        assert_eq!(
            drips_surface_crawler_movement_tail_loc_33730(false, true, true, true, 0x1111, 0x1111,),
            DripsSurfaceCrawlerMovementPathLoc33730::Loc338F8,
            "ASM Drips.asm loc_33730 goes to `loc_338F8` when `loc_33588` reports boundary contact"
        );
        assert_eq!(
            drips_surface_crawler_movement_tail_loc_33730(false, true, false, true, 0x1111, 0x6000,),
            DripsSurfaceCrawlerMovementPathLoc33730::Loc33964,
            "ASM Drips.asm loc_33730 routes `loc_33286`-d6 `$6000` to `loc_33964` for falling movement"
        );
        assert_eq!(
            drips_surface_crawler_movement_tail_loc_33730(false, true, false, true, 0x6000, 0x1111,),
            DripsSurfaceCrawlerMovementPathLoc33730::Loc33730,
            "ASM Drips.asm loc_33730 loops to itself when `loc_33286`-d5 is `$6000` while falling"
        );
        assert_eq!(
            drips_surface_crawler_movement_tail_loc_33730(false, true, false, true, 0x5000, 0x1111,),
            DripsSurfaceCrawlerMovementPathLoc33730::Loc339FA,
            "ASM Drips.asm loc_33730 routes `loc_33286`-d5 `$5000` to `loc_339FA`"
        );
        assert_eq!(
            drips_surface_crawler_movement_tail_loc_33730(false, true, false, true, 0x7000, 0x1111,),
            DripsSurfaceCrawlerMovementPathLoc33730::Loc339D2,
            "ASM Drips.asm loc_33730 falls through to `loc_339D2` when `loc_33286` flags are non-matching"
        );
        assert_eq!(
            drips_surface_crawler_movement_tail_loc_33730(
                false, true, false, false, 0x1111, 0x6000,
            ),
            DripsSurfaceCrawlerMovementPathLoc33730::Loc33946,
            "ASM Drips.asm loc_33730 (y_vel<0 branch) routes `loc_33286`-d6 `$6000` to `loc_33946`"
        );
        assert_eq!(
            drips_surface_crawler_movement_tail_loc_33730(
                false, true, false, false, 0x5000, 0x1111,
            ),
            DripsSurfaceCrawlerMovementPathLoc33730::Loc33A72,
            "ASM Drips.asm loc_33730 (y_vel<0 branch) routes `loc_33286`-d5 `$5000` to `loc_33A72`"
        );
        assert_eq!(
            drips_surface_crawler_movement_tail_loc_33730(
                false, true, false, false, 0x6000, 0x1111,
            ),
            DripsSurfaceCrawlerMovementPathLoc33730::Loc33730,
            "ASM Drips.asm loc_33730 (y_vel<0 branch) loops to itself when `loc_33286`-d5 is `$6000`"
        );
        assert_eq!(
            drips_surface_crawler_movement_tail_loc_33730(
                false, true, false, false, 0x7000, 0x1111,
            ),
            DripsSurfaceCrawlerMovementPathLoc33730::Loc339A0,
            "ASM Drips.asm loc_33730 (y_vel<0 branch) falls through to `loc_339A0` for non-matching tile flags"
        );
    }

    #[test]
    fn drips_surface_crawler_movement_tail_matches_loc_337a0() {
        assert_eq!(
            drips_surface_crawler_movement_tail_loc_337a0(true, true, false, true, 0x1111, 0x1111),
            DripsSurfaceCrawlerMovementPathLoc337A0::GuardExit,
            "ASM Drips.asm loc_337A0 takes the guard exit when loc_33550 reports contact"
        );
        assert_eq!(
            drips_surface_crawler_movement_tail_loc_337a0(
                false, false, false, true, 0x1111, 0x1111,
            ),
            DripsSurfaceCrawlerMovementPathLoc337A0::GuardExit,
            "ASM Drips.asm loc_337A0 takes the guard exit when the right `loc_331D2` probe is not `$6000`"
        );
        assert_eq!(
            drips_surface_crawler_movement_tail_loc_337a0(false, true, true, true, 0x1111, 0x1111,),
            DripsSurfaceCrawlerMovementPathLoc337A0::GuardExit,
            "ASM Drips.asm loc_337A0 takes the guard exit when `loc_33588` reports boundary contact"
        );
        assert_eq!(
            drips_surface_crawler_movement_tail_loc_337a0(
                false, true, false, false, 0x1111, 0x1111,
            ),
            DripsSurfaceCrawlerMovementPathLoc337A0::NonNegativeYVelocityBranch,
            "ASM Drips.asm loc_337A0 branches to the non-negative y-velocity label when `y_vel(a3)` is non-negative"
        );
        assert_eq!(
            drips_surface_crawler_movement_tail_loc_337a0(false, true, false, true, 0x6000, 0x1111,),
            DripsSurfaceCrawlerMovementPathLoc337A0::SolidD5Turn,
            "ASM Drips.asm loc_337A0 routes `loc_33286`-d5 `$6000` to the solid-d5 turn while y velocity is negative"
        );
        assert_eq!(
            drips_surface_crawler_movement_tail_loc_337a0(false, true, false, true, 0x1111, 0x6000,),
            DripsSurfaceCrawlerMovementPathLoc337A0::Loop,
            "ASM Drips.asm loc_337A0 loops to itself when `loc_33286`-d6 is `$6000` while y velocity is negative"
        );
        assert_eq!(
            drips_surface_crawler_movement_tail_loc_337a0(false, true, false, true, 0x1111, 0x4000,),
            DripsSurfaceCrawlerMovementPathLoc337A0::SolidD6Corner,
            "ASM Drips.asm loc_337A0 routes `loc_33286`-d6 `$4000` to the solid-d6 corner"
        );
        assert_eq!(
            drips_surface_crawler_movement_tail_loc_337a0(false, true, false, true, 0x7000, 0x1111,),
            DripsSurfaceCrawlerMovementPathLoc337A0::FallbackTurn,
            "ASM Drips.asm loc_337A0 falls through to the fallback turn for non-matching tile flags"
        );
    }

    #[test]
    fn drips_surface_crawler_movement_tail_matches_loc_337f2() {
        assert_eq!(
            drips_surface_crawler_movement_tail_loc_337f2(0x6000, 0x6000),
            DripsSurfaceCrawlerMovementPathLoc337F2::SolidD5Turn,
            "ASM Drips.asm loc_337F2 checks d5 `$6000` before d6, so the solid-d5 turn wins"
        );
        assert_eq!(
            drips_surface_crawler_movement_tail_loc_337f2(0x4000, 0x6000),
            DripsSurfaceCrawlerMovementPathLoc337F2::SolidD5Corner,
            "ASM Drips.asm loc_337F2 checks d5 `$4000` before the entry-loop d6 flag"
        );
        assert_eq!(
            drips_surface_crawler_movement_tail_loc_337f2(0x1111, 0x6000),
            DripsSurfaceCrawlerMovementPathLoc337F2::LoopToEntry,
            "ASM Drips.asm loc_337F2 loops to the entry label when only d6 is `$6000`"
        );
        assert_eq!(
            drips_surface_crawler_movement_tail_loc_337f2(0x7000, 0x1111),
            DripsSurfaceCrawlerMovementPathLoc337F2::FallbackTurn,
            "ASM Drips.asm loc_337F2 falls through to the fallback turn for non-matching tile flags"
        );
    }

    #[test]
    fn drips_surface_crawler_idle_loop_matches_loc_33866() {
        assert_eq!(
            drips_surface_crawler_idle_loop_loc_33866(),
            DripsSurfaceCrawlerIdleLoopLoc33866 {
                hibernates_one_frame: true,
                loops_forever: true,
            },
            "ASM Drips.asm loc_33866 hibernates one frame and branches back to itself"
        );
    }

    #[test]
    fn drips_surface_crawler_idle_loop_matches_loc_33a7c() {
        assert_eq!(
            drips_surface_crawler_idle_loop_loc_33a7c(),
            DripsSurfaceCrawlerIdleLoopLoc33866 {
                hibernates_one_frame: true,
                loops_forever: true,
            },
            "ASM Drips.asm loc_33A7C calls `j_Hibernate_Object_1Frame` then branches back to itself"
        );
    }

    #[test]
    fn drips_surface_crawler_movement_tail_matches_loc_33878() {
        assert_eq!(
            drips_surface_crawler_movement_tail_loc_33878(0x4000),
            DripsSurfaceCrawlerMovementPathLoc33878::DiagonalLoop,
            "ASM Drips.asm loc_33878 loops to loc_33810 when the `loc_3330E` d7 probe returns `$4000`"
        );
        assert_eq!(
            drips_surface_crawler_movement_tail_loc_33878(0x6000),
            DripsSurfaceCrawlerMovementPathLoc33878::SolidTurn,
            "ASM Drips.asm loc_33878 routes d7 `$6000` to loc_33A36"
        );
        assert_eq!(
            drips_surface_crawler_movement_tail_loc_33878(0x1111),
            DripsSurfaceCrawlerMovementPathLoc33878::FallbackTurn,
            "ASM Drips.asm loc_33878 falls through to loc_33A0E for non-matching d7 flags"
        );
    }

    #[test]
    fn drips_surface_crawler_movement_tail_matches_loc_33810() {
        assert_eq!(
            drips_surface_crawler_movement_tail_loc_33810(
                true, false, 0x0127, 0x00F8, false, 0x1111,
            ),
            DripsSurfaceCrawlerMovementPathLoc33810::GuardExit,
            "ASM Drips.asm loc_33810 exits through the guard path when loc_33550 reports contact"
        );
        assert_eq!(
            drips_surface_crawler_movement_tail_loc_33810(
                false, true, 0x0127, 0x00F8, false, 0x1111,
            ),
            DripsSurfaceCrawlerMovementPathLoc33810::GuardExit,
            "ASM Drips.asm loc_33810 exits through the guard path when loc_33588 reports boundary contact"
        );
        assert_eq!(
            drips_surface_crawler_movement_tail_loc_33810(
                false, false, 0x0127, 0x00F7, false, 0x1111,
            ),
            DripsSurfaceCrawlerMovementPathLoc33810::AlignmentLoop,
            "ASM Drips.asm loc_33810 loops until `(x_pos & $F) + (y_pos & $F)` equals `$F`"
        );
        assert_eq!(
            drips_surface_crawler_movement_tail_loc_33810(
                false, false, 0x0127, 0x00F8, true, 0x4000,
            ),
            DripsSurfaceCrawlerMovementPathLoc33810::NegativeCornerLoop,
            "ASM Drips.asm loc_33810 loops on d7 `$4000` after the negative-y `loc_3330E` probe"
        );
        assert_eq!(
            drips_surface_crawler_movement_tail_loc_33810(
                false, false, 0x0127, 0x00F8, true, 0x6000,
            ),
            DripsSurfaceCrawlerMovementPathLoc33810::NegativeSolidTurn,
            "ASM Drips.asm loc_33810 takes the negative-y solid turn on d7 `$6000`"
        );
        assert_eq!(
            drips_surface_crawler_movement_tail_loc_33810(
                false, false, 0x0127, 0x00F8, true, 0x1111,
            ),
            DripsSurfaceCrawlerMovementPathLoc33810::NegativeFallbackTurn,
            "ASM Drips.asm loc_33810 takes the negative-y fallback turn for non-matching d7"
        );
        assert_eq!(
            drips_surface_crawler_movement_tail_loc_33810(
                false, false, 0x0127, 0x00F8, false, 0x4000,
            ),
            DripsSurfaceCrawlerMovementPathLoc33810::NonNegativeCornerLoop,
            "ASM Drips.asm loc_33810 loops on d7 `$4000` after the non-negative-y `loc_3330E` probe"
        );
        assert_eq!(
            drips_surface_crawler_movement_tail_loc_33810(
                false, false, 0x0127, 0x00F8, false, 0x6000,
            ),
            DripsSurfaceCrawlerMovementPathLoc33810::NonNegativeSolidTurn,
            "ASM Drips.asm loc_33810 takes the non-negative-y solid turn on d7 `$6000`"
        );
        assert_eq!(
            drips_surface_crawler_movement_tail_loc_33810(
                false, false, 0x0127, 0x00F8, false, 0x1111,
            ),
            DripsSurfaceCrawlerMovementPathLoc33810::NonNegativeFallbackTurn,
            "ASM Drips.asm loc_33810 takes the non-negative-y fallback turn for non-matching d7"
        );
    }

    #[test]
    fn drips_surface_crawler_movement_tail_matches_loc_33892() {
        assert_eq!(
            drips_surface_crawler_movement_tail_loc_33892(true, false, false, 0x1111),
            DripsSurfaceCrawlerMovementPathLoc33892::GuardExit,
            "ASM Drips.asm loc_33892 exits through loc_338F8 when loc_33550 reports contact"
        );
        assert_eq!(
            drips_surface_crawler_movement_tail_loc_33892(false, true, false, 0x1111),
            DripsSurfaceCrawlerMovementPathLoc33892::GuardExit,
            "ASM Drips.asm loc_33892 exits through loc_338F8 when loc_33588 reports boundary contact"
        );
        assert_eq!(
            drips_surface_crawler_movement_tail_loc_33892(false, false, true, 0x1111),
            DripsSurfaceCrawlerMovementPathLoc33892::NegativeYBranch,
            "ASM Drips.asm loc_33892 branches to loc_338CE when `y_vel(a3)` is negative"
        );
        assert_eq!(
            drips_surface_crawler_movement_tail_loc_33892(false, false, false, 0x5000),
            DripsSurfaceCrawlerMovementPathLoc33892::DiagonalLoop,
            "ASM Drips.asm loc_33892 loops to itself when the non-negative-y `loc_3330E` d7 probe returns `$5000`"
        );
        assert_eq!(
            drips_surface_crawler_movement_tail_loc_33892(false, false, false, 0x6000),
            DripsSurfaceCrawlerMovementPathLoc33892::SolidTurn,
            "ASM Drips.asm loc_33892 routes d7 `$6000` to loc_33A22"
        );
        assert_eq!(
            drips_surface_crawler_movement_tail_loc_33892(false, false, false, 0x1111),
            DripsSurfaceCrawlerMovementPathLoc33892::FallbackTurn,
            "ASM Drips.asm loc_33892 falls through to loc_339F0 for non-matching d7 flags"
        );
    }

    #[test]
    fn drips_surface_crawler_delete_tail_matches_loc_3392c() {
        assert_eq!(
            drips_surface_crawler_delete_tail_loc_3392c(),
            DripsSurfaceCrawlerDeleteTailLoc3392C {
                is_moved: false,
                waits_for_animation_to_finish: true,
                current_drips_delta: -1,
                deletes_current_object: true,
            },
            "ASM Drips.asm loc_3392C clears movement, waits for animation completion, decrements current Drips count, and deletes the object"
        );
    }

    #[test]
    fn drips_surface_crawler_first_animation_matches_loc_33b30() {
        assert_eq!(
            drips_surface_crawler_first_animation_loc_33b30([0xFF, 0xFC, 0x44, 0xFF]),
            DripsSurfaceCrawlerFirstAnimationLoc33B30 {
                x_delta: -1,
                y_delta: -4,
                animation: DripsSurfaceCrawlerTurnAnimationLoc33AF6::Stru33DBC,
                waits_for_animation_to_finish: true,
                flip_after_animation: true,
            },
            "ASM Drips.asm loc_33B30 sign-extends byte x/y deltas, runs `j_Hibernate_UntilAnimFinished`, and flips x_direction only for non-zero script byte"
        );
        assert_eq!(
            drips_surface_crawler_first_animation_loc_33b30([0x03, 0x01, 0x38, 0x00]),
            DripsSurfaceCrawlerFirstAnimationLoc33B30 {
                x_delta: 3,
                y_delta: 1,
                animation: DripsSurfaceCrawlerTurnAnimationLoc33AF6::Stru33DF4,
                waits_for_animation_to_finish: true,
                flip_after_animation: false,
            },
            "ASM Drips.asm loc_33B30 skips `not.b x_direction(a3)` when the post-animation byte is zero"
        );
    }

    #[test]
    fn drips_surface_crawler_second_animation_matches_loc_33b5e() {
        assert_eq!(
            drips_surface_crawler_second_animation_loc_33b5e([
                0xFB, 0x00, 0xD0, 0x00, 0x00, 0x00, 0x18, 0x00,
            ]),
            DripsSurfaceCrawlerSecondAnimationLoc33B5E {
                x_delta: -5,
                y_delta: 0,
                x_velocity_delta: -0x3000,
                y_velocity_delta: 0,
                animation: DripsSurfaceCrawlerTurnAnimationLoc33AF6::Stru33F14,
                applies_child_velocity_deltas: true,
                next: DripsSurfaceCrawlerTurnNextLoc33AF6::Loc33650,
            },
            "ASM Drips.asm loc_33B5E sign-extends byte position deltas, sign-extends word velocity deltas, runs loc_33526, and dispatches through off_33ADA"
        );
        assert_eq!(
            drips_surface_crawler_second_animation_loc_33b5e([
                0x00, 0xFE, 0x00, 0x00, 0xD0, 0x00, 0x14, 0x0C,
            ]),
            DripsSurfaceCrawlerSecondAnimationLoc33B5E {
                x_delta: 0,
                y_delta: -2,
                x_velocity_delta: 0,
                y_velocity_delta: -0x3000,
                animation: DripsSurfaceCrawlerTurnAnimationLoc33AF6::Stru33EB2,
                applies_child_velocity_deltas: true,
                next: DripsSurfaceCrawlerTurnNextLoc33AF6::Loc337A0,
            },
            "ASM Drips.asm loc_33B5E preserves zero X velocity deltas while applying a negative Y velocity word before the next movement route"
        );
    }

    #[test]
    fn drips_surface_crawler_turn_script_matches_loc_3393c() {
        assert_eq!(
            drips_surface_crawler_turn_script_loc_3393c(),
            DripsSurfaceCrawlerTurnScriptLoc33AF6 {
                snap_field: DripsSurfaceCrawlerTurnFieldLoc33AF6::XPos,
                snap_offset: 0,
                snap_mask: 0xFFF0,
                flip_before_first_animation: false,
                first_x_delta: 1,
                first_y_delta: 0,
                first_animation: DripsSurfaceCrawlerTurnAnimationLoc33AF6::Stru33E72,
                flip_after_first_animation: true,
                second_x_delta: 0,
                second_y_delta: -5,
                x_velocity_delta: 0,
                y_velocity_delta: -0x3000,
                second_animation: DripsSurfaceCrawlerTurnAnimationLoc33AF6::Stru33EB2,
                next: DripsSurfaceCrawlerTurnNextLoc33AF6::Loc33730,
            },
            "ASM Drips.asm loc_3393C selects `unk_33C80`, snapping X, stepping up, seeding -$3000 y velocity, and jumping to loc_33730"
        );
    }

    #[test]
    fn drips_surface_crawler_turn_script_matches_loc_33a40() {
        assert_eq!(
            drips_surface_crawler_turn_script_loc_33a40(),
            DripsSurfaceCrawlerTurnScriptLoc33AF6 {
                snap_field: DripsSurfaceCrawlerTurnFieldLoc33AF6::XPos,
                snap_offset: 0,
                snap_mask: 0xFFF0,
                flip_before_first_animation: false,
                first_x_delta: 1,
                first_y_delta: 0,
                first_animation: DripsSurfaceCrawlerTurnAnimationLoc33AF6::Stru33DA0,
                flip_after_first_animation: false,
                second_x_delta: -4,
                second_y_delta: -3,
                x_velocity_delta: -0x2000,
                y_velocity_delta: -0x2000,
                second_animation: DripsSurfaceCrawlerTurnAnimationLoc33AF6::Stru33EE6,
                next: DripsSurfaceCrawlerTurnNextLoc33AF6::Loc33892,
            },
            "ASM Drips.asm loc_33A40 selects `unk_33D10`, snapping X, seeding -$2000/-$2000 diagonal velocity, and jumping to loc_33892"
        );
    }

    #[test]
    fn drips_surface_crawler_turn_script_matches_loc_3395a() {
        assert_eq!(
            drips_surface_crawler_turn_script_loc_3395a(),
            DripsSurfaceCrawlerTurnScriptLoc33AF6 {
                snap_field: DripsSurfaceCrawlerTurnFieldLoc33AF6::YPos,
                snap_offset: 9,
                snap_mask: 0xFFF0,
                flip_before_first_animation: false,
                first_x_delta: 1,
                first_y_delta: 0,
                first_animation: DripsSurfaceCrawlerTurnAnimationLoc33AF6::Stru33E2C,
                flip_after_first_animation: false,
                second_x_delta: 5,
                second_y_delta: 0,
                x_velocity_delta: 0x3000,
                y_velocity_delta: 0,
                second_animation: DripsSurfaceCrawlerTurnAnimationLoc33AF6::Stru33F52,
                next: DripsSurfaceCrawlerTurnNextLoc33AF6::Loc336D0,
            },
            "ASM Drips.asm loc_3395A selects `unk_33C00`, which loc_33AF6 snaps Y by +9, moves right, seeds +$3000 x velocity, and jumps to loc_336D0"
        );
    }

    #[test]
    fn drips_surface_crawler_turn_script_matches_loc_33964() {
        assert_eq!(
            drips_surface_crawler_turn_script_loc_33964(),
            DripsSurfaceCrawlerTurnScriptLoc33AF6 {
                snap_field: DripsSurfaceCrawlerTurnFieldLoc33AF6::YPos,
                snap_offset: 0,
                snap_mask: 0xFFF0,
                flip_before_first_animation: true,
                first_x_delta: 0,
                first_y_delta: 1,
                first_animation: DripsSurfaceCrawlerTurnAnimationLoc33AF6::Stru33E56,
                flip_after_first_animation: false,
                second_x_delta: 5,
                second_y_delta: 0,
                x_velocity_delta: 0x3000,
                y_velocity_delta: 0,
                second_animation: DripsSurfaceCrawlerTurnAnimationLoc33AF6::Stru33F52,
                next: DripsSurfaceCrawlerTurnNextLoc33AF6::Loc336D0,
            },
            "ASM Drips.asm loc_33964 selects `unk_33C70`, snapping Y, stepping right, seeding +$3000 x velocity, and jumping to loc_336D0"
        );
    }

    #[test]
    fn drips_surface_crawler_turn_script_matches_loc_33946() {
        assert_eq!(
            drips_surface_crawler_turn_script_loc_33946(),
            DripsSurfaceCrawlerTurnScriptLoc33AF6 {
                snap_field: DripsSurfaceCrawlerTurnFieldLoc33AF6::YPos,
                snap_offset: 8,
                snap_mask: 0xFFF0,
                flip_before_first_animation: true,
                first_x_delta: 0,
                first_y_delta: 0,
                first_animation: DripsSurfaceCrawlerTurnAnimationLoc33AF6::Stru33E64,
                flip_after_first_animation: true,
                second_x_delta: 5,
                second_y_delta: 0,
                x_velocity_delta: 0x3000,
                y_velocity_delta: 0,
                second_animation: DripsSurfaceCrawlerTurnAnimationLoc33AF6::Stru33F14,
                next: DripsSurfaceCrawlerTurnNextLoc33AF6::Loc33650,
            },
            "ASM Drips.asm loc_33946 selects `unk_33C30`, snapping Y by +8, stepping right, seeding +$3000 x velocity, and resuming loc_33650"
        );
    }

    #[test]
    fn drips_surface_crawler_turn_script_matches_loc_33978() {
        assert_eq!(
            drips_surface_crawler_turn_script_loc_33978(),
            DripsSurfaceCrawlerTurnScriptLoc33AF6 {
                snap_field: DripsSurfaceCrawlerTurnFieldLoc33AF6::YPos,
                snap_offset: 0,
                snap_mask: 0xFFF0,
                flip_before_first_animation: false,
                first_x_delta: 1,
                first_y_delta: 0,
                first_animation: DripsSurfaceCrawlerTurnAnimationLoc33AF6::Stru33E1E,
                flip_after_first_animation: true,
                second_x_delta: 5,
                second_y_delta: 0,
                x_velocity_delta: 0x3000,
                y_velocity_delta: 0,
                second_animation: DripsSurfaceCrawlerTurnAnimationLoc33AF6::Stru33F14,
                next: DripsSurfaceCrawlerTurnNextLoc33AF6::Loc33650,
            },
            "ASM Drips.asm loc_33978 selects `unk_33BD0`, which loc_33AF6 snaps Y without offset, flips direction after the first animation, and resumes loc_33650"
        );
    }

    #[test]
    fn drips_surface_crawler_turn_script_matches_loc_3398c() {
        assert_eq!(
            drips_surface_crawler_turn_script_loc_3398c(),
            DripsSurfaceCrawlerTurnScriptLoc33AF6 {
                snap_field: DripsSurfaceCrawlerTurnFieldLoc33AF6::YPos,
                snap_offset: 9,
                snap_mask: 0xFFF0,
                flip_before_first_animation: true,
                first_x_delta: 0,
                first_y_delta: -1,
                first_animation: DripsSurfaceCrawlerTurnAnimationLoc33AF6::Stru33E64,
                flip_after_first_animation: true,
                second_x_delta: -5,
                second_y_delta: 0,
                x_velocity_delta: -0x3000,
                y_velocity_delta: 0,
                second_animation: DripsSurfaceCrawlerTurnAnimationLoc33AF6::Stru33F14,
                next: DripsSurfaceCrawlerTurnNextLoc33AF6::Loc33650,
            },
            "ASM Drips.asm loc_3398C selects `unk_33C90`, which loc_33AF6 snaps Y by +9, flips direction before and after the first animation, seeds -$3000 x velocity, and resumes loc_33650"
        );
    }

    #[test]
    fn drips_surface_crawler_turn_script_matches_loc_339a0() {
        assert_eq!(
            drips_surface_crawler_turn_script_loc_339a0(),
            DripsSurfaceCrawlerTurnScriptLoc33AF6 {
                snap_field: DripsSurfaceCrawlerTurnFieldLoc33AF6::YPos,
                snap_offset: 9,
                snap_mask: 0xFFF0,
                flip_before_first_animation: false,
                first_x_delta: -1,
                first_y_delta: 0,
                first_animation: DripsSurfaceCrawlerTurnAnimationLoc33AF6::Stru33E2C,
                flip_after_first_animation: false,
                second_x_delta: -5,
                second_y_delta: 0,
                x_velocity_delta: -0x3000,
                y_velocity_delta: 0,
                second_animation: DripsSurfaceCrawlerTurnAnimationLoc33AF6::Stru33F52,
                next: DripsSurfaceCrawlerTurnNextLoc33AF6::Loc336D0,
            },
            "ASM Drips.asm loc_339A0 selects `unk_33BB0`, snapping Y by +9, stepping left, seeding -$3000 x velocity, and jumping to loc_336D0"
        );
    }

    #[test]
    fn drips_surface_crawler_turn_script_matches_loc_339be() {
        assert_eq!(
            drips_surface_crawler_turn_script_loc_339be(),
            DripsSurfaceCrawlerTurnScriptLoc33AF6 {
                snap_field: DripsSurfaceCrawlerTurnFieldLoc33AF6::YPos,
                snap_offset: 0,
                snap_mask: 0xFFF0,
                flip_before_first_animation: true,
                first_x_delta: 0,
                first_y_delta: 1,
                first_animation: DripsSurfaceCrawlerTurnAnimationLoc33AF6::Stru33E56,
                flip_after_first_animation: false,
                second_x_delta: -5,
                second_y_delta: 0,
                x_velocity_delta: -0x3000,
                y_velocity_delta: 0,
                second_animation: DripsSurfaceCrawlerTurnAnimationLoc33AF6::Stru33F52,
                next: DripsSurfaceCrawlerTurnNextLoc33AF6::Loc336D0,
            },
            "ASM Drips.asm loc_339BE selects `unk_33C50`, which loc_33AF6 snaps Y without offset, flips before the first animation, seeds -$3000 x velocity, and jumps to loc_336D0"
        );
    }

    #[test]
    fn drips_surface_crawler_turn_script_matches_loc_339c8() {
        assert_eq!(
            drips_surface_crawler_turn_script_loc_339c8(),
            DripsSurfaceCrawlerTurnScriptLoc33AF6 {
                snap_field: DripsSurfaceCrawlerTurnFieldLoc33AF6::XPos,
                snap_offset: 8,
                snap_mask: 0xFFF0,
                flip_before_first_animation: false,
                first_x_delta: 0,
                first_y_delta: 0,
                first_animation: DripsSurfaceCrawlerTurnAnimationLoc33AF6::Stru33E10,
                flip_after_first_animation: false,
                second_x_delta: 1,
                second_y_delta: 5,
                x_velocity_delta: 0,
                y_velocity_delta: 0x3000,
                second_animation: DripsSurfaceCrawlerTurnAnimationLoc33AF6::Stru33E80,
                next: DripsSurfaceCrawlerTurnNextLoc33AF6::Loc33730,
            },
            "ASM Drips.asm loc_339C8 selects `unk_33BA0`, snapping X by +8, stepping down-right, seeding +$3000 y velocity, and jumping to loc_33730"
        );
    }

    #[test]
    fn drips_surface_crawler_turn_script_matches_loc_33982() {
        assert_eq!(
            drips_surface_crawler_turn_script_loc_33982(),
            DripsSurfaceCrawlerTurnScriptLoc33AF6 {
                snap_field: DripsSurfaceCrawlerTurnFieldLoc33AF6::XPos,
                snap_offset: 0,
                snap_mask: 0xFFF0,
                flip_before_first_animation: false,
                first_x_delta: 0,
                first_y_delta: 0,
                first_animation: DripsSurfaceCrawlerTurnAnimationLoc33AF6::Stru33E10,
                flip_after_first_animation: false,
                second_x_delta: -1,
                second_y_delta: 5,
                x_velocity_delta: 0,
                y_velocity_delta: 0x3000,
                second_animation: DripsSurfaceCrawlerTurnAnimationLoc33AF6::Stru33E80,
                next: DripsSurfaceCrawlerTurnNextLoc33AF6::Loc337A0,
            },
            "ASM Drips.asm loc_33982 selects `unk_33BF0`, snapping X, stepping down-left, seeding +$3000 y velocity, and jumping to loc_337A0"
        );
    }

    #[test]
    fn drips_surface_crawler_turn_script_matches_loc_339e6() {
        assert_eq!(
            drips_surface_crawler_turn_script_loc_339e6(),
            DripsSurfaceCrawlerTurnScriptLoc33AF6 {
                snap_field: DripsSurfaceCrawlerTurnFieldLoc33AF6::YPos,
                snap_offset: 0,
                snap_mask: 0xFFF0,
                flip_before_first_animation: false,
                first_x_delta: 3,
                first_y_delta: 1,
                first_animation: DripsSurfaceCrawlerTurnAnimationLoc33AF6::Stru33DF4,
                flip_after_first_animation: false,
                second_x_delta: 3,
                second_y_delta: -1,
                x_velocity_delta: 0x3000,
                y_velocity_delta: 0,
                second_animation: DripsSurfaceCrawlerTurnAnimationLoc33AF6::Stru33F14,
                next: DripsSurfaceCrawlerTurnNextLoc33AF6::Loc33650,
            },
            "ASM Drips.asm loc_339E6 selects `unk_33CB0`, which loc_33AF6 snaps Y without offset, steps down-right then up-right, seeds +$3000 x velocity, and resumes loc_33650"
        );
    }

    #[test]
    fn drips_surface_crawler_turn_script_matches_loc_339d2() {
        assert_eq!(
            drips_surface_crawler_turn_script_loc_339d2(),
            DripsSurfaceCrawlerTurnScriptLoc33AF6 {
                snap_field: DripsSurfaceCrawlerTurnFieldLoc33AF6::YPos,
                snap_offset: 0,
                snap_mask: 0xFFF0,
                flip_before_first_animation: false,
                first_x_delta: -1,
                first_y_delta: 0,
                first_animation: DripsSurfaceCrawlerTurnAnimationLoc33AF6::Stru33E1E,
                flip_after_first_animation: true,
                second_x_delta: -5,
                second_y_delta: 0,
                x_velocity_delta: -0x3000,
                y_velocity_delta: 0,
                second_animation: DripsSurfaceCrawlerTurnAnimationLoc33AF6::Stru33F14,
                next: DripsSurfaceCrawlerTurnNextLoc33AF6::Loc33650,
            },
            "ASM Drips.asm loc_339D2 selects `unk_33BE0`, snapping Y, stepping left, seeding -$3000 x velocity, and jumping to loc_33650"
        );
    }

    #[test]
    fn drips_surface_crawler_turn_script_matches_loc_33996() {
        assert_eq!(
            drips_surface_crawler_turn_script_loc_33996(),
            DripsSurfaceCrawlerTurnScriptLoc33AF6 {
                snap_field: DripsSurfaceCrawlerTurnFieldLoc33AF6::XPos,
                snap_offset: 7,
                snap_mask: 0xFFF0,
                flip_before_first_animation: false,
                first_x_delta: 0,
                first_y_delta: 0,
                first_animation: DripsSurfaceCrawlerTurnAnimationLoc33AF6::Stru33E72,
                flip_after_first_animation: true,
                second_x_delta: 0,
                second_y_delta: -5,
                x_velocity_delta: 0,
                y_velocity_delta: -0x3000,
                second_animation: DripsSurfaceCrawlerTurnAnimationLoc33AF6::Stru33EB2,
                next: DripsSurfaceCrawlerTurnNextLoc33AF6::Loc337A0,
            },
            "ASM Drips.asm loc_33996 selects `unk_33C20`, snapping X by +7, stepping up, seeding -$3000 y velocity, and jumping to loc_337A0"
        );
    }

    #[test]
    fn drips_surface_crawler_turn_script_matches_loc_339dc() {
        assert_eq!(
            drips_surface_crawler_turn_script_loc_339dc(),
            DripsSurfaceCrawlerTurnScriptLoc33AF6 {
                snap_field: DripsSurfaceCrawlerTurnFieldLoc33AF6::XPos,
                snap_offset: 7,
                snap_mask: 0xFFF0,
                flip_before_first_animation: false,
                first_x_delta: 0,
                first_y_delta: 0,
                first_animation: DripsSurfaceCrawlerTurnAnimationLoc33AF6::Stru33DA0,
                flip_after_first_animation: false,
                second_x_delta: 4,
                second_y_delta: -3,
                x_velocity_delta: 0x2000,
                y_velocity_delta: -0x2000,
                second_animation: DripsSurfaceCrawlerTurnAnimationLoc33AF6::Stru33EE6,
                next: DripsSurfaceCrawlerTurnNextLoc33AF6::Loc33810,
            },
            "ASM Drips.asm loc_339DC selects `unk_33CA0`, snapping X by +7, seeding +$2000/-$2000 diagonal velocity, and jumping to loc_33810"
        );
    }

    #[test]
    fn drips_surface_crawler_turn_script_matches_loc_33a2c() {
        assert_eq!(
            drips_surface_crawler_turn_script_loc_33a2c(),
            DripsSurfaceCrawlerTurnScriptLoc33AF6 {
                snap_field: DripsSurfaceCrawlerTurnFieldLoc33AF6::XPos,
                snap_offset: 0,
                snap_mask: 0xFFF0,
                flip_before_first_animation: true,
                first_x_delta: -2,
                first_y_delta: 1,
                first_animation: DripsSurfaceCrawlerTurnAnimationLoc33AF6::Stru33E02,
                flip_after_first_animation: false,
                second_x_delta: -2,
                second_y_delta: 3,
                x_velocity_delta: -0x2000,
                y_velocity_delta: 0x2000,
                second_animation: DripsSurfaceCrawlerTurnAnimationLoc33AF6::Stru33EF6,
                next: DripsSurfaceCrawlerTurnNextLoc33AF6::Loc33810,
            },
            "ASM Drips.asm loc_33A2C selects `unk_33CE0`, snapping X, flipping before the first animation, seeding -$2000/+$2000 velocity, and jumping to loc_33810"
        );
    }

    #[test]
    fn drips_surface_crawler_turn_script_matches_loc_33a18() {
        assert_eq!(
            drips_surface_crawler_turn_script_loc_33a18(),
            DripsSurfaceCrawlerTurnScriptLoc33AF6 {
                snap_field: DripsSurfaceCrawlerTurnFieldLoc33AF6::XPos,
                snap_offset: 8,
                snap_mask: 0xFFF0,
                flip_before_first_animation: true,
                first_x_delta: 2,
                first_y_delta: 1,
                first_animation: DripsSurfaceCrawlerTurnAnimationLoc33AF6::Stru33E02,
                flip_after_first_animation: false,
                second_x_delta: 3,
                second_y_delta: 4,
                x_velocity_delta: 0x2000,
                y_velocity_delta: 0x2000,
                second_animation: DripsSurfaceCrawlerTurnAnimationLoc33AF6::Stru33EF6,
                next: DripsSurfaceCrawlerTurnNextLoc33AF6::Loc33892,
            },
            "ASM Drips.asm loc_33A18 selects `unk_33CC0`, snapping X by +8, flipping before the first animation, seeding +$2000/+$2000 velocity, and jumping to loc_33892"
        );
    }

    #[test]
    fn drips_surface_crawler_turn_script_matches_loc_33a72() {
        assert_eq!(
            drips_surface_crawler_turn_script_loc_33a72(),
            DripsSurfaceCrawlerTurnScriptLoc33AF6 {
                snap_field: DripsSurfaceCrawlerTurnFieldLoc33AF6::YPos,
                snap_offset: 8,
                snap_mask: 0xFFF0,
                flip_before_first_animation: true,
                first_x_delta: -1,
                first_y_delta: -1,
                first_animation: DripsSurfaceCrawlerTurnAnimationLoc33AF6::Stru33DCA,
                flip_after_first_animation: false,
                second_x_delta: 3,
                second_y_delta: 4,
                x_velocity_delta: 0x2000,
                y_velocity_delta: 0x2000,
                second_animation: DripsSurfaceCrawlerTurnAnimationLoc33AF6::Stru33EF6,
                next: DripsSurfaceCrawlerTurnNextLoc33AF6::Loc33892,
            },
            "ASM Drips.asm loc_33A72 selects `unk_33D60`, snapping Y by +8, seeding +$2000/+$2000 diagonal velocity, and jumping to loc_33892"
        );
    }

    #[test]
    fn drips_surface_crawler_turn_script_matches_loc_339fa() {
        assert_eq!(
            drips_surface_crawler_turn_script_loc_339fa(),
            DripsSurfaceCrawlerTurnScriptLoc33AF6 {
                snap_field: DripsSurfaceCrawlerTurnFieldLoc33AF6::YPos,
                snap_offset: 0,
                snap_mask: 0xFFF0,
                flip_before_first_animation: true,
                first_x_delta: 0,
                first_y_delta: 0,
                first_animation: DripsSurfaceCrawlerTurnAnimationLoc33AF6::Stru33DD8,
                flip_after_first_animation: false,
                second_x_delta: 0,
                second_y_delta: 1,
                x_velocity_delta: -0x2000,
                y_velocity_delta: -0x2000,
                second_animation: DripsSurfaceCrawlerTurnAnimationLoc33AF6::Stru33EE6,
                next: DripsSurfaceCrawlerTurnNextLoc33AF6::Loc33892,
            },
            "ASM Drips.asm loc_339FA selects `unk_33D80`, snapping Y, seeding -$2000/-$2000 diagonal velocity, and jumping to loc_33892"
        );
    }

    #[test]
    fn drips_surface_crawler_turn_script_matches_loc_33a04() {
        assert_eq!(
            drips_surface_crawler_turn_script_loc_33a04(),
            DripsSurfaceCrawlerTurnScriptLoc33AF6 {
                snap_field: DripsSurfaceCrawlerTurnFieldLoc33AF6::YPos,
                snap_offset: 0,
                snap_mask: 0xFFF0,
                flip_before_first_animation: true,
                first_x_delta: 0,
                first_y_delta: 0,
                first_animation: DripsSurfaceCrawlerTurnAnimationLoc33AF6::Stru33DD8,
                flip_after_first_animation: false,
                second_x_delta: 0,
                second_y_delta: 0,
                x_velocity_delta: 0x2000,
                y_velocity_delta: -0x2000,
                second_animation: DripsSurfaceCrawlerTurnAnimationLoc33AF6::Stru33EE6,
                next: DripsSurfaceCrawlerTurnNextLoc33AF6::Loc33810,
            },
            "ASM Drips.asm loc_33A04 selects `unk_33CF0`, which loc_33AF6 snaps Y, flips before the first animation, seeds +$2000/-$2000 velocity, and jumps to loc_33810"
        );
    }

    #[test]
    fn drips_surface_crawler_turn_script_matches_loc_33a4a() {
        assert_eq!(
            drips_surface_crawler_turn_script_loc_33a4a(),
            DripsSurfaceCrawlerTurnScriptLoc33AF6 {
                snap_field: DripsSurfaceCrawlerTurnFieldLoc33AF6::YPos,
                snap_offset: 0,
                snap_mask: 0xFFF0,
                flip_before_first_animation: false,
                first_x_delta: -2,
                first_y_delta: 1,
                first_animation: DripsSurfaceCrawlerTurnAnimationLoc33AF6::Stru33DF4,
                flip_after_first_animation: false,
                second_x_delta: -2,
                second_y_delta: -1,
                x_velocity_delta: -0x3000,
                y_velocity_delta: 0,
                second_animation: DripsSurfaceCrawlerTurnAnimationLoc33AF6::Stru33F14,
                next: DripsSurfaceCrawlerTurnNextLoc33AF6::Loc33650,
            },
            "ASM Drips.asm loc_33A4A selects `unk_33D20`, which loc_33AF6 snaps Y, steps down-left then up-left, seeds -$3000 x velocity, and resumes loc_33650"
        );
    }

    #[test]
    fn drips_surface_crawler_turn_script_matches_loc_33a54() {
        assert_eq!(
            drips_surface_crawler_turn_script_loc_33a54(),
            DripsSurfaceCrawlerTurnScriptLoc33AF6 {
                snap_field: DripsSurfaceCrawlerTurnFieldLoc33AF6::XPos,
                snap_offset: 8,
                snap_mask: 0xFFF0,
                flip_before_first_animation: false,
                first_x_delta: -1,
                first_y_delta: -4,
                first_animation: DripsSurfaceCrawlerTurnAnimationLoc33AF6::Stru33DBC,
                flip_after_first_animation: true,
                second_x_delta: 0,
                second_y_delta: -2,
                x_velocity_delta: 0,
                y_velocity_delta: -0x3000,
                second_animation: DripsSurfaceCrawlerTurnAnimationLoc33AF6::Stru33EB2,
                next: DripsSurfaceCrawlerTurnNextLoc33AF6::Loc337A0,
            },
            "ASM Drips.asm loc_33A54 selects `unk_33D30`, which loc_33AF6 snaps X by +8, climbs left/up, flips after the first animation, seeds -$3000 y velocity, and jumps to loc_337A0"
        );
    }

    #[test]
    fn drips_surface_crawler_turn_script_matches_loc_33a5e() {
        assert_eq!(
            drips_surface_crawler_turn_script_loc_33a5e(),
            DripsSurfaceCrawlerTurnScriptLoc33AF6 {
                snap_field: DripsSurfaceCrawlerTurnFieldLoc33AF6::YPos,
                snap_offset: 9,
                snap_mask: 0xFFF0,
                flip_before_first_animation: true,
                first_x_delta: 0,
                first_y_delta: -1,
                first_animation: DripsSurfaceCrawlerTurnAnimationLoc33AF6::Stru33DCA,
                flip_after_first_animation: false,
                second_x_delta: -2,
                second_y_delta: 2,
                x_velocity_delta: -0x2000,
                y_velocity_delta: 0x2000,
                second_animation: DripsSurfaceCrawlerTurnAnimationLoc33AF6::Stru33EF6,
                next: DripsSurfaceCrawlerTurnNextLoc33AF6::Loc33810,
            },
            "ASM Drips.asm loc_33A5E selects `unk_33D40`, which loc_33AF6 snaps Y by +9, flips before the first animation, seeds -$2000/+$2000 velocity, and jumps to loc_33810"
        );
    }

    #[test]
    fn drips_surface_crawler_turn_script_matches_loc_33a68() {
        assert_eq!(
            drips_surface_crawler_turn_script_loc_33a68(),
            DripsSurfaceCrawlerTurnScriptLoc33AF6 {
                snap_field: DripsSurfaceCrawlerTurnFieldLoc33AF6::XPos,
                snap_offset: 0,
                snap_mask: 0xFFF0,
                flip_before_first_animation: false,
                first_x_delta: 1,
                first_y_delta: -3,
                first_animation: DripsSurfaceCrawlerTurnAnimationLoc33AF6::Stru33DBC,
                flip_after_first_animation: true,
                second_x_delta: 0,
                second_y_delta: 0,
                x_velocity_delta: 0,
                y_velocity_delta: -0x3000,
                second_animation: DripsSurfaceCrawlerTurnAnimationLoc33AF6::Stru33EB2,
                next: DripsSurfaceCrawlerTurnNextLoc33AF6::Loc33730,
            },
            "ASM Drips.asm loc_33A68 selects `unk_33D50`, which loc_33AF6 snaps X, climbs right/up, flips after the first animation, seeds -$3000 y velocity, and jumps to loc_33730"
        );
    }

    #[test]
    fn drips_drop_ignores_generic_projectile_slot_cap_loc_33424() {
        let mut enemy = ActiveEnemy {
            family: EnemyFamily::Drips,
            wait_timer: 0,
            ..mini_hopping_skull_test_enemy()
        };
        let mut projectiles = vec![EnemyProjectile::default(); MAX_PROJECTILES];

        for _ in 0..=DRIPS_PRE_DROP_ANIMATION_FRAMES_LOC_33424 {
            step_drips(&mut enemy, &mut projectiles, None);
        }

        assert_eq!(
            projectiles.len(),
            MAX_PROJECTILES + 1,
            "ASM Drips.asm loc_33424 reuses the preallocated falling-drop object, so unrelated generic projectiles do not consume the drop child slot",
        );
        assert_eq!(
            projectiles.last().map(|projectile| projectile.kind),
            Some(EnemyProjectileKind::DripsDrop),
            "ASM Drips.asm loc_33424 starts the dedicated falling child, not a generic enemy projectile",
        );
    }
}
