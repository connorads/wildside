use crate::{
    HudDmaEventSub44DC, ModeLevelAirborneActionPath, ModeLevelAirborneHorizontalPath,
    ModeLevelAirborneVerticalPath, ModeLevelAirborneVisualMappingPathSubB270,
    ModeLevelBlockDebrisWorkSub2A4C, ModeLevelCeilingScanReturn, ModeLevelDamageFlashWaitLocBc44,
    ModeLevelDemoExitWaitLoc119E0, ModeLevelDiamondPowerCompileWorkSub3F798,
    ModeLevelEnemyLoadingPath, ModeLevelExecuteObjectsWork, ModeLevelFloatingPrizeFallPathLoc103B2,
    ModeLevelGfxCollisionKidListWorkSub1B76, ModeLevelGfxCollisionKidNodeWorkSub1B76,
    ModeLevelGfxCollisionListWork, ModeLevelGfxCollisionWork,
    ModeLevelGfxMoveAnimateNodeWorkSub1444, ModeLevelGfxMoveAnimateWorkSub1444,
    ModeLevelGfxSpriteNodeWorkSub1492, ModeLevelGfxSpriteWorkSub1492,
    ModeLevelInputWorkerPathLoc7452, ModeLevelKidGfxTerrainCollision,
    ModeLevelKidProjectileCollisionWorkSub1D76, ModeLevelPaletteFadeWorkerPathSub73D0,
    ModeLevelPlatformLoaderExistingSlotSub2728, ModeLevelPlatformLoaderRangeLoc2762,
    ModeLevelPlatformLoaderWorkLoc2744, ModeLevelPlatformWork, ModeLevelPlayerExecutionPath,
    ModeLevelPostHudPhase, ModeLevelPostHudWorkLedger, ModeLevelPreHudWorkLedger,
    ModeLevelPrizeAnimationWake, ModeLevelPrizeBlockTailState, ModeLevelPrizeBlockWobbleWake,
    ModeLevelPrizeFlashDeleteListWork, ModeLevelPrizeObjectExecutionSubE12,
    ModeLevelPrizeTailListWork, ModeLevelReadJoypadWork, ModeLevelRubberContactEffectLoc10DA4,
    ModeLevelShooterCollisionWork, ModeLevelTerrainSpecialScanSub1FA2,
    ModeLevelVerticalCollisionPath, ModeLevelWallCollisionPath,
};
use serde::{Deserialize, Serialize};

/// Raw Motorola 68000 instruction cycles, excluding refresh and bus waits.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct M68kCycleCount(u32);

impl M68kCycleCount {
    pub const fn from_raw(cycles: u32) -> Self {
        Self(cycles)
    }

    pub const fn get(self) -> u32 {
        self.0
    }
}

/// Hardware ownership already pending when ASM `loc_7D6` completes the
/// `loc_30104` H-scroll DMA call.
///
/// This delay is serviced at the next source-instruction boundary. It is not
/// ROM source work and is not a budget for waits generated later by GEMS.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ModeLevelPendingHardwareOwnershipAfterLoc7d6(M68kCycleCount);

impl ModeLevelPendingHardwareOwnershipAfterLoc7d6 {
    pub const NONE: Self = Self(M68kCycleCount::from_raw(0));

    pub const fn from_raw_cycles(cycles: u32) -> Self {
        Self(M68kCycleCount::from_raw(cycles))
    }

    pub const fn elapsed(self) -> M68kCycleCount {
        self.0
    }
}

/// Mega Drive master-clock cycles.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GenesisMclkCount(u32);

impl GenesisMclkCount {
    pub const fn from_raw(cycles: u32) -> Self {
        Self(cycles)
    }

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

/// Live object-list work consumed before ASM `sub_8E0` blocks in
/// `WaitForVint`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TransformRasterWorkSub8E0 {
    pub murder_wall_active_sub_5e02: bool,
    pub gfx_sprite_work_sub_1492: ModeLevelGfxSpriteWorkSub1492,
    pub platform_work_sub_2444: ModeLevelPlatformWork,
}

/// Classified ASM `Execute_Objects sub_E12` path whose Kid callback enters
/// `Kid_Transform` after collecting a Cyclone helmet prize.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CyclonePickupTransformEntrySubE12;

pub fn cyclone_pickup_transform_entry_sub_e12(
    execute: &ModeLevelExecuteObjectsWork,
) -> Option<CyclonePickupTransformEntrySubE12> {
    let [prize] = execute.floating_prize_paths.as_slice() else {
        return None;
    };
    if execute.rng_offset_before_sub_e12 != Some(0xFFDF)
        || execute.linked_object_count != 5
        || execute.linked_gfx_object_count != 5
        || execute.linked_non_transient_object_count != 4
        || execute.floating_prize_workers != 1
        || execute.rubber_block_skin_workers_sub_10de4 != 0
        || execute.active_enemy_workers != 0
        || execute.active_platform_workers != 0
        || execute.palette_rotation_worker_loc_11f96 != 0
        || execute.unclassified_runtime_transient_workers != 0
        || !matches!(
            execute.prize_object_execution_order_sub_e12.as_deref(),
            Some([
                ModeLevelPrizeObjectExecutionSubE12::FloatingPrize { index: 0 },
                ModeLevelPrizeObjectExecutionSubE12::Player,
            ])
        )
        || !matches!(
            execute.executed_player_path,
            Some(ModeLevelPlayerExecutionPath::Unclassified)
        )
        || execute.input_worker_loc_7452
            != Some(ModeLevelInputWorkerPathLoc7452::DemoNoLeftNoStartTimerClear)
        || execute.damage_flash_wait_loc_bc44 != Some(ModeLevelDamageFlashWaitLocBc44::AwaitDamage)
        || execute.demo_exit_wait_loc_119e0 != Some(ModeLevelDemoExitWaitLoc119E0::HibernateNoStart)
        || execute.palette_fade_worker_sub_73d0.is_some()
        || prize.content != crate::terrain::PrizeContent::Cyclone
        || prize.remaining != 765
        || prize.spawn_delay != 0
        || prize.first_row_grace
        || !prize.collision_latched
        || prize.fall_path_loc_103b2
            != Some(ModeLevelFloatingPrizeFallPathLoc103B2::DirectFallSameRow)
    {
        return None;
    }
    Some(CyclonePickupTransformEntrySubE12)
}

/// One source phase from ASM `Mode_Level loc_810 -> loc_892`, including
/// asynchronous hardware waits generated while that phase executes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ModeLevelPostHudPhaseAdvance {
    pub phase: ModeLevelPostHudPhase,
    pub phase_before: M68kRefreshPhase,
    pub raw_cycles: M68kCycleCount,
    pub elapsed: M68kCycleCount,
    pub refresh_events: u32,
    pub phase_after: M68kRefreshPhase,
    pub generated_hardware_wait_cycles: M68kCycleCount,
}

/// Source branch through ASM `Init_Timer_and_Bonus_Flags` during
/// `Load_InGame`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LoadInGameTimerBonusPath {
    /// `$FFFFFC36` is set, so the timer and bonus flags are initialised.
    Initialise,
    /// `$FFFFFC36` is clear, so the routine branches directly to its return.
    ReturnAlreadyInitialised,
}

/// Pure 68000 source work for one fixed platform-art upload in ASM
/// `Initialize_Platforms -> DecompressToVRAM`.
///
/// Hardware refresh, VINT entry/handler work, and VDP FIFO stalls are excluded
/// so the raster scheduler can position those effects independently.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LoadInGamePlatformArtSourceWorkload {
    pub decoder: M68kCycleCount,
    pub transfer: M68kCycleCount,
    pub output_words: u16,
}

impl LoadInGamePlatformArtSourceWorkload {
    pub const fn raw_cycles(self) -> M68kCycleCount {
        M68kCycleCount::from_raw(self.decoder.get() + self.transfer.get())
    }
}

/// Source-ordered workload before `Load_InGame $11A12`.
///
/// ASM `Initialize_Platforms sub_231C` initialises the 17-slot platform pool,
/// resolves the map-order platform layout, then uploads the fixed horizontal
/// and vertical platform-art resources through `DecompressToVRAM`. The two
/// compressed streams both emit 384 bytes, but take different decoder branches.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LoadInGamePlatformInitialisationSourceWorkload {
    pub prelude_through_first_decompress_entry: M68kCycleCount,
    pub horizontal_art: LoadInGamePlatformArtSourceWorkload,
    pub between_art_uploads: M68kCycleCount,
    pub vertical_art: LoadInGamePlatformArtSourceWorkload,
    pub epilogue: M68kCycleCount,
}

impl LoadInGamePlatformInitialisationSourceWorkload {
    pub const fn raw_cycles(self) -> M68kCycleCount {
        M68kCycleCount::from_raw(
            self.prelude_through_first_decompress_entry.get()
                + self.horizontal_art.raw_cycles().get()
                + self.between_art_uploads.get()
                + self.vertical_art.raw_cycles().get()
                + self.epilogue.get(),
        )
    }
}

/// Pure source workload from `Load_InGame $11A0E` to `$11A12`.
///
/// The decoder totals follow the real branch sequence through
/// `Decompress_Chunk sub_13684`; the common transfer total covers
/// `loc_14300..return_14334`, including all 192 `loc_14314` word writes.
/// External FIFO waits and the VINT crossed by the vertical decode are not
/// folded into these values.
pub const fn load_in_game_platform_initialisation_source_workload()
-> LoadInGamePlatformInitialisationSourceWorkload {
    let transfer = M68kCycleCount::from_raw(4_258);
    LoadInGamePlatformInitialisationSourceWorkload {
        // ASM Load_InGame $11A0E, j_Initialize_Platforms, sub_231C,
        // Get_PlatformLayoutAddress sub_2366, j_DecompressToVRAM, and the
        // first `bsr.s sub_142CC`.
        prelude_through_first_decompress_entry: M68kCycleCount::from_raw(910),
        horizontal_art: LoadInGamePlatformArtSourceWorkload {
            // ASM Decompress_Chunk sub_13684 over Horizontal_platform.bin.
            decoder: M68kCycleCount::from_raw(14_570),
            transfer,
            output_words: 192,
        },
        // ASM return_14334 -> Load_InGame's second j_DecompressToVRAM ->
        // `bsr.s sub_142CC`.
        between_art_uploads: M68kCycleCount::from_raw(64),
        vertical_art: LoadInGamePlatformArtSourceWorkload {
            // ASM Decompress_Chunk sub_13684 over Vertical_platform.bin.
            decoder: M68kCycleCount::from_raw(14_528),
            transfer,
            output_words: 192,
        },
        // ASM return_14334 -> Initialize_Platforms RTS.
        epilogue: M68kCycleCount::from_raw(16),
    }
}

/// Raw source work from the `j_sub_28FC` call at `Load_InGame $11A12` to
/// `StopMusic $E14C8`.
///
/// ASM `Load_InGame $11A12..$11A24` calls the fixed `sub_28FC` 123-longword
/// clear, `sub_44B0`, the selected `Init_Timer_and_Bonus_Flags` branch, and
/// finally the `j_StopMusic` thunk. Refresh and external hardware waits remain
/// scheduler work.
pub fn load_in_game_stop_music_tail_raw_cycles(
    timer_path: LoadInGameTimerBonusPath,
) -> M68kCycleCount {
    let mut raw_cycles = 0_u32;
    visit_load_in_game_stop_music_tail_instructions(timer_path, |instruction_cycles| {
        raw_cycles = raw_cycles
            .checked_add(instruction_cycles)
            .expect("Load_InGame tail fits u32 cycles");
    });
    M68kCycleCount::from_raw(raw_cycles)
}

/// Advance refresh scheduling across each real instruction in the
/// `Load_InGame $11A12..$11A24` tail.
///
/// Refresh is sampled between instructions, so advancing only the aggregate
/// raw total can miss a wait queued at an instruction boundary.
pub fn load_in_game_stop_music_tail_advance(
    mut phase: M68kRefreshPhase,
    timer_path: LoadInGameTimerBonusPath,
) -> M68kRefreshAdvance {
    let mut elapsed = 0_u32;
    let mut refresh_events = 0_u32;
    visit_load_in_game_stop_music_tail_instructions(timer_path, |instruction_cycles| {
        let advance = phase.advance_rom(M68kCycleCount::from_raw(instruction_cycles));
        elapsed = elapsed
            .checked_add(advance.elapsed.get())
            .expect("Load_InGame tail fits u32 cycles");
        refresh_events = refresh_events
            .checked_add(advance.refresh_events)
            .expect("Load_InGame refresh count fits u32");
        phase = advance.phase_after;
    });
    M68kRefreshAdvance {
        elapsed: M68kCycleCount::from_raw(elapsed),
        phase_after: phase,
        refresh_events,
    }
}

/// Co-schedule the `Load_InGame $11A12..$11A24` source tail with hardware.
///
/// ASM GEMS `XFER68K` can queue an 11-cycle 68000 wait after any completed
/// source instruction. The wait belongs to the next scheduler tick and may
/// itself let the Z80 generate another bank access, so this visitor drains
/// the chain before executing the following 68000 instruction.
pub fn load_in_game_stop_music_tail_advance_with_scheduler(
    mut phase: M68kRefreshPhase,
    timer_path: LoadInGameTimerBonusPath,
    mut advance_hardware: impl FnMut(M68kCycleCount) -> Option<M68kCycleCount>,
) -> Option<M68kRefreshAdvance> {
    let mut elapsed = 0_u32;
    let mut refresh_events = 0_u32;
    let mut valid = true;
    visit_load_in_game_stop_music_tail_instructions(timer_path, |instruction_cycles| {
        if !valid {
            return;
        }

        let phase_after_raw = u32::from(phase.get()) + instruction_cycles;
        let refresh_wait = if phase_after_raw >= 128 { 2 } else { 0 };
        phase = M68kRefreshPhase::new(
            u8::try_from(phase_after_raw % 128).expect("refresh phase remains below 128"),
        )
        .expect("refresh phase remains below 128");
        elapsed = if let Some(elapsed) = elapsed.checked_add(instruction_cycles) {
            elapsed
        } else {
            valid = false;
            return;
        };

        let Some(generated_wait) = advance_hardware(M68kCycleCount::from_raw(instruction_cycles))
        else {
            valid = false;
            return;
        };
        let Some(mut pending_wait) = generated_wait.get().checked_add(refresh_wait) else {
            valid = false;
            return;
        };
        if refresh_wait != 0 {
            refresh_events += 1;
        }
        while pending_wait != 0 {
            let wait = M68kCycleCount::from_raw(pending_wait);
            phase = phase.advance_external_wait(wait).phase_after;
            elapsed = if let Some(elapsed) = elapsed.checked_add(pending_wait) {
                elapsed
            } else {
                valid = false;
                return;
            };
            let Some(generated_wait) = advance_hardware(wait) else {
                valid = false;
                return;
            };
            pending_wait = generated_wait.get();
        }
    });
    valid.then_some(M68kRefreshAdvance {
        elapsed: M68kCycleCount::from_raw(elapsed),
        phase_after: phase,
        refresh_events,
    })
}

fn visit_load_in_game_stop_music_tail_instructions(
    timer_path: LoadInGameTimerBonusPath,
    mut visit: impl FnMut(u32),
) {
    // ASM Load_InGame $11A12 -> j_sub_28FC -> sub_28FC.
    for cycles in [18, 10, 8, 8, 4] {
        visit(cycles);
    }
    for iteration in 0..123 {
        visit(12); // move.l d6,(a0)+
        visit(if iteration == 122 { 14 } else { 10 }); // dbf d7,loc_2906
    }
    for cycles in [8, 12, 16, 16, 16] {
        visit(cycles);
    }

    // ASM Load_InGame $11A16 -> j_sub_44B0 -> sub_44B0.
    for cycles in [18, 10, 16, 16, 16, 16, 16, 16] {
        visit(cycles);
    }

    // ASM Load_InGame $11A1A -> Init_Timer_and_Bonus_Flags.
    for cycles in [18, 10, 12] {
        visit(cycles);
    }
    match timer_path {
        LoadInGameTimerBonusPath::Initialise => {
            visit(12); // beq.w not taken
            for _ in 0..8 {
                visit(16);
            }
        }
        LoadInGameTimerBonusPath::ReturnAlreadyInitialised => {
            visit(10); // beq.w taken
        }
    }
    visit(16); // rts

    // ASM Load_InGame $11A1E -> j_StopMusic -> StopMusic $E14C8.
    visit(20);
    visit(10);
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct M68kExternalWaitEvent {
    /// Raw CPU offset from ASM `sub_E12` entry.
    pub raw_offset: M68kCycleCount,
    pub cycles: M68kCycleCount,
}

/// One interval where ASM `StopMusic` has requested the Z80 bus.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GemsStopMusicBusWindow {
    pub request_complete_raw_offset: M68kCycleCount,
    pub release_complete_raw_offset: M68kCycleCount,
    pub m68k_z80_ram_accesses: u16,
}

/// Source branches at the seven Z80 bus-acquisition loops in ASM
/// `StopMusic -> sub_E13DA/sub_E149C/sub_E14AC` (kid.asm).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GemsStopMusicBusAckPolls {
    pub driver_copy_sub_e13da: u16,
    pub queue_transactions_sub_e143a: [u16; 6],
}

/// Source-ordered control schedule for ASM `StopMusic` (`$E14C8..$E150E`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GemsStopMusicTiming {
    pub raw_cycles: M68kCycleCount,
    pub bus_windows: [GemsStopMusicBusWindow; 7],
    pub reset_assert_complete_raw_offset: M68kCycleCount,
    pub reset_release_complete_raw_offset: M68kCycleCount,
}

/// Hardware work interleaved with one ASM `StopMusic` call.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GemsStopMusicArbitration {
    pub vint_interrupts: u16,
    pub refresh_wait_events: u16,
}

/// Derive ASM `StopMusic`'s raw CPU schedule without fitting elapsed route time.
///
/// `Stop_z80` and `sub_E143A loc_E1460` each execute at least one `$A11100`
/// poll. Every additional taken `btst/bne` iteration adds 30 raw 68000 cycles;
/// VINT pre-emption, refresh, and 68000 accesses to Z80 RAM remain external
/// work for the hardware scheduler.
pub fn gems_stop_music_timing(polls: GemsStopMusicBusAckPolls) -> Option<GemsStopMusicTiming> {
    let poll_counts = core::array::from_fn::<_, 7, _>(|index| {
        if index == 0 {
            polls.driver_copy_sub_e13da
        } else {
            polls.queue_transactions_sub_e143a[index - 1]
        }
    });
    if poll_counts.contains(&0) {
        return None;
    }

    // ASM StopMusic $E14C8, sub_E13DA, sub_E1416, sub_E149C/sub_E14AC.
    // These offsets come from the uninterrupted one-poll instruction stream;
    // only the explicit bus-ack loops can extend its raw source schedule.
    const BASE_WINDOWS: [(u32, u32); 7] = [
        (104, 214_784),
        (215_014, 215_184),
        (215_450, 215_620),
        (215_890, 216_244),
        (216_514, 216_868),
        (217_138, 217_492),
        (217_762, 218_116),
    ];
    const EXTRA_POLL_RAW_CYCLES: u32 = 30;

    let mut prior_extra = 0_u32;
    let mut windows = [GemsStopMusicBusWindow {
        request_complete_raw_offset: M68kCycleCount::from_raw(0),
        release_complete_raw_offset: M68kCycleCount::from_raw(0),
        m68k_z80_ram_accesses: 0,
    }; 7];
    // ASM loc_E1400/loc_E1406 fill all 8 KiB; each sub_E143A/sub_E148E
    // transaction reads the queue pointer, then writes data and pointer bytes.
    const M68K_Z80_RAM_ACCESSES: [u16; 7] = [8_192, 3, 3, 7, 7, 7, 7];
    for (index, ((base_request, base_release), poll_count)) in
        BASE_WINDOWS.into_iter().zip(poll_counts).enumerate()
    {
        let current_extra =
            u32::from(poll_count.checked_sub(1)?).checked_mul(EXTRA_POLL_RAW_CYCLES)?;
        windows[index] = GemsStopMusicBusWindow {
            request_complete_raw_offset: M68kCycleCount::from_raw(
                base_request.checked_add(prior_extra)?,
            ),
            release_complete_raw_offset: M68kCycleCount::from_raw(
                base_release
                    .checked_add(prior_extra)?
                    .checked_add(current_extra)?,
            ),
            m68k_z80_ram_accesses: M68K_Z80_RAM_ACCESSES[index],
        };
        prior_extra = prior_extra.checked_add(current_extra)?;
    }

    let driver_poll_extra = u32::from(polls.driver_copy_sub_e13da.checked_sub(1)?)
        .checked_mul(EXTRA_POLL_RAW_CYCLES)?;
    Some(GemsStopMusicTiming {
        raw_cycles: M68kCycleCount::from_raw(218_216_u32.checked_add(prior_extra)?),
        bus_windows: windows,
        reset_assert_complete_raw_offset: M68kCycleCount::from_raw(
            214_484_u32.checked_add(driver_poll_extra)?,
        ),
        reset_release_complete_raw_offset: M68kCycleCount::from_raw(
            214_804_u32.checked_add(driver_poll_extra)?,
        ),
    })
}

/// Elapsed 68000 cycles for ASM `StopMusic`, with hardware arbitration explicit.
pub fn gems_stop_music_elapsed_cycles(
    timing: GemsStopMusicTiming,
    arbitration: GemsStopMusicArbitration,
) -> Option<M68kCycleCount> {
    // jgenesis samples a pending interrupt for 10 cycles, then ASM V_Int $488
    // takes 44 raw cycles to enter the level-6 autovector and 60 for ADDQ.L/ST/RTE.
    const VINT_RAW_CYCLES: u32 = 10 + 44 + 60;

    let z80_ram_waits = timing
        .bus_windows
        .into_iter()
        .try_fold(0_u32, |total, window| {
            total.checked_add(u32::from(window.m68k_z80_ram_accesses))
        })?;
    let vint_raw_cycles = u32::from(arbitration.vint_interrupts).checked_mul(VINT_RAW_CYCLES)?;
    let refresh_wait_cycles = u32::from(arbitration.refresh_wait_events).checked_mul(2)?;
    Some(M68kCycleCount::from_raw(
        timing
            .raw_cycles
            .get()
            .checked_add(z80_ram_waits)?
            .checked_add(vint_raw_cycles)?
            .checked_add(refresh_wait_cycles)?,
    ))
}

/// One source bus window positioned on the elapsed 68000 timeline.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GemsStopMusicElapsedBusWindow {
    pub request_complete_elapsed_offset: M68kCycleCount,
    pub ack_complete_elapsed_offset: M68kCycleCount,
    pub release_complete_elapsed_offset: M68kCycleCount,
}

/// Event-positioned execution of ASM `StopMusic`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GemsStopMusicElapsedTiming {
    pub elapsed_cycles: M68kCycleCount,
    pub bus_windows: [GemsStopMusicElapsedBusWindow; 7],
    pub driver_reset_release_complete_elapsed_offset: M68kCycleCount,
    pub reset_assert_complete_elapsed_offset: M68kCycleCount,
    pub reset_release_complete_elapsed_offset: M68kCycleCount,
    pub phase_after: M68kRefreshPhase,
    pub refresh_wait_events: u32,
    pub driver_reset_release_elapsed_offset: M68kCycleCount,
    pub driver_bus_request_elapsed_offset: M68kCycleCount,
    pub reset_assert_elapsed_offset: M68kCycleCount,
    driver_copy_write_elapsed_offsets: Vec<M68kCycleCount>,
    driver_clear_write_elapsed_offsets: Vec<M68kCycleCount>,
    driver_ram_access_pending_ranges: Vec<(M68kCycleCount, M68kCycleCount)>,
}

impl GemsStopMusicElapsedTiming {
    /// Bytes made visible when ASM `loc_E1400` starts its write instruction.
    pub fn driver_copy_bytes_visible(&self, elapsed: M68kCycleCount) -> usize {
        self.driver_copy_write_elapsed_offsets
            .partition_point(|offset| offset.get() <= elapsed.get())
    }

    /// Bytes made visible when ASM `loc_E1406` starts its clear instruction.
    pub fn driver_clear_bytes_visible(&self, elapsed: M68kCycleCount) -> usize {
        self.driver_clear_write_elapsed_offsets
            .partition_point(|offset| offset.get() <= elapsed.get())
    }

    /// Whether a sampled M68K instruction quantum still carries the Z80-RAM
    /// access wait raised by `loc_E1400`/`loc_E1406`.
    pub fn driver_ram_access_wait_pending(&self, elapsed: M68kCycleCount) -> bool {
        self.driver_ram_access_pending_ranges
            .iter()
            .any(|(start, end)| start.get() <= elapsed.get() && elapsed.get() < end.get())
    }
}

/// Source wrapper used by one GEMS queue command.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GemsQueueCommandPath {
    /// ASM `sub_E151C`, shared by `PlaySound` and `PlaySound2`.
    StartOrStopSequenceSubE151c,
    /// ASM `sub_E1546 -> sub_E1552`: `$05 <speed>`.
    MusicSpeedPrefixSubE1546,
    /// ASM `sub_E1568`: `$0D`.
    MusicSpeedCommitSubE1568,
    /// ASM `sub_E133C -> loc_E1576`: acquire and release without queue data.
    OpenCloseSubE1576,
}

/// Hardware-visible control boundary in one GEMS queue command.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GemsQueueCommandControl {
    BeginQueueBus,
    WriteQueueBytes,
    ReleaseQueueBus,
}

/// Chronological result of one GEMS queue-command source path.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GemsQueueCommandAdvance {
    pub elapsed: M68kCycleCount,
    pub phase_after: M68kRefreshPhase,
    pub queue_bus_request_complete: M68kCycleCount,
    pub queue_write_complete: M68kCycleCount,
    pub queue_bus_release_complete: M68kCycleCount,
}

struct GemsQueueCommandClock {
    elapsed: u32,
    phase: M68kRefreshPhase,
    bus_owned: bool,
}

impl GemsQueueCommandClock {
    fn instruction(
        &mut self,
        raw_cycles: u32,
        z80_ram_access: bool,
        advance_hardware: &mut impl FnMut(M68kCycleCount, bool) -> Option<M68kCycleCount>,
    ) -> Option<()> {
        let phase_after_raw = u32::from(self.phase.get()).checked_add(raw_cycles)?;
        let refresh_wait = if phase_after_raw >= 128 { 2 } else { 0 };
        self.phase = M68kRefreshPhase::new(
            u8::try_from(phase_after_raw % 128).expect("refresh phase remains below 128"),
        )
        .expect("refresh phase remains below 128");
        self.elapsed = self.elapsed.checked_add(raw_cycles)?;

        let generated_wait =
            advance_hardware(M68kCycleCount::from_raw(raw_cycles), self.bus_owned)?.get();
        let mut pending_wait = generated_wait
            .checked_add(refresh_wait)?
            .checked_add(u32::from(z80_ram_access))?;
        while pending_wait != 0 {
            let wait = M68kCycleCount::from_raw(pending_wait);
            self.phase = self.phase.advance_external_wait(wait).phase_after;
            self.elapsed = self.elapsed.checked_add(pending_wait)?;
            pending_wait = advance_hardware(wait, self.bus_owned)?.get();
        }
        Some(())
    }

    fn instructions(
        &mut self,
        instructions: &[u32],
        advance_hardware: &mut impl FnMut(M68kCycleCount, bool) -> Option<M68kCycleCount>,
    ) -> Option<()> {
        for &cycles in instructions {
            self.instruction(cycles, false, advance_hardware)?;
        }
        Some(())
    }
}

/// Co-schedule one ASM GEMS queue command with asynchronous hardware waits.
///
/// The hardware callback advances each real interval and returns newly queued
/// 68000 wait cycles, allowing banked Z80 reads before BUSREQ to delay the
/// source instruction stream. Control events are emitted only after their
/// defining instruction and pending waits complete.
pub fn gems_queue_command_advance_with_scheduler(
    path: GemsQueueCommandPath,
    phase_before: M68kRefreshPhase,
    mut advance_hardware: impl FnMut(M68kCycleCount, bool) -> Option<M68kCycleCount>,
    mut apply_control: impl FnMut(GemsQueueCommandControl),
) -> Option<GemsQueueCommandAdvance> {
    let mut clock = GemsQueueCommandClock {
        elapsed: 0,
        phase: phase_before,
        bus_owned: false,
    };

    if path == GemsQueueCommandPath::MusicSpeedPrefixSubE1546 {
        // ASM sub_E1546 wrapper through entry to sub_E1552.
        clock.instructions(&[12, 12, 18], &mut advance_hardware)?;
    }

    // ASM sub_E151C/sub_E1552/sub_E1568 -> sub_E143A, through completion of
    // the `$A11100` BUSREQ write.
    clock.instructions(
        &[18, 12, 16, 24, 14, 12, 12, 12, 20, 20],
        &mut advance_hardware,
    )?;
    clock.bus_owned = true;
    let queue_bus_request_complete = M68kCycleCount::from_raw(clock.elapsed);
    apply_control(GemsQueueCommandControl::BeginQueueBus);

    // ASM loc_E1460 one-poll BUSACK path, queue-pointer read, and return into
    // the command-specific writer.
    for (cycles, access) in [(20, false), (8, false), (8, true), (4, false), (16, false)] {
        clock.instruction(cycles, access, &mut advance_hardware)?;
    }

    match path {
        GemsQueueCommandPath::StartOrStopSequenceSubE151c
        | GemsQueueCommandPath::MusicSpeedPrefixSubE1546 => {
            // ASM sub_E151C/sub_E1552 -> sub_E1482 -> sub_E148E writes
            // `FF,<command>`, then the argument and its pointer update.
            if path == GemsQueueCommandPath::MusicSpeedPrefixSubE1546 {
                // ASM sub_E1552 branches through loc_E1522.
                clock.instructions(&[4, 10, 18], &mut advance_hardware)?;
            } else {
                clock.instructions(&[4, 18], &mut advance_hardware)?;
            }
            for (cycles, access) in [
                (18, true),
                (4, false),
                (8, false),
                (14, true),
                (4, false),
                (8, false),
                (8, true),
                (16, false),
                (16, false),
                (18, false),
                (14, true),
                (4, false),
                (8, false),
                (8, true),
                (16, false),
                (10, false),
            ] {
                clock.instruction(cycles, access, &mut advance_hardware)?;
            }
        }
        GemsQueueCommandPath::MusicSpeedCommitSubE1568 => {
            // ASM sub_E1568 writes only `FF,$0D`.
            clock.instructions(&[4, 18], &mut advance_hardware)?;
            for (cycles, access) in [
                (18, true),
                (4, false),
                (8, false),
                (14, true),
                (4, false),
                (8, false),
                (8, true),
                (16, false),
                (10, false),
            ] {
                clock.instruction(cycles, access, &mut advance_hardware)?;
            }
        }
        GemsQueueCommandPath::OpenCloseSubE1576 => {
            // ASM loc_E1576 returns from sub_E143A directly to loc_E1470.
            clock.instruction(10, false, &mut advance_hardware)?;
        }
    }
    let queue_write_complete = M68kCycleCount::from_raw(clock.elapsed);
    if path != GemsQueueCommandPath::OpenCloseSubE1576 {
        apply_control(GemsQueueCommandControl::WriteQueueBytes);
    }

    clock.instruction(20, false, &mut advance_hardware)?;
    clock.bus_owned = false;
    let queue_bus_release_complete = M68kCycleCount::from_raw(clock.elapsed);
    apply_control(GemsQueueCommandControl::ReleaseQueueBus);
    clock.instructions(&[16, 28, 12, 16], &mut advance_hardware)?;

    if path == GemsQueueCommandPath::MusicSpeedPrefixSubE1546 {
        // Return from sub_E1552, then restore the wrapper's saved registers.
        clock.instructions(&[12, 12, 16], &mut advance_hardware)?;
    }

    Some(GemsQueueCommandAdvance {
        elapsed: M68kCycleCount::from_raw(clock.elapsed),
        phase_after: clock.phase,
        queue_bus_request_complete,
        queue_write_complete,
        queue_bus_release_complete,
    })
}

struct GemsStopMusicClock<'a> {
    raw_cycles: u32,
    elapsed_cycles: u32,
    last_source_instruction_completion_elapsed: u32,
    last_source_instruction_start_elapsed: u32,
    phase: M68kRefreshPhase,
    refresh_wait_events: u32,
    z80_ram_accesses: u32,
    vint_schedule: GemsStopMusicVintSchedule<'a>,
}

enum GemsStopMusicVintSchedule<'a> {
    Explicit {
        source_offsets: &'a [M68kCycleCount],
        next: usize,
    },
    Absolute {
        entry_master_clock: u64,
        next_assertion_master_clock: u64,
    },
}

impl GemsStopMusicClock<'_> {
    fn source_instruction(&mut self, raw_cycles: u32, z80_ram_access: bool) -> Option<()> {
        self.inject_vint_before_source_instruction()?;
        self.last_source_instruction_start_elapsed = self.elapsed_cycles;
        let advance = self.phase.advance_rom(M68kCycleCount::from_raw(raw_cycles));
        self.raw_cycles = self.raw_cycles.checked_add(raw_cycles)?;
        self.last_source_instruction_completion_elapsed =
            self.elapsed_cycles.checked_add(raw_cycles)?;
        self.elapsed_cycles = self.elapsed_cycles.checked_add(advance.elapsed.get())?;
        self.phase = advance.phase_after;
        self.refresh_wait_events = self
            .refresh_wait_events
            .checked_add(advance.refresh_events)?;
        if z80_ram_access {
            let wait = self
                .phase
                .advance_external_wait(M68kCycleCount::from_raw(1));
            self.elapsed_cycles = self.elapsed_cycles.checked_add(1)?;
            self.phase = wait.phase_after;
            self.z80_ram_accesses = self.z80_ram_accesses.checked_add(1)?;
        }
        Some(())
    }

    fn inject_vint_before_source_instruction(&mut self) -> Option<()> {
        loop {
            let inject = match &self.vint_schedule {
                GemsStopMusicVintSchedule::Explicit {
                    source_offsets,
                    next,
                } => {
                    let Some(offset) = source_offsets.get(*next) else {
                        return Some(());
                    };
                    if offset.get() != self.raw_cycles {
                        return (offset.get() > self.raw_cycles).then_some(());
                    }
                    true
                }
                GemsStopMusicVintSchedule::Absolute {
                    entry_master_clock,
                    next_assertion_master_clock,
                } => {
                    let current_master_clock = entry_master_clock
                        .checked_add(u64::from(self.elapsed_cycles).checked_mul(7)?)?;
                    current_master_clock >= *next_assertion_master_clock
                }
            };
            if !inject {
                return Some(());
            }
            // jgenesis samples the pending interrupt for 10 cycles. ASM V_Int
            // $488 then takes 44 to enter and 60 for ADDQ.L/ST/RTE.
            self.hardware_work(10)?;
            self.hardware_work(44)?;
            self.hardware_work(28)?;
            self.hardware_work(12)?;
            self.hardware_work(20)?;
            match &mut self.vint_schedule {
                GemsStopMusicVintSchedule::Explicit { next, .. } => *next += 1,
                GemsStopMusicVintSchedule::Absolute {
                    next_assertion_master_clock,
                    ..
                } => {
                    *next_assertion_master_clock =
                        next_assertion_master_clock.checked_add(3_420 * 262)?;
                }
            }
        }
    }

    fn consumed_explicit_vints(&self) -> bool {
        match self.vint_schedule {
            GemsStopMusicVintSchedule::Explicit {
                source_offsets,
                next,
            } => next == source_offsets.len(),
            GemsStopMusicVintSchedule::Absolute { .. } => true,
        }
    }

    fn hardware_work(&mut self, raw_cycles: u32) -> Option<()> {
        let advance = self.phase.advance_rom(M68kCycleCount::from_raw(raw_cycles));
        self.elapsed_cycles = self.elapsed_cycles.checked_add(advance.elapsed.get())?;
        self.phase = advance.phase_after;
        self.refresh_wait_events = self
            .refresh_wait_events
            .checked_add(advance.refresh_events)?;
        Some(())
    }

    fn source_instructions(&mut self, instructions: &[u32]) -> Option<()> {
        for &cycles in instructions {
            self.source_instruction(cycles, false)?;
        }
        Some(())
    }

    fn poll_bus_ack(&mut self, polls: u16) -> Option<()> {
        for poll in 0..polls {
            self.source_instruction(20, false)?;
            self.source_instruction(if poll + 1 == polls { 8 } else { 10 }, false)?;
        }
        Some(())
    }

    fn queue_byte_write_sub_e148e(&mut self) -> Option<()> {
        // ASM sub_E148E writes the data byte and then the updated queue pointer.
        self.source_instruction(14, true)?;
        self.source_instructions(&[4, 8])?;
        self.source_instruction(8, true)?;
        self.source_instruction(16, false)
    }
}

/// Position source control events after refresh, Z80-RAM waits, and VINT work.
///
/// VINT offsets are raw `StopMusic` instruction boundaries. They do not move
/// the source offsets, but their entry/handler work advances elapsed time.
pub fn gems_stop_music_elapsed_timing(
    polls: GemsStopMusicBusAckPolls,
    phase_before: M68kRefreshPhase,
    vint_source_offsets: &[M68kCycleCount],
) -> Option<GemsStopMusicElapsedTiming> {
    if vint_source_offsets
        .windows(2)
        .any(|pair| pair[0].get() >= pair[1].get())
    {
        return None;
    }
    gems_stop_music_elapsed_timing_with_vint_schedule(
        polls,
        phase_before,
        GemsStopMusicVintSchedule::Explicit {
            source_offsets: vint_source_offsets,
            next: 0,
        },
    )
}

/// Position ASM `StopMusic` VINT work from its absolute 68000 entry counter.
///
/// The Mega Drive VDP asserts the level-6 interrupt at H40 scanline 224,
/// master-clock 16. The 68000 samples that pending interrupt at the next
/// source instruction boundary, so no route-specific raw instruction offset
/// is needed. Callers must use the explicit-offset API with no events while
/// the status-register interrupt mask rejects level 6.
pub fn gems_stop_music_elapsed_timing_from_absolute_entry(
    polls: GemsStopMusicBusAckPolls,
    phase_before: M68kRefreshPhase,
    absolute_entry_cycles: M68kCycleCount,
) -> Option<GemsStopMusicElapsedTiming> {
    const MCLK_PER_M68K_CYCLE: u64 = 7;
    const MCLK_PER_SCANLINE: u64 = 3_420;
    const NTSC_SCANLINES_PER_FRAME: u64 = 262;
    const VINT_ASSERTION_SCANLINE: u64 = 224;
    const VINT_ASSERTION_SCANLINE_MCLK: u64 = 16;

    let entry_master_clock =
        u64::from(absolute_entry_cycles.get()).checked_mul(MCLK_PER_M68K_CYCLE)?;
    let frame_master_clocks = MCLK_PER_SCANLINE.checked_mul(NTSC_SCANLINES_PER_FRAME)?;
    let assertion_in_frame = VINT_ASSERTION_SCANLINE
        .checked_mul(MCLK_PER_SCANLINE)?
        .checked_add(VINT_ASSERTION_SCANLINE_MCLK)?;
    let entry_in_frame = entry_master_clock % frame_master_clocks;
    let until_assertion = if entry_in_frame < assertion_in_frame {
        assertion_in_frame - entry_in_frame
    } else {
        frame_master_clocks - entry_in_frame + assertion_in_frame
    };
    let next_assertion_master_clock = entry_master_clock.checked_add(until_assertion)?;

    gems_stop_music_elapsed_timing_with_vint_schedule(
        polls,
        phase_before,
        GemsStopMusicVintSchedule::Absolute {
            entry_master_clock,
            next_assertion_master_clock,
        },
    )
}

fn gems_stop_music_elapsed_timing_with_vint_schedule(
    polls: GemsStopMusicBusAckPolls,
    phase_before: M68kRefreshPhase,
    vint_schedule: GemsStopMusicVintSchedule<'_>,
) -> Option<GemsStopMusicElapsedTiming> {
    let source = gems_stop_music_timing(polls)?;
    let mut clock = GemsStopMusicClock {
        raw_cycles: 0,
        elapsed_cycles: 0,
        last_source_instruction_completion_elapsed: 0,
        last_source_instruction_start_elapsed: 0,
        phase: phase_before,
        refresh_wait_events: 0,
        z80_ram_accesses: 0,
        vint_schedule,
    };
    let mut bus_windows = [GemsStopMusicElapsedBusWindow {
        request_complete_elapsed_offset: M68kCycleCount::from_raw(0),
        ack_complete_elapsed_offset: M68kCycleCount::from_raw(0),
        release_complete_elapsed_offset: M68kCycleCount::from_raw(0),
    }; 7];
    let mut driver_copy_write_elapsed_offsets = Vec::with_capacity(5_764);
    let mut driver_clear_write_elapsed_offsets = Vec::with_capacity(2_428);
    let mut driver_ram_access_pending_ranges = Vec::with_capacity(8_192);

    // ASM StopMusic -> sub_E13DA first releases power-on reset, then enters
    // Stop_z80 and writes BUSREQ.
    clock.source_instructions(&[16, 18, 12, 20])?;
    let driver_reset_release_elapsed_offset =
        M68kCycleCount::from_raw(clock.last_source_instruction_start_elapsed);
    let driver_reset_release_complete_elapsed_offset =
        M68kCycleCount::from_raw(clock.last_source_instruction_completion_elapsed);
    clock.source_instructions(&[18, 20])?;
    let driver_bus_request_elapsed_offset =
        M68kCycleCount::from_raw(clock.last_source_instruction_start_elapsed);
    if clock.raw_cycles != source.bus_windows[0].request_complete_raw_offset.get() {
        return None;
    }
    bus_windows[0].request_complete_elapsed_offset =
        M68kCycleCount::from_raw(clock.last_source_instruction_completion_elapsed);
    clock.poll_bus_ack(polls.driver_copy_sub_e13da)?;
    bus_windows[0].ack_complete_elapsed_offset =
        M68kCycleCount::from_raw(clock.last_source_instruction_completion_elapsed);
    clock.source_instructions(&[16, 12, 12, 4, 8, 4, 12])?;

    // ASM loc_E1400 copies 5,764 driver bytes.
    for index in 0..5_764 {
        clock.source_instruction(12, true)?;
        driver_copy_write_elapsed_offsets.push(M68kCycleCount::from_raw(
            clock.last_source_instruction_start_elapsed,
        ));
        driver_ram_access_pending_ranges.push((
            M68kCycleCount::from_raw(clock.last_source_instruction_start_elapsed),
            M68kCycleCount::from_raw(clock.elapsed_cycles),
        ));
        clock.source_instruction(if index == 5_763 { 14 } else { 10 }, false)?;
    }
    // ASM loc_E1406 clears the remaining 2,428 bytes of Z80 RAM.
    for index in 0..2_428 {
        clock.source_instruction(12, true)?;
        driver_clear_write_elapsed_offsets.push(M68kCycleCount::from_raw(
            clock.last_source_instruction_start_elapsed,
        ));
        driver_ram_access_pending_ranges.push((
            M68kCycleCount::from_raw(clock.last_source_instruction_start_elapsed),
            M68kCycleCount::from_raw(clock.elapsed_cycles),
        ));
        clock.source_instruction(14, false)?;
        clock.source_instruction(if index == 2_427 { 8 } else { 10 }, false)?;
    }

    // ASM sub_E1416 asserts reset, delays, releases BUSREQ, then reset.
    clock.source_instructions(&[12, 16, 18, 20])?;
    let reset_assert_elapsed_offset =
        M68kCycleCount::from_raw(clock.last_source_instruction_start_elapsed);
    if clock.raw_cycles != source.reset_assert_complete_raw_offset.get() {
        return None;
    }
    let reset_assert_complete_elapsed_offset =
        M68kCycleCount::from_raw(clock.last_source_instruction_completion_elapsed);
    clock.source_instruction(12, false)?;
    for index in 0..15 {
        clock.source_instruction(8, false)?;
        clock.source_instruction(if index == 14 { 8 } else { 10 }, false)?;
    }
    clock.source_instruction(20, false)?;
    if clock.raw_cycles != source.bus_windows[0].release_complete_raw_offset.get() {
        return None;
    }
    bus_windows[0].release_complete_elapsed_offset =
        M68kCycleCount::from_raw(clock.last_source_instruction_completion_elapsed);
    clock.source_instruction(20, false)?;
    if clock.raw_cycles != source.reset_release_complete_raw_offset.get() {
        return None;
    }
    let reset_release_complete_elapsed_offset =
        M68kCycleCount::from_raw(clock.last_source_instruction_completion_elapsed);
    clock.source_instruction(16, false)?;

    for transaction in 0..6 {
        if transaction < 2 {
            clock.source_instructions(&[4, 12, 18, 18])?;
        } else {
            clock.source_instructions(&[20, 18, 18])?;
        }
        // ASM sub_E143A through the BUSREQ write and loc_E1460 poll.
        clock.source_instructions(&[12, 16, 24, 14, 12, 12, 12, 20, 20])?;
        let window = transaction + 1;
        if clock.raw_cycles != source.bus_windows[window].request_complete_raw_offset.get() {
            return None;
        }
        bus_windows[window].request_complete_elapsed_offset =
            M68kCycleCount::from_raw(clock.last_source_instruction_completion_elapsed);
        clock.poll_bus_ack(polls.queue_transactions_sub_e143a[transaction])?;
        bus_windows[window].ack_complete_elapsed_offset =
            M68kCycleCount::from_raw(clock.last_source_instruction_completion_elapsed);
        clock.source_instruction(8, true)?;
        clock.source_instructions(&[4, 16])?;

        if transaction < 2 {
            clock.source_instructions(&[16, 18])?;
            clock.queue_byte_write_sub_e148e()?;
        } else {
            clock.source_instructions(&[16, 18])?;
            clock.queue_byte_write_sub_e148e()?;
            clock.source_instructions(&[24, 18])?;
            clock.queue_byte_write_sub_e148e()?;
            clock.source_instructions(&[24, 18])?;
            clock.queue_byte_write_sub_e148e()?;
        }
        clock.source_instruction(10, false)?;
        clock.source_instruction(20, false)?;
        if clock.raw_cycles != source.bus_windows[window].release_complete_raw_offset.get() {
            return None;
        }
        bus_windows[window].release_complete_elapsed_offset =
            M68kCycleCount::from_raw(clock.last_source_instruction_completion_elapsed);
        clock.source_instructions(&[16, 28, 12, 16])?;
    }
    clock.source_instructions(&[12, 16])?;

    if clock.raw_cycles != source.raw_cycles.get()
        || !clock.consumed_explicit_vints()
        || clock.z80_ram_accesses
            != source
                .bus_windows
                .iter()
                .map(|window| u32::from(window.m68k_z80_ram_accesses))
                .sum::<u32>()
    {
        return None;
    }
    Some(GemsStopMusicElapsedTiming {
        elapsed_cycles: M68kCycleCount::from_raw(clock.elapsed_cycles),
        bus_windows,
        driver_reset_release_complete_elapsed_offset,
        reset_assert_complete_elapsed_offset,
        reset_release_complete_elapsed_offset,
        phase_after: clock.phase,
        refresh_wait_events: clock.refresh_wait_events,
        driver_reset_release_elapsed_offset,
        driver_bus_request_elapsed_offset,
        reset_assert_elapsed_offset,
        driver_copy_write_elapsed_offsets,
        driver_clear_write_elapsed_offsets,
        driver_ram_access_pending_ranges,
    })
}

/// Source branches taken before ASM `Mode_Level` reaches `Execute_Objects`.
///
/// VDP arbitration is deliberately excluded: the same ROM path can spend a
/// different number of cycles halted behind the VDP on adjacent frames.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct ModeLevelVblankWorkBeforeSubE12 {
    /// Tile lengths read by `Transfer_SpriteAndKidToVRAM loc_AD4`. Core
    /// traces leave this unclassified because the selected mapping dimensions
    /// belong to the render asset boundary.
    pub dynamic_art_tiles_loc_ad4: Option<[u16; 6]>,
    pub palette_dma_sub_970: bool,
    pub scroll_path: ModeLevelScrollTransferPath,
    pub foreground_custom_queue_entries_sub_1596: u16,
    pub foreground_cell_queue_sub_1596: ModeLevelForegroundCellQueueWorkSub1596,
    pub paused_sub_6874: bool,
    pub hud_path_sub_bdc0: ModeLevelHudBuildPath,
}

/// Ordered ASM `sub_1596` branch taken by one foreground cell-queue record.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ModeLevelForegroundCellQueuePathSub1596 {
    VisibleTheme,
    VisibleBlock,
    Culled,
}

/// Compact ordered foreground cell-queue workload consumed before `sub_E12`.
///
/// `FGPlane_UpdateQueueCell` has room for fewer than 128 six-byte records.
/// Two bits per record retain branch order without putting a heap-owned trace
/// value on every native frame. Construction fails closed beyond that bound.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct ModeLevelForegroundCellQueueWorkSub1596 {
    packed_paths: [u64; 4],
    len: u8,
}

impl ModeLevelForegroundCellQueueWorkSub1596 {
    pub const EMPTY: Self = Self {
        packed_paths: [0; 4],
        len: 0,
    };

    pub fn from_paths(paths: &[ModeLevelForegroundCellQueuePathSub1596]) -> Option<Self> {
        if paths.len() > 128 {
            return None;
        }
        let mut packed_paths = [0_u64; 4];
        for (index, path) in paths.iter().copied().enumerate() {
            let encoded = match path {
                ModeLevelForegroundCellQueuePathSub1596::VisibleTheme => 0_u64,
                ModeLevelForegroundCellQueuePathSub1596::VisibleBlock => 1,
                ModeLevelForegroundCellQueuePathSub1596::Culled => 2,
            };
            packed_paths[index / 32] |= encoded << ((index % 32) * 2);
        }
        Some(Self {
            packed_paths,
            len: paths.len() as u8,
        })
    }

    pub const fn is_empty(self) -> bool {
        self.len == 0
    }

    pub fn path(self, index: usize) -> Option<ModeLevelForegroundCellQueuePathSub1596> {
        if index >= usize::from(self.len) {
            return None;
        }
        match (self.packed_paths[index / 32] >> ((index % 32) * 2)) & 3 {
            0 => Some(ModeLevelForegroundCellQueuePathSub1596::VisibleTheme),
            1 => Some(ModeLevelForegroundCellQueuePathSub1596::VisibleBlock),
            2 => Some(ModeLevelForegroundCellQueuePathSub1596::Culled),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ModeLevelScrollTransferPath {
    /// ASM `loc_300BA`: two VSRAM words then the `$380`-byte H-scroll DMA.
    NormalLoc300ba,
    StormLoc30110,
    SpecialLoc30158,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ModeLevelHudBuildPath {
    /// ASM `loc_BE20 -> loc_BE82`, followed by the first gameplay pass
    /// updating the life and diamond display counters.
    LocBe20DirectInitialCounterUpdates,
    /// ASM `loc_BE20 -> loc_BE82`, with unchanged life and diamond display
    /// counters.
    LocBe20DirectStableCounters,
    /// ASM `loc_BE20 -> loc_BE0C -> loc_BE34 -> loc_BE4C`, with no timer
    /// rollover and unchanged life and diamond display counters.
    LocBe20ThenLocBe4cStableCounters,
    Other,
}

/// Classify the natural non-transforming ASM `Make_SpriteAttr_HUD` path.
pub fn mode_level_hud_build_path_sub_bdc0(
    number_hitpoints: u16,
    current_helmet: u16,
    extra_hitpoint_slots: u16,
    initial_counter_updates: bool,
) -> ModeLevelHudBuildPath {
    // ASM HelmetHitpoint_Table and loc_BE0C..loc_BE82 (kid.asm:15370-15461).
    const HELMET_HITPOINT_SLOTS: [u16; 10] = [2, 3, 3, 3, 3, 3, 5, 3, 3, 3];
    let Some(base_slots) = HELMET_HITPOINT_SLOTS.get(usize::from(current_helmet)) else {
        return ModeLevelHudBuildPath::Other;
    };
    let slots = base_slots.wrapping_add(extra_hitpoint_slots);
    match (number_hitpoints, slots, initial_counter_updates) {
        (2, 2, true) => ModeLevelHudBuildPath::LocBe20DirectInitialCounterUpdates,
        (2, 2, false) => ModeLevelHudBuildPath::LocBe20DirectStableCounters,
        (3, 3, false) => ModeLevelHudBuildPath::LocBe20ThenLocBe4cStableCounters,
        _ => ModeLevelHudBuildPath::Other,
    }
}

/// Hardware arbitration elapsed while the source path is otherwise fixed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ModeLevelVblankArbitrationBeforeSubE12 {
    /// One-cycle CPU re-entries while jgenesis reports that the VDP owns the
    /// 68k bus during memory-to-VDP DMA.
    pub vdp_bus_halt_cycles: u32,
    /// Already-pending VDP wait cycles processed outside CPU instructions.
    pub vdp_pending_wait_cycles: u32,
    /// Already-pending ROM refresh waits processed before `sub_E12`.
    pub refresh_wait_cycles: u32,
    /// Fixed source/pending-wait suffix after the final VDP-owned CPU
    /// re-entry resets jgenesis's refresh counter.
    pub refresh_suffix_after_final_vdp_ownership:
        ModeLevelVblankRefreshSuffixAfterFinalVdpOwnership,
}

/// Ordered work after the final VDP-owned re-entry before `sub_E12`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ModeLevelVblankRefreshSuffixAfterFinalVdpOwnership {
    pub raw_cycles_before_pending_wait: u32,
    pub pending_wait_cycles: u32,
    pub raw_cycles_after_pending_wait: u32,
}

/// ASM `sub_30194` branch selected by the current and cached camera tiles.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ModeLevelTailCameraCellMappingSub30194 {
    /// Theme mapping with the priority bit clear.
    OrdinaryTheme,
    /// Theme mapping with the priority bit set.
    PriorityTheme,
    /// Negative `Level_Layout` word selecting `Block_Mappings`.
    Block,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ModeLevelTailCameraStripeSub30194 {
    /// ASM loc_301B4/loc_30246: neither camera tile axis needs streaming.
    BothTileAxesUnchanged,
    /// ASM loc_301CE -> sub_3034A: upward, unsplit 21-longword row upload.
    UpwardHorizontalNoWrap {
        cells: [ModeLevelTailCameraCellMappingSub30194; 21],
    },
    /// ASM loc_301B4 -> loc_301CE -> sub_3034A: downward, unsplit
    /// 21-longword row upload.
    DownwardHorizontalNoWrap {
        cells: [ModeLevelTailCameraCellMappingSub30194; 21],
    },
    /// ASM `loc_301F6 -> sub_3034A` twice: upward row split at the plane-A
    /// boundary, with the first pass length retained for FIFO ordering.
    UpwardHorizontalSplit {
        cells: [ModeLevelTailCameraCellMappingSub30194; 21],
        first_pass_cells: u8,
    },
    /// ASM `loc_3023C -> loc_30262 -> sub_302F6`: the camera crossed a
    /// rightward tile boundary and the unsplit 15-cell column was uploaded.
    RightwardVerticalNoWrap {
        cells: [ModeLevelTailCameraCellMappingSub30194; 15],
    },
    /// ASM `loc_3023C -> loc_30262 -> sub_302F6`: leftward 15-cell column.
    LeftwardVerticalNoWrap {
        cells: [ModeLevelTailCameraCellMappingSub30194; 15],
    },
    /// ASM `loc_301CE -> sub_3034A`, then `loc_30262 -> sub_302F6`: an
    /// upward row followed by a rightward column, optionally split at the
    /// plane boundary.
    UpwardHorizontalRightwardVertical {
        row_cells: [ModeLevelTailCameraCellMappingSub30194; 21],
        column_cells: [ModeLevelTailCameraCellMappingSub30194; 15],
        first_row_pass_cells: u8,
        first_column_pass_cells: u8,
    },
    /// One or both axes use another streaming shape.
    StreamingOther,
}

/// Classify ASM `sub_30194` camera-stripe work from tile latches and selected
/// live `Level_Layout` cells.
pub fn mode_level_tail_camera_stripe_work_sub_30194(
    previous_camera_x_pos: u16,
    previous_camera_y_pos: u16,
    camera_x_pos: u16,
    camera_y_pos: u16,
    horizontal_cells: Option<&[u16]>,
    vertical_cells: Option<&[u16]>,
) -> ModeLevelTailCameraStripeSub30194 {
    fn classify_cells<const N: usize>(
        cells: &[u16],
    ) -> Option<[ModeLevelTailCameraCellMappingSub30194; N]> {
        let cells: &[u16; N] = cells.try_into().ok()?;
        Some(std::array::from_fn(|index| {
            let cell = cells[index];
            if (cell as i16) < 0 {
                ModeLevelTailCameraCellMappingSub30194::Block
            } else if cell & 0x0100 != 0 {
                ModeLevelTailCameraCellMappingSub30194::PriorityTheme
            } else {
                ModeLevelTailCameraCellMappingSub30194::OrdinaryTheme
            }
        }))
    }

    let previous_x_tile = previous_camera_x_pos >> 3;
    let previous_y_tile = (previous_camera_y_pos as i16) >> 3;
    let x_tile = camera_x_pos >> 3;
    let y_tile = (camera_y_pos as i16) >> 3;
    if previous_x_tile == x_tile && previous_y_tile == y_tile {
        return ModeLevelTailCameraStripeSub30194::BothTileAxesUnchanged;
    }

    if y_tile < previous_y_tile && previous_x_tile < x_tile {
        let (Some(row_cells), Some(column_cells)) = (
            horizontal_cells.and_then(classify_cells::<21>),
            vertical_cells.and_then(classify_cells::<15>),
        ) else {
            return ModeLevelTailCameraStripeSub30194::StreamingOther;
        };
        let plane_cell_column = (x_tile >> 1) & 0x001F;
        let first_row_pass_cells = if plane_cell_column < 0x000C {
            21
        } else {
            32_u16.wrapping_sub(plane_cell_column)
        };
        let plane_block_row = (y_tile as u16 >> 1) & 0x000F;
        let first_column_pass_cells = if plane_block_row < 2 {
            15
        } else {
            16_u16.wrapping_sub(plane_block_row)
        };
        if (1..=15).contains(&first_column_pass_cells) {
            return ModeLevelTailCameraStripeSub30194::UpwardHorizontalRightwardVertical {
                row_cells,
                column_cells,
                first_row_pass_cells: first_row_pass_cells as u8,
                first_column_pass_cells: first_column_pass_cells as u8,
            };
        }
        return ModeLevelTailCameraStripeSub30194::StreamingOther;
    }

    if previous_y_tile == y_tile && previous_x_tile < x_tile {
        let Some(cells) = vertical_cells.filter(|cells| cells.len() == 15) else {
            return ModeLevelTailCameraStripeSub30194::StreamingOther;
        };
        let Some(paths) = classify_cells::<15>(cells) else {
            return ModeLevelTailCameraStripeSub30194::StreamingOther;
        };
        return ModeLevelTailCameraStripeSub30194::RightwardVerticalNoWrap { cells: paths };
    }
    if previous_y_tile == y_tile && previous_x_tile > x_tile {
        let Some(paths) = vertical_cells.and_then(classify_cells::<15>) else {
            return ModeLevelTailCameraStripeSub30194::StreamingOther;
        };
        return ModeLevelTailCameraStripeSub30194::LeftwardVerticalNoWrap { cells: paths };
    }

    // ASM loc_301CE/loc_301FE: a vertical-only row at plane columns 0..11
    // emits one unsplit 21-longword sub_3034A pass. The imperative shell
    // supplies the current row while moving up and the d6+$1C row while
    // moving down. Other axis and wrap combinations remain unclassified.
    let plane_cell_column = (x_tile >> 1) & 0x001F;
    let Some(cells) = horizontal_cells.filter(|cells| cells.len() == 21) else {
        return ModeLevelTailCameraStripeSub30194::StreamingOther;
    };
    if previous_x_tile != x_tile || y_tile == previous_y_tile {
        return ModeLevelTailCameraStripeSub30194::StreamingOther;
    }

    let Some(cells) = classify_cells::<21>(cells) else {
        return ModeLevelTailCameraStripeSub30194::StreamingOther;
    };
    if y_tile < previous_y_tile {
        if plane_cell_column < 0x000C {
            ModeLevelTailCameraStripeSub30194::UpwardHorizontalNoWrap { cells }
        } else {
            ModeLevelTailCameraStripeSub30194::UpwardHorizontalSplit {
                cells,
                first_pass_cells: 32_u16.wrapping_sub(plane_cell_column) as u8,
            }
        }
    } else {
        ModeLevelTailCameraStripeSub30194::DownwardHorizontalNoWrap { cells }
    }
}

/// Raw source cycles for the classified ASM `sub_30194` call.
pub fn mode_level_tail_camera_stripe_raw_cycles_sub_30194(
    work: ModeLevelTailCameraStripeSub30194,
) -> Option<M68kCycleCount> {
    match work {
        ModeLevelTailCameraStripeSub30194::BothTileAxesUnchanged => {
            Some(M68kCycleCount::from_raw(238))
        }
        ModeLevelTailCameraStripeSub30194::UpwardHorizontalNoWrap { cells }
            if cells
                .iter()
                .all(|cell| *cell == ModeLevelTailCameraCellMappingSub30194::OrdinaryTheme) =>
        {
            // ASM sub_30194/loc_301CE -> sub_3034A. The fixed unsplit
            // upward-row spine plus the traced 21 ordinary cells is 2,320
            // raw cycles. Priority/block mixtures remain unclassified until
            // an oracle instruction trace fixes their complete path cost.
            Some(M68kCycleCount::from_raw(2_320))
        }
        ModeLevelTailCameraStripeSub30194::UpwardHorizontalNoWrap { cells }
            if cells
                .iter()
                .filter(|cell| **cell == ModeLevelTailCameraCellMappingSub30194::Block)
                .count()
                == 1
                && cells
                    .iter()
                    .all(|cell| *cell != ModeLevelTailCameraCellMappingSub30194::PriorityTheme) =>
        {
            // ASM sub_30194/loc_3037C: oracle frame 902 takes one block
            // mapping branch after 20 ordinary cells. The complete caller,
            // jump stub, routine, and return cost 2,312 raw cycles.
            Some(M68kCycleCount::from_raw(2_312))
        }
        ModeLevelTailCameraStripeSub30194::UpwardHorizontalNoWrap { cells }
            if cells
                .iter()
                .filter(|cell| **cell == ModeLevelTailCameraCellMappingSub30194::Block)
                .count()
                == 14
                && cells
                    .iter()
                    .all(|cell| *cell != ModeLevelTailCameraCellMappingSub30194::PriorityTheme) =>
        {
            // ASM sub_30194 -> sub_3034A, published TAS frame 1304. The
            // ordered full-PC journal fixes 2,156 raw cycles for seven theme
            // cells and fourteen loc_3037C block-mapping cells.
            Some(M68kCycleCount::from_raw(2_156))
        }
        ModeLevelTailCameraStripeSub30194::DownwardHorizontalNoWrap { cells }
            if cells
                .iter()
                .all(|cell| *cell == ModeLevelTailCameraCellMappingSub30194::OrdinaryTheme) =>
        {
            // ASM sub_30194 loc_301B4 -> loc_301CE -> sub_3034A: frame 1004
            // takes the same 21 ordinary cells as frame 895's upward pass.
            // The complete frame-1004 selected-PC path is 2,350 raw cycles;
            // FIFO retry cycles remain owned by the ordered VDP scheduler.
            Some(M68kCycleCount::from_raw(2_350))
        }
        ModeLevelTailCameraStripeSub30194::RightwardVerticalNoWrap { cells }
            if cells
                .iter()
                .all(|cell| *cell != ModeLevelTailCameraCellMappingSub30194::PriorityTheme) =>
        {
            // ASM loc_892 -> sub_30194 loc_3023C -> loc_30262 -> sub_302F6.
            // The published TAS frame-1214 full-PC journal sums to 2,114
            // raw cycles from sub_30194 entry through the first loc_302E8
            // latch write. Add the caller JSR/jump stub and remaining latch
            // write/RTS for the complete loc_892 call.
            let block_cells = cells
                .iter()
                .filter(|cell| **cell == ModeLevelTailCameraCellMappingSub30194::Block)
                .count();
            Some(M68kCycleCount::from_raw(
                2_180 - 12 * u32::try_from(block_cells).ok()?,
            ))
        }
        ModeLevelTailCameraStripeSub30194::RightwardVerticalNoWrap { .. } => None,
        ModeLevelTailCameraStripeSub30194::LeftwardVerticalNoWrap { cells }
            if cells
                .iter()
                .all(|cell| *cell != ModeLevelTailCameraCellMappingSub30194::PriorityTheme) =>
        {
            let block_cells = cells
                .iter()
                .filter(|cell| **cell == ModeLevelTailCameraCellMappingSub30194::Block)
                .count();
            // ASM loc_3023C (kid.asm:33819) takes `blt.s loc_30262` (10)
            // instead of the rightward `blt` untaken + `addi.w #$28,d6` +
            // `cmp.w (Level_width_tiles).w,d6` + `bcc.w` untaken (8+8+12+12),
            // 30 cycles below the equivalent rightward column path.
            Some(M68kCycleCount::from_raw(
                2_150 - 12 * u32::try_from(block_cells).ok()?,
            ))
        }
        ModeLevelTailCameraStripeSub30194::LeftwardVerticalNoWrap { .. } => None,
        ModeLevelTailCameraStripeSub30194::UpwardHorizontalRightwardVertical {
            row_cells,
            column_cells,
            first_row_pass_cells,
            first_column_pass_cells,
        } if row_cells
            .iter()
            .chain(column_cells.iter())
            .all(|cell| *cell != ModeLevelTailCameraCellMappingSub30194::PriorityTheme) =>
        {
            // ASM loc_892 -> sub_30194, published TAS frame 1218. The full
            // PC journals fix 4,252 raw cycles when the 15-cell column is
            // unsplit (frame 1299), or 4,336 when it wraps and needs two
            // sub_302F6 calls (frame 1218).
            let block_cells = row_cells
                .iter()
                .chain(column_cells.iter())
                .filter(|cell| **cell == ModeLevelTailCameraCellMappingSub30194::Block)
                .count();
            let base = if first_column_pass_cells == 15 {
                4_252
            } else {
                4_336
            } + if first_row_pass_cells == 21 { 0 } else { 88 };
            Some(M68kCycleCount::from_raw(
                base - 12 * u32::try_from(block_cells).ok()?,
            ))
        }
        ModeLevelTailCameraStripeSub30194::UpwardHorizontalRightwardVertical { .. } => None,
        ModeLevelTailCameraStripeSub30194::UpwardHorizontalSplit { cells, .. }
            if cells
                .iter()
                .all(|cell| *cell != ModeLevelTailCameraCellMappingSub30194::PriorityTheme) =>
        {
            // ASM loc_301F6 -> sub_3034A twice. The published TAS frame
            // 1334 full-PC journal fixes the 20+1 ordinary-cell path at
            // 2,398 raw cycles. Each loc_3037C block cell omits 12 cycles.
            let block_cells = cells
                .iter()
                .filter(|cell| **cell == ModeLevelTailCameraCellMappingSub30194::Block)
                .count();
            Some(M68kCycleCount::from_raw(
                2_398 - 12 * u32::try_from(block_cells).ok()?,
            ))
        }
        ModeLevelTailCameraStripeSub30194::UpwardHorizontalSplit { .. } => None,
        ModeLevelTailCameraStripeSub30194::StreamingOther => None,
        ModeLevelTailCameraStripeSub30194::UpwardHorizontalNoWrap { .. }
        | ModeLevelTailCameraStripeSub30194::DownwardHorizontalNoWrap { .. } => None,
    }
}

/// One of the 19 cached/new comparisons in ASM
/// `BackgroundScroll_ComputeShiftData loc_30452/loc_3048C`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ModeLevelTailShiftDataSlotLoc30452 {
    Unchanged,
    OldGreaterThanNew,
    OldLessThanNew { width_subtractions: u16 },
}

/// Canonical `4(a0)` words for the 19 ASM `Horiz_Scroll_Data` slots.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ModeLevelTailShiftDataStateLoc30452 {
    previous_scroll_words: [u16; 19],
}

/// Pure state transition for ASM `BackgroundScroll_ComputeShiftData`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ModeLevelTailShiftDataAdvanceLoc30452 {
    pub work: [ModeLevelTailShiftDataSlotLoc30452; 19],
    pub next_state: ModeLevelTailShiftDataStateLoc30452,
}

/// Canonical WRAM state consumed by ASM `Mode_Level loc_892`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ModeLevelTailStateLoc892 {
    camera_x_latch_f82c: u16,
    camera_y_latch_f82e: u16,
    background_row_latch_faa8: i16,
    shift_data_loc_30452: ModeLevelTailShiftDataStateLoc30452,
    pending_palette_0_mask_f888: u32,
    pending_palette_1_mask_f88c: u32,
    background_upload_busy_fb49: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ModeLevelTailInputsLoc892<'a> {
    pub camera_x_pos: u16,
    pub camera_y_pos: u16,
    /// Live `Level_Layout` words selected for an unsplit horizontal
    /// `sub_30194` upload. The imperative shell gathers these from the current
    /// block grid; the timing core classifies their ASM mapping branches.
    pub camera_stripe_cells_sub_30194: Option<&'a [u16]>,
    /// Live 15-cell column selected by ASM `loc_30262 -> sub_302F6`.
    pub camera_column_cells_sub_30194: Option<&'a [u16]>,
    pub time_frames_before_loc_3038a: u16,
    pub game_mode_zero: bool,
    pub special_effect_4: bool,
    pub background_width: u16,
    pub background_format_nonzero: bool,
    pub background_no_scroll: bool,
    pub background_theme: u16,
    pub bgscroll_profile: &'a [u8],
    pub bgscroll_has_terminator: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ModeLevelTailAdvanceLoc892 {
    pub next_state: ModeLevelTailStateLoc892,
    pub next_time_frames: u16,
    pub work: Option<ModeLevelTailWorkLoc892>,
}

impl ModeLevelTailStateLoc892 {
    /// ASM `GameProgram loc_5FE/loc_604` and `loc_720` clear these WRAM words.
    pub const fn power_on_zero() -> Self {
        Self {
            camera_x_latch_f82c: 0,
            camera_y_latch_f82e: 0,
            background_row_latch_faa8: 0,
            shift_data_loc_30452: ModeLevelTailShiftDataStateLoc30452::from_previous_scroll_words(
                [0; 19],
            ),
            pending_palette_0_mask_f888: 0,
            pending_palette_1_mask_f88c: 0,
            background_upload_busy_fb49: 0,
        }
    }

    /// ASM `loc_720`: clear the shared gameplay RAM range through `$FBC0`.
    pub fn clear_loc_720(&mut self) {
        *self = Self::power_on_zero();
    }

    /// ASM `loc_2FFE8`: latch camera/row state and run the initial ShiftData pass.
    pub fn preload_loc_2ffe8(
        &mut self,
        camera_x_pos: u16,
        camera_y_pos: u16,
        level_height_pixels: u16,
        time_frames: u16,
    ) {
        self.camera_x_latch_f82c = camera_x_pos;
        self.camera_y_latch_f82e = camera_y_pos;
        // ASM loc_3001E/loc_30054/loc_30066: the lower preload window starts
        // at d7-$380, may receive the extra -$40 bias, and loses one DBF
        // iteration when d7+$E0 reaches Level_height_pixels. sub_307D8 leaves
        // FAA8 at the final visited row before loc_30090 restores d7.
        let original_y = camera_y_pos as i16;
        let mut loop_counter = 0x001C_i16;
        if original_y.wrapping_add(0x00E0) >= level_height_pixels as i16 {
            loop_counter = loop_counter.wrapping_sub(1);
        }
        let mut preload_y = original_y.wrapping_sub(0x0380);
        if preload_y != i16::from_be_bytes([0xFC, 0x80]) {
            preload_y = preload_y.wrapping_sub(0x0040);
            loop_counter = loop_counter.wrapping_add(2);
        }
        self.background_row_latch_faa8 =
            preload_y.wrapping_add(loop_counter.wrapping_mul(0x0020)) >> 5;
        if original_y == 0 {
            // ASM loc_3007A/loc_30090: a zero restored d7 clears FAA8.
            self.background_row_latch_faa8 = 0;
        }
        self.shift_data_loc_30452 =
            ModeLevelTailShiftDataStateLoc30452::from_preload_loc_3001e(camera_x_pos, time_frames);
    }

    pub const fn camera_latches_f82c_f82e(&self) -> (u16, u16) {
        (self.camera_x_latch_f82c, self.camera_y_latch_f82e)
    }

    pub const fn background_row_latch_faa8(&self) -> i16 {
        self.background_row_latch_faa8
    }

    pub const fn shift_data_loc_30452(&self) -> &ModeLevelTailShiftDataStateLoc30452 {
        &self.shift_data_loc_30452
    }

    pub const fn pending_palette_masks_f888_f88c(&self) -> (u32, u32) {
        (
            self.pending_palette_0_mask_f888,
            self.pending_palette_1_mask_f88c,
        )
    }

    /// ASM `sub_73D0/loc_7414` and other palette workers write the shared
    /// `$FFFFF888/$FFFFF88C` masks before `Mode_Level loc_892` consumes them.
    pub fn set_pending_palette_masks_f888_f88c(
        &mut self,
        pending_palette_0_mask: u32,
        pending_palette_1_mask: u32,
    ) {
        self.pending_palette_0_mask_f888 = pending_palette_0_mask;
        self.pending_palette_1_mask_f88c = pending_palette_1_mask;
    }

    pub const fn background_upload_busy_fb49(&self) -> u8 {
        self.background_upload_busy_fb49
    }

    /// Gather/decide transition for ASM `Mode_Level loc_892`.
    pub fn advance_loc_892(
        self,
        inputs: ModeLevelTailInputsLoc892<'_>,
    ) -> ModeLevelTailAdvanceLoc892 {
        let mut next_state = self;
        let camera_stripe_sub_30194 = mode_level_tail_camera_stripe_work_sub_30194(
            self.camera_x_latch_f82c,
            self.camera_y_latch_f82e,
            inputs.camera_x_pos,
            inputs.camera_y_pos,
            inputs.camera_stripe_cells_sub_30194,
            inputs.camera_column_cells_sub_30194,
        );
        // ASM sub_30194 loc_302E8 updates both latches after either axis path.
        next_state.camera_x_latch_f82c = inputs.camera_x_pos;
        next_state.camera_y_latch_f82e = inputs.camera_y_pos;

        let palette_sub_14c0 = match (
            self.pending_palette_0_mask_f888,
            self.pending_palette_1_mask_f88c,
        ) {
            (0, 0) => ModeLevelTailPaletteWorkSub14C0::Inactive,
            (u32::MAX, u32::MAX) => ModeLevelTailPaletteWorkSub14C0::BothMasksFull,
            _ => ModeLevelTailPaletteWorkSub14C0::ActiveOther,
        };
        // ASM sub_14C0 consumes both shared masks before returning.
        next_state.pending_palette_0_mask_f888 = 0;
        next_state.pending_palette_1_mask_f88c = 0;

        if inputs.special_effect_4 {
            return ModeLevelTailAdvanceLoc892 {
                next_state,
                next_time_frames: inputs.time_frames_before_loc_3038a,
                work: None,
            };
        }

        let next_time_frames = if inputs.game_mode_zero {
            inputs.time_frames_before_loc_3038a
        } else {
            inputs.time_frames_before_loc_3038a.wrapping_add(1)
        };
        let Some(shift_advance) = self.shift_data_loc_30452.advance_loc_30452(
            inputs.camera_x_pos,
            next_time_frames,
            inputs.background_width,
        ) else {
            return ModeLevelTailAdvanceLoc892 {
                next_state,
                next_time_frames,
                work: None,
            };
        };
        next_state.shift_data_loc_30452 = shift_advance.next_state;

        let shift_layers = mode_level_tail_shift_layers_work(
            inputs.camera_y_pos,
            inputs.background_no_scroll,
            inputs.background_theme,
        );

        let background_upload_busy_fb49 = self.background_upload_busy_fb49 != 0;
        let mut piece_rows_sub_30744 = ModeLevelTailPieceRowsSub30744 {
            entries_before_sentinel: 0,
            rendered_nonnegative_entries: 0,
            rendered_entries: [false; 29],
        };
        let mut piece_refresh_sub_307d8 = ModeLevelTailPieceRefreshSub307D8::StreamingOther;
        let mut profile_classified = true;
        if !background_upload_busy_fb49 && !inputs.background_format_nonzero {
            let camera_row = usize::from(inputs.camera_y_pos >> 5);
            for row in 0..29_usize {
                let profile_index = camera_row + row;
                let Some(&selector) = inputs.bgscroll_profile.get(profile_index) else {
                    if !inputs.bgscroll_has_terminator {
                        profile_classified = false;
                    }
                    break;
                };
                if selector == 0xFF {
                    break;
                }
                if !selector.is_multiple_of(8) || usize::from(selector / 8) >= 19 {
                    profile_classified = false;
                    break;
                }
                piece_rows_sub_30744.entries_before_sentinel += 1;
                if shift_advance.work[usize::from(selector / 8)]
                    != ModeLevelTailShiftDataSlotLoc30452::Unchanged
                {
                    piece_rows_sub_30744.rendered_nonnegative_entries += 1;
                    piece_rows_sub_30744.rendered_entries[row] = true;
                }
            }

            let current_row = (inputs.camera_y_pos as i16) >> 5;
            piece_refresh_sub_307d8 = if self.background_row_latch_faa8 == current_row {
                ModeLevelTailPieceRefreshSub307D8::CachedRowUnchanged
            } else {
                mode_level_tail_piece_refresh_work_sub_307d8(
                    current_row,
                    self.background_row_latch_faa8,
                    inputs.background_width,
                    inputs.bgscroll_profile,
                    &shift_advance.next_state.previous_scroll_words,
                )
            };
            next_state.background_row_latch_faa8 = current_row;
        }

        let work = profile_classified.then_some(ModeLevelTailWorkLoc892 {
            camera_stripe_sub_30194,
            background_loc_3038a: ModeLevelTailBackgroundWorkLoc3038A {
                special_effect_4: false,
                game_mode_zero: inputs.game_mode_zero,
                background_upload_busy_fb49,
                background_format_nonzero: inputs.background_format_nonzero,
                shift_data_slots_loc_30452: shift_advance.work,
                shift_layers,
                piece_rows_sub_30744,
                piece_refresh_sub_307d8,
            },
            palette_sub_14c0,
        });

        ModeLevelTailAdvanceLoc892 {
            next_state,
            next_time_frames,
            work,
        }
    }
}

impl ModeLevelTailShiftDataStateLoc30452 {
    pub const fn from_previous_scroll_words(previous_scroll_words: [u16; 19]) -> Self {
        Self {
            previous_scroll_words,
        }
    }

    /// Seed the words written by the Load_InGame preload at ASM `loc_3001E`.
    pub fn from_preload_loc_3001e(camera_x_pos: u16, time_frames: u16) -> Self {
        Self::from_camera_and_time_loc_30452(camera_x_pos, time_frames)
    }

    /// Build the words written by one ASM `BackgroundScroll_ComputeShiftData`.
    pub fn from_camera_and_time_loc_30452(camera_x_pos: u16, time_frames: u16) -> Self {
        Self {
            previous_scroll_words: shift_data_words_loc_30452(camera_x_pos, time_frames),
        }
    }

    pub const fn previous_scroll_words(&self) -> &[u16; 19] {
        &self.previous_scroll_words
    }

    /// Run both ASM `loc_30452` loops and return the next cached words.
    pub fn advance_loc_30452(
        self,
        camera_x_pos: u16,
        time_frames: u16,
        background_width: u16,
    ) -> Option<ModeLevelTailShiftDataAdvanceLoc30452> {
        let background_width = i16::try_from(background_width).ok()?;
        if background_width <= 0 {
            return None;
        }

        let next_words = shift_data_words_loc_30452(camera_x_pos, time_frames);
        let mut work = [ModeLevelTailShiftDataSlotLoc30452::Unchanged; 19];
        for slot_index in 0..19 {
            let previous = self.previous_scroll_words[slot_index];
            let next = next_words[slot_index];
            let (previous_comparison, next_comparison) = if slot_index < 2 {
                (previous >> 3, next >> 3)
            } else {
                (previous, next)
            };
            if previous_comparison == next_comparison {
                continue;
            }

            work[slot_index] = if (previous_comparison as i16) > (next_comparison as i16) {
                ModeLevelTailShiftDataSlotLoc30452::OldGreaterThanNew
            } else {
                // ASM loc_304C2: rightward/new-greater work starts at the
                // newly exposed viewport edge, then repeatedly subtracts
                // Background_width until the row fits.
                let mut row = (next as i16 >> 3).wrapping_add(0x0028);
                let mut width_subtractions = 0_u16;
                while background_width <= row {
                    row = row.wrapping_sub(background_width);
                    width_subtractions = width_subtractions.wrapping_add(1);
                }
                ModeLevelTailShiftDataSlotLoc30452::OldLessThanNew { width_subtractions }
            };
        }

        Some(ModeLevelTailShiftDataAdvanceLoc30452 {
            work,
            next_state: Self {
                previous_scroll_words: next_words,
            },
        })
    }
}

fn shift_data_words_loc_30452(camera_x_pos: u16, time_frames: u16) -> [u16; 19] {
    let camera_tile = camera_x_pos >> 3;
    let mut words = [0_u16; 19];
    words[0] = time_frames.wrapping_add(camera_tile);
    words[1] = (time_frames >> 1).wrapping_add(camera_tile);

    // ASM loc_3048C/loc_304A8: `d5` starts at zero. Each loop exposes its
    // high word with SWAP, stores it, restores the accumulator ordering, then
    // adds sign-extended `Camera_X_pos << 12`.
    let phase_step = (i32::from(camera_x_pos as i16) as u32).wrapping_shl(12);
    let mut phase = 0_u32;
    for word in &mut words[2..] {
        *word = (phase >> 16) as u16;
        phase = phase.wrapping_add(phase_step);
    }
    words
}

/// ASM `BackgroundScroll_ComputeShiftLayers` source path.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ModeLevelTailShiftLayersWork {
    /// ASM loc_3050A/loc_3056A: aligned 8-pixel rows without a theme ripple.
    NormalAlignedNoRipple {
        full_rows: u8,
    },
    /// ASM loc_3050A/loc_30562/loc_3056A: split top and bottom partial rows.
    NormalUnalignedNoRipple {
        leading_rows: u8,
        full_rows: u8,
        trailing_rows: u8,
    },
    Other,
}

/// Classify ASM `BackgroundScroll_ComputeShiftLayers` row-loop work.
pub fn mode_level_tail_shift_layers_work(
    camera_y_pos: u16,
    background_no_scroll: bool,
    background_theme: u16,
) -> ModeLevelTailShiftLayersWork {
    if background_no_scroll || matches!(background_theme, 5 | 9) {
        return ModeLevelTailShiftLayersWork::Other;
    }

    let row_phase = ((camera_y_pos >> 2) & 7) as u8;
    if row_phase == 0 {
        ModeLevelTailShiftLayersWork::NormalAlignedNoRipple { full_rows: 28 }
    } else {
        ModeLevelTailShiftLayersWork::NormalUnalignedNoRipple {
            leading_rows: 8 - row_phase,
            full_rows: 27,
            trailing_rows: row_phase,
        }
    }
}

/// Raw source cycles for the classified shift-layer row loops.
pub fn mode_level_tail_shift_layers_raw_cycles(
    work: ModeLevelTailShiftLayersWork,
) -> Option<M68kCycleCount> {
    match work {
        ModeLevelTailShiftLayersWork::NormalAlignedNoRipple { full_rows: 28 } => {
            Some(M68kCycleCount::from_raw(4_020))
        }
        ModeLevelTailShiftLayersWork::NormalUnalignedNoRipple {
            leading_rows,
            full_rows: 27,
            trailing_rows,
        } if leading_rows != 0
            && trailing_rows != 0
            && leading_rows.checked_add(trailing_rows) == Some(8) =>
        {
            Some(M68kCycleCount::from_raw(4_136))
        }
        _ => None,
    }
}

/// ASM `sub_30744` piece-row scan work before its `$FF` sentinel.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct ModeLevelTailPieceRowsSub30744 {
    pub entries_before_sentinel: u8,
    pub rendered_nonnegative_entries: u8,
    /// ASM `loc_3079A`: true when the row's shift-data longword is
    /// non-negative and the sprite words are emitted.
    pub rendered_entries: [bool; 29],
}

/// ASM `sub_307D8` background-piece refresh path.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ModeLevelTailPieceRefreshSub307D8 {
    /// ASM loc_30810 -> loc_308B4: current and cached background rows match.
    CachedRowUnchanged,
    /// ASM loc_3081A -> loc_308A6: the 41-word row crosses neither source nor
    /// plane-B boundary.
    LinearNoWrap {
        width_subtractions: u16,
        copied_words: u8,
    },
    /// ASM loc_3080E -> loc_3081A -> loc_308A6: downward row with neither
    /// source nor Plane-B wrap.
    DownwardLinearNoWrap {
        width_subtractions: u16,
        copied_words: u8,
    },
    /// ASM loc_308DE: downward row whose source stays linear while Plane B
    /// wraps at its 64-cell boundary.
    DownwardPlaneWrapOnly {
        width_subtractions: u16,
        copied_words: u8,
        second_segment_words: u8,
    },
    /// ASM loc_308DE: upward row whose source stays linear while Plane B
    /// wraps at its 64-cell boundary.
    UpwardPlaneWrapOnly {
        width_subtractions: u16,
        copied_words: u8,
        second_segment_words: u8,
    },
    StreamingOther,
}

fn mode_level_tail_piece_refresh_work_sub_307d8(
    current_camera_row: i16,
    cached_camera_row: i16,
    background_width: u16,
    bgscroll_profile: &[u8],
    horiz_scroll_words: &[u16; 19],
) -> ModeLevelTailPieceRefreshSub307D8 {
    if current_camera_row < 0 || background_width == 0 {
        return ModeLevelTailPieceRefreshSub307D8::StreamingOther;
    }
    let moving_down = cached_camera_row < current_camera_row;
    let source_row = if moving_down {
        current_camera_row.wrapping_add(0x001C)
    } else {
        current_camera_row
    };
    let Ok(source_row) = usize::try_from(source_row) else {
        return ModeLevelTailPieceRefreshSub307D8::StreamingOther;
    };
    let Some(&selector) = bgscroll_profile.get(source_row) else {
        return ModeLevelTailPieceRefreshSub307D8::StreamingOther;
    };
    if selector == 0xFF || !selector.is_multiple_of(8) {
        return ModeLevelTailPieceRefreshSub307D8::StreamingOther;
    }
    let Some(&scroll_word) = horiz_scroll_words.get(usize::from(selector / 8)) else {
        return ModeLevelTailPieceRefreshSub307D8::StreamingOther;
    };

    // ASM loc_30836 reduces the coarse source column by Background_width.
    let mut reduced_column = scroll_word >> 3;
    let mut width_subtractions = 0_u16;
    while background_width <= reduced_column {
        reduced_column = reduced_column.wrapping_sub(background_width);
        width_subtractions = width_subtractions.wrapping_add(1);
    }

    // ASM loc_30840/loc_3086A: both source and 64-cell plane windows must
    // contain the full d4=$28 inclusive copy for loc_308A6.
    let source_wraps = background_width <= reduced_column.wrapping_add(0x0028);
    let plane_column = (scroll_word >> 3) & 0x003F;
    let plane_wraps = plane_column.wrapping_add(0x0028) > 0x003F;
    if !source_wraps && plane_wraps {
        let second_segment_words =
            u8::try_from(plane_column.wrapping_add(0x0028).wrapping_sub(0x003F)).unwrap_or(0);
        if moving_down {
            ModeLevelTailPieceRefreshSub307D8::DownwardPlaneWrapOnly {
                width_subtractions,
                copied_words: 41,
                second_segment_words,
            }
        } else {
            ModeLevelTailPieceRefreshSub307D8::UpwardPlaneWrapOnly {
                width_subtractions,
                copied_words: 41,
                second_segment_words,
            }
        }
    } else if source_wraps || plane_wraps {
        ModeLevelTailPieceRefreshSub307D8::StreamingOther
    } else if moving_down {
        ModeLevelTailPieceRefreshSub307D8::DownwardLinearNoWrap {
            width_subtractions,
            copied_words: 41,
        }
    } else {
        ModeLevelTailPieceRefreshSub307D8::LinearNoWrap {
            width_subtractions,
            copied_words: 41,
        }
    }
}

/// Raw source cycles for the classified ASM `sub_307D8` call.
pub fn mode_level_tail_piece_refresh_raw_cycles_sub_307d8(
    work: ModeLevelTailPieceRefreshSub307D8,
) -> Option<M68kCycleCount> {
    match work {
        ModeLevelTailPieceRefreshSub307D8::CachedRowUnchanged => {
            Some(M68kCycleCount::from_raw(218))
        }
        ModeLevelTailPieceRefreshSub307D8::LinearNoWrap {
            width_subtractions: 0,
            copied_words: 41,
        } => {
            // ASM sub_307D8/loc_308A6: the traced zero-subtraction linear
            // copy is 2,360 raw cycles. Other widths remain unclassified
            // until their complete path is fixed by an oracle trace.
            Some(M68kCycleCount::from_raw(2_360))
        }
        ModeLevelTailPieceRefreshSub307D8::DownwardLinearNoWrap {
            width_subtractions: 0,
            copied_words: 41,
        } => {
            // ASM sub_307D8 loc_3080E -> loc_3081A: frame 1004's untaken
            // BGT plus ADDI #$1C adds six raw cycles to the existing 2,360
            // no-wrap copy. VDP FIFO retries and its VINT stay external.
            Some(M68kCycleCount::from_raw(2_366))
        }
        ModeLevelTailPieceRefreshSub307D8::DownwardPlaneWrapOnly {
            width_subtractions: 0,
            copied_words: 41,
            second_segment_words: 1..=40,
        } => {
            // ASM sub_307D8 -> loc_308DE (kid.asm:34737): the wrap branch
            // replaces the linear tail's `tst/bne/moveq` + 41-write loop with
            // the `bne.s loc_308DE` taken path, the five-instruction segment
            // seed, the same 41 writes split across two destinations, and the
            // inter-segment control-port rewrite: +56 raw over the downward
            // linear spine, direction-independent. VDP FIFO retries and its
            // VINT stay external, exactly as in the linear arms.
            Some(M68kCycleCount::from_raw(2_366 + 56))
        }
        ModeLevelTailPieceRefreshSub307D8::UpwardPlaneWrapOnly {
            width_subtractions: 0,
            copied_words: 41,
            second_segment_words: 1..=40,
        } => {
            // ASM sub_307D8 -> loc_308DE (kid.asm:34737): the same +56 wrap
            // restructure composed over the upward linear spine.
            Some(M68kCycleCount::from_raw(2_360 + 56))
        }
        ModeLevelTailPieceRefreshSub307D8::StreamingOther => None,
        ModeLevelTailPieceRefreshSub307D8::LinearNoWrap { .. }
        | ModeLevelTailPieceRefreshSub307D8::DownwardLinearNoWrap { .. }
        | ModeLevelTailPieceRefreshSub307D8::DownwardPlaneWrapOnly { .. }
        | ModeLevelTailPieceRefreshSub307D8::UpwardPlaneWrapOnly { .. } => None,
    }
}

/// ASM `sub_14C0` palette work at the end of `Mode_Level`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ModeLevelTailPaletteWorkSub14C0 {
    /// Both pending palette masks `$FFFFF888/$FFFFF88C` are clear.
    Inactive,
    /// Both masks are `$FFFFFFFF`; ASM `sub_1536` visits all 64 words.
    BothMasksFull,
    /// Another non-zero mask combination not yet source-timed.
    ActiveOther,
}

/// Raw source cycles for the classified ASM `sub_14C0` call.
pub fn mode_level_tail_palette_raw_cycles_sub_14c0(
    work: ModeLevelTailPaletteWorkSub14C0,
) -> Option<M68kCycleCount> {
    match work {
        // ASM loc_892 BSR.W plus sub_14C0's zero-mask return.
        ModeLevelTailPaletteWorkSub14C0::Inactive => Some(M68kCycleCount::from_raw(82)),
        // ASM sub_14C0/loc_14CE -> sub_1536 twice. Oracle frame 902 fixes the
        // complete BSR.W call at 19,848 raw cycles for two full masks.
        ModeLevelTailPaletteWorkSub14C0::BothMasksFull => Some(M68kCycleCount::from_raw(19_848)),
        ModeLevelTailPaletteWorkSub14C0::ActiveOther => None,
    }
}

/// Source-typed work from ASM `Mode_Level loc_892` to the first
/// `WaitForVint $93E` poll.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct ModeLevelTailWorkLoc892 {
    pub camera_stripe_sub_30194: ModeLevelTailCameraStripeSub30194,
    pub background_loc_3038a: ModeLevelTailBackgroundWorkLoc3038A,
    pub palette_sub_14c0: ModeLevelTailPaletteWorkSub14C0,
}

/// ASM `loc_3038A` gates and background-scroll work.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct ModeLevelTailBackgroundWorkLoc3038A {
    pub special_effect_4: bool,
    pub game_mode_zero: bool,
    pub background_upload_busy_fb49: bool,
    pub background_format_nonzero: bool,
    pub shift_data_slots_loc_30452: [ModeLevelTailShiftDataSlotLoc30452; 19],
    pub shift_layers: ModeLevelTailShiftLayersWork,
    pub piece_rows_sub_30744: ModeLevelTailPieceRowsSub30744,
    pub piece_refresh_sub_307d8: ModeLevelTailPieceRefreshSub307D8,
}

/// Raw source cycles from ASM `Mode_Level loc_892` to the first
/// `WaitForVint $93E` poll.
///
/// Source-verified streaming variants contribute raw instruction cycles here.
/// Their ordered VDP waits, asynchronous GEMS steals, a possible VINT, ripple
/// rows, and unclassified palette masks remain outside this raw calculation.
pub fn mode_level_tail_raw_cycles_to_wait_for_vint_loc_892(
    work: &ModeLevelTailWorkLoc892,
) -> Option<M68kCycleCount> {
    let camera_stripe_cycles =
        mode_level_tail_camera_stripe_raw_cycles_sub_30194(work.camera_stripe_sub_30194)?.get();
    let palette_cycles = mode_level_tail_palette_raw_cycles_sub_14c0(work.palette_sub_14c0)?.get();

    let background = &work.background_loc_3038a;
    if background.special_effect_4
        || background.game_mode_zero
        || background.background_upload_busy_fb49
        || background.background_format_nonzero
        || background.piece_rows_sub_30744.entries_before_sentinel != 29
        || background.piece_rows_sub_30744.rendered_nonnegative_entries
            > background.piece_rows_sub_30744.entries_before_sentinel
    {
        return None;
    }
    let shift_layers_extra = mode_level_tail_shift_layers_raw_cycles(background.shift_layers)?
        .get()
        .checked_sub(4_020)?;
    let piece_refresh_extra =
        mode_level_tail_piece_refresh_raw_cycles_sub_307d8(background.piece_refresh_sub_307d8)?
            .get()
            .checked_sub(218)?;

    // ASM BackgroundScroll_ComputeShiftData: 2,040 covers all 19 unchanged
    // slots. loc_304B4's old>new path adds 114; old<new adds 120 plus 26
    // for every loc_304C2 background-width subtraction.
    let mut shift_data = 2_040_u32;
    for slot in background.shift_data_slots_loc_30452 {
        let extra = match slot {
            ModeLevelTailShiftDataSlotLoc30452::Unchanged => 0,
            ModeLevelTailShiftDataSlotLoc30452::OldGreaterThanNew => 114,
            ModeLevelTailShiftDataSlotLoc30452::OldLessThanNew { width_subtractions } => {
                120_u32.checked_add(26_u32.checked_mul(u32::from(width_subtractions))?)?
            }
        };
        shift_data = shift_data.checked_add(extra)?;
    }

    // ASM loc_3038A: the 6,800-cycle aligned fixed spine surrounds ShiftData,
    // ShiftLayers (4,020 aligned; +116 for split partial rows), sub_30744's
    // 29-entry scan (2,360 + 82 per rendered row), and cached-row sub_307D8
    // (218; streaming work is added relative to it).
    let background_cycles = 6_800_u32
        .checked_add(shift_data)?
        .checked_add(shift_layers_extra)?
        .checked_add(piece_refresh_extra)?
        .checked_add(82_u32.checked_mul(u32::from(
            background.piece_rows_sub_30744.rendered_nonnegative_entries,
        ))?)?;

    // ASM loc_892: the classified sub_14C0 call is followed by the
    // Mode_Level return/MainGameLoop/WaitForVint prefix at 60 raw cycles.
    Some(M68kCycleCount::from_raw(
        camera_stripe_cycles
            .checked_add(background_cycles)?
            .checked_add(palette_cycles)?
            .checked_add(60)?,
    ))
}

/// Advance the no-external-wait ASM `loc_892 -> WaitForVint $93E` path
/// through the Genesis ROM-refresh recurrence.
pub fn mode_level_tail_advance_to_wait_for_vint_loc_892(
    phase: M68kRefreshPhase,
    work: &ModeLevelTailWorkLoc892,
) -> Option<M68kRefreshAdvance> {
    // ASM sub_30194 and sub_307D8 streaming writes can stall on the VDP FIFO
    // and sub_307D8 can cross VINT. Their ordered arbitration is scheduled by
    // the persistent hardware shell, not flattened into ROM-refresh time.
    if work.camera_stripe_sub_30194 != ModeLevelTailCameraStripeSub30194::BothTileAxesUnchanged
        || work.background_loc_3038a.piece_refresh_sub_307d8
            != ModeLevelTailPieceRefreshSub307D8::CachedRowUnchanged
    {
        return None;
    }
    Some(phase.advance_rom(mode_level_tail_raw_cycles_to_wait_for_vint_loc_892(work)?))
}

/// Saved-PC path resumed by ASM `V_Int $488` before `WaitForVint` returns.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WaitForVintResumePath {
    /// Saved PC `$93E`: re-read `V_Int_Done`, then the branch at `$942` falls through.
    PollTestLoc93e,
    /// Saved PC `$942`: take one stale branch before re-reading `V_Int_Done`.
    StaleBranchLoc942,
}

/// Hardware arbitration after entry to ASM `V_Int $488` and before `Mode_Level $7C8`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WaitForVintArbitration {
    pub refresh_wait_events: u16,
    pub z80_68k_bus_steals: u16,
}

/// Instruction-boundary state when VINT becomes sampleable in `WaitForVint`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WaitForVintInterruptSample {
    /// Master clocks from VDP VINT assertion to the next 68k interrupt sample.
    ///
    /// This retains the partial instruction phase that an integer CPU-cycle
    /// count cannot represent.
    pub mclk_until_interrupt_sample: u32,
    /// Refresh waits processed during interrupt recognition/autovector entry,
    /// before ASM `V_Int $488`.
    pub interrupt_entry_refresh_wait_events: u16,
    pub resume: WaitForVintResumePath,
    pub arbitration_after_handler_entry: WaitForVintArbitration,
}

/// Raw source cycles from entry to ASM `V_Int $488` through `Mode_Level $7C8`.
pub const fn wait_for_vint_source_cycles_to_mode_level(
    resume: WaitForVintResumePath,
) -> M68kCycleCount {
    // ASM V_Int $488: ADDQ.L + ST + RTE costs 60. WaitForVint $93E-$954
    // costs 42 when the saved PC re-reads the flag, or 52 when the saved PC
    // executes the stale BEQ once. MainLoop $738-$752 then costs 60 through
    // the GameModesArray dispatch and level-mode branch.
    let cycles = match resume {
        WaitForVintResumePath::PollTestLoc93e => 162,
        WaitForVintResumePath::StaleBranchLoc942 => 172,
    };
    M68kCycleCount::from_raw(cycles)
}

/// Elapsed cycles from ASM `V_Int $488` through `Mode_Level $7C8`.
pub fn wait_for_vint_elapsed_cycles_to_mode_level(
    resume: WaitForVintResumePath,
    arbitration: WaitForVintArbitration,
) -> Option<M68kCycleCount> {
    // jgenesis charges two cycles per ROM refresh and eleven per asynchronous
    // Z80 ownership of the 68k bus on this source path.
    let elapsed = wait_for_vint_source_cycles_to_mode_level(resume)
        .get()
        .checked_add(u32::from(arbitration.refresh_wait_events).checked_mul(2)?)?
        .checked_add(u32::from(arbitration.z80_68k_bus_steals).checked_mul(11)?)?;
    Some(M68kCycleCount::from_raw(elapsed))
}

/// Elapsed master clocks from VDP VINT assertion through `Mode_Level $7C8`.
pub fn wait_for_vint_mclk_from_vint_assertion_to_mode_level(
    sample: WaitForVintInterruptSample,
) -> Option<GenesisMclkCount> {
    // jgenesis samples the pending interrupt for 10 cycles, then performs a
    // 44-cycle autovector acknowledge/stack/vector sequence before ASM V_Int
    // $488. The fractional instruction residue before sampling is already in
    // master clocks and must not be rounded to a 68k cycle.
    const INTERRUPT_ENTRY_CYCLES: u32 = 10 + 44;
    const MCLK_PER_M68K_CYCLE: u32 = 7;

    let after_handler_entry = wait_for_vint_elapsed_cycles_to_mode_level(
        sample.resume,
        sample.arbitration_after_handler_entry,
    )?;
    let interrupt_entry_refresh_cycles =
        u32::from(sample.interrupt_entry_refresh_wait_events).checked_mul(2)?;
    let cpu_cycles = INTERRUPT_ENTRY_CYCLES
        .checked_add(interrupt_entry_refresh_cycles)?
        .checked_add(after_handler_entry.get())?;
    let elapsed = sample
        .mclk_until_interrupt_sample
        .checked_add(cpu_cycles.checked_mul(MCLK_PER_M68K_CYCLE)?)?;
    Some(GenesisMclkCount::from_raw(elapsed))
}

/// Raw source cycles from ASM `Mode_Level $7C8` to `Execute_Objects sub_E12`.
///
/// The currently typed path is the natural published-gameplay path. Returning
/// `None` for another branch prevents its VDP delay from being mistaken for
/// source work.
pub fn mode_level_vblank_source_cycles_before_sub_e12(
    work: ModeLevelVblankWorkBeforeSubE12,
) -> Option<M68kCycleCount> {
    let dynamic_art_source_cycles = match work.dynamic_art_tiles_loc_ad4 {
        // ASM sub_A5C/loc_AD4: the exact source visitor measures 1,398 raw
        // cycles for the repeated `[16,4]` queue and 822 for an empty queue.
        Some([16, 4, 0, 0, 0, 0]) => 1_398,
        None => 822,
        _ => return None,
    };
    if work.scroll_path != ModeLevelScrollTransferPath::NormalLoc300ba
        || work.foreground_custom_queue_entries_sub_1596 != 0
        || !work.foreground_cell_queue_sub_1596.is_empty()
        || work.paused_sub_6874
    {
        return None;
    }

    // ASM Mode_Level (kid.asm:901-908), with branches in sub_A5C, sub_970,
    // loc_300BA, sub_1596, sub_6874, and sub_BDC0. Stable life/diamond
    // counters skip 110 raw cycles of calc_display_number work after the
    // first gameplay pass, but the later loc_BE20 -> loc_BE0C -> loc_BE34 ->
    // loc_BE4C hitpoint path adds the same 110 cycles. VDP-owned one-cycle
    // scheduler re-entries are excluded and represented by the arbitration
    // value below.
    let raw_cycles = match work.hud_path_sub_bdc0 {
        ModeLevelHudBuildPath::LocBe20DirectInitialCounterUpdates => 3_415,
        ModeLevelHudBuildPath::LocBe20DirectStableCounters => 3_305,
        ModeLevelHudBuildPath::LocBe20ThenLocBe4cStableCounters => 3_415,
        ModeLevelHudBuildPath::Other => return None,
    };
    // ASM Palette_to_VRAM sub_970: the suppressed branch consumes 56 raw
    // cycles rather than the active branch's 362.
    let palette_suppression_saving = if work.palette_dma_sub_970 { 0 } else { 306 };
    Some(M68kCycleCount::from_raw(
        raw_cycles - palette_suppression_saving - (1_398 - dynamic_art_source_cycles),
    ))
}

/// Elapsed cycles for the typed Mode_Level VBlank path, including hardware
/// arbitration already processed before `Execute_Objects sub_E12`.
pub fn mode_level_vblank_elapsed_cycles_before_sub_e12(
    work: ModeLevelVblankWorkBeforeSubE12,
    arbitration: ModeLevelVblankArbitrationBeforeSubE12,
) -> Option<M68kCycleCount> {
    let elapsed = mode_level_vblank_source_cycles_before_sub_e12(work)?
        .get()
        .checked_add(arbitration.vdp_bus_halt_cycles)?
        .checked_add(arbitration.vdp_pending_wait_cycles)?
        .checked_add(arbitration.refresh_wait_cycles)?;
    Some(M68kCycleCount::from_raw(elapsed))
}

/// Advance the natural VBlank path through the final H-scroll DMA reset.
///
/// jgenesis `timing.rs::record_68k_instruction` resets its approximate
/// refresh counter whenever the VDP owns the 68k bus. The final such reset is
/// ASM `loc_30104`; no earlier Mode-entry refresh phase survives it.
pub fn mode_level_vblank_advance_before_sub_e12(
    work: ModeLevelVblankWorkBeforeSubE12,
    arbitration: ModeLevelVblankArbitrationBeforeSubE12,
) -> Option<M68kRefreshAdvance> {
    let elapsed = mode_level_vblank_elapsed_cycles_before_sub_e12(work, arbitration)?;
    if arbitration.vdp_bus_halt_cycles == 0 {
        return None;
    }

    let suffix = arbitration.refresh_suffix_after_final_vdp_ownership;
    let before = M68kRefreshPhase::new(0)?.advance_rom(M68kCycleCount::from_raw(
        suffix.raw_cycles_before_pending_wait,
    ));
    let pending = before
        .phase_after
        .advance_external_wait(M68kCycleCount::from_raw(suffix.pending_wait_cycles));
    let after = pending.phase_after.advance_rom(M68kCycleCount::from_raw(
        suffix.raw_cycles_after_pending_wait,
    ));
    let suffix_elapsed = before
        .elapsed
        .get()
        .checked_add(pending.elapsed.get())?
        .checked_add(after.elapsed.get())?;
    let suffix_refresh_cycles = before
        .refresh_events
        .checked_add(after.refresh_events)?
        .checked_mul(2)?;
    if !arbitration.refresh_wait_cycles.is_multiple_of(2)
        || suffix_elapsed > elapsed.get()
        || suffix.pending_wait_cycles > arbitration.vdp_pending_wait_cycles
        || suffix_refresh_cycles > arbitration.refresh_wait_cycles
    {
        return None;
    }

    Some(M68kRefreshAdvance {
        elapsed,
        phase_after: after.phase_after,
        refresh_events: arbitration.refresh_wait_cycles / 2,
    })
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

/// Raw cycles for `loc_102B8`'s finished-animation delete through `loc_E1E`.
pub fn prize_flash_finish_cycles_loc_102b8(
    work: ModeLevelPrizeFlashDeleteListWork,
) -> M68kCycleCount {
    // ASM loc_102B8 -> sub_F1A -> sub_E9A -> sub_1072. The three deletion
    // loops cost 52, 48 and 52 cycles per predecessor. A selected-PC trace of
    // published frame 1715 pins the rank-independent E1E-to-E1E path at 1,028
    // raw cycles (2,452 total minus ranks 12/8/8).
    M68kCycleCount(
        1_028
            + 52 * u32::from(work.object_delete_rank_loc_ea2)
            + 48 * u32::from(work.gfx_delete_rank_loc_10ba)
            + 52 * u32::from(work.gfx_category4_delete_rank_loc_1106),
    )
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
    if let Some(ModeLevelPlayerExecutionPath::WallClingNoButtonsLocAa22 {
        form: crate::PlayerForm::TheKid,
        side: crate::Direction::Right,
        wall_scan_hit_row,
        vertical,
        y_velocity_after,
        gfx_object_collision_clear: true,
        vertical_collision: ModeLevelVerticalCollisionPath::Clear,
        ceiling_scan_sub_914a,
        airborne_visual_mapping_sub_b270,
    }) = execute.executed_player_path.as_ref()
    {
        let gravity_extra = match vertical {
            ModeLevelAirborneVerticalPath::HeldJumpGravityLocB1e8 => 0,
            ModeLevelAirborneVerticalPath::DefaultGravityLocB1e0 => 8,
            _ => return None,
        };
        let gfx_scan = execute.player_gfx_terrain_scan?;
        if gfx_scan.bounds.left_col != gfx_scan.bounds.right_col
            || gfx_scan.first_collision.is_some()
        {
            return None;
        }
        let row_count = gfx_scan
            .bounds
            .bottom_row
            .checked_sub(gfx_scan.bounds.top_row)?
            .checked_add(1)?;
        if gfx_scan.tile_iterations != row_count {
            return None;
        }
        if *vertical == ModeLevelAirborneVerticalPath::DefaultGravityLocB1e0
            && ceiling_scan_sub_914a.is_none()
            && *airborne_visual_mapping_sub_b270
                == ModeLevelAirborneVisualMappingPathSubB270::DescendingState3
        {
            // ASM loc_AA22 -> sub_B168 loc_B1E0 -> sub_A4EE -> sub_902A
            // -> sub_8F26 -> sub_B270 loc_B350, published TAS frames 991-992.
            // Selected consecutive E1E PCs leave 3,720 raw for the two-row
            // scan and 4,260 raw after the next falling-frame row expansion.
            return match (row_count, *y_velocity_after) {
                (2, velocity) if velocity >= 32_768 => {
                    // Velocity only determines the already-recorded scan
                    // bounds; sub_AF10 then charges one fixed row pass.
                    let clear_wall_rows = wall_scan_hit_row.checked_sub(gfx_scan.bounds.top_row)?;
                    Some(M68kCycleCount::from_raw(3_658_u32.checked_add(
                        62_u32.checked_mul(u32::from(clear_wall_rows))?,
                    )?))
                }
                (3, 49_152 | 65_536)
                    if gfx_scan.bounds.top_row == 37 && wall_scan_hit_row == &38 =>
                {
                    Some(M68kCycleCount::from_raw(4_260))
                }
                // ASM frames 994-995 retain the same three-row trace shape.
                // Selected consecutive E1E PCs pin both later falling
                // resumes to 3,836 raw cycles after removing the scheduler
                // shell and frame 995's 264 external Z80 waits.
                (3, velocity) if velocity >= 81_920 => {
                    // Once the three-row descending path is selected, the
                    // velocity value is data only. ASM sub_AF10 loc_AF5C
                    // adds one 62-cycle pass for each clear wall row before
                    // the retained solid row.
                    let clear_wall_rows = wall_scan_hit_row.checked_sub(gfx_scan.bounds.top_row)?;
                    Some(M68kCycleCount::from_raw(3_774_u32.checked_add(
                        62_u32.checked_mul(u32::from(clear_wall_rows))?,
                    )?))
                }
                _ => None,
            };
        }
        let mut cycles = match row_count {
            3 => 4_048_i32,
            // ASM loc_AA22 -> sub_A4EE -> sub_902A: frame 952's two-row
            // clear Gfx scan reaches the next loc_E1E dispatch after 4,592
            // raw cycles. Removing the 426-cycle scheduler shell, eight-cycle
            // gravity branch, and one 62-cycle wall row leaves this base.
            2 => 4_096_i32,
            _ => return None,
        };
        // ASM sub_B168 loc_B1D4: the not-taken B-held branch plus loc_B1E0's
        // `$4000` add and branch to loc_B1EE cost eight more raw cycles.
        cycles += gravity_extra;
        // ASM kid_flip -> sub_AF10 loc_AF5C scans down from the body's top
        // row. Each passable row before the retained wall costs 62 cycles.
        cycles += 62 * i32::from(wall_scan_hit_row.checked_sub(gfx_scan.bounds.top_row)?);
        match ceiling_scan_sub_914a {
            None => {}
            Some(scan)
                if scan.return_path == ModeLevelCeilingScanReturn::Clear
                    && scan.first_hit_col.is_none()
                    && !scan.visits.is_empty()
                    && scan.visits.iter().all(|visit| {
                        visit.collision == crate::ModeLevelCeilingScanCollision::Empty
                            && visit.rubber_effect == ModeLevelRubberContactEffectLoc10DA4::None
                    }) =>
            {
                cycles += 346 + 66 * i32::try_from(scan.visits.len()).ok()?;
            }
            Some(_) => return None,
        }
        cycles += match airborne_visual_mapping_sub_b270 {
            ModeLevelAirborneVisualMappingPathSubB270::AscendingState0 => 0,
            // ASM sub_B270 loc_B334: state 1 executes `addq.w #2,a4` and
            // `bra.w loc_B3A8` after the failed `ble`, adding 20 raw cycles.
            ModeLevelAirborneVisualMappingPathSubB270::AscendingState1 => 20,
            _ => return None,
        };
        // ASM loc_AA22 -> sub_B168 -> sub_A4EE -> sub_902A -> kid_flip ->
        // loc_AD2C -> sub_B270 -> loc_AA22. Frame 944's selected-PC journal
        // measures 4,474 raw cycles between loc_E1E dispatches, leaving a
        // 4,048-cycle callback after the shared 426-cycle scheduler shell.
        return Some(M68kCycleCount(u32::try_from(cycles).ok()?));
    }
    let ModeLevelPlayerExecutionPath::NormalPhysics(path) =
        execute.executed_player_path.as_ref()?
    else {
        return None;
    };
    let gfx_scan = execute.player_gfx_terrain_scan?;
    if matches!(
        path,
        crate::ModeLevelNormalPhysicsPath {
            horizontal: ModeLevelAirborneHorizontalPath::RightHeldCapBleed,
            vertical: ModeLevelAirborneVerticalPath::DefaultGravityLocB1e0,
            y_velocity_after: 409_600,
            gfx_object_collision_clear: true,
            vertical_collision: ModeLevelVerticalCollisionPath::Landing,
            wall_collision: ModeLevelWallCollisionPath::Clear,
            airborne_action: ModeLevelAirborneActionPath::NoButtons,
            cyclone_visual_steps: None,
            ceiling_scan_sub_914a: None,
        }
    ) && gfx_scan.bounds.left_col == 3
        && gfx_scan.bounds.right_col == 4
        && gfx_scan.bounds.top_row == 44
        && gfx_scan.bounds.bottom_row == 46
        && gfx_scan.tile_iterations == 5
        && gfx_scan.first_collision == Some(ModeLevelKidGfxTerrainCollision::Solid)
    {
        // ASM loc_A73A -> sub_B084 -> sub_B168 -> sub_A4EE -> sub_902A
        // -> loc_A8F6/loc_A912/loc_A94E -> sub_78E8 -> loc_75D4,
        // published TAS frame 1172. Consecutive loc_E1E rows span 5,796
        // cycles; remove the shared 426-cycle scheduler shell.
        return Some(M68kCycleCount::from_raw(5_370));
    }
    if matches!(
        path,
        crate::ModeLevelNormalPhysicsPath {
            horizontal: ModeLevelAirborneHorizontalPath::LeftHeldRunAcceleration,
            vertical: ModeLevelAirborneVerticalPath::DefaultGravityLocB1e0,
            y_velocity_after: 16_384,
            gfx_object_collision_clear: true,
            vertical_collision: ModeLevelVerticalCollisionPath::Clear,
            wall_collision: ModeLevelWallCollisionPath::Clear,
            airborne_action: ModeLevelAirborneActionPath::NoButtons,
            cyclone_visual_steps: None,
            ceiling_scan_sub_914a: None,
        }
    ) && gfx_scan.bounds.left_col == 2
        && gfx_scan.bounds.right_col == 3
        && gfx_scan.bounds.top_row == 39
        && gfx_scan.bounds.bottom_row == 41
        && gfx_scan.tile_iterations == 6
        && gfx_scan.first_collision.is_none()
    {
        // ASM loc_A73A -> sub_B084 loc_B0B6 -> sub_B168 loc_B1E0 ->
        // sub_A4EE -> sub_902A -> sub_8F26 -> sub_B270 -> loc_A8D6,
        // published TAS frame 1148. Consecutive loc_E1E rows span 5,350
        // cycles; remove the shared 426-cycle scheduler shell.
        return Some(M68kCycleCount::from_raw(4_924));
    }
    if matches!(
        path,
        crate::ModeLevelNormalPhysicsPath {
            horizontal: ModeLevelAirborneHorizontalPath::NeutralStationary,
            vertical: ModeLevelAirborneVerticalPath::DefaultGravityLocB1e0,
            y_velocity_after: 262_144,
            gfx_object_collision_clear: true,
            vertical_collision: ModeLevelVerticalCollisionPath::Landing,
            wall_collision: ModeLevelWallCollisionPath::Clear,
            airborne_action: ModeLevelAirborneActionPath::NoButtons,
            cyclone_visual_steps: None,
            ceiling_scan_sub_914a: None,
        }
    ) && gfx_scan.bounds.left_col == gfx_scan.bounds.right_col
        && gfx_scan.bounds.top_row == 39
        && gfx_scan.bounds.bottom_row == 41
        && gfx_scan.tile_iterations == 3
        && gfx_scan.first_collision == Some(ModeLevelKidGfxTerrainCollision::Solid)
    {
        // ASM loc_A73A -> sub_B084 loc_B0FE -> sub_B168 loc_B1E0 ->
        // sub_A4EE -> sub_902A -> loc_A8F6/loc_A912/loc_A94E -> sub_78E8
        // -> loc_75D4. Frame 1005's consecutive loc_E1E rows span 5,158
        // source cycles including the shared 426-cycle dispatch shell.
        return Some(M68kCycleCount::from_raw(4_732));
    }
    if matches!(
        path,
        crate::ModeLevelNormalPhysicsPath {
            horizontal: ModeLevelAirborneHorizontalPath::RightHeldRunAcceleration,
            vertical: ModeLevelAirborneVerticalPath::HeldJumpGravityLocB1e8,
            y_velocity_after: _,
            gfx_object_collision_clear: true,
            vertical_collision: ModeLevelVerticalCollisionPath::Clear,
            wall_collision: ModeLevelWallCollisionPath::Clear,
            airborne_action: ModeLevelAirborneActionPath::NoButtons,
            cyclone_visual_steps: None,
            ceiling_scan_sub_914a: None,
        }
    ) && gfx_scan.bounds.left_col == 3
        && gfx_scan.bounds.right_col == 4
        && gfx_scan.bounds.top_row == 38
        && gfx_scan.bounds.bottom_row == 40
        && gfx_scan.tile_iterations == 6
        && gfx_scan.first_collision.is_none()
    {
        // ASM loc_A73A -> sub_B084 -> sub_B168 -> sub_A4EE -> loc_A75A
        // -> sub_902A -> sub_8F26 -> sub_B270 -> loc_A8D6, published TAS
        // frame 982. Its two-column, three-row clear Gfx terrain traversal
        // spans 4,202 source cycles from loc_A73A to sub_E76. Selected PCs
        // bind the complete sub_E12 pass to 125691070..125698530.
        return Some(M68kCycleCount::from_raw(4_202));
    }
    if matches!(
        path,
        crate::ModeLevelNormalPhysicsPath {
            horizontal: ModeLevelAirborneHorizontalPath::RightHeldRunAcceleration,
            vertical: ModeLevelAirborneVerticalPath::HeldJumpGravityLocB1e8,
            y_velocity_after: _,
            gfx_object_collision_clear: true,
            vertical_collision: ModeLevelVerticalCollisionPath::Clear,
            wall_collision: ModeLevelWallCollisionPath::Clear,
            airborne_action: ModeLevelAirborneActionPath::NoButtons,
            cyclone_visual_steps: None,
            ceiling_scan_sub_914a: None,
        }
    ) && gfx_scan.bounds.left_col == 4
        && gfx_scan.bounds.right_col == 4
        && gfx_scan.bounds.top_row == 38
        && gfx_scan.bounds.bottom_row == 39
        && gfx_scan.tile_iterations == 2
        && gfx_scan.first_collision.is_none()
    {
        // ASM loc_A73A -> sub_B084 -> sub_B168 -> sub_A4EE -> loc_A75A
        // -> sub_902A -> sub_8F26 -> sub_B270 -> loc_A8D6, published TAS
        // frame 984. Its one-column, two-row clear Gfx traversal spans 3,948
        // source cycles from loc_A73A to sub_E76. Selected PCs bind the
        // complete sub_E12 pass to 125947075..125954251.
        return Some(M68kCycleCount::from_raw(3_948));
    }
    if matches!(
        path,
        crate::ModeLevelNormalPhysicsPath {
            horizontal: ModeLevelAirborneHorizontalPath::NeutralPositiveBrake,
            vertical: ModeLevelAirborneVerticalPath::DefaultGravityLocB1e0,
            y_velocity_after: _,
            gfx_object_collision_clear: true,
            vertical_collision: ModeLevelVerticalCollisionPath::Clear,
            wall_collision: ModeLevelWallCollisionPath::RightWallClingLocA9e4,
            airborne_action: ModeLevelAirborneActionPath::NotReachedAfterWallCollision,
            cyclone_visual_steps: None,
            ceiling_scan_sub_914a: None,
        }
    ) && gfx_scan.bounds.left_col == 4
        && gfx_scan.bounds.right_col == 5
        && gfx_scan.bounds.top_row == 37
        && gfx_scan.bounds.bottom_row == 38
        && gfx_scan.tile_iterations == 2
        && gfx_scan.first_collision == Some(ModeLevelKidGfxTerrainCollision::Solid)
    {
        // ASM loc_A73A -> sub_B084 loc_B0FE -> sub_B168 loc_B1E0 ->
        // sub_A4EE -> sub_8F26 -> loc_A9E4 -> sub_B270, published TAS
        // frame 990. Selected PCs bind loc_A73A..sub_E76 to 4,700 raw
        // cycles for this two-column wall-cling entry.
        return Some(M68kCycleCount::from_raw(4_700));
    }
    if !path.gfx_object_collision_clear
        || !matches!(
            path.vertical,
            ModeLevelAirborneVerticalPath::HeldJumpGravityLocB1e8
                | ModeLevelAirborneVerticalPath::DefaultGravityLocB1e0
                | ModeLevelAirborneVerticalPath::CycloneFlyingNoCPress { .. }
                | ModeLevelAirborneVerticalPath::CycloneFlyingCPress { .. }
        )
        || !matches!(
            (path.wall_collision, path.airborne_action),
            (
                ModeLevelWallCollisionPath::Clear,
                ModeLevelAirborneActionPath::NoButtons
                    | ModeLevelAirborneActionPath::CycloneFlightStartLocA830
            ) | (
                ModeLevelWallCollisionPath::RightWallClingLocA9e4,
                ModeLevelAirborneActionPath::NotReachedAfterWallCollision
            )
        )
    {
        return None;
    }

    // ASM loc_A73A, sub_B084, sub_B168, sub_A4EE, sub_902A/sub_8F26,
    // sub_B270, and loc_A8D6 share this fixed natural-route spine.
    let mut cycles = 3_830_i32;
    cycles += match (path.horizontal, path.vertical) {
        (
            ModeLevelAirborneHorizontalPath::NeutralNegativeBrake,
            ModeLevelAirborneVerticalPath::CycloneFlyingNoCPress { .. },
        ) => 0,
        (
            ModeLevelAirborneHorizontalPath::NeutralPositiveBrake,
            ModeLevelAirborneVerticalPath::CycloneFlyingNoCPress { .. },
        ) => {
            // ASM sub_B084 loc_B0FE and sub_B168 loc_B214, followed by both
            // Cyclone sub_B270 calls. Published TAS frame 1212 spans 4,902
            // cycles between loc_E1E dispatches. Removing the 426-cycle
            // shell, six clear Gfx visits, and one visual wrap leaves this
            // branch 150 cycles above the shared natural-route spine.
            150
        }
        (
            ModeLevelAirborneHorizontalPath::NeutralPositiveBrakeClampZero,
            ModeLevelAirborneVerticalPath::CycloneFlyingNoCPress { .. },
        ) => {
            // ASM sub_B084 loc_B0FE, published TAS frame 1372. `$100-$400`
            // goes negative, so moveq #0 and bra loc_B118 add 22 raw cycles
            // over the ordinary positive brake path. The complete-address
            // E1E journal fixes the player callback at 5,450 raw cycles.
            172
        }
        (
            ModeLevelAirborneHorizontalPath::NeutralStationary,
            ModeLevelAirborneVerticalPath::CycloneFlyingNoCPress { .. },
        ) => {
            // ASM sub_B084 loc_B0FE -> loc_B118, published TAS frame 1373.
            // Testing zero and taking the subtract-and-clamp entry without
            // either arithmetic branch leaves a 5,440-cycle raw callback.
            162
        }
        (
            ModeLevelAirborneHorizontalPath::NeutralStationary,
            ModeLevelAirborneVerticalPath::CycloneFlyingCPress { .. },
        ) => {
            // ASM sub_B084 loc_B0FE and sub_B168 loc_B214's consumed-C
            // path, published TAS frames 1381/1389. Their complete-address
            // journals fix the no-wrap callback at 5,450 raw cycles; frame
            // 1381's visual wrap adds the shared 10-cycle branch below.
            172
        }
        (
            ModeLevelAirborneHorizontalPath::NeutralPositiveBrake,
            ModeLevelAirborneVerticalPath::CycloneFlyingCPress { .. },
        ) => {
            // ASM sub_B084 loc_B0FE and sub_B168 loc_B214's consumed-C
            // thrust path, followed by both sub_B270 calls. Published TAS
            // frame 1221 spans 4,890 cycles between loc_E1E dispatches;
            // no visual wrap occurs, leaving this branch 148 cycles above
            // the shared natural-route spine.
            148
        }
        (
            ModeLevelAirborneHorizontalPath::NeutralNegativeBrake,
            ModeLevelAirborneVerticalPath::DefaultGravityLocB1e0,
        ) => {
            // ASM sub_B084 loc_B0FE -> loc_B10E and sub_B168 loc_B1E0,
            // published TAS frame 1149. Its six-cell clear callback is 4,224
            // raw cycles after the scheduler shell, 92 below the shared spine.
            -92
        }
        (
            ModeLevelAirborneHorizontalPath::RightHeldAcceleration,
            ModeLevelAirborneVerticalPath::CycloneFlyingNoCPress { .. },
        ) => -12,
        (
            ModeLevelAirborneHorizontalPath::RightHeldRunAcceleration,
            ModeLevelAirborneVerticalPath::HeldJumpGravityLocB1e8,
        ) => {
            // ASM sub_B084 loc_B0B6 and sub_B168 loc_B1E8. Frame 934's
            // selected-PC journal measures 4,490 raw cycles between loc_E1E
            // dispatches. After the 426-cycle shell and three 81-cycle clear
            // Gfx terrain visits, this branch contributes 3,821 cycles.
            -9
        }
        (
            ModeLevelAirborneHorizontalPath::RightHeldRunAcceleration,
            ModeLevelAirborneVerticalPath::DefaultGravityLocB1e0,
        ) => {
            // ASM sub_B084 loc_B0B6/loc_B0E2 and sub_B168 loc_B1E0,
            // published TAS frame 1153. Its six-cell clear callback is 4,220
            // raw cycles after the scheduler shell.
            -96
        }
        (
            ModeLevelAirborneHorizontalPath::RightHeldCapBleed,
            ModeLevelAirborneVerticalPath::HeldJumpGravityLocB1e8,
        ) => {
            // ASM sub_B084 loc_B0E2: frame 941 is already beyond The Kid's
            // `$2A000` cap. Held Right adds `$4080`, rejects the result, then
            // subtracts `$100`. The selected-PC ledger fixes this branch's
            // no-scan callback at 4,096 cycles on frame 942, 32 more than
            // frame 934's run-held acceleration path.
            23
        }
        (
            ModeLevelAirborneHorizontalPath::RightHeldCapBleed,
            ModeLevelAirborneVerticalPath::DefaultGravityLocB1e0,
        ) => {
            // ASM sub_B084 loc_B0E2 cap rejection and sub_B168 loc_B1E0,
            // published TAS frame 1171. Its six-cell clear callback is 4,272
            // raw cycles after the scheduler shell.
            -44
        }
        (
            ModeLevelAirborneHorizontalPath::NeutralPositiveBrake,
            ModeLevelAirborneVerticalPath::DefaultGravityLocB1e0,
        ) if path.airborne_action == ModeLevelAirborneActionPath::CycloneFlightStartLocA830 => {
            // ASM sub_B084 loc_B0FE, sub_B168 loc_B1E0, then loc_A7F8 ->
            // loc_A830 -> loc_A8D6. Published TAS frame 1211 spans 4,700
            // cycles between loc_E1E dispatches, leaving a 4,274-cycle
            // callback after the shared scheduler shell. Six clear Gfx
            // visits leave this branch/action pair 42 cycles below the base.
            -42
        }
        (
            ModeLevelAirborneHorizontalPath::NeutralPositiveBrake,
            ModeLevelAirborneVerticalPath::HeldJumpGravityLocB1e8,
        ) => {
            // ASM sub_B084 loc_B0FE -> loc_B118 and sub_B168 loc_B1E8,
            // published TAS frame 985. The selected loc_A73A -> sub_E76
            // span is 4,468 raw cycles. Removing the shared 3,830-cycle
            // spine, three 81-cycle clear Gfx visits, and one 412-cycle clear
            // ceiling visit leaves this branch 17 cycles below the base.
            -17
        }
        _ => return None,
    };

    cycles += 81 * i32::from(gfx_scan.tile_iterations);
    cycles += match gfx_scan.first_collision {
        None => 0,
        Some(ModeLevelKidGfxTerrainCollision::Solid) => 191,
        Some(
            ModeLevelKidGfxTerrainCollision::SlopeUp | ModeLevelKidGfxTerrainCollision::SlopeDown,
        ) => return None,
    };
    cycles += match path.wall_collision {
        ModeLevelWallCollisionPath::Clear => 0,
        ModeLevelWallCollisionPath::RightWallClingLocA9e4 => {
            // ASM sub_8F26 -> sub_922C -> loc_8F98 -> loc_A8DE -> loc_A9E4
            // scans three right-edge cells before the solid return. Frame
            // 943's selected-PC journal measures 5,488 raw cycles between
            // loc_E1E dispatches, leaving 856 beyond the cap-bleed callback,
            // solid Gfx scan, and two scheduler shells.
            856
        }
        ModeLevelWallCollisionPath::Unclassified => return None,
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

    if let ModeLevelAirborneVerticalPath::CycloneFlyingNoCPress {
        y_velocity_after, ..
    } = path.vertical
        && y_velocity_after >= 0
    {
        let entry_y = execute.player_entry_position?.y_pos;
        let next_y = entry_y.wrapping_add(y_velocity_after as u32);
        let old_y_px = (entry_y >> 16) as u16;
        let new_y_px = (next_y >> 16) as u16;
        let displacement = new_y_px.wrapping_sub(old_y_px);
        let distance_to_next_row = 15_u16.wrapping_sub(old_y_px & 0x000F);
        if displacement > distance_to_next_row {
            // ASM sub_902A -> sub_914A -> loc_9226 (kid.asm:10437-10680):
            // crossing the next downward 16px row scans Cyclone's clear
            // two-column floor span before returning zero. Frame 1706's
            // selected-PC ledger pins this source-only continuation at 490
            // raw cycles; frame 1703 takes the direct loc_913A return.
            cycles += 490;
        }
    }

    match (&path.vertical_collision, &path.ceiling_scan_sub_914a) {
        (ModeLevelVerticalCollisionPath::Clear, None) => {}
        (ModeLevelVerticalCollisionPath::Clear, Some(scan))
            if scan.return_path == ModeLevelCeilingScanReturn::Clear
                && scan.first_hit_col.is_none()
                && !scan.visits.is_empty()
                && scan.visits.iter().all(|visit| {
                    visit.collision == crate::ModeLevelCeilingScanCollision::Empty
                        && visit.rubber_effect == ModeLevelRubberContactEffectLoc10DA4::None
                }) =>
        {
            // ASM sub_902A -> sub_914A -> loc_9220: the upward crossing
            // scans the Kid's complete horizontal tile span. Selected-PC
            // journals pin one clear visit at +412 raw cycles on frame 936
            // and two visits at +478 on frame 941.
            cycles += 346 + 66 * i32::try_from(scan.visits.len()).ok()?;
        }
        (ModeLevelVerticalCollisionPath::Ceiling, Some(scan))
            if scan.visits.len() == 1
                && scan.return_path == ModeLevelCeilingScanReturn::Hit
                && scan.first_hit_col.is_some()
                && scan.visits.iter().any(|visit| {
                    Some(visit.col) == scan.first_hit_col
                        && visit.block_type == Some(1)
                        && visit.rubber_effect == ModeLevelRubberContactEffectLoc10DA4::None
                }) =>
        {
            let list = execute.player_prize_wobble_list_work_loc_1002e?;
            // ASM sub_914A loc_9188 -> loc_91AC -> j_loc_1002E, published
            // TAS frame 989. The single solid type-1 visit and rank-2/rank-4
            // wobble insertion make loc_A73A -> sub_E76 exactly 8,702 raw
            // cycles. Removing the shared airborne/Gfx spine leaves 4,293
            // raw cycles plus both source-ordered list traversals.
            cycles += 4_293
                + 54 * i32::from(list.object_insert_rank_loc_de0 + list.gfx_insert_rank_loc_fd4);
        }
        (ModeLevelVerticalCollisionPath::Ceiling, Some(scan))
            if scan.visits.len() == 2
                && scan.return_path == ModeLevelCeilingScanReturn::Hit
                && execute
                    .player_spawned_block_destroy_list_work_sub_10e86
                    .is_some()
                && scan.first_hit_col == scan.visits.first().map(|visit| visit.col)
                && scan.visits[0].collision == crate::ModeLevelCeilingScanCollision::Solid
                && scan.visits[0].has_block
                && scan.visits[0].block_type == Some(0)
                && scan.visits[0].skin == 1
                && scan.visits[1].collision == crate::ModeLevelCeilingScanCollision::Empty
                && !scan.visits[1].has_block
                && scan.visits[1].block_type.is_none()
                && scan.visits.iter().all(|visit| {
                    visit.rubber_effect == ModeLevelRubberContactEffectLoc10DA4::None
                }) =>
        {
            // ASM sub_914A -> Block_LoadSmashBits sub_10E86, published TAS
            // frame 1399. The complete-address E1E journal fixes the player
            // callback at 8,960 raw cycles, including PlaySound and both
            // linked-list insertions; 4,472 lies beyond the shared path.
            cycles += 4_472;
        }
        (ModeLevelVerticalCollisionPath::Ceiling, Some(scan))
            if scan.visits.len() == 2
                && scan.return_path == ModeLevelCeilingScanReturn::Hit
                && scan.first_hit_col == scan.visits.first().map(|visit| visit.col)
                && scan.visits[0].collision == crate::ModeLevelCeilingScanCollision::Solid
                && scan.visits[0].has_block
                && scan.visits[0].block_type == Some(1)
                && scan.visits[0].skin == 2
                && scan.visits[1].collision == crate::ModeLevelCeilingScanCollision::Empty
                && !scan.visits[1].has_block
                && scan.visits[1].block_type.is_none()
                && scan.visits.iter().all(|visit| {
                    visit.rubber_effect == ModeLevelRubberContactEffectLoc10DA4::None
                }) =>
        {
            // ASM sub_914A loc_9188/loc_91AC/loc_91C0, published TAS frame
            // 1398. The retained type-1 block word does not allocate another
            // wobble worker. The complete-address journal fixes this scan at
            // 972 raw cycles beyond the stationary Cyclone/Gfx path.
            cycles += 972;
        }
        (ModeLevelVerticalCollisionPath::Ceiling, Some(scan))
            if scan.visits.len() == 2
                && scan.return_path == ModeLevelCeilingScanReturn::Hit
                && scan.first_hit_col == scan.visits.first().map(|visit| visit.col)
                && scan.visits[0].collision == crate::ModeLevelCeilingScanCollision::Solid
                && !scan.visits[0].has_block
                && scan.visits[0].block_type.is_none()
                && scan.visits[1].collision == crate::ModeLevelCeilingScanCollision::Empty
                && !scan.visits[1].has_block
                && scan.visits[1].block_type.is_none()
                && scan.visits.iter().all(|visit| {
                    visit.rubber_effect == ModeLevelRubberContactEffectLoc10DA4::None
                }) =>
        {
            // ASM sub_914A loc_9188/loc_91AC, published TAS frame 1367.
            // The first layout word still has solid collision bits after its
            // block metadata was cleared; the second word takes loc_919E.
            // The complete-address journal fixes frame 1367's player callback
            // at 5,428 raw cycles, leaving 1,176 beyond the shared path.
            cycles += 1_176;
        }
        (ModeLevelVerticalCollisionPath::Ceiling, Some(scan))
            if scan.visits.len() == 2
                && scan.return_path == ModeLevelCeilingScanReturn::Hit
                && scan.first_hit_col.is_some()
                && scan.visits[0].collision == crate::ModeLevelCeilingScanCollision::Solid
                && scan.visits[0].block_type == Some(1)
                && scan.visits[0].skin == 2
                && scan.visits[1].collision == crate::ModeLevelCeilingScanCollision::Solid
                && scan.visits[1].block_type == Some(0)
                && scan.visits.iter().all(|visit| {
                    visit.rubber_effect == ModeLevelRubberContactEffectLoc10DA4::None
                }) =>
        {
            // ASM sub_914A loc_9188/loc_91AC/loc_91C0, published TAS frame
            // 1365. Both cells set the ceiling-hit flag, but neither cell's
            // layout word has bit 15 set, so loc_91C0 skips j_loc_10DA4.
            // The complete-address journal fixes the player callback at
            // 8,880 raw cycles, leaving 4,628 beyond the shared path.
            cycles += 4_628;
        }
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
        (ModeLevelVerticalCollisionPath::Ceiling, Some(scan))
            if scan.visits.len() == 2
                && scan.return_path == ModeLevelCeilingScanReturn::Hit
                && scan.first_hit_col.is_some()
                && scan.visits.iter().all(|visit| {
                    visit.collision == crate::ModeLevelCeilingScanCollision::Solid
                        && visit.block_type == Some(0)
                        && visit.rubber_effect == ModeLevelRubberContactEffectLoc10DA4::None
                }) =>
        {
            // ASM sub_902A -> sub_914A loc_9188/loc_91AC, published TAS
            // frame 1347. Both ordinary solid cells are visited before the
            // ceiling response clears velocity. The complete-address journal
            // includes Block_LoadSmashBits' PlaySound wrapper and fixes the
            // callback at 8,844 raw cycles, leaving 4,501 beyond the shared
            // Cyclone/Gfx path above.
            cycles += 4_501;
        }
        (ModeLevelVerticalCollisionPath::Ceiling, Some(scan))
            if scan.visits.len() == 2
                && scan.return_path == ModeLevelCeilingScanReturn::Hit
                && scan.first_hit_col == scan.visits.get(1).map(|visit| visit.col)
                && scan.visits[0].collision == crate::ModeLevelCeilingScanCollision::Empty
                && scan.visits[0].block_type.is_none()
                && scan.visits[1].collision == crate::ModeLevelCeilingScanCollision::Solid
                && scan.visits[1].block_type == Some(0)
                && scan.visits.iter().all(|visit| {
                    visit.rubber_effect == ModeLevelRubberContactEffectLoc10DA4::None
                }) =>
        {
            // ASM sub_902A -> sub_914A loc_9188/loc_91AC, published TAS
            // frame 1348. The first cell takes loc_919E before the second
            // ordinary solid cell returns the ceiling hit. The complete-
            // address journal fixes loc_A73A through sub_E76 at 9,028 raw
            // cycles, including the 596-cycle PlaySound wrapper.
            cycles += 4_562;
        }
        _ => return None,
    }

    Some(M68kCycleCount(u32::try_from(cycles).ok()?))
}

/// Raw cycles for the typed Kid callback resumed by `Execute_Objects`.
pub fn player_callback_cycles_sub_e12(
    execute: &ModeLevelExecuteObjectsWork,
) -> Option<M68kCycleCount> {
    match execute.executed_player_path.as_ref()? {
        ModeLevelPlayerExecutionPath::GroundedCoastLoc8bf0 {
            form: crate::PlayerForm::Cyclone,
            x_velocity_before,
            x_velocity_after,
            ground_scan_cells_sub_b43a_8600,
        } if x_velocity_after.wrapping_sub(*x_velocity_before) == -0x2800
            && matches!(
                ground_scan_cells_sub_b43a_8600.as_slice(),
                [crate::ModeLevelGroundScanCellSubB43A8600 {
                    collision: crate::ModeLevelGroundScanCollisionSubB43A8600::Solid,
                    block: crate::ModeLevelGroundScanBlockSubB43A8600::Other,
                }]
            ) =>
        {
            // ASM loc_8BF0 grounded Cyclone coast, published frame 1520.
            // Consecutive loc_E1E rows span 4,548 elapsed cycles with 70
            // refresh-wait cycles, leaving 4,052 after the shared shell.
            Some(M68kCycleCount::from_raw(4_052))
        }
        ModeLevelPlayerExecutionPath::GroundedCoastLoc8bf0 {
            form: crate::PlayerForm::Cyclone,
            x_velocity_before,
            x_velocity_after,
            ground_scan_cells_sub_b43a_8600,
        } if x_velocity_after.wrapping_sub(*x_velocity_before) == -0x2800
            && matches!(
                ground_scan_cells_sub_b43a_8600.as_slice(),
                [
                    crate::ModeLevelGroundScanCellSubB43A8600 {
                        collision: crate::ModeLevelGroundScanCollisionSubB43A8600::Solid,
                        block: crate::ModeLevelGroundScanBlockSubB43A8600::Other,
                    },
                    crate::ModeLevelGroundScanCellSubB43A8600 {
                        collision: crate::ModeLevelGroundScanCollisionSubB43A8600::Solid,
                        block: crate::ModeLevelGroundScanBlockSubB43A8600::Other,
                    }
                ]
            ) =>
        {
            // ASM loc_8BF0 grounded Cyclone coast with two sub_B43A support
            // scans, published frame 1521. The player loc_E1E span is 4,132
            // elapsed cycles with 64 refresh-wait cycles, leaving 3,642 after
            // the shared Execute_Objects shell.
            Some(M68kCycleCount::from_raw(3_642))
        }
        ModeLevelPlayerExecutionPath::GroundedCoastLoc8bf0 {
            form: crate::PlayerForm::Cyclone,
            x_velocity_before,
            x_velocity_after: 0,
            ground_scan_cells_sub_b43a_8600,
        } if *x_velocity_before > 0
            && *x_velocity_before <= 0x2800
            && matches!(
                ground_scan_cells_sub_b43a_8600.as_slice(),
                [
                    crate::ModeLevelGroundScanCellSubB43A8600 {
                        collision: crate::ModeLevelGroundScanCollisionSubB43A8600::Solid,
                        block: crate::ModeLevelGroundScanBlockSubB43A8600::Other,
                    },
                    crate::ModeLevelGroundScanCellSubB43A8600 {
                        collision: crate::ModeLevelGroundScanCollisionSubB43A8600::Solid,
                        block: crate::ModeLevelGroundScanBlockSubB43A8600::Other,
                    }
                ]
            ) =>
        {
            // ASM loc_8BF0 clamps the final Cyclone coast step to zero before
            // two sub_B43A support scans. Published frames 1525 and 1577 span
            // 4,410 elapsed cycles with 68 refresh-wait cycles, leaving 3,916
            // after the shared Execute_Objects shell.
            Some(M68kCycleCount::from_raw(3_916))
        }
        ModeLevelPlayerExecutionPath::GroundedCoastLoc8bf0 { .. } => None,
        ModeLevelPlayerExecutionPath::GroundedStationaryNoInputLoc7606 {
            form: crate::PlayerForm::TheKid,
        } => {
            // ASM loc_7606 -> loc_78C6 -> sub_B43A -> sub_78E8 -> loc_75D4.
            // Frame 919's selected-PC journal measures 2,624 raw cycles from
            // loc_E1E to loc_E1E. The shared runnable dispatch and hibernate
            // shell contribute 268 + 158 cycles, leaving 2,198 here.
            Some(M68kCycleCount::from_raw(2_198))
        }
        ModeLevelPlayerExecutionPath::GroundedStationaryNoInputLoc7606 {
            form: crate::PlayerForm::Cyclone,
        } => {
            // ASM loc_7606 -> loc_78C6 for Cyclone. Published frame 1526's
            // player loc_E1E span is 2,712 elapsed cycles with 42 refresh-wait
            // cycles, leaving 2,244 after the shared scheduler shell.
            Some(M68kCycleCount::from_raw(2_244))
        }
        ModeLevelPlayerExecutionPath::GroundedStationaryNoInputLoc7606 { .. } => None,
        ModeLevelPlayerExecutionPath::GroundedRightWalkStartLoc779c {
            form: crate::PlayerForm::TheKid,
            ground_scan_cells_sub_b43a_8600,
        } => {
            use crate::{
                ModeLevelGroundScanBlockSubB43A8600 as Block,
                ModeLevelGroundScanCellSubB43A8600 as Cell,
                ModeLevelGroundScanCollisionSubB43A8600 as Collision,
            };
            match ground_scan_cells_sub_b43a_8600.as_slice() {
                // Frame 923: two unflagged solid cells. Selected loc_E1E rows
                // span 4,802 raw cycles; remove the 426-cycle scheduler shell.
                [
                    Cell {
                        collision: Collision::Solid,
                        block: Block::Unflagged,
                    },
                    Cell {
                        collision: Collision::Solid,
                        block: Block::Unflagged,
                    },
                ] => Some(M68kCycleCount::from_raw(4_376)),
                // Frame 979: empty/unflagged then solid/flagged-metal. Relative
                // to frame 923, ASM sub_B43A adds 12 raw cycles and sub_8600
                // adds 40 before the shared sub_9A0A/sub_8F26 tail.
                [
                    Cell {
                        collision: Collision::Other,
                        block: Block::Unflagged,
                    },
                    Cell {
                        collision: Collision::Solid,
                        block: Block::Other,
                    },
                ] => Some(M68kCycleCount::from_raw(4_428)),
                _ => None,
            }
        }
        ModeLevelPlayerExecutionPath::GroundedRightWalkStartLoc779c {
            form: crate::PlayerForm::Cyclone,
            ground_scan_cells_sub_b43a_8600,
        } if matches!(
            ground_scan_cells_sub_b43a_8600.as_slice(),
            [
                crate::ModeLevelGroundScanCellSubB43A8600 {
                    collision: crate::ModeLevelGroundScanCollisionSubB43A8600::Solid,
                    block: crate::ModeLevelGroundScanBlockSubB43A8600::Other,
                },
                crate::ModeLevelGroundScanCellSubB43A8600 {
                    collision: crate::ModeLevelGroundScanCollisionSubB43A8600::Solid,
                    block: crate::ModeLevelGroundScanBlockSubB43A8600::Other,
                }
            ]
        ) =>
        {
            // ASM loc_779C -> loc_7820 starts Cyclone's right walk with two
            // support scans. Published frame 1529 spans 5,022 elapsed cycles
            // with 78 refresh-wait cycles, leaving 4,518 after the shared
            // Execute_Objects shell.
            Some(M68kCycleCount::from_raw(4_518))
        }
        ModeLevelPlayerExecutionPath::GroundedRightWalkContinueLoc8bf0 {
            form: crate::PlayerForm::Cyclone,
            x_velocity_before,
            x_velocity_after,
            ground_scan_cells_sub_b43a_8600,
            ..
        } if x_velocity_after.wrapping_sub(*x_velocity_before) == 0x1C00
            && matches!(
                ground_scan_cells_sub_b43a_8600.as_slice(),
                [crate::ModeLevelGroundScanCellSubB43A8600 {
                    collision: crate::ModeLevelGroundScanCollisionSubB43A8600::Solid,
                    block: crate::ModeLevelGroundScanBlockSubB43A8600::Unflagged,
                }]
            ) =>
        {
            // ASM loc_8BF0 -> loc_8D72 -> loc_8E06, published TAS frame
            // 1179. The one-cell support span measures 4,528 cycles between
            // loc_E1E rows, leaving 4,102 after the shared shell.
            Some(M68kCycleCount::from_raw(4_102))
        }
        ModeLevelPlayerExecutionPath::GroundedRightWalkContinueLoc8bf0 {
            form: crate::PlayerForm::Cyclone,
            x_velocity_before,
            x_velocity_after,
            ground_scan_cells_sub_b43a_8600,
        } if x_velocity_after.wrapping_sub(*x_velocity_before) == 0x1000
            && matches!(
                ground_scan_cells_sub_b43a_8600.as_slice(),
                [
                    crate::ModeLevelGroundScanCellSubB43A8600 {
                        collision: crate::ModeLevelGroundScanCollisionSubB43A8600::Solid,
                        block: crate::ModeLevelGroundScanBlockSubB43A8600::Other,
                    },
                    crate::ModeLevelGroundScanCellSubB43A8600 {
                        collision: crate::ModeLevelGroundScanCollisionSubB43A8600::Solid,
                        block: crate::ModeLevelGroundScanBlockSubB43A8600::Other,
                    }
                ]
            ) =>
        {
            // ASM loc_8BF0 continues Cyclone's low-speed right acceleration
            // through two support scans. Published frame 1530 spans 4,218
            // elapsed cycles with 66 refresh-wait cycles, leaving 3,726 after
            // the shared Execute_Objects shell.
            Some(M68kCycleCount::from_raw(3_726))
        }
        ModeLevelPlayerExecutionPath::GroundedRightWalkContinueLoc8bf0 {
            form: crate::PlayerForm::Cyclone,
            x_velocity_before,
            x_velocity_after,
            ground_scan_cells_sub_b43a_8600,
        } if x_velocity_after.wrapping_sub(*x_velocity_before) == 0x1000
            && matches!(
                ground_scan_cells_sub_b43a_8600.as_slice(),
                [crate::ModeLevelGroundScanCellSubB43A8600 {
                    collision: crate::ModeLevelGroundScanCollisionSubB43A8600::Solid,
                    block: crate::ModeLevelGroundScanBlockSubB43A8600::Other,
                }]
            ) =>
        {
            // ASM loc_8BF0 continues Cyclone's low-speed right acceleration
            // through one support scan. Published frame 1549 spans 3,996
            // elapsed cycles with 62 refresh-wait cycles, leaving 3,508 after
            // the shared Execute_Objects shell.
            Some(M68kCycleCount::from_raw(3_508))
        }
        ModeLevelPlayerExecutionPath::GroundedRightWalkContinueLoc8bf0 {
            form: crate::PlayerForm::Cyclone,
            x_velocity_before: 0x0002_0000,
            x_velocity_after: 0x0002_0000,
            ground_scan_cells_sub_b43a_8600,
        } if matches!(
            ground_scan_cells_sub_b43a_8600.as_slice(),
            [
                crate::ModeLevelGroundScanCellSubB43A8600 {
                    collision: crate::ModeLevelGroundScanCollisionSubB43A8600::Solid,
                    block: crate::ModeLevelGroundScanBlockSubB43A8600::Other,
                },
                crate::ModeLevelGroundScanCellSubB43A8600 {
                    collision: crate::ModeLevelGroundScanCollisionSubB43A8600::Solid,
                    block: crate::ModeLevelGroundScanBlockSubB43A8600::Other,
                }
            ]
        ) =>
        {
            // ASM loc_8BF0 retains Cyclone's capped right-walk speed through
            // two support scans. Published frame 1561 spans 4,232 elapsed
            // cycles with 66 refresh-wait cycles, leaving 3,740 after the
            // shared Execute_Objects shell.
            Some(M68kCycleCount::from_raw(3_740))
        }
        ModeLevelPlayerExecutionPath::GroundedRightWalkContinueLoc8bf0 {
            form: crate::PlayerForm::Cyclone,
            x_velocity_before,
            x_velocity_after,
            ground_scan_cells_sub_b43a_8600,
            ..
        } if x_velocity_after.wrapping_sub(*x_velocity_before) == 0x1C00
            && matches!(
                ground_scan_cells_sub_b43a_8600.as_slice(),
                [
                    crate::ModeLevelGroundScanCellSubB43A8600 {
                        collision: crate::ModeLevelGroundScanCollisionSubB43A8600::Solid,
                        block: crate::ModeLevelGroundScanBlockSubB43A8600::Unflagged,
                    },
                    crate::ModeLevelGroundScanCellSubB43A8600 {
                        collision: crate::ModeLevelGroundScanCollisionSubB43A8600::Solid,
                        block: crate::ModeLevelGroundScanBlockSubB43A8600::Unflagged,
                    },
                ]
            ) =>
        {
            // ASM loc_8BF0 -> loc_8D72 -> loc_8E06, published TAS frame
            // 1173. Consecutive loc_E1E rows span 4,678 cycles; remove the
            // shared 426-cycle scheduler shell.
            Some(M68kCycleCount::from_raw(4_252))
        }
        ModeLevelPlayerExecutionPath::GroundedRightWalkContinueLoc8bf0 {
            form: crate::PlayerForm::Cyclone,
            x_velocity_before: 228_992,
            x_velocity_after: 229_376,
            ground_scan_cells_sub_b43a_8600,
        } if matches!(
            ground_scan_cells_sub_b43a_8600.as_slice(),
            [
                crate::ModeLevelGroundScanCellSubB43A8600 {
                    collision: crate::ModeLevelGroundScanCollisionSubB43A8600::Solid,
                    block: crate::ModeLevelGroundScanBlockSubB43A8600::Unflagged,
                },
                crate::ModeLevelGroundScanCellSubB43A8600 {
                    collision: crate::ModeLevelGroundScanCollisionSubB43A8600::Solid,
                    block: crate::ModeLevelGroundScanBlockSubB43A8600::Unflagged,
                },
            ]
        ) =>
        {
            // ASM loc_8BF0 speed-cap branch, published TAS frame 1182.
            // Consecutive loc_E1E rows span 4,064 cycles; remove the shared
            // 426-cycle scheduler shell.
            Some(M68kCycleCount::from_raw(3_638))
        }
        ModeLevelPlayerExecutionPath::GroundedRightWalkContinueLoc8bf0 {
            form: crate::PlayerForm::Cyclone,
            x_velocity_before: 229_376,
            x_velocity_after: 229_376,
            ground_scan_cells_sub_b43a_8600,
        } if matches!(
            ground_scan_cells_sub_b43a_8600.as_slice(),
            [
                crate::ModeLevelGroundScanCellSubB43A8600 {
                    collision: crate::ModeLevelGroundScanCollisionSubB43A8600::Solid,
                    block: crate::ModeLevelGroundScanBlockSubB43A8600::Unflagged,
                },
                crate::ModeLevelGroundScanCellSubB43A8600 {
                    collision: crate::ModeLevelGroundScanCollisionSubB43A8600::Solid,
                    block: crate::ModeLevelGroundScanBlockSubB43A8600::Unflagged,
                },
            ]
        ) =>
        {
            // ASM loc_8BF0 stable maximum-speed branch, published TAS frame
            // 1183. Consecutive loc_E1E rows span 4,696 cycles; remove the
            // shared 426-cycle scheduler shell.
            Some(M68kCycleCount::from_raw(4_270))
        }
        ModeLevelPlayerExecutionPath::GroundedRightWalkContinueLoc8bf0 {
            form: crate::PlayerForm::Cyclone,
            x_velocity_before: 229_376,
            x_velocity_after: 229_376,
            ground_scan_cells_sub_b43a_8600,
        } if matches!(
            ground_scan_cells_sub_b43a_8600.as_slice(),
            [crate::ModeLevelGroundScanCellSubB43A8600 {
                collision: crate::ModeLevelGroundScanCollisionSubB43A8600::Solid,
                block: crate::ModeLevelGroundScanBlockSubB43A8600::Unflagged,
            }]
        ) =>
        {
            // ASM loc_8BF0 stable maximum-speed branch, published TAS frame
            // 1188. The one-cell support span measures 4,548 cycles between
            // loc_E1E rows, leaving 4,122 after the shared shell.
            Some(M68kCycleCount::from_raw(4_122))
        }
        ModeLevelPlayerExecutionPath::GroundedRightWalkContinueLoc8bf0 {
            form: crate::PlayerForm::TheKid,
            ground_scan_cells_sub_b43a_8600,
            ..
        } => {
            use crate::{
                ModeLevelGroundScanBlockSubB43A8600 as Block,
                ModeLevelGroundScanCellSubB43A8600 as Cell,
                ModeLevelGroundScanCollisionSubB43A8600 as Collision,
            };
            match ground_scan_cells_sub_b43a_8600.as_slice() {
                [
                    Cell {
                        collision: Collision::Solid,
                        block: Block::Unflagged,
                    },
                    Cell {
                        collision: Collision::Solid,
                        block: Block::Unflagged,
                    },
                ] => Some(M68kCycleCount::from_raw(3_584)),
                [
                    Cell {
                        collision: Collision::Other,
                        block: Block::Unflagged,
                    },
                    Cell {
                        collision: Collision::Solid,
                        block: Block::Other,
                    },
                ] => Some(M68kCycleCount::from_raw(3_636)),
                _ => None,
            }
        }
        ModeLevelPlayerExecutionPath::GroundedLeftWalkStartLoc779c {
            form: crate::PlayerForm::Cyclone,
            x_velocity_after: -0x3000,
            ground_scan_cells_sub_b43a_8600,
        } if matches!(
            ground_scan_cells_sub_b43a_8600.as_slice(),
            [crate::ModeLevelGroundScanCellSubB43A8600 {
                collision: crate::ModeLevelGroundScanCollisionSubB43A8600::Solid,
                block: crate::ModeLevelGroundScanBlockSubB43A8600::Other,
            }]
        ) =>
        {
            // ASM loc_7606 -> loc_779C -> loc_7820 -> loc_8BF0, published
            // TAS frame 1131. Consecutive loc_E1E rows span 5,434 raw cycles;
            // remove the shared 426-cycle dispatch/hibernate shell.
            Some(M68kCycleCount::from_raw(5_008))
        }
        ModeLevelPlayerExecutionPath::GroundedLeftWalkContinueLoc8bf0 {
            form: crate::PlayerForm::Cyclone,
            x_velocity_before,
            x_velocity_after,
            ground_scan_cells_sub_b43a_8600,
        } if x_velocity_after.wrapping_sub(*x_velocity_before) == -0x1C00
            && matches!(
                ground_scan_cells_sub_b43a_8600.as_slice(),
                [crate::ModeLevelGroundScanCellSubB43A8600 {
                    collision: crate::ModeLevelGroundScanCollisionSubB43A8600::Solid,
                    block: crate::ModeLevelGroundScanBlockSubB43A8600::Other,
                }]
            ) =>
        {
            // ASM loc_8BF0 -> loc_8D72 -> loc_8E06, published TAS frame
            // 1132. Consecutive loc_E1E rows span 3,972 raw cycles; remove
            // the shared 426-cycle dispatch/hibernate shell.
            Some(M68kCycleCount::from_raw(3_546))
        }
        ModeLevelPlayerExecutionPath::GroundedLeftWalkContinueLoc8bf0 {
            form: crate::PlayerForm::Cyclone,
            x_velocity_before,
            x_velocity_after,
            ground_scan_cells_sub_b43a_8600,
        } if x_velocity_after.wrapping_sub(*x_velocity_before) == -0x1C00
            && matches!(
                ground_scan_cells_sub_b43a_8600.as_slice(),
                [
                    crate::ModeLevelGroundScanCellSubB43A8600 {
                        collision: crate::ModeLevelGroundScanCollisionSubB43A8600::Other,
                        block: crate::ModeLevelGroundScanBlockSubB43A8600::Unflagged,
                    },
                    crate::ModeLevelGroundScanCellSubB43A8600 {
                        collision: crate::ModeLevelGroundScanCollisionSubB43A8600::Solid,
                        block: crate::ModeLevelGroundScanBlockSubB43A8600::Other,
                    },
                ]
            ) =>
        {
            // ASM loc_8BF0 -> sub_B43A -> sub_8600, published TAS frame
            // 1135. The extra clear cell adds 128 raw cycles to the
            // one-cell grounded continuation.
            Some(M68kCycleCount::from_raw(3_674))
        }
        ModeLevelPlayerExecutionPath::GroundedWalkReleaseLoc8c70 {
            form: crate::PlayerForm::Cyclone,
            direction: crate::Direction::Left,
            x_velocity_before,
            x_velocity_after,
        } if x_velocity_after.wrapping_sub(*x_velocity_before) == -0x1C00 => {
            // ASM loc_8BF0 -> loc_8C70 -> loc_A6F8, published TAS frame
            // 1147. Consecutive loc_E1E rows span 2,700 raw cycles; remove
            // the shared 426-cycle dispatch/hibernate shell.
            Some(M68kCycleCount::from_raw(2_274))
        }
        ModeLevelPlayerExecutionPath::GroundedJumpLaunchLocA426 {
            form: crate::PlayerForm::Cyclone,
            x_velocity_before: 229_376,
            x_velocity_after: 172_032,
            y_velocity_after: -329_472,
        } => {
            // ASM loc_8C70 -> loc_A426 -> sub_A432 -> loc_A75A, published
            // TAS frame 1195. Consecutive loc_E1E rows span 4,248 cycles;
            // remove the shared 426-cycle scheduler shell.
            Some(M68kCycleCount::from_raw(3_822))
        }
        ModeLevelPlayerExecutionPath::GroundedJumpLaunchLocA426 {
            form: crate::PlayerForm::Cyclone,
            x_velocity_before: 0,
            x_velocity_after: 0,
            y_velocity_after: -270_336,
        } => {
            // ASM loc_8C70 -> loc_A426 -> sub_A432 -> loc_A75A, stationary
            // Cyclone launch on published frame 1601. The player loc_E1E span
            // is 4,284 elapsed cycles with 72 refresh-wait cycles, leaving
            // 3,786 after the shared Execute_Objects shell.
            Some(M68kCycleCount::from_raw(3_786))
        }
        ModeLevelPlayerExecutionPath::GroundedJumpLaunchLocA426 {
            form: crate::PlayerForm::TheKid,
            x_velocity_before: 0x0000_3800,
            x_velocity_after: 0x0000_2A00,
            y_velocity_after: -273_504,
        } => {
            // ASM loc_8C70 -> loc_A426 -> sub_A432 -> loc_A75A, published
            // TAS frame 981: selected loc_E1E rows plus the ROM refresh
            // recurrence pin 4,180 source cycles. Removing the shared
            // 426-cycle scheduler shell leaves 3,754.
            Some(M68kCycleCount::from_raw(3_754))
        }
        ModeLevelPlayerExecutionPath::GroundedJumpLaunchLocA426 {
            form: crate::PlayerForm::TheKid,
            ..
        } => {
            // ASM loc_8BF0 -> loc_8C70 -> loc_A426 -> sub_A432 -> loc_A75A
            // -> loc_A6F8. Frame 933's selected-PC journal spans 4,114 raw
            // cycles between loc_E1E dispatches; remove the shared 426-cycle
            // dispatch/hibernate shell.
            Some(M68kCycleCount::from_raw(3_688))
        }
        ModeLevelPlayerExecutionPath::WallClingPullUpLocAbf0 {
            form: crate::PlayerForm::TheKid,
            side: crate::Direction::Right,
            pull_up_scan_rows: 1,
        } => {
            // ASM loc_AA22 -> kid_flip -> loc_ABA4 -> loc_ABF0 ->
            // PlaySound -> Hibernate_UntilAnimFinished. Frame 957's
            // selected-PC journal spans 4,694 raw cycles between loc_E1E
            // dispatches; remove the shared 426-cycle scheduler shell.
            Some(M68kCycleCount::from_raw(4_268))
        }
        ModeLevelPlayerExecutionPath::PullUpAnimationFinished {
            form: crate::PlayerForm::TheKid,
        } => {
            // ASM loc_1062 -> sub_78E8 -> loc_75D4, published TAS frame 978:
            // selected loc_E1E rows span 1,634 raw cycles. Removing the
            // shared 426-cycle dispatch/hibernate shell leaves 1,208 here.
            Some(M68kCycleCount::from_raw(1_208))
        }
        ModeLevelPlayerExecutionPath::PullUpAnimationHibernateLoc1062 {
            form: crate::PlayerForm::TheKid,
            ..
        } => {
            // ASM loc_1062 -> j_Hibernate_Object_1Frame: after loc_E1E has
            // restored the coroutine PC, the runnable callback executes only
            // the four-cycle JSR veneer before sub_E76 returns to loc_E1E.
            Some(M68kCycleCount::from_raw(4))
        }
        ModeLevelPlayerExecutionPath::NormalPhysics(_)
        | ModeLevelPlayerExecutionPath::WallClingNoButtonsLocAa22 { .. } => {
            player_callback_cycles_loc_a73a(execute)
        }
        ModeLevelPlayerExecutionPath::LevelStartActivationLoc7508 { .. }
        | ModeLevelPlayerExecutionPath::GroundedRightWalkStartLoc779c { .. }
        | ModeLevelPlayerExecutionPath::GroundedRightWalkContinueLoc8bf0 { .. }
        | ModeLevelPlayerExecutionPath::GroundedLeftWalkStartLoc779c { .. }
        | ModeLevelPlayerExecutionPath::GroundedLeftWalkContinueLoc8bf0 { .. }
        | ModeLevelPlayerExecutionPath::GroundedWalkReleaseLoc8c70 { .. }
        | ModeLevelPlayerExecutionPath::GroundedJumpLaunchLocA426 { .. }
        | ModeLevelPlayerExecutionPath::WallClingPullUpLocAbf0 { .. }
        | ModeLevelPlayerExecutionPath::PullUpAnimationHibernateLoc1062 { .. }
        | ModeLevelPlayerExecutionPath::PullUpAnimationFinished { .. }
        | ModeLevelPlayerExecutionPath::Unclassified => None,
    }
}

/// Nominal E1E-to-E1E cycles for the fixed damage-flash callback.
pub fn damage_flash_wait_cycles_loc_bc44(
    path: ModeLevelDamageFlashWaitLocBc44,
) -> Option<M68kCycleCount> {
    match path {
        // ASM loc_BC34 -> loc_BC40 -> sub_E76: the first dispatch and
        // initial hibernate span 444 raw cycles from loc_E1E to loc_E1E.
        ModeLevelDamageFlashWaitLocBc44::InitialHibernateLocBc34 => Some(M68kCycleCount(444)),
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
        // ASM loc_119D8 -> sub_E76: clear Ctrl_Pressed and hibernate.
        ModeLevelDemoExitWaitLoc119E0::InitialHibernateLoc119D8 => Some(M68kCycleCount(490)),
        // ASM loc_119D8..loc_119E0 (kid.asm:23595-23600): the runnable
        // dispatch, hibernate, and clear Start test total 450 raw CPU cycles.
        ModeLevelDemoExitWaitLoc119E0::HibernateNoStart => Some(M68kCycleCount(450)),
        ModeLevelDemoExitWaitLoc119E0::ExitDemoStart => None,
    }
}

/// Raw E1E-to-E1E cycles for the fixed input/pause callback.
pub fn input_worker_cycles_loc_7452(path: ModeLevelInputWorkerPathLoc7452) -> M68kCycleCount {
    match path {
        // ASM loc_7452 -> sub_E76: the first dispatch immediately hibernates.
        ModeLevelInputWorkerPathLoc7452::InitialHibernate => M68kCycleCount(408),
        // ASM loc_7452..loc_74B0 (kid.asm:7586-7625): the clear timer,
        // Left-clear controller, demo-mode, and clear Start path totals 636
        // raw cycles. Right alone takes the same first-test branch.
        ModeLevelInputWorkerPathLoc7452::DemoNoLeftNoStartTimerClear => M68kCycleCount(636),
        // ASM loc_7464 -> loc_749C: held Left executes the additional
        // Ctrl_Right_Held test before taking the shared demo path.
        ModeLevelInputWorkerPathLoc7452::DemoLeftOnlyNoStartTimerClear => M68kCycleCount(660),
    }
}

/// Raw E1E-to-E1E cost for the fixed first level-start palette callback.
pub fn palette_fade_worker_cycles_sub_73d0(
    path: ModeLevelPaletteFadeWorkerPathSub73D0,
) -> M68kCycleCount {
    match path {
        // ASM sub_73D0 copies 32 longwords and 64 words, seeds `$F888/$F88C`,
        // then reaches loc_7402's first sub_E76 hibernate.
        ModeLevelPaletteFadeWorkerPathSub73D0::InitialCopyAndHibernate => M68kCycleCount(2_932),
        // ASM loc_7402..loc_7414 (kid.asm:7547-7557): runnable dispatch,
        // hibernate, `$F876 += $10`, the <=$100 branch, both -1 mask stores,
        // and the back edge total 806 raw cycles from loc_E1E to loc_E1E.
        ModeLevelPaletteFadeWorkerPathSub73D0::AdvanceFadeAndHibernateLoc7402 => {
            M68kCycleCount(806)
        }
    }
}

fn is_first_level_start_execute_objects_sub_e12(execute: &ModeLevelExecuteObjectsWork) -> bool {
    execute.player == crate::ModeLevelPlayerWorkBeforeHud::Activation
        && execute.linked_non_transient_object_count == 5
        && execute.prize_block_wobble_workers_sub_1007a == 0
        && execute.prize_block_worker_tails_sub_1007a == 0
        && execute.prize_block_flash_workers_loc_102b8 == 0
        && execute.floating_prize_workers == 0
        && execute.rubber_block_skin_workers_sub_10de4 == 0
        && execute.active_enemy_workers == 0
        && execute.active_platform_workers == 0
        && execute.palette_rotation_worker_loc_11f96 == 0
        && execute.unclassified_runtime_transient_workers == 0
        && execute.player_entry_position
            == Some(crate::ModeLevelPlayerPosition {
                x_pos: 0x0020_0000,
                y_pos: 0x02DD_0000,
            })
        && matches!(
            execute.prize_object_execution_order_sub_e12.as_deref(),
            Some([ModeLevelPrizeObjectExecutionSubE12::Player])
        )
        && matches!(
            execute.executed_player_path,
            Some(ModeLevelPlayerExecutionPath::LevelStartActivationLoc7508 {
                form: crate::PlayerForm::TheKid,
            })
        )
        && execute.damage_flash_wait_loc_bc44
            == Some(ModeLevelDamageFlashWaitLocBc44::InitialHibernateLocBc34)
        && execute.demo_exit_wait_loc_119e0
            == Some(ModeLevelDemoExitWaitLoc119E0::InitialHibernateLoc119D8)
        && execute.input_worker_loc_7452 == Some(ModeLevelInputWorkerPathLoc7452::InitialHibernate)
        && execute.palette_fade_worker_sub_73d0
            == Some(ModeLevelPaletteFadeWorkerPathSub73D0::InitialCopyAndHibernate)
}

/// Raw 68000 cycles for one typed prize callback, measured from its
/// `Execute_Objects` loc_E1E list read through the following loc_E1E read.
///
/// `None` preserves branch uncertainty. VDP bus stalls are deliberately
/// excluded and belong to the raster scheduler.
// ASM loc_E1E -> loc_E3E -> callback costs 224 cycles on the runnable
// path. A one-frame hibernate at sub_E76 costs another 158 raw cycles before
// the next loc_E1E.
const RUNNABLE_HIBERNATE_SHELL_CYCLES: u32 = 380;

pub fn prize_object_cycles_sub_e12(
    execute: &ModeLevelExecuteObjectsWork,
    callback: ModeLevelPrizeObjectExecutionSubE12,
) -> Option<M68kCycleCount> {
    const RUNNABLE_DISPATCH_CYCLES: u32 = 222;

    let callback_cycles = match callback {
        ModeLevelPrizeObjectExecutionSubE12::FloatingPrize { index } => {
            let work = execute.floating_prize_paths.get(usize::from(index))?;
            let is_helmet = matches!(
                work.content,
                crate::terrain::PrizeContent::Skycutter
                    | crate::terrain::PrizeContent::Cyclone
                    | crate::terrain::PrizeContent::RedStealth
                    | crate::terrain::PrizeContent::Eyeclops
                    | crate::terrain::PrizeContent::Juggernaut
                    | crate::terrain::PrizeContent::IronKnight
                    | crate::terrain::PrizeContent::Berzerker
                    | crate::terrain::PrizeContent::Maniaxe
                    | crate::terrain::PrizeContent::Micromax
            );
            match (is_helmet, work.fall_path_loc_103b2?) {
                // ASM PrizeHelmetCyclone_Init loc_104AE -> sub_10C38. Oracle
                // selected-PC frames 1041/1042 measure 606/634 raw cycles
                // between loc_E1E reads, leaving 226/254 after the shell.
                (true, ModeLevelFloatingPrizeFallPathLoc103B2::GraceProbeFallSameRow) => 226,
                (true, ModeLevelFloatingPrizeFallPathLoc103B2::GraceProbeFallAdvanceRow) => 254,
                // After sub_10C38 clears bit $10, frame 1043 takes the direct
                // terrain-check path: 630 raw cycles between loc_E1E reads.
                (true, ModeLevelFloatingPrizeFallPathLoc103B2::DirectFallSameRow) => 250,
                (true, ModeLevelFloatingPrizeFallPathLoc103B2::DirectFallAdvanceRow) => 278,
                // ASM loc_103B2 -> loc_103DE -> loc_10404 -> loc_10396.
                (false, ModeLevelFloatingPrizeFallPathLoc103B2::DirectFallSameRow) => 232,
                // ASM loc_103F4 takes the changed-row branch through BCLR and
                // advances a2 by Level_width_tiles before rejoining loc_10402.
                (false, ModeLevelFloatingPrizeFallPathLoc103B2::DirectFallAdvanceRow) => 260,
                // The lower-cell Level_Layout probe adds 78 cycles.
                (false, ModeLevelFloatingPrizeFallPathLoc103B2::GraceProbeFallSameRow) => 308,
                (false, ModeLevelFloatingPrizeFallPathLoc103B2::GraceProbeFallAdvanceRow) => 336,
                // ASM loc_103DA skips gravity and rejoins the bottom/collision
                // checks at loc_10404.
                (false, ModeLevelFloatingPrizeFallPathLoc103B2::BlockedCurrentRow) => 190,
                // Blocked-grace costs need an independent source probe before
                // they can enter the cycle scheduler.
                _ => return None,
            }
        }
        ModeLevelPrizeObjectExecutionSubE12::FloatingPrizeInitialSub102F8 { index } => {
            let work = execute.floating_prize_paths.get(usize::from(index))?;
            if !work.initial_sub_102f8 {
                return None;
            }
            // ASM prize initialiser at sub_102F8, published TAS frame 1399.
            // The complete-address journal measures the whole E1E callback.
            return Some(M68kCycleCount::from_raw(1_790));
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
        ModeLevelPrizeObjectExecutionSubE12::PlayerSpawnedBlockDestroyInitialSub10EDA => {
            execute.player_spawned_block_destroy_list_work_sub_10e86?;
            // ASM Block_LoadSmashBits sub_10E86 inserts sub_10EDA after Kid;
            // its first loc_E1E callback is 888 raw cycles.
            return Some(M68kCycleCount::from_raw(888));
        }
        ModeLevelPrizeObjectExecutionSubE12::FlashLoc102B8 { index } => {
            let work = execute
                .prize_block_flash_paths_sub_1066
                .get(usize::from(index))?;
            match work.wake {
                // ASM loc_1066 -> loc_1062 -> sub_E76.
                ModeLevelPrizeAnimationWake::Hibernate => 50,
                ModeLevelPrizeAnimationWake::FinishAndDelete => {
                    return Some(prize_flash_finish_cycles_loc_102b8(work.delete_list_work?));
                }
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
        ModeLevelPrizeObjectExecutionSubE12::CollectedDiamond { index } => {
            return Some(collected_diamond_worker_cycles_sub_e12(
                *execute.collected_diamond_workers.get(usize::from(index))?,
            ));
        }
    };
    Some(M68kCycleCount(
        RUNNABLE_HIBERNATE_SHELL_CYCLES + callback_cycles,
    ))
}

/// Complete `Execute_Objects` visit cost for one collected-Diamond `$6000`
/// worker pass (ASM `PrizeDiamondCollected_Init` / `Increase_Diamonds`).
pub fn collected_diamond_worker_cycles_sub_e12(
    phase: crate::ModeLevelCollectedDiamondWorkerPhase,
) -> M68kCycleCount {
    match phase {
        crate::ModeLevelCollectedDiamondWorkerPhase::Initial => {
            // ASM PrizeDiamondCollected_Init through the first
            // Increase_Diamonds hibernate. The frame-1409 complete E1E
            // journal measures 546 elapsed = 538 raw + four refreshes;
            // `$19(a3)` suppresses the duplicate PlaySound path.
            M68kCycleCount::from_raw(538)
        }
        crate::ModeLevelCollectedDiamondWorkerPhase::Resumed { .. } => {
            // ASM Increase_Diamonds loc_FEFE: each resumed iteration updates
            // the cached HUD arc, writes world coordinates, DBFs, then
            // hibernates. Frame-1410 E1E journal: 548 elapsed from phase 96
            // to phase 4 is 540 raw plus four refreshes (phase 96 + 548
            // wraps to 4 mod 128, so the elapsed span pins raw at 540). The
            // shared runnable/hibernate shell is 380 raw; loc_FEFE plus the
            // short-jump trampoline and JSR contribute the remaining 160.
            M68kCycleCount::from_raw(RUNNABLE_HIBERNATE_SHELL_CYCLES + 160)
        }
        crate::ModeLevelCollectedDiamondWorkerPhase::Terminal => {
            // ASM Increase_Diamonds terminal DBF fall-through increments
            // Number_Diamonds and deletes both Object/GfxObject slots.
            // Terminal-frame E1E rows measure 1,400 elapsed from phase 96 to
            // 88: 1,378 raw plus eleven refresh events. Diamond count 1
            // skips both power-available PlaySound branches.
            M68kCycleCount::from_raw(1_378)
        }
    }
}

/// Complete raw CPU cost of the source-typed natural `Execute_Objects` pass.
///
/// Refresh, VDP, and Z80 waits are intentionally excluded. Any callback class
/// not yet represented by the ledger keeps the result unknown.
pub fn execute_objects_raw_cycles_sub_e12(
    execute: &ModeLevelExecuteObjectsWork,
) -> Option<M68kCycleCount> {
    if is_first_level_start_execute_objects_sub_e12(execute) {
        let cycles = execute_objects_setup_cycles_sub_e12(execute.rng_offset_before_sub_e12?)?
            .get()
            // ASM loc_E1E callback order follows the selector-linked Object
            // list: input, Kid, damage, palette fade, demo exit.
            .checked_add(input_worker_cycles_loc_7452(execute.input_worker_loc_7452?).get())?
            // Frame-902 selected PCs pin loc_E1E -> loc_7508 -> loc_E1E.
            .checked_add(4_804)?
            .checked_add(
                damage_flash_wait_cycles_loc_bc44(execute.damage_flash_wait_loc_bc44?)?.get(),
            )?
            .checked_add(
                palette_fade_worker_cycles_sub_73d0(execute.palette_fade_worker_sub_73d0?).get(),
            )?
            .checked_add(demo_exit_wait_cycles_loc_119e0(execute.demo_exit_wait_loc_119e0?)?.get())?
            // ASM loc_E1E's null-list return reaches sub_21F8 in 82 raw
            // cycles. One separate GEMS bank wait completes the 85-cycle
            // oracle interval after refresh.
            .checked_add(82)?;
        return Some(M68kCycleCount(cycles));
    }

    let terminal_collected_diamond = execute.linked_non_transient_object_count == 5
        && execute.collected_diamond_workers
            == [crate::ModeLevelCollectedDiamondWorkerPhase::Terminal];
    let materialised_diamond_replaces_tail = execute.linked_non_transient_object_count == 3
        && execute
            .prize_object_execution_order_sub_e12
            .as_deref()
            .is_some_and(|order| {
                order.iter().any(|callback| {
                    matches!(
                        callback,
                        ModeLevelPrizeObjectExecutionSubE12::FloatingPrizeInitialSub102F8 { .. }
                    )
                })
            });
    if (execute.linked_non_transient_object_count != 4
        && !terminal_collected_diamond
        && !materialised_diamond_replaces_tail)
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
            cycles = cycles.checked_add(player_callback_cycles_sub_e12(execute)?.get() + 426)?;
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
        .checked_add(
            execute
                .palette_fade_worker_sub_73d0
                .map(palette_fade_worker_cycles_sub_73d0)
                .map_or(0, M68kCycleCount::get),
        )?
        .checked_add(block_worker_raw_cycles_sub_10eda(execute)?)?
        .checked_add(demo_exit_wait_cycles_loc_119e0(execute.demo_exit_wait_loc_119e0?)?.get())?
        // ASM loc_E1E's final null-list return reaches Mode_Level $7E6 in 54
        // raw cycles. The following 18-cycle BSR.W sub_21F8 belongs to the
        // post-Execute span.
        .checked_add(54)?;
    Some(M68kCycleCount(cycles))
}

fn block_worker_raw_cycles_sub_10eda(execute: &ModeLevelExecuteObjectsWork) -> Option<u32> {
    // ASM sub_292E (kid.asm:4881-4945) never scans the debris scratch array:
    // it writes at the `$FFFFFA9C` cursor and blindly advances `+$52`, and
    // debris slots are not Object_RAM callbacks, so the live-swarm count
    // cannot alter any destroy-worker cost. The per-worker constants below
    // are therefore debris-independent by instruction sequence, not merely
    // by measurement (frames 1348/1349/1366/1367 corroborate).
    // TODO(fidelity): the sub_292E body still varies with the `d6` direction
    // dispatch (42..80 cycles) and the every-6th-allocation cursor rewind
    // (+14); the 2,382 first-pass constant bakes in one observed shape. Type
    // those inputs if a frame ever disagrees by exactly those deltas.
    let mut cycles = 0_u32;
    for worker in &execute.block_destroy_workers_sub_10eda {
        cycles = cycles.checked_add(match worker {
            // Published TAS frame 1348 full-PC journal, E1E-to-E1E.
            crate::ModeLevelBlockDestroyWorkerSub10EDA::Hibernate => 72,
            // Published TAS frame 1349: loc_10F38 (72), then the newly
            // linked sub_292E worker's first pass (2,382).
            crate::ModeLevelBlockDestroyWorkerSub10EDA::HibernateThenDebrisInitial => 2_454,
        })?;
    }
    Some(cycles)
}

/// Source-positioned non-refresh waits inside the typed Execute_Objects pass.
pub fn execute_objects_external_waits_sub_e12(
    execute: &ModeLevelExecuteObjectsWork,
) -> Option<Vec<M68kExternalWaitEvent>> {
    if is_first_level_start_execute_objects_sub_e12(execute) {
        return Some(vec![M68kExternalWaitEvent {
            raw_offset: execute_objects_raw_cycles_sub_e12(execute)?,
            cycles: M68kCycleCount::from_raw(1),
        }]);
    }

    if matches!(
        execute.executed_player_path,
        Some(ModeLevelPlayerExecutionPath::WallClingPullUpLocAbf0 {
            form: crate::PlayerForm::TheKid,
            side: crate::Direction::Right,
            pull_up_scan_rows: 1,
        })
    ) {
        let mut raw_before_player =
            execute_objects_setup_cycles_sub_e12(execute.rng_offset_before_sub_e12?)?
                .get()
                .checked_add(input_worker_cycles_loc_7452(execute.input_worker_loc_7452?).get())?;
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
        // ASM loc_ABF0 -> PlaySound -> sub_E151C -> sub_E143A/sub_E148E.
        // Relative to the callback entry, the Z80 bus request, grant read,
        // command-head read, marker write, sound-ID write, and updated-head
        // write complete at these six raw source offsets. Frame 957's
        // selected-PC journal confirms all six one-cycle waits.
        return [3_792, 3_852, 3_878, 3_898, 3_962, 3_982]
            .into_iter()
            .map(|offset| {
                Some(M68kExternalWaitEvent {
                    raw_offset: M68kCycleCount(
                        raw_before_player.checked_add(268)?.checked_add(offset)?,
                    ),
                    cycles: M68kCycleCount::from_raw(1),
                })
            })
            .collect();
    }

    if execute
        .player_spawned_block_destroy_list_work_sub_10e86
        .is_some()
    {
        let mut raw_before_player =
            execute_objects_setup_cycles_sub_e12(execute.rng_offset_before_sub_e12?)?
                .get()
                .checked_add(input_worker_cycles_loc_7452(execute.input_worker_loc_7452?).get())?;
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
        // ASM Block_LoadSmashBits sub_10E86 calls PlaySound. The frame-1399
        // complete-address journal puts the wrapper 4,206 raw cycles after
        // Kid's loc_E1E callback entry.
        return [238, 298, 324, 344, 408, 428]
            .into_iter()
            .map(|offset| {
                Some(M68kExternalWaitEvent {
                    raw_offset: M68kCycleCount(
                        raw_before_player.checked_add(4_206)?.checked_add(offset)?,
                    ),
                    cycles: M68kCycleCount::from_raw(1),
                })
            })
            .collect();
    }

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

fn append_rom_advance(total: &mut M68kRefreshAdvance, raw: M68kCycleCount) -> Option<()> {
    let advance = total.phase_after.advance_rom(raw);
    total.elapsed = M68kCycleCount(total.elapsed.get().checked_add(advance.elapsed.get())?);
    total.phase_after = advance.phase_after;
    total.refresh_events = total.refresh_events.checked_add(advance.refresh_events)?;
    Some(())
}

fn append_rom_instruction(total: &mut M68kRefreshAdvance, raw: M68kCycleCount) -> Option<()> {
    let phase_after_raw = u32::from(total.phase_after.get()).checked_add(raw.get())?;
    let refresh_wait = u32::from(phase_after_raw >= 128) * 2;
    total.phase_after = M68kRefreshPhase::new(
        u8::try_from((phase_after_raw + refresh_wait) % 128)
            .expect("instruction refresh phase remains below 128"),
    )?;
    total.elapsed = M68kCycleCount(
        total
            .elapsed
            .get()
            .checked_add(raw.get())?
            .checked_add(refresh_wait)?,
    );
    total.refresh_events = total
        .refresh_events
        .checked_add(u32::from(refresh_wait != 0))?;
    Some(())
}

fn append_execute_component(
    total: &mut M68kRefreshAdvance,
    raw_offset: &mut u32,
    raw: M68kCycleCount,
) -> Option<()> {
    append_rom_advance(total, raw)?;
    *raw_offset = raw_offset.checked_add(raw.get())?;
    Some(())
}

/// Advance one runnable Kid callback containing ASM `PlaySound sub_E1510`.
///
/// The surrounding player path is source-typed but not instruction-visited.
/// Its two long ROM spans are safe at their real call boundary; the shared
/// GEMS transaction must use its existing instruction scheduler because six
/// direct Z80-RAM accesses and refresh are interleaved inside `sub_E151C`.
fn player_callback_with_play_sound_advance_sub_e12(
    phase_before: M68kRefreshPhase,
    raw_total: M68kCycleCount,
    direct_wait_offsets: &[M68kExternalWaitEvent],
) -> Option<M68kRefreshAdvance> {
    const FIRST_DIRECT_WAIT_AFTER_WRAPPER_ENTRY: u32 = 238;
    const GET_XY_DIVU_BEFORE_WRAPPER_ENTRY: u32 = 420;
    const GET_XY_DIVU_RAW: u32 = 142;
    const PLAY_SOUND_WRAPPER_RAW: u32 = 586;
    const PLAY_SOUND_CORE_RAW: u32 = 504;
    const DIRECT_WAIT_OFFSETS: [u32; 6] = [238, 298, 324, 344, 408, 428];

    if direct_wait_offsets.len() != DIRECT_WAIT_OFFSETS.len()
        || direct_wait_offsets
            .iter()
            .any(|event| event.cycles != M68kCycleCount::from_raw(1))
    {
        return None;
    }
    let wrapper_offset = direct_wait_offsets[0]
        .raw_offset
        .get()
        .checked_sub(FIRST_DIRECT_WAIT_AFTER_WRAPPER_ENTRY)?;
    if !direct_wait_offsets
        .iter()
        .zip(DIRECT_WAIT_OFFSETS)
        .all(|(event, expected)| event.raw_offset.get() == wrapper_offset + expected)
    {
        return None;
    }
    let suffix_raw = raw_total
        .get()
        .checked_sub(wrapper_offset)?
        .checked_sub(PLAY_SOUND_WRAPPER_RAW)?;
    let divu_offset = wrapper_offset.checked_sub(GET_XY_DIVU_BEFORE_WRAPPER_ENTRY)?;
    let between_divu_and_wrapper = wrapper_offset
        .checked_sub(divu_offset)?
        .checked_sub(GET_XY_DIVU_RAW)?;

    let mut total = M68kRefreshAdvance {
        elapsed: M68kCycleCount::from_raw(0),
        phase_after: phase_before,
        refresh_events: 0,
    };
    append_rom_advance(&mut total, M68kCycleCount::from_raw(divu_offset))?;
    // ASM Get_XY_From_LevelLayoutAddress sub_FACE: DIVU is one 142-cycle
    // instruction. jgenesis can enqueue one refresh wait at its completion;
    // aggregate recurrence incorrectly invents a second wait inside it.
    append_rom_instruction(&mut total, M68kCycleCount::from_raw(GET_XY_DIVU_RAW))?;
    append_rom_advance(
        &mut total,
        M68kCycleCount::from_raw(between_divu_and_wrapper),
    )?;
    // ASM PlaySound $E1510..sub_E151C and its register-save wrapper.
    for raw in [12, 12, 18] {
        append_rom_advance(&mut total, M68kCycleCount::from_raw(raw))?;
    }
    let command = gems_queue_command_advance_with_scheduler(
        GemsQueueCommandPath::StartOrStopSequenceSubE151c,
        total.phase_after,
        |_, _| Some(M68kCycleCount::from_raw(0)),
        |_| {},
    )?;
    let command_refresh_cycles = command
        .elapsed
        .get()
        .checked_sub(PLAY_SOUND_CORE_RAW)?
        .checked_sub(DIRECT_WAIT_OFFSETS.len() as u32)?;
    if command_refresh_cycles % 2 != 0 {
        return None;
    }
    total.elapsed = M68kCycleCount(total.elapsed.get().checked_add(command.elapsed.get())?);
    total.phase_after = command.phase_after;
    total.refresh_events = total
        .refresh_events
        .checked_add(command_refresh_cycles / 2)?;
    for raw in [12, 12, 16] {
        append_rom_advance(&mut total, M68kCycleCount::from_raw(raw))?;
    }
    append_rom_advance(&mut total, M68kCycleCount::from_raw(suffix_raw))?;
    Some(total)
}

/// Execute the raw pass through the Genesis refresh recurrence and typed
/// external waits.
pub fn execute_objects_elapsed_cycles_sub_e12(
    execute: &ModeLevelExecuteObjectsWork,
    phase_before: M68kRefreshPhase,
) -> Option<M68kRefreshAdvance> {
    if !is_first_level_start_execute_objects_sub_e12(execute) {
        let raw_total = execute_objects_raw_cycles_sub_e12(execute)?;
        let waits = execute_objects_external_waits_sub_e12(execute)?;
        if !waits.is_empty()
            && matches!(
                execute.executed_player_path,
                Some(ModeLevelPlayerExecutionPath::WallClingPullUpLocAbf0 { .. })
            )
        {
            // The wall-cling PlaySound path does not call sub_FACE. Retain its
            // existing aggregate characterisation until that distinct source
            // prefix has an instruction boundary of its own.
            let raw = phase_before.advance_rom(raw_total);
            let mut elapsed = raw.elapsed.get();
            let mut phase = raw.phase_after;
            for event in waits {
                let wait = phase.advance_external_wait(event.cycles);
                elapsed = elapsed.checked_add(wait.elapsed.get())?;
                phase = wait.phase_after;
            }
            return Some(M68kRefreshAdvance {
                elapsed: M68kCycleCount(elapsed),
                phase_after: phase,
                refresh_events: raw.refresh_events,
            });
        }
        let mut total = M68kRefreshAdvance {
            elapsed: M68kCycleCount::from_raw(0),
            phase_after: phase_before,
            refresh_events: 0,
        };
        let mut raw_offset = 0_u32;
        let mut consumed_waits = 0_usize;

        append_execute_component(
            &mut total,
            &mut raw_offset,
            execute_objects_setup_cycles_sub_e12(execute.rng_offset_before_sub_e12?)?,
        )?;
        // ASM loc_E1E walks selector-linked order. The fixed input worker is
        // the first ordinary callback, before prizes and Kid.
        append_execute_component(
            &mut total,
            &mut raw_offset,
            input_worker_cycles_loc_7452(execute.input_worker_loc_7452?),
        )?;
        for callback in execute.prize_object_execution_order_sub_e12.as_ref()? {
            if *callback == ModeLevelPrizeObjectExecutionSubE12::Player {
                let raw = M68kCycleCount(
                    player_callback_cycles_sub_e12(execute)?
                        .get()
                        .checked_add(426)?,
                );
                let player_end = raw_offset.checked_add(raw.get())?;
                let player_waits = waits
                    .iter()
                    .filter(|event| {
                        let offset = event.raw_offset.get();
                        offset >= raw_offset && offset < player_end
                    })
                    .map(|event| M68kExternalWaitEvent {
                        raw_offset: M68kCycleCount::from_raw(event.raw_offset.get() - raw_offset),
                        cycles: event.cycles,
                    })
                    .collect::<Vec<_>>();
                if player_waits.is_empty() {
                    append_execute_component(&mut total, &mut raw_offset, raw)?;
                } else {
                    let advance = player_callback_with_play_sound_advance_sub_e12(
                        total.phase_after,
                        raw,
                        &player_waits,
                    )?;
                    total.elapsed =
                        M68kCycleCount(total.elapsed.get().checked_add(advance.elapsed.get())?);
                    total.phase_after = advance.phase_after;
                    total.refresh_events =
                        total.refresh_events.checked_add(advance.refresh_events)?;
                    raw_offset = player_end;
                    consumed_waits = consumed_waits.checked_add(player_waits.len())?;
                }
            } else {
                append_execute_component(
                    &mut total,
                    &mut raw_offset,
                    prize_object_cycles_sub_e12(execute, *callback)?,
                )?;
            }
        }
        {
            // Debris scratch slots are compiled later by sub_2A4C and do not
            // own an Execute_Objects callback; sub_292E never scans them, so
            // the destroy-worker components are debris-independent (see
            // block_worker_raw_cycles_sub_10eda).
            for worker in &execute.block_destroy_workers_sub_10eda {
                match worker {
                    crate::ModeLevelBlockDestroyWorkerSub10EDA::Hibernate => {
                        append_execute_component(
                            &mut total,
                            &mut raw_offset,
                            M68kCycleCount::from_raw(72),
                        )?;
                    }
                    crate::ModeLevelBlockDestroyWorkerSub10EDA::HibernateThenDebrisInitial => {
                        append_execute_component(
                            &mut total,
                            &mut raw_offset,
                            M68kCycleCount::from_raw(72),
                        )?;
                        append_execute_component(
                            &mut total,
                            &mut raw_offset,
                            M68kCycleCount::from_raw(2_382),
                        )?;
                    }
                }
            }
        }
        append_execute_component(
            &mut total,
            &mut raw_offset,
            damage_flash_wait_cycles_loc_bc44(execute.damage_flash_wait_loc_bc44?)?,
        )?;
        if let Some(palette) = execute.palette_fade_worker_sub_73d0 {
            append_execute_component(
                &mut total,
                &mut raw_offset,
                palette_fade_worker_cycles_sub_73d0(palette),
            )?;
        }
        append_execute_component(
            &mut total,
            &mut raw_offset,
            demo_exit_wait_cycles_loc_119e0(execute.demo_exit_wait_loc_119e0?)?,
        )?;
        append_execute_component(&mut total, &mut raw_offset, M68kCycleCount::from_raw(54))?;
        if raw_offset != raw_total.get() || consumed_waits != waits.len() {
            return None;
        }
        return Some(total);
    }

    let raw_total = execute_objects_raw_cycles_sub_e12(execute)?.get();
    let events = execute_objects_external_waits_sub_e12(execute)?;
    let raw = phase_before.advance_rom(M68kCycleCount(raw_total));
    let mut elapsed = raw.elapsed.get();
    let mut phase = raw.phase_after;
    // jgenesis timing.rs `record_68k_instruction`: a pending bus wait advances
    // `m68k_refresh_counter` but `m68k_wait` suppresses a new refresh enqueue.
    // Aggregate those non-refresh waits after solving the ROM instruction
    // stream; their source offsets remain available above for raster work.
    for event in events {
        let wait = phase.advance_external_wait(event.cycles);
        elapsed = elapsed.checked_add(wait.elapsed.get())?;
        phase = wait.phase_after;
    }
    Some(M68kRefreshAdvance {
        elapsed: M68kCycleCount(elapsed),
        phase_after: phase,
        refresh_events: raw.refresh_events,
    })
}

/// Extra raw cycles ASM `loc_EB4C` spends rejecting one bit-15 cell whose
/// block type is not `$A`.
///
/// The cheap cell is `move.w (a1)+,d0` (8), `bclr #$F,d0` (12), and a taken
/// `beq.w` (10). A flagged cell takes the `beq.w` not taken (12) and then
/// `andi.w #$F00,d0` (8), `cmpi.w #$A00,d0` (8), and a second not-taken
/// `beq.w` (12): 30 cycles more. Published TAS frames 919 and 978 measure
/// 2,462 and 2,492 over an otherwise identical scan, confirming the 30.
///
const FLAGGED_CELL_REJECTION_RAW_CYCLES: u32 = 30;

/// Complete raw CPU cost from the return of ASM `sub_21F8` through the entry
/// of `sub_44DC` for the typed natural Mode_Level path.
pub fn post_execute_raw_cycles_before_sub_44dc(
    ledger: &ModeLevelPreHudWorkLedger,
) -> Option<M68kCycleCount> {
    let post = &ledger.post_execute;
    let scan = post.evanescent_scan_loc_eae4?;
    let shooter = post.shooter_scan_sub_f096?;
    if post.active_evanescent_slots_before_sub_dfb0 != 0
        || scan.found_evanescent_block
        || post.active_shooter_slots_before_sub_f7e0 != 0
        || post.level_special_effects_sub_6034 > 1
        || shooter.active_shooter_slots_before != 0
    {
        return None;
    }

    if matches!(
        ledger.execute_objects.executed_player_path,
        Some(
            ModeLevelPlayerExecutionPath::PullUpAnimationHibernateLoc1062 {
                form: crate::PlayerForm::TheKid,
                ..
            }
        )
    ) {
        // ASM loc_7E6 -> sub_21F8 -> sub_DFB0 -> loc_EAE4 -> sub_F096 ->
        // sub_44DC: Kid is stationary throughout loc_1062's animation wait,
        // so the cached collision/shooter path is fixed at 2,500 raw cycles.
        return Some(M68kCycleCount::from_raw(2_500));
    }

    // ASM sub_21F8 loc_2232 installs The Kid's standing hitbox, so every
    // grounded path below enters loc_EAE4 with the same four-row/two-column
    // scan. `loc_1062 -> sub_78E8` installs it for the finished pull-up too.
    let grounded_standing_hitbox = matches!(
        ledger.execute_objects.executed_player_path,
        Some(
            ModeLevelPlayerExecutionPath::GroundedStationaryNoInputLoc7606 {
                form: crate::PlayerForm::TheKid,
            } | ModeLevelPlayerExecutionPath::GroundedRightWalkStartLoc779c {
                form: crate::PlayerForm::TheKid,
                ..
            } | ModeLevelPlayerExecutionPath::GroundedRightWalkContinueLoc8bf0 {
                form: crate::PlayerForm::TheKid,
                ..
            } | ModeLevelPlayerExecutionPath::PullUpAnimationFinished {
                form: crate::PlayerForm::TheKid,
            },
        )
    );
    // ASM sub_DFB0 (kid.asm:17317) and loc_EAE4 (kid.asm:18858):
    // 994 raw cycles surround the inclusive row/column scan. Each row costs
    // 38 plus 40 per cell; a bit-15 non-type-$A cell takes the loc_EB4C
    // `andi/cmpi/beq` rejection and adds 30 cycles before scanning continues.
    let sub_dfb0 = 994_u32
        .checked_add(
            u32::from(scan.row_count)
                .checked_mul(38_u32.checked_add(40_u32.checked_mul(u32::from(scan.col_count))?)?)?,
        )?
        .checked_add(
            FLAGGED_CELL_REJECTION_RAW_CYCLES
                .checked_mul(u32::from(scan.flagged_non_evanescent_cells_before_exit))?,
        )?;

    let shooter_scan = match (shooter.cached_col_matches, shooter.cached_row_matches) {
        // ASM sub_F096 loc_F09E -> loc_F0D8/loc_F106 -> loc_F186 ->
        // loc_F1A8/loc_F1DE: the pull-up moves Kid one block right and one
        // block up, so both entering layout strips are scanned. Frame 957's
        // selected-PC journal leaves 1,054 raw cycles beyond the cached path
        // for its eight-cell column and seven-cell row scans.
        (false, false)
            if shooter.cached_col_before < shooter.kid_block_col
                && shooter.cached_row_before > shooter.kid_block_row
                && shooter.kid_block_col.checked_add(3)?
                    < i16::try_from(shooter.level_width_blocks).ok()?
                && shooter.kid_block_row >= 5
                && shooter.kid_block_row.checked_add(2)?
                    < i16::try_from(shooter.level_height_blocks).ok()?
                && shooter.new_shooter_slots_registered == 0 =>
        {
            1_054
        }
        // ASM sub_F096 loc_F09E -> loc_F186 -> loc_F25C: cached Kid column
        // and row avoid both layout strips.
        (true, true) => 0,
        // ASM sub_F096 loc_F09E -> loc_F0D8 -> loc_F106: when Kid's column
        // moves right, scan the entering column three cells ahead over the
        // eight-row activation span. Frame 939's selected-PC journal measures
        // 824 raw cycles through loc_F25C versus 166 cached: 658 extra.
        (false, true)
            if shooter.cached_col_before < shooter.kid_block_col
                && shooter.kid_block_col.checked_add(3)?
                    < i16::try_from(shooter.level_width_blocks).ok()?
                && shooter.kid_block_row >= 5
                && shooter.kid_block_row.checked_add(2)?
                    < i16::try_from(shooter.level_height_blocks).ok()?
                && shooter.new_shooter_slots_registered == 0 =>
        {
            658
        }
        // ASM sub_F096 loc_F09E -> loc_F0D0 -> loc_F106: moving left scans
        // the entering column three cells behind Kid over the same eight-row
        // activation span. Frame 1141 spans 814 raw cycles through loc_F25C
        // versus 166 cached, for 648 extra.
        (false, true)
            if shooter.cached_col_before > shooter.kid_block_col
                && shooter.kid_block_col >= 3
                && shooter.new_shooter_slots_registered == 0 =>
        {
            648
        }
        // ASM loc_F0D0 clamps a negative entering column to zero. Frame 1151
        // reaches Kid column 2; the failed BPL plus MOVEQ adds two raw cycles
        // over the unclamped left-column path.
        (false, true)
            if shooter.cached_col_before > shooter.kid_block_col
                && shooter.kid_block_col >= 0
                && shooter.kid_block_col < 3
                && shooter.new_shooter_slots_registered == 0 =>
        {
            650
        }
        // ASM sub_F096 loc_F186 -> loc_F1A8 -> loc_F1DE: when Kid's row
        // moves upward, scan the entering row five cells above. The fixed
        // setup adds 150 raw cycles over the cached path; each empty cell
        // adds 54. Selected-PC journals pin six cells / 474 extra at frame
        // 936 and seven cells / 528 extra at frame 940.
        (true, false)
            if shooter.cached_row_before > shooter.kid_block_row
                && shooter.kid_block_row >= 5
                && shooter.kid_block_col >= 0
                && shooter.kid_block_col.checked_add(3)?
                    < i16::try_from(shooter.level_width_blocks).ok()?
                && shooter.new_shooter_slots_registered == 0 =>
        {
            let left_col = shooter.kid_block_col.saturating_sub(3).max(0);
            let cell_count = shooter
                .kid_block_col
                .checked_add(3)?
                .checked_sub(left_col)?
                .checked_add(1)?;
            150_u32.checked_add(54_u32.checked_mul(u32::try_from(cell_count).ok()?)?)?
        }
        // ASM sub_F096 loc_F186..loc_F25C: when only Kid's row advances,
        // scan the entering seven-cell row. With no shooter candidates every
        // loc_F1DE iteration takes the direct loc_F254 branch. The frame-1706
        // selected-PC journal measures 704 raw cycles from sub_F096 to
        // loc_F25C versus 166 on the cached-coordinate path: 538 extra.
        (true, false)
            if shooter.cached_row_before < shooter.kid_block_row
                && shooter.kid_block_row.checked_add(2)?
                    < i16::try_from(shooter.level_height_blocks).ok()?
                && shooter.kid_block_col >= 3
                && shooter.kid_block_col.checked_add(3)?
                    < i16::try_from(shooter.level_width_blocks).ok()?
                && shooter.new_shooter_slots_registered == 0 =>
        {
            538
        }
        // ASM loc_F186 -> loc_F1B0 -> loc_F1DE with the horizontal row span
        // clamped at column zero. Frame 1158 scans columns 0..5: 650 raw
        // cycles versus 166 cached, for 484 extra.
        (true, false)
            if shooter.cached_row_before < shooter.kid_block_row
                && shooter.kid_block_row.checked_add(2)?
                    < i16::try_from(shooter.level_height_blocks).ok()?
                && shooter.kid_block_col >= 0
                && shooter.kid_block_col < 3
                && shooter.kid_block_col.checked_add(3)?
                    < i16::try_from(shooter.level_width_blocks).ok()?
                && shooter.new_shooter_slots_registered == 0 =>
        {
            let cell_count = shooter.kid_block_col.checked_add(4)?;
            160_u32.checked_add(54_u32.checked_mul(u32::try_from(cell_count).ok()?)?)?
        }
        // ASM sub_F096 loc_F0D0/F106 and loc_F1A8/F1DE: the power-on -1
        // caches make frame 902 scan both initial layout strips.
        (false, false)
            if shooter.cached_col_before == -1
                && shooter.cached_row_before == -1
                && shooter.kid_block_col == 2
                && shooter.kid_block_row >= 5
                && u16::try_from(shooter.kid_block_row).ok()?.checked_add(2)?
                    < shooter.level_height_blocks
                && shooter.new_shooter_slots_registered == 0 =>
        {
            550
        }
        _ => return None,
    };

    let hidden_block_y_velocity = match ledger.execute_objects.executed_player_path.as_ref()? {
        ModeLevelPlayerExecutionPath::WallClingNoButtonsLocAa22 {
            form: crate::PlayerForm::TheKid,
            y_velocity_after,
            ..
        } => Some(*y_velocity_after),
        ModeLevelPlayerExecutionPath::NormalPhysics(path)
            if path.y_velocity_after < 0
                && matches!(
                    path.vertical,
                    ModeLevelAirborneVerticalPath::HeldJumpGravityLocB1e8
                        | ModeLevelAirborneVerticalPath::DefaultGravityLocB1e0
                ) =>
        {
            Some(path.y_velocity_after)
        }
        _ => None,
    };
    let hidden_block_scan = if let Some(y_velocity_after) = hidden_block_y_velocity {
        let entry = ledger.execute_objects.player_entry_position?;
        let old_y_px = (entry.y_pos >> 16) as i32;
        let new_y_px = (entry.y_pos.wrapping_add(y_velocity_after as u32) >> 16) as i32;
        let old_scan_row = (old_y_px - 32 - 1) >> 4;
        let new_scan_row = (new_y_px - 32 - 1) >> 4;
        if new_scan_row < old_scan_row {
            let x_px = (entry.x_pos >> 16) as i32;
            let start_col = (x_px - 4).max(0) >> 4;
            let end_col = (x_px + 4 - 1) >> 4;
            let cell_count = end_col.checked_sub(start_col)?.checked_add(1)?;
            // ASM sub_DFB0 loc_DFEE -> loc_E03A -> loc_E23A: frame
            // 951 crosses one upward row and visits one non-hidden cell.
            // Selected PCs measure 684 raw cycles to loc_EAE4 versus
            // the direct path's 562, so the scan adds 122. Each further
            // non-hidden cell repeats the 58-cycle loc_E03A/loc_E23A loop.
            122_u32.checked_add(58_u32.checked_mul(u32::try_from(cell_count - 1).ok()?)?)?
        } else {
            0
        }
    } else {
        0
    };

    // ASM Mode_Level $7E6..loc_80C: grounded movement branches directly from
    // sub_DFB0's Character_Movement check to loc_EAE4. Jumping movement runs
    // the 244-cycle hitbox/row-crossing setup first. The jumping shell also
    // owns Mode_Level's separate 18-cycle BSR.W sub_21F8 boundary; it must not
    // be hidden in the per-flagged-cell price.
    let jump_movement = matches!(
        ledger.execute_objects.executed_player_path,
        Some(
            ModeLevelPlayerExecutionPath::GroundedJumpLaunchLocA426 { .. }
                | ModeLevelPlayerExecutionPath::NormalPhysics(_)
                | ModeLevelPlayerExecutionPath::WallClingNoButtonsLocAa22 { .. }
                | ModeLevelPlayerExecutionPath::WallClingPullUpLocAbf0 { .. }
                | ModeLevelPlayerExecutionPath::PullUpAnimationHibernateLoc1062 { .. }
        )
    );
    let source_shell: u32 = if grounded_standing_hitbox {
        996
    } else if jump_movement {
        1_258
    } else {
        1_240
    };
    Some(M68kCycleCount(
        source_shell
            .checked_add(sub_dfb0)?
            .checked_add(shooter_scan)?
            .checked_add(hidden_block_scan)?,
    ))
}

/// Source-positioned non-refresh waits in the post-Execute path.
pub fn post_execute_external_waits_before_sub_44dc(
    ledger: &ModeLevelPreHudWorkLedger,
) -> Option<Vec<M68kExternalWaitEvent>> {
    let raw = post_execute_raw_cycles_before_sub_44dc(ledger)?;
    if is_first_level_start_execute_objects_sub_e12(&ledger.execute_objects) {
        return Some(vec![M68kExternalWaitEvent {
            raw_offset: raw,
            cycles: M68kCycleCount::from_raw(1),
        }]);
    }
    Some(Vec::new())
}

/// Advance the typed post-Execute path through the Genesis refresh recurrence
/// and any source-positioned external wait.
pub fn post_execute_elapsed_cycles_before_sub_44dc(
    ledger: &ModeLevelPreHudWorkLedger,
    phase_before: M68kRefreshPhase,
) -> Option<M68kRefreshAdvance> {
    let raw = phase_before.advance_rom(post_execute_raw_cycles_before_sub_44dc(ledger)?);
    let mut elapsed = raw.elapsed.get();
    let mut phase = raw.phase_after;
    for event in post_execute_external_waits_before_sub_44dc(ledger)? {
        let wait = phase.advance_external_wait(event.cycles);
        elapsed = elapsed.checked_add(wait.elapsed.get())?;
        phase = wait.phase_after;
    }
    Some(M68kRefreshAdvance {
        elapsed: M68kCycleCount::from_raw(elapsed),
        phase_after: phase,
        refresh_events: raw.refresh_events,
    })
}

/// Raw CPU work for `sub_44DC` when every HUD animation counter skips.
pub fn hud_no_upload_raw_cycles_sub_44dc(
    commands: &[HudDmaEventSub44DC],
) -> Option<M68kCycleCount> {
    if !commands.is_empty() {
        return None;
    }

    // ASM sub_44DC (kid.asm:5546-5752): five 42-cycle counter-test/store
    // sections, followed by the outer 16-cycle RTS.
    Some(M68kCycleCount::from_raw(226))
}

/// Raw CPU work for the coin-only `sub_44DC` path.
///
/// FIFO and refresh stalls are deliberately excluded. An offset of zero is
/// the wrapped `AniArt_Coin` selector path through the `$FF` terminator.
pub fn hud_coin_only_raw_cycles_sub_44dc(
    commands: &[HudDmaEventSub44DC],
) -> Option<M68kCycleCount> {
    let [
        HudDmaEventSub44DC::Coin {
            frame_table_byte_offset,
        },
    ] = commands
    else {
        return None;
    };

    // ASM sub_44DC loc_44F4 -> loc_4534 -> sub_44D0 (kid.asm): one
    // 128-byte CPU-to-VRAM upload costs 1,350 raw cycles. The `$FF` selector
    // retry at loc_44F4 adds 58 raw cycles before selecting frame zero.
    Some(M68kCycleCount::from_raw(
        1_350 + u32::from(*frame_table_byte_offset == 0) * 58,
    ))
}

/// Raw CPU work for the classified life-icon-only `sub_44DC` path.
///
/// FIFO and refresh stalls are deliberately excluded. Other HUD command
/// combinations remain unclassified until their distinct ROM branches and
/// DMA ownership are typed.
pub fn hud_life_icon_only_raw_cycles_sub_44dc(
    commands: &[HudDmaEventSub44DC],
) -> Option<M68kCycleCount> {
    let [
        HudDmaEventSub44DC::LifeIcon {
            wrapped_to_start, ..
        },
    ] = commands
    else {
        return None;
    };

    // ASM sub_44DC (kid.asm): 236 raw cycles reach completion of the first
    // loc_44D4 MOVE.L. The remaining 31 MOVE.L/taken-DBF pairs cost 930,
    // followed by final DBF 14, sub_44D0 RTS 16, three 42-cycle skipped HUD
    // sections, final store 12, and outer RTS 16.
    // ASM loc_455E (kid.asm:5612): the `$FF` terminator path executes the
    // failed BPL.W, MOVEQ, BRA.S, then repeats the selector loop. Its exact
    // instruction sequence costs 58 raw cycles.
    Some(M68kCycleCount::from_raw(
        1_350 + u32::from(*wrapped_to_start) * 58,
    ))
}

/// Raw CPU work for the clock-icon-only `sub_44DC` path.
pub fn hud_clock_icon_only_raw_cycles_sub_44dc(
    commands: &[HudDmaEventSub44DC],
) -> Option<M68kCycleCount> {
    let [
        HudDmaEventSub44DC::Clock {
            wrapped_to_start, ..
        },
    ] = commands
    else {
        return None;
    };

    // ASM sub_44DC -> loc_45B2 -> loc_45DA -> sub_44D0 (kid.asm): the
    // non-wrapping single 128-byte upload has the same 1,350-cycle total as
    // the life-icon path. A `$FF` selector retry at loc_45B2 adds 58 cycles.
    Some(M68kCycleCount::from_raw(
        1_350 + u32::from(*wrapped_to_start) * 58,
    ))
}

/// Raw CPU work for the diamond-only `sub_44DC` path.
///
/// The `$200`-byte VRAM DMA and Z80 BUSREQ waits are hardware ownership, not
/// source cycles. Other HUD command combinations stay unclassified.
pub fn hud_diamond_only_raw_cycles_sub_44dc(
    commands: &[HudDmaEventSub44DC],
) -> Option<M68kCycleCount> {
    if !matches!(commands, [HudDmaEventSub44DC::Diamond { .. }]) {
        return None;
    }

    // ASM sub_44DC loc_45EE -> loc_4626 -> sub_5D4A: the three earlier
    // animations skip, the diamond timer advances, sub_5D4A programs its
    // 256-word VRAM DMA, and the flag animation skips.
    Some(M68kCycleCount::from_raw(760))
}

/// Visit every source instruction and hardware edge in the diamond-only
/// `sub_44DC` path.
///
/// ASM `loc_4626 -> sub_5D4A` brackets the DMA with Stop_z80/Start_z80. The
/// DMA begins after the second `$FFFFF800` command word reaches the VDP.
pub fn visit_hud_diamond_only_sub_44dc(
    commands: &[HudDmaEventSub44DC],
    mut visit: impl FnMut(crate::intro_timing::MemoryDmaTimingEvent),
) -> Option<crate::intro_timing::IntroSourceTiming> {
    hud_diamond_only_raw_cycles_sub_44dc(commands)?;
    use crate::intro_timing::{IntroSourceTiming, MemoryDmaTimingEvent, MemoryToVdpDmaTarget};

    let mut timing = IntroSourceTiming {
        instruction_count: 0,
        raw_cycles: M68kCycleCount::from_raw(0),
    };
    let emit = |visit: &mut dyn FnMut(MemoryDmaTimingEvent),
                timing: &mut IntroSourceTiming,
                cycles: u32| {
        visit(MemoryDmaTimingEvent::Instruction(cycles));
        timing.instruction_count += 1;
        timing.raw_cycles = M68kCycleCount::from_raw(
            timing
                .raw_cycles
                .get()
                .checked_add(cycles)
                .expect("diamond HUD source timing fits u32"),
        );
    };

    // ASM sub_44DC $44DC..loc_45EE: coin, life and clock timers all skip.
    for cycles in [12, 4, 4, 10, 12, 12, 4, 4, 10, 12, 12, 4, 4, 10] {
        emit(&mut visit, &mut timing, cycles);
    }
    // ASM loc_45EE..loc_462C: the diamond timer reaches frame offset four,
    // selects that page, and calls sub_5D4A.
    for cycles in [12, 12, 4, 4, 12, 12, 4, 4, 12, 12, 8, 10, 8, 4, 4, 18] {
        emit(&mut visit, &mut timing, cycles);
    }
    // ASM sub_5D4A through Stop_z80's BUSREQ write and acknowledgement.
    for cycles in [16, 12, 8, 20, 10, 10, 20] {
        emit(&mut visit, &mut timing, cycles);
    }
    visit(MemoryDmaTimingEvent::SetZ80BusRequest(true));
    for cycles in [20, 8, 16] {
        emit(&mut visit, &mut timing, cycles);
    }
    // ASM sub_5D4A loc_5D5A..loc_5D98: program length, source and the two
    // destination command words. The final write starts a 256-word VRAM DMA.
    for cycles in [24, 4, 10, 4, 8, 8, 12, 4, 22, 8, 12, 4, 8, 12, 24, 20, 20] {
        emit(&mut visit, &mut timing, cycles);
    }
    visit(MemoryDmaTimingEvent::BeginMemoryDma {
        words: 0x100,
        target: MemoryToVdpDmaTarget::Vram,
    });
    // ASM sub_5D4A $5D9E -> Start_z80, then return to sub_44DC. The flag
    // timer skips and the outer routine returns.
    for cycles in [20, 10, 10, 20] {
        emit(&mut visit, &mut timing, cycles);
    }
    visit(MemoryDmaTimingEvent::SetZ80BusRequest(false));
    for cycles in [16, 16, 12, 12, 4, 4, 10, 12, 16] {
        emit(&mut visit, &mut timing, cycles);
    }

    debug_assert_eq!(
        timing.raw_cycles,
        hud_diamond_only_raw_cycles_sub_44dc(commands)?
    );
    Some(timing)
}

/// Raw CPU work for the flag-only `sub_44DC` path.
///
/// The `$280`-byte VRAM DMA and Z80 BUSREQ waits remain hardware ownership.
pub fn hud_flag_only_raw_cycles_sub_44dc(
    commands: &[HudDmaEventSub44DC],
) -> Option<M68kCycleCount> {
    let [
        HudDmaEventSub44DC::Flag {
            frame_table_byte_offset,
        },
    ] = commands
    else {
        return None;
    };

    // ASM sub_44DC loc_4630 -> loc_4680 -> sub_5DA6: four earlier animation
    // counters skip, the flag selects one art page, sub_5DA6 programs its
    // 320-word VRAM DMA, and loc_468C returns. Page zero follows the `$FF`
    // sentinel retry at loc_4648, which adds 58 raw cycles.
    Some(M68kCycleCount::from_raw(
        742 + u32::from(*frame_table_byte_offset == 0) * 58,
    ))
}

/// Visit every source instruction and hardware edge in the flag-only
/// `sub_44DC -> loc_4680 -> sub_5DA6` path.
pub fn visit_hud_flag_only_sub_44dc(
    commands: &[HudDmaEventSub44DC],
    mut visit: impl FnMut(crate::intro_timing::MemoryDmaTimingEvent),
) -> Option<crate::intro_timing::IntroSourceTiming> {
    hud_flag_only_raw_cycles_sub_44dc(commands)?;
    let [
        HudDmaEventSub44DC::Flag {
            frame_table_byte_offset,
        },
    ] = commands
    else {
        return None;
    };
    use crate::intro_timing::{IntroSourceTiming, MemoryDmaTimingEvent, MemoryToVdpDmaTarget};

    let mut timing = IntroSourceTiming {
        instruction_count: 0,
        raw_cycles: M68kCycleCount::from_raw(0),
    };
    let emit = |visit: &mut dyn FnMut(MemoryDmaTimingEvent),
                timing: &mut IntroSourceTiming,
                cycles: u32| {
        visit(MemoryDmaTimingEvent::Instruction(cycles));
        timing.instruction_count += 1;
        timing.raw_cycles = M68kCycleCount::from_raw(
            timing
                .raw_cycles
                .get()
                .checked_add(cycles)
                .expect("flag HUD source timing fits u32"),
        );
    };

    // ASM sub_44DC $44DC..loc_4630: coin, life, clock, and diamond skip.
    for cycles in [
        12, 4, 4, 10, 12, 12, 4, 4, 10, 12, 12, 4, 4, 10, 12, 12, 4, 4, 10,
    ] {
        emit(&mut visit, &mut timing, cycles);
    }
    // ASM loc_4630..loc_4648: advance the flag selector.
    for cycles in [12, 12, 4, 4, 12, 12, 4, 4] {
        emit(&mut visit, &mut timing, cycles);
    }
    if *frame_table_byte_offset == 0 {
        // ASM loc_4648 repeats after the AniArt_Flag `$FF` sentinel.
        for cycles in [12, 12, 8, 12, 4, 10] {
            emit(&mut visit, &mut timing, cycles);
        }
    }
    // ASM loc_4648..loc_4680: select the page and call sub_5DA6.
    for cycles in [12, 12, 8, 10, 8, 4, 18, 18] {
        emit(&mut visit, &mut timing, cycles);
    }
    // ASM sub_5DA6 through Stop_z80's BUSREQ write and acknowledgement.
    for cycles in [4, 20, 10, 10, 20] {
        emit(&mut visit, &mut timing, cycles);
    }
    visit(MemoryDmaTimingEvent::SetZ80BusRequest(true));
    for cycles in [20, 8, 16] {
        emit(&mut visit, &mut timing, cycles);
    }
    // ASM sub_5DA6 programs length, source and the two destination command
    // words. The final write begins the 320-word VRAM DMA.
    for cycles in [24, 4, 10, 4, 8, 8, 12, 4, 22, 8, 12, 4, 8, 12, 24, 20, 20] {
        emit(&mut visit, &mut timing, cycles);
    }
    visit(MemoryDmaTimingEvent::BeginMemoryDma {
        words: 0x140,
        target: MemoryToVdpDmaTarget::Vram,
    });
    // ASM sub_5DA6 -> Start_z80, then loc_468C stores the timer and returns.
    for cycles in [20, 10, 10, 20] {
        emit(&mut visit, &mut timing, cycles);
    }
    visit(MemoryDmaTimingEvent::SetZ80BusRequest(false));
    for cycles in [16, 16, 12, 16] {
        emit(&mut visit, &mut timing, cycles);
    }

    debug_assert_eq!(
        timing.raw_cycles,
        hud_flag_only_raw_cycles_sub_44dc(commands)?
    );
    Some(timing)
}

/// Hardware-visible control boundary in ASM `ReadJoypad`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReadJoypadControl {
    /// ASM `sub_914 -> Stop_z80`: the `$A11100` BUSREQ write completed.
    BeginZ80Bus,
    /// ASM `sub_924 -> Start_z80`: the `$A11100` release write completed.
    ReleaseZ80Bus,
}

fn visit_read_joypad_instructions(
    work: ModeLevelReadJoypadWork,
    mut visit: impl FnMut(u32) -> Option<()>,
    mut apply_control: impl FnMut(ReadJoypadControl),
) -> Option<()> {
    let ModeLevelReadJoypadWork::DemoPlaybackLocC22(Some(demo)) = work else {
        return None;
    };
    let advances_keypress_pointer = demo.counter_before_increment & 1 != 0;
    if advances_keypress_pointer != demo.advances_keypress_pointer {
        return None;
    }

    let mut visit_all = |instructions: &[u8]| {
        for &cycles in instructions {
            visit(u32::from(cycles))?;
        }
        Some(())
    };

    // ASM Mode_Level loc_81A, j_ReadJoypad, and ReadJoypad loc_B6C..loc_BB6.
    visit_all(&[18, 10, 12, 10, 20, 20, 12, 10, 18])?;

    // ASM j_sub_914, sub_914, j_Stop_z80, BranchTo_Stop_z80, and the
    // Stop_z80 BUSREQ write.
    visit_all(&[10, 16, 16, 20, 10, 10, 20])?;
    apply_control(ReadJoypadControl::BeginZ80Bus);
    // ASM Stop_z80 one-poll BUSACK path and the two returning RTS instructions.
    visit_all(&[20, 8, 16, 16])?;

    const READ_HARDWARE_SUB_CAA: &[u8] = &[12, 4, 4, 8, 10, 8, 12, 4, 4, 8, 8, 4, 4, 16];
    const PERMUTE_DEMO_SUB_CCE: &[u8] = &[12, 10, 16];

    // ASM loc_BBA..loc_BC4: controller 1 hardware read and Demo-mode
    // Permute_ABCButtons early return.
    visit_all(&[12, 18])?;
    visit_all(READ_HARDWARE_SUB_CAA)?;
    visit_all(&[18])?;
    visit_all(PERMUTE_DEMO_SUB_CCE)?;

    // ASM loc_BC8..loc_BD8: controller 2 hardware read and the same
    // Permute_ABCButtons path.
    visit_all(&[12, 4, 12, 18])?;
    visit_all(READ_HARDWARE_SUB_CAA)?;
    visit_all(&[18])?;
    visit_all(PERMUTE_DEMO_SUB_CCE)?;

    // ASM loc_BDC..sub_924, through the Start_z80 release write.
    visit_all(&[12, 18, 10, 16, 8, 16, 16, 20, 10, 10, 20])?;
    apply_control(ReadJoypadControl::ReleaseZ80Bus);
    visit_all(&[16, 16])?;

    // ASM loc_BE4..loc_C40: Demo counter and keypress stream.
    visit_all(&[12, 12, 12, 8, 10, 4, 12, 16, 8, 8, 12, 4, 12, 8])?;
    if advances_keypress_pointer {
        // ASM loc_C40 not-taken BNE.w, loc_C44 pointer update, and loc_C4A
        // branch to the shared edge detector.
        visit_all(&[12, 8, 16, 10])?;
    } else {
        // ASM loc_C40 taken BNE.w directly to loc_C4E.
        visit_all(&[10])?;
    }

    // ASM loc_C4E..loc_C9C and the return to Mode_Level loc_81E.
    visit_all(&[
        12, 12, 4, 12, 16, 20, 12, 12, 4, 12, 16, 20, 12, 10, 20, 20, 16,
    ])
}

/// Raw CPU work for ASM `ReadJoypad` inside `Mode_Level`.
///
/// Refresh and asynchronous GEMS bank waits are excluded. Controller and
/// incomplete Demo observations remain fail-closed.
pub fn read_joypad_raw_cycles(work: ModeLevelReadJoypadWork) -> Option<M68kCycleCount> {
    let mut raw_cycles = 0_u32;
    visit_read_joypad_instructions(
        work,
        |instruction_cycles| {
            raw_cycles = raw_cycles.checked_add(instruction_cycles)?;
            Some(())
        },
        |_| {},
    )?;
    // The visitor includes Mode_Level loc_81A's 18-cycle JSR and the
    // 10-cycle j_ReadJoypad shell.
    raw_cycles.checked_sub(28).map(M68kCycleCount::from_raw)
}

/// Co-schedule ASM `Mode_Level loc_81A -> ReadJoypad -> loc_81E` with
/// asynchronous hardware.
///
/// `advance_hardware` receives whether the source currently owns the Z80 bus.
/// `apply_control` changes that ownership only after the defining `$A11100`
/// write and all waits generated before it have completed.
pub fn read_joypad_advance_with_scheduler(
    mut phase: M68kRefreshPhase,
    work: ModeLevelReadJoypadWork,
    mut advance_hardware: impl FnMut(M68kCycleCount, bool) -> Option<M68kCycleCount>,
    mut apply_control: impl FnMut(ReadJoypadControl),
) -> Option<M68kRefreshAdvance> {
    let mut elapsed = 0_u32;
    let mut refresh_events = 0_u32;
    let bus_owned = std::cell::Cell::new(false);

    visit_read_joypad_instructions(
        work,
        |instruction_cycles| {
            let phase_after_raw = u32::from(phase.get()).checked_add(instruction_cycles)?;
            let refresh_wait = u32::from(phase_after_raw >= 128) * 2;
            phase = M68kRefreshPhase::new(
                u8::try_from(phase_after_raw % 128).expect("refresh phase remains below 128"),
            )
            .expect("refresh phase remains below 128");
            elapsed = elapsed.checked_add(instruction_cycles)?;

            let generated_wait = advance_hardware(
                M68kCycleCount::from_raw(instruction_cycles),
                bus_owned.get(),
            )?;
            let mut pending_wait = generated_wait.get().checked_add(refresh_wait)?;
            refresh_events = refresh_events.checked_add(refresh_wait / 2)?;
            while pending_wait != 0 {
                let wait = M68kCycleCount::from_raw(pending_wait);
                phase = phase.advance_external_wait(wait).phase_after;
                elapsed = elapsed.checked_add(pending_wait)?;
                pending_wait = advance_hardware(wait, bus_owned.get())?.get();
            }
            Some(())
        },
        |control| {
            bus_owned.set(matches!(control, ReadJoypadControl::BeginZ80Bus));
            apply_control(control);
        },
    )?;

    Some(M68kRefreshAdvance {
        elapsed: M68kCycleCount::from_raw(elapsed),
        phase_after: phase,
        refresh_events,
    })
}

/// Raw routine-body cost for ASM `Manage_EnemyLoading` (`kid.asm`).
///
/// The `Mode_Level loc_80C` caller and `j_Manage_EnemyLoading` jump-table
/// shell are excluded. Layout walks remain fail-closed until their cursor and
/// enemy-status branches are typed.
pub fn manage_enemy_loading_raw_cycles(path: ModeLevelEnemyLoadingPath) -> Option<M68kCycleCount> {
    match path {
        // ASM Manage_EnemyLoading loc_36814 -> return_3682A: Demo mode
        // passes the FB6A and Game_Mode tests, then rejects non-Knights Isle.
        ModeLevelEnemyLoadingPath::DemoNonKnightsReturnLoc3682a => {
            Some(M68kCycleCount::from_raw(84))
        }
        ModeLevelEnemyLoadingPath::LayoutWalkLoc3682c => None,
    }
}

/// Co-schedule the ASM `Mode_Level loc_810 -> Manage_EnemyLoading
/// return_3682A` phase with asynchronous hardware.
///
/// Each source instruction is retained because a GEMS bank read can queue an
/// 11-cycle 68000 wait after any completed instruction. Pending refresh and
/// hardware waits are drained before the next instruction.
pub fn manage_enemy_loading_advance_with_scheduler_loc_36814(
    mut phase: M68kRefreshPhase,
    path: ModeLevelEnemyLoadingPath,
    mut advance_hardware: impl FnMut(M68kCycleCount) -> Option<M68kCycleCount>,
) -> Option<M68kRefreshAdvance> {
    let mut elapsed = 0_u32;
    let mut refresh_events = 0_u32;

    visit_manage_enemy_loading_instructions_loc_36814(path, |instruction_cycles| {
        let phase_after_raw = u32::from(phase.get()).checked_add(instruction_cycles)?;
        let refresh_wait = u32::from(phase_after_raw >= 128) * 2;
        phase = M68kRefreshPhase::new(
            u8::try_from(phase_after_raw % 128).expect("refresh phase remains below 128"),
        )
        .expect("refresh phase remains below 128");
        elapsed = elapsed.checked_add(instruction_cycles)?;

        let generated_wait = advance_hardware(M68kCycleCount::from_raw(instruction_cycles))?;
        let mut pending_wait = generated_wait.get().checked_add(refresh_wait)?;
        refresh_events = refresh_events.checked_add(refresh_wait / 2)?;
        while pending_wait != 0 {
            let wait = M68kCycleCount::from_raw(pending_wait);
            phase = phase.advance_external_wait(wait).phase_after;
            elapsed = elapsed.checked_add(pending_wait)?;
            pending_wait = advance_hardware(wait)?.get();
        }
        Some(())
    })?;

    Some(M68kRefreshAdvance {
        elapsed: M68kCycleCount::from_raw(elapsed),
        phase_after: phase,
        refresh_events,
    })
}

fn visit_manage_enemy_loading_instructions_loc_36814(
    path: ModeLevelEnemyLoadingPath,
    mut visit: impl FnMut(u32) -> Option<()>,
) -> Option<()> {
    let ModeLevelEnemyLoadingPath::DemoNonKnightsReturnLoc3682a = path else {
        return None;
    };

    // ASM Mode_Level loc_810 JSR, j_Manage_EnemyLoading JMP, then
    // Manage_EnemyLoading loc_36814 -> return_3682A. Demo mode passes the
    // `$FB6A` and Game_Mode branches, then rejects non-Knights Isle.
    for cycles in [20, 10, 12, 8, 16, 8, 16, 8, 16] {
        visit(cycles)?;
    }
    Some(())
}

fn visit_execute_scripted_platforms_instructions_sub_23ea(
    work: ModeLevelPlatformWork,
    mut visit: impl FnMut(u32) -> Option<()>,
) -> Option<()> {
    if work.frozen_fa64 {
        // ASM Mode_Level loc_826 BSR and sub_23EA: `$FA64` takes the first
        // branch to return_2442.
        for cycles in [18, 12, 10, 16] {
            visit(cycles)?;
        }
        return Some(());
    }
    if work.active_object_count != 0 {
        return None;
    }

    // ASM Mode_Level loc_826 BSR, then loc_23F2 ->
    // Execute_ScriptedPlatforms_Loop: the sentinel's null link branches
    // directly to return_2442.
    for cycles in [18, 12, 12, 8, 12, 10, 16] {
        visit(cycles)?;
    }
    Some(())
}

/// Raw routine-body cost for ASM `Execute_ScriptedPlatforms` (`sub_23EA`).
pub fn execute_scripted_platforms_raw_cycles_sub_23ea(
    work: ModeLevelPlatformWork,
) -> Option<M68kCycleCount> {
    let mut raw_cycles = 0_u32;
    visit_execute_scripted_platforms_instructions_sub_23ea(work, |instruction_cycles| {
        raw_cycles = raw_cycles.checked_add(instruction_cycles)?;
        Some(())
    })?;
    raw_cycles.checked_sub(18).map(M68kCycleCount::from_raw)
}

/// Co-schedule ASM `Mode_Level loc_826 -> sub_23EA -> loc_82A` with
/// asynchronous hardware.
pub fn execute_scripted_platforms_advance_with_scheduler_sub_23ea(
    mut phase: M68kRefreshPhase,
    work: ModeLevelPlatformWork,
    mut advance_hardware: impl FnMut(M68kCycleCount) -> Option<M68kCycleCount>,
) -> Option<M68kRefreshAdvance> {
    let mut elapsed = 0_u32;
    let mut refresh_events = 0_u32;

    visit_execute_scripted_platforms_instructions_sub_23ea(work, |instruction_cycles| {
        let phase_after_raw = u32::from(phase.get()).checked_add(instruction_cycles)?;
        let refresh_wait = u32::from(phase_after_raw >= 128) * 2;
        phase = M68kRefreshPhase::new(
            u8::try_from(phase_after_raw % 128).expect("refresh phase remains below 128"),
        )
        .expect("refresh phase remains below 128");
        elapsed = elapsed.checked_add(instruction_cycles)?;

        let generated_wait = advance_hardware(M68kCycleCount::from_raw(instruction_cycles))?;
        let mut pending_wait = generated_wait.get().checked_add(refresh_wait)?;
        refresh_events = refresh_events.checked_add(refresh_wait / 2)?;
        while pending_wait != 0 {
            let wait = M68kCycleCount::from_raw(pending_wait);
            phase = phase.advance_external_wait(wait).phase_after;
            elapsed = elapsed.checked_add(pending_wait)?;
            pending_wait = advance_hardware(wait)?.get();
        }
        Some(())
    })?;

    Some(M68kRefreshAdvance {
        elapsed: M68kCycleCount::from_raw(elapsed),
        phase_after: phase,
        refresh_events,
    })
}

fn visit_platforms_check_collision_instructions_sub_2574(
    work: ModeLevelPlatformWork,
    mut visit: impl FnMut(u32) -> Option<()>,
) -> Option<()> {
    if work.frozen_fa64 {
        // ASM Mode_Level loc_82A BSR and sub_2574: non-zero `$FA64` skips
        // the saved-register frame.
        for cycles in [18, 12, 12, 16] {
            visit(cycles)?;
        }
        return Some(());
    }
    if work.active_object_count != 0 {
        return None;
    }

    // ASM Mode_Level loc_82A BSR, then loc_257E -> loc_25A0 -> loc_2720:
    // save the collision register set, initialise scratch/hitbox state,
    // observe the null sentinel, restore, and return.
    for cycles in [
        18, 12, 10, 88, 8, 24, 8, 16, 12, 12, 12, 12, 12, 10, 8, 92, 16,
    ] {
        visit(cycles)?;
    }
    Some(())
}

/// Raw routine-body cost for ASM `Platforms_CheckCollision` (`sub_2574`).
pub fn platforms_check_collision_raw_cycles_sub_2574(
    work: ModeLevelPlatformWork,
) -> Option<M68kCycleCount> {
    let mut raw_cycles = 0_u32;
    visit_platforms_check_collision_instructions_sub_2574(work, |instruction_cycles| {
        raw_cycles = raw_cycles.checked_add(instruction_cycles)?;
        Some(())
    })?;
    raw_cycles.checked_sub(18).map(M68kCycleCount::from_raw)
}

/// Co-schedule ASM `Mode_Level loc_82A -> sub_2574 -> loc_82E` with
/// asynchronous hardware.
pub fn platforms_check_collision_advance_with_scheduler_sub_2574(
    mut phase: M68kRefreshPhase,
    work: ModeLevelPlatformWork,
    mut advance_hardware: impl FnMut(M68kCycleCount) -> Option<M68kCycleCount>,
) -> Option<M68kRefreshAdvance> {
    let mut elapsed = 0_u32;
    let mut refresh_events = 0_u32;

    visit_platforms_check_collision_instructions_sub_2574(work, |instruction_cycles| {
        let phase_after_raw = u32::from(phase.get()).checked_add(instruction_cycles)?;
        let refresh_wait = u32::from(phase_after_raw >= 128) * 2;
        phase = M68kRefreshPhase::new(
            u8::try_from(phase_after_raw % 128).expect("refresh phase remains below 128"),
        )
        .expect("refresh phase remains below 128");
        elapsed = elapsed.checked_add(instruction_cycles)?;

        let generated_wait = advance_hardware(M68kCycleCount::from_raw(instruction_cycles))?;
        let mut pending_wait = generated_wait.get().checked_add(refresh_wait)?;
        refresh_events = refresh_events.checked_add(refresh_wait / 2)?;
        while pending_wait != 0 {
            let wait = M68kCycleCount::from_raw(pending_wait);
            phase = phase.advance_external_wait(wait).phase_after;
            elapsed = elapsed.checked_add(pending_wait)?;
            pending_wait = advance_hardware(wait)?.get();
        }
        Some(())
    })?;

    Some(M68kRefreshAdvance {
        elapsed: M68kCycleCount::from_raw(elapsed),
        phase_after: phase,
        refresh_events,
    })
}

fn visit_make_sprites_from_platforms_instructions_sub_2444(
    work: ModeLevelPlatformWork,
    mut visit: impl FnMut(u32) -> Option<()>,
) -> Option<()> {
    if work.active_object_count != 0 {
        return None;
    }

    // ASM Mode_Level loc_88E BSR, then sub_2444 loc_244C -> loc_256E:
    // the null sentinel stores the unchanged next sprite pointer and returns.
    // `$FA64` does not gate this routine.
    for cycles in [18, 8, 16, 12, 10, 16, 16] {
        visit(cycles)?;
    }
    Some(())
}

/// Raw routine-body cost for ASM `Make_SpritesFromPlatforms` (`sub_2444`).
pub fn make_sprites_from_platforms_raw_cycles_sub_2444(
    work: ModeLevelPlatformWork,
) -> Option<M68kCycleCount> {
    let mut raw_cycles = 0_u32;
    visit_make_sprites_from_platforms_instructions_sub_2444(work, |instruction_cycles| {
        raw_cycles = raw_cycles.checked_add(instruction_cycles)?;
        Some(())
    })?;
    raw_cycles.checked_sub(18).map(M68kCycleCount::from_raw)
}

/// Co-schedule ASM `Mode_Level loc_88E -> Make_SpritesFromPlatforms
/// sub_2444 -> loc_892` with asynchronous hardware.
pub fn make_sprites_from_platforms_advance_with_scheduler_sub_2444(
    mut phase: M68kRefreshPhase,
    work: ModeLevelPlatformWork,
    mut advance_hardware: impl FnMut(M68kCycleCount) -> Option<M68kCycleCount>,
) -> Option<M68kRefreshAdvance> {
    let mut elapsed = 0_u32;
    let mut refresh_events = 0_u32;

    visit_make_sprites_from_platforms_instructions_sub_2444(work, |instruction_cycles| {
        let phase_after_raw = u32::from(phase.get()).checked_add(instruction_cycles)?;
        let refresh_wait = u32::from(phase_after_raw >= 128) * 2;
        phase = M68kRefreshPhase::new(
            u8::try_from(phase_after_raw % 128).expect("refresh phase remains below 128"),
        )
        .expect("refresh phase remains below 128");
        elapsed = elapsed.checked_add(instruction_cycles)?;
        let generated_wait = advance_hardware(M68kCycleCount::from_raw(instruction_cycles))?;
        let mut pending_wait = generated_wait.get().checked_add(refresh_wait)?;
        refresh_events = refresh_events.checked_add(refresh_wait / 2)?;
        while pending_wait != 0 {
            let wait = M68kCycleCount::from_raw(pending_wait);
            phase = phase.advance_external_wait(wait).phase_after;
            elapsed = elapsed.checked_add(pending_wait)?;
            pending_wait = advance_hardware(wait)?.get();
        }
        Some(())
    })?;

    Some(M68kRefreshAdvance {
        elapsed: M68kCycleCount::from_raw(elapsed),
        phase_after: phase,
        refresh_events,
    })
}

fn visit_manage_platform_loading_instructions_loc_2744(
    work: ModeLevelPlatformLoaderWorkLoc2744,
    mut visit: impl FnMut(u32) -> Option<()>,
) -> Option<()> {
    match work {
        // ASM loc_2744 first consumes the `$FFFF` terminator and retries from
        // offset zero. The first record then fails the right-edge test at
        // loc_2788; sub_2728 observes a null active-list sentinel.
        ModeLevelPlatformLoaderWorkLoc2744::Placement {
            wrapped_to_start: true,
            range: ModeLevelPlatformLoaderRangeLoc2762::OutOfRangeRight,
            existing_slot: ModeLevelPlatformLoaderExistingSlotSub2728::Missing,
        } => {
            // ASM Mode_Level loc_822 BSR followed by the exact selected
            // loc_2744 -> loc_2762 -> loc_2850 -> sub_2728 -> loc_286E path.
            for cycles in [
                18, 16, 12, 4, 12, 12, 8, 12, 16, 10, 16, 12, 4, 12, 12, 8, 10, 16, 12, 4, 12, 4,
                12, 16, 4, 12, 4, 12, 16, 4, 10, 18, 8, 12, 10, 4, 16, 10, 20, 16,
            ] {
                visit(cycles)?;
            }
            Some(())
        }
        // ASM loc_2744 -> loc_2762 -> loc_2788 -> loc_2850: the first
        // gameplay pass starts on a live record, rejects it to Kid's right,
        // then finds no matching active platform in sub_2728.
        ModeLevelPlatformLoaderWorkLoc2744::Placement {
            wrapped_to_start: false,
            range: ModeLevelPlatformLoaderRangeLoc2762::OutOfRangeRight,
            existing_slot: ModeLevelPlatformLoaderExistingSlotSub2728::Missing,
        } => {
            for cycles in [
                18, 16, 12, 4, 12, 12, 8, 10, 16, 12, 4, 12, 4, 12, 16, 4, 12, 4, 12, 16, 4, 10,
                18, 8, 12, 10, 4, 16, 10, 20, 16,
            ] {
                visit(cycles)?;
            }
            Some(())
        }
        ModeLevelPlatformLoaderWorkLoc2744::NoLayoutReturn => {
            // ASM Mode_Level loc_822 BSR and loc_2744's null-layout return.
            for cycles in [18, 16, 10, 16] {
                visit(cycles)?;
            }
            Some(())
        }
        _ => None,
    }
}

/// Raw routine-body cost for classified ASM `Manage_PlatformLoading`
/// (`loc_2744`) work.
pub fn manage_platform_loading_raw_cycles_loc_2744(
    work: ModeLevelPlatformLoaderWorkLoc2744,
) -> Option<M68kCycleCount> {
    let mut raw_cycles = 0_u32;
    visit_manage_platform_loading_instructions_loc_2744(work, |instruction_cycles| {
        raw_cycles = raw_cycles.checked_add(instruction_cycles)?;
        Some(())
    })?;
    // The visitor includes Mode_Level loc_822's 18-cycle BSR shell.
    raw_cycles.checked_sub(18).map(M68kCycleCount::from_raw)
}

/// Co-schedule ASM `Mode_Level loc_822 -> Manage_PlatformLoading -> loc_826`
/// with asynchronous hardware.
pub fn manage_platform_loading_advance_with_scheduler_loc_2744(
    mut phase: M68kRefreshPhase,
    work: ModeLevelPlatformLoaderWorkLoc2744,
    mut advance_hardware: impl FnMut(M68kCycleCount) -> Option<M68kCycleCount>,
) -> Option<M68kRefreshAdvance> {
    let mut elapsed = 0_u32;
    let mut refresh_events = 0_u32;

    visit_manage_platform_loading_instructions_loc_2744(work, |instruction_cycles| {
        let phase_after_raw = u32::from(phase.get()).checked_add(instruction_cycles)?;
        let refresh_wait = u32::from(phase_after_raw >= 128) * 2;
        phase = M68kRefreshPhase::new(
            u8::try_from(phase_after_raw % 128).expect("refresh phase remains below 128"),
        )
        .expect("refresh phase remains below 128");
        elapsed = elapsed.checked_add(instruction_cycles)?;

        let generated_wait = advance_hardware(M68kCycleCount::from_raw(instruction_cycles))?;
        let mut pending_wait = generated_wait.get().checked_add(refresh_wait)?;
        refresh_events = refresh_events.checked_add(refresh_wait / 2)?;
        while pending_wait != 0 {
            let wait = M68kCycleCount::from_raw(pending_wait);
            phase = phase.advance_external_wait(wait).phase_after;
            elapsed = elapsed.checked_add(pending_wait)?;
            pending_wait = advance_hardware(wait)?.get();
        }
        Some(())
    })?;

    Some(M68kRefreshAdvance {
        elapsed: M68kCycleCount::from_raw(elapsed),
        phase_after: phase,
        refresh_events,
    })
}

fn visit_terrain_effects_instructions_sub_5e02(
    murder_wall_active: bool,
    mut visit: impl FnMut(u32) -> Option<()>,
) -> Option<()> {
    if murder_wall_active {
        return None;
    }

    // ASM Mode_Level loc_81E BSR, then sub_5E02: clear MurderWall_flag takes
    // the first branch directly to return_5F68 without touching Kid or the
    // sprite table.
    for cycles in [18, 12, 10, 16] {
        visit(cycles)?;
    }
    Some(())
}

/// Raw routine-body cost for ASM murder-wall sprite work in `sub_5E02`.
pub fn terrain_effects_raw_cycles_sub_5e02(murder_wall_active: bool) -> Option<M68kCycleCount> {
    let mut raw_cycles = 0_u32;
    visit_terrain_effects_instructions_sub_5e02(murder_wall_active, |instruction_cycles| {
        raw_cycles = raw_cycles.checked_add(instruction_cycles)?;
        Some(())
    })?;
    // The visitor includes Mode_Level loc_81E's 18-cycle BSR shell.
    raw_cycles.checked_sub(18).map(M68kCycleCount::from_raw)
}

/// Co-schedule ASM `Mode_Level loc_81E -> sub_5E02 -> loc_822` with
/// asynchronous hardware.
pub fn terrain_effects_advance_with_scheduler_sub_5e02(
    mut phase: M68kRefreshPhase,
    murder_wall_active: bool,
    mut advance_hardware: impl FnMut(M68kCycleCount) -> Option<M68kCycleCount>,
) -> Option<M68kRefreshAdvance> {
    let mut elapsed = 0_u32;
    let mut refresh_events = 0_u32;

    visit_terrain_effects_instructions_sub_5e02(murder_wall_active, |instruction_cycles| {
        let phase_after_raw = u32::from(phase.get()).checked_add(instruction_cycles)?;
        let refresh_wait = u32::from(phase_after_raw >= 128) * 2;
        phase = M68kRefreshPhase::new(
            u8::try_from(phase_after_raw % 128).expect("refresh phase remains below 128"),
        )
        .expect("refresh phase remains below 128");
        elapsed = elapsed.checked_add(instruction_cycles)?;

        let generated_wait = advance_hardware(M68kCycleCount::from_raw(instruction_cycles))?;
        let mut pending_wait = generated_wait.get().checked_add(refresh_wait)?;
        refresh_events = refresh_events.checked_add(refresh_wait / 2)?;
        while pending_wait != 0 {
            let wait = M68kCycleCount::from_raw(pending_wait);
            phase = phase.advance_external_wait(wait).phase_after;
            elapsed = elapsed.checked_add(pending_wait)?;
            pending_wait = advance_hardware(wait)?.get();
        }
        Some(())
    })?;

    Some(M68kRefreshAdvance {
        elapsed: M68kCycleCount::from_raw(elapsed),
        phase_after: phase,
        refresh_events,
    })
}

fn visit_block_debris_sprites_instructions_sub_2a4c(
    work: ModeLevelBlockDebrisWorkSub2A4C,
    mut visit: impl FnMut(u32) -> Option<()>,
) -> Option<()> {
    if work.active_slot_count > 6 {
        return None;
    }
    if work.active_slot_count != 0 {
        // ASM sub_2A4C loc_2A56, published TAS frames 1349-1350. Each
        // active scratch slot emits five visible debris sprites through the
        // same 112-instruction path; the remaining slots take the skip path.
        const ONE_ACTIVE_TRACE: [u8; 148] = [
            16, 8, 4, 8, 12, 8, 16, 4, 12, 8, 12, 12, 8, 12, 16, 24, 16, 24, 24, 12, 4, 8, 12, 8,
            12, 4, 12, 4, 12, 8, 12, 4, 8, 4, 8, 8, 8, 12, 16, 24, 16, 24, 24, 12, 4, 4, 12, 4, 4,
            8, 4, 8, 8, 8, 12, 16, 24, 16, 24, 24, 12, 4, 4, 12, 4, 4, 8, 4, 8, 8, 8, 12, 16, 24,
            16, 24, 24, 12, 4, 4, 12, 4, 4, 8, 4, 8, 8, 8, 12, 16, 24, 16, 24, 24, 12, 4, 4, 12, 4,
            8, 12, 4, 8, 4, 8, 8, 8, 10, 16, 12, 16, 8, 8, 10, 4, 8, 10, 16, 8, 8, 10, 4, 8, 10,
            16, 8, 8, 10, 4, 8, 10, 16, 8, 8, 10, 4, 8, 10, 16, 8, 8, 10, 4, 8, 10, 16, 10, 16,
        ];
        visit(18)?;
        for &cycles in &ONE_ACTIVE_TRACE[..2] {
            visit(u32::from(cycles))?;
        }
        for _ in 0..work.active_slot_count {
            for &cycles in &ONE_ACTIVE_TRACE[2..114] {
                visit(u32::from(cycles))?;
            }
        }
        for slot in work.active_slot_count..6 {
            let instruction_cycles: &[u8] = if slot == 5 {
                &[4, 8, 10, 16, 10, 16]
            } else {
                &[4, 8, 10, 16, 8, 8, 10]
            };
            for &cycles in instruction_cycles {
                visit(u32::from(cycles))?;
            }
        }
        return Some(());
    }

    // ASM Mode_Level loc_87A BSR and sub_2A4C setup through loc_2A56.
    for cycles in [18, 16, 8] {
        visit(cycles)?;
    }
    for slot in 0..6 {
        // ASM loc_2A56/loc_2BA6: the active byte is clear, so each fixed
        // scratch slot takes the skip path. The sixth slot returns instead
        // of advancing and branching to the next slot.
        let instruction_cycles: &[u32] = if slot == 5 {
            &[4, 8, 10, 16, 10, 16]
        } else {
            &[4, 8, 10, 16, 8, 8, 10]
        };
        for &cycles in instruction_cycles {
            visit(cycles)?;
        }
    }
    Some(())
}

/// Raw routine-body cost for ASM block-debris sprite compilation `sub_2A4C`.
pub fn block_debris_sprites_raw_cycles_sub_2a4c(
    work: ModeLevelBlockDebrisWorkSub2A4C,
) -> Option<M68kCycleCount> {
    let mut raw_cycles = 0_u32;
    visit_block_debris_sprites_instructions_sub_2a4c(work, |instruction_cycles| {
        raw_cycles = raw_cycles.checked_add(instruction_cycles)?;
        Some(())
    })?;
    raw_cycles.checked_sub(18).map(M68kCycleCount::from_raw)
}

/// Co-schedule ASM `Mode_Level loc_87A -> sub_2A4C -> loc_87E` with
/// asynchronous hardware.
pub fn block_debris_sprites_advance_with_scheduler_sub_2a4c(
    mut phase: M68kRefreshPhase,
    work: ModeLevelBlockDebrisWorkSub2A4C,
    mut advance_hardware: impl FnMut(M68kCycleCount) -> Option<M68kCycleCount>,
) -> Option<M68kRefreshAdvance> {
    let mut elapsed = 0_u32;
    let mut refresh_events = 0_u32;

    visit_block_debris_sprites_instructions_sub_2a4c(work, |instruction_cycles| {
        let phase_after_raw = u32::from(phase.get()).checked_add(instruction_cycles)?;
        let refresh_wait = u32::from(phase_after_raw >= 128) * 2;
        phase = M68kRefreshPhase::new(
            u8::try_from(phase_after_raw % 128).expect("refresh phase remains below 128"),
        )
        .expect("refresh phase remains below 128");
        elapsed = elapsed.checked_add(instruction_cycles)?;
        let generated_wait = advance_hardware(M68kCycleCount::from_raw(instruction_cycles))?;
        let mut pending_wait = generated_wait.get().checked_add(refresh_wait)?;
        refresh_events = refresh_events.checked_add(refresh_wait / 2)?;
        while pending_wait != 0 {
            let wait = M68kCycleCount::from_raw(pending_wait);
            phase = phase.advance_external_wait(wait).phase_after;
            elapsed = elapsed.checked_add(pending_wait)?;
            pending_wait = advance_hardware(wait)?.get();
        }
        Some(())
    })?;

    Some(M68kRefreshAdvance {
        elapsed: M68kCycleCount::from_raw(elapsed),
        phase_after: phase,
        refresh_events,
    })
}

/// Raw routine-body cost for ASM `DiamondPower_CompileSprites` (`sub_3F798`).
pub fn diamond_power_compile_sprites_raw_cycles_sub_3f798(
    work: ModeLevelDiamondPowerCompileWorkSub3F798,
) -> Option<M68kCycleCount> {
    if work.buffered_entry_count != 0 {
        return None;
    }

    // ASM sub_3F798: the buffer end pointer equals `$FFFFF5C2`, so the
    // initial comparison branches directly to return_3F7CE.
    Some(M68kCycleCount::from_raw(56))
}

/// Co-schedule ASM `Mode_Level loc_882 -> DiamondPower_CompileSprites
/// sub_3F798 -> loc_88E` with asynchronous hardware.
pub fn diamond_power_compile_sprites_advance_with_scheduler_sub_3f798(
    mut phase: M68kRefreshPhase,
    work: ModeLevelDiamondPowerCompileWorkSub3F798,
    mut advance_hardware: impl FnMut(M68kCycleCount) -> Option<M68kCycleCount>,
) -> Option<M68kRefreshAdvance> {
    if work.buffered_entry_count != 0 {
        return None;
    }

    // ASM loc_882 fall-through and j_DiamondPower_CompileSprites wrapper,
    // then sub_3F798's empty-buffer branch to return_3F7CE.
    const INSTRUCTIONS: &[u32] = &[12, 8, 20, 10, 16, 8, 6, 10, 16];
    let mut elapsed = 0_u32;
    let mut refresh_events = 0_u32;

    for &instruction_cycles in INSTRUCTIONS {
        let phase_after_raw = u32::from(phase.get()).checked_add(instruction_cycles)?;
        let refresh_wait = u32::from(phase_after_raw >= 128) * 2;
        phase = M68kRefreshPhase::new(
            u8::try_from(phase_after_raw % 128).expect("refresh phase remains below 128"),
        )
        .expect("refresh phase remains below 128");
        elapsed = elapsed.checked_add(instruction_cycles)?;
        let generated_wait = advance_hardware(M68kCycleCount::from_raw(instruction_cycles))?;
        let mut pending_wait = generated_wait.get().checked_add(refresh_wait)?;
        refresh_events = refresh_events.checked_add(refresh_wait / 2)?;
        while pending_wait != 0 {
            let wait = M68kCycleCount::from_raw(pending_wait);
            phase = phase.advance_external_wait(wait).phase_after;
            elapsed = elapsed.checked_add(pending_wait)?;
            pending_wait = advance_hardware(wait)?.get();
        }
    }

    Some(M68kRefreshAdvance {
        elapsed: M68kCycleCount::from_raw(elapsed),
        phase_after: phase,
        refresh_events,
    })
}

fn visit_kid_projectile_collision_instructions_sub_1d76(
    work: ModeLevelKidProjectileCollisionWorkSub1D76,
    mut visit: impl FnMut(u32) -> Option<()>,
) -> Option<()> {
    if work.kid_projectile_object_count != 0 || work.diamond_power_object_count != 0 {
        return None;
    }

    // ASM Mode_Level loc_872 BSR, then sub_1D76/sub_1D84: both `$FFFFF862`
    // and `$FFFFF5A0` sentinels have clear successor links, so each pass
    // returns after its first load.
    for cycles in [18, 8, 4, 18, 16, 10, 16, 8, 4, 16, 10, 16] {
        visit(cycles)?;
    }
    Some(())
}

/// Raw routine-body cost for ASM Kid-projectile collision wrapper `sub_1D76`.
pub fn kid_projectile_collision_raw_cycles_sub_1d76(
    work: ModeLevelKidProjectileCollisionWorkSub1D76,
) -> Option<M68kCycleCount> {
    let mut raw_cycles = 0_u32;
    visit_kid_projectile_collision_instructions_sub_1d76(work, |instruction_cycles| {
        raw_cycles = raw_cycles.checked_add(instruction_cycles)?;
        Some(())
    })?;
    raw_cycles.checked_sub(18).map(M68kCycleCount::from_raw)
}

/// Co-schedule ASM `Mode_Level loc_872 -> sub_1D76 -> loc_876` with
/// asynchronous hardware.
pub fn kid_projectile_collision_advance_with_scheduler_sub_1d76(
    mut phase: M68kRefreshPhase,
    work: ModeLevelKidProjectileCollisionWorkSub1D76,
    mut advance_hardware: impl FnMut(M68kCycleCount) -> Option<M68kCycleCount>,
) -> Option<M68kRefreshAdvance> {
    let mut elapsed = 0_u32;
    let mut refresh_events = 0_u32;

    visit_kid_projectile_collision_instructions_sub_1d76(work, |instruction_cycles| {
        let phase_after_raw = u32::from(phase.get()).checked_add(instruction_cycles)?;
        let refresh_wait = u32::from(phase_after_raw >= 128) * 2;
        phase = M68kRefreshPhase::new(
            u8::try_from(phase_after_raw % 128).expect("refresh phase remains below 128"),
        )
        .expect("refresh phase remains below 128");
        elapsed = elapsed.checked_add(instruction_cycles)?;
        let generated_wait = advance_hardware(M68kCycleCount::from_raw(instruction_cycles))?;
        let mut pending_wait = generated_wait.get().checked_add(refresh_wait)?;
        refresh_events = refresh_events.checked_add(refresh_wait / 2)?;
        while pending_wait != 0 {
            let wait = M68kCycleCount::from_raw(pending_wait);
            phase = phase.advance_external_wait(wait).phase_after;
            elapsed = elapsed.checked_add(pending_wait)?;
            pending_wait = advance_hardware(wait)?.get();
        }
        Some(())
    })?;

    Some(M68kRefreshAdvance {
        elapsed: M68kCycleCount::from_raw(elapsed),
        phase_after: phase,
        refresh_events,
    })
}

fn visit_gfx_objects_collision_instructions_sub_16f0(
    list: ModeLevelGfxCollisionListWork,
    mut visit: impl FnMut(u32) -> Option<()>,
) -> Option<()> {
    if list != ModeLevelGfxCollisionListWork::Empty {
        return None;
    }

    // ASM Mode_Level loc_82E/loc_836 LEA+BSR, then sub_16F0
    // loc_16FC/loc_17BE: save all registers, observe a null successor in
    // `4(a2)`, restore the frame, and return.
    for cycles in [8, 18, 128, 8, 12, 16, 10, 8, 132, 16] {
        visit(cycles)?;
    }
    Some(())
}

/// Raw routine-body cost for an empty ASM `GfxObjects_Collision` (`sub_16F0`)
/// special-chain pass.
pub fn gfx_objects_collision_raw_cycles_sub_16f0(
    list: ModeLevelGfxCollisionListWork,
) -> Option<M68kCycleCount> {
    let mut raw_cycles = 0_u32;
    visit_gfx_objects_collision_instructions_sub_16f0(list, |instruction_cycles| {
        raw_cycles = raw_cycles.checked_add(instruction_cycles)?;
        Some(())
    })?;
    // The visitor includes the Mode_Level 8-cycle LEA and 18-cycle BSR.
    raw_cycles.checked_sub(26).map(M68kCycleCount::from_raw)
}

/// Co-schedule either ASM `Mode_Level` empty-list `sub_16F0` call with
/// asynchronous hardware.
pub fn gfx_objects_collision_advance_with_scheduler_sub_16f0(
    mut phase: M68kRefreshPhase,
    list: ModeLevelGfxCollisionListWork,
    mut advance_hardware: impl FnMut(M68kCycleCount) -> Option<M68kCycleCount>,
) -> Option<M68kRefreshAdvance> {
    let mut elapsed = 0_u32;
    let mut refresh_events = 0_u32;

    visit_gfx_objects_collision_instructions_sub_16f0(list, |instruction_cycles| {
        let phase_after_raw = u32::from(phase.get()).checked_add(instruction_cycles)?;
        let refresh_wait = u32::from(phase_after_raw >= 128) * 2;
        phase = M68kRefreshPhase::new(
            u8::try_from(phase_after_raw % 128).expect("refresh phase remains below 128"),
        )
        .expect("refresh phase remains below 128");
        elapsed = elapsed.checked_add(instruction_cycles)?;

        let generated_wait = advance_hardware(M68kCycleCount::from_raw(instruction_cycles))?;
        let mut pending_wait = generated_wait.get().checked_add(refresh_wait)?;
        refresh_events = refresh_events.checked_add(refresh_wait / 2)?;
        while pending_wait != 0 {
            let wait = M68kCycleCount::from_raw(pending_wait);
            phase = phase.advance_external_wait(wait).phase_after;
            elapsed = elapsed.checked_add(pending_wait)?;
            pending_wait = advance_hardware(wait)?.get();
        }
        Some(())
    })?;

    Some(M68kRefreshAdvance {
        elapsed: M68kCycleCount::from_raw(elapsed),
        phase_after: phase,
        refresh_events,
    })
}

/// Co-schedule ASM `Mode_Level loc_83E..loc_84A` damage-state latching with
/// asynchronous hardware.
pub fn mode_level_latch_damage_state_advance_loc_83e(
    mut phase: M68kRefreshPhase,
    mut advance_hardware: impl FnMut(M68kCycleCount) -> Option<M68kCycleCount>,
) -> Option<M68kRefreshAdvance> {
    let mut elapsed = 0_u32;
    let mut refresh_events = 0_u32;

    // ASM loc_83E/loc_844 are memory-to-memory MOVE.B instructions and
    // loc_84A is the memory Scc that clears `$FFFFFA75`.
    for instruction_cycles in [20, 20, 16] {
        let phase_after_raw = u32::from(phase.get()).checked_add(instruction_cycles)?;
        let refresh_wait = u32::from(phase_after_raw >= 128) * 2;
        phase = M68kRefreshPhase::new(
            u8::try_from(phase_after_raw % 128).expect("refresh phase remains below 128"),
        )
        .expect("refresh phase remains below 128");
        elapsed = elapsed.checked_add(instruction_cycles)?;

        let generated_wait = advance_hardware(M68kCycleCount::from_raw(instruction_cycles))?;
        let mut pending_wait = generated_wait.get().checked_add(refresh_wait)?;
        refresh_events = refresh_events.checked_add(refresh_wait / 2)?;
        while pending_wait != 0 {
            let wait = M68kCycleCount::from_raw(pending_wait);
            phase = phase.advance_external_wait(wait).phase_after;
            elapsed = elapsed.checked_add(pending_wait)?;
            pending_wait = advance_hardware(wait)?.get();
        }
    }

    Some(M68kRefreshAdvance {
        elapsed: M68kCycleCount::from_raw(elapsed),
        phase_after: phase,
        refresh_events,
    })
}

fn visit_gfx_objects_collision_kid_instructions_sub_1b76(
    list: ModeLevelGfxCollisionListWork,
    work: &ModeLevelGfxCollisionWork,
    mut visit: impl FnMut(u32) -> Option<()>,
) -> Option<()> {
    if list != ModeLevelGfxCollisionListWork::Empty || work.kid_is_invulnerable {
        return None;
    }

    // ASM Mode_Level LEA+BSR, then sub_1B76 loc_1B94/loc_1D00: load the Kid
    // hitbox, observe a null successor in `4(a2)`, then take the clear
    // KidIsInvulnerable return.
    for cycles in [8, 18, 12, 10, 8, 16, 12, 12, 12, 12, 16, 10, 8, 12, 10, 16] {
        visit(cycles)?;
    }
    Some(())
}

/// Raw routine-body cost for an empty ASM `GfxObjects_CollisionKid`
/// (`sub_1B76`) special-chain pass.
pub fn gfx_objects_collision_kid_raw_cycles_sub_1b76(
    list: ModeLevelGfxCollisionListWork,
    work: &ModeLevelGfxCollisionWork,
) -> Option<M68kCycleCount> {
    let mut raw_cycles = 0_u32;
    visit_gfx_objects_collision_kid_instructions_sub_1b76(list, work, |instruction_cycles| {
        raw_cycles = raw_cycles.checked_add(instruction_cycles)?;
        Some(())
    })?;
    // The visitor includes Mode_Level's 8-cycle LEA and 18-cycle BSR.
    raw_cycles.checked_sub(26).map(M68kCycleCount::from_raw)
}

/// Co-schedule an ASM `Mode_Level` empty-list `sub_1B76` call with
/// asynchronous hardware.
pub fn gfx_objects_collision_kid_advance_with_scheduler_sub_1b76(
    mut phase: M68kRefreshPhase,
    list: ModeLevelGfxCollisionListWork,
    work: &ModeLevelGfxCollisionWork,
    mut advance_hardware: impl FnMut(M68kCycleCount) -> Option<M68kCycleCount>,
) -> Option<M68kRefreshAdvance> {
    let mut elapsed = 0_u32;
    let mut refresh_events = 0_u32;

    visit_gfx_objects_collision_kid_instructions_sub_1b76(list, work, |instruction_cycles| {
        let phase_after_raw = u32::from(phase.get()).checked_add(instruction_cycles)?;
        let refresh_wait = u32::from(phase_after_raw >= 128) * 2;
        phase = M68kRefreshPhase::new(
            u8::try_from(phase_after_raw % 128).expect("refresh phase remains below 128"),
        )
        .expect("refresh phase remains below 128");
        elapsed = elapsed.checked_add(instruction_cycles)?;

        let generated_wait = advance_hardware(M68kCycleCount::from_raw(instruction_cycles))?;
        let mut pending_wait = generated_wait.get().checked_add(refresh_wait)?;
        refresh_events = refresh_events.checked_add(refresh_wait / 2)?;
        while pending_wait != 0 {
            let wait = M68kCycleCount::from_raw(pending_wait);
            phase = phase.advance_external_wait(wait).phase_after;
            elapsed = elapsed.checked_add(pending_wait)?;
            pending_wait = advance_hardware(wait)?.get();
        }
        Some(())
    })?;

    Some(M68kRefreshAdvance {
        elapsed: M68kCycleCount::from_raw(elapsed),
        phase_after: phase,
        refresh_events,
    })
}

/// Raw routine-body cost for the kind-4 `$FFFFF86E` pass through ASM
/// `GfxObjects_CollisionKid` (`sub_1B76`).
fn visit_gfx_objects_collision_kid_f86e_instructions_sub_1b76(
    work: &ModeLevelGfxCollisionWork,
    mut visit: impl FnMut(u32) -> Option<()>,
) -> Option<()> {
    if work.kid_is_invulnerable {
        return None;
    }
    let ModeLevelGfxCollisionKidListWorkSub1B76::Classified { nodes } =
        &work.object_kind_four_list_f86e
    else {
        return None;
    };

    // ASM Mode_Level loc_86A LEA+BSR, then sub_1B76 through loc_1B94.
    for cycles in [8, 18, 12, 10, 8, 16, 12, 12, 12, 12] {
        visit(cycles)?;
    }

    for node in nodes {
        if *node == ModeLevelGfxCollisionKidNodeWorkSub1B76::PrizeOverlapLoc1d5a {
            // ASM frame 1057: loc_1B94 passes all four box comparisons, then
            // loc_1C18 recognises kind 4. loc_1D5A copies object_meta=$70 and
            // writes colid_kidbelow before the immediate loc_1D00 epilogue.
            for cycles in [
                16, 12, 4, 12, 8, 12, 8, 12, 18, 8, 12, 8, 8, 8, 8, 12, 4, 8, 12, 4, 8, 12, 12, 8,
                4, 8, 12, 4, 8, 10, 16, 10, 12, 8, 8, 12, 16, 16, 10, 8, 12, 10, 16,
            ] {
                visit(cycles)?;
            }
            return Some(());
        }
        let instruction_cycles: &[u32] = match node {
            // ASM loc_1B94 -> loc_1BC0: first CollisionSize word is zero.
            ModeLevelGfxCollisionKidNodeWorkSub1B76::CollisionRowsSentinelLoc1bc0 => {
                &[16, 12, 4, 12, 8, 12, 8, 12, 18, 8, 12, 8, 8, 8, 10]
            }
            // ASM loc_1B94 -> loc_1BC8 -> loc_1BC0: reject the first box on
            // its left edge, then read the zero terminator.
            ModeLevelGfxCollisionKidNodeWorkSub1B76::UnflippedLeftEdgeBeyondKidRightLoc1bc8 => &[
                16, 12, 4, 12, 8, 12, 8, 12, 18, 8, 12, 8, 8, 8, 8, 12, 4, 10, 8, 8, 10,
            ],
            // ASM loc_1B94 -> loc_1BD0 -> loc_1BC0: reject the first box on
            // its right edge, then read the zero terminator.
            ModeLevelGfxCollisionKidNodeWorkSub1B76::UnflippedRightEdgeBeforeKidLeftLoc1bd0 => &[
                16, 12, 4, 12, 8, 12, 8, 12, 18, 8, 12, 8, 8, 8, 8, 12, 4, 8, 12, 4, 10, 8, 8, 10,
            ],
            // ASM loc_1B94 -> loc_1BDE -> loc_1BC0: reject the first box on
            // its top edge, then read the zero terminator.
            ModeLevelGfxCollisionKidNodeWorkSub1B76::UnflippedTopEdgeBeyondKidBottomLoc1bde => &[
                16, 12, 4, 12, 8, 12, 8, 12, 18, 8, 12, 8, 8, 8, 8, 12, 4, 8, 12, 4, 8, 12, 12, 8,
                4, 10, 8, 8, 10,
            ],
            // ASM loc_1B94 -> loc_1BE6 -> loc_1BC0: reject on the bottom
            // edge, then read the zero terminator.
            ModeLevelGfxCollisionKidNodeWorkSub1B76::UnflippedBottomEdgeBeforeKidTopLoc1be6 => &[
                16, 12, 4, 12, 8, 12, 8, 12, 18, 8, 12, 8, 8, 8, 8, 12, 4, 8, 12, 4, 8, 12, 12, 8,
                4, 8, 12, 4, 10, 8, 8, 10,
            ],
            ModeLevelGfxCollisionKidNodeWorkSub1B76::PrizeOverlapLoc1d5a => unreachable!(),
        };
        for &cycles in instruction_cycles {
            visit(cycles)?;
        }
    }

    // ASM loc_1B94 null successor, loc_1D00 clear-invulnerability epilogue,
    // and return to Mode_Level loc_872.
    for cycles in [16, 10, 8, 12, 10, 16] {
        visit(cycles)?;
    }
    Some(())
}

/// Raw routine-body cost for the kind-4 `$FFFFF86E` pass through ASM
/// `GfxObjects_CollisionKid` (`sub_1B76`).
pub fn gfx_objects_collision_kid_f86e_raw_cycles_sub_1b76(
    work: &ModeLevelGfxCollisionWork,
) -> Option<M68kCycleCount> {
    let mut raw_cycles = 0_u32;
    visit_gfx_objects_collision_kid_f86e_instructions_sub_1b76(work, |instruction_cycles| {
        raw_cycles = raw_cycles.checked_add(instruction_cycles)?;
        Some(())
    })?;
    // The visitor includes Mode_Level's 8-cycle LEA and 18-cycle BSR.
    raw_cycles.checked_sub(26).map(M68kCycleCount::from_raw)
}

/// Co-schedule ASM `Mode_Level loc_86A -> sub_1B76 -> loc_872` with
/// asynchronous hardware.
pub fn gfx_objects_collision_kid_f86e_advance_with_scheduler_sub_1b76(
    mut phase: M68kRefreshPhase,
    work: &ModeLevelGfxCollisionWork,
    mut advance_hardware: impl FnMut(M68kCycleCount) -> Option<M68kCycleCount>,
) -> Option<M68kRefreshAdvance> {
    let mut elapsed = 0_u32;
    let mut refresh_events = 0_u32;

    visit_gfx_objects_collision_kid_f86e_instructions_sub_1b76(work, |instruction_cycles| {
        let phase_after_raw = u32::from(phase.get()).checked_add(instruction_cycles)?;
        let refresh_wait = u32::from(phase_after_raw >= 128) * 2;
        phase = M68kRefreshPhase::new(
            u8::try_from(phase_after_raw % 128).expect("refresh phase remains below 128"),
        )
        .expect("refresh phase remains below 128");
        elapsed = elapsed.checked_add(instruction_cycles)?;
        let generated_wait = advance_hardware(M68kCycleCount::from_raw(instruction_cycles))?;
        let mut pending_wait = generated_wait.get().checked_add(refresh_wait)?;
        refresh_events = refresh_events.checked_add(refresh_wait / 2)?;
        while pending_wait != 0 {
            let wait = M68kCycleCount::from_raw(pending_wait);
            phase = phase.advance_external_wait(wait).phase_after;
            elapsed = elapsed.checked_add(pending_wait)?;
            pending_wait = advance_hardware(wait)?.get();
        }
        Some(())
    })?;

    Some(M68kRefreshAdvance {
        elapsed: M68kCycleCount::from_raw(elapsed),
        phase_after: phase,
        refresh_events,
    })
}

#[derive(Clone, Copy)]
enum GfxAnimationBranchSub113E {
    Inactive,
    Countdown,
    CountdownToOne,
    DispatchNormal,
    DispatchBackThenNormal,
    DispatchStop,
}

fn non_looping_gfx_animation_branch_sub_113e(
    durations: &[u16],
    target_tick: u16,
) -> Option<GfxAnimationBranchSub113E> {
    let mut timer = 1_u16;
    let mut frame_index = 0_usize;
    let mut active = true;
    let mut branch = GfxAnimationBranchSub113E::Inactive;

    for _ in 0..=target_tick {
        if !active {
            branch = GfxAnimationBranchSub113E::Inactive;
            continue;
        }
        timer = timer.checked_sub(1)?;
        if timer != 0 {
            branch = if timer == 1 {
                GfxAnimationBranchSub113E::CountdownToOne
            } else {
                GfxAnimationBranchSub113E::Countdown
            };
            continue;
        }
        let Some(&duration) = durations.get(frame_index) else {
            active = false;
            timer = 1;
            branch = GfxAnimationBranchSub113E::DispatchStop;
            continue;
        };
        timer = duration;
        frame_index += 1;
        branch = GfxAnimationBranchSub113E::DispatchNormal;
    }

    Some(branch)
}

fn looping_diamond_gfx_animation_branch_sub_113e(
    target_tick: u16,
) -> Option<GfxAnimationBranchSub113E> {
    const DURATIONS: [u16; 4] = [3; 4];
    let mut timer = 1_u16;
    let mut frame_index = 0_usize;
    let mut branch = GfxAnimationBranchSub113E::Inactive;

    for _ in 0..=target_tick {
        timer = timer.checked_sub(1)?;
        if timer != 0 {
            branch = if timer == 1 {
                GfxAnimationBranchSub113E::CountdownToOne
            } else {
                GfxAnimationBranchSub113E::Countdown
            };
            continue;
        }
        let looped = frame_index == DURATIONS.len();
        if looped {
            frame_index = 0;
        }
        timer = DURATIONS[frame_index];
        frame_index += 1;
        branch = if looped {
            GfxAnimationBranchSub113E::DispatchBackThenNormal
        } else {
            GfxAnimationBranchSub113E::DispatchNormal
        };
    }

    Some(branch)
}

fn gfx_animation_branch_sub_113e(
    node: ModeLevelGfxMoveAnimateNodeWorkSub1444,
) -> Option<GfxAnimationBranchSub113E> {
    match node {
        ModeLevelGfxMoveAnimateNodeWorkSub1444::StationaryUnanimated => {
            Some(GfxAnimationBranchSub113E::Inactive)
        }
        ModeLevelGfxMoveAnimateNodeWorkSub1444::KidPullUpStru8bb4 { animation_tick } => {
            // ASM stru_8BB4: seven non-looping three-tick frames.
            non_looping_gfx_animation_branch_sub_113e(&[3; 7], u16::from(animation_tick))
        }
        ModeLevelGfxMoveAnimateNodeWorkSub1444::PrizeWobble { allocation_age } => {
            // ASM stru_101B6: eight non-looping three-tick frames.
            non_looping_gfx_animation_branch_sub_113e(&[3; 8], allocation_age)
        }
        ModeLevelGfxMoveAnimateNodeWorkSub1444::PrizeFlash { allocation_age } => {
            // ASM loc_102B8 is allocated after its Object-list insertion
            // point, so age zero has not run Init_Animation yet.
            if allocation_age == 0 {
                Some(GfxAnimationBranchSub113E::Inactive)
            } else {
                // ASM stru_102DE: four non-looping frames with these exact
                // animation_timer reloads.
                non_looping_gfx_animation_branch_sub_113e(&[4, 6, 10, 4], allocation_age - 1)
            }
        }
        ModeLevelGfxMoveAnimateNodeWorkSub1444::FloatingDiamond { allocation_age } => {
            // ASM stru_10D6E: four three-tick frames followed by the loc_11AC
            // signed-backstep dispatch.
            looping_diamond_gfx_animation_branch_sub_113e(allocation_age)
        }
        ModeLevelGfxMoveAnimateNodeWorkSub1444::FloatingHelmet { allocation_age } => {
            // ASM stru_106D0..stru_10700: one duration-1 frame followed by
            // the loc_11AC signed-backstep command.
            Some(if allocation_age == 0 {
                GfxAnimationBranchSub113E::DispatchNormal
            } else {
                GfxAnimationBranchSub113E::DispatchBackThenNormal
            })
        }
    }
}

fn visit_gfx_objects_move_and_animate_instructions_sub_1444(
    work: &ModeLevelGfxMoveAnimateWorkSub1444,
    mut visit: impl FnMut(u32) -> Option<()>,
) -> Option<()> {
    let ModeLevelGfxMoveAnimateWorkSub1444::Classified { nodes } = work else {
        return None;
    };
    let mut visit_all = |instructions: &[u8]| {
        for &cycles in instructions {
            visit(u32::from(cycles))?;
        }
        Some(())
    };

    // ASM Mode_Level loc_816 BSR and sub_1444 through the first loc_144E.
    visit_all(&[18, 12, 8, 8])?;
    for &node in nodes {
        // ASM loc_144E live successor and GfxObject_Move sub_1120.
        visit_all(&[16, 8, 4, 18])?;
        if matches!(
            node,
            ModeLevelGfxMoveAnimateNodeWorkSub1444::FloatingHelmet { .. }
        ) {
            visit_all(&[12, 8, 12, 8, 16, 20, 16, 20, 16])?;
        } else {
            visit_all(&[12, 8, 12, 10, 16])?;
        }
        visit_all(&[18])?;
        match gfx_animation_branch_sub_113e(node)? {
            GfxAnimationBranchSub113E::Inactive => {
                visit_all(&[12, 12, 12, 12, 10, 12, 16])?;
            }
            GfxAnimationBranchSub113E::Countdown => {
                visit_all(&[12, 12, 12, 12, 8, 16, 16, 10, 16, 10, 12, 16])?;
            }
            GfxAnimationBranchSub113E::CountdownToOne => {
                visit_all(&[12, 12, 12, 12, 8, 16, 16, 10, 16, 8, 8, 8, 12, 16])?;
            }
            GfxAnimationBranchSub113E::DispatchNormal => {
                // ASM loc_1158 -> sub_1194, including
                // j_nullsub_2/nullsub_2.
                visit_all(&[
                    12, 12, 12, 12, 8, 16, 16, 8, 4, 8, 4, 4, 26, 16, 4, 8, 12, 8, 12, 18, 10, 16,
                    16, 10, 16, 10, 12, 16,
                ])?;
            }
            GfxAnimationBranchSub113E::DispatchBackThenNormal => {
                // ASM loc_1158 -> sub_11AC -> loc_1158, then the normal
                // sub_1194 dispatch above.
                visit_all(&[12, 12, 12, 12, 8, 16, 16, 8, 4, 8, 4, 4, 26, 16])?;
                visit_all(&[8, 4, 4, 8, 10])?;
                visit_all(&[
                    4, 8, 4, 4, 26, 16, 4, 8, 12, 8, 12, 18, 10, 16, 16, 10, 16, 10, 12, 16,
                ])?;
            }
            GfxAnimationBranchSub113E::DispatchStop => {
                // ASM GfxObject_Animate loc_1158 -> off_1176 -> loc_1182:
                // dispatch the zero terminator, latch the stopped flag, clear
                // animation ownership, seed timer one, and return to sub_1444.
                visit_all(&[
                    12, 12, 12, 12, 8, 16, 16, 8, 4, 8, 4, 4, 26, 16, 12, 12, 16, 12, 16,
                ])?;
            }
        }
        // ASM loc_145E loops to loc_144E.
        visit_all(&[10])?;
    }
    // ASM loc_144E null successor -> return_1460.
    visit_all(&[16, 10, 16])
}

/// Raw routine-body cost for ASM `GfxObjects_MoveAndAnimate sub_1444`.
pub fn gfx_objects_move_and_animate_raw_cycles_sub_1444(
    work: &ModeLevelGfxMoveAnimateWorkSub1444,
) -> Option<M68kCycleCount> {
    let mut raw_cycles = 0_u32;
    visit_gfx_objects_move_and_animate_instructions_sub_1444(work, |cycles| {
        raw_cycles = raw_cycles.checked_add(cycles)?;
        Some(())
    })?;
    // The visitor includes Mode_Level loc_816's 18-cycle BSR shell.
    raw_cycles.checked_sub(18).map(M68kCycleCount::from_raw)
}

/// Co-schedule ASM `Mode_Level loc_816 -> GfxObjects_MoveAndAnimate
/// return_1460` with asynchronous hardware.
pub fn gfx_objects_move_and_animate_advance_with_scheduler_sub_1444(
    mut phase: M68kRefreshPhase,
    work: &ModeLevelGfxMoveAnimateWorkSub1444,
    mut advance_hardware: impl FnMut(M68kCycleCount) -> Option<M68kCycleCount>,
) -> Option<M68kRefreshAdvance> {
    let mut elapsed = 0_u32;
    let mut refresh_events = 0_u32;

    visit_gfx_objects_move_and_animate_instructions_sub_1444(work, |instruction_cycles| {
        let phase_after_raw = u32::from(phase.get()).checked_add(instruction_cycles)?;
        let refresh_wait = u32::from(phase_after_raw >= 128) * 2;
        phase = M68kRefreshPhase::new(
            u8::try_from(phase_after_raw % 128).expect("refresh phase remains below 128"),
        )
        .expect("refresh phase remains below 128");
        elapsed = elapsed.checked_add(instruction_cycles)?;

        let generated_wait = advance_hardware(M68kCycleCount::from_raw(instruction_cycles))?;
        let mut pending_wait = generated_wait.get().checked_add(refresh_wait)?;
        refresh_events = refresh_events.checked_add(refresh_wait / 2)?;
        while pending_wait != 0 {
            let wait = M68kCycleCount::from_raw(pending_wait);
            phase = phase.advance_external_wait(wait).phase_after;
            elapsed = elapsed.checked_add(pending_wait)?;
            pending_wait = advance_hardware(wait)?.get();
        }
        Some(())
    })?;

    Some(M68kRefreshAdvance {
        elapsed: M68kCycleCount::from_raw(elapsed),
        phase_after: phase,
        refresh_events,
    })
}

fn visit_make_sprites_from_gfx_objects_instructions_sub_1492(
    work: &ModeLevelGfxSpriteWorkSub1492,
) -> Option<impl Iterator<Item = u32> + '_> {
    let ModeLevelGfxSpriteWorkSub1492::Classified { nodes } = work else {
        return None;
    };

    const SETUP: &[u32] = &[18, 16, 16, 16, 8];
    const HIDDEN: &[u32] = &[16, 8, 4, 18, 10, 16, 12, 10, 16, 10];
    const MAPPING_POINTER_NULL: &[u32] = &[16, 8, 4, 18, 10, 16, 12, 12, 12, 12, 18, 10, 16, 10];
    const VERTICALLY_CULLED: &[u32] = &[
        16, 8, 4, 18, 10, 16, 12, 12, 12, 12, 18, 12, 4, 12, 12, 12, 8, 12, 10, 4, 8, 10, 16, 10,
    ];
    const VERTICALLY_CULLED_ABOVE_VIEWPORT: &[u32] = &[
        16, 8, 4, 18, 10, 16, 12, 12, 12, 12, 18, 12, 4, 12, 12, 12, 8, 12, 10, 4, 8, 12, 8, 12,
        12, 12, 8, 10, 16, 10,
    ];
    const VISIBLE_SINGLE: &[u32] = &[
        16, 8, 4, 18, 10, 16, 12, 12, 12, 12, 18, 12, 4, 12, 12, 12, 8, 12, 10, 4, 8, 12, 8, 12,
        12, 12, 8, 12, 8, 12, 16, 4, 12, 12, 16, 10, 12, 8, 8, 12, 10, 12, 12, 10, 12, 12, 10, 8,
        4, 4, 8, 8, 4, 4, 8, 8, 8, 4, 4, 10, 4, 8, 8, 4, 10, 4, 8, 8, 4, 4, 4, 8, 4, 12, 4, 18, 12,
        12, 12, 16, 20, 12, 8, 20, 4, 12, 4, 14, 4, 16, 8, 16, 16, 10,
    ];
    const VISIBLE_SINGLE_DYNAMIC_ART_20X32: &[u32] = &[
        16, 8, 4, 18, 10, 16, 12, 12, 12, 12, 18, 12, 4, 12, 12, 12, 10, 12, 4, 8, 12, 8, 12, 12,
        12, 8, 12, 8, 12, 16, 4, 12, 12, 16, 10, 12, 10, 12, 12, 12, 10, 12, 12, 10, 8, 4, 4, 8, 8,
        4, 4, 8, 8, 8, 4, 4, 8, 4, 4, 4, 10, 4, 8, 8, 4, 4, 8, 8, 18, 12, 12, 12, 16, 20, 12, 10,
        20, 4, 12, 4, 14, 4, 16, 16, 8, 12, 16, 8, 8, 16, 4, 4, 10, 4, 10, 4, 8, 10, 16, 10,
    ];
    // ASM Make_SpriteFromGfxObject loc_132A -> loc_13A2 -> sub_13C0:
    // the flipped 20x32 dynamic-art route adjusts d6 before emitting its
    // single sprite. Selected-PC ledger: published Cyclone frame 1058.
    const VISIBLE_SINGLE_DYNAMIC_ART_20X32_FLIPPED: &[u32] = &[
        16, 8, 4, 18, 10, 16, 12, 12, 12, 12, 18, 12, 4, 12, 12, 12, 10, 12, 4, 8, 12, 8, 12, 12,
        12, 8, 12, 8, 12, 16, 4, 12, 12, 16, 10, 12, 10, 12, 12, 12, 10, 12, 12, 10, 8, 4, 4, 8, 8,
        4, 4, 8, 8, 8, 4, 4, 10, 4, 8, 8, 4, 10, 4, 8, 8, 4, 4, 4, 8, 4, 12, 4, 18, 12, 12, 12, 16,
        20, 12, 10, 20, 4, 12, 4, 14, 4, 16, 16, 8, 12, 16, 8, 8, 16, 16, 10,
    ];
    const VISIBLE_SINGLE_DYNAMIC_ART_32X24: &[u32] = &[
        16, 8, 4, 18, 10, 16, 12, 12, 12, 12, 18, 12, 4, 12, 12, 12, 10, 12, 4, 8, 12, 8, 12, 12,
        12, 8, 12, 8, 12, 16, 4, 12, 12, 16, 10, 12, 10, 12, 12, 12, 10, 12, 12, 10, 8, 4, 4, 8, 8,
        4, 4, 8, 8, 8, 4, 4, 10, 4, 8, 8, 4, 10, 4, 8, 8, 4, 4, 4, 8, 4, 12, 4, 18, 12, 12, 12, 16,
        20, 12, 10, 20, 4, 12, 4, 14, 4, 16, 16, 8, 12, 16, 8, 8, 16, 16, 10,
    ];
    // ASM loc_129E consumes the pull-up mapping bytes `19 0D 001C 001D`
    // at ROM $A2E76. Width 28 and height 29 take the same instruction route
    // as the walking 32x24 mapping: loc_12EC -> loc_130E -> sub_13C0.
    const VISIBLE_SINGLE_DYNAMIC_ART_28X29_PULL_UP: &[u32] = VISIBLE_SINGLE_DYNAMIC_ART_32X24;
    const VISIBLE_TWO: &[u32] = &[
        16, 8, 4, 18, 10, 16, 12, 12, 12, 12, 18, 12, 4, 12, 12, 12, 10, 12, 4, 8, 12, 8, 12, 12,
        12, 8, 12, 8, 12, 16, 4, 12, 12, 16, 10, 12, 10, 12, 12, 12, 10, 12, 12, 10, 8, 4, 4, 8, 8,
        4, 4, 8, 8, 8, 4, 4, 8, 4, 4, 4, 10, 4, 8, 8, 4, 4, 8, 8, 18, 12, 12, 12, 16, 20, 12, 10,
        20, 4, 12, 4, 14, 4, 16, 16, 8, 12, 16, 8, 8, 16, 4, 4, 10, 4, 10, 4, 8, 8, 4, 10, 4, 8, 8,
        4, 4, 4, 8, 4, 12, 4, 18, 12, 12, 12, 16, 20, 12, 10, 20, 4, 12, 4, 14, 4, 16, 16, 8, 12,
        16, 8, 8, 16, 16, 10,
    ];
    const TAIL: &[u32] = &[16, 10, 16, 16, 12, 16];

    Some(
        SETUP
            .iter()
            .copied()
            .chain(nodes.iter().flat_map(|node| {
                match node {
                    // ASM sub_1492 -> j_Make_SpriteFromGfxObject -> sub_11B6:
                    // `$13(a3)` is clear and returns through return_129C.
                    ModeLevelGfxSpriteNodeWorkSub1492::HiddenFlagClear => HIDDEN,
                    // ASM sub_11B6 loc_11D0: `$13(a3)` is set but the selected
                    // Data_Index mapping pointer is null.
                    ModeLevelGfxSpriteNodeWorkSub1492::MappingPointerNull => MAPPING_POINTER_NULL,
                    // ASM loc_11E4 rejects the mapped object below the viewport
                    // before any sub_13C0 sprite entry is emitted.
                    ModeLevelGfxSpriteNodeWorkSub1492::VerticallyCulled => VERTICALLY_CULLED,
                    // ASM loc_120C rejects the mapped object above the viewport.
                    ModeLevelGfxSpriteNodeWorkSub1492::VerticallyCulledAboveViewport => {
                        VERTICALLY_CULLED_ABOVE_VIEWPORT
                    }
                    // ASM loc_129E -> sub_13C0 emits one normal-orientation 16x16
                    // mapping entry before the zero terminator.
                    ModeLevelGfxSpriteNodeWorkSub1492::VisibleSingle16x16Normal => VISIBLE_SINGLE,
                    // ASM sub_B270 state 0 selects `unk_A3B66` (22x32).
                    // Oracle frame 983 follows the same loc_129E -> sub_13C0
                    // selected-PC stream as the 20x32 `unk_A3CEC` mapping.
                    ModeLevelGfxSpriteNodeWorkSub1492::VisibleDynamicArt22x32AirborneNormal => {
                        VISIBLE_SINGLE_DYNAMIC_ART_20X32
                    }
                    // ASM loc_129E -> sub_13C0 loc_141A: the grounded Kid's
                    // 20x32 mapping emits one sprite and appends its dynamic
                    // art span to the DMA queue.
                    ModeLevelGfxSpriteNodeWorkSub1492::VisibleSingleDynamicArt20x32Normal => {
                        VISIBLE_SINGLE_DYNAMIC_ART_20X32
                    }
                    // ASM loc_1292 selects the x-flipped sprite compiler at
                    // loc_132A; loc_13A2 contributes its extra d6 adjustment.
                    ModeLevelGfxSpriteNodeWorkSub1492::VisibleSingleDynamicArt20x32Flipped => {
                        VISIBLE_SINGLE_DYNAMIC_ART_20X32_FLIPPED
                    }
                    // ASM loc_129E reads ROM $A2E76 as 28x29, then takes
                    // loc_12EC -> loc_130E -> sub_13C0 loc_141A.
                    ModeLevelGfxSpriteNodeWorkSub1492::VisibleSingleDynamicArt28x29PullUpNormal => {
                        VISIBLE_SINGLE_DYNAMIC_ART_28X29_PULL_UP
                    }
                    // ASM stru_8BB4's second frame is 32x18. loc_129E takes
                    // loc_12EC -> loc_12F4 -> return_1328, whose selected-PC
                    // instruction ledger matches the 20x32 single-sprite
                    // route while emitting a different hardware size word.
                    ModeLevelGfxSpriteNodeWorkSub1492::VisibleSingleDynamicArt32x18PullUpNormal => {
                        VISIBLE_SINGLE_DYNAMIC_ART_20X32
                    }
                    // ASM loc_129E -> loc_12EC -> loc_130E -> sub_13C0
                    // loc_141A emits the walking Kid's one 32x24 dynamic-art
                    // sprite before returning directly through return_1328.
                    ModeLevelGfxSpriteNodeWorkSub1492::VisibleSingleDynamicArt32x24Normal => {
                        VISIBLE_SINGLE_DYNAMIC_ART_32X24
                    }
                    // ASM loc_129E -> sub_13C0 emits two normal-orientation mapping
                    // entries before the zero terminator.
                    ModeLevelGfxSpriteNodeWorkSub1492::VisibleTwoSpriteNormal => VISIBLE_TWO,
                }
                .iter()
                .copied()
            }))
            // ASM loc_14A4 null successor, queue writeback/terminator, RTS.
            .chain(TAIL.iter().copied()),
    )
}

/// Raw routine-body cost for ASM `Make_SpritesFromGfxObjects sub_1492`.
pub fn make_sprites_from_gfx_objects_raw_cycles_sub_1492(
    work: &ModeLevelGfxSpriteWorkSub1492,
) -> Option<M68kCycleCount> {
    let raw_cycles = visit_make_sprites_from_gfx_objects_instructions_sub_1492(work)?
        .try_fold(0_u32, u32::checked_add)?;
    raw_cycles.checked_sub(18).map(M68kCycleCount::from_raw)
}

/// Co-schedule ASM `Mode_Level loc_87E -> sub_1492 -> loc_882` with
/// asynchronous hardware.
pub fn make_sprites_from_gfx_objects_advance_with_scheduler_sub_1492(
    mut phase: M68kRefreshPhase,
    work: &ModeLevelGfxSpriteWorkSub1492,
    mut advance_hardware: impl FnMut(M68kCycleCount) -> Option<M68kCycleCount>,
) -> Option<M68kRefreshAdvance> {
    let mut elapsed = 0_u32;
    let mut refresh_events = 0_u32;

    for instruction_cycles in visit_make_sprites_from_gfx_objects_instructions_sub_1492(work)? {
        let phase_after_raw = u32::from(phase.get()).checked_add(instruction_cycles)?;
        let refresh_wait = u32::from(phase_after_raw >= 128) * 2;
        phase = M68kRefreshPhase::new(
            u8::try_from(phase_after_raw % 128).expect("refresh phase remains below 128"),
        )
        .expect("refresh phase remains below 128");
        elapsed = elapsed.checked_add(instruction_cycles)?;
        let generated_wait = advance_hardware(M68kCycleCount::from_raw(instruction_cycles))?;
        let mut pending_wait = generated_wait.get().checked_add(refresh_wait)?;
        refresh_events = refresh_events.checked_add(refresh_wait / 2)?;
        while pending_wait != 0 {
            let wait = M68kCycleCount::from_raw(pending_wait);
            phase = phase.advance_external_wait(wait).phase_after;
            elapsed = elapsed.checked_add(pending_wait)?;
            pending_wait = advance_hardware(wait)?.get();
        }
    }

    Some(M68kRefreshAdvance {
        elapsed: M68kCycleCount::from_raw(elapsed),
        phase_after: phase,
        refresh_events,
    })
}

/// Co-schedule ASM `sub_8E0` entry through the first blocking
/// `WaitForVint $93E` poll.
pub fn transform_sub_8e0_advance_to_wait_for_vint_poll_with_scheduler(
    mut phase: M68kRefreshPhase,
    work: &TransformRasterWorkSub8E0,
    mut advance_hardware: impl FnMut(M68kCycleCount) -> Option<M68kCycleCount>,
) -> Option<M68kRefreshAdvance> {
    let mut elapsed = 0_u32;
    let mut refresh_events = 0_u32;

    macro_rules! apply_advance {
        ($advance:expr) => {{
            let advance = $advance?;
            elapsed = elapsed.checked_add(advance.elapsed.get())?;
            refresh_events = refresh_events.checked_add(advance.refresh_events)?;
            phase = advance.phase_after;
        }};
    }

    apply_advance!(terrain_effects_advance_with_scheduler_sub_5e02(
        phase,
        work.murder_wall_active_sub_5e02,
        &mut advance_hardware,
    ));
    apply_advance!(
        make_sprites_from_gfx_objects_advance_with_scheduler_sub_1492(
            phase,
            &work.gfx_sprite_work_sub_1492,
            &mut advance_hardware,
        )
    );
    apply_advance!(make_sprites_from_platforms_advance_with_scheduler_sub_2444(
        phase,
        work.platform_work_sub_2444,
        &mut advance_hardware,
    ));

    // ASM sub_8E0 $8EC BSR.W, then WaitForVint $93A clears the VINT latch
    // and falls through to the persistent $93E poll.
    for raw_cycles in [18, 16] {
        let advance = phase.advance_rom(M68kCycleCount::from_raw(raw_cycles));
        let generated_wait = advance_hardware(M68kCycleCount::from_raw(raw_cycles))?;
        elapsed = elapsed.checked_add(advance.elapsed.get())?;
        refresh_events = refresh_events.checked_add(advance.refresh_events)?;
        phase = advance.phase_after;
        let mut pending_wait = generated_wait;
        while pending_wait.get() != 0 {
            let wait = phase.advance_external_wait(pending_wait);
            elapsed = elapsed.checked_add(wait.elapsed.get())?;
            phase = wait.phase_after;
            pending_wait = advance_hardware(pending_wait)?;
        }
    }

    Some(M68kRefreshAdvance {
        elapsed: M68kCycleCount::from_raw(elapsed),
        phase_after: phase,
        refresh_events,
    })
}

fn visit_terrain_special_scan_instructions_sub_1fa2(
    work: ModeLevelTerrainSpecialScanSub1FA2,
    mut visit: impl FnMut(u32) -> Option<()>,
) -> Option<()> {
    if work.first_special_collision.is_some() {
        return None;
    }
    let rows = work
        .bounds
        .bottom_row
        .checked_sub(work.bounds.top_row)?
        .checked_add(1)?;
    let columns = work
        .bounds
        .right_col
        .checked_sub(work.bounds.left_col)?
        .checked_add(1)?;
    if work.tile_iterations != rows.checked_mul(columns)? {
        return None;
    }

    // ASM Mode_Level loc_876 BSR and sub_1FA2 setup through loc_1FEC.
    for cycles in [
        18, 16, 16, 16, 16, 12, 12, 16, 12, 12, 12, 14, 14, 4, 4, 12, 14, 12, 12, 14, 14, 4, 4, 8,
    ] {
        visit(cycles)?;
    }
    for row in 0..rows {
        // ASM loc_1FEE: reset the inclusive horizontal DBF counter and row
        // pointer before scanning each terrain row.
        for cycles in [4, 4] {
            visit(cycles)?;
        }
        for column in 0..columns {
            // ASM loc_1FF2: load and mask one collision word. Clear cells
            // reject the special dispatch and continue through the DBF.
            for cycles in [8, 8, 8, 12] {
                visit(cycles)?;
            }
            visit(if column + 1 == columns { 14 } else { 10 })?;
        }
        // ASM loc_2004/loc_2008: advance one layout row and continue the
        // inclusive vertical DBF.
        visit(16)?;
        visit(if row + 1 == rows { 14 } else { 10 })?;
    }
    // ASM loc_200C: discard the saved Kid position and return to loc_87A.
    for cycles in [8, 16] {
        visit(cycles)?;
    }
    Some(())
}

/// Raw routine-body cost for ASM special-terrain scan `sub_1FA2`.
pub fn terrain_special_scan_raw_cycles_sub_1fa2(
    work: ModeLevelTerrainSpecialScanSub1FA2,
) -> Option<M68kCycleCount> {
    let mut raw_cycles = 0_u32;
    visit_terrain_special_scan_instructions_sub_1fa2(work, |instruction_cycles| {
        raw_cycles = raw_cycles.checked_add(instruction_cycles)?;
        Some(())
    })?;
    raw_cycles.checked_sub(18).map(M68kCycleCount::from_raw)
}

/// Co-schedule ASM `Mode_Level loc_876 -> sub_1FA2 -> loc_87A` with
/// asynchronous hardware.
pub fn terrain_special_scan_advance_with_scheduler_sub_1fa2(
    mut phase: M68kRefreshPhase,
    work: ModeLevelTerrainSpecialScanSub1FA2,
    mut advance_hardware: impl FnMut(M68kCycleCount) -> Option<M68kCycleCount>,
) -> Option<M68kRefreshAdvance> {
    let mut elapsed = 0_u32;
    let mut refresh_events = 0_u32;

    visit_terrain_special_scan_instructions_sub_1fa2(work, |instruction_cycles| {
        let phase_after_raw = u32::from(phase.get()).checked_add(instruction_cycles)?;
        let refresh_wait = u32::from(phase_after_raw >= 128) * 2;
        phase = M68kRefreshPhase::new(
            u8::try_from(phase_after_raw % 128).expect("refresh phase remains below 128"),
        )
        .expect("refresh phase remains below 128");
        elapsed = elapsed.checked_add(instruction_cycles)?;
        let generated_wait = advance_hardware(M68kCycleCount::from_raw(instruction_cycles))?;
        let mut pending_wait = generated_wait.get().checked_add(refresh_wait)?;
        refresh_events = refresh_events.checked_add(refresh_wait / 2)?;
        while pending_wait != 0 {
            let wait = M68kCycleCount::from_raw(pending_wait);
            phase = phase.advance_external_wait(wait).phase_after;
            elapsed = elapsed.checked_add(pending_wait)?;
            pending_wait = advance_hardware(wait)?.get();
        }
        Some(())
    })?;

    Some(M68kRefreshAdvance {
        elapsed: M68kCycleCount::from_raw(elapsed),
        phase_after: phase,
        refresh_events,
    })
}

fn visit_shooter_support_collision_instructions_sub_1f52(
    work: ModeLevelShooterCollisionWork,
    mut visit: impl FnMut(u32) -> Option<()>,
) -> Option<()> {
    if work.active_support_slots != 0 {
        return None;
    }

    // ASM Mode_Level loc_856 BSR, then sub_1F52 loc_1F56: the first
    // support-rectangle word is the null sentinel, so the scan branches
    // directly to return_1F8C.
    for cycles in [18, 8, 8, 8, 10, 16] {
        visit(cycles)?;
    }
    Some(())
}

/// Raw routine-body cost for ASM shooter-support collision scan `sub_1F52`.
pub fn shooter_support_collision_raw_cycles_sub_1f52(
    work: ModeLevelShooterCollisionWork,
) -> Option<M68kCycleCount> {
    let mut raw_cycles = 0_u32;
    visit_shooter_support_collision_instructions_sub_1f52(work, |instruction_cycles| {
        raw_cycles = raw_cycles.checked_add(instruction_cycles)?;
        Some(())
    })?;
    raw_cycles.checked_sub(18).map(M68kCycleCount::from_raw)
}

/// Co-schedule ASM `Mode_Level loc_856 -> sub_1F52 -> loc_85A` with
/// asynchronous hardware.
pub fn shooter_support_collision_advance_with_scheduler_sub_1f52(
    mut phase: M68kRefreshPhase,
    work: ModeLevelShooterCollisionWork,
    mut advance_hardware: impl FnMut(M68kCycleCount) -> Option<M68kCycleCount>,
) -> Option<M68kRefreshAdvance> {
    let mut elapsed = 0_u32;
    let mut refresh_events = 0_u32;

    visit_shooter_support_collision_instructions_sub_1f52(work, |instruction_cycles| {
        let phase_after_raw = u32::from(phase.get()).checked_add(instruction_cycles)?;
        let refresh_wait = u32::from(phase_after_raw >= 128) * 2;
        phase = M68kRefreshPhase::new(
            u8::try_from(phase_after_raw % 128).expect("refresh phase remains below 128"),
        )
        .expect("refresh phase remains below 128");
        elapsed = elapsed.checked_add(instruction_cycles)?;
        let generated_wait = advance_hardware(M68kCycleCount::from_raw(instruction_cycles))?;
        let mut pending_wait = generated_wait.get().checked_add(refresh_wait)?;
        refresh_events = refresh_events.checked_add(refresh_wait / 2)?;
        while pending_wait != 0 {
            let wait = M68kCycleCount::from_raw(pending_wait);
            phase = phase.advance_external_wait(wait).phase_after;
            elapsed = elapsed.checked_add(pending_wait)?;
            pending_wait = advance_hardware(wait)?.get();
        }
        Some(())
    })?;

    Some(M68kRefreshAdvance {
        elapsed: M68kCycleCount::from_raw(elapsed),
        phase_after: phase,
        refresh_events,
    })
}

fn visit_shooter_kid_collision_instructions_sub_219c(
    work: ModeLevelShooterCollisionWork,
    mut visit: impl FnMut(u32) -> Option<()>,
) -> Option<()> {
    if work.active_moving_objects != 0 {
        return None;
    }

    // ASM Mode_Level loc_85A BSR, then sub_219C loc_21B8: save/load the Kid
    // hitbox, observe the null ShooterObject link, restore, and return.
    for cycles in [18, 88, 16, 12, 12, 12, 12, 8, 12, 10, 92, 16] {
        visit(cycles)?;
    }
    Some(())
}

/// Raw routine-body cost for ASM Kid-versus-shooter scan `sub_219C`.
pub fn shooter_kid_collision_raw_cycles_sub_219c(
    work: ModeLevelShooterCollisionWork,
) -> Option<M68kCycleCount> {
    let mut raw_cycles = 0_u32;
    visit_shooter_kid_collision_instructions_sub_219c(work, |instruction_cycles| {
        raw_cycles = raw_cycles.checked_add(instruction_cycles)?;
        Some(())
    })?;
    raw_cycles.checked_sub(18).map(M68kCycleCount::from_raw)
}

/// Co-schedule ASM `Mode_Level loc_85A -> sub_219C -> loc_85E` with
/// asynchronous hardware.
pub fn shooter_kid_collision_advance_with_scheduler_sub_219c(
    mut phase: M68kRefreshPhase,
    work: ModeLevelShooterCollisionWork,
    mut advance_hardware: impl FnMut(M68kCycleCount) -> Option<M68kCycleCount>,
) -> Option<M68kRefreshAdvance> {
    let mut elapsed = 0_u32;
    let mut refresh_events = 0_u32;

    visit_shooter_kid_collision_instructions_sub_219c(work, |instruction_cycles| {
        let phase_after_raw = u32::from(phase.get()).checked_add(instruction_cycles)?;
        let refresh_wait = u32::from(phase_after_raw >= 128) * 2;
        phase = M68kRefreshPhase::new(
            u8::try_from(phase_after_raw % 128).expect("refresh phase remains below 128"),
        )
        .expect("refresh phase remains below 128");
        elapsed = elapsed.checked_add(instruction_cycles)?;
        let generated_wait = advance_hardware(M68kCycleCount::from_raw(instruction_cycles))?;
        let mut pending_wait = generated_wait.get().checked_add(refresh_wait)?;
        refresh_events = refresh_events.checked_add(refresh_wait / 2)?;
        while pending_wait != 0 {
            let wait = M68kCycleCount::from_raw(pending_wait);
            phase = phase.advance_external_wait(wait).phase_after;
            elapsed = elapsed.checked_add(pending_wait)?;
            pending_wait = advance_hardware(wait)?.get();
        }
        Some(())
    })?;

    Some(M68kRefreshAdvance {
        elapsed: M68kCycleCount::from_raw(elapsed),
        phase_after: phase,
        refresh_events,
    })
}

fn visit_shooter_projectile_collision_instructions_sub_226a(
    work: ModeLevelShooterCollisionWork,
    mut visit: impl FnMut(u32) -> Option<()>,
) -> Option<()> {
    if work.active_moving_objects != 0 {
        return None;
    }

    // ASM Mode_Level loc_85E BSR, then sub_226A loc_226E: the first
    // ShooterObject link is the null sentinel, so no Kid-projectile list
    // state is read before return_231A.
    for cycles in [18, 8, 12, 10, 16] {
        visit(cycles)?;
    }
    Some(())
}

/// Raw routine-body cost for ASM shooter-versus-Kid-projectile scan `sub_226A`.
pub fn shooter_projectile_collision_raw_cycles_sub_226a(
    work: ModeLevelShooterCollisionWork,
) -> Option<M68kCycleCount> {
    let mut raw_cycles = 0_u32;
    visit_shooter_projectile_collision_instructions_sub_226a(work, |instruction_cycles| {
        raw_cycles = raw_cycles.checked_add(instruction_cycles)?;
        Some(())
    })?;
    raw_cycles.checked_sub(18).map(M68kCycleCount::from_raw)
}

/// Co-schedule ASM `Mode_Level loc_85E -> sub_226A -> loc_862` with
/// asynchronous hardware.
pub fn shooter_projectile_collision_advance_with_scheduler_sub_226a(
    mut phase: M68kRefreshPhase,
    work: ModeLevelShooterCollisionWork,
    mut advance_hardware: impl FnMut(M68kCycleCount) -> Option<M68kCycleCount>,
) -> Option<M68kRefreshAdvance> {
    let mut elapsed = 0_u32;
    let mut refresh_events = 0_u32;

    visit_shooter_projectile_collision_instructions_sub_226a(work, |instruction_cycles| {
        let phase_after_raw = u32::from(phase.get()).checked_add(instruction_cycles)?;
        let refresh_wait = u32::from(phase_after_raw >= 128) * 2;
        phase = M68kRefreshPhase::new(
            u8::try_from(phase_after_raw % 128).expect("refresh phase remains below 128"),
        )
        .expect("refresh phase remains below 128");
        elapsed = elapsed.checked_add(instruction_cycles)?;
        let generated_wait = advance_hardware(M68kCycleCount::from_raw(instruction_cycles))?;
        let mut pending_wait = generated_wait.get().checked_add(refresh_wait)?;
        refresh_events = refresh_events.checked_add(refresh_wait / 2)?;
        while pending_wait != 0 {
            let wait = M68kCycleCount::from_raw(pending_wait);
            phase = phase.advance_external_wait(wait).phase_after;
            elapsed = elapsed.checked_add(pending_wait)?;
            pending_wait = advance_hardware(wait)?.get();
        }
        Some(())
    })?;

    Some(M68kRefreshAdvance {
        elapsed: M68kCycleCount::from_raw(elapsed),
        phase_after: phase,
        refresh_events,
    })
}

/// Source-ordered raw-cycle spans after ASM `sub_44DC` returns at `$810`
/// through the `loc_892` join point.
///
/// Each entry includes its `Mode_Level` call or inline shell. Refresh and
/// asynchronous GEMS waits remain excluded. The phase granularity preserves
/// the ROM order for a later instruction-boundary timing adapter; it is not
/// itself fine enough to place an asynchronous wait inside a routine body.
pub fn mode_level_post_hud_phase_raw_cycles_to_loc_892(
    ledger: &ModeLevelPostHudWorkLedger,
) -> Option<[M68kCycleCount; 22]> {
    const PHASES: [ModeLevelPostHudPhase; 22] = [
        ModeLevelPostHudPhase::ManageEnemyLoading,
        ModeLevelPostHudPhase::GfxObjectsMoveAndAnimate,
        ModeLevelPostHudPhase::ReadJoypad,
        ModeLevelPostHudPhase::TerrainEffectsSub5e02,
        ModeLevelPostHudPhase::ManagePlatformLoading,
        ModeLevelPostHudPhase::ExecuteScriptedPlatforms,
        ModeLevelPostHudPhase::PlatformsCheckCollision,
        ModeLevelPostHudPhase::GfxObjectsCollisionF86a,
        ModeLevelPostHudPhase::GfxObjectsCollisionKidProjectile,
        ModeLevelPostHudPhase::LatchDamageState,
        ModeLevelPostHudPhase::GfxObjectsCollisionKidProjectileAgainstKid,
        ModeLevelPostHudPhase::CollisionSub1f52,
        ModeLevelPostHudPhase::CollisionSub219c,
        ModeLevelPostHudPhase::CollisionSub226a,
        ModeLevelPostHudPhase::GfxObjectsCollisionKidF86a,
        ModeLevelPostHudPhase::GfxObjectsCollisionKidF86e,
        ModeLevelPostHudPhase::CollisionSub1d76,
        ModeLevelPostHudPhase::CollisionSub1fa2,
        ModeLevelPostHudPhase::PostCollisionSub2a4c,
        ModeLevelPostHudPhase::MakeSpritesFromGfxObjects,
        ModeLevelPostHudPhase::DiamondPowerCompileSprites,
        ModeLevelPostHudPhase::MakeSpritesFromPlatforms,
    ];
    if ledger.phases.as_slice() != PHASES {
        return None;
    }

    let with_shell = |body: M68kCycleCount, shell: u32| {
        body.get().checked_add(shell).map(M68kCycleCount::from_raw)
    };
    let collision = &ledger.gfx_collision_work;
    Some([
        // ASM loc_810: JSR abs.l plus j_Manage_EnemyLoading's JMP (PC).
        with_shell(
            manage_enemy_loading_raw_cycles(ledger.enemy_loading_path)?,
            30,
        )?,
        with_shell(
            gfx_objects_move_and_animate_raw_cycles_sub_1444(
                &ledger.gfx_move_animate_work_sub_1444,
            )?,
            18,
        )?,
        // ASM loc_81A: JSR abs.w plus j_ReadJoypad's JMP (PC).
        with_shell(read_joypad_raw_cycles(ledger.read_joypad_work)?, 28)?,
        with_shell(
            terrain_effects_raw_cycles_sub_5e02(ledger.murder_wall_active_sub_5e02)?,
            18,
        )?,
        with_shell(
            manage_platform_loading_raw_cycles_loc_2744(ledger.platform_loader_work?)?,
            18,
        )?,
        with_shell(
            execute_scripted_platforms_raw_cycles_sub_23ea(ledger.platform_work)?,
            18,
        )?,
        with_shell(
            platforms_check_collision_raw_cycles_sub_2574(ledger.platform_work)?,
            18,
        )?,
        // ASM loc_832/loc_83A: LEA abs.w plus BSR.w.
        with_shell(
            gfx_objects_collision_raw_cycles_sub_16f0(collision.object_kind_two_list_f86a)?,
            26,
        )?,
        with_shell(
            gfx_objects_collision_raw_cycles_sub_16f0(collision.kid_projectile_list_f866)?,
            26,
        )?,
        // ASM loc_83E..loc_84A: two 20-cycle memory-to-memory MOVE.B
        // instructions and the 16-cycle memory Scc.
        M68kCycleCount::from_raw(56),
        with_shell(
            gfx_objects_collision_kid_raw_cycles_sub_1b76(
                collision.kid_projectile_list_f866,
                collision,
            )?,
            26,
        )?,
        with_shell(
            shooter_support_collision_raw_cycles_sub_1f52(ledger.shooter_collision_work)?,
            18,
        )?,
        with_shell(
            shooter_kid_collision_raw_cycles_sub_219c(ledger.shooter_collision_work)?,
            18,
        )?,
        with_shell(
            shooter_projectile_collision_raw_cycles_sub_226a(ledger.shooter_collision_work)?,
            18,
        )?,
        with_shell(
            gfx_objects_collision_kid_raw_cycles_sub_1b76(
                collision.object_kind_two_list_f86a,
                collision,
            )?,
            26,
        )?,
        with_shell(
            gfx_objects_collision_kid_f86e_raw_cycles_sub_1b76(collision)?,
            26,
        )?,
        with_shell(
            kid_projectile_collision_raw_cycles_sub_1d76(
                ledger.kid_projectile_collision_work_sub_1d76,
            )?,
            18,
        )?,
        with_shell(
            terrain_special_scan_raw_cycles_sub_1fa2(ledger.terrain_special_scan_sub_1fa2?)?,
            18,
        )?,
        with_shell(
            block_debris_sprites_raw_cycles_sub_2a4c(ledger.block_debris_work_sub_2a4c)?,
            18,
        )?,
        with_shell(
            make_sprites_from_gfx_objects_raw_cycles_sub_1492(&ledger.gfx_sprite_work_sub_1492)?,
            18,
        )?,
        // ASM loc_882: 12-cycle TST, BNE fall-through, JSR abs.l, and
        // wrapper JMP.
        with_shell(
            diamond_power_compile_sprites_raw_cycles_sub_3f798(
                ledger.diamond_power_compile_work_sub_3f798,
            )?,
            50,
        )?,
        with_shell(
            make_sprites_from_platforms_raw_cycles_sub_2444(ledger.platform_work)?,
            18,
        )?,
    ])
}

/// Total non-GEMS source work from ASM `$810` to `loc_892`.
pub fn mode_level_post_hud_raw_cycles_to_loc_892(
    ledger: &ModeLevelPostHudWorkLedger,
) -> Option<M68kCycleCount> {
    mode_level_post_hud_phase_raw_cycles_to_loc_892(ledger)?
        .into_iter()
        .try_fold(0_u32, |total, cycles| total.checked_add(cycles.get()))
        .map(M68kCycleCount::from_raw)
}

/// Co-schedule all 22 ASM `Mode_Level loc_810 -> loc_892` phases.
pub fn mode_level_post_hud_advance_to_loc_892_with_scheduler(
    phase: M68kRefreshPhase,
    ledger: &ModeLevelPostHudWorkLedger,
    advance_hardware: impl FnMut(M68kCycleCount, bool) -> Option<M68kCycleCount>,
    apply_read_joypad_control: impl FnMut(ReadJoypadControl),
) -> Option<M68kRefreshAdvance> {
    mode_level_post_hud_advance_to_loc_892_with_phase_observer(
        phase,
        ledger,
        advance_hardware,
        apply_read_joypad_control,
        |_| {},
    )
}

/// Co-schedule all 22 ASM `Mode_Level loc_810 -> loc_892` phases and observe
/// each typed phase after its instruction and asynchronous-wait work drains.
pub fn mode_level_post_hud_advance_to_loc_892_with_phase_observer(
    mut phase: M68kRefreshPhase,
    ledger: &ModeLevelPostHudWorkLedger,
    mut advance_hardware: impl FnMut(M68kCycleCount, bool) -> Option<M68kCycleCount>,
    mut apply_read_joypad_control: impl FnMut(ReadJoypadControl),
    mut observe_phase: impl FnMut(ModeLevelPostHudPhaseAdvance),
) -> Option<M68kRefreshAdvance> {
    // Validate source order and every typed branch before mutating external
    // hardware through the instruction callbacks below.
    let phase_raw_cycles = mode_level_post_hud_phase_raw_cycles_to_loc_892(ledger)?;

    let mut elapsed = 0_u32;
    let mut refresh_events = 0_u32;
    let mut phase_index = 0_usize;
    let generated_hardware_wait_cycles = std::cell::Cell::new(0_u32);
    let mut traced_hardware = |cycles, source_owns_z80_bus| {
        let generated = advance_hardware(cycles, source_owns_z80_bus)?;
        generated_hardware_wait_cycles.set(
            generated_hardware_wait_cycles
                .get()
                .checked_add(generated.get())?,
        );
        Some(generated)
    };
    macro_rules! apply_phase {
        ($source_phase:expr, $advance:expr) => {{
            generated_hardware_wait_cycles.set(0);
            let phase_before = phase;
            let advance = $advance?;
            let raw_cycles = phase_raw_cycles[phase_index];
            phase_index = phase_index.checked_add(1)?;
            elapsed = elapsed.checked_add(advance.elapsed.get())?;
            refresh_events = refresh_events.checked_add(advance.refresh_events)?;
            phase = advance.phase_after;
            observe_phase(ModeLevelPostHudPhaseAdvance {
                phase: $source_phase,
                phase_before,
                raw_cycles,
                elapsed: advance.elapsed,
                refresh_events: advance.refresh_events,
                phase_after: advance.phase_after,
                generated_hardware_wait_cycles: M68kCycleCount::from_raw(
                    generated_hardware_wait_cycles.get(),
                ),
            });
        }};
    }
    let collision = &ledger.gfx_collision_work;

    {
        let mut ordinary_hardware = |cycles| traced_hardware(cycles, false);
        apply_phase!(
            ModeLevelPostHudPhase::ManageEnemyLoading,
            manage_enemy_loading_advance_with_scheduler_loc_36814(
                phase,
                ledger.enemy_loading_path,
                &mut ordinary_hardware,
            )
        );
        apply_phase!(
            ModeLevelPostHudPhase::GfxObjectsMoveAndAnimate,
            gfx_objects_move_and_animate_advance_with_scheduler_sub_1444(
                phase,
                &ledger.gfx_move_animate_work_sub_1444,
                &mut ordinary_hardware,
            )
        );
    }
    apply_phase!(
        ModeLevelPostHudPhase::ReadJoypad,
        read_joypad_advance_with_scheduler(
            phase,
            ledger.read_joypad_work,
            &mut traced_hardware,
            &mut apply_read_joypad_control,
        )
    );
    let mut ordinary_hardware = |cycles| traced_hardware(cycles, false);
    apply_phase!(
        ModeLevelPostHudPhase::TerrainEffectsSub5e02,
        terrain_effects_advance_with_scheduler_sub_5e02(
            phase,
            ledger.murder_wall_active_sub_5e02,
            &mut ordinary_hardware,
        )
    );
    apply_phase!(
        ModeLevelPostHudPhase::ManagePlatformLoading,
        manage_platform_loading_advance_with_scheduler_loc_2744(
            phase,
            ledger.platform_loader_work?,
            &mut ordinary_hardware,
        )
    );
    apply_phase!(
        ModeLevelPostHudPhase::ExecuteScriptedPlatforms,
        execute_scripted_platforms_advance_with_scheduler_sub_23ea(
            phase,
            ledger.platform_work,
            &mut ordinary_hardware,
        )
    );
    apply_phase!(
        ModeLevelPostHudPhase::PlatformsCheckCollision,
        platforms_check_collision_advance_with_scheduler_sub_2574(
            phase,
            ledger.platform_work,
            &mut ordinary_hardware,
        )
    );
    apply_phase!(
        ModeLevelPostHudPhase::GfxObjectsCollisionF86a,
        gfx_objects_collision_advance_with_scheduler_sub_16f0(
            phase,
            collision.object_kind_two_list_f86a,
            &mut ordinary_hardware,
        )
    );
    apply_phase!(
        ModeLevelPostHudPhase::GfxObjectsCollisionKidProjectile,
        gfx_objects_collision_advance_with_scheduler_sub_16f0(
            phase,
            collision.kid_projectile_list_f866,
            &mut ordinary_hardware,
        )
    );
    apply_phase!(
        ModeLevelPostHudPhase::LatchDamageState,
        mode_level_latch_damage_state_advance_loc_83e(phase, &mut ordinary_hardware)
    );
    apply_phase!(
        ModeLevelPostHudPhase::GfxObjectsCollisionKidProjectileAgainstKid,
        gfx_objects_collision_kid_advance_with_scheduler_sub_1b76(
            phase,
            collision.kid_projectile_list_f866,
            collision,
            &mut ordinary_hardware,
        )
    );
    apply_phase!(
        ModeLevelPostHudPhase::CollisionSub1f52,
        shooter_support_collision_advance_with_scheduler_sub_1f52(
            phase,
            ledger.shooter_collision_work,
            &mut ordinary_hardware,
        )
    );
    apply_phase!(
        ModeLevelPostHudPhase::CollisionSub219c,
        shooter_kid_collision_advance_with_scheduler_sub_219c(
            phase,
            ledger.shooter_collision_work,
            &mut ordinary_hardware,
        )
    );
    apply_phase!(
        ModeLevelPostHudPhase::CollisionSub226a,
        shooter_projectile_collision_advance_with_scheduler_sub_226a(
            phase,
            ledger.shooter_collision_work,
            &mut ordinary_hardware,
        )
    );
    apply_phase!(
        ModeLevelPostHudPhase::GfxObjectsCollisionKidF86a,
        gfx_objects_collision_kid_advance_with_scheduler_sub_1b76(
            phase,
            collision.object_kind_two_list_f86a,
            collision,
            &mut ordinary_hardware,
        )
    );
    apply_phase!(
        ModeLevelPostHudPhase::GfxObjectsCollisionKidF86e,
        gfx_objects_collision_kid_f86e_advance_with_scheduler_sub_1b76(
            phase,
            collision,
            &mut ordinary_hardware,
        )
    );
    apply_phase!(
        ModeLevelPostHudPhase::CollisionSub1d76,
        kid_projectile_collision_advance_with_scheduler_sub_1d76(
            phase,
            ledger.kid_projectile_collision_work_sub_1d76,
            &mut ordinary_hardware,
        )
    );
    apply_phase!(
        ModeLevelPostHudPhase::CollisionSub1fa2,
        terrain_special_scan_advance_with_scheduler_sub_1fa2(
            phase,
            ledger.terrain_special_scan_sub_1fa2?,
            &mut ordinary_hardware,
        )
    );
    apply_phase!(
        ModeLevelPostHudPhase::PostCollisionSub2a4c,
        block_debris_sprites_advance_with_scheduler_sub_2a4c(
            phase,
            ledger.block_debris_work_sub_2a4c,
            &mut ordinary_hardware,
        )
    );
    apply_phase!(
        ModeLevelPostHudPhase::MakeSpritesFromGfxObjects,
        make_sprites_from_gfx_objects_advance_with_scheduler_sub_1492(
            phase,
            &ledger.gfx_sprite_work_sub_1492,
            &mut ordinary_hardware,
        )
    );
    apply_phase!(
        ModeLevelPostHudPhase::DiamondPowerCompileSprites,
        diamond_power_compile_sprites_advance_with_scheduler_sub_3f798(
            phase,
            ledger.diamond_power_compile_work_sub_3f798,
            &mut ordinary_hardware,
        )
    );
    apply_phase!(
        ModeLevelPostHudPhase::MakeSpritesFromPlatforms,
        make_sprites_from_platforms_advance_with_scheduler_sub_2444(
            phase,
            ledger.platform_work,
            &mut ordinary_hardware,
        )
    );

    debug_assert_eq!(phase_index, phase_raw_cycles.len());

    Some(M68kRefreshAdvance {
        elapsed: M68kCycleCount::from_raw(elapsed),
        phase_after: phase,
        refresh_events,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn transform_sub_8e0_composes_empty_lists_to_wait_poll() {
        let work = TransformRasterWorkSub8E0 {
            murder_wall_active_sub_5e02: false,
            gfx_sprite_work_sub_1492: ModeLevelGfxSpriteWorkSub1492::Classified {
                nodes: Vec::new(),
            },
            platform_work_sub_2444: ModeLevelPlatformWork {
                frozen_fa64: false,
                active_object_count: 0,
            },
        };
        let mut hardware_interval_cycles = 0_u32;
        let advance = transform_sub_8e0_advance_to_wait_for_vint_poll_with_scheduler(
            M68kRefreshPhase::new(84).expect("test sub_8E0 entry phase"),
            &work,
            |cycles| {
                hardware_interval_cycles += cycles.get();
                Some(M68kCycleCount::from_raw(0))
            },
        )
        .expect("empty-list sub_8E0 pre-VINT path");

        assert_eq!(hardware_interval_cycles, 352);
        assert_eq!(advance.elapsed, M68kCycleCount::from_raw(352));
        assert_eq!(advance.refresh_events, 3);
        assert_eq!(
            advance.phase_after,
            M68kRefreshPhase::new(52).expect("empty-list $93E phase"),
        );
    }

    #[test]
    fn transform_sub_8e0_reaches_frame_1058_wait_poll() {
        let work = TransformRasterWorkSub8E0 {
            murder_wall_active_sub_5e02: false,
            gfx_sprite_work_sub_1492: ModeLevelGfxSpriteWorkSub1492::Classified {
                nodes: vec![
                    ModeLevelGfxSpriteNodeWorkSub1492::HiddenFlagClear,
                    ModeLevelGfxSpriteNodeWorkSub1492::VisibleSingleDynamicArt20x32Flipped,
                    ModeLevelGfxSpriteNodeWorkSub1492::VerticallyCulledAboveViewport,
                    ModeLevelGfxSpriteNodeWorkSub1492::VerticallyCulledAboveViewport,
                ],
            },
            platform_work_sub_2444: ModeLevelPlatformWork {
                frozen_fa64: false,
                active_object_count: 0,
            },
        };
        let advance = transform_sub_8e0_advance_to_wait_for_vint_poll_with_scheduler(
            M68kRefreshPhase::new(84).expect("oracle sub_8E0 entry phase"),
            &work,
            |_| Some(M68kCycleCount::from_raw(0)),
        )
        .expect("published Cyclone sub_8E0 pre-VINT path");

        assert_eq!(advance.elapsed, M68kCycleCount::from_raw(2_128));
        assert_eq!(advance.refresh_events, 17);
        assert_eq!(
            advance.phase_after,
            M68kRefreshPhase::new(36).expect("oracle frame-1058 $93E phase"),
        );
    }

    #[test]
    fn cyclone_pickup_entry_matches_frame_1058_refresh_recurrence() {
        let timing = crate::intro_timing::visit_cyclone_pickup_transform_entry_sub_e12(
            CyclonePickupTransformEntrySubE12,
            |_| {},
        );
        let advance = M68kRefreshPhase::new(104)
            .expect("oracle frame-1058 sub_E12 phase")
            .advance_rom(timing.raw_cycles);

        assert_eq!(advance.elapsed, M68kCycleCount::from_raw(6_380));
        assert_eq!(advance.refresh_events, 50);
        assert_eq!(
            advance.phase_after,
            M68kRefreshPhase::new(84).expect("oracle frame-1058 sub_8E0 phase"),
        );
    }

    #[test]
    fn first_level_start_execute_objects_matches_frame_902_instruction_trace() {
        let execute = ModeLevelExecuteObjectsWork {
            rng_offset_before_sub_e12: Some(0xFFCC),
            linked_object_count: 5,
            linked_gfx_object_count: 4,
            linked_non_transient_object_count: 5,
            player: crate::ModeLevelPlayerWorkBeforeHud::Activation,
            prize_block_wobble_workers_sub_1007a: 0,
            prize_block_worker_tails_sub_1007a: 0,
            prize_block_flash_workers_loc_102b8: 0,
            floating_prize_workers: 0,
            collected_diamond_workers: Vec::new(),
            rubber_block_skin_workers_sub_10de4: 0,
            active_enemy_workers: 0,
            active_platform_workers: 0,
            palette_rotation_worker_loc_11f96: 0,
            unclassified_runtime_transient_workers: 0,
            block_destroy_workers_sub_10eda: Vec::new(),
            block_debris_workers_sub_292e: 0,
            player_entry_position: Some(crate::ModeLevelPlayerPosition {
                x_pos: 0x0020_0000,
                y_pos: 0x02DD_0000,
            }),
            player_gfx_terrain_scan: None,
            cyclone_visual_accumulator_before: Some(0),
            prize_block_wobble_paths_sub_1007a: Vec::new(),
            player_spawned_prize_block_initial_sub_1007a: None,
            player_prize_wobble_list_work_loc_1002e: None,
            player_spawned_block_destroy_list_work_sub_10e86: None,
            prize_object_execution_order_sub_e12: Some(vec![
                ModeLevelPrizeObjectExecutionSubE12::Player,
            ]),
            damage_flash_wait_loc_bc44: Some(
                ModeLevelDamageFlashWaitLocBc44::InitialHibernateLocBc34,
            ),
            demo_exit_wait_loc_119e0: Some(ModeLevelDemoExitWaitLoc119E0::InitialHibernateLoc119D8),
            input_worker_loc_7452: Some(ModeLevelInputWorkerPathLoc7452::InitialHibernate),
            palette_fade_worker_sub_73d0: Some(
                crate::ModeLevelPaletteFadeWorkerPathSub73D0::InitialCopyAndHibernate,
            ),
            prize_block_tail_paths_sub_1066: Vec::new(),
            prize_block_flash_paths_sub_1066: Vec::new(),
            floating_prize_paths: Vec::new(),
            executed_player_path: Some(ModeLevelPlayerExecutionPath::LevelStartActivationLoc7508 {
                form: crate::PlayerForm::TheKid,
            }),
        };

        assert_eq!(
            execute_objects_raw_cycles_sub_e12(&execute),
            Some(M68kCycleCount::from_raw(10_248)),
            "ASM sub_E12 -> sub_21F8: frame 902 executes input initial hibernate, Kid loc_7508, damage initial hibernate, sub_73D0, and demo initial hibernate",
        );
        assert_eq!(
            execute_objects_external_waits_sub_e12(&execute),
            Some(vec![M68kExternalWaitEvent {
                raw_offset: M68kCycleCount::from_raw(10_248),
                cycles: M68kCycleCount::from_raw(1),
            }]),
            "the final loc_E1E return carries one source-positioned GEMS bank wait",
        );
        assert_eq!(
            execute_objects_elapsed_cycles_sub_e12(
                &execute,
                M68kRefreshPhase::new(106).expect("oracle sub_E12 phase"),
            ),
            Some(M68kRefreshAdvance {
                elapsed: M68kCycleCount::from_raw(10_413),
                phase_after: M68kRefreshPhase::new(23).expect("oracle sub_21F8 phase"),
                refresh_events: 82,
            }),
            "frame-902 selected PCs close the first Execute_Objects interval",
        );

        let mut ledger = ModeLevelPreHudWorkLedger {
            phases: Vec::new(),
            execute_objects: execute,
            post_execute: crate::ModeLevelPostExecuteWork {
                active_evanescent_slots_before_sub_dfb0: 0,
                evanescent_scan_loc_eae4: Some(crate::ModeLevelEvanescentScanWorkLocEae4 {
                    row_count: 3,
                    col_count: 2,
                    flagged_non_evanescent_cells_before_exit: 0,
                    found_evanescent_block: false,
                }),
                active_shooter_slots_before_sub_f7e0: 0,
                level_special_effects_sub_6034: 0,
                shooter_scan_sub_f096: Some(crate::ModeLevelShooterScanWorkSubF096 {
                    cached_col_matches: false,
                    cached_row_matches: false,
                    active_shooter_slots_before: 0,
                    cached_col_before: -1,
                    cached_row_before: -1,
                    kid_block_col: 2,
                    kid_block_row: 45,
                    level_width_blocks: 64,
                    level_height_blocks: 64,
                    new_shooter_slots_registered: 0,
                }),
            },
        };
        assert_eq!(
            post_execute_raw_cycles_before_sub_44dc(&ledger),
            Some(M68kCycleCount::from_raw(3_138)),
            "ASM sub_21F8 -> sub_44DC: both initial sub_F096 coordinate scans add 550 raw cycles",
        );
        assert_eq!(
            post_execute_elapsed_cycles_before_sub_44dc(
                &ledger,
                M68kRefreshPhase::new(23).expect("oracle sub_21F8 phase"),
            ),
            Some(M68kRefreshAdvance {
                elapsed: M68kCycleCount::from_raw(3_189),
                phase_after: M68kRefreshPhase::new(12).expect("oracle sub_44DC phase"),
                refresh_events: 25,
            }),
            "frame-902 selected PCs close the post-object interval",
        );

        ledger.post_execute.evanescent_scan_loc_eae4 =
            Some(crate::ModeLevelEvanescentScanWorkLocEae4 {
                row_count: 3,
                col_count: 2,
                flagged_non_evanescent_cells_before_exit: 1,
                found_evanescent_block: false,
            });
        ledger.post_execute.shooter_scan_sub_f096 = Some(crate::ModeLevelShooterScanWorkSubF096 {
            cached_col_matches: true,
            cached_row_matches: true,
            active_shooter_slots_before: 0,
            cached_col_before: 3,
            cached_row_before: 43,
            kid_block_col: 3,
            kid_block_row: 43,
            level_width_blocks: 120,
            level_height_blocks: 56,
            new_shooter_slots_registered: 0,
        });
        assert_eq!(
            post_execute_raw_cycles_before_sub_44dc(&ledger),
            Some(M68kCycleCount::from_raw(2_618)),
            "ASM loc_EB4C bit-15 non-type-$A rejection adds 30 raw cycles",
        );

        let shooter = ledger
            .post_execute
            .shooter_scan_sub_f096
            .as_mut()
            .expect("typed shooter scan");
        shooter.cached_row_matches = false;
        shooter.cached_row_before = 43;
        shooter.kid_block_row = 42;
        assert_eq!(
            post_execute_raw_cycles_before_sub_44dc(&ledger),
            Some(M68kCycleCount::from_raw(3_146)),
            "ASM loc_F1A8/loc_F1DE frame-940 upward scan visits seven clear cells",
        );

        let shooter = ledger
            .post_execute
            .shooter_scan_sub_f096
            .as_mut()
            .expect("typed shooter scan");
        shooter.cached_col_matches = false;
        shooter.cached_row_matches = true;
        shooter.cached_col_before = 2;
        shooter.kid_block_col = 3;
        shooter.cached_row_before = 43;
        shooter.kid_block_row = 43;
        assert_eq!(
            post_execute_raw_cycles_before_sub_44dc(&ledger),
            Some(M68kCycleCount::from_raw(3_276)),
            "ASM loc_F0D8/loc_F106 frame-939 rightward scan adds 658 raw cycles",
        );

        ledger
            .post_execute
            .evanescent_scan_loc_eae4
            .as_mut()
            .expect("typed evanescent scan")
            .flagged_non_evanescent_cells_before_exit = 0;
        ledger.post_execute.shooter_scan_sub_f096 = Some(crate::ModeLevelShooterScanWorkSubF096 {
            cached_col_matches: true,
            cached_row_matches: false,
            active_shooter_slots_before: 0,
            cached_col_before: 6,
            cached_row_before: 6,
            kid_block_col: 26,
            kid_block_row: 7,
            level_width_blocks: 64,
            level_height_blocks: 56,
            new_shooter_slots_registered: 0,
        });
        assert_eq!(
            post_execute_raw_cycles_before_sub_44dc(&ledger),
            Some(M68kCycleCount::from_raw(3_126)),
            "ASM sub_F096 loc_F186..loc_F25C frame-1706 row-only scan adds 538 raw cycles",
        );
    }

    #[test]
    fn stop_music_source_schedule_tracks_each_z80_bus_window_e14c8() {
        let timing = gems_stop_music_timing(GemsStopMusicBusAckPolls {
            driver_copy_sub_e13da: 1,
            queue_transactions_sub_e143a: [1; 6],
        })
        .expect("ASM Stop_z80/sub_E143A poll at least once");

        assert_eq!(timing.raw_cycles.get(), 218_216);
        assert_eq!(
            timing.bus_windows.map(|window| (
                window.request_complete_raw_offset.get(),
                window.release_complete_raw_offset.get(),
            )),
            [
                (104, 214_784),
                (215_014, 215_184),
                (215_450, 215_620),
                (215_890, 216_244),
                (216_514, 216_868),
                (217_138, 217_492),
                (217_762, 218_116),
            ],
            "ASM sub_E13DA owns the bus for the driver copy, then each sub_E149C/sub_E14AC call reacquires it separately"
        );
        assert_eq!(
            timing
                .bus_windows
                .map(|window| window.m68k_z80_ram_accesses),
            [8_192, 3, 3, 7, 7, 7, 7],
            "ASM loc_E1400/loc_E1406 fill 8 KiB, then sub_E143A/sub_E148E read the queue pointer and write each byte plus its updated pointer"
        );
        assert_eq!(
            (
                timing.reset_assert_complete_raw_offset.get(),
                timing.reset_release_complete_raw_offset.get(),
            ),
            (214_484, 214_804),
            "ASM sub_E1416 resets the Z80 before releasing the copy window"
        );
    }

    #[test]
    fn stop_music_bus_ack_loops_extend_only_the_current_and_later_windows() {
        let timing = gems_stop_music_timing(GemsStopMusicBusAckPolls {
            driver_copy_sub_e13da: 2,
            queue_transactions_sub_e143a: [1, 3, 1, 1, 1, 1],
        })
        .expect("positive poll counts");

        assert_eq!(timing.raw_cycles.get(), 218_306);
        assert_eq!(
            (
                timing.bus_windows[0].request_complete_raw_offset.get(),
                timing.bus_windows[0].release_complete_raw_offset.get(),
                timing.bus_windows[1].request_complete_raw_offset.get(),
                timing.bus_windows[1].release_complete_raw_offset.get(),
                timing.bus_windows[2].request_complete_raw_offset.get(),
                timing.bus_windows[2].release_complete_raw_offset.get(),
                timing.bus_windows[3].request_complete_raw_offset.get(),
            ),
            (104, 214_814, 215_044, 215_214, 215_480, 215_710, 215_980,),
            "ASM Stop_z80/sub_E143A bus-poll repeats add 30 raw cycles without moving the already-completed bus request"
        );
        assert_eq!(
            (
                timing.reset_assert_complete_raw_offset.get(),
                timing.reset_release_complete_raw_offset.get(),
            ),
            (214_514, 214_834)
        );
        assert_eq!(
            gems_stop_music_timing(GemsStopMusicBusAckPolls {
                driver_copy_sub_e13da: 0,
                queue_transactions_sub_e143a: [1; 6],
            }),
            None,
            "ASM Stop_z80 always samples bus acknowledgement at least once"
        );
    }

    #[test]
    fn stop_music_elapsed_cost_keeps_vint_and_refresh_arbitration_explicit_e14c8() {
        let timing = gems_stop_music_timing(GemsStopMusicBusAckPolls {
            driver_copy_sub_e13da: 1,
            queue_transactions_sub_e143a: [1; 6],
        })
        .expect("one-poll oracle path");

        assert_eq!(
            gems_stop_music_elapsed_cycles(
                timing,
                GemsStopMusicArbitration {
                    vint_interrupts: 1,
                    refresh_wait_events: 1_507,
                },
            )
            .expect("bounded elapsed timing")
            .get(),
            229_570,
            "the frame-867 trace is pure StopMusic plus 8,226 Z80-RAM waits, V_Int entry/handler, and 1,507 refresh waits"
        );
    }

    #[test]
    fn stop_music_event_schedule_injects_vint_without_moving_source_offsets_e14c8() {
        let elapsed = gems_stop_music_elapsed_timing(
            GemsStopMusicBusAckPolls {
                driver_copy_sub_e13da: 1,
                queue_transactions_sub_e143a: [1; 6],
            },
            M68kRefreshPhase::new(14).expect("trace entry phase"),
            &[M68kCycleCount::from_raw(108_848)],
        )
        .expect("frame-867 VINT falls on a StopMusic instruction boundary");

        assert_eq!(elapsed.elapsed_cycles.get(), 229_570);
        assert_eq!(elapsed.refresh_wait_events, 1_507);
        assert_eq!(elapsed.phase_after.get(), 80);
        assert_eq!(
            elapsed.bus_windows.map(|window| (
                window.request_complete_elapsed_offset.get(),
                window.release_complete_elapsed_offset.get(),
            )),
            [
                (104, 226_048),
                (226_282, 226_459),
                (226_729, 226_904),
                (227_178, 227_545),
                (227_819, 228_186),
                (228_460, 228_827),
                (229_101, 229_468),
            ],
            "ASM sub_E13DA/sub_E143A windows retain VINT, refresh, and Z80-RAM waits on the elapsed timeline"
        );
        assert_eq!(
            (
                elapsed.reset_assert_complete_elapsed_offset.get(),
                elapsed.reset_release_complete_elapsed_offset.get(),
            ),
            (225_744, 226_070),
            "ASM sub_E1416 reset edges bracket the driver-bus release"
        );
        assert!(
            gems_stop_music_elapsed_timing(
                GemsStopMusicBusAckPolls {
                    driver_copy_sub_e13da: 1,
                    queue_transactions_sub_e143a: [1; 6],
                },
                M68kRefreshPhase::new(14).expect("trace entry phase"),
                &[M68kCycleCount::from_raw(108_849)],
            )
            .is_none(),
            "VINT must be injected at a real StopMusic instruction boundary"
        );
    }

    #[test]
    fn stop_music_absolute_entry_derives_frame_867_vint_boundary_e14c8() {
        let elapsed = gems_stop_music_elapsed_timing_from_absolute_entry(
            GemsStopMusicBusAckPolls {
                driver_copy_sub_e13da: 1,
                queue_transactions_sub_e143a: [1; 6],
            },
            M68kRefreshPhase::new(14).expect("trace entry phase"),
            M68kCycleCount::from_raw(110_975_266),
        )
        .expect("absolute frame-867 entry reaches VINT on a source boundary");

        assert_eq!(elapsed.elapsed_cycles.get(), 229_570);
        assert_eq!(elapsed.refresh_wait_events, 1_507);
        assert_eq!(elapsed.phase_after.get(), 80);
        assert_eq!(
            elapsed.bus_windows.map(|window| (
                window.request_complete_elapsed_offset.get(),
                window.release_complete_elapsed_offset.get(),
            )),
            [
                (104, 226_048),
                (226_282, 226_459),
                (226_729, 226_904),
                (227_178, 227_545),
                (227_819, 228_186),
                (228_460, 228_827),
                (229_101, 229_468),
            ],
            "absolute NTSC VINT cadence must reproduce the selected-PC frame-867 schedule"
        );
    }

    #[test]
    fn stop_music_absolute_entry_derives_two_vints_for_frame_30_call_e14c8() {
        let elapsed = gems_stop_music_elapsed_timing_from_absolute_entry(
            GemsStopMusicBusAckPolls {
                driver_copy_sub_e13da: 1,
                queue_transactions_sub_e143a: [1; 6],
            },
            M68kRefreshPhase::new(72).expect("selected-PC refresh counter"),
            M68kCycleCount::from_raw(3_868_666),
        )
        .expect("frame-30 call crosses two accepted scanline-224 assertions");

        assert_eq!(elapsed.elapsed_cycles.get(), 229_682);
        assert_eq!(elapsed.refresh_wait_events, 1_506);
        assert_eq!(elapsed.phase_after.get(), 122);
    }

    #[test]
    fn power_on_stop_music_releases_reset_with_masked_vints_e14c8() {
        // ASM StopMusic -> sub_E13DA releases power-on Z80 reset before
        // Stop_z80. Frame 14 has SR=$2700, so both crossed VINT assertions are
        // masked rather than injected into the source schedule.
        let elapsed = gems_stop_music_elapsed_timing(
            GemsStopMusicBusAckPolls {
                driver_copy_sub_e13da: 1,
                queue_transactions_sub_e143a: [1; 6],
            },
            M68kRefreshPhase::new(46).expect("frame-14 selected-PC phase"),
            &[],
        )
        .expect("power-on StopMusic schedule");

        assert_eq!(elapsed.elapsed_cycles.get(), 229_454);
        assert_eq!(
            elapsed.driver_reset_release_complete_elapsed_offset.get(),
            66,
            "ASM sub_E13DA reset-release write completes before Stop_z80 BUSREQ"
        );
        assert_eq!(
            (
                elapsed.driver_copy_bytes_visible(M68kCycleCount::from_raw(19_115)),
                elapsed.driver_clear_bytes_visible(M68kCycleCount::from_raw(19_115)),
            ),
            (813, 0),
            "oracle frame 14 observes ASM loc_E1400's write when its instruction starts"
        );
        assert!(
            elapsed.driver_ram_access_wait_pending(M68kCycleCount::from_raw(19_115)),
            "oracle frame 14 retains the active Z80-RAM access wait"
        );
        assert_eq!(
            (
                elapsed.driver_copy_bytes_visible(M68kCycleCount::from_raw(147_121)),
                elapsed.driver_clear_bytes_visible(M68kCycleCount::from_raw(147_121)),
            ),
            (5_764, 340),
            "oracle frame 15 samples after ASM loc_E1406's 340th clear"
        );
        assert!(
            !elapsed.driver_ram_access_wait_pending(M68kCycleCount::from_raw(147_121)),
            "oracle frame 15 samples after its preceding Z80-RAM access wait"
        );
    }

    #[test]
    fn gems_start_sequence_scheduler_matches_frame_899_selected_pcs_sub_e151c() {
        let mut controls = Vec::new();
        let advance = gems_queue_command_advance_with_scheduler(
            GemsQueueCommandPath::StartOrStopSequenceSubE151c,
            M68kRefreshPhase::new(28).expect("selected-PC refresh phase"),
            |_, _| Some(M68kCycleCount::from_raw(0)),
            |control| controls.push(control),
        )
        .expect("ASM sub_E151C one-poll path");

        assert_eq!(
            (
                advance.queue_bus_request_complete.get(),
                advance.queue_write_complete.get(),
                advance.queue_bus_release_complete.get(),
                advance.elapsed.get(),
                advance.phase_after.get(),
            ),
            (162, 424, 444, 518, 34),
            "selected PCs E151C/E1460/E1470/E1478 and final E1480 RTS"
        );
        assert_eq!(
            controls,
            vec![
                GemsQueueCommandControl::BeginQueueBus,
                GemsQueueCommandControl::WriteQueueBytes,
                GemsQueueCommandControl::ReleaseQueueBus,
            ]
        );
    }

    #[test]
    fn player_ceiling_prize_sound_schedules_divu_and_gems_accesses_once() {
        let waits = [4424, 4484, 4510, 4530, 4594, 4614].map(|raw_offset| M68kExternalWaitEvent {
            raw_offset: M68kCycleCount::from_raw(raw_offset),
            cycles: M68kCycleCount::from_raw(1),
        });
        let advance = player_callback_with_play_sound_advance_sub_e12(
            M68kRefreshPhase::new(2).expect("frame-1697 player entry phase"),
            M68kCycleCount::from_raw(9_908),
            &waits,
        )
        .expect("ASM sub_FACE -> PlaySound sub_E1510 player callback");

        assert_eq!(
            advance,
            M68kRefreshAdvance {
                elapsed: M68kCycleCount::from_raw(10_068),
                phase_after: M68kRefreshPhase::new(86).expect("frame-1697 next loc_E1E phase"),
                refresh_events: 77,
            },
            "ASM sub_FACE DIVU is one instruction; sub_E151C interleaves six direct Z80-RAM accesses",
        );
    }

    #[test]
    fn gems_start_sequence_scheduler_merges_pre_bus_external_wait_sub_e151c() {
        let mut inject_wait = true;
        let advance = gems_queue_command_advance_with_scheduler(
            GemsQueueCommandPath::StartOrStopSequenceSubE151c,
            M68kRefreshPhase::new(82).expect("frame-1669 selected-PC phase"),
            |_, bus_owned| {
                if inject_wait && !bus_owned {
                    inject_wait = false;
                    Some(M68kCycleCount::from_raw(20))
                } else {
                    Some(M68kCycleCount::from_raw(0))
                }
            },
            |_| {},
        )
        .expect("external scheduler work delays BUSREQ");

        assert_eq!(advance.queue_bus_request_complete.get(), 184);
        assert_eq!(advance.queue_write_complete.get(), 446);
    }

    #[test]
    fn gems_music_speed_schedulers_match_frame_1128_selected_pcs() {
        let prefix = gems_queue_command_advance_with_scheduler(
            GemsQueueCommandPath::MusicSpeedPrefixSubE1546,
            M68kRefreshPhase::new(98).expect("sub_E1546 selected-PC phase"),
            |_, _| Some(M68kCycleCount::from_raw(0)),
            |_| {},
        )
        .expect("ASM sub_E1546/sub_E1552 path");
        let commit = gems_queue_command_advance_with_scheduler(
            GemsQueueCommandPath::MusicSpeedCommitSubE1568,
            M68kRefreshPhase::new(100).expect("sub_E1568 selected-PC phase"),
            |_, _| Some(M68kCycleCount::from_raw(0)),
            |_| {},
        )
        .expect("ASM sub_E1568 path");

        assert_eq!(
            (
                prefix.queue_bus_request_complete.get(),
                prefix.queue_write_complete.get(),
                prefix.queue_bus_release_complete.get(),
                prefix.elapsed.get(),
            ),
            (206, 478, 498, 612),
            "selected PCs E1546/E1460/E1470/E1478 and wrapper return"
        );
        assert_eq!(
            (
                commit.queue_bus_request_complete.get(),
                commit.queue_write_complete.get(),
                commit.queue_bus_release_complete.get(),
                commit.elapsed.get(),
            ),
            (164, 338, 358, 432),
            "selected PCs E1568/E1460/E1470/E1478 and final RTS"
        );
    }

    #[test]
    fn gems_open_close_scheduler_matches_load_ingame_sub_e1576() {
        let mut controls = Vec::new();
        let advance = gems_queue_command_advance_with_scheduler(
            GemsQueueCommandPath::OpenCloseSubE1576,
            M68kRefreshPhase::new(4).expect("frame-868 selected-PC phase"),
            |_, _| Some(M68kCycleCount::from_raw(0)),
            |control| controls.push(control),
        )
        .expect("ASM sub_E133C/loc_E1576 path");

        assert_eq!(
            (
                advance.queue_bus_request_complete.get(),
                advance.queue_write_complete.get(),
                advance.queue_bus_release_complete.get(),
                advance.elapsed.get(),
                advance.phase_after.get(),
            ),
            (162, 229, 249, 323, 71),
            "selected PCs E1576/E1460/E1470/E1478 and final E1480 RTS"
        );
        assert_eq!(
            controls,
            vec![
                GemsQueueCommandControl::BeginQueueBus,
                GemsQueueCommandControl::ReleaseQueueBus,
            ],
            "ASM loc_E1576 does not write queue bytes"
        );
    }

    fn published_natural_vblank_work() -> ModeLevelVblankWorkBeforeSubE12 {
        ModeLevelVblankWorkBeforeSubE12 {
            dynamic_art_tiles_loc_ad4: Some([16, 4, 0, 0, 0, 0]),
            palette_dma_sub_970: true,
            scroll_path: ModeLevelScrollTransferPath::NormalLoc300ba,
            foreground_custom_queue_entries_sub_1596: 0,
            foreground_cell_queue_sub_1596: ModeLevelForegroundCellQueueWorkSub1596::EMPTY,
            paused_sub_6874: false,
            hud_path_sub_bdc0: ModeLevelHudBuildPath::LocBe20DirectInitialCounterUpdates,
        }
    }

    #[test]
    fn empty_shooter_collision_lists_take_null_sentinel_paths() {
        let work = ModeLevelShooterCollisionWork {
            active_support_slots: 0,
            active_moving_objects: 0,
        };

        assert_eq!(
            shooter_support_collision_raw_cycles_sub_1f52(work),
            Some(M68kCycleCount::from_raw(50))
        );
        assert_eq!(
            shooter_kid_collision_raw_cycles_sub_219c(work),
            Some(M68kCycleCount::from_raw(290))
        );
        assert_eq!(
            shooter_projectile_collision_raw_cycles_sub_226a(work),
            Some(M68kCycleCount::from_raw(46))
        );

        let support = shooter_support_collision_advance_with_scheduler_sub_1f52(
            M68kRefreshPhase::new(92).expect("frame-1697 loc_856 phase"),
            work,
            |_| Some(M68kCycleCount::from_raw(0)),
        )
        .expect("published empty support rectangle list");
        assert_eq!(support.elapsed, M68kCycleCount::from_raw(70));
        assert_eq!(support.refresh_events, 1);
        assert_eq!(
            support.phase_after,
            M68kRefreshPhase::new(34).expect("frame-1697 loc_85A phase"),
        );

        let kid = shooter_kid_collision_advance_with_scheduler_sub_219c(
            support.phase_after,
            work,
            |_| Some(M68kCycleCount::from_raw(0)),
        )
        .expect("published empty shooter-object list");
        assert_eq!(kid.elapsed, M68kCycleCount::from_raw(312));
        assert_eq!(kid.refresh_events, 2);
        assert_eq!(
            kid.phase_after,
            M68kRefreshPhase::new(90).expect("frame-1697 loc_85E phase"),
        );

        let projectile = shooter_projectile_collision_advance_with_scheduler_sub_226a(
            kid.phase_after,
            work,
            |_| Some(M68kCycleCount::from_raw(0)),
        )
        .expect("published empty shooter-versus-projectile list");
        assert_eq!(projectile.elapsed, M68kCycleCount::from_raw(66));
        assert_eq!(projectile.refresh_events, 1);
        assert_eq!(
            projectile.phase_after,
            M68kRefreshPhase::new(28).expect("frame-1697 loc_862 phase"),
            "all three shooter collision phases reach their oracle boundaries",
        );
        let object_kind_two_vs_kid = gfx_objects_collision_kid_advance_with_scheduler_sub_1b76(
            projectile.phase_after,
            ModeLevelGfxCollisionListWork::Empty,
            &ModeLevelGfxCollisionWork {
                kid_projectile_list_f866: ModeLevelGfxCollisionListWork::Empty,
                object_kind_two_list_f86a: ModeLevelGfxCollisionListWork::Empty,
                object_kind_four_list_f86e: ModeLevelGfxCollisionKidListWorkSub1B76::Unclassified,
                kid_is_invulnerable: false,
            },
            |_| Some(M68kCycleCount::from_raw(0)),
        )
        .expect("published empty object-kind-two versus Kid list");
        assert_eq!(
            object_kind_two_vs_kid.elapsed,
            M68kCycleCount::from_raw(194)
        );
        assert_eq!(object_kind_two_vs_kid.refresh_events, 1);
        assert_eq!(
            object_kind_two_vs_kid.phase_after,
            M68kRefreshPhase::new(94).expect("frame-1697 loc_86A phase"),
            "ASM loc_862 -> sub_1B76 -> loc_86A reaches the oracle boundary",
        );
    }

    #[test]
    fn nonempty_shooter_collision_lists_fail_closed() {
        assert_eq!(
            shooter_support_collision_raw_cycles_sub_1f52(ModeLevelShooterCollisionWork {
                active_support_slots: 1,
                active_moving_objects: 0,
            }),
            None
        );
        let moving = ModeLevelShooterCollisionWork {
            active_support_slots: 0,
            active_moving_objects: 1,
        };
        assert_eq!(shooter_kid_collision_raw_cycles_sub_219c(moving), None);
        assert_eq!(
            shooter_projectile_collision_raw_cycles_sub_226a(moving),
            None
        );
    }

    #[test]
    fn inactive_block_debris_slots_take_fixed_six_slot_scan() {
        let work = ModeLevelBlockDebrisWorkSub2A4C {
            active_slot_count: 0,
        };
        assert_eq!(
            block_debris_sprites_raw_cycles_sub_2a4c(work),
            Some(M68kCycleCount::from_raw(408))
        );
        let schedule = |phase, wait_after: &[usize]| {
            let mut source_instruction_index = 0_usize;
            block_debris_sprites_advance_with_scheduler_sub_2a4c(
                M68kRefreshPhase::new(phase).expect("published loc_87A phase"),
                work,
                |cycles| {
                    if cycles.get() < 4 || cycles.get() & 1 != 0 {
                        return Some(M68kCycleCount::from_raw(0));
                    }
                    let wait = u32::from(wait_after.contains(&source_instruction_index)) * 11;
                    source_instruction_index += 1;
                    Some(M68kCycleCount::from_raw(wait))
                },
            )
            .expect("published inactive block-debris slots")
        };
        for (advance, elapsed, refresh_events, phase_after) in [
            (schedule(12, &[]), 432, 3, 60),
            (schedule(112, &[0, 2]), 456, 4, 56),
            (schedule(26, &[]), 432, 3, 74),
        ] {
            assert_eq!(advance.elapsed, M68kCycleCount::from_raw(elapsed));
            assert_eq!(advance.refresh_events, refresh_events);
            assert_eq!(
                advance.phase_after,
                M68kRefreshPhase::new(phase_after).expect("published loc_87E phase"),
                "ASM sub_2A4C oracle frames 1697/1703/1709",
            );
        }
        assert_eq!(
            block_debris_sprites_raw_cycles_sub_2a4c(ModeLevelBlockDebrisWorkSub2A4C {
                active_slot_count: 1,
            }),
            Some(M68kCycleCount::from_raw(1_580)),
            "ASM sub_2A4C compiles one 112-instruction active slot before the five inactive scans",
        );
    }

    #[test]
    fn empty_diamond_power_sprite_buffer_takes_early_return() {
        assert_eq!(
            diamond_power_compile_sprites_raw_cycles_sub_3f798(
                ModeLevelDiamondPowerCompileWorkSub3F798 {
                    buffered_entry_count: 0,
                },
            ),
            Some(M68kCycleCount::from_raw(56))
        );
        assert_eq!(
            diamond_power_compile_sprites_raw_cycles_sub_3f798(
                ModeLevelDiamondPowerCompileWorkSub3F798 {
                    buffered_entry_count: 1,
                },
            ),
            None
        );
    }

    #[test]
    fn diamond_power_compile_sprites_scheduler_matches_published_empty_paths_sub_3f798() {
        let work = ModeLevelDiamondPowerCompileWorkSub3F798 {
            buffered_entry_count: 0,
        };
        for (phase, expected_phase) in [(114, 94), (54, 34), (78, 58)] {
            let advance = diamond_power_compile_sprites_advance_with_scheduler_sub_3f798(
                M68kRefreshPhase::new(phase).expect("published loc_882 phase"),
                work,
                |_| Some(M68kCycleCount::from_raw(0)),
            )
            .expect("published empty Diamond Power buffer");
            assert_eq!(advance.elapsed, M68kCycleCount::from_raw(108));
            assert_eq!(advance.refresh_events, 1);
            assert_eq!(
                advance.phase_after,
                M68kRefreshPhase::new(expected_phase).expect("published loc_88E phase"),
            );
        }
    }

    #[test]
    fn platform_sprite_scheduler_matches_published_empty_paths_sub_2444() {
        let work = ModeLevelPlatformWork {
            frozen_fa64: false,
            active_object_count: 0,
        };
        for (phase, expected_phase) in [(94, 64), (34, 4), (58, 28)] {
            let advance = make_sprites_from_platforms_advance_with_scheduler_sub_2444(
                M68kRefreshPhase::new(phase).expect("published loc_88E phase"),
                work,
                |_| Some(M68kCycleCount::from_raw(0)),
            )
            .expect("published empty platform list");
            assert_eq!(advance.elapsed, M68kCycleCount::from_raw(98));
            assert_eq!(advance.refresh_events, 1);
            assert_eq!(
                advance.phase_after,
                M68kRefreshPhase::new(expected_phase).expect("published loc_892 phase"),
            );
        }
    }

    #[test]
    fn empty_kid_projectile_collision_lists_take_two_early_returns() {
        let empty = ModeLevelKidProjectileCollisionWorkSub1D76 {
            kid_projectile_object_count: 0,
            diamond_power_object_count: 0,
        };
        assert_eq!(
            kid_projectile_collision_raw_cycles_sub_1d76(empty),
            Some(M68kCycleCount::from_raw(126))
        );
        for (phase, elapsed, refresh_events, phase_after) in
            [(48, 146, 1, 66), (104, 146, 1, 122), (120, 148, 2, 12)]
        {
            let advance = kid_projectile_collision_advance_with_scheduler_sub_1d76(
                M68kRefreshPhase::new(phase).expect("published loc_872 phase"),
                empty,
                |_| Some(M68kCycleCount::from_raw(0)),
            )
            .expect("published empty Kid-projectile and Diamond Power lists");
            assert_eq!(advance.elapsed, M68kCycleCount::from_raw(elapsed));
            assert_eq!(advance.refresh_events, refresh_events);
            assert_eq!(
                advance.phase_after,
                M68kRefreshPhase::new(phase_after).expect("published loc_876 phase"),
                "ASM sub_1D76 oracle frames 1697/1703/1709",
            );
        }
        assert_eq!(
            kid_projectile_collision_raw_cycles_sub_1d76(
                ModeLevelKidProjectileCollisionWorkSub1D76 {
                    kid_projectile_object_count: 1,
                    ..empty
                },
            ),
            None
        );
        assert_eq!(
            kid_projectile_collision_raw_cycles_sub_1d76(
                ModeLevelKidProjectileCollisionWorkSub1D76 {
                    diamond_power_object_count: 1,
                    ..empty
                },
            ),
            None
        );
    }

    #[test]
    fn empty_special_gfx_lists_take_source_derived_collision_returns() {
        let work = ModeLevelGfxCollisionWork {
            kid_projectile_list_f866: ModeLevelGfxCollisionListWork::Empty,
            object_kind_two_list_f86a: ModeLevelGfxCollisionListWork::Empty,
            object_kind_four_list_f86e: ModeLevelGfxCollisionKidListWorkSub1B76::Unclassified,
            kid_is_invulnerable: false,
        };
        assert_eq!(
            gfx_objects_collision_raw_cycles_sub_16f0(work.kid_projectile_list_f866),
            Some(M68kCycleCount::from_raw(330))
        );
        let first = gfx_objects_collision_advance_with_scheduler_sub_16f0(
            M68kRefreshPhase::new(14).expect("frame-1697 loc_82E phase"),
            work.object_kind_two_list_f86a,
            |_| Some(M68kCycleCount::from_raw(0)),
        )
        .expect("published empty object-kind-two collision list");
        assert_eq!(first.elapsed, M68kCycleCount::from_raw(360));
        assert_eq!(first.refresh_events, 2);
        assert_eq!(
            first.phase_after,
            M68kRefreshPhase::new(118).expect("frame-1697 loc_836 phase"),
        );
        let second = gfx_objects_collision_advance_with_scheduler_sub_16f0(
            first.phase_after,
            work.kid_projectile_list_f866,
            |_| Some(M68kCycleCount::from_raw(0)),
        )
        .expect("published empty Kid-projectile collision list");
        assert_eq!(second.elapsed, M68kCycleCount::from_raw(362));
        assert_eq!(second.refresh_events, 3);
        assert_eq!(
            second.phase_after,
            M68kRefreshPhase::new(96).expect("frame-1697 loc_83E phase"),
            "both ASM sub_16F0 calls reach their bounded oracle phases",
        );
        let latch = mode_level_latch_damage_state_advance_loc_83e(second.phase_after, |_| {
            Some(M68kCycleCount::from_raw(0))
        })
        .expect("frame-1697 damage-state latch");
        assert_eq!(latch.elapsed, M68kCycleCount::from_raw(58));
        assert_eq!(latch.refresh_events, 1);
        assert_eq!(
            latch.phase_after,
            M68kRefreshPhase::new(26).expect("frame-1697 loc_84E phase"),
            "ASM loc_83E..loc_84A reaches the oracle boundary",
        );
        let projectile_vs_kid = gfx_objects_collision_kid_advance_with_scheduler_sub_1b76(
            latch.phase_after,
            work.kid_projectile_list_f866,
            &work,
            |_| Some(M68kCycleCount::from_raw(0)),
        )
        .expect("published empty Kid-projectile versus Kid list");
        assert_eq!(projectile_vs_kid.elapsed, M68kCycleCount::from_raw(194));
        assert_eq!(projectile_vs_kid.refresh_events, 1);
        assert_eq!(
            projectile_vs_kid.phase_after,
            M68kRefreshPhase::new(92).expect("frame-1697 loc_856 phase"),
            "ASM sub_1B76 empty path reaches the oracle boundary",
        );
        assert_eq!(
            gfx_objects_collision_kid_raw_cycles_sub_1b76(work.object_kind_two_list_f86a, &work,),
            Some(M68kCycleCount::from_raw(166))
        );

        let unclassified = ModeLevelGfxCollisionListWork::NonEmptyOrUnclassified;
        assert_eq!(
            gfx_objects_collision_raw_cycles_sub_16f0(unclassified),
            None
        );
        assert_eq!(
            gfx_objects_collision_kid_raw_cycles_sub_1b76(unclassified, &work),
            None
        );
        assert_eq!(
            gfx_objects_collision_kid_raw_cycles_sub_1b76(
                ModeLevelGfxCollisionListWork::Empty,
                &ModeLevelGfxCollisionWork {
                    kid_is_invulnerable: true,
                    ..work.clone()
                },
            ),
            None,
            "the invulnerable loc_1D00 tail depends on live collision words",
        );
    }

    #[test]
    fn f86e_collision_walk_sums_each_oracle_observed_node_path_sub_1b76() {
        let work = |sentinels, misses_right, misses_left, misses_bottom| {
            let mut nodes = Vec::new();
            nodes.extend(std::iter::repeat_n(
                ModeLevelGfxCollisionKidNodeWorkSub1B76::CollisionRowsSentinelLoc1bc0,
                sentinels,
            ));
            nodes.extend(std::iter::repeat_n(
                ModeLevelGfxCollisionKidNodeWorkSub1B76::UnflippedLeftEdgeBeyondKidRightLoc1bc8,
                misses_right,
            ));
            nodes.extend(std::iter::repeat_n(
                ModeLevelGfxCollisionKidNodeWorkSub1B76::UnflippedRightEdgeBeforeKidLeftLoc1bd0,
                misses_left,
            ));
            nodes.extend(std::iter::repeat_n(
                ModeLevelGfxCollisionKidNodeWorkSub1B76::UnflippedTopEdgeBeyondKidBottomLoc1bde,
                misses_bottom,
            ));
            ModeLevelGfxCollisionWork {
                kid_projectile_list_f866: ModeLevelGfxCollisionListWork::Empty,
                object_kind_two_list_f86a: ModeLevelGfxCollisionListWork::Empty,
                object_kind_four_list_f86e: ModeLevelGfxCollisionKidListWorkSub1B76::Classified {
                    nodes,
                },
                kid_is_invulnerable: false,
            }
        };

        for (work, expected) in [
            (work(10, 3, 1, 0), 2_574),
            (work(10, 4, 1, 0), 2_780),
            (work(8, 5, 0, 1), 2_718),
        ] {
            assert_eq!(
                gfx_objects_collision_kid_f86e_raw_cycles_sub_1b76(&work),
                Some(M68kCycleCount::from_raw(expected)),
                "ASM sub_1B76 oracle frames 1697/1703/1709",
            );
        }

        let prize_overlap = ModeLevelGfxCollisionWork {
            kid_projectile_list_f866: ModeLevelGfxCollisionListWork::Empty,
            object_kind_two_list_f86a: ModeLevelGfxCollisionListWork::Empty,
            object_kind_four_list_f86e: ModeLevelGfxCollisionKidListWorkSub1B76::Classified {
                nodes: vec![
                    ModeLevelGfxCollisionKidNodeWorkSub1B76::PrizeOverlapLoc1d5a,
                    ModeLevelGfxCollisionKidNodeWorkSub1B76::CollisionRowsSentinelLoc1bc0,
                ],
            },
            kid_is_invulnerable: false,
        };
        assert_eq!(
            gfx_objects_collision_kid_f86e_raw_cycles_sub_1b76(&prize_overlap),
            Some(M68kCycleCount::from_raw(528)),
            "ASM frame 1057 sub_1B76 entry through loc_1D5A and return_1D32",
        );

        let scheduled_work = |node_cycles: &[u32]| {
            let nodes = node_cycles
                .iter()
                .map(|cycles| {
                    match cycles {
                    156 => {
                        ModeLevelGfxCollisionKidNodeWorkSub1B76::CollisionRowsSentinelLoc1bc0
                    }
                    206 => ModeLevelGfxCollisionKidNodeWorkSub1B76::
                        UnflippedLeftEdgeBeyondKidRightLoc1bc8,
                    230 => ModeLevelGfxCollisionKidNodeWorkSub1B76::
                        UnflippedRightEdgeBeforeKidLeftLoc1bd0,
                    274 => ModeLevelGfxCollisionKidNodeWorkSub1B76::
                        UnflippedTopEdgeBeyondKidBottomLoc1bde,
                    _ => panic!("unsupported published sub_1B76 node path"),
                }
                })
                .collect();
            ModeLevelGfxCollisionWork {
                kid_projectile_list_f866: ModeLevelGfxCollisionListWork::Empty,
                object_kind_two_list_f86a: ModeLevelGfxCollisionListWork::Empty,
                object_kind_four_list_f86e: ModeLevelGfxCollisionKidListWorkSub1B76::Classified {
                    nodes,
                },
                kid_is_invulnerable: false,
            }
        };
        let schedule = |phase, work: &ModeLevelGfxCollisionWork, wait_after: &[usize]| {
            let mut source_instruction_index = 0_usize;
            gfx_objects_collision_kid_f86e_advance_with_scheduler_sub_1b76(
                M68kRefreshPhase::new(phase).expect("published loc_86A phase"),
                work,
                |cycles| {
                    // The oracle's XFER68K bank steals are 11-cycle external
                    // waits. Combined refresh+bank waits are fed back as 13
                    // cycles and must not advance the source instruction index.
                    if cycles.get() < 4 || cycles.get() & 1 != 0 {
                        return Some(M68kCycleCount::from_raw(0));
                    }
                    let wait = u32::from(wait_after.contains(&source_instruction_index)) * 11;
                    source_instruction_index += 1;
                    Some(M68kCycleCount::from_raw(wait))
                },
            )
            .expect("published classified kind-4 collision walk")
        };

        let frame_1697 = scheduled_work(&[
            156, 156, 156, 206, 156, 156, 156, 206, 156, 156, 156, 230, 156, 206,
        ]);
        let advance_1697 = schedule(94, &frame_1697, &[]);
        assert_eq!(advance_1697.elapsed, M68kCycleCount::from_raw(2_642));
        assert_eq!(advance_1697.refresh_events, 21);
        assert_eq!(
            advance_1697.phase_after,
            M68kRefreshPhase::new(48).expect("frame-1697 loc_872 phase"),
        );

        let frame_1703 = scheduled_work(&[
            156, 156, 206, 156, 156, 206, 156, 156, 156, 206, 156, 156, 230, 156, 206,
        ]);
        let advance_1703 = schedule(
            126,
            &frame_1703,
            &[
                57, 61, 65, 69, 73, 77, 147, 150, 155, 158, 163, 166, 239, 242, 247, 250, 254, 257,
            ],
        );
        assert_eq!(advance_1703.elapsed, M68kCycleCount::from_raw(3_050));
        assert_eq!(advance_1703.refresh_events, 23);
        assert_eq!(
            advance_1703.phase_after,
            M68kRefreshPhase::new(104).expect("frame-1703 loc_872 phase"),
        );

        let frame_1709 = scheduled_work(&[
            206, 156, 206, 156, 156, 206, 156, 156, 156, 206, 156, 274, 156, 206,
        ]);
        let advance_1709 = schedule(
            80,
            &frame_1709,
            &[
                46, 50, 54, 58, 62, 67, 137, 141, 144, 148, 153, 156, 234, 237, 242, 246, 249, 253,
            ],
        );
        assert_eq!(advance_1709.elapsed, M68kCycleCount::from_raw(2_984));
        assert_eq!(advance_1709.refresh_events, 21);
        assert_eq!(
            advance_1709.phase_after,
            M68kRefreshPhase::new(120).expect("frame-1709 loc_872 phase"),
        );
    }

    #[test]
    fn gfx_move_animate_sums_each_prize_worker_script_state_sub_1444() {
        let work = |wobbles: &[u16], flashes: &[u16], diamonds: &[u16]| {
            let mut nodes = vec![ModeLevelGfxMoveAnimateNodeWorkSub1444::StationaryUnanimated; 5];
            nodes.extend(wobbles.iter().map(|&allocation_age| {
                ModeLevelGfxMoveAnimateNodeWorkSub1444::PrizeWobble { allocation_age }
            }));
            nodes.extend(flashes.iter().map(|&allocation_age| {
                ModeLevelGfxMoveAnimateNodeWorkSub1444::PrizeFlash { allocation_age }
            }));
            nodes.extend(diamonds.iter().map(|&allocation_age| {
                ModeLevelGfxMoveAnimateNodeWorkSub1444::FloatingDiamond { allocation_age }
            }));
            ModeLevelGfxMoveAnimateWorkSub1444::Classified { nodes }
        };

        for (work, expected) in [
            (work(&[26, 19, 13, 7, 0], &[17, 7, 0], &[202, 16, 6]), 4_500),
            (
                work(&[25, 19, 13, 6], &[23, 13, 6, 0], &[208, 22, 12, 5]),
                4_894,
            ),
            (
                work(&[25, 19, 12], &[19, 12, 6], &[214, 28, 18, 11, 5]),
                4_594,
            ),
        ] {
            assert_eq!(
                gfx_objects_move_and_animate_raw_cycles_sub_1444(&work),
                Some(M68kCycleCount::from_raw(expected)),
                "ASM sub_1444 oracle frames 1697/1703/1709",
            );
            if expected == 4_500 {
                let advance = gfx_objects_move_and_animate_advance_with_scheduler_sub_1444(
                    M68kRefreshPhase::new(46).expect("frame-1697 loc_816 phase"),
                    &work,
                    |_| Some(M68kCycleCount::from_raw(0)),
                )
                .expect("frame-1697 classified GfxObject branches");
                assert_eq!(advance.elapsed, M68kCycleCount::from_raw(4_590));
                assert_eq!(advance.refresh_events, 36);
                assert_eq!(
                    advance.phase_after,
                    M68kRefreshPhase::new(28).expect("frame-1697 loc_81A phase"),
                    "ASM sub_1444 instruction stream reaches the bounded oracle boundary",
                );
            }
        }

        let stopped_wobble = ModeLevelGfxMoveAnimateWorkSub1444::Classified {
            nodes: vec![
                ModeLevelGfxMoveAnimateNodeWorkSub1444::StationaryUnanimated,
                ModeLevelGfxMoveAnimateNodeWorkSub1444::StationaryUnanimated,
                ModeLevelGfxMoveAnimateNodeWorkSub1444::StationaryUnanimated,
                ModeLevelGfxMoveAnimateNodeWorkSub1444::StationaryUnanimated,
                ModeLevelGfxMoveAnimateNodeWorkSub1444::PrizeWobble { allocation_age: 24 },
            ],
        };
        assert_eq!(
            gfx_objects_move_and_animate_raw_cycles_sub_1444(&stopped_wobble),
            Some(M68kCycleCount::from_raw(1_300)),
            "ASM sub_1444 -> GfxObject_Animate loc_1158 -> loc_1182 frame-1013 stop dispatch",
        );
    }

    #[test]
    fn gfx_sprite_builder_sums_each_source_branch_sub_1492() {
        let work = |visible_single_count, hidden_count| {
            let mut nodes = vec![
                ModeLevelGfxSpriteNodeWorkSub1492::HiddenFlagClear,
                ModeLevelGfxSpriteNodeWorkSub1492::VisibleTwoSpriteNormal,
                ModeLevelGfxSpriteNodeWorkSub1492::VerticallyCulled,
                ModeLevelGfxSpriteNodeWorkSub1492::VerticallyCulled,
            ];
            nodes.extend(std::iter::repeat_n(
                ModeLevelGfxSpriteNodeWorkSub1492::VisibleSingle16x16Normal,
                visible_single_count,
            ));
            nodes.extend(std::iter::repeat_n(
                ModeLevelGfxSpriteNodeWorkSub1492::HiddenFlagClear,
                hidden_count,
            ));
            ModeLevelGfxSpriteWorkSub1492::Classified { nodes }
        };

        for (work, expected) in [
            (work(10, 2), 11_628),
            (work(11, 2), 12_554),
            (work(12, 0), 13_240),
        ] {
            assert_eq!(
                make_sprites_from_gfx_objects_raw_cycles_sub_1492(&work),
                Some(M68kCycleCount::from_raw(expected)),
                "ASM sub_1492 oracle frames 1697/1703/1709",
            );
        }

        let wall_cling = ModeLevelGfxSpriteWorkSub1492::Classified {
            nodes: vec![
                ModeLevelGfxSpriteNodeWorkSub1492::MappingPointerNull,
                ModeLevelGfxSpriteNodeWorkSub1492::VisibleSingleDynamicArt20x32Normal,
                ModeLevelGfxSpriteNodeWorkSub1492::VerticallyCulledAboveViewport,
                ModeLevelGfxSpriteNodeWorkSub1492::VerticallyCulledAboveViewport,
            ],
        };
        assert_eq!(
            make_sprites_from_gfx_objects_raw_cycles_sub_1492(&wall_cling),
            Some(M68kCycleCount::from_raw(1_968)),
            "ASM sub_1492 frame 950 wall-cling sprite walk",
        );
        let airborne_state_zero = ModeLevelGfxSpriteWorkSub1492::Classified {
            nodes: vec![
                ModeLevelGfxSpriteNodeWorkSub1492::MappingPointerNull,
                ModeLevelGfxSpriteNodeWorkSub1492::VisibleDynamicArt22x32AirborneNormal,
                ModeLevelGfxSpriteNodeWorkSub1492::VerticallyCulledAboveViewport,
                ModeLevelGfxSpriteNodeWorkSub1492::VerticallyCulledAboveViewport,
            ],
        };
        assert_eq!(
            make_sprites_from_gfx_objects_raw_cycles_sub_1492(&airborne_state_zero),
            Some(M68kCycleCount::from_raw(1_968)),
            "ASM sub_B270 -> sub_1492 oracle frame 983 airborne state-0 sprite walk",
        );
        let wall_cling_advance = make_sprites_from_gfx_objects_advance_with_scheduler_sub_1492(
            M68kRefreshPhase::new(60).expect("oracle frame-950 loc_87E phase"),
            &wall_cling,
            |_| Some(M68kCycleCount::from_raw(0)),
        )
        .expect("frame-950 wall-cling sprite walk");
        assert_eq!(wall_cling_advance.elapsed, M68kCycleCount::from_raw(2_018));
        assert_eq!(wall_cling_advance.refresh_events, 16);
        assert_eq!(
            wall_cling_advance.phase_after,
            M68kRefreshPhase::new(30).expect("oracle frame-950 loc_882 phase"),
        );

        let schedule = |phase, work: &ModeLevelGfxSpriteWorkSub1492, wait_after: &[usize]| {
            let mut source_instruction_index = 0_usize;
            make_sprites_from_gfx_objects_advance_with_scheduler_sub_1492(
                M68kRefreshPhase::new(phase).expect("published loc_87E phase"),
                work,
                |cycles| {
                    if cycles.get() < 4 || cycles.get() & 1 != 0 {
                        return Some(M68kCycleCount::from_raw(0));
                    }
                    let wait = u32::from(wait_after.contains(&source_instruction_index)) * 11;
                    source_instruction_index += 1;
                    Some(M68kCycleCount::from_raw(wait))
                },
            )
            .expect("published classified GfxObject sprite walk")
        };
        let frame_1697_work = work(10, 2);
        let frame_1697 = schedule(60, &frame_1697_work, &[]);
        assert_eq!(frame_1697.elapsed, M68kCycleCount::from_raw(11_830));
        assert_eq!(frame_1697.refresh_events, 92);
        assert_eq!(
            frame_1697.phase_after,
            M68kRefreshPhase::new(114).expect("frame-1697 loc_882 phase"),
        );
        let frame_1703_work = work(11, 2);
        let frame_1703 = schedule(
            56,
            &frame_1703_work,
            &[
                26, 30, 34, 38, 42, 46, 130, 133, 137, 140, 143, 147, 210, 214, 218, 222, 226, 230,
                304, 307, 312, 315, 320, 323, 396, 400,
            ],
        );
        assert_eq!(frame_1703.elapsed, M68kCycleCount::from_raw(13_054));
        assert_eq!(frame_1703.refresh_events, 98);
        assert_eq!(
            frame_1703.phase_after,
            M68kRefreshPhase::new(54).expect("frame-1703 loc_882 phase"),
        );
        let frame_1709_work = work(12, 0);
        let frame_1709 = schedule(
            74,
            &frame_1709_work,
            &[
                15, 18, 22, 25, 28, 32, 107, 113, 120, 125, 128, 131, 195, 197, 202, 205, 207, 211,
                288, 291, 295, 298, 301, 305, 385, 389, 392, 395, 399, 402, 479, 482, 485, 489,
                492, 496, 572, 576, 579, 582, 586, 589, 665, 669,
            ],
        );
        assert_eq!(frame_1709.elapsed, M68kCycleCount::from_raw(13_956));
        assert_eq!(frame_1709.refresh_events, 107);
        assert_eq!(
            frame_1709.phase_after,
            M68kRefreshPhase::new(78).expect("frame-1709 loc_882 phase"),
        );

        let grounded_kid = ModeLevelGfxSpriteWorkSub1492::Classified {
            nodes: vec![
                ModeLevelGfxSpriteNodeWorkSub1492::MappingPointerNull,
                ModeLevelGfxSpriteNodeWorkSub1492::VisibleSingleDynamicArt20x32Normal,
                ModeLevelGfxSpriteNodeWorkSub1492::VerticallyCulledAboveViewport,
                ModeLevelGfxSpriteNodeWorkSub1492::VerticallyCulledAboveViewport,
            ],
        };
        let frame_919 = make_sprites_from_gfx_objects_advance_with_scheduler_sub_1492(
            M68kRefreshPhase::new(34).expect("frame-919 loc_87E phase"),
            &grounded_kid,
            |_| Some(M68kCycleCount::from_raw(0)),
        )
        .expect("published grounded-Kid GfxObject sprite walk");
        assert_eq!(
            frame_919,
            M68kRefreshAdvance {
                elapsed: M68kCycleCount::from_raw(2_018),
                phase_after: M68kRefreshPhase::new(4).expect("frame-919 loc_882 phase"),
                refresh_events: 16,
            },
            "ASM loc_87E -> sub_1492 -> loc_882 selected-PC instruction journal"
        );

        let walking_kid = ModeLevelGfxSpriteWorkSub1492::Classified {
            nodes: vec![
                ModeLevelGfxSpriteNodeWorkSub1492::MappingPointerNull,
                ModeLevelGfxSpriteNodeWorkSub1492::VisibleSingleDynamicArt32x24Normal,
                ModeLevelGfxSpriteNodeWorkSub1492::VerticallyCulledAboveViewport,
                ModeLevelGfxSpriteNodeWorkSub1492::VerticallyCulledAboveViewport,
            ],
        };
        let frame_923 = make_sprites_from_gfx_objects_advance_with_scheduler_sub_1492(
            M68kRefreshPhase::new(126).expect("frame-923 loc_87E phase"),
            &walking_kid,
            |_| Some(M68kCycleCount::from_raw(0)),
        )
        .expect("published walking-Kid GfxObject sprite walk");
        assert_eq!(
            frame_923,
            M68kRefreshAdvance {
                elapsed: M68kCycleCount::from_raw(1_994),
                phase_after: M68kRefreshPhase::new(72).expect("frame-923 loc_882 phase"),
                refresh_events: 16,
            },
            "ASM loc_129E -> loc_12EC -> loc_130E -> sub_13C0 loc_141A selected-PC instruction journal",
        );
    }

    #[test]
    fn clear_special_terrain_scan_derives_inclusive_cell_walk() {
        let activation = ModeLevelTerrainSpecialScanSub1FA2 {
            bounds: crate::ModeLevelKidGfxTerrainScanBounds {
                left_col: 2,
                right_col: 2,
                top_row: 45,
                bottom_row: 45,
            },
            tile_iterations: 1,
            first_special_collision: None,
        };
        let scan = |bottom_row| ModeLevelTerrainSpecialScanSub1FA2 {
            bounds: crate::ModeLevelKidGfxTerrainScanBounds {
                left_col: 10,
                right_col: 11,
                top_row: 20,
                bottom_row,
            },
            tile_iterations: (bottom_row - 20 + 1) * 2,
            first_special_collision: None,
        };
        assert_eq!(
            terrain_special_scan_raw_cycles_sub_1fa2(scan(21)),
            Some(M68kCycleCount::from_raw(558))
        );
        assert_eq!(
            terrain_special_scan_raw_cycles_sub_1fa2(scan(22)),
            Some(M68kCycleCount::from_raw(688))
        );
        let schedule = |phase, work: ModeLevelTerrainSpecialScanSub1FA2, wait_after: &[usize]| {
            let mut source_instruction_index = 0_usize;
            terrain_special_scan_advance_with_scheduler_sub_1fa2(
                M68kRefreshPhase::new(phase).expect("published loc_876 phase"),
                work,
                |cycles| {
                    if cycles.get() < 4 || cycles.get() & 1 != 0 {
                        return Some(M68kCycleCount::from_raw(0));
                    }
                    let wait = u32::from(wait_after.contains(&source_instruction_index)) * 11;
                    source_instruction_index += 1;
                    Some(M68kCycleCount::from_raw(wait))
                },
            )
            .expect("published clear special-terrain scan")
        };
        assert_eq!(
            terrain_special_scan_raw_cycles_sub_1fa2(activation),
            Some(M68kCycleCount::from_raw(382)),
            "ASM frame-902 sub_21F8 loc_2252 leaves a one-cell activation hitbox",
        );
        let frame_902 = schedule(37, activation, &[]);
        assert_eq!(frame_902.elapsed, M68kCycleCount::from_raw(406));
        assert_eq!(frame_902.refresh_events, 3);
        assert_eq!(
            frame_902.phase_after,
            M68kRefreshPhase::new(59).expect("frame-902 loc_87A phase"),
        );
        let frame_1697 = schedule(66, scan(21), &[]);
        assert_eq!(frame_1697.elapsed, M68kCycleCount::from_raw(586));
        assert_eq!(frame_1697.refresh_events, 5);
        assert_eq!(
            frame_1697.phase_after,
            M68kRefreshPhase::new(12).expect("frame-1697 loc_87A phase"),
        );
        let frame_1703 = schedule(122, scan(21), &[37, 43, 48, 50]);
        assert_eq!(frame_1703.elapsed, M68kCycleCount::from_raw(630));
        assert_eq!(frame_1703.refresh_events, 5);
        assert_eq!(
            frame_1703.phase_after,
            M68kRefreshPhase::new(112).expect("frame-1703 loc_87A phase"),
        );
        let frame_1709 = schedule(12, scan(22), &[36, 42, 46, 50, 55, 60]);
        assert_eq!(frame_1709.elapsed, M68kCycleCount::from_raw(782));
        assert_eq!(frame_1709.refresh_events, 5);
        assert_eq!(
            frame_1709.phase_after,
            M68kRefreshPhase::new(26).expect("frame-1709 loc_87A phase"),
        );
        assert_eq!(
            terrain_special_scan_raw_cycles_sub_1fa2(ModeLevelTerrainSpecialScanSub1FA2 {
                tile_iterations: 3,
                ..scan(21)
            }),
            None
        );
    }

    fn published_refresh_suffix_after_final_vdp_ownership()
    -> ModeLevelVblankRefreshSuffixAfterFinalVdpOwnership {
        ModeLevelVblankRefreshSuffixAfterFinalVdpOwnership {
            // ASM loc_30104: the resumable command write takes 16 raw cycles,
            // its pending VDP wait takes 60, then 1,146 raw cycles reach E12.
            raw_cycles_before_pending_wait: 16,
            pending_wait_cycles: 60,
            raw_cycles_after_pending_wait: 1_146,
        }
    }

    #[test]
    fn mode_level_vblank_separates_source_work_from_vdp_arbitration() {
        let work = published_natural_vblank_work();
        assert_eq!(
            mode_level_vblank_source_cycles_before_sub_e12(work).map(M68kCycleCount::get),
            Some(3_415),
            "ASM Mode_Level $7C8 -> Execute_Objects sub_E12 source path"
        );
        assert_eq!(
            mode_level_vblank_source_cycles_before_sub_e12(ModeLevelVblankWorkBeforeSubE12 {
                hud_path_sub_bdc0: ModeLevelHudBuildPath::LocBe20DirectStableCounters,
                ..work
            })
            .map(M68kCycleCount::get),
            Some(3_305),
            "ASM sub_BDC0 direct stable path skips both calc_display_number calls"
        );
        assert_eq!(
            mode_level_vblank_source_cycles_before_sub_e12(ModeLevelVblankWorkBeforeSubE12 {
                palette_dma_sub_970: false,
                hud_path_sub_bdc0: ModeLevelHudBuildPath::LocBe20DirectStableCounters,
                ..work
            })
            .map(M68kCycleCount::get),
            Some(2_999),
            "ASM Palette_to_VRAM sub_970 suppressed branch omits 306 source cycles"
        );
        assert_eq!(
            mode_level_vblank_source_cycles_before_sub_e12(ModeLevelVblankWorkBeforeSubE12 {
                hud_path_sub_bdc0: ModeLevelHudBuildPath::LocBe20ThenLocBe4cStableCounters,
                ..work
            })
            .map(M68kCycleCount::get),
            Some(3_415),
            "ASM sub_BDC0 stable counters replace both calc_display_number paths with the second hitpoint loop"
        );
        assert_eq!(
            mode_level_vblank_source_cycles_before_sub_e12(ModeLevelVblankWorkBeforeSubE12 {
                dynamic_art_tiles_loc_ad4: None,
                ..work
            })
            .map(M68kCycleCount::get),
            Some(2_839),
            "ASM sub_A5C loc_AD4's empty queue removes the two typed Kid-art DMA source paths",
        );
        for (arbitration, expected) in [
            (
                ModeLevelVblankArbitrationBeforeSubE12 {
                    vdp_bus_halt_cycles: 3_383,
                    vdp_pending_wait_cycles: 2_002,
                    refresh_wait_cycles: 46,
                    refresh_suffix_after_final_vdp_ownership:
                        published_refresh_suffix_after_final_vdp_ownership(),
                },
                8_846,
            ),
            (
                ModeLevelVblankArbitrationBeforeSubE12 {
                    vdp_bus_halt_cycles: 3_380,
                    vdp_pending_wait_cycles: 2_002,
                    refresh_wait_cycles: 46,
                    refresh_suffix_after_final_vdp_ownership:
                        published_refresh_suffix_after_final_vdp_ownership(),
                },
                8_843,
            ),
            (
                ModeLevelVblankArbitrationBeforeSubE12 {
                    vdp_bus_halt_cycles: 3_384,
                    vdp_pending_wait_cycles: 2_002,
                    refresh_wait_cycles: 48,
                    refresh_suffix_after_final_vdp_ownership:
                        published_refresh_suffix_after_final_vdp_ownership(),
                },
                8_849,
            ),
        ] {
            assert_eq!(
                mode_level_vblank_elapsed_cycles_before_sub_e12(work, arbitration)
                    .map(M68kCycleCount::get),
                Some(expected),
                "published frames 1697/1703/1709"
            );
        }
    }

    #[test]
    fn final_hscroll_dma_reset_drives_the_sub_e12_refresh_phase() {
        for (vdp_bus_halt_cycles, refresh_wait_cycles, expected_elapsed) in
            [(3_383, 46, 8_846), (3_380, 46, 8_843), (3_384, 48, 8_849)]
        {
            let advance = mode_level_vblank_advance_before_sub_e12(
                published_natural_vblank_work(),
                ModeLevelVblankArbitrationBeforeSubE12 {
                    vdp_bus_halt_cycles,
                    vdp_pending_wait_cycles: 2_002,
                    refresh_wait_cycles,
                    refresh_suffix_after_final_vdp_ownership:
                        published_refresh_suffix_after_final_vdp_ownership(),
                },
            )
            .expect("ASM loc_30104 final VDP ownership has a typed suffix");
            assert_eq!(advance.elapsed.get(), expected_elapsed);
            assert_eq!(
                advance.phase_after.get(),
                88,
                "ASM loc_30104 -> Execute_Objects sub_E12 resets then advances the counter"
            );
        }
    }

    #[test]
    fn mode_level_vblank_rejects_untyped_source_branches() {
        let work = ModeLevelVblankWorkBeforeSubE12 {
            scroll_path: ModeLevelScrollTransferPath::StormLoc30110,
            ..published_natural_vblank_work()
        };
        assert_eq!(mode_level_vblank_source_cycles_before_sub_e12(work), None);
    }

    #[test]
    fn wait_for_vint_elapsed_cycles_preserve_saved_pc_and_bus_arbitration() {
        for (resume, arbitration, expected) in [
            (
                WaitForVintResumePath::PollTestLoc93e,
                WaitForVintArbitration {
                    refresh_wait_events: 1,
                    z80_68k_bus_steals: 0,
                },
                164,
            ),
            (
                WaitForVintResumePath::StaleBranchLoc942,
                WaitForVintArbitration {
                    refresh_wait_events: 1,
                    z80_68k_bus_steals: 0,
                },
                174,
            ),
            (
                WaitForVintResumePath::PollTestLoc93e,
                WaitForVintArbitration {
                    refresh_wait_events: 0,
                    z80_68k_bus_steals: 3,
                },
                195,
            ),
        ] {
            assert_eq!(
                wait_for_vint_elapsed_cycles_to_mode_level(resume, arbitration)
                    .map(M68kCycleCount::get),
                Some(expected),
                "ASM V_Int $488 -> Mode_Level $7C8"
            );
        }
    }

    #[test]
    fn vint_assertion_to_mode_level_retains_partial_instruction_mclk() {
        for (
            sample_mclk,
            interrupt_refresh_wait_events,
            resume,
            handler_refresh_wait_events,
            expected_mclk,
        ) in [
            (58, 0, WaitForVintResumePath::StaleBranchLoc942, 2, 1_668),
            (0, 1, WaitForVintResumePath::PollTestLoc93e, 1, 1_540),
            (82, 0, WaitForVintResumePath::StaleBranchLoc942, 1, 1_678),
        ] {
            let elapsed =
                wait_for_vint_mclk_from_vint_assertion_to_mode_level(WaitForVintInterruptSample {
                    mclk_until_interrupt_sample: sample_mclk,
                    interrupt_entry_refresh_wait_events: interrupt_refresh_wait_events,
                    resume,
                    arbitration_after_handler_entry: WaitForVintArbitration {
                        refresh_wait_events: handler_refresh_wait_events,
                        z80_68k_bus_steals: 0,
                    },
                })
                .expect("bounded VINT sample and ASM $488 -> $7C8 path");
            assert_eq!(
                elapsed.get(),
                expected_mclk,
                "helper frames 1696/1702/1708 produce native 1697/1703/1709"
            );
        }
    }

    #[test]
    fn mode_level_tail_loc_892_derives_published_wait_for_vint_phases() {
        fn work(
            old_greater_than_new: usize,
            old_less_than_new_width_subtractions: Option<u16>,
            rendered_piece_rows: u8,
        ) -> ModeLevelTailWorkLoc892 {
            let mut shift_data_slots = [ModeLevelTailShiftDataSlotLoc30452::Unchanged; 19];
            for slot in shift_data_slots.iter_mut().take(old_greater_than_new) {
                *slot = ModeLevelTailShiftDataSlotLoc30452::OldGreaterThanNew;
            }
            if let Some(width_subtractions) = old_less_than_new_width_subtractions {
                shift_data_slots[old_greater_than_new] =
                    ModeLevelTailShiftDataSlotLoc30452::OldLessThanNew { width_subtractions };
            }
            ModeLevelTailWorkLoc892 {
                camera_stripe_sub_30194: ModeLevelTailCameraStripeSub30194::BothTileAxesUnchanged,
                background_loc_3038a: ModeLevelTailBackgroundWorkLoc3038A {
                    special_effect_4: false,
                    game_mode_zero: false,
                    background_upload_busy_fb49: false,
                    background_format_nonzero: false,
                    shift_data_slots_loc_30452: shift_data_slots,
                    shift_layers: ModeLevelTailShiftLayersWork::NormalAlignedNoRipple {
                        full_rows: 28,
                    },
                    piece_rows_sub_30744: ModeLevelTailPieceRowsSub30744 {
                        entries_before_sentinel: 29,
                        rendered_nonnegative_entries: rendered_piece_rows,
                        rendered_entries: std::array::from_fn(|index| {
                            index < usize::from(rendered_piece_rows)
                        }),
                    },
                    piece_refresh_sub_307d8: ModeLevelTailPieceRefreshSub307D8::CachedRowUnchanged,
                },
                palette_sub_14c0: ModeLevelTailPaletteWorkSub14C0::Inactive,
            }
        }

        for (phase, work, expected_raw, expected_refresh, expected_elapsed, expected_phase) in [
            (56, work(11, None, 29), 12_852, 102, 13_056, 56),
            (102, work(13, None, 29), 13_080, 104, 13_288, 78),
            (16, work(10, Some(1), 22), 12_310, 97, 12_504, 104),
        ] {
            let phase = M68kRefreshPhase::new(phase).expect("bounded refresh phase");
            let raw = mode_level_tail_raw_cycles_to_wait_for_vint_loc_892(&work)
                .expect("published natural Mode_Level tail is classified");
            assert_eq!(raw.get(), expected_raw, "ASM loc_892 -> first $93E");

            let advance = mode_level_tail_advance_to_wait_for_vint_loc_892(phase, &work)
                .expect("published natural Mode_Level tail has no external waits");
            assert_eq!(advance.refresh_events, expected_refresh);
            assert_eq!(advance.elapsed.get(), expected_elapsed);
            assert_eq!(advance.phase_after.get(), expected_phase);
        }
    }

    #[test]
    fn mode_level_tail_loc_892_rejects_unclassified_streaming_work() {
        let work = ModeLevelTailWorkLoc892 {
            camera_stripe_sub_30194: ModeLevelTailCameraStripeSub30194::StreamingOther,
            background_loc_3038a: ModeLevelTailBackgroundWorkLoc3038A {
                special_effect_4: false,
                game_mode_zero: false,
                background_upload_busy_fb49: false,
                background_format_nonzero: false,
                shift_data_slots_loc_30452: [ModeLevelTailShiftDataSlotLoc30452::Unchanged; 19],
                shift_layers: ModeLevelTailShiftLayersWork::NormalAlignedNoRipple { full_rows: 28 },
                piece_rows_sub_30744: ModeLevelTailPieceRowsSub30744 {
                    entries_before_sentinel: 29,
                    rendered_nonnegative_entries: 29,
                    rendered_entries: [true; 29],
                },
                piece_refresh_sub_307d8: ModeLevelTailPieceRefreshSub307D8::CachedRowUnchanged,
            },
            palette_sub_14c0: ModeLevelTailPaletteWorkSub14C0::Inactive,
        };

        assert_eq!(
            mode_level_tail_raw_cycles_to_wait_for_vint_loc_892(&work),
            None,
            "ASM sub_30194 streaming branches require ordered mapping and VDP work"
        );
    }

    #[test]
    fn mode_level_tail_preload_streaming_work_matches_frame_895_instruction_trace() {
        let profile = [
            96, 96, 80, 80, 80, 112, 112, 64, 64, 64, 96, 96, 96, 96, 96, 96, 64, 64, 64, 64, 80,
            80, 80, 80, 80, 80, 112, 112, 112, 112, 80, 80, 80, 80, 80, 80, 64, 64, 64, 64, 16, 16,
            16, 16, 16, 16, 16, 16, 16,
        ];
        let ordinary_cells = [0x0001_u16; 21];
        let mut state = ModeLevelTailStateLoc892::power_on_zero();
        state.preload_loc_2ffe8(0, 0x02A0, 0x0500, 0);

        let advance = state.advance_loc_892(ModeLevelTailInputsLoc892 {
            camera_x_pos: 0,
            camera_y_pos: 0x0298,
            camera_stripe_cells_sub_30194: Some(&ordinary_cells),
            camera_column_cells_sub_30194: None,
            time_frames_before_loc_3038a: 0,
            game_mode_zero: false,
            special_effect_4: false,
            background_width: 90,
            background_format_nonzero: false,
            background_no_scroll: false,
            background_theme: 1,
            bgscroll_profile: &profile,
            bgscroll_has_terminator: false,
        });
        let work = advance.work.expect("first gameplay pass is source-typed");

        assert_eq!(
            work.camera_stripe_sub_30194,
            ModeLevelTailCameraStripeSub30194::UpwardHorizontalNoWrap {
                cells: [ModeLevelTailCameraCellMappingSub30194::OrdinaryTheme; 21],
            },
            "ASM sub_30194 loc_301CE -> sub_3034A: frame-895 oracle trace takes 21 ordinary mapping branches",
        );
        assert_eq!(
            work.background_loc_3038a.piece_refresh_sub_307d8,
            ModeLevelTailPieceRefreshSub307D8::LinearNoWrap {
                width_subtractions: 0,
                copied_words: 41,
            },
            "ASM sub_307D8 loc_3081A -> loc_308A6: row 20, slot 10, and coarse column zero select the linear 41-word copy",
        );
        assert_eq!(
            mode_level_tail_camera_stripe_raw_cycles_sub_30194(work.camera_stripe_sub_30194),
            Some(M68kCycleCount::from_raw(2_320)),
        );
        assert_eq!(
            mode_level_tail_piece_refresh_raw_cycles_sub_307d8(
                work.background_loc_3038a.piece_refresh_sub_307d8
            ),
            Some(M68kCycleCount::from_raw(2_360)),
        );
        assert_eq!(
            mode_level_tail_raw_cycles_to_wait_for_vint_loc_892(&work),
            Some(M68kCycleCount::from_raw(13_560)),
            "ASM loc_892 raw source work includes both streaming calls and the unaligned shift-layer path",
        );
        assert_eq!(
            mode_level_tail_advance_to_wait_for_vint_loc_892(
                M68kRefreshPhase::new(0).expect("bounded phase"),
                &work,
            ),
            None,
            "frame-895 oracle trace records 386+146 VDP waits and a VINT inside sub_307D8, so refresh-only composition must reject it",
        );
    }

    #[test]
    fn mode_level_tail_first_gameplay_work_matches_frame_902_instruction_trace() {
        let profile = [
            96, 96, 80, 80, 80, 112, 112, 64, 64, 64, 96, 96, 96, 96, 96, 96, 64, 64, 64, 64, 80,
            80, 80, 80, 80, 80, 112, 112, 112, 112, 80, 80, 80, 80, 80, 80, 64, 64, 64, 64, 16, 16,
            16, 16, 16, 16, 16, 16, 16,
        ];
        let mut camera_cells = [0x0001_u16; 21];
        camera_cells[20] = 0xFFFF;
        let mut state = ModeLevelTailStateLoc892::power_on_zero();
        state.preload_loc_2ffe8(0, 0x02A0, 0x0380, 0);
        state.set_pending_palette_masks_f888_f88c(u32::MAX, u32::MAX);

        let advance = state.advance_loc_892(ModeLevelTailInputsLoc892 {
            camera_x_pos: 0,
            camera_y_pos: 0x0298,
            camera_stripe_cells_sub_30194: Some(&camera_cells),
            camera_column_cells_sub_30194: None,
            time_frames_before_loc_3038a: 0,
            game_mode_zero: false,
            special_effect_4: false,
            background_width: 90,
            background_format_nonzero: false,
            background_no_scroll: false,
            background_theme: 1,
            bgscroll_profile: &profile,
            bgscroll_has_terminator: false,
        });
        let work = advance.work.expect("first gameplay pass is source-typed");

        assert_eq!(
            work.camera_stripe_sub_30194,
            ModeLevelTailCameraStripeSub30194::UpwardHorizontalNoWrap {
                cells: std::array::from_fn(|index| {
                    if index == 20 {
                        ModeLevelTailCameraCellMappingSub30194::Block
                    } else {
                        ModeLevelTailCameraCellMappingSub30194::OrdinaryTheme
                    }
                }),
            },
            "ASM sub_30194 loc_30354/loc_3037C: frame-902 oracle trace takes 20 ordinary branches and one block branch",
        );
        assert_eq!(
            work.background_loc_3038a.piece_refresh_sub_307d8,
            ModeLevelTailPieceRefreshSub307D8::CachedRowUnchanged,
            "ASM sub_307D8 loc_3080E -> return_308B4: frame-902 FAA8 already equals Camera_Y_pos>>5",
        );
        assert_eq!(
            work.palette_sub_14c0,
            ModeLevelTailPaletteWorkSub14C0::BothMasksFull,
            "ASM sub_14C0/loc_14CE: frame-902 F888 and F88C are both FFFFFFFF",
        );
        assert_eq!(
            mode_level_tail_raw_cycles_to_wait_for_vint_loc_892(&work),
            Some(M68kCycleCount::from_raw(31_176)),
            "ASM loc_892 -> WaitForVint $93E: frame-902 source instructions total 31,176 raw cycles before hardware waits",
        );
        assert_eq!(
            mode_level_tail_advance_to_wait_for_vint_loc_892(
                M68kRefreshPhase::new(95).expect("oracle loc_892 phase"),
                &work,
            ),
            None,
            "ASM sub_30194's live VDP writes require ordered FIFO/GEMS arbitration",
        );
    }

    #[test]
    fn mode_level_tail_shift_layers_classifies_first_gameplay_unaligned_rows() {
        // ASM BackgroundScroll_ComputeShiftLayers loc_3050A/loc_30562/
        // loc_3056A: Camera_Y_pos $0298 gives phase 6, so the first gameplay
        // pass writes 2 leading rows, 27 full eight-row groups, then 6
        // trailing rows. The two partial loops cost 116 raw cycles more than
        // the aligned 28-group path.
        let work = mode_level_tail_shift_layers_work(0x0298, false, 1);

        assert_eq!(
            work,
            ModeLevelTailShiftLayersWork::NormalUnalignedNoRipple {
                leading_rows: 2,
                full_rows: 27,
                trailing_rows: 6,
            }
        );
        assert_eq!(
            mode_level_tail_shift_layers_raw_cycles(work),
            Some(M68kCycleCount::from_raw(4_136))
        );
    }

    #[test]
    fn shift_data_state_loc_30452_seeds_the_load_ingame_preload_words() {
        let state = ModeLevelTailShiftDataStateLoc30452::from_preload_loc_3001e(0x0121, 0);

        assert_eq!(
            state.previous_scroll_words(),
            &[
                0x0024, 0x0024, 0x0000, 0x0012, 0x0024, 0x0036, 0x0048, 0x005A, 0x006C, 0x007E,
                0x0090, 0x00A2, 0x00B4, 0x00C6, 0x00D8, 0x00EA, 0x00FC, 0x010E, 0x0121,
            ],
            "ASM loc_3001E -> BackgroundScroll_ComputeShiftData seeds both time layers and the 17 fixed-point camera layers",
        );
    }

    #[test]
    fn shift_data_state_loc_30452_classifies_both_compare_loops_and_persists_new_words() {
        let state = ModeLevelTailShiftDataStateLoc30452::from_previous_scroll_words([
            0x0018, 0x0001, 0x0000, 0x0000, 0x0002, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000,
            0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000,
        ]);

        let advance = state
            .advance_loc_30452(0x0010, 0, 2)
            .expect("ASM sub_304B4 accepts a positive background width");

        assert_eq!(
            advance.work[0],
            ModeLevelTailShiftDataSlotLoc30452::OldGreaterThanNew,
            "the first two slots compare logical tile columns",
        );
        assert_eq!(
            advance.work[1],
            ModeLevelTailShiftDataSlotLoc30452::Unchanged,
            "different pixel words in the same first-loop tile column skip sub_304B4",
        );
        assert_eq!(
            advance.work[3],
            ModeLevelTailShiftDataSlotLoc30452::OldLessThanNew {
                width_subtractions: 20,
            },
            "the second loop compares exact words and retains every loc_304C2 width subtraction",
        );
        assert_eq!(
            advance.work[4],
            ModeLevelTailShiftDataSlotLoc30452::Unchanged
        );
        assert_eq!(
            advance.next_state.previous_scroll_words()[3],
            1,
            "the canonical state persists the new fixed-point layer word",
        );
    }

    #[test]
    fn shift_data_state_loc_30452_reproduces_the_three_published_oracle_vectors() {
        for (time_frames, camera_x_pos, expected) in [
            (
                0x02D3,
                0x0121,
                [
                    0x02F7, 0x018D, 0x0000, 0x0012, 0x0024, 0x0036, 0x0048, 0x005A, 0x006C, 0x007E,
                    0x0090, 0x00A2, 0x00B4, 0x00C6, 0x00D8, 0x00EA, 0x00FC, 0x010E, 0x0121,
                ],
            ),
            (
                0x02D9,
                0x0115,
                [
                    0x02FB, 0x018E, 0x0000, 0x0011, 0x0022, 0x0033, 0x0045, 0x0056, 0x0067, 0x0079,
                    0x008A, 0x009B, 0x00AD, 0x00BE, 0x00CF, 0x00E1, 0x00F2, 0x0103, 0x0115,
                ],
            ),
            (
                0x02DF,
                0x010B,
                [
                    0x0300, 0x0190, 0x0000, 0x0010, 0x0021, 0x0032, 0x0042, 0x0053, 0x0064, 0x0074,
                    0x0085, 0x0096, 0x00A6, 0x00B7, 0x00C8, 0x00D8, 0x00E9, 0x00FA, 0x010B,
                ],
            ),
        ] {
            assert_eq!(
                ModeLevelTailShiftDataStateLoc30452::from_camera_and_time_loc_30452(
                    camera_x_pos,
                    time_frames,
                )
                .previous_scroll_words(),
                &expected,
                "ASM loc_30452/loc_3048C register words from oracle helpers 1696/1702/1708",
            );
        }
    }

    #[test]
    fn shift_data_state_loc_30452_reproduces_the_three_published_branch_counts() {
        let cases = [
            (
                [
                    0x02F6, 0x018D, 0x0000, 0x0012, 0x0024, 0x0036, 0x0048, 0x005A, 0x006D, 0x007F,
                    0x0091, 0x00A3, 0x00B5, 0x00C8, 0x00DA, 0x00EC, 0x00FE, 0x0110, 0x0123,
                ],
                0x02D3,
                0x0121,
                11,
                None,
            ),
            (
                [
                    0x02FA, 0x018E, 0x0000, 0x0011, 0x0022, 0x0034, 0x0045, 0x0057, 0x0068, 0x007A,
                    0x008B, 0x009C, 0x00AE, 0x00BF, 0x00D1, 0x00E2, 0x00F4, 0x0105, 0x0117,
                ],
                0x02D9,
                0x0115,
                13,
                None,
            ),
            (
                [
                    0x02FF, 0x0190, 0x0000, 0x0010, 0x0021, 0x0032, 0x0043, 0x0053, 0x0064, 0x0075,
                    0x0086, 0x0096, 0x00A7, 0x00B8, 0x00C9, 0x00D9, 0x00EA, 0x00FB, 0x010C,
                ],
                0x02DF,
                0x010B,
                10,
                Some((0, 1)),
            ),
        ];

        for (previous, time_frames, camera_x_pos, expected_old_greater, expected_old_less) in cases
        {
            let advance = ModeLevelTailShiftDataStateLoc30452::from_previous_scroll_words(previous)
                .advance_loc_30452(camera_x_pos, time_frames, 0x005A)
                .expect("oracle Background_width is positive");
            assert_eq!(
                advance
                    .work
                    .iter()
                    .filter(|work| {
                        matches!(work, ModeLevelTailShiftDataSlotLoc30452::OldGreaterThanNew)
                    })
                    .count(),
                expected_old_greater,
                "ASM loc_304D8 branch count from oracle helpers 1696/1702/1708",
            );
            assert_eq!(
                advance.work.iter().enumerate().find_map(|(index, work)| {
                    if let ModeLevelTailShiftDataSlotLoc30452::OldLessThanNew {
                        width_subtractions,
                    } = work
                    {
                        Some((index, *width_subtractions))
                    } else {
                        None
                    }
                }),
                expected_old_less,
                "ASM loc_304C2 branch and width-loop count from oracle helpers 1696/1702/1708",
            );
        }
    }

    #[test]
    fn mode_level_tail_state_loc_720_clears_every_shared_wram_owner() {
        let mut state = ModeLevelTailStateLoc892 {
            camera_x_latch_f82c: 1,
            camera_y_latch_f82e: 2,
            background_row_latch_faa8: 3,
            shift_data_loc_30452: ModeLevelTailShiftDataStateLoc30452::from_previous_scroll_words(
                [4; 19],
            ),
            pending_palette_0_mask_f888: 5,
            pending_palette_1_mask_f88c: 6,
            background_upload_busy_fb49: 7,
        };

        state.clear_loc_720();

        assert_eq!(
            state,
            ModeLevelTailStateLoc892::power_on_zero(),
            "ASM loc_720 clears F82C/F82E, F888/F88C, FAA8, FB49 and Horiz_Scroll_Data",
        );
    }

    #[test]
    fn mode_level_tail_state_loc_2ffe8_preloads_camera_row_and_shift_history() {
        let mut state = ModeLevelTailStateLoc892::power_on_zero();

        state.preload_loc_2ffe8(0x0121, 0x0240, 0x0500, 0);

        assert_eq!(state.camera_latches_f82c_f82e(), (0x0121, 0x0240));
        assert_eq!(
            state.background_row_latch_faa8(),
            0x0012,
            "ASM sub_307D8 leaves FAA8 at current Camera_Y_pos>>5",
        );
        assert_eq!(
            state.shift_data_loc_30452().previous_scroll_words()[18],
            0x0121,
            "ASM loc_3001E runs BackgroundScroll_ComputeShiftData during preload",
        );
        assert_eq!(state.pending_palette_masks_f888_f88c(), (0, 0));
        assert_eq!(state.background_upload_busy_fb49(), 0);
    }

    #[test]
    fn mode_level_tail_state_loc_892_derives_the_three_published_work_ledgers() {
        let profile = [
            0x60, 0x60, 0x50, 0x50, 0x50, 0x70, 0x70, 0x40, 0x40, 0x40, 0x60, 0x60, 0x60, 0x60,
            0x60, 0x60, 0x40, 0x40, 0x40, 0x40, 0x50, 0x50, 0x50, 0x50, 0x50, 0x50, 0x70, 0x70,
            0x70,
        ];
        let cases = [
            (
                [
                    0x02F6, 0x018D, 0x0000, 0x0012, 0x0024, 0x0036, 0x0048, 0x005A, 0x006D, 0x007F,
                    0x0091, 0x00A3, 0x00B5, 0x00C8, 0x00DA, 0x00EC, 0x00FE, 0x0110, 0x0123,
                ],
                0x02D2,
                0x0121,
                29,
                12_852,
            ),
            (
                [
                    0x02FA, 0x018E, 0x0000, 0x0011, 0x0022, 0x0034, 0x0045, 0x0057, 0x0068, 0x007A,
                    0x008B, 0x009C, 0x00AE, 0x00BF, 0x00D1, 0x00E2, 0x00F4, 0x0105, 0x0117,
                ],
                0x02D8,
                0x0115,
                29,
                13_080,
            ),
            (
                [
                    0x02FF, 0x0190, 0x0000, 0x0010, 0x0021, 0x0032, 0x0043, 0x0053, 0x0064, 0x0075,
                    0x0086, 0x0096, 0x00A7, 0x00B8, 0x00C9, 0x00D9, 0x00EA, 0x00FB, 0x010C,
                ],
                0x02DE,
                0x010B,
                22,
                12_310,
            ),
        ];

        for (previous, time_frames, camera_x_pos, rendered_rows, raw_cycles) in cases {
            let state = ModeLevelTailStateLoc892 {
                camera_x_latch_f82c: camera_x_pos,
                camera_y_latch_f82e: 0,
                background_row_latch_faa8: 0,
                shift_data_loc_30452:
                    ModeLevelTailShiftDataStateLoc30452::from_previous_scroll_words(previous),
                pending_palette_0_mask_f888: 0,
                pending_palette_1_mask_f88c: 0,
                background_upload_busy_fb49: 0,
            };
            let advance = state.advance_loc_892(ModeLevelTailInputsLoc892 {
                camera_x_pos,
                camera_y_pos: 0,
                camera_stripe_cells_sub_30194: None,
                camera_column_cells_sub_30194: None,
                time_frames_before_loc_3038a: time_frames,
                game_mode_zero: false,
                special_effect_4: false,
                background_width: 0x005A,
                background_format_nonzero: false,
                background_no_scroll: false,
                background_theme: 1,
                bgscroll_profile: &profile,
                bgscroll_has_terminator: true,
            });
            let work = advance.work.expect("published target tail is classified");

            assert_eq!(
                work.background_loc_3038a
                    .piece_rows_sub_30744
                    .rendered_nonnegative_entries,
                rendered_rows,
                "ASM sub_30744 resource-$27 profile multiplicities select changed slots",
            );
            assert_eq!(
                mode_level_tail_raw_cycles_to_wait_for_vint_loc_892(&work)
                    .expect("published target has no external tail wait")
                    .get(),
                raw_cycles,
            );
            assert_eq!(advance.next_time_frames, time_frames + 1);
        }
    }

    #[test]
    fn life_icon_only_hud_path_has_source_derived_raw_cost_sub_44dc() {
        let life = HudDmaEventSub44DC::LifeIcon {
            frame_table_byte_offset: 4,
            wrapped_to_start: false,
        };
        assert_eq!(
            hud_life_icon_only_raw_cycles_sub_44dc(&[life]),
            Some(M68kCycleCount::from_raw(1_350)),
            "ASM sub_44DC loc_4586 -> sub_44D0 life-only path",
        );
        let wrapped_life = HudDmaEventSub44DC::LifeIcon {
            frame_table_byte_offset: 0,
            wrapped_to_start: true,
        };
        assert_eq!(
            hud_life_icon_only_raw_cycles_sub_44dc(&[wrapped_life]),
            Some(M68kCycleCount::from_raw(1_408)),
            "ASM loc_455E repeats the selector after the `$FF` sentinel",
        );
        assert_eq!(hud_life_icon_only_raw_cycles_sub_44dc(&[]), None);
        assert_eq!(
            hud_life_icon_only_raw_cycles_sub_44dc(&[
                life,
                HudDmaEventSub44DC::Clock {
                    frame_table_byte_offset: 8,
                    wrapped_to_start: false,
                },
            ]),
            None,
            "unclassified multi-upload paths must remain fail-closed",
        );
    }

    #[test]
    fn clock_icon_only_hud_path_has_source_derived_raw_cost_sub_44dc() {
        let clock = HudDmaEventSub44DC::Clock {
            frame_table_byte_offset: 4,
            wrapped_to_start: false,
        };
        assert_eq!(
            hud_clock_icon_only_raw_cycles_sub_44dc(&[clock]),
            Some(M68kCycleCount::from_raw(1_350)),
            "ASM loc_45B2 -> loc_45DA -> sub_44D0 clock-only path",
        );
        let wrapped_clock = HudDmaEventSub44DC::Clock {
            frame_table_byte_offset: 0,
            wrapped_to_start: true,
        };
        assert_eq!(
            hud_clock_icon_only_raw_cycles_sub_44dc(&[wrapped_clock]),
            Some(M68kCycleCount::from_raw(1_408)),
            "ASM loc_45B2 repeats the selector after the `$FF` sentinel",
        );
    }

    #[test]
    fn no_upload_hud_path_has_source_derived_raw_cost_sub_44dc() {
        // ASM sub_44DC: five skipped animation-counter sections and RTS.
        assert_eq!(
            hud_no_upload_raw_cycles_sub_44dc(&[]),
            Some(M68kCycleCount::from_raw(226))
        );
        assert_eq!(
            hud_no_upload_raw_cycles_sub_44dc(&[HudDmaEventSub44DC::Clock {
                frame_table_byte_offset: 4,
                wrapped_to_start: false,
            }]),
            None
        );
    }

    #[test]
    fn diamond_only_hud_path_has_source_derived_raw_cost_sub_44dc() {
        let diamond = HudDmaEventSub44DC::Diamond {
            frame_table_byte_offset: 4,
        };
        assert_eq!(
            hud_diamond_only_raw_cycles_sub_44dc(&[diamond]),
            Some(M68kCycleCount::from_raw(760)),
            "ASM sub_44DC loc_4626 -> sub_5D4A diamond-only path",
        );
        assert_eq!(hud_diamond_only_raw_cycles_sub_44dc(&[]), None);
        assert_eq!(
            hud_diamond_only_raw_cycles_sub_44dc(&[
                diamond,
                HudDmaEventSub44DC::Flag {
                    frame_table_byte_offset: 8,
                },
            ]),
            None,
            "unclassified multi-upload paths must remain fail-closed",
        );
    }

    #[test]
    fn flag_only_hud_path_has_source_derived_raw_cost_sub_44dc() {
        let flag = HudDmaEventSub44DC::Flag {
            frame_table_byte_offset: 20,
        };
        assert_eq!(
            hud_flag_only_raw_cycles_sub_44dc(&[flag]),
            Some(M68kCycleCount::from_raw(742)),
            "ASM sub_44DC loc_4630 -> loc_4680 -> sub_5DA6 flag-only path",
        );
        let timing = visit_hud_flag_only_sub_44dc(&[flag], |_| {})
            .expect("flag-only source path is classified");
        assert_eq!(timing.raw_cycles, M68kCycleCount::from_raw(742));
        let wrapped = HudDmaEventSub44DC::Flag {
            frame_table_byte_offset: 0,
        };
        assert_eq!(
            hud_flag_only_raw_cycles_sub_44dc(&[wrapped]),
            Some(M68kCycleCount::from_raw(800)),
            "ASM loc_4648 retries after the page-zero `$FF` sentinel",
        );
        assert_eq!(
            visit_hud_flag_only_sub_44dc(&[wrapped], |_| {})
                .expect("wrapped flag source path is classified")
                .raw_cycles,
            M68kCycleCount::from_raw(800),
        );
        assert_eq!(hud_flag_only_raw_cycles_sub_44dc(&[]), None);
        assert_eq!(
            hud_flag_only_raw_cycles_sub_44dc(&[
                HudDmaEventSub44DC::Diamond {
                    frame_table_byte_offset: 4,
                },
                flag,
            ]),
            None,
            "unclassified multi-upload paths must remain fail-closed",
        );
    }

    #[test]
    fn demo_read_joypad_raw_cost_follows_loc_c40_pointer_branch() {
        let demo = |counter_before_increment, advances_keypress_pointer| {
            ModeLevelReadJoypadWork::DemoPlaybackLocC22(Some(
                crate::ModeLevelDemoReadJoypadWorkLocC22 {
                    counter_before_increment,
                    advances_keypress_pointer,
                },
            ))
        };

        assert_eq!(
            read_joypad_raw_cycles(demo(723, true)),
            Some(M68kCycleCount::from_raw(1_266)),
            "ASM ReadJoypad loc_C40 falls through to the pointer update",
        );
        assert_eq!(
            read_joypad_raw_cycles(demo(724, false)),
            Some(M68kCycleCount::from_raw(1_230)),
            "ASM ReadJoypad loc_C40 branches directly to loc_C4E",
        );
        assert_eq!(
            read_joypad_raw_cycles(demo(723, false)),
            None,
            "inconsistent loc_C22 observations must remain fail-closed",
        );
        assert_eq!(
            read_joypad_raw_cycles(ModeLevelReadJoypadWork::DemoPlaybackLocC22(None)),
            None,
        );
        assert_eq!(
            read_joypad_raw_cycles(ModeLevelReadJoypadWork::FirstControllerLocB8c),
            None,
        );

        let mut controls = Vec::new();
        let advance = read_joypad_advance_with_scheduler(
            M68kRefreshPhase::new(28).expect("frame-1697 loc_81A phase"),
            demo(723, true),
            |_, _| Some(M68kCycleCount::from_raw(0)),
            |control| controls.push(control),
        )
        .expect("published Demo ReadJoypad path");
        assert_eq!(
            controls,
            [
                ReadJoypadControl::BeginZ80Bus,
                ReadJoypadControl::ReleaseZ80Bus,
            ],
            "ASM sub_914/sub_924 must bound both controller reads",
        );
        assert_eq!(advance.elapsed, M68kCycleCount::from_raw(1_314));
        assert_eq!(advance.refresh_events, 10);
        assert_eq!(
            advance.phase_after,
            M68kRefreshPhase::new(62).expect("frame-1697 loc_81E phase"),
            "the 110-instruction source path reaches the bounded oracle phase",
        );
    }

    #[test]
    fn demo_enemy_loader_early_return_has_source_derived_raw_cost_loc_36814() {
        assert_eq!(
            manage_enemy_loading_raw_cycles(
                ModeLevelEnemyLoadingPath::DemoNonKnightsReturnLoc3682a,
            ),
            Some(M68kCycleCount::from_raw(84)),
            "ASM Manage_EnemyLoading loc_36814 -> return_3682A",
        );
        assert_eq!(
            manage_enemy_loading_raw_cycles(ModeLevelEnemyLoadingPath::LayoutWalkLoc3682c),
            None,
            "variable layout walks must remain fail-closed",
        );
    }

    #[test]
    fn demo_enemy_loader_preserves_instruction_boundaries_for_gems_loc_36814() {
        let path = ModeLevelEnemyLoadingPath::DemoNonKnightsReturnLoc3682a;
        let mut ordinary_intervals = Vec::new();
        let ordinary = manage_enemy_loading_advance_with_scheduler_loc_36814(
            M68kRefreshPhase::new(58).expect("frame-1697 loc_810 phase"),
            path,
            |cycles| {
                ordinary_intervals.push(cycles);
                Some(M68kCycleCount::from_raw(0))
            },
        )
        .expect("published Demo enemy-loader branch");

        assert_eq!(
            ordinary_intervals,
            [20, 10, 12, 8, 16, 8, 2, 16, 8, 16].map(M68kCycleCount::from_raw),
            "ASM Mode_Level loc_810, j_Manage_EnemyLoading, and loc_36814 instruction order plus its pending refresh wait",
        );
        assert_eq!(ordinary.elapsed, M68kCycleCount::from_raw(116));
        assert_eq!(ordinary.refresh_events, 1);
        assert_eq!(
            ordinary.phase_after,
            M68kRefreshPhase::new(46).expect("frame-1697 loc_816 phase"),
            "bounded frame-1697 oracle reaches loc_816 after one refresh wait",
        );

        let mut intervals = Vec::new();
        let with_gems_wait = manage_enemy_loading_advance_with_scheduler_loc_36814(
            M68kRefreshPhase::new(58).expect("frame-1697 loc_810 phase"),
            path,
            |cycles| {
                intervals.push(cycles);
                Some(M68kCycleCount::from_raw(
                    u32::from(intervals.len() == 1) * 11,
                ))
            },
        )
        .expect("bounded chained hardware wait");
        assert_eq!(
            &intervals[..3],
            &[
                M68kCycleCount::from_raw(20),
                M68kCycleCount::from_raw(11),
                M68kCycleCount::from_raw(10),
            ],
            "a GEMS bank wait generated after loc_810 must drain before the jump-table instruction",
        );
        assert_eq!(with_gems_wait.elapsed, M68kCycleCount::from_raw(127));
        assert_eq!(with_gems_wait.refresh_events, 1);
        assert_eq!(
            with_gems_wait.phase_after,
            M68kRefreshPhase::new(57).expect("shifted loc_816 phase"),
        );
    }

    #[test]
    fn empty_platform_list_has_source_derived_post_hud_raw_costs() {
        let live_empty = ModeLevelPlatformWork {
            frozen_fa64: false,
            active_object_count: 0,
        };
        assert_eq!(
            execute_scripted_platforms_raw_cycles_sub_23ea(live_empty),
            Some(M68kCycleCount::from_raw(70)),
            "ASM Execute_ScriptedPlatforms loc_23F2 null sentinel",
        );
        let execute_advance = execute_scripted_platforms_advance_with_scheduler_sub_23ea(
            M68kRefreshPhase::new(60).expect("frame-1697 loc_826 phase"),
            live_empty,
            |_| Some(M68kCycleCount::from_raw(0)),
        )
        .expect("published empty scripted-platform path");
        assert_eq!(execute_advance.elapsed, M68kCycleCount::from_raw(90));
        assert_eq!(execute_advance.refresh_events, 1);
        assert_eq!(
            execute_advance.phase_after,
            M68kRefreshPhase::new(22).expect("frame-1697 loc_82A phase"),
        );
        assert_eq!(
            platforms_check_collision_raw_cycles_sub_2574(live_empty),
            Some(M68kCycleCount::from_raw(352)),
            "ASM Platforms_CheckCollision loc_25A0 null sentinel",
        );
        let collision_advance = platforms_check_collision_advance_with_scheduler_sub_2574(
            execute_advance.phase_after,
            live_empty,
            |_| Some(M68kCycleCount::from_raw(0)),
        )
        .expect("published empty platform-collision path");
        assert_eq!(collision_advance.elapsed, M68kCycleCount::from_raw(376));
        assert_eq!(collision_advance.refresh_events, 3);
        assert_eq!(
            collision_advance.phase_after,
            M68kRefreshPhase::new(14).expect("frame-1697 loc_82E phase"),
            "ASM sub_2574 selected path reaches the oracle boundary",
        );
        assert_eq!(
            make_sprites_from_platforms_raw_cycles_sub_2444(live_empty),
            Some(M68kCycleCount::from_raw(78)),
            "ASM Make_SpritesFromPlatforms loc_244C null sentinel",
        );

        let frozen = ModeLevelPlatformWork {
            frozen_fa64: true,
            active_object_count: 3,
        };
        assert_eq!(
            execute_scripted_platforms_raw_cycles_sub_23ea(frozen),
            Some(M68kCycleCount::from_raw(38)),
        );
        assert_eq!(
            platforms_check_collision_raw_cycles_sub_2574(frozen),
            Some(M68kCycleCount::from_raw(40)),
        );
        assert_eq!(
            make_sprites_from_platforms_raw_cycles_sub_2444(frozen),
            None,
            "ASM sub_2444 still walks the list while `$FA64` is set",
        );

        let live_nonempty = ModeLevelPlatformWork {
            frozen_fa64: false,
            active_object_count: 1,
        };
        assert_eq!(
            execute_scripted_platforms_raw_cycles_sub_23ea(live_nonempty),
            None,
        );
        assert_eq!(
            platforms_check_collision_raw_cycles_sub_2574(live_nonempty),
            None,
        );
    }

    #[test]
    fn wrapped_right_rejected_platform_loader_has_source_derived_raw_cost_loc_2744() {
        let work = ModeLevelPlatformLoaderWorkLoc2744::Placement {
            wrapped_to_start: true,
            range: ModeLevelPlatformLoaderRangeLoc2762::OutOfRangeRight,
            existing_slot: ModeLevelPlatformLoaderExistingSlotSub2728::Missing,
        };
        assert_eq!(
            manage_platform_loading_raw_cycles_loc_2744(work),
            Some(M68kCycleCount::from_raw(428)),
            "ASM loc_2744 terminator retry -> loc_2788 -> sub_2728 miss",
        );
        assert_eq!(
            manage_platform_loading_raw_cycles_loc_2744(
                ModeLevelPlatformLoaderWorkLoc2744::NoLayoutReturn,
            ),
            Some(M68kCycleCount::from_raw(42)),
        );

        let advance = manage_platform_loading_advance_with_scheduler_loc_2744(
            M68kRefreshPhase::new(118).expect("frame-1697 loc_822 phase"),
            work,
            |_| Some(M68kCycleCount::from_raw(0)),
        )
        .expect("published wrapped out-of-range platform path");
        assert_eq!(advance.elapsed, M68kCycleCount::from_raw(454));
        assert_eq!(advance.refresh_events, 4);
        assert_eq!(
            advance.phase_after,
            M68kRefreshPhase::new(60).expect("frame-1697 loc_826 phase"),
            "the 40-instruction loc_2744 path reaches the oracle boundary",
        );

        let first_gameplay = ModeLevelPlatformLoaderWorkLoc2744::Placement {
            wrapped_to_start: false,
            range: ModeLevelPlatformLoaderRangeLoc2762::OutOfRangeRight,
            existing_slot: ModeLevelPlatformLoaderExistingSlotSub2728::Missing,
        };
        assert_eq!(
            manage_platform_loading_raw_cycles_loc_2744(first_gameplay),
            Some(M68kCycleCount::from_raw(326)),
            "ASM frame-902 loc_2744 starts on the live record and skips the terminator retry",
        );
        assert_eq!(
            manage_platform_loading_advance_with_scheduler_loc_2744(
                M68kRefreshPhase::new(52).expect("oracle frame-902 loc_822 phase"),
                first_gameplay,
                |_| Some(M68kCycleCount::from_raw(0)),
            ),
            Some(M68kRefreshAdvance {
                elapsed: M68kCycleCount::from_raw(350),
                phase_after: M68kRefreshPhase::new(18).expect("no-GEMS frame-902 loc_826 phase"),
                refresh_events: 3,
            }),
            "ASM loc_822 -> loc_2744 -> loc_286E selected instructions",
        );
    }

    #[test]
    fn inactive_murder_wall_has_source_derived_raw_cost_sub_5e02() {
        assert_eq!(
            terrain_effects_raw_cycles_sub_5e02(false),
            Some(M68kCycleCount::from_raw(38)),
            "ASM sub_5E02 -> return_5F68",
        );
        assert_eq!(
            terrain_effects_raw_cycles_sub_5e02(true),
            None,
            "active murder-wall collision and sprite branches remain fail-closed",
        );

        let advance = terrain_effects_advance_with_scheduler_sub_5e02(
            M68kRefreshPhase::new(62).expect("frame-1697 loc_81E phase"),
            false,
            |_| Some(M68kCycleCount::from_raw(0)),
        )
        .expect("published inactive murder-wall path");
        assert_eq!(advance.elapsed, M68kCycleCount::from_raw(56));
        assert_eq!(advance.refresh_events, 0);
        assert_eq!(
            advance.phase_after,
            M68kRefreshPhase::new(118).expect("frame-1697 loc_822 phase"),
            "ASM sub_5E02 four-instruction path reaches the oracle boundary",
        );
    }

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
    fn coin_only_hud_raw_cycles_follow_loc_44f4_and_sub_44d0() {
        assert_eq!(
            hud_coin_only_raw_cycles_sub_44dc(&[HudDmaEventSub44DC::Coin {
                frame_table_byte_offset: 12,
            }]),
            Some(M68kCycleCount::from_raw(1_350)),
            "ASM frame 928 loc_44F4 -> loc_4534 -> sub_44D0 takes the non-wrapping coin path",
        );
        assert_eq!(
            hud_coin_only_raw_cycles_sub_44dc(&[HudDmaEventSub44DC::Coin {
                frame_table_byte_offset: 0,
            }]),
            Some(M68kCycleCount::from_raw(1_408)),
            "ASM loc_44F4 repeats the selector after the AniArt_Coin $FF terminator",
        );
        assert_eq!(hud_coin_only_raw_cycles_sub_44dc(&[]), None);
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

    #[test]
    fn load_in_game_tail_derives_source_cycles_to_stop_music_e14c8() {
        // ASM Load_InGame $11A12..$11A24 (kid.asm): j_sub_28FC clears 123
        // longwords, sub_44B0 seeds five HUD timers, the active
        // Init_Timer_and_Bonus_Flags path clears its timer/bonus state, and
        // j_StopMusic enters StopMusic at $E14C8.
        assert_eq!(
            load_in_game_stop_music_tail_raw_cycles(LoadInGameTimerBonusPath::Initialise),
            M68kCycleCount::from_raw(3_176),
        );
        assert_eq!(
            load_in_game_stop_music_tail_raw_cycles(
                LoadInGameTimerBonusPath::ReturnAlreadyInitialised,
            ),
            M68kCycleCount::from_raw(3_046),
        );
        let advance = load_in_game_stop_music_tail_advance(
            M68kRefreshPhase::new(114).expect("trace phase"),
            LoadInGameTimerBonusPath::Initialise,
        );
        assert_eq!(advance.elapsed, M68kCycleCount::from_raw(3_228));
        assert_eq!(advance.refresh_events, 26);
        assert_eq!(advance.phase_after, M68kRefreshPhase::new(14).unwrap());
    }

    #[test]
    fn load_in_game_tail_drains_gems_waits_before_the_next_instruction_e14c8() {
        // ASM Load_InGame $11A12..$11A24 and GEMS XFER68K $0285: a banked
        // read generated after the first JSR delays the next source
        // instruction, and a second read generated during that wait chains.
        let mut intervals = Vec::new();
        let advance = load_in_game_stop_music_tail_advance_with_scheduler(
            M68kRefreshPhase::new(114).expect("trace phase"),
            LoadInGameTimerBonusPath::Initialise,
            |cycles| {
                intervals.push(cycles);
                Some(M68kCycleCount::from_raw(match intervals.len() {
                    1 => 11,
                    2 => 7,
                    _ => 0,
                }))
            },
        )
        .expect("bounded Load_InGame tail");

        assert_eq!(
            &intervals[..4],
            &[
                M68kCycleCount::from_raw(18),
                M68kCycleCount::from_raw(13),
                M68kCycleCount::from_raw(7),
                M68kCycleCount::from_raw(10),
            ],
            "the first 11-cycle GEMS wait combines with the pending two-cycle refresh, then its chained wait completes before the next 10-cycle instruction",
        );
        assert_eq!(
            advance.elapsed.get(),
            load_in_game_stop_music_tail_raw_cycles(LoadInGameTimerBonusPath::Initialise).get()
                + 18
                + advance.refresh_events * 2,
        );
        assert_eq!(
            advance.phase_after.get(),
            u8::try_from((114 + advance.elapsed.get()) % 128).unwrap(),
        );
    }

    #[test]
    fn initialize_platforms_source_workload_excludes_fifo_and_vint_waits() {
        let workload = load_in_game_platform_initialisation_source_workload();

        assert_eq!(workload.horizontal_art.output_words, 192);
        assert_eq!(workload.vertical_art.output_words, 192);
        assert_eq!(workload.horizontal_art.decoder.get(), 14_570);
        assert_eq!(workload.vertical_art.decoder.get(), 14_528);
        assert_eq!(workload.horizontal_art.transfer.get(), 4_258);
        assert_eq!(workload.vertical_art.transfer.get(), 4_258);
        assert_eq!(
            workload.raw_cycles().get(),
            38_604,
            "ASM Initialize_Platforms sub_231C -> Decompress_Chunk sub_13684 -> \
             loc_14314 contributes 38,604 pure source cycles; the frame-866 \
             hardware ledger adds a separate 104-cycle VINT, 546 refresh \
             cycles, and 6,000 FIFO-halted ticks"
        );
    }
}
