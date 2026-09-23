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
    pub(crate) chunk_loop_instruction_count: u32,
    pub(crate) chunk_loop_raw_cycles: u32,
    pub(crate) chunk_wrapper_instruction_count: u32,
    pub(crate) chunk_wrapper_raw_cycles: u32,
    pub(crate) enigma_streams: u32,
    pub(crate) enigma_instruction_count: u32,
    pub(crate) enigma_raw_cycles: u32,
    pub(crate) enigma_output_words: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct BlockLayoutDecodeWorkloadLoc132a6 {
    pub(crate) x_width: u8,
    pub(crate) y_width: u8,
    pub(crate) groups: u32,
    pub(crate) entries: u32,
    pub(crate) property_words: u32,
    pub(crate) bit_read_calls: u32,
    pub(crate) bits_requested: u32,
    pub(crate) rolling_refills: u32,
    pub(crate) bytes_consumed: u32,
    pub(crate) bit_reader_instruction_count: u32,
    pub(crate) bit_reader_raw_cycles: u32,
    pub(crate) decoder_instruction_count: u32,
    pub(crate) decoder_raw_cycles: u32,
}

/// Trace the compressed block-layout branches selected by the ROM bit stream.
///
/// ASM `LoadBlockLayout loc_132A6` uses `sub_13622` for every field and
/// `loc_13650` for byte refills. Retaining those calls is required because two
/// layouts with the same decoded words can cross different refill boundaries.
pub(crate) fn block_layout_decode_workload_loc_132a6(
    bytes: &[u8],
) -> Option<BlockLayoutDecodeWorkloadLoc132a6> {
    if bytes.first() != Some(&0x40) {
        return None;
    }
    let mut padded = bytes.to_vec();
    padded.extend_from_slice(&[0; 8]);
    let mut reader = BlockLayoutBitReader::new(&padded)?;
    reader.read(8)?;
    let x_width = reader.read(4)? as u8;
    let y_width = reader.read(4)? as u8;
    let x_sentinel = if x_width == 0 {
        0
    } else {
        (1_u16 << x_width) - 1
    };
    let mut groups = 0_u32;
    let mut entries = 0_u32;
    let mut property_words = 0_u32;
    let mut decoder_instruction_count = 0_u32;
    let mut decoder_raw_cycles = 0_u32;
    let mut emit_decoder = |cycles: &[u32]| {
        decoder_instruction_count += cycles.len() as u32;
        decoder_raw_cycles += cycles.iter().sum::<u32>();
    };
    // ASM loc_132A6: save 14 registers (a1 and a7 excluded), initialise the
    // reader, decode the two field widths and select the X sentinel mask.
    emit_decoder(&[120, 4, 18, 12, 4, 18, 4, 18, 4, 4, 8, 14, 4, 18, 4]);

    loop {
        emit_decoder(&[4, 18, 8, 4, 4, 18, 8]);
        reader.read(1)?;
        let block_type = reader.read(5)?;
        if block_type == 0x1f {
            emit_decoder(&[10, 12, 12, 124, 16]);
            break;
        }
        emit_decoder(&[12, 4, 8, 4, 14, 14]);
        groups += 1;
        loop {
            emit_decoder(&[4, 18, 4]);
            let x = reader.read(x_width.max(1))?;
            if x == x_sentinel {
                emit_decoder(&[8, 12, 10]);
                break;
            }
            emit_decoder(&[10, 8, 4, 18, 8]);
            entries += 1;
            reader.read(y_width.max(1))?;
            match block_type {
                0 | 2 | 5 | 6 | 7 | 8 | 9 | 13 | 14 | 15 => {
                    emit_decoder(&[4, 18, 4, 8, 4, 18, 8, 10]);
                    reader.read(4)?;
                    reader.read(1)?;
                }
                1 => {
                    emit_decoder(&[4, 18, 4, 4, 8, 4, 18, 8]);
                    let count = reader.read(4)? + 1;
                    reader.read(1)?;
                    property_words += u32::from(count);
                    for index in 0..count {
                        emit_decoder(&[4, 18, 4]);
                        let high_bit = reader.read(1)? != 0;
                        emit_decoder(&[if high_bit { 8 } else { 10 }, 4, 18]);
                        reader.read(4)?;
                        if high_bit {
                            emit_decoder(&[8, 10]);
                        }
                        emit_decoder(&[8, if index + 1 == count { 14 } else { 10 }]);
                    }
                    emit_decoder(&[10]);
                }
                10 | 12 => {
                    emit_decoder(&[4, 18, 4, 4, 8, 4, 18, 8]);
                    let count = reader.read(4)? + 1;
                    reader.read(1)?;
                    property_words += u32::from(count);
                    for index in 0..count {
                        emit_decoder(&[4, 18]);
                        reader.read(4)?;
                        emit_decoder(&[8, if index + 1 == count { 14 } else { 10 }]);
                    }
                    emit_decoder(&[10]);
                }
                3 => {
                    for (index, width) in [3, 1, 8, 3, 8, 8].into_iter().enumerate() {
                        emit_decoder(&[4, 18]);
                        reader.read(width)?;
                        if index == 0 {
                            emit_decoder(&[4, 8]);
                        } else {
                            emit_decoder(&[8]);
                        }
                    }
                    emit_decoder(&[10]);
                }
                4 => {
                    emit_decoder(&[4, 18]);
                    reader.read(16)?;
                    emit_decoder(&[4, 8, 4, 18]);
                    reader.read(9)?;
                    emit_decoder(&[8, 10]);
                }
                11 => emit_decoder(&[10]),
                16 => {
                    emit_decoder(&[4, 18, 4, 4, 8, 4, 18, 8]);
                    let count = reader.read(4)? + 1;
                    reader.read(1)?;
                    property_words += u32::from(count);
                    for index in 0..count {
                        emit_decoder(&[4, 18]);
                        reader.read(3)?;
                        emit_decoder(&[8, if index + 1 == count { 14 } else { 10 }]);
                    }
                    emit_decoder(&[10]);
                }
                _ => return None,
            }
        }
    }

    Some(BlockLayoutDecodeWorkloadLoc132a6 {
        x_width,
        y_width,
        groups,
        entries,
        property_words,
        bit_read_calls: reader.bit_read_calls,
        bits_requested: reader.bits_requested,
        rolling_refills: reader.rolling_refills,
        bytes_consumed: reader.position as u32,
        bit_reader_instruction_count: reader.instruction_count,
        bit_reader_raw_cycles: reader.raw_cycles,
        decoder_instruction_count,
        decoder_raw_cycles,
    })
}

