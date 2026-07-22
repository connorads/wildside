/// Software VDP state for non-gameplay screens.
///
/// During boot/title/intro sequences, the original game drives the display
/// entirely through VDP register writes, DMA art uploads, and plane nametable
/// setup. This struct mirrors that state so the proven `wildside_md::render_vdp_into`
/// renderer can produce accurate frames.
///
/// The game init register values come from ASM VDPGameInitValues (kid.asm:550).
/// Art is Saxman-compressed in the ROM and decompressed to VRAM.  Palettes are
/// raw 9-bit BGR words written to CRAM.
use anyhow::{Context, Result};
use wildside_data::AssetSource;

use crate::boot::{
    VDP_GAME_INIT_VALUES, intro_text_buggy_substitution_loc_1C7DC, title_text_word_writes_sub_1B7B6,
};
use crate::constants::{
    INTRO_FIFTH_SEGMENT_FRAMES, INTRO_SEQUENCE_3_KID_VISIBLE_ELAPSED_FRAMES,
    INTRO_SEQUENCE_5_FADE_COUNTER_ITERATIONS, INTRO_SEQUENCE_6_NATURAL_TITLE_VDP_READY_FRAMES,
    INTRO_SEQUENCE_6_NATURAL_TITLE_WORKER_START_FRAMES,
    INTRO_SEQUENCE_6_START_SKIP_TITLE_VDP_READY_FRAMES, INTRO_SEQUENCE_6_TITLE_ATTRACT_LOCK_FRAMES,
    TIMEOUT_RESTART_INTRO_SEQUENCE_5_VDP_VISIBLE_ELAPSED_LOC_1BA28, TITLE_ATTRACT_LOCK_FRAMES,
};
use crate::{M68kCycleCount, M68kRefreshPhase};

/// Gameplay-owned VRAM state for HUD DMA operations.
///
/// The gameplay renderer previously inferred the diamond art page directly
/// from `$FFFFFA5D`.  That RAM byte is the `unk_461C` script cursor, whereas
/// `sub_5D4A` writes the selected `off_7B0AC` page into VRAM `$DD40`.  Keep the
/// latter as an independent fact so a later FIFO/scanout timeline can decide
/// when it becomes visible without reinterpreting gameplay RAM.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct GameplayHudVramState {
    diamond_frame_table_byte_offset_loc_5d4a: u8,
}

impl GameplayHudVramState {
    pub(crate) const fn initialised_from_sub_44b0() -> Self {
        // ASM sub_44B0 (kid.asm:5520) leaves the paired animation selectors at
        // zero. The first sub_5D4A transfer therefore selects off_7B0AC[0].
        Self {
            diamond_frame_table_byte_offset_loc_5d4a: 0,
        }
    }

    pub(crate) fn record_diamond_dma_sub_5d4a(&mut self, frame_table_byte_offset: u16) {
        // ASM sub_44DC loc_4626 (kid.asm:5692) passes the frame byte from
        // unk_461C to sub_5D4A, whose off_7B0AC selectors are 0, 4, 8, 12.
        self.diamond_frame_table_byte_offset_loc_5d4a =
            u8::try_from(frame_table_byte_offset).expect("sub_5D4A selector fits in u8");
    }

    pub(crate) const fn diamond_frame_table_byte_offset_loc_5d4a(self) -> u8 {
        self.diamond_frame_table_byte_offset_loc_5d4a
    }
}

const VRAM_SIZE: usize = 0x1_0000;
const CRAM_SIZE: usize = 0x80;
const VSRAM_SIZE: usize = 0x50;
const DECOMPRESS_FIRST_CHUNK_BYTES: usize = 0x1000;
const DECOMPRESS_RESUMED_CHUNK_BYTES: usize = 0x0800;
const DECOMPRESS_ROLLOVER_LONGWORD_DBF: u16 = 0x01FF;

#[allow(non_snake_case)]
fn decode_enigma_tilemap_loc_134D4(data: &[u8], base_tile_word: u16) -> Result<Vec<u16>> {
    crate::level::decode_enigma_eni_dec_loc_134D4(data, base_tile_word).map_err(anyhow::Error::msg)
}

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

/// ROM-visible first frame where Load_SegaScreen's VDP contents render.
///
/// ASM Load_SegaScreen loc_12E48 returns to the outer mode shell after enabling
/// display, then loc_12E64/loc_12E6C starts the logo wait loop. Oracle frames
/// stay black until the setup reaches the first rendered logo frame.
const SEGA_VDP_VISIBLE_START_ELAPSED: usize = 34;

/// ROM-visible elapsed frame where loc_12ED2 first copies a fade strip to CRAM.
const SEGA_FIRST_FADE_COPY_ELAPSED: usize = 67;

/// Fade loop d1 init (frame delay counter) and d3 init (palette cursor).
const SEGA_FADE_FRAME_DELAY_INIT: i16 = 0x14;
const SEGA_FADE_PALETTE_CURSOR_INIT: i16 = 0x28;
/// Oracle FIFO captures put each `loc_12ED2` CRAM strip below the preceding
/// three visible scanouts. The VDP state is post-DMA, while rasterisation must
/// retain the earlier displayed palette for those frames.
const SEGA_FADE_CRAM_PRESENTATION_LAG_LOC_12ED2: usize = 3;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VdpState {
    pub registers: [u8; 0x20],
    pub vram: Vec<u8>,
    pub cram: Vec<u8>,
    pub vsram: Vec<u8>,
    pub raster_frame: Option<VdpRasterFrame>,
}

/// VDP state at the start of visible scanout plus ordered writes while displayed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VdpRasterFrame {
    pub initial_vram: Vec<u8>,
    /// CRAM sampled at the start of visible scanout when a later DMA only
    /// changes the post-frame VDP state.
    pub initial_cram: Option<Vec<u8>>,
    pub vram_word_writes: Vec<VdpRasterVramWordWrite>,
}

/// One FIFO-committed VRAM word write at its visible VDP raster position.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct VdpRasterVramWordWrite {
    pub scanline: u16,
    pub pixel: u16,
    pub byte_addr: u16,
    pub value: u16,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum IntroSequence6RasterLineage {
    ColdBoot,
    WrappedTimeout,
    LaterTimeout,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct IntroSequence6LightningRasterUpload {
    previous_flash_frame: u8,
    start_scanline: u16,
    start_pixel: u16,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct IntroSequence6TitleRasterUpload {
    current_frame_index: Option<u8>,
    previous_frame_index: Option<u8>,
    start_scanline: u16,
    start_pixel: u16,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(crate) struct IntroSequence6RasterUploads {
    pub(crate) lightning: Option<IntroSequence6LightningRasterUpload>,
    pub(crate) title: Option<IntroSequence6TitleRasterUpload>,
}

impl VdpState {
    fn new() -> Self {
        Self {
            registers: [0; 0x20],
            vram: vec![0; VRAM_SIZE],
            cram: vec![0; CRAM_SIZE],
            vsram: vec![0; VSRAM_SIZE],
            raster_frame: None,
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
        if let Some(raster_frame) = self.raster_frame.as_mut() {
            raster_frame.initial_vram[byte_addr..end].copy_from_slice(&data[..copy_len]);
        }
    }

    /// Write a single big-endian word to VRAM at the given byte address.
    fn write_vram_word(&mut self, byte_addr: usize, word: u16) {
        if byte_addr + 1 < self.vram.len() {
            let bytes = word.to_be_bytes();
            self.vram[byte_addr..byte_addr + 2].copy_from_slice(&bytes);
            if let Some(raster_frame) = self.raster_frame.as_mut() {
                raster_frame.initial_vram[byte_addr..byte_addr + 2].copy_from_slice(&bytes);
            }
        }
    }

    /// Write a single big-endian word to VSRAM at the given byte address.
    fn write_vsram_word(&mut self, byte_addr: usize, word: u16) {
        if byte_addr + 1 < self.vsram.len() {
            self.vsram[byte_addr] = (word >> 8) as u8;
            self.vsram[byte_addr + 1] = word as u8;
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

pub(crate) fn blank_vdp_state() -> VdpState {
    VdpState::new()
}

pub(crate) fn solid_colour_vdp_state(colour_word: u16) -> VdpState {
    let mut vdp = VdpState::new();
    // ASM loc_119D8 -> Pal_FadeOut -> loc_6E2 leaves CRAM filled with FBCC's
    // `$082A` while the following VRAM/title rebuild is still hidden.
    vdp.write_cram_words(0, &[colour_word]);
    vdp
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
    if elapsed < SEGA_VDP_VISIBLE_START_ELAPSED {
        return VdpState::new();
    }

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

    let cram = sega_screen_cram_loc_12dfe_12ed2(data, elapsed);
    let visible_elapsed = elapsed.saturating_sub(SEGA_FADE_CRAM_PRESENTATION_LAG_LOC_12ED2);
    let visible_cram = sega_screen_cram_loc_12dfe_12ed2(data, visible_elapsed);
    vdp.cram = cram;
    if vdp.cram != visible_cram {
        vdp.raster_frame = Some(VdpRasterFrame {
            initial_vram: vdp.vram.clone(),
            initial_cram: Some(visible_cram),
            vram_word_writes: Vec::new(),
        });
    }

    vdp
}

/// ASM Load_SegaScreen loc_12DFE/loc_12E96/loc_12ED2: final CRAM state after
/// the initial palette copy and any completed fade-strip uploads.
fn sega_screen_cram_loc_12dfe_12ed2(data: &SegaScreenData, elapsed: usize) -> Vec<u8> {
    let mut cram = vec![0; CRAM_SIZE];

    // Initial palette: copy first 13 words to CRAM line 0.
    for (index, word) in data
        .palette_words
        .iter()
        .take(SEGA_INITIAL_PALETTE_WORD_COUNT)
        .enumerate()
    {
        let byte_offset = index * 2;
        cram[byte_offset..byte_offset + 2].copy_from_slice(&word.to_be_bytes());
    }

    // ASM palette fade animation (loc_12E96 → loc_12ED2):
    // After the hidden setup and initial logo hold, every 4th visible fade
    // frame copies 11 words from a sliding offset into CRAM words 2..12.  The
    // cursor starts at $28 and decreases by 2 each step until it goes negative.
    //
    // Using `saturating_sub` (rather than an `if elapsed > N` guard) keeps the
    // first-copy boundary unambiguous: the fade loop body only executes after
    // d1 has consumed its $14..0 countdown and underflowed.
    let fade_elapsed = elapsed
        .saturating_sub(SEGA_FIRST_FADE_COPY_ELAPSED - SEGA_FADE_FRAME_DELAY_INIT as usize - 1);
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
            for (index, word) in strip.iter().enumerate() {
                let byte_offset = (SEGA_FADE_STRIP_CRAM_WORD_OFFSET + index) * 2;
                cram[byte_offset..byte_offset + 2].copy_from_slice(&word.to_be_bytes());
            }
        }
        palette_cursor -= 2;
    }

    cram
}

/// Load and cache Sega screen art + palette data from the asset bundle.
pub(crate) fn load_sega_screen_data(source: &dyn AssetSource) -> Result<SegaScreenData> {
    let scenes_root = std::path::Path::new("scenes");

    // Load and decompress Saxman-compressed Sega logo art.
    let art_path = scenes_root.join("artcomp/Sega_Logo.bin");
    let art_compressed = source
        .read(&art_path)
        .with_context(|| format!("failed to read Sega logo art: {}", art_path.display()))?;
    let art_bytes = wildside_data::decode_saxman(&art_compressed)
        .context("failed to decompress Sega logo art")?;

    // Load raw palette data (SegaLogo.bin = 66 bytes = 33 big-endian words).
    let palette_path = scenes_root.join("palette/SegaLogo.bin");
    let palette_raw = source.read(&palette_path).with_context(|| {
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
    /// Segment 4 palette words (Pal_1C346 — intro3_alley.bin).
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
    /// IntroSequence2 sky/castle art (`byte_243D5`) staged in `Level_Layout`.
    intro_sequence_2_sky_castle_art: Vec<u8>,
    /// Load_IntroSequence5 common art uploads in ROM order.
    intro_sequence_5_common_art_chunks: [(usize, Vec<u8>); 3],
    /// Load_IntroSequence5 helmet art variants selected by `$FFFFFC82`.
    intro_sequence_5_helmet_art_variants: [Vec<u8>; 3],
    /// Fourteen-word Pal_1C3C6 overlay applied by sub_1C512 when
    /// `$FFFFFC82 == 1`, including its sequential spill into Pal_1C3D4.
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
pub(crate) fn load_intro_sub_1b850_data(source: &dyn AssetSource) -> Result<IntroSub1B850Data> {
    let scenes = std::path::Path::new("scenes");

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
        let decompressed = load_saxman_art(source, &scenes.join(filename))?;
        // VDP addresses in wildside_data are byte addresses (the VDP control port
        // encodes byte addresses, not word addresses, despite the field name).
        art_chunks.push((*vram_addr as usize, decompressed));
    }
    let art_chunks: [(usize, Vec<u8>); 5] = art_chunks
        .try_into()
        .map_err(|_| anyhow::anyhow!("expected 5 art chunks"))?;

    // Decode plane A tilemap: unk_2FDE0 with base tile $80BC → 40×28 tile words.
    // ASM sub_1B850: `move.w #$80BC,d0` then `jsr (j_EniDec)`.
    let tilemap_path = scenes.join("mapeni/intro1_wildside.bin");
    let tilemap_raw = source
        .read(&tilemap_path)
        .with_context(|| format!("failed to read intro tilemap: {}", tilemap_path.display()))?;
    let plane_a_tilemap = decode_enigma_tilemap_loc_134D4(&tilemap_raw, 0x80BC)
        .context("failed to decode intro tilemap")?;

    // Decode plane B row words: unk_2FDCE with base tile $413A → 28+ words.
    // ASM sub_1B850: `move.w #$413A,d0` then `jsr (j_EniDec)`.
    // Each of the first 28 decoded words is stamped across a full 64-word row.
    let row_tilemap_path = scenes.join("mapeni/2FDCE.bin");
    let row_raw = source.read(&row_tilemap_path).with_context(|| {
        format!(
            "failed to read intro row tilemap: {}",
            row_tilemap_path.display()
        )
    })?;
    let decoded_row_words = decode_enigma_tilemap_loc_134D4(&row_raw, 0x413A)
        .context("failed to decode intro row tilemap")?;

    // Use wildside_data to build the load plan (validates and stamps rows).
    let load_plan = wildside_data::title_intro_load_plan_sub_1B850(&decoded_row_words)
        .context("failed to build intro load plan")?;
    let plane_b_rows = load_plan.repeated_rows_upload.rows;

    // Load palettes for each segment variant.
    let palette_segment_0 =
        load_palette_words(source, &scenes.join("palette/intro1_wildside.bin"))?;
    let palette_segment_1 = load_palette_words(source, &scenes.join("palette/intro2_sky.bin"))?;
    let palette_segment_2 = load_palette_words(source, &scenes.join("palette/intro3_alley.bin"))?;

    let hologram_background_art =
        load_saxman_art(source, &scenes.join("artcomp/Hologram_background.bin"))?;
    let hologram_blocks_art = load_saxman_art(source, &scenes.join("artcomp/Hologram_blocks.bin"))?;
    let intro_text_letters_art =
        load_saxman_art(source, &scenes.join("artcomp/Intro_text_letters.bin"))?;

    let hologram_tilemap_raw = source
        .read(&scenes.join("mapeni/hologram_background.bin"))
        .context("failed to read hologram background tilemap")?;
    let intro_sequence_2_hologram_tilemap =
        decode_enigma_tilemap_loc_134D4(&hologram_tilemap_raw, 0x2114)
            .context("failed to decode intro sequence 2 hologram tilemap")?;
    let intro_sequence_4_hologram_tilemap =
        decode_enigma_tilemap_loc_134D4(&hologram_tilemap_raw, 0x20BC)
            .context("failed to decode intro sequence 4 hologram tilemap")?;

    let intro_sequence_2_tilemap_raw = source
        .read(&scenes.join("mapeni/intro2_sky.bin"))
        .context("failed to read intro sequence 2 tilemap")?;
    let intro_sequence_2_tilemap =
        decode_enigma_tilemap_loc_134D4(&intro_sequence_2_tilemap_raw, 0x00BC)
            .context("failed to decode intro sequence 2 tilemap")?;
    let intro_sequence_2_sky_castle_art = load_saxman_art(
        source,
        &scenes.join("artcomp/Sky_castle_and_night_sky_from_intro_scene.bin"),
    )?;
    let intro_sequence_2_sky_castle_art =
        intro_sequence_2_special_sky_art_loc_1B00A(&intro_sequence_2_sky_castle_art);

    let intro_sequence_5_common_art_files: [(u16, &str); 3] = [
        (0x1780, "artcomp/Dark_alley_from_intro.bin"),
        (0x4D80, "artcomp/P_block_from_intro.bin"),
        (0x6800, "artcomp/The_kid_from_intro.bin"),
    ];
    let mut intro_sequence_5_common_art_chunks = Vec::new();
    for (vram_addr, filename) in &intro_sequence_5_common_art_files {
        intro_sequence_5_common_art_chunks.push((
            *vram_addr as usize,
            load_saxman_art(source, &scenes.join(filename))?,
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
        intro_sequence_5_helmet_art_variants.push(load_saxman_art(source, &scenes.join(filename))?);
    }
    let intro_sequence_5_helmet_art_variants: [Vec<u8>; 3] = intro_sequence_5_helmet_art_variants
        .try_into()
        .map_err(|_| anyhow::anyhow!("expected 3 intro sequence 5 helmet art variants"))?;
    let mut intro_sequence_5_palette_variant_1_overlay =
        load_palette_words(source, &scenes.join("palette/title_maniaxe_mask.bin"))?;
    let intro_sequence_5_palette_variant_2_overlay =
        load_palette_words(source, &scenes.join("palette/title_juggernaut_mask.bin"))?;
    // ASM loc_1C540/loc_1C542 (kid.asm:31273-31278) copies 14 words from
    // Pal_1C3C6 even though the Maniaxe mask contains only seven words before
    // Pal_1C3D4. Preserve that sequential ROM read instead of leaving the
    // second half of Palette_Buffer_2's overlay stale.
    const INTRO_SEQUENCE_5_PALETTE_OVERLAY_WORDS_LOC_1C542: usize = 0x0D + 1;
    let spill_words = INTRO_SEQUENCE_5_PALETTE_OVERLAY_WORDS_LOC_1C542
        .saturating_sub(intro_sequence_5_palette_variant_1_overlay.len());
    intro_sequence_5_palette_variant_1_overlay.extend(
        intro_sequence_5_palette_variant_2_overlay
            .iter()
            .take(spill_words),
    );
    let intro_sequence_5_tilemap_raw = source
        .read(&scenes.join("mapeni/intro3_alley.bin"))
        .context("failed to read intro sequence 5 tilemap")?;
    let intro_sequence_5_tilemap =
        decode_enigma_tilemap_loc_134D4(&intro_sequence_5_tilemap_raw, 0x00BC)
            .context("failed to decode intro sequence 5 tilemap")?;
    let mut intro_sequence_5_frame_tilemaps = Vec::new();
    for frame_index in 0..8u16 {
        let plan = wildside_data::intro_sequence_5_frame_load_plan_sub_1C034(frame_index)
            .context("failed to build intro sequence 5 frame upload plan")?;
        let filename = asset_label_mapeni_filename(plan.tilemap_load.asset_label);
        let raw = source
            .read(&scenes.join("mapeni").join(&filename))
            .with_context(|| {
                format!("failed to read intro sequence 5 frame tilemap: {filename}")
            })?;
        let decoded = decode_enigma_tilemap_loc_134D4(&raw, plan.tilemap_load.vram_word_address)
            .with_context(|| {
                format!("failed to decode intro sequence 5 frame tilemap: {filename}")
            })?;
        intro_sequence_5_frame_tilemaps.push(decoded);
    }
    let panel_plan = wildside_data::intro_sequence_5_panel_load_plan_sub_1C0EE(3, 0x13);
    let panel_filename = asset_label_mapeni_filename(panel_plan.tilemap_load.asset_label);
    let panel_raw = source
        .read(&scenes.join("mapeni").join(&panel_filename))
        .with_context(|| {
            format!("failed to read intro sequence 5 panel tilemap: {panel_filename}")
        })?;
    let intro_sequence_5_panel_tilemap =
        decode_enigma_tilemap_loc_134D4(&panel_raw, panel_plan.tilemap_load.vram_word_address)
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
        intro_sequence_2_sky_castle_art,
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
    build_intro_sequence_vdp_with_title_variant_at_elapsed_and_visible_threshold(
        data,
        segment,
        title_variant,
        elapsed,
        intro_sequence_3_roster_index,
        INTRO_SEQUENCE_5_VISIBLE_ELAPSED_LOC_1BA28,
        IntroSequence5WorkerLineage::Shared,
    )
}

pub(crate) fn build_timeout_restart_intro_sequence_5_vdp_at_elapsed_loc_1ba28(
    data: &IntroSub1B850Data,
    title_variant: u8,
    elapsed: usize,
    intro_sequence_3_roster_index: usize,
    workers_wake_one_frame_early: bool,
) -> VdpState {
    // ASM Load_IntroSequence5 allocates loc_1BA28 and loc_1BCD0 together.
    // sub_1C204 exposes the loaded VDP three presented traces after the first
    // loc_1BCD0 pass on timeout-restart visits. The later Start-skip lineage
    // advances that complete outer pass by one frame, as independently proven
    // by its slot-2 WRAM visibility.
    let visible_elapsed = TIMEOUT_RESTART_INTRO_SEQUENCE_5_VDP_VISIBLE_ELAPSED_LOC_1BA28
        .saturating_sub(usize::from(workers_wake_one_frame_early));
    let mut vdp = build_intro_sequence_vdp_with_title_variant_at_elapsed_and_visible_threshold(
        data,
        4,
        title_variant,
        elapsed,
        intro_sequence_3_roster_index,
        visible_elapsed,
        // ASM loc_1BCD0 -> loc_1BE3C (kid.asm:30567-30667): the later FC82=1
        // Start-skip revisit exposes the newest allocated particle first and
        // one moved pass ahead. Oracle SAT frames 16647-16649 distinguish this
        // from the cold/shared FC82=1 allocation order.
        match (title_variant, workers_wake_one_frame_early) {
            (0, _) => IntroSequence5WorkerLineage::TimeoutFc82Zero,
            (1, true) => IntroSequence5WorkerLineage::TimeoutFc82OneStartSkip,
            (2, _) => IntroSequence5WorkerLineage::TimeoutFc82Two,
            _ => IntroSequence5WorkerLineage::Shared,
        },
    );
    if (visible_elapsed..visible_elapsed + 0x10).contains(&elapsed) {
        // ASM sub_1C204 starts `$FFFFF876` at zero and advances it by `$10`
        // per VBlank. Fade the complete Pal_1C346 plus loc_1C540 region
        // overlay already present in this VDP, rather than fading only the
        // common first 49 words.
        let target_palette = vdp_cram_words(&vdp);
        apply_intro_sequence_palette_fade_in_loc_1c204(
            &mut vdp,
            &target_palette,
            elapsed,
            // ASM loc_1C234 seeds the pending palette masks before the first
            // loc_1C21E hibernate. That hidden pass consumes `$10`; the first
            // presented alley frame therefore carries `$20`, not `$10`.
            visible_elapsed.saturating_sub(1),
        );
    }
    vdp
}

fn build_intro_sequence_vdp_with_title_variant_at_elapsed_and_visible_threshold(
    data: &IntroSub1B850Data,
    segment: usize,
    title_variant: u8,
    elapsed: usize,
    intro_sequence_3_roster_index: usize,
    intro_sequence_5_visible_elapsed: usize,
    intro_sequence_5_worker_lineage: IntroSequence5WorkerLineage,
) -> VdpState {
    if segment == 1 {
        return build_intro_sequence_2_handoff_vdp_at_elapsed(data, title_variant, elapsed);
    }
    if segment == 2 {
        return build_intro_sequence_3_handoff_vdp_at_elapsed(
            data,
            title_variant,
            elapsed,
            intro_sequence_3_roster_index,
            None,
        );
    }
    if segment == 3 {
        return build_intro_sequence_4_handoff_vdp_at_elapsed(data, title_variant, elapsed, None);
    }
    if segment == 4 && elapsed < intro_sequence_5_visible_elapsed {
        // ASM loc_1BA28 allocates IntroText5/loc_1BC36, then enters the same
        // sub_1C204 hidden shell before loc_1BA5C's visible loop. Oracle frame
        // 1920 has blank VRAM+CRAM at intro-sequence-5 elapsed 44.
        return blank_vdp_state();
    }

    let mut vdp = build_intro_sequence_vdp_base_with_title_variant(data, segment, title_variant);
    if segment == 0 && elapsed >= INTRO_SEQUENCE_1_PROMPT_VISIBLE_ELAPSED_SUB_1C54A {
        overlay_intro_sequence_1_prompt_sprite_sub_1C54A(&mut vdp, elapsed);
    }
    if segment == 4 {
        overlay_intro_sequence_5_tile_worker_loc_1BC36(
            &mut vdp,
            data,
            elapsed.saturating_sub(intro_sequence_5_worker_lineage.tile_worker_delay()),
            intro_sequence_5_worker_lineage,
        );
        overlay_intro_sequence_5_sprite_table_loc_1BCD0(
            &mut vdp,
            title_variant,
            elapsed,
            intro_sequence_5_worker_lineage,
        );
    }
    overlay_intro_text_worker_loc_1c7a0(&mut vdp, segment, elapsed);
    vdp
}

pub(crate) fn build_intro_sequence_5_fade_hold_vdp_at_elapsed(
    data: &IntroSub1B850Data,
    title_variant: u8,
    fade_elapsed: usize,
    intro_sequence_3_roster_index: usize,
) -> VdpState {
    let worker_elapsed = INTRO_FIFTH_SEGMENT_FRAMES + fade_elapsed;
    let mut vdp = build_intro_sequence_vdp_with_title_variant_at_elapsed(
        data,
        4,
        title_variant,
        worker_elapsed,
        intro_sequence_3_roster_index,
    );
    // ASM loc_1BA90 displays the previously latched VDP state for a frame via
    // Hibernate_Object_1Frame before sub_1C246/subi.w update the fade counter.
    apply_intro_sequence_5_loc_1ba90_fade_to_aaa(&mut vdp, fade_elapsed.saturating_sub(1));
    vdp
}

#[allow(non_snake_case)]
fn apply_intro_sequence_5_loc_1ba90_fade_to_aaa(vdp: &mut VdpState, fade_elapsed: usize) {
    let clamped_elapsed = fade_elapsed.min(INTRO_SEQUENCE_5_FADE_COUNTER_ITERATIONS);
    let fade_progress = 0x0100u16.saturating_sub((clamped_elapsed as u16).saturating_mul(0x20));
    let palette_buffer_2 = vdp_cram_words(vdp);
    let palette_buffer_3 = [0x0AAA; 0x40];
    let mut palette_buffer = [0u16; 0x40];
    let mut pending_palette_0_mask = u32::MAX;
    let mut pending_palette_1_mask = u32::MAX;

    // ASM loc_1BA82 fills Palette_Buffer_3 with `$AAA`, and loc_1BAA0 marks
    // both palette halves dirty before each loc_1BA90 fade step runs sub_1C246.
    crate::level::apply_pending_palette_fade_sub_14C0(
        fade_progress,
        &mut pending_palette_0_mask,
        &mut pending_palette_1_mask,
        &palette_buffer_2,
        &palette_buffer_3,
        &mut palette_buffer,
    );
    vdp.write_cram_words(0, &palette_buffer);
}

pub(crate) fn apply_intro_title_return_fade_to_black_loc_28c6(
    vdp: &mut VdpState,
    title_elapsed_frames: usize,
) -> bool {
    let fade_progress = match title_elapsed_frames {
        0..=3 => return true,
        4..=17 => {
            0x0100u16.saturating_sub(((title_elapsed_frames - 3) as u16).saturating_mul(0x10))
        }
        _ => return false,
    };
    // ASM sub_1B93E/loc_1BA02/loc_1BA60 -> sub_1C246 -> loc_1C264 jump
    // through loc_6E2. Because `$FFFFFBCE` is set, loc_6E2 runs Pal_FadeOut
    // (loc_28C6), stepping `$FFFFF876` by `$10` per VBlank before the VRAM
    // clear.
    apply_pal_fade_out_to_black_loc_28c6(vdp, fade_progress);
    true
}

pub(crate) fn apply_title_to_demo_fade_to_black_loc_28c6(
    vdp: &mut VdpState,
    demo_elapsed_frames: usize,
) -> bool {
    let fade_progress = match demo_elapsed_frames {
        0..=2 => return true,
        3..=17 => 0x0100u16.saturating_sub(((demo_elapsed_frames - 2) as u16).saturating_mul(0x10)),
        _ => return false,
    };
    // ASM Obj_TitleMenu loc_1BB02 writes Game_Mode=$10 before jumping through
    // loc_6E2. This entry reaches Pal_FadeOut one presentation boundary ahead
    // of the sub_1C246 title-return path: three resident frames, then all 15
    // descending `$10` steps before Load_DemoPlay's black bootstrap.
    apply_pal_fade_out_to_black_loc_28c6(vdp, fade_progress);
    true
}

pub(crate) fn apply_pal_fade_out_to_black_loc_28c6(vdp: &mut VdpState, fade_progress: u16) {
    let palette_buffer_2 = vdp_cram_words(vdp);
    let palette_buffer_3 = [0u16; 0x40];
    let mut palette_buffer = [0u16; 0x40];
    let mut pending_palette_0_mask = u32::MAX;
    let mut pending_palette_1_mask = u32::MAX;

    crate::level::apply_pending_palette_fade_sub_14C0(
        fade_progress,
        &mut pending_palette_0_mask,
        &mut pending_palette_1_mask,
        &palette_buffer_2,
        &palette_buffer_3,
        &mut palette_buffer,
    );
    vdp.write_cram_words(0, &palette_buffer);
}

pub(crate) fn build_intro_sequence_1_cold_boot_vdp_at_elapsed(
    data: &IntroSub1B850Data,
    sega_data: &SegaScreenData,
    title_variant: u8,
    elapsed: usize,
) -> VdpState {
    let mut vdp = build_intro_sequence_1_vdp_at_elapsed(
        data,
        sega_data,
        title_variant,
        elapsed,
        INTRO_SEQUENCE_1_COLD_BOOT_FADE_IN_START_ELAPSED,
        INTRO_SEQUENCE_1_COLD_BOOT_FULL_PALETTE_ELAPSED,
    );
    let first_intro_text_glyph = INTRO_TEXT_WORKER_START_ELAPSED_LOC_1C7A0[0] + 1 + 0x20;
    if elapsed >= first_intro_text_glyph {
        let previous_vdp = build_intro_sequence_1_vdp_at_elapsed(
            data,
            sega_data,
            title_variant,
            elapsed - 1,
            INTRO_SEQUENCE_1_COLD_BOOT_FADE_IN_START_ELAPSED,
            INTRO_SEQUENCE_1_COLD_BOOT_FULL_PALETTE_ELAPSED,
        );
        vdp.raster_frame = Some(VdpRasterFrame {
            initial_vram: previous_vdp.vram,
            initial_cram: None,
            vram_word_writes: Vec::new(),
        });
    }
    vdp
}

pub(crate) fn build_intro_sequence_1_demo_restart_vdp_at_elapsed(
    data: &IntroSub1B850Data,
    sega_data: &SegaScreenData,
    title_variant: u8,
    elapsed: usize,
) -> VdpState {
    build_intro_sequence_1_vdp_at_elapsed(
        data,
        sega_data,
        title_variant,
        elapsed,
        INTRO_SEQUENCE_1_DEMO_RESTART_FADE_IN_START_ELAPSED,
        INTRO_SEQUENCE_1_DEMO_RESTART_FULL_PALETTE_ELAPSED,
    )
}

fn build_intro_sequence_1_vdp_at_elapsed(
    data: &IntroSub1B850Data,
    sega_data: &SegaScreenData,
    title_variant: u8,
    elapsed: usize,
    fade_in_start_elapsed: usize,
    full_palette_elapsed: usize,
) -> VdpState {
    // ASM Load_IntroSequence1 runs `sub_1B8F4 -> sub_1C204 -> loc_1B924`
    // after `Load_SegaScreen` hands off at loc_12EDE. The ROM-visible VDP
    // output keeps the Sega palette fade on screen first, then holds black
    // before sub_1C204 fades Pal_1C28A into CRAM for the Wild Side scene.
    let cold_boot = fade_in_start_elapsed == INTRO_SEQUENCE_1_COLD_BOOT_FADE_IN_START_ELAPSED;
    if elapsed < INTRO_SEQUENCE_1_SEGA_FADE_OUT_ELAPSED
        || (cold_boot && elapsed == INTRO_SEQUENCE_1_SEGA_FADE_OUT_ELAPSED)
    {
        let mut vdp = build_sega_screen_vdp(sega_data, SEGA_SCREEN_INTRO_HANDOFF_ELAPSED + elapsed);
        if elapsed == INTRO_SEQUENCE_1_SEGA_FADE_OUT_ELAPSED {
            // ASM sub_1C204 returns through loc_1C240 after its last hibernate.
            // The following visible frame still scans the resident Sega planes;
            // the IntroSequence1 black interval begins on the next frame.
            apply_intro_sequence_1_sega_fade_out_loc_1c204(&mut vdp, elapsed - 1);
        } else if elapsed >= INTRO_SEQUENCE_1_SEGA_FADE_START_ELAPSED {
            let mut visible_vdp = vdp.clone();
            if elapsed > INTRO_SEQUENCE_1_SEGA_FADE_START_ELAPSED {
                apply_intro_sequence_1_sega_fade_out_loc_1c204(&mut visible_vdp, elapsed - 1);
            }
            apply_intro_sequence_1_sega_fade_out_loc_1c204(&mut vdp, elapsed);
            if cold_boot {
                // ASM sub_1C204 (kid.asm:31084-31113) changes Palette_Buffer,
                // then Palette_to_VRAM's CRAM DMA completes on oracle scanlines
                // 237-238. Those writes are below the 224-line visible field:
                // retain the post-DMA CRAM for state parity but rasterise this
                // frame from the pre-DMA palette.
                vdp.raster_frame = Some(VdpRasterFrame {
                    initial_vram: vdp.vram.clone(),
                    initial_cram: Some(visible_vdp.cram),
                    vram_word_writes: Vec::new(),
                });
            }
        }
        return vdp;
    }

    let mut vdp =
        build_intro_sequence_vdp_with_title_variant_at_elapsed(data, 0, title_variant, elapsed, 0);
    if elapsed < fade_in_start_elapsed {
        vdp.cram.fill(0);
        return vdp;
    }

    if (fade_in_start_elapsed..full_palette_elapsed).contains(&elapsed) {
        apply_intro_sequence_1_sub_1c204_palette_fade(
            &mut vdp,
            data,
            elapsed,
            fade_in_start_elapsed,
        );
    }
    vdp
}

const SEGA_SCREEN_INTRO_HANDOFF_ELAPSED: usize = 173;
const INTRO_SEQUENCE_1_SEGA_FADE_START_ELAPSED: usize = 3;
const INTRO_SEQUENCE_1_SEGA_FADE_OUT_ELAPSED: usize = 17;
// ASM sub_1B8F4's sub_1C204 fade writes Pal_1C28A after hibernating; the
// first updated palette is presented on the following VBlank.
const INTRO_SEQUENCE_1_COLD_BOOT_FADE_IN_START_ELAPSED: usize = 47;
const INTRO_SEQUENCE_1_COLD_BOOT_FULL_PALETTE_ELAPSED: usize = 62;
const INTRO_SEQUENCE_1_DEMO_RESTART_FADE_IN_START_ELAPSED: usize = 46;
const INTRO_SEQUENCE_1_DEMO_RESTART_FULL_PALETTE_ELAPSED: usize = 61;
const INTRO_SEQUENCE_1_PROMPT_VISIBLE_ELAPSED_SUB_1C54A: usize = 43;
const INTRO_SEQUENCE_2_PREVIOUS_FADE_OUT_ELAPSED: usize = 17;
const INTRO_SEQUENCE_2_FADE_IN_START_ELAPSED: usize = 44;
const INTRO_SEQUENCE_2_FULL_PALETTE_ELAPSED: usize = 60;
const INTRO_SEQUENCE_3_PREVIOUS_FADE_OUT_ELAPSED: usize = 17;
const INTRO_SEQUENCE_3_FADE_IN_START_ELAPSED: usize = 44;
const INTRO_SEQUENCE_3_FULL_PALETTE_ELAPSED: usize = 60;
const INTRO_SEQUENCE_4_PREVIOUS_FADE_OUT_ELAPSED: usize = 17;
const INTRO_SEQUENCE_4_FADE_IN_START_ELAPSED: usize = 39;
const INTRO_SEQUENCE_4_FULL_PALETTE_ELAPSED: usize = 55;
const INTRO_SEQUENCE_5_PREVIOUS_FADE_OUT_ELAPSED: usize = 17;
const INTRO_SEQUENCE_2_SKY_REVEAL_LOOP_START_ELAPSED_LOC_1BA02: usize = 42 + 0x3C + 18;
const INTRO_SEQUENCE_2_SKY_REVEAL_LOOP_FRAMES_LOC_1BA02: usize = 0x014A;

fn build_intro_sequence_2_handoff_vdp_at_elapsed(
    data: &IntroSub1B850Data,
    title_variant: u8,
    elapsed: usize,
) -> VdpState {
    // ASM loc_1B9D0 enters the same sub_1C204 fade shell used by
    // sub_1B8F4. ROM-visible output keeps the completed Wild Side VDP around
    // while fading its palette to black, then holds black before fading
    // Pal_1C2F0 into the hologram-room VDP.
    if elapsed < INTRO_SEQUENCE_2_PREVIOUS_FADE_OUT_ELAPSED {
        let mut vdp =
            build_intro_sequence_vdp_with_title_variant_at_elapsed(data, 0, title_variant, 999, 0);
        if elapsed > 0 {
            apply_intro_sequence_fade_out_to_black_loc_1c204(
                &mut vdp,
                elapsed,
                INTRO_SEQUENCE_2_PREVIOUS_FADE_OUT_ELAPSED,
            );
        }
        return vdp;
    }

    let mut vdp = build_intro_sequence_vdp_base_with_title_variant(data, 1, title_variant);
    overlay_intro_sequence_2_sky_reveal_loc_1BA02(&mut vdp, data, elapsed);
    overlay_intro_text_worker_loc_1c7a0(&mut vdp, 1, elapsed);

    if elapsed < INTRO_SEQUENCE_2_FADE_IN_START_ELAPSED {
        vdp.cram.fill(0);
        return vdp;
    }
    if (INTRO_SEQUENCE_2_FADE_IN_START_ELAPSED..INTRO_SEQUENCE_2_FULL_PALETTE_ELAPSED)
        .contains(&elapsed)
    {
        apply_intro_sequence_palette_fade_in_loc_1c204(
            &mut vdp,
            &data.palette_segment_1,
            elapsed,
            INTRO_SEQUENCE_2_FADE_IN_START_ELAPSED,
        );
    }
    vdp
}

pub(crate) fn build_intro_sequence_3_handoff_vdp_at_elapsed(
    data: &IntroSub1B850Data,
    title_variant: u8,
    elapsed: usize,
    intro_sequence_3_roster_index: usize,
    resident_intro_sequence_2_vdp: Option<&VdpState>,
) -> VdpState {
    // ASM Load_IntroSequence3 -> sub_1B93E -> sub_1C204 first presents the
    // resident IntroSequence2 image and fades its CRAM to black. The rebuilt
    // Wild Side scene remains black while loc_1AEEC allocates the roster, then
    // Pal_1C28A fades in from the first post-loader object passes.
    if elapsed < INTRO_SEQUENCE_3_PREVIOUS_FADE_OUT_ELAPSED {
        let mut vdp = resident_intro_sequence_2_vdp
            .cloned()
            .unwrap_or_else(blank_vdp_state);
        if elapsed > 0 {
            apply_intro_sequence_fade_out_to_black_loc_1c204(
                &mut vdp,
                elapsed,
                INTRO_SEQUENCE_3_PREVIOUS_FADE_OUT_ELAPSED,
            );
        }
        return vdp;
    }

    let mut vdp = build_intro_sequence_vdp_base_with_title_variant(data, 2, title_variant);
    if elapsed >= INTRO_SEQUENCE_3_KID_VISIBLE_ELAPSED_FRAMES {
        overlay_intro_sequence_3_sprite_table_loc_1AEEC(
            &mut vdp,
            intro_sequence_3_roster_index,
            elapsed - INTRO_SEQUENCE_3_KID_VISIBLE_ELAPSED_FRAMES,
        );
    }
    overlay_intro_text_worker_loc_1c7a0(&mut vdp, 2, elapsed);

    if elapsed < INTRO_SEQUENCE_3_FADE_IN_START_ELAPSED {
        vdp.cram.fill(0);
        return vdp;
    }
    if (INTRO_SEQUENCE_3_FADE_IN_START_ELAPSED..INTRO_SEQUENCE_3_FULL_PALETTE_ELAPSED)
        .contains(&elapsed)
    {
        apply_intro_sequence_palette_fade_in_loc_1c204(
            &mut vdp,
            &data.palette_segment_0,
            elapsed,
            INTRO_SEQUENCE_3_FADE_IN_START_ELAPSED,
        );
    }
    vdp
}

pub(crate) fn build_intro_sequence_4_handoff_vdp_at_elapsed(
    data: &IntroSub1B850Data,
    title_variant: u8,
    elapsed: usize,
    resident_intro_sequence_3_vdp: Option<&VdpState>,
) -> VdpState {
    // ASM sub_1B93E -> loc_1B96A -> loc_6E2 first fades the resident
    // IntroSequence3 VDP to black. Load_IntroSequence4 then rebuilds the
    // corridor image under black before sub_1B988 -> sub_1C204 fades
    // Pal_1C2F0 into CRAM.
    if elapsed < INTRO_SEQUENCE_4_PREVIOUS_FADE_OUT_ELAPSED {
        let mut vdp = resident_intro_sequence_3_vdp
            .cloned()
            .unwrap_or_else(blank_vdp_state);
        if elapsed > 0 {
            apply_intro_sequence_fade_out_to_black_loc_1c204(
                &mut vdp,
                elapsed,
                INTRO_SEQUENCE_4_PREVIOUS_FADE_OUT_ELAPSED,
            );
        }
        return vdp;
    }

    let mut vdp = build_intro_sequence_vdp_base_with_title_variant(data, 3, title_variant);
    overlay_intro_text_worker_loc_1c7a0(&mut vdp, 3, elapsed);
    if elapsed < INTRO_SEQUENCE_4_FADE_IN_START_ELAPSED {
        vdp.cram.fill(0);
        return vdp;
    }
    if (INTRO_SEQUENCE_4_FADE_IN_START_ELAPSED..INTRO_SEQUENCE_4_FULL_PALETTE_ELAPSED)
        .contains(&elapsed)
    {
        apply_intro_sequence_palette_fade_in_loc_1c204(
            &mut vdp,
            &data.palette_segment_1,
            elapsed,
            INTRO_SEQUENCE_4_FADE_IN_START_ELAPSED,
        );
    }
    vdp
}

pub(crate) fn build_intro_sequence_5_handoff_vdp_at_elapsed(
    elapsed: usize,
    resident_intro_sequence_4_vdp: Option<&VdpState>,
) -> Option<VdpState> {
    // ASM sub_1B988 loc_1B9B6 -> loc_6E2 first presents and fades the
    // resident IntroSequence4 VDP. Only after Pal_FadeOut returns does
    // Load_IntroSequence5 clear VRAM and build loc_1BA28 under black.
    if elapsed >= INTRO_SEQUENCE_5_PREVIOUS_FADE_OUT_ELAPSED {
        return None;
    }

    let mut vdp = resident_intro_sequence_4_vdp
        .cloned()
        .unwrap_or_else(blank_vdp_state);
    if elapsed > 2 {
        apply_intro_sequence_fade_out_to_black_loc_1c204(
            &mut vdp,
            elapsed,
            INTRO_SEQUENCE_5_PREVIOUS_FADE_OUT_ELAPSED,
        );
    }
    Some(vdp)
}

fn apply_intro_sequence_1_sub_1c204_palette_fade(
    vdp: &mut VdpState,
    data: &IntroSub1B850Data,
    elapsed: usize,
    fade_in_start_elapsed: usize,
) {
    apply_intro_sequence_palette_fade_in_loc_1c204(
        vdp,
        &data.palette_segment_0,
        elapsed,
        fade_in_start_elapsed,
    );
}

fn apply_intro_sequence_palette_fade_in_loc_1c204(
    vdp: &mut VdpState,
    target_palette: &[u16],
    elapsed: usize,
    fade_in_start_elapsed: usize,
) {
    let fade_step = (elapsed - fade_in_start_elapsed + 1).min(0x10);
    let fade_progress =
        u16::try_from(fade_step * 0x10).expect("intro sequence 1 fade progress should fit in u16");
    let mut palette_buffer_2 = [0u16; 0x40];
    for (slot, word) in palette_buffer_2
        .iter_mut()
        .zip(target_palette.iter().copied())
    {
        *slot = word;
    }
    let palette_buffer_3 = [0u16; 0x40];
    let mut palette_buffer = [0u16; 0x40];
    let mut pending_palette_0_mask = u32::MAX;
    let mut pending_palette_1_mask = u32::MAX;

    crate::level::apply_pending_palette_fade_sub_14C0(
        fade_progress,
        &mut pending_palette_0_mask,
        &mut pending_palette_1_mask,
        &palette_buffer_2,
        &palette_buffer_3,
        &mut palette_buffer,
    );
    vdp.write_cram_words(0, &palette_buffer);
}

fn apply_intro_sequence_1_sega_fade_out_loc_1c204(vdp: &mut VdpState, elapsed: usize) {
    apply_intro_sequence_fade_out_to_black_loc_1c204(
        vdp,
        elapsed,
        INTRO_SEQUENCE_1_SEGA_FADE_OUT_ELAPSED,
    );
}

fn apply_intro_sequence_fade_out_to_black_loc_1c204(
    vdp: &mut VdpState,
    elapsed: usize,
    fade_out_elapsed: usize,
) {
    let fade_step = fade_out_elapsed - elapsed + 1;
    let fade_progress =
        u16::try_from(fade_step * 0x10).expect("intro sequence 1 fade progress should fit in u16");
    let palette_buffer_2 = vdp_cram_words(vdp);
    let palette_buffer_3 = [0u16; 0x40];
    let mut palette_buffer = [0u16; 0x40];
    let mut pending_palette_0_mask = u32::MAX;
    let mut pending_palette_1_mask = u32::MAX;

    crate::level::apply_pending_palette_fade_sub_14C0(
        fade_progress,
        &mut pending_palette_0_mask,
        &mut pending_palette_1_mask,
        &palette_buffer_2,
        &palette_buffer_3,
        &mut palette_buffer,
    );
    vdp.write_cram_words(0, &palette_buffer);
}

fn vdp_cram_words(vdp: &VdpState) -> [u16; 0x40] {
    core::array::from_fn(|word_offset| {
        let byte_offset = word_offset * 2;
        u16::from_be_bytes([vdp.cram[byte_offset], vdp.cram[byte_offset + 1]])
    })
}

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
    let mut objects = Vec::new();
    push_intro_sequence_prompt_sprite_sub_1C54A(&mut objects);
    for object in objects {
        append_intro_sprite_entries_make_sprite_from_gfx_object_loc_129E(
            entries,
            object.gfx,
            object.frame,
        );
    }
}

#[allow(non_snake_case)]
fn push_intro_sequence_prompt_sprite_sub_1C54A(
    objects: &mut Vec<IntroGfxObjectForSpriteTableWithOrder>,
) {
    // ASM sub_1B8F4 -> sub_1C54A (kid.asm:30194,31288): after the Wild Side
    // VDP setup, the controller allocates a fixed visible sprite using
    // `unk_E109E` at VRAM tile `$140`. That art is the door/interior block
    // uploaded by sub_1B850's `Wild_Side_door_and_inside.bin` DMA.
    let prompt = crate::boot::title_menu_prompt_sprite_sub_1C54A();
    objects.push(IntroGfxObjectForSpriteTableWithOrder {
        order_key: 0x0100,
        gfx: IntroGfxObjectForSpriteTable {
            x_pos: prompt.x_pos,
            y_pos: prompt.y_pos,
            palette_line: 0,
            vram_tile: prompt.vram_tile,
            priority: 0,
            x_direction: 0,
        },
        frame: IntroSpriteFrameVram {
            sprite_index: 0,
            x_center: 0,
            y_center: 0,
            width: 0x3E,
            height: 0x2F,
        },
    });
}

#[allow(non_snake_case)]
fn overlay_intro_sequence_2_sky_reveal_loc_1BA02(
    vdp: &mut VdpState,
    data: &IntroSub1B850Data,
    elapsed: usize,
) {
    // ASM loc_1B9D0 -> loc_1B9F4 hibernates `$3C` frames after the shared
    // intro-text/fade shell, then loc_1BA02 calls sub_1C1A2 twice and
    // sub_1C180 once per frame. `d1` starts at `-$40`, so the first 32 frames
    // only increment through the negative selector range before visible sky
    // columns begin replacing the hologram-room art at VRAM `$1780`.
    let Some(loop_elapsed) =
        elapsed.checked_sub(INTRO_SEQUENCE_2_SKY_REVEAL_LOOP_START_ELAPSED_LOC_1BA02)
    else {
        return;
    };
    if data.intro_sequence_2_sky_castle_art.is_empty() {
        return;
    }

    let loop_frames = (loop_elapsed + 1).min(INTRO_SEQUENCE_2_SKY_REVEAL_LOOP_FRAMES_LOC_1BA02);
    let mut level_layout = data.intro_sequence_2_sky_castle_art.clone();
    let shadow_layout = level_layout.clone();
    let mut selector = -0x40i16;
    for _ in 0..loop_frames {
        wildside_data::mask_level_layout_column_sub_1C1A2(&mut level_layout, selector);
        selector = selector.saturating_add(1);
        wildside_data::mask_level_layout_column_sub_1C1A2(&mut level_layout, selector);
        selector = selector.saturating_add(1);
    }

    let Ok(upload) =
        wildside_data::level_layout_xor_vram_upload_sub_1C180(&level_layout, &shadow_layout)
    else {
        return;
    };
    let mut bytes = Vec::with_capacity(upload.longwords.len() * 4);
    for longword in upload.longwords {
        bytes.extend_from_slice(&longword.to_be_bytes());
    }
    vdp.write_vram(usize::from(upload.vram_word_address), &bytes);
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
    // ASM Data_Index entries at kid.asm:48231-48244 and `sprite_frame_vram`
    // records at unk_E1106..unk_E116E provide the frame metadata consumed by
    // Make_SpriteFromGfxObject for Load_IntroSequence3's loc_1AEEC objects.
    let (sprite_index, width, height) = match addroffset_sprite {
        0x1208 => (0x000, 0x18, 0x24),
        0x120C => (0x00F, 0x15, 0x23),
        0x1210 => (0x01E, 0x12, 0x28),
        0x1214 => (0x02D, 0x12, 0x25),
        0x1218 => (0x03C, 0x12, 0x23),
        0x121C => (0x04B, 0x13, 0x1F),
        0x1220 => (0x057, 0x1D, 0x2C),
        0x1224 => (0x06F, 0x20, 0x24),
        0x1228 => (0x083, 0x13, 0x1D),
        0x122C => (0x08F, 0x15, 0x2D),
        0x1230 => (0x0A1, 0x0F, 0x23),
        0x1234 => (0x0AB, 0x0F, 0x23),
        0x1238 => (0x0B5, 0x1D, 0x32),
        0x123C => (0x0D1, 0x18, 0x23),
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
    let mut sprite_objects = Vec::new();
    // ASM sub_1C54A allocates gfx slot `$01000000`, before the `$01FF/$0200/$0201`
    // intro roster and sliding-door slots, so emit it first to preserve hardware
    // sprite ordering.
    push_intro_sequence_prompt_sprite_sub_1C54A(&mut sprite_objects);
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
        let allocation_spec =
            crate::movement::intro_sequence_3_allocation_spec_loc_1AEE2(descriptor);
        let allocation_tail = crate::movement::gfx_object_alloc_tail_loc_FBE(
            allocation_spec,
            sprite_objects.len() as u16,
            0,
        );
        sprite_objects.push(IntroGfxObjectForSpriteTableWithOrder {
            // ASM Allocate_GfxObjectSlot loc_FBE stores the high word of the
            // caller's `$01FF0000`/`$02010000` allocation spec at `$A(a3)`;
            // loc_FD4 sorts the gfx list by that word before
            // Make_SpriteFromGfxObject builds the hardware sprite table.
            order_key: allocation_tail.priority_word,
            gfx: gfx.into(),
            frame,
        });
    }
    push_intro_sequence_3_sliding_sprites_sub_1C572(&mut sprite_objects, visible_elapsed);
    let sprite_objects = intro_gfx_objects_in_alloc_chain_order_loc_FD4_FE6(&sprite_objects);
    let mut sprite_entries = Vec::new();
    for object in sprite_objects {
        append_intro_sprite_entries_make_sprite_from_gfx_object_loc_129E(
            &mut sprite_entries,
            object.gfx,
            object.frame,
        );
    }
    write_sprite_table_entries(vdp, sprite_table_base, &sprite_entries);
}

#[allow(non_snake_case)]
fn intro_gfx_objects_in_alloc_chain_order_loc_FD4_FE6(
    objects: &[IntroGfxObjectForSpriteTableWithOrder],
) -> Vec<IntroGfxObjectForSpriteTableWithOrder> {
    let mut next_indices = vec![None];
    let mut priority_words = vec![0];
    for object in objects {
        let new_index = priority_words.len();
        next_indices.push(None);
        priority_words.push(object.order_key);
        let predecessor_index =
            crate::movement::gfx_object_alloc_insertion_predecessor_index_loc_FD4(
                &next_indices,
                &priority_words,
                object.order_key,
            );
        crate::movement::gfx_object_alloc_insert_loc_FE6(
            &mut next_indices,
            predecessor_index,
            new_index,
        );
    }

    let mut ordered = Vec::with_capacity(objects.len());
    for index in
        crate::level::gfx_object_sprite_build_order_loc_14A4(next_indices[0], &next_indices)
    {
        ordered.push(objects[index - 1]);
    }
    ordered
}

#[allow(non_snake_case)]
fn push_intro_sequence_3_sliding_sprites_sub_1C572(
    objects: &mut Vec<IntroGfxObjectForSpriteTableWithOrder>,
    visible_elapsed: usize,
) {
    // ASM Load_IntroSequence3 (kid.asm:29078-29088) allocates the Wild Side
    // kid roster and then the paired sub_1C572/loc_1C58C sliding door sprites.
    // Both door workers set their visible gfx slot immediately, hibernate for
    // `$3C`, then move one pixel per frame for `#$41+1` iterations.
    // ASM sub_1C572 / loc_1C58C allocates both workers with `$02000004`.
    // The low word `4` routes through loc_FB0/loc_FB8 and pushes each worker
    // onto the `$FFFFF872` special chain, so the later right-hand worker is
    // the head when the door halves share the `$0200` priority word.
    for sliding in intro_sliding_sprites_in_special_chain_order_loc_F9C_FB8(
        crate::boot::title_menu_sliding_sprites_sub_1C572(),
    ) {
        // ASM sub_1C572/loc_1C58C pushes `#$3C` before `Hibernate_Object`;
        // oracle sprite tables show the first visible door-position delta two
        // frames later than the raw hibernate count once the VDP table is
        // sampled after Load_IntroSequence3's hidden rebuild shell.
        let moving_elapsed =
            visible_elapsed.saturating_sub(usize::from(sliding.initial_hibernate_frames) + 2);
        let moved_frames = moving_elapsed.min(usize::from(sliding.moving_frames));
        let x_pos = i32::from(sliding.x_pos)
            + i32::from(sliding.x_velocity_per_frame) * moved_frames as i32;
        let (sprite_index, width, height) = match sliding.sprite_label {
            "unk_E10A6" => (0x030, 0x28, 0x30),
            "unk_E10AE" => (0x04E, 0x28, 0x30),
            _ => continue,
        };
        objects.push(IntroGfxObjectForSpriteTableWithOrder {
            order_key: 0x0200,
            gfx: IntroGfxObjectForSpriteTable {
                x_pos: x_pos as u16,
                y_pos: sliding.y_pos,
                palette_line: 0,
                vram_tile: sliding.vram_tile,
                priority: 0,
                x_direction: 0,
            },
            frame: IntroSpriteFrameVram {
                sprite_index,
                x_center: 0,
                y_center: 0,
                width,
                height,
            },
        });
    }
}

#[allow(non_snake_case)]
fn intro_sliding_sprites_in_special_chain_order_loc_F9C_FB8(
    sprites: [crate::boot::TitleMenuSlidingSpriteSub1C572; 2],
) -> Vec<crate::boot::TitleMenuSlidingSpriteSub1C572> {
    let mut next_indices = vec![None; sprites.len()];
    let mut unknown_f872_head_index = None;

    for (new_index, sliding) in sprites.iter().enumerate() {
        let allocation_tail =
            crate::movement::gfx_object_alloc_tail_loc_FBE(sliding.gfx_slot_base, 0, 0);
        if crate::movement::gfx_object_alloc_special_chain_loc_F9C(allocation_tail.object_kind_word)
            == crate::movement::GfxObjectAllocationSpecialChainLocF9C::UnknownF872
        {
            let splice = crate::movement::gfx_object_alloc_special_chain_splice_loc_FB8(
                unknown_f872_head_index,
                new_index,
            );
            next_indices[new_index] = splice.new_gfx_special_next_index;
            unknown_f872_head_index = Some(splice.new_special_chain_head_index);
        }
    }

    let mut ordered = Vec::with_capacity(sprites.len());
    let mut cursor = unknown_f872_head_index;
    while let Some(index) = cursor {
        ordered.push(sprites[index]);
        cursor = next_indices[index];
    }
    ordered
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct IntroSequence5GfxObjectLoc1BCD0 {
    x_pos: u16,
    y_pos: u16,
    vram_tile: u16,
    frame: IntroSpriteFrameVram,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum IntroSequence5ParticleLineage {
    TimeoutFc82Zero,
    Shared,
    TimeoutFc82OneStartSkip,
    TimeoutFc82Two,
}

impl IntroSequence5ParticleLineage {
    fn from_worker_lineage(title_variant: u8, worker_lineage: IntroSequence5WorkerLineage) -> Self {
        if worker_lineage == IntroSequence5WorkerLineage::TimeoutFc82OneStartSkip {
            return Self::TimeoutFc82OneStartSkip;
        }
        match title_variant {
            0 => Self::TimeoutFc82Zero,
            2 => Self::TimeoutFc82Two,
            _ => Self::Shared,
        }
    }

    fn presentation_delay(self) -> usize {
        usize::from(self == Self::TimeoutFc82Two)
    }

    fn simulation_phase(self) -> usize {
        usize::from(matches!(
            self,
            Self::TimeoutFc82Zero | Self::TimeoutFc82OneStartSkip
        ))
    }

    fn particle_order(self) -> [usize; 3] {
        match self {
            Self::TimeoutFc82Zero | Self::TimeoutFc82OneStartSkip | Self::TimeoutFc82Two => {
                [2, 1, 0]
            }
            Self::Shared => [0, 1, 2],
        }
    }

    fn gravity_steps(self, frames: i32) -> i32 {
        if matches!(self, Self::TimeoutFc82Zero | Self::TimeoutFc82OneStartSkip) {
            frames.saturating_sub(1)
        } else {
            frames + 1
        }
    }
}

#[allow(non_snake_case)]
fn overlay_intro_sequence_5_sprite_table_loc_1BCD0(
    vdp: &mut VdpState,
    title_variant: u8,
    elapsed: usize,
    worker_lineage: IntroSequence5WorkerLineage,
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
    let visible_particle_start = INTRO_SEQUENCE_5_PARTICLE_START_ELAPSED_LOC_1BCD0
        + INTRO_SEQUENCE_5_PARTICLE_VISIBLE_DELAY_LOC_1BCD0;
    // ASM loc_1BE3C/loc_1BE88 load moving GfxObjects, then immediately reach
    // Hibernate_Object_1Frame before the first sprite-table image is exposed.
    // Oracle frame 28022 therefore contains the post-hibernate positions, not
    // the allocation seeds represented by simulation frame zero.
    // The repeated timeout visits reach the same worker through different
    // Object/SAT presentation lineages. FC82=0 exposes one advanced worker
    // pass. FC82=2 keeps the pre-particle SAT for one final trace, then exposes
    // the newest allocated worker first without rewinding its simulation.
    let particle_lineage =
        IntroSequence5ParticleLineage::from_worker_lineage(title_variant, worker_lineage);
    let particle_elapsed = elapsed
        .checked_sub(visible_particle_start)
        .filter(|_| {
            elapsed >= visible_particle_start.saturating_add(particle_lineage.presentation_delay())
        })
        .map(|elapsed| elapsed.saturating_add(particle_lineage.simulation_phase()));
    if let Some(particle_elapsed) = particle_elapsed {
        append_intro_sequence_5_particles_loc_1BE3C(
            &mut sprite_entries,
            particle_elapsed,
            particle_lineage,
        );
        append_intro_sequence_5_center_particle_loc_1BE88(
            &mut sprite_entries,
            title_variant,
            particle_elapsed,
        );
    }
    let static_removal_elapsed = visible_particle_start
        .saturating_sub(1)
        .saturating_add(particle_lineage.presentation_delay());
    let static_start_index = if elapsed >= static_removal_elapsed {
        2
    } else {
        0
    };
    for object in objects.iter().skip(static_start_index) {
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
    write_sprite_table_entries(
        vdp,
        sprite_table_base_byte_addr(&vdp.registers),
        &sprite_entries,
    );
}

const INTRO_SEQUENCE_5_PARTICLE_START_ELAPSED_LOC_1BCD0: usize = 0x01A0;
// ASM loc_1BCD0 spawns the particle workers after the `$1A0` hibernate, but
// cold-boot oracle frames keep the pre-particle pose at 2280 and only expose
// the next visible intro-5 pose at 2340.
const INTRO_SEQUENCE_5_PARTICLE_VISIBLE_DELAY_LOC_1BCD0: usize = 0x3C;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct IntroSequence5ParticleLoc1BE3C {
    x_pos: i32,
    y_pos: i32,
    x_vel: i32,
    y_vel: i32,
    loop_count: usize,
    frame: IntroSpriteFrameVram,
}

#[allow(non_snake_case)]
fn append_intro_sequence_5_particles_loc_1BE3C(
    entries: &mut Vec<HardwareSpriteEntry>,
    particle_elapsed: usize,
    lineage: IntroSequence5ParticleLineage,
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
            frame: intro_sprite_frame_vram(0x066, 0x2A, 0x37),
        },
        IntroSequence5ParticleLoc1BE3C {
            x_pos: 0x0058_0000,
            y_pos: 0x0068_0000,
            x_vel: 0x0004_0000,
            y_vel: -0x0002_0000,
            loop_count: 0x3C,
            frame: intro_sprite_frame_vram(0x090, 0x27, 0x2B),
        },
        IntroSequence5ParticleLoc1BE3C {
            x_pos: 0x0030_0000,
            y_pos: 0x0080_0000,
            x_vel: 0,
            y_vel: 0x0004_0000,
            loop_count: 0x28,
            frame: intro_sprite_frame_vram(0x0AE, 0x3B, 0x1B),
        },
    ];
    // ASM loc_1BCD0 allocates unk_1BE18, unk_1BE24, then unk_1BE30. Oracle
    // frames 20850 and 28022 expose the newest worker first on the repeated
    // FC82=2 and FC82=0 Object/SAT lineages respectively.
    for particle_index in lineage.particle_order() {
        let particle = particles[particle_index];
        if particle_elapsed > particle.loop_count {
            continue;
        }
        let frames = particle_elapsed as i32;
        // ASM loc_1BE74 resumes after movement for the displayed pass, then
        // adds `$4000` to y_vel. The acceleration therefore lags position by
        // one pass, as confirmed by oracle sprite-table Y values from frames
        // 28022 onward.
        let gravity_steps = lineage.gravity_steps(frames);
        let y_gravity = 0x0000_4000 * frames.saturating_mul(gravity_steps) / 2;
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
            particle.frame,
        );
    }
}

#[allow(non_snake_case)]
fn append_intro_sequence_5_center_particle_loc_1BE88(
    entries: &mut Vec<HardwareSpriteEntry>,
    title_variant: u8,
    particle_elapsed: usize,
) {
    // ASM loc_1BE88/loc_1BEBE/loc_1BECE/loc_1BF02/loc_1BF0A/loc_1BF1E
    // (kid.asm:30689): the centre helmet particle uses palette line 3,
    // priority 1, a FC82-selected position/frame on VRAM tile `$5EB`, then
    // keeps hibernating forever after the velocity loop.
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
    // ASM loc_1BF02/loc_1BF0A: DBF `#$1C` displays 29 moved frames, then adds
    // `$9000` to y_vel after each hibernate, so the visible gravity sum lags
    // by one before loc_1BF1E's stationary hibernate loop takes over.
    let moving_frames = particle_elapsed.min(0x1D) as i32;
    let y_gravity = 0x0000_9000 * moving_frames.saturating_mul(moving_frames.saturating_sub(1)) / 2;
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct IntroGfxObjectForSpriteTableWithOrder {
    order_key: u16,
    gfx: IntroGfxObjectForSpriteTable,
    frame: IntroSpriteFrameVram,
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
    let uses_x_flip = crate::NativeRuntime::sprite_uses_x_flip_loc_1292(gfx.x_direction);
    let pattern_base = crate::NativeRuntime::sprite_tile_id_loc_1238(
        (u16::from(gfx.palette_line) << 13)
            | crate::NativeRuntime::sprite_priority_bit_loc_1234(gfx.priority, true)
            | if uses_x_flip { 0x0800 } else { 0 },
        frame.sprite_index,
        gfx.vram_tile,
        false,
    );
    let base_y_screen_px =
        i32::from(gfx.y_pos) - i32::from(frame.height) + i32::from(frame.y_center) + 0x80;
    let base_y_screen_px = crate::NativeRuntime::sprite_special_y_offset_loc_1286(
        frame.sprite_index,
        base_y_screen_px,
    );
    let chunks = if uses_x_flip {
        crate::NativeRuntime::sprite_x_flipped_chunks_loc_134A(
            (i32::from(gfx.x_pos) + i32::from(frame.x_center) + 0x80) as i16,
            base_y_screen_px as i16,
            frame.width as i16,
            frame.height as i16,
        )
    } else {
        crate::NativeRuntime::sprite_normal_orientation_chunks_loc_12B8(
            (i32::from(gfx.x_pos) - i32::from(frame.x_center) + 0x80) as i16,
            base_y_screen_px as i16,
            frame.width as i16,
            frame.height as i16,
        )
    };
    let mut tile_offset = 0u16;
    for chunk in chunks {
        entries.push(HardwareSpriteEntry {
            y_pos: chunk.y_screen_px as u16,
            size_byte: chunk.size_code,
            pattern: pattern_base.wrapping_add(tile_offset),
            x_pos: chunk.x_screen_px as u16,
        });
        tile_offset = tile_offset.wrapping_add(sprite_tile_count_from_size_code(chunk.size_code));
    }
}

fn sprite_tile_count_from_size_code(size_code: u8) -> u16 {
    let width_tiles = u16::from((size_code >> 2) & 0x03) + 1;
    let height_tiles = u16::from(size_code & 0x03) + 1;
    width_tiles * height_tiles
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
        vdp.write_vram_word(entry_base + 2, u16::from_be_bytes([entry.size_byte, link]));
        vdp.write_vram_word(entry_base + 4, entry.pattern);
        vdp.write_vram_word(entry_base + 6, entry.x_pos);
    }
}

fn sprite_table_base_byte_addr(registers: &[u8; 0x20]) -> usize {
    // ASM VDPGameInitValues reg 5 = `$08`, selecting sprite table VRAM
    // byte address `$1000` in H40 mode. This mirrors wildside-md's renderer helper.
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
const SEGA_TEXT_BYTES: &[u8] = b"\x6B\x6C\x5C\x64\x64\x5D SEGA\xFF";

const INTRO_TEXT_CLEAR_TILE_WORD_LOC_1C7A0: u16 = 0x67C0;

const INTRO_TEXT_WORKER_START_ELAPSED_LOC_1C7A0: [usize; 5] = [
    45, // ASM sub_1B8F4/loc_1C7A0; cold-boot IntroText1 glyph cadence matches oracle frame 280/360 checkpoints.
    43, // ASM loc_1B9D0/loc_1C7A0; oracle keeps plane-B $E2B8 unchanged at published frame 5787, then writes the final period on 5788.
    43, // ASM sub_1B93E exposes IntroText3 one rendered frame after the hidden VDP rebuild shell.
    38, // ASM sub_1B988/loc_1C7A0; oracle VRAM writes IntroText4 glyph cells at published frames 20000, 20002, 20004, and 20006 one frame before the prior anchor.
    58, // ASM loc_1BA28 allocates IntroText5 before sub_1C204; oracle frame 1980 shows `THIS IS` but not `THE`.
];
// ASM Load_IntroSequence5 allocates loc_1BA28 and loc_1BCD0 before entering
// the shared sub_1C204 fade shell. The text worker is allocated earlier, but
// oracle frame 1934 still presents black; the first visible cold-boot frame is
// frame 1980, at local elapsed 104.
const INTRO_SEQUENCE_5_VISIBLE_ELAPSED_LOC_1BA28: usize = 104;

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

pub(crate) fn intro_text_worker_counted_frames_loc_1c7a0(segment: usize) -> Option<usize> {
    let script = intro_text_script_loc_1c7a0(segment)?;
    // ASM loc_1C7A0 -> loc_1C7AE/loc_1C7B6 (kid.asm:31404-31423): the newly
    // allocated Object is visible in the allocation-frame trace, then spends
    // one initial hibernate, each `$FE` duration, and each header-sized glyph
    // delay alive. `$FD` line breaks consume no frame; `$FF` deletes the Object
    // in the same pass that dispatches the terminator.
    let mut hibernate_frames = 1usize;
    let mut byte_index = 0usize;
    while let Some(&byte) = script.text_bytes.get(byte_index) {
        byte_index += 1;
        match byte {
            0xFF => return Some(hibernate_frames.saturating_add(1)),
            0xFD => {}
            0xFE => {
                let duration = *script.text_bytes.get(byte_index)?;
                byte_index += 1;
                hibernate_frames = hibernate_frames.saturating_add(usize::from(duration));
            }
            _ => {
                hibernate_frames =
                    hibernate_frames.saturating_add(usize::from(script.delay_frames));
            }
        }
    }
    None
}

fn title_sega_text_script_loc_1c7a0() -> IntroTextScriptLoc1C7A0 {
    // ASM SegaText (kid.asm:31398): header `(0,$67D3,0,$19,$19)` followed by
    // the copyright glyph pair, `1992 SEGA`, and `$FF`.
    IntroTextScriptLoc1C7A0 {
        base_vram_byte_address: 0x0000,
        base_tile_word: 0x67D3,
        delay_frames: 0,
        x_pos: 0x19,
        y_pos: 0x19,
        text_bytes: SEGA_TEXT_BYTES,
    }
}

fn overlay_intro_text_worker_loc_1c7a0(vdp: &mut VdpState, segment: usize, elapsed: usize) {
    let Some(script) = intro_text_script_loc_1c7a0(segment) else {
        return;
    };
    if script.base_vram_byte_address != 0xE000 {
        clear_intro_text_script_cells_loc_1c7a0(vdp, &script);
    }
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

fn overlay_title_sega_text_worker_loc_1c7a0(
    vdp: &mut VdpState,
    elapsed: usize,
    intro_return_hidden_shell: bool,
    direct_intro_sequence_5_title_return: bool,
    fresh_title_worker_lineage: bool,
) {
    // ASM loc_1B6AC hibernates 30 frames, loc_1B6CE reveals Chameleon for 40
    // frames, and loc_1B6DC spawns SegaText when its preserved counter reaches
    // `$1E`. The loc_1C7A0 worker then performs its own initial one-frame
    // hibernate before writing this zero-delay text in one pass.
    let start_elapsed = if fresh_title_worker_lineage {
        // ASM loc_119D8 and sub_1B93E -> loc_1C264 both allocate fresh title
        // objects after presenting the base upload. Obj_TitleTextKid and
        // Obj_TitleTextChameleon are visible on effective loader frame 58;
        // the newly allocated loc_1C7A0 child runs on frame 59.
        58
    } else if direct_intro_sequence_5_title_return {
        // ASM loc_1BA60 -> sub_1C246 -> Load_TitleScreen reaches the direct
        // IntroSequence5 title-return shell at the title-attract lock frame.
        // The freshly allocated SegaText worker still spends loc_1C7A0's
        // one-frame hibernate there, so oracle frame 2160 has no copyright
        // stamp and frame 2161 has the full zero-delay string.
        TITLE_ATTRACT_LOCK_FRAMES
    } else if intro_return_hidden_shell {
        // ASM sub_1C246 -> Load_TitleScreen on the cold IntroSequence6
        // Start-skip path inherits the already-running hidden loc_1B6AC /
        // loc_1B6DC cadence; oracle frame 2700 has the visible title VDP and
        // SegaText together while trace title readiness is still false.
        INTRO_SEQUENCE_6_START_SKIP_TITLE_VDP_READY_FRAMES
    } else {
        30 + 40 + 30 + 1
    };
    let Some(worker_elapsed) = elapsed.checked_sub(start_elapsed) else {
        return;
    };
    let script = title_sega_text_script_loc_1c7a0();
    replay_intro_text_worker_loc_1c7a0(vdp, &script, worker_elapsed);
}

fn clear_intro_text_script_cells_loc_1c7a0(vdp: &mut VdpState, script: &IntroTextScriptLoc1C7A0) {
    // Native asset extraction can leave the final text tilemap in the intro
    // scene maps. ASM loc_1C7A0/sub_1C7F6 writes each glyph over blank plane
    // cells at runtime, so clear the scripted cells before replaying the
    // visible prefix for this frame.
    let clear_tile_word = intro_text_clear_tile_word_loc_1c7a0(script);
    let mut byte_index = 0usize;
    let mut x_pos = script.x_pos;
    let mut y_pos = script.y_pos;
    while let Some(&raw_byte) = script.text_bytes.get(byte_index) {
        let stream_byte_index = byte_index;
        byte_index += 1;
        debug_assert!(
            byte_index > stream_byte_index,
            "ASM loc_1C7B6 advances the intro text stream before dispatching a byte",
        );
        match raw_byte {
            0xFF => return,
            0xFD => {
                x_pos = script.x_pos;
                y_pos = y_pos.wrapping_add(2);
            }
            0xFE => {
                let delay_byte_index = byte_index;
                byte_index += 1;
                debug_assert!(
                    byte_index > delay_byte_index,
                    "ASM loc_1C7D2 consumes the delay byte after `$FE`",
                );
            }
            _ => {
                let write = wildside_data::title_intro_glyph_write_sub_1C7F6(
                    script.base_vram_byte_address,
                    script.base_tile_word,
                    x_pos,
                    y_pos,
                    0,
                );
                // Native-only cleanup for extracted maps uses the clear word
                // selected by the script's base tile. The `$67D3` Wild Side
                // scripts need a zeroed high-VRAM pattern, while the `$C7D3`
                // corridor scripts match oracle with literal zero plane cells.
                vdp.write_vram_word(usize::from(write.vram_word_address), clear_tile_word);
                x_pos = x_pos.wrapping_add(1);
            }
        }
    }
}

fn intro_text_clear_tile_word_loc_1c7a0(script: &IntroTextScriptLoc1C7A0) -> u16 {
    if script.base_tile_word == 0x67D3 {
        INTRO_TEXT_CLEAR_TILE_WORD_LOC_1C7A0
    } else {
        0
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
                let write = wildside_data::title_intro_glyph_write_sub_1C7F6(
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
        write_decompress_to_vram_art_loc_14300_14314(&mut vdp, *vram_byte_addr, art_bytes);
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
    // ASM Load_IntroSequence1/sub_1B8F4 and Load_IntroSequence3/sub_1B93E
    // both call sub_1C4EA, so segment 2 reuses Pal_1C28A.
    // Segment 1 (Load_IntroSequence2) and segments 3/4 (Load_IntroSequence4/5)
    // route through their dedicated build_intro_sequence_{2,4,5}_vdp helpers
    // before this fn runs, so this match only fires for segments 0, 2, and any
    // overflow values — there is no live `1 =>` arm to test here.
    let palette = match segment {
        // ASM loc_1C4F6 (kid.asm:31232): `move.w (a0)+,(a1)+; dbf`
        // copies 0x33 words from Pal_1C28A into Palette_Buffer_2.
        0 | 2 => &data.palette_segment_0,
        _ => &data.palette_segment_2,
    };
    vdp.write_cram_words(0, palette);

    vdp
}

#[allow(non_snake_case)]
fn write_decompress_to_vram_art_loc_14300_14314(
    vdp: &mut VdpState,
    vram_byte_addr: usize,
    art_bytes: &[u8],
) {
    // ASM loc_14300/loc_14314: DecompressToVRAM snapshots the emitted
    // decompression-buffer span, converts it to an inclusive DBF word count,
    // then streams big-endian words to the VDP data port. Native keeps the
    // decoded art as bytes, so preserve any odd trailing byte after the word
    // transfer.
    for DecompressChunkSpan {
        source_start,
        byte_len,
        buffer_start_addr,
        rollover_source_end,
    } in decompress_chunk_spans(art_bytes.len())
    {
        let setup = crate::level::decompress_to_vram_chunk_setup_loc_14300(
            buffer_start_addr,
            buffer_start_addr + byte_len as u32,
        );
        let source_end = source_start + byte_len;
        let buffered_words = art_bytes[source_start..source_end]
            .chunks_exact(2)
            .map(|bytes| u16::from_be_bytes([bytes[0], bytes[1]]))
            .collect::<Vec<_>>();
        debug_assert_eq!(
            usize::from(setup.word_dbf_counter) + 1,
            buffered_words.len(),
            "ASM loc_14300/loc_14314 d3+1 word count must match the decompressed chunk span",
        );
        for (word_index, word) in crate::level::decompress_to_vram_word_transfer_loc_14314(
            &buffered_words,
            setup.word_dbf_counter,
        )
        .into_iter()
        .enumerate()
        {
            vdp.write_vram_word(vram_byte_addr + source_start + word_index * 2, word);
        }
        if let Some(rollover_source_end) = rollover_source_end {
            let upper_half_longwords = rollover_source_longwords(art_bytes, rollover_source_end);
            let rolled = crate::level::decompress_to_vram_buffer_rollover_loc_1432C(
                &upper_half_longwords,
                DECOMPRESS_ROLLOVER_LONGWORD_DBF,
            );
            debug_assert_eq!(rolled, upper_half_longwords);
        }
    }
    let even_byte_len = art_bytes.len() & !1;
    if let Some(&last_byte) = art_bytes.get(even_byte_len) {
        vdp.write_vram(vram_byte_addr + even_byte_len, &[last_byte]);
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct DecompressChunkSpan {
    source_start: usize,
    byte_len: usize,
    buffer_start_addr: u32,
    rollover_source_end: Option<usize>,
}

fn decompress_chunk_spans(byte_len: usize) -> Vec<DecompressChunkSpan> {
    let even_byte_len = byte_len & !1;
    debug_assert!(
        even_byte_len <= byte_len,
        "ASM loc_14300 streams only the even byte prefix before preserving an odd tail",
    );
    let mut source_start = 0usize;
    let mut chunk_capacity = DECOMPRESS_FIRST_CHUNK_BYTES;
    let mut spans = Vec::new();
    while source_start < even_byte_len {
        let byte_len = (even_byte_len - source_start).min(chunk_capacity);
        let source_end = source_start + byte_len;
        debug_assert!(
            source_end > source_start,
            "ASM loc_14300 chunk iteration must make forward progress",
        );
        let more_chunks_follow = source_end < even_byte_len;
        spans.push(DecompressChunkSpan {
            source_start,
            byte_len,
            buffer_start_addr: if source_start == 0 {
                0
            } else {
                DECOMPRESS_RESUMED_CHUNK_BYTES as u32
            },
            rollover_source_end: more_chunks_follow.then_some(source_end),
        });
        source_start = source_end;
        chunk_capacity = DECOMPRESS_RESUMED_CHUNK_BYTES;
    }
    spans
}

fn rollover_source_longwords(art_bytes: &[u8], source_end: usize) -> Vec<u32> {
    let source_start = source_end - DECOMPRESS_RESUMED_CHUNK_BYTES;
    art_bytes[source_start..source_end]
        .chunks_exact(4)
        .map(|bytes| u32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]))
        .collect()
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
    wildside_data::decompression_buffer_rows_upload_sub_1CD88(decoded_words)
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
        let [bottom_left_address, bottom_right_address] = write.bottom_word_addresses();
        let bottom_left_index =
            usize::from(bottom_left_address.wrapping_sub(DECOMPRESSION_BUFFER_LOW_ADDR)) / 2;
        let bottom_right_index =
            usize::from(bottom_right_address.wrapping_sub(DECOMPRESSION_BUFFER_LOW_ADDR)) / 2;
        if bottom_right_index < decoded_words.len() {
            decoded_words[bottom_left_index] = write.bottom_words[0];
            decoded_words[bottom_right_index] = write.bottom_words[1];
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
    // ASM sub_1B988 (kid.asm:30273-30282): `move.w #4,(Camera_Y_pos).w`
    // before loc_1B9B2 owns the visible corridor loop. Oracle VSRAM at frame
    // 1380 exposes plane A scrolled by 4 and plane B by 1.
    vdp.write_vsram_word(0, 4);
    vdp.write_vsram_word(2, 1);

    vdp
}

#[allow(non_snake_case)]
fn intro_sequence_2_special_sky_art_loc_1B00A(art: &[u8]) -> Vec<u8> {
    // ASM Load_IntroSequence2 loc_1B00A enters DecompressToVRAM_Special with
    // `a3 = unk_1C336`, so loc_1434A maps each packed 4bpp nibble through this
    // permutation before the art is staged in `Level_Layout` and VRAM `$1780`.
    const PERMUTATION: [u8; 16] = [
        0x02, 0x01, 0x03, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0A, 0x0B, 0x0C, 0x0D, 0x0E,
        0x0F,
    ];
    remap_4bpp_art_via_decompress_to_vram_special_loc_14348_14382(&PERMUTATION, art)
}

#[allow(non_snake_case)]
fn intro_sequence_5_text_art_loc_1B0F4(art: &[u8]) -> Vec<u8> {
    // ASM Load_IntroSequence5 (kid.asm:29352): before `j_DecompressToRAM`,
    // `a3 = unk_1C3F0`, so DecompressToRAM's loc_143CE/loc_1440A path maps
    // each unpacked 4bpp nibble through this table while streaming the letters
    // to the VDP data port at `$FA60`.
    const PERMUTATION: [u8; 16] = [
        0x00, 0x08, 0x09, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00,
    ];
    remap_4bpp_art_via_decompress_to_ram_loc_143CC_1440A(&PERMUTATION, art)
}

#[allow(non_snake_case)]
fn title_text_chameleon_art_loc_1B4F4(art: &[u8]) -> Vec<u8> {
    // ASM Load_TitleArt (kid.asm:29816): `a3 = unk_1B4F4` before
    // `j_DecompressToRAM`, so the RAM-staged Chameleon logo remaps each
    // unpacked 4bpp nibble through this table while streaming to VRAM `$6760`.
    const PERMUTATION: [u8; 16] = [
        0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0A, 0x0B, 0x0C, 0x0D, 0x00,
        0x0F,
    ];
    remap_4bpp_art_via_decompress_to_ram_loc_143CC_1440A(&PERMUTATION, art)
}

#[allow(non_snake_case)]
fn remap_4bpp_art_via_decompress_to_vram_special_loc_14348_14382(
    permutation: &[u8; 16],
    art: &[u8],
) -> Vec<u8> {
    let mut table = [0; 256];
    for row in 0..16 {
        let start = crate::level::decompress_special_palette_row_start_loc_14348(row);
        let row_bytes = crate::level::decompress_special_palette_row_bytes_loc_1434A(
            permutation,
            start.row_index,
            15,
        );
        let table_start = usize::from(start.row_index) * 16;
        table[table_start..table_start + 16].copy_from_slice(&row_bytes);
    }
    remap_4bpp_art_words_via_table_loc_14382(&table, art)
}

#[allow(non_snake_case)]
fn remap_4bpp_art_via_decompress_to_ram_loc_143CC_1440A(
    permutation: &[u8; 16],
    art: &[u8],
) -> Vec<u8> {
    let mut table = [0; 256];
    for row in 0..16 {
        let start = crate::level::decompress_to_ram_palette_row_start_loc_143CC(row);
        let row_bytes = crate::level::decompress_to_ram_palette_row_bytes_loc_143CE(
            permutation,
            start.row_index,
            start.column_index,
        );
        let table_start = usize::from(start.row_index) * 16;
        table[table_start..table_start + 16].copy_from_slice(&row_bytes);
    }
    remap_4bpp_art_words_via_table_loc_1440A(&table, art)
}

#[allow(non_snake_case)]
fn remap_4bpp_art_words_via_table_loc_14382(permutation_table: &[u8; 256], art: &[u8]) -> Vec<u8> {
    let mut remapped = Vec::with_capacity(art.len());
    for DecompressChunkSpan {
        source_start,
        byte_len,
        buffer_start_addr,
        rollover_source_end,
    } in decompress_chunk_spans(art.len())
    {
        let setup = crate::level::decompress_to_vram_special_chunk_setup_loc_14366(
            buffer_start_addr,
            buffer_start_addr + byte_len as u32,
            0,
        );
        for word in crate::level::decompress_to_vram_special_word_transfer_loc_14382(
            permutation_table,
            &art[source_start..source_start + byte_len],
            setup.word_dbf_counter,
        ) {
            remapped.extend_from_slice(&word.to_be_bytes());
        }
        if let Some(rollover_source_end) = rollover_source_end {
            let upper_half_longwords = rollover_source_longwords(art, rollover_source_end);
            let rolled = crate::level::decompress_to_vram_special_buffer_rollover_loc_143AA(
                &upper_half_longwords,
                DECOMPRESS_ROLLOVER_LONGWORD_DBF,
            );
            debug_assert_eq!(rolled, upper_half_longwords);
        }
        let return_state =
            crate::level::decompress_to_vram_special_return_loc_143B2(setup.restored_output_addr);
        debug_assert_eq!(return_state.restored_input_addr, setup.restored_output_addr);
        debug_assert_eq!(return_state.vdp_data_port_addr, 0x00C0_0000);
    }
    if let Some(&last_byte) = art.get(remapped.len()) {
        remapped.push(permutation_table[usize::from(last_byte)]);
    }
    remapped
}

#[allow(non_snake_case)]
fn remap_4bpp_art_words_via_table_loc_1440A(permutation_table: &[u8; 256], art: &[u8]) -> Vec<u8> {
    let mut remapped = Vec::with_capacity(art.len());
    for DecompressChunkSpan {
        source_start,
        byte_len,
        buffer_start_addr,
        rollover_source_end,
    } in decompress_chunk_spans(art.len())
    {
        let setup = crate::level::decompress_to_ram_chunk_setup_loc_143EA(
            buffer_start_addr,
            buffer_start_addr + byte_len as u32,
            0,
        );
        for word in crate::level::decompress_to_ram_word_transfer_loc_1440A(
            permutation_table,
            &art[source_start..source_start + byte_len],
            setup.word_dbf_counter,
        ) {
            remapped.extend_from_slice(&word.to_be_bytes());
        }
        if let Some(rollover_source_end) = rollover_source_end {
            let upper_half_longwords = rollover_source_longwords(art, rollover_source_end);
            let rolled = crate::level::decompress_to_ram_buffer_rollover_loc_14430(
                &upper_half_longwords,
                DECOMPRESS_ROLLOVER_LONGWORD_DBF,
            );
            debug_assert_eq!(rolled, upper_half_longwords);
        }
    }
    if let Some(&last_byte) = art.get(remapped.len()) {
        remapped.push(permutation_table[usize::from(last_byte)]);
    }
    remapped
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
    let intro_sequence_5_text_art =
        intro_sequence_5_text_art_loc_1B0F4(&data.intro_text_letters_art);
    vdp.write_vram(0xFA60, &intro_sequence_5_text_art);
    // ASM sub_1C512 -> loc_1C51E (kid.asm:31257-31264): copy all
    // 64 words from Pal_1C346 into Palette_Buffer_2 before any region overlay.
    vdp.write_cram_words(0, &data.palette_segment_2);
    // ASM loc_1C540/loc_1C542 (kid.asm:31273-31278): non-zero `$FFFFFC82`
    // overlays 14 words at Palette_Buffer_2+$62, using Pal_1C3C6 for 1 and
    // Pal_1C3D4 for every other non-zero variant.
    match title_variant {
        0 => {}
        1 => vdp.write_cram_words(0x31, &data.intro_sequence_5_palette_variant_1_overlay),
        _ => vdp.write_cram_words(0x31, &data.intro_sequence_5_palette_variant_2_overlay),
    }

    vdp
}

const INTRO_SEQUENCE_5_LOC_1BC36_FRAME_TIMELINE: &[(usize, u16)] = &[
    (0, 0),
    (
        0xA0 + INTRO_SEQUENCE_5_LOC_1BC36_PRE_PANEL_PRESENTATION_DELAY,
        1,
    ),
    (
        0xA0 + 0x90 + INTRO_SEQUENCE_5_LOC_1BC36_PRE_PANEL_PRESENTATION_DELAY,
        2,
    ),
    (
        0xA0 + 0x90 + 0x0B + INTRO_SEQUENCE_5_LOC_1BC36_PRE_PANEL_PRESENTATION_DELAY,
        3,
    ),
    (
        0xA0 + 0x90 + 0x0B + 0x0B + INTRO_SEQUENCE_5_LOC_1BC36_PRE_PANEL_PRESENTATION_DELAY,
        4,
    ),
    (
        0xA0 + 0x90 + 0x0B + 0x0B + 0x0B + INTRO_SEQUENCE_5_LOC_1BC36_POST_PANEL_PRESENTATION_DELAY,
        5,
    ),
    (INTRO_SEQUENCE_5_LOC_1BC36_FRAME_6_RASTER_ELAPSED, 6),
    (INTRO_SEQUENCE_5_LOC_1BC36_FRAME_7_RASTER_ELAPSED, 7),
    (INTRO_SEQUENCE_5_LOC_1BC36_RETURN_FRAME_6_RASTER_ELAPSED, 6),
    (INTRO_SEQUENCE_5_LOC_1BC36_FINAL_FRAME_5_RASTER_ELAPSED, 5),
];
// ASM loc_1BC36 calls sub_1C034 on these hibernate boundaries. Exact oracle
// FIFO traces show the first four uploads entering active display after `$3A`;
// the combined d4=4/sub_1C0EE pass crosses VBlank and leaves later uploads one
// presented frame behind that clock.
const INTRO_SEQUENCE_5_LOC_1BC36_PRE_PANEL_PRESENTATION_DELAY: usize = 0x3A;
const INTRO_SEQUENCE_5_LOC_1BC36_POST_PANEL_PRESENTATION_DELAY: usize = 0x3B;
const INTRO_SEQUENCE_5_LOC_1BC36_FRAME_5_VISIBLE_ELAPSED: usize =
    0xA0 + 0x90 + 0x0B + 0x0B + 0x0B + INTRO_SEQUENCE_5_LOC_1BC36_POST_PANEL_PRESENTATION_DELAY;
const INTRO_SEQUENCE_5_LOC_1BC36_FRAME_6_RASTER_ELAPSED: usize =
    INTRO_SEQUENCE_5_LOC_1BC36_FRAME_5_VISIBLE_ELAPSED + 0x37;
const INTRO_SEQUENCE_5_LOC_1BC36_FRAME_7_RASTER_ELAPSED: usize =
    INTRO_SEQUENCE_5_LOC_1BC36_FRAME_6_RASTER_ELAPSED + 0x08;
const INTRO_SEQUENCE_5_LOC_1BC36_RETURN_FRAME_6_RASTER_ELAPSED: usize =
    INTRO_SEQUENCE_5_LOC_1BC36_FRAME_7_RASTER_ELAPSED + 0x08;
const INTRO_SEQUENCE_5_LOC_1BC36_FINAL_FRAME_5_RASTER_ELAPSED: usize =
    INTRO_SEQUENCE_5_LOC_1BC36_RETURN_FRAME_6_RASTER_ELAPSED + 0x0F;
// ASM loc_1BC36 sub_1C034(d4=4) -> sub_1C0EE presents the panel from exact
// cold frames 2260..2271 and timeout-restart frames 16555..16566.
const INTRO_SEQUENCE_5_LOC_1BC36_PANEL_VISIBLE_ELAPSED: usize =
    0xA0 + 0x90 + 0x0B + 0x0B + INTRO_SEQUENCE_5_LOC_1BC36_PRE_PANEL_PRESENTATION_DELAY;
const INTRO_SEQUENCE_5_LOC_1BC36_PANEL_CLEAR_ELAPSED: usize =
    INTRO_SEQUENCE_5_LOC_1BC36_FRAME_5_VISIBLE_ELAPSED;
const INTRO_SEQUENCE_5_LOC_1BC36_FRAME_1_RASTER_ELAPSED: usize =
    0xA0 + INTRO_SEQUENCE_5_LOC_1BC36_PRE_PANEL_PRESENTATION_DELAY;
const INTRO_SEQUENCE_5_LOC_1BC36_FRAME_2_RASTER_ELAPSED: usize =
    0xA0 + 0x90 + INTRO_SEQUENCE_5_LOC_1BC36_PRE_PANEL_PRESENTATION_DELAY;
const INTRO_SEQUENCE_5_LOC_1BC36_FRAME_3_RASTER_ELAPSED: usize =
    0xA0 + 0x90 + 0x0B + INTRO_SEQUENCE_5_LOC_1BC36_PRE_PANEL_PRESENTATION_DELAY;
const INTRO_SEQUENCE_5_LOC_1BC36_FRAME_4_RASTER_ELAPSED: usize =
    0xA0 + 0x90 + 0x0B + 0x0B + INTRO_SEQUENCE_5_LOC_1BC36_PRE_PANEL_PRESENTATION_DELAY;

const H40_VRAM_FIFO_ACCESS_PIXELS: [u16; 18] = [
    12, 28, 44, 76, 92, 108, 140, 156, 172, 204, 220, 236, 268, 284, 300, 330, 332, 380,
];
const MCLK_CYCLES_PER_SCANLINE: u32 = 3_420;
const MCLK_CYCLES_PER_M68K_CYCLE: u32 = 7;
const NTSC_SCANLINES_PER_FRAME: u16 = 262;

/// Exact H40 beam position, retaining the master-clock sub-pixel phase that a
/// scanline/pixel pair discards.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct H40BeamPosition {
    scanline: u16,
    scanline_mclk: u16,
}

impl H40BeamPosition {
    pub const fn new(scanline: u16, scanline_mclk: u16) -> Option<Self> {
        if scanline < NTSC_SCANLINES_PER_FRAME && scanline_mclk < MCLK_CYCLES_PER_SCANLINE as u16 {
            Some(Self {
                scanline,
                scanline_mclk,
            })
        } else {
            None
        }
    }

    pub const fn scanline(self) -> u16 {
        self.scanline
    }

    pub const fn scanline_mclk(self) -> u16 {
        self.scanline_mclk
    }

    #[must_use]
    pub fn advance_mclk(self, cycles: crate::GenesisMclkCount) -> Self {
        let mclk = u32::from(self.scanline_mclk)
            .checked_add(cycles.get())
            .expect("bounded raster timing must fit u32 master clocks");
        Self {
            scanline: self
                .scanline
                .wrapping_add((mclk / MCLK_CYCLES_PER_SCANLINE) as u16)
                % NTSC_SCANLINES_PER_FRAME,
            scanline_mclk: (mclk % MCLK_CYCLES_PER_SCANLINE) as u16,
        }
    }

    #[must_use]
    pub fn advance_m68k_cycles(self, cycles: M68kCycleCount) -> Self {
        let mclk = u32::from(self.scanline_mclk)
            + cycles
                .get()
                .checked_mul(MCLK_CYCLES_PER_M68K_CYCLE)
                .expect("bounded HUD timing must fit u32 master clocks");
        Self {
            scanline: self
                .scanline
                .wrapping_add((mclk / MCLK_CYCLES_PER_SCANLINE) as u16)
                % NTSC_SCANLINES_PER_FRAME,
            scanline_mclk: (mclk % MCLK_CYCLES_PER_SCANLINE) as u16,
        }
    }
}

/// Convert an absolute elapsed 68000-cycle count into the NTSC H40 beam.
pub fn h40_beam_from_absolute_m68k_cycles(cycles: M68kCycleCount) -> H40BeamPosition {
    const MCLK_PER_M68K_CYCLE: u64 = 7;
    let frame_mclk = u64::from(MCLK_CYCLES_PER_SCANLINE) * u64::from(NTSC_SCANLINES_PER_FRAME);
    let beam_mclk = u64::from(cycles.get()) * MCLK_PER_M68K_CYCLE % frame_mclk;
    H40BeamPosition {
        scanline: u16::try_from(beam_mclk / u64::from(MCLK_CYCLES_PER_SCANLINE))
            .expect("NTSC scanline fits u16"),
        scanline_mclk: u16::try_from(beam_mclk % u64::from(MCLK_CYCLES_PER_SCANLINE))
            .expect("H40 scanline master clock fits u16"),
    }
}

/// Master-clock cursor relative to the VBlank-to-VBlank scanline-224 origin.
pub fn ntsc_vblank_cursor_from_absolute_m68k_cycles(
    cycles: M68kCycleCount,
) -> crate::GenesisMclkCount {
    ntsc_vblank_cursor_from_h40_beam(h40_beam_from_absolute_m68k_cycles(cycles))
}

/// Master-clock cursor for an H40 beam relative to the VBlank-to-VBlank
/// scanline-224 origin.
pub fn ntsc_vblank_cursor_from_h40_beam(beam: H40BeamPosition) -> crate::GenesisMclkCount {
    const VBLANK_SCANLINE: u32 = 224;
    let frame_mclk = MCLK_CYCLES_PER_SCANLINE * u32::from(NTSC_SCANLINES_PER_FRAME);
    let absolute_mclk =
        u32::from(beam.scanline) * MCLK_CYCLES_PER_SCANLINE + u32::from(beam.scanline_mclk);
    let vblank_origin_mclk = VBLANK_SCANLINE * MCLK_CYCLES_PER_SCANLINE;
    crate::GenesisMclkCount::from_raw(
        absolute_mclk
            .checked_add(frame_mclk)
            .expect("one bounded NTSC frame fits u32")
            .checked_sub(vblank_origin_mclk)
            .expect("VBlank origin is within the NTSC frame")
            % frame_mclk,
    )
}

/// Explicit hardware boundary at ASM `Execute_Objects sub_E12`.
///
/// This is a seed, not a frame selector: callers must obtain it from a
/// persistent CPU/VDP scheduler or a bounded hardware checkpoint.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ModeLevelRasterSeedSubE12 {
    pub beam: H40BeamPosition,
    pub refresh_phase: M68kRefreshPhase,
}

/// CPU/beam scheduler state at a classified point in `Mode_Level`.
///
/// The current cursor is valid through VDP-idle intervals. Classified
/// transitions which cross port writes must schedule their FIFO ownership
/// explicitly.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ModeLevelCpuRasterCursor {
    pub beam: H40BeamPosition,
    pub refresh_phase: M68kRefreshPhase,
}

/// CPU/beam scheduler state at ASM `Load_InGame` source boundaries.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LoadInGameCpuRasterCursor {
    pub beam: H40BeamPosition,
    pub refresh_phase: M68kRefreshPhase,
}

/// CPU/raster result of one ASM `DecompressToVRAM loc_14314` transfer.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DecompressToVramTransferAdvance {
    pub cursor: LoadInGameCpuRasterCursor,
    pub fifo_halted_ticks: M68kCycleCount,
    pub refresh_events: u32,
    pub vint_serviced: bool,
    pub pending_external_wait_cycles: M68kCycleCount,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DecompressToVramTransferExit {
    Return,
    CompactAndResume,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DecompressToVramDecoderAdvance {
    pub cursor: LoadInGameCpuRasterCursor,
    pub timing: crate::saxman_timing::DecompressChunkTimingSub13684,
    pub resume: Option<crate::saxman_timing::DecompressChunkResumeSub13684>,
    pub refresh_events: u32,
    pub vint_serviced: bool,
    pub external_wait_cycles: M68kCycleCount,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DecompressToVramSetupAdvance {
    pub cursor: LoadInGameCpuRasterCursor,
    pub refresh_events: u32,
    pub external_wait_cycles: M68kCycleCount,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EnigmaDecoderAdvance {
    pub cursor: LoadInGameCpuRasterCursor,
    pub timing: crate::enigma_timing::EnigmaTiming,
    pub refresh_events: u32,
    pub vint_serviced: bool,
    pub external_wait_cycles: M68kCycleCount,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TitleIntroSourceAdvance {
    pub cursor: LoadInGameCpuRasterCursor,
    pub timing: crate::intro_timing::IntroSourceTiming,
    pub refresh_events: u32,
    pub external_wait_cycles: M68kCycleCount,
}

/// Persistent hardware result from the first title `WaitForVint` poll to
/// `Mode_Standard` entry.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TitleWaitForVintAdvance {
    pub cursor: LoadInGameCpuRasterCursor,
    pub resume: crate::WaitForVintResumePath,
    pub scheduler_tick_count: u32,
    pub raw_cycles: M68kCycleCount,
    pub refresh_events: u32,
    pub post_assertion_refresh_events: u32,
    pub external_wait_cycles: M68kCycleCount,
}

/// Load_InGame name retained for the platform initialisation boundary.
pub type LoadInGamePlatformArtTransferAdvance = DecompressToVramTransferAdvance;

/// VINT position within the vertical platform-art decoder's pure source work.
///
/// The CPU scheduler supplies this instruction-boundary offset when scanline
/// 224 asserts during `Decompress_Chunk sub_13684`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LoadInGamePlatformVintEvent {
    pub vertical_decoder_raw_offset: M68kCycleCount,
}

/// Instruction-boundary refresh ledger for the non-transfer source spans in
/// ASM `Initialize_Platforms`.
///
/// `loc_14314` transfer refreshes are derived internally because that loop is
/// already visited instruction by instruction. The branch-heavy
/// `Decompress_Chunk` spans remain explicit until their compressed-command
/// visitor supplies the same counts directly.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LoadInGamePlatformRefreshLedger {
    pub prelude: u16,
    pub horizontal_decoder: u16,
    pub between_uploads: u16,
    pub vertical_decoder_before_vint: u16,
    pub vint_exception: u16,
    pub vint_handler: u16,
    pub vertical_decoder_after_vint: u16,
    pub epilogue: u16,
}

/// CPU/raster result from `Load_InGame $11A0E` through `$11A12`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LoadInGamePlatformInitialisationAdvance {
    pub cursor: LoadInGameCpuRasterCursor,
    pub elapsed: M68kCycleCount,
    pub horizontal_fifo_halted_ticks: M68kCycleCount,
    pub vertical_fifo_halted_ticks: M68kCycleCount,
    pub refresh_events: u32,
}

#[derive(Debug, Clone, Copy)]
struct PlatformArtFifoEntry {
    latency: u8,
    first_vram_slot: bool,
}

#[derive(Debug, Clone, Copy)]
struct MemoryToVdpDma {
    words_remaining: u16,
    latency: u8,
    target: crate::intro_timing::MemoryToVdpDmaTarget,
}

#[derive(Debug)]
struct PlatformArtFifoScheduler {
    beam: H40BeamPosition,
    entries: std::collections::VecDeque<PlatformArtFifoEntry>,
    pending_words: u8,
    memory_dma: Option<MemoryToVdpDma>,
}

fn h40_in_vblank_at_access_slot(scanline: u16, pixel: u16) -> bool {
    // jgenesis VDP HBlankStart latches VBlank after the active scanline 223
    // access at pixel 330, then clears it at the same point on scanline 260.
    matches!(scanline, 224..=259)
        || (scanline == 223 && pixel >= 332)
        || (scanline == 260 && pixel < 332)
}

impl PlatformArtFifoScheduler {
    fn new(beam: H40BeamPosition) -> Self {
        Self {
            beam,
            entries: std::collections::VecDeque::with_capacity(4),
            pending_words: 0,
            memory_dma: None,
        }
    }

    fn push_word(&mut self) {
        self.push_word_with_latency(3, true);
    }

    fn push_word_with_latency(&mut self, latency: u8, first_vram_slot: bool) {
        if self.entries.len() == 4 {
            self.pending_words = self
                .pending_words
                .checked_add(1)
                .expect("VDP pending-word count stays bounded");
        } else {
            self.entries.push_back(PlatformArtFifoEntry {
                latency,
                first_vram_slot,
            });
        }
    }

    fn begin_memory_dma(&mut self, words: u16, target: crate::intro_timing::MemoryToVdpDmaTarget) {
        assert_ne!(words, 0, "zero-length VDP DMA means 65,536 words");
        assert!(self.memory_dma.is_none(), "VDP DMA cannot overlap");
        self.memory_dma = Some(MemoryToVdpDma {
            words_remaining: words,
            latency: match target {
                crate::intro_timing::MemoryToVdpDmaTarget::Vsram => 5,
                crate::intro_timing::MemoryToVdpDmaTarget::Vram
                | crate::intro_timing::MemoryToVdpDmaTarget::Cram => 8,
            },
            target,
        });
    }

    fn memory_dma_active(&self) -> bool {
        self.memory_dma.is_some()
    }

    fn advance_m68k_cycles(&mut self, cycles: M68kCycleCount) {
        for _ in 0..cycles.get().saturating_mul(MCLK_CYCLES_PER_M68K_CYCLE) {
            let previous_scanline = self.beam.scanline;
            let previous_pixel = h40_scanline_mclk_to_pixel(self.beam.scanline_mclk);
            self.beam = self.beam.advance_mclk(crate::GenesisMclkCount::from_raw(1));
            if self.beam.scanline != previous_scanline {
                continue;
            }
            let current_pixel = h40_scanline_mclk_to_pixel(self.beam.scanline_mclk);
            if current_pixel == previous_pixel {
                continue;
            }
            debug_assert_eq!(
                current_pixel,
                previous_pixel + 1,
                "H40 pixel clock advances one pixel at a time"
            );
            let pixel = previous_pixel;
            if !pixel.is_multiple_of(2) || pixel > 418 {
                continue;
            }
            let slot = pixel / 2;
            let refresh = matches!(slot, 26 | 58 | 90 | 122 | 154 | 204);
            let blank = h40_in_vblank_at_access_slot(self.beam.scanline, pixel);
            let access = if blank {
                !refresh
            } else {
                matches!(
                    slot,
                    6 | 14
                        | 22
                        | 38
                        | 46
                        | 54
                        | 70
                        | 78
                        | 86
                        | 102
                        | 110
                        | 118
                        | 134
                        | 142
                        | 150
                        | 165
                        | 166
                        | 190
                )
            };

            if !refresh {
                let previous_refresh =
                    slot != 0 && matches!(slot - 1, 26 | 58 | 90 | 122 | 154 | 204);
                let fifo_full = self.entries.len() == 4;
                if let Some(dma) = &mut self.memory_dma {
                    dma.latency = dma.latency.saturating_sub(1);
                    if !fifo_full && !previous_refresh {
                        let latency = dma.latency.max(3);
                        let first_vram_slot =
                            matches!(dma.target, crate::intro_timing::MemoryToVdpDmaTarget::Vram);
                        dma.words_remaining -= 1;
                        let complete = dma.words_remaining == 0;
                        self.entries.push_back(PlatformArtFifoEntry {
                            latency,
                            first_vram_slot,
                        });
                        if complete {
                            self.memory_dma = None;
                        }
                    }
                }
            }

            if access && self.entries.front().is_some_and(|entry| entry.latency == 0) {
                let front = self.entries.front_mut().expect("front checked above");
                if front.first_vram_slot {
                    front.first_vram_slot = false;
                } else {
                    self.entries.pop_front();
                    if self.pending_words != 0 {
                        self.entries.push_back(PlatformArtFifoEntry {
                            latency: 3,
                            first_vram_slot: true,
                        });
                        self.pending_words -= 1;
                    }
                }
            }

            if !refresh {
                for entry in &mut self.entries {
                    entry.latency = entry.latency.saturating_sub(1);
                }
            }
        }
    }
}

/// Hardware action emitted while scheduling a title memory-DMA routine.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TitleMemoryDmaHardwareEvent {
    Advance {
        cycles: M68kCycleCount,
        vdp_owns_bus: bool,
        source_instruction: bool,
    },
    SetZ80BusRequest(bool),
}

/// CPU/raster result from one title memory-DMA routine.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TitleMemoryDmaAdvance {
    pub cursor: LoadInGameCpuRasterCursor,
    pub timing: crate::intro_timing::IntroSourceTiming,
    pub refresh_events: u32,
    pub external_wait_cycles: M68kCycleCount,
    pub fifo_halt_cycles: M68kCycleCount,
    pub dma_halt_cycles: M68kCycleCount,
    pub vint_events: u8,
}

pub type TitlePaletteHardwareEvent = TitleMemoryDmaHardwareEvent;
pub type TitlePaletteToVramAdvance = TitleMemoryDmaAdvance;

fn advance_decompress_to_vram_source_instruction(
    raw_cycles: u32,
    fifo: &mut PlatformArtFifoScheduler,
    phase: &mut M68kRefreshPhase,
    refresh_events: &mut u32,
    drain_generated_wait: bool,
    advance_hardware: &mut dyn FnMut(M68kCycleCount, bool) -> Option<M68kCycleCount>,
) -> Option<M68kCycleCount> {
    let phase_after_raw = u32::from(phase.get()).checked_add(raw_cycles)?;
    let refresh_wait_cycles = if phase_after_raw >= 128 { 2 } else { 0 };
    *phase = M68kRefreshPhase::new(
        u8::try_from(phase_after_raw % 128).expect("refresh phase remains below 128"),
    )
    .expect("refresh phase remains below 128");
    *refresh_events = refresh_events.checked_add(refresh_wait_cycles / 2)?;

    let raw = M68kCycleCount::from_raw(raw_cycles);
    fifo.advance_m68k_cycles(raw);
    let mut pending_wait = advance_hardware(raw, false)?
        .get()
        .checked_add(refresh_wait_cycles)?;
    if !drain_generated_wait {
        return Some(M68kCycleCount::from_raw(pending_wait));
    }
    while pending_wait != 0 {
        let wait = M68kCycleCount::from_raw(pending_wait);
        fifo.advance_m68k_cycles(wait);
        *phase = phase.advance_external_wait(wait).phase_after;
        pending_wait = advance_hardware(wait, false)?.get();
    }
    Some(M68kCycleCount::from_raw(0))
}

fn advance_decompress_source_instruction_with_interval_kind(
    raw_cycles: u32,
    fifo: &mut PlatformArtFifoScheduler,
    phase: &mut M68kRefreshPhase,
    refresh_events: &mut u32,
    advance_hardware: &mut dyn FnMut(M68kCycleCount, bool, bool) -> Option<M68kCycleCount>,
) -> Option<u32> {
    let phase_after_raw = u32::from(phase.get()).checked_add(raw_cycles)?;
    let refresh_wait_cycles = if phase_after_raw >= 128 { 2 } else { 0 };
    *phase = M68kRefreshPhase::new(
        u8::try_from(phase_after_raw % 128).expect("refresh phase remains below 128"),
    )
    .expect("refresh phase remains below 128");
    *refresh_events = refresh_events.checked_add(refresh_wait_cycles / 2)?;

    let raw = M68kCycleCount::from_raw(raw_cycles);
    fifo.advance_m68k_cycles(raw);
    let mut generated_wait_cycles = advance_hardware(raw, false, true)?.get();
    let mut pending_wait = generated_wait_cycles.checked_add(refresh_wait_cycles)?;
    while pending_wait != 0 {
        let wait = M68kCycleCount::from_raw(pending_wait);
        fifo.advance_m68k_cycles(wait);
        *phase = phase.advance_external_wait(wait).phase_after;
        let chained_wait = advance_hardware(wait, false, false)?.get();
        generated_wait_cycles = generated_wait_cycles.checked_add(chained_wait)?;
        pending_wait = chained_wait;
    }
    Some(generated_wait_cycles)
}

fn drain_decompress_to_vram_wait(
    mut pending_wait: M68kCycleCount,
    fifo: &mut PlatformArtFifoScheduler,
    phase: &mut M68kRefreshPhase,
    advance_hardware: &mut dyn FnMut(M68kCycleCount, bool) -> Option<M68kCycleCount>,
) -> Option<()> {
    while pending_wait.get() != 0 {
        // jgenesis evaluates `should_halt_cpu` after the VDP-port MOVE has
        // enqueued its word. A simultaneous GEMS wait therefore inherits
        // FIFO ownership for the complete 68000 wait instruction.
        let vdp_owns_bus = fifo.pending_words != 0;
        fifo.advance_m68k_cycles(pending_wait);
        *phase = phase.advance_external_wait(pending_wait).phase_after;
        pending_wait = advance_hardware(pending_wait, vdp_owns_bus)?;
        if vdp_owns_bus && pending_wait.get() != 0 {
            return None;
        }
    }
    Some(())
}

#[derive(Debug, Clone, Copy)]
struct DecompressToVramVintTracker {
    remaining: u8,
    armed: bool,
}

impl DecompressToVramVintTracker {
    fn new(remaining: u8) -> Self {
        Self {
            remaining,
            armed: true,
        }
    }

    fn rearm_after_frame_wrap(&mut self, scanline: u16) {
        if !self.armed && scanline < 224 {
            self.armed = true;
        }
    }
}

fn service_decompress_to_vram_vint_if_pending(
    vint: &mut DecompressToVramVintTracker,
    fifo: &mut PlatformArtFifoScheduler,
    phase: &mut M68kRefreshPhase,
    refresh_events: &mut u32,
    advance_hardware: &mut dyn FnMut(M68kCycleCount, bool) -> Option<M68kCycleCount>,
) -> Option<bool> {
    vint.rearm_after_frame_wrap(fifo.beam.scanline);
    if vint.remaining == 0 || !vint.armed || fifo.beam.scanline < 224 {
        return Some(false);
    }

    // jgenesis instruction timing at ASM loc_14314: sample the level-6
    // interrupt before the next opcode, enter the autovector, then execute
    // V_Int $390..$3A2 (ADDQ/ST/RTE).
    for raw_cycles in [10, 44, 28, 12, 20] {
        advance_decompress_to_vram_source_instruction(
            raw_cycles,
            fifo,
            phase,
            refresh_events,
            true,
            advance_hardware,
        )?;
    }
    vint.remaining -= 1;
    vint.armed = false;
    Some(true)
}

/// Co-schedule ASM `DecompressToVRAM loc_14300` through the first
/// `loc_14314` transfer entry, servicing the requested recurring VINT count.
///
/// The callback sees the 22-cycle call prelude, every instruction selected by
/// the Saxman source stream in `Decompress_Chunk sub_13684`, and the 36-cycle
/// result-size setup. It returns newly queued external 68000 waits, which are
/// drained at the current instruction boundary.
pub fn decompress_to_vram_decoder_cursor_loc_14300_with_scheduler(
    entry: LoadInGameCpuRasterCursor,
    compressed: &[u8],
    resume: Option<crate::saxman_timing::DecompressChunkResumeSub13684>,
    vint_count: u8,
    advance_hardware: impl FnMut(M68kCycleCount, bool, bool) -> Option<M68kCycleCount>,
) -> Option<DecompressToVramDecoderAdvance> {
    decompress_saxman_decoder_cursor_with_scheduler(
        entry,
        compressed,
        resume,
        vint_count,
        &[4, 18],
        &[12, 4, 8, 8, 4],
        advance_hardware,
    )
}

/// Co-schedule ASM `DecompressToRAM loc_143EA` through the first
/// `loc_1440A` remapped-word transfer entry.
///
/// The Saxman body is shared with `loc_14300`; this path adds the two
/// `moveq/movea` instructions which seed `d0/a3` before the common
/// `move.l a2,a4; bsr.w Decompress_Chunk` pair.
pub fn decompress_to_ram_decoder_cursor_loc_143ea_with_scheduler(
    entry: LoadInGameCpuRasterCursor,
    compressed: &[u8],
    resume: Option<crate::saxman_timing::DecompressChunkResumeSub13684>,
    vint_count: u8,
    advance_hardware: impl FnMut(M68kCycleCount, bool, bool) -> Option<M68kCycleCount>,
) -> Option<DecompressToVramDecoderAdvance> {
    decompress_saxman_decoder_cursor_with_scheduler(
        entry,
        compressed,
        resume,
        vint_count,
        &[4, 4, 4, 18],
        &[12, 4, 8, 8, 4, 12, 4],
        advance_hardware,
    )
}

fn decompress_saxman_decoder_cursor_with_scheduler(
    entry: LoadInGameCpuRasterCursor,
    compressed: &[u8],
    resume: Option<crate::saxman_timing::DecompressChunkResumeSub13684>,
    vint_count: u8,
    entry_instruction_cycles: &[u32],
    exit_instruction_cycles: &[u32],
    mut advance_hardware: impl FnMut(M68kCycleCount, bool, bool) -> Option<M68kCycleCount>,
) -> Option<DecompressToVramDecoderAdvance> {
    let mut fifo = PlatformArtFifoScheduler::new(entry.beam);
    let mut phase = entry.refresh_phase;
    let mut refresh_events = 0_u32;
    let mut external_wait_cycles = 0_u32;
    let mut scheduling_failed = false;
    let mut vint_serviced = false;
    let mut vint = DecompressToVramVintTracker::new(vint_count);

    let mut advance_instruction = |raw_cycles: u32| {
        if scheduling_failed {
            return;
        }
        let mut source_cycles = [0_u32; 5];
        vint.rearm_after_frame_wrap(fifo.beam.scanline);
        let source_count = if vint.remaining != 0 && vint.armed && fifo.beam.scanline >= 224 {
            // ASM V_Int $488: the interrupted source PC first accounts for
            // the 44-cycle exception entry, then ADDQ/ST/RTE execute before
            // the same decoder instruction is retried.
            source_cycles = [44, 24, 16, 20, raw_cycles];
            vint.remaining -= 1;
            vint.armed = false;
            vint_serviced = true;
            5
        } else {
            source_cycles[0] = raw_cycles;
            1
        };
        for &cycles in &source_cycles[..source_count] {
            let generated_wait = advance_decompress_source_instruction_with_interval_kind(
                cycles,
                &mut fifo,
                &mut phase,
                &mut refresh_events,
                &mut advance_hardware,
            );
            if let Some(generated_wait) = generated_wait {
                external_wait_cycles = external_wait_cycles
                    .checked_add(generated_wait)
                    .expect("decoder wait count fits u32");
            } else {
                scheduling_failed = true;
                return;
            }
        }
    };

    for &raw_cycles in entry_instruction_cycles {
        advance_instruction(raw_cycles);
    }
    let visit = crate::saxman_timing::visit_decompress_chunk_instructions_sub_13684(
        compressed,
        resume,
        &mut advance_instruction,
    )
    .ok()?;
    for &raw_cycles in exit_instruction_cycles {
        advance_instruction(raw_cycles);
    }
    if scheduling_failed {
        return None;
    }

    Some(DecompressToVramDecoderAdvance {
        cursor: LoadInGameCpuRasterCursor {
            beam: fifo.beam,
            refresh_phase: phase,
        },
        timing: visit.timing,
        resume: visit.resume,
        refresh_events,
        vint_serviced,
        external_wait_cycles: M68kCycleCount::from_raw(external_wait_cycles),
    })
}

/// Co-schedule one ASM `EniDec` call with its source-level caller setup.
///
/// `caller_instruction_cycles` ends at the `j_EniDec` jump stub. The Enigma
/// command visitor then supplies every instruction through `loc_13582` and
/// the final RTS, while this shell owns refresh, recurring VINT, and external
/// GEMS waits.
pub fn enigma_decoder_cursor_with_scheduler(
    entry: LoadInGameCpuRasterCursor,
    caller_instruction_cycles: &[u32],
    compressed: &[u8],
    base_tile_word: u16,
    source_address: u32,
    vint_count: u8,
    mut advance_hardware: impl FnMut(M68kCycleCount, bool, bool) -> Option<M68kCycleCount>,
) -> Option<EnigmaDecoderAdvance> {
    let mut fifo = PlatformArtFifoScheduler::new(entry.beam);
    let mut phase = entry.refresh_phase;
    let mut refresh_events = 0_u32;
    let mut external_wait_cycles = 0_u32;
    let mut scheduling_failed = false;
    let mut vint_serviced = false;
    let mut vint = DecompressToVramVintTracker::new(vint_count);

    let mut advance_instruction = |raw_cycles: u32| {
        if scheduling_failed {
            return;
        }
        let mut source_cycles = [0_u32; 5];
        vint.rearm_after_frame_wrap(fifo.beam.scanline);
        let source_count = if vint.remaining != 0 && vint.armed && fifo.beam.scanline >= 224 {
            // ASM V_Int $488: exception entry, ADDQ/ST/RTE, then retry the
            // interrupted EniDec source instruction.
            source_cycles = [44, 24, 16, 20, raw_cycles];
            vint.remaining -= 1;
            vint.armed = false;
            vint_serviced = true;
            5
        } else {
            source_cycles[0] = raw_cycles;
            1
        };
        for &cycles in &source_cycles[..source_count] {
            let Some(generated_wait) = advance_decompress_source_instruction_with_interval_kind(
                cycles,
                &mut fifo,
                &mut phase,
                &mut refresh_events,
                &mut advance_hardware,
            ) else {
                scheduling_failed = true;
                return;
            };
            external_wait_cycles = external_wait_cycles
                .checked_add(generated_wait)
                .expect("Enigma wait count fits u32");
        }
    };

    for &raw_cycles in caller_instruction_cycles {
        advance_instruction(raw_cycles);
    }
    let timing = crate::enigma_timing::visit_enigma_instructions(
        compressed,
        base_tile_word,
        source_address,
        &mut advance_instruction,
    )
    .ok()?;
    if scheduling_failed {
        return None;
    }

    Some(EnigmaDecoderAdvance {
        cursor: LoadInGameCpuRasterCursor {
            beam: fifo.beam,
            refresh_phase: phase,
        },
        timing,
        refresh_events,
        vint_serviced,
        external_wait_cycles: M68kCycleCount::from_raw(external_wait_cycles),
    })
}

/// Co-schedule `Load_IntroSequence1`'s post-`sub_1B850` controller allocation.
pub fn title_intro_controller_allocation_cursor_with_scheduler(
    entry: LoadInGameCpuRasterCursor,
    mut advance_hardware: impl FnMut(M68kCycleCount, bool, bool) -> Option<M68kCycleCount>,
) -> Option<TitleIntroSourceAdvance> {
    let mut fifo = PlatformArtFifoScheduler::new(entry.beam);
    let mut phase = entry.refresh_phase;
    let mut refresh_events = 0_u32;
    let mut external_wait_cycles = 0_u32;
    let mut scheduling_failed = false;

    let timing =
        crate::intro_timing::visit_load_intro_sequence1_controller_allocation(|raw_cycles| {
            if scheduling_failed {
                return;
            }
            let Some(generated_wait) = advance_decompress_source_instruction_with_interval_kind(
                raw_cycles,
                &mut fifo,
                &mut phase,
                &mut refresh_events,
                &mut advance_hardware,
            ) else {
                scheduling_failed = true;
                return;
            };
            external_wait_cycles = external_wait_cycles
                .checked_add(generated_wait)
                .expect("title controller allocation wait count fits u32");
        });
    if scheduling_failed {
        return None;
    }

    Some(TitleIntroSourceAdvance {
        cursor: LoadInGameCpuRasterCursor {
            beam: fifo.beam,
            refresh_phase: phase,
        },
        timing,
        refresh_events,
        external_wait_cycles: M68kCycleCount::from_raw(external_wait_cycles),
    })
}

/// Co-schedule the intro loader return through the first `WaitForVint` poll.
pub fn title_intro_wait_for_vint_poll_cursor_with_scheduler(
    entry: LoadInGameCpuRasterCursor,
    mut advance_hardware: impl FnMut(M68kCycleCount, bool, bool) -> Option<M68kCycleCount>,
) -> Option<TitleIntroSourceAdvance> {
    let mut fifo = PlatformArtFifoScheduler::new(entry.beam);
    let mut phase = entry.refresh_phase;
    let mut refresh_events = 0_u32;
    let mut external_wait_cycles = 0_u32;
    let mut scheduling_failed = false;

    let timing = crate::intro_timing::visit_intro_controller_to_wait_for_vint_poll(|raw_cycles| {
        if scheduling_failed {
            return;
        }
        let Some(generated_wait) = advance_decompress_source_instruction_with_interval_kind(
            raw_cycles,
            &mut fifo,
            &mut phase,
            &mut refresh_events,
            &mut advance_hardware,
        ) else {
            scheduling_failed = true;
            return;
        };
        external_wait_cycles = external_wait_cycles
            .checked_add(generated_wait)
            .expect("title frame-handoff wait count fits u32");
    });
    if scheduling_failed {
        return None;
    }

    Some(TitleIntroSourceAdvance {
        cursor: LoadInGameCpuRasterCursor {
            beam: fifo.beam,
            refresh_phase: phase,
        },
        timing,
        refresh_events,
        external_wait_cycles: M68kCycleCount::from_raw(external_wait_cycles),
    })
}

/// Co-schedule a completed title frame through the next `WaitForVint` poll.
pub fn title_frame_return_to_wait_for_vint_poll_cursor_with_scheduler(
    entry: LoadInGameCpuRasterCursor,
    mut advance_hardware: impl FnMut(M68kCycleCount, bool, bool) -> Option<M68kCycleCount>,
) -> Option<TitleIntroSourceAdvance> {
    title_source_routine_cursor_with_scheduler(
        entry,
        |visit| Some(crate::intro_timing::visit_title_frame_return_to_wait_for_vint_poll(visit)),
        &mut advance_hardware,
    )
}

/// Co-schedule the blocking title `WaitForVint` poll through
/// `Mode_Standard` entry.
///
/// ASM `WaitForVint` $93E/$942 remains on the persistent GEMS/VDP timeline
/// until VINT asserts at H40 scanline 224, h_mclk 16. The interrupted PC,
/// exception entry, `V_Int` $488, resumed wait tail, and `MainGameLoop`
/// dispatch are all retained as separate scheduler ticks.
fn title_wait_for_vint_cursor_with_scheduler(
    entry: LoadInGameCpuRasterCursor,
    return_to_mode_standard: bool,
    mut advance_hardware: impl FnMut(M68kCycleCount, bool, bool) -> Option<M68kCycleCount>,
) -> Option<TitleWaitForVintAdvance> {
    #[derive(Clone, Copy)]
    enum PollPc {
        TestLoc93e,
        BranchLoc942,
    }

    const MAX_SCHEDULER_TICKS_PER_FRAME: usize = 200_000;

    let mut fifo = PlatformArtFifoScheduler::new(entry.beam);
    let mut phase = entry.refresh_phase;
    let mut refresh_events = 0_u32;
    let mut external_wait_cycles = 0_u32;
    let mut scheduler_tick_count = 0_u32;
    let mut raw_cycles = 0_u32;
    let mut pc = PollPc::TestLoc93e;

    let mut advance_tick = |cycles: u32,
                            fifo: &mut PlatformArtFifoScheduler,
                            phase: &mut M68kRefreshPhase,
                            refresh_events: &mut u32|
     -> Option<()> {
        let generated_wait = advance_decompress_source_instruction_with_interval_kind(
            cycles,
            fifo,
            phase,
            refresh_events,
            &mut advance_hardware,
        )?;
        external_wait_cycles = external_wait_cycles.checked_add(generated_wait)?;
        scheduler_tick_count = scheduler_tick_count.checked_add(1)?;
        raw_cycles = raw_cycles.checked_add(cycles)?;
        Some(())
    };

    let mut reached_vint = false;
    for _ in 0..MAX_SCHEDULER_TICKS_PER_FRAME {
        reached_vint = fifo.beam.scanline > 224
            || (fifo.beam.scanline == 224 && fifo.beam.scanline_mclk >= 16);
        if reached_vint {
            break;
        }

        // ASM WaitForVint $93E/$942: TST.B abs.w; taken BEQ.S.
        let cycles = match pc {
            PollPc::TestLoc93e => 12,
            PollPc::BranchLoc942 => 10,
        };
        advance_tick(cycles, &mut fifo, &mut phase, &mut refresh_events)?;
        pc = match pc {
            PollPc::TestLoc93e => PollPc::BranchLoc942,
            PollPc::BranchLoc942 => PollPc::TestLoc93e,
        };
    }
    if !reached_vint {
        return None;
    }

    let resume = match pc {
        PollPc::TestLoc93e => crate::WaitForVintResumePath::PollTestLoc93e,
        PollPc::BranchLoc942 => crate::WaitForVintResumePath::StaleBranchLoc942,
    };
    let refresh_events_at_assertion = refresh_events;

    // jgenesis samples the pending level-6 interrupt for 10 cycles, performs
    // the 44-cycle autovector entry, then executes ASM V_Int $488.
    for cycles in [10, 44, 24, 16, 20] {
        advance_tick(cycles, &mut fifo, &mut phase, &mut refresh_events)?;
    }
    if resume == crate::WaitForVintResumePath::StaleBranchLoc942 {
        // ASM $942: the saved condition codes take one stale branch.
        advance_tick(10, &mut fifo, &mut phase, &mut refresh_events)?;
    }
    // ASM WaitForVint $93E..$954 returns to its caller.
    for cycles in [12, 8, 12, 12, 16] {
        advance_tick(cycles, &mut fifo, &mut phase, &mut refresh_events)?;
    }
    if return_to_mode_standard {
        // ASM MainGameLoop $73C/$740 and GameModesArray branch to
        // Mode_Standard $77A.
        for cycles in [12, 20, 10] {
            advance_tick(cycles, &mut fifo, &mut phase, &mut refresh_events)?;
        }
    }

    Some(TitleWaitForVintAdvance {
        cursor: LoadInGameCpuRasterCursor {
            beam: fifo.beam,
            refresh_phase: phase,
        },
        resume,
        scheduler_tick_count,
        raw_cycles: M68kCycleCount::from_raw(raw_cycles),
        refresh_events,
        post_assertion_refresh_events: refresh_events.checked_sub(refresh_events_at_assertion)?,
        external_wait_cycles: M68kCycleCount::from_raw(external_wait_cycles),
    })
}

/// Co-schedule the blocking title wait through `Mode_Standard` entry.
pub fn title_wait_for_vint_to_mode_standard_cursor_with_scheduler(
    entry: LoadInGameCpuRasterCursor,
    advance_hardware: impl FnMut(M68kCycleCount, bool, bool) -> Option<M68kCycleCount>,
) -> Option<TitleWaitForVintAdvance> {
    title_wait_for_vint_cursor_with_scheduler(entry, true, advance_hardware)
}

/// Co-schedule a blocking `WaitForVint` through its subroutine return.
pub fn title_wait_for_vint_to_subroutine_return_cursor_with_scheduler(
    entry: LoadInGameCpuRasterCursor,
    advance_hardware: impl FnMut(M68kCycleCount, bool, bool) -> Option<M68kCycleCount>,
) -> Option<TitleWaitForVintAdvance> {
    title_wait_for_vint_cursor_with_scheduler(entry, false, advance_hardware)
}

/// Co-schedule a `Do_Nothing` call on the title hardware timeline.
pub fn title_do_nothing_call_cursor_with_scheduler(
    entry: LoadInGameCpuRasterCursor,
    mut advance_hardware: impl FnMut(M68kCycleCount, bool, bool) -> Option<M68kCycleCount>,
) -> Option<TitleIntroSourceAdvance> {
    title_source_routine_cursor_with_scheduler(
        entry,
        |visit| Some(crate::intro_timing::visit_do_nothing_call(visit)),
        &mut advance_hardware,
    )
}

/// Co-schedule `Mode_Standard -> Do_Nothing` on the title hardware timeline.
pub fn title_mode_standard_do_nothing_cursor_with_scheduler(
    entry: LoadInGameCpuRasterCursor,
    advance_hardware: impl FnMut(M68kCycleCount, bool, bool) -> Option<M68kCycleCount>,
) -> Option<TitleIntroSourceAdvance> {
    title_do_nothing_call_cursor_with_scheduler(entry, advance_hardware)
}

/// Co-schedule the title's empty `sub_1596` foreground-update queues.
pub fn title_mode_standard_empty_foreground_queues_cursor_with_scheduler(
    entry: LoadInGameCpuRasterCursor,
    mut advance_hardware: impl FnMut(M68kCycleCount, bool, bool) -> Option<M68kCycleCount>,
) -> Option<TitleIntroSourceAdvance> {
    title_source_routine_cursor_with_scheduler(
        entry,
        |visit| {
            Some(crate::intro_timing::visit_mode_standard_empty_foreground_queues_sub_1596(visit))
        },
        &mut advance_hardware,
    )
}

/// Co-schedule the first title `Execute_Objects` dispatch into `sub_1B8F4`.
pub fn title_execute_objects_to_intro_controller_cursor_with_scheduler(
    entry: LoadInGameCpuRasterCursor,
    rng_offset_before: u16,
    mut advance_hardware: impl FnMut(M68kCycleCount, bool, bool) -> Option<M68kCycleCount>,
) -> Option<TitleIntroSourceAdvance> {
    title_source_routine_cursor_with_scheduler(
        entry,
        |visit| {
            crate::intro_timing::visit_title_execute_objects_to_intro_controller_sub_1b8f4(
                rng_offset_before,
                visit,
            )
        },
        &mut advance_hardware,
    )
}

/// Co-schedule the second title text-worker and controller dispatches.
pub fn title_second_execute_objects_cursor_with_scheduler(
    entry: LoadInGameCpuRasterCursor,
    rng_offset_before: u16,
    mut advance_hardware: impl FnMut(M68kCycleCount, bool, bool) -> Option<M68kCycleCount>,
) -> Option<TitleIntroSourceAdvance> {
    title_source_routine_cursor_with_scheduler(
        entry,
        |visit| crate::intro_timing::visit_title_second_execute_objects(rng_offset_before, visit),
        &mut advance_hardware,
    )
}

/// Co-schedule the third title text-delay and controller dispatches.
pub fn title_third_execute_objects_cursor_with_scheduler(
    entry: LoadInGameCpuRasterCursor,
    rng_offset_before: u16,
    mut advance_hardware: impl FnMut(M68kCycleCount, bool, bool) -> Option<M68kCycleCount>,
) -> Option<TitleIntroSourceAdvance> {
    title_source_routine_cursor_with_scheduler(
        entry,
        |visit| crate::intro_timing::visit_title_third_execute_objects(rng_offset_before, visit),
        &mut advance_hardware,
    )
}

/// Co-schedule an ordinary title text-delay/controller-fade frame.
pub fn title_text_delay_fade_execute_objects_cursor_with_scheduler(
    entry: LoadInGameCpuRasterCursor,
    rng_offset_before: u16,
    text_delay_before: u8,
    controller_fade_before: u16,
    mut advance_hardware: impl FnMut(M68kCycleCount, bool, bool) -> Option<M68kCycleCount>,
) -> Option<TitleIntroSourceAdvance> {
    title_source_routine_cursor_with_scheduler(
        entry,
        |visit| {
            crate::intro_timing::visit_title_text_delay_fade_execute_objects(
                rng_offset_before,
                text_delay_before,
                controller_fade_before,
                visit,
            )
        },
        &mut advance_hardware,
    )
}

/// Co-schedule frame 98's title text delay and controller fade exit.
pub fn title_controller_fade_exit_execute_objects_cursor_with_scheduler(
    entry: LoadInGameCpuRasterCursor,
    rng_offset_before: u16,
    text_delay_before: u8,
    mut advance_hardware: impl FnMut(M68kCycleCount, bool, bool) -> Option<M68kCycleCount>,
) -> Option<TitleIntroSourceAdvance> {
    title_source_routine_cursor_with_scheduler(
        entry,
        |visit| {
            crate::intro_timing::visit_title_controller_fade_exit_execute_objects(
                rng_offset_before,
                text_delay_before,
                visit,
            )
        },
        &mut advance_hardware,
    )
}

/// Co-schedule an ordinary post-fade title text/controller wait frame.
pub fn title_post_fade_wait_execute_objects_cursor_with_scheduler(
    entry: LoadInGameCpuRasterCursor,
    rng_offset_before: u16,
    text_delay_before: u8,
    controller_wait_before: u16,
    mut advance_hardware: impl FnMut(M68kCycleCount, bool, bool) -> Option<M68kCycleCount>,
) -> Option<TitleIntroSourceAdvance> {
    title_source_routine_cursor_with_scheduler(
        entry,
        |visit| {
            crate::intro_timing::visit_title_post_fade_wait_execute_objects(
                rng_offset_before,
                text_delay_before,
                controller_wait_before,
                visit,
            )
        },
        &mut advance_hardware,
    )
}

/// Co-schedule frame 115's first title glyph and controller wait.
pub fn title_first_glyph_execute_objects_cursor_with_scheduler(
    entry: LoadInGameCpuRasterCursor,
    rng_offset_before: u16,
    controller_wait_before: u16,
    mut advance_hardware: impl FnMut(M68kCycleCount, bool, bool) -> Option<M68kCycleCount>,
) -> Option<TitleIntroSourceAdvance> {
    title_source_routine_cursor_with_scheduler(
        entry,
        |visit| {
            crate::intro_timing::visit_title_first_glyph_execute_objects(
                rng_offset_before,
                controller_wait_before,
                visit,
            )
        },
        &mut advance_hardware,
    )
}

/// Co-schedule an ordinary title glyph and controller wait.
pub fn title_ordinary_glyph_execute_objects_cursor_with_scheduler(
    entry: LoadInGameCpuRasterCursor,
    rng_offset_before: u16,
    controller_wait_before: u16,
    glyph: u8,
    mut advance_hardware: impl FnMut(M68kCycleCount, bool, bool) -> Option<M68kCycleCount>,
) -> Option<TitleIntroSourceAdvance> {
    title_source_routine_cursor_with_scheduler(
        entry,
        |visit| {
            crate::intro_timing::visit_title_ordinary_glyph_execute_objects(
                rng_offset_before,
                controller_wait_before,
                glyph,
                visit,
            )
        },
        &mut advance_hardware,
    )
}

/// Co-schedule IntroSequence1's glyph/Start path through `StopMusic $E14C8`.
pub fn title_intro_sequence1_glyph_start_to_stop_music_cursor_with_scheduler(
    entry: LoadInGameCpuRasterCursor,
    rng_offset_before: u16,
    controller_wait_before: u16,
    glyph: u8,
    intro_sequence_counter_before: u8,
    mut advance_hardware: impl FnMut(M68kCycleCount, bool, bool) -> Option<M68kCycleCount>,
) -> Option<TitleIntroSourceAdvance> {
    title_source_routine_cursor_with_scheduler(
        entry,
        |visit| {
            crate::intro_timing::visit_title_intro_sequence1_glyph_start_to_stop_music(
                rng_offset_before,
                controller_wait_before,
                glyph,
                intro_sequence_counter_before,
                visit,
            )
        },
        &mut advance_hardware,
    )
}

/// Co-schedule the title `StopMusic` return through the first fade wait poll.
pub fn title_stop_music_return_to_first_fade_wait_cursor_with_scheduler(
    entry: LoadInGameCpuRasterCursor,
    mut advance_hardware: impl FnMut(M68kCycleCount, bool, bool) -> Option<M68kCycleCount>,
) -> Option<TitleIntroSourceAdvance> {
    title_source_routine_cursor_with_scheduler(
        entry,
        |visit| Some(crate::intro_timing::visit_title_stop_music_return_to_first_fade_wait(visit)),
        &mut advance_hardware,
    )
}

/// Co-schedule initial `sub_1B8F4` work through `sub_1C4EA` entry.
pub fn title_intro_controller_prompt_gfx_to_palette_cursor_with_scheduler(
    entry: LoadInGameCpuRasterCursor,
    mut advance_hardware: impl FnMut(M68kCycleCount, bool, bool) -> Option<M68kCycleCount>,
) -> Option<TitleIntroSourceAdvance> {
    title_source_routine_cursor_with_scheduler(
        entry,
        |visit| {
            Some(crate::intro_timing::visit_title_intro_controller_prompt_gfx_to_sub_1c4ea(visit))
        },
        &mut advance_hardware,
    )
}

/// Co-schedule `sub_1C4EA` through the text-worker `sub_DAC` entry.
pub fn title_intro_palette_copy_to_text_object_cursor_with_scheduler(
    entry: LoadInGameCpuRasterCursor,
    mut advance_hardware: impl FnMut(M68kCycleCount, bool, bool) -> Option<M68kCycleCount>,
) -> Option<TitleIntroSourceAdvance> {
    title_source_routine_cursor_with_scheduler(
        entry,
        |visit| {
            Some(
                crate::intro_timing::visit_title_intro_palette_copy_to_text_object_allocation(
                    visit,
                ),
            )
        },
        &mut advance_hardware,
    )
}

/// Co-schedule the text-worker allocation through `sub_1C204` entry.
pub fn title_text_object_allocation_to_fade_cursor_with_scheduler(
    entry: LoadInGameCpuRasterCursor,
    mut advance_hardware: impl FnMut(M68kCycleCount, bool, bool) -> Option<M68kCycleCount>,
) -> Option<TitleIntroSourceAdvance> {
    title_source_routine_cursor_with_scheduler(
        entry,
        |visit| Some(crate::intro_timing::visit_title_text_object_allocation_to_fade(visit)),
        &mut advance_hardware,
    )
}

/// Co-schedule the first `sub_1C204` hibernation through `$794`.
pub fn title_first_fade_hibernate_to_mode_standard_cursor_with_scheduler(
    entry: LoadInGameCpuRasterCursor,
    mut advance_hardware: impl FnMut(M68kCycleCount, bool, bool) -> Option<M68kCycleCount>,
) -> Option<TitleIntroSourceAdvance> {
    title_source_routine_cursor_with_scheduler(
        entry,
        |visit| Some(crate::intro_timing::visit_title_first_fade_hibernate_to_mode_standard(visit)),
        &mut advance_hardware,
    )
}

/// Co-schedule the first title `GfxObjects_MoveAndAnimate` call through `$798`.
pub fn title_first_gfx_object_move_and_animate_cursor_with_scheduler(
    entry: LoadInGameCpuRasterCursor,
    mut advance_hardware: impl FnMut(M68kCycleCount, bool, bool) -> Option<M68kCycleCount>,
) -> Option<TitleIntroSourceAdvance> {
    title_source_routine_cursor_with_scheduler(
        entry,
        |visit| Some(crate::intro_timing::visit_title_first_gfx_object_move_and_animate(visit)),
        &mut advance_hardware,
    )
}

/// Co-schedule the first title `Make_SpritesFromGfxObjects` call through `$7A0`.
pub fn title_first_make_sprites_from_gfx_objects_cursor_with_scheduler(
    entry: LoadInGameCpuRasterCursor,
    mut advance_hardware: impl FnMut(M68kCycleCount, bool, bool) -> Option<M68kCycleCount>,
) -> Option<TitleIntroSourceAdvance> {
    title_source_routine_cursor_with_scheduler(
        entry,
        |visit| Some(crate::intro_timing::visit_title_first_make_sprites_from_gfx_objects(visit)),
        &mut advance_hardware,
    )
}

/// Co-schedule a full-dirty-mask `sub_14C0` interpolation call.
pub fn title_full_palette_interpolation_call_cursor_with_scheduler(
    entry: LoadInGameCpuRasterCursor,
    mut advance_hardware: impl FnMut(M68kCycleCount, bool, bool) -> Option<M68kCycleCount>,
) -> Option<TitleIntroSourceAdvance> {
    title_source_routine_cursor_with_scheduler(
        entry,
        |visit| {
            Some(crate::intro_timing::visit_full_palette_interpolation_call(
                visit,
            ))
        },
        &mut advance_hardware,
    )
}

/// Co-schedule the first title `sub_14C0` interpolation through frame RTS.
pub fn title_first_palette_interpolation_cursor_with_scheduler(
    entry: LoadInGameCpuRasterCursor,
    advance_hardware: impl FnMut(M68kCycleCount, bool, bool) -> Option<M68kCycleCount>,
) -> Option<TitleIntroSourceAdvance> {
    title_full_palette_interpolation_call_cursor_with_scheduler(entry, advance_hardware)
}

/// Co-schedule the first fade decrement through the next `WaitForVint` poll.
pub fn title_palette_fade_first_decrement_to_next_wait_poll_cursor_with_scheduler(
    entry: LoadInGameCpuRasterCursor,
    mut advance_hardware: impl FnMut(M68kCycleCount, bool, bool) -> Option<M68kCycleCount>,
) -> Option<TitleIntroSourceAdvance> {
    title_source_routine_cursor_with_scheduler(
        entry,
        |visit| {
            Some(crate::intro_timing::visit_palette_fade_first_decrement_to_next_wait_poll(visit))
        },
        &mut advance_hardware,
    )
}

/// Co-schedule the negative fade decrement through the final VINT poll.
pub fn title_palette_fade_negative_decrement_to_final_wait_poll_cursor_with_scheduler(
    entry: LoadInGameCpuRasterCursor,
    mut advance_hardware: impl FnMut(M68kCycleCount, bool, bool) -> Option<M68kCycleCount>,
) -> Option<TitleIntroSourceAdvance> {
    title_source_routine_cursor_with_scheduler(
        entry,
        |visit| {
            Some(
                crate::intro_timing::visit_palette_fade_negative_decrement_to_final_wait_poll(
                    visit,
                ),
            )
        },
        &mut advance_hardware,
    )
}

/// Co-schedule a clean-dirty-mask `sub_14C0` call.
pub fn title_clean_palette_interpolation_call_cursor_with_scheduler(
    entry: LoadInGameCpuRasterCursor,
    mut advance_hardware: impl FnMut(M68kCycleCount, bool, bool) -> Option<M68kCycleCount>,
) -> Option<TitleIntroSourceAdvance> {
    title_source_routine_cursor_with_scheduler(
        entry,
        |visit| {
            Some(crate::intro_timing::visit_clean_palette_interpolation_call(
                visit,
            ))
        },
        &mut advance_hardware,
    )
}

/// Co-schedule the `Pal_FadeOut` return and caller clear through `loc_700`.
pub fn title_palette_fade_return_to_loc_700_cursor_with_scheduler(
    entry: LoadInGameCpuRasterCursor,
    mut advance_hardware: impl FnMut(M68kCycleCount, bool, bool) -> Option<M68kCycleCount>,
) -> Option<TitleIntroSourceAdvance> {
    title_source_routine_cursor_with_scheduler(
        entry,
        |visit| {
            Some(crate::intro_timing::visit_palette_fade_return_to_loc_700(
                visit,
            ))
        },
        &mut advance_hardware,
    )
}

/// Co-schedule the title restart from `loc_726` to the Ice `sub_E151C` entry.
pub fn title_restart_initialisers_to_play_sound_cursor_with_scheduler(
    entry: LoadInGameCpuRasterCursor,
    mut advance_hardware: impl FnMut(M68kCycleCount, bool, bool) -> Option<M68kCycleCount>,
) -> Option<TitleIntroSourceAdvance> {
    title_source_routine_cursor_with_scheduler(
        entry,
        |visit| Some(crate::intro_timing::visit_title_restart_initialisers_to_play_sound(visit)),
        &mut advance_hardware,
    )
}

/// Co-schedule power-on `loc_726` through `Load_SegaScreen -> StopMusic`.
pub fn power_on_restart_initialisers_to_stop_music_cursor_with_scheduler(
    entry: LoadInGameCpuRasterCursor,
    mut advance_hardware: impl FnMut(M68kCycleCount, bool, bool) -> Option<M68kCycleCount>,
) -> Option<TitleIntroSourceAdvance> {
    title_source_routine_cursor_with_scheduler(
        entry,
        |visit| Some(crate::intro_timing::visit_power_on_restart_initialisers_to_stop_music(visit)),
        &mut advance_hardware,
    )
}

fn title_source_routine_cursor_with_scheduler(
    entry: LoadInGameCpuRasterCursor,
    visit_timing: impl FnOnce(&mut dyn FnMut(u32)) -> Option<crate::intro_timing::IntroSourceTiming>,
    advance_hardware: &mut dyn FnMut(M68kCycleCount, bool, bool) -> Option<M68kCycleCount>,
) -> Option<TitleIntroSourceAdvance> {
    let mut fifo = PlatformArtFifoScheduler::new(entry.beam);
    let mut phase = entry.refresh_phase;
    let mut refresh_events = 0_u32;
    let mut external_wait_cycles = 0_u32;
    let mut scheduling_failed = false;

    let timing = visit_timing(&mut |raw_cycles| {
        if scheduling_failed {
            return;
        }
        let Some(generated_wait) = advance_decompress_source_instruction_with_interval_kind(
            raw_cycles,
            &mut fifo,
            &mut phase,
            &mut refresh_events,
            advance_hardware,
        ) else {
            scheduling_failed = true;
            return;
        };
        external_wait_cycles = external_wait_cycles
            .checked_add(generated_wait)
            .expect("title source-routine wait count fits u32");
    })?;
    if scheduling_failed {
        return None;
    }

    Some(TitleIntroSourceAdvance {
        cursor: LoadInGameCpuRasterCursor {
            beam: fifo.beam,
            refresh_phase: phase,
        },
        timing,
        refresh_events,
        external_wait_cycles: M68kCycleCount::from_raw(external_wait_cycles),
    })
}

/// Co-schedule a title-path `Palette_to_VRAM` call.
pub fn title_palette_to_vram_call_cursor_with_scheduler(
    entry: LoadInGameCpuRasterCursor,
    mut advance_hardware: impl FnMut(TitlePaletteHardwareEvent) -> Option<M68kCycleCount>,
) -> Option<TitlePaletteToVramAdvance> {
    title_memory_dma_routine_cursor_with_scheduler(
        entry,
        0,
        true,
        |visit| crate::intro_timing::visit_palette_to_vram_call(visit),
        &mut advance_hardware,
    )
}

/// Co-schedule the `loc_700/loc_70E` CPU-driven full-VRAM clear.
pub fn title_loc_700_vram_clear_cursor_with_scheduler(
    entry: LoadInGameCpuRasterCursor,
    mut advance_hardware: impl FnMut(TitleMemoryDmaHardwareEvent) -> Option<M68kCycleCount>,
) -> Option<TitleMemoryDmaAdvance> {
    title_memory_dma_routine_cursor_with_scheduler(
        entry,
        9,
        false,
        |visit| crate::intro_timing::visit_loc_700_vram_clear(visit),
        &mut advance_hardware,
    )
}

/// Co-schedule the `loc_714/loc_720` Sprite_Table clear.
pub fn title_loc_714_sprite_table_clear_cursor_with_scheduler(
    entry: LoadInGameCpuRasterCursor,
    mut advance_hardware: impl FnMut(TitleMemoryDmaHardwareEvent) -> Option<M68kCycleCount>,
) -> Option<TitleMemoryDmaAdvance> {
    title_memory_dma_routine_cursor_with_scheduler(
        entry,
        4,
        false,
        |visit| crate::intro_timing::visit_loc_714_sprite_table_clear(visit),
        &mut advance_hardware,
    )
}

/// Co-schedule power-on `loc_714/loc_720`, which crosses five VINTs.
///
/// The later title-restart path above crosses four. Oracle selected-PC tracing
/// pins the power-on fifth handler after frame 29: its 114 source cycles plus
/// one refresh advance the `loc_726` boundary by 116 cycles.
pub fn power_on_loc_714_sprite_table_clear_cursor_with_scheduler(
    entry: LoadInGameCpuRasterCursor,
    mut advance_hardware: impl FnMut(TitleMemoryDmaHardwareEvent) -> Option<M68kCycleCount>,
) -> Option<TitleMemoryDmaAdvance> {
    title_memory_dma_routine_cursor_with_scheduler(
        entry,
        5,
        false,
        |visit| crate::intro_timing::visit_loc_714_sprite_table_clear(visit),
        &mut advance_hardware,
    )
}

/// Co-schedule the Ice return and `loc_1B258` fill through `sub_1B2A4` entry.
pub fn title_load_screen_vram_fill_to_sub_1b2a4_cursor_with_scheduler(
    entry: LoadInGameCpuRasterCursor,
    mut advance_hardware: impl FnMut(TitleMemoryDmaHardwareEvent) -> Option<M68kCycleCount>,
) -> Option<TitleMemoryDmaAdvance> {
    title_memory_dma_routine_cursor_with_scheduler(
        entry,
        1,
        true,
        |visit| crate::intro_timing::visit_title_load_screen_vram_fill_to_sub_1b2a4(visit),
        &mut advance_hardware,
    )
}

/// Co-schedule `sub_1B2A4`'s title-map tail through the first
/// `Load_TitleArt -> DecompressToRAM` permutation table.
pub fn title_map_to_load_art_palette_permutation_cursor_with_scheduler(
    entry: LoadInGameCpuRasterCursor,
    mut advance_hardware: impl FnMut(TitleMemoryDmaHardwareEvent) -> Option<M68kCycleCount>,
) -> Option<TitleMemoryDmaAdvance> {
    title_memory_dma_routine_cursor_with_scheduler(
        entry,
        1,
        true,
        |visit| crate::intro_timing::visit_title_map_to_load_art_palette_permutation(visit),
        &mut advance_hardware,
    )
}

/// Co-schedule `sub_1BF24`'s selected character-map tail through the
/// `sub_1B532` Enigma entry.
pub fn title_character_map_to_chameleon_enigma_cursor_with_scheduler(
    entry: LoadInGameCpuRasterCursor,
    mut advance_hardware: impl FnMut(TitleMemoryDmaHardwareEvent) -> Option<M68kCycleCount>,
) -> Option<TitleMemoryDmaAdvance> {
    title_memory_dma_routine_cursor_with_scheduler(
        entry,
        0,
        true,
        |visit| crate::intro_timing::visit_title_character_map_to_chameleon_enigma(visit),
        &mut advance_hardware,
    )
}

/// Co-schedule the first title `Palette_to_VRAM` call.
pub fn title_mode_standard_palette_to_vram_cursor_with_scheduler(
    entry: LoadInGameCpuRasterCursor,
    advance_hardware: impl FnMut(TitlePaletteHardwareEvent) -> Option<M68kCycleCount>,
) -> Option<TitlePaletteToVramAdvance> {
    title_palette_to_vram_call_cursor_with_scheduler(entry, advance_hardware)
}

/// Co-schedule the title `Transfer_SpriteAndKidToVRAM` call.
pub fn title_mode_standard_sprite_and_kid_to_vram_cursor_with_scheduler(
    entry: LoadInGameCpuRasterCursor,
    mut advance_hardware: impl FnMut(TitleMemoryDmaHardwareEvent) -> Option<M68kCycleCount>,
) -> Option<TitleMemoryDmaAdvance> {
    title_memory_dma_routine_cursor_with_scheduler(
        entry,
        0,
        true,
        |visit| crate::intro_timing::visit_mode_standard_sprite_and_kid_to_vram(visit),
        &mut advance_hardware,
    )
}

/// Co-schedule the title `Transfer_ScrollDataToVRAM` call.
pub fn title_mode_standard_scroll_data_to_vram_cursor_with_scheduler(
    entry: LoadInGameCpuRasterCursor,
    mut advance_hardware: impl FnMut(TitleMemoryDmaHardwareEvent) -> Option<M68kCycleCount>,
) -> Option<TitleMemoryDmaAdvance> {
    title_memory_dma_routine_cursor_with_scheduler(
        entry,
        0,
        true,
        |visit| crate::intro_timing::visit_mode_standard_scroll_data_to_vram(visit),
        &mut advance_hardware,
    )
}

/// Co-schedule the first title `ReadJoypad` call through `$79C`.
pub fn title_first_read_joypad_cursor_with_scheduler(
    entry: LoadInGameCpuRasterCursor,
    mut advance_hardware: impl FnMut(TitleMemoryDmaHardwareEvent) -> Option<M68kCycleCount>,
) -> Option<TitleMemoryDmaAdvance> {
    title_memory_dma_routine_cursor_with_scheduler(
        entry,
        0,
        true,
        |visit| crate::intro_timing::visit_title_first_read_joypad(visit),
        &mut advance_hardware,
    )
}

fn title_memory_dma_routine_cursor_with_scheduler(
    entry: LoadInGameCpuRasterCursor,
    vint_count: u8,
    vint_initially_armed: bool,
    visit_timing: impl FnOnce(
        &mut dyn FnMut(crate::intro_timing::MemoryDmaTimingEvent),
    ) -> crate::intro_timing::IntroSourceTiming,
    advance_hardware: &mut dyn FnMut(TitleMemoryDmaHardwareEvent) -> Option<M68kCycleCount>,
) -> Option<TitleMemoryDmaAdvance> {
    let mut fifo = PlatformArtFifoScheduler::new(entry.beam);
    let mut phase = entry.refresh_phase;
    let mut refresh_events = 0_u32;
    let mut external_wait_cycles = 0_u32;
    let mut fifo_halt_cycles = 0_u32;
    let mut dma_halt_cycles = 0_u32;
    let mut vint = DecompressToVramVintTracker::new(vint_count);
    vint.armed = vint_initially_armed;
    let mut scheduling_failed = false;

    let timing = visit_timing(&mut |event| {
        if scheduling_failed {
            return;
        }
        match event {
            crate::intro_timing::MemoryDmaTimingEvent::Instruction(raw_cycles) => {
                vint.rearm_after_frame_wrap(fifo.beam.scanline);
                if vint.remaining != 0 && vint.armed && fifo.beam.scanline >= 224 {
                    for vint_cycles in [10, 44, 28, 12, 20] {
                        let mut advance = |cycles, vdp_owns_bus, source_instruction| {
                            advance_hardware(TitleMemoryDmaHardwareEvent::Advance {
                                cycles,
                                vdp_owns_bus,
                                source_instruction,
                            })
                        };
                        let Some(generated_wait) =
                            advance_decompress_source_instruction_with_interval_kind(
                                vint_cycles,
                                &mut fifo,
                                &mut phase,
                                &mut refresh_events,
                                &mut advance,
                            )
                        else {
                            scheduling_failed = true;
                            return;
                        };
                        external_wait_cycles = external_wait_cycles
                            .checked_add(generated_wait)
                            .expect("title VINT external waits fit u32");
                    }
                    vint.remaining -= 1;
                    vint.armed = false;
                }
                let mut advance = |cycles, vdp_owns_bus, source_instruction| {
                    advance_hardware(TitleMemoryDmaHardwareEvent::Advance {
                        cycles,
                        vdp_owns_bus,
                        source_instruction,
                    })
                };
                let Some(generated_wait) = advance_decompress_source_instruction_with_interval_kind(
                    raw_cycles,
                    &mut fifo,
                    &mut phase,
                    &mut refresh_events,
                    &mut advance,
                ) else {
                    scheduling_failed = true;
                    return;
                };
                external_wait_cycles = external_wait_cycles
                    .checked_add(generated_wait)
                    .expect("Palette_to_VRAM external waits fit u32");
            }
            crate::intro_timing::MemoryDmaTimingEvent::SetZ80BusRequest(requested) => {
                if advance_hardware(TitleMemoryDmaHardwareEvent::SetZ80BusRequest(requested))
                    .is_none_or(|wait| wait.get() != 0)
                {
                    scheduling_failed = true;
                }
            }
            crate::intro_timing::MemoryDmaTimingEvent::WriteVdpWord { target } => {
                fifo.push_word_with_latency(
                    3,
                    matches!(target, crate::intro_timing::MemoryToVdpDmaTarget::Vram),
                );
                while fifo.pending_words != 0 {
                    let halt = M68kCycleCount::from_raw(1);
                    fifo.advance_m68k_cycles(halt);
                    if advance_hardware(TitleMemoryDmaHardwareEvent::Advance {
                        cycles: halt,
                        vdp_owns_bus: true,
                        source_instruction: false,
                    })
                    .is_none_or(|wait| wait.get() != 0)
                    {
                        scheduling_failed = true;
                        return;
                    }
                    fifo_halt_cycles = fifo_halt_cycles
                        .checked_add(1)
                        .expect("title FIFO halt fits u32");
                    // Pinned jgenesis resets the approximate refresh counter
                    // on every tick where VDP FIFO backpressure owns the CPU.
                    phase = M68kRefreshPhase::new(0).expect("zero is a valid refresh phase");
                }
            }
            crate::intro_timing::MemoryDmaTimingEvent::BeginMemoryDma { words, target } => {
                fifo.begin_memory_dma(words, target);
                while fifo.memory_dma_active() {
                    let tick = M68kCycleCount::from_raw(1);
                    fifo.advance_m68k_cycles(tick);
                    let Some(wait) = advance_hardware(TitleMemoryDmaHardwareEvent::Advance {
                        cycles: tick,
                        vdp_owns_bus: true,
                        source_instruction: false,
                    }) else {
                        scheduling_failed = true;
                        return;
                    };
                    if wait.get() != 0 {
                        scheduling_failed = true;
                        return;
                    }
                    dma_halt_cycles = dma_halt_cycles
                        .checked_add(1)
                        .expect("title memory-DMA halt fits u32");
                    // Pinned jgenesis resets the 68000 refresh counter while
                    // `Vdp::should_halt_cpu` owns the bus. The counter remains
                    // zero until normal instruction execution resumes.
                    phase = M68kRefreshPhase::new(0).expect("zero is a valid refresh phase");
                }
            }
        }
    });
    if scheduling_failed {
        return None;
    }

    Some(TitleMemoryDmaAdvance {
        cursor: LoadInGameCpuRasterCursor {
            beam: fifo.beam,
            refresh_phase: phase,
        },
        timing,
        refresh_events,
        external_wait_cycles: M68kCycleCount::from_raw(external_wait_cycles),
        fifo_halt_cycles: M68kCycleCount::from_raw(fifo_halt_cycles),
        dma_halt_cycles: M68kCycleCount::from_raw(dma_halt_cycles),
        vint_events: vint_count - vint.remaining,
    })
}

/// Co-schedule a caller and ASM `sub_142CC` up to `loc_14300`.
///
/// `caller_instruction_cycles` ends at the jump stub which enters
/// `DecompressToVRAM`; the fixed tail starts with `bsr.s sub_142CC`, executes
/// its setup body, then runs the `moveq/movea` pair before `loc_14300`.
pub fn decompress_to_vram_setup_cursor_sub_142cc_with_scheduler(
    entry: LoadInGameCpuRasterCursor,
    caller_instruction_cycles: &[u32],
    mut advance_hardware: impl FnMut(M68kCycleCount, bool, bool) -> Option<M68kCycleCount>,
) -> Option<DecompressToVramSetupAdvance> {
    let mut fifo = PlatformArtFifoScheduler::new(entry.beam);
    let mut phase = entry.refresh_phase;
    let mut refresh_events = 0_u32;
    let mut external_wait_cycles = 0_u32;

    for &raw_cycles in caller_instruction_cycles.iter().chain(
        [
            // ASM DecompressToVRAM $142FA: bsr.s sub_142CC.
            18, // ASM sub_142CC $142CC..$142F8.
            4, 10, 8, 8, 8, 4, 4, 16, 8, 22, 8, 12, 12, 4, 4, 16,
            // ASM DecompressToVRAM $142FC..$142FE.
            4, 4,
        ]
        .iter(),
    ) {
        external_wait_cycles = external_wait_cycles.checked_add(
            advance_decompress_source_instruction_with_interval_kind(
                raw_cycles,
                &mut fifo,
                &mut phase,
                &mut refresh_events,
                &mut advance_hardware,
            )?,
        )?;
    }

    Some(DecompressToVramSetupAdvance {
        cursor: LoadInGameCpuRasterCursor {
            beam: fifo.beam,
            refresh_phase: phase,
        },
        refresh_events,
        external_wait_cycles: M68kCycleCount::from_raw(external_wait_cycles),
    })
}

/// Advance the inclusive DBF transfer at ASM `loc_14314`.
///
/// The scheduler mirrors the four-entry jgenesis FIFO used as the oracle:
/// each VRAM word needs two access slots and three slots of initial latency.
/// A write made while the FIFO is full stalls subsequent 68000 execution one
/// cycle at a time; those VDP-owned ticks reset the refresh counter.
pub fn load_in_game_platform_art_transfer_cursor_loc_14314(
    entry: LoadInGameCpuRasterCursor,
    output_words: u16,
) -> Option<LoadInGamePlatformArtTransferAdvance> {
    load_in_game_platform_art_transfer_cursor_loc_14314_with_hardware(
        entry,
        output_words,
        |_, _| Some(()),
    )
}

/// Advance `loc_14314` while clocking a colocated hardware machine.
///
/// The callback observes each completed source/refresh interval and every
/// one-cycle VDP-owned FIFO halt in chronological order. Returning `None`
/// rejects a hardware state which cannot yet be represented at this boundary.
pub fn load_in_game_platform_art_transfer_cursor_loc_14314_with_hardware(
    entry: LoadInGameCpuRasterCursor,
    output_words: u16,
    advance_hardware: impl FnMut(M68kCycleCount, bool) -> Option<()>,
) -> Option<LoadInGamePlatformArtTransferAdvance> {
    decompress_to_vram_transfer_cursor_loc_14314_with_hardware(
        entry,
        output_words,
        0,
        DecompressToVramTransferExit::Return,
        advance_hardware,
    )
}

/// Advance a generic `DecompressToVRAM loc_14314` transfer.
///
/// Each requested VINT is serviced at the first instruction boundary after
/// scanline 224, then re-armed only after the next frame wrap while the same
/// FIFO and hardware timelines continue to advance.
pub fn decompress_to_vram_transfer_cursor_loc_14314_with_hardware(
    entry: LoadInGameCpuRasterCursor,
    output_words: u16,
    vint_count: u8,
    exit: DecompressToVramTransferExit,
    mut advance_hardware: impl FnMut(M68kCycleCount, bool) -> Option<()>,
) -> Option<DecompressToVramTransferAdvance> {
    decompress_to_vram_transfer_cursor_loc_14314_with_scheduler(
        entry,
        output_words,
        vint_count,
        exit,
        |cycles, vdp_owns_bus| {
            advance_hardware(cycles, vdp_owns_bus)?;
            Some(M68kCycleCount::from_raw(0))
        },
    )
}

/// Co-schedule a generic `DecompressToVRAM loc_14314` transfer.
///
/// The callback returns newly queued external 68000 wait cycles. A callback
/// must return zero while `vdp_owns_bus` is true because the competing
/// hardware access is halted instead of delaying the 68000.
pub fn decompress_to_vram_transfer_cursor_loc_14314_with_scheduler(
    entry: LoadInGameCpuRasterCursor,
    output_words: u16,
    vint_count: u8,
    exit: DecompressToVramTransferExit,
    advance_hardware: impl FnMut(M68kCycleCount, bool) -> Option<M68kCycleCount>,
) -> Option<DecompressToVramTransferAdvance> {
    decompress_to_vram_transfer_cursor_loc_14314_with_display_mode_scheduler(
        entry,
        output_words,
        vint_count,
        exit,
        true,
        advance_hardware,
    )
}

/// Co-schedule ASM `DecompressToRAM loc_1440A`'s remapped-word transfer.
///
/// Unlike `loc_14314`, each VDP word is assembled through two indexed palette
/// lookups before the port write. The FIFO and recurring VINT remain shared
/// hardware concerns, so every source instruction is visited at its real
/// boundary before the next lookup or write begins.
pub fn decompress_to_ram_transfer_cursor_loc_1440a_with_scheduler(
    entry: LoadInGameCpuRasterCursor,
    output_words: u16,
    vint_count: u8,
    exit: DecompressToVramTransferExit,
    mut advance_hardware: impl FnMut(M68kCycleCount, bool) -> Option<M68kCycleCount>,
) -> Option<DecompressToVramTransferAdvance> {
    if output_words == 0 {
        return None;
    }

    let mut fifo = PlatformArtFifoScheduler::new(entry.beam);
    let mut phase = entry.refresh_phase;
    let mut refresh_events = 0_u32;
    let mut fifo_halted_ticks = 0_u32;
    let mut vint_serviced = false;
    let mut vint = DecompressToVramVintTracker::new(vint_count);

    for word_index in 0..output_words {
        // ASM loc_1440A (kid.asm): two byte reads, two indexed permutation
        // lookups, the high-byte shift, one VDP data-port write, then DBF.
        for raw_cycles in [8, 14, 22, 8, 14] {
            vint_serviced |= service_decompress_to_vram_vint_if_pending(
                &mut vint,
                &mut fifo,
                &mut phase,
                &mut refresh_events,
                &mut advance_hardware,
            )?;
            advance_decompress_to_vram_source_instruction(
                raw_cycles,
                &mut fifo,
                &mut phase,
                &mut refresh_events,
                true,
                &mut advance_hardware,
            )?;
        }

        vint_serviced |= service_decompress_to_vram_vint_if_pending(
            &mut vint,
            &mut fifo,
            &mut phase,
            &mut refresh_events,
            &mut advance_hardware,
        )?;
        let generated_wait = advance_decompress_to_vram_source_instruction(
            8,
            &mut fifo,
            &mut phase,
            &mut refresh_events,
            false,
            &mut advance_hardware,
        )?;
        fifo.push_word();
        drain_decompress_to_vram_wait(
            generated_wait,
            &mut fifo,
            &mut phase,
            &mut advance_hardware,
        )?;
        while fifo.pending_words != 0 {
            let halt = M68kCycleCount::from_raw(1);
            fifo.advance_m68k_cycles(halt);
            if advance_hardware(halt, true)?.get() != 0 {
                return None;
            }
            fifo_halted_ticks = fifo_halted_ticks.checked_add(1)?;
            phase = M68kRefreshPhase::new(0).expect("zero is a valid refresh phase");
        }

        vint_serviced |= service_decompress_to_vram_vint_if_pending(
            &mut vint,
            &mut fifo,
            &mut phase,
            &mut refresh_events,
            &mut advance_hardware,
        )?;
        advance_decompress_to_vram_source_instruction(
            if word_index + 1 == output_words {
                14
            } else {
                10
            },
            &mut fifo,
            &mut phase,
            &mut refresh_events,
            true,
            &mut advance_hardware,
        )?;
    }

    let mut pending_external_wait_cycles = M68kCycleCount::from_raw(0);
    match exit {
        DecompressToVramTransferExit::Return => {
            // ASM loc_14422..return_14438: tst.w d0; beq.s; rts.
            for (instruction_index, raw_cycles) in [4, 10, 16].into_iter().enumerate() {
                pending_external_wait_cycles = advance_decompress_to_vram_source_instruction(
                    raw_cycles,
                    &mut fifo,
                    &mut phase,
                    &mut refresh_events,
                    instruction_index != 2,
                    &mut advance_hardware,
                )?;
            }
        }
        DecompressToVramTransferExit::CompactAndResume => {
            // ASM loc_14422..loc_14430 retains the upper $800 history bytes
            // before branching back to the next loc_143EA decode.
            for raw_cycles in [4, 8, 12, 8, 8] {
                advance_decompress_to_vram_source_instruction(
                    raw_cycles,
                    &mut fifo,
                    &mut phase,
                    &mut refresh_events,
                    true,
                    &mut advance_hardware,
                )?;
            }
            for copy_index in 0..512 {
                for raw_cycles in [20, if copy_index == 511 { 14 } else { 10 }] {
                    advance_decompress_to_vram_source_instruction(
                        raw_cycles,
                        &mut fifo,
                        &mut phase,
                        &mut refresh_events,
                        true,
                        &mut advance_hardware,
                    )?;
                }
            }
            advance_decompress_to_vram_source_instruction(
                10,
                &mut fifo,
                &mut phase,
                &mut refresh_events,
                true,
                &mut advance_hardware,
            )?;
        }
    }

    Some(DecompressToVramTransferAdvance {
        cursor: LoadInGameCpuRasterCursor {
            beam: fifo.beam,
            refresh_phase: phase,
        },
        fifo_halted_ticks: M68kCycleCount::from_raw(fifo_halted_ticks),
        refresh_events,
        vint_serviced,
        pending_external_wait_cycles,
    })
}

/// Co-schedule `DecompressToVRAM loc_14314` while VDP display is disabled.
///
/// ASM `Load_SegaScreen` writes register 1 as `$8134` before uploading its
/// logo. The blanked VDP drains each word before the next 22-cycle
/// `move.w`/`dbf` pair, so the 68000 never waits on a full FIFO.
pub fn decompress_to_vram_transfer_display_disabled_cursor_loc_14314_with_scheduler(
    entry: LoadInGameCpuRasterCursor,
    output_words: u16,
    vint_count: u8,
    exit: DecompressToVramTransferExit,
    advance_hardware: impl FnMut(M68kCycleCount, bool) -> Option<M68kCycleCount>,
) -> Option<DecompressToVramTransferAdvance> {
    decompress_to_vram_transfer_cursor_loc_14314_with_display_mode_scheduler(
        entry,
        output_words,
        vint_count,
        exit,
        false,
        advance_hardware,
    )
}

fn decompress_to_vram_transfer_cursor_loc_14314_with_display_mode_scheduler(
    entry: LoadInGameCpuRasterCursor,
    output_words: u16,
    vint_count: u8,
    exit: DecompressToVramTransferExit,
    display_enabled: bool,
    mut advance_hardware: impl FnMut(M68kCycleCount, bool) -> Option<M68kCycleCount>,
) -> Option<DecompressToVramTransferAdvance> {
    if output_words == 0 {
        return None;
    }

    let mut fifo = PlatformArtFifoScheduler::new(entry.beam);
    let mut phase = entry.refresh_phase;
    let mut refresh_events = 0_u32;
    let mut fifo_halted_ticks = 0_u32;
    let mut vint_serviced = false;
    let mut vint = DecompressToVramVintTracker::new(vint_count);

    for word_index in 0..output_words {
        vint_serviced |= service_decompress_to_vram_vint_if_pending(
            &mut vint,
            &mut fifo,
            &mut phase,
            &mut refresh_events,
            &mut advance_hardware,
        )?;
        // ASM loc_14314: move.w (a4)+,(a6)
        let generated_wait = advance_decompress_to_vram_source_instruction(
            12,
            &mut fifo,
            &mut phase,
            &mut refresh_events,
            false,
            &mut advance_hardware,
        )?;
        if display_enabled {
            fifo.push_word();
        }
        drain_decompress_to_vram_wait(
            generated_wait,
            &mut fifo,
            &mut phase,
            &mut advance_hardware,
        )?;

        while fifo.pending_words != 0 {
            let halt = M68kCycleCount::from_raw(1);
            fifo.advance_m68k_cycles(halt);
            if advance_hardware(halt, true)?.get() != 0 {
                return None;
            }
            fifo_halted_ticks = fifo_halted_ticks.checked_add(1)?;
            // jgenesis `record_68k_instruction`: VDP ownership resets the
            // approximate 68000 refresh counter on every halted tick.
            phase = M68kRefreshPhase::new(0).expect("zero is a valid refresh phase");
        }

        // ASM loc_14314: DBF d3,loc_14314.
        advance_decompress_to_vram_source_instruction(
            if word_index + 1 == output_words {
                14
            } else {
                10
            },
            &mut fifo,
            &mut phase,
            &mut refresh_events,
            true,
            &mut advance_hardware,
        )?;
    }

    let mut pending_external_wait_cycles = M68kCycleCount::from_raw(0);
    match exit {
        DecompressToVramTransferExit::Return => {
            // ASM loc_14318..return_14334: tst.w d0; beq.s; rts.
            for (instruction_index, raw_cycles) in [4, 10, 16].into_iter().enumerate() {
                vint_serviced |= service_decompress_to_vram_vint_if_pending(
                    &mut vint,
                    &mut fifo,
                    &mut phase,
                    &mut refresh_events,
                    &mut advance_hardware,
                )?;
                pending_external_wait_cycles = advance_decompress_to_vram_source_instruction(
                    raw_cycles,
                    &mut fifo,
                    &mut phase,
                    &mut refresh_events,
                    instruction_index != 2,
                    &mut advance_hardware,
                )?;
            }
        }
        DecompressToVramTransferExit::CompactAndResume => {
            // ASM loc_14318..loc_1432C: reject the final return, retain the
            // last $800 history bytes, then branch back to loc_14300.
            for raw_cycles in [4, 8, 12, 8, 8] {
                advance_decompress_to_vram_source_instruction(
                    raw_cycles,
                    &mut fifo,
                    &mut phase,
                    &mut refresh_events,
                    true,
                    &mut advance_hardware,
                )?;
            }
            for copy_index in 0..512 {
                for raw_cycles in [20, if copy_index == 511 { 14 } else { 10 }] {
                    advance_decompress_to_vram_source_instruction(
                        raw_cycles,
                        &mut fifo,
                        &mut phase,
                        &mut refresh_events,
                        true,
                        &mut advance_hardware,
                    )?;
                }
            }
            advance_decompress_to_vram_source_instruction(
                10,
                &mut fifo,
                &mut phase,
                &mut refresh_events,
                true,
                &mut advance_hardware,
            )?;
        }
    }

    Some(LoadInGamePlatformArtTransferAdvance {
        cursor: LoadInGameCpuRasterCursor {
            beam: fifo.beam,
            refresh_phase: phase,
        },
        fifo_halted_ticks: M68kCycleCount::from_raw(fifo_halted_ticks),
        refresh_events,
        vint_serviced,
        pending_external_wait_cycles,
    })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TitleIntroVdpSourceEvent {
    Instruction(u32),
    DataWord,
}

fn schedule_title_intro_vdp_events(
    entry: LoadInGameCpuRasterCursor,
    events: &[TitleIntroVdpSourceEvent],
    vint_count: u8,
    mut advance_hardware: impl FnMut(M68kCycleCount, bool) -> Option<M68kCycleCount>,
) -> Option<DecompressToVramTransferAdvance> {
    let mut fifo = PlatformArtFifoScheduler::new(entry.beam);
    let mut phase = entry.refresh_phase;
    let mut refresh_events = 0_u32;
    let mut fifo_halted_ticks = 0_u32;
    let mut vint_serviced = false;
    let mut vint = DecompressToVramVintTracker::new(vint_count);

    for event in events {
        vint_serviced |= service_decompress_to_vram_vint_if_pending(
            &mut vint,
            &mut fifo,
            &mut phase,
            &mut refresh_events,
            &mut advance_hardware,
        )?;
        let raw_cycles = match event {
            TitleIntroVdpSourceEvent::Instruction(raw_cycles) => *raw_cycles,
            TitleIntroVdpSourceEvent::DataWord => 12,
        };
        let pending_wait = advance_decompress_to_vram_source_instruction(
            raw_cycles,
            &mut fifo,
            &mut phase,
            &mut refresh_events,
            !matches!(event, TitleIntroVdpSourceEvent::DataWord),
            &mut advance_hardware,
        )?;
        if !matches!(event, TitleIntroVdpSourceEvent::DataWord) {
            continue;
        }

        fifo.push_word();
        drain_decompress_to_vram_wait(pending_wait, &mut fifo, &mut phase, &mut advance_hardware)?;
        while fifo.pending_words != 0 {
            let halt = M68kCycleCount::from_raw(1);
            fifo.advance_m68k_cycles(halt);
            if advance_hardware(halt, true)?.get() != 0 {
                return None;
            }
            fifo_halted_ticks = fifo_halted_ticks.checked_add(1)?;
            phase = M68kRefreshPhase::new(0).expect("zero is a valid refresh phase");
        }
    }

    Some(DecompressToVramTransferAdvance {
        cursor: LoadInGameCpuRasterCursor {
            beam: fifo.beam,
            refresh_phase: phase,
        },
        fifo_halted_ticks: M68kCycleCount::from_raw(fifo_halted_ticks),
        refresh_events,
        vint_serviced,
        pending_external_wait_cycles: M68kCycleCount::from_raw(0),
    })
}

/// Co-schedule sub_1B850's 40x28 plane-A blit through ASM `sub_1C5D0`.
pub fn title_intro_plane_a_blit_with_scheduler(
    entry: LoadInGameCpuRasterCursor,
    vint_count: u8,
    advance_hardware: impl FnMut(M68kCycleCount, bool) -> Option<M68kCycleCount>,
) -> Option<DecompressToVramTransferAdvance> {
    let mut events = Vec::with_capacity(2_400);
    // ASM sub_1B850 $1B8B0..$1B8C2: four MOVEQ instructions and BSR.
    events.extend(
        [4, 4, 4, 4, 18]
            .into_iter()
            .map(TitleIntroVdpSourceEvent::Instruction),
    );
    // ASM sub_1C5D0 (kid.asm:31341-31352): save registers and derive the
    // first plane-A control word before the nested row loops.
    events.extend(
        [40, 4, 20, 4, 4, 8, 4, 4, 4, 4]
            .into_iter()
            .map(TitleIntroVdpSourceEvent::Instruction),
    );
    for row in 0..28 {
        events.extend(
            [4, 16, 16]
                .into_iter()
                .map(TitleIntroVdpSourceEvent::Instruction),
        );
        for column in 0..40 {
            events.push(TitleIntroVdpSourceEvent::DataWord);
            events.push(TitleIntroVdpSourceEvent::Instruction(if column == 39 {
                14
            } else {
                10
            }));
        }
        events.push(TitleIntroVdpSourceEvent::Instruction(if row == 27 {
            14
        } else {
            10
        }));
    }
    // ASM sub_1C5D0 $1C5FE..$1C604: restore registers and return.
    events.extend(
        [44, 16]
            .into_iter()
            .map(TitleIntroVdpSourceEvent::Instruction),
    );

    schedule_title_intro_vdp_events(entry, &events, vint_count, advance_hardware)
}

/// Co-schedule sub_1B850's 28 full-width repeated plane-B rows.
pub fn title_intro_repeated_rows_with_scheduler(
    entry: LoadInGameCpuRasterCursor,
    vint_count: u8,
    advance_hardware: impl FnMut(M68kCycleCount, bool) -> Option<M68kCycleCount>,
) -> Option<DecompressToVramTransferAdvance> {
    let mut events = Vec::with_capacity(3_700);
    // ASM sub_1B850 $1B8DA..loc_1B8E4: set VRAM $E000 and seed d0=$1B.
    events.extend(
        [20, 4]
            .into_iter()
            .map(TitleIntroVdpSourceEvent::Instruction),
    );
    for row in 0..28 {
        events.push(TitleIntroVdpSourceEvent::Instruction(4));
        for column in 0..64 {
            events.push(TitleIntroVdpSourceEvent::DataWord);
            events.push(TitleIntroVdpSourceEvent::Instruction(if column == 63 {
                14
            } else {
                10
            }));
        }
        events.push(TitleIntroVdpSourceEvent::Instruction(8));
        events.push(TitleIntroVdpSourceEvent::Instruction(if row == 27 {
            14
        } else {
            10
        }));
    }
    events.push(TitleIntroVdpSourceEvent::Instruction(16));

    schedule_title_intro_vdp_events(entry, &events, vint_count, advance_hardware)
}

/// Advance ASM `Load_InGame $11A0E -> Initialize_Platforms -> $11A12`.
///
/// Source work comes from
/// [`crate::load_in_game_platform_initialisation_source_workload`]. Both
/// `loc_14314` transfers use raster-driven FIFO scheduling. VINT remains an
/// explicit hardware event because it may interrupt a different decoder
/// instruction when the caller supplies a different persistent raster seed.
pub fn load_in_game_platform_initialisation_cursor_from_entry(
    entry: LoadInGameCpuRasterCursor,
    vint: Option<LoadInGamePlatformVintEvent>,
    refresh: LoadInGamePlatformRefreshLedger,
) -> Option<LoadInGamePlatformInitialisationAdvance> {
    load_in_game_platform_initialisation_cursor_from_entry_with_hardware(
        entry,
        vint,
        refresh,
        |_, _| Some(()),
    )
}

/// Advance `Initialize_Platforms` while clocking colocated hardware.
///
/// Fixed source spans are emitted as one interval; the two `loc_14314`
/// transfers retain instruction and FIFO-halt granularity. A callback should
/// return `None` if its machine generates an external 68000 wait inside a
/// fixed span, until that decoder path has an instruction visitor.
pub fn load_in_game_platform_initialisation_cursor_from_entry_with_hardware(
    entry: LoadInGameCpuRasterCursor,
    vint: Option<LoadInGamePlatformVintEvent>,
    refresh: LoadInGamePlatformRefreshLedger,
    mut advance_hardware: impl FnMut(M68kCycleCount, bool) -> Option<()>,
) -> Option<LoadInGamePlatformInitialisationAdvance> {
    let workload = crate::load_in_game_platform_initialisation_source_workload();
    let mut cursor = entry;
    let mut refresh_events = 0_u32;

    let advance_source_span =
        |cursor: &mut LoadInGameCpuRasterCursor,
         raw: M68kCycleCount,
         span_refresh_events: u16,
         refresh_events: &mut u32,
         advance_hardware: &mut dyn FnMut(M68kCycleCount, bool) -> Option<()>| {
            let refresh_cycles = u32::from(span_refresh_events).checked_mul(2)?;
            let elapsed = raw.get().checked_add(refresh_cycles)?;
            let elapsed = M68kCycleCount::from_raw(elapsed);
            cursor.beam = cursor.beam.advance_m68k_cycles(elapsed);
            advance_hardware(elapsed, false)?;
            cursor.refresh_phase = M68kRefreshPhase::new(
                u8::try_from((u32::from(cursor.refresh_phase.get()) + elapsed.get()) % 128)
                    .expect("refresh phase remains below 128"),
            )
            .expect("refresh phase remains below 128");
            *refresh_events = refresh_events.checked_add(u32::from(span_refresh_events))?;
            Some(())
        };

    advance_source_span(
        &mut cursor,
        workload.prelude_through_first_decompress_entry,
        refresh.prelude,
        &mut refresh_events,
        &mut advance_hardware,
    )?;
    advance_source_span(
        &mut cursor,
        workload.horizontal_art.decoder,
        refresh.horizontal_decoder,
        &mut refresh_events,
        &mut advance_hardware,
    )?;
    let horizontal = load_in_game_platform_art_transfer_cursor_loc_14314_with_hardware(
        cursor,
        workload.horizontal_art.output_words,
        &mut advance_hardware,
    )?;
    cursor = horizontal.cursor;
    refresh_events = refresh_events.checked_add(horizontal.refresh_events)?;

    advance_source_span(
        &mut cursor,
        workload.between_art_uploads,
        refresh.between_uploads,
        &mut refresh_events,
        &mut advance_hardware,
    )?;

    if let Some(vint) = vint {
        let prefix = vint.vertical_decoder_raw_offset;
        let suffix = workload
            .vertical_art
            .decoder
            .get()
            .checked_sub(prefix.get())
            .map(M68kCycleCount::from_raw)?;
        advance_source_span(
            &mut cursor,
            prefix,
            refresh.vertical_decoder_before_vint,
            &mut refresh_events,
            &mut advance_hardware,
        )?;
        // Genesis VINT exception entry and the Kid handler path crossed by
        // Decompress_Chunk. ASM VInt $390..$3A2 returns after 60 source cycles.
        advance_source_span(
            &mut cursor,
            M68kCycleCount::from_raw(44),
            refresh.vint_exception,
            &mut refresh_events,
            &mut advance_hardware,
        )?;
        advance_source_span(
            &mut cursor,
            M68kCycleCount::from_raw(60),
            refresh.vint_handler,
            &mut refresh_events,
            &mut advance_hardware,
        )?;
        advance_source_span(
            &mut cursor,
            suffix,
            refresh.vertical_decoder_after_vint,
            &mut refresh_events,
            &mut advance_hardware,
        )?;
    } else {
        advance_source_span(
            &mut cursor,
            workload.vertical_art.decoder,
            refresh.vertical_decoder_before_vint,
            &mut refresh_events,
            &mut advance_hardware,
        )?;
        if refresh.vint_exception != 0
            || refresh.vint_handler != 0
            || refresh.vertical_decoder_after_vint != 0
        {
            return None;
        }
    }

    let vertical = load_in_game_platform_art_transfer_cursor_loc_14314_with_hardware(
        cursor,
        workload.vertical_art.output_words,
        &mut advance_hardware,
    )?;
    cursor = vertical.cursor;
    refresh_events = refresh_events.checked_add(vertical.refresh_events)?;
    advance_source_span(
        &mut cursor,
        workload.epilogue,
        refresh.epilogue,
        &mut refresh_events,
        &mut advance_hardware,
    )?;
    let vint_cycles = if vint.is_some() { 104 } else { 0 };
    let elapsed = workload
        .raw_cycles()
        .get()
        .checked_add(vint_cycles)?
        .checked_add(refresh_events.checked_mul(2)?)?
        .checked_add(horizontal.fifo_halted_ticks.get())?
        .checked_add(vertical.fifo_halted_ticks.get())?;

    Some(LoadInGamePlatformInitialisationAdvance {
        cursor,
        elapsed: M68kCycleCount::from_raw(elapsed),
        horizontal_fifo_halted_ticks: horizontal.fifo_halted_ticks,
        vertical_fifo_halted_ticks: vertical.fifo_halted_ticks,
        refresh_events,
    })
}

/// Advance the source tail from `Load_InGame`'s `j_sub_28FC` call at `$11A12`
/// to the first instruction of `StopMusic $E14C8`.
pub fn load_in_game_stop_music_cursor_from_sub_28fc(
    cursor: LoadInGameCpuRasterCursor,
    timer_path: crate::LoadInGameTimerBonusPath,
) -> LoadInGameCpuRasterCursor {
    let advance = crate::load_in_game_stop_music_tail_advance(cursor.refresh_phase, timer_path);
    LoadInGameCpuRasterCursor {
        beam: cursor.beam.advance_m68k_cycles(advance.elapsed),
        refresh_phase: advance.phase_after,
    }
}

/// Explicit hardware boundary at ASM `Mode_Level $7C8`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ModeLevelRasterSeedModeEntry {
    pub beam: H40BeamPosition,
}

/// VDP boundary where VINT asserts before `WaitForVint` resumes the main loop.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ModeLevelRasterSeedVintAssertion {
    pub beam: H40BeamPosition,
}

/// Hardware state at the first ASM `WaitForVint $93E` poll instruction.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WaitForVintPollSeedNoZ80 {
    pub first_poll_beam: H40BeamPosition,
    pub refresh_phase: M68kRefreshPhase,
}

/// Hardware state and ordered asynchronous Z80 68k-bus accesses at the first
/// ASM `WaitForVint $93E` poll instruction.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WaitForVintPollSeed<'a> {
    pub first_poll_beam: H40BeamPosition,
    pub refresh_phase: M68kRefreshPhase,
    /// Event-generation offsets in 68000 cycles from the first `$93E`.
    ///
    /// Each event occurs after the current 68000 scheduler tick and adds an
    /// 11-cycle wait to the following tick. Offsets must be strictly ordered.
    pub z80_68k_bus_access_offsets: &'a [M68kCycleCount],
}

/// Run the `$93E/$942` poll while merging ordered asynchronous Z80 accesses
/// to the 68k bus.
pub fn wait_for_vint_sample_with_z80_bus_accesses(
    seed: WaitForVintPollSeed<'_>,
    assertion: ModeLevelRasterSeedVintAssertion,
) -> Option<crate::WaitForVintInterruptSample> {
    #[derive(Clone, Copy)]
    enum PollPc {
        TestLoc93e,
        BranchLoc942,
    }

    const FRAME_MCLK: u32 = MCLK_CYCLES_PER_SCANLINE * NTSC_SCANLINES_PER_FRAME as u32;
    const INTERRUPT_ENTRY_RAW_CYCLES: u32 = 10 + 44;
    const MAX_SCHEDULER_TICKS_PER_FRAME: usize = 200_000;

    if seed
        .z80_68k_bus_access_offsets
        .windows(2)
        .any(|pair| pair[0].get() >= pair[1].get())
    {
        return None;
    }

    let poll_start = u32::from(seed.first_poll_beam.scanline) * MCLK_CYCLES_PER_SCANLINE
        + u32::from(seed.first_poll_beam.scanline_mclk);
    let assertion_at = u32::from(assertion.beam.scanline) * MCLK_CYCLES_PER_SCANLINE
        + u32::from(assertion.beam.scanline_mclk);
    let assertion_offset = assertion_at
        .checked_add(FRAME_MCLK)?
        .checked_sub(poll_start)?
        % FRAME_MCLK;

    let mut elapsed_cycles = 0_u32;
    let mut phase = seed.refresh_phase;
    let mut pc = PollPc::TestLoc93e;
    let mut pending_wait_cycles = 0_u32;
    let mut next_z80_access = 0_usize;

    for _ in 0..MAX_SCHEDULER_TICKS_PER_FRAME {
        if pending_wait_cycles != 0 {
            // jgenesis timing.rs `record_68k_instruction`: a pending wait
            // advances the refresh counter but cannot enqueue another
            // refresh. A refresh wait and Z80 steal can therefore be
            // processed together as one 13-cycle scheduler tick.
            let wait = phase.advance_external_wait(M68kCycleCount::from_raw(pending_wait_cycles));
            elapsed_cycles = elapsed_cycles.checked_add(wait.elapsed.get())?;
            phase = wait.phase_after;
            pending_wait_cycles = 0;

            while let Some(event) = seed.z80_68k_bus_access_offsets.get(next_z80_access) {
                if event.get() > elapsed_cycles {
                    break;
                }
                if event.get() != elapsed_cycles {
                    return None;
                }
                pending_wait_cycles = pending_wait_cycles.checked_add(11)?;
                next_z80_access += 1;
            }
            continue;
        }

        let elapsed_mclk = elapsed_cycles.checked_mul(MCLK_CYCLES_PER_M68K_CYCLE)?;
        if elapsed_mclk >= assertion_offset {
            // This contract covers bus accesses generated during the poll.
            // A schedule continuing into interrupt entry needs a wider typed
            // boundary so post-assertion events cannot be silently dropped.
            if next_z80_access != seed.z80_68k_bus_access_offsets.len() {
                return None;
            }
            let resume = match pc {
                PollPc::TestLoc93e => crate::WaitForVintResumePath::PollTestLoc93e,
                PollPc::BranchLoc942 => crate::WaitForVintResumePath::StaleBranchLoc942,
            };
            let interrupt_entry =
                phase.advance_rom(M68kCycleCount::from_raw(INTERRUPT_ENTRY_RAW_CYCLES));
            let after_handler_entry = interrupt_entry
                .phase_after
                .advance_rom(crate::wait_for_vint_source_cycles_to_mode_level(resume));
            return Some(crate::WaitForVintInterruptSample {
                mclk_until_interrupt_sample: elapsed_mclk.checked_sub(assertion_offset)?,
                interrupt_entry_refresh_wait_events: u16::try_from(interrupt_entry.refresh_events)
                    .ok()?,
                resume,
                arbitration_after_handler_entry: crate::WaitForVintArbitration {
                    refresh_wait_events: u16::try_from(after_handler_entry.refresh_events).ok()?,
                    z80_68k_bus_steals: 0,
                },
            });
        }

        // ASM WaitForVint: TST.B abs.w is 12 raw cycles; taken BEQ.S is 10.
        let raw_cycles = match pc {
            PollPc::TestLoc93e => 12,
            PollPc::BranchLoc942 => 10,
        };
        let phase_after_raw = u32::from(phase.get()).checked_add(raw_cycles)?;
        if phase_after_raw >= 128 {
            pending_wait_cycles = pending_wait_cycles.checked_add(2)?;
        }
        phase = M68kRefreshPhase::new(
            u8::try_from(phase_after_raw % 128).expect("refresh phase remains below 128"),
        )
        .expect("refresh phase remains below 128");
        elapsed_cycles = elapsed_cycles.checked_add(raw_cycles)?;
        pc = match pc {
            PollPc::TestLoc93e => PollPc::BranchLoc942,
            PollPc::BranchLoc942 => PollPc::TestLoc93e,
        };

        // jgenesis ticks the Z80 after the completed 68000 scheduler tick.
        // Banked reads generated there add their wait to the next tick.
        while let Some(event) = seed.z80_68k_bus_access_offsets.get(next_z80_access) {
            if event.get() > elapsed_cycles {
                break;
            }
            if event.get() != elapsed_cycles {
                return None;
            }
            pending_wait_cycles = pending_wait_cycles.checked_add(11)?;
            next_z80_access += 1;
        }
    }
    None
}

/// Run the `$93E/$942` poll until VINT without asynchronous Z80 bus access.
///
/// This is deliberately fail-closed to the no-Z80 path. A general scheduler
/// must merge ordered Z80 68k-bus accesses before calling the VINT transition.
pub fn wait_for_vint_sample_without_z80_bus_access(
    seed: WaitForVintPollSeedNoZ80,
    assertion: ModeLevelRasterSeedVintAssertion,
) -> Option<crate::WaitForVintInterruptSample> {
    wait_for_vint_sample_with_z80_bus_accesses(
        WaitForVintPollSeed {
            first_poll_beam: seed.first_poll_beam,
            refresh_phase: seed.refresh_phase,
            z80_68k_bus_access_offsets: &[],
        },
        assertion,
    )
}

/// Derive the `Mode_Level $7C8` beam from a VINT assertion and sampled poll state.
pub fn mode_level_raster_seed_mode_entry_from_vint_assertion(
    seed: ModeLevelRasterSeedVintAssertion,
    sample: crate::WaitForVintInterruptSample,
) -> Option<ModeLevelRasterSeedModeEntry> {
    let elapsed = crate::wait_for_vint_mclk_from_vint_assertion_to_mode_level(sample)?;
    Some(ModeLevelRasterSeedModeEntry {
        beam: seed.beam.advance_mclk(elapsed),
    })
}

/// Advance a typed natural `Mode_Level` VBlank path to `sub_E12`.
pub fn mode_level_raster_seed_sub_e12_from_mode_entry(
    seed: ModeLevelRasterSeedModeEntry,
    work: crate::ModeLevelVblankWorkBeforeSubE12,
    arbitration: crate::ModeLevelVblankArbitrationBeforeSubE12,
) -> Option<ModeLevelRasterSeedSubE12> {
    let advance = crate::mode_level_vblank_advance_before_sub_e12(work, arbitration)?;
    Some(ModeLevelRasterSeedSubE12 {
        beam: seed.beam.advance_m68k_cycles(advance.elapsed),
        refresh_phase: advance.phase_after,
    })
}

/// Advance the source-typed, VDP-idle interval from `sub_E12` to `sub_44DC`.
pub fn mode_level_cpu_raster_cursor_sub_44dc(
    seed: ModeLevelRasterSeedSubE12,
    ledger: &crate::ModeLevelPreHudWorkLedger,
) -> Option<ModeLevelCpuRasterCursor> {
    let execute =
        crate::execute_objects_elapsed_cycles_sub_e12(&ledger.execute_objects, seed.refresh_phase)?;
    let post = crate::post_execute_elapsed_cycles_before_sub_44dc(ledger, execute.phase_after)?;
    Some(ModeLevelCpuRasterCursor {
        beam: seed
            .beam
            .advance_m68k_cycles(execute.elapsed)
            .advance_m68k_cycles(post.elapsed),
        refresh_phase: post.phase_after,
    })
}

/// Derive the life-icon FIFO timing from a seeded `sub_E12` boundary and the
/// source-typed remainder of `Mode_Level`.
pub fn life_icon_raster_timing_from_sub_e12(
    seed: ModeLevelRasterSeedSubE12,
    ledger: &crate::ModeLevelPreHudWorkLedger,
    command: crate::HudDmaEventSub44DC,
    previous_frame_table_byte_offset: u8,
) -> Option<crate::HudRasterUploadTimingSub44DC> {
    if !matches!(command, crate::HudDmaEventSub44DC::LifeIcon { .. }) {
        return None;
    }
    let cursor = mode_level_cpu_raster_cursor_sub_44dc(seed, ledger)?;
    let first_commit = life_icon_first_commit_loc_4586(cursor.beam, cursor.refresh_phase);
    Some(crate::HudRasterUploadTimingSub44DC {
        command,
        previous_frame_table_byte_offset,
        first_commit_scanline: first_commit.scanline,
        first_commit_pixel: first_commit.pixel,
    })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct H40FifoCommitPosition {
    pub scanline: u16,
    pub pixel: u16,
}

fn h40_scanline_mclk_to_pixel(scanline_mclk: u16) -> u16 {
    // jgenesis VDP `scanline_mclk_to_pixel_h40`: H40 uses mclk/8 outside
    // HSYNC, with the documented 8/10/9/10 divider pattern during HSYNC.
    const INTERNAL_H_JUMP: u16 = 0x1C9 - 0x16D;
    const HSYNC_START_MCLK: u16 = (0x1CC - INTERNAL_H_JUMP) * 8;
    const HSYNC_PATTERN_MCLK: u16 = 8 + 7 * 10 + 2 * 9 + 7 * 10;
    const HSYNC_END_MCLK: u16 = HSYNC_START_MCLK + 2 * HSYNC_PATTERN_MCLK;

    if scanline_mclk < HSYNC_START_MCLK {
        return scanline_mclk / 8;
    }
    if scanline_mclk < HSYNC_END_MCLK {
        let hsync_mclk = scanline_mclk - HSYNC_START_MCLK;
        let pattern_mclk = hsync_mclk % HSYNC_PATTERN_MCLK;
        let pattern_pixel = match pattern_mclk {
            0..=7 => 0,
            8..=77 => 1 + (pattern_mclk - 8) / 10,
            78..=95 => 8 + (pattern_mclk - 78) / 9,
            96..=165 => 10 + (pattern_mclk - 96) / 10,
            _ => unreachable!("HSYNC pattern remainder stays below 166"),
        };
        return if hsync_mclk < HSYNC_PATTERN_MCLK {
            0x1CC - INTERNAL_H_JUMP + pattern_pixel
        } else {
            0x1CC - INTERNAL_H_JUMP + 17 + pattern_pixel
        };
    }
    0x1CC - INTERNAL_H_JUMP + 34 + (scanline_mclk - HSYNC_END_MCLK) / 8
}

/// First FIFO-committed word of ASM `loc_4586 -> sub_44D0`.
///
/// The target path has an empty FIFO at `sub_44DC` entry. The first MOVE.L
/// completes after the life-script branch and places two words into the FIFO;
/// the first word commits at the next H40 VRAM access slot.
pub fn life_icon_first_commit_loc_4586(
    entry_beam: H40BeamPosition,
    entry_refresh_phase: M68kRefreshPhase,
) -> H40FifoCommitPosition {
    // ASM sub_44DC loc_4546..loc_44D4 (kid.asm:5592-5541): 236 raw cycles
    // reach completion of the first MOVE.L data-port write on this branch.
    let first_write = entry_refresh_phase.advance_rom(M68kCycleCount::from_raw(236));
    let arrival = entry_beam.advance_m68k_cycles(first_write.elapsed);
    let arrival_pixel = h40_scanline_mclk_to_pixel(arrival.scanline_mclk);
    let (first_commit_scanline, first_commit_pixel) = H40_VRAM_FIFO_ACCESS_PIXELS
        .iter()
        .copied()
        .find(|&pixel| pixel > arrival_pixel)
        .map_or_else(
            || {
                (
                    arrival.scanline.wrapping_add(1) % NTSC_SCANLINES_PER_FRAME,
                    H40_VRAM_FIFO_ACCESS_PIXELS[0],
                )
            },
            |pixel| (arrival.scanline, pixel),
        );
    H40FifoCommitPosition {
        scanline: first_commit_scanline,
        pixel: first_commit_pixel,
    }
}

/// All 64 visible VRAM-word commits from `loc_4586 -> sub_44D0`.
///
/// Once the first word commits, the 32-longword copy keeps the four-entry FIFO
/// fed. A VRAM word occupies two H40 access slots, so successive visible word
/// commits are two slots apart.
pub fn life_icon_fifo_commit_positions_loc_4586(
    entry: ModeLevelCpuRasterCursor,
) -> Option<[H40FifoCommitPosition; 64]> {
    let first = life_icon_first_commit_loc_4586(entry.beam, entry.refresh_phase);
    let first_slot = H40_VRAM_FIFO_ACCESS_PIXELS
        .iter()
        .position(|&pixel| pixel == first.pixel)?;
    Some(core::array::from_fn(|word_index| {
        let slot = first_slot + 2 * word_index;
        H40FifoCommitPosition {
            scanline: first
                .scanline
                .wrapping_add((slot / H40_VRAM_FIFO_ACCESS_PIXELS.len()) as u16)
                % NTSC_SCANLINES_PER_FRAME,
            pixel: H40_VRAM_FIFO_ACCESS_PIXELS[slot % H40_VRAM_FIFO_ACCESS_PIXELS.len()],
        }
    }))
}

fn h40_mclk_after_pixel(pixel: u16) -> Option<u16> {
    (0..MCLK_CYCLES_PER_SCANLINE as u16).find(|&mclk| h40_scanline_mclk_to_pixel(mclk) > pixel)
}

fn life_icon_last_pending_enqueue_cursor_loc_44d4(
    entry: ModeLevelCpuRasterCursor,
) -> Option<ModeLevelCpuRasterCursor> {
    let first = life_icon_first_commit_loc_4586(entry.beam, entry.refresh_phase);
    let first_slot = H40_VRAM_FIFO_ACCESS_PIXELS
        .iter()
        .position(|&pixel| pixel == first.pixel)?;

    // ASM sub_44D0 emits 64 words into a four-entry FIFO. The final pending
    // word enters on the access slot seven slots before its visible commit:
    // three older words each retain their second VRAM slot, then the final
    // word consumes its first. Since visible commits are two slots apart,
    // this is first + 2*63 - 7.
    let enqueue_slot = first_slot.checked_add(2 * 63)?.checked_sub(7)?;
    let enqueue_scanline = first
        .scanline
        .wrapping_add((enqueue_slot / H40_VRAM_FIFO_ACCESS_PIXELS.len()) as u16)
        % NTSC_SCANLINES_PER_FRAME;
    let enqueue_pixel =
        H40_VRAM_FIFO_ACCESS_PIXELS[enqueue_slot % H40_VRAM_FIFO_ACCESS_PIXELS.len()];
    let enqueue_after_pixel_mclk = u32::from(h40_mclk_after_pixel(enqueue_pixel)?);

    let frame_mclk = MCLK_CYCLES_PER_SCANLINE * u32::from(NTSC_SCANLINES_PER_FRAME);
    let entry_mclk = u32::from(entry.beam.scanline) * MCLK_CYCLES_PER_SCANLINE
        + u32::from(entry.beam.scanline_mclk);
    let enqueue_after_pixel =
        u32::from(enqueue_scanline) * MCLK_CYCLES_PER_SCANLINE + enqueue_after_pixel_mclk;
    let forward_mclk = enqueue_after_pixel
        .checked_add(frame_mclk)?
        .checked_sub(entry_mclk)?
        % frame_mclk;

    // VDP slots are processed when a 68000 tick advances beyond the access
    // pixel. The returned cursor is immediately before that final one-cycle
    // halted tick; the tick itself empties the pending-write queue.
    let cycles_before_final_halt = forward_mclk
        .checked_add(6)?
        .checked_div(7)?
        .checked_sub(1)?;
    Some(ModeLevelCpuRasterCursor {
        beam: entry
            .beam
            .advance_m68k_cycles(M68kCycleCount::from_raw(cycles_before_final_halt)),
        // Every preceding halted tick resets jgenesis' refresh counter.
        refresh_phase: M68kRefreshPhase::new(0).expect("zero is a valid refresh phase"),
    })
}

fn advance_life_icon_non_vdp_interval(
    fifo: &mut PlatformArtFifoScheduler,
    cycles: M68kCycleCount,
    advance_hardware: &mut impl FnMut(M68kCycleCount, bool) -> Option<M68kCycleCount>,
) -> Option<M68kCycleCount> {
    fifo.advance_m68k_cycles(cycles);
    advance_hardware(cycles, false)
}

fn advance_life_icon_source_instruction(
    fifo: &mut PlatformArtFifoScheduler,
    phase: &mut M68kRefreshPhase,
    raw_cycles: u32,
    advance_hardware: &mut impl FnMut(M68kCycleCount, bool) -> Option<M68kCycleCount>,
) -> Option<()> {
    let phase_after_raw = u32::from(phase.get()).checked_add(raw_cycles)?;
    let refresh_wait_cycles = if phase_after_raw >= 128 { 2 } else { 0 };
    *phase = M68kRefreshPhase::new(
        u8::try_from(phase_after_raw % 128).expect("refresh phase remains below 128"),
    )
    .expect("refresh phase remains below 128");

    let mut pending_wait = advance_life_icon_non_vdp_interval(
        fifo,
        M68kCycleCount::from_raw(raw_cycles),
        advance_hardware,
    )?
    .get()
    .checked_add(refresh_wait_cycles)?;
    while pending_wait != 0 {
        let wait = M68kCycleCount::from_raw(pending_wait);
        *phase = phase.advance_external_wait(wait).phase_after;
        pending_wait = advance_life_icon_non_vdp_interval(fifo, wait, advance_hardware)?.get();
    }
    Some(())
}

fn drain_life_icon_pending_fifo_words(
    fifo: &mut PlatformArtFifoScheduler,
    phase: &mut M68kRefreshPhase,
    advance_hardware: &mut impl FnMut(M68kCycleCount, bool) -> Option<M68kCycleCount>,
) -> Option<()> {
    while fifo.pending_words != 0 {
        let tick = M68kCycleCount::from_raw(1);
        fifo.advance_m68k_cycles(tick);
        if advance_hardware(tick, true)?.get() != 0 {
            return None;
        }
        // jgenesis `record_68k_instruction` resets the approximate refresh
        // counter on every tick where the VDP owns the 68000 bus.
        *phase = M68kRefreshPhase::new(0).expect("zero is a valid refresh phase");
    }
    Some(())
}

/// Co-schedule the life-icon-only `sub_44DC` path with asynchronous hardware.
///
/// The callback advances one chronological 68000 interval and returns newly
/// queued external CPU-wait cycles. `vdp_owns_bus` is true only for FIFO-full
/// retry ticks; hardware accesses during those ticks must not return a CPU
/// wait. This is the port used by a persistent GEMS machine to replace
/// route-specific Z80-access offsets.
pub fn life_icon_cpu_raster_cursor_after_sub_44dc_with_scheduler(
    entry: ModeLevelCpuRasterCursor,
    commands: &[crate::HudDmaEventSub44DC],
    mut advance_hardware: impl FnMut(M68kCycleCount, bool) -> Option<M68kCycleCount>,
) -> Option<ModeLevelCpuRasterCursor> {
    crate::mode_level_timing::hud_life_icon_only_raw_cycles_sub_44dc(commands)?;

    let mut fifo = PlatformArtFifoScheduler::new(entry.beam);
    let mut phase = entry.refresh_phase;

    // ASM sub_44DC -> loc_4586 -> sub_44D0, through completion of the first
    // loc_44D4 MOVE.L. The life-icon-only branch skips the coin upload.
    const PRELUDE_THROUGH_FIRST_MOVE: [u32; 23] = [
        12, 4, 4, 10, 12, 12, 4, 4, 12, 12, 4, 4, 12, 12, 8, 10, 8, 4, 22, 20, 18, 8, 20,
    ];
    debug_assert_eq!(PRELUDE_THROUGH_FIRST_MOVE.into_iter().sum::<u32>(), 236);
    for raw_cycles in PRELUDE_THROUGH_FIRST_MOVE {
        advance_life_icon_source_instruction(
            &mut fifo,
            &mut phase,
            raw_cycles,
            &mut advance_hardware,
        )?;
    }
    fifo.push_word();
    fifo.push_word();
    drain_life_icon_pending_fifo_words(&mut fifo, &mut phase, &mut advance_hardware)?;

    // ASM loc_44D4: the remaining 31 MOVE.L writes each place two words into
    // the four-entry FIFO. Taken DBFs precede writes 2..32; the final DBF and
    // the routine/section epilogue follow the final enqueue.
    for _ in 1..32 {
        advance_life_icon_source_instruction(&mut fifo, &mut phase, 10, &mut advance_hardware)?;
        advance_life_icon_source_instruction(&mut fifo, &mut phase, 20, &mut advance_hardware)?;
        fifo.push_word();
        fifo.push_word();
        drain_life_icon_pending_fifo_words(&mut fifo, &mut phase, &mut advance_hardware)?;
    }

    const TRAILING_INSTRUCTIONS: [u32; 19] = [
        14, 16, 12, 12, 4, 4, 10, 12, 12, 4, 4, 10, 12, 12, 4, 4, 10, 12, 16,
    ];
    for raw_cycles in TRAILING_INSTRUCTIONS {
        advance_life_icon_source_instruction(
            &mut fifo,
            &mut phase,
            raw_cycles,
            &mut advance_hardware,
        )?;
    }

    Some(ModeLevelCpuRasterCursor {
        beam: fifo.beam,
        refresh_phase: phase,
    })
}

/// Advance the classified life-icon-only `sub_44DC` path through FIFO stalls
/// to the following `Manage_EnemyLoading` call.
///
/// Z80 offsets are generation times in 68000 cycles from the final pending
/// FIFO enqueue cursor. They must coincide with an instruction boundary; the
/// resulting 11-cycle bus steals are merged with any pending refresh wait.
pub fn life_icon_cpu_raster_cursor_after_sub_44dc(
    entry: ModeLevelCpuRasterCursor,
    commands: &[crate::HudDmaEventSub44DC],
    z80_68k_bus_access_offsets: &[M68kCycleCount],
) -> Option<ModeLevelCpuRasterCursor> {
    crate::mode_level_timing::hud_life_icon_only_raw_cycles_sub_44dc(commands)?;
    if z80_68k_bus_access_offsets
        .windows(2)
        .any(|pair| pair[0].get() >= pair[1].get())
    {
        return None;
    }

    let pending_enqueue = life_icon_last_pending_enqueue_cursor_loc_44d4(entry)?;
    let mut elapsed = 1_u32;
    let mut phase = M68kRefreshPhase::new(0).expect("zero is a valid refresh phase");
    let mut next_z80_access = 0_usize;

    // ASM loc_44D4 final DBF, sub_44D0 RTS, three skipped HUD sections,
    // `$FA5C` store, and outer RTS. The initial one-cycle tick above is the
    // final VDP-halted retry at loc_44D6.
    const TRAILING_INSTRUCTIONS: [u32; 19] = [
        14, 16, 12, 12, 4, 4, 10, 12, 12, 4, 4, 10, 12, 12, 4, 4, 10, 12, 16,
    ];

    for raw_cycles in TRAILING_INSTRUCTIONS {
        let phase_after_raw = u32::from(phase.get()).checked_add(raw_cycles)?;
        let mut pending_wait_cycles: u32 = if phase_after_raw >= 128 { 2 } else { 0 };
        phase = M68kRefreshPhase::new(
            u8::try_from(phase_after_raw % 128).expect("refresh phase remains below 128"),
        )
        .expect("refresh phase remains below 128");
        elapsed = elapsed.checked_add(raw_cycles)?;

        while let Some(event) = z80_68k_bus_access_offsets.get(next_z80_access) {
            if event.get() > elapsed {
                break;
            }
            if event.get() != elapsed {
                return None;
            }
            pending_wait_cycles = pending_wait_cycles.checked_add(11)?;
            next_z80_access += 1;
        }

        if pending_wait_cycles != 0 {
            let wait = phase.advance_external_wait(M68kCycleCount::from_raw(pending_wait_cycles));
            elapsed = elapsed.checked_add(wait.elapsed.get())?;
            phase = wait.phase_after;
        }
    }

    if next_z80_access != z80_68k_bus_access_offsets.len() {
        return None;
    }

    Some(ModeLevelCpuRasterCursor {
        beam: pending_enqueue
            .beam
            .advance_m68k_cycles(M68kCycleCount::from_raw(elapsed)),
        refresh_phase: phase,
    })
}

#[derive(Debug, Clone, Copy)]
struct IntroSequence5RasterUpload {
    previous_frame_index: u16,
    frame_index: u16,
    frame_start: (u16, u16),
    panel_start: Option<(u16, u16)>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum IntroSequence5WorkerLineage {
    Shared,
    TimeoutFc82Zero,
    TimeoutFc82OneStartSkip,
    TimeoutFc82Two,
}

impl IntroSequence5WorkerLineage {
    fn tile_worker_delay(self) -> usize {
        // ASM loc_1BC36 runs on an independently scheduled Object. Oracle
        // FC82=2 frames 20591..20856 retain each previous map for one outer
        // trace after the FC82=0/1 presentation clock used by earlier visits.
        usize::from(self == Self::TimeoutFc82Two)
    }
}

#[allow(non_snake_case)]
fn intro_sequence_5_raster_upload_loc_1BC36(
    elapsed: usize,
    lineage: IntroSequence5WorkerLineage,
) -> Option<IntroSequence5RasterUpload> {
    // ASM sub_1C034/loc_1C09E..loc_1C0D6 and sub_1C0EE/sub_1C5D0.
    // Positions are FIFO-pop times measured from jgenesis on both the cold and
    // timeout-restart lineages; they are routine-local, not TAS frame numbers.
    match elapsed {
        INTRO_SEQUENCE_5_LOC_1BC36_FRAME_1_RASTER_ELAPSED => Some(IntroSequence5RasterUpload {
            previous_frame_index: 0,
            frame_index: 1,
            frame_start: (74, 76),
            panel_start: None,
        }),
        INTRO_SEQUENCE_5_LOC_1BC36_FRAME_2_RASTER_ELAPSED => Some(IntroSequence5RasterUpload {
            previous_frame_index: 1,
            frame_index: 2,
            frame_start: (96, 76),
            panel_start: None,
        }),
        INTRO_SEQUENCE_5_LOC_1BC36_FRAME_3_RASTER_ELAPSED => Some(IntroSequence5RasterUpload {
            previous_frame_index: 2,
            frame_index: 3,
            frame_start: (108, 108),
            panel_start: None,
        }),
        INTRO_SEQUENCE_5_LOC_1BC36_FRAME_4_RASTER_ELAPSED => Some(IntroSequence5RasterUpload {
            previous_frame_index: 3,
            frame_index: 4,
            frame_start: (132, 380),
            panel_start: Some((196, 140)),
        }),
        INTRO_SEQUENCE_5_LOC_1BC36_FRAME_5_VISIBLE_ELAPSED => Some(IntroSequence5RasterUpload {
            previous_frame_index: 4,
            frame_index: 5,
            frame_start: (139, 12),
            panel_start: None,
        }),
        INTRO_SEQUENCE_5_LOC_1BC36_FRAME_6_RASTER_ELAPSED
            if lineage == IntroSequence5WorkerLineage::TimeoutFc82Two =>
        {
            Some(IntroSequence5RasterUpload {
                previous_frame_index: 5,
                frame_index: 6,
                frame_start: (125, 172),
                panel_start: None,
            })
        }
        INTRO_SEQUENCE_5_LOC_1BC36_FRAME_7_RASTER_ELAPSED
            if lineage == IntroSequence5WorkerLineage::TimeoutFc82Two =>
        {
            Some(IntroSequence5RasterUpload {
                previous_frame_index: 6,
                frame_index: 7,
                frame_start: (114, 204),
                panel_start: None,
            })
        }
        INTRO_SEQUENCE_5_LOC_1BC36_RETURN_FRAME_6_RASTER_ELAPSED
            if lineage == IntroSequence5WorkerLineage::TimeoutFc82Two =>
        {
            Some(IntroSequence5RasterUpload {
                previous_frame_index: 7,
                frame_index: 6,
                frame_start: (125, 172),
                panel_start: None,
            })
        }
        INTRO_SEQUENCE_5_LOC_1BC36_FINAL_FRAME_5_RASTER_ELAPSED
            if lineage == IntroSequence5WorkerLineage::TimeoutFc82Zero =>
        {
            Some(IntroSequence5RasterUpload {
                previous_frame_index: 6,
                frame_index: 5,
                frame_start: (136, 204),
                panel_start: None,
            })
        }
        INTRO_SEQUENCE_5_LOC_1BC36_FINAL_FRAME_5_RASTER_ELAPSED
            if lineage == IntroSequence5WorkerLineage::TimeoutFc82Two =>
        {
            Some(IntroSequence5RasterUpload {
                previous_frame_index: 6,
                frame_index: 5,
                frame_start: (134, 380),
                panel_start: None,
            })
        }
        _ => None,
    }
}

#[allow(non_snake_case)]
fn overlay_intro_sequence_5_tile_worker_loc_1BC36(
    vdp: &mut VdpState,
    data: &IntroSub1B850Data,
    elapsed: usize,
    lineage: IntroSequence5WorkerLineage,
) {
    // ASM loc_1BC36 (kid.asm:30516) calls sub_1C034 before each hibernate.
    // sub_1C034 decodes one `off_1C126` map with base tile `$A340` into a
    // 16x25 staging buffer, then streams 25 fixed rows into the Scroll A
    // nametable at VRAM `$0194..$0D94`.
    let raster_upload = intro_sequence_5_raster_upload_loc_1BC36(elapsed, lineage);
    let frame_index = INTRO_SEQUENCE_5_LOC_1BC36_FRAME_TIMELINE
        .iter()
        .rev()
        .find_map(|(start_elapsed, frame_index)| {
            (elapsed >= *start_elapsed).then_some(*frame_index)
        });
    if let Some(frame_index) = frame_index {
        let initial_vram = raster_upload.map(|upload| {
            let mut initial_vdp = vdp.clone();
            apply_intro_sequence_5_sub_1C034_frame(
                &mut initial_vdp,
                data,
                upload.previous_frame_index,
            );
            initial_vdp.vram
        });
        apply_intro_sequence_5_sub_1C034_frame(vdp, data, frame_index);

        if let (Some(upload), Some(initial_vram)) = (raster_upload, initial_vram) {
            let mut writes = intro_sequence_5_sub_1C034_word_writes(data, upload.frame_index);
            let mut vram_word_writes = schedule_h40_vram_fifo_word_writes(
                &writes,
                upload.frame_start.0,
                upload.frame_start.1,
            );
            if let Some(panel_start) = upload.panel_start {
                apply_intro_sequence_5_panel_sub_1C0EE(vdp, data, 3, 0x13);
                writes = intro_sequence_5_panel_word_writes_sub_1C0EE(data, 3, 0x13);
                vram_word_writes.extend(schedule_h40_vram_fifo_word_writes(
                    &writes,
                    panel_start.0,
                    panel_start.1,
                ));
            }
            vdp.raster_frame = Some(VdpRasterFrame {
                initial_vram,
                initial_cram: None,
                vram_word_writes,
            });
        }
    }

    if raster_upload.is_none()
        && (INTRO_SEQUENCE_5_LOC_1BC36_PANEL_VISIBLE_ELAPSED
            ..INTRO_SEQUENCE_5_LOC_1BC36_PANEL_CLEAR_ELAPSED)
            .contains(&elapsed)
    {
        apply_intro_sequence_5_panel_sub_1C0EE(vdp, data, 3, 0x13);
    }
}

fn schedule_h40_vram_fifo_word_writes(
    writes: &[(u16, u16)],
    start_scanline: u16,
    start_pixel: u16,
) -> Vec<VdpRasterVramWordWrite> {
    try_schedule_h40_vram_fifo_word_writes(writes, start_scanline, start_pixel)
        .expect("oracle-derived FIFO start pixel must be an H40 access slot")
}

/// Schedule VRAM words from a proven first H40 FIFO commit position.
///
/// Returns `None` when `start_pixel` is not a hardware access slot. Each VRAM
/// word occupies two access slots, matching the renderer's existing
/// byte-access timing model.
pub fn try_schedule_h40_vram_fifo_word_writes(
    writes: &[(u16, u16)],
    start_scanline: u16,
    start_pixel: u16,
) -> Option<Vec<VdpRasterVramWordWrite>> {
    let start_slot = H40_VRAM_FIFO_ACCESS_PIXELS
        .iter()
        .position(|&pixel| pixel == start_pixel)?;
    Some(
        writes
            .iter()
            .flat_map(|&(byte_addr, value)| [(byte_addr, value); 2])
            .enumerate()
            .map(|(write_index, (byte_addr, value))| {
                let slot = start_slot + write_index;
                VdpRasterVramWordWrite {
                    scanline: start_scanline + (slot / H40_VRAM_FIFO_ACCESS_PIXELS.len()) as u16,
                    pixel: H40_VRAM_FIFO_ACCESS_PIXELS[slot % H40_VRAM_FIFO_ACCESS_PIXELS.len()],
                    byte_addr,
                    value,
                }
            })
            .collect(),
    )
}

#[allow(non_snake_case)]
fn apply_intro_sequence_5_sub_1C034_frame(
    vdp: &mut VdpState,
    data: &IntroSub1B850Data,
    frame_index: u16,
) {
    let Ok(plan) = wildside_data::intro_sequence_5_frame_load_plan_sub_1C034(frame_index) else {
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
fn intro_sequence_5_sub_1C034_word_writes(
    data: &IntroSub1B850Data,
    frame_index: u16,
) -> Vec<(u16, u16)> {
    let Ok(plan) = wildside_data::intro_sequence_5_frame_load_plan_sub_1C034(frame_index) else {
        return Vec::new();
    };
    let mut staged_vdp = VdpState::new();
    apply_intro_sequence_5_sub_1C034_frame(&mut staged_vdp, data, frame_index);
    let mut writes = Vec::new();
    for row_write in plan.vram_rows {
        for word_offset in 0..row_write.word_count {
            let byte_addr = row_write.vram_word_address + word_offset * 2;
            let byte_offset = usize::from(byte_addr);
            writes.push((
                byte_addr,
                u16::from_be_bytes([
                    staged_vdp.vram[byte_offset],
                    staged_vdp.vram[byte_offset + 1],
                ]),
            ));
        }
    }
    writes
}

#[allow(non_snake_case)]
fn intro_sequence_5_panel_word_writes_sub_1C0EE(
    data: &IntroSub1B850Data,
    plane_cell_x: u16,
    plane_cell_y: u16,
) -> Vec<(u16, u16)> {
    let plan =
        wildside_data::intro_sequence_5_panel_load_plan_sub_1C0EE(plane_cell_x, plane_cell_y);
    let width = usize::from(plan.plane_upload.width_words);
    let height = usize::from(plan.plane_upload.height_words);
    (0..height)
        .flat_map(|row| {
            (0..width).filter_map(move |col| {
                let value = *data.intro_sequence_5_panel_tilemap.get(row * width + col)?;
                let byte_addr = (usize::from(plan.plane_upload.plane_cell_y) + row)
                    * NAMETABLE_ROW_STRIDE_BYTES
                    + (usize::from(plan.plane_upload.plane_cell_x) + col) * 2;
                Some((u16::try_from(byte_addr).ok()?, value))
            })
        })
        .collect()
}

#[allow(non_snake_case)]
fn apply_intro_sequence_5_panel_sub_1C0EE(
    vdp: &mut VdpState,
    data: &IntroSub1B850Data,
    plane_cell_x: u16,
    plane_cell_y: u16,
) {
    let plan =
        wildside_data::intro_sequence_5_panel_load_plan_sub_1C0EE(plane_cell_x, plane_cell_y);
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
#[cfg(test)]
fn apply_intro_sequence_5_blank_panel_sub_1C112(
    vdp: &mut VdpState,
    plane_cell_x: u16,
    plane_cell_y: u16,
) {
    let plan =
        wildside_data::intro_sequence_5_blank_panel_load_plan_sub_1C112(plane_cell_x, plane_cell_y);
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
    /// 4 sub_1B2A4 upload slots; slot 1 carries the default Red Stealth art.
    art_chunks: [(usize, Vec<u8>); 4],
    /// `$FFFFFC82`-selected art uploaded to VRAM $21E0 by sub_1B2A4.
    transformation_art_variants: [Vec<u8>; 3],
    /// 4 decompressed Saxman art chunks from ASM `Load_TitleArt`.
    title_art_chunks: [(usize, Vec<u8>); 4],
    /// Decoded Enigma tilemap for plane A (written starting at word $20C1).
    plane_a_tilemap: Vec<u16>,
    /// Plane-B title text tilemap rows from ASM `sub_1B532`.
    title_text_chameleon_rows: [[u16; 64]; 28],
    /// Decoded Scroll-A Chameleon animation tilemaps from ASM `sub_1B71A`.
    title_text_chameleon_anim_maps: [Vec<u16>; 3],
    /// Decoded interactive menu tilemap from ASM `Obj_TitleMenu loc_1BB52`.
    title_menu_tilemap_words: Vec<u16>,
    /// Decoded title character frames from ASM `sub_1BF24`.
    character_frames: Vec<TitleScreenCharacterFrameData>,
    /// Uncompressed flash/mixed-pixel tiles from ASM `ArtUnc_2A4D6`.
    lightning_art_bytes: Vec<u8>,
    /// 32-word tile patch for VRAM $F000.
    tile_patch_words: [u16; 32],
    /// Palette words from Pal_1C400 (title.bin — 0x33 words).
    palette_words: Vec<u16>,
    /// ASM loc_1C4E2 overlays at Palette_Buffer+$1A for `$FFFFFC82=1/2`.
    palette_variant_overlays: [Vec<u16>; 2],
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
    title_letter_palette_words: Vec<u16>,
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
pub(crate) fn load_title_screen_data(source: &dyn AssetSource) -> Result<TitleScreenData> {
    let scenes = std::path::Path::new("scenes");

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
        let compressed = source
            .read(&path)
            .with_context(|| format!("failed to read title art: {}", path.display()))?;
        let decompressed = wildside_data::decode_saxman(&compressed)
            .with_context(|| format!("failed to decompress title art: {filename}"))?;
        // VDP addresses are byte addresses (see intro loader comment).
        art_chunks.push((*vram_addr as usize, decompressed));
    }
    let art_chunks: [(usize, Vec<u8>); 4] = art_chunks
        .try_into()
        .map_err(|_| anyhow::anyhow!("expected 4 art chunks"))?;
    let transformation_art_variants = [
        art_chunks[1].1.clone(),
        load_saxman_art(
            source,
            &scenes.join("artcomp/Maniaxe_transformation_from_intro.bin"),
        )?,
        load_saxman_art(
            source,
            &scenes.join("artcomp/Juggernaut_transformation_from_intro.bin"),
        )?,
    ];

    // ASM Load_TitleArt (kid.asm:29810): `DecompressToRAM` expands
    // Title_text_CHAMELEON through RAM staging, then writes the transformed
    // words to the VDP data port at `$6760`; the remaining three chunks go
    // through the direct VRAM decompressor.
    let title_art_files: [(u16, &str, bool); 4] = [
        (0x6760, "artcomp/Title_text_CHAMELEON.bin", true),
        (0x9A60, "artcomp/Title_text_KID.bin", false),
        (0xA360, "artcomp/Title_menu.bin", false),
        (0xA980, "artcomp/Title_sparkles.bin", false),
    ];
    let mut title_art_chunks = Vec::new();
    for (vram_addr, filename, uses_ram_permutation) in &title_art_files {
        let path = scenes.join(filename);
        let compressed = source
            .read(&path)
            .with_context(|| format!("failed to read title art: {}", path.display()))?;
        let mut decompressed = wildside_data::decode_saxman(&compressed)
            .with_context(|| format!("failed to decompress title art: {filename}"))?;
        if *uses_ram_permutation {
            decompressed = title_text_chameleon_art_loc_1B4F4(&decompressed);
        }
        title_art_chunks.push((*vram_addr as usize, decompressed));
    }
    let title_art_chunks: [(usize, Vec<u8>); 4] = title_art_chunks
        .try_into()
        .map_err(|_| anyhow::anyhow!("expected 4 title art chunks"))?;

    // Decode Enigma tilemap: unk_2F7D0 with base tile $20C1.
    // ASM sub_1B2A4: `move.w #$20C1,d0` then `jsr (j_EniDec)`.
    // After decode, the caller (Load_IntroSequence6 / Load_TitleScreen) calls
    // sub_1C5D0 with d4=$15, d5=1, d6=$12, d7=$14 → 18×20 at cell (21,1).
    let tilemap_path = scenes.join("mapeni/2F7D0.bin");
    let tilemap_raw = source
        .read(&tilemap_path)
        .with_context(|| format!("failed to read title tilemap: {}", tilemap_path.display()))?;
    let plane_a_tilemap = decode_enigma_tilemap_loc_134D4(&tilemap_raw, 0x20C1)
        .context("failed to decode title tilemap")?;
    let chameleon_tilemap_path = scenes.join("mapeni/2F840.bin");
    let chameleon_tilemap_raw = source.read(&chameleon_tilemap_path).with_context(|| {
        format!(
            "failed to read title chameleon tilemap: {}",
            chameleon_tilemap_path.display()
        )
    })?;
    let chameleon_decoded_words = decode_enigma_tilemap_loc_134D4(&chameleon_tilemap_raw, 0x0331)
        .context("failed to decode title chameleon tilemap")?;
    let title_text_chameleon_upload =
        wildside_data::title_text_chameleon_tilemap_upload_sub_1B532(&chameleon_decoded_words)
            .context("failed to build title chameleon tilemap upload")?;
    let title_text_chameleon_anim_maps = [
        decode_title_text_chameleon_anim_map(source, scenes, "2E706.bin")?,
        decode_title_text_chameleon_anim_map(source, scenes, "2E728.bin")?,
        decode_title_text_chameleon_anim_map(source, scenes, "2E77A.bin")?,
    ];
    let title_menu_tilemap_path = scenes.join("mapeni/2EAAE.bin");
    let title_menu_tilemap_raw = source.read(&title_menu_tilemap_path).with_context(|| {
        format!(
            "failed to read title menu tilemap: {}",
            title_menu_tilemap_path.display()
        )
    })?;
    let title_menu_tilemap_words = decode_enigma_tilemap_loc_134D4(&title_menu_tilemap_raw, 0x451B)
        .context("failed to decode title menu tilemap")?;
    let mut character_frames = Vec::new();
    for region_byte in 0..=2u8 {
        for frame_index in 0..=7u8 {
            let plan = wildside_data::title_screen_character_load_plan_sub_1BF24(
                frame_index.into(),
                region_byte,
            )
            .context("failed to build title character load plan")?;
            let filename = asset_label_mapeni_filename(plan.tilemap_load.asset_label);
            let raw = source
                .read(&scenes.join("mapeni").join(&filename))
                .with_context(|| format!("failed to read title character tilemap: {filename}"))?;
            let tilemap_words =
                decode_enigma_tilemap_loc_134D4(&raw, plan.tilemap_load.vram_word_address)
                    .with_context(|| {
                        format!("failed to decode title character tilemap: {filename}")
                    })?;
            character_frames.push(TitleScreenCharacterFrameData {
                frame_index,
                region_byte,
                width_words: plan.tilemap_load.width_words,
                tilemap_words,
            });
        }
    }
    let lightning_art_path =
        scenes.join("artunc/Grey_mixed_pixels_during_title_transformation.bin");
    let lightning_art_bytes = source.read(&lightning_art_path).with_context(|| {
        format!(
            "failed to read title transformation lightning art: {}",
            lightning_art_path.display()
        )
    })?;

    // The symbolic plan supplies the shared tile-patch words. FC82-selected
    // transformation art and palette overlays are cached separately above
    // and below because the VDP builder chooses them for each presented frame.
    let plan = wildside_data::title_screen_load_plan_sub_1B2A4(0);
    let tile_patch_words = plan.tile_patch_words;

    // ASM loc_1C4BE (kid.asm:31202): after loc_1C4B2 seeds `d0=#$32`,
    // the inclusive DBF loop copies 0x33 words from Pal_1C400 into
    // Palette_Buffer; native loads those same title palette words for CRAM.
    // ASM loc_1C4E2 (kid.asm:31216): non-default regions copy their selected
    // 0x13-word overlay into Palette_Buffer+$1A; the default path leaves the
    // base palette unchanged.
    let palette_words = load_palette_words(source, &scenes.join("palette/title.bin"))?;
    let palette_variant_overlays = [
        load_palette_words(source, &scenes.join("palette/title_maniaxe.bin"))?,
        load_palette_words(source, &scenes.join("palette/title_juggernaut.bin"))?,
    ];

    Ok(TitleScreenData {
        art_chunks,
        transformation_art_variants,
        title_art_chunks,
        plane_a_tilemap,
        title_text_chameleon_rows: title_text_chameleon_upload.rows,
        title_text_chameleon_anim_maps,
        title_menu_tilemap_words,
        character_frames,
        lightning_art_bytes,
        tile_patch_words,
        palette_words,
        palette_variant_overlays,
    })
}

fn decode_title_text_chameleon_anim_map(
    source: &dyn AssetSource,
    scenes: &std::path::Path,
    filename: &str,
) -> Result<Vec<u16>> {
    let path = scenes.join("mapeni").join(filename);
    let raw = source.read(&path).with_context(|| {
        format!(
            "failed to read title chameleon animation tilemap: {}",
            path.display()
        )
    })?;
    decode_enigma_tilemap_loc_134D4(&raw, 0x433B)
        .with_context(|| format!("failed to decode title chameleon animation tilemap: {filename}"))
}

pub(crate) fn load_title_card_data(source: &dyn AssetSource) -> Result<TitleCardData> {
    let root = std::path::Path::new("theme/titlecard");
    let mut themes = vec![None; TITLE_CARD_THEME_NAMES.len()];
    let title_letter_palette_words = load_palette_words(
        source,
        std::path::Path::new("scenes/palette/Title_card_letters.bin"),
    )
    .context("failed to load title-card letter palette")?;

    for (theme_index, maybe_name) in TITLE_CARD_THEME_NAMES.iter().enumerate() {
        let Some(name) = maybe_name else {
            continue;
        };
        let art_path = root.join("artcomp").join(format!("{name}.bin"));
        let map_path = root.join("mapeni").join(format!("{name}.bin"));
        let palette_path = root.join("palette").join(format!("{name}.bin"));
        let art_bytes = load_saxman_art(source, &art_path)
            .with_context(|| format!("failed to load title-card art for {name}"))?;
        let map_raw = source
            .read(&map_path)
            .with_context(|| format!("failed to read title-card map: {}", map_path.display()))?;
        let tilemap_words = decode_enigma_tilemap_loc_134D4(&map_raw, 0x02FB)
            .with_context(|| format!("failed to decode title-card map for {name}"))?;
        let palette_words = load_palette_words(source, &palette_path)
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

    Ok(TitleCardData {
        themes,
        title_letter_palette_words,
    })
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

#[cfg(test)]
fn build_title_screen_vdp_with_character_frame_and_menu(
    data: &TitleScreenData,
    frame_index: u8,
    region_byte: u8,
    menu_cursor: Option<u8>,
) -> VdpState {
    build_title_screen_vdp_with_character_frame_menu_and_title_objects(
        data,
        frame_index,
        region_byte,
        menu_cursor,
        false,
        0,
    )
}

#[cfg(test)]
pub(crate) fn build_title_screen_vdp_with_character_frame_menu_and_title_objects(
    data: &TitleScreenData,
    frame_index: u8,
    region_byte: u8,
    menu_cursor: Option<u8>,
    title_ready: bool,
    title_elapsed_frames: usize,
) -> VdpState {
    build_title_screen_vdp_with_character_frame_menu_title_objects_and_lineage(
        data,
        frame_index,
        region_byte,
        menu_cursor,
        title_ready,
        title_elapsed_frames,
        TitleScreenVdpLineage::default(),
    )
}

#[derive(Debug, Clone, Copy, Default)]
pub(crate) struct TitleScreenVdpLineage {
    pub(crate) intro_return_hidden_shell: bool,
    pub(crate) direct_intro_sequence_5_title_return: bool,
    pub(crate) natural_intro_sequence_6_pre_ready_shell: bool,
    pub(crate) natural_intro_sequence_6_worker_elapsed: Option<usize>,
    pub(crate) fresh_title_worker_lineage: bool,
    /// `Obj_TitleMenu loc_1BB52` is streaming its first menu map this frame.
    pub(crate) title_menu_raster_upload: bool,
}

pub(crate) fn build_title_screen_vdp_with_character_frame_menu_title_objects_and_lineage(
    data: &TitleScreenData,
    frame_index: u8,
    region_byte: u8,
    menu_cursor: Option<u8>,
    title_ready: bool,
    title_elapsed_frames: usize,
    lineage: TitleScreenVdpLineage,
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
    for (chunk_index, (vram_byte_addr, art_bytes)) in data.art_chunks.iter().enumerate() {
        // ASM sub_1B2A4 loc_1B2D8 (kid.asm:29475-29484): the second upload
        // chooses Red Stealth, Maniaxe, or Juggernaut transformation art from
        // `$FFFFFC82`; the other three chunks are shared.
        let art_bytes = if chunk_index == 1 {
            &data.transformation_art_variants[usize::from(region_byte.min(2))]
        } else {
            art_bytes
        };
        vdp.write_vram(*vram_byte_addr, art_bytes);
    }

    // ASM loc_1C4A8 (kid.asm:31197): write 32 raw tile-data words to VRAM $F000.
    for (i, &word) in data.tile_patch_words.iter().enumerate() {
        vdp.write_vram_word(0xF000 + i * 2, word);
    }

    write_title_palette_sub_1b2a4_loc_1c4e2(&mut vdp, data, region_byte);

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
    // ASM loc_1B566 / loc_1B568 (kid.asm:29798-29803): each of the 28 decoded
    // row words is repeated for `d1=$3F` plus one writes before `a4` advances.
    // `title_text_chameleon_rows` stores that ROM row-repeat output pre-expanded.
    write_full_plane_rows(&mut vdp, 0xE000, &data.title_text_chameleon_rows);
    let mut sprite_entries = Vec::new();
    if title_ready {
        if let Some(elapsed) = lineage.natural_intro_sequence_6_worker_elapsed {
            let worker_elapsed =
                elapsed.saturating_sub(INTRO_SEQUENCE_6_NATURAL_TITLE_WORKER_START_FRAMES);
            // ASM loc_1B6AC -> loc_1BB10/loc_1BB36: the allocated child first
            // executes its slide calculation two object presentations after
            // the parent wakes, then advances eight pixels per live pass.
            let slide_elapsed = elapsed.saturating_sub(
                INTRO_SEQUENCE_6_NATURAL_TITLE_WORKER_START_FRAMES.saturating_add(2),
            );
            let counter =
                0x0027u16.saturating_sub(u16::try_from(slide_elapsed).unwrap_or(u16::MAX));
            let x_pos = crate::boot::title_text_intro_sprite_x_pos_loc_1BB36_and_loc_1BB44(counter);
            append_title_intro_text_sprite_loc_1bb10_with_x(&mut sprite_entries, x_pos as u16);

            // ASM loc_1B5CE/loc_1B610: y starts at `$FFA3`; after the same
            // `$1E` wait, `is_moved` applies y_vel=6 for the inclusive
            // 43-frame DBF drift, converging exactly on the normal y=$A5.
            let drift_frames = worker_elapsed.min(0x002A + 1);
            let y_pos = (-0x005D_i32 + i32::try_from(drift_frames).unwrap_or(i32::MAX) * 6) as u16;
            append_title_text_kid_sprite_loc_1b5e4_with_y(&mut sprite_entries, y_pos);
            // ASM loc_1B610 reaches sub_1B652 after the 30-frame wait, the
            // inclusive 43-frame drift and its 20-frame pause. Rebase this
            // natural worker clock onto the standard title helper's local-57
            // allocation origin so both popup children retain their ROM
            // animation cadence.
            let title_object_elapsed = elapsed
                .saturating_sub(
                    INTRO_SEQUENCE_6_NATURAL_TITLE_WORKER_START_FRAMES.saturating_sub(30),
                )
                .saturating_add(57);
            append_title_text_burst_sprites_sub_1b652(&mut sprite_entries, title_object_elapsed);
        } else if lineage.natural_intro_sequence_6_pre_ready_shell {
            // ASM loc_1B6AC allocates loc_1BB10 before the normal title shell
            // is trace-ready; oracle frame 2700 catches the sliding small
            // title-text sprite at hardware x=$0154, before the settled title
            // path. Make_Sprite adds the usual $80 hardware offset, so the
            // gfx object x is $00D4 here.
            append_title_intro_text_sprite_loc_1bb10_with_x(&mut sprite_entries, 0x00D4);
            // ASM loc_1B5CE seeds y=$FFA3,y_vel=6 before the settled
            // loc_1B5E4 pose; oracle frame 2700 catches the moving title Kid
            // at y=$006F while trace-facing title readiness is still false.
            append_title_text_kid_sprite_loc_1b5e4_with_y(&mut sprite_entries, 0x006F);
        } else {
            let title_return_first_object_frame = (lineage.direct_intro_sequence_5_title_return
                && title_elapsed_frames == TITLE_ATTRACT_LOCK_FRAMES)
                || (lineage.fresh_title_worker_lineage && title_elapsed_frames == 58);
            append_title_text_kid_sprite_loc_1b5e4(&mut sprite_entries, title_elapsed_frames);
            if !title_return_first_object_frame {
                // ASM Load_TitleScreen allocates Obj_TitleMenu before
                // Obj_TitleTextKid, but Obj_TitleMenu must run once more before
                // its loc_1BB10 child sprite appears. Both the direct
                // IntroSequence5 return and loc_119D8 manual demo return show
                // only the Kid title chunks on their first object pass.
                append_title_intro_text_sprite_loc_1bb10(&mut sprite_entries, title_elapsed_frames);
            }
            if lineage.fresh_title_worker_lineage {
                append_title_text_burst_sprites_sub_1b652(
                    &mut sprite_entries,
                    title_elapsed_frames,
                );
            }
        }
        write_title_text_chameleon_anim_loc_1b71a(&mut vdp, data, title_elapsed_frames, lineage);
        if let Some(elapsed) = lineage.natural_intro_sequence_6_worker_elapsed {
            // ASM loc_1B6DC allocates SegaText 101 passes after loc_1B6AC's
            // parent allocation. Rebase the natural mode-$28 clock onto that
            // allocation instead of treating trace-facing title readiness as
            // a newly allocated worker.
            let title_object_elapsed = elapsed.saturating_sub(
                INTRO_SEQUENCE_6_NATURAL_TITLE_WORKER_START_FRAMES.saturating_sub(30),
            );
            overlay_title_sega_text_worker_loc_1c7a0(
                &mut vdp,
                title_object_elapsed,
                false,
                false,
                false,
            );
        } else if !lineage.natural_intro_sequence_6_pre_ready_shell {
            overlay_title_sega_text_worker_loc_1c7a0(
                &mut vdp,
                title_elapsed_frames,
                lineage.intro_return_hidden_shell,
                lineage.direct_intro_sequence_5_title_return,
                lineage.fresh_title_worker_lineage,
            );
        }
    }
    let menu_raster_initial_vram = if menu_cursor.is_some() && lineage.title_menu_raster_upload {
        if !sprite_entries.is_empty() {
            let sprite_table_base = sprite_table_base_byte_addr(&vdp.registers);
            write_sprite_table_entries(&mut vdp, sprite_table_base, &sprite_entries);
        }
        Some(vdp.vram.clone())
    } else {
        None
    };
    if let Some(cursor) = menu_cursor {
        write_title_menu_tilemap_loc_1bb52(&mut vdp, &data.title_menu_tilemap_words);
        append_title_menu_cursor_sprite_loc_1bb52(&mut sprite_entries, cursor);
    }
    if !sprite_entries.is_empty() {
        let sprite_table_base = sprite_table_base_byte_addr(&vdp.registers);
        write_sprite_table_entries(&mut vdp, sprite_table_base, &sprite_entries);
    }
    if let Some(initial_vram) = menu_raster_initial_vram {
        // Direct oracle FIFO tracing places loc_1BB52 -> sub_1C5D0's first
        // duplicated menu word on line 81 pixel 220. The menu occupies rows
        // 32..71, so the completed post-frame VRAM is first visible on the
        // following frame rather than retroactively changing this one.
        let writes = title_menu_tilemap_word_writes_loc_1bb52(&data.title_menu_tilemap_words);
        vdp.raster_frame = Some(VdpRasterFrame {
            initial_vram,
            initial_cram: None,
            vram_word_writes: schedule_h40_vram_fifo_word_writes(&writes, 81, 220),
        });
    }

    vdp
}

pub(crate) fn build_intro_sequence_6_vdp_with_character_frame(
    data: &TitleScreenData,
    frame_index: Option<u8>,
    lightning_flash_frame: u8,
    raster_uploads: IntroSequence6RasterUploads,
    region_byte: u8,
    level_layout_x_pos: u32,
    camera_y_pos: u32,
) -> VdpState {
    let mut vdp = VdpState::new();
    let IntroSequence6RasterUploads {
        lightning: lightning_raster_upload,
        title: title_raster_upload,
    } = raster_uploads;

    for (i, &value) in VDP_GAME_INIT_VALUES.iter().enumerate() {
        vdp.registers[i] = value;
    }
    vdp.registers[1] = 0x74;

    // ASM Load_IntroSequence6 loc_1B170 (kid.asm:29368): seed the hidden
    // lightning art strip at VRAM `$1780` with eight `$11111111` longwords.
    for word_index in 0..16usize {
        vdp.write_vram_word(0x1780 + word_index * 2, 0x1111);
    }

    // ASM loc_1B182 clears the first 2048 VRAM words with `$20BC`.
    for word_index in 0..=0x7FFusize {
        vdp.write_vram_word(word_index * 2, 0x20BC);
    }

    // ASM loc_1B194 writes the `$BD/$BE/$BF/$C0` striped Scroll B bootstrap.
    let stripes = crate::boot::intro_sequence_6_striped_tile_fill_loc_1B194();
    for (word_index, word) in stripes.word_writes.iter().copied().enumerate() {
        vdp.write_vram_word(0xE000 + word_index * 2, word);
    }

    apply_intro_sequence_6_lightning_frame(&mut vdp, data, lightning_flash_frame);

    // ASM sub_1B222 -> loc_1B23A copies the current Level_Layout high word to
    // every Horiz_Scroll_Buffer Scroll A entry while Scroll B stays at zero.
    // GfxObjects_Collision_BottomBoundary4 -> loc_1B51A advances the initial
    // `$FFB8_0000` value by 4.5 pixels per pass until it reaches zero.
    let hscroll_base = usize::from(vdp.registers[0x0D]) << 10;
    for line in 0..224usize {
        vdp.write_vram_word(hscroll_base + line * 4, (level_layout_x_pos >> 16) as u16);
        vdp.write_vram_word(hscroll_base + line * 4 + 2, 0);
    }
    // ASM sub_1B222 copies `$FFFFA656` into Camera_Y_pos before the common VDP
    // scroll transfer mirrors it into the first VSRAM word. With the cold
    // `$FFD0_0000` hidden camera seed this shifts the transformation tilemap
    // down by 48px.
    vdp.write_vsram_word(0, (camera_y_pos >> 16) as u16);
    vdp.write_vsram_word(2, 0);

    // ASM sub_1B2A4 uploads the transformation/title base art and palette.
    for (chunk_index, (vram_byte_addr, art_bytes)) in data.art_chunks.iter().enumerate() {
        // ASM sub_1B2A4 loc_1B2D8 selects the same FC82 transformation-art
        // variant when called by Load_IntroSequence6.
        let art_bytes = if chunk_index == 1 {
            &data.transformation_art_variants[usize::from(region_byte.min(2))]
        } else {
            art_bytes
        };
        vdp.write_vram(*vram_byte_addr, art_bytes);
    }
    for (i, &word) in data.tile_patch_words.iter().enumerate() {
        vdp.write_vram_word(0xF000 + i * 2, word);
    }
    write_title_palette_sub_1b2a4_loc_1c4e2(&mut vdp, data, region_byte);

    // ASM Load_IntroSequence6 calls sub_1C5D0 with the sub_1B2A4 decoded map,
    // then sub_1B43A repeatedly calls sub_1BF24 for the hidden shell frames.
    if let Some(frame_index) = frame_index {
        write_title_character_frame_sub_1bf24(&mut vdp, data, frame_index, region_byte);
    } else {
        write_title_character_tilemap_rect(&mut vdp, &data.plane_a_tilemap, 18, 0x20BC);
    }

    if let Some(upload) = title_raster_upload {
        if let Some(current_frame_index) = upload.current_frame_index {
            write_title_character_frame_sub_1bf24(&mut vdp, data, current_frame_index, region_byte);
        } else {
            write_title_character_tilemap_rect(&mut vdp, &data.plane_a_tilemap, 18, 0x20BC);
        }
    }

    if lightning_raster_upload.is_some() || title_raster_upload.is_some() {
        let mut initial_vdp = vdp.clone();
        let mut vram_word_writes = Vec::new();
        if let Some(upload) = title_raster_upload {
            if let Some(previous_frame_index) = upload.previous_frame_index {
                write_title_character_frame_sub_1bf24(
                    &mut initial_vdp,
                    data,
                    previous_frame_index,
                    region_byte,
                );
            } else {
                write_title_character_tilemap_rect(
                    &mut initial_vdp,
                    &data.plane_a_tilemap,
                    18,
                    0x20BC,
                );
            }
            vram_word_writes.extend(schedule_h40_vram_fifo_word_writes(
                &intro_sequence_6_title_word_writes_sub_1bf24(&vdp),
                upload.start_scanline,
                upload.start_pixel,
            ));
        }
        if let Some(upload) = lightning_raster_upload {
            apply_intro_sequence_6_lightning_frame(
                &mut initial_vdp,
                data,
                upload.previous_flash_frame,
            );
            vram_word_writes.extend(schedule_h40_vram_fifo_word_writes(
                &intro_sequence_6_lightning_word_writes(data, lightning_flash_frame),
                upload.start_scanline,
                upload.start_pixel,
            ));
        }
        vram_word_writes.sort_unstable_by_key(|write| (write.scanline, write.pixel));
        vdp.raster_frame = Some(VdpRasterFrame {
            initial_vram: initial_vdp.vram,
            initial_cram: None,
            vram_word_writes,
        });
    }

    vdp
}

pub(crate) fn build_intro_sequence_6_vdp_after_loc_1b45a_title_art(
    data: &TitleScreenData,
    lightning_flash_frame: u8,
    region_byte: u8,
    level_layout_x_pos: u32,
    camera_y_pos: u32,
) -> VdpState {
    let mut vdp = build_intro_sequence_6_vdp_after_loc_1b494_palette_restore(
        data,
        lightning_flash_frame,
        region_byte,
        level_layout_x_pos,
        camera_y_pos,
    );

    // ASM loc_1B494 hibernates once after restoring Palette_Buffer, then
    // Load_TitleArt uploads the four title-text/menu chunks before the three
    // replacement workers are allocated.
    for (vram_byte_addr, art_bytes) in &data.title_art_chunks {
        vdp.write_vram(*vram_byte_addr, art_bytes);
    }

    vdp
}

pub(crate) fn build_intro_sequence_6_vdp_at_loc_1b470_palette_blackout(
    data: &TitleScreenData,
    lightning_flash_frame: u8,
    region_byte: u8,
    level_layout_x_pos: u32,
    camera_y_pos: u32,
    title_text_plane_rebuilt: bool,
) -> VdpState {
    let mut vdp = build_intro_sequence_6_vdp_with_character_frame(
        data,
        Some(crate::boot::title_screen_hidden_shell_palette_restore_loc_1B45A().blackout_frame),
        lightning_flash_frame,
        IntroSequence6RasterUploads::default(),
        region_byte,
        level_layout_x_pos,
        camera_y_pos,
    );
    if title_text_plane_rebuilt {
        rebuild_intro_sequence_6_title_text_plane_sub_1b532(&mut vdp, data, lightning_flash_frame);
    }

    // ASM loc_1B470 clears every Palette_Buffer word while copying it to
    // Palette_Buffer_2. Palette_to_VRAM presents that all-zero buffer across
    // both the initial and interstitial blackout frames.
    vdp.cram.fill(0);
    vdp
}

pub(crate) fn build_intro_sequence_6_vdp_after_loc_1b494_palette_restore(
    data: &TitleScreenData,
    lightning_flash_frame: u8,
    region_byte: u8,
    level_layout_x_pos: u32,
    camera_y_pos: u32,
) -> VdpState {
    let mut vdp = build_intro_sequence_6_vdp_with_character_frame(
        data,
        Some(crate::boot::title_screen_hidden_shell_palette_restore_loc_1B45A().blackout_frame),
        lightning_flash_frame,
        IntroSequence6RasterUploads::default(),
        region_byte,
        level_layout_x_pos,
        camera_y_pos,
    );
    rebuild_intro_sequence_6_title_text_plane_sub_1b532(&mut vdp, data, lightning_flash_frame);
    vdp
}

fn rebuild_intro_sequence_6_title_text_plane_sub_1b532(
    vdp: &mut VdpState,
    data: &TitleScreenData,
    lightning_flash_frame: u8,
) {
    // ASM sub_1B532 (kid.asm:29782): clear 8 longwords at $1780, then copy
    // 28 full plane rows from decoded `unk_2F840` to VRAM $E000. `loc_1B37C`
    // continues to upload lightning frames into $17A0 afterwards.
    for word_index in 0..usize::from(8u16 * 2) {
        vdp.write_vram_word(0x1780 + word_index * 2, 0);
    }
    write_full_plane_rows(vdp, 0xE000, &data.title_text_chameleon_rows);

    apply_intro_sequence_6_lightning_frame(vdp, data, lightning_flash_frame);
}

/// ASM `loc_1B37C -> sub_1B41C` (kid.asm): write one 128-byte lightning frame.
fn apply_intro_sequence_6_lightning_frame(
    vdp: &mut VdpState,
    data: &TitleScreenData,
    flash_frame: u8,
) {
    for (byte_addr, word) in intro_sequence_6_lightning_word_writes(data, flash_frame) {
        vdp.write_vram_word(usize::from(byte_addr), word);
    }
}

/// ASM `sub_1B41C/loc_1B432` (kid.asm): 32 longwords become 64 VRAM words.
fn intro_sequence_6_lightning_word_writes(
    data: &TitleScreenData,
    flash_frame: u8,
) -> Vec<(u16, u16)> {
    let upload = wildside_data::thunder_animation_frame_upload_sub_1B41C(
        flash_frame.into(),
        &data.lightning_art_bytes,
    )
    .expect("loaded ArtUnc_2A4D6 should contain the requested sub_1B41C flash frame");
    upload
        .longwords
        .into_iter()
        .enumerate()
        .flat_map(|(longword_index, longword)| {
            let byte_addr = upload.vram_word_address + (longword_index as u16) * 4;
            [
                (byte_addr, (longword >> 16) as u16),
                (byte_addr + 2, longword as u16),
            ]
        })
        .collect()
}

/// ASM `sub_1BF24 -> sub_1C5D0` (kid.asm): collect the 18x20 plane-A
/// nametable rectangle in the same row-major order used by the VDP port loop.
fn intro_sequence_6_title_word_writes_sub_1bf24(vdp: &VdpState) -> Vec<(u16, u16)> {
    const TILEMAP_WIDTH: usize = 18;
    const TILEMAP_HEIGHT: usize = 20;
    const CELL_X: usize = 0x15;
    const CELL_Y: usize = 1;

    (0..TILEMAP_HEIGHT)
        .flat_map(|row| {
            (0..TILEMAP_WIDTH).map(move |col| {
                let byte_addr = (CELL_Y + row) * NAMETABLE_ROW_STRIDE_BYTES + (CELL_X + col) * 2;
                (
                    byte_addr as u16,
                    u16::from_be_bytes([vdp.vram[byte_addr], vdp.vram[byte_addr + 1]]),
                )
            })
        })
        .collect()
}

/// ASM `loc_1B37C/sub_1B41C` (kid.asm): active-display FIFO start by lineage.
pub(crate) fn intro_sequence_6_lightning_raster_upload(
    presentation_elapsed: usize,
    previous_flash_frame: u8,
    lineage: IntroSequence6RasterLineage,
) -> Option<IntroSequence6LightningRasterUpload> {
    // Oracle FIFO traces show the same loc_1B3BC upload indices on all three
    // lineages, while the sibling object workloads move the 68k entry point.
    // These are routine-local elapsed positions, never global TAS frames.
    const COLD_BOOT_STARTS: [(u16, u16); 29] = [
        (143, 44),
        (133, 12),
        (144, 12),
        (133, 268),
        (140, 12),
        (133, 12),
        (159, 12),
        (133, 12),
        (158, 76),
        (165, 330),
        (158, 44),
        (165, 330),
        (158, 76),
        (165, 330),
        (179, 380),
        (165, 330),
        (182, 330),
        (168, 330),
        (182, 330),
        (193, 108),
        (182, 380),
        (192, 140),
        (181, 76),
        (192, 330),
        (180, 330),
        (192, 140),
        (182, 268),
        (190, 140),
        (179, 268),
    ];
    const WRAPPED_TIMEOUT_STARTS: [(u16, u16); 29] = [
        (142, 380),
        (132, 268),
        (143, 108),
        (135, 140),
        (140, 44),
        (135, 76),
        (150, 268),
        (135, 140),
        (151, 12),
        (140, 12),
        (149, 380),
        (138, 330),
        (150, 12),
        (138, 380),
        (140, 380),
        (138, 204),
        (140, 92),
        (139, 28),
        (140, 156),
        (143, 268),
        (140, 108),
        (147, 12),
        (141, 330),
        (145, 140),
        (143, 12),
        (146, 76),
        (141, 330),
        (143, 204),
        (141, 12),
    ];
    const LATER_TIMEOUT_STARTS: [(u16, u16); 29] = [
        (143, 92),
        (133, 12),
        (143, 380),
        (133, 268),
        (140, 12),
        (133, 12),
        (159, 76),
        (133, 12),
        (158, 44),
        (165, 330),
        (158, 76),
        (165, 330),
        (158, 44),
        (165, 330),
        (179, 380),
        (165, 330),
        (182, 284),
        (168, 268),
        (182, 284),
        (193, 140),
        (182, 380),
        (192, 28),
        (181, 28),
        (192, 330),
        (180, 380),
        (192, 268),
        (182, 204),
        (190, 140),
        (179, 268),
    ];

    let offset = presentation_elapsed.checked_sub(82)?;
    if !offset.is_multiple_of(3) {
        return None;
    }
    let starts = match lineage {
        IntroSequence6RasterLineage::ColdBoot => &COLD_BOOT_STARTS,
        IntroSequence6RasterLineage::WrappedTimeout => &WRAPPED_TIMEOUT_STARTS,
        IntroSequence6RasterLineage::LaterTimeout => &LATER_TIMEOUT_STARTS,
    };
    let &(start_scanline, start_pixel) = starts.get(offset / 3)?;
    Some(IntroSequence6LightningRasterUpload {
        previous_flash_frame,
        start_scanline,
        start_pixel,
    })
}

/// ASM `sub_1B43A -> loc_1B446 -> sub_1BF24` (kid.asm): active-display
/// plane-A nametable FIFO start by worker lineage.
pub(crate) fn intro_sequence_6_title_raster_upload(
    presentation_elapsed: usize,
    current_frame_index: Option<u8>,
    previous_frame_index: Option<u8>,
    lineage: IntroSequence6RasterLineage,
) -> Option<IntroSequence6TitleRasterUpload> {
    // Oracle FIFO traces at sub_1C5D0's first `$00AA` nametable word. Each
    // lineage runs the same three-frame byte script, but sibling object work
    // moves its 68k entry point independently from loc_1B37C's art upload.
    const COLD_BOOT_STARTS: [(u16, u16); 29] = [
        (102, 28),
        (92, 12),
        (102, 380),
        (92, 204),
        (98, 380),
        (92, 12),
        (118, 12),
        (92, 12),
        (117, 76),
        (124, 284),
        (117, 44),
        (124, 268),
        (117, 76),
        (124, 284),
        (138, 300),
        (124, 268),
        (141, 268),
        (127, 268),
        (141, 268),
        (152, 44),
        (141, 236),
        (151, 156),
        (140, 76),
        (151, 268),
        (139, 284),
        (151, 140),
        (141, 172),
        (149, 140),
        (138, 220),
    ];
    const WRAPPED_TIMEOUT_STARTS: [(u16, u16); 29] = [
        (101, 300),
        (91, 204),
        (102, 76),
        (94, 156),
        (98, 380),
        (94, 76),
        (109, 204),
        (94, 156),
        (109, 380),
        (98, 380),
        (108, 300),
        (97, 268),
        (109, 12),
        (97, 330),
        (99, 330),
        (97, 172),
        (99, 28),
        (97, 330),
        (99, 76),
        (102, 172),
        (99, 28),
        (105, 380),
        (100, 284),
        (104, 140),
        (101, 380),
        (105, 76),
        (100, 268),
        (102, 172),
        (99, 380),
    ];
    const LATER_TIMEOUT_STARTS: [(u16, u16); 29] = [
        (102, 28),
        (92, 12),
        (102, 330),
        (92, 204),
        (98, 380),
        (92, 12),
        (118, 76),
        (92, 12),
        (117, 44),
        (124, 284),
        (117, 76),
        (124, 284),
        (117, 44),
        (124, 268),
        (138, 330),
        (124, 284),
        (141, 236),
        (127, 204),
        (141, 236),
        (152, 76),
        (141, 268),
        (151, 28),
        (140, 28),
        (151, 268),
        (139, 330),
        (151, 204),
        (141, 140),
        (149, 140),
        (138, 220),
    ];

    let first_elapsed = match lineage {
        IntroSequence6RasterLineage::ColdBoot => 81,
        IntroSequence6RasterLineage::WrappedTimeout | IntroSequence6RasterLineage::LaterTimeout => {
            80
        }
    };
    let offset = presentation_elapsed.checked_sub(first_elapsed)?;
    if !offset.is_multiple_of(3) {
        return None;
    }
    let starts = match lineage {
        IntroSequence6RasterLineage::ColdBoot => &COLD_BOOT_STARTS,
        IntroSequence6RasterLineage::WrappedTimeout => &WRAPPED_TIMEOUT_STARTS,
        IntroSequence6RasterLineage::LaterTimeout => &LATER_TIMEOUT_STARTS,
    };
    let &(start_scanline, start_pixel) = starts.get(offset / 3)?;
    Some(IntroSequence6TitleRasterUpload {
        current_frame_index,
        previous_frame_index,
        start_scanline,
        start_pixel,
    })
}

#[allow(non_snake_case)]
fn write_title_menu_tilemap_loc_1bb52(vdp: &mut VdpState, tilemap_words: &[u16]) {
    // ASM Obj_TitleMenu loc_1BB52 (kid.asm:30428): decode `unk_2EAAE` with
    // base `$451B`, then call sub_1C5D0 with d4=$04,d5=$04,d6=$0D,d7=$05.
    // sub_1C5D0 pre-decrements d6/d7 before the DBF loops, so this writes a
    // 13x5 tile rectangle at cell (4,4), revealing the
    // interactive 1P/2P/options menu after the first Start press.
    let tilemap_words = title_menu_tilemap_buffer_loc_1bb52(tilemap_words);
    write_tilemap_rect_at_cell(vdp, &tilemap_words, 13, 5, 4, 4);
}

fn title_menu_tilemap_word_writes_loc_1bb52(tilemap_words: &[u16]) -> Vec<(u16, u16)> {
    const MENU_WIDTH: usize = 13;
    const MENU_HEIGHT: usize = 5;
    const MENU_CELL_X: usize = 4;
    const MENU_CELL_Y: usize = 4;
    let words = title_menu_tilemap_buffer_loc_1bb52(tilemap_words);
    words
        .into_iter()
        .enumerate()
        .map(|(index, word)| {
            let row = index / MENU_WIDTH;
            let col = index % MENU_WIDTH;
            let byte_addr = (MENU_CELL_Y + row) * 128 + (MENU_CELL_X + col) * 2;
            (
                u16::try_from(byte_addr).expect("title menu nametable address should fit u16"),
                word,
            )
        })
        .take(MENU_WIDTH * MENU_HEIGHT)
        .collect()
}

fn title_menu_tilemap_buffer_loc_1bb52(menu_words: &[u16]) -> Vec<u16> {
    // ASM loc_1BB52 (kid.asm:30428): EniDec writes the menu map at the start
    // of Decompression_Buffer, then sub_1C5D0 blindly consumes 13x5 words.
    // The source-port renderer is stateless and rebuilds the title VDP frame
    // after the ROM would have cached the menu rectangle, so pad the short
    // extracted decode with the menu's own blank interior word instead of
    // leaking current-frame background art into the persisted rectangle.
    const MENU_WORD_COUNT: usize = 13 * 5;
    const MENU_BLANK_WORD: u16 = 0x4528;
    let mut words = Vec::with_capacity(MENU_WORD_COUNT);
    for index in 0..MENU_WORD_COUNT {
        let word = menu_words.get(index).copied().unwrap_or(MENU_BLANK_WORD);
        words.push(word);
    }
    words
}

#[allow(non_snake_case)]
fn write_title_text_chameleon_anim_loc_1b71a(
    vdp: &mut VdpState,
    data: &TitleScreenData,
    elapsed: usize,
    lineage: TitleScreenVdpLineage,
) {
    let d3_elapsed = if let Some(elapsed) = lineage.natural_intro_sequence_6_worker_elapsed {
        // ASM loc_1B6AC seeds d3=0 after its `$1E` wait; loc_1B6CE then
        // hibernates once and sub_1B71A increments it on every live pass.
        elapsed.saturating_sub(INTRO_SEQUENCE_6_NATURAL_TITLE_WORKER_START_FRAMES)
    } else if lineage.fresh_title_worker_lineage {
        // ASM Load_TitleScreen allocates a fresh Obj_TitleTextChameleon with
        // its own coroutine registers. Its first visible sub_1B71A pass must
        // start from d3=0 instead of inheriting the outer title-frame clock.
        elapsed.saturating_sub(57)
    } else if lineage.direct_intro_sequence_5_title_return {
        // ASM loc_1BA60 -> sub_1C246 -> loc_1C264 enters Load_TitleScreen,
        // which allocates fresh Obj_TitleTextChameleon at kid.asm:29464-29466.
        // Oracle pixels on the direct `$24` title-return route show that this
        // worker is already seven sub_1B71A ticks ahead when the rebuilt title
        // VDP is first presented.
        elapsed.saturating_add(7)
    } else if lineage.intro_return_hidden_shell {
        // ASM loc_1B6AC/loc_1B6DC is allocated by loc_1B45A before the
        // Start-skip title VDP becomes visible. The hidden Chameleon worker's
        // preserved `d3` is five sub_1B71A ticks ahead through the visible
        // rebuild, then reaches one further tick after the Load_IntroSequence6
        // title-attract pre-roll (`loc_1BAB0`) has elapsed.
        elapsed.saturating_add(if elapsed >= INTRO_SEQUENCE_6_TITLE_ATTRACT_LOCK_FRAMES {
            6
        } else {
            5
        })
    } else {
        elapsed
    };
    let d3 = u16::try_from(d3_elapsed).unwrap_or(u16::MAX);
    let map_index = title_text_chameleon_anim_map_index(d3_elapsed);
    let d2 = if let Some(elapsed) = lineage.natural_intro_sequence_6_worker_elapsed {
        // ASM loc_1B6AC seeds d2=$28. The first loc_1B6CE call observes $28,
        // then each presented pass decrements it until the 40th reaches zero.
        let worker_elapsed =
            elapsed.saturating_sub(INTRO_SEQUENCE_6_NATURAL_TITLE_WORKER_START_FRAMES);
        0x0029_i16.saturating_sub(i16::try_from(worker_elapsed).unwrap_or(i16::MAX))
    } else if lineage.natural_intro_sequence_6_pre_ready_shell {
        // ASM loc_1B6AC -> loc_1B6CE (kid.asm:29917-29931): the natural
        // IntroSequence6 title handoff reaches the visible title shell while
        // the `$28` countdown reveal is still in progress. Oracle frame 2700
        // corresponds to incoming d2=7; after that the reveal converges to
        // the steady d2=0 path used by the normal title screen.
        let countdown = INTRO_SEQUENCE_6_NATURAL_TITLE_VDP_READY_FRAMES
            .saturating_add(7)
            .saturating_sub(elapsed);
        i16::try_from(countdown).unwrap_or(i16::MAX)
    } else {
        0
    };
    let Some(upload) = crate::boot::title_text_chameleon_frame_upload_sub_1B71A(
        &data.title_text_chameleon_anim_maps[map_index],
        d2,
        d3,
    ) else {
        return;
    };
    for row in upload.rows {
        let mut byte_addr = usize::from(row.vram_byte_address);
        if let Some(filler_word) = row.filler_word {
            vdp.write_vram_word(byte_addr.saturating_sub(2), filler_word);
        }
        for word in row.words {
            vdp.write_vram_word(byte_addr, word);
            byte_addr += 2;
        }
    }
}

fn title_text_chameleon_anim_map_index(elapsed: usize) -> usize {
    let d3 = u16::try_from(elapsed).unwrap_or(u16::MAX);
    match d3 & 0x000C {
        0x0000 => 0,
        0x0008 => 2,
        _ => 1,
    }
}

#[allow(non_snake_case)]
fn append_title_text_kid_sprite_loc_1b5e4(entries: &mut Vec<HardwareSpriteEntry>, _elapsed: usize) {
    // ASM Obj_TitleTextKid / loc_1B5CE / loc_1B5E4 (kid.asm:29834-29875):
    // the main title KID text sprite uses `LnkTo_unk_E10FE`, tile `$4D3`,
    // palette line 2, and keeps x=`$2E` after its drift settles at y=`$A5`.
    // ORIGINAL_QUIRK: the later `x=$2E,y=$8C` and `x=$5A,y=$98` writes occur
    // after `sub_1B652`, which allocates a child popup gfx object and leaves
    // `a3` pointing at that child, so those writes do not move the main KID
    // title sprite.
    append_title_text_kid_sprite_loc_1b5e4_with_y(entries, 0x00A5);
}

#[allow(non_snake_case)]
fn append_title_text_kid_sprite_loc_1b5e4_with_y(
    entries: &mut Vec<HardwareSpriteEntry>,
    y_pos: u16,
) {
    let setup = crate::boot::title_text_kid_setup_loc_1B5E4();
    append_intro_sprite_entries_make_sprite_from_gfx_object_loc_129E(
        entries,
        IntroGfxObjectForSpriteTable {
            x_pos: setup.x_pos,
            y_pos,
            palette_line: setup.palette_line,
            vram_tile: setup.vram_tile,
            priority: 0,
            x_direction: 0,
        },
        IntroSpriteFrameVram {
            sprite_index: 0,
            x_center: 0,
            y_center: 0,
            width: 0x48,
            height: 0x3D,
        },
    );
}

#[allow(non_snake_case)]
fn append_title_text_burst_sprites_sub_1b652(
    entries: &mut Vec<HardwareSpriteEntry>,
    elapsed: usize,
) {
    let burst = crate::boot::title_text_burst_sprite_sub_1B652();
    let setup = crate::boot::title_text_kid_setup_loc_1B5E4();
    let allocation_elapseds =
        crate::boot::title_text_kid_burst_allocation_elapseds_loc_1B5E4_sub_1B652();
    let animation_cycle_frames = burst
        .animation_frames
        .iter()
        .map(|frame| usize::from(frame.duration_frames))
        .sum::<usize>();

    for (allocation_elapsed, (x_pos, y_pos)) in
        allocation_elapseds.into_iter().zip(setup.popup_positions)
    {
        // ASM sub_1B652 calls Init_Animation after the current GfxObject pass,
        // so the new child first reaches Make_SpriteFromGfxObject one frame
        // after allocation. The command-2 tail rewinds the full animation.
        let Some(displayed_elapsed) = elapsed.checked_sub(allocation_elapsed.saturating_add(1))
        else {
            continue;
        };
        let mut cycle_elapsed = displayed_elapsed % animation_cycle_frames;
        let mut sprite_index = None;
        for (frame, frame_sprite_index) in burst.animation_frames.iter().zip([
            Some(0x000C),
            Some(0x0012),
            None,
            Some(0x0000),
            Some(0x0006),
        ]) {
            let duration = usize::from(frame.duration_frames);
            if cycle_elapsed < duration {
                sprite_index = frame_sprite_index;
                break;
            }
            cycle_elapsed -= duration;
        }
        let Some(sprite_index) = sprite_index else {
            continue;
        };

        // ORIGINAL_QUIRK: sub_1B652 returns with a3 pointing at the allocated
        // child. loc_1B610's following pose writes therefore move the first
        // burst to ($2E,$8C) and the second to ($5A,$98), not the main KID
        // title sprite.
        append_intro_sprite_entries_make_sprite_from_gfx_object_loc_129E(
            entries,
            IntroGfxObjectForSpriteTable {
                x_pos,
                y_pos,
                palette_line: burst.palette_line,
                vram_tile: burst.vram_tile,
                priority: u8::from(burst.priority_flag),
                x_direction: 0,
            },
            IntroSpriteFrameVram {
                sprite_index,
                x_center: 0,
                y_center: 0,
                width: 0x10,
                height: 0x18,
            },
        );
    }
}

#[allow(non_snake_case)]
fn append_title_intro_text_sprite_loc_1bb10(
    entries: &mut Vec<HardwareSpriteEntry>,
    elapsed: usize,
) {
    // ASM loc_1BB10/loc_1BB44 (kid.asm:30405-30425): the small title text
    // sprite slides in from x=-44 to x=$10C, then persists through the menu.
    let elapsed = u16::try_from(elapsed).unwrap_or(u16::MAX);
    let counter = 0x0027u16.saturating_sub(elapsed);
    let x_pos = crate::boot::title_text_intro_sprite_x_pos_loc_1BB36_and_loc_1BB44(counter);
    append_title_intro_text_sprite_loc_1bb10_with_x(entries, x_pos as u16);
}

#[allow(non_snake_case)]
fn append_title_intro_text_sprite_loc_1bb10_with_x(
    entries: &mut Vec<HardwareSpriteEntry>,
    x_pos: u16,
) {
    let setup = crate::boot::title_text_intro_sprite_loc_1BB10();
    append_intro_sprite_entries_make_sprite_from_gfx_object_loc_129E(
        entries,
        IntroGfxObjectForSpriteTable {
            x_pos,
            y_pos: setup.y_pos,
            palette_line: setup.palette_line,
            vram_tile: setup.vram_tile,
            priority: 0,
            x_direction: 0,
        },
        IntroSpriteFrameVram {
            // ASM unk_E0ED6: sprite_frame_vram $2C3, $00, $08, $10, $08.
            sprite_index: 0x02C3,
            x_center: 0,
            y_center: 0x0008,
            width: 0x0010,
            height: 0x0008,
        },
    );
}

#[allow(non_snake_case)]
fn append_title_menu_cursor_sprite_loc_1bb52(entries: &mut Vec<HardwareSpriteEntry>, cursor: u8) {
    // ASM Obj_TitleMenu loc_1BB52/loc_1BC0E (kid.asm:30438-30467) loads a
    // visible gfx slot on palette line 2, VRAM tile `$54B`, frame
    // `unk_E1196` (8x8), at x=$28 and y from word_1BC20.
    let y_pos = match cursor.min(2) {
        0 => 0x0030,
        1 => 0x003A,
        _ => 0x0044,
    };
    append_intro_sprite_entries_make_sprite_from_gfx_object_loc_129E(
        entries,
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

fn write_title_palette_sub_1b2a4_loc_1c4e2(
    vdp: &mut VdpState,
    data: &TitleScreenData,
    region_byte: u8,
) {
    // ASM sub_1B2A4 -> loc_1C4B2/loc_1C4BE (kid.asm:31197-31205):
    // copy the inclusive `$32` DBF range from Pal_1C400 first.
    vdp.write_cram_words(0, &data.palette_words);

    // ASM loc_1C4E0/loc_1C4E2 (kid.asm:31207-31218): non-zero FC82
    // variants overwrite 19 words at Palette_Buffer+$1A with either the
    // Maniaxe or Juggernaut palette before the next Palette_to_VRAM pass.
    let overlay = match region_byte.min(2) {
        0 => return,
        1 => &data.palette_variant_overlays[0],
        _ => &data.palette_variant_overlays[1],
    };
    vdp.write_cram_words(0x0D, overlay);
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

    let clear_vram = crate::level::load_title_card_clear_vram_loc_19ABE();
    for i in 0..usize::from(clear_vram.word_write_count) {
        vdp.write_vram_word(
            usize::from(clear_vram.vram_word_addr) + i * 2,
            clear_vram.fill_word,
        );
    }

    let tilemap_clear = crate::level::load_title_card_tilemap_clear_loc_19B58();
    for i in 0..usize::from(tilemap_clear.word_write_count) {
        vdp.write_vram_word(
            usize::from(tilemap_clear.vram_word_addr) + i * 2,
            tilemap_clear.fill_word,
        );
    }

    let name_table_clear = crate::level::load_title_card_name_table_clear_loc_19B6A();
    for i in 0..usize::from(name_table_clear.word_write_count) {
        vdp.write_vram_word(
            usize::from(name_table_clear.vram_word_addr) + i * 2,
            name_table_clear.fill_word,
        );
    }

    let palette_word_count =
        usize::from(crate::level::load_title_card_palette_copy_word_count_loc_19B80());
    let palette_words = theme
        .palette_words
        .get(..palette_word_count)
        .unwrap_or(&theme.palette_words);
    vdp.write_cram_words(0, palette_words);
    let title_letter_palette_word_count =
        usize::from(crate::level::level_entry_title_letter_palette_copy_word_count_loc_1AB40());
    let title_letter_palette_words = data
        .title_letter_palette_words
        .get(..title_letter_palette_word_count)
        .unwrap_or(&data.title_letter_palette_words);
    // ASM loc_1AB40 (kid.asm:28758): title-card letter colours are copied to
    // Palette_Buffer+$20, i.e. CRAM word offset $10.
    vdp.write_cram_words(0x10, title_letter_palette_words);

    let width = usize::from(theme.width_tiles);
    let height = usize::from(theme.height_tiles);
    let left_words = usize::from((0x0028u16.saturating_sub(theme.width_tiles)) >> 1);
    let mut dst_word_index = 0usize;

    if let Some(top_padding) =
        crate::level::load_title_card_top_padding_loc_19BC2(theme.height_tiles)
    {
        for _ in 0..usize::from(top_padding.row_count) {
            for _ in 0..usize::from(top_padding.words_per_row) {
                vdp.write_vram_word(dst_word_index * 2, top_padding.fill_word);
                dst_word_index += 1;
            }
        }
    }

    for row in 0..height {
        if let Some(left_padding) = crate::level::load_title_card_left_padding_loc_19BD4(
            u16::try_from(left_words).unwrap_or(u16::MAX),
        ) {
            for _ in 0..usize::from(left_padding.word_write_count) {
                vdp.write_vram_word(dst_word_index * 2, left_padding.fill_word);
                dst_word_index += 1;
            }
        }

        if let Some(row_copy_seed) =
            crate::level::load_title_card_row_copy_seed_loc_19BDA(theme.width_tiles)
        {
            for col in 0..usize::from(crate::level::load_title_card_row_copy_word_count_loc_19BDE(
                row_copy_seed,
            )) {
                let src_index = row * width + col;
                let word = theme
                    .tilemap_words
                    .get(src_index)
                    .copied()
                    .unwrap_or(0x02FB);
                vdp.write_vram_word(dst_word_index * 2, word);
                dst_word_index += 1;
            }
        }

        if let Some(right_padding) = crate::level::load_title_card_right_padding_loc_19BEE(
            u16::try_from(left_words).unwrap_or(u16::MAX),
            theme.width_tiles,
        ) {
            for _ in 0..usize::from(right_padding.word_write_count) {
                vdp.write_vram_word(dst_word_index * 2, right_padding.fill_word);
                dst_word_index += 1;
            }
        }
    }

    if let Some(bottom_padding) =
        crate::level::load_title_card_bottom_padding_loc_19C04(theme.height_tiles)
    {
        for _ in 0..usize::from(bottom_padding.row_count) {
            for _ in 0..usize::from(bottom_padding.words_per_row) {
                vdp.write_vram_word(dst_word_index * 2, bottom_padding.fill_word);
                dst_word_index += 1;
            }
        }
    }

    Some(vdp)
}

// ─── Helpers ────────────────────────────────────────────────────────────────

/// Load raw palette file (bundle-relative) as big-endian u16 words.
fn load_palette_words(source: &dyn AssetSource, rel: &std::path::Path) -> Result<Vec<u16>> {
    let raw = source
        .read(rel)
        .with_context(|| format!("failed to read palette: {}", rel.display()))?;
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
fn load_saxman_art(source: &dyn AssetSource, rel: &std::path::Path) -> Result<Vec<u8>> {
    let compressed = source
        .read(rel)
        .with_context(|| format!("failed to read intro art: {}", rel.display()))?;
    wildside_data::decode_saxman(&compressed)
        .with_context(|| format!("failed to decompress intro art: {}", rel.display()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn absolute_m68k_clock_resolves_stop_music_entry_beam_e14c8() {
        let cycles = M68kCycleCount::from_raw(111_231_277);

        assert_eq!(
            h40_beam_from_absolute_m68k_cycles(cycles),
            H40BeamPosition::new(250, 1_219).expect("bounded oracle beam"),
            "oracle StopMusic $E14C8 reports scanline 250, H40 pixel 152"
        );
        assert_eq!(
            ntsc_vblank_cursor_from_absolute_m68k_cycles(cycles).get(),
            90_139,
            "the persistent GEMS frame cursor starts at VDP scanline 224"
        );
    }

    #[test]
    fn title_wait_for_vint_reaches_oracle_mode_standard_boundary() {
        let entry_cycles = M68kCycleCount::from_raw(10_349_895);
        let advance = title_wait_for_vint_to_mode_standard_cursor_with_scheduler(
            LoadInGameCpuRasterCursor {
                // ASM WaitForVint $93E on oracle frame 80, immediately
                // before the final TST/BEQ pair crosses VINT assertion.
                beam: h40_beam_from_absolute_m68k_cycles(entry_cycles),
                refresh_phase: M68kRefreshPhase::new(60).unwrap(),
            },
            |_, _, _| Some(M68kCycleCount::from_raw(0)),
        )
        .expect("ASM WaitForVint/V_Int/Mode_Standard path");

        assert_eq!(
            advance.resume,
            crate::WaitForVintResumePath::StaleBranchLoc942
        );
        assert_eq!(advance.scheduler_tick_count, 15);
        assert_eq!(advance.raw_cycles.get(), 238);
        assert_eq!(advance.refresh_events, 2);
        assert_eq!(advance.post_assertion_refresh_events, 2);
        assert_eq!(advance.external_wait_cycles.get(), 0);
        assert_eq!(
            advance.cursor,
            LoadInGameCpuRasterCursor {
                beam: h40_beam_from_absolute_m68k_cycles(M68kCycleCount::from_raw(10_350_137)),
                refresh_phase: M68kRefreshPhase::new(46).unwrap(),
            },
            "oracle frame 80 reaches Mode_Standard $77A at cycle 10,350,137",
        );
    }

    #[test]
    fn title_palette_dma_reaches_oracle_caller_boundary() {
        let entry_cycles = M68kCycleCount::from_raw(10_356_475);
        let mut bus_requested = false;
        let advance = title_mode_standard_palette_to_vram_cursor_with_scheduler(
            LoadInGameCpuRasterCursor {
                // ASM Mode_Standard $77E immediately before BSR.W
                // Palette_to_VRAM on oracle frame 81.
                beam: h40_beam_from_absolute_m68k_cycles(entry_cycles),
                refresh_phase: M68kRefreshPhase::new(22).unwrap(),
            },
            |event| {
                match event {
                    TitlePaletteHardwareEvent::Advance {
                        vdp_owns_bus,
                        source_instruction,
                        ..
                    } => {
                        if vdp_owns_bus {
                            assert!(bus_requested, "CRAM DMA runs while Stop_z80 owns BUSREQ");
                            assert!(!source_instruction, "DMA halt ticks are not instructions");
                        }
                    }
                    TitlePaletteHardwareEvent::SetZ80BusRequest(requested) => {
                        bus_requested = requested;
                    }
                }
                Some(M68kCycleCount::from_raw(0))
            },
        )
        .expect("ASM Palette_to_VRAM CRAM DMA path");

        assert!(!bus_requested, "Start_z80 releases BUSREQ before returning");
        assert_eq!(advance.timing.raw_cycles.get(), 362);
        assert_eq!(advance.refresh_events, 2);
        assert_eq!(advance.external_wait_cycles.get(), 0);
        assert_eq!(advance.dma_halt_cycles.get(), 163);
        assert_eq!(
            advance.cursor,
            LoadInGameCpuRasterCursor {
                beam: h40_beam_from_absolute_m68k_cycles(M68kCycleCount::from_raw(10_357_004)),
                refresh_phase: M68kRefreshPhase::new(92).unwrap(),
            },
            "oracle frame 81 returns to Mode_Standard $782 at cycle 10,357,004",
        );
    }

    #[test]
    fn load_in_game_tail_positions_stop_music_from_sub_28fc_cursor() {
        // ASM Load_InGame $11A12..$11A24 (kid.asm): the active timer path
        // reaches StopMusic $E14C8 after the complete sub_28FC/sub_44B0/
        // Init_Timer_and_Bonus_Flags source tail.
        let cursor = load_in_game_stop_music_cursor_from_sub_28fc(
            LoadInGameCpuRasterCursor {
                beam: H40BeamPosition::new(243, 2_563).expect("oracle absolute-cycle beam"),
                refresh_phase: M68kRefreshPhase::new(114).expect("oracle refresh phase"),
            },
            crate::LoadInGameTimerBonusPath::Initialise,
        );

        assert_eq!(cursor.beam, H40BeamPosition::new(250, 1_219).unwrap());
        assert_eq!(cursor.refresh_phase, M68kRefreshPhase::new(14).unwrap());
    }

    #[test]
    fn platform_art_loc_14314_fifo_stalls_only_during_active_display() {
        let active = load_in_game_platform_art_transfer_cursor_loc_14314(
            LoadInGameCpuRasterCursor {
                // ASM Initialize_Platforms sub_231C -> DecompressToVRAM
                // loc_14314 first horizontal-art word on the published route.
                beam: H40BeamPosition::new(183, 1_060).expect("oracle absolute-cycle beam"),
                refresh_phase: M68kRefreshPhase::new(96).expect("oracle refresh phase"),
            },
            192,
        )
        .expect("non-empty inclusive DBF transfer");
        assert_eq!(active.fifo_halted_ticks, M68kCycleCount::from_raw(5_999));
        assert_eq!(active.refresh_events, 1);
        assert_eq!(
            active.cursor.beam,
            H40BeamPosition::new(204, 1_053).expect("oracle absolute-cycle beam")
        );
        assert_eq!(
            active.cursor.refresh_phase,
            M68kRefreshPhase::new(44).unwrap()
        );

        let vblank = load_in_game_platform_art_transfer_cursor_loc_14314(
            LoadInGameCpuRasterCursor {
                // ASM Initialize_Platforms' vertical-art loc_14314 begins
                // after VBlank, where blank-line FIFO bandwidth is ample.
                beam: H40BeamPosition::new(234, 2_949).expect("oracle absolute-cycle beam"),
                refresh_phase: M68kRefreshPhase::new(124).expect("oracle refresh phase"),
            },
            192,
        )
        .expect("non-empty inclusive DBF transfer");
        assert_eq!(vblank.fifo_halted_ticks, M68kCycleCount::from_raw(0));
        assert_eq!(vblank.refresh_events, 34);
        assert_eq!(
            vblank.cursor.beam,
            H40BeamPosition::new(243, 2_451).expect("oracle absolute-cycle beam")
        );
        assert_eq!(
            vblank.cursor.refresh_phase,
            M68kRefreshPhase::new(98).unwrap()
        );
    }

    #[test]
    fn sega_logo_loc_14314_uses_display_disabled_fifo_bandwidth() {
        let advance = decompress_to_vram_transfer_display_disabled_cursor_loc_14314_with_scheduler(
            LoadInGameCpuRasterCursor {
                // ASM Load_SegaScreen $12DDA writes VDP register 1 as
                // $8134 before DecompressToVRAM loc_14314. Selected-PC
                // oracle tracing pins the first transfer at cycle
                // 4,161,724 and return at cycle 4,178,922.
                beam: h40_beam_from_absolute_m68k_cycles(M68kCycleCount::from_raw(4_161_724)),
                refresh_phase: M68kRefreshPhase::new(10).unwrap(),
            },
            768,
            0,
            DecompressToVramTransferExit::Return,
            |_, _| Some(M68kCycleCount::from_raw(0)),
        )
        .expect("display-disabled Sega logo transfer schedule");

        assert_eq!(advance.fifo_halted_ticks, M68kCycleCount::from_raw(0));
        assert_eq!(
            advance.cursor.beam,
            h40_beam_from_absolute_m68k_cycles(M68kCycleCount::from_raw(4_178_922)),
        );
        assert_eq!(
            advance.cursor.refresh_phase,
            M68kRefreshPhase::new(56).unwrap()
        );
    }

    #[test]
    fn title_decompress_loc_14314_services_vint_inside_fifo_transfer() {
        let entry_cycles = M68kCycleCount::from_raw(9_161_190);
        let advance = decompress_to_vram_transfer_cursor_loc_14314_with_hardware(
            LoadInGameCpuRasterCursor {
                // ASM sub_1B2A4 -> DecompressToVRAM loc_14314 on the
                // published title route, selected-PC oracle frame 71.
                beam: h40_beam_from_absolute_m68k_cycles(entry_cycles),
                refresh_phase: M68kRefreshPhase::new(101).unwrap(),
            },
            2_016,
            1,
            DecompressToVramTransferExit::Return,
            |_, _| Some(()),
        )
        .expect("title transfer schedule");

        assert_eq!(advance.refresh_events, 143);
        assert_eq!(advance.fifo_halted_ticks.get(), 38_709);
        assert_eq!(
            advance.cursor.beam,
            h40_beam_from_absolute_m68k_cycles(M68kCycleCount::from_raw(9_244_685)),
            "ASM loc_14314 should reach return_14334 after the in-loop VINT and FIFO stalls when no external bus waits occur",
        );
        assert_eq!(
            advance.cursor.refresh_phase,
            M68kRefreshPhase::new(44).unwrap()
        );
        assert!(advance.vint_serviced);
    }

    #[test]
    fn title_decompress_to_ram_loc_1440a_clocks_eight_bit_shift() {
        // ASM loc_1440A (kid.asm): `lsl.w #8,d6` costs 6 + 2n = 22 cycles
        // on the 68000. This is materially longer than a one-bit shift.
        let mut source_cycles = Vec::new();
        decompress_to_ram_transfer_cursor_loc_1440a_with_scheduler(
            LoadInGameCpuRasterCursor {
                beam: H40BeamPosition::new(0, 0).unwrap(),
                refresh_phase: M68kRefreshPhase::new(0).unwrap(),
            },
            1,
            0,
            DecompressToVramTransferExit::Return,
            |cycles, vdp_owns_bus| {
                assert!(!vdp_owns_bus);
                source_cycles.push(cycles.get());
                Some(M68kCycleCount::from_raw(0))
            },
        )
        .expect("one loc_1440A word");

        assert_eq!(&source_cycles[..7], &[8, 14, 22, 8, 14, 8, 14]);
    }

    #[test]
    fn initialize_platforms_positions_sub_28fc_from_load_in_game_entry() {
        let advance = load_in_game_platform_initialisation_cursor_from_entry(
            LoadInGameCpuRasterCursor {
                // ASM Load_InGame $11A0E selected-PC boundary.
                beam: H40BeamPosition::new(151, 432).expect("oracle absolute-cycle beam"),
                refresh_phase: M68kRefreshPhase::new(116).expect("oracle refresh phase"),
            },
            Some(LoadInGamePlatformVintEvent {
                // ASM Decompress_Chunk sub_13684: the scanline-224 VINT
                // interrupts the vertical stream after 9,424 source cycles.
                vertical_decoder_raw_offset: M68kCycleCount::from_raw(9_424),
            }),
            LoadInGamePlatformRefreshLedger {
                prelude: 7,
                horizontal_decoder: 115,
                between_uploads: 0,
                vertical_decoder_before_vint: 75,
                vint_exception: 0,
                vint_handler: 1,
                vertical_decoder_after_vint: 40,
                epilogue: 0,
            },
        )
        .expect("bounded platform initialisation schedule");

        assert_eq!(
            advance.horizontal_fifo_halted_ticks,
            M68kCycleCount::from_raw(5_999)
        );
        assert_eq!(
            advance.vertical_fifo_halted_ticks,
            M68kCycleCount::from_raw(0)
        );
        assert_eq!(advance.refresh_events, 273);
        assert_eq!(advance.elapsed, M68kCycleCount::from_raw(45_253));
        assert_eq!(
            advance.cursor.beam,
            H40BeamPosition::new(243, 2_563).expect("oracle absolute-cycle beam")
        );
        assert_eq!(
            advance.cursor.refresh_phase,
            M68kRefreshPhase::new(114).unwrap()
        );
    }

    #[test]
    fn initialize_platforms_emits_exact_hardware_timeline_loc_14314() {
        // ASM Load_InGame $11A0E -> Initialize_Platforms sub_231C ->
        // DecompressToVRAM loc_14314. Full selected-PC oracle tracing gives
        // 38,604 source cycles, 273 two-cycle refreshes, a 104-cycle VINT,
        // and 5,999 one-cycle horizontal FIFO halts.
        let mut intervals = Vec::new();
        let advance = load_in_game_platform_initialisation_cursor_from_entry_with_hardware(
            LoadInGameCpuRasterCursor {
                beam: H40BeamPosition::new(151, 432).expect("oracle absolute-cycle beam"),
                refresh_phase: M68kRefreshPhase::new(116).expect("oracle refresh phase"),
            },
            Some(LoadInGamePlatformVintEvent {
                vertical_decoder_raw_offset: M68kCycleCount::from_raw(9_424),
            }),
            LoadInGamePlatformRefreshLedger {
                prelude: 7,
                horizontal_decoder: 115,
                between_uploads: 0,
                vertical_decoder_before_vint: 75,
                vint_exception: 0,
                vint_handler: 1,
                vertical_decoder_after_vint: 40,
                epilogue: 0,
            },
            |cycles, vdp_owns_bus| {
                intervals.push((cycles, vdp_owns_bus));
                Some(())
            },
        )
        .expect("published-route platform initialisation timeline");

        assert_eq!(
            intervals
                .iter()
                .map(|(cycles, _)| cycles.get())
                .sum::<u32>(),
            advance.elapsed.get(),
        );
        assert_eq!(advance.elapsed, M68kCycleCount::from_raw(45_253));
        let vdp_owned: Vec<_> = intervals
            .iter()
            .filter(|(_, vdp_owns_bus)| *vdp_owns_bus)
            .collect();
        assert_eq!(vdp_owned.len(), 5_999);
        assert!(
            vdp_owned
                .iter()
                .all(|(cycles, _)| *cycles == M68kCycleCount::from_raw(1)),
            "jgenesis retries the pending loc_14314 write in one-cycle VDP-owned scheduler ticks",
        );
    }

    #[test]
    fn no_z80_wait_for_vint_poll_reproduces_frontier_samples() {
        for (
            first_poll_scanline,
            first_poll_mclk,
            refresh_phase,
            expected_sample_mclk,
            expected_resume,
            expected_interrupt_refresh,
            expected_handler_refresh,
        ) in [
            (
                107,
                442,
                78,
                0,
                crate::WaitForVintResumePath::PollTestLoc93e,
                1,
                1,
            ),
            (
                105,
                1_008,
                104,
                82,
                crate::WaitForVintResumePath::StaleBranchLoc942,
                0,
                1,
            ),
        ] {
            let sample = wait_for_vint_sample_without_z80_bus_access(
                WaitForVintPollSeedNoZ80 {
                    first_poll_beam: H40BeamPosition::new(first_poll_scanline, first_poll_mclk)
                        .expect("bounded first WaitForVint $93E beam"),
                    refresh_phase: M68kRefreshPhase::new(refresh_phase)
                        .expect("bounded jgenesis refresh phase"),
                },
                ModeLevelRasterSeedVintAssertion {
                    beam: H40BeamPosition::new(224, 16).expect("H40 VINT assertion"),
                },
            )
            .expect("no-Z80 WaitForVint path reaches VINT in one frame");
            assert_eq!(sample.mclk_until_interrupt_sample, expected_sample_mclk);
            assert_eq!(sample.resume, expected_resume);
            assert_eq!(
                sample.interrupt_entry_refresh_wait_events,
                expected_interrupt_refresh
            );
            assert_eq!(
                sample.arbitration_after_handler_entry.refresh_wait_events,
                expected_handler_refresh
            );
            assert_eq!(sample.arbitration_after_handler_entry.z80_68k_bus_steals, 0);
        }
    }

    #[test]
    fn gems_banked_reads_interleave_with_wait_for_vint_poll() {
        // jgenesis helper frame 1696: GEMS `$01FB..$028A` copies ROM
        // `$EF23A..$EF2B9` into Z80 RAM `$1F00..$1F7F`. These are the 128
        // banked-read scheduler offsets from the first WaitForVint `$93E`
        // instruction, after removing `$93A sf V_Int_Done`'s 16 cycles.
        const ACCESS_OFFSETS_FROM_FIRST_LOC_93E: [u32; 128] = [
            10_816, 10_861, 10_916, 10_973, 11_028, 11_071, 11_908, 11_963, 12_010, 12_065, 12_122,
            12_165, 13_104, 13_149, 13_204, 13_249, 13_304, 13_359, 14_242, 14_287, 14_342, 14_397,
            14_452, 14_497, 15_380, 15_435, 15_480, 15_535, 15_592, 15_637, 16_520, 16_575, 16_630,
            16_685, 16_730, 16_785, 17_668, 17_713, 17_770, 17_825, 17_868, 17_925, 18_852, 18_909,
            18_954, 19_009, 19_064, 19_119, 19_990, 20_045, 20_102, 20_147, 20_204, 20_259, 21_142,
            21_185, 21_240, 21_295, 21_342, 21_397, 22_280, 22_335, 22_378, 22_433, 22_490, 22_535,
            23_462, 23_519, 23_574, 23_629, 23_674, 23_729, 24_612, 24_657, 24_714, 24_769, 24_812,
            24_867, 25_750, 25_805, 25_852, 25_907, 25_964, 26_007, 26_890, 26_945, 27_000, 27_045,
            27_102, 27_157, 28_040, 28_083, 28_140, 28_185, 28_240, 28_297, 29_180, 29_223, 29_280,
            29_335, 29_380, 29_435, 30_362, 30_417, 30_474, 30_517, 30_574, 30_629, 31_502, 31_557,
            31_612, 31_667, 31_712, 31_767, 32_650, 32_695, 32_752, 32_807, 32_852, 32_907, 33_790,
            33_845, 33_892, 33_947, 34_002, 34_047, 34_974, 35_031,
        ];
        let offsets = ACCESS_OFFSETS_FROM_FIRST_LOC_93E.map(M68kCycleCount::from_raw);

        let sample = wait_for_vint_sample_with_z80_bus_accesses(
            WaitForVintPollSeed {
                first_poll_beam: H40BeamPosition::new(99, 1_190)
                    .expect("bounded first WaitForVint beam"),
                refresh_phase: M68kRefreshPhase::new(56).expect("bounded refresh phase"),
                z80_68k_bus_access_offsets: &offsets,
            },
            ModeLevelRasterSeedVintAssertion {
                beam: H40BeamPosition::new(224, 16).expect("H40 VINT assertion"),
            },
        )
        .expect("typed GEMS banked-copy schedule reaches VINT");

        assert_eq!(
            sample,
            crate::WaitForVintInterruptSample {
                mclk_until_interrupt_sample: 58,
                interrupt_entry_refresh_wait_events: 0,
                resume: crate::WaitForVintResumePath::StaleBranchLoc942,
                arbitration_after_handler_entry: crate::WaitForVintArbitration {
                    refresh_wait_events: 2,
                    z80_68k_bus_steals: 0,
                },
            },
            "ASM WaitForVint `$93E/$942` plus GEMS `$0273/$0285` banked reads"
        );
    }

    #[test]
    fn wait_for_vint_z80_schedule_rejects_non_monotonic_offsets() {
        let offsets = [M68kCycleCount::from_raw(100), M68kCycleCount::from_raw(100)];
        assert_eq!(
            wait_for_vint_sample_with_z80_bus_accesses(
                WaitForVintPollSeed {
                    first_poll_beam: H40BeamPosition::new(107, 442)
                        .expect("bounded first WaitForVint beam"),
                    refresh_phase: M68kRefreshPhase::new(78).expect("bounded refresh phase"),
                    z80_68k_bus_access_offsets: &offsets,
                },
                ModeLevelRasterSeedVintAssertion {
                    beam: H40BeamPosition::new(224, 16).expect("H40 VINT assertion"),
                },
            ),
            None,
            "ordered Z80 scheduler events must be strictly monotonic"
        );
    }

    #[test]
    fn vint_sample_state_derives_frontier_mode_entry_beams() {
        for (
            sample_mclk,
            interrupt_refresh_wait_events,
            resume,
            handler_refresh_wait_events,
            expected_scanline_mclk,
        ) in [
            (
                58,
                0,
                crate::WaitForVintResumePath::StaleBranchLoc942,
                2,
                1_684,
            ),
            (0, 1, crate::WaitForVintResumePath::PollTestLoc93e, 1, 1_556),
            (
                82,
                0,
                crate::WaitForVintResumePath::StaleBranchLoc942,
                1,
                1_694,
            ),
        ] {
            let mode = mode_level_raster_seed_mode_entry_from_vint_assertion(
                ModeLevelRasterSeedVintAssertion {
                    // jgenesis H40 VINT assertion: scanline 224, h_mclk 16.
                    beam: H40BeamPosition::new(224, 16).expect("H40 VINT beam"),
                },
                crate::WaitForVintInterruptSample {
                    mclk_until_interrupt_sample: sample_mclk,
                    interrupt_entry_refresh_wait_events: interrupt_refresh_wait_events,
                    resume,
                    arbitration_after_handler_entry: crate::WaitForVintArbitration {
                        refresh_wait_events: handler_refresh_wait_events,
                        z80_68k_bus_steals: 0,
                    },
                },
            )
            .expect("ASM WaitForVint/V_Int/MainGameLoop path");
            assert_eq!(mode.beam.scanline(), 224);
            assert_eq!(mode.beam.scanline_mclk(), expected_scanline_mclk);
        }
    }

    #[test]
    fn life_icon_first_commit_follows_h40_fifo_slots_loc_4586() {
        // ASM sub_44DC/loc_4586 and jgenesis H40 traces. The master-clock
        // positions preserve the sub-pixel phase hidden by the corresponding
        // entry pixels 161, 95, and 303.
        let cases = [
            (42, 1_295, 64, (42, 380)),
            (32, 761, 116, (32, 330)),
            (14, 2_429, 110, (15, 92)),
        ];
        for (entry_scanline, entry_mclk, refresh_phase, expected) in cases {
            let commit = life_icon_first_commit_loc_4586(
                H40BeamPosition::new(entry_scanline, entry_mclk).expect("bounded H40 beam"),
                M68kRefreshPhase::new(refresh_phase).expect("bounded refresh phase"),
            );
            assert_eq!(
                (commit.scanline, commit.pixel),
                expected,
                "236 raw cycles, two refresh waits, and the next H40 VRAM slot"
            );
        }
    }

    #[test]
    fn life_icon_fifo_commit_train_matches_all_three_helper_boundaries_loc_4586() {
        for (entry_scanline, entry_mclk, refresh_phase, expected_first, expected_last) in [
            (42, 1_294, 64, (42, 380), (49, 380)),
            (32, 765, 116, (32, 330), (39, 330)),
            (14, 2_431, 110, (15, 92), (22, 92)),
        ] {
            let commits = life_icon_fifo_commit_positions_loc_4586(ModeLevelCpuRasterCursor {
                beam: H40BeamPosition::new(entry_scanline, entry_mclk)
                    .expect("bounded sub_44DC beam"),
                refresh_phase: M68kRefreshPhase::new(refresh_phase).expect("bounded refresh phase"),
            })
            .expect("life upload first commit must be an H40 access slot");
            assert_eq!((commits[0].scanline, commits[0].pixel), expected_first,);
            assert_eq!((commits[63].scanline, commits[63].pixel), expected_last,);
        }
    }

    #[test]
    fn life_icon_fifo_exit_matches_all_three_helper_boundaries_loc_44d4() {
        struct Case {
            entry: (u16, u16, u8),
            z80_offsets: &'static [u32],
            exit: (u16, u16, u8),
        }

        let command = [crate::HudDmaEventSub44DC::LifeIcon {
            frame_table_byte_offset: 4,
        }];
        let cases = [
            Case {
                entry: (42, 1_294, 64),
                z80_offsets: &[],
                exit: (49, 3_072, 58),
            },
            Case {
                entry: (32, 765, 116),
                z80_offsets: &[],
                exit: (39, 2_690, 58),
            },
            Case {
                entry: (14, 2_431, 110),
                z80_offsets: &[43, 96, 151, 192],
                exit: (22, 838, 102),
            },
        ];

        for case in cases {
            let z80_offsets = case
                .z80_offsets
                .iter()
                .copied()
                .map(M68kCycleCount::from_raw)
                .collect::<Vec<_>>();
            let exit = life_icon_cpu_raster_cursor_after_sub_44dc(
                ModeLevelCpuRasterCursor {
                    beam: H40BeamPosition::new(case.entry.0, case.entry.1)
                        .expect("bounded sub_44DC entry"),
                    refresh_phase: M68kRefreshPhase::new(case.entry.2)
                        .expect("bounded entry phase"),
                },
                &command,
                &z80_offsets,
            )
            .expect("classified life-icon FIFO exit");
            assert_eq!(exit.beam.scanline(), case.exit.0);
            assert_eq!(exit.beam.scanline_mclk(), case.exit.1);
            assert_eq!(exit.refresh_phase.get(), case.exit.2);
        }

        let entry = ModeLevelCpuRasterCursor {
            beam: H40BeamPosition::new(42, 1_294).expect("bounded sub_44DC entry"),
            refresh_phase: M68kRefreshPhase::new(64).expect("bounded entry phase"),
        };
        assert_eq!(
            life_icon_cpu_raster_cursor_after_sub_44dc(entry, &[], &[]),
            None,
            "unclassified HUD command lists remain fail-closed"
        );
        assert_eq!(
            life_icon_cpu_raster_cursor_after_sub_44dc(
                entry,
                &command,
                &[M68kCycleCount::from_raw(96), M68kCycleCount::from_raw(43),],
            ),
            None,
            "GEMS offsets must be source ordered"
        );
    }

    #[test]
    fn life_icon_chronological_scheduler_merges_generated_gems_waits_loc_44d4() {
        let entry = ModeLevelCpuRasterCursor {
            beam: H40BeamPosition::new(14, 2_431).expect("bounded sub_44DC entry"),
            refresh_phase: M68kRefreshPhase::new(110).expect("oracle refresh phase"),
        };
        let command = [crate::HudDmaEventSub44DC::LifeIcon {
            frame_table_byte_offset: 4,
        }];
        // The selected-PC offsets are relative to the final pending enqueue,
        // 3,450 cycles after this exact `sub_44DC` entry.
        let event_offsets = [3_493_u32, 3_546, 3_601, 3_642];
        let mut elapsed = 0_u32;
        let mut next_event = 0_usize;

        let exit = life_icon_cpu_raster_cursor_after_sub_44dc_with_scheduler(
            entry,
            &command,
            |cycles: M68kCycleCount, vdp_owns_bus| {
                elapsed = elapsed.checked_add(cycles.get())?;
                let mut wait_cycles = 0_u32;
                while event_offsets
                    .get(next_event)
                    .is_some_and(|&offset| offset <= elapsed)
                {
                    if event_offsets[next_event] != elapsed || vdp_owns_bus {
                        return None;
                    }
                    wait_cycles = wait_cycles.checked_add(11)?;
                    next_event += 1;
                }
                Some(M68kCycleCount::from_raw(wait_cycles))
            },
        )
        .expect("typed chronological scheduler");

        assert_eq!(next_event, event_offsets.len());
        assert_eq!(
            exit,
            ModeLevelCpuRasterCursor {
                beam: H40BeamPosition::new(22, 838).expect("oracle loc_80C beam"),
                refresh_phase: M68kRefreshPhase::new(102).expect("oracle exit phase"),
            },
            "ASM sub_44DC FIFO ownership and GEMS waits must share one chronological scheduler"
        );
    }

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
            intro_sequence_2_sky_castle_art: vec![],
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

    fn cram_word(vdp: &VdpState, word_offset: usize) -> u16 {
        let byte_addr = word_offset * 2;
        u16::from_be_bytes([vdp.cram[byte_addr], vdp.cram[byte_addr + 1]])
    }

    fn expected_palette_fade(
        fade_progress: u16,
        palette_buffer_2: &[u16],
        palette_buffer_3: &[u16],
    ) -> [u16; 0x40] {
        let mut expected = [0u16; 0x40];
        let mut pending_palette_0_mask = u32::MAX;
        let mut pending_palette_1_mask = u32::MAX;
        crate::level::apply_pending_palette_fade_sub_14C0(
            fade_progress,
            &mut pending_palette_0_mask,
            &mut pending_palette_1_mask,
            palette_buffer_2,
            palette_buffer_3,
            &mut expected,
        );
        expected
    }

    fn minimal_sega_data() -> SegaScreenData {
        let mut palette_words = (0..40).map(|word| 0x0100 | word as u16).collect::<Vec<_>>();
        palette_words[1] = 0x0EEE;
        palette_words[2] = 0x08EE;
        SegaScreenData {
            art_bytes: vec![],
            palette_words,
        }
    }

    fn intro_sequence_5_frame_tilemaps_fixture() -> Vec<Vec<u16>> {
        (0..8u16)
            .map(|frame_index| {
                let plan = wildside_data::intro_sequence_5_frame_load_plan_sub_1C034(frame_index)
                    .expect("ASM sub_1C034 should accept all eight frame indices");
                let word = 0x1100 | frame_index;
                vec![
                    word;
                    usize::from(plan.tilemap_load.width_words)
                        * usize::from(plan.tilemap_load.height_words)
                ]
            })
            .collect::<Vec<_>>()
    }

    #[test]
    fn intro_sequence_1_cold_boot_keeps_sega_vdp_until_fade_window_loc_12ede_1c204() {
        let intro = empty_intro_data();
        let sega = minimal_sega_data();

        let initial = build_intro_sequence_1_cold_boot_vdp_at_elapsed(&intro, &sega, 1, 0);
        assert_eq!(
            initial.vram,
            build_sega_screen_vdp(&sega, SEGA_SCREEN_INTRO_HANDOFF_ELAPSED).vram,
            "ASM loc_12EDE hands Load_IntroSequence1 the already-visible Sega VDP state at the first cold-boot intro frame",
        );

        let vdp = build_intro_sequence_1_cold_boot_vdp_at_elapsed(&intro, &sega, 1, 2);

        assert_eq!(
            cram_word(&vdp, 1),
            0x0EEE,
            "ASM Load_SegaScreen loc_12EDE hands to Load_IntroSequence1 before sub_1C204 has faded the Sega palette off-screen",
        );
    }

    #[test]
    fn intro_sequence_1_cold_boot_starts_sega_fade_on_elapsed_3_loc_12ede_1c204() {
        let intro = empty_intro_data();
        let sega = minimal_sega_data();

        let vdp = build_intro_sequence_1_cold_boot_vdp_at_elapsed(&intro, &sega, 1, 3);
        let expected = expected_palette_fade(
            0x00F0,
            &vdp_cram_words(&build_sega_screen_vdp(&sega, 176)),
            &[0; 0x40],
        );

        assert_eq!(
            vdp_cram_words(&vdp),
            expected,
            "ASM Load_SegaScreen loc_12EDE enters sub_1C204 on cold-boot elapsed 3, so the Sega CRAM must have taken the first fade-out step",
        );

        let initial_cram = vdp
            .raster_frame
            .as_ref()
            .and_then(|raster| raster.initial_cram.as_ref())
            .expect("post-visible CRAM DMA should retain the palette used for visible scanout");
        assert_eq!(
            u16::from_be_bytes([initial_cram[2], initial_cram[3]]),
            0x0EEE,
            "ASM sub_1C204's palette DMA completes below the visible field, so elapsed 3 rasterises the pre-fade Sega palette",
        );

        let next_vdp = build_intro_sequence_1_cold_boot_vdp_at_elapsed(&intro, &sega, 1, 4);
        let next_initial_cram = next_vdp
            .raster_frame
            .as_ref()
            .and_then(|raster| raster.initial_cram.as_ref())
            .expect("each sub_1C204 fade DMA should retain the prior visible palette");
        assert_eq!(
            next_initial_cram, &vdp.cram,
            "ASM sub_1C204 at the next VBlank presents the preceding post-DMA palette before completing its own below-field transfer",
        );
    }

    #[test]
    fn intro_sequence_1_cold_boot_uses_forward_sega_elapsed_during_fade_loc_12ede_1c204() {
        let intro = empty_intro_data();
        let sega = minimal_sega_data();
        let elapsed = INTRO_SEQUENCE_1_SEGA_FADE_OUT_ELAPSED - 1;

        let vdp = build_intro_sequence_1_cold_boot_vdp_at_elapsed(&intro, &sega, 1, elapsed);
        let mut expected =
            build_sega_screen_vdp(&sega, SEGA_SCREEN_INTRO_HANDOFF_ELAPSED + elapsed);
        apply_intro_sequence_1_sega_fade_out_loc_1c204(&mut expected, elapsed);

        assert_eq!(
            vdp_cram_words(&vdp),
            vdp_cram_words(&expected),
            "ASM loc_12EDE advances the existing Load_SegaScreen VDP state forward while sub_1C204 fades it out; cold-boot elapsed must be added to the handoff frame",
        );
    }

    #[test]
    fn intro_sequence_1_cold_boot_keeps_resident_sega_frame_before_black_loc_1c204() {
        let intro = empty_intro_data();
        let sega = minimal_sega_data();

        let resident_sega = build_intro_sequence_1_cold_boot_vdp_at_elapsed(
            &intro,
            &sega,
            1,
            INTRO_SEQUENCE_1_SEGA_FADE_OUT_ELAPSED,
        );
        assert!(
            resident_sega.cram.iter().any(|byte| *byte != 0),
            "ASM sub_1C204 returns after its final hibernate, so the resident Sega VDP is still presented for one frame",
        );

        let vdp = build_intro_sequence_1_cold_boot_vdp_at_elapsed(
            &intro,
            &sega,
            1,
            INTRO_SEQUENCE_1_SEGA_FADE_OUT_ELAPSED + 1,
        );

        assert!(
            vdp.cram.iter().all(|byte| *byte == 0),
            "ASM sub_1B8F4/sub_1C204 starts the black VDP window after the final resident Sega frame and before the Pal_1C28A fade-in",
        );
    }

    #[test]
    fn intro_sequence_1_cold_boot_starts_wild_side_fade_on_elapsed_47_loc_1c204() {
        let mut intro = empty_intro_data();
        intro.palette_segment_0 = vec![0x0EEE, 0x0444];
        let sega = minimal_sega_data();

        let black_tail = build_intro_sequence_1_cold_boot_vdp_at_elapsed(&intro, &sega, 1, 46);
        assert!(
            black_tail.cram.iter().all(|byte| *byte == 0),
            "ASM sub_1B8F4/sub_1C204 keeps CRAM black through the frame before Pal_1C28A begins fading in",
        );

        let first_fade = build_intro_sequence_1_cold_boot_vdp_at_elapsed(&intro, &sega, 1, 47);
        let mut target = [0u16; 0x40];
        target[0] = 0x0EEE;
        target[1] = 0x0444;
        let expected = expected_palette_fade(0x0010, &target, &[0; 0x40]);
        assert_eq!(
            vdp_cram_words(&first_fade),
            expected,
            "ASM sub_1C204's first Pal_1C28A fade step is presented at cold-boot elapsed 47",
        );
    }

    #[test]
    fn intro_sequence_1_cold_boot_keeps_fading_until_elapsed_61_loc_1c204() {
        let mut intro = empty_intro_data();
        intro.palette_segment_0 = vec![0x0EEE, 0x0444];
        let sega = minimal_sega_data();

        let fading = build_intro_sequence_1_cold_boot_vdp_at_elapsed(&intro, &sega, 1, 61);
        let normal = build_intro_sequence_vdp_with_title_variant_at_elapsed(&intro, 0, 1, 61, 0);

        assert_ne!(
            vdp_cram_words(&fading),
            vdp_cram_words(&normal),
            "ASM sub_1C204 still owns the Pal_1C28A fade on elapsed 61; the normal palette is restored on the next frame",
        );
    }

    #[test]
    fn intro_sequence_1_cold_boot_rejoins_normal_intro_vdp_after_fade_loc_1c204() {
        let mut intro = empty_intro_data();
        intro.palette_segment_0 = vec![0x0000, 0x0EEE, 0x0444];
        let sega = minimal_sega_data();

        let cold_boot = build_intro_sequence_1_cold_boot_vdp_at_elapsed(&intro, &sega, 1, 62);
        let normal = build_intro_sequence_vdp_with_title_variant_at_elapsed(&intro, 0, 1, 62, 0);

        assert_eq!(
            cold_boot.cram, normal.cram,
            "ASM sub_1B8F4 resumes loc_1B924 after sub_1C204, so the cold-boot-only palette transition must rejoin the normal IntroSequence1 VDP path",
        );
    }

    #[test]
    fn intro_sequence_1_demo_restart_keeps_existing_sub_1c204_presentation_schedule() {
        let mut intro = empty_intro_data();
        intro.palette_segment_0 = vec![0x0EEE, 0x0444];
        let sega = minimal_sega_data();

        let first_fade = build_intro_sequence_1_demo_restart_vdp_at_elapsed(&intro, &sega, 1, 46);
        let mut target = [0u16; 0x40];
        target[0] = 0x0EEE;
        target[1] = 0x0444;
        assert_eq!(
            vdp_cram_words(&first_fade),
            expected_palette_fade(0x0010, &target, &[0; 0x40]),
            "ASM sub_1B8F4/sub_1C204 on the post-demo restart lineage presents its first Pal_1C28A fade step at elapsed 46",
        );

        let restarted = build_intro_sequence_1_demo_restart_vdp_at_elapsed(&intro, &sega, 1, 61);
        let normal = build_intro_sequence_vdp_with_title_variant_at_elapsed(&intro, 0, 1, 61, 0);
        assert_eq!(restarted.cram, normal.cram);
    }

    #[test]
    fn intro_sequence_2_handoff_boundary_frames_follow_loc_1b9d0_1c204() {
        let mut data = empty_intro_data();
        data.palette_segment_0 = vec![0x0EEE, 0x0444];
        data.palette_segment_1 = vec![0x0A00, 0x00A0];

        let elapsed_0 = build_intro_sequence_vdp_with_title_variant_at_elapsed(&data, 1, 1, 0, 0);
        assert_eq!(
            vdp_cram_words(&elapsed_0)[..2],
            [0x0EEE, 0x0444],
            "ASM loc_1B9D0 displays the completed IntroSequence1 CRAM unchanged before the first fade-out step",
        );

        let elapsed_1 = build_intro_sequence_vdp_with_title_variant_at_elapsed(&data, 1, 1, 1, 0);
        let mut previous = [0u16; 0x40];
        previous[0] = 0x0EEE;
        previous[1] = 0x0444;
        let expected_fade_out = expected_palette_fade(0x0110, &previous, &[0; 0x40]);
        assert_eq!(
            vdp_cram_words(&elapsed_1),
            expected_fade_out,
            "ASM sub_1C204 starts fading the previous intro scene after the one-frame loc_1B9D0 handoff hold",
        );

        let black_tail = build_intro_sequence_vdp_with_title_variant_at_elapsed(
            &data,
            1,
            1,
            INTRO_SEQUENCE_2_FADE_IN_START_ELAPSED - 1,
            0,
        );
        assert!(
            black_tail.cram.iter().all(|byte| *byte == 0),
            "ASM loc_1B9D0/sub_1C204 keeps CRAM black until the frame before Pal_1C2F0 fades in",
        );

        let first_fade_in = build_intro_sequence_vdp_with_title_variant_at_elapsed(
            &data,
            1,
            1,
            INTRO_SEQUENCE_2_FADE_IN_START_ELAPSED,
            0,
        );
        let mut target = [0u16; 0x40];
        target[0] = 0x0A00;
        target[1] = 0x00A0;
        let expected_fade_in = expected_palette_fade(0x0010, &target, &[0; 0x40]);
        assert_eq!(
            vdp_cram_words(&first_fade_in),
            expected_fade_in,
            "ASM sub_1C204 starts fading Pal_1C2F0 into CRAM at intro-2 elapsed 44",
        );

        let second_fade_in = build_intro_sequence_vdp_with_title_variant_at_elapsed(
            &data,
            1,
            1,
            INTRO_SEQUENCE_2_FADE_IN_START_ELAPSED + 1,
            0,
        );
        let expected_second_fade_in = expected_palette_fade(0x0020, &target, &[0; 0x40]);
        assert_eq!(
            vdp_cram_words(&second_fade_in),
            expected_second_fade_in,
            "ASM sub_1C204 continues the Pal_1C2F0 fade on the frame after the first visible step",
        );
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

    fn sprite_table_index_without_link(
        vdp: &VdpState,
        expected: (u16, u8, u16, u16),
    ) -> Option<usize> {
        (0..80).find(|&entry_index| {
            let (y, size, _link, pattern, x) = sprite_entry_words(vdp, entry_index);
            (y, size, pattern, x) == expected
        })
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
    fn decode_enigma_tilemap_loc_134d4_returns_decoded_tile_words() {
        let enigma_fixture = [
            0x05, 0x00, 0x00, 0x01, 0x00, 0x0A, 0x09, 0x18, 0x10, 0xFE, 0x00,
        ];

        let decoded = decode_enigma_tilemap_loc_134D4(&enigma_fixture, 0x0040)
            .expect("ASM EniDec fixture should decode");

        assert_eq!(
            decoded,
            vec![0x0041, 0x0042, 0x0043, 0x004A, 0x004A, 0x0050]
        );
    }

    #[test]
    fn write_vsram_word_stores_big_endian_and_clips_at_end() {
        let mut vdp = VdpState::new();

        vdp.write_vsram_word(0x10, 0x1234);
        assert_eq!(&vdp.vsram[0x10..0x12], &[0x12, 0x34]);

        let before = vdp.vsram.clone();
        vdp.write_vsram_word(VSRAM_SIZE - 1, 0xBEEF);
        assert_eq!(
            vdp.vsram, before,
            "single trailing VSRAM byte is not enough for ASM-style word writes"
        );
    }

    #[test]
    fn vdp_cram_words_reads_big_endian_words_from_all_palette_slots() {
        let mut vdp = VdpState::new();
        vdp.write_cram_words(0, &[0x0123, 0x0456]);
        vdp.write_cram_words(0x3F, &[0x0EEE]);

        let words = vdp_cram_words(&vdp);

        assert_eq!(words[0], 0x0123);
        assert_eq!(words[1], 0x0456);
        assert_eq!(words[0x3F], 0x0EEE);
    }

    #[test]
    fn intro_sequence_palette_fade_helpers_use_sub_1c204_progression() {
        let mut vdp = VdpState::new();
        let mut target = [0u16; 0x40];
        target[0] = 0x0EEE;
        target[1] = 0x0444;

        apply_intro_sequence_palette_fade_in_loc_1c204(&mut vdp, &target, 46, 46);
        let expected_in = expected_palette_fade(0x0010, &target, &[0; 0x40]);
        assert_eq!(vdp_cram_words(&vdp), expected_in);

        let mut second_step = VdpState::new();
        apply_intro_sequence_palette_fade_in_loc_1c204(&mut second_step, &target, 47, 46);
        let expected_second_step = expected_palette_fade(0x0020, &target, &[0; 0x40]);
        assert_eq!(vdp_cram_words(&second_step), expected_second_step);

        let mut fade_out = VdpState::new();
        fade_out.write_cram_words(0, &target);
        apply_intro_sequence_fade_out_to_black_loc_1c204(&mut fade_out, 3, 17);
        let expected_out = expected_palette_fade(0x00F0, &target, &[0; 0x40]);
        assert_eq!(vdp_cram_words(&fade_out), expected_out);
    }

    #[test]
    fn intro_sequence_5_title_return_fade_to_black_honours_hold_and_fade_windows() {
        let mut hold = VdpState::new();
        hold.write_cram_words(0, &[0x0EEE, 0x0444]);
        assert!(apply_intro_title_return_fade_to_black_loc_28c6(
            &mut hold, 3
        ));
        assert_eq!(
            vdp_cram_words(&hold)[..2],
            [0x0EEE, 0x0444],
            "ASM loc_1BA60 holds the previous CRAM for the first four title-return frames"
        );

        let mut fading = VdpState::new();
        let mut source = [0u16; 0x40];
        source[0] = 0x0EEE;
        source[1] = 0x0444;
        fading.write_cram_words(0, &source);
        assert!(apply_intro_title_return_fade_to_black_loc_28c6(
            &mut fading,
            4
        ));
        let expected = expected_palette_fade(0x00F0, &source, &[0; 0x40]);
        assert_eq!(vdp_cram_words(&fading), expected);

        let mut second_step = VdpState::new();
        second_step.write_cram_words(0, &source);
        assert!(apply_intro_title_return_fade_to_black_loc_28c6(
            &mut second_step,
            5
        ));
        let expected_second_step = expected_palette_fade(0x00E0, &source, &[0; 0x40]);
        assert_eq!(vdp_cram_words(&second_step), expected_second_step);

        assert!(!apply_intro_title_return_fade_to_black_loc_28c6(
            &mut fading,
            18
        ));
    }

    #[test]
    #[allow(non_snake_case)]
    fn intro_sequence_5_fade_hold_uses_loc_1ba90_worker_elapsed_and_palette_fade() {
        let mut source = [0u16; 0x40];
        source[0] = 0x0EEE;
        source[1] = 0x0444;
        let data = IntroSub1B850Data {
            palette_segment_2: source.to_vec(),
            intro_sequence_5_frame_tilemaps: intro_sequence_5_frame_tilemaps_fixture(),
            ..empty_intro_data()
        };

        let first_hold = build_intro_sequence_5_fade_hold_vdp_at_elapsed(&data, 0, 0, 0);
        assert_eq!(
            vram_word(&first_hold, 0x019A),
            0x1105,
            "ASM loc_1BA90 displays the IntroSequence5 worker at INTRO_FIFTH_SEGMENT_FRAMES before the first fade counter update",
        );

        let later_hold = build_intro_sequence_5_fade_hold_vdp_at_elapsed(&data, 0, 40, 0);
        assert_eq!(
            vram_word(&later_hold, 0x019A),
            0x1105,
            "ASM loc_1BA90 advances the worker elapsed from INTRO_FIFTH_SEGMENT_FRAMES; subtracting the fade elapsed would regress to the frame-6 boundary",
        );

        let faded = build_intro_sequence_5_fade_hold_vdp_at_elapsed(&data, 0, 2, 0);
        let target = [0x0AAA; 0x40];
        let expected = expected_palette_fade(0x00E0, &source, &target);
        assert_eq!(
            vdp_cram_words(&faded),
            expected,
            "ASM loc_1BA90 fills Palette_Buffer_3 with `$AAA` and runs sub_1C246 after the one-frame latch",
        );
    }

    #[test]
    fn intro_sequence_3_sprite_frame_vram_maps_all_data_index_entries() {
        let expected = [
            (0x1208, 0x000, 0x18, 0x24),
            (0x120C, 0x00F, 0x15, 0x23),
            (0x1210, 0x01E, 0x12, 0x28),
            (0x1214, 0x02D, 0x12, 0x25),
            (0x1218, 0x03C, 0x12, 0x23),
            (0x121C, 0x04B, 0x13, 0x1F),
            (0x1220, 0x057, 0x1D, 0x2C),
            (0x1224, 0x06F, 0x20, 0x24),
            (0x1228, 0x083, 0x13, 0x1D),
            (0x122C, 0x08F, 0x15, 0x2D),
            (0x1230, 0x0A1, 0x0F, 0x23),
            (0x1234, 0x0AB, 0x0F, 0x23),
            (0x1238, 0x0B5, 0x1D, 0x32),
            (0x123C, 0x0D1, 0x18, 0x23),
        ];

        for (addroffset, sprite_index, width, height) in expected {
            let frame = intro_sequence_3_sprite_frame_vram(addroffset)
                .expect("known Load_IntroSequence3 frame");
            assert_eq!(frame.sprite_index, sprite_index);
            assert_eq!(frame.width, width);
            assert_eq!(frame.height, height);
        }
        assert_eq!(intro_sequence_3_sprite_frame_vram(0x1240), None);
    }

    #[test]
    fn make_sprite_from_gfx_object_loc_129e_packs_pattern_and_offsets_exactly() {
        let mut entries = Vec::new();

        append_intro_sprite_entries_make_sprite_from_gfx_object_loc_129E(
            &mut entries,
            IntroGfxObjectForSpriteTable {
                x_pos: 0x0040,
                y_pos: 0x0080,
                palette_line: 2,
                vram_tile: 0x0123,
                priority: 1,
                x_direction: 0,
            },
            IntroSpriteFrameVram {
                sprite_index: 0x0007,
                x_center: 3,
                y_center: 5,
                width: 0x10,
                height: 0x10,
            },
        );

        assert_eq!(
            entries[0],
            HardwareSpriteEntry {
                y_pos: 0x00F5,
                size_byte: 0x05,
                pattern: 0xC12A,
                x_pos: 0x00BD,
            }
        );

        entries.clear();
        append_intro_sprite_entries_make_sprite_from_gfx_object_loc_129E(
            &mut entries,
            IntroGfxObjectForSpriteTable {
                x_pos: 0x0040,
                y_pos: 0x0080,
                palette_line: 2,
                vram_tile: 0x0123,
                priority: 1,
                x_direction: 0xFF,
            },
            IntroSpriteFrameVram {
                sprite_index: 0x0007,
                x_center: 3,
                y_center: 5,
                width: 0x10,
                height: 0x10,
            },
        );

        assert_eq!(
            entries[0],
            HardwareSpriteEntry {
                y_pos: 0x00F5,
                size_byte: 0x05,
                pattern: 0xC92A,
                x_pos: 0x00B3,
            },
            "ASM loc_134A uses x_pos + x_center for the flipped right edge before subtracting the emitted chunk width"
        );
    }

    #[test]
    fn sprite_table_base_byte_addr_masks_register_5_by_h40_mode() {
        let mut registers = [0u8; 0x20];
        registers[5] = 0xFF;
        registers[0x0C] = 0;
        assert_eq!(sprite_table_base_byte_addr(&registers), 0xFE00);

        registers[0x0C] = 0x81;
        assert_eq!(sprite_table_base_byte_addr(&registers), 0xFC00);
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

        assert!(
            sprite_table_index_without_link(&vdp, (0x012C, 0x0B, 0xA1D6, 0x0178)).is_some(),
            "ASM Load_IntroSequence3 loc_1AEEC should expose roster 1 descriptor 0 through Make_SpriteFromGfxObject's first hardware sprite from Data_Index `$1208` / unk_E1106",
        );
        assert!(
            sprite_table_index_without_link(&vdp, (0x014C, 0x08, 0xA1E2, 0x0178)).is_some(),
            "ASM Make_SpriteFromGfxObject should split the 36px-tall Wild Side kid frame into the linked bottom chunk with the next tile run",
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

        assert!(
            sprite_table_index_without_link(&vdp, (0x012F, 0x0B, 0xA259, 0x0163)).is_some(),
            "ASM loc_1AEBA should choose roster 2's first descriptor instead of reusing roster 1's first x/y pair, even though lower gfx slots render before the roster",
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
    fn intro_sequence_3_sub_1C204_keeps_palette_black_until_post_roster_fade() {
        let data = IntroSub1B850Data {
            palette_segment_0: vec![0x0EEE; 0x40],
            ..empty_intro_data()
        };

        let before = build_intro_sequence_vdp_with_title_variant_at_elapsed(
            &data,
            2,
            1,
            INTRO_SEQUENCE_3_KID_VISIBLE_ELAPSED_FRAMES - 4,
            1,
        );
        assert_eq!(
            cram_word(&before, 1),
            0,
            "ASM sub_1B93E enters sub_1C204's hidden fade shell before loc_1AEEC exposes the Wild Side roster, so early intro-sequence-3 CRAM should stay black",
        );

        let roster_live = build_intro_sequence_vdp_with_title_variant_at_elapsed(
            &data,
            2,
            1,
            INTRO_SEQUENCE_3_KID_VISIBLE_ELAPSED_FRAMES,
            1,
        );
        assert_eq!(
            cram_word(&roster_live, 1),
            0,
            "ASM loc_1AEEC's first trace-visible roster pass still runs under sub_1C204's black palette",
        );

        let first_fade = build_intro_sequence_vdp_with_title_variant_at_elapsed(
            &data,
            2,
            1,
            INTRO_SEQUENCE_3_FADE_IN_START_ELAPSED,
            1,
        );
        assert_eq!(
            vdp_cram_words(&first_fade),
            expected_palette_fade(0x0010, &[0x0EEE; 0x40], &[0; 0x40]),
            "ASM sub_1B93E -> sub_1C204 exposes the first Pal_1C28A fade step after the roster is live",
        );
    }

    #[test]
    #[allow(non_snake_case)]
    fn intro_sequence_4_sub_1C204_keeps_palette_black_before_visible_elapsed() {
        let data = IntroSub1B850Data {
            palette_segment_1: vec![0x0EEE; 0x40],
            ..empty_intro_data()
        };

        let before = build_intro_sequence_vdp_with_title_variant_at_elapsed(
            &data,
            3,
            1,
            INTRO_SEQUENCE_3_KID_VISIBLE_ELAPSED_FRAMES - 3,
            1,
        );
        assert_eq!(
            cram_word(&before, 1),
            0,
            "ASM Load_IntroSequence4 -> sub_1B988 calls sub_1C204 before loc_1B9B2; frame 1320 oracle keeps the corridor black during this hidden fade shell",
        );

        let visible = build_intro_sequence_vdp_with_title_variant_at_elapsed(
            &data,
            3,
            1,
            INTRO_SEQUENCE_3_KID_VISIBLE_ELAPSED_FRAMES,
            1,
        );
        assert_eq!(
            vdp_cram_words(&visible),
            expected_palette_fade(0x0030, &[0x0EEE; 0x40], &[0; 0x40]),
            "ASM sub_1C204 should expose the third Pal_1C2F0 fade step at the first roster-visible elapsed frame",
        );
    }

    #[test]
    #[allow(non_snake_case)]
    fn intro_sequence_4_introtext4_matches_oracle_first_visible_cadence() {
        let data = IntroSub1B850Data {
            palette_segment_1: vec![0x0EEE; 0x40],
            ..empty_intro_data()
        };

        let vdp = build_intro_sequence_vdp_with_title_variant_at_elapsed(&data, 3, 1, 98, 1);
        assert_eq!(
            ((u16::from(vdp.vsram[0]) << 8) | u16::from(vdp.vsram[1])),
            4,
            "ASM sub_1B988 writes Camera_Y_pos=4 before the visible corridor loop; oracle frame 1380 exposes that as plane-A VSRAM",
        );
        assert_eq!(
            ((u16::from(vdp.vsram[2]) << 8) | u16::from(vdp.vsram[3])),
            1,
            "Oracle frame 1380 exposes the paired plane-B VSRAM offset while the corridor tilemap is visible",
        );
        let l_write = wildside_data::title_intro_glyph_write_sub_1C7F6(0, 0xC7D3, 19, 8, 0x0B);
        let i_write = wildside_data::title_intro_glyph_write_sub_1C7F6(0, 0xC7D3, 20, 8, 0x08);

        assert_eq!(
            vram_word(&vdp, usize::from(l_write.vram_word_address)),
            l_write.tile_word,
            "ASM IntroText4 via loc_1C7A0 should have reached the `L` in `BUT IT WAS A L` by the frame-1380 oracle checkpoint",
        );
        assert_eq!(
            vram_word(&vdp, usize::from(i_write.vram_word_address)),
            0,
            "ASM IntroText4 should not yet reveal the following `I` at the same checkpoint; oracle frame 1380 keeps `$C7D3` corridor-script unrevealed cells as literal zero",
        );
    }

    #[test]
    #[allow(non_snake_case)]
    fn intro_sequence_5_introtext5_matches_oracle_frame_1980_cadence() {
        let data = IntroSub1B850Data {
            palette_segment_1: vec![0x0EEE; 0x40],
            ..empty_intro_data()
        };

        let vdp = build_intro_sequence_vdp_with_title_variant_at_elapsed(&data, 4, 1, 104, 1);
        let s_write = wildside_data::title_intro_glyph_write_sub_1C7F6(0, 0xC7D3, 31, 3, 0x12);
        let t_write = wildside_data::title_intro_glyph_write_sub_1C7F6(0, 0xC7D3, 33, 3, 0x13);

        assert_eq!(
            vram_word(&vdp, usize::from(s_write.vram_word_address)),
            s_write.tile_word,
            "ASM loc_1BA28 allocates IntroText5 before sub_1C204; frame-1980 oracle shows `THIS IS` by intro-sequence-5 elapsed 104",
        );
        assert_eq!(
            vram_word(&vdp, usize::from(t_write.vram_word_address)),
            0,
            "ASM IntroText5 should not yet reveal the `T` in `THE` at the same frame-1980 checkpoint; `$C7D3` corridor-script unrevealed cells match oracle as literal zero",
        );
    }

    #[test]
    #[allow(non_snake_case)]
    fn intro_sequence_5_loc_1BA28_keeps_vdp_blank_before_visible_elapsed() {
        let data = IntroSub1B850Data {
            palette_segment_2: vec![0x0EEE; 0x40],
            intro_sequence_5_tilemap: vec![0x0ABC; 40 * 28],
            ..empty_intro_data()
        };

        let before = build_intro_sequence_vdp_with_title_variant_at_elapsed(&data, 4, 1, 44, 1);
        assert_eq!(
            cram_word(&before, 1),
            0,
            "ASM loc_1BA28 enters sub_1C204 before loc_1BA5C; oracle frame 1920 keeps intro-sequence-5 CRAM black at elapsed 44",
        );
        assert_eq!(
            vram_word(&before, 0xE000),
            0,
            "ASM loc_1BA28/sub_1C204 should not expose the alley nametable before the visible loop; oracle frame 1920 has blank VRAM at elapsed 44",
        );

        let old_visible_edge =
            build_intro_sequence_vdp_with_title_variant_at_elapsed(&data, 4, 1, 58, 1);
        assert_eq!(
            cram_word(&old_visible_edge, 1),
            0,
            "ASM loc_1BA28/sub_1C204 should still present black at local elapsed 58; oracle frame 1934 has not exposed the alley/Kid shell yet",
        );
        assert_eq!(
            vram_word(&old_visible_edge, 0xE000),
            0,
            "ASM loc_1BA28/sub_1C204 should not expose MapEni_2E154 at local elapsed 58; this guards the early bottom-left intro artefact seen before frame 1980",
        );

        let visible = build_intro_sequence_vdp_with_title_variant_at_elapsed(
            &data,
            4,
            1,
            INTRO_SEQUENCE_5_VISIBLE_ELAPSED_LOC_1BA28,
            1,
        );
        assert_eq!(
            cram_word(&visible, 1),
            0x0EEE,
            "ASM loc_1BA5C should expose IntroSequence5's loaded palette once the hidden sub_1C204 shell has completed",
        );
        assert_eq!(
            vram_word(&visible, 0xE000),
            0x0ABC,
            "ASM loc_1BA5C should expose Load_IntroSequence5's MapEni_2E154 alley plane after the hidden shell",
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
            sprite_table_index_without_link(&vdp, (0x0119, 0x0F, 0x0140, 0x0101)).is_some(),
            "ASM Load_IntroSequence3 runs sub_1B93E -> sub_1C54A, but sub_1C54A only sets display byte `$13(a1)`, not priority `$10(a1)`",
        );
    }

    #[test]
    #[allow(non_snake_case)]
    fn intro_sequence_3_sprite_order_follows_gfx_slot_order() {
        let vdp = build_intro_sequence_vdp_with_title_variant_at_elapsed(
            &empty_intro_data(),
            2,
            1,
            INTRO_SEQUENCE_3_KID_VISIBLE_ELAPSED_FRAMES,
            1,
        );

        assert_eq!(
            sprite_entry_words(&vdp, 0),
            (0x012C, 0x0B, 1, 0xA1D6, 0x0178),
            "Oracle VRAM frame 1080 shows ASM loc_FD4 hardware output starts with the `$0201` high-priority roster chain, using Data_Index `$1208` / unk_E1106",
        );
        let high_roster = sprite_table_index_without_link(&vdp, (0x012C, 0x0B, 0xA1D6, 0x0178))
            .expect("ASM loc_1AEEC should allocate `$0201` high-priority roster objects");
        let right_door = sprite_table_index_without_link(&vdp, (0x0118, 0x0F, 0x018E, 0x015E))
            .expect("ASM loc_1C58C should allocate the `$0200` right door object");
        let left_door = sprite_table_index_without_link(&vdp, (0x0118, 0x0F, 0x0170, 0x00BA))
            .expect("ASM sub_1C572 should allocate the `$0200` left door object");
        let low_roster = sprite_table_index_without_link(&vdp, (0x0125, 0x0B, 0x2221, 0x0118))
            .expect("ASM loc_1AEEC should allocate the `$01FF` low-priority roster object");
        let fixed_door = sprite_table_index_without_link(&vdp, (0x0119, 0x0F, 0x0140, 0x0101))
            .expect("ASM sub_1C54A should allocate the `$0100` fixed door/interior object");
        assert!(
            high_roster < right_door
                && right_door < left_door
                && left_door < low_roster
                && low_roster < fixed_door,
            "Oracle VRAM frame 1080 shows the linked hardware table order as `$0201` roster, `$0200` right/left doors, `$01FF` roster, then `$0100` fixed door/interior"
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
            sprite_table_index_without_link(&initial, (0x0118, 0x0F, 0x0170, 0x00BA)).is_some(),
            "ASM sub_1C572 should draw the low-priority left Wild Side door half at x=$3A before the `$3C` hibernate expires",
        );
        assert!(
            sprite_table_index_without_link(&initial, (0x0118, 0x0F, 0x018E, 0x015E)).is_some(),
            "ASM loc_1C58C should draw the low-priority right Wild Side door half at x=$DE before the `$3C` hibernate expires",
        );

        let after_first_move = build_intro_sequence_vdp_with_title_variant_at_elapsed(
            &empty_intro_data(),
            2,
            1,
            INTRO_SEQUENCE_3_KID_VISIBLE_ELAPSED_FRAMES + 0x3F,
            1,
        );

        assert!(
            sprite_table_index_without_link(&after_first_move, (0x0118, 0x0F, 0x0170, 0x00BB))
                .is_some(),
            "Oracle VRAM shows ASM loc_1C5BE's first visible left-door move two frames after the raw `$3C` hibernate expires",
        );
        assert!(
            sprite_table_index_without_link(&after_first_move, (0x0118, 0x0F, 0x018E, 0x015D))
                .is_some(),
            "Oracle VRAM shows ASM loc_1C5BE's first visible right-door move two frames after the raw `$3C` hibernate expires",
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
            (0x0119, 0x0F, 1, 0x0140, 0x0101),
            "ASM sub_1C54A should draw low-priority unk_E109E's first 32x32 chunk from the Wild Side door/interior art at VRAM tile `$140`",
        );
        assert_eq!(
            sprite_entry_words(&vdp, 1),
            (0x0119, 0x0F, 2, 0x0150, 0x0121),
            "ASM Make_SpriteFromGfxObject should split unk_E109E's 62px-wide top row into the linked right-hand chunk",
        );
        assert_eq!(
            sprite_entry_words(&vdp, 2),
            (0x0139, 0x0D, 3, 0x0160, 0x0101),
            "ASM Make_SpriteFromGfxObject should continue unk_E109E's bottom row after the two 32px-high chunks",
        );
        assert_eq!(
            sprite_entry_words(&vdp, 3),
            (0x0139, 0x0D, 0, 0x0168, 0x0121),
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
        let vdp = build_intro_sequence_vdp_with_title_variant_at_elapsed(
            &empty_intro_data(),
            4,
            1,
            INTRO_SEQUENCE_5_VISIBLE_ELAPSED_LOC_1BA28,
            0,
        );

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
        let visible_particle_start = INTRO_SEQUENCE_5_PARTICLE_START_ELAPSED_LOC_1BCD0
            + INTRO_SEQUENCE_5_PARTICLE_VISIBLE_DELAY_LOC_1BCD0;
        let before = build_intro_sequence_vdp_with_title_variant_at_elapsed(
            &empty_intro_data(),
            4,
            1,
            visible_particle_start - 1,
            0,
        );
        assert_eq!(
            sprite_entry_words(&before, 16),
            (0, 0, 0, 0, 0),
            "ASM loc_1BCD0 should not render loc_1BE3C particles until the cold-boot oracle-visible post-hibernate delay expires",
        );
        assert!(
            sprite_table_index_without_link(&before, (0x00D0, 0x0F, 0x426C, 0x00B0)).is_none(),
            "ASM loc_1BCD0 removes the first P-block gfx slot one frame before loc_1BE3C particles are visible",
        );
        assert!(
            sprite_table_index_without_link(&before, (0x0130, 0x0F, 0x429E, 0x0080)).is_some(),
            "ASM loc_1BCD0 keeps the lower static gfx slots during the one-frame pre-particle hand-off",
        );

        let after = build_intro_sequence_vdp_with_title_variant_at_elapsed(
            &empty_intro_data(),
            4,
            1,
            visible_particle_start,
            0,
        );
        assert_eq!(
            sprite_entry_words(&after, 0),
            (0x00C1, 0x0F, 1, 0x42D2, 0x0092),
            "ASM loc_1BE3C should spawn the first unk_1BE18 falling particle from Data_Index+$11C8/unk_E1086 at x=$12/y=$78",
        );
        assert_eq!(
            sprite_entry_words(&after, 10),
            (0x00B9, 0x0F, 11, 0xE5EB, 0x00B0),
            "ASM loc_1BE88 should spawn the FC82=1 centre particle on palette line 3 with priority and VRAM tile $5EB",
        );
    }

    #[test]
    #[allow(non_snake_case)]
    fn intro_sequence_5_loc_1BE3C_fc82_two_presents_particles_after_one_sat_latch() {
        let visible_particle_start = INTRO_SEQUENCE_5_PARTICLE_START_ELAPSED_LOC_1BCD0
            + INTRO_SEQUENCE_5_PARTICLE_VISIBLE_DELAY_LOC_1BCD0;
        let pre_boundary = build_timeout_restart_intro_sequence_5_vdp_at_elapsed_loc_1ba28(
            &empty_intro_data(),
            2,
            visible_particle_start - 1,
            0,
            false,
        );
        let boundary = build_timeout_restart_intro_sequence_5_vdp_at_elapsed_loc_1ba28(
            &empty_intro_data(),
            2,
            visible_particle_start,
            0,
            false,
        );
        let presented = build_timeout_restart_intro_sequence_5_vdp_at_elapsed_loc_1ba28(
            &empty_intro_data(),
            2,
            visible_particle_start + 1,
            0,
            false,
        );

        assert_eq!(
            sprite_entry_words(&pre_boundary, 0),
            (0x00D0, 0x0F, 1, 0x426C, 0x00B0),
            "oracle frame 20848 keeps loc_1BCD0's first P-block slot through the FC82=2 pre-particle SAT latch",
        );
        assert_eq!(
            sprite_entry_words(&boundary, 0),
            (0x0130, 0x0F, 1, 0x429E, 0x0080),
            "ASM loc_1BCD0/loc_1BE3C leaves the FC82=2 pre-particle SAT visible for one final outer trace on oracle frame 20849",
        );
        assert_eq!(
            sprite_entry_words(&presented, 0),
            (0x00E9, 0x0F, 1, 0x431A, 0x00B0),
            "oracle frame 20850 presents the newest loc_1BE30 worker first at its already-advanced position",
        );
    }

    #[test]
    #[allow(non_snake_case)]
    fn intro_sequence_5_loc_1BE3C_deletes_short_lived_particles_by_dbf_count() {
        let vdp = build_intro_sequence_vdp_with_title_variant_at_elapsed(
            &empty_intro_data(),
            4,
            1,
            INTRO_SEQUENCE_5_PARTICLE_START_ELAPSED_LOC_1BCD0
                + INTRO_SEQUENCE_5_PARTICLE_VISIBLE_DELAY_LOC_1BCD0
                + 0x29,
            0,
        );

        assert_eq!(
            sprite_entry_words(&vdp, 0),
            (0x0142, 0x0F, 1, 0x42FC, 0x017C),
            "ASM loc_1BE74 should delete the two `$28` DBF particles after frame $28, leaving the longer Data_Index+$11CC/unk_E108E particle as the first live particle",
        );
    }

    #[test]
    #[allow(non_snake_case)]
    fn intro_sequence_5_loc_1BE3C_particles_move_on_first_visible_frame() {
        let mut entries = Vec::new();
        append_intro_sequence_5_particles_loc_1BE3C(
            &mut entries,
            1,
            IntroSequence5ParticleLineage::TimeoutFc82Zero,
        );

        assert_eq!(
            entries[0],
            HardwareSpriteEntry {
                y_pos: 0x00E9,
                size_byte: 0x0F,
                pattern: 0x431A,
                x_pos: 0x00B0,
            },
            "ASM loc_1BCD0 allocates unk_1BE18/24/30 in order, but oracle frame 28022 exposes loc_1BE3C's newest unk_1BE30 worker first",
        );
        assert_eq!(
            entries[6],
            HardwareSpriteEntry {
                y_pos: 0x00BF,
                size_byte: 0x0F,
                pattern: 0x42D2,
                x_pos: 0x008E,
            },
            "ASM loc_1BE3C/loc_1BE74 applies the signed x/y velocities before the first post-spawn particle frame is displayed",
        );
    }

    #[test]
    #[allow(non_snake_case)]
    fn intro_sequence_5_loc_1BE3C_keeps_short_particles_through_dbf_zero() {
        let mut entries = Vec::new();
        append_intro_sequence_5_particles_loc_1BE3C(
            &mut entries,
            0x28,
            IntroSequence5ParticleLineage::TimeoutFc82Zero,
        );

        assert_eq!(
            entries.len(),
            10,
            "ASM loc_1BE74 uses DBF semantics: particles with loop count `$28` still display when the counter reaches zero",
        );
    }

    #[test]
    #[allow(non_snake_case)]
    fn intro_sequence_5_loc_1BE88_centre_particle_uses_post_hibernate_gravity_order() {
        let mut entries = Vec::new();
        append_intro_sequence_5_center_particle_loc_1BE88(&mut entries, 0, 0x40);

        assert_eq!(
            entries[0],
            HardwareSpriteEntry {
                y_pos: 0x0098,
                size_byte: 0x0F,
                pattern: 0xE5EB,
                x_pos: 0x00FF,
            },
            "ASM loc_1BE88/loc_1BF0A should keep moving for DBF #$1C's 29 hibernates, while y_vel's `$9000` gravity increment applies after the displayed frame",
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
        let cold_boot_after = build_intro_sequence_1_cold_boot_vdp_at_elapsed(
            &data,
            &minimal_sega_data(),
            1,
            first_glyph_elapsed,
        );
        let raster = cold_boot_after
            .raster_frame
            .as_ref()
            .expect("loc_1C7F6's active-display tile write should retain prior scanout VRAM");
        assert_eq!(
            u16::from_be_bytes([raster.initial_vram[0x0110], raster.initial_vram[0x0111]]),
            INTRO_TEXT_CLEAR_TILE_WORD_LOC_1C7A0,
            "ASM loc_1C7A0/sub_1C7F6 updates end-of-frame VRAM first; the `T` becomes visible on the following scanout",
        );
    }

    #[test]
    fn introtext4_counted_lifetime_reaches_ff_loc_1c7b6_after_298_traces() {
        assert_eq!(
            intro_text_worker_counted_frames_loc_1c7a0(3),
            Some(298),
            "ASM IntroText4 has one allocation trace, one initial loc_1C7AE hibernate, `$20`/`$40` pauses, and 100 glyphs at two frames each before loc_1C7B6 dispatches `$FF`",
        );
    }

    #[test]
    fn introtext5_counted_lifetime_reaches_ff_loc_1c7b6_after_254_traces() {
        assert_eq!(
            intro_text_worker_counted_frames_loc_1c7a0(4),
            Some(254),
            "ASM IntroText5 has one allocation trace, one initial loc_1C7AE hibernate, `$20`/`$60` pauses, and 62 glyphs at two frames each before loc_1C7B6 dispatches `$FF`",
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
    fn intro_sequence_2_text_delay_preserves_hologram_cells_before_introtext2_glyphs() {
        let mut hologram_tilemap = vec![0u16; 40 * 28];
        hologram_tilemap[3 * 40 + 7] = 0x213B;
        let data = IntroSub1B850Data {
            intro_sequence_2_hologram_tilemap: hologram_tilemap,
            ..empty_intro_data()
        };

        let before_introtext2_first_glyph = INTRO_TEXT_WORKER_START_ELAPSED_LOC_1C7A0[1] + 1 + 0x8F;
        let vdp = build_intro_sequence_vdp_with_title_variant_at_elapsed(
            &data,
            1,
            1,
            before_introtext2_first_glyph,
            0,
        );

        assert_eq!(
            vram_word(&vdp, 0xE18E),
            0x213B,
            "ASM IntroText2 starts with `$FE,$90`; loc_1C7A0 must not blank plane-B hologram cells before sub_1C7F6 reaches the first glyph",
        );
    }

    #[test]
    fn introtext2_allocation_anchor_waits_for_the_final_period_loc_1c7a0() {
        assert_eq!(
            INTRO_TEXT_WORKER_START_ELAPSED_LOC_1C7A0[1], 43,
            "ASM loc_1B9D0 allocates IntroText2 before loc_1C7A0's hibernate/parser loop; oracle VRAM keeps the final plane-B cell at `$E2B8` unchanged on published frame 5787 and writes `$C7F8` on 5788"
        );
    }

    #[test]
    fn introtext4_allocation_anchor_matches_oracle_glyph_cadence_loc_1c7a0() {
        assert_eq!(
            INTRO_TEXT_WORKER_START_ELAPSED_LOC_1C7A0[3], 38,
            "ASM sub_1B988 allocates IntroText4's loc_1C7A0 worker before the fade shell; oracle VRAM writes its successive glyph cells at published frames 20000, 20002, 20004, and 20006, while anchor 39 delays each native write one frame",
        );
    }

    #[test]
    fn intro_text_script_table_keeps_introtext2_and_introtext3_headers_loc_1c7a0() {
        let introtext2 = intro_text_script_loc_1c7a0(1)
            .expect("ASM IntroText2 table row should exist for Load_IntroSequence2");
        assert_eq!(
            (
                introtext2.base_vram_byte_address,
                introtext2.base_tile_word,
                introtext2.x_pos,
                introtext2.y_pos,
                introtext2.text_bytes,
            ),
            (0xE000, 0xC7D3, 7, 3, INTRO_TEXT_2_BYTES),
            "ASM IntroText2 (kid.asm:31384) targets the plane-B hologram nametable and must not fall through to another script row",
        );

        let introtext3 = intro_text_script_loc_1c7a0(2)
            .expect("ASM IntroText3 table row should exist for Load_IntroSequence3");
        assert_eq!(
            (
                introtext3.base_vram_byte_address,
                introtext3.base_tile_word,
                introtext3.x_pos,
                introtext3.y_pos,
                introtext3.text_bytes,
            ),
            (0x0000, 0x67D3, 0x0A, 4, INTRO_TEXT_3_BYTES),
            "ASM IntroText3 (kid.asm:31386) has its own plane-A row and must not disappear from loc_1C7A0's script table",
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
    fn intro_text_cleanup_honours_ff_terminator_and_fe_delay_marker_loc_1c7a0() {
        static TERMINATED: &[u8] = b"A\xFFB";
        let terminated = IntroTextScriptLoc1C7A0 {
            base_vram_byte_address: 0,
            base_tile_word: 0x67D3,
            delay_frames: 2,
            x_pos: 1,
            y_pos: 1,
            text_bytes: TERMINATED,
        };
        let mut vdp = VdpState::new();
        clear_intro_text_script_cells_loc_1c7a0(&mut vdp, &terminated);
        let after_terminator = wildside_data::title_intro_glyph_write_sub_1C7F6(0, 0x67D3, 2, 1, 0);
        assert_eq!(
            vram_word(&vdp, usize::from(after_terminator.vram_word_address)),
            0,
            "ASM loc_1C7A0 stops cleanup at `$FF`; bytes after the terminator are data-dead and must not clear later cells",
        );

        static DELAYED: &[u8] = b"\xFE\x02A\xFF";
        let delayed = IntroTextScriptLoc1C7A0 {
            text_bytes: DELAYED,
            ..terminated
        };
        let mut vdp = VdpState::new();
        clear_intro_text_script_cells_loc_1c7a0(&mut vdp, &delayed);
        let skipped_delay_byte =
            wildside_data::title_intro_glyph_write_sub_1C7F6(0, 0x67D3, 2, 1, 0);
        assert_eq!(
            vram_word(&vdp, usize::from(skipped_delay_byte.vram_word_address)),
            0,
            "ASM loc_1C7D2 treats `$FE,<duration>` as a delay marker; the duration byte is not a glyph cell to clear",
        );
    }

    #[test]
    fn intro_sequence_2_handoff_fades_intro1_to_black_before_corridor_loc_1b9d0_1c204() {
        let mut data = empty_intro_data();
        data.palette_segment_0 = vec![0x0EEE; 0x34];
        data.palette_segment_1 = vec![0x0A00; 0x24];

        let previous_scene =
            build_intro_sequence_vdp_with_title_variant_at_elapsed(&data, 1, 1, 0, 0);
        assert_eq!(
            cram_word(&previous_scene, 0),
            0x0EEE,
            "ASM loc_1B9D0 enters sub_1C204 with the completed IntroSequence1 palette still visible",
        );

        let black_hold = build_intro_sequence_vdp_with_title_variant_at_elapsed(
            &data,
            1,
            1,
            INTRO_SEQUENCE_2_PREVIOUS_FADE_OUT_ELAPSED,
            0,
        );
        assert_eq!(
            cram_word(&black_hold, 0),
            0,
            "ASM sub_1C204 should hold CRAM black after fading out the previous intro scene",
        );

        let full_corridor = build_intro_sequence_vdp_with_title_variant_at_elapsed(
            &data,
            1,
            1,
            INTRO_SEQUENCE_2_FULL_PALETTE_ELAPSED,
            0,
        );
        assert_eq!(
            cram_word(&full_corridor, 0),
            0x0A00,
            "ASM sub_1C204 should fade Pal_1C2F0 in before loc_1B9F4's corridor loop is fully visible",
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
        let vdp = build_sega_screen_vdp(&data, SEGA_VDP_VISIBLE_START_ELAPSED);

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
        let vdp = build_sega_screen_vdp(&data, SEGA_VDP_VISIBLE_START_ELAPSED);

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
    fn build_sega_screen_vdp_fade_loc_12e96_first_palette_strip_lands_at_elapsed_67() {
        // ASM Load_SegaScreen loc_12E48/loc_12E64 (kid.asm:25483-25510):
        // the ROM-visible VDP output stays black while the mode shell reaches
        // the logo wait loop, then holds the full Pal_12EEC copy before the
        // loc_12E96 fade loop changes CRAM.  In oracle frames this means:
        // black through native elapsed 33 and full logo from elapsed 34. The
        // native VDP latch must prepare the first loc_12ED2 strip at elapsed
        // 67 so the rendered checkpoint changes on oracle frame 70.
        //
        // loc_12E96 (kid.asm:25515-25543) seeds `moveq #$14,d1` (frame delay)
        // and `moveq #$28,d3` (palette cursor), then `subq.w #1,d1; bpl.s
        // loc_12ED8`, so a strip copy only fires when d1 transitions from 0
        // to -1.  At that point d0 captures the cursor BEFORE `subq.w #2,d3`,
        // and `lea Pal_12EEC+6(pc,d0.w),a0` resolves to palette word index
        // 2 + d0/2 (Pal_12EEC = [dest, count, SegaLogo.bin...]; the +6
        // offset and the absent header in `palette_words` cancel to +2 word
        // indices).  loc_12ED2 copies `moveq #$A,d0; dbf d0` = 11 words into
        // Palette_Buffer+$4 (CRAM word offset 2).  Distinct palette words pin
        // the source index, strip slice, destination, and cursor decrement.
        let mut palette_words = vec![0u16; 33];
        for (i, slot) in palette_words.iter_mut().enumerate() {
            *slot = 0x1000 + i as u16;
        }
        let data = SegaScreenData {
            art_bytes: vec![0; 32],
            palette_words: palette_words.clone(),
        };

        let vdp_hidden = build_sega_screen_vdp(&data, SEGA_VDP_VISIBLE_START_ELAPSED - 1);
        assert_eq!(
            u16::from_be_bytes([vdp_hidden.cram[4], vdp_hidden.cram[5]]),
            0,
            "elapsed=33: ASM Load_SegaScreen has not produced a visible logo frame yet",
        );

        // Sanity: at the first visible elapsed frame, loc_12DFE's initial
        // Pal_12EEC copy is visible and loc_12ED2 has not changed CRAM yet.
        let vdp_initial = build_sega_screen_vdp(&data, SEGA_VDP_VISIBLE_START_ELAPSED);
        assert_eq!(
            u16::from_be_bytes([vdp_initial.cram[4], vdp_initial.cram[5]]),
            palette_words[2],
            "elapsed=34: CRAM word 2 = initial palette word 2",
        );

        // Sanity: one frame before the first loc_12ED2 copy, d1 has not yet
        // underflowed, so CRAM word 2 still holds the initial palette word.
        let vdp_before_first_copy = build_sega_screen_vdp(&data, SEGA_FIRST_FADE_COPY_ELAPSED - 1);
        assert_eq!(
            u16::from_be_bytes([vdp_before_first_copy.cram[4], vdp_before_first_copy.cram[5],]),
            palette_words[2],
            "one frame before the first loc_12ED2 strip copy: CRAM still holds the initial palette",
        );

        // First copy fires at elapsed=67. Cursor on entry is $28, so
        // source_word_index = 2 + 0x28/2 = 22.  All 11 strip words must equal
        // palette_words[22..33].
        let vdp_first_copy = build_sega_screen_vdp(&data, SEGA_FIRST_FADE_COPY_ELAPSED);
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
        let vdp_second_copy = build_sega_screen_vdp(&data, SEGA_FIRST_FADE_COPY_ELAPSED + 4);
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

        // After 22 copies (elapsed = first copy + 21*4), cursor has
        // decremented from $28 to 0 and the 22nd copy uses source_word_index =
        // 2 + 0/2 = 2.  ASM loc_12E96 (kid.asm:25531): `bmi.w loc_12ED8`
        // skips only when d3 (the cursor) is already negative — at d3=0 the
        // branch is NOT taken, so the copy still fires for cursor==0.  This
        // pins the strict-less-than bound on line 161 (`palette_cursor < 0`):
        // any mutant that loosens it to `<=`/`==` would skip the cursor==0
        // copy and leave CRAM word 2 holding the previous strip's value
        // (palette_words[3], from the 21st copy at cursor=2).
        let vdp_cursor_zero = build_sega_screen_vdp(&data, SEGA_FIRST_FADE_COPY_ELAPSED + 21 * 4);
        assert_eq!(
            u16::from_be_bytes([vdp_cursor_zero.cram[4], vdp_cursor_zero.cram[5]]),
            palette_words[2],
            "22nd fade copy at cursor=0: CRAM word 2 = palette_words[2 + 0/2] = palette_words[2]",
        );
    }

    #[test]
    fn sega_fade_loc_12ed2_retains_previous_cram_during_three_visible_scanouts() {
        // ASM loc_12E96..loc_12ED2 (kid.asm:25515-25543) updates
        // Palette_Buffer and queues Palette_to_VRAM after WaitForVint. Oracle
        // FIFO captures at published-TAS frames 5037..5040 show the first
        // changed strip commits below the visible field, while the logo keeps
        // the preceding CRAM for three scanouts. The final VDP state remains
        // post-DMA; only raster scanout must use the delayed palette.
        let mut palette_words = vec![0u16; 33];
        for (index, word) in palette_words.iter_mut().enumerate() {
            *word = 0x1000 + index as u16;
        }
        let data = SegaScreenData {
            art_bytes: vec![0; 32],
            palette_words: palette_words.clone(),
        };

        let first_copy = build_sega_screen_vdp(&data, SEGA_FIRST_FADE_COPY_ELAPSED);
        assert_eq!(
            u16::from_be_bytes([first_copy.cram[4], first_copy.cram[5]]),
            palette_words[22],
            "loc_12ED2's completed VDP state keeps the newly copied strip"
        );
        assert_eq!(
            first_copy
                .raster_frame
                .as_ref()
                .and_then(|raster| raster.initial_cram.as_ref())
                .map(|cram| u16::from_be_bytes([cram[4], cram[5]])),
            Some(palette_words[2]),
            "the first three scanouts retain Pal_12EEC's preceding CRAM word"
        );

        let presented = build_sega_screen_vdp(
            &data,
            SEGA_FIRST_FADE_COPY_ELAPSED + SEGA_FADE_CRAM_PRESENTATION_LAG_LOC_12ED2,
        );
        assert!(
            presented.raster_frame.is_none(),
            "after the three delayed scanouts, the loc_12ED2 strip is visible directly"
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
    fn decompress_to_vram_chunk_spans_follow_rollover_windows() {
        // ASM DecompressToVRAM loc_14300 uploads the first full
        // Decompression_Buffer span, loc_1432C rolls the upper $800 bytes down,
        // then the next loc_14300 pass resumes from Decompression_Buffer+$800.
        assert_eq!(
            decompress_chunk_spans(0x2001),
            vec![
                DecompressChunkSpan {
                    source_start: 0x0000,
                    byte_len: 0x1000,
                    buffer_start_addr: 0x0000,
                    rollover_source_end: Some(0x1000),
                },
                DecompressChunkSpan {
                    source_start: 0x1000,
                    byte_len: 0x0800,
                    buffer_start_addr: 0x0800,
                    rollover_source_end: Some(0x1800),
                },
                DecompressChunkSpan {
                    source_start: 0x1800,
                    byte_len: 0x0800,
                    buffer_start_addr: 0x0800,
                    rollover_source_end: None,
                },
            ],
            "ASM loc_1432C/loc_143AA/loc_14430 should make resumed chunks start at the buffer midpoint and preserve odd trailing bytes for the caller",
        );

        let art = (0..0x1000).map(|i| i as u8).collect::<Vec<_>>();
        assert_eq!(
            rollover_source_longwords(&art, 0x1000).len(),
            0x0200,
            "ASM loc_1432C seeds d3=$1FF, so the rollover source is exactly $200 longwords from the upper $800 bytes",
        );
    }

    #[test]
    #[allow(non_snake_case)]
    fn write_decompress_to_vram_loc_14300_preserves_base_offset_chunks_and_odd_tail() {
        let mut vdp = VdpState::new();
        let art = (0..0x1005).map(|byte| byte as u8).collect::<Vec<_>>();

        write_decompress_to_vram_art_loc_14300_14314(&mut vdp, 0x0200, &art);

        for &source_index in &[0usize, 1, 0x0FFE, 0x0FFF, 0x1000, 0x1001, 0x1004] {
            assert_eq!(
                vdp.vram[0x0200 + source_index],
                art[source_index],
                "ASM loc_14300/loc_14314 should stream decompressed byte ${source_index:04X} to VRAM base+source_index across rollover chunks and odd tails",
            );
        }
        assert_eq!(
            vdp.vram[0x0200 + art.len()],
            0,
            "ASM loc_14314 writes exactly the emitted byte count; the odd-tail preservation must not overrun into the following VRAM byte",
        );
    }

    #[test]
    fn decompress_chunk_spans_stop_at_even_byte_length_without_empty_tail_span() {
        assert_eq!(
            decompress_chunk_spans(DECOMPRESS_FIRST_CHUNK_BYTES),
            vec![DecompressChunkSpan {
                source_start: 0,
                byte_len: DECOMPRESS_FIRST_CHUNK_BYTES,
                buffer_start_addr: 0,
                rollover_source_end: None,
            }],
            "ASM loc_14300 only emits the exact full first chunk when the decompressed byte count ends on the buffer boundary",
        );
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
            0x0111,
            "ASM Load_IntroSequence3 -> sub_1B93E calls sub_1C4EA, so segment 2 reuses intro1_wildside palette"
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
            intro_text_letters_art: vec![0x12],
            intro_sequence_2_hologram_tilemap: intro2_hologram,
            intro_sequence_4_hologram_tilemap: vec![0x20BC; 40 * 28],
            intro_sequence_2_tilemap: vec![0x00BC; 40 * 28],
            intro_sequence_2_sky_castle_art: vec![0xFF; 0x0B00],
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
        assert_eq!(
            intro5.vram[0xFA60], 0x89,
            "ASM Load_IntroSequence5 routes ArtComp_1DA86 through unk_1C3F0 before writing IntroText5 glyph patterns to VRAM",
        );
        assert_eq!(intro5.vram[0xBD60], 0xA7);
        assert_eq!(
            u16::from_be_bytes([intro5.vram[0xE000], intro5.vram[0xE001]]),
            0x0ABC,
            "ASM Load_IntroSequence5 should upload MapEni_2E154 through sub_1CD88",
        );
    }

    #[test]
    #[allow(non_snake_case)]
    fn intro_sequence_2_loc_1BA02_reveals_sky_art_after_negative_selector_preroll() {
        let data = IntroSub1B850Data {
            intro_sequence_2_sky_castle_art: vec![0xFF; 0x0B00],
            ..empty_intro_data()
        };

        let before = build_intro_sequence_vdp_with_title_variant_at_elapsed(
            &data,
            1,
            1,
            INTRO_SEQUENCE_2_SKY_REVEAL_LOOP_START_ELAPSED_LOC_1BA02 + 30,
            0,
        );
        assert_eq!(
            vram_word(&before, 0x1780),
            0,
            "ASM loc_1BA02 starts d1 at -$40; after 31 frames of two sub_1C1A2 calls, the selector is still negative and sub_1C180 uploads no XORed sky art",
        );

        let after = build_intro_sequence_vdp_with_title_variant_at_elapsed(
            &data,
            1,
            1,
            INTRO_SEQUENCE_2_SKY_REVEAL_LOOP_START_ELAPSED_LOC_1BA02 + 32,
            0,
        );
        assert_ne!(
            vram_word(&after, 0x1780),
            0,
            "ASM loc_1BA02 calls sub_1C1A2 twice per frame; once d1 reaches zero, sub_1C180 should upload XORed sky/castle art to VRAM $1780",
        );
    }

    #[test]
    #[allow(non_snake_case)]
    fn intro_sequence_2_loc_1BA02_cold_boot_reveal_starts_at_elapsed_151() {
        let data = IntroSub1B850Data {
            intro_sequence_2_sky_castle_art: vec![0xFF; 0x0B00],
            ..empty_intro_data()
        };

        let preroll_tail =
            build_intro_sequence_vdp_with_title_variant_at_elapsed(&data, 1, 1, 150, 0);
        assert_eq!(
            vram_word(&preroll_tail, 0x1780),
            0,
            "ASM loc_1B9D0 -> sub_1C204 -> loc_1B9F4 -> loc_1BA02 keeps the sky reveal hidden through the preroll; oracle issue-210 frame 700 showed native elapsed 166 matching oracle elapsed 184 before this 18-frame visual delay was accounted for",
        );

        let first_visible =
            build_intro_sequence_vdp_with_title_variant_at_elapsed(&data, 1, 1, 151, 0);
        assert_ne!(
            vram_word(&first_visible, 0x1780),
            0,
            "ASM sub_1C1A2 pre-increments d1 twice per loc_1BA02 frame; after the ROM-observed hidden handoff delay, elapsed 151 is the first visible sky/castle upload",
        );
    }

    #[test]
    #[allow(non_snake_case)]
    fn intro_sequence_2_loc_1BA02_clamps_sky_reveal_after_rom_loop_length() {
        let data = IntroSub1B850Data {
            intro_sequence_2_sky_castle_art: (0..0x0B00).map(|byte| byte as u8).collect(),
            ..empty_intro_data()
        };

        let final_rom_loop_frame = build_intro_sequence_vdp_with_title_variant_at_elapsed(
            &data,
            1,
            1,
            INTRO_SEQUENCE_2_SKY_REVEAL_LOOP_START_ELAPSED_LOC_1BA02
                + INTRO_SEQUENCE_2_SKY_REVEAL_LOOP_FRAMES_LOC_1BA02
                - 1,
            0,
        );
        let much_later = build_intro_sequence_vdp_with_title_variant_at_elapsed(
            &data,
            1,
            1,
            INTRO_SEQUENCE_2_SKY_REVEAL_LOOP_START_ELAPSED_LOC_1BA02
                + INTRO_SEQUENCE_2_SKY_REVEAL_LOOP_FRAMES_LOC_1BA02
                + 32,
            0,
        );

        assert_eq!(
            &much_later.vram[0x1780..0x1A00],
            &final_rom_loop_frame.vram[0x1780..0x1A00],
            "ASM loc_1BA02 runs a fixed $014A-frame sky reveal loop; later elapsed values must not keep advancing sub_1C1A2 selectors",
        );
    }

    #[test]
    #[allow(non_snake_case)]
    fn intro_sequence_2_loc_1B00A_applies_special_sky_art_nibble_permutation() {
        assert_eq!(
            intro_sequence_2_special_sky_art_loc_1B00A(&[0x01, 0x23, 0xF0]),
            vec![0x21, 0x33, 0xF2],
            "ASM Load_IntroSequence2 loc_1B00A passes unk_1C336 into DecompressToVRAM_Special, so each sky/castle art nibble should be remapped before VRAM $1780 receives it",
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
                let plan = wildside_data::intro_sequence_5_frame_load_plan_sub_1C034(frame_index)
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

        let first = build_intro_sequence_vdp_with_title_variant_at_elapsed(
            &data,
            4,
            0,
            INTRO_SEQUENCE_5_VISIBLE_ELAPSED_LOC_1BA28,
            0,
        );
        assert_eq!(
            vram_word(&first, 0x019A),
            0x1100,
            "ASM loc_1BC36 calls sub_1C034(d4=0) before the first `$A0` hibernate; left padding is three words"
        );

        let before_second = build_intro_sequence_vdp_with_title_variant_at_elapsed(
            &data,
            4,
            0,
            0xA0 + INTRO_SEQUENCE_5_LOC_1BC36_PRE_PANEL_PRESENTATION_DELAY - 1,
            0,
        );
        assert_eq!(
            vram_word(&before_second, 0x019A),
            0x1100,
            "ASM loc_1BC36 should keep frame 0 visible through the oracle-backed presentation delay after the first `$A0` hibernate",
        );

        let second = build_intro_sequence_vdp_with_title_variant_at_elapsed(
            &data,
            4,
            0,
            0xA0 + INTRO_SEQUENCE_5_LOC_1BC36_PRE_PANEL_PRESENTATION_DELAY,
            0,
        );
        assert_eq!(
            vram_word(&second, 0x019A),
            0x1101,
            "ASM loc_1BC36/sub_1C034(d4=1) becomes visible after the same delayed VDP presentation boundary seen at frame 2100",
        );

        let before_hands_centred =
            build_intro_sequence_vdp_with_title_variant_at_elapsed(&data, 4, 0, 0xA0 + 0x90, 0);
        assert_eq!(
            vram_word(&before_hands_centred, 0x019A),
            0x1101,
            "ASM loc_1BC36 has called sub_1C034(d4=2), but cold-boot oracle frame 2220 still presents the preceding lean"
        );

        let before_centred = build_intro_sequence_vdp_with_title_variant_at_elapsed(
            &data,
            4,
            0,
            0xA0 + 0x90 + 0x16,
            0,
        );
        assert_eq!(
            vram_word(&before_centred, 0x019A),
            0x1101,
            "ASM loc_1BC36 has called the later sub_1C034 frames, but oracle frame 2220 keeps the preceding lean visible during the post-panel presentation delay"
        );

        let centred = build_intro_sequence_vdp_with_title_variant_at_elapsed(
            &data,
            4,
            0,
            0xA0 + 0x90 + 0x16 + INTRO_SEQUENCE_5_LOC_1BC36_PRE_PANEL_PRESENTATION_DELAY,
            0,
        );
        assert_eq!(
            vram_word(&centred, 0x0194),
            0x1104,
            "ASM off_1C126 frame 4 has zero left padding, so loc_1C0A4 becomes visible at VRAM `$0194` after the oracle-backed presentation delay"
        );

        let before_standing = build_intro_sequence_vdp_with_title_variant_at_elapsed(
            &data,
            4,
            0,
            0xA0 + 0x90 + 0x21 + INTRO_SEQUENCE_5_LOC_1BC36_POST_PANEL_PRESENTATION_DELAY - 1,
            0,
        );
        assert_eq!(
            vram_word(&before_standing, 0x0194),
            0x1104,
            "ASM loc_1BC36 has already called sub_1C034(d4=5), but cold-boot oracle frame 2220 keeps the visible post-panel pose until the delayed VDP presentation boundary"
        );

        let standing = build_intro_sequence_vdp_with_title_variant_at_elapsed(
            &data,
            4,
            0,
            0xA0 + 0x90 + 0x21 + INTRO_SEQUENCE_5_LOC_1BC36_POST_PANEL_PRESENTATION_DELAY,
            0,
        );
        assert_eq!(
            vram_word(&standing, 0x0198),
            0x1105,
            "ASM loc_1BC36/sub_1C034(d4=5) becomes visible on the oracle-backed frame-2280 boundary"
        );

        let before_cross_arm = build_intro_sequence_vdp_with_title_variant_at_elapsed(
            &data,
            4,
            0,
            0xA0 + 0x90 + 0x21 + INTRO_SEQUENCE_5_LOC_1BC36_POST_PANEL_PRESENTATION_DELAY + 0x37
                - 1,
            0,
        );
        assert_eq!(
            vram_word(&before_cross_arm, 0x0198),
            0x1105,
            "ASM loc_1BC36 keeps sub_1C034(d4=5) visible through the `$37` hibernate before d4=6"
        );

        let cross_arm = build_intro_sequence_vdp_with_title_variant_at_elapsed(
            &data,
            4,
            0,
            0xA0 + 0x90 + 0x21 + INTRO_SEQUENCE_5_LOC_1BC36_POST_PANEL_PRESENTATION_DELAY + 0x37,
            0,
        );
        assert_eq!(
            vram_word(&cross_arm, 0x0198),
            0x1106,
            "ASM loc_1BC36 resumes with sub_1C034(d4=6) only after the `$37` hibernate, matching the frame-2340 cross-arm window"
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
        let frame_2400_clear_artefact_addr = 0x13 * NAMETABLE_ROW_STRIDE_BYTES + 12 * 2;

        let before_panel = build_intro_sequence_vdp_with_title_variant_at_elapsed(
            &data,
            4,
            0,
            INTRO_SEQUENCE_5_LOC_1BC36_PANEL_VISIBLE_ELAPSED - 1,
            0,
        );
        assert_ne!(
            vram_word(&before_panel, panel_byte_addr),
            0xCAFE,
            "ASM loc_1BC36 has not called sub_1C0EE before the d4=4 upload pass"
        );

        let raster_boundary = build_intro_sequence_vdp_with_title_variant_at_elapsed(
            &data,
            4,
            0,
            INTRO_SEQUENCE_5_LOC_1BC36_PANEL_VISIBLE_ELAPSED,
            0,
        );
        assert_eq!(
            vram_word(&raster_boundary, panel_byte_addr),
            0xCAFE,
            "ASM loc_1BC36 sub_1C034(d4=4) -> sub_1C0EE leaves the panel in final VRAM on the crossing frame",
        );
        let raster_frame = raster_boundary
            .raster_frame
            .as_ref()
            .expect("ASM loc_1BC36 crossing frame should retain ordered FIFO writes");
        assert_ne!(
            u16::from_be_bytes([
                raster_frame.initial_vram[panel_byte_addr],
                raster_frame.initial_vram[panel_byte_addr + 1],
            ]),
            0xCAFE,
            "ASM sub_1C0EE begins during active display, so the raster baseline must retain the pre-panel plane"
        );
        assert_eq!(
            raster_frame.vram_word_writes.len(),
            2 * (377 + 100),
            "ASM sub_1C034 streams 377 words and sub_1C0EE/sub_1C5D0 streams 100; jgenesis commits each 68k data-port word on two H40 FIFO slots"
        );

        let panel_tail = build_intro_sequence_vdp_with_title_variant_at_elapsed(
            &data,
            4,
            0,
            INTRO_SEQUENCE_5_LOC_1BC36_PANEL_CLEAR_ELAPSED - 1,
            0,
        );
        assert_eq!(
            vram_word(&panel_tail, panel_byte_addr),
            0xCAFE,
            "ASM loc_1BC36 keeps the sub_1C0EE panel through the complete `$B` hibernate"
        );

        let frame_5 = build_intro_sequence_vdp_with_title_variant_at_elapsed(
            &data,
            4,
            0,
            INTRO_SEQUENCE_5_LOC_1BC36_FRAME_5_VISIBLE_ELAPSED,
            0,
        );
        assert_ne!(
            vram_word(&frame_5, panel_byte_addr),
            0xCAFE,
            "ASM sub_1C112 clears the panel before sub_1C034(d4=5) crosses the next active display"
        );
        assert_eq!(
            vram_word(&frame_5, frame_2400_clear_artefact_addr),
            0x1111,
            "ASM loc_1BC36's later sub_1C034(d4=5) redraw should replace the temporary sub_1C112 clear instead of being overwritten by it",
        );
    }

    #[test]
    fn intro_sequence_5_loc_1bc36_final_frame_5_upload_retains_frame_6_raster_baseline() {
        let data = IntroSub1B850Data {
            intro_sequence_5_frame_tilemaps: (0..8)
                .map(|frame| vec![0x1100 + frame; 12 * 24])
                .collect(),
            ..empty_intro_data()
        };
        let before = build_timeout_restart_intro_sequence_5_vdp_at_elapsed_loc_1ba28(
            &data,
            0,
            INTRO_SEQUENCE_5_LOC_1BC36_FINAL_FRAME_5_RASTER_ELAPSED - 1,
            0,
            true,
        );
        let boundary = build_timeout_restart_intro_sequence_5_vdp_at_elapsed_loc_1ba28(
            &data,
            0,
            INTRO_SEQUENCE_5_LOC_1BC36_FINAL_FRAME_5_RASTER_ELAPSED,
            0,
            true,
        );
        let changed_byte_addr = (0..0x1000)
            .step_by(2)
            .find(|&byte_addr| {
                before.vram[byte_addr..byte_addr + 2] != boundary.vram[byte_addr..byte_addr + 2]
            })
            .expect("ASM loc_1BC36's final d4=5 upload must replace the preceding d4=6 map");
        let raster_frame = boundary
            .raster_frame
            .as_ref()
            .expect("ASM loc_1BC36's final d4=5 upload crosses active display");

        assert_eq!(
            &raster_frame.initial_vram[changed_byte_addr..changed_byte_addr + 2],
            &before.vram[changed_byte_addr..changed_byte_addr + 2],
            "ASM loc_1BC36 must render the preceding d4=6 map until the final d4=5 FIFO writes arrive",
        );
        assert_eq!(
            raster_frame
                .vram_word_writes
                .first()
                .map(|write| (write.scanline, write.pixel)),
            Some((136, 204)),
            "oracle FIFO tracing pins the final loc_1BC36 d4=5 upload entry point",
        );
    }

    #[test]
    fn intro_sequence_5_loc_1bc36_timeout_fc82_two_delays_and_schedules_tail_uploads() {
        let data = IntroSub1B850Data {
            intro_sequence_5_frame_tilemaps: (0..8)
                .map(|frame| vec![0x1100 + frame; 12 * 24])
                .collect(),
            ..empty_intro_data()
        };
        let before = build_timeout_restart_intro_sequence_5_vdp_at_elapsed_loc_1ba28(
            &data,
            2,
            INTRO_SEQUENCE_5_LOC_1BC36_FRAME_6_RASTER_ELAPSED,
            0,
            false,
        );
        let boundary = build_timeout_restart_intro_sequence_5_vdp_at_elapsed_loc_1ba28(
            &data,
            2,
            INTRO_SEQUENCE_5_LOC_1BC36_FRAME_6_RASTER_ELAPSED + 1,
            0,
            false,
        );

        assert!(
            before.raster_frame.is_none(),
            "ASM loc_1BC36's FC82=2 worker retains d4=5 for one outer trace before the d4=6 upload",
        );
        assert_eq!(
            boundary
                .raster_frame
                .as_ref()
                .and_then(|raster| raster.vram_word_writes.first())
                .map(|write| (write.scanline, write.pixel)),
            Some((125, 172)),
            "oracle frame 20825 pins the delayed FC82=2 d4=6 FIFO entry point",
        );

        let expected_tail = [
            (
                INTRO_SEQUENCE_5_LOC_1BC36_FRAME_6_RASTER_ELAPSED,
                5,
                6,
                (125, 172),
            ),
            (
                INTRO_SEQUENCE_5_LOC_1BC36_FRAME_7_RASTER_ELAPSED,
                6,
                7,
                (114, 204),
            ),
            (
                INTRO_SEQUENCE_5_LOC_1BC36_RETURN_FRAME_6_RASTER_ELAPSED,
                7,
                6,
                (125, 172),
            ),
            (
                INTRO_SEQUENCE_5_LOC_1BC36_FINAL_FRAME_5_RASTER_ELAPSED,
                6,
                5,
                (134, 380),
            ),
        ];
        for (elapsed, previous_frame_index, frame_index, frame_start) in expected_tail {
            let upload = intro_sequence_5_raster_upload_loc_1BC36(
                elapsed,
                IntroSequence5WorkerLineage::TimeoutFc82Two,
            )
            .expect("ASM loc_1BC36 FC82=2 tail upload should retain raster evidence");
            assert_eq!(
                (
                    upload.previous_frame_index,
                    upload.frame_index,
                    upload.frame_start,
                ),
                (previous_frame_index, frame_index, frame_start),
            );
        }
    }

    #[test]
    fn intro_sequence_5_sub_1c034_stages_row_major_tilemap_words_before_vram_rows() {
        let frame_index = 5u16;
        let plan = wildside_data::intro_sequence_5_frame_load_plan_sub_1C034(frame_index)
            .expect("ASM sub_1C034 should accept frame 5");
        let width = usize::from(plan.tilemap_load.width_words);
        let height = usize::from(plan.tilemap_load.height_words);
        let left_padding = usize::from(plan.tilemap_load.left_padding_words);
        let copied_width = width.min(16usize.saturating_sub(left_padding));

        let mut frame_words = vec![0u16; width * height];
        for row in 0..height {
            for col in 0..width {
                frame_words[row * width + col] = 0x8000 | ((row as u16) << 5) | col as u16;
            }
        }
        let mut frames = vec![vec![]; 8];
        frames[usize::from(frame_index)] = frame_words;
        let data = IntroSub1B850Data {
            intro_sequence_5_frame_tilemaps: frames,
            ..empty_intro_data()
        };
        let mut vdp = VdpState::new();

        apply_intro_sequence_5_sub_1C034_frame(&mut vdp, &data, frame_index);

        let target_addr_for_staged_offset = |staged_offset: usize| {
            plan.vram_rows.iter().find_map(|row_write| {
                let start = usize::from(row_write.source_word_offset);
                let end = start + usize::from(row_write.word_count);
                (start..end)
                    .contains(&staged_offset)
                    .then(|| usize::from(row_write.vram_word_address) + (staged_offset - start) * 2)
            })
        };

        for &(row, col) in &[
            (0usize, 0usize),
            (1, 0),
            (2, copied_width - 1),
            (height.min(25) - 1, copied_width / 2),
        ] {
            let staged_offset = row * 16 + left_padding + col;
            let addr = target_addr_for_staged_offset(staged_offset)
                .expect("sampled staged word should be streamed to VRAM");
            assert_eq!(
                vram_word(&vdp, addr),
                0x8000 | ((row as u16) << 5) | col as u16,
                "ASM sub_1C034 stages row-major source row {row}, col {col} into the 16-word buffer before streaming VRAM rows",
            );
        }

        if left_padding > 0 {
            let padding_addr = target_addr_for_staged_offset(0)
                .expect("left padding word should be part of the streamed first row");
            assert_eq!(
                vram_word(&vdp, padding_addr),
                0,
                "ASM sub_1C034 leaves the staged left-padding words blank before copying the source tilemap row",
            );
        }
    }

    #[test]
    fn intro_sequence_5_sub_1c0ee_and_sub_1c112_write_exact_plane_rect_cells() {
        let plane_cell_x = 3u16;
        let plane_cell_y = 0x13u16;
        let panel_plan =
            wildside_data::intro_sequence_5_panel_load_plan_sub_1C0EE(plane_cell_x, plane_cell_y);
        let width = usize::from(panel_plan.plane_upload.width_words);
        let height = usize::from(panel_plan.plane_upload.height_words);
        let cell_x = usize::from(panel_plan.plane_upload.plane_cell_x);
        let cell_y = usize::from(panel_plan.plane_upload.plane_cell_y);

        let mut panel_words = vec![0u16; width * height];
        for row in 0..height {
            for col in 0..width {
                panel_words[row * width + col] = 0x9000 | ((row as u16) << 5) | col as u16;
            }
        }
        let data = IntroSub1B850Data {
            intro_sequence_5_panel_tilemap: panel_words,
            ..empty_intro_data()
        };
        let mut vdp = VdpState::new();

        apply_intro_sequence_5_panel_sub_1C0EE(&mut vdp, &data, plane_cell_x, plane_cell_y);

        for &(row, col) in &[
            (0usize, 0usize),
            (0, width - 1),
            (1, 0),
            (height - 1, width / 2),
            (height - 1, width - 1),
        ] {
            let addr = (cell_y + row) * NAMETABLE_ROW_STRIDE_BYTES + (cell_x + col) * 2;
            assert_eq!(
                vram_word(&vdp, addr),
                0x9000 | ((row as u16) << 5) | col as u16,
                "ASM sub_1C0EE writes panel row {row}, col {col} to its exact Scroll A plane cell",
            );
        }

        let before_addr = cell_y * NAMETABLE_ROW_STRIDE_BYTES + (cell_x - 1) * 2;
        vdp.write_vram_word(before_addr, 0xA55A);

        apply_intro_sequence_5_blank_panel_sub_1C112(&mut vdp, plane_cell_x, plane_cell_y);

        for &(row, col) in &[(0usize, 0usize), (height - 1, width - 1)] {
            let addr = (cell_y + row) * NAMETABLE_ROW_STRIDE_BYTES + (cell_x + col) * 2;
            assert_eq!(
                vram_word(&vdp, addr),
                0,
                "ASM sub_1C112 blanks the same panel rectangle written by sub_1C0EE",
            );
        }
        assert_eq!(
            vram_word(&vdp, before_addr),
            0xA55A,
            "ASM sub_1C112 only clears the requested panel rectangle and leaves the neighbouring plane cell untouched",
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
            art_chunks: [(0, vec![]), (0x21E0, vec![0x11]), (0, vec![]), (0, vec![])],
            transformation_art_variants: [vec![0xA0], vec![0xB0], vec![0xC0]],
            title_art_chunks: [(0, vec![]), (0, vec![]), (0, vec![]), (0, vec![])],
            plane_a_tilemap: tilemap,
            title_text_chameleon_rows: [[0; 64]; 28],
            title_text_chameleon_anim_maps: [vec![], vec![], vec![]],
            title_menu_tilemap_words: vec![],
            character_frames: Vec::new(),
            lightning_art_bytes: vec![],
            tile_patch_words: [0; 32],
            palette_words: vec![],
            palette_variant_overlays: [vec![], vec![]],
        };
        let vdp = build_title_screen_vdp_with_character_frame(&data, 0, 0);
        assert_eq!(
            vdp.vram[0x21E0], 0xA0,
            "ASM sub_1B2A4 loc_1B2D8 keeps byte_26E3D Red Stealth transformation art when `$FFFFFC82=0`",
        );
        let maniaxe = build_title_screen_vdp_with_character_frame(&data, 0, 1);
        assert_eq!(maniaxe.vram[0x21E0], 0xB0);
        let juggernaut = build_title_screen_vdp_with_character_frame(&data, 0, 2);
        assert_eq!(
            juggernaut.vram[0x21E0], 0xC0,
            "ASM loc_1B2D8 selects byte_282C1 Juggernaut transformation art when `$FFFFFC82=2`",
        );

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
            let [bottom_left_address, bottom_right_address] = entry.bottom_word_addresses();
            let bottom_left_index =
                usize::from(bottom_left_address.wrapping_sub(DECOMPRESSION_BUFFER_LOW_ADDR)) / 2;
            let bottom_right_index =
                usize::from(bottom_right_address.wrapping_sub(DECOMPRESSION_BUFFER_LOW_ADDR)) / 2;
            expected_writes.insert(bottom_left_index, 0x21AD);
            expected_writes.insert(bottom_right_index, 0x21AE);
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
            transformation_art_variants: [vec![], vec![], vec![]],
            title_art_chunks: [(0, vec![]), (0, vec![]), (0, vec![]), (0, vec![])],
            plane_a_tilemap: vec![],
            title_text_chameleon_rows: [[0; 64]; 28],
            title_text_chameleon_anim_maps: [vec![], vec![], vec![]],
            title_menu_tilemap_words: vec![],
            character_frames: Vec::new(),
            lightning_art_bytes: vec![],
            tile_patch_words: patch,
            palette_words: vec![],
            palette_variant_overlays: [vec![], vec![]],
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
            transformation_art_variants: [vec![], vec![], vec![]],
            title_art_chunks: [
                (0x6760, vec![0x77, 0x88]),
                (0x9A60, vec![0x11, 0x22]),
                (0xA360, vec![0x33, 0x44]),
                (0xA980, vec![0x55, 0x66]),
            ],
            plane_a_tilemap: vec![],
            title_text_chameleon_rows: [[0; 64]; 28],
            title_text_chameleon_anim_maps: [vec![], vec![], vec![]],
            title_menu_tilemap_words: vec![],
            character_frames: Vec::new(),
            lightning_art_bytes: vec![],
            tile_patch_words: [0; 32],
            palette_words: vec![],
            palette_variant_overlays: [vec![], vec![]],
        };

        let vdp = build_title_screen_vdp_with_character_frame(&data, 0, 0);

        assert_eq!(
            &vdp.vram[0x6760..0x6762],
            &[0x77, 0x88],
            "ASM Load_TitleArt writes byte_22080 / Title_text_CHAMELEON to VRAM $6760 through DecompressToRAM"
        );
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
    fn title_text_chameleon_art_loc_1b4f4_remaps_e_nibbles_to_transparent_zero() {
        assert_eq!(
            title_text_chameleon_art_loc_1B4F4(&[0xEF, 0xFE, 0xDE, 0x12]),
            vec![0x0F, 0xF0, 0xD0, 0x12],
            "ASM Load_TitleArt seeds `DecompressToRAM` with unk_1B4F4, whose only non-identity visible change maps nibble `$E` to `$0` before writing Title_text_CHAMELEON to VRAM `$6760`"
        );
    }

    #[test]
    #[allow(non_snake_case)]
    fn title_screen_vdp_obj_title_menu_loc_1bb52_writes_menu_and_cursor() {
        let mut menu_words = vec![0u16; 13 * 5];
        for row in 0..5u16 {
            for col in 0..13u16 {
                menu_words[(row as usize) * 13 + (col as usize)] = 0x4500 | (row << 4) | col;
            }
        }
        let data = TitleScreenData {
            art_chunks: [(0, vec![]), (0, vec![]), (0, vec![]), (0, vec![])],
            transformation_art_variants: [vec![], vec![], vec![]],
            title_art_chunks: [(0, vec![]), (0, vec![]), (0, vec![]), (0, vec![])],
            plane_a_tilemap: vec![],
            title_text_chameleon_rows: [[0; 64]; 28],
            title_text_chameleon_anim_maps: [vec![], vec![], vec![]],
            title_menu_tilemap_words: menu_words,
            character_frames: Vec::new(),
            lightning_art_bytes: vec![],
            tile_patch_words: [0; 32],
            palette_words: vec![],
            palette_variant_overlays: [vec![], vec![]],
        };

        let vdp = build_title_screen_vdp_with_character_frame_and_menu(&data, 0, 0, Some(2));

        for &(row, col) in &[(0u16, 0u16), (0, 12), (4, 0), (4, 12), (3, 7)] {
            let addr = (4 + row as usize) * 128 + (4 + col as usize) * 2;
            let word = u16::from_be_bytes([vdp.vram[addr], vdp.vram[addr + 1]]);
            assert_eq!(
                word,
                0x4500 | (row << 4) | col,
                "ASM Obj_TitleMenu loc_1BB52 writes menu cell row {row}, col {col}"
            );
        }
        for &(row, col) in &[(0u16, 13u16), (5, 0)] {
            let addr = (4 + row as usize) * 128 + (4 + col as usize) * 2;
            let word = u16::from_be_bytes([vdp.vram[addr], vdp.vram[addr + 1]]);
            assert_eq!(
                word, 0x20BC,
                "ASM sub_1C5D0 pre-decrements d6/d7 before DBF, so menu row {row}, col {col} must stay on the title fill tile"
            );
        }

        assert_eq!(
            sprite_entry_words(&vdp, 0),
            (0x00BC, 0x00, 0, 0x454B, 0x00A8),
            "ASM Obj_TitleMenu loc_1BB52/loc_1BC0E should expose cursor 2 as unk_E1196 at x=$28,y=$44"
        );
    }

    #[test]
    #[allow(non_snake_case)]
    fn title_menu_cursor_loc_1bb52_uses_all_three_rom_rows() {
        let mut entries = Vec::new();

        append_title_menu_cursor_sprite_loc_1bb52(&mut entries, 0);
        assert_eq!(
            entries[0],
            HardwareSpriteEntry {
                y_pos: 0x00A8,
                size_byte: 0x00,
                pattern: 0x454B,
                x_pos: 0x00A8,
            },
            "ASM Obj_TitleMenu loc_1BB52 reads word_1BC20 row 0 for the Start Game cursor"
        );

        entries.clear();
        append_title_menu_cursor_sprite_loc_1bb52(&mut entries, 1);
        assert_eq!(
            entries[0],
            HardwareSpriteEntry {
                y_pos: 0x00B2,
                size_byte: 0x00,
                pattern: 0x454B,
                x_pos: 0x00A8,
            },
            "ASM Obj_TitleMenu loc_1BB52 reads word_1BC20 row 1 for the Options cursor"
        );
    }

    #[test]
    fn title_menu_tilemap_loc_1bb52_pads_short_decode_with_menu_blank() {
        let menu_words = (0x4500u16..0x4500 + 60).collect::<Vec<_>>();

        let composed = title_menu_tilemap_buffer_loc_1bb52(&menu_words);

        assert_eq!(composed.len(), 13 * 5);
        assert_eq!(
            &composed[..60],
            menu_words.as_slice(),
            "ASM loc_1BB52 should use the freshly decoded `unk_2EAAE` words first"
        );
        assert_eq!(
            &composed[60..],
            &[0x4528; 5],
            "the stateless title renderer should preserve the cached menu rectangle by padding the short extracted loc_1BB52 decode with the menu blank tile"
        );
    }

    #[test]
    fn title_screen_vdp_obj_title_text_chameleon_sub_1b71a_writes_scroll_a_reveal_rows() {
        let decoded_words = (0u16..(7 * 29) as u16).collect::<Vec<_>>();
        let data = TitleScreenData {
            art_chunks: [(0, vec![]), (0, vec![]), (0, vec![]), (0, vec![])],
            transformation_art_variants: [vec![], vec![], vec![]],
            title_art_chunks: [(0, vec![]), (0, vec![]), (0, vec![]), (0, vec![])],
            plane_a_tilemap: vec![],
            title_text_chameleon_rows: [[0; 64]; 28],
            title_text_chameleon_anim_maps: [decoded_words, vec![], vec![]],
            title_menu_tilemap_words: vec![],
            character_frames: Vec::new(),
            lightning_art_bytes: vec![],
            tile_patch_words: [0; 32],
            palette_words: vec![],
            palette_variant_overlays: [vec![], vec![]],
        };

        let vdp = build_title_screen_vdp_with_character_frame_menu_and_title_objects(
            &data, 0, 0, None, true, 0,
        );

        assert_eq!(
            vram_word(&vdp, 0x0908),
            0x00BC,
            "ASM sub_1B71A loc_1B780 writes the `$BC` filler one cell before the early Chameleon reveal"
        );
        assert_eq!(
            vram_word(&vdp, 0x090A),
            0,
            "ASM sub_1B71A loc_1B758 starts the first reveal row at row `$12`, column 5"
        );
        assert_eq!(
            vram_word(&vdp, 0x0932),
            20,
            "ASM sub_1B71A caps the first copied row at 21 words"
        );
        assert_eq!(
            vram_word(&vdp, 0x098A),
            29,
            "ASM sub_1B79E advances the next reveal row by one 64-cell nametable row"
        );
    }

    #[test]
    fn title_screen_vdp_loc_1b6dc_replays_sega_text_after_chameleon_reveal() {
        let data = TitleScreenData {
            art_chunks: [(0, vec![]), (0, vec![]), (0, vec![]), (0, vec![])],
            transformation_art_variants: [vec![], vec![], vec![]],
            title_art_chunks: [(0, vec![]), (0, vec![]), (0, vec![]), (0, vec![])],
            plane_a_tilemap: vec![],
            title_text_chameleon_rows: [[0; 64]; 28],
            title_text_chameleon_anim_maps: [vec![0; 7 * 29], vec![], vec![]],
            title_menu_tilemap_words: vec![],
            character_frames: Vec::new(),
            lightning_art_bytes: vec![],
            tile_patch_words: [0; 32],
            palette_words: vec![],
            palette_variant_overlays: [vec![], vec![]],
        };

        let early = build_title_screen_vdp_with_character_frame_menu_and_title_objects(
            &data, 0, 0, None, true, 101,
        );
        let copyright_write =
            wildside_data::title_intro_glyph_write_sub_1C7F6(0, 0x67D3, 0x19, 0x19, 0x2A);
        assert_eq!(
            vram_word(&early, usize::from(copyright_write.vram_word_address)),
            0x20BC,
            "ASM loc_1C7A0 should still spend its initial one-frame hibernate before SegaText writes the copyright glyph"
        );

        let visible = build_title_screen_vdp_with_character_frame_menu_and_title_objects(
            &data, 0, 0, None, true, 102,
        );
        assert_eq!(
            vram_word(&visible, usize::from(copyright_write.vram_word_address)),
            0x67FD,
            "ASM SegaText starts with byte `$6B`, which sub_1C7F6 renders as base `$67D3` plus glyph `$2A`"
        );
        let sega_s_write =
            wildside_data::title_intro_glyph_write_sub_1C7F6(0, 0x67D3, 0x20, 0x19, 0x12);
        assert_eq!(
            vram_word(&visible, usize::from(sega_s_write.vram_word_address)),
            0x67E5,
            "ASM SegaText writes the `S` in `SEGA` after the copyright and year glyphs"
        );
    }

    #[test]
    fn manual_demo_return_runs_sega_text_after_first_title_object_pass() {
        let data = TitleScreenData {
            art_chunks: [(0, vec![]), (0, vec![]), (0, vec![]), (0, vec![])],
            transformation_art_variants: [vec![], vec![], vec![]],
            title_art_chunks: [(0, vec![]), (0, vec![]), (0, vec![]), (0, vec![])],
            plane_a_tilemap: vec![],
            title_text_chameleon_rows: [[0; 64]; 28],
            title_text_chameleon_anim_maps: [
                vec![0x0100; 7 * 29],
                vec![0x0200; 7 * 29],
                vec![0x0300; 7 * 29],
            ],
            title_menu_tilemap_words: vec![],
            character_frames: Vec::new(),
            lightning_art_bytes: vec![],
            tile_patch_words: [0; 32],
            palette_words: vec![],
            palette_variant_overlays: [vec![], vec![]],
        };
        let copyright_write =
            wildside_data::title_intro_glyph_write_sub_1C7F6(0, 0x67D3, 0x19, 0x19, 0x2A);
        let lineage = TitleScreenVdpLineage {
            fresh_title_worker_lineage: true,
            ..TitleScreenVdpLineage::default()
        };

        let first_object_pass =
            build_title_screen_vdp_with_character_frame_menu_title_objects_and_lineage(
                &data, 7, 0, None, true, 58, lineage,
            );
        assert_eq!(
            vram_word(
                &first_object_pass,
                usize::from(copyright_write.vram_word_address)
            ),
            0x20BC,
            "ASM Load_TitleScreen's first Mode_Standard pass exposes the title objects before the new loc_1C7A0 child has run",
        );
        assert_eq!(
            vram_word(&first_object_pass, 0x090A),
            0x0100,
            "ASM Obj_TitleTextChameleon starts with d3=0, so its first sub_1B71A pass selects unk_2E706 instead of inheriting the global title elapsed phase",
        );
        assert!(
            sprite_table_index_without_link(&first_object_pass, (0x0114, 0x04, 0x4780, 0x018C))
                .is_none(),
            "ASM Obj_TitleMenu has not linked its loc_1BB10 child on the first post-Load_TitleScreen object pass",
        );

        let second_object_pass =
            build_title_screen_vdp_with_character_frame_menu_title_objects_and_lineage(
                &data, 7, 0, None, true, 59, lineage,
            );
        assert_eq!(
            vram_word(
                &second_object_pass,
                usize::from(copyright_write.vram_word_address)
            ),
            0x67FD,
            "ASM loc_1C7A0 writes the zero-delay SegaText string on the second post-loader object pass",
        );
        assert!(
            sprite_table_index_without_link(&second_object_pass, (0x0114, 0x04, 0x4780, 0x018C))
                .is_some(),
            "ASM Obj_TitleMenu links loc_1BB10 on the second post-loader object pass",
        );
    }

    #[test]
    fn fresh_title_workers_replay_sub_1b652_bursts_at_pointer_quirk_positions() {
        let mut entries = Vec::new();
        append_title_text_burst_sprites_sub_1b652(&mut entries, 150);
        assert!(
            entries.is_empty(),
            "ASM sub_1B652 allocates after the elapsed-150 GfxObject pass"
        );

        append_title_text_burst_sprites_sub_1b652(&mut entries, 151);
        assert_eq!(
            entries,
            vec![HardwareSpriteEntry {
                y_pos: 0x00F4,
                size_byte: 0x06,
                pattern: 0xC558,
                x_pos: 0x00AE,
            }],
            "ASM sub_1B652 -> Init_Animation exposes unk_E11B6 one pass later at the first child position written through a3",
        );

        entries.clear();
        append_title_text_burst_sprites_sub_1b652(&mut entries, 155);
        assert_eq!(
            entries[0].pattern, 0xC55E,
            "ASM stru_1B68A advances from unk_E11B6 to unk_E11BE after four frames",
        );

        entries.clear();
        append_title_text_burst_sprites_sub_1b652(&mut entries, 159);
        assert!(
            entries.is_empty(),
            "ASM stru_1B68A hides the first burst for its 100-frame null mapping"
        );

        append_title_text_burst_sprites_sub_1b652(&mut entries, 161);
        assert_eq!(
            entries,
            vec![HardwareSpriteEntry {
                y_pos: 0x0100,
                size_byte: 0x06,
                pattern: 0xC558,
                x_pos: 0x00DA,
            }],
            "ASM loc_1B610 allocates the second child ten frames later and writes ($5A,$98) through the retained a3 pointer",
        );

        entries.clear();
        append_title_text_burst_sprites_sub_1b652(&mut entries, 259);
        assert_eq!(
            entries[0].pattern, 0xC54C,
            "ASM stru_1B68A reaches unk_E11A6 after the 100-frame null mapping",
        );

        entries.clear();
        append_title_text_burst_sprites_sub_1b652(&mut entries, 267);
        assert_eq!(
            entries[0].pattern, 0xC558,
            "ASM command 2 rewinds the full 116-frame stru_1B68A cycle",
        );
    }

    #[test]
    fn title_screen_vdp_intro_return_replays_sega_text_when_vdp_becomes_visible() {
        let data = TitleScreenData {
            art_chunks: [(0, vec![]), (0, vec![]), (0, vec![]), (0, vec![])],
            transformation_art_variants: [vec![], vec![], vec![]],
            title_art_chunks: [(0, vec![]), (0, vec![]), (0, vec![]), (0, vec![])],
            plane_a_tilemap: vec![],
            title_text_chameleon_rows: [[0; 64]; 28],
            title_text_chameleon_anim_maps: [vec![0; 7 * 29], vec![], vec![]],
            title_menu_tilemap_words: vec![],
            character_frames: Vec::new(),
            lightning_art_bytes: vec![],
            tile_patch_words: [0; 32],
            palette_words: vec![],
            palette_variant_overlays: [vec![], vec![]],
        };
        let copyright_write =
            wildside_data::title_intro_glyph_write_sub_1C7F6(0, 0x67D3, 0x19, 0x19, 0x2A);
        let visible_elapsed = INTRO_SEQUENCE_6_START_SKIP_TITLE_VDP_READY_FRAMES + 1;

        let hidden = build_title_screen_vdp_with_character_frame_menu_title_objects_and_lineage(
            &data,
            0,
            0,
            None,
            true,
            INTRO_SEQUENCE_6_START_SKIP_TITLE_VDP_READY_FRAMES,
            TitleScreenVdpLineage {
                intro_return_hidden_shell: true,
                direct_intro_sequence_5_title_return: false,
                natural_intro_sequence_6_pre_ready_shell: false,
                natural_intro_sequence_6_worker_elapsed: None,
                fresh_title_worker_lineage: false,
                title_menu_raster_upload: false,
            },
        );
        assert_eq!(
            vram_word(&hidden, usize::from(copyright_write.vram_word_address)),
            0x20BC,
            "ASM loc_1C7A0 still spends its one-frame hibernate on the intro-return title path"
        );

        let visible = build_title_screen_vdp_with_character_frame_menu_title_objects_and_lineage(
            &data,
            0,
            0,
            None,
            true,
            visible_elapsed,
            TitleScreenVdpLineage {
                intro_return_hidden_shell: true,
                direct_intro_sequence_5_title_return: false,
                natural_intro_sequence_6_pre_ready_shell: false,
                natural_intro_sequence_6_worker_elapsed: None,
                fresh_title_worker_lineage: false,
                title_menu_raster_upload: false,
            },
        );
        assert_eq!(
            vram_word(&visible, usize::from(copyright_write.vram_word_address)),
            0x67FD,
            "ASM sub_1C246 -> Load_TitleScreen on the cold IntroSequence6 Start-skip path inherits the hidden loc_1B6AC/loc_1B6DC cadence, so SegaText is visible with the frame-2700 title VDP"
        );

        let direct_hidden =
            build_title_screen_vdp_with_character_frame_menu_title_objects_and_lineage(
                &data,
                0,
                0,
                None,
                true,
                TITLE_ATTRACT_LOCK_FRAMES,
                TitleScreenVdpLineage {
                    intro_return_hidden_shell: true,
                    direct_intro_sequence_5_title_return: true,
                    natural_intro_sequence_6_pre_ready_shell: false,
                    natural_intro_sequence_6_worker_elapsed: None,
                    fresh_title_worker_lineage: false,
                    title_menu_raster_upload: false,
                },
            );
        assert_eq!(
            vram_word(
                &direct_hidden,
                usize::from(copyright_write.vram_word_address)
            ),
            0x20BC,
            "ASM loc_1C7A0 still hibernates for the first direct IntroSequence5 title-return VDP frame",
        );

        let direct_visible =
            build_title_screen_vdp_with_character_frame_menu_title_objects_and_lineage(
                &data,
                0,
                0,
                None,
                true,
                TITLE_ATTRACT_LOCK_FRAMES + 1,
                TitleScreenVdpLineage {
                    intro_return_hidden_shell: true,
                    direct_intro_sequence_5_title_return: true,
                    natural_intro_sequence_6_pre_ready_shell: false,
                    natural_intro_sequence_6_worker_elapsed: None,
                    fresh_title_worker_lineage: false,
                    title_menu_raster_upload: false,
                },
            );
        assert_eq!(
            vram_word(
                &direct_visible,
                usize::from(copyright_write.vram_word_address)
            ),
            0x67FD,
            "ASM loc_1BA60 -> sub_1C246 -> Load_TitleScreen direct return shows SegaText one frame after the visible title VDP appears",
        );
    }

    #[test]
    fn title_screen_vdp_intro_return_chameleon_anim_inherits_loc_1b6dc_phase() {
        let map0 = vec![0x1000; 7 * 29];
        let map1 = vec![0x2000; 7 * 29];
        let map2 = vec![0x3000; 7 * 29];
        let data = TitleScreenData {
            art_chunks: [(0, vec![]), (0, vec![]), (0, vec![]), (0, vec![])],
            transformation_art_variants: [vec![], vec![], vec![]],
            title_art_chunks: [(0, vec![]), (0, vec![]), (0, vec![]), (0, vec![])],
            plane_a_tilemap: vec![],
            title_text_chameleon_rows: [[0; 64]; 28],
            title_text_chameleon_anim_maps: [map0, map1, map2],
            title_menu_tilemap_words: vec![],
            character_frames: Vec::new(),
            lightning_art_bytes: vec![],
            tile_patch_words: [0; 32],
            palette_words: vec![],
            palette_variant_overlays: [vec![], vec![]],
        };

        let visible = build_title_screen_vdp_with_character_frame_menu_title_objects_and_lineage(
            &data,
            0,
            0,
            None,
            true,
            INTRO_SEQUENCE_6_START_SKIP_TITLE_VDP_READY_FRAMES + 1,
            TitleScreenVdpLineage {
                intro_return_hidden_shell: true,
                direct_intro_sequence_5_title_return: false,
                natural_intro_sequence_6_pre_ready_shell: false,
                natural_intro_sequence_6_worker_elapsed: None,
                fresh_title_worker_lineage: false,
                title_menu_raster_upload: false,
            },
        );

        assert_eq!(
            vram_word(&visible, 0x090A),
            0x3000,
            "ASM loc_1B6AC/loc_1B6DC keeps sub_1B71A's preserved `d3` running before the Start-skip title VDP becomes visible, so frame 2700 uses the next four-frame Chameleon map phase"
        );

        let tail = build_title_screen_vdp_with_character_frame_menu_title_objects_and_lineage(
            &data,
            0,
            0,
            Some(0),
            true,
            666,
            TitleScreenVdpLineage {
                intro_return_hidden_shell: true,
                direct_intro_sequence_5_title_return: false,
                natural_intro_sequence_6_pre_ready_shell: false,
                natural_intro_sequence_6_worker_elapsed: None,
                fresh_title_worker_lineage: false,
                title_menu_raster_upload: false,
            },
        );

        assert_eq!(
            vram_word(&tail, 0x090A),
            0x1000,
            "ASM loc_1BAB0's title-attract pre-roll lets the inherited loc_1B6DC worker reach one additional sub_1B71A tick by oracle frame 3299"
        );

        let direct_intro_sequence_5_return =
            build_title_screen_vdp_with_character_frame_menu_title_objects_and_lineage(
                &data,
                7,
                0,
                None,
                true,
                crate::constants::TITLE_ATTRACT_LOCK_FRAMES + 2,
                TitleScreenVdpLineage {
                    intro_return_hidden_shell: true,
                    direct_intro_sequence_5_title_return: true,
                    natural_intro_sequence_6_pre_ready_shell: false,
                    natural_intro_sequence_6_worker_elapsed: None,
                    fresh_title_worker_lineage: false,
                    title_menu_raster_upload: false,
                },
            );

        assert_eq!(
            vram_word(&direct_intro_sequence_5_return, 0x090A),
            0x1000,
            "ASM loc_1BA60 -> sub_1C246 -> Load_TitleScreen allocates fresh Obj_TitleTextChameleon; oracle pixels show direct IntroSequence5 title return presents that worker seven sub_1B71A ticks ahead"
        );
    }

    #[test]
    #[allow(non_snake_case)]
    fn title_screen_vdp_direct_return_only_suppresses_loc_1bb10_on_first_object_frame() {
        let data = TitleScreenData {
            art_chunks: [(0, vec![]), (0, vec![]), (0, vec![]), (0, vec![])],
            transformation_art_variants: [vec![], vec![], vec![]],
            title_art_chunks: [(0, vec![]), (0, vec![]), (0, vec![]), (0, vec![])],
            plane_a_tilemap: vec![],
            title_text_chameleon_rows: [[0; 64]; 28],
            title_text_chameleon_anim_maps: [vec![0; 7 * 29], vec![], vec![]],
            title_menu_tilemap_words: vec![],
            character_frames: Vec::new(),
            lightning_art_bytes: vec![],
            tile_patch_words: [0; 32],
            palette_words: vec![],
            palette_variant_overlays: [vec![], vec![]],
        };

        let direct_first =
            build_title_screen_vdp_with_character_frame_menu_title_objects_and_lineage(
                &data,
                0,
                0,
                None,
                true,
                TITLE_ATTRACT_LOCK_FRAMES,
                TitleScreenVdpLineage {
                    intro_return_hidden_shell: true,
                    direct_intro_sequence_5_title_return: true,
                    natural_intro_sequence_6_pre_ready_shell: false,
                    natural_intro_sequence_6_worker_elapsed: None,
                    fresh_title_worker_lineage: false,
                    title_menu_raster_upload: false,
                },
            );
        assert!(
            sprite_table_index_without_link(&direct_first, (0x00E8, 0x0F, 0x44D3, 0x00AE))
                .is_some(),
            "ASM Obj_TitleTextKid loc_1B5E4 still draws the KID title sprite on the first direct IntroSequence5 title-return object frame",
        );
        assert!(
            sprite_table_index_without_link(&direct_first, (0x0114, 0x04, 0x4780, 0x018C))
                .is_none(),
            "ASM Obj_TitleMenu reaches loc_1BB10 one frame after the direct IntroSequence5 title-return handoff, not on the first object frame",
        );

        let direct_second =
            build_title_screen_vdp_with_character_frame_menu_title_objects_and_lineage(
                &data,
                0,
                0,
                None,
                true,
                TITLE_ATTRACT_LOCK_FRAMES + 1,
                TitleScreenVdpLineage {
                    intro_return_hidden_shell: true,
                    direct_intro_sequence_5_title_return: true,
                    natural_intro_sequence_6_pre_ready_shell: false,
                    natural_intro_sequence_6_worker_elapsed: None,
                    fresh_title_worker_lineage: false,
                    title_menu_raster_upload: false,
                },
            );
        assert!(
            sprite_table_index_without_link(&direct_second, (0x0114, 0x04, 0x4780, 0x018C))
                .is_some(),
            "ASM loc_1BA60 -> Load_TitleScreen links loc_1BB10 on the frame after the first direct-return object frame",
        );

        let normal_at_same_elapsed =
            build_title_screen_vdp_with_character_frame_menu_title_objects_and_lineage(
                &data,
                0,
                0,
                None,
                true,
                TITLE_ATTRACT_LOCK_FRAMES,
                TitleScreenVdpLineage {
                    intro_return_hidden_shell: true,
                    direct_intro_sequence_5_title_return: false,
                    natural_intro_sequence_6_pre_ready_shell: false,
                    natural_intro_sequence_6_worker_elapsed: None,
                    fresh_title_worker_lineage: false,
                    title_menu_raster_upload: false,
                },
            );
        assert!(
            sprite_table_index_without_link(
                &normal_at_same_elapsed,
                (0x0114, 0x04, 0x4780, 0x018C)
            )
            .is_some(),
            "the one-frame loc_1BB10 suppression is specific to the direct IntroSequence5 title-return path",
        );
    }

    #[test]
    fn title_screen_vdp_loc_1bb10_keeps_intro_text_sprite_visible_on_menu() {
        let data = TitleScreenData {
            art_chunks: [(0, vec![]), (0, vec![]), (0, vec![]), (0, vec![])],
            transformation_art_variants: [vec![], vec![], vec![]],
            title_art_chunks: [(0, vec![]), (0, vec![]), (0, vec![]), (0, vec![])],
            plane_a_tilemap: vec![],
            title_text_chameleon_rows: [[0; 64]; 28],
            title_text_chameleon_anim_maps: [vec![0; 7 * 29], vec![], vec![]],
            title_menu_tilemap_words: vec![],
            character_frames: Vec::new(),
            lightning_art_bytes: vec![],
            tile_patch_words: [0; 32],
            palette_words: vec![],
            palette_variant_overlays: [vec![], vec![]],
        };

        let vdp = build_title_screen_vdp_with_character_frame_menu_and_title_objects(
            &data,
            0,
            0,
            Some(0),
            true,
            0x0027,
        );

        assert!(
            sprite_table_index_without_link(&vdp, (0x0114, 0x04, 0x4780, 0x018C)).is_some(),
            "ASM loc_1BB10/unk_E0ED6 should keep the settled x=$10C,y=$94 title text sprite visible after Obj_TitleMenu reveals the menu; sprite_frame_vram's y_center `$08` makes the hardware Y word `$114`"
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
            transformation_art_variants: [vec![], vec![], vec![]],
            title_art_chunks: [(0, vec![]), (0, vec![]), (0, vec![]), (0, vec![])],
            plane_a_tilemap: vec![],
            title_text_chameleon_rows: rows,
            title_text_chameleon_anim_maps: [vec![], vec![], vec![]],
            title_menu_tilemap_words: vec![],
            character_frames: Vec::new(),
            lightning_art_bytes: vec![],
            tile_patch_words: [0; 32],
            palette_words: vec![],
            palette_variant_overlays: [vec![], vec![]],
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
            transformation_art_variants: [vec![], vec![], vec![]],
            title_art_chunks: [(0, vec![]), (0, vec![]), (0, vec![]), (0, vec![])],
            plane_a_tilemap: vec![],
            title_text_chameleon_rows: [[0; 64]; 28],
            title_text_chameleon_anim_maps: [vec![], vec![], vec![]],
            title_menu_tilemap_words: vec![],
            character_frames: vec![TitleScreenCharacterFrameData {
                frame_index: 7,
                region_byte: 2,
                width_words: 16,
                tilemap_words: frame_words,
            }],
            lightning_art_bytes: vec![],
            tile_patch_words: [0; 32],
            palette_words: (0x1000..0x1033).collect(),
            palette_variant_overlays: [(0x2000..0x2013).collect(), (0x3000..0x3013).collect()],
        };

        let vdp = build_title_screen_vdp_with_character_frame(&data, 7, 2);

        assert_eq!(
            u16::from_be_bytes([vdp.cram[0x1A], vdp.cram[0x1B]]),
            0x3000,
            "ASM sub_1B2A4 -> loc_1C4E2 overlays Pal_1C48C at Palette_Buffer+$1A for `$FFFFFC82=2`",
        );
        assert_eq!(
            u16::from_be_bytes([vdp.cram[0x3E], vdp.cram[0x3F]]),
            0x3012,
            "ASM loc_1C4E2 copies the full inclusive `$12` DBF range of 19 Juggernaut palette words",
        );

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
        let data = TitleCardData {
            themes,
            title_letter_palette_words: vec![0; 16],
        };

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
        assert_eq!(
            word_at((27 * 64 + 63) * 2),
            0x02FB,
            "ASM loc_19C04 advances through every bottom-padding word until the final title-card nametable cell"
        );
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
        let data = TitleCardData {
            themes,
            title_letter_palette_words: (0x0100..0x0110).collect(),
        };

        let vdp = build_title_card_vdp(&data, 1).expect("sky title-card VDP");

        assert_eq!(&vdp.vram[0x5F60..0x5F64], &[0xAA, 0xBB, 0xCC, 0xDD]);
        assert_eq!(u16::from_be_bytes([vdp.cram[2], vdp.cram[3]]), 0x0222);
        assert_eq!(
            u16::from_be_bytes([vdp.cram[0x20], vdp.cram[0x21]]),
            0x0100,
            "ASM loc_1AB40 copies the title-card letter palette to Palette_Buffer+$20",
        );
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
        for &(addr, expected, label) in &[
            (0xE000, 0x0094, "loc_19B58 first tilemap clear word"),
            (0xE002, 0x0094, "loc_19B58 second tilemap clear word"),
            (
                0xE000 + (0x0800 * 2 - 2),
                0x0094,
                "loc_19B58 final tilemap clear word",
            ),
        ] {
            assert_eq!(
                u16::from_be_bytes([vdp.vram[addr], vdp.vram[addr + 1]]),
                expected,
                "ASM {label} should land at VRAM byte ${addr:04X}",
            );
        }
    }

    #[test]
    fn load_palette_words_decodes_big_endian_pairs_through_source() {
        // ASM Palette_Buffer / Pal_1C28A / Pal_1C2F0 / Pal_1C346 (kid.asm:31232,
        // 31247, 31262) all hold raw 9-bit BGR palette words written through
        // `move.w (a0)+,(a1)+` — the layout extracted by the asset pipeline is
        // therefore big-endian u16 word pairs. load_palette_words mirrors that
        // with `chunks_exact(2).map(u16::from_be_bytes)`.
        //
        // Driving the loader with a known byte sequence (and asserting on both
        // length AND value) pins all three Result<Vec<u16>>-replacement mutants
        // cargo-mutants generates for the wrapper: `Ok(vec![])` shrinks the
        // length, `Ok(vec![0])` and `Ok(vec![1])` change both the length and
        // the first word's value.
        let rel = std::path::Path::new("palette/fixture.bin");
        let mut source = wildside_data::InMemoryAssetSource::new();
        source.insert(rel.to_path_buf(), vec![0x0E, 0xEE, 0x04, 0x44, 0x00, 0x00]);

        let words = load_palette_words(&source, rel).expect("load palette words");

        assert_eq!(words.len(), 3, "three big-endian word pairs from 6 bytes");
        assert_eq!(words[0], 0x0EEE, "first word is BE pair of bytes 0..2");
        assert_eq!(words[1], 0x0444, "second word is BE pair of bytes 2..4");
        assert_eq!(words[2], 0x0000, "third word is BE pair of bytes 4..6");
    }

    #[test]
    fn load_saxman_art_decompresses_known_fixture_through_source() {
        // ASM sub_1B850 (kid.asm:30172) and Load_IntroSequence{2,4,5} all upload
        // Saxman-compressed art via `jsr (j_SaxDec)` after seeding `a0` with the
        // ROM offset. load_saxman_art is the I/O wrapper around that decode and
        // must return the actual decoded bytes — `Ok(vec![])`, `Ok(vec![0])`,
        // and `Ok(vec![1])` mutants would all corrupt subsequent VRAM uploads.
        //
        // Saxman framing matches wildside_data::SAXMAN_FIXTURE: a 2-byte little-
        // endian compressed length, a control byte selecting eight literal
        // slots, the four ASCII bytes "ABCD", and a trailing zero pair.
        let rel = std::path::Path::new("artcomp/fixture.bin");
        let mut source = wildside_data::InMemoryAssetSource::new();
        source.insert(
            rel.to_path_buf(),
            vec![0x00, 0x02, 0xF4, 0x60, 0x41, 0x42, 0x43, 0x44, 0x00, 0x00],
        );

        let decoded = load_saxman_art(&source, rel).expect("decompress saxman art");

        assert_eq!(decoded.len(), 4, "fixture decodes to four literal bytes");
        assert_eq!(decoded, b"ABCD", "fixture decodes to ASCII 'ABCD'");
    }
}
