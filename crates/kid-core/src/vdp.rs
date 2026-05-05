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

use crate::boot::{VDP_GAME_INIT_VALUES, title_text_word_writes_sub_1B7B6};

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
    /// Each word is 2 bytes in Mega Drive 0BGR format (3 bits per channel).
    fn write_cram_words(&mut self, word_offset: usize, words: &[u16]) {
        for (i, &word) in words.iter().enumerate() {
            let byte_offset = (word_offset + i) * 2;
            if byte_offset + 1 < self.cram.len() {
                self.cram[byte_offset] = (word >> 8) as u8;
                self.cram[byte_offset + 1] = word as u8;
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
    /// Load_IntroSequence5 art uploads in ROM order.
    intro_sequence_5_art_chunks: [(usize, Vec<u8>); 4],
    /// IntroSequence5 tilemap (`MapEni_2E154`) decoded with base `$00BC`.
    intro_sequence_5_tilemap: Vec<u16>,
}

/// Load and cache intro sub_1B850 assets from the asset bundle.
///
/// ASM sub_1B850 (kid.asm:30172): decompress 5 art pieces to VRAM, decode
/// `unk_2FDE0` as 40×28 tilemap to plane A, decode `unk_2FDCE` and stamp
/// each of 28 words across full 64-word rows to plane B at $E000.
pub(crate) fn load_intro_sub_1b850_data(manifest: &AssetManifest) -> Result<IntroSub1B850Data> {
    let scenes = manifest.output_root.join("scenes");

    fn load_saxman_art(scenes: &std::path::Path, filename: &str) -> Result<Vec<u8>> {
        let path = scenes.join(filename);
        let compressed = std::fs::read(&path)
            .with_context(|| format!("failed to read intro art: {}", path.display()))?;
        kid_data::decode_saxman(&compressed)
            .with_context(|| format!("failed to decompress intro art: {filename}"))
    }

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
        let decompressed = load_saxman_art(&scenes, filename)?;
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

    let hologram_background_art = load_saxman_art(&scenes, "artcomp/Hologram_background.bin")?;
    let hologram_blocks_art = load_saxman_art(&scenes, "artcomp/Hologram_blocks.bin")?;
    let intro_text_letters_art = load_saxman_art(&scenes, "artcomp/Intro_text_letters.bin")?;

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

    let intro_sequence_5_art_files: [(u16, &str); 4] = [
        (0x1780, "artcomp/Dark_alley_from_intro.bin"),
        (0x4D80, "artcomp/P_block_from_intro.bin"),
        (0x6800, "artcomp/The_kid_from_intro.bin"),
        // ASM Load_IntroSequence5 starts with sub_1C278, so the cold-boot
        // region byte advances from 0 to 1 before loc_1B100 selects byte_219C1.
        (0xBD60, "artcomp/Maniaxe_helmet_from_intro.bin"),
    ];
    let mut intro_sequence_5_art_chunks = Vec::new();
    for (vram_addr, filename) in &intro_sequence_5_art_files {
        intro_sequence_5_art_chunks
            .push((*vram_addr as usize, load_saxman_art(&scenes, filename)?));
    }
    let intro_sequence_5_art_chunks: [(usize, Vec<u8>); 4] = intro_sequence_5_art_chunks
        .try_into()
        .map_err(|_| anyhow::anyhow!("expected 4 intro sequence 5 art chunks"))?;
    let intro_sequence_5_tilemap_raw = std::fs::read(scenes.join("mapeni/intro3_alley.bin"))
        .context("failed to read intro sequence 5 tilemap")?;
    let intro_sequence_5_tilemap = kid_data::decode_enigma(&intro_sequence_5_tilemap_raw, 0x00BC)
        .context("failed to decode intro sequence 5 tilemap")?;

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
        intro_sequence_5_art_chunks,
        intro_sequence_5_tilemap,
    })
}

