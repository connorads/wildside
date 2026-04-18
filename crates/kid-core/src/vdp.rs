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

use crate::boot::VDP_GAME_INIT_VALUES;

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
    if elapsed > SEGA_INITIAL_WAIT_FRAMES {
        let fade_elapsed = elapsed - SEGA_INITIAL_WAIT_FRAMES;
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
                let strip = &data.palette_words
                    [source_word_index..source_word_index + SEGA_FADE_STRIP_WORDS];
                vdp.write_cram_words(SEGA_FADE_STRIP_CRAM_WORD_OFFSET, strip);
            }
            palette_cursor -= 2;
        }
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

/// Cached decompressed assets for intro sequences that use sub_1B850
/// (segments 0 and 2 — IntroSequence1 and IntroSequence3).
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
        let path = scenes.join(filename);
        let compressed = std::fs::read(&path)
            .with_context(|| format!("failed to read intro art: {}", path.display()))?;
        let decompressed = kid_data::decode_saxman(&compressed)
            .with_context(|| format!("failed to decompress intro art: {filename}"))?;
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

    Ok(IntroSub1B850Data {
        art_chunks,
        plane_a_tilemap,
        plane_b_rows,
        palette_segment_0,
        palette_segment_1,
        palette_segment_2,
    })
}

/// Build VdpState for an intro sequence segment (0–4).
///
/// Segments 0 and 2 use sub_1B850 art/tilemaps with segment-specific palettes.
/// Segments 1 and 3 also reuse the same art layout (the hologram text overlays
/// are sprite-based and deferred). Segment 4 has its own setup but uses the
/// same base art for now.
pub(crate) fn build_intro_sequence_vdp(data: &IntroSub1B850Data, segment: usize) -> VdpState {
    let mut vdp = VdpState::new();

    // ASM VDPGameInitValues → register init, then reg 1 = $74 (display on).
    for (i, &value) in VDP_GAME_INIT_VALUES.iter().enumerate() {
        vdp.registers[i] = value;
    }
    vdp.registers[1] = 0x74;

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
        0 => &data.palette_segment_0,
        1 => &data.palette_segment_1,
        _ => &data.palette_segment_2,
    };
    vdp.write_cram_words(0, palette);

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
    let plan = kid_data::title_screen_load_plan_sub_1B2A4(0);
    let tile_patch_words = plan.tile_patch_words;

    // ASM loc_1C4BE (kid.asm:31202): after loc_1C4B2 seeds `d0=#$32`,
    // the inclusive DBF loop copies 0x33 words from Pal_1C400 into
    // Palette_Buffer; native loads those same title palette words for CRAM.
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
    fn write_cram_words_stores_big_endian() {
        let mut vdp = VdpState::new();
        vdp.write_cram_words(0, &[0x0EEE, 0x0444]);
        assert_eq!(vdp.cram[0], 0x0E);
        assert_eq!(vdp.cram[1], 0xEE);
        assert_eq!(vdp.cram[2], 0x04);
        assert_eq!(vdp.cram[3], 0x44);
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
    fn intro_sequence_plane_a_tilemap_at_nametable_origin() {
        // ASM sub_1B850: sub_1C5D0 called with d4=0, d5=0, d6=$28, d7=$1C
        // → 40×28 tilemap at cell (0,0) in the plane A nametable at $0000.
        let tilemap = vec![0x1234u16; 40 * 28];
        let data = IntroSub1B850Data {
            art_chunks: [
                (0, vec![]),
                (0, vec![]),
                (0, vec![]),
                (0, vec![]),
                (0, vec![]),
            ],
            plane_a_tilemap: tilemap,
            plane_b_rows: [[0; 64]; 28],
            palette_segment_0: vec![],
            palette_segment_1: vec![],
            palette_segment_2: vec![],
        };
        let vdp = build_intro_sequence_vdp(&data, 0);

        // First tile at (0,0) → byte address $0000.
        let w = u16::from_be_bytes([vdp.vram[0], vdp.vram[1]]);
        assert_eq!(w, 0x1234, "plane A first tile at (0,0)");

        // Last tile at (39,27) → byte $0000 + 27*128 + 39*2 = 3456 + 78 = 3534.
        let addr = 27 * 128 + 39 * 2;
        let w = u16::from_be_bytes([vdp.vram[addr], vdp.vram[addr + 1]]);
        assert_eq!(w, 0x1234, "plane A last tile at (39,27)");
    }

    #[test]
    fn intro_sequence_plane_b_repeated_rows_at_e000() {
        // ASM sub_1B850: each of 28 decoded words stamped across 64 columns
        // at VRAM $E000, row stride 128 bytes.
        let mut rows = [[0u16; 64]; 28];
        rows[0] = [0xAAAA; 64];
        rows[27] = [0xBBBB; 64];
        let data = IntroSub1B850Data {
            art_chunks: [
                (0, vec![]),
                (0, vec![]),
                (0, vec![]),
                (0, vec![]),
                (0, vec![]),
            ],
            plane_a_tilemap: vec![],
            plane_b_rows: rows,
            palette_segment_0: vec![],
            palette_segment_1: vec![],
            palette_segment_2: vec![],
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
            art_chunks: [
                (0, vec![]),
                (0, vec![]),
                (0, vec![]),
                (0, vec![]),
                (0, vec![]),
            ],
            plane_a_tilemap: vec![],
            plane_b_rows: [[0; 64]; 28],
            palette_segment_0: vec![0x0111],
            palette_segment_1: vec![0x0222],
            palette_segment_2: vec![0x0333],
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
