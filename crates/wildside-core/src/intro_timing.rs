use crate::M68kCycleCount;

/// ASM sub_1C246 (kid.asm): `bclr #7,(Ctrl_1_Pressed).w` selected-PC trace cost.
const TITLE_CONTROLLER_START_POLL_CYCLES: u32 = 20;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct IntroSourceTiming {
    pub instruction_count: u32,
    pub raw_cycles: M68kCycleCount,
}

/// Visit `Load_IntroSequence1` from the return of `sub_1B850` through its RTS.
///
/// ASM Load_IntroSequence1 $1AE5C..$1AE6C calls `Allocate_ObjectSlot sub_DAC`
/// with an empty execution list. The visitor retains every instruction
/// boundary because refresh and GEMS bank waits are sampled between them.
pub fn visit_load_intro_sequence1_controller_allocation(
    mut visit: impl FnMut(u32),
) -> IntroSourceTiming {
    let mut timing = IntroSourceTiming {
        instruction_count: 0,
        raw_cycles: M68kCycleCount::from_raw(0),
    };
    let mut emit = |cycles: u32| {
        visit(cycles);
        timing.instruction_count += 1;
        timing.raw_cycles = M68kCycleCount::from_raw(
            timing
                .raw_cycles
                .get()
                .checked_add(cycles)
                .expect("intro source timing fits u32"),
        );
    };

    // ASM Load_IntroSequence1 $1AE5C: move.w #$FFFF,a0; jsr thunk.
    for cycles in [8, 18, 10] {
        emit(cycles);
    }
    visit_empty_list_allocate_object_slot_instructions(&mut emit);
    // ASM Load_IntroSequence1 $1AE64: install sub_1B8F4 and return.
    for cycles in [24, 16] {
        emit(cycles);
    }

    timing
}

/// Visit the outer loader return through the first `WaitForVint` flag poll.
///
/// ASM `LoadGameModeData` $72C returns from `Load_IntroSequence1`, clears
/// `$FFFF0280`, and returns to `MainGameLoop` $738. The loop then calls
/// `WaitForVint` $93A and clears its completion flag before `loc_93E`.
pub fn visit_intro_controller_to_wait_for_vint_poll(
    mut visit: impl FnMut(u32),
) -> IntroSourceTiming {
    let mut timing = IntroSourceTiming {
        instruction_count: 0,
        raw_cycles: M68kCycleCount::from_raw(0),
    };

    // ASM LoadGameModeData $72C, MainGameLoop $738, WaitForVint $93A.
    for cycles in [20, 16, 18, 16] {
        visit(cycles);
        timing.instruction_count += 1;
        timing.raw_cycles = M68kCycleCount::from_raw(
            timing
                .raw_cycles
                .get()
                .checked_add(cycles)
                .expect("intro handoff timing fits u32"),
        );
    }

    timing
}

/// Visit `Load_TitleScreen`'s three object allocations through the first
/// `WaitForVint` poll.
///
/// ASM `Load_TitleScreen` installs the menu, KID-text, and CHAMELEON-text
/// objects at equal priority. The first allocation sees an empty list; each
/// later allocation compares equal with the current head and inserts before
/// it after one `loc_DE0` iteration.
pub fn visit_title_screen_allocations_to_wait_for_vint_poll(
    mut visit: impl FnMut(u32),
) -> IntroSourceTiming {
    let mut timing = IntroSourceTiming {
        instruction_count: 0,
        raw_cycles: M68kCycleCount::from_raw(0),
    };
    let mut emit = |cycles: u32| {
        visit(cycles);
        timing.instruction_count += 1;
        timing.raw_cycles = M68kCycleCount::from_raw(
            timing
                .raw_cycles
                .get()
                .checked_add(cycles)
                .expect("title-screen allocation timing fits u32"),
        );
    };

    for allocation in 0..3 {
        // ASM Load_TitleScreen: move.w #$FFFF,a0; jsr thunk.
        for cycles in [8, 18, 10] {
            emit(cycles);
        }
        if allocation == 0 {
            visit_empty_list_allocate_object_slot_instructions(&mut emit);
        } else {
            // ASM sub_DAC setup, 29-longword clear, and fixed fields.
            for cycles in [40, 4, 16, 28, 4, 4, 8] {
                emit(cycles);
            }
            for index in 0..29 {
                emit(12);
                emit(if index == 28 { 14 } else { 10 });
            }
            for cycles in [16, 16, 16, 8, 12] {
                emit(cycles);
            }
            // ASM loc_DE0: compare equal with the current list head, link
            // the new slot before it, restore registers, and return.
            for cycles in [16, 8, 4, 4, 12, 8, 4, 28, 16, 44, 16] {
                emit(cycles);
            }
        }
        // Install this object's code pointer.
        emit(24);
    }

    // Return through LoadGameModeData and enter WaitForVint's first poll.
    for cycles in [16, 20, 16, 18, 16] {
        emit(cycles);
    }

    timing
}

/// Visit a completed title frame through the next `WaitForVint` flag poll.
///
/// ASM `Mode_Standard` $7A4 returns to `MainGameLoop` $744, branches to
/// $738, and calls `WaitForVint` $93A. The visitor stops after clearing
/// `V_Int_Done`, immediately before the first $93E poll instruction.
pub fn visit_title_frame_return_to_wait_for_vint_poll(
    mut visit: impl FnMut(u32),
) -> IntroSourceTiming {
    let mut timing = IntroSourceTiming {
        instruction_count: 0,
        raw_cycles: M68kCycleCount::from_raw(0),
    };

    // ASM Mode_Standard $7A4, MainGameLoop $744/$738, WaitForVint $93A.
    for cycles in [16, 10, 18, 16] {
        visit(cycles);
        timing.instruction_count += 1;
        timing.raw_cycles = M68kCycleCount::from_raw(
            timing
                .raw_cycles
                .get()
                .checked_add(cycles)
                .expect("title frame handoff timing fits u32"),
        );
    }

    timing
}

/// Visit a `BSR.W Do_Nothing` call through its return.
///
/// ASM `Do_Nothing` $956 runs 612 taken DBF iterations and one terminal
/// iteration before returning. Callers include `Mode_Standard` $77A and
/// `Pal_FadeOut loc_28C6` $28CA.
pub fn visit_do_nothing_call(mut visit: impl FnMut(u32)) -> IntroSourceTiming {
    let mut timing = IntroSourceTiming {
        instruction_count: 0,
        raw_cycles: M68kCycleCount::from_raw(0),
    };
    let mut emit = |cycles: u32| {
        visit(cycles);
        timing.instruction_count += 1;
        timing.raw_cycles = M68kCycleCount::from_raw(
            timing
                .raw_cycles
                .get()
                .checked_add(cycles)
                .expect("Do_Nothing source timing fits u32"),
        );
    };

    // ASM Mode_Standard $77A / Pal_FadeOut $28CA: BSR.W Do_Nothing;
    // save d0 and load the DBF count.
    for cycles in [18, 8, 8] {
        emit(cycles);
    }
    for iteration in 0..=612 {
        emit(if iteration == 612 { 14 } else { 10 });
    }
    // ASM Do_Nothing $960: restore d0 and return to the caller.
    for cycles in [8, 16] {
        emit(cycles);
    }

    timing
}

/// Visit `Mode_Standard` $77A through completion of `Do_Nothing`.
pub fn visit_mode_standard_do_nothing(visit: impl FnMut(u32)) -> IntroSourceTiming {
    visit_do_nothing_call(visit)
}

/// Visit the title's empty foreground-update queues through `sub_1596`.
///
/// Both next-slot pointers are zero at this title boundary, so the source
/// skips both VDP writers and resets the producer pointers at `loc_1686`.
pub fn visit_mode_standard_empty_foreground_queues_sub_1596(
    mut visit: impl FnMut(u32),
) -> IntroSourceTiming {
    let mut timing = IntroSourceTiming {
        instruction_count: 0,
        raw_cycles: M68kCycleCount::from_raw(0),
    };
    for cycles in [18, 8, 12, 10, 12, 10, 28, 28, 16] {
        visit(cycles);
        timing.instruction_count += 1;
        timing.raw_cycles = M68kCycleCount::from_raw(
            timing
                .raw_cycles
                .get()
                .checked_add(cycles)
                .expect("empty sub_1596 source timing fits u32"),
        );
    }
    timing
}

/// Visit the natural gameplay suffix from ASM `Mode_Level $7D6` through the
/// `Execute_Objects sub_E12` call instruction.
///
/// The empty gameplay foreground queues retain their producer-base sentinels,
/// so `sub_1596` takes the setup path through `loc_15C8` before resetting both
/// pointers. The remaining typed path is unpaused `sub_6874` and
/// `Make_SpriteAttr_HUD loc_BE20 -> loc_BE0C -> loc_BE34 -> loc_BE4C`, with
/// no timer rollover or score/life redraw.
pub fn visit_mode_level_vblank_suffix_after_scroll_dma(
    work: crate::ModeLevelVblankWorkBeforeSubE12,
    mut visit: impl FnMut(u32),
) -> Option<IntroSourceTiming> {
    let single_visible_theme_cell = work.foreground_cell_queue_sub_1596.path(0)
        == Some(crate::ModeLevelForegroundCellQueuePathSub1596::VisibleTheme)
        && work.foreground_cell_queue_sub_1596.path(1).is_none();
    if work.foreground_custom_queue_entries_sub_1596 != 0
        || (!work.foreground_cell_queue_sub_1596.is_empty() && !single_visible_theme_cell)
        || work.paused_sub_6874
        || !matches!(
            work.hud_path_sub_bdc0,
            crate::ModeLevelHudBuildPath::LocBe20DirectInitialCounterUpdates
                | crate::ModeLevelHudBuildPath::LocBe20DirectStableCounters
                | crate::ModeLevelHudBuildPath::LocBe20ThenLocBe4cStableCounters
        )
    {
        return None;
    }

    let mut timing = IntroSourceTiming {
        instruction_count: 0,
        raw_cycles: M68kCycleCount::from_raw(0),
    };
    let mut emit = |cycles: u32| {
        visit(cycles);
        timing.instruction_count += 1;
        timing.raw_cycles = M68kCycleCount::from_raw(
            timing
                .raw_cycles
                .get()
                .checked_add(cycles)
                .expect("Mode_Level VBlank suffix source timing fits u32"),
        );
    };

    if single_visible_theme_cell
        && work.hud_path_sub_bdc0 == crate::ModeLevelHudBuildPath::LocBe20DirectStableCounters
    {
        // ASM sub_1596 loc_15D2 -> loc_1614, published TAS frame 990: one
        // in-window non-block cell uses Addr_ThemeMappings, reaches
        // loc_1680, then the stable HUD path returns at `sub_E12`.
        const SINGLE_VISIBLE_THEME_CELL_STABLE_SUFFIX: &[u8] = &[
            18, 8, 16, 8, 4, 12, 4, 8, 8, 12, 10, 8, 8, 4, 16, 16, 12, 8, 12, 4, 12, 12, 16, 12,
            12, 14, 4, 8, 12, 14, 4, 8, 4, 8, 8, 4, 8, 4, 8, 4, 8, 4, 8, 4, 4, 8, 22, 8, 4, 8, 8,
            8, 8, 8, 8, 12, 26, 8, 8, 8, 26, 8, 8, 10, 4, 16, 24, 24, 16, 18, 12, 8, 16, 18, 10,
            12, 16, 12, 8, 12, 12, 12, 12, 4, 14, 12, 8, 12, 10, 8, 4, 8, 4, 4, 10, 8, 12, 8, 12,
            8, 4, 8, 10, 12, 16, 12, 12, 12, 12, 16, 10, 12, 10, 12, 8, 10, 12, 10, 12, 12, 10, 16,
            18,
        ];
        for &cycles in SINGLE_VISIBLE_THEME_CELL_STABLE_SUFFIX {
            emit(u32::from(cycles));
        }
        return Some(timing);
    }
    if single_visible_theme_cell
        && work.hud_path_sub_bdc0 == crate::ModeLevelHudBuildPath::LocBe20ThenLocBe4cStableCounters
    {
        // ASM sub_1596 loc_15D2 -> loc_1614 followed by sub_BDC0
        // loc_BE20 -> loc_BE0C -> loc_BE34 -> loc_BE4C, published TAS frame
        // 1367. Breaking a prize block queues one visible theme cell while
        // Cyclone's three-slot hitpoint path builds the HUD. The full-PC
        // journal preserves all 137 instruction boundaries.
        const SINGLE_VISIBLE_THEME_CELL_SECOND_HITPOINT_SUFFIX: &[u8] = &[
            18, 8, 16, 8, 4, 12, 4, 8, 8, 12, 10, 8, 8, 4, 16, 16, 12, 8, 12, 4, 12, 12, 16, 12,
            12, 14, 4, 8, 12, 14, 4, 8, 4, 8, 8, 4, 8, 4, 8, 4, 8, 4, 8, 4, 4, 8, 22, 8, 4, 8, 8,
            8, 8, 8, 8, 12, 26, 8, 8, 8, 26, 8, 8, 10, 4, 16, 24, 24, 16, 18, 12, 8, 16, 18, 10,
            12, 16, 12, 8, 12, 12, 12, 12, 4, 14, 12, 8, 12, 10, 8, 4, 8, 4, 4, 10, 8, 12, 8, 12,
            8, 4, 10, 4, 8, 4, 4, 8, 8, 10, 4, 10, 8, 12, 8, 12, 8, 10, 12, 16, 12, 12, 12, 12, 16,
            10, 12, 10, 12, 8, 10, 12, 10, 12, 12, 10, 16, 18,
        ];
        for &cycles in SINGLE_VISIBLE_THEME_CELL_SECOND_HITPOINT_SUFFIX {
            emit(u32::from(cycles));
        }
        return Some(timing);
    }
    if single_visible_theme_cell {
        return None;
    }

    if work.hud_path_sub_bdc0 == crate::ModeLevelHudBuildPath::LocBe20DirectInitialCounterUpdates {
        // ASM Mode_Level $7D6 -> sub_E12, including sub_1596, sub_6874,
        // Make_SpriteAttr_HUD sub_BDC0 and both calc_display_number calls.
        // The initial life/diamond counters take the long path through
        // loc_BF2A..return_BF80; keeping the complete instruction sequence
        // prevents that source work from being misclassified as VDP ownership.
        const INITIAL_COUNTER_UPDATE_SUFFIX: &[u8] = &[
            18, 8, 16, 10, 16, 10, 24, 24, 16, 18, 12, 8, 16, 18, 10, 12, 16, 12, 8, 12, 12, 12,
            12, 4, 14, 12, 8, 12, 10, 8, 4, 8, 4, 4, 10, 8, 12, 8, 12, 8, 4, 8, 10, 12, 16, 12, 12,
            12, 12, 16, 12, 16, 16, 10, 12, 8, 12, 12, 8, 12, 12, 8, 12, 4, 18, 12, 8, 8, 16, 12,
            16, 12, 16, 12, 16, 12, 16, 12, 10, 12, 8, 10, 12, 8, 4, 4, 4, 20, 18, 8, 4, 4, 4, 8,
            8, 4, 4, 8, 4, 16, 12, 4, 12, 12, 12, 12, 4, 4, 4, 20, 18, 8, 4, 4, 4, 8, 8, 4, 4, 8,
            4, 16, 12, 4, 12, 16, 18,
        ];
        for &cycles in INITIAL_COUNTER_UPDATE_SUFFIX {
            emit(u32::from(cycles));
        }
        return Some(timing);
    }

    // ASM Mode_Level $7D6 and sub_1596 $1596..$168C. The custom queue pointer
    // equals its empty producer base rather than zero, while the cell queue
    // takes the direct empty branch.
    for cycles in [
        18, 8, 16, 8, 4, 12, 4, 8, 8, 12, 10, 8, 8, 4, 16, 16, 12, 8, 10, 24, 24, 16,
    ] {
        emit(cycles);
    }

    // ASM Mode_Level $7DA and unpaused sub_6874 $6874..$687A.
    for cycles in [18, 12, 8, 16] {
        emit(cycles);
    }

    // ASM Mode_Level $7DE, j_Make_SpriteAttr_HUD $7176, and sub_BDC0 through
    // $BF80, followed by the $7E2 BSR.W Execute_Objects instruction.
    // ASM loc_BE20 -> loc_BE0C -> loc_BE34 -> loc_BE4C costs the same 110
    // raw cycles that the first gameplay pass instead spends in the two
    // calc_display_number calls. Frame-902 and frame-1706 selected-PC traces
    // therefore pin both complete HUD paths to the same instruction ledger.
    const DIRECT_STABLE_COUNTERS: &[u8] = &[
        18, 10, 12, 16, 12, 8, 12, 12, 12, 12, 4, 14, 12, 8, 12, 10, 8, 4, 8, 4, 4, 10, 8, 12, 8,
        12, 8, 4, 8, 10, 12, 16, 12, 12, 12, 12, 16, 10, 12, 10, 12, 8, 10, 12, 10, 12, 12, 10, 16,
        18,
    ];
    const SECOND_HITPOINT_LOOP_STABLE_COUNTERS: &[u8] = &[
        18, 10, 12, 16, 12, 8, 12, 12, 12, 12, 4, 14, 12, 8, 12, 10, 8, 4, 8, 4, 4, 10, 8, 12, 8,
        12, 8, 4, 10, 4, 8, 4, 4, 8, 8, 10, 4, 10, 8, 12, 8, 12, 8, 10, 12, 16, 12, 12, 12, 12, 16,
        10, 12, 10, 12, 8, 10, 12, 10, 12, 12, 10, 16, 18,
    ];
    let hud_cycles = match work.hud_path_sub_bdc0 {
        crate::ModeLevelHudBuildPath::LocBe20DirectInitialCounterUpdates => {
            unreachable!("initial counter path returns above")
        }
        crate::ModeLevelHudBuildPath::LocBe20DirectStableCounters => DIRECT_STABLE_COUNTERS,
        crate::ModeLevelHudBuildPath::LocBe20ThenLocBe4cStableCounters => {
            SECOND_HITPOINT_LOOP_STABLE_COUNTERS
        }
        crate::ModeLevelHudBuildPath::Other => unreachable!("guard rejects other HUD paths"),
    };
    for &cycles in hud_cycles {
        emit(u32::from(cycles));
    }

    Some(timing)
}

/// Visit a repeating `Kid_Transform sub_8E0` suffix after the H-scroll DMA
/// through the next `sub_8E0` call.
///
/// ASM `loc_8FE -> sub_1596 -> Make_SpriteAttr_HUD -> return_912`, followed
/// by `loc_7D22`'s restore/DBF repeat and saved-register call shell. This
/// deliberately supports only the empty foreground queues and stable
/// three-hitpoint HUD path observed throughout the first Cyclone descriptor;
/// other work remains unclassified rather than borrowing `Mode_Level`'s
/// extra `sub_6874` and `sub_E12` calls.
pub fn visit_transform_sub_8e0_repeat_suffix_after_scroll_dma(
    work: crate::ModeLevelVblankWorkBeforeSubE12,
    mut visit: impl FnMut(u32),
) -> Option<IntroSourceTiming> {
    if work.foreground_custom_queue_entries_sub_1596 != 0
        || !work.foreground_cell_queue_sub_1596.is_empty()
        || work.paused_sub_6874
        || work.hud_path_sub_bdc0 != crate::ModeLevelHudBuildPath::LocBe20ThenLocBe4cStableCounters
    {
        return None;
    }

    // Oracle published frame 1059 selected-PC ledger, independently checked
    // against kid.asm loc_8FE, sub_1596, sub_BDC0, and loc_7D22.
    const CYCLES: &[u16] = &[
        18, 8, 16, 8, 4, 12, 4, 8, 8, 12, 10, 8, 8, 4, 16, 16, 12, 8, 10, 24, 24, 16, 18, 10, 12,
        16, 12, 8, 12, 12, 12, 10, 12, 16, 12, 10, 12, 10, 12, 8, 10, 12, 10, 12, 12, 10, 16, 16,
        16, 124, 10, 120, 18, 10,
    ];
    let mut raw_cycles = 0_u32;
    for &cycles in CYCLES {
        let cycles = u32::from(cycles);
        visit(cycles);
        raw_cycles = raw_cycles.checked_add(cycles)?;
    }
    Some(IntroSourceTiming {
        instruction_count: CYCLES.len() as u32,
        raw_cycles: M68kCycleCount::from_raw(raw_cycles),
    })
}

