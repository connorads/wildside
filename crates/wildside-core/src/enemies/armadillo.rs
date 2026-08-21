use super::*;

pub(crate) const ARMADILLO_COLID_FLOOR: i16 = colid::FLOOR as i16;
pub(crate) const ARMADILLO_COLID_HURT: i16 = colid::HURT as i16;
pub(crate) const ARMADILLO_COLID_KIDRIGHT: i16 = colid::KIDRIGHT as i16;
pub(crate) const ARMADILLO_COLID_KIDLEFT: i16 = colid::KIDLEFT as i16;
pub(crate) const ARMADILLO_COLID_KIDABOVE: i16 = colid::KIDABOVE as i16;
/// ASM Armadillo animation `stru_3C6D0`: one frame held for `$14` ticks.
pub(crate) const ARMADILLO_HURT_ANIMATION_FRAMES_STRU_3C6D0: u8 = 0x14;

/// ASM Armadillo.asm loc_3C766/loc_3C7F8: `$4C(a5)` supplies the integer
/// charge step while `$5E(a5)` retains the immutable init-tier walk fraction.
pub(crate) fn armadillo_charge_step_loc_3e4ce(enemy_level: u8) -> i32 {
    let charge_word = if enemy_level >= 2 { 4 } else { 3 };
    let initial_hit_points = enemy_level.saturating_add(1);
    (charge_word << 16) | (pacer_walk_speed(EnemyFamily::Armadillo, initial_hit_points) & 0xFFFF)
}

/// Atlas phase for the live Armadillo GfxObject.
/// ASM `stru_3C632`/`stru_3C64C` use walk mappings C76EE..C7716;
/// loc_3E4CE/loc_3E50A switch to `stru_3C666`/`stru_3C680` rolling mappings
/// C76A6..C76CE; loc_3C848 temporarily replaces either with C76D6.
pub(crate) fn armadillo_live_sprite_phase(
    enemy_level: u8,
    shared_walker_state: i16,
    animation_tick: u8,
    hurt_remaining: u8,
) -> u8 {
    if hurt_remaining > 0 {
        return 12;
    }

    let charging = matches!(
        shared_walker_state,
        SHARED_WALKER_STATE_FACE_RIGHT | SHARED_WALKER_STATE_FACE_LEFT
    );
    let frame_ticks = if charging {
        if enemy_level >= 2 { 2 } else { 4 }
    } else if enemy_level == 0 {
        0x0B
    } else {
        6
    };
    let first_frame = if charging { 6 } else { 0 };
    first_frame + (animation_tick / frame_ticks) % 6
}

