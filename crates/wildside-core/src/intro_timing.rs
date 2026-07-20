use crate::M68kCycleCount;

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

/// Visit `Mode_Standard` entry through completion of `Do_Nothing`.
///
/// ASM `Mode_Standard` $77A calls `Do_Nothing` $956, whose DBF delay runs
/// 612 taken iterations and one terminal iteration before returning.
pub fn visit_mode_standard_do_nothing(mut visit: impl FnMut(u32)) -> IntroSourceTiming {
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
                .expect("Mode_Standard source timing fits u32"),
        );
    };

    // ASM Mode_Standard $77A: BSR.W Do_Nothing; save d0 and load DBF count.
    for cycles in [18, 8, 8] {
        emit(cycles);
    }
    for iteration in 0..=612 {
        emit(if iteration == 612 { 14 } else { 10 });
    }
    // ASM Do_Nothing $960: restore d0 and return to $77E.
    for cycles in [8, 16] {
        emit(cycles);
    }

    timing
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
        18, 16, 8, 16, 16, 8, 4, 16, 16, 10, 18, 10, 16, 24, 16, 76, 16, 10,
    ] {
        emit(cycles);
    }
    // The controller has no successor; loc_E58 restores the caller stack.
    for cycles in [12, 10, 16, 16] {
        emit(cycles);
    }

    Some(timing)
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

/// Visit the first title `sub_14C0` palette interpolation through frame RTS.
///
/// Both 32-bit dirty masks are `$FFFFFFFF` after `sub_1C204`, so the routine
/// builds all 15 signed interpolation steps and runs the full 32-colour
/// `sub_1536` body for both palette halves before returning to `$7A4`.
pub fn visit_title_first_palette_interpolation(mut visit: impl FnMut(u32)) -> IntroSourceTiming {
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

    // ASM Mode_Standard $7A0 and sub_14C0 setup through loc_14EE.
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

/// Visit `Mode_Standard`'s first `Palette_to_VRAM` call from caller `$77E`.
///
/// ASM `Palette_to_VRAM $970`, `Stop_z80 $E13BC`, and `Start_z80 $E13D0`
/// retain their instruction boundaries. BUSREQ side effects occur after the
/// corresponding register write, while the CRAM DMA begins after the second
/// VDP command word.
pub fn visit_mode_standard_palette_to_vram(
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

    // ASM Mode_Standard $77E and Palette_to_VRAM $970: BSR, flag test,
    // fall-through branch and Stop_z80 JSR.
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

/// Visit the title path through `Transfer_SpriteAndKidToVRAM`.
///
/// The title has a live sprite-table pointer, `$FB49` clear, and a zero first
/// Kid-art length at `$FF0500`. It therefore performs the fixed 320-word
/// Sprite_Table DMA, skips the Kid-art DMA loop at `loc_AD4`, then resets the
/// two producer pointers through `sub_B52`.
pub fn visit_mode_standard_sprite_and_kid_to_vram(
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
    for cycles in [18, 16, 14, 12, 4, 18] {
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

    // ASM $AAC..loc_B36: clear title Kid list and take zero-length exit.
    for cycles in [12, 12, 16, 4, 10, 12, 4, 12, 8, 8, 10] {
        emit(&mut visit, &mut timing, cycles);
    }
    // ASM loc_B36 -> Start_z80.
    for cycles in [20, 10, 10, 20] {
        emit(&mut visit, &mut timing, cycles);
    }
    visit(MemoryDmaTimingEvent::SetZ80BusRequest(false));
    // Start_z80 RTS; JSR.W sub_B52 and its pointer-reset body; outer RTS.
    for cycles in [16, 18, 12, 12, 16, 12, 16, 16, 16] {
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
    for cycles in [20, 12, 4, 10, 12, 20, 8] {
        emit(&mut visit, &mut timing, cycles);
    }
    visit(MemoryDmaTimingEvent::WriteVdpWord {
        target: MemoryToVdpDmaTarget::Vsram,
    });
    for cycles in [10, 8, 10, 8] {
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
    for cycles in [18, 10, 12, 8, 12, 12, 20, 10, 10, 20] {
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
    fn second_title_execute_objects_hibernates_text_worker_then_controller() {
        let mut cycles = Vec::new();
        let timing = visit_title_second_execute_objects(0xFFD1, |cycle| cycles.push(cycle))
            .expect("frame-82 title RNG offset is initialised");

        assert_eq!(timing.instruction_count, 159);
        assert_eq!(timing.raw_cycles, M68kCycleCount::from_raw(2_040));
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
}
