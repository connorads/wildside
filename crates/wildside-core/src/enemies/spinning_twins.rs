use super::*;

// SpinningTwins: 8-entry orbit X offset table from ASM (word_3B9A0).
// Applied relative to spawn position during orbit phase.
pub(crate) const SPINNING_TWINS_ORBIT_OFFSETS: [i32; 8] =
    [-0x10, -0x0A, 0x05, 0x11, 0x10, 0x0A, -0x05, -0x11];
pub(crate) const SPINNING_TWINS_ORBIT_FRAME_DURATION_LOC_3B904: u16 = 6;
/// Orbit-to-walk transition: collision-triggered. ASM loc_3B904 checks
/// collision_type each frame — negative, $2C, or $1C all trigger the drop.
/// In native, this direct fallback checks the player anchor against the same
/// live CollisionSize row used by the ordinary collision pipeline.
const SPINNING_TWINS_LIVE_CONTACT_LEFT_LOC_1BCE: i32 = -5;
const SPINNING_TWINS_LIVE_CONTACT_RIGHT_LOC_1BCE: i32 = 5;
const SPINNING_TWINS_LIVE_CONTACT_TOP_LOC_1BCE: i32 = -31;
const SPINNING_TWINS_LIVE_CONTACT_BOTTOM_LOC_1BCE: i32 = 0;
// Gravity applied each frame once the twins drop. ASM: $2800 FP.
pub(crate) const SPINNING_TWINS_GRAVITY: i32 = 0x2800;
// Launch Y velocity when transitioning from orbit. ASM loc_3BB24: -$10000.
pub(crate) const SPINNING_TWINS_DROP_Y_VEL: i32 = -0x1_0000;
// Initial X velocity on drop: ASM sets $FFFC8000 (left) or $38000 (right).
pub(crate) const SPINNING_TWINS_DROP_X_VEL_LEFT: i32 = -0x3_8000;
pub(crate) const SPINNING_TWINS_DROP_X_VEL_RIGHT: i32 = 0x3_8000;
pub(crate) const SPINNING_TWINS_DROP_X_DRAG: i32 = 0x1200;
pub(crate) const SPINNING_TWINS_WALKING_CHILD_X_OFFSET_PX: u32 = 0x20;
pub(crate) const SPINNING_TWINS_WALKING_SETTLE_X_SPEED: i32 = 0x3000;
pub(crate) const SPINNING_TWINS_COLID_RIGHTWALL: u16 = colid::RIGHTWALL;
pub(crate) const SPINNING_TWINS_COLID_LEFTWALL: u16 = colid::LEFTWALL;
pub(crate) const SPINNING_TWINS_COLID_FLOOR: u16 = colid::FLOOR;
pub(crate) const SPINNING_TWINS_COLID_HURT: u16 = colid::HURT;
pub(crate) const SPINNING_TWINS_COLID_KIDRIGHT: u16 = colid::KIDRIGHT;
pub(crate) const SPINNING_TWINS_COLID_KIDLEFT: u16 = colid::KIDLEFT;
pub(crate) const SPINNING_TWINS_COLID_KIDABOVE: u16 = colid::KIDABOVE;
/// ASM Spinning_Twins.asm loc_3BC34 plays `stru_3B836`: one $14-tick frame
/// before restoring `stru_3B81C` and resuming the shared walker.
pub(crate) const SPINNING_TWINS_HURT_ANIMATION_FRAMES_STRU_3B836: u8 = 0x14;
pub(crate) const SPINNING_TWINS_DROP_SPLIT_PENDING_LOC_3B93C: u16 = 0xB93C;
pub(crate) const SPINNING_TWINS_PARTNER_DEATH_SIGNAL_LOC_3BCA6: i16 = -1;

