use super::*;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) struct OrcaInitStateLoc3D194 {
    pub(crate) status_flag_13: bool,
    pub(crate) object_meta: u16,
    pub(crate) priority: u8,
    pub(crate) current_hp: u8,
    pub(crate) walk_speed: i32,
    pub(crate) initial_lunge_timer: u16,
}

#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn orca_init_state_loc_3d194() -> OrcaInitStateLoc3D194 {
    // ASM Orca.asm loc_3D194: set `$13(a3)`, write object_meta 8,
    // clear priority, seed both speed slots with `$8000`, fix current_hp to 4,
    // and overwrite `$54(a5)` with `$12C` before entering the walker loop.
    OrcaInitStateLoc3D194 {
        status_flag_13: true,
        object_meta: 0x0008,
        priority: 0,
        current_hp: spawn_hit_points(EnemyFamily::Orca, 0),
        walk_speed: pacer_walk_speed(EnemyFamily::Orca, 4),
        initial_lunge_timer: ORCA_LUNGE_COOLDOWN,
    }
}

#[cfg_attr(not(test), allow(dead_code))]
pub(crate) const ORCA_COLID_RIGHTWALL: i16 = colid::RIGHTWALL as i16;
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) const ORCA_COLID_LEFTWALL: i16 = colid::LEFTWALL as i16;
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) const ORCA_COLID_FLOOR: i16 = colid::FLOOR as i16;
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) const ORCA_COLID_CEILING: i16 = colid::CEILING as i16;
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) const ORCA_COLID_SLOPEUP: i16 = colid::SLOPEUP as i16;
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) const ORCA_COLID_SLOPEDOWN: i16 = colid::SLOPEDOWN as i16;
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) const ORCA_COLID_HURT: i16 = colid::HURT as i16;
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) const ORCA_COLID_KIDRIGHT: i16 = colid::KIDRIGHT as i16;
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) const ORCA_COLID_KIDLEFT: i16 = colid::KIDLEFT as i16;
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) const ORCA_COLID_KIDABOVE: i16 = colid::KIDABOVE as i16;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) enum OrcaCollisionRouteLoc3D21E {
    Kill,
    Damage,
    BerzerkerSideGate,
    Continue { next_collision_type: u16 },
}

#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn orca_collision_dispatch_loc_3d21e(collision_type: i16) -> OrcaCollisionRouteLoc3D21E {
    // ASM Orca.asm loc_3D21E: negative collision words kill immediately,
    // hurt/Kid-above collisions enter the damage route, side contacts enter
    // the Berzerker-only gate, and all other non-zero IDs fall through to
    // the normal collision clear/resume path.
    if collision_type.is_negative() {
        return OrcaCollisionRouteLoc3D21E::Kill;
    }

    match collision_type {
        ORCA_COLID_HURT | ORCA_COLID_KIDABOVE => OrcaCollisionRouteLoc3D21E::Damage,
        ORCA_COLID_KIDRIGHT | ORCA_COLID_KIDLEFT => OrcaCollisionRouteLoc3D21E::BerzerkerSideGate,
        _ => OrcaCollisionRouteLoc3D21E::Continue {
            next_collision_type: 0,
        },
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) enum OrcaDamageResponseLoc3D23E {
    Kill,
    Hurt {
        remaining_hp: u16,
        damage_animation: &'static str,
        recoil_state_word: i16,
        restored_walk_speed: i32,
        restored_animation: &'static str,
    },
}

