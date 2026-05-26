/// Software VDP state for non-gameplay screens.
///
/// During boot/title/intro sequences, the original game drives the display
/// entirely through VDP register writes, DMA art uploads, and plane nametable
/// setup. This struct mirrors that state so the proven `kid_md::render_vdp_into`
/// renderer can produce accurate frames.
///
/// The game init register values come from ASM VDPGameInitValues (kid.asm:550).
/// Art is Saxman-compressed in the ROM and decompressed to VRAM.  Palettes are
/// raw 9-bit BGR words written to CRAM.
use anyhow::{Context, Result};
use kid_data::AssetManifest;

use crate::boot::{
    VDP_GAME_INIT_VALUES, intro_text_buggy_substitution_loc_1C7DC, title_text_word_writes_sub_1B7B6,
};
use crate::constants::INTRO_SEQUENCE_3_KID_VISIBLE_ELAPSED_FRAMES;

const VRAM_SIZE: usize = 0x1_0000;
const CRAM_SIZE: usize = 0x80;
const VSRAM_SIZE: usize = 0x50;

/// ASM Load_SegaScreen (kid.asm:25444): art destination in VRAM.
const SEGA_ART_VRAM_BYTE_ADDR: usize = 0x1780;

/// ASM Load_SegaScreen: nametable write addresses for the 4 tile rows.
/// Each row is 12 tiles wide starting at column 14, rows 12-15 of Scroll A.
const SEGA_NAMETABLE_VRAM_ADDRS: [usize; 4] = [0x061C, 0x069C, 0x071C, 0x079C];

/// ASM Load_SegaScreen: first nametable tile word — priority=1, palette=0,
/// tile index $BC.
const SEGA_FIRST_TILE_WORD: u16 = 0x80BC;

/// Tiles per nametable row for the Sega logo.
const SEGA_TILES_PER_ROW: usize = 12;

/// Total number of Sega palette words copied at loc_12DFE.
const SEGA_INITIAL_PALETTE_WORD_COUNT: usize = 13;

/// Fade strip: 11 words written to Palette_Buffer+$4 (CRAM word offset 2).
const SEGA_FADE_STRIP_WORDS: usize = 11;
const SEGA_FADE_STRIP_CRAM_WORD_OFFSET: usize = 2;

/// Number of frames the initial wait loop runs before the fade begins.
/// ASM loc_12E6C: `moveq #$A,d2` → dbf runs 11 iterations.
const SEGA_INITIAL_WAIT_FRAMES: usize = 11;

/// Fade loop d1 init (frame delay counter) and d3 init (palette cursor).
const SEGA_FADE_FRAME_DELAY_INIT: i16 = 0x14;
const SEGA_FADE_PALETTE_CURSOR_INIT: i16 = 0x28;

#[derive(Debug, Clone)]
pub struct VdpState {
    pub registers: [u8; 0x20],
    pub vram: Vec<u8>,
    pub cram: Vec<u8>,
    pub vsram: Vec<u8>,
}

impl VdpState {
    fn new() -> Self {
        Self {
            registers: [0; 0x20],
            vram: vec![0; VRAM_SIZE],
            cram: vec![0; CRAM_SIZE],
            vsram: vec![0; VSRAM_SIZE],
        }
    }

    /// Write tile data to VRAM at the given byte address.
    /// Silently clips if the address or data extends past VRAM bounds.
    fn write_vram(&mut self, byte_addr: usize, data: &[u8]) {
        if byte_addr >= self.vram.len() {
            return;
        }
        let end = (byte_addr + data.len()).min(self.vram.len());
        let copy_len = end - byte_addr;
        self.vram[byte_addr..end].copy_from_slice(&data[..copy_len]);
    }

    /// Write a single big-endian word to VRAM at the given byte address.
    fn write_vram_word(&mut self, byte_addr: usize, word: u16) {
        if byte_addr + 1 < self.vram.len() {
            self.vram[byte_addr] = (word >> 8) as u8;
            self.vram[byte_addr + 1] = word as u8;
        }
    }

    /// Write palette words to CRAM at the given word offset.
    ///
    /// Each word is 2 bytes in Mega Drive 0BGR format (3 bits per channel).
    /// The bound is expressed as a 2-byte slice borrow rather than
    /// `byte_offset + 1 < cram.len()` because cargo-mutants flagged the
    /// latter form's `<`/`+` mutations as equivalent: `byte_offset =
    /// (word_offset + i) * 2` is always even and `CRAM_SIZE = 0x80` is even,
    /// so `byte_offset + 1` can never equal `cram.len()`. Using
    /// `get_mut(byte_offset..byte_offset + 2)` makes the 2-byte span explicit
    /// so any arithmetic mutation produces a slice whose length mismatches
    /// `word.to_be_bytes()` and `copy_from_slice` panics.
    fn write_cram_words(&mut self, word_offset: usize, words: &[u16]) {
        for (i, &word) in words.iter().enumerate() {
            let byte_offset = (word_offset + i) * 2;
            if let Some(slot) = self.cram.get_mut(byte_offset..byte_offset + 2) {
                slot.copy_from_slice(&word.to_be_bytes());
            }
        }
    }
}

/// Cached Sega screen art and palette data, loaded once from the asset bundle
/// and reused across frames (only the palette changes during the fade).
#[derive(Debug, Clone)]
pub(crate) struct SegaScreenData {
    /// Decompressed tile data for VRAM.
    art_bytes: Vec<u8>,
    /// All palette words from SegaLogo.bin (33 words = 66 bytes).
    /// Words 0..12 are the initial palette, the rest are fade strip data.
    palette_words: Vec<u16>,
}

/// Build a VdpState for the Sega screen at the given elapsed frame.
///
/// ASM Load_SegaScreen (kid.asm:25444):
/// - Decompress `ArtComp_12F30_Sega` to VRAM $1780
/// - Copy 13 palette words from `Pal_12EEC` to CRAM line 0
/// - Write 4 rows × 12 tiles of nametable entries starting at tile $BC
/// - Run a palette fade animation over subsequent frames
pub(crate) fn build_sega_screen_vdp(data: &SegaScreenData, elapsed: usize) -> VdpState {
    let mut vdp = VdpState::new();

    // ASM VDPGameInitValues → register init, then reg 1 = $74 (display on).
    for (i, &value) in VDP_GAME_INIT_VALUES.iter().enumerate() {
        vdp.registers[i] = value;
    }
    // ASM loc_12E48: `move.w #$8174,4(a6)` — display on, VInt on, DMA on.
    vdp.registers[1] = 0x74;

    // Decompress art to VRAM at byte address $1780 (tile index $BC).
    vdp.write_vram(SEGA_ART_VRAM_BYTE_ADDR, &data.art_bytes);

    // Write nametable: 4 rows × 12 consecutive tile words starting at $80BC.
    // ASM calls sub_12E3C four times; d1 carries across calls.
    let mut tile_word = SEGA_FIRST_TILE_WORD;
    for &nametable_addr in &SEGA_NAMETABLE_VRAM_ADDRS {
        for col in 0..SEGA_TILES_PER_ROW {
            vdp.write_vram_word(nametable_addr + col * 2, tile_word);
            tile_word = tile_word.wrapping_add(1);
        }
    }

    // Initial palette: copy first 13 words to CRAM line 0.
    let initial_palette =
        &data.palette_words[..SEGA_INITIAL_PALETTE_WORD_COUNT.min(data.palette_words.len())];
    vdp.write_cram_words(0, initial_palette);

    // ASM palette fade animation (loc_12E96 → loc_12ED2):
    // After the initial 11-frame wait, every 4th frame copies 11 words from
    // a sliding offset into CRAM words 2..12.  The cursor starts at $28 and
    // decreases by 2 each step until it goes negative.
    //
    // Using `saturating_sub` (rather than an `if elapsed > N` guard) keeps
    // the boundary at elapsed == SEGA_INITIAL_WAIT_FRAMES unambiguous: the
    // fade loop body only executes when fade_elapsed > 0, so the strict-
    // versus-non-strict comparison can't survive as an equivalent mutant.
    let fade_elapsed = elapsed.saturating_sub(SEGA_INITIAL_WAIT_FRAMES);
    let mut frame_delay = SEGA_FADE_FRAME_DELAY_INIT;
    let mut palette_cursor = SEGA_FADE_PALETTE_CURSOR_INIT;

    for _ in 0..fade_elapsed {
        frame_delay -= 1;
        if frame_delay >= 0 {
            continue;
        }
        // Reset delay counter.
        frame_delay = 3;
        if palette_cursor < 0 {
            continue;
        }
        // ASM loc_12ED2: copy 11 words from Pal_12EEC+6+cursor to CRAM+$4.
        // Pal_12EEC+6 = byte 6 from the start of the palette block,
        // which is word 3 in our palette_words array (after the 2-byte header
        // that is NOT included in palette_words — the header was consumed
        // during loading).  But the fade data in the ROM is at Pal_12EEC+6,
        // and the first 2 bytes of Pal_12EEC are the header bytes (dest=0,
        // count=$0C).  So Pal_12EEC+6 = 4 bytes past the palette data start,
        // i.e. word index 2 + (cursor/2).
        //
        // Actually: Pal_12EEC byte layout:
        //   [0]=dest_offset, [1]=dbf_count, [2..67]=SegaLogo.bin contents
        // Pal_12EEC+6 = byte 6 = SegaLogo.bin byte 4 = word 2 of palette_words
        // The cursor d0=d3 is a byte offset added to Pal_12EEC+6, so the
        // source word index = 2 + cursor/2.
        let source_word_index = 2 + (palette_cursor as usize) / 2;
        if source_word_index + SEGA_FADE_STRIP_WORDS <= data.palette_words.len() {
            let strip =
                &data.palette_words[source_word_index..source_word_index + SEGA_FADE_STRIP_WORDS];
            vdp.write_cram_words(SEGA_FADE_STRIP_CRAM_WORD_OFFSET, strip);
        }
        palette_cursor -= 2;
    }

    vdp
}

/// Load and cache Sega screen art + palette data from the asset bundle.
pub(crate) fn load_sega_screen_data(manifest: &AssetManifest) -> Result<SegaScreenData> {
    let scenes_root = manifest.output_root.join("scenes");

    // Load and decompress Saxman-compressed Sega logo art.
    let art_path = scenes_root.join("artcomp/Sega_Logo.bin");
    let art_compressed = std::fs::read(&art_path)
        .with_context(|| format!("failed to read Sega logo art: {}", art_path.display()))?;
    let art_bytes =
        kid_data::decode_saxman(&art_compressed).context("failed to decompress Sega logo art")?;

    // Load raw palette data (SegaLogo.bin = 66 bytes = 33 big-endian words).
    let palette_path = scenes_root.join("palette/SegaLogo.bin");
    let palette_raw = std::fs::read(&palette_path).with_context(|| {
        format!(
            "failed to read Sega logo palette: {}",
            palette_path.display()
        )
    })?;
    let palette_words: Vec<u16> = palette_raw
        .chunks_exact(2)
        .map(|chunk| u16::from_be_bytes([chunk[0], chunk[1]]))
        .collect();

    Ok(SegaScreenData {
        art_bytes,
        palette_words,
    })
}

/// Nametable row stride in bytes for a 64-column scroll plane.
/// VDP register $10 = $01 → 64×32 cells, each entry 2 bytes → 128 bytes/row.
const NAMETABLE_ROW_STRIDE_BYTES: usize = 128;

// ─── Intro sequence (segments 0–4) ──────────────────────────────────────────

/// Cached decompressed assets for intro sequences.
#[derive(Debug, Clone)]
pub(crate) struct IntroSub1B850Data {
    /// 5 decompressed Saxman art chunks in upload order.
    art_chunks: [(usize, Vec<u8>); 5],
    /// Decoded Enigma tilemap for plane A (40×28 tile words).
    plane_a_tilemap: Vec<u16>,
    /// 28 rows of repeated words for plane B (each word stamped 64 times).
    plane_b_rows: [[u16; 64]; 28],
    /// Segment 0 palette words (Pal_1C28A — intro1_wildside.bin).
    palette_segment_0: Vec<u16>,
    /// Segment 1 palette words (Pal_1C2F0 — intro2_sky.bin).
    palette_segment_1: Vec<u16>,
    /// Segment 2+ palette words (Pal_1C346 — intro3_alley.bin).
    palette_segment_2: Vec<u16>,
    /// Shared Hologram_background art used by Load_IntroSequence2/4.
    hologram_background_art: Vec<u8>,
    /// Hologram_blocks art used by Load_IntroSequence2.
    hologram_blocks_art: Vec<u8>,
    /// Intro text letters used by Load_IntroSequence2/4.
    intro_text_letters_art: Vec<u8>,
    /// MapEni_CC0E decoded with the Load_IntroSequence2 base tile `$2114`.
    intro_sequence_2_hologram_tilemap: Vec<u16>,
    /// MapEni_CC0E decoded with the Load_IntroSequence4 base tile `$20BC`.
    intro_sequence_4_hologram_tilemap: Vec<u16>,
    /// IntroSequence2 foreground tilemap (`unk_2E7C6`) decoded with base `$00BC`.
    intro_sequence_2_tilemap: Vec<u16>,
    /// Load_IntroSequence5 common art uploads in ROM order.
    intro_sequence_5_common_art_chunks: [(usize, Vec<u8>); 3],
    /// Load_IntroSequence5 helmet art variants selected by `$FFFFFC82`.
    intro_sequence_5_helmet_art_variants: [Vec<u8>; 3],
    /// Pal_1C3C6 overlay applied by sub_1C512 when `$FFFFFC82 == 1`.
    intro_sequence_5_palette_variant_1_overlay: Vec<u16>,
    /// Pal_1C3D4 overlay applied by sub_1C512 when `$FFFFFC82 != 0 && != 1`.
    intro_sequence_5_palette_variant_2_overlay: Vec<u16>,
    /// IntroSequence5 tilemap (`MapEni_2E154`) decoded with base `$00BC`.
    intro_sequence_5_tilemap: Vec<u16>,
    /// IntroSequence5 staged `loc_1BC36` / `sub_1C034` tile upload maps,
    /// decoded with base `$A340`.
    intro_sequence_5_frame_tilemaps: Vec<Vec<u16>>,
    /// IntroSequence5 `sub_1C0EE` panel map (`unk_2FDA6`) decoded with base `$A340`.
    intro_sequence_5_panel_tilemap: Vec<u16>,
}

/// Load and cache intro sub_1B850 assets from the asset bundle.
///
/// ASM sub_1B850 (kid.asm:30172): decompress 5 art pieces to VRAM, decode
/// `unk_2FDE0` as 40×28 tilemap to plane A, decode `unk_2FDCE` and stamp
/// each of 28 words across full 64-word rows to plane B at $E000.
pub(crate) fn load_intro_sub_1b850_data(manifest: &AssetManifest) -> Result<IntroSub1B850Data> {
    let scenes = manifest.output_root.join("scenes");

    // Art uploads — label→filename mapping from kid.asm / extract pipeline.
    let art_files: [(u16, &str); 5] = [
        (0x1780, "artcomp/Wild_Side_arcade.bin"),
        (0x2740, "artcomp/Background_for_intro_(Wild_Side).bin"),
        (0x2800, "artcomp/Wild_Side_door_and_inside.bin"),
        (0x3AC0, "artcomp/Kids_hanging_around_Wild_Side.bin"),
        (0xFA60, "artcomp/Intro_text_letters.bin"),
    ];
    let mut art_chunks = Vec::new();
    for (vram_addr, filename) in &art_files {
        let decompressed = load_saxman_art(&scenes.join(filename))?;
        // VDP addresses in kid_data are byte addresses (the VDP control port
        // encodes byte addresses, not word addresses, despite the field name).
        art_chunks.push((*vram_addr as usize, decompressed));
    }
    let art_chunks: [(usize, Vec<u8>); 5] = art_chunks
        .try_into()
        .map_err(|_| anyhow::anyhow!("expected 5 art chunks"))?;

    // Decode plane A tilemap: unk_2FDE0 with base tile $80BC → 40×28 tile words.
    // ASM sub_1B850: `move.w #$80BC,d0` then `jsr (j_EniDec)`.
    let tilemap_path = scenes.join("mapeni/intro1_wildside.bin");
    let tilemap_raw = std::fs::read(&tilemap_path)
        .with_context(|| format!("failed to read intro tilemap: {}", tilemap_path.display()))?;
    let plane_a_tilemap =
        kid_data::decode_enigma(&tilemap_raw, 0x80BC).context("failed to decode intro tilemap")?;

    // Decode plane B row words: unk_2FDCE with base tile $413A → 28+ words.
    // ASM sub_1B850: `move.w #$413A,d0` then `jsr (j_EniDec)`.
    // Each of the first 28 decoded words is stamped across a full 64-word row.
    let row_tilemap_path = scenes.join("mapeni/2FDCE.bin");
    let row_raw = std::fs::read(&row_tilemap_path).with_context(|| {
        format!(
            "failed to read intro row tilemap: {}",
            row_tilemap_path.display()
        )
    })?;
    let decoded_row_words =
        kid_data::decode_enigma(&row_raw, 0x413A).context("failed to decode intro row tilemap")?;

    // Use kid_data to build the load plan (validates and stamps rows).
    let load_plan = kid_data::title_intro_load_plan_sub_1B850(&decoded_row_words)
        .context("failed to build intro load plan")?;
    let plane_b_rows = load_plan.repeated_rows_upload.rows;

    // Load palettes for each segment variant.
    let palette_segment_0 = load_palette_words(&scenes.join("palette/intro1_wildside.bin"))?;
    let palette_segment_1 = load_palette_words(&scenes.join("palette/intro2_sky.bin"))?;
    let palette_segment_2 = load_palette_words(&scenes.join("palette/intro3_alley.bin"))?;

    let hologram_background_art = load_saxman_art(&scenes.join("artcomp/Hologram_background.bin"))?;
    let hologram_blocks_art = load_saxman_art(&scenes.join("artcomp/Hologram_blocks.bin"))?;
    let intro_text_letters_art = load_saxman_art(&scenes.join("artcomp/Intro_text_letters.bin"))?;

    let hologram_tilemap_raw = std::fs::read(scenes.join("mapeni/hologram_background.bin"))
        .context("failed to read hologram background tilemap")?;
    let intro_sequence_2_hologram_tilemap = kid_data::decode_enigma(&hologram_tilemap_raw, 0x2114)
        .context("failed to decode intro sequence 2 hologram tilemap")?;
    let intro_sequence_4_hologram_tilemap = kid_data::decode_enigma(&hologram_tilemap_raw, 0x20BC)
        .context("failed to decode intro sequence 4 hologram tilemap")?;

    let intro_sequence_2_tilemap_raw = std::fs::read(scenes.join("mapeni/intro2_sky.bin"))
        .context("failed to read intro sequence 2 tilemap")?;
    let intro_sequence_2_tilemap = kid_data::decode_enigma(&intro_sequence_2_tilemap_raw, 0x00BC)
        .context("failed to decode intro sequence 2 tilemap")?;

    let intro_sequence_5_common_art_files: [(u16, &str); 3] = [
        (0x1780, "artcomp/Dark_alley_from_intro.bin"),
        (0x4D80, "artcomp/P_block_from_intro.bin"),
        (0x6800, "artcomp/The_kid_from_intro.bin"),
    ];
    let mut intro_sequence_5_common_art_chunks = Vec::new();
    for (vram_addr, filename) in &intro_sequence_5_common_art_files {
        intro_sequence_5_common_art_chunks.push((
            *vram_addr as usize,
            load_saxman_art(&scenes.join(filename))?,
        ));
    }
    let intro_sequence_5_common_art_chunks: [(usize, Vec<u8>); 3] =
        intro_sequence_5_common_art_chunks
            .try_into()
            .map_err(|_| anyhow::anyhow!("expected 3 intro sequence 5 common art chunks"))?;
    let intro_sequence_5_helmet_art_files = [
        // ASM loc_1B100: byte_213D9 when `$FFFFFC82 == 0`.
        "artcomp/Red_Stealth_helmet_from_intro.bin",
        // ASM loc_1B100: byte_219C1 when `$FFFFFC82 == 1`.
        "artcomp/Maniaxe_helmet_from_intro.bin",
        // ASM loc_1B100: byte_216B7 for every other non-zero value.
        "artcomp/Juggernaut_helmet_from_intro.bin",
    ];
    let mut intro_sequence_5_helmet_art_variants = Vec::new();
    for filename in intro_sequence_5_helmet_art_files {
        intro_sequence_5_helmet_art_variants.push(load_saxman_art(&scenes.join(filename))?);
    }
    let intro_sequence_5_helmet_art_variants: [Vec<u8>; 3] = intro_sequence_5_helmet_art_variants
        .try_into()
        .map_err(|_| anyhow::anyhow!("expected 3 intro sequence 5 helmet art variants"))?;
    let intro_sequence_5_palette_variant_1_overlay =
        load_palette_words(&scenes.join("palette/title_maniaxe_mask.bin"))?;
    let intro_sequence_5_palette_variant_2_overlay =
        load_palette_words(&scenes.join("palette/title_juggernaut_mask.bin"))?;
    let intro_sequence_5_tilemap_raw = std::fs::read(scenes.join("mapeni/intro3_alley.bin"))
        .context("failed to read intro sequence 5 tilemap")?;
    let intro_sequence_5_tilemap = kid_data::decode_enigma(&intro_sequence_5_tilemap_raw, 0x00BC)
        .context("failed to decode intro sequence 5 tilemap")?;
    let mut intro_sequence_5_frame_tilemaps = Vec::new();
    for frame_index in 0..8u16 {
        let plan = kid_data::intro_sequence_5_frame_load_plan_sub_1C034(frame_index)
            .context("failed to build intro sequence 5 frame upload plan")?;
        let filename = asset_label_mapeni_filename(plan.tilemap_load.asset_label);
        let raw = std::fs::read(scenes.join("mapeni").join(&filename)).with_context(|| {
            format!("failed to read intro sequence 5 frame tilemap: {filename}")
        })?;
        let decoded = kid_data::decode_enigma(&raw, plan.tilemap_load.vram_word_address)
            .with_context(|| {
                format!("failed to decode intro sequence 5 frame tilemap: {filename}")
            })?;
        intro_sequence_5_frame_tilemaps.push(decoded);
    }
    let panel_plan = kid_data::intro_sequence_5_panel_load_plan_sub_1C0EE(3, 0x13);
    let panel_filename = asset_label_mapeni_filename(panel_plan.tilemap_load.asset_label);
    let panel_raw =
        std::fs::read(scenes.join("mapeni").join(&panel_filename)).with_context(|| {
            format!("failed to read intro sequence 5 panel tilemap: {panel_filename}")
        })?;
    let intro_sequence_5_panel_tilemap =
        kid_data::decode_enigma(&panel_raw, panel_plan.tilemap_load.vram_word_address)
            .with_context(|| {
                format!("failed to decode intro sequence 5 panel tilemap: {panel_filename}")
            })?;

    Ok(IntroSub1B850Data {
        art_chunks,
        plane_a_tilemap,
        plane_b_rows,
        palette_segment_0,
        palette_segment_1,
        palette_segment_2,
        hologram_background_art,
        hologram_blocks_art,
        intro_text_letters_art,
        intro_sequence_2_hologram_tilemap,
        intro_sequence_4_hologram_tilemap,
        intro_sequence_2_tilemap,
        intro_sequence_5_common_art_chunks,
        intro_sequence_5_helmet_art_variants,
        intro_sequence_5_palette_variant_1_overlay,
        intro_sequence_5_palette_variant_2_overlay,
        intro_sequence_5_tilemap,
        intro_sequence_5_frame_tilemaps,
        intro_sequence_5_panel_tilemap,
    })
}

