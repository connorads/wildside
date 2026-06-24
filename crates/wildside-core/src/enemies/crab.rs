use super::*;

pub(crate) const CRAB_COLID_HURT: i16 = colid::HURT as i16;
pub(crate) const CRAB_COLID_KIDRIGHT: i16 = colid::KIDRIGHT as i16;
pub(crate) const CRAB_COLID_KIDLEFT: i16 = colid::KIDLEFT as i16;

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
pub(crate) enum CrabCollisionRouteLoc3B048 {
    ContinueSharedWalker,
    Death,
    BerzerkerSideGate,
    Damage,
}

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
            // ASM Crab.asm loc_3B086: when `crab_x - kid_x` is negative, Kid
            // is on the crab's right side; only a flipped/right-facing crab
            // falls through to the damage decrement.
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
pub(crate) struct CrabBerzerkerKnockbackStartLoc3B180 {
    pub(crate) has_level_collision: bool,
    pub(crate) is_moved: bool,
    pub(crate) collision_type: u16,
    pub(crate) x_velocity: i32,
    pub(crate) y_velocity: i32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum CrabBerzerkerSideGateLoc3B180 {
    ResumeProximityCallback,
    Knockback(CrabBerzerkerKnockbackStartLoc3B180),
}

pub(crate) fn crab_berzerker_side_gate_loc_3b180(
    berzerker_charging: bool,
    kid_x_velocity: i32,
) -> CrabBerzerkerSideGateLoc3B180 {
    // ASM Crab.asm loc_3B180: clear `Berzerker_charging` branches to
    // loc_3B0BE. A charging Berzerker enables level collision and movement,
    // clears collision_type, doubles Kid's x_vel into the crab, clears y_vel,
    // then falls through to the airborne deceleration loop.
    if !berzerker_charging {
        return CrabBerzerkerSideGateLoc3B180::ResumeProximityCallback;
    }

    CrabBerzerkerSideGateLoc3B180::Knockback(CrabBerzerkerKnockbackStartLoc3B180 {
        has_level_collision: true,
        is_moved: true,
        collision_type: 0,
        x_velocity: kid_x_velocity.wrapping_add(kid_x_velocity),
        y_velocity: 0,
    })
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct CrabAirborneDecelerationFrameLoc3B1A6 {
    pub(crate) collision_type: u16,
    pub(crate) hibernates_one_frame: bool,
    pub(crate) route: CrabAirborneDecelerationRouteLoc3B1A6,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum CrabAirborneDecelerationRouteLoc3B1A6 {
    NonNegativeXVelocity { x_velocity: i32 },
    NegativeXVelocityTail,
}

pub(crate) fn crab_airborne_deceleration_frame_loc_3b1a6(
    x_velocity: i32,
) -> CrabAirborneDecelerationFrameLoc3B1A6 {
    // ASM Crab.asm loc_3B1A6: clear collision_type, hibernate for one frame,
    // then test x_vel. Non-negative velocity is reduced by `$4000`; negative
    // velocity branches to the signed counterpart.
    let route = if x_velocity < 0 {
        CrabAirborneDecelerationRouteLoc3B1A6::NegativeXVelocityTail
    } else {
        CrabAirborneDecelerationRouteLoc3B1A6::NonNegativeXVelocity {
            x_velocity: x_velocity.wrapping_sub(0x0000_4000),
        }
    };

    CrabAirborneDecelerationFrameLoc3B1A6 {
        collision_type: 0,
        hibernates_one_frame: true,
        route,
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct CrabNegativeAirborneDecelerationLoc3B1BE {
    pub(crate) x_velocity: i32,
}

pub(crate) fn crab_negative_airborne_deceleration_loc_3b1be(
    x_velocity: i32,
) -> CrabNegativeAirborneDecelerationLoc3B1BE {
    // ASM Crab.asm loc_3B1BE: negative x_vel receives `addi.l #$4000`
    // before the shared threshold and collision response tail.
    CrabNegativeAirborneDecelerationLoc3B1BE {
        x_velocity: x_velocity.wrapping_add(0x0000_4000),
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum CrabAirborneThresholdRouteLoc3B1C6 {
    ContinueCollisionResponse,
    Settle,
}

pub(crate) fn crab_airborne_threshold_route_loc_3b1c6(
    x_velocity: i32,
) -> CrabAirborneThresholdRouteLoc3B1C6 {
    // ASM Crab.asm loc_3B1C6: signed `bgt` keeps large positive x_vel moving,
    // while signed `bgt` against `$FFFF8000` settles only (-$8000, $8000].
    if x_velocity > 0x0000_8000 || x_velocity <= -0x0000_8000 {
        CrabAirborneThresholdRouteLoc3B1C6::ContinueCollisionResponse
    } else {
        CrabAirborneThresholdRouteLoc3B1C6::Settle
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct CrabAirborneSettleTailLoc3B254 {
    pub(crate) is_moved: bool,
    pub(crate) has_level_collision: bool,
    pub(crate) x_velocity: i32,
    pub(crate) y_velocity: i32,
    pub(crate) resumes_proximity_callback: bool,
}

pub(crate) fn crab_airborne_settle_tail_loc_3b254() -> CrabAirborneSettleTailLoc3B254 {
    // ASM Crab.asm loc_3B254: `sf is_moved(a3)`, `sf has_level_collision(a3)`,
    // zero both velocity longwords, then branch back to loc_3B0BE.
    CrabAirborneSettleTailLoc3B254 {
        is_moved: false,
        has_level_collision: false,
        x_velocity: 0,
        y_velocity: 0,
        resumes_proximity_callback: true,
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) enum CrabAirborneCollisionRouteLoc3B1DA {
    GravityTail,
    ReverseXVelocity,
    SlopeTail { collision_type: u16 },
    Loop,
}

#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn crab_airborne_collision_dispatch_loc_3b1da(
    collision_type: u16,
) -> CrabAirborneCollisionRouteLoc3B1DA {
    // ASM Crab.asm loc_3B1DA: captured collision_type zero enters gravity
    // recovery, wall contacts reverse x_vel, slope contacts preserve d4 for
    // the slope tail, and unknown contacts restart the airborne loop.
    match collision_type {
        0 => CrabAirborneCollisionRouteLoc3B1DA::GravityTail,
        CRAB_COLID_RIGHTWALL | CRAB_COLID_LEFTWALL => {
            CrabAirborneCollisionRouteLoc3B1DA::ReverseXVelocity
        }
        CRAB_COLID_SLOPEUP | CRAB_COLID_SLOPEDOWN => {
            CrabAirborneCollisionRouteLoc3B1DA::SlopeTail { collision_type }
        }
        _ => CrabAirborneCollisionRouteLoc3B1DA::Loop,
    }
}

pub(crate) fn crab_reverse_x_velocity_loc_3b24c(x_velocity: i32) -> i32 {
    // ASM Crab.asm loc_3B24C: `neg.l x_vel(a3)` reverses the complete
    // FP16.16 longword and keeps 68000 two's-complement wrapping behaviour.
    0i32.wrapping_sub(x_velocity)
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) enum CrabSlopeCollisionRouteLoc3B1F8 {
    ReverseXVelocity { x_velocity: i32 },
    ReflectYVelocity { y_velocity: i32 },
    GravityTail,
}

#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn crab_reflect_y_velocity_loc_3b204(y_velocity: i32) -> i32 {
    // ASM Crab.asm loc_3B204: `move.l y_vel,d7; lsr.l #1,d7; neg.l d7;
    // move.l d7,y_vel` uses a logical shift before the two's-complement negate.
    let half_y_unsigned = u32::from_ne_bytes(y_velocity.to_ne_bytes()) >> 1;
    i32::from_ne_bytes(0u32.wrapping_sub(half_y_unsigned).to_ne_bytes())
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) enum CrabNegativeSlopeRouteLoc3B212 {
    ReflectYVelocity,
    GravityTail,
}

#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn crab_negative_slope_route_loc_3b212(
    collision_type: u16,
) -> CrabNegativeSlopeRouteLoc3B212 {
    // ASM Crab.asm loc_3B212: negative x_vel compares d4 with `$18`
    // (slope-down). Exact slope-down falls into the gravity tail; all other
    // slope contacts branch back to loc_3B204 for y-velocity reflection.
    if collision_type == CRAB_COLID_SLOPEDOWN {
        CrabNegativeSlopeRouteLoc3B212::GravityTail
    } else {
        CrabNegativeSlopeRouteLoc3B212::ReflectYVelocity
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct CrabGravityTailLoc3B218 {
    pub(crate) x_velocity: i32,
    pub(crate) y_velocity: i32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct CrabGravityAccelerationTailLoc3B226 {
    pub(crate) y_velocity: i32,
    pub(crate) route: CrabGravityAccelerationRouteLoc3B226,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum CrabGravityAccelerationRouteLoc3B226 {
    NonNegativeXVelocity { x_velocity: i32 },
    NegativeXVelocityTail,
}

pub(crate) fn crab_gravity_acceleration_tail_loc_3b226(
    x_velocity: i32,
    y_velocity: i32,
) -> CrabGravityAccelerationTailLoc3B226 {
    // ASM Crab.asm loc_3B226: add `$4000` to y_vel, then only add `$4000`
    // to x_vel when the signed test keeps x_vel on the non-negative branch.
    let y_velocity = y_velocity.wrapping_add(0x0000_4000);
    let route = if x_velocity < 0 {
        CrabGravityAccelerationRouteLoc3B226::NegativeXVelocityTail
    } else {
        CrabGravityAccelerationRouteLoc3B226::NonNegativeXVelocity {
            x_velocity: x_velocity.wrapping_add(0x0000_4000),
        }
    };

    CrabGravityAccelerationTailLoc3B226 { y_velocity, route }
}

pub(crate) fn crab_negative_gravity_x_tail_loc_3b240(x_velocity: i32) -> i32 {
    // ASM Crab.asm loc_3B240: negative x_vel from the gravity tail receives
    // `subi.l #$4000,x_vel(a3)` before looping back to loc_3B1A6.
    x_velocity.wrapping_sub(0x0000_4000)
}

pub(crate) fn crab_gravity_tail_loc_3b218(
    x_velocity: i32,
    y_velocity: i32,
) -> CrabGravityTailLoc3B218 {
    // ASM Crab.asm loc_3B218: zero y_vel is seeded with `$10000`; the seeded
    // value then falls through to the gravity acceleration tail.
    let seeded_y_velocity = if y_velocity == 0 {
        0x0001_0000
    } else {
        y_velocity
    };
    let acceleration_tail = crab_gravity_acceleration_tail_loc_3b226(x_velocity, seeded_y_velocity);
    let x_velocity = match acceleration_tail.route {
        CrabGravityAccelerationRouteLoc3B226::NonNegativeXVelocity { x_velocity } => x_velocity,
        CrabGravityAccelerationRouteLoc3B226::NegativeXVelocityTail => {
            crab_negative_gravity_x_tail_loc_3b240(x_velocity)
        }
    };

    CrabGravityTailLoc3B218 {
        x_velocity,
        y_velocity: acceleration_tail.y_velocity,
    }
}

#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn crab_slope_collision_response_loc_3b1f8(
    collision_type: u16,
    x_velocity: i32,
    y_velocity: i32,
) -> CrabSlopeCollisionRouteLoc3B1F8 {
    // ASM Crab.asm loc_3B1F8: non-negative x_vel on slope-up reverses x_vel;
    // negative x_vel on slope-down enters gravity recovery. The other
    // slope/direction pairs logically halve y_vel as a long, then negate it.
    if x_velocity >= 0 && collision_type == CRAB_COLID_SLOPEUP {
        return CrabSlopeCollisionRouteLoc3B1F8::ReverseXVelocity {
            x_velocity: crab_reverse_x_velocity_loc_3b24c(x_velocity),
        };
    }

    if x_velocity < 0 {
        match crab_negative_slope_route_loc_3b212(collision_type) {
            CrabNegativeSlopeRouteLoc3B212::GravityTail => {
                return CrabSlopeCollisionRouteLoc3B1F8::GravityTail;
            }
            CrabNegativeSlopeRouteLoc3B212::ReflectYVelocity => {}
        }
    }

    CrabSlopeCollisionRouteLoc3B1F8::ReflectYVelocity {
        y_velocity: crab_reflect_y_velocity_loc_3b204(y_velocity),
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum CrabDamageRouteLoc3B08C {
    HurtAnimation,
    DeathAnimation,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct CrabDamageResultLoc3B08C {
    pub(crate) current_hp: u16,
    pub(crate) route: CrabDamageRouteLoc3B08C,
}

pub(crate) fn crab_damage_after_hurt_loc_3b08c(current_hp: u16) -> CrabDamageResultLoc3B08C {
    // ASM Crab.asm loc_3B08C: decrement the `current_hp(a3)` word before
    // deciding whether the crab survives for the hurt animation or exits.
    let next_current_hp = current_hp.wrapping_sub(1);
    CrabDamageResultLoc3B08C {
        current_hp: next_current_hp,
        route: if next_current_hp == 0 {
            CrabDamageRouteLoc3B08C::DeathAnimation
        } else {
            CrabDamageRouteLoc3B08C::HurtAnimation
        },
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) struct CrabHurtSequenceLoc3B0A6 {
    pub(crate) hurt_animation_script: &'static str,
    pub(crate) waits_until_animation_finished: bool,
    pub(crate) resumed_animation_script: &'static str,
    pub(crate) falls_through_to_proximity_callback: bool,
}

#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn crab_hurt_sequence_loc_3b0a6() -> CrabHurtSequenceLoc3B0A6 {
    // ASM Crab.asm loc_3B0A6: initialise `stru_3AF7C`, hibernate until that
    // hurt animation finishes, then restore `stru_3AF62` before loc_3B0BE.
    CrabHurtSequenceLoc3B0A6 {
        hurt_animation_script: "stru_3AF7C",
        waits_until_animation_finished: true,
        resumed_animation_script: "stru_3AF62",
        falls_through_to_proximity_callback: true,
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) struct CrabDeathSequenceLoc3B094 {
    pub(crate) animation_script: &'static str,
    pub(crate) waits_until_animation_finished: bool,
    pub(crate) exits_enemy: bool,
}

#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn crab_death_sequence_loc_3b094() -> CrabDeathSequenceLoc3B094 {
    // ASM Crab.asm loc_3B094: initialise `stru_3AF82`, hibernate until that
    // death animation finishes, then branch to the shared enemy exit.
    CrabDeathSequenceLoc3B094 {
        animation_script: "stru_3AF82",
        waits_until_animation_finished: true,
        exits_enemy: true,
    }
}

/// Crab freeze distance constants from ASM.
/// ASM loc_3B0E4: cmpi.w #$40,d7 (freeze threshold = 64px).
const CRAB_FREEZE_DISTANCE: i64 = 0x40;
/// ASM loc_3B142: cmpi.w #$E0,d7 (unfreeze threshold = 224px).
const CRAB_UNFREEZE_DISTANCE: i64 = 0xE0;

pub(crate) fn crab_facing_toward_kid_loc_3b0e0(crab_x_minus_kid_x: i16) -> Direction {
    // ASM Crab.asm loc_3B0E0: a negative `d7 = crab_x - Kid_x` reaches
    // `st x_direction(a3)`, which is the crab's flipped/right-facing state.
    if crab_x_minus_kid_x < 0 {
        Direction::Right
    } else {
        Direction::Left
    }
}

pub(crate) fn crab_freeze_direction_gate_loc_3b0fe(
    kid_facing: Direction,
    crab_facing: Direction,
) -> bool {
    // ASM Crab.asm loc_3B0FE: when Kid's x_direction is clear, only a clear
    // crab x_direction freezes. Kid clear means right-facing; crab clear means
    // left-facing, so native directions must oppose each other.
    let kid_x_direction_set = matches!(kid_facing, Direction::Left);
    let crab_x_direction_set = matches!(crab_facing, Direction::Right);
    kid_x_direction_set == crab_x_direction_set
}

/// Crab player-proximity freeze callback (Crab.asm loc_3B0BE-loc_3B174).
///
/// When player is within 64px horizontally AND facing the crab:
/// - Freeze the crab (stop animation, set freeze sprite, zero velocity)
///
/// Unfreeze when:
/// - Distance > 224px, OR
/// - Within 64px while the player is moving toward the crab
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

    let was_frozen = enemy.crab_frozen;
    if !was_frozen {
        enemy.facing = crab_facing_toward_kid_loc_3b0e0(dx as i16);
    }

    // ASM loc_3B142-loc_3B170: unfreeze when distance > 224px
    if abs_dx > CRAB_UNFREEZE_DISTANCE {
        enemy.crab_frozen = false;
        return;
    }

    if was_frozen {
        if abs_dx <= CRAB_FREEZE_DISTANCE {
            // ASM loc_3B142-loc_3B16C: while frozen, close-range movement toward
            // the crab clears `$6E(a5)` and resumes the walking sprite.
            if let Some(p_vel) = player_x_velocity {
                let player_moving_toward_crab = if dx >= 0 { p_vel > 0 } else { p_vel < 0 };
                if player_moving_toward_crab {
                    enemy.crab_frozen = false;
                }
            }
        }
        return;
    }

    // ASM loc_3B0E4-loc_3B11E: check freeze condition (within 64px)
    if abs_dx <= CRAB_FREEZE_DISTANCE {
        // ASM loc_3B0E4: player within 64px
        let player_faces_crab = crab_freeze_direction_gate_loc_3b0fe(pfacing, enemy.facing);

        if player_faces_crab {
            // ASM loc_3B104: freeze the crab
            // sf is_animated(a3) - stop animation
            // bset #7,object_meta(a3) - set freeze bit
            // move.l #0,$50(a5) - zero x_velocity
            enemy.crab_frozen = true;
            enemy.x_velocity = 0;
        }
    }
    // ASM loc_3B11E: outside 64px but within 224px, keep current state
    // (already frozen stays frozen, already unfrozen stays unfrozen)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::enemies::test_fixtures::*;

    fn crab_test_enemy(x_px: u32, crab_frozen: bool, facing: Direction) -> ActiveEnemy {
        ActiveEnemy {
            family: EnemyFamily::Crab,
            x_pos: x_px << 16,
            x_velocity: 0x0001_8000,
            crab_frozen,
            facing,
            ..mini_hopping_skull_test_enemy()
        }
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
        assert_eq!(
            pacer_walk_speed(EnemyFamily::Crab, 5),
            0x0001_8000,
            "ASM Crab.asm loc_3AFD0/loc_3B00A keep HP above 3 on the same `$18000` tier"
        );
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
    fn crab_berzerker_side_gate_matches_loc_3b180() {
        assert_eq!(
            crab_berzerker_side_gate_loc_3b180(false, 0x0001_8000),
            CrabBerzerkerSideGateLoc3B180::ResumeProximityCallback,
            "ASM Crab.asm loc_3B180 should branch to loc_3B0BE when `Berzerker_charging` is clear"
        );

        assert_eq!(
            crab_berzerker_side_gate_loc_3b180(true, -0x0001_8000),
            CrabBerzerkerSideGateLoc3B180::Knockback(CrabBerzerkerKnockbackStartLoc3B180 {
                has_level_collision: true,
                is_moved: true,
                collision_type: 0,
                x_velocity: -0x0003_0000,
                y_velocity: 0,
            }),
            "ASM Crab.asm loc_3B180 should double Kid x_vel into the crab and clear y_vel before the airborne loop"
        );
    }

    #[test]
    fn crab_berzerker_knockback_runtime_skips_pacer_loc_3b1a6() {
        let mut enemy = mini_hopping_skull_test_enemy();
        enemy.family = EnemyFamily::Crab;
        enemy.facing = Direction::Left;
        enemy.x_velocity = 0x0003_0000;
        enemy.y_velocity = 0;
        enemy.on_ground = false;
        enemy.crab_berzerker_knockback = true;
        let mut enemies = [enemy];
        let grid = mini_hopping_skull_test_grid();
        let mut drilled_blocks = Vec::new();

        step_active_enemies(
            &mut enemies,
            Some(&grid),
            None,
            None,
            None,
            Some(0),
            PlayerForm::Berzerker,
            None,
            &mut drilled_blocks,
        );

        assert!(
            enemies[0].crab_berzerker_knockback,
            "ASM Crab.asm loc_3B1A6 remains in the airborne loop while x_vel is outside the settle window"
        );
        assert_ne!(
            enemies[0].x_velocity,
            horizontal_velocity(Direction::Left, pacer_walk_speed(EnemyFamily::Crab, 1)),
            "Berzerker knockback must not fall through to the normal pacer walker"
        );
        assert!(
            enemies[0].y_velocity > 0,
            "ASM Crab.asm loc_3B218 seeds downward recovery velocity from zero y_vel"
        );
    }

    #[test]
    fn crab_airborne_deceleration_frame_matches_loc_3b1a6() {
        assert_eq!(
            crab_airborne_deceleration_frame_loc_3b1a6(0x0001_0000),
            CrabAirborneDecelerationFrameLoc3B1A6 {
                collision_type: 0,
                hibernates_one_frame: true,
                route: CrabAirborneDecelerationRouteLoc3B1A6::NonNegativeXVelocity {
                    x_velocity: 0x0000_C000,
                },
            },
            "ASM Crab.asm loc_3B1A6 should clear collision_type, hibernate, then subtract `$4000` for non-negative x_vel"
        );

        assert_eq!(
            crab_airborne_deceleration_frame_loc_3b1a6(0),
            CrabAirborneDecelerationFrameLoc3B1A6 {
                collision_type: 0,
                hibernates_one_frame: true,
                route: CrabAirborneDecelerationRouteLoc3B1A6::NonNegativeXVelocity {
                    x_velocity: -0x0000_4000,
                },
            },
            "ASM Crab.asm loc_3B1A6 should treat zero x_vel as non-negative before subtracting `$4000`"
        );

        assert_eq!(
            crab_airborne_deceleration_frame_loc_3b1a6(-0x0001_0000),
            CrabAirborneDecelerationFrameLoc3B1A6 {
                collision_type: 0,
                hibernates_one_frame: true,
                route: CrabAirborneDecelerationRouteLoc3B1A6::NegativeXVelocityTail,
            },
            "ASM Crab.asm loc_3B1A6 should branch away when x_vel is negative"
        );
    }

    #[test]
    fn crab_negative_airborne_deceleration_matches_loc_3b1be() {
        assert_eq!(
            crab_negative_airborne_deceleration_loc_3b1be(-0x0001_0000),
            CrabNegativeAirborneDecelerationLoc3B1BE {
                x_velocity: -0x0000_C000,
            },
            "ASM Crab.asm loc_3B1BE should add `$4000` to a negative x_vel"
        );

        assert_eq!(
            crab_negative_airborne_deceleration_loc_3b1be(-0x0000_3000),
            CrabNegativeAirborneDecelerationLoc3B1BE {
                x_velocity: 0x0000_1000,
            },
            "ASM Crab.asm loc_3B1BE can cross zero because it unconditionally adds `$4000`"
        );
    }

    #[test]
    fn crab_airborne_threshold_route_matches_loc_3b1c6() {
        assert_eq!(
            crab_airborne_threshold_route_loc_3b1c6(0x0000_8001),
            CrabAirborneThresholdRouteLoc3B1C6::ContinueCollisionResponse,
            "ASM Crab.asm loc_3B1C6 should continue when signed x_vel is greater than `$8000`"
        );

        assert_eq!(
            crab_airborne_threshold_route_loc_3b1c6(0x0000_8000),
            CrabAirborneThresholdRouteLoc3B1C6::Settle,
            "ASM Crab.asm loc_3B1C6 should settle at the positive threshold"
        );

        assert_eq!(
            crab_airborne_threshold_route_loc_3b1c6(-0x0000_7FFF),
            CrabAirborneThresholdRouteLoc3B1C6::Settle,
            "ASM Crab.asm loc_3B1C6 should settle while x_vel is still greater than `-$8000`"
        );

        assert_eq!(
            crab_airborne_threshold_route_loc_3b1c6(-0x0000_8000),
            CrabAirborneThresholdRouteLoc3B1C6::ContinueCollisionResponse,
            "ASM Crab.asm loc_3B1C6 should continue at exact `-$8000` because the second signed `bgt` is false"
        );
    }

    #[test]
    fn crab_airborne_settle_tail_matches_loc_3b254() {
        assert_eq!(
            crab_airborne_settle_tail_loc_3b254(),
            CrabAirborneSettleTailLoc3B254 {
                is_moved: false,
                has_level_collision: false,
                x_velocity: 0,
                y_velocity: 0,
                resumes_proximity_callback: true,
            },
            "ASM Crab.asm loc_3B254 should clear movement/collision flags, zero both velocities, and resume loc_3B0BE"
        );
    }

    #[test]
    fn crab_airborne_collision_dispatch_matches_loc_3b1da() {
        assert_eq!(
            crab_airborne_collision_dispatch_loc_3b1da(0),
            CrabAirborneCollisionRouteLoc3B1DA::GravityTail,
            "ASM Crab.asm loc_3B1DA should send zero collision_type to gravity recovery"
        );

        assert_eq!(
            crab_airborne_collision_dispatch_loc_3b1da(CRAB_COLID_RIGHTWALL),
            CrabAirborneCollisionRouteLoc3B1DA::ReverseXVelocity,
            "ASM Crab.asm loc_3B1DA should reverse x_vel on right wall contact"
        );

        assert_eq!(
            crab_airborne_collision_dispatch_loc_3b1da(CRAB_COLID_LEFTWALL),
            CrabAirborneCollisionRouteLoc3B1DA::ReverseXVelocity,
            "ASM Crab.asm loc_3B1DA should reverse x_vel on left wall contact"
        );

        assert_eq!(
            crab_airborne_collision_dispatch_loc_3b1da(CRAB_COLID_SLOPEUP),
            CrabAirborneCollisionRouteLoc3B1DA::SlopeTail {
                collision_type: CRAB_COLID_SLOPEUP,
            },
            "ASM Crab.asm loc_3B1DA should keep slope-up collision_type for the slope tail"
        );

        assert_eq!(
            crab_airborne_collision_dispatch_loc_3b1da(CRAB_COLID_SLOPEDOWN),
            CrabAirborneCollisionRouteLoc3B1DA::SlopeTail {
                collision_type: CRAB_COLID_SLOPEDOWN,
            },
            "ASM Crab.asm loc_3B1DA should keep slope-down collision_type for the slope tail"
        );

        assert_eq!(
            crab_airborne_collision_dispatch_loc_3b1da(0x002C),
            CrabAirborneCollisionRouteLoc3B1DA::Loop,
            "ASM Crab.asm loc_3B1DA should loop for collisions outside zero, wall, and slope cases"
        );
    }

    #[test]
    fn crab_reverse_x_velocity_matches_loc_3b24c() {
        assert_eq!(
            crab_reverse_x_velocity_loc_3b24c(0x0000_8000),
            -0x0000_8000,
            "ASM Crab.asm loc_3B24C should negate positive x_vel"
        );

        assert_eq!(
            crab_reverse_x_velocity_loc_3b24c(i32::MIN),
            i32::MIN,
            "ASM Crab.asm loc_3B24C should preserve 68000 wrapping on the most negative long"
        );
    }

    #[test]
    fn crab_slope_collision_response_matches_loc_3b1f8() {
        assert_eq!(
            crab_slope_collision_response_loc_3b1f8(CRAB_COLID_SLOPEUP, 0x0000_8000, 0x0002_0000,),
            CrabSlopeCollisionRouteLoc3B1F8::ReverseXVelocity {
                x_velocity: -0x0000_8000,
            },
            "ASM Crab.asm loc_3B1F8 should reverse x_vel for non-negative x_vel on slope-up"
        );

        assert_eq!(
            crab_slope_collision_response_loc_3b1f8(
                CRAB_COLID_SLOPEDOWN,
                -0x0000_8000,
                0x0002_0000,
            ),
            CrabSlopeCollisionRouteLoc3B1F8::GravityTail,
            "ASM Crab.asm loc_3B1F8 should send negative x_vel on slope-down to gravity recovery"
        );

        assert_eq!(
            crab_slope_collision_response_loc_3b1f8(CRAB_COLID_SLOPEDOWN, 0x0000_8000, 0x0002_0000,),
            CrabSlopeCollisionRouteLoc3B1F8::ReflectYVelocity {
                y_velocity: -0x0001_0000,
            },
            "ASM Crab.asm loc_3B1F8 should reflect y_vel for non-negative x_vel on slope-down"
        );
        assert_eq!(
            crab_slope_collision_response_loc_3b1f8(CRAB_COLID_SLOPEDOWN, 0, 0x0002_0000,),
            CrabSlopeCollisionRouteLoc3B1F8::ReflectYVelocity {
                y_velocity: -0x0001_0000,
            },
            "ASM Crab.asm loc_3B1F8 should treat zero x_vel as the non-negative slope-down reflection path"
        );

        assert_eq!(
            crab_slope_collision_response_loc_3b1f8(CRAB_COLID_SLOPEUP, -0x0000_8000, -0x0002_0000,),
            CrabSlopeCollisionRouteLoc3B1F8::ReflectYVelocity {
                y_velocity: -0x7FFF_0000,
            },
            "ASM Crab.asm loc_3B1F8 should use logical half-y reflection for negative y_vel"
        );
    }

    #[test]
    fn crab_reflect_y_velocity_matches_loc_3b204() {
        assert_eq!(
            crab_reflect_y_velocity_loc_3b204(0x0002_0000),
            -0x0001_0000,
            "ASM Crab.asm loc_3B204 should logically halve positive y_vel, then negate it"
        );

        assert_eq!(
            crab_reflect_y_velocity_loc_3b204(-0x0002_0000),
            -0x7FFF_0000,
            "ASM Crab.asm loc_3B204 uses `lsr.l`, so negative y_vel is halved as unsigned before negation"
        );
    }

    #[test]
    fn crab_negative_slope_route_matches_loc_3b212() {
        assert_eq!(
            crab_negative_slope_route_loc_3b212(CRAB_COLID_SLOPEDOWN),
            CrabNegativeSlopeRouteLoc3B212::GravityTail,
            "ASM Crab.asm loc_3B212 should fall through to the gravity tail only for collision `$18`"
        );

        assert_eq!(
            crab_negative_slope_route_loc_3b212(CRAB_COLID_SLOPEUP),
            CrabNegativeSlopeRouteLoc3B212::ReflectYVelocity,
            "ASM Crab.asm loc_3B212 should branch to loc_3B204 for non-`$18` slope contact"
        );
    }

    #[test]
    fn crab_gravity_acceleration_tail_matches_loc_3b226() {
        assert_eq!(
            crab_gravity_acceleration_tail_loc_3b226(0x0000_8000, 0x0001_0000),
            CrabGravityAccelerationTailLoc3B226 {
                y_velocity: 0x0001_4000,
                route: CrabGravityAccelerationRouteLoc3B226::NonNegativeXVelocity {
                    x_velocity: 0x0000_C000,
                },
            },
            "ASM Crab.asm loc_3B226 should add `$4000` to y_vel and non-negative x_vel"
        );

        assert_eq!(
            crab_gravity_acceleration_tail_loc_3b226(-0x0000_8000, 0x0002_0000),
            CrabGravityAccelerationTailLoc3B226 {
                y_velocity: 0x0002_4000,
                route: CrabGravityAccelerationRouteLoc3B226::NegativeXVelocityTail,
            },
            "ASM Crab.asm loc_3B226 should still add gravity before branching on negative x_vel"
        );
        assert_eq!(
            crab_gravity_acceleration_tail_loc_3b226(0, 0x0002_0000),
            CrabGravityAccelerationTailLoc3B226 {
                y_velocity: 0x0002_4000,
                route: CrabGravityAccelerationRouteLoc3B226::NonNegativeXVelocity {
                    x_velocity: 0x0000_4000,
                },
            },
            "ASM Crab.asm loc_3B226 should treat zero x_vel as the non-negative acceleration branch"
        );
    }

    #[test]
    fn crab_negative_gravity_x_tail_matches_loc_3b240() {
        assert_eq!(
            crab_negative_gravity_x_tail_loc_3b240(-0x0000_8000),
            -0x0000_C000,
            "ASM Crab.asm loc_3B240 should subtract `$4000` from negative x_vel"
        );

        assert_eq!(
            crab_negative_gravity_x_tail_loc_3b240(i32::MIN),
            0x7FFF_C000,
            "ASM Crab.asm loc_3B240 should use 32-bit wrapping subtraction"
        );
    }

    #[test]
    fn crab_gravity_tail_matches_loc_3b218() {
        assert_eq!(
            crab_gravity_tail_loc_3b218(0x0000_8000, 0),
            CrabGravityTailLoc3B218 {
                x_velocity: 0x0000_C000,
                y_velocity: 0x0001_4000,
            },
            "ASM Crab.asm loc_3B218 should seed zero y_vel with `$10000`, then add `$4000`"
        );

        assert_eq!(
            crab_gravity_tail_loc_3b218(-0x0000_8000, 0x0002_0000),
            CrabGravityTailLoc3B218 {
                x_velocity: -0x0000_C000,
                y_velocity: 0x0002_4000,
            },
            "ASM Crab.asm loc_3B218 should preserve non-zero y_vel before adding `$4000` and push negative x_vel lower"
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
    fn crab_hurt_right_side_gate_matches_loc_3b086() {
        assert_eq!(
            crab_collision_dispatch_loc_3b048(CRAB_COLID_HURT, false, -0x0020, Direction::Left),
            CrabCollisionRouteLoc3B048::ContinueSharedWalker,
            "ASM Crab.asm loc_3B086 should branch back when Kid is right of an unflipped/left-facing crab"
        );
        assert_eq!(
            crab_collision_dispatch_loc_3b048(CRAB_COLID_HURT, false, -0x0020, Direction::Right),
            CrabCollisionRouteLoc3B048::Damage,
            "ASM Crab.asm loc_3B086 should accept a flipped/right-facing crab"
        );
    }

    #[test]
    fn crab_damage_decrements_hp_before_routing_loc_3b08c() {
        assert_eq!(
            crab_damage_after_hurt_loc_3b08c(2),
            CrabDamageResultLoc3B08C {
                current_hp: 1,
                route: CrabDamageRouteLoc3B08C::HurtAnimation,
            },
            "ASM Crab.asm loc_3B08C should keep the crab alive when `subi.w #1,current_hp` leaves a non-zero word"
        );
        assert_eq!(
            crab_damage_after_hurt_loc_3b08c(1),
            CrabDamageResultLoc3B08C {
                current_hp: 0,
                route: CrabDamageRouteLoc3B08C::DeathAnimation,
            },
            "ASM Crab.asm loc_3B08C should route out when `subi.w #1,current_hp` leaves zero"
        );
    }

    #[test]
    fn crab_hurt_sequence_matches_loc_3b0a6() {
        assert_eq!(
            crab_hurt_sequence_loc_3b0a6(),
            CrabHurtSequenceLoc3B0A6 {
                hurt_animation_script: "stru_3AF7C",
                waits_until_animation_finished: true,
                resumed_animation_script: "stru_3AF62",
                falls_through_to_proximity_callback: true,
            },
            "ASM Crab.asm loc_3B0A6 should restore the walking animation before falling through to loc_3B0BE"
        );
    }

    #[test]
    fn crab_death_sequence_matches_loc_3b094() {
        assert_eq!(
            crab_death_sequence_loc_3b094(),
            CrabDeathSequenceLoc3B094 {
                animation_script: "stru_3AF82",
                waits_until_animation_finished: true,
                exits_enemy: true,
            },
            "ASM Crab.asm loc_3B094 should play the death animation to completion before enemy exit"
        );
    }

    #[test]
    fn crab_faces_toward_kid_matches_loc_3b0e0() {
        assert_eq!(
            crab_facing_toward_kid_loc_3b0e0(-1),
            Direction::Right,
            "ASM Crab.asm loc_3B0E0 should set x_direction when Kid is right of the crab"
        );
        assert_eq!(
            crab_facing_toward_kid_loc_3b0e0(0),
            Direction::Left,
            "ASM Crab.asm loc_3B0BE should leave exact X alignment on the clear-x_direction branch"
        );
        assert_eq!(
            crab_facing_toward_kid_loc_3b0e0(1),
            Direction::Left,
            "ASM Crab.asm loc_3B0BE should clear x_direction when Kid is left of the crab"
        );
    }

    #[test]
    fn crab_freeze_direction_gate_matches_loc_3b0fe() {
        assert!(
            crab_freeze_direction_gate_loc_3b0fe(Direction::Right, Direction::Left),
            "ASM Crab.asm loc_3B0FE should freeze when Kid's clear x_direction faces a clear-x_direction crab"
        );
        assert!(
            !crab_freeze_direction_gate_loc_3b0fe(Direction::Right, Direction::Right),
            "ASM Crab.asm loc_3B0FE should skip freeze when the crab x_direction is set"
        );
        assert!(
            crab_freeze_direction_gate_loc_3b0fe(Direction::Left, Direction::Right),
            "ASM Crab.asm loc_3B0E4 fallthrough should freeze when both raw x_direction flags are set"
        );
        assert!(
            !crab_freeze_direction_gate_loc_3b0fe(Direction::Left, Direction::Left),
            "ASM Crab.asm loc_3B0E4 should skip freeze when Kid's set x_direction sees a clear-x_direction crab"
        );
    }

    #[test]
    fn crab_freeze_callback_loc_3b0be_freezes_when_kid_faces_crab_inside_64px() {
        let mut enemy = crab_test_enemy(100, false, Direction::Left);

        step_crab_freeze(&mut enemy, Some(140 << 16), Some(Direction::Left), Some(0));

        assert!(
            enemy.crab_frozen,
            "ASM Crab.asm loc_3B104 should set `$6E(a5)` when Kid is inside 64px and faces the crab"
        );
        assert_eq!(
            enemy.facing,
            Direction::Right,
            "ASM Crab.asm loc_3B0E0 should face the crab toward Kid before freezing"
        );
        assert_eq!(
            enemy.x_velocity, 0,
            "ASM Crab.asm loc_3B104 should zero `$50(a5)` while frozen"
        );
    }

    #[test]
    fn crab_frozen_callback_loc_3b142_keeps_facing_and_holds_at_224px_edge() {
        let mut enemy = crab_test_enemy(300, true, Direction::Left);

        step_crab_freeze(&mut enemy, Some(76 << 16), Some(Direction::Right), None);

        assert!(
            enemy.crab_frozen,
            "ASM Crab.asm loc_3B142 should hold the frozen flag at exact 224px distance"
        );
        assert_eq!(
            enemy.facing,
            Direction::Left,
            "ASM Crab.asm loc_3B142 skips the loc_3B0E0 facing update while `$6E(a5)` is set"
        );
    }

    #[test]
    fn crab_frozen_callback_loc_3b142_unfreezes_beyond_224px() {
        let mut enemy = crab_test_enemy(300, true, Direction::Left);

        step_crab_freeze(&mut enemy, Some(75 << 16), Some(Direction::Right), None);

        assert!(
            !enemy.crab_frozen,
            "ASM Crab.asm loc_3B142 should clear `$6E(a5)` once distance exceeds 224px"
        );
    }

    #[test]
    fn crab_frozen_callback_loc_3b142_unfreezes_when_kid_moves_toward_crab() {
        let mut kid_left = crab_test_enemy(100, true, Direction::Left);
        step_crab_freeze(
            &mut kid_left,
            Some(60 << 16),
            Some(Direction::Right),
            Some(0x0001_0000),
        );
        assert!(
            !kid_left.crab_frozen,
            "ASM Crab.asm loc_3B142 should clear `$6E(a5)` when non-negative d7 sees positive Kid x_vel"
        );

        let mut kid_right = crab_test_enemy(100, true, Direction::Right);
        step_crab_freeze(
            &mut kid_right,
            Some(140 << 16),
            Some(Direction::Left),
            Some(-0x0001_0000),
        );
        assert!(
            !kid_right.crab_frozen,
            "ASM Crab.asm loc_3B16C should clear `$6E(a5)` when negative d7 sees negative Kid x_vel"
        );
    }

    #[test]
    fn crab_frozen_callback_loc_3b142_holds_when_kid_stops_or_moves_away() {
        for (player_x, player_x_velocity) in [
            (100 << 16, 0),
            (60 << 16, 0),
            (60 << 16, -0x0001_0000),
            (140 << 16, 0),
            (140 << 16, 0x0001_0000),
        ] {
            let mut enemy = crab_test_enemy(100, true, Direction::Left);

            step_crab_freeze(
                &mut enemy,
                Some(player_x),
                Some(Direction::Right),
                Some(player_x_velocity),
            );

            assert!(
                enemy.crab_frozen,
                "ASM Crab.asm loc_3B142/loc_3B16C should hold `$6E(a5)` unless close-range Kid x_vel points toward the crab; player_x={player_x:#010X}, player_x_velocity={player_x_velocity:#010X}"
            );
        }
    }

    #[test]
    fn crab_hurt_animation_hibernates_instead_of_pacing_loc_3b0a6() {
        let mut enemy = mini_hopping_skull_test_enemy();
        enemy.family = EnemyFamily::Crab;
        enemy.x_pos = 100 << 16;
        enemy.y_pos = 96 << 16;
        enemy.hit_points = 2;
        enemy.facing = Direction::Right;
        enemy.damage_flash_frames = 3;

        let w = 20u16;
        let h = 15u16;
        let data = vec![0u8; (w * h * 2) as usize];
        let grid = terrain::BlockGrid::from_level_layout(&data, w, h);
        let start_x = enemy.x_pos;

        step_active_enemies(
            std::slice::from_mut(&mut enemy),
            Some(&grid),
            None,
            None,
            None,
            None,
            PlayerForm::TheKid,
            None,
            &mut Vec::new(),
        );

        assert_eq!(
            enemy.x_pos, start_x,
            "ASM Crab.asm loc_3B0A6 hibernates until the hurt animation finishes, so the crab should not pace during damage flash"
        );
        assert_eq!(
            enemy.damage_flash_frames, 2,
            "the native damage-flash window still advances while the hurt animation hibernates"
        );
    }
}
