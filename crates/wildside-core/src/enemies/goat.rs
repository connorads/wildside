use super::*;

pub(crate) const GOAT_COLID_HURT: i16 = colid::HURT as i16;
pub(crate) const GOAT_COLID_KIDRIGHT: i16 = colid::KIDRIGHT as i16;
pub(crate) const GOAT_COLID_KIDLEFT: i16 = colid::KIDLEFT as i16;
pub(crate) const GOAT_COLID_KIDABOVE: i16 = colid::KIDABOVE as i16;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct GoatInitAnimationProfile {
    pub(crate) initial_animation_script: &'static str,
    pub(crate) saved_patrol_animation: &'static str,
    pub(crate) saved_damage_resume_animation: &'static str,
}

pub(crate) fn goat_hp4_init_animation_loc_3_caca(
    current_hp: u8,
) -> Option<GoatInitAnimationProfile> {
    // ASM Goat.asm loc_3CACA: HP 4 skips the high-HP branch, starts
    // `stru_3C996`, saves it in `$62(a5)`, and saves `stru_3C9B0` in `$66(a5)`.
    (current_hp == 4).then_some(GoatInitAnimationProfile {
        initial_animation_script: "stru_3C996",
        saved_patrol_animation: "stru_3C996",
        saved_damage_resume_animation: "stru_3C9B0",
    })
}