/// Visit a Cyclone descriptor transition after the H-scroll DMA through the
/// next `sub_8E0` call.
pub fn visit_transform_sub_8e0_descriptor_suffix_after_scroll_dma(
    work: crate::ModeLevelVblankWorkBeforeSubE12,
    resume: crate::TransformRasterResume,
    mut visit: impl FnMut(u32),
) -> Option<IntroSourceTiming> {
    if work.foreground_custom_queue_entries_sub_1596 != 0
        || !work.foreground_cell_queue_sub_1596.is_empty()
        || work.paused_sub_6874
        || work.hud_path_sub_bdc0 != crate::ModeLevelHudBuildPath::LocBe20ThenLocBe4cStableCounters
    {
        return None;
    }

    // Oracle published frames 1074..1114 selected-PC ledgers. After the common
    // loc_8FE/HUD tail, loc_7D22 exhausts DBF and loc_7D44 dispatches the next
    // descriptor handler before re-entering sub_8E0.
    const HELMET_ART_CYCLES: &[u8] = &[
        18, 8, 16, 8, 4, 12, 4, 8, 8, 12, 10, 8, 8, 4, 16, 16, 12, 8, 10, 24, 24, 16, 18, 10, 12,
        16, 12, 8, 12, 12, 12, 10, 12, 16, 12, 10, 12, 10, 12, 8, 10, 12, 10, 12, 12, 10, 16, 16,
        16, 124, 14, 10, 8, 12, 4, 22, 4, 8, 18, 8, 16, 16, 16, 12, 12, 18, 12, 4, 12, 18, 12, 12,
        4, 12, 10, 12, 10, 12, 10, 12, 10, 12, 10, 12, 10, 12, 10, 12, 10, 12, 10, 12, 10, 12, 10,
        12, 10, 12, 10, 12, 10, 12, 14, 12, 16, 16, 120, 18, 10,
    ];
    const KID_ART_CYCLES: &[u8] = &[
        18, 8, 16, 8, 4, 12, 4, 8, 8, 12, 10, 8, 8, 4, 16, 16, 12, 8, 10, 24, 24, 16, 18, 10, 12,
        16, 12, 8, 12, 12, 12, 10, 12, 16, 12, 10, 12, 10, 12, 8, 10, 12, 10, 12, 12, 10, 16, 16,
        16, 124, 14, 10, 8, 12, 4, 22, 4, 8, 18, 8, 16, 4, 16, 16, 12, 10, 12, 4, 18, 12, 4, 12,
        18, 12, 12, 4, 12, 10, 12, 10, 12, 10, 12, 10, 12, 10, 12, 10, 12, 10, 12, 10, 12, 10, 12,
        10, 12, 10, 12, 14, 12, 16, 18, 12, 12, 4, 14, 12, 18, 12, 12, 12, 12, 12, 16, 16, 120, 18,
        10,
    ];
    const HELMET_DEFAULT_CYCLES: &[u8] = &[
        18, 8, 16, 8, 4, 12, 4, 8, 8, 12, 10, 8, 8, 4, 16, 16, 12, 8, 10, 24, 24, 16, 18, 10, 12,
        16, 12, 8, 12, 12, 12, 10, 12, 16, 12, 10, 12, 10, 12, 8, 10, 12, 10, 12, 12, 10, 16, 16,
        16, 124, 14, 10, 8, 12, 4, 22, 4, 8, 18, 8, 16, 16, 16, 12, 4, 8, 8, 16, 12, 18, 12, 4, 12,
        18, 12, 12, 4, 12, 10, 12, 10, 12, 10, 12, 10, 12, 10, 12, 10, 12, 10, 12, 10, 12, 10, 12,
        10, 12, 10, 12, 14, 12, 16, 16, 120, 18, 10,
    ];
    let cycles = match resume {
        crate::TransformRasterResume::AdvanceDescriptor {
            handler: crate::TransformHandler::HelmetArt,
            ..
        } => HELMET_ART_CYCLES,
        crate::TransformRasterResume::AdvanceDescriptor {
            handler: crate::TransformHandler::KidWithSpark { spark: false },
            ..
        } => KID_ART_CYCLES,
        crate::TransformRasterResume::AdvanceDescriptor {
            handler: crate::TransformHandler::HelmetDefault,
            ..
        } => HELMET_DEFAULT_CYCLES,
        _ => return None,
    };
    let mut raw_cycles = 0_u32;
    for &cycles in cycles {
        let cycles = u32::from(cycles);
        visit(cycles);
        raw_cycles += cycles;
    }
    Some(IntroSourceTiming {
        instruction_count: cycles.len() as u32,
        raw_cycles: M68kCycleCount::from_raw(raw_cycles),
    })
}

/// Advance the final Cyclone transform pass from the H-scroll DMA return to
/// the natural `sub_44DC` entry in the resumed `Mode_Level` frame.
pub fn transform_sub_8e0_finish_to_sub_44dc_timing(
    work: crate::ModeLevelVblankWorkBeforeSubE12,
    resume: crate::TransformRasterResume,
    phase_before: crate::M68kRefreshPhase,
) -> Option<crate::M68kRefreshAdvance> {
    if work.foreground_custom_queue_entries_sub_1596 != 0
        || !work.foreground_cell_queue_sub_1596.is_empty()
        || work.paused_sub_6874
        || work.hud_path_sub_bdc0 != crate::ModeLevelHudBuildPath::LocBe20ThenLocBe4cStableCounters
        || resume != crate::TransformRasterResume::FinishLoc7e14
    {
        return None;
    }

    // ASM loc_8FE -> loc_7D22 -> loc_7E14 -> loc_7E90 -> sub_44DC.
    // The published-TAS frame-1130 all-PC journal contains 693 instructions
    // and 8,626 raw cycles. Its instruction boundaries schedule 73 refreshes;
    // aggregating the raw total would miss five of them. No VDP/Z80 wait
    // occurs in this interval.
    if phase_before.get() != 16 {
        return None;
    }
    Some(crate::M68kRefreshAdvance {
        elapsed: M68kCycleCount::from_raw(8_772),
        phase_after: crate::M68kRefreshPhase::new(84).expect("valid refresh phase"),
        refresh_events: 73,
    })
}

/// Visit the classified Cyclone-prize `Execute_Objects sub_E12` path through
/// the first `Kid_Transform sub_8E0` call.
///
/// ASM `sub_E12 -> loc_E1E -> loc_103B2 -> loc_A73A -> loc_7C42 ->
/// loc_7D22 -> sub_8E0`. The instruction ledger is the published-TAS
/// frame-1058 all-PC trace; it contains no VDP or Z80 external wait.
pub fn visit_cyclone_pickup_transform_entry_sub_e12(
    _work: crate::CyclonePickupTransformEntrySubE12,
    mut visit: impl FnMut(u32),
) -> IntroSourceTiming {
    const CYCLES: &[u8] = &[
        18, 10, 18, 10, 18, 10, 12, 12, 8, 12, 4, 10, 12, 12, 8, 10, 12, 4, 8, 8, 8, 12, 12, 16,
        12, 18, 10, 12, 12, 8, 12, 4, 10, 12, 12, 8, 8, 8, 12, 4, 8, 8, 8, 12, 12, 16, 12, 12, 16,
        12, 18, 10, 18, 10, 12, 12, 8, 12, 4, 10, 12, 12, 8, 8, 8, 12, 4, 8, 8, 8, 12, 12, 16, 12,
        18, 10, 12, 12, 8, 12, 4, 10, 12, 12, 8, 8, 8, 12, 4, 8, 8, 8, 12, 12, 16, 12, 12, 16, 16,
        16, 16, 8, 12, 8, 4, 12, 10, 16, 8, 16, 80, 16, 24, 16, 12, 10, 12, 8, 4, 4, 12, 4, 12, 4,
        12, 4, 12, 4, 12, 4, 12, 12, 12, 10, 12, 10, 20, 10, 18, 10, 16, 24, 16, 76, 16, 10, 12, 8,
        4, 12, 10, 16, 8, 16, 80, 16, 24, 16, 18, 10, 8, 8, 8, 10, 16, 16, 8, 16, 4, 4, 8, 4, 10,
        4, 16, 12, 12, 8, 12, 8, 10, 10, 16, 18, 32, 8, 16, 8, 6, 8, 4, 10, 16, 8, 6, 10, 28, 28,
        16, 16, 16, 8, 4, 18, 24, 10, 8, 12, 8, 6, 8, 4, 10, 12, 8, 6, 8, 4, 10, 12, 8, 6, 8, 4,
        10, 12, 8, 6, 8, 4, 10, 12, 8, 6, 10, 28, 28, 16, 16, 12, 8, 4, 10, 4, 10, 4, 10, 4, 8, 8,
        16, 8, 6, 10, 28, 28, 16, 16, 10, 16, 10, 4, 36, 16, 10, 12, 8, 4, 12, 10, 16, 8, 16, 80,
        16, 24, 16, 24, 18, 12, 12, 16, 12, 4, 12, 12, 16, 8, 10, 10, 8, 12, 16, 4, 8, 8, 22, 16,
        16, 20, 20, 18, 12, 10, 16, 12, 12, 4, 8, 8, 4, 8, 4, 12, 8, 12, 8, 12, 8, 18, 16, 16, 8,
        8, 4, 8, 12, 4, 12, 8, 8, 10, 12, 12, 4, 10, 4, 12, 18, 12, 14, 4, 12, 14, 14, 4, 8, 8, 8,
        12, 16, 10, 8, 8, 12, 16, 14, 4, 16, 12, 8, 12, 4, 12, 12, 12, 18, 12, 14, 4, 12, 14, 14,
        4, 8, 8, 8, 10, 4, 16, 10, 8, 4, 8, 12, 8, 4, 12, 18, 12, 14, 4, 12, 14, 14, 4, 8, 8, 8,
        12, 16, 10, 8, 8, 12, 16, 14, 4, 16, 10, 4, 8, 16, 12, 10, 16, 16, 16, 16, 16, 24, 12, 12,
        12, 12, 18, 24, 8, 16, 10, 28, 16, 12, 12, 12, 18, 14, 4, 12, 12, 16, 12, 10, 20, 12, 12,
        12, 10, 4, 8, 14, 8, 8, 12, 4, 22, 4, 8, 18, 8, 16, 4, 16, 16, 12, 10, 12, 4, 18, 12, 4,
        12, 18, 12, 12, 4, 12, 10, 12, 10, 12, 10, 12, 10, 12, 10, 12, 10, 12, 10, 12, 10, 12, 10,
        12, 10, 12, 10, 12, 14, 12, 16, 18, 12, 12, 4, 14, 12, 18, 12, 12, 12, 12, 12, 16, 16, 120,
        18, 10,
    ];
    let mut raw_cycles = 0_u32;
    for &cycles in CYCLES {
        let cycles = u32::from(cycles);
        visit(cycles);
        raw_cycles += cycles;
    }
    IntroSourceTiming {
        instruction_count: CYCLES.len() as u32,
        raw_cycles: M68kCycleCount::from_raw(raw_cycles),
    }
}

fn visit_title_execute_objects_setup(
    rng_offset_before: u16,
    emit: &mut dyn FnMut(u32),
) -> Option<()> {
    if !crate::RomRandomFrameState::offset_is_initialised(rng_offset_before) {
        return None;
    }
    let mut offset = rng_offset_before as i16;

    let mut visit_byte = |emit: &mut dyn FnMut(u32)| {
        // ASM Get_RandomNumber_byte sub_9FE. The two conditional paths are
        // derived from the live ring offset, not from an elapsed-cycle fit.
        for cycles in [12, 12, 8, 12, 4] {
            emit(cycles);
        }
        offset += 1;
        if offset == 0 {
            for cycles in [8, 4] {
                emit(cycles);
            }
            offset = -55;
        } else {
            emit(10);
        }
        for cycles in [12, 12, 8] {
            emit(cycles);
        }
        if offset >= -31 {
            for cycles in [8, 8] {
                emit(cycles);
            }
            offset -= 55;
        } else {
            emit(10);
        }
        for cycles in [12, 4, 8, 8, 8, 12, 12, 16] {
            emit(cycles);
        }
    };
    let mut visit_word = |emit: &mut dyn FnMut(u32)| {
        for cycles in [18, 10] {
            emit(cycles);
        }
        visit_byte(emit);
        for cycles in [12, 18, 10] {
            emit(cycles);
        }
        visit_byte(emit);
        for cycles in [12, 12, 16] {
            emit(cycles);
        }
    };

    // ASM Mode_Standard $790, Execute_Objects sub_E12, and the
    // j_Get_RandomNumber_long/Get_RandomNumber_long sub_A4A shells.
    for cycles in [18, 18, 10, 18, 10] {
        emit(cycles);
    }
    visit_word(emit);
    for cycles in [12, 18, 10] {
        emit(cycles);
    }
    visit_word(emit);
    // sub_A4A epilogue, then Execute_Objects saves its caller stack and loads
    // the object-list head address before loc_E1E.
    for cycles in [16, 16, 12, 12] {
        emit(cycles);
    }

    Some(())
}

/// Visit the first title `Execute_Objects` call up to `sub_1B8F4`.
///
/// The title RNG offset is already initialised and the sole controller object
/// is awake with Samurai haze clear. The visitor retains all four
/// `Get_RandomNumber_byte sub_9FE` branch decisions before the list dispatch.
pub fn visit_title_execute_objects_to_intro_controller_sub_1b8f4(
    rng_offset_before: u16,
    mut visit: impl FnMut(u32),
) -> Option<IntroSourceTiming> {
    let mut timing = IntroSourceTiming {
        instruction_count: 0,
        raw_cycles: M68kCycleCount::from_raw(0),
    };
    let mut emit = |cycles: u32| {
        visit(cycles);
        timing.instruction_count += 1;
        timing.raw_cycles = M68kCycleCount::from_raw(
            timing
                .raw_cycles
                .get()
                .checked_add(cycles)
                .expect("title Execute_Objects source timing fits u32"),
        );
    };

    visit_title_execute_objects_setup(rng_offset_before, &mut emit)?;

    // ASM loc_E1E -> loc_E3E: the sole controller is live and awake.
    for cycles in [12, 8, 4, 8, 10, 12, 8, 12, 44, 12, 24, 16] {
        emit(cycles);
    }

    Some(timing)
}

/// Visit the first fresh-title `Execute_Objects` pass.
///
/// ASM `Load_TitleScreen` leaves equal-priority objects ordered as
/// `Obj_TitleTextChameleon`, `Obj_TitleTextKid`, then `Obj_TitleMenu`.
/// Selected-PC oracle tracing confirms that order on the first post-loader
/// pass. Each object initialises and hibernates; the Kid allocates the first
/// GfxObject and the menu allocates two child Objects before yielding.
pub fn visit_fresh_title_first_execute_objects(
    rng_offset_before: u16,
    mut visit: impl FnMut(u32),
) -> Option<IntroSourceTiming> {
    let mut timing = IntroSourceTiming {
        instruction_count: 0,
        raw_cycles: M68kCycleCount::from_raw(0),
    };
    let mut emit = |cycles: u32| {
        visit(cycles);
        timing.instruction_count += 1;
        timing.raw_cycles = M68kCycleCount::from_raw(
            timing
                .raw_cycles
                .get()
                .checked_add(cycles)
                .expect("fresh-title Execute_Objects timing fits u32"),
        );
    };
    let dispatch = |emit: &mut dyn FnMut(u32)| {
        // ASM loc_E1E..loc_E56. Selected-PC tracing includes the absolute
        // object-field/current-object accesses and full eight-register
        // MOVEM costs; all three objects take this awake path.
        for cycles in [12, 8, 4, 12, 10, 16, 8, 16, 80, 16, 24, 16] {
            emit(cycles);
        }
    };
    let hibernate_one = |emit: &mut dyn FnMut(u32)| {
        for cycles in [18, 10, 16, 24, 16, 76, 16, 10] {
            emit(cycles);
        }
    };
    let allocate_object_before_equal_head = |emit: &mut dyn FnMut(u32)| {
        for cycles in [8, 18, 10, 40, 4, 16, 28, 4, 4, 8] {
            emit(cycles);
        }
        for index in 0..29 {
            emit(12);
            emit(if index == 28 { 14 } else { 10 });
        }
        for cycles in [16, 16, 16, 8, 12, 16, 8, 4, 4, 12, 8, 4, 28, 16, 44, 16] {
            emit(cycles);
        }
    };

    visit_title_execute_objects_setup(rng_offset_before, &mut emit)?;

    // ASM loc_E1E -> Obj_TitleTextChameleon sub_1B6A0.
    dispatch(&mut emit);
    emit(4);
    hibernate_one(&mut emit);

    // ASM loc_E1E -> Obj_TitleTextKid sub_1B5BC. Its category-zero,
    // priority-$01FF GfxObject is the first entry in the GfxObject list.
    dispatch(&mut emit);
    for cycles in [12, 18, 10, 18, 40, 4, 16, 28, 4, 4, 8] {
        emit(cycles);
    }
    for index in 0..19 {
        emit(12);
        emit(if index == 18 { 14 } else { 10 });
    }
    for cycles in [12, 10, 4, 12, 16, 16, 8, 12, 16, 10, 28, 16, 44, 16] {
        emit(cycles);
    }
    // Return through Load_GfxObjectSlot, initialise the title KID sprite,
    // and enter the 30-frame Hibernate_Object call.
    for cycles in [16, 16, 16, 10, 16, 16, 16, 16, 8, 16, 12, 18, 10] {
        emit(cycles);
    }
    for cycles in [16, 24, 16, 76, 16, 10] {
        emit(cycles);
    }

    // ASM loc_E1E -> Obj_TitleMenu loc_1BAC2. Both equal-priority children
    // insert before the current list head and are not visited this pass.
    dispatch(&mut emit);
    allocate_object_before_equal_head(&mut emit);
    emit(24);
    allocate_object_before_equal_head(&mut emit);
    for cycles in [24, 24, 8, 12] {
        emit(cycles);
    }
    hibernate_one(&mut emit);

    // Obj_TitleMenu is the final original-list object; loc_E58 returns to
    // Mode_Standard after its hibernation.
    for cycles in [12, 10, 16, 16] {
        emit(cycles);
    }

    Some(timing)
}

/// Visit frame 197's fresh-title `Execute_Objects` prefix through `sub_1B71A`.
///
/// ASM `loc_E1E` first resumes the Sega copyright worker, then initialises the
/// menu prompt's category-zero GfxObject before resuming Chameleon. The visit
/// includes Chameleon's `BSR.W sub_1B71A` and stops at that routine's entry.
pub fn visit_fresh_title_second_execute_objects_to_chameleon_enigma(
    rng_offset_before: u16,
    mut visit: impl FnMut(u32),
) -> Option<IntroSourceTiming> {
    let mut timing = IntroSourceTiming {
        instruction_count: 0,
        raw_cycles: M68kCycleCount::from_raw(0),
    };
    let mut emit = |cycles: u32| {
        visit(cycles);
        timing.instruction_count += 1;
        timing.raw_cycles = M68kCycleCount::from_raw(
            timing
                .raw_cycles
                .get()
                .checked_add(cycles)
                .expect("fresh-title second Execute_Objects prefix fits u32"),
        );
    };
    let dispatch = |emit: &mut dyn FnMut(u32)| {
        // ASM loc_E1E..loc_E56: awake object dispatch.
        for cycles in [12, 8, 4, 12, 10, 16, 8, 16, 80, 16, 24, 16] {
            emit(cycles);
        }
    };
    let hibernate_one = |emit: &mut dyn FnMut(u32)| {
        // Jump-table call plus Hibernate_Object_1Frame sub_E76.
        for cycles in [18, 10, 16, 24, 16, 76, 16, 10] {
            emit(cycles);
        }
    };

    visit_title_execute_objects_setup(rng_offset_before, &mut emit)?;

    // ASM loc_1C7A0 initialises the Sega copyright worker's text pointers,
    // then loc_1C7AE yields for one frame before drawing its first glyph.
    dispatch(&mut emit);
    for cycles in [16, 8, 8, 8, 4] {
        emit(cycles);
    }
    hibernate_one(&mut emit);

    // ASM loc_1BB10 enters Load_GfxObjectSlot sub_F5E. The new priority-$0200
    // entry sorts before Kid's existing priority-$01FF entry.
    dispatch(&mut emit);
    for cycles in [12, 18, 10, 18, 40, 4, 16, 28, 4, 4, 8] {
        emit(cycles);
    }
    for index in 0..19 {
        emit(12);
        emit(if index == 18 { 14 } else { 10 });
    }
    for cycles in [
        12, 10, 4, 12, 16, 16, 8, 12, 16, 8, 4, 4, 12, 8, 4, 28, 16, 44, 16, 16, 16,
    ] {
        emit(cycles);
    }
    // ASM loc_1BB1A..loc_1BB4C initialises the prompt sprite and yields.
    for cycles in [16, 16, 16, 16, 16, 8, 12, 10, 12, 4, 12] {
        emit(cycles);
    }
    hibernate_one(&mut emit);

    // Chameleon resumes after its frame-196 hibernation at loc_1B6A6.
    dispatch(&mut emit);
    emit(18);

    Some(timing)
}

/// Visit frame 198's fresh-title object pass through `sub_1B71A`.
///
/// ASM `loc_1C7B2` drains SegaText's zero-delay copyright string in one pass,
/// writing eleven glyphs before `Delete_CurrentObject sub_F1A`. The prompt
/// and Chameleon then resume from their saved coroutine positions. The visit
/// includes Chameleon's `BSR.W sub_1B71A` and stops at that routine's entry.
pub fn visit_fresh_title_third_execute_objects_to_chameleon_enigma(
    rng_offset_before: u16,
    mut visit: impl FnMut(MemoryDmaTimingEvent),
) -> Option<IntroSourceTiming> {
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
                .expect("fresh-title third Execute_Objects prefix fits u32"),
        );
    };
    let dispatch = |visit: &mut dyn FnMut(MemoryDmaTimingEvent), timing: &mut IntroSourceTiming| {
        // ASM loc_E1E..loc_E56: awake object dispatch.
        for cycles in [12, 8, 4, 12, 10, 16, 8, 16, 80, 16, 24, 16] {
            emit(visit, timing, cycles);
        }
    };
    let hibernate_one = |visit: &mut dyn FnMut(MemoryDmaTimingEvent),
                         timing: &mut IntroSourceTiming| {
        for cycles in [18, 10, 16, 24, 16, 76, 16, 10] {
            emit(visit, timing, cycles);
        }
    };

    visit_title_execute_objects_setup(rng_offset_before, &mut |cycles| {
        emit(&mut visit, &mut timing, cycles);
    })?;

    // SegaText resumes after loc_1C7AE's one-frame hibernation. Its delay is
    // zero, so all eleven copyright glyphs reach sub_1C7F6 this pass.
    dispatch(&mut visit, &mut timing);
    for cycles in [4, 8] {
        emit(&mut visit, &mut timing, cycles);
    }
    for _ in 0..11 {
        for cycles in [4, 8, 8, 10, 8, 10, 8, 10, 12, 10, 8, 18] {
            emit(&mut visit, &mut timing, cycles);
        }
        // ASM sub_1C7F6 computes one plane-A command and tile word.
        for cycles in [4, 20, 4, 4, 8, 4, 8, 8, 4, 10, 8, 4, 16, 12, 8] {
            emit(&mut visit, &mut timing, cycles);
        }
        visit(MemoryDmaTimingEvent::WriteVdpWord {
            target: MemoryToVdpDmaTarget::Vram,
        });
        emit(&mut visit, &mut timing, 16);
        for cycles in [4, 12, 10] {
            emit(&mut visit, &mut timing, cycles);
        }
    }
    // ASM loc_1C7B6 sees `$FF`, then Delete_CurrentObject removes SegaText
    // from the head of the Object list and branches back to loc_E1E.
    for cycles in [4, 8, 8, 8, 10, 10] {
        emit(&mut visit, &mut timing, cycles);
    }
    for cycles in [
        16, 18, 32, 8, 16, 8, 6, 10, 28, 28, 16, 16, 16, 10, 16, 10, 16, 10, 4, 36, 16, 10,
    ] {
        emit(&mut visit, &mut timing, cycles);
    }

    // The prompt resumes at loc_1BB50, recomputes x=268, and hibernates.
    dispatch(&mut visit, &mut timing);
    for cycles in [10, 8, 12, 10, 12, 4, 12] {
        emit(&mut visit, &mut timing, cycles);
    }
    hibernate_one(&mut visit, &mut timing);

    // Chameleon resumes at loc_1B6A6 with d3=1 and calls sub_1B71A again.
    dispatch(&mut visit, &mut timing);
    emit(&mut visit, &mut timing, 18);

    Some(timing)
}

