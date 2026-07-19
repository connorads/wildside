use crate::{
    ModeLevelAirborneActionPath, ModeLevelAirborneHorizontalPath, ModeLevelAirborneVerticalPath,
    ModeLevelCeilingScanReturn, ModeLevelDamageFlashWaitLocBc44, ModeLevelDemoExitWaitLoc119E0,
    ModeLevelExecuteObjectsWork, ModeLevelFloatingPrizeFallPathLoc103B2,
    ModeLevelInputWorkerPathLoc7452, ModeLevelKidGfxTerrainCollision, ModeLevelPlayerExecutionPath,
    ModeLevelPrizeAnimationWake, ModeLevelPrizeBlockTailState, ModeLevelPrizeBlockWobbleWake,
    ModeLevelPrizeObjectExecutionSubE12, ModeLevelPrizeTailListWork,
    ModeLevelRubberContactEffectLoc10DA4, ModeLevelVerticalCollisionPath,
    ModeLevelWallCollisionPath,
};

/// Raw Motorola 68000 instruction cycles, excluding refresh and bus waits.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct M68kCycleCount(u32);

impl M68kCycleCount {
    pub const fn get(self) -> u32 {
        self.0
    }
}

/// Genesis 68000 memory-refresh phase, modulo the 128-cycle refresh interval.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct M68kRefreshPhase(u8);

impl M68kRefreshPhase {
    pub const fn new(value: u8) -> Option<Self> {
        if value < 128 { Some(Self(value)) } else { None }
    }

    pub const fn get(self) -> u8 {
        self.0
    }

    /// Advance raw ROM execution, inserting the two-cycle refresh waits used
    /// by the Genesis timing model when the carried phase crosses 128.
    pub fn advance_rom(self, raw: M68kCycleCount) -> M68kRefreshAdvance {
        let phase = u64::from(self.0);
        let raw = u64::from(raw.0);
        let mut refresh_events = (phase + raw) / 128;
        loop {
            let next = (phase + raw + 2 * refresh_events) / 128;
            if next == refresh_events {
                break;
            }
            refresh_events = next;
        }
        let elapsed = raw + 2 * refresh_events;
        M68kRefreshAdvance {
            elapsed: M68kCycleCount(
                u32::try_from(elapsed).expect("68000 timing span exceeds u32 cycles"),
            ),
            phase_after: Self(
                u8::try_from((phase + elapsed) % 128).expect("refresh phase remains below 128"),
            ),
            refresh_events: u32::try_from(refresh_events).expect("68000 refresh count exceeds u32"),
        }
    }