#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn orca_damage_response_loc_3d23e(
    current_hp: u16,
    saved_walk_speed: i32,
) -> OrcaDamageResponseLoc3D23E {
    // ASM Orca.asm loc_3D23E: `subq.w #1,$44(a3)` kills on zero; otherwise
    // play `stru_3D118`, set `$42(a3)` to `$FF88`, restore `$50(a5)` from
    // saved `$5E(a5)`, then resume the saved patrol animation.
    let remaining_hp = current_hp.wrapping_sub(1);
    if remaining_hp == 0 {
        OrcaDamageResponseLoc3D23E::Kill
    } else {
        OrcaDamageResponseLoc3D23E::Hurt {
            remaining_hp,
            damage_animation: "stru_3D118",
            recoil_state_word: -0x0078,
            restored_walk_speed: saved_walk_speed,
            restored_animation: "saved_orca_patrol_animation",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) struct OrcaDeathSequenceLoc3D268 {
    pub(crate) has_kid_collision: bool,
    pub(crate) death_animation: &'static str,
    pub(crate) waits_until_animation_finished: bool,
    pub(crate) exits_enemy: bool,
}

#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn orca_death_sequence_loc_3d268() -> OrcaDeathSequenceLoc3D268 {
    // ASM Orca.asm loc_3D268: `st has_kid_collision(a3)`, initialise
    // `stru_3D11E`, hibernate until that animation finishes, then branch to
    // the shared enemy exit tail.
    OrcaDeathSequenceLoc3D268 {
        has_kid_collision: true,
        death_animation: "stru_3D11E",
        waits_until_animation_finished: true,
        exits_enemy: true,
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct OrcaBerzerkerKnockbackStartLoc3D27E {
    pub(crate) has_level_collision: bool,
    pub(crate) has_kid_collision: bool,
    pub(crate) is_moved: bool,
    pub(crate) is_animated: bool,
    pub(crate) sprite_mapping: &'static str,
    pub(crate) x_velocity: i32,
    pub(crate) y_velocity: i32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum OrcaBerzerkerSideGateLoc3D27E {
    ResumePatrol { next_collision_type: u16 },
    Knockback(OrcaBerzerkerKnockbackStartLoc3D27E),
}

pub(crate) fn orca_berzerker_side_gate_loc_3d27e(
    berzerker_charging: bool,
) -> OrcaBerzerkerSideGateLoc3D27E {
    // ASM Orca.asm loc_3D27E: side contact returns to the normal clear/resume
    // path while `Berzerker_charging` is zero. A charging Berzerker disables
    // level collision/animation, re-enables Kid collision and movement, swaps
    // to `LnkTo_unk_C7656`, zeroes X velocity, and launches upward at `$FFFB8000`.
    if !berzerker_charging {
        return OrcaBerzerkerSideGateLoc3D27E::ResumePatrol {
            next_collision_type: 0,
        };
    }

    OrcaBerzerkerSideGateLoc3D27E::Knockback(OrcaBerzerkerKnockbackStartLoc3D27E {
        has_level_collision: false,
        has_kid_collision: true,
        is_moved: true,
        is_animated: false,
        sprite_mapping: "LnkTo_unk_C7656",
        x_velocity: 0,
        y_velocity: -0x0004_8000,
    })
}

pub(crate) const ORCA_BERZERKER_KNOCKBACK_GRAVITY_LOC_3D2AA: i32 = 0x0000_4000;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct OrcaBerzerkerAirborneFrameLoc3D2AA {
    pub(crate) applies_motion_subroutine: bool,
    pub(crate) next_y_velocity: i32,
    pub(crate) repeats_airborne_loop: bool,
}

pub(crate) fn orca_berzerker_airborne_frame_loc_3d2aa(
    y_velocity: i32,
) -> OrcaBerzerkerAirborneFrameLoc3D2AA {
    // ASM Orca.asm loc_3D2AA: hibernate one frame, apply the shared movement
    // subroutine with the current velocities, add `$4000` to `y_vel(a3)`, and
    // branch back to repeat the airborne knockback loop.
    OrcaBerzerkerAirborneFrameLoc3D2AA {
        applies_motion_subroutine: true,
        next_y_velocity: y_velocity.wrapping_add(ORCA_BERZERKER_KNOCKBACK_GRAVITY_LOC_3D2AA),
        repeats_airborne_loop: true,
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) enum OrcaResumeBranch {
    Loc3E7CA,
    Loc3E6AE,
    Loc3E560,
    Loc3E3AC,
    NormalResumeDispatch,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) struct OrcaCollisionClearLoc3D2BC {
    pub(crate) next_collision_type: u16,
    pub(crate) next: OrcaResumeBranch,
}

#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn orca_collision_clear_loc_3d2bc(
    wall_pinned: bool,
    facing: Direction,
) -> OrcaCollisionClearLoc3D2BC {
    // ASM Orca.asm loc_3D2BC: clear `collision_type(a3)`, then test `$5C(a5)`.
    // A wall-pinned Orca branches directly by `x_direction(a3)`; otherwise it
    // falls through to the normal patrol resume dispatch.
    let next = if wall_pinned {
        match facing {
            Direction::Left => OrcaResumeBranch::Loc3E7CA,
            Direction::Right => OrcaResumeBranch::Loc3E6AE,
        }
    } else {
        OrcaResumeBranch::NormalResumeDispatch
    };

    OrcaCollisionClearLoc3D2BC {
        next_collision_type: 0,
        next,
    }
}

#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn orca_normal_resume_loc_3d2d2(facing: Direction) -> OrcaResumeBranch {
    // ASM Orca.asm loc_3D2D2: with `$5C(a5)` clear, test `x_direction(a3)`.
    // Clear selects the left-facing shared walker path; set selects right.
    match facing {
        Direction::Left => OrcaResumeBranch::Loc3E560,
        Direction::Right => OrcaResumeBranch::Loc3E3AC,
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum OrcaLungeDecisionLoc3D2F4 {
    ResumePatrol,
    BeginWindup {
        is_moved: bool,
        is_animated: bool,
        sprite_mapping: &'static str,
        windup_frames: u8,
        launch_facing: Direction,
    },
}

pub(crate) fn orca_lunge_decision_loc_3d2f4(
    orca_x_pos: u32,
    kid_x_pos: u32,
    current_facing: Direction,
) -> OrcaLungeDecisionLoc3D2F4 {
    // ASM Orca.asm loc_3D2F4: compute signed word `orca_x - kid_x` and only
    // enter the lunge wind-up when that delta is in [-$30, $30]. The routine
    // sets movement flags and sprite mapping, but it does not update
    // `x_direction(a3)` before the later launch uses the current facing.
    let orca_x_word = (orca_x_pos >> 16) as u16;
    let kid_x_word = (kid_x_pos >> 16) as u16;
    let delta = orca_x_word.wrapping_sub(kid_x_word) as i16;
    if !(-ORCA_DETECT_RANGE..=ORCA_DETECT_RANGE).contains(&i32::from(delta)) {
        return OrcaLungeDecisionLoc3D2F4::ResumePatrol;
    }

    OrcaLungeDecisionLoc3D2F4::BeginWindup {
        is_moved: true,
        is_animated: false,
        sprite_mapping: "LnkTo_unk_C7656",
        windup_frames: orca_lunge_windup_loc_3d320(ORCA_LUNGE_WINDUP_DBF_SEED_LOC_3D320)
            .hibernate_frames,
        launch_facing: current_facing,
    }
}

pub(crate) const ORCA_LUNGE_WINDUP_DBF_SEED_LOC_3D320: u8 = 5;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct OrcaLungeWindupLoc3D320 {
    pub(crate) hibernate_frames: u8,
    pub(crate) polls_collision_each_frame: bool,
    pub(crate) branches_on_nonzero_collision: bool,
}

pub(crate) fn orca_lunge_windup_loc_3d320(dbf_seed: u8) -> OrcaLungeWindupLoc3D320 {
    // ASM Orca.asm loc_3D320: hibernate once, poll `collision_type(a3)`,
    // branch to loc_3D21E on non-zero collision, then `dbf d1,loc_3D320`.
    OrcaLungeWindupLoc3D320 {
        hibernate_frames: dbf_seed.saturating_add(1),
        polls_collision_each_frame: true,
        branches_on_nonzero_collision: true,
    }
}

pub(crate) fn orca_lunge_x_velocity_loc_3d340(facing: Direction) -> i32 {
    // ASM Orca.asm loc_3D340: when `x_direction(a3)` is set, launch right
    // with x_vel `$30000`; the clear-direction branch launches left.
    match facing {
        Direction::Right => ORCA_LUNGE_X_SPEED,
        Direction::Left => -ORCA_LUNGE_X_SPEED,
    }
}

pub(crate) fn orca_lunge_right_x_velocity_loc_3d3d2(x_velocity: i32) -> i32 {
    // ASM Orca.asm loc_3D3D2: set `x_direction(a3)` takes the right-facing
    // branch and adds `$200` to the signed long `x_vel(a3)`.
    x_velocity.wrapping_add(ORCA_LUNGE_X_ACCEL)
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct OrcaLungeLaunchTailLoc3D348 {
    pub(crate) y_velocity: i32,
    pub(crate) sprite_mapping_offset_delta: u16,
    pub(crate) has_level_collision: bool,
}

pub(crate) fn orca_lunge_launch_tail_loc_3d348() -> OrcaLungeLaunchTailLoc3D348 {
    // ASM Orca.asm loc_3D348: store y_vel `$FFFD8000`, advance
    // `addroffset_sprite(a3)` by 4, and set `has_level_collision(a3)`.
    OrcaLungeLaunchTailLoc3D348 {
        y_velocity: ORCA_LUNGE_Y_SPEED,
        sprite_mapping_offset_delta: 4,
        has_level_collision: true,
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) enum OrcaLungeCollisionRouteLoc3D35A {
    GravityAndAcceleration,
    FloorLanding,
    WallBounce,
    CeilingBounce,
    SlopeRelaunch,
    Damage,
    BerzerkerSideGate,
}

#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn orca_lunge_collision_dispatch_loc_3d35a(
    collision_type: i16,
) -> OrcaLungeCollisionRouteLoc3D35A {
    // ASM Orca.asm loc_3D35A: after one hibernated movement frame, dispatch
    // exact collision IDs; hurt/Kid-above contact jumps to loc_3D472's
    // hit-point/fatal-fall gate, side contacts jump to loc_3D4DE's
    // `Berzerker_charging` gate, and zero/unrecognised IDs fall through to
    // the loc_3D3B4 gravity/acceleration path.
    match collision_type {
        ORCA_COLID_FLOOR => OrcaLungeCollisionRouteLoc3D35A::FloorLanding,
        ORCA_COLID_RIGHTWALL | ORCA_COLID_LEFTWALL => OrcaLungeCollisionRouteLoc3D35A::WallBounce,
        ORCA_COLID_CEILING => OrcaLungeCollisionRouteLoc3D35A::CeilingBounce,
        ORCA_COLID_SLOPEUP | ORCA_COLID_SLOPEDOWN => OrcaLungeCollisionRouteLoc3D35A::SlopeRelaunch,
        ORCA_COLID_HURT | ORCA_COLID_KIDABOVE => OrcaLungeCollisionRouteLoc3D35A::Damage,
        ORCA_COLID_KIDRIGHT | ORCA_COLID_KIDLEFT => {
            OrcaLungeCollisionRouteLoc3D35A::BerzerkerSideGate
        }
        _ => OrcaLungeCollisionRouteLoc3D35A::GravityAndAcceleration,
    }
}

#[cfg_attr(not(test), allow(dead_code))]
pub(crate) const ORCA_LANDING_WAIT_DBF_SEED_LOC_3D44A: u8 = 5;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) struct OrcaLandingWaitLoc3D44A {
    pub(crate) hibernate_frames: u8,
    pub(crate) is_animated: bool,
    pub(crate) animation_script: &'static str,
    pub(crate) next_worker: &'static str,
}

#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn orca_landing_wait_loc_3d44a(dbf_seed: u8) -> OrcaLandingWaitLoc3D44A {
    // ASM Orca.asm loc_3D44A: `dbf d1,loc_3D44A` hibernates
    // `d1 + 1` frames, then `st is_animated(a3)`, initialises `stru_3D0A2`,
    // and stores `loc_3D21E` as the worker callback.
    OrcaLandingWaitLoc3D44A {
        hibernate_frames: dbf_seed.saturating_add(1),
        is_animated: true,
        animation_script: "stru_3D0A2",
        next_worker: "loc_3D21E",
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) enum OrcaLandingResumeLoc3D466 {
    Loc3E566,
    Loc3E3B2,
}

#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn orca_landing_resume_loc_3d466(facing: Direction) -> OrcaLandingResumeLoc3D466 {
    // ASM Orca.asm loc_3D466: `tst.b x_direction(a3)` branches clear
    // direction to loc_3E566 and set direction to loc_3E3B2.
    match facing {
        Direction::Left => OrcaLandingResumeLoc3D466::Loc3E566,
        Direction::Right => OrcaLandingResumeLoc3D466::Loc3E3B2,
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) struct OrcaFatalFallStartLoc3D48A {
    pub(crate) y_velocity: i32,
    pub(crate) has_kid_collision: bool,
    pub(crate) falls_through_to_landing_loop: bool,
}

#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn orca_fatal_fall_start_loc_3d48a() -> OrcaFatalFallStartLoc3D48A {
    // ASM Orca.asm loc_3D48A: after HP reaches zero, seed downward y_vel
    // `$18000`, set `has_kid_collision(a3)`, then fall through to landing.
    OrcaFatalFallStartLoc3D48A {
        y_velocity: 0x0001_8000,
        has_kid_collision: true,
        falls_through_to_landing_loop: true,
    }
}

#[cfg_attr(not(test), allow(dead_code))]
pub(crate) const ORCA_FATAL_FALL_GRAVITY_LOC_3D496: i32 = 0x0000_1800;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) enum OrcaFatalFallRouteLoc3D496 {
    Loop,
    DeathAnimation {
        animation_script: &'static str,
        waits_until_animation_finished: bool,
        exits_enemy: bool,
    },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) struct OrcaFatalFallFrameLoc3D496 {
    pub(crate) next_collision_type: i16,
    pub(crate) x_velocity: i32,
    pub(crate) y_velocity: i32,
    pub(crate) is_moved: bool,
    pub(crate) has_level_collision: bool,
    pub(crate) hibernates_one_frame: bool,
    pub(crate) route: OrcaFatalFallRouteLoc3D496,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) struct OrcaFatalFallDeathAnimationLoc3D4BC {
    pub(crate) next_collision_type: i16,
    pub(crate) x_velocity: i32,
    pub(crate) y_velocity: i32,
    pub(crate) is_moved: bool,
    pub(crate) has_level_collision: bool,
    pub(crate) animation_script: &'static str,
    pub(crate) waits_until_animation_finished: bool,
    pub(crate) exits_enemy: bool,
}

#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn orca_fatal_fall_death_animation_loc_3d4bc() -> OrcaFatalFallDeathAnimationLoc3D4BC {
    // ASM Orca.asm loc_3D4BC: after fatal-fall floor/slope contact, stop
    // both velocity components, clear movement/level collision, play
    // `stru_3D11E`, wait for it to finish, then branch to loc_3E956.
    OrcaFatalFallDeathAnimationLoc3D4BC {
        next_collision_type: 0,
        x_velocity: 0,
        y_velocity: 0,
        is_moved: false,
        has_level_collision: false,
        animation_script: "stru_3D11E",
        waits_until_animation_finished: true,
        exits_enemy: true,
    }
}

#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn orca_fatal_fall_frame_loc_3d496(
    collision_type: i16,
    x_velocity: i32,
    y_velocity: i32,
) -> OrcaFatalFallFrameLoc3D496 {
    // ASM Orca.asm loc_3D496: clear collision, hibernate once, add `$1800`
    // to y_vel, and repeat until floor/slope contact stops motion for death.
    let y_velocity_after_gravity = y_velocity.wrapping_add(ORCA_FATAL_FALL_GRAVITY_LOC_3D496);
    let landed = matches!(
        collision_type,
        ORCA_COLID_FLOOR | ORCA_COLID_SLOPEUP | ORCA_COLID_SLOPEDOWN
    );

    if landed {
        let death_animation = orca_fatal_fall_death_animation_loc_3d4bc();
        OrcaFatalFallFrameLoc3D496 {
            next_collision_type: death_animation.next_collision_type,
            x_velocity: death_animation.x_velocity,
            y_velocity: death_animation.y_velocity,
            is_moved: death_animation.is_moved,
            has_level_collision: death_animation.has_level_collision,
            hibernates_one_frame: true,
            route: OrcaFatalFallRouteLoc3D496::DeathAnimation {
                animation_script: death_animation.animation_script,
                waits_until_animation_finished: death_animation.waits_until_animation_finished,
                exits_enemy: death_animation.exits_enemy,
            },
        }
    } else {
        OrcaFatalFallFrameLoc3D496 {
            next_collision_type: 0,
            x_velocity,
            y_velocity: y_velocity_after_gravity,
            is_moved: true,
            has_level_collision: true,
            hibernates_one_frame: true,
            route: OrcaFatalFallRouteLoc3D496::Loop,
        }
    }
}

pub(crate) fn orca_fatal_fall_collision_type_loc_3d496(floor: terrain::FloorProbe) -> i16 {
    // ASM Orca.asm loc_3D496: floor, slope-up, and slope-down contacts all
    // enter loc_3D4BC, but the collision word is still classified first.
    if !floor.on_ground {
        0
    } else if floor.cell.collision == terrain::CollisionKind::SlopeUp {
        ORCA_COLID_SLOPEUP
    } else if floor.cell.collision == terrain::CollisionKind::SlopeDown {
        ORCA_COLID_SLOPEDOWN
    } else {
        ORCA_COLID_FLOOR
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
            let OrcaLungeDecisionLoc3D2F4::BeginWindup { launch_facing, .. } =
                orca_lunge_decision_loc_3d2f4(enemy.x_pos, px, enemy.facing)
            else {
                return;
            };

            // ASM Orca.asm loc_3D320: begin a six-frame hibernated wind-up
            // before loc_3D340/loc_3D348 seed the lunge velocities.
            enemy.orca_state = OrcaState::Windup;
            enemy.facing = launch_facing;
            enemy.x_velocity = 0;
            enemy.y_velocity = 0;
            enemy.wait_timer = u16::from(
                orca_lunge_windup_loc_3d320(ORCA_LUNGE_WINDUP_DBF_SEED_LOC_3D320).hibernate_frames,
            );
        }
        OrcaState::Windup => {
            if enemy.wait_timer > 1 {
                enemy.wait_timer -= 1;
                return;
            }
            enemy.wait_timer = 0;
            enemy.orca_state = OrcaState::Lunging;
            enemy.on_ground = false;
            enemy.x_velocity = orca_lunge_x_velocity_loc_3d340(enemy.facing);
            enemy.y_velocity = orca_lunge_launch_tail_loc_3d348().y_velocity;
        }
        OrcaState::Lunging => {
            // Gravity (Orca.asm loc_3D3B4: $1800 per frame)
            enemy.y_velocity = (enemy.y_velocity + ORCA_LUNGE_GRAVITY).min(physics::MAX_FALL_SPEED);

            // X acceleration during lunge unless wall-pinned via `$5C(a5)`.
            if !enemy.wall_pinned {
                enemy.x_velocity = match enemy.facing {
                    Direction::Right => orca_lunge_right_x_velocity_loc_3d3d2(enemy.x_velocity),
                    // ASM Orca.asm loc_3D3B4: clear `x_direction(a3)` subtracts
                    // `$200` from `x_vel(a3)` before looping to loc_3D35A.
                    Direction::Left => enemy.x_velocity.wrapping_sub(ORCA_LUNGE_X_ACCEL),
                };
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
                enemy.x_velocity = orca_lunge_x_velocity_loc_3d340(enemy.facing);
                enemy.y_velocity = orca_lunge_launch_tail_loc_3d348().y_velocity;
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
                if floor.on_ground {
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
        OrcaState::FatalFall => {
            // ASM Orca.asm loc_3D496: fatal lunge damage clears the current
            // collision word, hibernates one frame, moves downward, then
            // repeats until floor/slope contact starts `stru_3D11E`.
            enemy.x_pos = (enemy.x_pos as i64 + i64::from(enemy.x_velocity)) as u32;
            enemy.y_pos = (enemy.y_pos as i64 + i64::from(enemy.y_velocity)) as u32;

            let collision_type = block_grid.map_or(0, |grid| {
                let cx = (enemy.x_pos >> 16) as i32;
                let cy = (enemy.y_pos >> 16) as i32;
                let floor = terrain::probe_floor(grid, cx, cy);
                orca_fatal_fall_collision_type_loc_3d496(floor)
            });
            let frame =
                orca_fatal_fall_frame_loc_3d496(collision_type, enemy.x_velocity, enemy.y_velocity);
            enemy.x_velocity = frame.x_velocity;
            enemy.y_velocity = frame.y_velocity;
            if matches!(
                frame.route,
                OrcaFatalFallRouteLoc3D496::DeathAnimation { .. }
            ) {
                enemy.death_animation_active = true;
                enemy.wait_timer = ORCA_DEATH_ANIMATION_FRAMES;
                enemy.on_ground = true;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn orca_init_state_matches_loc_3d194() {
        let init = orca_init_state_loc_3d194();
        assert!(
            init.status_flag_13,
            "ASM Orca.asm loc_3D194 should mark byte `$13(a3)` after spawn-facing setup"
        );
        assert_eq!(init.object_meta, 0x0008);
        assert_eq!(init.priority, 0);
        assert_eq!(init.current_hp, 4);
        assert_eq!(init.walk_speed, 0x0000_8000);
        assert_eq!(init.initial_lunge_timer, 0x012C);

        let mut rng = RomRandomState::default();
        assert_eq!(
            spawn_wait_timer(EnemyFamily::Orca, 0, 0, &mut rng),
            ORCA_LUNGE_COOLDOWN,
            "ASM Orca.asm loc_3D194 seeds `$54(a5)` before loc_3D2DE can test player proximity"
        );
    }

    #[test]
    fn orca_collision_dispatch_matches_loc_3d21e_routes() {
        assert_eq!(
            orca_collision_dispatch_loc_3d21e(-1),
            OrcaCollisionRouteLoc3D21E::Kill,
            "ASM Orca.asm loc_3D21E should send negative collision words to the kill route"
        );
        assert_eq!(
            orca_collision_dispatch_loc_3d21e(ORCA_COLID_HURT),
            OrcaCollisionRouteLoc3D21E::Damage
        );
        assert_eq!(
            orca_collision_dispatch_loc_3d21e(ORCA_COLID_KIDABOVE),
            OrcaCollisionRouteLoc3D21E::Damage
        );
        assert_eq!(
            orca_collision_dispatch_loc_3d21e(ORCA_COLID_KIDRIGHT),
            OrcaCollisionRouteLoc3D21E::BerzerkerSideGate
        );
        assert_eq!(
            orca_collision_dispatch_loc_3d21e(ORCA_COLID_KIDLEFT),
            OrcaCollisionRouteLoc3D21E::BerzerkerSideGate
        );
        assert_eq!(
            orca_collision_dispatch_loc_3d21e(0x0010),
            OrcaCollisionRouteLoc3D21E::Continue {
                next_collision_type: 0
            },
            "ASM Orca.asm loc_3D21E should let unrelated collision IDs resume through the clear path"
        );
    }

    #[test]
    fn orca_damage_response_matches_loc_3d23e_hp_paths() {
        assert_eq!(
            orca_damage_response_loc_3d23e(4, 0x0000_8000),
            OrcaDamageResponseLoc3D23E::Hurt {
                remaining_hp: 3,
                damage_animation: "stru_3D118",
                recoil_state_word: -0x0078,
                restored_walk_speed: 0x0000_8000,
                restored_animation: "saved_orca_patrol_animation"
            },
            "ASM Orca.asm loc_3D23E should decrement HP, set `$42(a3)=$FF88`, and restore the saved speed"
        );
        assert_eq!(
            orca_damage_response_loc_3d23e(1, 0x0000_8000),
            OrcaDamageResponseLoc3D23E::Kill,
            "ASM Orca.asm loc_3D23E should take the kill path when the decrement reaches zero"
        );
    }

    #[test]
    fn orca_death_sequence_matches_loc_3d268() {
        assert_eq!(
            orca_death_sequence_loc_3d268(),
            OrcaDeathSequenceLoc3D268 {
                has_kid_collision: true,
                death_animation: "stru_3D11E",
                waits_until_animation_finished: true,
                exits_enemy: true,
            },
            "ASM Orca.asm loc_3D268 should re-enable Kid collision, play the death animation, then exit the enemy"
        );
    }

    #[test]
    fn orca_berzerker_side_gate_matches_loc_3d27e() {
        assert_eq!(
            orca_berzerker_side_gate_loc_3d27e(false),
            OrcaBerzerkerSideGateLoc3D27E::ResumePatrol {
                next_collision_type: 0
            },
            "ASM Orca.asm loc_3D27E should ignore side contact while Berzerker_charging is zero"
        );
        assert_eq!(
            orca_berzerker_side_gate_loc_3d27e(true),
            OrcaBerzerkerSideGateLoc3D27E::Knockback(OrcaBerzerkerKnockbackStartLoc3D27E {
                has_level_collision: false,
                has_kid_collision: true,
                is_moved: true,
                is_animated: false,
                sprite_mapping: "LnkTo_unk_C7656",
                x_velocity: 0,
                y_velocity: -0x0004_8000,
            }),
            "ASM Orca.asm loc_3D27E should seed the charging-Berzerker knockback state"
        );
    }

    #[test]
    fn orca_berzerker_airborne_loop_matches_loc_3d2aa() {
        assert_eq!(ORCA_BERZERKER_KNOCKBACK_GRAVITY_LOC_3D2AA, 0x0000_4000);
        assert_eq!(
            orca_berzerker_airborne_frame_loc_3d2aa(-0x0004_8000),
            OrcaBerzerkerAirborneFrameLoc3D2AA {
                applies_motion_subroutine: true,
                next_y_velocity: -0x0004_4000,
                repeats_airborne_loop: true,
            },
            "ASM Orca.asm loc_3D2AA should add `$4000` to y_vel after each airborne movement frame"
        );
        assert_eq!(
            orca_berzerker_airborne_frame_loc_3d2aa(-0x0004_4000).next_y_velocity,
            -0x0004_0000,
            "ASM Orca.asm loc_3D2AA should repeat the same gravity step on the next loop"
        );
    }

    #[test]
    fn orca_collision_clear_matches_loc_3d2bc() {
        assert_eq!(
            orca_collision_clear_loc_3d2bc(false, Direction::Left),
            OrcaCollisionClearLoc3D2BC {
                next_collision_type: 0,
                next: OrcaResumeBranch::NormalResumeDispatch,
            },
            "ASM Orca.asm loc_3D2BC should clear collision and fall through when `$5C(a5)` is clear"
        );
        assert_eq!(
            orca_collision_clear_loc_3d2bc(true, Direction::Left),
            OrcaCollisionClearLoc3D2BC {
                next_collision_type: 0,
                next: OrcaResumeBranch::Loc3E7CA,
            },
            "ASM Orca.asm loc_3D2BC should send clear x_direction to loc_3E7CA when wall-pinned"
        );
        assert_eq!(
            orca_collision_clear_loc_3d2bc(true, Direction::Right),
            OrcaCollisionClearLoc3D2BC {
                next_collision_type: 0,
                next: OrcaResumeBranch::Loc3E6AE,
            },
            "ASM Orca.asm loc_3D2BC should send set x_direction to loc_3E6AE when wall-pinned"
        );
    }

    #[test]
    fn orca_normal_resume_matches_loc_3d2d2() {
        assert_eq!(
            orca_normal_resume_loc_3d2d2(Direction::Left),
            OrcaResumeBranch::Loc3E560,
            "ASM Orca.asm loc_3D2D2 should send clear x_direction to loc_3E560"
        );
        assert_eq!(
            orca_normal_resume_loc_3d2d2(Direction::Right),
            OrcaResumeBranch::Loc3E3AC,
            "ASM Orca.asm loc_3D2D2 should send set x_direction to loc_3E3AC"
        );
    }

    #[test]
    fn orca_lunge_decision_matches_loc_3d2f4() {
        assert_eq!(
            orca_lunge_decision_loc_3d2f4(100 << 16, 149 << 16, Direction::Left),
            OrcaLungeDecisionLoc3D2F4::ResumePatrol,
            "ASM Orca.asm loc_3D2F4 should reject `orca_x - kid_x < -$30`"
        );
        assert_eq!(
            orca_lunge_decision_loc_3d2f4(100 << 16, 51 << 16, Direction::Right),
            OrcaLungeDecisionLoc3D2F4::ResumePatrol,
            "ASM Orca.asm loc_3D2F4 should reject `orca_x - kid_x > $30`"
        );
        assert_eq!(
            orca_lunge_decision_loc_3d2f4(100 << 16, 148 << 16, Direction::Left),
            OrcaLungeDecisionLoc3D2F4::BeginWindup {
                is_moved: true,
                is_animated: false,
                sprite_mapping: "LnkTo_unk_C7656",
                windup_frames: 6,
                launch_facing: Direction::Left,
            },
            "ASM Orca.asm loc_3D2F4 should accept the inclusive -$30 boundary without re-facing"
        );
        assert_eq!(
            orca_lunge_decision_loc_3d2f4(100 << 16, 52 << 16, Direction::Right),
            OrcaLungeDecisionLoc3D2F4::BeginWindup {
                is_moved: true,
                is_animated: false,
                sprite_mapping: "LnkTo_unk_C7656",
                windup_frames: 6,
                launch_facing: Direction::Right,
            },
            "ASM Orca.asm loc_3D2F4 should accept the inclusive $30 boundary without re-facing"
        );
    }

    #[test]
    fn orca_lunge_velocity_matches_loc_3d340_and_loc_3d3d2() {
        assert_eq!(
            orca_lunge_x_velocity_loc_3d340(Direction::Left),
            -ORCA_LUNGE_X_SPEED,
            "ASM Orca.asm loc_3D340 should launch clear x_direction left at -$30000"
        );
        assert_eq!(
            orca_lunge_x_velocity_loc_3d340(Direction::Right),
            ORCA_LUNGE_X_SPEED,
            "ASM Orca.asm loc_3D340 should launch set x_direction right at +$30000"
        );
        assert_eq!(
            orca_lunge_right_x_velocity_loc_3d3d2(ORCA_LUNGE_X_SPEED),
            ORCA_LUNGE_X_SPEED + ORCA_LUNGE_X_ACCEL,
            "ASM Orca.asm loc_3D3D2 should add `$200` to the right-facing lunge velocity"
        );
    }

    #[test]
    fn orca_lunge_collision_dispatch_matches_loc_3d35a() {
        assert_eq!(
            orca_lunge_collision_dispatch_loc_3d35a(0),
            OrcaLungeCollisionRouteLoc3D35A::GravityAndAcceleration,
            "ASM Orca.asm loc_3D35A should continue gravity/acceleration when collision_type is zero"
        );
        assert_eq!(
            orca_lunge_collision_dispatch_loc_3d35a(ORCA_COLID_FLOOR),
            OrcaLungeCollisionRouteLoc3D35A::FloorLanding,
            "ASM Orca.asm loc_3D35A should route floor collision to the landing path"
        );
        assert_eq!(
            orca_lunge_collision_dispatch_loc_3d35a(ORCA_COLID_RIGHTWALL),
            OrcaLungeCollisionRouteLoc3D35A::WallBounce,
            "ASM Orca.asm loc_3D35A should route right-wall collision to wall bounce"
        );
        assert_eq!(
            orca_lunge_collision_dispatch_loc_3d35a(ORCA_COLID_LEFTWALL),
            OrcaLungeCollisionRouteLoc3D35A::WallBounce,
            "ASM Orca.asm loc_3D35A should route left-wall collision to wall bounce"
        );
        assert_eq!(
            orca_lunge_collision_dispatch_loc_3d35a(ORCA_COLID_CEILING),
            OrcaLungeCollisionRouteLoc3D35A::CeilingBounce,
            "ASM Orca.asm loc_3D35A should route ceiling collision to vertical bounce"
        );
        assert_eq!(
            orca_lunge_collision_dispatch_loc_3d35a(ORCA_COLID_SLOPEUP),
            OrcaLungeCollisionRouteLoc3D35A::SlopeRelaunch,
            "ASM Orca.asm loc_3D35A should relaunch on slope-up contact"
        );
        assert_eq!(
            orca_lunge_collision_dispatch_loc_3d35a(ORCA_COLID_SLOPEDOWN),
            OrcaLungeCollisionRouteLoc3D35A::SlopeRelaunch,
            "ASM Orca.asm loc_3D35A should relaunch on slope-down contact"
        );
        assert_eq!(
            orca_lunge_collision_dispatch_loc_3d35a(ORCA_COLID_HURT),
            OrcaLungeCollisionRouteLoc3D35A::Damage,
            "ASM Orca.asm loc_3D35A should route hurt collision to damage"
        );
        assert_eq!(
            orca_lunge_collision_dispatch_loc_3d35a(ORCA_COLID_KIDABOVE),
            OrcaLungeCollisionRouteLoc3D35A::Damage,
            "ASM Orca.asm loc_3D35A should route Kid-above collision to damage"
        );
        assert_eq!(
            orca_lunge_collision_dispatch_loc_3d35a(ORCA_COLID_KIDRIGHT),
            OrcaLungeCollisionRouteLoc3D35A::BerzerkerSideGate,
            "ASM Orca.asm loc_3D35A should gate right-side Kid contact on Berzerker charge"
        );
        assert_eq!(
            orca_lunge_collision_dispatch_loc_3d35a(ORCA_COLID_KIDLEFT),
            OrcaLungeCollisionRouteLoc3D35A::BerzerkerSideGate,
            "ASM Orca.asm loc_3D35A should gate left-side Kid contact on Berzerker charge"
        );
        assert_eq!(
            orca_lunge_collision_dispatch_loc_3d35a(0x0030),
            OrcaLungeCollisionRouteLoc3D35A::GravityAndAcceleration,
            "ASM Orca.asm loc_3D35A should ignore unrecognised positive collision IDs"
        );
    }
}