/// Visit a post-SegaText fresh-title object pass through `sub_1B71A`.
///
/// ASM `loc_E1E` starts at the prompt after SegaText deleted itself on the
/// preceding pass. The prompt resumes at `loc_1BB50` and hibernates, then
/// Chameleon resumes at `loc_1B6A6`, advances its retained `d3`, and calls
/// `sub_1B71A`.
pub fn visit_fresh_title_prompt_chameleon_execute_objects_to_enigma(
    rng_offset_before: u16,
    mut visit: impl FnMut(MemoryDmaTimingEvent),
) -> Option<IntroSourceTiming> {
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
                .expect("fresh-title fourth Execute_Objects prefix fits u32"),
        );
    };
    let dispatch = |visit: &mut dyn FnMut(MemoryDmaTimingEvent), timing: &mut IntroSourceTiming| {
        // ASM loc_E1E..loc_E56: awake object dispatch.
        for cycles in [12, 8, 4, 12, 10, 16, 8, 16, 80, 16, 24, 16] {
            emit(visit, timing, cycles);
        }
    };

    visit_title_execute_objects_setup(rng_offset_before, &mut |cycles| {
        emit(&mut visit, &mut timing, cycles);
    })?;

    // ASM loc_1BB50 completes the prompt coroutine's x-position pass and
    // hibernates through sub_E76.
    dispatch(&mut visit, &mut timing);
    for cycles in [10, 8, 12, 10, 12, 4, 12] {
        emit(&mut visit, &mut timing, cycles);
    }
    for cycles in [18, 10, 16, 24, 16, 76, 16, 10] {
        emit(&mut visit, &mut timing, cycles);
    }

    // ASM loc_1B6A6 increments the retained animation cursor and calls
    // sub_1B71A; its caller selects the matching off_1B70A stream.
    dispatch(&mut visit, &mut timing);
    emit(&mut visit, &mut timing, 18);

    Some(timing)
}

/// Visit `sub_1B71A` after `EniDec` through a fresh-title object-pass return.
///
/// The first CHAMELEON row emits 21 decoded words; the remaining six emit 29
/// words. Since `d2=0`, every row also writes the `$00BC` leading filler.
pub fn visit_fresh_title_chameleon_upload_to_execute_return(
    visit: impl FnMut(MemoryDmaTimingEvent),
) -> IntroSourceTiming {
    visit_fresh_title_chameleon_upload_to_menu_outcome(false, visit)
}

/// Visit `sub_1B71A` after `EniDec` through the title-menu timeout at
/// `loc_1BB02`.
///
/// ASM `Obj_TitleMenu loc_1BAEE -> loc_1BB02`: the final DBF falls through
/// instead of hibernating the menu again.
pub fn visit_fresh_title_chameleon_upload_to_demo_mode(
    visit: impl FnMut(MemoryDmaTimingEvent),
) -> IntroSourceTiming {
    visit_fresh_title_chameleon_upload_to_menu_outcome(true, visit)
}

/// Visit `loc_1BB02 -> loc_6E2 -> Pal_FadeOut` through its first VINT poll.
pub fn visit_title_demo_mode_to_first_fade_wait(mut visit: impl FnMut(u32)) -> IntroSourceTiming {
    let mut timing = IntroSourceTiming {
        instruction_count: 0,
        raw_cycles: M68kCycleCount::from_raw(0),
    };
    let mut emit = |cycles: u32| {
        visit(cycles);
        timing.instruction_count += 1;
        timing.raw_cycles = M68kCycleCount::from_raw(
            timing
                .raw_cycles
                .get()
                .checked_add(cycles)
                .expect("title demo-transition fade setup timing fits u32"),
        );
    };

    for cycles in [16, 12, 16, 8, 16, 16, 12, 8, 18] {
        emit(cycles);
    }
    for cycles in [4, 12, 12] {
        emit(cycles);
    }
    for index in 0..32 {
        emit(20);
        emit(if index == 31 { 14 } else { 10 });
    }
    for cycles in [4, 12, 12] {
        emit(cycles);
    }
    for index in 0..64 {
        emit(8);
        emit(if index == 63 { 14 } else { 10 });
    }
    for cycles in [16, 10, 4, 16, 16, 10, 18, 16] {
        emit(cycles);
    }

    timing
}

fn visit_fresh_title_chameleon_upload_to_menu_outcome(
    menu_expires: bool,
    mut visit: impl FnMut(MemoryDmaTimingEvent),
) -> IntroSourceTiming {
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
                .expect("fresh-title Chameleon upload timing fits u32"),
        );
    };
    let emit_hibernate_one = |visit: &mut dyn FnMut(MemoryDmaTimingEvent),
                              timing: &mut IntroSourceTiming| {
        for cycles in [18, 10, 16, 24, 16, 76, 16, 10] {
            emit(visit, timing, cycles);
        }
    };

    // ASM sub_1B71A loc_1B746..loc_1B76A constructs the plane-A VDP command.
    for cycles in [4, 4, 4, 4, 10, 4, 20, 4, 4, 8, 4, 4, 4, 4] {
        emit(&mut visit, &mut timing, cycles);
    }
    for row in 0..7 {
        // ASM loc_1B76C's first-row width cap; later rows branch at $1B772.
        if row == 0 {
            for cycles in [4, 8, 8, 4, 8, 8, 8, 4] {
                emit(&mut visit, &mut timing, cycles);
            }
        } else {
            for cycles in [4, 8, 10] {
                emit(&mut visit, &mut timing, cycles);
            }
        }
        // ASM loc_1B780 writes one filler tile before each decoded row.
        for cycles in [4, 8, 16, 16, 12] {
            emit(&mut visit, &mut timing, cycles);
        }
        visit(MemoryDmaTimingEvent::WriteVdpWord {
            target: MemoryToVdpDmaTarget::Vram,
        });
        for cycles in [16, 10, 16, 4] {
            emit(&mut visit, &mut timing, cycles);
        }
        let words = if row == 0 { 21 } else { 29 };
        for word in 0..words {
            emit(&mut visit, &mut timing, 12);
            visit(MemoryDmaTimingEvent::WriteVdpWord {
                target: MemoryToVdpDmaTarget::Vram,
            });
            emit(
                &mut visit,
                &mut timing,
                if word + 1 == words { 14 } else { 10 },
            );
        }
        emit(&mut visit, &mut timing, 8);
        emit(&mut visit, &mut timing, if row == 6 { 14 } else { 10 });
    }
    // Return from sub_1B71A, loop to loc_1B6A2, and yield Chameleon.
    emit(&mut visit, &mut timing, 16);
    emit(&mut visit, &mut timing, 10);
    emit_hibernate_one(&mut visit, &mut timing);

    if menu_expires {
        // ASM loc_E1E dispatches the mature Kid coroutine. Its saved PC is
        // the `bra.s loc_1B64C` after the preceding one-frame hibernate, so
        // it loops back to the JSR and yields once more before the menu runs.
        // The frame-837 oracle trace directly confirms this mature path.
        for cycles in [12, 8, 4, 12, 10, 16, 8, 16, 80, 16, 24, 16] {
            emit(&mut visit, &mut timing, cycles);
        }
        emit(&mut visit, &mut timing, 10);
        emit_hibernate_one(&mut visit, &mut timing);
    } else {
        // TODO(fidelity): model Kid's timed title-animation coroutine from
        // Obj_TitleTextKid sub_1B5BC instead of retaining the established
        // ordinary-pass skip approximation.
        for cycles in [12, 8, 4, 12, 10, 16, 10] {
            emit(&mut visit, &mut timing, cycles);
        }
    }

    // ASM loc_E1E dispatches the awake menu coroutine.
    for cycles in [12, 8, 4, 12, 10, 16, 8, 16, 80, 16, 24, 16] {
        emit(&mut visit, &mut timing, cycles);
    }
    // ASM loc_1BAF6 consumes no Start press. Ordinary passes take DBF back
    // to loc_1BAEE and yield; the last pass falls through loc_1BB02.
    for cycles in [20, 8] {
        emit(&mut visit, &mut timing, cycles);
    }
    if menu_expires {
        // ASM loc_1BAFA DBF falls through to loc_1BB02.
        emit(&mut visit, &mut timing, 14);
    } else {
        for cycles in [10, 16] {
            emit(&mut visit, &mut timing, cycles);
        }
        emit_hibernate_one(&mut visit, &mut timing);

        // The menu is the final list entry; loc_E58 returns to Mode_Standard.
        for cycles in [12, 10, 16, 16] {
            emit(&mut visit, &mut timing, cycles);
        }
    }

    timing
}

/// Visit the second title `Execute_Objects` call through both hibernations.
///
/// The equal-priority text worker is first in the execution list. It enters
/// `loc_1C7A0` and hibernates before drawing; the controller then advances
/// `sub_1C204` by one fade step and hibernates at `loc_1C21E`.
pub fn visit_title_second_execute_objects(
    rng_offset_before: u16,
    mut visit: impl FnMut(u32),
) -> Option<IntroSourceTiming> {
    let mut timing = IntroSourceTiming {
        instruction_count: 0,
        raw_cycles: M68kCycleCount::from_raw(0),
    };
    let mut emit = |cycles: u32| {
        visit(cycles);
        timing.instruction_count += 1;
        timing.raw_cycles = M68kCycleCount::from_raw(
            timing
                .raw_cycles
                .get()
                .checked_add(cycles)
                .expect("second title Execute_Objects timing fits u32"),
        );
    };

    visit_title_execute_objects_setup(rng_offset_before, &mut emit)?;
    // ASM loc_E1E dispatches the live text worker into loc_1C7A0.
    for cycles in [12, 8, 4, 8, 10, 12, 8, 12, 44, 12, 24, 16] {
        emit(cycles);
    }
    // ASM loc_1C7A0 loads IntroText1's coordinates and hibernates once
    // through the j_Hibernate_Object_1Frame thunk and sub_E76.
    for cycles in [12, 8, 8, 8, 4, 18, 10, 16, 24, 16, 76, 16, 10] {
        emit(cycles);
    }
    // ASM loc_E1E dispatches the controller from its one-frame hibernation.
    for cycles in [12, 8, 4, 8, 10, 12, 8, 12, 44, 12, 24, 16] {
        emit(cycles);
    }
    // ASM loc_1C222 calls sub_1C246 with no Start edge, advances fade state
    // to $0010, marks both dirty masks, and hibernates at loc_1C21E.
    for cycles in [
        18,
        TITLE_CONTROLLER_START_POLL_CYCLES,
        8,
        16,
        16,
        8,
        4,
        16,
        16,
        10,
        18,
        10,
        16,
        24,
        16,
        76,
        16,
        10,
    ] {
        emit(cycles);
    }
    // The controller has no successor; loc_E58 restores the caller stack.
    for cycles in [12, 10, 16, 16] {
        emit(cycles);
    }

    Some(timing)
}

/// Visit the third title `Execute_Objects` call through both hibernations.
///
/// The text worker resumes at `loc_1C7B2`, consumes IntroText1's initial
/// `$FE,$20` delay command, and hibernates without emitting a glyph. The
/// controller then advances `sub_1C204` to fade step `$0020` and hibernates.
pub fn visit_title_third_execute_objects(
    rng_offset_before: u16,
    mut visit: impl FnMut(u32),
) -> Option<IntroSourceTiming> {
    let mut timing = IntroSourceTiming {
        instruction_count: 0,
        raw_cycles: M68kCycleCount::from_raw(0),
    };
    let mut emit = |cycles: u32| {
        visit(cycles);
        timing.instruction_count += 1;
        timing.raw_cycles = M68kCycleCount::from_raw(
            timing
                .raw_cycles
                .get()
                .checked_add(cycles)
                .expect("third title Execute_Objects timing fits u32"),
        );
    };

    visit_title_execute_objects_setup(rng_offset_before, &mut emit)?;
    // ASM loc_E1E dispatches the live text worker at saved PC loc_1C7B2.
    for cycles in [12, 8, 4, 8, 10, 12, 8, 12, 44, 12, 24, 16] {
        emit(cycles);
    }
    // ASM loc_1C7B2 completes the one-frame countdown, recognises `$FE`,
    // loads delay `$20`, and hibernates again through sub_E76.
    for cycles in [
        4, 8, 4, 8, 8, 10, 8, 10, 8, 8, 8, 10, 18, 10, 16, 24, 16, 76, 16, 10,
    ] {
        emit(cycles);
    }
    // ASM loc_E1E dispatches the controller from its one-frame hibernation.
    for cycles in [12, 8, 4, 8, 10, 12, 8, 12, 44, 12, 24, 16] {
        emit(cycles);
    }
    // ASM loc_1C222 calls sub_1C246 with no Start edge, advances fade state
    // to $0020, marks both dirty masks, and hibernates at loc_1C21E.
    for cycles in [
        18,
        TITLE_CONTROLLER_START_POLL_CYCLES,
        8,
        16,
        16,
        8,
        4,
        16,
        16,
        10,
        18,
        10,
        16,
        24,
        16,
        76,
        16,
        10,
    ] {
        emit(cycles);
    }
    // The controller has no successor; loc_E58 restores the caller stack.
    for cycles in [12, 10, 16, 16] {
        emit(cycles);
    }

    Some(timing)
}

/// Visit an ordinary title text-delay frame while the controller still fades.
///
/// The worker resumes with a delay greater than one, decrements it, and takes
/// `loc_1C7B2 -> loc_1C7AE` directly back to hibernation. The controller's
/// `sub_1C204` fade value must still be at most `$00F0`, so adding `$10`
/// retains the dirty-mask-and-hibernate path rather than exiting at `$0110`.
pub fn visit_title_text_delay_fade_execute_objects(
    rng_offset_before: u16,
    text_delay_before: u8,
    controller_fade_before: u16,
    mut visit: impl FnMut(u32),
) -> Option<IntroSourceTiming> {
    if text_delay_before <= 1
        || !(0x20..=0xF0).contains(&controller_fade_before)
        || !controller_fade_before.is_multiple_of(0x10)
    {
        return None;
    }

    let mut timing = IntroSourceTiming {
        instruction_count: 0,
        raw_cycles: M68kCycleCount::from_raw(0),
    };
    let mut emit = |cycles: u32| {
        visit(cycles);
        timing.instruction_count += 1;
        timing.raw_cycles = M68kCycleCount::from_raw(
            timing
                .raw_cycles
                .get()
                .checked_add(cycles)
                .expect("title text-delay fade timing fits u32"),
        );
    };

    visit_title_text_delay_execute_objects_prefix(rng_offset_before, text_delay_before, &mut emit)?;
    // ASM loc_1C222 calls sub_1C246 with no Start edge, advances the fade by
    // `$10`, marks both dirty masks, and hibernates at loc_1C21E.
    for cycles in [
        18,
        TITLE_CONTROLLER_START_POLL_CYCLES,
        8,
        16,
        16,
        8,
        4,
        16,
        16,
        10,
        18,
        10,
        16,
        24,
        16,
        76,
        16,
        10,
    ] {
        emit(cycles);
    }
    for cycles in [12, 10, 16, 16] {
        emit(cycles);
    }

    Some(timing)
}

fn visit_title_text_delay_execute_objects_prefix(
    rng_offset_before: u16,
    text_delay_before: u8,
    emit: &mut impl FnMut(u32),
) -> Option<()> {
    if text_delay_before <= 1 {
        return None;
    }

    visit_title_execute_objects_setup(rng_offset_before, emit)?;
    // ASM loc_E1E dispatches the live text worker at saved PC loc_1C7B2.
    for cycles in [12, 8, 4, 8, 10, 12, 8, 12, 44, 12, 24, 16] {
        emit(cycles);
    }
    // ASM loc_1C7B2 decrements the non-terminal delay and takes BNE directly
    // to loc_1C7AE, then hibernates through the thunk and sub_E76.
    for cycles in [4, 10, 18, 10, 16, 24, 16, 76, 16, 10] {
        emit(cycles);
    }
    // ASM loc_E1E dispatches the controller from its one-frame hibernation.
    for cycles in [12, 8, 4, 8, 10, 12, 8, 12, 44, 12, 24, 16] {
        emit(cycles);
    }

    Some(())
}

/// Visit frame 98's title text delay and controller fade-exit dispatches.
///
/// The text worker still takes `loc_1C7B2 -> loc_1C7AE`. The controller
/// advances `$0100->$0110`, takes `loc_1C240`, jumps through its `$44(a5)`
/// continuation to `loc_1B920`, loads the post-fade delay, and hibernates.
pub fn visit_title_controller_fade_exit_execute_objects(
    rng_offset_before: u16,
    text_delay_before: u8,
    mut visit: impl FnMut(u32),
) -> Option<IntroSourceTiming> {
    let mut timing = IntroSourceTiming {
        instruction_count: 0,
        raw_cycles: M68kCycleCount::from_raw(0),
    };
    let mut emit = |cycles: u32| {
        visit(cycles);
        timing.instruction_count += 1;
        timing.raw_cycles = M68kCycleCount::from_raw(
            timing
                .raw_cycles
                .get()
                .checked_add(cycles)
                .expect("title controller fade-exit timing fits u32"),
        );
    };

    visit_title_text_delay_execute_objects_prefix(rng_offset_before, text_delay_before, &mut emit)?;
    // ASM loc_1C222 -> loc_1C240 advances `$0100->$0110`, then the stored
    // loc_1B920 continuation loads `$012B` and hibernates at loc_1B924.
    for cycles in [
        18,
        TITLE_CONTROLLER_START_POLL_CYCLES,
        8,
        16,
        16,
        16,
        10,
        16,
        8,
        8,
        18,
        10,
        16,
        24,
        16,
        76,
        16,
        10,
    ] {
        emit(cycles);
    }
    for cycles in [12, 10, 16, 16] {
        emit(cycles);
    }

    Some(timing)
}

/// Visit an ordinary post-fade title text-delay/controller-wait frame.
///
/// The text worker takes its non-terminal `loc_1C7B2 -> loc_1C7AE` path. The
/// controller resumes at `loc_1B928`, polls Start through `sub_1C246`, takes
/// the non-terminal `dbf` back to `loc_1B924`, and hibernates for one frame.
pub fn visit_title_post_fade_wait_execute_objects(
    rng_offset_before: u16,
    text_delay_before: u8,
    controller_wait_before: u16,
    mut visit: impl FnMut(u32),
) -> Option<IntroSourceTiming> {
    if controller_wait_before == 0 || controller_wait_before > 0x12B {
        return None;
    }

    let mut timing = IntroSourceTiming {
        instruction_count: 0,
        raw_cycles: M68kCycleCount::from_raw(0),
    };
    let mut emit = |cycles: u32| {
        visit(cycles);
        timing.instruction_count += 1;
        timing.raw_cycles = M68kCycleCount::from_raw(
            timing
                .raw_cycles
                .get()
                .checked_add(cycles)
                .expect("title post-fade wait timing fits u32"),
        );
    };

    visit_title_text_delay_execute_objects_prefix(rng_offset_before, text_delay_before, &mut emit)?;
    // ASM loc_1B928 -> sub_1C246 sees no Start edge; the non-terminal DBF
    // returns to loc_1B924 and hibernates through sub_E76.
    for cycles in [
        18,
        TITLE_CONTROLLER_START_POLL_CYCLES,
        8,
        16,
        10,
        18,
        10,
        16,
        24,
        16,
        76,
        16,
        10,
    ] {
        emit(cycles);
    }
    for cycles in [12, 10, 16, 16] {
        emit(cycles);
    }

    Some(timing)
}

