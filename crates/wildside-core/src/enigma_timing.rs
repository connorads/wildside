use anyhow::Result;
use wildside_data::{
    EnigmaCommandTrace, EnigmaInlinePacketPath, EnigmaInlineValueTrace, EnigmaTrace, enigma_trace,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EnigmaTiming {
    pub instruction_count: u32,
    pub raw_cycles: u32,
    pub output_words: u16,
}

fn emit(cycles: &[u32], visit: &mut impl FnMut(u32), timing: &mut EnigmaTiming) {
    for &cycles in cycles {
        visit(cycles);
        timing.instruction_count += 1;
        timing.raw_cycles += cycles;
    }
}

fn emit_consume_bits(rolling_refill: bool, visit: &mut impl FnMut(u32), timing: &mut EnigmaTiming) {
    // ASM loc_13650 (kid.asm:26109-26121): consume d0 bits, refill one byte
    // when fewer than nine buffered bits remain, then return to the caller.
    emit(&[4, 8, if rolling_refill { 12 } else { 10 }], visit, timing);
    if rolling_refill {
        emit(&[8, 22, 8], visit, timing);
    }
    emit(&[16], visit, timing);
}

fn emit_inline_value(
    inline: EnigmaInlineValueTrace,
    visit: &mut impl FnMut(u32),
    timing: &mut EnigmaTiming,
) {
    // ASM sub_13588/loc_1359A/loc_135A8/loc_135B6/loc_135C4
    // (kid.asm:25973-26016): each enabled high tile-property bit consumes one
    // source bit; disabled properties take the short BCC path.
    emit(&[4, 4], visit, timing);
    for (mask, output_flag) in [
        (0x10, 0x8000),
        (0x08, 0x4000),
        (0x04, 0x2000),
        (0x02, 0x1000),
        (0x01, 0x0800),
    ] {
        emit(&[4], visit, timing);
        if inline.enabled_flag_mask & mask == 0 {
            emit(&[10], visit, timing);
            continue;
        }
        emit(&[8, 4, 6], visit, timing);
        if inline.set_flag_word & output_flag == 0 {
            emit(&[10], visit, timing);
        } else {
            emit(&[8, 8], visit, timing);
        }
    }

    // ASM loc_135D2: select the packet path using d7=d6-a5.
    emit(&[4, 4, 4], visit, timing);
    match inline.path {
        EnigmaInlinePacketPath::CrossesWordBoundary => {
            let missing_bits = u32::from(inline.packet_length)
                .checked_sub(u32::from(inline.bits_before_packet))
                .expect("crossing packet has missing bits");
            emit(
                &[
                    8,
                    4,
                    8,
                    4,
                    6 + 2 * missing_bits,
                    8,
                    6 + 2 * missing_bits,
                    4,
                    14,
                    4,
                ],
                visit,
                timing,
            );
            emit(&[4, 4, 14, 4, 8, 22, 8, 16], visit, timing);
        }
        EnigmaInlinePacketPath::ExactWordBoundary => {
            emit(&[10, 4, 10], visit, timing);
            emit(&[4, 4, 14, 4, 8, 22, 8, 16], visit, timing);
        }
        EnigmaInlinePacketPath::WithinBufferedWord { rolling_refill } => {
            let shift = u32::from(inline.bits_before_packet - u16::from(inline.packet_length));
            emit(&[10, 8, 6 + 2 * shift, 4, 4, 14, 4, 4, 10], visit, timing);
            emit_consume_bits(rolling_refill, visit, timing);
        }
    }
}

fn emit_command(
    command: &EnigmaCommandTrace,
    visit: &mut impl FnMut(u32),
    timing: &mut EnigmaTiming,
) {
    // ASM loc_134EA/loc_13504 (kid.asm:25850-25875): inspect seven bits,
    // reduce prefixes below $40 to six bits, consume through loc_13650, and
    // dispatch via the eight-entry jump table at loc_13560.
    let prefix_shift = u32::from(command.bits_before_prefix - 7);
    emit(&[4, 4, 4, 4, 6 + 2 * prefix_shift, 8, 4, 8], visit, timing);
    if command.prefix_bits == 6 {
        emit(&[8, 4, 8], visit, timing);
    } else {
        emit(&[10], visit, timing);
    }
    emit(&[18], visit, timing);
    emit_consume_bits(command.prefix_rolling_refill, visit, timing);
    emit(&[8, 14, 4, 14], visit, timing);

    let iterations = u16::from(command.dbf_counter) + 1;
    match command.command {
        0 | 1 => {
            for iteration in 0..iterations {
                emit(
                    &[8, 8, if iteration + 1 == iterations { 14 } else { 10 }],
                    visit,
                    timing,
                );
            }
            timing.output_words += iterations;
            emit(&[10], visit, timing);
        }
        2 | 3 => {
            for iteration in 0..iterations {
                emit(
                    &[8, if iteration + 1 == iterations { 14 } else { 10 }],
                    visit,
                    timing,
                );
            }
            timing.output_words += iterations;
            emit(&[10], visit, timing);
        }
        4..=6 => {
            emit(&[18], visit, timing);
            emit_inline_value(command.inline_values[0], visit, timing);
            for iteration in 0..iterations {
                emit(&[8], visit, timing);
                if command.command != 4 {
                    emit(&[4], visit, timing);
                }
                emit(
                    &[if iteration + 1 == iterations { 14 } else { 10 }],
                    visit,
                    timing,
                );
            }
            timing.output_words += iterations;
            emit(&[10], visit, timing);
        }
        7 if command.dbf_counter == 0x0F => {
            emit(&[8, 10], visit, timing);
        }
        7 => {
            emit(&[8, 8], visit, timing);
            for (iteration, inline) in command.inline_values.iter().copied().enumerate() {
                emit(&[18], visit, timing);
                emit_inline_value(inline, visit, timing);
                emit(
                    &[
                        8,
                        if iteration + 1 == command.inline_values.len() {
                            14
                        } else {
                            10
                        },
                    ],
                    visit,
                    timing,
                );
            }
            timing.output_words += iterations;
            emit(&[10], visit, timing);
        }
        _ => unreachable!("Enigma command is three bits"),
    }
}

fn emit_return(
    trace: &EnigmaTrace,
    source_address: u32,
    visit: &mut impl FnMut(u32),
    timing: &mut EnigmaTiming,
) {
    // ASM loc_13570/loc_1357A/loc_13582 (kid.asm:25935-25947): rewind the
    // speculative input byte(s), align a0 to an even ROM address, restore the
    // thirteen saved registers, and return.
    emit(&[8, 8], visit, timing);
    // Only the address parity reaches loc_1357A. Addition and subtraction have
    // identical parity, so model the branch directly instead of retaining an
    // otherwise unobservable full speculative address.
    let mut return_address_is_even =
        source_address.is_multiple_of(2) != trace.terminator_next_input_offset.is_multiple_of(2);
    if trace.terminator_bits_remaining == 0x10 {
        emit(&[8, 8], visit, timing);
        return_address_is_even = !return_address_is_even;
    } else {
        emit(&[10], visit, timing);
    }
    emit(&[4, 8], visit, timing);
    if return_address_is_even {
        emit(&[10], visit, timing);
    } else {
        emit(&[8, 8], visit, timing);
    }
    emit(&[116, 16], visit, timing);
}

/// Visit every source instruction selected by ASM `EniDec`.
///
/// `source_address` is the ROM address of the Enigma header. It determines
/// the final even-address alignment branch at `loc_1357A`.
pub fn visit_enigma_instructions(
    data: &[u8],
    base_tile_word: u16,
    source_address: u32,
    mut visit: impl FnMut(u32),
) -> Result<EnigmaTiming> {
    let trace = enigma_trace(data, base_tile_word)?;
    let mut timing = EnigmaTiming {
        instruction_count: 0,
        raw_cycles: 0,
        output_words: 0,
    };

    // ASM EniDec (kid.asm:25835-25849): save registers and parse the six-byte
    // header before loc_134EA starts consuming the rolling command buffer.
    emit(
        &[112, 4, 8, 4, 4, 8, 12, 8, 8, 8, 8, 8, 22, 8, 4],
        &mut visit,
        &mut timing,
    );
    for command in &trace.commands {
        emit_command(command, &mut visit, &mut timing);
    }
    emit_return(&trace, source_address, &mut visit, &mut timing);

    Ok(timing)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compact_intro_maps_cover_inline_packet_and_return_paths() {
        // ASM unk_2E77A, unk_2FDA6, and unk_2F840 are compact checked-in Enigma
        // streams. Together they exercise loc_1359A..loc_135D2 mixed property
        // flags, one- and two-bit crossing reads, exact packet reads, command-7
        // inline lists, and loc_13570's `$10` terminator rewind. Keeping the
        // bytes inline avoids a cache-only dependency while pinning every
        // timing branch to ROM data.
        const MAP_2E77A: &[u8] = &[
            0x09, 0x03, 0x00, 0x02, 0x00, 0x00, 0x46, 0x01, 0x15, 0x01, 0xC0, 0x1B, 0xB0, 0x45,
            0xB8, 0x00, 0xDA, 0xE4, 0x7F, 0x80, 0xA5, 0xAE, 0x49, 0xE8, 0x81, 0xFB, 0xE4, 0xBE,
            0x89, 0x3F, 0xE1, 0x87, 0xEB, 0x94, 0x14, 0x39, 0x82, 0x02, 0x92, 0x52, 0x80, 0x05,
            0xE2, 0x97, 0xC0, 0x26, 0x57, 0x2B, 0x48, 0x70, 0x0A, 0xB7, 0xCB, 0x15, 0xA2, 0xE4,
            0x8D, 0xC2, 0xFB, 0xC0, 0x3F, 0x5C, 0x31, 0x48, 0xF1, 0x11, 0x86, 0x5E, 0xA8, 0x8E,
            0xE3, 0x00, 0xC4, 0xA3, 0xE8, 0xFF,
        ];
        const MAP_2FDA6: &[u8] = &[
            0x0A, 0x02, 0x00, 0x34, 0x00, 0x00, 0x4E, 0xAA, 0x70, 0x47, 0x85, 0x34, 0xAC, 0x9D,
            0x90, 0xB6, 0x9F, 0x60, 0x12, 0x50, 0x28, 0xA8, 0x94, 0xAB, 0xA8, 0xB4, 0x6C, 0x29,
            0x34, 0x6B, 0xA9, 0xC4, 0xEA, 0x2A, 0x45, 0x28, 0xAA, 0x95, 0x3F, 0x80,
        ];
        const MAP_2F840: &[u8] = &[
            0x04, 0x02, 0x00, 0x00, 0x00, 0x07, 0x02, 0x10, 0x00, 0x80, 0x00, 0x20, 0x00, 0x0E,
            0x20, 0x98, 0x10, 0x26, 0x13, 0x20, 0xFF,
        ];

        for (data, base_tile_word, source_address, expected) in [
            (
                MAP_2E77A,
                0,
                0x2E77A,
                EnigmaTiming {
                    instruction_count: 2_481,
                    raw_cycles: 20_678,
                    output_words: 203,
                },
            ),
            (
                MAP_2FDA6,
                0,
                0x2FDA6,
                EnigmaTiming {
                    instruction_count: 1_138,
                    raw_cycles: 9_912,
                    output_words: 90,
                },
            ),
            (
                MAP_2F840,
                0,
                0x2F840,
                EnigmaTiming {
                    instruction_count: 615,
                    raw_cycles: 5_602,
                    output_words: 28,
                },
            ),
        ] {
            let timing = visit_enigma_instructions(data, base_tile_word, source_address, |_| {})
                .expect("valid ROM Enigma stream");
            assert_eq!(
                timing, expected,
                "ASM EniDec timing for {source_address:#x}"
            );
        }
    }

    #[test]
    fn repeated_intro_rows_follow_eni_dec_source_instruction_path() {
        // ASM unk_2FDCE is the compact Enigma stream consumed by sub_1B850's
        // second EniDec call. Keep the 18-byte fixture cache-independent.
        const REPEATED_INTRO_ROWS: &[u8] = &[
            0x03, 0x00, 0x00, 0x00, 0x00, 0x04, 0x02, 0x00, 0x02, 0x01, 0x06, 0x0B, 0x01, 0x78,
            0x2C, 0x08, 0xF7, 0xF8,
        ];
        let mut cycles = Vec::new();

        let timing = visit_enigma_instructions(REPEATED_INTRO_ROWS, 0x413A, 0x2FDCE, |raw| {
            cycles.push(raw);
        })
        .expect("valid Enigma command stream");

        assert_eq!(
            timing,
            EnigmaTiming {
                instruction_count: 504,
                raw_cycles: 4_672,
                output_words: 28,
            },
        );
        assert_eq!(cycles.iter().sum::<u32>(), timing.raw_cycles);
    }
}
