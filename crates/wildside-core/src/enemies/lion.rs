use super::*;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct LionInitVisibleState {
    pub(crate) status_flag_13: bool,
    pub(crate) object_meta: u16,
    pub(crate) palette_line: u8,
    pub(crate) current_hp: u16,
    pub(crate) facing: Direction,
}

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
        object_meta: 0x0092,
        palette_line: 2,
        current_hp: enemy_level.wrapping_add(6),
        facing,
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct LionInitMotionProfile {
    pub(crate) walk_speed: i32,
    pub(crate) charge_speed: i32,
    pub(crate) animation_script: &'static str,
}

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
pub(crate) enum LionFacingDispatch {
    LeftPacing,
    RightPacing,
}

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
pub(crate) enum LionTrackingGate {
    Hold,
    TrackKid,
    ClearLatchAndTrackKid,
}

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
pub(crate) struct LionLatchClearResult {
    pub(crate) range_latch_set: bool,
    pub(crate) facing: Direction,
}

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

pub(crate) const LION_COLID_HURT: u16 = colid::HURT;
pub(crate) const LION_COLID_KIDABOVE: u16 = colid::KIDABOVE;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
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
    //
    // The `ReadyToFire` branch below implements the facing decision and child
    // allocation that the ROM splits across two labels:
    // - ASM Lion.asm loc_3E0D6 is the `st x_direction(a3)` arm taken when Kid is
    //   to the Lion's right (`x_pos(a4) > x_pos(a3)`), giving `facing = Right`;
    //   the fall-through `sf x_direction(a3)` arm gives `facing = Left`.
    // - ASM Lion.asm loc_3E0DA then `exg a0,a1; Allocate_ObjectSlot;
    //   move.l #loc_3E12A,4(a0)` allocates the linked child GfxObject running
    //   the projectile worker (selector `$8000`) and bumps `$5B(a5)` —
    //   captured here as `projectile_worker_selector`/`projectile_worker`/
    //   `active_projectile_count_delta`.
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

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum LionCollisionResumeLoc3E318 {
    Loc3E7CA,
    Loc3E6AE,
    Loc3E560,
    Loc3E3AC,
}

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
pub(crate) struct LionProjectileLaunchTail {
    pub(crate) x_offset_pixels: i16,
    pub(crate) x_velocity: i32,
    pub(crate) animation_script: &'static str,
}

