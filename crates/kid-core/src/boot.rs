use super::*;
use kid_data::{DiamondHudDmaTransferSub5D4A, diamond_hud_dma_transfer_sub_5D4A};

// ASM loc_454 (kid.asm:513): final `bra.w returnFromException` in the M68k
// exception vector table (Vector58, exception ID $37).  The entire vector table
// (BusError through Vector64) loads an identifier into d0 and executes `rte`.
// The source port does not emulate the 68k exception mechanism — these vectors
// are hardware infrastructure with no gameplay effect.

/// ASM VDPGameInitValues (kid.asm:550-570): 19 bytes of per-register init data.
/// Each byte is the low byte of a VDP register command ($80xx–$92xx).
pub(crate) const VDP_GAME_INIT_VALUES: [u8; 19] = [
    0x14, // $8014 — HInt on, enable HV counter read
    0x74, // $8174 — display on, VInt on, DMA on, PAL off
    0x00, // $8200 — Scroll A address $0000
    0x00, // $8300 — Window address $0000
    0x07, // $8407 — Scroll B address $E000
    0x08, // $8508 — Sprite table address $1000
    0x00, // $8600 — null
    0x00, // $8700 — background colour pal 0 colour 0
    0x00, // $8800 — null
    0x00, // $8900 — null
    0x00, // $8A00 — HInt timing 0 scanlines
    0x03, // $8B03 — ext int off, VScroll full, HScroll per-pixel
    0x81, // $8C81 — 40-cell mode, no shadow/highlight, no interlace
    0x05, // $8D05 — HScroll table address $1400
    0x00, // $8E00 — null
    0x02, // $8F02 — VDP auto-increment 2 bytes
    0x01, // $9001 — 64×32 cell scroll size
    0x00, // $9100 — Window H left side, base point 0
    0x00, // $9200 — Window V upside, base point 0
];

/// ASM loc_5E6 (kid.asm:749): VDP register init loop in GameProgram.
/// ```asm
/// lea VDPGameInitValues(pc),a0
/// moveq #$12,d0          ; 19 registers ($80–$92)
/// move.w #$8000,d1
/// loc_5E6:
///   move.b (a0)+,d1      ; low byte = init value
///   move.w d1,4(a6)      ; write to VDP control port
///   addi.w #$100,d1      ; advance to next register
///   dbf d0,loc_5E6
/// ```
/// Returns the 19 register command words written to the VDP control port.
/// The source port does not write to real VDP hardware, but the register
/// values inform initial VDP state (scroll addresses, display mode, etc.).
#[allow(dead_code)]
#[allow(non_snake_case)]
pub(crate) fn vdp_game_init_register_commands_loc_5E6() -> [u16; 19] {
    let mut commands = [0u16; 19];
    let mut reg_base: u16 = 0x8000;
    for (i, &value) in VDP_GAME_INIT_VALUES.iter().enumerate() {
        // ASM loc_5E6: move.b (a0)+,d1 — overwrite low byte of d1
        commands[i] = reg_base | u16::from(value);
        // ASM: addi.w #$100,d1 — next register
        reg_base = reg_base.wrapping_add(0x100);
    }
    commands
}

/// ASM loc_5FE (kid.asm:757): RAM clear entry in GameProgram, immediately after
/// VDP register init.  Saves `Options_Suboption_2PController` longword ($FFFFFDC8,
/// 4 bytes covering 2PController / Speed / Controls) into d7, then:
/// ```asm
/// loc_5FE:
///   move.w #$3FFF,d0      ; 16384 iterations (longwords)
///   moveq  #0,d1
/// loc_604:
///   move.l d1,(a0)+        ; clear 4 bytes
///   dbf    d0,loc_604       ; loop $3FFF+1 = $10000 longwords = 64 KB
///   move.l d7,(Options_Suboption_2PController).w  ; restore saved options
///   cmpi.w #5,d7
///   bls.s  loc_61C          ; valid (0–5) → keep
///   move.l #0,(Options_Suboption_2PController).w  ; invalid → reset
/// ```
/// Returns the validated options longword.  In the native port, Rust struct
/// initialisation zeroes all state; this function models only the options
/// preserve-and-validate logic that survives the RAM clear.
#[allow(dead_code)]
#[allow(non_snake_case)]
pub(crate) fn validate_options_after_ram_clear_loc_5FE(saved_options_longword: u32) -> u32 {
    // ASM loc_604 (kid.asm:761): `move.l d1,(a0)+; dbf d0,loc_604` clears
    // $10000 longwords (64 KB) from Sprite_Table ($FFFF0000).
    // After restore, `cmpi.w #5,d7; bls.s loc_61C` validates the low word
    // (Options_Suboption_Controls).  The .w comparison tests only the bottom
    // 16 bits of d7 as an unsigned value.
    let controls_word = (saved_options_longword & 0xFFFF) as u16;
    if controls_word <= 5 {
        saved_options_longword
    } else {
        0 // Default_Options = 0
    }
}

/// RAM clear dimensions from loc_5FE/loc_604.
#[allow(dead_code)]
pub(crate) const RAM_CLEAR_START_ADDR_LOC_5FE: u32 = 0xFFFF_0000; // Sprite_Table
/// `move.w #$3FFF,d0` + dbf = $3FFF+1 = $4000 longwords = $10000 bytes.
#[allow(dead_code)]
pub(crate) const RAM_CLEAR_LONGWORD_COUNT_LOC_5FE: u32 = 0x4000;

/// ASM loc_61C (kid.asm:774): VRAM clear entry — `move.l #vdpComm($0000,VRAM,WRITE),4(a6)`
/// sets the VDP write address to VRAM $0000, then loc_62A loops.
///
/// ASM loc_62A (kid.asm:779): `move.l d1,(a6); dbf d0,loc_62A` — inner VRAM
/// clear loop.  d0=$3FFF (from `move.w #$3FFF,d0`), d1=0 (from `moveq #0,d1`).
/// Each iteration writes one zero longword (4 bytes) to the VDP data port;
/// $3FFF+1 = $4000 longwords = $10000 bytes = the full 64 KB VRAM.
///
/// After the VRAM clear, loc_61C continues with VSRAM clear (2 words at
/// VSRAM $0000) and hscroll clear (2 words at VRAM $1400), then
/// `move.w #$FFFF,($FFFFFC32).w` and `jsr (j_StopMusic)`.
///
/// In the native port, Rust struct initialisation zeroes all tile/scroll state;
/// this constant documents the original dimensions for parity tests.
#[allow(dead_code)]
pub(crate) const VRAM_CLEAR_VRAM_ADDR_LOC_61C: u16 = 0x0000;
/// `move.w #$3FFF,d0` + dbf = $4000 longwords = $10000 bytes (full 64 KB VRAM).
#[allow(dead_code)]
pub(crate) const VRAM_CLEAR_LONGWORD_COUNT_LOC_62A: u32 = 0x4000;

/// ASM loc_700 (kid.asm:832): second VRAM clear entry in the game-restart path
/// (loc_6E2).  Structurally identical to loc_61C: `move.l #vdpComm($0000,VRAM,WRITE),4(a6)`
/// followed by the inner loop at loc_70E.
///
/// ASM loc_70E (kid.asm:837): `move.l d1,(a6); dbf d0,loc_70E` — inner loop of
/// the second VRAM clear.  d0=$3FFF, d1=0 — same parameters as loc_62A.
/// The ROM clears VRAM twice: once at initial boot (loc_61C/loc_62A) and again
/// every time the game restarts via loc_6E2, which first conditionally runs
/// Pal_FadeOut then re-clears all 64 KB of VRAM.
///
/// In the native port, Rust struct initialisation handles the equivalent reset;
/// these constants document that the second clear uses identical dimensions.
#[allow(dead_code)]
pub(crate) const VRAM_CLEAR_VRAM_ADDR_LOC_700: u16 = 0x0000;
/// `move.w #$3FFF,d0` + dbf = $4000 longwords = $10000 bytes (full 64 KB VRAM).
#[allow(dead_code)]
pub(crate) const VRAM_CLEAR_LONGWORD_COUNT_LOC_70E: u32 = 0x4000;

/// ASM loc_720 (kid.asm:844): `move.w d1,(a0)+; dbf d0,loc_720` — Sprite_Table
/// clear loop in the game-restart path (loc_6E2).  After the VRAM clear at
/// loc_700/loc_70E, loads `lea (Sprite_Table).l,a0` and clears `$7DE0+1 = $7DE1`
/// (32225) zero words.  `d0=$7DE0` from `move.w #$7DE0,d0`, `d1=0` from
/// `moveq #0,d1`.  This zeroes the 64 450-byte sprite table buffer at $FFFF0000
/// before reinitialising object/gfx slots.
///
/// In the native port, Rust struct initialisation zeroes all sprite state; this
/// constant documents the original loop dimensions for parity tests.
#[allow(dead_code)]
pub(crate) const SPRITE_TABLE_CLEAR_WORD_COUNT_LOC_720: u32 = 0x7DE1; // $7DE0 + 1 (dbf)

/// ASM loc_A70 (kid.asm:1242): VBlank sprite-table DMA within
/// `Transfer_SpriteAndKidToVRAM` (sub_A5C).  Sequence:
///   1. Stop Z80
///   2. `dma68kToVDP Sprite_Table,$1000,$280,VRAM` — DMA 640 bytes ($280)
///      of the 68k-side Sprite_Table buffer to VDP VRAM address $1000.
///   3. Start Z80
///   4. Stop Z80 again (for Kid art DMA at loc_AD4)
///   5. loc_AC2 (kid.asm:1254): if `$FFFFF838` (art-list pointer) is null,
///      default a0 to `$FFFF0500`.  Write terminator `move.w #0,(a0)`,
///      then reset a0 to `$FFFF0500` for the DMA loop.
///   6. If `$FFFFFB49` (art-upload-skip flag) is set → loc_B36 (skip art).
///   7. Otherwise loop at loc_AD4: read (length, source) pairs from
///      `$FFFF0500`, DMA each tile run to VRAM starting at $C4A0.
///      loc_B24 (kid.asm:1294) is the loop tail: write the assembled VDP
///      command via `$FFFFF800`, advance the VRAM destination pointer
///      (`add.w d0,d2; add.w d0,d2`), and branch back to loc_AD4.
///
/// In the native port, rendering bypasses VDP entirely — sprite attributes
/// feed directly into the software renderer.  This constant documents the
/// original DMA destination address for parity reference.
#[allow(dead_code)]
pub(crate) const SPRITE_TABLE_DMA_VRAM_DEST_LOC_A70: u16 = 0x1000;
/// DMA transfer length in bytes at loc_A70: $280 = 640 bytes (160 sprites × 4 words).
#[allow(dead_code)]
pub(crate) const SPRITE_TABLE_DMA_LENGTH_BYTES_LOC_A70: u16 = 0x0280;
/// Kid art DMA destination in VRAM at loc_AD4 (within the loc_A70 art-upload loop).
#[allow(dead_code)]
pub(crate) const KID_ART_DMA_VRAM_BASE_LOC_AD4: u16 = 0xC4A0;

/// ASM loc_B42 (kid.asm:1309): empty-sprite-table early path in
/// `Transfer_SpriteAndKidToVRAM` (sub_A5C).  When `Addr_NextSpriteSlot ==
/// $FFFF0000` (no sprites allocated this frame), the ROM clears the first
/// 8 bytes of the sprite table (`clr.l (Sprite_Table).l; clr.l ($FFFF0004).l`)
/// to zero the link/y and size/x fields of sprite slot 0, ensuring a proper
/// terminator.  Then `bra.w loc_A70` proceeds to DMA the (now-terminated)
/// table to VRAM.
///
/// In the native port, VDP DMA is bypassed; sprite attributes feed the
/// software renderer directly.  This constant documents the sentinel
/// `Addr_NextSpriteSlot` value that triggers the loc_B42 early path.
#[allow(dead_code)]
pub(crate) const SPRITE_TABLE_BASE_SENTINEL_LOC_B42: u32 = 0xFFFF_0000;
/// Number of bytes cleared at loc_B42: two `clr.l` instructions = 8 bytes.
#[allow(dead_code)]
pub(crate) const SPRITE_TABLE_EMPTY_CLEAR_BYTES_LOC_B42: u32 = 8;

#[cfg_attr(not(test), allow(dead_code))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Prize10000PointsStarCleanupLoc10BE6 {
    pub(crate) visited_slots: [u32; 5],
    pub(crate) should_delete_current_object: bool,
}

/// ASM sub_40236 (kid.asm:45620): angle-to-sine/cosine lookup.
/// Takes an 8-bit angle (0-255 representing 0-360° in 256 steps), returns (cos, sin) as
/// 16-bit signed values in the range [-$3FFE, $3FFE] (table max is $3FFE).
/// The ROM uses a 65-entry sine table for 0°-90° and mirrors/negates for other quadrants.
/// Quadrant boundaries based on doubled angle comparisons:
/// Q1: doubled < 128 (angle < 64) → 0° to ~90°
/// Q2: 128 <= doubled < 256 (64 <= angle < 128) → ~90° to ~180°
/// Q3: 256 <= doubled < 384 (128 <= angle < 192) → ~180° to ~270°
/// Q4: doubled >= 384 (angle >= 192) → ~270° to ~360°
#[allow(non_snake_case)]
#[allow(dead_code)]
pub(crate) fn sin_cos_sub_40236(angle: u8) -> (i16, i16) {
    let angle = angle as u16;
    let doubled = angle.wrapping_mul(2);

    let (cos, sin) = if doubled < 0x80 {
        // Q1: angle in [0, 63], doubled in [0, 126]
        // ASM: move.w d0,d1; subi.w #$80,d0; neg.w d0; read table for cos and sin
        // cos offset = (128 - doubled) / 2, sin offset = doubled / 2
        let cos_offset = (0x80 - doubled) >> 1;
        let sin_offset = doubled >> 1;
        let cos_idx = (cos_offset as usize).min(64);
        let sin_idx = (sin_offset as usize).min(64);
        (
            SINE_TABLE_WORD_402B0[cos_idx] as i16,
            SINE_TABLE_WORD_402B0[sin_idx] as i16,
        )
    } else if doubled < 0x100 {
        // Q2: angle in [64, 127], doubled in [128, 254]
        // ASM loc_4025A: subi.w #$100,d0; neg.w d0; move.w d0,d1; subi.w #$80,d0; neg.w d0
        // Read cos from table then negate; read sin from table (no negation for sin in Q2)
        let d0_wrapped = (doubled as i16).wrapping_sub(0x100);
        let d0_neg = -d0_wrapped;
        let d1 = d0_neg as u16;
        let d0_sub = (d1 as i16).wrapping_sub(0x80);
        let d0_final = (-d0_sub) as u16;
        let cos_idx = ((d0_final >> 1) as usize).min(64);
        let sin_idx = ((d1 >> 1) as usize).min(64);
        (
            -(SINE_TABLE_WORD_402B0[cos_idx] as i16),
            SINE_TABLE_WORD_402B0[sin_idx] as i16,
        )
    } else if doubled < 0x180 {
        // Q3: angle in [128, 191], doubled in [256, 382]
        // ASM loc_40274 fallthrough: subi.w #$100,d0; move.w d0,d1; subi.w #$80,d0; neg.w d0
        // Read cos from table then negate; read sin from table then negate
        let d0_after_sub = doubled.wrapping_sub(0x100);
        let d1 = d0_after_sub;
        let d0_neg = (-(d1.wrapping_sub(0x80) as i16)) as u16;
        let cos_idx = ((d0_neg >> 1) as usize).min(64);
        let sin_idx = ((d1 >> 1) as usize).min(64);
        (
            -(SINE_TABLE_WORD_402B0[cos_idx] as i16),
            -(SINE_TABLE_WORD_402B0[sin_idx] as i16),
        )
    } else {
        // Q4: angle in [192, 255], doubled in [384, 510]
        // ASM loc_40294: subi.w #$200,d0; neg.w d0; move.w d0,d1; subi.w #$80,d0; neg.w d0
        // Read cos from table (no negation); read sin from table then negate
        let d0_after_sub = doubled.wrapping_sub(0x200);
        let d0_neg = (-(d0_after_sub as i16)) as u16;
        let d1 = d0_neg;
        let d0_final = (-(d1.wrapping_sub(0x80) as i16)) as u16;
        let cos_idx = ((d0_final >> 1) as usize).min(64);
        let sin_idx = ((d1 >> 1) as usize).min(64);
        (
            SINE_TABLE_WORD_402B0[cos_idx] as i16,
            -(SINE_TABLE_WORD_402B0[sin_idx] as i16),
        )
    };

    (cos, sin)
}

/// ASM sub_44B0 (kid.asm:5520): initialize HUD animation frame indices.
/// Sets 5 consecutive RAM locations for coin/heart animation frame counters.
/// $FFFFFA58=1, $FFFFFA5C=2, $FFFFFA5B=3, $FFFFFA59=4, $FFFFFA5A=5.
#[allow(non_snake_case)]
#[allow(dead_code)]
pub(crate) fn init_hud_animation_frames_sub_44B0() -> HudAnimationFramesSub44B0 {
    HudAnimationFramesSub44B0 {
        coin_1_frame: 1,
        coin_2_frame: 2,
        coin_3_frame: 3,
        heart_1_frame: 4,
        heart_2_frame: 5,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct HudAnimationFramesSub44B0 {
    pub(crate) coin_1_frame: u8,  // $FFFFFA58
    pub(crate) coin_2_frame: u8,  // $FFFFFA5C
    pub(crate) coin_3_frame: u8,  // $FFFFFA5B
    pub(crate) heart_1_frame: u8, // $FFFFFA59
    pub(crate) heart_2_frame: u8, // $FFFFFA5A
}

/// ASM sub_44D0 (kid.asm:5533): copy 32 longwords (128 bytes) from ROM to VDP.
/// ASM: move.w #$1F,d6; loc_44D4: move.l (a4)+,(a6); dbf d6,loc_44D4
/// loc_44D4 (kid.asm:5536) is the inner loop body — each iteration transfers
/// one longword from the source art pointer (a4) to the VDP data port (a6).
/// Used for uploading animation art to VDP RAM during HUD rendering.
#[allow(non_snake_case)]
#[allow(dead_code)]
pub(crate) fn vdp_upload_32_longwords_sub_44D0(data: &[u8; 128]) -> [u32; 32] {
    core::array::from_fn(|i| {
        let offset = i * 4;
        u32::from_be_bytes([
            data[offset],
            data[offset + 1],
            data[offset + 2],
            data[offset + 3],
        ])
    })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct HudAnimationStateSub44DC {
    pub(crate) coin_countdown_fb5b: u8,
    pub(crate) coin_frame_offset_fb60: u8,
    pub(crate) life_countdown_fb59: u8,
    pub(crate) life_frame_offset_fb5e: u8,
    pub(crate) clock_countdown_fb5a: u8,
    pub(crate) clock_frame_offset_fb5f: u8,
    pub(crate) diamond_countdown_fb58: u8,
    pub(crate) diamond_frame_offset_fb5d: u8,
    pub(crate) flag_countdown_fb5c: u8,
    pub(crate) flag_frame_offset_fb61: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct HudFrameUploadSub44DC {
    pub(crate) asset_label: &'static str,
    pub(crate) vram_word_address: u16,
    pub(crate) frame_table_byte_offset: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct FlagHudDmaUploadSub44DC {
    pub(crate) asset_label: &'static str,
    pub(crate) vram_byte_address: u16,
    pub(crate) frame_table_byte_offset: u8,
}

#[allow(non_snake_case)]
#[allow(dead_code)]
pub(crate) fn flag_hud_dma_upload_sub_5DA6(frame_table_byte_offset: u8) -> FlagHudDmaUploadSub44DC {
    // ASM loc_4680 (kid.asm:5748): read the art frame byte from `(a4)`,
    // sign-extend to word, index into `off_4668` (6 longword pointers at
    // `ANIART_FLAG_SIZE` = `$280`-byte stride into `ArtUnc_4C92`), then call
    // sub_5DA6 for the DMA upload.
    //
    // ASM sub_5DA6 (kid.asm:5803): stop Z80, configure a DMA of
    // `ANIART_FLAG_SIZE` bytes from `a1`, and write it to VRAM byte `$D340`.
    // `sub_44DC` selects the source frame by advancing the byte offset inside
    // `ArtUnc_4C92`, so the native plan preserves that selector and the fixed
    // destination.
    FlagHudDmaUploadSub44DC {
        asset_label: "ArtUnc_4C92",
        vram_byte_address: 0xD340,
        frame_table_byte_offset,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct HudTimerSpritePositionSubBD0A {
    pub(crate) y_pos: u16,
    pub(crate) x_pos: u16,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct HudTimerSpriteLayoutSubBD0A {
    pub(crate) minutes: HudTimerSpritePositionSubBD0A,
    pub(crate) separator: HudTimerSpritePositionSubBD0A,
    pub(crate) seconds_tens: HudTimerSpritePositionSubBD0A,
    pub(crate) seconds_ones: HudTimerSpritePositionSubBD0A,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct HudAnimationStepSub44DC {
    pub(crate) next_state: HudAnimationStateSub44DC,
    pub(crate) coin_upload: Option<HudFrameUploadSub44DC>,
    pub(crate) life_upload: Option<HudFrameUploadSub44DC>,
    pub(crate) clock_upload: Option<HudFrameUploadSub44DC>,
    pub(crate) diamond_upload: Option<DiamondHudDmaTransferSub5D4A>,
    pub(crate) flag_upload: Option<FlagHudDmaUploadSub44DC>,
}

#[allow(non_snake_case)]
#[allow(dead_code)]
pub(crate) fn hud_timer_sprite_layout_sub_BD0A(
    mut layout: HudTimerSpriteLayoutSubBD0A,
    track_kid: bool,
    timer_frozen: bool,
    kid_x_pos: u16,
    kid_y_pos: u16,
    camera_x_pos: u16,
    camera_y_pos: u16,
) -> HudTimerSpriteLayoutSubBD0A {
    // ASM sub_BD0A (kid.asm:15286): when `d7==0`, seed the four timer sprites
    // onto the fixed HUD row at `y=$90` with X positions `$90,$98,$9D,$A5`.
    // When `d7!=0`, bail out early if `$FFFFFB4B` is set; otherwise hide the
    // minute and separator sprites at `x=1` and pin the two seconds digits to
    // Kid's screen position using `x_pos-Camera_X_pos+$80` and
    // `y_pos-Camera_Y_pos-$2C+$80`.
    if !track_kid {
        return HudTimerSpriteLayoutSubBD0A {
            minutes: HudTimerSpritePositionSubBD0A {
                y_pos: 0x0090,
                x_pos: 0x0090,
            },
            separator: HudTimerSpritePositionSubBD0A {
                y_pos: 0x0090,
                x_pos: 0x0098,
            },
            seconds_tens: HudTimerSpritePositionSubBD0A {
                y_pos: 0x0090,
                x_pos: 0x009D,
            },
            seconds_ones: HudTimerSpritePositionSubBD0A {
                y_pos: 0x0090,
                x_pos: 0x00A5,
            },
        };
    }

    // ASM loc_BD38 (kid.asm:15301): `tst.b ($FFFFFB4B).w; beq.s loc_BD40; rts`
    // — if the timer-frozen flag is set, return the layout unchanged.
    if timer_frozen {
        return layout;
    }

    // ASM loc_BD40 (kid.asm:15307): Kid-tracking timer sprite positioning.
    // `move.l (Addr_GfxObject_Kid).w,a2; move.w $1A(a2),d5; sub.w (Camera_X_pos).w,d5;
    //  addi.w #$80,d5` → screen-x.  Same for y with `$1E(a2)`, Camera_Y_pos, `subi.w #$2C`,
    //  `addi.w #$80`.  Hides minute/separator at x=1, pins seconds digits at
    //  `kid_screen_x + unk_BD8A[-7, +1]`.
    let kid_screen_x = kid_x_pos.wrapping_sub(camera_x_pos).wrapping_add(0x0080);
    let kid_screen_y = kid_y_pos
        .wrapping_sub(camera_y_pos)
        .wrapping_sub(0x002C)
        .wrapping_add(0x0080);
    layout.minutes.x_pos = 1;
    layout.separator.x_pos = 1;
    layout.seconds_tens.y_pos = kid_screen_y;
    layout.seconds_tens.x_pos = kid_screen_x.wrapping_sub(7);
    layout.seconds_ones.y_pos = kid_screen_y;
    layout.seconds_ones.x_pos = kid_screen_x.wrapping_add(1);
    layout
}

/// A single hitpoint sprite entry as built by `Make_SpriteAttr_HUD` (sub_BDC0).
///
/// Each entry is 8 bytes in the VDP sprite table: y-pos(2), size+link(2), tile(2), x-pos(2).
/// - Tile `$822D` = full (2 HP filled), `$822E` = half (1 HP filled), `$822F` = empty (0 HP).
/// - `visible=true` means `size=$04` (1×1 sprite shown); `false` means `size=$00` (hidden).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)]
pub(crate) struct HudHitpointSpriteSubBDC0 {
    pub(crate) y_pos: u16,
    pub(crate) x_pos: u16,
    pub(crate) tile: u16,
    pub(crate) visible: bool,
}

/// ASM Make_SpriteAttr_HUD / sub_BDC0 (kid.asm:15382): builds hitpoint sprite
/// entries for the HUD.  Returns an empty vec when `currently_transforming` is
/// true (the ASM jumps straight to loc_BE82).
///
/// The loop at loc_BE0C iterates over `max_slots / 2` positions (each sprite
/// represents 2 HP).  `d0` (current HP) and `d1` (max HP slots) both decrement
/// by 2 per iteration.  The first `x` is `$80 + $10 = $90` (loc_BE0C adds $10
/// before the first sprite write).  When `hud_slide_active` ($FFFFFB49), the
/// base `x` slides left by `min(time_frames, $50)` (loc_BE06).
#[allow(dead_code)]
#[allow(non_snake_case)]
pub(crate) fn make_hud_hitpoint_layout_sub_BDC0(
    number_hitpoints: u16,
    max_hitpoint_slots: u16,
    hud_slide_active: bool,
    time_frames: u16,
    currently_transforming: bool,
) -> Vec<HudHitpointSpriteSubBDC0> {
    // ASM sub_BDC0: `tst.b (Currently_transforming).w; bne.w loc_BE82`
    if currently_transforming {
        return Vec::new();
    }

    let mut d0 = number_hitpoints as i16;
    let mut d1 = max_hitpoint_slots as i16;

    // ASM: `move.w #$80,d2` — base x-position.
    let mut d2: u16 = 0x0080;

    // ASM loc_BE06 (kid.asm:15402): `sub.w d3,d2` — slide HUD left by
    // min(Time_Frames, $50) when $FFFFFB49 is set.
    if hud_slide_active {
        let d3 = time_frames.min(0x0050);
        d2 = d2.wrapping_sub(d3);
    }

    // ASM loc_BE08 (kid.asm:15405): `move.w #$A4,d3` — y-position for all
    // hitpoint sprites.
    let y: u16 = 0x00A4;

    let mut sprites = Vec::new();

    // ASM loc_BE0C (kid.asm:15408): main loop.
    // `addq.w #1,d4` (link — not modelled); `addi.w #$10,d2` (x spacing);
    // `subq.w #2,d1; subq.w #2,d0`.
    loop {
        d2 = d2.wrapping_add(0x0010);
        d1 -= 2;
        d0 -= 2;

        if d0 >= 0 {
            // ASM loc_BE20 (kid.asm:15419): full hitpoint — tile $822D, visible.
            // `move.w d3,(a2)+; move.b #4,(a2)+; … move.w #$822D,(a2)+; move.w d2,(a2)+`
            sprites.push(HudHitpointSpriteSubBDC0 {
                y_pos: y,
                x_pos: d2,
                tile: 0x822D,
                visible: true,
            });
            if d1 <= 0 {
                break;
            }
        } else if d0 == -1 {
            // ASM loc_BE34: half hitpoint.
            if d1 < 0 {
                // loc_BE4C: hidden half — size=$00, tile=$822D. `bra.s loc_BE82`.
                sprites.push(HudHitpointSpriteSubBDC0 {
                    y_pos: y,
                    x_pos: d2,
                    tile: 0x822D,
                    visible: false,
                });
                break;
            }
            // Visible half — tile $822E.
            sprites.push(HudHitpointSpriteSubBDC0 {
                y_pos: y,
                x_pos: d2,
                tile: 0x822E,
                visible: true,
            });
            if d1 <= 0 {
                break;
            }
        } else {
            // ASM loc_BE5C: empty hitpoint.
            if d1 < 0 {
                // loc_BE74: hidden empty — size=$00, tile=$822F. Falls through to loc_BE82.
                sprites.push(HudHitpointSpriteSubBDC0 {
                    y_pos: y,
                    x_pos: d2,
                    tile: 0x822F,
                    visible: false,
                });
                break;
            }
            // Visible empty — tile $822F.
            sprites.push(HudHitpointSpriteSubBDC0 {
                y_pos: y,
                x_pos: d2,
                tile: 0x822F,
                visible: true,
            });
            if d1 <= 0 {
                break;
            }
        }
    }

    sprites
}

/// HUD timer digit tile IDs as written by loc_BEE8 (kid.asm:15505).
///
/// After `Decrease_Time_Left` updates the three digit counters, loc_BEE8
/// converts each digit to a VRAM tile ID by adding `$86BA`:
/// ```text
/// move.w (Time_Seconds_low_digit).w,d0; addi.w #$86BA,d0; move.w d0,$34(a0)
/// move.w (Time_Seconds_high_digit).w,d0; addi.w #$86BA,d0; move.w d0,$2C(a0)
/// move.w (Time_Minutes).w,d0; addi.w #$86BA,d0; move.w d0,$1C(a0)
/// ```
/// The tile base `$86BA` addresses the digit font in VRAM: digit 0 → tile
/// `$86BA`, digit 1 → `$86BB`, … digit 9 → `$86C3`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct HudTimerDigitTilesLocBEE8 {
    /// Tile ID for the minutes digit (stored at `$1C(a0)`).
    pub(crate) minutes_tile: u16,
    /// Tile ID for the seconds tens digit (stored at `$2C(a0)`).
    pub(crate) seconds_tens_tile: u16,
    /// Tile ID for the seconds ones digit (stored at `$34(a0)`).
    pub(crate) seconds_ones_tile: u16,
}

/// ASM loc_BEE8 (kid.asm:15505): convert timer digit values to VRAM tile IDs.
///
/// Each of `Time_Minutes`, `Time_Seconds_high_digit`, `Time_Seconds_low_digit`
/// is a single BCD-ish digit (0–9 for seconds digits, 0–5 for seconds tens,
/// 0–9 for minutes).  The tile ID is simply `digit + $86BA`.
#[allow(dead_code)]
#[allow(non_snake_case)]
pub(crate) fn hud_timer_digit_tiles_loc_BEE8(
    time_minutes: u16,
    time_seconds_high_digit: u16,
    time_seconds_low_digit: u16,
) -> HudTimerDigitTilesLocBEE8 {
    // ASM loc_BEE8: addi.w #$86BA,d0 for each digit.
    HudTimerDigitTilesLocBEE8 {
        minutes_tile: time_minutes.wrapping_add(0x86BA),
        seconds_tens_tile: time_seconds_high_digit.wrapping_add(0x86BA),
        seconds_ones_tile: time_seconds_low_digit.wrapping_add(0x86BA),
    }
}

/// Result of `End_Decrease_Time_Left` low-time check (kid.asm:15518).
///
/// After writing digit tiles, End_Decrease_Time_Left checks whether the
/// timer is at or below 0:29 (minutes==0 && seconds_tens<=2).  If so,
/// `sub_BD0A` is called with `d7=1` to switch timer sprites to Kid-tracking
/// mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct HudTimerLowTimeCheckEndDecreaseTimeLeft {
    /// Whether the timer is low enough to track Kid (`d7=1` path).
    pub(crate) track_kid: bool,
}

/// ASM End_Decrease_Time_Left (kid.asm:15518): low-time detection.
///
/// ```text
/// tst.w (Time_Minutes).w; bne.w loc_BF2A
/// cmpi.w #2,(Time_Seconds_high_digit).w; bgt.w loc_BF2A
/// moveq #1,d7; bsr.w sub_BD0A
/// ```
/// Returns `track_kid=true` when minutes==0 and seconds_tens<=2 (i.e. ≤29s).
#[allow(dead_code)]
pub(crate) fn hud_timer_low_time_check_end_decrease_time_left(
    time_minutes: u16,
    time_seconds_high_digit: u16,
) -> HudTimerLowTimeCheckEndDecreaseTimeLeft {
    // ASM: tst.w (Time_Minutes).w; bne.w loc_BF2A → skip if minutes != 0
    // cmpi.w #2,(Time_Seconds_high_digit).w; bgt.w loc_BF2A → skip if tens > 2
    let track_kid = time_minutes == 0 && time_seconds_high_digit <= 2;
    HudTimerLowTimeCheckEndDecreaseTimeLeft { track_kid }
}

/// Tile ID pair for a two-digit HUD number, as computed by
/// `calc_display_number` (kid.asm:15568).
///
/// Used by loc_BF2A for both the lives count and diamond count displays.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct HudDisplayNumberTilesCalcDisplayNumber {
    /// Tile ID for the tens digit (high word of d1).
    pub(crate) tens_tile: u16,
    /// Tile ID for the ones digit (low word of d1).
    pub(crate) ones_tile: u16,
}

/// ASM calc_display_number (kid.asm:15568): convert a 0–99 number to tile IDs.
///
/// ```text
/// move.w #$86BA,d6; move.w d6,d1; swap d1; move.w d6,d1
/// cmpi.w #$A,d0; bge.s split_double_digit
/// add.w d0,d1; swap d1; addi.w #$B,d1; swap d1; rts
/// split_double_digit: divu.w #$A,d0; add.l d0,d1; rts
/// ```
///
/// For values < 10: ones = `$86BA + value`, tens = `$86BA + $B` (blank tile).
/// For values >= 10: `divu.w #$A` puts quotient in low word, remainder in
/// high word; `add.l` adds both halves to the `$86BA86BA` base.
#[allow(dead_code)]
pub(crate) fn calc_display_number(value: u16) -> HudDisplayNumberTilesCalcDisplayNumber {
    let base: u16 = 0x86BA;
    if value < 10 {
        // ASM: add.w d0,d1 (ones); swap; addi.w #$B,d1 (tens = blank)
        HudDisplayNumberTilesCalcDisplayNumber {
            tens_tile: base.wrapping_add(0x0B),
            ones_tile: base.wrapping_add(value),
        }
    } else {
        // ASM: divu.w #$A,d0 → quotient in d0.w, remainder in d0[31:16]
        // add.l d0,d1 where d1 = $86BA_86BA
        let quotient = value / 10;
        let remainder = value % 10;
        HudDisplayNumberTilesCalcDisplayNumber {
            tens_tile: base.wrapping_add(quotient),
            ones_tile: base.wrapping_add(remainder),
        }
    }
}

/// ASM loc_BF2A (kid.asm:15526): lives display clamp and update.
///
/// ```text
/// move.w (Number_Lives).w,d0; cmpi.w #100,d0; blt.s life_display
/// moveq #99,d0; move.w d0,(Number_Lives).w
/// ```
/// Clamps lives to 99 before display.  Returns the clamped value.
#[allow(dead_code)]
#[allow(non_snake_case)]
pub(crate) fn hud_lives_clamp_loc_BF2A(number_lives: u16) -> u16 {
    // ASM loc_BF2A: cmpi.w #100,d0; blt.s life_display; moveq #99,d0
    if number_lives >= 100 {
        99
    } else {
        number_lives
    }
}

/// ASM loc_C03A (kid.asm:15628): hide flagpole sprites on boss levels.
///
/// `Flagpole_Boss` allocates two gfx objects for the flagpole.  At the end,
/// it checks the current level against the four boss level IDs:
/// ```text
/// cmpi.w #Boss1_LevelID,(Current_LevelID).w; beq.s loc_C03A
/// cmpi.w #Boss2_LevelID,(Current_LevelID).w; beq.s loc_C03A
/// cmpi.w #Boss3_LevelID,(Current_LevelID).w; beq.s loc_C03A
/// cmpi.w #Boss4_LevelID,(Current_LevelID).w; bne.s return_C046
/// loc_C03A: sf $13(a4); sf $13(a3); st $3D(a4)
/// ```
/// On boss levels, both flagpole objects are hidden (`$13`=0 disables
/// rendering) and the first object's `$3D` flag is set (boss-mode marker).
///
/// Boss level IDs: $10, $1E, $2A, $33.
#[allow(dead_code)]
#[allow(non_snake_case)]
pub(crate) fn is_boss_level_loc_C03A(level_id: u16) -> bool {
    // ASM: Boss1_LevelID=$10, Boss2_LevelID=$1E, Boss3_LevelID=$2A, Boss4_LevelID=$33
    matches!(level_id, 0x10 | 0x1E | 0x2A | 0x33)
}

fn step_hud_animation_pair_table(
    countdown: u8,
    frame_offset: u8,
    table: &[u8],
    sentinel: u8,
) -> (u8, u8, Option<u8>) {
    let decremented = countdown.wrapping_sub(1);
    if decremented != 0 {
        return (decremented, frame_offset, None);
    }

    let mut next_offset = frame_offset.wrapping_add(2);
    loop {
        let speed = table
            .get(usize::from(next_offset))
            .copied()
            .unwrap_or(sentinel);
        if speed != sentinel {
            let frame_table_byte_offset = table[usize::from(next_offset) + 1];
            return (speed, next_offset, Some(frame_table_byte_offset));
        }
        next_offset = 0;
    }
}

#[allow(non_snake_case)]
#[allow(dead_code)]
pub(crate) fn step_hud_animation_sub_44DC(
    state: HudAnimationStateSub44DC,
) -> HudAnimationStepSub44DC {
    // ASM sub_44DC (kid.asm:5546): decrement five staggered HUD animation
    // countdown bytes, and whenever one expires, advance its paired speed/frame
    // table by two bytes, wrap on the `$FF,$FF` sentinel, then issue the
    // matching VRAM write or DMA for coin, life, clock, diamond, and flag art.
    const COIN_TABLE: &[u8] = &[
        6, 0x00, 6, 0x04, 6, 0x08, 6, 0x14, 6, 0x10, 6, 0x0C, 6, 0x10, 6, 0x14, 6, 0x08, 6, 0x04,
        0xFF, 0xFF,
    ];
    const LIFE_TABLE: &[u8] = &[0x48, 0x00, 6, 0x04, 6, 0x08, 6, 0x04, 0xFF, 0xFF];
    const CLOCK_TABLE: &[u8] = &[0x48, 0x00, 6, 0x04, 6, 0x08, 6, 0x04, 0xFF, 0xFF];
    const DIAMOND_TABLE: &[u8] = &[6, 0x00, 6, 0x04, 6, 0x08, 6, 0x0C, 0xFF, 0xFF];
    const FLAG_TABLE: &[u8] = &[
        6, 0x00, 6, 0x04, 6, 0x08, 6, 0x0C, 6, 0x10, 6, 0x14, 0xFF, 0xFF,
    ];

    let (coin_countdown_fb5b, coin_frame_offset_fb60, coin_frame) = step_hud_animation_pair_table(
        state.coin_countdown_fb5b,
        state.coin_frame_offset_fb60,
        COIN_TABLE,
        0xFF,
    );
    // ASM loc_4546 (kid.asm:5592): store coin result to $FB5B, then load life
    // icon countdown from $FFFFFA59, sign-extend, decrement.  If nonzero → skip
    // to loc_459A (clock).  If zero → load life frame offset from $FFFFFA5E,
    // advance by 2, walk AniArt_LifeIcon table (wrap on $FF sentinel via
    // loc_455E), then fall through to loc_4586 for the art frame load.
    let (life_countdown_fb59, life_frame_offset_fb5e, life_frame) = step_hud_animation_pair_table(
        state.life_countdown_fb59,
        state.life_frame_offset_fb5e,
        LIFE_TABLE,
        0xFF,
    );
    let (clock_countdown_fb5a, clock_frame_offset_fb5f, clock_frame) =
        step_hud_animation_pair_table(
            state.clock_countdown_fb5a,
            state.clock_frame_offset_fb5f,
            CLOCK_TABLE,
            0xFF,
        );
    // ASM loc_45EE (kid.asm:5670): store clock countdown result to $FFFFFA5A,
    // then load diamond countdown from $FFFFFA58, sign-extend, decrement.  If
    // nonzero → skip to loc_4630 (flag section).  If zero → load diamond frame
    // offset from $FFFFFA5D, advance by 2, fall through to loc_460A.
    //
    // ASM loc_460A (kid.asm:5681): diamond animation script read — reads byte
    // from unk_461C at script index.  If $FF sentinel → reset offset to 0 and
    // loop (bra.s loc_460A).  If positive → delay byte, fall through to
    // loc_4626 which reads the art frame index and calls sub_5D4A (DMA to VRAM
    // $DD40).
    let (diamond_countdown_fb58, diamond_frame_offset_fb5d, diamond_frame) =
        step_hud_animation_pair_table(
            state.diamond_countdown_fb58,
            state.diamond_frame_offset_fb5d,
            DIAMOND_TABLE,
            0xFF,
        );
    let (flag_countdown_fb5c, flag_frame_offset_fb61, flag_frame) = step_hud_animation_pair_table(
        state.flag_countdown_fb5c,
        state.flag_frame_offset_fb61,
        FLAG_TABLE,
        0xFF,
    );

    HudAnimationStepSub44DC {
        next_state: HudAnimationStateSub44DC {
            coin_countdown_fb5b,
            coin_frame_offset_fb60,
            life_countdown_fb59,
            life_frame_offset_fb5e,
            clock_countdown_fb5a,
            clock_frame_offset_fb5f,
            diamond_countdown_fb58,
            diamond_frame_offset_fb5d,
            flag_countdown_fb5c,
            flag_frame_offset_fb61,
        },
        // ASM loc_4534 (kid.asm:5585): coin art frame load — reads frame index
        // byte from AniArt_Coin script, sign-extends to word, indexes into
        // off_451C (6 art pointers at $80-byte stride into ArtUnc_4992), sets
        // VDP write command for VRAM $4720, then calls sub_44D0 to upload 128
        // bytes of tile data.
        coin_upload: coin_frame.map(|frame_table_byte_offset| HudFrameUploadSub44DC {
            asset_label: "ArtUnc_4992",
            vram_word_address: 0x4720,
            frame_table_byte_offset,
        }),
        // ASM loc_4586 (kid.asm:5622): life icon art frame load — reads the
        // frame index byte from AniArt_LifeIcon, sign-extends to word, indexes
        // into off_457A (3 art pointers at $80-byte stride into ArtUnc_4692),
        // sets VDP write command for VRAM $4620, then calls sub_44D0 to upload
        // 128 bytes of tile data.
        life_upload: life_frame.map(|frame_table_byte_offset| HudFrameUploadSub44DC {
            asset_label: "ArtUnc_4692",
            vram_word_address: 0x4620,
            frame_table_byte_offset,
        }),
        // ASM loc_45DA (kid.asm:5663): clock art frame load — reads the frame
        // index byte from AniArt_Clock, sign-extends to word, indexes into
        // off_45CE (3 art pointers at $80-byte stride into ArtUnc_4812), sets
        // VDP write command for VRAM $46A0, then calls sub_44D0 to upload 128
        // bytes of tile data.
        clock_upload: clock_frame.map(|frame_table_byte_offset| HudFrameUploadSub44DC {
            asset_label: "ArtUnc_4812",
            vram_word_address: 0x46A0,
            frame_table_byte_offset,
        }),
        diamond_upload: diamond_frame.map(|frame_table_byte_offset| {
            diamond_hud_dma_transfer_sub_5D4A(u16::from(frame_table_byte_offset))
                .expect("ASM sub_44DC only emits selectors from `unk_461C` that fit sub_5D4A")
        }),
        // ASM loc_468C (kid.asm:5754): `move.b d7,($FFFFFA5C).w; rts` — store
        // the speed byte (d7) from `AniArt_Flag` as the next flag countdown.
        // The native equivalent is the `flag_countdown_fb5c` field returned by
        // `step_hud_animation_pair_table`.
        flag_upload: flag_frame.map(flag_hud_dma_upload_sub_5DA6),
    }
}

#[allow(non_snake_case)]
#[allow(dead_code)]
pub(crate) fn copy_sega_screen_palette_loc_12DFE(
    source_words: &[u16],
    palette_buffer: &mut [u16],
    destination_byte_offset: u8,
    dbf_count: u8,
) {
    // ASM loc_12DFE (kid.asm:25459): `add.w d7,a3` treats the first header byte
    // as a byte offset into `Palette_Buffer`, then the `dbf d7,loc_12DFE` loop
    // copies `d7+1` palette words from `(a2)+` into `(a3)+`.
    assert_eq!(
        destination_byte_offset & 1,
        0,
        "Sega screen palette copy should start on a word-aligned Palette_Buffer offset"
    );
    let start_index = usize::from(destination_byte_offset >> 1);
    let words_to_copy = usize::from(dbf_count) + 1;
    let end_index = start_index + words_to_copy;
    palette_buffer[start_index..end_index].copy_from_slice(&source_words[..words_to_copy]);
}

#[allow(non_snake_case)]
#[allow(dead_code)]
pub(crate) fn sega_screen_tile_strip_sub_12E3C(first_tile: u16) -> [u16; 12] {
    // ASM sub_12E3C / loc_12E3E (kid.asm:25476): `moveq #$B,d0` followed by the
    // `dbf` loop writes 12 successive tile words starting from d1 to the VDP
    // data port, incrementing d1 after each write.
    core::array::from_fn(|index| {
        first_tile
            .checked_add(index as u16)
            .expect("Sega screen tile strip should stay within u16 tile words")
    })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct SegaScreenObjectBootstrapLoc12E48 {
    pub(crate) vdp_register_word: u16,
    pub(crate) restored_sr: u16,
    pub(crate) allocate_object_slot_arg: u16,
    pub(crate) object_routine_label: &'static str,
}

#[allow(non_snake_case)]
#[allow(dead_code)]
pub(crate) fn sega_screen_object_bootstrap_loc_12E48() -> SegaScreenObjectBootstrapLoc12E48 {
    // ASM loc_12E48 (kid.asm:25488): restore VDP register 1 with `$8174`,
    // lower SR back to `$2500`, call `j_Allocate_ObjectSlot` with `a0=0`, and
    // seed the allocated object's routine longword with `#loc_12E64`.
    SegaScreenObjectBootstrapLoc12E48 {
        vdp_register_word: 0x8174,
        restored_sr: 0x2500,
        allocate_object_slot_arg: 0,
        object_routine_label: "loc_12E64",
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct SegaScreenStartPollLoc12E6C {
    pub(crate) wait_iterations: u16,
    pub(crate) palette_upload_calls: u16,
    pub(crate) start_pressed: bool,
}

#[allow(non_snake_case)]
#[allow(dead_code)]
pub(crate) fn sega_screen_start_poll_loc_12E6C(
    ctrl_1_pressed_reads: &[u8],
) -> SegaScreenStartPollLoc12E6C {
    // ASM loc_12E6C (kid.asm:25501): seed `d2 = #$A`, then on each pass wait
    // for VBlank, read the joypad, consume the Start edge with
    // `bclr #7,(Ctrl_1_Pressed).w`, and only call `j_Palette_to_VRAM` when
    // Start was not newly pressed. `dbf d2,loc_12E6C` makes the loop run
    // exactly 11 iterations when Start never interrupts it.
    let mut wait_iterations = 0u16;
    let mut palette_upload_calls = 0u16;
    let mut counter = 0x000Au16;

    loop {
        wait_iterations = wait_iterations.wrapping_add(1);
        let ctrl_1_pressed = ctrl_1_pressed_reads
            .get((wait_iterations - 1) as usize)
            .copied()
            .unwrap_or(0);
        let start_pressed = ctrl_1_pressed & 0x80 != 0;
        if start_pressed {
            return SegaScreenStartPollLoc12E6C {
                wait_iterations,
                palette_upload_calls,
                start_pressed: true,
            };
        }

        palette_upload_calls = palette_upload_calls.wrapping_add(1);
        if counter == 0 {
            return SegaScreenStartPollLoc12E6C {
                wait_iterations,
                palette_upload_calls,
                start_pressed: false,
            };
        }
        counter -= 1;
    }
}

#[allow(non_snake_case)]
#[allow(dead_code)]
pub(crate) fn copy_sega_screen_palette_fade_strip_loc_12ED2(
    source_words: &[u16],
    palette_buffer: &mut [u16],
) {
    // ASM loc_12ED2 (kid.asm:25537): after `lea Pal_12EEC+6(pc,d0.w),a0` and
    // `lea (Palette_Buffer+$4).l,a1`, `moveq #$A,d0` drives a `dbf` loop that
    // copies 11 words from `(a0)+` into `(a1)+`.
    const FADE_STRIP_WORDS_LOC_12ED2: usize = 11;
    const PALETTE_BUFFER_START_WORD_LOC_12ED2: usize = 2;

    palette_buffer[PALETTE_BUFFER_START_WORD_LOC_12ED2
        ..PALETTE_BUFFER_START_WORD_LOC_12ED2 + FADE_STRIP_WORDS_LOC_12ED2]
        .copy_from_slice(&source_words[..FADE_STRIP_WORDS_LOC_12ED2]);
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct SegaScreenPaletteFadeTickLoc12ED8 {
    pub(crate) frame_delay_counter: i16,
    pub(crate) palette_cursor: i16,
    pub(crate) source_byte_offset_for_copy: Option<u16>,
}

#[allow(non_snake_case)]
#[allow(dead_code)]
pub(crate) fn sega_screen_palette_fade_tick_loc_12ED8(
    frame_delay_counter: i16,
    palette_cursor: i16,
) -> SegaScreenPaletteFadeTickLoc12ED8 {
    // ASM loc_12E96 -> loc_12ED8 (kid.asm:25515-25544): decrement `d1` each
    // frame; when the result stays >= 0 the loop branches straight to
    // `loc_12ED8` without touching the fade strip. Once it goes negative,
    // reset `d1` to 3, and only perform the `loc_12ED2` palette copy when
    // `d3` was still non-negative; exhausted negative cursors branch to
    // `loc_12ED8` unchanged.
    let decremented_frame_delay = frame_delay_counter.wrapping_sub(1);
    if decremented_frame_delay >= 0 {
        return SegaScreenPaletteFadeTickLoc12ED8 {
            frame_delay_counter: decremented_frame_delay,
            palette_cursor,
            source_byte_offset_for_copy: None,
        };
    }

    if palette_cursor < 0 {
        return SegaScreenPaletteFadeTickLoc12ED8 {
            frame_delay_counter: 3,
            palette_cursor,
            source_byte_offset_for_copy: None,
        };
    }

    SegaScreenPaletteFadeTickLoc12ED8 {
        frame_delay_counter: 3,
        palette_cursor: palette_cursor.wrapping_sub(2),
        source_byte_offset_for_copy: Some(
            u16::try_from(palette_cursor).expect("non-negative palette cursor should fit in u16"),
        ),
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct PauseMenuSpriteEntrySub6874 {
    pub(crate) y_pos: u16,
    pub(crate) vram_tile_word: u16,
}

/// Result of the pause-menu gate check at sub_6874 / loc_687C.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum PauseMenuGateLoc687C {
    /// Game_Paused == 0 → sub_6874 returns immediately (rts).
    NotPaused,
    /// Game_Paused != 0 and == 3 → loc_687C darkens palette, runs UI, then
    /// returns before reaching the sprite-hiding loop (unpausing).
    UnpausingReturn,
    /// Game_Paused != 0 and != 3 → loc_687C darkens palette, runs UI, then
    /// falls through to loc_6894 (sprite hiding + pause art load).
    PausedContinueToSpriteHide,
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn pause_menu_gate_loc_687C(game_paused: u8) -> PauseMenuGateLoc687C {
    // ASM sub_6874 (kid.asm:6575-6578):
    //   tst.b (Game_Paused).w; bne.s loc_687C; rts
    // ASM loc_687C (kid.asm:6581-6587):
    //   bsr.w Pause_DarkenPalette
    //   bsr.w Palette_to_VRAM
    //   jsr   (sub_E1334).l
    //   cmpi.b #3,(Game_Paused).w; bne.s loc_6894; rts
    if game_paused == 0 {
        PauseMenuGateLoc687C::NotPaused
    } else if game_paused == 3 {
        PauseMenuGateLoc687C::UnpausingReturn
    } else {
        PauseMenuGateLoc687C::PausedContinueToSpriteHide
    }
}

#[allow(non_snake_case)]
#[allow(dead_code)]
pub(crate) fn hide_pause_menu_overwritten_sprites_sub_6874(
    sprites: &mut [PauseMenuSpriteEntrySub6874],
) {
    // ASM loc_68AC–loc_68C0 (kid.asm:6600-6611): scan sprite entries and clear
    // the Y word for any whose masked tile index falls inside the pause-menu
    // VRAM range `$676..=$68E`. Sprites outside the range skip the clear via
    // `blt.s loc_68C0` / `bgt.s loc_68C0` (the advance-and-loop label).
    // loc_68C0: `addq.w #8,a0; dbf d3,loc_68AC` — advance to next 8-byte
    // sprite entry and loop for all $51 entries, then fall through to DMA
    // the updated Sprite_Table to VDP and load pause menu art.
    for sprite in sprites {
        let masked_tile = sprite.vram_tile_word & 0x07FF;
        if (0x0676..=0x068E).contains(&masked_tile) {
            sprite.y_pos = 0;
        }
    }
}

/// Number of bytes per iteration of the pause menu art copy inner loop.
/// ASM loc_690C (kid.asm:6622-6631): 8 × `move.l (a0)+,(a6)` = 32 bytes per
/// iteration, copying uncompressed tile art to VDP data port.
pub(crate) const PAUSE_MENU_ART_COPY_BYTES_PER_ITERATION_LOC_690C: u32 = 8 * 4;

/// Compute the total byte count for the pause menu uncompressed art DMA.
///
/// ASM kid.asm:6617-6631: reads a word count from the art header
/// (`move.w (a0)+,d0; subq.w #1,d0`), then `dbf d0,loc_690C` copies
/// `header_word` iterations of 32 bytes each to VRAM starting at `$CEC0`
/// (tile index `$676 × $20`).
#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn pause_menu_art_total_bytes_loc_690C(header_word: u16) -> u32 {
    // loc_690C inner loop: 8 longwords (32 bytes) per iteration, `header_word`
    // iterations total (dbf with initial `subq #1` means count = header_word).
    u32::from(header_word) * PAUSE_MENU_ART_COPY_BYTES_PER_ITERATION_LOC_690C
}

/// Layout for the pause menu's plane-tile save/restore, describing how many
/// word-reads per row and whether the menu wraps around the plane edge.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct PauseMenuPlaneColumnLayout {
    /// Tile column of the menu's left edge within the 64-tile-wide plane A.
    pub(crate) tile_col: u16,
    /// When `None`: menu fits entirely (d2 = -1 flag, 8 longword reads per row).
    /// When `Some((first_words, wrapped_words))`: menu wraps around the plane
    /// edge; `first_words` word-reads before the edge, `wrapped_words` after.
    /// Total always equals 16 words per row.
    pub(crate) wrap_split: Option<(u16, u16)>,
}

/// Compute the pause menu tile column and plane-edge wrapping layout.
///
/// ASM kid.asm:6633-6652 (loc_6894 flow, between art load and plane tile save):
/// ```text
///   move.w (Camera_X_pos).w,d0
///   addi.w #$60,d0        ; menu offset from camera
///   lsr.w  #3,d0          ; pixel → tile column
///   andi.w #$3F,d0        ; wrap to 64-tile plane width
///   cmpi.w #$30,d0        ; fits without wrapping?
///   bgt.s  loc_693A       ; no → wrapping case
///   moveq  #8,d1          ; 8 longword reads (= 16 words) per row
///   moveq  #-1,d2         ; flag: no split
///   bra.s  loc_6948
/// loc_693A:               ; wrapping case
///   moveq  #$40,d1; sub.w d0,d1   ; d1 = words before plane edge
///   moveq  #$10,d2; sub.w d1,d2   ; d2 = words after wrap
///   swap d2; move.w d1,d2; swap d2 ; pack (d1, $10-d1) into d2
/// ```
///
/// The ROM duplicates this exact computation at loc_69D2 (kid.asm:6716-6724)
/// for the text-write phase. loc_69D2 is the non-wrapped branch (d1=8, d2=-1,
/// `bra.s restart_text`) and loc_69EC (kid.asm:6727-6734) is the wrapped
/// branch (`$40-d0` / `$10-first` / swap-pack). Both produce identical
/// `tile_col` and `wrap_split` values — the difference is what happens after:
/// tile save reads from VDP, text write writes to VDP with `ori.w #$4000`.
#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn pause_menu_plane_column_layout_loc_693A(
    camera_x_pos: u16,
) -> PauseMenuPlaneColumnLayout {
    // ASM: d0 = ((Camera_X_pos + $60) >> 3) & $3F
    let tile_col = (camera_x_pos.wrapping_add(0x60) >> 3) & 0x3F;

    if tile_col <= 0x30 {
        // No wrapping: d1=8 (longword reads), d2=-1 (no-split flag).
        // 8 longword reads = 16 word-width nametable entries per row.
        PauseMenuPlaneColumnLayout {
            tile_col,
            wrap_split: None,
        }
    } else {
        // ASM loc_693A: menu wraps around the 64-tile plane edge.
        // d1 = $40 - d0 (word-reads before edge), d2 = $10 - d1 (after wrap).
        let first_words = 0x40_u16.wrapping_sub(tile_col);
        let wrapped_words = 0x10_u16.wrapping_sub(first_words);
        PauseMenuPlaneColumnLayout {
            tile_col,
            wrap_split: Some((first_words, wrapped_words)),
        }
    }
}

/// Number of rows saved by the pause menu tile-save loop at loc_695C.
///
/// ASM kid.asm:6660: `moveq #4,d5` — `dbf d5,loc_695C` iterates 5 times (0..=4).
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) const PAUSE_MENU_TILE_SAVE_ROW_COUNT_LOC_695C: u16 = 5;

/// Y-pixel offset added to Camera_Y_pos before the row loop at loc_6948/loc_695C.
///
/// ASM kid.asm:6656: `addi.w #$40,d7` — menu sits $40 pixels below camera top.
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) const PAUSE_MENU_Y_OFFSET_LOC_6948: u16 = 0x40;

/// Compute the VDP VRAM read command for one row of the pause menu tile save.
///
/// ASM loc_695C (kid.asm:6664-6674): per-row address computation within the
/// 5-row save loop.
/// ```text
///   move.w d7,d3          ; d7 = Camera_Y_pos + $40 + row*8
///   lsr.w  #3,d3          ; pixel → tile row
///   andi.w #$1F,d3        ; wrap to 32-row plane height
///   lsl.w  #7,d3          ; × 128 bytes = nametable row byte offset
///   move.w d6,d0          ; d6 = tile_col * 2 (column byte offset)
///   add.w  d3,d0          ; combine row + column
///   ori.w  #0,d0          ; no-op (read mode, not $4000 for write)
///   swap   d0             ; high word = VRAM address
///   move.w #0,d0          ; low word = 0 → VDP read command
///   move.l d0,4(a6)       ; send to VDP control port
/// ```
///
/// Returns the 32-bit VDP read command (`(vram_addr << 16) | 0x0000`).
#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn pause_menu_tile_save_row_vdp_read_cmd_loc_695C(
    camera_y_pos: u16,
    tile_col: u16,
    row: u16,
) -> u32 {
    // ASM loc_6948: d7 = Camera_Y_pos + $40
    // ASM loc_695C: d7 advances by 8 each row (addi.w #8,d7 at kid.asm:6687)
    let y_pixel = camera_y_pos
        .wrapping_add(PAUSE_MENU_Y_OFFSET_LOC_6948)
        .wrapping_add(row.wrapping_mul(8));

    // ASM loc_695C: lsr.w #3,d3; andi.w #$1F,d3; lsl.w #7,d3
    let plane_row = (y_pixel >> 3) & 0x1F;
    let row_byte_offset = plane_row << 7;

    // ASM loc_6948: add.w d0,d0 doubles tile_col to byte offset, stored in d6
    let col_byte_offset = tile_col.wrapping_mul(2);

    // ASM loc_695C: add.w d3,d0; ori.w #0,d0; swap d0; move.w #0,d0
    let vram_addr = row_byte_offset.wrapping_add(col_byte_offset);
    u32::from(vram_addr) << 16
}

/// Compute the column-reset VDP read command for the wrapped second segment.
///
/// ASM loc_69AE (kid.asm:6703): after reading the first segment of a
/// plane-edge-wrapped row, the column offset is cleared to read the
/// remaining words from column 0 of the same plane row.
/// ```text
///   andi.l #$FF80FFFF,d0  ; clear bits 6-0 of high word (column byte offset)
///   move.l d0,4(a6)       ; set new VDP read address at column 0
/// ```
#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn pause_menu_tile_save_wrap_column_reset_loc_69AE(vdp_read_cmd: u32) -> u32 {
    // ASM loc_69AE: andi.l #$FF80FFFF,d0
    // High word bits 6-0 are the column byte offset within the 128-byte row.
    // Clearing them resets to column 0 while preserving the row address.
    vdp_read_cmd & 0xFF80_FFFF
}

/// Describes one row of the pause menu tile save, including VDP addresses
/// and word counts for both segments of a potentially wrapped read.
///
/// ASM loc_695C-loc_69C0 (kid.asm:6664-6712): each of the 5 rows either
/// reads 8 longwords straight (16 words, non-wrapped) or splits into two
/// word-read loops (loc_69A0/loc_69AE + loc_69C0) when the menu wraps
/// around the 64-tile plane edge.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct PauseMenuTileSaveRow {
    /// VDP read command for the start of this row.
    pub(crate) vdp_read_cmd: u32,
    /// When `None`: non-wrapped — 8 longword reads (16 words) via `move.l (a6),(a1)+`.
    /// When `Some((first_words, second_cmd, second_words))`: wrapped read.
    /// `first_words` word-reads from `vdp_read_cmd`, then `second_cmd` (column-reset)
    /// for `second_words` word-reads. Total always = 16 words.
    pub(crate) wrap_segments: Option<(u16, u32, u16)>,
}

/// Compute the full 5-row tile save plan for the pause menu.
///
/// ASM loc_695C (kid.asm:6664-6712): combines the per-row VDP address
/// computation with the wrap layout from loc_693A to produce a complete
/// read plan. The ROM saves these tiles into `Decompression_Buffer` so
/// they can be restored when the menu closes.
///
/// - `loc_695C`: row address + dispatch (non-wrapped vs wrapped)
/// - `loc_69A0`: wrapped setup — unpack d2 into (first_words, wrapped_words)
/// - `loc_69AE`: first-segment word-read loop + column reset
/// - `loc_69C0`: second-segment word-read loop
#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn pause_menu_tile_save_plan_loc_695C(
    camera_x_pos: u16,
    camera_y_pos: u16,
) -> [PauseMenuTileSaveRow; 5] {
    let layout = pause_menu_plane_column_layout_loc_693A(camera_x_pos);

    core::array::from_fn(|i| {
        let row = i as u16;
        let vdp_read_cmd =
            pause_menu_tile_save_row_vdp_read_cmd_loc_695C(camera_y_pos, layout.tile_col, row);

        let wrap_segments = layout.wrap_split.map(|(first_words, wrapped_words)| {
            // ASM loc_69AE: after reading first_words, column-reset for second segment
            let second_cmd = pause_menu_tile_save_wrap_column_reset_loc_69AE(vdp_read_cmd);
            (first_words, second_cmd, wrapped_words)
        });

        PauseMenuTileSaveRow {
            vdp_read_cmd,
            wrap_segments,
        }
    })
}

/// Pause menu text selection at `restart_text` (kid.asm:6741-6745).
///
/// ASM:
/// ```text
///   cmpi.w #1,(Number_Lives).w   ; lives > 1?
///   bgt.s  +                     ; yes → Restart Round
///   lea    (MapUnc_PauseMenu_GiveUp).l,a1
///   bra.s  ++
/// + lea    (MapUnc_PauseMenu_Restart).l,a1
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum PauseMenuTextSelectionRestartText {
    /// Number_Lives <= 1 — player cannot restart, only give up.
    GiveUp,
    /// Number_Lives > 1 — player can restart the round.
    RestartRound,
}

/// Select the pause menu text mapping based on remaining lives.
///
/// ASM `restart_text` (kid.asm:6741-6745): `cmpi.w #1,(Number_Lives).w;
/// bgt.s +` — if lives > 1, use "Restart Round" mappings; otherwise "Give Up".
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn pause_menu_text_selection_restart_text(
    number_lives: u16,
) -> PauseMenuTextSelectionRestartText {
    if number_lives > 1 {
        PauseMenuTextSelectionRestartText::RestartRound
    } else {
        PauseMenuTextSelectionRestartText::GiveUp
    }
}

/// Compute the VDP VRAM write command for one row of the pause menu text write.
///
/// ASM loc_6A1E (kid.asm:6750-6760): per-row address computation within the
/// 5-row text write loop. Identical to `pause_menu_tile_save_row_vdp_read_cmd_loc_695C`
/// except `ori.w #$4000,d0` sets the VDP write bit (vs `ori.w #0` for read).
/// ```text
///   move.w d7,d3          ; d7 = Camera_Y_pos + $40 + row*8
///   lsr.w  #3,d3          ; pixel → tile row
///   andi.w #$1F,d3        ; wrap to 32-row plane height
///   lsl.w  #7,d3          ; × 128 bytes = nametable row byte offset
///   move.w d6,d0          ; d6 = tile_col * 2 (column byte offset)
///   add.w  d3,d0          ; combine row + column
///   ori.w  #$4000,d0      ; VDP WRITE mode
///   swap   d0             ; high word = VRAM address + write flag
///   move.w #0,d0          ; low word = 0 → VDP write command
///   move.l d0,4(a6)       ; send to VDP control port
/// ```
///
/// Returns the 32-bit VDP write command (`((vram_addr | $4000) << 16) | 0x0000`).
#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn pause_menu_text_write_row_vdp_write_cmd_loc_6A1E(
    camera_y_pos: u16,
    tile_col: u16,
    row: u16,
) -> u32 {
    // ASM restart_text: d7 = Camera_Y_pos + $40
    // ASM loc_6A1E: d7 advances by 8 each row (addi.w #8,d7 at kid.asm:6771)
    let y_pixel = camera_y_pos
        .wrapping_add(PAUSE_MENU_Y_OFFSET_LOC_6948)
        .wrapping_add(row.wrapping_mul(8));

    // ASM loc_6A1E: lsr.w #3,d3; andi.w #$1F,d3; lsl.w #7,d3
    let plane_row = (y_pixel >> 3) & 0x1F;
    let row_byte_offset = plane_row << 7;

    // ASM restart_text: add.w d0,d0 doubles tile_col to byte offset, stored in d6
    let col_byte_offset = tile_col.wrapping_mul(2);

    // ASM loc_6A1E: add.w d3,d0; ori.w #$4000,d0; swap d0; move.w #0,d0
    let vram_addr = row_byte_offset.wrapping_add(col_byte_offset);
    u32::from(vram_addr | 0x4000) << 16
}

/// Describes one row of the pause menu text write, including VDP addresses
/// and word counts for both segments of a potentially wrapped write.
///
/// ASM loc_6A1E-loc_6A76 (kid.asm:6750-6795): each of the 5 rows either
/// writes 8 longwords straight (16 words, non-wrapped) or splits into two
/// word-write loops (loc_6A5A/loc_6A64 + loc_6A76) when the menu wraps
/// around the 64-tile plane edge.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct PauseMenuTextWriteRow {
    /// VDP write command for the start of this row.
    pub(crate) vdp_write_cmd: u32,
    /// When `None`: non-wrapped — 8 longword writes (16 words) via `move.l (a1)+,(a6)`.
    /// When `Some((first_words, second_cmd, second_words))`: wrapped write.
    /// `first_words` word-writes from `vdp_write_cmd`, then `second_cmd` (column-reset
    /// with $4000 write bit) for `second_words` word-writes. Total always = 16 words.
    pub(crate) wrap_segments: Option<(u16, u32, u16)>,
}

/// Compute the full 5-row text write plan for the pause menu.
///
/// ASM loc_69D2–loc_6A76 (kid.asm:6716-6795): combines the column layout
/// (loc_69D2/loc_69EC — identical to loc_693A) with per-row VDP write
/// addresses (loc_6A1E — `ori.w #$4000` for write mode) to produce a
/// complete write plan for the pause menu text mappings.
///
/// - `loc_69D2`: column layout + dispatch (non-wrapped vs loc_69EC wrapped)
/// - `loc_69EC`: wrapped layout — same as loc_693A
/// - `loc_6A1E`: per-row VDP write address + dispatch (non-wrapped vs loc_6A5A)
/// - `loc_6A5A`: wrapped setup — unpack d2 into (first_words, wrapped_words)
/// - `loc_6A64`: first-segment word-write loop + column reset (`andi.l #$FF80FFFF`)
/// - `loc_6A76`: second-segment word-write loop
#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn pause_menu_text_write_plan_loc_69D2(
    camera_x_pos: u16,
    camera_y_pos: u16,
) -> [PauseMenuTextWriteRow; 5] {
    // ASM loc_69D2/loc_69EC: identical column layout to loc_693A.
    let layout = pause_menu_plane_column_layout_loc_693A(camera_x_pos);

    core::array::from_fn(|i| {
        let row = i as u16;
        let vdp_write_cmd =
            pause_menu_text_write_row_vdp_write_cmd_loc_6A1E(camera_y_pos, layout.tile_col, row);

        let wrap_segments = layout.wrap_split.map(|(first_words, wrapped_words)| {
            // ASM loc_6A64: after writing first_words, column-reset for second segment.
            // andi.l #$FF80FFFF,d0 — identical to loc_69AE in the tile save path.
            let second_cmd = pause_menu_tile_save_wrap_column_reset_loc_69AE(vdp_write_cmd);
            (first_words, second_cmd, wrapped_words)
        });

        PauseMenuTextWriteRow {
            vdp_write_cmd,
            wrap_segments,
        }
    })
}

/// Describes one row of the pause menu tile restore, including VDP write
/// addresses and word counts for both segments of a potentially wrapped write.
///
/// ASM loc_6BA4–loc_6BFC (kid.asm:6911-6956): each of the 5 rows either
/// writes 8 longwords straight (16 words, non-wrapped) or splits into two
/// word-write loops (loc_6BE0/loc_6BEA + loc_6BFC) when the menu wraps
/// around the 64-tile plane edge. Data source is `Decompression_Buffer`
/// (tiles saved by the tile save loop at loc_695C).
///
/// Structurally identical to [`PauseMenuTextWriteRow`] — the ROM reuses the
/// same VDP write address computation and column-reset mask.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct PauseMenuTileRestoreRow {
    /// VDP write command for the start of this row.
    pub(crate) vdp_write_cmd: u32,
    /// When `None`: non-wrapped — 8 longword writes (16 words) via `move.l (a1)+,(a6)`.
    /// When `Some((first_words, second_cmd, second_words))`: wrapped write.
    /// `first_words` word-writes from `vdp_write_cmd`, then `second_cmd` (column-reset
    /// with $4000 write bit) for `second_words` word-writes. Total always = 16 words.
    pub(crate) wrap_segments: Option<(u16, u32, u16)>,
}

/// Compute the full 5-row tile restore plan for the pause menu.
///
/// ASM loc_6B68–loc_6BFC (kid.asm:6880-6956): after unpausing, writes the
/// saved tile data from `Decompression_Buffer` back to VDP plane A. This is
/// the inverse of the tile save at loc_695C.
///
/// - `loc_6B68`: column layout computation — identical to loc_693A
/// - `loc_6B82`: wrapped layout — same as loc_693A wrap path
///   (`d1 = $40 - d0` first_words, `d2 = $10 - d1` wrapped_words, swap-pack)
/// - `loc_6B90`: common setup — `d7 = Camera_Y_pos + $40`, `a1 = Decompression_Buffer`,
///   `d5 = 4` (5 rows), `d6 = tile_col * 2`
/// - `loc_6BA4`: per-row VDP write address — identical to loc_6A1E
///   (`ori.w #$4000` for VRAM write mode)
/// - `loc_6BE0`: wrapped dispatch — unpack d2 into (first_words, wrapped_words)
/// - `loc_6BEA`: first-segment word-write loop + column reset (`andi.l #$FF80FFFF`)
/// - `loc_6BFC`: second-segment word-write loop
#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn pause_menu_tile_restore_plan_loc_6B68(
    camera_x_pos: u16,
    camera_y_pos: u16,
) -> [PauseMenuTileRestoreRow; 5] {
    // ASM loc_6B68/loc_6B82: identical column layout to loc_693A.
    let layout = pause_menu_plane_column_layout_loc_693A(camera_x_pos);

    core::array::from_fn(|i| {
        let row = i as u16;
        // ASM loc_6BA4: same VDP write address as loc_6A1E (ori.w #$4000).
        let vdp_write_cmd =
            pause_menu_text_write_row_vdp_write_cmd_loc_6A1E(camera_y_pos, layout.tile_col, row);

        let wrap_segments = layout.wrap_split.map(|(first_words, wrapped_words)| {
            // ASM loc_6BEA: after writing first_words, column-reset for second segment.
            // andi.l #$FF80FFFF,d0 — identical to loc_69AE/loc_6A64.
            let second_cmd = pause_menu_tile_save_wrap_column_reset_loc_69AE(vdp_write_cmd);
            (first_words, second_cmd, wrapped_words)
        });

        PauseMenuTileRestoreRow {
            vdp_write_cmd,
            wrap_segments,
        }
    })
}

/// VDP write commands for the pause menu arrow cursor indicators.
///
/// ASM loc_6A84 (kid.asm:6799-6826): after drawing the pause menu text, this
/// routine clears Up/Down from `Ctrl_Pressed`, resets `Pause_Option` to 0, and
/// computes two VDP VRAM write commands for the arrow cursor positions:
/// - `upper_arrow_cmd`: at pixel row `Camera_Y_pos + $48` (the "Continue" line)
/// - `lower_arrow_cmd`: at pixel row `Camera_Y_pos + $58` (the "Restart/Give Up" line)
///
/// Both use column `Camera_X_pos + $68` (the right edge of the menu text area).
///
/// ```text
///   move.w (Camera_X_pos).w,d0     ; column pixel position
///   addi.w #$68,d0                 ; offset to arrow column
///   lsr.w  #3,d0                   ; pixel → tile column
///   andi.w #$3F,d0                 ; wrap to 64-tile plane width
///   add.w  d0,d0                   ; tile → byte offset (×2)
///   move.w d0,d1                   ; d1 = same column byte offset
///   ; --- upper arrow (d0): Camera_Y + $48 ---
///   move.w (Camera_Y_pos).w,d3
///   addi.w #$48,d3
///   lsr.w  #3,d3; andi.w #$1F,d3; lsl.w #7,d3
///   add.w  d3,d0
///   ori.w  #$4000,d0; swap d0; move.w #0,d0
///   ; --- lower arrow (d1): Camera_Y + $58 ---
///   move.w (Camera_Y_pos).w,d3
///   addi.w #$58,d3
///   lsr.w  #3,d3; andi.w #$1F,d3; lsl.w #7,d3
///   add.w  d3,d1
///   ori.w  #$4000,d1; swap d1; move.w #0,d1
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct PauseMenuArrowCursorCmdsLoc6A84 {
    /// VDP write command for the upper arrow position (Camera_Y + $48).
    pub(crate) upper_arrow_cmd: u32,
    /// VDP write command for the lower arrow position (Camera_Y + $58).
    pub(crate) lower_arrow_cmd: u32,
}

/// Compute the VDP write commands for the pause menu arrow cursor positions.
///
/// ASM loc_6A84 (kid.asm:6799-6826): both arrows share the same column
/// (`Camera_X_pos + $68` → tile → byte offset) but differ in row:
/// `$48` for upper, `$58` for lower.
#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn pause_menu_arrow_cursor_cmds_loc_6A84(
    camera_x_pos: u16,
    camera_y_pos: u16,
) -> PauseMenuArrowCursorCmdsLoc6A84 {
    // ASM loc_6A84: addi.w #$68,d0; lsr.w #3,d0; andi.w #$3F,d0; add.w d0,d0
    let col_byte_offset = {
        let tile_col = (camera_x_pos.wrapping_add(0x68) >> 3) & 0x3F;
        tile_col * 2
    };

    // Helper: compute VDP write command for a given Y pixel offset.
    // ASM: addi.w #offset,d3; lsr.w #3,d3; andi.w #$1F,d3; lsl.w #7,d3;
    //      add.w d3,d0; ori.w #$4000,d0; swap d0; move.w #0,d0
    let vdp_write_cmd = |y_offset: u16| -> u32 {
        let plane_row = (camera_y_pos.wrapping_add(y_offset) >> 3) & 0x1F;
        let row_byte_offset = plane_row << 7;
        let vram_addr = row_byte_offset.wrapping_add(col_byte_offset);
        u32::from(vram_addr | 0x4000) << 16
    };

    PauseMenuArrowCursorCmdsLoc6A84 {
        upper_arrow_cmd: vdp_write_cmd(0x48),
        lower_arrow_cmd: vdp_write_cmd(0x58),
    }
}

/// Result of a pause loop cursor-move action.
///
/// ASM Game_Paused_Loop (kid.asm:6829-6857): when Up or Down is pressed and
/// the cursor can move, the loop writes two tile words to the arrow positions
/// and updates `Pause_Option`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct PauseMenuCursorMove {
    /// Tile word written to the upper arrow VDP address.
    pub(crate) upper_arrow_tile: u16,
    /// Tile word written to the lower arrow VDP address.
    pub(crate) lower_arrow_tile: u16,
    /// New value of `Pause_Option` after the move.
    pub(crate) new_pause_option: u8,
}

/// Arrow tile constants used by the pause menu cursor.
///
/// ASM Game_Paused_Loop (kid.asm:6839-6856):
/// - `$867C`: the arrow-present tile (points at the selected option)
/// - `$8678`: the arrow-absent tile (blank / no arrow)
const PAUSE_ARROW_PRESENT_TILE: u16 = 0x867C;
const PAUSE_ARROW_ABSENT_TILE: u16 = 0x8678;

/// Handle the Down button press inside the pause loop.
///
/// ASM loc_6B0E (kid.asm:6846-6857):
/// ```text
///   bclr  #Button_Down,(Ctrl_Pressed).w
///   beq.s Game_Paused_ChkStart     ; not pressed → skip
///   tst.b (Pause_Option).w
///   bne.s Game_Paused_ChkStart     ; already on restart → skip
///   ; move cursor down
///   move.l d0,4(a6)               ; upper arrow VDP cmd
///   move.w #$8678,(a6)            ; clear upper arrow
///   move.l d1,4(a6)               ; lower arrow VDP cmd
///   move.w #$867C,(a6)            ; draw lower arrow
///   addi.b #1,(Pause_Option).w    ; Pause_Option = 1
/// ```
///
/// Returns `Some(move)` if the cursor moved down, `None` if the button
/// was not pressed or the cursor was already at the bottom.
#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn pause_loop_down_handler_loc_6B0E(
    down_pressed: bool,
    pause_option: u8,
) -> Option<PauseMenuCursorMove> {
    // ASM loc_6B0E: bclr #Button_Down; beq.s skip; tst.b Pause_Option; bne.s skip
    if !down_pressed || pause_option != 0 {
        return None;
    }
    // ASM loc_6B0E: move.w #$8678 to upper, #$867C to lower, addi.b #1
    Some(PauseMenuCursorMove {
        upper_arrow_tile: PAUSE_ARROW_ABSENT_TILE,
        lower_arrow_tile: PAUSE_ARROW_PRESENT_TILE,
        new_pause_option: pause_option.wrapping_add(1),
    })
}

/// Handle the Up button press inside the pause loop.
///
/// ASM Game_Paused_Loop (kid.asm:6832-6843):
/// ```text
///   bclr  #Button_Up,(Ctrl_Pressed).w
///   beq.s loc_6B0E                 ; not pressed → check Down
///   tst.b (Pause_Option).w
///   beq.s Game_Paused_ChkStart     ; already on continue → skip
///   ; move cursor up
///   move.l d0,4(a6)               ; upper arrow VDP cmd
///   move.w #$867C,(a6)            ; draw upper arrow
///   move.l d1,4(a6)               ; lower arrow VDP cmd
///   move.w #$8678,(a6)            ; clear lower arrow
///   subi.b #1,(Pause_Option).w    ; Pause_Option = 0
/// ```
///
/// Returns `Some(move)` if the cursor moved up, `None` if the button
/// was not pressed or the cursor was already at the top.
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn pause_loop_up_handler_game_paused_loop(
    up_pressed: bool,
    pause_option: u8,
) -> Option<PauseMenuCursorMove> {
    // ASM Game_Paused_Loop: bclr #Button_Up; beq.s loc_6B0E; tst.b Pause_Option; beq.s skip
    if !up_pressed || pause_option == 0 {
        return None;
    }
    // ASM: move.w #$867C to upper, #$8678 to lower, subi.b #1
    Some(PauseMenuCursorMove {
        upper_arrow_tile: PAUSE_ARROW_PRESENT_TILE,
        lower_arrow_tile: PAUSE_ARROW_ABSENT_TILE,
        new_pause_option: pause_option.wrapping_sub(1),
    })
}

/// Result of the Start button press inside the pause loop.
///
/// ASM Game_Paused_ChkStart (kid.asm:6859-6878): when Start is pressed, the
/// outcome depends on `Pause_Option`:
/// - 0 → continue game (unpause, fall through to tile restore at loc_6B68)
/// - non-zero → restart/give up: decrement `Number_Lives`, then either
///   restart the level (lives remaining) or set `Game_Paused = 3` at loc_6B62
///   (last life → game over path).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum PauseMenuStartResult {
    /// Start not pressed — continue the pause loop.
    NotPressed,
    /// `Pause_Option == 0` → continue game (unpause).
    ContinueGame,
    /// `Pause_Option != 0`, lives remaining after decrement → restart level.
    RestartLevel {
        /// `Number_Lives` after the `subi.w #1` decrement.
        remaining_lives: u16,
    },
    /// ASM loc_6B62 (kid.asm:6877): `Pause_Option != 0` and `Number_Lives - 1 == 0`
    /// → last life lost, set `Game_Paused = 3` (game over / "bummer" voice path).
    LastLifeGameOver,
}

/// Handle the Start button press inside the pause loop.
///
/// ASM Game_Paused_ChkStart + loc_6B62 (kid.asm:6859-6878):
/// ```text
///   bclr  #Button_Start,(Ctrl_Pressed).w
///   beq.s Game_Paused_Loop         ; not pressed → loop
///   tst.b (Pause_Option).w
///   beq.s loc_6B68                 ; option 0 → continue game
///   ; restart / give up
///   subi.w #1,(Number_Lives).w
///   beq.s  loc_6B62               ; lives == 0 → last life
///   ; lives > 0 → restart level ...
/// loc_6B62:
///   move.b #3,(Game_Paused).w      ; signal game over path
/// ```
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn pause_loop_start_handler_chk_start(
    start_pressed: bool,
    pause_option: u8,
    number_lives: u16,
) -> PauseMenuStartResult {
    // ASM: bclr #Button_Start; beq.s Game_Paused_Loop
    if !start_pressed {
        return PauseMenuStartResult::NotPressed;
    }
    // ASM: tst.b (Pause_Option).w; beq.s loc_6B68
    if pause_option == 0 {
        return PauseMenuStartResult::ContinueGame;
    }
    // ASM: subi.w #1,(Number_Lives).w; beq.s loc_6B62
    let remaining = number_lives.wrapping_sub(1);
    if remaining == 0 {
        // ASM loc_6B62: move.b #3,(Game_Paused).w
        PauseMenuStartResult::LastLifeGameOver
    } else {
        PauseMenuStartResult::RestartLevel {
            remaining_lives: remaining,
        }
    }
}

/// Result of the pause menu exit after tile restore completes.
///
/// ASM loc_6C0A (kid.asm:6958-6983): after `Pause_RestorePalette`, the routine
/// branches on `Game_Paused`:
/// - `== 3` → last life game over: play "bummer" voice, set Kid dying
///   (`$38(a4) = 4`, `$3A(a4) = $30`), set `Number_Lives = 1`, clear
///   `Game_Paused`.
/// - `!= 3` → normal unpause at loc_6C3E: clear `Game_Paused`, call
///   `sub_E1338`, reload Eyeclops beam art via `off_Load_EyclopsBeamArt`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum PauseMenuUnpauseExitLoc6C0A {
    /// ASM loc_6C0A, Game_Paused == 3: last life lost during pause menu.
    /// Play sfx_Voice_bummer, set Kid action = 4 / param = $30, lives = 1.
    LastLifeGameOver {
        /// Kid object `$38` field — ASM: `move.w #4,$38(a4)`.
        kid_action: u16,
        /// Kid object `$3A` field — ASM: `move.w #$30,$3A(a4)`.
        kid_action_param: u16,
        /// ASM: `move.w #1,(Number_Lives).w`.
        number_lives: u16,
    },
    /// ASM loc_6C3E (kid.asm:6976): normal unpause path.
    /// Clears `Game_Paused`, calls `sub_E1338`, reloads Eyeclops beam art.
    NormalUnpause,
}

/// Determine the pause menu exit path after tile restore and palette restore.
///
/// ASM loc_6C0A (kid.asm:6958):
/// ```text
///   bsr.w  Pause_RestorePalette
///   cmpi.b #3,(Game_Paused).w
///   bne.s  loc_6C3E
///   ; last life game over path
///   moveq  #sfx_Voice_bummer,d0
///   jsr    (j_PlaySound).l
///   move.l (Addr_GfxObject_Kid).w,a4
///   move.w #4,$38(a4)
///   move.w #$30,$3A(a4)
///   move.w #1,(Number_Lives).w
///   sf     (Game_Paused).w
///   rts
/// loc_6C3E:
///   sf     (Game_Paused).w
///   jsr    (sub_E1338).l
///   lea    (off_Load_EyclopsBeamArt).l,a0
///   move.l (a0),a0
///   jsr    (a0)
///   rts
/// ```
#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn pause_menu_unpause_exit_loc_6C0A(game_paused: u8) -> PauseMenuUnpauseExitLoc6C0A {
    // ASM loc_6C0A: cmpi.b #3,(Game_Paused).w; bne.s loc_6C3E
    if game_paused == 3 {
        // ASM loc_6C0A game-over path: move.w #4,$38(a4); move.w #$30,$3A(a4);
        // move.w #1,(Number_Lives).w; sf (Game_Paused).w
        PauseMenuUnpauseExitLoc6C0A::LastLifeGameOver {
            kid_action: 4,
            kid_action_param: 0x30,
            number_lives: 1,
        }
    } else {
        // ASM loc_6C3E: sf (Game_Paused).w; jsr (sub_E1338).l;
        // lea (off_Load_EyclopsBeamArt).l,a0; move.l (a0),a0; jsr (a0)
        PauseMenuUnpauseExitLoc6C0A::NormalUnpause
    }
}

/// Darken a palette buffer by halving each RGB channel independently.
///
/// ASM Pause_DarkenPalette / loc_6C70 (kid.asm:7001-7036):
/// ```text
///   ; First span: 15 colours (d7=$E) starting at Palette_Buffer[0]
/// loc_6C70:
///   move.w (a4),d6       ; d6 = original word
///   move.w d6,d5
///   andi.w #$F,d6        ; blue channel
///   lsr.w  #1,d6         ; blue >>= 1
///   move.w d5,d4
///   andi.w #$F0,d4       ; green channel
///   lsr.w  #1,d4         ; green >>= 1 (spills into blue nibble)
///   andi.w #$F0,d4       ; mask back to green position
///   andi.w #$F00,d5      ; red channel
///   lsr.w  #1,d5         ; red >>= 1 (spills into green nibble)
///   andi.w #$F00,d5      ; mask back to red position
///   or.w   d4,d5
///   or.w   d5,d6
///   move.w d6,(a4)+
///   dbf    d7,loc_6C70
///   addq.w #2,a4         ; skip one word (colour 15 of first palette line)
///   moveq  #$2F,d7       ; 48 colours
/// loc_6C9E:              ; identical halving loop for remaining 48 colours
///   ...
/// ```
///
/// Layout: darkens colours 0–14 of the first palette line, skips colour 15,
/// then darkens all 48 colours of palette lines 1–3.
#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn pause_darken_palette_loc_6C70(palette_buffer: &mut [u16; 0x40]) {
    // ASM loc_6C70: inner halving — each 4-bit channel is independently halved
    // via lsr.w #1 + mask.
    fn halve_colour(word: u16) -> u16 {
        // ASM: andi.w #$F,d6; lsr.w #1,d6
        let blue = (word & 0x000F) >> 1;
        // ASM: andi.w #$F0,d4; lsr.w #1,d4; andi.w #$F0,d4
        let green = ((word & 0x00F0) >> 1) & 0x00F0;
        // ASM: andi.w #$F00,d5; lsr.w #1,d5; andi.w #$F00,d5
        let red = ((word & 0x0F00) >> 1) & 0x0F00;
        red | green | blue
    }

    // First span: 15 colours (d7=$E → 15 iterations)
    // ASM: lea (Palette_Buffer).l,a4; moveq #$E,d7
    for word in &mut palette_buffer[..15] {
        *word = halve_colour(*word);
    }
    // ASM: addq.w #2,a4 — skip colour 15 (index 15)
    // Second span: 48 colours (d7=$2F → 48 iterations)
    // ASM: moveq #$2F,d7; loc_6C9E identical loop
    for word in &mut palette_buffer[16..] {
        *word = halve_colour(*word);
    }
}

/// ASM loc_7508 (kid.asm:7652–7660): at level start, the Kid object entry
/// point loads a helmet-specific palette into palette line 3.
///
/// `move.w (Current_Helmet).w,d0; add.w d0,d0` indexes the `off_80F2` table
/// (10 word offsets, one per helmet) to find a Data_Index pointer.  That
/// pointer references 12 words of BGR palette data copied to
/// `Palette_Buffer+$62` (palette line 3, colours $31–$3C):
///
/// ```text
/// lea (Data_Index).l,a0          ; base ROM asset pointer table
/// lea off_80F2(pc),a1            ; per-helmet palette offset table
/// add.w (a1,d0.w),a0             ; a0 → helmet palette data
/// move.l (a0),a0                 ; dereference Data_Index entry
/// lea (Palette_Buffer+$62).l,a1  ; dest = palette line 3 + $22
/// moveq #$B,d0                   ; 12 words ($B+1)
/// ```
///
/// ASM loc_7526 (kid.asm:7662–7664) is the inner loop:
/// `move.w (a0)+,(a1)+; dbf d0,loc_7526`.
///
/// The native port does not model CRAM palette buffers at the rendering level,
/// so this function documents the table structure for auditing.
///
/// Returns `(dest_word_offset, word_count)`.
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) const fn helmet_palette_copy_params_loc_7508() -> (usize, usize) {
    // Palette_Buffer+$62 = byte offset $62 / 2 = word offset $31.
    // 12 words = $B+1 (dbf counter $B = 12 iterations).
    (0x31, 12)
}

/// Number of entries in the ASM `off_80F2` helmet palette offset table
/// (kid.asm:8850–8859).  One entry per helmet ID (0=TheKid through
/// 9=Micromax).
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) const HELMET_PALETTE_TABLE_ENTRIES_LOC_7508: usize = 10;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct IoPortFlagsSub6E24 {
    pub(crate) bit_7_clear_flag: bool,
    pub(crate) bit_6_set_flag: bool,
}

#[allow(non_snake_case)]
#[allow(dead_code)]
pub(crate) fn read_io_port_a10001_flags_sub_6E24(port_value: u8) -> IoPortFlagsSub6E24 {
    // ASM sub_6E24 (kid.asm:7224): after the `$A10001` read, `btst #7,d0`
    // followed by `seq` stores `$FF` when bit 7 is clear, while `btst #6,d0`
    // followed by `sne` stores `$FF` when bit 6 is set.
    IoPortFlagsSub6E24 {
        bit_7_clear_flag: port_value & 0x80 == 0,
        bit_6_set_flag: port_value & 0x40 != 0,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct TitleTextBurstAnimationFrameSub1B652 {
    pub(crate) duration_frames: u16,
    pub(crate) sprite_label: Option<&'static str>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct TitleTextBurstSpriteSub1B652 {
    pub(crate) gfx_slot_base: u32,
    pub(crate) palette_line: u8,
    pub(crate) x_pos: u16,
    pub(crate) y_pos: u16,
    pub(crate) vram_tile: u16,
    pub(crate) is_moved: bool,
    pub(crate) priority_flag: bool,
    pub(crate) animation_frames: [TitleTextBurstAnimationFrameSub1B652; 5],
    pub(crate) animation_command: u8,
    pub(crate) animation_rewind_distance: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct TitleMenuPromptSpriteSub1C54A {
    pub(crate) gfx_slot_base: u32,
    pub(crate) priority_flag: bool,
    pub(crate) x_pos: u16,
    pub(crate) y_pos: u16,
    pub(crate) sprite_label: &'static str,
    pub(crate) vram_tile: u16,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct TitleMenuSlidingSpriteSub1C572 {
    pub(crate) gfx_slot_base: u32,
    pub(crate) priority_flag: bool,
    pub(crate) x_pos: u16,
    pub(crate) y_pos: u16,
    pub(crate) sprite_label: &'static str,
    pub(crate) vram_tile: u16,
    pub(crate) x_velocity_per_frame: i16,
    pub(crate) initial_hibernate_frames: u16,
    pub(crate) moving_frames: u16,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct IntroSequence1PlayerTitleSpriteLoc1ADEA {
    pub(crate) gfx_slot_base: u32,
    pub(crate) priority_flag: bool,
    pub(crate) display_enabled: bool,
    pub(crate) sprite_label: &'static str,
    pub(crate) vram_tile: u16,
    pub(crate) target_x_pos: u16,
    pub(crate) target_y_pos: u16,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct EndSequenceDisplayRefreshSub1D830 {
    pub(crate) camera_y_scroll: i16,
    pub(crate) h_scroll_value: i16,
    pub(crate) h_scroll_word_pairs: u16,
    pub(crate) panel_toggle_byte: u8,
    pub(crate) left_panel_x: i16,
    pub(crate) right_panel_x: i16,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct EndSequenceHorizontalScrollUploadLoc1DA50 {
    pub(crate) vram_word_addr: u16,
    pub(crate) repeated_word: i16,
    pub(crate) word_write_count: u16,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct EndSequenceTileFillLoc1DA6A {
    pub(crate) fill_word: u16,
    pub(crate) word_write_count: u16,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct EndSequenceVramZeroFillLoc1DA7E {
    pub(crate) vram_word_addr: u16,
    pub(crate) fill_word: u16,
    pub(crate) word_write_count: u16,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct EndSequenceFadeLoopLoc1D270 {
    pub(crate) loop_seed: u16,
    pub(crate) fade_palette_byte_offset: u16,
    pub(crate) fade_iterations: u16,
    pub(crate) hibernate_frames: u16,
    pub(crate) post_loop_vram_word_addr: u16,
    pub(crate) post_loop_subroutine: &'static str,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct EndSequencePanelSetupLoc1D306 {
    pub(crate) palette_words_copied: u16,
    pub(crate) first_panel_x: u16,
    pub(crate) first_panel_y: u16,
    pub(crate) second_panel_x: u16,
    pub(crate) second_panel_y: u16,
    pub(crate) stripe_row_seed: u16,
    pub(crate) stripe_column_seed: u16,
    pub(crate) scroll_counter_seed: u16,
    pub(crate) scroll_distance_seed: u16,
    pub(crate) panel_toggle_byte: u8,
    pub(crate) panel_delta: u16,
    pub(crate) tilemap_row_upload_subroutine: &'static str,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct EndSequencePanelScrollLoopLoc1D3A0 {
    pub(crate) display_refresh_subroutine: &'static str,
    pub(crate) hibernate_subroutine: &'static str,
    pub(crate) panel_step_subroutine: &'static str,
    pub(crate) row_upload_subroutine: &'static str,
    pub(crate) scroll_counter_seed: u16,
    pub(crate) scroll_distance_seed: u16,
    pub(crate) scroll_step: u16,
    pub(crate) active_iterations: u16,
    pub(crate) panel_refresh_iterations: u16,
}

#[cfg_attr(not(test), allow(dead_code))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct EndSequenceHideGlyphsLoc1D7F0 {
    pub(crate) glyph_pointer_table_base: u16,
    pub(crate) loop_seed: u16,
    pub(crate) hidden_glyphs: u16,
    pub(crate) cleared_flag_offset: u16,
}

#[cfg_attr(not(test), allow(dead_code))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct EndSequencePageFadeOuterLoopLoc1D80C {
    pub(crate) outer_loop_seed: u16,
    pub(crate) outer_loop_iterations: u16,
    pub(crate) fade_palette_byte_offset: u16,
    pub(crate) fade_setup_subroutine: &'static str,
    pub(crate) inner_loop_seed: u16,
    pub(crate) inner_loop_iterations: u16,
    pub(crate) inner_loop_entry_label: &'static str,
}

#[cfg_attr(not(test), allow(dead_code))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct EndSequencePageFadeInnerLoopLoc1D814 {
    pub(crate) loop_seed: u16,
    pub(crate) iterations: u16,
    pub(crate) refresh_subroutine: &'static str,
    pub(crate) hibernate_subroutine: &'static str,
    pub(crate) next_outer_label: &'static str,
}

#[cfg_attr(not(test), allow(dead_code))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct EndSequenceCreditsGlyphCacheLoc1D3D0 {
    pub(crate) glyph_count: u16,
    pub(crate) glyph_pointer_table_base: u16,
    pub(crate) glyph_pointer_slot_offsets_in_write_order: [u16; 6],
    pub(crate) glyph_init_subroutine: &'static str,
    pub(crate) platform_layout_label: &'static str,
    pub(crate) text_script_label: &'static str,
}

#[cfg_attr(not(test), allow(dead_code))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct EndSequencePageBreakDispatchLoc1D40E {
    pub(crate) branch_target: &'static str,
    pub(crate) has_padding_nop: bool,
}

#[cfg_attr(not(test), allow(dead_code))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct EndSequencePageBreakWaitLoopLoc1D418 {
    pub(crate) wait_seed: u16,
    pub(crate) wait_iterations: u16,
    pub(crate) refresh_subroutine: &'static str,
    pub(crate) hibernate_subroutine: &'static str,
    pub(crate) hide_glyphs_subroutine: &'static str,
    pub(crate) fade_subroutine: &'static str,
    pub(crate) stripe_subroutine: &'static str,
    pub(crate) palette_buffer_word_offset: u16,
    pub(crate) palette_buffer_longword: u32,
    pub(crate) palette_upload_subroutine: &'static str,
    pub(crate) glyph_layout_source: &'static str,
    pub(crate) glyph_load_subroutine: &'static str,
}

#[cfg_attr(not(test), allow(dead_code))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct EndSequenceTerminatorExitLoc1D450 {
    pub(crate) wait_seed: u16,
    pub(crate) wait_iterations: u16,
    pub(crate) refresh_subroutine: &'static str,
    pub(crate) hibernate_subroutine: &'static str,
    pub(crate) palette_clear_dbf_seed: u16,
    pub(crate) palette_clear_words: u16,
    pub(crate) wait_for_vint_subroutine: &'static str,
    pub(crate) idle_subroutine: &'static str,
    pub(crate) palette_upload_subroutine: &'static str,
    pub(crate) sets_unknown_flag_fbce: bool,
    pub(crate) clears_word_fbcc: bool,
    pub(crate) clears_game_mode: bool,
    pub(crate) clears_number_lives: bool,
    pub(crate) continuation_pointer_source: &'static str,
}

#[cfg_attr(not(test), allow(dead_code))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct EndSequenceTextLoopLoc1D3F0 {
    pub(crate) page_text_subroutine: &'static str,
    pub(crate) page_break_word: u16,
    pub(crate) page_break_wait_seed: u16,
    pub(crate) page_break_wait_iterations: u16,
    pub(crate) page_break_refresh_subroutine: &'static str,
    pub(crate) page_break_hide_glyphs_subroutine: &'static str,
    pub(crate) page_break_fade_subroutine: &'static str,
    pub(crate) page_break_stripe_subroutine: &'static str,
    pub(crate) page_break_glyph_load_subroutine: &'static str,
    pub(crate) page_break_platform_layout_source: &'static str,
    pub(crate) terminator_word: u16,
    pub(crate) terminator_branch: &'static str,
}

#[cfg_attr(not(test), allow(dead_code))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct ContinueScreenOptionalFadeLocD372 {
    pub(crate) score_object_fades: bool,
    pub(crate) bonus_object_fades: bool,
    pub(crate) display_timer_seed: u16,
    pub(crate) falling_velocity: i16,
}

#[cfg_attr(not(test), allow(dead_code))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct ContinueScreenMandatoryFadeLocD386 {
    pub(crate) continue_text_fades: bool,
    pub(crate) continue_number_fades: bool,
    pub(crate) text_display_timer_seed: u16,
    pub(crate) number_display_timer_seed: u16,
    pub(crate) falling_velocity: i16,
    pub(crate) post_fade_wait_seed: u16,
}

#[cfg_attr(not(test), allow(dead_code))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct ContinueTimeoutTitlePaletteBackupLocD40A {
    pub(crate) palette_backup_dbf_seed: u16,
    pub(crate) palette_backup_longwords: u16,
}

#[cfg_attr(not(test), allow(dead_code))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct ContinueTimeoutTitlePaletteClearLocD418 {
    pub(crate) palette_clear_dbf_seed: u16,
    pub(crate) palette_clear_words: u16,
    pub(crate) fade_progress_seed: u16,
}

#[cfg_attr(not(test), allow(dead_code))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct ContinueTimeoutTitleFadeStepLocD428 {
    pub(crate) fade_progress: u16,
    pub(crate) pending_palette_0_mask: u32,
    pub(crate) pending_palette_1_mask: u32,
    pub(crate) return_to_title: bool,
}

#[cfg_attr(not(test), allow(dead_code))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct ContinueTimeoutTitleReturnLocD44C {
    pub(crate) final_refresh_subroutine: &'static str,
    pub(crate) idle_subroutine: &'static str,
    pub(crate) palette_upload_subroutine: &'static str,
    pub(crate) fade_helper_subroutine: &'static str,
    pub(crate) clears_continue_fade_colour: bool,
    pub(crate) sets_teleport_flag_fc36: bool,
    pub(crate) tail_subroutine: &'static str,
}

#[cfg_attr(not(test), allow(dead_code))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct EndSequenceStripeLoopLoc1CDE4 {
    pub(crate) start_column: u16,
    pub(crate) iteration_seed: u16,
    pub(crate) iterations: u16,
    pub(crate) stripe_columns_per_iteration: u16,
    pub(crate) final_next_column: u16,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct EndSequenceStripeFillLoopLoc1CE1C {
    pub(crate) fill_word: u16,
    pub(crate) iteration_seed: u16,
    pub(crate) iterations: u16,
    pub(crate) z80_guard_subroutines: [&'static str; 2],
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct OptionsMenuTextDrawLoc1CE3A {
    pub(crate) first_line_row: u16,
    pub(crate) first_line_highlighted: bool,
    pub(crate) second_line_row: u16,
    pub(crate) second_line_text_label: &'static str,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct OptionsMenuControlsHeaderLoc1CEB2 {
    pub(crate) highlighted: bool,
    pub(crate) first_line_x: u16,
    pub(crate) first_line_y: u16,
    pub(crate) first_line_text_label: &'static str,
    pub(crate) second_line_x: u16,
    pub(crate) second_line_y: u16,
    pub(crate) second_line_text_label: &'static str,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct OptionsMenuControlsBindingsLoc1CED6 {
    pub(crate) highlighted: bool,
    pub(crate) selection_tile_word: u16,
    pub(crate) binding_label_rows: [u16; 3],
    pub(crate) binding_text_labels: [&'static str; 3],
    pub(crate) button_badge_rows: [u16; 3],
    pub(crate) button_badge_text_labels: [&'static str; 3],
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct OptionsMenuControlsBindingLoopLoc1CEF8 {
    pub(crate) dbf_seed: u16,
    pub(crate) iterations: u16,
    pub(crate) row_start: u16,
    pub(crate) row_step: u16,
    pub(crate) binding_text_labels: [&'static str; 3],
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct OptionsMenuSpeedTextLoc1CF78 {
    pub(crate) highlighted: bool,
    pub(crate) header_x: u16,
    pub(crate) header_y: u16,
    pub(crate) header_text_label: &'static str,
    pub(crate) mode_x: u16,
    pub(crate) mode_y: u16,
    pub(crate) mode_text_label: &'static str,
    pub(crate) prompt_x: u16,
    pub(crate) prompt_y: u16,
    pub(crate) prompt_text_label: &'static str,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct OptionsMenuExitTextLoc1CFFE {
    pub(crate) highlighted: bool,
    pub(crate) text_x: u16,
    pub(crate) text_y: u16,
    pub(crate) text_label: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct DrawTextLineGlyphLoopLoc1D068 {
    pub(crate) glyph_tiles: Vec<u16>,
    pub(crate) bytes_consumed: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct DrawTextLineTerminatorLoc1D07A {
    pub(crate) flushes_vdp_guard: bool,
    pub(crate) returns_after_flush: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct CreditsGlyphPreviewEntryLoc1D0A2 {
    pub(crate) vdp_command: u32,
    pub(crate) tile_word: Option<u16>,
    pub(crate) z80_guard_subroutines: Option<[&'static str; 2]>,
    pub(crate) refresh_dbf_seed: Option<u16>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct CreditsGlyphPreviewRefreshLoopLoc1D0C2 {
    pub(crate) dbf_seed: u16,
    pub(crate) iterations: u16,
    pub(crate) subroutine_order: [&'static str; 5],
    pub(crate) preserves_register_window: bool,
    pub(crate) next_vdp_command: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct LevelLayoutReadPassLoc1CCC2 {
    pub(crate) dbf_seed: u16,
    pub(crate) iterations: u16,
    pub(crate) subroutine_order: [&'static str; 2],
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct LevelLayoutCopyToVramLoc1CCDE {
    pub(crate) dbf_seed: u16,
    pub(crate) words_copied: u16,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct LevelLayoutDelayLoc1CCE8 {
    pub(crate) dbf_seed: u16,
    pub(crate) iterations: u16,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct EndSequenceBurstSpriteInitSub1D86C {
    pub(crate) callback_label: &'static str,
    pub(crate) x_pos: i16,
    pub(crate) y_pos: i16,
    pub(crate) priority: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct EndSequenceFallingSpriteInitSub1D8B4 {
    pub(crate) gfx_slot_base: u32,
    pub(crate) display_enabled: bool,
    pub(crate) palette_line: u8,
    pub(crate) vram_tile: u16,
    pub(crate) x_pos: u16,
    pub(crate) y_pos: u16,
    pub(crate) sprite_mapping: u16,
    pub(crate) initial_hibernate_frames: u16,
    pub(crate) fall_frames: u16,
    pub(crate) first_frame_y_delta_fp: u32,
    pub(crate) last_frame_y_delta_fp: u32,
    pub(crate) total_y_delta_fp: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Prize10000PointsStarSpawnLoc10B3A {
    pub(crate) palette_line: u8,
    pub(crate) display_enabled: bool,
    pub(crate) sprite_mapping_label: &'static str,
    pub(crate) x_pos: u16,
    pub(crate) y_pos: u16,
    pub(crate) x_velocity_fp: i32,
    pub(crate) y_velocity_fp: i32,
    pub(crate) is_moved: bool,
    pub(crate) timer: u16,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Prize10000PointsStarDispatchLoc10B76 {
    pub(crate) should_step_slot: bool,
    pub(crate) next_slot_offset: u16,
    pub(crate) finished_scan: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Prize10000PointsStarAdvanceLoc10BCE {
    pub(crate) next_slot_offset: u16,
    pub(crate) finished_scan: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Prize10000PointsStarStepLoc10B74 {
    pub(crate) latched_screen_pos: Option<(i16, i16)>,
    pub(crate) x_pos: i16,
    pub(crate) is_moved: bool,
    pub(crate) timer: u16,
    pub(crate) delete_prize_worker: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct TitleTextChameleonRowWriteSub1B71A {
    pub(crate) vram_word_address: u16,
    pub(crate) filler_word: Option<u16>,
    pub(crate) words: Vec<u16>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct TitleTextChameleonFrameUploadSub1B71A {
    pub(crate) tilemap_label: &'static str,
    pub(crate) rows: Vec<TitleTextChameleonRowWriteSub1B71A>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct TitleTextWordWritesSub1B7B6 {
    pub(crate) base_word_address: u16,
    pub(crate) top_words: [u16; 2],
    pub(crate) bottom_words: [u16; 2],
}

#[cfg_attr(not(test), allow(dead_code))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct IntroTextInitialHibernateLoc1C7AE {
    pub(crate) hibernate_frames: u8,
}

impl TitleTextWordWritesSub1B7B6 {
    #[cfg_attr(not(test), allow(dead_code))]
    pub(crate) fn bottom_word_addresses(self) -> [u16; 2] {
        [self.base_word_address + 0x50, self.base_word_address + 0x52]
    }
}

#[allow(non_snake_case)]
#[allow(dead_code)]
pub(crate) fn intro_text_initial_hibernate_loc_1C7AE() -> IntroTextInitialHibernateLoc1C7AE {
    // ASM loc_1C7A0 -> loc_1C7AE (kid.asm:31404-31414): the intro text worker
    // seeds `d0=1`, hibernates one object frame, then decrements `d0` to zero
    // before the byte parser can consume the text stream.
    IntroTextInitialHibernateLoc1C7AE {
        hibernate_frames: 1,
    }
}

#[allow(non_snake_case)]
#[allow(dead_code)]
pub(crate) fn title_text_burst_sprite_sub_1B652() -> TitleTextBurstSpriteSub1B652 {
    // ASM sub_1B652 (kid.asm:29882): allocate the fixed `$2010000` gfx slot,
    // seed the title-text burst sprite at `(x,y)=(#$2E,#$40)`, then hand
    // Init_Animation the five-step `stru_1B68A` script.
    TitleTextBurstSpriteSub1B652 {
        gfx_slot_base: 0x0201_0000,
        palette_line: 2,
        x_pos: 0x002E,
        y_pos: 0x0040,
        vram_tile: 0x054C,
        is_moved: true,
        priority_flag: true,
        animation_frames: [
            TitleTextBurstAnimationFrameSub1B652 {
                duration_frames: 4,
                sprite_label: Some("unk_E11B6"),
            },
            TitleTextBurstAnimationFrameSub1B652 {
                duration_frames: 4,
                sprite_label: Some("unk_E11BE"),
            },
            TitleTextBurstAnimationFrameSub1B652 {
                duration_frames: 100,
                sprite_label: None,
            },
            TitleTextBurstAnimationFrameSub1B652 {
                duration_frames: 4,
                sprite_label: Some("unk_E11A6"),
            },
            TitleTextBurstAnimationFrameSub1B652 {
                duration_frames: 4,
                sprite_label: Some("unk_E11AE"),
            },
        ],
        animation_command: 2,
        animation_rewind_distance: 0x15,
    }
}

#[allow(non_snake_case)]
#[allow(dead_code)]
pub(crate) fn title_menu_prompt_sprite_sub_1C54A() -> TitleMenuPromptSpriteSub1C54A {
    // ASM sub_1C54A (kid.asm:31288): allocate gfx slot `$1000000`, set the
    // sprite priority flag at `$13(a1)`, then seed the fixed title prompt
    // sprite at `(x,y)=(#$81,#$C8)` with tile `$140` and `unk_E109E`.
    TitleMenuPromptSpriteSub1C54A {
        gfx_slot_base: 0x0100_0000,
        priority_flag: true,
        x_pos: 0x0081,
        y_pos: 0x00C8,
        sprite_label: "unk_E109E",
        vram_tile: 0x0140,
    }
}

#[allow(non_snake_case)]
#[allow(dead_code)]
pub(crate) fn title_menu_sliding_sprites_sub_1C572() -> [TitleMenuSlidingSpriteSub1C572; 2] {
    // ASM sub_1C572 (kid.asm:31303) seeds the left title sprite, then
    // loc_1C58C seeds the mirrored right sprite. Both hibernate for `$3C`
    // frames, slide 1 px/frame for `#$41+1` hibernated iterations, then loop
    // forever on `Hibernate_Object_1Frame`.
    [
        TitleMenuSlidingSpriteSub1C572 {
            gfx_slot_base: 0x0200_0004,
            priority_flag: true,
            x_pos: 0x003A,
            y_pos: 0x00C8,
            sprite_label: "unk_E10A6",
            vram_tile: 0x0140,
            x_velocity_per_frame: 1,
            initial_hibernate_frames: 0x003C,
            moving_frames: 0x0042,
        },
        TitleMenuSlidingSpriteSub1C572 {
            gfx_slot_base: 0x0200_0004,
            priority_flag: true,
            x_pos: 0x00DE,
            y_pos: 0x00C8,
            sprite_label: "unk_E10AE",
            vram_tile: 0x0140,
            x_velocity_per_frame: -1,
            initial_hibernate_frames: 0x003C,
            moving_frames: 0x0042,
        },
    ]
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct TitleTextIntroSpriteLoc1BB10 {
    pub(crate) gfx_slot_base: u32,
    pub(crate) visibility_flag: bool,
    pub(crate) palette_line: u8,
    pub(crate) y_pos: u16,
    pub(crate) vram_tile: u16,
    pub(crate) sprite_label: &'static str,
}

#[allow(non_snake_case)]
#[allow(dead_code)]
pub(crate) fn title_text_intro_sprite_loc_1BB10() -> TitleTextIntroSpriteLoc1BB10 {
    // ASM loc_1BB10 (kid.asm:30405-30412): allocate gfx slot `$02000000`, set
    // visibility (`st $13(a3)`), set `palette_line(a3)=2`, seed `y_pos=$94`,
    // `vram_tile=$4BD`, and the sprite offset `LnkTo_unk_E0ED6-Data_Index`.
    // Called from loc_1B6AC (kid.asm:29922) with `$27` in `$44(a0)`, and from
    // Obj_TitleMenu (kid.asm:30383).
    TitleTextIntroSpriteLoc1BB10 {
        gfx_slot_base: 0x0200_0000,
        visibility_flag: true,
        palette_line: 2,
        y_pos: 0x0094,
        vram_tile: 0x04BD,
        sprite_label: "LnkTo_unk_E0ED6-Data_Index",
    }
}

#[allow(non_snake_case)]
#[allow(dead_code)]
pub(crate) fn title_text_intro_sprite_x_pos_loc_1BB36_and_loc_1BB44(counter: u16) -> i16 {
    // ASM loc_1BB36 (kid.asm:30414-30418) and loc_1BB44 (kid.asm:30420-30425):
    // The intro text sprite slides in from the left. Each frame:
    // - loc_1BB36: d0 = #$10C (268), d1 = counter from $44(a5)
    // - If d1 == 0, skip decrement; otherwise subq.w #1,$44(a5)
    // - loc_1BB44: d1 <<= 3 (multiply by 8), then d0 -= d1
    // - x_pos = 268 - (counter * 8)
    // Initial counter is $27 (39) set at loc_1B6AC line 29923.
    // First visible frame: x_pos = 268 - (39 * 8) = -44 (off-screen left).
    // Final frame (counter=0): x_pos = 268.
    // ASM loading order matters: d0 is set first, then d1, so decrement happens
    // before the calculation when counter > 0.
    let effective_counter = if counter == 0 {
        0
    } else {
        counter.saturating_sub(1)
    };
    0x010C_u16 as i16 - (effective_counter * 8) as i16
}

#[allow(non_snake_case)]
#[allow(dead_code)]
pub(crate) fn intro_sequence_1_player_title_sprite_loc_1ADEA(
    current_player_index: u8,
) -> IntroSequence1PlayerTitleSpriteLoc1ADEA {
    // ASM loc_1ADEA (kid.asm:29034): after the shared intro-sequence-1 title
    // worker allocates `$3000004` into `a1`, player 0 keeps `d7=$81` while
    // player 1 increments it to `$82`; both paths then index `off_1AC2A` and
    // seed tile `$409` with target position `(x,y)=(#$160,#$C8)`.
    IntroSequence1PlayerTitleSpriteLoc1ADEA {
        gfx_slot_base: 0x0300_0004,
        priority_flag: true,
        display_enabled: true,
        sprite_label: match current_player_index {
            0 => "LnkTo_unk_C87E8-Data_Index",
            _ => "LnkTo_unk_C87F0-Data_Index",
        },
        vram_tile: 0x0409,
        target_x_pos: 0x0160,
        target_y_pos: 0x00C8,
    }
}

#[allow(non_snake_case)]
#[allow(dead_code)]
pub(crate) fn intro_sequence_1_left_title_sprite_step_loc_1AE10(
    current_x_pos: i16,
    target_x_pos: i16,
) -> i16 {
    // ASM loc_1AE10 (kid.asm:29044): each loop iteration hibernates for one
    // frame, compares `x_pos(a3)` against `d0`, and only runs `addi.w #4` on
    // the left title sprite while it still sits short of the `$60` target.
    if current_x_pos == target_x_pos {
        current_x_pos
    } else {
        current_x_pos.wrapping_add(4)
    }
}

#[allow(non_snake_case)]
#[allow(dead_code)]
pub(crate) fn intro_sequence_1_right_title_sprite_step_loc_1AE20(
    current_x_pos: i16,
    target_x_pos: i16,
) -> i16 {
    // ASM loc_1AE20 (kid.asm:29050): after the shared loc_1AE10 left-sprite
    // check, the mirrored title sprite compares `$1A(a1)` against `d1` and
    // only runs `subi.w #4,$1A(a1)` while it still sits right of the `$D0`
    // destination.
    if current_x_pos == target_x_pos {
        current_x_pos
    } else {
        current_x_pos.wrapping_sub(4)
    }
}

#[cfg_attr(not(test), allow(dead_code))]
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct IntroSequence2LayoutCopyLoc1B012 {
    pub(crate) destination_addr: u32,
    pub(crate) copied_longwords: Vec<u32>,
}

#[allow(non_snake_case)]
#[allow(dead_code)]
pub(crate) fn intro_sequence_2_layout_copy_loc_1B012(
    level_layout_longwords: &[u32],
) -> IntroSequence2LayoutCopyLoc1B012 {
    // ASM loc_1B012 (kid.asm:29283): seed `d0 = #$2BF`, copy one longword per
    // pass from `Level_Layout` into `$FFFFB152`, and let `dbf d0,loc_1B012`
    // run inclusively for `#$2BF + 1 = $2C0` longwords.
    const COPY_LONGWORDS: usize = 0x02C0;
    assert!(
        level_layout_longwords.len() >= COPY_LONGWORDS,
        "intro-sequence-2 layout copy needs at least $2C0 longwords"
    );

    IntroSequence2LayoutCopyLoc1B012 {
        destination_addr: 0xFFFF_B152,
        copied_longwords: level_layout_longwords[..COPY_LONGWORDS].to_vec(),
    }
}

#[allow(non_snake_case)]
#[allow(dead_code)]
pub(crate) fn intro_sequence_5_art_source_loc_1B100(title_variant: u8) -> &'static str {
    // ASM Load_IntroSequence5 -> loc_1B100 (kid.asm:29335-29351): start from
    // `byte_213D9`, switch to `byte_219C1` when `($FFFFFC82).w` is non-zero,
    // and only fall through to `byte_216B7` when that byte is neither 0 nor 1.
    match title_variant {
        0 => "byte_213D9",
        1 => "byte_219C1",
        _ => "byte_216B7",
    }
}

#[allow(non_snake_case)]
#[allow(dead_code)]
pub(crate) fn title_screen_art_source_loc_1B2D8(title_variant: u8) -> &'static str {
    // ASM sub_1B2A4 -> loc_1B2D8 (kid.asm:29479-29484): start from
    // `byte_26E3D`, switch to `byte_2927C` when `($FFFFFC82).w == 1`, and
    // otherwise use `byte_282C1` for the remaining non-zero title variants
    // before the shared VRAM decompress tail at `loc_1B2D8`.
    match title_variant {
        0 => "byte_26E3D",
        1 => "byte_2927C",
        _ => "byte_282C1",
    }
}

const TITLE_SCREEN_TILE_ATTRIBUTE_BYTES_LOC_1B322: [u8; 64] = [
    0x11, 0x11, 0x10, 0x10, 0x00, 0x10, 0x00, 0x11, 0x00, 0x10, 0x00, 0x10, 0x00, 0x10, 0x00, 0x10,
    0x00, 0x10, 0x00, 0x10, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x10, 0x00, 0x00, 0x01, 0x10, 0x00, 0x00, 0x10, 0x10, 0x00, 0x00, 0x00, 0x10, 0x00, 0x00,
    0x00, 0x10, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
];

#[cfg_attr(not(test), allow(dead_code))]
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct TitleScreenTileAttributeUploadLoc1B322 {
    pub(crate) vram_word_addr: u16,
    pub(crate) words: Vec<u16>,
}

#[allow(non_snake_case)]
#[allow(dead_code)]
pub(crate) fn title_screen_tile_attribute_upload_loc_1B322()
-> TitleScreenTileAttributeUploadLoc1B322 {
    // ASM sub_1B2A4 -> loc_1B322 (kid.asm:29499-29503): after
    // `move.l #vdpComm($F000,VRAM,WRITE),4(a6)`, the code reads 32 words from
    // `unk_1B32C` and streams each word to the VDP data port before the
    // inclusive `dbf d0,loc_1B322` terminates.
    TitleScreenTileAttributeUploadLoc1B322 {
        vram_word_addr: 0xF000,
        words: TITLE_SCREEN_TILE_ATTRIBUTE_BYTES_LOC_1B322
            .chunks_exact(2)
            .map(|word| u16::from_be_bytes([word[0], word[1]]))
            .collect(),
    }
}

#[cfg_attr(not(test), allow(dead_code))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct TitleScreenLightningFlashUploadLoc1B432 {
    pub(crate) flash_frame: u8,
    pub(crate) art_byte_offset: u16,
    pub(crate) vram_word_addr: u16,
    pub(crate) longword_write_count: u16,
}

#[allow(non_snake_case)]
#[allow(dead_code)]
pub(crate) fn title_screen_lightning_flash_upload_loc_1B432(
    flash_frame: u8,
) -> TitleScreenLightningFlashUploadLoc1B432 {
    // ASM sub_1B41C / loc_1B432 (kid.asm:29642-29654): copy `d0` into `d4`,
    // `lsl.w #7,d4`, add that byte offset to `ArtUnc_2A4D6`, then stream one
    // longword per pass before the inclusive `dbf d4,loc_1B432` terminates.
    TitleScreenLightningFlashUploadLoc1B432 {
        flash_frame,
        art_byte_offset: u16::from(flash_frame) << 7,
        vram_word_addr: 0x17A0,
        longword_write_count: 0x001F + 1,
    }
}

#[cfg_attr(not(test), allow(dead_code))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct TitleScreenLightningFlashStepLoc1B37C {
    pub(crate) hibernate_frames: u16,
    pub(crate) flash_frame: u8,
    pub(crate) play_thunderstorm: bool,
}

#[cfg_attr(not(test), allow(dead_code))]
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct TitleScreenLightningSequenceLoc1B37C {
    pub(crate) flash_vram_word_addr: u16,
    pub(crate) flash_longword_write_count: u16,
    pub(crate) steps: Vec<TitleScreenLightningFlashStepLoc1B37C>,
}

#[allow(non_snake_case)]
#[allow(dead_code)]
pub(crate) fn title_screen_lightning_sequence_loc_1B37C() -> TitleScreenLightningSequenceLoc1B37C {
    // ASM loc_1B37C (kid.asm:29587-29635): hibernate for 3 frames before each
    // flash step, route `d0` through `sub_1B41C`, and interleave five
    // `sub_1B36C` thunder calls around a 45-step `(d1 & 3) + 1` flash loop.
    // ASM sub_1B41C (kid.asm:29642-29650): each flash uploads 32 longwords
    // from `ArtUnc_2A4D6 + (d0 << 7)` into VRAM word address `$17A0`.
    let mut steps = Vec::with_capacity(55);
    steps.extend([
        TitleScreenLightningFlashStepLoc1B37C {
            hibernate_frames: 3,
            flash_frame: 1,
            play_thunderstorm: false,
        },
        TitleScreenLightningFlashStepLoc1B37C {
            hibernate_frames: 3,
            flash_frame: 0,
            play_thunderstorm: true,
        },
        TitleScreenLightningFlashStepLoc1B37C {
            hibernate_frames: 3,
            flash_frame: 2,
            play_thunderstorm: false,
        },
        TitleScreenLightningFlashStepLoc1B37C {
            hibernate_frames: 3,
            flash_frame: 0,
            play_thunderstorm: true,
        },
    ]);
    for d1 in (0..=0x002C_u16).rev() {
        steps.push(TitleScreenLightningFlashStepLoc1B37C {
            hibernate_frames: 3,
            flash_frame: ((d1 & 3) + 1) as u8,
            play_thunderstorm: false,
        });
    }
    steps.extend([
        TitleScreenLightningFlashStepLoc1B37C {
            hibernate_frames: 3,
            flash_frame: 0,
            play_thunderstorm: true,
        },
        TitleScreenLightningFlashStepLoc1B37C {
            hibernate_frames: 3,
            flash_frame: 1,
            play_thunderstorm: false,
        },
        TitleScreenLightningFlashStepLoc1B37C {
            hibernate_frames: 3,
            flash_frame: 0,
            play_thunderstorm: true,
        },
        TitleScreenLightningFlashStepLoc1B37C {
            hibernate_frames: 3,
            flash_frame: 2,
            play_thunderstorm: false,
        },
        TitleScreenLightningFlashStepLoc1B37C {
            hibernate_frames: 3,
            flash_frame: 0,
            play_thunderstorm: true,
        },
    ]);

    let flash_upload = title_screen_lightning_flash_upload_loc_1B432(0);

    TitleScreenLightningSequenceLoc1B37C {
        flash_vram_word_addr: flash_upload.vram_word_addr,
        flash_longword_write_count: flash_upload.longword_write_count,
        steps,
    }
}

#[cfg_attr(not(test), allow(dead_code))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct IntroSequence6LongwordFillLoc1B170 {
    pub(crate) vram_word_addr: u16,
    pub(crate) fill_longword: u32,
    pub(crate) longword_write_count: u16,
}

#[allow(non_snake_case)]
#[allow(dead_code)]
pub(crate) fn intro_sequence_6_longword_fill_loc_1B170() -> IntroSequence6LongwordFillLoc1B170 {
    // ASM Load_IntroSequence6 -> loc_1B170 (kid.asm:29371-29373): after
    // `move.l #vdpComm($1780,VRAM,WRITE),4(a6)`, `moveq #7,d0`, and
    // `move.l #$11111111,d1`, the loop writes that longword to the VDP data
    // port once per pass before the inclusive `dbf d0,loc_1B170` test.
    IntroSequence6LongwordFillLoc1B170 {
        vram_word_addr: 0x1780,
        fill_longword: 0x1111_1111,
        longword_write_count: 0x0007 + 1,
    }
}

#[cfg_attr(not(test), allow(dead_code))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct IntroSequence6WordFillLoc1B182 {
    pub(crate) vram_word_addr: u16,
    pub(crate) fill_word: u16,
    pub(crate) word_write_count: u16,
}

#[allow(non_snake_case)]
#[allow(dead_code)]
pub(crate) fn intro_sequence_6_tilemap_clear_loc_1B182() -> IntroSequence6WordFillLoc1B182 {
    // ASM Load_IntroSequence6 -> loc_1B182 (kid.asm:29377-29379): after
    // `move.l #vdpComm($0000,VRAM,WRITE),4(a6)` and `move.w #$7FF,d0`, the
    // loop writes `#$20BC` to the VDP data port once per pass before the
    // inclusive `dbf d0,loc_1B182` test. That fills 2048 words from VRAM word
    // address `$0000`.
    IntroSequence6WordFillLoc1B182 {
        vram_word_addr: 0x0000,
        fill_word: 0x20BC,
        word_write_count: 0x07FF + 1,
    }
}

#[allow(non_snake_case)]
#[allow(dead_code)]
pub(crate) fn title_screen_tilemap_clear_loc_1B258() -> IntroSequence6WordFillLoc1B182 {
    // ASM Load_TitleScreen -> loc_1B258 (kid.asm:29451-29453): the title
    // screen reuses the same `move.w #$7FF,d0` / `move.w #$20BC,(a6)` clear
    // loop as Load_IntroSequence6 before it loads the title art and menu
    // objects.
    intro_sequence_6_tilemap_clear_loc_1B182()
}

#[cfg_attr(not(test), allow(dead_code))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct IntroSequence6ScrollBootstrapLoc1B23A {
    pub(crate) camera_y_pos: i32,
    pub(crate) scroll_fill_longword: u32,
    pub(crate) longword_write_count: u16,
}

#[allow(non_snake_case)]
#[allow(dead_code)]
pub(crate) fn intro_sequence_6_scroll_bootstrap_loc_1B23A(
    hidden_camera_word: u16,
    level_layout_word: u16,
) -> IntroSequence6ScrollBootstrapLoc1B23A {
    // ASM sub_1B222 -> loc_1B23A (kid.asm:29425-29438): copy the word at
    // `($FFFFA656).w` into `(Camera_Y_pos).w`, seed `d5` from `(Level_Layout).w`,
    // `swap d5`, then write that longword into `(Horiz_Scroll_Buffer).l`
    // once per pass before the inclusive `dbf d4,loc_1B23A` test.
    IntroSequence6ScrollBootstrapLoc1B23A {
        camera_y_pos: i32::from(hidden_camera_word as i16) << 16,
        scroll_fill_longword: u32::from(level_layout_word) << 16,
        longword_write_count: 0x00DF + 1,
    }
}

#[cfg_attr(not(test), allow(dead_code))]
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct IntroSequence6HiddenTitleShellLoc1B1E8 {
    pub(crate) scroll_bootstrap: IntroSequence6ScrollBootstrapLoc1B23A,
    pub(crate) background_no_scroll: bool,
    pub(crate) object_handlers: [&'static str; 3],
}

#[allow(non_snake_case)]
#[allow(dead_code)]
pub(crate) fn intro_sequence_6_hidden_title_shell_loc_1B1E8(
    hidden_camera_word: u16,
    level_layout_word: u16,
) -> IntroSequence6HiddenTitleShellLoc1B1E8 {
    // ASM loc_1B1E8 (kid.asm:29411-29423): call `sub_1B222`, set
    // `(Background_NoScrollFlag).w`, then allocate three gfx object slots in
    // order for `loc_1BAB0`, `loc_1B37C`, and `sub_1B43A`.
    IntroSequence6HiddenTitleShellLoc1B1E8 {
        scroll_bootstrap: intro_sequence_6_scroll_bootstrap_loc_1B23A(
            hidden_camera_word,
            level_layout_word,
        ),
        background_no_scroll: true,
        object_handlers: ["loc_1BAB0", "loc_1B37C", "sub_1B43A"],
    }
}

const TITLE_SCREEN_HIDDEN_SHELL_FRAMES_LOC_1B446: [u8; 29] = [
    0, 1, 0, 1, 0, 1, 2, 1, 2, 3, 2, 3, 2, 3, 4, 3, 4, 3, 4, 5, 4, 5, 4, 5, 6, 5, 6, 5, 6,
];

#[cfg_attr(not(test), allow(dead_code))]
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct TitleScreenHiddenShellScriptLoc1B446 {
    pub(crate) initial_hibernate_frames: u16,
    pub(crate) loop_hibernate_frames: u16,
    pub(crate) title_frames: Vec<u8>,
    pub(crate) sentinel: i8,
}

#[allow(non_snake_case)]
#[allow(dead_code)]
pub(crate) fn title_screen_hidden_shell_script_loc_1B446() -> TitleScreenHiddenShellScriptLoc1B446 {
    // ASM sub_1B43A / loc_1B446 (kid.asm:29658-29676): wait `$2A` frames,
    // then loop on a 3-frame hibernate, `move.b (a0)+,d4`, and `bsr.w
    // sub_1BF24` until the next byte is negative and branches to loc_1B45A.
    TitleScreenHiddenShellScriptLoc1B446 {
        initial_hibernate_frames: 0x002A,
        loop_hibernate_frames: 0x0003,
        title_frames: TITLE_SCREEN_HIDDEN_SHELL_FRAMES_LOC_1B446.to_vec(),
        sentinel: -1,
    }
}

#[cfg_attr(not(test), allow(dead_code))]
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct TitleScreenHiddenShellPaletteRestoreLoc1B45A {
    pub(crate) blackout_hibernate_frames: u16,
    pub(crate) palette_word_count: u16,
    pub(crate) blackout_frame: u8,
    pub(crate) interstitial_hibernate_frames: u16,
    pub(crate) reload_title_art: bool,
    pub(crate) spawned_handlers: [&'static str; 3],
}

#[allow(non_snake_case)]
#[allow(dead_code)]
pub(crate) fn title_screen_hidden_shell_palette_restore_loc_1B45A()
-> TitleScreenHiddenShellPaletteRestoreLoc1B45A {
    // ASM loc_1B45A (kid.asm:29677-29709): after the `$FF` sentinel, wait
    // `$39` frames, copy all `$40` palette words from `Palette_Buffer` into
    // `Palette_Buffer_2` while clearing the original via loc_1B470, hibernate
    // one frame, show frame 7, rebuild the title shell, restore those same
    // `$40` words via loc_1B494, hibernate one more frame, reload title art,
    // then allocate the bottom-boundary, `loc_1B6AC`, and `loc_1B5CE` workers.
    TitleScreenHiddenShellPaletteRestoreLoc1B45A {
        blackout_hibernate_frames: 0x0039,
        palette_word_count: 0x003F + 1,
        blackout_frame: 7,
        interstitial_hibernate_frames: 1,
        reload_title_art: true,
        spawned_handlers: [
            "GfxObjects_Collision_BottomBoundary4",
            "loc_1B6AC",
            "loc_1B5CE",
        ],
    }
}

#[cfg_attr(not(test), allow(dead_code))]
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct TitleScreenHiddenCameraRampLoc1B51A {
    pub(crate) initial_hibernate_frames: u16,
    pub(crate) iteration_count: u16,
    pub(crate) level_layout_step_fp: i32,
    pub(crate) hidden_camera_step_fp: i32,
    pub(crate) level_layout_values_fp: Vec<i32>,
    pub(crate) hidden_camera_values_fp: Vec<i32>,
    pub(crate) deletes_current_object: bool,
}

#[allow(non_snake_case)]
#[allow(dead_code)]
pub(crate) fn title_screen_hidden_camera_ramp_loc_1B51A(
    level_layout_fp: i32,
    hidden_camera_fp: i32,
) -> TitleScreenHiddenCameraRampLoc1B51A {
    // ASM GfxObjects_Collision_BottomBoundary4 -> loc_1B51A
    // (kid.asm:29764-29777): after the initial `$A`-frame hibernate, copy
    // `(Level_Layout).w` and `($FFFFA656).w` into `d0/d1`, arithmetic-shift
    // both right by 4, then run 16 inclusive `dbf` passes that hibernate one
    // frame, subtract those fixed-point deltas from both accumulators, and
    // call `sub_1B222` before deleting the worker.
    let level_layout_step_fp = level_layout_fp >> 4;
    let hidden_camera_step_fp = hidden_camera_fp >> 4;
    let iteration_count = 0x000F + 1;
    let mut next_level_layout_fp = level_layout_fp;
    let mut next_hidden_camera_fp = hidden_camera_fp;
    let mut level_layout_values_fp = Vec::with_capacity(usize::from(iteration_count));
    let mut hidden_camera_values_fp = Vec::with_capacity(usize::from(iteration_count));

    for _ in 0..usize::from(iteration_count) {
        next_level_layout_fp = next_level_layout_fp.wrapping_sub(level_layout_step_fp);
        next_hidden_camera_fp = next_hidden_camera_fp.wrapping_sub(hidden_camera_step_fp);
        level_layout_values_fp.push(next_level_layout_fp);
        hidden_camera_values_fp.push(next_hidden_camera_fp);
    }

    TitleScreenHiddenCameraRampLoc1B51A {
        initial_hibernate_frames: 0x000A,
        iteration_count,
        level_layout_step_fp,
        hidden_camera_step_fp,
        level_layout_values_fp,
        hidden_camera_values_fp,
        deletes_current_object: true,
    }
}

#[cfg_attr(not(test), allow(dead_code))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct TitleScreenTileStripeClearLoc1B53E {
    pub(crate) vram_word_addr: u16,
    pub(crate) fill_longword: u32,
    pub(crate) longword_write_count: u16,
}

#[allow(non_snake_case)]
#[allow(dead_code)]
pub(crate) fn title_screen_tile_stripe_clear_loc_1B53E() -> TitleScreenTileStripeClearLoc1B53E {
    // ASM sub_1B532 -> loc_1B53E (kid.asm:29783-29790): seed VRAM write
    // `$1780`, clear `d1`, and stream eight zero longwords into the VDP data
    // port before the later `EniDec`/tile upload rebuilds the title stripe.
    TitleScreenTileStripeClearLoc1B53E {
        vram_word_addr: 0x1780,
        fill_longword: 0x0000_0000,
        longword_write_count: 0x0007 + 1,
    }
}

#[cfg_attr(not(test), allow(dead_code))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct TitleShellArtIndexLoc1BF3E {
    pub(crate) table_entry_size: u16,
    pub(crate) tile_pattern_index_zero: u16,
    pub(crate) tile_pattern_index_nonzero: u16,
}

#[allow(non_snake_case)]
#[allow(dead_code)]
pub(crate) fn title_shell_art_index_loc_1BF3E() -> TitleShellArtIndexLoc1BF3E {
    // ASM sub_1BF24 -> loc_1BF3E (kid.asm:30751-30758): after conditional index
    // adjustment based on `$FFFFFC82` region flags, multiply `d4` by 6 (each
    // table entry is 4-byte pointer + 2-byte width), load `off_1BFB0` table
    // base, and select tile pattern `$20C1` if the offset is zero or `$210F`
    // otherwise.
    TitleShellArtIndexLoc1BF3E {
        table_entry_size: 6,
        tile_pattern_index_zero: 0x20C1,
        tile_pattern_index_nonzero: 0x210F,
    }
}

#[cfg_attr(not(test), allow(dead_code))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct TitleShellArtDecompressLoc1BF54 {
    pub(crate) decompress_target: u32,
    pub(crate) vram_fill_iterations: u16,
    pub(crate) vram_fill_addr: u32,
    pub(crate) vram_fill_pattern: u16,
}

#[allow(non_snake_case)]
#[allow(dead_code)]
pub(crate) fn title_shell_art_decompress_loc_1BF54() -> TitleShellArtDecompressLoc1BF54 {
    // ASM sub_1BF24 -> loc_1BF54 (kid.asm:30760-30766): after tile pattern
    // selection, load mapping pointer from table, set `a1` to
    // `Decompression_Buffer`, call `EniDec`, then fill 360 words at
    // `$FFFF7F82` with pattern `$20BC`.
    TitleShellArtDecompressLoc1BF54 {
        decompress_target: 0xFFFF7F82,
        vram_fill_iterations: 0x0167 + 1,
        vram_fill_addr: 0xFFFF7F82,
        vram_fill_pattern: 0x20BC,
    }
}

#[cfg_attr(not(test), allow(dead_code))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct TitleShellVRamFillLoc1BF70 {
    pub(crate) fill_pattern: u16,
    pub(crate) fill_iterations: u16,
}

#[allow(non_snake_case)]
#[allow(dead_code)]
pub(crate) fn title_shell_vram_fill_loc_1BF70() -> TitleShellVRamFillLoc1BF70 {
    // ASM sub_1BF24 -> loc_1BF70 (kid.asm:30768-30770): DBF loop that writes
    // pattern `$20BC` to consecutive words at `$FFFF7F82` for `d0+1` iterations
    // where `d0=$167`, filling 360 words total.
    TitleShellVRamFillLoc1BF70 {
        fill_pattern: 0x20BC,
        fill_iterations: 0x0167 + 1,
    }
}

#[cfg_attr(not(test), allow(dead_code))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct TitleShellRowCopyLoopLoc1BF84 {
    pub(crate) row_count: u16,
    pub(crate) vram_row_stride_bytes: u16,
}

#[allow(non_snake_case)]
#[allow(dead_code)]
pub(crate) fn title_shell_row_copy_loop_loc_1BF84() -> TitleShellRowCopyLoopLoc1BF84 {
    // ASM sub_1BF24 -> loc_1BF84 (kid.asm:30775-30784): after filling VRAM work
    // RAM with 360 words of `$20BC` at loc_1BF70, the routine loads up to 20
    // rows of decompressed mapping data. Each pass loads `move.w (a4),d0; subq.w
    // #1,d0` to get the width from the off_1BFB0 table (`$12` for Kid frames),
    // copies that many words from `Decompression_Buffer` to VRAM work RAM via
    // loc_1BF8A, then `addi.w #$24,a0` advances the destination by 36 bytes
    // (18 words per row). The outer `dbf d1,loc_1BF84` runs `d1=$13` + 1 = 20
    // iterations. Widths vary by character: `$12` (18) for Kid, `$10` (16) for
    // Maniaxe/Juggernaut, `$11` (17) and `$E` (14) for edge frames.
    TitleShellRowCopyLoopLoc1BF84 {
        row_count: 0x0013 + 1,
        vram_row_stride_bytes: 0x0024,
    }
}

#[cfg_attr(not(test), allow(dead_code))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct TitleShellWordCopyLoopLoc1BF8A {
    pub(crate) source: u32,
    pub(crate) dest: u32,
}

#[allow(non_snake_case)]
#[allow(dead_code)]
pub(crate) fn title_shell_word_copy_loop_loc_1BF8A() -> TitleShellWordCopyLoopLoc1BF8A {
    // ASM sub_1BF24 -> loc_1BF8A (kid.asm:30780-30782): inner DBF loop that copies
    // one row of decompressed mapping words from `Decompression_Buffer` to
    // VRAM work RAM at `$FFFF7F82`. Each iteration does `move.w (a1)+,(a2)+` to
    // copy a word while advancing both pointers. The loop runs `d0+1` times
    // where `d0` was set from the table entry width minus one. This fills one
    // row before `addi.w #$24,a0` moves to the next row.
    TitleShellWordCopyLoopLoc1BF8A {
        source: 0xFFFF0000,
        dest: 0xFFFF7F82,
    }
}

#[cfg_attr(not(test), allow(dead_code))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct IntroTileUploadClearLoopLoc1C060 {
    pub(crate) vram_work_ram_addr: u32,
    pub(crate) clear_iterations: u16,
    pub(crate) fill_value: u16,
}

#[allow(non_snake_case)]
#[allow(dead_code)]
pub(crate) fn intro_tile_upload_clear_loop_loc_1C060() -> IntroTileUploadClearLoopLoc1C060 {
    // ASM sub_1C034 -> loc_1C060 (kid.asm:30862-30864): after decompressing
    // title art via `EniDec`, the routine zeros VRAM work RAM before copying
    // in the mapping data. The loop runs `move.w d1,(a0)+` for `d0+1` iterations
    // where `d0=$18F` (399), filling 400 words with `d1=0` (set by
    // `moveq #0,d1`). The destination `$FFFF7F82` is the same scratch area used
    // by sub_1BF24 for title-shell art uploads but with a different fill value
    // and count than loc_1BF70's `$20BC`/360-word prefill.
    IntroTileUploadClearLoopLoc1C060 {
        vram_work_ram_addr: 0xFFFF7F82,
        clear_iterations: 0x018F + 1,
        fill_value: 0x0000,
    }
}

#[cfg_attr(not(test), allow(dead_code))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct IntroTileUploadRowCopyLoopLoc1C07C {
    pub(crate) vram_row_stride_bytes: u16,
}

#[allow(non_snake_case)]
#[allow(dead_code)]
pub(crate) fn intro_tile_upload_row_copy_loop_loc_1C07C() -> IntroTileUploadRowCopyLoopLoc1C07C {
    // ASM sub_1C034 -> loc_1C07C (kid.asm:30871-30880): after zeros clearing at
    // loc_1C060, the routine copies decompressed mapping rows into VRAM work RAM.
    // Each pass loads `move.w (a4),d0; subq.w #1,d0` to get the width from the
    // off_1C126 table (e.g., `$C` = 12for first entry), copies that many words via
    // loc_1C082, then `addi.w #$20,a0` advances the destination by 32 bytes
    // (16 words per row). The outer `dbf d1,loc_1C07C` runs `height` passes
    // where `d1` was loaded from `2(a4)` (table entry byte 3-4, e.g., `$18` = 24).
    // Video DMA follows at loc_1C09E/loc_1C0B8/loc_1C0D0 pattern groups.
    IntroTileUploadRowCopyLoopLoc1C07C {
        vram_row_stride_bytes: 0x0020,
    }
}

#[cfg_attr(not(test), allow(dead_code))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct IntroTileUploadWordCopyLoopLoc1C082 {
    pub(crate) source: u32,
    pub(crate) dest: u32,
}

#[allow(non_snake_case)]
#[allow(dead_code)]
pub(crate) fn intro_tile_upload_word_copy_loop_loc_1C082() -> IntroTileUploadWordCopyLoopLoc1C082 {
    // ASM sub_1C034 -> loc_1C082 (kid.asm:30876-30878): inner DBF loop that copies
    // one row of decompressed mapping words from `Decompression_Buffer` to
    // VRAM work RAM at `$FFFF7F82 + offset`. Each iteration does
    // `move.w (a1)+,(a2)+` to copy a word while advancing both pointers. The loop
    // runs `d0+1` times where `d0` was set from the table entry width minus one.
    // After the row completes, `addi.w #$20,a0` moves to the next 32-byte row.
    IntroTileUploadWordCopyLoopLoc1C082 {
        source: 0xFFFF0000,
        dest: 0xFFFF7F82,
    }
}

#[cfg_attr(not(test), allow(dead_code))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct IntroTileUploadVramDmaLoopLoc1C0A4 {
    pub(crate) words_per_row: u16,
    pub(crate) vram_address_step: u32,
    pub(crate) row_count: u8,
}

#[allow(non_snake_case)]
#[allow(dead_code)]
pub(crate) fn intro_tile_upload_vram_dma_loop_loc_1C0A4() -> IntroTileUploadVramDmaLoopLoc1C0A4 {
    // ASM sub_1C034 -> loc_1C0A4 (kid.asm:30889-30891): inner DBF loop that streams
    // one row of tile pattern data from VRAM work RAM `$FFFF7F82` to the VDP data
    // port via `move.w (a0)+,(a6)`. Each iteration writes one word, and the loop
    // runs `d6+1` = 15 times per row (set via `moveq #$E,d6` at loc_1C09E).
    // After the inner loop, `addq.w #2,a0` skips 2 bytes of padding,
    // `addi.l #$800000,d0` advances the VRAM write address by $800 = 2048 bytes,
    // and the outer `dbf d7,loc_1C09E` runs 8 iterations (d7 initialized to 7).
    // VDP command port write at `4(a6)` sets destination address before each row.
    // Initial `d0` = `$41940000` encodes VRAM write to address $40000-1 = $A340*2.
    IntroTileUploadVramDmaLoopLoc1C0A4 {
        words_per_row: 0x0E + 1,
        vram_address_step: 0x0080_0000,
        row_count: 0x07 + 1,
    }
}

#[cfg_attr(not(test), allow(dead_code))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct IntroTileUploadVramDmaGroup2Loc1C0BE {
    pub(crate) words_per_row: u16,
    pub(crate) vram_address_step: u32,
    pub(crate) row_count: u8,
}

#[allow(non_snake_case)]
#[allow(dead_code)]
pub(crate) fn intro_tile_upload_vram_dma_group_2_loc_1C0BE() -> IntroTileUploadVramDmaGroup2Loc1C0BE
{
    // ASM sub_1C034 -> loc_1C0BE (kid.asm:30901-30905): second VRAM DMA group
    // after the 8-row initial upload at loc_1C09E. The outer loop sets d7=1
    // via `moveq #1,d7` before loc_1C0B8, so this group runs 2 iterations.
    // Each iteration writes 16 words (d6=$F via `moveq #$F,d6`) to the VDP
    // data port via `move.w (a0)+,(a6)`. Between rows, `addi.l #$800000,d0`
    // advances the VRAM write address by $800 = 2048 bytes. The VDP command
    // register write at `4(a6)` before each row sets destination from d0.
    IntroTileUploadVramDmaGroup2Loc1C0BE {
        words_per_row: 0x0F + 1,
        vram_address_step: 0x0080_0000,
        row_count: 0x01 + 1,
    }
}

#[cfg_attr(not(test), allow(dead_code))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct IntroTileUploadVramDmaGroup3Loc1C0D6 {
    pub(crate) words_per_row: u16,
    pub(crate) vram_address_step: u32,
    pub(crate) row_count: u8,
    pub(crate) stride_padding_bytes: u16,
}

#[allow(non_snake_case)]
#[allow(dead_code)]
pub(crate) fn intro_tile_upload_vram_dma_group_3_loc_1C0D6() -> IntroTileUploadVramDmaGroup3Loc1C0D6
{
    // ASM sub_1C034 -> loc_1C0D6 (kid.asm:30912-30917): third VRAM DMA group
    // after the 2-row group at loc_1C0B8. The outer loop sets d7=14 via
    // `moveq #$E,d7` before loc_1C0D0, so this group runs 15 iterations.
    // Each iteration writes 15 words (d6=$E via `moveq #$E,d6`) to the VDP
    // data port via `move.w (a0)+,(a6)`. Unlike groups 1 and 2, this group
    // executes `addq.w #2,a0` after each row to skip 2 bytes of padding.
    // `addi.l #$800000,d0` still advances VRAM address by $800 per row.
    IntroTileUploadVramDmaGroup3Loc1C0D6 {
        words_per_row: 0x0E + 1,
        vram_address_step: 0x0080_0000,
        row_count: 0x0E + 1,
        stride_padding_bytes: 0x0002,
    }
}

#[cfg_attr(not(test), allow(dead_code))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct IntroDecompressionBufferZeroFillLoc1C11E {
    pub(crate) word_count: u16,
    pub(crate) fill_value: u16,
}

#[allow(non_snake_case)]
#[allow(dead_code)]
pub(crate) fn intro_decompression_buffer_zero_fill_loc_1C11E()
-> IntroDecompressionBufferZeroFillLoc1C11E {
    // ASM sub_1C112 -> loc_1C11E (kid.asm:30948-30950): zero-fills the
    // Decompression_Buffer area before the general VDP upload at loc_1C104.
    // The routine sets `d0=$59` (90-1 for loop count) via `move.w #$59,d0` and
    // `d1=0` via `moveq #0,d1` before entering the loop. Each iteration does
    // `move.w d1,(a0)+` to write a zero word and advance the pointer. The loop
    // runs `d0+1` = 90 iterations, zeroing 180 bytes. Target address is
    // Decompression_Buffer at `$FFFF0000`.
    IntroDecompressionBufferZeroFillLoc1C11E {
        word_count: 0x0059 + 1,
        fill_value: 0x0000,
    }
}

#[cfg_attr(not(test), allow(dead_code))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct IntroSequence4PaletteMaskStartLoc1C1E2 {
    pub(crate) start_byte_offset: usize,
    pub(crate) mask: u8,
}

#[allow(non_snake_case)]
#[allow(dead_code)]
pub(crate) fn intro_sequence_4_palette_mask_start_loc_1C1E2(
    d3_before_lsr: u16,
) -> IntroSequence4PaletteMaskStartLoc1C1E2 {
    // ASM sub_1C1D2 -> loc_1C1E2 (kid.asm:31045-31051): `lsr.w #1,d3`
    // halves the byte offset and leaves the old bit 0 in carry. Carry set
    // keeps `d4=$F0`; carry clear falls through to `move.w #$F,d4`. The label
    // then adds the shifted `d3` to the Level_Layout base pointer.
    IntroSequence4PaletteMaskStartLoc1C1E2 {
        start_byte_offset: usize::from(d3_before_lsr >> 1),
        mask: if d3_before_lsr & 1 == 0 { 0x0F } else { 0xF0 },
    }
}

pub(crate) const INTRO_SEQUENCE_4_PALETTE_MASK_ROWS_LOC_1C1E4: usize = 0x57 + 1;
pub(crate) const INTRO_SEQUENCE_4_PALETTE_MASK_STRIDE_BYTES_LOC_1C1E4: usize = 0x20;

#[allow(non_snake_case)]
#[allow(dead_code)]
pub(crate) fn apply_intro_sequence_4_palette_mask_loop_loc_1C1E4(
    level_layout: &mut [u8],
    start: IntroSequence4PaletteMaskStartLoc1C1E2,
) -> usize {
    // ASM loc_1C1E4 (kid.asm:31053-31056): apply `and.b d4,(a0)`, advance
    // `a0` by `$20` bytes, and let `dbf d2,loc_1C1E4` run inclusively from
    // `d2=$57`, touching 88 Level_Layout bytes in one column of palette nibbles.
    let mut offset = start.start_byte_offset;
    let mut masked_bytes = 0_usize;
    for _ in 0..INTRO_SEQUENCE_4_PALETTE_MASK_ROWS_LOC_1C1E4 {
        if let Some(byte) = level_layout.get_mut(offset) {
            *byte &= start.mask;
            masked_bytes += 1;
        }
        offset = offset.saturating_add(INTRO_SEQUENCE_4_PALETTE_MASK_STRIDE_BYTES_LOC_1C1E4);
    }
    masked_bytes
}

#[cfg_attr(not(test), allow(dead_code))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct VdpPatternFillRowLoopLoc1C5E8 {
    pub(crate) vram_address_step: u32,
}

#[allow(non_snake_case)]
#[allow(dead_code)]
pub(crate) fn vdp_pattern_fill_row_loop_loc_1C5E8() -> VdpPatternFillRowLoopLoc1C5E8 {
    // ASM sub_1C5D0 -> loc_1C5E8 (kid.asm:31353-31356): outer loop of the VDP
    // pattern fill routine called from multiple intro/end screen initialization
    // functions. Each iteration:
    // 1. Copies the current row counter from d6 to d1 (inner loop counter)
    // 2. Writes d0.long to VDP control port 4(a6) setting VRAM write address
    // 3. Advances d0 by $800000 (2048 bytes = VSRAM row stride)
    // Called at lines 29295, 29404, 30197, 30438, 30790, 30936 with varying
    // dimensions. Initial d0 = ((y << 7) + (x << 1)) | $4000 << 16.
    VdpPatternFillRowLoopLoc1C5E8 {
        vram_address_step: 0x0080_0000,
    }
}

#[cfg_attr(not(test), allow(dead_code))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct VdpPatternFillWordLoopLoc1C5F4 {
    pub(crate) source_register: &'static str,
    pub(crate) dest_register: &'static str,
}

#[allow(non_snake_case)]
#[allow(dead_code)]
pub(crate) fn vdp_pattern_fill_word_loop_loc_1C5F4() -> VdpPatternFillWordLoopLoc1C5F4 {
    // ASM sub_1C5D0 -> loc_1C5F4 (kid.asm:31358-31360): inner DBF loop that copies
    // one row of pattern data from source buffer to VDP data port. Each iteration
    // does `move.w (a4)+,(a6)` copying a word while advancing the source pointer.
    // The loop runs d1+1 times where d1 was set from d6 (row height in words).
    // After filling one row, control returns to loc_1C5E8 which advances to the
    // next VRAM address row.
    VdpPatternFillWordLoopLoc1C5F4 {
        source_register: "a4",
        dest_register: "a6",
    }
}

#[cfg_attr(not(test), allow(dead_code))]
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct TitleTextKidSetupLoc1B5E4 {
    pub(crate) palette_line: u8,
    pub(crate) x_pos: u16,
    pub(crate) vram_tile: u16,
    pub(crate) sprite_mapping_label: &'static str,
    pub(crate) display_enabled: bool,
    pub(crate) pre_drift_hibernate_frames: u16,
    pub(crate) drift_frame_count: u16,
    pub(crate) moves_during_drift: bool,
    pub(crate) post_drift_hibernate_frames: u16,
    pub(crate) interstitial_hibernate_frames: u16,
    pub(crate) popup_positions: [(u16, u16); 2],
    pub(crate) loops_forever: bool,
}

#[allow(non_snake_case)]
#[allow(dead_code)]
pub(crate) fn title_text_kid_setup_loc_1B5E4() -> TitleTextKidSetupLoc1B5E4 {
    // ASM Obj_TitleTextKid / loc_1B5E4 (kid.asm:29847-29866): both title-text
    // entry points share the same visible shell here, setting palette line 2,
    // x=`$2E`, tile `$4D3`, mapping `LnkTo_unk_E10FE`, display flag `$13(a3)`,
    // a `$1E`-frame wait, a 43-frame `is_moved` drift, a `$14`-frame pause,
    // then two `sub_1B652` popups around the fixed poses `($2E,$8C)` and
    // `($5A,$98)` before the worker hibernates forever.
    TitleTextKidSetupLoc1B5E4 {
        palette_line: 2,
        x_pos: 0x002E,
        vram_tile: 0x04D3,
        sprite_mapping_label: "LnkTo_unk_E10FE-Data_Index",
        display_enabled: true,
        pre_drift_hibernate_frames: 0x001E,
        drift_frame_count: 0x002A + 1,
        moves_during_drift: true,
        post_drift_hibernate_frames: 0x0014,
        interstitial_hibernate_frames: 0x000A,
        popup_positions: [(0x002E, 0x008C), (0x005A, 0x0098)],
        loops_forever: true,
    }
}

#[cfg_attr(not(test), allow(dead_code))]
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct TitleTextChameleonIdleStepLoc1B6A2 {
    pub(crate) hibernate_frames: u16,
    pub(crate) next_d2: i16,
    pub(crate) next_d3: u16,
    pub(crate) upload: Option<TitleTextChameleonFrameUploadSub1B71A>,
    pub(crate) loops_forever: bool,
}

#[allow(non_snake_case)]
#[allow(dead_code)]
pub(crate) fn title_text_chameleon_idle_step_loc_1B6A2(
    decoded_words: &[u16],
    d2: i16,
    d3: u16,
) -> TitleTextChameleonIdleStepLoc1B6A2 {
    // ASM Obj_TitleTextChameleon / loc_1B6A2 (kid.asm:29911-29914): hibernate
    // exactly one frame, call `sub_1B71A`, preserve `d2`, advance `d3` by one
    // through that helper, then branch straight back to `loc_1B6A2` forever.
    let next_d3 = d3.wrapping_add(1);
    TitleTextChameleonIdleStepLoc1B6A2 {
        hibernate_frames: 1,
        next_d2: d2,
        next_d3,
        upload: title_text_chameleon_frame_upload_sub_1B71A(decoded_words, d2, next_d3),
        loops_forever: true,
    }
}

#[cfg_attr(not(test), allow(dead_code))]
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct TitleTextChameleonRevealIterationLoc1B6CE {
    pub(crate) incoming_d2: i16,
    pub(crate) next_d2: i16,
    pub(crate) next_d3: u16,
    pub(crate) upload: Option<TitleTextChameleonFrameUploadSub1B71A>,
}

#[cfg_attr(not(test), allow(dead_code))]
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct TitleTextChameleonRevealLoopLoc1B6CE {
    pub(crate) hibernate_frames_per_iteration: u16,
    pub(crate) iterations: Vec<TitleTextChameleonRevealIterationLoc1B6CE>,
    pub(crate) final_d0: u16,
}

#[allow(non_snake_case)]
#[allow(dead_code)]
pub(crate) fn title_text_chameleon_reveal_loop_loc_1B6CE(
    decoded_words: &[u16],
) -> TitleTextChameleonRevealLoopLoc1B6CE {
    // ASM loc_1B6CE (kid.asm:29927-29931): starting from `d2=#$28,d3=0`, each
    // pass hibernates one frame, calls `sub_1B71A`, decrements `d2`, and
    // loops until that post-decrement value reaches zero, after which the
    // caller seeds `d0=0` for the later Sega text cadence at `loc_1B6DC`.
    let mut d2 = 0x0028_i16;
    let mut d3 = 0_u16;
    let mut iterations = Vec::with_capacity(0x0028);
    loop {
        d3 = d3.wrapping_add(1);
        let upload = title_text_chameleon_frame_upload_sub_1B71A(decoded_words, d2, d3);
        let next_d2 = d2.wrapping_sub(1);
        iterations.push(TitleTextChameleonRevealIterationLoc1B6CE {
            incoming_d2: d2,
            next_d2,
            next_d3: d3,
            upload,
        });
        d2 = next_d2;
        if d2 == 0 {
            break;
        }
    }

    TitleTextChameleonRevealLoopLoc1B6CE {
        hibernate_frames_per_iteration: 1,
        iterations,
        final_d0: 0,
    }
}

#[cfg_attr(not(test), allow(dead_code))]
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct TitleTextChameleonSegaStepLoc1B6DC {
    pub(crate) hibernate_frames: u16,
    pub(crate) next_d0: u16,
    pub(crate) next_d3: u16,
    pub(crate) upload: Option<TitleTextChameleonFrameUploadSub1B71A>,
    pub(crate) spawned_handler: Option<&'static str>,
    pub(crate) spawned_art_label: Option<&'static str>,
    pub(crate) loops_forever: bool,
}

#[allow(non_snake_case)]
#[allow(dead_code)]
pub(crate) fn title_text_chameleon_sega_step_loc_1B6DC(
    decoded_words: &[u16],
    d0: u16,
    d3: u16,
) -> TitleTextChameleonSegaStepLoc1B6DC {
    // ASM loc_1B6DC (kid.asm:29934-29948): hibernate one frame, save `d0`
    // across `sub_1B71A`, restore it, then increment the preserved counter.
    // Only the transition to `$1E` allocates the fixed `loc_1C7A0` Sega text
    // worker; later passes branch back to `loc_1B6DC` without spawning again.
    let next_d3 = d3.wrapping_add(1);
    let upload = title_text_chameleon_frame_upload_sub_1B71A(decoded_words, 0, next_d3);
    let next_d0 = d0.wrapping_add(1);
    let (spawned_handler, spawned_art_label) = if next_d0 == 0x001E {
        (Some("loc_1C7A0"), Some("SegaText"))
    } else {
        (None, None)
    };

    TitleTextChameleonSegaStepLoc1B6DC {
        hibernate_frames: 1,
        next_d0,
        next_d3,
        upload,
        spawned_handler,
        spawned_art_label,
        loops_forever: true,
    }
}

#[cfg_attr(not(test), allow(dead_code))]
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct IntroSequence6StripedTileFillLoc1B194 {
    pub(crate) vram_word_addr: u16,
    pub(crate) word_writes: Vec<u16>,
}

#[allow(non_snake_case)]
#[allow(dead_code)]
pub(crate) fn intro_sequence_6_striped_tile_fill_loc_1B194() -> IntroSequence6StripedTileFillLoc1B194
{
    // ASM Load_IntroSequence6 -> loc_1B194 (kid.asm:29383-29392): after
    // `move.l #vdpComm($E000,VRAM,WRITE),4(a6)` and `moveq #$D,d0`, each outer
    // pass runs `loc_1B196` with `d1=#$1F` to emit `$00BD,$00BE` 32 times,
    // reloads `d1=#$1F`, then runs `loc_1B1A4` to emit `$00BF,$00C0` 32 times
    // before the inclusive `dbf d0,loc_1B194` repeats the whole 64-pair band
    // 14 times.
    const OUTER_PASSES: usize = 0x000D + 1;
    const PAIRS_PER_HALF: usize = 0x001F + 1;
    let mut word_writes = Vec::with_capacity(OUTER_PASSES * PAIRS_PER_HALF * 4);
    for _ in 0..OUTER_PASSES {
        for _ in 0..PAIRS_PER_HALF {
            word_writes.push(0x00BD);
            word_writes.push(0x00BE);
        }
        for _ in 0..PAIRS_PER_HALF {
            word_writes.push(0x00BF);
            word_writes.push(0x00C0);
        }
    }

    IntroSequence6StripedTileFillLoc1B194 {
        vram_word_addr: 0xE000,
        word_writes,
    }
}

#[allow(non_snake_case)]
#[allow(dead_code)]
pub(crate) fn title_text_chameleon_frame_upload_sub_1B71A(
    decoded_words: &[u16],
    d2: i16,
    d3: u16,
) -> Option<TitleTextChameleonFrameUploadSub1B71A> {
    // ASM sub_1B71A / loc_1B734 / loc_1B758 / loc_1B76C (kid.asm:29957-30011):
    // pick one of four Enigma tilemaps from `off_1B70A` via `d3 & $C`,
    // clamp negative reveal indices back to zero, build the base VRAM word
    // address from row `$12` plus the clipped horizontal offset, cap the first
    // row to 21 words, and optionally emit a `$00BC` filler word one cell to
    // the left when the reveal still starts in blank space.
    if d2 >= 0x22 {
        return None;
    }

    const ROW_COUNT: usize = 7;
    const SOURCE_ROW_WORDS: usize = 29;
    const TOP_ROW_MAX_WORDS: usize = 21;
    let required_words = ROW_COUNT * SOURCE_ROW_WORDS;
    if decoded_words.len() < required_words {
        return None;
    }

    let tilemap_label = match d3 & 0x000C {
        0x0000 => "unk_2E706",
        0x0004 => "unk_2E728",
        0x0008 => "unk_2E77A",
        _ => "unk_2E728",
    };

    // ASM loc_1B734: `tst.w d2; bpl.s loc_1B734; moveq #0,d2`.
    let reveal = d2.max(0);
    let mut horizontal_offset = 5_i16 - reveal;
    let mut source_start_word = 0_usize;
    let mut row_word_count = SOURCE_ROW_WORDS;
    if horizontal_offset < 0 {
        let clipped_words = usize::try_from(-horizontal_offset).ok()?;
        source_start_word = clipped_words;
        row_word_count = row_word_count.saturating_sub(clipped_words);
        horizontal_offset = 0;
    }
    if row_word_count == 0 {
        return Some(TitleTextChameleonFrameUploadSub1B71A {
            tilemap_label,
            rows: Vec::new(),
        });
    }

    let horizontal_offset_words = u16::try_from(horizontal_offset).ok()?;
    let mut rows = Vec::with_capacity(ROW_COUNT);
    for row_index in 0..ROW_COUNT {
        let row_start = row_index * SOURCE_ROW_WORDS + source_start_word;
        // ASM loc_1B76C: while `d7==6` (the first row), non-negative `d4`
        // plus `d1 >= $14` force `d1=$14`, so the inclusive DBF copy emits 21
        // words on the top row even when later rows keep the full width.
        let top_row_limit = if row_index == 0 {
            row_word_count.min(TOP_ROW_MAX_WORDS)
        } else {
            row_word_count
        };
        // ASM loc_1B7A6: `move.w (a0)+,(a6)` runs before `dbf d1,loc_1B7A6`,
        // so each row copies `d1 + 1` words and still emits one word when the
        // clipped counter has already reached zero.
        let words = decoded_words[row_start..row_start + top_row_limit].to_vec();
        // ASM loc_1B758: `move.w d5,d0; lsl.w #7,d0; add.w d4,d0; add.w d4,d0`
        // builds the row-`$12` VRAM word address, then `addi.w #$4000,d0;
        // swap d0; clr.w d0` turns that into the VDP command whose high word
        // exposes the `$04xx` word address we store here.
        let row_base = 0x0480_u16
            .checked_add(horizontal_offset_words.saturating_mul(2))?
            .checked_add(u16::try_from(row_index).ok()?.saturating_mul(64))?;
        // ASM loc_1B780 / loc_1B79A: positive `d4` first writes filler tile
        // `$00BC` one cell to the left, while zero `d4` falls through to the
        // plain `move.l d0,4(a6)` path with no filler write at all.
        rows.push(TitleTextChameleonRowWriteSub1B71A {
            vram_word_address: row_base,
            filler_word: (horizontal_offset_words != 0).then_some(0x00BC),
            words,
        });
    }

    Some(TitleTextChameleonFrameUploadSub1B71A {
        tilemap_label,
        rows,
    })
}

#[allow(non_snake_case)]
#[allow(dead_code)]
pub(crate) fn title_text_word_writes_sub_1B7B6() -> Vec<TitleTextWordWritesSub1B7B6> {
    // ASM sub_1B7B6 (kid.asm:30035): iterate `unk_1B7E0` until the `$FFFF`
    // sentinel and stamp the fixed `$21AB..$21AE` 2x2 word pattern at each
    // listed title-screen VRAM word address.
    const BASE_WORD_ADDRESSES: [u16; 55] = [
        0x7C12, 0x7C1A, 0x7C22, 0x7C2A, 0x7C32, 0x7CB2, 0x7CB6, 0x7CBA, 0x7CBE, 0x7CC2, 0x7CC6,
        0x7CCA, 0x7CCE, 0x7CD2, 0x7D52, 0x7D56, 0x7D5A, 0x7D5E, 0x7D62, 0x7D66, 0x7D6A, 0x7D6E,
        0x7D72, 0x7DF2, 0x7DF6, 0x7DFA, 0x7DFE, 0x7E02, 0x7E06, 0x7E0A, 0x7E0E, 0x7E12, 0x7E92,
        0x7E96, 0x7E9A, 0x7E9E, 0x7EA2, 0x7EA6, 0x7EAA, 0x7EAE, 0x7EB2, 0x7F36, 0x7F3A, 0x7F3E,
        0x7F42, 0x7F46, 0x7F4A, 0x7F4E, 0x7FD6, 0x7FDA, 0x7FDE, 0x7FE2, 0x7FE6, 0x7FEA, 0x7FEE,
    ];

    BASE_WORD_ADDRESSES
        .into_iter()
        // ASM loc_1B7BA reloads the next table word, exits on `$FFFF`, and
        // otherwise branches back to itself after stamping the same 2x2 tile
        // block at the new base address.
        .map(|base_word_address| TitleTextWordWritesSub1B7B6 {
            base_word_address,
            top_words: [0x21AB, 0x21AC],
            bottom_words: [0x21AD, 0x21AE],
        })
        .collect()
}

#[allow(non_snake_case)]
pub(crate) fn step_option_menu_panels_sub_1CC88(
    toggle_byte: u8,
    panel_delta: i16,
    left_panel_x: i16,
    right_panel_x: i16,
) -> (u8, i16, i16) {
    // ASM sub_1CC88 (kid.asm:31808): `not.b ($FFFFFB5C).w`, then if the byte
    // is non-zero add/subtract `($FFFFFB5E).w` across the two option-menu
    // workers; otherwise undo that same offset on the following frame.
    let toggle_byte = !toggle_byte;
    // ASM loc_1CCAA (kid.asm:31822): shared epilogue after either panel branch,
    // restoring `d7` from the stack and returning with the updated toggle byte
    // plus both worker `$24` words.
    if toggle_byte != 0 {
        (
            toggle_byte,
            left_panel_x.wrapping_add(panel_delta),
            right_panel_x.wrapping_sub(panel_delta),
        )
    } else {
        // ASM loc_1CCA2 (kid.asm:31818): zero-byte branch after `beq.w
        // loc_1CCA2`, reversing the previous frame's worker offsets with
        // `sub.w d7,$24(a0)` and `add.w d7,$24(a1)`.
        (
            toggle_byte,
            left_panel_x.wrapping_sub(panel_delta),
            right_panel_x.wrapping_add(panel_delta),
        )
    }
}

#[allow(non_snake_case)]
#[allow(dead_code)]
pub(crate) fn end_sequence_display_refresh_sub_1D830(
    camera_x_pos: i16,
    camera_y_pos: i16,
    toggle_byte: u8,
    panel_delta: i16,
    left_panel_x: i16,
    right_panel_x: i16,
) -> EndSequenceDisplayRefreshSub1D830 {
    // ASM sub_1D830 (kid.asm:33070): call sub_1DA24 to upload one display
    // refresh from the live camera, then advance the options-panel worker via
    // sub_1CC88. sub_1DA24 writes Camera_Y_pos into VSRAM, zeroes the second
    // VSRAM slot, and repeats `-Camera_X_pos` across `$DF+1` horizontal-scroll
    // word pairs before returning.
    let (panel_toggle_byte, left_panel_x, right_panel_x) =
        step_option_menu_panels_sub_1CC88(toggle_byte, panel_delta, left_panel_x, right_panel_x);
    EndSequenceDisplayRefreshSub1D830 {
        camera_y_scroll: camera_y_pos,
        h_scroll_value: camera_x_pos.wrapping_neg(),
        h_scroll_word_pairs: 0x00E0,
        panel_toggle_byte,
        left_panel_x,
        right_panel_x,
    }
}

#[allow(non_snake_case)]
#[allow(dead_code)]
pub(crate) fn end_sequence_h_scroll_upload_loc_1DA50(
    camera_x_pos: i16,
) -> EndSequenceHorizontalScrollUploadLoc1DA50 {
    // ASM loc_1DA50 (kid.asm:33323): after `move.l #vdpComm($1400,VRAM,WRITE),4(a6)`,
    // `move.w (Camera_X_pos).w,d7`, `neg.w d7`, and `move.w #$DF,d6`, the loop
    // writes `d7` twice before `dbf d6,loc_1DA50`, so it fills 448 horizontal-
    // scroll words at VRAM word address `$1400` with `-Camera_X_pos`.
    EndSequenceHorizontalScrollUploadLoc1DA50 {
        vram_word_addr: 0x1400,
        repeated_word: camera_x_pos.wrapping_neg(),
        word_write_count: (0x00DFu16 + 1) * 2,
    }
}

#[allow(non_snake_case)]
#[allow(dead_code)]
pub(crate) fn end_sequence_tile_fill_loc_1DA6A() -> EndSequenceTileFillLoc1DA6A {
    // ASM loc_1DA6A (kid.asm:33340): `sub_1DA62` seeds `d0=#$7FF` and
    // `d1=#$94`, then `move.w d1,(a6)` executes before `dbf d0,loc_1DA6A`, so
    // the shared fill loop emits 2048 copies of tile word `$0094`.
    EndSequenceTileFillLoc1DA6A {
        fill_word: 0x0094,
        word_write_count: 0x07FF + 1,
    }
}

#[allow(non_snake_case)]
#[allow(dead_code)]
pub(crate) fn end_sequence_vram_zero_fill_loc_1DA7E() -> EndSequenceVramZeroFillLoc1DA7E {
    // ASM loc_1DA7E (kid.asm:33355): `sub_1DA72` first programs
    // `vdpComm($1280,VRAM,WRITE)`, then seeds `d0=#$F` and `d1=0`, so the
    // inclusive `move.w d1,(a6); dbf d0,loc_1DA7E` loop clears 16 words.
    EndSequenceVramZeroFillLoc1DA7E {
        vram_word_addr: 0x1280,
        fill_word: 0x0000,
        word_write_count: 0x000F + 1,
    }
}

#[allow(non_snake_case)]
#[allow(dead_code)]
pub(crate) fn end_sequence_fade_loop_loc_1D270() -> EndSequenceFadeLoopLoc1D270 {
    // ASM loc_1D270 (kid.asm:32637): after `moveq #7,d0`, each loop body loads
    // `d7 = #$20`, darkens `Palette_Buffer+$20` via `sub_1C85E`, hibernates for
    // six frames, then `dbf d0,loc_1D270` repeats for `$7 + 1 = 8` fades before
    // retargeting the VDP command port to `vdpComm($E000,VRAM,WRITE)` and
    // continuing with `sub_1DA62`.
    EndSequenceFadeLoopLoc1D270 {
        loop_seed: 0x0007,
        fade_palette_byte_offset: 0x0020,
        fade_iterations: 0x0008,
        hibernate_frames: 0x0006,
        post_loop_vram_word_addr: 0xE000,
        post_loop_subroutine: "sub_1DA62",
    }
}

#[allow(non_snake_case)]
#[allow(dead_code)]
pub(crate) fn end_sequence_panel_setup_loc_1D306() -> EndSequencePanelSetupLoc1D306 {
    // ASM loc_1D306 (kid.asm:32671): `moveq #8,d0` feeds `dbf d0,loc_1D306`,
    // so nine palette words copy from `Pal_1C92A` into `Palette_Buffer+$E`.
    // The label then spawns the two menu-panel workers, seeds their positions,
    // clears `Camera_Y_pos`, primes `sub_1CDA8` with row `$1B` / column `$1F`,
    // and initialises the later `loc_1D3A0` scroll state with `d2 = 8`,
    // `d3 = $E0`, toggle byte 0, and panel delta `$118`.
    EndSequencePanelSetupLoc1D306 {
        palette_words_copied: 0x0009,
        first_panel_x: 0x0030,
        first_panel_y: 0x0048,
        second_panel_x: 0x0110,
        second_panel_y: 0x0048,
        stripe_row_seed: 0x001B,
        stripe_column_seed: 0x001F,
        scroll_counter_seed: 0x0008,
        scroll_distance_seed: 0x00E0,
        panel_toggle_byte: 0x00,
        panel_delta: 0x0118,
        tilemap_row_upload_subroutine: "sub_1CDA8",
    }
}

#[allow(non_snake_case)]
#[allow(dead_code)]
pub(crate) fn end_sequence_panel_scroll_loop_loc_1D3A0() -> EndSequencePanelScrollLoopLoc1D3A0 {
    // ASM loc_1D3A0 (kid.asm:32706): every active pass runs `sub_1DA24`,
    // hibernates one frame, subtracts 2 from both `Camera_Y_pos` and `d3`, and
    // decrements `d2` by 2 before `sub_1CC88`.
    // ASM loc_1D3C4 (kid.asm:32720): `bsr.w sub_1CC88; bra.s loc_1D3A0` is the
    // steady-state back-edge taken after both the plain passes and the rollover
    // path that refreshes one more tilemap row through `sub_1CDA8`. Because the
    // top-of-loop `tst.w d3` exits only on the following pass, the `$E0` seed
    // yields `$E0 / 2 = $70` active iterations and `$70 / 4 = $1C` row uploads.
    EndSequencePanelScrollLoopLoc1D3A0 {
        display_refresh_subroutine: "sub_1DA24",
        hibernate_subroutine: "j_Hibernate_Object_1Frame",
        panel_step_subroutine: "sub_1CC88",
        row_upload_subroutine: "sub_1CDA8",
        scroll_counter_seed: 0x0008,
        scroll_distance_seed: 0x00E0,
        scroll_step: 0x0002,
        active_iterations: 0x0070,
        panel_refresh_iterations: 0x001C,
    }
}

#[allow(non_snake_case)]
#[allow(dead_code)]
pub(crate) fn end_sequence_hide_glyphs_loc_1D7F0() -> EndSequenceHideGlyphsLoc1D7F0 {
    // ASM loc_1D7F0 (kid.asm:33034-33037): seed `a4` from `$FFFFFB72`, load
    // each cached glyph pointer with `move.l (a4)+,a3`, clear the display flag
    // at `$13(a3)` via `sf`, and let `dbf d7,loc_1D7F0` consume the `d7 = 5`
    // seed from `sub_1D7E6` for six inclusive passes.
    EndSequenceHideGlyphsLoc1D7F0 {
        glyph_pointer_table_base: 0xFB72,
        loop_seed: 0x0005,
        hidden_glyphs: 0x0006,
        cleared_flag_offset: 0x0013,
    }
}

#[allow(non_snake_case)]
#[allow(dead_code)]
pub(crate) fn end_sequence_page_fade_outer_loop_loc_1D80C() -> EndSequencePageFadeOuterLoopLoc1D80C
{
    // ASM loc_1D80C (kid.asm:33051-33060): after `sub_1D800` saves registers
    // and seeds `d0 = 7`, each outer pass reloads `d7 = $60`, calls
    // `sub_1C85E`, reloads `d1 = 3`, and then enters `loc_1D814`. The closing
    // `dbf d0,loc_1D80C` therefore repeats the fade setup eight inclusive
    // times.
    EndSequencePageFadeOuterLoopLoc1D80C {
        outer_loop_seed: 0x0007,
        outer_loop_iterations: 0x0008,
        fade_palette_byte_offset: 0x0060,
        fade_setup_subroutine: "sub_1C85E",
        inner_loop_seed: 0x0003,
        inner_loop_iterations: 0x0004,
        inner_loop_entry_label: "loc_1D814",
    }
}

#[allow(non_snake_case)]
#[allow(dead_code)]
pub(crate) fn end_sequence_page_fade_inner_loop_loc_1D814() -> EndSequencePageFadeInnerLoopLoc1D814
{
    // ASM loc_1D814 (kid.asm:33056-33060): each inner pass refreshes the end
    // sequence through `sub_1D830`, hibernates one frame, and lets
    // `dbf d1,loc_1D814` consume the `d1 = 3` seed from `loc_1D80C` for four
    // inclusive refresh/wait pairs before control falls through to
    // `dbf d0,loc_1D80C`.
    EndSequencePageFadeInnerLoopLoc1D814 {
        loop_seed: 0x0003,
        iterations: 0x0004,
        refresh_subroutine: "sub_1D830",
        hibernate_subroutine: "j_Hibernate_Object_1Frame",
        next_outer_label: "loc_1D80C",
    }
}

#[allow(non_snake_case)]
#[allow(dead_code)]
pub(crate) fn end_sequence_credits_glyph_cache_loc_1D3D0() -> EndSequenceCreditsGlyphCacheLoc1D3D0 {
    // ASM loc_1D3CA / loc_1D3D0 (kid.asm:32726-32739): seed `d0 = 5`, point
    // `a4` at `$FFFFFB72`, then iterate `dbf d0,loc_1D3D0` while `sub_1D786`
    // allocates six glyph workers. `lsl.w #2,d7` turns the descending counter
    // into 4-byte pointer slots written in reverse order: `$14,$10,$C,$8,$4,$0`.
    // After the cache is full, the routine seeds `(Addr_PlatformLayout).w` from
    // `unk_1D75A` and starts the text loop at `EndText1`.
    EndSequenceCreditsGlyphCacheLoc1D3D0 {
        glyph_count: 0x0006,
        glyph_pointer_table_base: 0xFB72,
        glyph_pointer_slot_offsets_in_write_order: [0x0014, 0x0010, 0x000C, 0x0008, 0x0004, 0x0000],
        glyph_init_subroutine: "sub_1D786",
        platform_layout_label: "unk_1D75A",
        text_script_label: "EndText1",
    }
}

#[allow(non_snake_case)]
#[allow(dead_code)]
pub(crate) fn end_sequence_text_loop_loc_1D3F0() -> EndSequenceTextLoopLoc1D3F0 {
    // ASM loc_1D3F0 (kid.asm:32741): read the next big-endian word from `a2`.
    // Positive words are page entries, so the loop preserves `a2` via `a4`,
    // calls `sub_1D080` to stream text until the page's `0,0` sentinel, then
    // resumes at the next word. The `beq.w loc_1D44C` zero-word exit ends the
    // sequence. The negative `$FFFF` page-break sentinel falls through
    // `loc_1D404 -> loc_1D412`, waits `$C8 + 1 = $C9` refresh/hibernate pairs,
    // hides cached glyphs, fades/restores the page, reloads glyphs from
    // `(Addr_PlatformLayout).w` via `sub_1D7A8`, and loops back for the next
    // page.
    let page_break_wait_loop = end_sequence_page_break_wait_loop_loc_1D418();

    EndSequenceTextLoopLoc1D3F0 {
        page_text_subroutine: "sub_1D080",
        page_break_word: 0xFFFF,
        page_break_wait_seed: page_break_wait_loop.wait_seed,
        page_break_wait_iterations: page_break_wait_loop.wait_iterations,
        page_break_refresh_subroutine: page_break_wait_loop.refresh_subroutine,
        page_break_hide_glyphs_subroutine: page_break_wait_loop.hide_glyphs_subroutine,
        page_break_fade_subroutine: page_break_wait_loop.fade_subroutine,
        page_break_stripe_subroutine: page_break_wait_loop.stripe_subroutine,
        page_break_glyph_load_subroutine: page_break_wait_loop.glyph_load_subroutine,
        page_break_platform_layout_source: page_break_wait_loop.glyph_layout_source,
        terminator_word: 0x0000,
        terminator_branch: "loc_1D44C",
    }
}

#[allow(non_snake_case)]
#[allow(dead_code)]
pub(crate) fn continue_screen_optional_fade_loc_D372(
    score_object_present: bool,
    bonus_object_present: bool,
) -> ContinueScreenOptionalFadeLocD372 {
    // ASM loc_D372 (kid.asm:16539): after the first countdown expires, the
    // optional score object at `$44(a5)` has already been handled if present.
    // This label checks the optional bonus object at `$48(a5)` independently:
    // null branches straight on, non-null writes display timer `$1000` and
    // velocity `1`. Native does not keep these visual EOL object slots, so this
    // helper documents the decision before the lifecycle timeout returns title.
    ContinueScreenOptionalFadeLocD372 {
        score_object_fades: score_object_present,
        bonus_object_fades: bonus_object_present,
        display_timer_seed: 0x1000,
        falling_velocity: 1,
    }
}

#[allow(non_snake_case)]
#[allow(dead_code)]
pub(crate) fn continue_screen_mandatory_fade_loc_D386() -> ContinueScreenMandatoryFadeLocD386 {
    // ASM loc_D386 (kid.asm:16546): the CONTINUE? text pointer `$4C(a5)` and
    // number pointer `$50(a5)` are mandatory on this path. Both are given
    // display timer `$1000` and falling velocity `1`; the number object stores
    // its timer at `$A(a0)` instead of `$46(a0)`. The ROM then seeds `d0=$EF`
    // for the second wait before the title return.
    ContinueScreenMandatoryFadeLocD386 {
        continue_text_fades: true,
        continue_number_fades: true,
        text_display_timer_seed: 0x1000,
        number_display_timer_seed: 0x1000,
        falling_velocity: 1,
        post_fade_wait_seed: 0x00EF,
    }
}

#[allow(non_snake_case)]
#[allow(dead_code)]
pub(crate) fn continue_timeout_title_palette_backup_loc_D40A()
-> ContinueTimeoutTitlePaletteBackupLocD40A {
    // ASM loc_D40A (kid.asm:16586): after the continue screen times out and
    // loc_D3F0 clears Game_Mode, the ROM copies `Palette_Buffer` into
    // `Palette_Buffer_2` with `moveq #$1F,d0; move.l (a0)+,(a1)+; dbf`.
    // Native does not retain CRAM staging buffers on the title-return path, so
    // this documents the full 128-byte backup before the following fade setup.
    ContinueTimeoutTitlePaletteBackupLocD40A {
        palette_backup_dbf_seed: 0x001F,
        palette_backup_longwords: 0x0020,
    }
}

#[allow(non_snake_case)]
#[allow(dead_code)]
pub(crate) fn continue_timeout_title_palette_clear_loc_D418()
-> ContinueTimeoutTitlePaletteClearLocD418 {
    // ASM loc_D418 (kid.asm:16592): after loc_D40A's backup, `moveq #$3F,d0`
    // drives `move.w #0,(a0)+; dbf d0,loc_D418`, clearing all 64 words of
    // `Palette_Buffer_3`. The following instruction seeds fade progress
    // `$FFFFF876` with `$0100` before the title-return fade loop starts.
    ContinueTimeoutTitlePaletteClearLocD418 {
        palette_clear_dbf_seed: 0x003F,
        palette_clear_words: 0x0040,
        fade_progress_seed: 0x0100,
    }
}

#[allow(non_snake_case)]
#[allow(dead_code)]
pub(crate) fn continue_timeout_title_fade_step_loc_D428(
    fade_progress: u16,
    pending_palette_0_mask: u32,
    pending_palette_1_mask: u32,
) -> ContinueTimeoutTitleFadeStepLocD428 {
    // ASM loc_D428 (kid.asm:16599): each title-return fade frame waits for
    // VBlank, idles once, uploads the palette, runs the palette fade helper,
    // then subtracts `$10` from `$FFFFF876`. Non-negative results rewrite both
    // shadow palette longwords to `-1`; a negative result exits without
    // refreshing them.
    let fade_progress = fade_progress.wrapping_sub(0x0010);
    if (fade_progress as i16) < 0 {
        return ContinueTimeoutTitleFadeStepLocD428 {
            fade_progress,
            pending_palette_0_mask,
            pending_palette_1_mask,
            return_to_title: true,
        };
    }

    // ASM loc_D440 (kid.asm:16607): non-final fade frames seed `d0=-1`,
    // store it to both pending palette longwords, then branch back to loc_D428.
    ContinueTimeoutTitleFadeStepLocD428 {
        fade_progress,
        pending_palette_0_mask: u32::MAX,
        pending_palette_1_mask: u32::MAX,
        return_to_title: false,
    }
}

#[allow(non_snake_case)]
#[allow(dead_code)]
pub(crate) fn continue_timeout_title_return_loc_D44C() -> ContinueTimeoutTitleReturnLocD44C {
    // ASM loc_D44C (kid.asm:16614): after loc_D428 underflows the fade word,
    // the ROM performs one final VBlank/idle/palette/fade-helper refresh,
    // clears `$FFFFFBCC`, sets `$FFFFFC36`, and jumps to Teleport.
    ContinueTimeoutTitleReturnLocD44C {
        final_refresh_subroutine: "j_WaitForVint",
        idle_subroutine: "j_Do_Nothing",
        palette_upload_subroutine: "j_Palette_to_VRAM",
        fade_helper_subroutine: "j_sub_14C0",
        clears_continue_fade_colour: true,
        sets_teleport_flag_fc36: true,
        tail_subroutine: "Teleport",
    }
}

#[allow(non_snake_case)]
#[allow(dead_code)]
pub(crate) fn end_sequence_page_break_dispatch_loc_1D40E() -> EndSequencePageBreakDispatchLoc1D40E {
    // ASM loc_1D40E (kid.asm:32757-32760): the page-break jump-table entry is
    // just `bra.s loc_1D412`, with a trailing `nop` padding word that keeps the
    // PC-relative jump table aligned for the following entry point.
    EndSequencePageBreakDispatchLoc1D40E {
        branch_target: "loc_1D412",
        has_padding_nop: true,
    }
}

#[allow(non_snake_case)]
#[allow(dead_code)]
pub(crate) fn end_sequence_page_break_wait_loop_loc_1D418() -> EndSequencePageBreakWaitLoopLoc1D418
{
    // ASM loc_1D418 (kid.asm:32766-32779): after `loc_1D412` seeds `d0 = $C8`,
    // the loop repeatedly calls `sub_1D830`, hibernates one frame, and uses
    // `dbf d0,loc_1D418` for `$C8 + 1 = $C9` refreshes.
    // ASM loc_1D418 (kid.asm:32771-32779): once the wait expires, hide glyphs
    // with `sub_1D7E6`, fade via `sub_1D800`, rebuild the stripes with
    // `sub_1CDE0`, write `$0EEE0EC0` to `Palette_Buffer+$62`, upload the
    // palette, reload `a4` from `(Addr_PlatformLayout).w`, and call `sub_1D7A8`
    // before returning to the main text loop.
    EndSequencePageBreakWaitLoopLoc1D418 {
        wait_seed: 0x00C8,
        wait_iterations: 0x00C9,
        refresh_subroutine: "sub_1D830",
        hibernate_subroutine: "j_Hibernate_Object_1Frame",
        hide_glyphs_subroutine: "sub_1D7E6",
        fade_subroutine: "sub_1D800",
        stripe_subroutine: "sub_1CDE0",
        palette_buffer_word_offset: 0x0062,
        palette_buffer_longword: 0x0EEE0EC0,
        palette_upload_subroutine: "j_Palette_to_VRAM",
        glyph_layout_source: "Addr_PlatformLayout",
        glyph_load_subroutine: "sub_1D7A8",
    }
}

#[allow(non_snake_case)]
#[allow(dead_code)]
pub(crate) fn end_sequence_terminator_exit_loc_1D450() -> EndSequenceTerminatorExitLoc1D450 {
    // ASM loc_1D450 (kid.asm:32784-32798): seed `d0 = $190`, then run the same
    // `sub_1D830 -> j_Hibernate_Object_1Frame -> dbf` loop for `$190 + 1 =
    // $191` refreshes before clearing `Palette_Buffer` word-by-word.
    // ASM loc_1D464 (kid.asm:32791): `clr.w (a0)+; dbf d0,loc_1D464` starts
    // from `d0 = $3F`, so the inclusive DBF clears `$40` palette words before
    // the final upload/reset tail runs.
    // ASM loc_1D450 (kid.asm:32794-32798): after the `loc_1D464` clear loop,
    // the tail waits for VBlank, idles once, uploads the cleared palette, sets
    // `($FFFFFBCE).w`, clears `($FFFFFBCC).w`, `(Game_Mode).w`, and
    // `(Number_Lives).w`, then jumps through the callback in `(off_7192).w`.
    EndSequenceTerminatorExitLoc1D450 {
        wait_seed: 0x0190,
        wait_iterations: 0x0191,
        refresh_subroutine: "sub_1D830",
        hibernate_subroutine: "j_Hibernate_Object_1Frame",
        palette_clear_dbf_seed: 0x003F,
        palette_clear_words: 0x0040,
        wait_for_vint_subroutine: "j_WaitForVint",
        idle_subroutine: "j_Do_Nothing",
        palette_upload_subroutine: "j_Palette_to_VRAM",
        sets_unknown_flag_fbce: true,
        clears_word_fbcc: true,
        clears_game_mode: true,
        clears_number_lives: true,
        continuation_pointer_source: "off_7192",
    }
}

#[allow(non_snake_case)]
#[allow(dead_code)]
pub(crate) fn end_sequence_stripe_loop_loc_1CDE4() -> EndSequenceStripeLoopLoc1CDE4 {
    // ASM sub_1CDE0 (kid.asm:31978): seed both `d0` and `d1` with `#9`
    // before entering the outer stripe loop.
    // ASM loc_1CDE4 (kid.asm:31985): each pass calls `sub_1DA24`, runs
    // `sub_1CDFA` twice with the current column then the incremented column,
    // advances `d0` a second time, and loops with `dbf d1,loc_1CDE4`. That
    // yields `$9 + 1 = $A` iterations, painting column pairs `(9,10)` through
    // `(27,28)` before leaving `d0` poised at 29 for the caller.
    EndSequenceStripeLoopLoc1CDE4 {
        start_column: 0x0009,
        iteration_seed: 0x0009,
        iterations: 0x000A,
        stripe_columns_per_iteration: 0x0002,
        final_next_column: 0x001D,
    }
}

#[allow(non_snake_case)]
#[allow(dead_code)]
pub(crate) fn end_sequence_stripe_fill_loop_loc_1CE1C() -> EndSequenceStripeFillLoopLoc1CE1C {
    // ASM sub_1CDFA (kid.asm:31994): after the `j_sub_914` Z80 guard and the
    // stripe base-address setup, the routine seeds `d7` with `#$1B`.
    // ASM loc_1CE1C (kid.asm:32012): `move.w #$94,(a6)` executes before
    // `dbf d7,loc_1CE1C`, so the fill loop writes `$1B + 1 = $1C` copies of
    // tile/pattern word `$0094`, then exits through `j_sub_924`.
    EndSequenceStripeFillLoopLoc1CE1C {
        fill_word: 0x0094,
        iteration_seed: 0x001B,
        iterations: 0x001C,
        z80_guard_subroutines: ["j_sub_914", "j_sub_924"],
    }
}

#[allow(non_snake_case)]
#[allow(dead_code)]
pub(crate) fn options_menu_text_draw_loc_1CE3A(
    selected_option: u16,
    two_player_two_controllers: bool,
) -> OptionsMenuTextDrawLoc1CE3A {
    // ASM DrawOptText1 (kid.asm:32028): `d3` starts at 0 and only flips to 1
    // when `Options_Selected_Option == 0`, so the first line is highlighted
    // only while the 2 PLAYERS row is selected.
    // ASM loc_1CE3A (kid.asm:32034): this shared entry always draws row 0 from
    // `OptText1`, then seeds `d4 = 1` and `a4 = OptText2` for the second row.
    // ASM loc_1CE56 (kid.asm:32044): both the default path and the `OptText3`
    // override rejoin here and issue the shared second `DrawTextLine_Offset`.
    OptionsMenuTextDrawLoc1CE3A {
        first_line_row: 0,
        first_line_highlighted: selected_option == 0,
        second_line_row: 1,
        second_line_text_label: if two_player_two_controllers {
            "OptText3"
        } else {
            "OptText2"
        },
    }
}

#[allow(non_snake_case)]
#[allow(dead_code)]
pub(crate) fn options_menu_controls_header_loc_1CEB2(
    selected_option: u16,
) -> OptionsMenuControlsHeaderLoc1CEB2 {
    // ASM DrawOptText2 (kid.asm:32126): `d3` stays zero unless
    // `Options_Selected_Option == 1`, so only the CONTROLS/MODE header pair
    // highlights when the cursor is on row 1.
    // ASM loc_1CEB2 (kid.asm:32132): draw the first packed `OptText4` entry at
    // `(0,3)`, then rely on the first `DrawTextLine_Offset` call advancing
    // `a4` to the next NUL-terminated entry before `addq.w #1,a4` lands on the
    // second `(11,3)` header entry.
    OptionsMenuControlsHeaderLoc1CEB2 {
        highlighted: selected_option == 1,
        first_line_x: 0,
        first_line_y: 3,
        first_line_text_label: "OptText4.CONTROLS",
        second_line_x: 11,
        second_line_y: 3,
        second_line_text_label: "OptText4.MODE",
    }
}

#[allow(non_snake_case)]
#[allow(dead_code)]
pub(crate) fn options_menu_controls_binding_loop_loc_1CEF8(
    controls_scheme: u8,
) -> OptionsMenuControlsBindingLoopLoc1CEF8 {
    fn controls_binding_labels(controls_scheme: u8) -> [&'static str; 3] {
        match controls_scheme {
            0 => ["OptText5.SPEED", "OptText5.JUMP", "OptText5.SPECIAL"],
            1 => ["OptText5.SPEED", "OptText5.SPECIAL", "OptText5.JUMP"],
            2 => ["OptText5.JUMP", "OptText5.SPEED", "OptText5.SPECIAL"],
            3 => ["OptText5.JUMP", "OptText5.SPECIAL", "OptText5.SPEED"],
            4 => ["OptText5.SPECIAL", "OptText5.SPEED", "OptText5.JUMP"],
            5 => ["OptText5.SPECIAL", "OptText5.JUMP", "OptText5.SPEED"],
            _ => panic!("unsupported controls scheme: {controls_scheme}"),
        }
    }

    // ASM loc_1CEF8 (kid.asm:32159): `moveq #2,d2` seeds a three-iteration DBF
    // loop. Each pass reads one byte from `unk_1CF28 + controls_scheme * 4`,
    // uses it as an offset into `OptText5`, draws the label at the current row,
    // then `addq.w #2,d1` advances to the next binding row.
    OptionsMenuControlsBindingLoopLoc1CEF8 {
        dbf_seed: 0x0002,
        iterations: 0x0003,
        row_start: 15,
        row_step: 2,
        binding_text_labels: controls_binding_labels(controls_scheme),
    }
}

#[allow(non_snake_case)]
#[allow(dead_code)]
pub(crate) fn draw_text_line_palette_base_loc_1D05E(highlighted: bool) -> u16 {
    // ASM loc_1D05E (kid.asm:32424): after `tst.b d3`, leave `d6 = $C000`
    // when zero and overwrite it with `$E000` when non-zero, then add that
    // palette-line base into the glyph tile word.
    if highlighted { 0xE000 } else { 0xC000 }
}

#[allow(non_snake_case)]
#[allow(dead_code)]
pub(crate) fn draw_text_line_tile_base_loc_1D050(highlighted: bool, alternate_font: bool) -> u16 {
    // ASM DrawTextLine (kid.asm:32380): seed `d7 = $4DC`, switch to `$509`
    // when `tst.b d4` is non-zero, then fall through `loc_1D05E` to add the
    // palette line selected from the highlight bit in `d3`.
    let glyph_base = if alternate_font { 0x0509 } else { 0x04DC };
    let palette_base = draw_text_line_palette_base_loc_1D05E(highlighted);
    palette_base + glyph_base
}

#[allow(non_snake_case)]
#[allow(dead_code)]
pub(crate) fn draw_text_line_glyph_loop_loc_1D068(
    tile_base: u16,
    text_bytes: &[u8],
) -> DrawTextLineGlyphLoopLoc1D068 {
    // ASM loc_1D068 (kid.asm:32429): clear `d5`, read the next `(a4)+` byte,
    // stop the write loop when that byte is zero, otherwise subtract `$41`,
    // add the result onto the tile base in `d7`, and stream the word to the
    // plane before looping for the next byte.
    let mut glyph_tiles = Vec::new();
    let mut bytes_consumed = 0usize;

    for &next_letter in text_bytes {
        bytes_consumed += 1;
        if next_letter == 0 {
            break;
        }

        let glyph_offset = u16::from(next_letter.wrapping_sub(0x41));
        glyph_tiles.push(tile_base.wrapping_add(glyph_offset));
    }

    DrawTextLineGlyphLoopLoc1D068 {
        glyph_tiles,
        bytes_consumed,
    }
}

#[allow(non_snake_case)]
#[allow(dead_code)]
pub(crate) fn draw_text_line_terminator_loc_1D07A(
    next_letter: u8,
) -> Option<DrawTextLineTerminatorLoc1D07A> {
    // ASM loc_1D07A (kid.asm:32439): the NUL-byte branch from `loc_1D068`
    // lands here, calls `j_sub_924`, and immediately returns from
    // `DrawTextLine`.
    (next_letter == 0).then_some(DrawTextLineTerminatorLoc1D07A {
        flushes_vdp_guard: true,
        returns_after_flush: true,
    })
}

#[allow(non_snake_case)]
#[allow(dead_code)]
pub(crate) fn credits_glyph_preview_entry_loc_1D0A2(
    vdp_command: u32,
    next_letter: u8,
) -> CreditsGlyphPreviewEntryLoc1D0A2 {
    // ASM loc_1D0A2 (kid.asm:32463): always write the precomputed VDP command
    // longword from `d5` to `4(a6)` before reading the next glyph byte.
    // ASM loc_1D0A2 (kid.asm:32465-32472): a zero byte branches straight to
    // `return_1D0EA`; otherwise subtract `$41`, add `-$1B24`, guard the VDP
    // write with `j_sub_914/j_sub_924`, store the resulting tile word to
    // `(a6)`, and seed `d6 = 2` for the shared three-pass refresh loop.
    let tile_word = (next_letter != 0).then(|| {
        u16::from(next_letter)
            .wrapping_sub(0x0041)
            .wrapping_sub(0x1B24)
    });

    CreditsGlyphPreviewEntryLoc1D0A2 {
        vdp_command,
        tile_word,
        z80_guard_subroutines: tile_word.map(|_| ["j_sub_914", "j_sub_924"]),
        refresh_dbf_seed: tile_word.map(|_| 0x0002),
    }
}

#[allow(non_snake_case)]
#[allow(dead_code)]
pub(crate) fn credits_glyph_preview_refresh_loop_loc_1D0C2(
    vdp_command: u32,
) -> CreditsGlyphPreviewRefreshLoopLoc1D0C2 {
    // ASM loc_1D0C2 (kid.asm:32475-32482): with `d6 = 2` from the preceding
    // entry path, the loop calls `sub_1DA24`, `sub_1CC88`,
    // `j_Make_SpritesFromGfxObjects`, `j_WaitForVint`, and
    // `j_Transfer_SpriteAndKidToVRAM` in that order on each pass.
    // ASM loc_1D0C2 (kid.asm:32477-32481): `movem.l d0-a5,-(sp)` preserves the
    // caller-visible register window across the VBlank/render helpers.
    // ASM loc_1D0C2 (kid.asm:32482-32484): `dbf d6,loc_1D0C2` yields
    // `$2 + 1 = 3` refresh passes before `addi.l #$20000,d5` advances the VDP
    // command to the next row for the following glyph.
    CreditsGlyphPreviewRefreshLoopLoc1D0C2 {
        dbf_seed: 0x0002,
        iterations: 0x0003,
        subroutine_order: [
            "sub_1DA24",
            "sub_1CC88",
            "j_Make_SpritesFromGfxObjects",
            "j_WaitForVint",
            "j_Transfer_SpriteAndKidToVRAM",
        ],
        preserves_register_window: true,
        next_vdp_command: vdp_command.wrapping_add(0x0002_0000),
    }
}

#[allow(non_snake_case)]
#[allow(dead_code)]
pub(crate) fn options_menu_controls_bindings_loc_1CED6(
    selected_option: u16,
    controls_scheme: u8,
) -> OptionsMenuControlsBindingsLoc1CED6 {
    let highlighted = selected_option == 1;
    let loop_shape = options_menu_controls_binding_loop_loc_1CEF8(controls_scheme);

    // ASM loc_1CED6 (kid.asm:32145): after the highlight palette choice, add
    // `Options_Suboption_Controls + $1B` onto the alternate-font tile base
    // chosen by DrawTextLine's `loc_1D050`, then write the resulting selector
    // word to the VDP data port.
    // ASM loc_1CEF8 (kid.asm:32159) then consumes the selected `unk_1CF28`
    // stride as three `OptText5` offsets for the binding labels.
    // The trailing `OptText6` sequence draws the fixed A/B/C badges on those
    // same three rows.
    OptionsMenuControlsBindingsLoc1CED6 {
        highlighted,
        selection_tile_word: draw_text_line_tile_base_loc_1D050(highlighted, true)
            + 0x001B
            + u16::from(controls_scheme),
        binding_label_rows: [
            loop_shape.row_start,
            loop_shape.row_start + loop_shape.row_step,
            loop_shape.row_start + (loop_shape.row_step * 2),
        ],
        binding_text_labels: loop_shape.binding_text_labels,
        button_badge_rows: [15, 17, 19],
        button_badge_text_labels: ["OptText6.A", "OptText6.B", "OptText6.C"],
    }
}

#[allow(non_snake_case)]
#[allow(dead_code)]
pub(crate) fn options_menu_speed_text_loc_1CF78(
    selected_option: u16,
    speed_button_fast: bool,
) -> OptionsMenuSpeedTextLoc1CF78 {
    let (mode_text_label, prompt_text_label) = if speed_button_fast {
        ("OptText9.FAST_ACTION", "OptText9.PUSH_FOR_NORMAL")
    } else {
        ("OptText8.NORMAL_ACTION", "OptText8.PUSH_FOR_FAST")
    };

    // ASM DrawOptText3 (kid.asm:32248): `d3` only flips to 1 when
    // `Options_Selected_Option == 2`, so the SPEED BUTTON block highlights
    // only while row 2 is selected.
    // ASM loc_1CF78 (kid.asm:32254): always draw `OptText7` at `(0,$C)`, then
    // switch to the packed `( $E,$C )` / `( $E,$E )` entries from `OptText8`
    // or `OptText9` depending on `Options_Suboption_Speed`.
    OptionsMenuSpeedTextLoc1CF78 {
        highlighted: selected_option == 2,
        header_x: 0,
        header_y: 12,
        header_text_label: "OptText7.SPEED_BUTTON",
        mode_x: 14,
        mode_y: 12,
        mode_text_label,
        prompt_x: 14,
        prompt_y: 14,
        prompt_text_label,
    }
}

#[allow(non_snake_case)]
#[allow(dead_code)]
pub(crate) fn options_menu_exit_text_loc_1CFFE(
    selected_option: u16,
) -> OptionsMenuExitTextLoc1CFFE {
    // ASM DrawOptText4 (kid.asm:32357): `d3` stays zero unless the cursor is on
    // row 3, so only EXIT highlights there.
    // ASM loc_1CFFE (kid.asm:32363): regardless of highlight, seed `d4 = 0`,
    // point `a4` at `OptText10`, and tail into `DrawTextLine_Offset`.
    OptionsMenuExitTextLoc1CFFE {
        highlighted: selected_option == 3,
        text_x: 0,
        text_y: 17,
        text_label: "OptText10.EXIT",
    }
}

#[allow(non_snake_case)]
#[allow(dead_code)]
pub(crate) fn level_layout_read_pass_loc_1CCC2() -> LevelLayoutReadPassLoc1CCC2 {
    // ASM sub_1CCAE (kid.asm:31826): seed `d0` with `$45F` before entering the
    // first VRAM readback loop.
    // ASM loc_1CCC2 (kid.asm:31837): `bsr.w sub_1CD20`, then `bsr.w sub_1CD32`,
    // then `dbf d0,loc_1CCC2`, so the loop runs `$45F+1 = $460` iterations.
    LevelLayoutReadPassLoc1CCC2 {
        dbf_seed: 0x045F,
        iterations: 0x0460,
        subroutine_order: ["sub_1CD20", "sub_1CD32"],
    }
}

#[allow(non_snake_case)]
#[allow(dead_code)]
pub(crate) fn level_layout_copy_to_vram_loc_1CCDE() -> LevelLayoutCopyToVramLoc1CCDE {
    // ASM sub_1CCAE (kid.asm:31826): after the first readback pass the routine
    // retargets the VDP port for VRAM writes and seeds `d0` with `$117F`.
    // ASM loc_1CCDE (kid.asm:31845): `move.w (a0)+,(a6)` then
    // `dbf d0,loc_1CCDE`, so the Level_Layout upload writes `$117F + 1 = $1180`
    // words to VRAM.
    LevelLayoutCopyToVramLoc1CCDE {
        dbf_seed: 0x117F,
        words_copied: 0x1180,
    }
}

#[allow(non_snake_case)]
#[allow(dead_code)]
pub(crate) fn level_layout_delay_loc_1CCE8() -> LevelLayoutDelayLoc1CCE8 {
    // ASM loc_1CCE8 (kid.asm:31850): `move.w #$100,d0` then `dbf d0,loc_1CCE8`,
    // so this is a pure busy-wait lasting `$100 + 1 = $101` DBF iterations.
    LevelLayoutDelayLoc1CCE8 {
        dbf_seed: 0x0100,
        iterations: 0x0101,
    }
}

#[allow(non_snake_case)]
#[allow(dead_code)]
pub(crate) fn level_layout_read_pass_loc_1CCFC() -> LevelLayoutReadPassLoc1CCC2 {
    // ASM sub_1CCAE (kid.asm:31826): after the delay loop the routine reloads
    // the VDP read command, re-seeds `d0` with `$45F`, and re-enters the
    // readback pass.
    // ASM loc_1CCFC (kid.asm:31856): `bsr.w sub_1CD32`, then `bsr.w sub_1CD20`,
    // then `dbf d0,loc_1CCFC`, reversing the earlier subroutine order while
    // keeping the same `$45F+1 = $460` iteration count.
    LevelLayoutReadPassLoc1CCC2 {
        dbf_seed: 0x045F,
        iterations: 0x0460,
        subroutine_order: ["sub_1CD32", "sub_1CD20"],
    }
}

#[allow(non_snake_case)]
#[allow(dead_code)]
pub(crate) fn level_layout_final_copy_to_vram_loc_1CD18() -> LevelLayoutCopyToVramLoc1CCDE {
    // ASM sub_1CCAE (kid.asm:31826): after the second readback pass the routine
    // reuses the same `lea (Level_Layout).w,a0` / `move.w #$117F,d0` setup for
    // the final VRAM upload.
    // ASM loc_1CD18 (kid.asm:31864): `move.w (a0)+,(a6)` then
    // `dbf d0,loc_1CD18`, so the closing Level_Layout upload writes
    // `$117F + 1 = $1180` words to the VDP data port.
    LevelLayoutCopyToVramLoc1CCDE {
        dbf_seed: 0x117F,
        words_copied: 0x1180,
    }
}

#[cfg_attr(not(test), allow(dead_code))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct VdpBit15ClearToDecompressionBufferLoc1CD5A {
    pub(crate) dbf_seed: u16,
    pub(crate) words_copied: u16,
    pub(crate) cleared_mask: u16,
    pub(crate) destination_base: u32,
    pub(crate) vram_word_addr: u16,
}

#[allow(non_snake_case)]
#[allow(dead_code)]
pub(crate) fn vdp_bit15_clear_to_decompression_buffer_loc_1CD5A()
-> VdpBit15ClearToDecompressionBufferLoc1CD5A {
    // ASM sub_1CD44 (kid.asm:31898): clear the VDP command register first,
    // point `a4` at `Decompression_Buffer`, and seed `d7` with `$7FF`.
    // ASM loc_1CD5A (kid.asm:31908): each pass reads one VDP word, clears bit
    // 15 with `bclr #$F,d6`, stores the masked word to `Decompression_Buffer`,
    // then loops inclusively through `$7FF + 1 = $800` words before retargeting
    // the VDP to `vdpComm($0000,VRAM,WRITE)`.
    VdpBit15ClearToDecompressionBufferLoc1CD5A {
        dbf_seed: 0x07FF,
        words_copied: 0x0800,
        cleared_mask: 0x7FFF,
        destination_base: 0xFFFF_77B2,
        vram_word_addr: 0x0000,
    }
}

#[cfg_attr(not(test), allow(dead_code))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct VdpUploadDecompressionBufferLoc1CD78 {
    pub(crate) dbf_seed: u16,
    pub(crate) words_uploaded: u16,
    pub(crate) source_base: u32,
}

#[allow(non_snake_case)]
#[allow(dead_code)]
pub(crate) fn vdp_upload_decompression_buffer_loc_1CD78() -> VdpUploadDecompressionBufferLoc1CD78 {
    // ASM sub_1CD44 (kid.asm:31898): after the bit-15 clear pass, the routine
    // retargets the VDP port for VRAM writes, rewinds `a4` to
    // `Decompression_Buffer`, and re-seeds `d7` with `$7FF`.
    // ASM loc_1CD78 (kid.asm:31917): `move.w (a4)+,(a6)` then
    // `dbf d7,loc_1CD78`, so the upload writes `$7FF + 1 = $800` masked words
    // from `Decompression_Buffer` back into VRAM.
    VdpUploadDecompressionBufferLoc1CD78 {
        dbf_seed: 0x07FF,
        words_uploaded: 0x0800,
        source_base: 0xFFFF_77B2,
    }
}

#[cfg_attr(not(test), allow(dead_code))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct VdpPostUploadDelayLoc1CD82 {
    pub(crate) dbf_seed: u16,
    pub(crate) iterations: u16,
}

#[allow(non_snake_case)]
#[allow(dead_code)]
pub(crate) fn vdp_post_upload_delay_loc_1CD82() -> VdpPostUploadDelayLoc1CD82 {
    // ASM sub_1CD44 (kid.asm:31898): once the `$800`-word upload at loc_1CD78
    // completes, the routine seeds `d7` with `$100` for a final fixed delay.
    // ASM loc_1CD82 (kid.asm:31922): `dbf d7,loc_1CD82` performs a pure
    // busy-wait, so the delay lasts `$100 + 1 = $101` iterations.
    VdpPostUploadDelayLoc1CD82 {
        dbf_seed: 0x0100,
        iterations: 0x0101,
    }
}

#[cfg_attr(not(test), allow(dead_code))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct VdpPatternStripeOuterLoopLoc1CD90 {
    pub(crate) row_seed: u16,
    pub(crate) row_iterations: u16,
}

#[allow(non_snake_case)]
#[allow(dead_code)]
pub(crate) fn vdp_pattern_stripe_outer_loop_loc_1CD90() -> VdpPatternStripeOuterLoopLoc1CD90 {
    // ASM sub_1CD88 (kid.asm:31931): before entering the nested write loops,
    // the routine seeds `d1` with `$1B` and points `a0` at
    // `Decompression_Buffer`.
    // ASM loc_1CD90 (kid.asm:31935): `moveq #$1B,d1` feeds `dbf d1,loc_1CD90`,
    // so the outer stripe loop runs `$1B + 1 = $1C` rows.
    VdpPatternStripeOuterLoopLoc1CD90 {
        row_seed: 0x001B,
        row_iterations: 0x001C,
    }
}

#[cfg_attr(not(test), allow(dead_code))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct VdpPatternStripeWordCopyLoopLoc1CD92 {
    pub(crate) source: u32,
    pub(crate) copy_seed: u16,
    pub(crate) copy_iterations: u16,
}

#[cfg_attr(not(test), allow(dead_code))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct PlaneLineWordCopyLoc1CDD4 {
    pub(crate) source_base: u32,
    pub(crate) source_byte_offset: u16,
    pub(crate) words_copied: u16,
}

#[allow(non_snake_case)]
#[allow(dead_code)]
pub(crate) fn vdp_pattern_stripe_word_copy_loop_loc_1CD92() -> VdpPatternStripeWordCopyLoopLoc1CD92
{
    // ASM loc_1CD90 (kid.asm:31935): each outer-row pass reloads `d0` with
    // `$27` before entering the inner copy loop.
    // ASM loc_1CD92 (kid.asm:31938): `move.w (a0)+,(a6)` executes before
    // `dbf d0,loc_1CD92`, so the loop copies `$27 + 1 = $28` words from
    // `Decompression_Buffer` into the VDP data port on each row.
    VdpPatternStripeWordCopyLoopLoc1CD92 {
        source: 0xFFFF77B2,
        copy_seed: 0x0027,
        copy_iterations: 0x0028,
    }
}

#[allow(non_snake_case)]
#[allow(dead_code)]
pub(crate) fn plane_line_word_copy_loc_1CDD4(row_index: u16) -> PlaneLineWordCopyLoc1CDD4 {
    // ASM sub_1CDA8 (kid.asm:31952): `lea (Decompression_Buffer).l,a4`
    // followed by `mulu.w #$50,d7; add.w d7,a4` picks one `$50`-byte source
    // row from `Decompression_Buffer` before the inner line copy starts.
    // ASM loc_1CDD4 (kid.asm:31970): `move.w (a4)+,(a6)` executes before
    // `dbf d7,loc_1CDD4`, so the loop copies `$27 + 1 = $28` words from that
    // row into the VDP data port.
    PlaneLineWordCopyLoc1CDD4 {
        source_base: 0xFFFF77B2,
        source_byte_offset: row_index.wrapping_mul(0x0050),
        words_copied: 0x0028,
    }
}

#[cfg_attr(not(test), allow(dead_code))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct VdpPatternStripeFillLoopLoc1CD9A {
    pub(crate) fill_word: u16,
    pub(crate) fill_seed: u16,
    pub(crate) fill_iterations: u16,
}

#[allow(non_snake_case)]
#[allow(dead_code)]
pub(crate) fn vdp_pattern_stripe_fill_loop_loc_1CD9A() -> VdpPatternStripeFillLoopLoc1CD9A {
    // ASM loc_1CD92 (kid.asm:31938): after the 40 copied words, the routine
    // reloads `d0` with `$17` for the trailing fill segment on the same row.
    // ASM loc_1CD9A (kid.asm:31943): `move.w #$94,(a6)` executes before
    // `dbf d0,loc_1CD9A`, so each row appends `$17 + 1 = $18` copies of
    // tile/pattern word `$0094` to the VDP data port.
    VdpPatternStripeFillLoopLoc1CD9A {
        fill_word: 0x0094,
        fill_seed: 0x0017,
        fill_iterations: 0x0018,
    }
}

#[allow(non_snake_case)]
#[allow(dead_code)]
pub(crate) fn end_sequence_camera_shake_sub_1D846(
    random_x_byte: u8,
    random_y_byte: u8,
    v_int_counter: u8,
) -> (i16, i16) {
    // ASM sub_1D846 (kid.asm:33087): fetch two random bytes, XOR both with
    // `V_Int_counter` after `bclr #7,d5`, sign-extend each byte to a word,
    // arithmetic-shift right by 5, then store the results in Camera_X/Y_pos.
    let masked_v_int = v_int_counter & 0x7F;
    let camera_x_pos = i16::from((random_x_byte ^ masked_v_int) as i8) >> 5;
    let camera_y_pos = i16::from((random_y_byte ^ masked_v_int) as i8) >> 5;
    (camera_x_pos, camera_y_pos)
}

#[cfg_attr(not(test), allow(dead_code))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct EndSequenceBurstPreludeLoc1D1D6 {
    pub(crate) burst_subroutine: &'static str,
    pub(crate) burst_priority: u8,
    pub(crate) wait_seed: u16,
    pub(crate) wait_iterations: u16,
}

#[allow(non_snake_case)]
#[allow(dead_code)]
pub(crate) fn end_sequence_burst_prelude_loc_1D1D6() -> EndSequenceBurstPreludeLoc1D1D6 {
    // ASM loc_1D1D6 (kid.asm:32582-32585): load `d4 = 1`, call `sub_1D86C`,
    // then seed `d1 = 1` so the following `dbf d1,loc_1D1DE` hibernates
    // twice before the next burst spawn.
    EndSequenceBurstPreludeLoc1D1D6 {
        burst_subroutine: "sub_1D86C",
        burst_priority: 1,
        wait_seed: 0x0001,
        wait_iterations: 0x0002,
    }
}

#[cfg_attr(not(test), allow(dead_code))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct EndSequenceBurstWaitLoopLoc1D1DE {
    pub(crate) hibernate_subroutine: &'static str,
    pub(crate) inner_wait_seed: u16,
    pub(crate) inner_wait_iterations: u16,
    pub(crate) outer_loop_seed: u16,
    pub(crate) outer_loop_iterations: u16,
    pub(crate) next_outer_label: &'static str,
}

#[allow(non_snake_case)]
#[allow(dead_code)]
pub(crate) fn end_sequence_burst_wait_loop_loc_1D1DE() -> EndSequenceBurstWaitLoopLoc1D1DE {
    // ASM loc_1D1DE (kid.asm:32587-32590): hibernate once per inner-loop body,
    // let `dbf d1,loc_1D1DE` consume the `d1 = 1` seed from `loc_1D1D6`, then
    // let `dbf d0,loc_1D1D6` consume the `d0 = $14` seed from `loc_1D1CE`.
    EndSequenceBurstWaitLoopLoc1D1DE {
        hibernate_subroutine: "j_Hibernate_Object_1Frame",
        inner_wait_seed: 0x0001,
        inner_wait_iterations: 0x0002,
        outer_loop_seed: 0x0014,
        outer_loop_iterations: 0x0015,
        next_outer_label: "loc_1D1D6",
    }
}

#[cfg_attr(not(test), allow(dead_code))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct EndSequenceBurstCameraLoopLoc1D1EC {
    pub(crate) burst_priority: u8,
    pub(crate) outer_loop_seed: u16,
    pub(crate) outer_loop_iterations: u16,
    pub(crate) per_iteration_subroutines: [&'static str; 4],
}

#[allow(non_snake_case)]
#[allow(dead_code)]
pub(crate) fn end_sequence_burst_camera_loop_loc_1D1EC() -> EndSequenceBurstCameraLoopLoc1D1EC {
    // ASM loc_1D1EC (kid.asm:32593-32599): load `d4 = 1`, then on each outer
    // loop body call `sub_1D86C`, `sub_1D846`, `sub_1DA24`, and
    // `j_Hibernate_Object_1Frame` before `dbf d0,loc_1D1EC` consumes the
    // `$5A` counter loaded after `loc_1D1DE`.
    EndSequenceBurstCameraLoopLoc1D1EC {
        burst_priority: 1,
        outer_loop_seed: 0x005A,
        outer_loop_iterations: 0x005B,
        per_iteration_subroutines: [
            "sub_1D86C",
            "sub_1D846",
            "sub_1DA24",
            "j_Hibernate_Object_1Frame",
        ],
    }
}

#[allow(non_snake_case)]
#[allow(dead_code)]
pub(crate) fn end_sequence_burst_sprite_init_sub_1D86C(
    random_x_byte: u8,
    random_y_byte: u8,
    v_int_counter: u8,
    priority: u8,
) -> EndSequenceBurstSpriteInitSub1D86C {
    // ASM sub_1D86C (kid.asm:33107): fetch two random bytes, XOR them with
    // `V_Int_counter` after `bclr #7,d5`, sign-extend and `asr.w #1` them,
    // then allocate an object running `sub_1D9D6` at `(#$A0+d6,#$70+d7)`
    // while copying `d4` into `$48(a0)`.
    let masked_v_int = v_int_counter & 0x7F;
    let x_offset = i16::from((random_x_byte ^ masked_v_int) as i8) >> 1;
    let y_offset = i16::from((random_y_byte ^ masked_v_int) as i8) >> 1;
    EndSequenceBurstSpriteInitSub1D86C {
        callback_label: "sub_1D9D6",
        x_pos: 0x00A0_i16.wrapping_add(x_offset),
        y_pos: 0x0070_i16.wrapping_add(y_offset),
        priority,
    }
}

#[allow(non_snake_case)]
#[allow(dead_code)]
pub(crate) fn end_sequence_falling_sprite_init_sub_1D8B4(
    x_pos: u16,
    y_pos: u16,
    sprite_mapping: u16,
    random_delay_byte: u8,
    v_int_counter: u8,
) -> EndSequenceFallingSpriteInitSub1D8B4 {
    // ASM sub_1D8B4 (kid.asm:33134): load gfx slot `$1000000`, set `$13(a3)`,
    // palette line 2, tile `$3DD`, and copy the caller's `$44/$46/$48` words
    // into x/y/sprite fields. The front delay uses the full `V_Int_counter`
    // byte (`eor.b d0,d7`; `lsr.w #1,d7`), then `loc_1D8FA` hibernates and
    // adds an accelerating `d1 += $800` fall for `#$78+1` frames.
    let initial_hibernate_frames = u16::from(random_delay_byte ^ v_int_counter) >> 1;
    let fall_frames = 0x0079;
    let last_frame_y_delta_fp = u32::from(fall_frames - 1) * 0x0800;
    let total_y_delta_fp = (0..u32::from(fall_frames))
        .map(|frame| frame * 0x0800)
        .sum();
    EndSequenceFallingSpriteInitSub1D8B4 {
        gfx_slot_base: 0x0100_0000,
        display_enabled: true,
        palette_line: 2,
        vram_tile: 0x03DD,
        x_pos,
        y_pos,
        sprite_mapping,
        initial_hibernate_frames,
        fall_frames,
        first_frame_y_delta_fp: 0,
        last_frame_y_delta_fp,
        total_y_delta_fp,
    }
}

#[cfg_attr(not(test), allow(dead_code))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct EndSequenceFallingSpriteSpawnInput {
    pub(crate) x_pos: u16,
    pub(crate) y_pos: u16,
    pub(crate) sprite_mapping: u16,
}

#[cfg_attr(not(test), allow(dead_code))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct EndSequenceFallingSpriteSpawnLoopLoc1D226 {
    pub(crate) object_slot_flags: u16,
    pub(crate) callback_label: &'static str,
    pub(crate) loop_seed: u16,
    pub(crate) iterations: u16,
    pub(crate) spawn_inputs: [EndSequenceFallingSpriteSpawnInput; 3],
}

#[allow(non_snake_case)]
#[allow(dead_code)]
pub(crate) fn end_sequence_falling_sprite_spawn_loop_loc_1D226()
-> EndSequenceFallingSpriteSpawnLoopLoc1D226 {
    // ASM loc_1D226 (kid.asm:32611-32619): point `a4` at `word_1D83A`,
    // start `d0 = 2` / `d1 = $1268`, then each `dbf d0,loc_1D226` pass sets
    // `a0 = $6000`, calls `j_Allocate_ObjectSlot`, writes callback
    // `#sub_1D8B4`, copies one x/y pair from `word_1D83A`, stores the current
    // `d1` sprite word at `$48(a0)`, and `addq.w #4,d1` for the next spawn.
    const WORD_1D83A_PAIRS: [(u16, u16); 3] =
        [(0x00AF, 0x0064), (0x0099, 0x007F), (0x008C, 0x005A)];
    let mut sprite_mapping = 0x1268;
    let spawn_inputs = WORD_1D83A_PAIRS.map(|(x_pos, y_pos)| {
        let input = EndSequenceFallingSpriteSpawnInput {
            x_pos,
            y_pos,
            sprite_mapping,
        };
        sprite_mapping += 4;
        input
    });

    EndSequenceFallingSpriteSpawnLoopLoc1D226 {
        object_slot_flags: 0x6000,
        callback_label: "sub_1D8B4",
        loop_seed: 0x0002,
        iterations: 0x0003,
        spawn_inputs,
    }
}

#[cfg_attr(not(test), allow(dead_code))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct EndSequenceBurstLowPriorityPreludeLoc1D24A {
    pub(crate) burst_subroutine: &'static str,
    pub(crate) burst_priority: u8,
    pub(crate) wait_seed: u16,
}

#[allow(non_snake_case)]
#[allow(dead_code)]
pub(crate) fn end_sequence_burst_low_priority_prelude_loc_1D24A()
-> EndSequenceBurstLowPriorityPreludeLoc1D24A {
    // ASM loc_1D24A (kid.asm:32622-32625): after `moveq #$32,d0`,
    // clear `d4`, call `sub_1D86C`, then seed `d1 = 1` for the inner
    // `dbf d1,loc_1D252` wait loop.
    EndSequenceBurstLowPriorityPreludeLoc1D24A {
        burst_subroutine: "sub_1D86C",
        burst_priority: 0,
        wait_seed: 0x0001,
    }
}

#[cfg_attr(not(test), allow(dead_code))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct EndSequenceBurstLowPriorityWaitLoopLoc1D252 {
    pub(crate) hibernate_subroutine: &'static str,
    pub(crate) inner_wait_seed: u16,
    pub(crate) inner_wait_iterations: u16,
    pub(crate) outer_loop_seed: u16,
    pub(crate) outer_loop_iterations: u16,
    pub(crate) next_outer_label: &'static str,
}

#[allow(non_snake_case)]
#[allow(dead_code)]
pub(crate) fn end_sequence_burst_low_priority_wait_loop_loc_1D252()
-> EndSequenceBurstLowPriorityWaitLoopLoc1D252 {
    // ASM loc_1D252 (kid.asm:32627-32630): hibernate once per inner-loop
    // body, let `dbf d1,loc_1D252` consume the `d1 = 1` seed from
    // `loc_1D24A`, then let `dbf d0,loc_1D24A` consume the `d0 = $32` seed
    // prepared immediately before `loc_1D24A`.
    EndSequenceBurstLowPriorityWaitLoopLoc1D252 {
        hibernate_subroutine: "j_Hibernate_Object_1Frame",
        inner_wait_seed: 0x0001,
        inner_wait_iterations: 0x0002,
        outer_loop_seed: 0x0032,
        outer_loop_iterations: 0x0033,
        next_outer_label: "loc_1D24A",
    }
}

#[allow(non_snake_case)]
#[allow(dead_code)]
pub(crate) fn prize_10000_points_star_spawn_loc_10B3A(
    frame_counter: u16,
    x_pos: u16,
    y_pos: u16,
) -> Option<Prize10000PointsStarSpawnLoc10B3A> {
    // ASM loc_10B3A (kid.asm:22414): `unk_10A7A` only emits one star every
    // fifth wake, using indices 0..4 for `d3=0,5,10,15,20`; each emitted star
    // gets palette line 1, display flag `$13(a1)`, world position `$6C/$70(a5)`,
    // fixed-point velocities from `unk_10BF4` (`swap/clr/asr.l #3`), `st $14(a1)`,
    // and countdown `$46(a1)=#$78`. `d3=0` alone uses `LnkTo_unk_E1056`.
    const SPAWN_INDEX_BY_FRAME: [i8; 21] = [
        0, -1, -1, -1, -1, 1, -1, -1, -1, -1, 2, -1, -1, -1, -1, 3, -1, -1, -1, -1, 4,
    ];
    const VELOCITY_WORDS: [(i16, i16); 5] = [
        (-0x0010, -0x0014),
        (-0x0008, -0x0014),
        (0x0000, -0x0014),
        (0x0008, -0x0014),
        (0x0010, -0x0014),
    ];

    let spawn_index = *SPAWN_INDEX_BY_FRAME.get(usize::from(frame_counter))?;
    if spawn_index < 0 {
        return None;
    }
    let spawn_index = spawn_index as usize;
    let (x_velocity_word, y_velocity_word) = VELOCITY_WORDS[spawn_index];
    Some(Prize10000PointsStarSpawnLoc10B3A {
        palette_line: 1,
        display_enabled: true,
        sprite_mapping_label: if frame_counter == 0 {
            "LnkTo_unk_E1056-Data_Index"
        } else {
            "LnkTo_unk_E104E-Data_Index"
        },
        x_pos,
        y_pos,
        x_velocity_fp: i32::from(x_velocity_word) << 13,
        y_velocity_fp: i32::from(y_velocity_word) << 13,
        is_moved: true,
        timer: 0x0078,
    })
}

#[allow(non_snake_case)]
#[allow(dead_code)]
pub(crate) fn prize_10000_points_star_dispatch_loc_10B76(
    slot_offset: u16,
    slot_present: bool,
) -> Prize10000PointsStarDispatchLoc10B76 {
    // ASM loc_10B76 (kid.asm:22436): load `$44(a5,d2.w)` and skip empty star
    // slots with `beq.w loc_10BCE`; populated slots stay on the current `d2`
    // so the timer branch can execute first.
    if slot_present {
        Prize10000PointsStarDispatchLoc10B76 {
            should_step_slot: true,
            next_slot_offset: slot_offset,
            finished_scan: false,
        }
    } else {
        let advance = prize_10000_points_star_advance_loc_10BCE(slot_offset);
        Prize10000PointsStarDispatchLoc10B76 {
            should_step_slot: false,
            next_slot_offset: advance.next_slot_offset,
            finished_scan: advance.finished_scan,
        }
    }
}

#[allow(non_snake_case)]
#[allow(dead_code)]
pub(crate) fn prize_10000_points_star_advance_loc_10BCE(
    slot_offset: u16,
) -> Prize10000PointsStarAdvanceLoc10BCE {
    // ASM loc_10BCE (kid.asm:22472): `addq.w #4,d2` walks the five longword
    // star slots, and `cmpi.w #$14,d2` stops the scan after the fifth entry.
    let next_slot_offset = slot_offset.wrapping_add(4);
    Prize10000PointsStarAdvanceLoc10BCE {
        next_slot_offset,
        finished_scan: next_slot_offset == 0x0014,
    }
}

#[allow(non_snake_case)]
#[allow(dead_code)]
pub(crate) fn prize_10000_points_star_cleanup_loc_10BE6(
    worker_slots: [u32; 5],
) -> Prize10000PointsStarCleanupLoc10BE6 {
    // ASM loc_10BE6 (kid.asm:22484): seed `a2 = $44(a5)` and `d0 = #4`, then
    // run `move.l (a2)+,a3; jsr (j_loc_1078).w; dbf d0,loc_10BE6` so all five
    // cached star gfx slots are visited before the parent worker jumps to
    // `Delete_CurrentObject`.
    Prize10000PointsStarCleanupLoc10BE6 {
        visited_slots: worker_slots,
        should_delete_current_object: true,
    }
}

#[allow(non_snake_case)]
#[allow(dead_code)]
pub(crate) fn prize_10000_points_star_latch_screen_pos_loc_10B9A(
    x_pos: i16,
    y_pos: i16,
    camera_x_pos: i16,
    camera_y_pos: i16,
) -> (i16, i16) {
    // ASM loc_10B9A (kid.asm:22452): copy `$1A/$1E(a1) - Camera_X/Y_pos`
    // into `$3E/$42(a1)` before the shared left-drift path at `loc_10BB2`.
    (
        x_pos.wrapping_sub(camera_x_pos),
        y_pos.wrapping_sub(camera_y_pos),
    )
}

#[allow(non_snake_case)]
#[allow(dead_code)]
pub(crate) fn prize_10000_points_star_step_loc_10B74(
    slot_offset: u16,
    timer: u16,
    x_pos: i16,
    y_pos: i16,
    camera_x_pos: i16,
    camera_y_pos: i16,
    is_moved: bool,
) -> Prize10000PointsStarStepLoc10B74 {
    // ASM loc_10B74 (kid.asm:22433): iterate the five cached star slots and
    // branch by `$46(a1)`. `#$70` clears `$14(a1)` once before the shared
    // loc_10B98/loc_10BCA countdown tail. `#$20` enters `loc_10B9A`, which
    // latches screen-space `$3E/$42` from world position minus camera before
    // falling into loc_10BB2. Timers `<= #$20` move left by 6 px/frame, and
    // the final slot (`d2 == #$10`) jumps to loc_10BE0 once its updated
    // screen X falls below `#$FFE0` (-32), skipping the countdown decrement on
    // that deletion frame.
    let mut next_x_pos = x_pos;
    let mut next_is_moved = is_moved;
    let mut next_timer = timer;
    let mut latched_screen_pos = None;
    let mut delete_prize_worker = false;

    if timer == 0x0020 {
        latched_screen_pos = Some(prize_10000_points_star_latch_screen_pos_loc_10B9A(
            x_pos,
            y_pos,
            camera_x_pos,
            camera_y_pos,
        ));
    }

    if timer <= 0x0020 {
        next_x_pos = next_x_pos.wrapping_sub(6);
        if slot_offset == 0x0010 {
            let updated_screen_x = next_x_pos.wrapping_sub(camera_x_pos);
            if updated_screen_x < -0x0020 {
                delete_prize_worker = true;
            }
        }
    } else if timer == 0x0070 {
        next_is_moved = false;
    }

    if !delete_prize_worker {
        next_timer = next_timer.wrapping_sub(1);
    }

    Prize10000PointsStarStepLoc10B74 {
        latched_screen_pos,
        x_pos: next_x_pos,
        is_moved: next_is_moved,
        timer: next_timer,
        delete_prize_worker,
    }
}

impl NativeRuntime {
    fn activate_intro_sequence_3_roster(&mut self) {
        self.active_intro_sequence_3_roster_index = self.next_intro_sequence_3_roster_index;
        self.next_intro_sequence_3_roster_index = (self.next_intro_sequence_3_roster_index + 1)
            % PlayerState::intro_sequence_3_roster_count();
    }

    pub(crate) fn enter_title_screen(&mut self) {
        self.boot_state = BootState::TitleScreen { cursor: 0 };
        self.title_idle_frames = 0;
        self.title_screen_elapsed_frames = 0;
        self.title_screen_uses_intro_return_hidden_shell = false;
        self.title_screen_ready_frame = if self.frame_index < TITLE_SCREEN_READY_GLOBAL_FRAME {
            self.frame_index
                .saturating_add(TITLE_SCREEN_EARLY_ENTRY_LOCAL_READY_FRAMES)
                .max(TITLE_SCREEN_READY_GLOBAL_FRAME)
        } else {
            self.frame_index
        };
        self.title_menu_arm_frame = if self.frame_index < TITLE_SCREEN_READY_GLOBAL_FRAME {
            if self.level_has_title_card(self.current_level) {
                TITLE_SCREEN_MENU_ARM_GLOBAL_FRAME
            } else {
                // Title-card-less test manifests use the shorter hidden title
                // shell, so Obj_TitleMenu can accept the immediate early-start
                // hand-off once `sub_1C246 -> loc_1C250` reaches title.
                self.frame_index
            }
        } else {
            self.frame_index
                .saturating_add(TITLE_TO_LEVEL_FRAMES.saturating_sub(1))
        };
        self.title_menu_active = false;
        self.title_menu_cursor_pending = false;
        self.title_menu_selection_pending = None;
        self.title_menu_selection_delay_frames = 0;
        self.demo_buffered_input = None;
        self.pending_demo_title_return = 0;
        self.demo_title_return_after_frame = false;
        self.demo_auto_timeout_return_pending = false;
        self.demo_auto_timeout_freeze_gameplay_frame = false;
        self.demo_auto_timeout_handoff_active = false;
        self.demo_auto_timeout_intro_sequence_1_frames_override = None;
        self.demo_auto_timeout_intro_sequence_2_frames_override = None;
        self.demo_auto_timeout_intro_sequence_3_frames_override = None;
        self.demo_auto_timeout_intro_sequence_4_frames_override = None;
        self.demo_auto_timeout_intro_sequence_5_frames_override = None;
        self.timeout_restart_title_intro_sequence_3_return_pending = false;
        self.timeout_restart_title_intro_sequence_4_return_pending = false;
        self.timeout_restart_title_intro_sequence_5_return_pending = false;
        self.preserve_demo_title_snapshot = false;
        self.timeout_restart_title_intro_sequence_3_pose = false;
        self.timeout_restart_title_intro_sequence_4_camera_carry = false;
        self.timeout_restart_title_intro_sequence_5_pose = false;
        self.timeout_restart_title_intro_sequence_5_uses_intro_return_hidden_delay = false;
        self.timeout_restart_intro_sequence_4_pose_carry = false;
        self.hide_demo_return_title_player = false;
        self.ignore_demo_start_abort_until_gameplay = false;
        self.next_demo_uses_start_skip_intro_return_camera_wrap = false;
        self.demo_uses_start_skip_intro_return_camera_wrap = false;
        self.next_demo_uses_demo_title_return_camera_wrap = false;
        self.demo_uses_demo_title_return_camera_wrap = false;
        self.demo_uses_intro_sequence_6_hidden_camera_wrap = false;
        self.demo_uses_intro_sequence_6_early_title_restore = false;
        self.timeout_restart_intro_sequence_1_uses_early_title_restore = false;
        self.demo_uses_intro_sequence_6_later_demo_2_preload_row = false;
        self.demo_title_return_uses_intro_sequence_6_early_title_restore = false;
        self.demo_timeout_restart_uses_intro_return_slot_2_early_visibility = false;
        self.demo_timeout_restart_uses_start_skip_intro_sequence_5_delay = false;
        self.demo_timeout_restart_uses_intro_sequence_6_early_intro_sequence_5_handoff = false;
        self.pending_prize_diamond_awards.clear();
    }

    fn enter_title_screen_from_intro_return(
        &mut self,
        intro_sequence_1_uses_early_title_restore: bool,
    ) {
        self.enter_title_screen();
        self.title_screen_uses_intro_return_hidden_shell = true;
        // ASM sub_1C246 writes `Game_Mode=$2C` immediately, but the later
        // Load_IntroSequence6 / loc_1BCD0 title rebuild still needs the local
        // settle window before the standard title pose appears.
        // ASM loc_1C264 (kid.asm:31132): both `Game_Mode=$24` and `$28`
        // Start-skip branches land on `move.w #$2C,(Game_Mode).w`, set the
        // intro-sequence flag at `$FFFFFBCE`, stop music, and jump to loc_6E2.
        let mut ready_frame = self
            .frame_index
            .saturating_add(TITLE_SCREEN_EARLY_ENTRY_LOCAL_READY_FRAMES);
        if intro_sequence_1_uses_early_title_restore {
            // ASM loc_1B5CE had already restored the standard title pose
            // before this timeout-restart demo timed out back into intro
            // sequence 1. When Start skips that later intro-1 revisit back to
            // title, oracle ends the local rebuild two frames early and
            // re-exposes the same pose on local title frames 148 and 149.
            ready_frame = ready_frame
                .saturating_sub(DEMO_AUTO_TIMEOUT_INTRO_SEQUENCE_6_EARLY_TITLE_RESTORE_FRAMES);
            // ASM sub_1C246 already returned to `Game_Mode=$2C`, so once that
            // rebuilt loc_1B5CE pose reappears two title frames early this
            // lineage is also two frames farther through Obj_TitleMenu
            // `loc_1BAEA`'s shared `$280` DBF idle before `loc_1BB02`
            // flips back into demo play.
            self.title_idle_frames = DEMO_AUTO_TIMEOUT_INTRO_SEQUENCE_6_EARLY_TITLE_RESTORE_FRAMES
                .try_into()
                .expect("two-frame intro-sequence-1 title restore offset should fit in u16");
        }
        self.title_screen_ready_frame = ready_frame;
    }

    fn enter_intro_sequence_6_from_timeout_restart_intro_sequence_5(&mut self) {
        let carry_timeout_restart_hidden_camera_wrap =
            !self.demo_timeout_restart_uses_intro_sequence_6_early_intro_sequence_5_handoff;
        self.enter_title_screen();
        self.boot_state = BootState::IntroSequence6 {
            cursor: 0,
            carry_intro_sequence_5_slot_2_pose: true,
            carry_timeout_restart_hidden_camera_wrap,
        };
        // ASM loc_1BA5C: `move.w #$28,(Game_Mode).w` hands control to
        // Load_IntroSequence6 rather than the direct `sub_1C246 -> $2C`
        // title return. Keep the carried loc_1BCD0 slot-2 pose visible while
        // this timeout-restart lineage rebuilds the title shell.
        self.title_screen_ready_frame = self
            .frame_index
            .saturating_add(TITLE_SCREEN_EARLY_ENTRY_LOCAL_READY_FRAMES);
        // ASM Load_IntroSequence6 `loc_1BAB0` runs a separate `$B8` `dbf`
        // loop before it ever reaches Obj_TitleMenu `loc_1BAEA`'s shared
        // `$280` title idle countdown. Delay this lineage's title arm until
        // that 185-frame pre-roll has elapsed.
        self.title_menu_arm_frame = self
            .frame_index
            .saturating_add(INTRO_SEQUENCE_6_TITLE_ATTRACT_LOCK_FRAMES);
    }

    pub(crate) fn title_screen_ready(&self) -> bool {
        self.frame_index >= self.title_screen_ready_frame
    }

    pub(crate) fn title_screen_rng_long_burn_active(&self) -> bool {
        // ASM `sub_1C246` can hand native into `Game_Mode=$002C` before the
        // oracle-visible title shell actually reaches `Obj_TitleMenu`.
        // Oracle WRAM probes for the cold-boot start-to-level route still see
        // one final pre-freeze long burn on native-local title frame 59, then
        // keep `RNG_Offset` frozen from frame 60 through 115 before the later
        // hidden title rebuild resumes the per-frame
        // `Execute_Objects -> Get_RandomNumber_long` cadence.
        !(60..=115).contains(&self.title_screen_elapsed_frames)
    }

    pub(crate) fn hidden_title_screen_object_counts(&self) -> Option<(u16, u16)> {
        if !matches!(self.boot_state, BootState::TitleScreen { .. })
            || self.title_screen_ready()
            || !self.title_screen_uses_intro_return_hidden_shell
        {
            return None;
        }

        // ASM `sub_1C246 -> Load_TitleScreen -> Obj_TitleMenu` rebuilds the
        // hidden title shell in stages on the cold-boot intro-return lineage.
        // Oracle gameplay windows for the early-title Start route show the
        // carried 2/1 shell on local frames 0..26, a blank rebuild window on
        // 27..46, then the 3/0, 5/1, 5/2, and steady 4/2 shell before
        // `loc_1BB52` exposes the menu cursor on local frame 58.
        Some(match self.title_screen_elapsed_frames {
            0..=26 => (2, 1),
            27..=46 => (0, 0),
            47 => (3, 0),
            48 => (5, 1),
            49 => (5, 2),
            _ => (4, 2),
        })
    }

    pub(crate) fn demo_startup_frame(&self) -> Option<u16> {
        match self.boot_state {
            BootState::DemoPlay { frame_counter, .. } => Some(frame_counter),
            _ => None,
        }
    }

    pub(crate) fn demo_uses_second_recording(&self) -> Option<bool> {
        match self.boot_state {
            BootState::DemoPlay { use_demo_2, .. } => Some(use_demo_2),
            _ => None,
        }
    }

    pub(crate) fn demo_gameplay_active(&self) -> bool {
        self.demo_startup_frame()
            .zip(self.demo_uses_second_recording())
            .is_some_and(|(frame, use_demo_2)| frame >= self.demo_gameplay_start_frame(use_demo_2))
    }

    pub(crate) fn demo_gameplay_start_frame(&self, use_demo_2: bool) -> u16 {
        let mut frame = demo::gameplay_start_frame(use_demo_2);
        if !use_demo_2 && self.demo_uses_intro_sequence_6_hidden_camera_wrap {
            // ASM `loc_1BB02 -> Load_DemoPlay -> Load_InGame` still runs on the
            // timeout-restart `IntroSequence6` lineage, but oracle keeps the
            // first demo-1 bootstrap frame hidden for one extra startup frame
            // after the wrapped `$FD20_0000` preload row.
            frame = frame.saturating_add(1);
        }
        frame
    }

    pub(crate) fn demo_level_id(use_demo_2: bool) -> LevelId {
        // ASM Load_DemoPlay `not.b ($FFFFFBC8).w; beq.w loc_119B6` keeps `d7`
        // at Knights Isle for demo 1 and overwrites it with Skydragon Castle 1
        // for demo 2 before the shared `loc_119B6` tail stores
        // `(Current_LevelID).w`.
        LevelId(if use_demo_2 {
            demo::DEMO_2_LEVEL
        } else {
            demo::DEMO_1_LEVEL
        })
    }

    fn demo_timeout_counter(frame_counter: u16, use_demo_2: bool) -> u16 {
        if use_demo_2 {
            frame_counter.saturating_sub(demo::DEMO_2_TIMEOUT_COUNTER_OFFSET_FRAMES)
        } else {
            frame_counter.saturating_sub(demo::DEMO_1_TIMEOUT_COUNTER_OFFSET_FRAMES)
        }
    }

    pub(crate) fn demo_title_full_snapshot_frames(&self) -> usize {
        if self.demo_auto_timeout_handoff_active {
            DEMO_AUTO_TIMEOUT_FULL_SNAPSHOT_FRAMES
        } else {
            DEMO_TITLE_FULL_SNAPSHOT_FRAMES
        }
    }

    pub(crate) fn demo_title_partial_snapshot_frames(&self) -> usize {
        if self.demo_auto_timeout_handoff_active {
            DEMO_AUTO_TIMEOUT_PARTIAL_SNAPSHOT_FRAMES
        } else {
            DEMO_TITLE_PARTIAL_SNAPSHOT_FRAMES
        }
    }

    fn begin_demo_play(&mut self, use_demo_2: bool) {
        let timeout_restart_demo_bootstrap = self.timeout_restart_title_intro_sequence_3_pose;
        let start_skip_intro_return_camera_wrap =
            self.next_demo_uses_start_skip_intro_return_camera_wrap;
        let demo_title_return_camera_wrap =
            !use_demo_2 && self.next_demo_uses_demo_title_return_camera_wrap;
        let title_screen_uses_timeout_restart_intro_sequence_4_title_restore = matches!(
            self.boot_state,
            // ASM Obj_TitleMenu `loc_1BAEA -> loc_1BB02` can also hand a demo
            // off from the later intro-sequence-4 title return after
            // `loc_1B5CE` has already restored the standard title pose.
            BootState::TitleScreen { .. }
        ) && self
            .timeout_restart_title_intro_sequence_4_camera_carry;
        let intro_sequence_6_hidden_camera_wrap = matches!(
            self.boot_state,
            // ASM loc_1BA5C enters Load_IntroSequence6, and the later
            // Obj_TitleMenu `loc_1BB02 -> Load_DemoPlay -> Load_InGame`
            // hand-off preserves a distinct demo-1 hidden camera wrap on the
            // timeout-restart lineage that came from intro sequence 5.
            BootState::IntroSequence6 {
                carry_intro_sequence_5_slot_2_pose: true,
                ..
            }
        );
        let intro_sequence_6_later_demo_2_preload_row = matches!(
            self.boot_state,
            // ASM `loc_1BB02 -> Load_DemoPlay -> Load_InGame` keeps the later
            // demo-2 preload row only on the revisit that still carries
            // `loc_1BCD0`'s slot-2 pose but no longer reuses the earlier
            // timeout-restart hidden-camera wrap.
            BootState::IntroSequence6 {
                carry_intro_sequence_5_slot_2_pose: true,
                carry_timeout_restart_hidden_camera_wrap: false,
                ..
            }
        );
        let intro_sequence_6_early_title_restore = intro_sequence_6_hidden_camera_wrap
            || title_screen_uses_timeout_restart_intro_sequence_4_title_restore
            || self.demo_title_return_uses_intro_sequence_6_early_title_restore;
        self.boot_state = BootState::DemoPlay {
            frame_counter: 0,
            data_index: 0,
            use_demo_2,
        };
        self.current_level = LevelId(0);
        self.level_start_frame = None;
        self.opening_slice = None;
        self.active_enemies.clear();
        self.killed_enemies.clear();
        self.active_enemy_roster.clear();
        self.active_platforms.clear();
        self.active_platform_roster.clear();
        self.projectiles.clear();
        self.pending_prize_diamond_awards.clear();
        self.player_lifecycle = PlayerLifecycle::Alive;
        self.progression.current_form = PlayerForm::TheKid;
        self.progression.pending_form = None;
        self.progression.number_hitpoints = PlayerForm::TheKid.max_hitpoints();
        self.demo_buffered_input = None;
        self.pending_demo_title_return = 0;
        self.demo_title_return_after_frame = false;
        self.demo_auto_timeout_return_pending = false;
        self.demo_auto_timeout_freeze_gameplay_frame = false;
        self.demo_auto_timeout_handoff_active = false;
        self.demo_auto_timeout_intro_sequence_1_frames_override = None;
        self.demo_auto_timeout_intro_sequence_2_frames_override = None;
        self.demo_auto_timeout_intro_sequence_3_frames_override = None;
        self.demo_auto_timeout_intro_sequence_4_frames_override = None;
        self.demo_auto_timeout_intro_sequence_5_frames_override = None;
        self.timeout_restart_title_intro_sequence_5_return_pending = false;
        self.timeout_restart_title_intro_sequence_3_pose = false;
        self.timeout_restart_title_intro_sequence_5_pose = false;
        self.timeout_restart_intro_sequence_4_pose_carry = false;
        self.hide_demo_return_title_player = false;
        self.ignore_demo_start_abort_until_gameplay = timeout_restart_demo_bootstrap;
        self.demo_uses_start_skip_intro_return_camera_wrap = start_skip_intro_return_camera_wrap;
        self.demo_uses_demo_title_return_camera_wrap = demo_title_return_camera_wrap;
        self.demo_uses_intro_sequence_6_hidden_camera_wrap = intro_sequence_6_hidden_camera_wrap;
        self.demo_uses_intro_sequence_6_early_title_restore = intro_sequence_6_early_title_restore;
        self.timeout_restart_intro_sequence_1_uses_early_title_restore = false;
        self.demo_uses_intro_sequence_6_later_demo_2_preload_row =
            intro_sequence_6_later_demo_2_preload_row;
        // ASM loc_1BCD0 still owns the later mode-$24 slot-2 pose after the
        // timeout-restore title shell, so this lineage bit must survive the
        // intermediate title screen before the next demo auto-timeout revisit.
        self.demo_timeout_restart_uses_intro_return_slot_2_early_visibility =
            start_skip_intro_return_camera_wrap || intro_sequence_6_early_title_restore;
        self.next_demo_uses_start_skip_intro_return_camera_wrap = false;
        self.next_demo_uses_demo_title_return_camera_wrap = false;
        self.reload_block_grid_for_level(self.current_level);
    }

    pub(crate) fn finish_demo_title_return(&mut self, input: FrameInput) {
        let intro_sequence_6_early_title_restore =
            self.demo_uses_intro_sequence_6_early_title_restore;
        self.enter_title_screen();
        self.current_level = LevelId(0);
        // Demo abort returns to an inert title rebuild path rather than the
        // already-ready cold-boot title state. Oracle traces keep Kid hidden
        // until that attract-lock window elapses.
        self.title_screen_ready_frame = self.frame_index.saturating_add(TITLE_ATTRACT_LOCK_FRAMES);
        self.paused = true;
        self.pause_toggle_pending = false;
        self.preserve_demo_title_snapshot = true;
        self.hide_demo_return_title_player = true;
        self.next_demo_uses_demo_title_return_camera_wrap = true;
        self.demo_title_return_uses_intro_sequence_6_early_title_restore =
            intro_sequence_6_early_title_restore;
        if intro_sequence_6_early_title_restore {
            // ASM loc_1B5CE has already restored the standard title pose
            // before the later IntroSequence6 demo starts. When the published
            // TAS aborts that later demo with Start, oracle re-enters the
            // shared Obj_TitleMenu loc_1BAEA idle loop two title frames ahead
            // of the generic demo-title return, so seed those already-burned
            // idle frames here before loc_1BB02 flips back into demo play.
            self.title_idle_frames = DEMO_AUTO_TIMEOUT_INTRO_SEQUENCE_6_EARLY_TITLE_RESTORE_FRAMES
                .try_into()
                .expect("two-frame IntroSequence6 title restore offset should fit in u16");
        }
        self.player_lifecycle = PlayerLifecycle::Alive;
        self.prior_input = input;
        self.previous_input = input;
    }

    pub(crate) fn finish_demo_auto_timeout_return(&mut self) {
        let intro_return_slot_2_early_visibility =
            self.demo_timeout_restart_uses_intro_return_slot_2_early_visibility;
        let start_skip_intro_sequence_5_delay = self.demo_uses_start_skip_intro_return_camera_wrap;
        let intro_sequence_6_early_intro_sequence_5_handoff =
            self.demo_uses_intro_sequence_6_early_title_restore;
        let intro_sequence_6_early_title_restore =
            self.demo_uses_intro_sequence_6_early_title_restore;
        self.enter_title_screen();
        self.current_level = LevelId(0);
        self.paused = false;
        self.pause_toggle_pending = false;
        self.demo_auto_timeout_freeze_gameplay_frame = false;
        self.demo_auto_timeout_handoff_active = true;
        self.demo_auto_timeout_intro_sequence_1_frames_override =
            Some(DEMO_AUTO_TIMEOUT_INTRO_FIRST_SEGMENT_FRAMES);
        self.demo_auto_timeout_intro_sequence_2_frames_override =
            Some(DEMO_AUTO_TIMEOUT_INTRO_SECOND_SEGMENT_FRAMES);
        self.demo_auto_timeout_intro_sequence_3_frames_override =
            Some(DEMO_AUTO_TIMEOUT_INTRO_THIRD_SEGMENT_FRAMES);
        self.demo_auto_timeout_intro_sequence_4_frames_override =
            Some(DEMO_AUTO_TIMEOUT_INTRO_FOURTH_SEGMENT_FRAMES);
        self.demo_auto_timeout_intro_sequence_5_frames_override =
            Some(DEMO_AUTO_TIMEOUT_INTRO_FIFTH_SEGMENT_FRAMES);
        self.preserve_demo_title_snapshot = true;
        self.hide_demo_return_title_player = true;
        self.player_lifecycle = PlayerLifecycle::Alive;
        self.demo_title_return_uses_intro_sequence_6_early_title_restore =
            intro_sequence_6_early_title_restore;
        self.timeout_restart_intro_sequence_1_uses_early_title_restore =
            intro_sequence_6_early_title_restore;
        self.demo_timeout_restart_uses_intro_return_slot_2_early_visibility =
            intro_return_slot_2_early_visibility;
        self.demo_timeout_restart_uses_start_skip_intro_sequence_5_delay =
            start_skip_intro_sequence_5_delay;
        self.demo_timeout_restart_uses_intro_sequence_6_early_intro_sequence_5_handoff =
            intro_sequence_6_early_intro_sequence_5_handoff;
    }

    fn finish_timeout_restart_intro_sequence_3_title_return(&mut self) {
        self.enter_title_screen();
        self.timeout_restart_title_intro_sequence_3_return_pending = false;
        self.hide_demo_return_title_player = true;
        self.timeout_restart_title_intro_sequence_3_pose = true;
        // Oracle frame 6906 shows the timeout-restart rebuild handing straight
        // back into demo playback one frame earlier than the cold-boot title
        // lock. Keep the shared title choreography, but shorten only this
        // timeout-restart attract window by one frame so the $280 title idle
        // countdown reaches demo mode on the correct frame.
        self.title_menu_arm_frame = self.title_menu_arm_frame.saturating_sub(1);
    }

    fn finish_timeout_restart_intro_sequence_4_title_return(&mut self) {
        self.enter_title_screen_from_intro_return(false);
        self.timeout_restart_title_intro_sequence_4_return_pending = false;
        self.timeout_restart_title_intro_sequence_4_camera_carry = true;
        // ASM sub_1C246 already switched back to `Game_Mode=$2C`; on this
        // later intro-sequence-4 revisit oracle exposes the rebuilt
        // `loc_1B5CE` title pose one title frame earlier than the generic
        // intro-return settle window.
        self.title_screen_ready_frame = self
            .title_screen_ready_frame
            .saturating_sub(TIMEOUT_RESTART_TITLE_INTRO_SEQUENCE_4_EARLY_READY_FRAMES);
        // ASM Obj_TitleMenu `loc_1BAEA` / `loc_1BB02` own the later title idle
        // countdown after `sub_1C246` returns. Once the rebuilt `loc_1B5CE`
        // pose surfaces a frame early on this lineage, oracle also reaches
        // the final `$2C -> $10` attract hand-off one title frame earlier.
        self.title_menu_arm_frame = self
            .title_menu_arm_frame
            .saturating_sub(TIMEOUT_RESTART_TITLE_INTRO_SEQUENCE_4_EARLY_ATTRACT_LOCK_FRAMES);
    }

    fn finish_timeout_restart_intro_sequence_5_title_return(&mut self) {
        let intro_return_hidden_delay =
            self.demo_timeout_restart_uses_intro_return_slot_2_early_visibility;
        self.enter_title_screen();
        self.timeout_restart_title_intro_sequence_5_return_pending = false;
        self.hide_demo_return_title_player = true;
        self.timeout_restart_title_intro_sequence_5_pose = true;
        self.timeout_restart_title_intro_sequence_5_uses_intro_return_hidden_delay =
            intro_return_hidden_delay;
        let carried_title_wait_frames = if intro_return_hidden_delay { 1 } else { 2 };
        // ASM sub_1C246 returns to title after the shared 2-frame Start
        // deferral, but the later Start-skip intro-return lineage reaches
        // Obj_TitleMenu `loc_1BAEA` one rebuild frame later than the plain
        // timeout-restart path. Only one title pre-wait frame has already
        // elapsed by the time the later lineage becomes visible, so keep that
        // lineage one frame farther from Obj_TitleMenu `loc_1BB02`'s
        // `move.w #$10,(Game_Mode).w` flip.
        self.title_menu_arm_frame = self
            .title_menu_arm_frame
            .saturating_sub(carried_title_wait_frames);
    }

    fn finish_demo_title_snapshot_handoff(&mut self) {
        self.preserve_demo_title_snapshot = false;
        self.opening_slice = None;
        self.active_enemies.clear();
        self.killed_enemies.clear();
        self.active_enemy_roster.clear();
        self.active_platforms.clear();
        self.active_platform_roster.clear();
        self.projectiles.clear();
        self.pending_prize_diamond_awards.clear();
        self.enemy_death_effects.clear();
        self.block_bounces.clear();
        self.block_destroy_effects.clear();
        self.floating_items.clear();
        self.progression.current_form = PlayerForm::TheKid;
        self.progression.pending_form = None;
        self.progression.number_hitpoints = PlayerForm::TheKid.max_hitpoints();
        self.progression.form_change_flash_frames = 0;
        self.progression.form_width_delta = 0;
        self.progression.invulnerable_frames = 0;
        self.progression.diamond_power_frames = 0;
        self.progression.diamond_count = 0;
        self.paused = false;
        self.pause_toggle_pending = false;
        self.player_lifecycle = PlayerLifecycle::Alive;
        self.level_damage_taken = false;
        self.level_prize_collected = false;
        self.clocks_collected = 0;
        self.hundred_k_trip_timer = 0;
        self.y_look_offset = 0;
    }

    pub(crate) fn begin_demo_gameplay(&mut self, level_id: LevelId) {
        self.current_level = level_id;
        self.level_start_frame = Some(self.frame_index);
        self.reload_block_grid_for_level(level_id);
        let mut opening_slice = self.level_respawn_slice(level_id);
        if self.block_grid.is_some() {
            let settled_slice = self.level_respawn_slice_with_floor_snap(level_id);
            opening_slice.terrain.floor_y_pos = settled_slice.player.y_pos;
        }
        self.opening_slice = Some(opening_slice);
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
        self.init_murder_wall(level_id, 0);
        self.init_weather(level_id);
        self.level_timer = LevelTimer::new();
        self.level_damage_taken = false;
        self.level_prize_collected = false;
        self.clocks_collected = 0;
        self.hundred_k_trip_timer = 0;
        self.y_look_offset = 0;
        self.paused = false;
        self.pause_toggle_pending = false;
    }

    pub(crate) fn apply_demo_bootstrap_initial_settle(&mut self) {
        let Some(slice) = self.opening_slice.as_mut() else {
            return;
        };
        // Load_InGame leaves the demo player one pixel above the later
        // grounded hand-off pose, then the hidden bootstrap drops that pixel.
        // on_ground=false starts the freefall phase (frames 67-69) where
        // gravity-trajectory position updates happen without touching y_vel.
        slice.player.y_pos = slice.player.y_pos.saturating_add(0x0001_0000);
        slice.player.on_ground = false;
        slice.player.y_velocity = 0;
        slice.update_camera_follow(0);
    }

    /// During frames between initial_settle and grounded_hold, the ASM's demo
    /// bootstrap applies gravity-trajectory position updates without touching
    /// the kid object's y_vel field. The kid object is hibernating — sub_B168
    /// (gravity) and sub_A4EE (position) don't run. Instead, the boot code
    /// directly adjusts y_pos with accumulated gravity: Δy = frame * 0x4000.
    pub(crate) fn apply_demo_bootstrap_freefall(&mut self, frame_counter: u16) {
        let Some(slice) = self.opening_slice.as_mut() else {
            return;
        };
        let freefall_frame = u32::from(frame_counter - demo::DEMO_BOOTSTRAP_INIT_FRAME);
        let shadow_vel = freefall_frame * 0x4000;
        slice.player.y_pos = slice.player.y_pos.wrapping_add(shadow_vel);
        // y_vel stays 0 — the kid object's velocity field is not used
        slice.player.y_velocity = 0;
        slice.update_camera_follow(0);
    }

    pub(crate) fn apply_demo_bootstrap_grounded_hold(&mut self) {
        let Some(slice) = self.opening_slice.as_mut() else {
            return;
        };
        // Oracle-backed demo traces keep the player on a short grounded
        // bootstrap hold after a 3-frame free-fall hand-off from the hidden
        // settle. Camera follow keeps converging during the hold.
        slice.player.y_pos = slice
            .player
            .y_pos
            .saturating_sub(slice.player.y_pos & 0x0000_FFFF);
        slice.player.y_velocity = 0;
        slice.player.on_ground = true;
        slice.player.on_spikes = false;
        slice.terrain.floor_y_pos = slice.player.y_pos;
        slice.update_camera_follow(0);
    }

    pub(crate) fn try_apply_demo_bootstrap_grounded_run(
        &mut self,
        input: FrameInput,
        jump_pressed: bool,
    ) -> bool {
        let Some(frame_counter) = self.demo_startup_frame() else {
            return false;
        };
        if frame_counter <= demo::DEMO_OBJECT_BOOTSTRAP_END_FRAME || jump_pressed || input.b {
            return false;
        }

        let Some(slice) = self.opening_slice.as_mut() else {
            return false;
        };
        if !slice.player.on_ground || slice.player.y_velocity != 0 {
            return false;
        }

        let Some(grid) = self.block_grid.as_ref() else {
            return false;
        };
        let x_px = (slice.player.x_pos >> 16) as i32;
        let y_px = (slice.player.y_pos >> 16) as i32;
        let hw = self.progression.current_form.half_width();
        let support_probe = terrain::probe_floor_hitbox(grid, x_px, y_px, hw);
        if support_probe.on_ground || support_probe.cell.is_solid_floor() {
            return false;
        }

        // ASM sub_B43A floor check: scan cells at (y_pos + 1) / 16 across the
        // player's width. If any Solid/Spikes cell found, floor still exists
        // (edge case the hitbox probe missed). If no floor at all, this is a
        // real walkoff — let normal AsmFaithful physics handle it (loc_B5F6).
        let probe_row = (((y_px + 1) >> 4).max(0)) as u16;
        let left_col = (((x_px - hw) >> 4).max(0)) as u16;
        let right_col = (((x_px + hw) >> 4).max(0).min(grid.width_cells as i32 - 1)) as u16;
        let mut b43a_has_floor = false;
        for col in left_col..=right_col {
            let cell = grid.cell_at(col, probe_row);
            if matches!(
                cell.collision,
                terrain::CollisionKind::Solid | terrain::CollisionKind::Spikes
            ) {
                b43a_has_floor = true;
                break;
            }
        }
        if !b43a_has_floor {
            // Real walkoff: let AsmFaithful physics handle via step_with_order.
            return false;
        }

        // Demo playback keeps the actor grounded on an invisible hand-off path
        // after the bootstrap hold, then regular physics resumes once the
        // recorded jump lifts off from that pinned floor.
        slice.update_horizontal_motion(input, false, self.progression.current_form, Some(grid));
        slice.apply_wall_collision(Some(grid), hw);
        slice.player.y_pos = slice.terrain.floor_y_pos;
        slice.player.y_velocity = 0;
        slice.player.on_ground = true;
        slice.player.on_spikes = false;
        slice.update_camera_follow(self.y_look_offset);
        true
    }

    pub(crate) fn apply_demo_bootstrap_camera_handoff(&mut self) {
        let Some(slice) = self.opening_slice.as_mut() else {
            return;
        };
        // The first visible demo gameplay frame keeps the held bootstrap
        // player pose but lets camera follow advance once from the respawn
        // preload position before the trace is sampled.
        slice.update_camera_follow(0);
    }

    pub(crate) fn apply_demo_1_bootstrap_pose_hold(&mut self) {
        let Some(slice) = self.opening_slice.as_mut() else {
            return;
        };
        // Knights Isle demo startup exposes the respawn pose unchanged while
        // only the camera converges during the hidden warm-up window.
        slice.player.y_velocity = 0;
        slice.update_camera_follow(0);
    }

    pub(crate) fn try_snap_demo_bootstrap_floor(&self, slice: &mut OpeningSliceState) -> bool {
        let Some(frame_counter) = self.demo_startup_frame() else {
            return false;
        };
        if !(demo::DEMO_BOOTSTRAP_GROUNDED_FRAME..=demo::DEMO_OBJECT_BOOTSTRAP_END_FRAME)
            .contains(&frame_counter)
            || slice.player.y_velocity < 0
            || self.block_grid.is_none()
        {
            return false;
        }

        let settled_slice = self.level_respawn_slice_with_floor_snap(self.current_level);
        let settled_floor_y = settled_slice.player.y_pos;
        if slice.player.y_pos.abs_diff(settled_floor_y) > GROUND_SNAP_TOLERANCE_FP {
            return false;
        }

        slice.player.y_pos = settled_floor_y;
        slice.player.y_velocity = 0;
        slice.player.on_ground = true;
        slice.player.on_spikes = false;
        slice.terrain.floor_y_pos = settled_floor_y;
        slice.terrain.floor_terrain_type = settled_slice.terrain.floor_terrain_type;
        true
    }

    pub(crate) fn boot_phase_label(&self) -> Option<&'static str> {
        match self.boot_state {
            BootState::SegaScreen { .. } => Some("sega_screen"),
            BootState::IntroSequence { segment: 0, .. } => Some("intro_sequence_1"),
            BootState::IntroSequence { segment: 1, .. } => Some("intro_sequence_2"),
            BootState::IntroSequence { segment: 2, .. } => Some("intro_sequence_3"),
            BootState::IntroSequence { segment: 3, .. } => Some("intro_sequence_4"),
            BootState::IntroSequence { .. } => Some("intro_sequence_5"),
            BootState::IntroSequence6 { .. } => Some("intro_sequence_6"),
            BootState::TitleScreen { .. } => Some("title_screen"),
            BootState::OptionsMenu { .. } => Some("options_menu"),
            BootState::SoundTest { .. } => Some("sound_test"),
            BootState::StartingLevel { .. } => Some("starting_level"),
            BootState::InGame
                if self.level_start_frame == Some(self.frame_index)
                    && !self.starting_level_title_workers.is_empty() =>
            {
                Some("starting_level")
            }
            BootState::LevelSkipHold => Some("level_skip_hold"),
            BootState::DemoPlay { .. } => Some("demo_play"),
            BootState::EndingSequence { .. } => Some("ending_sequence"),
            BootState::InGame => None,
        }
    }

    pub(crate) fn boot_phase_elapsed_frames(&self) -> Option<usize> {
        match self.boot_state {
            BootState::SegaScreen { elapsed } => Some(elapsed),
            BootState::IntroSequence { elapsed, .. } => Some(elapsed),
            BootState::IntroSequence6 { .. }
            | BootState::TitleScreen { .. }
            | BootState::OptionsMenu { .. }
            | BootState::SoundTest { .. } => Some(self.title_screen_elapsed_frames),
            BootState::StartingLevel { .. } => None,
            BootState::DemoPlay { frame_counter, .. } => Some(frame_counter as usize),
            BootState::EndingSequence { elapsed } => Some(elapsed),
            BootState::LevelSkipHold | BootState::InGame => None,
        }
    }

    /// Handle demo playback input override and exit conditions.
    /// Returns `Some(input)` with the (possibly overridden) input to continue,
    /// or `None` if the frame was fully handled (early return with trace).
    #[allow(
        clippy::unnecessary_wraps,
        reason = "Option return used by test callers"
    )]
    pub(crate) fn step_demo_input(
        &mut self,
        input: FrameInput,
        replay_timeout_handoff_frame: bool,
    ) -> Option<FrameInput> {
        let Some((frame_counter, data_index, use_demo_2)) = (match self.boot_state {
            BootState::DemoPlay {
                frame_counter,
                data_index,
                use_demo_2,
            } => Some((frame_counter, data_index, use_demo_2)),
            _ => None,
        }) else {
            return Some(input);
        };

        if self.pending_demo_title_return > 0 {
            self.pending_demo_title_return -= 1;
            if self.pending_demo_title_return == 0 {
                self.demo_title_return_after_frame = true;
            }
        }

        let gameplay_start_frame = self.demo_gameplay_start_frame(use_demo_2);
        if self.ignore_demo_start_abort_until_gameplay && frame_counter >= gameplay_start_frame {
            self.ignore_demo_start_abort_until_gameplay = false;
        }
        // ASM loc_119D8 clears Ctrl_Pressed, hibernates, then checks Start on
        // wake-up. The timeout-restart title rebuild feeds a queued Start bit
        // into the first restarted demo frames, but oracle ignores that input
        // until demo gameplay is live again. Keep manual demo aborts active
        // everywhere else, including cold-boot demo starts.
        let manual_abort_locked =
            self.ignore_demo_start_abort_until_gameplay && frame_counter < gameplay_start_frame;
        let transform_hibernating = self.gameplay_hibernating_for_transform();
        if input.start
            && !self.previous_input.start
            && self.pending_demo_title_return == 0
            && !manual_abort_locked
            && !transform_hibernating
        {
            // ASM loc_119D8 polls Start on the demo controller object, but
            // Kid_Transform replaces the regular gameplay/object loop with a
            // hibernating coroutine. Oracle-backed published TAS traces keep
            // the later demo-2 transform window in `Game_Mode=$10` through
            // frame 35148 even though Start is pressed on 35146, so suppress
            // demo abort edges while Kid_Transform still owns the frame.
            self.pending_demo_title_return = 2;
        }
        if replay_timeout_handoff_frame {
            // ORIGINAL_QUIRK: `ReadJoypad` flips demo 1 into mode 0 after the
            // frame's gameplay/object work has already consumed the previously
            // buffered demo input. Replay that final buffered step, but do not
            // advance the demo stream again on the visible handoff frame.
            return Some(self.demo_buffered_input.unwrap_or_default());
        }
        // ASM loc_C0A (kid.asm:1380): demo counter ($FFFFFBC2) reaches $3F2 →
        // `move.w #$82A,($FFFFFBCC); clr.w Current_LevelID; jsr StopMusic;
        // jmp j_loc_6E2`.  Native defers the title-return to
        // `finish_demo_auto_timeout_return` via `demo_auto_timeout_return_pending`.
        if Self::demo_timeout_counter(frame_counter, use_demo_2) >= demo::DEMO_MAX_FRAMES {
            self.demo_auto_timeout_return_pending = true;
            self.demo_auto_timeout_freeze_gameplay_frame = use_demo_2;
            return Some(input);
        }
        let level_id = Self::demo_level_id(use_demo_2);
        if frame_counter == demo::DEMO_LEVEL_SWITCH_FRAME {
            self.current_level = level_id;
            self.reload_block_grid_for_level(level_id);
        }
        if frame_counter >= demo::DEMO_OBJECT_WARMUP_FRAME && self.opening_slice.is_none() {
            self.begin_demo_gameplay(level_id);
        }
        if frame_counter + 1 < gameplay_start_frame {
            if let BootState::DemoPlay { frame_counter, .. } = &mut self.boot_state {
                *frame_counter += 1;
            }
            return Some(FrameInput::default());
        }

        let output_input = if frame_counter >= gameplay_start_frame {
            self.demo_buffered_input.unwrap_or_default()
        } else {
            FrameInput::default()
        };
        // Read demo byte and decode into input for the NEXT frame. ASM loc_C22
        // runs after object/gameplay logic, so the controller state fetched on
        // the camera-handoff frame is first consumed on the hidden settle frame.
        let byte = demo::demo_byte(use_demo_2, data_index);
        let demo_input = demo::decode_demo_byte(byte);
        // ASM demo input (loc_C22) freezes once the queued transform frame
        // hands control to Kid_Transform. The pending_form frame itself still
        // consumes the previously buffered byte, then the coroutine hibernates
        // until flash frames begin. Transform-exit parity is restored
        // separately in update_demo_transform_parity().
        if self.progression.pending_form.is_none() && self.progression.form_change_flash_frames == 0
        {
            self.demo_buffered_input = Some(demo_input);
            // Advance data pointer every 2 frames (ASM: `andi.w #1,d7; bne`)
            if let BootState::DemoPlay {
                frame_counter,
                data_index,
                ..
            } = &mut self.boot_state
            {
                let playback_frame = *frame_counter - (gameplay_start_frame - 1);
                *frame_counter += 1;
                if playback_frame % 2 == 1 {
                    *data_index += 1;
                }
            }
        }
        Some(output_input)
    }

    /// Process Start button presses: pause/unpause, level skip, demo trigger.
    ///
    /// ASM loc_74B0 (kid.asm:7623–7637): the main game loop's Start handler.
    ///
    /// ```asm
    /// loc_74B0:
    ///   bclr  #Button_Start,(Ctrl_Pressed).w   ; consume Start edge
    ///   beq.s loc_7452                          ; not pressed → loop
    ///   tst.b (LevelSkip_Cheat).w
    ///   beq.s loc_74C8                          ; no cheat → skip level skip
    ///   btst  #Button_C,(Ctrl_Held).w
    ///   bne.w loc_74E0                          ; C held + cheat → level skip
    /// loc_74C8:
    ///   btst  #Button_A,(Ctrl_Held).w
    ///   bne.s loc_7452                          ; A held → suppress pause
    ///   tst.b (Allow_Pausing).w
    ///   beq.w loc_7452                          ; pausing disabled → skip
    ///   st    (Game_Paused).w                   ; PAUSE
    ///   bra.w loc_7452
    /// ```
    ///
    /// The `LevelSkip_Cheat` flag is armed by ReadJoypad's loc_B98
    /// (kid.asm:1346): after storing per-player controller state,
    /// `tst.w (word_7190); cmpi.b #$C0,(Ctrl_1_Held); cmpi.b #$10,
    /// (Ctrl_2_Held)` — if word_7190 (debug mode) is set AND P1 holds
    /// A+B ($C0) AND P2 holds Right ($10), loc_BB2 (kid.asm:1355):
    /// `st (LevelSkip_Cheat)` sets the flag to $FF.
    /// Native: the cheat is always armed (no word_7190 gate).
    ///
    /// Native maps `bclr #Button_Start` to the `start_pressed` rising-edge
    /// parameter.  The level-skip cheat path uses `input.a && input.c`
    /// (A+C+Start).  The `btst #Button_A` pause suppression is implicit:
    /// when A is held, the branch falls into either level-skip or Diamond
    /// Power paths, never reaching the pause toggle.
    pub(crate) fn handle_start_press(&mut self, input: FrameInput, start_pressed: bool) {
        if !start_pressed {
            return;
        }
        self.start_press_count += 1;
        if matches!(self.player_lifecycle, PlayerLifecycle::Alive) {
            match self.boot_state {
                BootState::InGame => {
                    // ASM loc_74E0: LevelSkip_Cheat + C held → level skip
                    if input.a
                        && input.c
                        && (!self.level_skip_resume_hold
                            || self.level_skip_resume_elapsed_frames()
                                == Some(LEVEL_SKIP_RESUME_TOTAL_FRAMES - 1))
                    {
                        self.pending_level_skip = Some(self.next_level_id());
                    } else if input.a
                        && !input.c
                        && self.progression.current_form != PlayerForm::Eyeclops
                        && self.progression.diamond_count >= 20
                    {
                        // Diamond Power activation suppresses pause toggle
                    } else if self.paused {
                        // ASM loc_74B0 path: unpausing (Game_Paused already set)
                        self.paused = false;
                        self.pause_toggle_pending = false;
                    } else {
                        // ASM loc_74B0: st (Game_Paused).w — pause the game
                        self.pause_toggle_pending = true;
                    }
                }
                BootState::LevelSkipHold if input.a && input.c => {
                    self.pending_level_skip_resume = true;
                }
                _ => {}
            }
        }
    }

    /// Advance the boot state machine one frame (Sega screen, intros, title,
    /// options, sound test, level transition, ending).
    pub(crate) fn step_boot_state(&mut self, input: FrameInput, start_pressed: bool) {
        let intro_sequence_1_frames_override =
            self.demo_auto_timeout_intro_sequence_1_frames_override;
        let intro_sequence_2_frames_override =
            self.demo_auto_timeout_intro_sequence_2_frames_override;
        let intro_sequence_3_frames_override =
            self.demo_auto_timeout_intro_sequence_3_frames_override;
        let intro_sequence_4_frames_override =
            self.demo_auto_timeout_intro_sequence_4_frames_override;
        let intro_sequence_5_frames_override =
            self.demo_auto_timeout_intro_sequence_5_frames_override;
        let mut activate_intro_sequence_3_roster = false;
        let mut play_navigate_jingle = false;
        match &mut self.boot_state {
            BootState::SegaScreen { elapsed } => {
                *elapsed += 1;
                if start_pressed || *elapsed >= SEGA_SCREEN_FRAMES {
                    // ASM loc_12E96 -> loc_12EDE: once `bclr #7,(Ctrl_1_Pressed).w`
                    // sees Start, the Sega logo loop branches straight to
                    // `move.w #4,(Game_Mode).w` on that same frame.
                    self.boot_state = BootState::IntroSequence {
                        segment: 0,
                        elapsed: 0,
                        pending_title_skip: 0,
                    };
                }
            }
            // ASM has 5 intro sequences (Load_IntroSequence1-5) plus
            // Load_IntroSequence6 (title screen). Each loads unique VDP art.
            BootState::IntroSequence {
                segment,
                elapsed,
                pending_title_skip,
            } => {
                // ASM sub_1B8F4 / sub_1B93E / sub_1B988 / loc_1BA28 all call
                // sub_1C246 from their per-frame hibernate loops. The Start
                // edge is consumed there on the following outer frame, so the
                // visible mode flip is deferred by one traced frame, not two.
                if *pending_title_skip > 0 {
                    *pending_title_skip -= 1;
                    if *pending_title_skip == 0 {
                        if self.timeout_restart_title_intro_sequence_3_return_pending {
                            self.finish_timeout_restart_intro_sequence_3_title_return();
                        } else if self.timeout_restart_title_intro_sequence_4_return_pending {
                            self.finish_timeout_restart_intro_sequence_4_title_return();
                        } else if self.timeout_restart_title_intro_sequence_5_return_pending {
                            self.finish_timeout_restart_intro_sequence_5_title_return();
                        } else {
                            let intro_sequence_1_uses_early_title_restore = *segment == 0
                                && self.timeout_restart_intro_sequence_1_uses_early_title_restore;
                            if intro_sequence_1_uses_early_title_restore {
                                self.enter_title_screen_from_intro_return(true);
                                self.next_demo_uses_start_skip_intro_return_camera_wrap = true;
                            } else if self.level_has_title_card(self.current_level) {
                                // ASM sub_1C246 -> loc_1C250 still returns the
                                // stock title-card levels to the slower hidden
                                // title rebuild before Obj_TitleMenu accepts
                                // the later frame-360 / frame-460 Start taps.
                                self.enter_title_screen_from_intro_return(false);
                                self.next_demo_uses_start_skip_intro_return_camera_wrap = true;
                            } else {
                                // ASM sub_1C246 -> loc_1C250 jumps to
                                // Load_TitleScreen on the cold-boot Start-skip
                                // path, but oracle exposes the rebuilt title
                                // cursor only after the short 58-frame hidden
                                // shell, not the longer demo intro-return
                                // settle window.
                                self.enter_title_screen();
                                self.title_screen_uses_intro_return_hidden_shell = true;
                                self.title_screen_ready_frame =
                                    self.frame_index.saturating_add(TITLE_ATTRACT_LOCK_FRAMES);
                                self.title_menu_arm_frame =
                                    self.frame_index.saturating_add(TITLE_ATTRACT_LOCK_FRAMES);
                                self.title_menu_active = true;
                                self.title_menu_cursor_pending = true;
                            }
                        }
                        return;
                    }
                    *elapsed += 1;
                    return;
                }
                *elapsed += 1;
                let mut segment_frames = match *segment {
                    0 => intro_sequence_1_frames_override.unwrap_or(INTRO_FIRST_SEGMENT_FRAMES),
                    1 => intro_sequence_2_frames_override.unwrap_or(INTRO_SEGMENT_FRAMES),
                    2 => intro_sequence_3_frames_override.unwrap_or(INTRO_SEGMENT_FRAMES),
                    3 => intro_sequence_4_frames_override.unwrap_or(INTRO_SEGMENT_FRAMES),
                    4 => intro_sequence_5_frames_override.unwrap_or(INTRO_SEGMENT_FRAMES),
                    _ => INTRO_SEGMENT_FRAMES,
                };
                if *segment == 4
                    && intro_sequence_5_frames_override.is_some()
                    && self
                        .demo_timeout_restart_uses_intro_sequence_6_early_intro_sequence_5_handoff
                {
                    // ASM loc_1BA5C still owns the later timeout edge on this
                    // lineage, but once IntroSequence6 has already restored
                    // the standard title pose before the next attract demo,
                    // oracle reaches the later `$24 -> $28` hand-off one frame
                    // earlier than the older Start-skip revisit.
                    segment_frames = segment_frames.saturating_sub(1);
                }
                if *segment == 4
                    && intro_sequence_5_frames_override.is_some()
                    && self.demo_timeout_restart_uses_start_skip_intro_sequence_5_delay
                {
                    // ASM loc_1BA28 / sub_1C204 keep the timeout-restart copy
                    // of intro sequence 5 hibernating before loc_1BA5C starts
                    // polling sub_1C246. Oracle's later Start-skip
                    // intro-return lineage still sits in `Game_Mode=$24` on
                    // frame 16649 and only lets the frame-16648 Start press
                    // land on sub_1C246's shared 2-frame deferral. Hold this
                    // lineage for one extra sub_1C204 fade window instead of
                    // expiring to title before that Start edge arrives.
                    segment_frames +=
                        DEMO_AUTO_TIMEOUT_INTRO_FIFTH_SEGMENT_START_SKIP_LINEAGE_EXTRA_FRAMES;
                }
                // ASM sub_1B93E / loc_1B96A naturally count intro sequence 3
                // down to `Game_Mode=$20` (intro sequence 4). Only ASM
                // sub_1C246 / loc_1C250 jumps straight to title here, and that
                // path is gated by a Start press.
                let timeout_restart_start_returns_to_title =
                    *segment == 2 && intro_sequence_3_frames_override.is_some();
                let timeout_restart_intro_sequence_4_title_return =
                    *segment == 3 && self.timeout_restart_intro_sequence_4_pose_carry;
                let timeout_restart_intro_sequence_5_title_return =
                    *segment == 4 && intro_sequence_5_frames_override.is_some();
                let start_advances_segment = start_pressed && *segment != 0;
                let start_jumps_to_title = start_pressed
                    && (*segment == 0
                        || (*segment == 1 && *elapsed >= INTRO_FIRST_SEGMENT_DIRECT_TITLE_FRAMES)
                        || *segment >= 3);
                if start_jumps_to_title {
                    self.timeout_restart_title_intro_sequence_3_return_pending = false;
                    self.timeout_restart_title_intro_sequence_4_return_pending =
                        timeout_restart_intro_sequence_4_title_return;
                    self.timeout_restart_title_intro_sequence_5_return_pending =
                        timeout_restart_intro_sequence_5_title_return;
                    if !timeout_restart_intro_sequence_4_title_return
                        && !timeout_restart_intro_sequence_5_title_return
                    {
                        self.timeout_restart_intro_sequence_4_pose_carry = false;
                    }
                    *pending_title_skip = 1;
                } else if start_advances_segment {
                    if *segment == 0 {
                        self.demo_auto_timeout_intro_sequence_1_frames_override = None;
                    } else if *segment == 1 {
                        self.demo_auto_timeout_intro_sequence_2_frames_override = None;
                    } else if *segment == 2 {
                        self.demo_auto_timeout_intro_sequence_3_frames_override = None;
                    } else if *segment == 3 {
                        self.demo_auto_timeout_intro_sequence_4_frames_override = None;
                    } else if *segment == 4 {
                        self.demo_auto_timeout_intro_sequence_5_frames_override = None;
                    }
                    if *segment >= 4 || timeout_restart_start_returns_to_title {
                        self.timeout_restart_title_intro_sequence_3_return_pending =
                            timeout_restart_start_returns_to_title;
                        self.timeout_restart_title_intro_sequence_4_return_pending = false;
                        self.timeout_restart_title_intro_sequence_5_return_pending =
                            timeout_restart_intro_sequence_5_title_return;
                        if !timeout_restart_intro_sequence_5_title_return {
                            self.timeout_restart_intro_sequence_4_pose_carry = false;
                        }
                        *pending_title_skip = 1;
                    } else {
                        self.timeout_restart_title_intro_sequence_3_return_pending = false;
                        self.timeout_restart_title_intro_sequence_4_return_pending = false;
                        self.timeout_restart_title_intro_sequence_5_return_pending = false;
                        self.timeout_restart_intro_sequence_4_pose_carry = false;
                        *segment += 1;
                        activate_intro_sequence_3_roster = *segment == 2;
                        *elapsed = 0;
                    }
                } else if *elapsed >= segment_frames {
                    // Natural timeout: advance immediately (no deferral)
                    if *segment == 0 {
                        self.demo_auto_timeout_intro_sequence_1_frames_override = None;
                    } else if *segment == 1 {
                        self.demo_auto_timeout_intro_sequence_2_frames_override = None;
                    } else if *segment == 2 {
                        self.demo_auto_timeout_intro_sequence_3_frames_override = None;
                    } else if *segment == 3 {
                        self.demo_auto_timeout_intro_sequence_4_frames_override = None;
                    } else if *segment == 4 {
                        self.demo_auto_timeout_intro_sequence_5_frames_override = None;
                    }
                    if *segment >= 4 {
                        let intro_sequence_5_timeout_restart =
                            *segment == 4 && intro_sequence_5_frames_override.is_some();
                        self.timeout_restart_title_intro_sequence_3_return_pending = false;
                        self.timeout_restart_title_intro_sequence_4_return_pending = false;
                        self.timeout_restart_title_intro_sequence_5_return_pending = false;
                        self.timeout_restart_intro_sequence_4_pose_carry = false;
                        if intro_sequence_5_timeout_restart {
                            self.enter_intro_sequence_6_from_timeout_restart_intro_sequence_5();
                        } else {
                            self.enter_title_screen_from_intro_return(false);
                        }
                    } else {
                        let carry_intro_sequence_3_pose_into_intro_sequence_4 =
                            *segment == 2 && intro_sequence_3_frames_override.is_some();
                        let carry_intro_sequence_4_camera_into_intro_sequence_5 =
                            *segment == 3 && self.timeout_restart_intro_sequence_4_pose_carry;
                        self.timeout_restart_title_intro_sequence_3_return_pending = false;
                        self.timeout_restart_title_intro_sequence_4_return_pending = false;
                        self.timeout_restart_title_intro_sequence_5_return_pending = false;
                        self.timeout_restart_intro_sequence_4_pose_carry =
                            carry_intro_sequence_3_pose_into_intro_sequence_4
                                || carry_intro_sequence_4_camera_into_intro_sequence_5;
                        *segment += 1;
                        activate_intro_sequence_3_roster = *segment == 2;
                        *elapsed = 0;
                    }
                }
            }
            BootState::IntroSequence6 {
                cursor,
                carry_timeout_restart_hidden_camera_wrap,
                ..
            } => {
                let mut just_activated_title_menu = false;
                if self.demo_auto_timeout_handoff_active
                    && self.title_screen_elapsed_frames + 1 >= DEMO_AUTO_TIMEOUT_MODE_0_FRAMES
                {
                    self.demo_auto_timeout_handoff_active = false;
                    self.timeout_restart_intro_sequence_4_pose_carry = false;
                    self.boot_state = BootState::IntroSequence {
                        segment: 0,
                        elapsed: 0,
                        pending_title_skip: 0,
                    };
                    return;
                }
                let up_pressed = input.up && !self.prior_input.up;
                let down_pressed = input.down && !self.prior_input.down;

                if self.title_menu_selection_pending.is_some()
                    && self.title_menu_selection_delay_frames > 0
                {
                    self.title_menu_selection_delay_frames -= 1;
                    self.title_screen_elapsed_frames += 1;
                    return;
                }
                if let Some(selection) = self.title_menu_selection_pending.take() {
                    self.title_idle_frames = 0;
                    self.title_menu_selection_delay_frames = 0;
                    match selection {
                        0 => {
                            self.two_player = false;
                            self.begin_level_entry(self.current_level);
                        }
                        1 => {
                            self.two_player = true;
                            self.begin_level_entry(self.current_level);
                        }
                        _ => {
                            self.boot_state = BootState::OptionsMenu { cursor: 0 };
                        }
                    }
                    self.title_screen_elapsed_frames += 1;
                    return;
                }
                if self.frame_index < self.title_menu_arm_frame {
                    // Oracle-backed imported TAS traces keep title fully inert for
                    // one attract-lock window after entry, even if Start was
                    // already being tapped during the intro skip path.
                } else if !self.title_menu_active {
                    if start_pressed {
                        self.title_idle_frames = 0;
                        self.title_menu_active = true;
                        self.title_menu_cursor_pending = true;
                        just_activated_title_menu = true;
                    } else {
                        self.title_idle_frames += 1;
                        if self.title_idle_frames >= INTRO_SEQUENCE_6_TITLE_IDLE_FRAMES {
                            self.title_idle_frames = 0;
                            self.demo_toggle = !self.demo_toggle;
                            self.begin_demo_play(self.demo_toggle);
                        }
                    }
                } else if start_pressed {
                    self.title_idle_frames = 0;
                    // ASM Obj_TitleMenu loc_1BBD6: after Start selects the
                    // current row, the object marks the hand-off latch and
                    // jumps through loc_6E2 / level-select entry.
                    self.title_menu_selection_pending = Some(*cursor);
                    self.title_menu_selection_delay_frames = TITLE_TO_LEVEL_SELECTION_DELAY_FRAMES;
                } else if up_pressed && *cursor > 0 {
                    // ASM Obj_TitleMenu loc_1BBDE: Up only moves when `d1`
                    // is non-zero, then decrements the row, calls sub_1BC26,
                    // and resets the title idle countdown.
                    self.title_idle_frames = 0;
                    *cursor -= 1;
                    play_navigate_jingle = true;
                } else if down_pressed && *cursor < 2 {
                    // ASM Obj_TitleMenu loc_1BBF6: Down only moves while
                    // `d1 != 2`, then calls sub_1BC26 before refreshing the
                    // cursor row from word_1BC20.
                    self.title_idle_frames = 0;
                    *cursor += 1;
                    play_navigate_jingle = true;
                } else {
                    let mut title_idle_frames = demo::TITLE_IDLE_FRAMES;
                    if !*carry_timeout_restart_hidden_camera_wrap {
                        // ASM Obj_TitleMenu `loc_1BAEE` keeps the later
                        // title-restore revisit polling inside the active menu
                        // for three extra local frames before `loc_1BB02`
                        // flips into demo play.
                        title_idle_frames +=
                            TIMEOUT_RESTART_INTRO_SEQUENCE_6_LATER_DEMO_HANDOFF_EXTRA_FRAMES;
                    }
                    self.title_idle_frames += 1;
                    if self.title_idle_frames >= title_idle_frames {
                        self.title_idle_frames = 0;
                        self.demo_toggle = !self.demo_toggle;
                        self.begin_demo_play(self.demo_toggle);
                    }
                }
                if self.title_menu_cursor_pending
                    && !just_activated_title_menu
                    && self.frame_index >= self.title_menu_arm_frame
                {
                    self.title_menu_cursor_pending = false;
                }
                self.title_screen_elapsed_frames += 1;
                if self.preserve_demo_title_snapshot
                    && self.title_screen_elapsed_frames >= self.demo_title_partial_snapshot_frames()
                {
                    self.finish_demo_title_snapshot_handoff();
                }
            }
            BootState::TitleScreen { cursor } => {
                let mut just_activated_title_menu = false;
                if self.demo_auto_timeout_handoff_active
                    && self.title_screen_elapsed_frames + 1 >= DEMO_AUTO_TIMEOUT_MODE_0_FRAMES
                {
                    self.demo_auto_timeout_handoff_active = false;
                    self.timeout_restart_intro_sequence_4_pose_carry = false;
                    self.boot_state = BootState::IntroSequence {
                        segment: 0,
                        elapsed: 0,
                        pending_title_skip: 0,
                    };
                    return;
                }
                let up_pressed = input.up && !self.prior_input.up;
                let down_pressed = input.down && !self.prior_input.down;

                if self.title_menu_selection_pending.is_some()
                    && self.title_menu_selection_delay_frames > 0
                {
                    self.title_menu_selection_delay_frames -= 1;
                    self.title_screen_elapsed_frames += 1;
                    return;
                }
                if let Some(selection) = self.title_menu_selection_pending.take() {
                    self.title_idle_frames = 0;
                    self.title_menu_selection_delay_frames = 0;
                    match selection {
                        0 => {
                            self.two_player = false;
                            self.begin_level_entry(self.current_level);
                        }
                        1 => {
                            self.two_player = true;
                            self.begin_level_entry(self.current_level);
                        }
                        _ => {
                            self.boot_state = BootState::OptionsMenu { cursor: 0 };
                        }
                    }
                    self.title_screen_elapsed_frames += 1;
                    return;
                }
                if self.frame_index < self.title_menu_arm_frame {
                    // Oracle-backed imported TAS traces keep title fully inert for
                    // one attract-lock window after entry, even if Start was
                    // already being tapped during the intro skip path.
                } else if !self.title_menu_active {
                    if start_pressed {
                        self.title_idle_frames = 0;
                        self.title_menu_active = true;
                        self.title_menu_cursor_pending = true;
                        just_activated_title_menu = true;
                    } else {
                        self.title_idle_frames += 1;
                        if self.title_idle_frames >= demo::TITLE_IDLE_FRAMES {
                            self.title_idle_frames = 0;
                            self.demo_toggle = !self.demo_toggle;
                            self.begin_demo_play(self.demo_toggle);
                        }
                    }
                } else if start_pressed {
                    self.title_idle_frames = 0;
                    // ASM Obj_TitleMenu loc_1BBD6: after Start selects the
                    // current row, the object marks the hand-off latch and
                    // jumps through loc_6E2 / level-select entry.
                    self.title_menu_selection_pending = Some(*cursor);
                    self.title_menu_selection_delay_frames = TITLE_TO_LEVEL_SELECTION_DELAY_FRAMES;
                } else if up_pressed && *cursor > 0 {
                    // ASM Obj_TitleMenu loc_1BBDE: Up only moves when `d1`
                    // is non-zero, then decrements the row, calls sub_1BC26,
                    // and resets the title idle countdown.
                    self.title_idle_frames = 0;
                    *cursor -= 1;
                    play_navigate_jingle = true;
                } else if down_pressed && *cursor < 2 {
                    // ASM Obj_TitleMenu loc_1BBF6: Down only moves while
                    // `d1 != 2`, then calls sub_1BC26 before refreshing the
                    // cursor row from word_1BC20.
                    self.title_idle_frames = 0;
                    *cursor += 1;
                    play_navigate_jingle = true;
                } else {
                    self.title_idle_frames += 1;
                    if self.title_idle_frames >= demo::TITLE_IDLE_FRAMES {
                        self.title_idle_frames = 0;
                        self.demo_toggle = !self.demo_toggle;
                        self.begin_demo_play(self.demo_toggle);
                    }
                }
                if self.title_menu_cursor_pending
                    && !just_activated_title_menu
                    && self.frame_index >= self.title_menu_arm_frame
                {
                    self.title_menu_cursor_pending = false;
                }
                self.title_screen_elapsed_frames += 1;
                if self.preserve_demo_title_snapshot
                    && self.title_screen_elapsed_frames >= self.demo_title_partial_snapshot_frames()
                {
                    self.finish_demo_title_snapshot_handoff();
                }
            }
            BootState::OptionsMenu { cursor } => {
                let b_pressed = input.b && !self.prior_input.b;
                let left_pressed = input.left && !self.prior_input.left;
                let right_pressed = input.right && !self.prior_input.right;
                // ASM loc_1CB88 (kid.asm:31684): loop entry that steps panel
                // animation (sub_1CC88) then checks Start to exit. Native
                // omits the intro-loop camera scroll and enters directly.
                (
                    self.options_menu_panel_toggle_byte,
                    self.options_menu_left_panel_x,
                    self.options_menu_right_panel_x,
                ) = step_option_menu_panels_sub_1CC88(
                    self.options_menu_panel_toggle_byte,
                    self.options_menu_panel_delta,
                    self.options_menu_left_panel_x,
                    self.options_menu_right_panel_x,
                );
                // ASM OptionScreen_Input: 4 items:
                // 0 = 2 PLAYERS (one/two controllers), 1 = CONTROLS (0..5),
                // 2 = SPEED BUTTON (normal/fast), 3 = EXIT.
                // ASM loc_1CBF2 (kid.asm:31727): branch target when Up not pressed
                // or cursor already at 0; continues to check Down below.
                if input.up && !self.prior_input.up && *cursor > 0 {
                    *cursor -= 1;
                }
                // ASM loc_1CC0E (kid.asm:31736): branch target when Down not pressed
                // or cursor already at 3; continues to check Left/Right below.
                if input.down && !self.prior_input.down && *cursor < 3 {
                    *cursor += 1;
                }

                // ASM loc_1CC22 (kid.asm:31744): Left button jump table dispatch.
                // After Left press check at loc_1CC0E, the ROM doubles the
                // cursor index and uses PC-relative indexed jump:
                //   move.w (Options_Selected_Option).w,d7
                //   add.w d7,d7
                //   jmp loc_1CC22(pc,d7.w)
                // The jump table at loc_1CC22 then branches to option handlers:
                //   cursor 0 → Option_2PInput
                //   cursor 1 → Option_Controls_Left
                //   cursor 2 → Option_SpeedButton
                //   cursor 3 → Option_Exit
                // ASM loc_1CC2A (kid.asm:31754): Right button handling entry.
                // If Left was not pressed, execution continues here and checks
                // Right. The ROM clears the Right button bit and dispatches:
                //   bclr #Button_Right,(Ctrl_Pressed).w
                //   beq.w return_1CC86         ; skip if Right not pressed
                //   move.w (Options_Selected_Option).w,d7
                //   add.w d7,d7
                //   jmp loc_1CC3E(pc,d7.w)
                // ASM loc_1CC3E (kid.asm:31761): Right button jump table dispatch.
                // Similar indexed jump table for Right, with one difference:
                //   cursor 0 → Option_2PInput
                //   cursor 1 → Option_Controls_Right (increment, not decrement)
                //   cursor 2 → Option_SpeedButton
                //   cursor 3 → Option_Exit
                // Native uses match on cursor for equivalent dispatch.
                match *cursor {
                    0 if left_pressed || right_pressed => {
                        // ASM Option_2PInput toggles `Options_Suboption_2PController`
                        // with either left or right.
                        self.options_two_player_two_controllers =
                            !self.options_two_player_two_controllers;
                        play_navigate_jingle = true;
                    }
                    1 if left_pressed => {
                        // ASM Option_Controls_Left decrements within 0..=5.
                        self.options_controls_scheme =
                            self.options_controls_scheme.saturating_sub(1);
                        play_navigate_jingle = true;
                    }
                    1 if right_pressed => {
                        // ASM Option_Controls_Right increments within 0..=5.
                        self.options_controls_scheme =
                            self.options_controls_scheme.saturating_add(1).min(5);
                        play_navigate_jingle = true;
                    }
                    2 if left_pressed || right_pressed => {
                        // ASM Option_SpeedButton toggles the fast-action flag
                        // with either left or right.
                        self.options_speed_button_fast = !self.options_speed_button_fast;
                        play_navigate_jingle = true;
                    }
                    _ => {}
                }

                if *cursor == 3 && (left_pressed || right_pressed) || start_pressed || b_pressed {
                    self.enter_title_screen();
                }
                self.title_screen_elapsed_frames += 1;
            }
            BootState::SoundTest { track } => {
                let b_pressed = input.b && !self.prior_input.b;
                let a_pressed = input.a && !self.prior_input.a;
                let c_pressed = input.c && !self.prior_input.c;
                if input.right && !self.prior_input.right {
                    *track = track.wrapping_add(1);
                }
                if input.left && !self.prior_input.left {
                    *track = track.wrapping_sub(1);
                }
                if input.up && !self.prior_input.up {
                    *track = track.wrapping_add(16);
                }
                if input.down && !self.prior_input.down {
                    *track = track.wrapping_sub(16);
                }
                // A/C plays the current track (handled by renderer via sound_test_track)
                // Start pressed flag is set on the trace for the audio layer to detect
                if a_pressed || c_pressed {
                    self.sound_test_play_pending = true;
                }
                if b_pressed || start_pressed {
                    self.boot_state = BootState::OptionsMenu { cursor: 1 };
                }
                self.title_screen_elapsed_frames += 1;
            }
            BootState::StartingLevel {
                start_latched,
                elapsed_frames,
                uses_load_ingame_bootstrap,
                load_ingame_bootstrap_started,
            } => {
                let mut enter_gameplay = false;
                let gameplay_input_active =
                    input.left || input.right || input.a || input.b || input.c;
                let title_card_bootstrap_active = (!*uses_load_ingame_bootstrap
                    || !*load_ingame_bootstrap_started)
                    && !self.starting_level_title_workers.is_empty();
                if *uses_load_ingame_bootstrap && !*load_ingame_bootstrap_started {
                    if *start_latched {
                        // ASM loc_1AE2E hibernates first, then consumes the
                        // later Start edge and jumps to loc_6E2 on the
                        // following traced frame. Keep that edge latched so the
                        // hidden Load_InGame bootstrap can later release into
                        // live gameplay without needing a third Start press.
                        // ASM loc_1AE2E -> loc_6E2 still carries the hidden
                        // loc_1AB26 / loc_1AD48 shell through the first traced
                        // mode-$000C frames, so keep those workers alive until
                        // live gameplay finishes consuming the carry window.
                        *load_ingame_bootstrap_started = true;
                        *elapsed_frames = 0;
                    } else if start_pressed {
                        *start_latched = true;
                    }
                    *elapsed_frames = elapsed_frames.saturating_add(1);
                    return;
                }
                let handoff_ready = if *uses_load_ingame_bootstrap {
                    *elapsed_frames >= EARLY_START_LOAD_INGAME_HANDOFF_FRAMES
                } else if title_card_bootstrap_active {
                    let active_input_handoff = gameplay_input_active
                        && *elapsed_frames
                            > crate::movement::LEVEL_ENTRY_TITLE_ACTIVE_INPUT_HANDOFF_FRAMES;
                    active_input_handoff
                        || *elapsed_frames
                            > crate::movement::LEVEL_ENTRY_TITLE_GAMEPLAY_HANDOFF_FRAMES + 1
                } else {
                    *elapsed_frames > 0
                };
                if handoff_ready {
                    // ASM loc_1AE2E: the title-card worker hibernates for one
                    // frame, then polls Ctrl_Pressed before it writes
                    // `Game_Mode=$000C` and jumps to `loc_6E2`. Oracle-backed
                    // boot traces keep the real title-card lineage on the
                    // hidden bootstrap far longer than the synthetic no-card
                    // path, so only release the queued Start edge once that
                    // title shell has finished its hidden pre-roll.
                    enter_gameplay = if title_card_bootstrap_active {
                        gameplay_input_active
                    } else {
                        *start_latched || start_pressed
                    };
                    *start_latched = false;
                }
                *elapsed_frames = elapsed_frames.saturating_add(1);
                if enter_gameplay {
                    if *uses_load_ingame_bootstrap {
                        // ASM loc_1AE2E -> loc_6E2 reloads the early
                        // no-title-card bootstrap through Load_InGame, so the
                        // hidden carry window must collapse back to the fresh
                        // settled respawn slice before live gameplay begins.
                        self.reset_current_level_runtime_state_for_respawn();
                        self.starting_level_title_workers.clear();
                    } else if title_card_bootstrap_active {
                        // ASM loc_1AE2E only gates the later visible
                        // `Game_Mode=$000C` write on the title-card path; the
                        // hidden mode-$0008 shell has already been ticking the
                        // real gameplay/object slice underneath. Keep that
                        // progressed slice and just discard the title workers
                        // once gameplay input makes the live state visible.
                        self.starting_level_title_workers.clear();
                    }
                    self.boot_state = BootState::InGame;
                    self.level_start_frame = Some(self.frame_index);
                }
            }
            BootState::LevelSkipHold => {}
            BootState::InGame => {}
            BootState::DemoPlay { .. } => {
                // Demo input feeding and frame counting handled in step_demo_input()
            }
            BootState::EndingSequence { elapsed } => {
                *elapsed += 1;
                let total_ending_frames = ENDING_PAUSE_FRAMES
                    + ENDING_MUSIC_DELAY_FRAMES
                    + ENDING_FADE_OUT_FRAMES
                    + CREDITS_SCREEN_COUNT * CREDITS_SCREEN_TOTAL;
                if *elapsed >= total_ending_frames {
                    // Return to title screen after credits
                    self.enter_title_screen();
                    self.player_lifecycle = PlayerLifecycle::Alive;
                    self.progression.number_lives = 3;
                    self.progression.number_continues = 3;
                    self.progression.diamond_count = 0;
                    self.progression.score = 0;
                    self.current_level = LevelId(0);
                    self.two_player = false;
                    self.current_player_index = 0;
                    self.player_slots = [PlayerSlot::default(), PlayerSlot::default()];
                }
            }
        }
        if play_navigate_jingle {
            let _ = self.play_navigate_jingle_sub_1BC26(0);
        }
        if activate_intro_sequence_3_roster {
            self.activate_intro_sequence_3_roster();
        }
    }

    pub(crate) fn maybe_begin_pending_level_skip(&mut self) {
        let Some(level_id) = self.pending_level_skip.take() else {
            return;
        };
        let (
            frozen_player,
            frozen_camera,
            frozen_active_objects,
            mut frozen_number_objects,
            mut frozen_number_gfx_objects,
        ) = if self.level_skip_resume_hold {
            let elapsed = self
                .level_skip_resume_elapsed_frames()
                .unwrap_or(0)
                .saturating_sub(1);
            let Some(frozen_player) = self
                .level_skip_resume_player_state_at(elapsed)
                .or(self.opening_slice.map(|slice| slice.player))
            else {
                self.enter_level_skip_hold(level_id);
                return;
            };
            let frozen_camera = self.level_skip_resume_camera_state_at(elapsed);
            let frozen_active_objects = self.level_skip_resume_active_objects_at(elapsed);
            let (frozen_number_objects, frozen_number_gfx_objects) =
                self.level_skip_resume_object_counts_at(elapsed);
            (
                frozen_player,
                frozen_camera,
                frozen_active_objects,
                frozen_number_objects,
                frozen_number_gfx_objects,
            )
        } else {
            let Some(frozen_player) = self
                .current_player_state()
                .or(self.opening_slice.map(|slice| slice.player))
            else {
                self.enter_level_skip_hold(level_id);
                return;
            };
            let frozen_camera = self.current_camera_state();
            let frozen_active_objects = self.active_objects();
            let (frozen_number_objects, frozen_number_gfx_objects) = self
                .object_counts_from_roster(
                    &frozen_active_objects,
                    frozen_camera.x_pos,
                    frozen_camera.y_pos,
                );
            (
                frozen_player,
                frozen_camera,
                frozen_active_objects,
                frozen_number_objects,
                frozen_number_gfx_objects,
            )
        };
        if self.current_level == LevelId(0) && level_id == LevelId(1) {
            frozen_number_objects = 4;
            frozen_number_gfx_objects = 5;
        }
        self.enter_level_skip_hold(level_id);
        self.level_skip_transition = Some(LevelSkipTransitionState {
            elapsed_frames: 0,
            frozen_player,
            frozen_camera,
            frozen_active_objects,
            frozen_number_objects,
            frozen_number_gfx_objects,
        });
    }

    pub(crate) fn maybe_begin_pending_level_skip_resume(&mut self) {
        if !self.pending_level_skip_resume {
            return;
        }
        self.pending_level_skip_resume = false;
        self.boot_state = BootState::InGame;
        self.level_start_frame = Some(self.frame_index);
        self.level_skip_transition = None;
        self.level_skip_resume_hold = true;
        self.opening_slice = Some(self.level_respawn_slice(self.current_level));
        self.active_enemy_roster.clear();
        self.active_enemies.clear();
        self.killed_enemies.clear();
        self.active_platforms.clear();
        self.active_platform_roster.clear();
        self.block_state.evanescent_blocks.clear();
        self.block_state.reset_shooter_state_sub_F06A();
        self.block_state.ghost_groups.clear();
        self.level_timer = LevelTimer::new();
        self.level_prize_collected = false;
        self.clocks_collected = 0;
        self.hundred_k_trip_timer = 0;
        self.paused = false;
        self.pause_toggle_pending = false;
    }

    pub(crate) fn maybe_finish_level_skip_resume(&mut self) {
        if self.level_skip_resume_elapsed_frames() != Some(LEVEL_SKIP_RESUME_TOTAL_FRAMES) {
            return;
        }
        self.level_skip_resume_hold = false;
        self.level_start_frame = Some(self.frame_index);
        self.opening_slice = Some(self.level_skip_resume_final_slice(self.current_level));
    }

    /// Returns true if the frame should continue to physics/gameplay processing.
    /// Returns false for early-return cases (non-alive lifecycle, paused, etc.).
    pub(crate) fn step_lifecycle(&mut self, input: FrameInput) -> bool {
        if matches!(self.boot_state, BootState::EndingSequence { .. }) {
            self.opening_slice = None;
            return false;
        }
        if !matches!(
            self.boot_state,
            BootState::StartingLevel { .. }
                | BootState::LevelSkipHold
                | BootState::InGame
                | BootState::DemoPlay { .. }
        ) {
            // ASM loc_65C (kid.asm:793): new-game init — clears Game_Mode,
            // Check_Helmet_Change, Demo_Mode_flag, Two_player_flag, resets
            // Player_1/2 lives=3, hitpoints=2, continues=3, score=0,
            // helmet=0 (TheKid), and sets $FFFFFC36 (timer-init flag).
            // The native equivalent resets progression and lifecycle state
            // every frame the boot machine hasn't yet entered gameplay.
            self.opening_slice = None;
            self.progression.current_form = PlayerForm::TheKid;
            self.progression.pending_form = None;
            self.progression.number_lives = 3;
            self.progression.number_continues = 3;
            self.progression.number_hitpoints = PlayerForm::TheKid.max_hitpoints();
            self.progression.invulnerable_frames = 0;
            self.paused = false;
            self.pause_toggle_pending = false;
            self.player_lifecycle = PlayerLifecycle::Alive;
            self.death_camera_lock = None;
            self.death_shell_snapshot = None;
            return false;
        }

        match self.player_lifecycle {
            PlayerLifecycle::Alive => {
                // ASM loc_BC5C/loc_BC6E (kid.asm:15203/15210): the damage-flash
                // thread hibernates 2+4 frames per cycle, 11 cycles = 66 frames.
                // Native models this as a simple countdown.
                if self.progression.invulnerable_frames > 0 {
                    self.progression.invulnerable_frames -= 1;
                }
                if self.progression.form_change_flash_frames > 0 {
                    self.progression.form_change_flash_frames -= 1;
                }
                self.resolve_pending_form_change();
            }
            PlayerLifecycle::DeathWindup { remaining_frames } => {
                if remaining_frames <= 1 {
                    self.enter_death_hidden_state();
                    return false;
                }
                let hidden_handoff_ready = self.advance_death_windup_motion();
                if hidden_handoff_ready {
                    self.enter_death_hidden_state();
                } else {
                    self.player_lifecycle = PlayerLifecycle::DeathWindup {
                        remaining_frames: remaining_frames - 1,
                    };
                }
                return false;
            }
            PlayerLifecycle::Dying { remaining_frames } => {
                if remaining_frames > 1 {
                    self.player_lifecycle = PlayerLifecycle::Dying {
                        remaining_frames: remaining_frames - 1,
                    };
                } else if self.progression.number_lives == 0 {
                    // ASM lose_life (kid.asm:14764): `subq.w #1,(Number_Lives).w;
                    // beq.w loc_D052` — when lives reach zero, branch to loc_D052
                    // (kid.asm:16325) which sets `$FFFFFB4B` (timer-frozen flag)
                    // via `st`, then enters the loc_D05A (kid.asm:16330) loop:
                    // iterate the object linked list clearing collision fields
                    // ($36/$3A/$3E longwords) for every object except `a5`
                    // (current), before falling through to loc_D074 which deletes
                    // all objects and sets up the score/game-over screen.
                    //
                    // loc_D074 hibernates $24 frames (for loc_C2F2 fade-out),
                    // then loc_D0AA (kid.asm:16356) iterates Addr_FirstGfxObjectSlot:
                    //   move.l d0,a3; _move.l 0(a3),d0; move.l d0,-(sp)
                    //   cmp.l (Addr_GfxObject_Kid).w,a3; beq.s loc_D0BC
                    //   jsr (j_loc_1078).w        — delete non-Kid GFX objects
                    // loc_D0BC (kid.asm:16364): move.l (sp)+,d0; bne.s loc_D0AA
                    //   — loop tail: pop saved next-pointer, continue if non-null.
                    //
                    // loc_D0C0 (kid.asm:16368) then sets up the score screen:
                    //   hibernate 1 frame, st $FFFFFB49, clr Time_Frames,
                    //   clr MurderWall_flag, allocate loc_C326 + loc_C428 objects,
                    //   decompress TtlCrdLetters to VRAM $8120 and unk_E06B5
                    //   to VRAM $A2E0, then palette copy loops loc_D11C/loc_D12E.
                    //
                    // Native: object collision state is not carried across
                    // lifecycle transitions, so the per-object clear is implicit.
                    // GFX object cleanup and palette loading are visual-only.
                    // ASM loc_D17E (kid.asm:16428) allocates the EOL score
                    // number object, copies `(Score).w` into object offset
                    // `$60`, then clears the live Score longword. Native does
                    // not model that visual object, so clear progression score
                    // at the collapsed GameOver entry.
                    self.progression.score = 0;
                    self.player_lifecycle = PlayerLifecycle::GameOver {
                        remaining_frames: PLAYER_GAME_OVER_HOLD_FRAMES,
                    };
                } else if self.try_switch_player() {
                    // 2P: both have lives, switch turns
                    self.player_lifecycle = PlayerLifecycle::Alive;
                } else {
                    self.reset_current_level_runtime_state_for_respawn();
                    self.player_lifecycle = PlayerLifecycle::Respawning {
                        remaining_frames: PLAYER_DEATH_RESPAWN_SETTLE_FRAMES,
                    };
                }
                return false;
            }
            PlayerLifecycle::Respawning { remaining_frames } => {
                if remaining_frames > 1 {
                    self.player_lifecycle = PlayerLifecycle::Respawning {
                        remaining_frames: remaining_frames - 1,
                    };
                } else {
                    self.player_lifecycle = PlayerLifecycle::Alive;
                    self.progression.invulnerable_frames = PLAYER_DAMAGE_INVULNERABLE_FRAMES;
                }
                return false;
            }
            PlayerLifecycle::LevelComplete { remaining_frames } => {
                if remaining_frames > 1 {
                    self.player_lifecycle = PlayerLifecycle::LevelComplete {
                        remaining_frames: remaining_frames - 1,
                    };
                } else {
                    self.advance_level();
                }
                return false;
            }
            PlayerLifecycle::GameOver { remaining_frames } => {
                // ASM loc_D25E (kid.asm:16472): after the "GAME OVER" text
                // setup, `tst.w (Number_Continues).w; bne.w Continue_Screen`
                // chooses the continue countdown when any continue remains;
                // otherwise the no-continues hold waits, then returns to
                // title via loc_D3F0.
                //
                // ASM loc_D2CE (kid.asm:16495): the no-continues wait loop
                // hibernates one frame, then `tst.b (Ctrl_Held).w; bmi.s
                // loc_D2DC`; bit 7 is Start, so held Start skips the timeout
                // and immediately falls through to the title-return path.
                if self.progression.number_continues == 0 && input.start {
                    self.return_to_title();
                    return false;
                }
                if remaining_frames > 1 {
                    self.player_lifecycle = PlayerLifecycle::GameOver {
                        remaining_frames: remaining_frames - 1,
                    };
                } else if self.progression.number_continues > 0 {
                    // Transition to continue countdown screen
                    self.player_lifecycle = PlayerLifecycle::ContinueCountdown {
                        remaining_frames: CONTINUE_COUNTDOWN_FRAMES,
                    };
                } else if self.try_switch_player() {
                    self.player_lifecycle = PlayerLifecycle::Alive;
                } else {
                    self.return_to_title();
                }
                return false;
            }
            PlayerLifecycle::ContinueCountdown { remaining_frames } => {
                // ASM loc_D350 (kid.asm:16528): the first CONTINUE? wait loop
                // hibernates one frame, polls `Ctrl_Held`, and branches to
                // Use_Continue as soon as Start sets bit 7. Native stores the
                // whole visual countdown as `remaining_frames` and keeps the
                // same input-first ordering before decrementing the timer.
                let any_button = input.a || input.b || input.c || input.start;
                if any_button {
                    // Use a continue: restore lives and respawn at current level.
                    // ASM Use_Continue: clears helmet, diamonds, resets to 3 lives / 2 HP.
                    self.progression.number_continues -= 1;
                    self.progression.number_lives = 3;
                    self.progression.number_hitpoints = PlayerForm::TheKid.max_hitpoints();
                    self.progression.current_form = PlayerForm::TheKid;
                    self.progression.pending_form = None;
                    self.progression.diamond_count = 0;
                    self.reset_current_level_runtime_state_for_respawn();
                    self.player_lifecycle = PlayerLifecycle::Respawning {
                        remaining_frames: PLAYER_DEATH_RESPAWN_SETTLE_FRAMES,
                    };
                } else if remaining_frames > 1 {
                    self.player_lifecycle = PlayerLifecycle::ContinueCountdown {
                        remaining_frames: remaining_frames - 1,
                    };
                } else {
                    // ASM loc_D386 (kid.asm:16546): after the countdown expires,
                    // the mandatory CONTINUE? text and number objects are kicked
                    // into the falling fade, then loc_D3AA waits before loc_D3F0
                    // returns to title. Native compresses that visual tail into
                    // this lifecycle expiry.
                    self.return_to_title();
                }
                return false;
            }
        }

        if matches!(self.boot_state, BootState::LevelSkipHold) {
            return false;
        }

        if self.level_skip_resume_hold {
            return false;
        }

        if self.paused {
            // ASM loc_687C (kid.asm:6581): pause-active path in sub_6874.
            //   bsr.w Pause_DarkenPalette    — darken palette each paused frame
            //   bsr.w Palette_to_VRAM        — upload darkened palette
            //   jsr   (sub_E1334).l          — run pause menu UI
            //   cmpi.b #3,(Game_Paused).w    — check unpause condition
            //   bne.s  loc_6894              — if not 3, continue to sprite hiding
            //   rts                          — if 3, return (unpausing)
            // The native port models this as a boolean `paused` gate: when true,
            // step_lifecycle returns false (suppressing gameplay), and the
            // rendering layer handles palette darkening separately.  Unpause
            // is handled by handle_start_press setting `paused = false` on the
            // next Start press (equivalent to Game_Paused transitioning to 3).
            return false;
        }

        true
    }
}

#[cfg(test)]
#[allow(non_snake_case)]
mod tests {
    use super::*;

    #[test]
    fn sub_1b652_seeds_the_fixed_title_text_burst_sprite() {
        let sprite = title_text_burst_sprite_sub_1B652();

        assert_eq!(sprite.gfx_slot_base, 0x0201_0000);
        assert_eq!(sprite.palette_line, 2);
        assert_eq!(sprite.x_pos, 0x002E);
        assert_eq!(sprite.y_pos, 0x0040);
        assert_eq!(
            sprite.vram_tile, 0x054C,
            "ASM sub_1B652 writes `move.w #$54C,vram_tile(a3)` before starting the burst animation"
        );
        assert!(sprite.is_moved);
        assert!(
            sprite.priority_flag,
            "ASM sub_1B652 sets both `is_moved(a3)` and `$13(a3)` on the allocated sprite"
        );
    }

    #[test]
    fn sub_1b652_uses_the_five_step_stru_1b68a_animation_script() {
        let sprite = title_text_burst_sprite_sub_1B652();

        assert_eq!(
            sprite.animation_frames,
            [
                TitleTextBurstAnimationFrameSub1B652 {
                    duration_frames: 4,
                    sprite_label: Some("unk_E11B6"),
                },
                TitleTextBurstAnimationFrameSub1B652 {
                    duration_frames: 4,
                    sprite_label: Some("unk_E11BE"),
                },
                TitleTextBurstAnimationFrameSub1B652 {
                    duration_frames: 100,
                    sprite_label: None,
                },
                TitleTextBurstAnimationFrameSub1B652 {
                    duration_frames: 4,
                    sprite_label: Some("unk_E11A6"),
                },
                TitleTextBurstAnimationFrameSub1B652 {
                    duration_frames: 4,
                    sprite_label: Some("unk_E11AE"),
                },
            ],
            "ASM stru_1B68A cycles through the two opening frames, a 100-frame blank hold, then the two closing frames"
        );
        assert_eq!(sprite.animation_command, 2);
        assert_eq!(
            sprite.animation_rewind_distance, 0x15,
            "ASM stru_1B68A ends with `dc.b 2, $15` after the five `anim_frame` entries"
        );
    }

    #[test]
    fn sub_1c54a_seeds_the_fixed_title_menu_prompt_sprite() {
        let sprite = title_menu_prompt_sprite_sub_1C54A();

        assert_eq!(sprite.gfx_slot_base, 0x0100_0000);
        assert!(
            sprite.priority_flag,
            "ASM sub_1C54A sets `$13(a1)` immediately after allocating the title prompt sprite",
        );
        assert_eq!(sprite.x_pos, 0x0081);
        assert_eq!(sprite.y_pos, 0x00C8);
        assert_eq!(
            sprite.sprite_label, "unk_E109E",
            "ASM sub_1C54A writes `#(LnkTo_unk_E109E-Data_Index)` into `$22(a1)`",
        );
        assert_eq!(
            sprite.vram_tile, 0x0140,
            "ASM sub_1C54A writes `move.w #$140,$24(a1)` for the prompt sprite tile base",
        );
    }

    #[test]
    fn sub_1c572_seeds_the_mirrored_title_menu_sliding_sprites() {
        let [left, right] = title_menu_sliding_sprites_sub_1C572();

        assert_eq!(left.gfx_slot_base, 0x0200_0004);
        assert_eq!(left.x_pos, 0x003A);
        assert_eq!(left.y_pos, 0x00C8);
        assert_eq!(left.sprite_label, "unk_E10A6");
        assert_eq!(left.vram_tile, 0x0140);
        assert_eq!(left.x_velocity_per_frame, 1);
        assert!(
            left.priority_flag,
            "ASM loc_1C5A4 sets `$13(a3)` for the left sliding title sprite",
        );

        assert_eq!(right.gfx_slot_base, 0x0200_0004);
        assert_eq!(right.x_pos, 0x00DE);
        assert_eq!(right.y_pos, 0x00C8);
        assert_eq!(right.sprite_label, "unk_E10AE");
        assert_eq!(right.vram_tile, 0x0140);
        assert_eq!(right.x_velocity_per_frame, -1);
        assert!(
            right.priority_flag,
            "ASM loc_1C5A4 sets `$13(a3)` for the right sliding title sprite after loc_1C58C",
        );
    }

    #[test]
    fn sub_1c572_hibernates_for_60_frames_then_moves_for_66_frames() {
        let [left, right] = title_menu_sliding_sprites_sub_1C572();

        assert_eq!(
            left.initial_hibernate_frames, 0x003C,
            "ASM sub_1C572 pushes `#$3C` before `j_Hibernate_Object` for the left sprite",
        );
        assert_eq!(
            right.initial_hibernate_frames, 0x003C,
            "ASM loc_1C58C shares the same 60-frame `Hibernate_Object` front delay",
        );
        assert_eq!(
            left.moving_frames, 0x0042,
            "ASM loc_1C5BE runs `dbf d1` from `#$41`, so the left sprite slides for 66 hibernated frames",
        );
        assert_eq!(
            right.moving_frames, 0x0042,
            "ASM loc_1C5BE also slides the right sprite for 66 hibernated frames before the infinite idle loop",
        );
    }

    #[test]
    fn loc_1_c5_ca_is_the_infinite_idle_loop_after_sliding() {
        // ASM loc_1C5CA (kid.asm:31332-31334): after `loc_1C5BE` completes the
        // 66-frame slide, the sprite enters an infinite idle state:
        //   loc_1C5CA:
        //     jsr (j_Hibernate_Object_1Frame).w
        //     bra.s loc_1C5CA
        // This is a terminal state - the sprite hibernates forever and never
        // moves again. Both left and right sliding title sprites reach this
        // state after their movement completes.
        let [left, right] = title_menu_sliding_sprites_sub_1C572();
        assert_eq!(left.moving_frames, 0x0042);
        assert_eq!(right.moving_frames, 0x0042);
    }

    #[test]
    fn loc_1_c5_e8_is_the_vdp_pattern_fill_row_loop() {
        // ASM sub_1C5D0 -> loc_1C5E8 (kid.asm:31353-31356): outer loop that sets
        // VRAM write address for each row in the pattern fill routine. Called from
        // intro/end screen initialization at lines 29295, 29404, 30197, 30438,
        // 30790, 30936. Each iteration:
        // 1. Copies d6 (row height in words) to d1 for inner loop counter
        // 2. Writes d0.long to VDP control port 4(a6) setting VRAM write address
        // 3. Adds $800000 to d0 advancing VRAM by 2048 bytes (VSRAM stride)
        let row_loop = vdp_pattern_fill_row_loop_loc_1C5E8();
        assert_eq!(
            row_loop.vram_address_step, 0x0080_0000,
            "ASM loc_1C5E8 adds `#$800000` to the VRAM address between rows, advancing by 2048 bytes",
        );
    }

    #[test]
    fn loc_1_c5_f4_is_the_vdp_pattern_fill_word_loop() {
        // ASM sub_1C5D0 -> loc_1C5F4 (kid.asm:31358-31360): inner DBF loop that
        // copies one row of pattern data to VDP. Each iteration does
        // `move.w (a4)+,(a6)` copying a word from source buffer (a4) to VDP data
        // port (a6). The loop runs d1+1 times where d1 is the row height in words.
        // After this row completes, control returns to loc_1C5E8 for the next row.
        let word_loop = vdp_pattern_fill_word_loop_loc_1C5F4();
        assert_eq!(
            word_loop.source_register, "a4",
            "ASM loc_1C5F4 reads from source buffer via `(a4)+`",
        );
        assert_eq!(
            word_loop.dest_register, "a6",
            "ASM loc_1C5F4 writes to VDP data port via `(a6)`",
        );
    }

    #[test]
    fn loc_1bb10_allocates_the_intro_text_sprite() {
        let sprite = title_text_intro_sprite_loc_1BB10();

        assert_eq!(
            sprite.gfx_slot_base, 0x0200_0000,
            "ASM loc_1BB10 loads `#$2000000` into a3 before the gfx slot allocation",
        );
        assert!(
            sprite.visibility_flag,
            "ASM loc_1BB10 uses `st $13(a3)` to set the visibility flag",
        );
        assert_eq!(
            sprite.palette_line, 2,
            "ASM loc_1BB10 writes `move.b #2,palette_line(a3)`",
        );
        assert_eq!(
            sprite.y_pos, 0x0094,
            "ASM loc_1BB10 writes `move.w #$94,y_pos(a3)` for the intro text vertical position",
        );
        assert_eq!(
            sprite.vram_tile, 0x04BD,
            "ASM loc_1BB10 writes `move.w #$4BD,vram_tile(a3)` for the intro text tile base",
        );
        assert_eq!(
            sprite.sprite_label, "LnkTo_unk_E0ED6-Data_Index",
            "ASM loc_1BB10 writes `move.w #(LnkTo_unk_E0ED6-Data_Index),addroffset_sprite(a3)`",
        );
    }

    #[test]
    #[ignore = "TODO(fidelity): loc_1BB36/loc_1BB44 not yet implemented"]
    fn loc_1bb36_and_loc_1bb44_slide_intro_text_from_left() {
        // ASM loc_1BB36 (kid.asm:30414-30418) and loc_1BB44 (kid.asm:30420-30425):
        // The slide loop calculates x_pos = 268 - (counter * 8).
        // Initial counter is $27 (39) from loc_1B6AC line 29923.
        // The loop decrements counter first, then calculates position.

        // First frame: counter starts at 39, becomes 38 after decrement
        let first_frame = title_text_intro_sprite_x_pos_loc_1BB36_and_loc_1BB44(39);
        assert_eq!(
            first_frame, -44,
            "ASM loc_1BB44 should compute 268 - (38 * 8) = 268 - 304 = -44 for the first visible frame (counter decrements before calculation)",
        );

        // Mid-animation: counter at 20, becomes 19 after decrement
        let mid_frame = title_text_intro_sprite_x_pos_loc_1BB36_and_loc_1BB44(20);
        assert_eq!(
            mid_frame, 116,
            "ASM loc_1BB44 should compute 268 - (19 * 8) = 268 - 152 = 116 for mid-animation",
        );

        // Late animation: counter at 1, becomes 0 after decrement
        let last_moving_frame = title_text_intro_sprite_x_pos_loc_1BB36_and_loc_1BB44(1);
        assert_eq!(
            last_moving_frame, 260,
            "ASM loc_1BB44 should compute 268 - (0 * 8) = 268 for the last moving frame",
        );

        // Final position: counter = 0, stays at 0 (beq.s skips decrement)
        let final_frame = title_text_intro_sprite_x_pos_loc_1BB36_and_loc_1BB44(0);
        assert_eq!(
            final_frame, 268,
            "ASM loc_1BB44 should compute 268 - (0 * 8) = 268 when counter reaches zero and the sprite holds position",
        );
    }

    #[test]
    fn loc_1bb36_decrements_counter_before_calculation() {
        // ASM loc_1BB36: the counter decrement happens in two different code paths:
        // 1. If counter != 0: subq.w #1,$44(a5) executes, then falls through to loc_1BB44
        // 2. If counter == 0: beq.s skips the decrement, uses counter=0 directly
        // This means the first frame with counter=39 produces x_pos using counter=38.

        for initial_counter in [39, 20, 5, 1] {
            let x_pos = title_text_intro_sprite_x_pos_loc_1BB36_and_loc_1BB44(initial_counter);
            let expected_counter = initial_counter - 1;
            let expected_x = 0x010C_u16 as i16 - (expected_counter * 8) as i16;
            assert_eq!(
                x_pos, expected_x,
                "ASM loc_1BB44 with counter={initial_counter} should use effective counter={expected_counter} for x_pos={expected_x}",
            );
        }
    }

    #[test]
    #[ignore = "TODO(fidelity): loc_1BB36/loc_1BB44 not yet implemented"]
    fn loc_1bb36_and_loc_1bb44_full_slide_trajectory() {
        // Verify the slide progresses from off-screen left to final position.
        // The intro text ("THIS IS THE STORY OF SOMEONE TOO TOUGH TO BEAT")
        // starts invisible at x=-44 and slides right 8 pixels per frame.
        let frames: Vec<i16> = (0..=39)
            .map(|frame_idx| {
                // Counter starts at 39 and decrements each frame
                let counter = 39 - frame_idx;
                title_text_intro_sprite_x_pos_loc_1BB36_and_loc_1BB44(counter)
            })
            .collect();

        // First visible position (after 1 frame): -44
        assert_eq!(frames[0], -44, "First visible frame should be at x=-44");

        // The sprite moves 8 pixels right each frame
        for i in 1..frames.len() {
            assert_eq!(
                frames[i] - frames[i - 1],
                8,
                "ASM loc_1BB44 should increment x_pos by 8 pixels per frame (counter decrements by 1, x = 268 - counter*8)",
            );
        }

        // Final position: 268 (stays there forever after counter reaches 0)
        assert_eq!(
            *frames.last().unwrap(),
            268,
            "Final position should be x=268"
        );
    }

    #[test]
    fn loc_1b23a_copies_the_hidden_camera_row_into_camera_y_and_fills_224_scroll_entries() {
        let wrapped_seed = intro_sequence_6_scroll_bootstrap_loc_1B23A(0xFFD0, 0xFFB8);
        let positive_seed = intro_sequence_6_scroll_bootstrap_loc_1B23A(0x0012, 0x1234);

        assert_eq!(
            wrapped_seed.camera_y_pos, 0xFFD0_0000u32 as i32,
            "ASM sub_1B222 should mirror the `($FFFFA656).w` hidden camera row into `(Camera_Y_pos).w` before loc_1B23A writes the scroll buffer",
        );
        assert_eq!(
            wrapped_seed.scroll_fill_longword, 0xFFB8_0000,
            "ASM loc_1B23A should `swap d5` after loading `(Level_Layout).w`, so the horizontal scroll fill longword keeps the layout word in the high half",
        );
        assert_eq!(
            positive_seed.camera_y_pos, 0x0012_0000,
            "ASM sub_1B222 should preserve non-negative hidden camera rows when it copies the word into the high half of `Camera_Y_pos`",
        );
        assert_eq!(
            wrapped_seed.longword_write_count,
            0x00DF + 1,
            "ASM loc_1B23A seeds `d4=#$DF`, so the inclusive `dbf d4,loc_1B23A` loop should fill 224 longwords in `Horiz_Scroll_Buffer`",
        );
    }

    #[test]
    fn loc_1b258_reuses_the_intro_sequence_6_2048_word_vram_clear_before_loading_title_art() {
        let result = title_screen_tilemap_clear_loc_1B258();

        assert_eq!(
            result,
            IntroSequence6WordFillLoc1B182 {
                vram_word_addr: 0x0000,
                fill_word: 0x20BC,
                word_write_count: 0x07FF + 1,
            },
            "ASM loc_1B258 should rerun the same inclusive 2048-word `$20BC` VRAM clear loop as loc_1B182 before `sub_1B2A4` and `Load_TitleArt`",
        );
    }

    #[test]
    fn loc_1b2d8_selects_the_title_art_variant_from_fc82() {
        assert_eq!(title_screen_art_source_loc_1B2D8(0), "byte_26E3D");
        assert_eq!(title_screen_art_source_loc_1B2D8(1), "byte_2927C");
        assert_eq!(
            title_screen_art_source_loc_1B2D8(2),
            "byte_282C1",
            "ASM loc_1B2D8 should only keep the middle title-art source when `($FFFFFC82).w == 1`; every other non-zero variant falls through to `byte_282C1`",
        );
    }

    #[test]
    fn loc_1b322_streams_32_words_from_unk_1b32c_into_vram_f000() {
        let upload = title_screen_tile_attribute_upload_loc_1B322();

        assert_eq!(upload.vram_word_addr, 0xF000);
        assert_eq!(
            upload.words.len(),
            0x001F + 1,
            "ASM loc_1B322 seeds `d0=#$1F`, so the inclusive `dbf d0,loc_1B322` loop should stream 32 tile-attribute words",
        );
        assert_eq!(
            &upload.words[..8],
            &[
                0x1111, 0x1010, 0x0010, 0x0011, 0x0010, 0x0010, 0x0010, 0x0010
            ],
            "ASM loc_1B322 should read big-endian words from `unk_1B32C` before writing them to the VDP data port",
        );
        assert_eq!(
            &upload.words[16..24],
            &[
                0x0010, 0x0000, 0x0110, 0x0000, 0x1010, 0x0000, 0x0010, 0x0000
            ],
        );
        assert_eq!(
            &upload.words[24..],
            &[
                0x0010, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000
            ]
        );
    }

    #[test]
    #[ignore = "TODO(fidelity): loc_1B37C/sub_1B41C not yet implemented"]
    fn loc_1b37c_runs_the_55_step_title_lightning_script_through_sub_1b41c() {
        let sequence = title_screen_lightning_sequence_loc_1B37C();

        assert_eq!(sequence.flash_vram_word_addr, 0x17A0);
        assert_eq!(
            sequence.flash_longword_write_count,
            0x001F + 1,
            "ASM sub_1B41C seeds `d4=#$1F`, so each title lightning flash should upload 32 longwords into VRAM `$17A0`",
        );
        assert_eq!(
            sequence.steps.len(),
            55,
            "ASM loc_1B37C runs four opening flashes, a 45-iteration `dbf d1,loc_1B3BC` loop, then five closing flashes before deleting the worker",
        );
        assert_eq!(
            &sequence.steps[..4],
            &[
                TitleScreenLightningFlashStepLoc1B37C {
                    hibernate_frames: 3,
                    flash_frame: 1,
                    play_thunderstorm: false,
                },
                TitleScreenLightningFlashStepLoc1B37C {
                    hibernate_frames: 3,
                    flash_frame: 0,
                    play_thunderstorm: true,
                },
                TitleScreenLightningFlashStepLoc1B37C {
                    hibernate_frames: 3,
                    flash_frame: 2,
                    play_thunderstorm: false,
                },
                TitleScreenLightningFlashStepLoc1B37C {
                    hibernate_frames: 3,
                    flash_frame: 0,
                    play_thunderstorm: true,
                },
            ],
        );
        assert_eq!(
            sequence.steps[4..12]
                .iter()
                .map(|step| step.flash_frame)
                .collect::<Vec<_>>(),
            vec![1, 4, 3, 2, 1, 4, 3, 2],
            "ASM loc_1B3BC masks `d1` with 3 and adds 1, so the looped flashes should cycle `1,4,3,2` while `d1` counts down from `$2C`",
        );
        assert_eq!(
            &sequence.steps[50..],
            &[
                TitleScreenLightningFlashStepLoc1B37C {
                    hibernate_frames: 3,
                    flash_frame: 0,
                    play_thunderstorm: true,
                },
                TitleScreenLightningFlashStepLoc1B37C {
                    hibernate_frames: 3,
                    flash_frame: 1,
                    play_thunderstorm: false,
                },
                TitleScreenLightningFlashStepLoc1B37C {
                    hibernate_frames: 3,
                    flash_frame: 0,
                    play_thunderstorm: true,
                },
                TitleScreenLightningFlashStepLoc1B37C {
                    hibernate_frames: 3,
                    flash_frame: 2,
                    play_thunderstorm: false,
                },
                TitleScreenLightningFlashStepLoc1B37C {
                    hibernate_frames: 3,
                    flash_frame: 0,
                    play_thunderstorm: true,
                },
            ],
        );
        assert_eq!(
            sequence
                .steps
                .iter()
                .filter(|step| step.play_thunderstorm)
                .count(),
            5,
            "ASM loc_1B37C calls `sub_1B36C` five times across the opening and closing lightning bursts",
        );
    }

    #[test]
    fn loc_1b432_uploads_32_longwords_from_flash_frame_art_banks() {
        let flash_0 = title_screen_lightning_flash_upload_loc_1B432(0);
        let flash_4 = title_screen_lightning_flash_upload_loc_1B432(4);

        assert_eq!(flash_0.flash_frame, 0);
        assert_eq!(flash_0.art_byte_offset, 0);
        assert_eq!(flash_0.vram_word_addr, 0x17A0);
        assert_eq!(
            flash_0.longword_write_count,
            0x001F + 1,
            "ASM loc_1B432 runs one `move.l (a4)+,(a6)` before each inclusive `dbf d4,loc_1B432` check, so every flash uploads 32 longwords",
        );
        assert_eq!(
            flash_4.art_byte_offset, 0x0200,
            "ASM sub_1B41C shifts `d0` left by 7 before loc_1B432 starts, so flash frame 4 reads from `ArtUnc_2A4D6 + $200`",
        );
    }

    #[test]
    fn loc_1b1e8_sets_no_scroll_and_spawns_the_three_hidden_title_shell_workers_in_order() {
        let result = intro_sequence_6_hidden_title_shell_loc_1B1E8(0xFFD0, 0xFFB8);

        assert!(
            result.background_no_scroll,
            "ASM loc_1B1E8 should `st (Background_NoScrollFlag).w` before it allocates the hidden title-shell workers",
        );
        assert_eq!(
            result.scroll_bootstrap,
            IntroSequence6ScrollBootstrapLoc1B23A {
                camera_y_pos: 0xFFD0_0000u32 as i32,
                scroll_fill_longword: 0xFFB8_0000,
                longword_write_count: 0x00DF + 1,
            },
            "ASM loc_1B1E8 should enter through `sub_1B222`, so the hidden title shell inherits the same camera row seed and 224-entry horizontal scroll fill as loc_1B23A",
        );
        assert_eq!(
            result.object_handlers,
            ["loc_1BAB0", "loc_1B37C", "sub_1B43A"],
            "ASM loc_1B1E8 should allocate the hidden title shell workers in `loc_1BAB0`, `loc_1B37C`, then `sub_1B43A` order",
        );
    }

    #[test]
    fn loc_1b446_runs_the_hidden_title_shell_frame_script_until_ff() {
        let script = title_screen_hidden_shell_script_loc_1B446();

        assert_eq!(script.initial_hibernate_frames, 0x002A);
        assert_eq!(script.loop_hibernate_frames, 0x0003);
        assert_eq!(script.sentinel, -1);
        assert_eq!(
            script.title_frames.len(),
            29,
            "ASM loc_1B446 should keep dispatching `sub_1BF24` until the first negative byte, so the 29 non-negative `unk_1B4D6` entries all become title-shell steps",
        );
        assert_eq!(
            &script.title_frames[..10],
            &[0, 1, 0, 1, 0, 1, 2, 1, 2, 3],
            "ASM loc_1B446 must consume `unk_1B4D6` in order, including the repeated early `0,1` flicker pairs before the higher frame ids appear",
        );
        assert_eq!(
            &script.title_frames[24..],
            &[6, 5, 6, 5, 6],
            "ASM loc_1B446 should stop only after the trailing `$FF`, so the final visible passes are the five non-negative bytes immediately before that sentinel",
        );
    }

    #[test]
    fn loc_1b45a_blacks_out_and_restores_all_64_palette_words_before_spawning_title_workers() {
        let restore = title_screen_hidden_shell_palette_restore_loc_1B45A();

        assert_eq!(restore.blackout_hibernate_frames, 0x0039);
        assert_eq!(
            restore.palette_word_count,
            0x003F + 1,
            "ASM loc_1B45A seeds `d0=#$3F` for both loc_1B470 and loc_1B494, so the blackout and restore passes each process all 64 palette words",
        );
        assert_eq!(
            restore.blackout_frame, 7,
            "ASM loc_1B45A loads `d4=7` before `bsr.w sub_1BF24`, so the shell shows frame 7 during the blackout interstitial",
        );
        assert_eq!(restore.interstitial_hibernate_frames, 1);
        assert!(
            restore.reload_title_art,
            "ASM loc_1B45A calls `Load_TitleArt` after restoring the palette data and before allocating the next title workers",
        );
        assert_eq!(
            restore.spawned_handlers,
            [
                "GfxObjects_Collision_BottomBoundary4",
                "loc_1B6AC",
                "loc_1B5CE",
            ],
            "ASM loc_1B45A should allocate the bottom boundary, `loc_1B6AC`, then `loc_1B5CE` after the palette restore tail completes",
        );
    }

    #[test]
    fn loc_1b51a_runs_a_16_frame_hidden_camera_ramp_before_deleting_the_worker() {
        let ramp =
            title_screen_hidden_camera_ramp_loc_1B51A(0xFFB8_0000u32 as i32, 0xFFF0_0000u32 as i32);

        assert_eq!(ramp.initial_hibernate_frames, 0x000A);
        assert_eq!(
            ramp.iteration_count,
            0x000F + 1,
            "ASM loc_1B51A seeds `d2=#$F`, so the inclusive `dbf d2,loc_1B51A` loop should run for 16 one-frame steps",
        );
        assert_eq!(
            ramp.level_layout_step_fp, 0xFFFB_8000u32 as i32,
            "ASM loc_1B51A arithmetic-shifts `(Level_Layout).w` right by 4 before the loop, so `$FFB8_0000` advances upward by 4.5 pixels per pass",
        );
        assert_eq!(
            ramp.hidden_camera_step_fp, 0xFFFF_0000u32 as i32,
            "ASM loc_1B51A also arithmetic-shifts `($FFFFA656).w` right by 4, so the wrapped `$FFF0_0000` hidden camera climbs one pixel per pass",
        );
        assert_eq!(
            &ramp.hidden_camera_values_fp[..3],
            &[
                0xFFF1_0000u32 as i32,
                0xFFF2_0000u32 as i32,
                0xFFF3_0000u32 as i32
            ],
            "ASM loc_1B51A subtracts the negative hidden-camera delta after each one-frame hibernate, so the first three passes should expose a one-pixel-per-frame ramp",
        );
        assert_eq!(
            ramp.hidden_camera_values_fp.last().copied(),
            Some(0),
            "ASM loc_1B51A runs 16 passes from the wrapped `$FFF0_0000` seed, landing exactly on zero before the worker deletes itself",
        );
        assert_eq!(
            ramp.level_layout_values_fp.last().copied(),
            Some(0),
            "ASM loc_1B51A applies the same 16-step subtraction to `(Level_Layout).w`, so the hidden scroll seed should also settle on zero",
        );
        assert!(
            ramp.deletes_current_object,
            "ASM loc_1B51A ends with `jmp (j_Delete_CurrentObject).w` once the ramp finishes",
        );
    }

    #[test]
    fn loc_1b53e_clears_eight_longwords_before_the_title_stripe_rebuild() {
        let clear = title_screen_tile_stripe_clear_loc_1B53E();

        assert_eq!(clear.vram_word_addr, 0x1780);
        assert_eq!(clear.fill_longword, 0x0000_0000);
        assert_eq!(
            clear.longword_write_count,
            0x0007 + 1,
            "ASM loc_1B53E seeds `d0=#7`, so the inclusive `dbf d0,loc_1B53E` loop should write eight zero longwords into VRAM `$1780`",
        );
    }

    #[test]
    fn loc_1b5e4_builds_the_shared_title_text_kid_shell_before_the_popup_spawns() {
        let setup = title_text_kid_setup_loc_1B5E4();

        assert_eq!(setup.palette_line, 2);
        assert_eq!(setup.x_pos, 0x002E);
        assert_eq!(setup.vram_tile, 0x04D3);
        assert_eq!(setup.sprite_mapping_label, "LnkTo_unk_E10FE-Data_Index");
        assert!(
            setup.display_enabled,
            "ASM loc_1B5E4 executes `st $13(a3)` before the first hibernate, so the shared title-text worker should be visible once the later y-position entry path settles",
        );
        assert_eq!(setup.pre_drift_hibernate_frames, 0x001E);
        assert_eq!(
            setup.drift_frame_count,
            0x002A + 1,
            "ASM loc_1B5E4 seeds `d0=#$2A`, so the inclusive `dbf d0,loc_1B610` drift should run for 43 one-frame steps",
        );
        assert!(
            setup.moves_during_drift,
            "ASM loc_1B5E4 sets `is_moved(a3)` immediately before loc_1B610 and clears it only after the DBF loop finishes",
        );
        assert_eq!(setup.post_drift_hibernate_frames, 0x0014);
        assert_eq!(setup.interstitial_hibernate_frames, 0x000A);
        assert_eq!(setup.popup_positions, [(0x002E, 0x008C), (0x005A, 0x0098)]);
        assert!(
            setup.loops_forever,
            "ASM loc_1B64C falls into a perpetual one-frame hibernate loop after the second popup position is written",
        );
    }

    #[test]
    fn loc_1adea_selects_the_intro_sequence_1_player_title_sprite_from_current_player() {
        let player_1 = intro_sequence_1_player_title_sprite_loc_1ADEA(0);
        let player_2 = intro_sequence_1_player_title_sprite_loc_1ADEA(1);

        assert_eq!(player_1.gfx_slot_base, 0x0300_0004);
        assert!(
            player_1.priority_flag,
            "ASM loc_1ADEA inherits the earlier `st $13(a1)` priority flag on the second intro-sequence-1 title sprite",
        );
        assert!(
            player_1.display_enabled,
            "ASM loc_1ADEA keeps `$11(a1)=1` after the shared allocation shell",
        );
        assert_eq!(
            player_1.sprite_label, "LnkTo_unk_C87E8-Data_Index",
            "ASM loc_1ADEA keeps `d7=$81` for player 1, so `off_1AC2A[d7-$61]` resolves to the `$C87E8` title sprite mapping",
        );
        assert_eq!(
            player_2.sprite_label, "LnkTo_unk_C87F0-Data_Index",
            "ASM loc_1ADEA increments `d7` to `$82` when `(Current_player).w` is non-zero, selecting the next `$C87F0` mapping for player 2",
        );
        assert_eq!(player_1.vram_tile, 0x0409);
        assert_eq!(player_1.target_x_pos, 0x0160);
        assert_eq!(player_1.target_y_pos, 0x00C8);
    }

    #[test]
    fn loc_1b6a2_hibernates_one_frame_then_calls_sub_1b71a_without_touching_d2() {
        let decoded_words = (0u16..(7 * 29) as u16).collect::<Vec<_>>();

        let step = title_text_chameleon_idle_step_loc_1B6A2(&decoded_words, 0, 3);

        assert_eq!(step.hibernate_frames, 1);
        assert_eq!(
            step.next_d2, 0,
            "ASM loc_1B6A2 only calls `Hibernate_Object_1Frame` and `sub_1B71A`, so the `moveq #0,d2` value from Obj_TitleTextChameleon should stay latched forever",
        );
        assert_eq!(
            step.next_d3, 4,
            "ASM sub_1B71A starts with `addq.w #1,d3`, so loc_1B6A2 should advance the animation selector once per loop",
        );
        assert!(
            step.loops_forever,
            "ASM loc_1B6A2 ends in `bra.s loc_1B6A2`, so the title-text worker should never leave this cadence on its own",
        );
        assert_eq!(
            step.upload.as_ref().map(|upload| upload.tilemap_label),
            Some("unk_2E728"),
            "with `d3=3` on entry, ASM loc_1B6A2 reaches sub_1B71A with `d3=4`, so `d3 & $C` should pick the second `off_1B70A` tilemap",
        );
    }

    #[test]
    fn loc_1b6ce_keeps_the_first_seven_reveal_frames_hidden_then_seeds_d0_for_loc_1b6dc() {
        let decoded_words = (0u16..(7 * 29) as u16).collect::<Vec<_>>();

        let reveal = title_text_chameleon_reveal_loop_loc_1B6CE(&decoded_words);

        assert_eq!(reveal.hibernate_frames_per_iteration, 1);
        assert_eq!(
            reveal.iterations.len(),
            0x28,
            "ASM loc_1B6CE starts from `d2=#$28` and exits only after the post-`subq.w #1,d2` value reaches zero, so the loop should hibernate exactly 40 times",
        );
        assert!(
            reveal.iterations[..7]
                .iter()
                .all(|iteration| iteration.upload.is_none()),
            "ASM sub_1B71A returns immediately while `d2 >= $22`, so the first seven loc_1B6CE iterations should stay hidden",
        );
        assert_eq!(
            reveal.iterations[7].incoming_d2, 0x0021,
            "the eighth loc_1B6CE pass should be the first visible one because `d2` has just dropped below sub_1B71A's `$22` guard",
        );
        assert!(
            reveal.iterations[7].upload.is_some(),
            "once `d2=$21`, ASM loc_1B6CE should finally let sub_1B71A decode and upload the Chameleon strip",
        );
        assert_eq!(
            reveal.final_d0, 0,
            "ASM loc_1B6CE falls through to `moveq #0,d0` immediately after the countdown, seeding the later loc_1B6DC Sega-text delay",
        );
    }

    #[test]
    fn loc_1b6dc_restores_d0_after_sub_1b71a_and_spawns_sega_text_only_on_count_1e() {
        let decoded_words = (0u16..(7 * 29) as u16).collect::<Vec<_>>();

        let spawn_step = title_text_chameleon_sega_step_loc_1B6DC(&decoded_words, 0x001D, 3);
        let post_spawn_step = title_text_chameleon_sega_step_loc_1B6DC(&decoded_words, 0x001E, 4);

        assert_eq!(spawn_step.hibernate_frames, 1);
        assert_eq!(
            spawn_step.next_d0, 0x001E,
            "ASM loc_1B6DC saves `d0` on the stack before `sub_1B71A`, restores it, then runs `addq.w #1,d0`, so the Sega-text threshold should be based on the preserved counter rather than the helper's scratch value",
        );
        assert_eq!(
            spawn_step.spawned_handler,
            Some("loc_1C7A0"),
            "the preserved `d0=$1D -> $1E` transition is the only loc_1B6DC pass that should allocate the fixed Sega text worker",
        );
        assert_eq!(spawn_step.spawned_art_label, Some("SegaText"));
        assert_eq!(
            spawn_step
                .upload
                .as_ref()
                .map(|upload| upload.tilemap_label),
            Some("unk_2E728"),
            "with `d3=3` on entry, loc_1B6DC still reaches sub_1B71A after the leading `addq.w #1,d3`, so the visible upload should use the second `off_1B70A` entry while `d2` stays zero",
        );
        assert_eq!(
            post_spawn_step.next_d0, 0x001F,
            "after the spawn frame, ASM loc_1B6DC keeps incrementing the restored counter on each hibernated pass",
        );
        assert_eq!(
            post_spawn_step.spawned_handler, None,
            "ASM loc_1B6DC branches back to the top after allocating the Sega text worker, so later frames should not keep spawning duplicate handlers",
        );
        assert!(post_spawn_step.loops_forever);
    }

    #[test]
    fn loc_1ae10_moves_the_left_intro_sequence_1_title_sprite_4px_until_x60() {
        assert_eq!(
            intro_sequence_1_left_title_sprite_step_loc_1AE10(-0x0030, 0x0060),
            -0x002C,
            "ASM loc_1AE10 runs `addi.w #4,x_pos(a3)` whenever the left title sprite has not yet reached `d0=$60`",
        );
        assert_eq!(
            intro_sequence_1_left_title_sprite_step_loc_1AE10(0x0060, 0x0060),
            0x0060,
            "ASM loc_1AE10 branches straight to loc_1AE20 once `cmp.w x_pos(a3),d0` sees the target already reached",
        );
    }

    #[test]
    fn loc_1ae20_moves_the_right_intro_sequence_1_title_sprite_4px_until_xd0() {
        assert_eq!(
            intro_sequence_1_right_title_sprite_step_loc_1AE20(0x0160, 0x00D0),
            0x015C,
            "ASM loc_1AE20 runs `subi.w #4,$1A(a1)` whenever the mirrored title sprite has not yet reached `d1=$D0`",
        );
        assert_eq!(
            intro_sequence_1_right_title_sprite_step_loc_1AE20(0x00D0, 0x00D0),
            0x00D0,
            "ASM loc_1AE20 branches back to loc_1AE10 without changing `$1A(a1)` once the right sprite already matches the target",
        );
    }

    #[test]
    fn sub_1b71a_uses_d3_bits_2_and_3_to_rotate_the_tilemap_source() {
        let decoded_words = (0u16..(7 * 29) as u16).collect::<Vec<_>>();

        let first = title_text_chameleon_frame_upload_sub_1B71A(&decoded_words, 0, 0x0000)
            .expect("ASM sub_1B71A should upload when d2 is below $22");
        let second = title_text_chameleon_frame_upload_sub_1B71A(&decoded_words, 0, 0x0004)
            .expect("second table entry should stay valid");
        let third = title_text_chameleon_frame_upload_sub_1B71A(&decoded_words, 0, 0x0008)
            .expect("third table entry should stay valid");
        let fourth = title_text_chameleon_frame_upload_sub_1B71A(&decoded_words, 0, 0x000C)
            .expect("fourth table entry should stay valid");

        assert_eq!(first.tilemap_label, "unk_2E706");
        assert_eq!(second.tilemap_label, "unk_2E728");
        assert_eq!(third.tilemap_label, "unk_2E77A");
        assert_eq!(
            fourth.tilemap_label, "unk_2E728",
            "ASM off_1B70A points both the second and fourth entries at unk_2E728"
        );
    }

    #[test]
    fn sub_1b71a_caps_the_first_row_and_emits_the_left_filler_during_the_early_reveal() {
        let decoded_words = (0u16..(7 * 29) as u16).collect::<Vec<_>>();

        let upload = title_text_chameleon_frame_upload_sub_1B71A(&decoded_words, 0, 0x0000)
            .expect("ASM sub_1B71A should build a clipped upload for early reveal frames");

        assert_eq!(upload.rows.len(), 7);
        assert_eq!(upload.rows[0].vram_word_address, 0x048A);
        assert_eq!(
            upload.rows[0].filler_word,
            Some(0x00BC),
            "ASM loc_1B780 writes tile $00BC one word to the left while d4 is still positive"
        );
        assert_eq!(upload.rows[0].words.len(), 21);
        assert_eq!(upload.rows[0].words[0], 0);
        assert_eq!(upload.rows[0].words[20], 20);
        assert_eq!(upload.rows[1].words.len(), 29);
        assert_eq!(upload.rows[1].words[0], 29);
    }

    #[test]
    fn loc_1b734_clamps_negative_reveal_indices_back_to_zero() {
        let decoded_words = (0u16..(7 * 29) as u16).collect::<Vec<_>>();

        let upload = title_text_chameleon_frame_upload_sub_1B71A(&decoded_words, -2, 0x0000)
            .expect("ASM loc_1B734 should still upload after negative d2 is clamped back to zero");

        assert_eq!(
            upload.rows[0].vram_word_address, 0x048A,
            "ASM loc_1B734 forces negative `d2` to zero before loc_1B758 builds the `$048A` top-row write address"
        );
        assert_eq!(
            upload.rows[0].filler_word,
            Some(0x00BC),
            "ASM loc_1B734 should preserve the early-reveal filler path after the negative `d2` clamp"
        );
        assert_eq!(
            upload.rows[1].words.len(),
            29,
            "ASM loc_1B734 resets negative `d2` before the row-width math, so later rows still copy the full 29 decoded words"
        );
    }

    #[test]
    fn sub_1b71a_clips_from_the_left_once_d2_has_passed_the_initial_window() {
        let decoded_words = (0u16..(7 * 29) as u16).collect::<Vec<_>>();

        let upload = title_text_chameleon_frame_upload_sub_1B71A(&decoded_words, 8, 0x0008)
            .expect("ASM sub_1B71A should keep uploading after d2 passes the blank lead-in");

        assert_eq!(upload.tilemap_label, "unk_2E77A");
        assert_eq!(upload.rows[0].vram_word_address, 0x0480);
        assert_eq!(upload.rows[0].filler_word, None);
        assert_eq!(
            upload.rows[0].words.len(),
            21,
            "after clipping three words from the source row, the first row still caps at 21 words"
        );
        assert_eq!(upload.rows[0].words[0], 3);
        assert_eq!(upload.rows[0].words[20], 23);
        assert_eq!(upload.rows[1].words.len(), 26);
        assert_eq!(upload.rows[1].words[0], 32);
    }

    #[test]
    fn loc_1b758_builds_the_vram_word_address_from_row_12_and_the_horizontal_offset() {
        let decoded_words = (0u16..(7 * 29) as u16).collect::<Vec<_>>();

        let upload = title_text_chameleon_frame_upload_sub_1B71A(&decoded_words, 2, 0x0004).expect(
            "ASM loc_1B758 should still upload while the early reveal keeps three blank columns",
        );

        assert_eq!(upload.tilemap_label, "unk_2E728");
        assert_eq!(
            upload.rows[0].vram_word_address, 0x0486,
            "ASM loc_1B758 starts from row `$12 << 7 = $900`, adds `d4+d4 = 6`, then the swapped VDP command exposes word address `$0486`"
        );
        assert_eq!(
            upload.rows[1].vram_word_address, 0x04C6,
            "ASM loc_1B79E adds `$800000` between rows, so the next word address should advance by `$40`"
        );
        assert_eq!(
            upload.rows[0].filler_word,
            Some(0x00BC),
            "ASM loc_1B758 still feeds loc_1B780's filler path while `d4` stays positive"
        );
    }

    #[test]
    fn sub_1b71a_returns_none_once_d2_reaches_the_terminal_guard() {
        let decoded_words = (0u16..(7 * 29) as u16).collect::<Vec<_>>();

        assert!(
            title_text_chameleon_frame_upload_sub_1B71A(&decoded_words, 0x22, 0).is_none(),
            "ASM sub_1B71A returns immediately once `cmpi.w #$22,d2` succeeds"
        );
    }

    #[test]
    fn loc_1b76c_caps_only_the_first_row_at_21_words_when_the_reveal_starts_on_column_five() {
        let decoded_words = (0u16..(7 * 29) as u16).collect::<Vec<_>>();

        let upload = title_text_chameleon_frame_upload_sub_1B71A(&decoded_words, 5, 0x0008).expect(
            "ASM loc_1B76C should still upload once the reveal reaches the first visible column",
        );

        assert_eq!(upload.tilemap_label, "unk_2E77A");
        assert_eq!(
            upload.rows[0].words.len(),
            21,
            "ASM loc_1B76C clamps the first `d1` copy count to `$14`, so the inclusive DBF loop writes 21 top-row words"
        );
        assert_eq!(
            upload.rows[0].filler_word, None,
            "ASM loc_1B76C only clamps the top-row copy count here; with `d4=0`, loc_1B780 should skip the left filler write"
        );
        assert_eq!(
            upload.rows[1].words.len(),
            29,
            "ASM loc_1B76C only applies while `d7==6`, so later rows keep the full 29-word width"
        );
    }

    #[test]
    fn loc_1b79a_writes_the_row_base_directly_once_the_blank_lead_in_is_gone() {
        let decoded_words = (0u16..(7 * 29) as u16).collect::<Vec<_>>();

        let upload = title_text_chameleon_frame_upload_sub_1B71A(&decoded_words, 5, 0x0000).expect(
            "ASM loc_1B79A should still upload once the reveal reaches the first visible column",
        );

        assert_eq!(
            upload.rows[0].vram_word_address, 0x0480,
            "ASM loc_1B79A falls through to `move.l d0,4(a6)` with `d4=0`, so the top row should use the unclipped `$0480` base address"
        );
        assert_eq!(
            upload.rows[0].filler_word, None,
            "ASM loc_1B79A is the no-filler branch after `tst.w d4; beq.s loc_1B79A`"
        );
    }

    #[test]
    fn loc_1b7a6_copies_the_clipped_row_width_with_inclusive_dbf_semantics() {
        let decoded_words = (0u16..(7 * 29) as u16).collect::<Vec<_>>();

        let upload = title_text_chameleon_frame_upload_sub_1B71A(&decoded_words, 33, 0x0008)
            .expect("ASM loc_1B7A6 should still emit one decoded word when the clipped copy count reaches zero");

        assert_eq!(
            upload.rows.len(),
            7,
            "ASM loc_1B7A6 still runs once per row while `dbf d7,loc_1B76C` walks all seven rows"
        );
        assert!(upload.rows.iter().all(|row| row.words.len() == 1));
        assert_eq!(
            upload.rows[0].words,
            vec![28],
            "ASM loc_1B7A6 writes before `dbf d1,loc_1B7A6`, so a clipped `d1=0` top row should still copy the final decoded word"
        );
        assert_eq!(
            upload.rows[6].words,
            vec![202],
            "ASM loc_1B7A6 should keep the same inclusive one-word copy rule on later rows after the source pointer advances by `$1D` words per row"
        );
    }

    #[test]
    fn sub_1b7b6_replays_the_full_title_word_write_table() {
        let writes = title_text_word_writes_sub_1B7B6();

        assert_eq!(
            writes.len(),
            55,
            "ASM sub_1B7B6 walks the 55-entry `unk_1B7E0` table before the `$FFFF` sentinel",
        );
        assert_eq!(
            writes.first(),
            Some(&TitleTextWordWritesSub1B7B6 {
                base_word_address: 0x7C12,
                top_words: [0x21AB, 0x21AC],
                bottom_words: [0x21AD, 0x21AE],
            }),
            "ASM sub_1B7B6 starts by writing the fixed four-word pattern at $7C12",
        );
        assert_eq!(
            writes.last(),
            Some(&TitleTextWordWritesSub1B7B6 {
                base_word_address: 0x7FEE,
                top_words: [0x21AB, 0x21AC],
                bottom_words: [0x21AD, 0x21AE],
            }),
            "ASM sub_1B7B6 should stop after the last `$7FEE` entry and not include the `$FFFF` terminator",
        );
    }

    #[test]
    fn sub_1b7b6_keeps_the_bottom_row_offset_by_0x50_words() {
        let writes = title_text_word_writes_sub_1B7B6();

        assert!(writes.iter().all(|entry| {
            entry.bottom_word_addresses()
                == [
                    entry.base_word_address + 0x50,
                    entry.base_word_address + 0x52,
                ]
        }));
    }

    #[test]
    fn loc_1b7ba_repeats_the_same_four_tile_words_for_every_non_sentinel_entry() {
        let writes = title_text_word_writes_sub_1B7B6();

        assert!(writes.iter().all(|entry| {
            entry.top_words == [0x21AB, 0x21AC] && entry.bottom_words == [0x21AD, 0x21AE]
        }));
        assert_eq!(
            writes
                .iter()
                .filter(|entry| entry.base_word_address == 0xFFFF)
                .count(),
            0,
            "ASM loc_1B7BA exits on `cmpi.w #$FFFF,d0; beq.s return_1B7DE`, so the sentinel must never become a write target"
        );
    }

    // ── loc_6A84: pause menu arrow cursor VDP commands ──────────────

    #[test]
    fn loc_6A84_arrow_cmds_at_origin_camera() {
        // ASM loc_6A84: Camera_X=0, Camera_Y=0
        // Column: (0+$68)>>3 = $D, &$3F = $D, *2 = $1A
        // Upper row: (0+$48)>>3 = 9, &$1F = 9, <<7 = $480; $480+$1A = $49A
        //   VDP cmd = ($49A | $4000) << 16 = $449A_0000
        // Lower row: (0+$58)>>3 = $B, &$1F = $B, <<7 = $580; $580+$1A = $59A
        //   VDP cmd = ($59A | $4000) << 16 = $459A_0000
        let cmds = pause_menu_arrow_cursor_cmds_loc_6A84(0, 0);
        assert_eq!(cmds.upper_arrow_cmd, 0x449A_0000);
        assert_eq!(cmds.lower_arrow_cmd, 0x459A_0000);
    }

    #[test]
    fn loc_6A84_arrow_cmds_wrap_plane_row() {
        // ASM loc_6A84: Camera_Y=$D8 — upper row pixel = $D8+$48 = $120
        //   ($120 >> 3) = $24, & $1F = 4, << 7 = $200
        // Lower row pixel = $D8+$58 = $130
        //   ($130 >> 3) = $26, & $1F = 6, << 7 = $300
        // Column: Camera_X=$100 → ($100+$68)>>3 = $2D, &$3F = $2D, *2 = $5A
        let cmds = pause_menu_arrow_cursor_cmds_loc_6A84(0x100, 0xD8);
        let upper_vram = 0x200 + 0x5A;
        let lower_vram = 0x300 + 0x5A;
        assert_eq!(cmds.upper_arrow_cmd, ((upper_vram | 0x4000) as u32) << 16);
        assert_eq!(cmds.lower_arrow_cmd, ((lower_vram | 0x4000) as u32) << 16);
    }

    #[test]
    fn loc_6A84_upper_and_lower_differ_only_in_row() {
        // ASM loc_6A84: both commands share the same column byte offset;
        // only the row component ($48 vs $58) differs.
        for cam_x in [0u16, 0x80, 0x1F0] {
            for cam_y in [0u16, 0x40, 0xF0] {
                let cmds = pause_menu_arrow_cursor_cmds_loc_6A84(cam_x, cam_y);
                // Column bits are in the low 7 bits of the VRAM address (high word)
                let upper_col = (cmds.upper_arrow_cmd >> 16) & 0x7F;
                let lower_col = (cmds.lower_arrow_cmd >> 16) & 0x7F;
                assert_eq!(
                    upper_col, lower_col,
                    "ASM loc_6A84: d0 and d1 share the same column byte offset"
                );
                // Low word is always 0
                assert_eq!(cmds.upper_arrow_cmd & 0xFFFF, 0);
                assert_eq!(cmds.lower_arrow_cmd & 0xFFFF, 0);
            }
        }
    }

    // ── loc_6B0E: pause loop down-button handler ────────────────────

    #[test]
    fn loc_6B0E_down_pressed_at_option_0_moves_cursor_down() {
        // ASM loc_6B0E: Down pressed, Pause_Option == 0 → move cursor down
        let result = pause_loop_down_handler_loc_6B0E(true, 0);
        let m = result.expect("ASM loc_6B0E: cursor should move when Down pressed at option 0");
        assert_eq!(
            m.upper_arrow_tile, PAUSE_ARROW_ABSENT_TILE,
            "ASM loc_6B0E: move.w #$8678,(a6) clears the upper arrow"
        );
        assert_eq!(
            m.lower_arrow_tile, PAUSE_ARROW_PRESENT_TILE,
            "ASM loc_6B0E: move.w #$867C,(a6) draws the lower arrow"
        );
        assert_eq!(
            m.new_pause_option, 1,
            "ASM loc_6B0E: addi.b #1,(Pause_Option).w"
        );
    }

    #[test]
    fn loc_6B0E_down_pressed_at_option_1_does_nothing() {
        // ASM loc_6B0E: tst.b (Pause_Option).w; bne.s Game_Paused_ChkStart
        assert!(
            pause_loop_down_handler_loc_6B0E(true, 1).is_none(),
            "ASM loc_6B0E: already at option 1 → skip"
        );
    }

    #[test]
    fn loc_6B0E_down_not_pressed_does_nothing() {
        // ASM loc_6B0E: bclr #Button_Down; beq.s Game_Paused_ChkStart
        assert!(
            pause_loop_down_handler_loc_6B0E(false, 0).is_none(),
            "ASM loc_6B0E: Down not pressed → skip"
        );
    }

    #[test]
    fn game_paused_loop_up_pressed_at_option_1_moves_cursor_up() {
        // ASM Game_Paused_Loop (kid.asm:6832-6843): Up pressed, Pause_Option != 0
        let result = pause_loop_up_handler_game_paused_loop(true, 1);
        let m = result.expect("cursor should move when Up pressed at option 1");
        assert_eq!(
            m.upper_arrow_tile, PAUSE_ARROW_PRESENT_TILE,
            "ASM Game_Paused_Loop: move.w #$867C,(a6) draws the upper arrow"
        );
        assert_eq!(
            m.lower_arrow_tile, PAUSE_ARROW_ABSENT_TILE,
            "ASM Game_Paused_Loop: move.w #$8678,(a6) clears the lower arrow"
        );
        assert_eq!(
            m.new_pause_option, 0,
            "ASM Game_Paused_Loop: subi.b #1,(Pause_Option).w"
        );
    }

    #[test]
    fn game_paused_loop_up_pressed_at_option_0_does_nothing() {
        // ASM Game_Paused_Loop: tst.b (Pause_Option).w; beq.s Game_Paused_ChkStart
        assert!(
            pause_loop_up_handler_game_paused_loop(true, 0).is_none(),
            "already at option 0 → skip"
        );
    }

    // ── loc_6B62 + Game_Paused_ChkStart: start handler ─────────────

    #[test]
    fn chk_start_not_pressed_returns_not_pressed() {
        // ASM Game_Paused_ChkStart: bclr #Button_Start; beq.s Game_Paused_Loop
        assert_eq!(
            pause_loop_start_handler_chk_start(false, 0, 3),
            PauseMenuStartResult::NotPressed,
        );
    }

    #[test]
    fn chk_start_option_0_continues_game() {
        // ASM Game_Paused_ChkStart: tst.b (Pause_Option).w; beq.s loc_6B68
        assert_eq!(
            pause_loop_start_handler_chk_start(true, 0, 3),
            PauseMenuStartResult::ContinueGame,
        );
    }

    #[test]
    fn chk_start_option_1_with_multiple_lives_restarts_level() {
        // ASM: subi.w #1,(Number_Lives).w; result != 0 → restart
        let result = pause_loop_start_handler_chk_start(true, 1, 3);
        assert_eq!(
            result,
            PauseMenuStartResult::RestartLevel { remaining_lives: 2 },
        );
    }

    #[test]
    fn loc_6B62_last_life_triggers_game_over() {
        // ASM loc_6B62 (kid.asm:6877): subi.w #1,(Number_Lives).w; beq.s loc_6B62
        // → move.b #3,(Game_Paused).w
        assert_eq!(
            pause_loop_start_handler_chk_start(true, 1, 1),
            PauseMenuStartResult::LastLifeGameOver,
            "ASM loc_6B62: last life → Game_Paused = 3"
        );
    }

    #[test]
    fn chk_start_option_1_with_two_lives_restarts_with_one() {
        // ASM: Number_Lives=2, subi.w #1 → 1 (not zero), restart
        assert_eq!(
            pause_loop_start_handler_chk_start(true, 1, 2),
            PauseMenuStartResult::RestartLevel { remaining_lives: 1 },
        );
    }

    // ── loc_6C0A + loc_6C3E: pause menu unpause exit ─────────────────

    #[test]
    fn loc_6C0A_game_paused_3_triggers_last_life_game_over() {
        // ASM loc_6C0A (kid.asm:6958): cmpi.b #3,(Game_Paused).w; bne.s loc_6C3E
        // When Game_Paused == 3 (set by loc_6B62), the kid dies.
        let result = pause_menu_unpause_exit_loc_6C0A(3);
        match result {
            PauseMenuUnpauseExitLoc6C0A::LastLifeGameOver {
                kid_action,
                kid_action_param,
                number_lives,
            } => {
                assert_eq!(kid_action, 4, "ASM loc_6C0A: move.w #4,$38(a4)");
                assert_eq!(kid_action_param, 0x30, "ASM loc_6C0A: move.w #$30,$3A(a4)");
                assert_eq!(number_lives, 1, "ASM loc_6C0A: move.w #1,(Number_Lives).w");
            }
            other @ PauseMenuUnpauseExitLoc6C0A::NormalUnpause => {
                panic!("expected LastLifeGameOver, got {other:?}")
            }
        }
    }

    #[test]
    fn loc_6C3E_game_paused_not_3_triggers_normal_unpause() {
        // ASM loc_6C3E (kid.asm:6976): sf (Game_Paused).w; jsr (sub_E1338).l;
        // lea (off_Load_EyclopsBeamArt).l,a0; move.l (a0),a0; jsr (a0)
        // Any value other than 3 takes the normal unpause path.
        for game_paused in [1u8, 2, 4, 0xFF] {
            assert_eq!(
                pause_menu_unpause_exit_loc_6C0A(game_paused),
                PauseMenuUnpauseExitLoc6C0A::NormalUnpause,
                "ASM loc_6C0A: Game_Paused={game_paused} → bne.s loc_6C3E"
            );
        }
    }

    #[test]
    fn loc_6C0A_both_paths_clear_game_paused() {
        // Both paths end with `sf (Game_Paused).w` — the native port clears
        // the paused flag. This test documents that both branches share the
        // same post-condition (Game_Paused = 0) even though the enum doesn't
        // carry it — the caller is responsible for clearing the flag.
        // Game_Paused==3 path (loc_6C0A):
        assert!(matches!(
            pause_menu_unpause_exit_loc_6C0A(3),
            PauseMenuUnpauseExitLoc6C0A::LastLifeGameOver { .. }
        ));
        // Game_Paused!=3 path (loc_6C3E):
        assert!(matches!(
            pause_menu_unpause_exit_loc_6C0A(1),
            PauseMenuUnpauseExitLoc6C0A::NormalUnpause
        ));
    }

    // ── loc_6C70 / Pause_DarkenPalette: palette halving ──────────────

    #[test]
    fn loc_6C70_halves_each_rgb_channel_independently() {
        // ASM loc_6C70 (kid.asm:7001): each 4-bit channel is halved via lsr #1.
        // BGR word $ECA → blue=$A>>1=5, green=$C0>>1&$F0=$60, red=$E00>>1&$F00=$700
        // → $765
        let mut buf = [0u16; 0x40];
        buf[0] = 0x0ECA;
        pause_darken_palette_loc_6C70(&mut buf);
        assert_eq!(
            buf[0], 0x0765,
            "ASM loc_6C70: $ECA → blue=$A>>1=5, green=$C>>1=6, red=$E>>1=7"
        );
    }

    #[test]
    fn loc_6C70_odd_channels_truncate_low_bit() {
        // ASM: lsr.w #1 on an odd nibble value drops the low bit.
        // $111 → blue=1>>1=0, green=$10>>1&$F0=$00, red=$100>>1&$F00=$000 → $000
        let mut buf = [0u16; 0x40];
        buf[0] = 0x0111;
        pause_darken_palette_loc_6C70(&mut buf);
        assert_eq!(
            buf[0], 0x0000,
            "ASM loc_6C70: $111 each channel has value 1 → >>1 = 0"
        );
    }

    #[test]
    fn loc_6C70_skips_colour_15_of_first_palette_line() {
        // ASM Pause_DarkenPalette: first loop d7=$E (15 colours), then addq.w #2,a4
        // skips colour 15, then d7=$2F (48 colours from index 16 onward).
        let mut buf = [0x0EEE_u16; 0x40];
        let sentinel = 0x0ABC_u16;
        buf[15] = sentinel;
        pause_darken_palette_loc_6C70(&mut buf);
        assert_eq!(
            buf[15], sentinel,
            "ASM Pause_DarkenPalette: colour 15 is skipped (addq.w #2,a4)"
        );
        // Verify adjacent colours WERE darkened
        assert_eq!(buf[14], 0x0777, "colour 14 should be darkened");
        assert_eq!(buf[16], 0x0777, "colour 16 should be darkened");
    }

    #[test]
    fn loc_6C70_darkens_all_64_colours_except_15() {
        // ASM: 15 + skip + 48 = 63 darkened out of 64 total.
        let mut buf = [0x0EEE_u16; 0x40];
        pause_darken_palette_loc_6C70(&mut buf);
        for (i, &val) in buf.iter().enumerate() {
            if i == 15 {
                assert_eq!(val, 0x0EEE, "colour 15 unchanged");
            } else {
                assert_eq!(val, 0x0777, "colour {i} should be halved");
            }
        }
    }

    #[test]
    fn loc_6C70_zero_palette_stays_zero() {
        // ASM: 0 >> 1 = 0 for all channels.
        let mut buf = [0u16; 0x40];
        pause_darken_palette_loc_6C70(&mut buf);
        for (i, &val) in buf.iter().enumerate() {
            assert_eq!(val, 0, "zero stays zero at index {i}");
        }
    }

    #[test]
    fn loc_6C70_max_value_halves_correctly() {
        // ASM: $FFF → blue=$F>>1=7, green=$F0>>1&$F0=$70, red=$F00>>1&$F00=$700
        // → $777
        let mut buf = [0x0FFF_u16; 0x40];
        pause_darken_palette_loc_6C70(&mut buf);
        assert_eq!(buf[0], 0x0777, "$FFF → $777");
        assert_eq!(buf[63], 0x0777, "last colour also darkened");
    }
}