/// Full duration of the live animation script selected by Armadillo init or
/// the shared charge controller.
pub(crate) fn armadillo_live_animation_cycle_ticks(
    enemy_level: u8,
    shared_walker_state: i16,
) -> u8 {
    let charging = matches!(
        shared_walker_state,
        SHARED_WALKER_STATE_FACE_RIGHT | SHARED_WALKER_STATE_FACE_LEFT
    );
    if charging {
        if enemy_level >= 2 { 12 } else { 24 }
    } else if enemy_level == 0 {
        66
    } else {
        36
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct ArmadilloInitAnimationProfile {
    pub(crate) initial_animation_script: &'static str,
    pub(crate) saved_patrol_animation: &'static str,
    pub(crate) saved_damage_resume_animation: &'static str,
}

pub(crate) fn armadillo_hp2_init_animation_loc_3c7c0(
    current_hp: u8,
) -> Option<ArmadilloInitAnimationProfile> {
    // ASM Armadillo.asm loc_3C7C0: HP 2 falls through after the high-HP guard,
    // starts and saves `stru_3C64C`, and keeps `stru_3C666` for damage resume.
    (current_hp == 2).then_some(ArmadilloInitAnimationProfile {
        initial_animation_script: "stru_3C64C",
        saved_patrol_animation: "stru_3C64C",
        saved_damage_resume_animation: "stru_3C666",
    })
}

pub(crate) fn armadillo_high_hp_init_animation_loc_3c7de(
    current_hp: u8,
) -> Option<ArmadilloInitAnimationProfile> {
    // ASM Armadillo.asm loc_3C7DE: HP above 2 starts and saves `stru_3C64C`
    // for patrol, but stores `stru_3C680` as the damage-resume animation.
    (current_hp > 2).then_some(ArmadilloInitAnimationProfile {
        initial_animation_script: "stru_3C64C",
        saved_patrol_animation: "stru_3C64C",
        saved_damage_resume_animation: "stru_3C680",
    })
}

const ARMADILLO_OBJECT_META_MIRROR_TARGET_BIT: u16 = 0x0040;
const ARMADILLO_OBJECT_META_MIRROR_SOURCE_BIT: u16 = 0x0080;

/// ASM Armadillo.asm loc_3C810: `bset #6,object_meta(a3)` after bit 6 was
/// already cleared by the preceding `bclr`.
///
/// The caller guarantees bit 6 is clear. Adding the bit is equivalent to `bset`
/// under that invariant and avoids an otherwise-equivalent `|` -> `^` mutation.
#[inline]
fn armadillo_object_meta_set_mirror_target_bit_loc_3c810(cleared_object_meta: u16) -> u16 {
    cleared_object_meta.wrapping_add(ARMADILLO_OBJECT_META_MIRROR_TARGET_BIT)
}

pub(crate) fn armadillo_object_meta_sync_loc_3c810(object_meta: u16) -> u16 {
    // ASM Armadillo.asm loc_3C810: clear bit 6, then copy bit 7 into bit 6 so
    // the collision handler sees the current recoil/orientation metadata pair.
    let cleared = object_meta & !ARMADILLO_OBJECT_META_MIRROR_TARGET_BIT;
    if cleared & ARMADILLO_OBJECT_META_MIRROR_SOURCE_BIT == 0 {
        cleared
    } else {
        armadillo_object_meta_set_mirror_target_bit_loc_3c810(cleared)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct ArmadilloHurtHibernateMoveLoc3C848 {
    pub(crate) x_position: u32,
    pub(crate) y_position: u32,
}

pub(crate) fn armadillo_hurt_hibernate_move_loc_3c848(
    x_position: u32,
    y_position: u32,
    x_velocity: i32,
    y_velocity: i32,
) -> ArmadilloHurtHibernateMoveLoc3C848 {
    // ASM Armadillo.asm loc_3C848 waits in Hibernate_UntilAnimFinished while
    // kid.asm sub_1444 still calls GfxObject_Move sub_1120 once per frame.
    // The Armadillo moves, but its loc_3DEA0 patrol callback stays suspended.
    ArmadilloHurtHibernateMoveLoc3C848 {
        x_position: (i64::from(x_position) + i64::from(x_velocity)) as u32,
        y_position: (i64::from(y_position) + i64::from(y_velocity)) as u32,
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ArmadilloCollisionRouteLoc3C824 {
    ContinueSharedWalker,
    Damage,
    BerzerkerSideGate,
    Death,
}

pub(crate) fn armadillo_collision_dispatch_loc_3c824(
    collision_type: i16,
) -> ArmadilloCollisionRouteLoc3C824 {
    // ASM Armadillo.asm loc_3C824: read `collision_type(a3)` into d4 before
    // the later clear/resume tail. Negative words enter death, zero and
    // unknown positives resume walking, hurt/Kid-above contacts damage HP,
    // and side contacts branch to the Berzerker side-contact gate.
    if collision_type.is_negative() {
        return ArmadilloCollisionRouteLoc3C824::Death;
    }

    match collision_type {
        ARMADILLO_COLID_HURT | ARMADILLO_COLID_KIDABOVE => ArmadilloCollisionRouteLoc3C824::Damage,
        ARMADILLO_COLID_KIDRIGHT | ARMADILLO_COLID_KIDLEFT => {
            ArmadilloCollisionRouteLoc3C824::BerzerkerSideGate
        }
        _ => ArmadilloCollisionRouteLoc3C824::ContinueSharedWalker,
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ArmadilloDeathRouteLoc3C87E {
    DeathAnimation {
        animation_script: &'static str,
        waits_until_animation_finished: bool,
        status_action: DeactivationStatusActionLoc3C4C0,
        exits_enemy: bool,
    },
    SpawnedChildFallBeforeDeath {
        x_velocity: i32,
        y_velocity: i32,
        has_level_collision: bool,
        is_moved: bool,
    },
}

pub(crate) fn armadillo_death_route_loc_3c87e(
    spawned_child_flag: bool,
    object_status_slot: u8,
) -> ArmadilloDeathRouteLoc3C87E {
    // ASM Armadillo.asm loc_3C87E: `$5C(a5)` selects whether death starts
    // immediately or first runs a falling settle path for spawned children.
    if spawned_child_flag {
        let fall_setup = armadillo_spawned_child_fall_setup_loc_3c93c();
        ArmadilloDeathRouteLoc3C87E::SpawnedChildFallBeforeDeath {
            x_velocity: fall_setup.x_velocity,
            y_velocity: fall_setup.y_velocity,
            has_level_collision: fall_setup.has_level_collision,
            is_moved: fall_setup.is_moved,
        }
    } else {
        let death_animation = armadillo_death_animation_loc_3c886(object_status_slot);
        ArmadilloDeathRouteLoc3C87E::DeathAnimation {
            animation_script: death_animation.animation_script,
            waits_until_animation_finished: death_animation.waits_until_animation_finished,
            status_action: death_animation.status_action,
            exits_enemy: death_animation.exits_enemy,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct ArmadilloDeathAnimationLoc3C886 {
    pub(crate) animation_script: &'static str,
    pub(crate) waits_until_animation_finished: bool,
    pub(crate) status_action: DeactivationStatusActionLoc3C4C0,
    pub(crate) exits_enemy: bool,
}

pub(crate) fn armadillo_death_animation_loc_3c886(
    object_status_slot: u8,
) -> ArmadilloDeathAnimationLoc3C886 {
    // ASM Armadillo.asm loc_3C886: initialise `stru_3C69A`, hibernate until
    // that animation completes, then branch to loc_3E956's shared delete tail.
    ArmadilloDeathAnimationLoc3C886 {
        animation_script: "stru_3C69A",
        waits_until_animation_finished: true,
        status_action: deactivation_status_update_loc_3C4C0(object_status_slot),
        exits_enemy: true,
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct ArmadilloBerzerkerLaunchSetupLoc3C898 {
    pub(crate) has_level_collision: bool,
    pub(crate) has_kid_collision: bool,
    pub(crate) is_moved: bool,
    pub(crate) is_animated: bool,
    pub(crate) sprite_mapping: &'static str,
    pub(crate) x_velocity: i32,
    pub(crate) y_velocity: i32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ArmadilloBerzerkerSideContactLoc3C898 {
    ResumeSharedWalker,
    PositiveRecoilWord,
    Launch(ArmadilloBerzerkerLaunchSetupLoc3C898),
}

pub(crate) fn armadillo_berzerker_side_contact_loc_3c898(
    berzerker_charging: bool,
    recoil_state_word: i16,
) -> ArmadilloBerzerkerSideContactLoc3C898 {
    // ASM Armadillo.asm loc_3C898: clear `Berzerker_charging` resumes at
    // the shared walker tail. When set, positive `$42(a3)` branches to the
    // recoil path; zero or negative prepares the launched-armadillo loop.
    if !berzerker_charging {
        return ArmadilloBerzerkerSideContactLoc3C898::ResumeSharedWalker;
    }

    if recoil_state_word > 0 {
        return ArmadilloBerzerkerSideContactLoc3C898::PositiveRecoilWord;
    }

    ArmadilloBerzerkerSideContactLoc3C898::Launch(ArmadilloBerzerkerLaunchSetupLoc3C898 {
        has_level_collision: false,
        has_kid_collision: true,
        is_moved: true,
        is_animated: false,
        sprite_mapping: "LnkTo_unk_C76D6",
        x_velocity: 0,
        y_velocity: -0x0004_8000,
    })
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct ArmadilloBerzerkerLaunchLoopTickLoc3C8CC {
    pub(crate) hibernates_one_frame: bool,
    pub(crate) applies_shared_motion: bool,
    pub(crate) next_y_velocity: i32,
}

pub(crate) fn armadillo_berzerker_launch_loop_tick_loc_3c8cc(
    y_velocity: i32,
) -> ArmadilloBerzerkerLaunchLoopTickLoc3C8CC {
    // ASM Armadillo.asm loc_3C8CC: hibernate one frame, apply the shared
    // launched-object motion helper, add `$4000` to `y_vel(a3)`, then loop.
    ArmadilloBerzerkerLaunchLoopTickLoc3C8CC {
        hibernates_one_frame: true,
        applies_shared_motion: true,
        next_y_velocity: y_velocity.wrapping_add(0x0000_4000),
    }
}

pub(crate) const ARMADILLO_BERZERKER_RECOVERY_SHOVE_PIXELS_LOC_3C8DE: i32 = 0x0014_0000;

pub(crate) fn armadillo_berzerker_clear_direction_shove_loc_3c8de(x_position: i32) -> i32 {
    // ASM Armadillo.asm loc_3C8DE: clear `x_direction` falls through to
    // `addi.w #$14,x_pos(a3)`, nudging the integer word 20 pixels right.
    x_position.wrapping_add(ARMADILLO_BERZERKER_RECOVERY_SHOVE_PIXELS_LOC_3C8DE)
}

pub(crate) fn armadillo_berzerker_flipped_direction_shove_loc_3c8ec(x_position: i32) -> i32 {
    // ASM Armadillo.asm loc_3C8EC: flipped `x_direction` subtracts `$14`
    // from the integer word of `x_pos(a3)` before recovery animation setup.
    x_position.wrapping_sub(ARMADILLO_BERZERKER_RECOVERY_SHOVE_PIXELS_LOC_3C8DE)
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct ArmadilloBerzerkerRecoveryLoc3C8F2 {
    pub(crate) x_position: i32,
    pub(crate) damage_animation: &'static str,
    pub(crate) waits_until_animation_finished: bool,
    pub(crate) recoil_state_word: i16,
    pub(crate) restored_walk_speed: i32,
    pub(crate) restored_animation: &'static str,
    pub(crate) object_meta: u16,
    pub(crate) falls_through_to_collision_clear: bool,
}

pub(crate) fn armadillo_berzerker_recovery_loc_3c8f2(
    facing: Direction,
    x_position: i32,
    saved_walk_speed: i32,
    object_meta: u16,
) -> ArmadilloBerzerkerRecoveryLoc3C8F2 {
    let x_position = match facing {
        Direction::Left => armadillo_berzerker_clear_direction_shove_loc_3c8de(x_position),
        Direction::Right => armadillo_berzerker_flipped_direction_shove_loc_3c8ec(x_position),
    };

    // ASM Armadillo.asm loc_3C8F2: initialise `stru_3C6D6`, wait for it to
    // finish, seed `$42(a3)` with `$FF88`, restore `$5E/$62(a5)`, clear
    // object_meta bit 7, then fall through to the collision clear tail.
    ArmadilloBerzerkerRecoveryLoc3C8F2 {
        x_position,
        damage_animation: "stru_3C6D6",
        waits_until_animation_finished: true,
        recoil_state_word: -0x0078,
        restored_walk_speed: saved_walk_speed,
        restored_animation: "saved_armadillo_patrol_animation",
        object_meta: object_meta & !0x0080,
        falls_through_to_collision_clear: true,
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ArmadilloSpawnedChildResumeRouteLoc3C91A {
    SpawnedChildLeft,
    SpawnedChildRight,
    OrdinaryTail,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct ArmadilloSpawnedChildResumeLoc3C91A {
    pub(crate) collision_type: u16,
    pub(crate) next: ArmadilloSpawnedChildResumeRouteLoc3C91A,
}

pub(crate) fn armadillo_spawned_child_resume_loc_3c91a(
    parent_spawned_child: bool,
    facing: Direction,
) -> ArmadilloSpawnedChildResumeLoc3C91A {
    // ASM Armadillo.asm loc_3C91A: clear `collision_type(a3)`, then `$5C(a5)`
    // selects the spawned-child branch. Spawned children use clear `x_direction`
    // for the left shared resume and set `x_direction` for the right one.
    let next = if parent_spawned_child {
        match facing {
            Direction::Left => ArmadilloSpawnedChildResumeRouteLoc3C91A::SpawnedChildLeft,
            Direction::Right => ArmadilloSpawnedChildResumeRouteLoc3C91A::SpawnedChildRight,
        }
    } else {
        ArmadilloSpawnedChildResumeRouteLoc3C91A::OrdinaryTail
    };

    ArmadilloSpawnedChildResumeLoc3C91A {
        collision_type: 0,
        next,
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ArmadilloOrdinaryResumeRouteLoc3C930 {
    OrdinaryLeft,
    OrdinaryRight,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct ArmadilloOrdinaryResumeLoc3C930 {
    pub(crate) collision_type: u16,
    pub(crate) next: ArmadilloOrdinaryResumeRouteLoc3C930,
}

pub(crate) fn armadillo_ordinary_resume_loc_3c930(
    facing: Direction,
) -> ArmadilloOrdinaryResumeLoc3C930 {
    // ASM Armadillo.asm loc_3C930: ordinary Armadillos branch by
    // `x_direction(a3)` after the shared collision-clear tail has zeroed
    // `collision_type(a3)`.
    let next = match facing {
        Direction::Left => ArmadilloOrdinaryResumeRouteLoc3C930::OrdinaryLeft,
        Direction::Right => ArmadilloOrdinaryResumeRouteLoc3C930::OrdinaryRight,
    };

    ArmadilloOrdinaryResumeLoc3C930 {
        collision_type: 0,
        next,
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct ArmadilloSpawnedChildFallSetupLoc3C93C {
    pub(crate) x_velocity: i32,
    pub(crate) y_velocity: i32,
    pub(crate) has_level_collision: bool,
    pub(crate) is_moved: bool,
}

pub(crate) fn armadillo_spawned_child_fall_setup_loc_3c93c()
-> ArmadilloSpawnedChildFallSetupLoc3C93C {
    // ASM Armadillo.asm loc_3C93C: spawned-child death setup clears `x_vel(a3)`,
    // stores `$2C000` in `y_vel(a3)`, and enables level collision plus movement
    // before entering the floor-contact wait.
    ArmadilloSpawnedChildFallSetupLoc3C93C {
        x_velocity: 0,
        y_velocity: 0x0002_C000,
        has_level_collision: true,
        is_moved: true,
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ArmadilloSpawnedChildFallWaitRouteLoc3C954 {
    Loop,
    DeathAnimation {
        animation_script: &'static str,
        waits_until_animation_finished: bool,
        status_action: DeactivationStatusActionLoc3C4C0,
        exits_enemy: bool,
    },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct ArmadilloSpawnedChildFallWaitLoc3C954 {
    pub(crate) next_collision_type: i16,
    pub(crate) y_velocity: i32,
    pub(crate) has_level_collision: bool,
    pub(crate) is_moved: bool,
    pub(crate) hibernates_one_frame: bool,
    pub(crate) route: ArmadilloSpawnedChildFallWaitRouteLoc3C954,
}

pub(crate) fn armadillo_spawned_child_fall_wait_loc_3c954(
    collision_type: i16,
    y_velocity: i32,
    object_status_slot: u8,
) -> ArmadilloSpawnedChildFallWaitLoc3C954 {
    // ASM Armadillo.asm loc_3C954: clear stale collision_type, hibernate one
    // frame, then wait until the post-hibernate collision word is colid_floor.
    if collision_type == ARMADILLO_COLID_FLOOR {
        let death_animation = armadillo_death_animation_loc_3c886(object_status_slot);
        ArmadilloSpawnedChildFallWaitLoc3C954 {
            next_collision_type: 0,
            y_velocity: 0,
            has_level_collision: false,
            is_moved: false,
            hibernates_one_frame: true,
            route: ArmadilloSpawnedChildFallWaitRouteLoc3C954::DeathAnimation {
                animation_script: death_animation.animation_script,
                waits_until_animation_finished: death_animation.waits_until_animation_finished,
                status_action: death_animation.status_action,
                exits_enemy: death_animation.exits_enemy,
            },
        }
    } else {
        ArmadilloSpawnedChildFallWaitLoc3C954 {
            next_collision_type: 0,
            y_velocity,
            has_level_collision: true,
            is_moved: true,
            hibernates_one_frame: true,
            route: ArmadilloSpawnedChildFallWaitRouteLoc3C954::Loop,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn armadillo_charge_step_uses_init_tier_loc_3c766_3e4ce() {
        assert_eq!(armadillo_charge_step_loc_3e4ce(0), 0x0003_8000);
        assert_eq!(armadillo_charge_step_loc_3e4ce(1), 0x0003_8000);
        assert_eq!(armadillo_charge_step_loc_3e4ce(2), 0x0004_8000);
        assert_eq!(armadillo_charge_step_loc_3e4ce(3), 0x0004_8000);
    }

    #[test]
    fn armadillo_live_sprite_phase_follows_walk_charge_and_hurt_scripts() {
        // ASM stru_3C632/stru_3C64C: low-tier walk entries last 11 ticks;
        // every higher-tier walk entry lasts six ticks.
        assert_eq!(armadillo_live_sprite_phase(0, 0, 10, 0), 0);
        assert_eq!(armadillo_live_sprite_phase(0, 0, 11, 0), 1);
        assert_eq!(armadillo_live_sprite_phase(1, 0, 5, 0), 0);
        assert_eq!(armadillo_live_sprite_phase(1, 0, 6, 0), 1);

        // ASM loc_3E4CE/loc_3E50A install stru_3C666 (four ticks) below
        // enemy level 2 and stru_3C680 (two ticks) at level 2+.
        assert_eq!(
            armadillo_live_sprite_phase(0, SHARED_WALKER_STATE_FACE_RIGHT, 3, 0),
            6
        );
        assert_eq!(
            armadillo_live_sprite_phase(0, SHARED_WALKER_STATE_FACE_RIGHT, 4, 0),
            7
        );
        assert_eq!(
            armadillo_live_sprite_phase(2, SHARED_WALKER_STATE_FACE_LEFT, 2, 0),
            7
        );

        // ASM loc_3C848/stru_3C6D0 holds C76D6 for all 20 hurt ticks.
        assert_eq!(armadillo_live_sprite_phase(2, 0, 41, 1), 12);

        assert_eq!(armadillo_live_animation_cycle_ticks(0, 0), 66);
        assert_eq!(armadillo_live_animation_cycle_ticks(1, 0), 36);
        assert_eq!(
            armadillo_live_animation_cycle_ticks(0, SHARED_WALKER_STATE_FACE_RIGHT),
            24
        );
        assert_eq!(
            armadillo_live_animation_cycle_ticks(2, SHARED_WALKER_STATE_FACE_LEFT),
            12
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
    fn armadillo_hp_and_walk_speed_follow_loc_3c756_and_loc_3c766() {
        // ASM Armadillo.asm loc_3C756/loc_3C766: enemy_level is copied to d7,
        // then `addq.w #1,d7` produces HP. HP above 2 reaches the high-speed
        // path once, so extra HP tiers keep the same `$28000` walk speed.
        let hp1 = spawn_hit_points(EnemyFamily::Armadillo, 0);
        let hp2 = spawn_hit_points(EnemyFamily::Armadillo, 1);
        let hp3 = spawn_hit_points(EnemyFamily::Armadillo, 2);
        let hp4 = spawn_hit_points(EnemyFamily::Armadillo, 3);

        assert_eq!(hp1, 1);
        assert_eq!(hp2, 2);
        assert_eq!(hp3, 3);
        assert_eq!(hp4, 4);

        assert_eq!(pacer_walk_speed(EnemyFamily::Armadillo, hp1), 0x0000_8000);
        assert_eq!(pacer_walk_speed(EnemyFamily::Armadillo, hp2), 0x0001_8000);
        assert_eq!(pacer_walk_speed(EnemyFamily::Armadillo, hp3), 0x0002_8000);
        assert_eq!(pacer_walk_speed(EnemyFamily::Armadillo, hp4), 0x0002_8000);
        assert_eq!(
            armadillo_walk_speed_loc_3c756_3c766(6),
            0x0002_8000,
            "ASM Armadillo.asm loc_3C756/loc_3C766 has no third `$10000` speed add"
        );
    }

    #[test]
    fn armadillo_hp2_init_animation_matches_loc_3c7c0() {
        assert_eq!(
            armadillo_hp2_init_animation_loc_3c7c0(2),
            Some(ArmadilloInitAnimationProfile {
                initial_animation_script: "stru_3C64C",
                saved_patrol_animation: "stru_3C64C",
                saved_damage_resume_animation: "stru_3C666",
            }),
            "ASM Armadillo.asm loc_3C7C0 handles HP 2 by selecting stru_3C64C and preserving stru_3C666 as the damage-resume animation"
        );
        assert_eq!(
            armadillo_hp2_init_animation_loc_3c7c0(1),
            None,
            "ASM Armadillo.asm loc_3C7C0 is only reached after `cmpi.w #2,d7; bge.s loc_3C7C0`"
        );
    }

    #[test]
    fn armadillo_high_hp_init_animation_matches_loc_3c7de() {
        assert_eq!(
            armadillo_high_hp_init_animation_loc_3c7de(3),
            Some(ArmadilloInitAnimationProfile {
                initial_animation_script: "stru_3C64C",
                saved_patrol_animation: "stru_3C64C",
                saved_damage_resume_animation: "stru_3C680",
            }),
            "ASM Armadillo.asm loc_3C7DE keeps the fast patrol animation and swaps the damage-resume animation to stru_3C680"
        );
        assert_eq!(
            armadillo_high_hp_init_animation_loc_3c7de(2),
            None,
            "ASM Armadillo.asm loc_3C7DE is only reached when the HP word is greater than two"
        );
    }

    #[test]
    fn armadillo_object_meta_sync_matches_loc_3c810() {
        assert_eq!(
            armadillo_object_meta_sync_loc_3c810(0x0000),
            0x0000,
            "ASM Armadillo.asm loc_3C810 clears object_meta bit 6 when bit 7 is clear"
        );
        assert_eq!(
            armadillo_object_meta_sync_loc_3c810(0x0040),
            0x0000,
            "ASM Armadillo.asm loc_3C810 begins with `bclr #6,object_meta(a3)`"
        );
        assert_eq!(
            armadillo_object_meta_sync_loc_3c810(0x0080),
            0x00C0,
            "ASM Armadillo.asm loc_3C810 sets bit 6 only after `btst #7,object_meta(a3)` succeeds"
        );
        assert_eq!(
            armadillo_object_meta_sync_loc_3c810(0xA5C0),
            0xA5C0,
            "ASM Armadillo.asm loc_3C810 preserves unrelated object_meta bits while mirroring bit 7 into bit 6"
        );
    }

    #[test]
    fn armadillo_collision_dispatch_matches_loc_3c824() {
        assert_eq!(
            armadillo_collision_dispatch_loc_3c824(-1),
            ArmadilloCollisionRouteLoc3C824::Death,
            "ASM Armadillo.asm loc_3C824 sends signed-negative collision_type to the death route"
        );
        assert_eq!(
            armadillo_collision_dispatch_loc_3c824(0),
            ArmadilloCollisionRouteLoc3C824::ContinueSharedWalker,
            "ASM Armadillo.asm loc_3C824 sends zero collision_type to the normal resume route"
        );
        assert_eq!(
            armadillo_collision_dispatch_loc_3c824(ARMADILLO_COLID_HURT),
            ArmadilloCollisionRouteLoc3C824::Damage,
            "ASM Armadillo.asm loc_3C824 sends colid_hurt to the damage route"
        );
        assert_eq!(
            armadillo_collision_dispatch_loc_3c824(ARMADILLO_COLID_KIDABOVE),
            ArmadilloCollisionRouteLoc3C824::Damage,
            "ASM Armadillo.asm loc_3C824 sends colid_kidabove through the same damage path"
        );
        assert_eq!(
            armadillo_collision_dispatch_loc_3c824(ARMADILLO_COLID_KIDRIGHT),
            ArmadilloCollisionRouteLoc3C824::BerzerkerSideGate,
            "ASM Armadillo.asm loc_3C824 sends colid_kidright to the Berzerker side gate"
        );
        assert_eq!(
            armadillo_collision_dispatch_loc_3c824(ARMADILLO_COLID_KIDLEFT),
            ArmadilloCollisionRouteLoc3C824::BerzerkerSideGate,
            "ASM Armadillo.asm loc_3C824 sends colid_kidleft to the Berzerker side gate"
        );
        assert_eq!(
            armadillo_collision_dispatch_loc_3c824(0x000C),
            ArmadilloCollisionRouteLoc3C824::ContinueSharedWalker,
            "ASM Armadillo.asm loc_3C824 ignores unmatched positive collision IDs"
        );
    }

    #[test]
    fn armadillo_damage_response_matches_loc_3c848() {
        assert_eq!(
            armadillo_damage_response_loc_3c848(3, 1, 0x0002_8000),
            ArmadilloDamageResponseLoc3C848::RecoilActiveResume { current_hp: 3 },
            "ASM Armadillo.asm loc_3C848 skips the HP decrement while `$42(a3)` is positive"
        );
        assert_eq!(
            armadillo_damage_response_loc_3c848(1, 0, 0x0002_8000),
            ArmadilloDamageResponseLoc3C848::Death,
            "ASM Armadillo.asm loc_3C848 branches to death when the decremented HP word reaches zero"
        );
        assert_eq!(
            armadillo_damage_response_loc_3c848(3, 0, 0x0002_8000),
            ArmadilloDamageResponseLoc3C848::Hurt {
                remaining_hp: 2,
                damage_animation: "stru_3C6D0",
                waits_until_animation_finished: true,
                recoil_state_word: -0x0078,
                restored_walk_speed: 0x0002_8000,
                restored_animation: "saved_armadillo_patrol_animation",
            },
            "ASM Armadillo.asm loc_3C848 plays the hurt animation, seeds `$42(a3)` with `$FF88`, and restores the saved patrol speed/animation"
        );
        assert_eq!(
            armadillo_damage_response_loc_3c848(2, -1, 0x0001_8000),
            ArmadilloDamageResponseLoc3C848::Hurt {
                remaining_hp: 1,
                damage_animation: "stru_3C6D0",
                waits_until_animation_finished: true,
                recoil_state_word: -0x0078,
                restored_walk_speed: 0x0001_8000,
                restored_animation: "saved_armadillo_patrol_animation",
            },
            "ASM Armadillo.asm loc_3C848 allows zero and negative `$42(a3)` values to take damage"
        );
    }

    #[test]
    fn armadillo_death_route_matches_loc_3c87e() {
        assert_eq!(
            armadillo_death_route_loc_3c87e(false, 0x05),
            ArmadilloDeathRouteLoc3C87E::DeathAnimation {
                animation_script: "stru_3C69A",
                waits_until_animation_finished: true,
                status_action: DeactivationStatusActionLoc3C4C0::WriteRespawnMarker {
                    index: 5,
                    word_value: 0x2168
                },
                exits_enemy: true,
            },
            "ASM Armadillo.asm loc_3C87E immediately plays the death animation when `$5C(a5)` is clear"
        );
        assert_eq!(
            armadillo_death_route_loc_3c87e(true, 0x05),
            ArmadilloDeathRouteLoc3C87E::SpawnedChildFallBeforeDeath {
                x_velocity: 0,
                y_velocity: 0x0002_C000,
                has_level_collision: true,
                is_moved: true,
            },
            "ASM Armadillo.asm loc_3C87E sends set `$5C(a5)` through the falling settle route before death animation"
        );
    }

    #[test]
    fn armadillo_death_animation_matches_loc_3c886() {
        assert_eq!(
            armadillo_death_animation_loc_3c886(0x05),
            ArmadilloDeathAnimationLoc3C886 {
                animation_script: "stru_3C69A",
                waits_until_animation_finished: true,
                status_action: DeactivationStatusActionLoc3C4C0::WriteRespawnMarker {
                    index: 5,
                    word_value: 0x2168
                },
                exits_enemy: true,
            },
            "ASM Armadillo.asm loc_3C886 plays `stru_3C69A`, waits for it to finish, then exits through loc_3E956"
        );
    }

    #[test]
    fn armadillo_spawned_child_fall_wait_matches_loc_3c954() {
        assert_eq!(
            armadillo_spawned_child_fall_wait_loc_3c954(0, 0x0002_C000, 0x05),
            ArmadilloSpawnedChildFallWaitLoc3C954 {
                next_collision_type: 0,
                y_velocity: 0x0002_C000,
                has_level_collision: true,
                is_moved: true,
                hibernates_one_frame: true,
                route: ArmadilloSpawnedChildFallWaitRouteLoc3C954::Loop,
            },
            "ASM Armadillo.asm loc_3C954 clears collision_type, hibernates, and loops while collision remains zero"
        );
        assert_eq!(
            armadillo_spawned_child_fall_wait_loc_3c954(0x0004, 0x0002_C000, 0x05),
            ArmadilloSpawnedChildFallWaitLoc3C954 {
                next_collision_type: 0,
                y_velocity: 0x0002_C000,
                has_level_collision: true,
                is_moved: true,
                hibernates_one_frame: true,
                route: ArmadilloSpawnedChildFallWaitRouteLoc3C954::Loop,
            },
            "ASM Armadillo.asm loc_3C954 ignores non-floor collision words and waits for colid_floor"
        );
        assert_eq!(
            armadillo_spawned_child_fall_wait_loc_3c954(ARMADILLO_COLID_FLOOR, 0x0002_C000, 0x05),
            ArmadilloSpawnedChildFallWaitLoc3C954 {
                next_collision_type: 0,
                y_velocity: 0,
                has_level_collision: false,
                is_moved: false,
                hibernates_one_frame: true,
                route: ArmadilloSpawnedChildFallWaitRouteLoc3C954::DeathAnimation {
                    animation_script: "stru_3C69A",
                    waits_until_animation_finished: true,
                    status_action: DeactivationStatusActionLoc3C4C0::WriteRespawnMarker {
                        index: 5,
                        word_value: 0x2168,
                    },
                    exits_enemy: true,
                },
            },
            "ASM Armadillo.asm loc_3C954 exits only on colid_floor, then clears movement, y_vel, and level collision before loc_3C886"
        );
    }

    #[test]
    fn armadillo_berzerker_side_contact_matches_loc_3c898() {
        assert_eq!(
            armadillo_berzerker_side_contact_loc_3c898(false, 0),
            ArmadilloBerzerkerSideContactLoc3C898::ResumeSharedWalker,
            "ASM Armadillo.asm loc_3C898 resumes the shared walker tail while `Berzerker_charging` is clear"
        );
        assert_eq!(
            armadillo_berzerker_side_contact_loc_3c898(true, 1),
            ArmadilloBerzerkerSideContactLoc3C898::PositiveRecoilWord,
            "ASM Armadillo.asm loc_3C898 sends positive `$42(a3)` through the recoil path"
        );
        assert_eq!(
            armadillo_berzerker_side_contact_loc_3c898(true, 0),
            ArmadilloBerzerkerSideContactLoc3C898::Launch(ArmadilloBerzerkerLaunchSetupLoc3C898 {
                has_level_collision: false,
                has_kid_collision: true,
                is_moved: true,
                is_animated: false,
                sprite_mapping: "LnkTo_unk_C76D6",
                x_velocity: 0,
                y_velocity: -0x0004_8000,
            }),
            "ASM Armadillo.asm loc_3C898 prepares the Berzerker launch loop for zero or negative `$42(a3)`"
        );
        assert_eq!(
            armadillo_berzerker_side_contact_loc_3c898(true, -1),
            ArmadilloBerzerkerSideContactLoc3C898::Launch(ArmadilloBerzerkerLaunchSetupLoc3C898 {
                has_level_collision: false,
                has_kid_collision: true,
                is_moved: true,
                is_animated: false,
                sprite_mapping: "LnkTo_unk_C76D6",
                x_velocity: 0,
                y_velocity: -0x0004_8000,
            }),
            "ASM Armadillo.asm loc_3C898 uses a signed greater-than branch, so only positive `$42(a3)` avoids launch"
        );
    }

    #[test]
    fn armadillo_berzerker_launch_loop_tick_matches_loc_3c8cc() {
        assert_eq!(
            armadillo_berzerker_launch_loop_tick_loc_3c8cc(-0x0004_8000),
            ArmadilloBerzerkerLaunchLoopTickLoc3C8CC {
                hibernates_one_frame: true,
                applies_shared_motion: true,
                next_y_velocity: -0x0004_4000,
            },
            "ASM Armadillo.asm loc_3C8CC waits one frame, applies shared motion, then adds `$4000` to y_vel"
        );
        assert_eq!(
            armadillo_berzerker_launch_loop_tick_loc_3c8cc(i32::MAX),
            ArmadilloBerzerkerLaunchLoopTickLoc3C8CC {
                hibernates_one_frame: true,
                applies_shared_motion: true,
                next_y_velocity: i32::MIN + 0x3FFF,
            },
            "ASM Armadillo.asm loc_3C8CC uses long-word wrapping arithmetic for `addi.l #$4000,y_vel(a3)`"
        );
    }

    #[test]
    fn armadillo_berzerker_clear_direction_shove_matches_loc_3c8de() {
        assert_eq!(
            armadillo_berzerker_clear_direction_shove_loc_3c8de(0x0123_8000),
            0x0137_8000,
            "ASM Armadillo.asm loc_3C8DE clear x_direction should add 20px and preserve the subpixel word"
        );
        assert_eq!(
            armadillo_berzerker_clear_direction_shove_loc_3c8de(i32::MAX),
            i32::MIN + 0x0013_FFFF,
            "ASM Armadillo.asm loc_3C8DE uses word-sized position arithmetic, so native keeps wrapping behaviour"
        );
    }

    #[test]
    fn armadillo_berzerker_flipped_direction_shove_matches_loc_3c8ec() {
        assert_eq!(
            armadillo_berzerker_flipped_direction_shove_loc_3c8ec(0x0123_8000),
            0x010F_8000,
            "ASM Armadillo.asm loc_3C8EC flipped x_direction should subtract 20px and preserve the subpixel word"
        );
        assert_eq!(
            armadillo_berzerker_flipped_direction_shove_loc_3c8ec(i32::MIN),
            i32::MAX - 0x0013_FFFF,
            "ASM Armadillo.asm loc_3C8EC uses word-sized position arithmetic, so native keeps wrapping behaviour"
        );
    }

    #[test]
    fn armadillo_berzerker_recovery_matches_loc_3c8f2() {
        assert_eq!(
            armadillo_berzerker_recovery_loc_3c8f2(
                Direction::Left,
                0x0123_8000,
                0x0002_8000,
                0x00C4
            ),
            ArmadilloBerzerkerRecoveryLoc3C8F2 {
                x_position: 0x0137_8000,
                damage_animation: "stru_3C6D6",
                waits_until_animation_finished: true,
                recoil_state_word: -0x0078,
                restored_walk_speed: 0x0002_8000,
                restored_animation: "saved_armadillo_patrol_animation",
                object_meta: 0x0044,
                falls_through_to_collision_clear: true,
            },
            "ASM Armadillo.asm loc_3C8F2 should play the recovery animation, restore saved patrol state, and clear object_meta bit 7"
        );
        assert_eq!(
            armadillo_berzerker_recovery_loc_3c8f2(
                Direction::Right,
                0x0123_8000,
                -0x0001_8000,
                0x0044
            ),
            ArmadilloBerzerkerRecoveryLoc3C8F2 {
                x_position: 0x010F_8000,
                damage_animation: "stru_3C6D6",
                waits_until_animation_finished: true,
                recoil_state_word: -0x0078,
                restored_walk_speed: -0x0001_8000,
                restored_animation: "saved_armadillo_patrol_animation",
                object_meta: 0x0044,
                falls_through_to_collision_clear: true,
            },
            "ASM Armadillo.asm loc_3C8F2 should leave object_meta bit 7 clear and preserve the saved walk speed value"
        );
    }

    #[test]
    fn armadillo_initial_patrol_timer_matches_loc_3c7f8() {
        let mut rng = RomRandomState::default();
        assert_eq!(
            spawn_shared_walker_patrol_timer(EnemyFamily::Armadillo),
            0x012C,
            "ASM Armadillo.asm loc_3C7F8 seeds `$54(a5)` with `$012C` for the shared loc_3DEA0 cadence"
        );
        assert!(spawn_shared_walker_patrol_short_next(
            EnemyFamily::Armadillo
        ));
        assert_eq!(
            spawn_wait_timer(EnemyFamily::Armadillo, 0, 0, &mut rng),
            0,
            "native keeps Armadillo's live `$54(a5)` cadence out of the generic wait_timer field"
        );
    }
}