pub(crate) fn lion_projectile_launch_tail_loc_3e180(
    parent_facing: Direction,
) -> LionProjectileLaunchTail {
    // ASM Lion.asm loc_3E180: child setup enters with `d7` as the projectile
    // X offset and `d6` as X velocity. Clear parent `x_direction` negates both
    // before the label; set parent `x_direction` keeps `+$A` and `+$60000`.
    // The label applies `d7` to `x_pos(a3)`, stores `d6` in `x_vel(a3)`, then
    // initialises `stru_3DF34`.
    //
    // This is the tail of the linked-child worker installed at loc_3E0DA.
    // ASM Lion.asm loc_3E12A — the worker's init prologue runs
    // `Load_GfxObjectSlot`, copies the parent's `$71`, sets `palette_line=2`,
    // `vram_tile=$2FB`, `st $13(a3)`, `st is_moved`/`st has_level_collision`,
    // then seeds `x_pos`/`y_pos` from the parent (`-$18` y) before reaching
    // loc_3E180.
    //
    // SUBSUMED, not a gap (verified 2026-06-04): the prologue fields are either
    // render-only or already covered by native's projectile behaviour, and none
    // are part of the state-level oracle comparison.
    //   - `palette_line=2` / `vram_tile=$2FB` are pure sprite-art selectors. The
    //     trace/oracle diff (trace.rs / tas_diff.rs) carries no `palette_line`
    //     or `vram_tile` field for projectiles, so these never affect parity.
    //   - `st is_moved` (object advanced by its velocity) is subsumed: the
    //     native `EnemyProjectile` integrates `x_pos`/`y_pos` from velocity each
    //     frame in `fire_lion_projectile_loc_3e09e` / the projectile step.
    //   - `st has_level_collision` (terrain collision) is subsumed by the spawn
    //     setting `wall_bounce: true` (ASM loc_3E260) plus homing/cap.
    //   - `$71` copy and `st $13` are engine slot/hibernate bookkeeping with no
    //     compared-state effect on the simplified projectile carrier.
    // Level 0x07 recording-settle probes are now clean enough to compare the
    // first Lion projectile timing: the respawn-settle hidden camera tail ages
    // the Lion through loc_3E09E before frame 0. Only the offset/velocity
    // launch tail is modelled here; the prologue art/flag fields are
    // deliberately not ported because they would not change oracle parity.
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

fn lion_projectile_x_offset_from_native_carrier_loc_3e12a(parent_facing: Direction) -> i16 {
    let tail = lion_projectile_launch_tail_loc_3e180(parent_facing);
    match parent_facing {
        Direction::Left => {
            // ASM Lion.asm loc_3E12A copies `x_pos` from the parent GfxObject,
            // not the enemy object carrier. Issue #289 Level 0x07 oracle frame
            // 19 has parent gfx `@517,704` and shot `@501,680` after the first
            // `$FFFA0000` movement step; native's carrier is 12 px right of
            // that gfx origin for the left-facing Lion.
            tail.x_offset_pixels - 12
        }
        Direction::Right => tail.x_offset_pixels,
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
pub(crate) enum LionLoc3E434Branch {
    KeepAndContinue,
    Check60Band,
    Check70Band,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct LionLoc3E434Result {
    pub(crate) next_shared_state: i16,
    pub(crate) branch: LionLoc3E434Branch,
}

fn lion_kid_minus_lion_x_delta(lion_x_word: u16, kid_x_word: u16) -> i16 {
    let lion_x = i16::from_be_bytes(lion_x_word.to_be_bytes());
    let kid_x = i16::from_be_bytes(kid_x_word.to_be_bytes());
    kid_x.wrapping_sub(lion_x)
}

pub(crate) fn lion_charge_window_gate_loc_3e434(shared_walker_state: i16) -> LionLoc3E434Result {
    // ASM Lion.asm loc_3E434:
    // - `$42 == $A` skips the inner distance gates and continues movement.
    // - `$42 > 0` routes to the tighter ±$70 check in `loc_3E476`.
    // - `$42 == 0` routes to the looser ±$60 check in `loc_3E450`.
    // - `$42 < 0` increments toward zero and continues movement.
    if shared_walker_state == 0x000A {
        LionLoc3E434Result {
            next_shared_state: shared_walker_state,
            branch: LionLoc3E434Branch::KeepAndContinue,
        }
    } else if shared_walker_state > 0 {
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
pub(crate) enum LionLoc3E604Branch {
    ContinueMovement,
    EnterLoc3E4CE,
    EnterLoc3E50A,
}

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
pub(crate) enum LionLoc3E62ABranch {
    ContinueMovement,
    EnterLoc3E642,
}

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
    let landing_resume = lion_left_charge_landing_resume_loc_3e76e();
    LionLeftChargeDownwardCollisionLoc3E732::Landed {
        y_position: u32::from(snapped_y_word) << 16,
        y_velocity: 0,
        left_charge_latch_set: false,
        object_meta_bit7_cleared: true,
        restores_saved_charge_speed: true,
        next_shared_walker_state: (shared_walker_state != 0x000A).then_some(-0x78),
        restored_animation: landing_resume.restored_animation,
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct LionLeftChargeLandingResumeLoc3E76E {
    pub(crate) restored_animation: &'static str,
    pub(crate) next_branch: &'static str,
}

pub(crate) fn lion_left_charge_landing_resume_loc_3e76e() -> LionLeftChargeLandingResumeLoc3E76E {
    // ASM Lion.asm loc_3E76E: load `$62(a5)` into d7, reinitialise the saved
    // charge animation, then branch to loc_3E392's velocity-clear/facing tail.
    LionLeftChargeLandingResumeLoc3E76E {
        restored_animation: "saved_lion_charge_animation",
        next_branch: "loc_3E392",
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) struct LionLeftChargeYCommitLoc3E77A {
    pub(crate) y_position: u32,
    pub(crate) gravity_tail: LionLeftChargeGravityTailLoc3E77E,
}

#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn lion_left_charge_y_commit_loc_3e77a(
    candidate_y_position: u32,
    y_velocity: i32,
) -> LionLeftChargeYCommitLoc3E77A {
    // ASM Lion.asm loc_3E77A: commit projected `d2` to `y_pos(a3)`, then
    // immediately fall through to loc_3E77E's gravity/clamp tail.
    LionLeftChargeYCommitLoc3E77A {
        y_position: candidate_y_position,
        gravity_tail: lion_left_charge_gravity_tail_loc_3e77e(y_velocity),
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct LionLeftChargeGravityTailLoc3E77E {
    pub(crate) y_velocity: i32,
    pub(crate) next_tail: LionLoc3E556Tail,
}

pub(crate) fn lion_left_charge_gravity_tail_loc_3e77e(
    y_velocity: i32,
) -> LionLeftChargeGravityTailLoc3E77E {
    // ASM Lion.asm loc_3E77E: add `$4000` gravity, clamp only when the signed
    // result is at least `$180000`, then branch to loc_3E6A4's hibernate tail.
    let accelerated = y_velocity.wrapping_add(0x0000_4000);
    let capped_velocity = accelerated.min(0x0018_0000);
    lion_left_charge_y_velocity_commit_loc_3e796(capped_velocity)
}

pub(crate) fn lion_left_charge_y_velocity_commit_loc_3e796(
    y_velocity: i32,
) -> LionLeftChargeGravityTailLoc3E77E {
    // ASM Lion.asm loc_3E796: commit the prepared d0 value into `y_vel(a3)`,
    // then branch to loc_3E6A4's airborne hibernate/cull/resume tail.
    LionLeftChargeGravityTailLoc3E77E {
        y_velocity,
        next_tail: lion_left_charge_airborne_resume_loc_3e6a4(),
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
pub(crate) struct LionChargeCollisionReentryLoc3E79E {
    pub(crate) left_charge_latch_set: bool,
    pub(crate) restored_sprite_offset: Option<u16>,
    pub(crate) is_animated: Option<bool>,
}

#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn lion_charge_collision_reentry_loc_3e79e(
    saved_sprite_offset: u16,
) -> LionChargeCollisionReentryLoc3E79E {
    // ASM Lion.asm loc_3E79E: set `$5C(a5)`, then restore
    // `addroffset_sprite(a3)` and clear `is_animated(a3)` only when `$4A(a3)`
    // is non-zero; zero skips the sprite/animation writes before motion seeding.
    LionChargeCollisionReentryLoc3E79E {
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

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) struct LionChargeCollisionMotionSeedLoc3E7B0 {
    pub(crate) x_velocity: i32,
    pub(crate) y_velocity: i32,
    pub(crate) y_position: u32,
}

#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn lion_charge_collision_motion_seed_loc_3e7b0(
    saved_x_velocity: i32,
    y_position: u32,
) -> LionChargeCollisionMotionSeedLoc3E7B0 {
    // ASM Lion.asm loc_3E7B0: clear `y_vel(a3)`, restore `$50(a5)` into
    // `x_vel(a3)`, and apply `addq.w #1,y_pos(a3)` with 16-bit word wrapping.
    LionChargeCollisionMotionSeedLoc3E7B0 {
        x_velocity: saved_x_velocity,
        y_velocity: 0,
        y_position: y_position.wrapping_add(0x0001_0000),
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) struct LionChargeCollisionXCommitLoc3E80C {
    pub(crate) x_position: u32,
    pub(crate) reloads_d0_from_x_position: bool,
    pub(crate) enters_vertical_motion: bool,
}

#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn lion_charge_collision_x_commit_loc_3e80c(
    candidate_x_position: u32,
) -> LionChargeCollisionXCommitLoc3E80C {
    // ASM Lion.asm loc_3E80C: unobstructed collision-reentry motion commits
    // `d2` into `x_pos(a3)`, redundantly reloads d0 from d2, then falls through
    // to the vertical motion block.
    LionChargeCollisionXCommitLoc3E80C {
        x_position: candidate_x_position,
        reloads_d0_from_x_position: true,
        enters_vertical_motion: true,
    }
}

#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn lion_charge_collision_vertical_entry_loc_3e812(
    y_position: u32,
    y_velocity: i32,
    ceiling_probe_offset: u16,
) -> LionLeftChargeVerticalEntryLoc3E6F6 {
    // ASM Lion.asm loc_3E812: collision-reentry vertical motion adds `y_vel`
    // to `y_pos`, compares integer words, and only takes the ceiling-probe path
    // when the signed pixel delta is negative.
    lion_left_charge_vertical_entry_loc_3e6f6(y_position, y_velocity, ceiling_probe_offset)
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) enum LionChargeCollisionDownwardCollisionLoc3E84E {
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
        landing_resume: LionChargeCollisionLandingResumeLoc3E88A,
    },
}

#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn lion_charge_collision_downward_collision_loc_3e84e(
    y_position: u32,
    candidate_y_position: u32,
    downward_pixels: u16,
    floor_probe_hit: bool,
    shared_walker_state: i16,
) -> LionChargeCollisionDownwardCollisionLoc3E84E {
    // ASM Lion.asm loc_3E84E: downward/stationary collision-reentry motion
    // checks the low-nibble clearance to the next 16px floor boundary. Only
    // movement crossing that boundary probes floor terrain; a hit snaps to the
    // boundary, clears `$5C(a5)`/object_meta bit 7, restores `$5E(a5)` into
    // `$50(a5)`, and falls through to loc_3E88A.
    let current_y_word = u16::from_be_bytes(fixed_integer_word(y_position).to_be_bytes());
    let floor_clearance_pixels = current_y_word.wrapping_neg() & 0x000F;
    let needs_floor_probe = floor_clearance_pixels < downward_pixels;

    if !needs_floor_probe || !floor_probe_hit {
        return LionChargeCollisionDownwardCollisionLoc3E84E::Airborne {
            y_position: candidate_y_position,
            floor_clearance_pixels,
            needs_floor_probe,
        };
    }

    let snapped_y_word = current_y_word.wrapping_add(floor_clearance_pixels);
    LionChargeCollisionDownwardCollisionLoc3E84E::Landed {
        y_position: u32::from(snapped_y_word) << 16,
        y_velocity: 0,
        left_charge_latch_set: false,
        object_meta_bit7_cleared: true,
        restores_saved_charge_speed: true,
        next_shared_walker_state: (shared_walker_state != 0x000A).then_some(-0x78),
        landing_resume: lion_charge_collision_landing_resume_loc_3e88a(),
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) struct LionChargeCollisionLandingResumeLoc3E88A {
    pub(crate) restored_animation: &'static str,
    pub(crate) next_branch: &'static str,
}

#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn lion_charge_collision_landing_resume_loc_3e88a()
-> LionChargeCollisionLandingResumeLoc3E88A {
    // ASM Lion.asm loc_3E88A: load `$62(a5)` into d7, reinitialise the saved
    // charge animation, then branch to loc_3E546's velocity-clear/facing tail.
    LionChargeCollisionLandingResumeLoc3E88A {
        restored_animation: "saved_lion_charge_animation",
        next_branch: "loc_3E546",
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) struct LionChargeCollisionYCommitLoc3E896 {
    pub(crate) y_position: u32,
    pub(crate) enters_gravity_tail: bool,
}

#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn lion_charge_collision_y_commit_loc_3e896(
    candidate_y_position: u32,
) -> LionChargeCollisionYCommitLoc3E896 {
    // ASM Lion.asm loc_3E896: floor/ceiling probe misses and no-crossing paths
    // commit projected `d2` into `y_pos(a3)`, then continue through the
    // collision-reentry gravity/clamp tail.
    LionChargeCollisionYCommitLoc3E896 {
        y_position: candidate_y_position,
        enters_gravity_tail: true,
    }
}

#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn lion_charge_collision_gravity_cap_loc_3e89a(y_velocity: i32) -> i32 {
    // ASM Lion.asm loc_3E89A: load `y_vel(a3)`, add `$4000`, then use a
    // signed compare against `$180000`; upward velocities are not clamped.
    let accelerated = y_velocity.wrapping_add(0x0000_4000);
    accelerated.min(0x0018_0000)
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) struct LionChargeCollisionYVelocityCommitLoc3E8B2 {
    pub(crate) y_velocity: i32,
    pub(crate) next_tail: LionLoc3E556Tail,
}

#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn lion_charge_collision_y_velocity_commit_loc_3e8b2(
    y_velocity: i32,
) -> LionChargeCollisionYVelocityCommitLoc3E8B2 {
    // ASM Lion.asm loc_3E8B2: commit prepared d0 into `y_vel(a3)`, then branch
    // to loc_3E7C0's collision-reentry hibernate/cull/resume tail.
    LionChargeCollisionYVelocityCommitLoc3E8B2 {
        y_velocity,
        next_tail: lion_charge_collision_airborne_resume_loc_3e7c0(),
    }
}

pub(crate) fn lion_left_charge_airborne_resume_loc_3e6a4() -> LionLoc3E556Tail {
    // ASM Lion.asm loc_3E6A4: after the airborne Y-velocity update, hibernate
    // for one frame, run sub_3E8BA's live object cull, then `jmp (a0)` back
    // into the active Lion routine. It deliberately preserves velocities.
    lion_hibernate_cull_resume_loc_3e556()
}

#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn lion_charge_collision_airborne_resume_loc_3e7c0() -> LionLoc3E556Tail {
    // ASM Lion.asm loc_3E7C0: after collision-reentry airborne movement,
    // hibernate one frame, run sub_3E8BA's live object cull, then `jmp (a0)`.
    lion_hibernate_cull_resume_loc_3e556()
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct LionLoc3E556Tail {
    pub(crate) hibernate_one_frame: bool,
    pub(crate) cull_routine: &'static str,
    pub(crate) resume_via_callback: bool,
}

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
pub(crate) struct LionCullRejectLoc3E920 {
    pub(crate) next_number_of_enemy: u16,
    pub(crate) status_action: DeactivationStatusActionLoc3C4C0,
    pub(crate) deletes_current_object: bool,
}

#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn lion_cull_reject_loc_3e920(
    number_of_enemy: u16,
    object_status_slot: u8,
) -> LionCullRejectLoc3E920 {
    // ASM Lion.asm loc_3E920: every failed sub_3E8BA bounds/camera-retention
    // check branches to loc_3E956, decrementing `Number_of_Enemy`, updating
    // `EnemyStatus_Table` from `$42(a5)`, then deleting the current object.
    LionCullRejectLoc3E920 {
        next_number_of_enemy: number_of_enemy.wrapping_sub(1),
        status_action: lion_cull_status_update_loc_3e956(object_status_slot),
        deletes_current_object: lion_delete_tail_loc_3e98a().deletes_current_object,
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) struct LionDeleteTailLoc3E98A {
    pub(crate) deletes_current_object: bool,
}

#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn lion_delete_tail_loc_3e98a() -> LionDeleteTailLoc3E98A {
    // ASM Lion.asm loc_3E98A: all loc_3E956 status routes converge on
    // `jmp (j_Delete_CurrentObject).w`.
    LionDeleteTailLoc3E98A {
        deletes_current_object: true,
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) struct LionRespawnMarkerLoc3E97C {
    pub(crate) index: usize,
    pub(crate) word_value: u16,
}

#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn lion_respawn_marker_loc_3e97c(object_status_slot: u8) -> LionRespawnMarkerLoc3E97C {
    // ASM Lion.asm loc_3E97C: positive `$42(a5)` has already been loaded into
    // d0; `add.w d0,d0` makes it an EnemyStatus_Table word offset, then `$2168`
    // is written before the common delete jump at loc_3E98A.
    LionRespawnMarkerLoc3E97C {
        index: usize::from(object_status_slot),
        word_value: 0x2168,
    }
}

pub(crate) fn lion_cull_status_update_loc_3e956(
    object_status_slot: u8,
) -> DeactivationStatusActionLoc3C4C0 {
    // ASM Lion.asm loc_3E956 duplicates loc_3C4C0's delete-tail status-table
    // update, including the positive-slot respawn marker at loc_3E97C.
    deactivation_status_update_loc_3C4C0(object_status_slot)
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) enum LionMediumLoadCullLoc3E924 {
    Return,
    DeleteTail(LionCullRejectLoc3E920),
}

#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn lion_medium_load_cull_loc_3e924(
    x_pos_word: u16,
    y_pos_word: u16,
    camera_x_word: u16,
    camera_y_word: u16,
    number_of_enemy: u16,
    object_status_slot: u8,
) -> LionMediumLoadCullLoc3E924 {
    // ASM Lion.asm loc_3E924: with 11..20 objects live, subtract camera words
    // from the Lion's live integer position and keep only x[-$1A4,+$2E4] and
    // y[-$1A4,+$284]. Failed comparisons branch through loc_3E920's delete tail.
    let relative_x = x_pos_word.wrapping_sub(camera_x_word) as i16;
    let relative_y = y_pos_word.wrapping_sub(camera_y_word) as i16;

    if !(-0x01A4..=0x02E4).contains(&relative_x) || !(-0x01A4..=0x0284).contains(&relative_y) {
        return LionMediumLoadCullLoc3E924::DeleteTail(lion_cull_reject_loc_3e920(
            number_of_enemy,
            object_status_slot,
        ));
    }

    LionMediumLoadCullLoc3E924::Return
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) struct LionTerrainCollisionLoc3E94E {
    pub(crate) next_collision_type: i16,
    pub(crate) resume_via_callback: bool,
}

#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn lion_terrain_collision_loc_3e94e() -> LionTerrainCollisionLoc3E94E {
    // ASM Lion.asm loc_3E94E: terrain contact from the charge movement probes
    // stores `$FFFF` in `collision_type(a3)`, then resumes through `jmp (a0)`.
    LionTerrainCollisionLoc3E94E {
        next_collision_type: -1,
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

// ASM Lion.asm loc_3DFF0 seeds `$48(a5)=$14` and `$4A(a5)=$2F` before
// entering the charge collision loops.
pub(crate) const LION_CHARGE_PROBE_HALF_WIDTH: i32 = 0x14;
pub(crate) const LION_CHARGE_PROBE_HEIGHT: i32 = 0x2F;

/// Lion post-projectile cooldown: $B4 (180 frames) before next charge after
/// firing a projectile (Lion.asm loc_3E096).
pub(crate) const LION_COOLDOWN_AFTER_FIRE: u16 = 180;
/// Lion.asm `stru_3DF0E`: seven 6-tick roar frames before `loc_3E09E`
/// sees `LnkTo_unk_C7E78` and allocates the projectile.
pub(crate) const LION_ROAR_FIRE_DELAY: u16 = 7 * 6;
/// Lion.asm `loc_3E0F4` waits until `$18(a3)` is set after the projectile
/// frame, then restores `$62(a5)` and resumes `loc_3E11E`. The raw
/// `stru_3DF0E` tail is 3 + $19 ticks, but the animation engine's callback
/// timing makes the effective post-fire wait 17 frames in the Level 0x07
/// oracle trace.
pub(crate) const LION_POST_FIRE_RESUME_DELAY: u16 = 17;

/// Lion charge overshoot distance for termination: $70 (112px).
/// ASM does not use a fixed duration — charge ends when Lion overshoots
/// the player by >112px (loc_3E450/3E604).
#[cfg(test)]
pub(crate) const LION_CHARGE_OVERSHOOT: i32 = 0x70;

/// Lion projectile speed (Lion.asm line 165: $60000).
pub(crate) const LION_PROJECTILE_SPEED: i32 = 0x6_0000;

/// Lion projectile spawn X offset: 10px ahead in facing direction.
pub(crate) const LION_PROJECTILE_X_OFFSET: i32 = 10;

/// Lion projectile spawn Y offset: 24px above Lion's Y position ($18).
pub(crate) const LION_PROJECTILE_Y_OFFSET: i32 = 24;

fn increment_lion_active_projectile_count(enemy: &mut ActiveEnemy) {
    enemy.ufo_beam_segments = enemy.ufo_beam_segments.saturating_add(1);
}

fn fire_lion_projectile_loc_3e09e(
    enemy: &mut ActiveEnemy,
    player_x_pos: u32,
    projectiles: &mut Option<&mut Vec<EnemyProjectile>>,
) {
    let dispatch = lion_roar_wait_dispatch_loc_3e09e(
        0,
        true,
        fixed_position_integer_word(enemy.x_pos),
        fixed_position_integer_word(player_x_pos),
    );
    let LionRoarWaitDispatch::ReadyToFire { facing, .. } = dispatch else {
        return;
    };

    enemy.facing = facing;
    if let Some(projs) = projectiles {
        let tail = lion_projectile_launch_tail_loc_3e180(enemy.facing);
        let proj_x = (enemy.x_pos as i64
            + ((i64::from(lion_projectile_x_offset_from_native_carrier_loc_3e12a(
                enemy.facing,
            ))) << 16)) as u32;
        let proj_y = (enemy.y_pos as i64 - ((i64::from(LION_PROJECTILE_Y_OFFSET)) << 16)) as u32;
        projs.push(EnemyProjectile {
            x_pos: proj_x,
            y_pos: proj_y,
            x_velocity: tail.x_velocity,
            lifetime: 120,
            kind: EnemyProjectileKind::LionProjectile,
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
}

fn step_lion_walking_tracking_loc_3e03c(enemy: &mut ActiveEnemy, player_x_pos: Option<u32>) {
    let Some(px) = player_x_pos else { return };
    let lion_x_word = fixed_position_integer_word(enemy.x_pos);
    let kid_x_word = fixed_position_integer_word(px);
    let lion_x = i16::from_ne_bytes(lion_x_word.to_ne_bytes());
    let kid_x = i16::from_ne_bytes(kid_x_word.to_ne_bytes());
    let delta = lion_x.wrapping_sub(kid_x);

    // ASM Lion.asm loc_3E03C: signed `d4 = lion_x - Kid_x` values in
    // `(-$40..=$40)` skip tracking and fall straight into loc_3E078.
    if delta <= 0x40 && delta > -0x40 {
        return;
    }

    match lion_tracking_gate_loc_3e054(lion_x_word, kid_x_word, enemy.scorpion_tracking_lock) {
        LionTrackingGate::Hold => {}
        LionTrackingGate::ClearLatchAndTrackKid => {
            let cleared = lion_clear_tracking_latch_loc_3e066(lion_x_word, kid_x_word);
            enemy.scorpion_tracking_lock = cleared.range_latch_set;
            enemy.facing = cleared.facing;
        }
        LionTrackingGate::TrackKid => {
            enemy.facing = lion_clear_tracking_latch_loc_3e066(lion_x_word, kid_x_word).facing;
        }
    }
}

fn lion_live_walk_animation_cycle_loc_3dfd8(hit_points: u8) -> u8 {
    // ASM Lion.asm loc_3DFD8 selects `stru_3DEDA` for every HP value except
    // exactly 8; HP 8 selects `stru_3DEF4`. The byte counts below are the
    // summed `anim_frame` durations from ingame/anim/enemy/Lion.asm.
    if hit_points == 8 { 27 } else { 38 }
}

fn reset_lion_live_walk_animation_loc_3dff0(enemy: &mut ActiveEnemy) {
    // ASM Init_Animation (kid.asm sub_1040) installs the first sprite and
    // starts the script timer from the beginning. Native object snapshots are
    // post-update while Mode_Level loc_901 presents the previous sprite table;
    // the visible resumed charge body in issue #289 matches phase 29 of the
    // restored `stru_3DEDA` duration cursor.
    enemy.dragon_script_cmd = 29 % lion_live_walk_animation_cycle_loc_3dfd8(enemy.hit_points);
}

fn advance_lion_live_walk_animation_loc_3dfd8(enemy: &mut ActiveEnemy) {
    let cycle = lion_live_walk_animation_cycle_loc_3dfd8(enemy.hit_points);
    enemy.dragon_script_cmd = enemy.dragon_script_cmd.wrapping_add(1) % cycle;
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
            step_lion_walking_tracking_loc_3e03c(enemy, player_x_pos);

            // Count down charge timer. ASM Lion.asm loc_3E03C only resolves
            // facing/tracking before falling into loc_3E078, where the timer
            // always ticks and the charge fires unconditionally on expiry.
            if enemy.wait_timer > 0 {
                enemy.wait_timer -= 1;
                advance_lion_live_walk_animation_loc_3dfd8(enemy);
                return;
            }

            // ASM loc_3E078: after reloading $54(a5) to $B4, a live projectile
            // count in $5B(a5) suppresses the next roar/fire cycle.
            if lion_active_projectile_count(enemy) > 0 {
                enemy.wait_timer = LION_COOLDOWN_AFTER_FIRE;
                advance_lion_live_walk_animation_loc_3dfd8(enemy);
                return;
            }

            // ASM Lion.asm loc_3E078 starts `stru_3DF0E`; loc_3E09E
            // hibernates until the fire sprite before allocating the shot.
            enemy.x_velocity = 0;
            enemy.y_velocity = 0;
            enemy.lion_state = LionState::Roaring;
            enemy.wait_timer = LION_ROAR_FIRE_DELAY;
        }
        LionState::Roaring => {
            if enemy.on_ground {
                enemy.x_velocity = 0;
                enemy.y_velocity = 0;
            } else {
                // ASM Lion.asm loc_3E09E hibernates the object while the roar
                // animation advances; because Lion projectiles set the parent
                // up through the shared movement flags, the off-ledge fall can
                // continue during this wait.
                step_dragon_walking_airborne(enemy, block_grid);
            }
            if enemy.wait_timer > 0 {
                enemy.wait_timer -= 1;
                return;
            }

            let Some(px) = player_x_pos else {
                enemy.lion_state = LionState::Charging;
                reset_lion_live_walk_animation_loc_3dff0(enemy);
                advance_lion_live_walk_animation_loc_3dfd8(enemy);
                return;
            };
            fire_lion_projectile_loc_3e09e(enemy, px, projectiles);
            enemy.lion_state = LionState::PostFire;
            enemy.wait_timer = LION_POST_FIRE_RESUME_DELAY;
        }
        LionState::PostFire => {
            if enemy.on_ground {
                enemy.x_velocity = 0;
                enemy.y_velocity = 0;
            } else {
                // ASM Lion.asm loc_3E0F4 is the post-fire hibernate loop; keep
                // the shared airborne movement active until floor contact.
                step_dragon_walking_airborne(enemy, block_grid);
            }
            if enemy.wait_timer > 0 {
                enemy.wait_timer -= 1;
                return;
            }
            // ASM Lion.asm loc_3E0F4 restores `$62(a5)` before dispatching to
            // loc_3E11E's direction-specific charge branch.
            reset_lion_live_walk_animation_loc_3dff0(enemy);
            enemy.lion_state = LionState::ResumingCharge;
        }
        LionState::Charging => {
            if enemy.wait_timer > 0 {
                enemy.wait_timer -= 1;
            }

            if enemy.wait_timer == 0 {
                if lion_active_projectile_count(enemy) > 0 {
                    enemy.wait_timer = LION_COOLDOWN_AFTER_FIRE;
                    advance_lion_live_walk_animation_loc_3dfd8(enemy);
                } else {
                    enemy.lion_state = LionState::Roaring;
                    enemy.wait_timer = LION_ROAR_FIRE_DELAY;
                }
                return;
            }

            step_lion_charge_movement_loc_3e11e(enemy, block_grid, player_x_pos);
            advance_lion_live_walk_animation_loc_3dfd8(enemy);
        }
        LionState::ResumingCharge => {
            // ASM Lion.asm loc_3E0F4 falls directly into loc_3E11E after the
            // fire animation finishes. It must not pass through loc_3E078's
            // `$54/$5B` roar/cooldown gate, or the Level 0x07 charge body
            // contact happens several frames late.
            step_lion_charge_movement_loc_3e11e(enemy, block_grid, player_x_pos);
            advance_lion_live_walk_animation_loc_3dfd8(enemy);
        }
    }
}

fn step_lion_charge_movement_loc_3e11e(
    enemy: &mut ActiveEnemy,
    block_grid: Option<&terrain::BlockGrid>,
    player_x_pos: Option<u32>,
) {
    if !enemy.on_ground {
        if let Some(grid) = block_grid {
            let center_x_px = (enemy.x_pos >> 16) as i32;
            let foot_y_px = (enemy.y_pos >> 16) as i32;
            let scan = match enemy.facing {
                Direction::Right => terrain::horizontal_collision_scan_right_sub_36B3C(
                    grid,
                    center_x_px,
                    foot_y_px,
                    LION_CHARGE_PROBE_HALF_WIDTH,
                    LION_CHARGE_PROBE_HEIGHT,
                ),
                Direction::Left => terrain::horizontal_collision_scan_left_sub_36BD6(
                    grid,
                    center_x_px,
                    foot_y_px,
                    LION_CHARGE_PROBE_HALF_WIDTH,
                    LION_CHARGE_PROBE_HEIGHT,
                ),
            };

            let floor = terrain::probe_floor_hitbox(
                grid,
                center_x_px,
                foot_y_px,
                LION_CHARGE_PROBE_HALF_WIDTH,
            );
            let floor_edge_overlap =
                floor.on_ground && (floor.floor_y_px..=floor.floor_y_px + 3).contains(&foot_y_px);
            if scan.hitbox_blocked && !floor_edge_overlap {
                // ASM Lion.asm loc_3E6AE/loc_3E7CA: after sub_36B3C/sub_36BD6,
                // non-zero d5 branches to loc_3E94E and stores collision_type=$FFFF.
                // Native can transiently carry `on_ground=false` while the
                // feet are already snapped onto the floor row; do not treat
                // that floor edge as the body-overlap collision.
                register_enemy_terrain_kill(enemy);
                enemy.shared_walker_debug_branch = Some("lion_charge_airborne_terrain_kill");
                return;
            }
            if let Some(snapped_x) = lion_airborne_charge_lead_column_snap_loc_3e6ae_3e7ca(
                enemy.x_pos,
                enemy.x_velocity,
                enemy.facing,
                scan.lead_column_blocked,
            ) {
                // ASM Lion.asm loc_3E6AE/loc_3E7CA: a blocked lead column only
                // stops X after the low-nibble boundary crossing test succeeds.
                // The generic Dragon/Lion wall probe cannot see the ROM's d6=1
                // level-boundary sentinel, so consume sub_36B3C/sub_36BD6 here
                // before the shared vertical fall tail.
                enemy.x_pos = snapped_x;
                enemy.x_velocity = 0;
                step_lion_charge_airborne_vertical_loc_3e6f6_3e812(enemy, grid);
                enemy.shared_walker_debug_branch = Some("lion_charge_airborne_lead_column_snap");
                return;
            }
        }
        // ASM Lion.asm loc_3E6AE/loc_3E77E and loc_3E7CA/loc_3E89A
        // share the Dragon/Lion airborne walker. The `$54(a5)` roar
        // timer keeps ageing, but native defers the stationary
        // loc_3E078 roar/fire transition until floor contact restores
        // the grounded charge route.
        enemy.x_pos = enemy.x_pos.wrapping_add_signed(enemy.x_velocity);
        if let Some(grid) = block_grid {
            step_lion_charge_airborne_vertical_loc_3e6f6_3e812(enemy, grid);
        } else {
            step_dragon_walking_airborne_after_x_commit(enemy, block_grid);
        }
        return;
    }

    let speed = lion_charge_speed(enemy.hit_points);
    let walk_delta = horizontal_velocity(enemy.facing, speed);
    let moving_right = walk_delta >= 0;

    let next_x = (enemy.x_pos as i64 + i64::from(walk_delta)) as u32;

    if let Some(grid) = block_grid {
        let current_x_px = (enemy.x_pos >> 16) as i32;
        let next_x_px = (next_x >> 16) as i32;
        let current_y_px = (enemy.y_pos >> 16) as i32;
        let wall =
            dragon_walking_probe_wall(grid, current_x_px, next_x_px, current_y_px, moving_right);
        let floor_support = dragon_walking_floor_support(grid, next_x_px, current_y_px);
        enemy.shared_walker_terrain_trace = SharedWalkerTerrainTrace {
            next_x_px: Some(next_x_px),
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
            enemy.lion_state = LionState::Walking;
            enemy.wait_timer = LION_COOLDOWN_AFTER_FIRE;
            // ASM Lion.asm loc_3E48E/loc_3E642: charge exits set `$4E(a5)`.
            enemy.scorpion_tracking_lock = true;
            return;
        }

        let floor = floor_support.selected;
        if !floor.on_ground {
            // ORIGINAL_QUIRK: Lion.asm loc_3E682/loc_3E79E does not
            // turn around at ledges. It latches the airborne charge,
            // keeps `$50(a5)` as x_vel, nudges y down one pixel, then
            // falls through loc_3E6AE/loc_3E7CA until terrain catches it.
            enemy.on_ground = false;
            enemy.x_velocity = walk_delta;
            enemy.y_velocity = 0;
            enemy.y_pos = enemy.y_pos.saturating_add(1 << 16);
            enemy.shared_walker_debug_branch = Some("lion_charge_airborne_start");
            return;
        }

        enemy.y_pos = (floor.floor_y_px as u32) << 16;
    }

    if let Some(px) = player_x_pos {
        let lion_x_word = fixed_integer_word(enemy.x_pos) as u16;
        let kid_x_word = fixed_integer_word(px) as u16;
        let window = lion_charge_window_gate_loc_3e434(enemy.shared_walker_state);
        enemy.shared_walker_state = window.next_shared_state;
        match window.branch {
            LionLoc3E434Branch::KeepAndContinue => {}
            LionLoc3E434Branch::Check60Band => {
                match lion_left_charge_calm_distance_gate_loc_3e604(lion_x_word, kid_x_word) {
                    LionLoc3E604Branch::ContinueMovement => {}
                    LionLoc3E604Branch::EnterLoc3E4CE => {
                        // ASM Lion.asm loc_3E450 -> loc_3E4CE: Kid inside the
                        // calm +$60 band sets right-facing alert state before
                        // loc_3E4C6 can commit the candidate X position.
                        enemy.facing = Direction::Right;
                        enemy.shared_walker_state = SHARED_WALKER_STATE_FACE_RIGHT;
                        enemy.x_velocity = 0;
                        enemy.y_velocity = 0;
                        enemy.shared_walker_debug_branch = Some("lion_charge_calm_alert_right");
                        return;
                    }
                    LionLoc3E604Branch::EnterLoc3E50A => {
                        // ASM Lion.asm loc_3E450 -> loc_3E50A mirrors the
                        // calm-band alert state for Kid at or left of Lion.
                        enemy.facing = Direction::Left;
                        enemy.shared_walker_state = SHARED_WALKER_STATE_FACE_LEFT;
                        enemy.x_velocity = 0;
                        enemy.y_velocity = 0;
                        enemy.shared_walker_debug_branch = Some("lion_charge_calm_alert_left");
                        return;
                    }
                }
            }
            LionLoc3E434Branch::Check70Band => {
                match lion_left_charge_alert_release_gate_loc_3e62a(lion_x_word, kid_x_word) {
                    LionLoc3E62ABranch::ContinueMovement => {}
                    LionLoc3E62ABranch::EnterLoc3E642 => {
                        // ASM Lion.asm loc_3E476/loc_3E62A -> loc_3E48E/
                        // loc_3E642: leaving the alert band drops back to
                        // walking, latches `$4E(a5)`, and stores `$42=$FF88`
                        // for non-patrol alert states.
                        enemy.lion_state = LionState::Walking;
                        enemy.wait_timer = LION_COOLDOWN_AFTER_FIRE;
                        enemy.scorpion_tracking_lock = true;
                        enemy.shared_walker_state = -0x78;
                        enemy.x_velocity = 0;
                        enemy.y_velocity = 0;
                        enemy.shared_walker_debug_branch = Some("lion_charge_alert_release");
                        return;
                    }
                }
            }
        }
    }

    enemy.x_pos = next_x;
    enemy.x_velocity = 0;
    enemy.y_velocity = 0;
    enemy.shared_walker_debug_branch = Some("lion_charge_grounded");
}

fn lion_airborne_charge_lead_column_snap_loc_3e6ae_3e7ca(
    x_pos: u32,
    x_velocity: i32,
    facing: Direction,
    lead_column_blocked: bool,
) -> Option<u32> {
    if !lead_column_blocked || x_velocity == 0 {
        return None;
    }

    let current_px = fixed_integer_word(x_pos) as i32;
    let candidate_x = x_pos.wrapping_add_signed(x_velocity);
    let candidate_px = fixed_integer_word(candidate_x) as i32;

    match facing {
        Direction::Right => {
            let moved_px = candidate_px.wrapping_sub(current_px);
            if moved_px <= 0 {
                return None;
            }
            // ASM Lion.asm loc_3E6AE: `d3 = (-(x_pos + $48(a5))) & $F`;
            // `cmp.w d1,d3; bcc loc_3E6F0`; only `d1 > d3` and d6!=0 snaps.
            let boundary_px = (-(current_px + LION_CHARGE_PROBE_HALF_WIDTH)).rem_euclid(16);
            (moved_px > boundary_px).then_some(((current_px + boundary_px) as u32) << 16)
        }
        Direction::Left => {
            let moved_px = current_px.wrapping_sub(candidate_px);
            if moved_px <= 0 {
                return None;
            }
            // ASM Lion.asm loc_3E7CA: mirror using
            // `d3 = (x_pos - $48(a5)) & $F`, snapping to `x_pos - d3`.
            let boundary_px = (current_px - LION_CHARGE_PROBE_HALF_WIDTH).rem_euclid(16);
            (moved_px > boundary_px).then_some(((current_px - boundary_px) as u32) << 16)
        }
    }
}

fn step_lion_charge_airborne_vertical_loc_3e6f6_3e812(
    enemy: &mut ActiveEnemy,
    grid: &terrain::BlockGrid,
) {
    // ASM Lion.asm loc_3E6F6/loc_3E812: after the X commit, add y_vel to
    // y_pos and branch by signed integer-pixel delta. Both directions share
    // the same sub_36CB8 ceiling and sub_36C6A floor scans using `$48/$4A`.
    match lion_left_charge_vertical_entry_loc_3e6f6(
        enemy.y_pos,
        enemy.y_velocity,
        LION_CHARGE_PROBE_HEIGHT as u16,
    ) {
        LionLeftChargeVerticalEntryLoc3E6F6::UpwardCollisionCheck {
            candidate_y_position,
            needs_ceiling_probe,
            ceiling_clearance_pixels,
            ..
        } => {
            let ceiling_hit = needs_ceiling_probe
                && terrain::vertical_collision_scan_up_sub_36CB8(
                    grid,
                    (enemy.x_pos >> 16) as i32,
                    (enemy.y_pos >> 16) as i32,
                    LION_CHARGE_PROBE_HALF_WIDTH,
                    LION_CHARGE_PROBE_HEIGHT,
                );
            if ceiling_hit {
                let current_y_word = fixed_integer_word(enemy.y_pos) as u16;
                enemy.y_pos =
                    u32::from(current_y_word.wrapping_sub(ceiling_clearance_pixels)) << 16;
                enemy.y_velocity = 0;
                enemy.shared_walker_debug_branch = Some("lion_charge_airborne_ceiling_snap");
            } else {
                enemy.y_pos = candidate_y_position;
            }
        }
        LionLeftChargeVerticalEntryLoc3E6F6::DownwardOrStationary {
            candidate_y_position,
            pixel_delta,
        } => {
            let downward_pixels = pixel_delta as u16;
            let floor_hit = terrain::vertical_collision_scan_down_sub_36C6A(
                grid,
                (enemy.x_pos >> 16) as i32,
                (enemy.y_pos >> 16) as i32,
                LION_CHARGE_PROBE_HALF_WIDTH,
            );
            match lion_left_charge_downward_collision_loc_3e732(
                enemy.y_pos,
                candidate_y_position,
                downward_pixels,
                floor_hit,
                enemy.shared_walker_state,
            ) {
                LionLeftChargeDownwardCollisionLoc3E732::Landed {
                    y_position,
                    y_velocity,
                    next_shared_walker_state,
                    ..
                } => {
                    enemy.y_pos = y_position;
                    enemy.y_velocity = y_velocity;
                    enemy.x_velocity = 0;
                    enemy.on_ground = true;
                    if let Some(next_state) = next_shared_walker_state {
                        enemy.shared_walker_state = next_state;
                    }
                    enemy.shared_walker_debug_branch = Some("lion_charge_airborne_land");
                    return;
                }
                LionLeftChargeDownwardCollisionLoc3E732::Airborne { y_position, .. } => {
                    enemy.y_pos = y_position;
                }
            }
        }
    }

    enemy.y_velocity = lion_left_charge_gravity_tail_loc_3e77e(enemy.y_velocity).y_velocity;
    enemy.on_ground = false;
    if enemy.shared_walker_debug_branch != Some("lion_charge_airborne_ceiling_snap") {
        enemy.shared_walker_debug_branch = Some("lion_charge_airborne_fall");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::enemies::test_fixtures::*;

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
            lion_init_visible_state_loc_3df92(0, 0x0200, 0x0200).facing,
            Direction::Right,
            "ASM Lion.asm loc_3DF92 should set x_direction when the signed spawn-facing compare is equal"
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
        assert_eq!(
            lion_clear_tracking_latch_loc_3e066(0x0200, 0x0200),
            LionLatchClearResult {
                range_latch_set: false,
                facing: Direction::Left,
            },
            "ASM Lion.asm loc_3E066 should treat zero d4 as the non-negative clear-x_direction path"
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
        assert_eq!(
            lion_track_kid_negative_delta_facing_loc_3e074(0x0200, 0x0200),
            Direction::Left,
            "ASM Lion.asm loc_3E074 should not take the negative branch when d4 is zero"
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
        assert_eq!(
            lion_roar_wait_dispatch_loc_3e09e(0, true, 0x0200, 0x0200),
            LionRoarWaitDispatch::ReadyToFire {
                next_collision_type: 0,
                facing: Direction::Left,
                projectile_worker_selector: 0x8000,
                projectile_worker: "LionProjectileWorker",
                active_projectile_count_delta: 1,
            },
            "ASM Lion.asm loc_3E09E should keep clear x_direction when Kid and Lion share the same x word"
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
    fn lion_post_fire_resume_exposes_walk_phase_loc_3e0f4() {
        let mut enemy = mini_hopping_skull_test_enemy();
        enemy.family = EnemyFamily::Lion;
        enemy.hit_points = 6;
        enemy.lion_state = LionState::PostFire;
        enemy.wait_timer = 0;
        enemy.dragon_script_cmd = 0;

        let mut projectiles = None;
        step_lion(&mut enemy, None, Some(0), &mut projectiles);

        assert_eq!(enemy.lion_state, LionState::ResumingCharge);
        assert_eq!(
            enemy.dragon_script_cmd, 29,
            "ASM Lion.asm loc_3E0F4 restores `$62(a5)` before loc_3E11E; issue #289 oracle frames show the resumed HP-6 `stru_3DEDA` body at phase 29"
        );
        assert_eq!(
            enemy.snapshot().walk_phase,
            Some(29),
            "Lion snapshots should expose the live walk phase only after the `loc_3E0F4` restore resumes the charge script"
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
    fn lion_projectile_left_launch_offsets_from_native_carrier_loc_3e12a() {
        assert_eq!(
            lion_projectile_x_offset_from_native_carrier_loc_3e12a(Direction::Left),
            -22,
            "ASM Lion.asm loc_3E12A copies parent GfxObject x, not native's carrier x; issue #289 oracle frame 19 needs native carrier x - 22 before the first movement step reaches @501"
        );
    }

    #[test]
    fn lion_roar_fire_delay_matches_stru_3df0e_fire_sprite_loc_3e09e() {
        assert_eq!(
            LION_ROAR_FIRE_DELAY, 42,
            "ASM ingame/anim/enemy/Lion.asm stru_3DF0E has seven 6-tick frames before Lion.asm loc_3E09E sees LnkTo_unk_C7E78"
        );
    }

    #[test]
    fn lion_fire_projectile_wires_right_facing_child_loc_3e09e_3e12a() {
        let mut lion = ActiveEnemy {
            family: EnemyFamily::Lion,
            x_pos: 0x0100 << 16,
            y_pos: 0x0080 << 16,
            facing: Direction::Left,
            ufo_beam_segments: 2,
            ufo_beam_interval: 0x5A,
            segment_index: 3,
            layout_index: 7,
            ..mini_hopping_skull_test_enemy()
        };
        let mut projectiles = Vec::new();

        {
            let mut projectile_sink = Some(&mut projectiles);
            fire_lion_projectile_loc_3e09e(&mut lion, 0x0120 << 16, &mut projectile_sink);
        }

        assert_eq!(
            lion.facing,
            Direction::Right,
            "ASM Lion.asm loc_3E09E should set x_direction when Kid is to the right before allocating loc_3E12A"
        );
        assert_eq!(
            lion.ufo_beam_segments, 3,
            "ASM Lion.asm loc_3E0DA increments `$5B(a5)` after installing the projectile worker"
        );
        assert_eq!(
            projectiles,
            vec![EnemyProjectile {
                x_pos: 0x010A << 16,
                y_pos: 0x0068 << 16,
                x_velocity: LION_PROJECTILE_SPEED,
                lifetime: 120,
                kind: EnemyProjectileKind::LionProjectile,
                homing_accel: 0x2000,
                speed_cap: LION_PROJECTILE_SPEED,
                wall_bounce: true,
                owner: Some(EnemyProjectileOwner {
                    segment_index: 3,
                    layout_index: 7,
                    token: 0x5A,
                }),
                ..Default::default()
            }],
            "ASM Lion.asm loc_3E12A/loc_3E180 should spawn the right-facing homing roar child with the parent identity"
        );
    }

    #[test]
    fn lion_fire_projectile_wires_left_facing_child_loc_3e09e_3e12a() {
        let mut lion = ActiveEnemy {
            family: EnemyFamily::Lion,
            x_pos: 0x0100 << 16,
            y_pos: 0x0080 << 16,
            facing: Direction::Right,
            ufo_beam_segments: 0,
            ufo_beam_interval: 0x2C,
            segment_index: 1,
            layout_index: 4,
            ..mini_hopping_skull_test_enemy()
        };
        let mut projectiles = Vec::new();

        {
            let mut projectile_sink = Some(&mut projectiles);
            fire_lion_projectile_loc_3e09e(&mut lion, 0x00F0 << 16, &mut projectile_sink);
        }

        assert_eq!(
            lion.facing,
            Direction::Left,
            "ASM Lion.asm loc_3E09E should clear x_direction when Kid is not to the right"
        );
        assert_eq!(
            lion.ufo_beam_segments, 1,
            "ASM Lion.asm loc_3E0DA increments `$5B(a5)` for the left-facing projectile too"
        );
        assert_eq!(
            projectiles,
            vec![EnemyProjectile {
                x_pos: 0x00EA << 16,
                y_pos: 0x0068 << 16,
                x_velocity: -LION_PROJECTILE_SPEED,
                lifetime: 120,
                kind: EnemyProjectileKind::LionProjectile,
                homing_accel: 0x2000,
                speed_cap: LION_PROJECTILE_SPEED,
                wall_bounce: true,
                owner: Some(EnemyProjectileOwner {
                    segment_index: 1,
                    layout_index: 4,
                    token: 0x2C,
                }),
                ..Default::default()
            }],
            "ASM Lion.asm loc_3E12A copies parent GfxObject x; native's left carrier correction keeps the shot at x-22"
        );
    }

    #[test]
    fn lion_walking_tracking_edges_match_loc_3e03c() {
        let mut lion = ActiveEnemy {
            family: EnemyFamily::Lion,
            x_pos: 100 << 16,
            facing: Direction::Left,
            scorpion_tracking_lock: false,
            ..mini_hopping_skull_test_enemy()
        };

        step_lion_walking_tracking_loc_3e03c(&mut lion, Some((100 + 0x3F) << 16));
        assert_eq!(
            lion.facing,
            Direction::Left,
            "ASM Lion.asm loc_3E03C should skip tracking while signed d4 is inside (-$40..=$40)"
        );

        step_lion_walking_tracking_loc_3e03c(&mut lion, Some((100 + 0x40) << 16));
        assert_eq!(
            lion.facing,
            Direction::Right,
            "ASM Lion.asm loc_3E03C should track at exact d4=-$40 because `cmpi.w #$FFC0,d4; bgt` is false"
        );

        lion.facing = Direction::Right;
        step_lion_walking_tracking_loc_3e03c(&mut lion, Some((100 - 0x41) << 16));
        assert_eq!(
            lion.facing,
            Direction::Left,
            "ASM Lion.asm loc_3E03C should track once positive d4 exceeds +$40"
        );
    }

    #[test]
    fn lion_live_walk_animation_wraps_hp_specific_cycles_loc_3dfd8() {
        let mut fast_lion = ActiveEnemy {
            family: EnemyFamily::Lion,
            hit_points: 8,
            dragon_script_cmd: 26,
            ..mini_hopping_skull_test_enemy()
        };
        advance_lion_live_walk_animation_loc_3dfd8(&mut fast_lion);
        assert_eq!(
            fast_lion.dragon_script_cmd, 0,
            "ASM ingame/anim/enemy/Lion.asm stru_3DEF4 sums to 27 ticks for HP 8"
        );

        let mut regular_lion = ActiveEnemy {
            family: EnemyFamily::Lion,
            hit_points: 6,
            dragon_script_cmd: 37,
            ..mini_hopping_skull_test_enemy()
        };
        advance_lion_live_walk_animation_loc_3dfd8(&mut regular_lion);
        assert_eq!(
            regular_lion.dragon_script_cmd, 0,
            "ASM ingame/anim/enemy/Lion.asm stru_3DEDA sums to 38 ticks for non-8 HP"
        );
    }

    #[test]
    fn lion_walking_live_projectile_suppresses_roar_loc_3e078() {
        let mut lion = ActiveEnemy {
            family: EnemyFamily::Lion,
            lion_state: LionState::Walking,
            on_ground: true,
            wait_timer: 0,
            ufo_beam_segments: 1,
            dragon_script_cmd: 5,
            ..mini_hopping_skull_test_enemy()
        };

        step_lion(&mut lion, None, Some(0x0140 << 16), &mut None);

        assert_eq!(
            lion.lion_state,
            LionState::Walking,
            "ASM Lion.asm loc_3E078 should skip the roar/fire animation while `$5B(a5)` is positive"
        );
        assert_eq!(lion.wait_timer, LION_COOLDOWN_AFTER_FIRE);
        assert_eq!(
            lion.dragon_script_cmd, 6,
            "ASM Lion.asm loc_3E078 falls back into loc_3E11E pacing animation after suppressing the roar"
        );
    }

    #[test]
    fn lion_postfire_wait_timer_holds_until_done_loc_3e0f4() {
        let mut lion = ActiveEnemy {
            family: EnemyFamily::Lion,
            lion_state: LionState::PostFire,
            on_ground: true,
            wait_timer: 1,
            x_velocity: 0x1234,
            y_velocity: -0x1234,
            ..mini_hopping_skull_test_enemy()
        };

        step_lion(&mut lion, None, Some(0x0100 << 16), &mut None);

        assert_eq!(
            lion.lion_state,
            LionState::PostFire,
            "ASM Lion.asm loc_3E0F4 should keep hibernating while `$18(a3)` is clear"
        );
        assert_eq!(lion.wait_timer, 0);
        assert_eq!(lion.x_velocity, 0);
        assert_eq!(lion.y_velocity, 0);
    }

    #[test]
    fn lion_charging_timer_decrements_before_movement_loc_3e078_3e11e() {
        let mut lion = ActiveEnemy {
            family: EnemyFamily::Lion,
            lion_state: LionState::Charging,
            on_ground: true,
            wait_timer: 2,
            facing: Direction::Right,
            x_pos: 0x0100 << 16,
            ..mini_hopping_skull_test_enemy()
        };

        step_lion(&mut lion, None, None, &mut None);

        assert_eq!(
            lion.wait_timer, 1,
            "ASM Lion.asm loc_3E078 should decrement the live charge timer before continuing through loc_3E11E"
        );
        assert_eq!(
            lion.x_pos,
            (0x0100 << 16) + lion_charge_speed(lion.hit_points) as u32,
            "ASM Lion.asm loc_3E11E should still commit movement after a non-zero charge timer tick"
        );
    }

    #[test]
    fn lion_charging_timer_expiry_respects_live_projectile_gate_loc_3e078() {
        let mut lion = ActiveEnemy {
            family: EnemyFamily::Lion,
            lion_state: LionState::Charging,
            on_ground: true,
            wait_timer: 1,
            ufo_beam_segments: 1,
            dragon_script_cmd: 3,
            ..mini_hopping_skull_test_enemy()
        };

        step_lion(&mut lion, None, Some(0x0100 << 16), &mut None);

        assert_eq!(
            lion.lion_state,
            LionState::Charging,
            "ASM Lion.asm loc_3E078 should remain in the charge loop when a live shot suppresses the next roar"
        );
        assert_eq!(lion.wait_timer, LION_COOLDOWN_AFTER_FIRE);
        assert_eq!(
            lion.dragon_script_cmd, 4,
            "ASM Lion.asm loc_3E078 should advance the live walk animation on the suppressed-roar frame"
        );
    }

    #[test]
    fn lion_charging_zero_timer_without_projectile_enters_roar_loc_3e078() {
        let mut lion = ActiveEnemy {
            family: EnemyFamily::Lion,
            lion_state: LionState::Charging,
            on_ground: true,
            wait_timer: 0,
            ufo_beam_segments: 0,
            ..mini_hopping_skull_test_enemy()
        };

        step_lion(&mut lion, None, Some(0x0100 << 16), &mut None);

        assert_eq!(
            lion.lion_state,
            LionState::Roaring,
            "ASM Lion.asm loc_3E078 should start `stru_3DF0E` when the timer is zero and `$5B(a5)` is clear"
        );
        assert_eq!(lion.wait_timer, LION_ROAR_FIRE_DELAY);
    }

    #[test]
    fn lion_grounded_charge_commits_next_x_without_grid_loc_3e4c6_3e67a() {
        let mut lion = ActiveEnemy {
            family: EnemyFamily::Lion,
            lion_state: LionState::Charging,
            on_ground: true,
            facing: Direction::Right,
            hit_points: 6,
            x_pos: 0x0100 << 16,
            ..mini_hopping_skull_test_enemy()
        };

        step_lion_charge_movement_loc_3e11e(&mut lion, None, None);

        assert_eq!(
            lion.x_pos,
            (0x0100 << 16) + 0x0001_8000,
            "ASM Lion.asm loc_3E4C6/loc_3E67A should commit d2 = x_pos + charge speed"
        );
        assert_eq!(lion.x_velocity, 0);
        assert_eq!(lion.y_velocity, 0);
    }

    #[test]
    fn lion_grounded_charge_enters_airborne_at_ledge_loc_3e682_3e694() {
        let grid = terrain::BlockGrid::from_level_layout(&[0; 8 * 4 * 2], 8, 4);
        let mut lion = ActiveEnemy {
            family: EnemyFamily::Lion,
            lion_state: LionState::Charging,
            on_ground: true,
            facing: Direction::Right,
            hit_points: 6,
            x_pos: 0x0020 << 16,
            y_pos: 0x0020 << 16,
            ..mini_hopping_skull_test_enemy()
        };

        step_lion_charge_movement_loc_3e11e(&mut lion, Some(&grid), None);

        assert!(
            !lion.on_ground,
            "ASM Lion.asm loc_3E682 should latch the airborne charge instead of turning around at ledges"
        );
        assert_eq!(lion.x_velocity, 0x0001_8000);
        assert_eq!(lion.y_velocity, 0);
        assert_eq!(
            lion.y_pos,
            0x0021 << 16,
            "ASM Lion.asm loc_3E694 should add one pixel to y_pos before entering loc_3E6AE"
        );
        assert_eq!(
            lion.shared_walker_debug_branch,
            Some("lion_charge_airborne_start")
        );
    }

    #[test]
    fn lion_grounded_charge_snaps_to_floor_support_loc_3e11e() {
        let grid = mini_hopping_skull_test_grid();
        let mut lion = ActiveEnemy {
            family: EnemyFamily::Lion,
            lion_state: LionState::Charging,
            on_ground: true,
            facing: Direction::Right,
            hit_points: 6,
            x_pos: 0x0020 << 16,
            y_pos: 0x001F << 16,
            ..mini_hopping_skull_test_enemy()
        };

        step_lion_charge_movement_loc_3e11e(&mut lion, Some(&grid), None);

        assert_eq!(
            lion.y_pos,
            0x0020 << 16,
            "ASM shared Lion/Dragon grounded walker should snap y_pos to the selected floor support before committing charge movement"
        );
        assert!(lion.on_ground);
        assert_eq!(
            lion.shared_walker_debug_branch,
            Some("lion_charge_grounded")
        );
    }

    #[test]
    fn lion_grounded_left_charge_slope_boundary_uses_current_x_loc_3e566() {
        let mut grid = terrain::BlockGrid::from_level_layout(&[0; 4 * 4 * 2], 4, 4);
        grid.set_cell_at(1, 2, terrain::Cell::from_be_bytes(0x40, 0x00));
        let mut lion = ActiveEnemy {
            family: EnemyFamily::Lion,
            lion_state: LionState::Charging,
            on_ground: true,
            facing: Direction::Left,
            hit_points: 6,
            x_pos: 41 << 16,
            y_pos: 32 << 16,
            ..mini_hopping_skull_test_enemy()
        };

        step_lion_charge_movement_loc_3e11e(&mut lion, Some(&grid), None);

        let wall = lion
            .shared_walker_terrain_trace
            .wall
            .expect("grounded Lion charge should record the wall probe");
        assert!(
            wall.hit,
            "ASM Lion.asm loc_3E566 -> sub_36A58 should treat the crossed left SlopeUp foot-row boundary as a wall"
        );
        assert_eq!(wall.cell_x, 1);
        assert_eq!(wall.cell_y, 2);
        assert_eq!(
            lion.lion_state,
            LionState::Walking,
            "ASM Lion.asm loc_3E5BA/loc_3E642 should leave charge movement after the left slope boundary hit"
        );
        assert_eq!(
            lion.x_pos,
            41 << 16,
            "ASM shared walker left-wall snap should place the Lion one half-width past the slope cell edge"
        );
        assert_eq!(lion.wait_timer, LION_COOLDOWN_AFTER_FIRE);
        assert!(lion.scorpion_tracking_lock);
    }

    #[test]
    fn lion_grounded_charge_alert_release_sets_ff88_loc_3e48e_3e642() {
        let mut lion = ActiveEnemy {
            family: EnemyFamily::Lion,
            lion_state: LionState::Charging,
            on_ground: true,
            facing: Direction::Right,
            shared_walker_state: SHARED_WALKER_STATE_FACE_RIGHT,
            wait_timer: 12,
            x_pos: 0x0100 << 16,
            ..mini_hopping_skull_test_enemy()
        };

        step_lion_charge_movement_loc_3e11e(&mut lion, None, Some(0x0080 << 16));

        assert_eq!(
            lion.lion_state,
            LionState::Walking,
            "ASM Lion.asm loc_3E476 -> loc_3E48E should leave the charge when Kid exits the alert band"
        );
        assert_eq!(lion.wait_timer, LION_COOLDOWN_AFTER_FIRE);
        assert!(lion.scorpion_tracking_lock);
        assert_eq!(
            lion.shared_walker_state, -0x78,
            "ASM Lion.asm loc_3E48E/loc_3E642 should write `$FF88` into `$42(a3)` for non-patrol alert states"
        );
    }

    #[test]
    fn lion_airborne_charge_deep_floor_overlap_still_kills_loc_3e6ae() {
        let mut grid = terrain::BlockGrid::from_level_layout(&[0; 6 * 6 * 2], 6, 6);
        grid.set_cell_at(1, 1, terrain::Cell::from_be_bytes(0x40, 0x00));
        grid.set_cell_at(1, 2, terrain::Cell::from_be_bytes(0x60, 0x00));
        let mut lion = ActiveEnemy {
            family: EnemyFamily::Lion,
            x_pos: 24 << 16,
            y_pos: 40 << 16,
            x_velocity: 0x0001_0000,
            y_velocity: 0,
            facing: Direction::Right,
            on_ground: false,
            hit_points: 8,
            lion_state: LionState::ResumingCharge,
            wait_timer: 0,
            ..mini_hopping_skull_test_enemy()
        };
        let mut projectiles = Vec::new();
        let mut projectile_sink = Some(&mut projectiles);

        step_lion(&mut lion, Some(&grid), None, &mut projectile_sink);

        assert!(
            lion.death_animation_active,
            "ASM Lion.asm loc_3E6AE should still branch to loc_3E94E when the body overlaps terrain and the feet are deeper than the floor-edge tolerance"
        );
        assert_eq!(lion.hit_points, 0);
        assert_eq!(
            lion.shared_walker_debug_branch,
            Some("lion_charge_airborne_terrain_kill")
        );
    }

    #[test]
    fn lion_airborne_lead_column_snap_requires_blocked_motion_past_boundary_loc_3e6ae_3e7ca() {
        assert_eq!(
            lion_airborne_charge_lead_column_snap_loc_3e6ae_3e7ca(
                100 << 16,
                9 << 16,
                Direction::Right,
                false,
            ),
            None,
            "ASM Lion.asm loc_3E6AE should only snap when sub_36B3C reports a blocked lead column"
        );
        assert_eq!(
            lion_airborne_charge_lead_column_snap_loc_3e6ae_3e7ca(
                100 << 16,
                0,
                Direction::Right,
                true,
            ),
            None,
            "ASM Lion.asm loc_3E6AE should not snap without positive horizontal movement"
        );
        assert_eq!(
            lion_airborne_charge_lead_column_snap_loc_3e6ae_3e7ca(
                100 << 16,
                8 << 16,
                Direction::Right,
                true,
            ),
            None,
            "ASM Lion.asm loc_3E6AE uses `bcc`, so exact boundary equality continues to loc_3E6F0"
        );
        assert_eq!(
            lion_airborne_charge_lead_column_snap_loc_3e6ae_3e7ca(
                100 << 16,
                9 << 16,
                Direction::Right,
                true,
            ),
            Some(108 << 16),
            "ASM Lion.asm loc_3E6AE snaps to x+d3 only after the step crosses the boundary"
        );
        assert_eq!(
            lion_airborne_charge_lead_column_snap_loc_3e6ae_3e7ca(
                101 << 16,
                -(1 << 16),
                Direction::Left,
                true,
            ),
            None,
            "ASM Lion.asm loc_3E7CA also lets exact boundary equality continue"
        );
        assert_eq!(
            lion_airborne_charge_lead_column_snap_loc_3e6ae_3e7ca(
                101 << 16,
                -(2 << 16),
                Direction::Left,
                true,
            ),
            Some(100 << 16),
            "ASM Lion.asm loc_3E7CA snaps to x-d3 after crossing the left boundary"
        );
    }

    #[test]
    fn lion_airborne_upward_clearance_skips_ceiling_probe_loc_3e6f6() {
        let mut grid = terrain::BlockGrid::from_level_layout(&[0; 6 * 6 * 2], 6, 6);
        grid.set_cell_at(2, 0, terrain::Cell::from_be_bytes(0x40, 0x00));
        let mut lion = ActiveEnemy {
            family: EnemyFamily::Lion,
            x_pos: 32 << 16,
            y_pos: 65 << 16,
            y_velocity: -0x0002_0000,
            on_ground: false,
            hit_points: 8,
            lion_state: LionState::ResumingCharge,
            ..mini_hopping_skull_test_enemy()
        };

        step_lion_charge_airborne_vertical_loc_3e6f6_3e812(&mut lion, &grid);

        assert_eq!(
            lion.y_pos,
            63 << 16,
            "ASM Lion.asm loc_3E6F6 should skip sub_36CB8 while the upward step is within the nibble clearance"
        );
        assert_eq!(
            lion.y_velocity, -0x0001_C000,
            "ASM Lion.asm loc_3E77E should apply `$4000` gravity after committing the unobstructed upward candidate"
        );
        assert_eq!(
            lion.shared_walker_debug_branch,
            Some("lion_charge_airborne_fall")
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
                branch: LionLoc3E434Branch::KeepAndContinue,
            },
            "ASM Lion.asm loc_3E434 should branch `$42==$0A` directly to loc_3E4C6/loc_3E67A without running the ±$70 release gate"
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
            lion_left_charge_vertical_entry_loc_3e6f6(0x0042_0000, -0x000C_0000, 6),
            LionLeftChargeVerticalEntryLoc3E6F6::UpwardCollisionCheck {
                candidate_y_position: 0x0036_0000,
                upward_pixels: 12,
                ceiling_clearance_pixels: 12,
                needs_ceiling_probe: false,
            },
            "ASM Lion.asm loc_3E6F6 uses `cmp.w d1,d3; bcc.s loc_3E77A`, so exact clearance equality skips the ceiling probe"
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
            lion_left_charge_downward_collision_loc_3e732(0x0042_0000, 0x0050_0000, 14, true, 0,),
            LionLeftChargeDownwardCollisionLoc3E732::Airborne {
                y_position: 0x0050_0000,
                floor_clearance_pixels: 14,
                needs_floor_probe: false,
            },
            "ASM Lion.asm loc_3E732 uses `cmp.w d1,d3; bcc.s loc_3E77A`, so exact floor clearance equality skips the floor probe even if terrain is present"
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
    fn lion_left_charge_landing_resume_matches_loc_3e76e() {
        assert_eq!(
            lion_left_charge_landing_resume_loc_3e76e(),
            LionLeftChargeLandingResumeLoc3E76E {
                restored_animation: "saved_lion_charge_animation",
                next_branch: "loc_3E392",
            },
            "ASM Lion.asm loc_3E76E should init the saved `$62(a5)` animation before branching to loc_3E392"
        );
    }

    #[test]
    fn lion_left_charge_gravity_tail_matches_loc_3e77e() {
        assert_eq!(
            lion_left_charge_gravity_tail_loc_3e77e(0x0002_0000),
            LionLeftChargeGravityTailLoc3E77E {
                y_velocity: 0x0002_4000,
                next_tail: LionLoc3E556Tail {
                    hibernate_one_frame: true,
                    cull_routine: "sub_3E8BA",
                    resume_via_callback: true,
                },
            },
            "ASM Lion.asm loc_3E77E should add `$4000` gravity and branch through loc_3E6A4"
        );
        assert_eq!(
            lion_left_charge_gravity_tail_loc_3e77e(0x0017_D000).y_velocity,
            0x0018_0000,
            "ASM Lion.asm loc_3E77E should clamp signed velocities at `$180000`"
        );
        assert_eq!(
            lion_left_charge_gravity_tail_loc_3e77e(-0x0001_0000).y_velocity,
            -0x0000_C000,
            "ASM Lion.asm loc_3E77E has no upward-speed clamp; signed negative values stay below `$180000`"
        );
    }

    #[test]
    fn lion_left_charge_y_velocity_commit_matches_loc_3e796() {
        assert_eq!(
            lion_left_charge_y_velocity_commit_loc_3e796(0x0012_4000),
            LionLeftChargeGravityTailLoc3E77E {
                y_velocity: 0x0012_4000,
                next_tail: LionLoc3E556Tail {
                    hibernate_one_frame: true,
                    cull_routine: "sub_3E8BA",
                    resume_via_callback: true,
                },
            },
            "ASM Lion.asm loc_3E796 should commit d0 into y_vel and branch to loc_3E6A4"
        );
    }

    #[test]
    fn lion_left_charge_y_commit_matches_loc_3e77a() {
        assert_eq!(
            lion_left_charge_y_commit_loc_3e77a(0x0052_8000, 0x0017_D000),
            LionLeftChargeYCommitLoc3E77A {
                y_position: 0x0052_8000,
                gravity_tail: LionLeftChargeGravityTailLoc3E77E {
                    y_velocity: 0x0018_0000,
                    next_tail: LionLoc3E556Tail {
                        hibernate_one_frame: true,
                        cull_routine: "sub_3E8BA",
                        resume_via_callback: true,
                    },
                },
            },
            "ASM Lion.asm loc_3E77A should commit `d2` into y_pos before falling through to loc_3E77E"
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
    fn lion_charge_collision_reentry_matches_loc_3e79e() {
        assert_eq!(
            lion_charge_collision_reentry_loc_3e79e(0x2340),
            LionChargeCollisionReentryLoc3E79E {
                left_charge_latch_set: true,
                restored_sprite_offset: Some(0x2340),
                is_animated: Some(false),
            },
            "ASM Lion.asm loc_3E79E should set `$5C(a5)`, restore a non-zero `$4A(a3)` sprite offset, and clear animation"
        );
        assert_eq!(
            lion_charge_collision_reentry_loc_3e79e(0),
            LionChargeCollisionReentryLoc3E79E {
                left_charge_latch_set: true,
                restored_sprite_offset: None,
                is_animated: None,
            },
            "ASM Lion.asm loc_3E79E should keep zero `$4A(a3)` from writing sprite offset or animation state"
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
    fn lion_charge_collision_motion_seed_matches_loc_3e7b0() {
        assert_eq!(
            lion_charge_collision_motion_seed_loc_3e7b0(-0x0001_8000, 0x0042_8000),
            LionChargeCollisionMotionSeedLoc3E7B0 {
                x_velocity: -0x0001_8000,
                y_velocity: 0,
                y_position: 0x0043_8000,
            },
            "ASM Lion.asm loc_3E7B0 should restore `$50(a5)` into x_vel, clear y_vel, and add one pixel to y_pos"
        );
        assert_eq!(
            lion_charge_collision_motion_seed_loc_3e7b0(0x0002_8000, 0xFFFF_1234),
            LionChargeCollisionMotionSeedLoc3E7B0 {
                x_velocity: 0x0002_8000,
                y_velocity: 0,
                y_position: 0x0000_1234,
            },
            "ASM Lion.asm loc_3E7B0 should preserve the ROM's 16-bit y_pos word wrapping"
        );
    }

    #[test]
    fn lion_charge_collision_x_commit_matches_loc_3e80c() {
        assert_eq!(
            lion_charge_collision_x_commit_loc_3e80c(0x0455_8000),
            LionChargeCollisionXCommitLoc3E80C {
                x_position: 0x0455_8000,
                reloads_d0_from_x_position: true,
                enters_vertical_motion: true,
            },
            "ASM Lion.asm loc_3E80C should commit unobstructed d2 into x_pos, reload d0, and enter vertical motion"
        );
    }

    #[test]
    fn lion_charge_collision_vertical_entry_matches_loc_3e812() {
        assert_eq!(
            lion_charge_collision_vertical_entry_loc_3e812(0x0042_0000, 0x0001_0000, 6),
            LionLeftChargeVerticalEntryLoc3E6F6::DownwardOrStationary {
                candidate_y_position: 0x0043_0000,
                pixel_delta: 1,
            },
            "ASM Lion.asm loc_3E812 should route non-negative integer Y deltas to the downward/stationary path"
        );
        assert_eq!(
            lion_charge_collision_vertical_entry_loc_3e812(0x0042_0000, -0x0002_0000, 6),
            LionLeftChargeVerticalEntryLoc3E6F6::UpwardCollisionCheck {
                candidate_y_position: 0x0040_0000,
                upward_pixels: 2,
                ceiling_clearance_pixels: 12,
                needs_ceiling_probe: false,
            },
            "ASM Lion.asm loc_3E812 should skip the ceiling probe when block-grid clearance covers upward movement"
        );
        assert_eq!(
            lion_charge_collision_vertical_entry_loc_3e812(0x0042_0000, -0x000D_0000, 6),
            LionLeftChargeVerticalEntryLoc3E6F6::UpwardCollisionCheck {
                candidate_y_position: 0x0035_0000,
                upward_pixels: 13,
                ceiling_clearance_pixels: 12,
                needs_ceiling_probe: true,
            },
            "ASM Lion.asm loc_3E812 should request a ceiling probe when upward movement crosses the nibble clearance"
        );
    }

    #[test]
    fn lion_charge_collision_downward_collision_matches_loc_3e84e() {
        assert_eq!(
            lion_charge_collision_downward_collision_loc_3e84e(
                0x0042_0000,
                0x0043_0000,
                1,
                true,
                0,
            ),
            LionChargeCollisionDownwardCollisionLoc3E84E::Airborne {
                y_position: 0x0043_0000,
                floor_clearance_pixels: 14,
                needs_floor_probe: false,
            },
            "ASM Lion.asm loc_3E84E should skip floor probing when nibble clearance covers downward motion"
        );
        assert_eq!(
            lion_charge_collision_downward_collision_loc_3e84e(
                0x0042_0000,
                0x0050_0000,
                14,
                true,
                0,
            ),
            LionChargeCollisionDownwardCollisionLoc3E84E::Airborne {
                y_position: 0x0050_0000,
                floor_clearance_pixels: 14,
                needs_floor_probe: false,
            },
            "ASM Lion.asm loc_3E84E uses `cmp.w d1,d3; bcc`: equality skips the floor probe"
        );
        assert_eq!(
            lion_charge_collision_downward_collision_loc_3e84e(
                0x0042_0000,
                0x0052_0000,
                16,
                false,
                0,
            ),
            LionChargeCollisionDownwardCollisionLoc3E84E::Airborne {
                y_position: 0x0052_0000,
                floor_clearance_pixels: 14,
                needs_floor_probe: true,
            },
            "ASM Lion.asm loc_3E84E should keep airborne motion when the floor probe misses"
        );
        assert_eq!(
            lion_charge_collision_downward_collision_loc_3e84e(
                0x0042_0000,
                0x0052_0000,
                16,
                true,
                0,
            ),
            LionChargeCollisionDownwardCollisionLoc3E84E::Landed {
                y_position: 0x0050_0000,
                y_velocity: 0,
                left_charge_latch_set: false,
                object_meta_bit7_cleared: true,
                restores_saved_charge_speed: true,
                next_shared_walker_state: Some(-0x78),
                landing_resume: LionChargeCollisionLandingResumeLoc3E88A {
                    restored_animation: "saved_lion_charge_animation",
                    next_branch: "loc_3E546",
                },
            },
            "ASM Lion.asm loc_3E84E should snap to the floor boundary, clear airborne state, restore charge speed, reset non-$0A shared state, and enter loc_3E88A"
        );
        assert_eq!(
            lion_charge_collision_downward_collision_loc_3e84e(
                0x0042_0000,
                0x0052_0000,
                16,
                true,
                0x000A,
            ),
            LionChargeCollisionDownwardCollisionLoc3E84E::Landed {
                y_position: 0x0050_0000,
                y_velocity: 0,
                left_charge_latch_set: false,
                object_meta_bit7_cleared: true,
                restores_saved_charge_speed: true,
                next_shared_walker_state: None,
                landing_resume: LionChargeCollisionLandingResumeLoc3E88A {
                    restored_animation: "saved_lion_charge_animation",
                    next_branch: "loc_3E546",
                },
            },
            "ASM Lion.asm loc_3E84E should preserve shared state $0A while still restoring the saved animation"
        );
    }

    #[test]
    fn lion_charge_collision_landing_resume_matches_loc_3e88a() {
        assert_eq!(
            lion_charge_collision_landing_resume_loc_3e88a(),
            LionChargeCollisionLandingResumeLoc3E88A {
                restored_animation: "saved_lion_charge_animation",
                next_branch: "loc_3E546",
            },
            "ASM Lion.asm loc_3E88A should init the saved `$62(a5)` animation before branching to loc_3E546"
        );
    }

    #[test]
    fn lion_charge_collision_y_commit_matches_loc_3e896() {
        assert_eq!(
            lion_charge_collision_y_commit_loc_3e896(0x0052_8000),
            LionChargeCollisionYCommitLoc3E896 {
                y_position: 0x0052_8000,
                enters_gravity_tail: true,
            },
            "ASM Lion.asm loc_3E896 should commit projected `d2` into y_pos before the collision-reentry gravity tail"
        );
    }

    #[test]
    fn lion_charge_collision_gravity_cap_matches_loc_3e89a() {
        assert_eq!(
            lion_charge_collision_gravity_cap_loc_3e89a(0x0002_0000),
            0x0002_4000,
            "ASM Lion.asm loc_3E89A should add `$4000` gravity before the cap check"
        );
        assert_eq!(
            lion_charge_collision_gravity_cap_loc_3e89a(0x0017_D000),
            0x0018_0000,
            "ASM Lion.asm loc_3E89A should clamp signed downward velocities at `$180000`"
        );
        assert_eq!(
            lion_charge_collision_gravity_cap_loc_3e89a(-0x0001_0000),
            -0x0000_C000,
            "ASM Lion.asm loc_3E89A has no upward-speed clamp; signed negative values stay below `$180000`"
        );
    }

    #[test]
    fn lion_charge_collision_y_velocity_commit_matches_loc_3e8b2() {
        assert_eq!(
            lion_charge_collision_y_velocity_commit_loc_3e8b2(0x0018_0000),
            LionChargeCollisionYVelocityCommitLoc3E8B2 {
                y_velocity: 0x0018_0000,
                next_tail: LionLoc3E556Tail {
                    hibernate_one_frame: true,
                    cull_routine: "sub_3E8BA",
                    resume_via_callback: true,
                },
            },
            "ASM Lion.asm loc_3E8B2 should commit prepared d0 into y_vel and branch to loc_3E7C0"
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
    fn lion_charge_collision_airborne_resume_matches_loc_3e7c0() {
        assert_eq!(
            lion_charge_collision_airborne_resume_loc_3e7c0(),
            LionLoc3E556Tail {
                hibernate_one_frame: true,
                cull_routine: "sub_3E8BA",
                resume_via_callback: true,
            },
            "ASM Lion.asm loc_3E7C0 should hibernate, run sub_3E8BA, and jump through a0 after collision-reentry airborne motion"
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
    fn lion_cull_reject_matches_loc_3e920_trampoline() {
        assert_eq!(
            lion_cull_reject_loc_3e920(3, 0x05),
            LionCullRejectLoc3E920 {
                next_number_of_enemy: 2,
                status_action: DeactivationStatusActionLoc3C4C0::WriteRespawnMarker { index: 5 },
                deletes_current_object: true,
            },
            "ASM Lion.asm loc_3E920 should branch to the delete tail and write a respawn marker for positive `$42(a5)` slots"
        );
        assert_eq!(
            lion_cull_reject_loc_3e920(0, 0xC2),
            LionCullRejectLoc3E920 {
                next_number_of_enemy: 0xFFFF,
                status_action: DeactivationStatusActionLoc3C4C0::SubtractSpawnCounter { index: 2 },
                deletes_current_object: true,
            },
            "ASM Lion.asm loc_3E920 should preserve the ROM's word decrement and bit-6 spawn-counter route"
        );
        assert_eq!(
            lion_cull_reject_loc_3e920(1, 0x80),
            LionCullRejectLoc3E920 {
                next_number_of_enemy: 0,
                status_action: DeactivationStatusActionLoc3C4C0::DeleteOnly,
                deletes_current_object: true,
            },
            "ASM Lion.asm loc_3E920 should delete without a table update when bit 7 is set and bit 6 is clear"
        );
    }

    #[test]
    fn lion_delete_tail_matches_loc_3e98a() {
        assert_eq!(
            lion_delete_tail_loc_3e98a(),
            LionDeleteTailLoc3E98A {
                deletes_current_object: true,
            },
            "ASM Lion.asm loc_3E98A should jump to j_Delete_CurrentObject after every cull status route"
        );
    }

    #[test]
    fn lion_respawn_marker_matches_loc_3e97c_positive_status_slot() {
        assert_eq!(
            lion_respawn_marker_loc_3e97c(0x05),
            LionRespawnMarkerLoc3E97C {
                index: 5,
                word_value: 0x2168,
            },
            "ASM Lion.asm loc_3E97C should double positive `$42(a5)` as an EnemyStatus_Table word offset and write `$2168`"
        );
    }

    #[test]
    fn lion_medium_load_cull_matches_loc_3e924_window() {
        assert_eq!(
            lion_medium_load_cull_loc_3e924(0x0E5C, 0x0E5C, 0x1000, 0x1000, 7, 0x05),
            LionMediumLoadCullLoc3E924::Return,
            "ASM Lion.asm loc_3E924 should keep the lower inclusive medium-load x/y edge at `-$1A4`"
        );
        assert_eq!(
            lion_medium_load_cull_loc_3e924(0x12E4, 0x1284, 0x1000, 0x1000, 7, 0x05),
            LionMediumLoadCullLoc3E924::Return,
            "ASM Lion.asm loc_3E924 should keep the upper inclusive medium-load edges x=`$2E4`, y=`$284`"
        );
        assert_eq!(
            lion_medium_load_cull_loc_3e924(0x0E5B, 0x1000, 0x1000, 0x1000, 7, 0x05),
            LionMediumLoadCullLoc3E924::DeleteTail(LionCullRejectLoc3E920 {
                next_number_of_enemy: 6,
                status_action: DeactivationStatusActionLoc3C4C0::WriteRespawnMarker { index: 5 },
                deletes_current_object: true,
            }),
            "ASM Lion.asm loc_3E924 should branch through loc_3E920 once relative x drops below `-$1A4`"
        );
    }

    #[test]
    fn lion_terrain_collision_matches_loc_3e94e_sentinel() {
        assert_eq!(
            lion_terrain_collision_loc_3e94e(),
            LionTerrainCollisionLoc3E94E {
                next_collision_type: -1,
                resume_via_callback: true,
            },
            "ASM Lion.asm loc_3E94E should set collision_type to `$FFFF` before jumping through a0"
        );
    }

    #[test]
    fn lion_airborne_charge_body_terrain_enters_death_animation_loc_3e94e() {
        let mut grid = terrain::BlockGrid::from_level_layout(&[0; 6 * 6 * 2], 6, 6);
        grid.set_cell_at(1, 1, terrain::Cell::from_be_bytes(0x40, 0x00));
        let mut lion = ActiveEnemy {
            family: EnemyFamily::Lion,
            x_pos: 24 << 16,
            y_pos: 32 << 16,
            x_velocity: 0x0001_0000,
            y_velocity: 0,
            facing: Direction::Right,
            on_ground: false,
            hit_points: 8,
            lion_state: LionState::ResumingCharge,
            wait_timer: 0,
            ..mini_hopping_skull_test_enemy()
        };
        let mut projectiles = Vec::new();
        let mut projectile_sink = Some(&mut projectiles);

        step_lion(&mut lion, Some(&grid), None, &mut projectile_sink);

        assert!(
            lion.death_animation_active,
            "ASM Lion.asm loc_3E6AE should branch to loc_3E94E when sub_36B3C returns non-zero d5 for a bit-$4000 body tile"
        );
        assert_eq!(
            lion.hit_points, 0,
            "ASM Lion.asm loc_3E94E writes collision_type=$FFFF, which Lion.asm loc_3E318 routes to the death tail"
        );
        assert_eq!(
            lion.wait_timer, LION_DEATH_ANIMATION_FRAMES,
            "native represents the loc_3E94E `$FFFF` collision word with the family death-animation timer"
        );
        assert_eq!(
            lion.x_pos,
            24 << 16,
            "ASM Lion.asm loc_3E6AE checks sub_36B3C before committing horizontal motion"
        );
    }

    #[test]
    fn lion_airborne_charge_floor_edge_overlap_does_not_route_to_loc_3e94e() {
        let mut grid = terrain::BlockGrid::from_level_layout(&[0; 6 * 6 * 2], 6, 6);
        grid.set_cell_at(1, 2, terrain::Cell::from_be_bytes(0x60, 0x00));
        let mut lion = ActiveEnemy {
            family: EnemyFamily::Lion,
            x_pos: 24 << 16,
            y_pos: 35 << 16,
            x_velocity: -0x0001_8000,
            y_velocity: 0,
            facing: Direction::Left,
            on_ground: false,
            hit_points: 8,
            lion_state: LionState::ResumingCharge,
            wait_timer: 0,
            ..mini_hopping_skull_test_enemy()
        };
        let mut projectiles = Vec::new();
        let mut projectile_sink = Some(&mut projectiles);

        step_lion(&mut lion, Some(&grid), None, &mut projectile_sink);

        assert_eq!(
            lion.hit_points, 8,
            "ASM Lion.asm loc_3E7CA should not branch to loc_3E94E just because native's transient airborne state overlaps the floor row"
        );
        assert!(
            !lion.death_animation_active,
            "Level 0x07 oracle keeps the lower Lion alive when its feet ride the floor edge"
        );
    }

    #[test]
    fn lion_airborne_charge_consumes_right_lead_column_boundary_loc_3e6ae() {
        let grid = terrain::BlockGrid::from_level_layout(&[0; 4 * 6 * 2], 4, 6);
        let mut lion = ActiveEnemy {
            family: EnemyFamily::Lion,
            x_pos: 43 << 16,
            y_pos: 32 << 16,
            x_velocity: 0x0002_0000,
            y_velocity: 0,
            facing: Direction::Right,
            on_ground: false,
            hit_points: 8,
            lion_state: LionState::ResumingCharge,
            wait_timer: 0,
            ..mini_hopping_skull_test_enemy()
        };
        let mut projectiles = Vec::new();
        let mut projectile_sink = Some(&mut projectiles);

        step_lion(&mut lion, Some(&grid), None, &mut projectile_sink);

        assert_eq!(
            lion.x_pos,
            44 << 16,
            "ASM Lion.asm loc_3E6AE -> sub_36B3C should consume d6=1 at the right level boundary and snap to the next tile edge"
        );
        assert_eq!(
            lion.x_velocity, 0,
            "ASM Lion.asm loc_3E6AE clears x_vel when the blocked lead column stops the airborne charge"
        );
    }

    #[test]
    fn lion_airborne_charge_consumes_left_lead_column_boundary_loc_3e7ca() {
        let grid = terrain::BlockGrid::from_level_layout(&[0; 3 * 6 * 2], 3, 6);
        let mut lion = ActiveEnemy {
            family: EnemyFamily::Lion,
            x_pos: 21 << 16,
            y_pos: 32 << 16,
            x_velocity: -0x0002_0000,
            y_velocity: 0,
            facing: Direction::Left,
            on_ground: false,
            hit_points: 8,
            lion_state: LionState::ResumingCharge,
            wait_timer: 0,
            ..mini_hopping_skull_test_enemy()
        };
        let mut projectiles = Vec::new();
        let mut projectile_sink = Some(&mut projectiles);

        step_lion(&mut lion, Some(&grid), None, &mut projectile_sink);

        assert_eq!(
            lion.x_pos,
            20 << 16,
            "ASM Lion.asm loc_3E7CA -> sub_36BD6 should consume d6=1 at the left level boundary and snap to the previous tile edge"
        );
        assert_eq!(
            lion.x_velocity, 0,
            "ASM Lion.asm loc_3E7CA clears x_vel when the blocked lead column stops the airborne charge"
        );
    }

    #[test]
    fn lion_airborne_charge_lands_on_full_width_floor_scan_loc_3e732() {
        let mut grid = terrain::BlockGrid::from_level_layout(&[0; 5 * 6 * 2], 5, 6);
        grid.set_cell_at(3, 3, terrain::Cell::from_be_bytes(0x40, 0x00));
        let mut lion = ActiveEnemy {
            family: EnemyFamily::Lion,
            x_pos: 40 << 16,
            y_pos: 34 << 16,
            x_velocity: 0,
            y_velocity: 0x0010_0000,
            facing: Direction::Right,
            on_ground: false,
            hit_points: 8,
            lion_state: LionState::ResumingCharge,
            shared_walker_state: 0,
            ..mini_hopping_skull_test_enemy()
        };
        let mut projectiles = Vec::new();
        let mut projectile_sink = Some(&mut projectiles);

        step_lion(&mut lion, Some(&grid), None, &mut projectile_sink);

        assert_eq!(
            lion.y_pos,
            48 << 16,
            "ASM Lion.asm loc_3E732 -> sub_36C6A scans x +/- $14 and should land on a far-edge bit-$4000 floor cell"
        );
        assert!(
            lion.on_ground,
            "ASM Lion.asm loc_3E732 clears the airborne latch when the full-width floor scan hits"
        );
        assert_eq!(
            lion.y_velocity, 0,
            "ASM Lion.asm loc_3E732 clears y_vel on landing before the loc_3E392 tail"
        );
        assert_eq!(
            lion.x_velocity, 0,
            "ASM Lion.asm loc_3E392 clears x_vel after the landing resume"
        );
        assert_eq!(
            lion.shared_walker_state, -0x78,
            "ASM Lion.asm loc_3E732 writes `$FF88` into `$42(a3)` unless the shared state is `$0A`"
        );
    }

    #[test]
    fn lion_airborne_charge_hits_full_width_ceiling_scan_loc_3e6f6() {
        let mut grid = terrain::BlockGrid::from_level_layout(&[0; 5 * 6 * 2], 5, 6);
        grid.set_cell_at(3, 0, terrain::Cell::from_be_bytes(0x40, 0x00));
        let mut lion = ActiveEnemy {
            family: EnemyFamily::Lion,
            x_pos: 40 << 16,
            y_pos: 64 << 16,
            x_velocity: 0,
            y_velocity: -0x0010_0000,
            facing: Direction::Right,
            on_ground: false,
            hit_points: 8,
            lion_state: LionState::ResumingCharge,
            ..mini_hopping_skull_test_enemy()
        };
        let mut projectiles = Vec::new();
        let mut projectile_sink = Some(&mut projectiles);

        step_lion(&mut lion, Some(&grid), None, &mut projectile_sink);

        assert_eq!(
            lion.y_pos,
            63 << 16,
            "ASM Lion.asm loc_3E6F6 -> sub_36CB8 scans x +/- $14 and should snap to the ceiling boundary on a far-edge bit-$4000 cell"
        );
        assert_eq!(
            lion.y_velocity, 0x0000_4000,
            "ASM Lion.asm loc_3E6F6 clears y_vel on ceiling hit, then loc_3E77E immediately reapplies `$4000` gravity"
        );
        assert!(
            !lion.on_ground,
            "ASM Lion.asm loc_3E6F6 ceiling hits stay in the airborne charge loop"
        );
    }

    #[test]
    fn lion_left_airborne_charge_body_terrain_enters_death_animation_loc_3e94e() {
        let mut grid = terrain::BlockGrid::from_level_layout(&[0; 6 * 6 * 2], 6, 6);
        grid.set_cell_at(1, 1, terrain::Cell::from_be_bytes(0x40, 0x00));
        let mut lion = ActiveEnemy {
            family: EnemyFamily::Lion,
            x_pos: 24 << 16,
            y_pos: 32 << 16,
            x_velocity: -0x0001_0000,
            y_velocity: 0,
            facing: Direction::Left,
            on_ground: false,
            hit_points: 8,
            lion_state: LionState::ResumingCharge,
            wait_timer: 0,
            ..mini_hopping_skull_test_enemy()
        };
        let mut projectiles = Vec::new();
        let mut projectile_sink = Some(&mut projectiles);

        step_lion(&mut lion, Some(&grid), None, &mut projectile_sink);

        assert!(
            lion.death_animation_active,
            "ASM Lion.asm loc_3E7CA should branch to loc_3E94E when sub_36BD6 returns non-zero d5 for a bit-$4000 body tile"
        );
        assert_eq!(
            lion.wait_timer, LION_DEATH_ANIMATION_FRAMES,
            "native represents the loc_3E94E `$FFFF` collision word with the family death-animation timer"
        );
        assert_eq!(
            lion.x_pos,
            24 << 16,
            "ASM Lion.asm loc_3E7CA checks sub_36BD6 before committing horizontal motion"
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
}
