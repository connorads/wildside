use super::*;

// Private items from lib.rs that don't come through the glob import.
use super::{
    ACTIVATION_WINDOW_PADDING, ACTIVATION_WINDOW_X_LEAD, ACTIVATION_WINDOW_Y_LEAD, CameraState,
    EnemyProjectile, FLYER_RETENTION_X_PADDING, FLYER_RETENTION_Y_PADDING, MAX_PROJECTILES,
};

// MiniHoppingSkull hopper constants recovered from the enemy module.
pub(crate) const MINI_SKULL_JUMP_X_SPEED_FP: i32 = 0x0001_8000;
pub(crate) const MINI_SKULL_JUMP_Y_SPEED_FP: i32 = -0x0003_8000;
pub(crate) const MINI_SKULL_GRAVITY_FP: i32 = 0x0000_1200;
pub(crate) const MINI_SKULL_INITIAL_Y_SPEED_FP: i32 = 0x0002_8000;
pub(crate) const MINI_SKULL_LEVEL_INITIAL_Y_SPEED_BONUS_FP: i32 = 0x0000_1000;
pub(crate) const MINI_SKULL_GROUND_DELAY_FRAMES: u8 = 21; // ASM Mini_Hopping_Skull.asm:99,141 — dbf #$14 = 21 iterations

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[allow(dead_code)]
pub(crate) struct NinjaInitState {
    pub(crate) status_flag_13: bool,
    pub(crate) object_meta: u16,
    pub(crate) priority: u8,
    pub(crate) current_hp: u8,
    pub(crate) walk_speed_override: i32,
}

/// ASM Enemy11_Ninja_Init `loc_3D554` computes `current_hp` from layout level
/// and seeds shared ninja state used by the stealth pacer speed path.
#[allow(dead_code)]
pub(crate) fn ninja_init_state_loc_3d554(enemy_level: u8) -> NinjaInitState {
    let current_hp = spawn_hit_points(EnemyFamily::Ninja, enemy_level);
    NinjaInitState {
        status_flag_13: true,
        object_meta: 0x0011,
        priority: 0x00,
        current_hp,
        walk_speed_override: ninja_walk_speed_override_loc_3d59a_and_3d5aa(current_hp),
    }
}

/// ASM Enemy11_Ninja_Init `loc_3D59A`/`loc_3D5AA` final speed arithmetic in `$50/$5E(a5)`:
/// - `current_hp < 3`: one unconditional `loc_3D5AA` fallthrough path (`+$8000` ×2).
/// - `current_hp == 3`: `loc_3D59A` then `loc_3D5AA` (`+$8000` ×2).
/// - `current_hp > 3`: base `+$8000` + both branch adds (`+$8000` ×3), so net 3×.
///   This is the native selection used by `step_pacer` speed resolution.
#[allow(dead_code)]
pub(crate) fn ninja_walk_speed_override_loc_3d59a_and_3d5aa(current_hp: u8) -> i32 {
    if current_hp > 3 {
        0x0002_4000
    } else {
        0x0001_8000
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) struct LionInitVisibleState {
    pub(crate) status_flag_13: bool,
    pub(crate) object_meta: u16,
    pub(crate) palette_line: u8,
    pub(crate) current_hp: u16,
    pub(crate) facing: Direction,
}

#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn lion_init_visible_state_loc_3df92(
    enemy_level: u16,
    lion_x_word: u16,
    kid_x_word: u16,
) -> LionInitVisibleState {
    // ASM Lion.asm loc_3DF92: after the signed spawn-facing compare, mark the
    // object visible via `$13(a3)`, seed object_meta `$12`, initialise art,
    // force palette line 2, set object_meta bit 7, and store `enemy_level+6`
    // into current_hp.
    let facing = if (kid_x_word as i16) < (lion_x_word as i16) {
        Direction::Left
    } else {
        Direction::Right
    };

    LionInitVisibleState {
        status_flag_13: true,
        object_meta: 0x0012 | 0x0080,
        palette_line: 2,
        current_hp: enemy_level.wrapping_add(6),
        facing,
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) struct LionInitMotionProfile {
    pub(crate) walk_speed: i32,
    pub(crate) charge_speed: i32,
    pub(crate) animation_script: &'static str,
}

#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn lion_init_motion_profile_loc_3dfd8(current_hp: u16) -> LionInitMotionProfile {
    // ASM Lion.asm loc_3DFD8 is reached by `cmpi.w #8,d7; bne.s`: every HP
    // value except exactly 8 uses `$18000` for both `$50(a5)` and `$5E(a5)`
    // and installs `stru_3DEDA`.
    if current_hp == 8 {
        LionInitMotionProfile {
            walk_speed: 0x0002_8000,
            charge_speed: 0x0002_8000,
            animation_script: "stru_3DEF4",
        }
    } else {
        LionInitMotionProfile {
            walk_speed: 0x0001_8000,
            charge_speed: 0x0001_8000,
            animation_script: "stru_3DEDA",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) struct LionInitRuntimeFields {
    pub(crate) ufo_beam_interval: u16,
    pub(crate) ufo_beam_segments: u16,
    pub(crate) tracking_callback: &'static str,
    pub(crate) animation_script: &'static str,
    pub(crate) shared_walker_state: i16,
    pub(crate) charge_timer: u16,
    pub(crate) collision_routine: &'static str,
    pub(crate) random_phase_byte: u8,
}

#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn lion_init_runtime_fields_loc_3dff0(
    motion_profile: LionInitMotionProfile,
    random_byte: u8,
) -> LionInitRuntimeFields {
    // ASM Lion.asm loc_3DFF0: common tail after the HP-specific motion branch.
    // It writes `$4A/$48/$6A`, initialises the selected animation, seeds
    // `$42(a3)` and `$54(a5)`, points a0 at the Lion collision routine, then
    // stores the RNG byte in `$71(a5)` with the original zero-byte remap to `$4B`.
    LionInitRuntimeFields {
        ufo_beam_interval: 0x002F,
        ufo_beam_segments: 0x0014,
        tracking_callback: "LionTrackingCallback",
        animation_script: motion_profile.animation_script,
        shared_walker_state: 0x000A,
        charge_timer: 0x003C,
        collision_routine: "LionCollisionRoutine",
        random_phase_byte: if random_byte == 0 { 0x4B } else { random_byte },
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) enum LionFacingDispatch {
    LeftPacing,
    RightPacing,
}

#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn lion_init_facing_dispatch_loc_3e030(facing: Direction) -> LionFacingDispatch {
    // ASM Lion.asm loc_3E030: `tst.b x_direction(a3); beq.w loc_3E546;
    // bra.w loc_3E392`. For Lion art, clear x_direction is the left-facing
    // pacing loop and set x_direction is the right-facing pacing loop.
    match facing {
        Direction::Left => LionFacingDispatch::LeftPacing,
        Direction::Right => LionFacingDispatch::RightPacing,
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) enum LionTrackingGate {
    Hold,
    TrackKid,
    ClearLatchAndTrackKid,
}

#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn lion_tracking_gate_loc_3e054(
    lion_x_word: u16,
    kid_x_word: u16,
    range_latch_set: bool,
) -> LionTrackingGate {
    let lion_x = i16::from_ne_bytes(lion_x_word.to_ne_bytes());
    let kid_x = i16::from_ne_bytes(kid_x_word.to_ne_bytes());
    let delta = lion_x.wrapping_sub(kid_x);
    // ASM Lion.asm loc_3E054: after the close [-$3F,+$40] band has been
    // rejected, clear `$4E(a5)` only outside the wider [-$9F,+$9F] band.
    // While `$4E(a5)` is clear, fall straight through to the facing update.
    if !range_latch_set {
        return LionTrackingGate::TrackKid;
    }

    if (-0x9F..=0x9F).contains(&delta) {
        LionTrackingGate::Hold
    } else {
        LionTrackingGate::ClearLatchAndTrackKid
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) struct LionLatchClearResult {
    pub(crate) range_latch_set: bool,
    pub(crate) facing: Direction,
}

#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn lion_clear_tracking_latch_loc_3e066(
    lion_x_word: u16,
    kid_x_word: u16,
) -> LionLatchClearResult {
    let lion_x = i16::from_ne_bytes(lion_x_word.to_ne_bytes());
    let kid_x = i16::from_ne_bytes(kid_x_word.to_ne_bytes());
    let delta = lion_x.wrapping_sub(kid_x);
    // ASM Lion.asm loc_3E066: `sf $4E(a5)` clears the tracking latch before
    // the same signed `d4 = lion_x - kid_x` value selects the Lion's facing.
    LionLatchClearResult {
        range_latch_set: false,
        facing: if delta < 0 {
            Direction::Right
        } else {
            Direction::Left
        },
    }
}

#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn lion_track_kid_nonnegative_delta_facing_loc_3e06a(
    lion_x_word: u16,
    kid_x_word: u16,
) -> Direction {
    let lion_x = i16::from_ne_bytes(lion_x_word.to_ne_bytes());
    let kid_x = i16::from_ne_bytes(kid_x_word.to_ne_bytes());
    let delta = lion_x.wrapping_sub(kid_x);
    // ASM Lion.asm loc_3E06A: `tst.w d4`; negative values branch away,
    // otherwise the fallthrough executes `sf x_direction(a3)`.
    // The fallthrough path handles zero and positive deltas by clearing
    // x_direction, which is the Lion's left-facing orientation.
    if delta < 0 {
        Direction::Right
    } else {
        Direction::Left
    }
}

#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn lion_track_kid_negative_delta_facing_loc_3e074(
    lion_x_word: u16,
    kid_x_word: u16,
) -> Direction {
    let lion_x = i16::from_ne_bytes(lion_x_word.to_ne_bytes());
    let kid_x = i16::from_ne_bytes(kid_x_word.to_ne_bytes());
    let delta = lion_x.wrapping_sub(kid_x);
    // ASM Lion.asm loc_3E074: the negative `d4 = lion_x - kid_x` branch
    // executes `st x_direction(a3)`, selecting the right-facing orientation.
    if delta < 0 {
        Direction::Right
    } else {
        Direction::Left
    }
}

pub(crate) const LION_COLID_HURT: u16 = 0x001C;
pub(crate) const LION_COLID_KIDABOVE: u16 = 0x002C;
pub(crate) const NINJA_COLID_FLOOR: u16 = 0x000C;
pub(crate) const NINJA_COLID_HURT: u16 = 0x001C;
pub(crate) const NINJA_COLID_KIDABOVE: u16 = 0x002C;
pub(crate) const NINJA_COLID_KIDRIGHT: u16 = 0x0020;
pub(crate) const NINJA_COLID_KIDLEFT: u16 = 0x0024;
pub(crate) const CRAB_COLID_HURT: i16 = 0x001C;
pub(crate) const CRAB_COLID_KIDRIGHT: i16 = 0x0020;
pub(crate) const CRAB_COLID_KIDLEFT: i16 = 0x0024;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) struct CrabInitVisibleStateLoc3AFD0 {
    pub(crate) status_flag_13: bool,
    pub(crate) is_animated: bool,
    pub(crate) object_meta: u16,
    pub(crate) priority: u8,
    pub(crate) current_hp: u8,
    pub(crate) speed_preload: i32,
}

#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn crab_init_visible_state_loc_3afd0(enemy_level: u8) -> CrabInitVisibleStateLoc3AFD0 {
    // ASM Crab.asm loc_3AFD0: set `$13(a3)` and `is_animated(a3)`,
    // write object_meta 9, clear priority, store `enemy_level + 2` HP, then
    // preload `$50/$5E(a5)` with `$10000` only when HP is at least 3.
    let current_hp = spawn_hit_points(EnemyFamily::Crab, enemy_level);
    CrabInitVisibleStateLoc3AFD0 {
        status_flag_13: true,
        is_animated: true,
        object_meta: 0x0009,
        priority: 0,
        current_hp,
        speed_preload: if current_hp >= 3 { 0x0001_0000 } else { 0 },
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) struct CrabMotionProfileLoc3B00A {
    pub(crate) walk_speed: i32,
    pub(crate) stored_speed: i32,
    pub(crate) timer_4a: u16,
    pub(crate) timer_48: u16,
    pub(crate) animation_script: &'static str,
    pub(crate) increments_enemy_count: bool,
}

#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn crab_motion_profile_loc_3b00a(current_hp: u8) -> CrabMotionProfileLoc3B00A {
    // ASM Crab.asm loc_3B00A: both speed slots receive one final `$8000` add
    // after loc_3AFD0's optional `$10000` preload, then `$4A/$48` and
    // `stru_3AF62` are installed before the shared walker loop.
    let speed = if current_hp >= 3 {
        0x0001_8000
    } else {
        0x0000_8000
    };
    CrabMotionProfileLoc3B00A {
        walk_speed: speed,
        stored_speed: speed,
        timer_4a: 0x001E,
        timer_48: 0x000C,
        animation_script: "stru_3AF62",
        increments_enemy_count: true,
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) enum CrabCollisionRouteLoc3B048 {
    ContinueSharedWalker,
    Death,
    BerzerkerSideGate,
    Damage,
}

#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn crab_collision_dispatch_loc_3b048(
    collision_type: i16,
    using_freeze_sprite: bool,
    crab_x_minus_kid_x: i16,
    crab_facing: Direction,
) -> CrabCollisionRouteLoc3B048 {
    // ASM Crab.asm loc_3B048: read `collision_type(a3)` before clearing it.
    // Zero resumes walking, negative values enter the death path, side contacts
    // are gated by Berzerker charging, and `colid_hurt` only decrements HP when
    // the frozen sprite is not active and Kid is on the side the crab faces.
    if collision_type == 0 {
        return CrabCollisionRouteLoc3B048::ContinueSharedWalker;
    }
    if collision_type.is_negative() {
        return CrabCollisionRouteLoc3B048::Death;
    }
    match collision_type {
        CRAB_COLID_KIDRIGHT | CRAB_COLID_KIDLEFT => CrabCollisionRouteLoc3B048::BerzerkerSideGate,
        CRAB_COLID_HURT if !using_freeze_sprite => {
            let kid_is_right_of_crab = crab_x_minus_kid_x < 0;
            let crab_faces_kid_side = if kid_is_right_of_crab {
                crab_facing == Direction::Right
            } else {
                crab_facing == Direction::Left
            };
            if crab_faces_kid_side {
                CrabCollisionRouteLoc3B048::Damage
            } else {
                CrabCollisionRouteLoc3B048::ContinueSharedWalker
            }
        }
        _ => CrabCollisionRouteLoc3B048::ContinueSharedWalker,
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) enum LionRoarWaitDispatch {
    Damage,
    ContinueWaiting {
        next_collision_type: u16,
    },
    ReadyToFire {
        next_collision_type: u16,
        facing: Direction,
        projectile_worker_selector: u16,
        projectile_worker: &'static str,
        active_projectile_count_delta: u8,
    },
}

#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn lion_roar_wait_dispatch_loc_3e09e(
    collision_type: u16,
    roar_fire_sprite_reached: bool,
    lion_x_word: u16,
    kid_x_word: u16,
) -> LionRoarWaitDispatch {
    // ASM Lion.asm loc_3E09E: after one hibernated frame, `colid_hurt` and
    // `colid_kidabove` jump to the damage path before the normal collision
    // clear. Non-damaging frames loop until the roar animation reaches its
    // fire sprite; then the Lion faces Kid, allocates selector `$8000`, installs
    // the projectile worker, and increments `$5B(a5)`.
    if matches!(collision_type, LION_COLID_HURT | LION_COLID_KIDABOVE) {
        return LionRoarWaitDispatch::Damage;
    }

    if !roar_fire_sprite_reached {
        return LionRoarWaitDispatch::ContinueWaiting {
            next_collision_type: 0,
        };
    }

    let lion_x = i16::from_ne_bytes(lion_x_word.to_ne_bytes());
    let kid_x = i16::from_ne_bytes(kid_x_word.to_ne_bytes());
    let delta = kid_x.wrapping_sub(lion_x);
    LionRoarWaitDispatch::ReadyToFire {
        next_collision_type: 0,
        facing: if delta > 0 {
            Direction::Right
        } else {
            Direction::Left
        },
        projectile_worker_selector: 0x8000,
        projectile_worker: "LionProjectileWorker",
        active_projectile_count_delta: 1,
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) enum LionPostFireWaitDispatch {
    Damage,
    ContinueWaiting {
        next_collision_type: u16,
    },
    ResumeCharge {
        next_collision_type: u16,
        animation_script: &'static str,
    },
}

#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn lion_post_fire_wait_dispatch_loc_3e0f4(
    collision_type: u16,
    animation_done_flag_18: bool,
) -> LionPostFireWaitDispatch {
    // ASM Lion.asm loc_3E0F4: the post-fire parent waits one hibernated frame,
    // sends hurt/Kid-above collisions to the damage path before clearing
    // `collision_type`, loops while `$18(a3)` is clear, then restores the
    // animation pointer saved in `$62(a5)` before the direction dispatch.
    if matches!(collision_type, LION_COLID_HURT | LION_COLID_KIDABOVE) {
        return LionPostFireWaitDispatch::Damage;
    }

    if !animation_done_flag_18 {
        return LionPostFireWaitDispatch::ContinueWaiting {
            next_collision_type: 0,
        };
    }

    LionPostFireWaitDispatch::ResumeCharge {
        next_collision_type: 0,
        animation_script: "saved_lion_charge_animation",
    }
}

#[cfg_attr(not(test), allow(dead_code))]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum LionCollisionResumeLoc3E318 {
    Loc3E7CA,
    Loc3E6AE,
    Loc3E560,
    Loc3E3AC,
}

#[cfg_attr(not(test), allow(dead_code))]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum LionCollisionLoc3E318 {
    Kill,
    Hurt {
        remaining_hp: u16,
        damage_animation: &'static str,
        restored_animation: &'static str,
    },
    Continue {
        next: LionCollisionResumeLoc3E318,
    },
}

#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn lion_collision_response_loc_3e318(
    collision_type: i16,
    current_hp: u16,
    post_charge_state: bool,
    x_direction_right: bool,
) -> LionCollisionLoc3E318 {
    // ASM Lion.asm loc_3E318:
    // - negative collision IDs (`bmi`) kill Lion via `stru_3DF48`, set
    //   `has_kid_collision`, and clear `$71(a5)`.
    // - `colid_hurt` and `colid_kidabove` subtract one HP; if HP reaches 0,
    //   `loc_3E378` is taken as a kill path.
    // - otherwise, clear `collision_type` and branch by `$5C(a5)` and
    //   `x_direction(a3)` to one of `loc_3E7CA/3E6AE/3E560/3E3AC`.
    if collision_type.is_negative() {
        return LionCollisionLoc3E318::Kill;
    }

    match collision_type as u16 {
        0 => {
            let next = match (post_charge_state, x_direction_right) {
                (true, true) => LionCollisionResumeLoc3E318::Loc3E6AE,
                (true, false) => LionCollisionResumeLoc3E318::Loc3E7CA,
                (false, true) => LionCollisionResumeLoc3E318::Loc3E3AC,
                (false, false) => LionCollisionResumeLoc3E318::Loc3E560,
            };
            LionCollisionLoc3E318::Continue { next }
        }
        LION_COLID_KIDABOVE | LION_COLID_HURT => {
            let remaining_hp = current_hp.wrapping_sub(1);
            if remaining_hp == 0 {
                LionCollisionLoc3E318::Kill
            } else {
                LionCollisionLoc3E318::Hurt {
                    remaining_hp,
                    damage_animation: "stru_3DF42",
                    restored_animation: "saved_lion_charge_animation",
                }
            }
        }
        _ => {
            let next = match (post_charge_state, x_direction_right) {
                (true, true) => LionCollisionResumeLoc3E318::Loc3E6AE,
                (true, false) => LionCollisionResumeLoc3E318::Loc3E7CA,
                (false, true) => LionCollisionResumeLoc3E318::Loc3E3AC,
                (false, false) => LionCollisionResumeLoc3E318::Loc3E560,
            };
            LionCollisionLoc3E318::Continue { next }
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) enum LionChargeDirectionDispatch {
    LeftCharge,
    RightCharge,
}

#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn lion_charge_direction_dispatch_loc_3e11e(
    facing: Direction,
) -> LionChargeDirectionDispatch {
    // ASM Lion.asm loc_3E11E: `tst.b x_direction(a3); beq.w loc_3E566;
    // bra.w loc_3E3B2`. Clear x_direction selects the Lion's left-facing
    // charge loop; set x_direction selects the right-facing charge loop.
    match facing {
        Direction::Left => LionChargeDirectionDispatch::LeftCharge,
        Direction::Right => LionChargeDirectionDispatch::RightCharge,
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) struct LionProjectileLaunchTail {
    pub(crate) x_offset_pixels: i16,
    pub(crate) x_velocity: i32,
    pub(crate) animation_script: &'static str,
}

#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn lion_projectile_launch_tail_loc_3e180(
    parent_facing: Direction,
) -> LionProjectileLaunchTail {
    // ASM Lion.asm loc_3E180: child setup enters with `d7` as the projectile
    // X offset and `d6` as X velocity. Clear parent `x_direction` negates both
    // before the label; set parent `x_direction` keeps `+$A` and `+$60000`.
    // The label applies `d7` to `x_pos(a3)`, stores `d6` in `x_vel(a3)`, then
    // initialises `stru_3DF34`.
    match parent_facing {
        Direction::Left => LionProjectileLaunchTail {
            x_offset_pixels: -(LION_PROJECTILE_X_OFFSET as i16),
            x_velocity: -LION_PROJECTILE_SPEED,
            animation_script: "stru_3DF34",
        },
        Direction::Right => LionProjectileLaunchTail {
            x_offset_pixels: LION_PROJECTILE_X_OFFSET as i16,
            x_velocity: LION_PROJECTILE_SPEED,
            animation_script: "stru_3DF34",
        },
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) struct LionHibernateLoc3E3A2 {
    pub(crate) x_velocity: i32,
    pub(crate) y_velocity: i32,
}

#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn lion_hibernate_tail_loc_3e3a2() -> LionHibernateLoc3E3A2 {
    // ASM Lion.asm loc_3E3A2: clear `x_vel` and `y_vel` when the routine hands
    // off to `sub_3E8BA` after a short movement decision.
    LionHibernateLoc3E3A2 {
        x_velocity: 0,
        y_velocity: 0,
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) enum LionLoc3E406Branch {
    ContinueTo3E434,
    EnterLoc3E3A2,
    EnterLoc3E48E,
}

#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn lion_charge_terrain_gate_loc_3e406(
    bit0_set: bool,
    object_meta_low_byte_is_0x10: bool,
) -> LionLoc3E406Branch {
    // ASM Lion.asm loc_3E406: when terrain bit 0 is clear, continue into
    // `loc_3E434`; if terrain bit 0 is set and object_meta low byte is $10,
    // force the shared charge timeout to $FF88 and reinitialize the walk speed/
    // animation before jumping to `loc_3E3A2`; otherwise continue to
    // `loc_3E48E`.
    if !bit0_set {
        return LionLoc3E406Branch::ContinueTo3E434;
    }

    if object_meta_low_byte_is_0x10 {
        LionLoc3E406Branch::EnterLoc3E3A2
    } else {
        LionLoc3E406Branch::EnterLoc3E48E
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) enum LionLoc3E434Branch {
    KeepAndContinue,
    Check60Band,
    Check70Band,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) struct LionLoc3E434Result {
    pub(crate) next_shared_state: i16,
    pub(crate) branch: LionLoc3E434Branch,
}

fn lion_kid_minus_lion_x_delta(lion_x_word: u16, kid_x_word: u16) -> i16 {
    let lion_x = i16::from_be_bytes(lion_x_word.to_be_bytes());
    let kid_x = i16::from_be_bytes(kid_x_word.to_be_bytes());
    kid_x.wrapping_sub(lion_x)
}

#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn lion_charge_window_gate_loc_3e434(shared_walker_state: i16) -> LionLoc3E434Result {
    // ASM Lion.asm loc_3E434:
    // - `$42 == $A` skips the inner distance gates and continues movement.
    // - `$42 > 0` routes to the tighter ±$70 check in `loc_3E476`.
    // - `$42 == 0` routes to the looser ±$60 check in `loc_3E450`.
    // - `$42 < 0` increments toward zero and continues movement.
    if shared_walker_state > 0 || shared_walker_state == 0x000A {
        LionLoc3E434Result {
            next_shared_state: shared_walker_state,
            branch: LionLoc3E434Branch::Check70Band,
        }
    } else if shared_walker_state == 0 {
        LionLoc3E434Result {
            next_shared_state: 0,
            branch: LionLoc3E434Branch::Check60Band,
        }
    } else {
        LionLoc3E434Result {
            next_shared_state: shared_walker_state.saturating_add(1),
            branch: LionLoc3E434Branch::KeepAndContinue,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) enum LionLoc3E604Branch {
    ContinueMovement,
    EnterLoc3E4CE,
    EnterLoc3E50A,
}

#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn lion_left_charge_calm_distance_gate_loc_3e604(
    lion_x_word: u16,
    kid_x_word: u16,
) -> LionLoc3E604Branch {
    // ASM Lion.asm loc_3E604: `d4 = Kid.x - Lion.x` as a signed word. Values
    // outside [-$60,+$60] continue the candidate movement; inside that band,
    // positive deltas enter loc_3E4CE and zero/negative deltas enter loc_3E50A.
    let delta = lion_kid_minus_lion_x_delta(lion_x_word, kid_x_word);
    if !(-0x60..=0x60).contains(&delta) {
        LionLoc3E604Branch::ContinueMovement
    } else if delta > 0 {
        LionLoc3E604Branch::EnterLoc3E4CE
    } else {
        LionLoc3E604Branch::EnterLoc3E50A
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) enum LionLoc3E62ABranch {
    ContinueMovement,
    EnterLoc3E642,
}

#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn lion_left_charge_alert_release_gate_loc_3e62a(
    lion_x_word: u16,
    kid_x_word: u16,
) -> LionLoc3E62ABranch {
    // ASM Lion.asm loc_3E62A: after `d4 = Kid.x - Lion.x`, `bgt` makes the
    // release band asymmetric at the negative edge: +$70 continues, but -$70
    // falls through to loc_3E642.
    let delta = lion_kid_minus_lion_x_delta(lion_x_word, kid_x_word);
    if delta > 0x70 || delta <= -0x70 {
        LionLoc3E62ABranch::EnterLoc3E642
    } else {
        LionLoc3E62ABranch::ContinueMovement
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) struct LionLoc3E4C6Result {
    pub(crate) x_position: u32,
    pub(crate) x_velocity: i32,
    pub(crate) y_velocity: i32,
}

#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn lion_right_charge_continue_loc_3e4c6(next_x_position: u32) -> LionLoc3E4C6Result {
    // ASM Lion.asm loc_3E4C6: `move.l d2,x_pos(a3)` commits the right-charge
    // candidate position, then branches to loc_3E3A2 to zero both velocities.
    let tail = lion_hibernate_tail_loc_3e3a2();
    LionLoc3E4C6Result {
        x_position: next_x_position,
        x_velocity: tail.x_velocity,
        y_velocity: tail.y_velocity,
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) struct LionLoc3E67AResult {
    pub(crate) x_position: u32,
    pub(crate) hibernate_one_frame: bool,
    pub(crate) cull_routine: &'static str,
    pub(crate) resume_via_callback: bool,
}

#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn lion_left_charge_continue_loc_3e67a(next_x_position: u32) -> LionLoc3E67AResult {
    // ASM Lion.asm loc_3E67A: `move.l d2,x_pos(a3)` commits the left-charge
    // candidate position, then branches to loc_3E556. Unlike loc_3E4C6, this
    // path does not clear velocity fields before the hibernate/cull/resume tail.
    let tail = lion_hibernate_cull_resume_loc_3e556();
    LionLoc3E67AResult {
        x_position: next_x_position,
        hibernate_one_frame: tail.hibernate_one_frame,
        cull_routine: tail.cull_routine,
        resume_via_callback: tail.resume_via_callback,
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) struct LionLeftChargeXCommitLoc3E6F0 {
    pub(crate) x_position: u32,
    pub(crate) enters_vertical_motion: bool,
}

#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn lion_left_charge_x_commit_loc_3e6f0(
    candidate_x_position: u32,
) -> LionLeftChargeXCommitLoc3E6F0 {
    // ASM Lion.asm loc_3E6F0: unobstructed left-charge movement commits `d2`
    // into `x_pos(a3)` and falls through to the vertical motion block.
    LionLeftChargeXCommitLoc3E6F0 {
        x_position: candidate_x_position,
        enters_vertical_motion: true,
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) enum LionLeftChargeVerticalEntryLoc3E6F6 {
    DownwardOrStationary {
        candidate_y_position: u32,
        pixel_delta: i16,
    },
    UpwardCollisionCheck {
        candidate_y_position: u32,
        upward_pixels: u16,
        ceiling_clearance_pixels: u16,
        needs_ceiling_probe: bool,
    },
}

#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn lion_left_charge_vertical_entry_loc_3e6f6(
    y_position: u32,
    y_velocity: i32,
    ceiling_probe_offset: u16,
) -> LionLeftChargeVerticalEntryLoc3E6F6 {
    // ASM Lion.asm loc_3E6F6: add `y_vel` to `y_pos`, compare the integer
    // words, and only run the upward ceiling-clearance path when the signed
    // word delta is negative.
    let candidate_y_position = y_position.wrapping_add_signed(y_velocity);
    let current_y_word = fixed_integer_word(y_position) as u16;
    let candidate_y_word = fixed_integer_word(candidate_y_position) as u16;
    let delta_word = candidate_y_word.wrapping_sub(current_y_word);
    let pixel_delta = i16::from_be_bytes(delta_word.to_be_bytes());

    if pixel_delta >= 0 {
        return LionLeftChargeVerticalEntryLoc3E6F6::DownwardOrStationary {
            candidate_y_position,
            pixel_delta,
        };
    }

    let upward_pixels = delta_word.wrapping_neg();
    let ceiling_clearance_pixels = current_y_word.wrapping_sub(ceiling_probe_offset) & 0x000F;
    LionLeftChargeVerticalEntryLoc3E6F6::UpwardCollisionCheck {
        candidate_y_position,
        upward_pixels,
        ceiling_clearance_pixels,
        needs_ceiling_probe: ceiling_clearance_pixels < upward_pixels,
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) enum LionLeftChargeDownwardCollisionLoc3E732 {
    Airborne {
        y_position: u32,
        floor_clearance_pixels: u16,
        needs_floor_probe: bool,
    },
    Landed {
        y_position: u32,
        y_velocity: i32,
        left_charge_latch_set: bool,
        object_meta_bit7_cleared: bool,
        restores_saved_charge_speed: bool,
        next_shared_walker_state: Option<i16>,
        restored_animation: &'static str,
    },
}

#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn lion_left_charge_downward_collision_loc_3e732(
    y_position: u32,
    candidate_y_position: u32,
    downward_pixels: u16,
    floor_probe_hit: bool,
    shared_walker_state: i16,
) -> LionLeftChargeDownwardCollisionLoc3E732 {
    // ASM Lion.asm loc_3E732: downward motion first checks the low-nibble
    // clearance to the next 16px boundary. Only movement crossing that boundary
    // probes floor terrain; a hit snaps to the boundary and restores charge.
    let current_y_word = fixed_integer_word(y_position) as u16;
    let floor_clearance_pixels = current_y_word.wrapping_neg() & 0x000F;
    let needs_floor_probe = floor_clearance_pixels < downward_pixels;

    if !needs_floor_probe || !floor_probe_hit {
        return LionLeftChargeDownwardCollisionLoc3E732::Airborne {
            y_position: candidate_y_position,
            floor_clearance_pixels,
            needs_floor_probe,
        };
    }

    let snapped_y_word = current_y_word.wrapping_add(floor_clearance_pixels);
    LionLeftChargeDownwardCollisionLoc3E732::Landed {
        y_position: u32::from(snapped_y_word) << 16,
        y_velocity: 0,
        left_charge_latch_set: false,
        object_meta_bit7_cleared: true,
        restores_saved_charge_speed: true,
        next_shared_walker_state: (shared_walker_state != 0x000A).then_some(-0x78),
        restored_animation: "saved_lion_charge_animation",
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) struct LionLeftChargeEntryLoc3E682 {
    pub(crate) left_charge_latch_set: bool,
    pub(crate) restored_sprite_offset: Option<u16>,
    pub(crate) is_animated: Option<bool>,
}

#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn lion_left_charge_entry_loc_3e682(
    saved_sprite_offset: u16,
) -> LionLeftChargeEntryLoc3E682 {
    // ASM Lion.asm loc_3E682: set `$5C(a5)`, then only restore
    // `addroffset_sprite(a3)` and clear `is_animated(a3)` when `$4A(a3)` is
    // non-zero; zero skips straight to the motion-seed tail.
    LionLeftChargeEntryLoc3E682 {
        left_charge_latch_set: true,
        restored_sprite_offset: (saved_sprite_offset != 0).then_some(saved_sprite_offset),
        is_animated: (saved_sprite_offset != 0).then_some(false),
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) struct LionLeftChargeMotionSeedLoc3E694 {
    pub(crate) x_velocity: i32,
    pub(crate) y_velocity: i32,
    pub(crate) y_position: u32,
}

#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn lion_left_charge_motion_seed_loc_3e694(
    saved_x_velocity: i32,
    y_position: u32,
) -> LionLeftChargeMotionSeedLoc3E694 {
    // ASM Lion.asm loc_3E694: zero `y_vel(a3)`, restore `$50(a5)` into
    // `x_vel(a3)`, and `addq.w #1,y_pos(a3)`. Since `y_pos` is FP16.16, that
    // word add advances the integer pixel half by one without changing the
    // fractional word.
    LionLeftChargeMotionSeedLoc3E694 {
        x_velocity: saved_x_velocity,
        y_velocity: 0,
        y_position: y_position.wrapping_add(0x0001_0000),
    }
}

#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn lion_left_charge_airborne_resume_loc_3e6a4() -> LionLoc3E556Tail {
    // ASM Lion.asm loc_3E6A4: after the airborne Y-velocity update, hibernate
    // for one frame, run sub_3E8BA's live object cull, then `jmp (a0)` back
    // into the active Lion routine. It deliberately preserves velocities.
    lion_hibernate_cull_resume_loc_3e556()
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) struct LionLoc3E556Tail {
    pub(crate) hibernate_one_frame: bool,
    pub(crate) cull_routine: &'static str,
    pub(crate) resume_via_callback: bool,
}

#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn lion_hibernate_cull_resume_loc_3e556() -> LionLoc3E556Tail {
    // ASM Lion.asm loc_3E556: hibernate for one frame, run sub_3E8BA's live
    // object cull, then `jmp (a0)` back into the current Lion routine pointer.
    LionLoc3E556Tail {
        hibernate_one_frame: true,
        cull_routine: "sub_3E8BA",
        resume_via_callback: true,
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) enum LionLeftChargeTerrainBranch {
    ContinueToLoc3E5E8,
    EnterLoc3E642,
    ResetAndEnterLoc3E556,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) struct LionLeftChargeTerrainResult {
    pub(crate) branch: LionLeftChargeTerrainBranch,
    pub(crate) next_shared_state: Option<i16>,
    pub(crate) restores_charge_speed: bool,
    pub(crate) restored_animation: Option<&'static str>,
}

#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn lion_left_charge_terrain_gate_loc_3e5ba(
    bit0_set: bool,
    object_meta_low_byte_is_0x10: bool,
) -> LionLeftChargeTerrainResult {
    // ASM Lion.asm loc_3E5BA: clear terrain bit 0 falls through to loc_3E5E8;
    // otherwise object id $10 restores the saved charge motion/animation and
    // enters loc_3E556, while other ids route to loc_3E642.
    if !bit0_set {
        return LionLeftChargeTerrainResult {
            branch: LionLeftChargeTerrainBranch::ContinueToLoc3E5E8,
            next_shared_state: None,
            restores_charge_speed: false,
            restored_animation: None,
        };
    }

    if object_meta_low_byte_is_0x10 {
        LionLeftChargeTerrainResult {
            branch: LionLeftChargeTerrainBranch::ResetAndEnterLoc3E556,
            next_shared_state: Some(-0x78),
            restores_charge_speed: true,
            restored_animation: Some("saved_lion_charge_animation"),
        }
    } else {
        LionLeftChargeTerrainResult {
            branch: LionLeftChargeTerrainBranch::EnterLoc3E642,
            next_shared_state: None,
            restores_charge_speed: false,
            restored_animation: None,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) enum MiniHoppingSkullAirSprite {
    Rising,
    Falling,
}

#[cfg_attr(not(test), allow(dead_code))]
const MINI_SKULL_COLID_RIGHTWALL: u16 = 0x0004;
const MINI_SKULL_COLID_LEFTWALL: u16 = 0x0008;
const MINI_SKULL_COLID_FLOOR: u16 = 0x000C;
const MINI_SKULL_COLID_CEILING: u16 = 0x0010;
const MINI_SKULL_COLID_SLOPEUP: u16 = 0x0014;
const MINI_SKULL_COLID_HURT: u16 = 0x001C;
const MINI_SKULL_COLID_GROUND_KIDABOVE: u16 = 0x001C;
const MINI_SKULL_COLID_GROUND_KIDLEFT: u16 = 0x0020;
const MINI_SKULL_COLID_GROUND_KIDRIGHT: u16 = 0x0024;
const MINI_SKULL_COLID_GROUND_KIDBELOW: u16 = 0x0028;
const MINI_SKULL_COLID_GROUND_HURT: u16 = 0x002C;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) enum MiniHoppingSkullCollisionDispatch {
    AirborneLoop,
    WallBounce,
    FloorLanding,
    CeilingHit,
    HurtOrWeapon,
    SlopeOrOtherTerrain,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) enum MiniHoppingSkullTerrainContactRoute {
    NotSlopeUp,
    SlopeUpNonPositiveXVelocity,
    SlopeUpPositiveXVelocity,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) struct MiniHoppingSkullTerrainContact {
    pub(crate) is_moved: bool,
    pub(crate) is_animated: bool,
    pub(crate) has_level_collision: bool,
    pub(crate) route: MiniHoppingSkullTerrainContactRoute,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) enum MiniHoppingSkullSlopeDownWait {
    Left,
    Right,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) struct MiniHoppingSkullSlopeDownContact {
    pub(crate) x_velocity: i32,
    pub(crate) y_velocity: i32,
    pub(crate) x_direction: u8,
    pub(crate) sprite: &'static str,
    pub(crate) dbf_counter: u16,
    pub(crate) wait_frames: u8,
    pub(crate) wait: MiniHoppingSkullSlopeDownWait,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) enum MiniHoppingSkullGroundWaitSprite {
    SlopeUpRight,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) struct MiniHoppingSkullGroundWaitSetup {
    pub(crate) x_velocity: i32,
    pub(crate) y_velocity: i32,
    pub(crate) x_direction: u8,
    pub(crate) sprite: MiniHoppingSkullGroundWaitSprite,
    pub(crate) dbf_counter: u16,
    pub(crate) wait_frames: u8,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) struct MiniHoppingSkullStoppedWaitSetup {
    pub(crate) is_moved: bool,
    pub(crate) is_animated: bool,
    pub(crate) has_level_collision: bool,
    pub(crate) x_velocity: i32,
    pub(crate) y_velocity: i32,
    pub(crate) sprite: &'static str,
    pub(crate) dbf_counter: u16,
    pub(crate) wait_frames: u8,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) struct MiniHoppingSkullFacingMatchedPauseSetup {
    pub(crate) sprite: &'static str,
    pub(crate) dbf_counter: u16,
    pub(crate) wait_frames: u8,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) enum MiniHoppingSkullStoppedWaitTail {
    ContinueWait {
        next_dbf_counter: u16,
    },
    RestartStoppedWaitFacingLeft {
        x_direction: u8,
        dbf_counter: u16,
        wait_frames: u8,
    },
    RestartStoppedWaitFacingRight {
        x_direction: u8,
        dbf_counter: u16,
        wait_frames: u8,
    },
    BeginFacingMatchedPause {
        sprite: &'static str,
        dbf_counter: u16,
        wait_frames: u8,
    },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) enum MiniHoppingSkullGroundWaitCollisionRoute {
    ContinueWait,
    DamageSkull,
    QueueScreamWorker,
    BerzerkerChargeKillsSkull,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) enum MiniHoppingSkullDamageSequence {
    SkullDestroyed,
    PlayHitAnimation {
        remaining_hits: u16,
        animation: &'static str,
        hibernate_until_finished: bool,
    },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) enum MiniHoppingSkullGroundWaitTail {
    ContinueWait {
        next_dbf_counter: u16,
    },
    SwitchToLeftGroundWaitSetup,
    SwitchToRightGroundWaitSetup,
    BeginRightHopWindup {
        sprite: &'static str,
        dbf_counter: u16,
        wait_frames: u8,
    },
    BeginLeftHopWindup {
        sprite: &'static str,
        dbf_counter: u16,
        wait_frames: u8,
    },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) struct MiniHoppingSkullScreamWorkerAllocation {
    pub(crate) allocation_selector: u16,
    pub(crate) routine: &'static str,
    pub(crate) delay_frames: u8,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) struct MiniHoppingSkullRelaunchTail {
    pub(crate) is_moved: bool,
    pub(crate) has_level_collision: bool,
    pub(crate) next_loop: &'static str,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) enum MiniHoppingSkullLeftLandingLoopDecision {
    ContinueWait {
        next_dbf_counter: u16,
    },
    BeginSkullFacingLeftLanding {
        dbf_counter: u16,
        x_direction: u8,
        sprite: &'static str,
    },
    BeginSkullLeftRelaunch {
        dbf_counter: u16,
        sprite: &'static str,
    },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) enum MiniHoppingSkullLandingLoopDecision {
    ContinueWait {
        next_dbf_counter: u16,
    },
    BeginRelaunchArc {
        y_velocity: i32,
        x_velocity: i32,
        is_moved: bool,
        has_level_collision: bool,
        next_loop: &'static str,
    },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) enum MiniHoppingSkullLandingTailCollisionOutcome {
    ContinueWait {
        next_dbf_counter: u16,
    },
    DamageSkullThenRelaunch {
        sequence: MiniHoppingSkullDamageSequence,
    },
    DamageSkullAndDestroy,
    SpawnScreamWorkerAndRelaunch,
    BerzerkerChargeKillsSkull,
    BeginRelaunchArc {
        y_velocity: i32,
        x_velocity: i32,
        is_moved: bool,
        has_level_collision: bool,
        next_loop: &'static str,
    },
}

// Big Hopping Skull: continuous bouncer from Big_Hopping_Skull.asm.
// FP16.16 velocities: x_vel = ±$10000 (1px/frame), y_vel init = $FFFE0000 (-2px/frame).
// Gravity: $FA0/frame. Deflects off all surfaces with no ground delay.
pub(crate) const BIG_SKULL_INIT_X_SPEED: i32 = 0x0001_0000; // ASM: move.w #1,x_vel (high word)
pub(crate) const BIG_SKULL_INIT_Y_SPEED: i32 = -0x0002_0000; // ASM: move.w #$FFFE,y_vel (high word)
pub(crate) const BIG_SKULL_GRAVITY: i32 = 0x0000_0FA0; // ASM: addi.l #$FA0,y_vel

pub(crate) struct HopperConstants {
    pub(crate) jump_x_speed: i32,
    pub(crate) jump_y_speed: i32,
    pub(crate) gravity: i32,
    pub(crate) ground_delay: u8,
}

pub(crate) fn hopper_constants(_family: EnemyFamily) -> HopperConstants {
    HopperConstants {
        jump_x_speed: MINI_SKULL_JUMP_X_SPEED_FP,
        jump_y_speed: MINI_SKULL_JUMP_Y_SPEED_FP,
        gravity: MINI_SKULL_GRAVITY_FP,
        ground_delay: MINI_SKULL_GROUND_DELAY_FRAMES,
    }
}
// ── Boss multi-head constants (ASM kid.asm:40177–40700) ──
// Each boss spawns 3 head objects at offsets from the layout spawn position.
// Offsets are pixel-level (word), applied to the layout x_pos/y_pos.
/// Sentinel: not a boss head.
pub(crate) const BOSS_HEAD_NONE: u8 = 0xFF;

/// Shishkaboss head Y offsets (X is same for all 3).
pub(crate) const SHISHKABOSS_HEAD_Y_OFFSETS: [i16; 3] = [0, 0x48, 0x90]; // 0, +72, +144
pub(crate) const SHISHKABOSS_HEAD_X_OFFSETS: [i16; 3] = [0, 0, 0];
/// Shishkaboss per-head HP: head1=$14(20), head2=$1E(30), head3=$1E(30).
pub(crate) const SHISHKABOSS_HEAD_HP: [u8; 3] = [20, 30, 30];

/// BoomerangBosses head X offsets (Y is same for all 3).
pub(crate) const BOOMERANG_HEAD_X_OFFSETS: [i16; 3] = [0, -0xC8, -0x1C2]; // 0, -200, -450
pub(crate) const BOOMERANG_HEAD_Y_OFFSETS: [i16; 3] = [0, 0, 0];
/// BoomerangBosses per-head HP: $1E(30) each.
pub(crate) const BOOMERANG_HEAD_HP: [u8; 3] = [30, 30, 30];

/// BagelBrothers head offsets.
pub(crate) const BAGEL_HEAD_X_OFFSETS: [i16; 3] = [0, -0x64, 0]; // 0, -100, 0
pub(crate) const BAGEL_HEAD_Y_OFFSETS: [i16; 3] = [0, -0x96, -0x12C]; // 0, -150, -300
/// BagelBrothers per-head HP: $1E(30) each.
pub(crate) const BAGEL_HEAD_HP: [u8; 3] = [30, 30, 30];

/// Returns true if this enemy family uses multi-head boss spawning.
pub(crate) fn is_multi_head_boss(family: EnemyFamily) -> bool {
    matches!(
        family,
        EnemyFamily::Shishkaboss | EnemyFamily::BoomerangBosses | EnemyFamily::BagelBrothers
    )
}

/// Get per-head HP and position offsets for a multi-head boss.
pub(crate) fn boss_head_config(family: EnemyFamily) -> ([u8; 3], [i16; 3], [i16; 3]) {
    match family {
        EnemyFamily::Shishkaboss => (
            SHISHKABOSS_HEAD_HP,
            SHISHKABOSS_HEAD_X_OFFSETS,
            SHISHKABOSS_HEAD_Y_OFFSETS,
        ),
        EnemyFamily::BoomerangBosses => (
            BOOMERANG_HEAD_HP,
            BOOMERANG_HEAD_X_OFFSETS,
            BOOMERANG_HEAD_Y_OFFSETS,
        ),
        EnemyFamily::BagelBrothers => (BAGEL_HEAD_HP, BAGEL_HEAD_X_OFFSETS, BAGEL_HEAD_Y_OFFSETS),
        _ => ([1, 1, 1], [0, 0, 0], [0, 0, 0]),
    }
}

// Enemy half-width for terrain probes (8px, matching the 16px sprite).
pub(crate) const ENEMY_PROBE_HALF_WIDTH: i32 = 8;
// Enemy height for terrain probes (16px).
pub(crate) const ENEMY_PROBE_HEIGHT: i32 = 16;

/// Compute pacer walk speed from the enemy family and HP.
///
/// The ASM init routines all follow the same pattern: start at $8000 and
/// add $10000 for each HP tier above the family's minimum.
pub(crate) fn pacer_walk_speed(family: EnemyFamily, hit_points: u8) -> i32 {
    if matches!(family, EnemyFamily::Ninja) {
        return ninja_walk_speed_override_loc_3d59a_and_3d5aa(hit_points);
    }

    let (base_hp, base_speed, speed_step) = match family {
        // Goat: HP = enemy_level + 3 (min 3). Base speed $8000, step $10000.
        // Player-tracking callback (Goat.asm loc_3CC18) wired in step dispatch.
        EnemyFamily::Goat => (3u8, 0x0000_8000i32, 0x0001_0000i32),
        // Armadillo.asm loc_3C756/loc_3C766: `enemy_level + 1` yields HP 1/2/3+,
        // then the walk-speed slot starts at `$8000` and gains `$10000` at each tier.
        EnemyFamily::Armadillo => (1, 0x0000_8000, 0x0001_0000),
        // Crab.asm loc_3B00A: HP 2 keeps `$8000`; HP 3+ uses the single
        // `$10000` preload plus one final `$8000` add, so speed caps at `$18000`.
        EnemyFamily::Crab => return crab_motion_profile_loc_3b00a(hit_points).walk_speed,
        // Scorpion: HP = enemy_level + 3 (min 3). Fixed speed $18000.
        EnemyFamily::Scorpion => (3, 0x0001_8000, 0),
        // Driller: HP = enemy_level + 2 (min 2). Base speed $8000, step $8000.
        EnemyFamily::Driller => (2, 0x0000_8000, 0x0000_8000),
        // Tank: HP = enemy_level + 2 (min 2). Base speed $8000, step $8000.
        // ASM Tank.asm loc_3B36A/loc_3B372: HP 2 gets one `$8000` add,
        // HP 3 gets two, HP 4+ gets three.
        EnemyFamily::Tank | EnemyFamily::TankShooting => (2, 0x0000_8000, 0x0000_8000),
        // Fire_Demon.asm loc_3A40C: HP 4 executes the second `$8000` speed add;
        // HP 5+ reaches it after the earlier high-HP preload.
        // Fire_Demon.asm loc_3A414: every tier then falls through to the final
        // `$8000` speed add; HP 3 reaches only this add.
        EnemyFamily::FireDemon => (3, 0x0000_8000, 0x0000_8000),
        // Robot: HP = enemy_level + 2 (min 2). Base speed $8000, step $8000.
        EnemyFamily::Robot => (2, 0x0000_8000, 0x0000_8000),
        // Orca: fixed 4 HP. Fixed speed $8000.
        EnemyFamily::Orca => (4, 0x0000_8000, 0),
        // Ninja: uses a custom init path in Enemy11_Ninja_Init:
        // loc_3D554 sets `current_hp = enemy_level + 2`, then loc_3D59A/loc_3D5AA
        // add to `$50/$5E(a5)` for final speed.
        EnemyFamily::Ninja => (2, 0x0000_8000, 0x0000_8000),
        // Lion: HP = enemy_level + 6 (min 6). Binary speed check: $28000 at HP==8,
        // $18000 otherwise. ASM: Lion.asm lines 32-41, `cmpi.b #8`.
        // Using step=0 and handling HP==8 via lion_charge_speed instead.
        EnemyFamily::Lion => return lion_charge_speed(hit_points),
        // DragonWalking: HP = enemy_level + 1 (min 1). Speed $8000 at HP≤2, $18000 at HP=3.
        // ASM: Dragon.asm lines 64-74 — only HP==3 sets $10000 base, then always +$8000.
        EnemyFamily::DragonWalking => (2, 0x0000_8000, 0x0001_0000),
        // Diamond.asm loc_3AD16/loc_3AD4C: `addq.w #2,d7` yields HP 2/3+, and
        // the <=3 HP path walks at $10000 then $18000.
        EnemyFamily::Diamond => (2, 0x0001_0000, 0x0000_8000),
        // Hand: HP = 1. Speed from table: $3000/$4800/$6000 by HP.
        EnemyFamily::Hand => (1, 0x0000_3000, 0x0000_1800),
        // SpinningTwins: walk speed $18000 after landing phase.
        EnemyFamily::SpinningTwins => (1, 0x0001_8000, 0),
        // TarMonster: walk speed $18000 after emerging.
        EnemyFamily::TarMonster => (2, 0x0001_8000, 0),
        _ => (1, 0x0001_0000, 0),
    };
    let tiers = hit_points.saturating_sub(base_hp) as i32;
    base_speed + tiers * speed_step
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) struct FireDemonInitAnimation {
    pub(crate) collision_routine: &'static str,
    pub(crate) animation_script: &'static str,
    pub(crate) saved_animation_script: &'static str,
}

#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn fire_demon_hp5_animation_loc_3a444(current_hp: u8) -> Option<FireDemonInitAnimation> {
    // ASM Fire_Demon.asm loc_3A444: set `a0` to the Fire Demon collision routine,
    // and keep HP5 on the `stru_3A334` branch while saving the same script in
    // `$62(a5)` for later collision recovery.
    (current_hp == 5).then_some(FireDemonInitAnimation {
        collision_routine: "fire_demon_collision",
        animation_script: "stru_3A334",
        saved_animation_script: "stru_3A334",
    })
}

#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn fire_demon_non_hp5_animation_loc_3a464(
    current_hp: u8,
) -> Option<FireDemonInitAnimation> {
    // ASM Fire_Demon.asm loc_3A464: the non-HP5 branch initialises `stru_3A31A`
    // and stores the same animation pointer in `$62(a5)`.
    (current_hp != 5).then_some(FireDemonInitAnimation {
        collision_routine: "fire_demon_collision",
        animation_script: "stru_3A31A",
        saved_animation_script: "stru_3A31A",
    })
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) enum FireDemonInitWalkDispatch {
    LeftSharedWalker,
    RightSharedWalker,
}

#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn fire_demon_init_walk_dispatch_loc_3a476(
    facing: Direction,
) -> FireDemonInitWalkDispatch {
    // ASM Fire_Demon.asm loc_3A476: `tst.b x_direction(a3)` dispatches clear
    // raw direction into the left shared walker and set raw direction into the
    // right shared walker. Native `Direction` is the decoded raw flag.
    match facing {
        Direction::Left => FireDemonInitWalkDispatch::LeftSharedWalker,
        Direction::Right => FireDemonInitWalkDispatch::RightSharedWalker,
    }
}

/// Tank bullet speed in FP16.16 (3 px/frame → $30000).
pub(crate) const TANK_BULLET_SPEED: i32 = 0x3_0000;
/// Tank bullet lifetime in frames (enough to cross a screen width).
pub(crate) const TANK_BULLET_LIFETIME: u16 = 120;
/// ASM Tank_Shooting.asm `stru_3B4D8`: one shot every 116 frames at HP 4.
pub(crate) const TANK_SHOOTING_FAST_CYCLE_FRAMES: u16 = 116;
/// ASM Tank_Shooting.asm `stru_3B4F6`: one shot every 232 frames at HP 2-3.
pub(crate) const TANK_SHOOTING_SLOW_CYCLE_FRAMES: u16 = 232;
/// ASM GfxObject_Animate + `stru_3B4D8`/`stru_3B4F6`: `loc_3C50E` fires one
/// frame after the turret enters `LnkTo_unk_C7F10`, which is 74 frames after init.
pub(crate) const TANK_SHOOTING_FIRST_SHOT_DELAY_FRAMES: u16 = 74;
/// ASM Tank_Shooting.asm loc_3B75A/loc_3B77E: spawn at ±15px from tank centre.
pub(crate) const TANK_BULLET_SPAWN_X_OFFSET_PX: i32 = 15;
/// ASM Tank_Shooting.asm loc_3B786/loc_3B78E: projectile Y = turret_y - (8 + 6) px.
pub(crate) const TANK_BULLET_SPAWN_Y_OFFSET_PX: i32 = 14;

/// Scorpion tracking range in pixels (Scorpion.asm line 158: $AA = 170).
/// Outside this range, speed resets to base $18000.
const SCORPION_TRACK_RANGE: i64 = 0xAA;
/// Scorpion boosted speed when within tracking range and enemy_level > 1 (HP > 3).
/// From Scorpion.asm line 164: $38000.
const SCORPION_BOOST_SPEED: i32 = 0x3_8000;
/// Scorpion base speed (always $18000, from Scorpion.asm init line 30).
const SCORPION_BASE_SPEED: i32 = 0x1_8000;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct ScorpionTailGfxPosition {
    pub(crate) x_word: u16,
    pub(crate) y_word: u16,
}

#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn scorpion_tail_sync_loc_3dc86(
    parent_x_pos: u32,
    parent_y_pos: u32,
    parent_facing: Direction,
) -> ScorpionTailGfxPosition {
    // ASM Scorpion.asm loc_3DC86 copies the parent integer X/Y words to the
    // linked tail gfx object, lifts it by $16 px, then applies a facing offset.
    let parent_x_word = (parent_x_pos >> 16) as u16;
    let parent_y_word = (parent_y_pos >> 16) as u16;
    let x_offset = match parent_facing {
        Direction::Left => 1_u16,
        // ASM Scorpion.asm loc_3DCAA: when `x_direction` is set, subtract $16
        // from the linked tail object's integer X word.
        Direction::Right => 0x16_u16,
    };

    ScorpionTailGfxPosition {
        x_word: parent_x_word.wrapping_sub(x_offset),
        y_word: parent_y_word.wrapping_sub(0x16),
    }
}

/// Archer fire interval indexed by enemy_level (HP 1/2/3).
/// From Archer.asm `arrow_accuracy`: $1E=30, $14=20, $0A=10 frames.
pub(crate) const ARCHER_FIRE_TIMER: [u16; 3] = [0x1E, 0x14, 0x0A];

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
pub(crate) struct ActivationWindow {
    pub(crate) x_min: u32,
    pub(crate) x_max: u32,
    pub(crate) y_min: u32,
    pub(crate) y_max: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum EnemyFamily {
    Diamond,
    Dragon,        // Flying dragon ($0C): choreographed flight pattern
    DragonWalking, // Walking dragon ($0D): stationary with breath attack
    Cloud,
    Sphere,
    BigHoppingSkull,
    EmoRock,
    Archer,
    Fireball,
    FireDemon,
    Robot,
    TarMonster,
    Goat,
    Armadillo,
    Tank,
    TankShooting,
    Crab,
    Orca,
    #[allow(clippy::upper_case_acronyms)]
    UFO,
    Ninja,
    Lion,
    Scorpion,
    SpinningTwins,
    Drips,
    Hand,
    Tornado,
    Driller,
    MiniHoppingSkull,
    HeadyMetal,
    Shishkaboss,
    BoomerangBosses,
    BagelBrothers,
    Unknown(u8),
}

/// Dragon Flying state machine (Dragon_Flying.asm).
/// ASM uses a script interpreter with 3 variant flight tables selected by skin byte.
/// Commands: Hover, Drift, DriftWithFire, FacePlayer, Descent, ReturnToAnchor.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub(crate) enum DragonFlyingState {
    /// Not a Dragon Flying enemy (default for all other families).
    #[default]
    None,
    /// Hovering in place with zero x velocity (ASM cmd 0, loc_34804).
    Hovering,
    /// Accelerating drift (ASM cmd 1, loc_34776). $800 start, $200/frame, $18000 cap.
    Drifting,
    /// Same as Drift but checks fire-breath cone each frame (ASM cmd 5, loc_34774).
    DriftingWithFire,
    /// Descent attack sub-script: fixed velocity segments from unk_34554.
    Descending,
    /// Rising back to anchor Y with deceleration (ASM cmd 6, loc_34B44).
    ReturningToAnchor,
}

impl DragonFlyingState {
    /// Stationary phases where the dragon is hovering — suppress walk animation.
    /// `None` is excluded: non-Dragon enemies use their own stationary logic.
    pub(crate) fn is_stationary(self) -> bool {
        matches!(self, Self::Hovering)
    }
}

/// Ninja stealth/attack state machine (Ninja.asm).
/// Stealth = pacing invisibly. JumpAttack = airborne lunge at player.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub(crate) enum NinjaState {
    /// Walking invisibly, checking detection cone each frame.
    #[default]
    Stealth,
    /// Airborne jump-attack toward the player.
    JumpAttack,
}

/// Orca lunge state machine (Orca.asm).
/// Paces normally, detects nearby player, launches a ballistic lunge arc.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub(crate) enum OrcaState {
    /// Walking patrol with proximity detection.
    #[default]
    Walking,
    /// Airborne ballistic lunge toward the player.
    Lunging,
}

/// Robot phase state machine (Robot.asm).
/// Paces at HP-dependent speed, periodically shoots, then charges fast.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub(crate) enum RobotState {
    /// Walking patrol with periodic shooting.
    #[default]
    Walking,
    /// Charging at high speed after jump counter expires.
    Charging,
}

/// Lion charge state machine (Lion.asm).
/// Paces normally, then charges toward player and fires a fast projectile.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub(crate) enum LionState {
    /// Walking patrol, checking for player in range.
    #[default]
    Walking,
    /// Charging toward the player at high speed.
    Charging,
}

/// Hand patrol/attack state machine (Hand.asm).
/// Walks until player enters forward detection range, then jump-attacks.
/// On landing near player, grabs and follows player for 10 frames.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub(crate) enum HandState {
    /// Walking on ground, checking for player in detection cone.
    #[default]
    Walking,
    /// Airborne jump toward the player.
    Jumping,
    /// Grabbing: hand follows player position for 10 frames (Hand.asm loc_3292E).
    Grabbing,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub(crate) enum TarMonsterState {
    /// Hidden underground, waiting to emerge.
    #[default]
    Submerged,
    /// Rising from the floor (animation frames, 24f).
    Emerging,
    /// Visible and vulnerable above ground.
    Visible,
    /// Attack wind-up + glob shot + wind-down (8+8+8 = 24f). ASM Op 5.
    Attacking,
    /// Sinking back into the floor (24f).
    Submerging,
}

/// SpinningTwins: orbits first, then drops into walking.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub(crate) enum SpinningTwinsPhase {
    /// Orbiting at spawn position with visual oscillation.
    #[default]
    Orbiting,
    /// Dropped to ground, walking as a pacer.
    Walking,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub(crate) struct SharedWalkerTerrainTrace {
    pub(crate) next_x_px: Option<i32>,
    pub(crate) wall: Option<terrain::WallProbe>,
    pub(crate) center_floor: Option<terrain::FloorProbe>,
    pub(crate) left_floor: Option<terrain::FloorProbe>,
    pub(crate) right_floor: Option<terrain::FloorProbe>,
}

/// Persistent per-enemy state that survives across frames.
/// Enemies are activated when they enter the camera window and
/// deactivated when they leave.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct ActiveEnemy {
    /// Index into the enemy layout segment list, or 0 for legacy flat layouts.
    pub(crate) segment_index: usize,
    /// Index into the level's enemy layout objects list.
    pub(crate) layout_index: usize,
    /// Raw layout `enemy_level(a3)` copied from `hit_points_plus_one`.
    pub(crate) enemy_level: u8,
    pub(crate) family: EnemyFamily,
    pub(crate) x_pos: u32,
    pub(crate) y_pos: u32,
    pub(crate) x_velocity: i32,
    pub(crate) y_velocity: i32,
    /// Per-axis desired velocity for acceleration-driven flyers.
    pub(crate) target_x_velocity: i32,
    pub(crate) target_y_velocity: i32,
    /// Per-frame acceleration (UFO, Tornado)
    pub(crate) x_acceleration: i32,
    pub(crate) y_acceleration: i32,
    pub(crate) facing: Direction,
    pub(crate) on_ground: bool,
    pub(crate) hit_points: u8,
    pub(crate) grounded_frames: u8,
    pub(crate) damage_flash_frames: u8,
    /// Palette variant (0, 1, or 2) from the enemy type slot.
    pub(crate) palette_variant: u8,
    /// Generic countdown timer for phased enemies (EmoRock hover wait, etc.)
    pub(crate) wait_timer: u16,
    pub(crate) dragon_phase: DragonFlyingState,
    pub(crate) dragon_phase_frames: u16,
    /// Dragon Flying: script variant (0, 1, 2) from layout skin byte.
    pub(crate) dragon_script_variant: u8,
    /// Dragon Flying: current command index within the script.
    /// Tornado reuses this as the next `loc_359F2` pattern index from `unk_35A0C`.
    pub(crate) dragon_script_cmd: u8,
    /// Dragon Flying: anchor Y position (FP16.16) — initial spawn Y to return to.
    pub(crate) dragon_anchor_y: u32,
    /// Dragon Flying: descent sub-script entry index.
    pub(crate) dragon_descent_index: u8,
    /// Dragon Flying: frames remaining in current descent entry.
    pub(crate) dragon_descent_frames: u8,
    pub(crate) ninja_state: NinjaState,
    pub(crate) hand_state: HandState,
    pub(crate) lion_state: LionState,
    pub(crate) robot_state: RobotState,
    pub(crate) orca_state: OrcaState,
    /// Orca: wall-pinned flag ($5C(a5)), suppresses X accel during lunge.
    pub(crate) wall_pinned: bool,
    pub(crate) tar_monster_state: TarMonsterState,
    pub(crate) spinning_twins_phase: SpinningTwinsPhase,
    pub(crate) spinning_twins_orbit_index: u8,
    /// UFO beam: segments remaining to spawn (0=not firing). ASM $48(a5).
    pub(crate) ufo_beam_segments: u8,
    /// UFO beam: frames until next segment. ASM $4A(a5).
    pub(crate) ufo_beam_interval: u8,
    /// Boss head index (0, 1, 2) for multi-head bosses. BOSS_HEAD_NONE if not a boss head.
    pub(crate) boss_head_index: u8,
    /// Scorpion: tracking-lock flag ($4E(a5)). Suppresses re-facing when locked onto player.
    pub(crate) scorpion_tracking_lock: bool,
    /// ASM UFO.asm `$4E(a5)`: doubles per-axis acceleration for UFO/Tornado tiers 1-2.
    pub(crate) flight_double_acceleration: bool,
    /// ASM UFO.asm `$4F(a5)`: doubles target speed for UFO/Tornado tier 2.
    pub(crate) flight_double_terminal_speed: bool,
    /// Walk speed override from tracking callback ($50(a5)). When non-zero, step_pacer
    /// uses this instead of pacer_walk_speed(). Set by Scorpion tracking (Scorpion.asm loc_3DD7E).
    pub(crate) walk_speed_override: i32,
    /// Crab: frozen state flag ($6E(a5)). ASM loc_3B104 sets this when player
    /// within 64px and facing the crab; loc_3B170 clears it when distance >224px.
    pub(crate) crab_frozen: bool,
    /// Shared Dragon/Lion grounded-state word ($42(a3)).
    pub(crate) shared_walker_state: i16,
    /// Shared Dragon/Lion patrol cadence countdown ($54(a5)).
    pub(crate) shared_walker_patrol_timer: u16,
    /// Dragon alternates the next patrol cadence reload between $32 and $12C via $6E(a5).
    pub(crate) shared_walker_patrol_short_next: bool,
    /// Frontier-only trace label for the last shared-walker branch taken.
    pub(crate) shared_walker_debug_branch: Option<&'static str>,
    /// Signed horizontal distance to Kid when the grounded shared walker last evaluated him.
    pub(crate) shared_walker_dx_to_kid_px: Option<i16>,
    /// Frontier-only trace of the last terrain probes used by the shared walker.
    pub(crate) shared_walker_terrain_trace: SharedWalkerTerrainTrace,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum EnemyArchetype {
    Bouncer,
    Collectible,
    Flyer,
    Hopper,
    Hoverer,
    Pacer,
    Stationary,
    Tracker,
    Inert,
}

pub(crate) const DRAGON_SPAWN_X_OFFSET_FP: u32 = 0x001A_8000;
/// Dragon families spawn 1px below layout Y. ASM-traced offset.
const DRAGON_SPAWN_Y_OFFSET_PX: u32 = 1;
/// Walking dragons also use the 1px Y offset.
const DRAGON_WALKING_SPAWN_Y_OFFSET_PX: u32 = 1;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) struct DragonWalkingChildInitLoc3CD16 {
    pub(crate) x_pos: u32,
    pub(crate) y_pos: u32,
    pub(crate) facing: Direction,
    pub(crate) inherited_state_word: u16,
    pub(crate) parent_spawned_child: bool,
}

#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn dragon_walking_child_init_loc_3cd16(
    parent_x_pos: u32,
    parent_y_pos: u32,
    parent_facing: Direction,
    parent_state_word_4a: u16,
) -> DragonWalkingChildInitLoc3CD16 {
    // ASM Dragon.asm loc_3CD16: the flying-dragon parent allocates the
    // walking-dragon child, copies `$44/$46/$48/$4A(a5)`, and sets `$5C(a5)`.
    DragonWalkingChildInitLoc3CD16 {
        x_pos: parent_x_pos,
        y_pos: parent_y_pos,
        facing: parent_facing,
        inherited_state_word: parent_state_word_4a,
        parent_spawned_child: true,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) struct DragonArtInitLoc3CDB8 {
    pub(crate) art_slot_index: u8,
    pub(crate) status_flag_13: bool,
    pub(crate) priority: u8,
    pub(crate) current_hp: u16,
}

#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn dragon_art_init_loc_3cdb8(
    first_header_word: u16,
    second_header_word: u16,
    enemy_level: u8,
) -> DragonArtInitLoc3CDB8 {
    fn is_dragon_header_word(raw: u16) -> bool {
        matches!(raw & 0x0FFF, 0x000C | 0x000D)
    }

    let art_slot_index = if is_dragon_header_word(first_header_word) {
        0
    } else if is_dragon_header_word(second_header_word) {
        1
    } else {
        2
    };

    // ASM Dragon.asm loc_3CDB8: d4 selects the palette/VRAM table entry,
    // `$13(a3)` is set, priority is cleared, and current_hp is
    // `enemy_level + 1`.
    DragonArtInitLoc3CDB8 {
        art_slot_index,
        status_flag_13: true,
        priority: 0,
        current_hp: u16::from(enemy_level) + 1,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) struct DragonWalkingSetupLoc3CDFC {
    pub(crate) speed_slot_50: i32,
    pub(crate) target_speed_slot_5e: i32,
    pub(crate) state_word_4a: u16,
    pub(crate) timer_48: u16,
    pub(crate) patrol_timer_pair_6e: u32,
    pub(crate) short_patrol_timer_54: u16,
    pub(crate) callback_role: &'static str,
    pub(crate) animation_label: &'static str,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) struct DragonWalkingNormalAnimationLoc3CE48 {
    pub(crate) animation_label: &'static str,
    pub(crate) saved_animation_label: &'static str,
}

#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn dragon_walking_normal_animation_loc_3ce48() -> DragonWalkingNormalAnimationLoc3CE48 {
    // ASM Dragon.asm loc_3CE48: initialise `stru_3CC5E`, save the same
    // animation pointer in `$62(a5)`, then fall through to loc_3CE5A.
    DragonWalkingNormalAnimationLoc3CE48 {
        animation_label: "stru_3CC5E",
        saved_animation_label: "stru_3CC5E",
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) enum DragonWalkingInitialDispatchLoc3CE5A {
    NormalFacingDispatch,
    SpawnedChildLeftPacing,
    SpawnedChildRightPacing,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) struct DragonWalkingCommonInitLoc3CE5A {
    pub(crate) shared_walker_state: i16,
    pub(crate) breath_timer: u16,
    pub(crate) patrol_timer: u16,
    pub(crate) collision_callback: &'static str,
    pub(crate) next: DragonWalkingInitialDispatchLoc3CE5A,
}

#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn dragon_walking_common_init_loc_3ce5a(
    parent_spawned_child: bool,
    facing: Direction,
) -> DragonWalkingCommonInitLoc3CE5A {
    // ASM Dragon.asm loc_3CE5A: seed `$42(a3)`, `$66(a5)`, `$54(a5)`, and
    // `a0=loc_3CE90`; when `$5C(a5)` is set, branch by x_direction directly
    // to the spawned-child pacing loops, otherwise continue to normal-facing
    // initial dispatch.
    let next = if parent_spawned_child {
        match facing {
            Direction::Left => DragonWalkingInitialDispatchLoc3CE5A::SpawnedChildLeftPacing,
            Direction::Right => DragonWalkingInitialDispatchLoc3CE5A::SpawnedChildRightPacing,
        }
    } else {
        DragonWalkingInitialDispatchLoc3CE5A::NormalFacingDispatch
    };

    DragonWalkingCommonInitLoc3CE5A {
        shared_walker_state: SHARED_WALKER_STATE_PATROL,
        breath_timer: DRAGON_WALKING_BREATH_INITIAL,
        patrol_timer: DRAGON_WALKING_PATROL_LONG_FRAMES,
        collision_callback: "loc_3CE90",
        next,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) enum DragonWalkingNormalDispatchLoc3CE84 {
    Loc3E546,
    Loc3E392,
}

#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn dragon_walking_normal_dispatch_loc_3ce84(
    facing: Direction,
) -> DragonWalkingNormalDispatchLoc3CE84 {
    // ASM Dragon.asm loc_3CE84: `tst.b x_direction(a3); beq.w loc_3E546;
    // bra.w loc_3E392`. Clear x_direction is the left-facing pacing loop;
    // set x_direction is the right-facing pacing loop.
    match facing {
        Direction::Left => DragonWalkingNormalDispatchLoc3CE84::Loc3E546,
        Direction::Right => DragonWalkingNormalDispatchLoc3CE84::Loc3E392,
    }
}

#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn dragon_walking_setup_loc_3cdfc(current_hp: u16) -> DragonWalkingSetupLoc3CDFC {
    let hp3_base_speed = if current_hp == 3 { 0x0001_0000 } else { 0 };
    let speed = hp3_base_speed + 0x0000_8000;
    let normal_animation = dragon_walking_normal_animation_loc_3ce48();

    // ASM Dragon.asm loc_3CDFC: HP 3 preloads `$10000`, all HP tiers add
    // `$8000`, then shared setup seeds `$4A/$48/$6E/$54/$6A` before choosing
    // the HP-3 animation.
    DragonWalkingSetupLoc3CDFC {
        speed_slot_50: speed,
        target_speed_slot_5e: speed,
        state_word_4a: 0x0010,
        timer_48: 9,
        patrol_timer_pair_6e: 0x0032_012C,
        short_patrol_timer_54: DRAGON_WALKING_PATROL_SHORT_FRAMES,
        callback_role: "dragon_breath_callback",
        animation_label: if current_hp == 3 {
            "stru_3CC90"
        } else {
            normal_animation.animation_label
        },
    }
}

// --- Dragon Flying script interpreter (Dragon_Flying.asm) ---

/// Drift initial speed: $800 (subpixel FP16.16). ASM loc_34776.
const DRAGON_DRIFT_INITIAL_SPEED: i32 = 0x800;
/// Drift acceleration per frame: $200. ASM loc_3478C.
const DRAGON_DRIFT_ACCEL: i32 = 0x200;
/// Drift speed cap: $18000. ASM loc_3478C.
const DRAGON_DRIFT_SPEED_CAP: i32 = 0x1_8000;
/// Return-to-anchor deceleration per frame: $400. ASM loc_34B4C.
const DRAGON_RETURN_DECEL: i32 = 0x400;
/// Return-to-anchor intermediate Y velocity cap: $FFFF1000. ASM loc_34B4C.
const DRAGON_RETURN_VEL_CAP_SOFT: i32 = -0xF000_i32; // $FFFF1000 as signed
/// Return-to-anchor hard Y velocity cap: $FFFF8000. ASM loc_34B70.
const DRAGON_RETURN_VEL_CAP_HARD: i32 = -0x8000_i32; // $FFFF8000 as signed
/// Face-player initial x velocity: $8000. ASM loc_3473A.
const DRAGON_FACE_PLAYER_SPEED: i32 = 0x8000;

#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn dragon_drift_negative_velocity_loc_347b2(x_velocity: i32) -> i32 {
    // ASM Dragon_Flying.asm loc_347B2: negative drift subtracts `$200`
    // from d7 and clamps at `$FFFE8000` (-$18000).
    (x_velocity - DRAGON_DRIFT_ACCEL).max(-DRAGON_DRIFT_SPEED_CAP)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) struct DragonDriftStoreLoc347C8 {
    pub(crate) x_velocity: i32,
    pub(crate) checks_fire_breath_cone: bool,
}

#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn dragon_drift_store_loc_347c8(
    d7_velocity: i32,
    d1_fire_gate: u16,
) -> DragonDriftStoreLoc347C8 {
    // ASM Dragon_Flying.asm loc_347C8: store prepared d7 into x_vel, then
    // call loc_34836 only when d1 is non-zero.
    DragonDriftStoreLoc347C8 {
        x_velocity: d7_velocity,
        checks_fire_breath_cone: d1_fire_gate != 0,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) struct DragonDriftDbfTailLoc347FC {
    pub(crate) next_d0: u16,
    pub(crate) loops_to_loc_3478c: bool,
    pub(crate) advances_script: bool,
}

#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn dragon_drift_dbf_tail_loc_347fc(d0_before_dbf: u16) -> DragonDriftDbfTailLoc347FC {
    // ASM Dragon_Flying.asm loc_347FC: `dbf d0,loc_3478C` decrements the
    // low word, loops until it becomes `$FFFF`, then falls through to loc_346DA.
    let next_d0 = d0_before_dbf.wrapping_sub(1);
    let loops_to_loc_3478c = next_d0 != 0xFFFF;
    DragonDriftDbfTailLoc347FC {
        next_d0,
        loops_to_loc_3478c,
        advances_script: !loops_to_loc_3478c,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) struct DragonHoverDbfTailLoc3482E {
    pub(crate) next_d0: u16,
    pub(crate) loops_to_loc_3480a: bool,
    pub(crate) advances_script: bool,
}

#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn dragon_hover_dbf_tail_loc_3482e(d0_before_dbf: u16) -> DragonHoverDbfTailLoc3482E {
    // ASM Dragon_Flying.asm loc_3482E: `dbf d0,loc_3480A` uses the same
    // low-word decrement/fall-through rule as the drift tail, but targets the
    // hover hibernation loop.
    let next_d0 = d0_before_dbf.wrapping_sub(1);
    let loops_to_loc_3480a = next_d0 != 0xFFFF;
    DragonHoverDbfTailLoc3482E {
        next_d0,
        loops_to_loc_3480a,
        advances_script: !loops_to_loc_3480a,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) struct DragonDescentMoveSetupLoc348E4 {
    pub(crate) x_velocity: i32,
    pub(crate) y_velocity: i32,
    pub(crate) dbf_counter_after_subq: u16,
}

#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn dragon_descent_move_setup_loc_348e4(
    facing: Direction,
    left_facing_x_vel_px: i8,
    y_vel_px: i8,
    frame_count: u8,
) -> DragonDescentMoveSetupLoc348E4 {
    // ASM Dragon_Flying.asm loc_348E4: after loc_348CE decodes signed
    // byte-sized x/y velocities, the x word is written to `x_vel`; the next
    // script byte supplies d2 and `subq.w #1,d2` seeds the DBF loop counter.
    let left_facing_x_velocity = i32::from(left_facing_x_vel_px) << 16;
    let x_velocity = match facing {
        Direction::Right => -left_facing_x_velocity,
        Direction::Left => left_facing_x_velocity,
    };
    DragonDescentMoveSetupLoc348E4 {
        x_velocity,
        y_velocity: i32::from(y_vel_px) << 16,
        dbf_counter_after_subq: u16::from(frame_count).wrapping_sub(1),
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) enum DragonDescentCollisionOutcomeLoc348FC {
    ContinueLoop,
    Explosion,
    TerrainResponse(u16),
}

#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn dragon_descent_collision_outcome_loc_348fc(
    collision_type: u16,
) -> DragonDescentCollisionOutcomeLoc348FC {
    // ASM Dragon_Flying.asm loc_348FC: after one hibernated descent frame,
    // zero collision falls into the DBF tail, negative/$1C/$2C collisions use
    // the death/explosion path, and other positive collisions use terrain response.
    if collision_type == 0 {
        DragonDescentCollisionOutcomeLoc348FC::ContinueLoop
    } else if (collision_type & 0x8000) != 0 || matches!(collision_type, 0x001C | 0x002C) {
        DragonDescentCollisionOutcomeLoc348FC::Explosion
    } else {
        DragonDescentCollisionOutcomeLoc348FC::TerrainResponse(collision_type)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) struct DragonDescentDbfTailLoc34924 {
    pub(crate) next_d2: u16,
    pub(crate) loops_to_loc_348fc: bool,
    pub(crate) advances_descent_script: bool,
}

#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn dragon_descent_dbf_tail_loc_34924(
    d2_before_dbf: u16,
) -> DragonDescentDbfTailLoc34924 {
    // ASM Dragon_Flying.asm loc_34924: `dbf d2,loc_348FC` decrements the
    // movement-frame counter, loops while it is not `$FFFF`, then falls through
    // to the next descent script word.
    let next_d2 = d2_before_dbf.wrapping_sub(1);
    let loops_to_loc_348fc = next_d2 != 0xFFFF;
    DragonDescentDbfTailLoc34924 {
        next_d2,
        loops_to_loc_348fc,
        advances_descent_script: !loops_to_loc_348fc,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) struct DragonFlyingRespawnMarkerLoc34998 {
    pub(crate) index: usize,
    pub(crate) word_value: u16,
}

#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn dragon_flying_respawn_marker_loc_34998(
    object_status_slot: u8,
) -> DragonFlyingRespawnMarkerLoc34998 {
    // ASM Dragon_Flying.asm loc_34998: the positive `$42(a5)` byte is doubled
    // as a word offset into EnemyStatus_Table, then `$2168` is written as the
    // respawn marker before falling through to loc_349A6.
    DragonFlyingRespawnMarkerLoc34998 {
        index: usize::from(object_status_slot),
        word_value: 0x2168,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) struct DragonFlyingDeleteTailLoc349A6 {
    pub(crate) deletes_current_object: bool,
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn dragon_flying_delete_tail_loc_349A6() -> DragonFlyingDeleteTailLoc349A6 {
    // ASM Dragon_Flying.asm loc_349A6: all death-status routes converge on the
    // `j_Delete_CurrentObject` tail after the explosion animation finishes.
    DragonFlyingDeleteTailLoc349A6 {
        deletes_current_object: true,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) enum DragonFlyingDeathStatusRouteLoc3492C {
    WriteRespawnMarker(DragonFlyingRespawnMarkerLoc34998),
    SubtractSpawnCounter { index: usize, decrement: u16 },
    DeleteOnly,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) struct DragonFlyingDeathSequenceLoc3492C {
    pub(crate) animation_label: &'static str,
    pub(crate) is_moved: bool,
    pub(crate) vram_tile: u16,
    pub(crate) pre_child_hibernate_frames: u16,
    pub(crate) child_x_word: u16,
    pub(crate) child_y_word: u16,
    pub(crate) child_facing: Direction,
    pub(crate) child_state_word_4a: u16,
    pub(crate) hibernate_until_animation_finished: bool,
    pub(crate) status_route: DragonFlyingDeathStatusRouteLoc3492C,
    pub(crate) delete_tail: DragonFlyingDeleteTailLoc349A6,
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn dragon_flying_death_sequence_loc_3492C(
    parent_x_pos: u32,
    parent_y_pos: u32,
    parent_facing: Direction,
    parent_word_46: u16,
    object_status_slot: u8,
) -> DragonFlyingDeathSequenceLoc3492C {
    // ASM Dragon_Flying.asm loc_3492C: play `stru_345E4`, clear `is_moved`
    // and `vram_tile`, wait two frames, then allocate the loc_3CD16 walking
    // dragon child from integer X/Y words before waiting for the animation.
    let status_route = if object_status_slot & 0x80 == 0 {
        DragonFlyingDeathStatusRouteLoc3492C::WriteRespawnMarker(
            dragon_flying_respawn_marker_loc_34998(object_status_slot),
        )
    } else if object_status_slot & 0x40 != 0 {
        DragonFlyingDeathStatusRouteLoc3492C::SubtractSpawnCounter {
            index: usize::from(object_status_slot & 0x3F),
            decrement: 0x0400,
        }
    } else {
        DragonFlyingDeathStatusRouteLoc3492C::DeleteOnly
    };

    DragonFlyingDeathSequenceLoc3492C {
        animation_label: "stru_345E4",
        is_moved: false,
        vram_tile: 0,
        pre_child_hibernate_frames: 2,
        child_x_word: (parent_x_pos >> 16) as u16,
        child_y_word: ((parent_y_pos >> 16) as u16).wrapping_add(0x10),
        child_facing: parent_facing,
        child_state_word_4a: parent_word_46,
        hibernate_until_animation_finished: true,
        status_route,
        delete_tail: dragon_flying_delete_tail_loc_349A6(),
    }
}

/// Script command for Dragon Flying flight scripts.
#[derive(Debug, Clone, Copy)]
pub(crate) enum DragonFlyingCmd {
    /// Hover in place for N frames (ASM cmd 0).
    Hover(u16),
    /// Accelerating drift for N frames (ASM cmd 1).
    Drift(u16),
    /// Drift + fire-breath cone check for N frames (ASM cmd 5).
    DriftWithFire(u16),
    /// Face player and set initial drift speed (ASM cmd 3). Immediate.
    FacePlayer,
    /// Descent attack sub-script (ASM cmd 4).
    Descent,
    /// Return to anchor Y position (ASM cmd 6).
    ReturnToAnchor,
}

/// Variant 0 script (unk_34710): hover → descent → drift → hover → drift → return.
const DRAGON_SCRIPT_0: &[DragonFlyingCmd] = &[
    DragonFlyingCmd::Hover(0x70), // 112 frames
    DragonFlyingCmd::Descent,
    DragonFlyingCmd::Drift(0x50), // 80 frames
    DragonFlyingCmd::Hover(0x32), // 50 frames
    DragonFlyingCmd::Drift(0x78), // 120 frames
    DragonFlyingCmd::ReturnToAnchor,
];

/// Variant 1 script (unk_34706): aggressive fire-heavy patrol.
const DRAGON_SCRIPT_1: &[DragonFlyingCmd] = &[
    DragonFlyingCmd::DriftWithFire(0x28), // 40 frames
    DragonFlyingCmd::DriftWithFire(0x1E), // 30 frames
    DragonFlyingCmd::DriftWithFire(0x32), // 50 frames
    DragonFlyingCmd::ReturnToAnchor,
];

/// Variant 2 script (unk_346FA): long fire patrols with retargeting.
const DRAGON_SCRIPT_2: &[DragonFlyingCmd] = &[
    DragonFlyingCmd::DriftWithFire(0x70), // 112 frames
    DragonFlyingCmd::DriftWithFire(0x70), // 112 frames
    DragonFlyingCmd::FacePlayer,
    DragonFlyingCmd::DriftWithFire(0x50), // 80 frames
    DragonFlyingCmd::ReturnToAnchor,
];

pub(crate) fn dragon_script(variant: u8) -> &'static [DragonFlyingCmd] {
    match variant {
        0 => DRAGON_SCRIPT_0,
        1 => DRAGON_SCRIPT_1,
        _ => DRAGON_SCRIPT_2,
    }
}

/// Descent sub-script entry: fixed velocity for N frames (unk_34554).
#[derive(Debug, Clone, Copy)]
enum DragonDiveAction {
    /// Move at (x_vel, y_vel) pixels/frame for N frames. x_vel is for facing-left;
    /// negated when facing right (ASM loc_348CE).
    Move { x_vel: i8, y_vel: i8, frames: u8 },
    /// Spawn fire-breath projectile (ASM $E19F sentinel).
    SpawnFire,
    /// Flip facing direction (ASM $DD48 sentinel).
    FlipDirection,
    /// Restart main flight script (ASM $D8F1 sentinel).
    RestartScript,
}

/// Descent attack animation script (unk_34554, kid.asm:37771).
/// U-shaped dive: descend → level out → ascend → flip → continue ascending → restart.
const DRAGON_DIVE_SCRIPT: &[DragonDiveAction] = &[
    DragonDiveAction::Move {
        x_vel: -1,
        y_vel: 1,
        frames: 12,
    },
    DragonDiveAction::Move {
        x_vel: -1,
        y_vel: 2,
        frames: 9,
    },
    DragonDiveAction::Move {
        x_vel: -1,
        y_vel: 3,
        frames: 6,
    },
    DragonDiveAction::SpawnFire,
    DragonDiveAction::Move {
        x_vel: -1,
        y_vel: 4,
        frames: 6,
    },
    DragonDiveAction::Move {
        x_vel: -2,
        y_vel: 3,
        frames: 3,
    },
    DragonDiveAction::Move {
        x_vel: -2,
        y_vel: 2,
        frames: 3,
    },
    DragonDiveAction::Move {
        x_vel: -2,
        y_vel: 1,
        frames: 3,
    },
    DragonDiveAction::Move {
        x_vel: -2,
        y_vel: 0,
        frames: 3,
    },
    DragonDiveAction::Move {
        x_vel: -2,
        y_vel: -1,
        frames: 3,
    },
    DragonDiveAction::Move {
        x_vel: -2,
        y_vel: -2,
        frames: 3,
    },
    DragonDiveAction::Move {
        x_vel: -2,
        y_vel: -3,
        frames: 3,
    },
    DragonDiveAction::Move {
        x_vel: -1,
        y_vel: -4,
        frames: 6,
    },
    DragonDiveAction::FlipDirection,
    DragonDiveAction::Move {
        x_vel: -1,
        y_vel: -3,
        frames: 6,
    },
    DragonDiveAction::Move {
        x_vel: -1,
        y_vel: -2,
        frames: 9,
    },
    DragonDiveAction::Move {
        x_vel: -1,
        y_vel: -1,
        frames: 12,
    },
    DragonDiveAction::RestartScript,
];

#[derive(Debug, Clone, Copy)]
pub(crate) struct EnemyLayoutView<'a> {
    pub(crate) segment_index: usize,
    pub(crate) enemy_type_slots: &'a [u16],
    pub(crate) objects: &'a [EnemyLayoutObject],
}

impl ActivationWindow {
    pub(crate) fn from_camera(camera: CameraState) -> Self {
        let camera_x_px = camera.x_pos >> 16;
        let camera_y_px = camera.y_pos >> 16;
        Self {
            x_min: camera_x_px.saturating_sub(ACTIVATION_WINDOW_PADDING),
            x_max: camera_x_px.saturating_add(ACTIVATION_WINDOW_X_LEAD),
            y_min: camera_y_px.saturating_sub(ACTIVATION_WINDOW_PADDING),
            y_max: camera_y_px.saturating_add(ACTIVATION_WINDOW_Y_LEAD),
        }
    }

    pub(crate) fn contains(&self, x_pos: u32, y_pos: u32) -> bool {
        (self.x_min..=self.x_max).contains(&x_pos) && (self.y_min..=self.y_max).contains(&y_pos)
    }

    pub(crate) fn contains_fixed_signed_with_margin(
        &self,
        x_pos: u32,
        y_pos: u32,
        x_margin: u32,
        y_margin: u32,
    ) -> bool {
        let x_pos = (x_pos as i32) >> 16;
        let y_pos = (y_pos as i32) >> 16;
        let x_min = (self.x_min as i32).saturating_sub(x_margin as i32);
        let x_max = (self.x_max as i32).saturating_add(x_margin as i32);
        let y_min = (self.y_min as i32).saturating_sub(y_margin as i32);
        let y_max = (self.y_max as i32).saturating_add(y_margin as i32);
        (x_min..=x_max).contains(&x_pos) && (y_min..=y_max).contains(&y_pos)
    }
}

impl ActiveEnemy {
    /// Whether this enemy can be damaged (stomped, hit by projectile, etc.)
    pub(crate) fn is_vulnerable(&self) -> bool {
        if self.hit_points == 0 {
            return false;
        }
        if matches!(self.family, EnemyFamily::TarMonster) {
            return matches!(
                self.tar_monster_state,
                TarMonsterState::Visible | TarMonsterState::Attacking
            );
        }
        true
    }

    pub(crate) fn snapshot(&self) -> NativeObjectSnapshot {
        let shared_walker_state =
            matches!(self.family, EnemyFamily::DragonWalking | EnemyFamily::Lion)
                .then_some(self.shared_walker_state);
        let shared_walker_patrol_timer =
            matches!(self.family, EnemyFamily::DragonWalking | EnemyFamily::Lion)
                .then_some(self.shared_walker_patrol_timer);
        let shared_walker_patrol_short_next = matches!(self.family, EnemyFamily::DragonWalking)
            .then_some(self.shared_walker_patrol_short_next);
        let walk_phase =
            matches!(self.family, EnemyFamily::DragonWalking).then_some(self.dragon_script_cmd);
        let wait_timer =
            matches!(self.family, EnemyFamily::DragonWalking).then_some(self.wait_timer);
        let mut debug_fields = std::collections::BTreeMap::new();
        debug_fields.insert("segment_index".into(), self.segment_index.to_string());
        if matches!(self.family, EnemyFamily::DragonWalking) {
            debug_fields.insert("grounded_frames".into(), self.grounded_frames.to_string());
            if let Some(branch) = self.shared_walker_debug_branch {
                debug_fields.insert("shared_walker_branch".into(), branch.into());
            }
            if let Some(dx_to_kid_px) = self.shared_walker_dx_to_kid_px {
                debug_fields.insert(
                    "shared_walker_dx_to_kid_px".into(),
                    dx_to_kid_px.to_string(),
                );
            }
            if let Some(next_x_px) = self.shared_walker_terrain_trace.next_x_px {
                debug_fields.insert(
                    "shared_walker_probe_next_x_px".into(),
                    next_x_px.to_string(),
                );
            }
            if let Some(probe) = self.shared_walker_terrain_trace.center_floor {
                debug_fields.insert(
                    "shared_walker_probe_center_floor".into(),
                    format_shared_walker_floor_probe(probe),
                );
            }
            if let Some(probe) = self.shared_walker_terrain_trace.left_floor {
                debug_fields.insert(
                    "shared_walker_probe_left_floor".into(),
                    format_shared_walker_floor_probe(probe),
                );
            }
            if let Some(probe) = self.shared_walker_terrain_trace.right_floor {
                debug_fields.insert(
                    "shared_walker_probe_right_floor".into(),
                    format_shared_walker_floor_probe(probe),
                );
            }
            if let Some(probe) = self.shared_walker_terrain_trace.wall {
                debug_fields.insert(
                    "shared_walker_probe_wall".into(),
                    format_shared_walker_wall_probe(probe),
                );
            }
        }
        NativeObjectSnapshot {
            kind: NativeObjectKind::Enemy {
                object_type: self.family.object_type(),
            },
            x_pos: self.x_pos,
            y_pos: self.y_pos,
            layout_index: Some(self.layout_index),
            x_velocity: Some(self.x_velocity),
            y_velocity: Some(self.y_velocity),
            facing: Some(self.facing),
            on_ground: Some(self.on_ground),
            hit_points: Some(self.hit_points),
            damage_flash: self.damage_flash_frames > 0,
            palette_variant: self.palette_variant,
            stationary: self.dragon_phase.is_stationary(),
            shared_walker_state,
            shared_walker_patrol_timer,
            shared_walker_patrol_short_next,
            walk_phase,
            wait_timer,
            debug_fields,
        }
    }
}

fn format_shared_walker_floor_probe(probe: terrain::FloorProbe) -> String {
    format!(
        "ground={}/y={}/cell={},{}",
        u8::from(probe.on_ground),
        probe.floor_y_px,
        probe.cell_x,
        probe.cell_y
    )
}

fn format_shared_walker_wall_probe(probe: terrain::WallProbe) -> String {
    format!(
        "hit={}/x={}/cell={},{}",
        u8::from(probe.hit),
        probe.wall_x_px,
        probe.cell_x,
        probe.cell_y
    )
}

impl EnemyFamily {
    pub(crate) fn archetype(self) -> EnemyArchetype {
        match self {
            Self::Diamond => EnemyArchetype::Collectible,
            Self::Dragon | Self::Cloud => EnemyArchetype::Flyer,
            Self::DragonWalking => EnemyArchetype::Pacer, // paces + breath attack
            Self::Sphere | Self::Fireball => EnemyArchetype::Bouncer,
            Self::UFO | Self::Tornado => EnemyArchetype::Flyer,
            Self::MiniHoppingSkull => EnemyArchetype::Hopper,
            Self::BigHoppingSkull => EnemyArchetype::Bouncer, // continuous bouncer, handled in step_enemy
            Self::Drips => EnemyArchetype::Stationary,        // ceiling dropper
            Self::Goat
            | Self::Armadillo
            | Self::Tank
            | Self::TankShooting
            | Self::Crab
            | Self::Scorpion
            | Self::Driller
            | Self::Orca
            | Self::Hand
            | Self::Robot
            | Self::Lion
            | Self::FireDemon => EnemyArchetype::Pacer,
            Self::Ninja => EnemyArchetype::Pacer, // uses Pacer in stealth, but step_active_enemies dispatches to step_ninja
            Self::EmoRock => EnemyArchetype::Hoverer,
            Self::Archer => EnemyArchetype::Stationary,
            Self::SpinningTwins => EnemyArchetype::Pacer, // orbit phase dispatched to step_spinning_twins
            Self::TarMonster => EnemyArchetype::Stationary, // emerge/submerge cycle
            Self::HeadyMetal | Self::Shishkaboss | Self::BoomerangBosses | Self::BagelBrothers => {
                EnemyArchetype::Tracker
            }
            Self::Unknown(_) => EnemyArchetype::Inert,
        }
    }

    pub(crate) fn from_layout_object(enemy_type_slots: &[u16], object: &EnemyLayoutObject) -> Self {
        match resolved_enemy_id(enemy_type_slots, object.object_type) {
            Some(0x01) => Self::Diamond,
            Some(0x00) => Self::FireDemon,
            Some(0x03) => Self::Robot,
            Some(0x04) => Self::Armadillo,
            Some(0x05) => Self::TarMonster,
            Some(0x09) => Self::Crab,
            Some(0x0A) => Self::Tank,
            Some(0x0B) => Self::TankShooting,
            Some(0x0C) => Self::Dragon,
            Some(0x0D) => Self::DragonWalking,
            Some(0x06) => Self::Sphere,
            Some(0x07) => Self::Archer,
            Some(0x08) => Self::Orca,
            Some(0x0E) => Self::Cloud,
            Some(0x0F) => Self::UFO,
            Some(0x10) => Self::Goat,
            Some(0x11) => Self::Ninja,
            Some(0x12) => Self::Lion,
            Some(0x13) => Self::Scorpion,
            Some(0x14) => Self::SpinningTwins,
            Some(0x16) => Self::Drips,
            Some(0x17) => Self::Hand,
            Some(0x18) => Self::Tornado,
            Some(0x1A) => Self::Driller,
            Some(0x19) => Self::Fireball,
            Some(0x1B) => Self::EmoRock,
            Some(0x1C) => Self::MiniHoppingSkull,
            Some(0x1D) => Self::BigHoppingSkull,
            Some(0x20) => Self::HeadyMetal,
            Some(0x22) => Self::Shishkaboss,
            Some(0x23) => Self::BoomerangBosses,
            Some(0x24) => Self::BagelBrothers,
            Some(enemy_id) => Self::Unknown(enemy_id),
            None => Self::Unknown(object.object_type),
        }
    }

    pub(crate) fn object_type(self) -> u8 {
        match self {
            Self::FireDemon => 0x00,
            Self::Diamond => 0x01,
            Self::Robot => 0x03,
            Self::Armadillo => 0x04,
            Self::TarMonster => 0x05,
            Self::Sphere => 0x06,
            Self::Archer => 0x07,
            Self::Orca => 0x08,
            Self::Crab => 0x09,
            Self::Tank => 0x0A,
            Self::TankShooting => 0x0B,
            Self::Dragon => 0x0C,
            Self::DragonWalking => 0x0D,
            Self::Cloud => 0x0E,
            Self::UFO => 0x0F,
            Self::Goat => 0x10,
            Self::Ninja => 0x11,
            Self::Lion => 0x12,
            Self::Scorpion => 0x13,
            Self::SpinningTwins => 0x14,
            Self::Drips => 0x16,
            Self::Hand => 0x17,
            Self::Tornado => 0x18,
            Self::Fireball => 0x19,
            Self::Driller => 0x1A,
            Self::EmoRock => 0x1B,
            Self::MiniHoppingSkull => 0x1C,
            Self::BigHoppingSkull => 0x1D,
            Self::HeadyMetal => 0x20,
            Self::Shishkaboss => 0x22,
            Self::BoomerangBosses => 0x23,
            Self::BagelBrothers => 0x24,
            Self::Unknown(enemy_id) => enemy_id,
        }
    }

    #[cfg(test)]
    fn snapshot(
        self,
        object: &EnemyLayoutObject,
        window: ActivationWindow,
    ) -> Option<NativeObjectSnapshot> {
        let object_x = u32::from(object.x);
        let object_y = u32::from(object.y);
        if !window.contains(object_x, object_y) {
            return None;
        }
        Some(NativeObjectSnapshot {
            kind: NativeObjectKind::Enemy {
                object_type: self.object_type(),
            },
            x_pos: object_x << 16,
            y_pos: object_y << 16,
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
        })
    }
}

#[cfg(test)]
pub(crate) fn build_enemy_roster(
    layout: &EnemyLayout,
    window: ActivationWindow,
) -> Vec<NativeObjectSnapshot> {
    let view = enemy_layout_view(layout, window.x_min, window.y_min);
    view.objects
        .iter()
        .filter_map(|object| {
            EnemyFamily::from_layout_object(view.enemy_type_slots, object).snapshot(object, window)
        })
        .collect()
}

/// Activate enemies that enter the camera window and deactivate those that leave.
/// Newly activated enemies start at their layout position.
#[allow(clippy::too_many_arguments)]
#[cfg(test)]
pub(crate) fn activate_enemies(
    active: &mut Vec<ActiveEnemy>,
    layout: &EnemyLayout,
    window: ActivationWindow,
    camera_x_px: u32,
    camera_y_px: u32,
    level_bounds: physics::LevelBounds,
    player_x: u32,
    player_y: u32,
    killed: &std::collections::HashSet<(usize, usize)>,
) {
    let mut rng = RomRandomState::default();
    activate_enemies_with_rng(
        active,
        layout,
        window,
        camera_x_px,
        camera_y_px,
        level_bounds,
        player_x,
        player_y,
        0,
        killed,
        &mut rng,
    );
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn activate_enemies_with_rng(
    active: &mut Vec<ActiveEnemy>,
    layout: &EnemyLayout,
    window: ActivationWindow,
    camera_x_px: u32,
    camera_y_px: u32,
    level_bounds: physics::LevelBounds,
    player_x: u32,
    player_y: u32,
    blank_top_rows: u16,
    killed: &std::collections::HashSet<(usize, usize)>,
    rng: &mut RomRandomState,
) {
    let view = enemy_layout_view(layout, camera_x_px, camera_y_px);
    // Most enemies despawn on spawn-point exit. Flyers keep their live object
    // around longer; the Level 1 Dragon continues drifting after its layout
    // anchor scrolls behind the camera.
    // ASM loc_3C46E (kid.asm:43965-44020): the shared deactivation routine
    // entered via loc_3C436 trampoline. Reads object_meta, dispatches
    // SpinningTwins to loc_3BC56, Driller/Tank to loc_3C494 (drive sound
    // stop with refcount decrement and optional PlaySound2), then falls
    // through to loc_3C4C0 which decrements Number_of_Enemy, updates
    // EnemyStatus_Table from $42(a5), and calls j_Delete_CurrentObject.
    //
    // loc_3C4A2 (kid.asm:43987): $19(a3)==0 path — tests $6A(a5);
    // if nonzero skips sound (bne.s loc_3C4C0), else falls to loc_3C4A8.
    // loc_3C4A8 (kid.asm:43991): subi.b #1,(a4,d7.w) refcount decrement;
    // plays j_PlaySound2 with byte_3C46C[d6] when counter reaches zero.
    // loc_3C4E6 (kid.asm:44013): positive $42(a5) slot writes $2168 into
    // EnemyStatus_Table[d0] as the respawn marker.
    //
    // Native handles deactivation by dropping the enemy from the active
    // list; sound cleanup is managed separately by the sound system.
    let active_enemy_count = active.len();
    active.retain(|enemy| {
        enemy_should_remain_active(
            enemy,
            &view,
            window,
            camera_x_px,
            camera_y_px,
            level_bounds,
            active_enemy_count,
        )
    });

    // Activate new enemies that entered the window.
    for (index, object) in view.objects.iter().enumerate() {
        let object_x = u32::from(object.x);
        let object_y = u32::from(object.y);
        if !window.contains(object_x, object_y) {
            continue;
        }
        let family = EnemyFamily::from_layout_object(view.enemy_type_slots, object);
        if matches!(family, EnemyFamily::MiniHoppingSkull)
            && !mini_hopping_skull_spawn_gate_loc_3e9d0(object_x, object_y, player_x, player_y)
        {
            continue;
        }
        if active
            .iter()
            .any(|e| e.segment_index == view.segment_index && e.layout_index == index)
        {
            continue;
        }
        // Don't respawn killed enemies (ASM: per-segment killed bitfield).
        if killed.contains(&(view.segment_index, index)) {
            continue;
        }
        // ASM init routines copy the layout word at offset +2 straight into
        // enemy_level(a3). Dragon Flying is the exception: it reuses the same
        // layout word as a script variant selector instead of HP scaling.
        let enemy_level = object.hit_points_plus_one as u8;
        let spawn_y = spawn_y_pos_sub_36FF4(family, object_y, blank_top_rows);
        // Dragon Flying: variant from layout, anchor Y from spawn position (ASM $44(a5)).
        let dragon_variant = if matches!(family, EnemyFamily::Dragon) {
            object.hit_points_plus_one as u8
        } else {
            0
        };

        // Multi-head bosses spawn 3 head objects per layout entry.
        let head_count = if is_multi_head_boss(family) { 3u8 } else { 1 };
        let (head_hp_table, head_x_offsets, head_y_offsets) = if is_multi_head_boss(family) {
            boss_head_config(family)
        } else {
            ([0u8; 3], [0i16; 3], [0i16; 3])
        };

        for head_idx in 0..head_count {
            let hit_points = if is_multi_head_boss(family) {
                head_hp_table[head_idx as usize]
            } else {
                spawn_hit_points(family, enemy_level)
            };
            let base_x = spawn_x_pos(family, object_x);
            let base_y = spawn_y;
            let head_x = if is_multi_head_boss(family) {
                let offset_px = head_x_offsets[head_idx as usize];
                (base_x as i64 + (i64::from(offset_px) << 16)).clamp(0, i64::from(u32::MAX)) as u32
            } else {
                base_x
            };
            let head_y = if is_multi_head_boss(family) {
                let offset_px = head_y_offsets[head_idx as usize];
                (base_y as i64 + (i64::from(offset_px) << 16)).clamp(0, i64::from(u32::MAX)) as u32
            } else {
                base_y
            };

            let mut enemy = ActiveEnemy {
                segment_index: view.segment_index,
                layout_index: index,
                enemy_level,
                family,
                x_pos: head_x,
                y_pos: head_y,
                x_velocity: spawn_x_velocity(family, enemy_level),
                y_velocity: spawn_y_velocity(family, enemy_level),
                target_x_velocity: 0,
                target_y_velocity: 0,
                x_acceleration: spawn_x_acceleration(family),
                y_acceleration: spawn_y_acceleration(family),
                facing: spawn_facing(family, head_x, player_x),
                on_ground: matches!(family.archetype(), EnemyArchetype::Pacer),
                hit_points,
                grounded_frames: if matches!(family, EnemyFamily::DragonWalking) {
                    1
                } else {
                    0
                },
                damage_flash_frames: 0,
                palette_variant: resolved_enemy_palette(view.enemy_type_slots, object.object_type),
                wait_timer: spawn_wait_timer(family, enemy_level, player_x, rng),
                dragon_phase: spawn_dragon_phase(family),
                dragon_phase_frames: 0,
                dragon_script_variant: dragon_variant,
                dragon_script_cmd: 0,
                dragon_anchor_y: head_y,
                dragon_descent_index: 0,
                dragon_descent_frames: 0,
                ninja_state: NinjaState::Stealth,
                hand_state: HandState::Walking,
                lion_state: LionState::Walking,
                robot_state: RobotState::Walking,
                orca_state: OrcaState::Walking,
                wall_pinned: false,
                tar_monster_state: TarMonsterState::default(),
                spinning_twins_phase: SpinningTwinsPhase::default(),
                spinning_twins_orbit_index: 0,
                ufo_beam_segments: 0,
                ufo_beam_interval: 0,
                boss_head_index: if is_multi_head_boss(family) {
                    head_idx
                } else {
                    BOSS_HEAD_NONE
                },
                scorpion_tracking_lock: false,
                crab_frozen: false,
                flight_double_acceleration: matches!(
                    family,
                    EnemyFamily::UFO | EnemyFamily::Tornado
                ) && enemy_level >= 1,
                flight_double_terminal_speed: matches!(
                    family,
                    EnemyFamily::UFO | EnemyFamily::Tornado
                ) && enemy_level >= 2,
                walk_speed_override: 0,
                shared_walker_state: spawn_shared_walker_state(family),
                shared_walker_patrol_timer: spawn_shared_walker_patrol_timer(family),
                shared_walker_patrol_short_next: spawn_shared_walker_patrol_short_next(family),
                shared_walker_debug_branch: None,
                shared_walker_dx_to_kid_px: None,
                shared_walker_terrain_trace: SharedWalkerTerrainTrace::default(),
            };
            if matches!(family, EnemyFamily::Tornado) {
                // ASM Tornado.asm seeds a movement pattern before entering the
                // per-frame loop, so activate the first script entry here.
                reload_tornado_pattern_script(&mut enemy);
            }
            active.push(enemy);
        }
    }
}

/// Step all active enemies: apply movement, gravity, and terrain collision.
#[cfg(test)]
#[allow(clippy::too_many_arguments)]
pub(crate) fn step_active_enemies(
    active: &mut [ActiveEnemy],
    block_grid: Option<&terrain::BlockGrid>,
    player_x_pos: Option<u32>,
    player_y_pos: Option<u32>,
    player_facing: Option<Direction>,
    player_x_velocity: Option<i32>,
    player_form: PlayerForm,
    projectiles: Option<&mut Vec<EnemyProjectile>>,
    drilled_blocks: &mut Vec<(u16, u16)>,
) {
    let mut rng = RomRandomState::default();
    step_active_enemies_with_rng(
        active,
        block_grid,
        player_x_pos,
        player_y_pos,
        None,
        player_facing,
        player_x_velocity,
        player_form,
        projectiles,
        &mut rng,
        drilled_blocks,
    );
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn step_active_enemies_with_rng(
    active: &mut [ActiveEnemy],
    block_grid: Option<&terrain::BlockGrid>,
    player_x_pos: Option<u32>,
    player_y_pos: Option<u32>,
    camera_x_px: Option<u32>,
    player_facing: Option<Direction>,
    player_x_velocity: Option<i32>,
    player_form: PlayerForm,
    mut projectiles: Option<&mut Vec<EnemyProjectile>>,
    rng: &mut RomRandomState,
    drilled_blocks: &mut Vec<(u16, u16)>,
) {
    for enemy in active.iter_mut() {
        enemy.damage_flash_frames = enemy.damage_flash_frames.saturating_sub(1);
        match enemy.family.archetype() {
            EnemyArchetype::Collectible | EnemyArchetype::Inert => {}
            EnemyArchetype::Bouncer => {
                if matches!(enemy.family, EnemyFamily::BigHoppingSkull) {
                    step_big_hopping_skull(enemy, block_grid);
                } else {
                    step_bouncer_with_rng(enemy, block_grid, camera_x_px, rng);
                }
            }
            EnemyArchetype::Flyer => {
                if matches!(enemy.family, EnemyFamily::Dragon) {
                    step_dragon(
                        enemy,
                        block_grid,
                        player_x_pos,
                        player_y_pos,
                        &mut projectiles,
                    );
                } else {
                    step_flyer_with_rng(enemy, block_grid, rng);
                    if matches!(enemy.family, EnemyFamily::UFO)
                        && let Some(ref mut projs) = projectiles
                    {
                        step_ufo_beam(enemy, projs, player_x_pos, player_y_pos);
                    }
                }
            }
            EnemyArchetype::Hopper => step_hopper(enemy, block_grid, player_x_pos),
            EnemyArchetype::Hoverer => step_emo_rock(enemy, block_grid, player_x_pos, player_y_pos),
            EnemyArchetype::Pacer => {
                if matches!(enemy.family, EnemyFamily::Ninja) {
                    step_ninja(enemy, block_grid, player_x_pos, player_y_pos, player_form);
                } else if matches!(enemy.family, EnemyFamily::Hand) {
                    step_hand(enemy, block_grid, player_x_pos, player_y_pos);
                } else if matches!(enemy.family, EnemyFamily::Lion) {
                    step_lion(enemy, block_grid, player_x_pos, &mut projectiles);
                } else if matches!(enemy.family, EnemyFamily::Robot) {
                    step_robot_with_rng(enemy, block_grid, player_x_pos, &mut projectiles, rng);
                } else if matches!(enemy.family, EnemyFamily::Orca) {
                    step_orca(enemy, block_grid, player_x_pos);
                } else if matches!(enemy.family, EnemyFamily::SpinningTwins) {
                    step_spinning_twins(enemy, block_grid, player_x_pos, player_y_pos);
                } else if matches!(enemy.family, EnemyFamily::DragonWalking) {
                    if let Some(ref mut projs) = projectiles {
                        step_dragon_walking(enemy, block_grid, projs, player_x_pos);
                    } else {
                        step_dragon_walking(enemy, block_grid, &mut Vec::new(), player_x_pos);
                    }
                } else if matches!(enemy.family, EnemyFamily::Driller) {
                    // Driller.asm loc_3BE78: drills through Smash/Ice blocks while pacing.
                    let cells = step_driller(enemy, block_grid);
                    drilled_blocks.extend(cells);
                } else {
                    // Crab: player-proximity freeze callback (Crab.asm loc_3B0BE-loc_3B174).
                    // Check freeze before stepping; if frozen, skip movement.
                    if matches!(enemy.family, EnemyFamily::Crab) {
                        step_crab_freeze(enemy, player_x_pos, player_facing, player_x_velocity);
                        if enemy.crab_frozen {
                            continue; // skip pacer walk while frozen
                        }
                    }
                    step_pacer(enemy, block_grid);
                    // Goat: player-tracking callback (Goat.asm loc_3CC18).
                    // If player is >32px away, turn to face them.
                    if matches!(enemy.family, EnemyFamily::Goat)
                        && let Some(px) = player_x_pos
                    {
                        let dx = (px as i64 - enemy.x_pos as i64) >> 16;
                        if dx.abs() > 0x20 {
                            enemy.facing = if dx > 0 {
                                Direction::Right
                            } else {
                                Direction::Left
                            };
                        }
                    }
                    // Scorpion: tracking callback (Scorpion.asm loc_3DD7E).
                    // Within 170px: face toward player (unless tracking-locked),
                    // boost speed to $38000 if enemy_level > 1 (HP > 3).
                    // Outside 170px: reset speed to base $18000, clear tracking lock
                    // if player is airborne.
                    if matches!(enemy.family, EnemyFamily::Scorpion) {
                        step_scorpion_tracking(enemy, player_x_pos, player_y_pos);
                    }
                    // ASM Tank_Shooting.asm loc_3C50E/loc_3B730: only the
                    // shooting tank variant emits bullets, and only on the
                    // turret animation gate.
                    if matches!(enemy.family, EnemyFamily::TankShooting)
                        && let Some(ref mut projs) = projectiles
                    {
                        step_tank_shooting_fire(enemy, projs);
                    }
                }
            }
            EnemyArchetype::Stationary => {
                if matches!(enemy.family, EnemyFamily::TarMonster) {
                    if let Some(ref mut projs) = projectiles {
                        step_tar_monster(enemy, projs);
                    } else {
                        step_tar_monster(enemy, &mut Vec::new());
                    }
                } else if matches!(enemy.family, EnemyFamily::Drips) {
                    if let Some(projs) = projectiles.as_deref_mut() {
                        step_drips(enemy, projs);
                    }
                } else if let Some(ref mut projs) = projectiles {
                    step_archer(enemy, projs, player_x_pos, player_y_pos);
                }
            }
            EnemyArchetype::Tracker => {
                step_tracker(enemy, player_x_pos, player_y_pos);
                // Bosses periodically spawn projectiles while tracking
                if let Some(ref mut projs) = projectiles {
                    step_boss_fire(enemy, projs, player_x_pos, player_y_pos);
                }
            }
        }
    }
}

pub(crate) fn step_flyer_with_rng(
    enemy: &mut ActiveEnemy,
    block_grid: Option<&terrain::BlockGrid>,
    rng: &mut RomRandomState,
) {
    // Cloud: horizontal flight with wall reversal
    if matches!(enemy.family, EnemyFamily::Cloud) {
        step_cloud(enemy, block_grid);
        return;
    }

    // UFO: acceleration-based drifting with periodic direction changes
    if matches!(enemy.family, EnemyFamily::UFO) {
        step_ufo(enemy, block_grid);
        return;
    }

    // Tornado: acceleration-based bouncing patrol with wall/floor reversal
    if matches!(enemy.family, EnemyFamily::Tornado) {
        step_tornado_with_rng(enemy, block_grid, rng);
        return;
    }

    enemy.x_pos = (enemy.x_pos as i64 + i64::from(enemy.x_velocity)) as u32;
    enemy.y_pos = (enemy.y_pos as i64 + i64::from(enemy.y_velocity)) as u32;
}

/// Cloud flies horizontally, reversing direction on wall contact.
/// ASM `Cloud.asm loc_35BDE -> loc_35C54`: `move.w #$3C,d3` is written after
/// wall reversal, but the active loop never reads `d3`, so Cloud resumes
/// flying immediately on the next frame instead of idling.
const CLOUD_CONTROL_SCRIPT_WORDS_LOC_35BAC: [u16; 5] = [0x7800, 0x0004, 0x1E00, 0x0008, 0xFFFF];

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn cloud_control_script_words_loc_35BAC() -> &'static [u16] {
    // ASM Cloud.asm loc_35BAC: `lea (unk_35BC4).l,a0` restarts the command
    // stream before the dispatcher fetches the next big-endian word.
    &CLOUD_CONTROL_SCRIPT_WORDS_LOC_35BAC
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CloudControlDispatchLoc35BB2 {
    Wait { frames: u8 },
    SetStatus44,
    ClearStatus44,
    PlayTransitionAnimation,
    RestartScript,
    Unknown { table_offset: u8 },
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn cloud_control_script_dispatch_loc_35BB2(
    command_word: u16,
) -> CloudControlDispatchLoc35BB2 {
    // ASM Cloud.asm loc_35BB2: `move.w (a0)+,d0; bmi.s ...; move.b d0,d7`
    // uses the low byte as a longword-table offset while wait commands keep
    // their duration in the high byte for the wait loop.
    if command_word & 0x8000 != 0 {
        return CloudControlDispatchLoc35BB2::RestartScript;
    }

    let [frames, table_offset] = command_word.to_be_bytes();
    match table_offset {
        0x00 => CloudControlDispatchLoc35BB2::Wait { frames },
        0x04 => CloudControlDispatchLoc35BB2::SetStatus44,
        0x08 => CloudControlDispatchLoc35BB2::ClearStatus44,
        0x0C => CloudControlDispatchLoc35BB2::PlayTransitionAnimation,
        _ => CloudControlDispatchLoc35BB2::Unknown { table_offset },
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct CloudDeathSequenceLoc35C0E {
    pub(crate) x_velocity: i32,
    pub(crate) animation_script: &'static str,
    pub(crate) status_route: DeactivationStatusActionLoc3C4C0,
    pub(crate) deletes_current_object: bool,
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn cloud_death_sequence_loc_35C0E(object_status_slot: u8) -> CloudDeathSequenceLoc35C0E {
    // ASM Cloud.asm loc_35C0E: clear `x_vel(a3)`, run `stru_35EBA` to
    // completion, apply the `$42(a5)` status route, then delete the object.
    CloudDeathSequenceLoc35C0E {
        x_velocity: 0,
        animation_script: "stru_35EBA",
        status_route: deactivation_status_update_loc_3C4C0(object_status_slot),
        deletes_current_object: true,
    }
}

pub(crate) fn step_cloud(enemy: &mut ActiveEnemy, block_grid: Option<&terrain::BlockGrid>) {
    // Advance position
    enemy.x_pos = (enemy.x_pos as i64 + i64::from(enemy.x_velocity)) as u32;

    // Wall probe: reverse on wall contact (Cloud.asm loc_35BDE)
    if let Some(grid) = block_grid {
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
            enemy.x_velocity = -enemy.x_velocity;
            enemy.facing = opposite_direction(enemy.facing);
        }
    }
}

/// UFO: acceleration-based drifting, reverses direction periodically.
/// Uses `wait_timer` as a countdown; every 60 frames reverses acceleration.
/// Acceleration ±$400 per frame, terminal velocity ±$18000.
pub(crate) const UFO_ACCELERATION: i32 = 0x400;
pub(crate) const UFO_TERMINAL_VELOCITY: i32 = 0x18000;
pub(crate) const UFO_PATTERN_FRAMES: u16 = 60;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) struct UfoMotionCommand {
    pub(crate) x_acceleration: i32,
    pub(crate) target_x_velocity: i32,
    pub(crate) y_acceleration: i32,
    pub(crate) target_y_velocity: i32,
}

#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn ufo_double_acceleration_loc_34e10(
    mut command: UfoMotionCommand,
    double_acceleration: bool,
) -> UfoMotionCommand {
    if double_acceleration {
        // ASM UFO.asm loc_34E10: non-zero `$4E(a5)` doubles d0 and d2 only.
        command.x_acceleration = command.x_acceleration.saturating_mul(2);
        command.y_acceleration = command.y_acceleration.saturating_mul(2);
    }
    command
}

#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn ufo_double_terminal_speed_loc_34e1c(
    mut command: UfoMotionCommand,
    double_terminal_speed: bool,
) -> UfoMotionCommand {
    if double_terminal_speed {
        // ASM UFO.asm loc_34E1C: non-zero `$4F(a5)` doubles d1 and d3 only.
        command.target_x_velocity = command.target_x_velocity.saturating_mul(2);
        command.target_y_velocity = command.target_y_velocity.saturating_mul(2);
    }
    command
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) struct UfoTrackingCommandLoc34E2A {
    pub(crate) motion: UfoMotionCommand,
    pub(crate) y_velocity: i32,
    pub(crate) facing: Direction,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) struct UfoBeamStartLoc34Eec {
    pub(crate) beam_segments: u8,
    pub(crate) beam_interval: u8,
    pub(crate) x_velocity: i32,
    pub(crate) y_velocity: i32,
    pub(crate) wait_timer: u16,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) struct UfoOnScreenCountLoc34F64 {
    pub(crate) number_ufos_on_screen: u16,
    pub(crate) ufo_on_screen_latch: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) struct UfoOffScreenGateLoc34F6E {
    pub(crate) number_ufos_on_screen: u16,
    pub(crate) play_hover_stop_sound: bool,
    pub(crate) clear_latch_tail: bool,
}

#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn ufo_on_screen_count_loc_34f64(
    number_ufos_on_screen: u16,
) -> UfoOnScreenCountLoc34F64 {
    // ASM UFO.asm loc_34F64: when a previously-unlatched UFO is inside the
    // screen window, `addq.w #1,(Number_UFOs_OnScreen).w; st $50(a5)`.
    UfoOnScreenCountLoc34F64 {
        number_ufos_on_screen: number_ufos_on_screen.wrapping_add(1),
        ufo_on_screen_latch: true,
    }
}

#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn ufo_off_screen_gate_loc_34f6e(
    number_ufos_on_screen: u16,
    ufo_on_screen_latch: bool,
) -> UfoOffScreenGateLoc34F6E {
    // ASM UFO.asm loc_34F6E: off-screen UFOs with clear `$50(a5)` return
    // without touching `Number_UFOs_OnScreen`; latched UFOs decrement it and
    // only call `j_PlaySound2` when the decremented word is zero.
    if !ufo_on_screen_latch {
        return UfoOffScreenGateLoc34F6E {
            number_ufos_on_screen,
            play_hover_stop_sound: false,
            clear_latch_tail: false,
        };
    }

    let number_ufos_on_screen = number_ufos_on_screen.wrapping_sub(1);
    UfoOffScreenGateLoc34F6E {
        number_ufos_on_screen,
        play_hover_stop_sound: number_ufos_on_screen == 0,
        clear_latch_tail: true,
    }
}

#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn ufo_clear_on_screen_latch_loc_34f88() -> bool {
    // ASM UFO.asm loc_34F88: the off-screen tail clears `$50(a5)` with `sf`.
    false
}

#[cfg_attr(not(test), allow(dead_code))]
fn fixed_integer_word(pos: u32) -> i16 {
    (pos >> 16) as u16 as i16
}

#[cfg_attr(not(test), allow(dead_code))]
#[allow(clippy::too_many_arguments)]
pub(crate) fn ufo_tracking_command_loc_34e2a(
    ufo_x_pos: u32,
    ufo_y_pos: u32,
    kid_x_pos: u32,
    kid_y_pos: u32,
    tracking_enabled: bool,
    some_ufo_shooting: bool,
    double_acceleration: bool,
    double_terminal_speed: bool,
) -> Option<UfoTrackingCommandLoc34E2A> {
    // ASM UFO.asm loc_34E2A: `$4D(a5)` and `Some_UFO_Shooting` gate the whole
    // below-and-near Kid chase branch before any velocity state is changed.
    if !tracking_enabled || some_ufo_shooting {
        return None;
    }

    let ufo_x = fixed_integer_word(ufo_x_pos);
    let ufo_y = fixed_integer_word(ufo_y_pos);
    let kid_x = fixed_integer_word(kid_x_pos);
    let kid_y = fixed_integer_word(kid_y_pos);
    if kid_y <= ufo_y {
        return None;
    }

    let left_edge = kid_x.wrapping_sub(0x30);
    if ufo_x < left_edge {
        return None;
    }
    let right_edge = left_edge.wrapping_add(0x60);
    if ufo_x > right_edge {
        return None;
    }

    let command = if ufo_x < kid_x {
        ufo_right_tracking_command_loc_34e86()
    } else {
        UfoMotionCommand {
            x_acceleration: -0x1000,
            target_x_velocity: -0x20000,
            y_acceleration: 0,
            target_y_velocity: 0,
        }
    };
    let motion = ufo_tracking_tail_loc_34e96(command, double_acceleration, double_terminal_speed);

    Some(UfoTrackingCommandLoc34E2A {
        motion,
        y_velocity: 0,
        facing: if ufo_x < kid_x {
            Direction::Right
        } else {
            Direction::Left
        },
    })
}

#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn ufo_right_tracking_command_loc_34e86() -> UfoMotionCommand {
    // ASM UFO.asm loc_34E86: Kid is to the right, so d1/d0 become
    // +$20000/+$1000 and `st x_direction(a3)` marks the UFO as flipped.
    UfoMotionCommand {
        x_acceleration: 0x1000,
        target_x_velocity: 0x20000,
        y_acceleration: 0,
        target_y_velocity: 0,
    }
}

#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn ufo_tracking_tail_loc_34e96(
    command: UfoMotionCommand,
    double_acceleration: bool,
    double_terminal_speed: bool,
) -> UfoMotionCommand {
    // ASM UFO.asm loc_34E96: both left/right Kid chase branches share the
    // `bsr.w loc_34E10` scaling tail before returning.
    ufo_double_terminal_speed_loc_34e1c(
        ufo_double_acceleration_loc_34e10(command, double_acceleration),
        double_terminal_speed,
    )
}

pub(crate) fn step_ufo(enemy: &mut ActiveEnemy, block_grid: Option<&terrain::BlockGrid>) {
    // Apply acceleration to velocity (clamped to terminal)
    enemy.x_velocity = (enemy.x_velocity + enemy.x_acceleration)
        .clamp(-UFO_TERMINAL_VELOCITY, UFO_TERMINAL_VELOCITY);
    enemy.y_velocity = (enemy.y_velocity + enemy.y_acceleration)
        .clamp(-UFO_TERMINAL_VELOCITY, UFO_TERMINAL_VELOCITY);

    // Advance position
    enemy.x_pos = (enemy.x_pos as i64 + i64::from(enemy.x_velocity)) as u32;
    enemy.y_pos = (enemy.y_pos as i64 + i64::from(enemy.y_velocity)) as u32;

    // Periodic direction change
    if enemy.wait_timer == 0 {
        enemy.wait_timer = UFO_PATTERN_FRAMES;
        enemy.x_acceleration = -enemy.x_acceleration;
        enemy.y_acceleration = -enemy.y_acceleration;
    } else {
        enemy.wait_timer -= 1;
    }

    // Wall/floor reversal
    if let Some(grid) = block_grid {
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
            enemy.x_velocity = -enemy.x_velocity;
            enemy.x_acceleration = -enemy.x_acceleration;
            enemy.facing = opposite_direction(enemy.facing);
        }
    }
}

/// UFO beam: spawns downward beam segments at the UFO's position.
/// ASM: `ChkShoot` fires when player is directly below (±2px), `ChkLoadBeam` fires
/// randomly (~4/256 per frame). Spawns 4 segments at 5-frame intervals.
const UFO_BEAM_SEGMENT_COUNT: u8 = 4;
const UFO_BEAM_INTERVAL_FRAMES: u8 = 5;
/// Beam segment y_velocity: $30000 from ASM.
const UFO_BEAM_Y_VELOCITY: i32 = 0x3_0000;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) struct UfoBeamLoadTickLoc34FD6 {
    pub(crate) next_beam_segments: u8,
    pub(crate) next_beam_interval: u8,
    pub(crate) spawn_beam_segment: bool,
    pub(crate) clear_ufo_shooting_state: bool,
}

#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn ufo_beam_load_tick_loc_34fd6(
    shooting_active: bool,
    beam_segments: u8,
    beam_interval: u8,
) -> UfoBeamLoadTickLoc34FD6 {
    // ASM UFO.asm loc_34FD6: when `$46(a5)` is set, this counts down `$4A(a5)`
    // each frame, reloads it to 5 every beam segment, and decrements `$48(a5)`.
    if !shooting_active {
        return UfoBeamLoadTickLoc34FD6 {
            next_beam_segments: beam_segments,
            next_beam_interval: beam_interval,
            spawn_beam_segment: false,
            clear_ufo_shooting_state: false,
        };
    }

    let mut beam_interval = beam_interval.wrapping_sub(1);
    if beam_interval != 0 {
        return UfoBeamLoadTickLoc34FD6 {
            next_beam_segments: beam_segments,
            next_beam_interval: beam_interval,
            spawn_beam_segment: false,
            clear_ufo_shooting_state: false,
        };
    }

    beam_interval = UFO_BEAM_INTERVAL_FRAMES;
    let next_beam_segments = beam_segments.wrapping_sub(1);
    let clear_ufo_shooting_state = next_beam_segments == 0;
    UfoBeamLoadTickLoc34FD6 {
        next_beam_segments,
        next_beam_interval: beam_interval,
        spawn_beam_segment: !clear_ufo_shooting_state,
        clear_ufo_shooting_state,
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) struct UfoBeamClearLoc34FFE {
    pub(crate) ufo_shooting_latch: bool,
    pub(crate) global_shooting_flag: bool,
}

#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn ufo_beam_clear_shooting_state_loc_34ffe() -> UfoBeamClearLoc34FFE {
    // ASM UFO.asm loc_34FFE: when `$48(a5)` reaches zero, clear `$46(a5)` and
    // `Some_UFO_Shooting`.
    UfoBeamClearLoc34FFE {
        ufo_shooting_latch: false,
        global_shooting_flag: false,
    }
}

#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn ufo_beam_start_loc_34eec() -> UfoBeamStartLoc34Eec {
    // ASM UFO.asm loc_34EEC: targeted shooting starts the shared UFO beam
    // sequence, freezes the UFO, and reloads the one-second loop counter to $32.
    UfoBeamStartLoc34Eec {
        beam_segments: 4,
        beam_interval: 5,
        x_velocity: 0,
        y_velocity: 0,
        wait_timer: 0x32,
    }
}

fn apply_ufo_beam_start_loc_34eec(enemy: &mut ActiveEnemy) {
    let beam_start = ufo_beam_start_loc_34eec();
    enemy.ufo_beam_segments = beam_start.beam_segments;
    enemy.ufo_beam_interval = beam_start.beam_interval;
    enemy.x_velocity = beam_start.x_velocity;
    enemy.y_velocity = beam_start.y_velocity;
    enemy.wait_timer = beam_start.wait_timer;
}

pub(crate) fn step_ufo_beam(
    enemy: &mut ActiveEnemy,
    projectiles: &mut Vec<EnemyProjectile>,
    player_x: Option<u32>,
    player_y: Option<u32>,
) {
    // Active beam sequence: spawn segments at intervals
    if enemy.ufo_beam_segments > 0 {
        let tick =
            ufo_beam_load_tick_loc_34fd6(true, enemy.ufo_beam_segments, enemy.ufo_beam_interval);
        enemy.ufo_beam_segments = tick.next_beam_segments;
        enemy.ufo_beam_interval = tick.next_beam_interval;
        if tick.clear_ufo_shooting_state {
            let clear = ufo_beam_clear_shooting_state_loc_34ffe();
            if !clear.ufo_shooting_latch {
                enemy.ufo_beam_segments = 0;
            }
            let _ = clear.global_shooting_flag;
        }

        if tick.spawn_beam_segment {
            projectiles.push(EnemyProjectile {
                x_pos: enemy.x_pos,
                y_pos: (enemy.y_pos as i64 + (10 << 16)) as u32,
                x_velocity: 0,
                y_velocity: UFO_BEAM_Y_VELOCITY,
                lifetime: 60,
                gravity: 0,
                friendly: false,
                homing_accel: 0,
                speed_cap: 0,
                wall_bounce: false,
                ..Default::default()
            });
        }
        return;
    }

    // Check if player is directly below — targeted shoot (ASM ChkShoot)
    if let (Some(px), Some(py)) = (player_x, player_y) {
        let ex = enemy.x_pos >> 16;
        let ey = enemy.y_pos >> 16;
        let kx = px >> 16;
        let ky = py >> 16;
        // Player must be below UFO and within ±2px horizontally
        if ky > ey && kx.abs_diff(ex) <= 2 {
            apply_ufo_beam_start_loc_34eec(enemy);
            return;
        }
    }

    // Random beam: ~4/256 chance per frame (ASM ChkLoadBeam)
    // Use a deterministic pseudo-random from enemy position + wait_timer
    let pseudo_rand = (enemy
        .x_pos
        .wrapping_add(enemy.wait_timer as u32)
        .wrapping_mul(2654435761))
        >> 24;
    if pseudo_rand < 4 {
        enemy.ufo_beam_segments = UFO_BEAM_SEGMENT_COUNT;
        enemy.ufo_beam_interval = 0;
    }
}

/// Tornado: acceleration-based flight. ASM UFO.asm loc_35326/unk_35344/loc_35444
/// shares `Enemy_HandleAcceleration` with UFO. Terminal velocity from the table
/// entries is typically $18000. Cycle timer from `unk_3588C`: 60/40/30 by enemy_level.
pub(crate) const TORNADO_ACCELERATION: i32 = 0xC00;
/// Per-level cycle timers: level 0 = 60, level 1 = 40, level 2 = 30 (unk_3588C).
pub(crate) const TORNADO_PATTERN_FRAMES: [u16; 3] = [60, 40, 30];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct TornadoPattern {
    pub(crate) target_x_velocity: i32,
    pub(crate) target_y_velocity: i32,
    pub(crate) x_acceleration: i32,
    pub(crate) y_acceleration: i32,
}

// ASM Tornado.asm `unk_35A0C`: 16-entry motion script, followed by a `$00000001`
// sentinel that wraps back to entry 0 in `loc_359F2`.
pub(crate) const TORNADO_PATTERN_SCRIPT: [TornadoPattern; 16] = [
    TornadoPattern {
        target_x_velocity: 0x0001_8000,
        target_y_velocity: -0x0001_8000,
        x_acceleration: 0x0000_1800,
        y_acceleration: -0x0000_1800,
    },
    TornadoPattern {
        target_x_velocity: 0x0001_8000,
        target_y_velocity: 0x0001_8000,
        x_acceleration: 0x0000_1800,
        y_acceleration: 0x0000_1800,
    },
    TornadoPattern {
        target_x_velocity: 0x0001_8000,
        target_y_velocity: -0x0001_8000,
        x_acceleration: 0x0000_1800,
        y_acceleration: -0x0000_1800,
    },
    TornadoPattern {
        target_x_velocity: 0x0001_8000,
        target_y_velocity: 0x0001_8000,
        x_acceleration: 0x0000_1800,
        y_acceleration: 0x0000_1800,
    },
    TornadoPattern {
        target_x_velocity: 0x0001_8000,
        target_y_velocity: -0x0001_8000,
        x_acceleration: 0x0000_1800,
        y_acceleration: -0x0000_1800,
    },
    TornadoPattern {
        target_x_velocity: 0x0001_8000,
        target_y_velocity: 0x0001_8000,
        x_acceleration: 0x0000_1800,
        y_acceleration: 0x0000_1800,
    },
    TornadoPattern {
        target_x_velocity: 0x0001_8000,
        target_y_velocity: -0x0001_8000,
        x_acceleration: 0x0000_1800,
        y_acceleration: -0x0000_1800,
    },
    TornadoPattern {
        target_x_velocity: 0x0001_8000,
        target_y_velocity: 0x0001_8000,
        x_acceleration: 0x0000_1800,
        y_acceleration: 0x0000_1800,
    },
    TornadoPattern {
        target_x_velocity: -0x0001_8000,
        target_y_velocity: -0x0001_8000,
        x_acceleration: -0x0000_1800,
        y_acceleration: -0x0000_1800,
    },
    TornadoPattern {
        target_x_velocity: -0x0001_8000,
        target_y_velocity: 0x0001_8000,
        x_acceleration: -0x0000_1800,
        y_acceleration: 0x0000_1800,
    },
    TornadoPattern {
        target_x_velocity: -0x0001_8000,
        target_y_velocity: -0x0001_8000,
        x_acceleration: -0x0000_1800,
        y_acceleration: -0x0000_1800,
    },
    TornadoPattern {
        target_x_velocity: -0x0001_8000,
        target_y_velocity: 0x0001_8000,
        x_acceleration: -0x0000_1800,
        y_acceleration: 0x0000_1800,
    },
    TornadoPattern {
        target_x_velocity: -0x0001_8000,
        target_y_velocity: -0x0001_8000,
        x_acceleration: -0x0000_1800,
        y_acceleration: -0x0000_1800,
    },
    TornadoPattern {
        target_x_velocity: -0x0001_8000,
        target_y_velocity: 0x0001_8000,
        x_acceleration: -0x0000_1800,
        y_acceleration: 0x0000_1800,
    },
    TornadoPattern {
        target_x_velocity: -0x0001_8000,
        target_y_velocity: -0x0001_8000,
        x_acceleration: -0x0000_1800,
        y_acceleration: -0x0000_1800,
    },
    TornadoPattern {
        target_x_velocity: -0x0001_8000,
        target_y_velocity: 0x0001_8000,
        x_acceleration: -0x0000_1800,
        y_acceleration: 0x0000_1800,
    },
];

fn acceleration_for_target(
    current_velocity: i32,
    target_velocity: i32,
    base_acceleration: i32,
) -> i32 {
    if base_acceleration == 0 {
        // ASM UFO.asm loc_35452/loc_35496: zero accel entry means `$C00` toward zero.
        return if current_velocity < 0 {
            TORNADO_ACCELERATION
        } else {
            -TORNADO_ACCELERATION
        };
    }

    let mut acceleration = base_acceleration;
    if current_velocity >= 0 {
        // ASM UFO.asm loc_3546E/loc_354B0: non-negative velocity keeps acceleration unless a positive
        // target speed is already below the current velocity, where the acceleration register is negated.
        if target_velocity >= 0 && target_velocity < current_velocity {
            acceleration = -acceleration;
        }
    } else {
        // ASM UFO.asm loc_35488/loc_354CA: negative velocity keeps acceleration for non-negative targets;
        // only negate it when a negative target is above the current velocity.
        if target_velocity < 0 && target_velocity > current_velocity {
            acceleration = -acceleration;
        }
    }
    acceleration
}

fn apply_acceleration_axis(current_velocity: i32, target_velocity: i32, acceleration: i32) -> i32 {
    if acceleration >= 0 {
        ufo_apply_acceleration_axis_loc_3502a(current_velocity, target_velocity, acceleration)
    } else {
        ufo_apply_acceleration_axis_loc_3501e(current_velocity, target_velocity, acceleration)
    }
}

#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn ufo_apply_acceleration_axis_loc_3502a(
    current_velocity: i32,
    target_velocity: i32,
    acceleration: i32,
) -> i32 {
    // ASM UFO.asm loc_3502A: add positive acceleration and cap to desired axis speed.
    let next_velocity = current_velocity.saturating_add(acceleration);
    ufo_apply_acceleration_axis_loc_35050(next_velocity.min(target_velocity))
}

#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn ufo_apply_acceleration_axis_loc_3501e(
    current_velocity: i32,
    target_velocity: i32,
    acceleration: i32,
) -> i32 {
    // ASM UFO.asm loc_3501E: negative-branch accelerant path in
    // `Enemy_HandleAcceleration` follows `loc_35044`.
    ufo_apply_acceleration_axis_loc_35044(current_velocity, target_velocity, acceleration)
}

#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn ufo_apply_acceleration_axis_loc_35044(
    current_velocity: i32,
    target_velocity: i32,
    acceleration: i32,
) -> i32 {
    // ASM UFO.asm loc_35044: keep the accelerated value while it stays above the target.
    let next_velocity = current_velocity.saturating_add(acceleration);
    if next_velocity > target_velocity {
        ufo_apply_acceleration_axis_loc_35050(next_velocity)
    } else {
        ufo_apply_acceleration_axis_loc_35050(target_velocity)
    }
}

#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn ufo_apply_acceleration_axis_loc_35050(final_velocity: i32) -> i32 {
    // ASM UFO.asm loc_35050: write the chosen `d7` into Y velocity and return.
    final_velocity
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) enum UfoCollisionBehaviorLoc35056 {
    BounceWall,
    BounceFloorCeil,
    BounceSlopeUp,
    BounceSlopeDown,
    Hurt,
    ReturnOnly,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) struct UfoCollisionDispatchLoc35056 {
    pub(crate) collision_type: u16,
    pub(crate) behavior: Option<UfoCollisionBehaviorLoc35056>,
}

#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn ufo_collision_dispatch_loc_35056(
    collision_type: i16,
) -> UfoCollisionDispatchLoc35056 {
    // ASM UFO.asm loc_35056: negative values skip to `loc_35070` (clear only),
    // zero returns directly, otherwise table-dispatch uses `collision_type-4` and
    // `move.l (a4,d7.w),a4`.
    if collision_type < 0 {
        return UfoCollisionDispatchLoc35056 {
            collision_type: 0,
            behavior: None,
        };
    }

    if collision_type == 0 {
        return UfoCollisionDispatchLoc35056 {
            collision_type: 0,
            behavior: None,
        };
    }

    let behavior = match collision_type as u16 {
        0x0004 | 0x0008 => Some(UfoCollisionBehaviorLoc35056::BounceWall),
        0x000C | 0x0010 => Some(UfoCollisionBehaviorLoc35056::BounceFloorCeil),
        0x0014 => Some(UfoCollisionBehaviorLoc35056::BounceSlopeUp),
        0x0018 => Some(UfoCollisionBehaviorLoc35056::BounceSlopeDown),
        0x001C | 0x002C => Some(UfoCollisionBehaviorLoc35056::Hurt),
        0x0020 | 0x0024 | 0x0028 => Some(UfoCollisionBehaviorLoc35056::ReturnOnly),
        _ => None,
    };

    UfoCollisionDispatchLoc35056 {
        collision_type: 0,
        behavior,
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) struct UfoCollisionClearLoc35070 {
    pub(crate) collision_type: u16,
}

#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn ufo_collision_clear_loc_35070(collision_type: u16) -> UfoCollisionClearLoc35070 {
    // ASM UFO.asm loc_35070: clear collision type and return.
    let _ = collision_type;
    UfoCollisionClearLoc35070 { collision_type: 0 }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) struct UfoPositionRestoreLoc350F2 {
    pub(crate) x_pos: i32,
    pub(crate) y_pos: i32,
}

#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn ufo_restore_position_loc_350f2(
    x_pos: i32,
    y_pos: i32,
    x_vel: i32,
    y_vel: i32,
) -> UfoPositionRestoreLoc350F2 {
    // ASM UFO.asm loc_350F2: rollback one frame by subtracting the per-frame
    // velocity delta from object position.
    UfoPositionRestoreLoc350F2 {
        x_pos: x_pos.wrapping_sub(x_vel),
        y_pos: y_pos.wrapping_sub(y_vel),
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) struct UfoWallBounceLoc35120 {
    pub(crate) x_velocity: i32,
    pub(crate) d0: i32,
}

#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn ufo_bounce_wall_tail_loc_35120(x_velocity: i32, d0: i32) -> UfoWallBounceLoc35120 {
    // ASM UFO.asm loc_35120: after the wall bounce has negated `x_vel`, keep it.
    // Clear-only behavior means positive `d0` values are negated so they can be
    // used as a bounce impulse; negative `d0` passes through unchanged.
    let x_velocity = x_velocity.wrapping_neg();
    let d0 = if d0 >= 0 { d0.wrapping_neg() } else { d0 };

    UfoWallBounceLoc35120 { x_velocity, d0 }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) struct UfoFloorCeilBounceLoc35146 {
    pub(crate) y_velocity: i32,
    pub(crate) d2: i32,
}

#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn ufo_bounce_floor_ceil_tail_loc_35146(
    y_velocity: i32,
    d2: i32,
) -> UfoFloorCeilBounceLoc35146 {
    // ASM UFO.asm loc_35146: after the floor/ceiling bounce has negated `y_vel`,
    // keep it and negate only non-negative `d2` values as a secondary impulse.
    let y_velocity = y_velocity.wrapping_neg();
    let d2 = if d2 >= 0 { d2.wrapping_neg() } else { d2 };

    UfoFloorCeilBounceLoc35146 { y_velocity, d2 }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) struct UfoHurtSpawnTailLoc35174 {
    pub(crate) some_ufo_shooting_cleared: bool,
    pub(crate) allocation_selector: u16,
    pub(crate) driver_routine: &'static str,
    pub(crate) driver_x_velocity: i32,
    pub(crate) driver_timer_flag: i16,
    pub(crate) parent_moved: bool,
    pub(crate) parent_y_velocity: i32,
    pub(crate) driver_animation: &'static str,
    pub(crate) parent_animation_flag_13_cleared: bool,
}

#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn ufo_hurt_tail_loc_35174(
    parent_x_velocity: i32,
    some_ufo_shooting: bool,
) -> UfoHurtSpawnTailLoc35174 {
    // ASM UFO.asm loc_35174: clear `$46(a5)`/`Some_UFO_Shooting` only when set,
    // allocate a driver with selector `$6000`, install `loc_355E2`, seed `$44` with
    // parent `x_vel`, set a timer word to `-1`, clear child `$13`, set
    // `is_moved(a3)`, force parent `y_vel` to `$30000`, then run `stru_34CA0`.
    UfoHurtSpawnTailLoc35174 {
        some_ufo_shooting_cleared: some_ufo_shooting,
        allocation_selector: 0x6000,
        driver_routine: "loc_355E2",
        driver_x_velocity: parent_x_velocity,
        driver_timer_flag: -1,
        parent_moved: true,
        parent_y_velocity: 0x0003_0000,
        driver_animation: "stru_34CA0",
        parent_animation_flag_13_cleared: true,
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) struct UfoHurtDriverSpawnLoc35180 {
    pub(crate) allocation_selector: u16,
    pub(crate) driver_routine: &'static str,
    pub(crate) driver_flag_46: i16,
    pub(crate) driver_x_velocity: i32,
    pub(crate) parent_moved: bool,
    pub(crate) parent_y_velocity: i32,
}

#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn ufo_hurt_driver_spawn_loc_35180(
    parent_x_velocity: i32,
) -> UfoHurtDriverSpawnLoc35180 {
    // ASM UFO.asm loc_35180: allocate `$6000` object slot and install `loc_355E2`,
    // set `$46(a0)=$FFFF`, copy parent `x_vel(a3)` into child `current_hp` (word write),
    // clear parent `$13(a1)`, set is_moved(parent), and force parent `$2A(a3)=$30000`.
    UfoHurtDriverSpawnLoc35180 {
        allocation_selector: 0x6000,
        driver_routine: "loc_355E2",
        driver_flag_46: -1,
        driver_x_velocity: parent_x_velocity,
        parent_moved: true,
        parent_y_velocity: 0x0003_0000,
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) struct UfoHurtHibernateGateLoc351B6 {
    pub(crate) keep_hibernating: bool,
    pub(crate) delete_current_object: bool,
}

#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn ufo_hurt_hibernate_gate_loc_351b6(
    a3_flag_19_set: bool,
) -> UfoHurtHibernateGateLoc351B6 {
    // ASM UFO.asm loc_351B6: after one-frame hibernate, stay in this routine
    // while `$19(a3)` is zero; delete the object when the byte is nonzero.
    UfoHurtHibernateGateLoc351B6 {
        keep_hibernating: !a3_flag_19_set,
        delete_current_object: a3_flag_19_set,
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) struct UfoBeamSpawnLoc351EE {
    pub(crate) allocation_selector: u16,
    pub(crate) beam_routine: &'static str,
    pub(crate) beam_x_velocity: i32,
    pub(crate) beam_y_velocity: i32,
}

#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn ufo_beam_segment_spawn_loc_351f2(
    parent_x_velocity: i32,
    raw_x_velocity: i16,
    raw_y_velocity: i16,
) -> Option<UfoBeamSpawnLoc351EE> {
    // ASM UFO.asm loc_351F2: allocate selector `$8000`, install `loc_354DA`,
    // stop on a zero x-word, then convert table words with `swap; asr.l #1`.
    if raw_x_velocity == 0 {
        return None;
    }

    Some(UfoBeamSpawnLoc351EE {
        allocation_selector: 0x8000,
        beam_routine: "loc_354DA",
        beam_x_velocity: (i32::from(raw_x_velocity) << 15).wrapping_add(parent_x_velocity >> 1),
        beam_y_velocity: i32::from(raw_y_velocity) << 15,
    })
}

#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn ufo_beam_spawn_loc_351ee(
    parent_x_velocity: i32,
    beam_velocity_pairs: &[(i16, i16)],
) -> Vec<UfoBeamSpawnLoc351EE> {
    // ASM UFO.asm loc_351EE: load `unk_351C4` and repeatedly allocate
    // `loc_354DA` beams from paired table entries until the first x-word sentinel 0.
    let mut spawns = Vec::new();
    for (raw_x_vel, raw_y_vel) in beam_velocity_pairs {
        let Some(spawn) =
            ufo_beam_segment_spawn_loc_351f2(parent_x_velocity, *raw_x_vel, *raw_y_vel)
        else {
            break;
        };
        spawns.push(spawn);
    }

    spawns
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) struct UfoBeamFinishLoc35228 {
    pub(crate) parent_beam_spawned_latch: bool,
    pub(crate) linked_object_meta: u16,
    pub(crate) linked_object_animation: &'static str,
    pub(crate) linked_object_flag_13: bool,
    pub(crate) linked_object_flag_19: bool,
    pub(crate) linked_object_is_moved: bool,
    pub(crate) linked_object_level_collision: bool,
    pub(crate) linked_object_routine: u8,
    pub(crate) parent_animation: &'static str,
    pub(crate) parent_y_velocity: i32,
    pub(crate) d2: i32,
    pub(crate) d3: i32,
}

#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn ufo_beam_finish_loc_35228(parent_y_velocity: i32) -> UfoBeamFinishLoc35228 {
    // ASM UFO.asm loc_35228: mark `$47(a5)`, link the loc_32188 gfx object,
    // initialise it as UFO object meta `$0F`, reset its state flags, replay the
    // parent `stru_34C92` animation, then bias parent y velocity downward.
    UfoBeamFinishLoc35228 {
        parent_beam_spawned_latch: true,
        linked_object_meta: 0x000F,
        linked_object_animation: "stru_34D02",
        linked_object_flag_13: true,
        linked_object_flag_19: false,
        linked_object_is_moved: false,
        linked_object_level_collision: false,
        linked_object_routine: 1,
        parent_animation: "stru_34C92",
        parent_y_velocity: parent_y_velocity.wrapping_add(0x0004_0000),
        d2: -0x0000_0C00,
        d3: 0,
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) enum UfoLinkedObjectRangeGateLoc35280 {
    Return,
    RunCleanupDeleteTail,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) struct UfoLinkedCleanupCountLoc3528A {
    pub(crate) number_ufos_on_screen: u16,
    pub(crate) play_hover_sound: bool,
}

#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn ufo_linked_cleanup_count_loc_3528a(
    number_ufos_on_screen: u16,
    ufo_on_screen_latch: bool,
) -> UfoLinkedCleanupCountLoc3528A {
    // ASM UFO.asm loc_3528A: if `$50(a5)` is clear, skip the global
    // `Number_UFOs_OnScreen` decrement; otherwise decrement and replay
    // `sfx_UFO_hovering` only when the word reaches zero.
    if !ufo_on_screen_latch {
        return UfoLinkedCleanupCountLoc3528A {
            number_ufos_on_screen,
            play_hover_sound: false,
        };
    }

    let number_ufos_on_screen = number_ufos_on_screen.wrapping_sub(1);
    UfoLinkedCleanupCountLoc3528A {
        number_ufos_on_screen,
        play_hover_sound: number_ufos_on_screen == 0,
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) struct UfoLinkedCleanupShootingLoc352A6 {
    pub(crate) global_shooting_flag: bool,
}

#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn ufo_linked_cleanup_shooting_loc_352a6(
    ufo_shooting_latch: bool,
    global_shooting_flag: bool,
) -> UfoLinkedCleanupShootingLoc352A6 {
    // ASM UFO.asm loc_352A6: test `$46(a5)` and clear `Some_UFO_Shooting`
    // only when this UFO's shooting latch is set.
    UfoLinkedCleanupShootingLoc352A6 {
        global_shooting_flag: if ufo_shooting_latch {
            false
        } else {
            global_shooting_flag
        },
    }
}

#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn ufo_linked_cleanup_status_loc_352b2(
    object_status_slot: u8,
) -> DeactivationStatusActionLoc3C4C0 {
    // ASM UFO.asm loc_352B2: read `$42(a5)` into d0, update the corresponding
    // `EnemyStatus_Table` slot for positive or bit-6 sentinel values, then
    // delete the current object.
    // ASM UFO.asm loc_352D2: positive slots write `$2168` as the respawn marker.
    // ASM UFO.asm loc_352E0: every branch reaches `j_Delete_CurrentObject`.
    deactivation_status_update_loc_3C4C0(object_status_slot)
}

#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn ufo_linked_object_x_loc_3531a(parent_x_pos: u32, parent_facing: Direction) -> u16 {
    // ASM UFO.asm loc_3531A: after `Enemy0F_UFO_SpeedToPos`, the linked object
    // gets the parent's integer X word minus d6. Clear `x_direction` leaves
    // d6 as +4; set `x_direction` negates d6 to -4, so the subtraction adds 4.
    let parent_x_word = (parent_x_pos >> 16) as u16;
    let d6 = match parent_facing {
        Direction::Left => 4_u16,
        Direction::Right => 0xFFFC_u16,
    };
    parent_x_word.wrapping_sub(d6)
}

#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn ufo_linked_object_range_gate_loc_35280(
    range_check_result_nonzero: bool,
) -> UfoLinkedObjectRangeGateLoc35280 {
    // ASM UFO.asm loc_35280: after the range-check subroutine, a zero result returns
    // immediately; any non-zero result falls through into the cleanup/delete tail.
    if range_check_result_nonzero {
        UfoLinkedObjectRangeGateLoc35280::RunCleanupDeleteTail
    } else {
        UfoLinkedObjectRangeGateLoc35280::Return
    }
}

pub(crate) fn reload_tornado_pattern_script(enemy: &mut ActiveEnemy) {
    // ASM Tornado.asm loc_359F2 / unk_35A0C: advance through the 16-entry
    // motion script and wrap when the following longword sentinel equals 1.
    let index = usize::from(enemy.dragon_script_cmd) % TORNADO_PATTERN_SCRIPT.len();
    let pattern = TORNADO_PATTERN_SCRIPT[index];
    enemy.dragon_script_cmd = ((index + 1) % TORNADO_PATTERN_SCRIPT.len()) as u8;
    let target_x_velocity = if enemy.flight_double_terminal_speed {
        pattern.target_x_velocity.saturating_mul(2)
    } else {
        pattern.target_x_velocity
    };
    let target_y_velocity = if enemy.flight_double_terminal_speed {
        pattern.target_y_velocity.saturating_mul(2)
    } else {
        pattern.target_y_velocity
    };
    let x_acceleration =
        acceleration_for_target(enemy.x_velocity, target_x_velocity, pattern.x_acceleration);
    let y_acceleration =
        acceleration_for_target(enemy.y_velocity, target_y_velocity, pattern.y_acceleration);
    enemy.target_x_velocity = target_x_velocity;
    enemy.target_y_velocity = target_y_velocity;
    enemy.x_acceleration = if enemy.flight_double_acceleration {
        x_acceleration.saturating_mul(2)
    } else {
        x_acceleration
    };
    enemy.y_acceleration = if enemy.flight_double_acceleration {
        y_acceleration.saturating_mul(2)
    } else {
        y_acceleration
    };
    enemy.facing = if target_x_velocity < 0 {
        Direction::Left
    } else {
        Direction::Right
    };
}

#[cfg(test)]
pub(crate) fn step_tornado(enemy: &mut ActiveEnemy, block_grid: Option<&terrain::BlockGrid>) {
    let mut rng = RomRandomState::default();
    step_tornado_with_rng(enemy, block_grid, &mut rng);
}

pub(crate) fn step_tornado_with_rng(
    enemy: &mut ActiveEnemy,
    block_grid: Option<&terrain::BlockGrid>,
    _rng: &mut RomRandomState,
) {
    // ASM UFO.asm Enemy_HandleAcceleration: move toward desired axis speeds.
    enemy.x_velocity = apply_acceleration_axis(
        enemy.x_velocity,
        enemy.target_x_velocity,
        enemy.x_acceleration,
    );
    enemy.y_velocity = apply_acceleration_axis(
        enemy.y_velocity,
        enemy.target_y_velocity,
        enemy.y_acceleration,
    );

    // Advance position
    enemy.x_pos = (enemy.x_pos as i64 + i64::from(enemy.x_velocity)) as u32;
    enemy.y_pos = (enemy.y_pos as i64 + i64::from(enemy.y_velocity)) as u32;

    // ASM Tornado.asm loc_359F2 / unk_3588C: reload the next script entry on
    // each sub-cycle and reset the per-level countdown.
    if enemy.wait_timer == 0 {
        let level = usize::from(enemy.enemy_level).min(2);
        enemy.wait_timer = TORNADO_PATTERN_FRAMES[level];
        reload_tornado_pattern_script(enemy);
    } else {
        enemy.wait_timer -= 1;
    }

    // Wall and floor reversal
    if let Some(grid) = block_grid {
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
            enemy.x_velocity = -enemy.x_velocity;
            enemy.x_acceleration = -enemy.x_acceleration;
            enemy.facing = opposite_direction(enemy.facing);
        }
        let floor = terrain::probe_floor(grid, cx, cy);
        if floor.on_ground && enemy.y_velocity > 0 {
            enemy.y_velocity = -enemy.y_velocity;
            enemy.y_acceleration = -enemy.y_acceleration;
        }
    }
}

/// Sphere bounces off walls/floors/ceilings with gravity.
/// Sphere.asm: gravity +$800/frame, wall→negate x_vel, floor→negate y_vel,
/// ceiling→negate y_vel and halve.
pub(crate) const SPHERE_GRAVITY: i32 = 0x800;

/// Fireball max upward velocity indexed by live HP
/// (Fireball.asm `Enemy19_Fireball_Init`: `move.w 2(a0),d7; asl.w #3,d7` into `unk_32BA6`).
/// HP 1/2/3 use the second/third/final table entries because spawn HP already includes the +1 base.
pub(crate) const FIREBALL_UPWARD_CAP_BY_HP: [i32; 3] = [-0x4_0000, -0x5_0000, -0x5_0000];

#[cfg(test)]
pub(crate) fn fireball_bounce_perturbation(enemy: &ActiveEnemy) -> i32 {
    let mut rng = RomRandomState::default();
    let camera_x_px = enemy.x_pos >> 16;
    fireball_bounce_perturbation_with_rng(&mut rng, camera_x_px)
}

pub(crate) fn fireball_bounce_perturbation_with_rng(
    rng: &mut RomRandomState,
    camera_x_px: u32,
) -> i32 {
    // ASM Enemy19_Fireball_BounceFloorCeiling: `Get_RandomNumber_byte`,
    // `eor.b (Camera_X_pos).w,d7`, `ext.w d7`, `asl.w #7,d7`.
    let random_byte = rng.next_byte();
    let camera_x_low_byte = (camera_x_px & 0xFF) as u8;
    i32::from((random_byte ^ camera_x_low_byte) as i8) << 7
}

#[cfg(test)]
pub(crate) fn fireball_bounce_y_velocity(enemy: &ActiveEnemy) -> i32 {
    let mut rng = RomRandomState::default();
    let camera_x_px = enemy.x_pos >> 16;
    fireball_bounce_y_velocity_with_rng(enemy, &mut rng, camera_x_px)
}

pub(crate) fn fireball_bounce_y_velocity_with_rng(
    enemy: &ActiveEnemy,
    rng: &mut RomRandomState,
    camera_x_px: u32,
) -> i32 {
    let perturbed = enemy
        .y_velocity
        .saturating_add(fireball_bounce_perturbation_with_rng(rng, camera_x_px));
    let mut rebound = perturbed.saturating_neg();
    if rebound < 0 {
        let hp_index = enemy.hit_points.saturating_sub(1).min(2) as usize;
        let cap = FIREBALL_UPWARD_CAP_BY_HP[hp_index];
        if rebound < cap {
            rebound = cap;
        }
    }
    rebound
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct FireballBounceCommitLoc32CE4 {
    pub(crate) y_velocity: i32,
    pub(crate) spawns_fire_trail: bool,
    pub(crate) next_fire_trail_count: u16,
}

#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn fireball_floor_ceiling_bounce_loc_32ce4(
    enemy: &ActiveEnemy,
    rng: &mut RomRandomState,
    camera_x_px: u32,
    fire_trail_count: u16,
) -> FireballBounceCommitLoc32CE4 {
    // ASM Fireball.asm loc_32CE4: commit d7 into `y_vel(a3)`, then allocate
    // `loc_33128` only while `Number_of_Fire_Trails` remains below five.
    let y_velocity = fireball_bounce_y_velocity_with_rng(enemy, rng, camera_x_px);
    let spawns_fire_trail = fire_trail_count < 5;
    FireballBounceCommitLoc32CE4 {
        y_velocity,
        spawns_fire_trail,
        next_fire_trail_count: if spawns_fire_trail {
            fire_trail_count.saturating_add(1)
        } else {
            fire_trail_count
        },
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct FireballSlopeTrailMarkerLoc32D7E {
    pub(crate) x_px: i16,
    pub(crate) y_px: i16,
}

#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn fireball_slope_up_trail_marker_loc_32d7e(
    probe_x_px: i16,
    probe_y_px: i16,
    tile_word: u16,
) -> Option<FireballSlopeTrailMarkerLoc32D7E> {
    // ASM Fireball.asm loc_32D7E: read the tile at d7/d6, require collision
    // nibble `$4000`, then align d6 to the tile row and subtract d7's low nibble.
    if tile_word & 0x7000 != 0x4000 {
        return None;
    }

    let x_fraction = probe_x_px & 0x000F_i16;
    let aligned_y = probe_y_px & !0x000F_i16;
    Some(FireballSlopeTrailMarkerLoc32D7E {
        x_px: probe_x_px.wrapping_add(2),
        y_px: aligned_y
            .wrapping_add(0x000F)
            .wrapping_sub(x_fraction)
            .wrapping_sub(2),
    })
}

pub(crate) fn bouncer_gravity(enemy: &ActiveEnemy) -> i32 {
    match enemy.family {
        // Fireball.asm `Enemy19_Fireball_Init` indexes `unk_32BA6` with the raw
        // layout `hit_points_plus_one`, so live HP 1/2/3 maps to $2000/$3000/$3000.
        EnemyFamily::Fireball => {
            const FIREBALL_GRAVITY_BY_HP: [i32; 3] = [0x2000, 0x3000, 0x3000];
            let hp_index = enemy.hit_points.saturating_sub(1) as usize;
            FIREBALL_GRAVITY_BY_HP[hp_index.min(2)]
        }
        _ => SPHERE_GRAVITY,
    }
}

pub(crate) fn fireball_main_loop_y_velocity_loc_32c1a(enemy: &ActiveEnemy) -> i32 {
    // ASM Fireball.asm loc_32C1A: after the one-frame hibernate and
    // `Object_CheckInRange`, add d0 gravity into `y_vel(a3)` before `loc_32C60`.
    enemy.y_velocity.saturating_add(bouncer_gravity(enemy))
}

/// Big Hopping Skull: continuous bouncer from Big_Hopping_Skull.asm.
/// Gravity $FA0/frame, deflects off all surfaces. No ground delay.
/// Floor: negate y_vel. Wall: backup 2x, negate x_vel, clear y_vel.
/// Ceiling: negate y_vel. Slope up: swap+negate both. Slope down: swap only.
pub(crate) fn step_big_hopping_skull(
    enemy: &mut ActiveEnemy,
    block_grid: Option<&terrain::BlockGrid>,
) {
    // Apply gravity (ASM loc_36540: addi.l #$FA0,y_vel)
    enemy.y_velocity = enemy.y_velocity.saturating_add(BIG_SKULL_GRAVITY);

    // Advance position
    enemy.x_pos = (enemy.x_pos as i64 + i64::from(enemy.x_velocity)) as u32;
    enemy.y_pos = (enemy.y_pos as i64 + i64::from(enemy.y_velocity)) as u32;

    if let Some(grid) = block_grid {
        let cx = (enemy.x_pos >> 16) as i32;
        let cy = (enemy.y_pos >> 16) as i32;

        // Wall collision (ASM loc_366EC: backup 2x, negate x_vel, clear y_vel)
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
            // Backup position by 2x x_velocity
            let backup = 2 * i64::from(enemy.x_velocity);
            enemy.x_pos = (enemy.x_pos as i64 - backup).max(0) as u32;
            enemy.y_velocity = 0;
            enemy.x_velocity = -enemy.x_velocity;
            enemy.facing = opposite_direction(enemy.facing);
        }

        // Floor collision (ASM loc_36580: triggers kid-hit state, but for physics
        // the restart at loc_366D0 resets y_vel to $FFFE0000. We negate as bounce.)
        let floor = terrain::probe_floor(grid, cx, cy);
        if floor.on_ground && enemy.y_velocity > 0 {
            let floor_y_fp = (floor.floor_y_px as u32) << 16;
            if enemy.y_pos >= floor_y_fp {
                enemy.y_pos = floor_y_fp;
                // Floor bounce: negate y_velocity
                enemy.y_velocity = -enemy.y_velocity;
            }

            // Slope deflection: check if floor cell is a slope
            if floor.cell.collision == terrain::CollisionKind::SlopeUp {
                // ASM loc_36706: swap and negate both
                let old_x = enemy.x_velocity;
                let old_y = enemy.y_velocity;
                enemy.y_velocity = -old_x;
                enemy.x_velocity = -old_y;
                enemy.facing = opposite_direction(enemy.facing);
            } else if floor.cell.collision == terrain::CollisionKind::SlopeDown {
                // ASM loc_36720: swap without negate
                let old_x = enemy.x_velocity;
                let old_y = enemy.y_velocity;
                enemy.y_velocity = old_x;
                enemy.x_velocity = old_y;
                enemy.facing = opposite_direction(enemy.facing);
            }
        }

        // Ceiling bounce (ASM loc_366E6: neg.l y_vel)
        let ceil = terrain::probe_ceiling(grid, cx, cy, ENEMY_PROBE_HALF_WIDTH, ENEMY_PROBE_HEIGHT);
        if ceil.hit && enemy.y_velocity < 0 {
            enemy.y_velocity = -enemy.y_velocity;
        }
    }
}

#[cfg(test)]
pub(crate) fn step_bouncer(enemy: &mut ActiveEnemy, block_grid: Option<&terrain::BlockGrid>) {
    let mut rng = RomRandomState::default();
    let camera_x_px = enemy.x_pos >> 16;
    step_bouncer_with_rng(enemy, block_grid, Some(camera_x_px), &mut rng);
}

pub(crate) fn step_bouncer_with_rng(
    enemy: &mut ActiveEnemy,
    block_grid: Option<&terrain::BlockGrid>,
    camera_x_px: Option<u32>,
    rng: &mut RomRandomState,
) {
    // Apply gravity
    enemy.y_velocity = if matches!(enemy.family, EnemyFamily::Fireball) {
        fireball_main_loop_y_velocity_loc_32c1a(enemy)
    } else {
        enemy.y_velocity.saturating_add(bouncer_gravity(enemy))
    };

    // Advance position
    enemy.x_pos = (enemy.x_pos as i64 + i64::from(enemy.x_velocity)) as u32;
    enemy.y_pos = (enemy.y_pos as i64 + i64::from(enemy.y_velocity)) as u32;

    if let Some(grid) = block_grid {
        let cx = (enemy.x_pos >> 16) as i32;
        let cy = (enemy.y_pos >> 16) as i32;

        // Wall bounce
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
            enemy.x_velocity = -enemy.x_velocity;
            if !matches!(enemy.family, EnemyFamily::Fireball | EnemyFamily::Sphere) {
                // ASM Sphere.asm loc_36148 negates `x_vel(a3)` and immediately
                // resumes the loop. The sphere family never flips x_direction on
                // wall contact, so keep facing unchanged here.
                enemy.facing = opposite_direction(enemy.facing);
            }
        }

        // Floor bounce
        let floor = terrain::probe_floor(grid, cx, cy);
        if floor.on_ground && enemy.y_velocity > 0 {
            // Check for slope deflection first (ASM Sphere.asm loc_36012/loc_36032,
            // Fireball.asm loc_32D06/loc_32D46, Big_Hopping_Skull.asm loc_36706/loc_36720).
            // Slopes take precedence over floor bounce.
            if floor.cell.collision == terrain::CollisionKind::SlopeUp {
                let old_x = enemy.x_velocity;
                let old_y = enemy.y_velocity;
                if matches!(enemy.family, EnemyFamily::Sphere) {
                    // ASM Sphere.asm loc_3614E mirrors Enemy06_Sphere_SlopeUp:
                    // negate both source velocities, back the object out by one
                    // frame along that vector, then swap into x/y velocity.
                    enemy.x_pos = (enemy.x_pos as i64 - i64::from(old_x)).max(0) as u32;
                    enemy.y_pos = (enemy.y_pos as i64 - i64::from(old_y)).max(0) as u32;
                    enemy.y_velocity = -old_x;
                    enemy.x_velocity = -old_y;
                } else {
                    // ASM: swap velocities, negate both
                    enemy.y_velocity = -old_x;
                    enemy.x_velocity = -old_y;
                    enemy.facing = opposite_direction(enemy.facing);
                }
            } else if floor.cell.collision == terrain::CollisionKind::SlopeDown {
                let old_x = enemy.x_velocity;
                let old_y = enemy.y_velocity;
                if matches!(enemy.family, EnemyFamily::Sphere) {
                    // ASM Sphere.asm loc_36164 mirrors Enemy06_Sphere_SlopeDown:
                    // back out by the pre-collision motion, then swap the live
                    // x/y velocities without changing facing.
                    enemy.x_pos = (enemy.x_pos as i64 - i64::from(old_x)).max(0) as u32;
                    enemy.y_pos = (enemy.y_pos as i64 - i64::from(old_y)).max(0) as u32;
                    enemy.y_velocity = old_x;
                    enemy.x_velocity = old_y;
                } else {
                    // ASM: swap velocities without negation
                    enemy.y_velocity = old_x;
                    enemy.x_velocity = old_y;
                    enemy.facing = opposite_direction(enemy.facing);
                }
            } else {
                // Regular floor bounce
                if matches!(enemy.family, EnemyFamily::Fireball) {
                    enemy.y_velocity =
                        fireball_bounce_y_velocity_with_rng(enemy, rng, camera_x_px.unwrap_or(0));
                } else {
                    enemy.y_velocity = -enemy.y_velocity;
                }
                // Snap out of floor
                let floor_y_fp = (floor.floor_y_px as u32) << 16;
                if enemy.y_pos > floor_y_fp {
                    enemy.y_pos = floor_y_fp;
                }
            }
        }

        // Ceiling bounce
        let ceil = terrain::probe_ceiling(grid, cx, cy, ENEMY_PROBE_HALF_WIDTH, ENEMY_PROBE_HEIGHT);
        if ceil.hit && enemy.y_velocity < 0 {
            if matches!(enemy.family, EnemyFamily::Fireball) {
                // ASM Enemy19_Fireball_BounceFloorCeiling reuses the same
                // perturb-and-negate path for both floor and ceiling contact.
                enemy.y_velocity =
                    fireball_bounce_y_velocity_with_rng(enemy, rng, camera_x_px.unwrap_or(0));
            } else {
                // Sphere.asm: negate and halve
                enemy.y_velocity = -(enemy.y_velocity >> 1);
            }
        }
    }
}

/// Dragon Flying script interpreter (Dragon_Flying.asm).
/// Reads commands from one of 3 variant scripts, executing each per-frame.
/// Commands: Hover (idle), Drift (accelerate), DriftWithFire (drift + cone check),
/// FacePlayer (retarget), Descent (dive sub-script), ReturnToAnchor (rise to spawn Y).
pub(crate) fn step_dragon(
    enemy: &mut ActiveEnemy,
    block_grid: Option<&terrain::BlockGrid>,
    player_x_pos: Option<u32>,
    player_y_pos: Option<u32>,
    projectiles: &mut Option<&mut Vec<EnemyProjectile>>,
) {
    match enemy.dragon_phase {
        DragonFlyingState::None => {}
        DragonFlyingState::Hovering => {
            // Hover: zero velocity, count down frames. ASM loc_34804 clears
            // x_vel; Dragon_Flying.asm loc_3480A then hibernates one frame
            // before checking collision_type and continuing the hover loop.
            enemy.x_velocity = 0;
            enemy.y_velocity = 0;
            enemy.dragon_phase_frames = enemy.dragon_phase_frames.saturating_sub(1);
            if enemy.dragon_phase_frames == 0 {
                dragon_advance_script(enemy, player_x_pos, projectiles);
            }
            dragon_apply_velocity(enemy);
        }
        DragonFlyingState::Drifting | DragonFlyingState::DriftingWithFire => {
            // Drift: accelerate x_velocity toward cap, count down frames.
            // ASM loc_3478C: add/sub $200 per frame, cap at ±$18000.
            if enemy.x_velocity > 0 {
                enemy.x_velocity =
                    (enemy.x_velocity + DRAGON_DRIFT_ACCEL).min(DRAGON_DRIFT_SPEED_CAP);
            } else if enemy.x_velocity < 0 {
                enemy.x_velocity = dragon_drift_negative_velocity_loc_347b2(enemy.x_velocity);
            }
            let d1_fire_gate = if enemy.dragon_phase == DragonFlyingState::DriftingWithFire {
                1
            } else {
                0
            };
            let drift_tail = dragon_drift_store_loc_347c8(enemy.x_velocity, d1_fire_gate);
            enemy.x_velocity = drift_tail.x_velocity;
            // Fire-breath cone check during DriftWithFire (ASM loc_34836).
            if drift_tail.checks_fire_breath_cone
                && dragon_fire_breath_cone(enemy, player_x_pos, player_y_pos)
            {
                // Cone hit: enter descent attack.
                dragon_enter_descent(enemy);
                dragon_apply_velocity(enemy);
                return;
            }
            enemy.dragon_phase_frames = enemy.dragon_phase_frames.saturating_sub(1);
            if enemy.dragon_phase_frames == 0 {
                // ASM Dragon_Flying.asm loc_347FC falls through to loc_346DA
                // when the drift loop counter expires; native stores the
                // remaining loop count directly in `dragon_phase_frames`.
                dragon_advance_script(enemy, player_x_pos, projectiles);
            }
            dragon_apply_velocity(enemy);
            // Wall collision during Drift: reverse direction on terrain hit.
            // ASM Dragon_Flying.asm loc_347D6: collision_type check after position update.
            // If terrain collision (not hurt/death), flip direction and negate x_velocity.
            if let Some(grid) = block_grid {
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
                    let (facing, x_velocity) =
                        dragon_reverse_x_motion(enemy.facing, enemy.x_velocity);
                    enemy.facing = facing;
                    enemy.x_velocity = x_velocity;
                }
            }
        }
        DragonFlyingState::Descending => {
            step_dragon_descent(enemy, projectiles);
            dragon_apply_velocity(enemy);
        }
        DragonFlyingState::ReturningToAnchor => {
            // Return to anchor: decelerate upward. ASM loc_34B4C.
            enemy.x_velocity = 0;
            let anchor_y = enemy.dragon_anchor_y as i64;
            let current_y = enemy.y_pos as i64;
            if current_y <= anchor_y {
                // Reached anchor: restart script. ASM bra loc_346C4.
                dragon_restart_script(enemy);
            } else {
                // Decelerate upward: subtract $400 per frame from y_vel.
                enemy.y_velocity -= DRAGON_RETURN_DECEL;
                if enemy.y_velocity < DRAGON_RETURN_VEL_CAP_SOFT {
                    // Soft cap exceeded: use hard cap. ASM loc_34B70.
                    enemy.y_velocity = DRAGON_RETURN_VEL_CAP_HARD;
                }
            }
            dragon_apply_velocity(enemy);
        }
    }
}

/// Apply velocity to position (FP16.16 arithmetic).
fn dragon_apply_velocity(enemy: &mut ActiveEnemy) {
    enemy.x_pos = (enemy.x_pos as i64 + i64::from(enemy.x_velocity)) as u32;
    enemy.y_pos = (enemy.y_pos as i64 + i64::from(enemy.y_velocity)) as u32;
}

/// Advance to the next command in the dragon's flight script.
/// Immediate commands (FacePlayer, Reverse) execute inline and continue advancing.
fn dragon_advance_script(
    enemy: &mut ActiveEnemy,
    player_x_pos: Option<u32>,
    _projectiles: &mut Option<&mut Vec<EnemyProjectile>>,
) {
    let script = dragon_script(enemy.dragon_script_variant);
    loop {
        // ASM Dragon_Flying.asm loc_346DA: fetch one script word, restart on
        // $FFFF, then use the low byte as a byte offset into off_3471E.
        enemy.dragon_script_cmd += 1;
        if enemy.dragon_script_cmd as usize >= script.len() {
            // Script end: restart. ASM $FFFF sentinel → loc_346C0.
            dragon_restart_script(enemy);
            return;
        }
        match script[enemy.dragon_script_cmd as usize] {
            DragonFlyingCmd::Hover(frames) => {
                enemy.dragon_phase = DragonFlyingState::Hovering;
                enemy.dragon_phase_frames = frames;
                enemy.x_velocity = 0;
                enemy.y_velocity = 0;
                return;
            }
            DragonFlyingCmd::Drift(frames) => {
                dragon_begin_drift(enemy, frames, false);
                return;
            }
            DragonFlyingCmd::DriftWithFire(frames) => {
                dragon_begin_drift(enemy, frames, true);
                return;
            }
            DragonFlyingCmd::FacePlayer => {
                // Immediate: face player and set initial drift speed. ASM loc_3473A.
                if let Some(px) = player_x_pos {
                    let (facing, x_velocity) = dragon_face_player_motion(enemy.x_pos, px);
                    enemy.facing = facing;
                    enemy.x_velocity = x_velocity;
                }
                // Continue to next command immediately (no frame consumed).
            }
            DragonFlyingCmd::Descent => {
                dragon_enter_descent(enemy);
                return;
            }
            DragonFlyingCmd::ReturnToAnchor => {
                enemy.dragon_phase = DragonFlyingState::ReturningToAnchor;
                enemy.x_velocity = 0;
                enemy.y_velocity = 0;
                return;
            }
        }
    }
}

pub(crate) fn dragon_face_player_motion(dragon_x_pos: u32, player_x_pos: u32) -> (Direction, i32) {
    let dragon_x_px = dragon_x_pos >> 16;
    let player_x_px = player_x_pos >> 16;
    if dragon_x_px < player_x_px {
        // ASM Dragon_Flying.asm loc_34758: x_direction=$FF and x_vel=$8000
        // only when the dragon's integer X word is strictly left of Kid.
        (Direction::Right, DRAGON_FACE_PLAYER_SPEED)
    } else {
        (Direction::Left, -DRAGON_FACE_PLAYER_SPEED)
    }
}

pub(crate) fn dragon_reverse_x_motion(facing: Direction, x_velocity: i32) -> (Direction, i32) {
    let facing = match facing {
        Direction::Right => Direction::Left,
        Direction::Left => Direction::Right,
    };
    // ASM Dragon_Flying.asm loc_34768: not.b x_direction(a3); neg.l x_vel(a3).
    (facing, x_velocity.wrapping_neg())
}

/// Begin a drift command: set initial speed and direction. ASM loc_34776.
fn dragon_begin_drift(enemy: &mut ActiveEnemy, frames: u16, with_fire: bool) {
    enemy.dragon_phase = if with_fire {
        DragonFlyingState::DriftingWithFire
    } else {
        DragonFlyingState::Drifting
    };
    enemy.dragon_phase_frames = frames;
    // Initial drift speed: $800, direction-dependent. ASM loc_34776.
    enemy.x_velocity = match enemy.facing {
        Direction::Right => DRAGON_DRIFT_INITIAL_SPEED,
        Direction::Left => -DRAGON_DRIFT_INITIAL_SPEED,
    };
}

/// Restart the flight script from the beginning. ASM loc_346C0 → loc_346C4.
fn dragon_restart_script(enemy: &mut ActiveEnemy) {
    enemy.dragon_script_cmd = 0;
    enemy.x_velocity = 0;
    enemy.y_velocity = 0;
    let script = dragon_script(enemy.dragon_script_variant);
    match script[0] {
        DragonFlyingCmd::Hover(frames) => {
            enemy.dragon_phase = DragonFlyingState::Hovering;
            enemy.dragon_phase_frames = frames;
        }
        DragonFlyingCmd::Drift(frames) => dragon_begin_drift(enemy, frames, false),
        DragonFlyingCmd::DriftWithFire(frames) => dragon_begin_drift(enemy, frames, true),
        _ => {
            enemy.dragon_phase = DragonFlyingState::Hovering;
            enemy.dragon_phase_frames = 1;
        }
    }
}

/// Enter the descent attack sub-script. ASM loc_3487E.
fn dragon_enter_descent(enemy: &mut ActiveEnemy) {
    enemy.dragon_phase = DragonFlyingState::Descending;
    enemy.dragon_descent_index = 0;
    enemy.dragon_descent_frames = 0;
    // First entry will be processed immediately in step_dragon_descent.
}

/// Step through the descent attack sub-script (unk_34554).
/// Each entry sets fixed x/y velocities for N frames. Special entries
/// spawn fire, flip direction, or restart the main script.
fn step_dragon_descent(
    enemy: &mut ActiveEnemy,
    projectiles: &mut Option<&mut Vec<EnemyProjectile>>,
) {
    loop {
        // ASM Dragon_Flying.asm loc_3488A reads the next `unk_34554` word:
        // restart, flip, and fire sentinels are immediate; movement entries
        // consume frames after velocity setup.
        if enemy.dragon_descent_index as usize >= DRAGON_DIVE_SCRIPT.len() {
            // Shouldn't happen — script ends with RestartScript.
            dragon_restart_script(enemy);
            return;
        }
        match DRAGON_DIVE_SCRIPT[enemy.dragon_descent_index as usize] {
            DragonDiveAction::Move {
                x_vel,
                y_vel,
                frames,
            } => {
                if enemy.dragon_descent_frames == 0 {
                    // Start this entry: set velocities (pixel/frame → FP16.16).
                    let setup =
                        dragon_descent_move_setup_loc_348e4(enemy.facing, x_vel, y_vel, frames);
                    enemy.x_velocity = setup.x_velocity;
                    enemy.y_velocity = setup.y_velocity;
                    // Native keeps a hibernated frame count here; ASM loc_348E4
                    // seeds the equivalent DBF counter with `frame_count - 1`.
                    enemy.dragon_descent_frames = frames;
                }
                // ASM Dragon_Flying.asm loc_34924 advances the descent script
                // only after the DBF counter underflows to `$FFFF`; native uses
                // a direct remaining-frame count for the same hibernated loop.
                enemy.dragon_descent_frames -= 1;
                if enemy.dragon_descent_frames == 0 {
                    enemy.dragon_descent_index += 1;
                }
                return; // Consume one frame.
            }
            DragonDiveAction::SpawnFire => {
                // Spawn fire-breath projectile during descent.
                // ASM loc_34BAC: x_pos += $FFF6 (-10px, negated if facing right),
                // y_pos += $A (+10px), x_vel from dragon, y_vel = 3<<16.
                // No frame-based lifetime — dies off-screen ($19) or on terrain.
                if let Some(projs) = projectiles {
                    let x_offset: i32 = match enemy.facing {
                        Direction::Right => 10,
                        Direction::Left => -10,
                    };
                    projs.push(EnemyProjectile {
                        x_pos: (enemy.x_pos as i64 + ((i64::from(x_offset)) << 16)) as u32,
                        y_pos: (enemy.y_pos as i64 + (10 << 16)) as u32,
                        // ASM Dragon_Flying loc_348A0: copy the dragon's live
                        // x_vel into the projectile and force y_vel to +3px/frame.
                        x_velocity: enemy.x_velocity,
                        y_velocity: 3 << 16,
                        // ASM: no lifetime timer; projectile persists until off-screen
                        // or terrain contact. Use large lifetime + terrain_kill.
                        lifetime: 600,
                        gravity: 0,
                        friendly: false,
                        homing_accel: 0,
                        speed_cap: 0,
                        wall_bounce: false,
                        terrain_kill: true,
                        ..Default::default()
                    });
                }
                enemy.dragon_descent_index += 1;
                // Immediate — continue to next entry without consuming a frame.
            }
            DragonDiveAction::FlipDirection => {
                // Flip facing direction. ASM $DD48.
                enemy.facing = match enemy.facing {
                    Direction::Left => Direction::Right,
                    Direction::Right => Direction::Left,
                };
                enemy.dragon_descent_index += 1;
                // Immediate — continue.
            }
            DragonDiveAction::RestartScript => {
                // Descent complete: restart main flight script. ASM $D8F1.
                dragon_restart_script(enemy);
                return;
            }
        }
    }
}

/// Fire-breath cone check (ASM loc_34836).
/// Returns true if the player is within the dragon's breath cone.
/// Cone geometry: player must be below dragon (Y >= dragon Y), and within
/// a triangular region defined by the relative position and facing direction.
/// The cone is 3:1 aspect (3× lateral distance = vertical threshold).
fn dragon_fire_breath_cone(
    enemy: &ActiveEnemy,
    player_x_pos: Option<u32>,
    player_y_pos: Option<u32>,
) -> bool {
    let (Some(px), Some(py)) = (player_x_pos, player_y_pos) else {
        return false;
    };
    let dragon_x = (enemy.x_pos >> 16) as i32;
    let dragon_y = (enemy.y_pos >> 16) as i32;
    let player_x = (px >> 16) as i32;
    let player_y = (py >> 16) as i32;

    // Player must be at or below dragon. ASM: cmp.w y_pos(a4),y_pos(a3) / blt return.
    if player_y < dragon_y {
        return false;
    }

    // Compute lateral distance and cone threshold.
    // ASM: d7 = dragon_x - player_x, then d7 = d7*3 (or d7*2+abs for facing).
    // The exact ASM: d7=dx, d6=dx, d7=d7+d7+d6=3*dx.
    // If facing right: d7 = d7 - d6 then neg → d7 = -(3*dx - dx) = -2*dx.
    // Wait, re-reading: d7=dx, d6=dx, d7+=d7 (d7=2*dx), d7+=d6 (d7=3*dx).
    // If facing right (x_direction set): sub d6 from d7 → d7=2*dx, then neg → -2*dx.
    let dx = dragon_x - player_x;
    let d7 = if enemy.facing == Direction::Right {
        -(dx * 2)
    } else {
        dx * 3
    };

    // Cone Y range: [d7 + dragon_y - 8, d7 + dragon_y].
    // Player Y must be within this range. ASM loc_34860-34878.
    let cone_y = d7 + dragon_y;
    player_y >= (cone_y - 8) && player_y <= cone_y
}

pub(crate) fn enemy_should_remain_active(
    enemy: &ActiveEnemy,
    view: &EnemyLayoutView<'_>,
    window: ActivationWindow,
    camera_x_px: u32,
    camera_y_px: u32,
    level_bounds: physics::LevelBounds,
    active_enemy_count: usize,
) -> bool {
    if enemy.segment_index != view.segment_index {
        return false;
    }
    let Some(object) = view.objects.get(enemy.layout_index) else {
        return false;
    };

    match enemy.family.archetype() {
        // Bosses are always retained — they're confined to their arena.
        EnemyArchetype::Tracker => true,
        EnemyArchetype::Flyer => window.contains_fixed_signed_with_margin(
            enemy.x_pos,
            enemy.y_pos,
            FLYER_RETENTION_X_PADDING,
            FLYER_RETENTION_Y_PADDING,
        ),
        EnemyArchetype::Collectible if matches!(enemy.family, EnemyFamily::Diamond) => {
            shared_enemy_should_remain_active_sub_3C3CE(
                enemy.x_pos,
                enemy.y_pos,
                level_bounds,
                active_enemy_count.saturating_add(3) as u16,
                camera_x_px,
                camera_y_px,
            )
        }
        EnemyArchetype::Pacer
            if matches!(enemy.family, EnemyFamily::DragonWalking | EnemyFamily::Lion) =>
        {
            large_pacer_should_remain_active(
                enemy,
                camera_x_px,
                camera_y_px,
                level_bounds,
                active_enemy_count,
            )
        }
        EnemyArchetype::Pacer
            if matches!(
                enemy.family,
                EnemyFamily::Crab
                    | EnemyFamily::Driller
                    | EnemyFamily::FireDemon
                    | EnemyFamily::Robot
                    | EnemyFamily::SpinningTwins
                    | EnemyFamily::Tank
                    | EnemyFamily::TankShooting
            ) =>
        {
            shared_enemy_should_remain_active_sub_3C3CE(
                enemy.x_pos,
                enemy.y_pos,
                level_bounds,
                active_enemy_count.saturating_add(3) as u16,
                camera_x_px,
                camera_y_px,
            )
        }
        EnemyArchetype::Hopper if matches!(enemy.family, EnemyFamily::MiniHoppingSkull) => {
            mini_hopping_skull_should_remain_active(
                enemy,
                camera_x_px,
                camera_y_px,
                level_bounds,
                active_enemy_count,
            )
        }
        _ => window.contains(u32::from(object.x), u32::from(object.y)),
    }
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn shared_enemy_should_remain_active_sub_3C3CE(
    x_pos: u32,
    y_pos: u32,
    level_bounds: physics::LevelBounds,
    number_objects: u16,
    camera_x_px: u32,
    camera_y_px: u32,
) -> bool {
    // ASM loc_3C044 / loc_3C0F0 (kid.asm:43505-43508): after the walker
    // hibernate frame, `bsr.w sub_3C3CE` checks whether the enemy should
    // remain active before dispatching to the next state via `jmp (a0)`.
    // ASM loc_3C194 (kid.asm:43653-43655): same hibernate + sub_3C3CE in the
    // leftward walking loop, but falls through to loc_3C19C (body collision
    // scan) instead of dispatching via `jmp (a0)`.
    // ASM sub_3C3CE (kid.asm:43901-43955): the shared Diamond/FireDemon/
    // walker retention path first rejects live positions outside
    // [-$20, level+$20] on both axes before applying the camera-relative
    // medium-load or heavy-load culling boxes.
    // ASM loc_3C436 (kid.asm:43936-43937): `bra.w loc_3C46E` — all four
    // level-bounds checks and both camera-relative culling boxes branch
    // here when the enemy falls outside the retention zone. loc_3C436 is
    // a trampoline to loc_3C46E (the full deactivation routine).
    let x_pos = (x_pos as i32) >> 16;
    let y_pos = (y_pos as i32) >> 16;
    let level_x_max = i32::from(level_bounds.width_pixels) + 0x20;
    let level_y_max = i32::from(level_bounds.height_pixels) + 0x20;
    if x_pos <= -0x20 || y_pos <= -0x20 || x_pos > level_x_max || y_pos > level_y_max {
        return false;
    }
    if number_objects <= 0x000A {
        return true;
    }

    let relative_x = x_pos - camera_x_px as i32;
    let relative_y = y_pos - camera_y_px as i32;
    let (left_margin, right_margin, top_margin, bottom_margin) = if number_objects <= 0x0014 {
        // ASM loc_3C43A: medium load uses [-$1A4,+$2E4]x[-$1A4,+$284].
        (0x1A4, 0x2E4, 0x1A4, 0x284)
    } else {
        // ASM sub_3C3CE fallthrough: heavy load tightens to
        // [-$104,+$244]x[-$104,+$1E4].
        (0x104, 0x244, 0x104, 0x1E4)
    };
    relative_x >= -left_margin
        && relative_x <= right_margin
        && relative_y >= -top_margin
        && relative_y <= bottom_margin
}

fn large_pacer_should_remain_active(
    enemy: &ActiveEnemy,
    camera_x_px: u32,
    camera_y_px: u32,
    level_bounds: physics::LevelBounds,
    active_enemy_count: usize,
) -> bool {
    // ASM Dragon.asm loc_374E8 and Lion.asm sub_3E8BA both cull the live
    // object from its current world position rather than the layout anchor.
    let x_pos = (enemy.x_pos as i32) >> 16;
    let y_pos = (enemy.y_pos as i32) >> 16;
    let level_x_max = i32::from(level_bounds.width_pixels) + 0x20;
    let level_y_max = i32::from(level_bounds.height_pixels) + 0x20;
    if x_pos <= -0x20 || y_pos <= -0x20 || x_pos > level_x_max || y_pos > level_y_max {
        return false;
    }
    if active_enemy_count <= 10 {
        return true;
    }

    // ASM Dragon.asm loc_37552 / Lion.asm sub_3E8BA:
    // 11..20 active objects use the wider [-$1A4,+$2E4]x[-$1A4,+$284] box,
    // 21+ active objects tighten to [-$104,+$244]x[-$104,+$1E4].
    let (left_margin, right_margin, top_margin, bottom_margin) = if active_enemy_count <= 20 {
        (0x1A4, 0x2E4, 0x1A4, 0x284)
    } else {
        (0x104, 0x244, 0x104, 0x1E4)
    };
    let relative_x = x_pos - camera_x_px as i32;
    let relative_y = y_pos - camera_y_px as i32;
    relative_x >= -left_margin
        && relative_x <= right_margin
        && relative_y >= -top_margin
        && relative_y <= bottom_margin
}

fn mini_hopping_skull_should_remain_active(
    enemy: &ActiveEnemy,
    camera_x_px: u32,
    camera_y_px: u32,
    level_bounds: physics::LevelBounds,
    active_enemy_count: usize,
) -> bool {
    // ASM Mini_Hopping_Skull.asm sub_3F29A culls from the skull's live
    // object position, not its layout anchor. The routine first rejects
    // positions outside [-$20, level+$20] on both axes.
    let x_pos = (enemy.x_pos as i32) >> 16;
    let y_pos = (enemy.y_pos as i32) >> 16;
    let level_x_max = i32::from(level_bounds.width_pixels) + 0x20;
    let level_y_max = i32::from(level_bounds.height_pixels) + 0x20;
    if x_pos <= -0x20 || y_pos <= -0x20 || x_pos > level_x_max || y_pos > level_y_max {
        return false;
    }
    if active_enemy_count <= 10 {
        return true;
    }

    // ASM Mini_Hopping_Skull.asm sub_3F29A:
    // 11..20 active objects use [-$1A4,+$2E4]x[-$1A4,+$284] relative to camera.
    // 21+ active objects tighten to [-$40,+$180]x[-$40,+$120].
    let (left_margin, right_margin, top_margin, bottom_margin) = if active_enemy_count <= 20 {
        (0x1A4, 0x2E4, 0x1A4, 0x284)
    } else {
        (0x40, 0x180, 0x40, 0x120)
    };
    let relative_x = x_pos - camera_x_px as i32;
    let relative_y = y_pos - camera_y_px as i32;
    relative_x >= -left_margin
        && relative_x <= right_margin
        && relative_y >= -top_margin
        && relative_y <= bottom_margin
}

fn mini_hopping_skull_spawn_gate_loc_3e9d0(
    object_x_px: u32,
    object_y_px: u32,
    player_x_pos: u32,
    player_y_pos: u32,
) -> bool {
    // ASM Mini_Hopping_Skull.asm loc_3E9D0: hibernate until
    // `kid_x-layout_x` is in [-$50,$50] and `kid_y-layout_y` is in [0,$A0].
    let player_x_px = (player_x_pos as i32) >> 16;
    let player_y_px = (player_y_pos as i32) >> 16;
    let dx = player_x_px - object_x_px as i32;
    let dy = player_y_px - object_y_px as i32;
    (-0x50..=0x50).contains(&dx) && (0..=0xA0).contains(&dy)
}

fn mini_hopping_skull_snap_slopeup_x(grid: &terrain::BlockGrid, x_px: i32, y_px: i32) -> u32 {
    let row = terrain::pixel_to_cell(y_px).max(0) as u16;
    let start_col = terrain::pixel_to_cell(x_px).max(0) as u16;

    for col in start_col..grid.width_cells {
        let cell = grid.cell_at(col, row);
        if cell.collision == terrain::CollisionKind::SlopeUp {
            // ASM Mini_Hopping_Skull.asm sub_3F366: `addi.w #$10,d6` before the
            // collision check means the snap uses the matched slope cell's right
            // edge, then subtracts the low nibble of the current Y position.
            let snapped_x_px = ((i32::from(col) + 1) * 16) - (y_px & 0x0F);
            return (snapped_x_px.max(0) as u32) << 16;
        }
    }

    (x_px.max(0) as u32) << 16
}

fn mini_hopping_skull_snap_slopedown_x(grid: &terrain::BlockGrid, x_px: i32, y_px: i32) -> u32 {
    let row = terrain::pixel_to_cell(y_px).max(0) as u16;
    let start_col = terrain::pixel_to_cell(x_px).max(0) + 1;

    for col in (0..=start_col).rev() {
        let cell = grid.cell_at(col as u16, row);
        if cell.collision == terrain::CollisionKind::SlopeDown {
            // ASM Mini_Hopping_Skull.asm sub_3F3A6: `addi.w #2,a4` advances to
            // the next cell on the right before each loop subtracts 16px, so
            // the final snap uses the matched slope cell's left edge plus the
            // current Y low nibble.
            let snapped_x_px = (col * 16) + (y_px & 0x0F);
            return (snapped_x_px.max(0) as u32) << 16;
        }
    }

    (x_px.max(0) as u32) << 16
}

#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn mini_hopping_skull_air_sprite_after_loc_3ea88(
    is_animated: bool,
    y_velocity: i32,
) -> Option<MiniHoppingSkullAirSprite> {
    if is_animated {
        return None;
    }

    if y_velocity < 0 {
        Some(MiniHoppingSkullAirSprite::Rising)
    } else {
        // ASM Mini_Hopping_Skull.asm loc_3EAAC: non-negative `y_vel(a3)`
        // selects `LnkTo_unk_C8088` after the animated-state guard is clear.
        Some(MiniHoppingSkullAirSprite::Falling)
    }
}

#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn mini_hopping_skull_collision_dispatch_loc_3eab2(
    collision_type: u16,
) -> MiniHoppingSkullCollisionDispatch {
    // ASM Mini_Hopping_Skull.asm loc_3EAB2 reads `collision_type(a3)` into d4
    // and dispatches by compare order, not by a dense table.
    if collision_type == 0 {
        MiniHoppingSkullCollisionDispatch::AirborneLoop
    } else if collision_type >= MINI_SKULL_COLID_HURT {
        MiniHoppingSkullCollisionDispatch::HurtOrWeapon
    } else if collision_type <= MINI_SKULL_COLID_LEFTWALL {
        MiniHoppingSkullCollisionDispatch::WallBounce
    } else if collision_type == MINI_SKULL_COLID_FLOOR {
        MiniHoppingSkullCollisionDispatch::FloorLanding
    } else if collision_type == MINI_SKULL_COLID_CEILING {
        MiniHoppingSkullCollisionDispatch::CeilingHit
    } else {
        MiniHoppingSkullCollisionDispatch::SlopeOrOtherTerrain
    }
}

#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn mini_hopping_skull_terrain_contact_loc_3ead8(
    collision_type: u16,
    x_velocity: i32,
) -> MiniHoppingSkullTerrainContact {
    let route = if collision_type != MINI_SKULL_COLID_SLOPEUP {
        MiniHoppingSkullTerrainContactRoute::NotSlopeUp
    } else if x_velocity <= 0 {
        MiniHoppingSkullTerrainContactRoute::SlopeUpNonPositiveXVelocity
    } else {
        MiniHoppingSkullTerrainContactRoute::SlopeUpPositiveXVelocity
    };

    // ASM Mini_Hopping_Skull.asm loc_3EAD8: terrain contact clears
    // `is_moved(a3)`, `is_animated(a3)`, and `has_level_collision(a3)`
    // before the slope-up-specific velocity gate.
    MiniHoppingSkullTerrainContact {
        is_moved: false,
        is_animated: false,
        has_level_collision: false,
        route,
    }
}

#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn mini_hopping_skull_slopedown_contact_loc_3ed6c(
    x_velocity: i32,
) -> MiniHoppingSkullSlopeDownContact {
    // ASM Mini_Hopping_Skull.asm loc_3ED6C: after `sub_3F3A6` snaps to the
    // slope-down surface, `tst.w x_vel(a3)` tests the high word. Negative
    // velocity clears both velocities and enters the left wait; a
    // non-negative high word jumps to the mirrored right wait setup.
    if x_velocity < 0 {
        MiniHoppingSkullSlopeDownContact {
            x_velocity: 0,
            y_velocity: 0,
            x_direction: 0x00,
            sprite: "LnkTo_unk_C8030",
            dbf_counter: 0x0014,
            wait_frames: 21,
            wait: MiniHoppingSkullSlopeDownWait::Left,
        }
    } else {
        MiniHoppingSkullSlopeDownContact {
            x_velocity: 0,
            y_velocity: 0,
            x_direction: 0xFF,
            sprite: "LnkTo_unk_C8010",
            dbf_counter: 0x0014,
            wait_frames: 21,
            wait: MiniHoppingSkullSlopeDownWait::Right,
        }
    }
}

#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn mini_hopping_skull_left_hop_wait_setup_loc_3ed80() -> MiniHoppingSkullGroundWaitSetup
{
    // ASM Mini_Hopping_Skull.asm loc_3ED80: left-facing slope-landing setup.
    // Clears to zero velocities, sets `x_direction(a3)` clear, selects
    // `LnkTo_unk_C8030`, and seeds the DBF wait loop with `$14`.
    MiniHoppingSkullGroundWaitSetup {
        x_velocity: 0,
        y_velocity: 0,
        x_direction: 0x00,
        sprite: MiniHoppingSkullGroundWaitSprite::SlopeUpRight,
        dbf_counter: 0x0014,
        wait_frames: 21,
    }
}

#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn mini_hopping_skull_left_hop_windup_collision_loc_3ed8e(
    collision_type: u16,
    berzerker_charging: bool,
) -> MiniHoppingSkullGroundWaitCollisionRoute {
    // ASM Mini_Hopping_Skull.asm loc_3ED8E: clear `collision_type(a3)`, hibernate
    // one frame, poll `sub_3F29A`, then dispatch by ordered Kid/hurt
    // comparisons. Side collisions branch through `loc_3EDD6`, below collisions
    // flow back to the same wait loop.
    match collision_type {
        0 => MiniHoppingSkullGroundWaitCollisionRoute::ContinueWait,
        MINI_SKULL_COLID_GROUND_KIDABOVE | MINI_SKULL_COLID_GROUND_HURT => {
            MiniHoppingSkullGroundWaitCollisionRoute::DamageSkull
        }
        MINI_SKULL_COLID_GROUND_KIDLEFT | MINI_SKULL_COLID_GROUND_KIDRIGHT => {
            if berzerker_charging {
                MiniHoppingSkullGroundWaitCollisionRoute::BerzerkerChargeKillsSkull
            } else {
                MiniHoppingSkullGroundWaitCollisionRoute::QueueScreamWorker
            }
        }
        _ => MiniHoppingSkullGroundWaitCollisionRoute::ContinueWait,
    }
}

#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn mini_hopping_skull_left_hop_landing_gate_loc_3edf8(
    dbf_counter: u16,
    kid_x_word: u16,
    skull_x_word: u16,
) -> MiniHoppingSkullLeftLandingLoopDecision {
    // ASM Mini_Hopping_Skull.asm loc_3EDF8: if `dbf d1` is still active, this
    // loops back to loc_3ED8E. On fallthrough, Kid-to-skull signed comparison
    // decides whether to branch to loc_3EEB2 (`d1=$14`) or to seed `d1=$A`
    // with sprite `LnkTo_unk_C8038` before landing-loop continuation.
    let next_dbf_counter = dbf_counter.wrapping_sub(1);
    if next_dbf_counter != 0xFFFF {
        return MiniHoppingSkullLeftLandingLoopDecision::ContinueWait { next_dbf_counter };
    }

    if (kid_x_word as i16) > (skull_x_word as i16) {
        MiniHoppingSkullLeftLandingLoopDecision::BeginSkullFacingLeftLanding {
            dbf_counter: 0x0014,
            x_direction: 0xFF,
            sprite: "LnkTo_unk_C8010",
        }
    } else {
        MiniHoppingSkullLeftLandingLoopDecision::BeginSkullLeftRelaunch {
            dbf_counter: 0x000A,
            sprite: "LnkTo_unk_C8038",
        }
    }
}

#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn mini_hopping_skull_left_hop_scream_worker_allocation_loc_3edc0()
-> MiniHoppingSkullScreamWorkerAllocation {
    // ASM Mini_Hopping_Skull.asm loc_3EDC0: swap object pointers and allocate
    // with selector `$A000`, then install `sub_3F3F0` at slot +4 before
    // returning to the shared collision-tail flow.
    MiniHoppingSkullScreamWorkerAllocation {
        allocation_selector: 0xA000,
        routine: "sub_3F3F0",
        delay_frames: 15,
    }
}

#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn mini_hopping_skull_landing_collision_route_loc_3ee12(
    collision_type: u16,
    berzerker_charging: bool,
) -> MiniHoppingSkullGroundWaitCollisionRoute {
    // ASM Mini_Hopping_Skull.asm loc_3EE12: clear `collision_type(a3)`, hibernate
    // one frame, call `sub_3F29A`, then dispatch by collision order:
    // kid-above and `colid_hurt` go through the same damage branch as `loc_3EE64`,
    // side contact routes via `loc_3EE5A`, below contact queues the scream worker.
    match collision_type {
        0 => MiniHoppingSkullGroundWaitCollisionRoute::ContinueWait,
        MINI_SKULL_COLID_GROUND_KIDABOVE | MINI_SKULL_COLID_GROUND_HURT => {
            MiniHoppingSkullGroundWaitCollisionRoute::DamageSkull
        }
        MINI_SKULL_COLID_GROUND_KIDLEFT | MINI_SKULL_COLID_GROUND_KIDRIGHT => {
            if berzerker_charging {
                MiniHoppingSkullGroundWaitCollisionRoute::BerzerkerChargeKillsSkull
            } else {
                MiniHoppingSkullGroundWaitCollisionRoute::QueueScreamWorker
            }
        }
        MINI_SKULL_COLID_GROUND_KIDBELOW => {
            MiniHoppingSkullGroundWaitCollisionRoute::QueueScreamWorker
        }
        _ => MiniHoppingSkullGroundWaitCollisionRoute::ContinueWait,
    }
}

#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn mini_hopping_skull_landing_collision_route_loc_3eec8(
    collision_type: u16,
    berzerker_charging: bool,
) -> MiniHoppingSkullGroundWaitCollisionRoute {
    // ASM Mini_Hopping_Skull.asm loc_3EEC8: same collision compare order as
    // loc_3EE12, but `loc_3EF10` handles left/right contact with a Berzerker
    // branch and below-contact goes through the contact worker path.
    match collision_type {
        0 => MiniHoppingSkullGroundWaitCollisionRoute::ContinueWait,
        MINI_SKULL_COLID_GROUND_KIDABOVE | MINI_SKULL_COLID_GROUND_HURT => {
            MiniHoppingSkullGroundWaitCollisionRoute::DamageSkull
        }
        MINI_SKULL_COLID_GROUND_KIDLEFT | MINI_SKULL_COLID_GROUND_KIDRIGHT => {
            mini_hopping_skull_landing_side_contact_loc_3ef10(berzerker_charging)
        }
        MINI_SKULL_COLID_GROUND_KIDBELOW => {
            MiniHoppingSkullGroundWaitCollisionRoute::QueueScreamWorker
        }
        _ => MiniHoppingSkullGroundWaitCollisionRoute::ContinueWait,
    }
}

#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn mini_hopping_skull_landing_side_contact_loc_3ef10(
    berzerker_charging: bool,
) -> MiniHoppingSkullGroundWaitCollisionRoute {
    // ASM Mini_Hopping_Skull.asm loc_3EF10: side contact gates a Berzerker kill;
    // if not charging, allocate `sub_3F3F0` through loc_3EEFA.
    if berzerker_charging {
        MiniHoppingSkullGroundWaitCollisionRoute::BerzerkerChargeKillsSkull
    } else {
        MiniHoppingSkullGroundWaitCollisionRoute::QueueScreamWorker
    }
}

#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn mini_hopping_skull_landing_scream_worker_allocation_loc_3eefa(
    route: MiniHoppingSkullGroundWaitCollisionRoute,
) -> Option<MiniHoppingSkullScreamWorkerAllocation> {
    if route != MiniHoppingSkullGroundWaitCollisionRoute::QueueScreamWorker {
        return None;
    }

    // ASM Mini_Hopping_Skull.asm loc_3EEFA: `exg a0,a4`, then allocate selector
    // `$A000`, store `sub_3F3F0` at offset 4, and restore a0/a4.
    Some(MiniHoppingSkullScreamWorkerAllocation {
        allocation_selector: 0xA000,
        routine: "sub_3F3F0",
        delay_frames: 15,
    })
}

#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn mini_hopping_skull_landing_tail_loc_3ee7c(
    dbf_counter: u16,
    is_boosted: bool,
) -> MiniHoppingSkullLandingLoopDecision {
    // ASM Mini_Hopping_Skull.asm loc_3EE7C: DBCC loop first (`dbf d1`). On fallthrough,
    // seed airborne relaunch velocities and set movement/collision flags, then jump to
    // `loc_3EA88`.
    let next_dbf_counter = dbf_counter.wrapping_sub(1);
    if next_dbf_counter != 0xFFFF {
        return MiniHoppingSkullLandingLoopDecision::ContinueWait { next_dbf_counter };
    }

    let mut y_velocity = 0xFFFC8000u32 as i32;
    let mut x_velocity = 0xFFFE8000u32 as i32;
    if is_boosted {
        // ASM loc_3EEA6 (`tst.w $40(a3)` + `addi`) bumps both speeds negative.
        y_velocity = y_velocity.wrapping_add(-0x1_0000);
        x_velocity = x_velocity.wrapping_add(-0x4000);
    }

    MiniHoppingSkullLandingLoopDecision::BeginRelaunchArc {
        y_velocity,
        x_velocity,
        is_moved: true,
        has_level_collision: true,
        next_loop: "loc_3EA88",
    }
}

#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn mini_hopping_skull_ground_wait_setup_loc_3eb00() -> MiniHoppingSkullGroundWaitSetup {
    // ASM Mini_Hopping_Skull.asm loc_3EB00: after positive-X slope-up contact,
    // clear both velocities, `st x_direction(a3)`, select `LnkTo_unk_C8030`,
    // and seed d1 with `$14`; the following DBF loop runs 21 hibernated frames.
    MiniHoppingSkullGroundWaitSetup {
        x_velocity: 0,
        y_velocity: 0,
        x_direction: 0xFF,
        sprite: MiniHoppingSkullGroundWaitSprite::SlopeUpRight,
        dbf_counter: 0x0014,
        wait_frames: 21,
    }
}

#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn mini_hopping_skull_ground_wait_collision_loc_3eb0e(
    collision_type: u16,
    berzerker_charging: bool,
) -> MiniHoppingSkullGroundWaitCollisionRoute {
    // ASM Mini_Hopping_Skull.asm loc_3EB0E: each grounded wait frame clears
    // `collision_type(a3)`, hibernates one frame, polls `sub_3F29A`, then
    // dispatches by ordered comparisons against Kid/hurt collision IDs.
    match collision_type {
        0 => MiniHoppingSkullGroundWaitCollisionRoute::ContinueWait,
        MINI_SKULL_COLID_GROUND_KIDABOVE | MINI_SKULL_COLID_GROUND_HURT => {
            MiniHoppingSkullGroundWaitCollisionRoute::DamageSkull
        }
        MINI_SKULL_COLID_GROUND_KIDLEFT | MINI_SKULL_COLID_GROUND_KIDRIGHT => {
            mini_hopping_skull_side_contact_loc_3eb56(berzerker_charging)
        }
        MINI_SKULL_COLID_GROUND_KIDBELOW => {
            MiniHoppingSkullGroundWaitCollisionRoute::QueueScreamWorker
        }
        _ => MiniHoppingSkullGroundWaitCollisionRoute::ContinueWait,
    }
}

#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn mini_hopping_skull_side_contact_loc_3eb56(
    berzerker_charging: bool,
) -> MiniHoppingSkullGroundWaitCollisionRoute {
    // ASM Mini_Hopping_Skull.asm loc_3EB56: side contact tests the global
    // `Berzerker_charging` byte; clear branches back to loc_3EB40's contact
    // damage worker, non-clear jumps to loc_3F262 to kill the skull.
    if berzerker_charging {
        MiniHoppingSkullGroundWaitCollisionRoute::BerzerkerChargeKillsSkull
    } else {
        MiniHoppingSkullGroundWaitCollisionRoute::QueueScreamWorker
    }
}

#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn mini_hopping_skull_right_hop_side_contact_loc_3ebda(
    berzerker_charging: bool,
) -> MiniHoppingSkullGroundWaitCollisionRoute {
    // ASM Mini_Hopping_Skull.asm loc_3EBDA: the right-hop windup side-contact
    // gate repeats loc_3EB56's Berzerker check; clear falls through to
    // the contact worker, set jumps to loc_3F262 to kill the skull.
    if berzerker_charging {
        MiniHoppingSkullGroundWaitCollisionRoute::BerzerkerChargeKillsSkull
    } else {
        MiniHoppingSkullGroundWaitCollisionRoute::QueueScreamWorker
    }
}

#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn mini_hopping_skull_left_ground_side_contact_loc_3ec90(
    berzerker_charging: bool,
) -> MiniHoppingSkullGroundWaitCollisionRoute {
    // ASM Mini_Hopping_Skull.asm loc_3EC90: side contact during the
    // left-grounded wait tests `Berzerker_charging`; clear branches to
    // loc_3EC7A's contact worker, set jumps to loc_3F262 to kill the skull.
    if berzerker_charging {
        MiniHoppingSkullGroundWaitCollisionRoute::BerzerkerChargeKillsSkull
    } else {
        MiniHoppingSkullGroundWaitCollisionRoute::QueueScreamWorker
    }
}

#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn mini_hopping_skull_left_hop_side_contact_loc_3ed14(
    berzerker_charging: bool,
) -> MiniHoppingSkullGroundWaitCollisionRoute {
    // ASM Mini_Hopping_Skull.asm loc_3ED14: side contact during the left-hop
    // windup tests `Berzerker_charging`; clear branches to loc_3ECFE's contact
    // worker, set jumps to the skull kill path.
    if berzerker_charging {
        MiniHoppingSkullGroundWaitCollisionRoute::BerzerkerChargeKillsSkull
    } else {
        MiniHoppingSkullGroundWaitCollisionRoute::QueueScreamWorker
    }
}

#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn mini_hopping_skull_damage_sequence_loc_3eb60(
    hit_counter: u16,
) -> MiniHoppingSkullDamageSequence {
    // ASM Mini_Hopping_Skull.asm loc_3EB60: `subi.w #1,$44(a3)` mutates the
    // live hit counter before the zero test; non-zero replays `stru_3E9C2`
    // and hibernates until that animation finishes.
    let remaining_hits = hit_counter.wrapping_sub(1);
    if remaining_hits == 0 {
        MiniHoppingSkullDamageSequence::SkullDestroyed
    } else {
        MiniHoppingSkullDamageSequence::PlayHitAnimation {
            remaining_hits,
            animation: "stru_3E9C2",
            hibernate_until_finished: true,
        }
    }
}

#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn mini_hopping_skull_right_hop_damage_sequence_loc_3ebe4(
    hit_counter: u16,
) -> MiniHoppingSkullDamageSequence {
    // ASM Mini_Hopping_Skull.asm loc_3EBE4: the right-hop windup damage path
    // repeats loc_3EB60's word decrement of `$44(a3)`; zero branches to
    // loc_3F176, otherwise `stru_3E9C2` plays before the wait loop resumes.
    mini_hopping_skull_damage_sequence_loc_3eb60(hit_counter)
}

#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn mini_hopping_skull_left_hop_damage_sequence_loc_3ed1e(
    hit_counter: u16,
) -> MiniHoppingSkullDamageSequence {
    // ASM Mini_Hopping_Skull.asm loc_3ED1E: the left-hop windup damage path
    // repeats the `$44(a3)` word decrement; zero branches to the death path,
    // otherwise `stru_3E9C2` plays before the windup DBF loop resumes.
    mini_hopping_skull_damage_sequence_loc_3eb60(hit_counter)
}

#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn mini_hopping_skull_landing_collision_damage_loc_3ef1a(
    hit_counter: u16,
) -> MiniHoppingSkullDamageSequence {
    // ASM Mini_Hopping_Skull.asm loc_3EF1A: `subi.w #1,$44(a3)` then
    // branches to loc_3F18C on zero, else replays `stru_3E9C2` and waits.
    mini_hopping_skull_damage_sequence_loc_3eb60(hit_counter)
}

#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn mini_hopping_skull_landing_collision_wait_loc_3ef32(
    dbf_counter: u16,
    kid_x_word: u16,
    skull_x_word: u16,
) -> MiniHoppingSkullLeftLandingLoopDecision {
    // ASM Mini_Hopping_Skull.asm loc_3EF32: decrements the wait DBF counter;
    // if it still runs, loop. On exhaustion, a signed Kid-vs-skull X compare
    // routes either to loc_3ED80 (`<=`) or seeds the right-facing C8018 phase
    // with d1 = $A and continues through loc_3EF4C.
    let next_dbf_counter = dbf_counter.wrapping_sub(1);
    if next_dbf_counter != 0xFFFF {
        return MiniHoppingSkullLeftLandingLoopDecision::ContinueWait { next_dbf_counter };
    }

    if (kid_x_word as i16) <= (skull_x_word as i16) {
        MiniHoppingSkullLeftLandingLoopDecision::BeginSkullFacingLeftLanding {
            dbf_counter: 0x0014,
            x_direction: 0x00,
            sprite: "LnkTo_unk_C8030",
        }
    } else {
        MiniHoppingSkullLeftLandingLoopDecision::BeginSkullLeftRelaunch {
            dbf_counter: 0x000A,
            sprite: "LnkTo_unk_C8018",
        }
    }
}

#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn mini_hopping_skull_landing_tail_collision_loc_3ef4c(
    dbf_counter: u16,
    collision_type: u16,
    berzerker_charging: bool,
    hit_counter: u16,
    is_boosted: bool,
) -> MiniHoppingSkullLandingTailCollisionOutcome {
    // ASM Mini_Hopping_Skull.asm loc_3EF4C: clear/inspect collision each frame;
    // on DBF expiry, collision outcome can damage the skull, queue a worker, kill
    // on Berzerker charge, or fall through to the relaunch tail shared with
    // `loc_3EE7C`.
    let next_dbf_counter = dbf_counter.wrapping_sub(1);
    if next_dbf_counter != 0xFFFF {
        return MiniHoppingSkullLandingTailCollisionOutcome::ContinueWait { next_dbf_counter };
    }

    match collision_type {
        MINI_SKULL_COLID_GROUND_KIDABOVE | MINI_SKULL_COLID_GROUND_HURT => {
            match mini_hopping_skull_landing_collision_damage_loc_3ef1a(hit_counter) {
                MiniHoppingSkullDamageSequence::SkullDestroyed => {
                    MiniHoppingSkullLandingTailCollisionOutcome::DamageSkullAndDestroy
                }
                sequence @ MiniHoppingSkullDamageSequence::PlayHitAnimation { .. } => {
                    MiniHoppingSkullLandingTailCollisionOutcome::DamageSkullThenRelaunch {
                        sequence,
                    }
                }
            }
        }
        MINI_SKULL_COLID_GROUND_KIDLEFT | MINI_SKULL_COLID_GROUND_KIDRIGHT => {
            if berzerker_charging {
                MiniHoppingSkullLandingTailCollisionOutcome::BerzerkerChargeKillsSkull
            } else {
                MiniHoppingSkullLandingTailCollisionOutcome::SpawnScreamWorkerAndRelaunch
            }
        }
        MINI_SKULL_COLID_GROUND_KIDBELOW => {
            MiniHoppingSkullLandingTailCollisionOutcome::SpawnScreamWorkerAndRelaunch
        }
        _ => {
            let mut y_velocity = 0xFFFC8000u32 as i32;
            let mut x_velocity = 0x0001_8000u32 as i32;
            if is_boosted {
                y_velocity = y_velocity.wrapping_add(-0x1_0000);
                x_velocity = x_velocity.wrapping_add(0x4000);
            }

            MiniHoppingSkullLandingTailCollisionOutcome::BeginRelaunchArc {
                y_velocity,
                x_velocity,
                is_moved: true,
                has_level_collision: true,
                next_loop: "loc_3EA88",
            }
        }
    }
}

#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn mini_hopping_skull_damage_sequence_loc_3ede0(
    hit_counter: u16,
) -> MiniHoppingSkullDamageSequence {
    // ASM Mini_Hopping_Skull.asm loc_3EDE0: this second HP-decrement path in the
    // left-hop flow mirrors `loc_3EB60`, subtracting one HP, jumping to death
    // on zero, and replaying `stru_3E9C2` otherwise.
    let remaining_hits = hit_counter.wrapping_sub(1);
    if remaining_hits == 0 {
        MiniHoppingSkullDamageSequence::SkullDestroyed
    } else {
        MiniHoppingSkullDamageSequence::PlayHitAnimation {
            remaining_hits,
            animation: "stru_3E9C2",
            hibernate_until_finished: true,
        }
    }
}

#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn mini_hopping_skull_ground_wait_tail_loc_3eb78(
    dbf_counter: u16,
    kid_x_word: u16,
    skull_x_word: u16,
) -> MiniHoppingSkullGroundWaitTail {
    // ASM Mini_Hopping_Skull.asm loc_3EB78: `dbf d1` consumes the inclusive
    // wait counter first. Only the `$FFFF` fall-through compares Kid X against
    // skull X as signed words; Kid to the right starts the short right-hop windup.
    let next_dbf_counter = dbf_counter.wrapping_sub(1);
    if next_dbf_counter != 0xFFFF {
        return MiniHoppingSkullGroundWaitTail::ContinueWait { next_dbf_counter };
    }

    if (kid_x_word as i16) <= (skull_x_word as i16) {
        MiniHoppingSkullGroundWaitTail::SwitchToLeftGroundWaitSetup
    } else {
        MiniHoppingSkullGroundWaitTail::BeginRightHopWindup {
            sprite: "LnkTo_unk_C8038",
            dbf_counter: 0x000A,
            wait_frames: 11,
        }
    }
}

#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn mini_hopping_skull_left_ground_wait_tail_loc_3ecb2(
    dbf_counter: u16,
    kid_x_word: u16,
    skull_x_word: u16,
) -> MiniHoppingSkullGroundWaitTail {
    // ASM Mini_Hopping_Skull.asm loc_3ECB2: `dbf d1` consumes the inclusive
    // left-grounded wait counter first. Only the `$FFFF` fall-through compares
    // Kid X against skull X as signed words; Kid to the right returns to
    // loc_3EB00, otherwise the short left-hop windup uses `LnkTo_unk_C8018`.
    let next_dbf_counter = dbf_counter.wrapping_sub(1);
    if next_dbf_counter != 0xFFFF {
        return MiniHoppingSkullGroundWaitTail::ContinueWait { next_dbf_counter };
    }

    if (kid_x_word as i16) > (skull_x_word as i16) {
        MiniHoppingSkullGroundWaitTail::SwitchToRightGroundWaitSetup
    } else {
        MiniHoppingSkullGroundWaitTail::BeginLeftHopWindup {
            sprite: "LnkTo_unk_C8018",
            dbf_counter: 0x000A,
            wait_frames: 11,
        }
    }
}

#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn mini_hopping_skull_right_hop_windup_collision_loc_3eb92(
    collision_type: u16,
    berzerker_charging: bool,
) -> MiniHoppingSkullGroundWaitCollisionRoute {
    // ASM Mini_Hopping_Skull.asm loc_3EB92: each right-hop windup frame clears
    // `collision_type(a3)`, hibernates one frame, polls `sub_3F29A`, then
    // dispatches by the same ordered Kid/hurt collision comparisons as the
    // grounded wait path.
    match collision_type {
        0 => MiniHoppingSkullGroundWaitCollisionRoute::ContinueWait,
        MINI_SKULL_COLID_GROUND_KIDABOVE | MINI_SKULL_COLID_GROUND_HURT => {
            MiniHoppingSkullGroundWaitCollisionRoute::DamageSkull
        }
        MINI_SKULL_COLID_GROUND_KIDLEFT | MINI_SKULL_COLID_GROUND_KIDRIGHT => {
            mini_hopping_skull_right_hop_side_contact_loc_3ebda(berzerker_charging)
        }
        MINI_SKULL_COLID_GROUND_KIDBELOW => {
            MiniHoppingSkullGroundWaitCollisionRoute::QueueScreamWorker
        }
        _ => MiniHoppingSkullGroundWaitCollisionRoute::ContinueWait,
    }
}

#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn mini_hopping_skull_left_ground_wait_collision_loc_3ec48(
    collision_type: u16,
    berzerker_charging: bool,
) -> MiniHoppingSkullGroundWaitCollisionRoute {
    // ASM Mini_Hopping_Skull.asm loc_3EC48: the left-facing grounded wait loop
    // clears `collision_type(a3)`, hibernates one frame, polls `sub_3F29A`,
    // then dispatches by ordered Kid/hurt collision comparisons.
    match collision_type {
        0 => MiniHoppingSkullGroundWaitCollisionRoute::ContinueWait,
        MINI_SKULL_COLID_GROUND_KIDABOVE | MINI_SKULL_COLID_GROUND_HURT => {
            MiniHoppingSkullGroundWaitCollisionRoute::DamageSkull
        }
        MINI_SKULL_COLID_GROUND_KIDLEFT | MINI_SKULL_COLID_GROUND_KIDRIGHT => {
            mini_hopping_skull_left_ground_side_contact_loc_3ec90(berzerker_charging)
        }
        MINI_SKULL_COLID_GROUND_KIDBELOW => {
            MiniHoppingSkullGroundWaitCollisionRoute::QueueScreamWorker
        }
        _ => MiniHoppingSkullGroundWaitCollisionRoute::ContinueWait,
    }
}

#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn mini_hopping_skull_left_hop_windup_collision_loc_3eccc(
    collision_type: u16,
    berzerker_charging: bool,
) -> MiniHoppingSkullGroundWaitCollisionRoute {
    // ASM Mini_Hopping_Skull.asm loc_3ECCC: the left-hop windup loop clears
    // `collision_type(a3)`, hibernates one frame, polls `sub_3F29A`, then
    // dispatches by ordered Kid/hurt collision comparisons before the DBF tail.
    match collision_type {
        0 => MiniHoppingSkullGroundWaitCollisionRoute::ContinueWait,
        MINI_SKULL_COLID_GROUND_KIDABOVE | MINI_SKULL_COLID_GROUND_HURT => {
            MiniHoppingSkullGroundWaitCollisionRoute::DamageSkull
        }
        MINI_SKULL_COLID_GROUND_KIDLEFT | MINI_SKULL_COLID_GROUND_KIDRIGHT => {
            mini_hopping_skull_left_hop_side_contact_loc_3ed14(berzerker_charging)
        }
        MINI_SKULL_COLID_GROUND_KIDBELOW => {
            MiniHoppingSkullGroundWaitCollisionRoute::QueueScreamWorker
        }
        _ => MiniHoppingSkullGroundWaitCollisionRoute::ContinueWait,
    }
}

#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn mini_hopping_skull_scream_worker_allocation_loc_3eb40(
    route: MiniHoppingSkullGroundWaitCollisionRoute,
) -> Option<MiniHoppingSkullScreamWorkerAllocation> {
    if route != MiniHoppingSkullGroundWaitCollisionRoute::QueueScreamWorker {
        return None;
    }

    // ASM Mini_Hopping_Skull.asm loc_3EB40: exchange a0/a4, allocate with
    // selector `$A000`, store `sub_3F3F0` at object slot +4, then restore a0/a4.
    // `sub_3F3F0` hibernates for `$F` frames before playing the scream SFX.
    Some(MiniHoppingSkullScreamWorkerAllocation {
        allocation_selector: 0xA000,
        routine: "sub_3F3F0",
        delay_frames: 15,
    })
}

#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn mini_hopping_skull_right_hop_scream_worker_allocation_loc_3ebc4(
    route: MiniHoppingSkullGroundWaitCollisionRoute,
) -> Option<MiniHoppingSkullScreamWorkerAllocation> {
    if route != MiniHoppingSkullGroundWaitCollisionRoute::QueueScreamWorker {
        return None;
    }

    // ASM Mini_Hopping_Skull.asm loc_3EBC4: the right-hop windup contact path
    // mirrors the grounded contact worker allocation: swap a0/a4, allocate with
    // selector `$A000`, install `sub_3F3F0` at slot +4, then restore a0/a4.
    Some(MiniHoppingSkullScreamWorkerAllocation {
        allocation_selector: 0xA000,
        routine: "sub_3F3F0",
        delay_frames: 15,
    })
}

#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn mini_hopping_skull_left_ground_scream_worker_allocation_loc_3ec7a(
    route: MiniHoppingSkullGroundWaitCollisionRoute,
) -> Option<MiniHoppingSkullScreamWorkerAllocation> {
    if route != MiniHoppingSkullGroundWaitCollisionRoute::QueueScreamWorker {
        return None;
    }

    // ASM Mini_Hopping_Skull.asm loc_3EC7A: the left-grounded contact path
    // mirrors the other grounded worker allocation: swap a0/a4, allocate with
    // selector `$A000`, install `sub_3F3F0` at slot +4, then restore a0/a4.
    Some(MiniHoppingSkullScreamWorkerAllocation {
        allocation_selector: 0xA000,
        routine: "sub_3F3F0",
        delay_frames: 15,
    })
}

#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn mini_hopping_skull_left_hop_scream_worker_allocation_loc_3ecfe(
    route: MiniHoppingSkullGroundWaitCollisionRoute,
) -> Option<MiniHoppingSkullScreamWorkerAllocation> {
    if route != MiniHoppingSkullGroundWaitCollisionRoute::QueueScreamWorker {
        return None;
    }

    // ASM Mini_Hopping_Skull.asm loc_3ECFE: the left-hop windup contact path
    // swaps a0/a4, allocates selector `$A000`, installs `sub_3F3F0` at slot +4,
    // then restores a0/a4 before the DBF tail.
    Some(MiniHoppingSkullScreamWorkerAllocation {
        allocation_selector: 0xA000,
        routine: "sub_3F3F0",
        delay_frames: 15,
    })
}

#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn mini_hopping_skull_stopped_wait_setup_loc_3f000() -> MiniHoppingSkullStoppedWaitSetup
{
    // ASM Mini_Hopping_Skull.asm loc_3EFEC -> loc_3F000: stop motion,
    // disable animation/collision, choose `LnkTo_unk_C8070`, and seed d1
    // with `$14`; the following DBF loop waits 21 hibernated frames.
    MiniHoppingSkullStoppedWaitSetup {
        is_moved: false,
        is_animated: false,
        has_level_collision: false,
        x_velocity: 0,
        y_velocity: 0,
        sprite: "LnkTo_unk_C8070",
        dbf_counter: 0x0014,
        wait_frames: 21,
    }
}

#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn mini_hopping_skull_stopped_wait_side_contact_loc_3f052(
    berzerker_charging: bool,
) -> MiniHoppingSkullGroundWaitCollisionRoute {
    // ASM Mini_Hopping_Skull.asm loc_3F052: side contact tests the global
    // `Berzerker_charging` flag; clear queues the `sub_3F3F0` scream worker
    // via loc_3F03C, set jumps to the shared Berzerker kill path at loc_3F262.
    if berzerker_charging {
        MiniHoppingSkullGroundWaitCollisionRoute::BerzerkerChargeKillsSkull
    } else {
        MiniHoppingSkullGroundWaitCollisionRoute::QueueScreamWorker
    }
}

#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn mini_hopping_skull_stopped_wait_damage_sequence_loc_3f05c(
    hit_counter: u16,
) -> MiniHoppingSkullDamageSequence {
    // ASM Mini_Hopping_Skull.asm loc_3F05C: stopped-wait damage subtracts
    // one from `$44(a3)` as a word, branches to the death path on zero, otherwise
    // plays `stru_3E9C2` and hibernates until that animation finishes.
    mini_hopping_skull_damage_sequence_loc_3eb60(hit_counter)
}

#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn mini_hopping_skull_stopped_wait_collision_loc_3f00a(
    collision_type: u16,
    berzerker_charging: bool,
) -> MiniHoppingSkullGroundWaitCollisionRoute {
    // ASM Mini_Hopping_Skull.asm loc_3F00A: each stopped-wait frame clears
    // `collision_type(a3)`, hibernates, runs `sub_3F29A`, then compares Kid
    // and hurt collision IDs in order before the DBF tail at loc_3F074.
    match collision_type {
        0 => MiniHoppingSkullGroundWaitCollisionRoute::ContinueWait,
        MINI_SKULL_COLID_GROUND_KIDABOVE | MINI_SKULL_COLID_GROUND_HURT => {
            MiniHoppingSkullGroundWaitCollisionRoute::DamageSkull
        }
        MINI_SKULL_COLID_GROUND_KIDLEFT | MINI_SKULL_COLID_GROUND_KIDRIGHT => {
            mini_hopping_skull_stopped_wait_side_contact_loc_3f052(berzerker_charging)
        }
        MINI_SKULL_COLID_GROUND_KIDBELOW => {
            MiniHoppingSkullGroundWaitCollisionRoute::QueueScreamWorker
        }
        _ => MiniHoppingSkullGroundWaitCollisionRoute::ContinueWait,
    }
}

#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn mini_hopping_skull_stopped_wait_scream_worker_allocation_loc_3f03c(
    route: MiniHoppingSkullGroundWaitCollisionRoute,
) -> Option<MiniHoppingSkullScreamWorkerAllocation> {
    if route != MiniHoppingSkullGroundWaitCollisionRoute::QueueScreamWorker {
        return None;
    }

    // ASM Mini_Hopping_Skull.asm loc_3F03C: swap a0/a4, allocate selector
    // `$A000`, install `sub_3F3F0` at slot +4, restore a0/a4, then rejoin
    // the stopped-wait DBF tail.
    Some(MiniHoppingSkullScreamWorkerAllocation {
        allocation_selector: 0xA000,
        routine: "sub_3F3F0",
        delay_frames: 15,
    })
}

#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn mini_hopping_skull_facing_matched_pause_setup_loc_3f09e()
-> MiniHoppingSkullFacingMatchedPauseSetup {
    // ASM Mini_Hopping_Skull.asm loc_3F09E: once the stopped-wait tail finds
    // the skull already facing Kid, it selects `LnkTo_unk_C8078` and seeds
    // d1 with `$A`, making the following DBF loop last 11 frames.
    MiniHoppingSkullFacingMatchedPauseSetup {
        sprite: "LnkTo_unk_C8078",
        dbf_counter: 0x000A,
        wait_frames: 11,
    }
}

#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn mini_hopping_skull_facing_matched_pause_collision_loc_3f0a8(
    collision_type: u16,
    berzerker_charging: bool,
) -> MiniHoppingSkullGroundWaitCollisionRoute {
    // ASM Mini_Hopping_Skull.asm loc_3F0A8: the matched-facing pause loop
    // clears `collision_type(a3)`, hibernates one frame, polls `sub_3F29A`,
    // then checks no-collision, above, hurt, side, and below contact in order.
    match collision_type {
        0 => MiniHoppingSkullGroundWaitCollisionRoute::ContinueWait,
        MINI_SKULL_COLID_GROUND_KIDABOVE | MINI_SKULL_COLID_GROUND_HURT => {
            MiniHoppingSkullGroundWaitCollisionRoute::DamageSkull
        }
        MINI_SKULL_COLID_GROUND_KIDLEFT | MINI_SKULL_COLID_GROUND_KIDRIGHT => {
            if berzerker_charging {
                MiniHoppingSkullGroundWaitCollisionRoute::BerzerkerChargeKillsSkull
            } else {
                MiniHoppingSkullGroundWaitCollisionRoute::QueueScreamWorker
            }
        }
        MINI_SKULL_COLID_GROUND_KIDBELOW => {
            MiniHoppingSkullGroundWaitCollisionRoute::QueueScreamWorker
        }
        _ => MiniHoppingSkullGroundWaitCollisionRoute::ContinueWait,
    }
}

#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn mini_hopping_skull_facing_matched_pause_scream_worker_allocation_loc_3f0da(
    route: MiniHoppingSkullGroundWaitCollisionRoute,
) -> Option<MiniHoppingSkullScreamWorkerAllocation> {
    if route != MiniHoppingSkullGroundWaitCollisionRoute::QueueScreamWorker {
        return None;
    }

    // ASM Mini_Hopping_Skull.asm loc_3F0DA: matched-facing pause contact swaps
    // object pointers, allocates with selector `$A000`, installs `sub_3F3F0`
    // at slot +4, restores a0/a4, then rejoins the pause tail.
    Some(MiniHoppingSkullScreamWorkerAllocation {
        allocation_selector: 0xA000,
        routine: "sub_3F3F0",
        delay_frames: 15,
    })
}

#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn mini_hopping_skull_stopped_wait_tail_loc_3f090(
    dbf_counter: u16,
    kid_x_word: u16,
    skull_x_word: u16,
    x_direction: u8,
) -> MiniHoppingSkullStoppedWaitTail {
    // ASM Mini_Hopping_Skull.asm loc_3F090: after the stopped-wait DBF falls
    // through, Kid to the right enters this branch; a clear `x_direction(a3)`
    // is set to `$FF` and the stopped wait restarts, while non-zero direction
    // means the skull already faces Kid and can continue to the next pause.
    let next_dbf_counter = dbf_counter.wrapping_sub(1);
    if next_dbf_counter != 0xFFFF {
        return MiniHoppingSkullStoppedWaitTail::ContinueWait { next_dbf_counter };
    }

    if (kid_x_word as i16) > (skull_x_word as i16) {
        if x_direction == 0 {
            MiniHoppingSkullStoppedWaitTail::RestartStoppedWaitFacingRight {
                x_direction: 0xFF,
                dbf_counter: 0x0014,
                wait_frames: 21,
            }
        } else {
            let setup = mini_hopping_skull_facing_matched_pause_setup_loc_3f09e();
            MiniHoppingSkullStoppedWaitTail::BeginFacingMatchedPause {
                sprite: setup.sprite,
                dbf_counter: setup.dbf_counter,
                wait_frames: setup.wait_frames,
            }
        }
    } else if x_direction == 0 {
        let setup = mini_hopping_skull_facing_matched_pause_setup_loc_3f09e();
        MiniHoppingSkullStoppedWaitTail::BeginFacingMatchedPause {
            sprite: setup.sprite,
            dbf_counter: setup.dbf_counter,
            wait_frames: setup.wait_frames,
        }
    } else {
        MiniHoppingSkullStoppedWaitTail::RestartStoppedWaitFacingLeft {
            x_direction: 0x00,
            dbf_counter: 0x0014,
            wait_frames: 21,
        }
    }
}

#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn mini_hopping_skull_right_hop_relaunch_tail_loc_3ec26() -> MiniHoppingSkullRelaunchTail
{
    // ASM Mini_Hopping_Skull.asm loc_3EC26: after loc_3EBFC has seeded the
    // rightward hop velocities, this shared tail sets `is_moved(a3)` and
    // `has_level_collision(a3)`, then branches back to the airborne loop.
    MiniHoppingSkullRelaunchTail {
        is_moved: true,
        has_level_collision: true,
        next_loop: "loc_3EA88",
    }
}

#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn mini_hopping_skull_left_hop_relaunch_tail_loc_3ed60() -> MiniHoppingSkullRelaunchTail
{
    // ASM Mini_Hopping_Skull.asm loc_3ED60: after the preceding tail has seeded the
    // leftward hop velocities, this tail sets `is_moved(a3)` and
    // `has_level_collision(a3)`, then branches back to loc_3EA88.
    MiniHoppingSkullRelaunchTail {
        is_moved: true,
        has_level_collision: true,
        next_loop: "loc_3EA88",
    }
}

pub(crate) fn step_hopper(
    enemy: &mut ActiveEnemy,
    block_grid: Option<&terrain::BlockGrid>,
    player_x_pos: Option<u32>,
) {
    let Some(grid) = block_grid else {
        return;
    };
    let hc = hopper_constants(enemy.family);

    let cx = (enemy.x_pos >> 16) as i32;
    let cy = (enemy.y_pos >> 16) as i32;
    if enemy.on_ground {
        let mut facing =
            player_relative_direction(enemy.x_pos, player_x_pos).unwrap_or(enemy.facing);
        if hopper_should_reverse(facing, grid, cx, cy) {
            facing = opposite_direction(facing);
        }
        enemy.facing = facing;
        if enemy.grounded_frames > 0 {
            enemy.grounded_frames -= 1;
            enemy.x_velocity = 0;
            return;
        }
        let mut jx = hc.jump_x_speed;
        let mut jy = hc.jump_y_speed;
        // ASM Mini_Hopping_Skull.asm loc_3EBFC/loc_3ED36/loc_3EE7C/loc_3EFB6:
        // enemy_level >= 1 (HP > 1) adds `$4000` to X speed and `$10000` extra
        // upward Y speed for either facing.
        if matches!(enemy.family, EnemyFamily::MiniHoppingSkull) && enemy.hit_points > 1 {
            jx += 0x4000;
            jy -= 0x1_0000;
        }
        enemy.x_velocity = horizontal_velocity(enemy.facing, jx);
        enemy.y_velocity = jy;
        enemy.on_ground = false;
    }

    // ASM Mini_Hopping_Skull.asm loc_3EA88: after the hibernated frame and
    // retention check, `add.l d3,y_vel(a3)` applies `$1200` gravity before the
    // collision-type dispatch.
    enemy.y_velocity = (enemy.y_velocity + hc.gravity).min(physics::MAX_FALL_SPEED);
    enemy.x_pos = (enemy.x_pos as i64 + i64::from(enemy.x_velocity)) as u32;
    enemy.y_pos = (enemy.y_pos as i64 + i64::from(enemy.y_velocity)) as u32;

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
        // ASM Mini_Hopping_Skull.asm loc_3F1AA flips x_direction and negates
        // the live X velocity; it does not reset to the base hop speed.
        reverse_enemy_direction(enemy);
        let wall_fp = (wall.wall_x_px as u32) << 16;
        let edge_fp = (ENEMY_PROBE_HALF_WIDTH as u32) << 16;
        if moving_right {
            if enemy.x_pos + edge_fp > wall_fp {
                enemy.x_pos = wall_fp.saturating_sub(edge_fp);
            }
        } else if enemy.x_pos < wall_fp + edge_fp {
            enemy.x_pos = wall_fp + edge_fp;
        }
        enemy.x_velocity = -enemy.x_velocity;
    }

    let floor = terrain::probe_floor(grid, cx, cy);
    if floor.on_ground {
        let floor_y_fp = (floor.floor_y_px as u32) << 16;
        if enemy.y_velocity >= 0 && enemy.y_pos >= floor_y_fp {
            if floor.cell.collision == terrain::CollisionKind::SlopeUp {
                enemy.x_pos = mini_hopping_skull_snap_slopeup_x(
                    grid,
                    (enemy.x_pos >> 16) as i32,
                    (enemy.y_pos >> 16) as i32,
                );
            } else if floor.cell.collision == terrain::CollisionKind::SlopeDown {
                enemy.x_pos = mini_hopping_skull_snap_slopedown_x(
                    grid,
                    (enemy.x_pos >> 16) as i32,
                    (enemy.y_pos >> 16) as i32,
                );
            }
            enemy.y_pos = floor_y_fp;
            enemy.x_velocity = 0;
            enemy.y_velocity = 0;
            enemy.on_ground = true;
            enemy.grounded_frames = hc.ground_delay;
        }
    } else {
        enemy.on_ground = false;
    }
}

pub(crate) fn hopper_should_reverse(
    facing: Direction,
    grid: &terrain::BlockGrid,
    cx: i32,
    cy: i32,
) -> bool {
    let moving_right = matches!(facing, Direction::Right);
    let wall = terrain::probe_wall(
        grid,
        cx,
        cy,
        ENEMY_PROBE_HALF_WIDTH,
        ENEMY_PROBE_HEIGHT,
        moving_right,
    );
    if wall.hit {
        return true;
    }
    // Reverse at platform edge: look ahead for floor support
    let look_ahead_x = if moving_right {
        cx + ENEMY_PROBE_HALF_WIDTH
    } else {
        cx - ENEMY_PROBE_HALF_WIDTH
    };
    !terrain::probe_floor(grid, look_ahead_x, cy).on_ground
}

/// Step a Pacer-archetype enemy: walk at a fixed speed, reverse on wall or
/// floor edge. Matches the shared walker subroutines in the disassembly
/// (loc_3E392 / loc_3E546 / loc_3E560 / loc_3E3AC).
pub(crate) fn step_pacer(enemy: &mut ActiveEnemy, block_grid: Option<&terrain::BlockGrid>) {
    let Some(grid) = block_grid else {
        return;
    };

    let walk_speed = if enemy.walk_speed_override != 0 {
        enemy.walk_speed_override
    } else {
        pacer_walk_speed(enemy.family, enemy.hit_points)
    };
    enemy.x_velocity = horizontal_velocity(enemy.facing, walk_speed);
    enemy.y_velocity = 0;

    // Apply horizontal movement.
    // ASM loc_3C1DE (kid.asm:43684-43686): no-collision X path in the leftward
    // walker loop — move.l d2,x_pos(a3) stores the predicted position when the
    // displacement doesn't cross a tile boundary (bcc.s) or d6==0 (no floor
    // status). The rightward mirror is loc_3C0C6. Native stores the predicted
    // x_pos unconditionally then corrects via probe_wall if a collision is found.
    enemy.x_pos = (enemy.x_pos as i64 + i64::from(enemy.x_velocity)) as u32;

    let cx = (enemy.x_pos >> 16) as i32;
    let cy = (enemy.y_pos >> 16) as i32;
    let moving_right = enemy.x_velocity >= 0;

    // Wall probe: reverse if we hit a wall.
    let wall = terrain::probe_wall(
        grid,
        cx,
        cy,
        ENEMY_PROBE_HALF_WIDTH,
        ENEMY_PROBE_HEIGHT,
        moving_right,
    );
    if wall.hit {
        // ASM loc_3C026 (rightward walkers) / loc_3C0D2 (leftward walkers):
        // clr.l x_vel(a3); clr.l y_vel(a3); st/sf x_direction(a3) — stop
        // and face the opposite direction after wall collision. The ASM also
        // bumps a byte counter at $5A(a5) by 4 and deactivates (loc_3C464)
        // if the counter wraps to zero (i.e. after 64 turnarounds). Native
        // handles deactivation via the retain/cull check each frame instead.
        // Push out of wall.
        let wall_fp = (wall.wall_x_px as u32) << 16;
        let edge_fp = (ENEMY_PROBE_HALF_WIDTH as u32) << 16;
        if moving_right {
            if enemy.x_pos + edge_fp > wall_fp {
                enemy.x_pos = wall_fp.saturating_sub(edge_fp);
            }
        } else if enemy.x_pos < wall_fp + edge_fp {
            enemy.x_pos = wall_fp + edge_fp;
        }
        reverse_enemy_direction(enemy);
        enemy.x_velocity = horizontal_velocity(enemy.facing, walk_speed);
        return;
    }

    // ASM loc_3C04E (kid.asm:43511-43542) / loc_3C0FA (kid.asm:43585-43609):
    // rightward/leftward walker terrain dispatch after sub_3C4F8 + sub_36972/
    // sub_36A58. The ASM computes displacement, aligns to 16px grid, and checks
    // d7 flags (bit 0=floor edge, bit 1=wall, bit 2=hurt) plus d6 (floor
    // status) to decide: continue walking (loc_3C17A/loc_3C266), snap position
    // (loc_3C0C6/loc_3C172), or turnaround (loc_3C0A6→loc_3C0D2 /
    // loc_3C152→loc_3C026). Native collapses these into probe_floor.
    //
    // ASM loc_3C19C (kid.asm:43657-43660): secondary body collision scan entry
    // in the leftward walking loop — `bsr.w sub_36B3C; tst.w d5; bne.w
    // loc_3C464`. After loc_3C194 hibernate + remain-active, sub_36B3C scans
    // the walker's body rectangle for overlapping collision cells; if d5
    // (scan result) is nonzero, the walker deactivates. Native covers this
    // via the probe_wall check above.
    //
    // ASM loc_3C0A6 (kid.asm:43545-43549): btst #0,d7 — floor edge bit set →
    // st $6E(a5), bra loc_3C0D2 turnaround. Native: !probe_floor().on_ground.
    //
    // ASM loc_3C0B4 (kid.asm:43552-43558): no floor edge bit, but d6 >= 0
    // (no solid floor) and bit 1 not set (no wall) → st $6E(a5), bra loc_3C0D2
    // turnaround. This catches the "cliff with no wall to cling to" case.
    // Native: also covered by the probe_floor edge check below.
    //
    // ASM loc_3C1E4 (kid.asm:43688-43696): Y-axis movement entry in the
    // leftward walker loop — loads y_pos, adds y_vel, computes pixel
    // displacement, branches to loc_3C220 when >= 0 (moving down). Native
    // doesn't apply Y velocity to walkers; probe_floor keeps them grounded.
    //
    // ASM loc_3C220 (kid.asm:43713-43725): downward Y movement floor check —
    // sub_36C6A scans for floor; if found, snaps y_pos and branches to
    // loc_3C026 turnaround. If no floor, stores predicted y_pos at loc_3C242
    // and gravity continues. Native: probe_floor edge detection reverses the
    // walker at cliff edges, preventing the fall state entirely.
    //
    // ASM loc_3C276 (kid.asm:43751-43754): falling walker loop top —
    // tst.w $3A(a5); beq.s loc_3C280; bsr.w sub_3BF72. Identical attachment
    // check to loc_3C18A/loc_3C03A but in the falling walker loop (entered
    // via loc_3C266 when a walker walks off an edge). When $3A(a5) is nonzero
    // (child present), sub_3BF72 syncs the child; when zero, skips to
    // loc_3C280. Native: walkers never enter the fall state because
    // probe_floor reverses them at cliff edges.
    //
    // ASM loc_3C280 (kid.asm:43756-43758): falling walker hibernate +
    // remain-active — jsr (j_Hibernate_Object_1Frame).w; bsr.w sub_3C3CE.
    // Same hibernate + remain-active as loc_3C194/loc_3C044 but in the
    // falling walker loop. Falls through to loc_3C288. Native: collapsed
    // into the per-frame step_pacer cycle; walkers don't fall.
    //
    // ASM loc_3C288 (kid.asm:43760-43763): falling walker body collision
    // scan — bsr.w sub_36BD6; tst.w d5; bne.w loc_3C464 deactivation. If
    // no collision, performs X movement with wall checks (loc_3C2CA snap),
    // Y movement with floor/ceiling checks and gravity (addi.l #$2000,
    // capped at $80000). Native: walkers are always grounded; the falling
    // loop's body scan + gravity are unreachable.
    //
    // ASM loc_3C2D0 (kid.asm:43791-43813): falling walker Y-axis movement
    // entry — move.l y_pos(a3),d0; add.l y_vel(a3),d1; sub.w d0,d1;
    // bge.s loc_3C30C. When displacement < 0 (moving upward), checks
    // ceiling via sub_36CB8; if ceiling found, snaps y_pos and clears
    // y_vel. When displacement >= 0, falls through to loc_3C30C (floor
    // check). Native: unreachable — walkers never enter fall state.
    //
    // ASM loc_3C30C (kid.asm:43816-43828): falling walker downward Y floor
    // check — sub_36C6A floor scan; if found, snaps y_pos and branches to
    // loc_3C0D2 (leftward turnaround). If no floor, falls through to
    // loc_3C32E. Native: unreachable — walkers reverse at cliff edges
    // before falling, so the in-air floor search never runs.
    //
    // ASM loc_3C32E (kid.asm:43831-43832): falling walker no-Y-collision
    // path — move.l d2,y_pos(a3) stores predicted position. Falls through
    // to loc_3C332 gravity (addi.l #$2000, capped $80000), then loops
    // back to loc_3C276. Native: unreachable — probe_floor cliff-edge
    // reversal prevents the falling walker loop entirely.
    //
    // ASM loc_3C34A (kid.asm:43841-43843): falling walker gravity cap
    // convergence — move.l d0,y_vel(a3); bra.w loc_3C276. Stores the
    // (potentially capped) y_vel and loops back to the falling walker
    // loop top. Native: unreachable — walkers never enter fall state.
    let look_ahead_x = if moving_right {
        cx + ENEMY_PROBE_HALF_WIDTH
    } else {
        cx - ENEMY_PROBE_HALF_WIDTH
    };
    if !terrain::probe_floor(grid, look_ahead_x, cy).on_ground {
        reverse_enemy_direction(enemy);
        enemy.x_velocity = horizontal_velocity(enemy.facing, walk_speed);
    }
}

/// Driller probe half-width for block drilling (Driller.asm line 27: move.w #$C,$48(a5)).
/// ASM stores 12px ($C) in the Driller sprite's collision half-width field.
pub(crate) const DRILLER_PROBE_HALF_WIDTH: i32 = 12;

/// Step a Driller enemy: paces like normal, but drills through Smash and Ice blocks.
/// ASM loc_3BE78: when sprite timer matches direction (8 for left, 4 for right),
/// checks cell at (sprite_x ± offset, sprite_y - 16) for Smash/Ice type and destroys it.
/// Returns list of (col, row) cells to destroy this frame.
pub(crate) fn step_driller(
    enemy: &mut ActiveEnemy,
    block_grid: Option<&terrain::BlockGrid>,
) -> Vec<(u16, u16)> {
    let mut drilled = Vec::new();
    let Some(grid) = block_grid else {
        return drilled;
    };

    // Pace first
    step_pacer(enemy, block_grid);

    // Now check for drillable blocks in the walking direction.
    // ASM loc_3BE8E: get sprite position, check timer, probe cell 16px above.
    // Simplified: drill every frame when walking into Smash/Ice blocks.
    let cx = (enemy.x_pos >> 16) as i32;
    let cy = (enemy.y_pos >> 16) as i32;
    let probe_x = if enemy.facing == Direction::Right {
        cx + DRILLER_PROBE_HALF_WIDTH
    } else {
        cx - DRILLER_PROBE_HALF_WIDTH
    };
    // ASM sub_3BEF2: Y probe is 16px above sprite (subi.w #$10,d7).
    let probe_y = cy - 16;

    let col = terrain::pixel_to_cell(probe_x).max(0) as u16;
    let row = terrain::pixel_to_cell(probe_y).max(0) as u16;
    let cell = grid.cell_at(col, row);

    // ASM loc_3BEF2: check block type. $100 = Smash, $200 = Ice.
    // Any block with collision Solid + has_block + block_type Smash/Ice.
    if cell.has_block
        && cell.collision == terrain::CollisionKind::Solid
        && (cell.block_type == terrain::BlockType::Smash
            || cell.block_type == terrain::BlockType::Ice)
    {
        drilled.push((col, row));
    }

    drilled
}

/// Ninja detection cone: player must be 0x20–0x50 pixels above and in the
/// half-open X range [-0x10, +0x10) (Ninja.asm loc_3D706).
pub(crate) const NINJA_DETECT_Y_MIN: i32 = 0x20;
pub(crate) const NINJA_DETECT_Y_MAX: i32 = 0x50;
pub(crate) const NINJA_DETECT_X_RANGE: i32 = 0x10;

fn ninja_detects_player_while_stealthed(dx_px: i64, dy_px: i64) -> bool {
    // ASM loc_3D706: attack only when kid_y - ninja_y is in [-$50, -$20] and
    // kid_x - ninja_x is in [-$10, +$10). The right edge is exclusive because
    // `cmpi.w #$10,d4` falls through only on `d4 < $10`.
    dy_px >= NINJA_DETECT_Y_MIN as i64
        && dy_px <= NINJA_DETECT_Y_MAX as i64
        && dx_px >= -(NINJA_DETECT_X_RANGE as i64)
        && dx_px < NINJA_DETECT_X_RANGE as i64
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) enum NinjaKidThreatRouteLoc3D734 {
    /// No close Kid-driven guard/contact response.
    ProjectileThreatCheck,
    /// Enter the blocking/guard pose facing d4.
    GuardPose,
    /// Berzerker charging takes the kill path.
    BerzerkerKillPath,
}

#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn ninja_kid_threat_route_loc_3d734(
    kid_minus_ninja_x_px: i16,
    kid_minus_ninja_y_px: i16,
    player_form: PlayerForm,
    berzerker_charging: bool,
) -> NinjaKidThreatRouteLoc3D734 {
    // ASM Ninja.asm loc_3D734: d4 is `kid_x - ninja_x`. The broad +/-$50
    // band gates Berzerker's charge kill path; Red Stealth blocking then
    // narrows to +/-$30 and requires Kid to be no more than 4 px above.
    if !(-0x50..=0x50).contains(&kid_minus_ninja_x_px) {
        return NinjaKidThreatRouteLoc3D734::ProjectileThreatCheck;
    }

    if berzerker_charging {
        return NinjaKidThreatRouteLoc3D734::BerzerkerKillPath;
    }

    if !(-0x30..=0x30).contains(&kid_minus_ninja_x_px) || kid_minus_ninja_y_px < -4 {
        return NinjaKidThreatRouteLoc3D734::ProjectileThreatCheck;
    }

    if player_form == PlayerForm::RedStealth {
        NinjaKidThreatRouteLoc3D734::GuardPose
    } else {
        NinjaKidThreatRouteLoc3D734::ProjectileThreatCheck
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) enum NinjaProjectileThreatRouteLoc3D776 {
    /// No active Maniaxe projectile threat; continue normal post-threat handling.
    ContinuePostThreat,
    /// Enter the blocking/guard pose facing d4.
    GuardPose,
}

#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn ninja_projectile_threat_route_loc_3d776(
    projectile_minus_ninja_x_px: Option<i16>,
    player_form: PlayerForm,
) -> NinjaProjectileThreatRouteLoc3D776 {
    // ASM Ninja.asm loc_3D776: require a live Kid projectile, Current_Helmet
    // Maniaxe, and projectile_x - ninja_x in the half-open [-$5A, +$5A) band.
    let Some(projectile_dx) = projectile_minus_ninja_x_px else {
        return NinjaProjectileThreatRouteLoc3D776::ContinuePostThreat;
    };

    if player_form == PlayerForm::Maniaxe && (-0x5A..0x5A).contains(&projectile_dx) {
        NinjaProjectileThreatRouteLoc3D776::GuardPose
    } else {
        NinjaProjectileThreatRouteLoc3D776::ContinuePostThreat
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) enum NinjaPostThreatRouteLoc3D79C {
    /// Keep the current guard/pacing branch.
    ResumePacing,
    /// Clear the guard state, restore walk animation/speed, and resume moving.
    ResetStealthPacing,
    /// Positive `$4C(a5)` state branches to the airborne proximity check.
    PositiveStateCheck,
}

#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn ninja_post_threat_route_loc_3d79c(
    threat_state_4c: i8,
    kid_projectile_present: bool,
    kid_minus_ninja_x_px: i16,
    player_form: PlayerForm,
) -> NinjaPostThreatRouteLoc3D79C {
    // ASM Ninja.asm loc_3D79C: `$4C(a5)` is a signed byte. Zero keeps the
    // normal branch, positive delegates to the airborne proximity check, and
    // negative guard state persists while a Kid projectile exists or Red
    // Stealth remains inside the inclusive +/-$30 Kid X band.
    if threat_state_4c == 0 {
        return NinjaPostThreatRouteLoc3D79C::ResumePacing;
    }

    if threat_state_4c > 0 {
        return NinjaPostThreatRouteLoc3D79C::PositiveStateCheck;
    }

    if kid_projectile_present {
        return NinjaPostThreatRouteLoc3D79C::ResumePacing;
    }

    if (-0x30..=0x30).contains(&kid_minus_ninja_x_px) && player_form == PlayerForm::RedStealth {
        NinjaPostThreatRouteLoc3D79C::ResumePacing
    } else {
        NinjaPostThreatRouteLoc3D79C::ResetStealthPacing
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) enum NinjaResetStealthPacingRouteLoc3D7CA {
    Loc3E546,
    Loc3E392,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) struct NinjaResetStealthPacingStateLoc3D7CA {
    pub(crate) threat_state_4c: i8,
    pub(crate) animation_script: &'static str,
    pub(crate) is_animated: bool,
    pub(crate) walk_speed_override: i32,
    pub(crate) next: NinjaResetStealthPacingRouteLoc3D7CA,
}

#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn ninja_reset_stealth_pacing_loc_3d7ca(
    saved_walk_speed_override_5e: i32,
    x_direction_right: bool,
) -> NinjaResetStealthPacingStateLoc3D7CA {
    // ASM Ninja.asm loc_3D7CA: `sf $4C(a5)`, initialise `stru_3D4EA`,
    // `st is_animated(a3)`, restore `$5E(a5)` into `$50(a5)`, then dispatch
    // clear/set x_direction to loc_3E546/loc_3E392.
    NinjaResetStealthPacingStateLoc3D7CA {
        threat_state_4c: 0,
        animation_script: "stru_3D4EA",
        is_animated: true,
        walk_speed_override: saved_walk_speed_override_5e,
        next: if x_direction_right {
            NinjaResetStealthPacingRouteLoc3D7CA::Loc3E392
        } else {
            NinjaResetStealthPacingRouteLoc3D7CA::Loc3E546
        },
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) enum NinjaResumePacingRouteLoc3D7EE {
    Loc3E560,
    Loc3E3AC,
}

#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn ninja_resume_pacing_route_loc_3d7ee(
    x_direction_right: bool,
) -> NinjaResumePacingRouteLoc3D7EE {
    // ASM Ninja.asm loc_3D7EE: `tst.b x_direction(a3); beq.w loc_3E560;
    // bra.w loc_3E3AC`. Clear x_direction keeps the left-facing loop, set
    // x_direction resumes the right-facing loop.
    if x_direction_right {
        NinjaResumePacingRouteLoc3D7EE::Loc3E3AC
    } else {
        NinjaResumePacingRouteLoc3D7EE::Loc3E560
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) enum NinjaPositiveStateRouteLoc3D7FA {
    ResetStealthPacing {
        next: NinjaResetStealthPacingRouteLoc3D7CA,
    },
    ResumePacing {
        next: NinjaResumePacingRouteLoc3D7EE,
    },
}

#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn ninja_positive_state_route_loc_3d7fa(
    kid_minus_ninja_x_px: i16,
    kid_minus_ninja_y_px: i16,
    x_direction_right: bool,
) -> NinjaPositiveStateRouteLoc3D7FA {
    // ASM Ninja.asm loc_3D7FA: require Kid to be above the ninja with
    // `kid_y - ninja_y` in [-$60, 0) and `kid_x - ninja_x` in inclusive
    // [-$10, +$10]. Outside that box branches to loc_3D7CA; inside it branches
    // to loc_3D7EE.
    if !(-0x60..0).contains(&kid_minus_ninja_y_px)
        || !(-0x10..=0x10).contains(&kid_minus_ninja_x_px)
    {
        let reset = ninja_reset_stealth_pacing_loc_3d7ca(0, x_direction_right);
        return NinjaPositiveStateRouteLoc3D7FA::ResetStealthPacing { next: reset.next };
    }

    NinjaPositiveStateRouteLoc3D7FA::ResumePacing {
        next: ninja_resume_pacing_route_loc_3d7ee(x_direction_right),
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) enum NinjaThreatPoseRoute {
    Loc3D79C,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) struct NinjaThreatPoseState {
    pub(crate) threat_state_4c: i8,
    pub(crate) is_animated: bool,
    pub(crate) sprite_link: &'static str,
    pub(crate) walk_speed_override: i32,
    pub(crate) x_direction_right: bool,
    pub(crate) next: NinjaThreatPoseRoute,
}

#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn ninja_guard_pose_loc_3d826(
    d4_kid_or_projectile_minus_ninja_x_px: i16,
) -> NinjaThreatPoseState {
    // ASM Ninja.asm loc_3D826: `st $4C(a5)`, clear animation, select
    // `LnkTo_unk_C7D8E`, zero `$50(a5)`, face by signed d4, then branch to
    // loc_3D79C. Negative d4 keeps `x_direction` clear; zero/positive sets it.
    NinjaThreatPoseState {
        threat_state_4c: -1,
        is_animated: false,
        sprite_link: "LnkTo_unk_C7D8E",
        walk_speed_override: 0,
        x_direction_right: d4_kid_or_projectile_minus_ninja_x_px >= 0,
        next: NinjaThreatPoseRoute::Loc3D79C,
    }
}

#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn ninja_jump_threat_pose_loc_3d84e(
    d4_kid_minus_ninja_x_px: i16,
) -> NinjaThreatPoseState {
    // ASM Ninja.asm loc_3D84E: clear animation, set `$4C(a5)` to positive 1,
    // select `LnkTo_unk_C7D86`, zero `$50(a5)`, face by signed d4, then branch
    // to loc_3D79C.
    NinjaThreatPoseState {
        threat_state_4c: 1,
        is_animated: false,
        sprite_link: "LnkTo_unk_C7D86",
        walk_speed_override: 0,
        x_direction_right: d4_kid_minus_ninja_x_px >= 0,
        next: NinjaThreatPoseRoute::Loc3D79C,
    }
}

/// Ninja jump-attack velocities (Ninja.asm lines 384-392).
pub(crate) const NINJA_JUMP_X_SPEED_LEFT: i32 = 0x1_8000;
pub(crate) const NINJA_JUMP_X_SPEED_RIGHT: i32 = 0x1_0000;
pub(crate) const NINJA_JUMP_Y_SPEED: i32 = -0x4_8000;
pub(crate) const NINJA_GRAVITY: i32 = 0x2800;
pub(crate) const NINJA_KNOCKDOWN_Y_SPEED_LOC_3D8D6: i32 = 0x0001_8000;
pub(crate) const NINJA_KNOCKDOWN_GRAVITY_LOC_3D8E6: i32 = 0x0000_1800;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) enum NinjaCollisionResumeLoc3D638 {
    /// ASM Ninja.asm loc_3E7CA: `beq.w loc_3E7CA` path (left-facing).
    Loc3E7CA,
    /// ASM Ninja.asm loc_3E6AE: `bra.w loc_3E6AE` path (right-facing).
    Loc3E6AE,
    /// ASM Ninja.asm loc_3E560: `beq.w loc_3E560` path (left-facing when `$5C(a5)` is set).
    Loc3E560,
    /// ASM Ninja.asm loc_3E3AC: `bra.w loc_3E3AC` path (right-facing when `$5C(a5)` is set).
    Loc3E3AC,
}

#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn ninja_collision_resume_loc_3d638(
    wall_pinned: bool,
    x_direction_right: bool,
) -> NinjaCollisionResumeLoc3D638 {
    // ASM Ninja.asm loc_3D638: clear collision, branch by `$5C(a5)` and
    // x_direction to one of `loc_3E7CA/3E6AE/3E560/3E3AC`.
    if wall_pinned {
        if x_direction_right {
            NinjaCollisionResumeLoc3D638::Loc3E3AC
        } else {
            NinjaCollisionResumeLoc3D638::Loc3E560
        }
    } else if x_direction_right {
        NinjaCollisionResumeLoc3D638::Loc3E6AE
    } else {
        NinjaCollisionResumeLoc3D638::Loc3E7CA
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) enum NinjaAirborneCollisionRouteLoc3D878 {
    Resume { next: NinjaCollisionResumeLoc3D638 },
    To3DB4E,
    To3D8BC,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) struct NinjaAirborneCollisionStateLoc3D878 {
    pub(crate) next_collision_type: u16,
    pub(crate) route: NinjaAirborneCollisionRouteLoc3D878,
}

#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn ninja_airborne_collision_dispatch_loc_3d878(
    collision_type: i16,
    x_direction_right: bool,
    player_form: PlayerForm,
    kid_sprite_allows_berzerker_knockdown: bool,
) -> NinjaAirborneCollisionStateLoc3D878 {
    // ASM Ninja.asm loc_3D878: capture and clear `collision_type(a3)`. Side
    // contact only enters loc_3D8BC when Current_Helmet is Berzerker and Kid's
    // `addroffset_sprite` is not after `LnkTo_unk_ADFC8`; otherwise loc_3D89C
    // resumes by x_direction.
    let resume = NinjaAirborneCollisionRouteLoc3D878::Resume {
        next: ninja_collision_resume_loc_3d638(false, x_direction_right),
    };
    let route = match collision_type {
        0 => resume,
        v if v < 0 => NinjaAirborneCollisionRouteLoc3D878::To3DB4E,
        v if v == NINJA_COLID_KIDABOVE as i16 => NinjaAirborneCollisionRouteLoc3D878::To3D8BC,
        v if v == NINJA_COLID_KIDRIGHT as i16 || v == NINJA_COLID_KIDLEFT as i16 => {
            if player_form == PlayerForm::Berzerker && kid_sprite_allows_berzerker_knockdown {
                NinjaAirborneCollisionRouteLoc3D878::To3D8BC
            } else {
                resume
            }
        }
        _ => resume,
    };

    NinjaAirborneCollisionStateLoc3D878 {
        next_collision_type: 0,
        route,
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) enum NinjaKnockdownStartRouteLoc3D8D6 {
    FallthroughLandingLoop,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) struct NinjaKnockdownStartStateLoc3D8D6 {
    pub(crate) y_velocity: i32,
    pub(crate) is_moved: bool,
    pub(crate) has_level_collision: bool,
    pub(crate) next: NinjaKnockdownStartRouteLoc3D8D6,
}

#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn ninja_knockdown_start_loc_3d8d6() -> NinjaKnockdownStartStateLoc3D8D6 {
    // ASM Ninja.asm loc_3D8D6: set `y_vel(a3)` to `$18000`, set
    // `is_moved(a3)` and `has_level_collision(a3)`, then fall through to
    // the floor-wait loop.
    NinjaKnockdownStartStateLoc3D8D6 {
        y_velocity: NINJA_KNOCKDOWN_Y_SPEED_LOC_3D8D6,
        is_moved: true,
        has_level_collision: true,
        next: NinjaKnockdownStartRouteLoc3D8D6::FallthroughLandingLoop,
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) enum NinjaKnockdownLandingRouteLoc3D8E6 {
    Loop,
    KillPath,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) struct NinjaKnockdownLandingStateLoc3D8E6 {
    pub(crate) next_collision_type: u16,
    pub(crate) x_velocity: i32,
    pub(crate) y_velocity: i32,
    pub(crate) y_position: u32,
    pub(crate) is_moved: bool,
    pub(crate) has_level_collision: bool,
    pub(crate) hibernates_one_frame: bool,
    pub(crate) checked_sub_3e8ba: bool,
    pub(crate) route: NinjaKnockdownLandingRouteLoc3D8E6,
}

#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn ninja_knockdown_landing_loop_loc_3d8e6(
    collision_type: u16,
    x_velocity: i32,
    y_velocity: i32,
    y_position: u32,
) -> NinjaKnockdownLandingStateLoc3D8E6 {
    // ASM Ninja.asm loc_3D8E6: clear collision, hibernate one frame, run
    // `sub_3E8BA`, add `$1800` to y velocity, and loop until `colid_floor`.
    // On floor contact, clear both velocities/flags and snap the integer Y word
    // to `((y + $F) & $FFF0)` while clearing sub-pixels.
    let next_y_velocity = y_velocity.wrapping_add(NINJA_KNOCKDOWN_GRAVITY_LOC_3D8E6);
    if collision_type == NINJA_COLID_FLOOR {
        let snapped_y_position = (((y_position >> 16).wrapping_add(0x0F)) & 0x0000_FFF0) << 16;
        NinjaKnockdownLandingStateLoc3D8E6 {
            next_collision_type: 0,
            x_velocity: 0,
            y_velocity: 0,
            y_position: snapped_y_position,
            is_moved: false,
            has_level_collision: false,
            hibernates_one_frame: true,
            checked_sub_3e8ba: true,
            route: NinjaKnockdownLandingRouteLoc3D8E6::KillPath,
        }
    } else {
        NinjaKnockdownLandingStateLoc3D8E6 {
            next_collision_type: 0,
            x_velocity,
            y_velocity: next_y_velocity,
            y_position,
            is_moved: true,
            has_level_collision: true,
            hibernates_one_frame: true,
            checked_sub_3e8ba: true,
            route: NinjaKnockdownLandingRouteLoc3D8E6::Loop,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) enum NinjaFarKidFacingDecisionLoc3D92A {
    WithinBand,
    AlreadyFacingFarKid,
    TurnTowardFarKid,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) struct NinjaFarKidFacingStateLoc3D92A {
    pub(crate) x_direction_right: bool,
    pub(crate) decision: NinjaFarKidFacingDecisionLoc3D92A,
}

#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn ninja_far_kid_facing_loc_3d92a(
    kid_minus_ninja_x_px: i16,
    x_direction_right: bool,
) -> NinjaFarKidFacingStateLoc3D92A {
    // ASM Ninja.asm loc_3D92A: compute `d4 = kid_x - ninja_x`; only signed
    // values outside the inclusive [-$80, +$80] band reach the facing checks.
    let target_right = if kid_minus_ninja_x_px > 0x80 {
        Some(true)
    } else if kid_minus_ninja_x_px < -0x80 {
        Some(false)
    } else {
        None
    };

    let Some(target_right) = target_right else {
        return NinjaFarKidFacingStateLoc3D92A {
            x_direction_right,
            decision: NinjaFarKidFacingDecisionLoc3D92A::WithinBand,
        };
    };

    if target_right == x_direction_right {
        NinjaFarKidFacingStateLoc3D92A {
            x_direction_right,
            decision: NinjaFarKidFacingDecisionLoc3D92A::AlreadyFacingFarKid,
        }
    } else {
        NinjaFarKidFacingStateLoc3D92A {
            x_direction_right: target_right,
            decision: NinjaFarKidFacingDecisionLoc3D92A::TurnTowardFarKid,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) enum NinjaCollisionLoc3D606 {
    /// ASM Ninja.asm loc_3D6DA: no collision, so jump to stealth/attack gate.
    To3D6DA,
    /// ASM Ninja.asm loc_3DB4E: negative collision IDs send to generic hit-kill logic.
    To3DB4E,
    /// ASM Ninja.asm loc_3D65A: hurt or kid-above collisions.
    To3D65A,
    /// ASM Ninja.asm loc_3D698: side collisions against Kid or left/right.
    To3D698,
    /// ASM Ninja.asm loc_3D638: clear collision and route by wall-pinned/direction.
    To3D638 { next: NinjaCollisionResumeLoc3D638 },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) struct NinjaCollisionStateLoc3D606 {
    pub(crate) next_collision_type: u16,
    pub(crate) route: NinjaCollisionLoc3D606,
}

#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn ninja_collision_dispatch_loc_3d606(
    collision_type: i16,
    wall_pinned: bool,
    x_direction_right: bool,
) -> NinjaCollisionStateLoc3D606 {
    // ASM Ninja.asm loc_3D606:
    // - capture `$collision_type(a3)` in d4 and immediately clear it
    // - branch on zero/negative/hurt/side/other collision IDs
    let route = match collision_type {
        0 => NinjaCollisionLoc3D606::To3D6DA,
        v if v < 0 => NinjaCollisionLoc3D606::To3DB4E,
        v if v == NINJA_COLID_HURT as i16 || v == NINJA_COLID_KIDABOVE as i16 => {
            NinjaCollisionLoc3D606::To3D65A
        }
        v if v == NINJA_COLID_KIDRIGHT as i16 || v == NINJA_COLID_KIDLEFT as i16 => {
            NinjaCollisionLoc3D606::To3D698
        }
        _ => NinjaCollisionLoc3D606::To3D638 {
            next: ninja_collision_resume_loc_3d638(wall_pinned, x_direction_right),
        },
    };

    NinjaCollisionStateLoc3D606 {
        next_collision_type: 0,
        route,
    }
}

#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn ninja_collision_resume_loc_3d64e(
    x_direction_right: bool,
) -> NinjaCollisionResumeLoc3D638 {
    // ASM Ninja.asm loc_3D64E: with no wall pin, dispatch only by facing:
    // x_direction=0 → loc_3E560 (left-facing), x_direction=$FF → loc_3E3AC.
    if x_direction_right {
        NinjaCollisionResumeLoc3D638::Loc3E3AC
    } else {
        NinjaCollisionResumeLoc3D638::Loc3E560
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) struct NinjaCollisionStateLoc3D66E {
    pub(crate) next: NinjaCollisionResumeLoc3D638,
}

#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn ninja_collision_resume_loc_3d66e(
    next: NinjaCollisionResumeLoc3D638,
) -> NinjaCollisionStateLoc3D66E {
    // ASM Ninja.asm loc_3D66E is a plain branch to loc_3D638
    // with no side effects.
    NinjaCollisionStateLoc3D66E { next }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) enum NinjaCollisionLoc3D670Route {
    /// ASM Ninja.asm loc_3D6DA branch after `$44(a5)` underflow.
    To3DB4E,
    /// ASM Ninja.asm loc_3D638 route after blocking animation completes.
    To3D638 { next: NinjaCollisionResumeLoc3D638 },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) struct NinjaCollisionStateLoc3D670 {
    pub(crate) next_collision_type: u16,
    pub(crate) remaining_block_timer: u16,
    pub(crate) route: NinjaCollisionLoc3D670Route,
    pub(crate) uses_blocking_animation: bool,
    pub(crate) blocking_animation: Option<&'static str>,
    pub(crate) restored_animation: Option<&'static str>,
    pub(crate) hibernates_until_animation_finished: bool,
}

#[allow(clippy::too_many_arguments)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn ninja_collision_block_animation_loc_3d670(
    block_timer: u16,
    wall_pinned: bool,
    x_direction_right: bool,
    restored_animation: &'static str,
) -> NinjaCollisionStateLoc3D670 {
    // ASM Ninja.asm loc_3D670:
    // - subtracts 1 from `$44(a5)`
    // - if it reached zero, jumps to loc_3DB4E
    // - otherwise plays `stru_3D504`, waits for it, restores `$62(a5)` into
    //   `$50(a5)`, and reinitialises animation before jumping to loc_3D638.
    let remaining_block_timer = block_timer.wrapping_sub(1);
    if remaining_block_timer == 0 {
        NinjaCollisionStateLoc3D670 {
            next_collision_type: 0,
            remaining_block_timer,
            route: NinjaCollisionLoc3D670Route::To3DB4E,
            uses_blocking_animation: false,
            blocking_animation: None,
            restored_animation: None,
            hibernates_until_animation_finished: false,
        }
    } else {
        NinjaCollisionStateLoc3D670 {
            next_collision_type: 0,
            remaining_block_timer,
            route: NinjaCollisionLoc3D670Route::To3D638 {
                next: ninja_collision_resume_loc_3d638(wall_pinned, x_direction_right),
            },
            uses_blocking_animation: true,
            blocking_animation: Some("stru_3D504"),
            restored_animation: Some(restored_animation),
            hibernates_until_animation_finished: true,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) struct NinjaCollisionStateLoc3D6C8 {
    pub(crate) y_velocity: i32,
    pub(crate) hibernates_one_frame: bool,
    pub(crate) checked_sub_3e8ba: bool,
    pub(crate) loops: bool,
}

#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn ninja_collision_stun_loop_loc_3d6c8(y_velocity: i32) -> NinjaCollisionStateLoc3D6C8 {
    // ASM Ninja.asm loc_3D6C8:
    // - hibernate one frame
    // - call sub_3E8BA (shared deactivation gate)
    // - add `$4000` to y_vel(a3)
    // - branch back and repeat.
    NinjaCollisionStateLoc3D6C8 {
        y_velocity: y_velocity.wrapping_add(0x4000),
        hibernates_one_frame: true,
        checked_sub_3e8ba: true,
        loops: true,
    }
}

/// Step a Ninja enemy: stealth-paces until player enters detection cone,
/// then faces player and jump-attacks. Returns to stealth on landing.
pub(crate) fn step_ninja(
    enemy: &mut ActiveEnemy,
    block_grid: Option<&terrain::BlockGrid>,
    player_x_pos: Option<u32>,
    player_y_pos: Option<u32>,
    player_form: PlayerForm,
) {
    match enemy.ninja_state {
        NinjaState::Stealth => {
            // Pace like normal
            step_pacer(enemy, block_grid);

            // Check detection cone (skip if player wears Red Stealth)
            if player_form == PlayerForm::RedStealth {
                return;
            }
            let (Some(px), Some(py)) = (player_x_pos, player_y_pos) else {
                return;
            };
            let dx = (px as i64 - enemy.x_pos as i64) >> 16;
            let dy = (enemy.y_pos as i64 - py as i64) >> 16;

            // Player must be 0x20–0x50 px above ninja and in [-0x10, +0x10).
            if ninja_detects_player_while_stealthed(dx, dy) {
                // Face the player
                enemy.facing = if dx >= 0 {
                    Direction::Right
                } else {
                    Direction::Left
                };
                // Launch jump attack
                enemy.ninja_state = NinjaState::JumpAttack;
                enemy.x_velocity = match enemy.facing {
                    Direction::Left => -NINJA_JUMP_X_SPEED_LEFT,
                    Direction::Right => NINJA_JUMP_X_SPEED_RIGHT,
                };
                enemy.y_velocity = NINJA_JUMP_Y_SPEED;
                enemy.on_ground = false;
            }
        }
        NinjaState::JumpAttack => {
            // Apply gravity
            enemy.y_velocity = (enemy.y_velocity + NINJA_GRAVITY).min(physics::MAX_FALL_SPEED);

            // Move
            enemy.x_pos = (enemy.x_pos as i64 + i64::from(enemy.x_velocity)) as u32;
            enemy.y_pos = (enemy.y_pos as i64 + i64::from(enemy.y_velocity)) as u32;

            let Some(grid) = block_grid else {
                return;
            };
            let cx = (enemy.x_pos >> 16) as i32;
            let cy = (enemy.y_pos >> 16) as i32;

            // Wall bounce: reverse x on wall hit
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
                enemy.x_velocity = -enemy.x_velocity;
                enemy.facing = opposite_direction(enemy.facing);
            }

            // Floor landing: return to stealth
            if enemy.y_velocity > 0 {
                let floor = terrain::probe_floor(grid, cx, cy);
                if floor.on_ground {
                    let floor_y_fp = (floor.floor_y_px as u32) << 16;
                    if enemy.y_pos >= floor_y_fp {
                        enemy.y_pos = floor_y_fp;
                        enemy.x_velocity = 0;
                        enemy.y_velocity = 0;
                        enemy.on_ground = true;
                        enemy.ninja_state = NinjaState::Stealth;
                    }
                }
            }
        }
    }
}

/// Hand detection ranges by HP tier (Hand.asm unk_3276A table).
/// HP 1 = 64px, HP 2 = 80px, HP 3+ = 100px.
pub(crate) const HAND_DETECT_RANGE: [i32; 3] = [0x40, 0x50, 0x64];

/// Hand walk velocities by HP tier (Hand.asm unk_3276A table).
pub(crate) const HAND_WALK_SPEED: [i32; 3] = [0x3000, 0x4800, 0x6000];

/// Hand jump-attack gravity (Hand.asm line 130: $4000 per frame).
pub(crate) const HAND_GRAVITY: i32 = 0x4000;

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
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
#[cfg_attr(not(test), allow(dead_code))]
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
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn hand_x_velocity_loc_328e0(x_velocity_word: i16) -> i32 {
    // ASM Hand.asm loc_328E0: `swap d7; move.l d7,x_vel(a3)` promotes the
    // signed X word prepared by loc_328AA into the high word of 16.16 velocity.
    i32::from(x_velocity_word) << 16
}

fn fixed_position_integer_word(position: u32) -> u16 {
    let [high_a, high_b, _, _] = position.to_be_bytes();
    u16::from_be_bytes([high_a, high_b])
}

/// Hand max fall speed: $180000 (Hand.asm lines 728-730), NOT the shared MAX_FALL_SPEED.
pub(crate) const HAND_MAX_FALL_SPEED: i32 = 0x18_0000;

/// Hand grab duration: 10 frames (Hand.asm loc_32906: moveq #$A,d3).
pub(crate) const HAND_GRAB_DURATION: u16 = 10;

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
#[cfg_attr(not(test), allow(dead_code))]
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
#[cfg_attr(not(test), allow(dead_code))]
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
#[cfg_attr(not(test), allow(dead_code))]
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
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) struct HandInitialMotionLoc327F2 {
    pub(crate) x_velocity: i32,
    pub(crate) animation_label: &'static str,
    pub(crate) has_kid_collision: bool,
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn hand_initial_motion_loc_327f2(
    hit_points: u8,
    x_direction_flag: bool,
) -> HandInitialMotionLoc327F2 {
    // ASM Hand.asm loc_327F2: store d7 into x_vel, initialise `stru_32B6A`,
    // then clear has_kid_collision before entering the patrol loop.
    HandInitialMotionLoc327F2 {
        x_velocity: hand_velocity_candidate_loc_327e4(hit_points, x_direction_flag),
        animation_label: "stru_32B6A",
        has_kid_collision: false,
    }
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
#[cfg_attr(not(test), allow(dead_code))]
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

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
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
                        if enemy.x_pos + edge_fp > wall_fp {
                            enemy.x_pos = wall_fp.saturating_sub(edge_fp);
                        }
                    } else if enemy.x_pos < wall_fp + edge_fp {
                        enemy.x_pos = wall_fp + edge_fp;
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

            // Player detection: forward cone + 16px vertical + 45-degree angle
            let (Some(px), Some(py)) = (player_x_pos, player_y_pos) else {
                return;
            };
            let dx = (px as i64 - enemy.x_pos as i64) >> 16;
            let dy = (enemy.y_pos as i64 - py as i64) >> 16;

            // Player must be 0-16px above Hand
            if !(0..=16).contains(&dy) {
                return;
            }

            let detect_range = i64::from(HAND_DETECT_RANGE[tier]);
            let in_range = match enemy.facing {
                Direction::Right => dx > 0 && dx <= detect_range,
                Direction::Left => dx < 0 && (-dx) <= detect_range,
            };

            // 45-degree cone constraint: |dy| <= |dx| (Hand.asm loc_3284C lines 99-114)
            let in_cone = in_range && dy.abs() <= dx.abs();

            if in_cone {
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
            // Apply gravity, capped at Hand-specific $180000 (not shared MAX_FALL_SPEED)
            enemy.y_velocity = (enemy.y_velocity + HAND_GRAVITY).min(HAND_MAX_FALL_SPEED);

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
            // Hand.asm loc_3292E: hand snaps to kid's X±2, Y-4 each frame.
            if let (Some(px), Some(py)) = (player_x_pos, player_y_pos) {
                let x_off: i64 = match enemy.facing {
                    Direction::Right => 2 << 16,
                    Direction::Left => -(2i64 << 16),
                };
                enemy.x_pos = (px as i64 + x_off) as u32;
                enemy.y_pos = (py as i64 - (4i64 << 16)) as u32;
            }
            enemy.x_velocity = 0;
            enemy.y_velocity = 0;

            if enemy.wait_timer > 0 {
                enemy.wait_timer -= 1;
            } else {
                enemy.hand_state = HandState::Walking;
            }
        }
    }
}

/// Lion charge speed: $28000 for HP==8, $18000 otherwise (Lion.asm loc_3DFD8).
/// Also used as walk speed — ASM sets both $50(a5) and $5E(a5) from same value.
pub(crate) fn lion_charge_speed(hit_points: u8) -> i32 {
    if hit_points == 8 { 0x2_8000 } else { 0x1_8000 }
}

/// Lion charge timer: starts at $3C (60 frames), counts down while pacing.
/// Seeded by Lion.asm loc_3DFF0.
/// ASM loc_3E078: on expiry, unconditionally starts roar → charge regardless
/// of Kid distance.  Reload is $B4 (180) via LION_COOLDOWN_AFTER_FIRE.
pub(crate) const LION_CHARGE_TIMER_INIT: u16 = 60;

/// Lion post-projectile cooldown: $B4 (180 frames) before next charge after
/// firing a projectile (Lion.asm loc_3E096).
pub(crate) const LION_COOLDOWN_AFTER_FIRE: u16 = 180;

/// Lion charge overshoot distance for termination: $70 (112px).
/// ASM does not use a fixed duration — charge ends when Lion overshoots
/// the player by >112px (loc_3E450/3E604).
pub(crate) const LION_CHARGE_OVERSHOOT: i32 = 0x70;

/// Lion projectile speed (Lion.asm line 165: $60000).
pub(crate) const LION_PROJECTILE_SPEED: i32 = 0x6_0000;

// Lion homing accel is $2000/frame (Lion.asm loc_3E1C2) — set on projectile.
// Wall-bounce: Lion.asm loc_3E260 — reverses velocity on terrain collision.
// Speed cap: ±$60000 on both axes (Lion.asm loc_3E1C2).

/// Lion projectile spawn X offset: 10px ahead in facing direction.
pub(crate) const LION_PROJECTILE_X_OFFSET: i32 = 10;

/// Lion projectile spawn Y offset: 24px above Lion's Y position ($18).
pub(crate) const LION_PROJECTILE_Y_OFFSET: i32 = 24;

fn lion_active_projectile_count(enemy: &ActiveEnemy) -> u8 {
    enemy.ufo_beam_segments
}

fn increment_lion_active_projectile_count(enemy: &mut ActiveEnemy) {
    enemy.ufo_beam_segments = enemy.ufo_beam_segments.saturating_add(1);
}

/// Step a Lion enemy: paces normally, then charges toward player and fires a
/// homing projectile. Charge ends on wall/edge or player overshoot >112px.
/// ASM: Lion.asm — full state machine with detection, charge, and cooldown.
pub(crate) fn step_lion(
    enemy: &mut ActiveEnemy,
    block_grid: Option<&terrain::BlockGrid>,
    player_x_pos: Option<u32>,
    projectiles: &mut Option<&mut Vec<EnemyProjectile>>,
) {
    match enemy.lion_state {
        LionState::Walking => {
            // Pace like normal
            step_pacer(enemy, block_grid);

            // Count down charge timer.  ASM loc_3E078: timer always ticks and
            // the charge fires unconditionally on expiry (no Kid distance gate).
            if enemy.wait_timer > 0 {
                enemy.wait_timer -= 1;
                return;
            }

            // ASM loc_3E078: after reloading $54(a5) to $B4, a live projectile
            // count in $5B(a5) suppresses the next roar/fire cycle.
            if lion_active_projectile_count(enemy) > 0 {
                enemy.wait_timer = LION_COOLDOWN_AFTER_FIRE;
                return;
            }

            // Timer expired — face the player and start charging.
            // ASM loc_3E0D6/loc_3E0DA: after roar, always turns toward Kid.
            let Some(px) = player_x_pos else {
                // No player info (shouldn't happen in practice) — reload.
                enemy.wait_timer = LION_COOLDOWN_AFTER_FIRE;
                return;
            };
            let dx = (px as i64 - enemy.x_pos as i64) >> 16;

            // Face the player and start charging
            enemy.facing = if dx >= 0 {
                Direction::Right
            } else {
                Direction::Left
            };
            enemy.lion_state = LionState::Charging;

            // Fire a homing projectile toward the player (Lion.asm loc_3E12A).
            // Spawns 10px ahead in X, 24px above in Y.
            if let Some(projs) = projectiles {
                let speed = LION_PROJECTILE_SPEED;
                let (xv, x_off) = match enemy.facing {
                    Direction::Right => (speed, LION_PROJECTILE_X_OFFSET),
                    Direction::Left => (-speed, -LION_PROJECTILE_X_OFFSET),
                };
                let proj_x = (enemy.x_pos as i64 + ((x_off as i64) << 16)) as u32;
                let proj_y =
                    (enemy.y_pos as i64 - ((LION_PROJECTILE_Y_OFFSET as i64) << 16)) as u32;
                projs.push(EnemyProjectile {
                    x_pos: proj_x,
                    y_pos: proj_y,
                    x_velocity: xv,
                    y_velocity: 0,
                    lifetime: 120,
                    gravity: 0,
                    friendly: false,
                    homing_accel: 0x2000,             // Lion.asm loc_3E1C2
                    speed_cap: LION_PROJECTILE_SPEED, // ±$60000 cap (Lion.asm)
                    wall_bounce: true,                // Lion.asm loc_3E260
                    owner: Some(EnemyProjectileOwner {
                        segment_index: enemy.segment_index,
                        layout_index: enemy.layout_index,
                        token: enemy.ufo_beam_interval,
                    }),
                    ..Default::default()
                });
                increment_lion_active_projectile_count(enemy);
            }

            // Post-fire cooldown is longer: $B4 (180 frames) at loc_3E096.
            // The cooldown is applied when returning to walk after charge.
        }
        LionState::Charging => {
            // Move at charge speed
            let speed = lion_charge_speed(enemy.hit_points);
            enemy.x_velocity = horizontal_velocity(enemy.facing, speed);
            enemy.x_pos = (enemy.x_pos as i64 + i64::from(enemy.x_velocity)) as u32;

            // Wall check
            if let Some(grid) = block_grid {
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
                    reverse_enemy_direction(enemy);
                    enemy.lion_state = LionState::Walking;
                    enemy.wait_timer = LION_COOLDOWN_AFTER_FIRE;
                    return;
                }

                // Edge check
                let look_ahead_x = if moving_right {
                    cx + ENEMY_PROBE_HALF_WIDTH
                } else {
                    cx - ENEMY_PROBE_HALF_WIDTH
                };
                if !terrain::probe_floor(grid, look_ahead_x, cy).on_ground {
                    reverse_enemy_direction(enemy);
                    enemy.lion_state = LionState::Walking;
                    enemy.wait_timer = LION_COOLDOWN_AFTER_FIRE;
                    return;
                }
            }

            // Overshoot check: if Lion has passed player by >112px, stop charging
            // (Lion.asm loc_3E450/3E604 — no fixed duration timer).
            if let Some(px) = player_x_pos {
                let dx = match enemy.facing {
                    Direction::Right => (enemy.x_pos as i64 - px as i64) >> 16,
                    Direction::Left => (px as i64 - enemy.x_pos as i64) >> 16,
                };
                if dx > i64::from(LION_CHARGE_OVERSHOOT) {
                    enemy.lion_state = LionState::Walking;
                    enemy.wait_timer = LION_COOLDOWN_AFTER_FIRE;
                }
            }
        }
    }
}

/// Robot walk speed: HP 2 = $8000, HP 3+ = $10000.
/// ASM Robot.asm loc_3A7EE always adds one `$8000`, with a second `$8000`
/// preloaded only when `current_hp >= 3`.
pub(crate) fn robot_walk_speed(hit_points: u8) -> i32 {
    if hit_points >= 3 { 0x1_0000 } else { 0x8000 }
}

/// Robot charge speed: always $28000 (Robot.asm line 157).
pub(crate) const ROBOT_CHARGE_SPEED: i32 = 0x2_8000;

/// Robot shoot interval: ASM Robot.asm init sets $4A(a5) = $20 = 32 frames.
pub(crate) const ROBOT_SHOOT_INTERVAL: u16 = 32;

/// Robot charge direction-flip timer base: ASM Robot.asm sets $54(a5) = $F = 15.
/// Reloads to (random_byte >> 1) + $F (range 15-142) on each flip.
/// The charge is permanent until wall hit — no timed exit.
pub(crate) const ROBOT_CHARGE_FLIP_BASE: u16 = 15;

pub(crate) fn robot_charge_flip_reload_with_rng(rng: &mut RomRandomState) -> u16 {
    // ASM loc_3A9F0: `Get_RandomNumber_byte`, `lsr.w #1,d7`, `addi.w #$F,d7`.
    let random_byte = rng.next_byte();
    u16::from(random_byte >> 1) + ROBOT_CHARGE_FLIP_BASE
}

/// Robot projectile speed: ~5 px/frame horizontal (Robot.asm).
pub(crate) const ROBOT_PROJECTILE_SPEED: i32 = 0x5_0000;

#[cfg(test)]
pub(crate) fn robot_jump_sound_edge_loc_3a890(
    body_sprite_is_landing_frame: bool,
    animation_timer: u16,
    motion_suppressed: bool,
) -> bool {
    // ASM Robot.asm loc_3A890: C82A8 sprite, animation_timer == 4, and clear
    // `$19(a3)` replay sfx_Robot_jumping_on before collision dispatch.
    body_sprite_is_landing_frame && animation_timer == 4 && !motion_suppressed
}

#[cfg(test)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum RobotCollisionRouteLoc3A8Ae {
    ClearAndWalk,
    HitReaction,
    EnterCharge,
    ContinueCharge,
    Destroy,
}

#[cfg(test)]
pub(crate) fn robot_collision_dispatch_loc_3a8ae(
    parent_collision_type: i16,
    child_collision_type: i16,
    berzerker_charging: bool,
) -> RobotCollisionRouteLoc3A8Ae {
    // ASM Robot.asm loc_3A8AE: parent collision_type is tested before `$38(a2)`.
    if parent_collision_type.is_negative() {
        return RobotCollisionRouteLoc3A8Ae::Destroy;
    }
    match parent_collision_type {
        0x0800 => return RobotCollisionRouteLoc3A8Ae::ContinueCharge,
        0x001C => return RobotCollisionRouteLoc3A8Ae::HitReaction,
        _ => {}
    }

    if child_collision_type.is_negative() {
        return RobotCollisionRouteLoc3A8Ae::EnterCharge;
    }
    match child_collision_type {
        0x0000 => RobotCollisionRouteLoc3A8Ae::ClearAndWalk,
        0x001C | 0x002C => RobotCollisionRouteLoc3A8Ae::HitReaction,
        0x0020 | 0x0024 if berzerker_charging => RobotCollisionRouteLoc3A8Ae::Destroy,
        0x0020 | 0x0024 => RobotCollisionRouteLoc3A8Ae::ClearAndWalk,
        0x0800 => RobotCollisionRouteLoc3A8Ae::ContinueCharge,
        _ => RobotCollisionRouteLoc3A8Ae::ClearAndWalk,
    }
}

#[cfg(test)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum RobotHitReactionRouteLoc3A8Fc {
    ClearAndWalk,
    EnterCharge,
}

#[cfg(test)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum RobotChildSpriteUpdateLoc3A8Fc {
    KeepCurrent,
    UseHitFrame,
}

#[cfg(test)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct RobotHitReactionLoc3A8Fc {
    pub(crate) remaining_hp: u16,
    pub(crate) route: RobotHitReactionRouteLoc3A8Fc,
    pub(crate) next_parent_collision_type: u16,
    pub(crate) next_child_collision_type: u16,
    pub(crate) child_collision_latched: bool,
    pub(crate) transient_child_y_offset_delta_px: i16,
    pub(crate) child_y_offset_delta_px: i16,
    pub(crate) child_sprite_update: RobotChildSpriteUpdateLoc3A8Fc,
    pub(crate) restores_walk_animations: bool,
    pub(crate) replay_jump_sound: bool,
}

#[cfg(test)]
pub(crate) fn robot_hit_reaction_loc_3a8fc(
    current_hp: u16,
    tracking_latch_set: bool,
    motion_suppressed: bool,
) -> RobotHitReactionLoc3A8Fc {
    // ASM Robot.asm loc_3A8FC: latch child collision and optionally select the
    // hit frame while `$4E(a5)` is clear. loc_3A90C then moves the child down
    // 4px for the hit animation before decrementing HP.
    let remaining_hp = current_hp.wrapping_sub(1);
    let child_sprite_update = if tracking_latch_set {
        RobotChildSpriteUpdateLoc3A8Fc::KeepCurrent
    } else {
        RobotChildSpriteUpdateLoc3A8Fc::UseHitFrame
    };

    if remaining_hp == 0 {
        return RobotHitReactionLoc3A8Fc {
            remaining_hp,
            route: RobotHitReactionRouteLoc3A8Fc::EnterCharge,
            next_parent_collision_type: 0x0800,
            next_child_collision_type: 0x0800,
            child_collision_latched: true,
            transient_child_y_offset_delta_px: 4,
            child_y_offset_delta_px: 4,
            child_sprite_update,
            restores_walk_animations: false,
            replay_jump_sound: false,
        };
    }

    // ASM Robot.asm loc_3A936: non-lethal hits clear the child collision latch,
    // subtract the same 4px from child Y, and reinitialise the body/head walk
    // animations before clearing the pending collision words.
    RobotHitReactionLoc3A8Fc {
        remaining_hp,
        route: RobotHitReactionRouteLoc3A8Fc::ClearAndWalk,
        next_parent_collision_type: 0,
        next_child_collision_type: 0,
        child_collision_latched: false,
        transient_child_y_offset_delta_px: 4,
        child_y_offset_delta_px: 0,
        child_sprite_update,
        restores_walk_animations: true,
        replay_jump_sound: !motion_suppressed,
    }
}

#[cfg(test)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct RobotCollisionClearLoc3A958 {
    pub(crate) next_parent_collision_type: u16,
    pub(crate) next_child_collision_type: u16,
    pub(crate) resume_walk_direction: Direction,
}

#[cfg(test)]
pub(crate) fn robot_collision_clear_loc_3a958(facing: Direction) -> RobotCollisionClearLoc3A958 {
    // ASM Robot.asm loc_3A958: clear child `$38(a2)` and parent
    // `collision_type(a3)`, then dispatch to the shared left/right walker
    // from x_direction. Native `facing` is the decoded branch target.
    RobotCollisionClearLoc3A958 {
        next_parent_collision_type: 0,
        next_child_collision_type: 0,
        resume_walk_direction: facing,
    }
}

/// Step a Robot enemy: paces at HP-dependent speed, shoots at intervals,
/// then enters a fast charge phase.
#[cfg(test)]
pub(crate) fn step_robot(
    enemy: &mut ActiveEnemy,
    block_grid: Option<&terrain::BlockGrid>,
    player_x_pos: Option<u32>,
    projectiles: &mut Option<&mut Vec<EnemyProjectile>>,
) {
    let mut rng = RomRandomState::default();
    step_robot_with_rng(enemy, block_grid, player_x_pos, projectiles, &mut rng);
}

pub(crate) fn step_robot_with_rng(
    enemy: &mut ActiveEnemy,
    block_grid: Option<&terrain::BlockGrid>,
    player_x_pos: Option<u32>,
    projectiles: &mut Option<&mut Vec<EnemyProjectile>>,
    rng: &mut RomRandomState,
) {
    match enemy.robot_state {
        RobotState::Walking => {
            let walk_speed = robot_walk_speed(enemy.hit_points);
            // ASM Robot.asm loc_3A884 enters the shared walker left/right branch
            // from x_direction; native `facing` is that decoded branch choice.
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
                        if enemy.x_pos + edge_fp > wall_fp {
                            enemy.x_pos = wall_fp.saturating_sub(edge_fp);
                        }
                    } else if enemy.x_pos < wall_fp + edge_fp {
                        enemy.x_pos = wall_fp + edge_fp;
                    }
                    reverse_enemy_direction(enemy);
                    return;
                }

                let look_ahead_x = if moving_right {
                    cx + ENEMY_PROBE_HALF_WIDTH
                } else {
                    cx - ENEMY_PROBE_HALF_WIDTH
                };
                if !terrain::probe_floor(grid, look_ahead_x, cy).on_ground {
                    reverse_enemy_direction(enemy);
                }
            }

            // Shoot timer
            if enemy.wait_timer > 0 {
                enemy.wait_timer -= 1;
            } else {
                // Face player if available
                if let Some(px) = player_x_pos {
                    let dx = px as i64 - enemy.x_pos as i64;
                    enemy.facing = if dx >= 0 {
                        Direction::Right
                    } else {
                        Direction::Left
                    };
                }

                // Fire projectile
                if let Some(projs) = projectiles {
                    let speed = ROBOT_PROJECTILE_SPEED;
                    let xv = match enemy.facing {
                        Direction::Right => speed,
                        Direction::Left => -speed,
                    };
                    projs.push(EnemyProjectile {
                        x_pos: enemy.x_pos,
                        y_pos: enemy.y_pos,
                        x_velocity: xv,
                        y_velocity: 0,
                        lifetime: 60,
                        gravity: 0,
                        friendly: false,
                        homing_accel: 0,
                        speed_cap: 0,
                        wall_bounce: false,
                        ..Default::default()
                    });
                }

                // Enter charge mode — ASM: permanent until wall hit
                enemy.robot_state = RobotState::Charging;
                enemy.wait_timer = ROBOT_CHARGE_FLIP_BASE;
            }
        }
        RobotState::Charging => {
            // ASM Robot.asm: charge is permanent until wall hit.
            // Direction-flip timer fires periodically; Robot stays charging.
            enemy.x_velocity = horizontal_velocity(enemy.facing, ROBOT_CHARGE_SPEED);
            enemy.x_pos = (enemy.x_pos as i64 + i64::from(enemy.x_velocity)) as u32;

            if let Some(grid) = block_grid {
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
                    // Wall hit ends charge → return to walking
                    reverse_enemy_direction(enemy);
                    enemy.robot_state = RobotState::Walking;
                    enemy.wait_timer = ROBOT_SHOOT_INTERVAL;
                    return;
                }
                let look_ahead_x = if moving_right {
                    cx + ENEMY_PROBE_HALF_WIDTH
                } else {
                    cx - ENEMY_PROBE_HALF_WIDTH
                };
                if !terrain::probe_floor(grid, look_ahead_x, cy).on_ground {
                    // Edge → also exit charge
                    reverse_enemy_direction(enemy);
                    enemy.robot_state = RobotState::Walking;
                    enemy.wait_timer = ROBOT_SHOOT_INTERVAL;
                    return;
                }
            }

            // Direction-flip timer: flip direction, stay charging
            // ASM reload: (random_byte >> 1) + $F (range 15-142)
            if enemy.wait_timer > 0 {
                enemy.wait_timer -= 1;
            } else {
                reverse_enemy_direction(enemy);
                enemy.wait_timer = robot_charge_flip_reload_with_rng(rng);
            }
        }
    }
}

/// Orca proximity detection range (Orca.asm loc_3D2DE: ~$30 px).
pub(crate) const ORCA_DETECT_RANGE: i32 = 0x30;

/// Orca lunge velocities (Orca.asm).
pub(crate) const ORCA_LUNGE_X_SPEED: i32 = 0x3_0000;
pub(crate) const ORCA_LUNGE_Y_SPEED: i32 = -0x2_8000;

/// Orca lunge gravity (Orca.asm line loc_3D3B4: $1800 per frame).
pub(crate) const ORCA_LUNGE_GRAVITY: i32 = 0x1800;

/// Orca lunge X acceleration (Orca.asm loc_3D3A2: $200 per frame in facing dir).
pub(crate) const ORCA_LUNGE_X_ACCEL: i32 = 0x200;

/// Orca lunge cooldown (Orca.asm: $12C = 300 frames between lunges).
pub(crate) const ORCA_LUNGE_COOLDOWN: u16 = 300;

/// Step an Orca enemy: paces at $8000, detects nearby player, lunges in a
/// ballistic arc, lands and resumes walking with a cooldown.
pub(crate) fn step_orca(
    enemy: &mut ActiveEnemy,
    block_grid: Option<&terrain::BlockGrid>,
    player_x_pos: Option<u32>,
) {
    match enemy.orca_state {
        OrcaState::Walking => {
            step_pacer(enemy, block_grid);

            // Cooldown between lunges
            if enemy.wait_timer > 0 {
                enemy.wait_timer -= 1;
                return;
            }

            // Proximity detection
            let Some(px) = player_x_pos else {
                return;
            };
            let dx = (px as i64 - enemy.x_pos as i64) >> 16;
            if dx.abs() > i64::from(ORCA_DETECT_RANGE) {
                return;
            }

            // Launch lunge
            enemy.orca_state = OrcaState::Lunging;
            enemy.on_ground = false;
            enemy.facing = if dx >= 0 {
                Direction::Right
            } else {
                Direction::Left
            };
            enemy.x_velocity = match enemy.facing {
                Direction::Right => ORCA_LUNGE_X_SPEED,
                Direction::Left => -ORCA_LUNGE_X_SPEED,
            };
            enemy.y_velocity = ORCA_LUNGE_Y_SPEED;
        }
        OrcaState::Lunging => {
            // Gravity (Orca.asm loc_3D3B4: $1800 per frame)
            enemy.y_velocity = (enemy.y_velocity + ORCA_LUNGE_GRAVITY).min(physics::MAX_FALL_SPEED);

            // X acceleration during lunge (Orca.asm loc_3D3A2: $200 per frame in
            // facing direction, unless wall-pinned via $5C flag).
            if !enemy.wall_pinned {
                let accel = match enemy.facing {
                    Direction::Right => ORCA_LUNGE_X_ACCEL,
                    Direction::Left => -ORCA_LUNGE_X_ACCEL,
                };
                enemy.x_velocity += accel;
            }

            // Move
            enemy.x_pos = (enemy.x_pos as i64 + i64::from(enemy.x_velocity)) as u32;
            enemy.y_pos = (enemy.y_pos as i64 + i64::from(enemy.y_velocity)) as u32;

            let Some(grid) = block_grid else {
                return;
            };
            let cx = (enemy.x_pos >> 16) as i32;
            let cy = (enemy.y_pos >> 16) as i32;

            // Slope collision → flip direction and re-lunge (Orca.asm loc_3D3DE)
            let floor = terrain::probe_floor(grid, cx, cy);
            if floor.on_ground
                && (floor.cell.collision == terrain::CollisionKind::SlopeUp
                    || floor.cell.collision == terrain::CollisionKind::SlopeDown)
            {
                enemy.x_velocity = 0;
                enemy.y_velocity = 0;
                enemy.facing = opposite_direction(enemy.facing);
                enemy.wall_pinned = false;
                // Re-lunge in opposite direction
                enemy.x_velocity = match enemy.facing {
                    Direction::Right => ORCA_LUNGE_X_SPEED,
                    Direction::Left => -ORCA_LUNGE_X_SPEED,
                };
                enemy.y_velocity = ORCA_LUNGE_Y_SPEED;
                return;
            }

            // Wall bounce — ASM Orca.asm loc_3D40C: zero x_vel, set wall-pinned flag.
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
                enemy.wall_pinned = true;
                enemy.facing = opposite_direction(enemy.facing);
            }

            // Ceiling bounce — ASM Orca.asm loc_3D47A: y_vel = $18000
            if enemy.y_velocity < 0 {
                let ceil = terrain::probe_ceiling(
                    grid,
                    cx,
                    cy,
                    ENEMY_PROBE_HALF_WIDTH,
                    ENEMY_PROBE_HEIGHT,
                );
                if ceil.hit {
                    enemy.y_velocity = 0x1_8000; // bounce down
                }
            }

            // Floor landing (Orca.asm loc_3D420)
            if enemy.y_velocity > 0 {
                let floor = terrain::probe_floor(grid, cx, cy);
                if floor.on_ground
                    && floor.cell.collision != terrain::CollisionKind::SlopeUp
                    && floor.cell.collision != terrain::CollisionKind::SlopeDown
                {
                    let floor_y_fp = (floor.floor_y_px as u32) << 16;
                    if enemy.y_pos >= floor_y_fp {
                        enemy.y_pos = floor_y_fp;
                        enemy.x_velocity = 0;
                        enemy.y_velocity = 0;
                        enemy.on_ground = true;
                        enemy.wall_pinned = false;
                        enemy.orca_state = OrcaState::Walking;
                        enemy.wait_timer = ORCA_LUNGE_COOLDOWN;
                    }
                }
            }
        }
    }
}

// --- TarMonster constants (Tar_Monster.asm) ---
// Bytecode-driven cycle: submerged → emerge → visible → attack → submerge → loop.
// Timings from ASM script tables (unk_32264, unk_3226C):
//   Initial idle: 4f then 90f ($5A)
//   Emerge animation: 24f (stru_3272E, 6 sprites × 4fps)
//   Visible hold: 50f ($32) before attack
//   Attack: 27f (9+9+9 from `moveq #8,d0` + `dbf` in loc_3231C/loc_32348/loc_32360),
//   fires the horizontal glob shot after the first 9-frame wind-up
//   Submerge animation: 24f (stru_3274C, reverse of emerge)
/// Initial submerged wait before first emergence (Tar_Monster.asm unk_32264: 4 + 90).
pub(crate) const TAR_MONSTER_SUBMERGED_FRAMES: u16 = 94;
/// Emerge animation duration (6 frames × 4 ticks = 24 frames).
pub(crate) const TAR_MONSTER_EMERGE_FRAMES: u16 = 24;
/// Visible hold before attack sequence ($32 = 50 frames from bytecode).
pub(crate) const TAR_MONSTER_VISIBLE_FRAMES: u16 = 50;
/// Attack sequence duration. ASM loc_32310/loc_3231C/loc_32348/loc_32360:
/// three `moveq #8,d0` + `dbf` loops, so each phase lasts 9 frames.
pub(crate) const TAR_MONSTER_ATTACK_FRAMES: u16 = 27;
/// Zero-based elapsed frame index for the glob shot. ASM loc_3231C fires after
/// the first 9-frame `dbf` loop, which is elapsed frame 8 in the native countdown.
pub(crate) const TAR_MONSTER_ATTACK_FIRE_FRAME: u16 = 8;
/// Submerge animation duration (24 frames, reverse of emerge).
pub(crate) const TAR_MONSTER_SUBMERGE_FRAMES: u16 = 24;
/// Horizontal glob shot speed: ±4px/frame (Tar_Monster.asm line 316: $FFFC = -4).
pub(crate) const TAR_MONSTER_GLOB_SPEED_X: i32 = 4 << 16;
/// Glob gravity: $1000/frame (Tar_Monster.asm line 412).
pub(crate) const TAR_MONSTER_GLOB_GRAVITY: i32 = 0x1000;
/// Raw control script bytes used by `unk_32264` in `loc_32248`.
/// Bytecode pairs are read with `move.w (a0)+,d0`, so each table entry is a signed
/// 16-bit step command in the native script interpreter.
pub(crate) const TAR_MONSTER_CONTROL_BYTES_LOC_32248: [u8; 44] = [
    0x00, 0x04, 0x00, 0x10, 0x5A, 0x00, 0x00, 0x10, 0x05, 0x00, 0x00, 0x10, 0x0A, 0x00, 0x00, 0x10,
    0x07, 0x00, 0x00, 0x10, 0x32, 0x00, 0x00, 0x10, 0x00, 0x0A, 0x00, 0x00, 0x08, 0x46, 0x00, 0x00,
    0x0C, 0x00, 0x00, 0x04, 0x14, 0x00, 0x00, 0x08, 0x46, 0x00, 0xFF, 0xFF,
];

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) enum TarMonsterLoc32290Target {
    /// `off_32290[0]`: jump to `loc_322A8`.
    Loc322A8,
    /// `off_32290[1]`: jump to `loc_322BC`.
    Loc322BC,
    /// `off_32290[2]`: jump to `loc_322DA`.
    Loc322DA,
    /// `off_32290[3]`: jump to `loc_322FC`.
    Loc322FC,
    /// `off_32290[4]`: jump to `loc_32308`.
    Loc32308,
    /// `off_32290[5]`: jump to `loc_32310`.
    Loc32310,
    /// Byte index outside `off_32290` (out-of-range dispatch).
    Invalid(u8),
}

/// ASM `off_32290` jump table for `loc_32256`/`loc_3231C` dispatch.
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) const TAR_MONSTER_LOC_32290_TABLE: [TarMonsterLoc32290Target; 6] = [
    TarMonsterLoc32290Target::Loc322A8,
    TarMonsterLoc32290Target::Loc322BC,
    TarMonsterLoc32290Target::Loc322DA,
    TarMonsterLoc32290Target::Loc322FC,
    TarMonsterLoc32290Target::Loc32308,
    TarMonsterLoc32290Target::Loc32310,
];

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) enum TarMonsterLoc32256Route {
    Restart,
    Dispatch(TarMonsterLoc32290Target),
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) struct TarMonsterLoc3224EReturn {
    pub(crate) collision_type: u16,
    pub(crate) jump_table: &'static [TarMonsterLoc32290Target; 6],
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) struct TarMonsterLoc32256Return {
    pub(crate) raw_command: i16,
    pub(crate) route: TarMonsterLoc32256Route,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) struct TarMonsterWaitLoopLoc322AA {
    pub(crate) hibernate_frames: u32,
    pub(crate) checks_object_range_each_frame: bool,
    pub(crate) polls_collision_each_frame: bool,
    pub(crate) next_collision_type: u16,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) struct TarMonsterAppearSoundLoc322C0 {
    pub(crate) sound_effect_label: &'static str,
    pub(crate) preserves_script_command_d0: bool,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) struct TarMonsterAppearAnimationLoc322C8 {
    pub(crate) animation_label: &'static str,
    pub(crate) hibernate_until_animation_finished: bool,
    pub(crate) animation_frames: u16,
    pub(crate) resumes_control_script: bool,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) struct TarMonsterThirdAttackWaitLoc3235E {
    pub(crate) dbf_seed: u16,
    pub(crate) hibernate_frames: u16,
    pub(crate) checks_object_range_each_frame: bool,
    pub(crate) polls_collision_each_frame: bool,
    pub(crate) restored_sprite_link: &'static str,
    pub(crate) resumes_control_script: bool,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) enum TarMonsterCollisionGateLoc3237A {
    Kill,
    Return,
    NonZero { captured_collision_type: u16 },
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) enum TarMonsterCollisionRouteLoc32386 {
    ReturnOnly,
    Hurt,
    BerzerkerKill,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) struct TarMonsterBerzerkerCollisionLoc323AA {
    pub(crate) route: TarMonsterCollisionRouteLoc32386,
    pub(crate) clears_kid_x_velocity: bool,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) enum TarMonsterHurtRouteLoc323C0 {
    Death,
    HurtAnimation,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) struct TarMonsterHurtCollisionLoc323C0 {
    pub(crate) saved_child_visible: bool,
    pub(crate) child_visible_during_response: bool,
    pub(crate) remaining_hitpoints: u16,
    pub(crate) route: TarMonsterHurtRouteLoc323C0,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) struct TarMonsterHurtAnimationLoc323D6 {
    pub(crate) animation_label: &'static str,
    pub(crate) hibernate_until_animation_finished: bool,
    pub(crate) restored_child_visible: bool,
    pub(crate) resumes_control_script: bool,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) struct TarMonsterRespawnMarkerLoc3242C {
    pub(crate) index: usize,
    pub(crate) word_value: u16,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) enum TarMonsterDeathStatusRouteLoc3243A {
    WriteRespawnMarker(TarMonsterRespawnMarkerLoc3242C),
    SubtractSpawnCounter { index: usize, decrement: u16 },
    DeleteOnly,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) struct TarMonsterDeathStatusUpdateLoc3243A {
    pub(crate) route: TarMonsterDeathStatusRouteLoc3243A,
    pub(crate) deletes_current_object: bool,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) struct TarMonsterPostSubmergeHoldLoc32454 {
    pub(crate) child_visible_during_animation: bool,
    pub(crate) animation_label: &'static str,
    pub(crate) hibernate_until_animation_finished: bool,
    pub(crate) dbf_seed: u16,
    pub(crate) hibernate_frames: u16,
    pub(crate) checks_object_range_each_frame: bool,
    pub(crate) polls_collision_each_frame: bool,
    pub(crate) restarts_control_script: bool,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) struct TarMonsterCollisionDispatchLoc32386 {
    pub(crate) next_collision_type: u16,
    pub(crate) route: TarMonsterCollisionRouteLoc32386,
    pub(crate) clears_kid_x_velocity: bool,
}

#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn tar_monster_loc_32248() -> &'static [u8] {
    // ASM loc_32248: `lea (unk_32264).l,a0` loads this control script pointer.
    &TAR_MONSTER_CONTROL_BYTES_LOC_32248
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn tar_monster_loc_3224e() -> TarMonsterLoc3224EReturn {
    // ASM loc_3224E: clear `collision_type(a3)` and preload `off_32290` into `a1`.
    TarMonsterLoc3224EReturn {
        collision_type: 0,
        jump_table: &TAR_MONSTER_LOC_32290_TABLE,
    }
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn tar_monster_loc_32256(raw_command: i16) -> TarMonsterLoc32256Return {
    // ASM loc_32256: read one signed word; negative -> restart script at loc_32248;
    // otherwise `move.b d0,d7` indexes `off_32290` by low byte and jumps indirectly.
    if raw_command < 0 {
        return TarMonsterLoc32256Return {
            raw_command,
            route: TarMonsterLoc32256Route::Restart,
        };
    }

    let index = (raw_command & 0x00FF) as u8;
    let target = match index {
        0 => TarMonsterLoc32290Target::Loc322A8,
        1 => TarMonsterLoc32290Target::Loc322BC,
        2 => TarMonsterLoc32290Target::Loc322DA,
        3 => TarMonsterLoc32290Target::Loc322FC,
        4 => TarMonsterLoc32290Target::Loc32308,
        5 => TarMonsterLoc32290Target::Loc32310,
        _ => TarMonsterLoc32290Target::Invalid(index),
    };

    TarMonsterLoc32256Return {
        raw_command,
        route: TarMonsterLoc32256Route::Dispatch(target),
    }
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn tar_monster_wait_loop_loc_322aa(
    d0_after_loc_322a8: u16,
) -> TarMonsterWaitLoopLoc322AA {
    // ASM loc_322AA: hibernate one frame, check range, poll collision, then
    // `dbf d0,loc_322AA`. After the inclusive DBF loop, branch to loc_3224E.
    TarMonsterWaitLoopLoc322AA {
        hibernate_frames: u32::from(d0_after_loc_322a8) + 1,
        checks_object_range_each_frame: true,
        polls_collision_each_frame: true,
        next_collision_type: tar_monster_loc_3224e().collision_type,
    }
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn tar_monster_appear_sound_loc_322c0() -> TarMonsterAppearSoundLoc322C0 {
    // ASM loc_322C0: call `j_PlaySound` with `sfx_Tar_Monster_appears`, then
    // restore the script command from the stack before animation setup.
    TarMonsterAppearSoundLoc322C0 {
        sound_effect_label: "sfx_Tar_Monster_appears",
        preserves_script_command_d0: true,
    }
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn tar_monster_appear_animation_loc_322c8() -> TarMonsterAppearAnimationLoc322C8 {
    // ASM loc_322C8: load `stru_3272E`, initialise animation, hibernate until
    // it finishes, then branch back to the control-script dispatcher.
    TarMonsterAppearAnimationLoc322C8 {
        animation_label: "stru_3272E",
        hibernate_until_animation_finished: true,
        animation_frames: TAR_MONSTER_EMERGE_FRAMES,
        resumes_control_script: true,
    }
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn tar_monster_third_attack_wait_loc_3235e() -> TarMonsterThirdAttackWaitLoc3235E {
    // ASM loc_3235E seeds d0 with 8, then the following loop hibernates,
    // range-checks, and polls collision once per inclusive DBF iteration before restoring
    // the idle sprite link and resuming the bytecode loop at loc_3224E.
    TarMonsterThirdAttackWaitLoc3235E {
        dbf_seed: 8,
        hibernate_frames: 9,
        checks_object_range_each_frame: true,
        polls_collision_each_frame: true,
        restored_sprite_link: "LnkTo_unk_C77D6",
        resumes_control_script: true,
    }
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn tar_monster_collision_gate_loc_3237a(
    collision_type: i16,
) -> TarMonsterCollisionGateLoc3237A {
    // ASM loc_3237A copies `collision_type(a3)` into d7; negative values go
    // straight to the death path, zero returns, and positive values continue
    // with the captured collision id.
    if collision_type < 0 {
        return TarMonsterCollisionGateLoc3237A::Kill;
    }
    if collision_type == 0 {
        return TarMonsterCollisionGateLoc3237A::Return;
    }
    TarMonsterCollisionGateLoc3237A::NonZero {
        captured_collision_type: collision_type as u16,
    }
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn tar_monster_berzerker_collision_loc_323AA(
    berzerker_charging: bool,
) -> TarMonsterBerzerkerCollisionLoc323AA {
    // ASM loc_323AA: side contact only kills while `Berzerker_charging` is
    // non-zero. That accepted branch clears Kid's `$26(a4)` x velocity before
    // skipping the collision subroutine return and entering loc_323E6.
    if berzerker_charging {
        return TarMonsterBerzerkerCollisionLoc323AA {
            route: TarMonsterCollisionRouteLoc32386::BerzerkerKill,
            clears_kid_x_velocity: true,
        };
    }

    TarMonsterBerzerkerCollisionLoc323AA {
        route: TarMonsterCollisionRouteLoc32386::ReturnOnly,
        clears_kid_x_velocity: false,
    }
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn tar_monster_hurt_collision_loc_323C0(
    current_hitpoints: u16,
    child_visible: bool,
) -> TarMonsterHurtCollisionLoc323C0 {
    // ASM loc_323C0 saves `$13(a2)` into d2, clears `$13(a2)`, then decrements
    // d3. A zero result enters the death path; otherwise the hurt animation
    // runs while the linked child sprite stays hidden.
    let remaining_hitpoints = current_hitpoints.wrapping_sub(1);
    let route = if remaining_hitpoints == 0 {
        TarMonsterHurtRouteLoc323C0::Death
    } else {
        TarMonsterHurtRouteLoc323C0::HurtAnimation
    };

    TarMonsterHurtCollisionLoc323C0 {
        saved_child_visible: child_visible,
        child_visible_during_response: false,
        remaining_hitpoints,
        route,
    }
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn tar_monster_hurt_animation_loc_323D6(
    saved_child_visible: bool,
) -> TarMonsterHurtAnimationLoc323D6 {
    // ASM loc_323D6 runs the `stru_326EE` hurt animation to completion, restores
    // the child sprite visibility saved in d2, then resumes at loc_3224E.
    TarMonsterHurtAnimationLoc323D6 {
        animation_label: "stru_326EE",
        hibernate_until_animation_finished: true,
        restored_child_visible: saved_child_visible,
        resumes_control_script: true,
    }
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn tar_monster_respawn_marker_loc_3242C(
    object_status_slot: u8,
) -> TarMonsterRespawnMarkerLoc3242C {
    // ASM loc_3242C: the positive `$42(a5)` byte is doubled as a word offset
    // into EnemyStatus_Table, then `$2168` is written as the respawn marker.
    TarMonsterRespawnMarkerLoc3242C {
        index: usize::from(object_status_slot),
        word_value: 0x2168,
    }
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn tar_monster_death_status_update_loc_3243A(
    object_status_slot: u8,
) -> TarMonsterDeathStatusUpdateLoc3243A {
    // ASM loc_3243A: every status-table route converges on
    // j_Delete_CurrentObject; bit 7 clear reaches loc_3242C, bit 7+6 subtracts
    // `$400`, and bit 7 without bit 6 deletes without a table write.
    let route = if object_status_slot & 0x80 == 0 {
        TarMonsterDeathStatusRouteLoc3243A::WriteRespawnMarker(
            tar_monster_respawn_marker_loc_3242C(object_status_slot),
        )
    } else if object_status_slot & 0x40 != 0 {
        TarMonsterDeathStatusRouteLoc3243A::SubtractSpawnCounter {
            index: usize::from(object_status_slot & 0x3F),
            decrement: 0x0400,
        }
    } else {
        TarMonsterDeathStatusRouteLoc3243A::DeleteOnly
    };

    TarMonsterDeathStatusUpdateLoc3243A {
        route,
        deletes_current_object: true,
    }
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn tar_monster_post_submerge_hold_loc_32454() -> TarMonsterPostSubmergeHoldLoc32454 {
    // ASM loc_32454: after hiding the child and waiting for `stru_3274C`, d0
    // starts at `$50`; the one-frame hibernate/range/collision loop is DBF
    // inclusive and then branches back to loc_32248.
    TarMonsterPostSubmergeHoldLoc32454 {
        child_visible_during_animation: false,
        animation_label: "stru_3274C",
        hibernate_until_animation_finished: true,
        dbf_seed: 0x50,
        hibernate_frames: 0x51,
        checks_object_range_each_frame: true,
        polls_collision_each_frame: true,
        restarts_control_script: true,
    }
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn tar_monster_collision_dispatch_loc_32386(
    collision_type: u16,
    berzerker_charging: bool,
) -> TarMonsterCollisionDispatchLoc32386 {
    // ASM loc_32386 immediately clears `collision_type(a3)`, then accepts
    // `$1C/$2C` as ordinary hit ids. `$20/$24` kill only while the Berzerker
    // charge flag is set; the charge path also clears Kid's x velocity.
    let route = match collision_type {
        0x001C | 0x002C => TarMonsterCollisionRouteLoc32386::Hurt,
        0x0020 | 0x0024 => tar_monster_berzerker_collision_loc_323AA(berzerker_charging).route,
        _ => TarMonsterCollisionRouteLoc32386::ReturnOnly,
    };
    TarMonsterCollisionDispatchLoc32386 {
        next_collision_type: 0,
        route,
        clears_kid_x_velocity: matches!(route, TarMonsterCollisionRouteLoc32386::BerzerkerKill),
    }
}

/// HP lookup table for TarMonster init (`Hitpoints_Enemy_05`, Tar_Monster.asm).
pub(crate) const TAR_MONSTER_HITPOINTS_BY_LEVEL: [u8; 4] = [2, 4, 6, 0];

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) struct TarMonsterInitLoc321C6 {
    pub(crate) hit_points: u8,
    pub(crate) x_pos: u32,
    pub(crate) y_pos: u32,
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn tar_monster_hit_points_loc_321c6(enemy_level: u8) -> u8 {
    // ASM loc_321C6 indexes `dc.b Hitpoints_Enemy_05(pc,d3.w),d3`.
    TAR_MONSTER_HITPOINTS_BY_LEVEL[usize::from(enemy_level.min(3))]
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn tar_monster_init_loc_321c6(
    queued_x_word: u16,
    queued_y_word: u16,
    enemy_level: u8,
) -> TarMonsterInitLoc321C6 {
    // ASM loc_321C6 copies layout words 4/6 directly into x_pos/y_pos and uses the
    // layout level word as the Hitpoints_Enemy_05 table index.
    TarMonsterInitLoc321C6 {
        hit_points: tar_monster_hit_points_loc_321c6(enemy_level),
        x_pos: u32::from(queued_x_word) << 16,
        y_pos: u32::from(queued_y_word) << 16,
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) struct TarMonsterInitLoc321E4 {
    pub(crate) hit_points: u8,
    pub(crate) priority: u8,
    pub(crate) object_meta: u16,
    pub(crate) x_pos: u32,
    pub(crate) y_pos: u32,
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn tar_monster_init_loc_321e4(
    queued_x_word: u16,
    queued_y_word: u16,
    enemy_level: u8,
    blank_top_rows: u16,
) -> TarMonsterInitLoc321E4 {
    // ASM loc_321E4 calls sub_36FF4, sets object priority and object_meta to 5,
    // then runs the shared setup path (loc_32146) before storing `a3` in `$3A(a5)`.
    let base = tar_monster_init_loc_321c6(queued_x_word, queued_y_word, enemy_level);
    TarMonsterInitLoc321E4 {
        hit_points: base.hit_points,
        priority: 0,
        object_meta: 5,
        x_pos: base.x_pos,
        y_pos: spawn_y_pos_sub_36FF4(
            EnemyFamily::TarMonster,
            u32::from(queued_y_word),
            blank_top_rows,
        ),
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) struct TarMonsterChildInitLoc32200 {
    pub(crate) sprite_frame_link: u16,
    pub(crate) object_meta: u16,
    pub(crate) priority: u8,
    pub(crate) x_pos: u32,
    pub(crate) y_pos: u32,
    pub(crate) status_flag_13: bool,
    pub(crate) animation_phase: u16,
    pub(crate) vram_tile: u16,
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn tar_monster_child_init_loc_32200(
    parent_init: TarMonsterInitLoc321E4,
    sprite_frame_link: u16,
) -> TarMonsterChildInitLoc32200 {
    // ASM loc_32200 configures the child gfx object: writes the linked sprite frame,
    // runs the shared setup path (loc_32146), sets status/animation fields, then
    // links it through `$FFFFF86A` as an active object.
    TarMonsterChildInitLoc32200 {
        sprite_frame_link,
        object_meta: 5,
        priority: 0,
        x_pos: parent_init.x_pos,
        y_pos: parent_init.y_pos,
        status_flag_13: true,
        animation_phase: 2,
        vram_tile: 0x0100,
    }
}

/// Step a TarMonster through its bytecode-driven emerge/attack/submerge cycle.
/// ASM: Tar_Monster.asm, jump table off_32290.
/// Submerged → Emerging → Visible → Attacking → Submerging → Submerged (loop).
/// While Submerged, the enemy is invulnerable and invisible (handled by collision/render).
pub(crate) fn step_tar_monster(enemy: &mut ActiveEnemy, projectiles: &mut Vec<EnemyProjectile>) {
    if enemy.wait_timer > 0 {
        // Check if we need to fire during attack sequence
        if enemy.tar_monster_state == TarMonsterState::Attacking {
            let elapsed = TAR_MONSTER_ATTACK_FRAMES - enemy.wait_timer;
            if elapsed == TAR_MONSTER_ATTACK_FIRE_FRAME {
                // ASM loc_3231C/loc_32348: spawn the glob after the first `dbf` loop.
                let xv = match enemy.facing {
                    Direction::Right => TAR_MONSTER_GLOB_SPEED_X,
                    Direction::Left => -TAR_MONSTER_GLOB_SPEED_X,
                };
                let spawn_y = (enemy.y_pos as i64 - (0x12 << 16)) as u32; // -18px Y offset
                let spawn_x = if matches!(enemy.facing, Direction::Left) {
                    (enemy.x_pos as i64 - (0x19 << 16)) as u32 // -25px X offset
                } else {
                    (enemy.x_pos as i64 + (0x19 << 16)) as u32 // +25px (mirrored)
                };
                projectiles.push(EnemyProjectile {
                    x_pos: spawn_x,
                    y_pos: spawn_y,
                    x_velocity: xv,
                    y_velocity: 0,
                    lifetime: 120,
                    gravity: TAR_MONSTER_GLOB_GRAVITY,
                    friendly: false,
                    homing_accel: 0,
                    speed_cap: 0,
                    wall_bounce: false,
                    ..Default::default()
                });
            }
        }
        enemy.wait_timer -= 1;
        return;
    }
    match enemy.tar_monster_state {
        TarMonsterState::Submerged => {
            enemy.tar_monster_state = TarMonsterState::Emerging;
            enemy.wait_timer = TAR_MONSTER_EMERGE_FRAMES;
        }
        TarMonsterState::Emerging => {
            // Flip direction on emerge (ASM Op 3: not.b x_direction)
            enemy.facing = opposite_direction(enemy.facing);
            enemy.tar_monster_state = TarMonsterState::Visible;
            enemy.wait_timer = TAR_MONSTER_VISIBLE_FRAMES;
        }
        TarMonsterState::Visible => {
            enemy.tar_monster_state = TarMonsterState::Attacking;
            enemy.wait_timer = TAR_MONSTER_ATTACK_FRAMES;
        }
        TarMonsterState::Attacking => {
            enemy.tar_monster_state = TarMonsterState::Submerging;
            enemy.wait_timer = TAR_MONSTER_SUBMERGE_FRAMES;
        }
        TarMonsterState::Submerging => {
            enemy.tar_monster_state = TarMonsterState::Submerged;
            enemy.wait_timer = TAR_MONSTER_SUBMERGED_FRAMES;
        }
    }
}

/// Spawn 5 tar blob projectiles in a spread pattern when a TarMonster is damaged.
pub(crate) fn spawn_tar_monster_retaliation(
    projectiles: &mut Vec<EnemyProjectile>,
    x: u32,
    y: u32,
) {
    let blobs: [(i32, i32); 5] = [
        (0x1_0000, -0x1_0000),  // up-right
        (-0x1_0000, -0x1_0000), // up-left
        (0x0_8000, -0x2_0000),  // steep up-right
        (-0x0_8000, -0x2_0000), // steep up-left
        (0x0_2000, -0x1_C000),  // gentle arc
    ];
    for (xv, yv) in blobs {
        projectiles.push(EnemyProjectile {
            x_pos: x,
            y_pos: y,
            x_velocity: xv,
            y_velocity: yv,
            lifetime: 60,
            gravity: 0x1000,
            friendly: false,
            homing_accel: 0,
            speed_cap: 0,
            wall_bounce: false,
            ..Default::default()
        });
    }
}

/// Fire Demon death trail: spawn 3 fire fragments arcing outward from death position.
pub(crate) fn spawn_fire_demon_death_trail(projectiles: &mut Vec<EnemyProjectile>, x: u32, y: u32) {
    // TODO(fidelity): visual — fire trail uses generic projectile rendering.
    // ASM uses fire fragment VRAM tiles. Needs ROM sprite extraction for visual parity.
    let fragments: [(i32, i32); 3] = [
        (0x1_8000, -0x2_0000),  // right arc
        (-0x1_8000, -0x2_0000), // left arc
        (0, -0x2_8000),         // straight up
    ];
    for (xv, yv) in fragments {
        projectiles.push(EnemyProjectile {
            x_pos: x,
            y_pos: y,
            x_velocity: xv,
            y_velocity: yv,
            lifetime: 45,
            gravity: 0x1800,
            friendly: false,
            homing_accel: 0,
            speed_cap: 0,
            wall_bounce: false,
            ..Default::default()
        });
    }
}

// --- DragonWalking constants ---
// From ASM Dragon.asm: initial cooldown $12C (300f, line 95), reset after breath $1F4 (500f, line 194).
pub(crate) const DRAGON_WALKING_BREATH_INITIAL: u16 = 300;
pub(crate) const DRAGON_WALKING_BREATH_RESET: u16 = 500;
/// ASM loc_3D02A: breath is a stationary GfxObject, not a travelling projectile.
/// Animation stru_3CCC8 lasts 68 frames (8+10+10+10+10+10+10), after which
/// $18(a3) signals completion. No velocity is applied in the ROM.
pub(crate) const DRAGON_WALKING_BREATH_LIFETIME: u16 = 68;
pub(crate) const DRAGON_WALKING_BREATH_HOLD_FRAMES: u16 = 68;
pub(crate) const DRAGON_WALKING_RECOVER_FRAMES: u16 = 0;
pub(crate) const DRAGON_WALKING_BREATH_OFFSET_PX: i32 = 0x1C;
pub(crate) const DRAGON_WALKING_GRAVITY: i32 = 0x4000;
pub(crate) const SHARED_WALKER_STATE_PATROL: i16 = 10;
pub(crate) const SHARED_WALKER_STATE_FACE_RIGHT: i16 = 1;
pub(crate) const SHARED_WALKER_STATE_FACE_LEFT: i16 = 2;
const SHARED_WALKER_STATE_COOLDOWN: i16 = -120;
const SHARED_WALKER_DETECT_RANGE: i32 = 0x60;
const SHARED_WALKER_RELEASE_RANGE: i32 = 0x70;
const DRAGON_WALKING_PATROL_SHORT_FRAMES: u16 = 0x32;
const DRAGON_WALKING_PATROL_LONG_FRAMES: u16 = 0x12C;
const DRAGON_WALKING_PHASE_WALK: u8 = 0;
const DRAGON_WALKING_PHASE_BREATH: u8 = 1;
const DRAGON_WALKING_PHASE_RECOVER: u8 = 2;

enum DragonWalkingWalkStep {
    Walked,
    Idled,
    StartedAirborne,
}

pub(crate) fn spawn_shared_walker_state(family: EnemyFamily) -> i16 {
    match family {
        EnemyFamily::DragonWalking | EnemyFamily::Lion => SHARED_WALKER_STATE_PATROL,
        _ => 0,
    }
}

pub(crate) fn spawn_shared_walker_patrol_timer(family: EnemyFamily) -> u16 {
    match family {
        EnemyFamily::DragonWalking => DRAGON_WALKING_PATROL_LONG_FRAMES,
        EnemyFamily::Lion => LION_CHARGE_TIMER_INIT,
        _ => 0,
    }
}

pub(crate) fn spawn_shared_walker_patrol_short_next(family: EnemyFamily) -> bool {
    matches!(family, EnemyFamily::DragonWalking)
}

fn shared_walker_turnaround(enemy: &mut ActiveEnemy) {
    reverse_enemy_direction(enemy);
    if enemy.shared_walker_state > 0 && enemy.shared_walker_state != SHARED_WALKER_STATE_PATROL {
        enemy.shared_walker_state = SHARED_WALKER_STATE_COOLDOWN;
    }
}

fn shared_walker_should_tick_patrol(enemy: &ActiveEnemy) -> bool {
    enemy.shared_walker_state <= 0 || enemy.shared_walker_state == SHARED_WALKER_STATE_PATROL
}

fn shared_walker_tick_patrol(enemy: &mut ActiveEnemy) -> bool {
    if !shared_walker_should_tick_patrol(enemy) || enemy.shared_walker_patrol_timer == 0 {
        return false;
    }

    enemy.shared_walker_patrol_timer -= 1;
    if enemy.shared_walker_patrol_timer != 0 {
        return false;
    }

    enemy.shared_walker_patrol_timer = match enemy.family {
        EnemyFamily::DragonWalking => {
            let next = if enemy.shared_walker_patrol_short_next {
                DRAGON_WALKING_PATROL_SHORT_FRAMES
            } else {
                DRAGON_WALKING_PATROL_LONG_FRAMES
            };
            enemy.shared_walker_patrol_short_next = !enemy.shared_walker_patrol_short_next;
            next
        }
        EnemyFamily::Lion => LION_CHARGE_TIMER_INIT,
        _ => 0,
    };
    true
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct DragonWalkingFloorSupport {
    selected: terrain::FloorProbe,
    center: terrain::FloorProbe,
    left: terrain::FloorProbe,
    right: terrain::FloorProbe,
}

fn dragon_walking_floor_support(
    grid: &terrain::BlockGrid,
    center_x_px: i32,
    foot_y_px: i32,
) -> DragonWalkingFloorSupport {
    let center = terrain::probe_floor(grid, center_x_px, foot_y_px);
    let left = terrain::probe_floor(grid, center_x_px - ENEMY_PROBE_HALF_WIDTH, foot_y_px);
    let right = terrain::probe_floor(grid, center_x_px + ENEMY_PROBE_HALF_WIDTH, foot_y_px);
    DragonWalkingFloorSupport {
        selected: terrain::probe_floor_hitbox(grid, center_x_px, foot_y_px, ENEMY_PROBE_HALF_WIDTH),
        center,
        left,
        right,
    }
}

fn dragon_walking_probe_wall(
    grid: &terrain::BlockGrid,
    center_x_px: i32,
    foot_y_px: i32,
    moving_right: bool,
) -> terrain::WallProbe {
    let wall_center_x = if moving_right {
        center_x_px
    } else {
        center_x_px - 1
    };
    let edge_x = if moving_right {
        wall_center_x + ENEMY_PROBE_HALF_WIDTH + 1
    } else {
        wall_center_x - ENEMY_PROBE_HALF_WIDTH
    };

    // ORIGINAL_QUIRK: the shared Dragon/Lion grounded walker helpers
    // (`sub_36972`/`sub_36A58`) treat hurt tiles as blocking side contacts for
    // the turnaround path. The frame-3126 TAS frontier depends on a spike
    // shoulder counting as a wall.
    let lower_body_y = foot_y_px - 8;
    let upper_body_y = foot_y_px - ENEMY_PROBE_HEIGHT + 8;
    let probe_ys = [
        lower_body_y,
        foot_y_px - ENEMY_PROBE_HEIGHT / 2,
        upper_body_y,
    ];

    for &py in &probe_ys {
        let probe = terrain::probe_at(grid, edge_x, py);
        if matches!(
            probe.cell.collision,
            terrain::CollisionKind::Solid | terrain::CollisionKind::Spikes
        ) {
            let wall_x = if moving_right {
                i32::from(probe.cell_x) * 16
            } else {
                (i32::from(probe.cell_x) + 1) * 16
            };
            return terrain::WallProbe {
                hit: true,
                wall_x_px: wall_x,
                cell: probe.cell,
                cell_x: probe.cell_x,
                cell_y: probe.cell_y,
            };
        }
    }

    terrain::WallProbe {
        hit: false,
        wall_x_px: 0,
        cell: terrain::Cell::from_be_bytes(0, 0),
        cell_x: 0,
        cell_y: 0,
    }
}

fn dragon_walking_wall_snap_x(wall_x_px: i32, moving_right: bool) -> u32 {
    let wall_fp = (wall_x_px as u32) << 16;
    let edge_fp = (ENEMY_PROBE_HALF_WIDTH as u32) << 16;
    if moving_right {
        wall_fp.saturating_sub(edge_fp)
    } else {
        wall_fp + edge_fp + (1 << 16)
    }
}

fn dragon_walking_blocked(enemy: &ActiveEnemy, block_grid: Option<&terrain::BlockGrid>) -> bool {
    let Some(grid) = block_grid else {
        return false;
    };
    let walk_speed = pacer_walk_speed(enemy.family, enemy.hit_points);
    let x_velocity = horizontal_velocity(enemy.facing, walk_speed);
    let moving_right = x_velocity >= 0;
    let next_x = (enemy.x_pos as i64 + i64::from(x_velocity)) as u32;
    let cx = (next_x >> 16) as i32;
    let cy = (enemy.y_pos >> 16) as i32;
    let wall = dragon_walking_probe_wall(grid, cx, cy, moving_right);
    if wall.hit {
        return true;
    }
    !dragon_walking_floor_support(grid, cx, cy)
        .selected
        .on_ground
}

fn step_dragon_walking_walk(
    enemy: &mut ActiveEnemy,
    block_grid: Option<&terrain::BlockGrid>,
    player_x_pos: Option<u32>,
) -> DragonWalkingWalkStep {
    // ASM routes the positive shared-walker states through loc_3E4CE/loc_3E50A,
    // which swap $50(a5) to the stationary $4C(a5) slot before returning to the
    // grounded walker loop. DragonWalking leaves $4C(a5) at zero, so the alert
    // branch holds position until Kid leaves the wider release window.
    let walk_speed = if enemy.shared_walker_state > 0
        && enemy.shared_walker_state != SHARED_WALKER_STATE_PATROL
    {
        0
    } else {
        pacer_walk_speed(enemy.family, enemy.hit_points)
    };
    let walk_delta = horizontal_velocity(enemy.facing, walk_speed);
    let moving_right = walk_delta >= 0;

    if shared_walker_tick_patrol(enemy) {
        reverse_enemy_direction(enemy);
        enemy.x_velocity = 0;
        enemy.y_velocity = 0;
        enemy.shared_walker_debug_branch = Some("patrol_timer_turn");
        return DragonWalkingWalkStep::Idled;
    }

    let current_y_px = (enemy.y_pos >> 16) as i32;

    if let Some(grid) = block_grid {
        let next_x = (enemy.x_pos as i64 + i64::from(walk_delta)) as u32;
        let cx = (next_x >> 16) as i32;
        let wall = dragon_walking_probe_wall(grid, cx, current_y_px, moving_right);
        let floor_support = dragon_walking_floor_support(grid, cx, current_y_px);
        enemy.shared_walker_terrain_trace = SharedWalkerTerrainTrace {
            next_x_px: Some(cx),
            wall: Some(wall),
            center_floor: Some(floor_support.center),
            left_floor: Some(floor_support.left),
            right_floor: Some(floor_support.right),
        };
        if wall.hit {
            enemy.x_pos = dragon_walking_wall_snap_x(wall.wall_x_px, moving_right);
            enemy.x_velocity = 0;
            enemy.y_velocity = 0;
            shared_walker_turnaround(enemy);
            enemy.shared_walker_debug_branch = Some("patrol_wall_turn");
            return DragonWalkingWalkStep::Idled;
        }

        let floor = floor_support.selected;
        if !floor.on_ground {
            enemy.on_ground = false;
            enemy.x_velocity = walk_delta;
            enemy.y_velocity = 0;
            enemy.y_pos = enemy.y_pos.saturating_add(1 << 16);
            enemy.shared_walker_debug_branch = Some("patrol_airborne_start");
            return DragonWalkingWalkStep::StartedAirborne;
        }

        enemy.y_pos = (floor.floor_y_px as u32) << 16;
        enemy.on_ground = true;
    }

    if enemy.shared_walker_state < 0 {
        enemy.shared_walker_state += 1;
    } else if enemy.shared_walker_state == 0 {
        if let Some(player_x_pos) = player_x_pos {
            let dx = ((player_x_pos as i64 - enemy.x_pos as i64) >> 16) as i32;
            enemy.shared_walker_dx_to_kid_px =
                Some(dx.clamp(i32::from(i16::MIN), i32::from(i16::MAX)) as i16);
            if dx.abs() <= SHARED_WALKER_DETECT_RANGE {
                if dx >= 0 {
                    enemy.facing = Direction::Right;
                    enemy.shared_walker_state = SHARED_WALKER_STATE_FACE_RIGHT;
                } else {
                    enemy.facing = Direction::Left;
                    enemy.shared_walker_state = SHARED_WALKER_STATE_FACE_LEFT;
                }
                enemy.x_velocity = 0;
                enemy.y_velocity = 0;
                enemy.shared_walker_debug_branch = Some("calm_face_kid");
                return DragonWalkingWalkStep::Idled;
            }
        }
    } else if enemy.shared_walker_state != SHARED_WALKER_STATE_PATROL
        && let Some(player_x_pos) = player_x_pos
    {
        let dx = ((player_x_pos as i64 - enemy.x_pos as i64) >> 16) as i32;
        enemy.shared_walker_dx_to_kid_px =
            Some(dx.clamp(i32::from(i16::MIN), i32::from(i16::MAX)) as i16);
        if dx.abs() > SHARED_WALKER_RELEASE_RANGE {
            enemy.x_velocity = 0;
            enemy.y_velocity = 0;
            shared_walker_turnaround(enemy);
            enemy.shared_walker_debug_branch = Some("alert_release_turn");
            return DragonWalkingWalkStep::Idled;
        }
    }

    enemy.x_pos = (enemy.x_pos as i64 + i64::from(walk_delta)) as u32;
    enemy.x_velocity = 0;
    enemy.y_velocity = 0;
    enemy.shared_walker_debug_branch = Some(
        if enemy.shared_walker_state > 0 && enemy.shared_walker_state != SHARED_WALKER_STATE_PATROL
        {
            "alert_hold"
        } else if enemy.shared_walker_state < 0 {
            "cooldown_walk"
        } else {
            "patrol_walk"
        },
    );
    DragonWalkingWalkStep::Walked
}

fn step_dragon_walking_airborne(enemy: &mut ActiveEnemy, block_grid: Option<&terrain::BlockGrid>) {
    enemy.x_pos = (enemy.x_pos as i64 + i64::from(enemy.x_velocity)) as u32;
    enemy.y_pos = (enemy.y_pos as i64 + i64::from(enemy.y_velocity)) as u32;

    let Some(grid) = block_grid else {
        enemy.y_velocity = (enemy.y_velocity + DRAGON_WALKING_GRAVITY).min(physics::MAX_FALL_SPEED);
        enemy.shared_walker_debug_branch = Some("airborne_fall");
        return;
    };

    let cx = (enemy.x_pos >> 16) as i32;
    let cy = (enemy.y_pos >> 16) as i32;
    let moving_right = enemy.x_velocity >= 0;
    let wall = dragon_walking_probe_wall(grid, cx, cy, moving_right);
    let floor = terrain::probe_floor(grid, cx, cy);
    enemy.shared_walker_terrain_trace = SharedWalkerTerrainTrace {
        next_x_px: Some(cx),
        wall: Some(wall),
        center_floor: Some(floor),
        left_floor: None,
        right_floor: None,
    };
    if wall.hit {
        enemy.x_pos = dragon_walking_wall_snap_x(wall.wall_x_px, moving_right);
        enemy.x_velocity = 0;
        enemy.shared_walker_debug_branch = Some("airborne_wall_snap");
    }

    if floor.on_ground {
        let floor_y_fp = (floor.floor_y_px as u32) << 16;
        if enemy.y_velocity >= 0 && enemy.y_pos >= floor_y_fp {
            enemy.y_pos = floor_y_fp;
            enemy.y_velocity = 0;
            enemy.x_velocity = 0;
            enemy.on_ground = true;
            if enemy.shared_walker_state == SHARED_WALKER_STATE_PATROL {
                enemy.shared_walker_debug_branch = Some("airborne_land_patrol");
            } else {
                enemy.shared_walker_state = SHARED_WALKER_STATE_COOLDOWN;
                enemy.shared_walker_debug_branch = Some("airborne_land_cooldown");
            }
            return;
        }
    }

    enemy.y_velocity = (enemy.y_velocity + DRAGON_WALKING_GRAVITY).min(physics::MAX_FALL_SPEED);
    enemy.shared_walker_debug_branch = Some("airborne_fall");
}
/// Fire breath for a walking dragon (called after step_pacer handles movement).
pub(crate) fn step_dragon_walking(
    enemy: &mut ActiveEnemy,
    block_grid: Option<&terrain::BlockGrid>,
    projectiles: &mut Vec<EnemyProjectile>,
    player_x_pos: Option<u32>,
) {
    enemy.shared_walker_debug_branch = None;
    enemy.shared_walker_dx_to_kid_px = None;
    enemy.shared_walker_terrain_trace = SharedWalkerTerrainTrace::default();

    let dragon_hurt_hibernate_armed = enemy.damage_flash_frames == ENEMY_DAMAGE_FLASH_FRAMES;
    if enemy.grounded_frames > 0 && !dragon_hurt_hibernate_armed {
        enemy.grounded_frames -= 1;
        enemy.x_velocity = 0;
        enemy.y_velocity = 0;
        enemy.shared_walker_debug_branch = Some("grounded_hibernate");
        return;
    }

    if enemy.dragon_script_cmd == DRAGON_WALKING_PHASE_WALK && enemy.wait_timer > 0 {
        enemy.wait_timer -= 1;
        if enemy.wait_timer == 0 {
            // ASM loc_3D02A: breath is a stationary animated hitbox at
            // the dragon's mouth (±28px). No velocity — the GfxObject
            // stays in place for the animation duration (stru_3CCC8, 68f).
            let x_offset = match enemy.facing {
                Direction::Right => DRAGON_WALKING_BREATH_OFFSET_PX,
                Direction::Left => -DRAGON_WALKING_BREATH_OFFSET_PX,
            };
            projectiles.push(EnemyProjectile {
                x_pos: (enemy.x_pos as i64 + ((i64::from(x_offset)) << 16)) as u32,
                y_pos: enemy.y_pos,
                x_velocity: 0,
                y_velocity: 0,
                lifetime: DRAGON_WALKING_BREATH_LIFETIME,
                gravity: 0,
                friendly: false,
                homing_accel: 0,
                speed_cap: 0,
                wall_bounce: false,
                ..Default::default()
            });
            enemy.wait_timer = DRAGON_WALKING_BREATH_RESET;
            enemy.dragon_script_cmd = DRAGON_WALKING_PHASE_BREATH;
            enemy.dragon_phase_frames = DRAGON_WALKING_BREATH_HOLD_FRAMES;
            enemy.x_velocity = 0;
            enemy.y_velocity = 0;
            return;
        }
    }

    if !enemy.on_ground {
        step_dragon_walking_airborne(enemy, block_grid);
        return;
    }

    let blocked = if enemy.dragon_script_cmd == DRAGON_WALKING_PHASE_WALK {
        match step_dragon_walking_walk(enemy, block_grid, player_x_pos) {
            DragonWalkingWalkStep::Walked => false,
            DragonWalkingWalkStep::Idled => false,
            DragonWalkingWalkStep::StartedAirborne => {
                step_dragon_walking_airborne(enemy, block_grid);
                return;
            }
        }
    } else {
        enemy.x_velocity = 0;
        enemy.y_velocity = 0;
        dragon_walking_blocked(enemy, block_grid)
    };

    if enemy.dragon_script_cmd == DRAGON_WALKING_PHASE_WALK && blocked {
        reverse_enemy_direction(enemy);
        enemy.x_velocity = 0;
        enemy.y_velocity = 0;
        return;
    }

    match enemy.dragon_script_cmd {
        DRAGON_WALKING_PHASE_BREATH => {
            if enemy.dragon_phase_frames > 0 {
                enemy.dragon_phase_frames -= 1;
            } else {
                enemy.dragon_script_cmd = DRAGON_WALKING_PHASE_RECOVER;
                enemy.dragon_phase_frames = DRAGON_WALKING_RECOVER_FRAMES;
            }
        }
        DRAGON_WALKING_PHASE_RECOVER => {
            if enemy.dragon_phase_frames > 0 {
                enemy.dragon_phase_frames -= 1;
            } else {
                enemy.dragon_script_cmd = DRAGON_WALKING_PHASE_WALK;
            }
        }
        _ => {}
    }
}

// --- Drips constants (Drips.asm) ---
// ASM Drips is a multi-object enemy: ceiling attachment, forming animation,
// animation-driven drop, splash, and surface-crawl spawning.
// Native approximation: periodic projectile drop with ASM-matched timing.
// Random dormant delay: Drips.asm loc_3340E uses `(RNG >> 2)`,
// then XORs the low byte with Kid's integer X low byte.
// Forming animation: ~35f. Total interval ≈ 67f average.
// Drop child starts with zero vertical speed, then gains $2000/frame gravity
// until it hits the floor. ASM loc_33424 clears `$2A(a1)` before the fall and
// loc_3345C adds `#$2000` every frame.
// Surface-crawl spawning (max 5 crawlers, x_vel=$3000) not implemented —
// would require multi-object child spawning system.
/// Drop interval: forming animation (~35f) + average random delay (~32f) = ~67f.
pub(crate) const DRIPS_DROP_INTERVAL: u16 = 67;
/// ASM loc_33424 clears the drop child's y-velocity before the fall begins.
pub(crate) const DRIPS_DROP_INITIAL_Y_SPEED: i32 = 0;
/// Gravity accumulation: ASM loc_3345C `addi.l #$2000,$2A(a1)`.
pub(crate) const DRIPS_DROP_GRAVITY: i32 = 0x2000;
/// Native fallback lifetime for the falling drop. The ROM deletes on terrain
/// contact after the splash sequence; native keeps a generous timer in case the
/// projectile never reaches solid terrain.
pub(crate) const DRIPS_DROP_LIFETIME: u16 = 120;
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) const DRIPS_COLID_CEILING: u16 = 0x0010;

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

pub(crate) fn drips_wait_timer_with_rng_loc_3340e(rng: &mut RomRandomState, kid_x_pos: u32) -> u16 {
    let x_pos_bytes = kid_x_pos.to_be_bytes();
    let kid_x_word = u16::from_be_bytes([x_pos_bytes[0], x_pos_bytes[1]]);
    drips_wait_timer_loc_3340e(rng.next_byte(), kid_x_word)
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
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) struct DripsSplashResetLoc33488 {
    pub(crate) drop_is_moved: bool,
    pub(crate) drop_status_flag: bool,
    pub(crate) drop_collision_type: u16,
    pub(crate) drop_y_pos: u32,
    pub(crate) surface_child_status_flag: bool,
    pub(crate) surface_child_x_pos: u32,
    pub(crate) surface_child_y_pos: u32,
}

#[cfg_attr(not(test), allow(dead_code))]
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
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) struct DripsVelocityDeltasLoc33526 {
    pub(crate) x_delta: i32,
    pub(crate) y_delta: i32,
}

#[cfg_attr(not(test), allow(dead_code))]
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
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) struct DripsVelocityApplyLoc33546 {
    pub(crate) x_velocity: i32,
    pub(crate) y_velocity: i32,
}

#[cfg_attr(not(test), allow(dead_code))]
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
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) enum DripsRightBoundsNextLoc33588 {
    ContactLoc335D4,
    CheckLeftLoc335AA,
    CheckVerticalLoc335B0,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) enum DripsLeftBoundsNextLoc335AA {
    ContactLoc335D4,
    CheckVerticalLoc335B0,
}

#[cfg_attr(not(test), allow(dead_code))]
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

#[cfg_attr(not(test), allow(dead_code))]
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

#[cfg_attr(not(test), allow(dead_code))]
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

#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn drips_bounds_no_contact_result_loc_335d0() -> u16 {
    // ASM Drips.asm loc_335D0: the in-bounds return tail moves 0 into d5.
    0
}

#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn drips_bounds_contact_result_loc_335d4() -> u16 {
    // ASM Drips.asm loc_335D4: the out-of-bounds return tail moves 1 into d5.
    1
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(not(test), allow(dead_code))]
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

#[cfg_attr(not(test), allow(dead_code))]
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

/// Step a Drips enemy: stationary ceiling hanger that periodically drops projectiles.
/// ASM: Drips.asm — forming animation → drop at 10px/frame → splash → reform → repeat.
/// Native approximation uses a physics projectile for the drop.
pub(crate) fn step_drips(enemy: &mut ActiveEnemy, projectiles: &mut Vec<EnemyProjectile>) {
    if enemy.wait_timer > 0 {
        enemy.wait_timer -= 1;
        return;
    }
    // ASM loc_33424/loc_3345C: the drop starts from rest and accelerates
    // downward under `$2000` gravity until it reaches the floor.
    projectiles.push(EnemyProjectile {
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
        ..Default::default()
    });
    enemy.wait_timer = DRIPS_DROP_INTERVAL;
}

// SpinningTwins: 8-entry orbit X offset table from ASM (word_3B9A0).
// Applied relative to spawn position during orbit phase.
pub(crate) const SPINNING_TWINS_ORBIT_OFFSETS: [i32; 8] =
    [-0x10, -0x0A, 0x05, 0x11, 0x10, 0x0A, -0x05, -0x11];
/// Orbit-to-walk transition: collision-triggered. ASM loc_3B904 checks
/// collision_type each frame — negative, $2C, or $1C all trigger the drop.
/// In native, we check player hitbox overlap with the orbiting enemy.
/// Hitbox half-width for orbit collision check (approximate from sprite size).
const SPINNING_TWINS_ORBIT_HITBOX_HALF: i32 = 16;
// Gravity applied each frame once the twins drop. ASM: $2800 FP.
pub(crate) const SPINNING_TWINS_GRAVITY: i32 = 0x2800;
// Launch Y velocity when transitioning from orbit. ASM: -$10000.
pub(crate) const SPINNING_TWINS_DROP_Y_VEL: i32 = -0x1_0000;
// Initial X velocity on drop: ASM sets $FFFC8000 (left) or $38000 (right).
pub(crate) const SPINNING_TWINS_DROP_X_VEL_LEFT: i32 = -0x3_8000;
pub(crate) const SPINNING_TWINS_DROP_X_VEL_RIGHT: i32 = 0x3_8000;
pub(crate) const SPINNING_TWINS_WALKING_CHILD_X_OFFSET_PX: u32 = 0x20;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) struct SpinningTwinsSecondWalkingChildInitLoc3BB06 {
    pub(crate) x_pos: u32,
    pub(crate) y_pos: u32,
    pub(crate) x_direction_flipped: bool,
    pub(crate) x_velocity: i32,
    pub(crate) y_velocity: i32,
}

#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn spinning_twins_second_walking_child_init_loc_3bb06(
    parent_x_pos: u32,
    parent_y_pos: u32,
) -> SpinningTwinsSecondWalkingChildInitLoc3BB06 {
    // ASM Spinning_Twins.asm loc_3BB06: the second walking child sees the
    // parent marker already set, keeps x_direction clear, and starts 32px to
    // the right before the shared launch path assigns rightward velocity.
    SpinningTwinsSecondWalkingChildInitLoc3BB06 {
        x_pos: parent_x_pos.wrapping_add(SPINNING_TWINS_WALKING_CHILD_X_OFFSET_PX << 16),
        y_pos: parent_y_pos,
        x_direction_flipped: false,
        x_velocity: SPINNING_TWINS_DROP_X_VEL_RIGHT,
        y_velocity: SPINNING_TWINS_DROP_Y_VEL,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) struct SpinningTwinsDropTransitionLoc3B93C {
    pub(crate) split_child_count: u8,
    pub(crate) orbit_child_hidden: bool,
    pub(crate) falling_visual_spawned: bool,
    pub(crate) original_deleted_after_animation: bool,
}

#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn spinning_twins_drop_transition_loc_3b93c(
    enemy: &mut ActiveEnemy,
) -> SpinningTwinsDropTransitionLoc3B93C {
    // ASM Spinning_Twins.asm loc_3B93C: collision hides the orbit child GFX,
    // allocates two walking twin objects plus one falling visual worker, then
    // deletes the original orbit object after its transition animation. The
    // walking children enter loc_3BA74 for their initial launch velocities.
    enemy.spinning_twins_phase = SpinningTwinsPhase::Walking;
    enemy.y_velocity = SPINNING_TWINS_DROP_Y_VEL;
    enemy.x_velocity = match enemy.facing {
        Direction::Left => SPINNING_TWINS_DROP_X_VEL_LEFT,
        Direction::Right => SPINNING_TWINS_DROP_X_VEL_RIGHT,
    };
    enemy.on_ground = false;

    SpinningTwinsDropTransitionLoc3B93C {
        split_child_count: 2,
        orbit_child_hidden: true,
        falling_visual_spawned: true,
        original_deleted_after_animation: true,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) struct SpinningTwinsFallingVisualInitLoc3B9B0 {
    pub(crate) x_pos: u32,
    pub(crate) y_pos: u32,
    pub(crate) priority: u8,
    pub(crate) display_enabled: bool,
    pub(crate) step_index: u8,
    pub(crate) frames_until_next_step_index: u8,
}

#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn spinning_twins_falling_visual_init_loc_3b9b0(
    parent_x_pos: u32,
    parent_y_pos: u32,
) -> SpinningTwinsFallingVisualInitLoc3B9B0 {
    // ASM Spinning_Twins.asm loc_3B9B0: copy the parent coordinates, bias the
    // falling visual 19px left and 64px up, make it visible at priority 1, and
    // seed the movement loop with d0=0/d1=4.
    SpinningTwinsFallingVisualInitLoc3B9B0 {
        x_pos: parent_x_pos.wrapping_sub(0x13 << 16),
        y_pos: parent_y_pos.wrapping_sub(0x40 << 16),
        priority: 1,
        display_enabled: true,
        step_index: 0,
        frames_until_next_step_index: 4,
    }
}

pub(crate) const SPINNING_TWINS_FALLING_VISUAL_X_STEPS_LOC_3BA00: [i32; 5] =
    [0x1_2000, 0x1_2000, 0x1_8000, 0x2_4000, 0x3_0000];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) struct SpinningTwinsFallingVisualStateLoc3BA00 {
    pub(crate) x_pos: u32,
    pub(crate) y_pos: u32,
    pub(crate) step_index: u8,
    pub(crate) frames_until_next_step_index: u8,
    pub(crate) reverse_x_motion: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) struct SpinningTwinsFallingVisualStepLoc3BA00 {
    pub(crate) state: SpinningTwinsFallingVisualStateLoc3BA00,
    pub(crate) alive: bool,
}

fn spinning_twins_falling_visual_screen_delta_px(position: u32, camera_position: u32) -> i16 {
    let position_px = (position >> 16) as u16;
    let camera_px = (camera_position >> 16) as u16;
    position_px.wrapping_sub(camera_px) as i16
}

#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn spinning_twins_falling_visual_step_loc_3ba00(
    mut state: SpinningTwinsFallingVisualStateLoc3BA00,
    camera_x_pos: u32,
    camera_y_pos: u32,
) -> SpinningTwinsFallingVisualStepLoc3BA00 {
    // ASM Spinning_Twins.asm loc_3BA00: delete when the visual leaves the
    // signed screen-relative box X[-$104,$244], Y[-$104,$1E4]; otherwise use
    // dword_3BA60 for X drift, move down one pixel, and advance d0 every four
    // frames, toggling d3 after the fifth table entry.
    let screen_x = i32::from(spinning_twins_falling_visual_screen_delta_px(
        state.x_pos,
        camera_x_pos,
    ));
    let screen_y = i32::from(spinning_twins_falling_visual_screen_delta_px(
        state.y_pos,
        camera_y_pos,
    ));
    if !(-0x104..=0x244).contains(&screen_x) || !(-0x104..=0x1E4).contains(&screen_y) {
        // ASM Spinning_Twins.asm loc_3BA5C: off-camera visuals immediately
        // jump to Delete_CurrentObject before movement is applied.
        return SpinningTwinsFallingVisualStepLoc3BA00 {
            state,
            alive: false,
        };
    }

    // ASM Spinning_Twins.asm loc_3BA3C: apply the selected signed X drift,
    // drop the visual one pixel, then maintain the four-frame table cadence.
    let step = SPINNING_TWINS_FALLING_VISUAL_X_STEPS_LOC_3BA00[usize::from(state.step_index) % 5];
    let x_delta = if state.reverse_x_motion { -step } else { step };
    state.x_pos = (i64::from(state.x_pos) + i64::from(x_delta)) as u32;
    state.y_pos = state.y_pos.wrapping_add(1 << 16);

    state.frames_until_next_step_index = state.frames_until_next_step_index.saturating_sub(1);
    if state.frames_until_next_step_index == 0 {
        state.frames_until_next_step_index = 4;
        state.step_index = state.step_index.wrapping_add(1);
        if state.step_index == 5 {
            state.reverse_x_motion = !state.reverse_x_motion;
            state.step_index = 0;
        }
    }

    SpinningTwinsFallingVisualStepLoc3BA00 { state, alive: true }
}

/// SpinningTwins: orbit at spawn position, then drop and walk as pacer.
/// Orbit-to-walk transition is collision-triggered (player touching the orbiting form).
/// ASM: loc_3B904 checks collision_type each frame.
pub(crate) fn step_spinning_twins(
    enemy: &mut ActiveEnemy,
    block_grid: Option<&terrain::BlockGrid>,
    player_x_pos: Option<u32>,
    player_y_pos: Option<u32>,
) {
    match enemy.spinning_twins_phase {
        SpinningTwinsPhase::Orbiting => {
            // Advance orbit index each frame
            enemy.spinning_twins_orbit_index =
                (enemy.spinning_twins_orbit_index + 1) % SPINNING_TWINS_ORBIT_OFFSETS.len() as u8;
            let offset = SPINNING_TWINS_ORBIT_OFFSETS[enemy.spinning_twins_orbit_index as usize];
            // Apply X oscillation as velocity (visual only, resets each frame)
            enemy.x_velocity = offset << 16;
            enemy.x_pos = (enemy.x_pos as i64 + i64::from(enemy.x_velocity)) as u32;
            enemy.x_velocity = 0;

            // Check player collision — triggers orbit-to-walk transition.
            // ASM loc_3B904: collision_type negative/$2C/$1C all trigger drop.
            let player_contact = if let (Some(px), Some(py)) = (player_x_pos, player_y_pos) {
                let ex = (enemy.x_pos >> 16) as i32;
                let ey = (enemy.y_pos >> 16) as i32;
                let px_i = (px >> 16) as i32;
                let py_i = (py >> 16) as i32;
                (ex - px_i).abs() < SPINNING_TWINS_ORBIT_HITBOX_HALF
                    && (ey - py_i).abs() < SPINNING_TWINS_ORBIT_HITBOX_HALF + 8
            } else {
                false
            };

            if player_contact {
                let _transition = spinning_twins_drop_transition_loc_3b93c(enemy);
            }
        }
        SpinningTwinsPhase::Walking => {
            // Apply gravity while airborne until grounded, then walk as pacer
            if enemy.on_ground {
                step_pacer(enemy, block_grid);
            } else {
                enemy.y_velocity += SPINNING_TWINS_GRAVITY;
                enemy.y_pos = (enemy.y_pos as i64 + i64::from(enemy.y_velocity)) as u32;
                // Check floor
                if let Some(grid) = block_grid {
                    let x_px = (enemy.x_pos >> 16) as i32;
                    let y_px = (enemy.y_pos >> 16) as i32;
                    let probe = terrain::probe_floor(grid, x_px, y_px);
                    if probe.on_ground {
                        enemy.y_pos = (probe.floor_y_px as u32) << 16;
                        enemy.y_velocity = 0;
                        enemy.on_ground = true;
                    }
                }
            }
        }
    }
}

/// EmoRock hover height above player, indexed by enemy_level (derived from HP).
/// Level 0: 80px, Level 1: 60px, Level 2: 40px.
pub(crate) const EMO_ROCK_HOVER_HEIGHT: [i32; 3] = [0x50, 0x3C, 0x28];

/// EmoRock gravity acceleration (applied each frame while dropping).
/// Level 0: $4000, Level 1: $6000, Level 2: $8000.
pub(crate) const EMO_ROCK_GRAVITY: [i32; 3] = [0x4000, 0x6000, 0x8000];

/// EmoRock terminal velocity threshold/cap while dropping.
/// ASM Emo_Rock.asm:72-74: if y_vel > $60000, reset to $40000.
pub(crate) const EMO_ROCK_VELOCITY_THRESHOLD: i32 = 0x6_0000;
pub(crate) const EMO_ROCK_VELOCITY_CAP: i32 = 0x4_0000;

/// EmoRock walk speed after landing.
pub(crate) const EMO_ROCK_WALK_SPEED: i32 = 0x8000;

/// Step an EmoRock enemy through its state machine:
/// - wait_timer > 0: hover above player, count down
/// - wait_timer == 0 && !on_ground: drop with gravity
/// - on_ground: walk like a pacer
pub(crate) fn step_emo_rock(
    enemy: &mut ActiveEnemy,
    block_grid: Option<&terrain::BlockGrid>,
    player_x_pos: Option<u32>,
    player_y_pos: Option<u32>,
) {
    if enemy.on_ground {
        // Walking phase: pacer-like ground movement facing player.
        if let Some(dir) = player_relative_direction(enemy.x_pos, player_x_pos) {
            enemy.facing = dir;
        }
        let Some(grid) = block_grid else { return };
        enemy.x_velocity = horizontal_velocity(enemy.facing, EMO_ROCK_WALK_SPEED);
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
            reverse_enemy_direction(enemy);
        }
        return;
    }

    if enemy.wait_timer > 0 {
        // Hovering phase: track player position, count down.
        let level_index = enemy.hit_points.saturating_sub(1).min(2) as usize;
        let hover_height_fp = (EMO_ROCK_HOVER_HEIGHT[level_index] as u32) << 16;
        if let Some(px) = player_x_pos {
            enemy.x_pos = px;
        }
        if let Some(py) = player_y_pos {
            enemy.y_pos = py.saturating_sub(hover_height_fp);
        }
        enemy.wait_timer -= 1;
        return;
    }

    // Dropping phase: apply gravity, check floor.
    let level_index = enemy.hit_points.saturating_sub(1).min(2) as usize;
    let gravity = EMO_ROCK_GRAVITY[level_index];
    let new_yv = enemy.y_velocity + gravity;
    // ASM Emo_Rock.asm:72-74: if exceeds $60000, snap to $40000
    enemy.y_velocity = if new_yv > EMO_ROCK_VELOCITY_THRESHOLD {
        EMO_ROCK_VELOCITY_CAP
    } else {
        new_yv
    };
    enemy.y_pos = (enemy.y_pos as i64 + i64::from(enemy.y_velocity)) as u32;

    if let Some(grid) = block_grid {
        let cx = (enemy.x_pos >> 16) as i32;
        let cy = (enemy.y_pos >> 16) as i32;
        let floor = terrain::probe_floor(grid, cx, cy);
        if floor.on_ground {
            // Land: snap to floor, enter walking state.
            enemy.y_pos = (floor.floor_y_px as u32) << 16;
            enemy.y_velocity = 0;
            enemy.on_ground = true;
            if let Some(dir) = player_relative_direction(enemy.x_pos, player_x_pos) {
                enemy.facing = dir;
            }
        }
    }
}

pub(crate) fn reverse_enemy_direction(enemy: &mut ActiveEnemy) {
    enemy.facing = opposite_direction(enemy.facing);
}

pub(crate) fn opposite_direction(facing: Direction) -> Direction {
    match facing {
        Direction::Left => Direction::Right,
        Direction::Right => Direction::Left,
    }
}

pub(crate) fn player_relative_direction(
    enemy_x_pos: u32,
    player_x_pos: Option<u32>,
) -> Option<Direction> {
    let player_x_pos = player_x_pos?;
    Some(if player_x_pos > enemy_x_pos {
        Direction::Right
    } else {
        Direction::Left
    })
}

/// Compute arrow angle index (0..27) from archer to player.
/// Faithfully mirrors Archer.asm `loc_34458`: quadrant dispatch + unk_34404 ratio lookup.
pub(crate) fn archer_angle_to_player(
    archer_x: u32,
    archer_y: u32,
    player_x: u32,
    player_y: u32,
) -> usize {
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
        return 0x15;
    }

    let abs_dx = dx.unsigned_abs();
    let abs_dy = dy.unsigned_abs();
    let ratio = (abs_dy * 8) / abs_dx;

    // ASM loc_34546: ratio >= $3F → nearly vertical, return 6 (up) or 21 (down).
    if ratio >= 0x3F {
        return if dy < 0 { 6 } else { 0x15 };
    }

    let sub = RATIO_TABLE[(ratio as usize).min(RATIO_TABLE.len() - 1)] as i32;

    match (dx > 0, dy >= 0) {
        // Quadrant 0: right+up (d5=0)
        (true, false) => sub as usize,
        // Quadrant 1: left+up (d5=1) → 0xD - sub
        (false, false) => (0x0D - sub) as usize,
        // Quadrant 2: left+down (d5=2) → 0xE + sub
        (false, true) => (0x0E + sub) as usize,
        // Quadrant 3: right+down (d5=3) → 0x1B - sub
        (true, true) => (0x1B - sub) as usize,
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
    let threshold: i16 = if target > current { 14 } else { -14 };
    if delta >= threshold {
        // Decrement path (ASM loc_34320)
        if current == 0 { 27 } else { current - 1 }
    } else {
        // Increment path (ASM loc_34308 fall-through)
        if current >= 27 { 0 } else { current + 1 }
    }
}

/// Convert an angle index (0..27) to (x_velocity, y_velocity) via ARROW_ANGLE_TABLE.
/// ASM Enemy07_ArcherArrow_AngleToSpeedAndAnim: byte * 0x1000.
pub(crate) fn arrow_velocity_from_angle(angle: usize) -> (i32, i32) {
    let (x_byte, y_byte) = ARROW_ANGLE_TABLE[angle.min(ARROW_ANGLE_TABLE.len() - 1)];
    ((x_byte as i32) * 0x1000, (y_byte as i32) * 0x1000)
}

/// Step Archer: stationary enemy that faces player and fires arrows at intervals.
/// ASM: arrows persist until terrain/object collision, re-aiming every `arrow_accuracy`
/// frames by ±1 angle step toward the player (loc_342D6 loop).
pub(crate) fn step_archer(
    enemy: &mut ActiveEnemy,
    projectiles: &mut Vec<EnemyProjectile>,
    player_x_pos: Option<u32>,
    player_y_pos: Option<u32>,
) {
    // Face towards player.
    if let Some(dir) = player_relative_direction(enemy.x_pos, player_x_pos) {
        enemy.facing = dir;
    }

    // Count down fire timer.
    if enemy.wait_timer > 0 {
        enemy.wait_timer -= 1;
        return;
    }

    // Reset timer.
    let level = enemy.hit_points.saturating_sub(1).min(2) as usize;
    enemy.wait_timer = ARCHER_FIRE_TIMER[level];

    // Need player position to compute angle.
    let (Some(px), Some(py)) = (player_x_pos, player_y_pos) else {
        return;
    };

    // ASM loc_34248: `cmpi.w #2,(Number_of_Arrows).w; bge.w loc_3408C`
    if projectiles.len() >= MAX_PROJECTILES {
        return;
    }

    // Compute initial angle and velocity.
    let angle = archer_angle_to_player(enemy.x_pos, enemy.y_pos, px, py);
    let (x_velocity, y_velocity) = arrow_velocity_from_angle(angle);
    let reaim = ARCHER_FIRE_TIMER[level];

    // Spawn offset from Archer.asm 3rd animation frame.
    // Vertical shooting (angle 6..=13): Y=-24. Horizontal: Y=-17.
    // X offset negated when facing right (ASM loc_34216).
    let y_offset = if (6..=13).contains(&angle) {
        ARCHER_ARROW_Y_OFFSET_VERTICAL
    } else {
        ARCHER_ARROW_Y_OFFSET_HORIZONTAL
    };
    let x_offset = if enemy.facing == Direction::Right {
        -ARCHER_ARROW_X_OFFSET
    } else {
        ARCHER_ARROW_X_OFFSET
    };
    let spawn_x = (enemy.x_pos as i64 + ((x_offset as i64) << 16)) as u32;
    let spawn_y = (enemy.y_pos as i64 + ((y_offset as i64) << 16)) as u32;

    projectiles.push(EnemyProjectile {
        x_pos: spawn_x,
        y_pos: spawn_y,
        x_velocity,
        y_velocity,
        // ASM arrows live until terrain collision, not a fixed timer.
        // Use large lifetime as fallback; terrain_kill handles normal removal.
        lifetime: u16::MAX,
        friendly: false,
        gravity: 0,
        homing_accel: 0,
        speed_cap: 0,
        wall_bounce: false,
        reaim_interval: reaim,
        reaim_countdown: reaim,
        current_angle: angle as u8,
        terrain_kill: true,
        owner: None,
    });
}

/// Scorpion tracking callback (Scorpion.asm loc_3DD7E).
///
/// Within 170px ($AA): face toward player (unless tracking-locked $4E),
/// boost speed to $38000 if enemy_level > 1 (HP > 3).
/// Outside 170px: reset speed to $18000, clear tracking lock if player airborne.
pub(crate) fn step_scorpion_tracking(
    enemy: &mut ActiveEnemy,
    player_x_pos: Option<u32>,
    _player_y_pos: Option<u32>,
) {
    let Some(px) = player_x_pos else { return };
    let dx = (enemy.x_pos as i64 - px as i64) >> 16; // scorpion_x - player_x

    // Outside tracking range: reset speed, optionally clear tracking lock.
    // ASM loc_3DE7E: $50(a5) = $18000; if player airborne, clear $4E.
    if dx.abs() > SCORPION_TRACK_RANGE {
        enemy.walk_speed_override = SCORPION_BASE_SPEED;
        // ASM checks player y_vel ($26(a4)) to clear tracking lock.
        // We approximate: clear lock when out of range (conservative).
        enemy.scorpion_tracking_lock = false;
        return;
    }

    // Within tracking range: face toward player unless tracking-locked.
    // ASM loc_3DD7E lines 146-155: if $4E clear, set x_direction toward player.
    if !enemy.scorpion_tracking_lock {
        if dx > 0 {
            enemy.facing = Direction::Left; // player is to the left
        } else if dx < 0 {
            enemy.facing = Direction::Right; // player is to the right
        }
    }

    // ASM line 162-164: if enemy_level > 1 (HP >= 5 since HP = level + 3), boost speed.
    if enemy.hit_points >= 5 {
        enemy.walk_speed_override = SCORPION_BOOST_SPEED;
    }
}

/// Crab freeze distance constants from ASM.
/// ASM loc_3B0E4: cmpi.w #$40,d7 (freeze threshold = 64px).
const CRAB_FREEZE_DISTANCE: i64 = 0x40;
/// ASM loc_3B142: cmpi.w #$E0,d7 (unfreeze threshold = 224px).
const CRAB_UNFREEZE_DISTANCE: i64 = 0xE0;

/// Crab player-proximity freeze callback (Crab.asm loc_3B0BE-loc_3B174).
///
/// When player is within 64px horizontally AND facing the crab:
/// - Freeze the crab (stop animation, set freeze sprite, zero velocity)
///
/// Unfreeze when:
/// - Distance > 224px, OR
/// - Within 64px but player is moving away from the crab
///
/// The freeze uses the player's facing direction and velocity to determine
/// whether to freeze or unfreeze.
pub(crate) fn step_crab_freeze(
    enemy: &mut ActiveEnemy,
    player_x_pos: Option<u32>,
    player_facing: Option<Direction>,
    player_x_velocity: Option<i32>,
) {
    let Some(px) = player_x_pos else { return };
    let Some(pfacing) = player_facing else { return };

    let dx = (enemy.x_pos as i64 - px as i64) >> 16; // crab_x - player_x
    let abs_dx = dx.abs();

    // ASM loc_3B142-loc_3B170: unfreeze when distance > 224px
    if abs_dx > CRAB_UNFREEZE_DISTANCE {
        enemy.crab_frozen = false;
        return;
    }

    // ASM loc_3B0E4-loc_3B11E: check freeze condition (within 64px)
    if abs_dx <= CRAB_FREEZE_DISTANCE {
        // ASM loc_3B0E4: player within 64px
        // Check if player is facing the crab:
        // - If player faces right (Direction::Right in native, $FF in ASM),
        //   crab must also face right for freeze
        // - If player faces left (Direction::Left in native, 0 in ASM),
        //   crab must also face left for freeze
        let player_faces_crab = match (pfacing, enemy.facing) {
            (Direction::Right, Direction::Right) => dx < 0, // player to left of crab, facing right toward it
            (Direction::Left, Direction::Left) => dx > 0, // player to right of crab, facing left toward it
            _ => false,
        };

        if player_faces_crab {
            // ASM loc_3B104: freeze the crab
            // sf is_animated(a3) - stop animation
            // bset #7,object_meta(a3) - set freeze bit
            // move.l #0,$50(a5) - zero x_velocity
            enemy.crab_frozen = true;
            enemy.x_velocity = 0;
            return;
        }

        // ASM loc_3B142-loc_3B16C: within 64px, check if player moving toward crab
        // If player moving away, unfreeze
        if let Some(p_vel) = player_x_velocity {
            // Player to left of crab (dx > 0), moving right (p_vel > 0): moving toward -> keep frozen
            // Player to left of crab (dx > 0), not moving right: moving away -> unfreeze
            // Player to right of crab (dx < 0), moving left (p_vel < 0): moving toward -> keep frozen
            // Player to right of crab (dx < 0), not moving left: moving away -> unfreeze
            let player_moving_toward_crab = if dx > 0 {
                p_vel > 0 // player to left, moving right toward crab
            } else {
                p_vel < 0 // player to right, moving left toward crab
            };
            if !player_moving_toward_crab {
                enemy.crab_frozen = false;
            }
        }
    }
    // ASM loc_3B11E: outside 64px but within 224px, keep current state
    // (already frozen stays frozen, already unfrozen stays unfrozen)
}

fn tank_shooting_cycle_frames(hit_points: u8) -> u16 {
    if hit_points >= 4 {
        TANK_SHOOTING_FAST_CYCLE_FRAMES
    } else {
        TANK_SHOOTING_SLOW_CYCLE_FRAMES
    }
}

/// Tank_Shooting.asm loc_3C50E only spawns `loc_3B730` when the turret child
/// is on `LnkTo_unk_C7F10` with `animation_timer == 5`.
fn tank_shooting_spawn_worker_now_sub_3c4f8() -> bool {
    shared_enemy_robot_helper_sub_3C4F8(SharedEnemyRobotHelperInputSub3C4F8 {
        helper_kind: SharedEnemyRobotHelperKindSub3C4F8::RockTankShooting,
        parent_mapping: ROCK_TANK_SHOOTING_MAPPING_SUB_3C4F8,
        parent_state_word_32: 5,
        child_state_word_40: 0,
        child_counter_word_4e: 0,
        child_timer_word_54: 0,
        parent_x_pos: 0,
        walker_x_pos: 0,
        kid_x_pos: 0,
        parent_x_direction: 0,
        parent_flag_15: false,
        random_word: 0,
    })
    .spawned_loc_3_b730_worker
}

/// ASM Tank_Shooting.asm loc_3C50E/loc_3B730: fire only when the turret
/// animation reaches `LnkTo_unk_C7F10` with `animation_timer == 5`.
pub(crate) fn step_tank_shooting_fire(
    enemy: &mut ActiveEnemy,
    projectiles: &mut Vec<EnemyProjectile>,
) {
    if enemy.wait_timer > 0 {
        enemy.wait_timer -= 1;
        return;
    }
    enemy.wait_timer = tank_shooting_cycle_frames(enemy.hit_points).saturating_sub(1);

    if projectiles.len() >= MAX_PROJECTILES {
        return;
    }
    if !tank_shooting_spawn_worker_now_sub_3c4f8() {
        return;
    }

    let x_velocity = horizontal_velocity(enemy.facing, TANK_BULLET_SPEED);
    let x_offset = horizontal_velocity(enemy.facing, TANK_BULLET_SPAWN_X_OFFSET_PX << 16);
    projectiles.push(EnemyProjectile {
        // ASM loc_3B75A/loc_3B786: start from the turret child and offset to
        // the muzzle instead of the body centre.
        x_pos: enemy.x_pos.wrapping_add_signed(x_offset),
        y_pos: enemy
            .y_pos
            .wrapping_sub((TANK_BULLET_SPAWN_Y_OFFSET_PX as u32) << 16),
        x_velocity,
        y_velocity: 0,
        lifetime: TANK_BULLET_LIFETIME,
        friendly: false,
        gravity: 0,
        homing_accel: 0,
        speed_cap: 0,
        wall_bounce: false,
        ..Default::default()
    });
}

/// Boss tracking movement: chase the player with fixed velocity components.
/// HeadyMetal ASM: x_vel ±$8000 towards player, y_vel $4800 down / -$B000 up,
/// targeting 40px above player. Other bosses use the same pattern at different speeds.
pub(crate) fn step_tracker(
    enemy: &mut ActiveEnemy,
    player_x_pos: Option<u32>,
    player_y_pos: Option<u32>,
) {
    let (Some(px), Some(py)) = (player_x_pos, player_y_pos) else {
        return;
    };

    let ex = enemy.x_pos as i64;
    let ey = enemy.y_pos as i64;
    let target_y = (py as i64).saturating_sub(0x28 << 16); // 40px above player

    // Horizontal: chase player at $8000 (0.5 px/frame)
    // HeadyMetal ASM: X velocity forced to 0 during first 4 hit phases ($3E < 4).
    // Phase = 37 - current_hp, so phase < 4 means hp > 33.
    let x_chase_speed: i32 =
        if matches!(enemy.family, EnemyFamily::HeadyMetal) && enemy.hit_points > 33 {
            0 // Early phases: vertical-only tracking
        } else {
            0x8000
        };
    if (px as i64) > ex {
        enemy.x_velocity = x_chase_speed;
        enemy.facing = Direction::Right;
    } else {
        enemy.x_velocity = -x_chase_speed;
        enemy.facing = Direction::Left;
    }

    // Vertical: faster upward (-$B000) than downward ($4800)
    if target_y < ey {
        enemy.y_velocity = -0xB000;
    } else {
        enemy.y_velocity = 0x4800;
    }

    // Apply velocities
    enemy.x_pos = (ex + i64::from(enemy.x_velocity)).clamp(0, i64::from(u32::MAX)) as u32;
    enemy.y_pos = (ey + i64::from(enemy.y_velocity)).clamp(0, i64::from(u32::MAX)) as u32;
}

/// Boss periodic projectile fire interval (frames).
pub(crate) const BOSS_FIRE_INTERVAL: u16 = 180; // ~3 seconds
/// Boss skewer speed (Shishkaboss downward projectiles).
pub(crate) const BOSS_SKEWER_SPEED: i32 = 0x2_0000;

/// Bosses periodically fire projectiles at the player.
pub(crate) fn step_boss_fire(
    enemy: &mut ActiveEnemy,
    projectiles: &mut Vec<EnemyProjectile>,
    player_x_pos: Option<u32>,
    player_y_pos: Option<u32>,
) {
    if enemy.wait_timer > 0 {
        enemy.wait_timer -= 1;
        return;
    }
    // HeadyMetal ASM: fire rate is animation-driven, faster in later phases.
    // Phase = 37 - hp. Phases < 6 (hp > 31): slower animation → ~120 frame interval.
    // Phases >= 6 (hp <= 31): rapid-fire animation → ~30 frame interval.
    let interval = if matches!(enemy.family, EnemyFamily::HeadyMetal) {
        if enemy.hit_points > 31 { 120 } else { 30 }
    } else {
        BOSS_FIRE_INTERVAL
    };
    enemy.wait_timer = interval;

    if projectiles.len() >= MAX_PROJECTILES {
        return;
    }

    match enemy.family {
        EnemyFamily::HeadyMetal => {
            // Spawn homing eye projectile with gravity
            projectiles.push(EnemyProjectile {
                x_pos: enemy.x_pos,
                y_pos: enemy.y_pos,
                x_velocity: 0,
                y_velocity: 0,
                lifetime: 120,
                gravity: 0x2710,
                friendly: false,
                homing_accel: 0x7D0, // HeadyMetal ASM
                speed_cap: 0,
                wall_bounce: false,
                ..Default::default()
            });
        }
        EnemyFamily::Shishkaboss => {
            // Downward skewer from above
            projectiles.push(EnemyProjectile {
                x_pos: enemy.x_pos,
                y_pos: enemy.y_pos,
                x_velocity: 0,
                y_velocity: BOSS_SKEWER_SPEED,
                lifetime: 90,
                gravity: 0,
                friendly: false,
                homing_accel: 0,
                speed_cap: 0,
                wall_bounce: false,
                ..Default::default()
            });
        }
        EnemyFamily::BoomerangBosses => {
            // Boomerang arc: fast horizontal, slight upward
            let dir = if let Some(px) = player_x_pos {
                if px >= enemy.x_pos { 1i32 } else { -1 }
            } else {
                match enemy.facing {
                    Direction::Right => 1,
                    Direction::Left => -1,
                }
            };
            projectiles.push(EnemyProjectile {
                x_pos: enemy.x_pos,
                y_pos: enemy.y_pos,
                x_velocity: dir * 0x3_0000,
                y_velocity: -0x1_0000,
                lifetime: 60,
                gravity: 0x1000,
                friendly: false,
                homing_accel: 0,
                speed_cap: 0,
                wall_bounce: false,
                ..Default::default()
            });
        }
        EnemyFamily::BagelBrothers => {
            // Fast horizontal dash projectile
            let (Some(px), Some(_py)) = (player_x_pos, player_y_pos) else {
                return;
            };
            let dir = if px >= enemy.x_pos { 1i32 } else { -1 };
            projectiles.push(EnemyProjectile {
                x_pos: enemy.x_pos,
                y_pos: enemy.y_pos,
                x_velocity: dir * 0x3_0000, // ASM: 3 px/frame (word)
                y_velocity: 0,
                lifetime: 50, // ASM: 50 frames moving phase
                gravity: 0,
                friendly: false,
                homing_accel: 0,
                speed_cap: 0,
                wall_bounce: false,
                ..Default::default()
            });
        }
        _ => {}
    }
}

pub(crate) fn spawn_x_pos(family: EnemyFamily, object_x: u32) -> u32 {
    match family {
        EnemyFamily::Dragon => (object_x << 16).saturating_add(DRAGON_SPAWN_X_OFFSET_FP),
        _ => object_x << 16,
    }
}

#[allow(dead_code)]
pub(crate) fn spawn_y_pos(family: EnemyFamily, object_y: u32) -> u32 {
    spawn_y_pos_sub_36FF4(family, object_y, 0)
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn enemy_spawn_y_adjustment_sub_36FF4(blank_top_rows: u16) -> u32 {
    // ASM sub_36FF4 (kid.asm:38882): load `Number_blank_top_rows`, shift left
    // by 4, and add that pixel offset to `y_pos(a3)`.
    u32::from(blank_top_rows) << 4
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn spawn_y_pos_sub_36FF4(
    family: EnemyFamily,
    object_y: u32,
    blank_top_rows: u16,
) -> u32 {
    let object_y = object_y.saturating_add(enemy_spawn_y_adjustment_sub_36FF4(blank_top_rows));
    match family {
        EnemyFamily::Dragon => object_y.saturating_add(DRAGON_SPAWN_Y_OFFSET_PX) << 16,
        EnemyFamily::DragonWalking => {
            object_y.saturating_add(DRAGON_WALKING_SPAWN_Y_OFFSET_PX) << 16
        }
        _ => object_y << 16,
    }
}

pub(crate) fn spawn_x_velocity(family: EnemyFamily, enemy_level: u8) -> i32 {
    match family {
        EnemyFamily::Cloud => {
            // Cloud.asm dword_35B38: speeds indexed by enemy_level
            const CLOUD_SPEEDS: [i32; 3] = [-0x3000, -0x5000, -0x7000];
            CLOUD_SPEEDS[usize::from(enemy_level).min(CLOUD_SPEEDS.len() - 1)]
        }
        // Sphere.asm: fixed x_vel = $10000
        EnemyFamily::Sphere => 0x10000,
        // Fireball.asm: x_vel = max_upwards >> 1, indexed by enemy_level
        EnemyFamily::Fireball => {
            const FIREBALL_MAX_UPWARD: [i32; 3] = [-0x30000, -0x40000, -0x50000];
            FIREBALL_MAX_UPWARD[usize::from(enemy_level).min(2)] >> 1
        }
        // Big_Hopping_Skull.asm: move.w #1,x_vel → $10000 FP16.16
        EnemyFamily::BigHoppingSkull => BIG_SKULL_INIT_X_SPEED,
        // ASM Hand.asm loc_327F2 stores the direction-gated table velocity
        // into x_vel before the first patrol hibernate.
        EnemyFamily::Hand => {
            let hit_points = spawn_hit_points(EnemyFamily::Hand, enemy_level);
            hand_initial_motion_loc_327f2(hit_points, false).x_velocity
        }
        _ => 0,
    }
}

pub(crate) fn spawn_facing(family: EnemyFamily, enemy_x: u32, player_x: u32) -> Direction {
    match family {
        // ASM Hand.asm loc_327B6 clears x_direction; loc_3284C's clear-flag
        // branch detects Kid only on the left side.
        EnemyFamily::Hand => Direction::Left,
        // Drips always start facing right (Drip.asm init)
        EnemyFamily::Drips => Direction::Right,
        // ASM Armadillo.asm loc_3C718, Dragon.asm loc_3CD70, Tank.asm loc_3B2E4,
        // Robot.asm loc_3A7AC, and Scorpion.asm loc_3DBD8: face TOWARD the player at spawn.
        // `move.w x_pos(a4),d4; cmp.w x_pos(a3),d4; blt.s loc_3CD82; st x_direction(a3)`
        // x_direction=0 → LEFT (default VRAM orientation), x_direction=$FF → RIGHT (flipped).
        // kid.x >= dragon.x → st x_direction → $FF → faces RIGHT (toward kid on the right).
        // kid.x < dragon.x → skip → x_direction stays 0 → faces LEFT (toward kid on the left).
        _ => {
            if player_x < enemy_x {
                Direction::Left
            } else {
                Direction::Right
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn mini_hopping_skull_test_enemy() -> ActiveEnemy {
        ActiveEnemy {
            segment_index: 0,
            layout_index: 0,
            enemy_level: 0,
            family: EnemyFamily::MiniHoppingSkull,
            x_pos: 32 << 16,
            y_pos: 32 << 16,
            x_velocity: 0,
            y_velocity: 0,
            target_x_velocity: 0,
            target_y_velocity: 0,
            x_acceleration: 0,
            y_acceleration: 0,
            facing: Direction::Right,
            on_ground: true,
            hit_points: 1,
            grounded_frames: 0,
            damage_flash_frames: 0,
            palette_variant: 0,
            wait_timer: 0,
            dragon_phase: DragonFlyingState::None,
            dragon_phase_frames: 0,
            dragon_script_variant: 0,
            dragon_script_cmd: 0,
            dragon_anchor_y: 0,
            dragon_descent_index: 0,
            dragon_descent_frames: 0,
            ninja_state: NinjaState::Stealth,
            hand_state: HandState::Walking,
            lion_state: LionState::Walking,
            robot_state: RobotState::Walking,
            orca_state: OrcaState::Walking,
            wall_pinned: false,
            tar_monster_state: TarMonsterState::default(),
            spinning_twins_phase: SpinningTwinsPhase::default(),
            spinning_twins_orbit_index: 0,
            ufo_beam_segments: 0,
            ufo_beam_interval: 0,
            boss_head_index: BOSS_HEAD_NONE,
            scorpion_tracking_lock: false,
            flight_double_acceleration: false,
            flight_double_terminal_speed: false,
            walk_speed_override: 0,
            crab_frozen: false,
            shared_walker_state: 0,
            shared_walker_patrol_timer: 0,
            shared_walker_patrol_short_next: false,
            shared_walker_debug_branch: None,
            shared_walker_dx_to_kid_px: None,
            shared_walker_terrain_trace: SharedWalkerTerrainTrace::default(),
        }
    }

    fn mini_hopping_skull_test_grid() -> terrain::BlockGrid {
        let data = vec![0u8; 8 * 4 * 2];
        let mut grid = terrain::BlockGrid::from_level_layout(&data, 8, 4);
        for col in 0..8 {
            grid.set_cell_at(col, 2, terrain::Cell::from_be_bytes(0x60, 0x00));
        }
        grid
    }

    fn tar_monster_test_enemy() -> ActiveEnemy {
        ActiveEnemy {
            family: EnemyFamily::TarMonster,
            hit_points: 2,
            facing: Direction::Right,
            tar_monster_state: TarMonsterState::Attacking,
            wait_timer: TAR_MONSTER_ATTACK_FRAMES,
            ..mini_hopping_skull_test_enemy()
        }
    }

    fn fireball_test_enemy() -> ActiveEnemy {
        ActiveEnemy {
            family: EnemyFamily::Fireball,
            hit_points: 1,
            facing: Direction::Right,
            ..mini_hopping_skull_test_enemy()
        }
    }

    fn sphere_test_enemy() -> ActiveEnemy {
        ActiveEnemy {
            family: EnemyFamily::Sphere,
            hit_points: 1,
            facing: Direction::Right,
            x_velocity: 0x0001_0000,
            ..mini_hopping_skull_test_enemy()
        }
    }

    fn ninja_test_enemy() -> ActiveEnemy {
        ActiveEnemy {
            family: EnemyFamily::Ninja,
            hit_points: 2,
            facing: Direction::Left,
            ..mini_hopping_skull_test_enemy()
        }
    }

    #[test]
    fn diamond_hp_and_walk_speed_follow_loc_3ad16_and_loc_3ad4c() {
        // ASM Diamond.asm loc_3AD16 copies `enemy_level(a3)` to d7, adds 2 HP,
        // then loc_3AD4C applies the <=3 HP walk-speed tiers.
        let hp2 = spawn_hit_points(EnemyFamily::Diamond, 0);
        let hp3 = spawn_hit_points(EnemyFamily::Diamond, 1);

        assert_eq!(hp2, 2);
        assert_eq!(hp3, 3);

        assert_eq!(pacer_walk_speed(EnemyFamily::Diamond, hp2), 0x0001_0000);
        assert_eq!(pacer_walk_speed(EnemyFamily::Diamond, hp3), 0x0001_8000);
    }

    #[test]
    fn tank_spawn_faces_toward_player_per_loc_3b2e4() {
        assert_eq!(
            spawn_facing(EnemyFamily::Tank, 100 << 16, 80 << 16),
            Direction::Left
        );
        assert_eq!(
            spawn_facing(EnemyFamily::Tank, 100 << 16, 100 << 16),
            Direction::Right
        );
        assert_eq!(
            spawn_facing(EnemyFamily::Tank, 100 << 16, 120 << 16),
            Direction::Right
        );
    }

    #[test]
    fn tank_hp_and_walk_speed_follow_loc_3b36a_and_loc_3b372() {
        // ASM Tank.asm loc_3B36A/loc_3B372: enemy_level is copied to d7, then
        // addq.w #2,d7 produces HP 2/3/4. The walk-speed longword receives one,
        // two, or three `$8000` increments at those tiers.
        let hp2 = spawn_hit_points(EnemyFamily::Tank, 0);
        let hp3 = spawn_hit_points(EnemyFamily::Tank, 1);
        let hp4 = spawn_hit_points(EnemyFamily::Tank, 2);

        assert_eq!(hp2, 2);
        assert_eq!(hp3, 3);
        assert_eq!(hp4, 4);

        assert_eq!(pacer_walk_speed(EnemyFamily::Tank, hp2), 0x0000_8000);
        assert_eq!(pacer_walk_speed(EnemyFamily::Tank, hp3), 0x0001_0000);
        assert_eq!(pacer_walk_speed(EnemyFamily::Tank, hp4), 0x0001_8000);
        assert_eq!(
            pacer_walk_speed(EnemyFamily::TankShooting, hp2),
            0x0000_8000
        );
        assert_eq!(
            pacer_walk_speed(EnemyFamily::TankShooting, hp3),
            0x0001_0000
        );
        assert_eq!(
            pacer_walk_speed(EnemyFamily::TankShooting, hp4),
            0x0001_8000
        );
    }

    #[test]
    fn crab_init_visible_state_matches_loc_3afd0() {
        assert_eq!(
            crab_init_visible_state_loc_3afd0(0),
            CrabInitVisibleStateLoc3AFD0 {
                status_flag_13: true,
                is_animated: true,
                object_meta: 0x0009,
                priority: 0,
                current_hp: 2,
                speed_preload: 0,
            },
            "ASM Crab.asm loc_3AFD0 should only preload speed for HP 3+"
        );

        assert_eq!(
            crab_init_visible_state_loc_3afd0(1),
            CrabInitVisibleStateLoc3AFD0 {
                status_flag_13: true,
                is_animated: true,
                object_meta: 0x0009,
                priority: 0,
                current_hp: 3,
                speed_preload: 0x0001_0000,
            },
            "ASM Crab.asm loc_3AFD0 writes `$10000` to both speed slots once `current_hp >= 3`"
        );
    }

    #[test]
    fn crab_motion_profile_matches_loc_3b00a() {
        assert_eq!(
            crab_motion_profile_loc_3b00a(2),
            CrabMotionProfileLoc3B00A {
                walk_speed: 0x0000_8000,
                stored_speed: 0x0000_8000,
                timer_4a: 0x001E,
                timer_48: 0x000C,
                animation_script: "stru_3AF62",
                increments_enemy_count: true,
            },
            "ASM Crab.asm loc_3B00A should give HP 2 only the final `$8000` speed add"
        );

        assert_eq!(
            crab_motion_profile_loc_3b00a(3).walk_speed,
            0x0001_8000,
            "ASM Crab.asm loc_3B00A should add `$8000` after the HP 3 preload"
        );
        assert_eq!(
            crab_motion_profile_loc_3b00a(4).walk_speed,
            0x0001_8000,
            "ASM Crab.asm loc_3B00A has no extra HP 4 speed tier"
        );
        assert_eq!(pacer_walk_speed(EnemyFamily::Crab, 4), 0x0001_8000);
    }

    #[test]
    fn crab_collision_dispatch_matches_loc_3b048() {
        assert_eq!(
            crab_collision_dispatch_loc_3b048(0, false, 0x0020, Direction::Left),
            CrabCollisionRouteLoc3B048::ContinueSharedWalker,
            "ASM Crab.asm loc_3B048 should ignore zero collision_type before any side checks"
        );
        assert_eq!(
            crab_collision_dispatch_loc_3b048(-1, false, 0x0020, Direction::Left),
            CrabCollisionRouteLoc3B048::Death,
            "ASM Crab.asm loc_3B048 should send negative collision_type to the death route"
        );
        assert_eq!(
            crab_collision_dispatch_loc_3b048(CRAB_COLID_KIDRIGHT, false, 0x0020, Direction::Left,),
            CrabCollisionRouteLoc3B048::BerzerkerSideGate,
            "ASM Crab.asm loc_3B048 should route right-side Kid contact through the Berzerker gate"
        );
        assert_eq!(
            crab_collision_dispatch_loc_3b048(CRAB_COLID_KIDLEFT, false, -0x0020, Direction::Right),
            CrabCollisionRouteLoc3B048::BerzerkerSideGate,
            "ASM Crab.asm loc_3B048 should route left-side Kid contact through the Berzerker gate"
        );
        assert_eq!(
            crab_collision_dispatch_loc_3b048(CRAB_COLID_HURT, true, 0x0020, Direction::Left),
            CrabCollisionRouteLoc3B048::ContinueSharedWalker,
            "ASM Crab.asm loc_3B048 should suppress hurt response while the frozen sprite is active"
        );
    }

    #[test]
    fn crab_hurt_side_gate_matches_loc_3b048() {
        assert_eq!(
            crab_collision_dispatch_loc_3b048(CRAB_COLID_HURT, false, 0x0020, Direction::Left),
            CrabCollisionRouteLoc3B048::Damage,
            "ASM Crab.asm loc_3B048 should damage a left-facing crab when Kid is to its left"
        );
        assert_eq!(
            crab_collision_dispatch_loc_3b048(CRAB_COLID_HURT, false, 0x0020, Direction::Right),
            CrabCollisionRouteLoc3B048::ContinueSharedWalker,
            "ASM Crab.asm loc_3B048 should skip hurt when Kid is not on the crab-facing side"
        );
        assert_eq!(
            crab_collision_dispatch_loc_3b048(CRAB_COLID_HURT, false, -0x0020, Direction::Right),
            CrabCollisionRouteLoc3B048::Damage,
            "ASM Crab.asm loc_3B048 should damage a right-facing crab when Kid is to its right"
        );
        assert_eq!(
            crab_collision_dispatch_loc_3b048(CRAB_COLID_HURT, false, 0, Direction::Left),
            CrabCollisionRouteLoc3B048::Damage,
            "ASM Crab.asm loc_3B048 should treat exact X alignment like the non-negative side"
        );
    }

    #[test]
    fn ninja_init_state_matches_loc_3d554() {
        // ASM Enemy11_Ninja_Init `loc_3D554` sets object_meta and current_hp from
        // layout level; loc_3D59A/3D5AA then shape the movement speed tier.
        let low_hp = ninja_init_state_loc_3d554(0);
        assert_eq!(low_hp.current_hp, 2);
        assert!(low_hp.status_flag_13);
        assert_eq!(low_hp.object_meta, 0x0011);
        assert_eq!(low_hp.priority, 0);
        assert_eq!(low_hp.walk_speed_override, 0x0001_8000);

        let mid_hp = ninja_init_state_loc_3d554(1);
        assert_eq!(mid_hp.current_hp, 3);
        assert_eq!(mid_hp.walk_speed_override, 0x0001_8000);
    }

    #[test]
    fn ninja_init_speed_follows_loc_3d59a_branch() {
        // ASM loc_3D59A path is taken when `current_hp > 3` after the init
        // compare against `#3`; this yields `$18000 + $8000`.
        assert_eq!(
            ninja_walk_speed_override_loc_3d59a_and_3d5aa(4),
            0x0002_4000
        );
        assert_eq!(
            ninja_walk_speed_override_loc_3d59a_and_3d5aa(5),
            0x0002_4000
        );
    }

    #[test]
    fn ninja_init_speed_follows_loc_3d5aa_fallthrough() {
        // ASM loc_3D5AA is the fallthrough path when `current_hp <= 3`
        // (either directly for `<3` or after loc_3D59A for `==3`).
        assert_eq!(
            ninja_walk_speed_override_loc_3d59a_and_3d5aa(2),
            0x0001_8000
        );
        assert_eq!(
            ninja_walk_speed_override_loc_3d59a_and_3d5aa(3),
            0x0001_8000
        );
    }

    #[test]
    fn tar_monster_hp_matches_loc_321c6_hitpoint_table() {
        // ASM Tar_Monster.asm `Hitpoints_Enemy_05` is indexed by `enemy_level`.
        assert_eq!(tar_monster_hit_points_loc_321c6(0), 2);
        assert_eq!(tar_monster_hit_points_loc_321c6(1), 4);
        assert_eq!(tar_monster_hit_points_loc_321c6(2), 6);
        assert_eq!(tar_monster_hit_points_loc_321c6(3), 0);
        assert_eq!(tar_monster_hit_points_loc_321c6(10), 0);
    }

    #[test]
    fn tar_monster_init_loc_321c6_copies_layout_xy_word_values_per_native() {
        // ASM loc_321C6 loads layout word-2/word-3 directly into x/y and reads
        // word-1 as the level index.
        assert_eq!(
            tar_monster_init_loc_321c6(0x0120, 0x0080, 1),
            TarMonsterInitLoc321C6 {
                hit_points: 4,
                x_pos: 0x0120 << 16,
                y_pos: 0x0080 << 16,
            }
        );

        assert_eq!(
            tar_monster_init_loc_321c6(0x003F, 0x0010, 2),
            TarMonsterInitLoc321C6 {
                hit_points: 6,
                x_pos: 0x003F << 16,
                y_pos: 0x0010 << 16,
            }
        );
    }

    #[test]
    fn tar_monster_init_loc_321e4_calls_36ff4_and_forwards_meta() {
        // ASM loc_321E4 applies the blank-row Y adjustment via sub_36FF4,
        // clears priority, and writes object_meta to 5.
        let init = tar_monster_init_loc_321e4(0x0120, 0x0080, 0, 2);
        assert_eq!(
            init,
            TarMonsterInitLoc321E4 {
                hit_points: 2,
                priority: 0,
                object_meta: 5,
                x_pos: 0x0120 << 16,
                y_pos: 0x0080_u32.saturating_add(2 * 16) << 16,
            }
        );
    }

    #[test]
    fn tar_monster_control_script_pointer_loc_32248_starts_at_unk_32264() {
        let bytes = tar_monster_loc_32248();

        assert_eq!(
            bytes,
            TAR_MONSTER_CONTROL_BYTES_LOC_32248.as_slice(),
            "ASM Tar_Monster.asm loc_32248 loads `unk_32264` into `a0`"
        );
        assert!(
            !bytes.is_empty(),
            "loc_32248 control-script source table must be populated"
        );
    }

    #[test]
    fn tar_monster_loc_3224e_clears_collision_type_and_picks_off_32290() {
        let init = tar_monster_loc_3224e();

        assert_eq!(
            init.collision_type, 0,
            "loc_3224E executes `clr.w collision_type(a3)`"
        );
        assert_eq!(init.jump_table[0], TarMonsterLoc32290Target::Loc322A8);
        assert_eq!(init.jump_table[1], TarMonsterLoc32290Target::Loc322BC);
        assert_eq!(init.jump_table[2], TarMonsterLoc32290Target::Loc322DA);
        assert_eq!(init.jump_table[3], TarMonsterLoc32290Target::Loc322FC);
        assert_eq!(init.jump_table[4], TarMonsterLoc32290Target::Loc32308);
        assert_eq!(init.jump_table[5], TarMonsterLoc32290Target::Loc32310);
    }

    #[test]
    fn tar_monster_loc_32256_restarts_on_negative_and_dispatches_by_low_byte() {
        let sentinel = tar_monster_loc_32256(-1);
        assert_eq!(
            sentinel,
            TarMonsterLoc32256Return {
                raw_command: -1,
                route: TarMonsterLoc32256Route::Restart,
            }
        );

        let command_322_a8 = i16::from_be_bytes([0x00, 0x00]);
        assert_eq!(
            tar_monster_loc_32256(command_322_a8).route,
            TarMonsterLoc32256Route::Dispatch(TarMonsterLoc32290Target::Loc322A8)
        );
        let command_323_10 = i16::from_be_bytes([0x00, 0x04]);
        assert_eq!(
            tar_monster_loc_32256(command_323_10).route,
            TarMonsterLoc32256Route::Dispatch(TarMonsterLoc32290Target::Loc32310)
        );
    }

    #[test]
    fn tar_monster_wait_loop_loc_322aa_uses_inclusive_dbf_count() {
        assert_eq!(
            tar_monster_wait_loop_loc_322aa(0),
            TarMonsterWaitLoopLoc322AA {
                hibernate_frames: 1,
                checks_object_range_each_frame: true,
                polls_collision_each_frame: true,
                next_collision_type: 0,
            },
            "ASM Tar_Monster.asm loc_322AA executes the loop body once before `dbf` exits from d0=0"
        );
        assert_eq!(
            tar_monster_wait_loop_loc_322aa(0x5A).hibernate_frames,
            91,
            "ASM Tar_Monster.asm loc_322AA makes script wait byte $5A last 91 one-frame hibernates"
        );
    }

    #[test]
    fn tar_monster_appear_sound_loc_322c0_preserves_script_command() {
        assert_eq!(
            tar_monster_appear_sound_loc_322c0(),
            TarMonsterAppearSoundLoc322C0 {
                sound_effect_label: "sfx_Tar_Monster_appears",
                preserves_script_command_d0: true,
            },
            "ASM Tar_Monster.asm loc_322C0 restores d0 after j_PlaySound so the script command survives the audio call"
        );
    }

    #[test]
    fn tar_monster_appear_animation_loc_322c8_waits_until_finished() {
        assert_eq!(
            tar_monster_appear_animation_loc_322c8(),
            TarMonsterAppearAnimationLoc322C8 {
                animation_label: "stru_3272E",
                hibernate_until_animation_finished: true,
                animation_frames: TAR_MONSTER_EMERGE_FRAMES,
                resumes_control_script: true,
            },
            "ASM Tar_Monster.asm loc_322C8 starts stru_3272E and waits for the animation to finish before reading the next command"
        );
    }

    #[test]
    fn tar_monster_third_attack_wait_loc_3235e_is_nine_collision_polled_frames() {
        assert_eq!(
            tar_monster_third_attack_wait_loc_3235e(),
            TarMonsterThirdAttackWaitLoc3235E {
                dbf_seed: 8,
                hibernate_frames: 9,
                checks_object_range_each_frame: true,
                polls_collision_each_frame: true,
                restored_sprite_link: "LnkTo_unk_C77D6",
                resumes_control_script: true,
            },
            "ASM Tar_Monster.asm loc_3235E seeds d0=8, so loc_32360 performs nine hibernated collision polls before restoring LnkTo_unk_C77D6"
        );
    }

    #[test]
    fn tar_monster_collision_gate_loc_3237a_routes_negative_zero_and_positive() {
        assert_eq!(
            tar_monster_collision_gate_loc_3237a(-1),
            TarMonsterCollisionGateLoc3237A::Kill,
            "ASM Tar_Monster.asm loc_3237A branches to the death path on a negative collision word"
        );
        assert_eq!(
            tar_monster_collision_gate_loc_3237a(0),
            TarMonsterCollisionGateLoc3237A::Return,
            "ASM Tar_Monster.asm loc_3237A returns immediately when collision_type(a3) is zero"
        );
        assert_eq!(
            tar_monster_collision_gate_loc_3237a(0x001C),
            TarMonsterCollisionGateLoc3237A::NonZero {
                captured_collision_type: 0x001C
            },
            "ASM Tar_Monster.asm loc_3237A preserves the non-zero collision id in d7 for the following dispatch"
        );
    }

    #[test]
    fn tar_monster_collision_dispatch_loc_32386_clears_and_routes_hit_ids() {
        assert_eq!(
            tar_monster_collision_dispatch_loc_32386(0x001C, false),
            TarMonsterCollisionDispatchLoc32386 {
                next_collision_type: 0,
                route: TarMonsterCollisionRouteLoc32386::Hurt,
                clears_kid_x_velocity: false,
            },
            "ASM Tar_Monster.asm loc_32386 clears collision_type(a3), then treats $1C as an ordinary hit"
        );
        assert_eq!(
            tar_monster_collision_dispatch_loc_32386(0x002C, false).route,
            TarMonsterCollisionRouteLoc32386::Hurt,
            "ASM Tar_Monster.asm loc_32386 accepts $2C through the same ordinary-hit branch"
        );
    }

    #[test]
    fn tar_monster_collision_dispatch_loc_32386_requires_berzerker_charge_for_side_ids() {
        assert_eq!(
            tar_monster_collision_dispatch_loc_32386(0x0020, false),
            TarMonsterCollisionDispatchLoc32386 {
                next_collision_type: 0,
                route: TarMonsterCollisionRouteLoc32386::ReturnOnly,
                clears_kid_x_velocity: false,
            },
            "ASM Tar_Monster.asm loc_32386 consumes $20 but ignores it when Berzerker_charging is clear"
        );
        assert_eq!(
            tar_monster_collision_dispatch_loc_32386(0x0024, true),
            TarMonsterCollisionDispatchLoc32386 {
                next_collision_type: 0,
                route: TarMonsterCollisionRouteLoc32386::BerzerkerKill,
                clears_kid_x_velocity: true,
            },
            "ASM Tar_Monster.asm loc_32386 accepts $24 only when Berzerker_charging is set and clears Kid x_vel on that path"
        );
        assert_eq!(
            tar_monster_collision_dispatch_loc_32386(0x0028, true),
            TarMonsterCollisionDispatchLoc32386 {
                next_collision_type: 0,
                route: TarMonsterCollisionRouteLoc32386::ReturnOnly,
                clears_kid_x_velocity: false,
            },
            "ASM Tar_Monster.asm loc_32386 clears even unrecognised non-zero collision ids before returning"
        );
    }

    #[test]
    fn tar_monster_berzerker_collision_loc_323aa_requires_charging_and_clears_kid_x_velocity() {
        assert_eq!(
            tar_monster_berzerker_collision_loc_323AA(false),
            TarMonsterBerzerkerCollisionLoc323AA {
                route: TarMonsterCollisionRouteLoc32386::ReturnOnly,
                clears_kid_x_velocity: false,
            },
            "ASM Tar_Monster.asm loc_323AA returns when Berzerker_charging is zero"
        );
        assert_eq!(
            tar_monster_berzerker_collision_loc_323AA(true),
            TarMonsterBerzerkerCollisionLoc323AA {
                route: TarMonsterCollisionRouteLoc32386::BerzerkerKill,
                clears_kid_x_velocity: true,
            },
            "ASM Tar_Monster.asm loc_323AA clears Kid `$26(a4)` x velocity before jumping to loc_323E6"
        );
    }

    #[test]
    fn tar_monster_hurt_collision_loc_323c0_hides_child_and_decrements_hitpoints() {
        assert_eq!(
            tar_monster_hurt_collision_loc_323C0(3, true),
            TarMonsterHurtCollisionLoc323C0 {
                saved_child_visible: true,
                child_visible_during_response: false,
                remaining_hitpoints: 2,
                route: TarMonsterHurtRouteLoc323C0::HurtAnimation,
            },
            "ASM Tar_Monster.asm loc_323C0 saves `$13(a2)`, clears it, subtracts one from d3, and continues the hurt response when d3 is still non-zero"
        );
    }

    #[test]
    fn tar_monster_hurt_collision_loc_323c0_enters_death_path_when_hp_reaches_zero() {
        assert_eq!(
            tar_monster_hurt_collision_loc_323C0(1, false),
            TarMonsterHurtCollisionLoc323C0 {
                saved_child_visible: false,
                child_visible_during_response: false,
                remaining_hitpoints: 0,
                route: TarMonsterHurtRouteLoc323C0::Death,
            },
            "ASM Tar_Monster.asm loc_323C0 branches to loc_323E6 when `subq.w #1,d3` leaves zero"
        );
    }

    #[test]
    fn tar_monster_hurt_animation_loc_323d6_restores_saved_child_visibility() {
        assert_eq!(
            tar_monster_hurt_animation_loc_323D6(true),
            TarMonsterHurtAnimationLoc323D6 {
                animation_label: "stru_326EE",
                hibernate_until_animation_finished: true,
                restored_child_visible: true,
                resumes_control_script: true,
            },
            "ASM Tar_Monster.asm loc_323D6 should wait for stru_326EE, restore d2 into `$13(a2)`, and return to loc_3224E"
        );
        assert!(
            !tar_monster_hurt_animation_loc_323D6(false).restored_child_visible,
            "ASM Tar_Monster.asm loc_323D6 restores the exact saved `$13(a2)` byte rather than forcing visibility on"
        );
    }

    #[test]
    fn tar_monster_death_status_loc_3242c_writes_respawn_marker_for_positive_slot() {
        assert_eq!(
            tar_monster_respawn_marker_loc_3242C(0x05),
            TarMonsterRespawnMarkerLoc3242C {
                index: 0x05,
                word_value: 0x2168,
            },
            "ASM Tar_Monster.asm loc_3242C doubles positive `$42(a5)` into an EnemyStatus_Table word offset and writes `$2168`"
        );
    }

    #[test]
    fn tar_monster_death_status_loc_3243a_always_deletes_after_status_route() {
        let positive = tar_monster_death_status_update_loc_3243A(0x02);
        assert!(
            positive.deletes_current_object,
            "ASM Tar_Monster.asm loc_3243A is the convergence jump to j_Delete_CurrentObject after loc_3242C"
        );

        let delete_only = tar_monster_death_status_update_loc_3243A(0x80);
        assert_eq!(
            delete_only.route,
            TarMonsterDeathStatusRouteLoc3243A::DeleteOnly,
            "ASM Tar_Monster.asm loc_3243A is also reached directly when `$42(a5)` has bit 7 set and bit 6 clear"
        );
        assert!(delete_only.deletes_current_object);

        assert_eq!(
            tar_monster_death_status_update_loc_3243A(0xC7).route,
            TarMonsterDeathStatusRouteLoc3243A::SubtractSpawnCounter {
                index: 0x07,
                decrement: 0x0400,
            },
            "ASM Tar_Monster.asm loc_3243A receives the bit-6 path after subtracting `$400` from EnemyStatus_Table[d0 & $3F]"
        );
    }

    #[test]
    fn tar_monster_post_submerge_hold_loc_32454_uses_inclusive_dbf_wait() {
        assert_eq!(
            tar_monster_post_submerge_hold_loc_32454(),
            TarMonsterPostSubmergeHoldLoc32454 {
                child_visible_during_animation: false,
                animation_label: "stru_3274C",
                hibernate_until_animation_finished: true,
                dbf_seed: 0x50,
                hibernate_frames: 0x51,
                checks_object_range_each_frame: true,
                polls_collision_each_frame: true,
                restarts_control_script: true,
            },
            "ASM Tar_Monster.asm loc_32454 starts from d0=$50, so the DBF loop hibernates and polls collision for 81 frames before branching to loc_32248"
        );
    }

    #[test]
    fn tar_monster_child_init_loc_32200_copies_position_and_anim_meta() {
        // ASM loc_32200 stores the parent coordinates onto the linked child object
        // and applies the animation defaults (`status_flag_13`, animation phase 2, vram=$100).
        let parent = tar_monster_init_loc_321e4(0x0100, 0x0040, 1, 1);
        let child = tar_monster_child_init_loc_32200(parent, 0x0F00);

        assert_eq!(
            child,
            TarMonsterChildInitLoc32200 {
                sprite_frame_link: 0x0F00,
                object_meta: 5,
                priority: 0,
                x_pos: 0x0100 << 16,
                y_pos: 0x0040_u32.saturating_add(16) << 16,
                status_flag_13: true,
                animation_phase: 2,
                vram_tile: 0x0100,
            }
        );
    }

    #[test]
    fn tank_shooting_layout_object_maps_enemy_id_0b() {
        let object = kid_data::EnemyLayoutObject {
            object_type: 0x0B,
            flags: 0,
            hit_points_plus_one: 1,
            x: 0x0120,
            y: 0x0088,
        };

        assert_eq!(
            EnemyFamily::from_layout_object(&[0x000A, 0x000B], &object),
            EnemyFamily::TankShooting,
            "ASM kid.asm enemyloaddata table maps object id `$0B` to `Enemy0B_RockTank_shooting_Init`"
        );
    }

    #[test]
    fn tank_shooting_fire_uses_loc_3c50e_cadence_and_loc_3b730_offsets() {
        let mut enemy = ActiveEnemy {
            family: EnemyFamily::TankShooting,
            hit_points: 4,
            facing: Direction::Right,
            x_pos: 100 << 16,
            y_pos: 80 << 16,
            wait_timer: 0,
            ..mini_hopping_skull_test_enemy()
        };
        let mut projectiles = Vec::new();

        step_tank_shooting_fire(&mut enemy, &mut projectiles);

        assert_eq!(projectiles.len(), 1);
        assert_eq!(
            projectiles[0].x_pos,
            (115_u32) << 16,
            "ASM Tank_Shooting.asm loc_3B75A adds `$000F` when facing right"
        );
        assert_eq!(
            projectiles[0].y_pos,
            (66_u32) << 16,
            "ASM Tank_Shooting.asm loc_3B786/loc_3B78E fires from turret_y-14"
        );
        assert_eq!(projectiles[0].x_velocity, 0x0003_0000);
        assert_eq!(projectiles[0].y_velocity, 0);
        assert_eq!(
            enemy.wait_timer,
            TANK_SHOOTING_FAST_CYCLE_FRAMES - 1,
            "ASM `stru_3B4D8` loops every 116 frames, so the countdown reload is 115 after a shot"
        );
    }

    #[test]
    fn tank_shooting_spawn_wait_timer_matches_first_loc_3c50e_fire_gate() {
        let mut rng = RomRandomState::default();

        assert_eq!(
            spawn_wait_timer(EnemyFamily::Tank, 0, 0, &mut rng),
            0,
            "Tank.asm never reaches the shooter helper"
        );
        assert_eq!(
            spawn_wait_timer(EnemyFamily::TankShooting, 0, 0, &mut rng),
            TANK_SHOOTING_FIRST_SHOT_DELAY_FRAMES - 1,
            "ASM GfxObject_Animate reaches `LnkTo_unk_C7F10`/timer 5 after 74 frames, so the countdown seed is 73"
        );
    }

    #[test]
    fn armadillo_spawn_faces_toward_player_per_loc_3c718() {
        assert_eq!(
            spawn_facing(EnemyFamily::Armadillo, 100 << 16, 80 << 16),
            Direction::Left
        );
        assert_eq!(
            spawn_facing(EnemyFamily::Armadillo, 100 << 16, 100 << 16),
            Direction::Right
        );
        assert_eq!(
            spawn_facing(EnemyFamily::Armadillo, 100 << 16, 120 << 16),
            Direction::Right
        );
    }

    #[test]
    fn scorpion_spawn_faces_toward_player_per_loc_3dbd8() {
        // ASM Scorpion.asm loc_3DBD8 is the common init tail reached after
        // `cmp.w x_pos(a3),d4; blt.s loc_3DBD8; st x_direction(a3)`.
        assert_eq!(
            spawn_facing(EnemyFamily::Scorpion, 100 << 16, 80 << 16),
            Direction::Left
        );
        assert_eq!(
            spawn_facing(EnemyFamily::Scorpion, 100 << 16, 100 << 16),
            Direction::Right
        );
        assert_eq!(
            spawn_facing(EnemyFamily::Scorpion, 100 << 16, 120 << 16),
            Direction::Right
        );
    }

    #[test]
    fn scorpion_tail_sync_clear_direction_matches_loc_3dc86() {
        assert_eq!(
            scorpion_tail_sync_loc_3dc86(0x0120_8000, 0x0040_4000, Direction::Left),
            ScorpionTailGfxPosition {
                x_word: 0x011F,
                y_word: 0x002A,
            },
            "ASM Scorpion.asm loc_3DC86 copies parent x/y words, subtracts $16 from y, and subtracts 1 from x when x_direction is clear"
        );
    }

    #[test]
    fn scorpion_tail_sync_set_direction_matches_loc_3dcaa() {
        assert_eq!(
            scorpion_tail_sync_loc_3dc86(0x0120_8000, 0x0040_4000, Direction::Right),
            ScorpionTailGfxPosition {
                x_word: 0x010A,
                y_word: 0x002A,
            },
            "ASM Scorpion.asm loc_3DCAA subtracts $16 from the linked tail x word when x_direction is set"
        );
    }

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
    fn dragon_negative_drift_acceleration_matches_loc_347b2() {
        assert_eq!(
            dragon_drift_negative_velocity_loc_347b2(-0x800),
            -0xA00,
            "ASM Dragon_Flying.asm loc_347B2 subtracts `$200` from a negative drift velocity"
        );
        assert_eq!(
            dragon_drift_negative_velocity_loc_347b2(-0x17F00),
            -0x18000,
            "ASM Dragon_Flying.asm loc_347B2 clamps when subtracting `$200` would pass `$FFFE8000`"
        );
        assert_eq!(
            dragon_drift_negative_velocity_loc_347b2(-0x18000),
            -0x18000,
            "ASM Dragon_Flying.asm loc_347B2 preserves the `$FFFE8000` negative speed cap"
        );
    }

    #[test]
    fn dragon_drift_tail_store_matches_loc_347c8() {
        assert_eq!(
            dragon_drift_store_loc_347c8(0x18000, 0),
            DragonDriftStoreLoc347C8 {
                x_velocity: 0x18000,
                checks_fire_breath_cone: false,
            },
            "ASM Dragon_Flying.asm loc_347C8 stores d7 into x_vel and skips loc_34836 when d1 is zero"
        );
        assert_eq!(
            dragon_drift_store_loc_347c8(-0x18000, 1),
            DragonDriftStoreLoc347C8 {
                x_velocity: -0x18000,
                checks_fire_breath_cone: true,
            },
            "ASM Dragon_Flying.asm loc_347C8 stores the signed d7 velocity before the non-zero d1 fire-cone branch"
        );
    }

    #[test]
    fn dragon_drift_dbf_tail_matches_loc_347fc() {
        assert_eq!(
            dragon_drift_dbf_tail_loc_347fc(0x0028),
            DragonDriftDbfTailLoc347FC {
                next_d0: 0x0027,
                loops_to_loc_3478c: true,
                advances_script: false,
            },
            "ASM Dragon_Flying.asm loc_347FC decrements d0 and branches back while the result is not `$FFFF`"
        );
        assert_eq!(
            dragon_drift_dbf_tail_loc_347fc(0x0000),
            DragonDriftDbfTailLoc347FC {
                next_d0: 0xFFFF,
                loops_to_loc_3478c: false,
                advances_script: true,
            },
            "ASM Dragon_Flying.asm loc_347FC falls through to loc_346DA only after dbf decrements zero to `$FFFF`"
        );
    }

    #[test]
    fn dragon_walking_child_seed_matches_loc_3cd16() {
        let init =
            dragon_walking_child_init_loc_3cd16(0x0123_8000, 0x0045_0000, Direction::Right, 0x0010);

        assert_eq!(
            init,
            DragonWalkingChildInitLoc3CD16 {
                x_pos: 0x0123_8000,
                y_pos: 0x0045_0000,
                facing: Direction::Right,
                inherited_state_word: 0x0010,
                parent_spawned_child: true,
            },
            "ASM Dragon.asm loc_3CD16 copies parent position, raw x_direction, and `$4A` into the spawned walking-dragon object before marking `$5C(a5)`"
        );
    }

    #[test]
    fn dragon_art_and_hp_seed_match_loc_3cdb8() {
        assert_eq!(
            dragon_art_init_loc_3cdb8(0x400D, 0xFFFF, 0),
            DragonArtInitLoc3CDB8 {
                art_slot_index: 0,
                status_flag_13: true,
                priority: 0,
                current_hp: 1,
            },
            "ASM Dragon.asm loc_3CDB8 uses d4=0 when header slot 0 masks to DragonWalking `$0D`"
        );
        assert_eq!(
            dragon_art_init_loc_3cdb8(0xFFFF, 0x800C, 2),
            DragonArtInitLoc3CDB8 {
                art_slot_index: 1,
                status_flag_13: true,
                priority: 0,
                current_hp: 3,
            },
            "ASM Dragon.asm loc_3CDB8 uses d4=1 when header slot 1 masks to Flying Dragon `$0C` and stores enemy_level+1 HP"
        );
        assert_eq!(
            dragon_art_init_loc_3cdb8(0x0001, 0x0002, 1).art_slot_index,
            2,
            "ASM Dragon.asm loc_3CDB8 falls through with d4=2 when neither loaded header slot is `$0C` or `$0D`"
        );
        assert_eq!(
            spawn_hit_points(EnemyFamily::DragonWalking, 2),
            3,
            "native DragonWalking HP should expose the same enemy_level+1 seed as ASM Dragon.asm loc_3CDB8"
        );
    }

    #[test]
    fn dragon_walking_speed_and_setup_match_loc_3cdfc() {
        assert_eq!(
            dragon_walking_setup_loc_3cdfc(1),
            DragonWalkingSetupLoc3CDFC {
                speed_slot_50: 0x0000_8000,
                target_speed_slot_5e: 0x0000_8000,
                state_word_4a: 0x0010,
                timer_48: 9,
                patrol_timer_pair_6e: 0x0032_012C,
                short_patrol_timer_54: DRAGON_WALKING_PATROL_SHORT_FRAMES,
                callback_role: "dragon_breath_callback",
                animation_label: "stru_3CC5E",
            },
            "ASM Dragon.asm loc_3CDFC adds `$8000` to both speed slots and uses the normal animation below HP 3"
        );
        assert_eq!(
            dragon_walking_setup_loc_3cdfc(3),
            DragonWalkingSetupLoc3CDFC {
                speed_slot_50: 0x0001_8000,
                target_speed_slot_5e: 0x0001_8000,
                state_word_4a: 0x0010,
                timer_48: 9,
                patrol_timer_pair_6e: 0x0032_012C,
                short_patrol_timer_54: DRAGON_WALKING_PATROL_SHORT_FRAMES,
                callback_role: "dragon_breath_callback",
                animation_label: "stru_3CC90",
            },
            "ASM Dragon.asm loc_3CDFC preserves the HP-3 `$10000` preload before the common `$8000` add"
        );
        assert_eq!(
            pacer_walk_speed(EnemyFamily::DragonWalking, 1),
            dragon_walking_setup_loc_3cdfc(1).speed_slot_50,
            "native DragonWalking HP 1 speed should match ASM Dragon.asm loc_3CDFC"
        );
        assert_eq!(
            pacer_walk_speed(EnemyFamily::DragonWalking, 3),
            dragon_walking_setup_loc_3cdfc(3).speed_slot_50,
            "native DragonWalking HP 3 speed should match ASM Dragon.asm loc_3CDFC"
        );
    }

    #[test]
    fn dragon_walking_normal_animation_matches_loc_3ce48() {
        assert_eq!(
            dragon_walking_normal_animation_loc_3ce48(),
            DragonWalkingNormalAnimationLoc3CE48 {
                animation_label: "stru_3CC5E",
                saved_animation_label: "stru_3CC5E",
            },
            "ASM Dragon.asm loc_3CE48 initialises `stru_3CC5E` and saves it in `$62(a5)`"
        );
        assert_eq!(
            dragon_walking_setup_loc_3cdfc(1).animation_label,
            dragon_walking_normal_animation_loc_3ce48().animation_label,
            "native non-HP3 DragonWalking setup should use the loc_3CE48 normal animation branch"
        );
    }

    #[test]
    fn dragon_walking_common_init_matches_loc_3ce5a() {
        assert_eq!(
            dragon_walking_common_init_loc_3ce5a(false, Direction::Left),
            DragonWalkingCommonInitLoc3CE5A {
                shared_walker_state: SHARED_WALKER_STATE_PATROL,
                breath_timer: DRAGON_WALKING_BREATH_INITIAL,
                patrol_timer: DRAGON_WALKING_PATROL_LONG_FRAMES,
                collision_callback: "loc_3CE90",
                next: DragonWalkingInitialDispatchLoc3CE5A::NormalFacingDispatch,
            },
            "ASM Dragon.asm loc_3CE5A seeds `$42/$66/$54/a0` before non-child dragons continue to normal facing dispatch"
        );
        assert_eq!(
            dragon_walking_common_init_loc_3ce5a(true, Direction::Left).next,
            DragonWalkingInitialDispatchLoc3CE5A::SpawnedChildLeftPacing,
            "ASM Dragon.asm loc_3CE5A sends spawned clear-x_direction dragons to loc_3E7CA"
        );
        assert_eq!(
            dragon_walking_common_init_loc_3ce5a(true, Direction::Right).next,
            DragonWalkingInitialDispatchLoc3CE5A::SpawnedChildRightPacing,
            "ASM Dragon.asm loc_3CE5A sends spawned set-x_direction dragons to loc_3E6AE"
        );
        assert_eq!(
            spawn_shared_walker_state(EnemyFamily::DragonWalking),
            dragon_walking_common_init_loc_3ce5a(false, Direction::Left).shared_walker_state,
            "native DragonWalking shared state should expose the `$42(a3)` patrol seed from loc_3CE5A"
        );
        assert_eq!(
            spawn_shared_walker_patrol_timer(EnemyFamily::DragonWalking),
            dragon_walking_common_init_loc_3ce5a(false, Direction::Left).patrol_timer,
            "native DragonWalking patrol timer should expose the `$54(a5)` seed from loc_3CE5A"
        );
    }

    #[test]
    fn dragon_walking_normal_dispatch_matches_loc_3ce84() {
        assert_eq!(
            dragon_walking_normal_dispatch_loc_3ce84(Direction::Left),
            DragonWalkingNormalDispatchLoc3CE84::Loc3E546,
            "ASM Dragon.asm loc_3CE84 should send clear x_direction to loc_3E546"
        );
        assert_eq!(
            dragon_walking_normal_dispatch_loc_3ce84(Direction::Right),
            DragonWalkingNormalDispatchLoc3CE84::Loc3E392,
            "ASM Dragon.asm loc_3CE84 should send set x_direction to loc_3E392"
        );
        assert_eq!(
            dragon_walking_normal_dispatch_loc_3ce84(spawn_facing(
                EnemyFamily::DragonWalking,
                200 << 16,
                50 << 16
            )),
            DragonWalkingNormalDispatchLoc3CE84::Loc3E546,
            "native DragonWalking spawn-facing should feed the same clear-x_direction path from loc_3CE84"
        );
        assert_eq!(
            dragon_walking_normal_dispatch_loc_3ce84(spawn_facing(
                EnemyFamily::DragonWalking,
                200 << 16,
                300 << 16
            )),
            DragonWalkingNormalDispatchLoc3CE84::Loc3E392,
            "native DragonWalking spawn-facing should feed the same set-x_direction path from loc_3CE84"
        );
    }

    #[test]
    fn armadillo_hp_and_walk_speed_follow_loc_3c756_and_loc_3c766() {
        // ASM Armadillo.asm loc_3C756/loc_3C766: enemy_level is copied to d7,
        // then `addq.w #1,d7` produces HP 1/2/3. The walk-speed slot receives
        // `$8000` base plus one `$10000` increment per tier.
        let hp1 = spawn_hit_points(EnemyFamily::Armadillo, 0);
        let hp2 = spawn_hit_points(EnemyFamily::Armadillo, 1);
        let hp3 = spawn_hit_points(EnemyFamily::Armadillo, 2);

        assert_eq!(hp1, 1);
        assert_eq!(hp2, 2);
        assert_eq!(hp3, 3);

        assert_eq!(pacer_walk_speed(EnemyFamily::Armadillo, hp1), 0x0000_8000);
        assert_eq!(pacer_walk_speed(EnemyFamily::Armadillo, hp2), 0x0001_8000);
        assert_eq!(pacer_walk_speed(EnemyFamily::Armadillo, hp3), 0x0002_8000);
    }

    #[test]
    fn armadillo_initial_patrol_timer_matches_loc_3c7f8() {
        let mut rng = RomRandomState::default();
        assert_eq!(
            spawn_wait_timer(EnemyFamily::Armadillo, 0, 0, &mut rng),
            0x012C
        );
        assert_eq!(
            spawn_wait_timer(EnemyFamily::Armadillo, 2, 0, &mut rng),
            0x012C
        );
    }

    #[test]
    fn emo_rock_spawn_wait_timer_matches_loc_361f4_rng_mix() {
        let mut rng = RomRandomState::default();

        assert_eq!(
            emo_rock_wait_timer_with_rng(&mut rng, 0x0123_0000),
            0x009F,
            "ASM Emo_Rock.asm loc_361F4 seeds d0 with `$3C`, XORs the first RNG byte with Kid X low byte `$23`, clears bit 7, then adds the result"
        );
    }

    #[test]
    fn emo_rock_spawn_wait_timer_stays_within_loc_361f4_masked_range() {
        let mut rng = RomRandomState::default();
        let delay = emo_rock_wait_timer_with_rng(&mut rng, 0);

        assert!(
            (0x003C..=0x00BB).contains(&delay),
            "ASM Emo_Rock.asm loc_361F4 clears bit 7 before adding the `$3C` base delay, so the hover seed must stay within 60..187 frames"
        );
    }

    #[test]
    fn ufo_motion_scaling_matches_loc_34e10_acceleration_gate() {
        let command = UfoMotionCommand {
            x_acceleration: -0x1000,
            target_x_velocity: -0x20000,
            y_acceleration: 0x0800,
            target_y_velocity: 0x18000,
        };

        assert_eq!(
            ufo_double_acceleration_loc_34e10(command, false),
            command,
            "ASM UFO.asm loc_34E10 falls through without changing d0/d2 when `$4E(a5)` is zero"
        );
        assert_eq!(
            ufo_double_acceleration_loc_34e10(command, true),
            UfoMotionCommand {
                x_acceleration: -0x2000,
                target_x_velocity: -0x20000,
                y_acceleration: 0x1000,
                target_y_velocity: 0x18000,
            },
            "ASM UFO.asm loc_34E10 doubles acceleration registers d0/d2 while leaving target speed d1/d3 unchanged"
        );
    }

    #[test]
    fn ufo_motion_scaling_matches_loc_34e1c_terminal_gate() {
        let command = UfoMotionCommand {
            x_acceleration: -0x1000,
            target_x_velocity: -0x20000,
            y_acceleration: 0x0800,
            target_y_velocity: 0x18000,
        };

        assert_eq!(
            ufo_double_terminal_speed_loc_34e1c(command, false),
            command,
            "ASM UFO.asm loc_34E1C returns without changing d1/d3 when `$4F(a5)` is zero"
        );
        assert_eq!(
            ufo_double_terminal_speed_loc_34e1c(command, true),
            UfoMotionCommand {
                x_acceleration: -0x1000,
                target_x_velocity: -0x40000,
                y_acceleration: 0x0800,
                target_y_velocity: 0x30000,
            },
            "ASM UFO.asm loc_34E1C doubles terminal-speed registers d1/d3 while leaving acceleration d0/d2 unchanged"
        );
    }

    #[test]
    fn ufo_tracking_matches_loc_34e2a_below_near_kid_branch() {
        assert_eq!(
            ufo_tracking_command_loc_34e2a(
                0x0120_8000,
                0x0080_4000,
                0x0100_0000,
                0x0090_0000,
                true,
                false,
                true,
                true,
            ),
            Some(UfoTrackingCommandLoc34E2A {
                motion: UfoMotionCommand {
                    x_acceleration: -0x2000,
                    target_x_velocity: -0x40000,
                    y_acceleration: 0,
                    target_y_velocity: 0,
                },
                y_velocity: 0,
                facing: Direction::Left,
            }),
            "ASM UFO.asm loc_34E2A clears y_vel, steers left when UFO X is not less than Kid X, then calls loc_34E10 scaling"
        );

        assert_eq!(
            ufo_tracking_command_loc_34e2a(
                0x00D0_0000,
                0x0080_0000,
                0x0100_0000,
                0x0090_0000,
                true,
                false,
                false,
                false,
            ),
            Some(UfoTrackingCommandLoc34E2A {
                motion: UfoMotionCommand {
                    x_acceleration: 0x1000,
                    target_x_velocity: 0x20000,
                    y_acceleration: 0,
                    target_y_velocity: 0,
                },
                y_velocity: 0,
                facing: Direction::Right,
            }),
            "ASM UFO.asm loc_34E2A includes the Kid X - $30 edge and steers right when UFO X is less than Kid X"
        );
    }

    #[test]
    fn ufo_tracking_right_command_matches_loc_34e86() {
        assert_eq!(
            ufo_right_tracking_command_loc_34e86(),
            UfoMotionCommand {
                x_acceleration: 0x1000,
                target_x_velocity: 0x20000,
                y_acceleration: 0,
                target_y_velocity: 0,
            },
            "ASM UFO.asm loc_34E86 writes positive d0/d1 for rightward Kid chase before setting flipped x_direction"
        );

        assert_eq!(
            ufo_tracking_command_loc_34e2a(
                0x00D0_0000,
                0x0080_0000,
                0x0100_0000,
                0x0090_0000,
                true,
                false,
                false,
                false,
            )
            .map(|command| command.facing),
            Some(Direction::Right),
            "ASM UFO.asm loc_34E86 reaches `st x_direction(a3)`, represented by the right-facing/flipped native command"
        );
    }

    #[test]
    fn ufo_tracking_tail_matches_loc_34e96() {
        let command = UfoMotionCommand {
            x_acceleration: 0x1000,
            target_x_velocity: 0x20000,
            y_acceleration: -0x0800,
            target_y_velocity: -0x18000,
        };

        assert_eq!(
            ufo_tracking_tail_loc_34e96(command, false, false),
            command,
            "ASM UFO.asm loc_34E96 calls loc_34E10; with both scale flags clear the motion command is unchanged"
        );
        assert_eq!(
            ufo_tracking_tail_loc_34e96(command, true, true),
            UfoMotionCommand {
                x_acceleration: 0x2000,
                target_x_velocity: 0x40000,
                y_acceleration: -0x1000,
                target_y_velocity: -0x30000,
            },
            "ASM UFO.asm loc_34E96 shares the loc_34E10 acceleration and terminal-speed scaling tail"
        );
    }

    #[test]
    fn ufo_beam_start_matches_loc_34eec() {
        assert_eq!(
            ufo_beam_start_loc_34eec(),
            UfoBeamStartLoc34Eec {
                beam_segments: 4,
                beam_interval: 5,
                x_velocity: 0,
                y_velocity: 0,
                wait_timer: 0x32,
            },
            "ASM UFO.asm loc_34EEC writes `$48=4`, `$4A=5`, clears x/y velocity, and reloads `$44=$32`"
        );
    }

    #[test]
    fn ufo_beam_load_tick_matches_loc_34fd6() {
        assert_eq!(
            ufo_beam_load_tick_loc_34fd6(false, 4, 5),
            UfoBeamLoadTickLoc34FD6 {
                next_beam_segments: 4,
                next_beam_interval: 5,
                spawn_beam_segment: false,
                clear_ufo_shooting_state: false,
            },
            "ASM UFO.asm loc_34FD6 does nothing while `$46(a5)` is clear"
        );

        assert_eq!(
            ufo_beam_load_tick_loc_34fd6(true, 4, 1),
            UfoBeamLoadTickLoc34FD6 {
                next_beam_segments: 3,
                next_beam_interval: 5,
                spawn_beam_segment: true,
                clear_ufo_shooting_state: false,
            },
            "ASM UFO.asm loc_34FD6 reloads the 5-frame interval counter and decrements `$48(a5)` on each segment"
        );

        assert_eq!(
            ufo_beam_load_tick_loc_34fd6(true, 1, 1),
            UfoBeamLoadTickLoc34FD6 {
                next_beam_segments: 0,
                next_beam_interval: 5,
                spawn_beam_segment: false,
                clear_ufo_shooting_state: true,
            },
            "ASM UFO.asm loc_34FD6 enters loc_34FFE when the segment countdown reaches zero"
        );
    }

    #[test]
    fn ufo_apply_acceleration_axis_matches_loc_3501e() {
        assert_eq!(
            ufo_apply_acceleration_axis_loc_3501e(0x0800, 0x0300, -0x0100),
            0x0700,
            "ASM UFO.asm loc_3501E keeps accelerating while the next value stays above target"
        );
        assert_eq!(
            ufo_apply_acceleration_axis_loc_3501e(0x0800, 0x0700, -0x0200),
            0x0700,
            "ASM UFO.asm loc_3501E clamps to target when the next value would undershoot"
        );
    }

    #[test]
    fn ufo_apply_acceleration_axis_matches_loc_35044() {
        assert_eq!(
            ufo_apply_acceleration_axis_loc_35044(0x0300, 0x0180, -0x0100),
            0x0200,
            "ASM UFO.asm loc_35044 keeps accelerating on the negative branch"
        );
        assert_eq!(
            ufo_apply_acceleration_axis_loc_35044(0x0300, 0x0201, -0x0100),
            0x0201,
            "ASM UFO.asm loc_35044 clamps to target when the next value would pass below"
        );
    }

    #[test]
    fn ufo_apply_acceleration_axis_matches_loc_35050() {
        assert_eq!(
            ufo_apply_acceleration_axis_loc_35050(0x1234),
            0x1234,
            "ASM UFO.asm loc_35050 stores the chosen velocity and returns"
        );
    }

    #[test]
    fn ufo_collision_dispatch_matches_loc_35056() {
        assert_eq!(
            ufo_collision_dispatch_loc_35056(0),
            UfoCollisionDispatchLoc35056 {
                collision_type: 0,
                behavior: None,
            },
            "ASM UFO.asm loc_35056 returns through `rts` when collision_type is zero"
        );
        assert_eq!(
            ufo_collision_dispatch_loc_35056(-1),
            UfoCollisionDispatchLoc35056 {
                collision_type: 0,
                behavior: None,
            },
            "ASM UFO.asm loc_35056 sends negatives to loc_35070 and clears collision_type"
        );
        assert_eq!(
            ufo_collision_dispatch_loc_35056(0x0004),
            UfoCollisionDispatchLoc35056 {
                collision_type: 0,
                behavior: Some(UfoCollisionBehaviorLoc35056::BounceWall),
            },
            "ASM UFO.asm loc_35056 table-selects wall bounce for collision_type=$04"
        );
        assert_eq!(
            ufo_collision_dispatch_loc_35056(0x000C),
            UfoCollisionDispatchLoc35056 {
                collision_type: 0,
                behavior: Some(UfoCollisionBehaviorLoc35056::BounceFloorCeil),
            },
            "ASM UFO.asm loc_35056 table-selects floor/ceil bounce for collision_type=$0C"
        );
        assert_eq!(
            ufo_collision_dispatch_loc_35056(0x0014),
            UfoCollisionDispatchLoc35056 {
                collision_type: 0,
                behavior: Some(UfoCollisionBehaviorLoc35056::BounceSlopeUp),
            },
            "ASM UFO.asm loc_35056 table-selects up-slope bounce for collision_type=$14"
        );
        assert_eq!(
            ufo_collision_dispatch_loc_35056(0x001C),
            UfoCollisionDispatchLoc35056 {
                collision_type: 0,
                behavior: Some(UfoCollisionBehaviorLoc35056::Hurt),
            },
            "ASM UFO.asm loc_35056 routes projectile hits (and kid-jump equivalents) to hurt behavior"
        );
        assert_eq!(
            ufo_collision_dispatch_loc_35056(0x0020),
            UfoCollisionDispatchLoc35056 {
                collision_type: 0,
                behavior: Some(UfoCollisionBehaviorLoc35056::ReturnOnly),
            },
            "ASM UFO.asm loc_35056 maps `$20` to a return-only jump"
        );
    }

    #[test]
    fn ufo_collision_clear_matches_loc_35070() {
        assert_eq!(
            ufo_collision_clear_loc_35070(0x1234),
            UfoCollisionClearLoc35070 { collision_type: 0 },
            "ASM UFO.asm loc_35070 clears the `collision_type(a3)` word"
        );
    }

    #[test]
    fn ufo_position_restore_matches_loc_350f2() {
        assert_eq!(
            ufo_restore_position_loc_350f2(0x1000_0000, 0x2000_0000, 0x0100_0000, -0x0200_0000),
            UfoPositionRestoreLoc350F2 {
                x_pos: 0x0F00_0000,
                y_pos: 0x2020_0000,
            },
            "ASM UFO.asm loc_350F2 rolls the UFO back by one velocity step before slope bounces"
        );
    }

    #[test]
    fn ufo_bounce_wall_tail_matches_loc_35120() {
        assert_eq!(
            ufo_bounce_wall_tail_loc_35120(0x0100_0000, 0x0200_0000),
            UfoWallBounceLoc35120 {
                x_velocity: -0x0100_0000,
                d0: -0x0200_0000,
            },
            "ASM UFO.asm loc_35120 keeps negated `x_vel` and negates positive `d0`"
        );

        assert_eq!(
            ufo_bounce_wall_tail_loc_35120(-0x0100_0000, -0x0200_0000),
            UfoWallBounceLoc35120 {
                x_velocity: 0x0100_0000,
                d0: -0x0200_0000,
            },
            "ASM UFO.asm loc_35120 preserves negative `d0` and negates `x_vel`"
        );
    }

    #[test]
    fn ufo_bounce_floor_ceil_tail_matches_loc_35146() {
        assert_eq!(
            ufo_bounce_floor_ceil_tail_loc_35146(0x0100_0000, 0x0200_0000),
            UfoFloorCeilBounceLoc35146 {
                y_velocity: -0x0100_0000,
                d2: -0x0200_0000,
            },
            "ASM UFO.asm loc_35146 keeps negated `y_vel` and negates positive `d2`"
        );

        assert_eq!(
            ufo_bounce_floor_ceil_tail_loc_35146(-0x0100_0000, -0x0200_0000),
            UfoFloorCeilBounceLoc35146 {
                y_velocity: 0x0100_0000,
                d2: -0x0200_0000,
            },
            "ASM UFO.asm loc_35146 preserves negative `d2` and negates `y_vel`"
        );
    }

    #[test]
    fn ufo_hurt_tail_matches_loc_35174() {
        assert_eq!(
            ufo_hurt_tail_loc_35174(0x00C0_0000, true),
            UfoHurtSpawnTailLoc35174 {
                some_ufo_shooting_cleared: true,
                allocation_selector: 0x6000,
                driver_routine: "loc_355E2",
                driver_x_velocity: 0x00C0_0000,
                driver_timer_flag: -1,
                parent_moved: true,
                parent_y_velocity: 0x0003_0000,
                driver_animation: "stru_34CA0",
                parent_animation_flag_13_cleared: true,
            },
            "ASM UFO.asm loc_35174 clears `Some_UFO_Shooting`, allocates `$6000` driver, and seeds parent motion"
        );

        assert_eq!(
            ufo_hurt_tail_loc_35174(-0x0040_0000, false),
            UfoHurtSpawnTailLoc35174 {
                some_ufo_shooting_cleared: false,
                allocation_selector: 0x6000,
                driver_routine: "loc_355E2",
                driver_x_velocity: -0x0040_0000,
                driver_timer_flag: -1,
                parent_moved: true,
                parent_y_velocity: 0x0003_0000,
                driver_animation: "stru_34CA0",
                parent_animation_flag_13_cleared: true,
            },
            "ASM UFO.asm loc_35174 leaves `Some_UFO_Shooting` untouched when `$46(a5)` is already clear"
        );
    }

    #[test]
    fn ufo_hurt_driver_spawn_matches_loc_35180() {
        assert_eq!(
            ufo_hurt_driver_spawn_loc_35180(0x00C0_0000),
            UfoHurtDriverSpawnLoc35180 {
                allocation_selector: 0x6000,
                driver_routine: "loc_355E2",
                driver_flag_46: -1,
                driver_x_velocity: 0x00C0_0000,
                parent_moved: true,
                parent_y_velocity: 0x0003_0000,
            },
            "ASM UFO.asm loc_35180 allocates `$6000` driver, sets `$46=$FFFF`, copies parent x_vel and sets parent y_vel"
        );

        assert_eq!(
            ufo_hurt_driver_spawn_loc_35180(-0x0080_0000),
            UfoHurtDriverSpawnLoc35180 {
                allocation_selector: 0x6000,
                driver_routine: "loc_355E2",
                driver_flag_46: -1,
                driver_x_velocity: -0x0080_0000,
                parent_moved: true,
                parent_y_velocity: 0x0003_0000,
            },
            "ASM UFO.asm loc_35180 applies the same negative parent x_vel path as the hurt tail"
        );
    }

    #[test]
    fn ufo_hurt_hibernate_gate_matches_loc_351b6() {
        assert_eq!(
            ufo_hurt_hibernate_gate_loc_351b6(false),
            UfoHurtHibernateGateLoc351B6 {
                keep_hibernating: true,
                delete_current_object: false,
            },
            "ASM UFO.asm loc_351B6 keeps hibernating while `$19(a3)` is zero"
        );

        assert_eq!(
            ufo_hurt_hibernate_gate_loc_351b6(true),
            UfoHurtHibernateGateLoc351B6 {
                keep_hibernating: false,
                delete_current_object: true,
            },
            "ASM UFO.asm loc_351B6 deletes current object when `$19(a3)` is set"
        );
    }

    #[test]
    fn ufo_beam_segment_spawn_matches_loc_351f2() {
        assert_eq!(
            ufo_beam_segment_spawn_loc_351f2(-0x0003_0000, -5, 4),
            Some(UfoBeamSpawnLoc351EE {
                allocation_selector: 0x8000,
                beam_routine: "loc_354DA",
                beam_x_velocity: -0x0004_0000,
                beam_y_velocity: 0x0002_0000,
            }),
            "ASM UFO.asm loc_351F2 adds half parent x_vel to the signed table x-word and shifts the y-word independently"
        );

        assert_eq!(
            ufo_beam_segment_spawn_loc_351f2(0x0010_0000, 0, -5),
            None,
            "ASM UFO.asm loc_351F2 branches to the finish tail before reading the y-word when the x-word sentinel is zero"
        );
    }

    #[test]
    fn ufo_beam_spawn_matches_loc_351ee() {
        assert_eq!(
            ufo_beam_spawn_loc_351ee(0x0006_0000, &[(5, 4), (-5, -3), (0, 0), (1, 1)]),
            vec![
                UfoBeamSpawnLoc351EE {
                    allocation_selector: 0x8000,
                    beam_routine: "loc_354DA",
                    beam_x_velocity: 0x0005_8000,
                    beam_y_velocity: 0x0008_0000,
                },
                UfoBeamSpawnLoc351EE {
                    allocation_selector: 0x8000,
                    beam_routine: "loc_354DA",
                    beam_x_velocity: 0x0000_8000,
                    beam_y_velocity: -0x0001_8000,
                },
            ],
            "ASM UFO.asm loc_351EE reads the beam table, spawns each segment, and stops on x-word zero"
        );

        assert_eq!(
            ufo_beam_spawn_loc_351ee(0x0010_0000, &[(0, -5), (5, 5)]),
            Vec::<UfoBeamSpawnLoc351EE>::new(),
            "ASM UFO.asm loc_351EE exits immediately when sentinel x-word is zero"
        );
    }

    #[test]
    fn ufo_beam_finish_matches_loc_35228() {
        assert_eq!(
            ufo_beam_finish_loc_35228(0x0001_0000),
            UfoBeamFinishLoc35228 {
                parent_beam_spawned_latch: true,
                linked_object_meta: 0x000F,
                linked_object_animation: "stru_34D02",
                linked_object_flag_13: true,
                linked_object_flag_19: false,
                linked_object_is_moved: false,
                linked_object_level_collision: false,
                linked_object_routine: 1,
                parent_animation: "stru_34C92",
                parent_y_velocity: 0x0005_0000,
                d2: -0x0000_0C00,
                d3: 0,
            },
            "ASM UFO.asm loc_35228 links the child gfx object, resets its state flags, bumps parent y_vel by $40000, and returns d2=$FFFFF400/d3=0"
        );
    }

    #[test]
    fn ufo_linked_object_range_gate_matches_loc_35280() {
        assert_eq!(
            ufo_linked_object_range_gate_loc_35280(false),
            UfoLinkedObjectRangeGateLoc35280::Return,
            "ASM UFO.asm loc_35280 returns when the range check leaves the zero condition set"
        );

        assert_eq!(
            ufo_linked_object_range_gate_loc_35280(true),
            UfoLinkedObjectRangeGateLoc35280::RunCleanupDeleteTail,
            "ASM UFO.asm loc_35280 enters the cleanup/delete tail when the range check reports a non-zero result"
        );
    }

    #[test]
    fn ufo_linked_cleanup_count_matches_loc_3528a() {
        assert_eq!(
            ufo_linked_cleanup_count_loc_3528a(3, false),
            UfoLinkedCleanupCountLoc3528A {
                number_ufos_on_screen: 3,
                play_hover_sound: false,
            },
            "ASM UFO.asm loc_3528A skips Number_UFOs_OnScreen while `$50(a5)` is clear"
        );

        assert_eq!(
            ufo_linked_cleanup_count_loc_3528a(2, true),
            UfoLinkedCleanupCountLoc3528A {
                number_ufos_on_screen: 1,
                play_hover_sound: false,
            },
            "ASM UFO.asm loc_3528A decrements the UFO count and falls through silently when another UFO remains"
        );

        assert_eq!(
            ufo_linked_cleanup_count_loc_3528a(1, true),
            UfoLinkedCleanupCountLoc3528A {
                number_ufos_on_screen: 0,
                play_hover_sound: true,
            },
            "ASM UFO.asm loc_3528A plays sfx_UFO_hovering through j_PlaySound2 when the decrement reaches zero"
        );
    }

    #[test]
    fn ufo_linked_cleanup_shooting_matches_loc_352a6() {
        assert_eq!(
            ufo_linked_cleanup_shooting_loc_352a6(false, true),
            UfoLinkedCleanupShootingLoc352A6 {
                global_shooting_flag: true,
            },
            "ASM UFO.asm loc_352A6 leaves Some_UFO_Shooting untouched while `$46(a5)` is clear"
        );

        assert_eq!(
            ufo_linked_cleanup_shooting_loc_352a6(true, true),
            UfoLinkedCleanupShootingLoc352A6 {
                global_shooting_flag: false,
            },
            "ASM UFO.asm loc_352A6 clears Some_UFO_Shooting when `$46(a5)` is set"
        );
    }

    #[test]
    fn ufo_linked_cleanup_status_matches_loc_352b2() {
        assert_eq!(
            ufo_linked_cleanup_status_loc_352b2(0x05),
            DeactivationStatusActionLoc3C4C0::WriteRespawnMarker { index: 5 },
            "ASM UFO.asm loc_352D2 writes `$2168` to the positive `$42(a5)` status slot"
        );

        assert_eq!(
            ufo_linked_cleanup_status_loc_352b2(0xC3),
            DeactivationStatusActionLoc3C4C0::SubtractSpawnCounter { index: 3 },
            "ASM UFO.asm loc_352B2 masks bit-7/bit-6 sentinel slots to six bits before subtracting `$400`"
        );

        assert_eq!(
            ufo_linked_cleanup_status_loc_352b2(0x82),
            DeactivationStatusActionLoc3C4C0::DeleteOnly,
            "ASM UFO.asm loc_352E0 deletes without an EnemyStatus_Table write when bit 7 is set and bit 6 is clear"
        );
    }

    #[test]
    fn ufo_linked_object_x_matches_loc_3531a() {
        assert_eq!(
            ufo_linked_object_x_loc_3531a(0x0120_8000, Direction::Left),
            0x011C,
            "ASM UFO.asm loc_3531A subtracts d6=4 when x_direction is clear"
        );

        assert_eq!(
            ufo_linked_object_x_loc_3531a(0x0120_8000, Direction::Right),
            0x0124,
            "ASM UFO.asm loc_3531A subtracts d6=-4 when x_direction is set"
        );
    }

    #[test]
    fn ufo_apply_acceleration_axis_matches_loc_3502a() {
        assert_eq!(
            ufo_apply_acceleration_axis_loc_3502a(0x0800, 0x1000, 0x0100),
            0x0900,
            "ASM UFO.asm loc_3502A keeps accelerating on the positive branch while below target"
        );
        assert_eq!(
            ufo_apply_acceleration_axis_loc_3502a(0x0800, 0x0400, 0x0100),
            0x0400,
            "ASM UFO.asm loc_3502A caps velocity to target when the next value would exceed it"
        );
    }

    #[test]
    fn ufo_beam_clear_state_matches_loc_34ffe() {
        assert_eq!(
            ufo_beam_clear_shooting_state_loc_34ffe(),
            UfoBeamClearLoc34FFE {
                ufo_shooting_latch: false,
                global_shooting_flag: false,
            },
            "ASM UFO.asm loc_34FFE clears `$46(a5)` and `Some_UFO_Shooting`"
        );
    }

    #[test]
    fn ufo_on_screen_count_matches_loc_34f64() {
        assert_eq!(
            ufo_on_screen_count_loc_34f64(0),
            UfoOnScreenCountLoc34F64 {
                number_ufos_on_screen: 1,
                ufo_on_screen_latch: true,
            },
            "ASM UFO.asm loc_34F64 increments Number_UFOs_OnScreen and sets `$50(a5)` for a newly visible UFO"
        );

        assert_eq!(
            ufo_on_screen_count_loc_34f64(0xFFFF),
            UfoOnScreenCountLoc34F64 {
                number_ufos_on_screen: 0,
                ufo_on_screen_latch: true,
            },
            "ASM UFO.asm loc_34F64 uses 16-bit addq.w wrapping semantics for the global UFO count"
        );
    }

    #[test]
    fn ufo_off_screen_gate_matches_loc_34f6e() {
        assert_eq!(
            ufo_off_screen_gate_loc_34f6e(3, false),
            UfoOffScreenGateLoc34F6E {
                number_ufos_on_screen: 3,
                play_hover_stop_sound: false,
                clear_latch_tail: false,
            },
            "ASM UFO.asm loc_34F6E returns immediately when `$50(a5)` is clear"
        );

        assert_eq!(
            ufo_off_screen_gate_loc_34f6e(2, true),
            UfoOffScreenGateLoc34F6E {
                number_ufos_on_screen: 1,
                play_hover_stop_sound: false,
                clear_latch_tail: true,
            },
            "ASM UFO.asm loc_34F6E decrements Number_UFOs_OnScreen and skips the stop sound while another UFO remains"
        );

        assert_eq!(
            ufo_off_screen_gate_loc_34f6e(1, true),
            UfoOffScreenGateLoc34F6E {
                number_ufos_on_screen: 0,
                play_hover_stop_sound: true,
                clear_latch_tail: true,
            },
            "ASM UFO.asm loc_34F6E plays the hover-stop sound only when the decrement reaches zero"
        );
    }

    #[test]
    fn ufo_clear_on_screen_latch_matches_loc_34f88() {
        assert!(
            !ufo_clear_on_screen_latch_loc_34f88(),
            "ASM UFO.asm loc_34F88 clears `$50(a5)` with `sf` after the off-screen count/sound gate"
        );
    }

    #[test]
    fn ufo_tracking_matches_loc_34e2a_early_returns() {
        assert_eq!(
            ufo_tracking_command_loc_34e2a(
                0x0100_0000,
                0x0080_0000,
                0x0100_0000,
                0x0090_0000,
                false,
                false,
                false,
                false,
            ),
            None,
            "ASM UFO.asm loc_34E2A returns before checking positions when `$4D(a5)` is zero"
        );
        assert_eq!(
            ufo_tracking_command_loc_34e2a(
                0x0100_0000,
                0x0080_0000,
                0x0100_0000,
                0x0090_0000,
                true,
                true,
                false,
                false,
            ),
            None,
            "ASM UFO.asm loc_34E2A returns while `Some_UFO_Shooting` is non-zero"
        );
        assert_eq!(
            ufo_tracking_command_loc_34e2a(
                0x0100_0000,
                0x0080_0000,
                0x0100_0000,
                0x0080_0000,
                true,
                false,
                false,
                false,
            ),
            None,
            "ASM UFO.asm loc_34E2A requires Kid's integer Y word to be greater than UFO's"
        );
        assert_eq!(
            ufo_tracking_command_loc_34e2a(
                0x00CF_0000,
                0x0080_0000,
                0x0100_0000,
                0x0090_0000,
                true,
                false,
                false,
                false,
            ),
            None,
            "ASM UFO.asm loc_34E2A rejects UFO X below Kid X - $30"
        );
        assert_eq!(
            ufo_tracking_command_loc_34e2a(
                0x0131_0000,
                0x0080_0000,
                0x0100_0000,
                0x0090_0000,
                true,
                false,
                false,
                false,
            ),
            None,
            "ASM UFO.asm loc_34E2A rejects UFO X above Kid X + $30"
        );
    }

    #[test]
    fn tornado_pattern_zero_x_acceleration_matches_loc_35452() {
        assert_eq!(
            acceleration_for_target(0x0000_1000, 0, 0),
            -TORNADO_ACCELERATION,
            "ASM UFO.asm loc_35452 substitutes `$C00` for a zero X acceleration entry and negates it when x_vel is non-negative"
        );
        assert_eq!(
            acceleration_for_target(-0x0000_1000, 0, 0),
            TORNADO_ACCELERATION,
            "ASM UFO.asm loc_35452 keeps the substituted `$C00` positive when x_vel is negative"
        );
    }

    #[test]
    fn tornado_pattern_zero_y_acceleration_matches_loc_35496() {
        assert_eq!(
            acceleration_for_target(0x0000_1000, 0, 0),
            -TORNADO_ACCELERATION,
            "ASM UFO.asm loc_35496 substitutes `$C00` for a zero Y acceleration entry and negates it when y_vel is non-negative"
        );
        assert_eq!(
            acceleration_for_target(-0x0000_1000, 0, 0),
            TORNADO_ACCELERATION,
            "ASM UFO.asm loc_35496 keeps the substituted `$C00` positive when y_vel is negative"
        );
    }

    #[test]
    fn tornado_pattern_non_negative_x_velocity_matches_loc_3546e() {
        assert_eq!(
            acceleration_for_target(0x0000_3000, -0x0001_0000, -0x0000_0800),
            -0x0000_0800,
            "ASM UFO.asm loc_3546E keeps d0 when x_vel is non-negative and target d1 is negative"
        );
        assert_eq!(
            acceleration_for_target(0x0000_3000, 0x0000_4000, 0x0000_0800),
            0x0000_0800,
            "ASM UFO.asm loc_3546E keeps d0 when positive target d1 is not below current x_vel"
        );
        assert_eq!(
            acceleration_for_target(0x0000_3000, 0x0000_2000, 0x0000_0800),
            -0x0000_0800,
            "ASM UFO.asm loc_3546E negates d0 when positive target d1 is below current x_vel"
        );
    }

    #[test]
    fn tornado_pattern_non_negative_y_velocity_matches_loc_354b0() {
        assert_eq!(
            acceleration_for_target(0x0000_3000, -0x0001_0000, -0x0000_0800),
            -0x0000_0800,
            "ASM UFO.asm loc_354B0 keeps d2 when y_vel is non-negative and target d3 is negative"
        );
        assert_eq!(
            acceleration_for_target(0x0000_3000, 0x0000_4000, 0x0000_0800),
            0x0000_0800,
            "ASM UFO.asm loc_354B0 keeps d2 when positive target d3 is not below current y_vel"
        );
        assert_eq!(
            acceleration_for_target(0x0000_3000, 0x0000_2000, 0x0000_0800),
            -0x0000_0800,
            "ASM UFO.asm loc_354B0 negates d2 when positive target d3 is below current y_vel"
        );
    }

    #[test]
    fn tornado_pattern_negative_x_velocity_matches_loc_35488() {
        assert_eq!(
            acceleration_for_target(-0x0000_3000, 0x0001_0000, 0x0000_0800),
            0x0000_0800,
            "ASM UFO.asm loc_35488 keeps d0 when x_vel is negative and target d1 is non-negative"
        );
        assert_eq!(
            acceleration_for_target(-0x0000_3000, -0x0000_4000, -0x0000_0800),
            -0x0000_0800,
            "ASM UFO.asm loc_35488 keeps d0 when negative target d1 is not above current x_vel"
        );
        assert_eq!(
            acceleration_for_target(-0x0000_3000, -0x0000_2000, -0x0000_0800),
            0x0000_0800,
            "ASM UFO.asm loc_35488 negates d0 when negative target d1 is above current x_vel"
        );
    }

    #[test]
    fn tornado_pattern_negative_y_velocity_matches_loc_354ca() {
        assert_eq!(
            acceleration_for_target(-0x0000_3000, 0x0001_0000, 0x0000_0800),
            0x0000_0800,
            "ASM UFO.asm loc_354CA keeps d2 when y_vel is negative and target d3 is non-negative"
        );
        assert_eq!(
            acceleration_for_target(-0x0000_3000, -0x0000_4000, -0x0000_0800),
            -0x0000_0800,
            "ASM UFO.asm loc_354CA keeps d2 when negative target d3 is not above current y_vel"
        );
        assert_eq!(
            acceleration_for_target(-0x0000_3000, -0x0000_2000, -0x0000_0800),
            0x0000_0800,
            "ASM UFO.asm loc_354CA negates d2 when negative target d3 is above current y_vel"
        );
    }

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

        let mut rng = RomRandomState::default();
        assert_eq!(
            spawn_wait_timer(EnemyFamily::Drips, 0, 0x0123_0000, &mut rng),
            0x0013,
            "ASM Drips.asm loc_3340E mixes the first ROM RNG byte `$C0` with Kid integer X low byte `$23`"
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
    fn lion_init_visible_state_matches_loc_3df92() {
        assert_eq!(
            lion_init_visible_state_loc_3df92(2, 0x0200, 0x0210),
            LionInitVisibleState {
                status_flag_13: true,
                object_meta: 0x0092,
                palette_line: 2,
                current_hp: 8,
                facing: Direction::Right,
            },
            "ASM Lion.asm loc_3DF92 should set `$13`, object_meta `$12|$80`, palette line 2, HP enemy_level+6, and keep the prior `st x_direction` result when Kid is right"
        );
        assert_eq!(
            lion_init_visible_state_loc_3df92(0, 0x0200, 0x01F0).facing,
            Direction::Left,
            "ASM Lion.asm loc_3DF92 should preserve zero x_direction when the signed compare finds Kid left of Lion"
        );
        assert_eq!(
            lion_init_visible_state_loc_3df92(0, 0x7FFF, 0x8000).facing,
            Direction::Left,
            "ASM Lion.asm loc_3DF92 inherits the signed word `blt` spawn-facing compare"
        );
    }

    #[test]
    fn lion_init_motion_profile_matches_loc_3dfd8_binary_hp_branch() {
        assert_eq!(
            lion_init_motion_profile_loc_3dfd8(7),
            LionInitMotionProfile {
                walk_speed: 0x0001_8000,
                charge_speed: 0x0001_8000,
                animation_script: "stru_3DEDA",
            },
            "ASM Lion.asm loc_3DFD8 should use `$18000` and `stru_3DEDA` when current_hp is not exactly 8"
        );
        assert_eq!(
            lion_init_motion_profile_loc_3dfd8(8),
            LionInitMotionProfile {
                walk_speed: 0x0002_8000,
                charge_speed: 0x0002_8000,
                animation_script: "stru_3DEF4",
            },
            "ASM Lion.asm skips loc_3DFD8 only for current_hp exactly 8"
        );
        assert_eq!(
            lion_charge_speed(9),
            0x0001_8000,
            "ASM Lion.asm loc_3DFD8 is selected by `bne`, so HP above 8 is not a fast tier"
        );
    }

    #[test]
    fn lion_init_runtime_fields_match_loc_3dff0_common_tail() {
        let fast_profile = lion_init_motion_profile_loc_3dfd8(8);
        assert_eq!(
            lion_init_runtime_fields_loc_3dff0(fast_profile, 0x2A),
            LionInitRuntimeFields {
                ufo_beam_interval: 0x002F,
                ufo_beam_segments: 0x0014,
                tracking_callback: "LionTrackingCallback",
                animation_script: "stru_3DEF4",
                shared_walker_state: 0x000A,
                charge_timer: 0x003C,
                collision_routine: "LionCollisionRoutine",
                random_phase_byte: 0x2A,
            },
            "ASM Lion.asm loc_3DFF0 should seed the common runtime fields after the HP-specific animation script is selected"
        );

        let regular_profile = lion_init_motion_profile_loc_3dfd8(7);
        assert_eq!(
            lion_init_runtime_fields_loc_3dff0(regular_profile, 0),
            LionInitRuntimeFields {
                ufo_beam_interval: 0x002F,
                ufo_beam_segments: 0x0014,
                tracking_callback: "LionTrackingCallback",
                animation_script: "stru_3DEDA",
                shared_walker_state: 0x000A,
                charge_timer: LION_CHARGE_TIMER_INIT,
                collision_routine: "LionCollisionRoutine",
                random_phase_byte: 0x4B,
            },
            "ASM Lion.asm loc_3DFF0 remaps RNG byte 0 to `$4B` after storing the selected animation"
        );
    }

    #[test]
    fn lion_init_facing_dispatch_matches_loc_3e030() {
        assert_eq!(
            lion_init_facing_dispatch_loc_3e030(Direction::Left),
            LionFacingDispatch::LeftPacing,
            "ASM Lion.asm loc_3E030 should send clear `x_direction(a3)` to loc_3E546"
        );
        assert_eq!(
            lion_init_facing_dispatch_loc_3e030(Direction::Right),
            LionFacingDispatch::RightPacing,
            "ASM Lion.asm loc_3E030 should send set `x_direction(a3)` to loc_3E392"
        );
    }

    #[test]
    fn lion_tracking_gate_matches_loc_3e054_latched_outer_band() {
        assert_eq!(
            lion_tracking_gate_loc_3e054(0x0100 + 0x0041, 0x0100, true),
            LionTrackingGate::Hold,
            "ASM Lion.asm loc_3E054 should keep current facing while `$4E(a5)` is set and d4 is below +$A0"
        );
        assert_eq!(
            lion_tracking_gate_loc_3e054(0x0100 - 0x0040, 0x0100, true),
            LionTrackingGate::Hold,
            "ASM Lion.asm loc_3E054 should keep current facing while `$4E(a5)` is set and d4 is above -$A0"
        );
    }

    #[test]
    fn lion_tracking_gate_matches_loc_3e054_clear_latch_bypass() {
        assert_eq!(
            lion_tracking_gate_loc_3e054(0x0100 + 0x0041, 0x0100, false),
            LionTrackingGate::TrackKid,
            "ASM Lion.asm loc_3E054 should fall through to the facing update immediately when `$4E(a5)` is clear"
        );
    }

    #[test]
    fn lion_clear_tracking_latch_matches_loc_3e066() {
        assert_eq!(
            lion_clear_tracking_latch_loc_3e066(0x0200, 0x0300),
            LionLatchClearResult {
                range_latch_set: false,
                facing: Direction::Right,
            },
            "ASM Lion.asm loc_3E066 should clear `$4E(a5)` and keep negative d4 facing toward Kid on the right"
        );
        assert_eq!(
            lion_clear_tracking_latch_loc_3e066(0x0300, 0x0200),
            LionLatchClearResult {
                range_latch_set: false,
                facing: Direction::Left,
            },
            "ASM Lion.asm loc_3E066 should clear `$4E(a5)` and keep non-negative d4 facing toward Kid on the left"
        );
    }

    #[test]
    fn lion_track_kid_facing_matches_loc_3e06a_nonnegative_delta() {
        assert_eq!(
            lion_track_kid_nonnegative_delta_facing_loc_3e06a(0x0300, 0x0200),
            Direction::Left,
            "ASM Lion.asm loc_3E06A should clear x_direction when signed d4 = lion_x - kid_x is positive"
        );
        assert_eq!(
            lion_track_kid_nonnegative_delta_facing_loc_3e06a(0x0200, 0x0200),
            Direction::Left,
            "ASM Lion.asm loc_3E06A should also clear x_direction when signed d4 is zero"
        );
    }

    #[test]
    fn lion_track_kid_facing_matches_loc_3e074_negative_delta() {
        assert_eq!(
            lion_track_kid_negative_delta_facing_loc_3e074(0x0200, 0x0300),
            Direction::Right,
            "ASM Lion.asm loc_3E074 should store x_direction when signed d4 = lion_x - kid_x is negative"
        );
        assert_eq!(
            lion_track_kid_negative_delta_facing_loc_3e074(0x7FFF, 0x8000),
            Direction::Right,
            "ASM Lion.asm loc_3E074 inherits 68k signed-word comparison semantics across the sign boundary"
        );
    }

    #[test]
    fn lion_roar_wait_dispatch_matches_loc_3e09e() {
        assert_eq!(
            lion_roar_wait_dispatch_loc_3e09e(LION_COLID_HURT, false, 0x0200, 0x0300),
            LionRoarWaitDispatch::Damage,
            "ASM Lion.asm loc_3E09E should send colid_hurt to the damage path before clearing collision_type"
        );
        assert_eq!(
            lion_roar_wait_dispatch_loc_3e09e(LION_COLID_KIDABOVE, true, 0x0200, 0x0300),
            LionRoarWaitDispatch::Damage,
            "ASM Lion.asm loc_3E09E should also prioritise colid_kidabove over the ready-to-fire sprite check"
        );
        assert_eq!(
            lion_roar_wait_dispatch_loc_3e09e(0x0020, false, 0x0200, 0x0300),
            LionRoarWaitDispatch::ContinueWaiting {
                next_collision_type: 0,
            },
            "ASM Lion.asm loc_3E09E should clear non-damaging collision_type and loop until the fire sprite is reached"
        );
        assert_eq!(
            lion_roar_wait_dispatch_loc_3e09e(0, true, 0x0200, 0x0300),
            LionRoarWaitDispatch::ReadyToFire {
                next_collision_type: 0,
                facing: Direction::Right,
                projectile_worker_selector: 0x8000,
                projectile_worker: "LionProjectileWorker",
                active_projectile_count_delta: 1,
            },
            "ASM Lion.asm loc_3E09E should clear collision_type, face Kid, allocate the projectile worker, and increment the active projectile count when the roar sprite is ready"
        );
        assert_eq!(
            lion_roar_wait_dispatch_loc_3e09e(0, true, 0x0300, 0x0200),
            LionRoarWaitDispatch::ReadyToFire {
                next_collision_type: 0,
                facing: Direction::Left,
                projectile_worker_selector: 0x8000,
                projectile_worker: "LionProjectileWorker",
                active_projectile_count_delta: 1,
            },
            "ASM Lion.asm loc_3E09E should choose the clear x_direction orientation when Kid is not to the right"
        );
    }

    #[test]
    fn lion_post_fire_wait_dispatch_matches_loc_3e0f4() {
        assert_eq!(
            lion_post_fire_wait_dispatch_loc_3e0f4(LION_COLID_HURT, false),
            LionPostFireWaitDispatch::Damage,
            "ASM Lion.asm loc_3E0F4 should send colid_hurt to the damage path before clearing collision_type"
        );
        assert_eq!(
            lion_post_fire_wait_dispatch_loc_3e0f4(LION_COLID_KIDABOVE, true),
            LionPostFireWaitDispatch::Damage,
            "ASM Lion.asm loc_3E0F4 should prioritise colid_kidabove over the `$18(a3)` completion flag"
        );
        assert_eq!(
            lion_post_fire_wait_dispatch_loc_3e0f4(0x0020, false),
            LionPostFireWaitDispatch::ContinueWaiting {
                next_collision_type: 0,
            },
            "ASM Lion.asm loc_3E0F4 should clear non-damaging collision_type and loop while `$18(a3)` is clear"
        );
        assert_eq!(
            lion_post_fire_wait_dispatch_loc_3e0f4(0, true),
            LionPostFireWaitDispatch::ResumeCharge {
                next_collision_type: 0,
                animation_script: "saved_lion_charge_animation",
            },
            "ASM Lion.asm loc_3E0F4 should restore the animation script saved in `$62(a5)` when `$18(a3)` is set"
        );
    }

    #[test]
    fn lion_charge_direction_dispatch_matches_loc_3e11e() {
        assert_eq!(
            lion_charge_direction_dispatch_loc_3e11e(Direction::Left),
            LionChargeDirectionDispatch::LeftCharge,
            "ASM Lion.asm loc_3E11E should branch clear x_direction to loc_3E566"
        );
        assert_eq!(
            lion_charge_direction_dispatch_loc_3e11e(Direction::Right),
            LionChargeDirectionDispatch::RightCharge,
            "ASM Lion.asm loc_3E11E should branch set x_direction to loc_3E3B2"
        );
    }

    #[test]
    fn lion_projectile_launch_tail_matches_loc_3e180() {
        assert_eq!(
            lion_projectile_launch_tail_loc_3e180(Direction::Left),
            LionProjectileLaunchTail {
                x_offset_pixels: -10,
                x_velocity: -0x0006_0000,
                animation_script: "stru_3DF34",
            },
            "ASM Lion.asm loc_3E180 should apply the negated d7/d6 pair prepared for clear parent x_direction"
        );
        assert_eq!(
            lion_projectile_launch_tail_loc_3e180(Direction::Right),
            LionProjectileLaunchTail {
                x_offset_pixels: 10,
                x_velocity: 0x0006_0000,
                animation_script: "stru_3DF34",
            },
            "ASM Lion.asm loc_3E180 should keep the positive d7/d6 pair when parent x_direction is set"
        );
    }

    #[test]
    fn lion_hibernate_tail_matches_loc_3e3a2() {
        assert_eq!(
            lion_hibernate_tail_loc_3e3a2(),
            LionHibernateLoc3E3A2 {
                x_velocity: 0,
                y_velocity: 0,
            },
            "ASM Lion.asm loc_3E3A2 should force zeroed velocities during the one-frame charge pause before re-entering the locomotion routine pointer"
        );
    }

    #[test]
    fn lion_charge_terrain_gate_matches_loc_3e406() {
        assert_eq!(
            lion_charge_terrain_gate_loc_3e406(false, false),
            LionLoc3E406Branch::ContinueTo3E434,
            "ASM Lion.asm loc_3E406 should continue to loc_3E434 when terrain bit0 is clear"
        );
        assert_eq!(
            lion_charge_terrain_gate_loc_3e406(true, false),
            LionLoc3E406Branch::EnterLoc3E48E,
            "ASM Lion.asm loc_3E406 should route terrain-bit hits to loc_3E48E unless the low object_meta byte equals $10"
        );
        assert_eq!(
            lion_charge_terrain_gate_loc_3e406(true, true),
            LionLoc3E406Branch::EnterLoc3E3A2,
            "ASM Lion.asm loc_3E406 should force a shared charge reset path into loc_3E3A2 when terrain bit0 is set and object_meta low byte is $10"
        );
    }

    #[test]
    fn lion_charge_window_gate_matches_loc_3e434() {
        assert_eq!(
            lion_charge_window_gate_loc_3e434(15),
            LionLoc3E434Result {
                next_shared_state: 15,
                branch: LionLoc3E434Branch::Check70Band,
            },
            "ASM Lion.asm loc_3E434 should select the tight ±$70 overshoot check path for positive shared_walker_state"
        );
        assert_eq!(
            lion_charge_window_gate_loc_3e434(0x000A),
            LionLoc3E434Result {
                next_shared_state: 0x000A,
                branch: LionLoc3E434Branch::Check70Band,
            },
            "ASM Lion.asm loc_3E434 should keep state `$0A` and continue to `loc_3E476`"
        );
        assert_eq!(
            lion_charge_window_gate_loc_3e434(0),
            LionLoc3E434Result {
                next_shared_state: 0,
                branch: LionLoc3E434Branch::Check60Band,
            },
            "ASM Lion.asm loc_3E434 should route state 0 to the ±$60 overshoot check path at loc_3E450"
        );
        assert_eq!(
            lion_charge_window_gate_loc_3e434(-1),
            LionLoc3E434Result {
                next_shared_state: 0,
                branch: LionLoc3E434Branch::KeepAndContinue,
            },
            "ASM Lion.asm loc_3E434 should increment negative states toward zero and continue the charge step"
        );
    }

    #[test]
    fn lion_left_charge_calm_distance_gate_matches_loc_3e604() {
        assert_eq!(
            lion_left_charge_calm_distance_gate_loc_3e604(0x0100, 0x0161),
            LionLoc3E604Branch::ContinueMovement,
            "ASM Lion.asm loc_3E604 should continue movement when d4 is greater than +$60"
        );
        assert_eq!(
            lion_left_charge_calm_distance_gate_loc_3e604(0x0100, 0x00A0),
            LionLoc3E604Branch::EnterLoc3E50A,
            "ASM Lion.asm loc_3E604 should keep -$60 inside the face-left branch band"
        );
        assert_eq!(
            lion_left_charge_calm_distance_gate_loc_3e604(0x0100, 0x0160),
            LionLoc3E604Branch::EnterLoc3E4CE,
            "ASM Lion.asm loc_3E604 should send positive in-band d4 values to loc_3E4CE"
        );
        assert_eq!(
            lion_left_charge_calm_distance_gate_loc_3e604(0x0100, 0x0100),
            LionLoc3E604Branch::EnterLoc3E50A,
            "ASM Lion.asm loc_3E604 should send zero d4 to loc_3E50A"
        );
        assert_eq!(
            lion_left_charge_calm_distance_gate_loc_3e604(0x7FFF, 0x8000),
            LionLoc3E604Branch::EnterLoc3E4CE,
            "ASM Lion.asm loc_3E604 should use signed-word wrapping for d4"
        );
    }

    #[test]
    fn lion_left_charge_alert_release_gate_matches_loc_3e62a() {
        assert_eq!(
            lion_left_charge_alert_release_gate_loc_3e62a(0x0100, 0x0171),
            LionLoc3E62ABranch::EnterLoc3E642,
            "ASM Lion.asm loc_3E62A should enter loc_3E642 when d4 is greater than +$70"
        );
        assert_eq!(
            lion_left_charge_alert_release_gate_loc_3e62a(0x0100, 0x0170),
            LionLoc3E62ABranch::ContinueMovement,
            "ASM Lion.asm loc_3E62A should keep +$70 in the movement band"
        );
        assert_eq!(
            lion_left_charge_alert_release_gate_loc_3e62a(0x0100, 0x0091),
            LionLoc3E62ABranch::ContinueMovement,
            "ASM Lion.asm loc_3E62A should continue when d4 is greater than -$70"
        );
        assert_eq!(
            lion_left_charge_alert_release_gate_loc_3e62a(0x0100, 0x0090),
            LionLoc3E62ABranch::EnterLoc3E642,
            "ASM Lion.asm loc_3E62A should release on the exact -$70 boundary"
        );
        assert_eq!(
            lion_left_charge_alert_release_gate_loc_3e62a(0x7FFF, 0x8000),
            LionLoc3E62ABranch::ContinueMovement,
            "ASM Lion.asm loc_3E62A should compute d4 with signed-word wrapping"
        );
    }

    #[test]
    fn lion_right_charge_continue_matches_loc_3e4c6() {
        assert_eq!(
            lion_right_charge_continue_loc_3e4c6(0x0123_8000),
            LionLoc3E4C6Result {
                x_position: 0x0123_8000,
                x_velocity: 0,
                y_velocity: 0,
            },
            "ASM Lion.asm loc_3E4C6 should commit `d2` to x_pos before entering the loc_3E3A2 zero-velocity hibernate tail"
        );
    }

    #[test]
    fn lion_left_charge_continue_matches_loc_3e67a() {
        assert_eq!(
            lion_left_charge_continue_loc_3e67a(0x0456_8000),
            LionLoc3E67AResult {
                x_position: 0x0456_8000,
                hibernate_one_frame: true,
                cull_routine: "sub_3E8BA",
                resume_via_callback: true,
            },
            "ASM Lion.asm loc_3E67A should commit `d2` to x_pos and enter the loc_3E556 hibernate/cull/resume tail"
        );
    }

    #[test]
    fn lion_left_charge_x_commit_matches_loc_3e6f0() {
        assert_eq!(
            lion_left_charge_x_commit_loc_3e6f0(0x0455_8000),
            LionLeftChargeXCommitLoc3E6F0 {
                x_position: 0x0455_8000,
                enters_vertical_motion: true,
            },
            "ASM Lion.asm loc_3E6F0 should commit unobstructed d2 into x_pos before entering vertical motion"
        );
    }

    #[test]
    fn lion_left_charge_vertical_entry_matches_loc_3e6f6() {
        assert_eq!(
            lion_left_charge_vertical_entry_loc_3e6f6(0x0042_0000, 0x0001_0000, 6),
            LionLeftChargeVerticalEntryLoc3E6F6::DownwardOrStationary {
                candidate_y_position: 0x0043_0000,
                pixel_delta: 1,
            },
            "ASM Lion.asm loc_3E6F6 should route non-negative integer Y deltas to the downward/stationary path"
        );
        assert_eq!(
            lion_left_charge_vertical_entry_loc_3e6f6(0x0042_0000, -0x0002_0000, 6),
            LionLeftChargeVerticalEntryLoc3E6F6::UpwardCollisionCheck {
                candidate_y_position: 0x0040_0000,
                upward_pixels: 2,
                ceiling_clearance_pixels: 12,
                needs_ceiling_probe: false,
            },
            "ASM Lion.asm loc_3E6F6 should skip the ceiling probe when block-grid clearance covers upward movement"
        );
        assert_eq!(
            lion_left_charge_vertical_entry_loc_3e6f6(0x0042_0000, -0x000D_0000, 6),
            LionLeftChargeVerticalEntryLoc3E6F6::UpwardCollisionCheck {
                candidate_y_position: 0x0035_0000,
                upward_pixels: 13,
                ceiling_clearance_pixels: 12,
                needs_ceiling_probe: true,
            },
            "ASM Lion.asm loc_3E6F6 should request a ceiling probe when upward movement crosses the nibble clearance"
        );
    }

    #[test]
    fn lion_left_charge_downward_collision_matches_loc_3e732() {
        assert_eq!(
            lion_left_charge_downward_collision_loc_3e732(0x0042_0000, 0x0043_0000, 1, true, 0,),
            LionLeftChargeDownwardCollisionLoc3E732::Airborne {
                y_position: 0x0043_0000,
                floor_clearance_pixels: 14,
                needs_floor_probe: false,
            },
            "ASM Lion.asm loc_3E732 should skip floor probing when nibble clearance covers downward motion"
        );
        assert_eq!(
            lion_left_charge_downward_collision_loc_3e732(0x0042_0000, 0x0052_0000, 16, false, 0,),
            LionLeftChargeDownwardCollisionLoc3E732::Airborne {
                y_position: 0x0052_0000,
                floor_clearance_pixels: 14,
                needs_floor_probe: true,
            },
            "ASM Lion.asm loc_3E732 should keep airborne motion when the floor probe misses"
        );
        assert_eq!(
            lion_left_charge_downward_collision_loc_3e732(0x0042_0000, 0x0052_0000, 16, true, 0,),
            LionLeftChargeDownwardCollisionLoc3E732::Landed {
                y_position: 0x0050_0000,
                y_velocity: 0,
                left_charge_latch_set: false,
                object_meta_bit7_cleared: true,
                restores_saved_charge_speed: true,
                next_shared_walker_state: Some(-0x78),
                restored_animation: "saved_lion_charge_animation",
            },
            "ASM Lion.asm loc_3E732 should snap to the floor boundary, clear airborne state, restore charge speed, and reset non-$0A shared state"
        );
        assert_eq!(
            lion_left_charge_downward_collision_loc_3e732(
                0x0042_0000,
                0x0052_0000,
                16,
                true,
                0x000A,
            ),
            LionLeftChargeDownwardCollisionLoc3E732::Landed {
                y_position: 0x0050_0000,
                y_velocity: 0,
                left_charge_latch_set: false,
                object_meta_bit7_cleared: true,
                restores_saved_charge_speed: true,
                next_shared_walker_state: None,
                restored_animation: "saved_lion_charge_animation",
            },
            "ASM Lion.asm loc_3E732 should preserve shared state $0A while still restoring the saved animation"
        );
    }

    #[test]
    fn lion_left_charge_entry_matches_loc_3e682() {
        assert_eq!(
            lion_left_charge_entry_loc_3e682(0x2340),
            LionLeftChargeEntryLoc3E682 {
                left_charge_latch_set: true,
                restored_sprite_offset: Some(0x2340),
                is_animated: Some(false),
            },
            "ASM Lion.asm loc_3E682 should set `$5C(a5)`, restore a non-zero `$4A(a3)` sprite offset, and clear `is_animated(a3)`"
        );
        assert_eq!(
            lion_left_charge_entry_loc_3e682(0),
            LionLeftChargeEntryLoc3E682 {
                left_charge_latch_set: true,
                restored_sprite_offset: None,
                is_animated: None,
            },
            "ASM Lion.asm loc_3E682 should skip sprite and animation writes when `$4A(a3)` is zero"
        );
    }

    #[test]
    fn lion_left_charge_motion_seed_matches_loc_3e694() {
        assert_eq!(
            lion_left_charge_motion_seed_loc_3e694(-0x0001_8000, 0x0042_8000),
            LionLeftChargeMotionSeedLoc3E694 {
                x_velocity: -0x0001_8000,
                y_velocity: 0,
                y_position: 0x0043_8000,
            },
            "ASM Lion.asm loc_3E694 should restore `$50(a5)` into x_vel, clear y_vel, and add one pixel to the y_pos word"
        );
        assert_eq!(
            lion_left_charge_motion_seed_loc_3e694(0x0002_8000, 0xFFFF_1234),
            LionLeftChargeMotionSeedLoc3E694 {
                x_velocity: 0x0002_8000,
                y_velocity: 0,
                y_position: 0x0000_1234,
            },
            "ASM Lion.asm loc_3E694 inherits 16-bit word wrapping for `addq.w #1,y_pos(a3)`"
        );
    }

    #[test]
    fn lion_left_charge_airborne_resume_matches_loc_3e6a4() {
        assert_eq!(
            lion_left_charge_airborne_resume_loc_3e6a4(),
            LionLoc3E556Tail {
                hibernate_one_frame: true,
                cull_routine: "sub_3E8BA",
                resume_via_callback: true,
            },
            "ASM Lion.asm loc_3E6A4 should hibernate, run sub_3E8BA, and jump through a0 without zeroing velocities"
        );
    }

    #[test]
    fn lion_hibernate_cull_resume_matches_loc_3e556() {
        assert_eq!(
            lion_hibernate_cull_resume_loc_3e556(),
            LionLoc3E556Tail {
                hibernate_one_frame: true,
                cull_routine: "sub_3E8BA",
                resume_via_callback: true,
            },
            "ASM Lion.asm loc_3E556 should hibernate, run sub_3E8BA, then jump through the restored a0 routine pointer without touching velocity fields"
        );
    }

    #[test]
    fn lion_left_charge_terrain_gate_matches_loc_3e5ba() {
        assert_eq!(
            lion_left_charge_terrain_gate_loc_3e5ba(false, false),
            LionLeftChargeTerrainResult {
                branch: LionLeftChargeTerrainBranch::ContinueToLoc3E5E8,
                next_shared_state: None,
                restores_charge_speed: false,
                restored_animation: None,
            },
            "ASM Lion.asm loc_3E5BA should continue to loc_3E5E8 when terrain bit 0 is clear"
        );
        assert_eq!(
            lion_left_charge_terrain_gate_loc_3e5ba(true, false),
            LionLeftChargeTerrainResult {
                branch: LionLeftChargeTerrainBranch::EnterLoc3E642,
                next_shared_state: None,
                restores_charge_speed: false,
                restored_animation: None,
            },
            "ASM Lion.asm loc_3E5BA should route non-$10 object ids with terrain bit 0 set to loc_3E642"
        );
        assert_eq!(
            lion_left_charge_terrain_gate_loc_3e5ba(true, true),
            LionLeftChargeTerrainResult {
                branch: LionLeftChargeTerrainBranch::ResetAndEnterLoc3E556,
                next_shared_state: Some(-0x78),
                restores_charge_speed: true,
                restored_animation: Some("saved_lion_charge_animation"),
            },
            "ASM Lion.asm loc_3E5BA should set `$42(a3)` to `$FF88`, restore `$5E/$62`, then enter loc_3E556 for object id $10"
        );
    }

    #[test]
    fn loc_3e318_kid_collision_routes_to_kill() {
        assert_eq!(
            lion_collision_response_loc_3e318(-1, 4, false, true),
            LionCollisionLoc3E318::Kill,
            "ASM Lion.asm loc_3E378 must be used for signed negative collision IDs"
        );
    }

    #[test]
    fn loc_3e318_hurt_collision_decrements_hp_and_restores_animation() {
        assert_eq!(
            lion_collision_response_loc_3e318(LION_COLID_HURT as i16, 3, false, false),
            LionCollisionLoc3E318::Hurt {
                remaining_hp: 2,
                damage_animation: "stru_3DF42",
                restored_animation: "saved_lion_charge_animation",
            },
            "ASM Lion.asm loc_3E32C should subtract one HP and restore the saved `$62(a5)` animation"
        );
    }

    #[test]
    fn loc_3e318_hurt_collision_with_last_hp_kills() {
        assert_eq!(
            lion_collision_response_loc_3e318(LION_COLID_KIDABOVE as i16, 1, false, false),
            LionCollisionLoc3E318::Kill,
            "ASM Lion.asm loc_3E32C must fall into the same kill path when HP reaches zero"
        );
    }

    #[test]
    fn loc_3e318_dispatches_by_5c_and_facing() {
        assert_eq!(
            lion_collision_response_loc_3e318(0, 4, false, false,),
            LionCollisionLoc3E318::Continue {
                next: LionCollisionResumeLoc3E318::Loc3E560,
            },
            "ASM Lion.asm loc_3E36C should jump to loc_3E560 when `$5C(a5)` is clear and x_direction is not set"
        );
        assert_eq!(
            lion_collision_response_loc_3e318(0, 4, false, true),
            LionCollisionLoc3E318::Continue {
                next: LionCollisionResumeLoc3E318::Loc3E3AC,
            },
            "ASM Lion.asm loc_3E36C should jump to loc_3E3AC when `$5C(a5)` is clear and x_direction is set"
        );
        assert_eq!(
            lion_collision_response_loc_3e318(0, 4, true, false),
            LionCollisionLoc3E318::Continue {
                next: LionCollisionResumeLoc3E318::Loc3E7CA,
            },
            "ASM Lion.asm loc_3E356 should jump to loc_3E7CA when `$5C(a5)` is set and x_direction is clear"
        );
        assert_eq!(
            lion_collision_response_loc_3e318(0, 4, true, true),
            LionCollisionLoc3E318::Continue {
                next: LionCollisionResumeLoc3E318::Loc3E6AE,
            },
            "ASM Lion.asm loc_3E356 should jump to loc_3E6AE when `$5C(a5)` and x_direction are set"
        );
    }

    #[test]
    fn tank_child_layout_matches_init_offsets_from_loc_3b372() {
        // ASM Tank.asm loc_3B372 copies the parent x/y into the child sprite,
        // then `subi.w #8,$1E(a1)` lifts the child 8px above the body.
        let layout = shared_enemy_attachment_sub_3BF72(SharedEnemyAttachmentInputSub3BF72 {
            family: EnemyFamily::Tank,
            parent_x_pos: 0x0120,
            parent_y_pos: 0x0088,
            parent_x_direction: 0xFF,
            child_y_offset: 8,
            robot_parent_state: 0,
            robot_child_mapping: 0,
            child_x_direction: 0,
        });

        assert_eq!(
            layout.primary,
            SharedEnemyAttachmentChildSub3BF72 {
                x_pos: 0x0120,
                y_pos: 0x0080,
                x_velocity: 0,
                x_direction: Some(0xFF),
            }
        );
        assert_eq!(layout.secondary, None);
    }

    #[test]
    fn mini_hopping_skull_spawn_gate_matches_loc_3e9d0_edges() {
        // ASM Mini_Hopping_Skull.asm loc_3E9D0 keeps hibernating outside
        // Kid-relative X [-$50,$50] and Y [0,$A0] from the layout point.
        assert!(mini_hopping_skull_spawn_gate_loc_3e9d0(
            200,
            100,
            120 << 16,
            100 << 16
        ));
        assert!(mini_hopping_skull_spawn_gate_loc_3e9d0(
            200,
            100,
            280 << 16,
            260 << 16
        ));
        assert!(!mini_hopping_skull_spawn_gate_loc_3e9d0(
            200,
            100,
            119 << 16,
            100 << 16
        ));
        assert!(!mini_hopping_skull_spawn_gate_loc_3e9d0(
            200,
            100,
            200 << 16,
            99 << 16
        ));
        assert!(!mini_hopping_skull_spawn_gate_loc_3e9d0(
            200,
            100,
            200 << 16,
            261 << 16
        ));
    }

    #[test]
    fn mini_hopping_skull_initial_y_velocity_matches_loc_3ea48() {
        assert_eq!(
            spawn_y_velocity(EnemyFamily::MiniHoppingSkull, 0),
            0x0002_8000,
            "ASM Mini_Hopping_Skull.asm loc_3EA48 seeds base y_vel with `$28000`"
        );
        assert_eq!(
            spawn_y_velocity(EnemyFamily::MiniHoppingSkull, 1),
            0x0002_9000,
            "ASM Mini_Hopping_Skull.asm loc_3EA48 adds `$1000` when enemy_level is non-zero"
        );
        assert_eq!(
            spawn_y_velocity(EnemyFamily::MiniHoppingSkull, 2),
            0x0002_9000,
            "ASM Mini_Hopping_Skull.asm loc_3EA48 tests non-zero enemy_level, not a tiered table"
        );
    }

    #[test]
    fn mini_hopping_skull_first_air_tick_applies_loc_3ea88_gravity() {
        let data = vec![0u8; 8 * 4 * 2];
        let grid = terrain::BlockGrid::from_level_layout(&data, 8, 4);
        let mut enemy = ActiveEnemy {
            on_ground: false,
            y_velocity: spawn_y_velocity(EnemyFamily::MiniHoppingSkull, 0),
            ..mini_hopping_skull_test_enemy()
        };

        step_hopper(&mut enemy, Some(&grid), None);

        assert_eq!(
            enemy.y_velocity, 0x0002_9200,
            "ASM Mini_Hopping_Skull.asm loc_3EA88 adds `$1200` gravity to the `$28000` loc_3EA48 seed"
        );
        assert_eq!(
            enemy.y_pos,
            (32 << 16) + 0x0002_9200,
            "ASM Mini_Hopping_Skull.asm loc_3EA88 applies the gravity-adjusted velocity during the airborne frame"
        );
    }

    #[test]
    fn mini_hopping_skull_air_sprite_matches_loc_3eaac_falling_branch() {
        assert_eq!(
            mini_hopping_skull_air_sprite_after_loc_3ea88(false, -1),
            Some(MiniHoppingSkullAirSprite::Rising),
            "ASM Mini_Hopping_Skull.asm branches around loc_3EAAC while `y_vel(a3)` is negative"
        );
        assert_eq!(
            mini_hopping_skull_air_sprite_after_loc_3ea88(false, 0),
            Some(MiniHoppingSkullAirSprite::Falling),
            "ASM Mini_Hopping_Skull.asm loc_3EAAC selects the falling sprite once `y_vel(a3)` is non-negative"
        );
        assert_eq!(
            mini_hopping_skull_air_sprite_after_loc_3ea88(true, 0),
            None,
            "ASM Mini_Hopping_Skull.asm checks `is_animated(a3)` before the loc_3EAAC sprite branch"
        );
    }

    #[test]
    fn mini_hopping_skull_collision_dispatch_matches_loc_3eab2_compare_order() {
        assert_eq!(
            mini_hopping_skull_collision_dispatch_loc_3eab2(0),
            MiniHoppingSkullCollisionDispatch::AirborneLoop,
            "ASM Mini_Hopping_Skull.asm loc_3EAB2 branches back to loc_3EA88 when `collision_type(a3)` is zero"
        );
        assert_eq!(
            mini_hopping_skull_collision_dispatch_loc_3eab2(MINI_SKULL_COLID_RIGHTWALL),
            MiniHoppingSkullCollisionDispatch::WallBounce,
            "ASM Mini_Hopping_Skull.asm loc_3EAB2 sends right-wall collision to loc_3F1AA"
        );
        assert_eq!(
            mini_hopping_skull_collision_dispatch_loc_3eab2(MINI_SKULL_COLID_LEFTWALL),
            MiniHoppingSkullCollisionDispatch::WallBounce,
            "ASM Mini_Hopping_Skull.asm loc_3EAB2 sends left-wall collision to loc_3F1AA"
        );
        assert_eq!(
            mini_hopping_skull_collision_dispatch_loc_3eab2(MINI_SKULL_COLID_FLOOR),
            MiniHoppingSkullCollisionDispatch::FloorLanding,
            "ASM Mini_Hopping_Skull.asm loc_3EAB2 sends floor collision to loc_3EFEC"
        );
        assert_eq!(
            mini_hopping_skull_collision_dispatch_loc_3eab2(MINI_SKULL_COLID_CEILING),
            MiniHoppingSkullCollisionDispatch::CeilingHit,
            "ASM Mini_Hopping_Skull.asm loc_3EAB2 sends ceiling collision to loc_3F1A2"
        );
        assert_eq!(
            mini_hopping_skull_collision_dispatch_loc_3eab2(MINI_SKULL_COLID_HURT),
            MiniHoppingSkullCollisionDispatch::HurtOrWeapon,
            "ASM Mini_Hopping_Skull.asm loc_3EAB2 sends hurt-or-above collision to loc_3F1B8 before terrain checks"
        );
        assert_eq!(
            mini_hopping_skull_collision_dispatch_loc_3eab2(MINI_SKULL_COLID_SLOPEUP),
            MiniHoppingSkullCollisionDispatch::SlopeOrOtherTerrain,
            "ASM Mini_Hopping_Skull.asm loc_3EAB2 falls through to loc_3EAD8 for slope-up collision"
        );
    }

    #[test]
    fn mini_hopping_skull_terrain_contact_matches_loc_3ead8_flag_clear_and_slope_gate() {
        assert_eq!(
            mini_hopping_skull_terrain_contact_loc_3ead8(MINI_SKULL_COLID_CEILING, 0),
            MiniHoppingSkullTerrainContact {
                is_moved: false,
                is_animated: false,
                has_level_collision: false,
                route: MiniHoppingSkullTerrainContactRoute::NotSlopeUp,
            },
            "ASM Mini_Hopping_Skull.asm loc_3EAD8 clears live movement flags before branching away from non-slope-up terrain"
        );
        assert_eq!(
            mini_hopping_skull_terrain_contact_loc_3ead8(MINI_SKULL_COLID_SLOPEUP, 0),
            MiniHoppingSkullTerrainContact {
                is_moved: false,
                is_animated: false,
                has_level_collision: false,
                route: MiniHoppingSkullTerrainContactRoute::SlopeUpNonPositiveXVelocity,
            },
            "ASM Mini_Hopping_Skull.asm loc_3EAD8/sub_3F366 sends zero or negative `x_vel(a3)` to loc_3EC32"
        );
        assert_eq!(
            mini_hopping_skull_terrain_contact_loc_3ead8(MINI_SKULL_COLID_SLOPEUP, 1),
            MiniHoppingSkullTerrainContact {
                is_moved: false,
                is_animated: false,
                has_level_collision: false,
                route: MiniHoppingSkullTerrainContactRoute::SlopeUpPositiveXVelocity,
            },
            "ASM Mini_Hopping_Skull.asm loc_3EAD8 clears both velocity components after slope-up contact with positive `x_vel(a3)`"
        );
    }

    #[test]
    fn mini_hopping_skull_slopedown_contact_matches_loc_3ed6c_x_velocity_gate() {
        assert_eq!(
            mini_hopping_skull_slopedown_contact_loc_3ed6c(-0x0001_0000),
            MiniHoppingSkullSlopeDownContact {
                x_velocity: 0,
                y_velocity: 0,
                x_direction: 0x00,
                sprite: "LnkTo_unk_C8030",
                dbf_counter: 0x0014,
                wait_frames: 21,
                wait: MiniHoppingSkullSlopeDownWait::Left,
            },
            "ASM Mini_Hopping_Skull.asm loc_3ED6C should clear velocities and enter the left wait when `tst.w x_vel(a3)` is negative"
        );
        assert_eq!(
            mini_hopping_skull_slopedown_contact_loc_3ed6c(0),
            MiniHoppingSkullSlopeDownContact {
                x_velocity: 0,
                y_velocity: 0,
                x_direction: 0xFF,
                sprite: "LnkTo_unk_C8010",
                dbf_counter: 0x0014,
                wait_frames: 21,
                wait: MiniHoppingSkullSlopeDownWait::Right,
            },
            "ASM Mini_Hopping_Skull.asm loc_3ED6C should jump to the mirrored right wait for a zero high word"
        );
        assert_eq!(
            mini_hopping_skull_slopedown_contact_loc_3ed6c(0x0001_0000).wait,
            MiniHoppingSkullSlopeDownWait::Right,
            "ASM Mini_Hopping_Skull.asm loc_3ED6C should jump to the mirrored right wait for positive X velocity"
        );
    }

    #[test]
    fn mini_hopping_skull_left_hop_wait_setup_matches_loc_3ed80() {
        assert_eq!(
            mini_hopping_skull_left_hop_wait_setup_loc_3ed80(),
            MiniHoppingSkullGroundWaitSetup {
                x_velocity: 0,
                y_velocity: 0,
                x_direction: 0x00,
                sprite: MiniHoppingSkullGroundWaitSprite::SlopeUpRight,
                dbf_counter: 0x0014,
                wait_frames: 21,
            },
            "ASM Mini_Hopping_Skull.asm loc_3ED80 should clear both velocities, face left, and seed a 21-frame wait"
        );
    }

    #[test]
    fn mini_hopping_skull_left_hop_windup_collision_matches_loc_3ed8e_compare_order() {
        assert_eq!(
            mini_hopping_skull_left_hop_windup_collision_loc_3ed8e(0, false),
            MiniHoppingSkullGroundWaitCollisionRoute::ContinueWait,
            "ASM Mini_Hopping_Skull.asm loc_3ED8E should continue the DBF loop while `collision_type(a3)` stays zero"
        );
        assert_eq!(
            mini_hopping_skull_left_hop_windup_collision_loc_3ed8e(
                MINI_SKULL_COLID_GROUND_KIDABOVE,
                false
            ),
            MiniHoppingSkullGroundWaitCollisionRoute::DamageSkull,
            "ASM Mini_Hopping_Skull.asm loc_3ED8E sends `colid_kidabove` to the HP decrement branch"
        );
        assert_eq!(
            mini_hopping_skull_left_hop_windup_collision_loc_3ed8e(
                MINI_SKULL_COLID_GROUND_KIDLEFT,
                false
            ),
            MiniHoppingSkullGroundWaitCollisionRoute::QueueScreamWorker,
            "ASM Mini_Hopping_Skull.asm loc_3ED8E sends non-charging side contact to the scare/queue path"
        );
        assert_eq!(
            mini_hopping_skull_left_hop_windup_collision_loc_3ed8e(
                MINI_SKULL_COLID_GROUND_KIDLEFT,
                true
            ),
            MiniHoppingSkullGroundWaitCollisionRoute::BerzerkerChargeKillsSkull,
            "ASM Mini_Hopping_Skull.asm loc_3ED8E jumps to loc_3F262 while `Berzerker_charging` is set"
        );
        assert_eq!(
            mini_hopping_skull_left_hop_windup_collision_loc_3ed8e(
                MINI_SKULL_COLID_GROUND_KIDBELOW,
                false
            ),
            MiniHoppingSkullGroundWaitCollisionRoute::ContinueWait,
            "ASM Mini_Hopping_Skull.asm loc_3ED8E falls through for `colid_kidbelow` contact"
        );
        assert_eq!(
            mini_hopping_skull_left_hop_windup_collision_loc_3ed8e(
                MINI_SKULL_COLID_GROUND_HURT,
                false
            ),
            MiniHoppingSkullGroundWaitCollisionRoute::DamageSkull,
            "ASM Mini_Hopping_Skull.asm loc_3ED8E sends `colid_hurt` to the same HP decrement branch as `colid_kidabove`"
        );
    }

    #[test]
    fn mini_hopping_skull_left_hop_landing_gate_matches_loc_3edf8() {
        assert_eq!(
            mini_hopping_skull_left_hop_landing_gate_loc_3edf8(0x0014, 0, 0),
            MiniHoppingSkullLeftLandingLoopDecision::ContinueWait {
                next_dbf_counter: 0x0013,
            },
            "ASM Mini_Hopping_Skull.asm loc_3EDF8 should consume the DBF counter before the signed X compare"
        );
        assert_eq!(
            mini_hopping_skull_left_hop_landing_gate_loc_3edf8(0x0000, 0x0001, 0x0000),
            MiniHoppingSkullLeftLandingLoopDecision::BeginSkullFacingLeftLanding {
                dbf_counter: 0x0014,
                x_direction: 0xFF,
                sprite: "LnkTo_unk_C8010",
            },
            "ASM Mini_Hopping_Skull.asm loc_3EDF8 jumps to loc_3EEB2 and seeds `$14` when Kid is to the right of the skull"
        );
        assert_eq!(
            mini_hopping_skull_left_hop_landing_gate_loc_3edf8(0x0000, 0x0000, 0x0001),
            MiniHoppingSkullLeftLandingLoopDecision::BeginSkullLeftRelaunch {
                dbf_counter: 0x000A,
                sprite: "LnkTo_unk_C8038",
            },
            "ASM Mini_Hopping_Skull.asm loc_3EDF8 seeds `$A` and sets `LnkTo_unk_C8038` when Kid is not to the right"
        );
        assert_eq!(
            mini_hopping_skull_left_hop_landing_gate_loc_3edf8(0x0000, 0x8000, 0x7FFF),
            MiniHoppingSkullLeftLandingLoopDecision::BeginSkullLeftRelaunch {
                dbf_counter: 0x000A,
                sprite: "LnkTo_unk_C8038",
            },
            "ASM Mini_Hopping_Skull.asm loc_3EDF8 uses signed compares (`cmp.w`) to treat `0x8000` as negative when comparing Kid and skull X"
        );
    }

    #[test]
    fn mini_hopping_skull_landing_collision_route_matches_loc_3ee12() {
        assert_eq!(
            mini_hopping_skull_landing_collision_route_loc_3ee12(0, false,),
            MiniHoppingSkullGroundWaitCollisionRoute::ContinueWait,
            "ASM Mini_Hopping_Skull.asm loc_3EE12 should continue when no collision is reported"
        );
        assert_eq!(
            mini_hopping_skull_landing_collision_route_loc_3ee12(
                MINI_SKULL_COLID_GROUND_KIDABOVE,
                false,
            ),
            MiniHoppingSkullGroundWaitCollisionRoute::DamageSkull,
            "ASM Mini_Hopping_Skull.asm loc_3EE12 routes `colid_kidabove` to the damage path (`loc_3EE64`)"
        );
        assert_eq!(
            mini_hopping_skull_landing_collision_route_loc_3ee12(
                MINI_SKULL_COLID_GROUND_HURT,
                false,
            ),
            MiniHoppingSkullGroundWaitCollisionRoute::DamageSkull,
            "ASM Mini_Hopping_Skull.asm loc_3EE12 routes `colid_hurt` to the same damage path as `loc_3EE64`"
        );
        assert_eq!(
            mini_hopping_skull_landing_collision_route_loc_3ee12(
                MINI_SKULL_COLID_GROUND_KIDLEFT,
                false,
            ),
            MiniHoppingSkullGroundWaitCollisionRoute::QueueScreamWorker,
            "ASM Mini_Hopping_Skull.asm loc_3EE5A falls through to worker allocation when `Berzerker_charging` is clear"
        );
        assert_eq!(
            mini_hopping_skull_landing_collision_route_loc_3ee12(
                MINI_SKULL_COLID_GROUND_KIDRIGHT,
                true,
            ),
            MiniHoppingSkullGroundWaitCollisionRoute::BerzerkerChargeKillsSkull,
            "ASM Mini_Hopping_Skull.asm loc_3EE5A jumps to loc_3F262 when `Berzerker_charging` is set"
        );
        assert_eq!(
            mini_hopping_skull_landing_collision_route_loc_3ee12(
                MINI_SKULL_COLID_GROUND_KIDBELOW,
                false,
            ),
            MiniHoppingSkullGroundWaitCollisionRoute::QueueScreamWorker,
            "ASM Mini_Hopping_Skull.asm loc_3EE44 queues the scream-worker routine on `colid_kidbelow`"
        );
        assert_eq!(
            mini_hopping_skull_landing_collision_route_loc_3ee12(0x0123, false),
            MiniHoppingSkullGroundWaitCollisionRoute::ContinueWait,
            "ASM Mini_Hopping_Skull.asm loc_3EE12 ignores non-Kid collision IDs"
        );
    }

    #[test]
    // ASM Mini_Hopping_Skull.asm loc_3EEC8
    fn mini_hopping_skull_landing_collision_route_matches_loc_3eec8() {
        assert_eq!(
            mini_hopping_skull_landing_collision_route_loc_3eec8(0, false),
            MiniHoppingSkullGroundWaitCollisionRoute::ContinueWait,
            "ASM Mini_Hopping_Skull.asm loc_3EEC8 should continue when `collision_type(a3)` is zero"
        );
        assert_eq!(
            mini_hopping_skull_landing_collision_route_loc_3eec8(
                MINI_SKULL_COLID_GROUND_KIDABOVE,
                false,
            ),
            MiniHoppingSkullGroundWaitCollisionRoute::DamageSkull,
            "ASM Mini_Hopping_Skull.asm loc_3EEC8 should route `colid_kidabove` to the same HP decrement path as `loc_3EE64`"
        );
        assert_eq!(
            mini_hopping_skull_landing_collision_route_loc_3eec8(
                MINI_SKULL_COLID_GROUND_KIDLEFT,
                false,
            ),
            MiniHoppingSkullGroundWaitCollisionRoute::QueueScreamWorker,
            "ASM Mini_Hopping_Skull.asm loc_3EEC8 should send side contact through loc_3EF10"
        );
        assert_eq!(
            mini_hopping_skull_landing_collision_route_loc_3eec8(
                MINI_SKULL_COLID_GROUND_KIDRIGHT,
                true,
            ),
            MiniHoppingSkullGroundWaitCollisionRoute::BerzerkerChargeKillsSkull,
            "ASM Mini_Hopping_Skull.asm loc_3EF10 should jump to loc_3F262 when `Berzerker_charging` is set"
        );
        assert_eq!(
            mini_hopping_skull_landing_collision_route_loc_3eec8(
                MINI_SKULL_COLID_GROUND_KIDBELOW,
                false,
            ),
            MiniHoppingSkullGroundWaitCollisionRoute::QueueScreamWorker,
            "ASM Mini_Hopping_Skull.asm loc_3EEC8 sends `colid_kidbelow` to the contact worker allocation path"
        );
        assert_eq!(
            mini_hopping_skull_landing_collision_route_loc_3eec8(0x0123, false),
            MiniHoppingSkullGroundWaitCollisionRoute::ContinueWait,
            "ASM Mini_Hopping_Skull.asm loc_3EEC8 ignores non-Kid collision IDs"
        );
    }

    #[test]
    // ASM Mini_Hopping_Skull.asm loc_3EF10
    fn mini_hopping_skull_landing_side_contact_matches_loc_3ef10() {
        assert_eq!(
            mini_hopping_skull_landing_side_contact_loc_3ef10(false),
            MiniHoppingSkullGroundWaitCollisionRoute::QueueScreamWorker,
            "ASM Mini_Hopping_Skull.asm loc_3EF10 should jump to loc_3EEFA when `Berzerker_charging` is clear"
        );
        assert_eq!(
            mini_hopping_skull_landing_side_contact_loc_3ef10(true),
            MiniHoppingSkullGroundWaitCollisionRoute::BerzerkerChargeKillsSkull,
            "ASM Mini_Hopping_Skull.asm loc_3EF10 should jump to loc_3F262 while `Berzerker_charging` is set"
        );
    }

    #[test]
    // ASM Mini_Hopping_Skull.asm loc_3EEFA
    fn mini_hopping_skull_landing_scream_worker_allocation_matches_loc_3eefa() {
        assert_eq!(
            mini_hopping_skull_landing_scream_worker_allocation_loc_3eefa(
                MiniHoppingSkullGroundWaitCollisionRoute::QueueScreamWorker
            ),
            Some(MiniHoppingSkullScreamWorkerAllocation {
                allocation_selector: 0xA000,
                routine: "sub_3F3F0",
                delay_frames: 15,
            }),
            "ASM Mini_Hopping_Skull.asm loc_3EEFA should allocate `sub_3F3F0` with selector `$A000`"
        );
        assert_eq!(
            mini_hopping_skull_landing_scream_worker_allocation_loc_3eefa(
                MiniHoppingSkullGroundWaitCollisionRoute::DamageSkull
            ),
            None,
            "ASM Mini_Hopping_Skull.asm loc_3EEFA is skipped when the collision branch decrements HP"
        );
        assert_eq!(
            mini_hopping_skull_landing_scream_worker_allocation_loc_3eefa(
                MiniHoppingSkullGroundWaitCollisionRoute::BerzerkerChargeKillsSkull
            ),
            None,
            "ASM Mini_Hopping_Skull.asm loc_3EEFA is skipped while `Berzerker_charging` takes the kill path"
        );
    }

    #[test]
    // ASM Mini_Hopping_Skull.asm loc_3EF1A
    fn mini_hopping_skull_landing_collision_damage_matches_loc_3ef1a() {
        assert_eq!(
            mini_hopping_skull_landing_collision_damage_loc_3ef1a(1),
            MiniHoppingSkullDamageSequence::SkullDestroyed,
            "ASM Mini_Hopping_Skull.asm loc_3EF1A should branch to death when `$44(a3)` reaches zero"
        );
        assert_eq!(
            mini_hopping_skull_landing_collision_damage_loc_3ef1a(2),
            MiniHoppingSkullDamageSequence::PlayHitAnimation {
                remaining_hits: 1,
                animation: "stru_3E9C2",
                hibernate_until_finished: true,
            },
            "ASM Mini_Hopping_Skull.asm loc_3EF1A should replay `stru_3E9C2` and wait when hits remain"
        );
    }

    #[test]
    // ASM Mini_Hopping_Skull.asm loc_3EF32
    fn mini_hopping_skull_landing_collision_wait_matches_loc_3ef32() {
        assert_eq!(
            mini_hopping_skull_landing_collision_wait_loc_3ef32(0x0014, 0x0010, 0x0020),
            MiniHoppingSkullLeftLandingLoopDecision::ContinueWait {
                next_dbf_counter: 0x0013
            },
            "ASM Mini_Hopping_Skull.asm loc_3EF32 decrements the DBF counter while waiting"
        );
        assert_eq!(
            mini_hopping_skull_landing_collision_wait_loc_3ef32(0x0000, 0x0010, 0x0010),
            MiniHoppingSkullLeftLandingLoopDecision::BeginSkullFacingLeftLanding {
                dbf_counter: 0x0014,
                x_direction: 0x00,
                sprite: "LnkTo_unk_C8030",
            },
            "ASM Mini_Hopping_Skull.asm loc_3EF32 routes Kid-at-or-left-of-skull to the `loc_3ED80` left-facing setup path"
        );
        assert_eq!(
            mini_hopping_skull_landing_collision_wait_loc_3ef32(0x0000, 0x0021, 0x0020),
            MiniHoppingSkullLeftLandingLoopDecision::BeginSkullLeftRelaunch {
                dbf_counter: 0x000A,
                sprite: "LnkTo_unk_C8018",
            },
            "ASM Mini_Hopping_Skull.asm loc_3EF32 routes Kid-right-of-skull into the `LnkTo_unk_C8018` branch before loc_3EF4C"
        );
    }

    #[test]
    // ASM Mini_Hopping_Skull.asm loc_3EF4C
    fn mini_hopping_skull_landing_tail_collision_matches_loc_3ef4c() {
        assert_eq!(
            mini_hopping_skull_landing_tail_collision_loc_3ef4c(
                0x0014,
                MINI_SKULL_COLID_GROUND_KIDABOVE,
                false,
                0x0002,
                false
            ),
            MiniHoppingSkullLandingTailCollisionOutcome::ContinueWait {
                next_dbf_counter: 0x0013
            },
            "ASM Mini_Hopping_Skull.asm loc_3EF4C continues waiting while DBF is active"
        );
        assert_eq!(
            mini_hopping_skull_landing_tail_collision_loc_3ef4c(
                0x0000,
                MINI_SKULL_COLID_GROUND_KIDABOVE,
                false,
                0x0002,
                false
            ),
            MiniHoppingSkullLandingTailCollisionOutcome::DamageSkullThenRelaunch {
                sequence: MiniHoppingSkullDamageSequence::PlayHitAnimation {
                    remaining_hits: 0x0001,
                    animation: "stru_3E9C2",
                    hibernate_until_finished: true,
                },
            },
            "ASM Mini_Hopping_Skull.asm loc_3EF9E decrements HP and falls through to relaunch on non-zero hits"
        );
        assert_eq!(
            mini_hopping_skull_landing_tail_collision_loc_3ef4c(
                0x0000,
                MINI_SKULL_COLID_GROUND_HURT,
                false,
                0x0001,
                false
            ),
            MiniHoppingSkullLandingTailCollisionOutcome::DamageSkullAndDestroy,
            "ASM Mini_Hopping_Skull.asm loc_3EF9E jumps to the death path when HP reaches zero"
        );
        assert_eq!(
            mini_hopping_skull_landing_tail_collision_loc_3ef4c(
                0x0000,
                MINI_SKULL_COLID_GROUND_KIDLEFT,
                false,
                0x0003,
                false
            ),
            MiniHoppingSkullLandingTailCollisionOutcome::SpawnScreamWorkerAndRelaunch,
            "ASM Mini_Hopping_Skull.asm loc_3EF7E queues `sub_3F3F0` on side contact when Berzerker is clear"
        );
        assert_eq!(
            mini_hopping_skull_landing_tail_collision_loc_3ef4c(
                0x0000,
                MINI_SKULL_COLID_GROUND_KIDRIGHT,
                true,
                0x0003,
                false
            ),
            MiniHoppingSkullLandingTailCollisionOutcome::BerzerkerChargeKillsSkull,
            "ASM Mini_Hopping_Skull.asm loc_3EF94 jumps directly to loc_3F262 under `Berzerker_charging`"
        );
        assert_eq!(
            mini_hopping_skull_landing_tail_collision_loc_3ef4c(
                0x0000, 0x0000, false, 0x0003, false
            ),
            MiniHoppingSkullLandingTailCollisionOutcome::BeginRelaunchArc {
                y_velocity: 0xFFFC8000u32 as i32,
                x_velocity: 0x0001_8000u32 as i32,
                is_moved: true,
                has_level_collision: true,
                next_loop: "loc_3EA88",
            },
            "ASM Mini_Hopping_Skull.asm loc_3EFB6 sets base relaunch velocities when DBF expires"
        );
        assert_eq!(
            mini_hopping_skull_landing_tail_collision_loc_3ef4c(
                0x0000, 0x0000, false, 0x0003, true
            ),
            MiniHoppingSkullLandingTailCollisionOutcome::BeginRelaunchArc {
                y_velocity: 0xFFFB8000u32 as i32,
                x_velocity: 0x0001C000u32 as i32,
                is_moved: true,
                has_level_collision: true,
                next_loop: "loc_3EA88",
            },
            "ASM Mini_Hopping_Skull.asm loc_3EFE0 (`tst.w $40(a3)`) adds boosted x/y to the relaunch arc"
        );
        assert_eq!(
            mini_hopping_skull_landing_tail_collision_loc_3ef4c(
                0x0000,
                MINI_SKULL_COLID_GROUND_KIDBELOW,
                false,
                0x0004,
                false
            ),
            MiniHoppingSkullLandingTailCollisionOutcome::SpawnScreamWorkerAndRelaunch,
            "ASM Mini_Hopping_Skull.asm loc_3EF7E also runs for `colid_kidbelow` and falls through to relaunch"
        );
    }

    #[test]
    fn mini_hopping_skull_landing_tail_matches_loc_3ee7c_and_boost() {
        assert_eq!(
            mini_hopping_skull_landing_tail_loc_3ee7c(0x0014, false),
            MiniHoppingSkullLandingLoopDecision::ContinueWait {
                next_dbf_counter: 0x0013,
            },
            "ASM Mini_Hopping_Skull.asm loc_3EE7C should consume the DBF counter before relaunching"
        );
        assert_eq!(
            mini_hopping_skull_landing_tail_loc_3ee7c(0x0000, false),
            MiniHoppingSkullLandingLoopDecision::BeginRelaunchArc {
                y_velocity: 0xFFFC8000u32 as i32,
                x_velocity: 0xFFFE8000u32 as i32,
                is_moved: true,
                has_level_collision: true,
                next_loop: "loc_3EA88",
            },
            "ASM Mini_Hopping_Skull.asm loc_3EE7C seeds base airborne velocities and movement/collision flags"
        );
        assert_eq!(
            mini_hopping_skull_landing_tail_loc_3ee7c(0x0000, true),
            MiniHoppingSkullLandingLoopDecision::BeginRelaunchArc {
                y_velocity: 0xFFFB8000u32 as i32,
                x_velocity: 0xFFFE4000u32 as i32,
                is_moved: true,
                has_level_collision: true,
                next_loop: "loc_3EA88",
            },
            "ASM Mini_Hopping_Skull.asm loc_3EEA6 adds `-$10000` and `-$4000` when `$40(a3)` is non-zero"
        );
    }

    #[test]
    fn mini_hopping_skull_left_hop_scream_worker_allocation_matches_loc_3edc0() {
        assert_eq!(
            mini_hopping_skull_left_hop_scream_worker_allocation_loc_3edc0(),
            MiniHoppingSkullScreamWorkerAllocation {
                allocation_selector: 0xA000,
                routine: "sub_3F3F0",
                delay_frames: 15,
            },
            "ASM Mini_Hopping_Skull.asm loc_3EDC0 should allocate a helper object with selector `$A000` and routine `sub_3F3F0`"
        );
    }

    #[test]
    fn mini_hopping_skull_ground_wait_setup_matches_loc_3eb00() {
        let setup = mini_hopping_skull_ground_wait_setup_loc_3eb00();

        assert_eq!(
            setup,
            MiniHoppingSkullGroundWaitSetup {
                x_velocity: 0,
                y_velocity: 0,
                x_direction: 0xFF,
                sprite: MiniHoppingSkullGroundWaitSprite::SlopeUpRight,
                dbf_counter: 0x0014,
                wait_frames: 21,
            },
            "ASM Mini_Hopping_Skull.asm loc_3EB00 should face right, choose the slope-up wait sprite, and seed the `$14` DBF counter"
        );
    }

    #[test]
    fn mini_hopping_skull_ground_wait_collision_matches_loc_3eb0e_compare_order() {
        assert_eq!(
            mini_hopping_skull_ground_wait_collision_loc_3eb0e(0, false),
            MiniHoppingSkullGroundWaitCollisionRoute::ContinueWait,
            "ASM Mini_Hopping_Skull.asm loc_3EB0E should continue the DBF wait when `collision_type(a3)` stays zero"
        );
        assert_eq!(
            mini_hopping_skull_ground_wait_collision_loc_3eb0e(
                MINI_SKULL_COLID_GROUND_KIDABOVE,
                false
            ),
            MiniHoppingSkullGroundWaitCollisionRoute::DamageSkull,
            "ASM Mini_Hopping_Skull.asm loc_3EB0E sends `colid_kidabove` to the HP decrement branch"
        );
        assert_eq!(
            mini_hopping_skull_ground_wait_collision_loc_3eb0e(MINI_SKULL_COLID_GROUND_HURT, false),
            MiniHoppingSkullGroundWaitCollisionRoute::DamageSkull,
            "ASM Mini_Hopping_Skull.asm loc_3EB0E sends `colid_hurt` to the same HP decrement branch"
        );
        assert_eq!(
            mini_hopping_skull_ground_wait_collision_loc_3eb0e(
                MINI_SKULL_COLID_GROUND_KIDLEFT,
                false
            ),
            MiniHoppingSkullGroundWaitCollisionRoute::QueueScreamWorker,
            "ASM Mini_Hopping_Skull.asm loc_3EB0E treats non-charging side contact as contact damage"
        );
        assert_eq!(
            mini_hopping_skull_ground_wait_collision_loc_3eb0e(
                MINI_SKULL_COLID_GROUND_KIDRIGHT,
                true
            ),
            MiniHoppingSkullGroundWaitCollisionRoute::BerzerkerChargeKillsSkull,
            "ASM Mini_Hopping_Skull.asm loc_3EB0E routes charging Berzerker side contact to loc_3F262"
        );
        assert_eq!(
            mini_hopping_skull_ground_wait_collision_loc_3eb0e(
                MINI_SKULL_COLID_GROUND_KIDBELOW,
                true
            ),
            MiniHoppingSkullGroundWaitCollisionRoute::QueueScreamWorker,
            "ASM Mini_Hopping_Skull.asm loc_3EB0E sends `colid_kidbelow` to contact damage before considering Berzerker charge"
        );
        assert_eq!(
            mini_hopping_skull_ground_wait_collision_loc_3eb0e(MINI_SKULL_COLID_SLOPEUP, false),
            MiniHoppingSkullGroundWaitCollisionRoute::ContinueWait,
            "ASM Mini_Hopping_Skull.asm loc_3EB0E ignores unrecognised collision IDs during the grounded wait loop"
        );
    }

    #[test]
    fn mini_hopping_skull_side_contact_matches_loc_3eb56_berzerker_gate() {
        assert_eq!(
            mini_hopping_skull_side_contact_loc_3eb56(false),
            MiniHoppingSkullGroundWaitCollisionRoute::QueueScreamWorker,
            "ASM Mini_Hopping_Skull.asm loc_3EB56 should branch to loc_3EB40 when `Berzerker_charging` is clear"
        );
        assert_eq!(
            mini_hopping_skull_side_contact_loc_3eb56(true),
            MiniHoppingSkullGroundWaitCollisionRoute::BerzerkerChargeKillsSkull,
            "ASM Mini_Hopping_Skull.asm loc_3EB56 should jump to loc_3F262 while `Berzerker_charging` is set"
        );
    }

    #[test]
    fn mini_hopping_skull_right_hop_side_contact_matches_loc_3ebda_berzerker_gate() {
        assert_eq!(
            mini_hopping_skull_right_hop_side_contact_loc_3ebda(false),
            MiniHoppingSkullGroundWaitCollisionRoute::QueueScreamWorker,
            "ASM Mini_Hopping_Skull.asm loc_3EBDA should fall through to the contact worker when `Berzerker_charging` is clear"
        );
        assert_eq!(
            mini_hopping_skull_right_hop_side_contact_loc_3ebda(true),
            MiniHoppingSkullGroundWaitCollisionRoute::BerzerkerChargeKillsSkull,
            "ASM Mini_Hopping_Skull.asm loc_3EBDA should jump to loc_3F262 while `Berzerker_charging` is set"
        );
    }

    #[test]
    fn mini_hopping_skull_damage_sequence_matches_loc_3eb60_counter_update() {
        assert_eq!(
            mini_hopping_skull_damage_sequence_loc_3eb60(1),
            MiniHoppingSkullDamageSequence::SkullDestroyed,
            "ASM Mini_Hopping_Skull.asm loc_3EB60 should branch to the skull death routine when `$44(a3)` becomes zero after decrement"
        );
        assert_eq!(
            mini_hopping_skull_damage_sequence_loc_3eb60(2),
            MiniHoppingSkullDamageSequence::PlayHitAnimation {
                remaining_hits: 1,
                animation: "stru_3E9C2",
                hibernate_until_finished: true,
            },
            "ASM Mini_Hopping_Skull.asm loc_3EB60 should replay `stru_3E9C2` and hibernate until it finishes when hits remain"
        );
        assert_eq!(
            mini_hopping_skull_damage_sequence_loc_3eb60(0),
            MiniHoppingSkullDamageSequence::PlayHitAnimation {
                remaining_hits: 0xFFFF,
                animation: "stru_3E9C2",
                hibernate_until_finished: true,
            },
            "ASM Mini_Hopping_Skull.asm loc_3EB60 uses word wrapping for the decrement before the zero branch"
        );
    }

    #[test]
    fn mini_hopping_skull_right_hop_damage_sequence_matches_loc_3ebe4_counter_update() {
        assert_eq!(
            mini_hopping_skull_right_hop_damage_sequence_loc_3ebe4(1),
            MiniHoppingSkullDamageSequence::SkullDestroyed,
            "ASM Mini_Hopping_Skull.asm loc_3EBE4 should branch to loc_3F176 when `$44(a3)` reaches zero"
        );
        assert_eq!(
            mini_hopping_skull_right_hop_damage_sequence_loc_3ebe4(2),
            MiniHoppingSkullDamageSequence::PlayHitAnimation {
                remaining_hits: 1,
                animation: "stru_3E9C2",
                hibernate_until_finished: true,
            },
            "ASM Mini_Hopping_Skull.asm loc_3EBE4 should replay `stru_3E9C2` and hibernate until it finishes when hits remain"
        );
        assert_eq!(
            mini_hopping_skull_right_hop_damage_sequence_loc_3ebe4(0),
            MiniHoppingSkullDamageSequence::PlayHitAnimation {
                remaining_hits: 0xFFFF,
                animation: "stru_3E9C2",
                hibernate_until_finished: true,
            },
            "ASM Mini_Hopping_Skull.asm loc_3EBE4 uses word wrapping for the hit-counter decrement"
        );
    }

    #[test]
    fn mini_hopping_skull_left_hop_damage_sequence_matches_loc_3ed1e_counter_update() {
        assert_eq!(
            mini_hopping_skull_left_hop_damage_sequence_loc_3ed1e(1),
            MiniHoppingSkullDamageSequence::SkullDestroyed,
            "ASM Mini_Hopping_Skull.asm loc_3ED1E should branch to the death path when `$44(a3)` reaches zero"
        );
        assert_eq!(
            mini_hopping_skull_left_hop_damage_sequence_loc_3ed1e(2),
            MiniHoppingSkullDamageSequence::PlayHitAnimation {
                remaining_hits: 1,
                animation: "stru_3E9C2",
                hibernate_until_finished: true,
            },
            "ASM Mini_Hopping_Skull.asm loc_3ED1E should replay `stru_3E9C2` and hibernate until it finishes when hits remain"
        );
        assert_eq!(
            mini_hopping_skull_left_hop_damage_sequence_loc_3ed1e(0),
            MiniHoppingSkullDamageSequence::PlayHitAnimation {
                remaining_hits: 0xFFFF,
                animation: "stru_3E9C2",
                hibernate_until_finished: true,
            },
            "ASM Mini_Hopping_Skull.asm loc_3ED1E uses word wrapping for the hit-counter decrement"
        );
    }

    #[test]
    fn mini_hopping_skull_damage_sequence_matches_loc_3ede0() {
        assert_eq!(
            mini_hopping_skull_damage_sequence_loc_3ede0(1),
            MiniHoppingSkullDamageSequence::SkullDestroyed,
            "ASM Mini_Hopping_Skull.asm loc_3EDE0 should jump to loc_3F176 when the decremented hit counter becomes zero"
        );
        assert_eq!(
            mini_hopping_skull_damage_sequence_loc_3ede0(2),
            MiniHoppingSkullDamageSequence::PlayHitAnimation {
                remaining_hits: 1,
                animation: "stru_3E9C2",
                hibernate_until_finished: true,
            },
            "ASM Mini_Hopping_Skull.asm loc_3EDE0 should replay `stru_3E9C2` when hits remain"
        );
        assert_eq!(
            mini_hopping_skull_damage_sequence_loc_3ede0(0),
            MiniHoppingSkullDamageSequence::PlayHitAnimation {
                remaining_hits: 0xFFFF,
                animation: "stru_3E9C2",
                hibernate_until_finished: true,
            },
            "ASM Mini_Hopping_Skull.asm loc_3EDE0 wraps the hit counter with word subtraction"
        );
    }

    #[test]
    fn mini_hopping_skull_left_ground_side_contact_matches_loc_3ec90_berzerker_gate() {
        assert_eq!(
            mini_hopping_skull_left_ground_side_contact_loc_3ec90(false),
            MiniHoppingSkullGroundWaitCollisionRoute::QueueScreamWorker,
            "ASM Mini_Hopping_Skull.asm loc_3EC90 should branch to loc_3EC7A when `Berzerker_charging` is clear"
        );
        assert_eq!(
            mini_hopping_skull_left_ground_side_contact_loc_3ec90(true),
            MiniHoppingSkullGroundWaitCollisionRoute::BerzerkerChargeKillsSkull,
            "ASM Mini_Hopping_Skull.asm loc_3EC90 should jump to loc_3F262 when `Berzerker_charging` is set"
        );
        assert_eq!(
            mini_hopping_skull_left_ground_wait_collision_loc_3ec48(
                MINI_SKULL_COLID_GROUND_KIDLEFT,
                false
            ),
            MiniHoppingSkullGroundWaitCollisionRoute::QueueScreamWorker,
            "ASM Mini_Hopping_Skull.asm loc_3EC48 should route `colid_kidleft` through loc_3EC90"
        );
    }

    #[test]
    fn mini_hopping_skull_ground_wait_tail_matches_loc_3eb78_dbf_and_x_compare() {
        assert_eq!(
            mini_hopping_skull_ground_wait_tail_loc_3eb78(0x0014, 40, 32),
            MiniHoppingSkullGroundWaitTail::ContinueWait {
                next_dbf_counter: 0x0013,
            },
            "ASM Mini_Hopping_Skull.asm loc_3EB78 should loop while `dbf d1` leaves a non-$FFFF counter"
        );
        assert_eq!(
            mini_hopping_skull_ground_wait_tail_loc_3eb78(0x0000, 32, 32),
            MiniHoppingSkullGroundWaitTail::SwitchToLeftGroundWaitSetup,
            "ASM Mini_Hopping_Skull.asm loc_3EB78 should switch grounded setup when Kid X is equal to skull X"
        );
        assert_eq!(
            mini_hopping_skull_ground_wait_tail_loc_3eb78(0x0000, 31, 32),
            MiniHoppingSkullGroundWaitTail::SwitchToLeftGroundWaitSetup,
            "ASM Mini_Hopping_Skull.asm loc_3EB78 should use the signed `ble` path when Kid X is left of the skull"
        );
        assert_eq!(
            mini_hopping_skull_ground_wait_tail_loc_3eb78(0x0000, 33, 32),
            MiniHoppingSkullGroundWaitTail::BeginRightHopWindup {
                sprite: "LnkTo_unk_C8038",
                dbf_counter: 0x000A,
                wait_frames: 11,
            },
            "ASM Mini_Hopping_Skull.asm loc_3EB78 should choose the right-hop windup sprite and `$A` DBF seed when Kid X is greater"
        );
        assert_eq!(
            mini_hopping_skull_ground_wait_tail_loc_3eb78(0x0000, 0x8000, 0x7FFF),
            MiniHoppingSkullGroundWaitTail::SwitchToLeftGroundWaitSetup,
            "ASM Mini_Hopping_Skull.asm loc_3EB78 uses signed word comparison, so `$8000` is less than `$7FFF`"
        );
    }

    #[test]
    fn mini_hopping_skull_left_ground_wait_tail_matches_loc_3ecb2_dbf_and_x_compare() {
        assert_eq!(
            mini_hopping_skull_left_ground_wait_tail_loc_3ecb2(0x0014, 24, 32),
            MiniHoppingSkullGroundWaitTail::ContinueWait {
                next_dbf_counter: 0x0013,
            },
            "ASM Mini_Hopping_Skull.asm loc_3ECB2 should loop while `dbf d1` leaves a non-$FFFF counter"
        );
        assert_eq!(
            mini_hopping_skull_left_ground_wait_tail_loc_3ecb2(0x0000, 33, 32),
            MiniHoppingSkullGroundWaitTail::SwitchToRightGroundWaitSetup,
            "ASM Mini_Hopping_Skull.asm loc_3ECB2 should branch to loc_3EB00 when Kid X is greater than skull X"
        );
        assert_eq!(
            mini_hopping_skull_left_ground_wait_tail_loc_3ecb2(0x0000, 32, 32),
            MiniHoppingSkullGroundWaitTail::BeginLeftHopWindup {
                sprite: "LnkTo_unk_C8018",
                dbf_counter: 0x000A,
                wait_frames: 11,
            },
            "ASM Mini_Hopping_Skull.asm loc_3ECB2 should start the left-hop windup when Kid X is equal to skull X"
        );
        assert_eq!(
            mini_hopping_skull_left_ground_wait_tail_loc_3ecb2(0x0000, 31, 32),
            MiniHoppingSkullGroundWaitTail::BeginLeftHopWindup {
                sprite: "LnkTo_unk_C8018",
                dbf_counter: 0x000A,
                wait_frames: 11,
            },
            "ASM Mini_Hopping_Skull.asm loc_3ECB2 should start the left-hop windup when Kid X is less than skull X"
        );
        assert_eq!(
            mini_hopping_skull_left_ground_wait_tail_loc_3ecb2(0x0000, 0x7FFF, 0x8000),
            MiniHoppingSkullGroundWaitTail::SwitchToRightGroundWaitSetup,
            "ASM Mini_Hopping_Skull.asm loc_3ECB2 uses signed word comparison, so `$7FFF` is greater than `$8000`"
        );
    }

    #[test]
    fn mini_hopping_skull_right_hop_windup_collision_matches_loc_3eb92_compare_order() {
        assert_eq!(
            mini_hopping_skull_right_hop_windup_collision_loc_3eb92(0, false),
            MiniHoppingSkullGroundWaitCollisionRoute::ContinueWait,
            "ASM Mini_Hopping_Skull.asm loc_3EB92 should continue the DBF windup when `collision_type(a3)` stays zero"
        );
        assert_eq!(
            mini_hopping_skull_right_hop_windup_collision_loc_3eb92(
                MINI_SKULL_COLID_GROUND_KIDABOVE,
                false
            ),
            MiniHoppingSkullGroundWaitCollisionRoute::DamageSkull,
            "ASM Mini_Hopping_Skull.asm loc_3EB92 sends `colid_kidabove` to the HP decrement branch"
        );
        assert_eq!(
            mini_hopping_skull_right_hop_windup_collision_loc_3eb92(
                MINI_SKULL_COLID_GROUND_HURT,
                false
            ),
            MiniHoppingSkullGroundWaitCollisionRoute::DamageSkull,
            "ASM Mini_Hopping_Skull.asm loc_3EB92 sends `colid_hurt` to the same HP decrement branch"
        );
        assert_eq!(
            mini_hopping_skull_right_hop_windup_collision_loc_3eb92(
                MINI_SKULL_COLID_GROUND_KIDLEFT,
                false
            ),
            MiniHoppingSkullGroundWaitCollisionRoute::QueueScreamWorker,
            "ASM Mini_Hopping_Skull.asm loc_3EB92 treats non-charging side contact as contact damage"
        );
        assert_eq!(
            mini_hopping_skull_right_hop_windup_collision_loc_3eb92(
                MINI_SKULL_COLID_GROUND_KIDRIGHT,
                true
            ),
            MiniHoppingSkullGroundWaitCollisionRoute::BerzerkerChargeKillsSkull,
            "ASM Mini_Hopping_Skull.asm loc_3EB92 routes charging Berzerker side contact to loc_3F262"
        );
        assert_eq!(
            mini_hopping_skull_right_hop_windup_collision_loc_3eb92(
                MINI_SKULL_COLID_GROUND_KIDBELOW,
                true
            ),
            MiniHoppingSkullGroundWaitCollisionRoute::QueueScreamWorker,
            "ASM Mini_Hopping_Skull.asm loc_3EB92 sends `colid_kidbelow` to contact damage before considering Berzerker charge"
        );
        assert_eq!(
            mini_hopping_skull_right_hop_windup_collision_loc_3eb92(
                MINI_SKULL_COLID_SLOPEUP,
                false
            ),
            MiniHoppingSkullGroundWaitCollisionRoute::ContinueWait,
            "ASM Mini_Hopping_Skull.asm loc_3EB92 ignores unrecognised collision IDs during the right-hop windup loop"
        );
    }

    #[test]
    fn mini_hopping_skull_left_ground_wait_collision_matches_loc_3ec48_compare_order() {
        assert_eq!(
            mini_hopping_skull_left_ground_wait_collision_loc_3ec48(0, false),
            MiniHoppingSkullGroundWaitCollisionRoute::ContinueWait,
            "ASM Mini_Hopping_Skull.asm loc_3EC48 should continue the DBF wait when `collision_type(a3)` stays zero"
        );
        assert_eq!(
            mini_hopping_skull_left_ground_wait_collision_loc_3ec48(
                MINI_SKULL_COLID_GROUND_KIDABOVE,
                false
            ),
            MiniHoppingSkullGroundWaitCollisionRoute::DamageSkull,
            "ASM Mini_Hopping_Skull.asm loc_3EC48 sends `colid_kidabove` to loc_3EC9A's HP decrement branch"
        );
        assert_eq!(
            mini_hopping_skull_left_ground_wait_collision_loc_3ec48(
                MINI_SKULL_COLID_GROUND_HURT,
                false
            ),
            MiniHoppingSkullGroundWaitCollisionRoute::DamageSkull,
            "ASM Mini_Hopping_Skull.asm loc_3EC48 sends `colid_hurt` to the same HP decrement branch"
        );
        assert_eq!(
            mini_hopping_skull_left_ground_wait_collision_loc_3ec48(
                MINI_SKULL_COLID_GROUND_KIDLEFT,
                false
            ),
            MiniHoppingSkullGroundWaitCollisionRoute::QueueScreamWorker,
            "ASM Mini_Hopping_Skull.asm loc_3EC48 treats non-charging side contact as contact damage"
        );
        assert_eq!(
            mini_hopping_skull_left_ground_wait_collision_loc_3ec48(
                MINI_SKULL_COLID_GROUND_KIDRIGHT,
                true
            ),
            MiniHoppingSkullGroundWaitCollisionRoute::BerzerkerChargeKillsSkull,
            "ASM Mini_Hopping_Skull.asm loc_3EC48 routes charging Berzerker side contact to loc_3F262"
        );
        assert_eq!(
            mini_hopping_skull_left_ground_wait_collision_loc_3ec48(
                MINI_SKULL_COLID_GROUND_KIDBELOW,
                true
            ),
            MiniHoppingSkullGroundWaitCollisionRoute::QueueScreamWorker,
            "ASM Mini_Hopping_Skull.asm loc_3EC48 sends `colid_kidbelow` to contact damage before considering Berzerker charge"
        );
        assert_eq!(
            mini_hopping_skull_left_ground_wait_collision_loc_3ec48(
                MINI_SKULL_COLID_SLOPEUP,
                false
            ),
            MiniHoppingSkullGroundWaitCollisionRoute::ContinueWait,
            "ASM Mini_Hopping_Skull.asm loc_3EC48 ignores unrecognised collision IDs during the left grounded wait loop"
        );
    }

    #[test]
    fn mini_hopping_skull_left_hop_side_contact_matches_loc_3ed14_berzerker_gate() {
        assert_eq!(
            mini_hopping_skull_left_hop_side_contact_loc_3ed14(false),
            MiniHoppingSkullGroundWaitCollisionRoute::QueueScreamWorker,
            "ASM Mini_Hopping_Skull.asm loc_3ED14 should fall through to the contact worker when `Berzerker_charging` is clear"
        );
        assert_eq!(
            mini_hopping_skull_left_hop_side_contact_loc_3ed14(true),
            MiniHoppingSkullGroundWaitCollisionRoute::BerzerkerChargeKillsSkull,
            "ASM Mini_Hopping_Skull.asm loc_3ED14 should take the kill path while `Berzerker_charging` is set"
        );
    }

    #[test]
    fn mini_hopping_skull_left_hop_windup_collision_matches_loc_3eccc_compare_order() {
        assert_eq!(
            mini_hopping_skull_left_hop_windup_collision_loc_3eccc(0, false),
            MiniHoppingSkullGroundWaitCollisionRoute::ContinueWait,
            "ASM Mini_Hopping_Skull.asm loc_3ECCC should continue the DBF windup when `collision_type(a3)` stays zero"
        );
        assert_eq!(
            mini_hopping_skull_left_hop_windup_collision_loc_3eccc(
                MINI_SKULL_COLID_GROUND_KIDABOVE,
                false
            ),
            MiniHoppingSkullGroundWaitCollisionRoute::DamageSkull,
            "ASM Mini_Hopping_Skull.asm loc_3ECCC sends `colid_kidabove` to the HP decrement branch"
        );
        assert_eq!(
            mini_hopping_skull_left_hop_windup_collision_loc_3eccc(
                MINI_SKULL_COLID_GROUND_HURT,
                false
            ),
            MiniHoppingSkullGroundWaitCollisionRoute::DamageSkull,
            "ASM Mini_Hopping_Skull.asm loc_3ECCC sends `colid_hurt` to the same HP decrement branch"
        );
        assert_eq!(
            mini_hopping_skull_left_hop_windup_collision_loc_3eccc(
                MINI_SKULL_COLID_GROUND_KIDLEFT,
                false
            ),
            MiniHoppingSkullGroundWaitCollisionRoute::QueueScreamWorker,
            "ASM Mini_Hopping_Skull.asm loc_3ECCC treats non-charging side contact as contact damage"
        );
        assert_eq!(
            mini_hopping_skull_left_hop_windup_collision_loc_3eccc(
                MINI_SKULL_COLID_GROUND_KIDRIGHT,
                true
            ),
            MiniHoppingSkullGroundWaitCollisionRoute::BerzerkerChargeKillsSkull,
            "ASM Mini_Hopping_Skull.asm loc_3ECCC routes charging Berzerker side contact to the skull kill path"
        );
        assert_eq!(
            mini_hopping_skull_left_hop_windup_collision_loc_3eccc(
                MINI_SKULL_COLID_GROUND_KIDBELOW,
                true
            ),
            MiniHoppingSkullGroundWaitCollisionRoute::QueueScreamWorker,
            "ASM Mini_Hopping_Skull.asm loc_3ECCC sends `colid_kidbelow` to contact damage before considering Berzerker charge"
        );
        assert_eq!(
            mini_hopping_skull_left_hop_windup_collision_loc_3eccc(MINI_SKULL_COLID_SLOPEUP, false),
            MiniHoppingSkullGroundWaitCollisionRoute::ContinueWait,
            "ASM Mini_Hopping_Skull.asm loc_3ECCC ignores unrecognised collision IDs during the left-hop windup loop"
        );
    }

    #[test]
    fn mini_hopping_skull_scream_worker_allocation_matches_loc_3eb40() {
        assert_eq!(
            mini_hopping_skull_scream_worker_allocation_loc_3eb40(
                MiniHoppingSkullGroundWaitCollisionRoute::QueueScreamWorker
            ),
            Some(MiniHoppingSkullScreamWorkerAllocation {
                allocation_selector: 0xA000,
                routine: "sub_3F3F0",
                delay_frames: 15,
            }),
            "ASM Mini_Hopping_Skull.asm loc_3EB40 should allocate a helper object for `sub_3F3F0`"
        );
        assert_eq!(
            mini_hopping_skull_scream_worker_allocation_loc_3eb40(
                MiniHoppingSkullGroundWaitCollisionRoute::DamageSkull
            ),
            None,
            "ASM Mini_Hopping_Skull.asm loc_3EB40 is skipped when the collision decrements skull HP"
        );
        assert_eq!(
            mini_hopping_skull_scream_worker_allocation_loc_3eb40(
                MiniHoppingSkullGroundWaitCollisionRoute::BerzerkerChargeKillsSkull
            ),
            None,
            "ASM Mini_Hopping_Skull.asm loc_3EB40 is skipped by the charging-Berzerker side-contact branch"
        );
    }

    #[test]
    fn mini_hopping_skull_right_hop_scream_worker_allocation_matches_loc_3ebc4() {
        assert_eq!(
            mini_hopping_skull_right_hop_scream_worker_allocation_loc_3ebc4(
                MiniHoppingSkullGroundWaitCollisionRoute::QueueScreamWorker
            ),
            Some(MiniHoppingSkullScreamWorkerAllocation {
                allocation_selector: 0xA000,
                routine: "sub_3F3F0",
                delay_frames: 15,
            }),
            "ASM Mini_Hopping_Skull.asm loc_3EBC4 should allocate the `sub_3F3F0` contact worker with selector `$A000`"
        );
        assert_eq!(
            mini_hopping_skull_right_hop_scream_worker_allocation_loc_3ebc4(
                MiniHoppingSkullGroundWaitCollisionRoute::DamageSkull
            ),
            None,
            "ASM Mini_Hopping_Skull.asm loc_3EBC4 is skipped when the windup collision decrements skull HP"
        );
        assert_eq!(
            mini_hopping_skull_right_hop_scream_worker_allocation_loc_3ebc4(
                MiniHoppingSkullGroundWaitCollisionRoute::BerzerkerChargeKillsSkull
            ),
            None,
            "ASM Mini_Hopping_Skull.asm loc_3EBC4 is skipped by the charging-Berzerker side-contact branch"
        );
    }

    #[test]
    fn mini_hopping_skull_left_ground_scream_worker_allocation_matches_loc_3ec7a() {
        assert_eq!(
            mini_hopping_skull_left_ground_scream_worker_allocation_loc_3ec7a(
                MiniHoppingSkullGroundWaitCollisionRoute::QueueScreamWorker
            ),
            Some(MiniHoppingSkullScreamWorkerAllocation {
                allocation_selector: 0xA000,
                routine: "sub_3F3F0",
                delay_frames: 15,
            }),
            "ASM Mini_Hopping_Skull.asm loc_3EC7A should allocate the `sub_3F3F0` contact worker with selector `$A000`"
        );
        assert_eq!(
            mini_hopping_skull_left_ground_scream_worker_allocation_loc_3ec7a(
                MiniHoppingSkullGroundWaitCollisionRoute::DamageSkull
            ),
            None,
            "ASM Mini_Hopping_Skull.asm loc_3EC7A is skipped when the left-grounded collision decrements skull HP"
        );
        assert_eq!(
            mini_hopping_skull_left_ground_scream_worker_allocation_loc_3ec7a(
                MiniHoppingSkullGroundWaitCollisionRoute::BerzerkerChargeKillsSkull
            ),
            None,
            "ASM Mini_Hopping_Skull.asm loc_3EC7A is skipped by the charging-Berzerker side-contact branch"
        );
    }

    #[test]
    fn mini_hopping_skull_left_hop_scream_worker_allocation_matches_loc_3ecfe() {
        assert_eq!(
            mini_hopping_skull_left_hop_scream_worker_allocation_loc_3ecfe(
                MiniHoppingSkullGroundWaitCollisionRoute::QueueScreamWorker
            ),
            Some(MiniHoppingSkullScreamWorkerAllocation {
                allocation_selector: 0xA000,
                routine: "sub_3F3F0",
                delay_frames: 15,
            }),
            "ASM Mini_Hopping_Skull.asm loc_3ECFE should allocate the `sub_3F3F0` contact worker with selector `$A000`"
        );
        assert_eq!(
            mini_hopping_skull_left_hop_scream_worker_allocation_loc_3ecfe(
                MiniHoppingSkullGroundWaitCollisionRoute::DamageSkull
            ),
            None,
            "ASM Mini_Hopping_Skull.asm loc_3ECFE is skipped when the windup collision decrements skull HP"
        );
        assert_eq!(
            mini_hopping_skull_left_hop_scream_worker_allocation_loc_3ecfe(
                MiniHoppingSkullGroundWaitCollisionRoute::BerzerkerChargeKillsSkull
            ),
            None,
            "ASM Mini_Hopping_Skull.asm loc_3ECFE is skipped by the charging-Berzerker side-contact branch"
        );
    }

    #[test]
    fn mini_hopping_skull_stopped_wait_setup_matches_loc_3f000() {
        assert_eq!(
            mini_hopping_skull_stopped_wait_setup_loc_3f000(),
            MiniHoppingSkullStoppedWaitSetup {
                is_moved: false,
                is_animated: false,
                has_level_collision: false,
                x_velocity: 0,
                y_velocity: 0,
                sprite: "LnkTo_unk_C8070",
                dbf_counter: 0x0014,
                wait_frames: 21,
            },
            "ASM Mini_Hopping_Skull.asm loc_3F000 should enter the stopped wait with sprite `LnkTo_unk_C8070` and a `$14` DBF counter"
        );
    }

    #[test]
    fn mini_hopping_skull_stopped_wait_collision_matches_loc_3f00a_compare_order() {
        assert_eq!(
            mini_hopping_skull_stopped_wait_collision_loc_3f00a(0, false),
            MiniHoppingSkullGroundWaitCollisionRoute::ContinueWait,
            "ASM Mini_Hopping_Skull.asm loc_3F00A should continue the DBF wait when `collision_type(a3)` stays zero"
        );
        assert_eq!(
            mini_hopping_skull_stopped_wait_collision_loc_3f00a(
                MINI_SKULL_COLID_GROUND_KIDABOVE,
                false
            ),
            MiniHoppingSkullGroundWaitCollisionRoute::DamageSkull,
            "ASM Mini_Hopping_Skull.asm loc_3F00A sends `colid_kidabove` to the HP decrement branch"
        );
        assert_eq!(
            mini_hopping_skull_stopped_wait_collision_loc_3f00a(
                MINI_SKULL_COLID_GROUND_HURT,
                false
            ),
            MiniHoppingSkullGroundWaitCollisionRoute::DamageSkull,
            "ASM Mini_Hopping_Skull.asm loc_3F00A sends `colid_hurt` to the same HP decrement branch"
        );
        assert_eq!(
            mini_hopping_skull_stopped_wait_collision_loc_3f00a(
                MINI_SKULL_COLID_GROUND_KIDLEFT,
                false
            ),
            MiniHoppingSkullGroundWaitCollisionRoute::QueueScreamWorker,
            "ASM Mini_Hopping_Skull.asm loc_3F00A treats non-charging side contact as contact damage"
        );
        assert_eq!(
            mini_hopping_skull_stopped_wait_collision_loc_3f00a(
                MINI_SKULL_COLID_GROUND_KIDRIGHT,
                true
            ),
            MiniHoppingSkullGroundWaitCollisionRoute::BerzerkerChargeKillsSkull,
            "ASM Mini_Hopping_Skull.asm loc_3F00A routes charging Berzerker side contact to loc_3F262"
        );
        assert_eq!(
            mini_hopping_skull_stopped_wait_collision_loc_3f00a(
                MINI_SKULL_COLID_GROUND_KIDBELOW,
                true
            ),
            MiniHoppingSkullGroundWaitCollisionRoute::QueueScreamWorker,
            "ASM Mini_Hopping_Skull.asm loc_3F00A sends `colid_kidbelow` to contact damage before considering Berzerker charge"
        );
        assert_eq!(
            mini_hopping_skull_stopped_wait_collision_loc_3f00a(MINI_SKULL_COLID_SLOPEUP, false),
            MiniHoppingSkullGroundWaitCollisionRoute::ContinueWait,
            "ASM Mini_Hopping_Skull.asm loc_3F00A ignores unrecognised collision IDs during the stopped wait loop"
        );
    }

    #[test]
    fn mini_hopping_skull_stopped_wait_side_contact_matches_loc_3f052_berzerker_gate() {
        assert_eq!(
            mini_hopping_skull_stopped_wait_side_contact_loc_3f052(false),
            MiniHoppingSkullGroundWaitCollisionRoute::QueueScreamWorker,
            "ASM Mini_Hopping_Skull.asm loc_3F052 should branch to loc_3F03C when `Berzerker_charging` is clear"
        );
        assert_eq!(
            mini_hopping_skull_stopped_wait_side_contact_loc_3f052(true),
            MiniHoppingSkullGroundWaitCollisionRoute::BerzerkerChargeKillsSkull,
            "ASM Mini_Hopping_Skull.asm loc_3F052 should jump to loc_3F262 when `Berzerker_charging` is set"
        );
    }

    #[test]
    fn mini_hopping_skull_stopped_wait_damage_sequence_matches_loc_3f05c() {
        assert_eq!(
            mini_hopping_skull_stopped_wait_damage_sequence_loc_3f05c(1),
            MiniHoppingSkullDamageSequence::SkullDestroyed,
            "ASM Mini_Hopping_Skull.asm loc_3F05C should branch to the death path when `$44(a3)` reaches zero"
        );
        assert_eq!(
            mini_hopping_skull_stopped_wait_damage_sequence_loc_3f05c(2),
            MiniHoppingSkullDamageSequence::PlayHitAnimation {
                remaining_hits: 1,
                animation: "stru_3E9C2",
                hibernate_until_finished: true,
            },
            "ASM Mini_Hopping_Skull.asm loc_3F05C should replay `stru_3E9C2` and hibernate until it finishes when hits remain"
        );
        assert_eq!(
            mini_hopping_skull_stopped_wait_damage_sequence_loc_3f05c(0),
            MiniHoppingSkullDamageSequence::PlayHitAnimation {
                remaining_hits: 0xFFFF,
                animation: "stru_3E9C2",
                hibernate_until_finished: true,
            },
            "ASM Mini_Hopping_Skull.asm loc_3F05C uses word wrapping for the hit-counter decrement"
        );
    }

    #[test]
    fn mini_hopping_skull_stopped_wait_scream_worker_allocation_matches_loc_3f03c() {
        assert_eq!(
            mini_hopping_skull_stopped_wait_scream_worker_allocation_loc_3f03c(
                MiniHoppingSkullGroundWaitCollisionRoute::QueueScreamWorker
            ),
            Some(MiniHoppingSkullScreamWorkerAllocation {
                allocation_selector: 0xA000,
                routine: "sub_3F3F0",
                delay_frames: 15,
            }),
            "ASM Mini_Hopping_Skull.asm loc_3F03C should allocate the delayed `sub_3F3F0` contact worker with selector `$A000`"
        );
        assert_eq!(
            mini_hopping_skull_stopped_wait_scream_worker_allocation_loc_3f03c(
                MiniHoppingSkullGroundWaitCollisionRoute::DamageSkull
            ),
            None,
            "ASM Mini_Hopping_Skull.asm loc_3F03C is skipped when the stopped-wait collision decrements skull HP"
        );
        assert_eq!(
            mini_hopping_skull_stopped_wait_scream_worker_allocation_loc_3f03c(
                MiniHoppingSkullGroundWaitCollisionRoute::BerzerkerChargeKillsSkull
            ),
            None,
            "ASM Mini_Hopping_Skull.asm loc_3F03C is skipped by the charging-Berzerker side-contact branch"
        );
    }

    #[test]
    fn mini_hopping_skull_facing_matched_pause_setup_matches_loc_3f09e() {
        assert_eq!(
            mini_hopping_skull_facing_matched_pause_setup_loc_3f09e(),
            MiniHoppingSkullFacingMatchedPauseSetup {
                sprite: "LnkTo_unk_C8078",
                dbf_counter: 0x000A,
                wait_frames: 11,
            },
            "ASM Mini_Hopping_Skull.asm loc_3F09E should select the matched-facing sprite and seed d1 with `$A`"
        );
    }

    #[test]
    fn mini_hopping_skull_facing_matched_pause_collision_matches_loc_3f0a8() {
        assert_eq!(
            mini_hopping_skull_facing_matched_pause_collision_loc_3f0a8(0, false),
            MiniHoppingSkullGroundWaitCollisionRoute::ContinueWait,
            "ASM Mini_Hopping_Skull.asm loc_3F0A8 should continue when no collision is reported"
        );
        assert_eq!(
            mini_hopping_skull_facing_matched_pause_collision_loc_3f0a8(
                MINI_SKULL_COLID_GROUND_KIDABOVE,
                false
            ),
            MiniHoppingSkullGroundWaitCollisionRoute::DamageSkull,
            "ASM Mini_Hopping_Skull.asm loc_3F0A8 sends `colid_kidabove` to the damage branch"
        );
        assert_eq!(
            mini_hopping_skull_facing_matched_pause_collision_loc_3f0a8(
                MINI_SKULL_COLID_GROUND_HURT,
                false
            ),
            MiniHoppingSkullGroundWaitCollisionRoute::DamageSkull,
            "ASM Mini_Hopping_Skull.asm loc_3F0A8 sends `colid_hurt` to the same damage branch"
        );
        assert_eq!(
            mini_hopping_skull_facing_matched_pause_collision_loc_3f0a8(
                MINI_SKULL_COLID_GROUND_KIDLEFT,
                false
            ),
            MiniHoppingSkullGroundWaitCollisionRoute::QueueScreamWorker,
            "ASM Mini_Hopping_Skull.asm loc_3F0A8 treats non-charging side contact as contact damage"
        );
        assert_eq!(
            mini_hopping_skull_facing_matched_pause_collision_loc_3f0a8(
                MINI_SKULL_COLID_GROUND_KIDRIGHT,
                true
            ),
            MiniHoppingSkullGroundWaitCollisionRoute::BerzerkerChargeKillsSkull,
            "ASM Mini_Hopping_Skull.asm loc_3F0A8 routes charging Berzerker side contact to the skull kill path"
        );
        assert_eq!(
            mini_hopping_skull_facing_matched_pause_collision_loc_3f0a8(
                MINI_SKULL_COLID_GROUND_KIDBELOW,
                true
            ),
            MiniHoppingSkullGroundWaitCollisionRoute::QueueScreamWorker,
            "ASM Mini_Hopping_Skull.asm loc_3F0A8 sends `colid_kidbelow` to contact damage before considering Berzerker charge"
        );
        assert_eq!(
            mini_hopping_skull_facing_matched_pause_collision_loc_3f0a8(
                MINI_SKULL_COLID_SLOPEUP,
                false
            ),
            MiniHoppingSkullGroundWaitCollisionRoute::ContinueWait,
            "ASM Mini_Hopping_Skull.asm loc_3F0A8 ignores unrecognised collision IDs during the pause loop"
        );
    }

    #[test]
    fn mini_hopping_skull_facing_matched_pause_scream_worker_allocation_matches_loc_3f0da() {
        assert_eq!(
            mini_hopping_skull_facing_matched_pause_scream_worker_allocation_loc_3f0da(
                MiniHoppingSkullGroundWaitCollisionRoute::QueueScreamWorker
            ),
            Some(MiniHoppingSkullScreamWorkerAllocation {
                allocation_selector: 0xA000,
                routine: "sub_3F3F0",
                delay_frames: 15,
            }),
            "ASM Mini_Hopping_Skull.asm loc_3F0DA should allocate the delayed `sub_3F3F0` contact worker with selector `$A000`"
        );
        assert_eq!(
            mini_hopping_skull_facing_matched_pause_scream_worker_allocation_loc_3f0da(
                MiniHoppingSkullGroundWaitCollisionRoute::DamageSkull
            ),
            None,
            "ASM Mini_Hopping_Skull.asm loc_3F0DA is skipped when the pause collision decrements skull HP"
        );
        assert_eq!(
            mini_hopping_skull_facing_matched_pause_scream_worker_allocation_loc_3f0da(
                MiniHoppingSkullGroundWaitCollisionRoute::BerzerkerChargeKillsSkull
            ),
            None,
            "ASM Mini_Hopping_Skull.asm loc_3F0DA is skipped by the charging-Berzerker side-contact branch"
        );
    }

    #[test]
    fn mini_hopping_skull_stopped_wait_tail_matches_loc_3f090_direction_gate() {
        assert_eq!(
            mini_hopping_skull_stopped_wait_tail_loc_3f090(0x0014, 40, 32, 0x00),
            MiniHoppingSkullStoppedWaitTail::ContinueWait {
                next_dbf_counter: 0x0013
            },
            "ASM Mini_Hopping_Skull.asm loc_3F090 should not run until the stopped-wait DBF counter falls through"
        );
        assert_eq!(
            mini_hopping_skull_stopped_wait_tail_loc_3f090(0x0000, 40, 32, 0x00),
            MiniHoppingSkullStoppedWaitTail::RestartStoppedWaitFacingRight {
                x_direction: 0xFF,
                dbf_counter: 0x0014,
                wait_frames: 21,
            },
            "ASM Mini_Hopping_Skull.asm loc_3F090 should set `x_direction(a3)` when Kid is right and the skull is not already flipped"
        );
        assert_eq!(
            mini_hopping_skull_stopped_wait_tail_loc_3f090(0x0000, 40, 32, 0xFF),
            MiniHoppingSkullStoppedWaitTail::BeginFacingMatchedPause {
                sprite: "LnkTo_unk_C8078",
                dbf_counter: 0x000A,
                wait_frames: 11,
            },
            "ASM Mini_Hopping_Skull.asm loc_3F090 should continue when Kid is right and `x_direction(a3)` is already non-zero"
        );
        assert_eq!(
            mini_hopping_skull_stopped_wait_tail_loc_3f090(0x0000, 24, 32, 0xFF),
            MiniHoppingSkullStoppedWaitTail::RestartStoppedWaitFacingLeft {
                x_direction: 0x00,
                dbf_counter: 0x0014,
                wait_frames: 21,
            },
            "ASM Mini_Hopping_Skull.asm loc_3F090 is skipped when Kid is left, so the opposite direction branch clears `x_direction(a3)`"
        );
        assert_eq!(
            mini_hopping_skull_stopped_wait_tail_loc_3f090(0x0000, 0x8000, 0x7FFF, 0xFF),
            MiniHoppingSkullStoppedWaitTail::RestartStoppedWaitFacingLeft {
                x_direction: 0x00,
                dbf_counter: 0x0014,
                wait_frames: 21,
            },
            "ASM Mini_Hopping_Skull.asm loc_3F090 inherits the signed `cmp.w`/`bgt` X comparison"
        );
    }

    #[test]
    fn mini_hopping_skull_right_hop_relaunch_tail_matches_loc_3ec26_flags() {
        assert_eq!(
            mini_hopping_skull_right_hop_relaunch_tail_loc_3ec26(),
            MiniHoppingSkullRelaunchTail {
                is_moved: true,
                has_level_collision: true,
                next_loop: "loc_3EA88",
            },
            "ASM Mini_Hopping_Skull.asm loc_3EC26 should re-enable movement and level collision before returning to loc_3EA88"
        );
    }

    #[test]
    fn mini_hopping_skull_left_hop_relaunch_tail_matches_loc_3ed60_flags() {
        assert_eq!(
            mini_hopping_skull_left_hop_relaunch_tail_loc_3ed60(),
            MiniHoppingSkullRelaunchTail {
                is_moved: true,
                has_level_collision: true,
                next_loop: "loc_3EA88",
            },
            "ASM Mini_Hopping_Skull.asm loc_3ED60 should re-enable movement and level collision before returning to loc_3EA88"
        );
    }

    #[test]
    fn mini_hopping_skull_hp_tier_boost_matches_loc_3ebfc() {
        let grid = mini_hopping_skull_test_grid();
        let mut enemy = mini_hopping_skull_test_enemy();
        enemy.hit_points = 2;

        step_hopper(&mut enemy, Some(&grid), None);

        assert_eq!(
            enemy.x_velocity, 0x0001_C000,
            "ASM Mini_Hopping_Skull.asm loc_3EBFC adds `$4000` to the base `$18000` hop speed for levelled skulls"
        );
        assert_eq!(
            enemy.y_velocity, -0x0004_6E00,
            "ASM Mini_Hopping_Skull.asm loc_3EBFC applies the extra `$10000` upward boost before the shared `$1200` gravity tick"
        );
    }

    #[test]
    fn mini_hopping_skull_wall_bounce_preserves_boosted_speed_per_loc_3f1aa() {
        let mut grid = mini_hopping_skull_test_grid();
        grid.set_cell_at(2, 1, terrain::Cell::from_be_bytes(0x60, 0x00));

        let mut enemy = mini_hopping_skull_test_enemy();
        enemy.on_ground = false;
        enemy.facing = Direction::Right;
        enemy.x_pos = 23 << 16;
        enemy.y_pos = 24 << 16;
        enemy.x_velocity = 0x0001_C000;
        enemy.y_velocity = 0;

        step_hopper(&mut enemy, Some(&grid), None);

        assert_eq!(enemy.facing, Direction::Left);
        assert_eq!(
            enemy.x_velocity, -0x0001_C000,
            "ASM Mini_Hopping_Skull.asm loc_3F1AA negates the live X velocity, so wall hits must preserve boosted magnitude"
        );
    }

    #[test]
    fn fireball_wall_bounce_keeps_facing_through_loc_32c60_dispatch() {
        let mut grid = mini_hopping_skull_test_grid();
        grid.set_cell_at(2, 1, terrain::Cell::from_be_bytes(0x60, 0x00));

        let mut enemy = fireball_test_enemy();
        enemy.on_ground = false;
        enemy.facing = Direction::Right;
        enemy.x_pos = 23 << 16;
        enemy.y_pos = 24 << 16;
        enemy.x_velocity = 0x0001_8000;
        enemy.y_velocity = 0;

        step_bouncer(&mut enemy, Some(&grid));

        assert_eq!(
            enemy.facing,
            Direction::Right,
            "ASM Fireball.asm loc_32C60/loc_32C86 dispatches the wall-collision handler, and Enemy19_Fireball_BounceWall only negates x_vel without touching x_direction"
        );
        assert_eq!(enemy.x_velocity, -0x0001_8000);
    }

    #[test]
    fn fireball_main_loop_loc_32c1a_adds_hp_gravity_before_collision_dispatch() {
        let mut hp1 = fireball_test_enemy();
        hp1.y_velocity = 0x0001_8000;
        assert_eq!(
            fireball_main_loop_y_velocity_loc_32c1a(&hp1),
            0x0001_A000,
            "ASM Fireball.asm loc_32C1A should add the HP1 `$2000` gravity d0 into `y_vel(a3)` before `loc_32C60` dispatch"
        );

        let mut hp2 = fireball_test_enemy();
        hp2.hit_points = 2;
        hp2.y_velocity = 0x0001_8000;
        assert_eq!(
            fireball_main_loop_y_velocity_loc_32c1a(&hp2),
            0x0001_B000,
            "ASM Fireball.asm loc_32C1A should use the HP2 `$3000` gravity selected from `unk_32BA6`"
        );
    }

    #[test]
    fn fireball_floor_ceiling_bounce_loc_32ce4_commits_velocity_and_gates_trails() {
        let mut enemy = fireball_test_enemy();
        enemy.x_pos = 0x0123_0000;
        enemy.y_velocity = 0x0003_0000;

        let mut rng = RomRandomState::default();
        let open_slot =
            fireball_floor_ceiling_bounce_loc_32ce4(&enemy, &mut rng, enemy.x_pos >> 16, 4);
        assert_eq!(
            open_slot,
            FireballBounceCommitLoc32CE4 {
                y_velocity: -0x0002_F180,
                spawns_fire_trail: true,
                next_fire_trail_count: 5,
            },
            "ASM Fireball.asm loc_32CE4 should store the rebound d7 into `y_vel(a3)` and allocate `loc_33128` while the trail count is below five"
        );

        let mut rng = RomRandomState::default();
        let full = fireball_floor_ceiling_bounce_loc_32ce4(&enemy, &mut rng, enemy.x_pos >> 16, 5);
        assert_eq!(
            full.next_fire_trail_count, 5,
            "ASM Fireball.asm loc_32CE4 should return without allocation when `Number_of_Fire_Trails` is already five"
        );
        assert!(!full.spawns_fire_trail);
    }

    #[test]
    fn fireball_slope_up_trail_marker_loc_32d7e_accepts_only_slope_up_tiles() {
        assert_eq!(
            fireball_slope_up_trail_marker_loc_32d7e(0x005B, 0x007A, 0x4000),
            Some(FireballSlopeTrailMarkerLoc32D7E {
                x_px: 0x005D,
                y_px: 0x0072,
            }),
            "ASM Fireball.asm loc_32D7E should place the slope-up trail at x+2 and `(y & $FFF0) + $F - (x & $F) - 2`"
        );

        assert_eq!(
            fireball_slope_up_trail_marker_loc_32d7e(0x005B, 0x007A, 0x5000),
            None,
            "ASM Fireball.asm loc_32D7E should return through loc_32D06 unless the sampled tile collision nibble is exactly `$4000`"
        );
    }

    #[test]
    fn sphere_wall_bounce_keeps_facing_per_loc_36148() {
        let mut grid = mini_hopping_skull_test_grid();
        grid.set_cell_at(2, 1, terrain::Cell::from_be_bytes(0x60, 0x00));

        let mut enemy = sphere_test_enemy();
        enemy.on_ground = false;
        enemy.facing = Direction::Right;
        enemy.x_pos = 23 << 16;
        enemy.y_pos = 24 << 16;
        enemy.x_velocity = 0x0001_0000;
        enemy.y_velocity = 0;

        step_bouncer(&mut enemy, Some(&grid));

        assert_eq!(
            enemy.facing,
            Direction::Right,
            "ASM Sphere.asm loc_36148 only negates `x_vel(a3)`, so wall hits must not rewrite x_direction"
        );
        assert_eq!(enemy.x_velocity, -0x0001_0000);
    }

    #[test]
    fn sphere_slope_up_backs_out_motion_per_loc_3614e() {
        let data = vec![0u8; 10 * 10 * 2];
        let mut grid = terrain::BlockGrid::from_level_layout(&data, 10, 10);
        grid.set_cell_at(5, 5, terrain::Cell::from_be_bytes(0x40, 0x00));

        let mut enemy = sphere_test_enemy();
        enemy.facing = Direction::Right;
        enemy.x_pos = (5 * 16 + 8) << 16;
        enemy.y_pos = 85 << 16;
        enemy.x_velocity = 0x0002_0000;
        enemy.y_velocity = 0x0002_0000;

        step_bouncer(&mut enemy, Some(&grid));

        assert_eq!(
            enemy.x_pos,
            (5 * 16 + 8) << 16,
            "ASM Sphere.asm loc_3614E subtracts the pre-collision x motion before swapping velocities"
        );
        assert_eq!(
            enemy.y_pos,
            85 << 16,
            "ASM Sphere.asm loc_3614E also subtracts the pre-collision y motion after gravity is applied"
        );
        assert_eq!(enemy.x_velocity, -(0x0002_0000 + SPHERE_GRAVITY));
        assert_eq!(enemy.y_velocity, -0x0002_0000);
        assert_eq!(enemy.facing, Direction::Right);
    }

    #[test]
    fn sphere_slope_down_backs_out_motion_per_loc_36164() {
        let data = vec![0u8; 10 * 10 * 2];
        let mut grid = terrain::BlockGrid::from_level_layout(&data, 10, 10);
        grid.set_cell_at(5, 5, terrain::Cell::from_be_bytes(0x50, 0x00));

        let mut enemy = sphere_test_enemy();
        enemy.facing = Direction::Right;
        enemy.x_pos = (5 * 16 + 8) << 16;
        enemy.y_pos = 89 << 16;
        enemy.x_velocity = 0x0002_0000;
        enemy.y_velocity = 0x0001_0000;

        step_bouncer(&mut enemy, Some(&grid));

        assert_eq!(
            enemy.x_pos,
            (5 * 16 + 8) << 16,
            "ASM Sphere.asm loc_36164 backs the sphere out by the original x motion before keeping the swapped vector"
        );
        assert_eq!(
            enemy.y_pos,
            89 << 16,
            "ASM Sphere.asm loc_36164 also backs out the gravity-adjusted y step before resuming"
        );
        assert_eq!(enemy.x_velocity, 0x0001_0000 + SPHERE_GRAVITY);
        assert_eq!(enemy.y_velocity, 0x0002_0000);
        assert_eq!(enemy.facing, Direction::Right);
    }

    #[test]
    fn tar_monster_attack_waits_nine_frames_before_firing_per_loc_32310() {
        let mut enemy = tar_monster_test_enemy();
        let mut projectiles = Vec::new();

        for frame in 0..8 {
            step_tar_monster(&mut enemy, &mut projectiles);
            assert!(
                projectiles.is_empty(),
                "ASM Tar_Monster.asm loc_3231C uses `moveq #8,d0` + `dbf`, so frame {} is still part of the 9-frame wind-up",
                frame + 1
            );
        }

        step_tar_monster(&mut enemy, &mut projectiles);

        assert_eq!(projectiles.len(), 1);
    }

    #[test]
    fn tar_monster_attack_runs_twenty_seven_frames_per_loc_3231c() {
        let mut enemy = tar_monster_test_enemy();
        let mut projectiles = Vec::new();

        for _ in 0..27 {
            step_tar_monster(&mut enemy, &mut projectiles);
        }

        assert_eq!(enemy.tar_monster_state, TarMonsterState::Attacking);
        assert_eq!(enemy.wait_timer, 0);

        step_tar_monster(&mut enemy, &mut projectiles);

        assert_eq!(enemy.tar_monster_state, TarMonsterState::Submerging);
    }

    #[test]
    fn ninja_stealth_detection_excludes_positive_0x10_edge_per_loc_3d706() {
        let mut enemy = ninja_test_enemy();
        let player_x = (enemy.x_pos as i64 + ((NINJA_DETECT_X_RANGE as i64) << 16)) as u32;
        let player_y = (enemy.y_pos as i64 - ((NINJA_DETECT_Y_MIN as i64) << 16)) as u32;

        step_ninja(
            &mut enemy,
            None,
            Some(player_x),
            Some(player_y),
            PlayerForm::TheKid,
        );

        assert_eq!(enemy.ninja_state, NinjaState::Stealth);
        assert_eq!(
            enemy.x_velocity, 0,
            "ASM Ninja.asm loc_3D706 only enters the jump branch when kid_x - ninja_x < $10; the +$10 edge must stay in stealth"
        );
    }

    #[test]
    fn ninja_kid_threat_route_loc_3d734_matches_charge_and_red_stealth_gates() {
        assert_eq!(
            ninja_kid_threat_route_loc_3d734(0x50, -5, PlayerForm::TheKid, true),
            NinjaKidThreatRouteLoc3D734::BerzerkerKillPath,
            "ASM Ninja.asm loc_3D734 checks Berzerker_charging inside the inclusive +/-$50 Kid X band before the tighter Red Stealth guard gates"
        );
        assert_eq!(
            ninja_kid_threat_route_loc_3d734(0x51, 0, PlayerForm::TheKid, true),
            NinjaKidThreatRouteLoc3D734::ProjectileThreatCheck,
            "ASM Ninja.asm loc_3D734 should skip the Berzerker kill path when kid_x - ninja_x is greater than +$50"
        );
        assert_eq!(
            ninja_kid_threat_route_loc_3d734(0x30, -4, PlayerForm::RedStealth, false),
            NinjaKidThreatRouteLoc3D734::GuardPose,
            "ASM Ninja.asm loc_3D734 should enter the guard pose at the inclusive +$30 Red Stealth edge when Kid is not more than 4 px above"
        );
        assert_eq!(
            ninja_kid_threat_route_loc_3d734(0x31, -4, PlayerForm::RedStealth, false),
            NinjaKidThreatRouteLoc3D734::ProjectileThreatCheck,
            "ASM Ninja.asm loc_3D734 should continue to the projectile-threat check outside the tighter +/-$30 Red Stealth guard band"
        );
        assert_eq!(
            ninja_kid_threat_route_loc_3d734(0, -5, PlayerForm::RedStealth, false),
            NinjaKidThreatRouteLoc3D734::ProjectileThreatCheck,
            "ASM Ninja.asm loc_3D734 should require kid_y - ninja_y >= -4 before entering the guard pose"
        );
    }

    #[test]
    fn ninja_projectile_threat_route_loc_3d776_matches_maniaxe_half_open_band() {
        assert_eq!(
            ninja_projectile_threat_route_loc_3d776(None, PlayerForm::Maniaxe),
            NinjaProjectileThreatRouteLoc3D776::ContinuePostThreat,
            "ASM Ninja.asm loc_3D776 should ignore the projectile threat gate when Addr_GfxObject_KidProjectile is zero"
        );
        assert_eq!(
            ninja_projectile_threat_route_loc_3d776(Some(0), PlayerForm::RedStealth),
            NinjaProjectileThreatRouteLoc3D776::ContinuePostThreat,
            "ASM Ninja.asm loc_3D776 should require Current_Helmet == Maniaxe before checking projectile X range"
        );
        assert_eq!(
            ninja_projectile_threat_route_loc_3d776(Some(-0x5A), PlayerForm::Maniaxe),
            NinjaProjectileThreatRouteLoc3D776::GuardPose,
            "ASM Ninja.asm loc_3D776 should include the left projectile edge because d4 < -$5A is the only left-side reject"
        );
        assert_eq!(
            ninja_projectile_threat_route_loc_3d776(Some(0x59), PlayerForm::Maniaxe),
            NinjaProjectileThreatRouteLoc3D776::GuardPose,
            "ASM Ninja.asm loc_3D776 should include +$59 because the right compare branches while d4 < +$5A"
        );
        assert_eq!(
            ninja_projectile_threat_route_loc_3d776(Some(0x5A), PlayerForm::Maniaxe),
            NinjaProjectileThreatRouteLoc3D776::ContinuePostThreat,
            "ASM Ninja.asm loc_3D776 should reject +$5A exactly, making the right edge exclusive"
        );
    }

    #[test]
    fn ninja_post_threat_route_loc_3d79c_matches_signed_state_and_guard_persistence() {
        assert_eq!(
            ninja_post_threat_route_loc_3d79c(0, false, 0, PlayerForm::TheKid),
            NinjaPostThreatRouteLoc3D79C::ResumePacing,
            "ASM Ninja.asm loc_3D79C should keep normal pacing when `$4C(a5)` is zero"
        );
        assert_eq!(
            ninja_post_threat_route_loc_3d79c(1, false, 0, PlayerForm::TheKid),
            NinjaPostThreatRouteLoc3D79C::PositiveStateCheck,
            "ASM Ninja.asm loc_3D79C should treat positive `$4C(a5)` as the airborne proximity branch"
        );
        assert_eq!(
            ninja_post_threat_route_loc_3d79c(-1, true, 0x7F, PlayerForm::TheKid),
            NinjaPostThreatRouteLoc3D79C::ResumePacing,
            "ASM Ninja.asm loc_3D79C should keep negative guard state while Addr_GfxObject_KidProjectile is non-zero"
        );
        assert_eq!(
            ninja_post_threat_route_loc_3d79c(-1, false, 0x30, PlayerForm::RedStealth),
            NinjaPostThreatRouteLoc3D79C::ResumePacing,
            "ASM Ninja.asm loc_3D79C should keep Red Stealth guard state at the inclusive +$30 Kid X edge"
        );
        assert_eq!(
            ninja_post_threat_route_loc_3d79c(-1, false, 0x31, PlayerForm::RedStealth),
            NinjaPostThreatRouteLoc3D79C::ResetStealthPacing,
            "ASM Ninja.asm loc_3D79C should reset guard state when Red Stealth moves beyond +$30"
        );
        assert_eq!(
            ninja_post_threat_route_loc_3d79c(-1, false, 0, PlayerForm::TheKid),
            NinjaPostThreatRouteLoc3D79C::ResetStealthPacing,
            "ASM Ninja.asm loc_3D79C should reset negative guard state without a projectile unless Current_Helmet is Red Stealth"
        );
    }

    #[test]
    fn ninja_reset_stealth_pacing_loc_3d7ca_restores_walk_state_and_facing_route() {
        let left = ninja_reset_stealth_pacing_loc_3d7ca(0x0000_8000, false);
        assert_eq!(
            left,
            NinjaResetStealthPacingStateLoc3D7CA {
                threat_state_4c: 0,
                animation_script: "stru_3D4EA",
                is_animated: true,
                walk_speed_override: 0x0000_8000,
                next: NinjaResetStealthPacingRouteLoc3D7CA::Loc3E546,
            },
            "ASM Ninja.asm loc_3D7CA should clear `$4C(a5)`, restore `$5E(a5)` into `$50(a5)`, and send clear x_direction to loc_3E546"
        );

        let right = ninja_reset_stealth_pacing_loc_3d7ca(0x0001_0000, true);
        assert_eq!(
            right,
            NinjaResetStealthPacingStateLoc3D7CA {
                threat_state_4c: 0,
                animation_script: "stru_3D4EA",
                is_animated: true,
                walk_speed_override: 0x0001_0000,
                next: NinjaResetStealthPacingRouteLoc3D7CA::Loc3E392,
            },
            "ASM Ninja.asm loc_3D7CA should send set x_direction to loc_3E392 after restoring the walk state"
        );
    }

    #[test]
    fn ninja_resume_pacing_route_loc_3d7ee_dispatches_by_x_direction() {
        assert_eq!(
            ninja_resume_pacing_route_loc_3d7ee(false),
            NinjaResumePacingRouteLoc3D7EE::Loc3E560,
            "ASM Ninja.asm loc_3D7EE should send clear x_direction to loc_3E560"
        );
        assert_eq!(
            ninja_resume_pacing_route_loc_3d7ee(true),
            NinjaResumePacingRouteLoc3D7EE::Loc3E3AC,
            "ASM Ninja.asm loc_3D7EE should send set x_direction to loc_3E3AC"
        );
    }

    #[test]
    fn ninja_positive_state_route_loc_3d7fa_matches_proximity_box() {
        assert_eq!(
            ninja_positive_state_route_loc_3d7fa(-0x10, -0x60, false),
            NinjaPositiveStateRouteLoc3D7FA::ResumePacing {
                next: NinjaResumePacingRouteLoc3D7EE::Loc3E560,
            },
            "ASM Ninja.asm loc_3D7FA should include the left X edge and top Y edge before branching to loc_3D7EE"
        );
        assert_eq!(
            ninja_positive_state_route_loc_3d7fa(0x10, -1, true),
            NinjaPositiveStateRouteLoc3D7FA::ResumePacing {
                next: NinjaResumePacingRouteLoc3D7EE::Loc3E3AC,
            },
            "ASM Ninja.asm loc_3D7FA should include the +$10 X edge and require kid_y - ninja_y to stay negative"
        );
        assert_eq!(
            ninja_positive_state_route_loc_3d7fa(0, 0, false),
            NinjaPositiveStateRouteLoc3D7FA::ResetStealthPacing {
                next: NinjaResetStealthPacingRouteLoc3D7CA::Loc3E546,
            },
            "ASM Ninja.asm loc_3D7FA should branch to loc_3D7CA once Kid is level with or below the ninja"
        );
        assert_eq!(
            ninja_positive_state_route_loc_3d7fa(0x11, -1, true),
            NinjaPositiveStateRouteLoc3D7FA::ResetStealthPacing {
                next: NinjaResetStealthPacingRouteLoc3D7CA::Loc3E392,
            },
            "ASM Ninja.asm loc_3D7FA should branch to loc_3D7CA outside the inclusive +/-$10 X band"
        );
    }

    #[test]
    fn ninja_guard_pose_loc_3d826_sets_negative_guard_state_and_faces_by_d4() {
        assert_eq!(
            ninja_guard_pose_loc_3d826(-1),
            NinjaThreatPoseState {
                threat_state_4c: -1,
                is_animated: false,
                sprite_link: "LnkTo_unk_C7D8E",
                walk_speed_override: 0,
                x_direction_right: false,
                next: NinjaThreatPoseRoute::Loc3D79C,
            },
            "ASM Ninja.asm loc_3D826 should leave x_direction clear for negative d4 before branching to loc_3D79C"
        );

        assert_eq!(
            ninja_guard_pose_loc_3d826(0),
            NinjaThreatPoseState {
                threat_state_4c: -1,
                is_animated: false,
                sprite_link: "LnkTo_unk_C7D8E",
                walk_speed_override: 0,
                x_direction_right: true,
                next: NinjaThreatPoseRoute::Loc3D79C,
            },
            "ASM Ninja.asm loc_3D826 should set x_direction when d4 is zero or positive"
        );
    }

    #[test]
    fn ninja_jump_threat_pose_loc_3d84e_sets_positive_state_and_jump_sprite() {
        assert_eq!(
            ninja_jump_threat_pose_loc_3d84e(-1),
            NinjaThreatPoseState {
                threat_state_4c: 1,
                is_animated: false,
                sprite_link: "LnkTo_unk_C7D86",
                walk_speed_override: 0,
                x_direction_right: false,
                next: NinjaThreatPoseRoute::Loc3D79C,
            },
            "ASM Ninja.asm loc_3D84E should set `$4C(a5)` to positive 1 and keep x_direction clear for negative d4"
        );

        assert_eq!(
            ninja_jump_threat_pose_loc_3d84e(1),
            NinjaThreatPoseState {
                threat_state_4c: 1,
                is_animated: false,
                sprite_link: "LnkTo_unk_C7D86",
                walk_speed_override: 0,
                x_direction_right: true,
                next: NinjaThreatPoseRoute::Loc3D79C,
            },
            "ASM Ninja.asm loc_3D84E should set x_direction for non-negative d4 before loc_3D79C"
        );
    }

    #[test]
    fn ninja_airborne_collision_dispatch_loc_3d878_clears_and_routes_collision_types() {
        assert_eq!(
            ninja_airborne_collision_dispatch_loc_3d878(0, false, PlayerForm::TheKid, true),
            NinjaAirborneCollisionStateLoc3D878 {
                next_collision_type: 0,
                route: NinjaAirborneCollisionRouteLoc3D878::Resume {
                    next: NinjaCollisionResumeLoc3D638::Loc3E7CA,
                },
            },
            "ASM Ninja.asm loc_3D878 should clear collision_type and route zero through loc_3D89C by x_direction"
        );

        assert_eq!(
            ninja_airborne_collision_dispatch_loc_3d878(-1, true, PlayerForm::TheKid, true),
            NinjaAirborneCollisionStateLoc3D878 {
                next_collision_type: 0,
                route: NinjaAirborneCollisionRouteLoc3D878::To3DB4E,
            },
            "ASM Ninja.asm loc_3D878 should send negative collision IDs to loc_3DB4E"
        );

        assert_eq!(
            ninja_airborne_collision_dispatch_loc_3d878(
                NINJA_COLID_KIDABOVE as i16,
                true,
                PlayerForm::TheKid,
                false
            ),
            NinjaAirborneCollisionStateLoc3D878 {
                next_collision_type: 0,
                route: NinjaAirborneCollisionRouteLoc3D878::To3D8BC,
            },
            "ASM Ninja.asm loc_3D878 should send colid_kidabove directly to loc_3D8BC"
        );

        assert_eq!(
            ninja_airborne_collision_dispatch_loc_3d878(
                NINJA_COLID_KIDRIGHT as i16,
                true,
                PlayerForm::TheKid,
                true
            ),
            NinjaAirborneCollisionStateLoc3D878 {
                next_collision_type: 0,
                route: NinjaAirborneCollisionRouteLoc3D878::Resume {
                    next: NinjaCollisionResumeLoc3D638::Loc3E6AE,
                },
            },
            "ASM Ninja.asm loc_3D8A8 should require Current_Helmet == Berzerker before side contact can reach loc_3D8BC"
        );

        assert_eq!(
            ninja_airborne_collision_dispatch_loc_3d878(
                NINJA_COLID_KIDLEFT as i16,
                false,
                PlayerForm::Berzerker,
                false
            ),
            NinjaAirborneCollisionStateLoc3D878 {
                next_collision_type: 0,
                route: NinjaAirborneCollisionRouteLoc3D878::Resume {
                    next: NinjaCollisionResumeLoc3D638::Loc3E7CA,
                },
            },
            "ASM Ninja.asm loc_3D8A8 should fall back to loc_3D89C when Kid's sprite offset is after LnkTo_unk_ADFC8"
        );

        assert_eq!(
            ninja_airborne_collision_dispatch_loc_3d878(
                NINJA_COLID_KIDLEFT as i16,
                false,
                PlayerForm::Berzerker,
                true
            ),
            NinjaAirborneCollisionStateLoc3D878 {
                next_collision_type: 0,
                route: NinjaAirborneCollisionRouteLoc3D878::To3D8BC,
            },
            "ASM Ninja.asm loc_3D878 should allow Berzerker side contact through to loc_3D8BC when the Kid sprite threshold passes"
        );
    }

    #[test]
    fn ninja_knockdown_start_loc_3d8d6_seeds_landing_loop() {
        assert_eq!(
            ninja_knockdown_start_loc_3d8d6(),
            NinjaKnockdownStartStateLoc3D8D6 {
                y_velocity: 0x0001_8000,
                is_moved: true,
                has_level_collision: true,
                next: NinjaKnockdownStartRouteLoc3D8D6::FallthroughLandingLoop,
            },
            "ASM Ninja.asm loc_3D8D6 should start the HP-depleted knockdown fall before the floor-contact wait"
        );
    }

    #[test]
    fn ninja_knockdown_landing_loop_loc_3d8e6_waits_for_floor_then_snaps_y() {
        assert_eq!(
            ninja_knockdown_landing_loop_loc_3d8e6(0, 0x0000_4000, 0x0001_8000, 0x0031_8000),
            NinjaKnockdownLandingStateLoc3D8E6 {
                next_collision_type: 0,
                x_velocity: 0x0000_4000,
                y_velocity: 0x0001_9800,
                y_position: 0x0031_8000,
                is_moved: true,
                has_level_collision: true,
                hibernates_one_frame: true,
                checked_sub_3e8ba: true,
                route: NinjaKnockdownLandingRouteLoc3D8E6::Loop,
            },
            "ASM Ninja.asm loc_3D8E6 should clear collision_type, add $1800 to y_vel, and loop while collision is not floor"
        );

        assert_eq!(
            ninja_knockdown_landing_loop_loc_3d8e6(
                NINJA_COLID_FLOOR,
                0x0000_4000,
                0x0001_8000,
                0x0031_8000,
            ),
            NinjaKnockdownLandingStateLoc3D8E6 {
                next_collision_type: 0,
                x_velocity: 0,
                y_velocity: 0,
                y_position: 0x0040_0000,
                is_moved: false,
                has_level_collision: false,
                hibernates_one_frame: true,
                checked_sub_3e8ba: true,
                route: NinjaKnockdownLandingRouteLoc3D8E6::KillPath,
            },
            "ASM Ninja.asm loc_3D8E6 should clear motion/flags and snap y_pos to ((integer_y + $F) & $FFF0) on floor contact"
        );
    }

    #[test]
    fn ninja_far_kid_facing_loc_3d92a_only_turns_outside_inclusive_band() {
        assert_eq!(
            ninja_far_kid_facing_loc_3d92a(0x80, false),
            NinjaFarKidFacingStateLoc3D92A {
                x_direction_right: false,
                decision: NinjaFarKidFacingDecisionLoc3D92A::WithinBand,
            },
            "ASM Ninja.asm loc_3D92A should keep current facing at the inclusive +$80 Kid X edge"
        );
        assert_eq!(
            ninja_far_kid_facing_loc_3d92a(-0x80, true),
            NinjaFarKidFacingStateLoc3D92A {
                x_direction_right: true,
                decision: NinjaFarKidFacingDecisionLoc3D92A::WithinBand,
            },
            "ASM Ninja.asm loc_3D92A should keep current facing at the inclusive -$80 Kid X edge"
        );
        assert_eq!(
            ninja_far_kid_facing_loc_3d92a(0x81, false),
            NinjaFarKidFacingStateLoc3D92A {
                x_direction_right: true,
                decision: NinjaFarKidFacingDecisionLoc3D92A::TurnTowardFarKid,
            },
            "ASM Ninja.asm loc_3D92A should set x_direction when Kid is more than $80 px to the right"
        );
        assert_eq!(
            ninja_far_kid_facing_loc_3d92a(-0x81, false),
            NinjaFarKidFacingStateLoc3D92A {
                x_direction_right: false,
                decision: NinjaFarKidFacingDecisionLoc3D92A::AlreadyFacingFarKid,
            },
            "ASM Ninja.asm loc_3D92A should leave clear x_direction unchanged when Kid is already far left"
        );
    }

    #[test]
    fn ninja_collision_dispatch_loc_3d606_clears_and_routes_collision_types() {
        assert_eq!(
            ninja_collision_dispatch_loc_3d606(0, false, false),
            NinjaCollisionStateLoc3D606 {
                next_collision_type: 0,
                route: NinjaCollisionLoc3D606::To3D6DA,
            },
            "ASM Ninja.asm loc_3D606 should clear collision_type and jump to loc_3D6DA when d4 is zero"
        );

        assert_eq!(
            ninja_collision_dispatch_loc_3d606(-1, false, false),
            NinjaCollisionStateLoc3D606 {
                next_collision_type: 0,
                route: NinjaCollisionLoc3D606::To3DB4E,
            },
            "ASM Ninja.asm loc_3D606 should route negative collision IDs to loc_3DB4E via bmi"
        );

        assert_eq!(
            ninja_collision_dispatch_loc_3d606(NINJA_COLID_HURT as i16, true, false),
            NinjaCollisionStateLoc3D606 {
                next_collision_type: 0,
                route: NinjaCollisionLoc3D606::To3D65A,
            },
            "ASM Ninja.asm loc_3D606 should route colid_hurt to loc_3D65A after clearing collision_type"
        );

        assert_eq!(
            ninja_collision_dispatch_loc_3d606(NINJA_COLID_KIDABOVE as i16, false, true),
            NinjaCollisionStateLoc3D606 {
                next_collision_type: 0,
                route: NinjaCollisionLoc3D606::To3D65A,
            },
            "ASM Ninja.asm loc_3D606 should route colid_kidabove to loc_3D65A after clearing collision_type"
        );

        assert_eq!(
            ninja_collision_dispatch_loc_3d606(NINJA_COLID_KIDRIGHT as i16, false, true),
            NinjaCollisionStateLoc3D606 {
                next_collision_type: 0,
                route: NinjaCollisionLoc3D606::To3D698,
            },
            "ASM Ninja.asm loc_3D606 should route colid_kidright to loc_3D698"
        );

        assert_eq!(
            ninja_collision_dispatch_loc_3d606(NINJA_COLID_KIDLEFT as i16, true, false),
            NinjaCollisionStateLoc3D606 {
                next_collision_type: 0,
                route: NinjaCollisionLoc3D606::To3D698,
            },
            "ASM Ninja.asm loc_3D606 should route colid_kidleft to loc_3D698"
        );

        assert_eq!(
            ninja_collision_dispatch_loc_3d606(0x00FF, true, true),
            NinjaCollisionStateLoc3D606 {
                next_collision_type: 0,
                route: NinjaCollisionLoc3D606::To3D638 {
                    next: NinjaCollisionResumeLoc3D638::Loc3E3AC
                },
            },
            "ASM Ninja.asm loc_3D606 should fall through to loc_3D638 when collision_id is unhandled"
        );
    }

    #[test]
    fn ninja_collision_resume_loc_3d638_dispatches_wallpin_and_facing() {
        assert_eq!(
            ninja_collision_resume_loc_3d638(false, false),
            NinjaCollisionResumeLoc3D638::Loc3E7CA,
            "ASM Ninja.asm loc_3D638 should jump to loc_3E7CA for left-facing when `$5C(a5)` is clear"
        );
        assert_eq!(
            ninja_collision_resume_loc_3d638(false, true),
            NinjaCollisionResumeLoc3D638::Loc3E6AE,
            "ASM Ninja.asm loc_3D638 should jump to loc_3E6AE for right-facing when `$5C(a5)` is clear"
        );
        assert_eq!(
            ninja_collision_resume_loc_3d638(true, false),
            NinjaCollisionResumeLoc3D638::Loc3E560,
            "ASM Ninja.asm loc_3D638 should jump to loc_3E560 for left-facing when `$5C(a5)` is set"
        );
        assert_eq!(
            ninja_collision_resume_loc_3d638(true, true),
            NinjaCollisionResumeLoc3D638::Loc3E3AC,
            "ASM Ninja.asm loc_3D638 should jump to loc_3E3AC for right-facing when `$5C(a5)` is set"
        );
    }

    #[test]
    fn ninja_collision_resume_loc_3d64e_removes_wallpin_gate_from_loc_3d638() {
        assert_eq!(
            ninja_collision_resume_loc_3d64e(false),
            NinjaCollisionResumeLoc3D638::Loc3E560,
            "ASM Ninja.asm loc_3D64E should branch to loc_3E560 with x_direction cleared"
        );
        assert_eq!(
            ninja_collision_resume_loc_3d64e(true),
            NinjaCollisionResumeLoc3D638::Loc3E3AC,
            "ASM Ninja.asm loc_3D64E should branch to loc_3E3AC when x_direction indicates right-facing"
        );
    }

    #[test]
    fn ninja_collision_loc_3d66e_is_a_fallthrough_gate_to_loc_3d638() {
        assert_eq!(
            ninja_collision_resume_loc_3d66e(NinjaCollisionResumeLoc3D638::Loc3E7CA),
            NinjaCollisionStateLoc3D66E {
                next: NinjaCollisionResumeLoc3D638::Loc3E7CA,
            },
            "ASM Ninja.asm loc_3D66E should pass the collision-resume route through unchanged"
        );
        assert_eq!(
            ninja_collision_resume_loc_3d66e(NinjaCollisionResumeLoc3D638::Loc3E6AE),
            NinjaCollisionStateLoc3D66E {
                next: NinjaCollisionResumeLoc3D638::Loc3E6AE,
            },
            "ASM Ninja.asm loc_3D66E should always behave as an unconditional branch to loc_3D638"
        );
    }

    #[test]
    fn ninja_collision_loc_3d670_handles_block_timer_and_animation_swap() {
        assert_eq!(
            ninja_collision_block_animation_loc_3d670(1, false, false, "test_restore_animation"),
            NinjaCollisionStateLoc3D670 {
                next_collision_type: 0,
                remaining_block_timer: 0,
                route: NinjaCollisionLoc3D670Route::To3DB4E,
                uses_blocking_animation: false,
                blocking_animation: None,
                restored_animation: None,
                hibernates_until_animation_finished: false,
            },
            "ASM Ninja.asm loc_3D670 should jump to loc_3DB4E when `$44(a5)` becomes zero after decrement"
        );
        assert_eq!(
            ninja_collision_block_animation_loc_3d670(5, false, false, "saved_animation"),
            NinjaCollisionStateLoc3D670 {
                next_collision_type: 0,
                remaining_block_timer: 4,
                route: NinjaCollisionLoc3D670Route::To3D638 {
                    next: NinjaCollisionResumeLoc3D638::Loc3E7CA,
                },
                uses_blocking_animation: true,
                blocking_animation: Some("stru_3D504"),
                restored_animation: Some("saved_animation"),
                hibernates_until_animation_finished: true,
            },
            "ASM Ninja.asm loc_3D670 should decrement `$44(a5)`, run blocking animation + Hibernate_UntilAnimFinished, then restore `$62(a5)` into `$50(a5)` before loc_3D638"
        );
        assert_eq!(
            ninja_collision_block_animation_loc_3d670(2, true, true, "other_animation"),
            NinjaCollisionStateLoc3D670 {
                next_collision_type: 0,
                remaining_block_timer: 1,
                route: NinjaCollisionLoc3D670Route::To3D638 {
                    next: NinjaCollisionResumeLoc3D638::Loc3E3AC,
                },
                uses_blocking_animation: true,
                blocking_animation: Some("stru_3D504"),
                restored_animation: Some("other_animation"),
                hibernates_until_animation_finished: true,
            },
            "ASM Ninja.asm loc_3D670 should branch to loc_3D638 with wallpin/facing resume preserved after the blocking animation gate"
        );
    }

    #[test]
    fn ninja_collision_loc_3d6c8_loops_after_sub_3e8ba_and_y_vel_bump() {
        assert_eq!(
            ninja_collision_stun_loop_loc_3d6c8(-0x0004_0000),
            NinjaCollisionStateLoc3D6C8 {
                y_velocity: -0x0003_C000,
                hibernates_one_frame: true,
                checked_sub_3e8ba: true,
                loops: true,
            },
            "ASM Ninja.asm loc_3D6C8 should hibernate, run `sub_3E8BA`, add `$4000` to y_vel(a3), then loop"
        );
        assert_eq!(
            ninja_collision_stun_loop_loc_3d6c8(-0x3FFF_0000),
            NinjaCollisionStateLoc3D6C8 {
                y_velocity: (-0x3FFF_0000_i32).wrapping_add(0x4000),
                hibernates_one_frame: true,
                checked_sub_3e8ba: true,
                loops: true,
            },
            "ASM Ninja.asm loc_3D6C8 increments y_vel(a3) each frame by `$4000` regardless of starting sign"
        );
    }
}

pub(crate) fn spawn_y_velocity(family: EnemyFamily, enemy_level: u8) -> i32 {
    match family {
        // Big_Hopping_Skull.asm: move.w #$FFFE,y_vel → $FFFE0000 FP16.16
        EnemyFamily::BigHoppingSkull => BIG_SKULL_INIT_Y_SPEED,
        EnemyFamily::MiniHoppingSkull => {
            // ASM Mini_Hopping_Skull.asm loc_3EA48 seeds y_vel with `$28000`;
            // non-zero enemy_level at `$40(a3)` adds `$1000`.
            MINI_SKULL_INITIAL_Y_SPEED_FP
                + if enemy_level != 0 {
                    MINI_SKULL_LEVEL_INITIAL_Y_SPEED_BONUS_FP
                } else {
                    0
                }
        }
        _ => 0,
    }
}

pub(crate) fn spawn_x_acceleration(family: EnemyFamily) -> i32 {
    match family {
        EnemyFamily::UFO => UFO_ACCELERATION,
        EnemyFamily::Tornado => TORNADO_ACCELERATION,
        _ => 0,
    }
}

pub(crate) fn spawn_y_acceleration(family: EnemyFamily) -> i32 {
    match family {
        EnemyFamily::UFO => -UFO_ACCELERATION,
        EnemyFamily::Tornado => -TORNADO_ACCELERATION,
        _ => 0,
    }
}

/// Compute initial HP from the enemy family and the raw `enemy_level` field
/// in the layout. Each family's ASM init adds a base offset to `enemy_level`
/// to produce the actual HP value.
pub(crate) fn spawn_hit_points(family: EnemyFamily, enemy_level: u8) -> u8 {
    let base = match family {
        EnemyFamily::Goat => 3,
        EnemyFamily::Armadillo => 1,
        EnemyFamily::Crab => 2,
        EnemyFamily::Tank | EnemyFamily::TankShooting => 2,
        EnemyFamily::Scorpion => 3,
        EnemyFamily::Driller => 2,
        EnemyFamily::Cloud => 1,
        EnemyFamily::Sphere => 1,
        EnemyFamily::BigHoppingSkull => 2,
        EnemyFamily::EmoRock => 1,
        EnemyFamily::Archer => 1,
        EnemyFamily::Fireball => 1,
        // Fire_Demon.asm loc_3A3D2: copy enemy_level(a3), add 3, store
        // current_hp in `$44(a3)`.
        EnemyFamily::FireDemon => 3,
        EnemyFamily::Robot => 2, // HP 2-3
        EnemyFamily::TarMonster => tar_monster_hit_points_loc_321c6(enemy_level),
        EnemyFamily::Orca => 4, // fixed 4 HP
        EnemyFamily::UFO => 1,
        // Ninja: HP is `enemy_level + 2` and uses loc_3D59A/loc_3D5AA for speed tiers.
        EnemyFamily::Ninja => 2,
        EnemyFamily::Lion => 6, // HP 6-8
        EnemyFamily::SpinningTwins => 1,
        EnemyFamily::Drips => 1,
        EnemyFamily::Hand => 1,
        EnemyFamily::Tornado => 1,
        EnemyFamily::DragonWalking => 1, // HP 1-3 (ASM Dragon.asm:65 addq.w #1)
        EnemyFamily::HeadyMetal => 37,   // ASM: $25 = 37 hits
        // Multi-head bosses: HP is set per-head in activate_enemies via boss_head_config.
        // These values are fallback defaults (head 1 HP).
        EnemyFamily::Shishkaboss => 20,
        EnemyFamily::BoomerangBosses => 30,
        EnemyFamily::BagelBrothers => 30,
        EnemyFamily::Diamond => 2, // Diamond.asm loc_3AD16: `addq.w #2,d7`
        // Mini Hopping Skull still uses raw level as HP with a minimum of 1.
        _ => 0,
    };
    (enemy_level + base).max(1)
}

pub(crate) fn emo_rock_wait_timer_with_rng(rng: &mut RomRandomState, player_x: u32) -> u16 {
    // ASM Emo_Rock.asm loc_361F4: seed d0 with `$3C`, XOR the RNG byte with
    // Kid's X low byte, clear bit 7, then add the masked offset.
    let random_byte = rng.next_byte();
    let kid_x_low_byte = ((player_x >> 16) & 0xFF) as u8;
    u16::from((random_byte ^ kid_x_low_byte) & 0x7F) + 0x003C
}

pub(crate) fn spawn_wait_timer(
    family: EnemyFamily,
    enemy_level: u8,
    player_x: u32,
    rng: &mut RomRandomState,
) -> u16 {
    match family {
        EnemyFamily::EmoRock => emo_rock_wait_timer_with_rng(rng, player_x),
        // ASM Tank_Shooting.asm `stru_3B4D8`/`stru_3B4F6` reaches the
        // `loc_3C50E` fire gate 74 frames after init. Plain Tank.asm never fires.
        EnemyFamily::Tank => 0,
        EnemyFamily::TankShooting => TANK_SHOOTING_FIRST_SHOT_DELAY_FRAMES.saturating_sub(1),
        // Archer: fire first arrow after one full interval (HP1 rate).
        EnemyFamily::Archer => ARCHER_FIRE_TIMER[0],
        // Lion: charge timer counts down from $3C (60 frames) before first charge.
        EnemyFamily::Lion => LION_CHARGE_TIMER_INIT,
        // Robot: ASM init $54(a5) low word = 1 (fires on 2nd frame).
        EnemyFamily::Robot => 1,
        // TarMonster: starts submerged, waits before first emergence.
        EnemyFamily::TarMonster => TAR_MONSTER_SUBMERGED_FRAMES,
        // ASM Drips.asm loc_3340E seeds the dormant delay from RNG and Kid X.
        EnemyFamily::Drips => drips_wait_timer_with_rng_loc_3340e(rng, player_x),
        // DragonWalking: first breath after cooldown.
        EnemyFamily::DragonWalking => DRAGON_WALKING_BREATH_INITIAL,
        // Armadillo.asm loc_3C7F8 seeds `$54(a5)` with `$12C` before entering
        // the shared grounded walker loop.
        EnemyFamily::Armadillo => 0x012C,
        // ASM UFO.asm `unk_3588C`: 60/40/30 by raw layout level.
        EnemyFamily::Tornado => TORNADO_PATTERN_FRAMES[usize::from(enemy_level).min(2)],
        // SpinningTwins: orbit-to-walk is collision-triggered, no timer needed.
        EnemyFamily::SpinningTwins => 0,
        // Bosses: fire first projectile after initial delay.
        EnemyFamily::HeadyMetal
        | EnemyFamily::Shishkaboss
        | EnemyFamily::BoomerangBosses
        | EnemyFamily::BagelBrothers => BOSS_FIRE_INTERVAL,
        _ => 0,
    }
}

pub(crate) fn spawn_dragon_phase(family: EnemyFamily) -> DragonFlyingState {
    match family {
        EnemyFamily::Dragon => DragonFlyingState::Hovering,
        _ => DragonFlyingState::None,
    }
}

pub(crate) fn horizontal_velocity(facing: Direction, speed: i32) -> i32 {
    match facing {
        Direction::Left => -speed,
        Direction::Right => speed,
    }
}

pub(crate) fn enemy_layout_view(
    layout: &EnemyLayout,
    camera_x_px: u32,
    camera_y_px: u32,
) -> EnemyLayoutView<'_> {
    if layout.segments.is_empty() {
        return EnemyLayoutView {
            segment_index: 0,
            enemy_type_slots: &layout.enemy_type_slots,
            objects: &layout.objects,
        };
    }

    let position = if layout.axis_uses_y_position {
        camera_y_px.min(u32::from(u16::MAX)) as u16
    } else {
        camera_x_px.min(u32::from(u16::MAX)) as u16
    };
    let mut selected_index = 0usize;
    for (index, segment) in layout.segments.iter().enumerate() {
        if position >= segment.lower_bound {
            selected_index = index;
        }
        if position < segment.upper_bound {
            break;
        }
    }
    let segment = &layout.segments[selected_index];
    EnemyLayoutView {
        segment_index: selected_index,
        enemy_type_slots: &segment.enemy_type_slots,
        objects: &segment.objects,
    }
}

pub(crate) fn resolved_enemy_id(enemy_type_slots: &[u16], object_type: u8) -> Option<u8> {
    if object_type & 0x80 == 0 {
        return Some(object_type);
    }
    let slot_index = usize::from(object_type & 0x7F);
    let raw_slot = *enemy_type_slots.get(slot_index)?;
    if raw_slot == 0xFFFF {
        return None;
    }
    u8::try_from(raw_slot & 0x3FFF).ok()
}

/// Extract the palette variant (0, 1, or 2) from the enemy type slot.
pub(crate) fn resolved_enemy_palette(enemy_type_slots: &[u16], object_type: u8) -> u8 {
    if object_type & 0x80 == 0 {
        return 0;
    }
    let slot_index = usize::from(object_type & 0x7F);
    let Some(&raw_slot) = enemy_type_slots.get(slot_index) else {
        return 0;
    };
    ((raw_slot >> 14) & 3) as u8
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct EnemyArtPaletteEntryLoc36D16 {
    pub(crate) enemy_id: u16,
    pub(crate) palette_variant: u8,
    pub(crate) enemy_load_index_offset: u16,
}

#[allow(non_snake_case)]
#[allow(dead_code)]
pub(crate) fn enemy_art_palette_entry_loc_36D16(
    raw_slot: u16,
) -> Option<EnemyArtPaletteEntryLoc36D16> {
    // ASM loc_36D16 (kid.asm:38603): consume one enemy-layout header word,
    // treat `$FFFF` as a blank art slot, mask the low 12 bits for
    // `EnemyLoad_Index`, and rotate the top two bits down for the palette
    // variant before choosing the palette pointer.
    if raw_slot == 0xFFFF {
        return None;
    }

    let enemy_id = raw_slot & 0x0FFF;
    Some(EnemyArtPaletteEntryLoc36D16 {
        enemy_id,
        palette_variant: u8::try_from(raw_slot.rotate_left(2) & 0x0003)
            .expect("palette variant is masked to two bits"),
        enemy_load_index_offset: enemy_id << 3,
    })
}

#[allow(non_snake_case)]
#[allow(dead_code)]
pub(crate) fn enemy_palette_variant_addend_loc_36D4E(palette_variant: u8) -> u16 {
    // ASM loc_36D4E (kid.asm:38623): variant 1 branches directly here for a
    // single `addi.w #4,d2`; variants 2 and 3 run the preceding add first,
    // then fall through here for the second `+4`.
    match palette_variant & 0x03 {
        0 => 0,
        1 => 4,
        _ => 8,
    }
}

#[allow(non_snake_case)]
#[allow(dead_code)]
pub(crate) fn enemy_palette_data_index_offset_loc_36D52(
    enemy_load_palette_offset: u16,
    palette_variant: u8,
) -> u16 {
    // ASM loc_36D52 (kid.asm:38626): after the variant-specific `d2` addend,
    // `move.l (Data_Index,d2.w),a0` dereferences the chosen palette pointer.
    enemy_load_palette_offset.wrapping_add(enemy_palette_variant_addend_loc_36D4E(palette_variant))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum EnemyArtDecompressRouteLoc36E24 {
    ForestRam,
    ForestTarMonsterRam,
    Vram,
    SpecialVram,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct EnemyArtLoadTailLoc36E3E {
    pub(crate) next_slot_index: u16,
    pub(crate) finished: bool,
}

#[allow(non_snake_case)]
#[allow(dead_code)]
pub(crate) fn enemy_art_decompress_route_loc_36E24(
    slot_index: u16,
    background_is_forest: bool,
    family: EnemyFamily,
    use_special_vram_palette: bool,
) -> EnemyArtDecompressRouteLoc36E24 {
    // ASM loc_36E24 (kid.asm:38718): only the first enemy slot on Forest
    // backgrounds reaches `j_DecompressToRAM`; TarMonster swaps `a3` to the
    // alternate forest permutation immediately before this shared call.
    if slot_index == 0 && background_is_forest {
        if matches!(family, EnemyFamily::TarMonster) {
            EnemyArtDecompressRouteLoc36E24::ForestTarMonsterRam
        } else {
            EnemyArtDecompressRouteLoc36E24::ForestRam
        }
    } else if use_special_vram_palette {
        EnemyArtDecompressRouteLoc36E24::SpecialVram
    } else {
        // ASM loc_36E38 (kid.asm:38730): when neither the Forest RAM path nor
        // the special VRAM decompressor applies, call `j_DecompressToVRAM`.
        EnemyArtDecompressRouteLoc36E24::Vram
    }
}

#[allow(non_snake_case)]
#[allow(dead_code)]
pub(crate) fn enemy_art_load_tail_loc_36E3E(slot_index: u16) -> EnemyArtLoadTailLoc36E3E {
    // ASM loc_36E3E/loc_36E42 (kid.asm:38733-38739): restore the saved
    // `d7/a1`, increment the restored slot index, and loop back to loc_36D16
    // until the post-increment value reaches three.
    let next_slot_index = slot_index.wrapping_add(1);
    EnemyArtLoadTailLoc36E3E {
        next_slot_index,
        finished: next_slot_index == 3,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct EnemyArtSlotAssignmentSub36E84 {
    pub(crate) slot_index: u16,
    pub(crate) palette_line: u8,
    pub(crate) vram_tile: u16,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct EnemyGfxSlotLinkLoc32188 {
    pub(crate) next_free_gfx_slot: u32,
    pub(crate) allocated_slot_next: u32,
    pub(crate) owner_gfx_head: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct EnemyGfxSlotInitLoc3219E {
    pub(crate) active_word_offset: u16,
    pub(crate) active_word_value: u16,
    pub(crate) next_number_gfx_objects: u16,
    pub(crate) a1_slot_address: u32,
    pub(crate) clear_start_offset: u16,
    pub(crate) cleared_byte_count: u16,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct SharedEnemyAttachmentChildSub3BF72 {
    pub(crate) x_pos: i16,
    pub(crate) y_pos: i16,
    pub(crate) x_velocity: i32,
    pub(crate) x_direction: Option<u8>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct SharedEnemyAttachmentLayoutSub3BF72 {
    pub(crate) primary: SharedEnemyAttachmentChildSub3BF72,
    pub(crate) secondary: Option<SharedEnemyAttachmentChildSub3BF72>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct SharedEnemyAttachmentInputSub3BF72 {
    pub(crate) family: EnemyFamily,
    pub(crate) parent_x_pos: i16,
    pub(crate) parent_y_pos: i16,
    pub(crate) parent_x_direction: u8,
    pub(crate) child_y_offset: i16,
    pub(crate) robot_parent_state: u16,
    pub(crate) robot_child_mapping: u16,
    pub(crate) child_x_direction: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum SharedEnemyDriveSoundEffectSub3C352 {
    DrillMoving,
    TankDriving,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum SharedEnemyDriveSoundCallSub3C352 {
    PlaySound(SharedEnemyDriveSoundEffectSub3C352),
    PlaySound2(SharedEnemyDriveSoundEffectSub3C352),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct SharedEnemyDriveSoundResultSub3C352 {
    pub(crate) next_previous_motion_active: bool,
    pub(crate) next_refcount: u8,
    pub(crate) sound_call: Option<SharedEnemyDriveSoundCallSub3C352>,
}

// ASM loc_3C03A / loc_3C0E6 (kid.asm:43500-43503): after velocity clear +
// direction set, `tst.w $3A(a5)` checks whether the enemy has a child
// attachment pointer; if nonzero, `bsr.w sub_3BF72` syncs the child's
// position/direction to the parent before the hibernate frame at loc_3C044.
// ASM loc_3C18A (kid.asm:43648-43651): identical attachment check in the
// leftward walking loop — reached via loc_3C25E after gravity update,
// runs once per frame before hibernate at loc_3C194.
#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn shared_enemy_attachment_sub_3BF72(
    input: SharedEnemyAttachmentInputSub3BF72,
) -> SharedEnemyAttachmentLayoutSub3BF72 {
    let SharedEnemyAttachmentInputSub3BF72 {
        family,
        parent_x_pos,
        parent_y_pos,
        parent_x_direction,
        child_y_offset,
        robot_parent_state,
        robot_child_mapping,
        child_x_direction,
    } = input;

    if matches!(family, EnemyFamily::Driller) {
        // ASM sub_3BF72 / loc_3BFD4 (kid.asm:43464-43487): Driller keeps a
        // paired child layout centred on the parent, with asymmetric 23/24px
        // X offsets and opposing `$8000` tread velocities.
        // ASM loc_3C000: nonzero x_direction → subi.w #$17,x_pos(a4); addi.w #$18,$1A(a2)
        // ASM zero direction: subi.w #$18,x_pos(a4); addi.w #$17,$1A(a2); bra.s loc_3C00C
        let (primary_x, secondary_x) = if parent_x_direction == 0 {
            (parent_x_pos - 0x18, parent_x_pos + 0x17)
        } else {
            // ASM loc_3C000: right-facing Driller offsets
            (parent_x_pos - 0x17, parent_x_pos + 0x18)
        };
        // ASM loc_3C00C: convergence — copy y_pos to both objects, set
        // x_vel=$FFFF8000 for primary (a4) and x_vel=$8000 for secondary (a2)
        return SharedEnemyAttachmentLayoutSub3BF72 {
            primary: SharedEnemyAttachmentChildSub3BF72 {
                x_pos: primary_x,
                y_pos: parent_y_pos,
                x_velocity: -0x8000,
                x_direction: None,
            },
            secondary: Some(SharedEnemyAttachmentChildSub3BF72 {
                x_pos: secondary_x,
                y_pos: parent_y_pos,
                x_velocity: 0x8000,
                x_direction: None,
            }),
        };
    }

    // ASM sub_3BF72 (kid.asm:43425-43443): shared child sprite copies the
    // parent's world position, then subtracts the child's `$46(a4)` Y offset.
    // ASM loc_3BFA0: move.b x_direction(a3),$16(a4) — default direction copy
    let mut primary_x_pos = parent_x_pos;
    let primary_y_pos = parent_y_pos - child_y_offset;
    let mut primary_x_direction = parent_x_direction;

    if matches!(family, EnemyFamily::Robot)
        // ASM loc_3BFA8 only enters the offset-preserving Robot path once the
        // child leaves the special `$800` mapping and `$40(a3)` is non-zero.
        && robot_child_mapping != 0x0800
        && robot_parent_state != 0
        && child_x_direction != parent_x_direction
    {
        // ASM loc_3BFBE/loc_3BFCC preserves the old child flip while nudging
        // the child ±4px so the arm does not pop when the parent turns.
        primary_x_direction = child_x_direction;
        if child_x_direction == 0 {
            primary_x_pos += 4;
        } else {
            primary_x_pos -= 4;
        }
    }

    SharedEnemyAttachmentLayoutSub3BF72 {
        primary: SharedEnemyAttachmentChildSub3BF72 {
            x_pos: primary_x_pos,
            y_pos: primary_y_pos,
            x_velocity: 0,
            x_direction: Some(primary_x_direction),
        },
        secondary: None,
    }
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn shared_enemy_drive_sound_sub_3C352(
    family: EnemyFamily,
    motion_active: bool,
    previous_motion_active: bool,
    refcount: u8,
) -> SharedEnemyDriveSoundResultSub3C352 {
    let Some(effect) = (match family {
        // ASM sub_3C352 (kid.asm:43849-43880): `d6=0` selects
        // `byte_3C46C[0]=sfx_Drill_moving`.
        EnemyFamily::Driller => Some(SharedEnemyDriveSoundEffectSub3C352::DrillMoving),
        // ASM sub_3C352: `d6=1` selects `byte_3C46C[1]=sfx_Tank_driving` for
        // both RockTank variants.
        EnemyFamily::Tank | EnemyFamily::TankShooting => {
            Some(SharedEnemyDriveSoundEffectSub3C352::TankDriving)
        }
        _ => None,
    }) else {
        return SharedEnemyDriveSoundResultSub3C352 {
            next_previous_motion_active: previous_motion_active,
            next_refcount: refcount,
            sound_call: None,
        };
    };

    // ASM loc_3C372 (kid.asm:43861-43872): motion_active ($19(a3)) is
    // nonzero — stopped→moving edge. Checks previous_motion_active
    // ($6A(a5)); if already active, skip (loc_3C3C6). Otherwise
    // subi.b #1 on shared refcount at $FFFFFA34+d7; when counter
    // reaches zero, plays sound via j_PlaySound2 using byte_3C46C[d6].
    if motion_active {
        if previous_motion_active {
            return SharedEnemyDriveSoundResultSub3C352 {
                next_previous_motion_active: true,
                next_refcount: refcount,
                sound_call: None,
            };
        }

        let next_refcount = refcount.wrapping_sub(1);
        return SharedEnemyDriveSoundResultSub3C352 {
            next_previous_motion_active: true,
            next_refcount,
            sound_call: (next_refcount == 0)
                .then_some(SharedEnemyDriveSoundCallSub3C352::PlaySound2(effect)),
        };
    }

    // ASM loc_3C39C (kid.asm:43875-43889): motion_active ($19(a3)) is
    // zero — moving→stopped edge. Checks previous_motion_active
    // ($6A(a5)); if already stopped, skip (loc_3C3C6). Otherwise
    // plays sound via j_PlaySound using byte_3C46C[d6] when shared
    // refcount at $FFFFFA34+d7 is zero, then addi.b #1 increments it.
    if !previous_motion_active {
        return SharedEnemyDriveSoundResultSub3C352 {
            next_previous_motion_active: false,
            next_refcount: refcount,
            sound_call: None,
        };
    }

    // ASM loc_3C3BE (kid.asm:43886-43889): convergence point after the
    // optional j_PlaySound — `move.l d5,a4; addi.b #1,(a4,d7.w)` restores
    // the $FFFFFA34 base and increments the shared refcount unconditionally.
    // The sound plays only when refcount was zero (loc_3C39C tst.b check);
    // the increment always executes regardless.
    SharedEnemyDriveSoundResultSub3C352 {
        next_previous_motion_active: false,
        next_refcount: refcount.wrapping_add(1),
        sound_call: (refcount == 0).then_some(SharedEnemyDriveSoundCallSub3C352::PlaySound(effect)),
    }
}

/// Result of the deactivation-time drive sound cleanup at loc_3C494.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct DeactivationDriveSoundResultLoc3C494 {
    pub(crate) next_refcount: u8,
    pub(crate) sound_call: Option<SharedEnemyDriveSoundCallSub3C352>,
}

/// ASM loc_3C494 (kid.asm:43979-43997): deactivation-time drive sound cleanup
/// for Driller/Tank enemies.  Reached from loc_3C46E's object_meta dispatch
/// when d7 matches objid_Driller or objid_RockTank/objid_RockTank_shooting.
///
/// The routine checks two flags — `$19(a3)` (motion_active) and `$6A(a5)`
/// (previous_motion_active) — to decide whether to decrement the shared drive
/// sound refcount at `$FFFFFA34+d7`.
///
/// - loc_3C4A2 (kid.asm:43987): `$19(a3)==0` path — tests `$6A(a5)`;
///   if nonzero skips sound cleanup (`bne.s loc_3C4C0`), otherwise falls
///   through to loc_3C4A8.
/// - loc_3C4A8 (kid.asm:43991): shared refcount decrement path —
///   `lea ($FFFFFA34).w,a4; subi.b #1,(a4,d7.w)`.  When counter reaches
///   zero, plays `j_PlaySound2` with `byte_3C46C[d6]` (Drill or Tank sfx).
///
/// In both `$19` states the gate is the same: `$6A != 0` → skip sound.
/// So the deactivation sound cleanup runs iff `$6A(a5) == 0`.
#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn deactivation_drive_sound_cleanup_loc_3C494(
    family: EnemyFamily,
    motion_active: bool,
    previous_motion_active: bool,
    refcount: u8,
) -> DeactivationDriveSoundResultLoc3C494 {
    let Some(effect) = (match family {
        EnemyFamily::Driller => Some(SharedEnemyDriveSoundEffectSub3C352::DrillMoving),
        EnemyFamily::Tank | EnemyFamily::TankShooting => {
            Some(SharedEnemyDriveSoundEffectSub3C352::TankDriving)
        }
        _ => None,
    }) else {
        return DeactivationDriveSoundResultLoc3C494 {
            next_refcount: refcount,
            sound_call: None,
        };
    };

    // ASM loc_3C494: tst.b $19(a3); beq.s loc_3C4A2
    // Both paths check $6A(a5) and skip if nonzero.
    if motion_active {
        // $19 != 0: tst.b $6A(a5); bne.s loc_3C4C0; bra.s loc_3C4A8
        if previous_motion_active {
            return DeactivationDriveSoundResultLoc3C494 {
                next_refcount: refcount,
                sound_call: None,
            };
        }
    } else {
        // ASM loc_3C4A2: $19 == 0: tst.b $6A(a5); bne.s loc_3C4C0
        if previous_motion_active {
            return DeactivationDriveSoundResultLoc3C494 {
                next_refcount: refcount,
                sound_call: None,
            };
        }
    }

    // ASM loc_3C4A8: lea ($FFFFFA34).w,a4; subi.b #1,(a4,d7.w); bne.s loc_3C4C0
    let next_refcount = refcount.wrapping_sub(1);
    DeactivationDriveSoundResultLoc3C494 {
        next_refcount,
        sound_call: (next_refcount == 0)
            .then_some(SharedEnemyDriveSoundCallSub3C352::PlaySound2(effect)),
    }
}

/// Result of the deactivation status table update at loc_3C4C0.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum DeactivationStatusActionLoc3C4C0 {
    /// loc_3C4E6: bit 7 clear (positive slot) — write `$2168` to
    /// `EnemyStatus_Table[d0]`.
    WriteRespawnMarker { index: usize },
    /// Bit 7 set + bit 6 set — subtract `$400` from `EnemyStatus_Table[d0 & $3F]`.
    SubtractSpawnCounter { index: usize },
    /// Bit 7 set + bit 6 clear — no table update, just delete.
    DeleteOnly,
}

/// ASM loc_3C4C0 (kid.asm:43999-44020): deactivation status table update.
/// After decrementing `Number_of_Enemy`, reads `$42(a5)` and dispatches:
///
/// - `bpl.s loc_3C4E6`: bit 7 clear → positive slot.
///   loc_3C4E6 (kid.asm:44013): `add.w d0,d0; lea (EnemyStatus_Table).w,a0;
///   move.w #$2168,d7; move.w d7,(a0,d0.w)` writes the respawn marker.
///
/// - Bit 7 set + bit 6 set → `andi.w #$3F,d0; add.w d0,d0;
///   subi.w #$400,(a0,d0.w)` decrements the spawn counter.
///
/// - Bit 7 set + bit 6 clear → falls straight to loc_3C4F4 (delete).
#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn deactivation_status_update_loc_3C4C0(
    object_status_slot: u8,
) -> DeactivationStatusActionLoc3C4C0 {
    // ASM: moveq #0,d0; move.b $42(a5),d0; bpl.s loc_3C4E6
    if object_status_slot & 0x80 == 0 {
        // ASM loc_3C4E6: add.w d0,d0 → word index; move.w #$2168,(a0,d0.w)
        return DeactivationStatusActionLoc3C4C0::WriteRespawnMarker {
            index: usize::from(object_status_slot),
        };
    }
    // ASM: btst #6,d0; beq.s loc_3C4F4
    if object_status_slot & 0x40 == 0 {
        return DeactivationStatusActionLoc3C4C0::DeleteOnly;
    }
    // ASM: andi.w #$3F,d0; add.w d0,d0; subi.w #$400,(a0,d0.w)
    DeactivationStatusActionLoc3C4C0::SubtractSpawnCounter {
        index: usize::from(object_status_slot & 0x3F),
    }
}

pub(crate) const ROCK_TANK_SHOOTING_MAPPING_SUB_3C4F8: u16 = 0x7F10;
pub(crate) const ROBOT_IDLE_MAPPING_SUB_3C4F8: u16 = 0x82F0;
pub(crate) const ROBOT_ATTACK_MAPPING_A_SUB_3C4F8: u16 = 0x8338;
pub(crate) const ROBOT_ATTACK_MAPPING_B_SUB_3C4F8: u16 = 0x8340;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) enum SharedEnemyRobotHelperKindSub3C4F8 {
    RockTankShooting,
    Robot,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) struct SharedEnemyRobotHelperInputSub3C4F8 {
    pub(crate) helper_kind: SharedEnemyRobotHelperKindSub3C4F8,
    pub(crate) parent_mapping: u16,
    pub(crate) parent_state_word_32: u16,
    pub(crate) child_state_word_40: u16,
    pub(crate) child_counter_word_4e: u16,
    pub(crate) child_timer_word_54: u16,
    /// ASM loc_3C56A: `x_pos(a4)` — the turret child's X position.
    pub(crate) parent_x_pos: u16,
    /// ASM loc_3C5FC: `x_pos(a3)` — the walker's own X position, used when
    /// `$40(a3)==2` selects the alternate direction-check path.
    pub(crate) walker_x_pos: u16,
    pub(crate) kid_x_pos: u16,
    pub(crate) parent_x_direction: u8,
    pub(crate) parent_flag_15: bool,
    pub(crate) random_word: u16,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) struct SharedEnemyRobotHelperResultSub3C4F8 {
    pub(crate) spawned_loc_3_b730_worker: bool,
    pub(crate) next_parent_mapping: u16,
    pub(crate) next_parent_x_direction: u8,
    pub(crate) next_parent_flag_15: bool,
    pub(crate) next_child_counter_word_4e: u16,
    pub(crate) next_child_timer_word_54: u16,
    pub(crate) initialised_parent_animation_stru_3_a716: bool,
    pub(crate) branch_to_robot_shoot: bool,
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn shared_enemy_robot_helper_sub_3C4F8(
    input: SharedEnemyRobotHelperInputSub3C4F8,
) -> SharedEnemyRobotHelperResultSub3C4F8 {
    let mut result = SharedEnemyRobotHelperResultSub3C4F8 {
        spawned_loc_3_b730_worker: false,
        next_parent_mapping: input.parent_mapping,
        next_parent_x_direction: input.parent_x_direction,
        next_parent_flag_15: input.parent_flag_15,
        next_child_counter_word_4e: input.child_counter_word_4e,
        next_child_timer_word_54: input.child_timer_word_54,
        initialised_parent_animation_stru_3_a716: false,
        branch_to_robot_shoot: false,
    };

    match input.helper_kind {
        SharedEnemyRobotHelperKindSub3C4F8::RockTankShooting => {
            // ASM sub_3C4F8 loc_3C50E only allocates `loc_3B730` when the
            // RockTank parent still uses `LnkTo_unk_C7F10` and `$32(a4)=5`.
            if input.parent_mapping == ROCK_TANK_SHOOTING_MAPPING_SUB_3C4F8
                && input.parent_state_word_32 == 5
            {
                result.spawned_loc_3_b730_worker = true;
            }
            result
        }
        SharedEnemyRobotHelperKindSub3C4F8::Robot => {
            if input.child_counter_word_4e == 0 {
                // ASM loc_3C538 only runs the Robot helper from the idle
                // parent shell `LnkTo_unk_C82F0` with `$32(a4)=9`.
                if input.parent_mapping != ROBOT_IDLE_MAPPING_SUB_3C4F8
                    || input.parent_state_word_32 != 9
                {
                    return result;
                }
                // ASM loc_3C554 (kid.asm:44066-44072): cmpi.w #2,$40(a3);
                // beq.w loc_3C5FC — $40==2 skips the random gate entirely
                // and uses x_pos(a3) for direction check. Otherwise
                // j_Get_RandomNumber; cmpi.w #$A,d7; ble loc_3C56A — only
                // proceed when random <= $A.
                if input.child_state_word_40 != 2 && input.random_word > 0x000A {
                    return result;
                }
                if input.child_state_word_40 < 1 {
                    result.branch_to_robot_shoot = true;
                    return result;
                }

                // ASM loc_3C56A (kid.asm:44075-44084): uses x_pos(a4) for
                // the direction check. loc_3C5FC (kid.asm:44138-44157):
                // uses x_pos(a3) — the walker's own position — when
                // $40(a3)==2.
                let x_for_check = if input.child_state_word_40 == 2 {
                    input.walker_x_pos // ASM loc_3C5FC: x_pos(a3)
                } else {
                    input.parent_x_pos // ASM loc_3C56A: x_pos(a4)
                };

                // ASM loc_3C588 (kid.asm:44087-44089): bmi.s path — when
                // x_pos < Kid_x (entity is left of Kid), tst.b $16(a4);
                // bne.s loc_3C5EE — facing right already means facing Kid,
                // so branch to Robot_Shoot. The non-bmi path (entity
                // right of or at Kid) mirrors: beq.s loc_3C5EE when
                // $16(a4)==0 (facing left, towards Kid).
                // ASM loc_3C612 (kid.asm:44148-44150): identical check in
                // the loc_3C5FC ($40==2) path — bmi.s to loc_3C612: tst.b
                // $16(a4); bne.s loc_3C5EE when turret left of Kid and
                // already facing right (towards Kid).
                let entity_left_of_kid = (x_for_check as i16) < (input.kid_x_pos as i16);
                let parent_faces_right = input.parent_x_direction != 0;
                let facing_matches_kid = if entity_left_of_kid {
                    parent_faces_right
                } else {
                    !parent_faces_right
                };
                if facing_matches_kid {
                    result.branch_to_robot_shoot = true;
                    return result;
                }

                // ASM loc_3C618 (kid.asm:44152-44157): arms the helper
                // phase — identical to loc_3C58E: st $4E(a5), sf $15(a4),
                // timer=9, mapping=C8338, then bra.w loc_3C5A2. Reached
                // from the loc_3C5FC ($40==2) path when the turret is not
                // already facing Kid.
                result.next_child_counter_word_4e |= 0xFF00;
                result.next_parent_flag_15 = false;
                result.next_child_timer_word_54 = 9;
                result.next_parent_mapping = ROBOT_ATTACK_MAPPING_A_SUB_3C4F8;
            }

            // ASM loc_3C5A2 decrements `$54(a5)` before inspecting the low byte
            // of `$4E(a5)`. The helper therefore burns one frame immediately
            // when loc_3C5FC falls through into the countdown path.
            if result.next_child_timer_word_54 > 0 {
                result.next_child_timer_word_54 -= 1;
            }
            if result.next_child_timer_word_54 != 0 {
                return result;
            }

            let phase = (result.next_child_counter_word_4e & 0x00FF).wrapping_add(1);
            // ASM loc_3C5D8 (kid.asm:44116-44126): phase > 2 completes the
            // turn sequence — st $15(a4), clr.w $4E(a5), init stru_3A716 on
            // parent, then falls through to loc_3C5EE which pops the return
            // address and bra.w Enemy03_Robot_Shoot.
            if phase > 2 {
                result.next_parent_flag_15 = true;
                result.next_child_counter_word_4e = 0;
                result.initialised_parent_animation_stru_3_a716 = true;
                result.branch_to_robot_shoot = true;
                return result;
            }

            if phase == 2 {
                result.next_parent_x_direction ^= 0xFF;
            }
            // ASM loc_3C5C4 (kid.asm:44108-44113): or.w d7,$4E(a5) stores
            // the incremented phase; add.w d7,d7 doubles it as a word
            // offset into off_3C5F6 → $22(a4). Timer reset to 9.
            // off_3C5F6: [LnkTo_unk_C8338, LnkTo_unk_C8340, LnkTo_unk_C8338]
            //   phase 1 → offset 2 → C8340 (ROBOT_ATTACK_MAPPING_B)
            //   phase 2 → offset 4 → C8338 (ROBOT_ATTACK_MAPPING_A)
            result.next_child_counter_word_4e |= phase;
            result.next_parent_mapping = match phase {
                1 => ROBOT_ATTACK_MAPPING_B_SUB_3C4F8,
                2 => ROBOT_ATTACK_MAPPING_A_SUB_3C4F8,
                _ => unreachable!(),
            };
            result.next_child_timer_word_54 = 9;
            result
        }
    }
}

#[allow(non_snake_case)]
#[allow(dead_code)]
pub(crate) fn enemy_art_slot_assignment_sub_36E84(
    enemy_type_slots: &[u16],
    enemy_id: u16,
) -> EnemyArtSlotAssignmentSub36E84 {
    // ASM sub_36E84 / loc_32146 / loc_3215E / loc_3216C / loc_3217C
    // (kid.asm:37582-37609) mirrors the enemy art slot search used by
    // loc_36EA6 (kid.asm:38819): seed the slot index at zero, compare the
    // masked 12-bit enemy id against the first two header slots, then use the
    // selected index to write `palette_line(a3)`, `$3E(a3)`, and
    // `vram_tile(a3)` from the ROM art tables.
    const ENEMY_ART_PALETTE_LINES: [u8; 4] = [1, 2, 2, 0];
    const ENEMY_ART_VRAM_TILE_ADDRESSES: [u16; 3] = [0x02FB, 0x0409, 0x0517];

    let enemy_id = enemy_id & 0x0FFF;
    let slot_index = if enemy_type_slots
        .first()
        .is_some_and(|slot| slot & 0x0FFF == enemy_id)
    {
        0
    } else if enemy_type_slots
        // ASM loc_3215E (kid.asm:37596-37602): read `4(a0)`, mask it with
        // `$0FFF`, compare it to `d0`, and allocate slot 1 when it matches;
        // otherwise increment `d1` again so loc_3216C falls back to slot 2.
        .get(1)
        .is_some_and(|slot| slot & 0x0FFF == enemy_id)
    {
        1
    } else {
        2
    };

    EnemyArtSlotAssignmentSub36E84 {
        slot_index,
        palette_line: ENEMY_ART_PALETTE_LINES[usize::from(slot_index)],
        vram_tile: ENEMY_ART_VRAM_TILE_ADDRESSES[usize::from(slot_index)],
    }
}

#[allow(non_snake_case)]
#[allow(dead_code)]
pub(crate) fn enemy_gfx_slot_link_loc_32188(
    free_slot_address: u32,
    next_free_slot_address: u32,
    owner_gfx_head: u32,
) -> EnemyGfxSlotLinkLoc32188 {
    // ASM loc_32188 (kid.asm:37597-37603): save `a0`, pop the first node from
    // `Addr_NextFreeGfxObjectSlot`, copy the owner's current `0(a3)` link into
    // the allocated slot's `0(a0)`, then install that slot as the new
    // owner-head link at `0(a3)`.
    EnemyGfxSlotLinkLoc32188 {
        next_free_gfx_slot: next_free_slot_address,
        allocated_slot_next: owner_gfx_head,
        owner_gfx_head: free_slot_address,
    }
}

#[allow(non_snake_case)]
#[allow(dead_code)]
pub(crate) fn enemy_gfx_slot_init_loc_3219E(
    allocated_slot_address: u32,
    number_gfx_objects: u16,
) -> EnemyGfxSlotInitLoc3219E {
    // ASM loc_3219E (kid.asm:37604-37608) writes `#1` to `$32(a0)`,
    // increments `Number_GfxObjects`, copies the allocated slot address into
    // `a1`, then advances `a0` to `4(a0)` so loc_321B2 can clear `$48` bytes
    // across the rest of the gfx slot body.
    EnemyGfxSlotInitLoc3219E {
        active_word_offset: 0x0032,
        active_word_value: 0x0001,
        next_number_gfx_objects: number_gfx_objects.wrapping_add(1),
        a1_slot_address: allocated_slot_address,
        clear_start_offset: 0x0004,
        cleared_byte_count: 0x0048,
    }
}

impl NativeRuntime {
    fn enemy_loading_enabled(&self) -> bool {
        !matches!(self.boot_state, BootState::DemoPlay { .. })
            || (self.current_level == LevelId(demo::DEMO_1_LEVEL)
                && self
                    .demo_startup_frame()
                    .zip(self.demo_uses_second_recording())
                    .is_some_and(|(frame, use_demo_2)| {
                        frame >= demo::enemy_warmup_frame(use_demo_2)
                    }))
    }

    pub(crate) fn refresh_active_roster(&mut self) {
        if self.paused {
            return;
        }
        if self.gameplay_hibernating_for_transform() {
            return;
        }
        self.active_enemy_roster.clear();
        self.active_platforms.clear();
        self.active_platform_roster.clear();
        if self.level_skip_resume_hold {
            self.active_enemies.clear();
            self.block_state.evanescent_blocks.clear();
            self.block_state.reset_shooter_state_sub_F06A();
            self.block_state.ghost_groups.clear();
            return;
        }
        if !matches!(
            self.boot_state,
            BootState::InGame | BootState::DemoPlay { .. }
        ) {
            return;
        }

        let Some(slice) = self.opening_slice else {
            return;
        };
        if !matches!(
            self.player_lifecycle,
            PlayerLifecycle::Alive
                | PlayerLifecycle::DeathWindup { .. }
                | PlayerLifecycle::Respawning { .. }
                | PlayerLifecycle::LevelComplete { .. }
        ) {
            return;
        }
        let window = ActivationWindow::from_camera(slice.camera);
        let camera_x_px = slice.camera.x_pos >> 16;
        let camera_y_px = slice.camera.y_pos >> 16;
        let level_bounds = self.level_bounds(self.current_level);
        let enemy_layout = self.enemy_layout(self.current_level).cloned();
        let platform_layout = self.platform_layout(self.current_level).cloned();
        let blank_top_rows = self.level_header(self.current_level).map_or(0, |header| {
            terrain::blank_top_rows_from_height_flag_bits_sub_140F6(header.height_flag_bits)
        });
        if let Some(enemy_layout) = enemy_layout.as_ref() {
            if self.enemy_loading_enabled() {
                activate_enemies_with_rng(
                    &mut self.active_enemies,
                    enemy_layout,
                    window,
                    camera_x_px,
                    camera_y_px,
                    level_bounds,
                    slice.player.x_pos,
                    slice.player.y_pos,
                    blank_top_rows,
                    &self.killed_enemies,
                    &mut self.rom_random,
                );
                step_active_enemies_with_rng(
                    &mut self.active_enemies,
                    self.block_grid.as_ref(),
                    Some(slice.player.x_pos),
                    Some(slice.player.y_pos),
                    Some(camera_x_px),
                    Some(slice.player.facing),
                    Some(slice.player.x_velocity),
                    self.progression.current_form,
                    Some(&mut self.projectiles),
                    &mut self.rom_random,
                    &mut self.drilled_blocks,
                );
                // Apply drilled blocks to the block grid after enemy step.
                // Driller.asm loc_3BE78 destroys Smash/Ice blocks during pacing.
                if let Some(ref mut grid) = self.block_grid {
                    for (col, row) in self.drilled_blocks.drain(..) {
                        grid.set_cell_at(col, row, terrain::Cell::from_be_bytes(0, 0));
                    }
                }
                self.drilled_blocks.clear();
                self.active_enemy_roster = self
                    .active_enemies
                    .iter()
                    .map(ActiveEnemy::snapshot)
                    .collect();
            } else {
                // ORIGINAL_QUIRK: during title-screen demo playback, the game
                // suppresses enemy loading for every level except Knights Isle.
                self.active_enemies.clear();
                self.active_enemy_roster.clear();
            }
        }
        if let Some(platform_layout) = platform_layout.as_ref() {
            let mut objects = slice.objects;
            objects
                .platform_loader
                .advance_to_window(platform_layout, window);
            self.active_platform_roster = objects
                .platform_loader
                .active_platforms(platform_layout, window);
            let script_table = self.platform_script_table.as_ref();
            self.active_platforms = platform_layout
                .placements
                .iter()
                .take(objects.platform_loader.next_index)
                .filter(|p| platform_is_active(p, window))
                .map(|p| ActivePlatform::from_placement(p, script_table))
                .collect();
            self.opening_slice = Some(OpeningSliceState { objects, ..slice });
        }
    }
}