pub(crate) fn goat_high_hp_init_animation_loc_3_cae8(
    current_hp: u8,
) -> Option<GoatInitAnimationProfile> {
    // ASM Goat.asm loc_3CAE8: HP above 4 starts and saves `stru_3C996` for
    // patrol, but stores `stru_3C9CA` as the damage-resume animation.
    (current_hp > 4).then_some(GoatInitAnimationProfile {
        initial_animation_script: "stru_3C996",
        saved_patrol_animation: "stru_3C996",
        saved_damage_resume_animation: "stru_3C9CA",
    })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum GoatInitialDispatchRouteLoc3CB02 {
    ClearXDirection,
    FlippedXDirection,
}

pub(crate) fn goat_initial_dispatch_loc_3_cb02(
    facing: Direction,
) -> GoatInitialDispatchRouteLoc3CB02 {
    // ASM Goat.asm loc_3CB02: after installing the goat collision callback,
    // `tst.b x_direction(a3)` selects the clear-flag path for the default
    // left-facing art and the flipped path for right-facing goats.
    match facing {
        Direction::Left => GoatInitialDispatchRouteLoc3CB02::ClearXDirection,
        Direction::Right => GoatInitialDispatchRouteLoc3CB02::FlippedXDirection,
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum GoatCollisionRouteLoc3CB14 {
    NoCollisionClearResume,
    UnknownPositiveClearResume,
    Damage,
    BerzerkerSideGate,
    Death,
}

pub(crate) fn goat_collision_dispatch_loc_3cb14(collision_type: i16) -> GoatCollisionRouteLoc3CB14 {
    // ASM Goat.asm loc_3CB14: read `collision_type(a3)` before clearing it.
    // Signed-negative words enter the death path, zero and unknown positives
    // both resume walking through loc_3CBF6, but the ASM reaches that tail by
    // different branches (`beq` for zero, `bne` after the Kid-above compare for
    // unknown positives). Keep those routes distinct so tests pin the branch
    // shape even though live side effects are identical.
    if collision_type.is_negative() {
        return GoatCollisionRouteLoc3CB14::Death;
    }

    match collision_type {
        0 => GoatCollisionRouteLoc3CB14::NoCollisionClearResume,
        GOAT_COLID_HURT | GOAT_COLID_KIDABOVE => GoatCollisionRouteLoc3CB14::Damage,
        GOAT_COLID_KIDRIGHT | GOAT_COLID_KIDLEFT => GoatCollisionRouteLoc3CB14::BerzerkerSideGate,
        _ => GoatCollisionRouteLoc3CB14::UnknownPositiveClearResume,
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct GoatDeathSequenceLoc3CB64 {
    pub(crate) death_animation: &'static str,
    pub(crate) waits_until_animation_finished: bool,
    pub(crate) exits_enemy: bool,
}

pub(crate) fn goat_death_sequence_loc_3cb64() -> GoatDeathSequenceLoc3CB64 {
    // ASM Goat.asm loc_3CB64: initialise `stru_3C9EA`, hibernate until the
    // animation completes, then branch to the shared enemy exit routine.
    GoatDeathSequenceLoc3CB64 {
        death_animation: "stru_3C9EA",
        waits_until_animation_finished: true,
        exits_enemy: true,
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct GoatBerzerkerKnockbackStartLoc3CB76 {
    pub(crate) has_level_collision: bool,
    pub(crate) has_kid_collision: bool,
    pub(crate) is_moved: bool,
    pub(crate) is_animated: bool,
    pub(crate) sprite_mapping: &'static str,
    pub(crate) x_velocity: i32,
    pub(crate) y_velocity: i32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum GoatBerzerkerSideGateLoc3CB76 {
    ResumeSharedWalker { next_collision_type: u16 },
    AirborneKnockback(GoatBerzerkerKnockbackStartLoc3CB76),
    SideShoveThenHurtResume,
}

pub(crate) fn goat_berzerker_side_gate_loc_3cb76(
    berzerker_charging: bool,
    recoil_state_word: i16,
) -> GoatBerzerkerSideGateLoc3CB76 {
    // ASM Goat.asm loc_3CB76: clear `Berzerker_charging` returns to the
    // normal collision clear/resume path. With charge active, positive `$42`
    // branches to the shove/hurt-resume path; otherwise the goat enters the
    // airborne knockback loop with collision/animation flags and launch
    // velocity reset here.
    if !berzerker_charging {
        return GoatBerzerkerSideGateLoc3CB76::ResumeSharedWalker {
            next_collision_type: 0,
        };
    }

    if recoil_state_word > 0 {
        return GoatBerzerkerSideGateLoc3CB76::SideShoveThenHurtResume;
    }

    GoatBerzerkerSideGateLoc3CB76::AirborneKnockback(GoatBerzerkerKnockbackStartLoc3CB76 {
        has_level_collision: false,
        has_kid_collision: true,
        is_moved: true,
        is_animated: false,
        sprite_mapping: "LnkTo_unk_C7C56-Data_Index",
        x_velocity: 0,
        y_velocity: -0x0004_8000,
    })
}

pub(crate) const GOAT_BERZERKER_SIDE_SHOVE_PIXELS_LOC_3CBBA: i32 = 0x0014_0000;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct GoatBerzerkerSideShoveLoc3CBBA {
    pub(crate) x_position: i32,
    pub(crate) damage_animation: &'static str,
    pub(crate) waits_until_animation_finished: bool,
    pub(crate) recoil_state_word: i16,
    pub(crate) restored_walk_speed: i32,
    pub(crate) restored_animation: &'static str,
    pub(crate) clears_object_meta_bit_7: bool,
    pub(crate) falls_through_to_collision_clear: bool,
}

pub(crate) fn goat_berzerker_side_shove_loc_3cbba(
    facing: Direction,
    x_position: i32,
    saved_walk_speed: i32,
) -> GoatBerzerkerSideShoveLoc3CBBA {
    // ASM Goat.asm loc_3CBBA: clear `x_direction` adds 20px to the fixed-point
    // X position, flipped direction subtracts 20px, then the goat plays
    // `stru_3C9E4`, waits, restores `$42(a3)`, walk speed and patrol animation,
    // clears object_meta bit 7, and falls through to the collision clear tail.
    let x_position = match facing {
        Direction::Left => x_position.wrapping_add(GOAT_BERZERKER_SIDE_SHOVE_PIXELS_LOC_3CBBA),
        Direction::Right => {
            // ASM Goat.asm loc_3CBC8: flipped `x_direction` subtracts `$14`
            // pixels from `x_pos(a3)` before the hurt-resume sequence.
            x_position.wrapping_sub(GOAT_BERZERKER_SIDE_SHOVE_PIXELS_LOC_3CBBA)
        }
    };

    // ASM Goat.asm loc_3CBCE: initialise `stru_3C9E4`, wait for it to finish,
    // write `$FF88` to `$42(a3)`, restore `$5E/$62(a5)`, then clear
    // object_meta bit 7 before falling into collision clear.
    GoatBerzerkerSideShoveLoc3CBBA {
        x_position,
        damage_animation: "stru_3C9E4",
        waits_until_animation_finished: true,
        recoil_state_word: -0x0078,
        restored_walk_speed: saved_walk_speed,
        restored_animation: "saved_goat_patrol_animation",
        clears_object_meta_bit_7: true,
        falls_through_to_collision_clear: true,
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum GoatCollisionClearResumeRouteLoc3CBF6 {
    SpawnedChildLeft,
    SpawnedChildRight,
    OrdinaryLeft,
    OrdinaryRight,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct GoatCollisionClearResumeLoc3CBF6 {
    pub(crate) collision_type: u16,
    pub(crate) next: GoatCollisionClearResumeRouteLoc3CBF6,
}

pub(crate) fn goat_collision_clear_resume_loc_3cbf6(
    parent_spawned_child: bool,
    facing: Direction,
) -> GoatCollisionClearResumeLoc3CBF6 {
    // ASM Goat.asm loc_3CBF6: clear `collision_type(a3)`, test `$5C(a5)`,
    // then branch by `x_direction(a3)`. Spawned children use loc_3E7CA/loc_3E6AE;
    // ASM Goat.asm loc_3CC0C: ordinary goats use loc_3E560/loc_3E3AC.
    let next = match (parent_spawned_child, facing) {
        (true, Direction::Left) => GoatCollisionClearResumeRouteLoc3CBF6::SpawnedChildLeft,
        (true, Direction::Right) => GoatCollisionClearResumeRouteLoc3CBF6::SpawnedChildRight,
        (false, Direction::Left) => GoatCollisionClearResumeRouteLoc3CBF6::OrdinaryLeft,
        (false, Direction::Right) => GoatCollisionClearResumeRouteLoc3CBF6::OrdinaryRight,
    };

    GoatCollisionClearResumeLoc3CBF6 {
        collision_type: 0,
        next,
    }
}
