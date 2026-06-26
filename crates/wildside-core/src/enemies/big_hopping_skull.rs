use super::*;

pub(crate) const BIG_SKULL_GRAVITY: i32 = 0x0000_0FA0; // ASM: addi.l #$FA0,y_vel
/// ASM Big_Hopping_Skull.asm loc_36580 initialises `stru_367AE` (`$50` ticks),
/// then loc_366B6 plays `stru_367B4` (`$0C + 1` ticks) before loc_366D0 relaunches.
pub(crate) const BIG_HOPPING_SKULL_FLOOR_CONTACT_WAIT_FRAMES_LOC_36580: u16 = 0x50 + 0x0C + 1;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct BigHoppingSkullStartupLoc3652E {
    pub(crate) hibernates_one_frame: bool,
    pub(crate) checks_object_range: bool,
    pub(crate) animation: &'static str,
}

pub(crate) fn big_hopping_skull_startup_loc_3652e() -> BigHoppingSkullStartupLoc3652E {
    // ASM Big_Hopping_Skull.asm loc_3652E: hibernate for one frame, call
    // Object_CheckInRange, initialise `stru_367BE`, then enter loc_36540.
    BigHoppingSkullStartupLoc3652E {
        hibernates_one_frame: true,
        checks_object_range: true,
        animation: "stru_367BE",
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum BigHoppingSkullLevelHeightRouteLoc3656C {
    ReturnLoc3657C,
    DeleteCurrentObject,
}

pub(crate) fn big_hopping_skull_level_height_gate_loc_3656c(
    y_pos_word: i16,
    level_height_pixels: i16,
) -> BigHoppingSkullLevelHeightRouteLoc3656C {
    // ASM Big_Hopping_Skull.asm loc_3656C: compare signed `y_pos(a3)` word
    // with `Level_height_pixels`; only strictly less returns at loc_3657C.
    if y_pos_word < level_height_pixels {
        BigHoppingSkullLevelHeightRouteLoc3656C::ReturnLoc3657C
    } else {
        BigHoppingSkullLevelHeightRouteLoc3656C::DeleteCurrentObject
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum BigHoppingSkullContactWaitRouteLoc365AC {
    Relaunch,
    SharedTail,
    DamageSkull,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct BigHoppingSkullContactWaitLoc365AC {
    pub(crate) hibernates_one_frame: bool,
    pub(crate) checks_object_range: bool,
    pub(crate) collision_probe_extent: u16,
    pub(crate) route: BigHoppingSkullContactWaitRouteLoc365AC,
}

pub(crate) fn big_hopping_skull_contact_wait_loc_365ac(
    collision_probe_nonzero: bool,
    collision_type: u16,
) -> BigHoppingSkullContactWaitLoc365AC {
    const COLID_HURT: u16 = 0x001C;
    const COLID_KIDABOVE: u16 = 0x002C;

    // ASM Big_Hopping_Skull.asm loc_365AC: hibernate, range-check, run the
    // `loc_33286` collision probe with d7=#8, then dispatch only if d6 is set.
    let route = if !collision_probe_nonzero {
        BigHoppingSkullContactWaitRouteLoc365AC::Relaunch
    } else if matches!(collision_type, COLID_HURT | COLID_KIDABOVE) {
        BigHoppingSkullContactWaitRouteLoc365AC::DamageSkull
    } else {
        BigHoppingSkullContactWaitRouteLoc365AC::SharedTail
    };

    BigHoppingSkullContactWaitLoc365AC {
        hibernates_one_frame: true,
        checks_object_range: true,
        collision_probe_extent: 8,
        route,
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum BigHoppingSkullDamageEntryRouteLoc365D8 {
    DestroySkull,
    HitAnimationThenWait,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct BigHoppingSkullDamageEntryLoc365D8 {
    pub(crate) has_kid_collision: bool,
    pub(crate) kid_y_velocity: i32,
    pub(crate) remaining_hit_points: u16,
    pub(crate) animation: Option<&'static str>,
    pub(crate) route: BigHoppingSkullDamageEntryRouteLoc365D8,
}

pub(crate) fn big_hopping_skull_damage_entry_loc_365d8(
    hit_points_word: u16,
    kid_y_velocity: i32,
) -> BigHoppingSkullDamageEntryLoc365D8 {
    let kid_y_velocity = write_velocity_high_word(kid_y_velocity, -3);
    let remaining_hit_points = hit_points_word.wrapping_sub(1);
    let skull_destroyed = remaining_hit_points == 0;

    // ASM Big_Hopping_Skull.asm loc_365D8: set `has_kid_collision(a3)`,
    // ORIGINAL_QUIRK: `move.w #$FFFD,y_vel(a2)` writes only Kid's high
    // velocity word, preserving the fractional low word while bouncing Kid up.
    BigHoppingSkullDamageEntryLoc365D8 {
        has_kid_collision: true,
        kid_y_velocity,
        remaining_hit_points,
        animation: (!skull_destroyed).then_some("stru_367CC"),
        route: if skull_destroyed {
            BigHoppingSkullDamageEntryRouteLoc365D8::DestroySkull
        } else {
            BigHoppingSkullDamageEntryRouteLoc365D8::HitAnimationThenWait
        },
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct BigHoppingSkullHitGroanGateLoc365F2 {
    pub(crate) hibernates_one_frame: bool,
    pub(crate) checks_object_range: bool,
    pub(crate) sound_effect: Option<&'static str>,
    pub(crate) d0_preserved: bool,
}

pub(crate) fn big_hopping_skull_hit_groan_gate_loc_365f2(
    sprite_is_groan_frame: bool,
    animation_timer: u16,
) -> BigHoppingSkullHitGroanGateLoc365F2 {
    // ASM Big_Hopping_Skull.asm loc_365F2: after the one-frame hibernate
    // and Object_CheckInRange, play the groan only for `LnkTo_unk_C7FB0`
    // when `animation_timer(a3)` is exactly 6; d0 is pushed around PlaySound.
    BigHoppingSkullHitGroanGateLoc365F2 {
        hibernates_one_frame: true,
        checks_object_range: true,
        sound_effect: (sprite_is_groan_frame && animation_timer == 6)
            .then_some("sfx_Big_Hopping_Skull_groan"),
        d0_preserved: true,
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct BigHoppingSkullHitVoiceGateLoc36616 {
    pub(crate) sound_effect: Option<&'static str>,
    pub(crate) d0_preserved: bool,
}

pub(crate) fn big_hopping_skull_hit_voice_gate_loc_36616(
    sprite_is_voice_frame: bool,
    animation_timer: u16,
) -> BigHoppingSkullHitVoiceGateLoc36616 {
    // ASM Big_Hopping_Skull.asm loc_36616: play `sfx_Voice_die` only for
    // `LnkTo_unk_C7FE0` when `animation_timer(a3)` is exactly $C; d0 is
    // restored after the sound call before collision probing continues.
    BigHoppingSkullHitVoiceGateLoc36616 {
        sound_effect: (sprite_is_voice_frame && animation_timer == 0x000C)
            .then_some("sfx_Voice_die"),
        d0_preserved: true,
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum BigHoppingSkullHitAnimationWaitRouteLoc36632 {
    Relaunch,
    LoopHitAnimation,
    SharedTail,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct BigHoppingSkullHitAnimationWaitLoc36632 {
    pub(crate) collision_probe_extent: u16,
    pub(crate) clear_has_kid_collision: bool,
    pub(crate) route: BigHoppingSkullHitAnimationWaitRouteLoc36632,
}

pub(crate) fn big_hopping_skull_hit_animation_wait_loc_36632(
    collision_probe_nonzero: bool,
    animation_finished: bool,
) -> BigHoppingSkullHitAnimationWaitLoc36632 {
    // ASM Big_Hopping_Skull.asm loc_36632: probe `loc_33286` with d7=#8;
    // a zero d6 relaunches immediately, otherwise the hit animation loops
    // until `$18(a3)` is set, then `has_kid_collision(a3)` is cleared.
    let route = if !collision_probe_nonzero {
        BigHoppingSkullHitAnimationWaitRouteLoc36632::Relaunch
    } else if animation_finished {
        BigHoppingSkullHitAnimationWaitRouteLoc36632::SharedTail
    } else {
        BigHoppingSkullHitAnimationWaitRouteLoc36632::LoopHitAnimation
    };

    BigHoppingSkullHitAnimationWaitLoc36632 {
        collision_probe_extent: 8,
        clear_has_kid_collision: matches!(
            route,
            BigHoppingSkullHitAnimationWaitRouteLoc36632::SharedTail
        ),
        route,
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct BigHoppingSkullDeathEntryLoc3664E {
    pub(crate) sound_effect: &'static str,
    pub(crate) d0_preserved: bool,
    pub(crate) animation: &'static str,
    pub(crate) next_loop: &'static str,
}

pub(crate) fn big_hopping_skull_death_entry_loc_3664e() -> BigHoppingSkullDeathEntryLoc3664E {
    // ASM Big_Hopping_Skull.asm loc_3664E: push d0, play
    // `sfx_Big_Hopping_Skull_dies`, restore d0, initialise `stru_367FA`,
    // then fall through to the death-animation wait loop.
    BigHoppingSkullDeathEntryLoc3664E {
        sound_effect: "sfx_Big_Hopping_Skull_dies",
        d0_preserved: true,
        animation: "stru_367FA",
        next_loop: "death_animation_wait",
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum BigHoppingSkullDeathWaitRouteLoc36664 {
    LoopDeathAnimation,
    FinalizeDeletion,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct BigHoppingSkullDeathWaitLoc36664 {
    pub(crate) hibernates_one_frame: bool,
    pub(crate) checks_object_range: bool,
    pub(crate) collision_probe_extent: u16,
    pub(crate) y_velocity: i32,
    pub(crate) route: BigHoppingSkullDeathWaitRouteLoc36664,
}

pub(crate) fn big_hopping_skull_death_wait_loc_36664(
    collision_probe_nonzero: bool,
    animation_finished: bool,
    y_velocity: i32,
) -> BigHoppingSkullDeathWaitLoc36664 {
    let y_velocity = if collision_probe_nonzero {
        y_velocity
    } else {
        write_velocity_high_word(y_velocity, 2)
    };

    // ASM Big_Hopping_Skull.asm loc_36664: hibernate once, range-check, probe
    // collision with d7=#8, write `#2` into the high word of `y_vel(a3)` when
    // d6 is zero, then keep waiting until `$18(a3)` is set.
    BigHoppingSkullDeathWaitLoc36664 {
        hibernates_one_frame: true,
        checks_object_range: true,
        collision_probe_extent: 8,
        y_velocity,
        route: if animation_finished {
            BigHoppingSkullDeathWaitRouteLoc36664::FinalizeDeletion
        } else {
            BigHoppingSkullDeathWaitRouteLoc36664::LoopDeathAnimation
        },
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum BigHoppingSkullDeathAnimationGateLoc3667E {
    LoopDeathAnimation,
    FinalizeDeathStatus,
}

pub(crate) fn big_hopping_skull_death_animation_gate_loc_3667e(
    animation_finished: bool,
) -> BigHoppingSkullDeathAnimationGateLoc3667E {
    // ASM Big_Hopping_Skull.asm loc_3667E: `tst.b $18(a3)` loops to
    // loc_36664 while the death animation is still active; once non-zero,
    // the routine loads `$42(a5)` into d0 for the death-status tail.
    if animation_finished {
        BigHoppingSkullDeathAnimationGateLoc3667E::FinalizeDeathStatus
    } else {
        BigHoppingSkullDeathAnimationGateLoc3667E::LoopDeathAnimation
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct BigHoppingSkullRespawnMarkerLoc366A4 {
    pub(crate) index: usize,
    pub(crate) word_value: u16,
}

pub(crate) fn big_hopping_skull_respawn_marker_loc_366a4(
    object_status_slot: u8,
) -> BigHoppingSkullRespawnMarkerLoc366A4 {
    // ASM Big_Hopping_Skull.asm loc_366A4: positive `$42(a5)` is doubled as
    // an EnemyStatus_Table word offset, then `$2168` is written before the
    // shared death finalisation tail.
    BigHoppingSkullRespawnMarkerLoc366A4 {
        index: usize::from(object_status_slot),
        word_value: 0x2168,
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct BigHoppingSkullDeleteTailLoc366B2 {
    pub(crate) deletes_current_object: bool,
}

pub(crate) fn big_hopping_skull_delete_tail_loc_366b2() -> BigHoppingSkullDeleteTailLoc366B2 {
    // ASM Big_Hopping_Skull.asm loc_366B2: all death-status finalisation paths
    // converge on `jmp (j_Delete_CurrentObject).w`.
    BigHoppingSkullDeleteTailLoc366B2 {
        deletes_current_object: true,
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum BigHoppingSkullRelaunchRouteLoc366B6 {
    DeleteCurrentObject,
    ReturnToContactWait,
    Relaunch(BigHoppingSkullBoundsCorrectionLoc36770),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct BigHoppingSkullRelaunchEntryLoc366B6 {
    pub(crate) checks_level_height: bool,
    pub(crate) animation: Option<&'static str>,
    pub(crate) hibernates_until_animation_finished: bool,
    pub(crate) has_level_collision: bool,
    pub(crate) route: BigHoppingSkullRelaunchRouteLoc366B6,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct BigHoppingSkullRelaunchInputLoc366B6 {
    pub(crate) y_pos_word: i16,
    pub(crate) level_height_pixels: i16,
    pub(crate) animation_finished: bool,
    pub(crate) saved_x_velocity: i32,
    pub(crate) current_y_velocity: i32,
    pub(crate) x_pos_word: i16,
    pub(crate) left_bound_word: i16,
    pub(crate) right_bound_word: i16,
    pub(crate) facing: Direction,
}

pub(crate) fn big_hopping_skull_relaunch_entry_loc_366b6(
    input: BigHoppingSkullRelaunchInputLoc366B6,
) -> BigHoppingSkullRelaunchEntryLoc366B6 {
    // ASM Big_Hopping_Skull.asm loc_366B6: rerun the level-height delete gate,
    // branch back to the contact wait while `$18(a3)` is clear, otherwise play
    // `stru_367B4`, hibernate until it finishes, restore level collision, and
    // relaunch from saved d0 x_vel before the corridor bound correction.
    if matches!(
        big_hopping_skull_level_height_gate_loc_3656c(input.y_pos_word, input.level_height_pixels,),
        BigHoppingSkullLevelHeightRouteLoc3656C::DeleteCurrentObject
    ) {
        return BigHoppingSkullRelaunchEntryLoc366B6 {
            checks_level_height: true,
            animation: None,
            hibernates_until_animation_finished: false,
            has_level_collision: false,
            route: BigHoppingSkullRelaunchRouteLoc366B6::DeleteCurrentObject,
        };
    }

    if !input.animation_finished {
        return BigHoppingSkullRelaunchEntryLoc366B6 {
            checks_level_height: true,
            animation: None,
            hibernates_until_animation_finished: false,
            has_level_collision: false,
            route: BigHoppingSkullRelaunchRouteLoc366B6::ReturnToContactWait,
        };
    }

    let y_velocity = write_velocity_high_word(input.current_y_velocity, -2);
    let motion = big_hopping_skull_bounds_correction_loc_36770(
        input.x_pos_word,
        input.left_bound_word,
        input.right_bound_word,
        input.saved_x_velocity,
        y_velocity,
        input.facing,
    );

    BigHoppingSkullRelaunchEntryLoc366B6 {
        checks_level_height: true,
        animation: Some("stru_367B4"),
        hibernates_until_animation_finished: true,
        has_level_collision: true,
        route: BigHoppingSkullRelaunchRouteLoc366B6::Relaunch(motion),
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum BigHoppingSkullCollisionActionLoc3674E {
    Return,
    WallBounce,
    FloorContact,
    CeilingBounce,
    SlopeUpDeflect,
    SlopeDownDeflect,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct BigHoppingSkullCollisionDispatchLoc3674E {
    pub(crate) captured_collision_type: u16,
    pub(crate) next_collision_type: u16,
    pub(crate) action: BigHoppingSkullCollisionActionLoc3674E,
}

pub(crate) fn big_hopping_skull_collision_dispatch_loc_3674e(
    collision_type: u16,
) -> BigHoppingSkullCollisionDispatchLoc3674E {
    const COLID_RIGHT_WALL: u16 = 0x0004;
    const COLID_LEFT_WALL: u16 = 0x0008;
    const COLID_FLOOR: u16 = 0x000C;
    const COLID_CEILING: u16 = 0x0010;
    const COLID_SLOPE_UP: u16 = 0x0014;
    const COLID_SLOPE_DOWN: u16 = 0x0018;
    // ASM Big_Hopping_Skull.asm loc_3674E: zero collision returns without
    // clearing; any non-zero word is cleared before hurt-or-higher contacts are
    // ignored or terrain contacts dispatch through off_36736.
    let next_collision_type = if collision_type == 0 {
        collision_type
    } else {
        0
    };
    let action = match collision_type {
        COLID_RIGHT_WALL | COLID_LEFT_WALL => BigHoppingSkullCollisionActionLoc3674E::WallBounce,
        COLID_FLOOR => BigHoppingSkullCollisionActionLoc3674E::FloorContact,
        COLID_CEILING => BigHoppingSkullCollisionActionLoc3674E::CeilingBounce,
        COLID_SLOPE_UP => BigHoppingSkullCollisionActionLoc3674E::SlopeUpDeflect,
        COLID_SLOPE_DOWN => BigHoppingSkullCollisionActionLoc3674E::SlopeDownDeflect,
        _ => BigHoppingSkullCollisionActionLoc3674E::Return,
    };

    BigHoppingSkullCollisionDispatchLoc3674E {
        captured_collision_type: collision_type,
        next_collision_type,
        action,
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum BigHoppingSkullBoundsRouteLoc36770 {
    Unchanged,
    HitLeftBound,
    HitRightBound,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct BigHoppingSkullBoundsCorrectionLoc36770 {
    pub(crate) x_velocity: i32,
    pub(crate) y_velocity: i32,
    pub(crate) facing: Direction,
    pub(crate) route: BigHoppingSkullBoundsRouteLoc36770,
}

pub(crate) fn big_hopping_skull_bounds_correction_loc_36770(
    x_pos_word: i16,
    left_bound_word: i16,
    right_bound_word: i16,
    x_velocity: i32,
    y_velocity: i32,
    facing: Direction,
) -> BigHoppingSkullBoundsCorrectionLoc36770 {
    // ASM Big_Hopping_Skull.asm loc_36770: signed word compare against `$3E/$40(a3)`.
    // Hitting either bound writes only the high word of y_vel/x_vel, preserving
    // the low fractional words, then sets x_direction ($FF right, $00 left).
    if x_pos_word <= left_bound_word {
        // ASM Big_Hopping_Skull.asm loc_36782: y_vel.high=$FFFE,
        // x_vel.high=1, and `st x_direction(a3)` faces right.
        BigHoppingSkullBoundsCorrectionLoc36770 {
            x_velocity: write_velocity_high_word(x_velocity, 1),
            y_velocity: write_velocity_high_word(y_velocity, -2),
            facing: Direction::Right,
            route: BigHoppingSkullBoundsRouteLoc36770::HitLeftBound,
        }
    } else if x_pos_word >= right_bound_word {
        // ASM Big_Hopping_Skull.asm loc_36798: y_vel.high=$FFFE,
        // x_vel.high=$FFFF, and `sf x_direction(a3)` faces left.
        BigHoppingSkullBoundsCorrectionLoc36770 {
            x_velocity: write_velocity_high_word(x_velocity, -1),
            y_velocity: write_velocity_high_word(y_velocity, -2),
            facing: Direction::Left,
            route: BigHoppingSkullBoundsRouteLoc36770::HitRightBound,
        }
    } else {
        BigHoppingSkullBoundsCorrectionLoc36770 {
            x_velocity,
            y_velocity,
            facing,
            route: BigHoppingSkullBoundsRouteLoc36770::Unchanged,
        }
    }
}

/// Big Hopping Skull: continuous bouncer from Big_Hopping_Skull.asm.
/// Gravity $FA0/frame, deflects off all surfaces. No ground delay.
/// Floor: relaunch with y_vel high word $FFFE. Wall: backup 2x, negate x_vel, clear y_vel.
/// Ceiling: negate y_vel. Slope up: swap+negate both. Slope down: swap only.
#[cfg(test)]
pub(crate) fn step_big_hopping_skull(
    enemy: &mut ActiveEnemy,
    block_grid: Option<&terrain::BlockGrid>,
) {
    step_big_hopping_skull_with_bounds(enemy, block_grid, i16::MIN, i16::MAX);
}

pub(crate) fn step_big_hopping_skull_with_bounds(
    enemy: &mut ActiveEnemy,
    block_grid: Option<&terrain::BlockGrid>,
    left_bound_word: i16,
    right_bound_word: i16,
) {
    if enemy.wait_timer > 0 {
        enemy.wait_timer = enemy.wait_timer.saturating_sub(1);
        if enemy.wait_timer == 0 {
            let contact_wait = big_hopping_skull_contact_wait_loc_365ac(true, 0);
            debug_assert!(contact_wait.hibernates_one_frame);
            debug_assert!(contact_wait.checks_object_range);
            debug_assert_eq!(contact_wait.collision_probe_extent, 8);
            debug_assert_eq!(
                contact_wait.route,
                BigHoppingSkullContactWaitRouteLoc365AC::SharedTail,
                "native Big Hopping Skull wait_timer collapses ASM loc_365AC's no-collision shared-tail path into the loc_366B6 relaunch boundary",
            );
            let relaunch =
                big_hopping_skull_relaunch_entry_loc_366b6(BigHoppingSkullRelaunchInputLoc366B6 {
                    y_pos_word: (enemy.y_pos >> 16) as i16,
                    level_height_pixels: i16::MAX,
                    animation_finished: true,
                    saved_x_velocity: enemy.target_x_velocity,
                    current_y_velocity: enemy.y_velocity,
                    x_pos_word: (enemy.x_pos >> 16) as i16,
                    left_bound_word,
                    right_bound_word,
                    facing: enemy.facing,
                });
            debug_assert!(relaunch.checks_level_height);
            debug_assert_eq!(relaunch.animation, Some("stru_367B4"));
            debug_assert!(relaunch.hibernates_until_animation_finished);
            debug_assert!(relaunch.has_level_collision);
            let corrected = match relaunch.route {
                BigHoppingSkullRelaunchRouteLoc366B6::Relaunch(corrected) => corrected,
                BigHoppingSkullRelaunchRouteLoc366B6::DeleteCurrentObject
                | BigHoppingSkullRelaunchRouteLoc366B6::ReturnToContactWait => {
                    debug_assert!(
                        false,
                        "native Big Hopping Skull wait_timer reaches only ASM loc_366B6's animation-finished relaunch path",
                    );
                    let y_velocity = write_velocity_high_word(enemy.y_velocity, -2);
                    big_hopping_skull_bounds_correction_loc_36770(
                        (enemy.x_pos >> 16) as i16,
                        left_bound_word,
                        right_bound_word,
                        enemy.target_x_velocity,
                        y_velocity,
                        enemy.facing,
                    )
                }
            };
            // ASM Big_Hopping_Skull.asm loc_366D0: after the loc_36580 floor
            // contact and loc_366B6 relaunch animations, restore saved d0 into
            // x_vel and write only y_vel's high word to $FFFE before loc_36770.
            enemy.x_velocity = corrected.x_velocity;
            enemy.y_velocity = corrected.y_velocity;
            enemy.facing = corrected.facing;
            enemy.on_ground = false;
        }
        return;
    }

    // Apply gravity (ASM loc_36540: addi.l #$FA0,y_vel)
    enemy.y_velocity = enemy.y_velocity.saturating_add(BIG_SKULL_GRAVITY);

    // Advance position
    enemy.x_pos = (enemy.x_pos as i64 + i64::from(enemy.x_velocity)) as u32;
    enemy.y_pos = (enemy.y_pos as i64 + i64::from(enemy.y_velocity)) as u32;

    if let Some(grid) = block_grid {
        let cx = (enemy.x_pos >> 16) as i32;
        let cy = (enemy.y_pos >> 16) as i32;

        // Wall collision (ASM loc_366EC: backup 2x, negate x_vel, clear y_vel)
        let moving_right = !enemy.x_velocity.is_negative();
        let wall = terrain::probe_wall(
            grid,
            cx,
            cy,
            ENEMY_PROBE_HALF_WIDTH,
            ENEMY_PROBE_HEIGHT,
            moving_right,
        );
        if wall.hit {
            let collision_type = if moving_right { 0x0004 } else { 0x0008 };
            let dispatch = big_hopping_skull_collision_dispatch_loc_3674e(collision_type);
            debug_assert_eq!(dispatch.captured_collision_type, collision_type);
            debug_assert_eq!(dispatch.next_collision_type, 0);
            debug_assert_eq!(
                dispatch.action,
                BigHoppingSkullCollisionActionLoc3674E::WallBounce,
                "ASM Big_Hopping_Skull.asm loc_3674E should route wall collision to loc_366EC",
            );
            // Backup position by 2x x_velocity
            let backup = 2 * i64::from(enemy.x_velocity);
            enemy.x_pos = (enemy.x_pos as i64 - backup).max(0) as u32;
            enemy.y_velocity = 0;
            enemy.x_velocity = -enemy.x_velocity;
            enemy.facing = opposite_direction(enemy.facing);
        }

        // Floor collision. ASM loc_36580 clears motion during the contact wait;
        // loc_366D0 relaunches by restoring saved d0=x_vel and writing only
        // y_vel's high word to $FFFE.
        let floor = terrain::probe_floor(grid, cx, cy);
        if floor.on_ground && enemy.y_velocity.is_positive() {
            let collision_type = match floor.cell.collision {
                terrain::CollisionKind::SlopeUp => 0x0014,
                terrain::CollisionKind::SlopeDown => 0x0018,
                _ => 0x000C,
            };
            let dispatch = big_hopping_skull_collision_dispatch_loc_3674e(collision_type);
            debug_assert_eq!(dispatch.captured_collision_type, collision_type);
            debug_assert_eq!(dispatch.next_collision_type, 0);

            match dispatch.action {
                BigHoppingSkullCollisionActionLoc3674E::FloorContact => {
                    let saved_x_velocity = enemy.x_velocity;
                    let floor_y_fp = (floor.floor_y_px as u32) << 16;
                    if enemy.y_pos >= floor_y_fp {
                        enemy.y_pos = floor_y_fp;
                        // ASM Big_Hopping_Skull.asm loc_36580: floor contact saves
                        // d0=x_vel, clears both velocities, disables level collision,
                        // and hibernates through `stru_367AE` before loc_366B6 can
                        // relaunch. Native stores saved d0 in target_x_velocity while
                        // wait_timer carries the two animation waits.
                        enemy.target_x_velocity = saved_x_velocity;
                        enemy.x_velocity = 0;
                        enemy.y_velocity = 0;
                        enemy.on_ground = true;
                        enemy.wait_timer = BIG_HOPPING_SKULL_FLOOR_CONTACT_WAIT_FRAMES_LOC_36580;
                    }
                }
                BigHoppingSkullCollisionActionLoc3674E::SlopeUpDeflect => {
                    // ASM loc_36706: swap and negate both
                    let old_x = enemy.x_velocity;
                    let old_y = enemy.y_velocity;
                    enemy.y_velocity = -old_x;
                    enemy.x_velocity = -old_y;
                    enemy.facing = opposite_direction(enemy.facing);
                }
                BigHoppingSkullCollisionActionLoc3674E::SlopeDownDeflect => {
                    // ASM loc_36720: swap without negate
                    let old_x = enemy.x_velocity;
                    let old_y = enemy.y_velocity;
                    enemy.y_velocity = old_x;
                    enemy.x_velocity = old_y;
                    enemy.facing = opposite_direction(enemy.facing);
                }
                BigHoppingSkullCollisionActionLoc3674E::Return
                | BigHoppingSkullCollisionActionLoc3674E::WallBounce
                | BigHoppingSkullCollisionActionLoc3674E::CeilingBounce => {}
            }
        }

        // Ceiling bounce (ASM loc_366E6: neg.l y_vel)
        let ceil = terrain::probe_ceiling(grid, cx, cy, ENEMY_PROBE_HALF_WIDTH, ENEMY_PROBE_HEIGHT);
        if ceil.hit && enemy.y_velocity.is_negative() {
            let dispatch = big_hopping_skull_collision_dispatch_loc_3674e(0x0010);
            debug_assert_eq!(dispatch.captured_collision_type, 0x0010);
            debug_assert_eq!(dispatch.next_collision_type, 0);
            debug_assert_eq!(
                dispatch.action,
                BigHoppingSkullCollisionActionLoc3674E::CeilingBounce,
                "ASM Big_Hopping_Skull.asm loc_3674E should route ceiling collision to loc_366E6",
            );
            enemy.y_velocity = -enemy.y_velocity;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::enemies::test_fixtures::mini_hopping_skull_test_enemy;

    fn big_skull_at(x_pos: u32, y_pos: u32) -> ActiveEnemy {
        ActiveEnemy {
            family: EnemyFamily::BigHoppingSkull,
            x_pos,
            y_pos,
            hit_points: 3,
            on_ground: false,
            facing: Direction::Right,
            ..mini_hopping_skull_test_enemy()
        }
    }

    fn empty_grid() -> terrain::BlockGrid {
        terrain::BlockGrid::from_level_layout(&[0; 8 * 8 * 2], 8, 8)
    }

    fn grid_with_cell(col: u16, row: u16, cell: terrain::Cell) -> terrain::BlockGrid {
        let mut grid = empty_grid();
        grid.set_cell_at(col, row, cell);
        grid
    }

    fn solid_cell() -> terrain::Cell {
        terrain::Cell::from_be_bytes(0x60, 0x00)
    }

    fn slope_up_cell() -> terrain::Cell {
        terrain::Cell::from_be_bytes(0x40, 0x00)
    }

    fn slope_down_cell() -> terrain::Cell {
        terrain::Cell::from_be_bytes(0x50, 0x00)
    }

    #[test]
    fn big_hopping_skull_floor_contact_wait_frames_match_loc_36580_366b6() {
        assert_eq!(
            BIG_HOPPING_SKULL_FLOOR_CONTACT_WAIT_FRAMES_LOC_36580, 0x005D,
            "ASM Big_Hopping_Skull.asm loc_36580 uses `$50` ticks, then loc_366B6/stru_367B4 contributes `$0C + 1` ticks before relaunch",
        );
    }

    #[test]
    fn big_hopping_skull_damage_and_death_word_writes_preserve_fraction_loc_365d8_36664() {
        assert_eq!(
            big_hopping_skull_damage_entry_loc_365d8(2, 0x1234_8001).kid_y_velocity,
            i32::from_ne_bytes(0xFFFD_8001_u32.to_ne_bytes()),
            "ASM Big_Hopping_Skull.asm loc_365D8 writes only Kid y_vel.high=$FFFD",
        );
        assert_eq!(
            big_hopping_skull_death_wait_loc_36664(false, false, 0x1234_CDEF).y_velocity,
            0x0002_CDEF,
            "ASM Big_Hopping_Skull.asm loc_36664 writes only skull y_vel.high=2 when the collision probe is clear",
        );
        assert_eq!(
            big_hopping_skull_death_wait_loc_36664(true, false, 0x1234_CDEF).y_velocity,
            0x1234_CDEF,
            "ASM Big_Hopping_Skull.asm loc_36664 leaves y_vel untouched when d6 is non-zero",
        );
    }

    #[test]
    fn big_hopping_skull_collision_dispatch_covers_floor_and_ignored_contacts_loc_3674e() {
        assert_eq!(
            big_hopping_skull_collision_dispatch_loc_3674e(0x000C),
            BigHoppingSkullCollisionDispatchLoc3674E {
                captured_collision_type: 0x000C,
                next_collision_type: 0,
                action: BigHoppingSkullCollisionActionLoc3674E::FloorContact,
            },
            "ASM Big_Hopping_Skull.asm loc_3674E indexes off_36736's floor entry at colid_floor",
        );
        assert_eq!(
            big_hopping_skull_collision_dispatch_loc_3674e(0x0020),
            BigHoppingSkullCollisionDispatchLoc3674E {
                captured_collision_type: 0x0020,
                next_collision_type: 0,
                action: BigHoppingSkullCollisionActionLoc3674E::Return,
            },
            "ASM Big_Hopping_Skull.asm loc_3674E clears but ignores contacts at or above colid_hurt",
        );
    }

    #[test]
    fn big_hopping_skull_live_step_advances_position_before_collision_loc_36540() {
        let mut skull = big_skull_at(32 << 16, 32 << 16);
        skull.x_velocity = 0x0001_0000;
        skull.y_velocity = 0;

        step_big_hopping_skull(&mut skull, None);

        assert_eq!(
            skull.x_pos,
            33 << 16,
            "ASM Big_Hopping_Skull.asm loc_36540 advances x_pos by x_vel before terrain dispatch",
        );
        assert_eq!(skull.y_velocity, BIG_SKULL_GRAVITY);
        assert_eq!(skull.y_pos, (32 << 16) + BIG_SKULL_GRAVITY as u32);
    }

    #[test]
    fn big_hopping_skull_wall_bounce_backs_up_twice_and_flips_loc_366ec() {
        let grid = grid_with_cell(3, 3, solid_cell());
        let mut skull = big_skull_at(38 << 16, 64 << 16);
        skull.x_velocity = 0x0001_0000;
        skull.y_velocity = 0;
        skull.facing = Direction::Right;

        step_big_hopping_skull(&mut skull, Some(&grid));

        assert_eq!(
            skull.x_pos,
            37 << 16,
            "ASM Big_Hopping_Skull.asm loc_366EC subtracts x_vel twice from the already-advanced x_pos",
        );
        assert_eq!(skull.x_velocity, -0x0001_0000);
        assert_eq!(skull.y_velocity, 0);
        assert_eq!(skull.facing, Direction::Left);
    }

    #[test]
    fn big_hopping_skull_empty_fall_does_not_start_floor_wait_loc_36580() {
        let grid = empty_grid();
        let mut skull = big_skull_at(32 << 16, 32 << 16);
        skull.x_velocity = 0x0001_0000;
        skull.y_velocity = 0;

        step_big_hopping_skull(&mut skull, Some(&grid));

        assert_eq!(skull.wait_timer, 0);
        assert_eq!(skull.x_velocity, 0x0001_0000);
        assert_eq!(skull.y_velocity, BIG_SKULL_GRAVITY);
        assert_eq!(skull.y_pos, (32 << 16) + BIG_SKULL_GRAVITY as u32);
    }

    #[test]
    fn big_hopping_skull_zero_vertical_speed_does_not_land_loc_36580() {
        let grid = grid_with_cell(2, 4, solid_cell());
        let mut skull = big_skull_at(32 << 16, 64 << 16);
        skull.x_velocity = 0x0001_2222;
        skull.y_velocity = -BIG_SKULL_GRAVITY;

        step_big_hopping_skull(&mut skull, Some(&grid));

        assert_eq!(
            skull.wait_timer, 0,
            "ASM Big_Hopping_Skull.asm loc_3674E only reaches loc_36580 for a downward floor collision",
        );
        assert_eq!(skull.x_velocity, 0x0001_2222);
        assert_eq!(skull.y_velocity, 0);
    }

    #[test]
    fn big_hopping_skull_slope_up_deflects_nonzero_velocity_loc_36706() {
        let grid = grid_with_cell(2, 3, slope_up_cell());
        let mut skull = big_skull_at(45 << 16, 47 << 16);
        skull.x_velocity = 0x0002_0000;
        skull.y_velocity = 0;
        skull.facing = Direction::Right;

        step_big_hopping_skull(&mut skull, Some(&grid));

        assert_eq!(
            skull.y_velocity, -0x0002_0000,
            "ASM Big_Hopping_Skull.asm loc_36706 negates old x_vel into y_vel",
        );
        assert_eq!(
            skull.x_velocity, -BIG_SKULL_GRAVITY,
            "ASM Big_Hopping_Skull.asm loc_36706 negates old y_vel into x_vel after gravity",
        );
        assert_eq!(skull.facing, Direction::Left);
    }

    #[test]
    fn big_hopping_skull_slope_down_swaps_nonzero_velocity_loc_36720() {
        let grid = grid_with_cell(2, 3, slope_down_cell());
        let mut skull = big_skull_at(30 << 16, 47 << 16);
        skull.x_velocity = 0x0002_0000;
        skull.y_velocity = 0;
        skull.facing = Direction::Right;

        step_big_hopping_skull(&mut skull, Some(&grid));

        assert_eq!(
            skull.y_velocity, 0x0002_0000,
            "ASM Big_Hopping_Skull.asm loc_36720 copies old x_vel into y_vel",
        );
        assert_eq!(
            skull.x_velocity, BIG_SKULL_GRAVITY,
            "ASM Big_Hopping_Skull.asm loc_36720 copies old y_vel into x_vel after gravity",
        );
        assert_eq!(skull.facing, Direction::Left);
    }

    #[test]
    fn big_hopping_skull_ceiling_bounce_negates_only_rising_velocity_loc_366e6() {
        let grid = grid_with_cell(2, 2, solid_cell());
        let mut skull = big_skull_at(32 << 16, 64 << 16);
        skull.y_velocity = -0x0001_0000 - BIG_SKULL_GRAVITY;

        step_big_hopping_skull(&mut skull, Some(&grid));

        assert_eq!(
            skull.y_velocity, 0x0001_0000,
            "ASM Big_Hopping_Skull.asm loc_366E6 negates y_vel on ceiling collision",
        );
    }

    #[test]
    fn big_hopping_skull_rising_without_ceiling_keeps_negative_velocity_loc_366e6() {
        let grid = empty_grid();
        let mut skull = big_skull_at(32 << 16, 64 << 16);
        skull.y_velocity = -0x0001_0000 - BIG_SKULL_GRAVITY;

        step_big_hopping_skull(&mut skull, Some(&grid));

        assert_eq!(
            skull.y_velocity, -0x0001_0000,
            "ASM Big_Hopping_Skull.asm loc_366E6 is only reached when the ceiling collision dispatch fires",
        );
    }
}
