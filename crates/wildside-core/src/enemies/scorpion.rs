use super::*;

pub(crate) const SCORPION_COLID_HURT: i16 = colid::HURT as i16;
pub(crate) const SCORPION_COLID_KIDRIGHT: i16 = colid::KIDRIGHT as i16;
pub(crate) const SCORPION_COLID_KIDLEFT: i16 = colid::KIDLEFT as i16;
pub(crate) const SCORPION_COLID_KIDABOVE: i16 = colid::KIDABOVE as i16;

/// Scorpion tracking range in pixels (Scorpion.asm line 158: $AA = 170).
/// Outside this range, speed resets to base $18000.
const SCORPION_TRACK_RANGE: i64 = 0xAA;
/// Scorpion boosted speed when within tracking range and enemy_level > 1 (HP > 3).
/// From Scorpion.asm line 164: $38000.
const SCORPION_BOOST_SPEED: i32 = 0x3_8000;
/// Scorpion base speed (always $18000, from Scorpion.asm init line 30).
const SCORPION_BASE_SPEED: i32 = 0x1_8000;
const SCORPION_STING_X_RANGE_LOC_3DDC8: i16 = 0x24;
const SCORPION_STING_Y_MIN_LOC_3DDEE: i16 = -0x20;
const SCORPION_STING_Y_MAX_LOC_3DDEE: i16 = 0x30;
const SCORPION_STING_ANIMATION_FRAMES_LOC_3DDEE: u8 = 0x06 + 0x0F + 0x06;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ScorpionCloseRangeRouteLoc3DDC8 {
    ContinueSharedWalker,
    TrackingLockedVerticalMiss,
    TrackingLockedNoStingAnimation,
    StingAnimation {
        linked_tail_has_kid_collision: bool,
        walk_speed_override: i32,
        object_meta_or_mask: u16,
        animation: &'static str,
        hibernate_until_animation_finished: bool,
    },
}

impl ScorpionCloseRangeRouteLoc3DDC8 {
    pub(crate) const fn locks_tracking(self) -> bool {
        !matches!(self, Self::ContinueSharedWalker)
    }
}

