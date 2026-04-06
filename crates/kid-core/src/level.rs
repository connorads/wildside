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
        header_player_start
    };
    (player_start, flag_position, false)
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
        let uses_load_ingame_bootstrap = matches!(self.boot_state, BootState::TitleScreen { .. })
            && !self.title_screen_ready()
            && self
                .level_record(level_id)
                .and_then(|level| level.title_card.as_ref())
                .is_some();
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