/// Build VdpState for an intro sequence segment (0–4).
///
/// Segments 0 and 2 use sub_1B850 art/tilemaps with segment-specific palettes.
/// Segments 1, 3, and 4 follow the separate Load_IntroSequence2/4/5 loaders.
#[cfg(test)]
fn build_intro_sequence_vdp(data: &IntroSub1B850Data, segment: usize) -> VdpState {
    build_intro_sequence_vdp_with_title_variant(data, segment, 1)
}

#[cfg(test)]
pub(crate) fn build_intro_sequence_vdp_with_title_variant(
    data: &IntroSub1B850Data,
    segment: usize,
    title_variant: u8,
) -> VdpState {
    build_intro_sequence_vdp_base_with_title_variant(data, segment, title_variant)
}

pub(crate) fn build_intro_sequence_vdp_with_title_variant_at_elapsed(
    data: &IntroSub1B850Data,
    segment: usize,
    title_variant: u8,
    elapsed: usize,
    intro_sequence_3_roster_index: usize,
) -> VdpState {
    let mut vdp = build_intro_sequence_vdp_base_with_title_variant(data, segment, title_variant);
    if segment == 0 && elapsed >= INTRO_SEQUENCE_1_PROMPT_VISIBLE_ELAPSED_SUB_1C54A {
        overlay_intro_sequence_1_prompt_sprite_sub_1C54A(&mut vdp, elapsed);
    }
    if segment == 2 && elapsed >= INTRO_SEQUENCE_3_KID_VISIBLE_ELAPSED_FRAMES {
        overlay_intro_sequence_3_sprite_table_loc_1AEEC(
            &mut vdp,
            intro_sequence_3_roster_index,
            elapsed - INTRO_SEQUENCE_3_KID_VISIBLE_ELAPSED_FRAMES,
        );
    }
    if segment == 4 {
        overlay_intro_sequence_5_tile_worker_loc_1BC36(&mut vdp, data, elapsed);
        overlay_intro_sequence_5_sprite_table_loc_1BCD0(&mut vdp, title_variant, elapsed);
    }
    overlay_intro_text_worker_loc_1c7a0(&mut vdp, segment, elapsed);
    vdp
}

const INTRO_SEQUENCE_1_PROMPT_VISIBLE_ELAPSED_SUB_1C54A: usize = 43;

#[allow(non_snake_case)]
fn overlay_intro_sequence_1_prompt_sprite_sub_1C54A(vdp: &mut VdpState, _elapsed: usize) {
    let mut sprite_entries = Vec::new();
    append_intro_sequence_prompt_sprite_sub_1C54A(&mut sprite_entries);
    write_sprite_table_entries(
        vdp,
        sprite_table_base_byte_addr(&vdp.registers),
        &sprite_entries,
    );
}

