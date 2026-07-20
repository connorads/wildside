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

/// Destination selected by a 68000-to-VDP DMA command.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MemoryToVdpDmaTarget {
    Vram,
    Cram,
    Vsram,
}

/// Source-positioned event within `Palette_to_VRAM`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PaletteToVramTimingEvent {
    Instruction(u32),
    SetZ80BusRequest(bool),
    BeginMemoryDma {
        words: u16,
        target: MemoryToVdpDmaTarget,
    },
}

/// Visit `Mode_Standard`'s first `Palette_to_VRAM` call from caller `$77E`.
///
/// ASM `Palette_to_VRAM $970`, `Stop_z80 $E13BC`, and `Start_z80 $E13D0`
/// retain their instruction boundaries. BUSREQ side effects occur after the
/// corresponding register write, while the CRAM DMA begins after the second
/// VDP command word.
pub fn visit_mode_standard_palette_to_vram(
    mut visit: impl FnMut(PaletteToVramTimingEvent),
) -> IntroSourceTiming {
    let mut timing = IntroSourceTiming {
        instruction_count: 0,
        raw_cycles: M68kCycleCount::from_raw(0),
    };
    let emit = |visit: &mut dyn FnMut(PaletteToVramTimingEvent),
                timing: &mut IntroSourceTiming,
                cycles: u32| {
        visit(PaletteToVramTimingEvent::Instruction(cycles));
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
    visit(PaletteToVramTimingEvent::SetZ80BusRequest(true));
    for cycles in [20, 8, 16] {
        emit(&mut visit, &mut timing, cycles);
    }

    // ASM dma68kToVDP Palette_Buffer,$0000,$80,CRAM.
    for cycles in [24, 24, 16, 24, 20, 20] {
        emit(&mut visit, &mut timing, cycles);
    }
    visit(PaletteToVramTimingEvent::BeginMemoryDma {
        words: 0x40,
        target: MemoryToVdpDmaTarget::Cram,
    });

    // ASM j_Start_z80 $E1308 -> BranchTo_Start_z80 -> Start_z80 $E13D0.
    for cycles in [20, 10, 10, 20] {
        emit(&mut visit, &mut timing, cycles);
    }
    visit(PaletteToVramTimingEvent::SetZ80BusRequest(false));
    // Start_z80 RTS, then Palette_to_VRAM RTS.
    for cycles in [16, 16] {
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
                .filter(|event| !matches!(event, PaletteToVramTimingEvent::Instruction(_)))
                .copied()
                .collect::<Vec<_>>(),
            [
                PaletteToVramTimingEvent::SetZ80BusRequest(true),
                PaletteToVramTimingEvent::BeginMemoryDma {
                    words: 0x40,
                    target: MemoryToVdpDmaTarget::Cram,
                },
                PaletteToVramTimingEvent::SetZ80BusRequest(false),
            ],
        );
    }
}
