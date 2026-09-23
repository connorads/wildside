#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct ForegroundCollisionExpansionTimingLoc11c34 {
    pub(crate) cells: u32,
    pub(crate) instruction_count: u32,
    pub(crate) raw_cycles: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct ForegroundTerrainDecoderWorkloadLoc11c10 {
    pub(crate) decoder_calls: u32,
    pub(crate) output_bytes: u32,
    pub(crate) instruction_count: u32,
    pub(crate) raw_cycles: u32,
    pub(crate) wrapper_instruction_count: u32,
    pub(crate) wrapper_raw_cycles: u32,
}

/// Measure each ASM `Decompress_Chunk loc_11C10` call selected by the
/// foreground terrain stream, including resumed decoder windows.
pub(crate) fn foreground_terrain_decoder_workload_loc_11c10(
    stream: &[u8],
) -> anyhow::Result<ForegroundTerrainDecoderWorkloadLoc11c10> {
    visit_foreground_terrain_decoder_instructions_loc_11c10(stream, |_| {})
}

/// Visit the ordered source instructions selected by ASM `loc_11C10`.
pub(crate) fn visit_foreground_terrain_decoder_instructions_loc_11c10(
    stream: &[u8],
    mut visit: impl FnMut(u32),
) -> anyhow::Result<ForegroundTerrainDecoderWorkloadLoc11c10> {
    let mut workload = ForegroundTerrainDecoderWorkloadLoc11c10 {
        decoder_calls: 0,
        output_bytes: 0,
        instruction_count: 0,
        raw_cycles: 0,
        wrapper_instruction_count: 0,
        wrapper_raw_cycles: 0,
    };
    let mut resume = None;
    loop {
        // ASM loc_11C10: BSR.W Decompress_Chunk.
        visit(18);
        let decoder = crate::saxman_timing::visit_decompress_chunk_instructions_sub_13684(
            stream, resume, &mut visit,
        )?;
        workload.decoder_calls += 1;
        workload.output_bytes += u32::from(decoder.timing.output_bytes);
        workload.instruction_count += decoder.timing.instruction_count;
        workload.raw_cycles += decoder.timing.raw_cycles;
        resume = decoder.resume;
        // ASM loc_11C14/loc_11C18: TST.W d0; BNE.S loc_11C10.
        visit(4);
        let branch_cycles = if resume.is_some() { 10 } else { 8 };
        visit(branch_cycles);
        workload.wrapper_instruction_count += 3;
        workload.wrapper_raw_cycles += 22 + branch_cycles;
        if resume.is_none() {
            return Ok(workload);
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct ThemeArtDecoderWorkloadLoc11e60 {
    pub(crate) streams: u32,
    pub(crate) output_bytes: u32,
    pub(crate) decoder_calls: u32,
    pub(crate) instruction_count: u32,
    pub(crate) raw_cycles: u32,
    pub(crate) wrapper_instruction_count: u32,
    pub(crate) wrapper_raw_cycles: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct MurderWallArtWorkloadLoc140d2 {
    pub(crate) output_bytes: u32,
    pub(crate) decoder_calls: u32,
    pub(crate) instruction_count: u32,
    pub(crate) raw_cycles: u32,
    pub(crate) wrapper_instruction_count: u32,
    pub(crate) wrapper_raw_cycles: u32,
}

/// Measure ASM `Murderwall loc_140D2`'s `DecompressToVRAM` art load.
pub(crate) fn murder_wall_art_workload_loc_140d2(
    stream: &[u8],
) -> anyhow::Result<MurderWallArtWorkloadLoc140d2> {
    let mut workload = MurderWallArtWorkloadLoc140d2 {
        output_bytes: 0,
        decoder_calls: 0,
        instruction_count: 0,
        raw_cycles: 0,
        // ASM loc_140D2: LEA/MOVE/BSR, then DecompressToVRAM's sub_142CC setup.
        wrapper_instruction_count: 3 + 19,
        wrapper_raw_cycles: 12 + 8 + 18 + 174,
    };
    let mut resume = None;
    loop {
        let visit = crate::saxman_timing::visit_decompress_chunk_instructions_sub_13684(
            stream,
            resume,
            |_| {},
        )?;
        workload.decoder_calls += 1;
        workload.output_bytes += u32::from(visit.timing.output_bytes);
        workload.instruction_count += visit.timing.instruction_count;
        workload.raw_cycles += visit.timing.raw_cycles;
        let output_words = u32::from(visit.timing.output_bytes / 2);
        // ASM loc_14300/loc_14314: decoder entry/exit, one VDP write and DBF
        // per word, then either the history rollover or final return.
        workload.wrapper_instruction_count += 7 + 2 * output_words;
        workload.wrapper_raw_cycles += 58 + 22 * output_words + 4;
        if visit.resume.is_some() {
            workload.wrapper_instruction_count += 1_034;
            workload.wrapper_raw_cycles += 15_414;
        } else {
            workload.wrapper_instruction_count += 3;
            workload.wrapper_raw_cycles += 30;
        }
        resume = visit.resume;
        if resume.is_none() {
            return Ok(workload);
        }
    }
}

/// Measure the data-dependent Saxman decoder work selected by the theme loads.
///
/// ASM `loc_11E60`, `loc_11EA2` and the conditional `loc_11EF2` calls all
/// enter `DecompressToRAM`, whose source-dependent body is `Decompress_Chunk
/// sub_13684`. The fixed permutation and VDP-write wrapper is deliberately
/// outside this workload until it can be scheduled at its real VDP waits.
pub(crate) fn theme_art_decoder_workload_loc_11e60(
    streams: &[&[u8]],
) -> anyhow::Result<ThemeArtDecoderWorkloadLoc11e60> {
    let mut workload = ThemeArtDecoderWorkloadLoc11e60 {
        streams: streams.len() as u32,
        output_bytes: 0,
        decoder_calls: 0,
        instruction_count: 0,
        raw_cycles: 0,
        wrapper_instruction_count: 0,
        wrapper_raw_cycles: 0,
    };
    for stream in streams {
        // ASM loc_11E60/loc_11EA2/loc_11EF2 caller BSR, DecompressToRAM's
        // sub_142CC setup and loc_143CC/loc_143CE permutation-table loops.
        workload.wrapper_instruction_count += 1 + 19 + 1_856;
        workload.wrapper_raw_cycles += 18 + 182 + 18_814;
        let mut resume = None;
        loop {
            let visit = crate::saxman_timing::visit_decompress_chunk_instructions_sub_13684(
                stream,
                resume,
                |_| {},
            )?;
            workload.decoder_calls += 1;
            workload.output_bytes += u32::from(visit.timing.output_bytes);
            workload.instruction_count += visit.timing.instruction_count;
            workload.raw_cycles += visit.timing.raw_cycles;
            let output_words = u32::from(visit.timing.output_bytes / 2);
            // ASM loc_143EA through loc_1440A: 11 setup instructions bracket
            // the decoder, then seven source instructions transfer each
            // output word. A resumed window copies 512 history longwords at
            // loc_14430; the final window takes return_14438.
            if visit.resume.is_some() {
                workload.wrapper_instruction_count += 1_041 + 7 * output_words;
                workload.wrapper_raw_cycles += 15_500 + 84 * output_words;
            } else {
                workload.wrapper_instruction_count += 14 + 7 * output_words;
                workload.wrapper_raw_cycles += 116 + 84 * output_words;
            }
            resume = visit.resume;
            if resume.is_none() {
                break;
            }
        }
    }
    Ok(workload)
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg(test)]
pub(crate) struct BlockOverlayWorkloadLoc11c7e {
    pub(crate) groups: u32,
    pub(crate) entries: u32,
    pub(crate) placement_cells: u32,
    pub(crate) written_cells: u32,
    pub(crate) property_words: u32,
    pub(crate) hidden_groups: u32,
    pub(crate) handler_groups: [u16; 17],
    pub(crate) fixed_handler_instruction_count: u32,
    pub(crate) fixed_handler_raw_cycles: u32,
    pub(crate) visible_prize_handler_instruction_count: u32,
    pub(crate) visible_prize_handler_raw_cycles: u32,
}

/// Census the `$FFFF`-terminated stream consumed by ASM `loc_11C7E` and
/// `AddrTbl_BlockTypes` before assigning instruction timing to each handler.
#[cfg(test)]
pub(crate) fn block_overlay_workload_loc_11c7e(
    words: &[u16],
) -> Option<BlockOverlayWorkloadLoc11c7e> {
    let mut workload = BlockOverlayWorkloadLoc11c7e {
        groups: 0,
        entries: 0,
        placement_cells: 0,
        written_cells: 0,
        property_words: 0,
        hidden_groups: 0,
        handler_groups: [0; 17],
        fixed_handler_instruction_count: 0,
        fixed_handler_raw_cycles: 0,
        visible_prize_handler_instruction_count: 0,
        visible_prize_handler_raw_cycles: 0,
    };
    let mut pos = 0_usize;
    let mut prize_handler_seen = false;
    loop {
        let type_word = *words.get(pos)?;
        pos += 1;
        if type_word == 0xffff {
            return Some(workload);
        }
        let hidden = type_word & 0x8000 != 0;
        let block_type = usize::from(type_word & 0x7fff);
        let handler_count = workload.handler_groups.get_mut(block_type)?;
        *handler_count = handler_count.checked_add(1)?;
        workload.groups = workload.groups.checked_add(1)?;
        workload.hidden_groups += u32::from(hidden);
        let fixed_handler = matches!(block_type, 0 | 2 | 5 | 7 | 9);
        let visible_prize_handler = block_type == 1 && !hidden;
        if fixed_handler {
            // ASM loc_122B8..loc_122BC: select word_122C6[block_type].
            workload.fixed_handler_instruction_count += 3;
            workload.fixed_handler_raw_cycles += 22;
        }
        if visible_prize_handler {
            // ASM loc_123DC..loc_123E2: initialise the prize-record base once.
            workload.visible_prize_handler_instruction_count +=
                if prize_handler_seen { 2 } else { 3 };
            workload.visible_prize_handler_raw_cycles += if prize_handler_seen { 26 } else { 40 };
        }
        if block_type == 1 {
            prize_handler_seen = true;
        }

        loop {
            let x = *words.get(pos)?;
            pos += 1;
            if x == 0xffff {
                if fixed_handler {
                    // ASM loc_122C0..loc_122C4: read the group terminator and
                    // return to loc_11C94.
                    workload.fixed_handler_instruction_count += 3;
                    workload.fixed_handler_raw_cycles += 32;
                }
                if visible_prize_handler {
                    // ASM loc_123E6..loc_123EA: read the group terminator and return.
                    workload.visible_prize_handler_instruction_count += 3;
                    workload.visible_prize_handler_raw_cycles += 32;
                }
                break;
            }
            workload.entries = workload.entries.checked_add(1)?;
            match block_type {
                0 | 2 | 5 | 6 | 7 | 8 | 9 | 13 | 14 | 15 => {
                    let count = u32::from(*words.get(pos + 1)?);
                    let stride = *words.get(pos + 2)? != 0;
                    pos = pos.checked_add(3)?;
                    workload.placement_cells = workload.placement_cells.checked_add(count)?;
                    let writes = if block_type == 8 {
                        count.checked_mul(4)?
                    } else {
                        count
                    };
                    workload.written_cells = workload.written_cells.checked_add(writes)?;
                    if fixed_handler {
                        // ASM loc_122C0..loc_122F4: X/Y/count/stride setup.
                        workload.fixed_handler_instruction_count += 13;
                        workload.fixed_handler_raw_cycles += if stride { 106 } else { 104 };
                        // Visible loc_122F6 or loc_12318 body, including DBF.
                        workload.fixed_handler_instruction_count += count.checked_mul(6)?;
                        let body_cycles = if stride { 56 } else { 48 };
                        workload.fixed_handler_raw_cycles +=
                            count.checked_mul(body_cycles)?.checked_add(4)?;
                        // ASM loc_12316/loc_1233A: return to the next X word.
                        workload.fixed_handler_instruction_count += 1;
                        workload.fixed_handler_raw_cycles += 10;
                    }
                }
                1 | 10 | 12 | 16 => {
                    let count = usize::from(*words.get(pos + 1)?);
                    let stride = *words.get(pos + 2)? != 0;
                    let properties = words.get(pos + 3..pos + 3 + count)?;
                    if visible_prize_handler {
                        // ASM loc_123EC..loc_124A6: consume one visible horizontal
                        // or vertical record through the nullsub_1 path.
                        workload.visible_prize_handler_instruction_count +=
                            14_u32.checked_add(u32::try_from(count).ok()?.checked_mul(20)?)?;
                        let negative_properties = u32::try_from(
                            properties
                                .iter()
                                .filter(|property| **property & 0x8000 != 0)
                                .count(),
                        )
                        .ok()?;
                        let count = u32::try_from(count).ok()?;
                        let entry_cycles = if stride { 122 } else { 118 };
                        let cell_cycles = if stride { 210 } else { 202 };
                        workload.visible_prize_handler_raw_cycles += entry_cycles
                            + count.checked_mul(cell_cycles)?
                            + negative_properties.checked_mul(2)?;
                    }
                    pos = pos.checked_add(3)?.checked_add(count)?;
                    let count = u32::try_from(count).ok()?;
                    workload.placement_cells = workload.placement_cells.checked_add(count)?;
                    workload.written_cells = workload.written_cells.checked_add(count)?;
                    workload.property_words = workload.property_words.checked_add(count)?;
                }
                3 => {
                    let count = u32::from(*words.get(pos + 1)?);
                    let priority = *words.get(pos + 4)?;
                    pos = pos.checked_add(7)?;
                    workload.placement_cells = workload.placement_cells.checked_add(count)?;
                    if priority == 0 {
                        workload.written_cells = workload.written_cells.checked_add(count)?;
                    }
                }
                4 => {
                    pos = pos.checked_add(3)?;
                    workload.placement_cells = workload.placement_cells.checked_add(1)?;
                    workload.written_cells = workload.written_cells.checked_add(2)?;
                }
                11 => {
                    pos = pos.checked_add(1)?;
                    workload.placement_cells = workload.placement_cells.checked_add(1)?;
                    workload.written_cells = workload.written_cells.checked_add(1)?;
                }
                _ => return None,
            }
        }
    }
}

/// Trace the compressed block-layout branches selected by the ROM bit stream.
///
/// ASM `LoadBlockLayout loc_132A6` uses `sub_13622` for every field and
/// `loc_13650` for byte refills. Retaining those calls is required because two
/// layouts with the same decoded words can cross different refill boundaries.
pub(crate) fn block_layout_decode_workload_loc_132a6(
    bytes: &[u8],
) -> Option<BlockLayoutDecodeWorkloadLoc132a6> {
    visit_block_layout_decode_instructions_loc_132a6(bytes, |_| {})
}

fn visit_block_layout_decode_instructions_loc_132a6(
    bytes: &[u8],
    mut visit: impl FnMut(u32),
) -> Option<BlockLayoutDecodeWorkloadLoc132a6> {
    if bytes.first() != Some(&0x40) {
        return None;
    }
    let mut padded = bytes.to_vec();
    padded.extend_from_slice(&[0; 8]);
    let mut reader = BlockLayoutBitReader::new(&padded, &mut visit)?;
    reader.read(8, &mut visit)?;
    let x_width = reader.read(4, &mut visit)? as u8;
    let y_width = reader.read(4, &mut visit)? as u8;
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
    macro_rules! emit_decoder {
        ($cycles:expr) => {{
            let cycles: &[u32] = $cycles;
            decoder_instruction_count += cycles.len() as u32;
            decoder_raw_cycles += cycles.iter().sum::<u32>();
            for &cycle in cycles {
                visit(cycle);
            }
        }};
    }
    // ASM loc_132A6: save 14 registers (a1 and a7 excluded), initialise the
    // reader, decode the two field widths and select the X sentinel mask.
    emit_decoder!(&[120, 4, 18, 12, 4, 18, 4, 18, 4, 4, 8, 14, 4, 18, 4]);

    loop {
        emit_decoder!(&[4, 18, 8, 4, 4, 18, 8]);
        reader.read(1, &mut visit)?;
        let block_type = reader.read(5, &mut visit)?;
        if block_type == 0x1f {
            emit_decoder!(&[10, 12, 12, 124, 16]);
            break;
        }
        emit_decoder!(&[12, 4, 8, 4, 14, 14]);
        groups += 1;
        loop {
            emit_decoder!(&[4, 18, 4]);
            let x = reader.read(x_width.max(1), &mut visit)?;
            if x == x_sentinel {
                emit_decoder!(&[8, 12, 10]);
                break;
            }
            emit_decoder!(&[10, 8, 4, 18, 8]);
            entries += 1;
            reader.read(y_width.max(1), &mut visit)?;
            match block_type {
                0 | 2 | 5 | 6 | 7 | 8 | 9 | 13 | 14 | 15 => {
                    emit_decoder!(&[4, 18, 4, 8, 4, 18, 8, 10]);
                    reader.read(4, &mut visit)?;
                    reader.read(1, &mut visit)?;
                }
                1 => {
                    emit_decoder!(&[4, 18, 4, 4, 8, 4, 18, 8]);
                    let count = reader.read(4, &mut visit)? + 1;
                    reader.read(1, &mut visit)?;
                    property_words += u32::from(count);
                    for index in 0..count {
                        emit_decoder!(&[4, 18, 4]);
                        let high_bit = reader.read(1, &mut visit)? != 0;
                        emit_decoder!(&[if high_bit { 8 } else { 10 }, 4, 18]);
                        reader.read(4, &mut visit)?;
                        if high_bit {
                            emit_decoder!(&[8, 10]);
                        }
                        emit_decoder!(&[8, if index + 1 == count { 14 } else { 10 }]);
                    }
                    emit_decoder!(&[10]);
                }
                10 | 12 => {
                    emit_decoder!(&[4, 18, 4, 4, 8, 4, 18, 8]);
                    let count = reader.read(4, &mut visit)? + 1;
                    reader.read(1, &mut visit)?;
                    property_words += u32::from(count);
                    for index in 0..count {
                        emit_decoder!(&[4, 18]);
                        reader.read(4, &mut visit)?;
                        emit_decoder!(&[8, if index + 1 == count { 14 } else { 10 }]);
                    }
                    emit_decoder!(&[10]);
                }
                3 => {
                    for (index, width) in [3, 1, 8, 3, 8, 8].into_iter().enumerate() {
                        emit_decoder!(&[4, 18]);
                        reader.read(width, &mut visit)?;
                        if index == 0 {
                            emit_decoder!(&[4, 8]);
                        } else {
                            emit_decoder!(&[8]);
                        }
                    }
                    emit_decoder!(&[10]);
                }
                4 => {
                    emit_decoder!(&[4, 18]);
                    reader.read(16, &mut visit)?;
                    emit_decoder!(&[4, 8, 4, 18]);
                    reader.read(9, &mut visit)?;
                    emit_decoder!(&[8, 10]);
                }
                11 => emit_decoder!(&[10]),
                16 => {
                    emit_decoder!(&[4, 18, 4, 4, 8, 4, 18, 8]);
                    let count = reader.read(4, &mut visit)? + 1;
                    reader.read(1, &mut visit)?;
                    property_words += u32::from(count);
                    for index in 0..count {
                        emit_decoder!(&[4, 18]);
                        reader.read(3, &mut visit)?;
                        emit_decoder!(&[8, if index + 1 == count { 14 } else { 10 }]);
                    }
                    emit_decoder!(&[10]);
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
    fn new(bytes: &'a [u8], visit: &mut impl FnMut(u32)) -> Option<Self> {
        let reader = Self {
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
        };
        for cycle in [8, 22, 8, 4, 16] {
            visit(cycle);
        }
        Some(reader)
    }

    fn read(&mut self, width: u8, visit: &mut impl FnMut(u32)) -> Option<u16> {
        if width == 0 || width > 16 {
            return None;
        }
        self.bit_read_calls += 1;
        self.bits_requested += u32::from(width);
        if width <= 8 {
            self.emit(&[8, 10, 18], visit);
            let value = self.read_short(width, visit)?;
            return Some(value);
        }
        self.emit(&[8, 8, 4, 18, 18], visit);
        let high = self.read_short(width - 8, visit)?;
        self.emit(&[22, 8, 4, 18, 18], visit);
        let low = self.read_short(8, visit)?;
        self.emit(&[8, 16], visit);
        Some((high << 8) | low)
    }

    fn read_short(&mut self, width: u8, visit: &mut impl FnMut(u32)) -> Option<u16> {
        if width > self.remaining {
            return None;
        }
        let shift = self.remaining - width;
        let mask = (1_u16 << width) - 1;
        let value = (self.word >> shift) & mask;
        // ASM sub_1363C, the immediate shift in sub_1364C, then loc_13650.
        self.emit(&[4, 4, 4, 6 + 2 * u32::from(shift), 4, 14, 16, 8], visit);
        self.remaining -= width;
        let rolling_refill = self.remaining < 9;
        self.emit(&[4, 8, if rolling_refill { 12 } else { 10 }], visit);
        if rolling_refill {
            self.remaining += 8;
            self.word = (self.word << 8) | u16::from(*self.bytes.get(self.position)?);
            self.position += 1;
            self.rolling_refills += 1;
            self.emit(&[8, 22, 8], visit);
        }
        self.emit(&[16], visit);
        Some(value)
    }

    fn emit(&mut self, cycles: &[u32], visit: &mut impl FnMut(u32)) {
        self.instruction_count += cycles.len() as u32;
        self.raw_cycles += cycles.iter().sum::<u32>();
        for &cycle in cycles {
            visit(cycle);
        }
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
            for cycles in [8, 14, 14, 8, 8, 4, 12] {
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
        assert_eq!(timing.raw_cycles, 374);
        assert_eq!(visited.len(), timing.instruction_count as usize);
        assert_eq!(visited.iter().copied().sum::<u32>(), timing.raw_cycles);
        assert_eq!(visited.iter().filter(|&&cycles| cycles == 8).count(), 15);
        assert_eq!(visited.iter().filter(|&&cycles| cycles == 14).count(), 8);
        assert_eq!(visited.iter().filter(|&&cycles| cycles == 10).count(), 3);
    }

    #[test]
    fn issue_300_foreground_cell_work_separates_both_teleporter_destinations() {
        let short = visit_foreground_collision_expansion_instructions_loc_11c34(20, 56, |_| {});
        let long = visit_foreground_collision_expansion_instructions_loc_11c34(260, 28, |_| {});

        assert_eq!(
            (short.cells, short.instruction_count, short.raw_cycles),
            (1_120, 9_186, 88_934)
        );
        assert_eq!(
            (long.cells, long.instruction_count, long.raw_cycles),
            (7_280, 58_354, 568_630)
        );
        assert_eq!(long.raw_cycles - short.raw_cycles, 479_696);
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
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct LevelLayoutPreludeWorkloadLoc11ae4 {
    pub(crate) instruction_count: u32,
    pub(crate) raw_cycles: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg(test)]
pub(crate) struct ForegroundPrefixAdvanceLoc11ae4 {
    pub(crate) prelude_cursor_loc_11c10: crate::vdp::LoadInGameCpuRasterCursor,
    pub(crate) decoder_cursor_loc_11c18: crate::vdp::LoadInGameCpuRasterCursor,
    pub(crate) collision_cursor_loc_11c30: crate::vdp::LoadInGameCpuRasterCursor,
    pub(crate) collision_exit_cursor_loc_11c52: crate::vdp::LoadInGameCpuRasterCursor,
    pub(crate) foreground_refresh_events: u32,
    pub(crate) block_post_guard_cursor_loc_11c7e: crate::vdp::LoadInGameCpuRasterCursor,
    pub(crate) cursor: crate::vdp::LoadInGameCpuRasterCursor,
    pub(crate) refresh_events: u32,
}

#[cfg(test)]
pub(crate) struct LoadLayoutScheduleInputsLoc11ae4<'a> {
    pub(crate) width_blocks: u16,
    pub(crate) height_blocks: u16,
    pub(crate) background_height: u16,
    pub(crate) murder_wall_flags: u8,
    pub(crate) layered_background: bool,
    pub(crate) foreground_stream: &'a [u8],
    pub(crate) block_layout_stream: &'a [u8],
}

/// Visit ASM `loc_11AE4..loc_11C0E` for a teleporter `Load_InGame`.
pub(crate) fn visit_level_layout_prelude_instructions_loc_11ae4(
    width_blocks: u16,
    height_blocks: u16,
    background_height: u16,
    murder_wall_flags: u8,
    layered_background: bool,
    mut visit: impl FnMut(u32),
) -> LevelLayoutPreludeWorkloadLoc11ae4 {
    let mut instruction_count = 0_u32;
    let mut raw_cycles = 0_u32;
    macro_rules! emit {
        ($cycles:expr) => {{
            let cycles = $cycles;
            instruction_count += 1;
            raw_cycles += cycles;
            visit(cycles);
        }};
    }
    macro_rules! emit_many {
        ($cycles:expr) => {{
            for cycles in $cycles {
                emit!(*cycles);
            }
        }};
    }

    // ASM loc_11AE4..loc_11AFE: select the small row-pointer buffer.
    emit_many!(&[12, 12, 8, 8, 8, 12, 4]);
    for index in 0..height_blocks {
        emit_many!(&[8, 16, if index + 1 == height_blocks { 14 } else { 10 }]);
    }
    emit_many!(&[12, 4, 8]);
    for index in 0..width_blocks {
        emit_many!(&[8, if index + 1 == width_blocks { 14 } else { 10 }]);
    }
    emit!(4);
    for index in 0..8 {
        emit_many!(&[8, if index == 7 { 14 } else { 10 }]);
    }
    emit_many!(&[12, 4, 4]);
    for index in 0..width_blocks {
        emit_many!(&[8, if index + 1 == width_blocks { 14 } else { 10 }]);
    }
    emit_many!(&[12, 12, 12, 4]);
    for index in 0..height_blocks {
        emit_many!(&[8, 16, if index + 1 == height_blocks { 14 } else { 10 }]);
    }
    emit_many!(&[12, 10, 8, 12, 12, 10, 8, 12, 12, 8, 12, 4, 12]);
    for index in 0..background_height {
        emit_many!(&[
            8,
            8,
            if index + 1 == background_height {
                14
            } else {
                10
            },
        ]);
    }

    // ASM loc_11B82..loc_11B9A: decode foreground Murderwall flags.
    emit_many!(&[4, 8]);
    if murder_wall_flags == 0 {
        emit!(10);
    } else {
        emit_many!(&[8, 10]);
        if murder_wall_flags & 1 != 0 {
            emit!(16);
        }
        emit_many!(&[10, 8, 16]);
    }
    // ASM loc_11B9A..loc_11BD2: themes and background-format override.
    emit_many!(&[12, 16, 16, 10, 4, 8, 4, 14, 8, 12, 8, 12, 8, 6, 16]);
    emit!(if layered_background { 8 } else { 10 });
    if layered_background {
        emit!(16);
    }
    // Teleport has FC29 set, so loc_11BD2 skips the header start coordinates.
    emit_many!(&[12, 12, 8, 16, 10]);
    emit_many!(&[16, 16, 12, 32, 4, 8, 12, 12, 4, 4, 4, 4]);

    LevelLayoutPreludeWorkloadLoc11ae4 {
        instruction_count,
        raw_cycles,
    }
}

/// Schedule ASM `loc_11AE4..loc_11C7E` without resetting raster state between
/// the level-layout prelude, foreground decoder, collision expansion and block
/// layout decoder.
#[cfg(test)]
pub(crate) fn schedule_foreground_prefix_loc_11ae4(
    entry: crate::vdp::LoadInGameCpuRasterCursor,
    inputs: &LoadLayoutScheduleInputsLoc11ae4<'_>,
) -> anyhow::Result<ForegroundPrefixAdvanceLoc11ae4> {
    let mut scheduler = crate::vdp::LoadInGameRasterScheduler::new(entry);
    let mut schedule_ok = true;

    visit_level_layout_prelude_instructions_loc_11ae4(
        inputs.width_blocks,
        inputs.height_blocks,
        inputs.background_height,
        inputs.murder_wall_flags,
        inputs.layered_background,
        |cycles| {
            if schedule_ok {
                schedule_ok = scheduler.advance_instruction(cycles).is_some();
            }
        },
    );
    let prelude_cursor_loc_11c10 = scheduler.cursor();
    visit_foreground_terrain_decoder_instructions_loc_11c10(inputs.foreground_stream, |cycles| {
        if schedule_ok {
            schedule_ok = scheduler.advance_instruction(cycles).is_some();
        }
    })?;
    let decoder_cursor_loc_11c18 = scheduler.cursor();
    // ASM loc_11C18..loc_11C2C: restore registers and select the theme's
    // collision lookup before loc_11C30 seeds the expansion counters.
    for cycles in [36, 12, 8, 20, 18] {
        if schedule_ok {
            schedule_ok = scheduler.advance_instruction(cycles).is_some();
        }
    }
    let collision_cursor_loc_11c30 = scheduler.cursor();
    visit_foreground_collision_expansion_instructions_loc_11c34(
        inputs.width_blocks,
        inputs.height_blocks,
        |cycles| {
            if schedule_ok {
                schedule_ok = scheduler.advance_instruction(cycles).is_some();
            }
        },
    );
    let collision_exit_cursor_loc_11c52 = scheduler.cursor();
    let foreground_refresh_events = scheduler.refresh_events();
    // ASM loc_11C52..loc_11C6A: seed the temporary block-list buffer, fetch
    // the compressed layout and call LoadBlockLayout. LoadBlockLayout
    // loc_1329E verifies the $40 marker before entering loc_132A6.
    for cycles in [24, 20, 12, 12, 18, 12, 10] {
        if schedule_ok {
            schedule_ok = scheduler.advance_instruction(cycles).is_some();
        }
    }
    visit_block_layout_decode_instructions_loc_132a6(inputs.block_layout_stream, |cycles| {
        if schedule_ok {
            schedule_ok = scheduler.advance_instruction(cycles).is_some();
        }
    })
    .ok_or_else(|| anyhow::anyhow!("invalid LoadBlockLayout stream"))?;
    // ASM loc_11C6E..loc_11C74: the Forced Entry guard follows the decoder
    // return. Moving through both instructions services any refresh slot
    // pending at the return-PC boundary.
    for cycles in [16, 10] {
        if schedule_ok {
            schedule_ok = scheduler.advance_instruction(cycles).is_some();
        }
    }
    let block_post_guard_cursor_loc_11c7e = scheduler.cursor();

    anyhow::ensure!(schedule_ok, "Load_InGame layout schedule overflow");
    Ok(ForegroundPrefixAdvanceLoc11ae4 {
        prelude_cursor_loc_11c10,
        decoder_cursor_loc_11c18,
        collision_cursor_loc_11c30,
        collision_exit_cursor_loc_11c52,
        foreground_refresh_events,
        block_post_guard_cursor_loc_11c7e,
        cursor: scheduler.cursor(),
        refresh_events: scheduler.refresh_events(),
    })
}
