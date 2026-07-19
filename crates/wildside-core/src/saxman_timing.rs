use anyhow::Result;
use wildside_data::{SaxmanTokenKind, saxman_tokens};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DecompressChunkTimingSub13684 {
    pub instruction_count: u32,
    pub raw_cycles: u32,
    pub output_bytes: u16,
}

// ASM sub_142CC loads Decompression_Buffer ($FFFF77B2), while
// DecompressToVRAM sets a3=-1. Decompress_Chunk compares the low words after
// each reference, so the initial call can emit this many bytes before it
// returns to preserve the final $800-byte history window.
const INITIAL_WINDOW_BYTES: u16 = 0xFFFF - 0x77B2;

fn emit(cycles: &[u32], visit: &mut impl FnMut(u32), timing: &mut DecompressChunkTimingSub13684) {
    for &cycles in cycles {
        visit(cycles);
        timing.instruction_count += 1;
        timing.raw_cycles += cycles;
    }
}

fn emit_mode_dispatch(
    lane: usize,
    kind: SaxmanTokenKind,
    visit: &mut impl FnMut(u32),
    timing: &mut DecompressChunkTimingSub13684,
) {
    // ASM Decompress_BitPos0/loc_13736/loc_137B0/loc_1382E/loc_138AA/
    // loc_13928/loc_139A6/loc_13A24: the unrolled decoder consumes one
    // control byte over eight source-bit lanes. Lane 0 loads that byte before
    // its first test; lane 7 reloads after its first non-literal bit.
    if lane == 0 {
        emit(&[8], visit, timing);
    }
    emit(&[4], visit, timing);
    if matches!(kind, SaxmanTokenKind::Literal { .. }) {
        emit(&[10, 12], visit, timing);
        return;
    }
    emit(&[8], visit, timing);
    if lane == 7 {
        emit(&[8], visit, timing);
    }
    emit(&[4, 4], visit, timing);
    if matches!(
        kind,
        SaxmanTokenKind::LongReference { .. }
            | SaxmanTokenKind::Flush { .. }
            | SaxmanTokenKind::End
    ) {
        emit(&[10], visit, timing);
        return;
    }
    emit(&[8], visit, timing);
    if lane == 6 {
        emit(&[8], visit, timing);
    }
}

fn emit_short_reference(
    count: u16,
    finished: bool,
    visit: &mut impl FnMut(u32),
    timing: &mut DecompressChunkTimingSub13684,
) {
    // ASM loc_136C6/loc_13740/loc_137BA/loc_13838/loc_138B4/loc_13932/
    // loc_139B0/loc_13A30: load the one-byte distance, optionally copy a
    // third byte, then execute the shared two-byte tail.
    emit(&[8, 8, 4], visit, timing);
    if count == 3 {
        emit(&[8, 12], visit, timing);
    } else {
        emit(&[10], visit, timing);
    }
    emit(&[12, 12, 4], visit, timing);
    emit_decode_boundary(finished, visit, timing);
}

fn emit_long_header(
    lane: usize,
    visit: &mut impl FnMut(u32),
    timing: &mut DecompressChunkTimingSub13684,
) {
    // ASM Decompress_BP0_LongRef/loc_13756/loc_137D0/loc_1384E/
    // loc_138CA/loc_13948/loc_139C8/loc_13A46. The instruction ordering
    // differs only where the next control byte is pulled into d1.
    match lane {
        0 | 1 | 2 | 7 => emit(&[12, 4, 4, 8, 8], visit, timing),
        3 => emit(&[12, 8, 4, 4, 8, 8], visit, timing),
        4 => emit(&[10, 8, 4, 4, 4, 8, 8], visit, timing),
        5 => emit(&[4, 8, 10, 4, 4, 8, 8], visit, timing),
        6 => emit(&[8, 12, 4, 4, 8, 8], visit, timing),
        _ => unreachable!("Saxman bit lane is modulo eight"),
    }
}