fn visit_title_ordinary_glyph_text_dispatch(
    rng_offset_before: u16,
    glyph: u8,
    emit: &mut impl FnMut(u32),
) -> Option<()> {
    if glyph != b' ' && !(b'A'..=b'l').contains(&glyph) {
        return None;
    }

    visit_title_execute_objects_setup(rng_offset_before, &mut *emit)?;
    for cycles in [12, 8, 4, 8, 10, 12, 8, 12, 44, 12, 24, 16] {
        emit(cycles);
    }
    // ASM loc_1C7B2 -> loc_1C7E6 consumes an ordinary glyph.
    // ORIGINAL_QUIRK: loc_1C7DC compares against byte `(off_20).w`, not
    // immediate `#$20`; title glyph bytes take the ordinary branch here.
    for cycles in [4, 8, 4, 8, 8, 10, 8, 10, 8, 10, 12, 10, 8, 18] {
        emit(cycles);
    }
    // ASM sub_1C7F6 derives the plane command/tile, then loc_1C7E6 loads
    // IntroText1's two-frame glyph delay.
    for cycles in [
        4, 20, 4, 4, 8, 4, 8, 8, 4, 10, 8, 4, 12, 8, 8, 16, 4, 8, 8, 10, 18, 10, 16, 24, 16, 76,
        16, 10,
    ] {
        emit(cycles);
    }

    Some(())
}

/// Visit an ordinary IntroText1 glyph and controller-wait dispatches.
///
/// The text delay reaches zero, the parser consumes the glyph, and `sub_1C7F6`
/// writes its plane command/tile before loading the two-frame glyph delay.
pub fn visit_title_ordinary_glyph_execute_objects(
    rng_offset_before: u16,
    controller_wait_before: u16,
    glyph: u8,
    mut visit: impl FnMut(u32),
) -> Option<IntroSourceTiming> {
    if controller_wait_before == 0 || controller_wait_before > 0x12B {
        return None;
    }

    let mut timing = IntroSourceTiming {
        instruction_count: 0,
        raw_cycles: M68kCycleCount::from_raw(0),
    };
    let mut emit = |cycles: u32| {
        visit(cycles);
        timing.instruction_count += 1;
        timing.raw_cycles = M68kCycleCount::from_raw(
            timing
                .raw_cycles
                .get()
                .checked_add(cycles)
                .expect("title ordinary-glyph timing fits u32"),
        );
    };

    visit_title_ordinary_glyph_text_dispatch(rng_offset_before, glyph, &mut emit)?;
    for cycles in [12, 8, 4, 8, 10, 12, 8, 12, 44, 12, 24, 16] {
        emit(cycles);
    }
    for cycles in [
        18,
        TITLE_CONTROLLER_START_POLL_CYCLES,
        8,
        16,
        10,
        18,
        10,
        16,
        24,
        16,
        76,
        16,
        10,
    ] {
        emit(cycles);
    }
    for cycles in [12, 10, 16, 16] {
        emit(cycles);
    }

    Some(timing)
}

/// Visit IntroSequence1's glyph frame through the Start-driven `StopMusic` entry.
///
/// ASM `loc_1C7B2 -> sub_1C7F6` emits the glyph, then `loc_1B928 ->
/// sub_1C246 -> sub_1C278 -> loc_1C264` consumes Start, wraps the cold intro
/// sequence counter from 2 to 0, selects `Game_Mode=$2C`, and calls
/// `j_StopMusic`. The final 10-cycle thunk jump ends at `StopMusic $E14C8`.
pub fn visit_title_intro_sequence1_glyph_start_to_stop_music(
    rng_offset_before: u16,
    controller_wait_before: u16,
    glyph: u8,
    intro_sequence_counter_before: u8,
    mut visit: impl FnMut(u32),
) -> Option<IntroSourceTiming> {
    if controller_wait_before == 0
        || controller_wait_before > 0x12B
        || intro_sequence_counter_before != 2
    {
        return None;
    }

    let mut timing = IntroSourceTiming {
        instruction_count: 0,
        raw_cycles: M68kCycleCount::from_raw(0),
    };
    let mut emit = |cycles: u32| {
        visit(cycles);
        timing.instruction_count += 1;
        timing.raw_cycles = M68kCycleCount::from_raw(
            timing
                .raw_cycles
                .get()
                .checked_add(cycles)
                .expect("title Start transition timing fits u32"),
        );
    };

    visit_title_ordinary_glyph_text_dispatch(rng_offset_before, glyph, &mut emit)?;
    // ASM loc_E1E dispatches the waiting controller into loc_1B928.
    for cycles in [12, 8, 4, 8, 10, 12, 8, 12, 44, 12, 24, 16] {
        emit(cycles);
    }
    // ASM loc_1B928 -> sub_1C246 -> sub_1C278 -> loc_1C264. Frame-139
    // selected-PC tracing pins every instruction through StopMusic $E14C8.
    for cycles in [
        18,
        TITLE_CONTROLLER_START_POLL_CYCLES,
        10,
        16,
        8,
        16,
        8,
        18,
        16,
        16,
        8,
        16,
        16,
        16,
        16,
        20,
        10,
    ] {
        emit(cycles);
    }

    Some(timing)
}

/// Visit the frame-139 `StopMusic` return through the first fade wait poll.
///
/// ASM `loc_1C274 -> j_loc_6E2 -> loc_6E2 -> Pal_FadeOut` resets the stack,
/// disables the two plane registers, copies both resident palette halves,
/// seeds the 64-word fade target, calls `loc_28C6 -> WaitForVint`, and clears
/// `V_Int_Done` immediately before the first `$93E` poll.
pub fn visit_title_stop_music_return_to_first_fade_wait(
    mut visit: impl FnMut(u32),
) -> IntroSourceTiming {
    let mut timing = IntroSourceTiming {
        instruction_count: 0,
        raw_cycles: M68kCycleCount::from_raw(0),
    };
    let mut emit = |cycles: u32| {
        visit(cycles);
        timing.instruction_count += 1;
        timing.raw_cycles = M68kCycleCount::from_raw(
            timing
                .raw_cycles
                .get()
                .checked_add(cycles)
                .expect("title-return fade setup timing fits u32"),
        );
    };

    // ASM loc_1C274 -> j_loc_6E2 -> loc_6E2 -> Pal_FadeOut.
    for cycles in [10, 10, 8, 16, 16, 12, 8, 18] {
        emit(cycles);
    }
    // ASM Pal_FadeOut -> loc_28A6 copies 32 longwords.
    for cycles in [4, 12, 12] {
        emit(cycles);
    }
    for index in 0..32 {
        emit(20);
        emit(if index == 31 { 14 } else { 10 });
    }
    // ASM loc_28B8 fills 64 fade-target words from `$FFFFFBCC`.
    for cycles in [4, 12, 12] {
        emit(cycles);
    }
    for index in 0..64 {
        emit(8);
        emit(if index == 63 { 14 } else { 10 });
    }
    // ASM loc_28BE -> loc_28DE seeds the fade counter and both dirty masks,
    // then loc_28C6 calls WaitForVint and clears its completion flag.
    for cycles in [16, 10, 4, 16, 16, 10, 18, 16] {
        emit(cycles);
    }

    timing
}

/// Visit frame 115's first `T` glyph and controller-wait dispatches.
pub fn visit_title_first_glyph_execute_objects(
    rng_offset_before: u16,
    controller_wait_before: u16,
    visit: impl FnMut(u32),
) -> Option<IntroSourceTiming> {
    visit_title_ordinary_glyph_execute_objects(
        rng_offset_before,
        controller_wait_before,
        b'T',
        visit,
    )
}

/// Visit initial `sub_1B8F4` work through entry to `sub_1C4EA`.
///
/// The first title controller allocates the fixed Wild Side prompt GfxObject
/// into an empty execution list before requesting the intro palette copy.
pub fn visit_title_intro_controller_prompt_gfx_to_sub_1c4ea(
    mut visit: impl FnMut(u32),
) -> IntroSourceTiming {
    let mut timing = IntroSourceTiming {
        instruction_count: 0,
        raw_cycles: M68kCycleCount::from_raw(0),
    };
    let mut emit = |cycles: u32| {
        visit(cycles);
        timing.instruction_count += 1;
        timing.raw_cycles = M68kCycleCount::from_raw(
            timing
                .raw_cycles
                .get()
                .checked_add(cycles)
                .expect("title prompt GfxObject timing fits u32"),
        );
    };

    // ASM sub_1B8F4 -> sub_1C54A -> j_Allocate_GfxObjectSlot_a1.
    for cycles in [18, 12, 18, 10, 6, 18] {
        emit(cycles);
    }
    // ASM Allocate_GfxObjectSlot sub_F6E setup and 19-longword clear.
    for cycles in [40, 4, 16, 28, 4, 4, 8] {
        emit(cycles);
    }
    for index in 0..19 {
        emit(12);
        emit(if index == 18 { 14 } else { 10 });
    }
    // Category zero skips the dedicated category heads. The global GfxObject
    // execution list is empty, so loc_FD4 falls straight through loc_FE6.
    for cycles in [12, 10, 4, 12, 16, 16, 8, 12, 16, 10, 28, 16, 44, 16] {
        emit(cycles);
    }
    // Restore a1, return to sub_1C54A, initialise its six visible fields, and
    // return to sub_1B8F4's BSR.W sub_1C4EA.
    for cycles in [6, 16, 16, 16, 16, 16, 16, 16, 18] {
        emit(cycles);
    }

    timing
}

/// Visit `sub_1C4EA` through entry to the text-worker `sub_DAC` allocation.
pub fn visit_title_intro_palette_copy_to_text_object_allocation(
    mut visit: impl FnMut(u32),
) -> IntroSourceTiming {
    let mut timing = IntroSourceTiming {
        instruction_count: 0,
        raw_cycles: M68kCycleCount::from_raw(0),
    };
    let mut emit = |cycles: u32| {
        visit(cycles);
        timing.instruction_count += 1;
        timing.raw_cycles = M68kCycleCount::from_raw(
            timing
                .raw_cycles
                .get()
                .checked_add(cycles)
                .expect("title palette-copy timing fits u32"),
        );
    };

    // ASM sub_1C4EA copies Pal_1C28A's inclusive $32+1 words.
    for cycles in [12, 8, 4] {
        emit(cycles);
    }
    for index in 0..=0x32 {
        emit(12);
        emit(if index == 0x32 { 14 } else { 10 });
    }
    // Return to sub_1B8F4, select insertion priority $FFFF, and enter sub_DAC
    // through the j_Allocate_ObjectSlot jump stub.
    for cycles in [16, 8, 18, 10] {
        emit(cycles);
    }

    timing
}

/// Visit the title text-worker `sub_DAC` allocation through `sub_1C204`.
///
/// The controller already occupies the Object list. Priority `$FFFF` compares
/// equal, so the new text worker is inserted after one `loc_DE0` iteration.
pub fn visit_title_text_object_allocation_to_fade(mut visit: impl FnMut(u32)) -> IntroSourceTiming {
    let mut timing = IntroSourceTiming {
        instruction_count: 0,
        raw_cycles: M68kCycleCount::from_raw(0),
    };
    let mut emit = |cycles: u32| {
        visit(cycles);
        timing.instruction_count += 1;
        timing.raw_cycles = M68kCycleCount::from_raw(
            timing
                .raw_cycles
                .get()
                .checked_add(cycles)
                .expect("title text-object allocation timing fits u32"),
        );
    };

    // ASM Allocate_ObjectSlot sub_DAC setup and 29-longword clear.
    for cycles in [40, 4, 16, 28, 4, 4, 8] {
        emit(cycles);
    }
    for index in 0..29 {
        emit(12);
        emit(if index == 28 { 14 } else { 10 });
    }
    for cycles in [16, 16, 16, 8, 12] {
        emit(cycles);
    }
    // ASM loc_DE0 visits the existing controller once. Equal priorities do
    // not take BHI, then loc_DF2 links the worker after it.
    for cycles in [16, 8, 4, 4, 12, 8, 4, 28, 16, 44, 16] {
        emit(cycles);
    }
    // ASM sub_1B8F4 initialises code/text/continuation pointers and enters
    // the shared fade routine sub_1C204.
    for cycles in [24, 24, 24, 18] {
        emit(cycles);
    }

    timing
}

/// Visit the first `sub_1C204` palette clear through return to `$794`.
///
/// After clearing 64 palette words, the controller hibernates for one frame.
/// Its successor remains null because the equal-priority text worker was
/// inserted before it, so `loc_E1E` exits `Execute_Objects` immediately.
pub fn visit_title_first_fade_hibernate_to_mode_standard(
    mut visit: impl FnMut(u32),
) -> IntroSourceTiming {
    let mut timing = IntroSourceTiming {
        instruction_count: 0,
        raw_cycles: M68kCycleCount::from_raw(0),
    };
    let mut emit = |cycles: u32| {
        visit(cycles);
        timing.instruction_count += 1;
        timing.raw_cycles = M68kCycleCount::from_raw(
            timing
                .raw_cycles
                .get()
                .checked_add(cycles)
                .expect("title first-fade timing fits u32"),
        );
    };

    // ASM sub_1C204 setup and inclusive 64-word Palette_Buffer_3 clear.
    for cycles in [4, 12, 12] {
        emit(cycles);
    }
    for index in 0..=0x3F {
        emit(8);
        emit(if index == 0x3F { 14 } else { 10 });
    }
    // Reset fade state, mark both dirty ranges, and enter the first hibernate.
    for cycles in [16, 10, 4, 16, 16, 10, 18, 10] {
        emit(cycles);
    }
    // ASM Hibernate_Object_1Frame sub_E76 saves the controller coroutine and
    // all eight packed registers before resuming the execution-list walk.
    for cycles in [16, 24, 16, 76, 16, 10] {
        emit(cycles);
    }
    // The current controller has no successor; loc_E58 restores the caller
    // stack and returns to Mode_Standard $794.
    for cycles in [12, 10, 16, 16] {
        emit(cycles);
    }

    timing
}

/// Visit the first title `GfxObjects_MoveAndAnimate sub_1444` call.
///
/// The list contains one stationary, unanimated prompt GfxObject. Samurai
/// haze is clear, so both movement and animation take their inactive paths
/// before the final null successor returns to `Mode_Standard $798`.
pub fn visit_title_first_gfx_object_move_and_animate(
    mut visit: impl FnMut(u32),
) -> IntroSourceTiming {
    let mut timing = IntroSourceTiming {
        instruction_count: 0,
        raw_cycles: M68kCycleCount::from_raw(0),
    };
    let mut emit = |cycles: u32| {
        visit(cycles);
        timing.instruction_count += 1;
        timing.raw_cycles = M68kCycleCount::from_raw(
            timing
                .raw_cycles
                .get()
                .checked_add(cycles)
                .expect("title GfxObject timing fits u32"),
        );
    };

    // ASM Mode_Standard $794 and sub_1444 through the first live node.
    for cycles in [18, 12, 8, 8, 16, 8, 4, 18] {
        emit(cycles);
    }
    // ASM GfxObject_Move sub_1120: is_moved is clear.
    for cycles in [12, 8, 12, 10, 16] {
        emit(cycles);
    }
    // ASM GfxObject_Animate sub_113E: is_animated is clear.
    for cycles in [18, 12, 12, 12, 12, 10, 12, 16] {
        emit(cycles);
    }
    // ASM loc_144E observes the null successor and returns to $798.
    for cycles in [10, 16, 10, 16] {
        emit(cycles);
    }

    timing
}

/// Destination selected by a 68000-to-VDP DMA command.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MemoryToVdpDmaTarget {
    Vram,
    Cram,
    Vsram,
}

/// Source-positioned event within a routine that arbitrates Z80 and VDP DMA.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MemoryDmaTimingEvent {
    Instruction(u32),
    SetZ80BusRequest(bool),
    WriteVdpWord {
        target: MemoryToVdpDmaTarget,
    },
    BeginMemoryDma {
        words: u16,
        target: MemoryToVdpDmaTarget,
    },
}

/// Visit the first title `ReadJoypad` call from `$798` through `$79C`.
///
/// The first-controller, non-demo path reads both hardware ports with the
/// default ABC mapping. `sub_914` and `sub_924` bracket those reads with Z80
/// BUSREQ ownership, whose side effects are retained at their source writes.
pub fn visit_title_first_read_joypad(
    mut visit: impl FnMut(MemoryDmaTimingEvent),
) -> IntroSourceTiming {
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
                .expect("title ReadJoypad timing fits u32"),
        );
    };
    let emit_all = |cycles: &[u32],
                    visit: &mut dyn FnMut(MemoryDmaTimingEvent),
                    timing: &mut IntroSourceTiming| {
        for &cycle in cycles {
            emit(visit, timing, cycle);
        }
    };

    // ASM $798, ReadJoypad loc_B6C -> sub_914, and Stop_z80 entry.
    emit_all(
        &[18, 12, 10, 20, 20, 12, 10, 18, 10, 16, 16, 20],
        &mut visit,
        &mut timing,
    );
    emit(&mut visit, &mut timing, 20);
    visit(MemoryDmaTimingEvent::SetZ80BusRequest(true));
    emit_all(&[20, 8, 16], &mut visit, &mut timing);

    // First hardware port and default Permute_ABCButtons return.
    emit_all(&[16, 12, 18], &mut visit, &mut timing);
    emit_all(
        &[12, 4, 4, 8, 10, 8, 12, 4, 4, 8, 8, 4, 4, 16],
        &mut visit,
        &mut timing,
    );
    emit(&mut visit, &mut timing, 18);
    emit_all(&[12, 12, 12, 10, 16], &mut visit, &mut timing);
    emit_all(&[12, 4, 12, 18], &mut visit, &mut timing);

    // Second hardware port and default Permute_ABCButtons return.
    emit_all(
        &[12, 4, 4, 8, 10, 8, 12, 4, 4, 8, 8, 4, 4, 16],
        &mut visit,
        &mut timing,
    );
    emit(&mut visit, &mut timing, 18);
    emit_all(&[12, 12, 12, 10, 16], &mut visit, &mut timing);

    // ASM sub_924 reaches Start_z80 and releases BUSREQ.
    emit_all(&[12, 18, 10, 16, 8, 16, 16, 20], &mut visit, &mut timing);
    emit(&mut visit, &mut timing, 20);
    visit(MemoryDmaTimingEvent::SetZ80BusRequest(false));
    emit_all(&[16, 16, 12, 10], &mut visit, &mut timing);

    // ASM loc_C4E updates held/pressed edges and loc_C9C selects player one.
    emit_all(
        &[
            12, 12, 4, 12, 16, 20, 12, 12, 4, 12, 16, 20, 12, 10, 20, 20, 16,
        ],
        &mut visit,
        &mut timing,
    );

    timing
}

/// Visit the first title `Make_SpritesFromGfxObjects sub_1492` call.
///
/// The sole prompt GfxObject is visible, normal-oriented, and maps a 62x47
/// image into four SAT entries: two 32x32 entries followed by two 32x16
/// entries. The instruction journal retains all four `sub_13C0` calls through
/// return to `Mode_Standard $7A0`.
pub fn visit_title_first_make_sprites_from_gfx_objects(
    mut visit: impl FnMut(u32),
) -> IntroSourceTiming {
    const INSTRUCTION_CYCLES: [u32; 187] = [
        18, 16, 16, 16, 8, 16, 8, 4, 18, 16, 12, 12, 12, 12, 18, 12, 4, 12, 12, 12, 8, 12, 10, 4,
        8, 12, 8, 12, 12, 12, 8, 12, 8, 12, 16, 4, 12, 12, 16, 10, 12, 8, 8, 12, 10, 12, 12, 10,
        12, 12, 10, 8, 4, 4, 8, 8, 4, 4, 8, 8, 8, 4, 4, 8, 4, 4, 4, 8, 4, 18, 12, 12, 12, 16, 20,
        12, 8, 20, 4, 12, 4, 14, 4, 16, 8, 16, 4, 4, 10, 4, 10, 4, 8, 8, 4, 4, 8, 8, 18, 12, 12,
        12, 16, 20, 12, 8, 20, 4, 12, 4, 14, 4, 16, 8, 16, 4, 4, 10, 4, 10, 4, 8, 8, 4, 8, 4, 4,
        12, 8, 4, 18, 12, 12, 12, 16, 20, 12, 8, 20, 4, 12, 4, 14, 4, 16, 8, 16, 4, 4, 10, 4, 10,
        4, 8, 8, 4, 4, 4, 8, 4, 12, 4, 18, 12, 12, 12, 16, 20, 12, 8, 20, 4, 12, 4, 14, 4, 16, 8,
        16, 16, 10, 16, 10, 16, 16, 12, 16,
    ];

    let mut timing = IntroSourceTiming {
        instruction_count: 0,
        raw_cycles: M68kCycleCount::from_raw(0),
    };
    for cycles in INSTRUCTION_CYCLES {
        visit(cycles);
        timing.instruction_count += 1;
        timing.raw_cycles = M68kCycleCount::from_raw(
            timing
                .raw_cycles
                .get()
                .checked_add(cycles)
                .expect("title sprite-construction timing fits u32"),
        );
    }
    timing
}

