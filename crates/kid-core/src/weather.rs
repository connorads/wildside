use super::*;

#[allow(dead_code)]
const MURDER_WALL_PATTERN_SUB_5E02: [u16; 4] = [0xA2FB, 0xA31F, 0xA343, 0xA367];

#[allow(non_snake_case)]
#[allow(dead_code)]
pub(crate) fn copy_murder_wall_palette_loc_140EE(
    source_palette: &[u16; 8],
    palette_buffer: &mut [u16; 0x40],
) {
    // ASM loc_140EE (kid.asm:26915): `moveq #7,d0` then `move.w (a0)+,(a1)+`
    // copies exactly eight palette words from `Pal_1408A` into
    // `Palette_Buffer+$20`, leaving every other palette slot untouched.
    palette_buffer[0x10..0x18].copy_from_slice(source_palette);
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct MurderWallWarningInputSub5E02 {
    pub(crate) murder_wall_enabled: bool,
    pub(crate) murder_wall_x_px: i16,
    pub(crate) murder_wall_reversed: bool,
    pub(crate) kid_hitbox_left_px: i16,
    pub(crate) kid_hitbox_right_px: i16,
    pub(crate) camera_x_px: i16,
    pub(crate) camera_y_px: u16,
    pub(crate) animation_timer: u8,
    pub(crate) animation_frame: u8,
    pub(crate) starting_number_sprites: u8,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct MurderWallWarningSpriteSub5E02 {
    pub(crate) y: u16,
    pub(crate) link: u16,
    pub(crate) tile_word: u16,
    pub(crate) x: u16,
}

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct MurderWallWarningRenderSub5E02 {
    pub(crate) push_animation_timer: Option<u16>,
    pub(crate) push_animation_offset: Option<u16>,
    pub(crate) animation_timer: u8,
    pub(crate) animation_frame: u8,
    pub(crate) number_sprites: u8,
    pub(crate) sprites: Vec<MurderWallWarningSpriteSub5E02>,
}

#[allow(non_snake_case)]
#[allow(dead_code)]
pub(crate) fn compile_murder_wall_warning_sprites_sub_5E02(
    input: MurderWallWarningInputSub5E02,
) -> MurderWallWarningRenderSub5E02 {
    // ASM sub_5E02 (kid.asm:5840): when the murder wall is active and within
    // `$30` pixels of the camera, write Kid's push animation words and emit a
    // 2x12 warning-tile strip into the shared sprite table.
    let mut render = MurderWallWarningRenderSub5E02 {
        push_animation_timer: None,
        push_animation_offset: None,
        animation_timer: input.animation_timer.wrapping_sub(1),
        animation_frame: input.animation_frame,
        number_sprites: input.starting_number_sprites,
        sprites: Vec::new(),
    };

    if !input.murder_wall_enabled {
        return render;
    }

    // ASM loc_5E94 (kid.asm:5901): subi.b #1,($FFFFFAC0).w; bne.s loc_5E94;
    // move.b #5,($FFFFFAC0).w; addi.b #1,($FFFFFABF).w; andi.b #3,($FFFFFABF).w
    // When the animation countdown reaches zero, reset to 5 and advance the
    // 2-bit pattern frame index.  At loc_5E94 itself, read the current frame,
    // double it, sign-extend, index unk_5DFA for the tile pattern word, load
    // Addr_NextSpriteSlot, and branch on MurderWall_reversed.
    if render.animation_timer == 0 {
        render.animation_timer = 5;
        render.animation_frame = render.animation_frame.wrapping_add(1) & 3;
    }

    if input.murder_wall_reversed {
        // ASM kid.asm:5847: addi.w #$110,d0; sub.w (Kid_hitbox_right).w,d0; ble.s loc_5E34
        let overlap =
            i32::from(input.murder_wall_x_px) + 0x110 - i32::from(input.kid_hitbox_right_px);
        if overlap <= 0 {
            // ASM loc_5E34: move.w #4,$38(a0) — set push animation timer
            render.push_animation_timer = Some(4);
        }
    } else {
        // ASM loc_5E24: addi.w #$30,d0; sub.w (Kid_hitbox_left).w,d0; blt.s loc_5E3A
        let overlap =
            i32::from(input.murder_wall_x_px) + 0x30 - i32::from(input.kid_hitbox_left_px);
        if overlap >= 0 {
            render.push_animation_offset = Some(0x30);
            // ASM loc_5E34 (fall-through): move.w #4,$38(a0)
            render.push_animation_timer = Some(4);
        }
    }

    // ASM loc_5E3A: move.w (Camera_X_pos).w,d3; sub.w (MurderWall_X_pos).w,d3;
    // tst.b (MurderWall_reversed).w; beq.s loc_5E4A; neg.w d3
    // cmpi.w #$30,d3; blt.s loc_5E52; rts
    let mut relative_wall_x = i32::from(input.camera_x_px) - i32::from(input.murder_wall_x_px);
    if input.murder_wall_reversed {
        relative_wall_x = -relative_wall_x;
    }
    if relative_wall_x >= 0x30 {
        return render;
    }

    // ASM loc_5E52 (kid.asm:5875): move.w #$80,d6
    // Reversed path: addi.w #$110,d6; add.w d3,d6 → $190 + d3
    // ASM loc_5E64 (kid.asm:5884): sub.w d3,d6 — non-reversed X = $80 - camera_distance
    let mut x = if input.murder_wall_reversed {
        0x190 + relative_wall_x
    } else {
        0x80 - relative_wall_x
    };
    // ASM loc_5E66 (kid.asm:5887): move.w #$80,d7; clr.l d5;
    // move.w (Camera_Y_pos).w,d5; divu.w #$30,d5; swap d5; sub.w d5,d7
    // DIVU puts quotient in low word, remainder in high word; SWAP moves
    // remainder to low word.  So d7 = $80 - (Camera_Y_pos % $30).
    let base_y = 0x80_i32 - i32::from(input.camera_y_px % 0x30);
    let pattern = MURDER_WALL_PATTERN_SUB_5E02[usize::from(render.animation_frame & 3)];

    // ASM loc_5EB2 (kid.asm:5912): non-reversed first column — 6 iterations
    // (d0=5, dbf), each emitting 2 sprites with tile offsets +0 and +9,
    // advancing y by $18 each sprite.  After the loop: restore base_y,
    // advance x by $18, reload d0=5 for the second column.
    // ASM loc_5F0C (kid.asm:5938): non-reversed second column — same 6x2
    // loop with tile offsets +$12 and +$1B.  After loop, stores a0 back to
    // Addr_NextSpriteSlot.  Reversed equivalents: loc_5F6A (col 1: +$812/+$81B)
    // and loc_5FCA (col 2: +$809/+$800).
    for column in 0..2 {
        let tile_offsets: [u16; 2] = if input.murder_wall_reversed {
            if column == 0 {
                [0x0812, 0x081B]
            } else {
                [0x0809, 0x0800]
            }
        } else if column == 0 {
            [0x0000, 0x0009]
        } else {
            [0x0012, 0x001B]
        };

        let mut y = base_y;
        for _ in 0..6 {
            for tile_offset in tile_offsets {
                render.number_sprites = render.number_sprites.wrapping_add(1);
                render.sprites.push(MurderWallWarningSpriteSub5E02 {
                    y: y as u16,
                    link: 0x0A00 + u16::from(render.number_sprites),
                    tile_word: pattern.wrapping_add(tile_offset),
                    x: x as u16,
                });
                y += 0x18;
            }
        }
        x += 0x18;
    }

    render
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum LevelSpecialEffectsRoutineSub6034 {
    WeatherSub6048,
}

#[allow(non_snake_case)]
#[allow(dead_code)]
pub(crate) fn dispatch_level_special_effects_sub_6034(
    level_special_effects: u16,
) -> Option<LevelSpecialEffectsRoutineSub6034> {
    // ASM sub_6034 (kid.asm:6032): `Level_Special_Effects` values 2 and 3
    // both dispatch through the local jump table to sub_6048; 0 and 1 return.
    let dispatch_index = level_special_effects.checked_sub(1)?;
    if dispatch_index == 0 {
        return None;
    }
    if dispatch_index <= 2 {
        return Some(LevelSpecialEffectsRoutineSub6034::WeatherSub6048);
    }
    None
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum StormArtAssetLoc14CCE {
    Rain,
    Hail,
}

#[allow(non_snake_case)]
#[allow(dead_code)]
pub(crate) fn select_storm_art_loc_14CCE(background_theme_is_ice: bool) -> StormArtAssetLoc14CCE {
    // ASM Init_SpecialEffect_Storm -> loc_14CCE (kid.asm:28339-28346): Ice
    // backgrounds branch away from the default rain art and load the hail art
    // pointer instead before the shared VRAM upload tail.
    if background_theme_is_ice {
        StormArtAssetLoc14CCE::Hail
    } else {
        StormArtAssetLoc14CCE::Rain
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct StormArtUploadLoc14CD8 {
    pub(crate) asset: StormArtAssetLoc14CCE,
    pub(crate) vram_word: u16,
}

#[allow(non_snake_case)]
#[allow(dead_code)]
pub(crate) fn upload_storm_art_loc_14CD8(asset: StormArtAssetLoc14CCE) -> StormArtUploadLoc14CD8 {
    // ASM loc_14CD8 (kid.asm:28347-28348): both the rain and hail branches
    // converge here and always decompress the previously selected art into
    // VRAM word offset `$7000`.
    StormArtUploadLoc14CD8 {
        asset,
        vram_word: 0x7000,
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum StormPaletteAssetLoc14CEC {
    Rain,
    Hail,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum StormSavedPaletteAssetLoc14D82 {
    RainPalette7B85C,
}

#[allow(non_snake_case)]
#[allow(dead_code)]
pub(crate) fn select_saved_storm_palette_loc_14D82(
    level_special_effects: u16,
) -> StormSavedPaletteAssetLoc14D82 {
    // ASM loc_14D82 (kid.asm:28420-28423): `cmpi.w #2,(Level_Special_Effects).w`
    // branches to the label for rain, but both the branch target and the
    // fallthrough write the same `LnkTo_Pal_7B85C` pointer into `($FFFFFADE).w`.
    debug_assert!(
        matches!(level_special_effects, 2 | 3),
        "loc_14D82 is only reached for storm weather"
    );
    let _ = level_special_effects;
    StormSavedPaletteAssetLoc14D82::RainPalette7B85C
}

#[allow(non_snake_case)]
#[allow(dead_code)]
pub(crate) fn select_storm_palette_loc_14CEC(
    background_theme_is_ice: bool,
) -> StormPaletteAssetLoc14CEC {
    // ASM loc_14CEC (kid.asm:28356-28358): the shared palette-selection tail
    // swaps from the default rain palette to the hail palette when the
    // background theme is Ice.
    if background_theme_is_ice {
        StormPaletteAssetLoc14CEC::Hail
    } else {
        StormPaletteAssetLoc14CEC::Rain
    }
}

#[allow(non_snake_case)]
#[allow(dead_code)]
pub(crate) fn copy_storm_palette_loc_14CF2(
    source_palette: &[u16; 8],
    palette_buffer: &mut [u16; 0x40],
) {
    // ASM loc_14CF2 (kid.asm:28360-28363): after choosing the rain or hail
    // palette pointer, `lea (Palette_Buffer+$20).l,a1` and four `move.l`
    // instructions copy exactly eight words into palette entries $10..$17.
    palette_buffer[0x10..0x18].copy_from_slice(source_palette);
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum StormPatternAssetLoc14D1E {
    Rain,
    Hail,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct StormPatternUploadSetupLoc14D1E {
    pub(crate) vram_tile_addend: u32,
    pub(crate) source_asset: StormPatternAssetLoc14D1E,
    pub(crate) skipped_header_longwords: usize,
}

#[allow(non_snake_case)]
#[allow(dead_code)]
pub(crate) fn select_storm_pattern_upload_loc_14D1E(
    background_theme_is_ice: bool,
) -> StormPatternUploadSetupLoc14D1E {
    // ASM loc_14CF2 -> loc_14D1E (kid.asm:28364-28385): after the palette
    // copy, both branches build `d7 = $0380_0380`, skip the first longword of
    // the selected table with `addq.w #4,a0`, and only differ in whether `a0`
    // comes from the rain (`LnkTo_unk_9B65C`) or hail (`LnkTo_unk_9B6E0`) data.
    StormPatternUploadSetupLoc14D1E {
        vram_tile_addend: 0x0380_0380,
        source_asset: if background_theme_is_ice {
            StormPatternAssetLoc14D1E::Hail
        } else {
            StormPatternAssetLoc14D1E::Rain
        },
        skipped_header_longwords: 1,
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct StormPatternUploadLoc14D32 {
    pub(crate) vdp_command: u32,
    pub(crate) written_longwords: Vec<u32>,
}

#[allow(non_snake_case)]
#[allow(dead_code)]
pub(crate) fn storm_pattern_outer_pass_count_loc_14D3C() -> usize {
    // ASM loc_14D3C (kid.asm:28390): `moveq #3,d2` falls through to the shared
    // upload loop, and `dbf d2,loc_14D3C` runs inclusively from 3 down to -1.
    4
}

#[allow(non_snake_case)]
#[allow(dead_code)]
pub(crate) fn storm_pattern_group_repeat_count_loc_14D3E() -> usize {
    // ASM loc_14D3E (kid.asm:28393): `moveq #7,d1` selects eight passes through
    // the group loop before `dbf d1,loc_14D3E` stops at -1.
    8
}

#[allow(non_snake_case)]
#[allow(dead_code)]
pub(crate) fn storm_pattern_emit_group_loc_14D40(
    source_group: &[u32; 4],
    vram_tile_addend: u32,
) -> [u32; 4] {
    // ASM loc_14D40 (kid.asm:28396-28405): load four consecutive longwords from
    // `(a0)`, add the shared `d7` tile offset to each, and write the adjusted
    // values to the VDP data port before the inclusive `dbf d0,loc_14D40`.
    source_group.map(|word| word.wrapping_add(vram_tile_addend))
}

#[allow(non_snake_case)]
#[allow(dead_code)]
pub(crate) fn upload_storm_pattern_blocks_loc_14D32(
    source_longwords: &[u32; 32],
    vram_tile_addend: u32,
) -> StormPatternUploadLoc14D32 {
    // ASM loc_14D32 (kid.asm:28387-28409): program `vdpComm($6000,VRAM,WRITE)`
    // and write the selected 32-longword storm table in 4 outer passes. Each
    // pass walks eight 4-longword groups; each group is emitted eight times
    // before `lea $10(a0),a0` advances to the next group, then `move.l a3,a0`
    // restarts the whole table for the next outer pass.
    const VDP_COMMAND: u32 = 0x4000_0003; // vdpComm($6000,VRAM,WRITE)
    let mut written_longwords = Vec::with_capacity(4 * 8 * 8 * 4);

    for _ in 0..storm_pattern_outer_pass_count_loc_14D3C() {
        for group in source_longwords.chunks_exact(4) {
            for _ in 0..storm_pattern_group_repeat_count_loc_14D3E() {
                let emitted_group = storm_pattern_emit_group_loc_14D40(
                    &[group[0], group[1], group[2], group[3]],
                    vram_tile_addend,
                );
                for word in emitted_group {
                    written_longwords.push(word);
                }
            }
        }
    }

    StormPatternUploadLoc14D32 {
        vdp_command: VDP_COMMAND,
        written_longwords,
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum StormSharedArtAssetLoc14D8A {
    ArtComp9A7D2,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct StormSlotFillSetupLoc14D8A {
    pub(crate) loop_count: u16,
    pub(crate) falling_slot_sentinel: u32,
    pub(crate) falling_slot_y_word: u16,
    pub(crate) impact_slot_sentinel: u32,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct StormInitSetupLoc14D8A {
    pub(crate) art_asset: StormSharedArtAssetLoc14D8A,
    pub(crate) art_vram_word: u16,
    pub(crate) fill_setup: StormSlotFillSetupLoc14D8A,
}

#[allow(non_snake_case)]
#[allow(dead_code)]
pub(crate) fn initialise_storm_setup_loc_14D8A() -> StormInitSetupLoc14D8A {
    // ASM loc_14D8A (kid.asm:28423-28433): load the shared `LnkTo_ArtComp_9A7D2`
    // weather art, decompress it to VRAM word `$76A0`, then seed the later
    // `loc_14DAC` fill loop with `d0=#4`, `d2=#$100`, and `d1=#$FFFF_FFFF`.
    StormInitSetupLoc14D8A {
        art_asset: StormSharedArtAssetLoc14D8A::ArtComp9A7D2,
        art_vram_word: 0x76A0,
        fill_setup: StormSlotFillSetupLoc14D8A {
            loop_count: 4,
            falling_slot_sentinel: 0xFFFF_FFFF,
            falling_slot_y_word: 0x0100,
            impact_slot_sentinel: 0xFFFF_FFFF,
        },
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct FallingWeatherInitSlotLoc14DAC {
    pub(crate) sentinel_long: u32,
    pub(crate) y_word: u16,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct WeatherImpactInitSlotLoc14DAC {
    pub(crate) sentinel_words: [u32; 3],
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct StormSlotInitLoc14DAC {
    pub(crate) falling_slots: [FallingWeatherInitSlotLoc14DAC; 5],
    pub(crate) impact_slots: [WeatherImpactInitSlotLoc14DAC; 5],
    pub(crate) spawn_cooldown: u8,
    pub(crate) free_falling_slots: u8,
    pub(crate) weather_mode_byte: u8,
    pub(crate) fade_countdown: u16,
    pub(crate) thunder_timer: i16,
    pub(crate) fb52_word: u16,
    pub(crate) fb54_word: u16,
}

#[allow(non_snake_case)]
#[allow(dead_code)]
pub(crate) fn initialise_storm_slots_loc_14DAC(
    setup: StormSlotFillSetupLoc14D8A,
) -> StormSlotInitLoc14DAC {
    // ASM loc_14DAC (kid.asm:28434-28448): `dbf d0,loc_14DAC` runs inclusively
    // from `d0=#4`, so the loop fills five falling-weather slots at `$FFFFFAE2`
    // with `[$FFFF_FFFF, $0100]`, five impact slots at `$FFFFFB00` with three
    // `#$FFFF_FFFF` longwords each, then seeds the worker state bytes/words that
    // sub_6048/sub_61CA consume immediately afterwards.
    let fill_count = usize::from(setup.loop_count) + 1;
    debug_assert_eq!(fill_count, 5, "loc_14DAC clears five weather slots");

    StormSlotInitLoc14DAC {
        falling_slots: core::array::from_fn(|_| FallingWeatherInitSlotLoc14DAC {
            sentinel_long: setup.falling_slot_sentinel,
            y_word: setup.falling_slot_y_word,
        }),
        impact_slots: core::array::from_fn(|_| WeatherImpactInitSlotLoc14DAC {
            sentinel_words: [setup.impact_slot_sentinel; 3],
        }),
        spawn_cooldown: 1,
        free_falling_slots: fill_count as u8,
        weather_mode_byte: 4,
        fade_countdown: 0x0100,
        thunder_timer: 1,
        fb52_word: 1,
        fb54_word: 0,
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct SpecialEffectUnknownArtUploadLoc14DF6 {
    pub(crate) written_longwords: Vec<u32>,
}

#[allow(non_snake_case)]
#[allow(dead_code)]
pub(crate) fn upload_special_effect_unknown_art_loc_14DF6(
    source_longwords: &[u32],
    dbf_counter: u16,
) -> SpecialEffectUnknownArtUploadLoc14DF6 {
    // ASM loc_14DF6 (kid.asm:28458-28465): after `move.w (a0)+,d0` and
    // `subq.w #1,d0`, each inclusive `dbf d0,loc_14DF6` iteration streams
    // eight consecutive longwords from `(a0)+` to the VDP data port.
    let burst_count = usize::from(dbf_counter) + 1;
    let longword_count = burst_count * 8;

    SpecialEffectUnknownArtUploadLoc14DF6 {
        written_longwords: source_longwords[..longword_count].to_vec(),
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct SpecialEffectUnknownPlaneFillOuterPassLoc14E42 {
    pub(crate) written_longwords: Vec<u32>,
}

fn special_effect_unknown_plane_seed_pair() -> (u32, u32) {
    // Matches the `move.l #$8380,d7` / `move.w d7,d6` / `swap d7` /
    // `move.w d6,d7` / `move.l d7,d6` / `addi.l` sequence between loc_14DF6 and
    // loc_14E42.
    (0x8380_8381, 0x8382_8383)
}

#[allow(non_snake_case)]
#[allow(dead_code)]
pub(crate) fn special_effect_unknown_group_count_loc_14E42() -> usize {
    // ASM loc_14E42 (kid.asm:28484): `moveq #3,d1` seeds the middle loop, so an
    // outer pass emits four consecutive 16-pair groups before the later
    // `move.l d3,d7` / `move.l d4,d6` reset.
    4
}

#[allow(non_snake_case)]
#[allow(dead_code)]
pub(crate) fn special_effect_unknown_inner_repeat_count_loc_14E44() -> usize {
    // ASM loc_14E44 (kid.asm:28487): `moveq #$F,d0` reloads the innermost loop
    // counter, so each group writes the current `d6/d7` pair 16 times before
    // the later `add.l d5,d6` / `add.l d5,d7` bump to the next pair.
    16
}

#[allow(non_snake_case)]
#[allow(dead_code)]
pub(crate) fn special_effect_unknown_pair_write_loc_14E46() -> [u32; 2] {
    // ASM loc_14E46 (kid.asm:28490): the innermost loop body writes the
    // current `d6` seed first, then the current `d7` seed, before `dbf d0`
    // decides whether another pair should follow.
    let (d6, d7) = special_effect_unknown_plane_seed_pair();
    [d6, d7]
}

#[allow(non_snake_case)]
#[allow(dead_code)]
pub(crate) fn special_effect_unknown_group_loc_14E44() -> Vec<u32> {
    let (d6, d7) = special_effect_unknown_plane_seed_pair();
    let mut written_longwords =
        Vec::with_capacity(special_effect_unknown_inner_repeat_count_loc_14E44() * 2);

    for _ in 0..special_effect_unknown_inner_repeat_count_loc_14E44() {
        written_longwords.push(d6);
        written_longwords.push(d7);
    }

    written_longwords
}

#[allow(non_snake_case)]
#[allow(dead_code)]
pub(crate) fn special_effect_unknown_outer_pass_loc_14E42(
    inner_repeat_count: usize,
) -> SpecialEffectUnknownPlaneFillOuterPassLoc14E42 {
    let (mut d6, mut d7) = special_effect_unknown_plane_seed_pair();
    let mut written_longwords =
        Vec::with_capacity(special_effect_unknown_group_count_loc_14E42() * inner_repeat_count * 2);

    for _ in 0..special_effect_unknown_group_count_loc_14E42() {
        for _ in 0..inner_repeat_count {
            written_longwords.push(d6);
            written_longwords.push(d7);
        }
        d6 = d6.wrapping_add(0x0004_0004);
        d7 = d7.wrapping_add(0x0004_0004);
    }

    SpecialEffectUnknownPlaneFillOuterPassLoc14E42 { written_longwords }
}

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum WeatherPaletteOpSub6048 {
    /// ASM loc_607A (kid.asm:6062): `move.w (a0)+,(a1)+; dbf d0,loc_607A`
    /// copies 32 words ($1F+1) from `Palette_Buffer` to `Palette_Buffer_2`
    /// (64 bytes = 4 palette lines × 16 colours).
    CopyPaletteBufferToBuffer2,
    /// ASM loc_6090 (kid.asm:6069): identical copy loop from `Palette_Buffer`
    /// to `Palette_Buffer_3`.
    CopyPaletteBufferToBuffer3,
    WritePaletteBuffer {
        start_word: u8,
        words: Vec<u16>,
    },
    WritePaletteBuffer3 {
        start_word: u8,
        words: Vec<u16>,
    },
}

const FALLING_WEATHER_SLOT_COUNT_SUB_61CA: usize = 5;
const WEATHER_IMPACT_SLOT_COUNT_SUB_61CA: usize = 10;
const FALLING_WEATHER_SPRITES_SUB_61CA: [u16; 6] = [0xA3B5, 0xA3B9, 0xA3BD, 0xA3C1, 0xA3C5, 0xA3C9];
const IMPACT_WEATHER_SPRITES_SUB_61CA: [u16; 2] = [0xA3CD, 0xA3D1];

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct FallingWeatherSlotSub61CA {
    pub(crate) y_px: i16,
    pub(crate) x_px: i16,
    pub(crate) animation_cooldown: u8,
    pub(crate) animation_frame: u8,
}

impl FallingWeatherSlotSub61CA {
    #[allow(dead_code)]
    pub(crate) const fn inactive(animation_cooldown: u8, animation_frame: u8) -> Self {
        Self {
            y_px: -1,
            x_px: -1,
            animation_cooldown,
            animation_frame,
        }
    }

    fn is_active(self) -> bool {
        self.y_px >= 0
    }

    fn clear_position(&mut self) {
        self.y_px = -1;
        self.x_px = -1;
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct WeatherImpactSlotSub61CA {
    pub(crate) y_px: i16,
    pub(crate) x_px: i16,
    pub(crate) animation_cooldown: u8,
    pub(crate) animation_frame: u8,
}

impl WeatherImpactSlotSub61CA {
    #[allow(dead_code)]
    pub(crate) const fn inactive(animation_cooldown: u8, animation_frame: u8) -> Self {
        Self {
            y_px: -1,
            x_px: -1,
            animation_cooldown,
            animation_frame,
        }
    }

    fn is_active(self) -> bool {
        self.y_px >= 0
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct WeatherCollisionRowsSub61CA {
    pub(crate) rows: Vec<Vec<u16>>,
}

impl WeatherCollisionRowsSub61CA {
    fn word_at(&self, row: usize, col: usize) -> u16 {
        self.rows
            .get(row)
            .and_then(|cells| cells.get(col))
            .copied()
            .unwrap_or(0)
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct WeatherSpriteSub61CA {
    pub(crate) y: u16,
    pub(crate) link: u16,
    pub(crate) tile_word: u16,
    pub(crate) x: u16,
}

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct WeatherWorkerInputSub61CA {
    pub(crate) camera_x_px: i16,
    pub(crate) camera_y_px: i16,
    pub(crate) spawn_cooldown: u8,
    pub(crate) free_falling_slots: u8,
    pub(crate) falling_slots: [FallingWeatherSlotSub61CA; FALLING_WEATHER_SLOT_COUNT_SUB_61CA],
    pub(crate) impact_slots: [WeatherImpactSlotSub61CA; WEATHER_IMPACT_SLOT_COUNT_SUB_61CA],
    pub(crate) kid_hitbox_top_px: i16,
    pub(crate) kid_hitbox_bottom_px: i16,
    pub(crate) kid_hitbox_left_px: i16,
    pub(crate) kid_hitbox_right_px: i16,
    pub(crate) random_byte: u8,
    pub(crate) collision_rows: WeatherCollisionRowsSub61CA,
    pub(crate) starting_number_sprites: u8,
}

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct WeatherWorkerOutputSub61CA {
    pub(crate) spawn_cooldown: u8,
    pub(crate) free_falling_slots: u8,
    pub(crate) falling_slots: [FallingWeatherSlotSub61CA; FALLING_WEATHER_SLOT_COUNT_SUB_61CA],
    pub(crate) impact_slots: [WeatherImpactSlotSub61CA; WEATHER_IMPACT_SLOT_COUNT_SUB_61CA],
    pub(crate) kid_collision_timer: Option<u16>,
    pub(crate) number_sprites: u8,
    pub(crate) sprites: Vec<WeatherSpriteSub61CA>,
}

#[allow(non_snake_case)]
#[allow(dead_code)]
#[allow(clippy::needless_pass_by_value, reason = "test callers pass by value")]
pub(crate) fn step_weather_worker_sub_61CA(
    input: WeatherWorkerInputSub61CA,
) -> WeatherWorkerOutputSub61CA {
    // ASM sub_61CA (kid.asm:6190): advance five falling weather slots,
    // respawn one free slot from the top camera row when `$FFFFFB3C` expires,
    // splash on terrain, hurt Kid on overlap, and emit splash/falling sprites.
    let mut output = WeatherWorkerOutputSub61CA {
        spawn_cooldown: input.spawn_cooldown,
        free_falling_slots: input.free_falling_slots,
        falling_slots: input.falling_slots,
        impact_slots: input.impact_slots,
        kid_collision_timer: None,
        number_sprites: input.starting_number_sprites,
        sprites: Vec::new(),
    };

    let delete_bottom_px = input.camera_y_px.wrapping_add(0x00E0);
    let delete_left_px = input.camera_x_px.wrapping_sub(0x0010);

    // ASM loc_61E2 (kid.asm:6199): per-slot loop — tst.w (a0); bmi.s loc_621C
    // skips inactive slots, then advances y/x and decrements animation counter.
    for slot in &mut output.falling_slots {
        if !slot.is_active() {
            continue;
        }

        // ASM loc_61E2: addi.w #4,(a0); subi.w #1,2(a0); subi.b #1,4(a0)
        slot.y_px = slot.y_px.wrapping_add(4);
        slot.x_px = slot.x_px.wrapping_sub(1);
        slot.animation_cooldown = slot.animation_cooldown.wrapping_sub(1);
        if slot.animation_cooldown == 0 {
            slot.animation_cooldown = 6;
            // ASM loc_620C (kid.asm:6211): addi.b #1,5(a0) — advance frame.
            // When frame >= 5, ASM sets 5(a0)=$FF then falls through to +1 = 0.
            slot.animation_frame = if slot.animation_frame >= 5 {
                0
            } else {
                slot.animation_frame.wrapping_add(1)
            };
        }

        // ASM loc_6212 (kid.asm:6214): cmp.w (a0),d1; blt.s loc_6226;
        // cmp.w 2(a0),d0; bgt.s loc_6226 — off-screen check deactivates slot.
        if delete_bottom_px < slot.y_px || delete_left_px > slot.x_px {
            slot.clear_position();
            output.free_falling_slots = output.free_falling_slots.wrapping_add(1);
        }
    }

    // ASM loc_623A (kid.asm:6232): subq.b #1,($FFFFFB3C).w; bne.w loc_62CE
    // Decrement spawn cooldown. When zero: reload to $25, subtract random 0–15
    // jitter, then check free_falling_slots before scanning for an inactive slot.
    // TODO(fidelity): ASM calls j_Get_RandomNumber_byte here and again at
    // loc_626C — two independent RNG draws. Native reuses one `random_byte` for
    // both cooldown jitter and spawn X position.
    output.spawn_cooldown = output.spawn_cooldown.wrapping_sub(1);
    if output.spawn_cooldown == 0 {
        output.spawn_cooldown = 0x25u8.wrapping_sub(input.random_byte & 0x0F);
        if output.free_falling_slots != 0 {
            // ASM loc_625E (kid.asm:6244): move.w (a0),d0; bmi.s loc_626C
            // Scan slots for the first inactive one (y_px < 0). If all active,
            // bra.s loc_62CE skips spawning entirely.
            for slot in &mut output.falling_slots {
                if slot.is_active() {
                    continue;
                }

                // ASM loc_626C (kid.asm:6252): spawn a new falling weather particle.
                // jsr Get_RandomNumber_byte; andi.w #$FF,d7; add.w Camera_X_pos,d7;
                // addi.w #$28,d7 → spawn_x. Camera_Y_pos → spawn_y.
                // Then tile collision check: row = (spawn_y+$F)>>4, col = spawn_x>>4.
                // First tile: word & $F must be 0. If spawn crosses a tile boundary
                // (col_ceil != col_floor), second tile: word & $FF must also be 0.
                // loc_62C2: store spawn_y/spawn_x to slot, decrement free count.
                let spawn_x_px = input
                    .camera_x_px
                    .wrapping_add(i16::from(input.random_byte))
                    .wrapping_add(0x0028);
                let spawn_y_px = input.camera_y_px;
                let row_index = usize::from(
                    (u16::from_be_bytes(spawn_y_px.to_be_bytes())).wrapping_add(0x000F) >> 4,
                );
                let col_floor = usize::from(u16::from_be_bytes(spawn_x_px.to_be_bytes()) >> 4);
                let col_ceil = usize::from(
                    u16::from_be_bytes(spawn_x_px.to_be_bytes()).wrapping_add(0x000F) >> 4,
                );
                let first_word = input.collision_rows.word_at(row_index, col_floor);
                // ASM loc_62AA (kid.asm:6275): first test masks the low nibble
                // of `(a4)+`; if the 16px spawn footprint crosses into the next
                // collision column, the second test masks that next word's low
                // byte before falling through to loc_62C2.
                let blocked_first_tile = first_word & 0x000F != 0;
                let crosses_tile = col_ceil != col_floor;
                let blocked_second_tile = crosses_tile
                    && (input.collision_rows.word_at(row_index, col_floor + 1) & 0x00FF != 0);

                if blocked_first_tile || blocked_second_tile {
                    break;
                }

                // ASM sub_61CA loc_62C2 only rewrites Y/X; the animation word
                // is intentionally preserved from the reused slot.
                slot.y_px = spawn_y_px;
                slot.x_px = spawn_x_px;
                output.free_falling_slots = output.free_falling_slots.wrapping_sub(1);
                break;
            }
        }
    }

    // ASM loc_62CE (kid.asm:6293): moveq #4,d5; move.l (Addr_GfxObject_Kid).w,a5;
    // move.l a1,a0 — sets up 5-slot collision scan (d5=4, dbf).
    //
    // ASM loc_62D6 (kid.asm:6298): per-slot Kid hitbox overlap test.
    //   move.w (a0),d0        ; slot y_px
    //   bmi.s  loc_630C       ; inactive (y < 0) → next slot
    //   addi.w #2,d0          ; top edge = y + 2
    //   cmp.w  (Kid_hitbox_top).w,d0
    //   blt.s  loc_630C       ; above Kid → skip
    //   addi.w #$C,d0         ; bottom edge = y + 2 + $C = y + $E
    //   cmp.w  (Kid_hitbox_bottom).w,d0
    //   bgt.s  loc_630C       ; below Kid → skip
    //   move.w 2(a0),d0       ; slot x_px
    //   addi.w #2,d0          ; left edge = x + 2
    //   cmp.w  (Kid_hitbox_right).w,d0
    //   bgt.s  loc_630C       ; right of Kid → skip
    //   addi.w #$C,d0         ; right edge = x + 2 + $C = x + $E
    //   cmp.w  (Kid_hitbox_left).w,d0
    //   blt.s  loc_630C       ; left of Kid → skip
    //   move.w #$28,$38(a5)   ; hit! set push timer on Kid
    //
    // ASM loc_630C (kid.asm:6316): lea 6(a0),a0; dbf d5,loc_62D6 — advance
    // to next slot (6 bytes per entry) and loop.
    for slot in &output.falling_slots {
        if !slot.is_active() {
            continue;
        }

        let y_plus_2 = slot.y_px.wrapping_add(2);
        if y_plus_2 < input.kid_hitbox_top_px {
            continue;
        }
        if y_plus_2.wrapping_add(0x000C) > input.kid_hitbox_bottom_px {
            continue;
        }
        let x_plus_2 = slot.x_px.wrapping_add(2);
        if x_plus_2 > input.kid_hitbox_right_px {
            continue;
        }
        if x_plus_2.wrapping_add(0x000C) < input.kid_hitbox_left_px {
            continue;
        }

        output.kid_collision_timer = Some(0x0028);
    }

    // ASM loc_6318 (kid.asm:6322): per-slot terrain collision loop — d5=4 (5 iters).
    //   lea ($FFFFFB00).w,a2  ; impact slot base
    //   clr.b d0               ; boundary-crossing flag (set but unused in splash path)
    //   move.w (a0),d7; bmi.s loc_6384  ; inactive → skip
    //   move.w 2(a0),d6                 ; x_px
    //   addi.w #$F,d7; lsr.w #4,d7; add.w d7,d7  ; row = (y+$F)>>4, word offset
    //   lea ($FFFF4A04).l,a4; move.w (a4,d7.w),a4 ; row pointer from collision map
    //   move.w d6,d7; addi.w #$F,d7; lsr.w #4,d7  ; col_ceil = (x+$F)>>4
    //   lsr.w #4,d6                                ; col_floor = x>>4
    //   cmp.w d6,d7; beq.s loc_6348; st d0         ; set flag if crosses boundary
    // The boundary flag (d0) is computed but never consumed in the splash path —
    // it appears to be dead code in the original ROM.
    for slot in &mut output.falling_slots {
        if !slot.is_active() {
            continue;
        }

        let row_index =
            usize::from(u16::from_be_bytes(slot.y_px.to_be_bytes()).wrapping_add(0x000F) >> 4);
        let col_floor = usize::from(u16::from_be_bytes(slot.x_px.to_be_bytes()) >> 4);
        // ASM loc_6348 (kid.asm:6341): add.w d6,d6; add.w d6,a4; move.w (a4)+,d7;
        //   andi.w #$7000,d7; beq.s loc_6384      ; terrain_kind == 0 → no collision
        //   cmpi.w #$4000,d7; beq.s loc_638E       ; slope terrain → offset splash
        //   cmpi.w #$5000,d7; beq.s loc_638E       ; slope terrain → offset splash
        //   (fall through to loc_6360)              ; normal terrain → direct splash
        let collision_word = input.collision_rows.word_at(row_index, col_floor);
        let terrain_kind = collision_word & 0x7000;
        if terrain_kind == 0 {
            continue;
        }

        // ASM loc_6360 (kid.asm:6352): scan impact slots for first free (y < 0).
        //   tst.w (a2); bmi.s loc_636A; lea 6(a2),a2; bra.s loc_6360
        // loc_636A: move.w (a0),(a2); move.w 2(a0),2(a2); move.w #$600,4(a2)
        //   move.l #$FFFFFFFF,(a0); addi.b #1,($FFFFFB3D).w
        // For $4000/$5000 terrain (loc_638E → loc_6398, kid.asm:6379):
        //   same scan loop, then move.w (a0),(a2); addi.w #$C,(a2) — y offset +$C.
        //   move.w 2(a0),2(a2); move.w #$600,4(a2); move.l #$FFFFFFFF,(a0);
        //   addi.b #1,($FFFFFB3D).w; lea 6(a0),a0; dbf d5,loc_6318.
        if let Some(impact_slot) = output
            .impact_slots
            .iter_mut()
            .find(|slot| !slot.is_active())
        {
            impact_slot.y_px = if terrain_kind == 0x4000 || terrain_kind == 0x5000 {
                slot.y_px.wrapping_add(0x000C)
            } else {
                slot.y_px
            };
            impact_slot.x_px = slot.x_px;
            impact_slot.animation_cooldown = 6;
            impact_slot.animation_frame = 0;
        }

        slot.clear_position();
        output.free_falling_slots = output.free_falling_slots.wrapping_add(1);
    }

    // ASM loc_63BE (kid.asm:6389): lea ($FFFFFB00).w,a0; moveq #9,d5
    //   Set up impact slot loop: base address $FFFFFB00, 10 iterations (d5=9, dbf).
    // ASM loc_63C4 (kid.asm:6393): per-slot impact update loop.
    //   move.w (a0),d7; bmi.w loc_6456       ; inactive → skip
    //   subi.b #1,4(a0); bne.s loc_6402      ; decrement cooldown, render if nonzero
    //   subi.w #4,(a0); subi.w #4,2(a0)      ; y -= 4, x -= 4
    //   move.b #6,4(a0)                      ; reload cooldown = 6
    //   addi.b #1,5(a0); cmpi.b #2,5(a0)     ; advance frame, check >= 2
    //   blt.s loc_6402                        ; < 2 → render
    //   move.l #$FFFFFFFF,(a0); move.w #$100,4(a0); bra.s loc_6456 ; deactivate
    for slot in &mut output.impact_slots {
        if !slot.is_active() {
            continue;
        }

        slot.animation_cooldown = slot.animation_cooldown.wrapping_sub(1);
        if slot.animation_cooldown == 0 {
            slot.y_px = slot.y_px.wrapping_sub(4);
            slot.x_px = slot.x_px.wrapping_sub(4);
            slot.animation_cooldown = 6;
            slot.animation_frame = slot.animation_frame.wrapping_add(1);
            if slot.animation_frame >= 2 {
                slot.y_px = -1;
                slot.x_px = -1;
                slot.animation_cooldown = 1;
                slot.animation_frame = 0;
                continue;
            }
        }

        // ASM loc_6402 (kid.asm:6410): screen-space conversion + sprite emission.
        //   loc_6432: link base = $500 (frame 0) or $A00 (frame 1).
        //   clr.w d6; tst.b 5(a0); bne.s loc_642C; ... loc_643E: addi.w #$500,d6
        //   Frame 0: 0+$500=$500 (2×2 cells). Frame 1: $500+$500=$A00 (3×3 cells).
        let sprite_y = slot
            .y_px
            .wrapping_sub(input.camera_y_px)
            .wrapping_add(0x0080);
        let sprite_x = slot
            .x_px
            .wrapping_sub(input.camera_x_px)
            .wrapping_add(0x0080);
        let frame_index = usize::from(slot.animation_frame.min(1));
        let link_base: u16 = if slot.animation_frame != 0 {
            0x0A00
        } else {
            0x0500
        };
        output.number_sprites = output.number_sprites.wrapping_add(1);
        output.sprites.push(WeatherSpriteSub61CA {
            y: u16::from_be_bytes(sprite_y.to_be_bytes()),
            link: link_base.wrapping_add(u16::from(output.number_sprites)),
            tile_word: IMPACT_WEATHER_SPRITES_SUB_61CA[frame_index],
            x: u16::from_be_bytes(sprite_x.to_be_bytes()),
        });
    }

    // ASM loc_6462 (kid.asm:6447): per-slot falling weather sprite rendering.
    //   move.w (a0),d7; bmi.s loc_64AE          ; inactive (y < 0) → skip
    //   move.w 2(a0),d6                          ; x from slot
    //   sub.w Camera_Y_pos,d7; addi.w #$80,d7   ; world → screen Y
    //   sub.w Camera_X_pos,d6; addi.w #$80,d6   ; world → screen X
    //   move.l Addr_NextSpriteSlot,a4            ; sprite output pointer
    //   move.w d6,6(a4); move.w d7,0(a4)        ; store screen x/y
    //   move.w 4(a0),d0; andi.w #$FF,d0         ; animation_frame byte (offset 5)
    //   add.w d0,d0; move.w unk_64BC(pc,d0.w),4(a4)  ; tile from lookup table
    //   move.w #$500,d6                          ; link base = $500 (1×1 cell)
    //   addq.b #1,Number_Sprites; add.b Number_Sprites,d6
    //   move.w d6,2(a4)                          ; store link
    //   lea 8(a4),a4; move.l a4,Addr_NextSpriteSlot
    // ASM loc_64AE (kid.asm:6469): lea 6(a0),a0; dbf d5,loc_6462; rts
    //   Advance to next falling slot (6 bytes each), loop 5 slots (d5=4, dbf).
    for slot in &output.falling_slots {
        if !slot.is_active() {
            continue;
        }

        let sprite_y = slot
            .y_px
            .wrapping_sub(input.camera_y_px)
            .wrapping_add(0x0080);
        let sprite_x = slot
            .x_px
            .wrapping_sub(input.camera_x_px)
            .wrapping_add(0x0080);
        let frame_index = usize::from(slot.animation_frame.min(5));
        output.number_sprites = output.number_sprites.wrapping_add(1);
        output.sprites.push(WeatherSpriteSub61CA {
            y: u16::from_be_bytes(sprite_y.to_be_bytes()),
            link: 0x0500u16.wrapping_add(u16::from(output.number_sprites)),
            tile_word: FALLING_WEATHER_SPRITES_SUB_61CA[frame_index],
            x: u16::from_be_bytes(sprite_x.to_be_bytes()),
        });
    }

    output
}

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct WeatherSpecialEffectsStepSub6048 {
    pub(crate) frame_counter: u16,
    pub(crate) level_special_effects: u16,
    pub(crate) foreground_theme_is_mountain: bool,
    pub(crate) background_theme_is_mountain: bool,
    pub(crate) fade_countdown: u16,
    pub(crate) fade_progress_latch: u16,
    pub(crate) palette_animation_speed: u8,
    pub(crate) pending_palette_mask_0: i16,
    pub(crate) weather_mode_byte: u8,
    pub(crate) thunder_timer: i16,
    pub(crate) hail_period_timer: u16,
    pub(crate) hail_active: bool,
    pub(crate) snow_spawn_cooldown: u8,
    pub(crate) snow_slot_count: u8,
    pub(crate) play_thunder_sound: bool,
    pub(crate) call_snow_worker: bool,
    pub(crate) palette_ops: Vec<WeatherPaletteOpSub6048>,
}

#[allow(non_snake_case)]
#[allow(dead_code)]
pub(crate) fn step_level_special_effects_sub_6048(
    mut state: WeatherSpecialEffectsStepSub6048,
    saved_storm_palette: [u16; 8],
    mountain_storm_foreground_palette: [u16; 15],
    thunder_flash_palette: [u16; 7],
    thunder_calm_palette: [u16; 7],
    random_byte: u8,
) -> WeatherSpecialEffectsStepSub6048 {
    // ASM sub_6048 (kid.asm:6050): drive storm palette fades, Mountain
    // thunder flashes, and the later snow/hail cadence before setting
    // `($FFFFFAD6).w=4`.
    state.fade_progress_latch = state.fade_countdown;
    state.play_thunder_sound = false;
    state.call_snow_worker = false;
    state.palette_ops.clear();

    if state.frame_counter <= 0x04B0 {
        state.weather_mode_byte = 4;
        return state;
    }

    if state.fade_countdown != 0 {
        state.palette_animation_speed = 0x7F;
        state.pending_palette_mask_0 = 0x0080;
        state
            .palette_ops
            .push(WeatherPaletteOpSub6048::CopyPaletteBufferToBuffer2);
        state
            .palette_ops
            .push(WeatherPaletteOpSub6048::CopyPaletteBufferToBuffer3);

        if state.foreground_theme_is_mountain {
            // ASM loc_60C0 (kid.asm:6088): Mountain path — first word from
            // saved storm palette goes to Palette_Buffer_3[0].
            state.pending_palette_mask_0 = -1;
            state
                .palette_ops
                .push(WeatherPaletteOpSub6048::WritePaletteBuffer3 {
                    start_word: 0,
                    words: vec![saved_storm_palette[0]],
                });
            // ASM loc_60C6 (kid.asm:6092): `move.w (a0)+,(a1)+; dbf d0,loc_60C6`
            // with d0=6 — copies 7 remaining words to Palette_Buffer_3+$32.
            state
                .palette_ops
                .push(WeatherPaletteOpSub6048::WritePaletteBuffer3 {
                    start_word: 0x19,
                    words: saved_storm_palette[1..].to_vec(),
                });
            // ASM loc_60E0 (kid.asm:6101): `moveq #$E,d0` then `move.w
            // (a0)+,(a1)+; dbf d0,loc_60E0` copies all 15 Mountain foreground
            // overlay palette words to `Palette_Buffer_3+2`.
            state
                .palette_ops
                .push(WeatherPaletteOpSub6048::WritePaletteBuffer3 {
                    start_word: 1,
                    words: mountain_storm_foreground_palette.to_vec(),
                });
        } else {
            // ASM loc_60AA (kid.asm:6078): non-Mountain path —
            // `move.w (a0)+,(a1)+; dbf d0,loc_60AA` with d0=6 copies 7 words
            // to Palette_Buffer_3+$32, then clears Palette_Buffer_3[0].
            state
                .palette_ops
                .push(WeatherPaletteOpSub6048::WritePaletteBuffer3 {
                    start_word: 0x19,
                    words: saved_storm_palette[..7].to_vec(),
                });
            state
                .palette_ops
                .push(WeatherPaletteOpSub6048::WritePaletteBuffer3 {
                    start_word: 0,
                    words: vec![0],
                });
        }

        state.fade_countdown = state.fade_countdown.wrapping_sub(4);
        state.weather_mode_byte = 4;
        return state;
    }

    // ASM loc_60EE (kid.asm:6108): Mountain thunder guard —
    // cmpi.w #WeatherID_Storm_and_Hail,(Level_Special_Effects).w; bgt.s loc_6172
    // cmpi.w #Mountain,(Background_theme).w; bne.s loc_6172
    // cmpi.w #$5DC,d6; bls.w loc_61C2
    // subi.w #1,($FFFFFADC).w; bmi.s loc_613E; bne.s loc_6172
    if state.level_special_effects <= 3
        && state.background_theme_is_mountain
        && state.frame_counter > 0x05DC
    {
        state.thunder_timer = state.thunder_timer.wrapping_sub(1);
        if state.thunder_timer == 0 {
            // ASM loc_60EE fall-through (kid.asm:6118): timer==0 → play thunder,
            // load flash palette (LnkTo_Pal_7B86C) into Palette_Buffer[0] and
            // Palette_Buffer+$32 via loc_6136 copy loop.
            state.play_thunder_sound = true;
            state
                .palette_ops
                .push(WeatherPaletteOpSub6048::WritePaletteBuffer {
                    start_word: 0,
                    words: vec![thunder_flash_palette[0]],
                });
            // ASM loc_6136 (kid.asm:6128): `move.w (a0)+,(a1)+; dbf d0,loc_6136`
            // with d0=6 — copies 7 words of flash palette to Palette_Buffer+$32.
            state
                .palette_ops
                .push(WeatherPaletteOpSub6048::WritePaletteBuffer {
                    start_word: 0x19,
                    words: thunder_flash_palette[1..].to_vec(),
                });
        } else if state.thunder_timer == -27 {
            // ASM loc_613E (kid.asm:6134): cmpi.w #$FFE5,($FFFFFADC).w; bne.s loc_6172
            // Reseed timer: move.w #$F7,($FFFFFADC).w; random & $F5 subtracted.
            // Load calm palette (LnkTo_Pal_7B85C) into Palette_Buffer[0] and
            // Palette_Buffer+$32 via loc_616C copy loop (same pattern as loc_6136).
            state.thunder_timer = 0x00F7 - i16::from(random_byte & 0xF5);
            state
                .palette_ops
                .push(WeatherPaletteOpSub6048::WritePaletteBuffer {
                    start_word: 0,
                    words: vec![thunder_calm_palette[0]],
                });
            state
                .palette_ops
                .push(WeatherPaletteOpSub6048::WritePaletteBuffer {
                    start_word: 0x19,
                    words: thunder_calm_palette[1..].to_vec(),
                });
        }
    }

    // ASM loc_6172 (kid.asm:6151): cmpi.w #$834,d6; bls.s loc_61C2
    // also gates this block, but its early-exit just sets weather_mode_byte=4
    // (which we always emit at the end of the function) and the second gate
    // `cmpi.w #$873,(Frame_Counter)` strictly dominates it. Collapse the two
    // checks to a single binding boundary so a `> with >=` mutant on the
    // weaker $834 check cannot survive as an equivalent mutant.
    if state.level_special_effects == 3 && state.frame_counter > 0x0873 {
        state.hail_period_timer = state.hail_period_timer.wrapping_sub(1);
        if state.hail_period_timer == 0 {
            // ASM loc_619C (kid.asm:6163): addi.w #$258,($FFFFFB52).w;
            // eori.b #$FF,($FFFFFB54).w — when inactive→active the pre-set
            // $258 is added to give $4B0; when active→inactive only the $258
            // from loc_619C applies.
            state.hail_period_timer = if state.hail_active { 0x0258 } else { 0x04B0 };
            state.hail_active = !state.hail_active;
        }

        // ASM loc_61A8 (kid.asm:6167): tst.b ($FFFFFB54).w; bne.s loc_61BC
        // — active period always falls through to sub_61CA.
        // Inactive period: cmpi.b #5,($FFFFFB3D).w; beq.s return_61C0
        // — full slots (5) skip sub_61CA entirely.
        if state.hail_active {
            // ASM loc_61BC (kid.asm:6174): bsr.w sub_61CA
            state.call_snow_worker = true;
        } else if state.snow_slot_count != 5 {
            state.snow_spawn_cooldown = 2;
            // ASM loc_61BC (kid.asm:6174): bsr.w sub_61CA
            state.call_snow_worker = true;
        }
        // else: ASM beq.s return_61C0 — inactive with all 5 slots, no sub_61CA
    }

    state.weather_mode_byte = 4;
    state
}

#[derive(Clone, Copy)]
struct LavaFireballSegment {
    frames: u8,
    delta_y: i16,
    sprite_kind: LavaFireballSpriteKind,
}

#[derive(Clone, Copy)]
pub(crate) enum LavaFireballSpriteKind {
    TallRise,
    MidRise,
    MidBody,
    MidFall,
    TallFall,
}

const LAVA_FIREBALL_SCRIPT: [LavaFireballSegment; 11] = [
    LavaFireballSegment {
        frames: 0x0B,
        delta_y: -4,
        sprite_kind: LavaFireballSpriteKind::TallRise,
    },
    LavaFireballSegment {
        frames: 0x0B,
        delta_y: -3,
        sprite_kind: LavaFireballSpriteKind::TallRise,
    },
    LavaFireballSegment {
        frames: 0x0B,
        delta_y: -3,
        sprite_kind: LavaFireballSpriteKind::MidRise,
    },
    LavaFireballSegment {
        frames: 0x0B,
        delta_y: -2,
        sprite_kind: LavaFireballSpriteKind::MidRise,
    },
    LavaFireballSegment {
        frames: 0x09,
        delta_y: -1,
        sprite_kind: LavaFireballSpriteKind::MidBody,
    },
    LavaFireballSegment {
        frames: 0x05,
        delta_y: 0,
        sprite_kind: LavaFireballSpriteKind::MidBody,
    },
    LavaFireballSegment {
        frames: 0x05,
        delta_y: 0,
        sprite_kind: LavaFireballSpriteKind::MidBody,
    },
    LavaFireballSegment {
        frames: 0x09,
        delta_y: 1,
        sprite_kind: LavaFireballSpriteKind::MidBody,
    },
    LavaFireballSegment {
        frames: 0x0B,
        delta_y: 2,
        sprite_kind: LavaFireballSpriteKind::MidFall,
    },
    LavaFireballSegment {
        frames: 0x0B,
        delta_y: 2,
        sprite_kind: LavaFireballSpriteKind::TallFall,
    },
    LavaFireballSegment {
        frames: 0x15,
        delta_y: 4,
        sprite_kind: LavaFireballSpriteKind::TallFall,
    },
];

#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn kid_hitbox_bounds_px_at_screen(
    kid_x_px: i32,
    kid_foot_y_px: i32,
) -> (i32, i32, i32, i32) {
    // ASM sub_14A58/sub_14AB0 read the four `Kid_hitbox_*` words straight from
    // RAM. Native rebuilds them from the player's screen-pixel position by
    // subtracting `PLAYER_HALF_WIDTH = 5` from `x_px` for the left column,
    // adding the same five for the right column, and subtracting
    // `PLAYER_HEIGHT = 31` from the foot row to get the head row. The bottom
    // row sits exactly on the foot pixel — Kid is anchored at his feet.
    (
        kid_x_px - PLAYER_HALF_WIDTH,
        kid_x_px + PLAYER_HALF_WIDTH,
        kid_foot_y_px - PLAYER_HEIGHT,
        kid_foot_y_px,
    )
}

fn kid_hitbox_bounds_px(slice: &OpeningSliceState) -> (i32, i32, i32, i32) {
    let x_px = (slice.player.x_pos >> 16) as i32;
    let foot_y_px = (slice.player.y_pos >> 16) as i32;
    kid_hitbox_bounds_px_at_screen(x_px, foot_y_px)
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct LavaGeyserHitboxWindowLoc14AAA {
    pub(crate) left_px: i32,
    pub(crate) right_px: i32,
    pub(crate) top_px: i32,
    pub(crate) bottom_px: i32,
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn lava_geyser_hitbox_window_loc_14AAA(
    geyser_x_px: u16,
    geyser_base_y_px: u16,
    geyser_current_height: u16,
) -> Option<LavaGeyserHitboxWindowLoc14AAA> {
    if geyser_current_height == 0 {
        return None;
    }

    let left_px = i32::from(geyser_x_px) + 3;
    let bottom_px = i32::from(geyser_base_y_px);
    // ASM sub_14A58 -> loc_14AAA (kid.asm:28116-28137): each failed compare
    // branches straight to `loc_14AAA`, so damage only applies when Kid stays
    // within the inclusive X window `[x+3, x+3+$1A]` and Y window
    // `[top_sprite_y+$10, base_y]`.
    Some(LavaGeyserHitboxWindowLoc14AAA {
        left_px,
        right_px: left_px + 0x1A,
        top_px: bottom_px - i32::from(geyser_current_height) + 0x10,
        bottom_px,
    })
}

#[allow(non_snake_case)]
fn lava_geyser_hits_kid_sub_14A58(geyser: &LavaGeyser, slice: &OpeningSliceState) -> bool {
    let Some(hitbox_window) =
        lava_geyser_hitbox_window_loc_14AAA(geyser.x_px, geyser.y_px, geyser.current_height)
    else {
        return false;
    };

    let (kid_left, kid_right, kid_top, kid_bottom) = kid_hitbox_bounds_px(slice);

    kid_right >= hitbox_window.left_px
        && kid_left <= hitbox_window.right_px
        && kid_top <= hitbox_window.bottom_px
        && kid_bottom >= hitbox_window.top_px
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct LavaGeyserSpriteTemplateLoc14BE8 {
    pub(crate) x_offset_px: u16,
    pub(crate) y_step_px: u16,
    pub(crate) size_word: u16,
    pub(crate) tile_base_word: u16,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct LavaGeyserSpriteChunkLoc14BE8 {
    pub(crate) screen_y_with_bias: u16,
    pub(crate) tile_word: u16,
    pub(crate) size_word: u16,
    pub(crate) screen_x_px: u16,
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn lava_geyser_sprite_chunks_loc_14BE8(
    base_x_world_px: u16,
    camera_x_px: u16,
    initial_screen_y_with_bias: u16,
    number_sprites: u8,
    templates: &[LavaGeyserSpriteTemplateLoc14BE8],
) -> Vec<LavaGeyserSpriteChunkLoc14BE8> {
    // ASM loc_14BE8 (kid.asm:28262): each row copies the current geyser base X
    // through `x_pos - Camera_X_pos + entry_x_offset`, subtracts the entry's
    // Y step from the running biased screen Y, increments `Number_Sprites`
    // before biasing the tile word, and writes the row to the next sprite slot.
    let mut sprite_rows = Vec::with_capacity(templates.len());
    let mut screen_y_with_bias = initial_screen_y_with_bias;
    let mut sprite_count = number_sprites;

    for template in templates {
        screen_y_with_bias = screen_y_with_bias.wrapping_sub(template.y_step_px);
        sprite_count = sprite_count.wrapping_add(1);
        sprite_rows.push(LavaGeyserSpriteChunkLoc14BE8 {
            screen_y_with_bias,
            tile_word: template
                .tile_base_word
                .wrapping_add(u16::from(sprite_count)),
            size_word: template.size_word,
            screen_x_px: base_x_world_px
                .wrapping_sub(camera_x_px)
                .wrapping_add(template.x_offset_px),
        });
    }

    sprite_rows
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct LavaFireballVerticalWindowLoc14AFC {
    pub(crate) top_px: i32,
    pub(crate) bottom_px: i32,
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn lava_fireball_vertical_window_loc_14AFC(
    baseline_y_px: u16,
    sprite_kind: LavaFireballSpriteKind,
) -> LavaFireballVerticalWindowLoc14AFC {
    let baseline_y = i32::from(baseline_y_px);
    // ASM sub_14AB0 -> loc_14AFC (kid.asm:28165-28177): the `a2 < unk_1466E`
    // bucket subtracts 7 more from `d7` after the shared `subq.w #1,d7`,
    // yielding the tall-rise bottom row at `baseline_y - 8`. The remaining
    // buckets follow the neighbouring branches at `loc_14B00` / `loc_14B04`
    // or fall through unchanged.
    let (top_px, bottom_px) = match sprite_kind {
        LavaFireballSpriteKind::TallRise => (baseline_y - 23, baseline_y - 8),
        LavaFireballSpriteKind::MidRise => (baseline_y - 18, baseline_y - 1),
        LavaFireballSpriteKind::MidBody => (baseline_y - 7, baseline_y - 1),
        LavaFireballSpriteKind::MidFall => (baseline_y - 12, baseline_y - 1),
        LavaFireballSpriteKind::TallFall => (baseline_y - 23, baseline_y + 6),
    };
    LavaFireballVerticalWindowLoc14AFC { top_px, bottom_px }
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn lava_fireball_rejects_kid_loc_14B22(
    kid_left_px: i32,
    kid_right_px: i32,
    kid_top_px: i32,
    kid_bottom_px: i32,
    fireball_left_px: i32,
    fireball_right_px: i32,
    vertical_window: LavaFireballVerticalWindowLoc14AFC,
) -> bool {
    // ASM loc_14B22 (kid.asm:28197): every failed compare in the overlap gate
    // branches here, restores registers, and returns without applying damage.
    kid_right_px < fireball_left_px
        || kid_left_px > fireball_right_px
        || kid_top_px > vertical_window.bottom_px
        || kid_bottom_px < vertical_window.top_px
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn lava_fireball_overlap_gate_loc_14B06(
    kid_left_px: i32,
    kid_right_px: i32,
    kid_top_px: i32,
    kid_bottom_px: i32,
    fireball_left_px: i32,
    fireball_right_px: i32,
    vertical_window: LavaFireballVerticalWindowLoc14AFC,
) -> bool {
    // ASM loc_14B06 (kid.asm:28187): after the sprite-kind buckets finish
    // shaping `d6/d7`, the fireball only damages Kid when all four inclusive
    // overlap compares succeed.
    !lava_fireball_rejects_kid_loc_14B22(
        kid_left_px,
        kid_right_px,
        kid_top_px,
        kid_bottom_px,
        fireball_left_px,
        fireball_right_px,
        vertical_window,
    )
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn lava_fireball_hitbox_x_window_loc_14AB0(fireball_x_px: u16) -> (i32, i32) {
    // ASM sub_14AB0 (kid.asm:28151-28157): `move.w $44(a5),d6; subi.w #$80,d6;
    // addq.w #1,d6` shapes the inclusive left column after the shared sprite
    // bias subtract — native stores `fireball.x_px` already unbiased so only
    // the `+1` needs replaying. `addi.w #6,d6` then advances to the inclusive
    // right column before the second `bgt.s loc_14B22` reject.
    let left = i32::from(fireball_x_px) + 1;
    let right = left + 6;
    (left, right)
}

#[allow(non_snake_case)]
fn lava_fireball_hits_kid_sub_14AB0(
    fireball: &LavaFireball,
    slice: &OpeningSliceState,
    sprite_kind: LavaFireballSpriteKind,
) -> bool {
    let (kid_left, kid_right, kid_top, kid_bottom) = kid_hitbox_bounds_px(slice);
    let (fireball_left, fireball_right) = lava_fireball_hitbox_x_window_loc_14AB0(fireball.x_px);
    let vertical_window =
        lava_fireball_vertical_window_loc_14AFC(fireball.current_y_px, sprite_kind);

    lava_fireball_overlap_gate_loc_14B06(
        kid_left,
        kid_right,
        kid_top,
        kid_bottom,
        fireball_left,
        fireball_right,
        vertical_window,
    )
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn lava_geyser_initial_delay_frames_loc_14772(encoded_height_word: u16) -> u16 {
    // ASM Obj_Lava_Geyser -> loc_14772 (kid.asm:27810-27820): the low nibble of
    // the encoded height indexes `word_14710`, whose entries are `0, 10, 20,
    // ... 150`. `loc_14772` then hibernates once per stored frame before the
    // eruption continues into the sound/effect script.
    (encoded_height_word & 0xF) * 10
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct LavaGeyserPaletteLoopLoc14B2A {
    pub(crate) hibernation_frames: u16,
    pub(crate) reloaded_delay_counter: u16,
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn rotate_lava_geyser_palette_loc_14B2A(
    palette_buffer: &mut [u16; 0x40],
) -> LavaGeyserPaletteLoopLoc14B2A {
    const LAVA_GEYSER_PALETTE_START_LOC_14B2A: usize = 0x14;

    // ASM loc_14B2A (kid.asm:28209): the geyser animator hibernates for five
    // frames, reloads `d0 = #5`, then rotates palette words `$28/$2A/$2C/$2E`
    // upward so `$2E` wraps back into `$28` before looping forever.
    let saved = palette_buffer[LAVA_GEYSER_PALETTE_START_LOC_14B2A + 3];
    palette_buffer[LAVA_GEYSER_PALETTE_START_LOC_14B2A + 3] =
        palette_buffer[LAVA_GEYSER_PALETTE_START_LOC_14B2A + 2];
    palette_buffer[LAVA_GEYSER_PALETTE_START_LOC_14B2A + 2] =
        palette_buffer[LAVA_GEYSER_PALETTE_START_LOC_14B2A + 1];
    palette_buffer[LAVA_GEYSER_PALETTE_START_LOC_14B2A + 1] =
        palette_buffer[LAVA_GEYSER_PALETTE_START_LOC_14B2A];
    palette_buffer[LAVA_GEYSER_PALETTE_START_LOC_14B2A] = saved;

    LavaGeyserPaletteLoopLoc14B2A {
        hibernation_frames: 5,
        reloaded_delay_counter: 5,
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct LavaFireballInitStateLoc1499C {
    pub(crate) startup_delay_frames: u16,
    pub(crate) rise_height_steps: u16,
    pub(crate) biased_x_px: u16,
    pub(crate) biased_y_px: u16,
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn initialise_lava_fireball_state_loc_1499C(
    x_px: u16,
    y_px: u16,
    encoded_height_word: u16,
) -> LavaFireballInitStateLoc1499C {
    // ASM loc_1499C (kid.asm:28038): the USM2 fireball clears the sign bit from
    // `$1A(a5)`, uses the low nibble as a `word_14710` index for `$4E(a5)`,
    // shifts the remaining height into `$4A(a5)`, then stores `$44/$46(a5)`
    // with the usual `$80` sprite bias.
    let untagged_height_word = encoded_height_word & 0x7FFF;
    LavaFireballInitStateLoc1499C {
        startup_delay_frames: lava_geyser_initial_delay_frames_loc_14772(untagged_height_word),
        rise_height_steps: untagged_height_word >> 4,
        biased_x_px: x_px.wrapping_add(0x80),
        biased_y_px: y_px.wrapping_add(0x80),
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct LavaFireballScriptStartLoc149CC {
    pub(crate) script_name: &'static str,
    pub(crate) x_px: u16,
    pub(crate) y_px: u16,
    pub(crate) sprite_top_y_px: u16,
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn initialise_lava_fireball_script_loc_149CC(
    biased_x_px: u16,
    biased_y_px: u16,
) -> LavaFireballScriptStartLoc149CC {
    // ASM loc_149CC (kid.asm:28052): the USM2 fireball restarts from
    // `unk_146CC`, un-biases `$44/$46(a5)` back into `d0/d1`, and seeds
    // `$54(a5)` from the same unbiased baseline row before the segment loop at
    // `loc_149E4`.
    let x_px = biased_x_px.wrapping_sub(0x80);
    let y_px = biased_y_px.wrapping_sub(0x80);
    LavaFireballScriptStartLoc149CC {
        script_name: "unk_146CC",
        x_px,
        y_px,
        sprite_top_y_px: y_px,
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct LavaFireballSegmentLoc149E4 {
    pub(crate) dbf_counter: u16,
    pub(crate) frames: u16,
    pub(crate) delta_y: i16,
    pub(crate) sprite_script_word: u16,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum LavaFireballScriptStepLoc149E4 {
    CooldownAndRestart,
    EmitSegment(LavaFireballSegmentLoc149E4),
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct LavaFireballLoopResumeLoc149FC {
    pub(crate) hibernation_frames: u16,
    pub(crate) collision_refreshes: u16,
    pub(crate) sprite_script_restored: bool,
    pub(crate) next_baseline_y_px: u16,
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn decode_lava_fireball_script_step_loc_149E4(
    script_words: &[u16],
) -> Option<LavaFireballScriptStepLoc149E4> {
    // ASM loc_149E4 (kid.asm:28060): each fireball script entry starts with
    // `move.w (a0)+,d2`; negative values branch to `loc_14A4C` for the `$3C`
    // frame hibernation. Non-negative entries load `$52(a5)` as the per-frame
    // Y delta, then resolve the next word as a `LoadGameModeData` sprite
    // script pointer before entering the segment loop at `loc_14A0A`.
    let (&dbf_counter, rest) = script_words.split_first()?;
    if (dbf_counter as i16) < 0 {
        return Some(LavaFireballScriptStepLoc149E4::CooldownAndRestart);
    }

    let (&delta_y_word, remaining) = rest.split_first()?;
    let &sprite_script_word = remaining.first()?;
    Some(LavaFireballScriptStepLoc149E4::EmitSegment(
        LavaFireballSegmentLoc149E4 {
            dbf_counter,
            frames: dbf_counter.wrapping_add(1),
            delta_y: delta_y_word as i16,
            sprite_script_word,
        },
    ))
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn lava_fireball_loop_resume_loc_149FC(
    baseline_y_px: u16,
    delta_y: i16,
) -> LavaFireballLoopResumeLoc149FC {
    // ASM loc_149FC (kid.asm:28072): each taken fireball-segment back-edge
    // hibernates for one frame, re-runs `sub_14AB0`, restores `a2` from `a1`,
    // then advances `d1` by the signed `$52(a5)` delta before re-entering the
    // shared camera-window path at `loc_14A0A`.
    LavaFireballLoopResumeLoc149FC {
        hibernation_frames: 1,
        collision_refreshes: 1,
        sprite_script_restored: true,
        next_baseline_y_px: baseline_y_px.wrapping_add_signed(delta_y),
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct LavaFireballSegmentBackEdgeLoc14A46 {
    pub(crate) next_dbf_counter: u16,
    pub(crate) reenters_segment_loop: bool,
    pub(crate) advances_to_next_script_step: bool,
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn lava_fireball_segment_back_edge_loc_14A46(
    dbf_counter: u16,
) -> LavaFireballSegmentBackEdgeLoc14A46 {
    // ASM loc_14A46 (kid.asm:28098): `dbf d2,loc_149FC` decrements the low
    // word once and loops back while the post-decrement value is not `$FFFF`.
    // When the counter reaches `$FFFF`, the segment finishes and control falls
    // through to the next script entry at `loc_149E4`.
    let next_dbf_counter = dbf_counter.wrapping_sub(1);
    let reenters_segment_loop = next_dbf_counter != u16::MAX;
    LavaFireballSegmentBackEdgeLoc14A46 {
        next_dbf_counter,
        reenters_segment_loop,
        advances_to_next_script_step: !reenters_segment_loop,
    }
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn lava_geyser_startup_script_loc_14788() -> &'static str {
    // ASM loc_14788 (kid.asm:27825): after the initial hibernation and optional
    // `sfx_Lava_Geyser_starting`, the very first eruption seeds `a0` from
    // `unk_146B6`, not the steady-state geyser script used by later cycles.
    "unk_146B6"
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn lava_geyser_repeat_sound_loc_1478E(sound_suppressed: bool) -> Option<&'static str> {
    // ASM loc_1478E (kid.asm:27830): the repeat-cycle path plays
    // `sfx_Lava_Geyser` only when `$19(a3)` is clear; non-zero skips the sound
    // call and falls straight through to the steady-state script at loc_1479C.
    (!sound_suppressed).then_some("sfx_Lava_Geyser")
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn lava_geyser_repeat_script_loc_1479C() -> &'static str {
    // ASM loc_1479C (kid.asm:27836): repeat eruptions reload `a0` from
    // `unk_146A0`, the steady-state geyser script that runs after the optional
    // sound gate at loc_1478E.
    "unk_146A0"
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct LavaGeyserFullHeightStateLoc147A0 {
    pub(crate) x_px: u16,
    pub(crate) y_px: u16,
    pub(crate) hold_frames: u16,
    pub(crate) sprite_top_y_px: u16,
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn initialise_lava_geyser_full_height_state_loc_147A0(
    biased_x_px: u16,
    biased_y_px: u16,
) -> LavaGeyserFullHeightStateLoc147A0 {
    // ASM loc_147A0 (kid.asm:27839): `$44/$46(a5)` already include the VDP's
    // `$80` sprite bias, so the worker subtracts `$80` back out before seeding
    // `d0/d1`, stores the fixed `$1E` full-height timer in `$50(a5)`, and
    // initialises `$54(a5)` from that unbiased base Y row.
    let x_px = biased_x_px.saturating_sub(0x80);
    let y_px = biased_y_px.saturating_sub(0x80);
    LavaGeyserFullHeightStateLoc147A0 {
        x_px,
        y_px,
        hold_frames: 0x1E,
        sprite_top_y_px: y_px,
    }
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn lava_geyser_peak_row_target_loc_14966(sprite_row_target: u16) -> u16 {
    // ASM loc_14966 (kid.asm:28018): when the alternating top-cap frame wraps
    // back to zero, the worker grows `$4A(a5)` by 2 before reloading `d3`.
    sprite_row_target.wrapping_add(2)
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct LavaGeyserPeakHoldStepLoc1496C {
    pub(crate) sprite_row_index: u16,
    pub(crate) frames_per_sprite: u16,
    pub(crate) next_hold_frames: u16,
    pub(crate) descending: bool,
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn lava_geyser_peak_hold_step_loc_1496C(
    sprite_row_target: u16,
    hold_frames_remaining: u16,
) -> LavaGeyserPeakHoldStepLoc1496C {
    // ASM loc_1496C (kid.asm:28021): reload `d3` from `$4A(a5)`, use the
    // 6-frame top-cap cadence while holding, decrement `$50(a5)`, and only
    // switch to the 1-frame collapse cadence once the decremented timer hits 0.
    let next_hold_frames = hold_frames_remaining.wrapping_sub(1);
    let descending = next_hold_frames == 0;
    LavaGeyserPeakHoldStepLoc1496C {
        sprite_row_index: sprite_row_target,
        frames_per_sprite: if descending { 1 } else { 6 },
        next_hold_frames,
        descending,
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct LavaGeyserCollapseStepLoc1498C {
    pub(crate) next_sprite_row_index: u16,
    pub(crate) descending: bool,
    pub(crate) restart_startup_script: bool,
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn lava_geyser_collapse_step_loc_1498C(
    sprite_row_index: u16,
) -> LavaGeyserCollapseStepLoc1498C {
    // ASM loc_1498C (kid.asm:28031): decrement `d3` once per collapse tick and
    // keep looping through loc_1485E while rows remain; when the decremented
    // count reaches zero, clear `$4D(a5)` and jump back to loc_14788.
    let next_sprite_row_index = sprite_row_index.wrapping_sub(1);
    let restart_startup_script = next_sprite_row_index == 0;
    LavaGeyserCollapseStepLoc1498C {
        next_sprite_row_index,
        descending: !restart_startup_script,
        restart_startup_script,
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum LavaGeyserScriptStepLoc147BA {
    HoldAtFullHeight,
    DisableForCooldown,
    EmitSpriteRows {
        sprite_rows_minus_one: u16,
        sprite_script_word: u16,
    },
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn decode_lava_geyser_script_step_loc_147BA(
    script_words: &[u16],
) -> Option<LavaGeyserScriptStepLoc147BA> {
    // ASM loc_147BA (kid.asm:27847): each geyser script entry starts with a
    // word loaded into `d2`. `$FFFF` jumps to the dedicated full-height loop,
    // `$FFFE` disables the geyser for `$3C` frames, and every other value keeps
    // the next word in place so `d2` can count sprite rows while `a1/a2` point
    // at the `LoadGameModeData + script_word` sprite template.
    let (&sprite_rows_minus_one, rest) = script_words.split_first()?;
    Some(match sprite_rows_minus_one {
        0xFFFF => LavaGeyserScriptStepLoc147BA::HoldAtFullHeight,
        0xFFFE => LavaGeyserScriptStepLoc147BA::DisableForCooldown,
        _ => LavaGeyserScriptStepLoc147BA::EmitSpriteRows {
            sprite_rows_minus_one,
            sprite_script_word: *rest.first()?,
        },
    })
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct LavaGeyserDisableStepLoc1483A {
    pub(crate) sound: &'static str,
    pub(crate) disabled_frames: u16,
    pub(crate) resumes_at_repeat_loop: bool,
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn lava_geyser_disable_step_loc_1483A() -> LavaGeyserDisableStepLoc1483A {
    // ASM loc_1483A (kid.asm:27900): the `$FFFE` script sentinel falls here,
    // queues `sfx_Lava_Geyser`, hibernates for `$3C` frames, then resumes at
    // the repeat-eruption path (`loc_1478E`) rather than the startup script.
    LavaGeyserDisableStepLoc1483A {
        sound: "sfx_Lava_Geyser",
        disabled_frames: 0x003C,
        resumes_at_repeat_loop: true,
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct LavaGeyserRiseResetLoc1484E {
    pub(crate) sprite_row_index: u16,
    pub(crate) frames_per_sprite_row: u16,
    pub(crate) sprite_frame_within_row: u8,
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn lava_geyser_rise_reset_loc_1484E() -> LavaGeyserRiseResetLoc1484E {
    // ASM loc_1484E (kid.asm:27908): the upward-motion path restarts the row
    // cursor in `d3`, seeds `$48(a5)` with a fixed 2-frame cadence per sprite
    // row, and clears `$4C(a5)` before entering the shared `loc_14866` loop.
    LavaGeyserRiseResetLoc1484E {
        sprite_row_index: 0,
        frames_per_sprite_row: 2,
        sprite_frame_within_row: 0,
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct LavaGeyserPreLoopHibernateLoc1485E {
    pub(crate) hibernation_frames: u16,
    pub(crate) collision_refreshes: u16,
    pub(crate) falls_through_to_shared_loop: bool,
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn lava_geyser_pre_loop_hibernate_loc_1485E() -> LavaGeyserPreLoopHibernateLoc1485E {
    // ASM loc_1485E (kid.asm:27915): before reloading `$48(a5)` into `d2` at
    // `loc_14866`, the worker hibernates exactly one frame, re-runs
    // `sub_14A58`, then falls through into the shared sprite-row loop.
    LavaGeyserPreLoopHibernateLoc1485E {
        hibernation_frames: 1,
        collision_refreshes: 1,
        falls_through_to_shared_loop: true,
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct LavaGeyserLoopBackEdgeLoc1486C {
    pub(crate) hibernation_frames: u16,
    pub(crate) collision_refreshes: u16,
    pub(crate) reenters_shared_loop: bool,
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn lava_geyser_loop_back_edge_loc_1486C() -> LavaGeyserLoopBackEdgeLoc1486C {
    // ASM loc_1486C (kid.asm:27924): each taken `dbf d2,loc_1486C` back-edge
    // hibernates one frame, re-runs `sub_14A58`, then re-enters the shared
    // `loc_14874` sprite-row builder.
    LavaGeyserLoopBackEdgeLoc1486C {
        hibernation_frames: 1,
        collision_refreshes: 1,
        reenters_shared_loop: true,
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct LavaGeyserCameraWindowInputLoc147D6 {
    pub(crate) x_px: u16,
    pub(crate) y_px: u16,
    pub(crate) camera_x_px: u16,
    pub(crate) camera_y_px: u16,
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn lava_geyser_is_inside_camera_window_loc_147D6(
    input: LavaGeyserCameraWindowInputLoc147D6,
) -> bool {
    // ASM loc_147D6 (kid.asm:27858): before building one sprite row, the lava
    // geyser worker culls against a padded camera window. X must stay within
    // `[Camera_X_pos - $20, Camera_X_pos + $160]` and Y within
    // `[Camera_Y_pos - $48, Camera_Y_pos + $128]`; anything outside branches to
    // `loc_1482E`, which hibernates one frame without drawing.
    let x_px = i32::from(input.x_px);
    let y_px = i32::from(input.y_px);
    let camera_x_px = i32::from(input.camera_x_px);
    let camera_y_px = i32::from(input.camera_y_px);

    let min_x = camera_x_px - 0x20;
    let max_x = min_x + 0x180;
    let min_y = camera_y_px - 0x48;
    let max_y = min_y + 0x170;

    x_px >= min_x && x_px <= max_x && y_px >= min_y && y_px <= max_y
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct LavaGeyserTopSpriteWindowInputLoc14874 {
    pub(crate) x_px: u16,
    pub(crate) y_px: u16,
    pub(crate) sprite_row_index: u16,
    pub(crate) camera_x_px: u16,
    pub(crate) camera_y_px: u16,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum LavaGeyserTopSpriteWindowResultLoc14874 {
    Culled,
    SkipTopSprite { sprite_row_y_px: u16 },
    BuildTopSprite { sprite_row_y_px: u16 },
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn lava_geyser_top_sprite_window_loc_14874(
    input: LavaGeyserTopSpriteWindowInputLoc14874,
) -> LavaGeyserTopSpriteWindowResultLoc14874 {
    // ASM loc_14874 (kid.asm:27928): the shared geyser row loop first un-biases
    // `$44/$46(a5)` by `$80`, subtracts `$10 + ((d3 + 1) << 4)` from Y for the
    // current row, and culls against X `[Camera_X_pos - $20, Camera_X_pos +
    // $160]`. Rows above `Camera_Y_pos - $28` skip only the top-cap build via
    // `loc_148E6`; rows below `Camera_Y_pos + $108` branch straight to
    // `loc_14940` and emit nothing.
    let x_px = input.x_px.wrapping_sub(0x80);
    let sprite_row_y_px = input
        .y_px
        .wrapping_sub(0x80)
        .wrapping_sub(0x10)
        .wrapping_sub((input.sprite_row_index.wrapping_add(1)) << 4);

    let x_px_i32 = i32::from(x_px);
    let sprite_row_y_px_i32 = i32::from(sprite_row_y_px);
    let min_x = i32::from(input.camera_x_px) - 0x20;
    let max_x = min_x + 0x180;
    if x_px_i32 < min_x || x_px_i32 > max_x {
        return LavaGeyserTopSpriteWindowResultLoc14874::Culled;
    }

    let min_y = i32::from(input.camera_y_px) - 0x28;
    if sprite_row_y_px_i32 < min_y {
        return LavaGeyserTopSpriteWindowResultLoc14874::SkipTopSprite { sprite_row_y_px };
    }

    let max_y = min_y + 0x130;
    if sprite_row_y_px_i32 > max_y {
        return LavaGeyserTopSpriteWindowResultLoc14874::Culled;
    }

    LavaGeyserTopSpriteWindowResultLoc14874::BuildTopSprite { sprite_row_y_px }
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn lava_geyser_top_sprite_script_loc_148CE(sprite_frame_within_row: u8) -> &'static str {
    // ASM loc_148CE (kid.asm:27959): when `$4C(a5)` is non-zero the geyser uses
    // `dword_14636` for the top sprite-row template; zero keeps `unk_14624`.
    if sprite_frame_within_row == 0 {
        "unk_14624"
    } else {
        "dword_14636"
    }
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn lava_geyser_store_top_sprite_y_loc_148D4(
    sprite_screen_y_with_bias: u16,
    camera_y_px: u16,
) -> u16 {
    // ASM loc_148D4 (kid.asm:27962): after `sub_14BD8` emits the selected
    // top-cap script, `d7` holds the built screen Y with the usual `$80` bias.
    // The worker un-biases it and stores world Y in `$54(a5)` via
    // `d7 - $80 + Camera_Y_pos`.
    sprite_screen_y_with_bias
        .wrapping_sub(0x80)
        .wrapping_add(camera_y_px)
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum LavaGeyserMiddleSpriteRowResultLoc148F8 {
    SkipAboveWindow,
    BuildSpriteRow,
    CulledBelowWindow,
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn lava_geyser_middle_sprite_row_window_loc_148F8(
    sprite_row_y_px: u16,
    camera_y_px: u16,
) -> LavaGeyserMiddleSpriteRowResultLoc148F8 {
    // ASM loc_148F8 (kid.asm:27977): each middle row checks `d1` against the
    // inclusive vertical band `[Camera_Y_pos - $10, Camera_Y_pos + $F0]`.
    // Rows above the band skip this build and fall through to loc_14914;
    // rows below it branch to loc_14940 and stop the current geyser frame.
    let sprite_row_y_px = i32::from(sprite_row_y_px);
    let min_y = i32::from(camera_y_px) - 0x10;
    if sprite_row_y_px < min_y {
        return LavaGeyserMiddleSpriteRowResultLoc148F8::SkipAboveWindow;
    }

    let max_y = min_y + 0x100;
    if sprite_row_y_px > max_y {
        return LavaGeyserMiddleSpriteRowResultLoc148F8::CulledBelowWindow;
    }

    LavaGeyserMiddleSpriteRowResultLoc148F8::BuildSpriteRow
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct LavaGeyserMiddleRowAdvanceResultLoc14914 {
    pub(crate) next_sprite_row_y_px: u16,
    pub(crate) next_middle_row_index: u16,
    pub(crate) continue_middle_rows: bool,
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn lava_geyser_middle_row_advance_loc_14914(
    sprite_row_y_px: u16,
    middle_row_index: u16,
    middle_row_limit: u16,
) -> LavaGeyserMiddleRowAdvanceResultLoc14914 {
    // ASM loc_14914 (kid.asm:27988): after each middle-row visit, the geyser
    // advances `d1` by `$10`, increments `d5`, and loops back to loc_148F8
    // while the updated index still satisfies `d5 <= d3`.
    let next_sprite_row_y_px = sprite_row_y_px.wrapping_add(0x10);
    let next_middle_row_index = middle_row_index.wrapping_add(1);
    LavaGeyserMiddleRowAdvanceResultLoc14914 {
        next_sprite_row_y_px,
        next_middle_row_index,
        continue_middle_rows: next_middle_row_index <= middle_row_limit,
    }
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn lava_geyser_inter_row_hibernations_loc_14804(sprite_rows_minus_one: u16) -> u16 {
    // ASM loc_14804 (kid.asm:27876): the `dbf d2,loc_14804` back-edge runs only
    // between sprite rows. With `d2` seeded from the "rows minus one" script
    // word, the worker hibernates one frame and re-runs `sub_14A58` exactly
    // once per additional row after the first.
    sprite_rows_minus_one
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn lava_geyser_sprite_top_y_loc_1480C(
    sprite_screen_y_with_bias: u16,
    camera_y_px: u16,
) -> u16 {
    // ASM loc_1480C (kid.asm:27880): after `sub_14BD8` builds the current row,
    // the worker un-biases the returned screen Y by `$80`, adds `Camera_Y_pos`,
    // and stores the result in `$54(a5)` for the following collision checks.
    lava_geyser_store_top_sprite_y_loc_148D4(sprite_screen_y_with_bias, camera_y_px)
}

const LAVA_GEYSER_POSITIONS_USM2_LOC_14C5E: &[(u16, u16, u16)] =
    &[(0x180, 0x312, 0x60), (0x240, 0x312, 0x63)];
const LAVA_GEYSER_POSITIONS_BLACK_PIT_LOC_14C5E: &[(u16, u16, u16)] = &[
    (0x290, 0x170, 0x20),
    (0x3D0, 0x170, 0x20),
    (0x510, 0x170, 0x40),
    (0x650, 0x170, 0x60),
    (0x790, 0x170, 0x80),
    (0x8D0, 0x170, 0xA0),
    (0xA10, 0x170, 0xC0),
    (0xB50, 0x170, 0xE0),
    (0xC90, 0x170, 0x100),
    (0xDD0, 0x170, 0x100),
    (0xF10, 0x170, 0x100),
    (0x1050, 0x170, 0x100),
    (0x1190, 0x170, 0x100),
];
const LAVA_GEYSER_POSITIONS_OTHER_LOC_14C5E: &[(u16, u16, u16)] = &[
    (0x090, 0x330, 0x260),
    (0x010, 0x1D0, 0x090),
    (0x110, 0x1D0, 0x090),
];

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum LavaGeyserPositionTableLoc14C5E {
    Usm2,
    BlackPit,
    Other,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct LavaGeyserInitLoc14C6A {
    pub(crate) x_px: u16,
    pub(crate) y_px: u16,
    pub(crate) height_px: u16,
    pub(crate) startup_delay_frames: u16,
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn lava_geyser_position_table_loc_14C5E(
    resource_level_id: u8,
) -> LavaGeyserPositionTableLoc14C5E {
    // ASM loc_14C5E (kid.asm:28305): after resolving `Current_LevelID`
    // through `MapOrder_Index`, the lava special-effect worker picks USM2's
    // position table for resource level 0, Black Pit's table for resource
    // levels 2 and 4, and the shared fallback table for every other lava level.
    match resource_level_id {
        0 => LavaGeyserPositionTableLoc14C5E::Usm2,
        2 | 4 => LavaGeyserPositionTableLoc14C5E::BlackPit,
        _ => LavaGeyserPositionTableLoc14C5E::Other,
    }
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn initialise_lava_geyser_loc_14C6A(
    x_px: u16,
    y_px: u16,
    encoded_height_word: u16,
) -> LavaGeyserInitLoc14C6A {
    // ASM loc_14C6A (kid.asm:28312): each loop iteration allocates one geyser
    // worker, copies the table's longword into `$16(a0)` as `(x,y)`, copies
    // the encoded height word into `$1A(a0)`, and leaves the worker to decode
    // `height >> 4` plus the low-nibble startup delay later at `loc_14772`.
    LavaGeyserInitLoc14C6A {
        x_px,
        y_px,
        height_px: encoded_height_word >> 4,
        startup_delay_frames: lava_geyser_initial_delay_frames_loc_14772(encoded_height_word),
    }
}

fn advance_lava_fireball(fireball: &mut LavaFireball) {
    if fireball.cooldown_timer > 0 {
        fireball.cooldown_timer -= 1;
        if fireball.cooldown_timer == 0 {
            fireball.segment_index = 0;
            fireball.segment_frame = 0;
            fireball.current_y_px = fireball.base_y_px;
        }
        return;
    }

    let segment = LAVA_FIREBALL_SCRIPT[usize::from(fireball.segment_index)];
    fireball.current_y_px = fireball.current_y_px.saturating_add_signed(segment.delta_y);
    fireball.segment_frame = fireball.segment_frame.saturating_add(1);

    if fireball.segment_frame >= segment.frames {
        fireball.segment_frame = 0;
        if usize::from(fireball.segment_index) + 1 == LAVA_FIREBALL_SCRIPT.len() {
            fireball.segment_index = 0;
            fireball.cooldown_timer = 0x3C;
            fireball.current_y_px = fireball.base_y_px;
        } else {
            fireball.segment_index += 1;
        }
    }
}

impl NativeRuntime {
    /// Advance the murder wall one frame: accelerate then move.
    ///
    /// **Forward path** (non-reversed, kid.asm:7447–7478):
    /// ```asm
    /// ; loc_72E2 gates on MurderWall_flag → falls through to forward/reversed
    ///   clr.l   d1
    ///   tst.b   (MurderWall_reversed).w
    ///   bne.s   loc_736E               ; reversed path
    ///   cmpi.w  #L_Hills_of_the_Warrior_1,(Current_LevelID).w
    ///   bne.s   loc_7316
    ///   move.w  #$80,d1               ; Hills: extra camera-push threshold
    /// loc_7316:                        ; ← kid.asm:7454
    ///   addi.w  #$80,d1               ; d1 = $80 (normal) or $100 (Hills)
    ///   addi.l  #$100,(MurderWall_speed).w
    ///   move.l  (MurderWall_speed).w,d7
    ///   cmp.l   (MurderWall_max_speed).w,d7
    ///   blt.s   loc_7334
    ///   move.l  (MurderWall_max_speed).w,d7
    ///   move.l  d7,(MurderWall_speed).w
    /// loc_7334:                        ; ← kid.asm:7463
    ///   add.l   d7,(MurderWall_X_pos).w
    ///   move.w  (Camera_max_X_pos).w,d0
    ///   cmp.w   (MurderWall_X_pos).w,d0
    ///   bgt.s   loc_7346
    ///   move.w  d0,(MurderWall_X_pos).w  ; clamp to level bound
    /// ```
    ///
    /// **Reversed path** (kid.asm:7486–7514): mirrors with `sub.l`.
    ///
    /// The forward camera-push logic (loc_7346–loc_7364) and the
    /// Hills-of-the-Warrior threshold (`d1`) are not yet modelled.
    // TODO(fidelity): loc_7346–loc_7364 forward camera push: when the forward
    // wall overtakes the camera, the ROM snaps Camera_X_pos = MurderWall_X_pos.
    // When the camera leads the forward wall by more than d1 ($80 or $100 for
    // Hills), it snaps MurderWall_X_pos = Camera_X_pos - d1.
    pub(crate) fn step_murder_wall(&mut self, camera_x: &mut u32) {
        let Some(wall) = self.murder_wall.as_mut() else {
            return;
        };
        // ASM loc_7316 (kid.asm:7456) / loc_736E (kid.asm:7487):
        // `addi.l #$100,(MurderWall_speed).w` then clamp to max_speed.
        wall.speed = (wall.speed + MURDER_WALL_ACCELERATION).min(wall.max_speed);
        if wall.reversed {
            // ASM loc_7388 (kid.asm:7495): `sub.l d7,(MurderWall_X_pos).w`
            wall.x_pos -= i64::from(wall.speed);
            // ASM loc_7388 (kid.asm:7496–7497): `bgt.s loc_7396` / floor clamp
            // to zero when the reversed wall overshoots the left level edge.
            if wall.x_pos <= 0 {
                wall.x_pos = 0;
            }
            // ASM loc_7396–loc_73B8 (kid.asm:7499–7514): reverse camera push.
            let (new_cam, new_wall) =
                physics::reverse_murder_wall_camera_push_loc_7396(*camera_x, wall.x_pos);
            *camera_x = new_cam;
            wall.x_pos = new_wall;
        } else {
            // ASM loc_7334 (kid.asm:7464): `add.l d7,(MurderWall_X_pos).w`
            wall.x_pos += i64::from(wall.speed);
        }
    }

    /// Check if the murder wall overlaps the player, triggering instant death.
    pub(crate) fn murder_wall_kills_player(&self, slice: &OpeningSliceState) -> bool {
        let Some(wall) = &self.murder_wall else {
            return false;
        };
        let player_x = slice.player.x_pos as i64;
        let wall_x = wall.x_pos;
        if wall.reversed {
            // Wall moves right-to-left: kill if player's right side touches wall
            let wall_left = wall_x;
            player_x + (i64::from(MURDER_WALL_CAMERA_OFFSET_PX) << 16) >= wall_left
        } else {
            // Wall moves left-to-right: kill if wall passes player's left side
            let wall_right = wall_x + (i64::from(MURDER_WALL_CAMERA_OFFSET_PX) << 16);
            wall_right >= player_x
        }
    }

    pub(crate) fn step_weather(&mut self, slice: &OpeningSliceState) {
        // Hail only for Storm+Hail (weather 3), active after 35s of level time
        if self.weather.weather_type != 3 {
            return;
        }
        // Level timer counts down from 180s. 35s elapsed = 145s remaining.
        if self.level_timer.seconds > 145 {
            return;
        }

        // Toggle hail active/inactive every 600 frames ($258)
        if self.weather.hail_period_timer > 0 {
            self.weather.hail_period_timer -= 1;
        } else {
            self.weather.hail_active = !self.weather.hail_active;
            self.weather.hail_period_timer = 600;
        }

        if !self.weather.hail_active {
            return;
        }

        // Spawn hail every 30 frames during active period
        if self.weather.hail_spawn_timer > 0 {
            self.weather.hail_spawn_timer -= 1;
            return;
        }
        self.weather.hail_spawn_timer = 30;

        // Spawn a hail projectile above the camera, at a random-ish X
        let camera_x = slice.camera.x_pos >> 16;
        // Simple pseudo-random X offset based on frame index
        let x_offset = ((self.frame_index * 7 + 13) % 320) as u32;
        let spawn_x = camera_x.wrapping_add(x_offset);
        let spawn_y = (slice.camera.y_pos >> 16).saturating_sub(16);

        // ASM sub_61CA loc_61E2 (kid.asm:6199): hail particles fall at
        // `addi.w #4,(a0)` = 4 px/frame downward. The remaining EnemyProjectile
        // fields (x_velocity, friendly, gravity, homing_accel, speed_cap,
        // wall_bounce, ...) all match `Default::default()` for hail and are
        // covered by the spread; listing them explicitly only invites
        // equivalent-mutant noise from cargo-mutants.
        // TODO(fidelity): ROM also applies `subi.w #1,2(a0)` (-1 px/frame X
        // drift) and an off-screen kill instead of a 120-frame lifetime; this
        // simplified path uses a fixed lifetime as a stub.
        self.projectiles.push(EnemyProjectile {
            x_pos: spawn_x << 16,
            y_pos: spawn_y << 16,
            y_velocity: 0x4_0000,
            lifetime: 120,
            ..Default::default()
        });
    }

    pub(crate) fn init_weather(&mut self, level_id: LevelId) {
        let header = self.level_header(level_id);
        self.weather.weather_type = header.map_or(0, |h| h.weather_flags);
        self.weather.hail_active = false;
        self.weather.hail_spawn_timer = 0;
        self.weather.hail_period_timer = 0;
        self.weather.lava_geysers.clear();
        self.weather.lava_fireballs.clear();
        if self.weather.weather_type == 1 {
            self.init_lava_geysers(level_id);
        }
    }

    /// Initialise lava geyser positions for lava weather levels.
    /// Uses hardcoded per-level position tables matching the original ASM data.
    pub(crate) fn init_lava_geysers(&mut self, level_id: LevelId) {
        let resource_level_id = kid_data::resource_level_id(&self.assets, level_id.0);
        let selected_table = lava_geyser_position_table_loc_14C5E(resource_level_id);
        let positions = match selected_table {
            LavaGeyserPositionTableLoc14C5E::Usm2 => LAVA_GEYSER_POSITIONS_USM2_LOC_14C5E,
            LavaGeyserPositionTableLoc14C5E::BlackPit => LAVA_GEYSER_POSITIONS_BLACK_PIT_LOC_14C5E,
            LavaGeyserPositionTableLoc14C5E::Other => LAVA_GEYSER_POSITIONS_OTHER_LOC_14C5E,
        };

        for &(x, y, h) in positions {
            let geyser = initialise_lava_geyser_loc_14C6A(x, y, h);
            self.weather.lava_geysers.push(LavaGeyser {
                x_px: geyser.x_px,
                y_px: geyser.y_px,
                height_px: geyser.height_px,
                phase: 0,
                timer: geyser.startup_delay_frames,
                current_height: 0,
            });
        }

        if matches!(selected_table, LavaGeyserPositionTableLoc14C5E::Usm2) {
            self.weather.lava_fireballs.push(LavaFireball {
                x_px: 0x346,
                base_y_px: 0x312,
                current_y_px: 0x312,
                segment_index: 0,
                segment_frame: 0,
                cooldown_timer: 0,
            });
        }
    }

    /// Step all active lava geysers through their eruption cycle.
    pub(crate) fn step_lava_geysers(&mut self, slice: &OpeningSliceState) {
        if self.weather.weather_type != 1
            || (self.weather.lava_geysers.is_empty() && self.weather.lava_fireballs.is_empty())
        {
            return;
        }

        let mut should_damage = false;

        for geyser in &mut self.weather.lava_geysers {
            match geyser.phase {
                0 => {
                    // Waiting/dormant
                    if geyser.timer > 0 {
                        geyser.timer -= 1;
                    } else {
                        geyser.phase = 1;
                        geyser.current_height = 0;
                    }
                }
                1 => {
                    // Erupting upward
                    geyser.current_height += LAVA_GEYSER_ERUPT_SPEED;
                    if geyser.current_height >= geyser.height_px {
                        geyser.current_height = geyser.height_px;
                        geyser.phase = 2;
                        geyser.timer = LAVA_GEYSER_HOLD_FRAMES;
                    }
                }
                2 => {
                    // Holding at full height
                    if geyser.timer > 0 {
                        geyser.timer -= 1;
                    } else {
                        geyser.phase = 3;
                    }
                }
                3 => {
                    // Collapsing
                    if geyser.current_height > LAVA_GEYSER_ERUPT_SPEED {
                        geyser.current_height -= LAVA_GEYSER_ERUPT_SPEED;
                    } else {
                        geyser.current_height = 0;
                        geyser.phase = 0;
                        geyser.timer = LAVA_GEYSER_DORMANT_FRAMES;
                    }
                }
                _ => {}
            }

            if lava_geyser_hits_kid_sub_14A58(geyser, slice) {
                should_damage = true;
            }
        }

        for fireball in &mut self.weather.lava_fireballs {
            let segment = LAVA_FIREBALL_SCRIPT[usize::from(fireball.segment_index)];
            if fireball.cooldown_timer == 0
                && lava_fireball_hits_kid_sub_14AB0(fireball, slice, segment.sprite_kind)
            {
                should_damage = true;
            }
            advance_lava_fireball(fireball);
        }

        if should_damage && self.progression.invulnerable_frames == 0 {
            self.apply_damage();
        }
    }

    pub(crate) fn init_murder_wall(&mut self, level_id: LevelId, camera_x: u32) {
        let header = self.level_header(level_id);
        let flags = header.map_or(0, |h| h.murder_wall_flags);
        if flags & 0b10 != 0 {
            let reversed = flags & 0b01 != 0;
            let max_speed = if level_id.0 == 0x08 {
                MURDER_WALL_SLOW_MAX_SPEED
            } else {
                MURDER_WALL_DEFAULT_MAX_SPEED
            };
            // ASM loc_140BE clears `MurderWall_speed` after the level-specific
            // max-speed branch, so native always starts the wall from rest.
            let initial_x = if camera_x == 0 {
                // ASM loc_140CE runs after `move.w (Camera_X_pos).w,d0` / `beq.s`,
                // so the zero-camera case subtracts `$30` from zero before the
                // shared `loc_140D2` store.
                -(i64::from(MURDER_WALL_CAMERA_OFFSET_PX) << 16)
            } else {
                // ASM loc_140D2 receives the non-zero-camera `addi.w #$30,d0`
                // result and stores that shared branch value as Murderwall X.
                (i64::from(camera_x) << 16) + (i64::from(MURDER_WALL_CAMERA_OFFSET_PX) << 16)
            };
            self.murder_wall = Some(MurderWallState {
                x_pos: initial_x,
                speed: 0,
                max_speed,
                reversed,
            });
        } else {
            self.murder_wall = None;
        }
    }
}
