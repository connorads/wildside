use super::*;
use kid_data::{DiamondHudDmaTransferSub5D4A, diamond_hud_dma_transfer_sub_5D4A};

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
/// ASM: move.w #$1F,d6; dbf loop: move.l (a4)+,(a6)
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

    if timer_frozen {
        return layout;
    }

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
        coin_upload: coin_frame.map(|frame_table_byte_offset| HudFrameUploadSub44DC {
            asset_label: "ArtUnc_4992",
            vram_word_address: 0x4720,
            frame_table_byte_offset,
        }),
        life_upload: life_frame.map(|frame_table_byte_offset| HudFrameUploadSub44DC {
            asset_label: "ArtUnc_4692",
            vram_word_address: 0x4620,
            frame_table_byte_offset,
        }),
        clock_upload: clock_frame.map(|frame_table_byte_offset| HudFrameUploadSub44DC {
            asset_label: "ArtUnc_4812",
            vram_word_address: 0x46A0,
            frame_table_byte_offset,
        }),
        diamond_upload: diamond_frame.map(|frame_table_byte_offset| {
            diamond_hud_dma_transfer_sub_5D4A(u16::from(frame_table_byte_offset))
                .expect("ASM sub_44DC only emits selectors from `unk_461C` that fit sub_5D4A")
        }),
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

#[allow(non_snake_case)]
#[allow(dead_code)]
pub(crate) fn hide_pause_menu_overwritten_sprites_sub_6874(
    sprites: &mut [PauseMenuSpriteEntrySub6874],
) {
    // ASM sub_6874 / loc_68AC (kid.asm:6575): when the pause menu overwrites
    // VRAM tiles `$676..=$68E`, scan the `$51` sprite entries and clear the
    // Y word for any sprite whose masked tile index (`tile_word & $7FF`)
    // falls inside that inclusive range.
    for sprite in sprites {
        let masked_tile = sprite.vram_tile_word & 0x07FF;
        if (0x0676..=0x068E).contains(&masked_tile) {
            sprite.y_pos = 0;
        }
    }
}

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

impl TitleTextWordWritesSub1B7B6 {
    #[cfg_attr(not(test), allow(dead_code))]
    pub(crate) fn bottom_word_addresses(self) -> [u16; 2] {
        [self.base_word_address + 0x50, self.base_word_address + 0x52]
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

    TitleScreenLightningSequenceLoc1B37C {
        flash_vram_word_addr: 0x17A0,
        flash_longword_write_count: 0x001F + 1,
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
    // ASM sub_1B71A (kid.asm:29957): pick one of four Enigma tilemaps from
    // `off_1B70A` via `d3 & $C`, clip the 7x29 decoded strip against the
    // reveal index `d2`, cap the first row to 21 words, and optionally emit a
    // `$00BC` filler word one cell to the left when the reveal still starts in
    // blank space.
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
        let top_row_limit = if row_index == 0 {
            row_word_count.min(TOP_ROW_MAX_WORDS)
        } else {
            row_word_count
        };
        let words = decoded_words[row_start..row_start + top_row_limit].to_vec();
        let row_base = 0x0480_u16
            .checked_add(horizontal_offset_words.saturating_mul(2))?
            .checked_add(u16::try_from(row_index).ok()?.saturating_mul(64))?;
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
    if toggle_byte != 0 {
        (
            toggle_byte,
            left_panel_x.wrapping_add(panel_delta),
            right_panel_x.wrapping_sub(panel_delta),
        )
    } else {
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
        // Auto-end after DEMO_MAX_FRAMES
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
    pub(crate) fn handle_start_press(&mut self, input: FrameInput, start_pressed: bool) {
        if !start_pressed {
            return;
        }
        self.start_press_count += 1;
        if matches!(self.player_lifecycle, PlayerLifecycle::Alive) {
            match self.boot_state {
                BootState::InGame => {
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
                        self.paused = false;
                        self.pause_toggle_pending = false;
                    } else {
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
                    self.title_menu_selection_pending = Some(*cursor);
                    self.title_menu_selection_delay_frames = TITLE_TO_LEVEL_SELECTION_DELAY_FRAMES;
                } else if up_pressed && *cursor > 0 {
                    self.title_idle_frames = 0;
                    *cursor -= 1;
                    play_navigate_jingle = true;
                } else if down_pressed && *cursor < 2 {
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
                    self.title_menu_selection_pending = Some(*cursor);
                    self.title_menu_selection_delay_frames = TITLE_TO_LEVEL_SELECTION_DELAY_FRAMES;
                } else if up_pressed && *cursor > 0 {
                    self.title_idle_frames = 0;
                    *cursor -= 1;
                    play_navigate_jingle = true;
                } else if down_pressed && *cursor < 2 {
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
                if input.up && !self.prior_input.up && *cursor > 0 {
                    *cursor -= 1;
                }
                if input.down && !self.prior_input.down && *cursor < 3 {
                    *cursor += 1;
                }

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
                // ASM: "GAME OVER" display, then transition to continue screen or title.
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
                // ASM Continue_Screen: button press uses continue, timeout goes to title.
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
                    // Countdown expired without input — return to title
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
            return false;
        }

        true
    }
}

#[cfg(test)]
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
    fn sub_1b71a_returns_none_once_d2_reaches_the_terminal_guard() {
        let decoded_words = (0u16..(7 * 29) as u16).collect::<Vec<_>>();

        assert!(
            title_text_chameleon_frame_upload_sub_1B71A(&decoded_words, 0x22, 0).is_none(),
            "ASM sub_1B71A returns immediately once `cmpi.w #$22,d2` succeeds"
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
}