pub(crate) fn scorpion_close_range_route_loc_3ddc8(
    scorpion_x_minus_kid_x_px: i16,
    scorpion_y_minus_kid_y_px: i16,
    facing: Direction,
    enemy_level: u16,
) -> ScorpionCloseRangeRouteLoc3DDC8 {
    // ASM Scorpion.asm loc_3DDC8 keeps only d4 in [-$24, +$24], then tests
    // signed d4 against `x_direction(a3)` before loc_3DDEE locks tracking.
    if !(-SCORPION_STING_X_RANGE_LOC_3DDC8..=SCORPION_STING_X_RANGE_LOC_3DDC8)
        .contains(&scorpion_x_minus_kid_x_px)
    {
        // ASM Scorpion.asm loc_3DE64 is the shared sprite-gated resume for
        // close-range misses that continue normal shared-walker processing.
        return ScorpionCloseRangeRouteLoc3DDC8::ContinueSharedWalker;
    }

    let facing_kid = if scorpion_x_minus_kid_x_px < 0 {
        // ASM Scorpion.asm loc_3DDE6: negative d4 means Kid is right of the
        // scorpion; only set x_direction reaches loc_3DDEE.
        matches!(facing, Direction::Right)
    } else {
        // ASM Scorpion.asm loc_3DDC8: non-negative d4 falls through to
        // loc_3DDEE only when x_direction is clear.
        matches!(facing, Direction::Left)
    };
    if !facing_kid {
        return ScorpionCloseRangeRouteLoc3DDC8::ContinueSharedWalker;
    }

    // ASM Scorpion.asm loc_3DDEE sets `$4E(a5)` before the vertical and level
    // gates, so even a vertical miss suppresses future re-facing.
    if !(SCORPION_STING_Y_MIN_LOC_3DDEE..=SCORPION_STING_Y_MAX_LOC_3DDEE)
        .contains(&scorpion_y_minus_kid_y_px)
    {
        // ASM Scorpion.asm loc_3DE96 handles vertical misses after `$4E(a5)`
        // has been set, so the scorpion stays tracking-locked without stinging.
        return ScorpionCloseRangeRouteLoc3DDC8::TrackingLockedVerticalMiss;
    }

    // ASM Scorpion.asm loc_3DDEE branches to loc_3DE3C when `$40(a3) < 1`,
    // keeping the tracking lock but skipping the tail-collision sting animation.
    if enemy_level < 1 {
        return ScorpionCloseRangeRouteLoc3DDC8::TrackingLockedNoStingAnimation;
    }

    ScorpionCloseRangeRouteLoc3DDC8::StingAnimation {
        linked_tail_has_kid_collision: true,
        walk_speed_override: 0,
        object_meta_or_mask: 0xC000,
        animation: "stru_3DB8E",
        hibernate_until_animation_finished: true,
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) struct ScorpionStingRecoveryLoc3DE40 {
    pub(crate) linked_tail_has_kid_collision: bool,
    pub(crate) sprite_mapping: &'static str,
    pub(crate) animation_script: &'static str,
    pub(crate) walk_speed_override: i32,
    pub(crate) object_meta: u16,
}

#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn scorpion_sting_recovery_loc_3de40(
    saved_walk_speed_override_5e: i32,
    object_meta: u16,
) -> ScorpionStingRecoveryLoc3DE40 {
    // ASM Scorpion.asm loc_3DE40 clears the linked tail object's `$3D` flag,
    // restores the base scorpion sprite/animation, copies `$5E(a5)` back to
    // `$50(a5)`, and strips the high object_meta flags with `andi.w #$FFF`.
    ScorpionStingRecoveryLoc3DE40 {
        linked_tail_has_kid_collision: false,
        sprite_mapping: "LnkTo_unk_C7DDE-Data_Index",
        animation_script: "stru_3DB60",
        walk_speed_override: saved_walk_speed_override_5e,
        object_meta: object_meta & 0x0FFF,
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) enum ScorpionResumeRouteLoc3DE6C {
    ClearDirection,
    SetDirection,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) struct ScorpionResumeStateLoc3DE6C {
    pub(crate) cached_kid_x_word: u16,
    pub(crate) next_collision_type: u16,
    pub(crate) next: ScorpionResumeRouteLoc3DE6C,
}

#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn scorpion_resume_after_tracking_loc_3de6c(
    kid_x_word_d3: u16,
    facing: Direction,
) -> ScorpionResumeStateLoc3DE6C {
    // ASM Scorpion.asm loc_3DE6C stores `d3` in `a2` as the cached Kid X word,
    // clears `collision_type(a3)`, then dispatches by raw `x_direction(a3)`.
    ScorpionResumeStateLoc3DE6C {
        cached_kid_x_word: kid_x_word_d3,
        next_collision_type: 0,
        next: match facing {
            Direction::Left => ScorpionResumeRouteLoc3DE6C::ClearDirection,
            Direction::Right => ScorpionResumeRouteLoc3DE6C::SetDirection,
        },
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) enum ScorpionTimerRestartRouteLoc3DEA0 {
    ClearDirection,
    SetDirection,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) enum ScorpionTimerResumeRouteLoc3DECE {
    ClearDirection,
    SetDirection,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) enum ScorpionTimerRouteLoc3DEA0 {
    ResumeSharedWalker {
        next: ScorpionTimerResumeRouteLoc3DECE,
    },
    RestartPacing {
        next: ScorpionTimerRestartRouteLoc3DEA0,
    },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) struct ScorpionTimerGateLoc3DEA0 {
    pub(crate) timer_54: u16,
    pub(crate) alternating_timer_pair_6e: u32,
    pub(crate) route: ScorpionTimerRouteLoc3DEA0,
}

#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn scorpion_timer_gate_loc_3dea0(
    relative_y_word_42: i16,
    timer_54: u16,
    alternating_timer_pair_6e: u32,
    facing: Direction,
) -> ScorpionTimerGateLoc3DEA0 {
    // ASM Scorpion.asm loc_3DEA0 only advances the `$54(a5)` timer when
    // `$42(a3)` is signed <= 0 or exactly `$000A`; other values resume without
    // touching the packed timer pair in `$6E(a5)`.
    if relative_y_word_42 > 0 && relative_y_word_42 != 0x000A {
        return ScorpionTimerGateLoc3DEA0 {
            timer_54,
            alternating_timer_pair_6e,
            route: ScorpionTimerRouteLoc3DEA0::ResumeSharedWalker {
                next: scorpion_timer_resume_route_loc_3dece(facing),
            },
        };
    }

    // ASM Scorpion.asm loc_3DEAE decrements `$54(a5)` and resumes until it
    // reaches zero, then swaps the packed `$6E(a5)` timer pair.
    let timer_54 = timer_54.wrapping_sub(1);
    if timer_54 != 0 {
        return ScorpionTimerGateLoc3DEA0 {
            timer_54,
            alternating_timer_pair_6e,
            route: ScorpionTimerRouteLoc3DEA0::ResumeSharedWalker {
                next: scorpion_timer_resume_route_loc_3dece(facing),
            },
        };
    }

    let swapped_timer_pair = alternating_timer_pair_6e.rotate_left(16);
    ScorpionTimerGateLoc3DEA0 {
        timer_54: swapped_timer_pair as u16,
        alternating_timer_pair_6e: swapped_timer_pair,
        route: ScorpionTimerRouteLoc3DEA0::RestartPacing {
            next: match facing {
                Direction::Left => ScorpionTimerRestartRouteLoc3DEA0::ClearDirection,
                Direction::Right => ScorpionTimerRestartRouteLoc3DEA0::SetDirection,
            },
        },
    }
}

