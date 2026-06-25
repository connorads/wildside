use super::*;

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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) enum DragonWalkingCollisionRouteLoc3CE9A {
    Continue,
    Damage,
    BerzerkerSideGate,
    Death,
}

#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn dragon_walking_collision_route_loc_3ce9a(
    collision_type: i16,
) -> DragonWalkingCollisionRouteLoc3CE9A {
    // ASM Dragon.asm loc_3CE9A: read `collision_type(a3)` into d4; negative
    // words take the death path, zero/unknown positives clear and resume, hurt
    // or Kid-above contacts damage HP, and side contacts gate on Berzerker.
    if collision_type.is_negative() {
        return DragonWalkingCollisionRouteLoc3CE9A::Death;
    }

    match collision_type as u16 {
        DRAGON_COLID_HURT | DRAGON_COLID_KIDABOVE => DragonWalkingCollisionRouteLoc3CE9A::Damage,
        DRAGON_COLID_KIDRIGHT | DRAGON_COLID_KIDLEFT => {
            DragonWalkingCollisionRouteLoc3CE9A::BerzerkerSideGate
        }
        _ => DragonWalkingCollisionRouteLoc3CE9A::Continue,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct DragonWalkingDeathSequenceLoc3CEFC {
    pub(crate) death_animation: &'static str,
    pub(crate) waits_until_animation_finished: bool,
    pub(crate) status_action: DeactivationStatusActionLoc3C4C0,
    pub(crate) exits_enemy: bool,
}

pub(crate) fn dragon_walking_death_sequence_loc_3cefc(
    object_status_slot: u8,
) -> DragonWalkingDeathSequenceLoc3CEFC {
    // ASM Dragon.asm loc_3CEFC: initialise `stru_3CCEC`, hibernate until that
    // animation finishes, then branch to the shared enemy exit tail at loc_3E956.
    DragonWalkingDeathSequenceLoc3CEFC {
        death_animation: "stru_3CCEC",
        waits_until_animation_finished: true,
        status_action: deactivation_status_update_loc_3C4C0(object_status_slot),
        exits_enemy: true,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) struct DragonWalkingBerzerkerKnockbackStartLoc3CF0E {
    pub(crate) has_level_collision: bool,
    pub(crate) has_kid_collision: bool,
    pub(crate) is_moved: bool,
    pub(crate) is_animated: bool,
    pub(crate) sprite_mapping: &'static str,
    pub(crate) x_velocity: i32,
    pub(crate) y_velocity: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) enum DragonWalkingBerzerkerSideGateLoc3CF0E {
    ResumePatrol { next_collision_type: u16 },
    Knockback(DragonWalkingBerzerkerKnockbackStartLoc3CF0E),
}

#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn dragon_walking_berzerker_side_gate_loc_3cf0e(
    berzerker_charging: bool,
) -> DragonWalkingBerzerkerSideGateLoc3CF0E {
    // ASM Dragon.asm loc_3CF0E: side contact falls through to the normal
    // clear/resume path when `Berzerker_charging` is zero. A charging Berzerker
    // disables level collision/animation, re-enables Kid collision and movement,
    // swaps to `LnkTo_unk_C755E`, zeroes X velocity, and launches upward.
    if !berzerker_charging {
        return DragonWalkingBerzerkerSideGateLoc3CF0E::ResumePatrol {
            next_collision_type: 0,
        };
    }

    DragonWalkingBerzerkerSideGateLoc3CF0E::Knockback(
        DragonWalkingBerzerkerKnockbackStartLoc3CF0E {
            has_level_collision: false,
            has_kid_collision: true,
            is_moved: true,
            is_animated: false,
            sprite_mapping: "LnkTo_unk_C755E-Data_Index",
            x_velocity: 0,
            y_velocity: -0x0004_8000,
        },
    )
}

#[cfg_attr(not(test), allow(dead_code))]
pub(crate) const DRAGON_WALKING_BERZERKER_KNOCKBACK_GRAVITY_LOC_3CF3A: i32 = 0x0000_4000;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) struct DragonWalkingBerzerkerAirborneFrameLoc3CF3A {
    pub(crate) applies_motion_subroutine: bool,
    pub(crate) next_y_velocity: i32,
    pub(crate) repeats_airborne_loop: bool,
}

#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn dragon_walking_berzerker_airborne_frame_loc_3cf3a(
    y_velocity: i32,
) -> DragonWalkingBerzerkerAirborneFrameLoc3CF3A {
    // ASM Dragon.asm loc_3CF3A: hibernate one frame, apply shared movement
    // using the current velocities, add `$4000` to `y_vel(a3)`, and branch
    // back to repeat the airborne knockback loop.
    DragonWalkingBerzerkerAirborneFrameLoc3CF3A {
        applies_motion_subroutine: true,
        next_y_velocity: y_velocity
            .wrapping_add(DRAGON_WALKING_BERZERKER_KNOCKBACK_GRAVITY_LOC_3CF3A),
        repeats_airborne_loop: true,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) struct DragonWalkingCollisionClearLoc3CF4C {
    pub(crate) collision_type: u16,
    pub(crate) falls_through_to_resume_dispatch: bool,
}

#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn dragon_walking_collision_clear_loc_3cf4c() -> DragonWalkingCollisionClearLoc3CF4C {
    // ASM Dragon.asm loc_3CF4C: `clr.w collision_type(a3)`, then fall through
    // to the current `$5C(a5)`/x_direction pacing route.
    DragonWalkingCollisionClearLoc3CF4C {
        collision_type: 0,
        falls_through_to_resume_dispatch: true,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) enum DragonWalkingSpawnedResumeDispatchLoc3CF50 {
    OrdinaryDragonResume,
    SpawnedChildLeftPacing,
    SpawnedChildRightPacing,
}

#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn dragon_walking_spawned_resume_dispatch_loc_3cf50(
    parent_spawned_child: bool,
    facing: Direction,
) -> DragonWalkingSpawnedResumeDispatchLoc3CF50 {
    // ASM Dragon.asm loc_3CF50: when `$5C(a5)` is clear, ordinary walking
    // dragons continue into the normal resume gate. Parent-spawned children
    // test `x_direction(a3)` here: clear goes to loc_3E7CA, set to loc_3E6AE.
    if !parent_spawned_child {
        return DragonWalkingSpawnedResumeDispatchLoc3CF50::OrdinaryDragonResume;
    }

    match facing {
        Direction::Left => DragonWalkingSpawnedResumeDispatchLoc3CF50::SpawnedChildLeftPacing,
        Direction::Right => DragonWalkingSpawnedResumeDispatchLoc3CF50::SpawnedChildRightPacing,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) enum DragonWalkingNormalResumeDispatchLoc3CF62 {
    Loc3E560,
    Loc3E3AC,
}

#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn dragon_walking_normal_resume_dispatch_loc_3cf62(
    facing: Direction,
) -> DragonWalkingNormalResumeDispatchLoc3CF62 {
    // ASM Dragon.asm loc_3CF62: ordinary walking dragons test
    // `x_direction(a3)` after a clear/resume path; clear resumes at loc_3E560,
    // set resumes at loc_3E3AC.
    match facing {
        Direction::Left => DragonWalkingNormalResumeDispatchLoc3CF62::Loc3E560,
        Direction::Right => DragonWalkingNormalResumeDispatchLoc3CF62::Loc3E3AC,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum DragonWalkingBreathTriggerNextLoc3CF6E {
    SpawnedChildRecover,
    AllocateBreathObject,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct DragonWalkingBreathTriggerLoc3CF6E {
    pub(crate) breath_timer: u16,
    pub(crate) phase_done_flag_5b: u8,
    pub(crate) play_sound: bool,
    pub(crate) next: DragonWalkingBreathTriggerNextLoc3CF6E,
}

pub(crate) fn dragon_walking_breath_trigger_loc_3cf6e(
    parent_spawned_child: bool,
    object_flag_19_set: bool,
) -> DragonWalkingBreathTriggerLoc3CF6E {
    // ASM Dragon.asm loc_3CF6E: reset `$66` to `$1F4`, clear `$5B(a5)`,
    // bypass ordinary breath setup for spawned children, and only play the
    // flame sound when `$19(a3)` is clear.
    DragonWalkingBreathTriggerLoc3CF6E {
        breath_timer: DRAGON_WALKING_BREATH_RESET,
        phase_done_flag_5b: 0,
        play_sound: !parent_spawned_child && !object_flag_19_set,
        next: if parent_spawned_child {
            DragonWalkingBreathTriggerNextLoc3CF6E::SpawnedChildRecover
        } else {
            DragonWalkingBreathTriggerNextLoc3CF6E::AllocateBreathObject
        },
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct DragonWalkingBreathObjectStartLoc3CF90 {
    pub(crate) swaps_parent_and_child_registers: bool,
    pub(crate) allocation_mask: u16,
    pub(crate) child_callback: &'static str,
    pub(crate) parent_animation: &'static str,
}

pub(crate) fn dragon_walking_breath_object_start_loc_3cf90()
-> DragonWalkingBreathObjectStartLoc3CF90 {
    // ASM Dragon.asm loc_3CF90: exchange `a0/a1`, allocate a child object slot
    // using `$8000`, install the breath worker callback, restore `a0/a1`, and
    // start the parent's `stru_3CCC2` breath animation.
    DragonWalkingBreathObjectStartLoc3CF90 {
        swaps_parent_and_child_registers: true,
        allocation_mask: 0x8000,
        child_callback: "loc_3D02A",
        parent_animation: "stru_3CCC2",
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) enum DragonWalkingBreathPollNextLoc3CFAE {
    WaitForPhaseDoneFlag,
    SharedCollisionRoute,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) struct DragonWalkingBreathCollisionPollLoc3CFAE {
    pub(crate) hibernates_one_frame: bool,
    pub(crate) sets_parent_phase_done_flag_5b: bool,
    pub(crate) next: DragonWalkingBreathPollNextLoc3CFAE,
}

#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn dragon_walking_breath_collision_poll_loc_3cfae(
    collision_type: i16,
    berzerker_charging: bool,
) -> DragonWalkingBreathCollisionPollLoc3CFAE {
    // ASM Dragon.asm loc_3CFAE: hibernate, read `collision_type(a3)`, route
    // negative collisions to the shared collision handler, mark `$5B(a1)` for
    // damaging contacts, and gate side contacts on `Berzerker_charging`.
    // ASM Dragon.asm loc_3CFD4: side contacts test `Berzerker_charging`;
    // clear keeps the breath poll waiting without setting the parent flag.
    // ASM Dragon.asm loc_3CFDA: accepted damage/charging-side contacts write
    // `#1` to `$5B(a1)` before returning through the shared collision route.
    let damaging_contact = matches!(
        collision_type as u16,
        DRAGON_COLID_HURT | DRAGON_COLID_KIDABOVE
    );
    let charging_side_contact = berzerker_charging
        && matches!(
            collision_type as u16,
            DRAGON_COLID_KIDRIGHT | DRAGON_COLID_KIDLEFT
        );
    let sets_parent_phase_done_flag_5b = damaging_contact || charging_side_contact;
    let next = if collision_type.is_negative() || sets_parent_phase_done_flag_5b {
        DragonWalkingBreathPollNextLoc3CFAE::SharedCollisionRoute
    } else {
        DragonWalkingBreathPollNextLoc3CFAE::WaitForPhaseDoneFlag
    };

    DragonWalkingBreathCollisionPollLoc3CFAE {
        hibernates_one_frame: true,
        sets_parent_phase_done_flag_5b,
        next,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) enum DragonWalkingBreathDoneNextLoc3CFE4 {
    RepeatBreathPoll,
    RestoreWalkingAnimation { animation_label: &'static str },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) struct DragonWalkingBreathDoneWaitLoc3CFE4 {
    pub(crate) hibernates_after_done_flag: bool,
    pub(crate) next: DragonWalkingBreathDoneNextLoc3CFE4,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) struct DragonWalkingBreathNormalRestoreLoc3D002 {
    pub(crate) animation_label: &'static str,
}

#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn dragon_walking_breath_normal_restore_loc_3d002()
-> DragonWalkingBreathNormalRestoreLoc3D002 {
    // ASM Dragon.asm loc_3D002: non-HP3 walking dragons restore the normal
    // `stru_3CC5E` animation after the breath done flag is observed.
    DragonWalkingBreathNormalRestoreLoc3D002 {
        animation_label: "stru_3CC5E",
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) enum DragonWalkingBreathSpawnedDispatchLoc3D00C {
    OrdinaryResume,
    SpawnedChildLeftPacing,
    SpawnedChildRightPacing,
}

#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn dragon_walking_breath_spawned_dispatch_loc_3d00c(
    parent_spawned_child: bool,
    facing: Direction,
) -> DragonWalkingBreathSpawnedDispatchLoc3D00C {
    // ASM Dragon.asm loc_3D00C: clear `$5C(a5)` continues through the ordinary
    // resume path; parent-spawned children branch by `x_direction(a3)`, where
    // clear goes to `loc_3E7CA` and set goes to `loc_3E6AE`.
    if !parent_spawned_child {
        return DragonWalkingBreathSpawnedDispatchLoc3D00C::OrdinaryResume;
    }

    match facing {
        Direction::Left => DragonWalkingBreathSpawnedDispatchLoc3D00C::SpawnedChildLeftPacing,
        Direction::Right => DragonWalkingBreathSpawnedDispatchLoc3D00C::SpawnedChildRightPacing,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) enum DragonWalkingBreathNormalDispatchLoc3D01E {
    Loc3E560,
    Loc3E3AC,
}

#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn dragon_walking_breath_normal_dispatch_loc_3d01e(
    facing: Direction,
) -> DragonWalkingBreathNormalDispatchLoc3D01E {
    // ASM Dragon.asm loc_3D01E: ordinary breath recovery tests
    // `x_direction(a3)`; clear resumes at loc_3E560, and set resumes at
    // loc_3E3AC.
    match facing {
        Direction::Left => DragonWalkingBreathNormalDispatchLoc3D01E::Loc3E560,
        Direction::Right => DragonWalkingBreathNormalDispatchLoc3D01E::Loc3E3AC,
    }
}

#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn dragon_walking_breath_done_wait_loc_3cfe4(
    parent_phase_done_flag_5b: bool,
    enemy_level_word_40: u16,
) -> DragonWalkingBreathDoneWaitLoc3CFE4 {
    // ASM Dragon.asm loc_3CFE4: clear `$5B(a5)` branches back to the breath
    // poll. Once set, the parent hibernates once, then `$40(a3) == 2`
    // restores HP-3 `stru_3CC90`; all other levels restore `stru_3CC5E`.
    if !parent_phase_done_flag_5b {
        return DragonWalkingBreathDoneWaitLoc3CFE4 {
            hibernates_after_done_flag: false,
            next: DragonWalkingBreathDoneNextLoc3CFE4::RepeatBreathPoll,
        };
    }

    let animation_label = if enemy_level_word_40 == 2 {
        "stru_3CC90"
    } else {
        dragon_walking_breath_normal_restore_loc_3d002().animation_label
    };
    DragonWalkingBreathDoneWaitLoc3CFE4 {
        hibernates_after_done_flag: true,
        next: DragonWalkingBreathDoneNextLoc3CFE4::RestoreWalkingAnimation { animation_label },
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

pub(crate) const DRAGON_WALKING_BREATH_RESET: u16 = 500;
/// ASM loc_3D02A: breath is a stationary GfxObject, not a travelling projectile.
/// Animation stru_3CCC8 lasts 68 frames (8+10+10+10+10+10+10), after which
/// $18(a3) signals completion. No velocity is applied in the ROM.
pub(crate) const DRAGON_WALKING_BREATH_LIFETIME: u16 = 68;
pub(crate) const DRAGON_WALKING_BREATH_HOLD_FRAMES: u16 = 68;
pub(crate) const DRAGON_WALKING_RECOVER_FRAMES: u16 = 0;
pub(crate) const DRAGON_WALKING_BREATH_OFFSET_PX: i32 = 0x1C;

const DRAGON_WALKING_PHASE_BREATH: u8 = 1;
const DRAGON_WALKING_PHASE_RECOVER: u8 = 2;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct DragonWalkingBreathChildPositionLoc3D070 {
    pub(crate) x_pos: u32,
    pub(crate) y_pos: u32,
    pub(crate) status_flag_13: u8,
    pub(crate) animation_label: &'static str,
}

pub(crate) fn dragon_walking_breath_child_position_loc_3d070(
    parent_x_pos: u32,
    parent_y_pos: u32,
    parent_direction: Direction,
) -> DragonWalkingBreathChildPositionLoc3D070 {
    // ASM Dragon.asm loc_3D070: after `d0` is seeded to `-$1C` and negated
    // only for set `x_direction`, add the word offset to the child X word,
    // set `$13(a3)`, and initialise `stru_3CCC8`.
    let x_offset = match parent_direction {
        Direction::Left => -DRAGON_WALKING_BREATH_OFFSET_PX,
        Direction::Right => DRAGON_WALKING_BREATH_OFFSET_PX,
    };

    DragonWalkingBreathChildPositionLoc3D070 {
        x_pos: (i64::from(parent_x_pos) + (i64::from(x_offset) << 16)) as u32,
        y_pos: parent_y_pos,
        status_flag_13: 0xFF,
        animation_label: "stru_3CCC8",
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum DragonWalkingBreathChildPollNextLoc3D082 {
    RepeatPoll,
    DeleteChild,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct DragonWalkingBreathChildPollLoc3D082 {
    pub(crate) hibernates_one_frame: bool,
    pub(crate) sets_parent_phase_done_flag_5b: bool,
    pub(crate) next: DragonWalkingBreathChildPollNextLoc3D082,
}

pub(crate) fn dragon_walking_breath_child_poll_loc_3d082(
    parent_phase_done_flag_5b: u8,
    child_animation_done_flag_18: bool,
) -> DragonWalkingBreathChildPollLoc3D082 {
    // ASM Dragon.asm loc_3D082: hibernate one frame, delete when the parent
    // `$5B` flag is non-zero, otherwise keep polling until child `$18` marks
    // the animation complete; completion writes `#1` back to parent `$5B`.
    if parent_phase_done_flag_5b != 0 {
        return DragonWalkingBreathChildPollLoc3D082 {
            hibernates_one_frame: true,
            sets_parent_phase_done_flag_5b: false,
            next: DragonWalkingBreathChildPollNextLoc3D082::DeleteChild,
        };
    }

    if !child_animation_done_flag_18 {
        return DragonWalkingBreathChildPollLoc3D082 {
            hibernates_one_frame: true,
            sets_parent_phase_done_flag_5b: false,
            next: DragonWalkingBreathChildPollNextLoc3D082::RepeatPoll,
        };
    }

    DragonWalkingBreathChildPollLoc3D082 {
        hibernates_one_frame: true,
        sets_parent_phase_done_flag_5b: true,
        next: DragonWalkingBreathChildPollNextLoc3D082::DeleteChild,
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct DragonWalkingBreathChildDeleteLoc3D09E {
    pub(crate) deletes_current_object: bool,
}

pub(crate) fn dragon_walking_breath_child_delete_loc_3d09e()
-> DragonWalkingBreathChildDeleteLoc3D09E {
    // ASM Dragon.asm loc_3D09E: tail-jump to `Delete_CurrentObject`; native
    // represents the breath GfxObject as an expiring projectile.
    DragonWalkingBreathChildDeleteLoc3D09E {
        deletes_current_object: true,
    }
}

enum DragonWalkingWalkStep {
    Walked,
    Idled,
    StartedAirborne,
}

fn dragon_walking_blocked(enemy: &ActiveEnemy, block_grid: Option<&terrain::BlockGrid>) -> bool {
    let Some(grid) = block_grid else {
        return false;
    };
    let walk_speed = pacer_walk_speed(enemy.family, enemy.hit_points);
    let x_velocity = horizontal_velocity(enemy.facing, walk_speed);
    let moving_right = x_velocity >= 0;
    let next_x = (enemy.x_pos as i64 + i64::from(x_velocity)) as u32;
    let current_cx = (enemy.x_pos >> 16) as i32;
    let cx = (next_x >> 16) as i32;
    let cy = (enemy.y_pos >> 16) as i32;
    let wall = dragon_walking_probe_wall(grid, current_cx, cx, cy, moving_right);
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
        let current_cx = (enemy.x_pos >> 16) as i32;
        let cx = (next_x >> 16) as i32;
        let wall = dragon_walking_probe_wall(grid, current_cx, cx, current_y_px, moving_right);
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
            // ORIGINAL_QUIRK (#239): ASM Lion.asm loc_3E682 (shared with the
            // walking dragon) has NO horizontal arena clamp and NO ledge
            // turnaround. With no floor ahead the ROM sets the airborne flag
            // `$5C(a5)`, keeps the walk velocity, nudges y down a pixel and
            // hands off to the gravity-fall loop (loc_3E6AE/loc_3E77E). The
            // walker therefore walks off the level edge and falls — the
            // "dragon falls off the end of the level and respawns" the
            // 2026-06-03 playtest saw. Oracle-matched; not a missing clamp.
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
            let breath_trigger = dragon_walking_breath_trigger_loc_3cf6e(false, false);
            let breath_object = dragon_walking_breath_object_start_loc_3cf90();
            debug_assert_eq!(breath_object.child_callback, "loc_3D02A");
            // ASM loc_3D02A: breath is a stationary animated hitbox at
            // the dragon's mouth (±28px). No velocity — the GfxObject
            // stays in place for the animation duration (stru_3CCC8, 68f).
            //
            // ORIGINAL_QUIRK (#240): the breath hitbox sits on the dragon's
            // FACING side only (loc_3D070: d0=-$1C, negated when x_direction
            // is set). The facing follows the shared-walker FacePlayer/patrol
            // state, so a dragon mid-patrol can momentarily face away from Kid
            // and "breathe the wrong way" — this is ROM-accurate, not a bug.
            // Oracle proof: tas-diff .cache/debug/issue240_l0_dragon_settle.json
            // (warp L0 + settle_oracle_warp) frame 241 (breath spawn): native
            // AND jgenesis dragon @x=50px face=Left, breath child @x=22px
            // (50-$1C) — byte-exact MATCH. The breath direction is the parent's
            // x_direction, copied to the child at loc_3D02A `$16(a0)`.
            let child_position = dragon_walking_breath_child_position_loc_3d070(
                enemy.x_pos,
                enemy.y_pos,
                enemy.facing,
            );
            debug_assert_eq!(child_position.status_flag_13, 0xFF);
            debug_assert_eq!(child_position.animation_label, "stru_3CCC8");
            projectiles.push(EnemyProjectile {
                x_pos: child_position.x_pos,
                y_pos: child_position.y_pos,
                lifetime: DRAGON_WALKING_BREATH_LIFETIME,
                kind: EnemyProjectileKind::DragonWalkingBreath,
                ..Default::default()
            });
            enemy.wait_timer = breath_trigger.breath_timer;
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
            let child_poll =
                dragon_walking_breath_child_poll_loc_3d082(0, enemy.dragon_phase_frames == 0);
            debug_assert!(child_poll.hibernates_one_frame);

            if child_poll.next == DragonWalkingBreathChildPollNextLoc3D082::RepeatPoll {
                enemy.dragon_phase_frames -= 1;
            } else {
                debug_assert!(child_poll.sets_parent_phase_done_flag_5b);
                let child_delete = dragon_walking_breath_child_delete_loc_3d09e();
                debug_assert!(child_delete.deletes_current_object);
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::enemies::test_fixtures::*;

    fn dragon_walking_test_enemy() -> ActiveEnemy {
        ActiveEnemy {
            family: EnemyFamily::DragonWalking,
            hit_points: 3,
            facing: Direction::Right,
            on_ground: true,
            dragon_script_cmd: DRAGON_WALKING_PHASE_WALK,
            shared_walker_state: SHARED_WALKER_STATE_PATROL,
            ..mini_hopping_skull_test_enemy()
        }
    }

    fn dragon_walking_grid(
        width: u16,
        height: u16,
        solid_cells: &[(u16, u16)],
    ) -> terrain::BlockGrid {
        let mut grid = terrain::BlockGrid::from_level_layout(
            &vec![0; usize::from(width) * usize::from(height) * 2],
            width,
            height,
        );
        for &(col, row) in solid_cells {
            grid.set_cell_at(col, row, terrain::Cell::from_be_bytes(0x60, 0x00));
        }
        grid
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
    fn dragon_walking_collision_route_matches_loc_3ce9a() {
        assert_eq!(
            dragon_walking_collision_route_loc_3ce9a(-1),
            DragonWalkingCollisionRouteLoc3CE9A::Death,
            "ASM Dragon.asm loc_3CE9A sends negative collision words to the death path"
        );
        assert_eq!(
            dragon_walking_collision_route_loc_3ce9a(0),
            DragonWalkingCollisionRouteLoc3CE9A::Continue,
            "ASM Dragon.asm loc_3CE9A clears zero collision words and resumes walking"
        );
        assert_eq!(
            dragon_walking_collision_route_loc_3ce9a(DRAGON_COLID_HURT as i16),
            DragonWalkingCollisionRouteLoc3CE9A::Damage,
            "ASM Dragon.asm loc_3CE9A sends colid_hurt to the damage path"
        );
        assert_eq!(
            dragon_walking_collision_route_loc_3ce9a(DRAGON_COLID_KIDABOVE as i16),
            DragonWalkingCollisionRouteLoc3CE9A::Damage,
            "ASM Dragon.asm loc_3CE9A sends colid_kidabove to the same damage path"
        );
        assert_eq!(
            dragon_walking_collision_route_loc_3ce9a(DRAGON_COLID_KIDRIGHT as i16),
            DragonWalkingCollisionRouteLoc3CE9A::BerzerkerSideGate,
            "ASM Dragon.asm loc_3CE9A sends colid_kidright to the Berzerker side-contact gate"
        );
        assert_eq!(
            dragon_walking_collision_route_loc_3ce9a(DRAGON_COLID_KIDLEFT as i16),
            DragonWalkingCollisionRouteLoc3CE9A::BerzerkerSideGate,
            "ASM Dragon.asm loc_3CE9A sends colid_kidleft to the Berzerker side-contact gate"
        );
        assert_eq!(
            dragon_walking_collision_route_loc_3ce9a(0x0004),
            DragonWalkingCollisionRouteLoc3CE9A::Continue,
            "ASM Dragon.asm loc_3CE9A treats unrelated positive collision words as clear-and-resume"
        );
    }

    #[test]
    fn dragon_walking_blocked_requires_grid_wall_or_missing_floor_loc_3e3b2_3e566() {
        let mut dragon = dragon_walking_test_enemy();
        dragon.x_pos = 23 << 16;
        dragon.y_pos = 32 << 16;

        assert!(
            !dragon_walking_blocked(&dragon, None),
            "ASM Dragon.asm loc_3CE90 reaches the shared walker only after level collision data exists; native must not treat a missing grid as blocked"
        );

        let floor_grid = dragon_walking_grid(4, 4, &[(2, 2)]);
        assert!(
            !dragon_walking_blocked(&dragon, Some(&floor_grid)),
            "ASM Lion.asm loc_3E3B2/loc_3E566 shared walker should use the projected next X and keep walking when the leading foot span has floor support"
        );

        let empty_grid = dragon_walking_grid(4, 4, &[]);
        assert!(
            dragon_walking_blocked(&dragon, Some(&empty_grid)),
            "ASM Lion.asm loc_3E682/loc_3E79E should enter the airborne branch when the projected support scan has no floor"
        );

        let wall_grid = dragon_walking_grid(4, 4, &[(2, 1), (2, 2)]);
        assert!(
            dragon_walking_blocked(&dragon, Some(&wall_grid)),
            "ASM Lion.asm sub_36972 shared by Dragon should treat the right leading body cell as a wall before accepting floor support"
        );
    }

    #[test]
    fn dragon_walking_blocked_left_slope_boundary_uses_current_x_loc_3e566() {
        let mut grid = dragon_walking_grid(4, 4, &[]);
        grid.set_cell_at(1, 2, terrain::Cell::from_be_bytes(0x40, 0x00));
        let mut dragon = dragon_walking_test_enemy();
        dragon.x_pos = 41 << 16;
        dragon.y_pos = 32 << 16;
        dragon.facing = Direction::Left;

        assert!(
            dragon_walking_blocked(&dragon, Some(&grid)),
            "ASM Lion.asm loc_3E566 -> sub_36A58 should use the current X high word when deciding whether a left-moving Dragon has crossed the leading SlopeUp foot-row boundary"
        );
    }

    #[test]
    fn dragon_walking_grounded_walk_uses_shared_walker_speed_loc_3e4c6_3e67a() {
        let mut dragon = dragon_walking_test_enemy();
        dragon.x_pos = 0x0100 << 16;
        dragon.shared_walker_state = 0;

        let step = step_dragon_walking_walk(&mut dragon, None, None);

        assert!(matches!(step, DragonWalkingWalkStep::Walked));
        assert_eq!(
            dragon.x_pos,
            (0x0100 << 16) + pacer_walk_speed(EnemyFamily::DragonWalking, 3) as u32,
            "ASM Lion.asm loc_3E4C6/loc_3E67A shared walker should commit d2 = x_pos + DragonWalking HP-3 speed while calm"
        );
        assert_eq!(dragon.shared_walker_debug_branch, Some("patrol_walk"));
    }

    #[test]
    fn dragon_walking_cooldown_walk_counts_back_toward_zero_loc_3e434_3e5e8() {
        let mut dragon = dragon_walking_test_enemy();
        dragon.shared_walker_state = -2;
        dragon.x_pos = 0x0100 << 16;

        let step = step_dragon_walking_walk(&mut dragon, None, None);

        assert!(matches!(step, DragonWalkingWalkStep::Walked));
        assert_eq!(
            dragon.shared_walker_state, -1,
            "ASM Lion.asm loc_3E434/loc_3E5E8 should add one to negative shared walker cooldown states before continuing the walk"
        );
        assert_eq!(dragon.shared_walker_debug_branch, Some("cooldown_walk"));
    }

    #[test]
    fn dragon_walking_detects_and_releases_kid_at_shared_walker_bands_loc_3e450_3e476() {
        let mut calm = dragon_walking_test_enemy();
        calm.x_pos = 0x0100 << 16;
        calm.shared_walker_state = 0;

        let step = step_dragon_walking_walk(&mut calm, None, Some(0x0160 << 16));

        assert!(matches!(step, DragonWalkingWalkStep::Idled));
        assert_eq!(
            calm.shared_walker_state, SHARED_WALKER_STATE_FACE_RIGHT,
            "ASM Lion.asm loc_3E450 should enter the right-facing alert state at the inclusive `$60` detect boundary"
        );
        assert_eq!(calm.facing, Direction::Right);
        assert_eq!(calm.shared_walker_dx_to_kid_px, Some(0x60));
        assert_eq!(calm.shared_walker_debug_branch, Some("calm_face_kid"));

        let mut held = dragon_walking_test_enemy();
        held.x_pos = 0x0100 << 16;
        held.shared_walker_state = SHARED_WALKER_STATE_FACE_RIGHT;
        held.facing = Direction::Right;

        let step = step_dragon_walking_walk(&mut held, None, Some(0x0170 << 16));

        assert!(matches!(step, DragonWalkingWalkStep::Walked));
        assert_eq!(
            held.shared_walker_state, SHARED_WALKER_STATE_FACE_RIGHT,
            "ASM Lion.asm loc_3E476 should keep alert hold at the inclusive `$70` release boundary"
        );
        assert_eq!(held.facing, Direction::Right);
        assert_eq!(held.shared_walker_debug_branch, Some("alert_hold"));

        let mut released = held;
        let step = step_dragon_walking_walk(&mut released, None, Some(0x0171 << 16));

        assert!(matches!(step, DragonWalkingWalkStep::Idled));
        assert_eq!(
            released.shared_walker_state, -120,
            "ASM Lion.asm loc_3E48E/loc_3E642 should write `$FF88` when Kid leaves the alert release band"
        );
        assert_eq!(released.facing, Direction::Left);
        assert_eq!(
            released.shared_walker_debug_branch,
            Some("alert_release_turn")
        );
    }

    #[test]
    fn dragon_walking_grounded_ledge_enters_airborne_loc_3e682() {
        let grid = dragon_walking_grid(8, 4, &[]);
        let mut dragon = dragon_walking_test_enemy();
        dragon.x_pos = 0x0020 << 16;
        dragon.y_pos = 0x0020 << 16;
        dragon.facing = Direction::Right;

        let step = step_dragon_walking_walk(&mut dragon, Some(&grid), None);

        assert!(matches!(step, DragonWalkingWalkStep::StartedAirborne));
        assert!(
            !dragon.on_ground,
            "ASM Lion.asm loc_3E682 shared by Dragon should latch airborne instead of turning around when the next floor scan is empty"
        );
        assert_eq!(
            dragon.x_velocity,
            pacer_walk_speed(EnemyFamily::DragonWalking, 3)
        );
        assert_eq!(dragon.y_velocity, 0);
        assert_eq!(
            dragon.y_pos,
            0x0021 << 16,
            "ASM Lion.asm loc_3E694 should add one pixel to y_pos before entering the gravity loop"
        );
        assert_eq!(
            dragon.shared_walker_debug_branch,
            Some("patrol_airborne_start")
        );
    }

    #[test]
    fn dragon_walking_grounded_hibernate_decrements_unless_hurt_animation_is_armed_loc_3ce90() {
        let mut dragon = dragon_walking_test_enemy();
        dragon.grounded_frames = 2;
        dragon.damage_flash_frames = 0;
        dragon.x_velocity = 0x1234;
        dragon.y_velocity = -0x1234;
        let mut projectiles = Vec::new();

        step_dragon_walking(&mut dragon, None, &mut projectiles, None);

        assert_eq!(
            dragon.grounded_frames, 1,
            "ASM Dragon.asm loc_3CE90 should spend one hibernate frame before resuming ordinary walking"
        );
        assert_eq!(dragon.x_velocity, 0);
        assert_eq!(dragon.y_velocity, 0);
        assert_eq!(
            dragon.shared_walker_debug_branch,
            Some("grounded_hibernate")
        );
        assert!(projectiles.is_empty());

        dragon.grounded_frames = 2;
        dragon.damage_flash_frames = ENEMY_DAMAGE_FLASH_FRAMES;

        step_dragon_walking(&mut dragon, None, &mut projectiles, None);

        assert_eq!(
            dragon.grounded_frames, 2,
            "ASM Dragon.asm loc_3CEBE hurt hibernate is already represented by the armed damage-flash frame and must not spend the generic grounded hibernate"
        );
    }

    #[test]
    fn dragon_walking_live_breath_spawn_copies_child_payload_loc_3cf6e_3d070() {
        let mut dragon = dragon_walking_test_enemy();
        dragon.x_pos = 0x0123_8000;
        dragon.y_pos = 0x0045_4000;
        dragon.facing = Direction::Left;
        dragon.wait_timer = 1;
        dragon.dragon_script_cmd = DRAGON_WALKING_PHASE_WALK;
        let mut projectiles = Vec::new();

        step_dragon_walking(&mut dragon, None, &mut projectiles, None);

        assert_eq!(projectiles.len(), 1);
        let breath = projectiles[0];
        assert_eq!(
            breath.x_pos, 0x0107_8000,
            "ASM Dragon.asm loc_3D070 should place the child breath hitbox 28px on the clear-x_direction side"
        );
        assert_eq!(
            breath.y_pos, 0x0045_4000,
            "ASM Dragon.asm loc_3D02A should copy the parent y_pos word into the breath child"
        );
        assert_eq!(breath.x_velocity, 0);
        assert_eq!(breath.y_velocity, 0);
        assert_eq!(breath.lifetime, DRAGON_WALKING_BREATH_LIFETIME);
        assert_eq!(breath.kind, EnemyProjectileKind::DragonWalkingBreath);
        assert_eq!(dragon.wait_timer, DRAGON_WALKING_BREATH_RESET);
        assert_eq!(dragon.dragon_script_cmd, DRAGON_WALKING_PHASE_BREATH);
        assert_eq!(
            dragon.dragon_phase_frames,
            DRAGON_WALKING_BREATH_HOLD_FRAMES
        );
    }

    #[test]
    fn dragon_walking_breath_phase_ignores_blocked_terrain_until_child_done_loc_3d082() {
        let grid = dragon_walking_grid(4, 4, &[]);
        let mut dragon = dragon_walking_test_enemy();
        dragon.dragon_script_cmd = DRAGON_WALKING_PHASE_BREATH;
        dragon.dragon_phase_frames = 1;
        dragon.facing = Direction::Right;
        let mut projectiles = Vec::new();

        step_dragon_walking(&mut dragon, Some(&grid), &mut projectiles, None);

        assert_eq!(
            dragon.dragon_script_cmd, DRAGON_WALKING_PHASE_BREATH,
            "ASM Dragon.asm loc_3D082 should poll the breath child; blocked ground terrain does not run the walking turnaround while breathing"
        );
        assert_eq!(
            dragon.dragon_phase_frames, 0,
            "ASM Dragon.asm loc_3D082 should keep polling until the child animation done flag is set"
        );
        assert_eq!(dragon.facing, Direction::Right);

        step_dragon_walking(&mut dragon, Some(&grid), &mut projectiles, None);

        assert_eq!(
            dragon.dragon_script_cmd, DRAGON_WALKING_PHASE_RECOVER,
            "ASM Dragon.asm loc_3D09E should delete the child and hand the parent to breath recovery once `$18(a3)` is set"
        );
        assert_eq!(dragon.dragon_phase_frames, DRAGON_WALKING_RECOVER_FRAMES);
    }

    #[test]
    fn dragon_walking_recover_phase_counts_down_then_walks_loc_3cfe4_3d01e() {
        let mut dragon = dragon_walking_test_enemy();
        dragon.dragon_script_cmd = DRAGON_WALKING_PHASE_RECOVER;
        dragon.dragon_phase_frames = 1;
        let mut projectiles = Vec::new();

        step_dragon_walking(&mut dragon, None, &mut projectiles, None);

        assert_eq!(dragon.dragon_script_cmd, DRAGON_WALKING_PHASE_RECOVER);
        assert_eq!(
            dragon.dragon_phase_frames, 0,
            "ASM Dragon.asm loc_3CFE4 hibernates through the recovery frame before restoring the walk animation"
        );

        step_dragon_walking(&mut dragon, None, &mut projectiles, None);

        assert_eq!(
            dragon.dragon_script_cmd, DRAGON_WALKING_PHASE_WALK,
            "ASM Dragon.asm loc_3D01E should resume the ordinary walking dispatch after recovery finishes"
        );
    }

    #[test]
    fn dragon_walking_damage_route_matches_loc_3cebe() {
        assert_eq!(
            dragon_walking_damage_route_loc_3cebe(1, false),
            DragonWalkingDamageRouteLoc3CEBE::Death { remaining_hp: 0 },
            "ASM Dragon.asm loc_3CEBE branches to death when the HP decrement reaches zero"
        );
        assert_eq!(
            dragon_walking_damage_route_loc_3cebe(0, false),
            DragonWalkingDamageRouteLoc3CEBE::Death {
                remaining_hp: 0xFFFF
            },
            "ASM Dragon.asm loc_3CEBE uses signed BLE flags after wrapping the HP word"
        );
        assert_eq!(
            dragon_walking_damage_route_loc_3cebe(2, true),
            DragonWalkingDamageRouteLoc3CEBE::SpawnedChildContinue { remaining_hp: 1 },
            "ASM Dragon.asm loc_3CEBE skips the hurt animation for parent-spawned walking dragons"
        );
        assert_eq!(
            dragon_walking_damage_route_loc_3cebe(3, false),
            DragonWalkingDamageRouteLoc3CEBE::ParentHurt {
                remaining_hp: 2,
                damage_animation: "stru_3CCE6",
                restored_animation: "saved_dragon_walking_animation",
            },
            "ASM Dragon.asm loc_3CEBE plays `stru_3CCE6`, waits for it, then restores the saved walking animation"
        );
    }

    #[test]
    fn dragon_walking_terminal_route_matches_loc_3cee4() {
        assert_eq!(
            dragon_walking_terminal_route_loc_3cee4(-1, true),
            DragonWalkingTerminalRouteLoc3CEE4::DeathAnimation {
                set_has_kid_collision: false,
                animation: "stru_3CCEC",
            },
            "ASM Dragon.asm loc_3CEE4 sends `$FFFF` straight to the death animation even for spawned children"
        );
        assert_eq!(
            dragon_walking_terminal_route_loc_3cee4(DRAGON_COLID_HURT as i16, false),
            DragonWalkingTerminalRouteLoc3CEE4::DeathAnimation {
                set_has_kid_collision: true,
                animation: "stru_3CCEC",
            },
            "ASM Dragon.asm loc_3CEE4 sets has_kid_collision before ordinary DragonWalking death animation"
        );
        assert_eq!(
            dragon_walking_terminal_route_loc_3cee4(DRAGON_COLID_KIDABOVE as i16, true),
            DragonWalkingTerminalRouteLoc3CEE4::SpawnedChildResume {
                set_has_kid_collision: true,
                next_collision_type: 0xFFFE,
            },
            "ASM Dragon.asm loc_3CEE4 converts spawned-child terminal contacts into collision_type `$FFFE` and resumes"
        );
    }

    #[test]
    fn dragon_walking_death_sequence_matches_loc_3cefc() {
        assert_eq!(
            dragon_walking_death_sequence_loc_3cefc(0x05),
            DragonWalkingDeathSequenceLoc3CEFC {
                death_animation: "stru_3CCEC",
                waits_until_animation_finished: true,
                status_action: DeactivationStatusActionLoc3C4C0::WriteRespawnMarker { index: 5 },
                exits_enemy: true,
            },
            "ASM Dragon.asm loc_3CEFC should play the walking-dragon death animation, wait for it, then exit through loc_3E956"
        );
    }

    #[test]
    fn dragon_walking_berzerker_side_gate_matches_loc_3cf0e() {
        assert_eq!(
            dragon_walking_berzerker_side_gate_loc_3cf0e(false),
            DragonWalkingBerzerkerSideGateLoc3CF0E::ResumePatrol {
                next_collision_type: 0,
            },
            "ASM Dragon.asm loc_3CF0E should clear collision and resume while Berzerker_charging is zero"
        );
        assert_eq!(
            dragon_walking_berzerker_side_gate_loc_3cf0e(true),
            DragonWalkingBerzerkerSideGateLoc3CF0E::Knockback(
                DragonWalkingBerzerkerKnockbackStartLoc3CF0E {
                    has_level_collision: false,
                    has_kid_collision: true,
                    is_moved: true,
                    is_animated: false,
                    sprite_mapping: "LnkTo_unk_C755E-Data_Index",
                    x_velocity: 0,
                    y_velocity: -0x0004_8000,
                }
            ),
            "ASM Dragon.asm loc_3CF0E should seed the charging-Berzerker knockback state"
        );
    }

    #[test]
    fn dragon_walking_berzerker_airborne_loop_matches_loc_3cf3a() {
        assert_eq!(
            DRAGON_WALKING_BERZERKER_KNOCKBACK_GRAVITY_LOC_3CF3A,
            0x0000_4000
        );
        assert_eq!(
            dragon_walking_berzerker_airborne_frame_loc_3cf3a(-0x0004_8000),
            DragonWalkingBerzerkerAirborneFrameLoc3CF3A {
                applies_motion_subroutine: true,
                next_y_velocity: -0x0004_4000,
                repeats_airborne_loop: true,
            },
            "ASM Dragon.asm loc_3CF3A should add `$4000` to y_vel after each airborne movement frame"
        );
        assert_eq!(
            dragon_walking_berzerker_airborne_frame_loc_3cf3a(-0x0004_4000).next_y_velocity,
            -0x0004_0000,
            "ASM Dragon.asm loc_3CF3A should repeat the same gravity step on the next loop"
        );
    }

    #[test]
    fn dragon_walking_collision_clear_matches_loc_3cf4c() {
        assert_eq!(
            dragon_walking_collision_clear_loc_3cf4c(),
            DragonWalkingCollisionClearLoc3CF4C {
                collision_type: 0,
                falls_through_to_resume_dispatch: true,
            },
            "ASM Dragon.asm loc_3CF4C should clear collision_type before resuming the pacing dispatch"
        );
    }

    #[test]
    fn dragon_walking_spawned_resume_dispatch_matches_loc_3cf50() {
        assert_eq!(
            dragon_walking_spawned_resume_dispatch_loc_3cf50(false, Direction::Left),
            DragonWalkingSpawnedResumeDispatchLoc3CF50::OrdinaryDragonResume,
            "ASM Dragon.asm loc_3CF50 should fall through to ordinary resume when `$5C(a5)` is clear"
        );
        assert_eq!(
            dragon_walking_spawned_resume_dispatch_loc_3cf50(true, Direction::Left),
            DragonWalkingSpawnedResumeDispatchLoc3CF50::SpawnedChildLeftPacing,
            "ASM Dragon.asm loc_3CF50 should send clear x_direction spawned children to loc_3E7CA"
        );
        assert_eq!(
            dragon_walking_spawned_resume_dispatch_loc_3cf50(true, Direction::Right),
            DragonWalkingSpawnedResumeDispatchLoc3CF50::SpawnedChildRightPacing,
            "ASM Dragon.asm loc_3CF50 should send set x_direction spawned children to loc_3E6AE"
        );
    }

    #[test]
    fn dragon_walking_normal_resume_dispatch_matches_loc_3cf62() {
        assert_eq!(
            dragon_walking_normal_resume_dispatch_loc_3cf62(Direction::Left),
            DragonWalkingNormalResumeDispatchLoc3CF62::Loc3E560,
            "ASM Dragon.asm loc_3CF62 should send clear x_direction ordinary dragons to loc_3E560"
        );
        assert_eq!(
            dragon_walking_normal_resume_dispatch_loc_3cf62(Direction::Right),
            DragonWalkingNormalResumeDispatchLoc3CF62::Loc3E3AC,
            "ASM Dragon.asm loc_3CF62 should send set x_direction ordinary dragons to loc_3E3AC"
        );
    }

    #[test]
    fn dragon_walking_breath_trigger_matches_loc_3cf6e() {
        assert_eq!(
            dragon_walking_breath_trigger_loc_3cf6e(false, false),
            DragonWalkingBreathTriggerLoc3CF6E {
                breath_timer: DRAGON_WALKING_BREATH_RESET,
                phase_done_flag_5b: 0,
                play_sound: true,
                next: DragonWalkingBreathTriggerNextLoc3CF6E::AllocateBreathObject,
            },
            "ASM Dragon.asm loc_3CF6E resets `$66`, clears `$5B`, and plays flame sound when `$19(a3)` is clear"
        );
        assert!(
            !dragon_walking_breath_trigger_loc_3cf6e(false, true).play_sound,
            "ASM Dragon.asm loc_3CF6E skips sound when `$19(a3)` is set"
        );
        assert_eq!(
            dragon_walking_breath_trigger_loc_3cf6e(true, false),
            DragonWalkingBreathTriggerLoc3CF6E {
                breath_timer: DRAGON_WALKING_BREATH_RESET,
                phase_done_flag_5b: 0,
                play_sound: false,
                next: DragonWalkingBreathTriggerNextLoc3CF6E::SpawnedChildRecover,
            },
            "ASM Dragon.asm loc_3CF6E sends `$5C(a5)` spawned children directly to recovery"
        );
    }

    #[test]
    fn dragon_walking_breath_object_start_matches_loc_3cf90() {
        assert_eq!(
            dragon_walking_breath_object_start_loc_3cf90(),
            DragonWalkingBreathObjectStartLoc3CF90 {
                swaps_parent_and_child_registers: true,
                allocation_mask: 0x8000,
                child_callback: "loc_3D02A",
                parent_animation: "stru_3CCC2",
            },
            "ASM Dragon.asm loc_3CF90 allocates the breath child object and starts parent animation `stru_3CCC2`"
        );
    }

    #[test]
    fn dragon_walking_breath_child_position_matches_loc_3d070() {
        assert_eq!(
            dragon_walking_breath_child_position_loc_3d070(
                0x0123_8000,
                0x0045_4000,
                Direction::Left,
            ),
            DragonWalkingBreathChildPositionLoc3D070 {
                x_pos: 0x0107_8000,
                y_pos: 0x0045_4000,
                status_flag_13: 0xFF,
                animation_label: "stru_3CCC8",
            },
            "ASM Dragon.asm loc_3D070 should add `-$1C` pixels for clear x_direction and initialise `stru_3CCC8`"
        );
        assert_eq!(
            dragon_walking_breath_child_position_loc_3d070(
                0x0123_8000,
                0x0045_4000,
                Direction::Right,
            )
            .x_pos,
            0x013F_8000,
            "ASM Dragon.asm loc_3D070 should negate the `-$1C` offset when x_direction is set"
        );
    }

    #[test]
    fn dragon_walking_breath_child_poll_matches_loc_3d082() {
        assert_eq!(
            dragon_walking_breath_child_poll_loc_3d082(0, false),
            DragonWalkingBreathChildPollLoc3D082 {
                hibernates_one_frame: true,
                sets_parent_phase_done_flag_5b: false,
                next: DragonWalkingBreathChildPollNextLoc3D082::RepeatPoll,
            },
            "ASM Dragon.asm loc_3D082 should keep polling while parent `$5B` and child `$18` are clear"
        );
        assert_eq!(
            dragon_walking_breath_child_poll_loc_3d082(0, true),
            DragonWalkingBreathChildPollLoc3D082 {
                hibernates_one_frame: true,
                sets_parent_phase_done_flag_5b: true,
                next: DragonWalkingBreathChildPollNextLoc3D082::DeleteChild,
            },
            "ASM Dragon.asm loc_3D082 should set parent `$5B` once child animation flag `$18` is set"
        );
        assert_eq!(
            dragon_walking_breath_child_poll_loc_3d082(1, false),
            DragonWalkingBreathChildPollLoc3D082 {
                hibernates_one_frame: true,
                sets_parent_phase_done_flag_5b: false,
                next: DragonWalkingBreathChildPollNextLoc3D082::DeleteChild,
            },
            "ASM Dragon.asm loc_3D082 should branch to deletion when parent `$5B` is already non-zero"
        );
    }

    #[test]
    fn dragon_walking_breath_child_delete_matches_loc_3d09e() {
        assert_eq!(
            dragon_walking_breath_child_delete_loc_3d09e(),
            DragonWalkingBreathChildDeleteLoc3D09E {
                deletes_current_object: true,
            },
            "ASM Dragon.asm loc_3D09E should tail-jump to Delete_CurrentObject"
        );
    }

    #[test]
    fn dragon_walking_breath_marks_parent_done_loc_3cfda() {
        for (collision_type, berzerker_charging) in [
            (DRAGON_COLID_HURT, false),
            (DRAGON_COLID_KIDABOVE, false),
            (DRAGON_COLID_KIDRIGHT, true),
            (DRAGON_COLID_KIDLEFT, true),
        ] {
            assert_eq!(
                dragon_walking_breath_collision_poll_loc_3cfae(
                    collision_type as i16,
                    berzerker_charging,
                ),
                DragonWalkingBreathCollisionPollLoc3CFAE {
                    hibernates_one_frame: true,
                    sets_parent_phase_done_flag_5b: true,
                    next: DragonWalkingBreathPollNextLoc3CFAE::SharedCollisionRoute,
                },
                "ASM Dragon.asm loc_3CFDA should mark parent breath phase done before shared collision handling"
            );
        }
    }

    #[test]
    fn dragon_walking_breath_side_contacts_gate_on_berzerker_loc_3cfd4() {
        for collision_type in [DRAGON_COLID_KIDRIGHT, DRAGON_COLID_KIDLEFT] {
            assert_eq!(
                dragon_walking_breath_collision_poll_loc_3cfae(collision_type as i16, false),
                DragonWalkingBreathCollisionPollLoc3CFAE {
                    hibernates_one_frame: true,
                    sets_parent_phase_done_flag_5b: false,
                    next: DragonWalkingBreathPollNextLoc3CFAE::WaitForPhaseDoneFlag,
                },
                "ASM Dragon.asm loc_3CFD4 should ignore side contact while Berzerker_charging is clear"
            );
            assert!(
                dragon_walking_breath_collision_poll_loc_3cfae(collision_type as i16, true)
                    .sets_parent_phase_done_flag_5b,
                "ASM Dragon.asm loc_3CFD4 should allow side contact while Berzerker_charging is set"
            );
        }
    }

    #[test]
    fn dragon_walking_breath_collision_poll_matches_loc_3cfae() {
        assert_eq!(
            dragon_walking_breath_collision_poll_loc_3cfae(-1, false),
            DragonWalkingBreathCollisionPollLoc3CFAE {
                hibernates_one_frame: true,
                sets_parent_phase_done_flag_5b: false,
                next: DragonWalkingBreathPollNextLoc3CFAE::SharedCollisionRoute,
            },
            "ASM Dragon.asm loc_3CFAE sends negative collision words to the shared collision route"
        );
        assert_eq!(
            dragon_walking_breath_collision_poll_loc_3cfae(0, false),
            DragonWalkingBreathCollisionPollLoc3CFAE {
                hibernates_one_frame: true,
                sets_parent_phase_done_flag_5b: false,
                next: DragonWalkingBreathPollNextLoc3CFAE::WaitForPhaseDoneFlag,
            },
            "ASM Dragon.asm loc_3CFAE waits when collision_type is zero and the breath done flag is still clear"
        );
        assert_eq!(
            dragon_walking_breath_collision_poll_loc_3cfae(DRAGON_COLID_HURT as i16, false),
            DragonWalkingBreathCollisionPollLoc3CFAE {
                hibernates_one_frame: true,
                sets_parent_phase_done_flag_5b: true,
                next: DragonWalkingBreathPollNextLoc3CFAE::SharedCollisionRoute,
            },
            "ASM Dragon.asm loc_3CFAE marks `$5B(a1)` before routing hurt contacts"
        );
        assert!(
            dragon_walking_breath_collision_poll_loc_3cfae(DRAGON_COLID_KIDABOVE as i16, false)
                .sets_parent_phase_done_flag_5b,
            "ASM Dragon.asm loc_3CFAE treats Kid-above contacts like hurt contacts"
        );
        assert_eq!(
            dragon_walking_breath_collision_poll_loc_3cfae(DRAGON_COLID_KIDRIGHT as i16, false)
                .next,
            DragonWalkingBreathPollNextLoc3CFAE::WaitForPhaseDoneFlag,
            "ASM Dragon.asm loc_3CFAE ignores side contacts while Berzerker is not charging"
        );
        assert_eq!(
            dragon_walking_breath_collision_poll_loc_3cfae(DRAGON_COLID_KIDLEFT as i16, true),
            DragonWalkingBreathCollisionPollLoc3CFAE {
                hibernates_one_frame: true,
                sets_parent_phase_done_flag_5b: true,
                next: DragonWalkingBreathPollNextLoc3CFAE::SharedCollisionRoute,
            },
            "ASM Dragon.asm loc_3CFAE routes side contacts when Berzerker is charging"
        );
        assert_eq!(
            dragon_walking_breath_collision_poll_loc_3cfae(0x0004, true).next,
            DragonWalkingBreathPollNextLoc3CFAE::WaitForPhaseDoneFlag,
            "ASM Dragon.asm loc_3CFAE keeps unrelated positive collisions in the breath wait loop"
        );
    }

    #[test]
    fn dragon_walking_breath_done_wait_matches_loc_3cfe4() {
        assert_eq!(
            dragon_walking_breath_done_wait_loc_3cfe4(false, 2),
            DragonWalkingBreathDoneWaitLoc3CFE4 {
                hibernates_after_done_flag: false,
                next: DragonWalkingBreathDoneNextLoc3CFE4::RepeatBreathPoll,
            },
            "ASM Dragon.asm loc_3CFE4 should loop while the parent `$5B` done flag is clear"
        );
        assert_eq!(
            dragon_walking_breath_done_wait_loc_3cfe4(true, 2),
            DragonWalkingBreathDoneWaitLoc3CFE4 {
                hibernates_after_done_flag: true,
                next: DragonWalkingBreathDoneNextLoc3CFE4::RestoreWalkingAnimation {
                    animation_label: "stru_3CC90",
                },
            },
            "ASM Dragon.asm loc_3CFE4 should restore HP-3 animation when enemy_level `$40` is 2"
        );
        assert_eq!(
            dragon_walking_breath_done_wait_loc_3cfe4(true, 1),
            DragonWalkingBreathDoneWaitLoc3CFE4 {
                hibernates_after_done_flag: true,
                next: DragonWalkingBreathDoneNextLoc3CFE4::RestoreWalkingAnimation {
                    animation_label: "stru_3CC5E",
                },
            },
            "ASM Dragon.asm loc_3CFE4 should restore the normal walking animation for non-HP3 levels"
        );
    }

    #[test]
    fn dragon_walking_breath_normal_restore_matches_loc_3d002() {
        assert_eq!(
            dragon_walking_breath_normal_restore_loc_3d002(),
            DragonWalkingBreathNormalRestoreLoc3D002 {
                animation_label: "stru_3CC5E",
            },
            "ASM Dragon.asm loc_3D002 should restore the normal walking animation for non-HP3 breath recovery"
        );
        assert_eq!(
            dragon_walking_breath_done_wait_loc_3cfe4(true, 1).next,
            DragonWalkingBreathDoneNextLoc3CFE4::RestoreWalkingAnimation {
                animation_label: dragon_walking_breath_normal_restore_loc_3d002().animation_label,
            },
            "native non-HP3 breath recovery should use the loc_3D002 animation branch"
        );
    }

    #[test]
    fn dragon_walking_breath_spawned_dispatch_matches_loc_3d00c() {
        assert_eq!(
            dragon_walking_breath_spawned_dispatch_loc_3d00c(false, Direction::Left),
            DragonWalkingBreathSpawnedDispatchLoc3D00C::OrdinaryResume,
            "ASM Dragon.asm loc_3D00C should continue through ordinary resume when `$5C(a5)` is clear"
        );
        assert_eq!(
            dragon_walking_breath_spawned_dispatch_loc_3d00c(true, Direction::Left),
            DragonWalkingBreathSpawnedDispatchLoc3D00C::SpawnedChildLeftPacing,
            "ASM Dragon.asm loc_3D00C should send spawned clear x_direction dragons to loc_3E7CA"
        );
        assert_eq!(
            dragon_walking_breath_spawned_dispatch_loc_3d00c(true, Direction::Right),
            DragonWalkingBreathSpawnedDispatchLoc3D00C::SpawnedChildRightPacing,
            "ASM Dragon.asm loc_3D00C should send spawned set x_direction dragons to loc_3E6AE"
        );
    }

    #[test]
    fn dragon_walking_breath_normal_dispatch_matches_loc_3d01e() {
        assert_eq!(
            dragon_walking_breath_normal_dispatch_loc_3d01e(Direction::Left),
            DragonWalkingBreathNormalDispatchLoc3D01E::Loc3E560,
            "ASM Dragon.asm loc_3D01E should send ordinary clear x_direction dragons to loc_3E560"
        );
        assert_eq!(
            dragon_walking_breath_normal_dispatch_loc_3d01e(Direction::Right),
            DragonWalkingBreathNormalDispatchLoc3D01E::Loc3E3AC,
            "ASM Dragon.asm loc_3D01E should send ordinary set x_direction dragons to loc_3E3AC"
        );
        assert_eq!(
            dragon_walking_breath_normal_dispatch_loc_3d01e(spawn_facing(
                EnemyFamily::DragonWalking,
                200 << 16,
                50 << 16,
            )),
            DragonWalkingBreathNormalDispatchLoc3D01E::Loc3E560,
            "native DragonWalking spawn-facing should feed the same clear-x_direction branch from loc_3D01E"
        );
    }
}