#[allow(non_snake_case)]
fn append_intro_sequence_prompt_sprite_sub_1C54A(entries: &mut Vec<HardwareSpriteEntry>) {
    // ASM sub_1B8F4 -> sub_1C54A (kid.asm:30194,31288): after the Wild Side
    // VDP setup, the controller allocates a fixed priority sprite using
    // `unk_E109E` at VRAM tile `$140`. That art is the door/interior block
    // uploaded by sub_1B850's `Wild_Side_door_and_inside.bin` DMA.
    let prompt = crate::boot::title_menu_prompt_sprite_sub_1C54A();
    append_intro_sprite_entries_make_sprite_from_gfx_object_loc_129E(
        entries,
        IntroGfxObjectForSpriteTable {
            x_pos: prompt.x_pos,
            y_pos: prompt.y_pos,
            palette_line: 0,
            vram_tile: prompt.vram_tile,
            priority: u8::from(prompt.priority_flag),
            x_direction: 0,
        },
        IntroSpriteFrameVram {
            sprite_index: 0,
            x_center: 0,
            y_center: 0,
            width: 0x3E,
            height: 0x2F,
        },
    );
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct IntroSpriteFrameVram {
    sprite_index: u16,
    x_center: i16,
    y_center: i16,
    width: u16,
    height: u16,
}

fn intro_sequence_3_sprite_frame_vram(addroffset_sprite: u16) -> Option<IntroSpriteFrameVram> {
    // ASM Data_Index entries at kid.asm:48180 and `sprite_frame_vram` records
    // at unk_E1066..unk_E10C6 provide the frame metadata consumed by
    // Make_SpriteFromGfxObject for Load_IntroSequence3's loc_1AEEC objects.
    let (sprite_index, width, height) = match addroffset_sprite {
        0x1208 => (0x000, 0x30, 0x30),
        0x120C => (0x024, 0x0C, 0x31),
        0x1210 => (0x032, 0x30, 0x30),
        0x1214 => (0x056, 0x0C, 0x39),
        0x1218 => (0x066, 0x2A, 0x37),
        0x121C => (0x090, 0x27, 0x2B),
        0x1220 => (0x0AE, 0x3B, 0x1B),
        0x1228 => (0x000, 0x3E, 0x2F),
        0x122C => (0x030, 0x28, 0x30),
        0x1230 => (0x04E, 0x28, 0x30),
        0x1234 => (0x000, 0x34, 0x3D),
        0x1238 => (0x000, 0x34, 0x4C),
        0x123C => (0x000, 0x27, 0x37),
        0x1224 => (0x0CC, 0x30, 0x30),
        _ => return None,
    };
    Some(IntroSpriteFrameVram {
        sprite_index,
        x_center: 0,
        y_center: 0,
        width,
        height,
    })
}

#[allow(non_snake_case)]
fn overlay_intro_sequence_3_sprite_table_loc_1AEEC(
    vdp: &mut VdpState,
    roster_index: usize,
    visible_elapsed: usize,
) {
    let sprite_table_base = sprite_table_base_byte_addr(&vdp.registers);
    let mut sprite_entries = Vec::new();
    for descriptor_slot in 0..8 {
        let Some(descriptor) =
            crate::movement::intro_sequence_3_descriptor_loc_1AEBA(roster_index, descriptor_slot)
        else {
            break;
        };
        let gfx = crate::movement::intro_sequence_3_gfx_object_loc_1AEEC(descriptor);
        if !gfx.display_enabled {
            continue;
        }
        let Some(frame) = intro_sequence_3_sprite_frame_vram(gfx.addroffset_sprite) else {
            continue;
        };
        append_intro_sprite_entries_make_sprite_from_gfx_object_loc_129E(
            &mut sprite_entries,
            gfx,
            frame,
        );
    }
    append_intro_sequence_prompt_sprite_sub_1C54A(&mut sprite_entries);
    append_intro_sequence_3_sliding_sprites_sub_1C572(&mut sprite_entries, visible_elapsed);
    write_sprite_table_entries(vdp, sprite_table_base, &sprite_entries);
}

#[allow(non_snake_case)]
fn append_intro_sequence_3_sliding_sprites_sub_1C572(
    entries: &mut Vec<HardwareSpriteEntry>,
    visible_elapsed: usize,
) {
    // ASM Load_IntroSequence3 (kid.asm:29078-29088) allocates the Wild Side
    // kid roster and then the paired sub_1C572/loc_1C58C sliding door sprites.
    // Both door workers set their visible gfx slot immediately, hibernate for
    // `$3C`, then move one pixel per frame for `#$41+1` iterations.
    for sliding in crate::boot::title_menu_sliding_sprites_sub_1C572() {
        let moving_elapsed =
            visible_elapsed.saturating_sub(usize::from(sliding.initial_hibernate_frames));
        let moved_frames = moving_elapsed.min(usize::from(sliding.moving_frames));
        let x_pos = i32::from(sliding.x_pos)
            + i32::from(sliding.x_velocity_per_frame) * moved_frames as i32;
        let (sprite_index, width, height) = match sliding.sprite_label {
            "unk_E10A6" => (0x030, 0x28, 0x30),
            "unk_E10AE" => (0x04E, 0x28, 0x30),
            _ => continue,
        };
        append_intro_sprite_entries_make_sprite_from_gfx_object_loc_129E(
            entries,
            IntroGfxObjectForSpriteTable {
                x_pos: x_pos as u16,
                y_pos: sliding.y_pos,
                palette_line: 0,
                vram_tile: sliding.vram_tile,
                priority: u8::from(sliding.priority_flag),
                x_direction: 0,
            },
            IntroSpriteFrameVram {
                sprite_index,
                x_center: 0,
                y_center: 0,
                width,
                height,
            },
        );
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct IntroSequence5GfxObjectLoc1BCD0 {
    x_pos: u16,
    y_pos: u16,
    vram_tile: u16,
    frame: IntroSpriteFrameVram,
}

#[allow(non_snake_case)]
fn overlay_intro_sequence_5_sprite_table_loc_1BCD0(
    vdp: &mut VdpState,
    title_variant: u8,
    elapsed: usize,
) {
    // ASM loc_1BCD0 (kid.asm:30567): allocates five visible gfx slots with
    // palette line 2, VRAM tile `$26C`, and frame links unk_E1066/E106E/
    // unk_E1076/unk_E1076/unk_E107E before hibernating the controller.
    let objects = [
        IntroSequence5GfxObjectLoc1BCD0 {
            x_pos: 0x0030,
            y_pos: 0x0080,
            vram_tile: 0x026C,
            frame: intro_sprite_frame_vram(0x000, 0x30, 0x30),
        },
        IntroSequence5GfxObjectLoc1BCD0 {
            x_pos: 0x0060,
            y_pos: 0x0080,
            vram_tile: 0x026C,
            frame: intro_sprite_frame_vram(0x024, 0x0C, 0x31),
        },
        IntroSequence5GfxObjectLoc1BCD0 {
            x_pos: 0x0000,
            y_pos: 0x00E0,
            vram_tile: 0x026C,
            frame: intro_sprite_frame_vram(0x032, 0x30, 0x30),
        },
        IntroSequence5GfxObjectLoc1BCD0 {
            x_pos: 0x0030,
            y_pos: 0x00B0,
            vram_tile: 0x026C,
            frame: intro_sprite_frame_vram(0x032, 0x30, 0x30),
        },
        IntroSequence5GfxObjectLoc1BCD0 {
            x_pos: 0x0060,
            y_pos: 0x00B0,
            vram_tile: 0x026C,
            frame: intro_sprite_frame_vram(0x056, 0x0C, 0x39),
        },
    ];
    let mut sprite_entries = Vec::new();
    for object in objects {
        append_intro_sprite_entries_make_sprite_from_gfx_object_loc_129E(
            &mut sprite_entries,
            IntroGfxObjectForSpriteTable {
                x_pos: object.x_pos,
                y_pos: object.y_pos,
                palette_line: 2,
                vram_tile: object.vram_tile,
                priority: 0,
                x_direction: 0,
            },
            object.frame,
        );
    }
    if let Some(particle_elapsed) =
        elapsed.checked_sub(INTRO_SEQUENCE_5_PARTICLE_START_ELAPSED_LOC_1BCD0)
    {
        append_intro_sequence_5_particles_loc_1BE3C(&mut sprite_entries, particle_elapsed);
        append_intro_sequence_5_center_particle_loc_1BE88(
            &mut sprite_entries,
            title_variant,
            particle_elapsed,
        );
    }
    write_sprite_table_entries(
        vdp,
        sprite_table_base_byte_addr(&vdp.registers),
        &sprite_entries,
    );
}

const INTRO_SEQUENCE_5_PARTICLE_START_ELAPSED_LOC_1BCD0: usize = 0x01A0;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct IntroSequence5ParticleLoc1BE3C {
    x_pos: i32,
    y_pos: i32,
    x_vel: i32,
    y_vel: i32,
    loop_count: usize,
}

#[allow(non_snake_case)]
fn append_intro_sequence_5_particles_loc_1BE3C(
    entries: &mut Vec<HardwareSpriteEntry>,
    particle_elapsed: usize,
) {
    // ASM loc_1BE3C/loc_1BE74 (kid.asm:30667): after loc_1BCD0's `$1A0`
    // hibernate, three workers load `unk_1BE18/24/30`, use addroffset
    // `$11C8` (LnkTo_unk_E0F56-Data_Index), then add `$4000` to y_vel once
    // per frame until DBF deletes them.
    let particles = [
        IntroSequence5ParticleLoc1BE3C {
            x_pos: 0x0012_0000,
            y_pos: 0x0078_0000,
            x_vel: -0x0004_0000,
            y_vel: -0x0002_0000,
            loop_count: 0x28,
        },
        IntroSequence5ParticleLoc1BE3C {
            x_pos: 0x0058_0000,
            y_pos: 0x0068_0000,
            x_vel: 0x0004_0000,
            y_vel: -0x0002_0000,
            loop_count: 0x3C,
        },
        IntroSequence5ParticleLoc1BE3C {
            x_pos: 0x0030_0000,
            y_pos: 0x0080_0000,
            x_vel: 0,
            y_vel: 0x0004_0000,
            loop_count: 0x28,
        },
    ];
    for particle in particles {
        if particle_elapsed > particle.loop_count {
            continue;
        }
        let frames = particle_elapsed as i32;
        let y_gravity = 0x0000_4000 * frames.saturating_mul(frames + 1) / 2;
        append_intro_sprite_entries_make_sprite_from_gfx_object_loc_129E(
            entries,
            IntroGfxObjectForSpriteTable {
                x_pos: fixed_to_u16_px(particle.x_pos + particle.x_vel.saturating_mul(frames)),
                y_pos: fixed_to_u16_px(
                    particle.y_pos + particle.y_vel.saturating_mul(frames) + y_gravity,
                ),
                palette_line: 2,
                vram_tile: 0x026C,
                priority: 0,
                x_direction: 0,
            },
            IntroSpriteFrameVram {
                sprite_index: 0x028C,
                x_center: 0,
                y_center: 8,
                width: 8,
                height: 8,
            },
        );
    }
}

#[allow(non_snake_case)]
fn append_intro_sequence_5_center_particle_loc_1BE88(
    entries: &mut Vec<HardwareSpriteEntry>,
    title_variant: u8,
    particle_elapsed: usize,
) {
    // ASM loc_1BE88/loc_1BF0A (kid.asm:30689): the centre helmet particle uses
    // palette line 3, priority 1, a FC82-selected frame on VRAM tile `$5EB`,
    // and keeps hibernating forever after the 28-frame velocity loop.
    let (x_seed, y_seed, frame) = match title_variant {
        0 => (
            0x002A_0000,
            0x0074_0000,
            intro_sprite_frame_vram(0, 0x34, 0x3D),
        ),
        1 => (
            0x0030_0000,
            0x0070_0000,
            intro_sprite_frame_vram(0, 0x27, 0x37),
        ),
        _ => (
            0x002A_0000,
            0x0074_0000,
            intro_sprite_frame_vram(0, 0x34, 0x4C),
        ),
    };
    let moving_frames = particle_elapsed.min(0x1C) as i32;
    let y_gravity = 0x0000_9000 * moving_frames.saturating_mul(moving_frames + 1) / 2;
    append_intro_sprite_entries_make_sprite_from_gfx_object_loc_129E(
        entries,
        IntroGfxObjectForSpriteTable {
            x_pos: fixed_to_u16_px(x_seed + 0x0002_F000 * moving_frames),
            y_pos: fixed_to_u16_px(y_seed - 0x0008_F000 * moving_frames + y_gravity),
            palette_line: 3,
            vram_tile: 0x05EB,
            priority: 1,
            x_direction: 0,
        },
        frame,
    );
}

fn fixed_to_u16_px(value: i32) -> u16 {
    (value >> 16) as u16
}

fn intro_sprite_frame_vram(sprite_index: u16, width: u16, height: u16) -> IntroSpriteFrameVram {
    IntroSpriteFrameVram {
        sprite_index,
        x_center: 0,
        y_center: 0,
        width,
        height,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct HardwareSpriteEntry {
    y_pos: u16,
    size_byte: u8,
    pattern: u16,
    x_pos: u16,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct IntroGfxObjectForSpriteTable {
    x_pos: u16,
    y_pos: u16,
    palette_line: u8,
    vram_tile: u16,
    priority: u8,
    x_direction: u8,
}

impl From<crate::movement::IntroSequence3GfxObjectLoc1AEEC> for IntroGfxObjectForSpriteTable {
    fn from(gfx: crate::movement::IntroSequence3GfxObjectLoc1AEEC) -> Self {
        Self {
            x_pos: gfx.x_pos,
            y_pos: gfx.y_pos,
            palette_line: gfx.palette_line,
            vram_tile: gfx.vram_tile,
            priority: gfx.priority,
            x_direction: gfx.x_direction,
        }
    }
}

#[allow(non_snake_case)]
fn append_intro_sprite_entries_make_sprite_from_gfx_object_loc_129E(
    entries: &mut Vec<HardwareSpriteEntry>,
    gfx: impl Into<IntroGfxObjectForSpriteTable>,
    frame: IntroSpriteFrameVram,
) {
    let gfx = gfx.into();
    let pattern_base = ((u16::from(gfx.palette_line) << 13)
        | if gfx.priority >= 1 { 0x8000 } else { 0 }
        | if gfx.x_direction != 0 { 0x0800 } else { 0 }
        | gfx.vram_tile)
        .wrapping_add(frame.sprite_index);
    let mut tile_offset = 0u16;
    for chunk in sprite_chunks(frame.width, frame.height) {
        let x_pos = if gfx.x_direction == 0 {
            i32::from(gfx.x_pos) - i32::from(frame.x_center) + i32::from(chunk.x) + 0x80
        } else {
            i32::from(gfx.x_pos) + i32::from(frame.x_center) - i32::from(chunk.x + chunk.width)
                + 0x80
        };
        let y_pos = i32::from(gfx.y_pos) - i32::from(frame.height)
            + i32::from(frame.y_center)
            + i32::from(chunk.y)
            + 0x80;
        entries.push(HardwareSpriteEntry {
            y_pos: y_pos as u16,
            size_byte: sprite_size_byte(chunk.width, chunk.height),
            pattern: pattern_base.wrapping_add(tile_offset),
            x_pos: x_pos as u16,
        });
        tile_offset = tile_offset.wrapping_add(chunk.width.div_ceil(8) * chunk.height.div_ceil(8));
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct SpriteChunk {
    x: u16,
    y: u16,
    width: u16,
    height: u16,
}

fn sprite_chunks(width: u16, height: u16) -> Vec<SpriteChunk> {
    let mut chunks = Vec::new();
    let mut y = 0;
    while y < height {
        let chunk_height = (height - y).min(32);
        let mut x = 0;
        while x < width {
            let chunk_width = (width - x).min(32);
            chunks.push(SpriteChunk {
                x,
                y,
                width: chunk_width,
                height: chunk_height,
            });
            x += chunk_width;
        }
        y += chunk_height;
    }
    chunks
}

fn sprite_size_byte(width: u16, height: u16) -> u8 {
    let width_tiles = width.div_ceil(8).saturating_sub(1);
    let height_tiles = height.div_ceil(8).saturating_sub(1);
    ((width_tiles << 2) | height_tiles) as u8
}

fn write_sprite_table_entries(
    vdp: &mut VdpState,
    sprite_table_base: usize,
    entries: &[HardwareSpriteEntry],
) {
    for (index, entry) in entries.iter().enumerate().take(80) {
        let entry_base = sprite_table_base + index * 8;
        let link = if index + 1 < entries.len().min(80) {
            (index + 1) as u8
        } else {
            0
        };
        vdp.write_vram_word(entry_base, entry.y_pos);
        vdp.vram[entry_base + 2] = entry.size_byte;
        vdp.vram[entry_base + 3] = link;
        vdp.write_vram_word(entry_base + 4, entry.pattern);
        vdp.write_vram_word(entry_base + 6, entry.x_pos);
    }
}

fn sprite_table_base_byte_addr(registers: &[u8; 0x20]) -> usize {
    // ASM VDPGameInitValues reg 5 = `$08`, selecting sprite table VRAM
    // byte address `$1000` in H40 mode. This mirrors kid-md's renderer helper.
    let mask = if registers[0x0C] & 0x81 != 0 {
        0x7E
    } else {
        0x7F
    };
    usize::from(registers[5] & mask) << 9
}

fn build_intro_sequence_vdp_base_with_title_variant(
    data: &IntroSub1B850Data,
    segment: usize,
    title_variant: u8,
) -> VdpState {
    match segment {
        1 => build_intro_sequence_2_vdp(data),
        3 => build_intro_sequence_4_vdp(data),
        4 => build_intro_sequence_5_vdp(data, title_variant),
        _ => build_intro_sub_1b850_vdp(data, segment),
    }
}

struct IntroTextScriptLoc1C7A0 {
    base_vram_byte_address: u16,
    base_tile_word: u16,
    delay_frames: u16,
    x_pos: u16,
    y_pos: u16,
    text_bytes: &'static [u8],
}

const INTRO_TEXT_1_BYTES: &[u8] = b"\xFE\x20THERE WAS A NEW MACHINE\xFDIN THE ARCADE\xFD\xFE\x20THAT ONE COULD WALK INTO\xFDAND PLAY\x66\xFF";
const INTRO_TEXT_2_BYTES: &[u8] =
    b"\xFE\x90IT USED HOLOGRAMS TO CREATE\xFDA REALITY NOT OUR OWN\x66\xFF";
const INTRO_TEXT_3_BYTES: &[u8] = b"\xFE\x20EVERYBODY PLAYED IT\x66\xFF";
const INTRO_TEXT_4_BYTES: &[u8] = b"\xFE\x20BUT IT WAS A LITTLE TOO REAL\x66\xFD\xFD\xFE\x40THE BOSS ESCAPED\x68 AND BEGAN\xFDCAPTURING KIDS BY DEFEATING\xFDTHEM AT THE GAME\x66\xFF";
const INTRO_TEXT_5_BYTES: &[u8] = b"\xFE\x20THIS IS THE\xFDSTORY OF\xFDSOMEONE TOO\xFDTOUGH TO BEAT\x66\xFD\xFD\xFE\x60SOMEONE KNOWN\xFDAS\x66\x66\x66\xFF";

const INTRO_TEXT_CLEAR_TILE_WORD_LOC_1C7A0: u16 = 0x67C0;

const INTRO_TEXT_WORKER_START_ELAPSED_LOC_1C7A0: [usize; 5] = [
    67, // ASM sub_1B8F4/loc_1C7A0; frame-360 oracle text writes lag the object-count exposure by 22 frames.
    42, // ASM loc_1B9D0; timeout-return traces expose controller+IntroText2 at elapsed 42.
    42, // ASM sub_1B93E follows the same Load_IntroSequence3 hidden rebuild shell.
    42, // ASM sub_1B988 allocates IntroText4 before sub_1C204's fade shell.
    42, // ASM loc_1BA28 allocates IntroText5 before sub_1C204's fade shell.
];

fn intro_text_script_loc_1c7a0(segment: usize) -> Option<IntroTextScriptLoc1C7A0> {
    // ASM IntroText1-5 (kid.asm:31380-31396): headers are
    // `(VRAM destination byte address, VRAM base tile, delay, x, y)`.
    match segment {
        0 => Some(IntroTextScriptLoc1C7A0 {
            base_vram_byte_address: 0x0000,
            base_tile_word: 0x67D3,
            delay_frames: 2,
            x_pos: 8,
            y_pos: 2,
            text_bytes: INTRO_TEXT_1_BYTES,
        }),
        1 => Some(IntroTextScriptLoc1C7A0 {
            base_vram_byte_address: 0xE000,
            base_tile_word: 0xC7D3,
            delay_frames: 2,
            x_pos: 7,
            y_pos: 3,
            text_bytes: INTRO_TEXT_2_BYTES,
        }),
        2 => Some(IntroTextScriptLoc1C7A0 {
            base_vram_byte_address: 0x0000,
            base_tile_word: 0x67D3,
            delay_frames: 2,
            x_pos: 0x0A,
            y_pos: 4,
            text_bytes: INTRO_TEXT_3_BYTES,
        }),
        3 => Some(IntroTextScriptLoc1C7A0 {
            base_vram_byte_address: 0x0000,
            base_tile_word: 0xC7D3,
            delay_frames: 2,
            x_pos: 6,
            y_pos: 8,
            text_bytes: INTRO_TEXT_4_BYTES,
        }),
        4 => Some(IntroTextScriptLoc1C7A0 {
            base_vram_byte_address: 0x0000,
            base_tile_word: 0xC7D3,
            delay_frames: 2,
            x_pos: 0x19,
            y_pos: 3,
            text_bytes: INTRO_TEXT_5_BYTES,
        }),
        _ => None,
    }
}

fn overlay_intro_text_worker_loc_1c7a0(vdp: &mut VdpState, segment: usize, elapsed: usize) {
    let Some(script) = intro_text_script_loc_1c7a0(segment) else {
        return;
    };
    clear_intro_text_script_cells_loc_1c7a0(vdp, &script);
    let Some(start_elapsed) = INTRO_TEXT_WORKER_START_ELAPSED_LOC_1C7A0
        .get(segment)
        .copied()
    else {
        return;
    };
    let Some(worker_elapsed) = elapsed.checked_sub(start_elapsed) else {
        return;
    };
    replay_intro_text_worker_loc_1c7a0(vdp, &script, worker_elapsed);
}

fn clear_intro_text_script_cells_loc_1c7a0(vdp: &mut VdpState, script: &IntroTextScriptLoc1C7A0) {
    // Native asset extraction can leave the final text tilemap in the intro
    // scene maps. ASM loc_1C7A0/sub_1C7F6 writes each glyph over blank plane
    // cells at runtime, so clear the scripted cells before replaying the
    // visible prefix for this frame.
    let mut byte_index = 0usize;
    let mut x_pos = script.x_pos;
    let mut y_pos = script.y_pos;
    while let Some(&raw_byte) = script.text_bytes.get(byte_index) {
        byte_index += 1;
        match raw_byte {
            0xFF => return,
            0xFD => {
                x_pos = script.x_pos;
                y_pos = y_pos.wrapping_add(2);
            }
            0xFE => {
                byte_index += 1;
            }
            _ => {
                let write = kid_data::title_intro_glyph_write_sub_1C7F6(
                    script.base_vram_byte_address,
                    script.base_tile_word,
                    x_pos,
                    y_pos,
                    0,
                );
                // Native-only cleanup for extracted maps must not use tile
                // word 0: Scroll A's nametable also lives at VRAM $0000, so
                // tile index 0 samples nametable bytes as pattern data and
                // produces the orange bars seen before ASM loc_1C7A0 writes
                // real glyphs via sub_1C7F6.
                vdp.write_vram_word(
                    usize::from(write.vram_word_address),
                    INTRO_TEXT_CLEAR_TILE_WORD_LOC_1C7A0,
                );
                x_pos = x_pos.wrapping_add(1);
            }
        }
    }
}

fn replay_intro_text_worker_loc_1c7a0(
    vdp: &mut VdpState,
    script: &IntroTextScriptLoc1C7A0,
    worker_elapsed: usize,
) {
    // ASM loc_1C7A0 -> loc_1C7AE (kid.asm:31404-31414): the text worker
    // hibernates once before parsing its stream. `$FE` delays and per-glyph
    // header delays also return to this hibernate loop before the next byte.
    if worker_elapsed < 1 {
        return;
    }
    let mut remaining = worker_elapsed - 1;
    let mut byte_index = 0usize;
    let mut x_pos = script.x_pos;
    let mut y_pos = script.y_pos;
    while let Some(&raw_byte) = script.text_bytes.get(byte_index) {
        byte_index += 1;
        match raw_byte {
            0xFF => return,
            0xFD => {
                // ASM loc_1C7C4: reload X from the header and advance Y by two rows.
                x_pos = script.x_pos;
                y_pos = y_pos.wrapping_add(2);
            }
            0xFE => {
                // ASM loc_1C7D2: following byte is the hibernate duration.
                let Some(&duration) = script.text_bytes.get(byte_index) else {
                    return;
                };
                byte_index += 1;
                let duration = usize::from(duration);
                if remaining < duration {
                    return;
                }
                remaining -= duration;
            }
            byte => {
                let substitution = intro_text_buggy_substitution_loc_1C7DC(byte, 0);
                let glyph_index = u16::from(substitution.output_byte.wrapping_sub(0x41));
                let write = kid_data::title_intro_glyph_write_sub_1C7F6(
                    script.base_vram_byte_address,
                    script.base_tile_word,
                    x_pos,
                    y_pos,
                    glyph_index,
                );
                vdp.write_vram_word(usize::from(write.vram_word_address), write.tile_word);
                x_pos = x_pos.wrapping_add(1);
                if script.delay_frames != 0 {
                    let delay_frames = usize::from(script.delay_frames);
                    if remaining < delay_frames {
                        return;
                    }
                    remaining -= delay_frames;
                }
            }
        }
    }
}

fn intro_sequence_base_vdp() -> VdpState {
    let mut vdp = VdpState::new();
    // ASM VDPGameInitValues → register init, then reg 1 = $74 (display on).
    for (i, &value) in VDP_GAME_INIT_VALUES.iter().enumerate() {
        vdp.registers[i] = value;
    }
    vdp.registers[1] = 0x74;
    vdp
}

fn build_intro_sub_1b850_vdp(data: &IntroSub1B850Data, segment: usize) -> VdpState {
    let mut vdp = intro_sequence_base_vdp();

    // Upload decompressed art to VRAM at specified addresses.
    for (vram_byte_addr, art_bytes) in &data.art_chunks {
        vdp.write_vram(*vram_byte_addr, art_bytes);
    }

    // Write plane A tilemap: 40 words per row, 28 rows, nametable at VRAM $0000.
    // Row stride = 128 bytes (64 columns × 2 bytes). Only columns 0–39 filled.
    let plane_a_base: usize = 0x0000; // Register 2 = $00 → plane A at $0000
    for row in 0..28usize {
        for col in 0..40usize {
            let word_index = row * 40 + col;
            if word_index < data.plane_a_tilemap.len() {
                let byte_addr = plane_a_base + row * NAMETABLE_ROW_STRIDE_BYTES + col * 2;
                vdp.write_vram_word(byte_addr, data.plane_a_tilemap[word_index]);
            }
        }
    }

    // Write plane B repeated rows at VRAM $E000 (register 4 = $07 → $E000).
    let plane_b_base: usize = 0xE000;
    for (row, words) in data.plane_b_rows.iter().enumerate() {
        for (col, &word) in words.iter().enumerate() {
            let byte_addr = plane_b_base + row * NAMETABLE_ROW_STRIDE_BYTES + col * 2;
            vdp.write_vram_word(byte_addr, word);
        }
    }

    // Load segment-specific palette to CRAM.
    // ASM: segment 0 → Pal_1C28A (0x33 words), segments 2+ → Pal_1C346.
    // Segment 1 (Load_IntroSequence2) and segments 3/4 (Load_IntroSequence4/5)
    // route through their dedicated build_intro_sequence_{2,4,5}_vdp helpers
    // before this fn runs, so this match only fires for segments 0, 2, and any
    // overflow values — there is no live `1 =>` arm to test here.
    let palette = match segment {
        // ASM loc_1C4F6 (kid.asm:31232): `move.w (a0)+,(a1)+; dbf`
        // copies 0x33 words from Pal_1C28A into Palette_Buffer_2.
        0 => &data.palette_segment_0,
        // ASM loc_1C51E (kid.asm:31262): `move.w (a0)+,(a1)+; dbf`
        // copies 0x40 words from Pal_1C346 into Palette_Buffer_2.
        // ASM loc_1C540 (kid.asm:31273): non-zero region variants seed
        // `d0=#$D` for a 14-word overlay after this base palette copy; this
        // native loader currently follows the default FC82==0 path.
        // ASM loc_1C542 (kid.asm:31276): the overlay itself is the inclusive
        // `move.w (a0)+,(a1)+; dbf` loop into Palette_Buffer_2+$62.
        _ => &data.palette_segment_2,
    };
    vdp.write_cram_words(0, palette);

    vdp
}

fn write_full_plane_rows(vdp: &mut VdpState, plane_base: usize, rows: &[[u16; 64]; 28]) {
    for (row, words) in rows.iter().enumerate() {
        for (col, &word) in words.iter().enumerate() {
            let byte_addr = plane_base + row * NAMETABLE_ROW_STRIDE_BYTES + col * 2;
            vdp.write_vram_word(byte_addr, word);
        }
    }
}

fn write_plane_a_40x28(vdp: &mut VdpState, words: &[u16]) {
    for row in 0..28usize {
        for col in 0..40usize {
            let word_index = row * 40 + col;
            if let Some(&word) = words.get(word_index) {
                let byte_addr = row * NAMETABLE_ROW_STRIDE_BYTES + col * 2;
                vdp.write_vram_word(byte_addr, word);
            }
        }
    }
}

fn sub_1cd88_rows(decoded_words: &[u16]) -> Option<[[u16; 64]; 28]> {
    kid_data::decompression_buffer_rows_upload_sub_1CD88(decoded_words)
        .ok()
        .map(|upload| upload.rows)
}

fn intro_sequence_2_hologram_rows(data: &IntroSub1B850Data) -> Option<[[u16; 64]; 28]> {
    let mut decoded_words = data.intro_sequence_2_hologram_tilemap.clone();
    apply_title_text_word_writes_sub_1b7b6(&mut decoded_words);
    sub_1cd88_rows(&decoded_words)
}

fn apply_title_text_word_writes_sub_1b7b6(decoded_words: &mut [u16]) {
    // ASM sub_1B7B6 (kid.asm:30035-30053): after Load_IntroSequence2 decodes
    // MapEni_CC0E into Decompression_Buffer, it patches the listed absolute
    // RAM addresses with the fixed `$21AB..$21AE` 2x2 blocks before sub_1CD88.
    const DECOMPRESSION_BUFFER_LOW_ADDR: u16 = 0x77B2;
    for write in title_text_word_writes_sub_1B7B6() {
        let byte_offset = usize::from(
            write
                .base_word_address
                .wrapping_sub(DECOMPRESSION_BUFFER_LOW_ADDR),
        );
        let word_index = byte_offset / 2;
        if word_index + 1 < decoded_words.len() {
            decoded_words[word_index] = write.top_words[0];
            decoded_words[word_index + 1] = write.top_words[1];
        }
        let bottom_word_index = word_index + 0x28;
        if bottom_word_index + 1 < decoded_words.len() {
            decoded_words[bottom_word_index] = write.bottom_words[0];
            decoded_words[bottom_word_index + 1] = write.bottom_words[1];
        }
    }
}

fn build_intro_sequence_2_vdp(data: &IntroSub1B850Data) -> VdpState {
    let mut vdp = intro_sequence_base_vdp();

    // ASM Load_IntroSequence2 (kid.asm:29255-29300): load HoloBG art, the
    // paired HoloBlocks art, patch the hologram tilemap via sub_1B7B6, then
    // write the intro2_sky plane-A tilemap and copy the sub_1C4FE palette.
    vdp.write_vram(0x2280, &data.hologram_background_art);
    vdp.write_vram(0x3540, &data.hologram_blocks_art);
    if let Some(rows) = intro_sequence_2_hologram_rows(data) {
        write_full_plane_rows(&mut vdp, 0xE000, &rows);
    }
    vdp.write_vram(0xFA60, &data.intro_text_letters_art);
    write_plane_a_40x28(&mut vdp, &data.intro_sequence_2_tilemap);
    // ASM loc_1C50A (kid.asm:31247): `move.w (a0)+,(a1)+; dbf` inner copy
    // loop of sub_1C4FE seeds 0x23 words from Pal_1C2F0 into Palette_Buffer_2.
    vdp.write_cram_words(0, &data.palette_segment_1);

    vdp
}

fn build_intro_sequence_4_vdp(data: &IntroSub1B850Data) -> VdpState {
    let mut vdp = intro_sequence_base_vdp();

    // ASM Load_IntroSequence4 (kid.asm:29303-29321): only the HoloBG art is
    // loaded, MapEni_CC0E is decoded with base `$20BC`, sub_1CD88 writes it
    // to plane B, and sub_1C4FE supplies the intro2_sky palette again.
    vdp.write_vram(0x1780, &data.hologram_background_art);
    if let Some(rows) = sub_1cd88_rows(&data.intro_sequence_4_hologram_tilemap) {
        write_full_plane_rows(&mut vdp, 0xE000, &rows);
    }
    vdp.write_vram(0xFA60, &data.intro_text_letters_art);
    // ASM loc_1C50A (kid.asm:31247): same `move.w (a0)+,(a1)+; dbf` inner
    // copy loop as build_intro_sequence_2_vdp — sub_1C4FE seeds 0x23 words
    // from Pal_1C2F0 into Palette_Buffer_2.
    vdp.write_cram_words(0, &data.palette_segment_1);

    vdp
}

fn build_intro_sequence_5_vdp(data: &IntroSub1B850Data, title_variant: u8) -> VdpState {
    let mut vdp = intro_sequence_base_vdp();

    // ASM Load_IntroSequence5 (kid.asm:29324-29364): use the Dark Alley/P
    // block/Kid/helmet art set and MapEni_2E154, not the shared sub_1B850
    // Wild Side art path.
    for (vram_byte_addr, art_bytes) in &data.intro_sequence_5_common_art_chunks {
        vdp.write_vram(*vram_byte_addr, art_bytes);
    }
    // ASM loc_1B100 (kid.asm:29335-29351): `$FFFFFC82` selects byte_213D9,
    // byte_219C1, or byte_216B7 before the decompression target `$BD60`.
    let helmet_variant_index = match title_variant {
        0 => 0,
        1 => 1,
        _ => 2,
    };
    vdp.write_vram(
        0xBD60,
        &data.intro_sequence_5_helmet_art_variants[helmet_variant_index],
    );
    if let Some(rows) = sub_1cd88_rows(&data.intro_sequence_5_tilemap) {
        write_full_plane_rows(&mut vdp, 0xE000, &rows);
    }
    vdp.write_cram_words(0, &data.palette_segment_2);
    // ASM sub_1C512 (kid.asm:31257-31278): non-zero `$FFFFFC82` overlays
    // 14 words at Palette_Buffer_2+$62, using Pal_1C3C6 for 1 and Pal_1C3D4
    // for every other non-zero variant.
    match title_variant {
        0 => {}
        1 => vdp.write_cram_words(0x31, &data.intro_sequence_5_palette_variant_1_overlay),
        _ => vdp.write_cram_words(0x31, &data.intro_sequence_5_palette_variant_2_overlay),
    }

    vdp
}

const INTRO_SEQUENCE_5_LOC_1BC36_FRAME_TIMELINE: &[(usize, u16)] = &[
    (0, 0),
    (0xA0, 1),
    (0xA0 + 0x90, 2),
    (0xA0 + 0x90 + 0x0B, 3),
    (0xA0 + 0x90 + 0x0B + 0x0B, 4),
    (0xA0 + 0x90 + 0x0B + 0x0B + 0x0B, 5),
    (0xA0 + 0x90 + 0x0B + 0x0B + 0x0B + 0x37, 6),
    (0xA0 + 0x90 + 0x0B + 0x0B + 0x0B + 0x37 + 0x08, 7),
    (0xA0 + 0x90 + 0x0B + 0x0B + 0x0B + 0x37 + 0x08 + 0x08, 6),
    (
        0xA0 + 0x90 + 0x0B + 0x0B + 0x0B + 0x37 + 0x08 + 0x08 + 0x0F,
        5,
    ),
];
const INTRO_SEQUENCE_5_LOC_1BC36_PANEL_VISIBLE_ELAPSED: usize = 0xA0 + 0x90 + 0x0B + 0x0B;
const INTRO_SEQUENCE_5_LOC_1BC36_PANEL_CLEAR_ELAPSED: usize =
    INTRO_SEQUENCE_5_LOC_1BC36_PANEL_VISIBLE_ELAPSED + 0x0B;

#[allow(non_snake_case)]
fn overlay_intro_sequence_5_tile_worker_loc_1BC36(
    vdp: &mut VdpState,
    data: &IntroSub1B850Data,
    elapsed: usize,
) {
    // ASM loc_1BC36 (kid.asm:30516) calls sub_1C034 before each hibernate.
    // sub_1C034 decodes one `off_1C126` map into a 16x25 staging buffer, then
    // streams 25 fixed rows to VRAM `$A340..$A940`.
    if let Some(frame_index) = INTRO_SEQUENCE_5_LOC_1BC36_FRAME_TIMELINE
        .iter()
        .rev()
        .find_map(|(start_elapsed, frame_index)| {
            (elapsed >= *start_elapsed).then_some(*frame_index)
        })
    {
        apply_intro_sequence_5_sub_1C034_frame(vdp, data, frame_index);
    }

    if elapsed >= INTRO_SEQUENCE_5_LOC_1BC36_PANEL_CLEAR_ELAPSED {
        apply_intro_sequence_5_blank_panel_sub_1C112(vdp, 3, 0x13);
    } else if elapsed >= INTRO_SEQUENCE_5_LOC_1BC36_PANEL_VISIBLE_ELAPSED {
        apply_intro_sequence_5_panel_sub_1C0EE(vdp, data, 3, 0x13);
    }
}

#[allow(non_snake_case)]
fn apply_intro_sequence_5_sub_1C034_frame(
    vdp: &mut VdpState,
    data: &IntroSub1B850Data,
    frame_index: u16,
) {
    let Ok(plan) = kid_data::intro_sequence_5_frame_load_plan_sub_1C034(frame_index) else {
        return;
    };
    let Some(decoded_words) = data
        .intro_sequence_5_frame_tilemaps
        .get(usize::from(frame_index))
    else {
        return;
    };
    let mut staged_words = [0u16; 25 * 16];
    let width = usize::from(plan.tilemap_load.width_words);
    let height = usize::from(plan.tilemap_load.height_words);
    let left_padding = usize::from(plan.tilemap_load.left_padding_words);
    for row in 0..height.min(25) {
        for col in 0..width.min(16usize.saturating_sub(left_padding)) {
            if let Some(&word) = decoded_words.get(row * width + col) {
                staged_words[row * 16 + left_padding + col] = word;
            }
        }
    }

    for row_write in plan.vram_rows {
        let source_offset = usize::from(row_write.source_word_offset);
        for word_offset in 0..usize::from(row_write.word_count) {
            let Some(&word) = staged_words.get(source_offset + word_offset) else {
                continue;
            };
            let byte_addr = usize::from(row_write.vram_word_address) + word_offset * 2;
            vdp.write_vram_word(byte_addr, word);
        }
    }
}

#[allow(non_snake_case)]
fn apply_intro_sequence_5_panel_sub_1C0EE(
    vdp: &mut VdpState,
    data: &IntroSub1B850Data,
    plane_cell_x: u16,
    plane_cell_y: u16,
) {
    let plan = kid_data::intro_sequence_5_panel_load_plan_sub_1C0EE(plane_cell_x, plane_cell_y);
    write_plane_rect_words(
        vdp,
        &data.intro_sequence_5_panel_tilemap,
        usize::from(plan.plane_upload.plane_cell_x),
        usize::from(plan.plane_upload.plane_cell_y),
        usize::from(plan.plane_upload.width_words),
        usize::from(plan.plane_upload.height_words),
    );
}

#[allow(non_snake_case)]
fn apply_intro_sequence_5_blank_panel_sub_1C112(
    vdp: &mut VdpState,
    plane_cell_x: u16,
    plane_cell_y: u16,
) {
    let plan =
        kid_data::intro_sequence_5_blank_panel_load_plan_sub_1C112(plane_cell_x, plane_cell_y);
    let blank_words = vec![
        0u16;
        usize::from(plan.plane_upload.width_words)
            * usize::from(plan.plane_upload.height_words)
    ];
    write_plane_rect_words(
        vdp,
        &blank_words,
        usize::from(plan.plane_upload.plane_cell_x),
        usize::from(plan.plane_upload.plane_cell_y),
        usize::from(plan.plane_upload.width_words),
        usize::from(plan.plane_upload.height_words),
    );
}

fn write_plane_rect_words(
    vdp: &mut VdpState,
    words: &[u16],
    plane_cell_x: usize,
    plane_cell_y: usize,
    width_words: usize,
    height_words: usize,
) {
    for row in 0..height_words {
        for col in 0..width_words {
            let Some(&word) = words.get(row * width_words + col) else {
                continue;
            };
            let byte_addr =
                (plane_cell_y + row) * NAMETABLE_ROW_STRIDE_BYTES + (plane_cell_x + col) * 2;
            vdp.write_vram_word(byte_addr, word);
        }
    }
}

// ─── Title screen ───────────────────────────────────────────────────────────

fn asset_label_mapeni_filename(label: &str) -> String {
    label
        .strip_prefix("unk_")
        .map_or_else(|| format!("{label}.bin"), |hex| format!("{hex}.bin"))
}

/// Cached decompressed assets for the title screen (`Load_TitleScreen`).
#[derive(Debug, Clone)]
pub(crate) struct TitleScreenData {
    /// 4 decompressed Saxman art chunks in upload order.
    art_chunks: [(usize, Vec<u8>); 4],
    /// 3 decompressed Saxman art chunks from ASM `Load_TitleArt`.
    title_art_chunks: [(usize, Vec<u8>); 3],
    /// Decoded Enigma tilemap for plane A (written starting at word $20C1).
    plane_a_tilemap: Vec<u16>,
    /// Plane-B title text tilemap rows from ASM `sub_1B532`.
    title_text_chameleon_rows: [[u16; 64]; 28],
    /// Decoded interactive menu tilemap from ASM `Obj_TitleMenu loc_1BB52`.
    title_menu_tilemap_words: Vec<u16>,
    /// Decoded title character frames from ASM `sub_1BF24`.
    character_frames: Vec<TitleScreenCharacterFrameData>,
    /// 32-word tile patch for VRAM $F000.
    tile_patch_words: [u16; 32],
    /// Palette words from Pal_1C400 (title.bin — 0x33 words).
    palette_words: Vec<u16>,
}

#[derive(Debug, Clone)]
struct TitleScreenCharacterFrameData {
    frame_index: u8,
    region_byte: u8,
    width_words: u16,
    tilemap_words: Vec<u16>,
}

#[derive(Debug, Clone)]
struct TitleCardThemeData {
    art_bytes: Vec<u8>,
    tilemap_words: Vec<u16>,
    palette_words: Vec<u16>,
    width_tiles: u16,
    height_tiles: u16,
}

/// Cached title-card art, map, palette, and dimensions by low theme nibble.
#[derive(Debug, Clone)]
pub(crate) struct TitleCardData {
    themes: Vec<Option<TitleCardThemeData>>,
}

const TITLE_CARD_THEME_NAMES: [Option<&str>; 11] = [
    None,
    Some("sky"),
    Some("ice"),
    Some("hill"),
    Some("island"),
    Some("desert"),
    Some("swamp"),
    Some("mountain"),
    Some("cave"),
    Some("forest"),
    Some("city"),
];

/// ASM TitleCardSize_Index (kid.asm:28722): width, height in tiles by theme.
const TITLE_CARD_THEME_SIZES: [(u16, u16); 11] = [
    (0, 0),
    (0x0E, 0x12),
    (0x11, 0x12),
    (0x12, 0x11),
    (0x10, 0x12),
    (0x13, 0x0F),
    (0x10, 0x0D),
    (0x11, 0x11),
    (0x12, 0x0C),
    (0x0F, 0x17),
    (0x13, 0x11),
];

/// Load and cache title screen assets from the asset bundle.
///
/// ASM Load_TitleScreen (kid.asm:29444): clear VRAM, run sub_1B2A4, Load_TitleArt,
/// sub_1BF24, and sub_1B532 before spawning menu/title text objects.
pub(crate) fn load_title_screen_data(manifest: &AssetManifest) -> Result<TitleScreenData> {
    let scenes = manifest.output_root.join("scenes");

    // Art uploads — label→filename from kid.asm.
    let art_files: [(u16, &str); 4] = [
        (0x1820, "artcomp/The_kid_transformation_from_intro.bin"),
        (0x21E0, "artcomp/Red_Stealth_transformation_from_intro.bin"),
        (0x6620, "artcomp/Background_on_title.bin"),
        (0xFA60, "artcomp/Intro_text_letters.bin"),
    ];
    let mut art_chunks = Vec::new();
    for (vram_addr, filename) in &art_files {
        let path = scenes.join(filename);
        let compressed = std::fs::read(&path)
            .with_context(|| format!("failed to read title art: {}", path.display()))?;
        let decompressed = kid_data::decode_saxman(&compressed)
            .with_context(|| format!("failed to decompress title art: {filename}"))?;
        // VDP addresses are byte addresses (see intro loader comment).
        art_chunks.push((*vram_addr as usize, decompressed));
    }
    let art_chunks: [(usize, Vec<u8>); 4] = art_chunks
        .try_into()
        .map_err(|_| anyhow::anyhow!("expected 4 art chunks"))?;

    // ASM Load_TitleArt (kid.asm:29810): byte_22080 is decompressed to RAM for
    // title text animation, while these three chunks are decompressed directly
    // to VRAM at $9A60, $A360, and $A980.
    let title_art_files: [(u16, &str); 3] = [
        (0x9A60, "artcomp/Title_text_KID.bin"),
        (0xA360, "artcomp/Title_menu.bin"),
        (0xA980, "artcomp/Title_sparkles.bin"),
    ];
    let mut title_art_chunks = Vec::new();
    for (vram_addr, filename) in &title_art_files {
        let path = scenes.join(filename);
        let compressed = std::fs::read(&path)
            .with_context(|| format!("failed to read title art: {}", path.display()))?;
        let decompressed = kid_data::decode_saxman(&compressed)
            .with_context(|| format!("failed to decompress title art: {filename}"))?;
        title_art_chunks.push((*vram_addr as usize, decompressed));
    }
    let title_art_chunks: [(usize, Vec<u8>); 3] = title_art_chunks
        .try_into()
        .map_err(|_| anyhow::anyhow!("expected 3 title art chunks"))?;

    // Decode Enigma tilemap: unk_2F7D0 with base tile $20C1.
    // ASM sub_1B2A4: `move.w #$20C1,d0` then `jsr (j_EniDec)`.
    // After decode, the caller (Load_IntroSequence6 / Load_TitleScreen) calls
    // sub_1C5D0 with d4=$15, d5=1, d6=$12, d7=$14 → 18×20 at cell (21,1).
    let tilemap_path = scenes.join("mapeni/2F7D0.bin");
    let tilemap_raw = std::fs::read(&tilemap_path)
        .with_context(|| format!("failed to read title tilemap: {}", tilemap_path.display()))?;
    let plane_a_tilemap =
        kid_data::decode_enigma(&tilemap_raw, 0x20C1).context("failed to decode title tilemap")?;
    let chameleon_tilemap_path = scenes.join("mapeni/2F840.bin");
    let chameleon_tilemap_raw = std::fs::read(&chameleon_tilemap_path).with_context(|| {
        format!(
            "failed to read title chameleon tilemap: {}",
            chameleon_tilemap_path.display()
        )
    })?;
    let chameleon_decoded_words = kid_data::decode_enigma(&chameleon_tilemap_raw, 0x0331)
        .context("failed to decode title chameleon tilemap")?;
    let title_text_chameleon_upload =
        kid_data::title_text_chameleon_tilemap_upload_sub_1B532(&chameleon_decoded_words)
            .context("failed to build title chameleon tilemap upload")?;
    let title_menu_tilemap_path = scenes.join("mapeni/2EAAE.bin");
    let title_menu_tilemap_raw = std::fs::read(&title_menu_tilemap_path).with_context(|| {
        format!(
            "failed to read title menu tilemap: {}",
            title_menu_tilemap_path.display()
        )
    })?;
    let title_menu_tilemap_words = kid_data::decode_enigma(&title_menu_tilemap_raw, 0x451B)
        .context("failed to decode title menu tilemap")?;
    let mut character_frames = Vec::new();
    for region_byte in 0..=2u8 {
        for frame_index in 0..=7u8 {
            let plan = kid_data::title_screen_character_load_plan_sub_1BF24(
                frame_index.into(),
                region_byte,
            )
            .context("failed to build title character load plan")?;
            let filename = asset_label_mapeni_filename(plan.tilemap_load.asset_label);
            let raw = std::fs::read(scenes.join("mapeni").join(&filename))
                .with_context(|| format!("failed to read title character tilemap: {filename}"))?;
            let tilemap_words = kid_data::decode_enigma(&raw, plan.tilemap_load.vram_word_address)
                .with_context(|| format!("failed to decode title character tilemap: {filename}"))?;
            character_frames.push(TitleScreenCharacterFrameData {
                frame_index,
                region_byte,
                width_words: plan.tilemap_load.width_words,
                tilemap_words,
            });
        }
    }

    // Get tile patch words from the load plan.
    // ASM loc_1C4E0 (kid.asm:31213): region byte 1 seeds the optional
    // 0x13-word title-palette overlay; native's title-screen cache currently
    // uses the default region path, so the symbolic plan has no overlay load.
    let plan = kid_data::title_screen_load_plan_sub_1B2A4(0);
    let tile_patch_words = plan.tile_patch_words;

    // ASM loc_1C4BE (kid.asm:31202): after loc_1C4B2 seeds `d0=#$32`,
    // the inclusive DBF loop copies 0x33 words from Pal_1C400 into
    // Palette_Buffer; native loads those same title palette words for CRAM.
    // ASM loc_1C4E2 (kid.asm:31216): non-default regions copy their selected
    // 0x13-word overlay into Palette_Buffer+$1A; the default region path leaves
    // the base palette unchanged before native writes it to CRAM.
    let palette_words = load_palette_words(&scenes.join("palette/title.bin"))?;

    Ok(TitleScreenData {
        art_chunks,
        title_art_chunks,
        plane_a_tilemap,
        title_text_chameleon_rows: title_text_chameleon_upload.rows,
        title_menu_tilemap_words,
        character_frames,
        tile_patch_words,
        palette_words,
    })
}

pub(crate) fn load_title_card_data(manifest: &AssetManifest) -> Result<TitleCardData> {
    let root = manifest.output_root.join("theme/titlecard");
    let mut themes = vec![None; TITLE_CARD_THEME_NAMES.len()];

    for (theme_index, maybe_name) in TITLE_CARD_THEME_NAMES.iter().enumerate() {
        let Some(name) = maybe_name else {
            continue;
        };
        let art_path = root.join("artcomp").join(format!("{name}.bin"));
        let map_path = root.join("mapeni").join(format!("{name}.bin"));
        let palette_path = root.join("palette").join(format!("{name}.bin"));
        let art_bytes = load_saxman_art(&art_path)
            .with_context(|| format!("failed to load title-card art for {name}"))?;
        let map_raw = std::fs::read(&map_path)
            .with_context(|| format!("failed to read title-card map: {}", map_path.display()))?;
        let tilemap_words = kid_data::decode_enigma(&map_raw, 0x02FB)
            .with_context(|| format!("failed to decode title-card map for {name}"))?;
        let palette_words = load_palette_words(&palette_path)
            .with_context(|| format!("failed to load title-card palette for {name}"))?;
        let (width_tiles, height_tiles) = TITLE_CARD_THEME_SIZES[theme_index];
        themes[theme_index] = Some(TitleCardThemeData {
            art_bytes,
            tilemap_words,
            palette_words,
            width_tiles,
            height_tiles,
        });
    }

    Ok(TitleCardData { themes })
}

/// Build VdpState for the title screen / IntroSequence6.
///
/// ASM Load_TitleScreen: fill VRAM with `$20BC`, run sub_1B2A4, Load_TitleArt,
/// sub_1BF24, then sub_1B532's plane-B title text upload.
#[cfg(test)]
fn build_title_screen_vdp_with_character_frame(
    data: &TitleScreenData,
    frame_index: u8,
    region_byte: u8,
) -> VdpState {
    build_title_screen_vdp_with_character_frame_and_menu(data, frame_index, region_byte, None)
}

pub(crate) fn build_title_screen_vdp_with_character_frame_and_menu(
    data: &TitleScreenData,
    frame_index: u8,
    region_byte: u8,
    menu_cursor: Option<u8>,
) -> VdpState {
    let mut vdp = VdpState::new();

    for (i, &value) in VDP_GAME_INIT_VALUES.iter().enumerate() {
        vdp.registers[i] = value;
    }
    vdp.registers[1] = 0x74;

    // ASM Load_TitleScreen loc_1B258 (kid.asm:29450): fill the first $800
    // VRAM words with tile word $20BC before title-specific uploads.
    for word_index in 0..=0x7FFusize {
        vdp.write_vram_word(word_index * 2, 0x20BC);
    }

    // ASM sub_1B2A4 (kid.asm:29472): upload the shared intro/title art chunks.
    for (vram_byte_addr, art_bytes) in &data.art_chunks {
        vdp.write_vram(*vram_byte_addr, art_bytes);
    }

    // ASM loc_1C4A8 (kid.asm:31197): write 32 raw tile-data words to VRAM $F000.
    for (i, &word) in data.tile_patch_words.iter().enumerate() {
        vdp.write_vram_word(0xF000 + i * 2, word);
    }

    // ASM loc_1C4BE/loc_1C4E2: write Pal_1C400 to Palette_Buffer/CRAM.
    vdp.write_cram_words(0, &data.palette_words);

    // ASM Load_TitleArt (kid.asm:29810): direct VRAM art uploads for the title
    // logo/menu elements. byte_22080 is RAM-only title text animation data.
    for (vram_byte_addr, art_bytes) in &data.title_art_chunks {
        vdp.write_vram(*vram_byte_addr, art_bytes);
    }

    write_title_character_frame_sub_1bf24(&mut vdp, data, frame_index, region_byte);

    // ASM sub_1B532 (kid.asm:29782): clear 8 longwords at $1780, then copy
    // 28 full plane rows from decoded `unk_2F840` to VRAM $E000.
    for word_index in 0..usize::from(8u16 * 2) {
        vdp.write_vram_word(0x1780 + word_index * 2, 0);
    }
    write_full_plane_rows(&mut vdp, 0xE000, &data.title_text_chameleon_rows);
    if let Some(cursor) = menu_cursor {
        write_title_menu_tilemap_loc_1bb52(&mut vdp, &data.title_menu_tilemap_words);
        write_title_menu_cursor_sprite_loc_1bb52(&mut vdp, cursor);
    }

    vdp
}

#[allow(non_snake_case)]
fn write_title_menu_tilemap_loc_1bb52(vdp: &mut VdpState, tilemap_words: &[u16]) {
    // ASM Obj_TitleMenu loc_1BB52 (kid.asm:30428): decode `unk_2EAAE` with
    // base `$451B`, then call sub_1C5D0 with d4=$04,d5=$04,d6=$0D,d7=$05.
    // That writes a 14x6 tile rectangle at cell (4,4), revealing the
    // interactive 1P/2P/options menu after the first Start press.
    write_tilemap_rect_at_cell(vdp, tilemap_words, 14, 6, 4, 4);
}

#[allow(non_snake_case)]
fn write_title_menu_cursor_sprite_loc_1bb52(vdp: &mut VdpState, cursor: u8) {
    // ASM Obj_TitleMenu loc_1BB52/loc_1BC0E (kid.asm:30438-30467) loads a
    // visible gfx slot on palette line 2, VRAM tile `$54B`, frame
    // `unk_E1196` (8x8), at x=$28 and y from word_1BC20.
    let y_pos = match cursor.min(2) {
        0 => 0x0030,
        1 => 0x003A,
        _ => 0x0044,
    };
    let mut sprite_entries = Vec::new();
    append_intro_sprite_entries_make_sprite_from_gfx_object_loc_129E(
        &mut sprite_entries,
        IntroGfxObjectForSpriteTable {
            x_pos: 0x0028,
            y_pos,
            palette_line: 2,
            vram_tile: 0x054B,
            priority: 0,
            x_direction: 0,
        },
        intro_sprite_frame_vram(0, 8, 8),
    );
    write_sprite_table_entries(
        vdp,
        sprite_table_base_byte_addr(&vdp.registers),
        &sprite_entries,
    );
}

fn write_title_character_frame_sub_1bf24(
    vdp: &mut VdpState,
    data: &TitleScreenData,
    frame_index: u8,
    region_byte: u8,
) {
    let Some(frame) = data
        .character_frames
        .iter()
        .find(|frame| frame.frame_index == frame_index && frame.region_byte == region_byte.min(2))
    else {
        write_title_character_tilemap_rect(vdp, &data.plane_a_tilemap, 18, 0x20BC);
        return;
    };

    write_title_character_tilemap_rect(vdp, &frame.tilemap_words, frame.width_words, 0x20BC);
}

fn write_title_character_tilemap_rect(
    vdp: &mut VdpState,
    tilemap_words: &[u16],
    source_width_words: u16,
    fill_word: u16,
) {
    // ASM sub_1BF24 (kid.asm:30740): decode the selected Enigma mappings,
    // fill an 18x20 staging rectangle with `$20BC`, copy `off_1BFB0`'s
    // per-frame width into each row, then blit it through sub_1C5D0 at
    // cell `(x,y)=(#$15,#1)`.
    const TILEMAP_WIDTH: usize = 18;
    const TILEMAP_HEIGHT: usize = 20;
    const CELL_X: usize = 0x15;
    const CELL_Y: usize = 1;
    let source_width = usize::from(source_width_words).min(TILEMAP_WIDTH);
    write_tilemap_rect_at_cell_with_fill(
        vdp,
        tilemap_words,
        TilemapRect {
            source_width,
            target_width: TILEMAP_WIDTH,
            target_height: TILEMAP_HEIGHT,
            cell_x: CELL_X,
            cell_y: CELL_Y,
        },
        fill_word,
    );
}

fn write_tilemap_rect_at_cell(
    vdp: &mut VdpState,
    tilemap_words: &[u16],
    width: usize,
    height: usize,
    cell_x: usize,
    cell_y: usize,
) {
    write_tilemap_rect_at_cell_with_fill(
        vdp,
        tilemap_words,
        TilemapRect {
            source_width: width,
            target_width: width,
            target_height: height,
            cell_x,
            cell_y,
        },
        0,
    );
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct TilemapRect {
    source_width: usize,
    target_width: usize,
    target_height: usize,
    cell_x: usize,
    cell_y: usize,
}

fn write_tilemap_rect_at_cell_with_fill(
    vdp: &mut VdpState,
    tilemap_words: &[u16],
    rect: TilemapRect,
    fill_word: u16,
) {
    for row in 0..rect.target_height {
        for col in 0..rect.target_width {
            let word = if col < rect.source_width {
                tilemap_words
                    .get(row * rect.source_width + col)
                    .copied()
                    .unwrap_or(fill_word)
            } else {
                fill_word
            };
            let byte_addr =
                (rect.cell_y + row) * NAMETABLE_ROW_STRIDE_BYTES + (rect.cell_x + col) * 2;
            vdp.write_vram_word(byte_addr, word);
        }
    }
}

/// Build VdpState for the level title-card shell.
///
/// ASM `Load_TitleCard` (kid.asm:28588) runs `sub_914`, decompresses the
/// selected theme art to VRAM `$5F60`, decodes the theme Enigma map with base
/// word `$02FB`, copies the theme palette, and centres the decoded map in
/// Scroll A's normal `$0000` nametable with `$02FB` padding.
pub(crate) fn build_title_card_vdp(data: &TitleCardData, theme_index: u8) -> Option<VdpState> {
    let theme = data
        .themes
        .get(usize::from(theme_index))
        .and_then(Option::as_ref)?;
    let mut vdp = VdpState::new();

    for (i, &value) in VDP_GAME_INIT_VALUES.iter().enumerate() {
        vdp.registers[i] = value;
    }
    vdp.registers[1] = 0x74;

    vdp.write_vram(0x5F60, &theme.art_bytes);
    for i in 0..0x01C0 {
        vdp.write_vram_word(0x1400 + i * 2, 0);
    }
    for i in 0..0x0800 {
        vdp.write_vram_word(0xE000 + i * 2, 0x0094);
    }
    for i in 0..0x0010 {
        vdp.write_vram_word(0x1280 + i * 2, 0);
    }
    vdp.write_cram_words(0, &theme.palette_words);

    let width = usize::from(theme.width_tiles);
    let height = usize::from(theme.height_tiles);
    let top_rows = usize::from((0x001Cu16.saturating_sub(theme.height_tiles)) >> 1);
    let left_words = usize::from((0x0028u16.saturating_sub(theme.width_tiles)) >> 1);
    let bottom_rows = 0x001Cusize.saturating_sub(top_rows + height);
    let right_words = 64usize.saturating_sub(left_words + width);
    let mut dst_word_index = 0usize;

    for _ in 0..top_rows {
        for _ in 0..64 {
            vdp.write_vram_word(dst_word_index * 2, 0x02FB);
            dst_word_index += 1;
        }
    }

    for row in 0..height {
        for _ in 0..left_words {
            vdp.write_vram_word(dst_word_index * 2, 0x02FB);
            dst_word_index += 1;
        }
        for col in 0..width {
            let src_index = row * width + col;
            let word = theme
                .tilemap_words
                .get(src_index)
                .copied()
                .unwrap_or(0x02FB);
            vdp.write_vram_word(dst_word_index * 2, word);
            dst_word_index += 1;
        }
        for _ in 0..right_words {
            vdp.write_vram_word(dst_word_index * 2, 0x02FB);
            dst_word_index += 1;
        }
    }

    for _ in 0..bottom_rows {
        for _ in 0..64 {
            vdp.write_vram_word(dst_word_index * 2, 0x02FB);
            dst_word_index += 1;
        }
    }

    Some(vdp)
}

// ─── Helpers ────────────────────────────────────────────────────────────────

/// Load raw palette file as big-endian u16 words.
fn load_palette_words(path: &std::path::Path) -> Result<Vec<u16>> {
    let raw = std::fs::read(path)
        .with_context(|| format!("failed to read palette: {}", path.display()))?;
    Ok(raw
        .chunks_exact(2)
        .map(|chunk| u16::from_be_bytes([chunk[0], chunk[1]]))
        .collect())
}

/// Load and decompress a Saxman-compressed art file.
///
/// Mirrors ASM `jsr (j_KosDec)` / `jsr (j_SaxDec)` style intro-art uploads
/// (e.g. sub_1B850 in kid.asm:30172) where each `move.l #ArtAddr,a0` is
/// followed by a decompression call before the resulting bytes are streamed
/// into VRAM.
fn load_saxman_art(path: &std::path::Path) -> Result<Vec<u8>> {
    let compressed = std::fs::read(path)
        .with_context(|| format!("failed to read intro art: {}", path.display()))?;
    kid_data::decode_saxman(&compressed)
        .with_context(|| format!("failed to decompress intro art: {}", path.display()))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn empty_intro_data() -> IntroSub1B850Data {
        IntroSub1B850Data {
            art_chunks: [
                (0, vec![]),
                (0, vec![]),
                (0, vec![]),
                (0, vec![]),
                (0, vec![]),
            ],
            plane_a_tilemap: vec![],
            plane_b_rows: [[0; 64]; 28],
            palette_segment_0: vec![],
            palette_segment_1: vec![],
            palette_segment_2: vec![],
            hologram_background_art: vec![],
            hologram_blocks_art: vec![],
            intro_text_letters_art: vec![],
            intro_sequence_2_hologram_tilemap: vec![],
            intro_sequence_4_hologram_tilemap: vec![],
            intro_sequence_2_tilemap: vec![],
            intro_sequence_5_common_art_chunks: [(0, vec![]), (0, vec![]), (0, vec![])],
            intro_sequence_5_helmet_art_variants: [vec![], vec![], vec![]],
            intro_sequence_5_palette_variant_1_overlay: vec![],
            intro_sequence_5_palette_variant_2_overlay: vec![],
            intro_sequence_5_tilemap: vec![],
            intro_sequence_5_frame_tilemaps: vec![],
            intro_sequence_5_panel_tilemap: vec![],
        }
    }

    fn vram_word(vdp: &VdpState, byte_addr: usize) -> u16 {
        u16::from_be_bytes([vdp.vram[byte_addr], vdp.vram[byte_addr + 1]])
    }

    fn sprite_entry_words(vdp: &VdpState, entry_index: usize) -> (u16, u8, u8, u16, u16) {
        let entry_base = 0x1000 + entry_index * 8;
        (
            vram_word(vdp, entry_base),
            vdp.vram[entry_base + 2],
            vdp.vram[entry_base + 3],
            vram_word(vdp, entry_base + 4),
            vram_word(vdp, entry_base + 6),
        )
    }

    fn sprite_table_contains(vdp: &VdpState, expected: (u16, u8, u8, u16, u16)) -> bool {
        (0..80).any(|entry_index| sprite_entry_words(vdp, entry_index) == expected)
    }

    #[test]
    fn vdp_state_new_has_correct_sizes() {
        let vdp = VdpState::new();
        assert_eq!(vdp.vram.len(), VRAM_SIZE);
        assert_eq!(vdp.cram.len(), CRAM_SIZE);
        assert_eq!(vdp.vsram.len(), VSRAM_SIZE);
        assert_eq!(vdp.registers.len(), 0x20);
    }

    #[test]
    fn write_vram_word_stores_big_endian() {
        let mut vdp = VdpState::new();
        vdp.write_vram_word(0x100, 0x80BC);
        assert_eq!(vdp.vram[0x100], 0x80);
        assert_eq!(vdp.vram[0x101], 0xBC);
    }

    #[test]
    #[allow(non_snake_case)]
    fn intro_sequence_3_loc_1AEEC_writes_rom_roster_sprite_table() {
        let vdp = build_intro_sequence_vdp_with_title_variant_at_elapsed(
            &empty_intro_data(),
            2,
            1,
            INTRO_SEQUENCE_3_KID_VISIBLE_ELAPSED_FRAMES,
            1,
        );

        assert_eq!(
            sprite_entry_words(&vdp, 0),
            (0x0120, 0x0F, 1, 0xA1D6, 0x0178),
            "ASM Load_IntroSequence3 loc_1AEEC should expose roster 1 descriptor 0 through Make_SpriteFromGfxObject's first 32x32 hardware sprite",
        );
        assert_eq!(
            sprite_entry_words(&vdp, 1),
            (0x0120, 0x07, 2, 0xA1E6, 0x0198),
            "ASM Make_SpriteFromGfxObject should split the 48px-wide Wild Side kid frame into a linked 16x32 tail chunk with the next tile run",
        );
    }

    #[test]
    #[allow(non_snake_case)]
    fn intro_sequence_3_loc_1AEEC_uses_active_roster_index() {
        let vdp = build_intro_sequence_vdp_with_title_variant_at_elapsed(
            &empty_intro_data(),
            2,
            1,
            INTRO_SEQUENCE_3_KID_VISIBLE_ELAPSED_FRAMES,
            2,
        );

        assert_eq!(
            sprite_entry_words(&vdp, 0),
            (0x011D, 0x0F, 1, 0xA1D6, 0x0163),
            "ASM loc_1AEBA should choose roster 2's first descriptor instead of reusing roster 1's first x/y pair",
        );
    }

    #[test]
    #[allow(non_snake_case)]
    fn intro_sequence_3_loc_1AEEC_is_hidden_before_visible_elapsed() {
        let vdp = build_intro_sequence_vdp_with_title_variant_at_elapsed(
            &empty_intro_data(),
            2,
            1,
            INTRO_SEQUENCE_3_KID_VISIBLE_ELAPSED_FRAMES - 1,
            1,
        );

        assert_eq!(
            sprite_entry_words(&vdp, 0),
            (0, 0, 0, 0, 0),
            "Trace-backed Load_IntroSequence3 hidden rebuild should not populate the sprite table before loc_1AEEC's visible roster frame",
        );
    }

    #[test]
    #[allow(non_snake_case)]
    fn intro_sequence_3_sub_1C54A_keeps_wild_side_door_sprite_visible() {
        let vdp = build_intro_sequence_vdp_with_title_variant_at_elapsed(
            &empty_intro_data(),
            2,
            1,
            INTRO_SEQUENCE_3_KID_VISIBLE_ELAPSED_FRAMES,
            1,
        );

        assert!(
            sprite_table_contains(&vdp, (0x0119, 0x0F, 21, 0x8140, 0x0101)),
            "ASM Load_IntroSequence3 runs sub_1B93E -> sub_1C54A, so the fixed Wild Side door/interior sprite should be present after the roster entries",
        );
    }

    #[test]
    #[allow(non_snake_case)]
    fn intro_sequence_3_sub_1C572_draws_and_slides_door_halves() {
        let initial = build_intro_sequence_vdp_with_title_variant_at_elapsed(
            &empty_intro_data(),
            2,
            1,
            INTRO_SEQUENCE_3_KID_VISIBLE_ELAPSED_FRAMES,
            1,
        );

        assert!(
            sprite_table_contains(&initial, (0x0118, 0x0F, 25, 0x8170, 0x00BA)),
            "ASM sub_1C572 should draw the left sliding Wild Side door half at x=$3A before the `$3C` hibernate expires",
        );
        assert!(
            sprite_table_contains(&initial, (0x0118, 0x0F, 29, 0x818E, 0x015E)),
            "ASM loc_1C58C should draw the right sliding Wild Side door half at x=$DE before the `$3C` hibernate expires",
        );

        let after_first_move = build_intro_sequence_vdp_with_title_variant_at_elapsed(
            &empty_intro_data(),
            2,
            1,
            INTRO_SEQUENCE_3_KID_VISIBLE_ELAPSED_FRAMES + 0x3D,
            1,
        );

        assert!(
            sprite_table_contains(&after_first_move, (0x0118, 0x0F, 25, 0x8170, 0x00BB)),
            "ASM loc_1C5BE should move the left door half right by one pixel after its first post-hibernate frame",
        );
        assert!(
            sprite_table_contains(&after_first_move, (0x0118, 0x0F, 29, 0x818E, 0x015D)),
            "ASM loc_1C5BE should move the right door half left by one pixel after its first post-hibernate frame",
        );
    }

    #[test]
    #[allow(non_snake_case)]
    fn intro_sequence_1_sub_1C54A_writes_wild_side_door_sprite_table() {
        let vdp = build_intro_sequence_vdp_with_title_variant_at_elapsed(
            &empty_intro_data(),
            0,
            1,
            INTRO_SEQUENCE_1_PROMPT_VISIBLE_ELAPSED_SUB_1C54A,
            0,
        );

        assert_eq!(
            sprite_entry_words(&vdp, 0),
            (0x0119, 0x0F, 1, 0x8140, 0x0101),
            "ASM sub_1C54A should draw unk_E109E's first 32x32 chunk from the Wild Side door/interior art at VRAM tile `$140`",
        );
        assert_eq!(
            sprite_entry_words(&vdp, 1),
            (0x0119, 0x0F, 2, 0x8150, 0x0121),
            "ASM Make_SpriteFromGfxObject should split unk_E109E's 62px-wide top row into the linked right-hand chunk",
        );
        assert_eq!(
            sprite_entry_words(&vdp, 2),
            (0x0139, 0x0D, 3, 0x8160, 0x0101),
            "ASM Make_SpriteFromGfxObject should continue unk_E109E's bottom row after the two 32px-high chunks",
        );
        assert_eq!(
            sprite_entry_words(&vdp, 3),
            (0x0139, 0x0D, 0, 0x8168, 0x0121),
            "ASM Make_SpriteFromGfxObject should terminate the final linked chunk for sub_1C54A's fixed prompt sprite",
        );
    }

    #[test]
    #[allow(non_snake_case)]
    fn intro_sequence_1_sub_1C54A_is_hidden_before_controller_exposes_it() {
        let vdp = build_intro_sequence_vdp_with_title_variant_at_elapsed(
            &empty_intro_data(),
            0,
            1,
            INTRO_SEQUENCE_1_PROMPT_VISIBLE_ELAPSED_SUB_1C54A - 1,
            0,
        );

        assert_eq!(
            sprite_entry_words(&vdp, 0),
            (0, 0, 0, 0, 0),
            "ASM sub_1B850's raw VDP rebuild should not contain the sub_1C54A sprite before the intro controller allocates it",
        );
    }

    #[test]
    #[allow(non_snake_case)]
    fn intro_sequence_5_loc_1BCD0_writes_five_static_gfx_slots_to_sprite_table() {
        let vdp =
            build_intro_sequence_vdp_with_title_variant_at_elapsed(&empty_intro_data(), 4, 1, 0, 0);

        assert_eq!(
            sprite_entry_words(&vdp, 0),
            (0x00D0, 0x0F, 1, 0x426C, 0x00B0),
            "ASM loc_1BCD0 first gfx slot should draw unk_E1066 from VRAM tile $26C on palette line 2",
        );
        assert_eq!(
            sprite_entry_words(&vdp, 1),
            (0x00D0, 0x07, 2, 0x427C, 0x00D0),
            "ASM Make_SpriteFromGfxObject should link the first slot's 16x32 tail chunk after the 32x32 chunk",
        );
        assert_eq!(
            sprite_entry_words(&vdp, 4),
            (0x00CF, 0x07, 5, 0x4290, 0x00E0),
            "ASM loc_1BCD0 second gfx slot should start at x=$60/y=$80 with unk_E106E's 12x49 frame",
        );
    }

    #[test]
    #[allow(non_snake_case)]
    fn intro_sequence_5_loc_1BE3C_particles_spawn_after_loc_1BCD0_hibernate() {
        let before = build_intro_sequence_vdp_with_title_variant_at_elapsed(
            &empty_intro_data(),
            4,
            1,
            INTRO_SEQUENCE_5_PARTICLE_START_ELAPSED_LOC_1BCD0 - 1,
            0,
        );
        assert_eq!(
            sprite_entry_words(&before, 16),
            (0, 0, 0, 0, 0),
            "ASM loc_1BCD0 should not allocate loc_1BE3C particles until the `$1A0` hibernate expires",
        );

        let after = build_intro_sequence_vdp_with_title_variant_at_elapsed(
            &empty_intro_data(),
            4,
            1,
            INTRO_SEQUENCE_5_PARTICLE_START_ELAPSED_LOC_1BCD0,
            0,
        );
        assert_eq!(
            sprite_entry_words(&after, 16),
            (0x00F8, 0x00, 17, 0x44F8, 0x0092),
            "ASM loc_1BE3C should spawn the first unk_1BE18 falling particle from LnkTo_unk_E0F56 at x=$12/y=$78",
        );
        assert_eq!(
            sprite_entry_words(&after, 19),
            (0x00B9, 0x0F, 20, 0xE5EB, 0x00B0),
            "ASM loc_1BE88 should spawn the FC82=1 centre particle on palette line 3 with priority and VRAM tile $5EB",
        );
    }

    #[test]
    #[allow(non_snake_case)]
    fn intro_sequence_5_loc_1BE3C_deletes_short_lived_particles_by_dbf_count() {
        let vdp = build_intro_sequence_vdp_with_title_variant_at_elapsed(
            &empty_intro_data(),
            4,
            1,
            INTRO_SEQUENCE_5_PARTICLE_START_ELAPSED_LOC_1BCD0 + 0x29,
            0,
        );

        assert_eq!(
            sprite_entry_words(&vdp, 16),
            (0x016D, 0x00, 17, 0x44F8, 0x017C),
            "ASM loc_1BE74 should delete the two `$28` DBF particles after frame $28, leaving the longer unk_1BE24 particle as the first live particle",
        );
    }

    #[test]
    fn intro_sequence_text_worker_waits_for_rom_initial_and_fe_delay() {
        let mut baked_text_tilemap = vec![0u16; 40 * 28];
        baked_text_tilemap[2 * 40 + 8] = 0x67E6;
        let data = IntroSub1B850Data {
            plane_a_tilemap: baked_text_tilemap,
            ..empty_intro_data()
        };
        let first_glyph_elapsed = INTRO_TEXT_WORKER_START_ELAPSED_LOC_1C7A0[0] + 1 + 0x20;

        // ASM sub_1B8F4 allocates loc_1C7A0 before the visible text writes;
        // loc_1C7A0 then hibernates once and IntroText1 starts with `$FE,$20`,
        // so a baked-in tilemap glyph must be cleared until the effective
        // text-write start plus loc_1C7AE's initial hibernate and marker delay.
        let before = build_intro_sequence_vdp_with_title_variant_at_elapsed(
            &data,
            0,
            1,
            first_glyph_elapsed - 1,
            0,
        );
        assert_eq!(
            vram_word(&before, 0x0110),
            INTRO_TEXT_CLEAR_TILE_WORD_LOC_1C7A0,
            "ASM loc_1C7A0/loc_1C7D2 should leave the first IntroText1 glyph hidden until the initial hibernate and `$FE,$20` delay expire",
        );

        let after = build_intro_sequence_vdp_with_title_variant_at_elapsed(
            &data,
            0,
            1,
            first_glyph_elapsed,
            0,
        );
        assert_eq!(
            vram_word(&after, 0x0110),
            0x67E6,
            "ASM sub_1C7F6 should write IntroText1's first `T` tile only once loc_1C7A0 reaches the glyph path",
        );
    }

    #[test]
    fn intro_sequence_text_worker_honours_second_fe_delay_before_third_line() {
        let data = empty_intro_data();
        let first_line = b"THERE WAS A NEW MACHINE";
        let second_line = b"IN THE ARCADE";
        let worker_elapsed_before_third_line =
            1 + 0x20 + first_line.len() * 2 + second_line.len() * 2 + 0x1F;
        let elapsed_before_third_line =
            INTRO_TEXT_WORKER_START_ELAPSED_LOC_1C7A0[0] + worker_elapsed_before_third_line;
        let third_line_first_glyph_addr = 0x0310;

        let before = build_intro_sequence_vdp_with_title_variant_at_elapsed(
            &data,
            0,
            1,
            elapsed_before_third_line,
            0,
        );
        assert_eq!(
            vram_word(&before, third_line_first_glyph_addr),
            INTRO_TEXT_CLEAR_TILE_WORD_LOC_1C7A0,
            "ASM IntroText1's second `$FE,$20` pause should stop the third line being pre-rendered",
        );

        let after = build_intro_sequence_vdp_with_title_variant_at_elapsed(
            &data,
            0,
            1,
            elapsed_before_third_line + 1,
            0,
        );
        assert_eq!(
            vram_word(&after, third_line_first_glyph_addr),
            0x67E6,
            "ASM loc_1C7D2 should reveal the third line immediately after the second `$FE,$20` delay expires",
        );
    }

    #[test]
    fn intro_sequence_text_cleanup_uses_blank_pattern_not_tile_zero() {
        let mut baked_text_tilemap = vec![0u16; 40 * 28];
        baked_text_tilemap[2 * 40 + 8] = 0x67E6;
        let data = IntroSub1B850Data {
            plane_a_tilemap: baked_text_tilemap,
            ..empty_intro_data()
        };

        let vdp = build_intro_sequence_vdp_with_title_variant_at_elapsed(
            &data,
            0,
            1,
            INTRO_TEXT_WORKER_START_ELAPSED_LOC_1C7A0[0] - 1,
            0,
        );

        assert_eq!(
            vram_word(&vdp, 0x0110),
            INTRO_TEXT_CLEAR_TILE_WORD_LOC_1C7A0,
            "ASM loc_1C7A0/sub_1C7F6 writes runtime glyphs later; native's baked-map cleanup must use a zeroed pattern tile instead of tile 0, because plane-A nametable bytes occupy tile 0's pattern slot",
        );
    }

    #[test]
    fn write_vram_word_at_origin_writes_both_bytes() {
        // ASM `move.w d1,(a6)` against a VRAM-write VDP control word writes
        // the high byte of d1 first and then the low byte at the next address.
        // Pinning byte_addr=0 separately from the existing 0x100 case means
        // any mutant that turns `byte_addr + 1` (line 80) into `byte_addr - 1`
        // underflows the bound check at the lowest legal address — the
        // 0x100 case wouldn't notice because the underflowed value is still
        // less than VRAM_SIZE.
        let mut vdp = VdpState::new();
        vdp.write_vram_word(0, 0x80BC);
        assert_eq!(vdp.vram[0], 0x80, "high byte at byte_addr=0");
        assert_eq!(vdp.vram[1], 0xBC, "low byte at byte_addr=0+1");
    }

    #[test]
    fn write_vram_word_skips_when_low_byte_would_overrun() {
        // ASM treats VDP VRAM as a 64KB linear buffer; native mirrors that
        // with `vram: vec![0; VRAM_SIZE]`.  At byte_addr = VRAM_SIZE - 1 the
        // high byte fits at vram[VRAM_SIZE-1] but the low byte would need
        // vram[VRAM_SIZE], so the bound `byte_addr + 1 < self.vram.len()`
        // (line 80) must reject the write.  Any mutant that loosens the
        // comparison to `<=` (or perturbs `+ 1` to `* 1`) would index past
        // the end and panic; the original silently skips and leaves the
        // last byte untouched.
        let mut vdp = VdpState::new();
        let last = VRAM_SIZE - 1;
        vdp.write_vram_word(last, 0x80BC);
        assert_eq!(
            vdp.vram[last], 0,
            "boundary write at byte_addr=VRAM_SIZE-1 must be rejected, vram[last] stays 0",
        );
    }

    #[test]
    fn write_cram_words_stores_big_endian() {
        let mut vdp = VdpState::new();
        vdp.write_cram_words(0, &[0x0EEE, 0x0444]);
        assert_eq!(vdp.cram[0], 0x0E);
        assert_eq!(vdp.cram[1], 0xEE);
        assert_eq!(vdp.cram[2], 0x04);
        assert_eq!(vdp.cram[3], 0x44);
    }

    #[test]
    fn write_cram_words_at_nonzero_offset_advances_byte_offset() {
        // ASM `lea (Palette_Buffer+$4).l,a1; move.w (a0)+,(a1)+` writes word
        // strips at non-zero CRAM offsets — `Palette_Buffer+$4` corresponds
        // to CRAM word offset 2 (byte offset 4).  Driving write_cram_words
        // with a non-zero `word_offset` AND multiple words pins the
        // `(word_offset + i) * 2` byte-offset formula on line 90: any mutant
        // that flips the `*` to `+` lands the second word at byte_offset
        // word_offset+i+2 instead of (word_offset+i)*2, and any mutant that
        // flips the `+` to `*` lands word i at i*word_offset*2 — both shift
        // the second strip word away from cram[6..8].
        let mut vdp = VdpState::new();
        vdp.write_cram_words(2, &[0x0EEE, 0x0444]);
        assert_eq!(vdp.cram[4], 0x0E, "word 0 high byte at cram[(2+0)*2]");
        assert_eq!(vdp.cram[5], 0xEE, "word 0 low byte at cram[(2+0)*2 + 1]");
        assert_eq!(vdp.cram[6], 0x04, "word 1 high byte at cram[(2+1)*2]");
        assert_eq!(vdp.cram[7], 0x44, "word 1 low byte at cram[(2+1)*2 + 1]");
    }

    #[test]
    fn sega_nametable_covers_48_consecutive_tiles() {
        let data = SegaScreenData {
            art_bytes: vec![0; 48 * 32], // 48 tiles × 32 bytes each
            palette_words: vec![0; 33],
        };
        let vdp = build_sega_screen_vdp(&data, 0);

        // Check first tile word at VRAM $061C.
        let w0 = u16::from_be_bytes([vdp.vram[0x061C], vdp.vram[0x061D]]);
        assert_eq!(
            w0, 0x80BC,
            "first nametable entry should be tile $BC with priority"
        );

        // Check last tile word (4th row, 12th column) at $079C + 11*2 = $07B2.
        let last_addr = 0x079C + 11 * 2;
        let w_last = u16::from_be_bytes([vdp.vram[last_addr], vdp.vram[last_addr + 1]]);
        assert_eq!(
            w_last,
            0x80BC + 47,
            "last nametable entry should be tile $BC+47"
        );
    }

    #[test]
    fn sega_registers_match_game_init_with_display_on() {
        let data = SegaScreenData {
            art_bytes: vec![],
            palette_words: vec![0; 33],
        };
        let vdp = build_sega_screen_vdp(&data, 0);

        // Register 1 should be $74 (display on, VInt on, DMA on).
        assert_eq!(vdp.registers[1], 0x74);
        // Register 4 should be $07 (Scroll B at VRAM $E000).
        assert_eq!(vdp.registers[4], 0x07);
        // Register 0x10 should be $01 (64×32 scroll size).
        assert_eq!(vdp.registers[0x10], 0x01);
    }

    #[test]
    fn write_vram_clips_past_end() {
        let mut vdp = VdpState::new();
        // Write at address near end — should not panic.
        vdp.write_vram(0xFFF0, &[0xAA; 32]);
        assert_eq!(vdp.vram[0xFFF0], 0xAA);
        assert_eq!(vdp.vram[0xFFFF], 0xAA);
        // Write past end — should silently do nothing.
        vdp.write_vram(VRAM_SIZE + 100, &[0xBB; 10]);
    }

    #[test]
    fn build_sega_screen_vdp_fade_loc_12e96_first_palette_strip_lands_at_elapsed_32() {
        // ASM Load_SegaScreen loc_12E64 (kid.asm:25497-25510): the initial
        // wait loop seeds `moveq #$A,d2` then `dbf d2,loc_12E6C`, so it spends
        // exactly 11 frames before the fade outer loop at loc_12E96 starts.
        // That outer loop seeds `moveq #$14,d1` (frame delay) and `moveq
        // #$28,d3` (palette cursor).  loc_12E96 (kid.asm:25515-25543) does
        // `subq.w #1,d1; bpl.s loc_12ED8`, so the first palette copy at
        // loc_12ED2 (kid.asm:25537) only fires when d1 transitions from 0 to
        // -1 — exactly 21 fade iterations later.  At that point d0 captures
        // the cursor BEFORE `subq.w #2,d3`, and `lea Pal_12EEC+6(pc,d0.w),a0`
        // resolves to palette word index 2 + d0/2 (Pal_12EEC = [dest, count,
        // SegaLogo.bin...]; the +6 offset and the absent header in
        // `palette_words` cancel to leave +2 word indices).  loc_12ED2 then
        // copies `moveq #$A,d0; dbf d0` = 11 words into Palette_Buffer+$4
        // (CRAM word offset 2).  This test seeds palette_words with distinct
        // values so any mutation of `2 + cursor/2`, the strip slice, the
        // fade-strip CRAM destination, or the cursor decrement lands the
        // wrong source words at the wrong CRAM offset.
        let mut palette_words = vec![0u16; 33];
        for (i, slot) in palette_words.iter_mut().enumerate() {
            *slot = 0x1000 + i as u16;
        }
        let data = SegaScreenData {
            art_bytes: vec![0; 32],
            palette_words: palette_words.clone(),
        };

        // Sanity: at elapsed=11 the initial wait loop is still consuming
        // frames, so the fade outer loop hasn't started — CRAM word 2 must
        // still match the initial Pal_12EEC copy at loc_12DFE (which writes
        // palette_words[0..13] to CRAM word offset 0).
        let vdp_initial = build_sega_screen_vdp(&data, SEGA_INITIAL_WAIT_FRAMES);
        assert_eq!(
            u16::from_be_bytes([vdp_initial.cram[4], vdp_initial.cram[5]]),
            palette_words[2],
            "elapsed=11: fade not yet running, CRAM word 2 = initial palette word 2",
        );

        // Sanity: at elapsed=12, the fade outer loop has run 1 iteration but
        // d1 went 0x14→0x13 — still bpl, no copy.  CRAM word 2 must still be
        // the initial palette word 2.
        let vdp_one_iter = build_sega_screen_vdp(&data, SEGA_INITIAL_WAIT_FRAMES + 1);
        assert_eq!(
            u16::from_be_bytes([vdp_one_iter.cram[4], vdp_one_iter.cram[5]]),
            palette_words[2],
            "elapsed=12: 1 fade iteration, d1=$13, no copy yet",
        );

        // First copy fires after exactly 21 fade iterations (d1 = 0x14, 0x13,
        // ..., 0, -1 → not bpl).  Cursor on entry is $28, so source_word_index
        // = 2 + 0x28/2 = 22.  All 11 strip words must equal palette_words[22..33].
        let vdp_first_copy = build_sega_screen_vdp(&data, SEGA_INITIAL_WAIT_FRAMES + 21);
        for i in 0..SEGA_FADE_STRIP_WORDS {
            let cram_byte_offset = (SEGA_FADE_STRIP_CRAM_WORD_OFFSET + i) * 2;
            let actual = u16::from_be_bytes([
                vdp_first_copy.cram[cram_byte_offset],
                vdp_first_copy.cram[cram_byte_offset + 1],
            ]);
            assert_eq!(
                actual,
                palette_words[22 + i],
                "first fade copy: CRAM word {} = palette_words[{}]",
                SEGA_FADE_STRIP_CRAM_WORD_OFFSET + i,
                22 + i,
            );
        }

        // Second copy fires 4 iterations later (d1 reset to 3, then 3,2,1,0,-1).
        // Cursor was $28, decremented by 2 after the first copy → $26 on
        // entry.  source_word_index = 2 + 0x26/2 = 21.  Strip = palette_words[21..32].
        let vdp_second_copy = build_sega_screen_vdp(&data, SEGA_INITIAL_WAIT_FRAMES + 25);
        for i in 0..SEGA_FADE_STRIP_WORDS {
            let cram_byte_offset = (SEGA_FADE_STRIP_CRAM_WORD_OFFSET + i) * 2;
            let actual = u16::from_be_bytes([
                vdp_second_copy.cram[cram_byte_offset],
                vdp_second_copy.cram[cram_byte_offset + 1],
            ]);
            assert_eq!(
                actual,
                palette_words[21 + i],
                "second fade copy: CRAM word {} = palette_words[{}]",
                SEGA_FADE_STRIP_CRAM_WORD_OFFSET + i,
                21 + i,
            );
        }

        // After 22 copies (elapsed = 11 initial + 21 + 21*4 = 116), cursor has
        // decremented from $28 to 0 and the 22nd copy uses source_word_index =
        // 2 + 0/2 = 2.  ASM loc_12E96 (kid.asm:25531): `bmi.w loc_12ED8`
        // skips only when d3 (the cursor) is already negative — at d3=0 the
        // branch is NOT taken, so the copy still fires for cursor==0.  This
        // pins the strict-less-than bound on line 161 (`palette_cursor < 0`):
        // any mutant that loosens it to `<=`/`==` would skip the cursor==0
        // copy and leave CRAM word 2 holding the previous strip's value
        // (palette_words[3], from the 21st copy at cursor=2).
        let vdp_cursor_zero = build_sega_screen_vdp(&data, SEGA_INITIAL_WAIT_FRAMES + 21 + 21 * 4);
        assert_eq!(
            u16::from_be_bytes([vdp_cursor_zero.cram[4], vdp_cursor_zero.cram[5]]),
            palette_words[2],
            "22nd fade copy at cursor=0: CRAM word 2 = palette_words[2 + 0/2] = palette_words[2]",
        );
    }

    #[test]
    fn intro_sequence_plane_a_tilemap_at_nametable_origin() {
        // ASM sub_1B850 (kid.asm:30193-30197) seeds d4=0, d5=0, d6=$28, d7=$1C
        // before bsr.w sub_1C5D0, and sub_1C5D0 (kid.asm:31341-31363) reads the
        // source pointer linearly with `move.w (a4)+,(a6)` for d6+1 = 40 words
        // per row while advancing the VRAM target by `addi.l #$800000` (128
        // bytes = one nametable row) for d7+1 = 28 rows. The source is
        // therefore row-major with stride 40, the VRAM target is row*128 +
        // col*2, and only columns 0..39 of each row are written.
        //
        // Each cell carries a unique value `(row << 8) | col | 0x8000` so that
        // mutating the source-side word index (`row * 40 + col`) to anything
        // else lands the wrong row at the right VRAM address — uniform fill
        // values cannot detect that class of mutation.
        let mut tilemap = vec![0u16; 40 * 28];
        for row in 0..28u16 {
            for col in 0..40u16 {
                tilemap[(row as usize) * 40 + (col as usize)] = 0x8000 | (row << 8) | col;
            }
        }
        let data = IntroSub1B850Data {
            plane_a_tilemap: tilemap,
            ..empty_intro_data()
        };
        let vdp = build_intro_sequence_vdp(&data, 0);

        // Sample row-major source-vs-target pairs: corners and one interior
        // cell so any mutation that breaks `row * 40 + col` indexing or
        // `row * 128 + col * 2` VRAM addressing produces a divergence.
        for &(row, col) in &[(0u16, 0u16), (0, 39), (1, 0), (5, 7), (27, 39)] {
            let addr = (row as usize) * 128 + (col as usize) * 2;
            let w = u16::from_be_bytes([vdp.vram[addr], vdp.vram[addr + 1]]);
            let expected = 0x8000 | (row << 8) | col;
            assert_eq!(
                w, expected,
                "plane A cell ({row},{col}) at byte ${addr:04X}"
            );
        }

        // Column 40 is past sub_1C5D0's d6=$28 inner-loop bound, so the byte
        // immediately after the last written column on row 0 must remain zero.
        let untouched = u16::from_be_bytes([vdp.vram[80], vdp.vram[81]]);
        assert_eq!(
            untouched, 0,
            "ASM sub_1C5D0 only writes 40 words per row; column 40 must stay 0"
        );
    }

    #[test]
    fn intro_sequence_plane_b_repeated_rows_at_e000() {
        // ASM sub_1B850: each of 28 decoded words stamped across 64 columns
        // at VRAM $E000, row stride 128 bytes.
        let mut rows = [[0u16; 64]; 28];
        rows[0] = [0xAAAA; 64];
        rows[27] = [0xBBBB; 64];
        let data = IntroSub1B850Data {
            plane_b_rows: rows,
            ..empty_intro_data()
        };
        let vdp = build_intro_sequence_vdp(&data, 0);

        // Row 0, col 0 at $E000.
        let w = u16::from_be_bytes([vdp.vram[0xE000], vdp.vram[0xE001]]);
        assert_eq!(w, 0xAAAA, "plane B row 0 col 0");

        // Row 0, col 63 at $E000 + 63*2 = $E07E.
        let w = u16::from_be_bytes([vdp.vram[0xE07E], vdp.vram[0xE07F]]);
        assert_eq!(w, 0xAAAA, "plane B row 0 col 63");

        // Row 27, col 0 at $E000 + 27*128 = $E000 + $D80 = $ED80.
        let w = u16::from_be_bytes([vdp.vram[0xED80], vdp.vram[0xED81]]);
        assert_eq!(w, 0xBBBB, "plane B row 27 col 0");
    }

    #[test]
    fn write_full_plane_rows_pins_byte_address_per_row_and_col() {
        // ASM sub_1CD88 (kid.asm:31931-31948): writes 28 plane rows starting
        // at the VRAM target seeded by the caller, advancing by `addi.l
        // #$800000` (= 128 bytes = one nametable row stride) between rows.
        // Native's write_full_plane_rows mirrors that with `plane_base + row *
        // NAMETABLE_ROW_STRIDE_BYTES + col * 2`.  Driving the helper with
        // unique cell values pins both the row-stride term and the col*2 term
        // — any mutation that swaps `+` for `-`/`*` or `*` for `+`/`/` lands
        // the wrong cell at the right address (or vice versa).
        let mut vdp = VdpState::new();
        let mut rows = [[0u16; 64]; 28];
        for (row, row_words) in rows.iter_mut().enumerate() {
            for (col, slot) in row_words.iter_mut().enumerate() {
                *slot = 0x8000 | ((row as u16) << 8) | col as u16;
            }
        }
        let plane_base = 0xE000usize;
        write_full_plane_rows(&mut vdp, plane_base, &rows);

        // Sample every edge plus an interior cell so any arithmetic mutation
        // on the byte address shifts at least one cell off the expected slot.
        for &(row, col) in &[(0usize, 0usize), (0, 63), (1, 0), (5, 7), (27, 0), (27, 63)] {
            let addr = plane_base + row * 128 + col * 2;
            let w = u16::from_be_bytes([vdp.vram[addr], vdp.vram[addr + 1]]);
            let expected = 0x8000 | ((row as u16) << 8) | col as u16;
            assert_eq!(
                w, expected,
                "plane row ({row},{col}) at byte ${addr:04X} must hold the row-major source word"
            );
        }
    }

    #[test]
    fn write_plane_a_40x28_pins_row_major_source_and_byte_target() {
        // ASM sub_1B850 (kid.asm:30193-30197) seeds d6=$28, d7=$1C and bsr.w
        // sub_1C5D0; sub_1C5D0 (kid.asm:31341-31363) reads the source linearly
        // with `move.w (a4)+,(a6)` for d6+1 = 40 words per row across d7+1 =
        // 28 rows, advancing the VRAM target by `addi.l #$800000` (128 bytes)
        // between rows.  The native helper mirrors that with `row * 40 + col`
        // for the source index and `row * NAMETABLE_ROW_STRIDE_BYTES + col *
        // 2` for the byte address.  Each cell is tagged with a unique value
        // so any mutation that swaps `*` for `+`/`/` or `+` for `-`/`*` on
        // either side lands the wrong source word at the right byte (or right
        // word at the wrong byte).
        let mut vdp = VdpState::new();
        let mut tilemap = vec![0u16; 40 * 28];
        for row in 0..28u16 {
            for col in 0..40u16 {
                tilemap[(row as usize) * 40 + (col as usize)] = 0x8000 | (row << 8) | col;
            }
        }
        write_plane_a_40x28(&mut vdp, &tilemap);

        for &(row, col) in &[(0u16, 0u16), (0, 39), (1, 0), (3, 5), (27, 0), (27, 39)] {
            let addr = (row as usize) * 128 + (col as usize) * 2;
            let w = u16::from_be_bytes([vdp.vram[addr], vdp.vram[addr + 1]]);
            let expected = 0x8000 | (row << 8) | col;
            assert_eq!(
                w, expected,
                "plane A cell ({row},{col}) at byte ${addr:04X} must hold the row-major source word"
            );
        }

        // sub_1C5D0 stops after d6+1 = 40 words per row, so column 40 must
        // remain 0 — pins the inner-loop bound and rules out a mutant that
        // turns `col < 40` into a permissive variant.
        let untouched = u16::from_be_bytes([vdp.vram[80], vdp.vram[81]]);
        assert_eq!(
            untouched, 0,
            "ASM sub_1C5D0 only writes 40 words per row; column 40 must stay 0"
        );
    }

    #[test]
    fn intro_sequence_selects_palette_by_segment() {
        let data = IntroSub1B850Data {
            palette_segment_0: vec![0x0111],
            palette_segment_1: vec![0x0222],
            palette_segment_2: vec![0x0333],
            ..empty_intro_data()
        };

        let vdp0 = build_intro_sequence_vdp(&data, 0);
        assert_eq!(
            u16::from_be_bytes([vdp0.cram[0], vdp0.cram[1]]),
            0x0111,
            "segment 0 uses intro1_wildside palette"
        );

        let vdp1 = build_intro_sequence_vdp(&data, 1);
        assert_eq!(
            u16::from_be_bytes([vdp1.cram[0], vdp1.cram[1]]),
            0x0222,
            "segment 1 uses intro2_sky palette"
        );

        let vdp2 = build_intro_sequence_vdp(&data, 2);
        assert_eq!(
            u16::from_be_bytes([vdp2.cram[0], vdp2.cram[1]]),
            0x0333,
            "segment 2+ uses intro3_alley palette"
        );
    }

    #[test]
    fn intro_sequence_2_4_and_5_follow_their_rom_vdp_loaders() {
        let mut intro2_hologram = vec![0x2114u16; 40 * 28];
        let base_word_index = (0x7C12u16 - 0x77B2u16) as usize / 2;
        intro2_hologram[base_word_index] = 0x1234;
        let data = IntroSub1B850Data {
            art_chunks: [
                (0x2740, vec![0xF1]),
                (0, vec![]),
                (0, vec![]),
                (0, vec![]),
                (0, vec![]),
            ],
            palette_segment_1: vec![0x0222],
            palette_segment_2: vec![0x0333],
            hologram_background_art: vec![0xA1],
            hologram_blocks_art: vec![0xB2],
            intro_text_letters_art: vec![0xC3],
            intro_sequence_2_hologram_tilemap: intro2_hologram,
            intro_sequence_4_hologram_tilemap: vec![0x20BC; 40 * 28],
            intro_sequence_2_tilemap: vec![0x00BC; 40 * 28],
            intro_sequence_5_common_art_chunks: [
                (0x1780, vec![0xD4]),
                (0x4D80, vec![0xE5]),
                (0x6800, vec![0xF6]),
            ],
            intro_sequence_5_helmet_art_variants: [vec![0xA0], vec![0xA7], vec![0xA2]],
            intro_sequence_5_tilemap: vec![0x0ABC; 40 * 28],
            ..empty_intro_data()
        };

        let intro2 = build_intro_sequence_vdp(&data, 1);
        assert_eq!(
            intro2.vram[0x2740], 0,
            "ASM Load_IntroSequence2 (kid.asm:29255) must not reuse the sub_1B850 Wild Side art upload at $2740",
        );
        assert_eq!(intro2.vram[0x2280], 0xA1);
        assert_eq!(intro2.vram[0x3540], 0xB2);
        assert_eq!(
            u16::from_be_bytes([intro2.vram[0], intro2.vram[1]]),
            0x00BC,
            "ASM Load_IntroSequence2 should write the intro2_sky tilemap to plane A",
        );
        assert_eq!(
            u16::from_be_bytes([intro2.vram[0xE700], intro2.vram[0xE701]]),
            0x21AB,
            "ASM sub_1B7B6 should patch the hologram tilemap before sub_1CD88 uploads it",
        );

        let intro4 = build_intro_sequence_vdp(&data, 3);
        assert_eq!(
            intro4.vram[0x1780], 0xA1,
            "ASM Load_IntroSequence4 (kid.asm:29303) should load HoloBG at $1780",
        );
        assert_eq!(
            u16::from_be_bytes([intro4.vram[0xE000], intro4.vram[0xE001]]),
            0x20BC,
            "ASM Load_IntroSequence4 should upload the HoloBG tilemap through sub_1CD88",
        );

        let intro5 = build_intro_sequence_vdp(&data, 4);
        assert_eq!(
            intro5.vram[0x2740], 0,
            "ASM Load_IntroSequence5 (kid.asm:29324) must not reuse sub_1B850's background art at $2740",
        );
        assert_eq!(intro5.vram[0x1780], 0xD4);
        assert_eq!(intro5.vram[0x4D80], 0xE5);
        assert_eq!(intro5.vram[0x6800], 0xF6);
        assert_eq!(intro5.vram[0xBD60], 0xA7);
        assert_eq!(
            u16::from_be_bytes([intro5.vram[0xE000], intro5.vram[0xE001]]),
            0x0ABC,
            "ASM Load_IntroSequence5 should upload MapEni_2E154 through sub_1CD88",
        );
    }

    #[test]
    fn intro_sequence_5_selects_art_and_palette_overlay_from_fc82() {
        let data = IntroSub1B850Data {
            palette_segment_2: vec![0x1000; 0x40],
            intro_sequence_5_helmet_art_variants: [vec![0xD9], vec![0xC1], vec![0xB7]],
            intro_sequence_5_palette_variant_1_overlay: vec![0x1C36; 0x0E],
            intro_sequence_5_palette_variant_2_overlay: vec![0x1C34; 0x0E],
            ..empty_intro_data()
        };

        let default_region = build_intro_sequence_vdp_with_title_variant(&data, 4, 0);
        assert_eq!(
            default_region.vram[0xBD60], 0xD9,
            "ASM loc_1B100 keeps byte_213D9 when ($FFFFFC82).w is zero",
        );
        assert_eq!(
            u16::from_be_bytes([default_region.cram[0x62], default_region.cram[0x63]]),
            0x1000,
            "ASM sub_1C512 should not overlay Palette_Buffer_2+$62 for FC82==0",
        );

        let variant_1 = build_intro_sequence_vdp_with_title_variant(&data, 4, 1);
        assert_eq!(
            variant_1.vram[0xBD60], 0xC1,
            "ASM loc_1B100 selects byte_219C1 when ($FFFFFC82).w equals 1",
        );
        assert_eq!(
            u16::from_be_bytes([variant_1.cram[0x62], variant_1.cram[0x63]]),
            0x1C36,
            "ASM sub_1C512 overlays Pal_1C3C6 at Palette_Buffer_2+$62 for FC82==1",
        );

        let variant_2 = build_intro_sequence_vdp_with_title_variant(&data, 4, 2);
        assert_eq!(
            variant_2.vram[0xBD60], 0xB7,
            "ASM loc_1B100 selects byte_216B7 when ($FFFFFC82).w is non-zero and not 1",
        );
        assert_eq!(
            u16::from_be_bytes([variant_2.cram[0x62], variant_2.cram[0x63]]),
            0x1C34,
            "ASM sub_1C512 overlays Pal_1C3D4 at Palette_Buffer_2+$62 for other non-zero FC82 values",
        );
    }

    #[test]
    fn intro_sequence_5_loc_1bc36_streams_sub_1c034_frames_on_rom_hibernate_boundaries() {
        let frame_tilemaps = (0..8u16)
            .map(|frame_index| {
                let plan = kid_data::intro_sequence_5_frame_load_plan_sub_1C034(frame_index)
                    .expect("ASM sub_1C034 should accept all eight frame indices");
                let word = 0x1100 | frame_index;
                vec![
                    word;
                    usize::from(plan.tilemap_load.width_words)
                        * usize::from(plan.tilemap_load.height_words)
                ]
            })
            .collect::<Vec<_>>();
        let data = IntroSub1B850Data {
            intro_sequence_5_frame_tilemaps: frame_tilemaps,
            ..empty_intro_data()
        };

        let first = build_intro_sequence_vdp_with_title_variant_at_elapsed(&data, 4, 0, 0, 0);
        assert_eq!(
            vram_word(&first, 0xA346),
            0x1100,
            "ASM loc_1BC36 calls sub_1C034(d4=0) before the first `$A0` hibernate; left padding is three words"
        );

        let before_second =
            build_intro_sequence_vdp_with_title_variant_at_elapsed(&data, 4, 0, 0xA0 - 1, 0);
        assert_eq!(
            vram_word(&before_second, 0xA346),
            0x1100,
            "ASM loc_1BC36 should keep frame 0 visible through the `$A0` hibernate"
        );

        let second = build_intro_sequence_vdp_with_title_variant_at_elapsed(&data, 4, 0, 0xA0, 0);
        assert_eq!(
            vram_word(&second, 0xA346),
            0x1101,
            "ASM loc_1BC36 resumes with sub_1C034(d4=1) exactly after the `$A0` hibernate"
        );

        let centred = build_intro_sequence_vdp_with_title_variant_at_elapsed(
            &data,
            4,
            0,
            0xA0 + 0x90 + 0x16,
            0,
        );
        assert_eq!(
            vram_word(&centred, 0xA340),
            0x1104,
            "ASM off_1C126 frame 4 has zero left padding, so loc_1C0A4 starts writing at `$A340`"
        );
    }

    #[test]
    fn intro_sequence_5_loc_1bc36_panel_blit_and_blank_follow_sub_1c0ee_order() {
        let data = IntroSub1B850Data {
            intro_sequence_5_frame_tilemaps: vec![vec![0x1111; 12 * 24]; 8],
            intro_sequence_5_panel_tilemap: vec![0xCAFE; 10 * 10],
            ..empty_intro_data()
        };
        let panel_byte_addr = 0x13 * NAMETABLE_ROW_STRIDE_BYTES + 3 * 2;

        let before_panel = build_intro_sequence_vdp_with_title_variant_at_elapsed(
            &data,
            4,
            0,
            0xA0 + 0x90 + 0x15,
            0,
        );
        assert_ne!(
            vram_word(&before_panel, panel_byte_addr),
            0xCAFE,
            "ASM loc_1BC36 does not call sub_1C0EE until after sub_1C034(d4=4)"
        );

        let panel = build_intro_sequence_vdp_with_title_variant_at_elapsed(
            &data,
            4,
            0,
            0xA0 + 0x90 + 0x16,
            0,
        );
        assert_eq!(
            vram_word(&panel, panel_byte_addr),
            0xCAFE,
            "ASM loc_1BC36 calls sub_1C0EE(d4=3,d5=$13) immediately after sub_1C034(d4=4)"
        );

        let cleared = build_intro_sequence_vdp_with_title_variant_at_elapsed(
            &data,
            4,
            0,
            0xA0 + 0x90 + 0x21,
            0,
        );
        assert_eq!(
            vram_word(&cleared, panel_byte_addr),
            0,
            "ASM loc_1BC36 calls sub_1C112 after the next `$B` hibernate, blanking the panel before frame 5"
        );
    }

    #[test]
    fn title_screen_tilemap_at_cell_21_1() {
        // ASM sub_1C5D0 (kid.asm:31341-31363): driven by d4=$15, d5=1, d6=$12,
        // d7=$14 after sub_1B2A4 returns, the routine reads the source pointer
        // linearly with `move.w (a4)+,(a6)` for d6+1 = 18 words per row while
        // advancing the VRAM target by `addi.l #$800000` (128 bytes = one
        // nametable row) for d7+1 = 20 rows; the per-cell VRAM byte address is
        // therefore `(cell_y + row)*128 + (cell_x + col)*2` with the source
        // word index laid out row-major as `row*tilemap_width + col`.
        //
        // Each cell carries a unique value `(row << 8) | col | 0x8000` so the
        // assertions pin both the row-major source index AND the row*128 +
        // col*2 VRAM target: any mutant that swaps `row * tilemap_width` for
        // `row + tilemap_width` / `row / tilemap_width`, perturbs `cell_x +
        // col`, or otherwise lands the wrong source word at the right byte
        // address would copy a different unique value into VRAM.
        let mut tilemap = vec![0u16; 18 * 20];
        for row in 0..20u16 {
            for col in 0..18u16 {
                tilemap[(row as usize) * 18 + (col as usize)] = 0x8000 | (row << 8) | col;
            }
        }
        let data = TitleScreenData {
            art_chunks: [(0, vec![]), (0, vec![]), (0, vec![]), (0, vec![])],
            title_art_chunks: [(0, vec![]), (0, vec![]), (0, vec![])],
            plane_a_tilemap: tilemap,
            title_text_chameleon_rows: [[0; 64]; 28],
            title_menu_tilemap_words: vec![],
            character_frames: Vec::new(),
            tile_patch_words: [0; 32],
            palette_words: vec![],
        };
        let vdp = build_title_screen_vdp_with_character_frame(&data, 0, 0);

        // Sample row-major source-vs-target pairs across every edge so any
        // arithmetic mutation on `(cell_y + row) * 128 + (cell_x + col) * 2`
        // or on `row * tilemap_width + col` produces a divergence on at least
        // one cell — corners alone cannot distinguish `*` from `+`/`/` when
        // the right-hand operand is 0 on both edges.
        for &(row, col) in &[
            (0u16, 0u16), // (cell_y=1, cell_x=21) → byte 128 + 42 = 170
            (0, 17),      // top-right edge — pins col*2 / cell_x+col
            (1, 0),       // exercises a non-zero row at col=0
            (3, 5),       // interior cell
            (19, 0),      // bottom-left — pins (cell_y + row) * 128
            (19, 17),     // bottom-right corner
        ] {
            let addr = (1 + row as usize) * 128 + (0x15 + col as usize) * 2;
            let w = u16::from_be_bytes([vdp.vram[addr], vdp.vram[addr + 1]]);
            let expected = 0x8000 | (row << 8) | col;
            assert_eq!(
                w, expected,
                "title tilemap cell (row={row}, col={col}) at byte ${addr:04X}"
            );
        }

        // ASM sub_1C5D0 only writes `d6+1 = 18` words per row, so the byte
        // immediately after the last written column on row 0 of the title
        // window must retain Load_TitleScreen loc_1B258's `$20BC` fill — pins
        // the inner-loop bound and rules out a mutant that turns
        // `col < tilemap_width` into a permissive variant.
        let untouched = u16::from_be_bytes([
            vdp.vram[128 + (0x15 + 18) * 2],
            vdp.vram[128 + (0x15 + 18) * 2 + 1],
        ]);
        assert_eq!(
            untouched, 0x20BC,
            "ASM sub_1C5D0 stops after column $11 (= tilemap_width-1); column 18 must keep the title fill word"
        );
    }

    #[test]
    fn apply_title_text_word_writes_sub_1b7b6_stamps_2x2_pattern_at_every_table_entry() {
        // ASM sub_1B7B6 (kid.asm:30035-30053): for each base RAM address in
        // `unk_1B7E0`, the routine writes
        //   move.w #$21AB,(a1)        ; top-left  (word offset +0)
        //   move.w #$21AC,2(a1)       ; top-right (word offset +1)
        //   move.w #$21AD,$50(a1)     ; bot-left  (word offset +$28)
        //   move.w #$21AE,$52(a1)     ; bot-right (word offset +$29)
        // The native code subtracts $77B2 from the table address to convert
        // the absolute RAM address into a word index inside the decompression
        // buffer.  This buffer is large enough to hold every write so the
        // top/bottom bound checks all evaluate true; the test then verifies
        // that the four words landed at the exact expected offsets for each
        // of the 55 table entries (and only those offsets).
        const DECOMPRESSION_BUFFER_LOW_ADDR: u16 = 0x77B2;
        let mut decoded_words = vec![0u16; 0x500];

        apply_title_text_word_writes_sub_1b7b6(&mut decoded_words);

        let entries = title_text_word_writes_sub_1B7B6();
        assert_eq!(entries.len(), 55);
        let mut expected_writes = std::collections::HashMap::new();
        for entry in &entries {
            let word_index = usize::from(
                entry
                    .base_word_address
                    .wrapping_sub(DECOMPRESSION_BUFFER_LOW_ADDR),
            ) / 2;
            expected_writes.insert(word_index, 0x21ABu16);
            expected_writes.insert(word_index + 1, 0x21AC);
            expected_writes.insert(word_index + 0x28, 0x21AD);
            expected_writes.insert(word_index + 0x29, 0x21AE);
        }

        for (idx, expected) in &expected_writes {
            assert_eq!(
                decoded_words[*idx], *expected,
                "ASM sub_1B7B6 must stamp ${expected:04X} at word index ${idx:04X}",
            );
        }
        for (idx, value) in decoded_words.iter().enumerate() {
            if expected_writes.contains_key(&idx) {
                continue;
            }
            assert_eq!(
                *value, 0,
                "word index ${idx:04X} is not in sub_1B7B6's table; native must not touch it",
            );
        }

        // First entry: ASM table starts at $7C12 → word index $230.
        assert_eq!(decoded_words[0x230], 0x21AB);
        assert_eq!(decoded_words[0x231], 0x21AC);
        assert_eq!(decoded_words[0x230 + 0x28], 0x21AD);
        assert_eq!(decoded_words[0x230 + 0x29], 0x21AE);

        // Last entry: ASM table ends at $7FEE before the $FFFF sentinel →
        // word index $41E.
        assert_eq!(decoded_words[0x41E], 0x21AB);
        assert_eq!(decoded_words[0x41F], 0x21AC);
        assert_eq!(decoded_words[0x41E + 0x28], 0x21AD);
        assert_eq!(decoded_words[0x41E + 0x29], 0x21AE);
    }

    #[test]
    fn apply_title_text_word_writes_sub_1b7b6_top_bound_skips_when_word_index_plus_one_equals_len()
    {
        // ASM sub_1B7B6: the first table entry is base $7C12 → word index $230
        // inside the decompression buffer (low address $77B2).  Sized at exactly
        // $231 words, the buffer can hold word index $230 but not $231, so the
        // bound `word_index + 1 < len` must reject the entry; any mutant that
        // loosens the comparison to `<=` (or perturbs `+ 1` to `* 1`/`- 1`)
        // turns the skipped write into an out-of-bounds index that panics.
        let mut decoded_words = vec![0u16; 0x231];

        apply_title_text_word_writes_sub_1b7b6(&mut decoded_words);

        // Original: every entry has word_index >= $230, so every top check
        // falls through and the buffer remains untouched.
        assert!(
            decoded_words.iter().all(|&w| w == 0),
            "sub_1B7B6 must not write when no entry's word_index + 1 fits in the buffer",
        );
    }

    #[test]
    fn apply_title_text_word_writes_sub_1b7b6_bottom_bound_skips_when_bottom_plus_one_equals_len() {
        // ASM sub_1B7B6: the bottom row is at offset $50 bytes = $28 words from
        // the top row, plus one more word for $52(a1).  With the buffer sized
        // at $259 words, the first table entry (word index $230) fits its top
        // pair ($230,$231) but the bottom pair ($258,$259) ends exactly at
        // len, so the bound `bottom + 1 < len` must reject the bottom write;
        // any mutant that loosens the check to `<=`/`==` (or perturbs `+ 1`
        // to `* 1`/`- 1`) turns the skipped write into an out-of-bounds
        // index that panics.
        let mut decoded_words = vec![0u16; 0x259];

        apply_title_text_word_writes_sub_1b7b6(&mut decoded_words);

        // First five entries (word indices $230..$240) all have their top
        // pair inside the buffer and their bottom pair off the end — top
        // writes must land, bottom slots must remain zero.
        for &word_index in &[0x230usize, 0x234, 0x238, 0x23C, 0x240] {
            assert_eq!(
                decoded_words[word_index], 0x21AB,
                "sub_1B7B6 must stamp top-left $21AB at word index ${word_index:04X}",
            );
            assert_eq!(
                decoded_words[word_index + 1],
                0x21AC,
                "sub_1B7B6 must stamp top-right $21AC at word index ${:04X}",
                word_index + 1,
            );
        }
        // Bottom of the first entry is exactly out of bounds — must stay 0.
        assert_eq!(decoded_words[0x258], 0);
    }

    #[test]
    fn title_screen_tile_patch_at_f000() {
        let mut patch = [0u16; 32];
        patch[0] = 0x1111;
        patch[31] = 0x2222;
        let data = TitleScreenData {
            art_chunks: [(0, vec![]), (0, vec![]), (0, vec![]), (0, vec![])],
            title_art_chunks: [(0, vec![]), (0, vec![]), (0, vec![])],
            plane_a_tilemap: vec![],
            title_text_chameleon_rows: [[0; 64]; 28],
            title_menu_tilemap_words: vec![],
            character_frames: Vec::new(),
            tile_patch_words: patch,
            palette_words: vec![],
        };
        let vdp = build_title_screen_vdp_with_character_frame(&data, 0, 0);

        let w0 = u16::from_be_bytes([vdp.vram[0xF000], vdp.vram[0xF001]]);
        assert_eq!(w0, 0x1111, "tile patch first word at $F000");
        let w31 = u16::from_be_bytes([vdp.vram[0xF000 + 62], vdp.vram[0xF000 + 63]]);
        assert_eq!(w31, 0x2222, "tile patch last word at $F03E");
    }

    #[test]
    fn title_screen_vdp_load_title_art_uploads_menu_chunks() {
        let data = TitleScreenData {
            art_chunks: [(0, vec![]), (0, vec![]), (0, vec![]), (0, vec![])],
            title_art_chunks: [
                (0x9A60, vec![0x11, 0x22]),
                (0xA360, vec![0x33, 0x44]),
                (0xA980, vec![0x55, 0x66]),
            ],
            plane_a_tilemap: vec![],
            title_text_chameleon_rows: [[0; 64]; 28],
            title_menu_tilemap_words: vec![],
            character_frames: Vec::new(),
            tile_patch_words: [0; 32],
            palette_words: vec![],
        };

        let vdp = build_title_screen_vdp_with_character_frame(&data, 0, 0);

        assert_eq!(
            &vdp.vram[0x9A60..0x9A62],
            &[0x11, 0x22],
            "ASM Load_TitleArt uploads byte_24985 / Title_text_KID to VRAM $9A60"
        );
        assert_eq!(
            &vdp.vram[0xA360..0xA362],
            &[0x33, 0x44],
            "ASM Load_TitleArt uploads byte_26036 / Title_menu to VRAM $A360"
        );
        assert_eq!(
            &vdp.vram[0xA980..0xA982],
            &[0x55, 0x66],
            "ASM Load_TitleArt uploads byte_2A479 / Title_sparkles to VRAM $A980"
        );
    }

    #[test]
    #[allow(non_snake_case)]
    fn title_screen_vdp_obj_title_menu_loc_1bb52_writes_menu_and_cursor() {
        let mut menu_words = vec![0u16; 14 * 6];
        for row in 0..6u16 {
            for col in 0..14u16 {
                menu_words[(row as usize) * 14 + (col as usize)] = 0x4500 | (row << 4) | col;
            }
        }
        let data = TitleScreenData {
            art_chunks: [(0, vec![]), (0, vec![]), (0, vec![]), (0, vec![])],
            title_art_chunks: [(0, vec![]), (0, vec![]), (0, vec![])],
            plane_a_tilemap: vec![],
            title_text_chameleon_rows: [[0; 64]; 28],
            title_menu_tilemap_words: menu_words,
            character_frames: Vec::new(),
            tile_patch_words: [0; 32],
            palette_words: vec![],
        };

        let vdp = build_title_screen_vdp_with_character_frame_and_menu(&data, 0, 0, Some(2));

        for &(row, col) in &[(0u16, 0u16), (0, 13), (5, 0), (5, 13), (3, 7)] {
            let addr = (4 + row as usize) * 128 + (4 + col as usize) * 2;
            let word = u16::from_be_bytes([vdp.vram[addr], vdp.vram[addr + 1]]);
            assert_eq!(
                word,
                0x4500 | (row << 4) | col,
                "ASM Obj_TitleMenu loc_1BB52 writes menu cell row {row}, col {col}"
            );
        }

        assert_eq!(
            sprite_entry_words(&vdp, 0),
            (0x00BC, 0x00, 0, 0x454B, 0x00A8),
            "ASM Obj_TitleMenu loc_1BB52/loc_1BC0E should expose cursor 2 as unk_E1196 at x=$28,y=$44"
        );
    }

    #[test]
    fn title_screen_vdp_sub_1b532_writes_chameleon_plane_b_rows_after_clear() {
        let mut rows = [[0u16; 64]; 28];
        for row in 0..28u16 {
            for col in 0..64u16 {
                rows[row as usize][col as usize] = 0x0300 | (row << 6) | col;
            }
        }
        let data = TitleScreenData {
            // ASM sub_1B532 loc_1B53E clears $1780 after earlier art uploads.
            art_chunks: [
                (0x1780, vec![0xAA; 32]),
                (0, vec![]),
                (0, vec![]),
                (0, vec![]),
            ],
            title_art_chunks: [(0, vec![]), (0, vec![]), (0, vec![])],
            plane_a_tilemap: vec![],
            title_text_chameleon_rows: rows,
            title_menu_tilemap_words: vec![],
            character_frames: Vec::new(),
            tile_patch_words: [0; 32],
            palette_words: vec![],
        };

        let vdp = build_title_screen_vdp_with_character_frame(&data, 0, 0);

        for byte_addr in 0x1780..0x17A0 {
            assert_eq!(
                vdp.vram[byte_addr], 0,
                "ASM sub_1B532 clears 8 longwords at VRAM $1780 before writing plane B"
            );
        }

        for &(row, col) in &[(0usize, 0usize), (0, 63), (9, 17), (27, 0), (27, 63)] {
            let addr = 0xE000 + row * NAMETABLE_ROW_STRIDE_BYTES + col * 2;
            let word = u16::from_be_bytes([vdp.vram[addr], vdp.vram[addr + 1]]);
            assert_eq!(
                word, rows[row][col],
                "ASM sub_1B532 loc_1B566 copies row {row}, col {col} to plane B"
            );
        }
    }

    #[test]
    fn title_screen_vdp_sub_1bf24_fills_narrow_character_frame_edges() {
        let mut frame_words = vec![0u16; 16 * 20];
        for row in 0..20u16 {
            for col in 0..16u16 {
                frame_words[(row as usize) * 16 + col as usize] = 0x9000 | (row << 8) | col;
            }
        }
        let data = TitleScreenData {
            art_chunks: [(0, vec![]), (0, vec![]), (0, vec![]), (0, vec![])],
            title_art_chunks: [(0, vec![]), (0, vec![]), (0, vec![])],
            plane_a_tilemap: vec![],
            title_text_chameleon_rows: [[0; 64]; 28],
            title_menu_tilemap_words: vec![],
            character_frames: vec![TitleScreenCharacterFrameData {
                frame_index: 7,
                region_byte: 2,
                width_words: 16,
                tilemap_words: frame_words,
            }],
            tile_patch_words: [0; 32],
            palette_words: vec![],
        };

        let vdp = build_title_screen_vdp_with_character_frame(&data, 7, 2);

        let row_3_col_15_addr = (1 + 3) * 128 + (0x15 + 15) * 2;
        let row_3_col_15 =
            u16::from_be_bytes([vdp.vram[row_3_col_15_addr], vdp.vram[row_3_col_15_addr + 1]]);
        assert_eq!(
            row_3_col_15, 0x930F,
            "ASM sub_1BF24 copies the selected frame width into the staging rectangle"
        );

        let row_3_col_16_addr = (1 + 3) * 128 + (0x15 + 16) * 2;
        let row_3_col_16 =
            u16::from_be_bytes([vdp.vram[row_3_col_16_addr], vdp.vram[row_3_col_16_addr + 1]]);
        assert_eq!(
            row_3_col_16, 0x20BC,
            "ASM sub_1BF24 keeps columns beyond the frame width filled with `$20BC`"
        );
    }

    #[test]
    fn title_card_vdp_centres_decoded_map_with_02fb_padding() {
        let mut tilemap = Vec::new();
        for row in 0..2u16 {
            for col in 0..4u16 {
                tilemap.push(0x8000 | (row << 8) | col);
            }
        }
        let mut themes = vec![None; 11];
        themes[9] = Some(TitleCardThemeData {
            art_bytes: vec![0x12, 0x34, 0x56],
            tilemap_words: tilemap,
            palette_words: vec![0x0000, 0x0EEE],
            width_tiles: 4,
            height_tiles: 2,
        });
        let data = TitleCardData { themes };

        let vdp = build_title_card_vdp(&data, 9).expect("forest title-card VDP");
        let word_at = |addr: usize| u16::from_be_bytes([vdp.vram[addr], vdp.vram[addr + 1]]);

        // ASM loc_19BC2 emits 13 full 64-word padding rows for a 2-tile-high card.
        assert_eq!(word_at(12 * 64 * 2), 0x02FB);
        // ASM loc_19BD4 emits `(0x28 - width) / 2 = 18` left-padding words.
        let first_art_word = (13 * 64 + 18) * 2;
        assert_eq!(word_at(first_art_word - 2), 0x02FB);
        // ASM loc_19BDE copies decoded map words row-major.
        assert_eq!(word_at(first_art_word), 0x8000);
        assert_eq!(word_at(first_art_word + 3 * 2), 0x8003);
        // ASM loc_19BEE pads the right side of the same 64-word row.
        assert_eq!(word_at(first_art_word + 4 * 2), 0x02FB);
        // Next art row lands exactly one 64-word plane row later.
        assert_eq!(word_at(first_art_word + 64 * 2), 0x8100);
        // ASM loc_19C04 pads the bottom rows after the copied art.
        assert_eq!(word_at((15 * 64) * 2), 0x02FB);
    }

    #[test]
    fn title_card_vdp_uploads_art_palette_and_load_title_card_registers() {
        let mut themes = vec![None; 11];
        themes[1] = Some(TitleCardThemeData {
            art_bytes: vec![0xAA, 0xBB, 0xCC, 0xDD],
            tilemap_words: vec![],
            palette_words: vec![0x0000, 0x0222, 0x0444],
            width_tiles: 0,
            height_tiles: 0,
        });
        let data = TitleCardData { themes };

        let vdp = build_title_card_vdp(&data, 1).expect("sky title-card VDP");

        assert_eq!(&vdp.vram[0x5F60..0x5F64], &[0xAA, 0xBB, 0xCC, 0xDD]);
        assert_eq!(u16::from_be_bytes([vdp.cram[2], vdp.cram[3]]), 0x0222);
        assert_eq!(
            vdp.registers[2], 0x00,
            "ASM Load_TitleCard inherits VDPGameInitValues Scroll A base $0000 before writing the card map to VRAM $0000",
        );
        assert_eq!(
            vdp.registers[4], 0x07,
            "ASM Load_TitleCard inherits VDPGameInitValues Scroll B base $E000; loc_19B58 fills that plane with $0094",
        );
        assert_eq!(
            u16::from_be_bytes([vdp.vram[0xE000], vdp.vram[0xE001]]),
            0x0094,
            "ASM loc_19B58 fills title-card tilemap work area with $0094",
        );
    }

    #[test]
    fn load_palette_words_decodes_big_endian_pairs_from_disk() {
        // ASM Palette_Buffer / Pal_1C28A / Pal_1C2F0 / Pal_1C346 (kid.asm:31232,
        // 31247, 31262) all hold raw 9-bit BGR palette words written through
        // `move.w (a0)+,(a1)+` — the on-disk layout extracted by the asset
        // pipeline is therefore big-endian u16 word pairs. load_palette_words
        // mirrors that with `chunks_exact(2).map(u16::from_be_bytes)`.
        //
        // Driving the loader with a known byte sequence (and asserting on both
        // length AND value) pins all three Result<Vec<u16>>-replacement mutants
        // cargo-mutants generates for the wrapper: `Ok(vec![])` shrinks the
        // length, `Ok(vec![0])` and `Ok(vec![1])` change both the length and
        // the first word's value.
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("system time should be after unix epoch")
            .as_nanos();
        let path =
            std::env::temp_dir().join(format!("kc-vdp-palette-{}-{nanos}.bin", std::process::id()));
        let bytes: [u8; 6] = [0x0E, 0xEE, 0x04, 0x44, 0x00, 0x00];
        std::fs::write(&path, bytes).expect("write temp palette");

        let words = load_palette_words(&path).expect("load palette words");
        let _ = std::fs::remove_file(&path);

        assert_eq!(words.len(), 3, "three big-endian word pairs from 6 bytes");
        assert_eq!(words[0], 0x0EEE, "first word is BE pair of bytes 0..2");
        assert_eq!(words[1], 0x0444, "second word is BE pair of bytes 2..4");
        assert_eq!(words[2], 0x0000, "third word is BE pair of bytes 4..6");
    }

    #[test]
    fn load_saxman_art_decompresses_known_fixture_from_disk() {
        // ASM sub_1B850 (kid.asm:30172) and Load_IntroSequence{2,4,5} all upload
        // Saxman-compressed art via `jsr (j_SaxDec)` after seeding `a0` with the
        // ROM offset. load_saxman_art is the I/O wrapper around that decode and
        // must return the actual decoded bytes — `Ok(vec![])`, `Ok(vec![0])`,
        // and `Ok(vec![1])` mutants would all corrupt subsequent VRAM uploads.
        //
        // Saxman framing matches kid_data::SAXMAN_FIXTURE: a 2-byte little-
        // endian compressed length, a control byte selecting eight literal
        // slots, the four ASCII bytes "ABCD", and a trailing zero pair.
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("system time should be after unix epoch")
            .as_nanos();
        let path =
            std::env::temp_dir().join(format!("kc-vdp-saxman-{}-{nanos}.bin", std::process::id()));
        let compressed: [u8; 10] = [0x00, 0x02, 0xF4, 0x60, 0x41, 0x42, 0x43, 0x44, 0x00, 0x00];
        std::fs::write(&path, compressed).expect("write temp saxman blob");

        let decoded = load_saxman_art(&path).expect("decompress saxman art");
        let _ = std::fs::remove_file(&path);

        assert_eq!(decoded.len(), 4, "fixture decodes to four literal bytes");
        assert_eq!(decoded, b"ABCD", "fixture decodes to ASCII 'ABCD'");
    }
}