fn emit_long_count_bits(
    lane: usize,
    count_code: u8,
    visit: &mut impl FnMut(u32),
    timing: &mut DecompressChunkTimingSub13684,
) {
    let first_set = count_code & 0b10 != 0;
    emit(&[4, if first_set { 10 } else { 8 }], visit, timing);
    if lane == 2 {
        emit(&[8], visit, timing);
    }
    let second_set = count_code & 0b01 != 0;
    let second_branch_taken = if first_set { !second_set } else { second_set };
    emit(
        &[4, if second_branch_taken { 10 } else { 8 }],
        visit,
        timing,
    );
    if count_code == 0 {
        emit(&[10], visit, timing);
    }
}

fn emit_decode_boundary(
    finished: bool,
    visit: &mut impl FnMut(u32),
    timing: &mut DecompressChunkTimingSub13684,
) {
    if finished {
        // ASM loc_13724/loc_1372C and their six lane-equivalent returns.
        emit(&[10, 8, 4, 16], visit, timing);
    } else {
        emit(&[8, 10], visit, timing);
    }
}

fn emit_long_reference(
    lane: usize,
    kind: SaxmanTokenKind,
    finished: bool,
    visit: &mut impl FnMut(u32),
    timing: &mut DecompressChunkTimingSub13684,
) -> bool {
    emit_long_header(lane, visit, timing);
    let count = match kind {
        SaxmanTokenKind::LongReference { count, .. } => count,
        SaxmanTokenKind::Flush { count } => u16::from(count),
        SaxmanTokenKind::End => 0,
        _ => unreachable!("caller only passes long forms"),
    };
    let count_code = match kind {
        SaxmanTokenKind::LongReference { count: 3, .. } => 0,
        SaxmanTokenKind::LongReference { count: 4, .. } => 1,
        SaxmanTokenKind::LongReference { count: 5, .. } => 2,
        _ => 3,
    };
    emit_long_count_bits(lane, count_code, visit, timing);

    if count_code == 3 {
        emit(&[4, 8], visit, timing);
        if count == 0 {
            // ASM loc_13790/loc_1380E/loc_1388A/loc_13908/loc_13986/
            // loc_13A04/loc_13A80: extended zero terminates the stream.
            emit(&[10, 8, 16], visit, timing);
            return true;
        }
        emit(&[8, 4], visit, timing);
        if count < 6 {
            // ASM loc_13796 and lane-equivalent flush returns.
            emit(&[10, 8, 4, 16], visit, timing);
            return true;
        }
        emit(&[8], visit, timing);
        let loop_iterations = count - 5;
        for iteration in 0..loop_iterations {
            emit(
                &[
                    12,
                    if iteration + 1 == loop_iterations {
                        14
                    } else {
                        10
                    },
                ],
                visit,
                timing,
            );
        }
        emit(&[12, 12, 12, 12, 12, 4], visit, timing);
    } else {
        for _ in 0..count {
            emit(&[12], visit, timing);
        }
        emit(&[4], visit, timing);
    }
    emit_decode_boundary(finished, visit, timing);
    false
}