/// Atlas phase for the visible object after the orbit parent splits.
pub(crate) fn spinning_twins_live_sprite_phase(
    phase: SpinningTwinsPhase,
    on_ground: bool,
    hurt_frames: u8,
    animation_elapsed: u8,
) -> Option<u8> {
    // ASM Spinning_Twins.asm loc_3BA74 fixes each launched child on C7CDE;
    // loc_3B9B0 fixes the falling transition worker on C7D26; loc_3BB94
    // installs the 32-tick `stru_3B81C` walk loop after settlement.
    match phase {
        SpinningTwinsPhase::Orbiting => None,
        SpinningTwinsPhase::Walking if !on_ground || hurt_frames > 0 => Some(4),
        SpinningTwinsPhase::Walking => Some(match animation_elapsed % 32 {
            0..=5 => 8,
            6..=10 => 9,
            11..=15 => 10,
            16..=21 => 11,
            22..=26 => 12,
            27..=31 => 13,
            _ => unreachable!("modulo-32 Spinning Twins animation phase"),
        }),
        SpinningTwinsPhase::FallingVisual => Some(7),
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct SpinningTwinsOrbitTickLoc3B904 {
    pub(crate) next_animation_timer: u16,
    pub(crate) next_orbit_index: u8,
    pub(crate) child_x_offset_px: i32,
    pub(crate) advances_child_x: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum SpinningTwinsOrbitCollisionRouteLoc3B904 {
    ContinueOrbit,
    DropSplit,
}

pub(crate) fn spinning_twins_orbit_collision_route_loc_3b904(
    collision_type: i16,
) -> SpinningTwinsOrbitCollisionRouteLoc3B904 {
    // ASM Spinning_Twins.asm loc_3B904: branch to loc_3B93C when
    // `collision_type(a3)` is signed-negative, `colid_kidabove` ($2C), or
    // `colid_hurt` ($1C); every other word keeps orbiting.
    if collision_type.is_negative()
        || matches!(
            collision_type as u16,
            SPINNING_TWINS_COLID_KIDABOVE | SPINNING_TWINS_COLID_HURT
        )
    {
        SpinningTwinsOrbitCollisionRouteLoc3B904::DropSplit
    } else {
        SpinningTwinsOrbitCollisionRouteLoc3B904::ContinueOrbit
    }
}

fn spinning_twins_orbit_player_point_contact_loc_1bce(
    enemy_x_px: i32,
    enemy_y_px: i32,
    player_x_px: i32,
    player_y_px: i32,
) -> bool {
    // ASM Spinning_Twins.asm live orbit frames `LnkTo_unk_C7D26..C7D4E`
    // route through GfxObjects_CollisionKid loc_1BCE to `unk_314D0`
    // (`dc.w -$5,$A,-$1F,$1F`). Native only has the player anchor here, so
    // use the enemy-side row inclusively instead of the old sprite-size guess.
    let left = enemy_x_px + SPINNING_TWINS_LIVE_CONTACT_LEFT_LOC_1BCE;
    let right = enemy_x_px + SPINNING_TWINS_LIVE_CONTACT_RIGHT_LOC_1BCE;
    let top = enemy_y_px + SPINNING_TWINS_LIVE_CONTACT_TOP_LOC_1BCE;
    debug_assert_eq!(SPINNING_TWINS_LIVE_CONTACT_BOTTOM_LOC_1BCE, 0);
    let bottom = enemy_y_px;
    (left..=right).contains(&player_x_px) && (top..=bottom).contains(&player_y_px)
}

pub(crate) fn spinning_twins_orbit_tick_loc_3b904(
    animation_timer: u16,
    orbit_index: u8,
) -> SpinningTwinsOrbitTickLoc3B904 {
    // ASM Spinning_Twins.asm loc_3B904: after the one-frame hibernate, the
    // global animation step leaves `animation_timer(a3)` at 6..1 for
    // `stru_3B7F0`; only the exact `#1` frame increments `d1` and writes
    // `d2 + word_3B9A0[d1]` to the linked child GFX `$1A(a1)`.
    let next_animation_timer = match animation_timer {
        // Native `0` is an uninitialised sentinel; the ROM starts at
        // `animation_timer=1`, so the first hibernate/animate tick reloads
        // `stru_3B7F0` frame duration 6 before loc_3B904 observes it.
        0 | 1 => SPINNING_TWINS_ORBIT_FRAME_DURATION_LOC_3B904,
        timer => timer - 1,
    };
    let advances_child_x = next_animation_timer == 1;
    let next_orbit_index = if advances_child_x {
        (orbit_index + 1) & 7
    } else {
        orbit_index & 7
    };
    SpinningTwinsOrbitTickLoc3B904 {
        next_animation_timer,
        next_orbit_index,
        child_x_offset_px: SPINNING_TWINS_ORBIT_OFFSETS[usize::from(next_orbit_index)],
        advances_child_x,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum SpinningTwinsWalkingVelocityGateLoc3BB5A {
    CollisionDispatch,
    Settle,
}

pub(crate) fn spinning_twins_walking_velocity_gate_loc_3bb5a(
    x_velocity: i32,
) -> SpinningTwinsWalkingVelocityGateLoc3BB5A {
    // ASM Spinning_Twins.asm loc_3BB5A: `cmpi.l #$3000,x_vel(a3)` keeps
    // high positive speed in the collision-dispatch path; otherwise
    // `cmpi.l #$FFFFD000,x_vel(a3)` selects the settle path.
    // The negative edge is exclusive because equal `-$3000` falls through to
    // the collision dispatch, while positive `+$3000` settles.
    if x_velocity > SPINNING_TWINS_WALKING_SETTLE_X_SPEED
        || x_velocity <= -SPINNING_TWINS_WALKING_SETTLE_X_SPEED
    {
        SpinningTwinsWalkingVelocityGateLoc3BB5A::CollisionDispatch
    } else {
        SpinningTwinsWalkingVelocityGateLoc3BB5A::Settle
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum SpinningTwinsWalkingCollisionRouteLoc3BB6E {
    FloorLanding,
    WallSettle,
    ContinueAirborne,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct SpinningTwinsWalkingCollisionDispatchLoc3BB6E {
    pub(crate) next_collision_type: u16,
    pub(crate) route: SpinningTwinsWalkingCollisionRouteLoc3BB6E,
}

pub(crate) fn spinning_twins_walking_collision_dispatch_loc_3bb6e(
    collision_type: u16,
) -> SpinningTwinsWalkingCollisionDispatchLoc3BB6E {
    // ASM Spinning_Twins.asm loc_3BB6E: copy `collision_type(a3)` to d4,
    // route floor contact to the landing path, route either side wall to the
    // settle path, and otherwise clear collision_type before the airborne loop.
    let route = match collision_type {
        SPINNING_TWINS_COLID_FLOOR => SpinningTwinsWalkingCollisionRouteLoc3BB6E::FloorLanding,
        SPINNING_TWINS_COLID_RIGHTWALL | SPINNING_TWINS_COLID_LEFTWALL => {
            SpinningTwinsWalkingCollisionRouteLoc3BB6E::WallSettle
        }
        _ => SpinningTwinsWalkingCollisionRouteLoc3BB6E::ContinueAirborne,
    };

    SpinningTwinsWalkingCollisionDispatchLoc3BB6E {
        next_collision_type: 0,
        route,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct SpinningTwinsFloorLandingLoc3BB8A {
    pub(crate) next_collision_type: u16,
    pub(crate) y_velocity: i32,
}

pub(crate) fn spinning_twins_floor_landing_loc_3bb8a(
    collision_type: u16,
    y_velocity: i32,
) -> SpinningTwinsFloorLandingLoc3BB8A {
    // ASM Spinning_Twins.asm loc_3BB8A: after floor contact, clear
    // `collision_type(a3)`, clear `y_vel(a3)`, then resume the walking-child
    // movement loop.
    let _ = collision_type;
    let _ = y_velocity;
    SpinningTwinsFloorLandingLoc3BB8A {
        next_collision_type: 0,
        y_velocity: 0,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum SpinningTwinsSettleWalkerEntryLoc3BB94 {
    LeftLoc3C0D2,
    RightLoc3C026,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct SpinningTwinsSettleEntryLoc3BB94 {
    pub(crate) next_collision_type: u16,
    pub(crate) x_velocity: i32,
    pub(crate) y_velocity: i32,
    pub(crate) has_level_collision: bool,
    pub(crate) is_moved: bool,
    pub(crate) restart_walk_animation: bool,
    pub(crate) walker_entry: SpinningTwinsSettleWalkerEntryLoc3BB94,
}

pub(crate) fn spinning_twins_settle_entry_loc_3bb94(
    x_direction: u8,
) -> SpinningTwinsSettleEntryLoc3BB94 {
    // ASM Spinning_Twins.asm loc_3BB94: clear collision_type, x_vel, y_vel,
    // has_level_collision, and is_moved, restart `stru_3B81C`, then branch by
    // x_direction: zero enters loc_3C0D2; nonzero enters loc_3C026.
    SpinningTwinsSettleEntryLoc3BB94 {
        next_collision_type: 0,
        x_velocity: 0,
        y_velocity: 0,
        has_level_collision: false,
        is_moved: false,
        restart_walk_animation: true,
        walker_entry: if x_direction == 0 {
            SpinningTwinsSettleWalkerEntryLoc3BB94::LeftLoc3C0D2
        } else {
            SpinningTwinsSettleWalkerEntryLoc3BB94::RightLoc3C026
        },
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum SpinningTwinsSettledCollisionRouteLoc3BBC4 {
    ResumeWalk,
    Death,
    Damage,
    BerzerkerSideGate,
}

pub(crate) fn spinning_twins_settled_collision_dispatch_loc_3bbc4(
    collision_type: u16,
) -> SpinningTwinsSettledCollisionRouteLoc3BBC4 {
    // ASM Spinning_Twins.asm loc_3BBC4: copy collision_type to d4, route zero
    // and unmatched positive words back to walk, route negative words to death,
    // Kid-above/hurt to damage, and side contacts to the Berzerker gate.
    if collision_type == 0 {
        return SpinningTwinsSettledCollisionRouteLoc3BBC4::ResumeWalk;
    }
    if collision_type & 0x8000 != 0 {
        return SpinningTwinsSettledCollisionRouteLoc3BBC4::Death;
    }
    match collision_type {
        SPINNING_TWINS_COLID_KIDABOVE | SPINNING_TWINS_COLID_HURT => {
            SpinningTwinsSettledCollisionRouteLoc3BBC4::Damage
        }
        SPINNING_TWINS_COLID_KIDRIGHT | SPINNING_TWINS_COLID_KIDLEFT => {
            SpinningTwinsSettledCollisionRouteLoc3BBC4::BerzerkerSideGate
        }
        _ => SpinningTwinsSettledCollisionRouteLoc3BBC4::ResumeWalk,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct SpinningTwinsResumeWalkLoc3BBE6 {
    pub(crate) next_collision_type: u16,
    pub(crate) resume_direction: Direction,
}

pub(crate) fn spinning_twins_resume_walk_loc_3bbe6(
    x_direction: u8,
) -> SpinningTwinsResumeWalkLoc3BBE6 {
    // ASM Spinning_Twins.asm loc_3BBE6: clear collision_type, then test
    // x_direction. Zero resumes the left-facing shared walker path; nonzero
    // resumes the right-facing path.
    SpinningTwinsResumeWalkLoc3BBE6 {
        next_collision_type: 0,
        resume_direction: if x_direction == 0 {
            Direction::Left
        } else {
            Direction::Right
        },
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct SpinningTwinsBerzerkerLaunchSetupLoc3BBF6 {
    pub(crate) has_level_collision: bool,
    pub(crate) has_kid_collision: bool,
    pub(crate) is_moved: bool,
    pub(crate) is_animated: bool,
    pub(crate) sprite_mapping: &'static str,
    pub(crate) x_velocity: i32,
    pub(crate) y_velocity: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum SpinningTwinsBerzerkerSideGateLoc3BBF6 {
    ResumeWalk(SpinningTwinsResumeWalkLoc3BBE6),
    Launch(SpinningTwinsBerzerkerLaunchSetupLoc3BBF6),
}

pub(crate) fn spinning_twins_berzerker_side_gate_loc_3bbf6(
    berzerker_charging: bool,
    x_direction: u8,
) -> SpinningTwinsBerzerkerSideGateLoc3BBF6 {
    // ASM Spinning_Twins.asm loc_3BBF6: clear `Berzerker_charging` branches
    // back to loc_3BBE6. A charging Berzerker disables terrain collision,
    // enables Kid collision and movement, freezes animation on
    // `LnkTo_unk_C7CDE`, zeroes X velocity, and launches upward at `-$48000`.
    if !berzerker_charging {
        return SpinningTwinsBerzerkerSideGateLoc3BBF6::ResumeWalk(
            spinning_twins_resume_walk_loc_3bbe6(x_direction),
        );
    }

    SpinningTwinsBerzerkerSideGateLoc3BBF6::Launch(SpinningTwinsBerzerkerLaunchSetupLoc3BBF6 {
        has_level_collision: false,
        has_kid_collision: true,
        is_moved: true,
        is_animated: false,
        sprite_mapping: "LnkTo_unk_C7CDE-Data_Index",
        x_velocity: 0,
        y_velocity: -0x0004_8000,
    })
}

pub(crate) const SPINNING_TWINS_BERZERKER_LAUNCH_GRAVITY_LOC_3BC22: i32 = 0x0000_4000;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct SpinningTwinsBerzerkerLaunchLoopTickLoc3BC22 {
    pub(crate) hibernates_one_frame: bool,
    pub(crate) runs_shared_retention_check_sub_3c3ce: bool,
    pub(crate) next_y_velocity: i32,
    pub(crate) loops: bool,
}

pub(crate) fn spinning_twins_berzerker_launch_loop_tick_loc_3bc22(
    y_velocity: i32,
) -> SpinningTwinsBerzerkerLaunchLoopTickLoc3BC22 {
    // ASM Spinning_Twins.asm loc_3BC22: each airborne frame hibernates once,
    // runs `sub_3C3CE`, adds `$4000` to `y_vel(a3)`, then branches back to
    // loc_3BC22.
    SpinningTwinsBerzerkerLaunchLoopTickLoc3BC22 {
        hibernates_one_frame: true,
        runs_shared_retention_check_sub_3c3ce: true,
        next_y_velocity: y_velocity.wrapping_add(SPINNING_TWINS_BERZERKER_LAUNCH_GRAVITY_LOC_3BC22),
        loops: true,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct SpinningTwinsDamageRecoveryLoc3BC34 {
    pub(crate) next_hit_points: u16,
    pub(crate) hit_animation: &'static str,
    pub(crate) waits_until_hit_animation_finished: bool,
    pub(crate) restored_walk_animation: &'static str,
    pub(crate) resume_walk: SpinningTwinsResumeWalkLoc3BBE6,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum SpinningTwinsDamageContactLoc3BC34 {
    Death { next_hit_points: u16 },
    Recover(SpinningTwinsDamageRecoveryLoc3BC34),
}

pub(crate) fn spinning_twins_damage_contact_loc_3bc34(
    hit_points: u16,
    x_direction: u8,
) -> SpinningTwinsDamageContactLoc3BC34 {
    // ASM Spinning_Twins.asm loc_3BC34: `subi.w #1,$44(a3)` sends the twins
    // to loc_3BC56 when HP reaches zero. Otherwise it plays `stru_3B836` to
    // completion, restores the walking `stru_3B81C` animation, then resumes
    // through loc_3BBE6.
    let next_hit_points = hit_points.wrapping_sub(1);
    if next_hit_points == 0 {
        return SpinningTwinsDamageContactLoc3BC34::Death { next_hit_points };
    }

    SpinningTwinsDamageContactLoc3BC34::Recover(SpinningTwinsDamageRecoveryLoc3BC34 {
        next_hit_points,
        hit_animation: "stru_3B836",
        waits_until_hit_animation_finished: true,
        restored_walk_animation: "stru_3B81C",
        resume_walk: spinning_twins_resume_walk_loc_3bbe6(x_direction),
    })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum SpinningTwinsDeathStatusRouteLoc3BC78 {
    ClearStatusSlot(SpinningTwinsClearStatusSlotLoc3BC98),
    SubtractSpawnCounter { index: usize, decrement: u16 },
    DeleteOnly,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct SpinningTwinsClearStatusSlotLoc3BC98 {
    pub(crate) index: usize,
    pub(crate) word_value: u16,
}

pub(crate) fn spinning_twins_clear_status_slot_loc_3bc98(
    object_status_slot: u8,
) -> SpinningTwinsClearStatusSlotLoc3BC98 {
    // ASM Spinning_Twins.asm loc_3BC98: the positive `$42(a5)` byte has already
    // been copied to d0; `add.w d0,d0` makes it a word offset into
    // EnemyStatus_Table, and `clr.w (a0,d0.w)` clears that status word.
    SpinningTwinsClearStatusSlotLoc3BC98 {
        index: usize::from(object_status_slot),
        word_value: 0,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct SpinningTwinsDeleteTailLoc3BCA2 {
    pub(crate) deletes_current_object: bool,
}

pub(crate) fn spinning_twins_delete_tail_loc_3bca2() -> SpinningTwinsDeleteTailLoc3BCA2 {
    // ASM Spinning_Twins.asm loc_3BCA2: every death-status path converges on
    // `jmp (j_Delete_CurrentObject).w`.
    SpinningTwinsDeleteTailLoc3BCA2 {
        deletes_current_object: true,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct SpinningTwinsPartnerSignalTailLoc3BCA6 {
    pub(crate) partner_slot: u8,
    pub(crate) next_partner_flag_5b: u8,
    pub(crate) deletes_current_object: bool,
}

pub(crate) fn spinning_twins_partner_signal_tail_loc_3bca6(
    partner_slot: u8,
) -> SpinningTwinsPartnerSignalTailLoc3BCA6 {
    // ASM Spinning_Twins.asm loc_3BCA6: `move.l $5C(a5),a2; st $5B(a2)`
    // marks the linked walking twin before deleting the current object.
    SpinningTwinsPartnerSignalTailLoc3BCA6 {
        partner_slot,
        next_partner_flag_5b: 0xFF,
        deletes_current_object: true,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct SpinningTwinsDeathStatusLoc3BC78 {
    pub(crate) status_route: SpinningTwinsDeathStatusRouteLoc3BC78,
    pub(crate) delete_tail: SpinningTwinsDeleteTailLoc3BCA2,
}

pub(crate) fn spinning_twins_death_status_route_loc_3bc78(
    object_status_slot: u8,
) -> SpinningTwinsDeathStatusRouteLoc3BC78 {
    // ASM Spinning_Twins.asm loc_3BC78: after `$44(a3)` reaches zero and
    // `Number_of_Enemy` is decremented, `$42(a5)` is copied into d0. A positive
    // byte takes the clear-slot path; a negative byte with bit 6 set masks to a
    // six-bit EnemyStatus_Table index and subtracts `$400`; the remaining
    // negative case deletes without touching the status table.
    if object_status_slot & 0x80 == 0 {
        return SpinningTwinsDeathStatusRouteLoc3BC78::ClearStatusSlot(
            spinning_twins_clear_status_slot_loc_3bc98(object_status_slot),
        );
    }
    if object_status_slot & 0x40 == 0 {
        return SpinningTwinsDeathStatusRouteLoc3BC78::DeleteOnly;
    }
    SpinningTwinsDeathStatusRouteLoc3BC78::SubtractSpawnCounter {
        index: usize::from(object_status_slot & 0x3F),
        decrement: 0x0400,
    }
}

pub(crate) fn spinning_twins_death_status_loc_3bc78(
    object_status_slot: u8,
) -> SpinningTwinsDeathStatusLoc3BC78 {
    SpinningTwinsDeathStatusLoc3BC78 {
        status_route: spinning_twins_death_status_route_loc_3bc78(object_status_slot),
        delete_tail: spinning_twins_delete_tail_loc_3bca2(),
    }
}

pub(crate) fn spinning_twins_walking_child_y_velocity_loc_3bb2c(y_velocity: i32) -> i32 {
    // ASM Spinning_Twins.asm loc_3BB2C: after hibernate + sub_3C3CE movement,
    // `tst.l y_vel(a3)` skips gravity when y_vel is already zero; otherwise
    // `addi.l #$2800,y_vel(a3)` eases the launched child toward falling.
    if y_velocity == 0 {
        0
    } else {
        y_velocity.wrapping_add(SPINNING_TWINS_GRAVITY)
    }
}

pub(crate) fn spinning_twins_walking_child_left_x_velocity_loc_3bb42(x_velocity: i32) -> i32 {
    // ASM Spinning_Twins.asm loc_3BB42: a set x_direction flag takes the
    // left-launched child through `addi.l #$1200,x_vel(a3)`.
    x_velocity.wrapping_add(SPINNING_TWINS_DROP_X_DRAG)
}

pub(crate) fn spinning_twins_walking_child_right_x_velocity_loc_3bb52(x_velocity: i32) -> i32 {
    // ASM Spinning_Twins.asm loc_3BB52: a clear x_direction flag takes the
    // right-launched child through `subi.l #$1200,x_vel(a3)`.
    x_velocity.wrapping_sub(SPINNING_TWINS_DROP_X_DRAG)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct SpinningTwinsFirstWalkingChildInitLoc3BB0C {
    pub(crate) x_pos: u32,
    pub(crate) y_pos: u32,
    pub(crate) x_direction_flipped: bool,
    pub(crate) x_velocity: i32,
    pub(crate) y_velocity: i32,
}

pub(crate) fn spinning_twins_first_walking_child_init_loc_3bb0c(
    parent_x_pos: u32,
    parent_y_pos: u32,
) -> SpinningTwinsFirstWalkingChildInitLoc3BB0C {
    // ASM Spinning_Twins.asm loc_3BB0C: the first walking child sets
    // x_direction, starts 32px left of the source object, then the shared
    // launch test assigns the negative X velocity. ASM loc_3BB24 then assigns
    // the shared upward Y velocity.
    let parent_x_word = (parent_x_pos >> 16) as u16;
    let parent_y_word = (parent_y_pos >> 16) as u16;
    SpinningTwinsFirstWalkingChildInitLoc3BB0C {
        x_pos: u32::from(
            parent_x_word.wrapping_sub(SPINNING_TWINS_WALKING_CHILD_X_OFFSET_PX as u16),
        ) << 16,
        y_pos: u32::from(parent_y_word) << 16,
        x_direction_flipped: true,
        x_velocity: SPINNING_TWINS_DROP_X_VEL_LEFT,
        y_velocity: SPINNING_TWINS_DROP_Y_VEL,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct SpinningTwinsSecondWalkingChildInitLoc3BB06 {
    pub(crate) x_pos: u32,
    pub(crate) y_pos: u32,
    pub(crate) x_direction_flipped: bool,
    pub(crate) x_velocity: i32,
    pub(crate) y_velocity: i32,
}

pub(crate) fn spinning_twins_second_walking_child_init_loc_3bb06(
    parent_x_pos: u32,
    parent_y_pos: u32,
) -> SpinningTwinsSecondWalkingChildInitLoc3BB06 {
    // ASM Spinning_Twins.asm loc_3BB06: the second walking child sees the
    // parent marker already set, keeps x_direction clear, and starts 32px to
    // the right before ASM loc_3BB1C assigns rightward velocity.
    let parent_x_word = (parent_x_pos >> 16) as u16;
    let parent_y_word = (parent_y_pos >> 16) as u16;
    SpinningTwinsSecondWalkingChildInitLoc3BB06 {
        x_pos: u32::from(
            parent_x_word.wrapping_add(SPINNING_TWINS_WALKING_CHILD_X_OFFSET_PX as u16),
        ) << 16,
        y_pos: u32::from(parent_y_word) << 16,
        x_direction_flipped: false,
        x_velocity: SPINNING_TWINS_DROP_X_VEL_RIGHT,
        y_velocity: SPINNING_TWINS_DROP_Y_VEL,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct SpinningTwinsDropTransitionLoc3B93C {
    pub(crate) split_child_count: u8,
    pub(crate) orbit_child_hidden: bool,
    pub(crate) falling_visual_spawned: bool,
    pub(crate) original_deleted_after_animation: bool,
}

pub(crate) fn spinning_twins_drop_transition_loc_3b93c(
    enemy: &mut ActiveEnemy,
) -> SpinningTwinsDropTransitionLoc3B93C {
    // ASM Spinning_Twins.asm loc_3B93C: collision hides the orbit child GFX,
    // allocates two walking twin objects plus one falling visual worker, then
    // deletes the original orbit object after its transition animation. The
    // walking children enter loc_3BA74 for their initial launch velocities.
    enemy.spinning_twins_phase = SpinningTwinsPhase::Walking;
    enemy.y_velocity = SPINNING_TWINS_DROP_Y_VEL;
    enemy.wait_timer = SPINNING_TWINS_DROP_SPLIT_PENDING_LOC_3B93C;
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
pub(crate) struct SpinningTwinsFallingVisualInitLoc3B9B0 {
    pub(crate) x_pos: u32,
    pub(crate) y_pos: u32,
    pub(crate) priority: u8,
    pub(crate) display_enabled: bool,
    pub(crate) step_index: u8,
    pub(crate) frames_until_next_step_index: u8,
}

pub(crate) fn spinning_twins_falling_visual_init_loc_3b9b0(
    parent_x_pos: u32,
    parent_y_pos: u32,
) -> SpinningTwinsFallingVisualInitLoc3B9B0 {
    // ASM Spinning_Twins.asm loc_3B9B0: copy the parent coordinates, bias the
    // falling visual 19px left and 64px up, make it visible at priority 1, and
    // seed the movement loop with d0=0/d1=4.
    let parent_x_word = (parent_x_pos >> 16) as u16;
    let parent_y_word = (parent_y_pos >> 16) as u16;
    SpinningTwinsFallingVisualInitLoc3B9B0 {
        x_pos: u32::from(parent_x_word.wrapping_sub(0x13)) << 16,
        y_pos: u32::from(parent_y_word.wrapping_sub(0x40)) << 16,
        priority: 1,
        display_enabled: true,
        step_index: 0,
        frames_until_next_step_index: 4,
    }
}

pub(crate) const SPINNING_TWINS_FALLING_VISUAL_X_STEPS_LOC_3BA00: [i32; 5] =
    [0x1_2000, 0x1_2000, 0x1_8000, 0x2_4000, 0x3_0000];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct SpinningTwinsFallingVisualStateLoc3BA00 {
    pub(crate) x_pos: u32,
    pub(crate) y_pos: u32,
    pub(crate) step_index: u8,
    pub(crate) frames_until_next_step_index: u8,
    pub(crate) reverse_x_motion: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct SpinningTwinsFallingVisualStepLoc3BA00 {
    pub(crate) state: SpinningTwinsFallingVisualStateLoc3BA00,
    pub(crate) alive: bool,
}

fn spinning_twins_falling_visual_screen_delta_px(position: u32, camera_position: u32) -> i16 {
    let position_px = (position >> 16) as u16;
    let camera_px = (camera_position >> 16) as u16;
    position_px.wrapping_sub(camera_px) as i16
}

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

pub(crate) fn materialize_spinning_twins_drop_splits(active: &mut Vec<ActiveEnemy>) {
    let original = std::mem::take(active);
    for source in original {
        let split_pending = matches!(source.family, EnemyFamily::SpinningTwins)
            && matches!(source.spinning_twins_phase, SpinningTwinsPhase::Walking)
            && source.wait_timer == SPINNING_TWINS_DROP_SPLIT_PENDING_LOC_3B93C;
        if split_pending {
            let first =
                spinning_twins_first_walking_child_init_loc_3bb0c(source.x_pos, source.y_pos);
            let second =
                spinning_twins_second_walking_child_init_loc_3bb06(source.x_pos, source.y_pos);
            let visual = spinning_twins_falling_visual_init_loc_3b9b0(source.x_pos, source.y_pos);

            let mut first_child = source;
            first_child.x_pos = first.x_pos;
            first_child.y_pos = first.y_pos;
            first_child.x_velocity = first.x_velocity;
            first_child.y_velocity = first.y_velocity;
            first_child.facing = Direction::Left;
            first_child.on_ground = false;
            first_child.wait_timer = 0;
            first_child.spinning_twins_phase = SpinningTwinsPhase::Walking;

            let mut second_child = source;
            second_child.x_pos = second.x_pos;
            second_child.y_pos = second.y_pos;
            second_child.x_velocity = second.x_velocity;
            second_child.y_velocity = second.y_velocity;
            second_child.facing = Direction::Right;
            second_child.on_ground = false;
            second_child.wait_timer = 0;
            second_child.spinning_twins_phase = SpinningTwinsPhase::Walking;

            let mut falling_visual = source;
            falling_visual.x_pos = visual.x_pos;
            falling_visual.y_pos = visual.y_pos;
            falling_visual.x_velocity = 0;
            falling_visual.y_velocity = 0;
            falling_visual.hit_points = 0;
            falling_visual.on_ground = false;
            falling_visual.wait_timer = 0;
            falling_visual.grounded_frames = visual.frames_until_next_step_index;
            falling_visual.spinning_twins_orbit_index = visual.step_index;
            falling_visual.shared_walker_patrol_short_next = false;
            falling_visual.spinning_twins_phase = SpinningTwinsPhase::FallingVisual;

            // ASM Spinning_Twins.asm loc_3B93C deletes the original orbit object
            // after allocating both loc_3BA74 walking children and the loc_3B9B0
            // transition visual; native replaces the source with those live slots.
            active.push(first_child);
            active.push(second_child);
            active.push(falling_visual);
            continue;
        }

        active.push(source);
    }
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
            // Check player collision — triggers orbit-to-walk transition.
            // ASM loc_3B904: collision_type negative/$2C/$1C all trigger drop.
            let player_contact = if let (Some(px), Some(py)) = (player_x_pos, player_y_pos) {
                let ex = (enemy.x_pos >> 16) as i32;
                let ey = (enemy.y_pos >> 16) as i32;
                let px_i = (px >> 16) as i32;
                let py_i = (py >> 16) as i32;
                spinning_twins_orbit_player_point_contact_loc_1bce(ex, ey, px_i, py_i)
            } else {
                false
            };

            let collision_type = if player_contact {
                SPINNING_TWINS_COLID_KIDABOVE as i16
            } else {
                0
            };
            if matches!(
                spinning_twins_orbit_collision_route_loc_3b904(collision_type),
                SpinningTwinsOrbitCollisionRouteLoc3B904::DropSplit
            ) {
                let _transition = spinning_twins_drop_transition_loc_3b93c(enemy);
                return;
            }

            let orbit_tick = spinning_twins_orbit_tick_loc_3b904(
                enemy.wait_timer,
                enemy.spinning_twins_orbit_index,
            );
            enemy.wait_timer = orbit_tick.next_animation_timer;
            enemy.spinning_twins_orbit_index = orbit_tick.next_orbit_index;
            enemy.x_velocity = 0;
        }
        SpinningTwinsPhase::Walking => {
            // Apply gravity while airborne until grounded, then walk as pacer
            if enemy.on_ground {
                step_pacer(enemy, block_grid, None);
                enemy.grounded_frames = enemy.grounded_frames.wrapping_add(1);
            } else {
                enemy.x_pos = (enemy.x_pos as i64 + i64::from(enemy.x_velocity)) as u32;
                enemy.y_pos = (enemy.y_pos as i64 + i64::from(enemy.y_velocity)) as u32;
                enemy.y_velocity =
                    spinning_twins_walking_child_y_velocity_loc_3bb2c(enemy.y_velocity);
                enemy.x_velocity = if enemy.x_velocity.is_negative() {
                    spinning_twins_walking_child_left_x_velocity_loc_3bb42(enemy.x_velocity)
                } else {
                    spinning_twins_walking_child_right_x_velocity_loc_3bb52(enemy.x_velocity)
                };
                if matches!(
                    spinning_twins_walking_velocity_gate_loc_3bb5a(enemy.x_velocity),
                    SpinningTwinsWalkingVelocityGateLoc3BB5A::Settle
                ) {
                    let _settle = spinning_twins_settle_entry_loc_3bb94(match enemy.facing {
                        Direction::Left => 0,
                        Direction::Right => 0xFF,
                    });
                    enemy.x_velocity = 0;
                    enemy.y_velocity = 0;
                    enemy.on_ground = true;
                    enemy.grounded_frames = 0;
                    return;
                }
                // Check floor
                if let Some(grid) = block_grid {
                    let x_px = (enemy.x_pos >> 16) as i32;
                    let y_px = (enemy.y_pos >> 16) as i32;
                    let probe = terrain::probe_floor(grid, x_px, y_px);
                    let collision_type = if probe.on_ground {
                        SPINNING_TWINS_COLID_FLOOR
                    } else {
                        0
                    };
                    let dispatch =
                        spinning_twins_walking_collision_dispatch_loc_3bb6e(collision_type);
                    debug_assert_eq!(dispatch.next_collision_type, 0);
                    match dispatch.route {
                        SpinningTwinsWalkingCollisionRouteLoc3BB6E::FloorLanding => {
                            let landing = spinning_twins_floor_landing_loc_3bb8a(
                                collision_type,
                                enemy.y_velocity,
                            );
                            debug_assert_eq!(landing.next_collision_type, 0);
                            enemy.y_pos = (probe.floor_y_px as u32) << 16;
                            enemy.y_velocity = landing.y_velocity;
                            // ASM Spinning_Twins.asm loc_3BB8A branches back
                            // to loc_3BB2C after clearing y_vel. The launch X
                            // drag continues until loc_3BB5A reaches loc_3BB94.
                        }
                        SpinningTwinsWalkingCollisionRouteLoc3BB6E::WallSettle
                        | SpinningTwinsWalkingCollisionRouteLoc3BB6E::ContinueAirborne => {}
                    }
                }
            }
        }
        SpinningTwinsPhase::FallingVisual => {
            let state = SpinningTwinsFallingVisualStateLoc3BA00 {
                x_pos: enemy.x_pos,
                y_pos: enemy.y_pos,
                step_index: enemy.spinning_twins_orbit_index,
                frames_until_next_step_index: enemy.grounded_frames,
                reverse_x_motion: enemy.shared_walker_patrol_short_next,
            };
            let stepped = spinning_twins_falling_visual_step_loc_3ba00(state, 0, 0);
            enemy.x_pos = stepped.state.x_pos;
            enemy.y_pos = stepped.state.y_pos;
            enemy.spinning_twins_orbit_index = stepped.state.step_index;
            enemy.grounded_frames = stepped.state.frames_until_next_step_index;
            enemy.shared_walker_patrol_short_next = stepped.state.reverse_x_motion;
            if !stepped.alive {
                enemy.hit_points = 0;
            }
        }
    }
}
