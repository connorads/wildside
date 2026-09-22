#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct ForegroundCollisionExpansionTimingLoc11c34 {
    pub(crate) cells: u32,
    pub(crate) instruction_count: u32,
    pub(crate) raw_cycles: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum BackgroundMappingsFormat {
    Chunked,
    Layered,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct BackgroundMappingsWorkload {
    pub(crate) format: BackgroundMappingsFormat,
    pub(crate) placements: u32,
    pub(crate) cells_visited: u32,
    pub(crate) cells_copied: u32,
    pub(crate) enigma_streams: u32,
    pub(crate) enigma_instruction_count: u32,
    pub(crate) enigma_raw_cycles: u32,
    pub(crate) enigma_output_words: u32,
}

/// Census the direct background-layout path and every source cell considered
/// by `Load_BackgroundChunk`.
///
/// ASM `loc_12830` reads `(chunk,x,y)` triples until a negative word. ASM
/// `loc_12884` then visits every byte in the selected chunk, while the four
/// bounds branches decide whether that byte reaches the background buffer.
pub(crate) fn background_chunk_mappings_workload_loc_12830(
    layout: &[u8],
    background_width: u16,
    background_height: u16,
    mut load_chunk: impl FnMut(u16) -> Option<Vec<u8>>,
) -> Option<BackgroundMappingsWorkload> {
    let mut offset = 0_usize;
    let mut placements = 0_u32;
    let mut cells_visited = 0_u32;
    let mut cells_copied = 0_u32;
    loop {
        let chunk_index = read_word(layout, offset)?;
        offset += 2;
        if chunk_index & 0x8000 != 0 {
            break;
        }
        let x = read_word(layout, offset)? as i16;
        let y = read_word(layout, offset + 2)? as i16;
        offset += 4;
        let chunk = load_chunk(chunk_index)?;
        let width = read_word(&chunk, 0)?;
        let height = read_word(&chunk, 2)?;
        if chunk.len() < 4 + usize::from(width) * usize::from(height) {
            return None;
        }
        placements += 1;
        cells_visited += u32::from(width) * u32::from(height);
        for row in 0..height {
            for column in 0..width {
                let target_x = x.wrapping_add_unsigned(column);
                let target_y = y.wrapping_add_unsigned(row);
                if target_x >= 0
                    && target_x < background_width as i16
                    && target_y >= 0
                    && target_y < background_height as i16
                {
                    cells_copied += 1;
                }
            }
        }
    }

    Some(BackgroundMappingsWorkload {
        format: BackgroundMappingsFormat::Chunked,
        placements,
        cells_visited,
        cells_copied,
        enigma_streams: 0,
        enigma_instruction_count: 0,
        enigma_raw_cycles: 0,
        enigma_output_words: 0,
    })
}

/// Census the layered background path using the existing exact EniDec visitor.
///
/// ASM `Load_BackgroundMappings_Layered loc_128BA` dispatches a theme-specific
/// list of Enigma streams. Extracted streams start at even ROM addresses, so
/// the visitor takes the even return-alignment branch at `loc_1357A`.
pub(crate) fn background_layered_mappings_workload_loc_128ba(
    streams: &[(&[u8], u16)],
) -> Option<BackgroundMappingsWorkload> {
    let mut enigma_instruction_count = 0_u32;
    let mut enigma_raw_cycles = 0_u32;
    let mut enigma_output_words = 0_u32;
    for &(stream, base_tile_word) in streams {
        let timing =
            crate::enigma_timing::visit_enigma_instructions(stream, base_tile_word, 0, |_| {})
                .ok()?;
        enigma_instruction_count += timing.instruction_count;
        enigma_raw_cycles += timing.raw_cycles;
        enigma_output_words += u32::from(timing.output_words);
    }
    Some(BackgroundMappingsWorkload {
        format: BackgroundMappingsFormat::Layered,
        placements: 0,
        cells_visited: 0,
        cells_copied: 0,
        enigma_streams: streams.len() as u32,
        enigma_instruction_count,
        enigma_raw_cycles,
        enigma_output_words,
    })
}

fn read_word(bytes: &[u8], offset: usize) -> Option<u16> {
    Some(u16::from_be_bytes([
        *bytes.get(offset)?,
        *bytes.get(offset + 1)?,
    ]))
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

    #[test]
    fn loc_12830_counts_visited_and_clipped_chunk_cells() {
        let layout = [
            0, 0, 0xff, 0xff, 0, 0, // chunk 0 at (-1, 0)
            0xff, 0xff, // terminator
        ];
        let workload = background_chunk_mappings_workload_loc_12830(&layout, 2, 2, |_| {
            Some(vec![0, 2, 0, 2, 1, 2, 3, 4])
        })
        .expect("valid direct mapping");

        assert_eq!(workload.format, BackgroundMappingsFormat::Chunked);
        assert_eq!(workload.placements, 1);
        assert_eq!(workload.cells_visited, 4);
        assert_eq!(workload.cells_copied, 2);
    }
}