/// Build VdpState for an intro sequence segment (0–4).
///
/// Segments 0 and 2 use sub_1B850 art/tilemaps with segment-specific palettes.
/// Segments 1, 3, and 4 follow the separate Load_IntroSequence2/4/5 loaders.
pub(crate) fn build_intro_sequence_vdp(data: &IntroSub1B850Data, segment: usize) -> VdpState {
    match segment {
        1 => build_intro_sequence_2_vdp(data),
        3 => build_intro_sequence_4_vdp(data),
        4 => build_intro_sequence_5_vdp(data),
        _ => build_intro_sub_1b850_vdp(data, segment),
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
    // ASM: segment 0 → Pal_1C28A (0x33 words), segment 1 → Pal_1C2F0 (0x23 words),
    // segments 2+ → Pal_1C346 (0x40 words).
    let palette = match segment {
        // ASM loc_1C4F6 (kid.asm:31232): `move.w (a0)+,(a1)+; dbf`
        // copies 0x33 words from Pal_1C28A into Palette_Buffer_2.
        0 => &data.palette_segment_0,
        // ASM loc_1C50A (kid.asm:31247): `move.w (a0)+,(a1)+; dbf`
        // copies 0x23 words from Pal_1C2F0 into Palette_Buffer_2.
        1 => &data.palette_segment_1,
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
    vdp.write_cram_words(0, &data.palette_segment_1);

    vdp
}

fn build_intro_sequence_5_vdp(data: &IntroSub1B850Data) -> VdpState {
    let mut vdp = intro_sequence_base_vdp();

    // ASM Load_IntroSequence5 (kid.asm:29324-29364): use the Dark Alley/P
    // block/Kid/helmet art set and MapEni_2E154, not the shared sub_1B850
    // Wild Side art path. Native follows the cold-boot post-sub_1C278
    // region-byte-1 art for this cached non-gameplay VDP state.
    for (vram_byte_addr, art_bytes) in &data.intro_sequence_5_art_chunks {
        vdp.write_vram(*vram_byte_addr, art_bytes);
    }
    if let Some(rows) = sub_1cd88_rows(&data.intro_sequence_5_tilemap) {
        write_full_plane_rows(&mut vdp, 0xE000, &rows);
    }
    vdp.write_cram_words(0, &data.palette_segment_2);

    vdp
}

// ─── Title screen ───────────────────────────────────────────────────────────

/// Cached decompressed assets for the title screen (sub_1B2A4).
#[derive(Debug, Clone)]
pub(crate) struct TitleScreenData {
    /// 4 decompressed Saxman art chunks in upload order.
    art_chunks: [(usize, Vec<u8>); 4],
    /// Decoded Enigma tilemap for plane A (written starting at word $20C1).
    plane_a_tilemap: Vec<u16>,
    /// 32-word tile patch for VRAM $F000.
    tile_patch_words: [u16; 32],
    /// Palette words from Pal_1C400 (title.bin — 0x33 words).
    palette_words: Vec<u16>,
}

/// Load and cache title screen assets from the asset bundle.
///
/// ASM sub_1B2A4 (kid.asm:29472): 4 art uploads, Enigma tilemap at $20C1,
/// 32-word tile patch at $F000, palette from Pal_1C400.
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

    // Decode Enigma tilemap: unk_2F7D0 with base tile $20C1.
    // ASM sub_1B2A4: `move.w #$20C1,d0` then `jsr (j_EniDec)`.
    // After decode, the caller (Load_IntroSequence6 / Load_TitleScreen) calls
    // sub_1C5D0 with d4=$15, d5=1, d6=$12, d7=$14 → 18×20 at cell (21,1).
    let tilemap_path = scenes.join("mapeni/2F7D0.bin");
    let tilemap_raw = std::fs::read(&tilemap_path)
        .with_context(|| format!("failed to read title tilemap: {}", tilemap_path.display()))?;
    let plane_a_tilemap =
        kid_data::decode_enigma(&tilemap_raw, 0x20C1).context("failed to decode title tilemap")?;

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
        plane_a_tilemap,
        tile_patch_words,
        palette_words,
    })
}