struct BlockLayoutBitReader<'a> {
    bytes: &'a [u8],
    position: usize,
    word: u16,
    remaining: u8,
    bit_read_calls: u32,
    bits_requested: u32,
    rolling_refills: u32,
    instruction_count: u32,
    raw_cycles: u32,
}

impl<'a> BlockLayoutBitReader<'a> {
    fn new(bytes: &'a [u8]) -> Option<Self> {
        Some(Self {
            bytes,
            position: 2,
            word: u16::from_be_bytes([*bytes.first()?, *bytes.get(1)?]),
            remaining: 16,
            bit_read_calls: 0,
            bits_requested: 0,
            rolling_refills: 0,
            // ASM sub_13618 seeds d5/d6 before the first sub_13622 call.
            instruction_count: 5,
            raw_cycles: 58,
        })
    }

    fn read(&mut self, width: u8) -> Option<u16> {
        if width == 0 || width > 16 {
            return None;
        }
        self.bit_read_calls += 1;
        self.bits_requested += u32::from(width);
        if width <= 8 {
            self.emit(&[8, 10, 18]);
            let value = self.read_short(width)?;
            self.emit(&[16]);
            return Some(value);
        }
        self.emit(&[8, 8, 4, 18]);
        let high = self.read_short(width - 8)?;
        self.emit(&[22, 8, 4, 18]);
        let low = self.read_short(8)?;
        self.emit(&[8, 16]);
        Some((high << 8) | low)
    }

