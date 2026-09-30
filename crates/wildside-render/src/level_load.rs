//! Load_Level art and background-mapping helpers that only the render
//! bundle loader consumes.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BackgroundMappingsChunkLoc12830 {
    pub chunk_index: u16,
    pub x_pos: i16,
    pub y_pos: i16,
}

#[allow(non_snake_case)]
pub fn background_mappings_read_chunk_loc_12830(
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
pub fn background_mappings_terminator_loc_12842() -> bool {
    // ASM loc_12842 (kid.asm:25176-25178): loop termination.
    //   movem.l (sp)+,d5-d7    ; restore saved registers
    //   rts                     ; return to caller
    // This is reached when loc_12830 reads a negative chunk index.
    // The terminator word has bit 15 set ($8000+).
    true
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BackgroundMountainLoadLoc12910 {
    pub pattern_vram_offset: u16,
    pub vram_buffer1_addr: u32,
    pub vram_buffer2_addr: u32,
    pub vram_buffer3_addr: u32,
    pub needs_hail_overlay_at_1294_a: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BackgroundHailOverlayLoc1294A {
    pub pattern_vram_offset: u16,
    pub vram_buffer4_addr: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BackgroundFinalPatternLoc12952 {
    pub pattern_vram_offset: u16,
    pub vram_buffer4_addr: u32,
}

#[allow(non_snake_case)]
pub fn background_mountain_tileset_loc_12910(
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
pub fn background_hail_overlay_loc_1294A() -> BackgroundHailOverlayLoc1294A {
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
pub fn background_final_pattern_loc_12952(
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
pub struct BackgroundHillsLoadLoc1295E {
    pub pattern_vram_offset_1_2: u16,
    pub pattern_vram_offset_3: u16,
    pub vram_buffer1_addr: u32,
    pub vram_buffer2_addr: u32,
    pub vram_buffer3_addr: u32,
}

#[allow(non_snake_case)]
pub fn background_hills_tileset_loc_1295E() -> BackgroundHillsLoadLoc1295E {
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
pub struct BackgroundDesertLoadLoc12988 {
    pub pattern_vram_offset: u16,
    pub vram_buffer1_addr: u32,
    pub vram_buffer2_addr: u32,
    pub vram_buffer3_addr: u32,
}

#[allow(non_snake_case)]
pub fn background_desert_tileset_loc_12988() -> BackgroundDesertLoadLoc12988 {
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
pub struct JuggernautSkullArtDmaLoc129DC {
    vram_word_address: u16,
    pub(crate) vdp_command: u32,
    pub(crate) art_source: u32,
    pub(crate) longword_count: u16,
}

impl JuggernautSkullArtDmaLoc129DC {
    pub const fn vram_word_address(&self) -> u16 {
        self.vram_word_address
    }

    pub const fn vram_tile_base_index(&self) -> u16 {
        self.vram_word_address / 0x20
    }
}

#[allow(non_snake_case)]
#[allow(dead_code)]
pub fn juggernaut_skull_art_dma_loop_loc_129DC() -> JuggernautSkullArtDmaLoc129DC {
    // ASM loc_129DC (kid.asm:25359-25361): Juggernaut skull frame 1 DMA loop.
    //   loc_129DC:
    //   move.l  (a1)+,(a6)          ; copy longword from ArtUnc_12A08 to VDP
    //   dbf     d0,loc_129DC        ; loop 32 times ($1F+1 iterations)
    // This is the first of three DMA loops in sub_129CE that load:
    //   - ArtUnc_12A08 (Juggernaut skull frame 1) -> $CDC0
    //   - ArtUnc_12A8C (Juggernaut skull frame 2) -> $CDC0 (continuation, 32 more longwords)
    //   - ArtUnc_12B0C (Some kind of star)         -> $DF40
    const VDP_COMMAND: u32 = 0xC000_00CD; // vdpComm($CDC0,VRAM,WRITE)
    const VRAM_WORD_ADDRESS: u16 = 0xCDC0;
    const ART_SOURCE_OFFSET: u32 = 0x0001_2A08; // ArtUnc_12A08
    const LONGWORD_COUNT: u16 = 32; // $1F + 1

    JuggernautSkullArtDmaLoc129DC {
        vram_word_address: VRAM_WORD_ADDRESS,
        vdp_command: VDP_COMMAND,
        art_source: ART_SOURCE_OFFSET,
        longword_count: LONGWORD_COUNT,
    }
}

#[allow(non_snake_case)]
pub fn juggernaut_skull_art_dma_loop_loc_129E8() -> JuggernautSkullArtDmaLoc129DC {
    // ASM loc_129E8 (kid.asm:25365-25367): second `dbf` loop in sub_129CE.
    // It keeps the same `$CDC0` VRAM write command and copies another
    // `$1F+1` longwords, but `a1` now points at `ArtUnc_12A8C`. The VDP
    // auto-increment leaves this loop's effective stream start at `$CE40`.
    const VDP_COMMAND: u32 = 0xC000_00CD; // vdpComm($CDC0,VRAM,WRITE)
    const VRAM_WORD_ADDRESS: u16 = 0xCE40;
    const ART_SOURCE_OFFSET: u32 = 0x0001_2A8C; // ArtUnc_12A8C
    const LONGWORD_COUNT: u16 = 32; // $1F + 1

    JuggernautSkullArtDmaLoc129DC {
        vram_word_address: VRAM_WORD_ADDRESS,
        vdp_command: VDP_COMMAND,
        art_source: ART_SOURCE_OFFSET,
        longword_count: LONGWORD_COUNT,
    }
}

#[allow(non_snake_case)]
pub fn juggernaut_star_art_dma_loop_loc_129FC() -> JuggernautSkullArtDmaLoc129DC {
    // ASM loc_129FC (kid.asm:25372-25374): final `dbf` loop in sub_129CE.
    // This pass switches the VDP destination to `$DF40` and streams
    // `ArtUnc_12B0C` for another `$1F+1` longwords.
    const VDP_COMMAND: u32 = 0x4000_00DF; // vdpComm($DF40,VRAM,WRITE)
    const VRAM_WORD_ADDRESS: u16 = 0xDF40;
    const ART_SOURCE_OFFSET: u32 = 0x0001_2B0C; // ArtUnc_12B0C
    const LONGWORD_COUNT: u16 = 32; // $1F + 1

    JuggernautSkullArtDmaLoc129DC {
        vram_word_address: VRAM_WORD_ADDRESS,
        vdp_command: VDP_COMMAND,
        art_source: ART_SOURCE_OFFSET,
        longword_count: LONGWORD_COUNT,
    }
}

#[allow(non_snake_case)]
pub fn flag_bottom_art_dma_loop_loc_12B9C() -> JuggernautSkullArtDmaLoc129DC {
    // ASM loc_12B9C (kid.asm:25398-25400): `Load_FlagBottomArt` writes
    // `vdpComm($D2C0,VRAM,WRITE)` to the VDP control port, points `a1` at
    // `ArtUnc_12BA4`, seeds `d0=#1F`, then streams 32 longwords via
    // `move.l (a1)+,(a6)` / `dbf d0,loc_12B9C`.
    const VDP_COMMAND: u32 = 0xC000_00D2; // vdpComm($D2C0,VRAM,WRITE)
    const VRAM_WORD_ADDRESS: u16 = 0xD2C0;
    const ART_SOURCE_OFFSET: u32 = 0x0001_2BA4; // ArtUnc_12BA4
    const LONGWORD_COUNT: u16 = 32; // $1F + 1

    JuggernautSkullArtDmaLoc129DC {
        vram_word_address: VRAM_WORD_ADDRESS,
        vdp_command: VDP_COMMAND,
        art_source: ART_SOURCE_OFFSET,
        longword_count: LONGWORD_COUNT,
    }
}

#[allow(non_snake_case)]
pub fn hitpoint_display_art_dma_loop_loc_12C34() -> JuggernautSkullArtDmaLoc129DC {
    // ASM loc_12C34 (kid.asm:25414-25416): `sub_12C24` writes
    // `vdpComm($45A0,VRAM,WRITE)` to the VDP control port, points `a1` at
    // `ArtUnc_12C3C`, seeds `d0=#1F`, then streams 32 longwords via
    // `move.l (a1)+,(a6)` / `dbf d0,loc_12C34`.
    const VDP_COMMAND: u32 = 0xA000_00C5; // vdpComm($45A0,VRAM,WRITE)
    const VRAM_WORD_ADDRESS: u16 = 0x45A0;
    const ART_SOURCE_OFFSET: u32 = 0x0001_2C3C; // ArtUnc_12C3C
    const LONGWORD_COUNT: u16 = 32; // $1F + 1

    JuggernautSkullArtDmaLoc129DC {
        vram_word_address: VRAM_WORD_ADDRESS,
        vdp_command: VDP_COMMAND,
        art_source: ART_SOURCE_OFFSET,
        longword_count: LONGWORD_COUNT,
    }
}

#[allow(non_snake_case)]
pub fn block_mapping_or_mask_loc_11D42(high_priority_flag: bool) -> u32 {
    // ASM loc_11D0C -> loc_11D42 (kid.asm:23917-23923) clears `d1`, tests
    // `($FFFFF896).w`, and only loads `$80008000` into `d1` when that byte is
    // non-zero before falling through into the block-mapping copy loop.
    if high_priority_flag { 0x8000_8000 } else { 0 }
}

#[allow(non_snake_case)]
pub fn copy_block_mappings_loc_11D46(source_longwords: &[u32], or_mask: u32) -> Vec<u32> {
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ForegroundArtPermutationLoc11E48 {
    Identity,
    City,
    Forest,
    Mountain,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BackgroundArtPermutationLoc11E8A {
    Identity,
    Forest,
    Mountain,
    Hill,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BackgroundArtDecompressLoc11EA2 {
    pub tile_base: u16,
    pub permutation: BackgroundArtPermutationLoc11E8A,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BackgroundFollowupArtSourceLoc11EC4 {
    HillAlt,
    CaveAlt,
    MountainLightning,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BackgroundFollowupArtPermutationLoc11EC4 {
    Identity,
    BgHillAlt,
    BgMountain,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BackgroundFollowupArtDecompressLoc11EC4 {
    pub tile_base: u16,
    pub permutation: BackgroundFollowupArtPermutationLoc11EC4,
    pub art_source: BackgroundFollowupArtSourceLoc11EC4,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BackgroundFollowupArtDecompressLoc11EF2 {
    pub tile_base: u16,
    pub permutation: BackgroundFollowupArtPermutationLoc11EC4,
    pub art_source: BackgroundFollowupArtSourceLoc11EC4,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SharedVramArtSourceLoc11EFA {
    Blocks,
    IngameNumbers,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SharedVramArtDecompressLoc11EFA {
    pub tile_base: u16,
    pub art_source: SharedVramArtSourceLoc11EFA,
}

#[allow(non_snake_case)]
pub fn foreground_art_decompress_loc_11E48_11E60(
    foreground_theme: u16,
) -> ForegroundArtDecompressLoc11E60 {
    const FOREGROUND_THEME_CITY_LOC_11E48: u16 = 0x000A;

    // ASM Load_Level tail loc_11E36-loc_11E60: identity is seeded first, City
    // may override it before the Forest and Mountain branch labels run.
    let selected = if foreground_theme == FOREGROUND_THEME_CITY_LOC_11E48 {
        ForegroundArtPermutationLoc11E48::City
    } else {
        ForegroundArtPermutationLoc11E48::Identity
    };
    let selected = select_foreground_art_permutation_loc_11E48(foreground_theme, selected);
    let selected = select_foreground_art_permutation_loc_11E54(foreground_theme, selected);
    prepare_foreground_art_decompress_loc_11E60(selected)
}

#[allow(non_snake_case)]
pub fn background_art_decompress_loc_11E8A_11EA2(
    background_theme: u16,
) -> BackgroundArtDecompressLoc11EA2 {
    // ASM Load_Level tail loc_11E78-loc_11EA2: identity is seeded first, then
    // Forest, Mountain, and Hills can replace the active permutation.
    let selected = select_background_art_permutation_loc_11E8A(
        background_theme,
        BackgroundArtPermutationLoc11E8A::Identity,
    );
    let selected = select_background_art_permutation_loc_11E96(background_theme, selected);
    prepare_background_art_decompress_loc_11EA2(selected)
}

#[allow(non_snake_case)]
pub fn background_followup_art_decompress_loc_11EA2_11EF2(
    background_theme: u16,
    level_special_effects: u16,
) -> Option<BackgroundFollowupArtDecompressLoc11EF2> {
    const BACKGROUND_THEME_HILL_LOC_11EA2: u16 = 0x0003;

    if background_theme == BACKGROUND_THEME_HILL_LOC_11EA2 {
        // ASM loc_11EA2-loc_11EC0: Hills immediately performs a second
        // `DecompressToRAM` of `ThemeArtBack_HillAlt` at VRAM `$F800`.
        return Some(BackgroundFollowupArtDecompressLoc11EF2 {
            tile_base: 0xF800,
            permutation: BackgroundFollowupArtPermutationLoc11EC4::BgHillAlt,
            art_source: BackgroundFollowupArtSourceLoc11EC4::HillAlt,
        });
    }

    let mountain = prepare_mountain_background_art_loc_11EC4(background_theme)?;
    let mountain = apply_mountain_weather_override_loc_11EEA(level_special_effects, mountain);
    Some(prepare_followup_background_art_decompress_loc_11EF2(
        mountain,
    ))
}

#[allow(non_snake_case)]
pub fn select_foreground_art_permutation_loc_11E48(
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
pub fn select_foreground_art_permutation_loc_11E54(
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
pub fn select_background_art_permutation_loc_11E8A(
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
pub fn select_background_art_permutation_loc_11E96(
    background_theme: u16,
    selected_permutation: BackgroundArtPermutationLoc11E8A,
) -> BackgroundArtPermutationLoc11E8A {
    const BACKGROUND_THEME_MOUNTAIN_LOC_11E96: u16 = 0x0007;
    const BACKGROUND_THEME_HILL_LOC_11E96: u16 = 0x0003;

    // ASM loc_11E96 (kid.asm:24074-24077): after the earlier Forest compare,
    // Mountain selects `Palette_Permutation_BGMountain`; otherwise the
    // immediate Hills compare selects `Palette_Permutation_BGHill`.
    if background_theme == BACKGROUND_THEME_MOUNTAIN_LOC_11E96 {
        BackgroundArtPermutationLoc11E8A::Mountain
    } else if background_theme == BACKGROUND_THEME_HILL_LOC_11E96 {
        BackgroundArtPermutationLoc11E8A::Hill
    } else {
        selected_permutation
    }
}

#[allow(non_snake_case)]
pub fn prepare_background_art_decompress_loc_11EA2(
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
pub fn prepare_mountain_background_art_loc_11EC4(
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
pub fn apply_mountain_weather_override_loc_11EEA(
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
pub fn prepare_followup_background_art_decompress_loc_11EF2(
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
pub fn shared_vram_art_decompresses_loc_11EFA() -> [SharedVramArtDecompressLoc11EFA; 2] {
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
pub struct ForegroundArtDecompressLoc11E60 {
    pub tile_base: u16,
    pub permutation: ForegroundArtPermutationLoc11E48,
}

#[allow(non_snake_case)]
pub fn prepare_foreground_art_decompress_loc_11E60(
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn loc_11d42_only_enables_the_block_priority_mask_when_f896_is_non_zero() {
        assert_eq!(
            block_mapping_or_mask_loc_11D42(false),
            0,
            "ASM loc_11D42 should leave `d1` cleared when `tst.b ($FFFFF896).w` branches through `beq.s loc_11D42`",
        );
        assert_eq!(
            block_mapping_or_mask_loc_11D42(true),
            0x8000_8000,
            "ASM loc_11D0C should load `$80008000` into `d1` before `loc_11D42` when `($FFFFF896).w` is non-zero",
        );
    }

    #[test]
    fn loc_11d46_copies_exactly_a2_longwords_and_ors_each_with_the_prepared_mask() {
        let source_longwords = (0..0xA4).map(|value| value as u32).collect::<Vec<_>>();
        let copied = copy_block_mappings_loc_11D46(&source_longwords, 0x8000_8000);

        assert_eq!(
            copied.len(),
            0xA2,
            "ASM loc_11D46 seeds `d0` with `$A1`, so `dbf d0,loc_11D46` must copy exactly `$A2` longwords into `Block_Mappings`",
        );
        assert_eq!(
            copied[0], 0x8000_8000,
            "ASM loc_11D46 should OR the prepared `d1` mask into the first copied longword before writing `(a2)+`",
        );
        assert_eq!(
            copied[0xA1], 0x8000_80A1,
            "ASM loc_11D46 should still apply the same OR mask on the final DBF iteration",
        );
        assert_eq!(
            copied.last().copied(),
            Some(0x8000_80A1),
            "ASM loc_11D46 must stop after the `$A2`nd longword rather than consuming trailing data from the source table",
        );
    }

    #[test]
    fn loc_11e48_only_overrides_the_existing_foreground_permutation_for_forest() {
        assert_eq!(
            select_foreground_art_permutation_loc_11E48(
                0x0009,
                ForegroundArtPermutationLoc11E48::Identity
            ),
            ForegroundArtPermutationLoc11E48::Forest,
            "ASM loc_11E48 should replace the identity permutation with `Palette_Permutation_FGForest` when `Foreground_theme == Forest`"
        );
        assert_eq!(
            select_foreground_art_permutation_loc_11E48(
                0x000A,
                ForegroundArtPermutationLoc11E48::City
            ),
            ForegroundArtPermutationLoc11E48::City,
            "ASM loc_11E48 should fall through to `loc_11E54` without disturbing the earlier City permutation when the Forest compare fails"
        );
    }

    #[test]
    fn loc_11e54_only_overrides_the_existing_foreground_permutation_for_mountain() {
        assert_eq!(
            select_foreground_art_permutation_loc_11E54(
                0x0007,
                ForegroundArtPermutationLoc11E48::City
            ),
            ForegroundArtPermutationLoc11E48::Mountain,
            "ASM loc_11E54 should replace the current permutation with `Palette_Permutation_FGMountain` when `Foreground_theme == Mountain`"
        );
        assert_eq!(
            select_foreground_art_permutation_loc_11E54(
                0x0009,
                ForegroundArtPermutationLoc11E48::Forest
            ),
            ForegroundArtPermutationLoc11E48::Forest,
            "ASM loc_11E54 should fall through to `loc_11E60` without disturbing the earlier Forest permutation when the Mountain compare fails"
        );
    }

    #[test]
    fn loc_11e60_passes_the_selected_permutation_and_vram_base_into_decompress() {
        assert_eq!(
            prepare_foreground_art_decompress_loc_11E60(ForegroundArtPermutationLoc11E48::Mountain),
            ForegroundArtDecompressLoc11E60 {
                tile_base: 0x1780,
                permutation: ForegroundArtPermutationLoc11E48::Mountain,
            },
            "ASM loc_11E60 should carry the already selected foreground permutation into `DecompressToRAM` and keep the VRAM base at `$1780`"
        );
        assert_eq!(
            prepare_foreground_art_decompress_loc_11E60(ForegroundArtPermutationLoc11E48::Forest)
                .permutation,
            ForegroundArtPermutationLoc11E48::Forest,
            "ASM loc_11E60 should not disturb the earlier Forest selection before the shared decompressor call"
        );
    }

    #[test]
    fn loc_11e8a_only_overrides_the_existing_background_permutation_for_forest() {
        assert_eq!(
            select_background_art_permutation_loc_11E8A(
                0x0009,
                BackgroundArtPermutationLoc11E8A::Identity
            ),
            BackgroundArtPermutationLoc11E8A::Forest,
            "ASM loc_11E8A should replace the identity permutation with `Palette_Permutation_BGForest` when `Background_theme == Forest`"
        );
        assert_eq!(
            select_background_art_permutation_loc_11E8A(
                0x0007,
                BackgroundArtPermutationLoc11E8A::Identity
            ),
            BackgroundArtPermutationLoc11E8A::Identity,
            "ASM loc_11E8A should fall through to `loc_11E96` without disturbing the earlier identity permutation when the Forest compare fails"
        );
    }

    #[test]
    fn loc_11e96_only_overrides_the_existing_background_permutation_for_mountain() {
        assert_eq!(
            select_background_art_permutation_loc_11E96(
                0x0007,
                BackgroundArtPermutationLoc11E8A::Forest
            ),
            BackgroundArtPermutationLoc11E8A::Mountain,
            "ASM loc_11E96 should replace the current permutation with `Palette_Permutation_BGMountain` when `Background_theme == Mountain`"
        );
        assert_eq!(
            select_background_art_permutation_loc_11E96(
                0x0005,
                BackgroundArtPermutationLoc11E8A::Forest
            ),
            BackgroundArtPermutationLoc11E8A::Forest,
            "ASM loc_11E96 should fall through to `loc_11EA2` without disturbing the earlier Forest permutation when the Mountain compare fails"
        );
        assert_eq!(
            select_background_art_permutation_loc_11E96(
                0x0003,
                BackgroundArtPermutationLoc11E8A::Identity
            ),
            BackgroundArtPermutationLoc11E8A::Hill,
            "ASM loc_11E96 should select `Palette_Permutation_BGHill` when `Background_theme == Hills`"
        );
    }

    #[test]
    fn loc_11ea2_passes_the_selected_background_permutation_and_f000_base_into_decompress() {
        assert_eq!(
            prepare_background_art_decompress_loc_11EA2(BackgroundArtPermutationLoc11E8A::Mountain),
            BackgroundArtDecompressLoc11EA2 {
                tile_base: 0xF000,
                permutation: BackgroundArtPermutationLoc11E8A::Mountain,
            },
            "ASM loc_11EA2 should reload `d0` with `$F000` and preserve the selected Mountain background permutation into the shared `DecompressToRAM` call"
        );
        assert_eq!(
            prepare_background_art_decompress_loc_11EA2(BackgroundArtPermutationLoc11E8A::Forest),
            BackgroundArtDecompressLoc11EA2 {
                tile_base: 0xF000,
                permutation: BackgroundArtPermutationLoc11E8A::Forest,
            },
            "ASM loc_11EA2 should also leave the earlier Forest permutation intact before the shared background-art decompress"
        );
        assert_eq!(
            background_art_decompress_loc_11E8A_11EA2(0x0003),
            BackgroundArtDecompressLoc11EA2 {
                tile_base: 0xF000,
                permutation: BackgroundArtPermutationLoc11E8A::Hill,
            },
            "ASM loc_11E96 should feed the Hills permutation into the shared loc_11EA2 background-art decompress"
        );
    }

    #[test]
    fn loc_11ea2_prepares_the_hill_followup_background_decompress() {
        assert_eq!(
            background_followup_art_decompress_loc_11EA2_11EF2(0x0003, 0x0000),
            Some(BackgroundFollowupArtDecompressLoc11EF2 {
                tile_base: 0xF800,
                permutation: BackgroundFollowupArtPermutationLoc11EC4::BgHillAlt,
                art_source: BackgroundFollowupArtSourceLoc11EC4::HillAlt,
            }),
            "ASM loc_11EA2 should append `ThemeArtBack_HillAlt` with `Palette_Permutation_BGHill_alt` at VRAM `$F800` for Hills backgrounds"
        );
    }

    #[test]
    fn loc_11ec4_only_prepares_the_mountain_followup_background_decompress() {
        assert_eq!(
            prepare_mountain_background_art_loc_11EC4(0x0007),
            Some(BackgroundFollowupArtDecompressLoc11EC4 {
                tile_base: 0xFCA0,
                permutation: BackgroundFollowupArtPermutationLoc11EC4::Identity,
                art_source: BackgroundFollowupArtSourceLoc11EC4::CaveAlt,
            }),
            "ASM loc_11EC4 should prepare the Mountain-only follow-up RAM decompress with `ThemeArtBack_CaveAlt`, `Palette_Permutation_Identity`, and VRAM base `$FCA0`"
        );
        assert_eq!(
            prepare_mountain_background_art_loc_11EC4(0x0005),
            None,
            "ASM loc_11EC4 should fall through to `loc_11EFA` without a second background decompress when `Background_theme` is not Mountain"
        );
    }

    #[test]
    fn loc_11eea_switches_mountain_background_art_to_lightning_for_storm_weather() {
        let baseline = BackgroundFollowupArtDecompressLoc11EC4 {
            tile_base: 0xFCA0,
            permutation: BackgroundFollowupArtPermutationLoc11EC4::Identity,
            art_source: BackgroundFollowupArtSourceLoc11EC4::CaveAlt,
        };

        assert_eq!(
            apply_mountain_weather_override_loc_11EEA(0x0000, baseline),
            baseline,
            "ASM loc_11EEA should leave the `loc_11EC4` CaveAlt selection intact when the weather compare falls through"
        );
        assert_eq!(
            apply_mountain_weather_override_loc_11EEA(0x0002, baseline),
            BackgroundFollowupArtDecompressLoc11EC4 {
                tile_base: 0xFCA0,
                permutation: BackgroundFollowupArtPermutationLoc11EC4::BgMountain,
                art_source: BackgroundFollowupArtSourceLoc11EC4::MountainLightning,
            },
            "ASM loc_11EEA should replace `ThemeArtBack_CaveAlt` with `ThemeArtBack_MountainLightning` and switch to `Palette_Permutation_BGMountain` when `Level_Special_Effects == WeatherID_Storm`"
        );
        assert_eq!(
            apply_mountain_weather_override_loc_11EEA(0x0003, baseline),
            BackgroundFollowupArtDecompressLoc11EC4 {
                tile_base: 0xFCA0,
                permutation: BackgroundFollowupArtPermutationLoc11EC4::BgMountain,
                art_source: BackgroundFollowupArtSourceLoc11EC4::MountainLightning,
            },
            "ASM loc_11EEA should take the same lightning-art override when `Level_Special_Effects == WeatherID_Storm_and_Hail`"
        );
    }

    #[test]
    fn loc_11ef2_preserves_the_selected_followup_background_art_into_the_shared_decompress_tail() {
        let lightning_override = BackgroundFollowupArtDecompressLoc11EC4 {
            tile_base: 0xFCA0,
            permutation: BackgroundFollowupArtPermutationLoc11EC4::BgMountain,
            art_source: BackgroundFollowupArtSourceLoc11EC4::MountainLightning,
        };

        assert_eq!(
            prepare_followup_background_art_decompress_loc_11EF2(lightning_override),
            BackgroundFollowupArtDecompressLoc11EF2 {
                tile_base: 0xFCA0,
                permutation: BackgroundFollowupArtPermutationLoc11EC4::BgMountain,
                art_source: BackgroundFollowupArtSourceLoc11EC4::MountainLightning,
            },
            "ASM loc_11EF2 should preserve the lightning-art override from `loc_11EEA` while reloading `d0` with the shared `$FCA0` RAM-decompress base"
        );
        assert_eq!(
            prepare_followup_background_art_decompress_loc_11EF2(
                BackgroundFollowupArtDecompressLoc11EC4 {
                    tile_base: 0xFCA0,
                    permutation: BackgroundFollowupArtPermutationLoc11EC4::Identity,
                    art_source: BackgroundFollowupArtSourceLoc11EC4::CaveAlt,
                }
            ),
            BackgroundFollowupArtDecompressLoc11EF2 {
                tile_base: 0xFCA0,
                permutation: BackgroundFollowupArtPermutationLoc11EC4::Identity,
                art_source: BackgroundFollowupArtSourceLoc11EC4::CaveAlt,
            },
            "ASM loc_11EF2 should also leave the earlier `loc_11EC4` CaveAlt selection intact when the weather branches fall through"
        );
    }

    #[test]
    fn loc_11efa_decompresses_blocks_then_ingame_numbers_into_their_vram_slots() {
        assert_eq!(
            shared_vram_art_decompresses_loc_11EFA(),
            [
                SharedVramArtDecompressLoc11EFA {
                    tile_base: 0x4400,
                    art_source: SharedVramArtSourceLoc11EFA::Blocks,
                },
                SharedVramArtDecompressLoc11EFA {
                    tile_base: 0xD740,
                    art_source: SharedVramArtSourceLoc11EFA::IngameNumbers,
                },
            ],
            "ASM loc_11EFA should always decompress `ArtComp_992E4_Blocks` to VRAM `$4400` before loading `ArtComp_99F34_IngameNumbers` into VRAM `$D740`"
        );
    }

    #[test]
    #[allow(non_snake_case)]
    fn loc_11D46_copies_block_mappings_with_priority_or_mask() {
        let mut source = vec![0x0000_0001; 0xA2];
        source[0] = 0x8000_0001;
        source[1] = 0x0000_8001;

        let copied = copy_block_mappings_loc_11D46(&source, 0x8000_8000);

        assert_eq!(
            copied.len(),
            0xA2,
            "ASM loc_11D42/loc_11D46 should copy exactly $A2 longwords"
        );
        assert_eq!(
            copied[0], 0x8000_8001,
            "ASM loc_11D46: or.l keeps an already-set high priority bit"
        );
        assert_eq!(
            copied[1], 0x8000_8001,
            "ASM loc_11D46: or.l keeps an already-set low priority bit"
        );
    }

    #[test]
    fn loc_12830_reads_chunk_placements_until_terminator() {
        // ASM loc_12830 (kid.asm:25165-25173): chunk placement loop.
        // Reads (chunk_index, x_pos, y_pos) triplets until negative index.
        //   move.w  (a1)+,d0    ; chunk index
        //   bmi.s   loc_12842   ; terminate if negative
        //   move.w  (a1)+,d2    ; x pos
        //   sub.w   d5,d2       ; apply x_offset
        //   move.w  (a1)+,d3    ; y pos
        //   sub.w   d6,d3       ; apply y_offset

        // First entry terminates immediately
        let result = background_mappings_read_chunk_loc_12830(&[0x8001], 0, 0);
        assert!(
            result.is_none(),
            "ASM loc_12830: negative chunk index should terminate"
        );

        // Normal chunk placement (no offsets)
        let layout = &[0x0005, 0x0100, 0x0200, 0x8000];
        let (chunk, remaining) = background_mappings_read_chunk_loc_12830(layout, 0, 0).unwrap();
        assert_eq!(
            chunk.chunk_index, 5,
            "ASM loc_12830: should read chunk index"
        );
        assert_eq!(
            chunk.x_pos, 0x0100,
            "ASM loc_12830: should read x position unchanged when offset is 0"
        );
        assert_eq!(
            chunk.y_pos, 0x0200,
            "ASM loc_12830: should read y position unchanged when offset is 0"
        );
        assert_eq!(
            remaining,
            &[0x8000],
            "ASM loc_12830: should return remaining layout after reading 3 words"
        );

        // Chunk placement with offsets (cross-level copy)
        let layout = &[0x0003, 0x0100, 0x0200];
        let (chunk, _) = background_mappings_read_chunk_loc_12830(layout, 0x10, 0x20).unwrap();
        assert_eq!(
            chunk.x_pos,
            0x0100 - 0x10,
            "ASM loc_12830: should subtract x_offset from x position"
        );
        assert_eq!(
            chunk.y_pos,
            0x0200 - 0x20,
            "ASM loc_12830: should subtract y_offset from y position"
        );

        // Negative x/y positions preserved with wrapping
        let layout: &[u16] = &[0x0002, 0x0000, 0xFFFF];
        let (chunk, _) = background_mappings_read_chunk_loc_12830(layout, 0, 0).unwrap();
        assert_eq!(
            chunk.y_pos, -1i16,
            "ASM loc_12830: should preserve signed y position"
        );
    }

    #[test]
    fn loc_12842_terminates_background_mappings_loop() {
        // ASM loc_12842 (kid.asm:25176-25178): loop termination.
        //   movem.l (sp)+,d5-d7  ; restore registers
        //   rts                    ; return
        // Reached when loc_12830 reads negative chunk index.
        let terminator = background_mappings_terminator_loc_12842();
        assert!(
            terminator,
            "ASM loc_12842: should return true for terminator"
        );
    }

    #[test]
    fn loc_12910_loads_mountain_background_tileset() {
        // ASM loc_12910 (kid.asm:25274-25291): Mountain theme background tileset load.
        //   move.w  #$2780,d0           ; pattern VRAM offset
        //   move.l  (a3)+,a0            ; load first art pointer
        //   lea     ($FFFF87B2).w,a1    ; dest: VRAM buffer 1
        //   bsr.w   EniDec
        //   move.l  (a3)+,a0            ; load second art pointer
        //   lea     ($FFFF8EB2).w,a1    ; dest: VRAM buffer 2
        //   bsr.w   EniDec
        //   move.l  (a3)+,a0            ; load third art pointer
        //   lea     ($FFFF95B2).w,a1    ; dest: VRAM buffer 3
        //   bsr.w   EniDec
        //   cmpi.w  #2,(Level_Special_Effects).w
        //   beq.s   loc_1294A           ; hail path for storm
        //   cmpi.w  #3,(Level_Special_Effects).w
        //   beq.s   loc_1294A           ; hail path for storm+hail
        //   move.w  #$7E5,d0           ; non-storm 4th pattern
        //   move.l  (a3)+,a0
        //   bra.s   loc_12952

        // Clear weather: no hail overlay needed
        let result_clear = background_mountain_tileset_loc_12910(0);
        assert_eq!(
            result_clear.pattern_vram_offset, 0x2780,
            "ASM loc_12910: should use pattern $2780 for mountain bg"
        );
        assert_eq!(
            result_clear.vram_buffer1_addr, 0xFFFF_87B2,
            "ASM loc_12910: should target VRAM buffer 1 at $FFFF87B2"
        );
        assert_eq!(
            result_clear.vram_buffer2_addr, 0xFFFF_8EB2,
            "ASM loc_12910: should target VRAM buffer 2 at $FFFF8EB2"
        );
        assert_eq!(
            result_clear.vram_buffer3_addr, 0xFFFF_95B2,
            "ASM loc_12910: should target VRAM buffer 3 at $FFFF95B2"
        );
        assert!(
            !result_clear.needs_hail_overlay_at_1294_a,
            "ASM loc_12910: clear weather should not need hail overlay (falls through to $7E5 load)"
        );

        // Storm weather: hail overlay needed
        let result_storm = background_mountain_tileset_loc_12910(2);
        assert!(
            result_storm.needs_hail_overlay_at_1294_a,
            "ASM loc_12910: storm (Level_Special_Effects=2) should set hail overlay flag"
        );

        // Storm+hail: hail overlay needed
        let result_storm_hail = background_mountain_tileset_loc_12910(3);
        assert!(
            result_storm_hail.needs_hail_overlay_at_1294_a,
            "ASM loc_12910: storm+hail (Level_Special_Effects=3) should set hail overlay flag"
        );
    }

    #[test]
    fn loc_1294_a_loads_hail_overlay_pattern() {
        // ASM loc_1294A (kid.asm:25294-25296): Hail overlay for stormy mountain.
        //   loc_1294A:	; hailstorm
        //   move.w  #$27E5,d0           ; hail pattern VRAM offset
        //   move.l  4(a3),a0           ; skip pointer, load from offset +4
        // Falls through to loc_12952.

        let result = background_hail_overlay_loc_1294A();
        assert_eq!(
            result.pattern_vram_offset, 0x27E5,
            "ASM loc_1294A: should use hail pattern offset $27E5 (not $7E5)"
        );
        assert_eq!(
            result.vram_buffer4_addr, 0xFFFF_98F2,
            "ASM loc_1294A: should target VRAM buffer 4 at $FFFF98F2 (via loc_12952)"
        );
    }

    #[test]
    fn loc_12952_loads_final_pattern_to_vram() {
        // ASM loc_12952 (kid.asm:25298-25301): Final pattern decompress.
        //   lea     ($FFFF98F2).w,a1    ; dest: VRAM buffer 4
        //   bsr.w   EniDec              ; decompress
        //   move.l  (sp)+,a0            ; restore return address
        //   rts

        // Non-storm path: pattern $7E5 passed from loc_12910
        let result_clear = background_final_pattern_loc_12952(0x07E5);
        assert_eq!(
            result_clear.pattern_vram_offset, 0x07E5,
            "ASM loc_12952: should preserve non-storm pattern offset $7E5"
        );
        assert_eq!(
            result_clear.vram_buffer4_addr, 0xFFFF_98F2,
            "ASM loc_12952: should target VRAM buffer 4 at $FFFF98F2"
        );

        // Storm path: pattern $27E5 passed from loc_1294A
        let result_storm = background_final_pattern_loc_12952(0x27E5);
        assert_eq!(
            result_storm.pattern_vram_offset, 0x27E5,
            "ASM loc_12952: should preserve storm pattern offset $27E5"
        );
        assert_eq!(
            result_storm.vram_buffer4_addr, 0xFFFF_98F2,
            "ASM loc_12952: should target VRAM buffer 4 at $FFFF98F2 for both paths"
        );
    }

    #[test]
    fn loc_129e8_loads_juggernaut_skull_frame_2_into_the_same_vram_span() {
        let result = juggernaut_skull_art_dma_loop_loc_129E8();

        assert_eq!(
            result.vdp_command, 0xC000_00CD,
            "ASM loc_129E8 should keep writing to vdpComm($CDC0,VRAM,WRITE)"
        );
        assert_eq!(
            result.art_source, 0x0001_2A8C,
            "ASM loc_129E8 should advance `a1` to ArtUnc_12A8C for Juggernaut skull frame 2"
        );
        assert_eq!(
            result.longword_count, 32,
            "ASM loc_129E8 seeds `d0=#1F`, so `dbf d0,loc_129E8` copies 32 longwords"
        );
    }

    #[test]
    fn loc_129fc_loads_the_final_star_art_block_into_vram_df40() {
        let result = juggernaut_star_art_dma_loop_loc_129FC();

        assert_eq!(
            result.vdp_command, 0x4000_00DF,
            "ASM loc_129FC should retarget the final DMA loop to vdpComm($DF40,VRAM,WRITE)"
        );
        assert_eq!(
            result.art_source, 0x0001_2B0C,
            "ASM loc_129FC should load the last art block from ArtUnc_12B0C"
        );
        assert_eq!(
            result.longword_count, 32,
            "ASM loc_129FC also seeds `d0=#1F`, so the closing `dbf` loop copies 32 longwords"
        );
    }

    #[test]
    fn loc_12b9c_loads_flag_bottom_art_into_vram_d2c0() {
        let result = flag_bottom_art_dma_loop_loc_12B9C();

        assert_eq!(
            result.vdp_command, 0xC000_00D2,
            "ASM loc_12B9C should program vdpComm($D2C0,VRAM,WRITE) before the copy loop"
        );
        assert_eq!(
            result.art_source, 0x0001_2BA4,
            "ASM loc_12B9C should stream Flag_bottom art from ArtUnc_12BA4"
        );
        assert_eq!(
            result.longword_count, 32,
            "ASM loc_12B9C seeds `d0=#1F`, so `dbf d0,loc_12B9C` copies 32 longwords"
        );
    }

    #[test]
    fn loc_12c34_loads_hitpoint_display_art_into_vram_45a0() {
        let result = hitpoint_display_art_dma_loop_loc_12C34();

        assert_eq!(
            result.vdp_command, 0xA000_00C5,
            "ASM loc_12C34 should program vdpComm($45A0,VRAM,WRITE) before the copy loop"
        );
        assert_eq!(
            result.art_source, 0x0001_2C3C,
            "ASM loc_12C34 should stream hitpoint display art from ArtUnc_12C3C"
        );
        assert_eq!(
            result.longword_count, 32,
            "ASM loc_12C34 seeds `d0=#1F`, so `dbf d0,loc_12C34` copies 32 longwords"
        );
    }
}