/// Visit the source instruction lengths for the initial ASM
/// `Decompress_Chunk sub_13684` call.
///
/// This follows the Saxman control stream rather than treating the decoder as
/// a fixed elapsed span, so colocated hardware can insert waits at the same
/// instruction boundaries as the ROM. The initial call starts with `d2=0` and
/// the `$77B2..$FFFF` initial output window used by
/// `DecompressToVRAM loc_14300`.
pub fn visit_initial_decompress_chunk_instructions_sub_13684(
    data: &[u8],
    mut visit: impl FnMut(u32),
) -> Result<DecompressChunkTimingSub13684> {
    let tokens = saxman_tokens(data)?;
    let mut timing = DecompressChunkTimingSub13684 {
        instruction_count: 0,
        raw_cycles: 0,
        output_bytes: 0,
    };

    // ASM Decompress_Chunk sub_13684: initialise d0/d4-d7, then reject all
    // seven saved-lane resume tags before entering Decompress_BitPos0.
    emit(
        &[
            4, 8, 4, 4, 4, 4, 12, 4, 12, 4, 12, 4, 12, 4, 12, 4, 12, 4, 12,
        ],
        &mut visit,
        &mut timing,
    );

    for token in tokens {
        let lane = token.bit_position % 8;
        emit_mode_dispatch(lane, token.kind, &mut visit, &mut timing);
        match token.kind {
            SaxmanTokenKind::Literal { .. } => {
                timing.output_bytes += 1;
            }
            SaxmanTokenKind::ShortReference { count, .. } => {
                timing.output_bytes = timing
                    .output_bytes
                    .checked_add(count)
                    .expect("one decoder window fits u16");
                let finished = timing.output_bytes >= INITIAL_WINDOW_BYTES;
                emit_short_reference(count, finished, &mut visit, &mut timing);
                if finished {
                    break;
                }
            }
            kind @ SaxmanTokenKind::LongReference { count, .. } => {
                timing.output_bytes = timing
                    .output_bytes
                    .checked_add(count)
                    .expect("one decoder window fits u16");
                let finished = timing.output_bytes >= INITIAL_WINDOW_BYTES;
                if emit_long_reference(lane, kind, finished, &mut visit, &mut timing) || finished {
                    break;
                }
            }
            kind @ SaxmanTokenKind::Flush { .. } => {
                emit_long_reference(lane, kind, false, &mut visit, &mut timing);
                break;
            }
            SaxmanTokenKind::End => {
                emit_long_reference(lane, SaxmanTokenKind::End, false, &mut visit, &mut timing);
                break;
            }
        }
    }

    Ok(timing)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wild_side_intro_decoder_matches_sub_13684_instruction_workload() {
        // ASM sub_1B850 -> byte_2D71A -> DecompressToVRAM loc_14300.
        const WILD_SIDE_BACKGROUND: &[u8] = &[
            0x00, 0x09, 0xD1, 0xCA, 0x34, 0x72, 0x8D, 0x1A, 0x34, 0x68, 0xC0, 0x33, 0x33, 0x01,
            0x1E, 0x22, 0x01, 0x08, 0x28, 0x01, 0x0C, 0x11, 0x01, 0x08, 0x28, 0x01, 0x1C, 0x58,
            0x10, 0x01, 0x10, 0x00, 0x00,
        ];
        let mut cycles = Vec::new();

        let timing =
            visit_initial_decompress_chunk_instructions_sub_13684(WILD_SIDE_BACKGROUND, |raw| {
                cycles.push(raw);
            })
            .expect("valid Wild Side background stream");

        assert_eq!(timing.output_bytes, 192);
        assert_eq!(timing.instruction_count, 576);
        assert_eq!(timing.raw_cycles, 5_390);
        assert_eq!(cycles.iter().sum::<u32>(), timing.raw_cycles);
        let mut digest = crc32fast::Hasher::new();
        digest.update(
            &cycles
                .iter()
                .map(|&raw| u8::try_from(raw).expect("68000 instruction fits one byte"))
                .collect::<Vec<_>>(),
        );
        assert_eq!(
            digest.finalize(),
            0xD2BB_29AE,
            "full selected-PC oracle trace from sub_13684 entry to RTS",
        );
    }

    #[test]
    fn initial_sub_13684_call_does_not_flush_at_256_output_bytes() {
        // ASM sub_142CC/DecompressToVRAM loc_14300 starts a2 at $FFFF77B2
        // and a3 at $FFFFFFFF. The low-word comparison in loc_13724 and its
        // lane equivalents therefore does not return at 256 bytes.
        let mut control_bits = vec![1_u8; 300];
        control_bits.extend([0, 1, 0, 0, 0, 1, 1]);
        let mut control_bytes = vec![0_u8; control_bits.len().div_ceil(8)];
        for (index, bit) in control_bits.into_iter().enumerate() {
            control_bytes[index / 8] |= bit << (7 - index % 8);
        }
        let mut compressed = Vec::with_capacity(2 + control_bytes.len() + 302);
        compressed.extend_from_slice(&(control_bytes.len() as u16).to_be_bytes());
        compressed.extend_from_slice(&control_bytes);
        compressed.extend((0_u8..=255).cycle().take(300));
        compressed.extend([0, 0]);

        let timing = visit_initial_decompress_chunk_instructions_sub_13684(&compressed, |_| {})
            .expect("300-byte stream remains inside the initial WRAM window");

        assert_eq!(timing.output_bytes, 300);
    }
}