    fn read_short(&mut self, width: u8) -> Option<u16> {
        if width > self.remaining {
            return None;
        }
        let shift = self.remaining - width;
        let mask = (1_u16 << width) - 1;
        let value = (self.word >> shift) & mask;
        // ASM sub_1363C, the immediate shift in sub_1364C, then loc_13650.
        self.emit(&[4, 4, 4, 6 + 2 * u32::from(shift), 4, 14, 16, 8]);
        self.remaining -= width;
        let rolling_refill = self.remaining < 9;
        self.emit(&[4, 8, if rolling_refill { 12 } else { 10 }]);
        if rolling_refill {
            self.remaining += 8;
            self.word = (self.word << 8) | u16::from(*self.bytes.get(self.position)?);
            self.position += 1;
            self.rolling_refills += 1;
            self.emit(&[8, 22, 8]);
        }
        self.emit(&[16]);
        Some(value)
    }

    fn emit(&mut self, cycles: &[u32]) {
        self.instruction_count += cycles.len() as u32;
        self.raw_cycles += cycles.iter().sum::<u32>();
    }
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
    let mut chunk_loop_instruction_count = 0_u32;
    let mut chunk_loop_raw_cycles = 0_u32;
    let mut chunk_wrapper_instruction_count = 0_u32;
    let mut chunk_wrapper_raw_cycles = 0_u32;
    let mut emit = |cycles: &[u32]| {
        chunk_loop_instruction_count += cycles.len() as u32;
        chunk_loop_raw_cycles += cycles.iter().sum::<u32>();
    };
    let mut emit_wrapper = |cycles: &[u32]| {
        chunk_wrapper_instruction_count += cycles.len() as u32;
        chunk_wrapper_raw_cycles += cycles.iter().sum::<u32>();
    };
    // ASM Load_BackgroundMappings: select the direct format, preserve d5-d7,
    // initialise both offsets and branch into loc_12830.
    emit_wrapper(&[12, 12, 12, 12, 32, 4, 4, 10]);
    loop {
        let chunk_index = read_word(layout, offset)?;
        offset += 2;
        if chunk_index & 0x8000 != 0 {
            // ASM loc_12830/loc_12842: terminal BMI, register restore and RTS.
            emit_wrapper(&[8, 10, 36, 16]);
            break;
        }
        let x = read_word(layout, offset)? as i16;
        let y = read_word(layout, offset + 2)? as i16;
        offset += 4;
        let chunk = load_chunk(chunk_index)?;
        let width = read_word(&chunk, 0)?;
        let height = read_word(&chunk, 2)?;
        if width == 0 || height == 0 || chunk.len() < 4 + usize::from(width) * usize::from(height) {
            return None;
        }
        // ASM loc_12830 plus Load_BackgroundChunk up to loc_12884. The MULS
        // uses Background_width as its source operand, whose bit transitions
        // determine 68000 execution time.
        emit_wrapper(&[
            8,
            8,
            8,
            4,
            8,
            4,
            18,
            24,
            4,
            4,
            12,
            20,
            12,
            4,
            4,
            18,
            8,
            muls_word_absolute_short_cycles(background_width),
            8,
            8,
            4,
            4,
            18,
            8,
            8,
            4,
            4,
        ]);
        placements += 1;
        cells_visited += u32::from(width) * u32::from(height);
        for row in 0..height {
            // ASM loc_12884: reset the inner DBF counter, destination and X.
            emit(&[4, 4, 4]);
            for column in 0..width {
                let target_x = x.wrapping_add_unsigned(column);
                let target_y = y.wrapping_add_unsigned(row);
                emit(&[4]);
                if target_x < 0 {
                    emit(&[10]);
                } else {
                    emit(&[8, 12]);
                    if target_x >= background_width as i16 {
                        emit(&[10]);
                    } else {
                        emit(&[8, 4]);
                        if target_y < 0 {
                            emit(&[10]);
                        } else {
                            emit(&[8, 12]);
                            if target_y >= background_height as i16 {
                                emit(&[10]);
                            } else {
                                emit(&[8, 12]);
                                cells_copied += 1;
                            }
                        }
                    }
                }
                // ASM loc_128A0 and the inner DBF.
                emit(&[8, 8, 4, if column + 1 == width { 14 } else { 10 }]);
            }
            // ASM loc_128A8 and the outer DBF.
            emit(&[12, 4, if row + 1 == height { 14 } else { 10 }]);
        }
        // ASM loc_128AC/loc_12842: restore the chunk registers, return to the
        // placement caller and branch back to loc_12830.
        emit_wrapper(&[28, 16, 10]);
    }

