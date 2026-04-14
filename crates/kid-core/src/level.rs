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
pub(crate) struct FgUpdateQueueCullAdvanceLoc167A {
    pub(crate) skipped_words: usize,
    pub(crate) next_x_word: Option<u16>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct EnemyLayoutResetAndSeekLoc31F8E {
    pub(crate) enemy_status_words: Vec<u16>,
    pub(crate) loaded_enemy_slot_sentinel: u8,
    pub(crate) enemy_layout_progress: u32,
    pub(crate) uses_vertical_axis: bool,
    pub(crate) active_camera_word: i16,
    pub(crate) current_header_threshold: u16,
    pub(crate) previous_header_threshold: u16,
    pub(crate) enemy_layout_header_addr: u32,
    pub(crate) enemy_layout_addr: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct EnemyLayoutResetAndSeekInputLoc31F8E {
    pub(crate) enemy_layout_header_addr: u32,
    pub(crate) enemy_layout_addr: u32,
    pub(crate) previous_header_threshold: u16,
    pub(crate) axis_selector_word: u16,
    pub(crate) camera_x_pos: i16,
    pub(crate) camera_y_pos: i16,
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
pub(crate) struct DecompressSpecialPaletteRowStartLoc14348 {
    pub(crate) row_index: u16,
    pub(crate) column_index: u16,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct DecompressToVramSpecialChunkSetupLoc14366 {
    pub(crate) decompress_chunk_status_seed: i16,
    pub(crate) decompress_chunk_a3_seed_addr: u32,
    pub(crate) restored_output_addr: u32,
    pub(crate) palette_permutation_table_addr: u32,
    pub(crate) byte_lookup_accumulator: u16,
    pub(crate) word_dbf_counter: u16,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct DecompressToVramSpecialReturnLoc143B2 {
    pub(crate) restored_input_addr: u32,
    pub(crate) vdp_data_port_addr: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct DecompressToRamPaletteRowStartLoc143CC {
    pub(crate) row_index: u16,
    pub(crate) column_index: u16,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct DecompressToRamChunkSetupLoc143EA {
    pub(crate) decompress_chunk_status_seed: i16,
    pub(crate) decompress_chunk_a3_seed_addr: u32,
    pub(crate) restored_output_addr: u32,
    pub(crate) vdp_data_port_addr: u32,
    pub(crate) word_dbf_counter: u16,
    pub(crate) palette_permutation_table_addr: u32,
    pub(crate) byte_lookup_accumulator: u16,
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
pub(crate) enum DecompressModeDispatchLoc139A6 {
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
pub(crate) enum DecompressModeDispatchLoc13A24 {
    DirectCopy {
        copied_byte: u8,
    },
    LongRef {
        output_copy_base_addr: u16,
        control_byte: u8,
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
pub(crate) struct DecompressToVramChunkSetupLoc14300 {
    pub(crate) vdp_data_port_addr: u32,
    pub(crate) word_dbf_counter: u16,
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

/// ASM loc_C1D0 / loc_C1DC (kid.asm:15765): one iteration of the flagpole
/// hologram fade-out loop.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct FlagpoleFadeStepLocC1D0 {
    pub(crate) fade_progress: u16,
    pub(crate) pending_palette_0_mask: u32,
    pub(crate) pending_palette_1_mask: u32,
    pub(crate) done: bool,
}

/// ASM loc_C056 (kid.asm:15646): object field offsets cleared during
/// `CharacterCollision_TouchFlagpole`.
///
/// The loop at loc_C056 iterates the object linked list (`Addr_FirstObjectSlot`)
/// and clears three longword fields per object (except the current `a5`):
/// ```text
/// loc_C056: _move.l 0(a0),d0; beq.s loc_C070; move.l d0,a0
///           cmp.l a5,a0; beq.s loc_C056   ; skip self
///           clr.l $36(a0); clr.l $3A(a0); clr.l $3E(a0)
///           bra.s loc_C056
/// ```
/// `$36`/`$3A` are x/y velocity; `$3E` is an auxiliary link/state pointer.
/// Clearing them freezes all other objects before the deletion pass at loc_C070.
#[allow(dead_code)]
pub(crate) const TOUCH_FLAGPOLE_CLEAR_OFFSETS_LOC_C056: [u16; 3] = [0x36, 0x3A, 0x3E];

/// ASM loc_C070 (kid.asm:15658): VDP register commands written during the
/// `CharacterCollision_TouchFlagpole` post-cleanup.
///
/// ```text
/// jsr (j_Delete_AllButCurrentObject).w
/// st ($FFFFFB6A).w                     ; flagpole-touched flag
/// move.w #$8200,4(a6)                  ; Scroll A name table base = $0000
/// move.w #$8407,4(a6)                  ; Scroll B name table base = $E000
/// clr.w (Level_Special_Effects).w      ; disable weather
/// ```
/// These two VDP writes reset the scroll plane addresses for the scoreboard
/// reveal sequence.  The hibernate of `$24` (36) frames follows.
#[allow(dead_code)]
pub(crate) const TOUCH_FLAGPOLE_VDP_CMDS_LOC_C070: [u16; 2] = [0x8200, 0x8407];

/// ASM loc_C070: hibernate duration (frames) before gfx object cleanup.
/// `move.w #$24,-(sp); jsr (j_Hibernate_Object).w`.
#[allow(dead_code)]
pub(crate) const TOUCH_FLAGPOLE_HIBERNATE_FRAMES_LOC_C070: u16 = 0x24;

/// ASM loc_C0B6 (kid.asm:15675): gfx object cleanup loop after the $24-frame
/// hibernate.  Walks `Addr_FirstGfxObjectSlot` linked list (each slot's offset
/// `0(a3)` points to the next), skips Kid's gfx object
/// (`cmp.l (Addr_GfxObject_Kid).w,a3`), and deallocates every other gfx object
/// via `jsr (j_loc_1078).w`.
///
/// ASM loc_C0C8 (kid.asm:15683): loop continuation — `move.l (sp)+,d0;
/// bne.s loc_C0B6` pops the stashed next-pointer and repeats until the linked
/// list is exhausted (d0 == 0).
///
/// Native does not model per-slot gfx objects; the deletion pass is implicit
/// when `LevelComplete` transitions to the next level.
#[allow(dead_code)]
pub(crate) const TOUCH_FLAGPOLE_GFX_CLEANUP_ENTRY_LOC_C0B6: u32 = 0xC0B6;

/// ASM loc_C186 (kid.asm:15734): hologram tilemap upload — outer row loop.
/// After `EniDec` decodes `MapEni_CC0E` into `Decompression_Buffer`, the ROM
/// writes the decoded tilemap to plane B at VRAM `$E000`:
/// ```text
/// move.l #vdpComm($E000,VRAM,WRITE),4(a6)
/// moveq  #$1B,d1                ; 28 rows (0..$1B inclusive via dbf)
/// lea    (Decompression_Buffer).l,a0
/// loc_C186:
///   moveq  #$27,d0              ; 40 words per row
/// ```
/// Each row copies 40 tilemap words then pads 24 zero words to fill the
/// 64-column nametable stride (40 + 24 = 64 words = 128 bytes per row).
#[allow(dead_code)]
pub(crate) const FLAGPOLE_HOLO_TILEMAP_ROWS_LOC_C186: u16 = 0x1B; // 28 rows (dbf)

/// ASM loc_C188 (kid.asm:15737): inner tilemap word copy loop.
/// `move.w (a0)+,(a6); dbf d0,loc_C188` — copies 40 decoded tilemap words
/// per row to the VDP data port.
#[allow(dead_code)]
pub(crate) const FLAGPOLE_HOLO_TILEMAP_COLS_LOC_C188: u16 = 0x27; // 40 words (dbf)

/// ASM loc_C190 (kid.asm:15742): zero-padding loop per tilemap row.
/// `move.w #0,(a6); dbf d0,loc_C190` — writes 24 zero words to fill the
/// remaining columns (64 − 40 = 24) in each nametable row, then
/// `dbf d1,loc_C186` loops back for the next row.
#[allow(dead_code)]
pub(crate) const FLAGPOLE_HOLO_TILEMAP_PAD_LOC_C190: u16 = 0x17; // 24 zero words (dbf)

/// ASM loc_C1AA (kid.asm:15750): palette backup before hologram screen.
/// `moveq #$1F,d0; lea (Palette_Buffer).l,a0; lea (Palette_Buffer_2).l,a1;
/// loc_C1AA: move.l (a0)+,(a1)+; dbf d0,loc_C1AA`.
/// Copies 32 longwords (64 words = 128 bytes) — the full palette —
/// from `Palette_Buffer` to `Palette_Buffer_2`.
#[allow(dead_code)]
pub(crate) const FLAGPOLE_PALETTE_BACKUP_COUNT_LOC_C1AA: u16 = 0x1F; // 32 longwords (dbf)

/// ASM loc_C1BC (kid.asm:15757): hologram palette load.
/// `moveq #$E,d0; lea Pal_D00C(pc),a0; lea (Palette_Buffer_3+$42).l,a1;
/// loc_C1BC: move.w (a0)+,(a1)+; dbf d0,loc_C1BC`.
/// Copies 15 words from `Pal_D00C` into `Palette_Buffer_3` at byte offset
/// `$42` (word index 33).
#[allow(dead_code)]
pub(crate) const FLAGPOLE_HOLO_PALETTE_WORD_COUNT_LOC_C1BC: u16 = 0x0E; // 15 words (dbf)

/// ASM loc_C1BC: destination byte offset within `Palette_Buffer_3`.
#[allow(dead_code)]
pub(crate) const FLAGPOLE_HOLO_PALETTE_DEST_OFFSET_LOC_C1BC: u16 = 0x42;

/// ASM loc_C1D0 (kid.asm:15765): flagpole hologram fade-out loop body.
/// ```text
/// loc_C1D0:
///   jsr   (j_Hibernate_Object_1Frame).w
///   subi.w #$10,($FFFFF876).w
///   bmi.s  loc_C1EE
/// ```
/// Each iteration hibernates one frame then decrements the shared fade
/// progress word by `$10`. When the result goes negative the fade is
/// complete and control falls through to `loc_C1EE` for the scoreboard
/// tile-bounds calculation.
#[allow(dead_code)]
pub(crate) const FLAGPOLE_FADE_STEP_LOC_C1D0: u16 = 0x10;

/// ASM loc_C1DC (kid.asm:15770): flagpole fade palette mask setup.
/// ```text
/// loc_C1DC:
///   move.l #$8000FFFF,($FFFFF888).w
///   move.l #$FFFF0000,($FFFFF88C).w
///   bra.s  loc_C1D0
/// ```
/// Seeds the two pending palette masks before each fade frame.
/// `$8000FFFF` covers the first palette half (top bit + lower 16 bits);
/// `$FFFF0000` covers the second palette half (upper 16 bits).
/// Entered both as the initial fall-through from `loc_C1BC` setup and
/// as the loop-back target from `loc_C1D0` while fade progress ≥ 0.
#[allow(dead_code)]
pub(crate) const FLAGPOLE_FADE_PALETTE_0_MASK_LOC_C1DC: u32 = 0x8000_FFFF;
#[allow(dead_code)]
pub(crate) const FLAGPOLE_FADE_PALETTE_1_MASK_LOC_C1DC: u32 = 0xFFFF_0000;

/// ASM loc_C1EE (kid.asm:15776): post-fade screen tile bounds for
/// scoreboard reveal.
/// ```text
/// loc_C1EE:
///   move.w (Camera_X_pos).w,d0
///   move.w d0,d2
///   addi.w #$13F,d2          ; d2 = cam_x + 319
///   move.w (Camera_Y_pos).w,d1
///   move.w d1,d3
///   addi.w #$DF,d3           ; d3 = cam_y + 223
///   lsr.w  #4,d0             ; pixel → tile
///   lsr.w  #4,d1
///   lsr.w  #4,d2
///   lsr.w  #4,d3
///   sub.w  d0,d2             ; column span
///   move.w d2,$44(a5)        ; remaining_cols_in_row
///   move.w d2,$46(a5)        ; row_width (backup)
///   sub.w  d1,d3             ; row span
///   move.w d3,$48(a5)        ; remaining_rows
///   move.w d0,$4A(a5)        ; row_start_col
/// ```
/// Converts camera pixel rectangle (320×224) into tile coordinates and
/// stores the cursor fields that `sub_C254` walks during the reveal.
#[allow(dead_code)]
pub(crate) const FLAGPOLE_SCREEN_WIDTH_OFFSET_LOC_C1EE: u16 = 0x013F; // 319
#[allow(dead_code)]
pub(crate) const FLAGPOLE_SCREEN_HEIGHT_OFFSET_LOC_C1EE: u16 = 0x00DF; // 223

/// ASM loc_C21E (kid.asm:15794): scoreboard reveal outer loop (per-frame).
/// ```text
/// loc_C21E:
///   jsr   (j_Hibernate_Object_1Frame).w
///   move.l (FGUpdateQueueCustom_NextSlot).w,a0
///   moveq  #$C,d7             ; budget = 12 per frame
/// ```
/// Each frame reloads the queue write pointer and resets the per-frame
/// tile budget to `$C` (12) before entering the inner loop at `loc_C228`.
#[allow(dead_code)]
pub(crate) const FLAGPOLE_SCOREBOARD_BUDGET_PER_FRAME_LOC_C21E: i16 = 0x0C;

/// ASM loc_C228 (kid.asm:15799): scoreboard reveal inner loop (per-tile).
/// ```text
/// loc_C228:
///   bsr.w  sub_C254
///   bmi.s  Score_Board        ; remaining_rows < 0 → done
///   tst.w  d7
///   bge.s  loc_C228           ; budget ≥ 0 → next tile
///   move.l a0,(FGUpdateQueueCustom_NextSlot).w
///   bra.s  loc_C21E           ; next frame
/// ```
/// Calls `sub_C254` per tile; the `bmi` tests `$48(a5)` (remaining_rows)
/// via the N flag set by `sub_C254`'s final `subq.w #1,$48(a5)`.
/// When budget `d7` exhausts, the queue slot is saved and the next frame
/// begins at `loc_C21E`.
#[allow(dead_code)]
pub(crate) const FLAGPOLE_SCOREBOARD_INNER_ENTRY_LOC_C228: u32 = 0xC228;

pub(crate) const SUB_28FC_SLOT_COUNT: usize = 6;
pub(crate) const SUB_28FC_SLOT_STRIDE_BYTES: usize = 0x52;
const SPRITE_TABLE_ADDR_SUB_B52: u32 = 0xFFFF_0000;
const SPRITE_DYNAMIC_ART_ADDR_SUB_B52: u32 = 0xFFFF_0500;
const EVANESCENT_OBJECT_RAM_ADDR_SUB_EABC: u32 = 0xFFFF_EBDA;
const EVANESCENT_OBJECT_SLOT_STRIDE_SUB_EABC: u16 = 0x000C;
const EVANESCENT_OBJECT_SLOT_COUNT_SUB_EABC: usize = 40;
const EVANESCENT_OBJECT_RAM_ADDR_LOC_DF30: u32 = 0xFFFF_E90A;
const EVANESCENT_OBJECT_SLOT_STRIDE_LOC_DF30: u16 = 0x000C;
const EVANESCENT_OBJECT_SLOT_COUNT_LOC_DF30: usize = 0x13;
const SHOOTER_OBJECT_RAM_ADDR_SUB_F06A: u32 = 0xFFFF_E7F2;
const SHOOTER_OBJECT_SLOT_STRIDE_SUB_F06A: u16 = 0x000E;
const SHOOTER_OBJECT_SLOT_COUNT_SUB_F06A: usize = 20;
const SHOOTER_OBJECT_RAM_ADDR_LOC_DF50: u32 = 0xFFFF_E9FA;
const SHOOTER_OBJECT_SLOT_STRIDE_LOC_DF50: u16 = 0x000C;
const SHOOTER_OBJECT_SLOT_COUNT_LOC_DF50: usize = 0x27;
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
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct ObjectSlotOrderLocDE0 {
    pub(crate) slot_addr: u32,
    pub(crate) caller_order: u16,
}

#[cfg_attr(not(test), allow(dead_code))]
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct AllocatedObjectSlotListLocDF2 {
    pub(crate) inserted_index: usize,
    pub(crate) first_slot_addr: u32,
    pub(crate) slots: Vec<LinkedObjectSlotInit>,
}

#[cfg_attr(not(test), allow(dead_code))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct DeallocateObjectSlotPredecessorLocEA2 {
    pub(crate) predecessor_link_addr: u32,
    pub(crate) scanned_links: usize,
}

#[cfg_attr(not(test), allow(dead_code))]
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct DeallocatedObjectSlotLocEB0 {
    pub(crate) first_active_slot_addr: u32,
    pub(crate) first_free_slot_addr: u32,
    pub(crate) number_objects: u16,
    pub(crate) current_object_addr_after_return: u32,
    pub(crate) deallocated_gfx_object_addrs: Vec<u32>,
    pub(crate) slots: Vec<LinkedObjectSlotInit>,
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

#[cfg_attr(not(test), allow(dead_code))]
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ShooterObjectSlotsLocDF50 {
    pub(crate) next_free_shooter_object_slot_addr: u32,
    pub(crate) first_shooter_object_slot_addr: u32,
    pub(crate) shooter_slots: Vec<LinkedObjectSlotInit>,
}

#[cfg_attr(not(test), allow(dead_code))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct HiddenBlockEvanescentScanStartLocE07A {
    pub(crate) preincrement_cursor_addr: u32,
    pub(crate) first_record_addr: u32,
    pub(crate) layout_word_addr: u16,
}

#[cfg_attr(not(test), allow(dead_code))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct HiddenBlockMetadataRecordLocE082 {
    pub(crate) layout_word_addr: u16,
    pub(crate) control_word: i16,
}

#[cfg_attr(not(test), allow(dead_code))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct HiddenBlockEvanescentScanLocE082 {
    pub(crate) matched_record_addr: u32,
    pub(crate) scanned_records: usize,
    pub(crate) evanescent_sprite_words: Option<(u16, u16)>,
}

#[cfg_attr(not(test), allow(dead_code))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct HiddenBlockRevealWorkerLocE096 {
    pub(crate) revealed_layout_byte: u8,
    pub(crate) allocation_spec: u16,
    pub(crate) object_handler_addr: u32,
    pub(crate) block_col: u16,
    pub(crate) block_row: u16,
    pub(crate) layout_word_addr: u32,
    pub(crate) sprite_tile_word: u16,
    pub(crate) sprite_mapping: u16,
}

#[cfg_attr(not(test), allow(dead_code))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct HiddenBlockPrizeRevealScanTailLocE48E {
    pub(crate) next_block_col: u16,
    pub(crate) next_layout_word_addr: u16,
    pub(crate) next_dbf_counter: u16,
    pub(crate) loops_to_sub_e446: bool,
}

#[cfg_attr(not(test), allow(dead_code))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct EvanescentBlockRemovalGateLocEA3E {
    pub(crate) next_timer: u16,
    pub(crate) should_try_remove_block: bool,
}

#[cfg_attr(not(test), allow(dead_code))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct EvanescentBlockEraseMatchLocEA5A {
    pub(crate) should_erase_block_from_level_layout: bool,
    pub(crate) erased_block_col: Option<u16>,
    pub(crate) erased_block_row: Option<u16>,
    pub(crate) final_layout_cell_word: u16,
}

#[cfg_attr(not(test), allow(dead_code))]
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct EvanescentObjectListMoveLocEA72 {
    pub(crate) first_active_slot_addr: u32,
    pub(crate) first_free_slot_addr: u32,
    pub(crate) next_active_slot_to_process_addr: u32,
    pub(crate) slots: Vec<LinkedObjectSlotInit>,
}

#[cfg_attr(not(test), allow(dead_code))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct EvanescentObjectPredecessorLocEA86 {
    pub(crate) predecessor_slot_addr: u32,
    pub(crate) scanned_links: usize,
}

#[cfg_attr(not(test), allow(dead_code))]
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct EvanescentObjectSpliceLocEA92 {
    pub(crate) slots: Vec<LinkedObjectSlotInit>,
}

#[cfg_attr(not(test), allow(dead_code))]
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct EvanescentObjectFreeListPushLocEA98 {
    pub(crate) first_free_slot_addr: u32,
    pub(crate) should_continue_active_scan: bool,
    pub(crate) next_active_slot_to_process_addr: u32,
    pub(crate) slots: Vec<LinkedObjectSlotInit>,
}

#[cfg_attr(not(test), allow(dead_code))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct EvanescentSpriteScanReturnLocEAB2 {
    pub(crate) next_sprite_slot_addr: u32,
    pub(crate) number_sprites: u8,
}

#[cfg_attr(not(test), allow(dead_code))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum EvanescentBulletHitboxScanLocEAE4 {
    SkipToLoadBullets,
    Scan {
        top_row: u16,
        bottom_row: u16,
        left_col: u16,
        right_col: u16,
        row_dbf_counter: u16,
        col_dbf_counter: u16,
        found_evanescent_block: bool,
    },
}

#[cfg_attr(not(test), allow(dead_code))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct EvanescentBulletHitboxScanInputLocEAE4 {
    pub(crate) skip_scan_flag: bool,
    pub(crate) kid_hitbox_top: u16,
    pub(crate) kid_hitbox_bottom: u16,
    pub(crate) kid_hitbox_left: u16,
    pub(crate) kid_hitbox_right: u16,
    pub(crate) level_height_pixels: u16,
    pub(crate) level_width_pixels: u16,
}

#[cfg_attr(not(test), allow(dead_code))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct EvanescentBulletBottomClampLocEAF8 {
    pub(crate) bottom_px: u16,
}

#[cfg_attr(not(test), allow(dead_code))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct EvanescentBulletVerticalRowsLocEB0C {
    pub(crate) top_row: u16,
    pub(crate) bottom_row: u16,
    pub(crate) row_dbf_counter: u16,
    pub(crate) first_row_pointer_table_byte_offset: u16,
}

#[cfg_attr(not(test), allow(dead_code))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct EvanescentBulletRightClampLocEB2A {
    pub(crate) right_px: u16,
}

#[cfg_attr(not(test), allow(dead_code))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct EvanescentBulletHorizontalColumnsLocEB3E {
    pub(crate) left_col: u16,
    pub(crate) right_col: u16,
    pub(crate) col_dbf_counter: u16,
    pub(crate) first_cell_byte_offset: u16,
}

#[cfg_attr(not(test), allow(dead_code))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct EvanescentBulletScanHitLocEB48 {
    pub(crate) col: u16,
    pub(crate) row: u16,
}

#[cfg_attr(not(test), allow(dead_code))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum EvanescentBulletCellScanLocEB62 {
    Continue,
    TriggerEvanescentBlock,
}

#[cfg_attr(not(test), allow(dead_code))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct EvanescentBulletLoadCandidateLocEC52 {
    pub(crate) layout_word_addr: u16,
    pub(crate) block_x_px: u16,
    pub(crate) block_y_px: u16,
    pub(crate) direction_bits: u16,
}

#[cfg_attr(not(test), allow(dead_code))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct LoadedShooterObjectLocEC76 {
    pub(crate) slot_addr: u32,
    pub(crate) layout_word_addr: u16,
    pub(crate) block_x_px: u16,
    pub(crate) block_y_px: u16,
    pub(crate) direction: u16,
    pub(crate) state: u16,
    pub(crate) next_active_slot_addr: u32,
}

#[cfg_attr(not(test), allow(dead_code))]
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct EvanescentBulletDirectionLoadLocEC76 {
    pub(crate) next_free_shooter_object_slot_addr: u32,
    pub(crate) first_shooter_object_slot_addr: u32,
    pub(crate) shooter_slots: Vec<LinkedObjectSlotInit>,
    pub(crate) loaded_slots: Vec<LoadedShooterObjectLocEC76>,
    pub(crate) stopped_on_empty_free_list: bool,
}

#[cfg_attr(not(test), allow(dead_code))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct EvanescentBulletDirectionLoopTailLocEC9E {
    pub(crate) next_direction_counter: u16,
    pub(crate) loops_to_loc_ec76: bool,
}

#[cfg_attr(not(test), allow(dead_code))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct EvanescentBulletRecyclePredecessorLocECC4 {
    pub(crate) predecessor_slot_addr: u32,
    pub(crate) scanned_links: usize,
}

#[cfg_attr(not(test), allow(dead_code))]
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct EvanescentBulletRecycleSpliceLocECD2 {
    pub(crate) slots: Vec<LinkedObjectSlotInit>,
}

#[cfg_attr(not(test), allow(dead_code))]
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct EvanescentBulletRecycleFreeListPushLocECD8 {
    pub(crate) first_free_slot_addr: u32,
    pub(crate) should_continue_active_scan: bool,
    pub(crate) next_active_slot_to_process_addr: u32,
    pub(crate) slots: Vec<LinkedObjectSlotInit>,
}

#[cfg_attr(not(test), allow(dead_code))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum EvanescentBulletActionDispatchLocED04 {
    ProcessAction { action_byte: u8 },
    ContinueActiveScan { next_active_slot_addr: u32 },
    ProcessShooterObjects,
}

#[cfg_attr(not(test), allow(dead_code))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct EvanescentBulletSpriteLocED16 {
    pub(crate) y: u16,
    pub(crate) link: u16,
    pub(crate) tile_word: u16,
    pub(crate) x: u16,
}

#[cfg_attr(not(test), allow(dead_code))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct EvanescentBulletActionInputLocED16 {
    pub(crate) action_byte: u8,
    pub(crate) block_col: u16,
    pub(crate) block_row: u16,
    pub(crate) min_visible_col: u16,
    pub(crate) max_visible_col: u16,
    pub(crate) min_visible_row: u16,
    pub(crate) max_visible_row: u16,
    pub(crate) camera_x_px: u16,
    pub(crate) camera_y_px: u16,
    pub(crate) current_sprite_link: u16,
    pub(crate) loc_edbc_layout_offset: u16,
}

#[cfg_attr(not(test), allow(dead_code))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum EvanescentBulletActionLocED16 {
    PositiveBranchToLocEDBC {
        layout_word: u16,
    },
    NegativeAction {
        sprite: Option<EvanescentBulletSpriteLocED16>,
        should_update_layout: bool,
        use_collision_overlay: bool,
    },
}

#[cfg_attr(not(test), allow(dead_code))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct FgUpdateQueueCellLocED98 {
    pub(crate) block_col: u16,
    pub(crate) block_row: u16,
    pub(crate) layout_word: u16,
}

#[cfg_attr(not(test), allow(dead_code))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum EvanescentBulletLayoutWriteNextLocED98 {
    ContinueActiveScan { next_active_slot_addr: u32 },
    ProcessShooterObjects,
}

#[cfg_attr(not(test), allow(dead_code))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct EvanescentBulletLayoutWriteLocED98 {
    pub(crate) layout_word_addr: u16,
    pub(crate) final_layout_word: u16,
    pub(crate) queued_cell: FgUpdateQueueCellLocED98,
    pub(crate) next_step: EvanescentBulletLayoutWriteNextLocED98,
}

#[cfg_attr(not(test), allow(dead_code))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ShooterObjectMovingRightNextLocEE2A {
    ContinueMovingRight,
    InteractLevelLayout,
}

#[cfg_attr(not(test), allow(dead_code))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct ShooterObjectMovingRightGateLocEE2A {
    pub(crate) state_word: u16,
    pub(crate) layout_word_addr: u16,
    pub(crate) next_step: ShooterObjectMovingRightNextLocEE2A,
}

#[cfg_attr(not(test), allow(dead_code))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct ShooterObjectSpriteTailLocEE52 {
    pub(crate) x_pos: u16,
    pub(crate) sprite_x_offset: i16,
    pub(crate) sprite_y_offset: i16,
    pub(crate) tile_word: u16,
}

#[cfg_attr(not(test), allow(dead_code))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ShooterObjectMovingUpNextLocEE80 {
    ContinueMovingUp,
    InteractLevelLayout,
}

#[cfg_attr(not(test), allow(dead_code))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct ShooterObjectMovingUpGateLocEE80 {
    pub(crate) state_word: u16,
    pub(crate) layout_word_addr: u16,
    pub(crate) next_step: ShooterObjectMovingUpNextLocEE80,
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct PaletteFadeOutInitLoc28A6 {
    pub(crate) palette_buffer_2: [u16; 0x40],
    pub(crate) palette_buffer_3: [u16; 0x40],
    pub(crate) fade_progress: u16,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct PaletteFadeOutStepLoc28C6 {
    pub(crate) fade_progress: u16,
    pub(crate) pending_palette_0_mask: u32,
    pub(crate) pending_palette_1_mask: u32,
    pub(crate) continue_fading: bool,
}

const SUB_292E_PIECE_COUNT: usize = 5;
pub(crate) const SUB_292E_SLOT_ACTIVE_OFFSET: usize = 0;
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
pub(crate) fn decompress_to_vram_chunk_setup_loc_14300(
    buffer_start_addr: u32,
    buffer_end_addr: u32,
) -> DecompressToVramChunkSetupLoc14300 {
    // ASM loc_14300 (kid.asm:27152-27162) snapshots the current
    // `Decompression_Buffer` cursor with `move.l a2,a4`, calls
    // `Decompress_Chunk`, points `a6` at VDP data port `$C00000`, then turns
    // the emitted byte span `(a2-a4)` into the inclusive `dbf d3,loc_14314`
    // word counter via `lsr.w #1,d3` followed by `subq.w #1,d3`.
    let emitted_word_count = buffer_end_addr.wrapping_sub(buffer_start_addr) >> 1;

    DecompressToVramChunkSetupLoc14300 {
        vdp_data_port_addr: 0x00C0_0000,
        word_dbf_counter: emitted_word_count.wrapping_sub(1) as u16,
    }
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn decompress_to_vram_word_transfer_loc_14314(
    buffered_words: &[u16],
    dbf_counter: u16,
) -> Vec<u16> {
    // ASM loc_14314 (kid.asm:27164-27165) writes one word from `(a4)+` to the
    // VDP data port `(a6)` before each `dbf d3,loc_14314` test, so the loop
    // transfers exactly `d3 + 1` buffered words and `d3 = 0` still emits one
    // VRAM word before the later `tst.w d0` status gate.
    buffered_words
        .iter()
        .copied()
        .take(usize::from(dbf_counter) + 1)
        .collect()
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn decompress_to_vram_buffer_rollover_loc_1432C(
    buffered_longwords: &[u32],
    dbf_counter: u16,
) -> Vec<u32> {
    // ASM loc_1432C (kid.asm:27174-27175) reseeds `a2` to
    // `Decompression_Buffer`, `a4` to `Decompression_Buffer+$800`, loads
    // `d3 = #$1FF`, then copies one longword from `(a4)+` to `(a2)+` before
    // each `dbf d3,loc_1432C` test. That makes the rollover move exactly
    // `d3+1` longwords, so the second `$800` bytes slide down over the first.
    buffered_longwords
        .iter()
        .copied()
        .take(usize::from(dbf_counter) + 1)
        .collect()
}

// ASM loc_64D0 (kid.asm:6502): dead code in the ROM — an orphan nibble-
// interleaving routine identical to loc_14348/loc_1434A below.  It sits between
// `sub_61CA`'s rts (kid.asm:6472) and `Init_Timer_and_Bonus_Flags`
// (kid.asm:6551) with no reachable entry point (no label, no branch, no jump
// table).  The preceding bytes ($43F9 $FFFF0280 $7200) decode to
// `clr.w ($FFFF0280).l; moveq #0,d1` — setup for the same 16×16
// nibble-pack loop that `DecompressToVRAM_Special` uses at loc_14348.
//
// ASM loc_64D2 (kid.asm:6505): the inner column loop body —
//   move.b (a0,d1.w),d3; lsl.b #4,d3; or.b (a0,d2.w),d3; move.b d3,(a1)+
//   addq.w #1,d2; cmpi.w #$10,d2; bne.s loc_64D2
// Packs permutation[d1] into the high nibble, permutation[d2] into the low
// nibble, for each column d2 in 0..16 within the current row d1.  Identical
// to the inner loop at loc_1434A in DecompressToVRAM_Special.
//
// A second phase at loc_6504 reads remap indices from a4 into $FFFF0380,
// mirroring loc_14382's palette-table lookup.  Both phases match
// `decompress_special_palette_row_bytes_loc_1434A` and
// `decompress_to_vram_special_word_transfer_loc_14382` below, so no separate
// native implementation is needed.

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn decompress_special_palette_row_start_loc_14348(
    row_index: u16,
) -> DecompressSpecialPaletteRowStartLoc14348 {
    // ASM loc_14348 (kid.asm:27199) is the outer nibble-permutation loop's
    // row prologue: `d3` already holds the current high-nibble selector and
    // `moveq #0,d4` resets the low-nibble column index before `loc_1434A`
    // emits the next 16 packed bytes.
    DecompressSpecialPaletteRowStartLoc14348 {
        row_index,
        column_index: 0,
    }
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn decompress_special_palette_row_bytes_loc_1434A(
    permutation: &[u8; 16],
    row_index: u16,
    dbf_counter: u16,
) -> Vec<u8> {
    // ASM loc_1434A (kid.asm:27202-27209) reads `move.b (a3,d3.w),d5`,
    // shifts it into the high nibble, ORs in `move.b (a3,d4.w),d5`, stores
    // that packed byte to the palette scratch table at `(a4)+`, then
    // `addq.w #1,d4` / `cmpi.w #$10,d4` repeat until all 16 low-nibble
    // permutations have been emitted for the current `d3` row.
    let high_nibble = permutation[usize::from(row_index & 0x000F)] << 4;

    permutation
        .iter()
        .copied()
        .take(usize::from(dbf_counter) + 1)
        .map(|low_nibble| high_nibble | low_nibble)
        .collect()
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn decompress_to_vram_special_chunk_setup_loc_14366(
    buffer_start_addr: u32,
    buffer_end_addr: u32,
    saved_output_addr: u32,
) -> DecompressToVramSpecialChunkSetupLoc14366 {
    // ASM loc_14366 (kid.asm:27214-27226) seeds `d0` and `a3` with `-1`
    // before `Decompress_Chunk`, snapshots the current output cursor with
    // `move.l a2,a4`, turns the emitted byte span `(a2-a4)` into the inclusive
    // special-VRAM transfer loop word count, then restores `a6` from the saved stack
    // pointer and points `a3` at the special palette table scratch buffer.
    let emitted_word_count = buffer_end_addr.wrapping_sub(buffer_start_addr) >> 1;

    DecompressToVramSpecialChunkSetupLoc14366 {
        decompress_chunk_status_seed: -1,
        decompress_chunk_a3_seed_addr: u32::MAX,
        restored_output_addr: saved_output_addr,
        palette_permutation_table_addr: 0xFFFF_0280,
        byte_lookup_accumulator: 0,
        word_dbf_counter: emitted_word_count.wrapping_sub(1) as u16,
    }
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn decompress_to_vram_special_word_transfer_loc_14382(
    permutation_table: &[u8; 256],
    buffered_bytes: &[u8],
    dbf_counter: u16,
) -> Vec<u16> {
    // ASM loc_14382 (kid.asm:27227-27234) reads two bytes from `(a4)+`, uses
    // each as an index into the 256-entry scratch permutation table at `a3`,
    // shifts the first lookup into the high byte with `lsl.w #8,d6`, overwrites
    // the low byte with the second lookup, writes the packed word to `(a6)+`,
    // then repeats exactly `d3+1` times via `dbf`.
    let word_count = usize::from(dbf_counter) + 1;

    buffered_bytes
        .chunks_exact(2)
        .take(word_count)
        .map(|byte_pair| {
            let high_byte = u16::from(permutation_table[usize::from(byte_pair[0])]);
            let low_byte = u16::from(permutation_table[usize::from(byte_pair[1])]);
            (high_byte << 8) | low_byte
        })
        .collect()
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn decompress_to_vram_special_buffer_rollover_loc_143AA(
    buffered_longwords: &[u32],
    dbf_counter: u16,
) -> Vec<u32> {
    // ASM loc_143AA (kid.asm:27242-27245) reloads `a2` with
    // `Decompression_Buffer`, points `a4` at `Decompression_Buffer+$800`,
    // copies one longword from `(a4)+` to `(a2)+` before each `dbf`, and then
    // loops back into the special chunk-setup path once the inclusive copy
    // count is exhausted.
    buffered_longwords
        .iter()
        .copied()
        .take(usize::from(dbf_counter) + 1)
        .collect()
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn decompress_to_vram_special_return_loc_143B2(
    saved_input_addr: u32,
) -> DecompressToVramSpecialReturnLoc143B2 {
    // ASM loc_143B2 (kid.asm:27248-27250) pops the caller's saved `a1` back
    // from the stack slot that `DecompressToVRAM_Special` pushed on entry, then
    // restores `a6` to the fixed VDP data port `$C00000` before returning.
    DecompressToVramSpecialReturnLoc143B2 {
        restored_input_addr: saved_input_addr,
        vdp_data_port_addr: 0x00C0_0000,
    }
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn decompress_to_ram_palette_row_start_loc_143CC(
    row_index: u16,
) -> DecompressToRamPaletteRowStartLoc143CC {
    // ASM loc_143CC (kid.asm:27269) is the `DecompressToRAM` outer
    // permutation-loop entry and simply resets `d4` with `moveq #0,d4`
    // before `loc_143CE` starts emitting the 16 packed bytes for the current
    // high-nibble row selected by `d3`.
    DecompressToRamPaletteRowStartLoc143CC {
        row_index,
        column_index: 0,
    }
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn decompress_to_ram_palette_row_bytes_loc_143CE(
    permutation: &[u8; 16],
    row_index: u16,
    start_column_index: u16,
) -> Vec<u8> {
    // ASM loc_143CE (kid.asm:27272-27279) reads the shared high nibble from
    // `(a3,d3.w)`, shifts it into the upper half of `d5` with `lsl.b #4,d5`,
    // ORs in each successive low nibble from `(a3,d4.w)`, stores the packed
    // byte to `(a4)+`, and keeps looping until `d4` reaches `$10`.
    let row_high_nibble = permutation[usize::from(row_index & 0x000F)] << 4;

    permutation
        .iter()
        .copied()
        .skip(usize::from(start_column_index))
        .map(|low_nibble| row_high_nibble | low_nibble)
        .collect()
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn decompress_to_ram_chunk_setup_loc_143EA(
    buffer_start_addr: u32,
    buffer_end_addr: u32,
    restored_output_addr: u32,
) -> DecompressToRamChunkSetupLoc143EA {
    // ASM loc_143EA (kid.asm:27284-27295) seeds `d0` and `a3` with `-1`
    // before `Decompress_Chunk`, snapshots the current output cursor with
    // `move.l a2,a4`, restores `a6` to the fixed VDP data port `$C00000`,
    // turns the emitted byte span `(a2-a4)` into the inclusive `dbf d3`
    // word count, repoints `a3` at the palette scratch table `$FFFF0280`,
    // and clears `d5` before `loc_1440A` starts packing words.
    let emitted_word_count = buffer_end_addr.wrapping_sub(buffer_start_addr) >> 1;

    DecompressToRamChunkSetupLoc143EA {
        decompress_chunk_status_seed: -1,
        decompress_chunk_a3_seed_addr: u32::MAX,
        restored_output_addr,
        vdp_data_port_addr: 0x00C0_0000,
        word_dbf_counter: emitted_word_count.wrapping_sub(1) as u16,
        palette_permutation_table_addr: 0xFFFF_0280,
        byte_lookup_accumulator: 0,
    }
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn decompress_to_ram_word_transfer_loc_1440A(
    permutation_table: &[u8; 256],
    buffered_bytes: &[u8],
    dbf_counter: u16,
) -> Vec<u16> {
    // ASM loc_1440A (kid.asm:27297-27305) reads two bytes from `(a4)+`, uses
    // each as an index into the 256-entry permutation table at `$FFFF0280`,
    // shifts the first lookup into the high byte with `lsl.w #8,d6`, overwrites
    // the low byte with the second lookup, writes the packed word to `(a6)`,
    // then repeats exactly `d3+1` times via `dbf`.
    let word_count = usize::from(dbf_counter) + 1;

    buffered_bytes
        .chunks_exact(2)
        .take(word_count)
        .map(|byte_pair| {
            let high_byte = u16::from(permutation_table[usize::from(byte_pair[0])]);
            let low_byte = u16::from(permutation_table[usize::from(byte_pair[1])]);
            (high_byte << 8) | low_byte
        })
        .collect()
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn decompress_to_ram_buffer_rollover_loc_14430(
    buffered_longwords: &[u32],
    dbf_counter: u16,
) -> Vec<u32> {
    // ASM loc_14430 (kid.asm:27311-27314) reloads `a2` with
    // `Decompression_Buffer`, points `a4` at `Decompression_Buffer+$800`,
    // copies one longword from `(a4)+` to `(a2)+` before each `dbf`, and then
    // loops back to `loc_143EA` once the inclusive rollover copy is exhausted.
    buffered_longwords
        .iter()
        .copied()
        .take(usize::from(dbf_counter) + 1)
        .collect()
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn indoor_rubber_block_source_words_loc_14506(
    mapping_words: &[u16],
    dbf_counter: u16,
) -> Vec<u16> {
    // ASM loc_14506 (kid.asm:27371-27374) reads one source word from `(a0)+`,
    // calls `sub_14582` to expand that tile into the scratch buffer, then
    // `dbf d2,loc_14506` repeats inclusively. With the `moveq #3,d2` setup in
    // `sub_144DA`, the loop consumes exactly four source words.
    mapping_words
        .iter()
        .copied()
        .take(usize::from(dbf_counter) + 1)
        .collect()
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn indoor_rubber_block_buffer_copy_loc_14512(
    buffered_longwords: &[u32],
    dbf_counter: u16,
) -> Vec<u32> {
    // ASM loc_14512 (kid.asm:27377-27378) copies one longword from `(a2)+` to
    // `(a1)+` before each `dbf d2,loc_14512` test. After `moveq #$1F,d2`, the
    // loop therefore copies exactly 32 longwords from the scratch cursor back
    // into `Decompression_Buffer`.
    buffered_longwords
        .iter()
        .copied()
        .take(usize::from(dbf_counter) + 1)
        .collect()
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn indoor_rubber_block_vram_walk_loc_1451E(
    start_vram_tile: u16,
    dbf_counter: u16,
) -> Vec<u16> {
    // ASM loc_1451E (kid.asm:27383-27386) copies the current VRAM tile from
    // `d3` into `d0`, calls `sub_14582`, increments `d3`, then lets the
    // inclusive `dbf d2,loc_1451E` loop keep walking one tile at a time. With
    // `moveq #7,d2`, the rubber-block upload visits eight consecutive tiles.
    let iterations = usize::from(dbf_counter) + 1;
    (0..iterations)
        .map(|step| start_vram_tile.wrapping_add(u16::try_from(step).unwrap_or(u16::MAX)))
        .collect()
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn indoor_rubber_block_repack_longword_loc_1453E(
    existing_longword: u32,
    buffered_longword: u32,
    seed_longword: u32,
    dbf_counter: u16,
) -> u32 {
    // ASM loc_1453E (kid.asm:27397-27405) shifts `d5` left four bits, rotates
    // both source longwords left by one nibble, prefers the rotated `(a1)` low
    // nibble when it is non-zero, and otherwise falls back to the rotated
    // scratch-buffer nibble from `d4`. The inclusive `dbf d1,loc_1453E` loop
    // therefore packs exactly `d1 + 1` nibbles into `d5`.
    let mut packed_longword = seed_longword;
    let mut existing_rotated = existing_longword;
    let mut buffered_rotated = buffered_longword;

    for _ in 0..=dbf_counter {
        packed_longword <<= 4;
        existing_rotated = existing_rotated.rotate_left(4);
        buffered_rotated = buffered_rotated.rotate_left(4);
        packed_longword = indoor_rubber_block_pack_selected_nibble_loc_14552(
            packed_longword,
            existing_rotated,
            buffered_rotated,
        );
    }

    packed_longword
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn indoor_rubber_block_pack_selected_nibble_loc_14552(
    shifted_seed_longword: u32,
    existing_rotated: u32,
    buffered_rotated: u32,
) -> u32 {
    // ASM loc_14552 (kid.asm:27407) ORs the nibble already loaded into `d3`
    // into `d5`. The preceding `bne.s loc_14552` keeps the rotated existing
    // nibble when it is non-zero; otherwise `move.w d4,d3` / `andi.w #$F,d3`
    // replaces it with the rotated scratch-buffer nibble before the OR.
    let selected_nibble = if (existing_rotated & 0xF) != 0 {
        existing_rotated & 0xF
    } else {
        buffered_rotated & 0xF
    };

    shifted_seed_longword | selected_nibble
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn indoor_rubber_block_repack_rows_loc_14538(
    existing_longwords: &[u32],
    buffered_longwords: &[u32],
    outer_dbf_counter: u16,
    inner_dbf_counter: u16,
) -> Vec<u32> {
    // ASM loc_14538 (kid.asm:27392-27407) reloads `d1=#7`, reads one existing
    // longword from `(a1)` and one buffered longword from `(a0)+`, runs the
    // `loc_1453E` nibble merge, stores the packed result to `(a1)+`, and then
    // lets the inclusive `dbf d0,loc_14538` loop advance to the next longword.
    existing_longwords
        .iter()
        .copied()
        .zip(buffered_longwords.iter().copied())
        .take(usize::from(outer_dbf_counter) + 1)
        .map(|(existing_longword, buffered_longword)| {
            indoor_rubber_block_repack_longword_loc_1453E(
                existing_longword,
                buffered_longword,
                0,
                inner_dbf_counter,
            )
        })
        .collect()
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn indoor_rubber_block_upload_vram_rows_loc_1456E(
    buffered_longwords: &[u32],
    dbf_counter: u16,
) -> Vec<u32> {
    // ASM loc_1456E (kid.asm:27416-27417) copies one longword from `(a0)+`
    // into the VDP data port at `(a6)` before each inclusive `dbf d0,loc_1456E`.
    // After `moveq #$3F,d0`, the upload therefore pushes 64 longwords from the
    // scratch area at `$FFFF78B2`.
    buffered_longwords
        .iter()
        .copied()
        .take(usize::from(dbf_counter) + 1)
        .collect()
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn indoor_rubber_block_read_vram_rows_loc_1459A(
    vram_longwords: &[u32],
    dbf_counter: u16,
) -> Vec<u32> {
    // ASM loc_1459A (kid.asm:27453-27454) reads one longword from the VDP data
    // port at `(a6)` into `(a1)+` before each inclusive `dbf d1,loc_1459A`.
    // Inside `sub_14582`, `moveq #7,d1` therefore captures eight consecutive
    // longwords from the selected VRAM tile into the scratch buffer.
    vram_longwords
        .iter()
        .copied()
        .take(usize::from(dbf_counter) + 1)
        .collect()
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
pub(crate) fn decompress_mode_dispatch_loc_139A6(
    bit_buffer: u8,
    output_addr: u16,
    direct_copy_byte: u8,
    short_ref_control_byte: u8,
    short_ref_back_distance: u8,
) -> DecompressModeDispatchLoc139A6 {
    // ASM loc_139A6 (kid.asm:26690-26701) consumes two mode bits from the
    // carried `d1` byte: `1` branches to `loc_13A22` for another literal-byte
    // copy, `01` seeds `a6` from `a2` and branches into the later long-ref
    // header at `loc_139C8`, and `00` loads a fresh control byte from `(a0)+`
    // plus the short back-reference byte from `(a1)+`. The high bit of that
    // fresh control byte decides whether the short-reference path copies just
    // the shared two-byte tail at `loc_139BC` or an extra leading byte first.
    if bit_buffer & 0x80 != 0 {
        return DecompressModeDispatchLoc139A6::DirectCopy {
            copied_byte: direct_copy_byte,
        };
    }

    if bit_buffer & 0x40 != 0 {
        return DecompressModeDispatchLoc139A6::LongRef {
            output_copy_base_addr: output_addr,
        };
    }

    let total_copy_len = if short_ref_control_byte & 0x80 != 0 {
        3
    } else {
        2
    };
    DecompressModeDispatchLoc139A6::ShortRef {
        source_copy_addr: output_addr.wrapping_sub(u16::from(short_ref_back_distance)),
        total_copy_len,
    }
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn decompress_mode_dispatch_loc_13A24(
    bit_buffer: u8,
    output_addr: u16,
    direct_copy_byte: u8,
    control_byte: u8,
    short_ref_back_distance: u8,
) -> DecompressModeDispatchLoc13A24 {
    // ASM loc_13A24 (kid.asm:26786-26798) runs after `loc_13A22` has already
    // copied one literal byte. One carried mode bit decides whether the next
    // step is another literal byte through `loc_13A9E`; otherwise a fresh
    // `(a0)+` control byte chooses between the later long-reference family at
    // `loc_13A46` and the shared 2/3-byte short-reference path at `loc_13A3A`.
    if bit_buffer & 0x80 != 0 {
        return DecompressModeDispatchLoc13A24::DirectCopy {
            copied_byte: direct_copy_byte,
        };
    }

    if control_byte & 0x80 != 0 {
        return DecompressModeDispatchLoc13A24::LongRef {
            output_copy_base_addr: output_addr,
            control_byte,
        };
    }

    let total_copy_len = if control_byte & 0x40 != 0 { 3 } else { 2 };
    DecompressModeDispatchLoc13A24::ShortRef {
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
pub(crate) fn decompress_short_ref_finished_loc_1393C(
    copied_bytes: [u8; 2],
    output_addr_after_tail: u16,
    output_limit_addr: u16,
) -> DecompressShortRefTailLoc137C4 {
    // ASM loc_1393C (kid.asm:26620-26625) is the later two-byte
    // short-reference tail after the optional extra byte from `loc_13928`:
    // it copies exactly two more bytes from `(a6)+` into `(a2)+`, then uses
    // `cmp.w a2,d7` / `bls.s loc_13994` to decide whether the decode has
    // finished.
    DecompressShortRefTailLoc137C4 {
        copied_bytes,
        finished: output_addr_after_tail >= output_limit_addr,
    }
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn decompress_short_ref_finished_loc_139BC(
    copied_bytes: [u8; 2],
    output_addr_after_tail: u16,
    output_limit_addr: u16,
) -> DecompressShortRefTailLoc137C4 {
    // ASM loc_139BC (kid.asm:26703-26707) is the two-byte short-reference
    // tail after the optional third-byte prelude from `loc_139A6`: it copies
    // exactly two more bytes from `(a6)+` into `(a2)+`, then uses `cmp.w a2,d7`
    // / `bls.s loc_13A12` to decide whether the decode has finished.
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
pub(crate) fn decompress_long_ref_header_loc_13948(
    bit_buffer: u8,
    control_byte: u8,
    long_ref_mask: u16,
    long_ref_low_byte: u8,
) -> DecompressLongRefHeaderLoc13756 {
    // ASM loc_13948 (kid.asm:26628-26638) first shifts the carried `d1` word
    // left one bit with `add.w d1,d1`, then replaces its low byte with
    // `(a0)+`, shifts the result left two more bits, masks the distance bits
    // through `d4`, appends the next source byte, and decodes the later copy
    // family from control bits 5:4: `00` => copy 3 at `loc_13978`, `01` =>
    // copy 4 at `loc_13976`, `10` => copy 5 via `loc_13974`, `11` =>
    // variable-length via `loc_13960`.
    let shifted_word = ((((u16::from(bit_buffer)) << 1) & 0xFF00) | u16::from(control_byte)) << 2;
    let source_back_distance = (shifted_word & long_ref_mask) | u16::from(long_ref_low_byte);
    let copy_mode = match (control_byte >> 4) & 0b11 {
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
pub(crate) fn decompress_long_ref_extended_loc_139DE(
    bit_buffer: u8,
    length_byte: u8,
) -> DecompressLongRefExtendedLoc1376A {
    // ASM loc_139DE (kid.asm:26732-26739) consumes the post-distance control
    // bit for the `loc_139C8` family: carry clear branches straight to
    // `loc_139F2` for the fixed five-byte tail, while carry set loads `BYTE2`
    // and uses the same quit / flush / variable-length split before feeding
    // the inclusive prefix loop at `loc_139EC`.
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
pub(crate) fn decompress_long_ref_extended_loc_13A5A(
    bit_buffer: u8,
    length_byte: u8,
) -> DecompressLongRefExtendedLoc1376A {
    // ASM loc_13A5A (kid.asm:26820-26827) consumes the post-distance control
    // bit for the `loc_13A46` family: carry clear branches straight to
    // `loc_13A6E` for the fixed five-byte tail, while carry set loads `BYTE2`
    // and uses the same quit / flush / variable-length split before feeding
    // the inclusive prefix loop at `loc_13A68`.
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
pub(crate) fn decompress_long_ref_loop_loc_1396E(
    source_bytes: &[u8],
    dbf_counter: u16,
) -> DecompressLongRefLoopLoc13778 {
    // ASM loc_1396E (kid.asm:26652-26653) copies one byte from `(a6)+` to
    // `(a2)+` before each `dbf d0,loc_1396E` check, so the later
    // variable-length branch from `loc_13960` emits exactly `d0+1` prefix
    // bytes before the fixed tail that starts at `loc_13974`.
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
pub(crate) fn decompress_long_ref_loop_loc_139EC(
    source_bytes: &[u8],
    dbf_counter: u16,
) -> DecompressLongRefLoopLoc13778 {
    // ASM loc_139EC (kid.asm:26741-26742) copies one byte from `(a6)+` to
    // `(a2)+` before each `dbf d0,loc_139EC` check, so the variable-length
    // branch from `loc_139DE` emits exactly `d0+1` prefix bytes before the
    // fixed tail that starts at `loc_139F2`.
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
pub(crate) fn decompress_long_ref_loop_loc_13A68(
    source_bytes: &[u8],
    dbf_counter: u16,
) -> DecompressLongRefLoopLoc13778 {
    // ASM loc_13A68 (kid.asm:26829-26830) copies one byte from `(a6)+` to
    // `(a2)+` before each `dbf d0,loc_13A68` check, so the variable-length
    // branch from `loc_13A5A` emits exactly `d0+1` prefix bytes before the
    // fixed tail that starts at `loc_13A6E`.
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
pub(crate) fn decompress_long_ref_quit_loc_13986() -> DecompressBp0ReturnStatus {
    // ASM loc_13986 (kid.asm:26671-26672) is the `BYTE2 = 0` early return from
    // `loc_13960`: it clears `d0` and returns immediately without reseeding
    // `d2`, so the caller keeps the existing copy mode.
    DecompressBp0ReturnStatus {
        status_word: 0,
        next_copy_mode: None,
    }
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn decompress_long_ref_quit_loc_13A04() -> DecompressBp0ReturnStatus {
    // ASM loc_13A04 (kid.asm:26760-26761) is the `BYTE2 = 0` early return from
    // `loc_139DE`: it clears `d0` and returns immediately without writing a
    // new copy mode to `d2`, so the caller keeps its existing state.
    DecompressBp0ReturnStatus {
        status_word: 0,
        next_copy_mode: None,
    }
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn decompress_long_ref_quit_loc_13A80() -> DecompressBp0ReturnStatus {
    // ASM loc_13A80 (kid.asm:26848-26849) is the `BYTE2 = 0` early return from
    // `loc_13A5A`: it clears `d0` and returns immediately without writing a
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
pub(crate) fn decompress_long_ref_tail_loc_139F2(
    copied_bytes: [u8; 5],
    output_addr_after_tail: u16,
    output_limit_addr: u16,
) -> DecompressLongRefTailLoc1377E {
    // ASM loc_139F2 (kid.asm:26745-26753) enters the fixed tail one byte
    // earlier than `loc_139F4`: it copies one byte at `loc_139F2`, one at
    // `loc_139F4`, then three more through the `loc_139F6` fallthrough before
    // `cmp.w a2,d7` branches to `loc_13A1A` or resumes the shared decode loop
    // at `loc_13928`.
    DecompressLongRefTailLoc1377E {
        copied_bytes,
        finished: output_addr_after_tail >= output_limit_addr,
    }
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn decompress_long_ref_tail_loc_13A6E(
    copied_bytes: [u8; 5],
    output_addr_after_tail: u16,
    output_limit_addr: u16,
) -> DecompressLongRefTailLoc1377E {
    // ASM loc_13A6E (kid.asm:26833-26841) enters the fixed tail one byte
    // later than `loc_139F2`: it copies one byte at `loc_13A6E`, one at
    // `loc_13A70`, then three more through the `loc_13A72` fallthrough before
    // `cmp.w a2,d7` branches to `loc_13A96` or resumes the shared decode loop
    // at `loc_139A6`.
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
pub(crate) fn decompress_long_ref_flush_loc_1398C() -> DecompressBp0ReturnStatus {
    // ASM loc_1398C (kid.asm:26676-26678) is the `0 < BYTE2 < 6` underflow
    // return from `loc_13960`: after `subq.w #6,d0` goes negative, the
    // routine reports `d0=$FFFF` and seeds `d2=4` so the caller resumes in
    // the four-byte tail copy mode.
    DecompressBp0ReturnStatus {
        status_word: u16::MAX,
        next_copy_mode: Some(4),
    }
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn decompress_long_ref_flush_loc_13A0A() -> DecompressBp0ReturnStatus {
    // ASM loc_13A0A (kid.asm:26765-26767) is the `0 < BYTE2 < 6` underflow
    // return from `loc_139DE`: after `subq.w #6,d0` goes negative, the
    // routine reports `d0=$FFFF` and seeds `d2=3` so the caller resumes in
    // the three-byte tail copy mode.
    DecompressBp0ReturnStatus {
        status_word: u16::MAX,
        next_copy_mode: Some(3),
    }
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn decompress_long_ref_flush_loc_13A86() -> DecompressBp0ReturnStatus {
    // ASM loc_13A86 (kid.asm:26853-26855) is the `0 < BYTE2 < 6` underflow
    // return from `loc_13A5A`: after `subq.w #6,d0` goes negative, the
    // routine reports `d0=$FFFF` and seeds `d2=2` so the caller resumes in
    // the two-byte tail copy mode.
    DecompressBp0ReturnStatus {
        status_word: u16::MAX,
        next_copy_mode: Some(2),
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
pub(crate) fn decompress_long_ref_finished_loc_1399C() -> DecompressBp0ReturnStatus {
    // ASM loc_1399C (kid.asm:26684-26686) is the successful return after the
    // later fixed three-byte tail at `loc_13978`: once `cmp.w a2,d7` / `bls.s
    // loc_1399C` reports the output cursor at or beyond the limit, the routine
    // returns `d0=1` and seeds `d2=4`.
    DecompressBp0ReturnStatus {
        status_word: 1,
        next_copy_mode: Some(4),
    }
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn decompress_long_ref_finished_loc_13A8E() -> DecompressBp0ReturnStatus {
    // ASM loc_13A8E (kid.asm:26859-26861) is the successful return after the
    // fixed five-byte tail at `loc_13A6E`: once `cmp.w a2,d7` / `bls.s
    // loc_13A96` reports the output cursor at or beyond the limit, the routine
    // returns `d0=1` and seeds `d2=6`.
    DecompressBp0ReturnStatus {
        status_word: 1,
        next_copy_mode: Some(6),
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
pub(crate) struct KidHitboxSolidScanSetupLoc1FB2 {
    pub(crate) row_start_addr: u16,
    pub(crate) row_count: u16,
    pub(crate) col_count: u16,
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn kid_hitbox_solid_scan_setup_loc_1FB2(
    scan_disabled_fa64: bool,
    kid_object_present: bool,
    kid_hitbox_top: u16,
    kid_hitbox_bottom: u16,
    kid_hitbox_left: u16,
    kid_hitbox_right: u16,
    row_pointer_table: &[u16],
) -> Option<KidHitboxSolidScanSetupLoc1FB2> {
    // ASM loc_1FB2 (kid.asm:3790-3809): bail out when `$FFFFFA64` is non-zero
    // or the Kid object pointer is null, otherwise convert the hitbox bounds
    // from pixels to tile rows/columns, seed inclusive DBF counters, and load
    // the top-row/left-column Level_Layout address from `$FFFF4A04`.
    if scan_disabled_fa64 || !kid_object_present {
        return None;
    }

    let top_row = kid_hitbox_top >> 4;
    let bottom_row = kid_hitbox_bottom >> 4;
    let left_col = kid_hitbox_left >> 4;
    let right_col = kid_hitbox_right >> 4;
    let row_start_addr = *row_pointer_table.get(usize::from(top_row))?;

    Some(KidHitboxSolidScanSetupLoc1FB2 {
        row_start_addr: row_start_addr.wrapping_add(left_col.wrapping_mul(2)),
        row_count: bottom_row.wrapping_sub(top_row),
        col_count: right_col.wrapping_sub(left_col),
    })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct KidHitboxSolidScanHitLoc1FEE {
    pub(crate) tile_word: u16,
    pub(crate) row_index: u16,
    pub(crate) col_index: u16,
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn kid_hitbox_solid_scan_loc_1FEE(
    level_layout_words: &[u16],
    start_index: usize,
    level_width_tiles: usize,
    row_count: u16,
    col_count: u16,
) -> Option<KidHitboxSolidScanHitLoc1FEE> {
    // ASM loc_1FEE / loc_1FF2 (kid.asm:3811-3822): scan the hitbox window in
    // row-major order, mask each layout word with `$7000`, branch out on the
    // first collision nibble `>= $6000`, otherwise advance to the next row with
    // `add.w (Level_width_tiles).w,a0` and continue the inclusive DBF loops.
    for row_index in 0..=row_count {
        let row_start = start_index + usize::from(row_index) * level_width_tiles;
        for col_index in 0..=col_count {
            let tile_word = *level_layout_words.get(row_start + usize::from(col_index))?;
            if (tile_word & 0x7000) >= 0x6000 {
                return Some(KidHitboxSolidScanHitLoc1FEE {
                    tile_word,
                    row_index,
                    col_index,
                });
            }
        }
    }

    None
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn palette_to_vram0_blocks_solid_scan_loc_2198(tile_word: u16) -> u16 {
    // ASM j_Palette_to_VRAM0 / loc_2198 (kid.asm:3978-3989): return zero only
    // when bit 15 was set before `bclr #$F,d0` and the remaining palette nibble
    // matches `$300`; every other tile classification falls through to
    // `loc_2198` and returns one.
    let bit_15_was_clear = (tile_word & 0x8000) == 0;
    if bit_15_was_clear {
        return 1;
    }

    if (tile_word & 0x0F00) != 0x0300 {
        return 1;
    }

    0
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct KidHitboxSolidScanRightSeedLoc201C {
    pub(crate) rewind_index: usize,
    pub(crate) tile_column: u16,
    pub(crate) left_px: u16,
    pub(crate) right_px: u16,
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn kid_hitbox_solid_scan_expand_right_loc_201C(
    solid_scan_locked_faa6: bool,
    start_index_after_initial_hit: usize,
    tile_column_after_rewind: u16,
    palette_results: &[u16],
) -> Option<KidHitboxSolidScanRightSeedLoc201C> {
    // ASM loc_201C (kid.asm:3834-3847): abort to `loc_2156` when `$FFFFFAA6`
    // is already set; otherwise keep consuming `(a1)+` until
    // `j_Palette_to_VRAM0` returns non-zero. The post-hit `subq.w #4,a1`
    // rewinds two words so `sub_1F8E` sees the last zero tile before the
    // blocking tile, then `d1/d2` seed that tile's inclusive 16-pixel span.
    if solid_scan_locked_faa6 {
        return None;
    }

    let blocking_offset = palette_results.iter().position(|&result| result != 0)?;
    let blocking_successor = start_index_after_initial_hit.checked_add(blocking_offset + 1)?;
    let rewind_index = blocking_successor.checked_sub(2)?;
    let left_px = tile_column_after_rewind << 4;
    Some(KidHitboxSolidScanRightSeedLoc201C {
        rewind_index,
        tile_column: tile_column_after_rewind,
        left_px,
        right_px: left_px.wrapping_add(0x000F),
    })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct KidHitboxSolidScanLeftSeedLoc2040 {
    pub(crate) blocking_index: usize,
    pub(crate) left_px: u16,
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn kid_hitbox_solid_scan_expand_left_loc_2040(
    rewind_index: usize,
    initial_left_px: u16,
    palette_results: &[u16],
) -> Option<KidHitboxSolidScanLeftSeedLoc2040> {
    // ASM loc_2040 (kid.asm:3849-3853): pre-decrement `a1` with `-(a1)` so the
    // first probe reads the tile immediately left of the rewind point, stop on
    // the first non-zero `j_Palette_to_VRAM0` result, and only subtract `$10`
    // from `d1` while the scanned tile still reports zero.
    let blocking_offset = palette_results.iter().position(|&result| result != 0)?;
    let blocking_index = rewind_index.checked_sub(blocking_offset + 1)?;
    let zero_tiles = u16::try_from(blocking_offset).ok()?;
    Some(KidHitboxSolidScanLeftSeedLoc2040 {
        blocking_index,
        left_px: initial_left_px.wrapping_sub(zero_tiles.wrapping_mul(0x0010)),
    })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct KidHitboxSolidScanUpSeedLoc2050 {
    pub(crate) restored_index: usize,
    pub(crate) row_index: u16,
    pub(crate) vertical_start_px: u16,
    pub(crate) vertical_end_px: u16,
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn kid_hitbox_solid_scan_expand_up_loc_2050(
    start_index_after_left_blocker: usize,
    level_width_tiles: usize,
    palette_results: &[u16],
) -> Option<KidHitboxSolidScanUpSeedLoc2050> {
    // ASM loc_2050 / loc_2056 (kid.asm:3857-3868): advance one word with
    // `addq.w #2,a1`, then probe one row up per `suba.w d3,a1` until
    // `j_Palette_to_VRAM0` returns non-zero. The post-hit `add.w d3,a1`
    // restores the last zero row, and `sub_1F8E` seeds the low words of
    // `d1/d2` from that restored row with `d1 = row << 4` and `d2 = d1 - 1`.
    let blocking_offset = palette_results.iter().position(|&result| result != 0)?;
    let restored_index = start_index_after_left_blocker
        .checked_sub(blocking_offset.checked_mul(level_width_tiles)?)?;
    let row_index = u16::try_from(restored_index / level_width_tiles).ok()?;
    let vertical_start_px = row_index << 4;
    Some(KidHitboxSolidScanUpSeedLoc2050 {
        restored_index,
        row_index,
        vertical_start_px,
        vertical_end_px: vertical_start_px.wrapping_sub(1),
    })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct KidHitboxSolidScanResolveUpLoc20C6 {
    pub(crate) kid_hitbox_top: u16,
    pub(crate) kid_hitbox_bottom: u16,
    pub(crate) object_top: u16,
    pub(crate) object_y_velocity: u16,
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn kid_hitbox_solid_scan_resolve_up_loc_20C6(
    upward_overlap_px: u16,
    kid_hitbox_top: u16,
    kid_hitbox_bottom: u16,
    object_top: u16,
    _object_y_velocity: u16,
) -> KidHitboxSolidScanResolveUpLoc20C6 {
    // ASM loc_20C6 (kid.asm:3907-3913): add one pixel to the chosen upward
    // overlap, subtract it from Kid's top/bottom hitbox words and from the
    // colliding object's Y position at `$1E(a0)`, then clear `$20(a0)`.
    let displacement = upward_overlap_px.wrapping_add(1);
    KidHitboxSolidScanResolveUpLoc20C6 {
        kid_hitbox_top: kid_hitbox_top.wrapping_sub(displacement),
        kid_hitbox_bottom: kid_hitbox_bottom.wrapping_sub(displacement),
        object_top: object_top.wrapping_sub(displacement),
        object_y_velocity: 0,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct KidHitboxSolidScanResolveRightLoc20DC {
    pub(crate) kid_hitbox_left: u16,
    pub(crate) kid_hitbox_right: u16,
    pub(crate) object_left: u16,
    pub(crate) object_x_velocity: u16,
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn kid_hitbox_solid_scan_resolve_right_loc_20DC(
    rightward_overlap_px: u16,
    kid_hitbox_left: u16,
    kid_hitbox_right: u16,
    object_left: u16,
    _object_x_velocity: u16,
) -> KidHitboxSolidScanResolveRightLoc20DC {
    // ASM loc_20DC (kid.asm:3916-3922): add one pixel to the chosen rightward
    // overlap, add it to Kid's left/right hitbox words and to the colliding
    // object's X position at `$1A(a0)`, then clear `$1C(a0)`.
    let displacement = rightward_overlap_px.wrapping_add(1);
    KidHitboxSolidScanResolveRightLoc20DC {
        kid_hitbox_left: kid_hitbox_left.wrapping_add(displacement),
        kid_hitbox_right: kid_hitbox_right.wrapping_add(displacement),
        object_left: object_left.wrapping_add(displacement),
        object_x_velocity: 0,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct KidHitboxSolidScanResolveLeftLoc2106 {
    pub(crate) kid_hitbox_left: u16,
    pub(crate) kid_hitbox_right: u16,
    pub(crate) object_left: u16,
    pub(crate) object_x_velocity: u16,
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn kid_hitbox_solid_scan_resolve_left_loc_2106(
    leftward_overlap_px: u16,
    kid_hitbox_left: u16,
    kid_hitbox_right: u16,
    object_left: u16,
    _object_x_velocity: u16,
) -> KidHitboxSolidScanResolveLeftLoc2106 {
    // ASM loc_2106 (kid.asm:3931-3937): add one pixel to the chosen leftward
    // overlap, subtract it from Kid's left/right hitbox words and from the
    // colliding object's X position at `$1A(a0)`, then clear `$1C(a0)`.
    let displacement = leftward_overlap_px.wrapping_add(1);
    KidHitboxSolidScanResolveLeftLoc2106 {
        kid_hitbox_left: kid_hitbox_left.wrapping_sub(displacement),
        kid_hitbox_right: kid_hitbox_right.wrapping_sub(displacement),
        object_left: object_left.wrapping_sub(displacement),
        object_x_velocity: 0,
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

#[cfg_attr(not(test), allow(dead_code))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct TitleCardVramWordFillLoop {
    pub(crate) vram_word_addr: u16,
    pub(crate) fill_word: u16,
    pub(crate) word_write_count: u16,
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn load_title_card_clear_vram_loc_19ABE() -> TitleCardVramWordFillLoop {
    // ASM loc_19ABE (kid.asm:28595-28598): after `move.l
    // #vdpComm($1400,VRAM,WRITE),4(a6)` and `move.w #$DF,d1`, the loop writes
    // `d0` to the VDP data port twice per pass before the inclusive
    // `dbf d1,loc_19ABE` test. With `d0=0`, this clears 448 words starting at
    // VRAM word address `$1400`.
    TitleCardVramWordFillLoop {
        vram_word_addr: 0x1400,
        fill_word: 0,
        word_write_count: (0x00DFu16 + 1) * 2,
    }
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn load_title_card_tilemap_clear_loc_19B58() -> TitleCardVramWordFillLoop {
    // ASM loc_19B58 (kid.asm:28636-28637): after `move.l
    // #vdpComm($E000,VRAM,WRITE),4(a6)`, `move.w #$7FF,d0`, and
    // `move.w #$94,d1`, the loop writes `d1` to the VDP data port once per
    // pass before the inclusive `dbf d0,loc_19B58` test. That fills 2048
    // words at VRAM word address `$E000` with tile word `$0094`.
    TitleCardVramWordFillLoop {
        vram_word_addr: 0xE000,
        fill_word: 0x0094,
        word_write_count: 0x07FF + 1,
    }
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn load_title_card_name_table_clear_loc_19B6A() -> TitleCardVramWordFillLoop {
    // ASM loc_19B6A (kid.asm:28643-28644): after `move.l
    // #vdpComm($1280,VRAM,WRITE),4(a6)`, `moveq #$F,d0`, and `moveq #0,d1`,
    // the loop writes `d1` to the VDP data port once per pass before the
    // inclusive `dbf d0,loc_19B6A` test. That clears 16 words at VRAM word
    // address `$1280`.
    TitleCardVramWordFillLoop {
        vram_word_addr: 0x1280,
        fill_word: 0,
        word_write_count: 0x000F + 1,
    }
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn load_title_card_palette_copy_word_count_loc_19B80() -> u16 {
    // ASM loc_19B80 (kid.asm:28651-28652): after loading the title-card
    // palette pointer into `a0`, `Palette_Buffer` into `a1`, and seeding
    // `d0=#$F`, the loop copies one palette word per pass before the
    // inclusive `dbf d0,loc_19B80` test. That always copies 16 words.
    0x000F + 1
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn level_entry_title_letter_palette_copy_word_count_loc_1AB40() -> u16 {
    // ASM loc_1AB40 (kid.asm:28758-28760): after `lea Pal_19C48(pc),a0`,
    // `lea (Palette_Buffer+$20).l,a1`, and `moveq #$F,d0`, the loop copies one
    // word per pass before the inclusive `dbf d0,loc_1AB40` test. That always
    // copies 16 words into the level-entry title-letter palette window.
    0x000F + 1
}

#[cfg_attr(not(test), allow(dead_code))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct TitleCardTopPaddingLoc19BC2 {
    pub(crate) fill_word: u16,
    pub(crate) words_per_row: u16,
    pub(crate) row_count: u16,
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn load_title_card_top_padding_loc_19BC2(
    title_card_height_tiles: u16,
) -> Option<TitleCardTopPaddingLoc19BC2> {
    // ASM loc_19BC2 (kid.asm:28669-28674): after `moveq #$1C,d6`,
    // `sub.w d1,d6`, and `lsr.w #1,d6`, the routine seeds `d2=d6-1` and
    // exits early on `bmi.s loc_19BCE`. Otherwise each outer pass reloads
    // `d4=#$3F` and writes `#$2FB` once per inclusive `dbf d4,loc_19BC4`,
    // so the top-padding stage emits 64 fill words per row for
    // `(0x1C - height_tiles) / 2` rows.
    let row_count = 0x001Cu16.checked_sub(title_card_height_tiles)? >> 1;
    if row_count == 0 {
        None
    } else {
        Some(TitleCardTopPaddingLoc19BC2 {
            fill_word: 0x02FB,
            words_per_row: 0x003F + 1,
            row_count,
        })
    }
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn load_title_card_left_padding_loc_19BD4(
    horizontal_padding_words: u16,
) -> Option<TitleCardVramWordFillLoop> {
    // ASM loc_19BD4 (kid.asm:28688-28689): `d4` arrives as `d7-1`, so the
    // `move.w d5,(a6)` store happens once per pass before the inclusive
    // `dbf d4,loc_19BD4` test. The loop therefore emits exactly `d7`
    // left-padding words, each still filled with `#$2FB`.
    if horizontal_padding_words == 0 {
        None
    } else {
        Some(TitleCardVramWordFillLoop {
            vram_word_addr: 0xE000,
            fill_word: 0x02FB,
            word_write_count: horizontal_padding_words,
        })
    }
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn load_title_card_row_copy_seed_loc_19BDA(title_card_width_tiles: u16) -> Option<u16> {
    // ASM loc_19BDA (kid.asm:28692-28693): `move.w d0,d5` followed by
    // `subq.w #1,d5` primes the following `dbf d5,loc_19BDE` loop with an
    // inclusive counter, so width zero skips the copy stage and every other
    // width becomes `width_tiles - 1`.
    title_card_width_tiles.checked_sub(1)
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn load_title_card_row_copy_word_count_loc_19BDE(row_copy_seed: u16) -> u16 {
    // ASM loc_19BDE (kid.asm:28696-28697): the `move.w (a0)+,(a6)` transfer
    // happens before `dbf d5,loc_19BDE` tests for `-1`, so a seed of
    // `width_tiles - 1` copies exactly `width_tiles` art words into the row.
    row_copy_seed + 1
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn load_title_card_right_padding_loc_19BEE(
    horizontal_padding_words: u16,
    title_card_width_tiles: u16,
) -> Option<TitleCardVramWordFillLoop> {
    // ASM loc_19BEE (kid.asm:28698-28704): after `loc_19BDE` copies one art
    // row, the routine seeds `d5 = $40 - d7 - d0 - 1` and writes `#$2FB`
    // once per inclusive `dbf d5,loc_19BEE`. Because `d7` is the already-used
    // left-padding width and `d0` is the copied art width, the loop emits the
    // remaining `64 - left_padding_words - title_card_width_tiles` fill words
    // needed to complete the row. When that remainder is zero the seed goes
    // negative immediately and the loop is skipped.
    let used_words = horizontal_padding_words.checked_add(title_card_width_tiles)?;
    let right_padding_words = 0x0040u16.checked_sub(used_words)?;
    if right_padding_words == 0 {
        None
    } else {
        Some(TitleCardVramWordFillLoop {
            vram_word_addr: 0xE000,
            fill_word: 0x02FB,
            word_write_count: right_padding_words,
        })
    }
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn load_title_card_bottom_padding_row_width_loc_19C02() -> u16 {
    // ASM loc_19C02 (kid.asm:28713): each bottom-padding row reloads
    // `d4=#$3F` immediately before `loc_19C04`, so the following inclusive
    // `dbf d4,loc_19C04` loop writes 64 fill words across the full row.
    0x003F + 1
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn load_title_card_bottom_padding_loc_19C04(
    title_card_height_tiles: u16,
) -> Option<TitleCardTopPaddingLoc19BC2> {
    // ASM loc_19C04 (kid.asm:28716-28718): after the art rows finish, the
    // routine seeds `d5 = $1C - d1 - d6 - 1`, reloads `d4=#$3F` via
    // `loc_19C02`, and writes `#$2FB` once per inclusive `dbf d4,loc_19C04`
    // before the outer `dbf d5,loc_19C02`. Because `d6` is the top-padding
    // row count from `lsr.w #1`, this emits the remaining
    // `0x1C - height_tiles - top_padding_rows` full 64-word rows at the
    // bottom. A full-height title card leaves the outer seed negative
    // immediately and skips the loop.
    let top_padding_rows = 0x001Cu16.checked_sub(title_card_height_tiles)? >> 1;
    let used_rows = title_card_height_tiles.checked_add(top_padding_rows)?;
    let bottom_padding_rows = 0x001Cu16.checked_sub(used_rows)?;
    if bottom_padding_rows == 0 {
        None
    } else {
        Some(TitleCardTopPaddingLoc19BC2 {
            fill_word: 0x02FB,
            words_per_row: load_title_card_bottom_padding_row_width_loc_19C02(),
            row_count: bottom_padding_rows,
        })
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
pub(crate) fn adjust_8_byte_block_list_entries_for_blank_top_rows_loc_141EE(
    entries: &[[u16; 4]],
    blank_top_rows: u16,
    level_width_tiles: u16,
) -> Vec<[u16; 4]> {
    // ASM loc_141EE (kid.asm:27037-27039) walks the 8-byte block-list entries
    // rooted at `($FFFFF8D4).w`, adding `Number_blank_top_rows * Level_width_tiles`
    // to word 0 and `Number_blank_top_rows` to word 2 before `addq.w #8,a0`
    // advances to the next entry. The other two words stay unchanged.
    let layout_word_delta = blank_top_rows.wrapping_mul(level_width_tiles);
    entries
        .iter()
        .map(|entry| {
            let mut adjusted = *entry;
            adjusted[0] = adjusted[0].wrapping_add(layout_word_delta);
            adjusted[2] = adjusted[2].wrapping_add(blank_top_rows);
            adjusted
        })
        .collect()
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn adjust_8_byte_block_list_entries_for_blank_top_rows_loc_141F6(
    entries: &[[u16; 4]],
    entry_count: u16,
    blank_top_rows: u16,
    level_width_tiles: u16,
) -> Vec<[u16; 4]> {
    // ASM loc_141F6 (kid.asm:27042-27043) branches into the `dbf d0,loc_141EE`
    // loop tail after loading `($FFFFF8D2).w` into `d0`, so the body at
    // `loc_141EE` runs exactly `entry_count` times. `d0 = 0` skips the loop
    // entirely; larger counts adjust only the first `d0` 8-byte entries.
    let mut adjusted = entries.to_vec();
    let adjusted_prefix = adjust_8_byte_block_list_entries_for_blank_top_rows_loc_141EE(
        &adjusted[..usize::from(entry_count).min(adjusted.len())],
        blank_top_rows,
        level_width_tiles,
    );
    adjusted[..adjusted_prefix.len()].copy_from_slice(&adjusted_prefix);
    adjusted
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn adjust_16_byte_hidden_block_entries_for_blank_top_rows_loc_14204(
    entries: &[[u16; 8]],
    blank_top_rows: u16,
    level_width_tiles: u16,
) -> Vec<[u16; 8]> {
    // ASM loc_14204 (kid.asm:27049-27051) walks each 16-byte hidden-block
    // entry rooted at `($FFFFF8DA).w`, adding `Number_blank_top_rows *
    // Level_width_tiles` to word offset `$8(a0)` (the stored layout word) and
    // `Number_blank_top_rows` to word offset `$C(a0)` (the stored row word)
    // before `lea $10(a0),a0` advances to the next entry.
    let layout_word_delta = blank_top_rows.wrapping_mul(level_width_tiles);
    entries
        .iter()
        .map(|entry| {
            let mut adjusted = *entry;
            adjusted[4] = adjusted[4].wrapping_add(layout_word_delta);
            adjusted[6] = adjusted[6].wrapping_add(blank_top_rows);
            adjusted
        })
        .collect()
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn adjust_16_byte_hidden_block_entries_for_blank_top_rows_loc_14210(
    entries: &[[u16; 8]],
    entry_count: u16,
    blank_top_rows: u16,
    level_width_tiles: u16,
) -> Vec<[u16; 8]> {
    // ASM loc_14210 (kid.asm:27054-27055) branches into `dbf d0,loc_14204`
    // after loading `($FFFFF8D8).w` into `d0`, so the body at `loc_14204`
    // runs exactly `entry_count` times. `d0 = 0` skips the 16-byte list
    // entirely; larger counts adjust only the first `d0` hidden-block entries.
    let mut adjusted = entries.to_vec();
    let adjusted_prefix = adjust_16_byte_hidden_block_entries_for_blank_top_rows_loc_14204(
        &adjusted[..usize::from(entry_count).min(adjusted.len())],
        blank_top_rows,
        level_width_tiles,
    );
    adjusted[..adjusted_prefix.len()].copy_from_slice(&adjusted_prefix);
    adjusted
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn adjust_10_byte_custom_write_entries_for_blank_top_rows_loc_1421E(
    entries: &[[u16; 5]],
    blank_top_rows: u16,
    level_width_tiles: u16,
) -> Vec<[u16; 5]> {
    // ASM loc_1421E (kid.asm:27061-27063) walks the 10-byte custom-write
    // entries rooted at `($FFFFF8E0).w`, adding `Number_blank_top_rows *
    // Level_width_tiles` to word 0 (the stored layout word) and
    // `Number_blank_top_rows` to word 2 before `lea $A(a0),a0` advances to the
    // next entry.
    let layout_word_delta = blank_top_rows.wrapping_mul(level_width_tiles);
    entries
        .iter()
        .map(|entry| {
            let mut adjusted = *entry;
            adjusted[0] = adjusted[0].wrapping_add(layout_word_delta);
            adjusted[2] = adjusted[2].wrapping_add(blank_top_rows);
            adjusted
        })
        .collect()
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn adjust_10_byte_custom_write_entries_for_blank_top_rows_loc_14228(
    entries: &[[u16; 5]],
    entry_count: u16,
    blank_top_rows: u16,
    level_width_tiles: u16,
) -> Vec<[u16; 5]> {
    // ASM loc_14228 (kid.asm:27066-27067) branches into `dbf d0,loc_1421E`
    // after loading `($FFFFF8DE).w` into `d0`, so the body at `loc_1421E`
    // runs exactly `entry_count` times. `d0 = 0` skips the 10-byte list
    // entirely; larger counts adjust only the first `d0` custom-write entries.
    let mut adjusted = entries.to_vec();
    let adjusted_prefix = adjust_10_byte_custom_write_entries_for_blank_top_rows_loc_1421E(
        &adjusted[..usize::from(entry_count).min(adjusted.len())],
        blank_top_rows,
        level_width_tiles,
    );
    adjusted[..adjusted_prefix.len()].copy_from_slice(&adjusted_prefix);
    adjusted
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn adjust_8_byte_hidden_block_entries_for_blank_top_rows_loc_14236(
    entries: &[[u16; 4]],
    blank_top_rows: u16,
    level_width_tiles: u16,
) -> Vec<[u16; 4]> {
    // ASM loc_14236 (kid.asm:27073-27075) walks the 8-byte hidden/background
    // entries rooted at `($FFFFF8EC).w`, adding `Number_blank_top_rows *
    // Level_width_tiles` to word 0 (the stored layout word) and
    // `Number_blank_top_rows` to word 2 before `addq.w #8,a0` advances to the
    // next entry.
    let layout_word_delta = blank_top_rows.wrapping_mul(level_width_tiles);
    entries
        .iter()
        .map(|entry| {
            let mut adjusted = *entry;
            adjusted[0] = adjusted[0].wrapping_add(layout_word_delta);
            adjusted[2] = adjusted[2].wrapping_add(blank_top_rows);
            adjusted
        })
        .collect()
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn adjust_8_byte_hidden_block_entries_for_blank_top_rows_loc_1423E(
    entries: &[[u16; 4]],
    entry_count: u16,
    blank_top_rows: u16,
    level_width_tiles: u16,
) -> Vec<[u16; 4]> {
    // ASM loc_1423E (kid.asm:27078) branches into `dbf d0,loc_14236` after
    // loading `($FFFFF8EA).w` into `d0`, so the body at `loc_14236` runs
    // exactly `entry_count` times. `d0 = 0` skips the 8-byte list entirely;
    // larger counts adjust only the first `d0` hidden/background entries.
    let mut adjusted = entries.to_vec();
    let adjusted_prefix = adjust_8_byte_hidden_block_entries_for_blank_top_rows_loc_14236(
        &adjusted[..usize::from(entry_count).min(adjusted.len())],
        blank_top_rows,
        level_width_tiles,
    );
    adjusted[..adjusted_prefix.len()].copy_from_slice(&adjusted_prefix);
    adjusted
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
        theme_mapping_table_offset_bytes.trailing_zeros() >= 2,
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
pub(crate) fn enemy_layout_reset_and_seek_loc_31F8E(
    input: EnemyLayoutResetAndSeekInputLoc31F8E,
    header_thresholds: &[u16],
    layout_group_counts: &[u16],
) -> Option<EnemyLayoutResetAndSeekLoc31F8E> {
    // ASM loc_31F8E (kid.asm:37368): reset the enemy-layout cursor state
    // before seeking the active header bucket.
    // ASM loc_31F9A (kid.asm:37374): `move.w d1,(a0)+` / `dbf d0,loc_31F9A`
    // clears exactly 30 words of `EnemyStatus_Table`, then seeds
    // `$FFFFF940 = $FF` and `$FFFFF942 = 0`.
    // ASM loc_31FCE (kid.asm:37388): after loading the axis-selector word,
    // the header cursor always skips forward 8 bytes to the first threshold
    // entry.
    // ASM loc_31FD4 (kid.asm:37391): while `d1 >= (a0)`, consume one layout
    // count word per skipped header, advance `a1` by `count << 3`, and stop
    // at the first threshold strictly above the active camera word. The
    // stored header pointer therefore lands on the matched record's
    // pre-threshold word.
    // ASM loc_31FE6 (kid.asm:37401): after the skip loop stops, the ROM
    // stores the matched threshold into `EnemyHeader7D`, rewinds `a0` by
    // eight bytes to recover the previous threshold into `$FFFFF93C`, and
    // reanchors `Addr_EnemyLayoutHeader` on that matched record before
    // loading the batch count from `(a1)+`.
    let (active_camera_word, uses_vertical_axis) = if input.axis_selector_word == 0 {
        (input.camera_x_pos, false)
    } else {
        (input.camera_y_pos, true)
    };

    let mut header_index = 0usize;
    let mut skipped_layout_bytes = 0u32;
    while active_camera_word >= (*header_thresholds.get(header_index)? as i16) {
        let group_count = u32::from(*layout_group_counts.get(header_index)?);
        skipped_layout_bytes = skipped_layout_bytes.wrapping_add(group_count << 3);
        header_index = header_index.wrapping_add(1);
    }

    let current_header_threshold = *header_thresholds.get(header_index)?;
    let previous_header_threshold = if header_index == 0 {
        input.previous_header_threshold
    } else {
        *header_thresholds.get(header_index.wrapping_sub(1))?
    };
    let matched_header_addr = input
        .enemy_layout_header_addr
        .wrapping_add(2)
        .wrapping_add((header_index as u32).wrapping_mul(8));

    Some(EnemyLayoutResetAndSeekLoc31F8E {
        enemy_status_words: vec![0; 0x001E],
        loaded_enemy_slot_sentinel: 0xFF,
        enemy_layout_progress: 0,
        uses_vertical_axis,
        active_camera_word,
        current_header_threshold,
        previous_header_threshold,
        enemy_layout_header_addr: matched_header_addr,
        enemy_layout_addr: input.enemy_layout_addr.wrapping_add(skipped_layout_bytes),
    })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct ManageEnemyLoadingSetupLoc3682C {
    pub(crate) enemy_count: u16,
    pub(crate) loop_counter: u16,
    pub(crate) first_entry_addr: u32,
    pub(crate) enemy_status_index: usize,
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn manage_enemy_loading_setup_loc_3682C(
    enemy_layout_addr: u32,
    enemy_count_word: u16,
) -> ManageEnemyLoadingSetupLoc3682C {
    // ASM Manage_EnemyLoading -> loc_3682C (kid.asm:37969-37974) reloads `a1`
    // from `Addr_EnemyLayout`, reads the leading word through `(a1)+`, masks
    // the loop counter down to its low byte with `andi.w #$FF,d7`, mirrors
    // that value into `d6`, and reanchors `a2` on `EnemyStatus_Table`.
    let enemy_count = enemy_count_word & 0x00FF;
    ManageEnemyLoadingSetupLoc3682C {
        enemy_count,
        loop_counter: enemy_count,
        first_entry_addr: enemy_layout_addr.wrapping_add(2),
        enemy_status_index: 0,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ManageEnemyLoadingStatusBranchLoc36840 {
    SpawnFresh,
    SkipAlreadyLoaded,
    CountDownActiveLoc3685E,
    CountDownReloadLoc3686E,
    ForceReloadLoc368A6,
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn manage_enemy_loading_status_branch_loc_36840(
    enemy_status_word: u16,
) -> ManageEnemyLoadingStatusBranchLoc36840 {
    // ASM loc_36840 (kid.asm:37978-37989) clears `d4`, reads `(a2)` into `d0`,
    // and branches by status word: `0` falls into `loc_368A8`, `$FFFF`
    // skips straight to `loc_36962`, bit 13 routes to `loc_3685E`, bit 14
    // clear forces `loc_368A6`, and the remaining bit-14-set words continue
    // into `loc_3686E`.
    if enemy_status_word == 0 {
        return ManageEnemyLoadingStatusBranchLoc36840::SpawnFresh;
    }
    if enemy_status_word == 0xFFFF {
        return ManageEnemyLoadingStatusBranchLoc36840::SkipAlreadyLoaded;
    }
    if enemy_status_word & 0x2000 != 0 {
        return ManageEnemyLoadingStatusBranchLoc36840::CountDownActiveLoc3685E;
    }
    if enemy_status_word & 0x4000 == 0 {
        return ManageEnemyLoadingStatusBranchLoc36840::ForceReloadLoc368A6;
    }
    ManageEnemyLoadingStatusBranchLoc36840::CountDownReloadLoc3686E
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct ManageEnemyLoadingCountDownActiveLoc3685E {
    pub(crate) reaches_spawn_window: bool,
    pub(crate) decremented_status_word: Option<u16>,
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn manage_enemy_loading_count_down_active_loc_3685E(
    enemy_status_word: u16,
) -> ManageEnemyLoadingCountDownActiveLoc3685E {
    // ASM loc_3685E (kid.asm:37991-37996) masks `d0` with `$03FF`; a zero
    // low-ten-bit countdown falls straight through to `loc_368A8`, otherwise
    // the ROM decrements `(a2)` in place and skips spawning via `loc_36962`.
    if enemy_status_word.trailing_zeros() >= 10 {
        return ManageEnemyLoadingCountDownActiveLoc3685E {
            reaches_spawn_window: true,
            decremented_status_word: None,
        };
    }
    ManageEnemyLoadingCountDownActiveLoc3685E {
        reaches_spawn_window: false,
        decremented_status_word: Some(enemy_status_word.wrapping_sub(1)),
    }
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn manage_enemy_loading_status_word_loc_36900(
    enemy_status_word: u16,
    d4_state: u8,
) -> u16 {
    // ASM loc_36900 (kid.asm:38051-38054) only mutates `(a2)` for the `d4 == 1`
    // path, bumping the enemy-status word by `$0400` before falling into
    // `loc_3690A`. All other `d4` states leave the status word untouched.
    if d4_state == 1 {
        enemy_status_word.wrapping_add(0x0400)
    } else {
        enemy_status_word
    }
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn manage_enemy_loading_allocate_slot_hint_loc_3690A(enemy_id: u8) -> u16 {
    // ASM loc_3690A (kid.asm:38056-38063) seeds `a0` with `1` before sign-
    // extending the enemy id byte in `d0`; only enemy `$17` overwrites that
    // allocation hint with `$FFFF` before entering `loc_3691E`.
    if enemy_id == 0x17 { 0xFFFF } else { 0x0001 }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct ManageEnemyLoadingSpawnObjectLoc3691E {
    pub(crate) enemy_load_index_offset: u16,
    pub(crate) priority_flag: bool,
    pub(crate) layout_entry_addr: u32,
    pub(crate) object_status_slot: u8,
    pub(crate) enemy_status_word: Option<u16>,
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn manage_enemy_loading_spawn_object_loc_3691E(
    enemy_id: u8,
    layout_entry_addr: u32,
    enemy_count: u16,
    loop_counter: u16,
    d4_state: u8,
) -> ManageEnemyLoadingSpawnObjectLoc3691E {
    // ASM loc_3691E (kid.asm:38064-38083) allocates the object slot, looks up
    // the worker pointer via `EnemyLoad_Index + enemy_id * 8`, marks `$10(a0)`
    // high priority with `st`, stores the source layout entry in `$44(a0)`,
    // then packs `(d6 - d7 - 1)` into `$42(a0)`.
    // ASM loc_36956 (kid.asm:38084): the `d4 == 1` path ORs `$C0` into `$42(a0)`
    // and branches straight to `loc_36962`, so it never reaches the later
    // loaded-sentinel write.
    // ASM loc_3695E (kid.asm:38089): the `d4 < 1` fallthrough and `d4 > 1`
    // forced-reload path both land here, writing `$FFFF` into `(a2)` after
    // leaving `$42(a0)` raw or ORed with `$80` respectively.
    let base_status_slot = enemy_count.wrapping_sub(loop_counter).wrapping_sub(1) as u8;
    let object_status_slot = match d4_state {
        1 => base_status_slot | 0xC0,
        2..=u8::MAX => base_status_slot | 0x80,
        _ => base_status_slot,
    };
    let enemy_status_word = if d4_state == 1 { None } else { Some(0xFFFF) };

    ManageEnemyLoadingSpawnObjectLoc3691E {
        enemy_load_index_offset: u16::from(enemy_id) << 3,
        priority_flag: true,
        layout_entry_addr,
        object_status_slot,
        enemy_status_word,
    }
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn enemy_layout_status_word_loc_3200A(entry_status_byte: u8) -> u16 {
    // ASM loc_3200A (kid.asm:37413): clear `d0`, read byte `1(a1)`, and
    // return zero immediately for empty slots. Positive bytes subtract one,
    // rotate that slot index right by four bits, OR in the big-endian word
    // from `unk_3680C`, then force the high bits with `ori.w #$C000,d0`.
    const ENEMY_LAYOUT_STATUS_LOOKUP_LOC_3200A: [u16; 4] = [0x00F0, 0x01E0, 0x02D0, 0x03C0];

    if entry_status_byte == 0 {
        return 0;
    }

    assert!(
        entry_status_byte < 0x80,
        "loc_3200A handles only non-negative entry-status bytes",
    );
    let slot_index = entry_status_byte.wrapping_sub(1);
    let lookup = ENEMY_LAYOUT_STATUS_LOOKUP_LOC_3200A
        .get(usize::from(slot_index))
        .copied()
        .expect("loc_3200A lookup covers only the four enemy art slots");

    u16::from(slot_index).rotate_right(4) | lookup | 0xC000
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn enemy_layout_status_word_with_negative_sentinel_loc_32036(
    entry_status_byte: u8,
) -> u16 {
    // ASM loc_32036 (kid.asm:37431): negative `1(a1)` bytes skip the
    // `loc_3200A` packing path and store the fixed sentinel word `$8000`
    // into the next `EnemyStatus_Table` slot before advancing by eight bytes.
    if (entry_status_byte as i8) < 0 {
        0x8000
    } else {
        enemy_layout_status_word_loc_3200A(entry_status_byte)
    }
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn enemy_layout_finish_loc_3204E() -> bool {
    // ASM loc_3204E (kid.asm:37443-37445) always calls
    // `Load_EnemyArtPaletteToVRAM` before returning from the enemy-layout
    // initialisation pass, regardless of whether the loop finished normally or
    // the batch count was empty.
    true
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct ObjectOutOfRangePositiveSlotLoc320A6 {
    pub(crate) enemy_status_index: usize,
    pub(crate) enemy_status_word: u16,
    pub(crate) should_delete_current_object: bool,
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn object_out_of_range_positive_slot_loc_320A6(
    object_status_slot: u8,
) -> ObjectOutOfRangePositiveSlotLoc320A6 {
    // ASM loc_320A6 (kid.asm:37487-37492) handles non-negative `$42(a5)` slot
    // bytes by doubling the byte into a word index, storing `$2168` into the
    // matching `EnemyStatus_Table` entry, then falling straight into
    // `loc_320B4` to delete the current object.
    ObjectOutOfRangePositiveSlotLoc320A6 {
        enemy_status_index: usize::from(object_status_slot),
        enemy_status_word: 0x2168,
        should_delete_current_object: true,
    }
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn object_out_of_range_delete_tail_loc_320B4() -> bool {
    // ASM loc_320B4 (kid.asm:37493) is the shared out-of-range tail used by
    // both the negative-slot and positive-slot paths: it jumps straight to
    // `j_Delete_CurrentObject`.
    true
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn object_check_in_range_normal_x_loc_320C0(
    object_x_pos: i16,
    camera_x_pos: i16,
) -> bool {
    // ASM Object_CheckInRange_NormalRange -> loc_320C0 (kid.asm:37496-37507)
    // subtracts `Camera_X_pos` from `x_pos(a3)` in a 16-bit register, then
    // accepts only the inclusive x window `[-$1A4, $2E4]`.
    let relative_x = object_x_pos.wrapping_sub(camera_x_pos);
    (-0x01A4..=0x02E4).contains(&relative_x)
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn object_check_in_range_normal_y_loc_320D8(
    object_y_pos: i16,
    camera_y_pos: i16,
) -> bool {
    // ASM Object_CheckInRange_NormalRange -> loc_320D8 (kid.asm:37508-37517)
    // subtracts `Camera_Y_pos` from `y_pos(a3)` in a 16-bit register, then
    // accepts only the inclusive y window `[-$1A4, $284]` before `rts`.
    let relative_y = object_y_pos.wrapping_sub(camera_y_pos);
    (-0x01A4..=0x0284).contains(&relative_y)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct ObjectCheckInRangeActiveCountGateLoc320E2 {
    pub(crate) uses_tight_window: bool,
    pub(crate) reaches_loc_32102: bool,
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn object_check_in_range_active_count_gate_loc_320E2(
    number_objects: u16,
    object_x_pos: i16,
    camera_x_pos: i16,
) -> ObjectCheckInRangeActiveCountGateLoc320E2 {
    // ASM loc_320E2 (kid.asm:37519-37530) only takes the tight active-object
    // window when `Number_Objects > $14`; otherwise it branches to `loc_3211A`.
    // Inside the tight path, it subtracts `Camera_X_pos` from `x_pos(a3)` and
    // reaches `loc_32102` only when relative x stays inside `[-$104, $244]`.
    let uses_tight_window = number_objects > 0x0014;
    let relative_x = object_x_pos.wrapping_sub(camera_x_pos);
    let reaches_loc_32102 = uses_tight_window && (-0x0104..=0x0244).contains(&relative_x);
    ObjectCheckInRangeActiveCountGateLoc320E2 {
        uses_tight_window,
        reaches_loc_32102,
    }
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn object_check_in_range_wide_load_x_loc_32122(
    object_x_pos: i16,
    camera_x_pos: i16,
) -> bool {
    // ASM loc_3211A -> loc_32122 (kid.asm:37531-37537) is the relaxed x-window
    // path taken once `Number_Objects <= $14`: it subtracts `Camera_X_pos`
    // from `x_pos(a3)` in a 16-bit register and keeps only the inclusive band
    // `[-$1A4, $2E4]` before the y check.
    let relative_x = object_x_pos.wrapping_sub(camera_x_pos);
    (-0x01A4..=0x02E4).contains(&relative_x)
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn object_check_in_range_wide_load_y_loc_32140(
    object_y_pos: i16,
    camera_y_pos: i16,
) -> bool {
    // ASM loc_32122 -> loc_32140 (kid.asm:37538-37548) reuses the relaxed
    // low-object-count path for Y: it subtracts `Camera_Y_pos` from `y_pos(a3)`
    // in a 16-bit register and accepts only the inclusive band `[-$1A4, $284]`
    // before returning `moveq #0,d7`.
    let relative_y = object_y_pos.wrapping_sub(camera_y_pos);
    (-0x01A4..=0x0284).contains(&relative_y)
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
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn gfx_object_sprite_build_order_loc_14A4(
    first_gfx_object_index: Option<usize>,
    next_indices: &[Option<usize>],
) -> Vec<usize> {
    // ASM loc_14A4 (kid.asm:2564): `Make_SpritesFromGfxObjects` loads `0(a3)`,
    // returns at `loc_14B2` when the sentinel head points at zero, otherwise
    // advances `a3` to that worker, calls `j_Make_SpriteFromGfxObject`, and
    // loops until the next link is null.
    let mut sprite_build_order = Vec::new();
    let mut current = first_gfx_object_index;

    while let Some(index) = current {
        sprite_build_order.push(index);
        current = next_indices[index];
    }

    sprite_build_order
}

#[allow(non_snake_case)]
#[allow(dead_code)]
pub(crate) fn initialise_scratch_queue_sub_28FC() -> ScratchQueueSub28FC {
    // ASM sub_28FC / loc_2906 (kid.asm:4854): `move.w #$7A,d7` and the
    // `dbf d7,loc_2906` loop clear 123 longwords starting at `$FFFFF01E`,
    // which covers 6 adjacent `$52`-byte scratch slots before the cursor is
    // rewound to slot 0 and the slot counter is reloaded to `#6`.
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
    // ASM loc_EAC6 (kid.asm:18839): the inclusive `dbf d0` loop starts with
    // `d0=$26`, so it writes 39 next links before clearing the 40th slot.
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
pub(crate) fn initialise_evanesc_object_slots_loc_DF30() -> EvanescShooterObjectSlotsSubEABC {
    // ASM loc_DF30 (kid.asm:17258): after `Initialize_EvanescShooterObjectSlots`
    // seeds `a0=EvanescObject_RAM` and `d0=$12`, the inclusive DBF loop writes
    // each `$A(a0)` link to the next `$0C`-byte slot before clearing the final
    // link and `Addr_FirstEvanescObjectSlot`.
    let mut evanescent_slots = Vec::with_capacity(EVANESCENT_OBJECT_SLOT_COUNT_LOC_DF30);
    let mut slot_addr = EVANESCENT_OBJECT_RAM_ADDR_LOC_DF30;
    for slot_index in 0..EVANESCENT_OBJECT_SLOT_COUNT_LOC_DF30 {
        let is_last = slot_index + 1 == EVANESCENT_OBJECT_SLOT_COUNT_LOC_DF30;
        let next_slot_addr = if is_last {
            0
        } else {
            slot_addr + u32::from(EVANESCENT_OBJECT_SLOT_STRIDE_LOC_DF30)
        };
        evanescent_slots.push(LinkedObjectSlotInit {
            slot_addr,
            next_slot_addr,
        });
        slot_addr += u32::from(EVANESCENT_OBJECT_SLOT_STRIDE_LOC_DF30);
    }

    EvanescShooterObjectSlotsSubEABC {
        next_free_evanescent_object_slot_addr: EVANESCENT_OBJECT_RAM_ADDR_LOC_DF30,
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
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn initialise_shooter_object_slots_loc_DF50() -> ShooterObjectSlotsLocDF50 {
    // ASM loc_DF50 (kid.asm:17269): after `Initialize_EvanescShooterObjectSlots`
    // seeds `a0=ShooterObject_RAM` and `d0=$26`, the inclusive DBF loop writes
    // each `$A(a0)` link to the next `$0C`-byte slot, clears the final link,
    // then clears `Addr_FirstShooterObjectSlot`.
    let mut shooter_slots = Vec::with_capacity(SHOOTER_OBJECT_SLOT_COUNT_LOC_DF50);
    let mut slot_addr = SHOOTER_OBJECT_RAM_ADDR_LOC_DF50;
    for slot_index in 0..SHOOTER_OBJECT_SLOT_COUNT_LOC_DF50 {
        let is_last = slot_index + 1 == SHOOTER_OBJECT_SLOT_COUNT_LOC_DF50;
        let next_slot_addr = if is_last {
            0
        } else {
            slot_addr + u32::from(SHOOTER_OBJECT_SLOT_STRIDE_LOC_DF50)
        };
        shooter_slots.push(LinkedObjectSlotInit {
            slot_addr,
            next_slot_addr,
        });
        slot_addr += u32::from(SHOOTER_OBJECT_SLOT_STRIDE_LOC_DF50);
    }

    ShooterObjectSlotsLocDF50 {
        next_free_shooter_object_slot_addr: SHOOTER_OBJECT_RAM_ADDR_LOC_DF50,
        first_shooter_object_slot_addr: 0,
        shooter_slots,
    }
}

/// ASM loc_E07A (kid.asm:17395): load the hidden/prize block list base from
/// `$FFFFF8D4`, copy the current layout word pointer from `a2` into `d4`, then
/// subtract 8 from `a0` so the following scan pre-increments onto the
/// first 8-byte metadata record before comparing `(a0)` with `d4`.
#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn hidden_block_evanescent_scan_start_loc_E07A(
    hidden_block_list_base_addr: u32,
    layout_word_addr: u16,
) -> HiddenBlockEvanescentScanStartLocE07A {
    HiddenBlockEvanescentScanStartLocE07A {
        preincrement_cursor_addr: hidden_block_list_base_addr.wrapping_sub(8),
        first_record_addr: hidden_block_list_base_addr,
        layout_word_addr,
    }
}

/// ASM loc_E082 (kid.asm:17400): advance `a0` by one 8-byte hidden-block
/// metadata record until `(a0)` matches the layout word address in `d4`. Once
/// matched, `tst.w 6(a0)` leaves the existing reveal sprite words alone for a
/// non-negative control word, or replaces them with `$E101/$1058` when that
/// word is negative.
#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn hidden_block_evanescent_metadata_scan_loc_E082(
    hidden_block_list_base_addr: u32,
    layout_word_addr: u16,
    records: &[HiddenBlockMetadataRecordLocE082],
) -> Option<HiddenBlockEvanescentScanLocE082> {
    records
        .iter()
        .position(|record| record.layout_word_addr == layout_word_addr)
        .map(|record_index| {
            let record = records[record_index];
            HiddenBlockEvanescentScanLocE082 {
                matched_record_addr: hidden_block_list_base_addr + (record_index as u32 * 8),
                scanned_records: record_index + 1,
                evanescent_sprite_words: (record.control_word < 0).then_some((0xE101, 0x1058)),
            }
        })
}

/// ASM loc_E096 (kid.asm:17409): reveal the hidden block by writing `$60` to
/// the layout byte at `(a2)`, allocate a `$2001` object slot, install handler
/// `$E29A`, then copy the block column/row, layout pointer, and selected sprite
/// words into object fields `$44/$46/$48/$4C/$4E`.
#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn hidden_block_reveal_worker_loc_E096(
    block_col: u16,
    block_row: u16,
    layout_word_addr: u32,
    sprite_tile_word: u16,
    sprite_mapping: u16,
) -> HiddenBlockRevealWorkerLocE096 {
    HiddenBlockRevealWorkerLocE096 {
        revealed_layout_byte: 0x60,
        allocation_spec: 0x2001,
        object_handler_addr: 0x0000_E29A,
        block_col,
        block_row,
        layout_word_addr,
        sprite_tile_word,
        sprite_mapping,
    }
}

/// ASM loc_E48E (kid.asm:17988): after either revealing a matching prize
/// block or skipping a non-prize cell, advance to the same column in the next
/// row by incrementing `d5`, adding `Level_width_tiles` to `a2`, and looping
/// back to `sub_E446` while the DBF post-decrement counter has not reached
/// `$FFFF`.
#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn hidden_block_prize_reveal_scan_tail_loc_E48E(
    block_col: u16,
    layout_word_addr: u16,
    level_width_tiles: u16,
    dbf_counter: u16,
) -> HiddenBlockPrizeRevealScanTailLocE48E {
    let next_dbf_counter = dbf_counter.wrapping_sub(1);
    HiddenBlockPrizeRevealScanTailLocE48E {
        next_block_col: block_col.wrapping_add(1),
        next_layout_word_addr: layout_word_addr.wrapping_add(level_width_tiles),
        next_dbf_counter,
        loops_to_sub_e446: next_dbf_counter != 0xFFFF,
    }
}

/// ASM loc_EA3E (kid.asm:18770): decrement the evanescent object's timer at
/// `$8(a2)`. Only the exact `1 -> 0` transition restores the timer to one and
/// inspects the pointed Level_Layout byte; removal continues to `loc_EA5A`
/// only when that byte is negative and its low nibble is less than `$D`.
#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn evanescent_block_removal_gate_loc_EA3E(
    timer: u16,
    layout_cell_hi_byte: u8,
) -> EvanescentBlockRemovalGateLocEA3E {
    let decremented_timer = timer.wrapping_sub(1);
    if decremented_timer != 0 {
        return EvanescentBlockRemovalGateLocEA3E {
            next_timer: decremented_timer,
            should_try_remove_block: false,
        };
    }

    EvanescentBlockRemovalGateLocEA3E {
        next_timer: 1,
        should_try_remove_block: (layout_cell_hi_byte as i8) < 0
            && (layout_cell_hi_byte & 0x0F) < 0x0D,
    }
}

/// ASM loc_EA5A (kid.asm:18783): compare the current Level_Layout word at
/// `(a1)` with the evanescent object's saved word at `$6(a2)`. A mismatch
/// skips the erase but still continues to `loc_EA72`; a match calls
/// `EraseBlockFromLevelLayout` with the object's column/row and then writes
/// byte `$20` back to `(a1)`.
#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn evanescent_block_erase_match_loc_EA5A(
    layout_cell_word: u16,
    saved_cell_word: u16,
    block_col: u16,
    block_row: u16,
    erased_layout_cell_word: u16,
) -> EvanescentBlockEraseMatchLocEA5A {
    if layout_cell_word != saved_cell_word {
        return EvanescentBlockEraseMatchLocEA5A {
            should_erase_block_from_level_layout: false,
            erased_block_col: None,
            erased_block_row: None,
            final_layout_cell_word: layout_cell_word,
        };
    }

    EvanescentBlockEraseMatchLocEA5A {
        should_erase_block_from_level_layout: true,
        erased_block_col: Some(block_col),
        erased_block_row: Some(block_row),
        final_layout_cell_word: 0x2000 | (erased_layout_cell_word & 0x00FF),
    }
}

/// ASM loc_EA72 (kid.asm:18792): save the removed evanescent object's old
/// `$A(a2)` next link in `d0`, unlink `a2` from `Addr_FirstEvanescObjectSlot`,
/// push `a2` onto `Addr_NextFreeEvanescObjectSlot`, and use the saved `d0` as
/// the next active object to process.
#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn evanescent_object_move_active_to_free_loc_EA72(
    active_slots: &[LinkedObjectSlotInit],
    target_slot_addr: u32,
    first_free_slot_addr: u32,
) -> Option<EvanescentObjectListMoveLocEA72> {
    let target_index = active_slots
        .iter()
        .position(|slot| slot.slot_addr == target_slot_addr)?;
    let mut slots = active_slots.to_vec();
    let next_active_slot_to_process_addr = slots[target_index].next_slot_addr;

    if target_index == 0 {
        let first_active_slot_addr = next_active_slot_to_process_addr;
        slots[target_index].next_slot_addr = first_free_slot_addr;
        return Some(EvanescentObjectListMoveLocEA72 {
            first_active_slot_addr,
            first_free_slot_addr: target_slot_addr,
            next_active_slot_to_process_addr,
            slots,
        });
    }

    let predecessor_index = slots
        .iter()
        .position(|slot| slot.next_slot_addr == target_slot_addr)?;
    slots[predecessor_index].next_slot_addr = next_active_slot_to_process_addr;
    slots[target_index].next_slot_addr = first_free_slot_addr;

    Some(EvanescentObjectListMoveLocEA72 {
        first_active_slot_addr: active_slots.first().map_or(0, |slot| slot.slot_addr),
        first_free_slot_addr: target_slot_addr,
        next_active_slot_to_process_addr,
        slots,
    })
}

/// ASM loc_EA86 (kid.asm:18801): when the removed evanescent object is not
/// `Addr_FirstEvanescObjectSlot`, start at the active-list head and follow each
/// `$A(a0)` next link until it equals `a2`; the current `a0` is the predecessor
/// whose link is spliced by the next block.
#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn evanescent_object_predecessor_scan_loc_EA86(
    active_slots: &[LinkedObjectSlotInit],
    target_slot_addr: u32,
) -> Option<EvanescentObjectPredecessorLocEA86> {
    let mut current_slot_addr = active_slots.first()?.slot_addr;

    for scanned_links in 1..=active_slots.len() {
        let current_slot = active_slots
            .iter()
            .find(|slot| slot.slot_addr == current_slot_addr)?;
        if current_slot.next_slot_addr == target_slot_addr {
            return Some(EvanescentObjectPredecessorLocEA86 {
                predecessor_slot_addr: current_slot.slot_addr,
                scanned_links,
            });
        }
        if current_slot.next_slot_addr == 0 {
            return None;
        }
        current_slot_addr = current_slot.next_slot_addr;
    }

    None
}

/// ASM loc_EA92 (kid.asm:18809): after the predecessor scan, copy the removed
/// evanescent object's `$A(a2)` next link into the predecessor's `$A(a0)` link.
#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn evanescent_object_splice_predecessor_loc_EA92(
    active_slots: &[LinkedObjectSlotInit],
    predecessor_slot_addr: u32,
    target_slot_addr: u32,
) -> Option<EvanescentObjectSpliceLocEA92> {
    let mut slots = active_slots.to_vec();
    let target_next_slot_addr = slots
        .iter()
        .find(|slot| slot.slot_addr == target_slot_addr)?
        .next_slot_addr;
    let predecessor = slots
        .iter_mut()
        .find(|slot| slot.slot_addr == predecessor_slot_addr)?;
    predecessor.next_slot_addr = target_next_slot_addr;

    Some(EvanescentObjectSpliceLocEA92 { slots })
}

/// ASM loc_EA98 (kid.asm:18812): push the removed evanescent object onto
/// `Addr_NextFreeEvanescObjectSlot` by writing the old free-list head to
/// `$A(a2)`, then continue active processing only when saved `d0` is non-zero.
#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn evanescent_object_push_free_list_loc_EA98(
    active_slots: &[LinkedObjectSlotInit],
    target_slot_addr: u32,
    first_free_slot_addr: u32,
    saved_next_active_slot_addr: u32,
) -> Option<EvanescentObjectFreeListPushLocEA98> {
    let mut slots = active_slots.to_vec();
    let target_slot = slots
        .iter_mut()
        .find(|slot| slot.slot_addr == target_slot_addr)?;
    target_slot.next_slot_addr = first_free_slot_addr;

    Some(EvanescentObjectFreeListPushLocEA98 {
        first_free_slot_addr: target_slot_addr,
        should_continue_active_scan: saved_next_active_slot_addr != 0,
        next_active_slot_to_process_addr: saved_next_active_slot_addr,
        slots,
    })
}

/// ASM loc_EAB2 (kid.asm:18824): when the evanescent object scan reaches the
/// null active-list link, copy the live sprite cursor in `a3` and count byte in
/// `d3` back to `Addr_NextSpriteSlot` and `Number_Sprites` before returning.
#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn evanescent_sprite_scan_return_loc_EAB2(
    sprite_cursor_addr: u32,
    sprite_count: u8,
) -> EvanescentSpriteScanReturnLocEAB2 {
    EvanescentSpriteScanReturnLocEAB2 {
        next_sprite_slot_addr: sprite_cursor_addr,
        number_sprites: sprite_count,
    }
}

/// ASM loc_EAE4 (kid.asm:18858): unless the `$FFFFFA64` flag is set, expand
/// Kid's hitbox by one pixel on each side, clamp it to level pixel bounds,
/// convert to inclusive tile rows/columns, and scan for bit-15 block cells
/// whose block type is `$A` before falling through to the bullet loader.
#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn evanescent_bullet_hitbox_scan_loc_EAE4(
    input: EvanescentBulletHitboxScanInputLocEAE4,
    grid: &terrain::BlockGrid,
) -> EvanescentBulletHitboxScanLocEAE4 {
    if input.skip_scan_flag {
        return EvanescentBulletHitboxScanLocEAE4::SkipToLoadBullets;
    }

    let top_px = i32::from(input.kid_hitbox_top).saturating_sub(1).max(0);
    let bottom_px = i32::from(
        evanescent_bullet_bottom_clamp_loc_EAF8(input.kid_hitbox_bottom, input.level_height_pixels)
            .bottom_px,
    );
    let vertical_rows = evanescent_bullet_vertical_rows_loc_EB0C(
        u16::try_from(top_px).expect("clamped top pixel should fit in u16"),
        u16::try_from(bottom_px).expect("clamped bottom pixel should fit in u16"),
    );
    let left_px = i32::from(input.kid_hitbox_left).saturating_sub(1).max(0);
    let right_px = i32::from(
        evanescent_bullet_right_clamp_loc_EB2A(input.kid_hitbox_right, input.level_width_pixels)
            .right_px,
    );

    let horizontal_columns = evanescent_bullet_horizontal_columns_loc_EB3E(
        u16::try_from(left_px).expect("clamped left pixel should fit in u16"),
        u16::try_from(right_px).expect("clamped right pixel should fit in u16"),
    );

    let found_evanescent_block = evanescent_bullet_scan_rows_loc_EB48(
        grid,
        vertical_rows.top_row,
        vertical_rows.row_dbf_counter,
        horizontal_columns.left_col,
        horizontal_columns.col_dbf_counter,
    )
    .is_some();

    EvanescentBulletHitboxScanLocEAE4::Scan {
        top_row: vertical_rows.top_row,
        bottom_row: vertical_rows.bottom_row,
        left_col: horizontal_columns.left_col,
        right_col: horizontal_columns.right_col,
        row_dbf_counter: vertical_rows.row_dbf_counter,
        col_dbf_counter: horizontal_columns.col_dbf_counter,
        found_evanescent_block,
    }
}

/// ASM loc_EAF8 (kid.asm:18866): expand Kid's bottom hitbox edge by one
/// pixel, then clamp it to `Level_height_pixels - 1` before the row conversion
/// at `loc_EB0C`.
#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn evanescent_bullet_bottom_clamp_loc_EAF8(
    kid_hitbox_bottom: u16,
    level_height_pixels: u16,
) -> EvanescentBulletBottomClampLocEAF8 {
    EvanescentBulletBottomClampLocEAF8 {
        bottom_px: kid_hitbox_bottom
            .saturating_add(1)
            .min(level_height_pixels.saturating_sub(1)),
    }
}

/// ASM loc_EB0C (kid.asm:18874): arithmetic-shift the clamped top and bottom
/// pixel edges by four to get tile rows, subtract top from bottom for the
/// inclusive DBF row counter, then double the top row for the `$FFFF4A04`
/// row-pointer table lookup.
#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn evanescent_bullet_vertical_rows_loc_EB0C(
    top_px: u16,
    bottom_px: u16,
) -> EvanescentBulletVerticalRowsLocEB0C {
    let top_row = top_px >> 4;
    let bottom_row = bottom_px >> 4;

    EvanescentBulletVerticalRowsLocEB0C {
        top_row,
        bottom_row,
        row_dbf_counter: bottom_row.wrapping_sub(top_row),
        first_row_pointer_table_byte_offset: top_row.wrapping_mul(2),
    }
}

/// ASM loc_EB2A (kid.asm:18886): expand Kid's right hitbox edge by one pixel.
/// If the expanded word is strictly below `Level_width_pixels`, keep it;
/// otherwise clamp to `Level_width_pixels - 1` before the horizontal tile
/// conversion at `loc_EB3E`.
#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn evanescent_bullet_right_clamp_loc_EB2A(
    kid_hitbox_right: u16,
    level_width_pixels: u16,
) -> EvanescentBulletRightClampLocEB2A {
    let expanded_right = kid_hitbox_right.saturating_add(1);
    EvanescentBulletRightClampLocEB2A {
        right_px: if expanded_right < level_width_pixels {
            expanded_right
        } else {
            level_width_pixels.saturating_sub(1)
        },
    }
}

/// ASM loc_EB3E (kid.asm:18894): arithmetic-shift the clamped left and right
/// pixel edges by four to get tile columns, subtract left from right for the
/// inclusive DBF column counter, then double the left column before adding it
/// into the current row pointer.
#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn evanescent_bullet_horizontal_columns_loc_EB3E(
    left_px: u16,
    right_px: u16,
) -> EvanescentBulletHorizontalColumnsLocEB3E {
    let left_col = left_px >> 4;
    let right_col = right_px >> 4;

    EvanescentBulletHorizontalColumnsLocEB3E {
        left_col,
        right_col,
        col_dbf_counter: right_col.wrapping_sub(left_col),
        first_cell_byte_offset: left_col.wrapping_mul(2),
    }
}

/// ASM loc_EB48 (kid.asm:18901): at the start of each scanned row, copy the
/// horizontal DBF counter from `d2` to `d3` and the row pointer from `a0` to
/// `a1`, so every row starts at the same left column before `loc_EB4C` walks
/// rightward through the inclusive column span.
#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn evanescent_bullet_scan_rows_loc_EB48(
    grid: &terrain::BlockGrid,
    top_row: u16,
    row_dbf_counter: u16,
    left_col: u16,
    col_dbf_counter: u16,
) -> Option<EvanescentBulletScanHitLocEB48> {
    for row_delta in 0..=row_dbf_counter {
        let row = top_row.wrapping_add(row_delta);
        // ASM loc_EB48: `move.w d2,d3; move.w a0,a1` resets the inner scan.
        for col_delta in 0..=col_dbf_counter {
            let col = left_col.wrapping_add(col_delta);
            let cell = grid.cell_at(col, row);
            if cell.has_block && cell.block_type == terrain::BlockType::Evanescent {
                return Some(EvanescentBulletScanHitLocEB48 { col, row });
            }
        }
    }

    None
}

/// ASM loc_EB62 (kid.asm:18913): after `bclr #$F,d0`, cells with no block
/// flag and block-flagged cells whose masked type is not `$A00` both continue
/// the inclusive column/row DBF scan instead of triggering an evanescent block.
#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn evanescent_bullet_cell_scan_loc_EB62(
    level_layout_word: u16,
) -> EvanescentBulletCellScanLocEB62 {
    if level_layout_word & 0x8000 == 0 {
        return EvanescentBulletCellScanLocEB62::Continue;
    }

    if level_layout_word & 0x0F00 == 0x0A00 {
        EvanescentBulletCellScanLocEB62::TriggerEvanescentBlock
    } else {
        EvanescentBulletCellScanLocEB62::Continue
    }
}

/// ASM loc_EC52 (kid.asm:19111): process the current evanescent object only
/// when its age word `$8(a0)` is zero. The ROM converts stored block col/row at
/// `$2/$4(a0)` to centred pixel coordinates via `lsl #4; addq #8`, then hands
/// `$6(a0)` direction bits to the bullet allocation loop.
#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn evanescent_bullet_load_candidate_loc_EC52(
    layout_word_addr: u16,
    block_col: u16,
    block_row: u16,
    direction_bits: u16,
    age: u16,
) -> Option<EvanescentBulletLoadCandidateLocEC52> {
    if age != 0 {
        return None;
    }

    Some(EvanescentBulletLoadCandidateLocEC52 {
        layout_word_addr,
        block_x_px: block_col.wrapping_mul(16).wrapping_add(8),
        block_y_px: block_row.wrapping_mul(16).wrapping_add(8),
        direction_bits,
    })
}

/// ASM loc_EC76 (kid.asm:19127): test evanescent direction bits from `d4=3`
/// down to zero. Each set bit pops one shooter slot from
/// `Addr_NextFreeShooterObjectSlot`, pushes it to `Addr_FirstShooterObjectSlot`,
/// copies source layout/x/y/direction words, writes state `$FFFE`, and exits
/// early when the free-list head is zero.
#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn evanescent_bullet_direction_load_loc_EC76(
    candidate: EvanescentBulletLoadCandidateLocEC52,
    free_slots: &[LinkedObjectSlotInit],
    next_free_shooter_object_slot_addr: u32,
    first_shooter_object_slot_addr: u32,
) -> Option<EvanescentBulletDirectionLoadLocEC76> {
    let mut shooter_slots = free_slots.to_vec();
    let mut next_free_slot_addr = next_free_shooter_object_slot_addr;
    let mut first_active_slot_addr = first_shooter_object_slot_addr;
    let mut loaded_slots = Vec::new();
    let mut stopped_on_empty_free_list = false;

    for direction in (0u16..=3).rev() {
        if candidate.direction_bits & (1u16 << direction) == 0 {
            continue;
        }

        if next_free_slot_addr == 0 {
            stopped_on_empty_free_list = true;
            break;
        }

        let slot_index = shooter_slots
            .iter()
            .position(|slot| slot.slot_addr == next_free_slot_addr)?;
        let loaded_slot_addr = shooter_slots[slot_index].slot_addr;
        next_free_slot_addr = shooter_slots[slot_index].next_slot_addr;
        shooter_slots[slot_index].next_slot_addr = first_active_slot_addr;

        loaded_slots.push(LoadedShooterObjectLocEC76 {
            slot_addr: loaded_slot_addr,
            layout_word_addr: candidate.layout_word_addr,
            block_x_px: candidate.block_x_px,
            block_y_px: candidate.block_y_px,
            direction,
            state: 0xFFFE,
            next_active_slot_addr: first_active_slot_addr,
        });
        first_active_slot_addr = loaded_slot_addr;
    }

    Some(EvanescentBulletDirectionLoadLocEC76 {
        next_free_shooter_object_slot_addr: next_free_slot_addr,
        first_shooter_object_slot_addr: first_active_slot_addr,
        shooter_slots,
        loaded_slots,
        stopped_on_empty_free_list,
    })
}

/// ASM loc_EC9E (kid.asm:19143): after either a clear direction bit or a
/// loaded shooter object, `dbf d4,loc_EC76` decrements the direction counter
/// and loops while it has not wrapped to `$FFFF`.
#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn evanescent_bullet_direction_loop_tail_loc_EC9E(
    direction_counter: u16,
) -> EvanescentBulletDirectionLoopTailLocEC9E {
    let next_direction_counter = direction_counter.wrapping_sub(1);
    EvanescentBulletDirectionLoopTailLocEC9E {
        next_direction_counter,
        loops_to_loc_ec76: next_direction_counter != 0xFFFF,
    }
}

/// ASM loc_ECC4 (kid.asm:19162): when the aged evanescent object is not the
/// active-list head, walk `$A(a3)` links from `Addr_FirstEvanescObjectSlot`
/// until the successor equals the current `a0` slot.
#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn evanescent_bullet_recycle_predecessor_scan_loc_ECC4(
    active_slots: &[LinkedObjectSlotInit],
    target_slot_addr: u32,
) -> Option<EvanescentBulletRecyclePredecessorLocECC4> {
    let mut current_slot_addr = active_slots.first()?.slot_addr;

    for scanned_links in 1..=active_slots.len() {
        let current_slot = active_slots
            .iter()
            .find(|slot| slot.slot_addr == current_slot_addr)?;
        let successor_slot_addr = current_slot.next_slot_addr;
        if successor_slot_addr == target_slot_addr {
            return Some(EvanescentBulletRecyclePredecessorLocECC4 {
                predecessor_slot_addr: current_slot.slot_addr,
                scanned_links,
            });
        }
        if successor_slot_addr == 0 {
            return None;
        }
        current_slot_addr = successor_slot_addr;
    }

    None
}

/// ASM loc_ECD2 (kid.asm:19170): after `loc_ECC4` finds the predecessor in
/// `a3`, splice the aged evanescent object out by copying `$A(a0)` into
/// `$A(a3)`.
#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn evanescent_bullet_recycle_splice_predecessor_loc_ECD2(
    active_slots: &[LinkedObjectSlotInit],
    predecessor_slot_addr: u32,
    target_slot_addr: u32,
) -> Option<EvanescentBulletRecycleSpliceLocECD2> {
    let mut slots = active_slots.to_vec();
    let target_next_slot_addr = slots
        .iter()
        .find(|slot| slot.slot_addr == target_slot_addr)?
        .next_slot_addr;
    let predecessor = slots
        .iter_mut()
        .find(|slot| slot.slot_addr == predecessor_slot_addr)?;
    predecessor.next_slot_addr = target_next_slot_addr;

    Some(EvanescentBulletRecycleSpliceLocECD2 { slots })
}

/// ASM loc_ECD8 (kid.asm:19173): after either the head removal path or the
/// predecessor splice, push the aged evanescent slot onto
/// `Addr_NextFreeEvanescObjectSlot`. The saved `$A(a0)` value in `d0` decides
/// whether the active evanescent scan continues at `loc_EC52` or falls through
/// to `Process_ShooterObjects`.
#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn evanescent_bullet_recycle_free_list_push_loc_ECD8(
    active_slots: &[LinkedObjectSlotInit],
    target_slot_addr: u32,
    saved_next_active_slot_addr: u32,
    first_free_slot_addr: u32,
) -> Option<EvanescentBulletRecycleFreeListPushLocECD8> {
    let mut slots = active_slots.to_vec();
    let target = slots
        .iter_mut()
        .find(|slot| slot.slot_addr == target_slot_addr)?;
    target.next_slot_addr = first_free_slot_addr;

    Some(EvanescentBulletRecycleFreeListPushLocECD8 {
        first_free_slot_addr: target_slot_addr,
        should_continue_active_scan: saved_next_active_slot_addr != 0,
        next_active_slot_to_process_addr: saved_next_active_slot_addr,
        slots,
    })
}

const EVANESCENT_BULLET_ACTION_TABLE_UNK_ECEC: [u8; 24] = [
    0x00, 0x81, 0x04, 0x81, 0x04, 0x81, 0x04, 0x81, 0x04, 0x81, 0x04, 0x81, 0x04, 0x83, 0x00, 0x80,
    0x00, 0x80, 0x00, 0x80, 0x00, 0x80, 0x00, 0x00,
];

/// ASM loc_ED04 (kid.asm:19208): for a non-expiring evanescent object, index
/// `unk_ECEC` by the pre-increment age word still in `d0`. A non-zero byte
/// dispatches to `loc_ED16`; a zero byte skips work and follows `$A(a0)` to
/// either the next active evanescent object or `Process_ShooterObjects`.
#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn evanescent_bullet_action_dispatch_loc_ED04(
    age_before_increment: u16,
    next_active_slot_addr: u32,
) -> Option<EvanescentBulletActionDispatchLocED04> {
    let action_byte =
        *EVANESCENT_BULLET_ACTION_TABLE_UNK_ECEC.get(usize::from(age_before_increment))?;
    if action_byte != 0 {
        return Some(EvanescentBulletActionDispatchLocED04::ProcessAction { action_byte });
    }
    if next_active_slot_addr != 0 {
        Some(EvanescentBulletActionDispatchLocED04::ContinueActiveScan {
            next_active_slot_addr,
        })
    } else {
        Some(EvanescentBulletActionDispatchLocED04::ProcessShooterObjects)
    }
}

/// ASM loc_ED16 (kid.asm:19216): negative action bytes optionally draw the
/// fading evanescent-block sprite when `$2/$4(a0)` is inside the inclusive
/// camera block window, then `loc_ED52` uses bit 0 to decide whether to rewrite
/// the level-layout word and bit 1 to choose the collision-overlay path.
/// Positive action bytes branch immediately to `loc_EDBC`.
#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn evanescent_bullet_action_loc_ED16(
    input: EvanescentBulletActionInputLocED16,
) -> EvanescentBulletActionLocED16 {
    if (input.action_byte & 0x80) == 0 {
        return EvanescentBulletActionLocED16::PositiveBranchToLocEDBC {
            layout_word: 0xE31C_u16.wrapping_add(input.loc_edbc_layout_offset),
        };
    }

    let sprite = (input.block_col >= input.min_visible_col
        && input.block_col <= input.max_visible_col
        && input.block_row >= input.min_visible_row
        && input.block_row <= input.max_visible_row)
        .then(|| EvanescentBulletSpriteLocED16 {
            y: input
                .block_row
                .wrapping_mul(16)
                .wrapping_sub(input.camera_y_px)
                .wrapping_add(0x84),
            link: input.current_sprite_link.wrapping_add(1),
            tile_word: 0x8294,
            x: input
                .block_col
                .wrapping_mul(16)
                .wrapping_sub(input.camera_x_px)
                .wrapping_add(0x84),
        });

    EvanescentBulletActionLocED16::NegativeAction {
        sprite,
        should_update_layout: (input.action_byte & 0x01) != 0,
        use_collision_overlay: (input.action_byte & 0x02) != 0,
    }
}

/// ASM loc_ED7C (kid.asm:19258): collision-overlay evanescent actions use
/// `Foreground_theme` to select a theme collision table, read one byte using
/// the terrain byte in `d1` as the index, `ror.w #4` that byte, then OR the
/// rotated word back into `d1` before the shared layout write.
#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn evanescent_bullet_collision_overlay_loc_ED7C(
    terrain_layout_byte: u8,
    theme_collision_byte: u8,
) -> u16 {
    u16::from(theme_collision_byte).rotate_right(4) | u16::from(terrain_layout_byte)
}

/// ASM loc_ED98 (kid.asm:19269): write the selected layout word `d1` through
/// the saved `(a0)` level-layout pointer, append `$2(a0)`, `$4(a0)`, and `d1`
/// to `FGPlane_UpdateQueueCell`, then either continue the active evanescent
/// scan when `$A(a0)` is non-zero or fall through to shooter-object handling.
#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn evanescent_bullet_layout_write_loc_ED98(
    layout_word_addr: u16,
    block_col: u16,
    block_row: u16,
    layout_word: u16,
    next_active_slot_addr: u32,
) -> EvanescentBulletLayoutWriteLocED98 {
    EvanescentBulletLayoutWriteLocED98 {
        layout_word_addr,
        final_layout_word: layout_word,
        queued_cell: FgUpdateQueueCellLocED98 {
            block_col,
            block_row,
            layout_word,
        },
        next_step: if next_active_slot_addr != 0 {
            EvanescentBulletLayoutWriteNextLocED98::ContinueActiveScan {
                next_active_slot_addr,
            }
        } else {
            EvanescentBulletLayoutWriteNextLocED98::ProcessShooterObjects
        },
    }
}

/// ASM loc_EE2A (kid.asm:19331): moving-right shooter objects with positive
/// age reset `$8(a0)` to zero and advance `(a0)` every fourth frame, then read
/// the current level-layout word. Bit 14 gates the interaction test; ages two
/// and three interact immediately, while ages zero/one interact only when
/// bits `$3000` are also present.
#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn shooter_object_moving_right_gate_loc_EE2A(
    age_after_increment: u16,
    current_layout_word_addr: u16,
    current_layout_word: u16,
) -> ShooterObjectMovingRightGateLocEE2A {
    let (state_word, layout_word_addr) = if age_after_increment == 4 {
        (0, current_layout_word_addr.wrapping_add(2))
    } else {
        (age_after_increment, current_layout_word_addr)
    };

    ShooterObjectMovingRightGateLocEE2A {
        state_word,
        layout_word_addr,
        next_step: shooter_object_moving_right_collision_gate_loc_EE38(
            state_word,
            current_layout_word,
        ),
    }
}

/// ASM loc_EE38 (kid.asm:19338): `btst #$E,d1` skips clear bit-14 cells; if bit
/// 14 is set, age words two and above interact immediately, while younger ages
/// only interact when `andi.w #$3000,d1` leaves a non-zero value.
#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn shooter_object_moving_right_collision_gate_loc_EE38(
    state_word: u16,
    current_layout_word: u16,
) -> ShooterObjectMovingRightNextLocEE2A {
    if (current_layout_word & 0x4000) == 0 {
        return ShooterObjectMovingRightNextLocEE2A::ContinueMovingRight;
    }

    if state_word >= 2 || (current_layout_word & 0x3000) != 0 {
        ShooterObjectMovingRightNextLocEE2A::InteractLevelLayout
    } else {
        ShooterObjectMovingRightNextLocEE2A::ContinueMovingRight
    }
}

/// ASM loc_EE52 (kid.asm:19348): `addq.w #4,2(a0)` advances the moving-right
/// shooter, then `d1=0`, `d2=-4`, and `d3=$8A96` feed the shared sprite maker.
#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn shooter_object_moving_right_sprite_tail_loc_EE52(
    x_pos: u16,
) -> ShooterObjectSpriteTailLocEE52 {
    ShooterObjectSpriteTailLocEE52 {
        x_pos: x_pos.wrapping_add(4),
        sprite_x_offset: 0,
        sprite_y_offset: -4,
        tile_word: 0x8A96,
    }
}

/// ASM loc_EE80 (kid.asm:19367): when the moving-up shooter age reaches four,
/// clear `$8(a0)` and subtract `Level_width_tiles` from the layout pointer.
/// The following collision gate interacts for bit-14 cells while age is below
/// two; older ages require the moving-up `$2000` directional bit.
#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn shooter_object_moving_up_gate_loc_EE80(
    age_after_increment: u16,
    current_layout_word_addr: u16,
    level_width_tiles: u16,
    selected_layout_word: u16,
) -> ShooterObjectMovingUpGateLocEE80 {
    let (state_word, layout_word_addr) = if age_after_increment == 4 {
        (0, current_layout_word_addr.wrapping_sub(level_width_tiles))
    } else {
        (age_after_increment, current_layout_word_addr)
    };

    let next_step = if (selected_layout_word & 0x4000) == 0 {
        ShooterObjectMovingUpNextLocEE80::ContinueMovingUp
    } else if state_word < 2 || (selected_layout_word & 0x2000) != 0 {
        ShooterObjectMovingUpNextLocEE80::InteractLevelLayout
    } else {
        ShooterObjectMovingUpNextLocEE80::ContinueMovingUp
    };

    ShooterObjectMovingUpGateLocEE80 {
        state_word,
        layout_word_addr,
        next_step,
    }
}

/// ASM loc_DE0 (kid.asm:1602): after `Allocate_ObjectSlot` pops a free object,
/// walk `Addr_FirstObjectSlot` while the new caller word (`d6`, copied from
/// incoming `a0`) is strictly higher than the current slot's `$E(a1)` word.
/// Equal caller words stop the scan, so the new slot inserts before equal keys.
#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn allocate_object_slot_insert_index_loc_DE0(
    active_slots: &[ObjectSlotOrderLocDE0],
    new_caller_order: u16,
) -> usize {
    active_slots
        .iter()
        .position(|slot| new_caller_order <= slot.caller_order)
        .unwrap_or(active_slots.len())
}

/// ASM loc_DF2 (kid.asm:1611): splice the allocated object after the
/// predecessor found by `loc_DE0`. The ROM copies the predecessor link into
/// `0(a0)`, then stores `a0` through the predecessor link slot.
#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn allocate_object_slot_splice_loc_DF2(
    active_slots: &[ObjectSlotOrderLocDE0],
    new_slot: ObjectSlotOrderLocDE0,
) -> AllocatedObjectSlotListLocDF2 {
    let inserted_index =
        allocate_object_slot_insert_index_loc_DE0(active_slots, new_slot.caller_order);
    let mut ordered_slots = active_slots.to_vec();
    ordered_slots.insert(inserted_index, new_slot);

    let slots = ordered_slots
        .iter()
        .enumerate()
        .map(|(slot_index, slot)| LinkedObjectSlotInit {
            slot_addr: slot.slot_addr,
            next_slot_addr: ordered_slots
                .get(slot_index + 1)
                .map_or(0, |next_slot| next_slot.slot_addr),
        })
        .collect::<Vec<_>>();

    AllocatedObjectSlotListLocDF2 {
        inserted_index,
        first_slot_addr: slots.first().map_or(0, |slot| slot.slot_addr),
        slots,
    }
}

/// ASM loc_EA2 (kid.asm:1720): `Deallocate_ObjectSlot` starts with `a0` at
/// `Addr_FirstObjectSlot`, then repeatedly loads the predecessor link word. A
/// null link exits without deallocating; a link equal to `a5` identifies the
/// predecessor whose `0(a0)` will be spliced over the removed object.
#[allow(non_snake_case)]
#[allow(dead_code)]
pub(crate) fn deallocate_object_slot_predecessor_loc_EA2(
    active_slots: &[LinkedObjectSlotInit],
    target_slot_addr: u32,
) -> Option<DeallocateObjectSlotPredecessorLocEA2> {
    let mut predecessor_link_addr = 0xFFFF_F842;
    let mut next_slot_addr = active_slots.first().map_or(0, |slot| slot.slot_addr);

    for scanned_links in 1..=active_slots.len() {
        if next_slot_addr == 0 {
            return None;
        }
        if next_slot_addr == target_slot_addr {
            return Some(DeallocateObjectSlotPredecessorLocEA2 {
                predecessor_link_addr,
                scanned_links,
            });
        }

        let current_slot = active_slots
            .iter()
            .find(|slot| slot.slot_addr == next_slot_addr)?;
        predecessor_link_addr = current_slot.slot_addr;
        next_slot_addr = current_slot.next_slot_addr;
    }

    None
}

/// ASM loc_EB0 (kid.asm:1730): splice the current object in `a5` out of
/// `Addr_FirstObjectSlot`, push it onto `Addr_NextFreeObjectSlot`, decrement
/// `Number_Objects`, deallocate each non-null gfx child at `$36/$3A/$3E`, and
/// resume with `a5` set to the predecessor link address in `a0`.
#[allow(non_snake_case)]
#[allow(dead_code)]
pub(crate) fn deallocate_object_slot_body_loc_EB0(
    active_slots: &[LinkedObjectSlotInit],
    target_slot_addr: u32,
    first_free_slot_addr: u32,
    number_objects: u16,
    gfx_child_slot_addrs: [Option<u32>; 3],
) -> Option<DeallocatedObjectSlotLocEB0> {
    let target_index = active_slots
        .iter()
        .position(|slot| slot.slot_addr == target_slot_addr)?;
    let predecessor = deallocate_object_slot_predecessor_loc_EA2(active_slots, target_slot_addr)?;
    let mut slots = active_slots.to_vec();
    let target_next_slot_addr = slots[target_index].next_slot_addr;

    if predecessor.predecessor_link_addr != 0xFFFF_F842 {
        let predecessor_slot = slots
            .iter_mut()
            .find(|slot| slot.slot_addr == predecessor.predecessor_link_addr)?;
        predecessor_slot.next_slot_addr = target_next_slot_addr;
    }

    slots[target_index].next_slot_addr = first_free_slot_addr;

    Some(DeallocatedObjectSlotLocEB0 {
        first_active_slot_addr: if predecessor.predecessor_link_addr == 0xFFFF_F842 {
            target_next_slot_addr
        } else {
            active_slots.first().map_or(0, |slot| slot.slot_addr)
        },
        first_free_slot_addr: target_slot_addr,
        number_objects: number_objects.wrapping_sub(1),
        current_object_addr_after_return: predecessor.predecessor_link_addr,
        deallocated_gfx_object_addrs: gfx_child_slot_addrs.into_iter().flatten().collect(),
        slots,
    })
}

#[allow(non_snake_case)]
#[allow(dead_code)]
pub(crate) fn spawn_block_debris_loc_2928(
    queue: &mut ScratchQueueSub28FC,
    x_pos_px: u16,
    y_pos_px: u16,
    direction: u16,
) {
    // ASM loc_2928 (kid.asm:4874): the alternate entry path sets byte 1 with
    // `st 1(a0)` before branching into the shared debris builder at `loc_293A`.
    spawn_block_debris_common_sub_292E(queue, x_pos_px, y_pos_px, direction, 0xFF);
}

#[allow(non_snake_case)]
#[allow(dead_code)]
pub(crate) fn spawn_block_debris_sub_292E(
    queue: &mut ScratchQueueSub28FC,
    x_pos_px: u16,
    y_pos_px: u16,
    direction: u16,
) {
    // ASM sub_292E (kid.asm:4881): the standard entry path clears byte 1 with
    // `sf 1(a0)` before branching into the shared debris builder at `loc_293A`.
    spawn_block_debris_common_sub_292E(queue, x_pos_px, y_pos_px, direction, 0x00);
}

#[allow(non_snake_case)]
fn spawn_block_debris_common_sub_292E(
    queue: &mut ScratchQueueSub28FC,
    x_pos_px: u16,
    y_pos_px: u16,
    direction: u16,
    variant_flag: u8,
) {
    // ASM loc_293A (kid.asm:4887): seed the shared x/y offset tables from
    // `unk_29E8`/`unk_29F2`, then select one of four signed velocity-table
    // pairs from `unk_29FC`..`unk_2A42` based on `d6`, with any value other
    // than 0/1/2 falling through to the final `unk_2A24`/`unk_2A2E` pair.
    // ASM loc_2988 (kid.asm:4916): the `cmpi.w #2,d6` case selects
    // `unk_29FC`/`unk_2A06`, yielding the straight-up spread where every
    // debris piece keeps the same `x` velocity.
    // ASM loc_297A (kid.asm:4910): the `tst.w d6` zero case swaps in
    // `unk_2A10`/`unk_2A1A`, giving the steep leftward spread used by the
    // first debris direction before control falls into `loc_2994`.
    // ASM loc_296C (kid.asm:4904): the `d6 == 1` branch swaps in
    // `unk_2A38`/`unk_2A42`, which produces the shallow-left spread used by
    // the second debris direction.
    // ASM loc_2994 (kid.asm:4920): copy the selected `a6/a2` velocity words
    // plus the shared `unk_29E8`/`unk_29F2` position offsets into five
    // consecutive `$10`-byte debris records because `moveq #4,d7` makes the
    // trailing `dbf d7,loc_2994` run inclusively from 4 down to -1.
    // ASM loc_29DE (kid.asm:4942): after the inclusive debris copy loop,
    // decrement `$FFFFFAA0`; if it remains non-zero, advance `$FFFFFA9C` by
    // `$52` bytes to the next scratch slot instead of rewinding.
    // ASM sub_292E epilogue (kid.asm:4938-4941): when the decrement reaches
    // zero, rewind `$FFFFFA9C` to `$FFFFF01E` and reload `$FFFFFAA0` with `#6`.
    let slot_index = queue.next_slot;
    let slot = &mut queue.slots[slot_index];
    slot.fill(0);
    slot[SUB_292E_SLOT_ACTIVE_OFFSET] = 0xFF;
    slot[SUB_292E_SLOT_FLAG_OFFSET] = variant_flag;

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
        // ASM sub_2A4C -> loc_2A56 (kid.asm:5057): `move.w a0,a1; tst.b (a0);
        // beq.w loc_2BA6` — skip slot if `slot[0]` is zero (inactive).
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

            // ASM sub_2A4C -> loc_2A56 (kid.asm:5077-5086): lead fragment bounds
            // check: `cmpi.w #$FF9C,d6; blt.w loc_2B9C; cmpi.w #$E0,d6; bge.w
            // loc_2B9C; ...; bmi.w loc_2B9C; cmpi.w #$140,d5; bge.w loc_2B9C`.
            if piece_index == 0
                && (!(-100..0x00E0).contains(&relative_y) || !(0..0x0140).contains(&relative_x))
            {
                slot_active = false;
                break;
            }

            // ASM sub_2A4C: tail fragment off-screen check (not a named loc_ label;
            // computed inline after sprite emission around line 5132-5158).
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

        // ASM sub_2A4C -> loc_2B9C (kid.asm:5166-5167): `sf (a0)` — clear
        // slot active byte when fragment leaves bounds.
        // ASM sub_2A4C -> loc_2B9E (kid.asm:5169-5171): `move.l a2,
        // (Addr_NextSpriteSlot).w; move.b d0,(Number_Sprites).w` — update
        // sprite table pointer and count after emitting sprites.
        // ASM sub_2A4C -> loc_2BA6 (kid.asm:5173-5177): decrement counter,
        // advance to next slot, loop back to loc_2A56.
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
pub(crate) fn initialise_palette_fade_out_loc_28A6(
    palette_buffer: [u16; 0x40],
    fill_colour: u16,
) -> PaletteFadeOutInitLoc28A6 {
    // ASM Pal_FadeOut / loc_28A6 / loc_28B8 (kid.asm:4801): seed `d0=#$1F`
    // and copy 32 longwords from `Palette_Buffer` into `Palette_Buffer_2`,
    // seed `d0=#$3F` and flood-fill all 64 palette words in
    // `Palette_Buffer_3` with `($FFFFFBCC).w`, then set `$FFFFF876` to
    // `$0100` before the fade loop at `loc_28DE` begins stepping toward
    // black.
    PaletteFadeOutInitLoc28A6 {
        palette_buffer_2: palette_buffer,
        palette_buffer_3: [fill_colour; 0x40],
        fade_progress: 0x0100,
    }
}

#[allow(non_snake_case)]
#[allow(dead_code)]
pub(crate) fn step_palette_fade_out_loc_28C6(
    fade_progress: u16,
    pending_palette_0_mask: u32,
    pending_palette_1_mask: u32,
) -> PaletteFadeOutStepLoc28C6 {
    // ASM loc_28C6 / loc_28DE / loc_28EA (kid.asm:4827): after the VBlank
    // refresh and palette upload, subtract `$10` from `$FFFFF876`.
    // Non-negative results reseed both pending palette masks with `-1` and
    // loop for another fade frame. Negative results branch to `loc_28EA`,
    // which performs one final VBlank refresh and returns without rewriting
    // either pending palette mask.
    let fade_progress = fade_progress.wrapping_sub(0x10);
    if (fade_progress as i16) < 0 {
        return PaletteFadeOutStepLoc28C6 {
            fade_progress,
            pending_palette_0_mask,
            pending_palette_1_mask,
            continue_fading: false,
        };
    }

    PaletteFadeOutStepLoc28C6 {
        fade_progress,
        pending_palette_0_mask: u32::MAX,
        pending_palette_1_mask: u32::MAX,
        continue_fading: true,
    }
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
pub(crate) fn blend_palette_entry_loc_1538(
    palette_word_2: u16,
    palette_word_3: u16,
    fade_curve: &[u8; 15],
) -> u16 {
    // ASM loc_1538 (kid.asm:2639): one DBF iteration masks both palette
    // words with `$0EEE`, blends the low, middle, and high colour channels in
    // sequence via the signed 15-byte fade curve at `7(a3,d?.w)`, then packs
    // the three nibbles back into one palette word with the same rotate/OR
    // sequence the ROM uses.
    let mut assembled_word = 0_u16;
    let mut src_word = palette_word_2 & 0x0EEE;
    let mut dst_word = palette_word_3 & 0x0EEE;

    for _ in 0..2 {
        let src_channel = (src_word & 0x000F) as u8;
        let dst_channel = (dst_word & 0x000F) as u8;
        let half_delta = ((i16::from(dst_channel) - i16::from(src_channel)) >> 1) + 7;
        let fade_step = fade_curve[usize::try_from(half_delta)
            .expect("loc_1538 channel deltas should stay inside the 15-byte fade curve")];
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
        .expect("loc_1538 top channel deltas should stay inside the 15-byte fade curve")];
    let blended_channel = src_channel.wrapping_sub(fade_step);

    assembled_word = assembled_word.rotate_right(4);
    assembled_word |= u16::from(blended_channel);
    assembled_word.rotate_right(8)
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
            // ASM loc_158E (kid.asm:2679): clear bits do not exit the DBF
            // loop; they only skip the `(a2)+` write, advance `a2` by one
            // palette word, and continue scanning later mask bits.
            continue;
        }

        palette_buffer[index] = blend_palette_entry_loc_1538(
            palette_buffer_2[index],
            palette_buffer_3[index],
            fade_curve,
        );
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

    // ASM loc_1518 (kid.asm:2618): after the optional first-half blend, repoint
    // `a0/a1/a2` at `Palette_Buffer_{2,3}/Palette_Buffer + $40`, load the
    // second pending mask from `$FFFFF88C`, and return immediately if it is
    // zero.
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
    // ASM loc_1534 (kid.asm:2628): once the second-half mask is non-zero,
    // clear `$FFFFF88C` before falling straight into sub_1536 for the actual
    // blend.
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
pub(crate) fn should_process_fg_update_queue_cell_loc_15D2(
    next_slot: u32,
    fg_plane_update_queue_cell_addr: u16,
) -> bool {
    // ASM loc_15D2 (kid.asm:2717): the cell-queue half of sub_1596 only runs
    // when `FGUpdateQueueCell_NextSlot` is non-zero and its low word differs
    // from `FGPlane_UpdateQueueCell`; zero or the queue-base sentinel both
    // branch straight to `loc_1686`.
    next_slot != 0 && (next_slot as u16) != fg_plane_update_queue_cell_addr
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct FgUpdateQueueResetLoc1680 {
    pub(crate) transient_cell_next_slot: u32,
    pub(crate) final_custom_next_slot: u32,
    pub(crate) final_cell_next_slot: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct BackgroundScrollUpperPassLoc2FFE8 {
    pub(crate) scroll_data_camera_x: i16,
    pub(crate) original_camera_y: i16,
    pub(crate) stepped_camera_y: i16,
    pub(crate) scroll_data_camera_y: i16,
    pub(crate) loop_iterations: u16,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct BackgroundScrollUpperPassResumeLoc30014 {
    pub(crate) restored_camera_y: i16,
    pub(crate) loop_iterations: u16,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct BackgroundScrollLowerPassSetupLoc3001E {
    pub(crate) original_camera_y: i16,
    pub(crate) stepped_camera_y: i16,
    pub(crate) shift_data_camera_y: i16,
    pub(crate) loop_iterations: u16,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct BackgroundScrollLowerPassResumeLoc30054 {
    pub(crate) original_camera_y: i16,
    pub(crate) stepped_camera_y: i16,
    pub(crate) shift_data_camera_y: i16,
    pub(crate) loop_iterations: u16,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum BackgroundScrollLowerPassRendererLoc30066 {
    PiecesSub307D8,
    EnigmaSub303BA,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct BackgroundScrollLowerPassLoopLoc30066 {
    pub(crate) renderer: BackgroundScrollLowerPassRendererLoc30066,
    pub(crate) final_camera_y: i16,
    pub(crate) final_scroll_data_camera_y: i16,
    pub(crate) final_shift_data_camera_y: i16,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct TransferScrollDataVsramSetupLoc300BA {
    pub(crate) first_vsram_word: i16,
    pub(crate) second_vsram_word: i16,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct TransferScrollDataPreludeLoc3038A {
    pub(crate) next_time_frames: u16,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum TransferScrollDataRendererLoc3039C {
    Return303B8,
    PiecesSub30744And307D8,
    EnigmaSub303BA,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum BackgroundScrollPieceRowDispatchLoc3079ABranch {
    Return307B0,
    ContinueLoc3079A,
    Branch307B2,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct BackgroundScrollPieceRowDispatchLoc3079A {
    pub(crate) row_selector_byte: u8,
    pub(crate) background_scroll_longword: i32,
    pub(crate) branch: BackgroundScrollPieceRowDispatchLoc3079ABranch,
    pub(crate) next_layout_row_offset: i16,
    pub(crate) next_vdp_row_seed: u16,
    pub(crate) next_dbf_counter: u16,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct BackgroundScrollPieceRowUploadLoc307B2 {
    pub(crate) vdp_control_word: u16,
    pub(crate) vdp_autoincrement_word: u16,
    pub(crate) tile_byte_offset: usize,
    pub(crate) tile_mapping_word: u16,
    pub(crate) next_layout_row_offset: i16,
    pub(crate) next_vdp_row_seed: u16,
    pub(crate) next_dbf_counter: u16,
    pub(crate) loops_to_loc_3079_a: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct BackgroundScrollPieceRefreshSeedLoc3081A {
    pub(crate) row_sequence_offset: usize,
    pub(crate) row_sequence_entry: u8,
    pub(crate) coarse_scroll_word: u16,
    pub(crate) reduced_scroll_word: i16,
    pub(crate) wrap_subtraction_count: u16,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct BackgroundScrollPieceRefreshWrapLoc30836 {
    pub(crate) reduced_scroll_word: i16,
    pub(crate) wrap_subtraction_count: u16,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct BackgroundScrollPieceRefreshWindowSplitLoc3086A {
    pub(crate) wrapped_row_window_end: i16,
    pub(crate) row_window_wraps: bool,
    pub(crate) tile_column_seed: u16,
    pub(crate) wrapped_tile_window_span: u16,
    pub(crate) tile_window_wraps: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct BackgroundScrollPieceRefreshVdpSeedLoc30884 {
    pub(crate) tile_column_offset: u16,
    pub(crate) vdp_control_word: u16,
    pub(crate) vdp_autoincrement_word: u16,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct BackgroundScrollPieceRefreshLinearCopyLoc308A6 {
    pub(crate) copied_mapping_words: Vec<u16>,
    pub(crate) bytes_consumed: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct BackgroundScrollPieceRefreshWrapCounterSetupLoc308B6 {
    pub(crate) wrapped_row_window_end: i16,
    pub(crate) first_pass_dbf_counter: u16,
    pub(crate) second_pass_dbf_counter: u16,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct BackgroundScrollPieceRefreshTileWrapCounterSetupLoc308DE {
    pub(crate) wrapped_tile_window_span: u16,
    pub(crate) first_pass_dbf_counter: u16,
    pub(crate) second_pass_dbf_counter: u16,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct BackgroundScrollPieceRefreshTileWrappedCopyLoc308E8 {
    pub(crate) first_pass_mapping_words: Vec<u16>,
    pub(crate) seam_vdp_control_word: u16,
    pub(crate) seam_vdp_autoincrement_word: u16,
    pub(crate) second_pass_mapping_words: Vec<u16>,
    pub(crate) bytes_consumed: usize,
    pub(crate) final_layout_row_offset: i16,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum BackgroundScrollPieceRefreshDualWrapBranchLoc30904 {
    SharedSplitLoc30916,
    RowWrapFirstLoc30936,
    TileWrapFirstLoc30978,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct BackgroundScrollPieceRefreshRowWrapFirstCopyLoc30942 {
    pub(crate) mapping_words: Vec<u16>,
    pub(crate) wrapped_layout_row_offset: i16,
    pub(crate) bytes_consumed: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct BackgroundScrollPieceRefreshRowWrapSecondCopyLoc30954 {
    pub(crate) mapping_words: Vec<u16>,
    pub(crate) seam_vdp_control_word: u16,
    pub(crate) seam_vdp_autoincrement_word: u16,
    pub(crate) next_layout_row_offset: i16,
    pub(crate) bytes_consumed: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct BackgroundScrollPieceRefreshRowWrapFinalCopyLoc30968 {
    pub(crate) mapping_words: Vec<u16>,
    pub(crate) next_layout_row_offset: i16,
    pub(crate) bytes_consumed: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct BackgroundScrollPieceRefreshTileWrapFirstCopyLoc30988 {
    pub(crate) mapping_words: Vec<u16>,
    pub(crate) seam_vdp_control_word: u16,
    pub(crate) seam_vdp_autoincrement_word: u16,
    pub(crate) next_layout_row_offset: i16,
    pub(crate) bytes_consumed: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct BackgroundScrollPieceRefreshTileWrapWrappedCopyLoc3099E {
    pub(crate) first_pass_mapping_words: Vec<u16>,
    pub(crate) second_pass_mapping_words: Vec<u16>,
    pub(crate) bytes_consumed: usize,
    pub(crate) final_layout_row_offset: i16,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct BackgroundScrollPieceRefreshWrappedCopyLoc308C4 {
    pub(crate) first_pass_mapping_words: Vec<u16>,
    pub(crate) second_pass_mapping_words: Vec<u16>,
    pub(crate) bytes_consumed: usize,
    pub(crate) final_layout_row_offset: i16,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct TransferScrollDataDispatchLoc3039C {
    pub(crate) computes_shift_data: bool,
    pub(crate) computes_shift_layers: bool,
    pub(crate) renderer: TransferScrollDataRendererLoc3039C,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum BackgroundScrollEnigmaRowSourceLoc303D4 {
    LowerDecompressionBuffer,
    UpperDecompressionBuffer,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct BackgroundScrollEnigmaRowTableLookupLoc303EA {
    pub(crate) map_header_byte_offset: usize,
    pub(crate) background_row_index: u8,
    pub(crate) row_mapping_byte_offset: usize,
    pub(crate) vdp_control_word: u16,
    pub(crate) vdp_autoincrement_word: u16,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct BackgroundScrollEnigmaPrimaryCopySetupLoc30426 {
    pub(crate) longword_dbf_counter: u16,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct BackgroundScrollShiftDataSlotLoc30452 {
    pub(crate) next_phase_word: u16,
    pub(crate) updated_scroll_word: u16,
    pub(crate) cached_previous_scroll_word: u16,
    pub(crate) previous_tile_column: u16,
    pub(crate) updated_tile_column: u16,
    pub(crate) branches_to_loc_30476: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct BackgroundScrollShiftDataSecondPassSlotLoc3048C {
    pub(crate) updated_scroll_word: u16,
    pub(crate) cached_previous_scroll_word: u16,
    pub(crate) branches_to_loc_304_a4: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct BackgroundScrollShiftDataSecondPassAdvanceLoc304A8 {
    pub(crate) next_slot_offset_bytes: u16,
    pub(crate) next_phase_accumulator: u32,
    pub(crate) next_dbf_counter: u16,
    pub(crate) loops_to_loc_3048_c: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct BackgroundScrollShiftDataWrapLoopLoc304C2 {
    pub(crate) seeded_phase_row: i16,
    pub(crate) reduced_phase_row: i16,
    pub(crate) subtraction_count: u16,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct BackgroundScrollShiftDataWrapStoreLoc304CA {
    pub(crate) reduced_phase_row_word: u16,
    pub(crate) strip_table_word_offset: u16,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct BackgroundScrollShiftDataWrapLoopLoc304D8 {
    pub(crate) seeded_phase_row: i16,
    pub(crate) reduced_phase_row: i16,
    pub(crate) subtraction_count: u16,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct BackgroundScrollShiftDataWrapLoopBodyLoc304E0 {
    pub(crate) reduced_phase_row: i16,
    pub(crate) subtraction_count: u16,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct BackgroundScrollEnigmaRowTransferLoc303D4 {
    pub(crate) map_header_byte_offset: usize,
    pub(crate) background_row_index: u8,
    pub(crate) row_mapping_byte_offset: usize,
    pub(crate) vdp_control_word: u16,
    pub(crate) vdp_autoincrement_word: u16,
    pub(crate) source: BackgroundScrollEnigmaRowSourceLoc303D4,
    pub(crate) primary_decompression_byte_offset: usize,
    pub(crate) primary_longword_dbf_counter: u16,
    pub(crate) secondary_decompression_byte_offset: usize,
    pub(crate) longword_rows: u16,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct TransferScrollDataDmaTailLoc300D2 {
    pub(crate) second_vsram_word: i16,
    pub(crate) dma_source_addr: u32,
    pub(crate) dma_vram_word_addr: u16,
    pub(crate) dma_word_count: u16,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct TransferScrollDataStormLoc30110 {
    pub(crate) plane_address_word: u16,
    pub(crate) h_scroll_word_pairs: u16,
    pub(crate) first_h_scroll_word: i16,
    pub(crate) second_h_scroll_word: i16,
    pub(crate) first_vsram_word: i16,
    pub(crate) second_vsram_word: i16,
    pub(crate) weather_mode_byte: u8,
    pub(crate) next_storm_scroll_offset: i16,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct TransferScrollDataStormLoopLoc30130 {
    pub(crate) h_scroll_word_pairs: u16,
    pub(crate) first_h_scroll_word: i16,
    pub(crate) second_h_scroll_word: i16,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct TransferScrollDataStormSetupLoc30158 {
    pub(crate) h_scroll_word_pairs: u16,
    pub(crate) first_h_scroll_word: i16,
    pub(crate) second_h_scroll_word: i16,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct TransferScrollDataStormLoopLoc30172 {
    pub(crate) h_scroll_word_pairs: u16,
    pub(crate) first_h_scroll_word: i16,
    pub(crate) second_h_scroll_word: i16,
    pub(crate) first_vsram_word: i16,
    pub(crate) second_vsram_word: i16,
    pub(crate) next_storm_scroll_offset: i16,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct BackgroundScrollShiftLayersNoScrollFillLoc30502 {
    pub(crate) repeated_scroll_longword: u32,
    pub(crate) longword_count: u16,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct BackgroundScrollShiftLayersSetupLoc3050A {
    pub(crate) map_order_entry: u8,
    pub(crate) background_scroll_table_index: usize,
    pub(crate) row_sequence_byte_offset: usize,
    pub(crate) block_dbf_counter: u16,
    pub(crate) negated_camera_x_longword: u32,
    pub(crate) quarter_row_phase: u16,
    pub(crate) branches_to_loc_3056_a: bool,
    pub(crate) leading_partial_longword_count: Option<u16>,
    pub(crate) trailing_partial_longword_count: Option<u16>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct BackgroundScrollShiftLayersLeadingPartialFillLoc30562 {
    pub(crate) repeated_scroll_longword: u32,
    pub(crate) longword_count: u16,
    pub(crate) next_block_dbf_counter: u16,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct BackgroundScrollShiftLayersTrailingPartialFillLoc30592 {
    pub(crate) repeated_scroll_longword: u32,
    pub(crate) longword_count: u16,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum BackgroundScrollThemeDispatchLoc30598 {
    Return,
    ApplyForestWaterRipple,
    ApplyDesertHeatRipple,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct BackgroundScrollForestWaterRippleVisibleBandLoc305D4 {
    pub(crate) start_scanline: u16,
    pub(crate) ripple_data_byte_offset: u16,
    pub(crate) affected_scanline_count: u16,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct BackgroundScrollForestWaterRippleSetupLoc305E6 {
    pub(crate) start_scanline: u16,
    pub(crate) target_word_offset_bytes: usize,
    pub(crate) base_scroll_word: i16,
    pub(crate) ripple_dbf_counter: u16,
    pub(crate) ripple_data_byte_offset: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct BackgroundScrollDesertHeatRippleVisibleBandLoc306C2 {
    pub(crate) start_scanline: u16,
    pub(crate) ripple_data_byte_offset: u16,
    pub(crate) affected_scanline_count: u16,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct BackgroundScrollDesertHeatRippleSetupLoc306D4 {
    pub(crate) start_scanline: u16,
    pub(crate) target_word_offset_bytes: usize,
    pub(crate) base_scroll_word: i16,
    pub(crate) ripple_dbf_counter: u16,
    pub(crate) ripple_data_byte_offset: usize,
}

#[allow(non_snake_case)]
#[allow(dead_code)]
pub(crate) fn reset_fg_update_queues_loc_1680(
    fg_plane_update_queue_custom_addr: u32,
    fg_plane_update_queue_cell_addr: u32,
) -> FgUpdateQueueResetLoc1680 {
    // ASM loc_1680 / loc_1686 (kid.asm:2791): the negative-sentinel exit from
    // the cell-drain loop first stores `d0=0` into `FGUpdateQueueCell_NextSlot`
    // before the shared epilogue repoints both next-slot longs at
    // `FGPlane_UpdateQueueCustom` and `FGPlane_UpdateQueueCell`.
    FgUpdateQueueResetLoc1680 {
        transient_cell_next_slot: 0,
        final_custom_next_slot: fg_plane_update_queue_custom_addr,
        final_cell_next_slot: fg_plane_update_queue_cell_addr,
    }
}

#[allow(non_snake_case)]
#[allow(dead_code)]
pub(crate) fn background_scroll_upper_pass_loc_2FFE8(
    camera_x_pos: i16,
    camera_y_pos: i16,
    level_height_pixels: i16,
) -> BackgroundScrollUpperPassLoc2FFE8 {
    // ASM loc_2FFE8 (kid.asm:33592): snapshot the current camera words into
    // `$FFFFF82C` and `d7`, subtract `$E0` from `Camera_Y_pos`, then write
    // `Camera_Y_pos - 8` to `$FFFFF82E` before seeding `d6 = #$1C`.
    // ASM loc_2FFE8 tail (kid.asm:33600-33606): after biasing the saved `d7`
    // by `$E0`, clamp the later `dbf d6,loc_30016` pass count to 28 rows when
    // the bottom sample stays inside the level, otherwise decrement to 27 rows
    // before branching to `loc_30014`.
    let stepped_camera_y = camera_y_pos.wrapping_sub(0x00E0);
    let mut loop_iterations: u16 = 0x001C;
    let lower_edge_sample = camera_y_pos.wrapping_add(0x00E0);
    if lower_edge_sample >= level_height_pixels {
        loop_iterations = loop_iterations.wrapping_sub(1);
    }

    BackgroundScrollUpperPassLoc2FFE8 {
        scroll_data_camera_x: camera_x_pos,
        original_camera_y: camera_y_pos,
        stepped_camera_y,
        scroll_data_camera_y: stepped_camera_y.wrapping_sub(8),
        loop_iterations,
    }
}

#[allow(non_snake_case)]
#[allow(dead_code)]
pub(crate) fn background_scroll_upper_pass_resume_loc_30014(
    prologue: BackgroundScrollUpperPassLoc2FFE8,
) -> BackgroundScrollUpperPassResumeLoc30014 {
    // ASM loc_30014 (kid.asm:33607): pop the saved longword from the stack so
    // `d7` once again holds the original camera Y before control falls through
    // to `loc_30016`.
    BackgroundScrollUpperPassResumeLoc30014 {
        restored_camera_y: prologue.original_camera_y,
        loop_iterations: prologue.loop_iterations,
    }
}

#[allow(non_snake_case)]
#[allow(dead_code)]
pub(crate) fn background_scroll_lower_pass_setup_loc_3001E(
    resume: BackgroundScrollUpperPassResumeLoc30014,
    level_height_pixels: i16,
) -> BackgroundScrollLowerPassSetupLoc3001E {
    // ASM loc_3001E (kid.asm:33614): once the upper `dbf d6,loc_30016` pass
    // finishes, restore `d6/d7`, then rebuild the lower-pass state from the
    // original camera word by storing `d7-$380` back into `Camera_Y_pos`,
    // `d7-$3A0` into `$FFFFFAA8`, and re-seeding `d6=#1C` with the same
    // bottom-edge clamp used for the second pass setup.
    let stepped_camera_y = resume.restored_camera_y.wrapping_sub(0x0380);
    let shift_data_camera_y = stepped_camera_y.wrapping_sub(0x0020);
    let mut loop_iterations: u16 = 0x001C;
    let lower_edge_sample = resume.restored_camera_y.wrapping_add(0x00E0);
    if lower_edge_sample >= level_height_pixels {
        loop_iterations = loop_iterations.wrapping_sub(1);
    }

    BackgroundScrollLowerPassSetupLoc3001E {
        original_camera_y: resume.restored_camera_y,
        stepped_camera_y,
        shift_data_camera_y,
        loop_iterations,
    }
}

#[allow(non_snake_case)]
#[allow(dead_code)]
pub(crate) fn background_scroll_lower_pass_resume_loc_30054(
    setup: BackgroundScrollLowerPassSetupLoc3001E,
) -> BackgroundScrollLowerPassResumeLoc30054 {
    // ASM loc_30054 (kid.asm:33633): after restoring the saved `d7`, the
    // wrapped `$FC80` seed branches straight onward; every other case subtracts
    // `$40` from `Camera_Y_pos` and adds 2 to `d6` before the lower row loop.
    if setup.stepped_camera_y == -0x0380 {
        return BackgroundScrollLowerPassResumeLoc30054 {
            original_camera_y: setup.original_camera_y,
            stepped_camera_y: setup.stepped_camera_y,
            shift_data_camera_y: setup.shift_data_camera_y,
            loop_iterations: setup.loop_iterations,
        };
    }

    BackgroundScrollLowerPassResumeLoc30054 {
        original_camera_y: setup.original_camera_y,
        stepped_camera_y: setup.stepped_camera_y.wrapping_sub(0x0040),
        shift_data_camera_y: setup.shift_data_camera_y,
        loop_iterations: setup.loop_iterations.wrapping_add(2),
    }
}

#[allow(non_snake_case)]
#[allow(dead_code)]
pub(crate) fn background_scroll_lower_pass_loop_loc_30066(
    resume: BackgroundScrollLowerPassResumeLoc30054,
    background_format: u8,
) -> BackgroundScrollLowerPassLoopLoc30066 {
    // ASM loc_30066 (kid.asm:33640): select `sub_307D8` when
    // `Background_format` is zero, otherwise branch to the enigma path.
    // ASM loc_30076 (kid.asm:33648): the non-zero `Background_format` branch
    // runs `bsr.w sub_303BA` before falling through to the shared epilogue.
    // ASM loc_3007A (kid.asm:33651): the shared lower-pass epilogue resumes
    // after either renderer and restores `Camera_Y_pos` plus `$FFFFF82E` from
    // the saved `d7` before testing whether `d7` is zero.
    // ASM loc_30090 (kid.asm:33659): the tail writes the saved `d7` back to
    // both camera words and returns; when the preceding zero check succeeds,
    // `$FFFFFAA8` has already been cleared before this final restore.
    let renderer = if background_format == 0 {
        BackgroundScrollLowerPassRendererLoc30066::PiecesSub307D8
    } else {
        BackgroundScrollLowerPassRendererLoc30066::EnigmaSub303BA
    };

    let final_shift_data_camera_y = if resume.original_camera_y == 0 {
        0
    } else {
        resume.shift_data_camera_y
    };

    BackgroundScrollLowerPassLoopLoc30066 {
        renderer,
        final_camera_y: resume.original_camera_y,
        final_scroll_data_camera_y: resume.original_camera_y,
        final_shift_data_camera_y,
    }
}

#[allow(non_snake_case)]
#[allow(dead_code)]
pub(crate) fn transfer_scroll_data_vsram_setup_loc_300BA(
    camera_y_pos: i16,
    background_no_scroll_flag: bool,
) -> TransferScrollDataVsramSetupLoc300BA {
    // ASM loc_300BA (kid.asm:33677): write `(Camera_Y_pos).w` into the first
    // VSRAM slot, arithmetic-free shift it right by 2 for the parallax slot,
    // then zero the shared-tail value when `(Background_NoScrollFlag).w` is
    // non-zero.
    let mut second_vsram_word = ((camera_y_pos as u16) >> 2) as i16;
    if background_no_scroll_flag {
        second_vsram_word = 0;
    }

    TransferScrollDataVsramSetupLoc300BA {
        first_vsram_word: camera_y_pos,
        second_vsram_word,
    }
}

#[allow(non_snake_case)]
#[allow(dead_code)]
pub(crate) fn transfer_scroll_data_prelude_loc_3038A(
    level_special_effects: u16,
    game_mode: u16,
    time_frames: u16,
) -> Option<TransferScrollDataPreludeLoc3038A> {
    // ASM loc_3038A (kid.asm:33983): `Level_Special_Effects == 4` returns
    // immediately. Otherwise the routine falls through to `loc_3039C`, only
    // incrementing `Time_Frames` first when `Game_Mode` is non-zero.
    if level_special_effects == 4 {
        return None;
    }

    let next_time_frames = if game_mode == 0 {
        time_frames
    } else {
        time_frames.wrapping_add(1)
    };

    Some(TransferScrollDataPreludeLoc3038A { next_time_frames })
}

#[allow(non_snake_case)]
#[allow(dead_code)]
pub(crate) fn transfer_scroll_data_dispatch_loc_3039C(
    background_upload_busy_flag: u8,
    background_format: u8,
) -> TransferScrollDataDispatchLoc3039C {
    // ASM loc_3039C (kid.asm:33990): this entry always runs
    // `BackgroundScroll_ComputeShiftData` and `BackgroundScroll_ComputeShiftLayers`
    // first. A non-zero `$FFFFFB49` returns immediately; otherwise
    // `Background_format == 0` calls the piece-background pair
    // `sub_30744`/`sub_307D8`, and non-zero formats branch to `sub_303BA`.
    let renderer = if background_upload_busy_flag != 0 {
        TransferScrollDataRendererLoc3039C::Return303B8
    } else if background_format == 0 {
        TransferScrollDataRendererLoc3039C::PiecesSub30744And307D8
    } else {
        TransferScrollDataRendererLoc3039C::EnigmaSub303BA
    };

    TransferScrollDataDispatchLoc3039C {
        computes_shift_data: true,
        computes_shift_layers: true,
        renderer,
    }
}

#[allow(non_snake_case)]
#[allow(dead_code)]
pub(crate) fn background_scroll_piece_row_dispatch_loc_3079A(
    row_selector_byte: u8,
    background_scroll_longword: i32,
    layout_row_offset: i16,
    background_width: i16,
    vdp_row_seed: u16,
    dbf_counter: u16,
) -> BackgroundScrollPieceRowDispatchLoc3079A {
    // ASM loc_3079A (kid.asm:34590): each pass reads the next row-selector
    // byte from `(a2)+` and returns immediately through `return_307B0` when it
    // is `$FF`. Otherwise the routine loads the matching `Horiz_Scroll_Data`
    // longword and only branches to `loc_307B2` when that signed longword is
    // non-negative; negative entries advance `a0` by `Background_width`, add
    // `$80` to the row seed in `d2`, and consume one inclusive `dbf d3` step
    // before retrying `loc_3079A`.
    if row_selector_byte == 0xFF {
        return BackgroundScrollPieceRowDispatchLoc3079A {
            row_selector_byte,
            background_scroll_longword,
            branch: BackgroundScrollPieceRowDispatchLoc3079ABranch::Return307B0,
            next_layout_row_offset: layout_row_offset,
            next_vdp_row_seed: vdp_row_seed,
            next_dbf_counter: dbf_counter,
        };
    }

    if background_scroll_longword >= 0 {
        return BackgroundScrollPieceRowDispatchLoc3079A {
            row_selector_byte,
            background_scroll_longword,
            branch: BackgroundScrollPieceRowDispatchLoc3079ABranch::Branch307B2,
            next_layout_row_offset: layout_row_offset,
            next_vdp_row_seed: vdp_row_seed,
            next_dbf_counter: dbf_counter,
        };
    }

    BackgroundScrollPieceRowDispatchLoc3079A {
        row_selector_byte,
        background_scroll_longword,
        branch: BackgroundScrollPieceRowDispatchLoc3079ABranch::ContinueLoc3079A,
        next_layout_row_offset: layout_row_offset.wrapping_add(background_width),
        next_vdp_row_seed: vdp_row_seed.wrapping_add(0x0080),
        next_dbf_counter: dbf_counter.wrapping_sub(1),
    }
}

#[allow(non_snake_case)]
#[allow(dead_code)]
pub(crate) fn background_scroll_piece_row_upload_loc_307B2(
    background_scroll_longword: u32,
    vdp_row_seed: u16,
    layout_row: &[u8],
    current_layout_row_offset: i16,
    block_mappings: &[u16],
    background_width: i16,
    dbf_counter: u16,
) -> Option<BackgroundScrollPieceRowUploadLoc307B2> {
    // ASM loc_307B2 (kid.asm:34604): this upload step masks `d2` with `$F80`,
    // ORs in `$6000`, merges that with the low scroll word from `d5`, and
    // writes the resulting plane-B control word plus autoincrement `3`. After
    // `swap d5`, the original high word becomes the `(a0,d5.w)` byte offset,
    // whose layout byte is doubled into the `(a3,d6.w)` mapping lookup. The
    // shared tail then advances `a0` by `Background_width`, adds `$80` to `d2`,
    // and consumes one inclusive `dbf d3` step before retrying `loc_3079A`.
    let tile_byte_offset = usize::from((background_scroll_longword >> 16) as u16);
    let tile_mapping_index = usize::from(*layout_row.get(tile_byte_offset)?);

    Some(BackgroundScrollPieceRowUploadLoc307B2 {
        vdp_control_word: (background_scroll_longword as u16) | ((vdp_row_seed & 0x0F80) | 0x6000),
        vdp_autoincrement_word: 3,
        tile_byte_offset,
        tile_mapping_word: *block_mappings.get(tile_mapping_index)?,
        next_layout_row_offset: current_layout_row_offset.wrapping_add(background_width),
        next_vdp_row_seed: vdp_row_seed.wrapping_add(0x0080),
        next_dbf_counter: dbf_counter.wrapping_sub(1),
        loops_to_loc_3079_a: dbf_counter != 0,
    })
}

#[allow(non_snake_case)]
#[allow(dead_code)]
pub(crate) fn background_scroll_piece_refresh_seed_loc_3081A(
    current_camera_row: i16,
    cached_previous_camera_row: i16,
    row_sequence: &[u8],
    horiz_scroll_bytes: &[u8],
    background_width: i16,
) -> Option<BackgroundScrollPieceRefreshSeedLoc3081A> {
    // ASM loc_3081A (kid.asm:34645): when the cached row differs from the
    // current `Camera_Y_pos>>5`, the preceding compare reaches this label with
    // either the current row (scrolling upward) or that row plus `$1C`
    // (scrolling downward). The label snapshots that row in `d3/d5`, reads the
    // next row-sequence byte from `(a2,d0.w)`, returns immediately on `$FF`,
    // then loads the coarse scroll word from `4(a1,d0.w)`, shifts it right by
    // 3, and keeps subtracting `Background_width` until the reduced word fits
    // before `loc_30840` seeds the later render state.
    let row_sequence_offset = if cached_previous_camera_row > current_camera_row {
        usize::from(current_camera_row as u16)
    } else {
        usize::from(current_camera_row.wrapping_add(0x001C) as u16)
    };
    let row_sequence_entry = *row_sequence.get(row_sequence_offset)?;
    if row_sequence_entry == 0xFF {
        return None;
    }

    let scroll_word_offset = 4usize.wrapping_add(usize::from(row_sequence_entry));
    let coarse_scroll_source = u16::from_be_bytes([
        *horiz_scroll_bytes.get(scroll_word_offset)?,
        *horiz_scroll_bytes.get(scroll_word_offset.wrapping_add(1))?,
    ]);
    let coarse_scroll_word = coarse_scroll_source >> 3;
    let BackgroundScrollPieceRefreshWrapLoc30836 {
        reduced_scroll_word,
        wrap_subtraction_count,
    } = background_scroll_piece_refresh_wrap_loc_30836(coarse_scroll_word, background_width);

    Some(BackgroundScrollPieceRefreshSeedLoc3081A {
        row_sequence_offset,
        row_sequence_entry,
        coarse_scroll_word,
        reduced_scroll_word,
        wrap_subtraction_count,
    })
}

#[allow(non_snake_case)]
#[allow(dead_code)]
pub(crate) fn background_scroll_piece_refresh_wrap_loc_30836(
    coarse_scroll_word: u16,
    background_width: i16,
) -> BackgroundScrollPieceRefreshWrapLoc30836 {
    // ASM loc_30836 (kid.asm:34656): once `loc_3081A` seeds `d0` from
    // `4(a1,d0.w)>>3`, this back-edge keeps subtracting `Background_width` from
    // that word until `cmp.w d0,a4` reports the reduced value is strictly below
    // the width, preserving the taken-wrap count along the way.
    let mut reduced_scroll_word = coarse_scroll_word as i16;
    let mut wrap_subtraction_count = 0_u16;
    while background_width <= reduced_scroll_word {
        reduced_scroll_word = reduced_scroll_word.wrapping_sub(background_width);
        wrap_subtraction_count = wrap_subtraction_count.wrapping_add(1);
    }

    BackgroundScrollPieceRefreshWrapLoc30836 {
        reduced_scroll_word,
        wrap_subtraction_count,
    }
}

#[allow(non_snake_case)]
#[allow(dead_code)]
pub(crate) fn background_scroll_piece_refresh_window_split_loc_3086A(
    reduced_scroll_word: i16,
    background_width: i16,
    coarse_scroll_word: u16,
) -> BackgroundScrollPieceRefreshWindowSplitLoc3086A {
    // ASM loc_3086A (kid.asm:34678): after `loc_30840` seeds `d7 = d0 + $28`,
    // this block wraps the row-window end back by `Background_width` when the
    // 40-column span crosses the background edge, then masks `d2` to 6 bits,
    // adds the same `$28` span, and wraps that local tile-window length by
    // `$40` when it crosses the 64-column phase boundary.
    let row_window_end = reduced_scroll_word.wrapping_add(0x0028);
    let (wrapped_row_window_end, row_window_wraps) = if background_width <= row_window_end {
        (row_window_end.wrapping_sub(background_width), true)
    } else {
        (row_window_end, false)
    };

    let tile_window_end = (coarse_scroll_word & 0x003F).wrapping_add(0x0028);
    let (wrapped_tile_window_span, tile_window_wraps) = if tile_window_end > 0x003F {
        (tile_window_end.wrapping_sub(0x0040), true)
    } else {
        (tile_window_end, false)
    };

    BackgroundScrollPieceRefreshWindowSplitLoc3086A {
        wrapped_row_window_end,
        row_window_wraps,
        tile_column_seed: coarse_scroll_word,
        wrapped_tile_window_span,
        tile_window_wraps,
    }
}

#[allow(non_snake_case)]
#[allow(dead_code)]
pub(crate) fn background_scroll_piece_refresh_vdp_seed_loc_30884(
    coarse_scroll_word: u16,
    row_sequence_offset: u16,
) -> BackgroundScrollPieceRefreshVdpSeedLoc30884 {
    // ASM loc_30884 (kid.asm:34689): this block masks `d0` down to the low
    // 6-bit tile column, doubles it into the word address, packs the row bits
    // from `d3 << 7` under the `$F80` mask, forces plane B with `$6000`, and
    // then writes the resulting control word followed by autoincrement `#3`.
    let tile_column_offset = (coarse_scroll_word & 0x003F) << 1;
    let vdp_control_word = tile_column_offset | 0x6000 | ((row_sequence_offset << 7) & 0x0F80);

    BackgroundScrollPieceRefreshVdpSeedLoc30884 {
        tile_column_offset,
        vdp_control_word,
        vdp_autoincrement_word: 3,
    }
}

#[allow(non_snake_case)]
#[allow(dead_code)]
pub(crate) fn background_scroll_piece_refresh_linear_copy_loc_308A6(
    layout_row: &[u8],
    block_mappings: &[u16],
) -> Option<BackgroundScrollPieceRefreshLinearCopyLoc308A6> {
    // ASM loc_308A6 (kid.asm:34704): when neither wrap flag was set, the
    // caller seeds `d4` with `$28` and this loop clears `d6`, reads one layout
    // byte from `(a0)+`, doubles it into the block-mapping word offset, copies
    // that word to `(a6)`, and repeats through the inclusive `dbf d4,loc_308A6`
    // count for 41 total transfers.
    let mut copied_mapping_words = Vec::with_capacity(0x29);
    for &mapping_index in layout_row.get(..0x29)? {
        copied_mapping_words.push(*block_mappings.get(usize::from(mapping_index))?);
    }

    Some(BackgroundScrollPieceRefreshLinearCopyLoc308A6 {
        copied_mapping_words,
        bytes_consumed: 0x29,
    })
}

#[allow(non_snake_case)]
#[allow(dead_code)]
pub(crate) fn background_scroll_piece_refresh_wrap_counter_setup_loc_308B6(
    wrapped_row_window_end: i16,
) -> BackgroundScrollPieceRefreshWrapCounterSetupLoc308B6 {
    // ASM loc_308B6 (kid.asm:34715): when the row window wraps but the tile
    // phase does not, this setup block snapshots the wrapped `d7` into `d4`,
    // seeds `d1` with `$28 - d7 - 1` for the first inclusive copy pass, and
    // loads `d2 = 1` so the shared tail runs exactly one wrapped retry.
    BackgroundScrollPieceRefreshWrapCounterSetupLoc308B6 {
        wrapped_row_window_end,
        first_pass_dbf_counter: 0x0028_u16
            .wrapping_sub(wrapped_row_window_end as u16)
            .wrapping_sub(1),
        second_pass_dbf_counter: 1,
    }
}

#[allow(non_snake_case)]
#[allow(dead_code)]
pub(crate) fn background_scroll_piece_refresh_wrapped_copy_loc_308C4(
    layout_row: &[u8],
    current_layout_row_offset: i16,
    block_mappings: &[u16],
    background_width: i16,
    wrapped_row_window_end: i16,
) -> Option<BackgroundScrollPieceRefreshWrappedCopyLoc308C4> {
    // ASM loc_308C4 (kid.asm:34724): the shared wrapped-copy tail consumes the
    // first inclusive `dbf d1` run from the current `(a0)+` position, rewinds
    // `a0` by `Background_width`, reloads `d1` from `d4`, and uses the `d2=1`
    // seed from loc_308B6 to execute exactly one second wrapped pass before
    // the final fall-through return.
    let counters =
        background_scroll_piece_refresh_wrap_counter_setup_loc_308B6(wrapped_row_window_end);
    let first_pass_count = usize::from(counters.first_pass_dbf_counter).wrapping_add(1);
    let first_pass_start = usize::from(current_layout_row_offset as u16);
    let first_pass_slice =
        layout_row.get(first_pass_start..first_pass_start.wrapping_add(first_pass_count))?;
    let first_pass_mapping_words = first_pass_slice
        .iter()
        .map(|&mapping_index| block_mappings.get(usize::from(mapping_index)).copied())
        .collect::<Option<Vec<_>>>()?;

    let wrapped_layout_row_offset = current_layout_row_offset
        .wrapping_add(first_pass_count as i16)
        .wrapping_sub(background_width);
    let second_pass_count = usize::from(counters.wrapped_row_window_end as u16).wrapping_add(1);
    let second_pass_start = usize::from(wrapped_layout_row_offset as u16);
    let second_pass_slice =
        layout_row.get(second_pass_start..second_pass_start.wrapping_add(second_pass_count))?;
    let second_pass_mapping_words = second_pass_slice
        .iter()
        .map(|&mapping_index| block_mappings.get(usize::from(mapping_index)).copied())
        .collect::<Option<Vec<_>>>()?;

    Some(BackgroundScrollPieceRefreshWrappedCopyLoc308C4 {
        first_pass_mapping_words,
        second_pass_mapping_words,
        bytes_consumed: first_pass_count.wrapping_add(second_pass_count),
        final_layout_row_offset: wrapped_layout_row_offset
            .wrapping_add(second_pass_count as i16)
            .wrapping_sub(background_width),
    })
}

#[allow(non_snake_case)]
#[allow(dead_code)]
pub(crate) fn background_scroll_piece_refresh_tile_wrap_counter_setup_loc_308DE(
    wrapped_tile_window_span: u16,
) -> BackgroundScrollPieceRefreshTileWrapCounterSetupLoc308DE {
    // ASM loc_308DE (kid.asm:34736): when the tile phase wraps but the row
    // window does not, this setup block snapshots wrapped `d6` into `d4`,
    // seeds `d1` with `$28 - d6 - 1` for the first inclusive copy pass, and
    // loads `d2 = 1` so loc_308E8 executes exactly one second pass after the
    // plane-B control-word reset.
    BackgroundScrollPieceRefreshTileWrapCounterSetupLoc308DE {
        wrapped_tile_window_span,
        first_pass_dbf_counter: 0x0028_u16
            .wrapping_sub(wrapped_tile_window_span)
            .wrapping_sub(1),
        second_pass_dbf_counter: 1,
    }
}

#[allow(non_snake_case)]
#[allow(dead_code)]
pub(crate) fn background_scroll_piece_refresh_tile_wrapped_copy_loc_308E8(
    layout_row: &[u8],
    current_layout_row_offset: i16,
    block_mappings: &[u16],
    wrapped_tile_window_span: u16,
    seam_vdp_control_word: u16,
) -> Option<BackgroundScrollPieceRefreshTileWrappedCopyLoc308E8> {
    // ASM loc_308E8 (kid.asm:34743): consume the first inclusive `dbf d1` span
    // from the current `(a0)+` position, then write `d3` and autoincrement
    // `#3` to `(a5)` before reloading `d1` from the saved wrapped tile span in
    // `d4` and emitting exactly one second pass without rewinding `a0`.
    let counters =
        background_scroll_piece_refresh_tile_wrap_counter_setup_loc_308DE(wrapped_tile_window_span);
    let first_pass_count = usize::from(counters.first_pass_dbf_counter).wrapping_add(1);
    let first_pass_start = usize::from(current_layout_row_offset as u16);
    let first_pass_slice =
        layout_row.get(first_pass_start..first_pass_start.wrapping_add(first_pass_count))?;
    let first_pass_mapping_words = first_pass_slice
        .iter()
        .map(|&mapping_index| block_mappings.get(usize::from(mapping_index)).copied())
        .collect::<Option<Vec<_>>>()?;

    let second_pass_count = usize::from(wrapped_tile_window_span).wrapping_add(1);
    let second_pass_start = first_pass_start.wrapping_add(first_pass_count);
    let second_pass_slice =
        layout_row.get(second_pass_start..second_pass_start.wrapping_add(second_pass_count))?;
    let second_pass_mapping_words = second_pass_slice
        .iter()
        .map(|&mapping_index| block_mappings.get(usize::from(mapping_index)).copied())
        .collect::<Option<Vec<_>>>()?;

    Some(BackgroundScrollPieceRefreshTileWrappedCopyLoc308E8 {
        first_pass_mapping_words,
        seam_vdp_control_word,
        seam_vdp_autoincrement_word: 3,
        second_pass_mapping_words,
        bytes_consumed: first_pass_count.wrapping_add(second_pass_count),
        final_layout_row_offset: current_layout_row_offset
            .wrapping_add((first_pass_count.wrapping_add(second_pass_count)) as i16),
    })
}

#[allow(non_snake_case)]
#[allow(dead_code)]
pub(crate) fn background_scroll_piece_refresh_dual_wrap_branch_loc_30904(
    wrapped_tile_window_span: u16,
    wrapped_row_window_end: i16,
) -> BackgroundScrollPieceRefreshDualWrapBranchLoc30904 {
    // ASM loc_30904 (kid.asm:34756): compare wrapped tile seam `d6` against
    // wrapped row seam `d7`; `bgt.s loc_30936` means the row wrap happens
    // later, `blt.w loc_30978` means the tile wrap happens later, and equality
    // falls through to loc_30916 where both seams share the same split point.
    match wrapped_row_window_end.cmp(&(wrapped_tile_window_span as i16)) {
        std::cmp::Ordering::Greater => {
            BackgroundScrollPieceRefreshDualWrapBranchLoc30904::RowWrapFirstLoc30936
        }
        std::cmp::Ordering::Less => {
            BackgroundScrollPieceRefreshDualWrapBranchLoc30904::TileWrapFirstLoc30978
        }
        std::cmp::Ordering::Equal => {
            BackgroundScrollPieceRefreshDualWrapBranchLoc30904::SharedSplitLoc30916
        }
    }
}

#[allow(non_snake_case)]
#[allow(dead_code)]
pub(crate) fn background_scroll_piece_refresh_row_wrap_first_copy_loc_30942(
    layout_row: &[u8],
    current_layout_row_offset: i16,
    block_mappings: &[u16],
    background_width: i16,
    wrapped_row_window_end: i16,
) -> Option<BackgroundScrollPieceRefreshRowWrapFirstCopyLoc30942> {
    // ASM loc_30942 (kid.asm:34788): after loc_30936 seeds `d4 = $27 - d7`,
    // this first inclusive `dbf d4,loc_30942` pass copies exactly `$28 - d7`
    // mapping words from the current `(a0)+` position, then rewinds `a0` by
    // `Background_width` before loc_30954 starts the wrapped-row middle span.
    let first_pass_count = 0x0028_usize.wrapping_sub(usize::from(wrapped_row_window_end as u16));
    let first_pass_start = usize::from(current_layout_row_offset as u16);
    let first_pass_slice =
        layout_row.get(first_pass_start..first_pass_start.wrapping_add(first_pass_count))?;
    let mapping_words = first_pass_slice
        .iter()
        .map(|&mapping_index| block_mappings.get(usize::from(mapping_index)).copied())
        .collect::<Option<Vec<_>>>()?;

    Some(BackgroundScrollPieceRefreshRowWrapFirstCopyLoc30942 {
        mapping_words,
        wrapped_layout_row_offset: current_layout_row_offset
            .wrapping_add(first_pass_count as i16)
            .wrapping_sub(background_width),
        bytes_consumed: first_pass_count,
    })
}

#[allow(non_snake_case)]
#[allow(dead_code)]
pub(crate) fn background_scroll_piece_refresh_row_wrap_second_copy_loc_30954(
    layout_row: &[u8],
    current_layout_row_offset: i16,
    block_mappings: &[u16],
    wrapped_tile_window_span: u16,
    wrapped_row_window_end: i16,
    seam_vdp_control_word: u16,
) -> Option<BackgroundScrollPieceRefreshRowWrapSecondCopyLoc30954> {
    // ASM loc_30954 (kid.asm:34796): after loc_30942 rewinds `a0`, this
    // inclusive `dbf d7,loc_30954` pass copies the middle span between the row
    // wrap seam and the tile wrap seam, where loc_30936 reduced `d7` to
    // `wrapped_row_window_end - wrapped_tile_window_span - 1`. The tail then
    // restores `d3` and autoincrement `#3` to `(a5)` before loc_30968 copies
    // the final wrapped-tile segment.
    let second_pass_count =
        usize::from((wrapped_row_window_end as u16).wrapping_sub(wrapped_tile_window_span));
    let second_pass_start = usize::from(current_layout_row_offset as u16);
    let second_pass_slice =
        layout_row.get(second_pass_start..second_pass_start.wrapping_add(second_pass_count))?;
    let mapping_words = second_pass_slice
        .iter()
        .map(|&mapping_index| block_mappings.get(usize::from(mapping_index)).copied())
        .collect::<Option<Vec<_>>>()?;

    Some(BackgroundScrollPieceRefreshRowWrapSecondCopyLoc30954 {
        mapping_words,
        seam_vdp_control_word,
        seam_vdp_autoincrement_word: 3,
        next_layout_row_offset: current_layout_row_offset.wrapping_add(second_pass_count as i16),
        bytes_consumed: second_pass_count,
    })
}

#[allow(non_snake_case)]
#[allow(dead_code)]
pub(crate) fn background_scroll_piece_refresh_row_wrap_final_copy_loc_30968(
    layout_row: &[u8],
    current_layout_row_offset: i16,
    block_mappings: &[u16],
    wrapped_tile_window_span: u16,
) -> Option<BackgroundScrollPieceRefreshRowWrapFinalCopyLoc30968> {
    // ASM loc_30968 (kid.asm:34805): once loc_30954 restores the saved plane-B
    // seam command, the final inclusive `dbf d5,loc_30968` pass consumes the
    // wrapped tile tail, where loc_30936 saved the original wrapped tile span
    // in `d5 = d6`. That makes the final copy length exactly `d6 + 1`.
    let final_pass_count = usize::from(wrapped_tile_window_span).wrapping_add(1);
    let final_pass_start = usize::from(current_layout_row_offset as u16);
    let final_pass_slice =
        layout_row.get(final_pass_start..final_pass_start.wrapping_add(final_pass_count))?;
    let mapping_words = final_pass_slice
        .iter()
        .map(|&mapping_index| block_mappings.get(usize::from(mapping_index)).copied())
        .collect::<Option<Vec<_>>>()?;

    Some(BackgroundScrollPieceRefreshRowWrapFinalCopyLoc30968 {
        mapping_words,
        next_layout_row_offset: current_layout_row_offset.wrapping_add(final_pass_count as i16),
        bytes_consumed: final_pass_count,
    })
}

#[allow(non_snake_case)]
#[allow(dead_code)]
pub(crate) fn background_scroll_piece_refresh_tile_wrap_first_copy_loc_30988(
    layout_row: &[u8],
    current_layout_row_offset: i16,
    block_mappings: &[u16],
    wrapped_tile_window_end: i16,
    seam_vdp_control_word: u16,
) -> Option<BackgroundScrollPieceRefreshTileWrapFirstCopyLoc30988> {
    // ASM loc_30988 (kid.asm:34823): after loc_30978 seeds `d4 = $27 - d6`,
    // this first inclusive `dbf d4,loc_30988` pass copies exactly `$28 - d6`
    // mapping words from the current `(a0)+` position. The tail then restores
    // the saved plane-B seam command `d3` and autoincrement `#3` to `(a5)`
    // before `moveq #1,d2` arms loc_3099E for exactly one wrapped retry.
    let first_pass_count = 0x0028_usize.wrapping_sub(usize::from(wrapped_tile_window_end as u16));
    let first_pass_start = usize::from(current_layout_row_offset as u16);
    let first_pass_slice =
        layout_row.get(first_pass_start..first_pass_start.wrapping_add(first_pass_count))?;
    let mapping_words = first_pass_slice
        .iter()
        .map(|&mapping_index| block_mappings.get(usize::from(mapping_index)).copied())
        .collect::<Option<Vec<_>>>()?;

    Some(BackgroundScrollPieceRefreshTileWrapFirstCopyLoc30988 {
        mapping_words,
        seam_vdp_control_word,
        seam_vdp_autoincrement_word: 3,
        next_layout_row_offset: current_layout_row_offset.wrapping_add(first_pass_count as i16),
        bytes_consumed: first_pass_count,
    })
}

#[allow(non_snake_case)]
#[allow(dead_code)]
pub(crate) fn background_scroll_piece_refresh_tile_wrap_wrapped_copy_loc_3099E(
    layout_row: &[u8],
    current_layout_row_offset: i16,
    block_mappings: &[u16],
    background_width: i16,
    wrapped_tile_window_end: i16,
    wrapped_row_window_end: i16,
) -> Option<BackgroundScrollPieceRefreshTileWrapWrappedCopyLoc3099E> {
    // ASM loc_3099E (kid.asm:34833): after loc_30988 arms `d2 = 1`, this loop
    // first runs the inclusive `dbf d1,loc_3099E` span where loc_30978
    // reduced `d1` to `d6 - d7 - 1`, so it copies exactly `d6 - d7` words up
    // to the tile seam. The tail then reloads `d1` from the saved row-wrap
    // end `d5`, rewinds `a0` by `Background_width`, and the `dbf d2` back-edge
    // permits exactly one wrapped retry that copies `d5 + 1` more words.
    let first_pass_count =
        usize::from((wrapped_tile_window_end as u16).wrapping_sub(wrapped_row_window_end as u16));
    let first_pass_start = usize::from(current_layout_row_offset as u16);
    let first_pass_slice =
        layout_row.get(first_pass_start..first_pass_start.wrapping_add(first_pass_count))?;
    let first_pass_mapping_words = first_pass_slice
        .iter()
        .map(|&mapping_index| block_mappings.get(usize::from(mapping_index)).copied())
        .collect::<Option<Vec<_>>>()?;

    let wrapped_layout_row_offset = current_layout_row_offset
        .wrapping_add(first_pass_count as i16)
        .wrapping_sub(background_width);
    let second_pass_count = usize::from(wrapped_row_window_end as u16).wrapping_add(1);
    let second_pass_start = usize::from(wrapped_layout_row_offset as u16);
    let second_pass_slice =
        layout_row.get(second_pass_start..second_pass_start.wrapping_add(second_pass_count))?;
    let second_pass_mapping_words = second_pass_slice
        .iter()
        .map(|&mapping_index| block_mappings.get(usize::from(mapping_index)).copied())
        .collect::<Option<Vec<_>>>()?;

    Some(BackgroundScrollPieceRefreshTileWrapWrappedCopyLoc3099E {
        first_pass_mapping_words,
        second_pass_mapping_words,
        bytes_consumed: first_pass_count.wrapping_add(second_pass_count),
        final_layout_row_offset: wrapped_layout_row_offset.wrapping_add(second_pass_count as i16),
    })
}

#[allow(non_snake_case)]
#[allow(dead_code)]
pub(crate) fn background_scroll_enigma_row_table_lookup_loc_303EA(
    tile_row: u16,
    background_theme: u16,
    background_row_table: &[u8],
) -> Option<BackgroundScrollEnigmaRowTableLookupLoc303EA> {
    // ASM loc_303EA (kid.asm:34025): after `cmpi.w #Desert,(Background_theme).w`
    // leaves the theme non-Desert or after the preceding `addq.w #1,a0`,
    // this entry clears `d1`, reads `1(a0,d0.w)` as the row byte, multiplies
    // that index by `$40`, masks the tile row with `$1F`, and seeds plane B
    // with `$6000 | ((d0 & $1F) << 7)` plus autoincrement `#3`.
    let desert_bias = usize::from(background_theme == 5);
    let map_header_byte_offset = usize::from(tile_row).wrapping_add(1 + desert_bias);
    let background_row_index = *background_row_table.get(map_header_byte_offset)?;

    Some(BackgroundScrollEnigmaRowTableLookupLoc303EA {
        map_header_byte_offset,
        background_row_index,
        row_mapping_byte_offset: usize::from(background_row_index) << 6,
        vdp_control_word: 0x6000 | ((tile_row & 0x001F) << 7),
        vdp_autoincrement_word: 3,
    })
}

#[allow(non_snake_case)]
#[allow(dead_code)]
pub(crate) fn background_scroll_enigma_primary_copy_setup_loc_30426()
-> BackgroundScrollEnigmaPrimaryCopySetupLoc30426 {
    // ASM loc_30426 (kid.asm:34048): both source-selection paths converge here
    // and reload `d0` with `#7`, so the following `dbf d0,loc_30428` loop runs
    // for 8 inclusive iterations.
    BackgroundScrollEnigmaPrimaryCopySetupLoc30426 {
        longword_dbf_counter: 7,
    }
}

#[allow(non_snake_case)]
#[allow(dead_code)]
pub(crate) fn background_scroll_enigma_primary_longword_copy_loc_30428(
    source_longwords: &[u32],
    dbf_counter: u16,
) -> Vec<u32> {
    // ASM loc_30428 (kid.asm:34051): the loop body writes two longwords from
    // `(a0)+` to the VDP data port before each `dbf d0,loc_30428` test, so the
    // inclusive counter copies `(d0 + 1) * 2` longwords from the primary
    // decompression stream.
    source_longwords
        .iter()
        .copied()
        .take((usize::from(dbf_counter) + 1) * 2)
        .collect()
}

#[allow(non_snake_case)]
#[allow(dead_code)]
pub(crate) fn background_scroll_enigma_secondary_longword_copy_loc_30432(
    source_longwords: &[u32],
    dbf_counter: u16,
) -> Vec<u32> {
    // ASM loc_30432 (kid.asm:34057): after `moveq #7,d0`, the secondary pass
    // streams two longwords from `(a1)+` to the VDP data port before each
    // `dbf d0,loc_30432` test, so the inclusive counter copies
    // `(d0 + 1) * 2` longwords from the secondary decompression stream.
    source_longwords
        .iter()
        .copied()
        .take((usize::from(dbf_counter) + 1) * 2)
        .collect()
}

#[allow(non_snake_case)]
#[allow(dead_code)]
pub(crate) fn background_scroll_shift_data_slot_loc_30452(
    phase_word: u16,
    previous_scroll_word: u16,
    camera_x_pos: u16,
) -> BackgroundScrollShiftDataSlotLoc30452 {
    // ASM loc_30452 (kid.asm:34078): each slot starts by loading the prior
    // scroll word from `4(a0)` into `d2`, then `lsr.w #1,d0`, caches that
    // halved phase word in `a5`, adds `Camera_X_pos>>3`, stores the new word
    // back to `4(a0)`, mirrors the previous word into `6(a0)`, and compares
    // the old/new words again after logical `>>3` tile-column quantisation.
    let next_phase_word = phase_word >> 1;
    let updated_scroll_word = next_phase_word.wrapping_add(camera_x_pos >> 3);
    let previous_tile_column = previous_scroll_word >> 3;
    let updated_tile_column = updated_scroll_word >> 3;

    BackgroundScrollShiftDataSlotLoc30452 {
        next_phase_word,
        updated_scroll_word,
        cached_previous_scroll_word: previous_scroll_word,
        previous_tile_column,
        updated_tile_column,
        branches_to_loc_30476: previous_tile_column == updated_tile_column,
    }
}

#[allow(non_snake_case)]
#[allow(dead_code)]
pub(crate) fn background_scroll_shift_data_unchanged_loc_30476(
    previous_scroll_word: u16,
    updated_scroll_word: u16,
) -> Option<u16> {
    // ASM loc_30476 (kid.asm:34095): when the logical `>>3` tile columns match,
    // the routine skips `sub_304B4` and writes `$FFFF` to `(a0)` as the
    // unchanged-column sentinel before falling into the shared tail at
    // `loc_3047A`.
    if (previous_scroll_word >> 3) == (updated_scroll_word >> 3) {
        Some(0xFFFF)
    } else {
        None
    }
}

#[allow(non_snake_case)]
#[allow(dead_code)]
pub(crate) fn background_scroll_shift_data_second_pass_slot_loc_3048C(
    next_scroll_word: u16,
    previous_scroll_word: u16,
) -> BackgroundScrollShiftDataSecondPassSlotLoc3048C {
    // ASM loc_3048C (kid.asm:34108): `swap d5` exposes the next scroll word in
    // the low half, then the routine loads `4(a0)` into `d2`, stores `d5` back
    // to `4(a0)`, mirrors the old word to `6(a0)`, and branches to `loc_304A4`
    // only when `cmp.w d5,d2` reports the words are identical.
    BackgroundScrollShiftDataSecondPassSlotLoc3048C {
        updated_scroll_word: next_scroll_word,
        cached_previous_scroll_word: previous_scroll_word,
        branches_to_loc_304_a4: previous_scroll_word == next_scroll_word,
    }
}

#[allow(non_snake_case)]
#[allow(dead_code)]
pub(crate) fn background_scroll_shift_data_second_pass_unchanged_loc_304A4(
    next_scroll_word: u16,
    previous_scroll_word: u16,
) -> Option<u16> {
    // ASM loc_304A4 (kid.asm:34119): the `beq.s` from `loc_3048C` lands here
    // only when `cmp.w d5,d2` found the new and previous scroll words equal,
    // so this arm writes the shared `$FFFF` unchanged sentinel to `(a0)`.
    if previous_scroll_word == next_scroll_word {
        Some(0xFFFF)
    } else {
        None
    }
}

#[allow(non_snake_case)]
#[allow(dead_code)]
pub(crate) fn background_scroll_shift_data_second_pass_advance_loc_304A8(
    swapped_phase_accumulator: u32,
    phase_step_fp: u32,
    dbf_counter: u16,
) -> BackgroundScrollShiftDataSecondPassAdvanceLoc304A8 {
    // ASM loc_304A8 (kid.asm:34122): advance `a0` by 8 bytes to the next slot,
    // `swap d5` back to the accumulator ordering used before `loc_3048C`, add
    // the fixed-point step from `d4`, then let `dbf d3,loc_3048C` consume one
    // inclusive iteration before the final `rts`.
    let restored_phase_accumulator = swapped_phase_accumulator.rotate_left(16);

    BackgroundScrollShiftDataSecondPassAdvanceLoc304A8 {
        next_slot_offset_bytes: 8,
        next_phase_accumulator: restored_phase_accumulator.wrapping_add(phase_step_fp),
        next_dbf_counter: dbf_counter.wrapping_sub(1),
        loops_to_loc_3048_c: dbf_counter != 0,
    }
}

fn reduce_background_scroll_phase(seed_phase_row: i16, background_width: i16) -> (i16, u16) {
    debug_assert!(background_width > 0, "ASM sub_304B4 expects positive width");

    let mut reduced_phase_row = seed_phase_row;
    let mut subtraction_count = 0_u16;

    while background_width <= reduced_phase_row {
        reduced_phase_row -= background_width;
        subtraction_count = subtraction_count.wrapping_add(1);
    }

    (reduced_phase_row, subtraction_count)
}

#[allow(non_snake_case)]
#[allow(dead_code)]
pub(crate) fn background_scroll_shift_data_wrap_loop_body_loc_304E0(
    seeded_phase_row: i16,
    background_width: i16,
) -> BackgroundScrollShiftDataWrapLoopBodyLoc304E0 {
    // ASM loc_304E0 (kid.asm:34161): after `loc_304D8` seeds `d7` from the
    // signed `asr.w #3,4(a0)` result, the shared body loops on
    // `cmp.w d7,d6`/`sub.w d6,d7` until `d6 > d7`, then falls through to
    // `loc_304CA` with the reduced row.
    let (reduced_phase_row, subtraction_count) =
        reduce_background_scroll_phase(seeded_phase_row, background_width);

    BackgroundScrollShiftDataWrapLoopBodyLoc304E0 {
        reduced_phase_row,
        subtraction_count,
    }
}

#[allow(non_snake_case)]
#[allow(dead_code)]
pub(crate) fn background_scroll_shift_data_wrap_loop_loc_304C2(
    updated_scroll_word: u16,
    background_width: i16,
) -> BackgroundScrollShiftDataWrapLoopLoc304C2 {
    // ASM loc_304C2 (kid.asm:34141): the non-`bgt` entry seeds `d7` from
    // `asr.w #3,4(a0)` plus `$28`, mirrors that biased row into `d1`, then
    // keeps subtracting `d6` until `cmp.w d7,d6` sees `d6 > d7` and falls into
    // `loc_304CA`.
    let seeded_phase_row = (updated_scroll_word as i16 >> 3) + 0x28;
    let (reduced_phase_row, subtraction_count) =
        reduce_background_scroll_phase(seeded_phase_row, background_width);

    BackgroundScrollShiftDataWrapLoopLoc304C2 {
        seeded_phase_row,
        reduced_phase_row,
        subtraction_count,
    }
}

#[allow(non_snake_case)]
#[allow(dead_code)]
pub(crate) fn background_scroll_shift_data_wrap_store_loc_304CA(
    seeded_phase_row: i16,
    reduced_phase_row: i16,
) -> BackgroundScrollShiftDataWrapStoreLoc304CA {
    // ASM loc_304CA (kid.asm:34148): after either wrap loop exits, the shared
    // tail stores the reduced `d7` word to `(a0)`, masks the original seed in
    // `d1` with `$3F`, doubles it, and writes that word offset to `2(a0)`.
    let reduced_phase_row_word = reduced_phase_row as u16;
    let strip_table_word_offset = ((seeded_phase_row as u16) & 0x003F) << 1;

    BackgroundScrollShiftDataWrapStoreLoc304CA {
        reduced_phase_row_word,
        strip_table_word_offset,
    }
}

#[allow(non_snake_case)]
#[allow(dead_code)]
pub(crate) fn background_scroll_shift_data_wrap_loop_loc_304D8(
    updated_scroll_word: u16,
    background_width: i16,
) -> BackgroundScrollShiftDataWrapLoopLoc304D8 {
    // ASM loc_304D8 (kid.asm:34156): the `bgt` arm skips the `$28` bias,
    // seeds both `d7` and `d1` from the signed `asr.w #3,4(a0)` result, then
    // branches into the shared subtract-until-`d6>d7` body at `loc_304E0`
    // before joining `loc_304CA`.
    let seeded_phase_row = updated_scroll_word as i16 >> 3;
    let loop_body =
        background_scroll_shift_data_wrap_loop_body_loc_304E0(seeded_phase_row, background_width);

    BackgroundScrollShiftDataWrapLoopLoc304D8 {
        seeded_phase_row,
        reduced_phase_row: loop_body.reduced_phase_row,
        subtraction_count: loop_body.subtraction_count,
    }
}

#[allow(non_snake_case)]
#[allow(dead_code)]
pub(crate) fn background_scroll_shift_layers_no_scroll_fill_loc_30502(
    camera_x_pos: i16,
) -> BackgroundScrollShiftLayersNoScrollFillLoc30502 {
    // ASM loc_30502 (kid.asm:34182): after the no-scroll path negates
    // `(Camera_X_pos).w`, swaps it into the high word of `d2`, and clears the
    // low word, the loop stores that 16.16 longword to `(a3)+` for the
    // inclusive `dbf d0,loc_30502` span of `$DF+1` entries.
    let repeated_scroll_longword = (u32::from(camera_x_pos.wrapping_neg() as u16)) << 16;

    BackgroundScrollShiftLayersNoScrollFillLoc30502 {
        repeated_scroll_longword,
        longword_count: 0x00DF + 1,
    }
}

#[allow(non_snake_case)]
#[allow(dead_code)]
pub(crate) fn background_scroll_shift_layers_setup_loc_3050A(
    current_level_id: u16,
    map_order_index: &[u8],
    background_scroll_index_offsets: &[u32],
    camera_y_pos: i16,
    camera_x_pos: i16,
) -> Option<BackgroundScrollShiftLayersSetupLoc3050A> {
    // ASM loc_3050A (kid.asm:34188): reload `a3` with `Horiz_Scroll_Buffer`,
    // seed `d5=#7`, resolve the level's row-sequence pointer through
    // `LnkTo_MapOrder_Index` and `LnkTo_BackgroundScroll_Index`, then derive
    // the row byte offset from logical `Camera_Y_pos>>5`. The setup keeps
    // `d0=#$1B`, seeds `d4=-1`, rebuilds `d2` as `-Camera_X_pos` in 16.16,
    // and uses `(Camera_Y_pos>>2) & 7` to decide whether to branch straight to
    // `loc_3056A` or split the buffer into leading/trailing partial spans
    // around the 8-row block loop.
    let map_order_entry = *map_order_index.get(usize::from(current_level_id))?;
    let background_scroll_table_index = usize::from(map_order_entry);
    background_scroll_index_offsets.get(background_scroll_table_index)?;

    let row_sequence_byte_offset = usize::try_from(
        background_scroll_index_offsets[background_scroll_table_index]
            .wrapping_add(u32::from((camera_y_pos as u16) >> 5)),
    )
    .ok()?;
    let quarter_row_phase = ((camera_y_pos as u16) >> 2) & 0x0007;
    let branches_to_loc_3056_a = quarter_row_phase == 0;
    let negated_camera_x_longword = (u32::from(camera_x_pos.wrapping_neg() as u16)) << 16;

    let (leading_partial_longword_count, trailing_partial_longword_count) =
        if branches_to_loc_3056_a {
            (None, None)
        } else {
            (
                Some(0x0008_u16.wrapping_sub(quarter_row_phase)),
                Some(quarter_row_phase),
            )
        };

    Some(BackgroundScrollShiftLayersSetupLoc3050A {
        map_order_entry,
        background_scroll_table_index,
        row_sequence_byte_offset,
        block_dbf_counter: 0x001B,
        negated_camera_x_longword,
        quarter_row_phase,
        branches_to_loc_3056_a,
        leading_partial_longword_count,
        trailing_partial_longword_count,
    })
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn background_scroll_shift_layers_leading_partial_fill_loc_30562(
    negated_camera_x_longword: u32,
    background_scroll_word: i16,
    quarter_row_phase: u16,
    block_dbf_counter: u16,
) -> Option<BackgroundScrollShiftLayersLeadingPartialFillLoc30562> {
    // ASM loc_30562 (kid.asm:34220): after `move.w d3,d4`, `eor.w #7,d3`,
    // `subq.w #1,d4`, and the preceding `move.w/neg.w` load rebuild `d2`, the
    // inclusive `dbf d3,loc_30562` loop writes the packed scroll longword
    // `8-phase` times, then `subq.w #1,d0` trims the later 8-row block count.
    if !(1..=7).contains(&quarter_row_phase) {
        return None;
    }

    Some(BackgroundScrollShiftLayersLeadingPartialFillLoc30562 {
        repeated_scroll_longword: negated_camera_x_longword
            | u32::from(background_scroll_word.wrapping_neg() as u16),
        longword_count: 0x0008 - quarter_row_phase,
        next_block_dbf_counter: block_dbf_counter.wrapping_sub(1),
    })
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn background_scroll_shift_layers_trailing_partial_fill_loc_30592(
    negated_camera_x_longword: u32,
    background_scroll_word: i16,
    trailing_partial_longword_count: u16,
) -> Option<BackgroundScrollShiftLayersTrailingPartialFillLoc30592> {
    // ASM loc_30592 (kid.asm:34244): once `tst.w d4` proves a trailing span
    // exists and the preceding `move.w/neg.w` pair rebuilds `d2`, the
    // inclusive `dbf d4,loc_30592` loop writes the final partial-row scroll
    // longword exactly `phase` times.
    if trailing_partial_longword_count == 0 {
        return None;
    }

    Some(BackgroundScrollShiftLayersTrailingPartialFillLoc30592 {
        repeated_scroll_longword: negated_camera_x_longword
            | u32::from(background_scroll_word.wrapping_neg() as u16),
        longword_count: trailing_partial_longword_count,
    })
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn background_scroll_theme_dispatch_loc_30598(
    background_theme: u16,
) -> BackgroundScrollThemeDispatchLoc30598 {
    const BACKGROUND_THEME_DESERT_LOC_30598: u16 = 0x0005;
    const BACKGROUND_THEME_FOREST_LOC_30598: u16 = 0x0009;

    // ASM loc_30598 (kid.asm:34248): after the partial-span loops finish, the
    // shared tail checks `(Background_theme).w`, branches to the Forest ripple
    // helper on `Forest`, branches to the Desert heat ripple helper on
    // `Desert`, and otherwise returns immediately.
    if background_theme == BACKGROUND_THEME_FOREST_LOC_30598 {
        BackgroundScrollThemeDispatchLoc30598::ApplyForestWaterRipple
    } else if background_theme == BACKGROUND_THEME_DESERT_LOC_30598 {
        BackgroundScrollThemeDispatchLoc30598::ApplyDesertHeatRipple
    } else {
        BackgroundScrollThemeDispatchLoc30598::Return
    }
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn background_scroll_forest_water_ripple_visible_band_loc_305D4(
    start_scanline: i16,
) -> Option<BackgroundScrollForestWaterRippleVisibleBandLoc305D4> {
    // ASM loc_305D4 (kid.asm:34276): when the prelude's `sub.w d1,d0` stays
    // non-negative, this path rebuilds `d1` as `$E0-d0`, returns when the
    // span is non-positive, clears `d2`, and clamps longer visible bands to
    // `$40` scanlines before falling into `loc_305E6`.
    let affected_scanline_count = 0x00E0_i16.wrapping_sub(start_scanline);
    if affected_scanline_count <= 0 {
        return None;
    }

    Some(BackgroundScrollForestWaterRippleVisibleBandLoc305D4 {
        start_scanline: start_scanline as u16,
        ripple_data_byte_offset: 0,
        affected_scanline_count: affected_scanline_count.min(0x0040) as u16,
    })
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn background_scroll_forest_water_ripple_setup_loc_305E6(
    horiz_scroll_buffer: &[u32],
    start_scanline: u16,
    affected_scanline_count: u16,
    ripple_data_byte_offset: u16,
    time_frames: u16,
) -> Option<BackgroundScrollForestWaterRippleSetupLoc305E6> {
    // ASM loc_305E6 (kid.asm:34285): scale `d0` by 4, bias `a0` to the second
    // word of the selected `Horiz_Scroll_Buffer` longword, cache that word in
    // `d3`, turn the visible span into an inclusive `dbf` counter with
    // `subq.w #1,d1`, and bias the ripple-table pointer by `(Time_Frames>>2) &
    // $3F` after adding the caller's `d2` offset.
    if affected_scanline_count == 0 {
        return None;
    }

    let base_scroll_longword = *horiz_scroll_buffer.get(usize::from(start_scanline))?;

    Some(BackgroundScrollForestWaterRippleSetupLoc305E6 {
        start_scanline,
        target_word_offset_bytes: usize::from(start_scanline) * 4 + 2,
        base_scroll_word: base_scroll_longword as u16 as i16,
        ripple_dbf_counter: affected_scanline_count.wrapping_sub(1),
        ripple_data_byte_offset: usize::from(
            ripple_data_byte_offset.wrapping_add((time_frames >> 2) & 0x003F),
        ),
    })
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn background_scroll_forest_water_ripple_loop_loc_3060A(
    horiz_scroll_buffer: &[u32],
    start_scanline: u16,
    ripple_dbf_counter: u16,
    base_scroll_word: i16,
    ripple_data: &[u8],
    ripple_data_byte_offset: usize,
) -> Option<Vec<u32>> {
    // ASM loc_3060A (kid.asm:34298): each inclusive `dbf d1,loc_3060A`
    // iteration sign-extends the next ripple byte into `d4`, adds the cached
    // base word from `d3`, stores only that low word to `(a0)`, then advances
    // `a0` by 4 bytes to the next `Horiz_Scroll_Buffer` longword.
    let write_count = usize::from(ripple_dbf_counter) + 1;
    let end_scanline = usize::from(start_scanline).checked_add(write_count)?;
    let end_ripple_byte = ripple_data_byte_offset.checked_add(write_count)?;
    if end_scanline > horiz_scroll_buffer.len() || end_ripple_byte > ripple_data.len() {
        return None;
    }

    let mut updated_scroll_buffer = horiz_scroll_buffer.to_vec();
    for write_index in 0..write_count {
        let ripple_delta = i16::from(ripple_data[ripple_data_byte_offset + write_index] as i8);
        let scanline_index = usize::from(start_scanline) + write_index;
        let next_scroll_word = base_scroll_word.wrapping_add(ripple_delta) as u16;
        updated_scroll_buffer[scanline_index] =
            (updated_scroll_buffer[scanline_index] & 0xFFFF_0000) | u32::from(next_scroll_word);
    }

    Some(updated_scroll_buffer)
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn background_scroll_desert_heat_ripple_visible_band_loc_306C2(
    map_header_row_count: u8,
    camera_y_pos: i16,
) -> Option<BackgroundScrollDesertHeatRippleVisibleBandLoc306C2> {
    // ASM loc_306C2 / prelude (kid.asm:34451-34477): `d0` starts as the map
    // header byte from `Addr_MapHeader+$14` multiplied by 8, then subtracts
    // `(Camera_Y_pos>>2)`. Negative starts branch through the prelude that
    // clamps the visible window to the first `$10` rows by rebasing `d0=0`,
    // negating the byte offset into `d2`, and returning only when
    // `d0 + $10 > 0`; non-negative starts rebuild `d1=$E0-d0`, return on a
    // non-positive span, clear `d2`, and clamp longer windows to `$10`.
    let start_scanline =
        (i16::from(map_header_row_count) << 3).wrapping_sub(((camera_y_pos as u16) >> 2) as i16);

    if start_scanline < 0 {
        let affected_scanline_count = start_scanline.wrapping_add(0x0010);
        if affected_scanline_count <= 0 {
            return None;
        }

        return Some(BackgroundScrollDesertHeatRippleVisibleBandLoc306C2 {
            start_scanline: 0,
            ripple_data_byte_offset: start_scanline.wrapping_neg() as u16,
            affected_scanline_count: affected_scanline_count as u16,
        });
    }

    let affected_scanline_count = 0x00E0_i16.wrapping_sub(start_scanline);
    if affected_scanline_count <= 0 {
        return None;
    }

    Some(BackgroundScrollDesertHeatRippleVisibleBandLoc306C2 {
        start_scanline: start_scanline as u16,
        ripple_data_byte_offset: 0,
        affected_scanline_count: affected_scanline_count.min(0x0010) as u16,
    })
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn background_scroll_desert_heat_ripple_setup_loc_306D4(
    horiz_scroll_buffer: &[u32],
    start_scanline: u16,
    affected_scanline_count: u16,
    ripple_data_byte_offset: u16,
    time_frames: u16,
) -> Option<BackgroundScrollDesertHeatRippleSetupLoc306D4> {
    // ASM loc_306D4 (kid.asm:34469): scale `d0` by 4, bias `a0` to the second
    // word of the selected `Horiz_Scroll_Buffer` longword, cache that word in
    // `d3`, mask `Time_Frames` with `$30`, add the caller's `d2` byte offset,
    // turn the visible span into an inclusive `dbf` counter with
    // `subq.w #1,d1`, and seed `a1` from `BackgroundScroll_DesertHeatRippleData`.
    if affected_scanline_count == 0 {
        return None;
    }

    let base_scroll_longword = *horiz_scroll_buffer.get(usize::from(start_scanline))?;

    Some(BackgroundScrollDesertHeatRippleSetupLoc306D4 {
        start_scanline,
        target_word_offset_bytes: usize::from(start_scanline) * 4 + 2,
        base_scroll_word: base_scroll_longword as u16 as i16,
        ripple_dbf_counter: affected_scanline_count.wrapping_sub(1),
        ripple_data_byte_offset: usize::from(
            (time_frames & 0x0030).wrapping_add(ripple_data_byte_offset),
        ),
    })
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn background_scroll_desert_heat_ripple_loop_loc_306F4(
    horiz_scroll_buffer: &[u32],
    start_scanline: u16,
    ripple_dbf_counter: u16,
    base_scroll_word: i16,
    ripple_data: &[u8],
    ripple_data_byte_offset: usize,
) -> Option<Vec<u32>> {
    // ASM loc_306F4 (kid.asm:34481): each inclusive `dbf d1,loc_306F4`
    // iteration sign-extends the next heat-ripple byte into `d4`, adds the
    // cached base word from `d3`, writes only that low word to `(a0)`, then
    // advances `a0` by 4 bytes to the next `Horiz_Scroll_Buffer` longword.
    let write_count = usize::from(ripple_dbf_counter) + 1;
    let end_scanline = usize::from(start_scanline).checked_add(write_count)?;
    let end_ripple_byte = ripple_data_byte_offset.checked_add(write_count)?;
    if end_scanline > horiz_scroll_buffer.len() || end_ripple_byte > ripple_data.len() {
        return None;
    }

    let mut updated_scroll_buffer = horiz_scroll_buffer.to_vec();
    for write_index in 0..write_count {
        let ripple_delta = i16::from(ripple_data[ripple_data_byte_offset + write_index] as i8);
        let scanline_index = usize::from(start_scanline) + write_index;
        let next_scroll_word = base_scroll_word.wrapping_add(ripple_delta) as u16;
        updated_scroll_buffer[scanline_index] =
            (updated_scroll_buffer[scanline_index] & 0xFFFF_0000) | u32::from(next_scroll_word);
    }

    Some(updated_scroll_buffer)
}

#[allow(non_snake_case)]
#[allow(dead_code)]
pub(crate) fn background_scroll_enigma_row_transfer_loc_303D4(
    tile_row: u16,
    background_theme: u16,
    background_row_table: &[u8],
) -> Option<BackgroundScrollEnigmaRowTransferLoc303D4> {
    // ASM loc_303D4 (kid.asm:34017): after the caller decides whether the row
    // index needs the preceding `addi.w #$1C,d0`, the common tail reloads the
    // map-header row table from `Addr_MapHeader+$14`, biases Desert tables by
    // one extra byte, reads `1(a0,d0.w)`, then seeds plane-B VRAM with
    // `$6000 | ((d0 & $1F) << 7)` and autoincrement `#3`. Row indices below
    // `$1C` read paired strips from `$FFFF87B2` and `$FFFF8EB2` (`+$700`);
    // indices at or above `$1C` read both passes from `$FFFF8EB2`.
    let table_lookup = background_scroll_enigma_row_table_lookup_loc_303EA(
        tile_row,
        background_theme,
        background_row_table,
    )?;
    let map_header_byte_offset = table_lookup.map_header_byte_offset;
    let background_row_index = table_lookup.background_row_index;
    let row_mapping_byte_offset = table_lookup.row_mapping_byte_offset;
    let source = if background_row_index < 0x1C {
        BackgroundScrollEnigmaRowSourceLoc303D4::LowerDecompressionBuffer
    } else {
        BackgroundScrollEnigmaRowSourceLoc303D4::UpperDecompressionBuffer
    };
    let (primary_decompression_byte_offset, secondary_decompression_byte_offset) = match source {
        BackgroundScrollEnigmaRowSourceLoc303D4::LowerDecompressionBuffer => {
            (row_mapping_byte_offset, row_mapping_byte_offset + 0x0700)
        }
        BackgroundScrollEnigmaRowSourceLoc303D4::UpperDecompressionBuffer => {
            (row_mapping_byte_offset, row_mapping_byte_offset)
        }
    };
    let primary_copy_setup = background_scroll_enigma_primary_copy_setup_loc_30426();

    Some(BackgroundScrollEnigmaRowTransferLoc303D4 {
        map_header_byte_offset,
        background_row_index,
        row_mapping_byte_offset,
        vdp_control_word: table_lookup.vdp_control_word,
        vdp_autoincrement_word: table_lookup.vdp_autoincrement_word,
        source,
        primary_decompression_byte_offset,
        primary_longword_dbf_counter: primary_copy_setup.longword_dbf_counter,
        secondary_decompression_byte_offset,
        longword_rows: 8,
    })
}

#[allow(non_snake_case)]
#[allow(dead_code)]
pub(crate) fn transfer_scroll_data_dma_tail_loc_300D2(
    second_vsram_word: i16,
) -> TransferScrollDataDmaTailLoc300D2 {
    // ASM loc_300D2 (kid.asm:33686): stream the second VSRAM word, stop the
    // Z80, DMA `Horiz_Scroll_Buffer` into VRAM word address `$1400` for
    // `$380` words, then restart the Z80 before the shared return.
    TransferScrollDataDmaTailLoc300D2 {
        second_vsram_word,
        dma_source_addr: 0xFFFF_7432,
        dma_vram_word_addr: 0x1400,
        dma_word_count: 0x0380,
    }
}

#[allow(non_snake_case)]
#[allow(dead_code)]
pub(crate) fn transfer_scroll_data_storm_loc_30110(
    camera_x_pos: i16,
    camera_y_pos: i16,
    storm_scroll_offset: i16,
) -> TransferScrollDataStormLoc30110 {
    // ASM loc_30110 (kid.asm:33695): once the storm countdown reaches zero,
    // switch plane B to `$8403`, fill `$DF+1` horizontal-scroll word pairs
    // with `-Camera_X_pos` then the current storm offset, mirror
    // `Camera_Y_pos/storm_scroll_offset` into the two VSRAM slots, reset
    // `($FFFFFAD6).w` to 4, and subtract 8 from `($FFFFFAD8).w`.
    TransferScrollDataStormLoc30110 {
        plane_address_word: 0x8403,
        h_scroll_word_pairs: 0x00DF + 1,
        first_h_scroll_word: camera_x_pos.wrapping_neg(),
        second_h_scroll_word: storm_scroll_offset,
        first_vsram_word: camera_y_pos,
        second_vsram_word: storm_scroll_offset,
        weather_mode_byte: 4,
        next_storm_scroll_offset: storm_scroll_offset.wrapping_sub(8),
    }
}

#[allow(non_snake_case)]
#[allow(dead_code)]
pub(crate) fn transfer_scroll_data_storm_loop_loc_30130(
    camera_x_pos: i16,
    storm_scroll_offset: i16,
) -> TransferScrollDataStormLoopLoc30130 {
    // ASM loc_30130 (kid.asm:33704): the storm upload loop writes
    // `-Camera_X_pos` to `(a6)` first, then `($FFFFFAD8).w`, and `dbf d1`
    // runs the pair `$DF+1` times.
    TransferScrollDataStormLoopLoc30130 {
        h_scroll_word_pairs: 0x00DF + 1,
        first_h_scroll_word: camera_x_pos.wrapping_neg(),
        second_h_scroll_word: storm_scroll_offset,
    }
}

#[allow(non_snake_case)]
#[allow(dead_code)]
pub(crate) fn transfer_scroll_data_storm_setup_loc_30158(
    camera_x_pos: i16,
    storm_scroll_offset: i16,
) -> TransferScrollDataStormSetupLoc30158 {
    // ASM loc_30158 (kid.asm:33718): reload plane B's storm upload at
    // `vdpComm($1400,VRAM,WRITE)`, seed `d1=#$DF`, load `d0=Camera_X_pos`,
    // load `d2=($FFFFFAD8).w`, then `neg.w d0` before the reversed loop.
    TransferScrollDataStormSetupLoc30158 {
        h_scroll_word_pairs: 0x00DF + 1,
        first_h_scroll_word: storm_scroll_offset,
        second_h_scroll_word: camera_x_pos.wrapping_neg(),
    }
}

#[allow(non_snake_case)]
#[allow(dead_code)]
pub(crate) fn transfer_scroll_data_storm_loop_loc_30172(
    camera_x_pos: i16,
    camera_y_pos: i16,
    storm_scroll_offset: i16,
) -> TransferScrollDataStormLoopLoc30172 {
    // ASM loc_30172 / loc_3017E (kid.asm:33726): the alternate storm loop
    // writes `($FFFFFAD8).w` first and `-Camera_X_pos` second for each of the
    // `$DF+1` pairs, then mirrors `Camera_Y_pos` into both VSRAM slots before
    // subtracting 8 from `($FFFFFAD8).w`.
    TransferScrollDataStormLoopLoc30172 {
        h_scroll_word_pairs: 0x00DF + 1,
        first_h_scroll_word: storm_scroll_offset,
        second_h_scroll_word: camera_x_pos.wrapping_neg(),
        first_vsram_word: camera_y_pos,
        second_vsram_word: camera_y_pos,
        next_storm_scroll_offset: storm_scroll_offset.wrapping_sub(8),
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct BackgroundScrollStripeRowSelectionLoc301CE {
    pub(crate) row_pair_index: u16,
    pub(crate) theme_mapping_longword_offset: usize,
    pub(crate) block_mapping_longword_offset: usize,
}

#[allow(non_snake_case)]
#[allow(dead_code)]
pub(crate) fn background_scroll_stripe_row_selection_loc_301CE(
    tile_row: u16,
) -> BackgroundScrollStripeRowSelectionLoc301CE {
    // ASM loc_301CE (kid.asm:33765): copy `d6` into `d0`, `lsr.w #1,d0`, and
    // use the shifted-out low bit as the carry-tested odd-row selector. Odd
    // rows advance both `a1` and `a2` by 4 bytes so the renderer reads the
    // second longword from each 8-byte mapping pair; even rows keep the base
    // pair.
    let row_pair_index = tile_row >> 1;
    let mapping_longword_offset = if tile_row & 1 == 0 { 0 } else { 1 };

    BackgroundScrollStripeRowSelectionLoc301CE {
        row_pair_index,
        theme_mapping_longword_offset: mapping_longword_offset,
        block_mapping_longword_offset: mapping_longword_offset,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct BackgroundScrollStripeLayoutSeedLoc301D8 {
    pub(crate) row_pointer: u16,
    pub(crate) half_column_index: u16,
    pub(crate) half_column_mod32: u16,
    pub(crate) branches_to_loc_301_f6: bool,
    pub(crate) copy_columns: Option<u16>,
    pub(crate) initial_column_bias: Option<i16>,
}

#[allow(non_snake_case)]
#[allow(dead_code)]
pub(crate) fn background_scroll_stripe_layout_seed_loc_301D8(
    row_pointer_table: &[u16],
    row_pair_index: u16,
    camera_tile_column: u16,
) -> Option<BackgroundScrollStripeLayoutSeedLoc301D8> {
    // ASM loc_301D8 (kid.asm:33772): double the `d0 = d6>>1` row-pair index to
    // address the word row-pointer table at `$FFFF4A04`, then rebuild `d0`
    // from `d7>>1` so `a3` advances by `2 * (Camera_X_pos>>4)`. The masked
    // `andi.w #$1F,d0` half-column selects the left-window fast path
    // (`d1=$14`, `d2=-1`) until the compare against `$C` branches onward to
    // `loc_301F6`.
    let row_pointer = *row_pointer_table.get(usize::from(row_pair_index))?;
    let half_column_index = camera_tile_column >> 1;
    let half_column_mod32 = half_column_index & 0x001F;
    let branches_to_loc_301F6 = half_column_mod32 >= 0x000C;

    Some(BackgroundScrollStripeLayoutSeedLoc301D8 {
        row_pointer: row_pointer.wrapping_add(half_column_index.wrapping_mul(2)),
        half_column_index,
        half_column_mod32,
        branches_to_loc_301_f6: branches_to_loc_301F6,
        copy_columns: (!branches_to_loc_301F6).then_some(0x0014),
        initial_column_bias: (!branches_to_loc_301F6).then_some(-1),
    })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct BackgroundScrollStripeRightWindowLoc301F6 {
    pub(crate) copy_columns: u16,
    pub(crate) initial_column_bias: i16,
}

#[allow(non_snake_case)]
#[allow(dead_code)]
pub(crate) fn background_scroll_stripe_right_window_loc_301F6(
    half_column_mod32: u16,
) -> BackgroundScrollStripeRightWindowLoc301F6 {
    // ASM loc_301F6 (kid.asm:33787): once `loc_301D8` branches here, `d0`
    // already holds `(Camera_X_pos>>4) & $1F`. The routine flips the span with
    // `moveq #$1F,d1; sub.w d0,d1`, then derives the starting bias from
    // `moveq #$13,d2; sub.w d1,d2`, which simplifies to `d2 = d0 - $C`.
    BackgroundScrollStripeRightWindowLoc301F6 {
        copy_columns: 0x001F_u16.wrapping_sub(half_column_mod32),
        initial_column_bias: i16::try_from(half_column_mod32).unwrap_or(i16::MAX) - 0x000C,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct BackgroundScrollStripeVerticalPassLoc301FE {
    pub(crate) vdp_command: u32,
    pub(crate) saved_camera_tile_column: u16,
    pub(crate) second_pass_vdp_command: Option<u32>,
}

#[allow(non_snake_case)]
#[allow(dead_code)]
pub(crate) fn background_scroll_stripe_vertical_pass_loc_301FE(
    tile_row: u16,
    half_column_mod32: u16,
    camera_tile_column: u16,
    initial_column_bias: i16,
) -> BackgroundScrollStripeVerticalPassLoc301FE {
    // ASM loc_301FE (kid.asm:33793): fold `d6 & $1F` into bits 7-11, fold the
    // current half-column into bits 2-6, set bit `$4000`, `swap` the word into
    // the upper half, clear the low word, and push `d7` before the first
    // `sub_3034A` call. If `d2 >= 0` after the call, `andi.l #$FF80FFFF,d0`
    // clears only the column bits for the wraparound second pass.
    let vram_word = ((tile_row & 0x001F) << 7) | ((half_column_mod32 & 0x001F) << 2) | 0x4000;
    let vdp_command = u32::from(vram_word) << 16;
    let second_pass_vdp_command = (initial_column_bias >= 0).then_some(vdp_command & 0xFF80_FFFF);

    BackgroundScrollStripeVerticalPassLoc301FE {
        vdp_command,
        saved_camera_tile_column: camera_tile_column,
        second_pass_vdp_command,
    }
}

#[allow(non_snake_case)]
#[allow(dead_code)]
pub(crate) fn background_scroll_stripe_restore_camera_column_loc_3023A(
    saved_camera_tile_column: u16,
) -> u16 {
    // ASM loc_3023A (kid.asm:33816): both the single-pass and wraparound
    // vertical paths rejoin here and `move.w (sp)+,d7` restores the original
    // `Camera_X_pos>>3` tile column before the horizontal pass starts at
    // `loc_3023C`.
    saved_camera_tile_column
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct BackgroundScrollStripeHorizontalGateLoc3023C {
    pub(crate) render_column: Option<u16>,
    pub(crate) latched_camera_x_pos: u16,
    pub(crate) latched_camera_y_pos: u16,
}

#[allow(non_snake_case)]
#[allow(dead_code)]
pub(crate) fn background_scroll_stripe_horizontal_gate_loc_3023C(
    current_camera_tile_column: u16,
    saved_camera_tile_column: u16,
    level_width_tiles: u16,
    camera_x_pos: u16,
    camera_y_pos: u16,
) -> BackgroundScrollStripeHorizontalGateLoc3023C {
    // ASM loc_3023C (kid.asm:33819): reload the saved `Camera_X_pos>>3` from
    // `($FFFFF82C).w`, compare it against the restored `d7`, and skip the
    // horizontal upload entirely when the tile column is unchanged. Scrolling
    // left reuses the current column; scrolling right advances the render edge
    // by `$28` tiles and drops the pass if that would reach or exceed
    // `Level_width_tiles`. The shared tail at `loc_302E8` always latches the
    // current camera X/Y words back into `$FFFFF82C/$FFFFF82E`.
    let render_column = match current_camera_tile_column.cmp(&saved_camera_tile_column) {
        std::cmp::Ordering::Equal => None,
        std::cmp::Ordering::Less => Some(current_camera_tile_column),
        std::cmp::Ordering::Greater => {
            let right_edge_column = current_camera_tile_column.wrapping_add(0x0028);
            (right_edge_column < level_width_tiles).then_some(right_edge_column)
        }
    };

    BackgroundScrollStripeHorizontalGateLoc3023C {
        render_column,
        latched_camera_x_pos: camera_x_pos,
        latched_camera_y_pos: camera_y_pos,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct BackgroundScrollStripeHorizontalLayoutSeedLoc3026C {
    pub(crate) theme_mapping_word_offset: usize,
    pub(crate) block_mapping_word_offset: usize,
    pub(crate) row_pointer: u16,
    pub(crate) half_row_index: u16,
    pub(crate) half_row_mod16: u16,
    pub(crate) branches_to_loc_3028_e: bool,
    pub(crate) copy_rows: Option<u16>,
    pub(crate) initial_row_bias: Option<i16>,
}

#[allow(non_snake_case)]
#[allow(dead_code)]
pub(crate) fn background_scroll_stripe_horizontal_layout_seed_loc_3026C(
    row_pointer_table: &[u16],
    camera_tile_row: u16,
    render_column: u16,
) -> Option<BackgroundScrollStripeHorizontalLayoutSeedLoc3026C> {
    // ASM loc_30262 / loc_3026C (kid.asm:33849): halve `d6` and use the
    // shifted-out carry to pick the first or second word inside each mapping
    // pair, then mask `d5` even with `andi.w #$FFFE,d1` to index the row
    // pointer table. `a3` advances by `2 * (d6 >> 1)`, and the masked
    // `andi.w #$F,d0` half-row keeps the top-window defaults `d1=$E` /
    // `d2=-1` until the compare against `$2` branches to `loc_3028E`.
    let mapping_word_offset = usize::from(render_column & 1);
    let row_pointer = *row_pointer_table.get(usize::from(camera_tile_row & 0xFFFE))?;
    let half_column_index = render_column >> 1;
    let half_row_index = camera_tile_row >> 1;
    let half_row_mod16 = half_row_index & 0x000F;
    let branches_to_loc_3028_e = half_row_mod16 >= 0x0002;

    Some(BackgroundScrollStripeHorizontalLayoutSeedLoc3026C {
        theme_mapping_word_offset: mapping_word_offset,
        block_mapping_word_offset: mapping_word_offset,
        row_pointer: row_pointer.wrapping_add(half_column_index.wrapping_mul(2)),
        half_row_index,
        half_row_mod16,
        branches_to_loc_3028_e,
        copy_rows: (!branches_to_loc_3028_e).then_some(0x000E),
        initial_row_bias: (!branches_to_loc_3028_e).then_some(-1),
    })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct BackgroundScrollStripeHorizontalLowerWindowLoc3028E {
    pub(crate) copy_rows: u16,
    pub(crate) initial_row_bias: i16,
}

#[allow(non_snake_case)]
#[allow(dead_code)]
pub(crate) fn background_scroll_stripe_horizontal_lower_window_loc_3028E(
    half_row_mod16: u16,
) -> BackgroundScrollStripeHorizontalLowerWindowLoc3028E {
    // ASM loc_3028E (kid.asm:33855): once `loc_3026C` branches here, `d0`
    // already holds `(d5 >> 1) & $F`. The routine mirrors the vertical-path
    // arithmetic with `moveq #$F,d1; sub.w d0,d1`, then derives the starting
    // row bias from `moveq #$D,d2; sub.w d1,d2`, which simplifies to
    // `d2 = d0 - 2`.
    BackgroundScrollStripeHorizontalLowerWindowLoc3028E {
        copy_rows: 0x000F_u16.wrapping_sub(half_row_mod16),
        initial_row_bias: i16::try_from(half_row_mod16).unwrap_or(i16::MAX) - 0x0002,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct BackgroundScrollStripeHorizontalPassLoc30296 {
    pub(crate) vdp_command: u32,
    pub(crate) level_width_stride: u16,
    pub(crate) second_pass_vdp_command: Option<u32>,
}

#[allow(non_snake_case)]
#[allow(dead_code)]
pub(crate) fn background_scroll_stripe_horizontal_pass_loc_30296(
    camera_tile_row: u16,
    render_column: u16,
    level_width_tiles: u16,
    initial_row_bias: i16,
) -> BackgroundScrollStripeHorizontalPassLoc30296 {
    // ASM loc_30296 (kid.asm:33861): keep only the even tile row with
    // `andi.w #$FFFE,d3`, mask it to five bits, and shift into bits 7-11.
    // The horizontal edge contributes `(d6 & $3F) << 1`, then `ori.w #$4000`
    // and `swap` build the first VDP command. `move.w (Level_width_tiles).w,a4`
    // seeds the per-row stride for `sub_302F6`. When `d2` stays negative the
    // first pass falls straight through `loc_302E2` to the shared camera-latch
    // tail at `loc_302E8`; otherwise the wrap command clears only the row bits
    // with `andi.l #$F07EFFFF,d0` before the second `sub_302F6` call.
    let vram_word =
        (((camera_tile_row & 0x001E) << 7) | ((render_column & 0x003F) << 1)).wrapping_add(0x4000);
    let vdp_command = u32::from(vram_word) << 16;
    let second_pass_vdp_command = (initial_row_bias >= 0).then_some(vdp_command & 0xF07E_FFFF);

    BackgroundScrollStripeHorizontalPassLoc30296 {
        vdp_command,
        level_width_stride: level_width_tiles,
        second_pass_vdp_command,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct BackgroundScrollStripeVerticalColumnTransferLoc30354 {
    pub(crate) mapping_index: u16,
    pub(crate) tile_longword: u32,
}

#[allow(non_snake_case)]
#[allow(dead_code)]
pub(crate) fn background_scroll_stripe_vertical_column_transfer_loc_30354(
    layout_word: u16,
    mapping_mask: u16,
    primary_mapping_longword: u32,
) -> BackgroundScrollStripeVerticalColumnTransferLoc30354 {
    // ASM loc_30354 (kid.asm:33950): after `move.w (a3)+,d3` keeps the layout
    // word non-negative and `btst d2,d3` sees bit 8 clear, the loop masks the
    // layout index with `and.w d4,d3`, shifts it by three bytes of mapping-pair
    // stride, and copies the selected primary-table longword from `(a1,d3.w)`
    // straight to `(a6)` without ORing in the `$8000` priority bits.
    BackgroundScrollStripeVerticalColumnTransferLoc30354 {
        mapping_index: (layout_word & mapping_mask) << 3,
        tile_longword: primary_mapping_longword,
    }
}

#[allow(non_snake_case)]
#[allow(dead_code)]
pub(crate) fn background_scroll_stripe_vertical_column_transfer_loc_3036A(
    primary_mapping_longword: u32,
) -> u32 {
    // ASM loc_3036A (kid.asm:33962): after `btst d2,d3` sees bit 8 set, the
    // loop keeps the same `(a1,d3.w)` primary-table longword but `or.l d7,d0`
    // forces bit 15 in both 16-bit tile words by ORing `$80008000`.
    primary_mapping_longword | 0x8000_8000
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct BackgroundScrollStripeVerticalColumnTransferLoc3037C {
    pub(crate) mapping_index: u16,
    pub(crate) tile_longword: u32,
}

#[allow(non_snake_case)]
#[allow(dead_code)]
pub(crate) fn background_scroll_stripe_vertical_column_transfer_loc_3037C(
    layout_word: u16,
    mapping_mask: u16,
    alternate_mapping_longword: u32,
) -> BackgroundScrollStripeVerticalColumnTransferLoc3037C {
    // ASM loc_3037C (kid.asm:33972): when `move.w (a3)+,d3` leaves the layout
    // word negative, `bmi.s loc_3037C` skips the bit-8 test entirely, masks the
    // same layout index with `and.w d4,d3`, shifts it by the 8-byte mapping
    // stride, and copies the longword from the alternate block-mapping table
    // `(a2,d3.w)` straight to `(a6)`.
    BackgroundScrollStripeVerticalColumnTransferLoc3037C {
        mapping_index: (layout_word & mapping_mask) << 3,
        tile_longword: alternate_mapping_longword,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct BackgroundScrollStripeHorizontalRowTransferLoc3031A {
    pub(crate) tile_words: [u16; 2],
}

#[allow(non_snake_case)]
#[allow(dead_code)]
pub(crate) fn background_scroll_stripe_horizontal_row_transfer_loc_3031A(
    mapping_words: [u16; 2],
) -> BackgroundScrollStripeHorizontalRowTransferLoc3031A {
    // ASM loc_3031A (kid.asm:33918): after `btst d2,d3` sees bit 8 set, the
    // routine still indexes the same `(a1,d3.w)` mapping pair but ORs `$8000`
    // into both emitted words before streaming them to `(a6)`.
    BackgroundScrollStripeHorizontalRowTransferLoc3031A {
        tile_words: mapping_words.map(|word| word | 0x8000),
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum BackgroundScrollStripeHorizontalRowTransferSource {
    PrimaryMappings,
    AlternateMappings,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct BackgroundScrollStripeHorizontalRowTransferLoc302FE {
    pub(crate) source: BackgroundScrollStripeHorizontalRowTransferSource,
    pub(crate) mapping_index: u16,
    pub(crate) tile_words: [u16; 2],
    pub(crate) next_row_pointer: u16,
}

#[allow(non_snake_case)]
#[allow(dead_code)]
pub(crate) fn background_scroll_stripe_horizontal_row_transfer_loc_302FE(
    layout_word: u16,
    mapping_mask: u16,
    primary_mapping_words: [u16; 2],
    alternate_mapping_words: [u16; 2],
    row_stride: u16,
) -> BackgroundScrollStripeHorizontalRowTransferLoc302FE {
    // ASM loc_302FE (kid.asm:33904): read the current layout word from `(a3)`,
    // branch to `loc_30336` when bit 15 makes it negative, otherwise test bit
    // 8 with `btst d2,d3`. Clear bit 8 reuses the indexed primary mapping pair
    // unchanged; set bit 8 branches to `loc_3031A`; all three paths finish by
    // advancing `a3` with `add.w a4,a3` before the inclusive `dbf d1,loc_302FE`.
    let mapping_index = (layout_word & mapping_mask) << 3;
    let (source, tile_words) = if layout_word & 0x8000 != 0 {
        (
            BackgroundScrollStripeHorizontalRowTransferSource::AlternateMappings,
            alternate_mapping_words,
        )
    } else if layout_word & 0x0100 != 0 {
        (
            BackgroundScrollStripeHorizontalRowTransferSource::PrimaryMappings,
            background_scroll_stripe_horizontal_row_transfer_loc_3031A(primary_mapping_words)
                .tile_words,
        )
    } else {
        (
            BackgroundScrollStripeHorizontalRowTransferSource::PrimaryMappings,
            primary_mapping_words,
        )
    };

    BackgroundScrollStripeHorizontalRowTransferLoc302FE {
        source,
        mapping_index,
        tile_words,
        next_row_pointer: row_stride,
    }
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn fg_update_queue_cell_vram_word_loc_1614(
    cell_x: u16,
    cell_y: u16,
    camera_x_pixels: u16,
    camera_y_pixels: u16,
) -> Option<u16> {
    // ASM loc_1614 (kid.asm:2738): after loading `d4/d6` from
    // `Camera_X_pos>>4` and `Camera_X_pos>>4 + $14`, and `d5/d7` from
    // `Camera_Y_pos>>4` and `Camera_Y_pos>>4 + $E`, only cells whose `(d0,d1)`
    // tile coordinates lie within both inclusive ranges reach the VDP write
    // path. The packed VRAM word is `((d1 << 8) & $F00) + ((d0 << 2) & $7C) |
    // $4000`.
    let min_x = camera_x_pixels >> 4;
    let max_x = min_x.wrapping_add(0x0014);
    let min_y = camera_y_pixels >> 4;
    let max_y = min_y.wrapping_add(0x000E);
    if !(min_x..=max_x).contains(&cell_x) || !(min_y..=max_y).contains(&cell_y) {
        return None;
    }

    Some((((cell_y << 8) & 0x0F00) | ((cell_x << 2) & 0x007C)).wrapping_add(0x4000))
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn fg_update_queue_cell_mapping_longwords_loc_165E(
    cell_word: u16,
    theme_mappings: &[[u32; 2]],
    block_mappings: &[[u32; 2]],
) -> [u32; 2] {
    // ASM loc_1614 / loc_165E (kid.asm:2757-2783): after the visible-cell
    // gate, `bmi.s loc_165E` uses the sign bit of the queued layout word to
    // decide which mapping table feeds the two `(a6)` writes. Both branches
    // keep only the low byte, `lsl.w #3` to reach the 8-byte pair, and copy
    // two longwords from either `Addr_ThemeMappings` or `Block_Mappings`.
    let mapping_index = usize::from(cell_word & 0x00FF);
    if (cell_word as i16).is_negative() {
        block_mappings[mapping_index]
    } else {
        theme_mappings[mapping_index]
    }
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn fg_update_queue_culled_entry_advance_loc_167A(
    queue_tail_words_from_cell: &[u16],
) -> FgUpdateQueueCullAdvanceLoc167A {
    // ASM loc_167A (kid.asm:2786): once a queued cell falls outside the
    // inclusive camera tile window from loc_1614, `a0` already points at the
    // cell word. `addq.w #2,a0` skips exactly that one word, and
    // `move.w (a0)+,d0` reloads the next entry's X word for the `bpl.s
    // loc_1614` loop test.
    FgUpdateQueueCullAdvanceLoc167A {
        skipped_words: queue_tail_words_from_cell.len().min(1),
        next_x_word: queue_tail_words_from_cell.get(1).copied(),
    }
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
pub(crate) fn flagpole_fade_step_loc_C1D0(fade_progress: u16) -> FlagpoleFadeStepLocC1D0 {
    // ASM loc_C1D0 (kid.asm:15765): subtract $10 from $FFFFF876. If the
    // result is negative (bmi), the fade is complete and we branch to
    // loc_C1EE. Otherwise loc_C1DC reseeds both pending palette masks and
    // loops back.
    let new_progress = fade_progress.wrapping_sub(FLAGPOLE_FADE_STEP_LOC_C1D0);
    let done = (new_progress as i16) < 0;
    FlagpoleFadeStepLocC1D0 {
        fade_progress: new_progress,
        // ASM loc_C1DC (kid.asm:15770): palette masks written each non-final
        // frame. When done (bmi taken), the masks are NOT refreshed — the ROM
        // branches directly to loc_C1EE.
        pending_palette_0_mask: if done {
            0
        } else {
            FLAGPOLE_FADE_PALETTE_0_MASK_LOC_C1DC
        },
        pending_palette_1_mask: if done {
            0
        } else {
            FLAGPOLE_FADE_PALETTE_1_MASK_LOC_C1DC
        },
        done,
    }
}

#[allow(non_snake_case)]
#[allow(dead_code)]
pub(crate) fn flagpole_scoreboard_cursor_loc_C1EE(
    camera_x: u16,
    camera_y: u16,
) -> FlagpoleScoreboardCursorSubC254 {
    // ASM loc_C1EE (kid.asm:15776): convert camera pixel rectangle into
    // tile coordinates for the scoreboard reveal walk.
    let col_start = camera_x >> 4;
    let col_end = (camera_x.wrapping_add(FLAGPOLE_SCREEN_WIDTH_OFFSET_LOC_C1EE)) >> 4;
    let row_start = camera_y >> 4;
    let row_end = (camera_y.wrapping_add(FLAGPOLE_SCREEN_HEIGHT_OFFSET_LOC_C1EE)) >> 4;

    let col_span = col_end.wrapping_sub(col_start);
    let row_span = row_end.wrapping_sub(row_start);

    // $44(a5) = remaining_cols_in_row = col_span
    // $46(a5) = row_width (backup)    = col_span
    // $48(a5) = remaining_rows        = row_span
    // $4A(a5) = row_start_col         = col_start
    FlagpoleScoreboardCursorSubC254 {
        col: col_start,
        row: row_start,
        remaining_cols_in_row: col_span as i16,
        row_width: col_span as i16,
        remaining_rows: row_span as i16,
        row_start_col: col_start,
        budget: FLAGPOLE_SCOREBOARD_BUDGET_PER_FRAME_LOC_C21E,
    }
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

    // ASM: `move.w (a1),d3; bpl.s loc_C288` — bit 15 set means block tile,
    // uses unk_C2EA and subtracts 6 from budget.
    let (longwords, budget_delta) = if cell.has_block {
        (BLOCK_LONGWORDS, 6)
    } else {
        // ASM loc_C288 (kid.asm:15844): d3 positive (non-block). Loads
        // unk_C2C2 default, then `btst #$E,d3; bne.s loc_C296` tests bit 14.
        // No bit 14 → empty/hidden → unk_C2C2, budget_delta=2.
        match cell.collision {
            terrain::CollisionKind::Empty | terrain::CollisionKind::HiddenBlock => {
                (EMPTY_LONGWORDS, 2)
            }
            // ASM loc_C296 (kid.asm:15852): bit 14 set → collision variant.
            // `andi.w #$3000,d3; rol.w #7,d3` extracts bits 13:12 as a 0–3
            // index into unk_C2CA (8-byte stride), budget_delta=6.
            terrain::CollisionKind::SlopeUp => (COLLISION_LONGWORDS[0], 6),
            terrain::CollisionKind::SlopeDown => (COLLISION_LONGWORDS[1], 6),
            terrain::CollisionKind::Solid => (COLLISION_LONGWORDS[2], 6),
            terrain::CollisionKind::Spikes => (COLLISION_LONGWORDS[3], 6),
        }
    };

    // ASM loc_C2A4 (kid.asm:15859): common tail — copy 8 bytes of tile
    // attributes from selected table to (a0)+, then advance cursor:
    // `addq.w #1,d0; subq.w #1,$44(a5); bpl.s return_C2C0` stays on same
    // row, else wraps to next row via `$4A(a5)/$46(a5)` reset.
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

/// ASM loc_C2F2 (kid.asm:15925): scoreboard digit-clear animation entry.
/// `moveq #0,d0` sets the step counter, then falls through to `loc_C2F4`.
///
/// ASM loc_C2F4 (kid.asm:15928): per-step loop body — hibernates 4 frames,
/// then calls `sub_C346` for each of 3 digit groups (`$2FB`, `$409`, `$517`)
/// with `d2=$10E` rows per group. Loops 8 steps (`d0` 0–7), then deletes
/// the object.
///
/// The 3 base-column values correspond to tile indices for the three
/// scoreboard digit regions; `d2=$10E` = 270 tile rows per region.
#[allow(dead_code)]
pub(crate) const SCOREBOARD_DIGIT_CLEAR_STEP_COUNT_LOC_C2F2: u16 = 8;
#[allow(dead_code)]
pub(crate) const SCOREBOARD_DIGIT_CLEAR_HIBERNATE_FRAMES_LOC_C2F4: u16 = 4;
#[allow(dead_code)]
pub(crate) const SCOREBOARD_DIGIT_CLEAR_BASES_LOC_C2F4: [u16; 3] = [0x02FB, 0x0409, 0x0517];
#[allow(dead_code)]
pub(crate) const SCOREBOARD_DIGIT_CLEAR_ROW_COUNT_LOC_C2F4: u16 = 0x010E;

/// ASM loc_C326 (kid.asm:15946): bonus-stage digit-clear entry.
/// `moveq #0,d0` sets the step counter, then falls through to `loc_C328`.
///
/// ASM loc_C328 (kid.asm:15949): per-step loop body — same 4-frame hibernate
/// and 8-step loop as `loc_C2F4`, but calls `sub_C346` for only ONE digit
/// group (`$625`) with `d2=$48` rows (72). After 8 steps, deletes the object.
///
/// The bonus stage has a single digit region rather than the main scoreboard's
/// three; `$48` rows vs `$10E` reflects the smaller tile area.
#[allow(dead_code)]
pub(crate) const BONUS_DIGIT_CLEAR_BASE_LOC_C328: u16 = 0x0625;
#[allow(dead_code)]
pub(crate) const BONUS_DIGIT_CLEAR_ROW_COUNT_LOC_C328: u16 = 0x0048;

/// A single VDP VRAM-write command generated by `sub_C346`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)]
pub(crate) struct VdpVramWriteCmd {
    /// 32-bit value written to VDP control port 4(a6).
    pub(crate) control: u32,
    /// 32-bit value written to VDP data port (a6) — always 0 for digit clear.
    pub(crate) data: u32,
}

/// ASM sub_C346 (kid.asm:15963): compute VDP VRAM-write commands that clear
/// tile patterns for one scoreboard digit group.
///
/// ```text
/// sub_C346:
///   lsl.w  #3,d1          ; d1 = base * 8
///   add.w  d0,d1          ; d1 += step
///   lsl.w  #2,d1          ; d1 *= 4  → VRAM byte address
///   subq.w #1,d2          ; loop counter for dbf
/// loc_C34E:
///   rol.w  #2,d3          ; CD bits = (addr >> 14) & 3
///   ori.w  #$4000,d4      ; set VRAM-write flag
///   move.l d4,4(a6)       ; VDP control port
///   move.l #0,(a6)        ; VDP data port (clear)
///   addi.w #$20,d1        ; next row (+32 bytes = one 8×8 tile)
///   dbf    d2,loc_C34E
/// ```
#[allow(non_snake_case)]
#[allow(dead_code)]
pub(crate) fn scoreboard_digit_vdp_clear_sub_C346(
    base_col: u16,
    step: u16,
    row_count: u16,
) -> Vec<VdpVramWriteCmd> {
    // d1 = (base_col << 3 + step) << 2
    let mut addr = (base_col.wrapping_shl(3).wrapping_add(step)).wrapping_shl(2);
    let iterations = row_count as usize; // dbf loops (d2-1)+1 = d2 times after subq
    let mut cmds = Vec::with_capacity(iterations);
    for _ in 0..iterations {
        // ASM loc_C34E: build VDP control longword
        let cd_bits = (addr.rotate_left(2)) & 0x0003;
        let addr_field = (addr & 0x3FFF) | 0x4000;
        let control = (u32::from(addr_field) << 16) | u32::from(cd_bits);
        cmds.push(VdpVramWriteCmd { control, data: 0 });
        addr = addr.wrapping_add(0x0020);
    }
    cmds
}

/// ASM sub_C378 (kid.asm:15989): VDP VRAM tile-fade animation constants.
///
/// `loc_C38A` (kid.asm:15994): inner loop reads `$400` longwords ($1000 bytes)
/// from VRAM `$F000` into Decompression_Buffer.
///
/// `loc_C392` (kid.asm:15999): per-step loop — hibernates 2 frames, calls
/// `sub_C3DA` to mask palette nibbles in 4 columns, then writes the buffer
/// back to VRAM `$F000` via the unrolled `loc_C3BA` loop. Runs 16 steps
/// (`d0` 0–15), then deletes the object.
///
/// `loc_C3BA` (kid.asm:16009): unrolled write — 8 longwords per iteration,
/// `$80` iterations = `$400` longwords = `$1000` bytes.
#[allow(dead_code)]
pub(crate) const TILE_FADE_VRAM_ADDRESS_SUB_C378: u32 = 0xF000;
#[allow(dead_code)]
pub(crate) const TILE_FADE_BUFFER_LONGWORDS_LOC_C38A: u16 = 0x0400;
#[allow(dead_code)]
pub(crate) const TILE_FADE_STEP_COUNT_LOC_C392: u16 = 16;
#[allow(dead_code)]
pub(crate) const TILE_FADE_HIBERNATE_FRAMES_LOC_C392: u16 = 2;
#[allow(dead_code)]
pub(crate) const TILE_FADE_MASK_COLUMNS_SUB_C3DA: u16 = 0x0080;
#[allow(dead_code)]
pub(crate) const TILE_FADE_WRITE_ITERATIONS_LOC_C3BA: u16 = 0x0080;

/// ASM unk_C418 (kid.asm:16064): column-offset table for `sub_C3DA`.
/// Each entry encodes a byte offset within a tile row; the low bit selects
/// which nibble to mask (even → high nibble cleared via `$0F`, odd → low
/// nibble cleared via `$F0`).
#[allow(dead_code)]
pub(crate) const TILE_FADE_COLUMN_TABLE_UNK_C418: [u8; 16] = [
    0x00, 0x12, 0x02, 0x10, 0x09, 0x1B, 0x0B, 0x19, 0x01, 0x13, 0x03, 0x11, 0x08, 0x1A, 0x0A, 0x18,
];

/// ASM sub_C3F6 (kid.asm:16045): mask one column of 4bpp tile data in a
/// buffer. `d3` encodes byte offset × 2 + nibble selector (bit 0): even →
/// AND with `$0F` (clear high nibble), odd → AND with `$F0` (clear low
/// nibble). Applies the mask to `row_count` rows spaced `$20` bytes apart.
///
/// ```text
/// sub_C3F6:
///   move.w #$F0,d4
///   lsr.w  #1,d3       ; byte_offset = d3/2, carry = nibble select
///   bcs.s  loc_C406    ; odd → keep $F0 (clear low nibble)
///   move.w #$0F,d4     ; even → $0F (clear high nibble)
/// loc_C406:
///   add.w  d3,a0       ; offset into buffer
/// loc_C408:
///   and.b  d4,(a0)     ; mask the nibble
///   addi.w #$20,a0     ; next tile row
///   dbf    d2,loc_C408
/// ```
#[allow(non_snake_case)]
#[allow(dead_code)]
pub(crate) fn tile_fade_mask_column_sub_C3F6(buf: &mut [u8], col_selector: u16, row_count: u16) {
    let byte_offset = (col_selector >> 1) as usize;
    let mask: u8 = if col_selector & 1 != 0 { 0xF0 } else { 0x0F };
    let rows = row_count as usize;
    let mut pos = byte_offset;
    for _ in 0..rows {
        if let Some(b) = buf.get_mut(pos) {
            *b &= mask;
        }
        pos = pos.wrapping_add(0x20);
    }
}

/// ASM sub_C3DA (kid.asm:16029): mask 4 columns of tile data for one fade
/// step. Reads `unk_C418[step]` as the base column selector, then calls
/// `sub_C3F6` four times at offsets +0, +4, +$20, +$24 from that base.
///
/// Note: `sub_C3DA` falls through into `sub_C3F6` for the 4th call (no `rts`
/// before `sub_C3F6`), so there are 3 explicit `bsr.w` + 1 fallthrough.
///
/// ```text
/// sub_C3DA:
///   subq.w #1,d2         ; adjust for dbf
///   move.b unk_C418(pc,d0.w),d3
///   bsr.w  sub_C3F6      ; col = table[d0]
///   addq.w #4,d3         ; +4
///   bsr.w  sub_C3F6
///   addi.w #$1C,d3       ; +$1C (total +$20 from base)
///   bsr.w  sub_C3F6
///   addq.w #4,d3         ; +4 (total +$24 from base)
///   ; falls through to sub_C3F6
/// ```
#[allow(non_snake_case)]
#[allow(dead_code)]
pub(crate) fn tile_fade_mask_step_sub_C3DA(buf: &mut [u8], step: u16, row_count: u16) {
    let dbf_count = row_count.wrapping_sub(1);
    let base = u16::from(TILE_FADE_COLUMN_TABLE_UNK_C418[step as usize]);
    // ASM sub_C3DA calls sub_C3F6 four times, each restoring d2/d3/a0
    let col0 = base;
    let col1 = base.wrapping_add(4);
    let col2 = col1.wrapping_add(0x1C);
    let col3 = col2.wrapping_add(4);
    tile_fade_mask_column_sub_C3F6(buf, col0, dbf_count.wrapping_add(1));
    tile_fade_mask_column_sub_C3F6(buf, col1, dbf_count.wrapping_add(1));
    tile_fade_mask_column_sub_C3F6(buf, col2, dbf_count.wrapping_add(1));
    tile_fade_mask_column_sub_C3F6(buf, col3, dbf_count.wrapping_add(1));
}

/// ASM loc_C428 (kid.asm:16082): end-of-level sprite scatter object routine.
///
/// Each frame hibernates, then iterates over 10 sprite table entries (stride 8
/// bytes starting at `$FFFF0006`, the x-position word of each VDP sprite).
/// Sprites in the visible range (`$71`–`$1CF`) are nudged outward: those left
/// of screen centre (`$120`) are decremented, those right are incremented.
/// When no sprite remains in visible range, `d1` stays 0 and the object
/// self-deletes via `j_Delete_CurrentObject`.
///
/// ```text
/// loc_C428:
///   jsr    (j_Hibernate_Object_1Frame).w
///   lea    ($FFFF0006).l,a0
///   moveq  #9,d0
///   moveq  #0,d1
/// loc_C436:
///   move.w (a0),d2
///   cmpi.w #$70,d2
///   ble.s  loc_C452
///   cmpi.w #$1D0,d2
///   bge.s  loc_C452
///   moveq  #1,d1
///   cmpi.w #$120,d2
///   bge.s  loc_C450
///   subq.w #1,(a0)
///   bra.s  loc_C452
/// loc_C450:
///   addq.w #1,(a0)
/// loc_C452:
///   addq.w #8,a0
///   dbf    d0,loc_C436
///   tst.w  d1
///   bne.s  loc_C428
///   jmp    (j_Delete_CurrentObject).w
/// ```
#[allow(dead_code)]
pub(crate) const SPRITE_SCATTER_TABLE_X_OFFSET_LOC_C428: u32 = 0xFFFF_0006;
#[allow(dead_code)]
pub(crate) const SPRITE_SCATTER_ENTRY_COUNT_LOC_C428: u16 = 10;
#[allow(dead_code)]
pub(crate) const SPRITE_SCATTER_ENTRY_STRIDE_LOC_C428: u16 = 8;
/// ASM loc_C436: lower visible bound — `cmpi.w #$70,d2; ble.s loc_C452`.
#[allow(dead_code)]
pub(crate) const SPRITE_SCATTER_VISIBLE_MIN_LOC_C436: u16 = 0x0070;
/// ASM loc_C436: upper visible bound — `cmpi.w #$1D0,d2; bge.s loc_C452`.
#[allow(dead_code)]
pub(crate) const SPRITE_SCATTER_VISIBLE_MAX_LOC_C436: u16 = 0x01D0;
/// ASM loc_C436: screen centre threshold — `cmpi.w #$120,d2; bge.s loc_C450`.
/// Sprites left of centre are nudged left (subq), right of centre nudged right
/// (addq at loc_C450).
#[allow(dead_code)]
pub(crate) const SPRITE_SCATTER_CENTRE_LOC_C436: u16 = 0x0120;

/// Pure function modelling one pass of the sprite scatter loop (loc_C436–
/// loc_C452). Takes a mutable slice of 10 sprite x-positions and returns
/// whether any sprite was nudged (i.e. whether the outer loop at loc_C428
/// should repeat).
///
/// Each visible sprite (`$71`–`$1CF` exclusive of bounds) is nudged ±1 from
/// centre (`$120`). Out-of-range sprites are skipped.
#[allow(non_snake_case)]
#[allow(dead_code)]
pub(crate) fn sprite_scatter_step_loc_C436(x_positions: &mut [u16; 10]) -> bool {
    // ASM loc_C436: d1 tracks whether any sprite was nudged this pass.
    let mut any_nudged = false;
    for x in x_positions.iter_mut() {
        let val = *x;
        // ASM loc_C436: cmpi.w #$70,d2; ble.s loc_C452
        if val <= SPRITE_SCATTER_VISIBLE_MIN_LOC_C436 {
            continue;
        }
        // ASM loc_C436: cmpi.w #$1D0,d2; bge.s loc_C452
        if val >= SPRITE_SCATTER_VISIBLE_MAX_LOC_C436 {
            continue;
        }
        // ASM loc_C436: moveq #1,d1 — at least one sprite was in range
        any_nudged = true;
        if val >= SPRITE_SCATTER_CENTRE_LOC_C436 {
            // ASM loc_C450: addq.w #1,(a0)
            *x = val.wrapping_add(1);
        } else {
            // ASM loc_C436: subq.w #1,(a0)
            *x = val.wrapping_sub(1);
        }
    }
    any_nudged
}

/// ASM Object_EOL_Number (kid.asm:16114): end-of-level number display object.
///
/// loc_C466 (kid.asm:16118) is the init loop that allocates 7 gfx object slots
/// for the 7-digit number display. Each slot is configured with:
/// - gfx spec `$2000004` for `j_Allocate_GfxObjectSlot_a1`
/// - display flag `$13(a1)` = `$FF` (enabled via `st`)
/// - `$11(a1)` = 2 (priority/layer)
/// - `$10(a1)` = 1 (palette line)
/// - `$24(a1)` = `$517` (initial sprite pattern/VRAM offset)
///
/// Slot pointers are stored at `$44(a5)` through `$60(a5)` (7 × 4 bytes).
///
/// ```text
/// Object_EOL_Number:
///   lea    $44(a5),a4
///   moveq  #6,d7
/// loc_C466:
///   move.l #$2000004,a1
///   jsr    (j_Allocate_GfxObjectSlot_a1).w
///   move.l a1,(a4)+
///   st     $13(a1)
///   move.b #2,$11(a1)
///   move.b #1,$10(a1)
///   move.w #$517,$24(a1)
///   dbf    d7,loc_C466
/// ```
#[allow(dead_code)]
pub(crate) const EOL_NUMBER_DIGIT_COUNT_LOC_C466: u16 = 7;
#[allow(dead_code)]
pub(crate) const EOL_NUMBER_GFX_SPEC_LOC_C466: u32 = 0x0200_0004;
#[allow(dead_code)]
pub(crate) const EOL_NUMBER_DISPLAY_FLAG_LOC_C466: u8 = 0xFF;
#[allow(dead_code)]
pub(crate) const EOL_NUMBER_PRIORITY_LOC_C466: u8 = 2;
#[allow(dead_code)]
pub(crate) const EOL_NUMBER_PALETTE_LOC_C466: u8 = 1;
#[allow(dead_code)]
pub(crate) const EOL_NUMBER_INITIAL_PATTERN_LOC_C466: u16 = 0x0517;
/// Offset within the parent object (`a5`) where slot pointers start.
#[allow(dead_code)]
pub(crate) const EOL_NUMBER_SLOT_ARRAY_OFFSET_LOC_C466: u16 = 0x0044;

/// ASM loc_C48C (kid.asm:16128): maximum displayable value — `cmpi.l #9999999,d6`.
/// If `$60(a5)` exceeds this, the display loop clamps to this ceiling.
#[allow(dead_code)]
pub(crate) const EOL_NUMBER_MAX_VALUE_LOC_C48C: u32 = 9_999_999;

/// ASM loc_C48C (kid.asm:16128): number of digits in the display (7 powers-of-ten
/// entries from 1_000_000 down to 1, matching `cmpi.w #7,d7` at loc_C4E2+10).
#[allow(dead_code)]
pub(crate) const EOL_NUMBER_DIGIT_COUNT_LOC_C48C: u16 = 7;

/// ASM loc_C4E2 (kid.asm:16171): per-digit horizontal spacing —
/// `mulu.w #$C,d0` multiplies digit index by 12 pixels.
#[allow(dead_code)]
pub(crate) const EOL_NUMBER_DIGIT_SPACING_LOC_C4E2: u16 = 0x000C;

/// ASM PowersOfTen (kid.asm:16196): divisor table for 7-digit extraction.
/// loc_C4AE (kid.asm:16139) walks this array left-to-right, one entry per digit.
#[allow(dead_code)]
pub(crate) const POWERS_OF_TEN_LOC_C48C: [u32; 7] = [1_000_000, 100_000, 10_000, 1_000, 100, 10, 1];

#[cfg_attr(not(test), allow(dead_code))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct LevelSkipCompletionTimeDigitSpriteLocD468 {
    pub(crate) digit: u8,
    pub(crate) x_pos: u16,
    pub(crate) y_pos: u16,
    pub(crate) display_flag: u8,
    pub(crate) priority_layer: u8,
    pub(crate) palette_line: u8,
    pub(crate) vram_tile: u16,
}

/// ASM loc_D468 (kid.asm:16626): level-skip cheat EOL time digit sprites.
///
/// When `LevelSkip_Cheat` is non-zero, the ROM allocates three gfx objects
/// before the normal EOL text path. `d3` starts from the byte
/// `Level_completion_time`; each `divu.w #$A,d3` stores the remainder digit
/// through `Spr_EOL_Digits_Index`, then keeps the quotient for the next sprite.
/// ASM loc_D47E (kid.asm:16635): this is the allocation loop body; `d6=#2`
/// makes `dbf d6,loc_D47E` run three times while `subi.w #$C,d2` moves the
/// next digit sprite left.
/// Sprites are placed right-to-left at camera X + `$11C`, `$110`, `$104` and
/// share camera Y + `$D4`, display flag `$FF`, priority `$02`, palette `$01`,
/// and VRAM tile `$517`.
#[allow(dead_code)]
#[allow(non_snake_case)]
pub(crate) fn level_skip_completion_time_digit_sprites_loc_D468(
    level_skip_cheat: bool,
    level_completion_time: u8,
    camera_x_pos: u16,
    camera_y_pos: u16,
) -> Option<[LevelSkipCompletionTimeDigitSpriteLocD468; 3]> {
    if !level_skip_cheat {
        return None;
    }

    let y_pos = camera_y_pos.wrapping_add(0x00D4);
    let common = |digit: u8, x_offset: u16| LevelSkipCompletionTimeDigitSpriteLocD468 {
        digit,
        x_pos: camera_x_pos.wrapping_add(x_offset),
        y_pos,
        display_flag: 0xFF,
        priority_layer: 2,
        palette_line: 1,
        vram_tile: 0x0517,
    };

    Some([
        common(level_completion_time % 10, 0x011C),
        common((level_completion_time / 10) % 10, 0x0110),
        common(level_completion_time / 100, 0x0104),
    ])
}

/// ASM loc_C4AE (kid.asm:16139): determine what to display for one digit position.
///
/// Returns `Some(digit)` (0–9) if a visible digit should be shown, or `None` if
/// the position should be blanked (leading-zero suppression via loc_C4C8).
///
/// - `remainder`: current value being decomposed (d6)
/// - `power`: current power-of-ten divisor from `PowersOfTen` (d0)
/// - `digit_index`: 0-based position, 0=millions, 6=ones (d7)
/// - `leading_seen`: whether a non-zero digit has appeared yet (d2 != 0)
///
/// ```text
/// loc_C4AE:                           ; per-digit loop entry
///   move.l (a4)+,a0                   ; slot pointer
///   move.l (a1)+,d0                   ; power-of-ten
///   cmp.l  d0,d6                      ; remainder >= power?
///   bge.s  loc_C4CE                   ; yes → extract digit
///   cmpi.w #6,d7                      ; ones place?
///   beq.s  loc_C4C0                   ; yes → always show 0
///   tst.w  d2                         ; leading digit seen?
///   beq.s  loc_C4C8                   ; no → blank
/// loc_C4C0:                           ; show digit 0
///   move.w #(LnkTo_Spr_EOL_Digit0-Data_Index),$22(a0)
/// loc_C4C8:                           ; blank (leading zero suppressed)
///   clr.w  $22(a0)
/// loc_C4CE:                           ; extract digit via repeated subtraction
///   moveq  #1,d2                      ; mark leading digit seen
///   moveq  #-1,d1
/// loc_C4D2:
///   addq.w #1,d1
///   sub.l  d0,d6
///   bpl.s  loc_C4D2
///   add.l  d0,d6                      ; restore remainder
/// ```
#[allow(dead_code)]
#[allow(non_snake_case)]
pub(crate) fn eol_number_digit_loc_C4AE(
    remainder: u32,
    power: u32,
    digit_index: u16,
    leading_seen: bool,
) -> Option<u8> {
    // ASM loc_C4AE: cmp.l d0,d6; bge.s loc_C4CE
    if remainder >= power {
        // ASM loc_C4CE/loc_C4D2: repeated subtraction to extract digit
        let digit = (remainder / power) as u8;
        return Some(digit);
    }
    // ASM loc_C4AE+8: cmpi.w #6,d7; beq.s loc_C4C0
    if digit_index == 6 {
        // ASM loc_C4C0: ones place always shows 0
        return Some(0);
    }
    // ASM loc_C4AE+12: tst.w d2; beq.s loc_C4C8
    if leading_seen {
        // ASM loc_C4C0: trailing zero after significant digit
        Some(0)
    } else {
        // ASM loc_C4C8: leading zero suppressed — blank
        None
    }
}

/// ASM loc_C4CE/loc_C4D2 (kid.asm:16159–16166): extract digit value and
/// return `(digit, new_remainder)`. Uses the original repeated-subtraction
/// algorithm: `d1 = -1; loop: d1++, d6 -= d0, bpl loop; d6 += d0`.
#[allow(dead_code)]
#[allow(non_snake_case)]
pub(crate) fn eol_extract_digit_loc_C4CE(remainder: u32, power: u32) -> (u8, u32) {
    // ASM loc_C4CE: moveq #-1,d1
    // ASM loc_C4D2: addq.w #1,d1; sub.l d0,d6; bpl.s loc_C4D2; add.l d0,d6
    let digit = (remainder / power) as u8;
    let new_remainder = remainder % power;
    (digit, new_remainder)
}

/// ASM loc_C48C–loc_C4E2 (kid.asm:16128–16182): decompose a clamped number
/// into 7 display digits, with leading-zero suppression. Returns an array of
/// `Option<u8>` per digit position (millions → ones). `None` = blank.
#[allow(dead_code)]
#[allow(non_snake_case)]
pub(crate) fn eol_number_digits_loc_C48C(raw_value: u32) -> [Option<u8>; 7] {
    // ASM loc_C48C: cmpi.l #9999999,d6; ble.s loc_C4AE; move.l #9999999,d6
    let mut d6 = raw_value.min(EOL_NUMBER_MAX_VALUE_LOC_C48C);
    let mut result = [None; 7];
    let mut leading_seen = false;

    for (i, &power) in POWERS_OF_TEN_LOC_C48C.iter().enumerate() {
        let digit_index = i as u16;
        let digit = eol_number_digit_loc_C4AE(d6, power, digit_index, leading_seen);
        result[i] = digit;

        if d6 >= power {
            // ASM loc_C4CE: extract and advance remainder
            let (_, new_remainder) = eol_extract_digit_loc_C4CE(d6, power);
            d6 = new_remainder;
            leading_seen = true;
        }
    }

    result
}

/// ASM loc_C564 (kid.asm:16218): EOL number-variant sprite object setup.
/// Used for 2-player arrow indicators — palette line 1 (yellow), VRAM tile
/// `$409`. Differs from `Object_EOL_Text` which uses palette line 2 and
/// VRAM tile `$517`.
///
/// ```text
/// loc_C564:
///   move.l #$2000004,a3
///   jsr    (j_Load_GfxObjectSlot).w
///   move.b #1,palette_line(a3)     ; palette 1 (yellow)
///   move.w #$409,vram_tile(a3)     ; VRAM tile base
///   bra.s  loc_C592                ; fall through to common init
/// ```
#[allow(dead_code)]
pub(crate) const EOL_NUMBER_VARIANT_PALETTE_LOC_C564: u8 = 1;
#[allow(dead_code)]
pub(crate) const EOL_NUMBER_VARIANT_VRAM_TILE_LOC_C564: u16 = 0x0409;

/// ASM Object_EOL_Text (kid.asm:16227): standard EOL text sprite object.
/// Palette line 2, VRAM tile `$517`. Falls through to loc_C592 for common init.
///
/// ```text
/// Object_EOL_Text:
///   move.l #$2000004,a3
///   jsr    (j_Load_GfxObjectSlot).w
///   move.b #2,palette_line(a3)     ; palette 2
///   move.w #$517,vram_tile(a3)     ; VRAM tile base
/// ```
#[allow(dead_code)]
pub(crate) const EOL_TEXT_PALETTE_LOC_C57C: u8 = 2;
#[allow(dead_code)]
pub(crate) const EOL_TEXT_VRAM_TILE_LOC_C57C: u16 = 0x0517;

/// ASM loc_C592 (kid.asm:16233): common EOL object init shared by loc_C564
/// and Object_EOL_Text. Sets display flag, priority, and sprite address.
///
/// ```text
/// loc_C592:
///   st     $13(a3)                 ; display enabled ($FF)
///   move.b #1,priority(a3)         ; priority 1
///   move.w $44(a5),addroffset_sprite(a3)  ; sprite link from parent
/// ```
#[allow(dead_code)]
pub(crate) const EOL_COMMON_DISPLAY_FLAG_LOC_C592: u8 = 0xFF;
#[allow(dead_code)]
pub(crate) const EOL_COMMON_PRIORITY_LOC_C592: u8 = 1;

/// ASM loc_C5A2 (kid.asm:16238): EOL object position-update loop.
/// Each frame: hibernate, add velocity to position, add camera offset for
/// screen coords. When timer `$46(a5)` reaches zero, clear velocity and
/// continue looping (object drifts to rest then holds position).
///
/// ```text
/// loc_C5A2:
///   jsr    (j_Hibernate_Object_1Frame).w
///   move.w $64(a5),d0
///   add.w  $6C(a5),d0              ; x_pos += x_vel
///   move.w d0,$64(a5)
///   add.w  (Camera_X_pos).w,d0     ; screen x = local x + camera
///   move.w d0,x_pos(a3)
///   move.w $68(a5),d1
///   add.w  $70(a5),d1              ; y_pos += y_vel
///   move.w d1,$68(a5)
///   add.w  (Camera_Y_pos).w,d1     ; screen y = local y + camera
///   move.w d1,y_pos(a3)
///   subq.w #1,$46(a5)              ; timer--
///   bne.s  loc_C5A2                ; loop while timer > 0
///   clr.w  $6C(a5)                 ; x_vel = 0
///   clr.w  $70(a5)                 ; y_vel = 0
///   bra.s  loc_C5A2                ; continue looping at rest
/// ```
///
/// Pure model of one frame's position update (no hibernation/camera).
/// Returns `(new_x, new_y, new_timer, velocity_cleared)`.
#[allow(dead_code)]
#[allow(non_snake_case)]
pub(crate) fn eol_object_position_step_loc_C5A2(
    x: i16,
    y: i16,
    x_vel: i16,
    y_vel: i16,
    timer: u16,
) -> (i16, i16, u16, bool) {
    let new_x = x.wrapping_add(x_vel);
    let new_y = y.wrapping_add(y_vel);
    let new_timer = timer.wrapping_sub(1);
    if new_timer == 0 {
        // ASM loc_C5A2+26: clr.w $6C(a5); clr.w $70(a5)
        (new_x, new_y, 0, true)
    } else {
        (new_x, new_y, new_timer, false)
    }
}

/// ASM loc_C5DE (kid.asm:16257): EOL extra-life bounce display.
/// Allocates a gfx slot, sets palette 2, priority 1, VRAM `$517`,
/// `LnkTo_Spr_EOL_ExtraLife` sprite, then enters a bouncing loop
/// (loc_C60C) with initial params `d0=-$40, d1=0, d2=2, d3=4`.
///
/// ```text
/// loc_C5DE:
///   move.l #$2000004,a3
///   jsr    (j_Load_GfxObjectSlot).w
///   st     $13(a3)
///   move.b #2,palette_line(a3)
///   move.b #1,priority(a3)
///   move.w #$517,vram_tile(a3)
///   move.w #(LnkTo_Spr_EOL_ExtraLife-Data_Index),addroffset_sprite(a3)
///   moveq  #-$40,d0               ; x = -64
///   moveq  #0,d1                  ; y = 0
///   moveq  #2,d2                  ; x_vel = 2
///   moveq  #4,d3                  ; y_vel = 4
/// ```
#[allow(dead_code)]
pub(crate) const EOL_EXTRA_LIFE_PALETTE_LOC_C5DE: u8 = 2;
#[allow(dead_code)]
pub(crate) const EOL_EXTRA_LIFE_PRIORITY_LOC_C5DE: u8 = 1;
#[allow(dead_code)]
pub(crate) const EOL_EXTRA_LIFE_VRAM_TILE_LOC_C5DE: u16 = 0x0517;
#[allow(dead_code)]
pub(crate) const EOL_EXTRA_LIFE_INITIAL_X_LOC_C5DE: i16 = -0x40;
#[allow(dead_code)]
pub(crate) const EOL_EXTRA_LIFE_INITIAL_Y_LOC_C5DE: i16 = 0;
#[allow(dead_code)]
pub(crate) const EOL_EXTRA_LIFE_INITIAL_X_VEL_LOC_C5DE: i16 = 2;
#[allow(dead_code)]
pub(crate) const EOL_EXTRA_LIFE_INITIAL_Y_VEL_LOC_C5DE: i16 = 4;

/// ASM loc_C60C–loc_C644 (kid.asm:16270–16299): bouncing-ball loop for the
/// extra-life icon. Each axis bounces between bounds: x in [-8, $D0],
/// y in [8, $D8]. Velocity is negated on boundary contact.
///
/// ```text
/// loc_C60C:
///   add.w  d2,d0                   ; x += x_vel
///   cmpi.w #$FFF8,d0               ; x >= -8?
///   bge.s  loc_C61E
///   tst.w  d2; bpl.s loc_C61E      ; only negate if moving left
///   neg.w  d2
/// loc_C61E:
///   cmpi.w #$D0,d0                 ; x <= $D0?
///   ble.s  loc_C62A
///   tst.w  d2; bmi.s loc_C62A      ; only negate if moving right
///   neg.w  d2
/// loc_C62A:
///   add.w  d3,d1                   ; y += y_vel
///   cmpi.w #8,d1                   ; y >= 8?
///   bge.s  loc_C638
///   tst.w  d3; bpl.s loc_C638
///   neg.w  d3
/// loc_C638:
///   cmpi.w #$D8,d1                 ; y <= $D8?
///   ble.s  loc_C644
///   tst.w  d3; bmi.s loc_C644
///   neg.w  d3
/// ```
///
/// Pure model of one bounce frame. Returns `(new_x, new_y, new_x_vel, new_y_vel)`.
#[allow(dead_code)]
#[allow(non_snake_case)]
pub(crate) fn eol_extra_life_bounce_step_loc_C60C(
    x: i16,
    y: i16,
    x_vel: i16,
    y_vel: i16,
) -> (i16, i16, i16, i16) {
    let new_x = x.wrapping_add(x_vel);
    let mut xv = x_vel;
    // ASM loc_C60C+4: cmpi.w #$FFF8,d0; bge.s loc_C61E; tst.w d2; bpl.s loc_C61E; neg.w d2
    if new_x < -8 && xv < 0 {
        xv = -xv;
    }
    // ASM loc_C61E: cmpi.w #$D0,d0; ble.s loc_C62A; tst.w d2; bmi.s loc_C62A; neg.w d2
    if new_x > 0xD0 && xv > 0 {
        xv = -xv;
    }

    let new_y = y.wrapping_add(y_vel);
    let mut yv = y_vel;
    // ASM loc_C62A: cmpi.w #8,d1; bge.s loc_C638; tst.w d3; bpl.s loc_C638; neg.w d3
    if new_y < 8 && yv < 0 {
        yv = -yv;
    }
    // ASM loc_C638: cmpi.w #$D8,d1; ble.s loc_C644; tst.w d3; bmi.s loc_C644; neg.w d3
    if new_y > 0xD8 && yv > 0 {
        yv = -yv;
    }

    (new_x, new_y, xv, yv)
}

// ── loc_D0AA–loc_D0C0: game-over GFX cleanup + score screen setup ──────
//
// After loc_D074 deletes all objects and hibernates $24 frames for the
// loc_C2F2 fade-out, the ASM enters a GFX-object cleanup loop:
//
// loc_D0AA (kid.asm:16356): loop body — walk Addr_FirstGfxObjectSlot linked
//   list; for each GFX object that is NOT Addr_GfxObject_Kid, call
//   j_loc_1078 to unlink and free it.
// loc_D0BC (kid.asm:16364): loop tail — pop saved next-pointer from stack;
//   bne.s loc_D0AA continues while the list has entries.
//
// loc_D0C0 (kid.asm:16368): score screen setup entry (also reached directly
//   when Addr_FirstGfxObjectSlot is null).  Sequence:
//   1. Hibernate 1 frame
//   2. st ($FFFFFB49).w — set score-display flag
//   3. clr.w (Time_Frames).w
//   4. clr.b (MurderWall_flag).w
//   5. Allocate object → loc_C326 (bonus-stage digit-clear)
//   6. Allocate object → loc_C428 (sprite scatter)
//   7. Decompress Addr_TtlCrdLetters to VRAM $8120
//   8. Decompress LnkTo_unk_E06B5 to VRAM $A2E0
//   9. Palette copy loops (loc_D11C, loc_D12E)

/// ASM loc_D0C0 (kid.asm:16368): VRAM target for TtlCrdLetters decompress.
/// `move.w #$8120,d0; jsr (j_DecompressToVRAM).l`
#[allow(dead_code)]
pub(crate) const GAME_OVER_VRAM_LETTERS_LOC_D0C0: u16 = 0x8120;

/// ASM loc_D0C0 (kid.asm:16382): VRAM target for unk_E06B5 decompress.
/// `move.w #$A2E0,d0; jsr (j_DecompressToVRAM).l`
#[allow(dead_code)]
pub(crate) const GAME_OVER_VRAM_DIGITS_LOC_D0C0: u16 = 0xA2E0;

/// ASM loc_D0C0 (kid.asm:16351): hibernate $24 frames before GFX cleanup.
/// `move.w #$24,-(sp); jsr (j_Hibernate_Object).w`
#[allow(dead_code)]
pub(crate) const GAME_OVER_FADEOUT_HIBERNATE_FRAMES_LOC_D074: u16 = 0x24;

// ── loc_D11C / loc_D12E: score screen palette copy loops ────────────────

/// ASM loc_D11C (kid.asm:16389): word count for Pal_D02A → Palette_Buffer+$42.
/// `moveq #$E,d0` → dbf loops $E+1 = 15 times.  Writes 15 colours into
/// palette line 2 starting at colour index 1 (byte offset $42 = line 2 base
/// $40 + 2 bytes).
#[allow(dead_code)]
pub(crate) const GAME_OVER_PAL_LINE2_WORD_COUNT_LOC_D11C: u16 = 15;

/// ASM loc_D11C: destination byte offset within Palette_Buffer.
/// `lea (Palette_Buffer+$42).l,a1`
#[allow(dead_code)]
pub(crate) const GAME_OVER_PAL_LINE2_DEST_OFFSET_LOC_D11C: u16 = 0x42;

/// ASM loc_D12E (kid.asm:16396): word count for Pal_D048 → Palette_Buffer+$22.
/// `moveq #4,d0` → dbf loops $4+1 = 5 times.  Writes 5 colours into palette
/// line 1 starting at colour index 1 (byte offset $22 = line 1 base $20 + 2).
#[allow(dead_code)]
pub(crate) const GAME_OVER_PAL_LINE1_WORD_COUNT_LOC_D12E: u16 = 5;

/// ASM loc_D12E: destination byte offset within Palette_Buffer.
/// `lea (Palette_Buffer+$22).l,a1`
#[allow(dead_code)]
pub(crate) const GAME_OVER_PAL_LINE1_DEST_OFFSET_LOC_D12E: u16 = 0x22;

/// ASM loc_D142 (kid.asm:16403): game-over palette backup loop.
/// `moveq #$1F,d0; lea (Palette_Buffer).l,a0; lea (Palette_Buffer_2).l,a1;
/// loc_D142: move.l (a0)+,(a1)+; dbf d0,loc_D142`.
/// Copies 32 longwords (64 words = 128 bytes) from `Palette_Buffer` to
/// `Palette_Buffer_2` before the fade target buffer is cleared at `loc_D150`.
#[allow(dead_code)]
pub(crate) const GAME_OVER_PALETTE_BACKUP_COUNT_LOC_D142: u16 = 0x1F;

/// ASM loc_D150 (kid.asm:16409): game-over fade target clear loop.
/// `moveq #$1F,d0; lea (Palette_Buffer_3).l,a1;
/// loc_D150: clr.l (a1)+; dbf d0,loc_D150`.
/// Clears 32 longwords (64 words = 128 bytes) in `Palette_Buffer_3` before
/// seeding the fade progress word at `$FFFFF876`.
#[allow(dead_code)]
pub(crate) const GAME_OVER_PALETTE_CLEAR_COUNT_LOC_D150: u16 = 0x1F;

/// ASM loc_D718 (kid.asm:16825): end-of-level title-return palette backup loop.
/// `moveq #$1F,d0; lea (Palette_Buffer).l,a0; lea (Palette_Buffer_2).l,a1;
/// loc_D718: move.l (a0)+,(a1)+; dbf d0,loc_D718`.
/// Copies 32 longwords (64 words = 128 bytes) from `Palette_Buffer` to
/// `Palette_Buffer_2` before the fade target buffer is cleared.
#[allow(non_snake_case)]
#[allow(dead_code)]
pub(crate) fn eol_title_return_palette_backup_loc_D718(palette_buffer: [u16; 0x40]) -> [u16; 0x40] {
    palette_buffer
}

/// ASM loc_D726 (kid.asm:16831): end-of-level title-return fade target clear.
/// `moveq #$3F,d0; lea (Palette_Buffer_3).l,a0;
/// loc_D726: move.w #0,(a0)+; dbf d0,loc_D726`.
/// Clears all 64 words in `Palette_Buffer_3` before seeding `$FFFFF876` with
/// `$0100` for the title-return fade.
#[allow(non_snake_case)]
#[allow(dead_code)]
pub(crate) fn eol_title_return_palette_clear_loc_D726(palette_buffer_3: &mut [u16; 0x40]) {
    palette_buffer_3.fill(0);
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct EolTitleReturnFadeLocD734 {
    pub(crate) hibernate_frames: u16,
    pub(crate) final_progress: u16,
    pub(crate) palette_mask: u16,
}

/// ASM loc_D734 (kid.asm:16836): the title-return fade waits one frame,
/// writes `$7C7F` to the shared fade mask, subtracts `$10` from `$FFFFF876`,
/// and loops until the progress word reaches `$0080`.
#[allow(non_snake_case)]
#[allow(dead_code)]
pub(crate) fn eol_title_return_fade_loc_D734(initial_progress: u16) -> EolTitleReturnFadeLocD734 {
    let mut progress = initial_progress;
    let mut hibernate_frames = 0;

    loop {
        hibernate_frames += 1;
        progress = progress.wrapping_sub(0x0010);
        if progress == 0x0080 {
            break;
        }
    }

    EolTitleReturnFadeLocD734 {
        hibernate_frames,
        final_progress: progress,
        palette_mask: 0x7C7F,
    }
}

/// ASM loc_D84E (kid.asm:16932): final-level end-credits palette backup loop.
/// `moveq #$1F,d0` followed by `move.l (a0)+,(a1)+; dbf d0,loc_D84E`
/// copies 32 longwords, i.e. the full 128-byte `Palette_Buffer`, into
/// `Palette_Buffer_2`.
#[allow(non_snake_case)]
#[allow(dead_code)]
pub(crate) fn end_credits_palette_backup_loc_D84E(palette_buffer: [u16; 0x40]) -> [u16; 0x40] {
    palette_buffer
}

/// ASM loc_D85E (kid.asm:16939): final-level end-credits fade target clear.
/// `moveq #$3F,d0; moveq #0,d1` followed by
/// `move.w d1,(a0)+; dbf d0,loc_D85E` clears all 64 words in
/// `Palette_Buffer_3`, then seeds `$FFFFF876` with `$0100`.
#[allow(non_snake_case)]
#[allow(dead_code)]
pub(crate) fn end_credits_palette_clear_loc_D85E(palette_buffer_3: &mut [u16; 0x40]) -> u16 {
    palette_buffer_3.fill(0);
    0x0100
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct EndCreditsFadeStepLocD86C {
    pub(crate) hibernate_frames: u16,
    pub(crate) fade_progress: u16,
    pub(crate) pending_palette_0_mask: u32,
    pub(crate) pending_palette_1_mask: u32,
    pub(crate) enter_end_sequence: bool,
}

/// ASM loc_D86C (kid.asm:16946): final-level end-credits fade step.
/// Each loop hibernates one frame, subtracts one from `$FFFFF876`, and exits
/// to the end-sequence tail only after the word underflows negative.
/// Non-negative frames fall through the loop-back path, which writes `-1` to
/// both pending palette masks.
#[allow(non_snake_case)]
#[allow(dead_code)]
pub(crate) fn end_credits_fade_step_loc_D86C(
    fade_progress: u16,
    pending_palette_0_mask: u32,
    pending_palette_1_mask: u32,
) -> EndCreditsFadeStepLocD86C {
    let fade_progress = fade_progress.wrapping_sub(1);
    if (fade_progress as i16) < 0 {
        return EndCreditsFadeStepLocD86C {
            hibernate_frames: 1,
            fade_progress,
            pending_palette_0_mask,
            pending_palette_1_mask,
            enter_end_sequence: true,
        };
    }

    EndCreditsFadeStepLocD86C {
        hibernate_frames: 1,
        fade_progress,
        pending_palette_0_mask: u32::MAX,
        pending_palette_1_mask: u32::MAX,
        enter_end_sequence: false,
    }
}

/// ASM loc_D876 (kid.asm:16951): the final-level end-credits fade loop-back
/// writes `-1` to both pending palette mask longwords before returning to the
/// one-frame hibernate/subtract loop at `loc_D86C`.
#[allow(non_snake_case)]
#[allow(dead_code)]
pub(crate) fn end_credits_fade_loopback_loc_D876() -> (u32, u32) {
    (u32::MAX, u32::MAX)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct EndCreditsTailLocD882 {
    pub(crate) enters_end_sequence: bool,
    pub(crate) fade_colour_word: u16,
    pub(crate) game_mode: u16,
    pub(crate) tail_jump: &'static str,
}

/// ASM loc_D882 (kid.asm:16958): once the end-credits fade progress underflows,
/// set `$FFFFFBCE`, clear `$FFFFFBCC`, switch `Game_Mode` to `$30`, and jump
/// through the shared `loc_6E2` mode-reset tail.
#[allow(non_snake_case)]
#[allow(dead_code)]
pub(crate) fn end_credits_tail_loc_D882() -> EndCreditsTailLocD882 {
    EndCreditsTailLocD882 {
        enters_end_sequence: true,
        fade_colour_word: 0,
        game_mode: 0x0030,
        tail_jump: "loc_6E2",
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct ObjectCollisionFieldsLocD980 {
    pub(crate) object_addr: u32,
    pub(crate) field_36: u32,
    pub(crate) field_3a: u32,
    pub(crate) field_3e: u32,
}

/// ASM loc_D980 (kid.asm:17007): walk the object list from
/// `Addr_FirstObjectSlot`, skip the current object in `a5`, and clear the
/// three collision/contact longwords at `$36`, `$3A`, and `$3E` on every other
/// object before the later `Delete_AllButCurrentObject` tail.
#[allow(non_snake_case)]
#[allow(dead_code)]
pub(crate) fn flagpole_clear_other_object_collision_loc_D980(
    objects: &mut [ObjectCollisionFieldsLocD980],
    current_object_addr: u32,
) {
    for object in objects {
        if object.object_addr == current_object_addr {
            continue;
        }

        object.field_36 = 0;
        object.field_3a = 0;
        object.field_3e = 0;
    }
}

/// ASM loc_D984 (kid.asm:17010): loop over `Addr_FirstObjectSlot`, skip the
/// current object in `a5`, and clear `$36/$3A/$3E` on every other object.
#[allow(non_snake_case)]
#[allow(dead_code)]
pub(crate) fn end_credits_clear_other_object_collision_loc_D984(
    objects: &mut [ObjectCollisionFieldsLocD980],
    current_object_addr: u32,
) {
    flagpole_clear_other_object_collision_loc_D980(objects, current_object_addr);
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct EndCreditsCleanupSetupLocD99E {
    pub(crate) deletes_all_but_current_object: bool,
    pub(crate) sets_flag_fb6a: bool,
    pub(crate) vdp_register_commands: [u16; 2],
    pub(crate) clears_level_special_effects: bool,
    pub(crate) fade_object_callback: &'static str,
    pub(crate) hibernate_frames: u16,
    pub(crate) gfx_cleanup_entry_label: &'static str,
}

/// ASM loc_D99E (kid.asm:17022): after the object-field clear loop, delete all
/// non-current objects, set `$FFFFFB6A`, reset the scroll-plane VDP registers,
/// clear `Level_Special_Effects`, allocate the `loc_C2F2` fade worker, and
/// hibernate for `$24` frames before the GFX cleanup loop at `loc_D9D4`.
#[allow(non_snake_case)]
#[allow(dead_code)]
pub(crate) fn end_credits_cleanup_setup_loc_D99E() -> EndCreditsCleanupSetupLocD99E {
    EndCreditsCleanupSetupLocD99E {
        deletes_all_but_current_object: true,
        sets_flag_fb6a: true,
        vdp_register_commands: [0x8200, 0x8407],
        clears_level_special_effects: true,
        fade_object_callback: "loc_C2F2",
        hibernate_frames: 0x0024,
        gfx_cleanup_entry_label: "loc_D9D4",
    }
}

/// ASM loc_D9D4 (kid.asm:17036): walk `Addr_FirstGfxObjectSlot`, stash each
/// next pointer before deletion, preserve `Addr_GfxObject_Kid`, and call
/// `j_loc_1078` for every other GFX object.
#[allow(non_snake_case)]
#[allow(dead_code)]
pub(crate) fn end_credits_deleted_gfx_objects_loc_D9D4(
    gfx_object_addrs: &[u32],
    kid_gfx_object_addr: u32,
) -> Vec<u32> {
    gfx_object_addrs
        .iter()
        .copied()
        .filter(|addr| *addr != kid_gfx_object_addr)
        .collect()
}

/// ASM loc_D9E6 (kid.asm:17044): after either deleting the current GFX object
/// or skipping `Addr_GfxObject_Kid`, pop the saved next pointer and branch
/// back to `loc_D9D4` only when that pointer is non-zero.
#[allow(non_snake_case)]
#[allow(dead_code)]
pub(crate) fn end_credits_gfx_cleanup_next_loc_D9E6(
    saved_next_gfx_object_addr: u32,
) -> Option<u32> {
    std::num::NonZeroU32::new(saved_next_gfx_object_addr).map(std::num::NonZeroU32::get)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct EndCreditsHundredKTripSetupLocD9EA {
    pub(crate) hibernates_one_frame: bool,
    pub(crate) allocated_object_callback: &'static str,
    pub(crate) sets_flag_fb49: bool,
    pub(crate) palette_loop_word_count: usize,
    pub(crate) palette_loop_entry_label: &'static str,
}

/// ASM loc_D9EA (kid.asm:17048): after GFX cleanup, hibernate for one frame,
/// allocate the `sub_C378` worker, set `$FFFFFB49`, and seed `d0=$3F` with
/// `a0/a1/a2` pointing at the three palette buffers before falling into
/// `loc_DA16`.
#[allow(non_snake_case)]
#[allow(dead_code)]
pub(crate) fn end_credits_hundred_k_trip_setup_loc_D9EA() -> EndCreditsHundredKTripSetupLocD9EA {
    EndCreditsHundredKTripSetupLocD9EA {
        hibernates_one_frame: true,
        allocated_object_callback: "sub_C378",
        sets_flag_fb49: true,
        palette_loop_word_count: 0x40,
        palette_loop_entry_label: "loc_DA16",
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct EndCreditsHundredKTripPaletteSetupLocDA16 {
    pub(crate) palette_buffer_2: [u16; 0x40],
    pub(crate) palette_buffer_3: [u16; 0x40],
    pub(crate) fade_progress: u16,
}

/// ASM loc_DA16 (kid.asm:17059): copy each of the 64 `Palette_Buffer` words
/// into `Palette_Buffer_2`, subtract `$0888` from the copied word, store that
/// darkened word into `Palette_Buffer_3`, then seed `$FFFFF876` with `$0100`.
#[allow(non_snake_case)]
#[allow(dead_code)]
pub(crate) fn end_credits_hundred_k_trip_palette_setup_loc_DA16(
    palette_buffer: [u16; 0x40],
) -> EndCreditsHundredKTripPaletteSetupLocDA16 {
    let mut palette_buffer_3 = [0; 0x40];
    for (target_word, source_word) in palette_buffer_3.iter_mut().zip(palette_buffer) {
        *target_word = source_word.wrapping_sub(0x0888);
    }

    EndCreditsHundredKTripPaletteSetupLocDA16 {
        palette_buffer_2: palette_buffer,
        palette_buffer_3,
        fade_progress: 0x0100,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct EndCreditsHundredKTripFadeStepLocDA30 {
    pub(crate) hibernates_one_frame: bool,
    pub(crate) fade_progress: u16,
    pub(crate) countdown: u16,
    pub(crate) exits_to_loc_da4a: bool,
}

/// ASM loc_DA30 (kid.asm:17070): each Hundred K Trip fade frame hibernates
/// once, subtracts `$0011` from `$FFFFF876`, decrements `d0`, and exits to
/// `loc_DA4A` only when the post-decrement `d0` is zero.
#[allow(non_snake_case)]
#[allow(dead_code)]
pub(crate) fn end_credits_hundred_k_trip_fade_step_loc_DA30(
    fade_progress: u16,
    countdown: u16,
) -> EndCreditsHundredKTripFadeStepLocDA30 {
    let fade_progress = fade_progress.wrapping_sub(0x0011);
    let countdown = countdown.wrapping_sub(1);

    EndCreditsHundredKTripFadeStepLocDA30 {
        hibernates_one_frame: true,
        fade_progress,
        countdown,
        exits_to_loc_da4a: countdown == 0,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct EndCreditsHundredKTripFadeLoopbackLocDA3E {
    pub(crate) pending_palette_0_mask: u32,
    pub(crate) pending_palette_1_mask: u32,
    pub(crate) loop_target: &'static str,
}

/// ASM loc_DA3E (kid.asm:17076): write `-1` to both pending palette masks
/// (`$FFFFF888` and `$FFFFF88C`) before branching back to `loc_DA30` for the
/// next hibernating fade frame.
#[allow(non_snake_case)]
#[allow(dead_code)]
pub(crate) fn end_credits_hundred_k_trip_fade_loopback_loc_DA3E()
-> EndCreditsHundredKTripFadeLoopbackLocDA3E {
    EndCreditsHundredKTripFadeLoopbackLocDA3E {
        pending_palette_0_mask: u32::MAX,
        pending_palette_1_mask: u32::MAX,
        loop_target: "loc_DA30",
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct EndCreditsHundredKTripTeleportLocDA4A {
    pub(crate) score: u32,
    pub(crate) fade_colour_word: u16,
    pub(crate) level_load_requested: bool,
    pub(crate) player_start_x: u16,
    pub(crate) player_start_y: u16,
    pub(crate) current_level_id: u16,
    pub(crate) current_helmet: u16,
    pub(crate) continue_latch: bool,
    pub(crate) tail_subroutine: &'static str,
}

/// ASM loc_DA4A (kid.asm:17083): once the Hundred K Trip fade countdown ends,
/// seed the score and destination state, clear the current helmet, set the
/// continue latch, and branch through the shared `Teleport` tail.
#[allow(non_snake_case)]
#[allow(dead_code)]
pub(crate) fn end_credits_hundred_k_trip_teleport_loc_DA4A() -> EndCreditsHundredKTripTeleportLocDA4A
{
    EndCreditsHundredKTripTeleportLocDA4A {
        score: 1_000_001,
        fade_colour_word: 0,
        level_load_requested: true,
        player_start_x: 0x0020,
        player_start_y: 0x048F,
        current_level_id: 0x001F,
        current_helmet: 0,
        continue_latch: true,
        tail_subroutine: "Teleport",
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct LevelSpriteStripForwardSetupLocE650 {
    pub(crate) source_x: i16,
    pub(crate) d3_span_word: i16,
    pub(crate) tile_attr_mask: u16,
    pub(crate) map_probe_x: i16,
}

/// ASM loc_E650 (kid.asm:18321): derive the sprite-strip span from `d7`,
/// advance the forward-facing screen X by twice that span plus `$20`, and
/// reject the strip when the adjusted sprite X reaches `$1C0`.
#[allow(non_snake_case)]
#[allow(dead_code)]
pub(crate) fn level_sprite_strip_forward_setup_loc_E650(
    screen_x: i16,
    d7_width_seed: i16,
) -> Option<LevelSpriteStripForwardSetupLocE650> {
    let d3_span_word = d7_width_seed.wrapping_sub(2).wrapping_mul(6);
    let source_x = screen_x.wrapping_add(d3_span_word.wrapping_mul(2));
    if source_x >= 0x01C0 {
        return None;
    }

    Some(LevelSpriteStripForwardSetupLocE650 {
        source_x,
        d3_span_word,
        tile_attr_mask: 0,
        map_probe_x: source_x.wrapping_add(0x0020),
    })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct LevelSpriteStripReverseSetupLocE67E {
    pub(crate) source_x: i16,
    pub(crate) d3_span_word: i16,
    pub(crate) tile_attr_mask: u16,
    pub(crate) map_probe_x: i16,
}

/// ASM loc_E67E (kid.asm:18342): reversed strip setup subtracts the doubled
/// span and `$20` from `d0`, uses that adjusted X for `a1`, sets bit `$0800`
/// in the sprite tile word, and rejects positions at or left of `$60`.
#[allow(non_snake_case)]
#[allow(dead_code)]
pub(crate) fn level_sprite_strip_reverse_setup_loc_E67E(
    screen_x: i16,
    d3_span_word: i16,
) -> Option<LevelSpriteStripReverseSetupLocE67E> {
    let source_x = screen_x
        .wrapping_sub(d3_span_word.wrapping_mul(2))
        .wrapping_sub(0x0020);
    if source_x <= 0x0060 {
        return None;
    }

    Some(LevelSpriteStripReverseSetupLocE67E {
        source_x,
        d3_span_word,
        tile_attr_mask: 0x0800,
        map_probe_x: source_x,
    })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct LevelSpriteStripEntryLocE692 {
    pub(crate) y_pos: i16,
    pub(crate) size: u8,
    pub(crate) link: u8,
    pub(crate) tile_word: u16,
    pub(crate) x_pos: i16,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct LevelSpriteStripFirstWriteLocE692 {
    pub(crate) top_probe_y: i16,
    pub(crate) bottom_probe_y: i16,
    pub(crate) remaining_span_counter: u16,
    pub(crate) next_y: i16,
    pub(crate) next_link: u8,
    pub(crate) first_sprite: LevelSpriteStripEntryLocE692,
}

/// ASM loc_E692 (kid.asm:18350): seed `d5/d6` from the adjusted vertical
/// span, convert `d3` into the remaining piece counter, then write the leading
/// sprite entry at `(a2)+` with tile `$E676 | d4`.
#[allow(non_snake_case)]
#[allow(dead_code)]
pub(crate) fn level_sprite_strip_first_write_loc_E692(
    base_y: i16,
    link: u8,
    source_x: i16,
    d3_span_word: i16,
    tile_attr_mask: u16,
) -> LevelSpriteStripFirstWriteLocE692 {
    let bottom_probe_y = base_y.wrapping_add(d3_span_word).wrapping_add(0x0018);
    let top_probe_y = base_y.wrapping_sub(d3_span_word).wrapping_sub(0x0010);
    let remaining_span_counter = ((d3_span_word as u16) >> 1).wrapping_add(2);
    let next_y = top_probe_y.wrapping_add(0x0010);
    let next_link = link.wrapping_add(1);

    LevelSpriteStripFirstWriteLocE692 {
        top_probe_y,
        bottom_probe_y,
        remaining_span_counter,
        next_y,
        next_link,
        first_sprite: LevelSpriteStripEntryLocE692 {
            y_pos: top_probe_y,
            size: 0x0D,
            link: next_link,
            tile_word: 0xE676 | tile_attr_mask,
            x_pos: source_x,
        },
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct LevelSpriteStripMiddleLoopLocE6BC {
    pub(crate) middle_sprites: Vec<LevelSpriteStripEntryLocE692>,
    pub(crate) remainder_counter: u16,
    pub(crate) next_y: i16,
    pub(crate) next_link: u8,
}

/// ASM loc_E6BC (kid.asm:18368): subtract eight from the remaining strip
/// counter before each test. Non-negative results emit a full middle sprite
/// with size `$0F`, tile `$E67E | d4`, then advance `d1` by `$20`.
#[allow(non_snake_case)]
#[allow(dead_code)]
pub(crate) fn level_sprite_strip_middle_loop_loc_E6BC(
    mut remaining_span_counter: u16,
    mut y_pos: i16,
    mut link: u8,
    source_x: i16,
    tile_attr_mask: u16,
) -> LevelSpriteStripMiddleLoopLocE6BC {
    let mut middle_sprites = Vec::new();

    loop {
        remaining_span_counter = remaining_span_counter.wrapping_sub(8);
        if (remaining_span_counter as i16) < 0 {
            break;
        }

        link = link.wrapping_add(1);
        middle_sprites.push(LevelSpriteStripEntryLocE692 {
            y_pos,
            size: 0x0F,
            link,
            tile_word: 0xE67E | tile_attr_mask,
            x_pos: source_x,
        });
        y_pos = y_pos.wrapping_add(0x0020);
    }

    LevelSpriteStripMiddleLoopLocE6BC {
        middle_sprites,
        remainder_counter: remaining_span_counter,
        next_y: y_pos,
        next_link: link,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct LevelSpriteStripRemainderLocE6D8 {
    pub(crate) remainder_sprite: Option<LevelSpriteStripEntryLocE692>,
    pub(crate) next_y: i16,
    pub(crate) next_link: u8,
}

/// ASM loc_E6D8 (kid.asm:18382): a remainder counter of `$FFF8` means the
/// middle chunks ended exactly on a `$20` boundary. Other negative counters
/// are shifted left twice, added into `d1`, then emitted as one size `$0F`
/// sprite with tile `$E67E | d4`.
#[allow(non_snake_case)]
#[allow(dead_code)]
pub(crate) fn level_sprite_strip_remainder_loc_E6D8(
    remainder_counter: u16,
    y_pos: i16,
    link: u8,
    source_x: i16,
    tile_attr_mask: u16,
) -> LevelSpriteStripRemainderLocE6D8 {
    if remainder_counter == 0xFFF8 {
        return LevelSpriteStripRemainderLocE6D8 {
            remainder_sprite: None,
            next_y: y_pos,
            next_link: link,
        };
    }

    let adjusted_y = y_pos.wrapping_add(remainder_counter.wrapping_shl(2) as i16);
    let next_link = link.wrapping_add(1);

    LevelSpriteStripRemainderLocE6D8 {
        remainder_sprite: Some(LevelSpriteStripEntryLocE692 {
            y_pos: adjusted_y,
            size: 0x0F,
            link: next_link,
            tile_word: 0xE67E | tile_attr_mask,
            x_pos: source_x,
        }),
        next_y: adjusted_y.wrapping_add(0x0020),
        next_link,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct LevelSpriteStripTailLocE6F8 {
    pub(crate) tail_sprite: LevelSpriteStripEntryLocE692,
    pub(crate) next_link: u8,
}

/// ASM loc_E6F8 (kid.asm:18396): write the trailing cap sprite at the current
/// `d1`, using size `$0D`, the next sprite link, and tile `$F676 | d4`.
#[allow(non_snake_case)]
#[allow(dead_code)]
pub(crate) fn level_sprite_strip_tail_loc_E6F8(
    y_pos: i16,
    link: u8,
    source_x: i16,
    tile_attr_mask: u16,
) -> LevelSpriteStripTailLocE6F8 {
    let next_link = link.wrapping_add(1);

    LevelSpriteStripTailLocE6F8 {
        tail_sprite: LevelSpriteStripEntryLocE692 {
            y_pos,
            size: 0x0D,
            link: next_link,
            tile_word: 0xF676 | tile_attr_mask,
            x_pos: source_x,
        },
        next_link,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct LevelSpriteStripCollisionColumnLocE70A {
    pub(crate) addr_next_sprite_slot_offset: u16,
    pub(crate) number_sprites: u8,
    pub(crate) tile_column: Option<i16>,
    pub(crate) top_world_y: i16,
    pub(crate) bottom_world_y: i16,
    pub(crate) top_tile_row_lower_clamped: i16,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct LevelSpriteStripCollisionColumnInputLocE70A {
    pub(crate) addr_next_sprite_slot_offset: u16,
    pub(crate) number_sprites: u8,
    pub(crate) map_probe_screen_x: i16,
    pub(crate) top_probe_screen_y: i16,
    pub(crate) bottom_probe_screen_y: i16,
    pub(crate) camera_x: i16,
    pub(crate) camera_y: i16,
    pub(crate) level_width_blocks: i16,
}

/// ASM loc_E70A (kid.asm:18405): after the terrain strip sprites are written,
/// commit `a2/d2` to the sprite globals, convert `a1` from biased screen X to
/// a level tile column, reject columns outside `Level_width_blocks`, then
/// unbias the vertical probe span and clamp the top row at zero.
#[allow(non_snake_case)]
#[allow(dead_code)]
pub(crate) fn level_sprite_strip_collision_column_loc_E70A(
    input: LevelSpriteStripCollisionColumnInputLocE70A,
) -> LevelSpriteStripCollisionColumnLocE70A {
    let tile_column = input
        .map_probe_screen_x
        .wrapping_sub(0x0080)
        .wrapping_add(input.camera_x)
        >> 4;
    let top_world_y = input
        .top_probe_screen_y
        .wrapping_add(input.camera_y)
        .wrapping_sub(0x0080);
    let bottom_world_y = input
        .bottom_probe_screen_y
        .wrapping_add(input.camera_y)
        .wrapping_sub(0x0080);
    let top_tile_row = top_world_y >> 4;

    LevelSpriteStripCollisionColumnLocE70A {
        addr_next_sprite_slot_offset: input.addr_next_sprite_slot_offset,
        number_sprites: input.number_sprites,
        tile_column: (tile_column >= 0 && tile_column < input.level_width_blocks)
            .then_some(tile_column),
        top_world_y,
        bottom_world_y,
        top_tile_row_lower_clamped: top_tile_row.max(0),
    }
}

/// ASM loc_E738 (kid.asm:18423): clamp the already non-negative top terrain
/// probe row to `Level_height_blocks - 1` when it reaches or passes the level
/// height.
#[allow(non_snake_case)]
#[allow(dead_code)]
pub(crate) fn level_sprite_strip_top_row_upper_clamp_loc_E738(
    top_tile_row: i16,
    level_height_blocks: i16,
) -> i16 {
    if top_tile_row < level_height_blocks {
        top_tile_row
    } else {
        level_height_blocks.wrapping_sub(1)
    }
}

/// ASM loc_E744 (kid.asm:18429): convert the bottom terrain probe from world
/// pixels to a tile row with arithmetic shift, then clamp negative rows to
/// zero before the bottom upper-bound check.
#[allow(non_snake_case)]
#[allow(dead_code)]
pub(crate) fn level_sprite_strip_bottom_row_lower_clamp_loc_E744(bottom_world_y: i16) -> i16 {
    (bottom_world_y >> 4).max(0)
}

/// ASM loc_E74A (kid.asm:18434): cap the already non-negative bottom terrain
/// probe row to `Level_height_blocks - 1` before seeding the vertical collision
/// scan at `loc_E756`.
#[allow(non_snake_case)]
#[allow(dead_code)]
pub(crate) fn level_sprite_strip_bottom_row_upper_clamp_loc_E74A(
    bottom_tile_row: i16,
    level_height_blocks: i16,
) -> i16 {
    if bottom_tile_row < level_height_blocks {
        bottom_tile_row
    } else {
        level_height_blocks.wrapping_sub(1)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct LevelSpriteStripCollisionScanLocE756 {
    pub(crate) inclusive_row_count: u16,
    pub(crate) start_row: i16,
    pub(crate) tile_column: i16,
    pub(crate) collision_cell_addr: u16,
}

/// ASM loc_E756 (kid.asm:18440): convert the clamped top/bottom rows into the
/// inclusive `sub_E446` DBF counter, load the top row pointer from `$FFFF4A04`,
/// then advance by `tile_column * 2` bytes to seed the first collision cell.
#[allow(non_snake_case)]
#[allow(dead_code)]
pub(crate) fn level_sprite_strip_collision_scan_seed_loc_E756(
    top_tile_row: i16,
    bottom_tile_row: i16,
    tile_column: i16,
    row_pointer_words: &[u16],
) -> Option<LevelSpriteStripCollisionScanLocE756> {
    let row_index = usize::try_from(top_tile_row).ok()?;
    let row_start_addr = row_pointer_words.get(row_index).copied()?;

    Some(LevelSpriteStripCollisionScanLocE756 {
        inclusive_row_count: bottom_tile_row.wrapping_sub(top_tile_row) as u16,
        start_row: top_tile_row,
        tile_column,
        collision_cell_addr: row_start_addr.wrapping_add((tile_column as u16).wrapping_mul(2)),
    })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct LevelSpriteStripRightAdjacentColumnLocE786 {
    pub(crate) tile_column: i16,
    pub(crate) collision_cell_addr: u16,
}

/// ASM loc_E786 (kid.asm:18460): when `$FFFFFABE` selects the reversed strip
/// path, scan the right-adjacent terrain column by incrementing `d1`, rejecting
/// columns at `Level_width_blocks`, then advancing `a2` by one collision word.
#[allow(non_snake_case)]
#[allow(dead_code)]
pub(crate) fn level_sprite_strip_right_adjacent_column_loc_E786(
    tile_column: i16,
    collision_cell_addr: u16,
    level_width_blocks: i16,
) -> Option<LevelSpriteStripRightAdjacentColumnLocE786> {
    let adjacent_tile_column = tile_column.wrapping_add(1);
    if adjacent_tile_column >= level_width_blocks {
        return None;
    }

    Some(LevelSpriteStripRightAdjacentColumnLocE786 {
        tile_column: adjacent_tile_column,
        collision_cell_addr: collision_cell_addr.wrapping_add(2),
    })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum LevelSpriteStripExitLocE7A8 {
    ContinueToVisibleObjectSpritePass,
}

/// ASM loc_E7A8 (kid.asm:18479): all rejected strip paths branch directly to
/// `loc_E99C`, leaving the current sprite globals to seed the visible object
/// sprite pass.
#[allow(non_snake_case)]
#[allow(dead_code)]
pub(crate) fn level_sprite_strip_exit_loc_E7A8() -> LevelSpriteStripExitLocE7A8 {
    LevelSpriteStripExitLocE7A8::ContinueToVisibleObjectSpritePass
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct EyeclopsHardBeamAllocationLocE7AC {
    pub(crate) gfx_spec: u32,
    pub(crate) object_slot_request: u16,
    pub(crate) object_worker_addr: u32,
    pub(crate) stores_gfx_slot_in_faba: bool,
    pub(crate) stores_object_slot_in_gfx_backref: bool,
    pub(crate) diamonds_after: u16,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct EyeclopsHardBeamGateLocE7AC {
    pub(crate) masked_counter: u16,
    pub(crate) allocation: Option<EyeclopsHardBeamAllocationLocE7AC>,
}

/// ASM loc_E7AC (kid.asm:18483): clear the hard-beam marker bit from `d7`.
/// Non-zero low bits skip allocation; exactly `$8000` allocates the beam GFX
/// object pair, stores their cross-links through `$FFFFFABA`, then spends two
/// diamonds before falling through to `loc_E7E0`.
#[allow(non_snake_case)]
#[allow(dead_code)]
pub(crate) fn eyeclops_hard_beam_gate_loc_E7AC(
    beam_counter: u16,
    diamonds: u16,
) -> EyeclopsHardBeamGateLocE7AC {
    let masked_counter = beam_counter & 0x7FFF;
    let allocation = (masked_counter == 0).then_some(EyeclopsHardBeamAllocationLocE7AC {
        gfx_spec: 0x0300_0001,
        object_slot_request: 0xFFFF,
        object_worker_addr: 0x0000_EADE,
        stores_gfx_slot_in_faba: true,
        stores_object_slot_in_gfx_backref: true,
        diamonds_after: diamonds.wrapping_sub(2),
    });

    EyeclopsHardBeamGateLocE7AC {
        masked_counter,
        allocation,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct EyeclopsHardBeamZeroWidthSpritesLocE7E0 {
    pub(crate) sprites: [LevelSpriteStripEntryLocE692; 3],
    pub(crate) next_link: u8,
    pub(crate) collision_probe_screen_x: i16,
    pub(crate) collision_probe_screen_y: i16,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct EyeclopsHardBeamZeroWidthDirectionLocE7F4 {
    pub(crate) sprite_x: i16,
    pub(crate) collision_probe_screen_x: i16,
    pub(crate) tile_attr_mask: u16,
}

/// ASM loc_E7F4 (kid.asm:18508): the non-zero `$FFFFFABE` hard-beam latch
/// flips the zero-width strip by subtracting `$10` from `d0`, copying that
/// adjusted X into `d6`, and ORing `$0800` into each sprite tile word.
#[allow(non_snake_case)]
#[allow(dead_code)]
pub(crate) fn eyeclops_hard_beam_zero_width_direction_loc_E7F4(
    source_x: i16,
    direction_latch_nonzero: bool,
) -> EyeclopsHardBeamZeroWidthDirectionLocE7F4 {
    if direction_latch_nonzero {
        let adjusted_x = source_x.wrapping_sub(0x0010);
        EyeclopsHardBeamZeroWidthDirectionLocE7F4 {
            sprite_x: adjusted_x,
            collision_probe_screen_x: adjusted_x,
            tile_attr_mask: 0x0800,
        }
    } else {
        EyeclopsHardBeamZeroWidthDirectionLocE7F4 {
            sprite_x: source_x,
            collision_probe_screen_x: source_x.wrapping_add(0x0010),
            tile_attr_mask: 0,
        }
    }
}

/// ASM loc_E7FE (kid.asm:18513): write the zero-width hard-beam sprite triplet
/// at `d1 - 8`, `d1`, and `d1 + 8`, each with size `$04`, sequential links,
/// and tile words `$868E`, `$8692`, `$968E` ORed with the direction mask.
#[allow(non_snake_case)]
#[allow(dead_code)]
pub(crate) fn eyeclops_hard_beam_zero_width_writer_loc_E7FE(
    sprite_x: i16,
    base_y: i16,
    link: u8,
    tile_attr_mask: u16,
    collision_probe_screen_x: i16,
) -> EyeclopsHardBeamZeroWidthSpritesLocE7E0 {
    let collision_probe_screen_y = base_y.wrapping_sub(0x0004);
    let first_y = base_y.wrapping_sub(0x0008);
    let second_y = base_y;
    let third_y = base_y.wrapping_add(0x0008);
    let first_link = link.wrapping_add(1);
    let second_link = first_link.wrapping_add(1);
    let third_link = second_link.wrapping_add(1);

    EyeclopsHardBeamZeroWidthSpritesLocE7E0 {
        sprites: [
            LevelSpriteStripEntryLocE692 {
                y_pos: first_y,
                size: 4,
                link: first_link,
                tile_word: 0x868E | tile_attr_mask,
                x_pos: sprite_x,
            },
            LevelSpriteStripEntryLocE692 {
                y_pos: second_y,
                size: 4,
                link: second_link,
                tile_word: 0x8692 | tile_attr_mask,
                x_pos: sprite_x,
            },
            LevelSpriteStripEntryLocE692 {
                y_pos: third_y,
                size: 4,
                link: third_link,
                tile_word: 0x968E | tile_attr_mask,
                x_pos: sprite_x,
            },
        ],
        next_link: third_link,
        collision_probe_screen_x,
        collision_probe_screen_y,
    }
}

/// ASM loc_E7E0 (kid.asm:18497): for the zero-width hard beam, adjust `d0/d6`
/// from the direction latch, set `d5` to `d1 - 4`, then emit three size-4
/// sprites at `d1 - 8`, `d1`, and `d1 + 8`.
#[allow(non_snake_case)]
#[allow(dead_code)]
pub(crate) fn eyeclops_hard_beam_zero_width_sprites_loc_E7E0(
    source_x: i16,
    base_y: i16,
    link: u8,
    direction_latch_nonzero: bool,
) -> EyeclopsHardBeamZeroWidthSpritesLocE7E0 {
    let direction =
        eyeclops_hard_beam_zero_width_direction_loc_E7F4(source_x, direction_latch_nonzero);
    eyeclops_hard_beam_zero_width_writer_loc_E7FE(
        direction.sprite_x,
        base_y,
        link,
        direction.tile_attr_mask,
        direction.collision_probe_screen_x,
    )
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct EyeclopsHardBeamOneWidthSpritesLocE842 {
    pub(crate) sprites: [LevelSpriteStripEntryLocE692; 3],
    pub(crate) next_link: u8,
    pub(crate) collision_probe_screen_x: i16,
    pub(crate) collision_probe_screen_y: i16,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct EyeclopsHardBeamOneWidthDirectionLocE858 {
    pub(crate) sprite_x: i16,
    pub(crate) collision_probe_screen_x: i16,
    pub(crate) tile_attr_mask: u16,
}

/// ASM loc_E858 (kid.asm:18554): the non-zero `$FFFFFABE` one-width beam
/// branch rewinds `d0` by `$18`, copies that X into `d6`, and ORs `$0800`
/// into each emitted sprite tile word.
#[allow(non_snake_case)]
#[allow(dead_code)]
pub(crate) fn eyeclops_hard_beam_one_width_flipped_direction_loc_E858(
    source_x: i16,
) -> EyeclopsHardBeamOneWidthDirectionLocE858 {
    let adjusted_x = source_x.wrapping_sub(0x0018);
    EyeclopsHardBeamOneWidthDirectionLocE858 {
        sprite_x: adjusted_x,
        collision_probe_screen_x: adjusted_x,
        tile_attr_mask: 0x0800,
    }
}

/// ASM loc_E862 (kid.asm:18559): subtract six from `d1` for `d5`, write the
/// three one-width beam sprites at `d1 - 8`, `d1`, and `d1 + 8`, each with
/// size `$08` and tile words `$868E/$8692/$968E | d4`.
#[allow(non_snake_case)]
#[allow(dead_code)]
pub(crate) fn eyeclops_hard_beam_one_width_writer_loc_E862(
    sprite_x: i16,
    base_y: i16,
    link: u8,
    tile_attr_mask: u16,
    collision_probe_screen_x: i16,
) -> EyeclopsHardBeamOneWidthSpritesLocE842 {
    let collision_probe_screen_y = base_y.wrapping_sub(0x0006);
    let first_y = base_y.wrapping_sub(0x0008);
    let second_y = base_y;
    let third_y = base_y.wrapping_add(0x0008);
    let first_link = link.wrapping_add(1);
    let second_link = first_link.wrapping_add(1);
    let third_link = second_link.wrapping_add(1);

    EyeclopsHardBeamOneWidthSpritesLocE842 {
        sprites: [
            LevelSpriteStripEntryLocE692 {
                y_pos: first_y,
                size: 8,
                link: first_link,
                tile_word: 0x868E | tile_attr_mask,
                x_pos: sprite_x,
            },
            LevelSpriteStripEntryLocE692 {
                y_pos: second_y,
                size: 8,
                link: second_link,
                tile_word: 0x8692 | tile_attr_mask,
                x_pos: sprite_x,
            },
            LevelSpriteStripEntryLocE692 {
                y_pos: third_y,
                size: 8,
                link: third_link,
                tile_word: 0x968E | tile_attr_mask,
                x_pos: sprite_x,
            },
        ],
        next_link: third_link,
        collision_probe_screen_x,
        collision_probe_screen_y,
    }
}

/// ASM loc_E842 (kid.asm:18543): `d7 == 1` uses the one-width hard-beam
/// branch. The direction latch chooses either `d6=d0+$18` or `d0=d0-$18`,
/// then the shared write block emits three size-8 sprites at `d1 - 8`, `d1`,
/// and `d1 + 8`; other `d7` values fall through to the wider-beam path.
#[allow(non_snake_case)]
#[allow(dead_code)]
pub(crate) fn eyeclops_hard_beam_one_width_sprites_loc_E842(
    beam_counter: u16,
    source_x: i16,
    base_y: i16,
    link: u8,
    direction_latch_nonzero: bool,
) -> Option<EyeclopsHardBeamOneWidthSpritesLocE842> {
    if beam_counter != 1 {
        return None;
    }

    let (sprite_x, collision_probe_screen_x, tile_attr_mask) = if direction_latch_nonzero {
        let direction = eyeclops_hard_beam_one_width_flipped_direction_loc_E858(source_x);
        (
            direction.sprite_x,
            direction.collision_probe_screen_x,
            direction.tile_attr_mask,
        )
    } else {
        (source_x, source_x.wrapping_add(0x0018), 0)
    };

    Some(eyeclops_hard_beam_one_width_writer_loc_E862(
        sprite_x,
        base_y,
        link,
        tile_attr_mask,
        collision_probe_screen_x,
    ))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct EyeclopsHardBeamWideSetupLocE8A6 {
    pub(crate) span_px: i16,
    pub(crate) sprite_x: i16,
    pub(crate) collision_probe_screen_x: i16,
    pub(crate) tile_attr_mask: u16,
}

/// ASM loc_E8A6 (kid.asm:18589): wider hard beams convert `d7` to an 8px
/// span. Forward beams probe at `d0 + span + $10` while drawing from
/// `d0 + span - $10` and reject at `$1C0`; flipped beams draw and probe at
/// `d0 - span - $10`, set `$0800`, and reject at or left of `$60`.
#[allow(non_snake_case)]
#[allow(dead_code)]
pub(crate) fn eyeclops_hard_beam_wide_setup_loc_E8A6(
    beam_counter: u16,
    source_x: i16,
    direction_latch_nonzero: bool,
) -> Option<EyeclopsHardBeamWideSetupLocE8A6> {
    let span_px = beam_counter.wrapping_shl(3) as i16;
    if direction_latch_nonzero {
        let sprite_x = source_x.wrapping_sub(span_px).wrapping_sub(0x0010);
        if sprite_x <= 0x0060 {
            return None;
        }

        Some(EyeclopsHardBeamWideSetupLocE8A6 {
            span_px,
            sprite_x,
            collision_probe_screen_x: sprite_x,
            tile_attr_mask: 0x0800,
        })
    } else {
        let collision_probe_screen_x = source_x.wrapping_add(span_px).wrapping_add(0x0010);
        let sprite_x = source_x.wrapping_add(span_px).wrapping_sub(0x0010);
        if sprite_x >= 0x01C0 {
            return None;
        }

        Some(EyeclopsHardBeamWideSetupLocE8A6 {
            span_px,
            sprite_x,
            collision_probe_screen_x,
            tile_attr_mask: 0,
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct EyeclopsHardBeamWideFirstWriteLocE8DE {
    pub(crate) first_sprite: LevelSpriteStripEntryLocE692,
    pub(crate) middle_loop_counter: u16,
    pub(crate) next_y: i16,
    pub(crate) next_link: u8,
    pub(crate) collision_probe_screen_y: i16,
}

/// ASM loc_E8DE (kid.asm:18613): wider hard beams double `d7`, back `d1`
/// up by four plus that doubled width, store this as `d5`, then emit the
/// leading size-`$0C` sprite with tile `$868E | d4`.
#[allow(non_snake_case)]
#[allow(dead_code)]
pub(crate) fn eyeclops_hard_beam_wide_first_write_loc_E8DE(
    beam_counter: u16,
    base_y: i16,
    link: u8,
    sprite_x: i16,
    tile_attr_mask: u16,
) -> EyeclopsHardBeamWideFirstWriteLocE8DE {
    let doubled_counter = beam_counter.wrapping_mul(2) as i16;
    let first_y = base_y.wrapping_sub(0x0004).wrapping_sub(doubled_counter);
    let next_y = first_y.wrapping_add(0x0008);
    let next_link = link.wrapping_add(1);
    let middle_loop_counter = (beam_counter >> 1).wrapping_sub(1);

    EyeclopsHardBeamWideFirstWriteLocE8DE {
        first_sprite: LevelSpriteStripEntryLocE692 {
            y_pos: first_y,
            size: 0x0C,
            link: next_link,
            tile_word: 0x868E | tile_attr_mask,
            x_pos: sprite_x,
        },
        middle_loop_counter,
        next_y,
        next_link,
        collision_probe_screen_y: first_y,
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct EyeclopsHardBeamWideMiddleLocE902 {
    pub(crate) middle_sprites: Vec<LevelSpriteStripEntryLocE692>,
    pub(crate) odd_width_sprite: Option<LevelSpriteStripEntryLocE692>,
    pub(crate) next_y: i16,
    pub(crate) next_link: u8,
}

/// ASM loc_E902 (kid.asm:18631): the wide hard-beam middle loop writes one
/// size-`$0C` `$8692 | d4` sprite for each `DBF` pass, so the `(d7 >> 1) - 1`
/// counter emits `counter + 1` sprites. Odd `d7` then backs `d1` up four
/// pixels and writes one extra middle tile before the tail cap.
#[allow(non_snake_case)]
#[allow(dead_code)]
pub(crate) fn eyeclops_hard_beam_wide_middle_loc_E902(
    middle_loop_counter: u16,
    beam_counter: u16,
    mut y_pos: i16,
    mut link: u8,
    sprite_x: i16,
    tile_attr_mask: u16,
) -> EyeclopsHardBeamWideMiddleLocE902 {
    let middle_count = usize::from(middle_loop_counter).saturating_add(1);
    let mut middle_sprites = Vec::with_capacity(middle_count);

    for _ in 0..middle_count {
        link = link.wrapping_add(1);
        middle_sprites.push(LevelSpriteStripEntryLocE692 {
            y_pos,
            size: 0x0C,
            link,
            tile_word: 0x8692 | tile_attr_mask,
            x_pos: sprite_x,
        });
        y_pos = y_pos.wrapping_add(0x0008);
    }

    let odd_width_sprite = if beam_counter & 1 == 0 {
        None
    } else {
        y_pos = y_pos.wrapping_sub(0x0004);
        link = link.wrapping_add(1);
        let sprite = LevelSpriteStripEntryLocE692 {
            y_pos,
            size: 0x0C,
            link,
            tile_word: 0x8692 | tile_attr_mask,
            x_pos: sprite_x,
        };
        y_pos = y_pos.wrapping_add(0x0008);
        Some(sprite)
    };

    EyeclopsHardBeamWideMiddleLocE902 {
        middle_sprites,
        odd_width_sprite,
        next_y: y_pos,
        next_link: link,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct EyeclopsHardBeamWideTailLocE936 {
    pub(crate) tail_sprite: LevelSpriteStripEntryLocE692,
    pub(crate) next_link: u8,
}

/// ASM loc_E936 (kid.asm:18653): the wide hard-beam strip always finishes at
/// the current `d1` with one size-`$0C` tail cap using tile `$968E | d4` and
/// the next sprite-table link.
#[allow(non_snake_case)]
#[allow(dead_code)]
pub(crate) fn eyeclops_hard_beam_wide_tail_loc_E936(
    y_pos: i16,
    link: u8,
    sprite_x: i16,
    tile_attr_mask: u16,
) -> EyeclopsHardBeamWideTailLocE936 {
    let next_link = link.wrapping_add(1);

    EyeclopsHardBeamWideTailLocE936 {
        tail_sprite: LevelSpriteStripEntryLocE692 {
            y_pos,
            size: 0x0C,
            link: next_link,
            tile_word: 0x968E | tile_attr_mask,
            x_pos: sprite_x,
        },
        next_link,
    }
}

/// ASM loc_E972 (kid.asm:18676): after the hard-beam collision probe tail,
/// the ROM masks off bit 0 of `d7`, doubles it, caps the result at `$20`,
/// then stores `$1160 + d7` into the beam object's `$22` mapping word.
#[allow(non_snake_case)]
#[allow(dead_code)]
pub(crate) fn eyeclops_hard_beam_object_mapping_loc_E972(beam_counter: u16) -> u16 {
    let doubled_even_counter = (beam_counter & 0xFFFE).wrapping_mul(2);
    0x1160 + doubled_even_counter.min(0x0020)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct TimedBlockVisibleSlotLocE9CA {
    pub(crate) object_id_index: u16,
    pub(crate) sprite: LevelSpriteStripEntryLocE692,
}

/// ASM loc_E9CA (kid.asm:18709): visible timed-block slots render only for
/// object IDs `$127..=$12C` whose block column/row lie inside the camera tile
/// window. The six-entry table offsets the end caps by four pixels and gives
/// the middle four entries VDP size byte `$05`.
#[allow(non_snake_case)]
#[allow(dead_code)]
pub(crate) fn timed_block_visible_slot_sprite_loc_E9CA(
    object_id: u16,
    block_col: i16,
    block_row: i16,
    camera_x: i16,
    camera_y: i16,
    number_sprites: u8,
) -> Option<TimedBlockVisibleSlotLocE9CA> {
    const POS_OFFSETS_LOC_EA26: [i16; 6] = [4, 0, 0, 0, 0, 4];
    const SIZE_BYTES_LOC_EA29: [u8; 6] = [0, 5, 5, 5, 5, 0];

    let object_id_index = object_id.wrapping_sub(0x0127);
    if (object_id_index as i16) < 0 || object_id_index >= 6 {
        return None;
    }

    let min_col = (camera_x >> 4).wrapping_sub(4);
    let max_col = min_col.wrapping_add(0x001C);
    if block_col < min_col || block_col > max_col {
        return None;
    }

    let min_row = (camera_y >> 4).wrapping_sub(4);
    let max_row = min_row.wrapping_add(0x001C);
    if block_row < min_row || block_row > max_row {
        return None;
    }

    let table_index = usize::from(object_id_index);
    let pos_offset = POS_OFFSETS_LOC_EA26[table_index];
    let screen_x = (block_col << 4)
        .wrapping_sub(camera_x)
        .wrapping_add(0x0080)
        .wrapping_add(pos_offset);
    let screen_y = (block_row << 4)
        .wrapping_sub(camera_y)
        .wrapping_add(0x0080)
        .wrapping_add(pos_offset);

    Some(TimedBlockVisibleSlotLocE9CA {
        object_id_index,
        sprite: LevelSpriteStripEntryLocE692 {
            y_pos: screen_y,
            size: SIZE_BYTES_LOC_EA29[table_index],
            link: number_sprites.wrapping_add(1),
            tile_word: 0x8692,
            x_pos: screen_x,
        },
    })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct EndCreditsHundredKTripTileUploadLocDA7A {
    pub(crate) hibernate_frames_per_iteration: u16,
    pub(crate) iterations: usize,
    pub(crate) upload_pairs: [(u16, u16); 2],
    pub(crate) deletes_current_object: bool,
}

/// ASM loc_DA7A (kid.asm:17096): hibernate the object for four frames, call
/// `sub_C346` with `(d1,d2)=($00BC,$0644)` and `($0780,$0080)`, repeat until
/// the post-increment counter reaches eight, then delete the current object.
#[allow(non_snake_case)]
#[allow(dead_code)]
pub(crate) fn end_credits_hundred_k_trip_tile_upload_loc_DA7A()
-> EndCreditsHundredKTripTileUploadLocDA7A {
    EndCreditsHundredKTripTileUploadLocDA7A {
        hibernate_frames_per_iteration: 4,
        iterations: 8,
        upload_pairs: [(0x00BC, 0x0644), (0x0780, 0x0080)],
        deletes_current_object: true,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct PlatformCeilingProbeHitboxLocDAD6 {
    pub(crate) left_px: i32,
    pub(crate) right_px: i32,
    pub(crate) top_px: i32,
    pub(crate) bottom_px: i32,
    pub(crate) half_width_px: i32,
    pub(crate) top_offset_px: i32,
}

fn platform_probe_rom_half_width_sub_daa6(current_form: PlayerForm) -> i32 {
    // ASM sub_DAA6 (kid.asm:17116): double Current_Helmet and read the first
    // byte of each `unk_B408` pair. Micromax is `$04` here even though the
    // current native movement helper still carries a broader TODO value.
    match current_form {
        PlayerForm::TheKid => 5,
        PlayerForm::Skycutter => 10,
        PlayerForm::Cyclone
        | PlayerForm::RedStealth
        | PlayerForm::Eyeclops
        | PlayerForm::IronKnight
        | PlayerForm::Berzerker
        | PlayerForm::Maniaxe => 7,
        PlayerForm::Juggernaut => 23,
        PlayerForm::Micromax => 4,
    }
}

/// ASM loc_DAD6 (kid.asm:17130): build the Kid-vs-platform ceiling probe
/// rectangle from `y_pos(a3)`, `x_pos(a3)`, the `unk_B408` half-width, and
/// `d5=$0F` for Juggernaut/Micromax or `$1F` for every other helmet.
#[allow(non_snake_case)]
#[allow(dead_code)]
pub(crate) fn platform_ceiling_probe_hitbox_loc_DAD6(
    current_form: PlayerForm,
    x_pos_px: i32,
    y_pos_px: i32,
) -> PlatformCeilingProbeHitboxLocDAD6 {
    let half_width_px = platform_probe_rom_half_width_sub_daa6(current_form);
    let top_offset_px = match current_form {
        PlayerForm::Juggernaut | PlayerForm::Micromax => 0x0F,
        _ => 0x1F,
    };

    PlatformCeilingProbeHitboxLocDAD6 {
        left_px: x_pos_px - half_width_px,
        right_px: x_pos_px + half_width_px,
        top_px: y_pos_px - top_offset_px,
        bottom_px: y_pos_px,
        half_width_px,
        top_offset_px,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct PlatformBoundsLocDAE8 {
    pub(crate) left_px: i32,
    pub(crate) top_px: i32,
    pub(crate) width_px: i32,
    pub(crate) height_px: i32,
}

/// ASM loc_DAE8 (kid.asm:17139): walk active platform slots until the zero
/// link sentinel, skipping entries only when one of the signed `bgt`/`blt`
/// separation checks proves the platform rectangle cannot overlap Kid's
/// `loc_DAD6` probe rectangle.
/// ASM loc_DB1A (kid.asm:17160): if the scan reaches the zero link sentinel,
/// restore the saved registers and return `d7=0`; native returns `None`.
#[allow(non_snake_case)]
#[allow(dead_code)]
pub(crate) fn first_platform_ceiling_overlap_loc_DAE8(
    kid_probe: PlatformCeilingProbeHitboxLocDAD6,
    platform_slots: &[Option<PlatformBoundsLocDAE8>],
) -> Option<usize> {
    for (slot_index, platform_slot) in platform_slots.iter().enumerate() {
        let platform = (*platform_slot)?;
        if platform.left_px > kid_probe.right_px {
            continue;
        }

        let platform_right_px = platform.left_px + platform.width_px;
        if platform_right_px < kid_probe.left_px {
            continue;
        }

        if platform.top_px > kid_probe.bottom_px {
            continue;
        }

        let platform_bottom_px = platform.top_px + platform.height_px;
        if platform_bottom_px < kid_probe.top_px {
            continue;
        }

        return Some(slot_index);
    }

    None
}

/// ASM loc_D770 (kid.asm:16855): each EOL score-counter frame loads the
/// remaining amount from the active number object's `$60` field and clamps the
/// transfer chunk to `d2` before falling through to `loc_D77E`.
#[allow(non_snake_case)]
#[allow(dead_code)]
pub(crate) fn eol_score_counter_transfer_chunk_loc_D770(
    remaining_amount: u32,
    transfer_chunk: u32,
) -> u32 {
    remaining_amount.min(transfer_chunk)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct EolScoreCounterApplyLocD77E {
    pub(crate) score: u32,
    pub(crate) awards_extra_life: bool,
}

/// ASM loc_D79E (kid.asm:16872): compare old/new `divu.w #50000`
/// quotients after the EOL score-counter add. Any quotient change awards one
/// life and spawns one `loc_C5DE` extra-life icon.
#[allow(non_snake_case)]
#[allow(dead_code)]
pub(crate) fn eol_score_counter_awards_life_loc_D79E(old_score: u32, new_score: u32) -> bool {
    old_score / SCORE_EXTRA_LIFE_INTERVAL != new_score / SCORE_EXTRA_LIFE_INTERVAL
}

/// ASM loc_D77E (kid.asm:16862): add the current EOL transfer chunk to
/// `(Score).w`, clamp to `9_999_999`, then compare old/new `divu #50000`
/// quotients. The EOL loop issues one `addq.w #1,(Number_Lives).w` when the
/// quotient changes, not one life per crossed boundary.
#[allow(non_snake_case)]
#[allow(dead_code)]
pub(crate) fn eol_score_counter_apply_loc_D77E(
    current_score: u32,
    transfer_chunk: u32,
) -> EolScoreCounterApplyLocD77E {
    let score = current_score.saturating_add(transfer_chunk).min(SCORE_CAP);
    let awards_extra_life = eol_score_counter_awards_life_loc_D79E(current_score, score);

    EolScoreCounterApplyLocD77E {
        score,
        awards_extra_life,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct EolScoreCounterDrainLocD7CE {
    pub(crate) displayed_score: u32,
    pub(crate) remaining_amount: u32,
    pub(crate) active_number_count: u16,
    pub(crate) advances_to_next_number: bool,
    pub(crate) all_numbers_complete: bool,
}

/// ASM loc_D7CE (kid.asm:16887): mirror live `(Score).w` into the score
/// display object's `$60`, subtract the full transfer chunk from the active
/// EOL number object's `$60`, and advance to the next number when the signed
/// result is not positive.
#[allow(non_snake_case)]
#[allow(dead_code)]
pub(crate) fn eol_score_counter_drain_current_number_loc_D7CE(
    score: u32,
    remaining_amount: u32,
    transfer_chunk: u32,
    active_number_count: u16,
) -> EolScoreCounterDrainLocD7CE {
    if remaining_amount > transfer_chunk {
        return EolScoreCounterDrainLocD7CE {
            displayed_score: score,
            remaining_amount: remaining_amount - transfer_chunk,
            active_number_count,
            advances_to_next_number: false,
            all_numbers_complete: false,
        };
    }

    let active_number_count = active_number_count.saturating_sub(1);
    EolScoreCounterDrainLocD7CE {
        displayed_score: score,
        remaining_amount: 0,
        active_number_count,
        advances_to_next_number: active_number_count != 0,
        all_numbers_complete: active_number_count == 0,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct EolScoreCounterLoopTailLocD7E4 {
    pub(crate) frame_delay: u16,
    pub(crate) transfer_chunk: u32,
    pub(crate) chunk_quadrupled: bool,
}

/// ASM loc_D7E4 (kid.asm:16896): after each score-counter transfer, decrement
/// the frame-delay word `d1`. A non-zero result loops with the same chunk;
/// zero resets `d1` to `$78` and quadruples `d2` before looping.
#[allow(non_snake_case)]
#[allow(dead_code)]
pub(crate) fn eol_score_counter_loop_tail_loc_D7E4(
    frame_delay: u16,
    transfer_chunk: u32,
) -> EolScoreCounterLoopTailLocD7E4 {
    let frame_delay = frame_delay.wrapping_sub(1);
    if frame_delay != 0 {
        return EolScoreCounterLoopTailLocD7E4 {
            frame_delay,
            transfer_chunk,
            chunk_quadrupled: false,
        };
    }

    EolScoreCounterLoopTailLocD7E4 {
        frame_delay: 0x0078,
        transfer_chunk: transfer_chunk.wrapping_mul(4),
        chunk_quadrupled: true,
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

    // ASM loc_1C868: loop entry, processes each of 16 palette words
    // ASM loc_1C876: mid-channel processing (after low channel)
    // ASM loc_1C880: high-channel processing (after mid channel)
    // ASM loc_1C88A: recombination point after high-channel adjustment,
    // where `or.w d4,d5; or.w d5,d6` merges all channels.
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
    ///
    /// ASM loc_8D2 (kid.asm:992): entry after player select — `tst.b
    ///   (Current_player); beq.w loc_8D2` selects Player_1_Lives (P1) or
    ///   Player_2_Lives (P2) as base address a0.  Then `moveq #$A,d0; lea
    ///   (Number_Lives),a1`.
    ///
    /// ASM loc_8D8 (kid.asm:996): copy loop body — `move.w (a1)+,(a0)+;
    ///   dbf d0,loc_8D8; rts`.  Copies 11 words (lives, hitpoints, diamonds,
    ///   level ID, helmet, continues, extra_hitpoint_slots, score[long]) from
    ///   working Number_Lives into the selected player's save slot.
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
    ///
    /// ASM loc_8B4 (kid.asm:972): fall-through after `tst.b (Current_player);
    ///   beq loc_8B4` selects Player_1_Lives or Player_2_Lives base address.
    ///   Then `moveq #$A,d0; lea (Number_Lives),a1; move.w (a0)+,(a1)+;
    ///   dbf d0,loc_8BA` — copies 11 words (lives, hitpoints, diamonds,
    ///   level ID, helmet, continues, extra_hitpoint_slots, score[long]) from
    ///   the selected player slot into the working Number_Lives region.
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
        // ASM loc_75C0 (kid.asm:7700–7701): `bsr.w sub_7B30` resolves wall
        // overlaps at the spawn position.  The full sub_7B30 transform-overlap
        // check is not needed at spawn (the player always starts as a known
        // form); native uses a simpler wall-probe pair instead.
        // loc_75C0 also sets `$22(a1) = LnkTo_unk_BEDF0-Data_Index` when
        // Current_Helmet == Juggernaut — this is a visual sprite-mapping
        // override with no gameplay effect (not modelled in native).
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
            .map_or(180, |(_, max_steps)| *max_steps)
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

    #[allow(clippy::unused_self, reason = "semantically a method")]
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
        // ASM loc_7526 (kid.asm:7674–7676): Kid gfx object setup —
        // `move.w (PlayerStart_X_pos).w,x_pos(a3)`.
        let kid_x_pos = u32::from(header.player_start_x) << 16;
        // ASM loc_7526 (kid.asm:7675–7676): `move.w (PlayerStart_Y_pos).w,
        // y_pos(a3); subq.w #1,y_pos(a3)` — places Kid 1 pixel above the
        // header spawn Y (into the floor cell above, not the empty cell below).
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
            _ => self.level_header(level_id).map_or(
                LevelSkipHoldPose {
                    kid_x_pos: 0x0020_0000,
                    kid_y_pos: OPENING_GROUNDED_KID_Y_POS,
                    camera_x_pos: 0,
                    camera_y_pos: 0,
                    number_objects: 4,
                    number_gfx_objects: 4,
                },
                |header| LevelSkipHoldPose {
                    kid_x_pos: u32::from(header.player_start_x) << 16,
                    kid_y_pos: u32::from(header.player_start_y) << 16,
                    camera_x_pos: 0,
                    camera_y_pos: 0,
                    number_objects: 4,
                    number_gfx_objects: 4,
                },
            ),
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
        self.level_header(level_id).map_or(
            LevelBounds {
                width_pixels: 320,
                height_pixels: 224,
            },
            |header| LevelBounds {
                width_pixels: u16::from(header.width_screens) * 320,
                height_pixels: u16::from(header.height_screens) * 224,
            },
        )
    }
}
