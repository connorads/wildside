use super::*;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum MiniHoppingSkullAirSprite {
    Rising,
    Falling,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum MiniHoppingSkullCollisionDispatch {
    AirborneLoop,
    WallBounce,
    FloorLanding,
    CeilingHit,
    HurtOrWeapon,
    SlopeOrOtherTerrain,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum MiniHoppingSkullTerrainContactRoute {
    NotSlopeUp,
    SlopeUpNonPositiveXVelocity,
    SlopeUpPositiveXVelocity,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct MiniHoppingSkullTerrainContact {
    pub(crate) is_moved: bool,
    pub(crate) is_animated: bool,
    pub(crate) has_level_collision: bool,
    pub(crate) route: MiniHoppingSkullTerrainContactRoute,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum MiniHoppingSkullSlopeDownWait {
    Left,
    Right,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
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
pub(crate) enum MiniHoppingSkullGroundWaitSprite {
    SlopeUpRight,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct MiniHoppingSkullGroundWaitSetup {
    pub(crate) x_velocity: i32,
    pub(crate) y_velocity: i32,
    pub(crate) x_direction: u8,
    pub(crate) sprite: MiniHoppingSkullGroundWaitSprite,
    pub(crate) dbf_counter: u16,
    pub(crate) wait_frames: u8,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
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
pub(crate) struct MiniHoppingSkullFacingMatchedPauseSetup {
    pub(crate) sprite: &'static str,
    pub(crate) dbf_counter: u16,
    pub(crate) wait_frames: u8,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum MiniHoppingSkullFacingMatchedPauseTail {
    ContinueWait {
        next_dbf_counter: u16,
    },
    BeginRelaunchArc {
        y_velocity: i32,
        x_velocity: i32,
        sprite: &'static str,
        is_moved: bool,
        has_level_collision: bool,
        next_loop: &'static str,
    },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
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
pub(crate) enum MiniHoppingSkullGroundWaitCollisionRoute {
    ContinueWait,
    DamageSkull,
    QueueScreamWorker,
    BerzerkerChargeKillsSkull,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum MiniHoppingSkullDamageSequence {
    SkullDestroyed,
    PlayHitAnimation {
        remaining_hits: u16,
        animation: &'static str,
        hibernate_until_finished: bool,
    },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct MiniHoppingSkullDestroyedSequence {
    pub(crate) is_moved: bool,
    pub(crate) animation: &'static str,
    pub(crate) hibernate_until_finished: bool,
    pub(crate) next_loop: &'static str,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum MiniHoppingSkullLoc3F158Route {
    RestartStoppedWait,
    DestroySkull(MiniHoppingSkullDestroyedSequence),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
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
pub(crate) struct MiniHoppingSkullScreamWorkerAllocation {
    pub(crate) allocation_selector: u16,
    pub(crate) routine: &'static str,
    pub(crate) delay_frames: u8,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
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

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct MiniHoppingSkullAirDamageSetup {
    pub(crate) remaining_hits: u16,
    pub(crate) y_velocity: i32,
    pub(crate) preserved_x_velocity: i32,
    pub(crate) x_velocity: i32,
    pub(crate) collision_type: u16,
    pub(crate) next_loop: &'static str,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum MiniHoppingSkullAirDamageLoopRoute {
    ContinueWait,
    FloorLanding,
    SlopeLanding,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct MiniHoppingSkullAirDamageLoopFrame {
    pub(crate) y_velocity: i32,
    pub(crate) collision_type: u16,
    pub(crate) hibernates_one_frame: bool,
    pub(crate) route: MiniHoppingSkullAirDamageLoopRoute,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum MiniHoppingSkullAirDamageSlopeLanding {
    ResumeTerrainContact {
        x_velocity: i32,
        next_loop: &'static str,
    },
    DestroySkull(MiniHoppingSkullDestroyedSequence),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum MiniHoppingSkullAirDamageDeathAnimation {
    Forward,
    BackwardOrNeutral,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct MiniHoppingSkullChargingBerzerkerLoopFrame {
    pub(crate) y_velocity: i32,
    pub(crate) hibernates_one_frame: bool,
    pub(crate) animation_tick_subroutine: &'static str,
    pub(crate) next_loop: &'static str,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct MiniHoppingSkullDeleteTailLoc3F32E {
    pub(crate) next_number_of_enemy: u16,
    pub(crate) status_action: DeactivationStatusActionLoc3C4C0,
    pub(crate) deletes_current_object: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum MiniHoppingSkullMediumLoadGateLoc3F304 {
    Return,
    DeleteTail(MiniHoppingSkullDeleteTailLoc3F32E),
}

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
                let allocation = mini_hopping_skull_left_hop_scream_worker_allocation_loc_3edc0();
                debug_assert_eq!(allocation.allocation_selector, 0xA000);
                debug_assert_eq!(allocation.routine, "sub_3F3F0");
                debug_assert_eq!(allocation.delay_frames, 15);
                MiniHoppingSkullGroundWaitCollisionRoute::QueueScreamWorker
            }
        }
        MINI_SKULL_COLID_GROUND_KIDBELOW => {
            let allocation = mini_hopping_skull_left_hop_scream_worker_allocation_loc_3edc0();
            debug_assert_eq!(allocation.allocation_selector, 0xA000);
            debug_assert_eq!(allocation.routine, "sub_3F3F0");
            debug_assert_eq!(allocation.delay_frames, 15);
            MiniHoppingSkullGroundWaitCollisionRoute::ContinueWait
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

pub(crate) fn mini_hopping_skull_landing_tail_loc_3ee7c(
    dbf_counter: u16,
    is_boosted: bool,
) -> MiniHoppingSkullLandingLoopDecision {
    // ASM Mini_Hopping_Skull.asm loc_3EE7C: DBCC loop first (`dbf d1`). On fallthrough,
    // seed airborne relaunch velocities and set movement/collision flags, then jump to
    // `loc_3EA88`.
    let route = mini_hopping_skull_landing_collision_route_loc_3ee12(0, false);
    debug_assert_eq!(
        route,
        MiniHoppingSkullGroundWaitCollisionRoute::ContinueWait
    );

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

pub(crate) fn mini_hopping_skull_right_hop_damage_sequence_loc_3ebe4(
    hit_counter: u16,
) -> MiniHoppingSkullDamageSequence {
    // ASM Mini_Hopping_Skull.asm loc_3EBE4: the right-hop windup damage path
    // repeats loc_3EB60's word decrement of `$44(a3)`; zero branches to
    // loc_3F176, otherwise `stru_3E9C2` plays before the wait loop resumes.
    mini_hopping_skull_damage_sequence_loc_3eb60(hit_counter)
}

pub(crate) fn mini_hopping_skull_left_hop_damage_sequence_loc_3ed1e(
    hit_counter: u16,
) -> MiniHoppingSkullDamageSequence {
    // ASM Mini_Hopping_Skull.asm loc_3ED1E: the left-hop windup damage path
    // repeats the same `$44(a3)` word decrement and `stru_3E9C2` replay as
    // loc_3EDE0 before this caller resumes through the windup DBF tail.
    mini_hopping_skull_damage_sequence_loc_3ede0(hit_counter)
}

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

    match mini_hopping_skull_landing_collision_route_loc_3eec8(collision_type, berzerker_charging) {
        MiniHoppingSkullGroundWaitCollisionRoute::DamageSkull => {
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
        MiniHoppingSkullGroundWaitCollisionRoute::BerzerkerChargeKillsSkull => {
            MiniHoppingSkullLandingTailCollisionOutcome::BerzerkerChargeKillsSkull
        }
        MiniHoppingSkullGroundWaitCollisionRoute::QueueScreamWorker => {
            let allocation = mini_hopping_skull_landing_scream_worker_allocation_loc_3eefa(
                MiniHoppingSkullGroundWaitCollisionRoute::QueueScreamWorker,
            )
            .expect("loc_3EEFA allocation route should spawn a worker");
            debug_assert_eq!(allocation.allocation_selector, 0xA000);
            debug_assert_eq!(allocation.routine, "sub_3F3F0");
            debug_assert_eq!(allocation.delay_frames, 15);
            MiniHoppingSkullLandingTailCollisionOutcome::SpawnScreamWorkerAndRelaunch
        }
        MiniHoppingSkullGroundWaitCollisionRoute::ContinueWait => {
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

pub(crate) fn mini_hopping_skull_stopped_wait_damage_sequence_loc_3f05c(
    hit_counter: u16,
) -> MiniHoppingSkullDamageSequence {
    // ASM Mini_Hopping_Skull.asm loc_3F05C: stopped-wait damage subtracts
    // one from `$44(a3)` as a word, branches to the death path on zero, otherwise
    // plays `stru_3E9C2` and hibernates until that animation finishes.
    mini_hopping_skull_damage_sequence_loc_3eb60(hit_counter)
}

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
            mini_hopping_skull_facing_matched_pause_side_contact_loc_3f0f0(berzerker_charging)
        }
        MINI_SKULL_COLID_GROUND_KIDBELOW => {
            MiniHoppingSkullGroundWaitCollisionRoute::QueueScreamWorker
        }
        _ => MiniHoppingSkullGroundWaitCollisionRoute::ContinueWait,
    }
}

pub(crate) fn mini_hopping_skull_facing_matched_pause_side_contact_loc_3f0f0(
    berzerker_charging: bool,
) -> MiniHoppingSkullGroundWaitCollisionRoute {
    // ASM Mini_Hopping_Skull.asm loc_3F0F0: side contact during the
    // matched-facing pause tests `Berzerker_charging`; clear falls through
    // to loc_3F0DA's scream worker, set jumps to loc_3F262.
    if berzerker_charging {
        MiniHoppingSkullGroundWaitCollisionRoute::BerzerkerChargeKillsSkull
    } else {
        MiniHoppingSkullGroundWaitCollisionRoute::QueueScreamWorker
    }
}

pub(crate) fn mini_hopping_skull_facing_matched_pause_damage_sequence_loc_3f0fa(
    hit_counter: u16,
) -> MiniHoppingSkullDamageSequence {
    // ASM Mini_Hopping_Skull.asm loc_3F0FA: matched-facing pause damage
    // subtracts one from `$44(a3)` as a word; zero branches to the death path,
    // otherwise `stru_3E9C2` plays and hibernates until finished.
    mini_hopping_skull_damage_sequence_loc_3eb60(hit_counter)
}

pub(crate) fn mini_hopping_skull_death_or_stopped_wait_loc_3f158(
    hit_counter: u16,
) -> MiniHoppingSkullLoc3F158Route {
    // ASM Mini_Hopping_Skull.asm loc_3F158: a non-zero `$44(a3)` falls back to
    // the stopped-wait path; zero clears movement, plays `stru_3E9AC`,
    // hibernates until finished, then continues to the deletion path.
    if hit_counter != 0 {
        return MiniHoppingSkullLoc3F158Route::RestartStoppedWait;
    }

    MiniHoppingSkullLoc3F158Route::DestroySkull(MiniHoppingSkullDestroyedSequence {
        is_moved: false,
        animation: "stru_3E9AC",
        hibernate_until_finished: true,
        next_loop: "delete_current_object",
    })
}

pub(crate) fn mini_hopping_skull_air_damage_loop_loc_3f200(
    y_velocity: i32,
    collision_type: u16,
) -> MiniHoppingSkullAirDamageLoopFrame {
    // ASM Mini_Hopping_Skull.asm loc_3F200: hibernate one frame, add `$4000`
    // to `y_vel(a3)`, read `collision_type(a3)` into d4, clear any non-zero
    // collision word, then dispatch only floor and slope contacts.
    let route = match collision_type {
        MINI_SKULL_COLID_FLOOR => MiniHoppingSkullAirDamageLoopRoute::FloorLanding,
        MINI_SKULL_COLID_SLOPEUP | MINI_SKULL_COLID_SLOPEDOWN => {
            MiniHoppingSkullAirDamageLoopRoute::SlopeLanding
        }
        _ => MiniHoppingSkullAirDamageLoopRoute::ContinueWait,
    };

    MiniHoppingSkullAirDamageLoopFrame {
        y_velocity: y_velocity.wrapping_add(0x0000_4000),
        collision_type: 0,
        hibernates_one_frame: true,
        route,
    }
}

pub(crate) fn mini_hopping_skull_air_damage_slope_landing_loc_3f22a(
    remaining_hits: u16,
    preserved_x_velocity: i32,
) -> MiniHoppingSkullAirDamageSlopeLanding {
    // ASM Mini_Hopping_Skull.asm loc_3F22A: if `$44(a3)` is still non-zero,
    // restore saved d2 to `x_vel(a3)` and resume terrain contact. With zero HP,
    // `swap d2; tst.w d2` tests the signed original high word of x_vel.
    if remaining_hits != 0 {
        return MiniHoppingSkullAirDamageSlopeLanding::ResumeTerrainContact {
            x_velocity: preserved_x_velocity,
            next_loop: "terrain_contact",
        };
    }

    let animation =
        match mini_hopping_skull_air_damage_death_animation_loc_3f238(preserved_x_velocity) {
            MiniHoppingSkullAirDamageDeathAnimation::Forward => "stru_3E998",
            MiniHoppingSkullAirDamageDeathAnimation::BackwardOrNeutral => "stru_3E9A2",
        };

    MiniHoppingSkullAirDamageSlopeLanding::DestroySkull(MiniHoppingSkullDestroyedSequence {
        is_moved: false,
        animation,
        hibernate_until_finished: true,
        next_loop: "delete_current_object",
    })
}

pub(crate) fn mini_hopping_skull_air_damage_death_animation_loc_3f238(
    preserved_x_velocity: i32,
) -> MiniHoppingSkullAirDamageDeathAnimation {
    // ASM Mini_Hopping_Skull.asm loc_3F238: `swap d2; tst.w d2; ble.w
    // loc_3F18C; bra.w loc_3F176` tests only the saved X velocity high word.
    let saved_x_integer_word = (preserved_x_velocity >> 16) as i16;
    if saved_x_integer_word > 0 {
        MiniHoppingSkullAirDamageDeathAnimation::Forward
    } else {
        MiniHoppingSkullAirDamageDeathAnimation::BackwardOrNeutral
    }
}

pub(crate) fn mini_hopping_skull_air_side_contact_loc_3f244(
    berzerker_charging: bool,
) -> MiniHoppingSkullGroundWaitCollisionRoute {
    // ASM Mini_Hopping_Skull.asm loc_3F244: airborne `$20/$24` side contact
    // tests `Berzerker_charging`; clear allocates `sub_3F3F0`, set jumps to
    // loc_3F262's charging-Berzerker kill setup.
    if berzerker_charging {
        MiniHoppingSkullGroundWaitCollisionRoute::BerzerkerChargeKillsSkull
    } else {
        MiniHoppingSkullGroundWaitCollisionRoute::QueueScreamWorker
    }
}

pub(crate) fn mini_hopping_skull_charging_berzerker_loop_loc_3f288(
    y_velocity: i32,
) -> MiniHoppingSkullChargingBerzerkerLoopFrame {
    // ASM Mini_Hopping_Skull.asm loc_3F288: hibernate one frame, run
    // `sub_3E8BA`, add `$4000` to `y_vel(a3)`, then branch back to loc_3F288.
    MiniHoppingSkullChargingBerzerkerLoopFrame {
        y_velocity: y_velocity.wrapping_add(0x0000_4000),
        hibernates_one_frame: true,
        animation_tick_subroutine: "sub_3E8BA",
        next_loop: "loc_3F288",
    }
}

pub(crate) fn mini_hopping_skull_cull_reject_loc_3f300(
    number_of_enemy: u16,
    object_status_slot: u8,
) -> MiniHoppingSkullDeleteTailLoc3F32E {
    // ASM Mini_Hopping_Skull.asm loc_3F300: `bra.w loc_3F32E`.
    mini_hopping_skull_delete_tail_loc_3f32e(number_of_enemy, object_status_slot)
}

pub(crate) fn mini_hopping_skull_medium_load_camera_gate_loc_3f304(
    x_pos_word: u16,
    y_pos_word: u16,
    camera_x_word: u16,
    camera_y_word: u16,
    number_of_enemy: u16,
    object_status_slot: u8,
) -> MiniHoppingSkullMediumLoadGateLoc3F304 {
    // ASM Mini_Hopping_Skull.asm loc_3F304: subtract camera words, keep only
    // signed deltas inside x[-$1A4,+$2E4] and y[-$1A4,+$284]; every failed
    // comparison branches back to loc_3F300.
    let relative_x = x_pos_word.wrapping_sub(camera_x_word) as i16;
    let relative_y = y_pos_word.wrapping_sub(camera_y_word) as i16;

    if !(-0x01A4..=0x02E4).contains(&relative_x) || !(-0x01A4..=0x0284).contains(&relative_y) {
        return MiniHoppingSkullMediumLoadGateLoc3F304::DeleteTail(
            mini_hopping_skull_cull_reject_loc_3f300(number_of_enemy, object_status_slot),
        );
    }

    MiniHoppingSkullMediumLoadGateLoc3F304::Return
}

pub(crate) fn mini_hopping_skull_delete_tail_loc_3f32e(
    number_of_enemy: u16,
    object_status_slot: u8,
) -> MiniHoppingSkullDeleteTailLoc3F32E {
    // ASM Mini_Hopping_Skull.asm loc_3F32E: decrement `Number_of_Enemy`, read
    // `$42(a5)`, update `EnemyStatus_Table` for positive slots or bit-6 spawn
    // counters, then converge on `j_Delete_CurrentObject`.
    // ASM Mini_Hopping_Skull.asm loc_3F354: positive `$42(a5)` doubles the
    // slot byte as a word offset and writes `$2168` into `EnemyStatus_Table`.
    // ASM Mini_Hopping_Skull.asm loc_3F362: all status routes converge on the
    // direct `jmp (j_Delete_CurrentObject).w` tail.
    MiniHoppingSkullDeleteTailLoc3F32E {
        next_number_of_enemy: number_of_enemy.wrapping_sub(1),
        status_action: deactivation_status_update_loc_3C4C0(object_status_slot),
        deletes_current_object: true,
    }
}

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

pub(crate) fn mini_hopping_skull_facing_matched_pause_tail_loc_3f112(
    dbf_counter: u16,
    x_direction: u8,
    is_boosted: bool,
) -> MiniHoppingSkullFacingMatchedPauseTail {
    // ASM Mini_Hopping_Skull.asm loc_3F112: DBF returns to the matched-facing
    // pause loop while active. On fall-through, it seeds the relaunch arc,
    // applies the HP-tier boost, negates X when `x_direction(a3)` is clear,
    // selects `LnkTo_unk_C8080`, and reenables movement/collision.
    let next_dbf_counter = dbf_counter.wrapping_sub(1);
    if next_dbf_counter != 0xFFFF {
        return MiniHoppingSkullFacingMatchedPauseTail::ContinueWait { next_dbf_counter };
    }

    let mut y_velocity = 0xFFFC8000u32 as i32;
    let mut x_velocity = 0x0001_8000u32 as i32;
    if is_boosted {
        y_velocity = y_velocity.wrapping_add(-0x1_0000);
        x_velocity = x_velocity.wrapping_add(0x4000);
    }
    // ASM Mini_Hopping_Skull.asm loc_3F13C: after the optional HP boost,
    // a clear `x_direction(a3)` falls through to `neg.l x_vel(a3)`.
    if x_direction == 0 {
        x_velocity = x_velocity.wrapping_neg();
    }

    // ASM Mini_Hopping_Skull.asm loc_3F146: select `LnkTo_unk_C8080`,
    // set movement and level-collision flags, then branch back to loc_3EA88.
    MiniHoppingSkullFacingMatchedPauseTail::BeginRelaunchArc {
        y_velocity,
        x_velocity,
        sprite: "LnkTo_unk_C8080",
        is_moved: true,
        has_level_collision: true,
        next_loop: "loc_3EA88",
    }
}

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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::enemies::test_fixtures::*;

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
    fn mini_hopping_skull_slopeup_nonpositive_x_enters_left_wait_loc_3ead8_3ec32() {
        let mut grid = terrain::BlockGrid::from_level_layout(&[0; 8 * 4 * 2], 8, 4);
        grid.set_cell_at(2, 2, terrain::Cell::from_be_bytes(0x40, 0x00));
        let mut enemy = ActiveEnemy {
            on_ground: false,
            facing: Direction::Right,
            x_pos: 41 << 16,
            y_pos: 38 << 16,
            x_velocity: -0x0001_0000,
            y_velocity: 0x0002_0000,
            ..mini_hopping_skull_test_enemy()
        };

        step_hopper(&mut enemy, Some(&grid), None);

        assert_eq!(
            enemy.facing,
            Direction::Left,
            "ASM Mini_Hopping_Skull.asm loc_3EAD8 -> loc_3EC32 clears x_direction for slope-up contact with non-positive x_vel"
        );
        assert_eq!(enemy.x_velocity, 0);
        assert_eq!(enemy.y_velocity, 0);
        assert_eq!(enemy.grounded_frames, MINI_SKULL_GROUND_DELAY_FRAMES);
        assert!(enemy.on_ground);
    }

    #[test]
    fn mini_hopping_skull_slopedown_negative_x_enters_left_wait_loc_3ed6c() {
        let mut grid = terrain::BlockGrid::from_level_layout(&[0; 8 * 4 * 2], 8, 4);
        grid.set_cell_at(2, 2, terrain::Cell::from_be_bytes(0x50, 0x00));
        let mut enemy = ActiveEnemy {
            on_ground: false,
            facing: Direction::Right,
            x_pos: 41 << 16,
            y_pos: 38 << 16,
            x_velocity: -0x0001_0000,
            y_velocity: 0x0002_0000,
            ..mini_hopping_skull_test_enemy()
        };

        step_hopper(&mut enemy, Some(&grid), None);

        assert_eq!(
            enemy.facing,
            Direction::Left,
            "ASM Mini_Hopping_Skull.asm loc_3ED6C clears x_direction for slope-down contact with negative x_vel"
        );
        assert_eq!(enemy.x_velocity, 0);
        assert_eq!(enemy.y_velocity, 0);
        assert_eq!(enemy.grounded_frames, MINI_SKULL_GROUND_DELAY_FRAMES);
        assert!(enemy.on_ground);
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
    fn mini_hopping_skull_facing_matched_pause_side_contact_matches_loc_3f0f0() {
        assert_eq!(
            mini_hopping_skull_facing_matched_pause_side_contact_loc_3f0f0(false),
            MiniHoppingSkullGroundWaitCollisionRoute::QueueScreamWorker,
            "ASM Mini_Hopping_Skull.asm loc_3F0F0 should branch to loc_3F0DA when `Berzerker_charging` is clear"
        );
        assert_eq!(
            mini_hopping_skull_facing_matched_pause_side_contact_loc_3f0f0(true),
            MiniHoppingSkullGroundWaitCollisionRoute::BerzerkerChargeKillsSkull,
            "ASM Mini_Hopping_Skull.asm loc_3F0F0 should jump to loc_3F262 while `Berzerker_charging` is set"
        );
    }

    #[test]
    fn mini_hopping_skull_facing_matched_pause_damage_sequence_matches_loc_3f0fa() {
        assert_eq!(
            mini_hopping_skull_facing_matched_pause_damage_sequence_loc_3f0fa(1),
            MiniHoppingSkullDamageSequence::SkullDestroyed,
            "ASM Mini_Hopping_Skull.asm loc_3F0FA should branch to the death path when `$44(a3)` reaches zero"
        );
        assert_eq!(
            mini_hopping_skull_facing_matched_pause_damage_sequence_loc_3f0fa(2),
            MiniHoppingSkullDamageSequence::PlayHitAnimation {
                remaining_hits: 1,
                animation: "stru_3E9C2",
                hibernate_until_finished: true,
            },
            "ASM Mini_Hopping_Skull.asm loc_3F0FA should replay `stru_3E9C2` and hibernate when hits remain"
        );
        assert_eq!(
            mini_hopping_skull_facing_matched_pause_damage_sequence_loc_3f0fa(0),
            MiniHoppingSkullDamageSequence::PlayHitAnimation {
                remaining_hits: 0xFFFF,
                animation: "stru_3E9C2",
                hibernate_until_finished: true,
            },
            "ASM Mini_Hopping_Skull.asm loc_3F0FA uses word wrapping for the hit-counter decrement"
        );
    }

    #[test]
    fn mini_hopping_skull_death_or_stopped_wait_matches_loc_3f158() {
        assert_eq!(
            mini_hopping_skull_death_or_stopped_wait_loc_3f158(1),
            MiniHoppingSkullLoc3F158Route::RestartStoppedWait,
            "ASM Mini_Hopping_Skull.asm loc_3F158 should branch back to the stopped-wait path while `$44(a3)` is non-zero"
        );
        assert_eq!(
            mini_hopping_skull_death_or_stopped_wait_loc_3f158(0),
            MiniHoppingSkullLoc3F158Route::DestroySkull(MiniHoppingSkullDestroyedSequence {
                is_moved: false,
                animation: "stru_3E9AC",
                hibernate_until_finished: true,
                next_loop: "delete_current_object",
            }),
            "ASM Mini_Hopping_Skull.asm loc_3F158 should clear movement, play the death animation, and continue to deletion when `$44(a3)` is zero"
        );
    }

    #[test]
    fn mini_hopping_skull_air_damage_setup_matches_loc_3f1e6() {
        assert_eq!(
            mini_hopping_skull_air_damage_setup_loc_3f1e6(2, 0x0001_C000),
            MiniHoppingSkullAirDamageSetup {
                remaining_hits: 1,
                y_velocity: 0x0002_8800,
                preserved_x_velocity: 0x0001_C000,
                x_velocity: 0,
                collision_type: 0,
                next_loop: "air_damage_landing_loop",
            },
            "ASM Mini_Hopping_Skull.asm loc_3F1E6 decrements `$44(a3)`, stores live `x_vel(a3)` in d2, clears live X/collision, and seeds y_vel with `$28800`"
        );
        assert_eq!(
            mini_hopping_skull_air_damage_setup_loc_3f1e6(0, -0x0001_8000).remaining_hits,
            0xFFFF,
            "ASM Mini_Hopping_Skull.asm loc_3F1E6 uses word wrapping for `subi.w #1,$44(a3)`"
        );
    }

    #[test]
    fn mini_hopping_skull_air_damage_loop_matches_loc_3f200() {
        assert_eq!(
            mini_hopping_skull_air_damage_loop_loc_3f200(0x0002_8800, 0),
            MiniHoppingSkullAirDamageLoopFrame {
                y_velocity: 0x0002_C800,
                collision_type: 0,
                hibernates_one_frame: true,
                route: MiniHoppingSkullAirDamageLoopRoute::ContinueWait,
            },
            "ASM Mini_Hopping_Skull.asm loc_3F200 hibernates one frame, adds `$4000` to y_vel(a3), and loops while collision_type is zero"
        );
        assert_eq!(
            mini_hopping_skull_air_damage_loop_loc_3f200(0x0002_C800, MINI_SKULL_COLID_FLOOR),
            MiniHoppingSkullAirDamageLoopFrame {
                y_velocity: 0x0003_0800,
                collision_type: 0,
                hibernates_one_frame: true,
                route: MiniHoppingSkullAirDamageLoopRoute::FloorLanding,
            },
            "ASM Mini_Hopping_Skull.asm loc_3F200 routes floor contact after clearing collision_type(a3)"
        );
        assert_eq!(
            mini_hopping_skull_air_damage_loop_loc_3f200(0x0002_C800, MINI_SKULL_COLID_SLOPEUP)
                .route,
            MiniHoppingSkullAirDamageLoopRoute::SlopeLanding,
            "ASM Mini_Hopping_Skull.asm loc_3F200 treats slope-up as a slope landing"
        );
        assert_eq!(
            mini_hopping_skull_air_damage_loop_loc_3f200(0x0002_C800, MINI_SKULL_COLID_SLOPEDOWN)
                .route,
            MiniHoppingSkullAirDamageLoopRoute::SlopeLanding,
            "ASM Mini_Hopping_Skull.asm loc_3F200 treats slope-down as the same slope landing path"
        );
        assert_eq!(
            mini_hopping_skull_air_damage_loop_loc_3f200(0x0002_C800, MINI_SKULL_COLID_CEILING),
            MiniHoppingSkullAirDamageLoopFrame {
                y_velocity: 0x0003_0800,
                collision_type: 0,
                hibernates_one_frame: true,
                route: MiniHoppingSkullAirDamageLoopRoute::ContinueWait,
            },
            "ASM Mini_Hopping_Skull.asm loc_3F200 clears unhandled non-zero collision IDs before looping"
        );
    }

    #[test]
    fn mini_hopping_skull_air_damage_slope_landing_matches_loc_3f22a() {
        assert_eq!(
            mini_hopping_skull_air_damage_slope_landing_loc_3f22a(1, -0x0001_8000),
            MiniHoppingSkullAirDamageSlopeLanding::ResumeTerrainContact {
                x_velocity: -0x0001_8000,
                next_loop: "terrain_contact",
            },
            "ASM Mini_Hopping_Skull.asm loc_3F22A restores saved d2 into x_vel(a3) when the decremented hit counter is non-zero"
        );
        assert_eq!(
            mini_hopping_skull_air_damage_slope_landing_loc_3f22a(0, 0x0001_C000),
            MiniHoppingSkullAirDamageSlopeLanding::DestroySkull(
                MiniHoppingSkullDestroyedSequence {
                    is_moved: false,
                    animation: "stru_3E998",
                    hibernate_until_finished: true,
                    next_loop: "delete_current_object",
                }
            ),
            "ASM Mini_Hopping_Skull.asm loc_3F22A zero-hit continuation uses the positive signed high word of saved X velocity for the forward death animation"
        );
        assert_eq!(
            mini_hopping_skull_air_damage_slope_landing_loc_3f22a(0, 0x0000_FFFF),
            MiniHoppingSkullAirDamageSlopeLanding::DestroySkull(
                MiniHoppingSkullDestroyedSequence {
                    is_moved: false,
                    animation: "stru_3E9A2",
                    hibernate_until_finished: true,
                    next_loop: "delete_current_object",
                }
            ),
            "ASM Mini_Hopping_Skull.asm loc_3F22A tests the swapped saved X high word, so fractional-only positive X still takes the non-positive death animation"
        );
        assert_eq!(
            mini_hopping_skull_air_damage_slope_landing_loc_3f22a(0, -0x0001_8000),
            MiniHoppingSkullAirDamageSlopeLanding::DestroySkull(
                MiniHoppingSkullDestroyedSequence {
                    is_moved: false,
                    animation: "stru_3E9A2",
                    hibernate_until_finished: true,
                    next_loop: "delete_current_object",
                }
            ),
            "ASM Mini_Hopping_Skull.asm loc_3F22A treats a negative signed high word as the non-positive death animation"
        );
    }

    #[test]
    fn mini_hopping_skull_air_damage_death_animation_matches_loc_3f238() {
        assert_eq!(
            mini_hopping_skull_air_damage_death_animation_loc_3f238(0x0001_C000),
            MiniHoppingSkullAirDamageDeathAnimation::Forward,
            "ASM Mini_Hopping_Skull.asm loc_3F238 swaps saved d2 and routes a positive high word to the forward death animation"
        );
        assert_eq!(
            mini_hopping_skull_air_damage_death_animation_loc_3f238(0x0000_FFFF),
            MiniHoppingSkullAirDamageDeathAnimation::BackwardOrNeutral,
            "ASM Mini_Hopping_Skull.asm loc_3F238 ignores positive fractional-only velocity after `swap d2`"
        );
        assert_eq!(
            mini_hopping_skull_air_damage_death_animation_loc_3f238(-0x0001_8000),
            MiniHoppingSkullAirDamageDeathAnimation::BackwardOrNeutral,
            "ASM Mini_Hopping_Skull.asm loc_3F238 sends negative high words through the `ble.w loc_3F18C` branch"
        );
    }

    #[test]
    fn mini_hopping_skull_air_side_contact_matches_loc_3f244_berzerker_gate() {
        assert_eq!(
            mini_hopping_skull_air_side_contact_loc_3f244(false),
            MiniHoppingSkullGroundWaitCollisionRoute::QueueScreamWorker,
            "ASM Mini_Hopping_Skull.asm loc_3F244 should allocate `sub_3F3F0` while `Berzerker_charging` is clear"
        );
        assert_eq!(
            mini_hopping_skull_air_side_contact_loc_3f244(true),
            MiniHoppingSkullGroundWaitCollisionRoute::BerzerkerChargeKillsSkull,
            "ASM Mini_Hopping_Skull.asm loc_3F244 should jump to loc_3F262 while `Berzerker_charging` is set"
        );
    }

    #[test]
    fn mini_hopping_skull_charging_berzerker_loop_matches_loc_3f288() {
        assert_eq!(
            mini_hopping_skull_charging_berzerker_loop_loc_3f288(-0x0004_8000),
            MiniHoppingSkullChargingBerzerkerLoopFrame {
                y_velocity: -0x0004_4000,
                hibernates_one_frame: true,
                animation_tick_subroutine: "sub_3E8BA",
                next_loop: "loc_3F288",
            },
            "ASM Mini_Hopping_Skull.asm loc_3F288 should hibernate, run `sub_3E8BA`, add `$4000` gravity, and loop"
        );
        assert_eq!(
            mini_hopping_skull_charging_berzerker_loop_loc_3f288(0x7FFF_F000).y_velocity,
            i32::MIN + 0x3000,
            "ASM Mini_Hopping_Skull.asm loc_3F288 uses 32-bit wrapping addition for `addi.l #$4000,y_vel(a3)`"
        );
    }

    #[test]
    fn mini_hopping_skull_delete_tail_matches_loc_3f32e_status_update() {
        assert_eq!(
            mini_hopping_skull_delete_tail_loc_3f32e(3, 0xC2),
            MiniHoppingSkullDeleteTailLoc3F32E {
                next_number_of_enemy: 2,
                status_action: DeactivationStatusActionLoc3C4C0::SubtractSpawnCounter { index: 2 },
                deletes_current_object: true,
            },
            "ASM Mini_Hopping_Skull.asm loc_3F32E decrements Number_of_Enemy, subtracts `$400` when `$42(a5)` has bits 7+6 set, then jumps to j_Delete_CurrentObject"
        );
    }

    #[test]
    fn mini_hopping_skull_delete_tail_positive_slot_matches_loc_3f354() {
        assert_eq!(
            mini_hopping_skull_delete_tail_loc_3f32e(4, 0x05),
            MiniHoppingSkullDeleteTailLoc3F32E {
                next_number_of_enemy: 3,
                status_action: DeactivationStatusActionLoc3C4C0::WriteRespawnMarker {
                    index: 5,
                    word_value: 0x2168
                },
                deletes_current_object: true,
            },
            "ASM Mini_Hopping_Skull.asm loc_3F354 doubles positive `$42(a5)` into an EnemyStatus_Table word offset and writes `$2168`"
        );
    }

    #[test]
    fn mini_hopping_skull_delete_tail_direct_delete_matches_loc_3f362() {
        assert_eq!(
            mini_hopping_skull_delete_tail_loc_3f32e(1, 0x80),
            MiniHoppingSkullDeleteTailLoc3F32E {
                next_number_of_enemy: 0,
                status_action: DeactivationStatusActionLoc3C4C0::DeleteOnly,
                deletes_current_object: true,
            },
            "ASM Mini_Hopping_Skull.asm loc_3F362 is the direct `j_Delete_CurrentObject` tail reached after the bit-7 set, bit-6 clear `$42(a5)` route"
        );
    }

    #[test]
    fn mini_hopping_skull_cull_reject_matches_loc_3f300_trampoline() {
        assert_eq!(
            mini_hopping_skull_cull_reject_loc_3f300(9, 0x05),
            MiniHoppingSkullDeleteTailLoc3F32E {
                next_number_of_enemy: 8,
                status_action: DeactivationStatusActionLoc3C4C0::WriteRespawnMarker {
                    index: 5,
                    word_value: 0x2168
                },
                deletes_current_object: true,
            },
            "ASM Mini_Hopping_Skull.asm loc_3F300 is only `bra.w loc_3F32E`, so every rejected culling check must take the delete-tail status update"
        );
    }

    #[test]
    fn mini_hopping_skull_medium_load_gate_matches_loc_3f304_bounds() {
        assert_eq!(
            mini_hopping_skull_medium_load_camera_gate_loc_3f304(0xFE5C, 0x0284, 0, 0, 4, 0x81),
            MiniHoppingSkullMediumLoadGateLoc3F304::Return,
            "ASM Mini_Hopping_Skull.asm loc_3F304 keeps the exact inclusive medium-load bounds x[-$1A4,+$2E4], y[-$1A4,+$284]"
        );
        assert_eq!(
            mini_hopping_skull_medium_load_camera_gate_loc_3f304(0xFE5B, 0x0100, 0, 0, 4, 0x81),
            MiniHoppingSkullMediumLoadGateLoc3F304::DeleteTail(
                MiniHoppingSkullDeleteTailLoc3F32E {
                    next_number_of_enemy: 3,
                    status_action: DeactivationStatusActionLoc3C4C0::DeleteOnly,
                    deletes_current_object: true,
                }
            ),
            "ASM Mini_Hopping_Skull.asm loc_3F304 branches to loc_3F300 when signed screen X is less than -$1A4"
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
    fn mini_hopping_skull_facing_matched_pause_tail_matches_loc_3f112() {
        assert_eq!(
            mini_hopping_skull_facing_matched_pause_tail_loc_3f112(0x000A, 0xFF, false),
            MiniHoppingSkullFacingMatchedPauseTail::ContinueWait {
                next_dbf_counter: 0x0009,
            },
            "ASM Mini_Hopping_Skull.asm loc_3F112 should consume the DBF counter before relaunching"
        );
        assert_eq!(
            mini_hopping_skull_facing_matched_pause_tail_loc_3f112(0x0000, 0x00, false),
            MiniHoppingSkullFacingMatchedPauseTail::BeginRelaunchArc {
                y_velocity: -0x0003_8000,
                x_velocity: -0x0001_8000,
                sprite: "LnkTo_unk_C8080",
                is_moved: true,
                has_level_collision: true,
                next_loop: "loc_3EA88",
            },
            "ASM Mini_Hopping_Skull.asm loc_3F112 should negate base X velocity when `x_direction(a3)` is clear"
        );
        assert_eq!(
            mini_hopping_skull_facing_matched_pause_tail_loc_3f112(0x0000, 0xFF, true),
            MiniHoppingSkullFacingMatchedPauseTail::BeginRelaunchArc {
                y_velocity: -0x0004_8000,
                x_velocity: 0x0001_C000,
                sprite: "LnkTo_unk_C8080",
                is_moved: true,
                has_level_collision: true,
                next_loop: "loc_3EA88",
            },
            "ASM Mini_Hopping_Skull.asm loc_3F112 should apply the HP-tier boost before the direction gate"
        );
    }

    #[test]
    fn mini_hopping_skull_facing_matched_pause_direction_gate_matches_loc_3f13c() {
        assert_eq!(
            mini_hopping_skull_facing_matched_pause_tail_loc_3f112(0x0000, 0x00, true),
            MiniHoppingSkullFacingMatchedPauseTail::BeginRelaunchArc {
                y_velocity: -0x0004_8000,
                x_velocity: -0x0001_C000,
                sprite: "LnkTo_unk_C8080",
                is_moved: true,
                has_level_collision: true,
                next_loop: "loc_3EA88",
            },
            "ASM Mini_Hopping_Skull.asm loc_3F13C should negate the boosted X velocity when `x_direction(a3)` is clear"
        );
        assert_eq!(
            mini_hopping_skull_facing_matched_pause_tail_loc_3f112(0x0000, 0xFF, false),
            MiniHoppingSkullFacingMatchedPauseTail::BeginRelaunchArc {
                y_velocity: -0x0003_8000,
                x_velocity: 0x0001_8000,
                sprite: "LnkTo_unk_C8080",
                is_moved: true,
                has_level_collision: true,
                next_loop: "loc_3EA88",
            },
            "ASM Mini_Hopping_Skull.asm loc_3F13C should skip `neg.l x_vel(a3)` when `x_direction(a3)` is non-zero"
        );
    }

    #[test]
    fn mini_hopping_skull_facing_matched_pause_relaunch_footer_matches_loc_3f146() {
        assert_eq!(
            mini_hopping_skull_facing_matched_pause_tail_loc_3f112(0x0000, 0xFF, true),
            MiniHoppingSkullFacingMatchedPauseTail::BeginRelaunchArc {
                y_velocity: -0x0004_8000,
                x_velocity: 0x0001_C000,
                sprite: "LnkTo_unk_C8080",
                is_moved: true,
                has_level_collision: true,
                next_loop: "loc_3EA88",
            },
            "ASM Mini_Hopping_Skull.asm loc_3F146 should restore the airborne sprite, movement flag, collision flag, and loc_3EA88 loop"
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
    fn mini_hopping_skull_ceiling_bounce_negates_y_velocity_loc_3f1a2() {
        let data = vec![0u8; 8 * 4 * 2];
        let mut grid = terrain::BlockGrid::from_level_layout(&data, 8, 4);
        grid.set_cell_at(2, 1, terrain::Cell::from_be_bytes(0x60, 0x00));

        let mut enemy = mini_hopping_skull_test_enemy();
        enemy.on_ground = false;
        enemy.x_pos = 40 << 16;
        enemy.y_pos = 34 << 16;
        enemy.y_velocity = -0x0002_0000;

        step_hopper(&mut enemy, Some(&grid), None);

        assert_eq!(
            enemy.y_velocity, 0x0001_EE00,
            "ASM Mini_Hopping_Skull.asm loc_3EA88 applies `$1200` gravity, then loc_3F1A2 negates y_vel on ceiling contact"
        );
        assert!(!enemy.on_ground);
    }

    #[test]
    fn mini_hopping_skull_ground_wait_does_not_edge_turnaround_loc_3f000() {
        let data = vec![0u8; 8 * 4 * 2];
        let mut grid = terrain::BlockGrid::from_level_layout(&data, 8, 4);
        grid.set_cell_at(1, 2, terrain::Cell::from_be_bytes(0x60, 0x00));

        let mut enemy = ActiveEnemy {
            x_pos: 24 << 16,
            y_pos: 32 << 16,
            facing: Direction::Right,
            grounded_frames: 5,
            tank_mouth_animation_phase: 0,
            ..mini_hopping_skull_test_enemy()
        };

        step_hopper(&mut enemy, Some(&grid), None);

        assert_eq!(
            enemy.facing,
            Direction::Right,
            "ASM Mini_Hopping_Skull.asm loc_3F000/loc_3F090 waits then compares Kid X; it has no generic platform-edge lookahead turnaround"
        );
        assert_eq!(enemy.grounded_frames, 4);
    }
}
