use crate::{
    HudDmaEventSub44DC, ModeLevelAirborneActionPath, ModeLevelAirborneHorizontalPath,
    ModeLevelAirborneVerticalPath, ModeLevelBlockDebrisWorkSub2A4C, ModeLevelCeilingScanReturn,
    ModeLevelDamageFlashWaitLocBc44, ModeLevelDemoExitWaitLoc119E0,
    ModeLevelDiamondPowerCompileWorkSub3F798, ModeLevelEnemyLoadingPath,
    ModeLevelExecuteObjectsWork, ModeLevelFloatingPrizeFallPathLoc103B2,
    ModeLevelGfxCollisionKidListWorkSub1B76, ModeLevelGfxCollisionKidNodeWorkSub1B76,
    ModeLevelGfxCollisionListWork, ModeLevelGfxCollisionWork,
    ModeLevelGfxMoveAnimateNodeWorkSub1444, ModeLevelGfxMoveAnimateWorkSub1444,
    ModeLevelGfxSpriteNodeWorkSub1492, ModeLevelGfxSpriteWorkSub1492,
    ModeLevelInputWorkerPathLoc7452, ModeLevelKidGfxTerrainCollision,
    ModeLevelKidProjectileCollisionWorkSub1D76, ModeLevelPlatformLoaderExistingSlotSub2728,
    ModeLevelPlatformLoaderRangeLoc2762, ModeLevelPlatformLoaderWorkLoc2744, ModeLevelPlatformWork,
    ModeLevelPlayerExecutionPath, ModeLevelPostHudPhase, ModeLevelPostHudWorkLedger,
    ModeLevelPreHudWorkLedger, ModeLevelPrizeAnimationWake, ModeLevelPrizeBlockTailState,
    ModeLevelPrizeBlockWobbleWake, ModeLevelPrizeObjectExecutionSubE12, ModeLevelPrizeTailListWork,
    ModeLevelReadJoypadWork, ModeLevelRubberContactEffectLoc10DA4, ModeLevelShooterCollisionWork,
    ModeLevelTerrainSpecialScanSub1FA2, ModeLevelVerticalCollisionPath, ModeLevelWallCollisionPath,
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
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ModeLevelVblankWorkBeforeSubE12 {
    /// Tile lengths read by `Transfer_SpriteAndKidToVRAM loc_AD4`.
    pub dynamic_art_tiles_loc_ad4: [u16; 2],
    pub palette_dma_sub_970: bool,
    pub scroll_path: ModeLevelScrollTransferPath,
    pub foreground_custom_queue_entries_sub_1596: u16,
    pub foreground_cell_queue_entries_sub_1596: u16,
    pub paused_sub_6874: bool,
    pub hud_path_sub_bdc0: ModeLevelHudBuildPath,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModeLevelScrollTransferPath {
    /// ASM `loc_300BA`: two VSRAM words then the `$380`-byte H-scroll DMA.
    NormalLoc300ba,
    StormLoc30110,
    SpecialLoc30158,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModeLevelHudBuildPath {
    /// ASM `loc_BE20 -> loc_BE0C -> loc_BE34 -> loc_BE4C`, with no timer
    /// rollover and unchanged lives/diamond counters.
    LocBe20ThenLocBe4cNoCounterUpdates,
    Other,
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
pub enum ModeLevelTailCameraStripeSub30194 {
    /// ASM loc_301B4/loc_30246: neither camera tile axis needs streaming.
    BothTileAxesUnchanged,
    /// ASM loc_301CE -> sub_3034A: upward, unsplit 21-longword row upload.
    UpwardHorizontalNoWrap {
        ordinary_theme_cells: u8,
        priority_theme_cells: u8,
        block_cells: u8,
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
) -> ModeLevelTailCameraStripeSub30194 {
    let previous_x_tile = previous_camera_x_pos >> 3;
    let previous_y_tile = (previous_camera_y_pos as i16) >> 3;
    let x_tile = camera_x_pos >> 3;
    let y_tile = (camera_y_pos as i16) >> 3;
    if previous_x_tile == x_tile && previous_y_tile == y_tile {
        return ModeLevelTailCameraStripeSub30194::BothTileAxesUnchanged;
    }

    // ASM loc_301CE/loc_301FE: an upward-only row at plane columns 0..11
    // emits one unsplit 21-longword sub_3034A pass. Other axis and wrap
    // combinations remain explicitly unclassified.
    let plane_cell_column = (x_tile >> 1) & 0x001F;
    let Some(cells) = horizontal_cells.filter(|cells| cells.len() == 21) else {
        return ModeLevelTailCameraStripeSub30194::StreamingOther;
    };
    if previous_x_tile != x_tile || y_tile >= previous_y_tile || plane_cell_column >= 0x000C {
        return ModeLevelTailCameraStripeSub30194::StreamingOther;
    }

    let mut ordinary_theme_cells = 0_u8;
    let mut priority_theme_cells = 0_u8;
    let mut block_cells = 0_u8;
    for &cell in cells {
        if (cell as i16) < 0 {
            block_cells += 1;
        } else if cell & 0x0100 != 0 {
            priority_theme_cells += 1;
        } else {
            ordinary_theme_cells += 1;
        }
    }

    ModeLevelTailCameraStripeSub30194::UpwardHorizontalNoWrap {
        ordinary_theme_cells,
        priority_theme_cells,
        block_cells,
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
        ModeLevelTailCameraStripeSub30194::UpwardHorizontalNoWrap {
            ordinary_theme_cells: 21,
            priority_theme_cells: 0,
            block_cells: 0,
        } => {
            // ASM sub_30194/loc_301CE -> sub_3034A. The fixed unsplit
            // upward-row spine plus the traced 21 ordinary cells is 2,320
            // raw cycles. Priority/block mixtures remain unclassified until
            // an oracle instruction trace fixes their complete path cost.
            Some(M68kCycleCount::from_raw(2_320))
        }
        ModeLevelTailCameraStripeSub30194::UpwardHorizontalNoWrap {
            ordinary_theme_cells: 20,
            priority_theme_cells: 0,
            block_cells: 1,
        } => {
            // ASM sub_30194/loc_3037C: oracle frame 902 takes one block
            // mapping branch after 20 ordinary cells. The complete caller,
            // jump stub, routine, and return cost 2,312 raw cycles.
            Some(M68kCycleCount::from_raw(2_312))
        }
        ModeLevelTailCameraStripeSub30194::StreamingOther => None,
        ModeLevelTailCameraStripeSub30194::UpwardHorizontalNoWrap { .. } => None,
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
    StreamingOther,
}

fn mode_level_tail_piece_refresh_work_sub_307d8(
    current_camera_row: i16,
    cached_camera_row: i16,
    background_width: u16,
    bgscroll_profile: &[u8],
    horiz_scroll_words: &[u16; 19],
) -> ModeLevelTailPieceRefreshSub307D8 {
    // ASM sub_307D8 only uses the current row directly while scrolling up.
    // Downward rows add $1C and remain a separate streaming shape.
    if cached_camera_row <= current_camera_row || current_camera_row < 0 || background_width == 0 {
        return ModeLevelTailPieceRefreshSub307D8::StreamingOther;
    }
    let Some(&selector) = bgscroll_profile.get(current_camera_row as usize) else {
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
    if source_wraps || plane_wraps {
        ModeLevelTailPieceRefreshSub307D8::StreamingOther
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
        ModeLevelTailPieceRefreshSub307D8::StreamingOther => None,
        ModeLevelTailPieceRefreshSub307D8::LinearNoWrap { .. } => None,
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
    if work.dynamic_art_tiles_loc_ad4 != [16, 4]
        || !work.palette_dma_sub_970
        || work.scroll_path != ModeLevelScrollTransferPath::NormalLoc300ba
        || work.foreground_custom_queue_entries_sub_1596 != 0
        || work.foreground_cell_queue_entries_sub_1596 != 0
        || work.paused_sub_6874
        || work.hud_path_sub_bdc0 != ModeLevelHudBuildPath::LocBe20ThenLocBe4cNoCounterUpdates
    {
        return None;
    }

    // ASM Mode_Level (kid.asm:901-908), with branches in sub_A5C, sub_970,
    // loc_300BA, sub_1596, sub_6874, and sub_BDC0. Instruction-level oracle
    // accounting confirms the identical path costs 3,415 raw cycles on all
    // three published target frames. VDP-owned one-cycle scheduler re-entries
    // are excluded and represented by the arbitration value below.
    Some(M68kCycleCount::from_raw(3_415))
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
        // ASM loc_7452..loc_74B0 (kid.asm:7586-7625): the clear timer,
        // Left-clear controller, demo-mode, and clear Start path totals 636
        // raw cycles. Right alone takes the same first-test branch.
        ModeLevelInputWorkerPathLoc7452::DemoNoLeftNoStartTimerClear => M68kCycleCount(636),
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
        || !shooter.cached_col_matches
        || !shooter.cached_row_matches
        || shooter.active_shooter_slots_before != 0
    {
        return None;
    }

    // ASM sub_DFB0 (kid.asm:17317) and loc_EAE4 (kid.asm:18858):
    // 994 raw cycles surround the inclusive row/column scan. Each row costs
    // 38 plus 40 per cell; a bit-15 non-type-$A cell adds the loc_E23A
    // 30-cycle branch before scanning continues.
    let sub_dfb0 = 994_u32
        .checked_add(
            u32::from(scan.row_count)
                .checked_mul(38_u32.checked_add(40_u32.checked_mul(u32::from(scan.col_count))?)?)?,
        )?
        .checked_add(
            30_u32.checked_mul(u32::from(scan.flagged_non_evanescent_cells_before_exit))?,
        )?;

    // ASM Mode_Level loc_7F4..loc_80C: the exact natural path outside
    // sub_DFB0 is 1,240 raw cycles: sub_21F8, DiamondPower_Run, sub_F7E0,
    // sub_6034, sub_F096, and their call/return shells.
    Some(M68kCycleCount(1_240_u32.checked_add(sub_dfb0)?))
}

/// Advance the typed post-Execute path through the Genesis refresh recurrence.
///
/// This interval performs no external or VDP accesses before `sub_44DC`.
pub fn post_execute_elapsed_cycles_before_sub_44dc(
    ledger: &ModeLevelPreHudWorkLedger,
    phase_before: M68kRefreshPhase,
) -> Option<M68kRefreshAdvance> {
    Some(phase_before.advance_rom(post_execute_raw_cycles_before_sub_44dc(ledger)?))
}

/// Raw CPU work for the classified life-icon-only `sub_44DC` path.
///
/// FIFO and refresh stalls are deliberately excluded. Other HUD command
/// combinations remain unclassified until their distinct ROM branches and
/// DMA ownership are typed.
pub fn hud_life_icon_only_raw_cycles_sub_44dc(
    commands: &[HudDmaEventSub44DC],
) -> Option<M68kCycleCount> {
    if !matches!(commands, [HudDmaEventSub44DC::LifeIcon { .. }]) {
        return None;
    }

    // ASM sub_44DC (kid.asm): 236 raw cycles reach completion of the first
    // loc_44D4 MOVE.L. The remaining 31 MOVE.L/taken-DBF pairs cost 930,
    // followed by final DBF 14, sub_44D0 RTS 16, three 42-cycle skipped HUD
    // sections, final store 12, and outer RTS 16.
    Some(M68kCycleCount::from_raw(1_350))
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
    if work.active_slot_count != 0 {
        return None;
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
        // ASM loc_144E live successor, GfxObject_Move sub_1120, and the
        // GfxObject_Animate sub_113E call.
        visit_all(&[16, 8, 4, 18, 12, 8, 12, 10, 16, 18])?;
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
            // The target frontier never reaches the loc_1182 stop dispatch;
            // keep that mutable flag transition unclassified.
            GfxAnimationBranchSub113E::DispatchStop => return None,
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
    const VERTICALLY_CULLED: &[u32] = &[
        16, 8, 4, 18, 10, 16, 12, 12, 12, 12, 18, 12, 4, 12, 12, 12, 8, 12, 10, 4, 8, 10, 16, 10,
    ];
    const VISIBLE_SINGLE: &[u32] = &[
        16, 8, 4, 18, 10, 16, 12, 12, 12, 12, 18, 12, 4, 12, 12, 12, 8, 12, 10, 4, 8, 12, 8, 12,
        12, 12, 8, 12, 8, 12, 16, 4, 12, 12, 16, 10, 12, 8, 8, 12, 10, 12, 12, 10, 12, 12, 10, 8,
        4, 4, 8, 8, 4, 4, 8, 8, 8, 4, 4, 10, 4, 8, 8, 4, 10, 4, 8, 8, 4, 4, 4, 8, 4, 12, 4, 18, 12,
        12, 12, 16, 20, 12, 8, 20, 4, 12, 4, 14, 4, 16, 8, 16, 16, 10,
    ];
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
                    // ASM loc_11E4 rejects the mapped object below the viewport
                    // before any sub_13C0 sprite entry is emitted.
                    ModeLevelGfxSpriteNodeWorkSub1492::VerticallyCulled => VERTICALLY_CULLED,
                    // ASM loc_129E -> sub_13C0 emits one normal-orientation 16x16
                    // mapping entry before the zero terminator.
                    ModeLevelGfxSpriteNodeWorkSub1492::VisibleSingle16x16Normal => VISIBLE_SINGLE,
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
    mut phase: M68kRefreshPhase,
    ledger: &ModeLevelPostHudWorkLedger,
    mut advance_hardware: impl FnMut(M68kCycleCount, bool) -> Option<M68kCycleCount>,
    mut apply_read_joypad_control: impl FnMut(ReadJoypadControl),
) -> Option<M68kRefreshAdvance> {
    // Validate source order and every typed branch before mutating external
    // hardware through the instruction callbacks below.
    mode_level_post_hud_phase_raw_cycles_to_loc_892(ledger)?;

    let mut elapsed = 0_u32;
    let mut refresh_events = 0_u32;
    macro_rules! apply_phase {
        ($advance:expr) => {{
            let advance = $advance?;
            elapsed = elapsed.checked_add(advance.elapsed.get())?;
            refresh_events = refresh_events.checked_add(advance.refresh_events)?;
            phase = advance.phase_after;
        }};
    }
    let collision = &ledger.gfx_collision_work;

    {
        let mut ordinary_hardware = |cycles| advance_hardware(cycles, false);
        apply_phase!(manage_enemy_loading_advance_with_scheduler_loc_36814(
            phase,
            ledger.enemy_loading_path,
            &mut ordinary_hardware,
        ));
        apply_phase!(
            gfx_objects_move_and_animate_advance_with_scheduler_sub_1444(
                phase,
                &ledger.gfx_move_animate_work_sub_1444,
                &mut ordinary_hardware,
            )
        );
    }
    apply_phase!(read_joypad_advance_with_scheduler(
        phase,
        ledger.read_joypad_work,
        &mut advance_hardware,
        &mut apply_read_joypad_control,
    ));
    let mut ordinary_hardware = |cycles| advance_hardware(cycles, false);
    apply_phase!(terrain_effects_advance_with_scheduler_sub_5e02(
        phase,
        ledger.murder_wall_active_sub_5e02,
        &mut ordinary_hardware,
    ));
    apply_phase!(manage_platform_loading_advance_with_scheduler_loc_2744(
        phase,
        ledger.platform_loader_work?,
        &mut ordinary_hardware,
    ));
    apply_phase!(execute_scripted_platforms_advance_with_scheduler_sub_23ea(
        phase,
        ledger.platform_work,
        &mut ordinary_hardware,
    ));
    apply_phase!(platforms_check_collision_advance_with_scheduler_sub_2574(
        phase,
        ledger.platform_work,
        &mut ordinary_hardware,
    ));
    apply_phase!(gfx_objects_collision_advance_with_scheduler_sub_16f0(
        phase,
        collision.object_kind_two_list_f86a,
        &mut ordinary_hardware,
    ));
    apply_phase!(gfx_objects_collision_advance_with_scheduler_sub_16f0(
        phase,
        collision.kid_projectile_list_f866,
        &mut ordinary_hardware,
    ));
    apply_phase!(mode_level_latch_damage_state_advance_loc_83e(
        phase,
        &mut ordinary_hardware,
    ));
    apply_phase!(gfx_objects_collision_kid_advance_with_scheduler_sub_1b76(
        phase,
        collision.kid_projectile_list_f866,
        collision,
        &mut ordinary_hardware,
    ));
    apply_phase!(shooter_support_collision_advance_with_scheduler_sub_1f52(
        phase,
        ledger.shooter_collision_work,
        &mut ordinary_hardware,
    ));
    apply_phase!(shooter_kid_collision_advance_with_scheduler_sub_219c(
        phase,
        ledger.shooter_collision_work,
        &mut ordinary_hardware,
    ));
    apply_phase!(
        shooter_projectile_collision_advance_with_scheduler_sub_226a(
            phase,
            ledger.shooter_collision_work,
            &mut ordinary_hardware,
        )
    );
    apply_phase!(gfx_objects_collision_kid_advance_with_scheduler_sub_1b76(
        phase,
        collision.object_kind_two_list_f86a,
        collision,
        &mut ordinary_hardware,
    ));
    apply_phase!(
        gfx_objects_collision_kid_f86e_advance_with_scheduler_sub_1b76(
            phase,
            collision,
            &mut ordinary_hardware,
        )
    );
    apply_phase!(kid_projectile_collision_advance_with_scheduler_sub_1d76(
        phase,
        ledger.kid_projectile_collision_work_sub_1d76,
        &mut ordinary_hardware,
    ));
    apply_phase!(terrain_special_scan_advance_with_scheduler_sub_1fa2(
        phase,
        ledger.terrain_special_scan_sub_1fa2?,
        &mut ordinary_hardware,
    ));
    apply_phase!(block_debris_sprites_advance_with_scheduler_sub_2a4c(
        phase,
        ledger.block_debris_work_sub_2a4c,
        &mut ordinary_hardware,
    ));
    apply_phase!(
        make_sprites_from_gfx_objects_advance_with_scheduler_sub_1492(
            phase,
            &ledger.gfx_sprite_work_sub_1492,
            &mut ordinary_hardware,
        )
    );
    apply_phase!(
        diamond_power_compile_sprites_advance_with_scheduler_sub_3f798(
            phase,
            ledger.diamond_power_compile_work_sub_3f798,
            &mut ordinary_hardware,
        )
    );
    apply_phase!(make_sprites_from_platforms_advance_with_scheduler_sub_2444(
        phase,
        ledger.platform_work,
        &mut ordinary_hardware,
    ));

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
            dynamic_art_tiles_loc_ad4: [16, 4],
            palette_dma_sub_970: true,
            scroll_path: ModeLevelScrollTransferPath::NormalLoc300ba,
            foreground_custom_queue_entries_sub_1596: 0,
            foreground_cell_queue_entries_sub_1596: 0,
            paused_sub_6874: false,
            hud_path_sub_bdc0: ModeLevelHudBuildPath::LocBe20ThenLocBe4cNoCounterUpdates,
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
            None
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
    }

    #[test]
    fn clear_special_terrain_scan_derives_inclusive_cell_walk() {
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
                ordinary_theme_cells: 21,
                priority_theme_cells: 0,
                block_cells: 0,
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
                ordinary_theme_cells: 20,
                priority_theme_cells: 0,
                block_cells: 1,
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
        };
        assert_eq!(
            hud_life_icon_only_raw_cycles_sub_44dc(&[life]),
            Some(M68kCycleCount::from_raw(1_350)),
            "ASM sub_44DC loc_4586 -> sub_44D0 life-only path",
        );
        assert_eq!(hud_life_icon_only_raw_cycles_sub_44dc(&[]), None);
        assert_eq!(
            hud_life_icon_only_raw_cycles_sub_44dc(&[
                life,
                HudDmaEventSub44DC::Clock {
                    frame_table_byte_offset: 8,
                },
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

        assert_eq!(
            manage_platform_loading_raw_cycles_loc_2744(
                ModeLevelPlatformLoaderWorkLoc2744::Placement {
                    wrapped_to_start: false,
                    range: ModeLevelPlatformLoaderRangeLoc2762::OutOfRangeRight,
                    existing_slot: ModeLevelPlatformLoaderExistingSlotSub2728::Missing,
                },
            ),
            None,
            "the non-wrapped source path has a distinct cost",
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
