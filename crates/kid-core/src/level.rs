use super::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct FgUpdateQueueCustomEntrySub1596 {
    pub(crate) vram_word_address: u16,
    pub(crate) first_longword: u32,
    pub(crate) second_longword: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct FgPlaneCustomWriteSub1596 {
    pub(crate) control_words: [u16; 2],
    pub(crate) data_longword: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct HiddenBlockCopySetupLoc11C96 {
    pub(crate) dbf_counter: u16,
    pub(crate) copied_list_addr: u32,
    pub(crate) temp_list_addr: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct BlockLayoutEntryLoc122C0 {
    pub(crate) x_pos: u16,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct BlockLayoutSetupLoc122DA {
    pub(crate) y_row: u16,
    pub(crate) count: u16,
    pub(crate) stride: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct BlockPlaceResultLoc122F6 {
    pub(crate) cell_addr: u32,
    pub(crate) cell_word: u16,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct BlockLayoutDispatchLoc132D6 {
    pub(crate) header_word: u16,
    pub(crate) terminated: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct BlockLayoutDispatchLoc13504 {
    pub(crate) property_word_dbf_counter: u16,
    pub(crate) jump_table_word_offset: u16,
    pub(crate) bits_remaining: u16,
    pub(crate) refilled_input_byte: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct BlockLayoutDecodeHighBitLoc1359A {
    pub(crate) decoded_word: u16,
    pub(crate) shifted_property_byte: u8,
    pub(crate) bits_remaining: u16,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct BlockLayoutDecodeBitLoc135A8 {
    pub(crate) decoded_word: u16,
    pub(crate) shifted_property_byte: u8,
    pub(crate) bits_remaining: u16,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct BlockLayoutDecodeBitLoc135B6 {
    pub(crate) decoded_word: u16,
    pub(crate) shifted_property_byte: u8,
    pub(crate) bits_remaining: u16,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct BlockLayoutDecodeBitLoc135C4 {
    pub(crate) decoded_word: u16,
    pub(crate) shifted_property_byte: u8,
    pub(crate) bits_remaining: u16,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct BlockLayoutFinalizeWordLoc135F0 {
    pub(crate) decoded_word: u16,
    pub(crate) bit_buffer: u16,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct DecompressShortRefTailLoc136D0 {
    pub(crate) copied_bytes: [u8; 2],
    pub(crate) finished: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct DecompressBp0ReturnStatus {
    pub(crate) status_word: u16,
    pub(crate) next_copy_mode: Option<u16>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct DecompressDirectCopyLoc137AE {
    pub(crate) copied_byte: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum DecompressModeDispatchLoc13736 {
    DirectCopy,
    LongRef {
        output_copy_base_addr: u16,
    },
    ShortRef {
        source_copy_addr: u16,
        total_copy_len: u8,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum DecompressModeDispatchLoc137B0 {
    DirectCopy,
    LongRef {
        output_copy_base_addr: u16,
    },
    ShortRef {
        source_copy_addr: u16,
        total_copy_len: u8,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum DecompressModeDispatchLoc1382E {
    DirectCopy,
    LongRef {
        output_copy_base_addr: u16,
    },
    ShortRef {
        source_copy_addr: u16,
        total_copy_len: u8,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum DecompressModeDispatchLoc138AA {
    DirectCopy {
        copied_byte: u8,
    },
    LongRef {
        output_copy_base_addr: u16,
    },
    ShortRef {
        source_copy_addr: u16,
        total_copy_len: u8,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum DecompressModeDispatchLoc13928 {
    DirectCopy,
    LongRef {
        output_copy_base_addr: u16,
    },
    ShortRef {
        source_copy_addr: u16,
        total_copy_len: u8,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum DecompressLongRefCopyModeLoc13756 {
    Copy3,
    Copy4,
    Copy5,
    VariableLength,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct DecompressLongRefHeaderLoc13756 {
    pub(crate) source_back_distance: u16,
    pub(crate) copy_mode: DecompressLongRefCopyModeLoc13756,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum DecompressLongRefExtendedLoc1376A {
    Copy5,
    QuitDecompression,
    FlushBuffer,
    VariableLength {
        total_copy_len: u8,
        dbf_counter: u16,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct DecompressLongRefLoopLoc13778 {
    pub(crate) copied_bytes: Vec<u8>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct DecompressLongRefTailLoc1377E {
    pub(crate) copied_bytes: [u8; 5],
    pub(crate) finished: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct DecompressLongRefTailLoc1387A {
    pub(crate) copied_bytes: [u8; 4],
    pub(crate) finished: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct DecompressShortRefTailLoc137C4 {
    pub(crate) copied_bytes: [u8; 2],
    pub(crate) finished: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum BlockLayoutLoopLoc1331E {
    RestartAtLoc132D6 { end_marker: u16 },
    ContinueLoc1332E,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum BlockLayoutLoopLoc133A0 {
    RestartAtLoc132D6 { end_marker: u16 },
    ContinueLoc133B2,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum BlockLayoutLoopLoc133DE {
    RestartAtLoc132D6 { end_marker: u16 },
    ContinueLoc133F0,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum BlockLayoutLoopLoc1342E {
    RestartAtLoc132D6 { end_marker: u16 },
    ContinueLoc13440,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum BlockLayoutLoopLoc1345E {
    RestartAtLoc132D6 { end_marker: u16 },
    ContinueLoc13470,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum BlockLayoutLoopLoc1347C {
    RestartAtLoc132D6 { end_marker: u16 },
    ContinueLoc1348E,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct BlockLayoutEntryWordsLoc1332E {
    pub(crate) x_pos: u16,
    pub(crate) y_row: u16,
    pub(crate) count: u16,
    pub(crate) stride_word: u16,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct BlockLayoutEntryWordsLoc13470 {
    pub(crate) x_pos: u16,
    pub(crate) y_row: u16,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct BlockLayoutRepeatWordsLoc13514 {
    pub(crate) words: Vec<u16>,
    pub(crate) next_word: u16,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct BlockLayoutRepeatWordsLoc1351E {
    pub(crate) words: Vec<u16>,
    pub(crate) next_word: u16,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum BlockLayoutDecodeWordsLoc1354E {
    ReturnViaLoc13570 { rewound_input_addr: u16 },
    ContinueLoc13554 { words: Vec<u16> },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct BlockLayoutTerminateLoc134BA {
    pub(crate) end_marker: u16,
    pub(crate) output_addr: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct BlockLayoutEntryWordsLoc133F0 {
    pub(crate) x_pos: u16,
    pub(crate) y_row: u16,
    pub(crate) count: u16,
    pub(crate) stride_word: u16,
    pub(crate) property_words: [u16; 4],
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct LevelLoadPointersLoc11D0C {
    pub(crate) theme_mappings_addr: u32,
    pub(crate) enemy_layout_addr: u32,
    pub(crate) enemy_layout_header_addr: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct FlagpoleScoreboardCursorSubC254 {
    pub(crate) col: u16,
    pub(crate) row: u16,
    pub(crate) remaining_cols_in_row: i16,
    pub(crate) row_width: i16,
    pub(crate) remaining_rows: i16,
    pub(crate) row_start_col: u16,
    pub(crate) budget: i16,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct FlagpoleScoreboardStepSubC254 {
    pub(crate) entry: FgUpdateQueueCustomEntrySub1596,
    pub(crate) next_cursor: FlagpoleScoreboardCursorSubC254,
    pub(crate) finished: bool,
}

pub(crate) const SUB_28FC_SLOT_COUNT: usize = 6;
pub(crate) const SUB_28FC_SLOT_STRIDE_BYTES: usize = 0x52;
const SPRITE_TABLE_ADDR_SUB_B52: u32 = 0xFFFF_0000;
const SPRITE_DYNAMIC_ART_ADDR_SUB_B52: u32 = 0xFFFF_0500;
const EVANESCENT_OBJECT_RAM_ADDR_SUB_EABC: u32 = 0xFFFF_EBDA;
const EVANESCENT_OBJECT_SLOT_STRIDE_SUB_EABC: u16 = 0x000C;
const EVANESCENT_OBJECT_SLOT_COUNT_SUB_EABC: usize = 40;
const SHOOTER_OBJECT_RAM_ADDR_SUB_F06A: u32 = 0xFFFF_E7F2;
const SHOOTER_OBJECT_SLOT_STRIDE_SUB_F06A: u16 = 0x000E;
const SHOOTER_OBJECT_SLOT_COUNT_SUB_F06A: usize = 20;
const HILLS_OF_THE_WARRIOR_1_LEVEL_ID_LOC_11AE4: u8 = 0x08;
const LEVEL_LAYOUT_ROW_POINTER_BASE_SMALL_LOC_11AE4: u32 = 0xFFFF_9E32;
const LEVEL_LAYOUT_ROW_POINTER_BASE_LARGE_LOC_11AEA: u32 = 0xFFFF_DD02;
const LEVEL_LAYOUT_RAM_BASE_LOC_11AFE: u16 = 0xA652;
const LEVEL_TERRAIN_LAYOUT_RAM_BASE_LOC_11B40: u16 = 0x52C2;
const BACKGROUND_ROW_POINTER_BASE_LOC_11B7A: u16 = 0x87B2;

#[cfg_attr(not(test), allow(dead_code))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct SpriteFrameStateSubB52 {
    pub(crate) number_sprites: u8,
    pub(crate) next_sprite_slot_addr: u32,
    pub(crate) dynamic_sprite_art_addr: u32,
}

#[cfg_attr(not(test), allow(dead_code))]
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ScratchQueueSub28FC {
    pub(crate) slots: [[u8; SUB_28FC_SLOT_STRIDE_BYTES]; SUB_28FC_SLOT_COUNT],
    pub(crate) next_slot: usize,
    pub(crate) slots_remaining: u16,
}

#[cfg_attr(not(test), allow(dead_code))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct LinkedObjectSlotInit {
    pub(crate) slot_addr: u32,
    pub(crate) next_slot_addr: u32,
}

#[cfg_attr(not(test), allow(dead_code))]
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct EvanescShooterObjectSlotsSubEABC {
    pub(crate) next_free_evanescent_object_slot_addr: u32,
    pub(crate) first_evanescent_object_slot_addr: u32,
    pub(crate) evanescent_slots: Vec<LinkedObjectSlotInit>,
}

#[cfg_attr(not(test), allow(dead_code))]
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ShooterObjectSlotsSubF06A {
    pub(crate) next_free_shooter_object_slot_addr: u32,
    pub(crate) first_shooter_object_slot_addr: u32,
    pub(crate) shooter_slots: Vec<LinkedObjectSlotInit>,
    pub(crate) last_shooter_block_col: i16,
    pub(crate) last_shooter_block_row: i16,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct ScratchDebrisPieceSub292E {
    pub(crate) y_velocity_px: i16,
    pub(crate) x_velocity_px: i16,
    pub(crate) y_pos_px: u16,
    pub(crate) x_pos_px: u16,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct ScratchDebrisSpriteSub2A4C {
    pub(crate) y: u16,
    pub(crate) link: u16,
    pub(crate) tile_word: u16,
    pub(crate) x: u16,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ScratchDebrisRenderSub2A4C {
    pub(crate) sprites: Vec<ScratchDebrisSpriteSub2A4C>,
    pub(crate) number_sprites: u8,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct FullscreenPaletteFadeSub73D0 {
    pub(crate) palette_to_dma_enabled: bool,
    pub(crate) palette_buffer_2: [u16; 0x40],
    pub(crate) palette_buffer_3: [u16; 0x40],
    pub(crate) fade_progress: u16,
    pub(crate) pending_palette_0_mask: u32,
    pub(crate) pending_palette_1_mask: u32,
    pub(crate) allow_pausing: bool,
}

const SUB_292E_PIECE_COUNT: usize = 5;
const SUB_292E_SLOT_ACTIVE_OFFSET: usize = 0;
const SUB_292E_SLOT_FLAG_OFFSET: usize = 1;
const SUB_292E_SLOT_PIECES_OFFSET: usize = 2;
const SUB_292E_PIECE_STRIDE_BYTES: usize = 0x10;
const SUB_292E_Y_OFFSETS: [u16; SUB_292E_PIECE_COUNT] = [0, 8, 8, 0, 4];
const SUB_292E_X_OFFSETS: [u16; SUB_292E_PIECE_COUNT] = [0, 0, 8, 8, 4];
const SUB_292E_Y_VELOCITIES: [[i16; SUB_292E_PIECE_COUNT]; 4] = [
    [-1, -1, 0, 1, 0],
    [-1, -1, 0, 1, -1],
    [-1, -1, 0, 1, 0],
    [0, 0, 0, 0, 0],
];
const SUB_292E_X_VELOCITIES: [[i16; SUB_292E_PIECE_COUNT]; 4] = [
    [-4, -3, -3, -4, -5],
    [-2, 0, 0, -1, -2],
    [-3, -3, -3, -3, -3],
    [-5, -4, -3, -2, -1],
];
const SUB_2A4C_GRAVITY_STEP_FP: i32 = 0x0000_36B0;
const SUB_2A4C_SPRITE_TILE_WORD: u16 = 0x8228;
const SUB_2A4C_SPRITE_COORD_BIAS: i16 = 0x0080;

#[cfg_attr(not(test), allow(dead_code))]
#[allow(non_snake_case)]
pub(crate) fn level_layout_row_pointer_base_loc_11AE4(level_id: LevelId, width_blocks: u16) -> u32 {
    // ASM Load_InGame (kid.asm:23679-23688) seeds `a3` with `$FFFFDD02`, but
    // branches to `loc_11AE4` for Hills of the Warrior 1 or any layout whose
    // width is not greater than `$12C` blocks; `loc_11AEA` is the shared tail
    // that consumes the selected row-pointer base.
    if level_id.0 == HILLS_OF_THE_WARRIOR_1_LEVEL_ID_LOC_11AE4 || width_blocks <= 0x012C {
        LEVEL_LAYOUT_ROW_POINTER_BASE_SMALL_LOC_11AE4
    } else {
        LEVEL_LAYOUT_ROW_POINTER_BASE_LARGE_LOC_11AEA
    }
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn level_layout_row_pointers_loc_11AFE(
    level_height_blocks: u16,
    level_width_tiles: u16,
) -> Vec<u16> {
    // ASM loc_11AFE (kid.asm:23696-23699) seeds `a2` with `Level_Layout`, then
    // emits one row-start pointer word per block row into `$FFFF4A04+`.
    // Because the ROM uses `move.w a2,(a1)+` and `add.w Level_width_tiles,a2`,
    // preserve the low-word RAM arithmetic rather than widening to a host index.
    let mut row_pointer = LEVEL_LAYOUT_RAM_BASE_LOC_11AFE;
    let mut row_pointers = Vec::with_capacity(usize::from(level_height_blocks));
    for _ in 0..level_height_blocks {
        row_pointers.push(row_pointer);
        row_pointer = row_pointer.wrapping_add(level_width_tiles);
    }
    row_pointers
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn solid_collision_row_loc_11B12(level_width_blocks: u16) -> Vec<u16> {
    // ASM loc_11B12 (kid.asm:23704-23706) reloads `d0` with
    // `Level_width_blocks-1`, seeds `d1` with `$6000`, then writes exactly one
    // solid-collision word per block column into the row buffer at `a3`.
    vec![0x6000; usize::from(level_width_blocks)]
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn row_buffer_pointer_table_loc_11B1A(
    row_pointer_base: u32,
    level_width_blocks: u16,
) -> [u16; 8] {
    // ASM loc_11B1A (kid.asm:23709-23711) runs after loc_11B12 advanced `a3`
    // across the just-filled collision row, so `move.w a3,(a1)+` repeats the
    // row-end pointer word eight times into the secondary pointer table.
    let row_end_pointer =
        (row_pointer_base as u16).wrapping_add(level_width_blocks.wrapping_mul(2));
    [row_end_pointer; 8]
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn empty_collision_row_loc_11B28(level_width_blocks: u16) -> Vec<u16> {
    // ASM loc_11B28 (kid.asm:23716-23718) reloads `d0` with
    // `Level_width_blocks-1`, clears `d1`, then writes exactly one zero word
    // per block column into the next row buffer at `a3`.
    vec![0; usize::from(level_width_blocks)]
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn terrain_layout_row_pointers_loc_11B40(
    level_height_blocks: u16,
    level_width_blocks: u16,
) -> Vec<u16> {
    // ASM loc_11B40 (kid.asm:23724-23727) seeds `a4` with
    // `Level_terrain_layout`, then emits one row-start pointer word per block
    // row into `$FFFF4BB8+`. Because terrain entries are one byte each, the
    // loop advances `a4` by `Level_width_blocks`, not `Level_width_tiles`.
    let mut row_pointer = LEVEL_TERRAIN_LAYOUT_RAM_BASE_LOC_11B40;
    let mut row_pointers = Vec::with_capacity(usize::from(level_height_blocks));
    for _ in 0..level_height_blocks {
        row_pointers.push(row_pointer);
        row_pointer = row_pointer.wrapping_add(level_width_blocks);
    }
    row_pointers
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn decompress_chunk_attempts_loc_11C10(status_words: &[u16]) -> usize {
    // ASM loc_11C10 (kid.asm:23803-23806) calls `Decompress_Chunk`, tests the
    // returned `d0`, and branches back to `loc_11C10` until the word result is
    // exactly zero. Return the number of attempts consumed through the first
    // successful zero status, or all provided attempts if no zero is present.
    match status_words.iter().position(|&status| status == 0) {
        Some(success_index) => success_index + 1,
        None => status_words.len(),
    }
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn decode_block_layout_loc_132A6(layout_bytes: &[u8]) -> Result<Vec<u16>, String> {
    // ASM LoadBlockLayout loc_132A6 (kid.asm:25563-25582) saves registers,
    // seeds the Enigma bit reader via `sub_13618`/`sub_13622`, and starts
    // decoding block-layout words into `Decompression_Buffer` once the caller
    // has already dispatched here for the `$40` layout tag.
    kid_data::decode_enigma(layout_bytes, 0).map_err(|error| error.to_string())
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn block_layout_dispatch_loc_132D6(
    high_bit_set: bool,
    block_type: u16,
) -> BlockLayoutDispatchLoc132D6 {
    // ASM loc_132D6 (kid.asm:25580-25593) reads a 1-bit high flag, rotates it
    // into bit 15 with `ror.w #1,d1`, reads the 5-bit block type, treats `$1F`
    // as the terminator, and otherwise ORs the pieces together before writing
    // the header word into `Decompression_Buffer`.
    BlockLayoutDispatchLoc132D6 {
        header_word: (if high_bit_set { 0x8000 } else { 0 }) | (block_type & 0x001F),
        terminated: block_type == 0x001F,
    }
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn block_layout_repeat_loop_loc_1331E(
    x_pos: u16,
    end_marker_x_pos: u16,
) -> BlockLayoutLoopLoc1331E {
    // ASM loc_1331E (kid.asm:25614-25621) reads the next X position through
    // `sub_13622`, compares it to the sentinel loaded into `a5`, writes
    // `$FFFF` and jumps back to loc_132D6 on a match, or falls through to
    // loc_1332E to emit the rest of the entry words.
    if x_pos == end_marker_x_pos {
        BlockLayoutLoopLoc1331E::RestartAtLoc132D6 { end_marker: 0xFFFF }
    } else {
        BlockLayoutLoopLoc1331E::ContinueLoc1332E
    }
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn block_layout_entry_words_loc_1332E(
    x_pos: u16,
    y_row: u16,
    count_bits: u16,
    stride_bit: bool,
) -> BlockLayoutEntryWordsLoc1332E {
    // ASM loc_1332E (kid.asm:25624-25635) writes the decoded X and Y words,
    // reads a 4-bit run length and stores `count+1`, reads one stride bit, and
    // appends all four words before looping back to loc_1331E.
    BlockLayoutEntryWordsLoc1332E {
        x_pos,
        y_row,
        count: count_bits.wrapping_add(1),
        stride_word: u16::from(stride_bit),
    }
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn block_layout_variable_repeat_loop_loc_1334C(
    x_pos: u16,
    end_marker_x_pos: u16,
) -> BlockLayoutLoopLoc1331E {
    // ASM loc_1334C (kid.asm:25639-25646) repeats the same sentinel gate as
    // loc_1331E, but through the alternate dispatch arm rooted at
    // `off_132FC[1]`: decode the next X word, compare it to `a5`, write
    // `$FFFF`, and jump back to loc_132D6 on a match; otherwise fall through
    // to loc_1335E for the wider entry format.
    if x_pos == end_marker_x_pos {
        BlockLayoutLoopLoc1331E::RestartAtLoc132D6 { end_marker: 0xFFFF }
    } else {
        BlockLayoutLoopLoc1331E::ContinueLoc1332E
    }
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn block_layout_variable_repeat_loop_loc_133A0(
    x_pos: u16,
    end_marker_x_pos: u16,
) -> BlockLayoutLoopLoc133A0 {
    // ASM loc_133A0 (kid.asm:25684-25691) repeats the same `a5` sentinel gate
    // through the third dispatch arm: decode the next X word with `sub_13622`,
    // write `$FFFF`, and jump back to loc_132D6 on a match; otherwise fall
    // through to loc_133B2 for the unflagged property-word format.
    if x_pos == end_marker_x_pos {
        BlockLayoutLoopLoc133A0::RestartAtLoc132D6 { end_marker: 0xFFFF }
    } else {
        BlockLayoutLoopLoc133A0::ContinueLoc133B2
    }
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn block_layout_fixed_pattern_repeat_loop_loc_133DE(
    x_pos: u16,
    end_marker_x_pos: u16,
) -> BlockLayoutLoopLoc133DE {
    // ASM loc_133DE (kid.asm:25716-25723) reuses the same `a5` sentinel gate:
    // decode the next X word with `sub_13622`, write `$FFFF`, and jump back to
    // `loc_132D6` on a match; otherwise fall through to `loc_133F0`.
    if x_pos == end_marker_x_pos {
        BlockLayoutLoopLoc133DE::RestartAtLoc132D6 { end_marker: 0xFFFF }
    } else {
        BlockLayoutLoopLoc133DE::ContinueLoc133F0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct BlockLayoutEntryWordsLoc1335E {
    pub(crate) x_pos: u16,
    pub(crate) y_row: u16,
    pub(crate) count: u16,
    pub(crate) stride_word: u16,
    pub(crate) property_word_dbf_counter: u16,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct BlockLayoutEntryWordsLoc133B2 {
    pub(crate) x_pos: u16,
    pub(crate) y_row: u16,
    pub(crate) count: u16,
    pub(crate) stride_word: u16,
    pub(crate) property_word_dbf_counter: u16,
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn block_layout_entry_words_loc_1335E(
    x_pos: u16,
    y_row: u16,
    count_bits: u16,
    stride_bit: bool,
) -> BlockLayoutEntryWordsLoc1335E {
    // ASM loc_1335E (kid.asm:25649-25661) writes the decoded X and Y words,
    // copies the raw 4-bit count into `d2` for the later `dbf d2,loc_1337C`
    // loop, stores `count+1`, then appends the initial 1-bit stride word
    // before falling through into loc_1337C.
    BlockLayoutEntryWordsLoc1335E {
        x_pos,
        y_row,
        count: count_bits.wrapping_add(1),
        stride_word: u16::from(stride_bit),
        property_word_dbf_counter: count_bits,
    }
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn block_layout_entry_words_loc_133B2(
    x_pos: u16,
    y_row: u16,
    count_bits: u16,
    stride_bit: bool,
) -> BlockLayoutEntryWordsLoc133B2 {
    // ASM loc_133B2 (kid.asm:25694-25706) mirrors the wider header write from
    // loc_1335E: store decoded X and Y words, keep the raw 4-bit count in `d2`
    // for the later `dbf d2,loc_133D0` loop, then write `count+1` and the
    // initial 1-bit stride word before the unflagged property-word loop.
    BlockLayoutEntryWordsLoc133B2 {
        x_pos,
        y_row,
        count: count_bits.wrapping_add(1),
        stride_word: u16::from(stride_bit),
        property_word_dbf_counter: count_bits,
    }
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn block_layout_entry_words_loc_133F0(
    x_pos: u16,
    y_row: u16,
    count_bits: u16,
    stride_bit: bool,
    property_words: [u16; 4],
) -> BlockLayoutEntryWordsLoc133F0 {
    // ASM loc_133F0 (kid.asm:25726-25748) writes the decoded X and Y words,
    // reads a 3-bit run length and stores `count+1`, reads one stride bit, then
    // appends the trailing 8/3/8/8-bit property words before looping back to
    // `loc_133DE`.
    BlockLayoutEntryWordsLoc133F0 {
        x_pos,
        y_row,
        count: count_bits.wrapping_add(1),
        stride_word: u16::from(stride_bit),
        property_words,
    }
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn block_layout_long_run_repeat_loop_loc_1342E(
    x_pos: u16,
    end_marker_x_pos: u16,
) -> BlockLayoutLoopLoc1342E {
    // ASM loc_1342E (kid.asm:25753-25760) repeats the same sentinel check in
    // the next dispatch arm: decode X via `sub_13622`, emit `$FFFF`, and jump
    // back to `loc_132D6` on a match; otherwise fall through to `loc_13440`.
    if x_pos == end_marker_x_pos {
        BlockLayoutLoopLoc1342E::RestartAtLoc132D6 { end_marker: 0xFFFF }
    } else {
        BlockLayoutLoopLoc1342E::ContinueLoc13440
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct BlockLayoutPropertyWordLoc1337C {
    pub(crate) high_bit_set: bool,
    pub(crate) low_nibble: u16,
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn block_layout_store_property_word_loc_13398(property_word: u16) -> u16 {
    // ASM loc_13398 (kid.asm:25678-25681) is the shared store/loop tail after
    // either loc_13392 leaves `d1` as the raw 4-bit property word or
    // loc_1337C sets bit 15 with `ori.w #$8000,d1`; the common tail writes that
    // final word to `(a1)+` before `dbf d2,loc_1337C`.
    property_word
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn block_layout_short_pair_repeat_loop_loc_1345E(
    x_pos: u16,
    end_marker_x_pos: u16,
) -> BlockLayoutLoopLoc1345E {
    // ASM loc_1345E (kid.asm:25778-25785) keeps the same sentinel gate as the
    // prior dispatch arms: decode X through `sub_13622`, compare it to the
    // `a5` sentinel, emit `$FFFF`, and jump back to `loc_132D6` on a match;
    // otherwise fall through to `loc_13470`.
    if x_pos == end_marker_x_pos {
        BlockLayoutLoopLoc1345E::RestartAtLoc132D6 { end_marker: 0xFFFF }
    } else {
        BlockLayoutLoopLoc1345E::ContinueLoc13470
    }
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn block_layout_entry_words_loc_13470(
    x_pos: u16,
    y_row: u16,
) -> BlockLayoutEntryWordsLoc13470 {
    // ASM loc_13470 (kid.asm:25788-25793) is the short two-word tail for this
    // dispatch arm: write the X word already decoded in `d1`, decode the next
    // Y word through `sub_13622`, write that second word, then loop back to
    // `loc_1345E`.
    BlockLayoutEntryWordsLoc13470 { x_pos, y_row }
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn block_layout_short_count_repeat_loop_loc_1347C(
    x_pos: u16,
    end_marker_x_pos: u16,
) -> BlockLayoutLoopLoc1347C {
    // ASM loc_1347C (kid.asm:25796-25803) repeats the same `a5` sentinel gate
    // for the next dispatch arm: decode X via `sub_13622`, write `$FFFF`, and
    // jump back to `loc_132D6` on a match; otherwise fall through to
    // `loc_1348E`.
    if x_pos == end_marker_x_pos {
        BlockLayoutLoopLoc1347C::RestartAtLoc132D6 { end_marker: 0xFFFF }
    } else {
        BlockLayoutLoopLoc1347C::ContinueLoc1348E
    }
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn block_layout_short_count_property_words_loc_134AC(
    property_words: &[u16],
    dbf_counter: u16,
) -> Vec<u16> {
    // ASM loc_134AC (kid.asm:25820-25824) is the short-count arm's trailing
    // property loop: each iteration forces `d0 = 3`, decodes one 3-bit word
    // through `sub_13622`, stores it to `(a1)+`, then `dbf d2,loc_134AC`
    // repeats exactly `d2 + 1` times.
    property_words
        .iter()
        .copied()
        .take(usize::from(dbf_counter) + 1)
        .collect()
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn block_layout_terminate_loc_134BA() -> BlockLayoutTerminateLoc134BA {
    // ASM loc_134BA (kid.asm:25828-25831) writes the `$FFFF` terminator into
    // the current decode stream, then resets `a1` to `Decompression_Buffer`
    // before restoring registers and returning from `EniDec`.
    BlockLayoutTerminateLoc134BA {
        end_marker: 0xFFFF,
        output_addr: 0xFFFF_77B2,
    }
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn block_layout_dispatch_loc_13504(
    decoded_word: u16,
    decode_width_bits: u16,
    bits_remaining_before: u16,
) -> BlockLayoutDispatchLoc13504 {
    // ASM loc_13504 (kid.asm:25870-25874) keeps the low nibble of `d2` as the
    // inclusive `dbf` counter, shifts the upper nibble of `d1` into a word
    // offset for the `loc_13560` jump table, and relies on `loc_13650` to
    // subtract the consumed width from `d6`, refilling one byte only when the
    // remaining bit budget drops below 9.
    let bits_after_subtract = bits_remaining_before.wrapping_sub(decode_width_bits);
    let (bits_remaining, refilled_input_byte) = if bits_after_subtract >= 9 {
        (bits_after_subtract, false)
    } else {
        (bits_after_subtract.wrapping_add(8), true)
    };

    BlockLayoutDispatchLoc13504 {
        property_word_dbf_counter: decoded_word & 0x000F,
        jump_table_word_offset: ((decoded_word >> 4) & 0x000F) << 1,
        bits_remaining,
        refilled_input_byte,
    }
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn block_layout_repeat_words_loc_13514(
    start_word: u16,
    dbf_counter: u16,
) -> BlockLayoutRepeatWordsLoc13514 {
    // ASM loc_13514 (kid.asm:25878-25881) writes `a2` to `(a1)+`, increments
    // `a2` by one word after every store, and lets `dbf d2,loc_13514` make the
    // loop inclusive, so `d2 = 0` still emits exactly one word.
    let iterations = usize::from(dbf_counter) + 1;
    let words = (0..iterations)
        .map(|step| start_word.wrapping_add(u16::try_from(step).unwrap_or(u16::MAX)))
        .collect();

    BlockLayoutRepeatWordsLoc13514 {
        words,
        next_word: start_word.wrapping_add(dbf_counter).wrapping_add(1),
    }
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn block_layout_repeat_words_loc_1351E(
    repeated_word: u16,
    dbf_counter: u16,
) -> BlockLayoutRepeatWordsLoc1351E {
    // ASM loc_1351E (kid.asm:25886-25888) stores `a4` to `(a1)+` for every
    // iteration but never changes `a4`, so the inclusive `dbf d2,loc_1351E`
    // loop repeats one constant word exactly `d2 + 1` times.
    let iterations = usize::from(dbf_counter) + 1;

    BlockLayoutRepeatWordsLoc1351E {
        words: vec![repeated_word; iterations],
        next_word: repeated_word,
    }
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn block_layout_repeat_current_word_loc_1352A(
    current_word: u16,
    dbf_counter: u16,
) -> BlockLayoutRepeatWordsLoc1351E {
    // ASM loc_1352A (kid.asm:25895-25898) stores the current `d1` word to
    // `(a1)+` and never mutates `d1` inside the loop, so `dbf d2,loc_1352A`
    // repeats the decoded word exactly `d2 + 1` times.
    let iterations = usize::from(dbf_counter) + 1;

    BlockLayoutRepeatWordsLoc1351E {
        words: vec![current_word; iterations],
        next_word: current_word,
    }
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn block_layout_repeat_decoded_word_loc_13526(
    decoded_word: u16,
    dbf_counter: u16,
) -> BlockLayoutRepeatWordsLoc1351E {
    // ASM loc_13526 (kid.asm:25892-25898) calls `sub_13588` once to decode the
    // seed word into `d1`, then falls through to `loc_1352A` without mutating
    // that word again, so the emitted run is whatever `sub_13588` produced.
    block_layout_repeat_current_word_loc_1352A(decoded_word, dbf_counter)
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn block_layout_increment_decoded_word_loc_13532(
    decoded_word: u16,
    dbf_counter: u16,
) -> BlockLayoutRepeatWordsLoc13514 {
    // ASM loc_13532 (kid.asm:25901-25906) calls `sub_13588` once, stores the
    // decoded `d1` word, then `addq.w #1,d1` after each write while
    // `dbf d2,loc_13536` keeps the loop inclusive.
    block_layout_repeat_words_loc_13514(decoded_word, dbf_counter)
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn block_layout_decrement_words_loc_13544(
    start_word: u16,
    dbf_counter: u16,
) -> BlockLayoutRepeatWordsLoc13514 {
    // ASM loc_13544 (kid.asm:25914-25917) stores `d1` to `(a1)+`, then
    // `subq.w #1,d1` after each write while `dbf d2,loc_13544` keeps the loop
    // inclusive, so `d2 = 0` still emits exactly one word.
    let iterations = usize::from(dbf_counter) + 1;
    let words = (0..iterations)
        .map(|step| start_word.wrapping_sub(u16::try_from(step).unwrap_or(u16::MAX)))
        .collect();

    BlockLayoutRepeatWordsLoc13514 {
        words,
        next_word: start_word.wrapping_sub(dbf_counter).wrapping_sub(1),
    }
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn block_layout_decrement_decoded_word_loc_13540(
    decoded_word: u16,
    dbf_counter: u16,
) -> BlockLayoutRepeatWordsLoc13514 {
    // ASM loc_13540 (kid.asm:25911-25917) calls `sub_13588` once to seed `d1`,
    // then falls through to `loc_13544`, so the emitted run decrements from
    // the single decoded word instead of decoding a fresh word per iteration.
    block_layout_decrement_words_loc_13544(decoded_word, dbf_counter)
}

fn block_layout_rewind_input_addr_loc_13570(mut next_input_addr: u16, bits_remaining: u16) -> u16 {
    // ASM loc_13570 (kid.asm:25935-25944) rewinds `a0` by one byte, rewinds a
    // second byte when `d6 == $10`, then falls through to `loc_1357A` for the
    // final even-address alignment before `loc_13582` restores registers and
    // returns from `EniDec`.
    next_input_addr = next_input_addr.wrapping_sub(1);
    if bits_remaining == 0x0010 {
        next_input_addr = next_input_addr.wrapping_sub(1);
    }
    block_layout_align_rewound_input_addr_loc_1357A(next_input_addr)
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn block_layout_align_rewound_input_addr_loc_1357A(mut next_input_addr: u16) -> u16 {
    // ASM loc_1357A (kid.asm:25941-25944) copies `a0` into `d0`, shifts right
    // one bit, and uses the carry from that shift to detect odd addresses. If
    // the rewind path left `a0` odd, it adds one byte so `loc_13582` returns
    // with an even-aligned input pointer.
    if next_input_addr & 1 != 0 {
        next_input_addr = next_input_addr.wrapping_add(1);
    }
    next_input_addr
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn block_layout_return_input_addr_loc_13582(next_input_addr: u16) -> u16 {
    // ASM loc_13582 (kid.asm:25946-25947) only restores `d0-d7/a1-a5` from the
    // stack and returns, so the aligned `a0` from `loc_1357A` passes through
    // unchanged as the caller-visible decode pointer.
    next_input_addr
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn block_layout_apply_high_bit_loc_1359A(
    base_word: u16,
    property_byte: u8,
    bits_remaining: u16,
    bit_buffer: u16,
) -> BlockLayoutDecodeHighBitLoc1359A {
    // ASM sub_13588 / loc_1359A (kid.asm:25952-25963) seeds `d3` from `a3`,
    // copies `d4` into `d1`, then uses the first `add.b d1,d1` carry as a gate
    // for the `$8000` contribution. Only when that carry is set does the code
    // consume one bit from `d5` via `subq.w #1,d6` + `btst d6,d5`; otherwise it
    // falls through to `loc_1359A` without touching `d6` or bit 15.
    let shifted_property_byte = property_byte.wrapping_add(property_byte);
    let mut decoded_word = base_word;
    let mut bits_remaining_after = bits_remaining;

    if property_byte & 0x80 != 0 {
        bits_remaining_after = bits_remaining_after.wrapping_sub(1);
        if bit_buffer & (1u16 << bits_remaining_after) != 0 {
            decoded_word |= 0x8000;
        }
    }

    BlockLayoutDecodeHighBitLoc1359A {
        decoded_word,
        shifted_property_byte,
        bits_remaining: bits_remaining_after,
    }
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn block_layout_apply_second_high_bit_loc_135A8(
    base_word: u16,
    property_byte: u8,
    bits_remaining: u16,
    bit_buffer: u16,
) -> BlockLayoutDecodeBitLoc135A8 {
    // ASM loc_135A8 (kid.asm:25964-25971) repeats the same property-bit gate:
    // `add.b d1,d1` shifts the second flag into carry, and only that carry path
    // consumes one `d5` bit through `subq.w #1,d6` + `btst d6,d5`. When the
    // tested bit is set it contributes `$4000` to the decoded word; otherwise
    // the code falls through to `loc_135A8` with just the shifted `d1`.
    let shifted_property_byte = property_byte.wrapping_add(property_byte);
    let mut decoded_word = base_word;
    let mut bits_remaining_after = bits_remaining;

    if property_byte & 0x80 != 0 {
        bits_remaining_after = bits_remaining_after.wrapping_sub(1);
        if bit_buffer & (1u16 << bits_remaining_after) != 0 {
            decoded_word = decoded_word.wrapping_add(0x4000);
        }
    }

    BlockLayoutDecodeBitLoc135A8 {
        decoded_word,
        shifted_property_byte,
        bits_remaining: bits_remaining_after,
    }
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn block_layout_apply_third_high_bit_loc_135B6(
    base_word: u16,
    property_byte: u8,
    bits_remaining: u16,
    bit_buffer: u16,
) -> BlockLayoutDecodeBitLoc135B6 {
    // ASM loc_135B6 (kid.asm:25972-25979) shifts the next property flag into
    // carry with `add.b d1,d1`, consumes one source bit only on that carry
    // path, and ORs `$2000` into `d3` when `btst d6,d5` sees a set bit.
    let shifted_property_byte = property_byte.wrapping_add(property_byte);
    let mut decoded_word = base_word;
    let mut bits_remaining_after = bits_remaining;

    if property_byte & 0x80 != 0 {
        bits_remaining_after = bits_remaining_after.wrapping_sub(1);
        if bit_buffer & (1u16 << bits_remaining_after) != 0 {
            decoded_word = decoded_word.wrapping_add(0x2000);
        }
    }

    BlockLayoutDecodeBitLoc135B6 {
        decoded_word,
        shifted_property_byte,
        bits_remaining: bits_remaining_after,
    }
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn block_layout_apply_fourth_high_bit_loc_135C4(
    base_word: u16,
    property_byte: u8,
    bits_remaining: u16,
    bit_buffer: u16,
) -> BlockLayoutDecodeBitLoc135C4 {
    // ASM loc_135C4 (kid.asm:25980-25987) shifts the fourth property flag into
    // carry, only decrements `d6` and probes `d5` on that carry path, and ORs
    // `$1000` into `d3` when the consumed source bit is set.
    let shifted_property_byte = property_byte.wrapping_add(property_byte);
    let mut decoded_word = base_word;
    let mut bits_remaining_after = bits_remaining;

    if property_byte & 0x80 != 0 {
        bits_remaining_after = bits_remaining_after.wrapping_sub(1);
        if bit_buffer & (1u16 << bits_remaining_after) != 0 {
            decoded_word |= 0x1000;
        }
    }

    BlockLayoutDecodeBitLoc135C4 {
        decoded_word,
        shifted_property_byte,
        bits_remaining: bits_remaining_after,
    }
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn block_layout_finalize_word_loc_135F0(
    partial_word: u16,
    base_word: u16,
    decode_width_bits: u16,
    next_input_word: u16,
) -> BlockLayoutFinalizeWordLoc135F0 {
    // ASM loc_135F0 (kid.asm:26031-26037) indexes `return_13662` with `a5`,
    // masks `d1` down to the requested decode width, adds the accumulated base
    // word from `d3`, then refills `d5` from the next two input bytes at `a0`.
    let decode_mask = match decode_width_bits {
        0 => 0,
        1..=15 => (1u16 << decode_width_bits) - 1,
        _ => u16::MAX,
    };

    BlockLayoutFinalizeWordLoc135F0 {
        decoded_word: (partial_word & decode_mask).wrapping_add(base_word),
        bit_buffer: next_input_word,
    }
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn decompress_short_ref_tail_loc_136D0(
    copied_bytes: [u8; 2],
    output_addr_after_tail: u16,
    output_limit_addr: u16,
) -> DecompressShortRefTailLoc136D0 {
    // ASM loc_136D0 (kid.asm:26177-26182) copies exactly two bytes from the
    // short-reference source into `(a2)+`, then uses `cmp.w a2,d7` / `bls.s
    // loc_13724` to finish once the post-copy output cursor has reached or
    // passed the end pointer carried in `d7`.
    DecompressShortRefTailLoc136D0 {
        copied_bytes,
        finished: output_addr_after_tail >= output_limit_addr,
    }
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn decompress_long_ref_prefix_loc_136FE(
    source_bytes: &[u8],
    dbf_counter: u16,
) -> Vec<u8> {
    // ASM loc_136FE (kid.asm:26207-26208) copies one byte before each
    // `dbf d0,loc_136FE` test, so after `subq.w #6,d0` the loop emits exactly
    // `d0 + 1` prefix bytes before the fixed five-byte tail at
    // `Decompress_BP0_LongRef_2` / `loc_13706` / `loc_13708`.
    source_bytes
        .iter()
        .copied()
        .take(usize::from(dbf_counter) + 1)
        .collect()
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn decompress_long_ref_zero_length_loc_13716() -> DecompressBp0ReturnStatus {
    // ASM loc_13716 (kid.asm:26226-26227) is the empty-length early return
    // from `Decompress_BP0_LongRef_2or3`: `beq.s loc_13716` zeroes `d0` and
    // returns without seeding `d2` with another copy mode.
    DecompressBp0ReturnStatus {
        status_word: 0,
        next_copy_mode: None,
    }
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn decompress_long_ref_too_short_loc_1371C() -> DecompressBp0ReturnStatus {
    // ASM loc_1371C (kid.asm:26231-26233) is the underflow return from
    // `subq.w #6,d0`: when the requested long-reference byte count cannot pay
    // for the fixed five-byte tail, the routine returns `d0=$FFFF` and seeds
    // `d2=1` so the caller retries with the one-byte direct-copy mode.
    DecompressBp0ReturnStatus {
        status_word: u16::MAX,
        next_copy_mode: Some(1),
    }
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn decompress_long_ref_finished_loc_1372C() -> DecompressBp0ReturnStatus {
    // ASM loc_1372C (kid.asm:26243-26245) is the successful long-reference
    // return after `cmp.w a2,d7` / `bls.s loc_1372C`: once the fixed five-byte
    // tail reaches or passes the output limit, the routine returns `d0=1` and
    // `d2=1`, matching the caller's "done" status plus direct-copy mode tag.
    DecompressBp0ReturnStatus {
        status_word: 1,
        next_copy_mode: Some(1),
    }
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn decompress_direct_copy_loc_137AE(source_byte: u8) -> DecompressDirectCopyLoc137AE {
    // ASM loc_137AE (kid.asm:26339) is the bit-position-0 direct-copy entry:
    // it copies exactly one literal byte from `(a1)+` to `(a2)+` before
    // falling through to `loc_137B0` for the next mode decode.
    DecompressDirectCopyLoc137AE {
        copied_byte: source_byte,
    }
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn decompress_mode_dispatch_loc_13736(
    bit_buffer: u8,
    output_addr: u16,
    short_ref_back_distance: u8,
) -> DecompressModeDispatchLoc13736 {
    // ASM loc_13736 (kid.asm:26252-26263) consumes the next mode bits with
    // three `add.b d1,d1` tests: `1` keeps the direct-copy path, `01` seeds
    // `a6` from `a2` and branches into the long-reference header, and `00:A`
    // subtracts the short back-reference distance from `a2` then chooses a
    // two-byte or three-byte copy depending on the trailing `A` bit.
    if bit_buffer & 0x80 != 0 {
        return DecompressModeDispatchLoc13736::DirectCopy;
    }

    if bit_buffer & 0x40 != 0 {
        return DecompressModeDispatchLoc13736::LongRef {
            output_copy_base_addr: output_addr,
        };
    }

    let total_copy_len = if bit_buffer & 0x20 != 0 { 3 } else { 2 };
    DecompressModeDispatchLoc13736::ShortRef {
        source_copy_addr: output_addr.wrapping_sub(u16::from(short_ref_back_distance)),
        total_copy_len,
    }
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn decompress_short_ref_finished_loc_1374A(
    copied_bytes: [u8; 2],
    output_addr_after_tail: u16,
    output_limit_addr: u16,
) -> DecompressShortRefTailLoc136D0 {
    // ASM loc_1374A (kid.asm:26266-26271) is the shared short-reference tail
    // after the optional extra byte from `loc_13736`: it copies exactly two
    // more bytes from `(a6)+` into `(a2)+`, then uses `cmp.w a2,d7` /
    // `bls.s loc_1379E` to decide whether the decode has finished.
    DecompressShortRefTailLoc136D0 {
        copied_bytes,
        finished: output_addr_after_tail >= output_limit_addr,
    }
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn decompress_mode_dispatch_loc_137B0(
    bit_buffer: u8,
    output_addr: u16,
    short_ref_back_distance: u8,
) -> DecompressModeDispatchLoc137B0 {
    // ASM loc_137B0 (kid.asm:26342-26353) re-runs the same three `add.b d1,d1`
    // mode tests after the literal-byte entry at `loc_137AE`: `1` branches to
    // `loc_1382C` for another direct-copy entry, `01` seeds `a6` from `a2`
    // and jumps into the long-reference header at `loc_137D0`, and `00:A`
    // subtracts the short back-reference byte before choosing a two-byte or
    // three-byte copy tail.
    if bit_buffer & 0x80 != 0 {
        return DecompressModeDispatchLoc137B0::DirectCopy;
    }

    if bit_buffer & 0x40 != 0 {
        return DecompressModeDispatchLoc137B0::LongRef {
            output_copy_base_addr: output_addr,
        };
    }

    let total_copy_len = if bit_buffer & 0x20 != 0 { 3 } else { 2 };
    DecompressModeDispatchLoc137B0::ShortRef {
        source_copy_addr: output_addr.wrapping_sub(u16::from(short_ref_back_distance)),
        total_copy_len,
    }
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn decompress_mode_dispatch_loc_1382E(
    bit_buffer: u8,
    output_addr: u16,
    short_ref_back_distance: u8,
) -> DecompressModeDispatchLoc1382E {
    // ASM loc_1382E (kid.asm:26418-26429) repeats the same three `add.b d1,d1`
    // mode tests used by `loc_137B0`, but this later decode lane branches to
    // `loc_138A8` for another direct-copy byte, `loc_1384E` for the later
    // long-reference family, and `loc_13842` for the two-byte or three-byte
    // short-reference tail.
    if bit_buffer & 0x80 != 0 {
        return DecompressModeDispatchLoc1382E::DirectCopy;
    }

    if bit_buffer & 0x40 != 0 {
        return DecompressModeDispatchLoc1382E::LongRef {
            output_copy_base_addr: output_addr,
        };
    }

    let total_copy_len = if bit_buffer & 0x20 != 0 { 3 } else { 2 };
    DecompressModeDispatchLoc1382E::ShortRef {
        source_copy_addr: output_addr.wrapping_sub(u16::from(short_ref_back_distance)),
        total_copy_len,
    }
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn decompress_mode_dispatch_loc_138AA(
    bit_buffer: u8,
    output_addr: u16,
    source_byte: u8,
    short_ref_back_distance: u8,
) -> DecompressModeDispatchLoc138AA {
    // ASM loc_138AA (kid.asm:26519-26534) is the later bit-position-1 decode
    // entry: `1` copies one literal byte through `loc_13926`, `01` seeds `a6`
    // from `a2` and branches into the later long-reference header at
    // `loc_138CA`, and `00:A` subtracts the short back-reference byte from
    // `a2` before choosing the two-byte or three-byte short-reference tail at
    // `loc_138BE`.
    if bit_buffer & 0x80 != 0 {
        return DecompressModeDispatchLoc138AA::DirectCopy {
            copied_byte: source_byte,
        };
    }

    if bit_buffer & 0x40 != 0 {
        return DecompressModeDispatchLoc138AA::LongRef {
            output_copy_base_addr: output_addr,
        };
    }

    let total_copy_len = if bit_buffer & 0x20 != 0 { 3 } else { 2 };
    DecompressModeDispatchLoc138AA::ShortRef {
        source_copy_addr: output_addr.wrapping_sub(u16::from(short_ref_back_distance)),
        total_copy_len,
    }
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn decompress_mode_dispatch_loc_13928(
    bit_buffer: u8,
    output_addr: u16,
    short_ref_back_distance: u8,
) -> DecompressModeDispatchLoc13928 {
    // ASM loc_13928 (kid.asm:26608-26619) re-runs the three `add.b d1,d1`
    // mode tests after the literal-byte prelude at `loc_13926`: `1` branches
    // to the next direct-copy entry at `loc_139A4`, `01` seeds `a6` from `a2`
    // and jumps into the later long-reference header at `loc_13948`, and
    // `00:A` subtracts the short back-reference byte before choosing the
    // three-byte or two-byte short-reference tail at `loc_1393C`.
    if bit_buffer & 0x80 != 0 {
        return DecompressModeDispatchLoc13928::DirectCopy;
    }

    if bit_buffer & 0x40 != 0 {
        return DecompressModeDispatchLoc13928::LongRef {
            output_copy_base_addr: output_addr,
        };
    }

    let total_copy_len = if bit_buffer & 0x20 != 0 { 3 } else { 2 };
    DecompressModeDispatchLoc13928::ShortRef {
        source_copy_addr: output_addr.wrapping_sub(u16::from(short_ref_back_distance)),
        total_copy_len,
    }
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn decompress_long_ref_header_loc_13756(
    bit_buffer: u8,
    long_ref_mask: u16,
    long_ref_low_byte: u8,
) -> DecompressLongRefHeaderLoc13756 {
    // ASM loc_13756 (kid.asm:26274-26284) shifts the already-consumed `d1`
    // word left three more bits so `BBB` lands in the high-byte distance
    // field, masks that field through `d4`, appends the next input byte, then
    // decodes `AA` into the 3/4/5/extended long-reference families.
    let shifted_word = u16::from(bit_buffer) << 3;
    let source_back_distance = (shifted_word & long_ref_mask) | u16::from(long_ref_low_byte);
    let copy_mode = match (shifted_word >> 6) & 0b11 {
        0b00 => DecompressLongRefCopyModeLoc13756::Copy3,
        0b01 => DecompressLongRefCopyModeLoc13756::Copy4,
        0b10 => DecompressLongRefCopyModeLoc13756::Copy5,
        0b11 => DecompressLongRefCopyModeLoc13756::VariableLength,
        _ => unreachable!(),
    };

    DecompressLongRefHeaderLoc13756 {
        source_back_distance,
        copy_mode,
    }
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn decompress_short_ref_finished_loc_137C4(
    copied_bytes: [u8; 2],
    output_addr_after_tail: u16,
    output_limit_addr: u16,
) -> DecompressShortRefTailLoc137C4 {
    // ASM loc_137C4 (kid.asm:26354-26359) is the shared short-reference tail
    // after the optional third byte in `loc_137B0`: it copies exactly two
    // more bytes from `(a6)+` into `(a2)+`, then uses `cmp.w a2,d7` /
    // `bls.s loc_1381C` to decide whether the decode has finished.
    DecompressShortRefTailLoc137C4 {
        copied_bytes,
        finished: output_addr_after_tail >= output_limit_addr,
    }
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn decompress_short_ref_finished_loc_13842(
    copied_bytes: [u8; 2],
    output_addr_after_tail: u16,
    output_limit_addr: u16,
) -> DecompressShortRefTailLoc137C4 {
    // ASM loc_13842 (kid.asm:26425-26429) is the later two-byte short-ref
    // tail after the optional third-byte prelude from `loc_1382E`: it copies
    // exactly two more bytes from `(a6)+` into `(a2)+`, then uses `cmp.w a2,d7`
    // / `bls.s loc_13898` to decide whether the decode has finished.
    DecompressShortRefTailLoc137C4 {
        copied_bytes,
        finished: output_addr_after_tail >= output_limit_addr,
    }
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn decompress_long_ref_header_loc_1384E(
    bit_buffer: u8,
    control_byte: u8,
    long_ref_mask: u16,
    long_ref_low_byte: u8,
) -> DecompressLongRefHeaderLoc13756 {
    // ASM loc_1384E (kid.asm:26432-26440) first shifts the carried `d1` word
    // left three bits, then replaces its low byte with `(a0)+` before masking
    // the distance bits through `d4`, appending the next source byte, and
    // decoding the later `AA` copy-mode family: `00` => copy 3, `01` => copy
    // 4, `10` => copy 5, `11` => variable-length via `loc_13864`.
    let shifted_word = (u16::from(bit_buffer) << 3) | u16::from(control_byte);
    let source_back_distance = (shifted_word & long_ref_mask) | u16::from(long_ref_low_byte);
    let copy_mode = match (control_byte >> 6) & 0b11 {
        0b00 => DecompressLongRefCopyModeLoc13756::Copy3,
        0b01 => DecompressLongRefCopyModeLoc13756::Copy4,
        0b10 => DecompressLongRefCopyModeLoc13756::Copy5,
        0b11 => DecompressLongRefCopyModeLoc13756::VariableLength,
        _ => unreachable!(),
    };

    DecompressLongRefHeaderLoc13756 {
        source_back_distance,
        copy_mode,
    }
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn decompress_long_ref_extended_loc_1376A(
    bit_buffer: u8,
    length_byte: u8,
) -> DecompressLongRefExtendedLoc1376A {
    // ASM loc_1376A (kid.asm:26287-26295) consumes the second `AA` bit: carry
    // clear means `AA=10`, so the code skips straight to `loc_1377E` for the
    // fixed five-byte tail. Carry set means `AA=11`, so the next source byte
    // selects quit (`0`), flush (`1..=5` after `subq.w #6,d0` underflows), or
    // the variable-length prefix that feeds `dbf d0,loc_13778`.
    if bit_buffer & 0x20 == 0 {
        return DecompressLongRefExtendedLoc1376A::Copy5;
    }

    if length_byte == 0 {
        return DecompressLongRefExtendedLoc1376A::QuitDecompression;
    }

    if length_byte < 6 {
        return DecompressLongRefExtendedLoc1376A::FlushBuffer;
    }

    DecompressLongRefExtendedLoc1376A::VariableLength {
        total_copy_len: length_byte,
        dbf_counter: u16::from(length_byte - 6),
    }
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn decompress_long_ref_extended_loc_137E6(
    bit_buffer: u8,
    length_byte: u8,
) -> DecompressLongRefExtendedLoc1376A {
    // ASM loc_137E6 (kid.asm:26376-26384) consumes the post-distance control
    // bit with `add.b d1,d1`: carry clear branches straight to `loc_137FC`
    // for the fixed five-byte tail, while carry set loads `BYTE2` and uses
    // the same quit / flush / variable-length split as the earlier
    // `loc_1376A` path before feeding `dbf d0,loc_137F6`.
    if bit_buffer & 0x80 == 0 {
        return DecompressLongRefExtendedLoc1376A::Copy5;
    }

    if length_byte == 0 {
        return DecompressLongRefExtendedLoc1376A::QuitDecompression;
    }

    if length_byte < 6 {
        return DecompressLongRefExtendedLoc1376A::FlushBuffer;
    }

    DecompressLongRefExtendedLoc1376A::VariableLength {
        total_copy_len: length_byte,
        dbf_counter: u16::from(length_byte - 6),
    }
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn decompress_long_ref_extended_loc_138E2(
    bit_buffer: u8,
    length_byte: u8,
) -> DecompressLongRefExtendedLoc1376A {
    // ASM loc_138E2 (kid.asm:26554-26562) consumes the post-distance control
    // bit for the `loc_138CA` family: carry clear branches straight to
    // `loc_138F6` for the fixed five-byte tail, while carry set loads `BYTE2`
    // and uses the same quit / flush / variable-length split before feeding
    // the inclusive prefix loop at `loc_138F0`.
    if bit_buffer & 0x80 == 0 {
        return DecompressLongRefExtendedLoc1376A::Copy5;
    }

    if length_byte == 0 {
        return DecompressLongRefExtendedLoc1376A::QuitDecompression;
    }

    if length_byte < 6 {
        return DecompressLongRefExtendedLoc1376A::FlushBuffer;
    }

    DecompressLongRefExtendedLoc1376A::VariableLength {
        total_copy_len: length_byte,
        dbf_counter: u16::from(length_byte - 6),
    }
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn decompress_long_ref_loop_loc_13778(
    source_bytes: &[u8],
    dbf_counter: u16,
) -> DecompressLongRefLoopLoc13778 {
    // ASM loc_13778 (kid.asm:26297-26299) copies one byte from `(a6)+` to
    // `(a2)+` before each `dbf d0,loc_13778` check, so the loop emits exactly
    // `d0+1` bytes from the variable-length long-reference prefix.
    DecompressLongRefLoopLoc13778 {
        copied_bytes: source_bytes
            .iter()
            .copied()
            .take(usize::from(dbf_counter) + 1)
            .collect(),
    }
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn decompress_long_ref_loop_loc_13872(
    source_bytes: &[u8],
    dbf_counter: u16,
) -> DecompressLongRefLoopLoc13778 {
    // ASM loc_13872 (kid.asm:26474-26475) is the later variable-length
    // back-reference prefix loop: after `subq.w #6,d0`, it copies one byte
    // from `(a6)+` to `(a2)+` before each `dbf d0,loc_13872` check, then
    // falls through to `loc_13878` for the fixed tail bytes.
    DecompressLongRefLoopLoc13778 {
        copied_bytes: source_bytes
            .iter()
            .copied()
            .take(usize::from(dbf_counter) + 1)
            .collect(),
    }
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn decompress_long_ref_loop_loc_137F6(
    source_bytes: &[u8],
    dbf_counter: u16,
) -> DecompressLongRefLoopLoc13778 {
    // ASM loc_137F6 (kid.asm:26386-26387) copies one byte from `(a6)+` to
    // `(a2)+` before each `dbf d0,loc_137F6` check, so the variable-length
    // branch from `loc_137E6` emits exactly `d0+1` prefix bytes before the
    // fixed tail that starts at `loc_137FC`.
    DecompressLongRefLoopLoc13778 {
        copied_bytes: source_bytes
            .iter()
            .copied()
            .take(usize::from(dbf_counter) + 1)
            .collect(),
    }
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn decompress_long_ref_loop_loc_138F0(
    source_bytes: &[u8],
    dbf_counter: u16,
) -> DecompressLongRefLoopLoc13778 {
    // ASM loc_138F0 (kid.asm:26564-26565) copies one byte from `(a6)+` to
    // `(a2)+` before each `dbf d0,loc_138F0` check, so the variable-length
    // branch from `loc_138E2` emits exactly `d0+1` prefix bytes before the
    // fixed tail that starts at `loc_138F6`.
    DecompressLongRefLoopLoc13778 {
        copied_bytes: source_bytes
            .iter()
            .copied()
            .take(usize::from(dbf_counter) + 1)
            .collect(),
    }
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn decompress_long_ref_quit_loc_1380E() -> DecompressBp0ReturnStatus {
    // ASM loc_1380E (kid.asm:26405-26406) is the `BYTE2 = 0` early return from
    // `loc_137E6`: it clears `d0` and returns immediately without reseeding
    // `d2`, so the caller keeps its existing copy mode.
    DecompressBp0ReturnStatus {
        status_word: 0,
        next_copy_mode: None,
    }
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn decompress_long_ref_quit_loc_1388A() -> DecompressBp0ReturnStatus {
    // ASM loc_1388A (kid.asm:26493-26494) is the later `BYTE2 = 0` early
    // return from the variable-length branch that feeds `loc_13872`: it clears
    // `d0` and returns immediately without writing a new copy mode to `d2`.
    DecompressBp0ReturnStatus {
        status_word: 0,
        next_copy_mode: None,
    }
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn decompress_long_ref_quit_loc_13908() -> DecompressBp0ReturnStatus {
    // ASM loc_13908 (kid.asm:26582-26583) is the `BYTE2 = 0` early return from
    // `loc_138E2`: it clears `d0` and returns immediately without writing a
    // new copy mode to `d2`, so the caller keeps its existing state.
    DecompressBp0ReturnStatus {
        status_word: 0,
        next_copy_mode: None,
    }
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn decompress_long_ref_quit_loc_13790() -> DecompressBp0ReturnStatus {
    // ASM loc_13790 (kid.asm:26316-26317) is the `BYTE2 = 0` early return from
    // `loc_1376A`: it clears `d0` and returns immediately without reseeding
    // `d2` with another copy mode for the caller.
    DecompressBp0ReturnStatus {
        status_word: 0,
        next_copy_mode: None,
    }
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn decompress_long_ref_tail_loc_13878(
    copied_bytes: [u8; 5],
    output_addr_after_tail: u16,
    output_limit_addr: u16,
) -> DecompressLongRefTailLoc1377E {
    // ASM loc_13878 (kid.asm:26477-26486) is the later fixed five-byte tail:
    // one byte at `loc_13878`, one at `loc_1387A`, then three more through
    // the `loc_1387C` fallthrough before `cmp.w a2,d7` branches to `loc_138A0`
    // or resumes the shared decode loop at `loc_137B0`.
    DecompressLongRefTailLoc1377E {
        copied_bytes,
        finished: output_addr_after_tail >= output_limit_addr,
    }
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn decompress_long_ref_tail_loc_138F6(
    copied_bytes: [u8; 5],
    output_addr_after_tail: u16,
    output_limit_addr: u16,
) -> DecompressLongRefTailLoc1377E {
    // ASM loc_138F6 (kid.asm:26567-26576) is the later fixed five-byte tail
    // for the `loc_138CA` family: one byte at `loc_138F6`, one at `loc_138F8`,
    // then three more through the `loc_138FA` fallthrough before `cmp.w a2,d7`
    // branches to `loc_1391E` or resumes the shared decode loop at
    // `loc_1382E`.
    DecompressLongRefTailLoc1377E {
        copied_bytes,
        finished: output_addr_after_tail >= output_limit_addr,
    }
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn decompress_long_ref_tail_loc_1387A(
    copied_bytes: [u8; 4],
    output_addr_after_tail: u16,
    output_limit_addr: u16,
) -> DecompressLongRefTailLoc1387A {
    // ASM loc_1387A (kid.asm:26479-26486) enters the later fixed tail one byte
    // later than `loc_13878`: it copies one byte at `loc_1387A`, then three
    // more through the `loc_1387C` fallthrough before `cmp.w a2,d7` branches
    // to `loc_138A0` or resumes the shared decode loop at `loc_137B0`.
    DecompressLongRefTailLoc1387A {
        copied_bytes,
        finished: output_addr_after_tail >= output_limit_addr,
    }
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn decompress_long_ref_tail_loc_137FC(
    copied_bytes: [u8; 5],
    output_addr_after_tail: u16,
    output_limit_addr: u16,
) -> DecompressLongRefTailLoc1377E {
    // ASM loc_137FC (kid.asm:26390-26397) falls through `loc_137FE` and
    // `loc_13800`, so once control lands here the decompressor always copies
    // exactly five trailing bytes before `cmp.w a2,d7` either returns through
    // `loc_13824` or resumes the main decoder at `loc_13736`.
    DecompressLongRefTailLoc1377E {
        copied_bytes,
        finished: output_addr_after_tail >= output_limit_addr,
    }
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn decompress_long_ref_flush_loc_13814() -> DecompressBp0ReturnStatus {
    // ASM loc_13814 (kid.asm:26410-26412) is the `0 < BYTE2 < 6` underflow
    // return from `loc_137E6`: after `subq.w #6,d0` goes negative, the routine
    // reports `d0=$FFFF` and seeds `d2=7` so the caller resumes in the later
    // fixed-tail copy mode.
    DecompressBp0ReturnStatus {
        status_word: u16::MAX,
        next_copy_mode: Some(7),
    }
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn decompress_long_ref_flush_loc_13890() -> DecompressBp0ReturnStatus {
    // ASM loc_13890 (kid.asm:26498-26500) is the `0 < BYTE2 < 6` underflow
    // return from the later variable-length branch above `loc_13872`: after
    // `subq.w #6,d0` goes negative, the routine reports `d0=$FFFF` and seeds
    // `d2=6` so the caller resumes in the six-byte tail copy mode.
    DecompressBp0ReturnStatus {
        status_word: u16::MAX,
        next_copy_mode: Some(6),
    }
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn decompress_long_ref_flush_loc_1390E() -> DecompressBp0ReturnStatus {
    // ASM loc_1390E (kid.asm:26587-26589) is the `0 < BYTE2 < 6` underflow
    // return from `loc_138E2`: after `subq.w #6,d0` goes negative, the routine
    // reports `d0=$FFFF` and seeds `d2=5` so the caller resumes in the
    // five-byte tail copy mode.
    DecompressBp0ReturnStatus {
        status_word: u16::MAX,
        next_copy_mode: Some(5),
    }
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn decompress_long_ref_flush_loc_13796() -> DecompressBp0ReturnStatus {
    // ASM loc_13796 (kid.asm:26321-26323) is the `0 < BYTE2 < 6` underflow
    // return from `loc_1376A`: after `subq.w #6,d0` goes negative, the routine
    // reports `d0=$FFFF` and explicitly seeds `d2=0` for the caller's flush
    // handling.
    DecompressBp0ReturnStatus {
        status_word: u16::MAX,
        next_copy_mode: Some(0),
    }
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn decompress_long_ref_finished_loc_137A6() -> DecompressBp0ReturnStatus {
    // ASM loc_137A6 (kid.asm:26333-26335) is the successful return after the
    // fixed five-byte tail at `loc_1377E`: once `cmp.w a2,d7` / `bls.s
    // loc_137A6` reports the output cursor at or beyond the limit, the routine
    // returns `d0=1` and seeds `d2=0`.
    DecompressBp0ReturnStatus {
        status_word: 1,
        next_copy_mode: Some(0),
    }
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn decompress_long_ref_finished_loc_13916() -> DecompressBp0ReturnStatus {
    // ASM loc_13916 (kid.asm:26593-26595) is the successful return after the
    // fixed five-byte tail at `loc_138F6`: once `cmp.w a2,d7` / `bls.s
    // loc_13916` reports the output cursor at or beyond the limit, the routine
    // returns `d0=1` and seeds `d2=1`.
    DecompressBp0ReturnStatus {
        status_word: 1,
        next_copy_mode: Some(1),
    }
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn decompress_long_ref_tail_loc_1377E(
    copied_bytes: [u8; 5],
    output_addr_after_tail: u16,
    output_limit_addr: u16,
) -> DecompressLongRefTailLoc1377E {
    // ASM loc_1377E (kid.asm:26301-26308) falls through `loc_13780` and
    // `loc_13782`, so once control lands here the decompressor always copies
    // exactly five trailing bytes before the closing `cmp.w a2,d7`.
    DecompressLongRefTailLoc1377E {
        copied_bytes,
        finished: output_addr_after_tail >= output_limit_addr,
    }
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn block_layout_decode_word_run_loc_1354E(
    decoded_words: &[u16],
    dbf_counter: u16,
    next_input_addr: u16,
    bits_remaining: u16,
) -> BlockLayoutDecodeWordsLoc1354E {
    // ASM loc_1354E (kid.asm:25921-25932) uses `$F` as a sentinel: that value
    // skips the shared `loc_13554` decode loop and returns through `loc_13570`.
    // Any other `d2` value decodes one fresh word via `sub_13588` on every
    // inclusive `dbf d2,loc_13554` iteration.
    if dbf_counter == 0x000F {
        BlockLayoutDecodeWordsLoc1354E::ReturnViaLoc13570 {
            rewound_input_addr: block_layout_rewind_input_addr_loc_13570(
                next_input_addr,
                bits_remaining,
            ),
        }
    } else {
        BlockLayoutDecodeWordsLoc1354E::ContinueLoc13554 {
            words: decoded_words
                .iter()
                .copied()
                .take(usize::from(dbf_counter) + 1)
                .collect(),
        }
    }
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn block_layout_property_words_loc_1337C(
    property_words: &[BlockLayoutPropertyWordLoc1337C],
    dbf_counter: u16,
) -> Vec<u16> {
    // ASM loc_1337C (kid.asm:25663-25676) runs exactly `dbf_counter+1`
    // iterations: read one flag bit, and when that bit is non-zero read the
    // next 4-bit value and set bit 15 with `ori.w #$8000,d1`; otherwise store
    // the raw 4-bit value from loc_13392.
    property_words
        .iter()
        .copied()
        .take(usize::from(dbf_counter) + 1)
        .map(|entry| {
            let low_nibble = entry.low_nibble & 0x000F;
            if entry.high_bit_set {
                block_layout_store_property_word_loc_13398(low_nibble | 0x8000)
            } else {
                block_layout_store_property_word_loc_13398(low_nibble)
            }
        })
        .collect()
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn level_layout_cell_loc_11C36(terrain_byte: u8, theme_collision_index: &[u8]) -> u16 {
    // ASM loc_11C36 (kid.asm:23818-23822) reads one terrain byte through `d2`,
    // loads the matching theme-collision nibble through `d3`, shifts that
    // nibble into the high-byte collision field with `lsl.b #4,d3`, then
    // writes `[d3, d2]` into successive `Level_Layout` bytes.
    let collision_byte = theme_collision_index[usize::from(terrain_byte)] << 4;
    u16::from_be_bytes([collision_byte, terrain_byte])
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn level_layout_cells_loc_11C34(
    terrain_layout: &[u8],
    theme_collision_index: &[u8],
    level_width_blocks: u16,
    level_height_blocks: u16,
) -> Vec<u16> {
    // ASM loc_11C34 (kid.asm:23815-23827) clears the per-row column counter
    // `d1`, then re-enters loc_11C36 exactly `Level_width_blocks` times before
    // incrementing `d0` and starting the next row.
    let expected_cells = usize::from(level_width_blocks) * usize::from(level_height_blocks);
    assert!(
        terrain_layout.len() >= expected_cells,
        "terrain_layout must provide at least one byte per block cell"
    );

    let mut level_layout = Vec::with_capacity(expected_cells);
    let width = usize::from(level_width_blocks);
    let height = usize::from(level_height_blocks);
    for row in 0..height {
        for col in 0..width {
            let terrain_byte = terrain_layout[row * width + col];
            level_layout.push(level_layout_cell_loc_11C36(
                terrain_byte,
                theme_collision_index,
            ));
        }
    }
    level_layout
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn block_layout_dispatch_words_loc_11C7E(block_layout_words: &[u16]) -> Vec<u16> {
    // ASM loc_11C7E (kid.asm:23839-23848) reads one block-layout word through
    // `(a1)+`, stops once that word is `$FFFF`, and otherwise dispatches the
    // word through `AddrTbl_BlockTypes` before looping back to the next entry.
    block_layout_words
        .iter()
        .copied()
        .take_while(|&word| word != 0xFFFF)
        .collect()
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn block_type_base_cell_loc_122B8(block_type: u16) -> u16 {
    // ASM loc_122B8 (kid.asm:24405-24408): dispatches block types that are
    // routed through AddrTbl_BlockTypes to this handler. The routine:
    //   move.w d0,d1
    //   add.w d1,d1         ; double block type for byte offset into word table
    //   move.w word_122C6(pc,d1.w),d5
    // The `word_122C6` table (kid.asm:24415-24425) contains base cell values
    // indexed by block type. Each word entry in the table corresponds to one
    // block type, so the ASM doubles the type for byte addressing.
    // Block types 0, 2, 3, 6, 8, 10 dispatch to loc_122B8 from AddrTbl_BlockTypes.
    // Other types (1, 4, 5, 7, 9, etc.) dispatch to different handlers.
    // Note: table entries may be 0, which is a valid cell value (air/placeholder).
    const WORD_122C6: [u16; 10] = [
        0xE001, // type 0 → Smash block base cell
        0x0000, // type 1 (dispatches to loc_123DC, but table has $0000 placeholder)
        0xE203, // type 2 → Ice block base cell
        0x0000, // type 3 (dispatches to loc_122B8, returns $0000)
        0x0000, // type 4 (dispatches to loc_124A6)
        0xE50B, // type 5 (dispatches to loc_1263C)
        0xE60C, // type 6 → Rubber block base cell
        0xE710, // type 7 (dispatches to loc_1233C)
        0x0000, // type 8 = Shooter (dispatches to loc_1253A, ASM writes $B811-$B814)
        0xE919, // type 9 = Mushroom (dispatches to loc_122B8, uses $E919 base cell)
    ];
    let idx = usize::from(block_type);
    if idx < WORD_122C6.len() {
        WORD_122C6[idx]
    } else {
        0x0000
    }
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn block_layout_parse_entry_loc_122C0(
    layout_word: u16,
) -> Option<BlockLayoutEntryLoc122C0> {
    // ASM loc_122C0 (kid.asm:24410-24413): reads the next block-layout word from
    // `(a1)+` into `d1` and branches to `loc_122DA` for processing if the word
    // is positive (bit 15 clear). Returns from the block handler if the word is
    // negative (bit 15 set = $8000+), which signals end-of-layout.
    //   move.w (a1)+,d1
    //   bpl.s loc_122DA
    //   rts
    if layout_word & 0x8000 != 0 {
        None
    } else {
        Some(BlockLayoutEntryLoc122C0 { x_pos: layout_word })
    }
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn block_layout_read_coords_loc_122DA(
    layout_words: &[u16],
) -> Option<(BlockLayoutSetupLoc122DA, usize)> {
    // ASM loc_122DA (kid.asm:24428-24439): block placement coordinate parser.
    // Called after loc_122C0 read an X position into `d1`. This routine reads:
    //   move.w (a1)+,d2       ; Y row coordinate
    //   move.w d2,d3
    //   add.w d3,d3          ; d3 = Y * 2 (row table index)
    //   lea ($FFFF4A04).l,a2 ; Level_Layout row pointer table
    //   move.w (a2,d3.w),a4 ; a4 = row_start_addr for Y
    //   add.w d1,a4
    //   add.w d1,a4          ; a4 = row_start + 2*X (cell address in Level_Layout)
    //   move.w (a1)+,d3      ; count (number of cells to place)
    //   subq.w #1,d3         ; d3 = count - 1 (for dbf loop)
    //   tst.w (a1)+          ; stride flag: 0 = horizontal, non-zero = row-by-row
    //   bne.s loc_12318      ; branch to row-by-row handler if stride != 0
    // Returns (setup, bytes_consumed) or None if layout exhausted.
    if layout_words.len() < 3 {
        return None;
    }
    let y_row = layout_words[0];
    let count = layout_words[1];
    let stride = layout_words[2] != 0;
    Some((
        BlockLayoutSetupLoc122DA {
            y_row,
            count,
            stride,
        },
        3, // consumed 3 words: Y, count, stride
    ))
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn block_place_cells_loc_122F6(
    cell_addr: u32,
    cell_word: u16,
    count: u16,
    stride: bool,
) -> Vec<BlockPlaceResultLoc122F6> {
    // ASM loc_122F6 (kid.asm:24441-24460): writes `d5` (the base cell value from
    // loc_122B8) to `Level_Layout[a4]` for each position in a horizontal run.
    // When `d0` is negative (special block marker), calls sub_127A8/sub_127CE
    // to queue a hidden-block entry before writing.
    //   tst.w d0
    //   bmi.s loc_122FE
    //   move.w d5,(a4)       ; write cell word directly
    //   bra.s loc_12310
    // loc_122FE:               ; hidden-block case
    //   bsr.w sub_127A8
    //   bsr.w sub_127CE
    //   ...
    // loc_12310:
    //   addq.w #2,a4         ; advance to next cell (horizontal stride)
    //   dbf d3,loc_122F6     ; loop for count cells
    //   bra.s loc_122C0      ; continue to next layout entry
    // When `stride` is false (word at `(a1)+` was 0), writes cells horizontally
    // at `cell_addr`, `cell_addr + 2`, ..., incrementing address by 2 per cell.
    // The returned addresses match the layout positions for horizontal placement.
    if stride {
        Vec::new()
    } else {
        (0..count)
            .map(|i| BlockPlaceResultLoc122F6 {
                cell_addr: cell_addr + u32::from(i) * 2,
                cell_word,
            })
            .collect()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct BlockPlaceRowByRowResultLoc12320 {
    pub(crate) cell_addr: u32,
    pub(crate) x_pos: u16,
    pub(crate) y_row: u16,
    pub(crate) block_type: i16,
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn block_place_row_by_row_hidden_loc_12320(
    cell_addr: u32,
    x_pos: u16,
    y_row: u16,
    hidden_block_type: i16,
) -> BlockPlaceRowByRowResultLoc12320 {
    // ASM loc_12320 (kid.asm:24470-24477): when `d0` is negative during row-by-row
    // placement (stride != 0), call sub_127A8/sub_127CE to queue a hidden-block
    // entry, then store the cell address, X position, Y row, and block type in
    // the hidden-block list at `(a2)+`:
    //   bsr.w sub_127A8
    //   bsr.w sub_127CE
    //   move.w a4,(a2)+    ; cell address
    //   move.w d1,(a2)+    ; X position
    //   move.w d2,(a2)+    ; Y row
    //   move.b d0,(a2)+    ; block type (negative byte stored)
    //   clr.b (a2)+        ; zero padding byte
    BlockPlaceRowByRowResultLoc12320 {
        cell_addr,
        x_pos,
        y_row,
        block_type: hidden_block_type,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct BlockPlaceRowByRowAdvanceLoc12332 {
    pub(crate) next_cell_addr: u32,
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn block_place_row_by_row_advance_loc_12332(
    cell_addr: u32,
    level_width_tiles: u16,
) -> BlockPlaceRowByRowAdvanceLoc12332 {
    // ASM loc_12332 (kid.asm:24479-24481): after writing (or skipping) a cell
    // in row-by-row stride mode, advance the cell address by one full row:
    //   add.w (Level_width_tiles).w,a4
    //   dbf d3,loc_12318
    //   bra.s loc_122C0
    // The address advances by Level_width_tiles (in words), returning to the
    // same column in the next row down.
    BlockPlaceRowByRowAdvanceLoc12332 {
        next_cell_addr: cell_addr.wrapping_add(u32::from(level_width_tiles) * 2),
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct BlockType7CellResultLoc12372 {
    pub(crate) cell_word: u16,
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn block_type_7_cell_value_loc_12372(
    block_type: i16,
    terrain_byte: u8,
) -> BlockType7CellResultLoc12372 {
    // ASM loc_12372 (kid.asm:24513-24527): block type 7 handler for horizontal
    // placement. After loc_1233C saved a3 (level-terrain data pointer) and
    // loc_12346 set up cell pointers, this runs when stride flag is 0:
    //   tst.w d0                  ; test block type (negative for hidden)
    //   bmi.s loc_12388           ; branch if hidden block
    //   move.b (a3),d5            ; load byte from level-terrain data
    //   bne.s loc_12380
    //   move.w #$E60C,d5          ; default rubber cell
    //   bra.s loc_12384
    // loc_12380:
    //   move.w #$E60E,d5          ; alternate cell
    // loc_12384:
    //   move.w d5,(a4)            ; write to Level_Layout
    // When block_type is non-negative, this returns $E60C if terrain_byte is 0,
    // or $E60E if terrain_byte is non-zero. These are the same cells used by
    // loc_123A4 for row-by-row placement with horizontal stride advance.
    // For negative block types (hidden blocks), the caller should use loc_12388
    // which calls sub_127A8/sub_127CE to queue hidden-block entries instead.
    if block_type < 0 {
        // Hidden block path - caller should use loc_12388
        BlockType7CellResultLoc12372 { cell_word: 0 }
    } else if terrain_byte == 0 {
        BlockType7CellResultLoc12372 { cell_word: 0xE60C }
    } else {
        BlockType7CellResultLoc12372 { cell_word: 0xE60E }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct BlockType7AdvanceLoc1239A {
    pub(crate) next_cell_addr: u32,
    pub(crate) next_terrain_addr: u32,
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn block_type_7_horizontal_advance_loc_1239a(
    cell_addr: u32,
    terrain_addr: u32,
) -> BlockType7AdvanceLoc1239A {
    // ASM loc_1239A (kid.asm:24539-24543): horizontal stride continuation for
    // block type 7. After writing a cell at loc_12384, advance pointers:
    //   addq.w #2,a4         ; advance cell pointer by 1 cell (2 bytes)
    //   addq.w #1,a3         ; advance terrain pointer by 1 byte
    //   dbf d3,loc_12372     ; loop for remaining cells
    //   bra.s loc_1233E      ; continue to next layout entry
    // The horizontal stride advances a4 by 2 (one cell) and a3 by 1 (one terrain byte).
    BlockType7AdvanceLoc1239A {
        next_cell_addr: cell_addr.wrapping_add(2),
        next_terrain_addr: terrain_addr.wrapping_add(1),
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct BlockType7VerticalCellLoc123B6 {
    pub(crate) cell_addr: u32,
    pub(crate) cell_word: u16,
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn block_type_7_vertical_write_loc_123b6(
    cell_addr: u32,
    terrain_byte: u8,
) -> BlockType7VerticalCellLoc123B6 {
    // ASM loc_123B2/loc_123B6 (kid.asm:24555-24560): vertical placement cell
    // selection and write. The cell value logic mirrors loc_12372:
    //   tst.w d0             ; test block type (checked at loc_123A4)
    //   bmi.s loc_123BA      ; hidden block path
    //   move.b (a3),d5       ; load terrain byte
    //   bne.s loc_123B2
    //   move.w #$E60C,d5     ; default cell (falls through to loc_123B6)
    //   bra.s loc_123B6
    // loc_123B2:
    //   move.w #$E60E,d5     ; alternate cell
    // loc_123B6:
    //   move.w d5,(a4)       ; write cell to Level_Layout
    //   bra.s loc_123CC      ; advance for next row
    // This function assumes block_type >= 0 (non-hidden).
    let cell_word = if terrain_byte == 0 { 0xE60C } else { 0xE60E };
    BlockType7VerticalCellLoc123B6 {
        cell_addr,
        cell_word,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct BlockType7AdvanceLoc123CC {
    pub(crate) next_cell_addr: u32,
    pub(crate) next_terrain_addr: u32,
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn block_type_7_vertical_advance_loc_123cc(
    cell_addr: u32,
    terrain_addr: u32,
    level_width_tiles: u16,
    level_width_blocks: u16,
) -> BlockType7AdvanceLoc123CC {
    // ASM loc_123CC (kid.asm:24572-24576): vertical stride continuation for
    // block type 7. After writing a cell at loc_123B6, advance to next row:
    //   add.w (Level_width_tiles).w,a4   ; advance cell pointer by one row
    //   add.w (Level_width_blocks).w,a3  ; advance terrain pointer by one row
    //   dbf d3,loc_123A4               ; loop for remaining rows
    //   bra.w loc_1233E                ; continue to next layout entry
    // The vertical stride advances by full row widths in both arrays.
    BlockType7AdvanceLoc123CC {
        next_cell_addr: cell_addr.wrapping_add(u32::from(level_width_tiles) * 2),
        next_terrain_addr: terrain_addr.wrapping_add(u32::from(level_width_blocks)),
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct BlockLayoutEntryLoc1233E {
    pub(crate) x_pos: u16,
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn block_layout_read_entry_loc_1233E(
    layout_word: u16,
) -> Option<BlockLayoutEntryLoc1233E> {
    // ASM loc_1233E (kid.asm:24488-24492): reads the next layout word from `(a1)+`
    // into `d1` and returns immediately if the value is negative (bit 15 set),
    // signalling end-of-layout. Otherwise continues to loc_12346 for processing:
    //   move.w (a1)+,d1
    //   bpl.s loc_12346
    //   move.l (sp)+,a3
    //   rts
    // This is the entry point for block types dispatched through AddrTbl_BlockTypes
    // entry 7 (loc_1233C pushes a3 first, then falls into loc_1233E).
    if layout_word & 0x8000 != 0 {
        None
    } else {
        Some(BlockLayoutEntryLoc1233E { x_pos: layout_word })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct BlockType1InitLoc123DC {
    pub(crate) block_list_addr: u32,
    pub(crate) already_initialized: bool,
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn block_type_1_init_list_loc_123dc(
    existing_list_addr: u32,
    prize_list_addr: u32,
) -> BlockType1InitLoc123DC {
    // ASM loc_123DC (kid.asm:24579-24582): block type 1 (prize block) entry point.
    // Initializes the block list pointer if not already set:
    //   move.l ($FFFFF8D4).w,d1   ; load existing block list pointer
    //   bne.s loc_123E6           ; if non-zero, skip init
    //   move.l a5,($FFFFF8D4).w   ; store prize list base address
    // The $FFFFF8D4 variable holds a pointer to the prize/hidden block list.
    // a5 contains the starting address for theprize block entries.
    // This allows multiple prize block groups to share a single list allocation.
    if existing_list_addr != 0 {
        BlockType1InitLoc123DC {
            block_list_addr: existing_list_addr,
            already_initialized: true,
        }
    } else {
        BlockType1InitLoc123DC {
            block_list_addr: prize_list_addr,
            already_initialized: false,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct BlockLayoutEntryLoc123E6 {
    pub(crate) x_pos: u16,
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn block_layout_read_x_pos_loc_123e6(
    layout_word: u16,
) -> Option<BlockLayoutEntryLoc123E6> {
    // ASM loc_123E6 (kid.asm:24584-24587): reads the next layout word from `(a1)+`
    // into `d1` for block type 1 processing. Returns if the value is negative
    // (bit 15 set), signalling end-of-layout. Otherwise continues to loc_123EC:
    //   move.w (a1)+,d1
    //   bpl.s loc_123EC
    //   rts
    // This mirrors loc_122C0 but for the block type 1 handler dispatch.
    // The block type 1 handler is reached via AddrTbl_BlockTypes entry 1.
    if layout_word & 0x8000 != 0 {
        None
    } else {
        Some(BlockLayoutEntryLoc123E6 { x_pos: layout_word })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct BlockType1CellSetupLoc123EC {
    pub(crate) cell_addr: u32,
    pub(crate) y_row: u16,
    pub(crate) count: u16,
    pub(crate) stride: bool,
    pub(crate) words_consumed: usize,
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn block_type_1_cell_coords_loc_123ec(
    layout_words: &[u16],
    row_pointer_table: &[u16],
    x_pos: u16,
) -> Option<BlockType1CellSetupLoc123EC> {
    // ASM loc_123EC (kid.asm:24590-24601): block type 1 coordinate and stride parser.
    // Called after loc_123E6 read an X position into `d1`. This routine reads:
    //   move.w (a1)+,d2       ; Y row coordinate
    //   move.w d2,d3
    //   add.w d3,d3          ; d3 = Y * 2 (row table index)
    //   lea ($FFFF4A04).l,a2 ; Level_Layout row pointer table
    //   move.w (a2,d3.w),a4  ; a4 = row_start_addr for Y
    //   add.w d1,a4
    //   add.w d1,a4          ; a4 = row_start + 2*X (cell address in Level_Layout)
    //   move.w (a1)+,d3      ; count (number of cells to place)
    //   subq.w #1,d3         ; d3 = count - 1 (for dbf loop)
    //   tst.w (a1)+          ; stride flag: 0 = horizontal, non-zero = row-by-row
    //   bne.s loc_12454      ; branch to row-by-row handler if stride != 0
    // Returns (setup, words_consumed) or None if layout exhausted.
    if layout_words.len() < 3 {
        return None;
    }
    let y_row = layout_words[0];
    let count = layout_words[1];
    let stride = layout_words[2] != 0;

    let y_idx = usize::from(y_row) * 2;
    if y_idx + 1 >= row_pointer_table.len() * 2 {
        return None;
    }
    let row_start = if y_idx < row_pointer_table.len() * 2 {
        u32::from(row_pointer_table[y_idx / 2])
    } else {
        return None;
    };

    let cell_addr = row_start.wrapping_add(u32::from(x_pos) * 2);

    Some(BlockType1CellSetupLoc123EC {
        cell_addr,
        y_row,
        count,
        stride,
        words_consumed: 3,
    })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct BlockType1CellValueLoc12408 {
    pub(crate) cell_addr: u32,
    pub(crate) cell_word: u16,
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn block_type_1_cell_value_loc_12408(
    cell_addr: u32,
    property_word: u16,
    block_type: i16,
) -> BlockType1CellValueLoc12408 {
    // ASM loc_12408 (kid.asm:24603-24611): determines the cell value for block type 1
    // (prize block) when stride is zero (horizontal placement).
    //   move.w (a1)+,d5       ; property/flags word
    //   bsr.w nullsub_1       ; does nothing, just returns
    //   tst.w d0              ; test pre-existing d0 (block type)
    //   bmi.s loc_12422       ; if negative (hidden), branch
    //   tst.w d5              ; test property word
    //   bmi.s loc_1241C       ; if negative, use $E101
    //   move.w #$E102,(a4)    ; write $E102 tile
    //   bra.s loc_12434
    // loc_1241C:
    //   move.w #$E101,(a4)    ; write $E101 tile
    //   bra.s loc_12434
    // The $E101/$E102 values are prize block tile IDs. $E101 has the high bit set
    // (disguised/hidden prize), $E102 is the normal visible prize block.
    // The bsr.w nullsub_1 is a no-op; d0 is tested to check if this is a hidden block
    // (negative block_type from a prior hidden-block detection routine).
    // When block_type is non-negative and property_word has bit 15 set, use $E101.
    // Otherwise use $E102 for normal prize blocks.
    let cell_word = if block_type < 0 {
        // Hidden block case branches to loc_12422, which calls sub_127A8/sub_127CE
        // For this native function, we return the cell_addr and let caller handle
        // the hidden-block queueing separately.
        0xE101
    } else if property_word & 0x8000 != 0 {
        0xE101
    } else {
        0xE102
    };
    BlockType1CellValueLoc12408 {
        cell_addr,
        cell_word,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct BlockType1RowByRowValueLoc12454 {
    pub(crate) cell_addr: u32,
    pub(crate) property_word: u16,
    pub(crate) block_type: i16,
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn block_type_1_row_by_row_read_property_loc_12454(
    cell_addr: u32,
    property_word: u16,
    block_type: i16,
) -> BlockType1RowByRowValueLoc12454 {
    // ASM loc_12454 (kid.asm:24643-24651): row-by-row placement entry for block type 1.
    // After stride word was non-zero at loc_123EC, this reads property word:
    //   move.w (a1)+,d5      ; load property word
    //   bsr.w nullsub_1      ; no-op
    //   tst.w d0             ; test block_type
    //   bmi.s loc_1246E      ; branch to hidden block handler if negative
    //   tst.w d5             ; test property word
    //   bmi.s loc_12468      ; branch to $E101 cell if bit 15 set
    //   move.w #$E102,(a4)   ; write $E102 (visible prize)
    //   bra.s loc_12480      ; continue to finalization
    // Dispatch based on block_type and property_word flags.
    BlockType1RowByRowValueLoc12454 {
        cell_addr,
        property_word,
        block_type,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct BlockType1DisguisedLoc12468 {
    pub(crate) cell_addr: u32,
    pub(crate) cell_word: u16,
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn block_type_1_row_by_row_disguised_cell_loc_12468(
    cell_addr: u32,
) -> BlockType1DisguisedLoc12468 {
    // ASM loc_12468 (kid.asm:24654-24656): write $E101 cell for row-by-row
    // placement with disguised/hidden property flag:
    //   move.w #$E101,(a4)   ; write disguised prize tile
    //   bra.s loc_12480      ; continue to finalization
    // Called when property_word has bit 15 set and block_type is non-negative.
    // The $E101 tile is a disguised prize block with the high bit set.
    BlockType1DisguisedLoc12468 {
        cell_addr,
        cell_word: 0xE101,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct BlockType1HiddenLoc1246E {
    pub(crate) cell_addr: u32,
    pub(crate) x_pos: u16,
    pub(crate) y_row: u16,
    pub(crate) block_type: i16,
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn block_type_1_row_by_row_hidden_loc_1246E(
    cell_addr: u32,
    x_pos: u16,
    y_row: u16,
    hidden_block_type: i16,
) -> BlockType1HiddenLoc1246E {
    // ASM loc_1246E (kid.asm:24659-24666): hidden block handling for row-by-row.
    // Called when block_type is negative (hidden block detected):
    //   bsr.w sub_127A8      ; allocate hidden-block entry
    //   bsr.w sub_127CE      ; set up hidden-block data
    //   move.w a4,(a2)+      ; store cell address
    //   move.w d1,(a2)+      ; store X position
    //   move.w d2,(a2)+      ; store Y row
    //   move.b d0,(a2)+      ; store block_type (negative byte)
    //   clr.b (a2)+          ; zero padding byte
    // After this, falls through to loc_12480 to finalize the placement.
    // The hidden-block list entry is 8 bytes: cell_addr(2) + x(2) + y(2) + type(1) + pad(1).
    BlockType1HiddenLoc1246E {
        cell_addr,
        x_pos,
        y_row,
        block_type: hidden_block_type,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct BlockType1FinalizeLoc12480 {
    pub(crate) next_cell_addr: u32,
    pub(crate) next_list_index: u16,
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn block_type_1_row_by_row_finalize_loc_12480(
    cell_addr: u32,
    list_index: u16,
    level_width_tiles: u16,
) -> BlockType1FinalizeLoc12480 {
    // ASM loc_12480 (kid.asm:24668-24679): finalize row-by-row placement.
    // After writing cell (or queuing hidden block), update the block list:
    //   move.l ($FFFFF8D4).w,a5    ; load block list base
    //   move.w ($FFFFF8D2).w,d6    ; load block list count
    //   lsl.w #3,d6              ; count * 8 (8 bytes per entry)
    //   add.w d6,a5              ; a5 = base + count*8
    //   move.w a4,(a5)+          ; store cell address
    //   move.w d1,(a5)+          ; store X position
    //   move.w d2,(a5)+          ; store Y row
    //   move.w d5,(a5)+          ; store property word
    //   addq.w #1,($FFFFF8D2).w  ; increment block list count
    //   add.w (Level_width_tiles).w,a4  ; advance cell to next row
    //   dbf d3,loc_12454         ; loop for remaining rows
    //   bra.w loc_123E6          ; continue layout processing
    // This both records the placement in the block/hidden list and advances
    // the cell address by one full row for the next iteration.
    let next_cell_addr = cell_addr.wrapping_add(u32::from(level_width_tiles) * 2);
    let next_list_index = list_index.wrapping_add(1);
    BlockType1FinalizeLoc12480 {
        next_cell_addr,
        next_list_index,
    }
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn hidden_block_copy_setup_loc_11C96(
    hidden_block_count: u16,
    copied_list_addr: u32,
) -> Option<HiddenBlockCopySetupLoc11C96> {
    // ASM loc_11C96 (kid.asm:23851-23856) branches to `loc_11CB2` when the
    // hidden-block count at `($FFFFF8EA).w` is zero; otherwise it seeds
    // `d0=count-1`, points `($FFFFF8EC).w` at the final block-list address in
    // `a5`, and reloads `a4` with the temp-list base `$FFFF0280`.
    if hidden_block_count == 0 {
        None
    } else {
        Some(HiddenBlockCopySetupLoc11C96 {
            dbf_counter: hidden_block_count - 1,
            copied_list_addr,
            temp_list_addr: 0xFFFF_0280,
        })
    }
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn copy_hidden_block_entries_loc_11CA8(
    temp_entries: &[[u32; 2]],
    dbf_counter: u16,
) -> Vec<[u32; 2]> {
    // ASM loc_11CA8 (kid.asm:23858-23860) copies two longwords from `(a4)+`
    // to `(a5)+` before each `dbf d0,loc_11CA8` check, so a counter of zero
    // still copies exactly one 8-byte hidden-block entry.
    temp_entries
        .iter()
        .copied()
        .take(usize::from(dbf_counter) + 1)
        .collect()
}

fn fg_queue_guard_low_word_loc_11_cb6(base_addr: u32, span_words: u16) -> bool {
    if base_addr == 0 {
        return true;
    }

    let guarded_end = (base_addr as u16).wrapping_add(span_words);
    guarded_end < 0x43BC
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn prize_block_list_guard_loc_11CB6(
    prize_block_list_addr: u32,
    prize_block_count: u16,
) -> bool {
    // ASM loc_11CB6 (kid.asm:23868-23874) skips the guard when
    // `($FFFFF8D4).w` is zero; otherwise it adds `($FFFFF8D2).w << 3` to the
    // low word of that pointer and branches to `loc_11D0A` once the result is
    // greater than or equal to `FGPlane_UpdateQueueCustom = $43BC`.
    fg_queue_guard_low_word_loc_11_cb6(prize_block_list_addr, prize_block_count << 3)
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn hidden_block_list_guard_loc_11CCA(
    hidden_block_list_addr: u32,
    hidden_block_count: u16,
) -> bool {
    // ASM loc_11CCA (kid.asm:23877-23883) skips the guard when
    // `($FFFFF8DA).w` is zero; otherwise it multiplies `($FFFFF8D8).w` by
    // `$10`, adds that span to the low word of the pointer, and branches to
    // `loc_11D0A` once the result reaches `FGPlane_UpdateQueueCustom = $43BC`.
    fg_queue_guard_low_word_loc_11_cb6(hidden_block_list_addr, hidden_block_count << 4)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct BlockType3InitLoc124A6 {
    pub(crate) hidden_block_list_addr: u32,
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn block_type_3_init_list_loc_124a6(list_addr: u32) -> BlockType3InitLoc124A6 {
    // ASM loc_124A6 (kid.asm:24691-24692): block type 3 (solid ghost) entry point.
    // Stores the hidden block list pointer for subsequent entries:
    //   move.l a5,($FFFFF8DA).w   ; save hidden block list base
    // The $FFFFF8DA variable holds the pointer to hidden block entries,
    // parallel to $FFFFF8D4 for prize blocks. After init, falls through
    // to loc_124AA which processes each hidden block entry.
    BlockType3InitLoc124A6 {
        hidden_block_list_addr: list_addr,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct BlockType3Entry {
    pub(crate) x_pos: u16,
    pub(crate) y_row: u16,
    pub(crate) count: u16,
    pub(crate) stride: u16,
    pub(crate) attr0: u16,
    pub(crate) priority: u16,
    pub(crate) attr2: u16,
    pub(crate) attr3: u16,
    pub(crate) byte5: u8,
    pub(crate) byte7: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct BlockType3ReadEntryLoc124AA {
    pub(crate) entry: Option<BlockType3Entry>,
    pub(crate) words_consumed: usize,
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn block_type_3_read_entry_loc_124aa(
    layout_words: &[u16],
) -> BlockType3ReadEntryLoc124AA {
    // ASM loc_124AA (kid.asm:24694-24698): loop entry for block type 3.
    // Reads x_pos from (a1), branches if negative (end marker):
    //   move.w (a1),d1       ; d1 = x_pos
    //   bpl.s loc_124BE      ; if >= 0, process entry
    //   addq.w #2,a1         ; skip past sentinel
    //   rts                   ; return (list exhausted)
    // Each hidden block entry is $10 bytes: x_pos(2) + y_row(2) + count(2) +
    // stride(2) + attr0(2) + priority(2) + attr2(2) + attr3(2).
    // Bytes at offset 5 and 7 are low bytes of count/stride fields.
    if layout_words.is_empty() {
        return BlockType3ReadEntryLoc124AA {
            entry: None,
            words_consumed: 0,
        };
    }
    let x_pos = layout_words[0];
    if x_pos & 0x8000 != 0 {
        // End marker: x_pos negative means list exhausted
        return BlockType3ReadEntryLoc124AA {
            entry: None,
            words_consumed: 1,
        };
    }
    if layout_words.len() < 8 {
        return BlockType3ReadEntryLoc124AA {
            entry: None,
            words_consumed: 0,
        };
    }
    let entry = BlockType3Entry {
        x_pos,
        y_row: layout_words[1],
        count: layout_words[2],
        stride: layout_words[3],
        attr0: layout_words[4],
        priority: layout_words[5],
        attr2: layout_words[6],
        attr3: layout_words[7],
        byte5: (layout_words[2] & 0xFF) as u8,
        byte7: (layout_words[3] & 0xFF) as u8,
    };
    BlockType3ReadEntryLoc124AA {
        entry: Some(entry),
        words_consumed: 8,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct BlockType3CellAddrLoc124BE {
    pub(crate) cell_addr: u32,
    pub(crate) y_row: u16,
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn block_type_3_cell_addr_loc_124be(
    row_pointer_table: &[u16],
    x_pos: u16,
    y_row: u16,
) -> Option<BlockType3CellAddrLoc124BE> {
    // ASM loc_124BE (kid.asm:24708-24715): compute cell address for block type 3.
    //   move.w 2(a1),d2          ; d2 = y_row
    //   move.w d2,d3
    //   add.w d3,d3               ; d3 = y_row * 2 (index into row pointer table)
    //   lea ($FFFF4A04).l,a2     ; Level_Layout row pointer table
    //   move.w (a2,d3.w),a4      ; a4 = row_start_addr[y_row]
    //   add.w d1,a4
    //   add.w d1,a4               ; a4 = row_start + 2*x_pos (cell address)
    let y_idx = usize::from(y_row) * 2;
    if y_idx / 2 >= row_pointer_table.len() {
        return None;
    }
    let row_start = u32::from(row_pointer_table[y_idx / 2]);
    let cell_addr = row_start.wrapping_add(u32::from(x_pos) * 2);
    Some(BlockType3CellAddrLoc124BE { cell_addr, y_row })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct BlockType3WriteEntryLoc124BE {
    pub(crate) priority: u16,
    pub(crate) words_written: usize,
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn block_type_3_write_entry_loc_124be(
    priority_raw: u16,
) -> BlockType3WriteEntryLoc124BE {
    // ASM loc_124BE (kid.asm:24716-24721): write first attribute and process priority.
    //   move.w 8(a1),(a5)+       ; write attr0 to hidden block list
    //   move.w $A(a1),d0         ; d0 = priority field
    //   beq.s loc_124E8          ; if 0, skip priority modification
    //   cmpi.w #3,d0             ; if 3, skip priority modification
    //   beq.s loc_124E8
    //   ori.w #$8000,d0          ; otherwise set high bit for hidden block visibility
    // Priority values 0 and 3 pass through unmodified; other values get $8000 ORed.
    // This controls sprite priority for the hidden block marker.
    let priority = if priority_raw == 0 || priority_raw == 3 {
        priority_raw
    } else {
        priority_raw | 0x8000
    };
    BlockType3WriteEntryLoc124BE {
        priority,
        words_written: 1,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct BlockType3HiddenBlockEntryLoc124E8 {
    pub(crate) priority: u16,
    pub(crate) attr2: u16,
    pub(crate) attr3: u16,
    pub(crate) cell_addr: u32,
    pub(crate) x_pos: u16,
    pub(crate) y_row: u16,
    pub(crate) byte5: u8,
    pub(crate) byte7: u8,
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn block_type_3_hidden_block_entry_loc_124e8(
    entry: &BlockType3Entry,
    cell_addr: u32,
) -> BlockType3HiddenBlockEntryLoc124E8 {
    // ASM loc_124E8 (kid.asm:24723-24732): write remaining hidden block entry.
    // After attr0 was written at loc_124BE and priority (possibly ORed with $8000),
    // writes the rest of the 16-byte hidden block entry:
    //   move.w d0,(a5)+          ; write priority
    //   move.w $C(a1),(a5)+     ; write attr2
    //   move.w $E(a1),(a5)+     ; write attr3
    //   move.w a4,(a5)+         ; write cell_addr (Level_Layout address)
    //   move.w d1,(a5)+         ; write x_pos
    //   move.w d2,(a5)+         ; write y_row
    //   move.b 5(a1),(a5)+      ; write low byte of count field
    //   move.b 7(a1),(a5)+      ; write low byte of stride field
    //   addq.w #1,($FFFFF8D8).w ; increment hidden block count
    // The 16-byte entry layout matches the multiplier in hidden_block_list_guard_loc_11CCA.
    let priority = if entry.priority == 0 || entry.priority == 3 {
        entry.priority
    } else {
        entry.priority | 0x8000
    };
    BlockType3HiddenBlockEntryLoc124E8 {
        priority,
        attr2: entry.attr2,
        attr3: entry.attr3,
        cell_addr,
        x_pos: entry.x_pos,
        y_row: entry.y_row,
        byte5: entry.byte5,
        byte7: entry.byte7,
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct BlockType3HorizontalFillLoc12516 {
    pub(crate) cell_addrs: Vec<u32>,
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn block_type_3_horizontal_fill_loc_12516(
    cell_addr: u32,
    count: u16,
) -> BlockType3HorizontalFillLoc12516 {
    // ASM loc_12516 (kid.asm:24740-24743): horizontal tile fill for hidden blocks.
    // Entered when stride field (offset 6 in entry) is 0 and priority is non-zero.
    //   move.w #$E304,(a4)    ; write $E304 tile at cell address
    //   addq.w #2,a4          ; advance to next horizontal cell
    //   dbf d3,loc_12516     ; loop for count-1 iterations
    // The dbf executes count times (counter starts at count-1, stops at -1).
    // Each iteration writes $E304 and advances by +2 cells.
    let iterations = usize::from(count);
    let cell_addrs: Vec<u32> = (0..iterations)
        .map(|i| cell_addr.wrapping_add((i as u32) * 2))
        .collect();
    BlockType3HorizontalFillLoc12516 { cell_addrs }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct BlockType3AdvanceEntryLoc12520 {
    pub(crate) next_entry_offset: usize,
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn block_type_3_advance_entry_loc_12520() -> BlockType3AdvanceEntryLoc12520 {
    // ASM loc_12520 (kid.asm:24745-24747): advance to next hidden block entry.
    // After filling (or skipping fill when priority is non-zero), advance:
    //   lea $10(a1),a1       ; move to next 16-byte hidden block entry
    //   bra.s loc_124AA      ; continue parsing entries
    // This is a control-flow label marking the end of hidden-block processing
    // for one entry; the actual work is just advancing the pointer by $10 bytes.
    BlockType3AdvanceEntryLoc12520 {
        next_entry_offset: 0x10,
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct BlockType3VerticalFillLoc12526 {
    pub(crate) cell_addrs: Vec<u32>,
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn block_type_3_vertical_fill_loc_12526(
    cell_addr: u32,
    count: u16,
    level_width_tiles: u16,
) -> BlockType3VerticalFillLoc12526 {
    // ASM loc_12526 (kid.asm:24750-24753): vertical tile fill for hidden blocks.
    // Entered when stride field (offset 6 in entry) is non-zero and priority is non-zero.
    //   move.w #$E304,(a4)                  ; write $E304 tile at cell address
    //   add.w (Level_width_tiles).w,a4      ; advance by one row (level width)
    //   dbf d3,loc_12526                    ; loop for count-1 iterations
    // The dbf executes count times. Each iteration writes $E304 and advances
    // by level_width_tiles cells (one full row down).
    let iterations = usize::from(count);
    let row_stride = u32::from(level_width_tiles);
    let cell_addrs: Vec<u32> = (0..iterations)
        .map(|i| cell_addr.wrapping_add((i as u32) * row_stride))
        .collect();
    BlockType3VerticalFillLoc12526 { cell_addrs }
}

// ---------------------------------------------------------------------------
// Block type 8 (Shooter) layout functions
// ASM loc_1253A, loc_12540, loc_1255C, loc_1257E (kid.asm:24758-24799)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct BlockType8ReadEntryLoc1253a {
    pub(crate) x_pos: u16,
    pub(crate) bytes_consumed: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct BlockType8SetupLoc12540 {
    pub(crate) y_row: u16,
    pub(crate) height: u16,
    pub(crate) width: u16,
    pub(crate) cell_addr: u32,
    pub(crate) bytes_consumed: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct BlockType8ShooterTile {
    pub(crate) cell_addr: u32,
    pub(crate) tile_word: u16,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct BlockType8PlacementResult {
    pub(crate) tiles: Vec<BlockType8ShooterTile>,
    pub(crate) next_column_offset: u32,
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn block_type_8_read_x_pos_loc_1253a(
    layout_words: &[u16],
) -> Option<BlockType8ReadEntryLoc1253a> {
    // ASM loc_1253A (kid.asm:24758-24761): block type 8 (Shooter) layout entry point.
    // Reads x_pos from (a1)+ and returns if sentinel ($8000+, negative x_pos).
    //   move.w (a1)+,d1       ; x_pos
    //   bpl.s loc_12540       ; continue if >= 0
    //   rts                   ; return on sentinel
    if layout_words.is_empty() {
        return None;
    }
    let x_pos = layout_words[0];
    if x_pos & 0x8000 != 0 {
        return None;
    }
    Some(BlockType8ReadEntryLoc1253a {
        x_pos,
        bytes_consumed: 1,
    })
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn block_type_8_parse_coords_loc_12540(
    layout_words: &[u16],
    x_pos: u16,
    row_table: &[u16],
    _level_width_tiles: u16,
) -> Option<BlockType8SetupLoc12540> {
    // ASM loc_12540 (kid.asm:24765-24776): parse Shooter block coordinates.
    //   move.w (a1)+,d2       ; y_row
    //   move.w d2,d3
    //   add.w d3,d3          ; d3 = y_row * 2
    //   lea ($FFFF4A04).l,a2 ; row pointer table
    //   move.w (a2,d3.w),a4  ; a4 = row_start[y_row]
    //   add.w d1,a4
    //   add.w d1,a4          ; a4 = row_start + 2 * x_pos
    //   move.w (a1)+,d3      ; height (number of 2x2 tiles)
    //   subq.w #1,d3         ; d3 = height - 1 (for dbf loop)
    //   tst.w (a1)+          ; width flag: 0 = single column, non-zero =wide
    //   bne.s loc_1257E      ; branch to wide placement if non-zero
    if layout_words.len() < 3 {
        return None;
    }
    let y_row = layout_words[0];
    let height = layout_words[1];
    let width = layout_words[2];
    let y_idx = usize::from(y_row) * 2;
    let row_start = u32::from(
        row_table
            .get(y_idx)?
            .wrapping_add(x_pos)
            .wrapping_add(x_pos),
    );
    Some(BlockType8SetupLoc12540 {
        y_row,
        height,
        width,
        cell_addr: row_start,
        bytes_consumed: 3,
    })
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn block_type_8_narrow_placement_loc_1255c(
    cell_addr: u32,
    height: u16,
    level_width_tiles: u16,
) -> BlockType8PlacementResult {
    // ASM loc_1255C (kid.asm:24778-24786): narrow Shooter block placement (width = 0).
    // Each iteration places a 2x2 Shooter tile (4 cell words) and advances
    // by 4 bytes (2 cells) for the next column in the same horizontal run.
    //   move.l a4,a2
    //   move.w #$B811,(a2)   ; top-left
    //   move.w #$B812,2(a2)  ; top-right
    //   add.w (Level_width_tiles).w,a2
    //   move.w #$B813,(a2)   ; bottom-left
    //   move.w #$B814,2(a2)  ; bottom-right
    //   addq.w #4,a4         ; advance by 2 cells (next column)
    //   dbf d3,loc_1255C
    const TILE_TOP_LEFT: u16 = 0xB811;
    const TILE_TOP_RIGHT: u16 = 0xB812;
    const TILE_BOTTOM_LEFT: u16 = 0xB813;
    const TILE_BOTTOM_RIGHT: u16 = 0xB814;
    let iterations = usize::from(height);
    let row_stride = u32::from(level_width_tiles) * 2;
    let mut tiles = Vec::with_capacity(iterations * 4);
    let mut addr = cell_addr;
    for _ in 0..iterations {
        tiles.push(BlockType8ShooterTile {
            cell_addr: addr,
            tile_word: TILE_TOP_LEFT,
        });
        tiles.push(BlockType8ShooterTile {
            cell_addr: addr + 2,
            tile_word: TILE_TOP_RIGHT,
        });
        tiles.push(BlockType8ShooterTile {
            cell_addr: addr + row_stride,
            tile_word: TILE_BOTTOM_LEFT,
        });
        tiles.push(BlockType8ShooterTile {
            cell_addr: addr + row_stride + 2,
            tile_word: TILE_BOTTOM_RIGHT,
        });
        addr = addr.wrapping_add(4);
    }
    BlockType8PlacementResult {
        tiles,
        next_column_offset: addr,
    }
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn block_type_8_wide_placement_loc_1257e(
    cell_addr: u32,
    height: u16,
    level_width_tiles: u16,
) -> BlockType8PlacementResult {
    // ASM loc_1257E (kid.asm:24790-24799): wide Shooter block placement (width > 0).
    // Each iteration places a 2x2 Shooter tile and advances by two rows
    // (2 * level_width_tiles) for the next tile in the same column.
    //   move.l a4,a2
    //   move.w #$B811,(a2)   ; top-left
    //   move.w #$B812,2(a2)  ; top-right
    //   add.w (Level_width_tiles).w,a2
    //   move.w #$B813,(a2)   ; bottom-left
    //   move.w #$B814,2(a2)  ; bottom-right
    //   add.w (Level_width_tiles).w,a4
    //   add.w (Level_width_tiles).w,a4 ; advance by 2 rows
    //   dbf d3,loc_1257E
    const TILE_TOP_LEFT: u16 = 0xB811;
    const TILE_TOP_RIGHT: u16 = 0xB812;
    const TILE_BOTTOM_LEFT: u16 = 0xB813;
    const TILE_BOTTOM_RIGHT: u16 = 0xB814;
    let iterations = usize::from(height);
    let row_stride = u32::from(level_width_tiles) * 2;
    let mut tiles = Vec::with_capacity(iterations * 4);
    let mut addr = cell_addr;
    for _ in 0..iterations {
        tiles.push(BlockType8ShooterTile {
            cell_addr: addr,
            tile_word: TILE_TOP_LEFT,
        });
        tiles.push(BlockType8ShooterTile {
            cell_addr: addr + 2,
            tile_word: TILE_TOP_RIGHT,
        });
        tiles.push(BlockType8ShooterTile {
            cell_addr: addr + row_stride,
            tile_word: TILE_BOTTOM_LEFT,
        });
        tiles.push(BlockType8ShooterTile {
            cell_addr: addr + row_stride + 2,
            tile_word: TILE_BOTTOM_RIGHT,
        });
        addr = addr.wrapping_add(row_stride);
    }
    BlockType8PlacementResult {
        tiles,
        next_column_offset: addr,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct BlockType1012ReadXPosLoc125A6 {
    pub(crate) x_pos: u16,
    pub(crate) bytes_consumed: usize,
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn block_type_1012_read_x_pos_loc_125a6(
    layout_words: &[u16],
) -> Option<BlockType1012ReadXPosLoc125A6> {
    // ASM loc_125A6 (kid.asm:24803-24807): entry point for block types 10 and 12.
    //   move.w (a1)+,d1    ; read x_pos
    //   bpl.s  loc_125AC  ; if >= 0, continue
    //   rts                ; else return (end of layout)
    // Block types 10 and 12 share this entry point via AddrTbl_BlockTypes entries 10 and 12.
    if layout_words.is_empty() {
        return None;
    }
    let x_pos = layout_words[0];
    if x_pos & 0x8000 != 0 {
        None
    } else {
        Some(BlockType1012ReadXPosLoc125A6 {
            x_pos,
            bytes_consumed: 1,
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct BlockType1012ParseCoordsLoc125AC {
    pub(crate) cell_addr: u32,
    pub(crate) y_row: u16,
    pub(crate) count: u16,
    pub(crate) stride: bool,
    pub(crate) bytes_consumed: usize,
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn block_type_1012_parse_coords_loc_125ac(
    layout_words: &[u16],
    row_pointer_table: &[u16],
    x_pos: u16,
) -> Option<BlockType1012ParseCoordsLoc125AC> {
    // ASM loc_125AC (kid.asm:24810-24821): coordinate parser for block types 10 and 12.
    //   move.w (a1)+,d2       ; Y row coordinate
    //   move.w d2,d3
    //   add.w d3,d3           ; d3 = Y * 2 (row table index)
    //   lea ($FFFF4A04).l,a2 ; Level_Layout row pointer table
    //   move.w (a2,d3.w),a4  ; a4 = row_start_addr for Y
    //   add.w d1,a4
    //   add.w d1,a4           ; a4 = row_start + 2*X (cell address)
    //   move.w (a1)+,d3       ; count (number of cells to place)
    //   subq.w #1,d3          ; d3 = count - 1 (for dbf loop)
    //   tst.w (a1)+           ; stride flag: 0 = horizontal, non-zero = row-by-row
    //   bne.s loc_12600       ; branch to row-by-row handler if stride != 0
    if layout_words.len() < 3 {
        return None;
    }
    let y_row = layout_words[0];
    let count = layout_words[1];
    let stride = layout_words[2] != 0;

    let y_idx = usize::from(y_row) * 2;
    if y_idx >= row_pointer_table.len() * 2 {
        return None;
    }
    let row_start = u32::from(row_pointer_table[y_idx / 2]);
    let cell_addr = row_start.wrapping_add(u32::from(x_pos) * 2);

    Some(BlockType1012ParseCoordsLoc125AC {
        cell_addr,
        y_row,
        count,
        stride,
        bytes_consumed: 3,
    })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct BlockType1012TileResultLoc125C8 {
    pub(crate) tile_word: u16,
    pub(crate) is_hidden: bool,
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn block_type_1012_tile_value_loc_125c8(
    property_byte: u16,
    block_type: i16,
) -> BlockType1012TileResultLoc125C8 {
    // ASM loc_125C8 (kid.asm:24823-24833): compute tile value for block types 10 and 12.
    // For visible (non-hidden) blocks:
    //   moveq #0,d6
    //   move.b d5,d6           ; d6 = low byte of property word
    //   move.w d6,(a4)         ; write to cell
    //   cmpi.w #$C,d0          ; is block_type == 12?
    //   bne.s loc_125E0
    //   addi.w #-$13D3,(a4)   ; type 12: subtract $13D3
    //   bra.s loc_125F8
    // loc_125E0:
    //   addi.w #-$15E4,(a4)   ; type 10: subtract $15E4
    //
    // For hidden blocks (d0 < 0), call sub_127A8/sub_127CE first (caller handles).
    // The tile adjustments produce values in the $E3xx range for special effects.
    if block_type < 0 {
        // Hidden block case - caller would call sub_127A8/sub_127CE
        // The property_byte is stored in hidden block list, not written to cell.
        return BlockType1012TileResultLoc125C8 {
            tile_word: property_byte,
            is_hidden: true,
        };
    }
    let low_byte = property_byte & 0xFF;
    let tile_word = if block_type == 12 {
        // ASM loc_125E0: type 12 subtracts $13D3
        // This produces $ECxx - $13D3 from low byte, but math is: (low_byte) - $13D3
        // In 16-bit arithmetic: low_byte - $13D3 = low_byte + $EC2D
        low_byte.wrapping_add(0xEC2D)
    } else {
        // ASM loc_125E0: type 10 subtracts $15E4
        // In 16-bit arithmetic: low_byte - $15E4 = low_byte + $EA1C
        low_byte.wrapping_add(0xEA1C)
    };
    BlockType1012TileResultLoc125C8 {
        tile_word,
        is_hidden: false,
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct BlockType1012HorizontalPlaceLoc125C8 {
    pub(crate) tiles: Vec<(u32, u16)>,
    pub(crate) next_addr: u32,
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn block_type_1012_horizontal_place_loc_125c8(
    cell_addr: u32,
    count: u16,
    property_words: &[u16],
    block_type: i16,
) -> BlockType1012HorizontalPlaceLoc125C8 {
    // ASM loc_125C8 (kid.asm:24823-24853): horizontal placement loop.
    // Reads property words from layout, computes tile values, writes to cells.
    //   move.w (a1)+,d5       ; read property word
    //   tst.w d0              ; check block_type
    //   bmi.s loc_125E6       ; if < 0, hidden block case
    //   <compute tile value>  ; visible case
    //   addq.w #2,a4          ; advance to next cell
    //   dbf d3,loc_125C8      ; loop for count-1 times
    //   bra.s loc_125A6       ; continue to next entry
    let iterations = usize::from(count);
    let mut tiles = Vec::with_capacity(iterations);
    let mut addr = cell_addr;

    for &property_word in property_words.iter().take(iterations) {
        let result = block_type_1012_tile_value_loc_125c8(property_word, block_type);
        if result.is_hidden {
            // Hidden blocks don't write tiles to level layout
            // (they queue hidden block entries instead - caller handles)
        } else {
            tiles.push((addr, result.tile_word));
        }
        addr = addr.wrapping_add(2);
    }

    BlockType1012HorizontalPlaceLoc125C8 {
        tiles,
        next_addr: addr,
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct BlockType1012VerticalPlaceLoc12600 {
    pub(crate) tiles: Vec<(u32, u16)>,
    pub(crate) hidden_entries: Vec<BlockType1012HiddenEntryLoc1261E>,
    pub(crate) next_addr: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct BlockType1012HiddenEntryLoc1261E {
    pub(crate) cell_addr: u32,
    pub(crate) x_pos: u16,
    pub(crate) y_row: u16,
    pub(crate) block_type: i16,
    pub(crate) property_byte: u16,
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn block_type_1012_vertical_place_loc_12600(
    cell_addr: u32,
    count: u16,
    property_words: &[u16],
    block_type: i16,
    x_pos: u16,
    y_row: u16,
    level_width_tiles: u16,
) -> BlockType1012VerticalPlaceLoc12600 {
    // ASM loc_12600 (kid.asm:24856-24886): vertical (row-by-row) placement loop.
    // This is the counterpart to loc_125C8 horizontal placement.
    // When the stride flag is non-zero, placement proceeds downward by row:
    //   move.w (a1)+,d5       ; read property word
    //   tst.w d0              ; check block_type
    //   bmi.s loc_1261E       ; if < 0, hidden block case (same as loc_125E6)
    // ASM loc_12618 (kid.asm:24869-24871): type 10 adjustment.
    //   moveq #0,d6
    //   move.b d5,d6
    //   move.w d6,(a4)        ; write to cell
    //   cmpi.w #$C,d0
    //   bne.s loc_12618       ; if not 12, branch to type 10 path
    //   addi.w #-$13D3,(a4)   ; type 12: subtract $13D3
    //   bra.s loc_12630
    // ASM loc_12618 (kid.asm:24869-24871):
    //   addi.w #-$15E4,(a4)   ; type 10: subtract $15E4 (same as loc_125E0)
    //   bra.s loc_12630
    // ASM loc_1261E (kid.asm:24874-24881): hidden block entry.
    // Same as loc_125E6: queue hidden block for later activation.
    //   bsr.w sub_127A8
    //   bsr.w sub_127CE
    //   move.w a4,(a2)+
    //   move.w d1,(a2)+
    //   move.w d2,(a2)+
    //   move.b d0,(a2)+
    //   move.b d5,(a2)+
    // ASM loc_12630 (kid.asm:24883-24886): vertical advance and loop.
    //   add.w (Level_width_tiles).w,a4  ; advance by one row (not +2 like horizontal)
    //   dbf d3,loc_12600      ; loop for count-1 times
    //   bra.w loc_125A6       ; done, continue to next entry
    let iterations = usize::from(count);
    let row_stride = u32::from(level_width_tiles) * 2;
    let mut tiles = Vec::with_capacity(iterations);
    let mut hidden_entries = Vec::new();
    let mut addr = cell_addr;

    for &property_word in property_words.iter().take(iterations) {
        let result = block_type_1012_tile_value_loc_125c8(property_word, block_type);
        if result.is_hidden {
            // ASM loc_1261E: queue hidden block entry
            hidden_entries.push(BlockType1012HiddenEntryLoc1261E {
                cell_addr: addr,
                x_pos,
                y_row,
                block_type,
                property_byte: property_word,
            });
        } else {
            // ASM loc_12618: write tile with type-specific adjustment
            tiles.push((addr, result.tile_word));
        }
        // ASM loc_12630: advance by row stride, not by 2
        addr = addr.wrapping_add(row_stride);
    }

    BlockType1012VerticalPlaceLoc12600 {
        tiles,
        hidden_entries,
        next_addr: addr,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct BlockType4InitListLoc1263C {
    pub(crate) hidden_block_list_addr: u32,
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn block_type_4_init_list_loc_1263c(
    existing_list_addr: u32,
    prize_list_addr: u32,
) -> BlockType4InitListLoc1263C {
    // ASM loc_1263C (kid.asm:24889-24892): block type 4 entry point.
    // Checks/initializes hidden block list pointer:
    //   move.l ($FFFFF8E0).w,d1  ; load existing hidden block list pointer
    //   bne.s loc_12646          ; if non-zero, skip init
    //   move.l a5,($FFFFF8E0).w  ; store prize list base address
    // The $FFFFF8E0 variable holds a pointer to the hidden block list for type 4.
    // This parallels $FFFFF8DA for block type 3 but uses a separate variable.
    // After init (or check), falls through to loc_12646.
    if existing_list_addr != 0 {
        BlockType4InitListLoc1263C {
            hidden_block_list_addr: existing_list_addr,
        }
    } else {
        BlockType4InitListLoc1263C {
            hidden_block_list_addr: prize_list_addr,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct BlockType4ReadXPosLoc12646 {
    pub(crate) x_pos: u16,
    pub(crate) words_consumed: usize,
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn block_type_4_read_x_pos_loc_12646(
    layout_words: &[u16],
) -> Option<BlockType4ReadXPosLoc12646> {
    // ASM loc_12646 (kid.asm:24894-24897): read next x_pos from block layout.
    //   move.w (a1)+,d1  ; read x_pos into d1
    //   bpl.s loc_1264C   ; if >= 0, continue processing
    //   rts              ; else return (end of layout)
    // This is the loop entry for processing block type 4 entries.
    // Negative x_pos ($8000+) signals end of the layout list.
    if layout_words.is_empty() {
        return None;
    }
    let x_pos = layout_words[0];
    if x_pos & 0x8000 != 0 {
        None
    } else {
        Some(BlockType4ReadXPosLoc12646 {
            x_pos,
            words_consumed: 1,
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct BlockType4ParseCoordsLoc1264C {
    pub(crate) cell_addr: u32,
    pub(crate) y_row: u16,
    pub(crate) is_hidden: bool,
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn block_type_4_parse_coords_loc_1264c(
    layout_words: &[u16],
    row_pointer_table: &[u16],
    x_pos: u16,
    block_type: i16,
) -> Option<BlockType4ParseCoordsLoc1264C> {
    // ASM loc_1264C (kid.asm:24900-24912): coordinate setup and cell placement for block type 4.
    //   move.w (a1)+,d2       ; read Y row
    //   move.w d2,d3
    //   add.w d3,d3           ; d3 = Y * 2 (row table index)
    //   lea ($FFFF4A04).l,a2 ; Level_Layout row pointer table
    //   move.w (a2,d3.w),a4  ; a4 = row pointer for Y
    //   add.w d1,a4           ; a4 += X
    //   add.w d1,a4           ; a4 += X (now a4 = row + X*2)
    //   tst.w d0              ; test block_type
    //   bmi.s loc_12670       ; if < 0 (hidden), branch to hidden case
    //   move.w #$E407,(a4)    ; write tile $E407 to cell
    //   move.w #$E408,2(a4)   ; write tile $E408 to cell+2
    //   bra.s loc_1269C       ; continue to metadata storage
    //
    // For visible (non-hidden) blocks, writes two consecutive tiles.
    // Block types 4 and 5 are special hidden block types (negative type values).
    if layout_words.is_empty() {
        return None;
    }
    let y_row = layout_words[0];

    let y_idx = usize::from(y_row) * 2;
    if y_idx >= row_pointer_table.len() * 2 {
        return None;
    }
    let row_start = u32::from(row_pointer_table[y_idx / 2]);
    let cell_addr = row_start.wrapping_add(u32::from(x_pos) * 2);

    Some(BlockType4ParseCoordsLoc1264C {
        cell_addr,
        y_row,
        is_hidden: block_type < 0,
    })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct BlockType4HiddenEntryLoc12670 {
    pub(crate) cell_addr: u32,
    pub(crate) cell_addr_plus_2: u32,
    pub(crate) x_pos: u16,
    pub(crate) y_row: u16,
    pub(crate) block_type: i16,
    pub(crate) flag_byte: u8,
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn block_type_4_hidden_entry_loc_12670(
    cell_addr: u32,
    x_pos: u16,
    y_row: u16,
    block_type: i16,
) -> BlockType4HiddenEntryLoc12670 {
    // ASM loc_12670 (kid.asm:24915-24932): create hidden block entry for block type 4.
    // Called when block_type < 0 (hidden block). Creates entry in hidden block list.
    //   bsr.w sub_127A8         ; write $20 placeholder to cell, copy terrain byte
    //   bsr.w sub_127CE         ; shift hidden block list to make room
    //   move.w a4,(a2)+        ; store cell_addr
    //   move.w d1,(a2)+        ; store x_pos
    //   move.w d2,(a2)+        ; store y_row
    //   move.b d0,(a2)+        ; store block_type (as byte)
    //   clr.b (a2)+            ; clear flag byte
    //   addq.w #2,a4           ; advance cell address by 2
    //   bsr.w sub_127A8         ; write $20 to second cell
    //   bsr.w sub_127CE         ; shift list again
    //   move.w a4,(a2)+        ; store cell_addr+2
    //   move.w d1,(a2)+        ; store x_pos again
    //   move.w d2,(a2)+        ; store y_row again
    //   move.b d0,(a2)+        ; store block_type again
    //   move.b #1,(a2)+        ; store flag byte = 1
    //   subq.w #2,a4           ; restore original cell address
    //
    // Creates TWO hidden block entries: one for the left tile, one for the right tile.
    // The flag byte distinguishes left (0) from right (1) tile in the pair.
    BlockType4HiddenEntryLoc12670 {
        cell_addr,
        cell_addr_plus_2: cell_addr.wrapping_add(2),
        x_pos,
        y_row,
        block_type,
        flag_byte: 0,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct BlockType11ReadXPosLoc126BA {
    pub(crate) x_pos: u16,
    pub(crate) bytes_consumed: usize,
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn block_type_11_read_x_pos_loc_126ba(
    layout_words: &[u16],
) -> Option<BlockType11ReadXPosLoc126BA> {
    // ASM loc_126BA (kid.asm:24948-24952): entry point for block type 11.
    //   move.w (a1)+,d1    ; read x_pos
    //   bpl.s  loc_126C0   ; if >= 0, continue
    //   rts                ; else return (end of layout)
    // Block type 11 dispatches here from AddrTbl_BlockTypes entry 11.
    if layout_words.is_empty() {
        return None;
    }
    let x_pos = layout_words[0];
    if x_pos & 0x8000 != 0 {
        None
    } else {
        Some(BlockType11ReadXPosLoc126BA {
            x_pos,
            bytes_consumed: 1,
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct BlockType11ParseCoordsLoc126C0 {
    pub(crate) cell_addr: u32,
    pub(crate) y_row: u16,
    pub(crate) is_hidden: bool,
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn block_type_11_parse_coords_loc_126c0(
    layout_words: &[u16],
    row_pointer_table: &[u16],
    x_pos: u16,
    block_type: i16,
) -> Option<BlockType11ParseCoordsLoc126C0> {
    // ASM loc_126C0 (kid.asm:24955-24966): coordinate parsing for block type 11.
    //   move.w (a1)+,d2       ; read Y row coordinate
    //   move.w d2,d3
    //   add.w d3,d3           ; d3 = Y * 2 (row table index)
    //   lea ($FFFF4A04).l,a2 ; Level_Layout row pointer table
    //   move.w (a2,d3.w),a4  ; a4 = row_start_addr for Y
    //   add.w d1,a4
    //   add.w d1,a4           ; a4 = row_start + 2*X (cell address)
    //   tst.w d0              ; test block_type
    //   bmi.s loc_126DE       ; if < 0 (hidden), branch to hidden case
    //   move.w #$EB2C,(a4)    ; write tile $EB2C to visible cell
    //   bra.s loc_126BA       ; loop back for next entry
    // Block type 11 handles single-cell hidden blocks (like block type 10 but with
    // tile value $EB2C instead of computed from property byte).
    if layout_words.is_empty() {
        return None;
    }
    let y_row = layout_words[0];

    let y_idx = usize::from(y_row) * 2;
    if y_idx >= row_pointer_table.len() * 2 {
        return None;
    }
    let row_start = u32::from(row_pointer_table[y_idx / 2]);
    let cell_addr = row_start.wrapping_add(u32::from(x_pos) * 2);

    Some(BlockType11ParseCoordsLoc126C0 {
        cell_addr,
        y_row,
        is_hidden: block_type < 0,
    })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct BlockType11VisibleTileLoc126C0 {
    pub(crate) tile_word: u16,
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn block_type_11_visible_tile_loc_126c0() -> BlockType11VisibleTileLoc126C0 {
    // ASM loc_126C0 (kid.asm:24963-24966): visible tile value for block type 11.
    //   tst.w d0              ; test block_type
    //   bmi.s loc_126DE       ; hidden case branches away
    //   move.w #$EB2C,(a4)    ; write tile $EB2C to visible cell
    //   bra.s loc_126BA       ; loop back for next entry
    // The visible case writes a fixed tile value $EB2C.
    BlockType11VisibleTileLoc126C0 { tile_word: 0xEB2C }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct BlockType11HiddenEntryLoc126DE {
    pub(crate) cell_addr: u32,
    pub(crate) x_pos: u16,
    pub(crate) y_row: u16,
    pub(crate) block_type: i16,
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn block_type_11_hidden_entry_loc_126de(
    cell_addr: u32,
    x_pos: u16,
    y_row: u16,
    block_type: i16,
) -> BlockType11HiddenEntryLoc126DE {
    // ASM loc_126DE (kid.asm:24969-24977): hidden block entry for block type 11.
    //   bsr.w sub_127A8        ; write $20 placeholder + terrain byte to cell
    //   bsr.w sub_127CE       ; shift hidden block list to make room
    //   move.w a4,(a2)+       ; store cell_addr in hidden block list
    //   move.w d1,(a2)+       ; store x_pos
    //   move.w d2,(a2)+       ; store y_row
    //   move.b d0,(a2)+       ; store block_type (negated, as byte)
    //   clr.b (a2)+           ; clear flag byte (0 = primary entry)
    //   bra.s loc_126BA       ; loop back for next entry
    // Creates a single hidden block entry. Block type 11 hidden blocks use a
    // single8-byte entry (unlike block type 4 which creates two entries).
    BlockType11HiddenEntryLoc126DE {
        cell_addr,
        x_pos,
        y_row,
        block_type,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct BlockType16ReadXPosLoc126F2 {
    pub(crate) x_pos: u16,
    pub(crate) bytes_consumed: usize,
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn block_type_16_read_x_pos_loc_126f2(
    layout_words: &[u16],
) -> Option<BlockType16ReadXPosLoc126F2> {
    // ASM loc_126F2 (kid.asm:24980-24984): entry point for block type 16.
    //   move.w (a1)+,d1    ; read x_pos
    //   bpl.s  loc_126F8   ; if >= 0, continue
    //   rts                ; else return (end of layout)
    // Block type 16 dispatches here from AddrTbl_BlockTypes entry 16.
    if layout_words.is_empty() {
        return None;
    }
    let x_pos = layout_words[0];
    if x_pos & 0x8000 != 0 {
        None
    } else {
        Some(BlockType16ReadXPosLoc126F2 {
            x_pos,
            bytes_consumed: 1,
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct BlockType16ParseCoordsLoc126F8 {
    pub(crate) cell_addr: u32,
    pub(crate) y_row: u16,
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn block_type_16_parse_coords_loc_126f8(
    layout_words: &[u16],
    row_pointer_table: &[u16],
    x_pos: u16,
) -> Option<BlockType16ParseCoordsLoc126F8> {
    // ASM loc_126F8 (kid.asm:24987-24996): coordinate parsing for block type 16.
    //   move.w (a1)+,d2       ; read Y row coordinate
    //   move.w d2,d3
    //   add.w d3,d3           ; d3 = Y * 2 (row table index)
    // ASM loc_126FE (kid.asm:24992-24996):
    //   lea ($FFFF4A04).l,a2  ; Level_Layout row pointer table
    //   move.w (a2,d3.w),a4   ; a4 = row_start_addr for Y
    //   add.w d1,a4
    //   add.w d1,a4           ; a4 = row_start + 2*X (cell address)
    if layout_words.is_empty() {
        return None;
    }
    let y_row = layout_words[0];
    let y_idx = usize::from(y_row) * 2;
    if y_idx >= row_pointer_table.len() * 2 {
        return None;
    }
    let row_start = u32::from(row_pointer_table[y_idx / 2]);
    let cell_addr = row_start.wrapping_add(u32::from(x_pos) * 2);
    Some(BlockType16ParseCoordsLoc126F8 { cell_addr, y_row })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct BlockType16SetupLoc126FE {
    pub(crate) count: u16,
    pub(crate) is_row_by_row: bool,
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn block_type_16_read_count_stride_loc_126fe(
    layout_words: &[u16],
) -> Option<BlockType16SetupLoc126FE> {
    // ASM loc_126FE (kid.asm:24997-25000): reads count and stride flag.
    //   move.w (a1)+,d3      ; read count
    //   subq.w #1,d3         ; d3 = count - 1 (for dbf loop)
    //   tst.w (a1)+          ; stride flag: 0 = horizontal, non-zero = row-by-row
    //   bne.s loc_12730      ; branch to row-by-row handler if stride != 0
    if layout_words.len() < 2 {
        return None;
    }
    let count = layout_words[0];
    let stride = layout_words[1];
    Some(BlockType16SetupLoc126FE {
        count,
        is_row_by_row: stride != 0,
    })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct BlockType16PropertyLookupLoc12714 {
    pub(crate) tile_index: u8,
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn block_type_16_property_lookup_loc_12714(
    property_word: u16,
) -> BlockType16PropertyLookupLoc12714 {
    // ASM loc_12714 (kid.asm:25002-25012): horizontal fill for block type 16.
    //   move.w (a1)+,d5                    ; read property index from layout
    //   move.b byte_1274E(pc,d5.w),d5      ; lookup in byte_1274E table
    //   lsl.b  #4,d5                       ; shift left by 4
    //   bsr.s  sub_12752                   ; handle special codes $40/$50
    //   addq.b #1,d5                       ; increment tile index
    //   bsr.w  sub_127A8                   ; write tile
    //   move.b d5,(a4)                     ; write to cell (low byte of tile)
    // ASM byte_1274E (kid.asm:25029-25032): property lookup table.
    //   dc.b 0, 5, 4, 6
    // Only the low byte of property_word is used as table index.
    const BYTE_1274E: [u8; 4] = [0, 5, 4, 6];
    let idx = (property_word as u8) as usize;
    let tile_base = if idx < BYTE_1274E.len() {
        BYTE_1274E[idx]
    } else {
        0
    };
    BlockType16PropertyLookupLoc12714 {
        tile_index: tile_base,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct HandlePropertyCode40Loc12752 {
    pub(crate) marks_diagonal_right: bool,
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn handle_property_code_40_loc_12752(property_byte: u8) -> HandlePropertyCode40Loc12752 {
    // ASM sub_12752 (kid.asm:25037-25050): handle property code $40.
    //   cmpi.b  #$40,d5
    //   bne.s   loc_12772               ; if not $40, try $50
    //   addq.w  #1,d1                   ; x_pos += 1 (move right)
    //   addq.w  #2,a4                   ; cell_addr += 2 (next cell)
    //   bsr.s   sub_12792               ; mark if in bounds
    //   subq.w  #2,a4                   ; restore cell_addr
    //   subq.w  #1,d1                   ; restore x_pos
    //   addq.w  #1,d2                   ; y_row += 1 (move down)
    //   add.w   (Level_width_tiles).w,a4 ; move down one row
    //   bsr.s   sub_12792               ; mark if in bounds
    //   suba.w  (Level_width_tiles).w,a4 ; restore
    //   subq.w  #1,d2                   ; restore y_row
    //   rts
    // Property code $40 (from byte_1274E[2]=4, shifted to $40) marks diagonal down-right.
    // Returns whether this code should trigger diagonal marking.
    HandlePropertyCode40Loc12752 {
        marks_diagonal_right: property_byte == 0x40,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct HandlePropertyCode50Loc12772 {
    pub(crate) marks_diagonal_left: bool,
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn handle_property_code_50_loc_12772(property_byte: u8) -> HandlePropertyCode50Loc12772 {
    // ASM loc_12772 (kid.asm:25053-25068): handle property code $50.
    //   cmpi.b  #$50,d5
    //   bne.s   return_12790            ; if not $50, return immediately
    //   subq.w  #1,d1                   ; x_pos -= 1 (move left)
    //   subq.w  #2,a4                   ; cell_addr -= 2 (previous cell)
    //   bsr.s   sub_12792               ; mark if in bounds
    //   addq.w  #2,a4                   ; restore cell_addr
    //   addq.w  #1,d1                   ; restore x_pos
    //   addq.w  #1,d2                   ; y_row += 1 (move down)
    //   add.w   (Level_width_tiles).w,a4 ; move down one row
    //   bsr.s   sub_12792               ; mark if in bounds
    //   suba.w  (Level_width_tiles).w,a4 ; restore
    //   subq.w  #1,d2                   ; restore y_row
    //   rts
    // Property code $50 (from byte_1274E[1]=5, shifted to $50) marks diagonal down-left.
    // Returns whether this code should trigger diagonal marking.
    HandlePropertyCode50Loc12772 {
        marks_diagonal_left: property_byte == 0x50,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct BoundsCheckMarkLoc12792 {
    pub(crate) in_bounds: bool,
    pub(crate) should_mark: bool,
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn bounds_check_mark_loc_12792(
    x_pos: u16,
    y_row: u16,
    width_blocks: u16,
    height_blocks: u16,
    cell_value: i8,
) -> BoundsCheckMarkLoc12792 {
    // ASM sub_12792 (kid.asm:25075-25085): bounds check and mark cell.
    //   cmp.w   (Level_width_blocks).w,d1   ; if x >= width_blocks
    //   bcc.s   return_127A6                ; return (out of bounds)
    //   cmp.w   (Level_height_blocks).w,d2  ; if y >= height_blocks
    //   bcc.s   return_127A6                ; return (out of bounds)
    //   tst.b   (a4)                         ; test cell value
    //   bmi.s   return_127A6                ; if negative (already special), return
    //   bset    #0,(a4)                      ; set bit 0 to mark cell
    //   rts
    // Returns whether position is in bounds and cell can be marked.
    let in_bounds = x_pos < width_blocks && y_row < height_blocks;
    let should_mark = in_bounds && cell_value >= 0;
    BoundsCheckMarkLoc12792 {
        in_bounds,
        should_mark,
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct HiddenBlockFindInsertionLoc127DA {
    pub(crate) insertion_index: usize,
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn hidden_block_find_insertion_loc_127da(
    cell_addrs: &[u16],
    target_addr: u16,
) -> HiddenBlockFindInsertionLoc127DA {
    // ASM loc_127DA (kid.asm:25116-25120): search for insertion position in hidden block list.
    //   loc_127DA:
    //   cmp.w   (a0),d0         ; compare cell address with entry
    //   bcs.s   loc_127E2       ; if current < entry, found position
    //   addq.w  #8,a0           ; else advance to next entry (8 bytes each)
    //   bra.s   loc_127DA       ; continue loop
    // The list is sorted by cell address. Find first entry >= target_addr.
    // Returns index where new entry should be inserted.
    let insertion_index = cell_addrs
        .iter()
        .position(|&addr| target_addr < addr)
        .unwrap_or(cell_addrs.len());
    HiddenBlockFindInsertionLoc127DA { insertion_index }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct HiddenBlockShiftInsertLoc127E2 {
    pub(crate) shifted_addrs: Vec<u16>,
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn hidden_block_shift_insert_loc_127e2(
    cell_addrs: &[u16],
    insert_index: usize,
    new_addr: u16,
) -> HiddenBlockShiftInsertLoc127E2 {
    // ASM loc_127E2 (kid.asm:25123-25138): shift entries and insert new one.
    //   move.l  a0,a2                   ; save insertion point
    //   move.w  ($FFFFF8EA).w,d0        ; d0 = hidden_block_count
    //   lsl.w   #3,d0                   ; d0 = count * 8 (bytes per entry)
    //   add.w   d0,a1                   ; a1 = end of list
    //   move.w  a1,d0
    //   sub.w   a0,d0                   ; d0 = distance to shift
    //   lsr.w   #3,d0                   ; d0 = number of entries to shift
    //   addq.w  #8,a1                   ; a1 = dest + 8 (make room for new entry)
    //   lea     8(a1),a0                ; a0 = dest + 16 (destination for shift)
    // loc_127F8:                            ; shift loop
    //   move.l  -(a1),-(a0)             ; copy entry (2 longwords = 8 bytes)
    //   move.l  -(a1),-(a0)
    //   dbf     d0,loc_127F8
    //   addq.w  #1,($FFFFF8EA).w       ; increment count
    // Returns the list with new entry inserted at the correct position.
    let mut shifted_addrs = cell_addrs.to_vec();
    shifted_addrs.insert(insert_index, new_addr);
    HiddenBlockShiftInsertLoc127E2 { shifted_addrs }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct BackgroundMappingsCrossLevelLoc12824 {
    pub(crate) x_offset: i16,
    pub(crate) y_offset: i16,
    pub(crate) source_layout_addr: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct BackgroundMappingsChunkLoc12830 {
    pub(crate) chunk_index: u16,
    pub(crate) x_pos: i16,
    pub(crate) y_pos: i16,
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn background_mappings_cross_level_loc_12824(
    layout_words: &[u16],
) -> Option<BackgroundMappingsCrossLevelLoc12824> {
    // ASM loc_12824 (kid.asm:25158-25164): cross-level layout copy handler.
    // When the layout starts with $8000 marker, this branch reads offset/pointer:
    //   movem.l d5-d7,-(sp)      ; save registers
    //   addq.w  #2,a1            ; skip $8000 marker word
    //   move.w  (a1)+,d5         ; x offset for copying from other level
    //   move.w  (a1)+,d6         ; y offset for copying from other level
    //   move.l  (a1),a1          ; load pointer to source level layout
    // After this preparation, execution falls through to loc_12830.
    // The x/y offsets are subtracted from positions read from the source layout,
    // effectively translating coordinates from source level to destination level.
    let marker = *layout_words.first()?;
    if marker != 0x8000 {
        return None;
    }
    let x_offset = layout_words.get(1).copied()? as i16;
    let y_offset = layout_words.get(2).copied()? as i16;
    let source_addr =
        u32::from(layout_words.get(4).copied()?) << 16 | u32::from(layout_words.get(3).copied()?);
    Some(BackgroundMappingsCrossLevelLoc12824 {
        x_offset,
        y_offset,
        source_layout_addr: source_addr,
    })
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn background_mappings_read_chunk_loc_12830(
    layout_words: &[u16],
    x_offset: i16,
    y_offset: i16,
) -> Option<(BackgroundMappingsChunkLoc12830, &[u16])> {
    // ASM loc_12830 (kid.asm:25165-25173): background chunk placement loop.
    // Reads sequential (chunk_index, x_pos, y_pos) entries until terminator:
    //   loc_12830:
    //   move.w  (a1)+,d0        ; read chunk index
    //   bmi.s   loc_12842       ; terminate if negative (bit 15 set)
    //   move.w  (a1)+,d2        ; read x position
    //   sub.w   d5,d2           ; apply x_offset (d5 from loc_12824 or 0)
    //   move.w  (a1)+,d3        ; read y position
    //   sub.w   d6,d3           ; apply y_offset (d6 from loc_12824 or 0)
    //   bsr.w   Load_BackgroundChunk
    //   bra.s   loc_12830       ; continue loop
    // The terminator is any word with bit 15 set ($8000+).
    // Each chunk placement consumes 3 words: index, x, y.
    // Offsets are applied when copying from another level (loc_12824 path).
    let chunk_index = layout_words.first().copied()?;
    if chunk_index & 0x8000 != 0 {
        return None; // terminator, bit 15 set
    }
    let x_pos = (layout_words.get(1).copied()? as i16).wrapping_sub(x_offset);
    let y_pos = (layout_words.get(2).copied()? as i16).wrapping_sub(y_offset);
    let remaining = &layout_words[3..];
    Some((
        BackgroundMappingsChunkLoc12830 {
            chunk_index,
            x_pos,
            y_pos,
        },
        remaining,
    ))
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn background_mappings_terminator_loc_12842() -> bool {
    // ASM loc_12842 (kid.asm:25176-25178): loop termination.
    //   movem.l (sp)+,d5-d7    ; restore saved registers
    //   rts                     ; return to caller
    // This is reached when loc_12830 reads a negative chunk index.
    // The terminator word has bit 15 set ($8000+).
    true
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct BackgroundChunkCopyLoc12884 {
    pub(crate) tiles_copied: usize,
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn background_chunk_row_loop_loc_12884(
    chunk_width: u16,
    start_x: i16,
    start_y: i16,
    background_width: u16,
    background_height: u16,
    chunk_tiles: &[u8],
) -> BackgroundChunkCopyLoc12884 {
    // ASM loc_12884 (kid.asm:25214-25218): outer loop setup for chunk row copying.
    //   loc_12884:
    //   move.w  d2,d4           ; d4 = chunk_x_size - 1 (inner loop counter)
    //   move.l  a2,a4           ; a4 = current row start in bg buffer
    //   move.w  d7,d5           ; d5 = x position (starting column)
    // Falls through to loc_1288A for inner column loop.
    // Each row: copy `chunk_width` tiles with bounds checking, advance to next row.
    let mut tiles_copied = 0usize;
    let mut tile_index = 0usize;
    for y_offset in 0..chunk_width {
        let y = start_y + y_offset as i16;
        for x_offset in 0..chunk_width {
            if tile_index >= chunk_tiles.len() {
                break;
            }
            // ASM loc_1288A (kid.asm:25219-25234): inner loop with bounds checking.
            //   loc_1288A:
            //   tst.w   d5              ; check if x < 0
            //   bmi.s   loc_128A0       ; skip if negative
            //   cmp.w   (Background_width).w,d5  ; check if x >= width
            //   bge.s   loc_128A0       ; skip if out of bounds
            //   tst.w   d6              ; check if y < 0
            //   bmi.s   loc_128A0       ; skip if negative
            //   cmp.w   (Background_height).w,d6 ; check if y >= height
            //   bge.s   loc_128A0       ; skip if out of bounds
            //   move.b  (a3),(a4)       ; copy tile if within bounds
            //   loc_128A0:
            //   addq.w  #1,a3           ; advance source pointer
            //   addq.w  #1,a4           ; advance dest pointer
            //   addq.w  #1,d5           ; advance x column
            //   dbf     d4,loc_1288A    ; inner loop
            let x = start_x + x_offset as i16;
            if x >= 0 && (x as u16) < background_width && y >= 0 && (y as u16) < background_height {
                tiles_copied += 1;
            }
            tile_index += 1;
        }
    }
    BackgroundChunkCopyLoc12884 { tiles_copied }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct BackgroundMountainLoadLoc12910 {
    pub(crate) pattern_vram_offset: u16,
    pub(crate) vram_buffer1_addr: u32,
    pub(crate) vram_buffer2_addr: u32,
    pub(crate) vram_buffer3_addr: u32,
    pub(crate) needs_hail_overlay_at_1294_a: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct BackgroundHailOverlayLoc1294A {
    pub(crate) pattern_vram_offset: u16,
    pub(crate) vram_buffer4_addr: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct BackgroundFinalPatternLoc12952 {
    pub(crate) pattern_vram_offset: u16,
    pub(crate) vram_buffer4_addr: u32,
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn background_mountain_tileset_loc_12910(
    level_special_effects: u16,
) -> BackgroundMountainLoadLoc12910 {
    // ASM loc_12910 (kid.asm:25274-25291): Mountain theme background tileset load.
    //   loc_12910:	; mountain
    //   move.w  #$2780,d0           ; pattern VRAM offset for mountain bg
    //   move.l  (a3)+,a0            ; load first art pointer
    //   lea     ($FFFF87B2).w,a1    ; dest: VRAM buffer 1 ($87B2)
    //   bsr.w   EniDec              ; decompress
    //   move.l  (a3)+,a0            ; load second art pointer
    //   lea     ($FFFF8EB2).w,a1    ; dest: VRAM buffer 2 ($8EB2)
    //   bsr.w   EniDec              ; decompress
    //   move.l  (a3)+,a0            ; load third art pointer
    //   lea     ($FFFF95B2).w,a1    ; dest: VRAM buffer 3 ($95B2)
    //   bsr.w   EniDec              ; decompress
    //   cmpi.w  #2,(Level_Special_Effects).w  ; stormy weather?
    //   beq.s   loc_1294A           ; hail overlay path
    //   cmpi.w  #3,(Level_Special_Effects).w  ; storm + hail?
    //   beq.s   loc_1294A           ; hail overlay path
    //   move.w  #$7E5,d0           ; non-storm 4th pattern
    //   move.l  (a3)+,a0            ; load 4th art pointer
    //   bra.s   loc_12952           ; final load
    const BACKGROUND_THEME_MOUNTAIN_PATTERN_LOC_12910: u16 = 0x2780;
    const VRAM_BUFFER_1_ADDR: u32 = 0xFFFF_87B2;
    const VRAM_BUFFER_2_ADDR: u32 = 0xFFFF_8EB2;
    const VRAM_BUFFER_3_ADDR: u32 = 0xFFFF_95B2;
    const WEATHER_ID_STORM_LOC_12910: u16 = 0x0002;
    const WEATHER_ID_STORM_AND_HAIL_LOC_12910: u16 = 0x0003;

    let needs_hail_overlay = level_special_effects == WEATHER_ID_STORM_LOC_12910
        || level_special_effects == WEATHER_ID_STORM_AND_HAIL_LOC_12910;

    BackgroundMountainLoadLoc12910 {
        pattern_vram_offset: BACKGROUND_THEME_MOUNTAIN_PATTERN_LOC_12910,
        vram_buffer1_addr: VRAM_BUFFER_1_ADDR,
        vram_buffer2_addr: VRAM_BUFFER_2_ADDR,
        vram_buffer3_addr: VRAM_BUFFER_3_ADDR,
        needs_hail_overlay_at_1294_a: needs_hail_overlay,
    }
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn background_hail_overlay_loc_1294A() -> BackgroundHailOverlayLoc1294A {
    // ASM loc_1294A (kid.asm:25294-25296): Hail overlay pattern for stormy weather.
    //   loc_1294A:	; hailstorm
    //   move.w  #$27E5,d0           ; hail pattern offset
    //   move.l  4(a3),a0           ; load art from offset +4 (skip one pointer)
    // Falls through to loc_12952 for final decompress.
    const HAIL_PATTERN_VRAM_OFFSET_LOC_1294A: u16 = 0x27E5;
    const VRAM_BUFFER_4_ADDR: u32 = 0xFFFF_98F2;

    BackgroundHailOverlayLoc1294A {
        pattern_vram_offset: HAIL_PATTERN_VRAM_OFFSET_LOC_1294A,
        vram_buffer4_addr: VRAM_BUFFER_4_ADDR,
    }
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn background_final_pattern_loc_12952(
    pattern_vram_offset: u16,
) -> BackgroundFinalPatternLoc12952 {
    // ASM loc_12952 (kid.asm:25298-25301): Final pattern decompress for mountain bg.
    //   loc_12952:
    //   lea     ($FFFF98F2).w,a1    ; dest: VRAM buffer 4 ($98F2)
    //   bsr.w   EniDec              ; decompress final pattern
    //   move.l  (sp)+,a0            ; restore return address
    //   rts                         ; return
    const VRAM_BUFFER_4_ADDR: u32 = 0xFFFF_98F2;

    BackgroundFinalPatternLoc12952 {
        pattern_vram_offset,
        vram_buffer4_addr: VRAM_BUFFER_4_ADDR,
    }
}

#[cfg_attr(test, allow(dead_code))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct BackgroundHillsLoadLoc1295E {
    pub(crate) pattern_vram_offset_1_2: u16,
    pub(crate) pattern_vram_offset_3: u16,
    pub(crate) vram_buffer1_addr: u32,
    pub(crate) vram_buffer2_addr: u32,
    pub(crate) vram_buffer3_addr: u32,
}

#[allow(non_snake_case)]
#[allow(dead_code)]
pub(crate) fn background_hills_tileset_loc_1295E() -> BackgroundHillsLoadLoc1295E {
    // ASM loc_1295E (kid.asm:25305-25318): Hills theme background tileset load.
    //   loc_1295E:	; hill
    //   move.w  #$2780,d0           ; pattern VRAM offset for first two buffers
    //   move.l  (a3)+,a0            ; load first art pointer
    //   lea     ($FFFF87B2).w,a1    ; dest: VRAM buffer 1 ($87B2)
    //   bsr.w   EniDec              ; decompress
    //   move.l  (a3)+,a0            ; load second art pointer
    //   lea     ($FFFF8EB2).w,a1    ; dest: VRAM buffer 2 ($8EB2)
    //   bsr.w   EniDec              ; decompress
    //   move.w  #$7C0,d0            ; pattern VRAM offset for third buffer
    //   move.l  (a3)+,a0            ; load third art pointer
    //   lea     ($FFFF95B2).w,a1    ; dest: VRAM buffer 3 ($95B2)
    //   bsr.w   EniDec              ; decompress
    //   move.l  (sp)+,a0            ; restore return address
    //   rts                         ; return
    const HILLS_PATTERN_1_2_LOC_1295E: u16 = 0x2780;
    const HILLS_PATTERN_3_LOC_1295E: u16 = 0x07C0;
    const VRAM_BUFFER_1_ADDR: u32 = 0xFFFF_87B2;
    const VRAM_BUFFER_2_ADDR: u32 = 0xFFFF_8EB2;
    const VRAM_BUFFER_3_ADDR: u32 = 0xFFFF_95B2;

    BackgroundHillsLoadLoc1295E {
        pattern_vram_offset_1_2: HILLS_PATTERN_1_2_LOC_1295E,
        pattern_vram_offset_3: HILLS_PATTERN_3_LOC_1295E,
        vram_buffer1_addr: VRAM_BUFFER_1_ADDR,
        vram_buffer2_addr: VRAM_BUFFER_2_ADDR,
        vram_buffer3_addr: VRAM_BUFFER_3_ADDR,
    }
}

#[cfg_attr(test, allow(dead_code))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct BackgroundDesertLoadLoc12988 {
    pub(crate) pattern_vram_offset: u16,
    pub(crate) vram_buffer1_addr: u32,
    pub(crate) vram_buffer2_addr: u32,
    pub(crate) vram_buffer3_addr: u32,
}

#[allow(non_snake_case)]
#[allow(dead_code)]
pub(crate) fn background_desert_tileset_loc_12988() -> BackgroundDesertLoadLoc12988 {
    // ASM loc_12988 (kid.asm:25321-25333): Desert theme background tileset load.
    //   loc_12988:	; desert
    //   move.w  #$2780,d0           ; pattern VRAM offset for all buffers
    //   move.l  (a3)+,a0            ; load first art pointer
    //   lea     ($FFFF87B2).w,a1    ; dest: VRAM buffer 1 ($87B2)
    //   bsr.w   EniDec              ; decompress
    //   move.l  (a3)+,a0            ; load second art pointer
    //   lea     ($FFFF8EB2).w,a1    ; dest: VRAM buffer 2 ($8EB2)
    //   bsr.w   EniDec              ; decompress
    //   move.l  (a3)+,a0            ; load third art pointer
    //   lea     ($FFFF95B2).w,a1    ; dest: VRAM buffer 3 ($95B2)
    //   bsr.w   EniDec              ; decompress
    //   move.l  (sp)+,a0            ; restore return address
    //   rts                         ; return
    const DESERT_PATTERN_LOC_12988: u16 = 0x2780;
    const VRAM_BUFFER_1_ADDR: u32 = 0xFFFF_87B2;
    const VRAM_BUFFER_2_ADDR: u32 = 0xFFFF_8EB2;
    const VRAM_BUFFER_3_ADDR: u32 = 0xFFFF_95B2;

    BackgroundDesertLoadLoc12988 {
        pattern_vram_offset: DESERT_PATTERN_LOC_12988,
        vram_buffer1_addr: VRAM_BUFFER_1_ADDR,
        vram_buffer2_addr: VRAM_BUFFER_2_ADDR,
        vram_buffer3_addr: VRAM_BUFFER_3_ADDR,
    }
}

#[cfg_attr(test, allow(dead_code))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct JuggernautSkullArtDmaLoc129DC {
    pub(crate) vdp_command: u32,
    pub(crate) art_source: u32,
    pub(crate) longword_count: u16,
}

#[allow(non_snake_case)]
#[allow(dead_code)]
pub(crate) fn juggernaut_skull_art_dma_loop_loc_129DC() -> JuggernautSkullArtDmaLoc129DC {
    // ASM loc_129DC (kid.asm:25359-25361): Juggernaut skull frame 1 DMA loop.
    //   loc_129DC:
    //   move.l  (a1)+,(a6)          ; copy longword from ArtUnc_12A08 to VDP
    //   dbf     d0,loc_129DC        ; loop 32 times ($1F+1 iterations)
    // This is the first of three DMA loops in sub_129CE that load:
    //   - ArtUnc_12A08 (Juggernaut skull frame 1) -> $CDC0
    //   - ArtUnc_12A8C (Juggernaut skull frame 2) -> $CDC0 (continuation, 32 more longwords)
    //   - ArtUnc_12B0C (Some kind of star)         -> $DF40
    const VDP_COMMAND: u32 = 0xC000_00CD; // vdpComm($CDC0,VRAM,WRITE)
    const ART_SOURCE_OFFSET: u32 = 0x0001_2A08; // ArtUnc_12A08
    const LONGWORD_COUNT: u16 = 32; // $1F + 1

    JuggernautSkullArtDmaLoc129DC {
        vdp_command: VDP_COMMAND,
        art_source: ART_SOURCE_OFFSET,
        longword_count: LONGWORD_COUNT,
    }
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn juggernaut_skull_art_dma_loop_loc_129E8() -> JuggernautSkullArtDmaLoc129DC {
    // ASM loc_129E8 (kid.asm:25365-25367): second `dbf` loop in sub_129CE.
    // It keeps the same `$CDC0` VRAM write command and copies another
    // `$1F+1` longwords, but `a1` now points at `ArtUnc_12A8C`.
    const VDP_COMMAND: u32 = 0xC000_00CD; // vdpComm($CDC0,VRAM,WRITE)
    const ART_SOURCE_OFFSET: u32 = 0x0001_2A8C; // ArtUnc_12A8C
    const LONGWORD_COUNT: u16 = 32; // $1F + 1

    JuggernautSkullArtDmaLoc129DC {
        vdp_command: VDP_COMMAND,
        art_source: ART_SOURCE_OFFSET,
        longword_count: LONGWORD_COUNT,
    }
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn juggernaut_star_art_dma_loop_loc_129FC() -> JuggernautSkullArtDmaLoc129DC {
    // ASM loc_129FC (kid.asm:25372-25374): final `dbf` loop in sub_129CE.
    // This pass switches the VDP destination to `$DF40` and streams
    // `ArtUnc_12B0C` for another `$1F+1` longwords.
    const VDP_COMMAND: u32 = 0x4000_00DF; // vdpComm($DF40,VRAM,WRITE)
    const ART_SOURCE_OFFSET: u32 = 0x0001_2B0C; // ArtUnc_12B0C
    const LONGWORD_COUNT: u16 = 32; // $1F + 1

    JuggernautSkullArtDmaLoc129DC {
        vdp_command: VDP_COMMAND,
        art_source: ART_SOURCE_OFFSET,
        longword_count: LONGWORD_COUNT,
    }
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn flag_bottom_art_dma_loop_loc_12B9C() -> JuggernautSkullArtDmaLoc129DC {
    // ASM loc_12B9C (kid.asm:25398-25400): `Load_FlagBottomArt` writes
    // `vdpComm($D2C0,VRAM,WRITE)` to the VDP control port, points `a1` at
    // `ArtUnc_12BA4`, seeds `d0=#1F`, then streams 32 longwords via
    // `move.l (a1)+,(a6)` / `dbf d0,loc_12B9C`.
    const VDP_COMMAND: u32 = 0xC000_00D2; // vdpComm($D2C0,VRAM,WRITE)
    const ART_SOURCE_OFFSET: u32 = 0x0001_2BA4; // ArtUnc_12BA4
    const LONGWORD_COUNT: u16 = 32; // $1F + 1

    JuggernautSkullArtDmaLoc129DC {
        vdp_command: VDP_COMMAND,
        art_source: ART_SOURCE_OFFSET,
        longword_count: LONGWORD_COUNT,
    }
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn hitpoint_display_art_dma_loop_loc_12C34() -> JuggernautSkullArtDmaLoc129DC {
    // ASM loc_12C34 (kid.asm:25414-25416): `sub_12C24` writes
    // `vdpComm($45A0,VRAM,WRITE)` to the VDP control port, points `a1` at
    // `ArtUnc_12C3C`, seeds `d0=#1F`, then streams 32 longwords via
    // `move.l (a1)+,(a6)` / `dbf d0,loc_12C34`.
    const VDP_COMMAND: u32 = 0xA000_00C5; // vdpComm($45A0,VRAM,WRITE)
    const ART_SOURCE_OFFSET: u32 = 0x0001_2C3C; // ArtUnc_12C3C
    const LONGWORD_COUNT: u16 = 32; // $1F + 1

    JuggernautSkullArtDmaLoc129DC {
        vdp_command: VDP_COMMAND,
        art_source: ART_SOURCE_OFFSET,
        longword_count: LONGWORD_COUNT,
    }
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn fg_plane_custom_write_guard_loc_11CE0(
    fg_plane_custom_write_addr: u32,
    fg_plane_custom_write_count: u16,
) -> bool {
    // ASM loc_11CE0 (kid.asm:23886-23892) skips the guard when
    // `($FFFFF8E0).w` is zero; otherwise it multiplies `($FFFFF8DE).w` by
    // `$A`, adds that span to the low word of the pointer, and branches to
    // `loc_11D0A` once the result reaches `FGPlane_UpdateQueueCustom = $43BC`.
    fg_queue_guard_low_word_loc_11_cb6(fg_plane_custom_write_addr, fg_plane_custom_write_count * 10)
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn background_tile_guard_loc_11CF6(
    background_tile_addr: u32,
    background_tile_count: u16,
) -> bool {
    // ASM loc_11CF6 (kid.asm:23895-23901) skips the guard when
    // `($FFFFF8EC).w` is zero; otherwise it multiplies `($FFFFF8EA).w` by
    // eight with `lsl.w #3,d1`, adds that span to the low word of the
    // pointer, and only falls through to `loc_11D0C` while the result stays
    // below `FGPlane_UpdateQueueCustom = $43BC`.
    fg_queue_guard_low_word_loc_11_cb6(background_tile_addr, background_tile_count << 3)
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn level_load_pointers_loc_11D0C(
    theme_mapping_index: &[u32],
    theme_mapping_table_offset_bytes: u16,
    enemy_layout_rom_addr: u32,
    enemy_layout_header_words: &[u32],
) -> LevelLoadPointersLoc11D0C {
    // ASM loc_11D0C (kid.asm:23908-23916) reloads `a1` from
    // `LnkTo_ThemeMappings_Index`, reads one longword through `(a1,d7.w)` into
    // `Addr_ThemeMappings`, then reloads `a1` from `(a0)+` and uses
    // `move.l (a1)+,(Addr_EnemyLayout).w` followed by `move.l a1,(Addr_EnemyLayoutHeader).w`.
    assert!(
        theme_mapping_table_offset_bytes & 3 == 0,
        "theme mapping offset must stay aligned to the ROM's longword table",
    );
    let theme_mapping_index_entry = usize::from(theme_mapping_table_offset_bytes >> 2);
    let enemy_layout_addr = *enemy_layout_header_words
        .first()
        .expect("enemy layout header must contain the leading longword");
    LevelLoadPointersLoc11D0C {
        theme_mappings_addr: theme_mapping_index[theme_mapping_index_entry],
        enemy_layout_addr,
        enemy_layout_header_addr: enemy_layout_rom_addr.wrapping_add(4),
    }
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn block_mapping_or_mask_loc_11D42(high_priority_flag: bool) -> u32 {
    // ASM loc_11D0C -> loc_11D42 (kid.asm:23917-23923) clears `d1`, tests
    // `($FFFFF896).w`, and only loads `$80008000` into `d1` when that byte is
    // non-zero before falling through into the block-mapping copy loop.
    if high_priority_flag { 0x8000_8000 } else { 0 }
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn copy_block_mappings_loc_11D46(source_longwords: &[u32], or_mask: u32) -> Vec<u32> {
    // ASM loc_11D42/loc_11D46 (kid.asm:23923-23928) seeds `d0` with `$A1`, so
    // `dbf d0,loc_11D46` copies exactly `$A2` longwords from `(a1)+` to
    // `Block_Mappings`, OR-ing each longword with the mask prepared in `d1`.
    const BLOCK_MAPPING_LONGWORD_COUNT_LOC_11D46: usize = 0xA2;
    assert!(
        source_longwords.len() >= BLOCK_MAPPING_LONGWORD_COUNT_LOC_11D46,
        "block mapping copy needs at least 162 source longwords",
    );
    source_longwords
        .iter()
        .copied()
        .take(BLOCK_MAPPING_LONGWORD_COUNT_LOC_11D46)
        .map(|longword| longword | or_mask)
        .collect()
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn background_dimensions_loc_11B40(
    level_width_tiles: u16,
    level_height_tiles: u16,
) -> (u16, u16) {
    // ASM loc_11B40's shared tail (kid.asm:23728-23736) derives
    // `Background_width` as `(Level_width_tiles >> 2) + $1E` and
    // `Background_height` as `(Level_height_tiles >> 2) + $15`.
    (
        (level_width_tiles >> 2).wrapping_add(0x001E),
        (level_height_tiles >> 2).wrapping_add(0x0015),
    )
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn background_row_pointers_loc_11B7A(
    background_height: u16,
    background_width: u16,
) -> Vec<u16> {
    // ASM loc_11B7A (kid.asm:23742-23745) seeds `a1` with `$FFFF87B2`, then
    // emits one row-start pointer word per background row into `$FFFF4D5C+`.
    // The loop advances `a1` by `Background_width`, preserving the ROM's
    // 16-bit low-word pointer arithmetic.
    let mut row_pointer = BACKGROUND_ROW_POINTER_BASE_LOC_11B7A;
    let mut row_pointers = Vec::with_capacity(usize::from(background_height));
    for _ in 0..background_height {
        row_pointers.push(row_pointer);
        row_pointer = row_pointer.wrapping_add(background_width);
    }
    row_pointers
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn decode_foreground_theme_flags_loc_11B92(
    raw_foreground_theme_flags: u8,
) -> (u16, bool, bool) {
    // ASM loc_11B92 (kid.asm:23753-23755) only enters the murder-wall path
    // after `bpl.s loc_11B9A` fails, so the sign bit gates the flag decode.
    // Bit 6 only means "reversed" once the sign bit already enabled the wall;
    // otherwise the positive byte flows through to `Foreground_theme` intact.
    if raw_foreground_theme_flags & 0x80 == 0 {
        return (u16::from(raw_foreground_theme_flags), false, false);
    }

    (
        u16::from(raw_foreground_theme_flags & 0x3F),
        true,
        raw_foreground_theme_flags & 0x40 != 0,
    )
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn decode_background_theme_flags_loc_11B9A(
    raw_background_theme_flags: u8,
    background_width: u16,
) -> (u16, u16, bool, u16) {
    // ASM loc_11B9A (kid.asm:23757-23775) stores the upper nibble in
    // `Level_Special_Effects`, the lower nibble in `Background_theme`, then
    // tests `(1<<Hills)|(1<<Desert)|(1<<Mountain)|(1<<Forest)` to select the
    // enigma background format and force `Background_width=$40`.
    let level_special_effects = u16::from(raw_background_theme_flags >> 4);
    let background_theme = u16::from(raw_background_theme_flags & 0x0F);
    let uses_enigma_format = matches!(background_theme, 3 | 5 | 7 | 9);
    let decoded_background_width = if uses_enigma_format {
        0x0040
    } else {
        background_width
    };
    (
        level_special_effects,
        background_theme,
        uses_enigma_format,
        decoded_background_width,
    )
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn player_start_position_loc_11BE4(header_player_start: (u16, u16)) -> (u16, u16) {
    // ASM loc_11BE4 (kid.asm:23785-23788): when `loc_11BD2` leaves the header
    // pointer at the player-start words, `move.w (a0)+` loads exactly the X
    // and Y start coordinates before execution falls through to `loc_11BEC`.
    header_player_start
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn decode_spawn_positions_loc_11BD2(
    preserve_existing_player_start: bool,
    existing_player_start: (u16, u16),
    header_player_start: (u16, u16),
    flag_position: (u16, u16),
) -> ((u16, u16), (u16, u16), bool) {
    // ASM loc_11BD2 (kid.asm:23777-23782) tests the `$FFFFFC29` latch before
    // reading `PlayerStart_X/Y_pos`. When set, it skips those two header words
    // with `addq.w #4,a0`, clears the latch, and still falls through to load
    // `Flag_X/Y_pos` from the following words.
    let player_start = if preserve_existing_player_start {
        existing_player_start
    } else {
        player_start_position_loc_11BE4(header_player_start)
    };
    (player_start, flag_position_loc_11BEC(flag_position), false)
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn flag_position_loc_11BEC(flag_position: (u16, u16)) -> (u16, u16) {
    // ASM loc_11BEC (kid.asm:23789-23792): regardless of whether `loc_11BD2`
    // skipped the player-start pair, the next two header words always become
    // `Flag_X_pos` and `Flag_Y_pos`.
    flag_position
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ForegroundPaletteOverrideLoc11D8E {
    None,
    AlternativeCave,
    RemovedSpecialPaletteNoop,
    LateAlternativePalette,
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn classify_foreground_palette_override_loc_11D8E(
    map_id: u8,
) -> ForegroundPaletteOverrideLoc11D8E {
    // ASM loc_11D8E (kid.asm:23951): after the default theme palette lookup,
    // Stairway to Oblivion (`#3`) and Elsewhere 23 (`#23`) replace `a1` with
    // the alternative cave palette before the later override checks run.
    if matches!(map_id, 0x03 | 23) {
        return ForegroundPaletteOverrideLoc11D8E::AlternativeCave;
    }
    // ASM loc_11D92 (kid.asm:23954): The Deadly Skyscrapers (`#$1C`) and
    // Elsewhere 30 (`#30`) branch into `loc_11D9E`, where the removed special
    // palette command now leaves the already selected theme palette intact.
    if matches!(map_id, 0x1C | 30) {
        return ForegroundPaletteOverrideLoc11D8E::RemovedSpecialPaletteNoop;
    }
    // ASM loc_11DA0 (kid.asm:23968): after the earlier cave/skyscraper
    // checks, The Crypt leftover (`#$1A`) and Alien Twilight (`#$1B`) branch
    // to `loc_11DAC`, which swaps in the later alternative foreground palette.
    if matches!(map_id, 0x1A | 0x1B) {
        return ForegroundPaletteOverrideLoc11D8E::LateAlternativePalette;
    }
    ForegroundPaletteOverrideLoc11D8E::None
}

#[allow(non_snake_case)]
#[allow(dead_code)]
pub(crate) fn apply_foreground_palette_override_loc_11D9E(
    selected_theme_palette: u16,
    override_kind: ForegroundPaletteOverrideLoc11D8E,
) -> u16 {
    // ASM loc_11D9E (kid.asm:23960): the removed `lea Pal_12D08(pc),a1`
    // command was replaced with `nop`, so both the explicit no-op path and
    // the ordinary fallthrough keep the already selected theme palette.
    match override_kind {
        ForegroundPaletteOverrideLoc11D8E::None
        | ForegroundPaletteOverrideLoc11D8E::RemovedSpecialPaletteNoop => selected_theme_palette,
        ForegroundPaletteOverrideLoc11D8E::AlternativeCave
        | ForegroundPaletteOverrideLoc11D8E::LateAlternativePalette => selected_theme_palette,
    }
}

#[allow(non_snake_case)]
#[allow(dead_code)]
pub(crate) fn apply_foreground_palette_override_loc_11DAC(
    selected_theme_palette: u16,
    late_alternative_palette: u16,
    override_kind: ForegroundPaletteOverrideLoc11D8E,
) -> u16 {
    // ASM loc_11DAC (kid.asm:23974): the taken `$1A/$1B` branch replaces `a1`
    // with `Pal_12D26(pc)` immediately before the shared copy loop at
    // `loc_11DB0`; all other paths keep the palette already selected above.
    match override_kind {
        ForegroundPaletteOverrideLoc11D8E::LateAlternativePalette => late_alternative_palette,
        ForegroundPaletteOverrideLoc11D8E::None
        | ForegroundPaletteOverrideLoc11D8E::AlternativeCave
        | ForegroundPaletteOverrideLoc11D8E::RemovedSpecialPaletteNoop => selected_theme_palette,
    }
}

#[allow(non_snake_case)]
#[allow(dead_code)]
pub(crate) fn copy_foreground_palette_loc_11DB0(
    foreground_palette: &[u16; 15],
    palette_buffer: &mut [u16; 0x40],
) {
    // ASM loc_11DB0 (kid.asm:23977): `lea (Palette_Buffer+$2).l,a2` and
    // `moveq #$E,d1` feed a `move.w (a1)+,(a2)+` / `dbf d1,-` loop, so the ROM
    // copies exactly 15 words into palette slots 1..15 and leaves word 0 plus
    // the remaining palette lines untouched.
    palette_buffer[1..=15].copy_from_slice(foreground_palette);
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum BackgroundPaletteOverrideLoc11DF4 {
    None,
    AlternativeCave,
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn classify_background_palette_override_loc_11DF4(
    map_id: u8,
) -> BackgroundPaletteOverrideLoc11DF4 {
    // ASM loc_11DF4 (kid.asm:24002): Stairway to Oblivion (`#3`) and
    // Elsewhere 23 (`#23`) replace `a1` with `Pal_12CF8(pc)` before the shared
    // continuation at `loc_11DF8`.
    if matches!(map_id, 0x03 | 23) {
        return BackgroundPaletteOverrideLoc11DF4::AlternativeCave;
    }
    BackgroundPaletteOverrideLoc11DF4::None
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum BackgroundPaletteFollowupLoc11DF8 {
    None,
    RemovedSpecialPaletteNoop,
    LateAlternativePalette,
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn classify_background_palette_followup_loc_11DF8(
    map_id: u8,
) -> BackgroundPaletteFollowupLoc11DF8 {
    // ASM loc_11DF8 (kid.asm:24005): whether the earlier `loc_11DF4` cave
    // branch fired or not, the shared continuation then checks The Deadly
    // Skyscrapers (`#$1C`) / Elsewhere 30 (`#30`) for the removed special
    // palette path before the later `$1A/$1B` alternative-palette override.
    if matches!(map_id, 0x1C | 30) {
        return BackgroundPaletteFollowupLoc11DF8::RemovedSpecialPaletteNoop;
    }
    if matches!(map_id, 0x1A | 0x1B) {
        return BackgroundPaletteFollowupLoc11DF8::LateAlternativePalette;
    }
    BackgroundPaletteFollowupLoc11DF8::None
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum LateBackgroundPaletteOverrideLoc11E06 {
    None,
    AlternativeAlienPalette,
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn classify_late_background_palette_override_loc_11E06(
    map_id: u8,
) -> LateBackgroundPaletteOverrideLoc11E06 {
    // ASM loc_11E06 (kid.asm:24020): after the earlier `loc_11DF8` / `loc_11E04`
    // checks, only map IDs `$1A` and `$1B` branch to `loc_11E12`; every other
    // map falls straight through to `loc_11E16`.
    if matches!(map_id, 0x1A | 0x1B) {
        return LateBackgroundPaletteOverrideLoc11E06::AlternativeAlienPalette;
    }
    LateBackgroundPaletteOverrideLoc11E06::None
}

#[allow(non_snake_case)]
#[allow(dead_code)]
pub(crate) fn apply_background_palette_override_loc_11E04(
    selected_theme_palette: u16,
    override_kind: BackgroundPaletteFollowupLoc11DF8,
) -> u16 {
    // ASM loc_11E04 (kid.asm:24011): the taken `$1C/$30` branch reaches a
    // removed `lea Pal_12D44(pc),a1` command that the ROM replaced with `nop`,
    // so both the explicit no-op path and the ordinary fallthrough preserve
    // the already selected background palette.
    match override_kind {
        BackgroundPaletteFollowupLoc11DF8::None
        | BackgroundPaletteFollowupLoc11DF8::RemovedSpecialPaletteNoop
        | BackgroundPaletteFollowupLoc11DF8::LateAlternativePalette => selected_theme_palette,
    }
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn apply_background_palette_override_loc_11E12(
    selected_theme_palette: u16,
    alternative_alien_palette: u16,
    override_kind: LateBackgroundPaletteOverrideLoc11E06,
) -> u16 {
    // ASM loc_11E12 (kid.asm:24026): the taken `$1A/$1B` branch replaces `a1`
    // with `Pal_12D54(pc)` immediately before the shared copy at `loc_11E16`.
    match override_kind {
        LateBackgroundPaletteOverrideLoc11E06::AlternativeAlienPalette => alternative_alien_palette,
        LateBackgroundPaletteOverrideLoc11E06::None => selected_theme_palette,
    }
}

#[allow(non_snake_case)]
#[allow(dead_code)]
pub(crate) fn copy_background_palette_loc_11E16(
    background_palette: &[u16; 8],
    palette_buffer: &mut [u16; 0x40],
) {
    // ASM loc_11E16 (kid.asm:24029): copy the first word into
    // `Palette_Buffer`, then seed `a2` with `Palette_Buffer+$32` and run a
    // `moveq #6,d1` / `dbf` loop that copies the remaining 7 words into
    // palette slots `$19..$1F`.
    palette_buffer[0] = background_palette[0];
    palette_buffer[0x19..=0x1F].copy_from_slice(&background_palette[1..]);
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ForegroundArtPermutationLoc11E48 {
    Identity,
    City,
    Forest,
    Mountain,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum BackgroundArtPermutationLoc11E8A {
    Identity,
    Forest,
    Mountain,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct BackgroundArtDecompressLoc11EA2 {
    pub(crate) tile_base: u16,
    pub(crate) permutation: BackgroundArtPermutationLoc11E8A,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum BackgroundFollowupArtSourceLoc11EC4 {
    CaveAlt,
    MountainLightning,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum BackgroundFollowupArtPermutationLoc11EC4 {
    Identity,
    BgMountain,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct BackgroundFollowupArtDecompressLoc11EC4 {
    pub(crate) tile_base: u16,
    pub(crate) permutation: BackgroundFollowupArtPermutationLoc11EC4,
    pub(crate) art_source: BackgroundFollowupArtSourceLoc11EC4,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct BackgroundFollowupArtDecompressLoc11EF2 {
    pub(crate) tile_base: u16,
    pub(crate) permutation: BackgroundFollowupArtPermutationLoc11EC4,
    pub(crate) art_source: BackgroundFollowupArtSourceLoc11EC4,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum SharedVramArtSourceLoc11EFA {
    Blocks,
    IngameNumbers,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct SharedVramArtDecompressLoc11EFA {
    pub(crate) tile_base: u16,
    pub(crate) art_source: SharedVramArtSourceLoc11EFA,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct LevelCameraBootstrapLoc11F48 {
    pub(crate) vram_fill_word: u16,
    pub(crate) vram_fill_count: u16,
    pub(crate) camera_x_pos: u32,
    pub(crate) camera_y_pos: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum PostLoadObjectRoutineLoc11F96 {
    RotatePaletteLavaWater {
        object_slot_seed: u16,
    },
    IslandPaletteCycle {
        object_slot_seed: u16,
        object_routine_addr: u32,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct LevelStartMusicCommandLoc11FD2 {
    pub(crate) sound_id: u16,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct ForegroundThemeMusicLoc12004 {
    pub(crate) sound_id: u16,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct IslandPaletteCyclePrimaryBandLoc12098 {
    pub(crate) next_countdown: u16,
    pub(crate) next_phase: u16,
    pub(crate) uploaded_frame_index: Option<u16>,
    pub(crate) dma_source_offset: Option<u32>,
    pub(crate) vram_word_address: Option<u16>,
    pub(crate) dma_length_bytes: Option<u16>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct IslandPaletteCycleMidBandLoc12110 {
    pub(crate) next_countdown: u16,
    pub(crate) next_phase: u16,
    pub(crate) uploaded_frame_index: Option<u16>,
    pub(crate) dma_source_offset: Option<u32>,
    pub(crate) vram_word_address: Option<u16>,
    pub(crate) dma_length_bytes: Option<u16>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct IslandPaletteCycleLowerBandLoc12188 {
    pub(crate) next_countdown: u16,
    pub(crate) next_phase: u16,
    pub(crate) uploaded_frame_index: Option<u16>,
    pub(crate) dma_source_offset: Option<u32>,
    pub(crate) vram_word_address: Option<u16>,
    pub(crate) dma_length_bytes: Option<u16>,
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn select_foreground_art_permutation_loc_11E48(
    foreground_theme: u16,
    selected_permutation: ForegroundArtPermutationLoc11E48,
) -> ForegroundArtPermutationLoc11E48 {
    const FOREGROUND_THEME_FOREST_LOC_11E48: u16 = 0x0009;

    // ASM loc_11E48 (kid.asm:24046): after the earlier City compare may have
    // already swapped `a3` to `Palette_Permutation_FGCity`, this branch only
    // overrides that selection when `Foreground_theme == Forest`.
    if foreground_theme == FOREGROUND_THEME_FOREST_LOC_11E48 {
        ForegroundArtPermutationLoc11E48::Forest
    } else {
        selected_permutation
    }
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn select_foreground_art_permutation_loc_11E54(
    foreground_theme: u16,
    selected_permutation: ForegroundArtPermutationLoc11E48,
) -> ForegroundArtPermutationLoc11E48 {
    const FOREGROUND_THEME_MOUNTAIN_LOC_11E54: u16 = 0x0007;

    // ASM loc_11E54 (kid.asm:24051): after the earlier City/Forest checks have
    // already selected `a3`, this compare only replaces that permutation with
    // `Palette_Permutation_FGMountain` when `Foreground_theme == Mountain`.
    if foreground_theme == FOREGROUND_THEME_MOUNTAIN_LOC_11E54 {
        ForegroundArtPermutationLoc11E48::Mountain
    } else {
        selected_permutation
    }
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn select_background_art_permutation_loc_11E8A(
    background_theme: u16,
    selected_permutation: BackgroundArtPermutationLoc11E8A,
) -> BackgroundArtPermutationLoc11E8A {
    const BACKGROUND_THEME_FOREST_LOC_11E8A: u16 = 0x0009;

    // ASM loc_11E8A (kid.asm:24069): after the shared setup seeds `a3` with
    // `Palette_Permutation_Identity`, only `Background_theme == Forest`
    // overrides that selection with `Palette_Permutation_BGForest`.
    if background_theme == BACKGROUND_THEME_FOREST_LOC_11E8A {
        BackgroundArtPermutationLoc11E8A::Forest
    } else {
        selected_permutation
    }
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn select_background_art_permutation_loc_11E96(
    background_theme: u16,
    selected_permutation: BackgroundArtPermutationLoc11E8A,
) -> BackgroundArtPermutationLoc11E8A {
    const BACKGROUND_THEME_MOUNTAIN_LOC_11E96: u16 = 0x0007;

    // ASM loc_11E96 (kid.asm:24074): after the earlier Forest compare may
    // already have swapped `a3` to `Palette_Permutation_BGForest`, only
    // `Background_theme == Mountain` replaces the current selection with
    // `Palette_Permutation_BGMountain`.
    if background_theme == BACKGROUND_THEME_MOUNTAIN_LOC_11E96 {
        BackgroundArtPermutationLoc11E8A::Mountain
    } else {
        selected_permutation
    }
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn prepare_background_art_decompress_loc_11EA2(
    selected_permutation: BackgroundArtPermutationLoc11E8A,
) -> BackgroundArtDecompressLoc11EA2 {
    // ASM loc_11EA2 (kid.asm:24079-24080): after the Forest/Mountain checks
    // have finished mutating `a3`, the shared continuation calls
    // `DecompressToRAM` with that same permutation and RAM base `$F000`.
    BackgroundArtDecompressLoc11EA2 {
        tile_base: 0xF000,
        permutation: selected_permutation,
    }
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn prepare_mountain_background_art_loc_11EC4(
    background_theme: u16,
) -> Option<BackgroundFollowupArtDecompressLoc11EC4> {
    const BACKGROUND_THEME_MOUNTAIN_LOC_11EC4: u16 = 0x0007;

    // ASM loc_11EC4 (kid.asm:24089): only the Mountain background runs this
    // second RAM decompress, seeding `a3` with the identity permutation,
    // selecting `ThemeArtBack_CaveAlt`, and using VRAM base `$FCA0`.
    (background_theme == BACKGROUND_THEME_MOUNTAIN_LOC_11EC4).then_some(
        BackgroundFollowupArtDecompressLoc11EC4 {
            tile_base: 0xFCA0,
            permutation: BackgroundFollowupArtPermutationLoc11EC4::Identity,
            art_source: BackgroundFollowupArtSourceLoc11EC4::CaveAlt,
        },
    )
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn apply_mountain_weather_override_loc_11EEA(
    level_special_effects: u16,
    selected_decompress: BackgroundFollowupArtDecompressLoc11EC4,
) -> BackgroundFollowupArtDecompressLoc11EC4 {
    const WEATHER_ID_STORM_LOC_11EEA: u16 = 0x0002;
    const WEATHER_ID_STORM_AND_HAIL_LOC_11EEA: u16 = 0x0003;

    // ASM loc_11EEA (kid.asm:24100): after the Mountain-only setup at
    // `loc_11EC4`, storm and storm+hail replace `ThemeArtBack_CaveAlt` with
    // `ThemeArtBack_MountainLightning` and swap the permutation to
    // `Palette_Permutation_BGMountain`; all other weather falls through.
    if matches!(
        level_special_effects,
        WEATHER_ID_STORM_LOC_11EEA | WEATHER_ID_STORM_AND_HAIL_LOC_11EEA
    ) {
        BackgroundFollowupArtDecompressLoc11EC4 {
            tile_base: selected_decompress.tile_base,
            permutation: BackgroundFollowupArtPermutationLoc11EC4::BgMountain,
            art_source: BackgroundFollowupArtSourceLoc11EC4::MountainLightning,
        }
    } else {
        selected_decompress
    }
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn prepare_followup_background_art_decompress_loc_11EF2(
    selected_decompress: BackgroundFollowupArtDecompressLoc11EC4,
) -> BackgroundFollowupArtDecompressLoc11EF2 {
    // ASM loc_11EF2 (kid.asm:24104): the shared Mountain follow-up tail
    // reloads `d0` with `$FCA0` and immediately calls `DecompressToRAM`,
    // preserving whichever art source and permutation `loc_11EC4/loc_11EEA`
    // left in `a0/a3`.
    BackgroundFollowupArtDecompressLoc11EF2 {
        tile_base: 0xFCA0,
        permutation: selected_decompress.permutation,
        art_source: selected_decompress.art_source,
    }
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn shared_vram_art_decompresses_loc_11EFA() -> [SharedVramArtDecompressLoc11EFA; 2] {
    // ASM loc_11EFA (kid.asm:24108-24115): after the background setup, the
    // shared tail always decompresses the block art to VRAM `$4400` and the
    // in-game number glyphs to VRAM `$D740` before continuing into the HUD
    // and helper-art loads.
    [
        SharedVramArtDecompressLoc11EFA {
            tile_base: 0x4400,
            art_source: SharedVramArtSourceLoc11EFA::Blocks,
        },
        SharedVramArtDecompressLoc11EFA {
            tile_base: 0xD740,
            art_source: SharedVramArtSourceLoc11EFA::IngameNumbers,
        },
    ]
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct ForegroundArtDecompressLoc11E60 {
    pub(crate) tile_base: u16,
    pub(crate) permutation: ForegroundArtPermutationLoc11E48,
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn prepare_foreground_art_decompress_loc_11E60(
    selected_permutation: ForegroundArtPermutationLoc11E48,
) -> ForegroundArtDecompressLoc11E60 {
    // ASM loc_11E60 (kid.asm:24056): once the earlier City/Forest/Mountain
    // compares have finished mutating `a3`, the shared continuation calls
    // `DecompressToRAM` with that same permutation and VRAM base `$1780`.
    ForegroundArtDecompressLoc11E60 {
        tile_base: 0x1780,
        permutation: selected_permutation,
    }
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn initialise_camera_bootstrap_loc_11F48(
    player_start_x_pos: u16,
    player_start_y_pos: u16,
) -> LevelCameraBootstrapLoc11F48 {
    const CAMERA_X_SCREEN_WIDTH_LOC_11F48: u16 = 0x0140;
    const CAMERA_Y_SCREEN_HEIGHT_LOC_11F48: u16 = 0x00E0;

    // ASM loc_11F48 (kid.asm:24129): after clearing `$E000..$EFFF` in VRAM by
    // writing `$0780` exactly `$800` times, the bootstrap floors each player
    // start coordinate to a full screen and stores it as 16.16 camera space by
    // moving the rounded word into the upper half of the longword.
    let camera_x_px =
        (player_start_x_pos / CAMERA_X_SCREEN_WIDTH_LOC_11F48) * CAMERA_X_SCREEN_WIDTH_LOC_11F48;
    let camera_y_px =
        (player_start_y_pos / CAMERA_Y_SCREEN_HEIGHT_LOC_11F48) * CAMERA_Y_SCREEN_HEIGHT_LOC_11F48;

    LevelCameraBootstrapLoc11F48 {
        vram_fill_word: 0x0780,
        vram_fill_count: 0x0800,
        camera_x_pos: u32::from(camera_x_px) << 16,
        camera_y_pos: u32::from(camera_y_px) << 16,
    }
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn classify_post_load_object_routine_loc_11F96(
    foreground_theme: u16,
    background_theme: u16,
) -> Option<PostLoadObjectRoutineLoc11F96> {
    const FOREGROUND_THEME_CAVE_LOC_11F96: u16 = 0x0008;
    const ROTATE_PALETTE_OBJECT_SLOT_SEED_LOC_11F96: u16 = 0xA000;
    const ISLAND_OBJECT_SLOT_SEED_LOC_11FB0: u16 = 0x0000;
    const ISLAND_OBJECT_ROUTINE_ADDR_LOC_11FB0: u32 = 0x0001_202A;

    // ASM loc_11F96 (kid.asm:24154): when the earlier Mountain compare at
    // `loc_11F86` falls through, Cave foregrounds allocate the same `$A000`
    // palette-rotation object. The shared tail then checks `Background_theme`
    // for Island and, if matched, allocates slot `0` and seeds `4(a0)` with
    // `loc_1202A`.
    if foreground_theme == FOREGROUND_THEME_CAVE_LOC_11F96 {
        Some(PostLoadObjectRoutineLoc11F96::RotatePaletteLavaWater {
            object_slot_seed: ROTATE_PALETTE_OBJECT_SLOT_SEED_LOC_11F96,
        })
    } else if classify_island_background_object_loc_11FB0(background_theme) {
        Some(PostLoadObjectRoutineLoc11F96::IslandPaletteCycle {
            object_slot_seed: ISLAND_OBJECT_SLOT_SEED_LOC_11FB0,
            object_routine_addr: ISLAND_OBJECT_ROUTINE_ADDR_LOC_11FB0,
        })
    } else {
        None
    }
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn classify_island_background_object_loc_11FB0(background_theme: u16) -> bool {
    const BACKGROUND_THEME_ISLAND_LOC_11FB0: u16 = 0x0004;

    // ASM loc_11FB0 (kid.asm:24163): once the foreground Cave branch at
    // `loc_11F96` falls through, only `Background_theme == Island` allocates
    // slot `0` and seeds `4(a0)` with `loc_1202A`; all other themes fall
    // through untouched into `loc_11FC8`.
    background_theme == BACKGROUND_THEME_ISLAND_LOC_11FB0
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn select_level_start_music_loc_11FD2(
    murder_wall_flag: bool,
    foreground_theme: u16,
) -> LevelStartMusicCommandLoc11FD2 {
    const BGM_MURDERWALL_LOC_11FD2: u16 = 0x0084;

    // ASM loc_11FD2 (kid.asm:24175): after the shared post-load init calls, a
    // non-zero `MurderWall_flag` forces `d0 = bgm_Murderwall` and returns
    // immediately; otherwise execution falls through to `loc_12004`.
    if murder_wall_flag {
        LevelStartMusicCommandLoc11FD2 {
            sound_id: BGM_MURDERWALL_LOC_11FD2,
        }
    } else {
        LevelStartMusicCommandLoc11FD2 {
            sound_id: select_foreground_theme_music_loc_12004(foreground_theme).sound_id,
        }
    }
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn select_foreground_theme_music_loc_12004(
    foreground_theme: u16,
) -> ForegroundThemeMusicLoc12004 {
    const FOREGROUND_THEME_BGM_TABLE_LOC_12004: [u16; 11] = [
        0x0080, 0x0081, 0x0082, 0x0083, 0x0085, 0x0086, 0x0087, 0x0080, 0x0088, 0x0089, 0x008A,
    ];

    // ASM loc_12004 (kid.asm:24190): load `Foreground_theme` into `d1`, index
    // `unk_1201E(pc)` bytewise, mirror that byte into `d0`, store it in
    // `($FFFFFC32).w`, then call `j_PlaySound`.
    ForegroundThemeMusicLoc12004 {
        sound_id: FOREGROUND_THEME_BGM_TABLE_LOC_12004[usize::from(foreground_theme)],
    }
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn step_island_palette_cycle_primary_band_loc_12098(
    countdown_44: u16,
    phase_d3: u16,
) -> IslandPaletteCyclePrimaryBandLoc12098 {
    // ASM loc_12098 (kid.asm:24246): after `Hibernate_Object_1Frame`, decrement
    // `$44(a5)`. A non-zero result falls straight through to `loc_12110`; zero
    // uses the current `d3` as the frame-table index, advances `d3` with
    // `addq.w #1` + `andi.w #3`, uploads `off_1220E[d4] + 2` to VRAM `$F600`,
    // and reloads `$44(a5)` with `#9`.
    let decremented = countdown_44.wrapping_sub(1);
    if decremented != 0 {
        return IslandPaletteCyclePrimaryBandLoc12098 {
            next_countdown: decremented,
            next_phase: phase_d3 & 3,
            uploaded_frame_index: None,
            dma_source_offset: None,
            vram_word_address: None,
            dma_length_bytes: None,
        };
    }

    IslandPaletteCyclePrimaryBandLoc12098 {
        next_countdown: 9,
        next_phase: phase_d3.wrapping_add(1) & 3,
        uploaded_frame_index: Some(phase_d3 & 3),
        dma_source_offset: Some(0x0002),
        vram_word_address: Some(0xF600),
        dma_length_bytes: Some(0x00C0),
    }
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn step_island_palette_cycle_mid_band_loc_12110(
    countdown_46: u16,
    phase_d2: u16,
) -> IslandPaletteCycleMidBandLoc12110 {
    // ASM loc_12110 (kid.asm:24279): decrement `$46(a5)` after the shared
    // hibernate/`loc_12098` path. A non-zero result falls through to
    // `loc_12188`; zero uses the current `d2` as the frame-table index,
    // advances `d2` with `addq.w #1` + `andi.w #3`, uploads `off_1220E[d4] +
    // $C2` to VRAM `$F6C0`, and reloads `$46(a5)` with `#9`.
    let decremented = countdown_46.wrapping_sub(1);
    if decremented != 0 {
        return IslandPaletteCycleMidBandLoc12110 {
            next_countdown: decremented,
            next_phase: phase_d2 & 3,
            uploaded_frame_index: None,
            dma_source_offset: None,
            vram_word_address: None,
            dma_length_bytes: None,
        };
    }

    IslandPaletteCycleMidBandLoc12110 {
        next_countdown: 9,
        next_phase: phase_d2.wrapping_add(1) & 3,
        uploaded_frame_index: Some(phase_d2 & 3),
        dma_source_offset: Some(0x00C2),
        vram_word_address: Some(0xF6C0),
        dma_length_bytes: Some(0x00A0),
    }
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn step_island_palette_cycle_lower_band_loc_12188(
    countdown_48: u16,
    phase_d1: u16,
) -> IslandPaletteCycleLowerBandLoc12188 {
    const COUNTDOWN_TABLE_LOC_121A6: [u16; 6] = [0x0019, 0x001E, 0x001E, 0x0014, 0x000F, 0x000A];

    // ASM loc_12188 (kid.asm:24311): decrement `$48(a5)` and branch back to
    // `loc_12098` while the result stays non-zero. Once it hits zero, copy the
    // current `d1` into `d4`, advance `d1` with `addq.w #1` and the
    // `cmpi.w #5`/`moveq #0` wrap, reload `$48(a5)` from `unk_12226[d4]`, then
    // DMA `off_1220E[d4] + $162` to VRAM `$F760`.
    let decremented = countdown_48.wrapping_sub(1);
    if decremented != 0 {
        return IslandPaletteCycleLowerBandLoc12188 {
            next_countdown: decremented,
            next_phase: phase_d1,
            uploaded_frame_index: None,
            dma_source_offset: None,
            vram_word_address: None,
            dma_length_bytes: None,
        };
    }

    // ASM loc_121A6 (kid.asm:24322): merge point after `cmpi.w #5,d1`; it
    // reloads `$48(a5)` from `unk_12226[d4]` using the pre-increment phase in
    // `d4`, while the already-incremented `d1` either advances or wraps to 0.
    let frame_index = usize::from(phase_d1);
    let next_phase = if phase_d1 >= 5 { 0 } else { phase_d1 + 1 };
    IslandPaletteCycleLowerBandLoc12188 {
        next_countdown: COUNTDOWN_TABLE_LOC_121A6[frame_index],
        next_phase,
        uploaded_frame_index: Some(phase_d1),
        dma_source_offset: Some(0x0162),
        vram_word_address: Some(0xF760),
        dma_length_bytes: Some(0x0180),
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct RotatePaletteLavaWaterDelayLoc1223A {
    pub(crate) delay_frames: u16,
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn compute_rotate_palette_delay_loc_1223A(
    foreground_theme: u16,
) -> RotatePaletteLavaWaterDelayLoc1223A {
    const FOREGROUND_THEME_MOUNTAIN_LOC_1223A: u16 = 0x0007;

    // ASM loc_1223A (kid.asm:24370): after `RotatePalette_Lava_Water`
    // initialises `d0 = #5`, the Mountain compare branches here on match;
    // otherwise `addq.w #3,d0` makes it `#8`. The result is saved into `d2`
    // before the hibernate loop at `loc_1223C`.
    let delay_frames = if foreground_theme == FOREGROUND_THEME_MOUNTAIN_LOC_1223A {
        5
    } else {
        8
    };
    RotatePaletteLavaWaterDelayLoc1223A { delay_frames }
}

pub(crate) const PALETTE_BUFFER_ROTATE_START_LOC_1223C: usize = 0x0E;

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn rotate_palette_lava_water_loc_1223C(palette_buffer: &mut [u16; 0x40]) {
    // ASM loc_1223C (kid.asm:24373): after the hibernate loop drains `d0`,
    // the routine rotates four palette words downward:
    // - `move.w (Palette_Buffer+$14).l,d1` saves word at $14
    // - `move.w (Palette_Buffer+$12).l,(Palette_Buffer+$14).l` copies $12→$14
    // - `move.w (Palette_Buffer+$10).l,(Palette_Buffer+$12).l` copies $10→$12
    // - `move.w (Palette_Buffer+$E).l,(Palette_Buffer+$10).l`  copies $E→$10
    // - `move.w d1,(Palette_Buffer+$E).l` restores saved $14→$E
    // - `bra.s loc_1223C` returns to hibernate for another cycle.
    let saved = palette_buffer[PALETTE_BUFFER_ROTATE_START_LOC_1223C + 3];
    palette_buffer[PALETTE_BUFFER_ROTATE_START_LOC_1223C + 3] =
        palette_buffer[PALETTE_BUFFER_ROTATE_START_LOC_1223C + 2];
    palette_buffer[PALETTE_BUFFER_ROTATE_START_LOC_1223C + 2] =
        palette_buffer[PALETTE_BUFFER_ROTATE_START_LOC_1223C + 1];
    palette_buffer[PALETTE_BUFFER_ROTATE_START_LOC_1223C + 1] =
        palette_buffer[PALETTE_BUFFER_ROTATE_START_LOC_1223C];
    palette_buffer[PALETTE_BUFFER_ROTATE_START_LOC_1223C] = saved;
}

#[allow(non_snake_case)]
#[allow(dead_code)]
pub(crate) fn initialise_sprite_frame_state_sub_B52() -> SpriteFrameStateSubB52 {
    // ASM sub_B52 (kid.asm:1319): clear `Number_Sprites`, point
    // `Addr_NextSpriteSlot` at `Sprite_Table`, and reset the dynamic-art
    // staging cursor at `$FFFF0500`.
    SpriteFrameStateSubB52 {
        number_sprites: 0,
        next_sprite_slot_addr: SPRITE_TABLE_ADDR_SUB_B52,
        dynamic_sprite_art_addr: SPRITE_DYNAMIC_ART_ADDR_SUB_B52,
    }
}

#[allow(non_snake_case)]
#[allow(dead_code)]
pub(crate) fn initialise_scratch_queue_sub_28FC() -> ScratchQueueSub28FC {
    // ASM sub_28FC (kid.asm:4854): clear the `$FFFFF01E` work area as
    // 6 adjacent `$52`-byte slots, rewind the cursor to slot 0, and reload
    // the slot counter to `#6`.
    ScratchQueueSub28FC {
        slots: [[0; SUB_28FC_SLOT_STRIDE_BYTES]; SUB_28FC_SLOT_COUNT],
        next_slot: 0,
        slots_remaining: SUB_28FC_SLOT_COUNT as u16,
    }
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn initialise_evanesc_object_slots_sub_EABC() -> EvanescShooterObjectSlotsSubEABC {
    // ASM sub_EABC (kid.asm:18834): seed the evanescent-object free list at
    // `$FFFFEBDA` as 40 contiguous `$0C`-byte slots whose `$A(a0)` word points
    // to the next slot, then clear `Addr_FirstEvanescObjectSlot`.
    let mut evanescent_slots = Vec::with_capacity(EVANESCENT_OBJECT_SLOT_COUNT_SUB_EABC);
    let mut slot_addr = EVANESCENT_OBJECT_RAM_ADDR_SUB_EABC;
    for slot_index in 0..EVANESCENT_OBJECT_SLOT_COUNT_SUB_EABC {
        let is_last = slot_index + 1 == EVANESCENT_OBJECT_SLOT_COUNT_SUB_EABC;
        let next_slot_addr = if is_last {
            0
        } else {
            slot_addr + u32::from(EVANESCENT_OBJECT_SLOT_STRIDE_SUB_EABC)
        };
        evanescent_slots.push(LinkedObjectSlotInit {
            slot_addr,
            next_slot_addr,
        });
        slot_addr += u32::from(EVANESCENT_OBJECT_SLOT_STRIDE_SUB_EABC);
    }

    EvanescShooterObjectSlotsSubEABC {
        next_free_evanescent_object_slot_addr: EVANESCENT_OBJECT_RAM_ADDR_SUB_EABC,
        first_evanescent_object_slot_addr: 0,
        evanescent_slots,
    }
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn initialise_shooter_object_slots_sub_F06A() -> ShooterObjectSlotsSubF06A {
    // ASM sub_F06A (kid.asm:19595): seed the shooter-object free list at
    // `$FFFFE7F2` as 20 contiguous `$0E`-byte slots whose `$C(a0)` word points
    // to the next slot, clear `Addr_FirstShooterObjectSlot`, then reset the
    // last-triggered shooter cell coordinates at `$FFFFF8BE/$FFFFF8C0` to -1.
    let mut shooter_slots = Vec::with_capacity(SHOOTER_OBJECT_SLOT_COUNT_SUB_F06A);
    let mut slot_addr = SHOOTER_OBJECT_RAM_ADDR_SUB_F06A;
    for slot_index in 0..SHOOTER_OBJECT_SLOT_COUNT_SUB_F06A {
        let is_last = slot_index + 1 == SHOOTER_OBJECT_SLOT_COUNT_SUB_F06A;
        let next_slot_addr = if is_last {
            0
        } else {
            slot_addr + u32::from(SHOOTER_OBJECT_SLOT_STRIDE_SUB_F06A)
        };
        shooter_slots.push(LinkedObjectSlotInit {
            slot_addr,
            next_slot_addr,
        });
        slot_addr += u32::from(SHOOTER_OBJECT_SLOT_STRIDE_SUB_F06A);
    }

    ShooterObjectSlotsSubF06A {
        next_free_shooter_object_slot_addr: SHOOTER_OBJECT_RAM_ADDR_SUB_F06A,
        first_shooter_object_slot_addr: 0,
        shooter_slots,
        last_shooter_block_col: -1,
        last_shooter_block_row: -1,
    }
}

#[allow(non_snake_case)]
#[allow(dead_code)]
pub(crate) fn spawn_block_debris_sub_292E(
    queue: &mut ScratchQueueSub28FC,
    x_pos_px: u16,
    y_pos_px: u16,
    direction: u16,
) {
    // ASM sub_292E (kid.asm:4881): mark the current `$52`-byte scratch slot
    // active, clear byte 1, then seed five 16-byte debris records with fixed
    // x/y offsets and direction-selected signed integer velocities. After the
    // write, decrement `$FFFFFAA0`; if it hit zero, rewind to slot 0 and
    // reload the six-slot counter, otherwise advance `$FFFFFA9C` by `$52`.
    let slot_index = queue.next_slot;
    let slot = &mut queue.slots[slot_index];
    slot.fill(0);
    slot[SUB_292E_SLOT_ACTIVE_OFFSET] = 0xFF;
    slot[SUB_292E_SLOT_FLAG_OFFSET] = 0x00;

    let direction_index = usize::from(direction.min(3));
    for piece_index in 0..SUB_292E_PIECE_COUNT {
        let base = SUB_292E_SLOT_PIECES_OFFSET + piece_index * SUB_292E_PIECE_STRIDE_BYTES;
        let y_velocity_bytes = SUB_292E_Y_VELOCITIES[direction_index][piece_index].to_be_bytes();
        let x_velocity_bytes = SUB_292E_X_VELOCITIES[direction_index][piece_index].to_be_bytes();
        let y_pos_bytes = y_pos_px
            .wrapping_add(SUB_292E_Y_OFFSETS[piece_index])
            .to_be_bytes();
        let x_pos_bytes = x_pos_px
            .wrapping_add(SUB_292E_X_OFFSETS[piece_index])
            .to_be_bytes();

        slot[base..base + 2].copy_from_slice(&y_velocity_bytes);
        slot[base + 4..base + 6].copy_from_slice(&y_pos_bytes);
        slot[base + 8..base + 10].copy_from_slice(&x_velocity_bytes);
        slot[base + 12..base + 14].copy_from_slice(&x_pos_bytes);
    }

    if queue.slots_remaining <= 1 {
        queue.slots_remaining = SUB_28FC_SLOT_COUNT as u16;
        queue.next_slot = 0;
    } else {
        queue.slots_remaining -= 1;
        queue.next_slot = (slot_index + 1) % SUB_28FC_SLOT_COUNT;
    }
}

#[allow(non_snake_case)]
#[allow(dead_code)]
pub(crate) fn scratch_debris_pieces_sub_292E(
    queue: &ScratchQueueSub28FC,
    slot_index: usize,
) -> [ScratchDebrisPieceSub292E; SUB_292E_PIECE_COUNT] {
    let slot = &queue.slots[slot_index];
    core::array::from_fn(|piece_index| {
        let base = SUB_292E_SLOT_PIECES_OFFSET + piece_index * SUB_292E_PIECE_STRIDE_BYTES;
        ScratchDebrisPieceSub292E {
            y_velocity_px: i16::from_be_bytes([slot[base], slot[base + 1]]),
            y_pos_px: u16::from_be_bytes([slot[base + 4], slot[base + 5]]),
            x_velocity_px: i16::from_be_bytes([slot[base + 8], slot[base + 9]]),
            x_pos_px: u16::from_be_bytes([slot[base + 12], slot[base + 13]]),
        }
    })
}

#[allow(non_snake_case)]
fn scratch_debris_slot_piece_long_sub_2A4C(
    slot: &[u8; SUB_28FC_SLOT_STRIDE_BYTES],
    piece_index: usize,
    field_offset: usize,
) -> i32 {
    let base =
        SUB_292E_SLOT_PIECES_OFFSET + piece_index * SUB_292E_PIECE_STRIDE_BYTES + field_offset;
    i32::from_be_bytes([slot[base], slot[base + 1], slot[base + 2], slot[base + 3]])
}

#[allow(non_snake_case)]
fn set_scratch_debris_slot_piece_long_sub_2A4C(
    slot: &mut [u8; SUB_28FC_SLOT_STRIDE_BYTES],
    piece_index: usize,
    field_offset: usize,
    value: i32,
) {
    let base =
        SUB_292E_SLOT_PIECES_OFFSET + piece_index * SUB_292E_PIECE_STRIDE_BYTES + field_offset;
    slot[base..base + 4].copy_from_slice(&value.to_be_bytes());
}

#[allow(non_snake_case)]
#[allow(dead_code)]
pub(crate) fn compile_block_debris_sprites_sub_2A4C(
    queue: &mut ScratchQueueSub28FC,
    number_sprites: u8,
    camera_x_pos: i16,
    camera_y_pos: i16,
) -> ScratchDebrisRenderSub2A4C {
    // ASM sub_2A4C (kid.asm:5053): iterate all six `$52`-byte scratch slots,
    // skip inactive ones, then advance each of the five debris records as
    // 16.16 fixed-point `(velocity += $36B0, position += velocity)` before
    // emitting sprite entries with tile `$8228`. The lead fragment kills the
    // slot if it leaves `x:[0,$13F]` or `y:[$FF9C,$DF]`; the tail fragment
    // also kills the slot once its x position reaches `$1EF`.
    let mut sprites = Vec::new();
    let mut link = u16::from(number_sprites);

    for slot in &mut queue.slots {
        if slot[SUB_292E_SLOT_ACTIVE_OFFSET] == 0 {
            continue;
        }

        let mut slot_active = true;
        for piece_index in 0..SUB_292E_PIECE_COUNT {
            let mut y_velocity_fp = scratch_debris_slot_piece_long_sub_2A4C(slot, piece_index, 0);
            let mut y_pos_fp = scratch_debris_slot_piece_long_sub_2A4C(slot, piece_index, 4);
            let x_velocity_fp = scratch_debris_slot_piece_long_sub_2A4C(slot, piece_index, 8);
            let mut x_pos_fp = scratch_debris_slot_piece_long_sub_2A4C(slot, piece_index, 12);

            y_pos_fp = y_pos_fp.wrapping_add(y_velocity_fp);
            x_pos_fp = x_pos_fp.wrapping_add(x_velocity_fp);
            y_velocity_fp = y_velocity_fp.wrapping_add(SUB_2A4C_GRAVITY_STEP_FP);

            set_scratch_debris_slot_piece_long_sub_2A4C(slot, piece_index, 0, y_velocity_fp);
            set_scratch_debris_slot_piece_long_sub_2A4C(slot, piece_index, 4, y_pos_fp);
            set_scratch_debris_slot_piece_long_sub_2A4C(slot, piece_index, 12, x_pos_fp);

            let world_y = (y_pos_fp >> 16) as i16;
            let world_x = (x_pos_fp >> 16) as i16;
            let relative_y = world_y.wrapping_sub(camera_y_pos);
            let relative_x = world_x.wrapping_sub(camera_x_pos);

            if piece_index == 0
                && (!(-100..0x00E0).contains(&relative_y) || !(0..0x0140).contains(&relative_x))
            {
                slot_active = false;
                break;
            }

            if piece_index == SUB_292E_PIECE_COUNT - 1 && relative_x >= 0x01EF {
                slot_active = false;
                break;
            }

            link = link.wrapping_add(1);
            sprites.push(ScratchDebrisSpriteSub2A4C {
                y: relative_y.wrapping_add(SUB_2A4C_SPRITE_COORD_BIAS) as u16,
                link,
                tile_word: SUB_2A4C_SPRITE_TILE_WORD,
                x: relative_x.wrapping_add(SUB_2A4C_SPRITE_COORD_BIAS) as u16,
            });
        }

        if !slot_active {
            slot[SUB_292E_SLOT_ACTIVE_OFFSET] = 0x00;
        }
    }

    ScratchDebrisRenderSub2A4C {
        sprites,
        number_sprites: link as u8,
    }
}

#[allow(non_snake_case)]
#[allow(dead_code)]
pub(crate) fn swap_forest_background_palette_entries_sub_129AE(
    header: Option<&kid_data::LevelHeader>,
    palette_buffer: &mut [u16],
) {
    // ASM sub_129AE (kid.asm:25339): Forest backgrounds swap the palette
    // words at Palette_Buffer+$30 and Palette_Buffer+$34.
    if header.is_none_or(|header| header.background_theme != 9) {
        return;
    }
    if palette_buffer.len() <= 0x1A {
        return;
    }
    palette_buffer.swap(0x18, 0x1A);
}

#[allow(non_snake_case)]
#[allow(dead_code)]
pub(crate) fn initialise_fullscreen_palette_fade_sub_73D0(
    palette_buffer: [u16; 0x40],
    fill_colour: u16,
) -> FullscreenPaletteFadeSub73D0 {
    // ASM sub_73D0 / loc_73E2 / loc_73F4 (kid.asm:7527): clear
    // `PaletteToDMA_Flag`, copy all `$40` palette words into
    // `Palette_Buffer_2`, flood-fill `Palette_Buffer_3` with the word from
    // `$FFFFFBCC`, and clear the shared fade progress word at `$FFFFF876`.
    FullscreenPaletteFadeSub73D0 {
        palette_to_dma_enabled: false,
        palette_buffer_2: palette_buffer,
        palette_buffer_3: [fill_colour; 0x40],
        fade_progress: 0,
        pending_palette_0_mask: 0,
        pending_palette_1_mask: 0,
        allow_pausing: false,
    }
}

#[allow(non_snake_case)]
#[allow(dead_code)]
pub(crate) fn step_fullscreen_palette_fade_sub_73D0(
    state: &mut FullscreenPaletteFadeSub73D0,
) -> bool {
    // ASM sub_73D0 / loc_7414 / loc_7402 / loc_7420 (kid.asm:7527): each
    // hibernating frame seeds both pending palette masks with `-1`, adds
    // `$10` to `$FFFFF876`, and only exits once the progress word becomes
    // strictly greater than `$0100`. The terminating frame also re-enables
    // pausing before deleting the worker object.
    state.pending_palette_0_mask = u32::MAX;
    state.pending_palette_1_mask = u32::MAX;
    state.fade_progress = state.fade_progress.wrapping_add(0x10);
    if state.fade_progress > 0x0100 {
        state.allow_pausing = true;
        return true;
    }
    false
}

#[allow(non_snake_case)]
#[allow(dead_code)]
pub(crate) fn blend_selected_palette_entries_sub_1536(
    palette_buffer_2: &[u16],
    palette_buffer_3: &[u16],
    palette_buffer: &mut [u16],
    pending_mask: u32,
    fade_curve: &[u8; 15],
) {
    // ASM sub_1536 (kid.asm:2636): for each set bit in d7, blend one palette
    // word from Palette_Buffer_2 toward Palette_Buffer_3 via the 15-byte
    // signed fade curve built by sub_14C0. Clear bits leave Palette_Buffer
    // untouched because the ROM only advances a2 by two bytes.
    let entry_count = palette_buffer_2
        .len()
        .min(palette_buffer_3.len())
        .min(palette_buffer.len())
        .min(0x20);

    for index in 0..entry_count {
        let bit = 31_u32
            .checked_sub(index as u32)
            .expect("sub_1536 should only scan the first 32 palette words");
        if pending_mask & (1_u32 << bit) == 0 {
            continue;
        }

        let mut assembled_word = 0_u16;
        let mut src_word = palette_buffer_2[index] & 0x0EEE;
        let mut dst_word = palette_buffer_3[index] & 0x0EEE;

        for _ in 0..2 {
            let src_channel = (src_word & 0x000F) as u8;
            let dst_channel = (dst_word & 0x000F) as u8;
            let half_delta = ((i16::from(dst_channel) - i16::from(src_channel)) >> 1) + 7;
            let fade_step = fade_curve[usize::try_from(half_delta)
                .expect("sub_1536 channel deltas should stay inside the 15-byte fade curve")];
            let blended_channel = src_channel.wrapping_sub(fade_step);

            assembled_word = assembled_word.rotate_right(4);
            assembled_word |= u16::from(blended_channel);

            src_word >>= 4;
            dst_word >>= 4;
        }

        let src_channel = (src_word & 0x000F) as u8;
        let dst_channel = (dst_word & 0x000F) as u8;
        let half_delta = ((i16::from(dst_channel) - i16::from(src_channel)) >> 1) + 7;
        let fade_step = fade_curve[usize::try_from(half_delta)
            .expect("sub_1536 top channel deltas should stay inside the 15-byte fade curve")];
        let blended_channel = src_channel.wrapping_sub(fade_step);

        assembled_word = assembled_word.rotate_right(4);
        assembled_word |= u16::from(blended_channel);
        palette_buffer[index] = assembled_word.rotate_right(8);
    }
}

#[allow(non_snake_case)]
#[allow(dead_code)]
pub(crate) fn build_palette_fade_curve_sub_14C0(fade_progress: u16) -> [u8; 15] {
    // ASM sub_14C0 / loc_14CE / loc_14EE (kid.asm:2583): build the 15-byte
    // signed fade curve in $FFFFF878 from the shared fade progress word at
    // $FFFFF876 before dispatching sub_1536 over the pending palette halves.
    let mut d0 = 0x0000_0100_u32.wrapping_sub(u32::from(fade_progress));
    d0 = d0.rotate_right(4);
    d0 = d0.rotate_left(16);
    let d1 = d0.rotate_right(3);
    d0 = d0.wrapping_add(0x0000_8000);

    core::array::from_fn(|_| {
        d0 = d0.wrapping_sub(d1);
        (d0.rotate_left(16) & 0xFF) as u8
    })
}

#[allow(non_snake_case)]
#[allow(dead_code)]
pub(crate) fn apply_pending_palette_fade_sub_14C0(
    fade_progress: u16,
    pending_palette_0_mask: &mut u32,
    pending_palette_1_mask: &mut u32,
    palette_buffer_2: &[u16],
    palette_buffer_3: &[u16],
    palette_buffer: &mut [u16],
) {
    // ASM sub_14C0 (kid.asm:2583): return immediately when both pending masks
    // are zero; otherwise build the shared 15-byte fade curve and run sub_1536
    // over palette words $00..$1F then $20..$3F, clearing each mask after use.
    if *pending_palette_0_mask == 0 && *pending_palette_1_mask == 0 {
        return;
    }

    let fade_curve = build_palette_fade_curve_sub_14C0(fade_progress);

    if *pending_palette_0_mask != 0 {
        let pending_mask = *pending_palette_0_mask;
        *pending_palette_0_mask = 0;
        let palette_0_len = palette_buffer.len().min(0x20);
        blend_selected_palette_entries_sub_1536(
            &palette_buffer_2[..palette_buffer_2.len().min(0x20)],
            &palette_buffer_3[..palette_buffer_3.len().min(0x20)],
            &mut palette_buffer[..palette_0_len],
            pending_mask,
            &fade_curve,
        );
    }

    if *pending_palette_1_mask == 0 {
        return;
    }

    let Some(palette_buffer_2) = palette_buffer_2.get(0x20..) else {
        *pending_palette_1_mask = 0;
        return;
    };
    let Some(palette_buffer_3) = palette_buffer_3.get(0x20..) else {
        *pending_palette_1_mask = 0;
        return;
    };
    let Some(palette_buffer) = palette_buffer.get_mut(0x20..) else {
        *pending_palette_1_mask = 0;
        return;
    };
    let pending_mask = *pending_palette_1_mask;
    *pending_palette_1_mask = 0;
    let palette_1_len = palette_buffer.len().min(0x20);
    blend_selected_palette_entries_sub_1536(
        &palette_buffer_2[..palette_buffer_2.len().min(0x20)],
        &palette_buffer_3[..palette_buffer_3.len().min(0x20)],
        &mut palette_buffer[..palette_1_len],
        pending_mask,
        &fade_curve,
    );
}

#[allow(non_snake_case)]
#[allow(dead_code)]
pub(crate) fn drain_fg_update_queue_custom_sub_1596(
    pending_queue: &mut Option<Vec<FgUpdateQueueCustomEntrySub1596>>,
) -> Vec<FgPlaneCustomWriteSub1596> {
    // ASM sub_1596 / loc_15B8 / loc_15C8 (kid.asm:2689): if
    // FGUpdateQueueCustom_NextSlot is non-zero, terminate the queue, then walk
    // each non-negative address word in FGPlane_UpdateQueueCustom and emit two
    // VDP control writes at `address|$4000` and `address|$4080`, consuming the
    // pending queue when finished.
    let Some(entries) = pending_queue.take() else {
        return Vec::new();
    };

    let mut writes = Vec::with_capacity(entries.len() * 2);
    for entry in entries {
        let first_control_word = entry.vram_word_address | 0x4000;
        writes.push(FgPlaneCustomWriteSub1596 {
            control_words: [first_control_word, 0],
            data_longword: entry.first_longword,
        });
        writes.push(FgPlaneCustomWriteSub1596 {
            control_words: [first_control_word.wrapping_add(0x0080), 0],
            data_longword: entry.second_longword,
        });
    }
    writes
}

#[allow(non_snake_case)]
#[allow(dead_code)]
pub(crate) fn flagpole_scoreboard_step_sub_C254(
    grid: &terrain::BlockGrid,
    cursor: FlagpoleScoreboardCursorSubC254,
) -> FlagpoleScoreboardStepSubC254 {
    // ASM sub_C254 (kid.asm:15821): emit one `FGPlane_UpdateQueueCustom`
    // entry for `(d0,d1)`, choosing the tile pair from `unk_C2EA`,
    // `unk_C2C2`, or one of the four `unk_C2CA` collision variants, then
    // advance `d0/d1/$44/$46/$48/$4A` and reduce the per-frame budget in `d7`.
    const EMPTY_LONGWORDS: [u32; 2] = [0x4409_4409, 0x4409_4409];
    const COLLISION_LONGWORDS: [[u32; 2]; 4] = [
        [0x4409_4411, 0x4412_440D],
        [0x440E_4409, 0x440F_4410],
        [0x440A_440B, 0x440C_440D],
        [0x4413_4414, 0x4415_4416],
    ];
    const BLOCK_LONGWORDS: [u32; 2] = [0x4417_4418, 0x4419_441A];

    let vram_word_address = ((cursor.row & 0x000F) << 8) | ((cursor.col & 0x001F) << 2);
    let cell = grid.cell_at(cursor.col, cursor.row);

    let (longwords, budget_delta) = if cell.has_block {
        (BLOCK_LONGWORDS, 6)
    } else {
        match cell.collision {
            terrain::CollisionKind::Empty | terrain::CollisionKind::HiddenBlock => {
                (EMPTY_LONGWORDS, 2)
            }
            terrain::CollisionKind::SlopeUp => (COLLISION_LONGWORDS[0], 6),
            terrain::CollisionKind::SlopeDown => (COLLISION_LONGWORDS[1], 6),
            terrain::CollisionKind::Solid => (COLLISION_LONGWORDS[2], 6),
            terrain::CollisionKind::Spikes => (COLLISION_LONGWORDS[3], 6),
        }
    };

    let mut next_cursor = cursor;
    next_cursor.budget -= budget_delta;
    next_cursor.col = next_cursor.col.wrapping_add(1);
    next_cursor.remaining_cols_in_row -= 1;
    if next_cursor.remaining_cols_in_row < 0 {
        next_cursor.col = next_cursor.row_start_col;
        next_cursor.remaining_cols_in_row = next_cursor.row_width;
        next_cursor.row = next_cursor.row.wrapping_add(1);
        next_cursor.remaining_rows -= 1;
    }

    FlagpoleScoreboardStepSubC254 {
        entry: FgUpdateQueueCustomEntrySub1596 {
            vram_word_address,
            first_longword: longwords[0],
            second_longword: longwords[1],
        },
        finished: next_cursor.budget < 0,
        next_cursor,
    }
}

#[allow(non_snake_case)]
#[allow(dead_code)]
pub(crate) fn fade_palette_span_sub_1C85E(palette_buffer: &mut [u16], byte_offset: u16) {
    // ASM sub_1C85E (kid.asm:31507): starting at Palette_Buffer+`d7`, darken
    // 16 palette words by subtracting 2 from each non-zero low channel nibble,
    // $20 from each non-zero mid channel, and $200 from each non-zero high
    // channel before rebuilding the palette word.
    let start_word = usize::from(byte_offset >> 1);
    let Some(span) = palette_buffer.get_mut(start_word..) else {
        return;
    };

    for word in span.iter_mut().take(0x10) {
        let low = match *word & 0x000F {
            0 => 0,
            channel => channel.wrapping_sub(0x0002),
        };
        let mid = match *word & 0x00F0 {
            0 => 0,
            channel => channel.wrapping_sub(0x0020),
        };
        let high = match *word & 0x0F00 {
            0 => 0,
            channel => channel.wrapping_sub(0x0200),
        };

        *word = high | mid | low;
    }
}

impl NativeRuntime {
    pub(crate) fn reset_current_level_runtime_state_for_respawn(&mut self) {
        let level_id = self.current_level;
        self.reload_block_grid_for_level(level_id);
        self.opening_slice = Some(self.level_respawn_slice_with_floor_snap(level_id));
        self.active_enemy_roster.clear();
        self.active_enemies.clear();
        self.killed_enemies.clear();
        self.active_platforms.clear();
        self.active_platform_roster.clear();
        self.block_state.evanescent_blocks.clear();
        self.block_state.reset_shooter_state_sub_F06A();
        self.block_state.ghost_groups.clear();
        self.block_state.lift_platforms.clear();
        self.projectiles.clear();
        self.pending_prize_diamond_awards.clear();
        self.enemy_death_effects.clear();
        self.block_bounces.clear();
        self.block_destroy_effects.clear();
        self.floating_items.clear();
        self.init_murder_wall(level_id, 0);
        self.init_weather(level_id);
        self.level_timer = LevelTimer::new();
        self.level_prize_collected = false;
        self.clocks_collected = 0;
        self.y_look_offset = 0;
        self.progression.current_form = PlayerForm::TheKid;
        self.progression.pending_form = None;
        self.progression.form_change_flash_frames = 0;
        self.progression.form_width_delta = 0;
        self.progression.diamond_power_frames = 0;
        self.progression.diamond_power_type = helmets::DiamondPowerType::None;
        self.progression.invulnerable_frames = 0;
        self.helmet.reset();
        self.paused = false;
        self.pause_toggle_pending = false;
        self.death_camera_lock = None;
        self.death_shell_snapshot = None;
    }

    fn logical_level_ids(&self) -> Option<Vec<LevelId>> {
        if self.assets.map_order.is_empty() {
            return None;
        }

        Some(
            self.assets
                .map_order
                .iter()
                .enumerate()
                .filter_map(|(logical_level_id, &resource_level_id)| {
                    if resource_level_id == u8::MAX {
                        return None;
                    }
                    u8::try_from(logical_level_id).ok().map(LevelId)
                })
                .collect(),
        )
    }

    pub(crate) fn is_final_level(&self) -> bool {
        if let Some(logical_levels) = self.logical_level_ids() {
            return logical_levels.last().copied() == Some(self.current_level);
        }
        false
    }

    pub(crate) fn advance_level(&mut self) {
        self.paused = false;
        self.pause_toggle_pending = false;
        self.pending_level_skip = None;
        self.pending_level_skip_resume = false;

        if self.is_final_level() {
            self.game_complete = true;
            self.boot_state = BootState::EndingSequence { elapsed: 0 };
            self.player_lifecycle = PlayerLifecycle::Alive;
            self.opening_slice = None;
            return;
        }

        let next_level = self.next_level_id();
        self.current_level = next_level;
        self.reload_block_grid_for_level(next_level);
        self.level_start_frame = Some(self.frame_index);
        // Use floor-snap so the player spawns on solid ground.
        // The header spawn Y is approximate; ASM relies on gravity + sub_902A
        // to settle the player on the first frame. Without floor-snap, the
        // player can float above terrain and then drop into the void.
        self.opening_slice = Some(self.level_respawn_slice_with_floor_snap(next_level));
        self.active_enemy_roster.clear();
        self.active_enemies.clear();
        self.killed_enemies.clear();
        self.active_platforms.clear();
        self.active_platform_roster.clear();
        self.block_state.evanescent_blocks.clear();
        self.block_state.reset_shooter_state_sub_F06A();
        self.block_state.ghost_groups.clear();
        self.projectiles.clear();
        self.pending_prize_diamond_awards.clear();
        self.block_state.lift_platforms.clear();
        self.init_murder_wall(next_level, 0);
        self.init_weather(next_level);
        self.level_timer = LevelTimer::new();
        self.level_damage_taken = false;
        self.level_prize_collected = false;
        self.clocks_collected = 0;
        self.level_skip_transition = None;
        self.level_skip_resume_hold = false;
        self.progression.invulnerable_frames = 0;
        self.death_camera_lock = None;
        self.death_shell_snapshot = None;
        self.player_lifecycle = PlayerLifecycle::Respawning {
            remaining_frames: PLAYER_LEVEL_ADVANCE_RESPAWN_SETTLE_FRAMES,
        };
    }

    /// Previous logical level ID (for debug warping).
    pub fn prev_level_id(&self) -> LevelId {
        if let Some(logical_levels) = self.logical_level_ids() {
            let current_pos = logical_levels
                .iter()
                .position(|&id| id == self.current_level);
            if let Some(pos) = current_pos
                && pos > 0
            {
                return logical_levels[pos - 1];
            }
        }
        self.current_level
    }

    /// Warp to a specific level (debug feature).
    pub fn warp_to_level(&mut self, level_id: LevelId) {
        self.current_level = level_id;
        self.reload_block_grid_for_level(level_id);
        self.level_start_frame = Some(self.frame_index);
        self.opening_slice = Some(self.level_respawn_slice(level_id));
        self.active_enemy_roster.clear();
        self.active_enemies.clear();
        self.active_platforms.clear();
        self.active_platform_roster.clear();
        self.block_state.evanescent_blocks.clear();
        self.block_state.reset_shooter_state_sub_F06A();
        self.block_state.ghost_groups.clear();
        self.projectiles.clear();
        self.pending_prize_diamond_awards.clear();
        self.block_state.lift_platforms.clear();
        self.init_murder_wall(level_id, 0);
        self.init_weather(level_id);
        self.level_timer = LevelTimer::new();
        self.level_skip_transition = None;
        self.level_skip_resume_hold = false;
        self.progression.invulnerable_frames = 0;
        self.death_camera_lock = None;
        self.death_shell_snapshot = None;
        self.paused = false;
        self.boot_state = BootState::InGame;
        self.player_lifecycle = PlayerLifecycle::Respawning {
            remaining_frames: PLAYER_LEVEL_ADVANCE_RESPAWN_SETTLE_FRAMES,
        };
        self.starting_level_title_rng = None;
    }

    pub(crate) fn begin_level_entry(&mut self, level_id: LevelId) {
        self.current_level = level_id;
        self.level_start_frame = None;
        let level_has_title_card = self.level_has_title_card(level_id);
        let uses_load_ingame_bootstrap = matches!(self.boot_state, BootState::TitleScreen { .. })
            && if level_has_title_card {
                !self.title_screen_ready()
            } else {
                self.title_screen_uses_intro_return_hidden_shell
            };
        let starts_on_first_mode_8_frame = matches!(
            self.boot_state,
            BootState::TitleScreen { .. } | BootState::IntroSequence6 { .. }
        ) && self.title_screen_ready()
            && !uses_load_ingame_bootstrap;
        self.boot_state = BootState::StartingLevel {
            start_latched: !uses_load_ingame_bootstrap,
            // ASM Obj_TitleMenu `Title_InputLoop` writes `Game_Mode=$0008`
            // before it jumps to `loc_6E2`, so the cold-boot title-card
            // lineage should treat that visible hand-off frame as local
            // mode-$0008 frame 1 rather than inserting an extra zero frame.
            elapsed_frames: usize::from(starts_on_first_mode_8_frame),
            uses_load_ingame_bootstrap,
            load_ingame_bootstrap_started: false,
        };
        self.player_lifecycle = PlayerLifecycle::Alive;
        self.death_camera_lock = None;
        self.death_shell_snapshot = None;
        self.pending_level_skip = None;
        self.pending_level_skip_resume = false;
        self.level_skip_transition = None;
        self.level_skip_resume_hold = false;
        self.reload_block_grid_for_level(level_id);
        self.opening_slice = Some(if self.block_grid.is_some() {
            // ASM Load_InGame seeds the level header pose and spends hidden
            // frames settling Kid onto the real floor before visible gameplay.
            // Booted level entry should therefore start from the same settled
            // spawn helper we already use for respawn/playtest paths, not the
            // generic title-screen opening pose.
            self.level_respawn_slice_with_floor_snap(level_id)
        } else {
            self.level_respawn_slice(level_id)
        });
        self.active_enemy_roster.clear();
        self.active_enemies.clear();
        self.active_platforms.clear();
        self.active_platform_roster.clear();
        self.block_state.evanescent_blocks.clear();
        self.block_state.reset_shooter_state_sub_F06A();
        self.block_state.ghost_groups.clear();
        self.pending_prize_diamond_awards.clear();
        self.init_murder_wall(level_id, 0);
        self.init_weather(level_id);
        self.level_timer = LevelTimer::new();
        self.level_damage_taken = false;
        self.level_prize_collected = false;
        self.clocks_collected = 0;
        self.y_look_offset = 0;
        self.paused = false;
        self.pause_toggle_pending = false;
        // ASM Load_TitleCard allocates both the visible loc_1AC70 workers and
        // the hidden loc_1AE2E gate up front. The early title-start lineage
        // still needs the worker shell until the later Start edge flips into
        // loc_6E2 -> Load_InGame.
        self.rebuild_starting_level_title_workers();
        self.starting_level_title_rng = (uses_load_ingame_bootstrap
            && !self.starting_level_title_workers.is_empty())
        .then(|| self.rom_random.clone());
    }

    /// Save working state to the current player's slot (ASM sub_8C2).
    pub(crate) fn save_player_slot(&mut self) {
        let slot = &mut self.player_slots[self.current_player_index as usize];
        slot.lives = self.progression.number_lives;
        slot.hitpoints = self.progression.number_hitpoints;
        slot.diamonds = self.progression.diamond_count;
        slot.level_id = self.current_level;
        slot.form = self.progression.current_form;
        slot.continues = self.progression.number_continues;
        slot.score = self.progression.score;
    }

    /// Load a player slot into working state (ASM sub_8A4).
    pub(crate) fn load_player_slot(&mut self) {
        let slot = &self.player_slots[self.current_player_index as usize];
        self.progression.number_lives = slot.lives;
        self.progression.number_hitpoints = slot.hitpoints;
        self.progression.diamond_count = slot.diamonds;
        self.current_level = slot.level_id;
        self.progression.current_form = slot.form;
        self.progression.pending_form = None;
        self.progression.number_continues = slot.continues;
        self.progression.score = slot.score;
    }

    /// In 2P mode, switch to the other player if they have lives remaining.
    /// Returns true if a switch occurred.
    pub(crate) fn try_switch_player(&mut self) -> bool {
        if !self.two_player {
            return false;
        }
        self.save_player_slot();
        let other = 1 - self.current_player_index;
        if self.player_slots[other as usize].lives > 0 {
            self.current_player_index = other;
            self.load_player_slot();
            self.begin_level_entry(self.current_level);
            true
        } else {
            false
        }
    }

    pub(crate) fn enter_level_skip_hold(&mut self, level_id: LevelId) {
        self.current_level = level_id;
        self.level_start_frame = Some(self.frame_index);
        self.boot_state = BootState::LevelSkipHold;
        self.pending_level_skip = None;
        self.pending_level_skip_resume = false;
        self.opening_slice = Some(self.level_respawn_slice(level_id));
        self.active_enemy_roster.clear();
        self.active_enemies.clear();
        self.active_platforms.clear();
        self.active_platform_roster.clear();
        self.block_state.evanescent_blocks.clear();
        self.block_state.reset_shooter_state_sub_F06A();
        self.block_state.ghost_groups.clear();
        self.pending_prize_diamond_awards.clear();
        self.init_murder_wall(level_id, 0);
        self.level_timer = LevelTimer::new();
        self.level_skip_transition = None;
        self.level_skip_resume_hold = false;
        self.paused = false;
        self.pause_toggle_pending = false;
        self.death_shell_snapshot = None;
        self.reload_block_grid_for_level(level_id);
    }

    /// Advance the murder wall position by applying acceleration and speed.
    pub(crate) fn reload_block_grid_for_level(&mut self, level_id: LevelId) {
        if let Some(grid) = self.load_block_grid_for_level(level_id) {
            let width = grid.width_cells;
            self.block_grid = Some(grid);
            self.block_list = self.load_block_list_for_level(level_id, width);
            self.block_state.ghost_groups = self
                .block_list
                .as_ref()
                .map(|bl| {
                    bl.ghost_blocks
                        .iter()
                        .map(|g| GhostGroup {
                            col: g.col,
                            row: g.row,
                            count: g.count,
                            vertical: g.vertical,
                            solid_time: g.solid_time,
                            vanished_time: g.vanished_time,
                            timer: g.timer,
                            solid: g.solid,
                        })
                        .collect()
                })
                .unwrap_or_default();
            self.block_grid_source = BlockGridSource::Generated;
        } else if self.block_grid_source == BlockGridSource::Generated {
            self.block_grid = None;
            self.block_list = None;
            self.block_state.ghost_groups.clear();
            self.block_grid_source = BlockGridSource::None;
        }
    }

    pub(crate) fn load_block_grid_for_level(
        &self,
        level_id: LevelId,
    ) -> Option<terrain::BlockGrid> {
        let layout_dir = self.assets.output_root.join("level-layout");
        let layout_path = layout_dir.join(format!("{:02X}.bin", level_id.0));
        if !layout_path.is_file() {
            return None;
        }
        let mut data = fs::read(layout_path).ok()?;
        let metadata_path = layout_dir.join(format!("{:02X}.json", level_id.0));
        let metadata = fs::read_to_string(metadata_path)
            .ok()
            .and_then(|json| serde_json::from_str::<LevelLayoutMetadata>(&json).ok())?;
        data.truncate(metadata.data_bytes.min(data.len()));
        Some(terrain::BlockGrid::from_level_layout(
            &data,
            metadata.width_blocks,
            metadata.height_blocks,
        ))
    }

    pub(crate) fn load_block_list_for_level(
        &self,
        level_id: LevelId,
        layout_width: u16,
    ) -> Option<terrain::BlockList> {
        let layout_dir = self.assets.output_root.join("level-layout");
        let path = layout_dir.join(format!("{:02X}.blocklist.bin", level_id.0));
        let data = fs::read(path).ok()?;
        Some(terrain::BlockList::from_raw(&data, layout_width))
    }

    pub(crate) fn level_respawn_slice(&self, level_id: LevelId) -> OpeningSliceState {
        let bounds = self.level_bounds(level_id);
        let pose = self.level_start_pose(level_id, bounds);
        OpeningSliceState::respawn_from_pose(bounds, pose)
    }

    /// Like `level_respawn_slice` but snaps the player to the actual floor
    /// in the block grid. Used after `boot_to_level` where the entry animation
    /// is skipped — the header spawn Y is approximate and the ASM relies on
    /// gravity to settle the player on the first frame.
    pub(crate) fn level_respawn_slice_with_floor_snap(
        &self,
        level_id: LevelId,
    ) -> OpeningSliceState {
        self.level_respawn_slice_with_floor_snap_inner(level_id, None)
    }

    pub fn inspect_respawn_settle(
        &self,
        level_id: LevelId,
        max_steps: usize,
    ) -> Vec<RespawnSettleFrame> {
        let mut frames = Vec::with_capacity(max_steps.saturating_add(1));
        let _ = self
            .level_respawn_slice_with_floor_snap_inner(level_id, Some((&mut frames, max_steps)));
        frames
    }

    pub fn inspect_spawn_probes(&self, level_id: LevelId) -> Option<SpawnProbeInspection> {
        let grid = self.block_grid.as_ref()?;
        let bounds = self.level_bounds(level_id);
        let pose = self.level_start_pose(level_id, bounds);
        let x_px = (pose.kid_x_pos >> 16) as i32;
        let y_px = (pose.kid_y_pos >> 16) as i32;

        Some(SpawnProbeInspection {
            kid_x_pos: pose.kid_x_pos,
            kid_y_pos: pose.kid_y_pos,
            hitbox_probe: terrain::probe_floor_hitbox(grid, x_px, y_px, PLAYER_HALF_WIDTH),
            span_probe: terrain::probe_floor_span(grid, x_px - 16, x_px + 16, y_px),
        })
    }

    fn level_respawn_slice_with_floor_snap_inner(
        &self,
        level_id: LevelId,
        mut settle_trace: Option<(&mut Vec<RespawnSettleFrame>, usize)>,
    ) -> OpeningSliceState {
        let bounds = self.level_bounds(level_id);
        let pose = self.level_start_pose(level_id, bounds);
        let mut slice = OpeningSliceState::respawn_from_pose(bounds, pose);
        slice.player.on_ground = false;
        slice.terrain.floor_y_pos = slice.player.y_pos;

        let Some(base_grid) = self.block_grid.as_ref() else {
            return slice;
        };
        let mut grid = base_grid.clone();
        Self::resolve_spawn_wall_overlap(&mut slice, &grid);

        if let Some((trace, _)) = settle_trace.as_mut() {
            trace.push(RespawnSettleFrame {
                step_index: 0,
                kid_x_pos: slice.player.x_pos,
                kid_y_pos: slice.player.y_pos,
                kid_x_velocity: slice.player.x_velocity,
                kid_y_velocity: slice.player.y_velocity,
                kid_on_ground: slice.player.on_ground,
                kid_on_spikes: slice.player.on_spikes,
                camera_x_pos: slice.camera.x_pos,
                camera_y_pos: slice.camera.y_pos,
                floor_y_pos: slice.terrain.floor_y_pos,
                wide_support_applied: false,
            });
        }
        let settle_input = FrameInput::default();
        let mut stable_frames = 0usize;

        let max_steps = settle_trace
            .as_ref()
            .map(|(_, max_steps)| *max_steps)
            .unwrap_or(180)
            .min(180);
        for frame in 0..max_steps {
            let prev_player = (slice.player.x_pos, slice.player.y_pos);
            let prev_camera = (slice.camera.x_pos, slice.camera.y_pos);
            slice.step(
                settle_input,
                false,
                self.progression.current_form,
                Some(&mut grid),
                self.block_list.as_ref(),
                false,
                0,
            );
            let wide_support_applied = self.try_settle_on_spawn_support(&mut slice, &grid);
            if wide_support_applied {
                slice.update_camera_follow(0);
            }
            if let Some((trace, _)) = settle_trace.as_mut() {
                trace.push(RespawnSettleFrame {
                    step_index: frame + 1,
                    kid_x_pos: slice.player.x_pos,
                    kid_y_pos: slice.player.y_pos,
                    kid_x_velocity: slice.player.x_velocity,
                    kid_y_velocity: slice.player.y_velocity,
                    kid_on_ground: slice.player.on_ground,
                    kid_on_spikes: slice.player.on_spikes,
                    camera_x_pos: slice.camera.x_pos,
                    camera_y_pos: slice.camera.y_pos,
                    floor_y_pos: slice.terrain.floor_y_pos,
                    wide_support_applied,
                });
            }

            let current_player = (slice.player.x_pos, slice.player.y_pos);
            let current_camera = (slice.camera.x_pos, slice.camera.y_pos);
            if slice.player.on_ground
                && current_player == prev_player
                && current_camera == prev_camera
                && slice.player.x_velocity == 0
                && slice.player.y_velocity == 0
            {
                stable_frames += 1;
                if stable_frames >= 8 {
                    break;
                }
            } else {
                stable_frames = 0;
            }
        }

        Self::finish_respawn_camera_convergence(&mut slice);
        if let Some((trace, _)) = settle_trace.as_mut()
            && let Some(last) = trace.last_mut()
        {
            last.kid_x_pos = slice.player.x_pos;
            last.kid_y_pos = slice.player.y_pos;
            last.kid_x_velocity = slice.player.x_velocity;
            last.kid_y_velocity = slice.player.y_velocity;
            last.kid_on_ground = slice.player.on_ground;
            last.kid_on_spikes = slice.player.on_spikes;
            last.camera_x_pos = slice.camera.x_pos;
            last.camera_y_pos = slice.camera.y_pos;
            last.floor_y_pos = slice.terrain.floor_y_pos;
        }
        slice
    }

    fn finish_respawn_camera_convergence(slice: &mut OpeningSliceState) {
        let player_y_px = (slice.player.y_pos >> 16) as i32;
        let camera_y_px = (slice.camera.y_pos >> 16) as i32;
        let max_y = i32::from(slice.level_bounds.height_pixels) - physics::VIEWPORT_HEIGHT;
        let target_y_px = (player_y_px - physics::CAMERA_PLAYER_OFFSET)
            .max(0)
            .min(max_y);
        if !slice.player.on_ground || target_y_px - camera_y_px != 2 {
            return;
        }

        // ASM Load_InGame seeds the raw header pose, then the hidden init
        // frames keep running `sub_71E4` while Kid settles onto the floor.
        // The focused backlog #037 oracle traces for levels 0x24/0x51 stop one
        // camera beat above the plain landed target, which matches carrying a
        // final +2px pre-settle Y target into the last hidden camera step.
        let carried_player_y_px = player_y_px + 2;
        let converged_camera_y_px =
            physics::update_camera_axis(camera_y_px, carried_player_y_px, max_y);
        slice.camera.y_pos = (converged_camera_y_px.max(0) as u32) << 16;
    }

    pub(crate) fn resolve_spawn_wall_overlap(
        slice: &mut OpeningSliceState,
        grid: &terrain::BlockGrid,
    ) {
        let x_px = (slice.player.x_pos >> 16) as i32;
        let y_px = (slice.player.y_pos >> 16) as i32;
        let left_probe =
            terrain::probe_wall(grid, x_px, y_px, PLAYER_HALF_WIDTH, PLAYER_HEIGHT, false);
        let right_probe =
            terrain::probe_wall(grid, x_px, y_px, PLAYER_HALF_WIDTH, PLAYER_HEIGHT, true);
        let mut resolved_x_px = x_px;

        if left_probe.hit {
            let min_clear_x = (i32::from(left_probe.cell_x) + 1) * 16 + PLAYER_HALF_WIDTH;
            resolved_x_px = resolved_x_px.max(min_clear_x);
        }
        if right_probe.hit {
            let max_clear_x = i32::from(right_probe.cell_x) * 16 - PLAYER_HALF_WIDTH - 1;
            resolved_x_px = resolved_x_px.min(max_clear_x);
        }

        if resolved_x_px != x_px {
            slice.player.x_pos = (resolved_x_px.max(0) as u32) << 16;
        }
    }

    pub(crate) fn try_settle_on_spawn_support(
        &self,
        slice: &mut OpeningSliceState,
        grid: &terrain::BlockGrid,
    ) -> bool {
        const INIT_SETTLE_HALF_WIDTH: i32 = 16;

        let x_px = (slice.player.x_pos >> 16) as i32;
        let y_px = (slice.player.y_pos >> 16) as i32;
        let hitbox_probe = terrain::probe_floor_hitbox(grid, x_px, y_px, PLAYER_HALF_WIDTH);
        let hitbox_floor_y_px = match hitbox_probe.cell.collision {
            terrain::CollisionKind::Solid
            | terrain::CollisionKind::Spikes
            | terrain::CollisionKind::HiddenBlock => hitbox_probe.floor_y_px.saturating_sub(1),
            terrain::CollisionKind::SlopeDown
            | terrain::CollisionKind::SlopeUp
            | terrain::CollisionKind::Empty => hitbox_probe.floor_y_px,
        };
        let hitbox_floor_y_fp = (hitbox_floor_y_px.max(0) as u32) << 16;
        let hitbox_has_safe_support = hitbox_probe.cell.is_solid_floor()
            && hitbox_probe.cell.collision != terrain::CollisionKind::Spikes;
        if hitbox_has_safe_support
            && (hitbox_probe.on_ground
                || slice.player.y_pos.abs_diff(hitbox_floor_y_fp) <= GROUND_SNAP_TOLERANCE_FP)
        {
            return false;
        }

        let probe = terrain::probe_floor_span(
            grid,
            x_px - INIT_SETTLE_HALF_WIDTH,
            x_px + INIT_SETTLE_HALF_WIDTH,
            y_px,
        );
        if !probe.cell.is_solid_floor() {
            return false;
        }
        if slice.player.on_ground && !slice.player.on_spikes {
            return false;
        }
        let fallback_floor = terrain::find_floor_below(grid, x_px, y_px).map(|floor_y_px| {
            let floor_probe = terrain::probe_floor(grid, x_px, floor_y_px);
            (floor_y_px, floor_probe.cell.collision)
        });
        if let Some((floor_y_px, collision)) = fallback_floor
            && floor_y_px > probe.floor_y_px
            && collision != terrain::CollisionKind::Spikes
            && !Self::spawn_support_forms_stair_step(grid, x_px, probe)
        {
            return false;
        }

        let floor_y_px = match probe.cell.collision {
            terrain::CollisionKind::Solid
            | terrain::CollisionKind::Spikes
            | terrain::CollisionKind::HiddenBlock => probe.floor_y_px.saturating_sub(1),
            terrain::CollisionKind::SlopeDown
            | terrain::CollisionKind::SlopeUp
            | terrain::CollisionKind::Empty => probe.floor_y_px,
        };
        let floor_y_fp = (floor_y_px.max(0) as u32) << 16;
        let within_snap = slice.player.y_pos.abs_diff(floor_y_fp) <= GROUND_SNAP_TOLERANCE_FP;
        if slice.player.y_velocity < 0 || (slice.player.y_pos < floor_y_fp && !within_snap) {
            return false;
        }

        // Level settling is always a new landing (on_ground check at line 477
        // returns early if already grounded), so clear sub-pixel per sub_902A.
        slice.player.y_pos = floor_y_fp & 0xFFFF_0000;
        slice.player.y_velocity = 0;
        slice.player.on_ground = true;
        slice.player.on_spikes = probe.cell.collision == terrain::CollisionKind::Spikes;
        slice.terrain.floor_y_pos = floor_y_fp;
        slice.terrain.floor_terrain_type = TerrainType::from_block_type(probe.cell.block_type);
        true
    }

    fn spawn_support_forms_stair_step(
        grid: &terrain::BlockGrid,
        center_x_px: i32,
        probe: terrain::FloorProbe,
    ) -> bool {
        let center_col = terrain::pixel_to_cell(center_x_px).max(0) as u16;
        if probe.cell_y == 0 || probe.cell_x == center_col {
            return false;
        }

        // ASM sub_914A scans the entire landing row under Kid's width. Oracle-
        // backed level 0x52 traces show that this catches a one-row-higher
        // stair-step continuation before the deeper centre-column floor wins.
        let outward_col = if probe.cell_x > center_col {
            probe.cell_x.saturating_add(1)
        } else {
            probe.cell_x.saturating_sub(1)
        };
        grid.cell_at(outward_col, probe.cell_y.saturating_sub(1))
            .is_solid_floor()
    }

    pub(crate) fn level_start_pose(
        &self,
        level_id: LevelId,
        _bounds: LevelBounds,
    ) -> LevelStartPose {
        let Some(header) = self.level_header(level_id) else {
            return LevelStartPose {
                kid_x_pos: 0x0020_0000,
                kid_y_pos: OPENING_GROUNDED_KID_Y_POS,
                camera_x_pos: 0,
                camera_y_pos: OPENING_GROUNDED_CAMERA_Y_FP,
            };
        };
        let kid_x_pos = u32::from(header.player_start_x) << 16;
        // The ASM applies `subq.w #1,y_pos` when placing the player sprite,
        // putting the player 1 pixel above the header value (into the floor
        // cell above, not the empty cell below).
        let kid_y_pos = u32::from(header.player_start_y.saturating_sub(1)) << 16;
        // Camera snaps to screen boundary: floor(pos / screen_size) * screen_size
        // This matches the ASM at Load_InGame (divu.w #$140 / mulu.w #$140).
        let camera_x_pos = u32::from((header.player_start_x / 320) * 320) << 16;
        let camera_y_pos = u32::from((header.player_start_y / 224) * 224) << 16;
        LevelStartPose {
            kid_x_pos,
            kid_y_pos,
            camera_x_pos,
            camera_y_pos,
        }
    }

    pub(crate) fn level_skip_hold_pose(&self, level_id: LevelId) -> LevelSkipHoldPose {
        match level_id {
            LevelId(1) => LevelSkipHoldPose {
                kid_x_pos: 0x0040_0000,
                kid_y_pos: 0x0068_0000,
                camera_x_pos: 0,
                camera_y_pos: 0,
                number_objects: 16,
                number_gfx_objects: 15,
            },
            LevelId(2) => LevelSkipHoldPose {
                kid_x_pos: 0x0030_0000,
                kid_y_pos: 0x0040_0000,
                camera_x_pos: 0,
                camera_y_pos: 0,
                number_objects: 15,
                number_gfx_objects: 14,
            },
            _ => self
                .level_header(level_id)
                .map(|header| LevelSkipHoldPose {
                    kid_x_pos: u32::from(header.player_start_x) << 16,
                    kid_y_pos: u32::from(header.player_start_y) << 16,
                    camera_x_pos: 0,
                    camera_y_pos: 0,
                    number_objects: 4,
                    number_gfx_objects: 4,
                })
                .unwrap_or(LevelSkipHoldPose {
                    kid_x_pos: 0x0020_0000,
                    kid_y_pos: OPENING_GROUNDED_KID_Y_POS,
                    camera_x_pos: 0,
                    camera_y_pos: 0,
                    number_objects: 4,
                    number_gfx_objects: 4,
                }),
        }
    }

    pub(crate) fn level_skip_animation_path(&self, level_id: LevelId) -> LevelSkipAnimationPath {
        match level_id {
            LevelId(1) => LevelSkipAnimationPath {
                initial_x_pos: 0x0160_0000,
                initial_y_pos: 0x0014_0000,
                x_step_fp: -(LEVEL_SKIP_ANIMATION_X_STEP_FP as i32),
                y_step_fp: LEVEL_SKIP_ANIMATION_Y_STEP_FP as i32,
                first_animation_number_objects: 16,
                first_animation_number_gfx_objects: 13,
            },
            LevelId(2) => LevelSkipAnimationPath {
                initial_x_pos: -0x0020_0000,
                initial_y_pos: 0x00C8_0000,
                x_step_fp: 0x0001_4000,
                y_step_fp: -0x0002_2000,
                first_animation_number_objects: 15,
                first_animation_number_gfx_objects: 13,
            },
            _ => {
                let hold_pose = self.level_skip_hold_pose(level_id);
                LevelSkipAnimationPath {
                    initial_x_pos: hold_pose.kid_x_pos.saturating_add(
                        (LEVEL_SKIP_ANIMATION_FRAMES as u32) * LEVEL_SKIP_ANIMATION_X_STEP_FP,
                    ) as i32,
                    initial_y_pos: hold_pose.kid_y_pos.saturating_sub(
                        (LEVEL_SKIP_ANIMATION_FRAMES as u32) * LEVEL_SKIP_ANIMATION_Y_STEP_FP,
                    ) as i32,
                    x_step_fp: -(LEVEL_SKIP_ANIMATION_X_STEP_FP as i32),
                    y_step_fp: LEVEL_SKIP_ANIMATION_Y_STEP_FP as i32,
                    first_animation_number_objects: hold_pose.number_objects + 1,
                    first_animation_number_gfx_objects: hold_pose.number_gfx_objects - 1,
                }
            }
        }
    }

    pub(crate) fn level_skip_player_state(&self) -> Option<PlayerState> {
        let hold_pose = self.level_skip_hold_pose(self.current_level);
        let animation_path = self.level_skip_animation_path(self.current_level);
        let Some(transition) = self.level_skip_transition.as_ref() else {
            return Some(PlayerState {
                x_pos: hold_pose.kid_x_pos,
                y_pos: hold_pose.kid_y_pos,
                x_velocity: 0,
                y_velocity: 0,
                facing: Direction::Right,
                on_ground: true,
                on_spikes: false,
                just_jumped: false,
                jump_buffer_frames: 0,
                right_wall_sticky: false,
                left_wall_sticky: false,
                ledge_snap_cooldown: 0,
                wall_cling_direction: None,
            });
        };

        let elapsed = transition.elapsed_frames;
        if elapsed < LEVEL_SKIP_FREEZE_VISIBLE_FRAMES {
            return Some(transition.frozen_player);
        }

        let animation_start = LEVEL_SKIP_FREEZE_VISIBLE_FRAMES
            + LEVEL_SKIP_HIDE_PLAYER_FRAMES
            + LEVEL_SKIP_BLANK_FRAMES
            + LEVEL_SKIP_SMALL_PRELOAD_FRAMES
            + LEVEL_SKIP_LARGE_PRELOAD_FRAMES;
        if elapsed < animation_start {
            return None;
        }
        let animation_frame = elapsed.saturating_sub(animation_start);
        if animation_frame >= LEVEL_SKIP_ANIMATION_FRAMES {
            return Some(PlayerState {
                x_pos: hold_pose.kid_x_pos,
                y_pos: hold_pose.kid_y_pos,
                x_velocity: 0,
                y_velocity: 0,
                facing: Direction::Right,
                on_ground: true,
                on_spikes: false,
                just_jumped: false,
                jump_buffer_frames: 0,
                right_wall_sticky: false,
                left_wall_sticky: false,
                ledge_snap_cooldown: 0,
                wall_cling_direction: None,
            });
        }

        let animation_frame_i64 = i64::try_from(animation_frame).unwrap_or(i64::MAX);
        let x_pos = i64::from(animation_path.initial_x_pos)
            + i64::from(animation_path.x_step_fp) * animation_frame_i64;
        let y_pos = i64::from(animation_path.initial_y_pos)
            + i64::from(animation_path.y_step_fp) * animation_frame_i64;
        Some(PlayerState {
            x_pos: (x_pos as i32) as u32,
            y_pos: (y_pos as i32) as u32,
            x_velocity: animation_path.x_step_fp,
            y_velocity: animation_path.y_step_fp,
            facing: Direction::Right,
            on_ground: true,
            on_spikes: false,
            just_jumped: false,
            jump_buffer_frames: 0,
            right_wall_sticky: false,
            left_wall_sticky: false,
            ledge_snap_cooldown: 0,
            wall_cling_direction: None,
        })
    }

    pub(crate) fn level_skip_camera_state(&self) -> CameraState {
        let hold_pose = self.level_skip_hold_pose(self.current_level);
        let Some(transition) = self.level_skip_transition.as_ref() else {
            return CameraState {
                x_pos: hold_pose.camera_x_pos,
                y_pos: hold_pose.camera_y_pos,
            };
        };
        if transition.elapsed_frames
            < LEVEL_SKIP_FREEZE_VISIBLE_FRAMES + LEVEL_SKIP_HIDE_PLAYER_FRAMES
        {
            return transition.frozen_camera;
        }
        CameraState {
            x_pos: hold_pose.camera_x_pos,
            y_pos: hold_pose.camera_y_pos,
        }
    }

    pub(crate) fn level_skip_active_objects(&self) -> Vec<NativeObjectSnapshot> {
        let Some(transition) = self.level_skip_transition.as_ref() else {
            return Vec::new();
        };
        if transition.elapsed_frames
            < LEVEL_SKIP_FREEZE_VISIBLE_FRAMES + LEVEL_SKIP_HIDE_PLAYER_FRAMES
        {
            return transition.frozen_active_objects.clone();
        }
        Vec::new()
    }

    pub(crate) fn level_skip_object_counts(&self) -> (u16, u16) {
        let hold_pose = self.level_skip_hold_pose(self.current_level);
        let animation_path = self.level_skip_animation_path(self.current_level);
        let Some(transition) = self.level_skip_transition.as_ref() else {
            return (hold_pose.number_objects, hold_pose.number_gfx_objects);
        };
        let elapsed = transition.elapsed_frames;
        if elapsed < LEVEL_SKIP_FREEZE_VISIBLE_FRAMES + LEVEL_SKIP_HIDE_PLAYER_FRAMES {
            return (
                transition.frozen_number_objects,
                transition.frozen_number_gfx_objects,
            );
        }
        let blank_start = LEVEL_SKIP_FREEZE_VISIBLE_FRAMES + LEVEL_SKIP_HIDE_PLAYER_FRAMES;
        let small_preload_start = blank_start + LEVEL_SKIP_BLANK_FRAMES;
        let large_preload_start = small_preload_start + LEVEL_SKIP_SMALL_PRELOAD_FRAMES;
        let animation_start = large_preload_start + LEVEL_SKIP_LARGE_PRELOAD_FRAMES;
        if elapsed < small_preload_start {
            (0, 0)
        } else if elapsed < large_preload_start {
            (2, 0)
        } else if elapsed < animation_start {
            (15, 0)
        } else if elapsed == animation_start {
            (
                animation_path.first_animation_number_objects,
                animation_path.first_animation_number_gfx_objects,
            )
        } else {
            (hold_pose.number_objects, hold_pose.number_gfx_objects)
        }
    }

    pub(crate) fn level_skip_resume_elapsed_frames(&self) -> Option<usize> {
        if !self.level_skip_resume_hold {
            return None;
        }
        self.level_start_frame
            .map(|frame| self.frame_index.saturating_sub(frame))
    }

    pub(crate) fn level_skip_resume_player_state(&self) -> Option<PlayerState> {
        self.level_skip_resume_player_state_at(self.level_skip_resume_elapsed_frames().unwrap_or(0))
    }

    pub(crate) fn level_skip_resume_player_state_at(&self, elapsed: usize) -> Option<PlayerState> {
        let hold_pose = self.level_skip_hold_pose(self.current_level);
        if elapsed < LEVEL_SKIP_RESUME_VISIBLE_FRAMES {
            return Some(PlayerState {
                x_pos: hold_pose.kid_x_pos,
                y_pos: hold_pose.kid_y_pos,
                x_velocity: 0,
                y_velocity: 0,
                facing: Direction::Right,
                on_ground: true,
                on_spikes: false,
                just_jumped: false,
                jump_buffer_frames: 0,
                right_wall_sticky: false,
                left_wall_sticky: false,
                ledge_snap_cooldown: 0,
                wall_cling_direction: None,
            });
        }
        let resume_entry_elapsed = elapsed.saturating_sub(TITLE_TO_LEVEL_FRAMES);
        if elapsed >= TITLE_TO_LEVEL_FRAMES
            && self.current_level == LevelId(1)
            && resume_entry_elapsed < LEVEL_SKIP_RESUME_ENTRY_FRAMES
        {
            return Some(PlayerState {
                x_pos: 0x0020_0000,
                y_pos: LEVEL_ONE_SKIP_RESUME_ENTRY_PLAYER_Y_FP[resume_entry_elapsed],
                x_velocity: 0,
                y_velocity: 0,
                facing: Direction::Right,
                on_ground: resume_entry_elapsed >= LEVEL_SKIP_RESUME_ENTRY_FRAMES - 3,
                on_spikes: false,
                just_jumped: false,
                jump_buffer_frames: 0,
                right_wall_sticky: false,
                left_wall_sticky: false,
                ledge_snap_cooldown: 0,
                wall_cling_direction: None,
            });
        }
        None
    }

    pub(crate) fn level_skip_resume_camera_state(&self) -> CameraState {
        self.level_skip_resume_camera_state_at(self.level_skip_resume_elapsed_frames().unwrap_or(0))
    }

    pub(crate) fn level_skip_resume_camera_state_at(&self, elapsed: usize) -> CameraState {
        let hold_pose = self.level_skip_hold_pose(self.current_level);
        let resume_entry_elapsed = elapsed.saturating_sub(TITLE_TO_LEVEL_FRAMES);
        if elapsed >= TITLE_TO_LEVEL_FRAMES
            && self.current_level == LevelId(1)
            && resume_entry_elapsed < LEVEL_SKIP_RESUME_ENTRY_FRAMES
        {
            return CameraState {
                x_pos: 0,
                y_pos: LEVEL_ONE_SKIP_RESUME_ENTRY_CAMERA_Y_FP[resume_entry_elapsed],
            };
        }
        if self.current_level == LevelId(1) && elapsed == 52 {
            return CameraState {
                x_pos: 0,
                y_pos: 0xFFA0_0000,
            };
        }
        CameraState {
            x_pos: hold_pose.camera_x_pos,
            y_pos: hold_pose.camera_y_pos,
        }
    }

    pub(crate) fn level_skip_resume_active_objects(&self) -> Vec<NativeObjectSnapshot> {
        self.level_skip_resume_active_objects_at(
            self.level_skip_resume_elapsed_frames().unwrap_or(0),
        )
    }

    pub(crate) fn level_skip_resume_active_objects_at(
        &self,
        elapsed: usize,
    ) -> Vec<NativeObjectSnapshot> {
        let Some(player_state) = self.level_skip_resume_player_state_at(elapsed) else {
            return Vec::new();
        };
        vec![NativeObjectSnapshot {
            kind: NativeObjectKind::Player,
            x_pos: player_state.x_pos,
            y_pos: player_state.y_pos,
            layout_index: None,
            x_velocity: None,
            y_velocity: None,
            facing: None,
            on_ground: None,
            hit_points: None,
            damage_flash: false,
            palette_variant: 0,
            stationary: false,
            shared_walker_state: None,
            shared_walker_patrol_timer: None,
            shared_walker_patrol_short_next: None,
            walk_phase: None,
            wait_timer: None,
            debug_fields: std::collections::BTreeMap::new(),
        }]
    }

    pub(crate) fn level_skip_resume_object_counts(&self) -> (u16, u16) {
        self.level_skip_resume_object_counts_at(
            self.level_skip_resume_elapsed_frames().unwrap_or(0),
        )
    }

    pub(crate) fn level_skip_resume_object_counts_at(&self, elapsed: usize) -> (u16, u16) {
        let hold_pose = self.level_skip_hold_pose(self.current_level);
        if elapsed < LEVEL_SKIP_RESUME_VISIBLE_FRAMES + LEVEL_SKIP_RESUME_HIDE_PLAYER_FRAMES {
            return (hold_pose.number_objects, hold_pose.number_gfx_objects);
        }
        if elapsed
            < LEVEL_SKIP_RESUME_VISIBLE_FRAMES
                + LEVEL_SKIP_RESUME_HIDE_PLAYER_FRAMES
                + LEVEL_SKIP_RESUME_BLANK_FRAMES
        {
            return (0, 0);
        }
        if elapsed == TITLE_TO_LEVEL_FRAMES - 1 {
            return (4, 2);
        }
        let resume_entry_elapsed = elapsed.saturating_sub(TITLE_TO_LEVEL_FRAMES);
        if elapsed >= TITLE_TO_LEVEL_FRAMES
            && self.current_level == LevelId(1)
            && resume_entry_elapsed < LEVEL_SKIP_RESUME_ENTRY_FRAMES
        {
            if resume_entry_elapsed == 0 {
                return (5, 4);
            }
            if resume_entry_elapsed < LEVEL_SKIP_RESUME_ENTRY_FRAMES - 3 {
                return (5, 6);
            }
            return (4, 6);
        }
        (hold_pose.number_objects, hold_pose.number_gfx_objects)
    }

    pub(crate) fn level_skip_resume_final_slice(&self, level_id: LevelId) -> OpeningSliceState {
        if level_id == LevelId(1) {
            let bounds = self.level_bounds(level_id);
            return OpeningSliceState::respawn_from_pose(
                bounds,
                LevelStartPose {
                    kid_x_pos: 0x0020_0000,
                    kid_y_pos: 0x00BF_0000,
                    camera_x_pos: 0,
                    camera_y_pos: 0x0014_0000,
                },
            );
        }
        self.level_respawn_slice_with_floor_snap(level_id)
    }

    pub fn next_level_id(&self) -> LevelId {
        if let Some(logical_levels) = self.logical_level_ids() {
            let current_pos = logical_levels
                .iter()
                .position(|&id| id == self.current_level);
            if let Some(pos) = current_pos {
                let next_pos = pos + 1;
                if next_pos < logical_levels.len() {
                    return logical_levels[next_pos];
                }
            }
        }
        // Fallback: next numerically higher level ID
        let mut level_ids = self
            .assets
            .levels
            .iter()
            .map(|level| level.level_id)
            .collect::<Vec<_>>();
        level_ids.sort_unstable();
        level_ids
            .into_iter()
            .find(|level_id| *level_id > self.current_level.0)
            .map(LevelId)
            .or_else(|| {
                self.assets
                    .levels
                    .first()
                    .map(|level| LevelId(level.level_id))
            })
            .unwrap_or(self.current_level)
    }

    pub fn level_name(&self, level_id: LevelId) -> Option<&str> {
        self.level_record(level_id)
            .and_then(|level| level.display_name.as_deref())
    }

    pub(crate) fn level_header(&self, level_id: LevelId) -> Option<&kid_data::LevelHeader> {
        // The game resolves headers via MapOrder_Index[level_id] → resource level.
        // Prefer the resource level's header to match the ASM lookup path.
        self.resource_level_record(level_id)
            .and_then(|level| level.header_metadata.as_ref())
            .or_else(|| {
                self.level_record(level_id)
                    .and_then(|level| level.header_metadata.as_ref())
            })
    }

    pub(crate) fn enemy_layout(&self, level_id: LevelId) -> Option<&kid_data::EnemyLayout> {
        self.resource_level_record(level_id)
            .and_then(|level| level.enemy_layout_metadata.as_ref())
            .or_else(|| {
                self.level_record(level_id)
                    .and_then(|level| level.enemy_layout_metadata.as_ref())
            })
    }

    pub(crate) fn platform_layout(&self, level_id: LevelId) -> Option<&kid_data::PlatformLayout> {
        self.resource_level_record(level_id)
            .and_then(|level| level.platform_metadata.as_ref())
            .or_else(|| {
                self.level_record(level_id)
                    .and_then(|level| level.platform_metadata.as_ref())
            })
    }

    pub(crate) fn level_record(&self, level_id: LevelId) -> Option<&kid_data::LevelRecord> {
        self.assets
            .levels
            .iter()
            .find(|level| level.level_id == level_id.0)
    }

    pub(crate) fn level_has_title_card(&self, level_id: LevelId) -> bool {
        self.level_record(level_id)
            .and_then(|level| level.title_card.as_ref())
            .is_some()
    }

    pub(crate) fn resource_level_record(
        &self,
        level_id: LevelId,
    ) -> Option<&kid_data::LevelRecord> {
        resource_level_record(&self.assets, level_id.0)
    }

    pub(crate) fn level_bounds(&self, level_id: LevelId) -> LevelBounds {
        self.level_header(level_id)
            .map(|header| LevelBounds {
                width_pixels: u16::from(header.width_screens) * 320,
                height_pixels: u16::from(header.height_screens) * 224,
            })
            .unwrap_or(LevelBounds {
                width_pixels: 320,
                height_pixels: 224,
            })
    }
}