/// Build VdpState for the title screen / IntroSequence6.
///
/// ASM sub_1B2A4: 4 art uploads, Enigma tilemap to plane A at word $20C1,
/// 32-word tile patch to VRAM $F000, palette to CRAM.
pub(crate) fn build_title_screen_vdp(data: &TitleScreenData) -> VdpState {
    let mut vdp = VdpState::new();

    for (i, &value) in VDP_GAME_INIT_VALUES.iter().enumerate() {
        vdp.registers[i] = value;
    }
    vdp.registers[1] = 0x74;

    // Upload decompressed art to VRAM.
    for (vram_byte_addr, art_bytes) in &data.art_chunks {
        vdp.write_vram(*vram_byte_addr, art_bytes);
    }

    // Write Enigma tilemap to plane A via sub_1C5D0 at cell (21, 1), 18×20.
    // ASM: after sub_1B2A4, the caller does `moveq #$15,d4; moveq #1,d5;
    // moveq #$12,d6; moveq #$14,d7; bsr.w sub_1C5D0`.
    // sub_1C5D0 computes: byte_addr = row*128 + col*2 within the nametable.
    // Plane A nametable at $0000 (register 2 = $00).
    let tilemap_width: usize = 18;
    let tilemap_height: usize = 20;
    let cell_x: usize = 0x15; // 21
    let cell_y: usize = 1;
    let plane_a_base: usize = 0x0000;
    for row in 0..tilemap_height {
        for col in 0..tilemap_width {
            let word_index = row * tilemap_width + col;
            if word_index < data.plane_a_tilemap.len() {
                let byte_addr =
                    plane_a_base + (cell_y + row) * NAMETABLE_ROW_STRIDE_BYTES + (cell_x + col) * 2;
                if byte_addr + 1 < VRAM_SIZE {
                    vdp.write_vram_word(byte_addr, data.plane_a_tilemap[word_index]);
                }
            }
        }
    }

    // Write 32-word tile patch to VRAM $F000.
    // These are raw tile data words (not nametable entries).
    for (i, &word) in data.tile_patch_words.iter().enumerate() {
        vdp.write_vram_word(0xF000 + i * 2, word);
    }

    // Write palette to CRAM.
    vdp.write_cram_words(0, &data.palette_words);

    vdp
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
            intro_sequence_5_art_chunks: [(0, vec![]), (0, vec![]), (0, vec![]), (0, vec![])],
            intro_sequence_5_tilemap: vec![],
        }
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
            intro_sequence_5_art_chunks: [
                (0x1780, vec![0xD4]),
                (0x4D80, vec![0xE5]),
                (0x6800, vec![0xF6]),
                (0xBD60, vec![0xA7]),
            ],
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
    fn title_screen_tilemap_at_cell_21_1() {
        // ASM: after sub_1B2A4, sub_1C5D0 with d4=$15, d5=1, d6=$12, d7=$14
        // → 18×20 tilemap at cell (21,1) in plane A nametable at $0000.
        let tilemap = vec![0x5678u16; 18 * 20];
        let data = TitleScreenData {
            art_chunks: [(0, vec![]), (0, vec![]), (0, vec![]), (0, vec![])],
            plane_a_tilemap: tilemap,
            tile_patch_words: [0; 32],
            palette_words: vec![],
        };
        let vdp = build_title_screen_vdp(&data);

        // First tile at cell (21,1) → byte 128 + 42 = 170.
        let addr = 128 + 21 * 2;
        let w = u16::from_be_bytes([vdp.vram[addr], vdp.vram[addr + 1]]);
        assert_eq!(w, 0x5678, "title tilemap first tile at cell (21,1)");

        // Last tile at cell (38,20) → (1+19)*128 + (21+17)*2 = 2560 + 76 = 2636.
        let addr = 20 * 128 + 38 * 2;
        let w = u16::from_be_bytes([vdp.vram[addr], vdp.vram[addr + 1]]);
        assert_eq!(w, 0x5678, "title tilemap last tile at cell (38,20)");
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
            plane_a_tilemap: vec![],
            tile_patch_words: patch,
            palette_words: vec![],
        };
        let vdp = build_title_screen_vdp(&data);

        let w0 = u16::from_be_bytes([vdp.vram[0xF000], vdp.vram[0xF001]]);
        assert_eq!(w0, 0x1111, "tile patch first word at $F000");
        let w31 = u16::from_be_bytes([vdp.vram[0xF000 + 62], vdp.vram[0xF000 + 63]]);
        assert_eq!(w31, 0x2222, "tile patch last word at $F03E");
    }
}