    /// Advance an already-pending external wait. The wait consumes phase but
    /// cannot itself enqueue another refresh wait.
    pub fn advance_external_wait(self, cycles: M68kCycleCount) -> M68kRefreshAdvance {
        let elapsed = cycles.0;
        M68kRefreshAdvance {
            elapsed: cycles,
            phase_after: Self(
                u8::try_from((u32::from(self.0) + elapsed) % 128)
                    .expect("refresh phase remains below 128"),
            ),
            refresh_events: 0,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct M68kRefreshAdvance {
    pub elapsed: M68kCycleCount,
    pub phase_after: M68kRefreshPhase,
    pub refresh_events: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct M68kExternalWaitEvent {
    /// Raw CPU offset from ASM `sub_E12` entry.
    pub raw_offset: M68kCycleCount,
    pub cycles: M68kCycleCount,
}

/// Raw cycles from the finished-animation wake at ASM loc_1066 through the next
/// `Execute_Objects` loc_E1E dispatch.
pub fn prize_tail_finish_cycles_sub_1007a(work: ModeLevelPrizeTailListWork) -> M68kCycleCount {
    // ASM sub_1007A (kid.asm:21407-21435) creates the flash and prize through
    // sub_E02, then Delete_CurrentObject walks the Object and GfxObject lists.
    // Allocation uses 54 raw cycles per visited node. The three deletion
    // walks use distinct source loops: 52 at loc_EA2, 48 at loc_10BA, and 52
    // at loc_1106. The rank-independent path costs 5,208 raw cycles.
    let cycles = 5_208
        + 54 * u32::from(work.flash_object_insert_rank_loc_de0)
        + 54 * u32::from(work.flash_gfx_insert_rank_loc_fd4)
        + 54 * u32::from(work.prize_object_insert_rank_loc_de0)
        + 54 * u32::from(work.prize_gfx_insert_rank_loc_fd4)
        + 52 * u32::from(work.tail_object_delete_rank_loc_ea2)
        + 48 * u32::from(work.tail_gfx_delete_rank_loc_10ba)
        + 52 * u32::from(work.tail_gfx_category4_delete_rank_loc_1106);
    M68kCycleCount(cycles)
}

/// Raw cycles from ASM `Execute_Objects sub_E12` through the first loc_E1E list
/// read, including all four `Get_RandomNumber_byte sub_9FE` calls.
pub fn execute_objects_setup_cycles_sub_e12(rng_offset_before: u16) -> Option<M68kCycleCount> {
    if !crate::RomRandomFrameState::offset_is_initialised(rng_offset_before) {
        return None;
    }
    // The no-wrap sub_A4A path costs 1,088 raw cycles. In sub_9FE, resetting
    // offset -1 to -55 adds two cycles; wrapping the +$1F tap adds six.
    let mut cycles = 1_088;
    let mut offset = rng_offset_before as i16;
    for _ in 0..4 {
        offset += 1;
        if offset == 0 {
            offset = -55;
            cycles += 2;
        }
        if offset >= -31 {
            cycles += 6;
        }
    }
    Some(M68kCycleCount(cycles))
}

/// Raw cycles from the ordinary airborne Kid resume at ASM loc_A73A
/// through its `Hibernate_Object_1Frame sub_E76` entry.
///
/// This intentionally accepts the whole Execute_Objects ledger because the
/// Gfx terrain scan and same-pass prize allocation are sibling observations
/// of the player path. Untyped sibling branches return `None`.
pub fn player_callback_cycles_loc_a73a(
    execute: &ModeLevelExecuteObjectsWork,
) -> Option<M68kCycleCount> {
    let ModeLevelPlayerExecutionPath::NormalPhysics(path) =
        execute.executed_player_path.as_ref()?
    else {
        return None;
    };
    if !path.gfx_object_collision_clear
        || !matches!(
            path.vertical,
            ModeLevelAirborneVerticalPath::CycloneFlyingNoCPress { .. }
        )
        || !matches!(path.wall_collision, ModeLevelWallCollisionPath::Clear)
        || !matches!(path.airborne_action, ModeLevelAirborneActionPath::NoButtons)
    {
        return None;
    }

    // ASM loc_A73A, sub_B084, sub_B168, sub_A4EE, sub_902A/sub_8F26,
    // sub_B270, and loc_A8D6 share this fixed natural-route spine.
    let mut cycles = 3_830_i32;
    cycles += match path.horizontal {
        ModeLevelAirborneHorizontalPath::NeutralNegativeBrake => 0,
        ModeLevelAirborneHorizontalPath::RightHeldAcceleration => -12,
        ModeLevelAirborneHorizontalPath::Unclassified { .. } => return None,
    };

    let gfx_scan = execute.player_gfx_terrain_scan?;
    cycles += 81 * i32::from(gfx_scan.tile_iterations);
    cycles += match gfx_scan.first_collision {
        None => 0,
        Some(ModeLevelKidGfxTerrainCollision::Solid) => 191,
        Some(
            ModeLevelKidGfxTerrainCollision::SlopeUp | ModeLevelKidGfxTerrainCollision::SlopeDown,
        ) => return None,
    };
    cycles += 10
        * i32::try_from(
            path.cyclone_visual_steps
                .into_iter()
                .flatten()
                .filter(|step| step.wrapped)
                .count(),
        )
        .ok()?;
    if matches!(
        path.vertical,
        ModeLevelAirborneVerticalPath::CycloneFlyingNoCPress {
            capped_acceleration_to_0x1200: true,
            ..
        }
    ) {
        cycles += 10;
    }

    match (&path.vertical_collision, &path.ceiling_scan_sub_914a) {
        (ModeLevelVerticalCollisionPath::Clear, None) => {}
        (ModeLevelVerticalCollisionPath::Ceiling, Some(scan))
            if scan.visits.len() == 2
                && scan.return_path == ModeLevelCeilingScanReturn::Hit
                && scan.first_hit_col.is_some()
                && scan.visits.iter().any(|visit| {
                    Some(visit.col) == scan.first_hit_col
                        && visit.block_type == Some(1)
                        && visit.rubber_effect == ModeLevelRubberContactEffectLoc10DA4::None
                }) =>
        {
            let list = execute.player_prize_wobble_list_work_loc_1002e?;
            cycles += 4_366
                + 54 * i32::from(list.object_insert_rank_loc_de0 + list.gfx_insert_rank_loc_fd4);
        }
        _ => return None,
    }

    Some(M68kCycleCount(u32::try_from(cycles).ok()?))
}

/// Nominal E1E-to-E1E cycles for the fixed damage-flash callback.
pub fn damage_flash_wait_cycles_loc_bc44(
    path: ModeLevelDamageFlashWaitLocBc44,
) -> Option<M68kCycleCount> {
    match path {
        // ASM loc_BC44..loc_BC40 (kid.asm:15187-15194): the runnable dispatch,
        // hibernate, both clear tests, and back edge total 462 raw CPU cycles.
        ModeLevelDamageFlashWaitLocBc44::AwaitDamage => Some(M68kCycleCount(462)),
        ModeLevelDamageFlashWaitLocBc44::DeathFreeze
        | ModeLevelDamageFlashWaitLocBc44::BeginDamageFlash => None,
    }
}

/// Nominal E1E-to-E1E cycles for the fixed demo-exit callback.
pub fn demo_exit_wait_cycles_loc_119e0(
    path: ModeLevelDemoExitWaitLoc119E0,
) -> Option<M68kCycleCount> {
    match path {
        // ASM loc_119D8..loc_119E0 (kid.asm:23595-23600): the runnable
        // dispatch, hibernate, and clear Start test total 450 raw CPU cycles.
        ModeLevelDemoExitWaitLoc119E0::HibernateNoStart => Some(M68kCycleCount(450)),
        ModeLevelDemoExitWaitLoc119E0::ExitDemoStart => None,
    }
}

/// Raw E1E-to-E1E cycles for the fixed input/pause callback.
pub fn input_worker_cycles_loc_7452(path: ModeLevelInputWorkerPathLoc7452) -> M68kCycleCount {
    match path {
        // ASM loc_7452..loc_74B0 (kid.asm:7586-7625): the clear timer, idle
        // controller, demo-mode, and clear Start path totals 636 raw cycles.
        ModeLevelInputWorkerPathLoc7452::DemoIdleNoInputTimerClear => M68kCycleCount(636),
    }
}

/// Raw 68000 cycles for one typed prize callback, measured from its
/// `Execute_Objects` loc_E1E list read through the following loc_E1E read.
///
/// `None` preserves branch uncertainty. VDP bus stalls are deliberately
/// excluded and belong to the raster scheduler.
pub fn prize_object_cycles_sub_e12(
    execute: &ModeLevelExecuteObjectsWork,
    callback: ModeLevelPrizeObjectExecutionSubE12,
) -> Option<M68kCycleCount> {
    // ASM loc_E1E -> loc_E3E -> callback costs 224 cycles on the runnable
    // path. A one-frame hibernate at sub_E76 costs another 158 raw cycles before
    // the next loc_E1E.
    const RUNNABLE_HIBERNATE_SHELL_CYCLES: u32 = 380;
    const RUNNABLE_DISPATCH_CYCLES: u32 = 222;

    let callback_cycles = match callback {
        ModeLevelPrizeObjectExecutionSubE12::FloatingPrize { index } => {
            let work = execute.floating_prize_paths.get(usize::from(index))?;
            match work.fall_path_loc_103b2? {
                // ASM loc_103B2 -> loc_103DE -> loc_10404 -> loc_10396.
                ModeLevelFloatingPrizeFallPathLoc103B2::DirectFallSameRow => 232,
                // The lower-cell Level_Layout probe adds 78 cycles.
                ModeLevelFloatingPrizeFallPathLoc103B2::GraceProbeFallSameRow => 308,
                // ASM loc_103DA skips gravity and rejoins the bottom/collision
                // checks at loc_10404.
                ModeLevelFloatingPrizeFallPathLoc103B2::BlockedCurrentRow => 190,
                // Row-advance and blocked-grace costs need independent source
                // probes before they can enter the cycle scheduler.
                ModeLevelFloatingPrizeFallPathLoc103B2::DirectFallAdvanceRow
                | ModeLevelFloatingPrizeFallPathLoc103B2::GraceProbeFallAdvanceRow
                | ModeLevelFloatingPrizeFallPathLoc103B2::BlockedGraceProbe => return None,
            }
        }
        ModeLevelPrizeObjectExecutionSubE12::Player => return None,
        ModeLevelPrizeObjectExecutionSubE12::PlayerSpawnedInitialSub1007A => {
            let work = execute.player_spawned_prize_block_initial_sub_1007a?;
            match work.path {
                // ASM sub_1007A through the first loc_100DA hibernate.
                crate::ModeLevelPrizeBlockInitialPathSub1007A::Prize => 464,
                crate::ModeLevelPrizeBlockInitialPathSub1007A::RevealOnlyLoc101D8 => return None,
            }
        }
        ModeLevelPrizeObjectExecutionSubE12::FlashLoc102B8 { index } => {
            let work = execute
                .prize_block_flash_paths_sub_1066
                .get(usize::from(index))?;
            match work.wake {
                // ASM loc_1066 -> loc_1062 -> sub_E76.
                ModeLevelPrizeAnimationWake::Hibernate => 50,
                ModeLevelPrizeAnimationWake::FinishAndDelete => return None,
            }
        }
        ModeLevelPrizeObjectExecutionSubE12::WobbleSub1007A { index } => {
            let work = execute
                .prize_block_wobble_paths_sub_1007a
                .get(usize::from(index))?;
            match work.wake {
                ModeLevelPrizeBlockWobbleWake::ContinuePhase => 54,
                ModeLevelPrizeBlockWobbleWake::BeginNextPhase => 62,
                ModeLevelPrizeBlockWobbleWake::BeginAnimationWait => 94,
                ModeLevelPrizeBlockWobbleWake::Unclassified { .. } => return None,
            }
        }
        ModeLevelPrizeObjectExecutionSubE12::TailSub1007A { index } => {
            let work = execute
                .prize_block_tail_paths_sub_1066
                .get(usize::from(index))?;
            match work.state {
                ModeLevelPrizeBlockTailState::FinishSpawnPrizeAndDelete => {
                    return Some(M68kCycleCount(
                        RUNNABLE_DISPATCH_CYCLES
                            + prize_tail_finish_cycles_sub_1007a(work.list_work?).get(),
                    ));
                }
                ModeLevelPrizeBlockTailState::DeletedWorkerOverlap { .. }
                | ModeLevelPrizeBlockTailState::UnclassifiedAnimationWait { .. } => return None,
            }
        }
    };
    Some(M68kCycleCount(
        RUNNABLE_HIBERNATE_SHELL_CYCLES + callback_cycles,
    ))
}

/// Complete raw CPU cost of the source-typed natural `Execute_Objects` pass.
///
/// Refresh, VDP, and Z80 waits are intentionally excluded. Any callback class
/// not yet represented by the ledger keeps the result unknown.
pub fn execute_objects_raw_cycles_sub_e12(
    execute: &ModeLevelExecuteObjectsWork,
) -> Option<M68kCycleCount> {
    if execute.linked_non_transient_object_count != 4
        || execute.rubber_block_skin_workers_sub_10de4 != 0
        || execute.active_enemy_workers != 0
        || execute.active_platform_workers != 0
        || execute.palette_rotation_worker_loc_11f96 != 0
        || execute.unclassified_runtime_transient_workers != 0
    {
        return None;
    }

    let mut cycles =
        execute_objects_setup_cycles_sub_e12(execute.rng_offset_before_sub_e12?)?.get();
    let mut player_seen = false;
    for callback in execute.prize_object_execution_order_sub_e12.as_ref()? {
        if *callback == ModeLevelPrizeObjectExecutionSubE12::Player {
            if player_seen {
                return None;
            }
            player_seen = true;
            // Raw loc_E1E -> loc_A73A is 268 cycles; sub_E76 -> next loc_E1E
            // is 158 cycles.
            cycles = cycles.checked_add(player_callback_cycles_loc_a73a(execute)?.get() + 426)?;
        } else {
            cycles = cycles.checked_add(prize_object_cycles_sub_e12(execute, *callback)?.get())?;
        }
    }
    if !player_seen {
        return None;
    }
    cycles = cycles
        .checked_add(input_worker_cycles_loc_7452(execute.input_worker_loc_7452?).get())?
        .checked_add(damage_flash_wait_cycles_loc_bc44(execute.damage_flash_wait_loc_bc44?)?.get())?
        .checked_add(demo_exit_wait_cycles_loc_119e0(execute.demo_exit_wait_loc_119e0?)?.get())?
        // ASM loc_E1E's final null-list return through sub_21F8 is 72 raw cycles.
        .checked_add(72)?;
    Some(M68kCycleCount(cycles))
}

/// Source-positioned non-refresh waits inside the typed Execute_Objects pass.
pub fn execute_objects_external_waits_sub_e12(
    execute: &ModeLevelExecuteObjectsWork,
) -> Option<Vec<M68kExternalWaitEvent>> {
    let Some(list) = execute.player_prize_wobble_list_work_loc_1002e else {
        return Some(Vec::new());
    };

    let mut raw_before_player =
        execute_objects_setup_cycles_sub_e12(execute.rng_offset_before_sub_e12?)?.get();
    let mut player_seen = false;
    for callback in execute.prize_object_execution_order_sub_e12.as_ref()? {
        if *callback == ModeLevelPrizeObjectExecutionSubE12::Player {
            player_seen = true;
            break;
        }
        raw_before_player = raw_before_player
            .checked_add(prize_object_cycles_sub_e12(execute, *callback)?.get())?;
    }
    if !player_seen {
        return None;
    }

    let ceiling_prize_cycles =
        4_366 + 54 * u32::from(list.object_insert_rank_loc_de0 + list.gfx_insert_rank_loc_fd4);
    let player_before_ceiling = player_callback_cycles_loc_a73a(execute)?
        .get()
        .checked_sub(ceiling_prize_cycles)?;
    // ASM E1E -> loc_A73A is 268 raw cycles. Within the ordinary ceiling-hit
    // branch, loc_1002E is 418 raw cycles after the non-ceiling player spine.
    let loc_1002e_offset = raw_before_player
        .checked_add(268)?
        .checked_add(player_before_ceiling)?
        .checked_add(418)?;

    // ASM loc_1002E -> PlaySound (kid.asm:21345-21368) performs six one-cycle
    // waits while the 68000 acquires and accesses the Z80 sound bus.
    [284, 344, 370, 390, 454, 474]
        .into_iter()
        .map(|offset| {
            Some(M68kExternalWaitEvent {
                raw_offset: M68kCycleCount(loc_1002e_offset.checked_add(offset)?),
                cycles: M68kCycleCount(1),
            })
        })
        .collect()
}

/// Execute the raw pass through the Genesis refresh recurrence and typed
/// external waits.
pub fn execute_objects_elapsed_cycles_sub_e12(
    execute: &ModeLevelExecuteObjectsWork,
    phase_before: M68kRefreshPhase,
) -> Option<M68kRefreshAdvance> {
    let raw_total = execute_objects_raw_cycles_sub_e12(execute)?.get();
    let events = execute_objects_external_waits_sub_e12(execute)?;
    let mut raw_cursor = 0;
    let mut phase = phase_before;
    let mut elapsed = 0_u32;
    let mut refresh_events = 0_u32;

    for event in events {
        let event_offset = event.raw_offset.get();
        let segment = event_offset.checked_sub(raw_cursor)?;
        let advance = phase.advance_rom(M68kCycleCount(segment));
        elapsed = elapsed.checked_add(advance.elapsed.get())?;
        refresh_events = refresh_events.checked_add(advance.refresh_events)?;
        phase = advance.phase_after;

        let wait = phase.advance_external_wait(event.cycles);
        elapsed = elapsed.checked_add(wait.elapsed.get())?;
        phase = wait.phase_after;
        raw_cursor = event_offset;
    }

    let tail = phase.advance_rom(M68kCycleCount(raw_total.checked_sub(raw_cursor)?));
    Some(M68kRefreshAdvance {
        elapsed: M68kCycleCount(elapsed.checked_add(tail.elapsed.get())?),
        phase_after: tail.phase_after,
        refresh_events: refresh_events.checked_add(tail.refresh_events)?,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn prize_tail_finish_cycles_follow_all_five_asm_list_walks_sub_1007a() {
        let frame_1697 = ModeLevelPrizeTailListWork {
            flash_object_insert_rank_loc_de0: 5,
            flash_gfx_insert_rank_loc_fd4: 15,
            prize_object_insert_rank_loc_de0: 1,
            prize_gfx_insert_rank_loc_fd4: 16,
            tail_object_delete_rank_loc_ea2: 14,
            tail_gfx_delete_rank_loc_10ba: 5,
            tail_gfx_category4_delete_rank_loc_1106: 11,
        };
        let frame_1703 = ModeLevelPrizeTailListWork {
            flash_object_insert_rank_loc_de0: 6,
            flash_gfx_insert_rank_loc_fd4: 16,
            prize_object_insert_rank_loc_de0: 1,
            prize_gfx_insert_rank_loc_fd4: 17,
            tail_object_delete_rank_loc_ea2: 15,
            tail_gfx_delete_rank_loc_10ba: 5,
            tail_gfx_category4_delete_rank_loc_1106: 12,
        };

        assert_eq!(
            prize_tail_finish_cycles_sub_1007a(frame_1697).get(),
            8_746,
            "bounded oracle: frame 1697 loc_1066 -> next loc_E1E"
        );
        assert_eq!(
            prize_tail_finish_cycles_sub_1007a(frame_1703).get(),
            9_012,
            "bounded oracle: frame 1703 loc_1066 -> next loc_E1E"
        );
    }

    #[test]
    fn execute_objects_rng_setup_cycles_follow_sub_9fe_wrap_branches() {
        assert_eq!(
            execute_objects_setup_cycles_sub_e12(0xFFCD)
                .expect("initialised -51 offset")
                .get(),
            1_088,
            "frame 1703 takes no tap-wrap branches"
        );
        assert_eq!(
            execute_objects_setup_cycles_sub_e12(0xFFEC)
                .expect("initialised -20 offset")
                .get(),
            1_112,
            "frame 1697 takes four +$1F tap-wrap branches"
        );
        assert_eq!(
            execute_objects_setup_cycles_sub_e12(0xFFE5)
                .expect("initialised -27 offset")
                .get(),
            1_112,
            "frame 1709 takes four +$1F tap-wrap branches"
        );
        assert_eq!(
            execute_objects_setup_cycles_sub_e12(0xFFFF)
                .expect("initialised -1 offset")
                .get(),
            1_090,
            "offset-ring reset costs two cycles and suppresses the tap wrap"
        );
        assert_eq!(execute_objects_setup_cycles_sub_e12(0), None);
    }

    #[test]
    fn fixed_tail_callbacks_follow_loc_bc44_and_loc_119e0() {
        assert_eq!(
            damage_flash_wait_cycles_loc_bc44(ModeLevelDamageFlashWaitLocBc44::AwaitDamage)
                .map(M68kCycleCount::get),
            Some(462)
        );
        assert_eq!(
            demo_exit_wait_cycles_loc_119e0(ModeLevelDemoExitWaitLoc119E0::HibernateNoStart)
                .map(M68kCycleCount::get),
            Some(450)
        );
        assert_eq!(
            damage_flash_wait_cycles_loc_bc44(ModeLevelDamageFlashWaitLocBc44::BeginDamageFlash),
            None
        );
        assert_eq!(
            demo_exit_wait_cycles_loc_119e0(ModeLevelDemoExitWaitLoc119E0::ExitDemoStart),
            None
        );
    }

    #[test]
    fn refresh_phase_reproduces_execute_objects_waits_without_vdp_contention() {
        let phase = M68kRefreshPhase::new(88).expect("valid phase");
        for (raw, refresh_cycles, phase_after) in
            [(27_002, 428, 126), (22_114, 352, 26), (13_814, 220, 42)]
        {
            let advance = phase.advance_rom(M68kCycleCount(raw));
            assert_eq!(advance.elapsed.get(), raw + refresh_cycles);
            assert_eq!(advance.refresh_events * 2, refresh_cycles);
            assert_eq!(advance.phase_after.get(), phase_after);
        }
        assert_eq!(
            phase
                .advance_rom(M68kCycleCount(27_002))
                .phase_after
                .advance_external_wait(M68kCycleCount(6))
                .phase_after
                .get(),
            4,
            "frame 1697 also carries six PlaySound BUSACK wait cycles"
        );
    }

    #[test]
    fn external_wait_advances_phase_without_recursively_enqueuing_refresh() {
        let advance = M68kRefreshPhase::new(127)
            .expect("valid phase")
            .advance_external_wait(M68kCycleCount(6));
        assert_eq!(advance.elapsed.get(), 6);
        assert_eq!(advance.refresh_events, 0);
        assert_eq!(advance.phase_after.get(), 5);
    }
}
