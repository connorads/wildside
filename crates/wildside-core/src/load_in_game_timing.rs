#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct ForegroundCollisionExpansionTimingLoc11c34 {
    pub(crate) cells: u32,
    pub(crate) instruction_count: u32,
    pub(crate) raw_cycles: u32,
}

/// Visit the source instructions which expand decompressed foreground bytes
/// into `Level_Layout` collision words.
///
/// ASM `loc_11C34`/`loc_11C36` (kid.asm:23770-23784) executes one inner loop
/// per `Level_width_blocks * Level_height_blocks` cell. The two loop-ending
/// branches use the shorter not-taken timing on the final cell and row.
pub(crate) fn visit_foreground_collision_expansion_instructions_loc_11c34(
    width_blocks: u16,
    height_blocks: u16,
    mut visit: impl FnMut(u32),
) -> ForegroundCollisionExpansionTimingLoc11c34 {
    let mut instruction_count = 0_u32;
    let mut raw_cycles = 0_u32;
    let mut emit = |cycles: u32| {
        instruction_count += 1;
        raw_cycles += cycles;
        visit(cycles);
    };

    // ASM setup immediately before loc_11C34: `moveq #0,d0; moveq #0,d2`.
    emit(4);
    emit(4);
    for row in 0..height_blocks {
        // ASM loc_11C34: `moveq #0,d1`.
        emit(4);
        for column in 0..width_blocks {
            // ASM loc_11C36 through the width-loop branch.
            for cycles in [8, 14, 8, 8, 8, 4, 12] {
                emit(cycles);
            }
            emit(if column + 1 == width_blocks { 8 } else { 10 });
        }
        // ASM loc_11C4E: advance and compare the row counter.
        emit(4);
        emit(12);
        emit(if row + 1 == height_blocks { 8 } else { 10 });
    }

    ForegroundCollisionExpansionTimingLoc11c34 {
        cells: u32::from(width_blocks) * u32::from(height_blocks),
        instruction_count,
        raw_cycles,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn loc_11c34_visits_every_cell_and_both_dbf_style_loop_exits() {
        let mut visited = Vec::new();
        let timing = visit_foreground_collision_expansion_instructions_loc_11c34(2, 2, |cycles| {
            visited.push(cycles);
        });

        assert_eq!(timing.cells, 4);
        assert_eq!(timing.instruction_count, 42);
        assert_eq!(timing.raw_cycles, 350);
        assert_eq!(visited.len(), timing.instruction_count as usize);
        assert_eq!(visited.iter().copied().sum::<u32>(), timing.raw_cycles);
        assert_eq!(visited.iter().filter(|&&cycles| cycles == 8).count(), 19);
        assert_eq!(visited.iter().filter(|&&cycles| cycles == 10).count(), 3);
    }

    #[test]
    fn issue_300_foreground_cell_work_separates_both_teleporter_destinations() {
        let short = visit_foreground_collision_expansion_instructions_loc_11c34(20, 56, |_| {});
        let long = visit_foreground_collision_expansion_instructions_loc_11c34(260, 28, |_| {});

        assert_eq!(
            (short.cells, short.instruction_count, short.raw_cycles),
            (1_120, 9_186, 82_214)
        );
        assert_eq!(
            (long.cells, long.instruction_count, long.raw_cycles),
            (7_280, 58_354, 524_950)
        );
        assert_eq!(long.raw_cycles - short.raw_cycles, 442_736);
    }
}
