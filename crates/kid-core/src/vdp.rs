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
    fn write_vram(&mut self, byte_addr: usize, data: &[u8]) {
        let end = (byte_addr + data.len()).min(self.vram.len());
        let copy_len = end.saturating_sub(byte_addr);
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
}