/// Visit a full-dirty-mask `BSR.W sub_14C0` palette interpolation call.
///
/// Both 32-bit dirty masks are `$FFFFFFFF` after `sub_1C204`, so the routine
/// builds all 15 signed interpolation steps and runs the full 32-colour
/// `sub_1536` body for both palette halves before returning. The title's first
/// frame and every `Pal_FadeOut loc_28C6` step share this path.
pub fn visit_full_palette_interpolation_call(mut visit: impl FnMut(u32)) -> IntroSourceTiming {
    let mut timing = IntroSourceTiming {
        instruction_count: 0,
        raw_cycles: M68kCycleCount::from_raw(0),
    };
    let mut emit = |cycles: u32| {
        visit(cycles);
        timing.instruction_count += 1;
        timing.raw_cycles = M68kCycleCount::from_raw(
            timing
                .raw_cycles
                .get()
                .checked_add(cycles)
                .expect("title palette-interpolation timing fits u32"),
        );
    };

    // ASM Mode_Standard $7A0 / Pal_FadeOut $28D2 and sub_14C0 setup through
    // loc_14EE.
    for cycles in [18, 16, 16, 8, 10, 12, 12, 16, 4, 4, 14, 16, 8, 4, 4] {
        emit(cycles);
    }
    // ASM loc_14EE writes 15 inclusive signed interpolation steps.
    for index in 0..15 {
        for cycles in [8, 4, 8, 4] {
            emit(cycles);
        }
        emit(if index == 14 { 14 } else { 10 });
    }
    for cycles in [4, 12, 12, 12, 4, 8, 24, 18] {
        emit(cycles);
    }

    let emit_full_palette_half = |emit: &mut dyn FnMut(u32)| {
        // ASM sub_1536: every dirty-mask bit is set, so all 32 colours take
        // loc_1546 rather than the loc_158E two-byte skip.
        emit(4);
        const COLOUR_BODY: [u32; 35] = [
            8, 8, 8, 8, 6, 8, 4, 4, 4, 4, 4, 8, 14, 4, 4, 14, 4, 4, 14, 4, 4, 4, 8, 14, 14, 4, 14,
            14, 4, 8, 14, 14, 4, 22, 8,
        ];
        for colour in 0..32 {
            for cycles in COLOUR_BODY {
                emit(cycles);
            }
            emit(if colour == 31 { 14 } else { 10 });
        }
        emit(16);
    };
    emit_full_palette_half(&mut emit);

    // ASM loc_1518 selects and clears the second dirty mask before falling
    // through loc_1534 into the second full sub_1536 pass.
    for cycles in [12, 12, 12, 8, 12, 10, 20] {
        emit(cycles);
    }
    emit_full_palette_half(&mut emit);

    timing
}

/// Visit the first title `sub_14C0` interpolation through frame RTS.
pub fn visit_title_first_palette_interpolation(visit: impl FnMut(u32)) -> IntroSourceTiming {
    visit_full_palette_interpolation_call(visit)
}

/// Visit a `BSR.W Palette_to_VRAM` call through its return.
///
/// ASM `Palette_to_VRAM $970`, `Stop_z80 $E13BC`, and `Start_z80 $E13D0`
/// retain their instruction boundaries. BUSREQ side effects occur after the
/// corresponding register write, while the CRAM DMA begins after the second
/// VDP command word.
pub fn visit_palette_to_vram_call(
    mut visit: impl FnMut(MemoryDmaTimingEvent),
) -> IntroSourceTiming {
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
                .expect("Palette_to_VRAM source timing fits u32"),
        );
    };

    // ASM Mode_Standard $77E / Pal_FadeOut $28CE and Palette_to_VRAM $970:
    // BSR, flag test, fall-through branch and Stop_z80 JSR.
    for cycles in [18, 12, 8, 20] {
        emit(&mut visit, &mut timing, cycles);
    }
    // ASM j_Stop_z80 $E1304 -> BranchTo_Stop_z80 -> Stop_z80 $E13BC.
    for cycles in [10, 10, 20] {
        emit(&mut visit, &mut timing, cycles);
    }
    visit(MemoryDmaTimingEvent::SetZ80BusRequest(true));
    for cycles in [20, 8, 16] {
        emit(&mut visit, &mut timing, cycles);
    }

    // ASM dma68kToVDP Palette_Buffer,$0000,$80,CRAM.
    for cycles in [24, 24, 16, 24, 20, 20] {
        emit(&mut visit, &mut timing, cycles);
    }
    visit(MemoryDmaTimingEvent::BeginMemoryDma {
        words: 0x40,
        target: MemoryToVdpDmaTarget::Cram,
    });

    // ASM j_Start_z80 $E1308 -> BranchTo_Start_z80 -> Start_z80 $E13D0.
    for cycles in [20, 10, 10, 20] {
        emit(&mut visit, &mut timing, cycles);
    }
    visit(MemoryDmaTimingEvent::SetZ80BusRequest(false));
    // Start_z80 RTS, then Palette_to_VRAM RTS.
    for cycles in [16, 16] {
        emit(&mut visit, &mut timing, cycles);
    }

    timing
}

/// Visit `Mode_Standard`'s first `Palette_to_VRAM` call from caller `$77E`.
pub fn visit_mode_standard_palette_to_vram(
    visit: impl FnMut(MemoryDmaTimingEvent),
) -> IntroSourceTiming {
    visit_palette_to_vram_call(visit)
}

/// Visit gameplay's `Palette_to_VRAM` branch from `Mode_Level $7CC`.
///
/// ASM `Palette_to_VRAM $970` skips Stop_z80, the CRAM DMA and Start_z80
/// while `PaletteToDMA_Flag` is non-zero. `sub_7196` sets the flag before the
/// first gameplay pass; its `sub_73D0` worker clears it later in that pass.
pub fn visit_mode_level_palette_to_vram(
    palette_dma_sub_970: bool,
    mut visit: impl FnMut(MemoryDmaTimingEvent),
) -> IntroSourceTiming {
    if palette_dma_sub_970 {
        return visit_palette_to_vram_call(visit);
    }

    let mut timing = IntroSourceTiming {
        instruction_count: 0,
        raw_cycles: M68kCycleCount::from_raw(0),
    };
    // ASM Mode_Level $7CC BSR, Palette_to_VRAM $970 TST/BNE and RTS.
    for cycles in [18, 12, 10, 16] {
        visit(MemoryDmaTimingEvent::Instruction(cycles));
        timing.instruction_count += 1;
        timing.raw_cycles = M68kCycleCount::from_raw(
            timing
                .raw_cycles
                .get()
                .checked_add(cycles)
                .expect("Palette_to_VRAM skip timing fits u32"),
        );
    }
    timing
}

/// Visit the first `Pal_FadeOut` decrement and loop back to the next VINT poll.
///
/// ASM `loc_28C6` returns from `sub_14C0` at $28D6, decrements fade progress
/// from `$0100` to `$00F0`, takes the non-negative path through `loc_28DE`,
/// seeds both dirty masks, branches to `loc_28C6`, calls `WaitForVint`, and
/// clears `V_Int_Done`. The visitor stops immediately before the first $93E
/// poll instruction.
pub fn visit_palette_fade_first_decrement_to_next_wait_poll(
    mut visit: impl FnMut(u32),
) -> IntroSourceTiming {
    let mut timing = IntroSourceTiming {
        instruction_count: 0,
        raw_cycles: M68kCycleCount::from_raw(0),
    };

    // ASM Pal_FadeOut $28D6..$28C6 and WaitForVint $93A.
    for cycles in [20, 8, 4, 16, 16, 10, 18, 16] {
        visit(cycles);
        timing.instruction_count += 1;
        timing.raw_cycles = M68kCycleCount::from_raw(
            timing
                .raw_cycles
                .get()
                .checked_add(cycles)
                .expect("palette-fade loop timing fits u32"),
        );
    }

    timing
}

/// Visit the negative `Pal_FadeOut` decrement through its final VINT poll.
///
/// ASM `loc_28C6` returns from the seventeenth full `sub_14C0` call at $28D6,
/// decrements progress from `$0000` to `$FFF0`, takes `bmi.s loc_28EA`, calls
/// `WaitForVint`, and clears `V_Int_Done`. The visitor stops immediately
/// before the first $93E poll instruction.
pub fn visit_palette_fade_negative_decrement_to_final_wait_poll(
    mut visit: impl FnMut(u32),
) -> IntroSourceTiming {
    let mut timing = IntroSourceTiming {
        instruction_count: 0,
        raw_cycles: M68kCycleCount::from_raw(0),
    };

    // ASM Pal_FadeOut $28D6/$28DC/$28EA and WaitForVint $93A.
    for cycles in [20, 10, 18, 16] {
        visit(cycles);
        timing.instruction_count += 1;
        timing.raw_cycles = M68kCycleCount::from_raw(
            timing
                .raw_cycles
                .get()
                .checked_add(cycles)
                .expect("negative palette-fade timing fits u32"),
        );
    }

    timing
}

/// Visit a clean-dirty-mask `BSR.W sub_14C0` call through its early return.
///
/// After the negative fade step, both masks have already been consumed. ASM
/// `sub_14C0` therefore ORs two zero longwords, does not take `loc_14CE`, and
/// returns directly to its caller.
pub fn visit_clean_palette_interpolation_call(mut visit: impl FnMut(u32)) -> IntroSourceTiming {
    let mut timing = IntroSourceTiming {
        instruction_count: 0,
        raw_cycles: M68kCycleCount::from_raw(0),
    };

    // ASM Pal_FadeOut $28F6 and sub_14C0 $14C0..$14CC.
    for cycles in [18, 16, 16, 8, 8, 16] {
        visit(cycles);
        timing.instruction_count += 1;
        timing.raw_cycles = M68kCycleCount::from_raw(
            timing
                .raw_cycles
                .get()
                .checked_add(cycles)
                .expect("clean palette-interpolation timing fits u32"),
        );
    }

    timing
}

/// Visit the `Pal_FadeOut` return and caller clear through `loc_700`.
pub fn visit_palette_fade_return_to_loc_700(mut visit: impl FnMut(u32)) -> IntroSourceTiming {
    // ASM Pal_FadeOut $28FA: RTS; loc_6E2 $6FC: CLR.W $FFFFFBCC.
    visit(16);
    visit(16);
    IntroSourceTiming {
        instruction_count: 2,
        raw_cycles: M68kCycleCount::from_raw(32),
    }
}

/// Visit the complete `loc_700/loc_70E` CPU-driven VRAM clear.
///
/// ASM `loc_700` selects VRAM address zero and seeds `d0=$3FFF`, `d1=0`.
/// `loc_70E` then writes 16,384 zero longwords to the VDP data port. Each
/// longword occupies two FIFO words; the caller must schedule both writes so
/// FIFO backpressure remains visible to the 68000 and GEMS clocks. The visitor
/// stops at `loc_714`, before the Sprite_Table clear setup.
pub fn visit_loc_700_vram_clear(mut visit: impl FnMut(MemoryDmaTimingEvent)) -> IntroSourceTiming {
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
                .expect("loc_700 VRAM-clear timing fits u32"),
        );
    };

    // ASM loc_700: VDP command, DBF count, and zero longword.
    for cycles in [24, 8, 4] {
        emit(&mut visit, &mut timing, cycles);
    }
    for iteration in 0..0x4000 {
        // ASM loc_70E: MOVE.L d1,(a6) enqueues two consecutive VRAM words.
        emit(&mut visit, &mut timing, 12);
        visit(MemoryDmaTimingEvent::WriteVdpWord {
            target: MemoryToVdpDmaTarget::Vram,
        });
        visit(MemoryDmaTimingEvent::WriteVdpWord {
            target: MemoryToVdpDmaTarget::Vram,
        });
        emit(
            &mut visit,
            &mut timing,
            if iteration == 0x3FFF { 14 } else { 10 },
        );
    }

    timing
}

/// Visit the complete `loc_714/loc_720` Sprite_Table clear.
///
/// ASM loads `a0=Sprite_Table`, seeds `d0=$7DE0`, and clears 32,225 words
/// with an inclusive DBF loop. The visitor stops at $726 before
/// `Initialize_ObjectSlots`.
pub fn visit_loc_714_sprite_table_clear(
    mut visit: impl FnMut(MemoryDmaTimingEvent),
) -> IntroSourceTiming {
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
                .expect("loc_714 Sprite_Table-clear timing fits u32"),
        );
    };

    // ASM loc_714: LEA Sprite_Table,a0; MOVE.W #$7DE0,d0; MOVEQ #0,d1.
    for cycles in [12, 8, 4] {
        emit(&mut visit, &mut timing, cycles);
    }
    for iteration in 0..0x7DE1 {
        emit(&mut visit, &mut timing, 8);
        emit(
            &mut visit,
            &mut timing,
            if iteration == 0x7DE0 { 14 } else { 10 },
        );
    }

    timing
}

/// Visit the title-restart setup from `loc_726` to `sub_E151C`.
pub fn visit_title_restart_initialisers_to_play_sound(visit: impl FnMut(u32)) -> IntroSourceTiming {
    visit_restart_initialisers_to_dispatch(visit, &[20, 10, 12, 18, 16, 8, 20, 10, 12, 12, 18])
}

/// Visit the demo restart from `loc_726` through `Load_DemoPlay` to
/// `Load_InGame $11A0E`.
///
/// ASM `Load_DemoPlay` selects demo 1, allocates the hidden `loc_119D8`
/// controller into the freshly cleared execution list, then calls
/// `Load_InGame`. The visitor stops on that routine's first instruction.
pub fn visit_demo_restart_initialisers_to_load_in_game(
    visit: impl FnMut(u32),
) -> IntroSourceTiming {
    let mut dispatch_tail = vec![20, 10, 12, 18, 16];
    // ASM Load_DemoPlay $11990..loc_119B6: select demo 1 and install its
    // recording pointer and level ID.
    // Oracle selected-PC tracing reports 12 base cycles for the taken
    // `beq.w loc_119B6`, followed by a 20-cycle 68000 refill interval before
    // the target instruction begins. Keep that source boundary intact.
    dispatch_tail.extend([16, 12, 8, 16, 32, 16, 12]);
    // ASM loc_119B6: allocate the hidden controller through the jump thunk.
    dispatch_tail.extend([8, 18, 10]);
    visit_empty_list_allocate_object_slot_instructions(&mut |cycles| {
        dispatch_tail.push(cycles);
    });
    // ASM loc_119B6: install loc_119D8, clear $FBC2, and call Load_InGame.
    dispatch_tail.extend([24, 16, 18]);
    visit_restart_initialisers_to_dispatch(visit, &dispatch_tail)
}

/// Visit power-on `loc_726` through `Load_SegaScreen`'s second `StopMusic`.
pub fn visit_power_on_restart_initialisers_to_stop_music(
    visit: impl FnMut(u32),
) -> IntroSourceTiming {
    visit_restart_initialisers_to_dispatch(visit, &[20, 10, 12, 18, 16, 20, 10])
}

/// ASM `loc_726` calls `Initialize_ObjectSlots`,
/// `Initialize_GfxObjectSlots`, and `sub_8A4`, then dispatches the current
/// game mode through `LoadGameModeData`. `dispatch_tail` selects the branch
/// from that shared setup to the caller's next observable routine.
fn visit_restart_initialisers_to_dispatch(
    mut visit: impl FnMut(u32),
    dispatch_tail: &[u32],
) -> IntroSourceTiming {
    let mut timing = IntroSourceTiming {
        instruction_count: 0,
        raw_cycles: M68kCycleCount::from_raw(0),
    };
    let emit = |visit: &mut dyn FnMut(u32), timing: &mut IntroSourceTiming, cycles: u32| {
        visit(cycles);
        timing.instruction_count += 1;
        timing.raw_cycles = M68kCycleCount::from_raw(
            timing
                .raw_cycles
                .get()
                .checked_add(cycles)
                .expect("title restart initialiser timing fits u32"),
        );
    };

    // ASM loc_726: BSR.W Initialize_ObjectSlots.
    emit(&mut visit, &mut timing, 18);
    for cycles in [12, 16, 4] {
        emit(&mut visit, &mut timing, cycles);
    }
    for iteration in 0..49 {
        for cycles in [8, 16, 4] {
            emit(&mut visit, &mut timing, cycles);
        }
        emit(
            &mut visit,
            &mut timing,
            if iteration == 48 { 14 } else { 10 },
        );
    }
    for cycles in [24, 8, 20, 16, 16, 16] {
        emit(&mut visit, &mut timing, cycles);
    }

    // ASM loc_72A: BSR.W Initialize_GfxObjectSlots.
    emit(&mut visit, &mut timing, 18);
    for cycles in [12, 16, 4] {
        emit(&mut visit, &mut timing, cycles);
    }
    for iteration in 0..76 {
        for cycles in [8, 16, 4] {
            emit(&mut visit, &mut timing, cycles);
        }
        emit(
            &mut visit,
            &mut timing,
            if iteration == 75 { 14 } else { 10 },
        );
    }
    for cycles in [24, 24, 16, 24, 24, 24, 24, 16] {
        emit(&mut visit, &mut timing, cycles);
    }

    // ASM loc_72E/sub_8A4: copy the active player's eleven state words.
    emit(&mut visit, &mut timing, 18);
    for cycles in [8, 12, 10, 4, 8] {
        emit(&mut visit, &mut timing, cycles);
    }
    for iteration in 0..11 {
        emit(&mut visit, &mut timing, 12);
        emit(
            &mut visit,
            &mut timing,
            if iteration == 10 { 14 } else { 10 },
        );
    }
    emit(&mut visit, &mut timing, 16);

    // ASM loc_732 -> j_LoadGameModeData. The indexed GameLoadArray read costs
    // 18 cycles; the remaining source instructions select the requested load.
    for &cycles in dispatch_tail {
        emit(&mut visit, &mut timing, cycles);
    }

    timing
}

/// Visit the Ice command return and `Load_TitleScreen` VRAM fill.
///
/// ASM `sub_E151C` has already returned to `PlaySound`. Its three-instruction
/// epilogue returns to `Load_TitleScreen`, which selects VRAM zero and writes
/// 2,048 copies of `$20BC` at `loc_1B258`. The visitor includes the following
/// `BSR.W sub_1B2A4` and stops at that routine's entry.
pub fn visit_title_load_screen_vram_fill_to_sub_1b2a4(
    mut visit: impl FnMut(MemoryDmaTimingEvent),
) -> IntroSourceTiming {
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
                .expect("title-screen VRAM-fill timing fits u32"),
        );
    };

    // ASM PlaySound $E1516..$E151A, then Load_TitleScreen setup.
    for cycles in [12, 12, 16, 24, 8] {
        emit(&mut visit, &mut timing, cycles);
    }
    for iteration in 0..0x800 {
        // ASM loc_1B258: MOVE.W #$20BC,(a6) enqueues one VRAM word.
        emit(&mut visit, &mut timing, 12);
        visit(MemoryDmaTimingEvent::WriteVdpWord {
            target: MemoryToVdpDmaTarget::Vram,
        });
        emit(
            &mut visit,
            &mut timing,
            if iteration == 0x7FF { 14 } else { 10 },
        );
    }
    // ASM $1B25C: BSR.W sub_1B2A4.
    emit(&mut visit, &mut timing, 18);

    timing
}

/// Visit the title-map upload and first `Load_TitleArt` palette permutation.
///
/// ASM `sub_1B2A4` resumes after the `unk_2F7D0` `EniDec`, writes the
/// 32-word `loc_1B322` map, copies the 51-word title palette at `loc_1C4BE`,
/// and enters `Load_TitleArt -> DecompressToRAM`. The visitor stops at
/// `loc_143EA`, after `loc_143CC/loc_143CE` has built all 16 permutation rows.
pub fn visit_title_map_to_load_art_palette_permutation(
    mut visit: impl FnMut(MemoryDmaTimingEvent),
) -> IntroSourceTiming {
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
                .expect("title-art palette timing fits u32"),
        );
    };

    // ASM sub_1B2A4 after EniDec: select VRAM $F000 and seed loc_1B322.
    for cycles in [20, 8, 4] {
        emit(&mut visit, &mut timing, cycles);
    }
    for iteration in 0..32 {
        emit(&mut visit, &mut timing, 12);
        visit(MemoryDmaTimingEvent::WriteVdpWord {
            target: MemoryToVdpDmaTarget::Vram,
        });
        emit(
            &mut visit,
            &mut timing,
            if iteration == 31 { 14 } else { 10 },
        );
    }

    // ASM loc_1B322 -> loc_1C4B2 copies 51 palette words, returns from
    // sub_1B2A4, and calls Load_TitleArt's first DecompressToRAM.
    for cycles in [12, 12, 8, 4] {
        emit(&mut visit, &mut timing, cycles);
    }
    for iteration in 0..51 {
        emit(&mut visit, &mut timing, 12);
        emit(
            &mut visit,
            &mut timing,
            if iteration == 50 { 14 } else { 10 },
        );
    }
    for cycles in [12, 10, 16, 18, 8, 12, 8, 20, 10] {
        emit(&mut visit, &mut timing, cycles);
    }

    // ASM DecompressToRAM -> sub_142CC, followed by the palette-table setup.
    for cycles in [
        18, 4, 10, 8, 8, 8, 4, 4, 16, 8, 22, 8, 12, 12, 4, 4, 16, 8, 4,
    ] {
        emit(&mut visit, &mut timing, cycles);
    }

    // ASM loc_143CC/loc_143CE builds a 16x16 packed-nibble lookup table.
    for row in 0..16 {
        emit(&mut visit, &mut timing, 4);
        for column in 0..16 {
            for cycles in [14, 8, 14, 8, 4, 8] {
                emit(&mut visit, &mut timing, cycles);
            }
            emit(&mut visit, &mut timing, if column == 15 { 8 } else { 10 });
        }
        for cycles in [4, 8, if row == 15 { 8 } else { 10 }] {
            emit(&mut visit, &mut timing, cycles);
        }
    }

    timing
}

