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
///
/// ASM `loc_726` calls `Initialize_ObjectSlots`,
/// `Initialize_GfxObjectSlots`, and `sub_8A4`, then dispatches the current
/// game mode through `LoadGameModeData`. The published title-return path
/// selects `Load_TitleScreen`; this visitor stops before the first
/// `sub_E151C` instruction which queues `bgm_Ice`.
pub fn visit_title_restart_initialisers_to_play_sound(
    mut visit: impl FnMut(u32),
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

    // ASM loc_732 -> j_LoadGameModeData -> Load_TitleScreen -> j_PlaySound
    // -> PlaySound. The indexed GameLoadArray read costs 18 cycles.
    for cycles in [20, 10, 12, 18, 16, 8, 20, 10, 12, 12, 18] {
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
