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
    ModeLevelPlayerExecutionPath, ModeLevelPreHudWorkLedger, ModeLevelPrizeAnimationWake,
    ModeLevelPrizeBlockTailState, ModeLevelPrizeBlockWobbleWake,
    ModeLevelPrizeObjectExecutionSubE12, ModeLevelPrizeTailListWork, ModeLevelReadJoypadWork,
    ModeLevelRubberContactEffectLoc10DA4, ModeLevelShooterCollisionWork,
    ModeLevelTerrainSpecialScanSub1FA2, ModeLevelVerticalCollisionPath, ModeLevelWallCollisionPath,
};

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

/// Derive ASM `StopMusic`'s raw CPU schedule without fitting elapsed route time.
///
/// `Stop_z80` and `sub_E143A loc_E1460` each execute at least one `$A11100`
/// poll. Every additional taken `btst/bne` iteration adds 30 raw 68000 cycles;
/// refresh, VDP halts, and the Z80 instruction-boundary grant remain external
/// waits for the hardware scheduler.
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
    // These offsets come from the complete one-poll instruction stream; only
    // the explicit bus-ack loops can extend it.
    const BASE_WINDOWS: [(u32, u32); 7] = [
        (104, 214_838),
        (215_068, 215_238),
        (215_504, 215_674),
        (215_944, 216_298),
        (216_568, 216_922),
        (217_192, 217_546),
        (217_816, 218_170),
    ];
    const EXTRA_POLL_RAW_CYCLES: u32 = 30;

    let mut prior_extra = 0_u32;
    let mut windows = [GemsStopMusicBusWindow {
        request_complete_raw_offset: M68kCycleCount::from_raw(0),
        release_complete_raw_offset: M68kCycleCount::from_raw(0),
    }; 7];
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
        };
        prior_extra = prior_extra.checked_add(current_extra)?;
    }

    let driver_poll_extra = u32::from(polls.driver_copy_sub_e13da.checked_sub(1)?)
        .checked_mul(EXTRA_POLL_RAW_CYCLES)?;
    Some(GemsStopMusicTiming {
        raw_cycles: M68kCycleCount::from_raw(218_270_u32.checked_add(prior_extra)?),
        bus_windows: windows,
        reset_assert_complete_raw_offset: M68kCycleCount::from_raw(
            214_538_u32.checked_add(driver_poll_extra)?,
        ),
        reset_release_complete_raw_offset: M68kCycleCount::from_raw(
            214_858_u32.checked_add(driver_poll_extra)?,
        ),
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
    /// ASM `loc_300BA`: two VSRAM words then the `$380`-word H-scroll DMA.
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
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModeLevelTailCameraStripeSub30194 {
    /// ASM loc_301B4/loc_30246: neither camera tile axis needs streaming.
    BothTileAxesUnchanged,
    /// One or both axes stream mappings and may write through the VDP.
    Streaming,
}

/// One of the 19 cached/new comparisons in ASM
/// `BackgroundScroll_ComputeShiftData loc_30452/loc_3048C`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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
    pub fn preload_loc_2ffe8(&mut self, camera_x_pos: u16, camera_y_pos: u16, time_frames: u16) {
        self.camera_x_latch_f82c = camera_x_pos;
        self.camera_y_latch_f82e = camera_y_pos;
        self.background_row_latch_faa8 = (camera_y_pos as i16) >> 5;
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

    pub const fn background_upload_busy_fb49(&self) -> u8 {
        self.background_upload_busy_fb49
    }

    /// Gather/decide transition for ASM `Mode_Level loc_892`.
    pub fn advance_loc_892(
        self,
        inputs: ModeLevelTailInputsLoc892<'_>,
    ) -> ModeLevelTailAdvanceLoc892 {
        let mut next_state = self;
        let camera_stripe_sub_30194 = if (self.camera_x_latch_f82c >> 3)
            == (inputs.camera_x_pos >> 3)
            && ((self.camera_y_latch_f82e as i16) >> 3) == ((inputs.camera_y_pos as i16) >> 3)
        {
            ModeLevelTailCameraStripeSub30194::BothTileAxesUnchanged
        } else {
            ModeLevelTailCameraStripeSub30194::Streaming
        };
        // ASM sub_30194 loc_302E8 updates both latches after either axis path.
        next_state.camera_x_latch_f82c = inputs.camera_x_pos;
        next_state.camera_y_latch_f82e = inputs.camera_y_pos;

        let palette_sub_14c0 =
            if self.pending_palette_0_mask_f888 == 0 && self.pending_palette_1_mask_f88c == 0 {
                ModeLevelTailPaletteWorkSub14C0::Inactive
            } else {
                ModeLevelTailPaletteWorkSub14C0::Active
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

        let shift_layers = if !inputs.background_no_scroll
            && (inputs.camera_y_pos >> 2).is_multiple_of(8)
            && !matches!(inputs.background_theme, 5 | 9)
        {
            ModeLevelTailShiftLayersWork::NormalAlignedNoRipple { full_rows: 28 }
        } else {
            ModeLevelTailShiftLayersWork::Other
        };

        let background_upload_busy_fb49 = self.background_upload_busy_fb49 != 0;
        let mut piece_rows_sub_30744 = ModeLevelTailPieceRowsSub30744 {
            entries_before_sentinel: 0,
            rendered_nonnegative_entries: 0,
        };
        let mut piece_refresh_sub_307d8 = ModeLevelTailPieceRefreshSub307D8::Streaming;
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
                ModeLevelTailPieceRefreshSub307D8::Streaming
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
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModeLevelTailShiftLayersWork {
    /// ASM loc_3050A/loc_3056A: aligned 8-pixel rows without a theme ripple.
    NormalAlignedNoRipple {
        full_rows: u8,
    },
    Other,
}

/// ASM `sub_30744` piece-row scan work before its `$FF` sentinel.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ModeLevelTailPieceRowsSub30744 {
    pub entries_before_sentinel: u8,
    pub rendered_nonnegative_entries: u8,
}

/// ASM `sub_307D8` background-piece refresh path.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModeLevelTailPieceRefreshSub307D8 {
    /// ASM loc_30810 -> loc_308B4: current and cached background rows match.
    CachedRowUnchanged,
    Streaming,
}

/// ASM `sub_14C0` palette work at the end of `Mode_Level`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModeLevelTailPaletteWorkSub14C0 {
    /// Both pending palette masks `$FFFFF888/$FFFFF88C` are clear.
    Inactive,
    Active,
}

/// Source-typed work from ASM `Mode_Level loc_892` to the first
/// `WaitForVint $93E` poll.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ModeLevelTailWorkLoc892 {
    pub camera_stripe_sub_30194: ModeLevelTailCameraStripeSub30194,
    pub background_loc_3038a: ModeLevelTailBackgroundWorkLoc3038A,
    pub palette_sub_14c0: ModeLevelTailPaletteWorkSub14C0,
}

/// ASM `loc_3038A` gates and background-scroll work.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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
/// Streaming camera/background paths, ripple rows, and active palette work
/// remain unknown because they can introduce ordered VDP waits.
pub fn mode_level_tail_raw_cycles_to_wait_for_vint_loc_892(
    work: &ModeLevelTailWorkLoc892,
) -> Option<M68kCycleCount> {
    if work.camera_stripe_sub_30194 != ModeLevelTailCameraStripeSub30194::BothTileAxesUnchanged
        || work.palette_sub_14c0 != ModeLevelTailPaletteWorkSub14C0::Inactive
    {
        return None;
    }

    let background = &work.background_loc_3038a;
    if background.special_effect_4
        || background.game_mode_zero
        || background.background_upload_busy_fb49
        || background.background_format_nonzero
        || background.shift_layers
            != (ModeLevelTailShiftLayersWork::NormalAlignedNoRipple { full_rows: 28 })
        || background.piece_refresh_sub_307d8
            != ModeLevelTailPieceRefreshSub307D8::CachedRowUnchanged
        || background.piece_rows_sub_30744.entries_before_sentinel != 29
        || background.piece_rows_sub_30744.rendered_nonnegative_entries
            > background.piece_rows_sub_30744.entries_before_sentinel
    {
        return None;
    }

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

    // ASM loc_3038A: the 6,800-cycle fixed spine surrounds ShiftData,
    // aligned/no-ripple ShiftLayers (4,020), sub_30744's 29-entry scan
    // (2,360 + 82 per rendered row), and cached-row sub_307D8 (218).
    let background_cycles = 6_800_u32
        .checked_add(shift_data)?
        .checked_add(82_u32.checked_mul(u32::from(
            background.piece_rows_sub_30744.rendered_nonnegative_entries,
        ))?)?;

    // ASM loc_892: sub_30194 call/unchanged path is 238, inactive sub_14C0
    // call/path is 82, and Mode_Level return/MainGameLoop/WaitForVint prefix
    // is 60.
    Some(M68kCycleCount::from_raw(
        238_u32
            .checked_add(background_cycles)?
            .checked_add(82)?
            .checked_add(60)?,
    ))
}

/// Advance the no-external-wait ASM `loc_892 -> WaitForVint $93E` path
/// through the Genesis ROM-refresh recurrence.
pub fn mode_level_tail_advance_to_wait_for_vint_loc_892(
    phase: M68kRefreshPhase,
    work: &ModeLevelTailWorkLoc892,
) -> Option<M68kRefreshAdvance> {
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

/// Raw CPU work for ASM `ReadJoypad` (`kid.asm`) inside `Mode_Level`.
///
/// Refresh and the external waits in `sub_914`/`sub_924` are deliberately
/// excluded. Controller and incomplete Demo observations remain fail-closed.
pub fn read_joypad_raw_cycles(work: ModeLevelReadJoypadWork) -> Option<M68kCycleCount> {
    let ModeLevelReadJoypadWork::DemoPlaybackLocC22(Some(demo)) = work else {
        return None;
    };
    let advances_keypress_pointer = demo.counter_before_increment & 1 != 0;
    if advances_keypress_pointer != demo.advances_keypress_pointer {
        return None;
    }

    // ASM ReadJoypad loc_B70..loc_C9C: the shared Demo path costs 1,198
    // raw cycles. At loc_C40, a post-increment even counter takes the
    // 36-cycle loc_C44 pointer-update path before the shared loc_C4E edge
    // update. Selected-PC oracle traces pin both source paths independently.
    Some(M68kCycleCount::from_raw(if advances_keypress_pointer {
        1_234
    } else {
        1_198
    }))
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

/// Raw routine-body cost for ASM `Execute_ScriptedPlatforms` (`sub_23EA`).
pub fn execute_scripted_platforms_raw_cycles_sub_23ea(
    work: ModeLevelPlatformWork,
) -> Option<M68kCycleCount> {
    if work.frozen_fa64 {
        // ASM sub_23EA: `$FA64` takes the first branch to return_2442.
        return Some(M68kCycleCount::from_raw(38));
    }
    if work.active_object_count != 0 {
        return None;
    }

    // ASM loc_23F2 -> Execute_ScriptedPlatforms_Loop: the sentinel's null
    // link branches directly to return_2442.
    Some(M68kCycleCount::from_raw(70))
}

/// Raw routine-body cost for ASM `Platforms_CheckCollision` (`sub_2574`).
pub fn platforms_check_collision_raw_cycles_sub_2574(
    work: ModeLevelPlatformWork,
) -> Option<M68kCycleCount> {
    if work.frozen_fa64 {
        // ASM sub_2574: non-zero `$FA64` skips the saved-register frame.
        return Some(M68kCycleCount::from_raw(40));
    }
    if work.active_object_count != 0 {
        return None;
    }

    // ASM loc_257E -> loc_25A0 -> loc_2720: save the collision register set,
    // initialise scratch/hitbox state, observe the null sentinel, then restore.
    Some(M68kCycleCount::from_raw(352))
}

/// Raw routine-body cost for ASM `Make_SpritesFromPlatforms` (`sub_2444`).
pub fn make_sprites_from_platforms_raw_cycles_sub_2444(
    work: ModeLevelPlatformWork,
) -> Option<M68kCycleCount> {
    if work.active_object_count != 0 {
        return None;
    }

    // ASM loc_244C -> loc_256E: the null sentinel stores the unchanged next
    // sprite pointer and returns. `$FA64` does not gate this routine.
    Some(M68kCycleCount::from_raw(78))
}

/// Raw routine-body cost for classified ASM `Manage_PlatformLoading`
/// (`loc_2744`) work.
pub fn manage_platform_loading_raw_cycles_loc_2744(
    work: ModeLevelPlatformLoaderWorkLoc2744,
) -> Option<M68kCycleCount> {
    match work {
        // ASM loc_2744 first consumes the `$FFFF` terminator and retries from
        // offset zero. The first record then fails the right-edge test at
        // loc_2788; sub_2728 observes a null active-list sentinel.
        ModeLevelPlatformLoaderWorkLoc2744::Placement {
            wrapped_to_start: true,
            range: ModeLevelPlatformLoaderRangeLoc2762::OutOfRangeRight,
            existing_slot: ModeLevelPlatformLoaderExistingSlotSub2728::Missing,
        } => Some(M68kCycleCount::from_raw(428)),
        ModeLevelPlatformLoaderWorkLoc2744::NoLayoutReturn => Some(M68kCycleCount::from_raw(42)),
        _ => None,
    }
}

/// Raw routine-body cost for ASM murder-wall sprite work in `sub_5E02`.
pub fn terrain_effects_raw_cycles_sub_5e02(murder_wall_active: bool) -> Option<M68kCycleCount> {
    if murder_wall_active {
        return None;
    }

    // ASM sub_5E02: clear MurderWall_flag takes the first branch directly to
    // return_5F68 without touching Kid or the sprite table.
    Some(M68kCycleCount::from_raw(38))
}

/// Raw routine-body cost for ASM block-debris sprite compilation `sub_2A4C`.
pub fn block_debris_sprites_raw_cycles_sub_2a4c(
    work: ModeLevelBlockDebrisWorkSub2A4C,
) -> Option<M68kCycleCount> {
    if work.active_slot_count != 0 {
        return None;
    }

    // ASM sub_2A4C loc_2A56/loc_2BA6: each of the six fixed scratch slots
    // has a clear active byte, so every slot takes the skip-and-advance path.
    Some(M68kCycleCount::from_raw(408))
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

/// Raw routine-body cost for ASM Kid-projectile collision wrapper `sub_1D76`.
pub fn kid_projectile_collision_raw_cycles_sub_1d76(
    work: ModeLevelKidProjectileCollisionWorkSub1D76,
) -> Option<M68kCycleCount> {
    if work.kid_projectile_object_count != 0 || work.diamond_power_object_count != 0 {
        return None;
    }

    // ASM sub_1D76/sub_1D84: both `$FFFFF862` and `$FFFFF5A0` sentinels
    // have clear successor links, so each pass returns after its first load.
    Some(M68kCycleCount::from_raw(126))
}

/// Raw routine-body cost for an empty ASM `GfxObjects_Collision` (`sub_16F0`)
/// special-chain pass.
pub fn gfx_objects_collision_raw_cycles_sub_16f0(
    list: ModeLevelGfxCollisionListWork,
) -> Option<M68kCycleCount> {
    if list != ModeLevelGfxCollisionListWork::Empty {
        return None;
    }

    // ASM sub_16F0 loc_16FC/loc_17BE: save all registers, observe a null
    // successor in `4(a2)`, restore the frame, and return.
    Some(M68kCycleCount::from_raw(330))
}

/// Raw routine-body cost for an empty ASM `GfxObjects_CollisionKid`
/// (`sub_1B76`) special-chain pass.
pub fn gfx_objects_collision_kid_raw_cycles_sub_1b76(
    list: ModeLevelGfxCollisionListWork,
    work: &ModeLevelGfxCollisionWork,
) -> Option<M68kCycleCount> {
    if list != ModeLevelGfxCollisionListWork::Empty || work.kid_is_invulnerable {
        return None;
    }

    // ASM sub_1B76 loc_1B94/loc_1D00: load the Kid hitbox, observe a null
    // successor in `4(a2)`, then take the clear KidIsInvulnerable return.
    Some(M68kCycleCount::from_raw(166))
}

/// Raw routine-body cost for the kind-4 `$FFFFF86E` pass through ASM
/// `GfxObjects_CollisionKid` (`sub_1B76`).
pub fn gfx_objects_collision_kid_f86e_raw_cycles_sub_1b76(
    work: &ModeLevelGfxCollisionWork,
) -> Option<M68kCycleCount> {
    if work.kid_is_invulnerable {
        return None;
    }
    let ModeLevelGfxCollisionKidListWorkSub1B76::Classified { nodes } =
        &work.object_kind_four_list_f86e
    else {
        return None;
    };

    // ASM sub_1B76 entry through loc_1B94 is 94 cycles. Each node cost
    // includes loc_1B94's successor load and the complete rejecting sub-box
    // path back to that loop. The final null successor plus clear
    // KidIsInvulnerable epilogue at loc_1D00 costs 72 cycles.
    let raw_cycles = nodes.iter().try_fold(94_u32, |cycles, node| {
        let node_cycles = match node {
            // ASM loc_1B94 -> loc_1BC0: first CollisionSize word is zero.
            ModeLevelGfxCollisionKidNodeWorkSub1B76::CollisionRowsSentinelLoc1bc0 => 156,
            // ASM loc_1B94 -> loc_1BC8 -> loc_1BC0: reject the first box on
            // its left edge, then read the zero terminator.
            ModeLevelGfxCollisionKidNodeWorkSub1B76::UnflippedLeftEdgeBeyondKidRightLoc1bc8 => 206,
            // ASM loc_1B94 -> loc_1BD0 -> loc_1BC0: reject the first box on
            // its right edge, then read the zero terminator.
            ModeLevelGfxCollisionKidNodeWorkSub1B76::UnflippedRightEdgeBeforeKidLeftLoc1bd0 => 230,
            // ASM loc_1B94 -> loc_1BDE -> loc_1BC0: reject the first box on
            // its top edge, then read the zero terminator.
            ModeLevelGfxCollisionKidNodeWorkSub1B76::UnflippedTopEdgeBeyondKidBottomLoc1bde => 274,
        };
        cycles.checked_add(node_cycles)
    })?;

    raw_cycles.checked_add(72).map(M68kCycleCount::from_raw)
}

/// Raw routine-body cost for ASM `GfxObjects_MoveAndAnimate sub_1444`.
pub fn gfx_objects_move_and_animate_raw_cycles_sub_1444(
    work: &ModeLevelGfxMoveAnimateWorkSub1444,
) -> Option<M68kCycleCount> {
    let ModeLevelGfxMoveAnimateWorkSub1444::Classified { nodes } = work else {
        return None;
    };

    #[derive(Clone, Copy)]
    enum AnimationBranch {
        Inactive,
        Countdown,
        CountdownToOne,
        DispatchNormal,
        DispatchBackThenNormal,
        DispatchStop,
    }

    fn non_looping_animation_branch(
        durations: &[u16],
        target_tick: u16,
    ) -> Option<AnimationBranch> {
        let mut timer = 1_u16;
        let mut frame_index = 0_usize;
        let mut active = true;
        let mut branch = AnimationBranch::Inactive;

        for _ in 0..=target_tick {
            if !active {
                branch = AnimationBranch::Inactive;
                continue;
            }
            timer = timer.checked_sub(1)?;
            if timer != 0 {
                branch = if timer == 1 {
                    AnimationBranch::CountdownToOne
                } else {
                    AnimationBranch::Countdown
                };
                continue;
            }
            let Some(&duration) = durations.get(frame_index) else {
                active = false;
                timer = 1;
                branch = AnimationBranch::DispatchStop;
                continue;
            };
            timer = duration;
            frame_index += 1;
            branch = AnimationBranch::DispatchNormal;
        }

        Some(branch)
    }

    fn looping_diamond_animation_branch(target_tick: u16) -> Option<AnimationBranch> {
        const DURATIONS: [u16; 4] = [3; 4];
        let mut timer = 1_u16;
        let mut frame_index = 0_usize;
        let mut branch = AnimationBranch::Inactive;

        for _ in 0..=target_tick {
            timer = timer.checked_sub(1)?;
            if timer != 0 {
                branch = if timer == 1 {
                    AnimationBranch::CountdownToOne
                } else {
                    AnimationBranch::Countdown
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
                AnimationBranch::DispatchBackThenNormal
            } else {
                AnimationBranch::DispatchNormal
            };
        }

        Some(branch)
    }

    let raw_cycles = nodes.iter().try_fold(28_u32, |cycles, node| {
        let branch = match *node {
            ModeLevelGfxMoveAnimateNodeWorkSub1444::StationaryUnanimated => {
                AnimationBranch::Inactive
            }
            ModeLevelGfxMoveAnimateNodeWorkSub1444::PrizeWobble { allocation_age } => {
                // ASM stru_101B6: eight non-looping three-tick frames.
                non_looping_animation_branch(&[3; 8], allocation_age)?
            }
            ModeLevelGfxMoveAnimateNodeWorkSub1444::PrizeFlash { allocation_age } => {
                // ASM loc_102B8 is allocated after its Object-list insertion
                // point, so age zero has not run Init_Animation yet.
                if allocation_age == 0 {
                    AnimationBranch::Inactive
                } else {
                    // ASM stru_102DE: four non-looping frames with these
                    // exact animation_timer reloads.
                    non_looping_animation_branch(&[4, 6, 10, 4], allocation_age - 1)?
                }
            }
            ModeLevelGfxMoveAnimateNodeWorkSub1444::FloatingDiamond { allocation_age } => {
                // ASM stru_10D6E: four three-tick frames followed by the
                // loc_11AC signed-backstep dispatch.
                looping_diamond_animation_branch(allocation_age)?
            }
        };
        let node_cycles = match branch {
            // ASM loc_144E, sub_1120 and sub_113E: stationary node with
            // is_animated clear.
            AnimationBranch::Inactive => 218,
            // ASM loc_114C -> loc_1166 -> loc_1172.
            AnimationBranch::Countdown => 284,
            // ASM loc_1166 tests the next nonzero script opcode.
            AnimationBranch::CountdownToOne => 298,
            // ASM loc_1158 -> sub_1194, including j_nullsub_2/nullsub_2.
            AnimationBranch::DispatchNormal => 458,
            // ASM loc_1158 -> sub_11AC -> loc_1158 -> sub_1194.
            AnimationBranch::DispatchBackThenNormal => 554,
            // The target frontier never reaches the loc_1182 stop dispatch;
            // keep that mutable flag transition unclassified.
            AnimationBranch::DispatchStop => return None,
        };
        cycles.checked_add(node_cycles)
    })?;

    // ASM return_1460: the final null successor loop and RTS cost 42 cycles.
    raw_cycles.checked_add(42).map(M68kCycleCount::from_raw)
}

/// Raw routine-body cost for ASM `Make_SpritesFromGfxObjects sub_1492`.
pub fn make_sprites_from_gfx_objects_raw_cycles_sub_1492(
    work: &ModeLevelGfxSpriteWorkSub1492,
) -> Option<M68kCycleCount> {
    let ModeLevelGfxSpriteWorkSub1492::Classified { nodes } = work else {
        return None;
    };

    // ASM sub_1492 has 126 cycles outside its live-node iterations:
    // setup, the final null successor at loc_14A4, queue terminator, and RTS.
    let raw_cycles = nodes.iter().try_fold(126_u32, |cycles, node| {
        let node_cycles = match node {
            // ASM sub_1492 -> j_Make_SpriteFromGfxObject -> sub_11B6:
            // `$13(a3)` is clear and returns through return_129C.
            ModeLevelGfxSpriteNodeWorkSub1492::HiddenFlagClear => 120,
            // ASM loc_11E4 rejects the mapped object below the viewport
            // before any sub_13C0 sprite entry is emitted.
            ModeLevelGfxSpriteNodeWorkSub1492::VerticallyCulled => 268,
            // ASM loc_129E -> sub_13C0 emits one normal-orientation 16x16
            // mapping entry before the zero terminator.
            ModeLevelGfxSpriteNodeWorkSub1492::VisibleSingle16x16Normal => 926,
            // ASM loc_129E -> sub_13C0 emits two normal-orientation mapping
            // entries before the zero terminator.
            ModeLevelGfxSpriteNodeWorkSub1492::VisibleTwoSpriteNormal => 1_330,
        };
        cycles.checked_add(node_cycles)
    })?;
    Some(M68kCycleCount::from_raw(raw_cycles))
}

/// Raw routine-body cost for ASM special-terrain scan `sub_1FA2`.
pub fn terrain_special_scan_raw_cycles_sub_1fa2(
    work: ModeLevelTerrainSpecialScanSub1FA2,
) -> Option<M68kCycleCount> {
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

    // ASM loc_1FEE..loc_2008: each clear cell costs 46 raw cycles except the
    // final DBF iteration in each row (+4); each completed row advances the
    // layout pointer and row DBF. The fixed setup/return costs 294 cycles.
    let loop_cycles = u32::from(rows)
        .checked_mul(46_u32.checked_mul(u32::from(columns))?.checked_add(38)?)?
        .checked_add(4)?;
    Some(M68kCycleCount::from_raw(294_u32.checked_add(loop_cycles)?))
}

/// Raw routine-body cost for ASM shooter-support collision scan `sub_1F52`.
pub fn shooter_support_collision_raw_cycles_sub_1f52(
    work: ModeLevelShooterCollisionWork,
) -> Option<M68kCycleCount> {
    if work.active_support_slots != 0 {
        return None;
    }

    // ASM sub_1F52 loc_1F56: the first support-rectangle word is the null
    // sentinel, so the scan branches directly to return_1F8C.
    Some(M68kCycleCount::from_raw(50))
}

/// Raw routine-body cost for ASM Kid-versus-shooter scan `sub_219C`.
pub fn shooter_kid_collision_raw_cycles_sub_219c(
    work: ModeLevelShooterCollisionWork,
) -> Option<M68kCycleCount> {
    if work.active_moving_objects != 0 {
        return None;
    }

    // ASM sub_219C loc_21B8: save/load the Kid hitbox, observe the null
    // ShooterObject link, restore the register set, and return.
    Some(M68kCycleCount::from_raw(290))
}

/// Raw routine-body cost for ASM shooter-versus-Kid-projectile scan `sub_226A`.
pub fn shooter_projectile_collision_raw_cycles_sub_226a(
    work: ModeLevelShooterCollisionWork,
) -> Option<M68kCycleCount> {
    if work.active_moving_objects != 0 {
        return None;
    }

    // ASM sub_226A loc_226E: the first ShooterObject link is the null
    // sentinel, so no Kid-projectile list state is read before return_231A.
    Some(M68kCycleCount::from_raw(46))
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

        assert_eq!(timing.raw_cycles.get(), 218_270);
        assert_eq!(
            timing.bus_windows.map(|window| (
                window.request_complete_raw_offset.get(),
                window.release_complete_raw_offset.get(),
            )),
            [
                (104, 214_838),
                (215_068, 215_238),
                (215_504, 215_674),
                (215_944, 216_298),
                (216_568, 216_922),
                (217_192, 217_546),
                (217_816, 218_170),
            ],
            "ASM sub_E13DA owns the bus for the driver copy, then each sub_E149C/sub_E14AC call reacquires it separately"
        );
        assert_eq!(
            (
                timing.reset_assert_complete_raw_offset.get(),
                timing.reset_release_complete_raw_offset.get(),
            ),
            (214_538, 214_858),
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

        assert_eq!(timing.raw_cycles.get(), 218_360);
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
            (104, 214_868, 215_098, 215_268, 215_534, 215_764, 216_034,),
            "ASM Stop_z80/sub_E143A bus-poll repeats add 30 raw cycles without moving the already-completed bus request"
        );
        assert_eq!(
            (
                timing.reset_assert_complete_raw_offset.get(),
                timing.reset_release_complete_raw_offset.get(),
            ),
            (214_568, 214_888)
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
        assert_eq!(
            block_debris_sprites_raw_cycles_sub_2a4c(ModeLevelBlockDebrisWorkSub2A4C {
                active_slot_count: 0,
            }),
            Some(M68kCycleCount::from_raw(408))
        );
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
    fn empty_kid_projectile_collision_lists_take_two_early_returns() {
        let empty = ModeLevelKidProjectileCollisionWorkSub1D76 {
            kid_projectile_object_count: 0,
            diamond_power_object_count: 0,
        };
        assert_eq!(
            kid_projectile_collision_raw_cycles_sub_1d76(empty),
            Some(M68kCycleCount::from_raw(126))
        );
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
            (work(10, 2), 11_612),
            (work(11, 2), 12_538),
            (work(12, 0), 13_224),
        ] {
            assert_eq!(
                make_sprites_from_gfx_objects_raw_cycles_sub_1492(&work),
                Some(M68kCycleCount::from_raw(expected)),
                "ASM sub_1492 oracle frames 1697/1703/1709",
            );
        }
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
            camera_stripe_sub_30194: ModeLevelTailCameraStripeSub30194::Streaming,
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

        state.preload_loc_2ffe8(0x0121, 0x0240, 0);

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
            Some(M68kCycleCount::from_raw(1_234)),
            "ASM ReadJoypad loc_C40 falls through to the pointer update",
        );
        assert_eq!(
            read_joypad_raw_cycles(demo(724, false)),
            Some(M68kCycleCount::from_raw(1_198)),
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
        assert_eq!(
            platforms_check_collision_raw_cycles_sub_2574(live_empty),
            Some(M68kCycleCount::from_raw(352)),
            "ASM Platforms_CheckCollision loc_25A0 null sentinel",
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
}