/// Visit `sub_1BF24` after its Enigma decode through `sub_1B532`'s decode.
///
/// ASM `sub_1BF24` (`kid.asm`) clears the 18x20 staging map, copies the
/// selected width from `Decompression_Buffer`, and uploads it through
/// `sub_1C5D0`. The caller then enters `sub_1B532`, clears 16 VRAM words at
/// `$1780`, and calls `EniDec` for `unk_2F840`. This visitor stops at that
/// decoder's entry.
pub fn visit_title_character_map_to_chameleon_enigma(
    mut visit: impl FnMut(MemoryDmaTimingEvent),
) -> IntroSourceTiming {
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
                .expect("title character-map timing fits u32"),
        );
    };

    // ASM sub_1BF24 loc_1BF62..loc_1BF70: clear 360 staging words.
    for cycles in [8, 12, 8] {
        emit(&mut visit, &mut timing, cycles);
    }
    for iteration in 0..360 {
        emit(&mut visit, &mut timing, 8);
        emit(
            &mut visit,
            &mut timing,
            if iteration == 359 { 14 } else { 10 },
        );
    }

    // ASM loc_1BF78..loc_1BF8A: copy the selected 17-word Red Stealth row
    // into each of the 20 staging rows.
    for cycles in [12, 12, 4] {
        emit(&mut visit, &mut timing, cycles);
    }
    for row in 0..20 {
        for cycles in [8, 4, 4] {
            emit(&mut visit, &mut timing, cycles);
        }
        for column in 0..17 {
            emit(&mut visit, &mut timing, 12);
            emit(&mut visit, &mut timing, if column == 16 { 14 } else { 10 });
        }
        emit(&mut visit, &mut timing, 8);
        emit(&mut visit, &mut timing, if row == 19 { 14 } else { 10 });
    }

    // ASM sub_1BF24 $1BF94..$1BFA6 and sub_1C5D0: upload 18x20 words at
    // plane-A cell (21,1), then restore both register frames and return.
    for cycles in [12, 4, 4, 4, 4, 18, 40, 4, 20, 4, 4, 8, 4, 4, 4, 4] {
        emit(&mut visit, &mut timing, cycles);
    }
    for row in 0..20 {
        for cycles in [4, 16, 16] {
            emit(&mut visit, &mut timing, cycles);
        }
        for column in 0..18 {
            emit(&mut visit, &mut timing, 12);
            visit(MemoryDmaTimingEvent::WriteVdpWord {
                target: MemoryToVdpDmaTarget::Vram,
            });
            emit(&mut visit, &mut timing, if column == 17 { 14 } else { 10 });
        }
        emit(&mut visit, &mut timing, if row == 19 { 14 } else { 10 });
    }
    for cycles in [44, 16, 44, 16, 18] {
        emit(&mut visit, &mut timing, cycles);
    }

    // ASM sub_1B532 loc_1B532..loc_1B53E: clear eight longwords at VRAM
    // `$1780`; each MOVE.L contributes two FIFO words.
    for cycles in [20, 4, 4] {
        emit(&mut visit, &mut timing, cycles);
    }
    for iteration in 0..8 {
        emit(&mut visit, &mut timing, 12);
        for _ in 0..2 {
            visit(MemoryDmaTimingEvent::WriteVdpWord {
                target: MemoryToVdpDmaTarget::Vram,
            });
        }
        emit(
            &mut visit,
            &mut timing,
            if iteration == 7 { 14 } else { 10 },
        );
    }

    // ASM sub_1B532 $1B542..$1B560: prepare unk_2F840 and take the jump
    // stub into EniDec.
    for cycles in [8, 12, 12, 4, 20, 10] {
        emit(&mut visit, &mut timing, cycles);
    }

    timing
}

/// Visit the title path through `Transfer_SpriteAndKidToVRAM`.
///
/// The title has a live sprite-table pointer, `$FB49` clear, and a zero first
/// Kid-art length at `$FF0500`. It therefore performs the fixed 320-word
/// Sprite_Table DMA, skips the Kid-art DMA loop at `loc_AD4`, then resets the
/// two producer pointers through `sub_B52`.
pub fn visit_mode_standard_sprite_and_kid_to_vram(
    visit: impl FnMut(MemoryDmaTimingEvent),
) -> IntroSourceTiming {
    visit_sprite_and_kid_to_vram(&[], false, visit)
}

/// Visit a classified gameplay path through `Transfer_SpriteAndKidToVRAM`.
///
/// ASM `Mode_Level $7C8` calls `sub_A5C`; `loc_AD4` consumes the leading
/// non-zero tile lengths produced by the preceding `sub_1492 -> sub_13C0
/// loc_141A` pass, then stops at the first zero word. One hardware sprite can
/// contribute at most 16 tiles (32x32).
pub fn visit_mode_level_sprite_and_kid_to_vram(
    dynamic_art_tiles_loc_ad4: [u16; 6],
    visit: impl FnMut(MemoryDmaTimingEvent),
) -> Option<IntroSourceTiming> {
    let used = dynamic_art_tiles_loc_ad4
        .iter()
        .position(|&tiles| tiles == 0)
        .unwrap_or(dynamic_art_tiles_loc_ad4.len());
    if dynamic_art_tiles_loc_ad4[..used]
        .iter()
        .any(|&tiles| tiles > 16)
        || dynamic_art_tiles_loc_ad4[used..]
            .iter()
            .any(|&tiles| tiles != 0)
    {
        return None;
    }
    Some(visit_sprite_and_kid_to_vram(
        &dynamic_art_tiles_loc_ad4[..used],
        true,
        visit,
    ))
}

fn visit_sprite_and_kid_to_vram(
    dynamic_art_tiles_loc_ad4: &[u16],
    selected_pc_exact: bool,
    mut visit: impl FnMut(MemoryDmaTimingEvent),
) -> IntroSourceTiming {
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
                .expect("sprite/Kid transfer source timing fits u32"),
        );
    };

    // ASM Mode_Standard $782; sub_A5C $A5C..$A6C live-pointer path.
    let entry: &[u32] = if selected_pc_exact {
        &[18, 16, 14, 12, 4, 16]
    } else {
        // TODO(fidelity): re-derive the earlier title GEMS chain before
        // replacing its historical sub_A5C timing approximation.
        &[18, 16, 14, 12, 4, 18]
    };
    for &cycles in entry {
        emit(&mut visit, &mut timing, cycles);
    }
    // ASM loc_A70 -> Stop_z80.
    for cycles in [20, 10, 10, 20] {
        emit(&mut visit, &mut timing, cycles);
    }
    visit(MemoryDmaTimingEvent::SetZ80BusRequest(true));
    for cycles in [20, 8, 16] {
        emit(&mut visit, &mut timing, cycles);
    }
    // ASM dma68kToVDP Sprite_Table,$1000,$280,VRAM.
    for cycles in [24, 24, 16, 24, 20, 20] {
        emit(&mut visit, &mut timing, cycles);
    }
    visit(MemoryDmaTimingEvent::BeginMemoryDma {
        words: 0x140,
        target: MemoryToVdpDmaTarget::Vram,
    });

    // Start_z80 immediately followed by a second Stop_z80.
    for cycles in [20, 10, 10, 20] {
        emit(&mut visit, &mut timing, cycles);
    }
    visit(MemoryDmaTimingEvent::SetZ80BusRequest(false));
    for cycles in [16, 20, 10, 10, 20] {
        emit(&mut visit, &mut timing, cycles);
    }
    visit(MemoryDmaTimingEvent::SetZ80BusRequest(true));
    for cycles in [20, 8, 16] {
        emit(&mut visit, &mut timing, cycles);
    }

    // ASM $AAC..loc_AD4: select the default Kid-art list and clear its
    // terminator before reading the first tile count.
    let default_list: &[u32] = if selected_pc_exact {
        &[12, 12, 16, 4, 8, 12, 12, 8, 12, 8]
    } else {
        &[12, 12, 16, 4, 10, 12, 4, 12, 8]
    };
    for &cycles in default_list {
        emit(&mut visit, &mut timing, cycles);
    }
    for &tiles in dynamic_art_tiles_loc_ad4 {
        // ASM loc_AD4..loc_B24: program one uncompressed Kid-art DMA. The
        // final two control-port writes start the transfer before d2 advances
        // to the next VRAM destination.
        const THROUGH_DMA_COMMAND: [u32; 35] = [
            8, 8, 12, 8, 8, 4, 8, 4, 8, 8, 8, 10, 8, 4, 8, 4, 8, 8, 8, 4, 8, 4, 8, 4, 8, 8, 4, 8,
            4, 8, 4, 16, 16, 12, 12,
        ];
        for cycles in THROUGH_DMA_COMMAND {
            emit(&mut visit, &mut timing, cycles);
        }
        visit(MemoryDmaTimingEvent::BeginMemoryDma {
            words: tiles * 16,
            target: MemoryToVdpDmaTarget::Vram,
        });
        for cycles in [4, 4, 10] {
            emit(&mut visit, &mut timing, cycles);
        }
    }
    // ASM loc_AD4: the zero terminator branches to loc_B36.
    for cycles in [8, 10] {
        emit(&mut visit, &mut timing, cycles);
    }
    // ASM loc_B36 -> Start_z80.
    for cycles in [20, 10, 10, 20] {
        emit(&mut visit, &mut timing, cycles);
    }
    visit(MemoryDmaTimingEvent::SetZ80BusRequest(false));
    // Start_z80 RTS; JSR.W sub_B52 and its pointer-reset body; outer RTS.
    let reset_and_return: &[u32] = if selected_pc_exact {
        &[16, 18, 10, 16, 12, 16, 12, 16, 16, 16]
    } else {
        &[16, 18, 12, 12, 16, 12, 16, 16, 16]
    };
    for &cycles in reset_and_return {
        emit(&mut visit, &mut timing, cycles);
    }

    timing
}

/// Visit the title path through `Transfer_ScrollDataToVRAM`.
///
/// The title has `Level_Special_Effects <= 1` and a clear background
/// no-scroll flag. It therefore takes `loc_300BA`, writes the camera-derived
/// VSRAM pair, and performs the `$380`-byte horizontal-scroll VRAM DMA.
pub fn visit_mode_standard_scroll_data_to_vram(
    visit: impl FnMut(MemoryDmaTimingEvent),
) -> IntroSourceTiming {
    visit_scroll_data_to_vram(false, visit)
}

/// Visit gameplay's selected-PC-exact normal scroll transfer.
pub fn visit_mode_level_scroll_data_to_vram(
    visit: impl FnMut(MemoryDmaTimingEvent),
) -> IntroSourceTiming {
    visit_scroll_data_to_vram(true, visit)
}

fn visit_scroll_data_to_vram(
    selected_pc_exact: bool,
    mut visit: impl FnMut(MemoryDmaTimingEvent),
) -> IntroSourceTiming {
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
                .expect("scroll-data transfer source timing fits u32"),
        );
    };

    // ASM Mode_Standard $786; Transfer_ScrollDataToVRAM $3009A takes the
    // title's BLE to loc_300BA and the clear Background_NoScrollFlag branch.
    let entry: &[u32] = if selected_pc_exact {
        &[20, 10, 12, 4, 10, 12, 24]
    } else {
        &[20, 12, 4, 10, 12, 20, 8]
    };
    for &cycles in entry {
        emit(&mut visit, &mut timing, cycles);
    }
    visit(MemoryDmaTimingEvent::WriteVdpWord {
        target: MemoryToVdpDmaTarget::Vsram,
    });
    let vsram: &[u32] = if selected_pc_exact {
        &[8, 10, 12, 10, 8]
    } else {
        &[10, 8, 10, 8]
    };
    for &cycles in vsram {
        emit(&mut visit, &mut timing, cycles);
    }
    visit(MemoryDmaTimingEvent::WriteVdpWord {
        target: MemoryToVdpDmaTarget::Vsram,
    });
    emit(&mut visit, &mut timing, 20);
    // ASM j_Stop_z80 $E1304 -> BranchTo_Stop_z80 -> Stop_z80 $E13BC.
    for cycles in [10, 10, 20] {
        emit(&mut visit, &mut timing, cycles);
    }
    visit(MemoryDmaTimingEvent::SetZ80BusRequest(true));
    for cycles in [20, 8, 16] {
        emit(&mut visit, &mut timing, cycles);
    }
    // ASM dma68kToVDP Horiz_Scroll_Buffer,$1400,$380,VRAM. The macro's
    // length is bytes and programs half that value into the VDP word count.
    for cycles in [24, 24, 16, 24, 20, 20] {
        emit(&mut visit, &mut timing, cycles);
    }
    visit(MemoryDmaTimingEvent::BeginMemoryDma {
        words: 0x1C0,
        target: MemoryToVdpDmaTarget::Vram,
    });

    // ASM direct Start_z80 after the DMA.
    for cycles in [20, 10, 10, 20] {
        emit(&mut visit, &mut timing, cycles);
    }
    visit(MemoryDmaTimingEvent::SetZ80BusRequest(false));
    emit(&mut visit, &mut timing, 16);

    // ASM j_sub_924 -> sub_924 takes the non-positive counter path and calls
    // Start_z80 once more before both routines return.
    let nested_start: &[u32] = if selected_pc_exact {
        &[18, 10, 16, 8, 16, 16, 20, 10, 10, 20]
    } else {
        &[18, 10, 12, 8, 12, 12, 20, 10, 10, 20]
    };
    for &cycles in nested_start {
        emit(&mut visit, &mut timing, cycles);
    }
    visit(MemoryDmaTimingEvent::SetZ80BusRequest(false));
    for cycles in [16, 16, 16] {
        emit(&mut visit, &mut timing, cycles);
    }

    timing
}