    Some(BackgroundMappingsWorkload {
        format: BackgroundMappingsFormat::Chunked,
        placements,
        cells_visited,
        cells_copied,
        chunk_loop_instruction_count,
        chunk_loop_raw_cycles,
        chunk_wrapper_instruction_count,
        chunk_wrapper_raw_cycles,
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
        chunk_loop_instruction_count: 0,
        chunk_loop_raw_cycles: 0,
        chunk_wrapper_instruction_count: 0,
        chunk_wrapper_raw_cycles: 0,
        enigma_streams: streams.len() as u32,
        enigma_instruction_count,
        enigma_raw_cycles,
        enigma_output_words,
    })
}

fn muls_word_absolute_short_cycles(source: u16) -> u32 {
    // Motorola 68000 MULS timing as implemented by the oracle's m68000 core:
    // 38 + two cycles for every source-bit transition, plus eight cycles for
    // the absolute-short word source used at Load_BackgroundChunk.
    let mut last_bit = false;
    let mut transitions = 0_u32;
    for bit in 0..16 {
        let current = source & (1 << bit) != 0;
        if current != last_bit {
            transitions += 1;
        }
        last_bit = current;
    }
    46 + 2 * transitions
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
        assert_eq!(workload.chunk_loop_instruction_count, 50);
        assert_eq!(workload.chunk_loop_raw_cycles, 388);
        assert_eq!(muls_word_absolute_short_cycles(2), 50);
        assert_eq!(workload.chunk_wrapper_instruction_count, 42);
        assert_eq!(workload.chunk_wrapper_raw_cycles, 506);
    }

    #[test]
    fn loc_132a6_retains_bit_reads_and_rolling_refills() {
        // Raw `level/block/05.bin`, also used by the data decoder golden.
        let bytes = [
            0x40, 0x65, 0x84, 0x60, 0x81, 0x92, 0xd0, 0x0c, 0x35, 0x80, 0x9b, 0x4c, 0x06, 0xbf,
            0x19, 0x8c, 0xf8, 0x8c, 0xf8, 0x0c, 0xbb, 0xf1, 0x45, 0x28, 0x89, 0x28, 0x9d, 0x28,
            0xa1, 0x28, 0xa5, 0x30, 0x0d, 0x41, 0x5a, 0x41, 0x44, 0x53, 0xe3, 0x53, 0xd0, 0x68,
            0xd7, 0x68, 0xcd, 0x89, 0x5a, 0x89, 0x7f, 0xfc,
        ];
        let workload =
            block_layout_decode_workload_loc_132a6(&bytes).expect("valid compressed block layout");

        assert_eq!(workload.x_width, 6);
        assert_eq!(workload.y_width, 5);
        assert!(workload.groups > 1);
        assert!(workload.entries > 1);
        assert!(workload.bit_read_calls > 3);
        assert!(workload.rolling_refills > 0);
        assert!(workload.bit_reader_instruction_count > workload.bit_read_calls);
        assert!(workload.bit_reader_raw_cycles > workload.bits_requested);
        assert_eq!(
            (
                workload.decoder_instruction_count,
                workload.decoder_raw_cycles
            ),
            (444, 4_348)
        );
    }
}