#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn scorpion_timer_resume_route_loc_3dece(
    facing: Direction,
) -> ScorpionTimerResumeRouteLoc3DECE {
    // ASM Scorpion.asm loc_3DECE tests raw `x_direction(a3)`: clear branches
    // to loc_3E566, set branches to loc_3E3B2.
    match facing {
        Direction::Left => ScorpionTimerResumeRouteLoc3DECE::ClearDirection,
        Direction::Right => ScorpionTimerResumeRouteLoc3DECE::SetDirection,
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ScorpionCollisionRouteLoc3DCB0 {
    ContinueSharedWalker,
    Damage,
    BerzerkerSideGate,
    Death,
}

pub(crate) fn scorpion_collision_dispatch_loc_3dcb0(
    collision_type: i16,
) -> ScorpionCollisionRouteLoc3DCB0 {
    // ASM Scorpion.asm loc_3DCB0 reads `collision_type(a3)` into d4 without
    // clearing it, then dispatches zero, negative, side, hurt, and above IDs.
    if collision_type == 0 {
        return ScorpionCollisionRouteLoc3DCB0::ContinueSharedWalker;
    }
    if collision_type.is_negative() {
        return ScorpionCollisionRouteLoc3DCB0::Death;
    }

    match collision_type {
        SCORPION_COLID_KIDRIGHT | SCORPION_COLID_KIDLEFT => {
            ScorpionCollisionRouteLoc3DCB0::BerzerkerSideGate
        }
        SCORPION_COLID_HURT | SCORPION_COLID_KIDABOVE => ScorpionCollisionRouteLoc3DCB0::Damage,
        _ => ScorpionCollisionRouteLoc3DCB0::ContinueSharedWalker,
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct ScorpionDeathRouteLoc3DCF4 {
    pub(crate) linked_tail_has_kid_collision: bool,
    pub(crate) death_animation: &'static str,
    pub(crate) hibernate_until_animation_finished: bool,
    pub(crate) delete_current_object: bool,
}

pub(crate) fn scorpion_death_route_loc_3dcf4() -> ScorpionDeathRouteLoc3DCF4 {
    // ASM Scorpion.asm loc_3DCF4: load linked tail gfx object from `$3A(a5)`,
    // set its `$3D` flag, run the scorpion death animation, then hibernate
    // until the animation completes before deleting the current object.
    ScorpionDeathRouteLoc3DCF4 {
        linked_tail_has_kid_collision: true,
        death_animation: "stru_3DB80",
        hibernate_until_animation_finished: true,
        delete_current_object: true,
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct ScorpionBerzerkerKnockbackStartLoc3DD0E {
    pub(crate) linked_tail_has_kid_collision: bool,
    pub(crate) has_level_collision: bool,
    pub(crate) has_kid_collision: bool,
    pub(crate) is_moved: bool,
    pub(crate) is_animated: bool,
    pub(crate) sprite_mapping: &'static str,
    pub(crate) x_velocity: i32,
    pub(crate) y_velocity: i32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ScorpionBerzerkerSideGateLoc3DD0E {
    ResumeSharedWalker { next_collision_type: u16 },
    Knockback(ScorpionBerzerkerKnockbackStartLoc3DD0E),
}

pub(crate) fn scorpion_berzerker_side_gate_loc_3dd0e(
    berzerker_charging: bool,
    sprite_before_sting_gate: bool,
) -> ScorpionBerzerkerSideGateLoc3DD0E {
    // ASM Scorpion.asm loc_3DD0E: side contact falls through to the normal
    // clear/resume path unless Berzerker is charging and the current sprite
    // offset is still before `LnkTo_unk_C7E0E-Data_Index`. The accepted path
    // sets the linked tail object's `$3D` flag, disables level
    // collision/animation, enables movement/Kid collision, swaps to
    // `LnkTo_unk_C7DD6`, and launches upward.
    if !berzerker_charging || !sprite_before_sting_gate {
        return ScorpionBerzerkerSideGateLoc3DD0E::ResumeSharedWalker {
            next_collision_type: 0,
        };
    }

    ScorpionBerzerkerSideGateLoc3DD0E::Knockback(ScorpionBerzerkerKnockbackStartLoc3DD0E {
        linked_tail_has_kid_collision: true,
        has_level_collision: false,
        has_kid_collision: true,
        is_moved: true,
        is_animated: false,
        sprite_mapping: "LnkTo_unk_C7DD6-Data_Index",
        x_velocity: 0,
        y_velocity: -0x0004_8000,
    })
}

pub(crate) const SCORPION_BERZERKER_KNOCKBACK_GRAVITY_LOC_3DD4A: i32 = 0x0000_4000;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct ScorpionBerzerkerAirborneFrameLoc3DD4A {
    pub(crate) hibernates_one_frame: bool,
    pub(crate) applies_motion_subroutine: bool,
    pub(crate) motion_subroutine: &'static str,
    pub(crate) next_y_velocity: i32,
    pub(crate) repeats_airborne_loop: bool,
}

pub(crate) fn scorpion_berzerker_airborne_frame_loc_3dd4a(
    y_velocity: i32,
) -> ScorpionBerzerkerAirborneFrameLoc3DD4A {
    // ASM Scorpion.asm loc_3DD4A: hibernate one frame, call `sub_3E8BA`
    // with the current velocities, add `$4000` to `y_vel(a3)`, and branch
    // back to repeat the airborne knockback loop.
    ScorpionBerzerkerAirborneFrameLoc3DD4A {
        hibernates_one_frame: true,
        applies_motion_subroutine: true,
        motion_subroutine: "sub_3E8BA",
        next_y_velocity: y_velocity.wrapping_add(SCORPION_BERZERKER_KNOCKBACK_GRAVITY_LOC_3DD4A),
        repeats_airborne_loop: true,
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ScorpionCollisionResumeBranchLoc3DD5C {
    PatrolLeft,
    PatrolRight,
    PostChargeLeft,
    PostChargeRight,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct ScorpionCollisionResumeLoc3DD5C {
    pub(crate) next_collision_type: u16,
    pub(crate) next: ScorpionCollisionResumeBranchLoc3DD5C,
}

pub(crate) fn scorpion_collision_resume_loc_3dd5c(
    post_charge_state: bool,
    facing: Direction,
) -> ScorpionCollisionResumeLoc3DD5C {
    // ASM Scorpion.asm loc_3DD5C: clear `collision_type(a3)`, test `$5C(a5)`,
    // then test `x_direction(a3)` to choose the post-charge or patrol resume
    // branch for the current facing.
    let next = match (post_charge_state, facing) {
        // ASM Scorpion.asm loc_3DD72: with `$5C(a5)` clear, a clear
        // `x_direction(a3)` resumes the left patrol branch; set resumes right.
        (false, Direction::Left) => ScorpionCollisionResumeBranchLoc3DD5C::PatrolLeft,
        (false, Direction::Right) => ScorpionCollisionResumeBranchLoc3DD5C::PatrolRight,
        (true, Direction::Left) => ScorpionCollisionResumeBranchLoc3DD5C::PostChargeLeft,
        (true, Direction::Right) => ScorpionCollisionResumeBranchLoc3DD5C::PostChargeRight,
    };

    ScorpionCollisionResumeLoc3DD5C {
        next_collision_type: 0,
        next,
    }
}

/// Scorpion tracking callback (Scorpion.asm loc_3DD7E).
///
/// Within 170px ($AA): face toward player (unless tracking-locked $4E),
/// boost speed to $38000 if enemy_level > 1 (HP > 3).
/// Outside 170px: reset speed to $18000, clear tracking lock if player airborne.
pub(crate) fn step_scorpion_tracking(
    enemy: &mut ActiveEnemy,
    player_x_pos: Option<u32>,
    player_y_pos: Option<u32>,
    player_y_velocity: Option<i32>,
) {
    let Some(px) = player_x_pos else { return };
    let dx = (i64::from(enemy.x_pos) - i64::from(px)) >> 16; // scorpion_x - player_x

    // Within tracking range: face toward player unless tracking-locked.
    // ASM loc_3DD7E lines 146-155: if $4E clear, set x_direction toward player
    // before the loc_3DDA8 range gate runs.
    if !enemy.scorpion_tracking_lock {
        if dx > 0 {
            enemy.facing = Direction::Left; // player is to the left
        } else if dx < 0 {
            // ASM Scorpion.asm loc_3DDA4: negative d4 (`scorpion_x - Kid_x`)
            // sets `x_direction(a3)`, the native right-facing scorpion state.
            enemy.facing = Direction::Right; // player is to the right
        }
    }

    // ASM Scorpion.asm loc_3DDA8: after the optional re-facing branch, compare
    // d4 against +/-$AA; outside the window exits to loc_3DE7E for base speed.
    if dx.abs() > SCORPION_TRACK_RANGE {
        enemy.walk_speed_override = SCORPION_BASE_SPEED;
        // ASM Scorpion.asm loc_3DE7E: out of range resets `$50(a5)`, then
        // checks Kid `y_vel(a4)` and clears `$4E(a5)` only when it is non-zero.
        if player_y_velocity.is_some_and(|velocity| velocity != 0) {
            enemy.scorpion_tracking_lock = false;
        }
        return;
    }

    // ASM line 162-164: if enemy_level > 1 (HP >= 5 since HP = level + 3), boost speed.
    if enemy.hit_points >= 5 {
        enemy.walk_speed_override = SCORPION_BOOST_SPEED;
    }

    let Ok(dx_word) = i16::try_from(dx) else {
        return;
    };
    let dy_word = player_y_pos
        .and_then(|py| i16::try_from((i64::from(enemy.y_pos) - i64::from(py)) >> 16).ok())
        .unwrap_or(0);
    let close_range_route = scorpion_close_range_route_loc_3ddc8(
        dx_word,
        dy_word,
        enemy.facing,
        u16::from(enemy.enemy_level),
    );
    if close_range_route.locks_tracking() {
        enemy.scorpion_tracking_lock = true;
        if let ScorpionCloseRangeRouteLoc3DDC8::StingAnimation {
            linked_tail_has_kid_collision,
            walk_speed_override,
            ..
        } = close_range_route
        {
            enemy.walk_speed_override = walk_speed_override;
            enemy.x_velocity = 0;
            enemy.y_velocity = 0;
            if linked_tail_has_kid_collision {
                enemy.scorpion_sting_frames = SCORPION_STING_ANIMATION_FRAMES_LOC_3DDEE;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::enemies::test_fixtures::*;

    fn scorpion_tracking_enemy(x_px: u32, y_px: u32, facing: Direction) -> ActiveEnemy {
        ActiveEnemy {
            family: EnemyFamily::Scorpion,
            enemy_level: 2,
            hit_points: 5,
            death_animation_active: false,
            x_pos: x_px << 16,
            y_pos: y_px << 16,
            facing,
            x_velocity: 0x0001_8000,
            walk_speed_override: SCORPION_BASE_SPEED,
            ..mini_hopping_skull_test_enemy()
        }
    }

    #[test]
    fn scorpion_sting_animation_duration_matches_stru_3db8e_loc_3ddee() {
        assert_eq!(
            SCORPION_STING_ANIMATION_FRAMES_LOC_3DDEE, 0x1B,
            "ASM Scorpion.asm loc_3DDEE hibernates through `stru_3DB8E`: `$06 + $0F + $06` frames",
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
    fn scorpion_active_snapshot_exposes_tail_child_from_loc_3dc86() {
        let scorpion = ActiveEnemy {
            family: EnemyFamily::Scorpion,
            x_pos: 0x0120_8000,
            y_pos: 0x0040_4000,
            facing: Direction::Right,
            hit_points: 3,
            death_animation_active: false,
            ..mini_hopping_skull_test_enemy()
        };

        let child = scorpion
            .child_attachment_snapshot()
            .expect("ASM Scorpion.asm loc_3DBD8 allocates a linked tail child");

        assert_eq!(
            child.kind,
            NativeObjectKind::Enemy {
                object_type: EnemyFamily::Scorpion.object_type()
            }
        );
        assert_eq!(
            child.x_pos, 0x010A_0000,
            "ASM Scorpion.asm loc_3DCAA subtracts $16 from the linked tail x word when x_direction is set"
        );
        assert_eq!(
            child.y_pos, 0x002A_0000,
            "ASM Scorpion.asm loc_3DC86 subtracts $16 from the linked tail y word"
        );
    }

    #[test]
    fn step_scorpion_tracking_arms_sting_tail_loc_3ddee() {
        let mut scorpion = ActiveEnemy {
            family: EnemyFamily::Scorpion,
            enemy_level: 2,
            hit_points: 5,
            death_animation_active: false,
            x_pos: 0x0120_0000,
            y_pos: 0x0040_0000,
            facing: Direction::Right,
            x_velocity: 0x0001_8000,
            ..mini_hopping_skull_test_enemy()
        };

        step_scorpion_tracking(&mut scorpion, Some(0x0124_0000), Some(0x0038_0000), Some(0));

        assert!(scorpion.scorpion_tracking_lock);
        assert_eq!(
            scorpion.scorpion_sting_frames, SCORPION_STING_ANIMATION_FRAMES_LOC_3DDEE,
            "ASM Scorpion.asm loc_3DDEE arms the linked tail for the whole `stru_3DB8E` animation"
        );
        assert_eq!(
            scorpion.walk_speed_override, 0,
            "ASM Scorpion.asm loc_3DDEE clears `$50(a5)` before hibernating through the sting animation"
        );
        assert_eq!(scorpion.x_velocity, 0);
        assert_eq!(scorpion.y_velocity, 0);
    }

    #[test]
    fn scorpion_close_range_continue_does_not_lock_tracking_loc_3ddc8() {
        let route = scorpion_close_range_route_loc_3ddc8(0x25, 0, Direction::Left, 1);

        assert_eq!(route, ScorpionCloseRangeRouteLoc3DDC8::ContinueSharedWalker);
        assert!(
            !route.locks_tracking(),
            "ASM Scorpion.asm loc_3DDC8 exits to loc_3DE64 outside the +/-$24 close range without setting `$4E(a5)`",
        );
    }

    #[test]
    fn step_scorpion_tracking_equal_x_preserves_facing_loc_3dd7e() {
        let mut right = scorpion_tracking_enemy(100, 64, Direction::Right);
        step_scorpion_tracking(&mut right, Some(100 << 16), Some(64 << 16), Some(0));
        assert_eq!(
            right.facing,
            Direction::Right,
            "ASM Scorpion.asm loc_3DD7E leaves x_direction unchanged when d4 == 0",
        );
        assert!(
            !right.scorpion_tracking_lock,
            "ASM Scorpion.asm loc_3DDC8 rejects d4 == 0 when the scorpion is still facing right",
        );

        let mut left = scorpion_tracking_enemy(100, 64, Direction::Left);
        step_scorpion_tracking(&mut left, Some(100 << 16), Some(64 << 16), Some(0));
        assert_eq!(
            left.facing,
            Direction::Left,
            "ASM Scorpion.asm loc_3DD7E also leaves clear x_direction unchanged when d4 == 0",
        );
        assert!(
            left.scorpion_tracking_lock,
            "ASM Scorpion.asm loc_3DDC8 accepts d4 == 0 only while x_direction is clear",
        );
    }

    #[test]
    fn step_scorpion_tracking_keeps_plus_minus_aa_inside_range_loc_3dda8() {
        let mut left_edge = scorpion_tracking_enemy(400, 64, Direction::Right);
        step_scorpion_tracking(&mut left_edge, Some(230 << 16), None, Some(0));
        assert_eq!(
            left_edge.facing,
            Direction::Left,
            "ASM Scorpion.asm loc_3DD7E clears x_direction when d4 is positive",
        );
        assert_eq!(
            left_edge.walk_speed_override, SCORPION_BOOST_SPEED,
            "ASM Scorpion.asm loc_3DDA8 keeps d4 == +$AA inside the boost window",
        );

        let mut right_edge = scorpion_tracking_enemy(400, 64, Direction::Left);
        step_scorpion_tracking(&mut right_edge, Some(570 << 16), None, Some(0));
        assert_eq!(
            right_edge.facing,
            Direction::Right,
            "ASM Scorpion.asm loc_3DDA4 sets x_direction when d4 is negative",
        );
        assert_eq!(
            right_edge.walk_speed_override, SCORPION_BOOST_SPEED,
            "ASM Scorpion.asm loc_3DDA8 keeps d4 == -$AA inside the boost window",
        );

        let mut outside = scorpion_tracking_enemy(400, 64, Direction::Right);
        outside.walk_speed_override = SCORPION_BOOST_SPEED;
        step_scorpion_tracking(&mut outside, Some(229 << 16), None, Some(0));
        assert_eq!(
            outside.walk_speed_override, SCORPION_BASE_SPEED,
            "ASM Scorpion.asm loc_3DDA8 exits to loc_3DE7E only when d4 is greater than +$AA",
        );
    }

    #[test]
    fn step_scorpion_tracking_vertical_miss_locks_without_sting_loc_3ddee() {
        let mut scorpion = scorpion_tracking_enemy(100, 64, Direction::Left);
        scorpion.x_velocity = 0x0001_2222;
        scorpion.y_velocity = 0x0000_3333;

        step_scorpion_tracking(&mut scorpion, Some(100 << 16), Some(97 << 16), Some(0));

        assert!(
            scorpion.scorpion_tracking_lock,
            "ASM Scorpion.asm loc_3DDEE sets `$4E(a5)` before testing the vertical sting window",
        );
        assert_eq!(
            scorpion.scorpion_sting_frames, 0,
            "ASM Scorpion.asm loc_3DE96 handles d5 below -$20 without starting `stru_3DB8E`",
        );
        assert_eq!(scorpion.x_velocity, 0x0001_2222);
        assert_eq!(scorpion.y_velocity, 0x0000_3333);
    }

    #[test]
    fn scorpion_collision_dispatch_matches_loc_3dcb0_compare_order() {
        assert_eq!(
            scorpion_collision_dispatch_loc_3dcb0(0),
            ScorpionCollisionRouteLoc3DCB0::ContinueSharedWalker,
            "ASM Scorpion.asm loc_3DCB0 sends zero collision_type to the normal shared-walker resume path"
        );
        assert_eq!(
            scorpion_collision_dispatch_loc_3dcb0(-1),
            ScorpionCollisionRouteLoc3DCB0::Death,
            "ASM Scorpion.asm loc_3DCB0 sends signed-negative collision words to the death path before positive ID checks"
        );
        assert_eq!(
            scorpion_collision_dispatch_loc_3dcb0(SCORPION_COLID_KIDRIGHT),
            ScorpionCollisionRouteLoc3DCB0::BerzerkerSideGate,
            "ASM Scorpion.asm loc_3DCB0 routes kid-right side contact to the Berzerker side gate"
        );
        assert_eq!(
            scorpion_collision_dispatch_loc_3dcb0(SCORPION_COLID_KIDLEFT),
            ScorpionCollisionRouteLoc3DCB0::BerzerkerSideGate,
            "ASM Scorpion.asm loc_3DCB0 routes kid-left side contact to the same Berzerker side gate"
        );
        assert_eq!(
            scorpion_collision_dispatch_loc_3dcb0(SCORPION_COLID_HURT),
            ScorpionCollisionRouteLoc3DCB0::Damage,
            "ASM Scorpion.asm loc_3DCB0 routes hurt collision to HP damage"
        );
        assert_eq!(
            scorpion_collision_dispatch_loc_3dcb0(SCORPION_COLID_KIDABOVE),
            ScorpionCollisionRouteLoc3DCB0::Damage,
            "ASM Scorpion.asm loc_3DCB0 routes kid-above collision to the same HP damage path"
        );
        assert_eq!(
            scorpion_collision_dispatch_loc_3dcb0(0x0010),
            ScorpionCollisionRouteLoc3DCB0::ContinueSharedWalker,
            "ASM Scorpion.asm loc_3DCB0 ignores unhandled positive collision IDs"
        );
    }

    #[test]
    fn scorpion_damage_route_matches_loc_3dcd4_hp_decrement() {
        assert_eq!(
            scorpion_damage_route_loc_3dcd4(3),
            ScorpionDamageRouteLoc3DCD4::Hurt {
                remaining_hp: 2,
                damage_animation: "stru_3DB7A",
                restored_animation: "stru_3DB60",
            },
            "ASM Scorpion.asm loc_3DCD4 decrements current_hp, plays the hurt animation, then restores the base animation while HP remains"
        );
        assert_eq!(
            scorpion_damage_route_loc_3dcd4(1),
            ScorpionDamageRouteLoc3DCD4::Death,
            "ASM Scorpion.asm loc_3DCD4 branches to the death path when the decremented current_hp word reaches zero"
        );
    }

    #[test]
    fn scorpion_death_route_matches_loc_3dcf4_tail_flag_and_animation() {
        assert_eq!(
            scorpion_death_route_loc_3dcf4(),
            ScorpionDeathRouteLoc3DCF4 {
                linked_tail_has_kid_collision: true,
                death_animation: "stru_3DB80",
                hibernate_until_animation_finished: true,
                delete_current_object: true,
            },
            "ASM Scorpion.asm loc_3DCF4 sets the linked tail object's $3D flag, plays the death animation, waits for it, then deletes the current object"
        );
    }

    #[test]
    fn scorpion_berzerker_side_gate_loc_3dd0e_requires_charge_and_sprite_gate() {
        assert_eq!(
            scorpion_berzerker_side_gate_loc_3dd0e(false, true),
            ScorpionBerzerkerSideGateLoc3DD0E::ResumeSharedWalker {
                next_collision_type: 0,
            },
            "ASM Scorpion.asm loc_3DD0E branches to the normal clear/resume path when Berzerker_charging is zero"
        );
        assert_eq!(
            scorpion_berzerker_side_gate_loc_3dd0e(true, false),
            ScorpionBerzerkerSideGateLoc3DD0E::ResumeSharedWalker {
                next_collision_type: 0,
            },
            "ASM Scorpion.asm loc_3DD0E branches to the normal clear/resume path when addroffset_sprite is at or after LnkTo_unk_C7E0E"
        );
        assert_eq!(
            scorpion_berzerker_side_gate_loc_3dd0e(true, true),
            ScorpionBerzerkerSideGateLoc3DD0E::Knockback(ScorpionBerzerkerKnockbackStartLoc3DD0E {
                linked_tail_has_kid_collision: true,
                has_level_collision: false,
                has_kid_collision: true,
                is_moved: true,
                is_animated: false,
                sprite_mapping: "LnkTo_unk_C7DD6-Data_Index",
                x_velocity: 0,
                y_velocity: -0x0004_8000,
            }),
            "ASM Scorpion.asm loc_3DD0E sets the linked tail `$3D` flag and seeds the charging-Berzerker knockback state"
        );
    }

    #[test]
    fn scorpion_berzerker_airborne_loop_loc_3dd4a_applies_gravity_and_repeats() {
        assert_eq!(
            scorpion_berzerker_airborne_frame_loc_3dd4a(-0x0004_8000),
            ScorpionBerzerkerAirborneFrameLoc3DD4A {
                hibernates_one_frame: true,
                applies_motion_subroutine: true,
                motion_subroutine: "sub_3E8BA",
                next_y_velocity: -0x0004_4000,
                repeats_airborne_loop: true,
            },
            "ASM Scorpion.asm loc_3DD4A hibernates one frame, runs sub_3E8BA, adds `$4000` to y_vel, and branches back"
        );
    }

    #[test]
    fn scorpion_collision_resume_loc_3dd5c_clears_collision_and_routes_by_state_and_facing() {
        assert_eq!(
            scorpion_collision_resume_loc_3dd5c(false, Direction::Left),
            ScorpionCollisionResumeLoc3DD5C {
                next_collision_type: 0,
                next: ScorpionCollisionResumeBranchLoc3DD5C::PatrolLeft,
            },
            "ASM Scorpion.asm loc_3DD5C clears collision_type, then resumes the left patrol branch when `$5C(a5)` and x_direction are clear"
        );
        assert_eq!(
            scorpion_collision_resume_loc_3dd5c(false, Direction::Right).next,
            ScorpionCollisionResumeBranchLoc3DD5C::PatrolRight,
            "ASM Scorpion.asm loc_3DD5C uses set x_direction for the right patrol branch while `$5C(a5)` is clear"
        );
        assert_eq!(
            scorpion_collision_resume_loc_3dd5c(true, Direction::Left).next,
            ScorpionCollisionResumeBranchLoc3DD5C::PostChargeLeft,
            "ASM Scorpion.asm loc_3DD5C uses clear x_direction for the left post-charge branch while `$5C(a5)` is set"
        );
        assert_eq!(
            scorpion_collision_resume_loc_3dd5c(true, Direction::Right).next,
            ScorpionCollisionResumeBranchLoc3DD5C::PostChargeRight,
            "ASM Scorpion.asm loc_3DD5C uses set x_direction for the right post-charge branch while `$5C(a5)` is set"
        );
    }
}