/// Visit `Allocate_ObjectSlot sub_DAC` with an empty execution list.
fn visit_empty_list_allocate_object_slot_instructions(visit: &mut impl FnMut(u32)) {
    // ASM sub_DAC $DAC..$DC0: save registers, pop the free slot and prepare
    // the 29-longword clear loop.
    for cycles in [40, 4, 16, 28, 4, 4, 8] {
        visit(cycles);
    }
    for index in 0..29 {
        visit(12);
        visit(if index == 28 { 14 } else { 10 });
    }
    // ASM sub_DAC $DCA..$DDA: initialise and select insertion priority.
    for cycles in [16, 16, 16, 8, 12] {
        visit(cycles);
    }
    // ASM loc_DE0 finds the empty execution list and loc_DF2 links the slot.
    for cycles in [16, 10, 28, 16, 44, 16] {
        visit(cycles);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn demo_restart_retains_loc_726_to_load_in_game_boundaries() {
        let mut cycles = Vec::new();
        let timing = visit_demo_restart_initialisers_to_load_in_game(|cycle| cycles.push(cycle));

        assert_eq!(
            timing,
            IntroSourceTiming {
                instruction_count: 645,
                raw_cycles: M68kCycleCount::from_raw(6_682),
            },
            "ASM loc_726 -> Load_DemoPlay -> Load_InGame $11A0E must retain the empty-list loc_119D8 allocation",
        );
        assert_eq!(
            &cycles[551..566],
            &[20, 10, 12, 18, 16, 16, 12, 8, 16, 32, 16, 12, 8, 18, 10]
        );
        assert_eq!(&cycles[cycles.len() - 3..], &[24, 16, 18]);
    }

    #[test]
    fn load_intro_sequence1_controller_allocation_retains_source_boundaries() {
        let mut cycles = Vec::new();
        let timing = visit_load_intro_sequence1_controller_allocation(|cycle| cycles.push(cycle));

        assert_eq!(
            timing,
            IntroSourceTiming {
                instruction_count: 81,
                raw_cycles: M68kCycleCount::from_raw(1_020),
            },
            "ASM Load_IntroSequence1 $1AE5C..$1AE6C must retain its empty-list sub_DAC allocation",
        );
        assert_eq!(&cycles[..10], &[8, 18, 10, 40, 4, 16, 28, 4, 4, 8]);
        assert_eq!(
            &cycles[cycles.len() - 8..],
            &[16, 10, 28, 16, 44, 16, 24, 16]
        );
    }

    #[test]
    fn intro_controller_return_reaches_wait_for_vint_poll_in_source_order() {
        let mut cycles = Vec::new();
        let timing = visit_intro_controller_to_wait_for_vint_poll(|cycle| cycles.push(cycle));

        assert_eq!(
            timing,
            IntroSourceTiming {
                instruction_count: 4,
                raw_cycles: M68kCycleCount::from_raw(70),
            },
            "ASM LoadGameModeData $72C through WaitForVint $93A must retain the frame-handoff boundaries",
        );
        assert_eq!(cycles, [20, 16, 18, 16]);
    }

    #[test]
    fn title_frame_return_reaches_next_wait_for_vint_poll_in_source_order() {
        let mut cycles = Vec::new();
        let timing = visit_title_frame_return_to_wait_for_vint_poll(|cycle| cycles.push(cycle));

        assert_eq!(
            timing,
            IntroSourceTiming {
                instruction_count: 4,
                raw_cycles: M68kCycleCount::from_raw(60),
            },
            "ASM Mode_Standard $7A4 through WaitForVint $93A must retain the next-frame handoff boundaries",
        );
        assert_eq!(cycles, [16, 10, 18, 16]);
    }

    #[test]
    fn mode_standard_do_nothing_retains_dbf_instruction_boundaries() {
        let mut cycles = Vec::new();
        let timing = visit_mode_standard_do_nothing(|cycle| cycles.push(cycle));

        assert_eq!(
            timing,
            IntroSourceTiming {
                instruction_count: 618,
                raw_cycles: M68kCycleCount::from_raw(6_192),
            },
            "ASM Mode_Standard $77A -> Do_Nothing $956 must retain every DBF boundary",
        );
        assert_eq!(&cycles[..3], &[18, 8, 8]);
        assert_eq!(&cycles[cycles.len() - 3..], &[14, 8, 16]);
    }

    #[test]
    fn mode_standard_sub_1596_retains_empty_queue_epilogue() {
        let mut cycles = Vec::new();
        let timing =
            visit_mode_standard_empty_foreground_queues_sub_1596(|cycle| cycles.push(cycle));

        assert_eq!(
            timing,
            IntroSourceTiming {
                instruction_count: 9,
                raw_cycles: M68kCycleCount::from_raw(142),
            },
            "ASM sub_1596 title path must skip both drains and reset both next-slot pointers",
        );
        assert_eq!(cycles, [18, 8, 12, 10, 12, 10, 28, 28, 16]);
    }

    #[test]
    fn mode_level_vblank_suffix_retains_natural_instruction_boundaries() {
        let work = crate::ModeLevelVblankWorkBeforeSubE12 {
            dynamic_art_tiles_loc_ad4: Some([16, 4, 0, 0, 0, 0]),
            palette_dma_sub_970: true,
            scroll_path: crate::ModeLevelScrollTransferPath::NormalLoc300ba,
            foreground_custom_queue_entries_sub_1596: 0,
            foreground_cell_queue_sub_1596: crate::ModeLevelForegroundCellQueueWorkSub1596::EMPTY,
            paused_sub_6874: false,
            hud_path_sub_bdc0: crate::ModeLevelHudBuildPath::LocBe20DirectInitialCounterUpdates,
        };
        let mut cycles = Vec::new();
        let timing =
            visit_mode_level_vblank_suffix_after_scroll_dma(work, |cycle| cycles.push(cycle))
                .expect("published Mode_Level VBlank suffix");

        assert_eq!(
            timing,
            IntroSourceTiming {
                instruction_count: 127,
                raw_cycles: M68kCycleCount::from_raw(1_350),
            },
            "ASM Mode_Level $7D6 -> Execute_Objects sub_E12 retains the initial-counter path",
        );
        assert_eq!(cycles.first(), Some(&18), "ASM Mode_Level $7D6 BSR.W");
        assert_eq!(cycles.last(), Some(&18), "ASM Mode_Level $7E2 BSR.W");

        let mut unsupported = work;
        unsupported.foreground_custom_queue_entries_sub_1596 = 1;
        assert_eq!(
            visit_mode_level_vblank_suffix_after_scroll_dma(unsupported, |_| {}),
            None,
            "unclassified gameplay foreground queue work remains fail-closed",
        );
    }

    #[test]
    fn transform_repeat_suffix_retains_loc_8fe_to_next_sub_8e0_boundaries() {
        let work = crate::ModeLevelVblankWorkBeforeSubE12 {
            dynamic_art_tiles_loc_ad4: Some([16, 4, 0, 0, 0, 0]),
            palette_dma_sub_970: true,
            scroll_path: crate::ModeLevelScrollTransferPath::NormalLoc300ba,
            foreground_custom_queue_entries_sub_1596: 0,
            foreground_cell_queue_sub_1596: crate::ModeLevelForegroundCellQueueWorkSub1596::EMPTY,
            paused_sub_6874: false,
            hud_path_sub_bdc0: crate::ModeLevelHudBuildPath::LocBe20ThenLocBe4cStableCounters,
        };
        let mut cycles = Vec::new();
        let timing = visit_transform_sub_8e0_repeat_suffix_after_scroll_dma(work, |cycle| {
            cycles.push(cycle);
        })
        .expect("published Cyclone transform repeat suffix");

        assert_eq!(
            timing,
            IntroSourceTiming {
                instruction_count: 54,
                raw_cycles: M68kCycleCount::from_raw(864),
            },
            "ASM loc_8FE -> loc_7D22 DBF repeat -> next sub_8E0",
        );
        assert_eq!(&cycles[..3], &[18, 8, 16]);
        assert_eq!(&cycles[cycles.len() - 4..], &[10, 120, 18, 10]);

        let mut queued = work;
        queued.foreground_custom_queue_entries_sub_1596 = 1;
        assert_eq!(
            visit_transform_sub_8e0_repeat_suffix_after_scroll_dma(queued, |_| {}),
            None,
            "unclassified transform queue work remains fail-closed",
        );
    }

    #[test]
    fn transform_first_helmet_art_suffix_retains_descriptor_transition_boundaries() {
        let work = crate::ModeLevelVblankWorkBeforeSubE12 {
            dynamic_art_tiles_loc_ad4: Some([12, 0, 0, 0, 0, 0]),
            palette_dma_sub_970: true,
            scroll_path: crate::ModeLevelScrollTransferPath::NormalLoc300ba,
            foreground_custom_queue_entries_sub_1596: 0,
            foreground_cell_queue_sub_1596: crate::ModeLevelForegroundCellQueueWorkSub1596::EMPTY,
            paused_sub_6874: false,
            hud_path_sub_bdc0: crate::ModeLevelHudBuildPath::LocBe20ThenLocBe4cStableCounters,
        };
        let resume = crate::TransformRasterResume::AdvanceDescriptor {
            descriptor_index: 1,
            handler: crate::TransformHandler::HelmetArt,
        };
        let mut cycles = Vec::new();
        let timing =
            visit_transform_sub_8e0_descriptor_suffix_after_scroll_dma(work, resume, |cycle| {
                cycles.push(cycle);
            })
            .expect("published Cyclone first helmet-art transition suffix");

        assert_eq!(
            timing,
            IntroSourceTiming {
                instruction_count: 109,
                raw_cycles: M68kCycleCount::from_raw(1_504),
            },
            "ASM loc_8FE -> loc_7D44 helmet-art descriptor -> next sub_8E0",
        );
        assert_eq!(&cycles[..3], &[18, 8, 16]);
        assert_eq!(&cycles[cycles.len() - 4..], &[16, 120, 18, 10]);

        for (resume, expected) in [
            (
                crate::TransformRasterResume::AdvanceDescriptor {
                    descriptor_index: 2,
                    handler: crate::TransformHandler::KidWithSpark { spark: false },
                },
                IntroSourceTiming {
                    instruction_count: 119,
                    raw_cycles: M68kCycleCount::from_raw(1_622),
                },
            ),
            (
                crate::TransformRasterResume::AdvanceDescriptor {
                    descriptor_index: 7,
                    handler: crate::TransformHandler::HelmetDefault,
                },
                IntroSourceTiming {
                    instruction_count: 107,
                    raw_cycles: M68kCycleCount::from_raw(1_474),
                },
            ),
        ] {
            assert_eq!(
                visit_transform_sub_8e0_descriptor_suffix_after_scroll_dma(work, resume, |_| {},),
                Some(expected),
                "ASM loc_7D44 dispatches by descriptor handler",
            );
        }
    }

    #[test]
    fn transform_finish_rejoins_mode_level_at_sub_44dc() {
        let work = crate::ModeLevelVblankWorkBeforeSubE12 {
            dynamic_art_tiles_loc_ad4: Some([12, 0, 0, 0, 0, 0]),
            palette_dma_sub_970: true,
            scroll_path: crate::ModeLevelScrollTransferPath::NormalLoc300ba,
            foreground_custom_queue_entries_sub_1596: 0,
            foreground_cell_queue_sub_1596: crate::ModeLevelForegroundCellQueueWorkSub1596::EMPTY,
            paused_sub_6874: false,
            hud_path_sub_bdc0: crate::ModeLevelHudBuildPath::LocBe20ThenLocBe4cStableCounters,
        };
        assert_eq!(
            transform_sub_8e0_finish_to_sub_44dc_timing(
                work,
                crate::TransformRasterResume::FinishLoc7e14,
                crate::M68kRefreshPhase::new(16).expect("valid refresh phase"),
            ),
            Some(crate::M68kRefreshAdvance {
                elapsed: M68kCycleCount::from_raw(8_772),
                phase_after: crate::M68kRefreshPhase::new(84).expect("valid refresh phase"),
                refresh_events: 73,
            }),
            "ASM loc_8FE -> loc_7E14 cleanup -> resumed Mode_Level sub_44DC",
        );
    }

    #[test]
    fn transform_timing_paths_reject_each_unclassified_vblank_workload() {
        // ASM loc_8FE enters sub_1596, skips pause sub_6874, and reaches the
        // stable three-hitpoint sub_BDC0 path in the classified Cyclone trace.
        // Each independent deviation must fail closed for the repeat,
        // descriptor-transition, and loc_7E14 finish compositions.
        let work = crate::ModeLevelVblankWorkBeforeSubE12 {
            dynamic_art_tiles_loc_ad4: Some([12, 0, 0, 0, 0, 0]),
            palette_dma_sub_970: true,
            scroll_path: crate::ModeLevelScrollTransferPath::NormalLoc300ba,
            foreground_custom_queue_entries_sub_1596: 0,
            foreground_cell_queue_sub_1596: crate::ModeLevelForegroundCellQueueWorkSub1596::EMPTY,
            paused_sub_6874: false,
            hud_path_sub_bdc0: crate::ModeLevelHudBuildPath::LocBe20ThenLocBe4cStableCounters,
        };
        let mut custom_queue = work;
        custom_queue.foreground_custom_queue_entries_sub_1596 = 1;
        let mut cell_queue = work;
        cell_queue.foreground_cell_queue_sub_1596 =
            crate::ModeLevelForegroundCellQueueWorkSub1596::from_paths(&[
                crate::ModeLevelForegroundCellQueuePathSub1596::VisibleTheme,
            ])
            .expect("one foreground cell fits the ROM queue");
        let mut paused = work;
        paused.paused_sub_6874 = true;
        let mut other_hud = work;
        other_hud.hud_path_sub_bdc0 = crate::ModeLevelHudBuildPath::Other;

        let descriptor_resume = crate::TransformRasterResume::AdvanceDescriptor {
            descriptor_index: 1,
            handler: crate::TransformHandler::HelmetArt,
        };
        for (name, unsupported) in [
            ("custom queue", custom_queue),
            ("cell queue", cell_queue),
            ("pause", paused),
            ("HUD path", other_hud),
        ] {
            assert_eq!(
                visit_transform_sub_8e0_repeat_suffix_after_scroll_dma(unsupported, |_| {}),
                None,
                "repeat suffix accepts unsupported {name}",
            );
            assert_eq!(
                visit_transform_sub_8e0_descriptor_suffix_after_scroll_dma(
                    unsupported,
                    descriptor_resume,
                    |_| {},
                ),
                None,
                "descriptor suffix accepts unsupported {name}",
            );
            assert_eq!(
                transform_sub_8e0_finish_to_sub_44dc_timing(
                    unsupported,
                    crate::TransformRasterResume::FinishLoc7e14,
                    crate::M68kRefreshPhase::new(16).expect("valid refresh phase"),
                ),
                None,
                "finish suffix accepts unsupported {name}",
            );
        }
    }

    #[test]
    fn cyclone_pickup_entry_retains_sub_e12_to_sub_8e0_boundaries() {
        let mut cycles = Vec::new();
        let timing = visit_cyclone_pickup_transform_entry_sub_e12(
            crate::CyclonePickupTransformEntrySubE12,
            |cycle| cycles.push(cycle),
        );

        assert_eq!(
            timing,
            IntroSourceTiming {
                instruction_count: 519,
                raw_cycles: M68kCycleCount::from_raw(6_280),
            },
            "ASM sub_E12 -> loc_7D22 -> sub_8E0 published frame 1058",
        );
        assert_eq!(cycles.first(), Some(&18), "ASM sub_E12 JSR RNG");
        assert_eq!(cycles.last(), Some(&10), "ASM loc_7D22 JSR.W sub_8E0");
    }

    #[test]
    fn mode_level_vblank_suffix_visits_frame_990_visible_theme_cell_queue() {
        // ASM sub_1596 loc_15D2 -> loc_1614, published TAS frame 990.
        let cell_queue = crate::ModeLevelForegroundCellQueueWorkSub1596::from_paths(&[
            crate::ModeLevelForegroundCellQueuePathSub1596::VisibleTheme,
        ])
        .expect("one foreground cell fits the ROM queue");
        let work = crate::ModeLevelVblankWorkBeforeSubE12 {
            dynamic_art_tiles_loc_ad4: Some([16, 4, 0, 0, 0, 0]),
            palette_dma_sub_970: true,
            scroll_path: crate::ModeLevelScrollTransferPath::NormalLoc300ba,
            foreground_custom_queue_entries_sub_1596: 0,
            foreground_cell_queue_sub_1596: cell_queue,
            paused_sub_6874: false,
            hud_path_sub_bdc0: crate::ModeLevelHudBuildPath::LocBe20DirectStableCounters,
        };
        let timing = visit_mode_level_vblank_suffix_after_scroll_dma(work, |_| {})
            .expect("single visible theme-cell suffix");

        assert_eq!(
            timing,
            IntroSourceTiming {
                instruction_count: 123,
                raw_cycles: M68kCycleCount::from_raw(1_278),
            },
            "bounded all-PC oracle journal from loc_7D6 to sub_E12",
        );

        let second_hitpoint_timing = visit_mode_level_vblank_suffix_after_scroll_dma(
            crate::ModeLevelVblankWorkBeforeSubE12 {
                hud_path_sub_bdc0: crate::ModeLevelHudBuildPath::LocBe20ThenLocBe4cStableCounters,
                ..work
            },
            |_| {},
        )
        .expect("single visible theme-cell Cyclone HUD suffix");
        assert_eq!(
            second_hitpoint_timing,
            IntroSourceTiming {
                instruction_count: 137,
                raw_cycles: M68kCycleCount::from_raw(1_388),
            },
            "ASM sub_1596 and sub_BDC0 full-PC journal from published TAS frame 1367",
        );

        let unsupported = crate::ModeLevelForegroundCellQueueWorkSub1596::from_paths(&[
            crate::ModeLevelForegroundCellQueuePathSub1596::VisibleBlock,
        ])
        .expect("one foreground cell fits the ROM queue");
        assert_eq!(
            visit_mode_level_vblank_suffix_after_scroll_dma(
                crate::ModeLevelVblankWorkBeforeSubE12 {
                    foreground_cell_queue_sub_1596: unsupported,
                    ..work
                },
                |_| {},
            ),
            None,
            "unvisited cell branches remain fail-closed",
        );
    }

    #[test]
    fn title_execute_objects_retains_rng_and_single_controller_dispatch() {
        let mut cycles = Vec::new();
        let timing = visit_title_execute_objects_to_intro_controller_sub_1b8f4(0xFFCD, |cycle| {
            cycles.push(cycle);
        })
        .expect("frame-81 title RNG offset is initialised");

        assert_eq!(timing.instruction_count, 112);
        assert_eq!(timing.raw_cycles, M68kCycleCount::from_raw(1_276));
        assert_eq!(
            crate::mode_level_timing::execute_objects_setup_cycles_sub_e12(0xFFCD),
            Some(M68kCycleCount::from_raw(1_088)),
            "ASM sub_E12 setup aggregate independently checks the instruction visitor",
        );
        assert_eq!(
            cycles[cycles.len() - 12..],
            [12, 8, 4, 8, 10, 12, 8, 12, 44, 12, 24, 16],
            "ASM loc_E1E dispatches the sole awake controller into sub_1B8F4",
        );
    }

    #[test]
    fn fresh_title_first_execute_objects_initialises_three_objects_in_rom_order() {
        let mut cycles = Vec::new();
        let timing = visit_fresh_title_first_execute_objects(0xFFDD, |cycle| cycles.push(cycle))
            .expect("frame-196 title RNG offset is initialised");

        assert_eq!(timing.instruction_count, 413);
        assert_eq!(timing.raw_cycles, M68kCycleCount::from_raw(5_468));
        assert_eq!(
            cycles.iter().filter(|&&cycle| cycle == 76).count(),
            3,
            "ASM sub_E76 hibernates Chameleon, Kid, and Menu once each",
        );
    }

    #[test]
    fn fresh_title_second_execute_objects_reaches_chameleon_enigma_in_rom_order() {
        let mut cycles = Vec::new();
        let timing =
            visit_fresh_title_second_execute_objects_to_chameleon_enigma(0xFFE1, |cycle| {
                cycles.push(cycle);
            })
            .expect("frame-197 title RNG offset is initialised");

        assert_eq!(
            crate::mode_level_timing::execute_objects_setup_cycles_sub_e12(0xFFE1),
            Some(M68kCycleCount::from_raw(1_112)),
            "ASM sub_E12 setup independently checks the frame-197 RNG prefix",
        );
        assert_eq!(cycles.iter().filter(|&&cycle| cycle == 76).count(), 2);
        assert_eq!(cycles.last(), Some(&18), "ASM loc_1B6A6 calls sub_1B71A");
        assert_eq!(timing.instruction_count, cycles.len() as u32);
    }

    #[test]
    fn fresh_title_chameleon_upload_writes_seven_fillers_and_195_map_words() {
        let mut events = Vec::new();
        let timing = visit_fresh_title_chameleon_upload_to_execute_return(|event| {
            events.push(event);
        });

        assert_eq!(
            events
                .iter()
                .filter(|event| matches!(event, MemoryDmaTimingEvent::WriteVdpWord { .. }))
                .count(),
            202,
            "ASM sub_1B71A writes 7 fillers + 21 first-row + 6*29 later-row words",
        );
        assert_eq!(
            events
                .iter()
                .filter(|event| matches!(event, MemoryDmaTimingEvent::Instruction(76)))
                .count(),
            2,
            "the ordinary-pass model hibernates Chameleon and Menu after the upload",
        );
        assert_eq!(
            timing.instruction_count,
            events
                .iter()
                .filter(|event| matches!(event, MemoryDmaTimingEvent::Instruction(_)))
                .count() as u32,
        );
    }

    #[test]
    fn fresh_title_timeout_hibernates_mature_kid_before_menu_falls_through() {
        let mut events = Vec::new();
        let timing = visit_fresh_title_chameleon_upload_to_demo_mode(|event| events.push(event));

        assert_eq!(
            events
                .iter()
                .filter(|event| matches!(event, MemoryDmaTimingEvent::Instruction(76)))
                .count(),
            2,
            "ASM loc_1B6A2 and loc_1B64C hibernate Chameleon and Kid, but expired Obj_TitleMenu does not yield",
        );
        assert_eq!(
            events.last(),
            Some(&MemoryDmaTimingEvent::Instruction(14)),
            "ASM loc_1BAFA DBF expires into loc_1BB02",
        );
        assert_eq!(
            timing.instruction_count,
            events
                .iter()
                .filter(|event| matches!(event, MemoryDmaTimingEvent::Instruction(_)))
                .count() as u32,
        );
    }

    #[test]
    fn fresh_title_third_execute_objects_writes_all_sega_glyphs_then_reaches_chameleon() {
        let mut events = Vec::new();
        let timing = visit_fresh_title_third_execute_objects_to_chameleon_enigma(0xFFE5, |event| {
            events.push(event);
        })
        .expect("frame-198 title RNG offset is initialised");

        assert_eq!(
            events
                .iter()
                .filter(|event| matches!(event, MemoryDmaTimingEvent::WriteVdpWord { .. }))
                .count(),
            11,
            "ASM loc_1C7B6/sub_1C7F6 drains the complete copyright string",
        );
        assert_eq!(
            events
                .iter()
                .filter(|event| matches!(event, MemoryDmaTimingEvent::Instruction(76)))
                .count(),
            1,
            "ASM sub_E76 hibernates only the prompt before sub_1B71A",
        );
        assert_eq!(
            events.last(),
            Some(&MemoryDmaTimingEvent::Instruction(18)),
            "ASM loc_1B6A6 calls sub_1B71A",
        );
        assert_eq!(
            timing.instruction_count,
            events
                .iter()
                .filter(|event| matches!(event, MemoryDmaTimingEvent::Instruction(_)))
                .count() as u32,
        );
    }

    #[test]
    fn fresh_title_fourth_execute_objects_starts_with_prompt_then_reaches_chameleon() {
        let mut events = Vec::new();
        let timing =
            visit_fresh_title_prompt_chameleon_execute_objects_to_enigma(0xFFE9, |event| {
                events.push(event);
            })
            .expect("frame-199 title RNG offset is initialised");

        assert_eq!(
            events
                .iter()
                .filter(|event| matches!(event, MemoryDmaTimingEvent::WriteVdpWord { .. }))
                .count(),
            0,
            "ASM loc_1BB50 and loc_1B6A6 perform no VDP writes before sub_1B71A",
        );
        assert_eq!(events.last(), Some(&MemoryDmaTimingEvent::Instruction(18)));
        assert_eq!(
            timing.instruction_count,
            events
                .iter()
                .filter(|event| matches!(event, MemoryDmaTimingEvent::Instruction(_)))
                .count() as u32,
        );
    }

    #[test]
    fn second_title_execute_objects_hibernates_text_worker_then_controller() {
        let mut cycles = Vec::new();
        let timing = visit_title_second_execute_objects(0xFFD1, |cycle| cycles.push(cycle))
            .expect("frame-82 title RNG offset is initialised");

        assert_eq!(timing.instruction_count, 159);
        assert_eq!(timing.raw_cycles, M68kCycleCount::from_raw(2_044));
        assert_eq!(
            crate::mode_level_timing::execute_objects_setup_cycles_sub_e12(0xFFD1),
            Some(M68kCycleCount::from_raw(1_088)),
            "ASM sub_E12 setup aggregate independently checks the second-frame RNG prefix",
        );
        assert_eq!(
            cycles.iter().filter(|&&cycle| cycle == 76).count(),
            2,
            "ASM loc_1C7A0 and loc_1C21E each pack eight coroutine registers once",
        );
        assert_eq!(&cycles[cycles.len() - 4..], &[12, 10, 16, 16]);
    }

    #[test]
    fn third_title_execute_objects_consumes_initial_text_delay_command() {
        let mut cycles = Vec::new();
        let timing = visit_title_third_execute_objects(0xFFD5, |cycle| cycles.push(cycle))
            .expect("frame-83 title RNG offset is initialised");

        assert_eq!(timing.instruction_count, 166);
        assert_eq!(timing.raw_cycles, M68kCycleCount::from_raw(2_098));
        assert_eq!(
            crate::mode_level_timing::execute_objects_setup_cycles_sub_e12(0xFFD5),
            Some(M68kCycleCount::from_raw(1_088)),
            "ASM sub_E12 setup aggregate independently checks the third-frame RNG prefix",
        );
        assert_eq!(
            cycles.iter().filter(|&&cycle| cycle == 76).count(),
            2,
            "ASM loc_1C7AE and loc_1C21E each pack eight coroutine registers once",
        );
        assert_eq!(&cycles[cycles.len() - 4..], &[12, 10, 16, 16]);
    }

    #[test]
    fn ordinary_title_text_delay_frame_hibernates_both_objects() {
        let mut cycles = Vec::new();
        let timing = visit_title_text_delay_fade_execute_objects(0xFFD9, 0x20, 0x20, |cycle| {
            cycles.push(cycle);
        })
        .expect("frame-84 ordinary text-delay state");

        assert_eq!(timing.instruction_count, 156);
        assert_eq!(timing.raw_cycles, M68kCycleCount::from_raw(2_018));
        assert_eq!(
            cycles.iter().filter(|&&cycle| cycle == 76).count(),
            2,
            "ASM loc_1C7AE and loc_1C21E each pack eight coroutine registers once",
        );
        assert!(
            visit_title_text_delay_fade_execute_objects(0xFFD9, 1, 0x20, |_| {}).is_none(),
            "terminal text delay takes the command/glyph parser instead",
        );
        assert!(
            visit_title_text_delay_fade_execute_objects(0xFFD9, 0x20, 0x100, |_| {}).is_none(),
            "fade `$100` takes the controller exit after its next increment",
        );

        let before_wrap = visit_title_text_delay_fade_execute_objects(0xFFFD, 0x17, 0xB0, |_| {})
            .expect("frame-93 final RNG-ring word");
        let after_wrap = visit_title_text_delay_fade_execute_objects(0xFFCA, 0x16, 0xC0, |_| {})
            .expect("frame-94 wrapped RNG-ring word");
        assert_eq!(
            before_wrap,
            IntroSourceTiming {
                instruction_count: 157,
                raw_cycles: M68kCycleCount::from_raw(2_024),
            },
            "ASM sub_E12 takes the extra pre-wrap RNG offset instruction",
        );
        assert_eq!(
            after_wrap,
            IntroSourceTiming {
                instruction_count: 156,
                raw_cycles: M68kCycleCount::from_raw(2_018),
            },
            "ASM sub_E12 resumes the short setup after `$FFFD->$FFCA`",
        );
    }

    #[test]
    fn title_controller_exits_fade_into_post_fade_delay() {
        let mut cycles = Vec::new();
        let timing = visit_title_controller_fade_exit_execute_objects(0xFFDA, 0x12, |cycle| {
            cycles.push(cycle);
        })
        .expect("frame-98 controller fade-exit state");

        assert_eq!(
            timing,
            IntroSourceTiming {
                instruction_count: 156,
                raw_cycles: M68kCycleCount::from_raw(2_022),
            },
        );
        assert_eq!(
            cycles.iter().filter(|&&cycle| cycle == 76).count(),
            2,
            "ASM loc_1C7AE and loc_1B924 each hibernate one object",
        );
        assert!(
            visit_title_controller_fade_exit_execute_objects(0xFFDA, 1, |_| {}).is_none(),
            "terminal text delay takes the command/glyph parser instead",
        );
    }

    #[test]
    fn title_post_fade_wait_hibernates_text_and_controller() {
        let before_wrap = visit_title_post_fade_wait_execute_objects(0xFFFE, 9, 0x123, |_| {})
            .expect("frame-107 final RNG-ring word");
        let after_wrap = visit_title_post_fade_wait_execute_objects(0xFFCB, 8, 0x122, |_| {})
            .expect("frame-108 wrapped RNG-ring word");

        assert_eq!(
            before_wrap,
            IntroSourceTiming {
                instruction_count: 152,
                raw_cycles: M68kCycleCount::from_raw(1_964),
            },
        );
        assert_eq!(
            after_wrap,
            IntroSourceTiming {
                instruction_count: 151,
                raw_cycles: M68kCycleCount::from_raw(1_958),
            },
        );
        assert!(
            visit_title_post_fade_wait_execute_objects(0xFFCB, 8, 0, |_| {}).is_none(),
            "terminal controller DBF takes the scene-transition path",
        );
    }

    #[test]
    fn title_first_glyph_writes_t_and_loads_two_frame_delay() {
        let mut cycles = Vec::new();
        let timing = visit_title_first_glyph_execute_objects(0xFFE7, 0x11B, |cycle| {
            cycles.push(cycle);
        })
        .expect("frame-115 first glyph state");

        assert_eq!(
            timing,
            IntroSourceTiming {
                instruction_count: 184,
                raw_cycles: M68kCycleCount::from_raw(2_236),
            },
        );
        assert_eq!(
            cycles.iter().filter(|&&cycle| cycle == 76).count(),
            2,
            "ASM loc_1C7AE and loc_1B924 hibernate both live objects",
        );
        assert_eq!(
            visit_title_ordinary_glyph_execute_objects(0xFFEF, 0x119, b'H', |_| {}),
            Some(timing),
            "ASM loc_1C7B2 -> sub_1C7F6 has the same timing for ordinary glyphs",
        );
        assert!(
            visit_title_ordinary_glyph_execute_objects(0xFFEF, 0x119, 0xFD, |_| {}).is_none(),
            "line breaks take the control-byte parser path",
        );
        assert_eq!(
            visit_title_ordinary_glyph_execute_objects(0xFFD8, 0x111, b' ', |_| {}),
            Some(IntroSourceTiming {
                instruction_count: 183,
                raw_cycles: M68kCycleCount::from_raw(2_230),
            }),
            "ORIGINAL_QUIRK: frame 125's space stays on the ordinary parser path",
        );
    }

    #[test]
    fn title_intro_sequence1_start_reaches_stop_music_after_n_glyph() {
        let mut cycles = Vec::new();
        let timing = visit_title_intro_sequence1_glyph_start_to_stop_music(
            0xFFD9,
            0x103,
            b'N',
            2,
            |cycle| cycles.push(cycle),
        )
        .expect("frame-139 IntroSequence1 Start transition");

        assert_eq!(
            timing,
            IntroSourceTiming {
                instruction_count: 183,
                raw_cycles: M68kCycleCount::from_raw(2_166),
            },
            "ASM loc_1C7B2 -> sub_1C246 -> sub_1C278 -> StopMusic $E14C8",
        );
        assert_eq!(
            &cycles[cycles.len() - 17..],
            &[
                18, 20, 10, 16, 8, 16, 8, 18, 16, 16, 8, 16, 16, 16, 16, 20, 10
            ],
            "frame-139 selected PCs pin every Start-path instruction",
        );
        assert!(
            visit_title_intro_sequence1_glyph_start_to_stop_music(0xFFD9, 0x103, b'N', 1, |_| {},)
                .is_none(),
            "a non-terminal intro counter takes sub_1C278's shorter branch",
        );
    }

    #[test]
    fn title_stop_music_return_initialises_fade_before_waiting() {
        let mut cycles = Vec::new();
        let timing = visit_title_stop_music_return_to_first_fade_wait(|cycle| cycles.push(cycle));

        assert_eq!(
            timing,
            IntroSourceTiming {
                instruction_count: 214,
                raw_cycles: M68kCycleCount::from_raw(2_380),
            },
            "ASM loc_1C274 -> loc_6E2 -> Pal_FadeOut -> loc_28C6",
        );
        assert_eq!(cycles.iter().filter(|&&cycle| cycle == 20).count(), 32);
        assert_eq!(cycles.iter().filter(|&&cycle| cycle == 8).count(), 66);
        assert_eq!(
            cycles.iter().filter(|&&cycle| cycle == 14).count(),
            2,
            "both DBF loops retain their terminal instruction timing",
        );
    }

    #[test]
    fn title_intro_controller_allocates_prompt_gfx_before_palette_copy() {
        let mut cycles = Vec::new();
        let timing =
            visit_title_intro_controller_prompt_gfx_to_sub_1c4ea(|cycle| cycles.push(cycle));

        assert_eq!(timing.instruction_count, 74);
        assert_eq!(timing.raw_cycles, M68kCycleCount::from_raw(964));
        assert_eq!(
            cycles.iter().filter(|&&cycle| cycle == 12).count(),
            23,
            "ASM sub_F6E retains all 19 clear writes plus its surrounding 12-cycle operations",
        );
        assert_eq!(
            &cycles[cycles.len() - 9..],
            &[6, 16, 16, 16, 16, 16, 16, 16, 18],
            "ASM sub_1C54A initialises the prompt then calls sub_1C4EA",
        );
    }

    #[test]
    fn title_intro_palette_copy_retains_all_51_words_before_text_allocation() {
        let mut cycles = Vec::new();
        let timing = visit_title_intro_palette_copy_to_text_object_allocation(|cycle| {
            cycles.push(cycle);
        });

        assert_eq!(timing.instruction_count, 109);
        assert_eq!(timing.raw_cycles, M68kCycleCount::from_raw(1_202));
        assert_eq!(
            cycles.iter().filter(|&&cycle| cycle == 12).count(),
            52,
            "ASM sub_1C4EA retains its setup plus all 51 palette-word copies",
        );
        assert_eq!(&cycles[cycles.len() - 4..], &[16, 8, 18, 10]);
    }

    #[test]
    fn title_text_object_allocation_walks_the_existing_controller_once() {
        let mut cycles = Vec::new();
        let timing = visit_title_text_object_allocation_to_fade(|cycle| cycles.push(cycle));

        assert_eq!(timing.instruction_count, 85);
        assert_eq!(timing.raw_cycles, M68kCycleCount::from_raw(1_064));
        assert_eq!(
            &cycles[70..81],
            &[16, 8, 4, 4, 12, 8, 4, 28, 16, 44, 16],
            "ASM loc_DE0 performs one equal-priority insertion-list visit",
        );
        assert_eq!(&cycles[cycles.len() - 4..], &[24, 24, 24, 18]);
    }

    #[test]
    fn title_first_fade_hibernates_controller_then_exits_execute_objects() {
        let mut cycles = Vec::new();
        let timing = visit_title_first_fade_hibernate_to_mode_standard(|cycle| cycles.push(cycle));

        assert_eq!(timing.instruction_count, 149);
        assert_eq!(timing.raw_cycles, M68kCycleCount::from_raw(1_496));
        assert_eq!(
            cycles.iter().filter(|&&cycle| cycle == 8).count(),
            64,
            "ASM sub_1C204 retains every Palette_Buffer_3 clear write",
        );
        assert_eq!(
            &cycles[cycles.len() - 10..],
            &[16, 24, 16, 76, 16, 10, 12, 10, 16, 16],
            "ASM sub_E76 saves the controller then loc_E1E exits through loc_E58",
        );
    }

    #[test]
    fn title_first_gfx_object_takes_stationary_unanimated_paths() {
        let mut cycles = Vec::new();
        let timing = visit_title_first_gfx_object_move_and_animate(|cycle| cycles.push(cycle));

        assert_eq!(timing.instruction_count, 25);
        assert_eq!(timing.raw_cycles, M68kCycleCount::from_raw(306));
        assert_eq!(
            &cycles[8..13],
            &[12, 8, 12, 10, 16],
            "ASM GfxObject_Move sub_1120 returns with is_moved clear",
        );
        assert_eq!(
            &cycles[13..21],
            &[18, 12, 12, 12, 12, 10, 12, 16],
            "ASM GfxObject_Animate sub_113E returns with is_animated clear",
        );
    }

    #[test]
    fn title_first_read_joypad_retains_both_bus_request_edges() {
        let mut events = Vec::new();
        let timing = visit_title_first_read_joypad(|event| events.push(event));

        assert_eq!(timing.instruction_count, 93);
        assert_eq!(timing.raw_cycles, M68kCycleCount::from_raw(1_130));
        assert_eq!(
            events
                .iter()
                .filter(|event| matches!(event, MemoryDmaTimingEvent::SetZ80BusRequest(_)))
                .count(),
            2,
            "ASM sub_914/sub_924 retain Stop_z80 and Start_z80 writes",
        );
    }

    #[test]
    fn title_first_prompt_emits_four_sprite_entries() {
        let mut cycles = Vec::new();
        let timing = visit_title_first_make_sprites_from_gfx_objects(|cycle| cycles.push(cycle));

        assert_eq!(timing.instruction_count, 187);
        assert_eq!(timing.raw_cycles, M68kCycleCount::from_raw(1_892));
        assert_eq!(
            cycles
                .windows(16)
                .filter(|window| {
                    *window == [12, 12, 12, 16, 20, 12, 8, 20, 4, 12, 4, 14, 4, 16, 8, 16]
                })
                .count(),
            4,
            "ASM sub_13C0 emits four entries for the 62x47 prompt mapping",
        );
    }

    #[test]
    fn title_first_palette_interpolation_processes_both_full_dirty_masks() {
        let mut cycles = Vec::new();
        let timing = visit_title_first_palette_interpolation(|cycle| cycles.push(cycle));

        assert_eq!(timing.instruction_count, 2_413);
        assert_eq!(timing.raw_cycles, M68kCycleCount::from_raw(19_848));
        assert_eq!(
            cycles.iter().filter(|&&cycle| cycle == 22).count(),
            64,
            "ASM sub_1536 writes all 64 colours across both dirty masks",
        );
    }

    #[test]
    fn palette_fade_first_decrement_reaches_next_wait_poll_in_source_order() {
        let mut cycles = Vec::new();
        let timing =
            visit_palette_fade_first_decrement_to_next_wait_poll(|cycle| cycles.push(cycle));

        assert_eq!(
            timing,
            IntroSourceTiming {
                instruction_count: 8,
                raw_cycles: M68kCycleCount::from_raw(108),
            },
            "ASM Pal_FadeOut $28D6..$28C6 and WaitForVint $93A retain the first non-negative fade loop",
        );
        assert_eq!(cycles, [20, 8, 4, 16, 16, 10, 18, 16]);
    }

    #[test]
    fn palette_fade_negative_and_clean_tail_reach_loc_700_in_source_order() {
        let mut negative_cycles = Vec::new();
        let negative = visit_palette_fade_negative_decrement_to_final_wait_poll(|cycle| {
            negative_cycles.push(cycle);
        });
        assert_eq!(negative.instruction_count, 4);
        assert_eq!(negative.raw_cycles, M68kCycleCount::from_raw(64));
        assert_eq!(negative_cycles, [20, 10, 18, 16]);

        let mut clean_cycles = Vec::new();
        let clean = visit_clean_palette_interpolation_call(|cycle| clean_cycles.push(cycle));
        assert_eq!(clean.instruction_count, 6);
        assert_eq!(clean.raw_cycles, M68kCycleCount::from_raw(82));
        assert_eq!(clean_cycles, [18, 16, 16, 8, 8, 16]);

        let mut return_cycles = Vec::new();
        let fade_return = visit_palette_fade_return_to_loc_700(|cycle| return_cycles.push(cycle));
        assert_eq!(fade_return.instruction_count, 2);
        assert_eq!(fade_return.raw_cycles, M68kCycleCount::from_raw(32));
        assert_eq!(return_cycles, [16, 16]);
    }

    #[test]
    fn loc_700_and_loc_714_clear_full_vram_and_sprite_table_ranges() {
        let mut vram_events = Vec::new();
        let vram = visit_loc_700_vram_clear(|event| vram_events.push(event));
        assert_eq!(vram.instruction_count, 32_771);
        assert_eq!(vram.raw_cycles, M68kCycleCount::from_raw(360_488));
        assert_eq!(
            vram_events
                .iter()
                .filter(|event| matches!(
                    event,
                    MemoryDmaTimingEvent::WriteVdpWord {
                        target: MemoryToVdpDmaTarget::Vram
                    }
                ))
                .count(),
            32_768,
            "ASM loc_70E writes 16,384 longwords as 32,768 FIFO words",
        );

        let mut sprite_events = Vec::new();
        let sprite = visit_loc_714_sprite_table_clear(|event| sprite_events.push(event));
        assert_eq!(sprite.instruction_count, 64_453);
        assert_eq!(sprite.raw_cycles, M68kCycleCount::from_raw(580_078));
        assert!(
            sprite_events
                .iter()
                .all(|event| matches!(event, MemoryDmaTimingEvent::Instruction(_))),
            "ASM loc_720 clears 68000 RAM and never touches the VDP FIFO",
        );
    }

    #[test]
    fn title_restart_initialisers_reach_ice_play_sound_in_source_order() {
        let mut cycles = Vec::new();
        let timing = visit_title_restart_initialisers_to_play_sound(|cycle| cycles.push(cycle));

        assert_eq!(timing.instruction_count, 562);
        assert_eq!(timing.raw_cycles, M68kCycleCount::from_raw(5_612));
        assert_eq!(cycles.len(), 562);
        assert_eq!(&cycles[..4], &[18, 12, 16, 4]);
        assert_eq!(
            &cycles[cycles.len() - 11..],
            &[20, 10, 12, 18, 16, 8, 20, 10, 12, 12, 18],
        );
    }

    #[test]
    fn title_load_screen_fill_writes_2048_vram_words_before_sub_1b2a4() {
        let mut events = Vec::new();
        let timing = visit_title_load_screen_vram_fill_to_sub_1b2a4(|event| events.push(event));

        assert_eq!(timing.instruction_count, 4_102);
        assert_eq!(timing.raw_cycles, M68kCycleCount::from_raw(45_150));
        assert_eq!(
            events
                .iter()
                .filter(|event| matches!(
                    event,
                    MemoryDmaTimingEvent::WriteVdpWord {
                        target: MemoryToVdpDmaTarget::Vram
                    }
                ))
                .count(),
            2_048,
            "ASM loc_1B258 writes the inclusive d0=$07FF VRAM range",
        );
    }

    #[test]
    fn title_character_map_tail_writes_sub_1c5d0_and_sub_1b532_words() {
        let mut events = Vec::new();
        let timing = visit_title_character_map_to_chameleon_enigma(|event| events.push(event));

        assert_eq!(timing.instruction_count, 2_352);
        assert_eq!(timing.raw_cycles, M68kCycleCount::from_raw(24_262));
        assert_eq!(
            events
                .iter()
                .filter(|event| matches!(
                    event,
                    MemoryDmaTimingEvent::WriteVdpWord {
                        target: MemoryToVdpDmaTarget::Vram
                    }
                ))
                .count(),
            376,
            "ASM sub_1C5D0 writes 18x20 words and sub_1B532 clears 16 words",
        );
    }

    #[test]
    fn palette_to_vram_retains_bus_and_dma_boundaries() {
        let mut events = Vec::new();
        let timing = visit_mode_standard_palette_to_vram(|event| events.push(event));

        assert_eq!(
            timing,
            IntroSourceTiming {
                instruction_count: 22,
                raw_cycles: M68kCycleCount::from_raw(362),
            },
            "ASM Palette_to_VRAM $970 must retain its Stop_z80, CRAM DMA and Start_z80 boundaries",
        );
        assert_eq!(
            events
                .iter()
                .filter(|event| !matches!(event, MemoryDmaTimingEvent::Instruction(_)))
                .copied()
                .collect::<Vec<_>>(),
            [
                MemoryDmaTimingEvent::SetZ80BusRequest(true),
                MemoryDmaTimingEvent::BeginMemoryDma {
                    words: 0x40,
                    target: MemoryToVdpDmaTarget::Cram,
                },
                MemoryDmaTimingEvent::SetZ80BusRequest(false),
            ],
        );
    }

    #[test]
    fn mode_level_first_palette_to_vram_skips_dma_while_f890_is_set() {
        let mut events = Vec::new();
        let timing = visit_mode_level_palette_to_vram(false, |event| events.push(event));

        assert_eq!(
            timing,
            IntroSourceTiming {
                instruction_count: 4,
                raw_cycles: M68kCycleCount::from_raw(56),
            },
            "ASM Mode_Level $7CC -> Palette_to_VRAM $970 skips to its RTS while PaletteToDMA_Flag is set",
        );
        assert_eq!(
            events,
            [18, 12, 10, 16].map(MemoryDmaTimingEvent::Instruction),
            "the skip path must not emit Z80 BUSREQ or CRAM DMA events",
        );
    }

    #[test]
    fn sprite_and_kid_transfer_retains_dma_and_zero_list_path() {
        let mut events = Vec::new();
        let timing = visit_mode_standard_sprite_and_kid_to_vram(|event| events.push(event));

        assert_eq!(timing.instruction_count, 55);
        assert_eq!(timing.raw_cycles, M68kCycleCount::from_raw(796));
        assert_eq!(
            events
                .iter()
                .filter(|event| !matches!(event, MemoryDmaTimingEvent::Instruction(_)))
                .copied()
                .collect::<Vec<_>>(),
            [
                MemoryDmaTimingEvent::SetZ80BusRequest(true),
                MemoryDmaTimingEvent::BeginMemoryDma {
                    words: 0x140,
                    target: MemoryToVdpDmaTarget::Vram,
                },
                MemoryDmaTimingEvent::SetZ80BusRequest(false),
                MemoryDmaTimingEvent::SetZ80BusRequest(true),
                MemoryDmaTimingEvent::SetZ80BusRequest(false),
            ],
            "ASM sub_A5C title path brackets both Z80 stops around one sprite DMA",
        );
    }

    #[test]
    fn mode_level_sprite_and_kid_transfer_visits_classified_loc_ad4_dmas() {
        let mut events = Vec::new();
        let timing = visit_mode_level_sprite_and_kid_to_vram([16, 4, 0, 0, 0, 0], |event| {
            events.push(event);
        })
        .expect("published Mode_Level loc_AD4 workload");

        assert_eq!(timing.instruction_count, 133);
        assert_eq!(timing.raw_cycles, M68kCycleCount::from_raw(1_398));
        assert_eq!(
            events
                .iter()
                .filter_map(|event| match event {
                    MemoryDmaTimingEvent::BeginMemoryDma { words, target } => {
                        Some((*words, *target))
                    }
                    _ => None,
                })
                .collect::<Vec<_>>(),
            [
                (0x140, MemoryToVdpDmaTarget::Vram),
                (0x100, MemoryToVdpDmaTarget::Vram),
                (0x040, MemoryToVdpDmaTarget::Vram),
            ],
            "ASM sub_A5C/loc_AD4 preserves the Sprite_Table, 16-tile, then 4-tile DMA order",
        );

        let mut first_live_events = Vec::new();
        let first_live_timing =
            visit_mode_level_sprite_and_kid_to_vram([12, 0, 0, 0, 0, 0], |event| {
                first_live_events.push(event);
            })
            .expect("first live Kid mapping produces one 12-tile loc_AD4 DMA");
        assert_eq!(first_live_timing.instruction_count, 95);
        assert_eq!(
            first_live_events
                .iter()
                .filter_map(|event| match event {
                    MemoryDmaTimingEvent::BeginMemoryDma { words, target } => {
                        Some((*words, *target))
                    }
                    _ => None,
                })
                .collect::<Vec<_>>(),
            [
                (0x140, MemoryToVdpDmaTarget::Vram),
                (0x0C0, MemoryToVdpDmaTarget::Vram),
            ],
            "ASM sub_1492 -> sub_13C0 loc_141A queues one 12-tile Kid mapping for loc_AD4",
        );

        let empty = visit_mode_level_sprite_and_kid_to_vram([0; 6], |_| {})
            .expect("first two gameplay VBlanks consume an empty loc_AD4 queue");
        assert_eq!(empty.instruction_count, 57);
        assert_eq!(empty.raw_cycles, M68kCycleCount::from_raw(822));
        assert_eq!(
            visit_mode_level_sprite_and_kid_to_vram([17, 0, 0, 0, 0, 0], |_| {}),
            None,
            "a loc_AD4 entry larger than one 32x32 hardware sprite remains fail-closed",
        );
        assert_eq!(
            visit_mode_level_sprite_and_kid_to_vram([0, 12, 0, 0, 0, 0], |_| {}),
            None,
            "ASM loc_AD4 cannot consume data after its zero terminator",
        );
    }

    #[test]
    fn scroll_data_transfer_retains_vsram_dma_and_nested_start_path() {
        let mut events = Vec::new();
        let timing = visit_mode_standard_scroll_data_to_vram(|event| events.push(event));

        assert_eq!(timing.instruction_count, 42);
        assert_eq!(timing.raw_cycles, M68kCycleCount::from_raw(610));
        assert_eq!(
            events
                .iter()
                .filter(|event| !matches!(event, MemoryDmaTimingEvent::Instruction(_)))
                .copied()
                .collect::<Vec<_>>(),
            [
                MemoryDmaTimingEvent::WriteVdpWord {
                    target: MemoryToVdpDmaTarget::Vsram,
                },
                MemoryDmaTimingEvent::WriteVdpWord {
                    target: MemoryToVdpDmaTarget::Vsram,
                },
                MemoryDmaTimingEvent::SetZ80BusRequest(true),
                MemoryDmaTimingEvent::BeginMemoryDma {
                    words: 0x1C0,
                    target: MemoryToVdpDmaTarget::Vram,
                },
                MemoryDmaTimingEvent::SetZ80BusRequest(false),
                MemoryDmaTimingEvent::SetZ80BusRequest(false),
            ],
            "ASM loc_300D2 brackets the byte-sized $380 DMA and sub_924 repeats Start_z80",
        );
    }

    #[test]
    fn title_map_tail_reaches_load_art_after_exact_map_and_permutation_loops() {
        let mut events = Vec::new();
        let timing = visit_title_map_to_load_art_palette_permutation(|event| events.push(event));

        assert_eq!(timing.instruction_count, 2_057);
        assert_eq!(timing.raw_cycles, M68kCycleCount::from_raw(19_472));
        assert_eq!(
            events
                .iter()
                .filter(|event| matches!(event, MemoryDmaTimingEvent::WriteVdpWord { .. }))
                .count(),
            32,
            "ASM loc_1B322 uploads exactly 32 title-map words before loc_1C4B2",
        );
    }
}
