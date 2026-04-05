use anyhow::{Context, Result, anyhow, bail, ensure};
use image::{GrayImage, Luma};
use serde::{Deserialize, Serialize};
use sha1::{Digest, Sha1};
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process;

mod level_layout;

const COPIED_DIRECTORIES: &[&str] = &[
    "code/enemy",
    "doc",
    "ingame/anim",
    "ingame/artcomp",
    "ingame/artunc",
    "ingame/artunc_kid",
    "ingame/palette_enemy",
    "ingame/palette_kid",
    "level",
    "scenes",
    "sound",
    "theme",
];
const COPIED_FILES: &[&str] = &["constants.asm", "variables.asm"];
const PRESERVED_BUNDLE_DIRECTORIES: &[&str] = &["level-layout"];
pub const GENERATED_BUNDLE_VERSION: u32 = 9;
pub const DECODED_LEVEL_ASSET_VERSION: u32 = 3;
const BACKGROUND_PREVIEW_WIDTH_TILES: usize = 64;
const OPTION_MENU_TILE_SPLIT_WORD_COUNT_SUB_1CCAE: usize = 0x1180;
#[cfg_attr(not(test), allow(dead_code))]
const ENIGMA_BIT_MASKS: [u16; 16] = [
    0x0001, 0x0003, 0x0007, 0x000F, 0x001F, 0x003F, 0x007F, 0x00FF, 0x01FF, 0x03FF, 0x07FF, 0x0FFF,
    0x1FFF, 0x3FFF, 0x7FFF, 0xFFFF,
];

fn decoded_level_output_dir(level_id: u8) -> String {
    format!("decoded/level/{level_id:02x}")
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AssetManifest {
    #[serde(default)]
    pub generated_bundle_version: u32,
    pub rom_sha1: String,
    pub upstream_commit: String,
    pub source_root: PathBuf,
    pub output_root: PathBuf,
    #[serde(default)]
    pub map_order: Vec<u8>,
    pub levels: Vec<LevelRecord>,
    pub constants: Vec<NamedValue>,
    pub ram_variables: Vec<RamVariable>,
    pub enemy_sources: Vec<AssetFile>,
    pub theme_assets: Vec<AssetFile>,
    pub scene_assets: Vec<AssetFile>,
    pub animation_assets: Vec<AssetFile>,
    pub sound_assets: Vec<AssetFile>,
    #[serde(default)]
    pub ingame_assets: Vec<AssetFile>,
    #[serde(default)]
    pub decoded_levels: Vec<DecodedLevelAssets>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DecodedLevelAssets {
    pub contract_version: u32,
    pub level_id: u8,
    pub foreground: DecodedAssetFile,
    pub block: DecodedAssetFile,
    pub background: DecodedAssetFile,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DecodedAssetFile {
    pub relative_path: String,
    pub preview_png_path: String,
    pub size_bytes: u64,
    pub sha1: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LevelRecord {
    pub level_id: u8,
    pub title_text_id: Option<u8>,
    pub display_name: Option<String>,
    #[serde(default)]
    pub title_card: Option<LevelTitleCard>,
    pub resource_paths: Vec<String>,
    #[serde(default)]
    pub resources: LevelResources,
    #[serde(default)]
    pub header_metadata: Option<LevelHeader>,
    #[serde(default)]
    pub enemy_layout_metadata: Option<EnemyLayout>,
    #[serde(default)]
    pub bgscroll_metadata: Option<BgscrollProfile>,
    #[serde(default)]
    pub platform_metadata: Option<PlatformLayout>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LevelTitleCard {
    pub script_bytes: Vec<u8>,
    pub line_destinations: Vec<TitleCardDestination>,
    pub level_number_destination: Option<TitleCardDestination>,
    pub level_number_glyph_count: u16,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TitleCardDestination {
    pub x: u16,
    pub y: u16,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct LevelResources {
    pub platform_script: Option<String>,
    pub bgscroll: Option<String>,
    pub header: Option<String>,
    pub enemy: Option<String>,
    pub foreground: Option<String>,
    pub block: Option<String>,
    pub background: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LevelAssetBundle {
    pub level_id: u8,
    pub display_name: Option<String>,
    pub title_card: Option<LevelTitleCard>,
    pub platform_script: Option<LoadedAsset>,
    pub parsed_platform_layout: Option<PlatformLayout>,
    pub bgscroll: Option<LoadedAsset>,
    pub parsed_bgscroll: Option<BgscrollProfile>,
    pub header: Option<LoadedAsset>,
    pub parsed_header: Option<LevelHeader>,
    pub enemy: Option<LoadedAsset>,
    pub parsed_enemy_layout: Option<EnemyLayout>,
    pub foreground: Option<LoadedAsset>,
    pub block: Option<LoadedAsset>,
    pub background: Option<LoadedAsset>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LoadedAsset {
    pub relative_path: String,
    pub bytes: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IngameVramUpload {
    pub vram_word_address: u16,
    pub asset: LoadedAsset,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TitleScreenRegionVariantSub1B2A4 {
    Default,
    Variant1,
    Variant2,
}

impl TitleScreenRegionVariantSub1B2A4 {
    fn from_region_byte(region_byte: u8) -> Self {
        match region_byte {
            0 => Self::Default,
            1 => Self::Variant1,
            _ => Self::Variant2,
        }
    }
}

#[allow(non_snake_case)]
pub fn advance_title_screen_region_byte_sub_1C278(region_byte: u8) -> u8 {
    // ASM sub_1C278 (kid.asm:31143): increment ($FFFFFC82).w as a byte and
    // wrap the title-screen region selector back to zero once it reaches 3.
    let next_region_byte = region_byte.wrapping_add(1);
    if next_region_byte == 3 {
        0
    } else {
        next_region_byte
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SymbolicVramAssetLoadSub1B2A4 {
    pub vram_word_address: u16,
    pub asset_label: &'static str,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SymbolicEnigmaTilemapLoadSub1B2A4 {
    pub vram_word_address: u16,
    pub asset_label: &'static str,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SymbolicPaletteLoadSub1B2A4 {
    pub destination_word_offset: u16,
    pub palette_label: &'static str,
    pub word_count: u16,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TitleScreenLoadPlanSub1B2A4 {
    pub region_variant: TitleScreenRegionVariantSub1B2A4,
    pub art_uploads: [SymbolicVramAssetLoadSub1B2A4; 4],
    pub tilemap_load: SymbolicEnigmaTilemapLoadSub1B2A4,
    pub tile_patch_words: [u16; 32],
    pub palette_loads: Vec<SymbolicPaletteLoadSub1B2A4>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SymbolicVramAssetLoadSub1B850 {
    pub vram_word_address: u16,
    pub asset_label: &'static str,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SymbolicEnigmaTilemapLoadSub1B850 {
    pub vram_word_address: u16,
    pub asset_label: &'static str,
    pub width_words: u16,
    pub height_words: u16,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RepeatedWordRowsVramUploadSub1B850 {
    pub vram_byte_address: u16,
    pub rows: [[u16; 64]; 28],
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SymbolicPaletteBuffer2Load {
    pub destination_word_offset: u16,
    pub palette_label: &'static str,
    pub word_count: u16,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TitleCharacterPaletteLoadSub1C512 {
    pub base_load: SymbolicPaletteBuffer2Load,
    pub variant_overlay: Option<SymbolicPaletteBuffer2Load>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TitleIntroLoadPlanSub1B850 {
    pub art_uploads: [SymbolicVramAssetLoadSub1B850; 5],
    pub tilemap_load: SymbolicEnigmaTilemapLoadSub1B850,
    pub repeated_rows_upload: RepeatedWordRowsVramUploadSub1B850,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TitleIntroGlyphWriteSub1C7F6 {
    pub vram_word_address: u16,
    pub vdp_control_longword: u32,
    pub tile_word: u16,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DecompressToVramSetupSub142CC {
    pub vram_word_address: u16,
    pub vdp_control_longword: u32,
    pub input_stream_offset: usize,
    pub command_stream_offset: usize,
    pub bit_reader_word: u16,
    pub bit_index: u16,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PaletteWordOverlaySub1C820 {
    pub destination_word_offset: u16,
    pub words: [u16; 5],
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PaletteWordOverlaySub80BC {
    pub destination_word_offset: u16,
    pub words: [u16; 12],
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EndingSequencePaletteLoadSub1C820 {
    pub base_loads: [SymbolicPaletteBuffer2Load; 2],
    pub variant_overlay: Option<PaletteWordOverlaySub1C820>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SymbolicEnigmaTilemapLoadSub1BF24 {
    pub vram_word_address: u16,
    pub asset_label: &'static str,
    pub width_words: u16,
    pub height_words: u16,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PlaneRectUploadSub1BF24 {
    pub plane_cell_x: u16,
    pub plane_cell_y: u16,
    pub width_words: u16,
    pub height_words: u16,
    pub fill_word: u16,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TitleScreenCharacterLoadPlanSub1BF24 {
    pub region_variant: TitleScreenRegionVariantSub1B2A4,
    pub frame_index: u16,
    pub tilemap_load: SymbolicEnigmaTilemapLoadSub1BF24,
    pub plane_upload: PlaneRectUploadSub1BF24,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SymbolicEnigmaTilemapLoadSub1C034 {
    pub vram_word_address: u16,
    pub asset_label: &'static str,
    pub width_words: u16,
    pub height_words: u16,
    pub left_padding_words: u16,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct VramRowWriteSub1C034 {
    pub vram_word_address: u16,
    pub source_word_offset: u16,
    pub word_count: u16,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IntroSequence5FrameLoadPlanSub1C034 {
    pub frame_index: u16,
    pub tilemap_load: SymbolicEnigmaTilemapLoadSub1C034,
    pub vram_rows: [VramRowWriteSub1C034; 25],
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SymbolicEnigmaTilemapLoadSub1C0EE {
    pub vram_word_address: u16,
    pub asset_label: &'static str,
    pub width_words: u16,
    pub height_words: u16,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PlaneRectUploadSub1C0EE {
    pub plane_cell_x: u16,
    pub plane_cell_y: u16,
    pub width_words: u16,
    pub height_words: u16,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IntroSequence5PanelLoadPlanSub1C0EE {
    pub tilemap_load: SymbolicEnigmaTilemapLoadSub1C0EE,
    pub plane_upload: PlaneRectUploadSub1C0EE,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IntroSequence5BlankPanelLoadPlanSub1C112 {
    pub cleared_word_count: u16,
    pub plane_upload: PlaneRectUploadSub1C0EE,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LongwordVramUploadSub1B41C {
    pub vram_word_address: u16,
    pub longwords: [u32; 32],
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LevelLayoutXorVramUploadSub1C180 {
    pub vram_word_address: u16,
    pub longwords: [u32; 704],
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlaneLongwordStripUploadSub30194 {
    pub vram_word_address: u16,
    pub longwords: Vec<u32>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlaneWordStripUploadSub30194 {
    pub vram_word_address: u16,
    pub words: Vec<u16>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LevelPlaneScrollUpdatePlanSub30194 {
    pub horizontal_uploads: Vec<PlaneLongwordStripUploadSub30194>,
    pub vertical_uploads: Vec<PlaneWordStripUploadSub30194>,
}

#[derive(Debug, Clone, Copy)]
pub struct LevelPlaneScrollUpdateInputsSub30194<'a> {
    pub camera_x_pos: i16,
    pub camera_y_pos: i16,
    pub previous_camera_x_pos: i16,
    pub previous_camera_y_pos: i16,
    pub level_layout_row_offsets_cells: &'a [u16],
    pub level_layout_cells: &'a [u16],
    pub level_width_tiles: u16,
    pub level_height_tiles: u16,
    pub theme_mappings: &'a [u16],
    pub block_mappings: &'a [u16],
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BackgroundRowUploadSub303BA {
    pub vram_word_address: u16,
    pub upper_longwords: Vec<u32>,
    pub lower_longwords: Vec<u32>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BackgroundScrollUpdateSub304B4 {
    pub background_buffer_column: u16,
    pub vram_word_offset: u16,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BackgroundScrollStateSub30744 {
    pub background_buffer_column: Option<u16>,
    pub vram_word_offset: u16,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BackgroundTileUploadSub30744 {
    pub vram_word_address: u16,
    pub tile_word: u16,
    pub background_chunk_id: u8,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BackgroundRowUploadSegmentSub307D8 {
    pub vram_word_address: u16,
    pub tile_words: Vec<u16>,
    pub background_chunk_ids: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OptionMenuTileSplitUploadSub1CCAE {
    pub upper_vram_word_address: u16,
    pub upper_words: Vec<u16>,
    pub lower_vram_word_address: u16,
    pub lower_words: Vec<u16>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClearBit15VramUploadSub1CD44 {
    pub read_vram_byte_address: u16,
    pub write_vram_byte_address: u16,
    pub masked_words: Vec<u16>,
    pub delay_loop_iterations: u16,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DecompressionBufferRowsUploadSub1CD88 {
    pub rows: [[u16; 64]; 28],
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlaneLineUploadSub1CDA8 {
    pub source_word_offset: usize,
    pub vram_byte_address: u16,
    pub vdp_control_longword: u32,
    pub words: [u16; 40],
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FilledPlaneLineUploadSub1CDFA {
    pub row_index: u16,
    pub vram_byte_address: u16,
    pub vdp_control_longword: u32,
    pub words: [u16; 28],
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EndSequencePlaneClearPlanSub1CDE0 {
    pub display_refresh_count: u16,
    pub line_uploads: [FilledPlaneLineUploadSub1CDFA; 20],
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PlaneLineUploadSub1D990 {
    pub row_index: u16,
    pub source_word_offset: usize,
    pub vram_byte_address: u16,
    pub vdp_control_longword: u32,
    pub words: [u16; 14],
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SymbolicEnigmaTilemapDecodeSub1D944 {
    pub asset_label: &'static str,
    pub vram_word_address: u16,
    pub panel_flag_fb5c: u8,
    pub panel_flag_fb5e: u8,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TitlePanelLoadPlanSub1D944 {
    pub tilemap_decode: SymbolicEnigmaTilemapDecodeSub1D944,
    pub line_uploads: [PlaneLineUploadSub1D990; 14],
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct VdpDataFillSub1DA62 {
    pub word_count: u16,
    pub fill_word: u16,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VramZeroFillSub1DA72 {
    pub vram_byte_address: u16,
    pub vdp_control_longword: u32,
    pub words: [u16; 16],
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DiamondHudDmaTransferSub5D4A {
    pub frame_table_byte_offset: u16,
    pub asset_label: &'static str,
    pub dma_source_address: u32,
    pub dma_length_registers: [u16; 2],
    pub dma_source_registers: [u16; 3],
    pub vram_byte_address: u16,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LevelLayoutCellCoordsSub1F8E {
    pub row_index: u16,
    pub column_index: u16,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EndSequenceGlyphUploadSub1D080 {
    pub vram_byte_address: u16,
    pub vdp_control_longword: u32,
    pub glyph_tile_word: u16,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EndSequenceTextRenderPlanSub1D080 {
    pub glyph_uploads: Vec<EndSequenceGlyphUploadSub1D080>,
    pub frames_per_glyph: u16,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SymbolicRamAssetDecodeSub1D0EC {
    pub vram_word_address: u16,
    pub asset_label: &'static str,
    pub remap_bytes: [u8; 16],
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EndSequenceHoloBackgroundLoadPlanSub1D0EC {
    pub art_decode: SymbolicRamAssetDecodeSub1D0EC,
    pub tilemap_load: SymbolicEnigmaTilemapLoadSub1B2A4,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TitleTextChameleonTilemapUploadSub1B532 {
    pub clear_vram_word_address: u16,
    pub clear_longword_count: u16,
    pub tilemap_vram_word_address: u16,
    pub rows: [[u16; 64]; 28],
}

const TITLE_SCREEN_TILE_PATCH_WORDS_SUB_1B2A4: [u16; 32] = [
    0x1111, 0x1010, 0x0010, 0x0011, 0x0010, 0x0010, 0x0010, 0x0010, 0x0010, 0x0010, 0x0000, 0x0000,
    0x0000, 0x0000, 0x0000, 0x0000, 0x0010, 0x0000, 0x0110, 0x0000, 0x1010, 0x0000, 0x0010, 0x0000,
    0x0010, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000,
];
const COLUMN_MASK_LOOKUP_SUB_1C1A2: [u8; 16] = [
    0x00, 0x12, 0x02, 0x10, 0x09, 0x1B, 0x0B, 0x19, 0x01, 0x13, 0x03, 0x11, 0x08, 0x1A, 0x0A, 0x18,
];

const SPRITE_SIZE_TILE_COUNTS_SUB_13C0: [u16; 16] =
    [0, 1, 2, 3, 1, 3, 5, 7, 2, 5, 8, 11, 3, 7, 11, 15];

#[derive(Debug, Clone, PartialEq, Eq)]
struct SpriteTableEntrySub13C0 {
    y: u16,
    size: u8,
    link: u8,
    tile_word: u16,
    x: u16,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct SpriteDmaTransferSub13C0 {
    tile_count: u16,
    art_source: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct SpriteBuilderStateSub13C0 {
    number_sprites: u8,
    tile_word_cursor: u16,
    dma_tile_word_cursor: u16,
    next_art_source: u32,
    dma_transfers: Vec<SpriteDmaTransferSub13C0>,
}

#[cfg_attr(not(test), allow(dead_code))]
fn append_sprite_entry_sub_13c0(
    state: &mut SpriteBuilderStateSub13C0,
    x: u16,
    y: u16,
    size: u8,
    uses_dynamic_art: bool,
) -> SpriteTableEntrySub13C0 {
    // ASM sub_13C0 (kid.asm:2452): append one sprite-table entry, bump
    // Number_Sprites, then advance the shared tile-word cursor by
    // NumberTilesPerSpriteSize[size]+1. Dynamic-art entries also mirror that
    // increment into $FFFFF83C and queue a DMA transfer from a4.
    let link = state.number_sprites.wrapping_add(1);
    state.number_sprites = link;
    let tile_word = state.tile_word_cursor;
    let tiles_used = SPRITE_SIZE_TILE_COUNTS_SUB_13C0[usize::from(size)].wrapping_add(1);
    state.tile_word_cursor = state.tile_word_cursor.wrapping_add(tiles_used);

    if uses_dynamic_art {
        state.dma_tile_word_cursor = state.dma_tile_word_cursor.wrapping_add(tiles_used);
        state.dma_transfers.push(SpriteDmaTransferSub13C0 {
            tile_count: tiles_used,
            art_source: state.next_art_source,
        });
        state.next_art_source = state
            .next_art_source
            .wrapping_add(u32::from(tiles_used) * 32);
    }

    SpriteTableEntrySub13C0 {
        y,
        size,
        link,
        tile_word,
        x,
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EnemyDefinition {
    pub id: u8,
    pub constant_name: String,
    pub symbolic_name: String,
    pub source_path: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LevelHeader {
    pub width_screens: u8,
    pub height_screens: u8,
    pub height_flag_bits: u8,
    pub level_theme: u8,
    pub murder_wall_flags: u8,
    pub background_theme: u8,
    pub weather_flags: u8,
    pub player_start_x: u16,
    pub player_start_y: u16,
    pub flag_x: u16,
    pub flag_y: u16,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EnemyLayout {
    #[serde(default)]
    pub axis_uses_y_position: bool,
    pub unknown_prefix: u32,
    pub enemy_type_slots: Vec<u16>,
    pub unknown_word: u16,
    pub header_byte: u8,
    pub object_count: u8,
    pub objects: Vec<EnemyLayoutObject>,
    #[serde(default)]
    pub segments: Vec<EnemyLayoutSegment>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EnemyLayoutObject {
    pub object_type: u8,
    pub flags: u8,
    pub hit_points_plus_one: u16,
    pub x: u16,
    pub y: u16,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EnemyLayoutSegment {
    pub lower_bound: u16,
    pub upper_bound: u16,
    pub enemy_type_slots: Vec<u16>,
    pub object_batch_flags: u8,
    pub object_count: u8,
    pub objects: Vec<EnemyLayoutObject>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BgscrollProfile {
    pub layer_speeds: Vec<u8>,
    pub has_terminator: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlatformLayout {
    pub placements: Vec<PlatformPlacement>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlatformPlacement {
    pub x: u16,
    pub y: u16,
    pub buffer_left: u8,
    pub buffer_right: u8,
    pub buffer_top: u8,
    pub buffer_bottom: u8,
    pub kind: u8,
    pub subtype: u8,
    pub h_variant: u8,
    pub v_variant: u8,
    pub script: PlatformScriptRef,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlatformScriptRef {
    pub raw: String,
    pub relative_offset: Option<u16>,
}

/// A single movement command from a platform script (`ptfm_move`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlatformMoveCommand {
    /// Number of frames to apply this velocity.
    pub duration: u16,
    /// FP16.16 X velocity (positive = right).
    pub x_velocity: i32,
    /// FP16.16 Y velocity (positive = down).
    pub y_velocity: i32,
}

/// Parsed platform movement script (a sequence of moves that loops).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlatformScript {
    pub commands: Vec<PlatformMoveCommand>,
}

/// All platform scripts from platform_scripts.asm, indexed by relative offset.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlatformScriptTable {
    pub scripts: std::collections::BTreeMap<u16, PlatformScript>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NamedValue {
    pub name: String,
    pub value: i64,
    pub comment: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RamVariable {
    pub name: String,
    pub address: u32,
    pub detail: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AssetFile {
    pub relative_path: String,
    pub size_bytes: u64,
    pub sha1: String,
}

pub fn generated_bundle_dir(repo_root: &Path, rom_sha1: &str) -> PathBuf {
    repo_root.join(".cache/generated").join(rom_sha1)
}

pub fn manifest_path(bundle_dir: &Path) -> PathBuf {
    bundle_dir.join("asset-manifest.json")
}

pub fn export_split_bundle(
    source_root: &Path,
    output_root: &Path,
    rom_sha1: &str,
    upstream_commit: &str,
) -> Result<AssetManifest> {
    if !source_root.is_dir() {
        bail!(
            "split source root does not exist: {}",
            source_root.display()
        );
    }

    let staging_root = create_bundle_staging_dir(output_root)?;
    let mut manifest =
        export_split_bundle_into(source_root, &staging_root, rom_sha1, upstream_commit)?;
    preserve_bundle_sidecars(output_root, &staging_root)?;
    replace_bundle_from_staging(&staging_root, output_root)?;
    manifest.output_root = output_root.to_path_buf();
    fs::write(
        manifest_path(output_root),
        serde_json::to_vec_pretty(&manifest)?,
    )
    .with_context(|| format!("failed to write {}", manifest_path(output_root).display()))?;
    Ok(manifest)
}

fn preserve_bundle_sidecars(existing_bundle_root: &Path, staging_root: &Path) -> Result<()> {
    if !existing_bundle_root.exists() {
        return Ok(());
    }
    for relative in PRESERVED_BUNDLE_DIRECTORIES {
        copy_directory_recursive(
            &existing_bundle_root.join(relative),
            &staging_root.join(relative),
        )?;
    }
    Ok(())
}

fn export_split_bundle_into(
    source_root: &Path,
    output_root: &Path,
    rom_sha1: &str,
    upstream_commit: &str,
) -> Result<AssetManifest> {
    for relative in COPIED_DIRECTORIES {
        copy_directory_recursive(&source_root.join(relative), &output_root.join(relative))?;
    }
    for relative in COPIED_FILES {
        copy_file(&source_root.join(relative), &output_root.join(relative))?;
    }

    let level_names = parse_level_names(
        &fs::read_to_string(output_root.join("doc/level_list.txt")).with_context(|| {
            format!(
                "failed to read {}",
                output_root.join("doc/level_list.txt").display()
            )
        })?,
    )?;
    let level_title_cards = parse_level_title_cards(
        &fs::read_to_string(output_root.join("level/levelnames.asm")).with_context(|| {
            format!(
                "failed to read {}",
                output_root.join("level/levelnames.asm").display()
            )
        })?,
    )?;
    let level_files = parse_level_files(
        &fs::read_to_string(output_root.join("level/level_files.txt")).with_context(|| {
            format!(
                "failed to read {}",
                output_root.join("level/level_files.txt").display()
            )
        })?,
    )?;
    let map_order = parse_map_order(
        &fs::read_to_string(output_root.join("level/maporder.asm")).with_context(|| {
            format!(
                "failed to read {}",
                output_root.join("level/maporder.asm").display()
            )
        })?,
    )?;
    let constants = parse_named_values(
        &fs::read_to_string(output_root.join("constants.asm")).with_context(|| {
            format!(
                "failed to read {}",
                output_root.join("constants.asm").display()
            )
        })?,
    )?;
    let ram_variables = parse_ram_variables(
        &fs::read_to_string(output_root.join("variables.asm")).with_context(|| {
            format!(
                "failed to read {}",
                output_root.join("variables.asm").display()
            )
        })?,
    )?;

    let manifest = AssetManifest {
        generated_bundle_version: GENERATED_BUNDLE_VERSION,
        rom_sha1: rom_sha1.to_owned(),
        upstream_commit: upstream_commit.to_owned(),
        source_root: source_root.to_path_buf(),
        output_root: output_root.to_path_buf(),
        map_order,
        levels: merge_level_catalogues(level_names, level_title_cards, level_files),
        constants,
        ram_variables,
        enemy_sources: collect_asset_files(&output_root.join("code/enemy"))?,
        theme_assets: collect_asset_files(&output_root.join("theme"))?,
        scene_assets: collect_asset_files(&output_root.join("scenes"))?,
        animation_assets: collect_asset_files(&output_root.join("ingame/anim"))?,
        sound_assets: collect_asset_files(&output_root.join("sound"))?,
        ingame_assets: collect_asset_files(&output_root.join("ingame"))?,
        decoded_levels: vec![],
    };
    let mut manifest = manifest;
    normalise_manifest(&mut manifest)?;
    manifest.decoded_levels = export_decoded_all_levels(&manifest)?;
    ensure_generated_bundle_contract(&manifest)?;

    fs::write(
        manifest_path(output_root),
        serde_json::to_vec_pretty(&manifest)?,
    )
    .with_context(|| format!("failed to write {}", manifest_path(output_root).display()))?;

    Ok(manifest)
}

fn create_bundle_staging_dir(output_root: &Path) -> Result<PathBuf> {
    let parent = output_root.parent().ok_or_else(|| {
        anyhow!(
            "generated bundle path has no parent directory: {}",
            output_root.display()
        )
    })?;
    fs::create_dir_all(parent).with_context(|| format!("failed to create {}", parent.display()))?;
    let staging_root = unique_bundle_temp_path(output_root, "staging");
    fs::create_dir_all(&staging_root)
        .with_context(|| format!("failed to create {}", staging_root.display()))?;
    Ok(staging_root)
}

fn replace_bundle_from_staging(staging_root: &Path, output_root: &Path) -> Result<()> {
    let backup_root = unique_bundle_temp_path(output_root, "backup");
    let had_existing_bundle = output_root.exists();
    if had_existing_bundle {
        fs::rename(output_root, &backup_root).with_context(|| {
            format!(
                "failed to move existing generated bundle {} to {}",
                output_root.display(),
                backup_root.display()
            )
        })?;
    }

    match fs::rename(staging_root, output_root) {
        Ok(()) => {
            if had_existing_bundle {
                fs::remove_dir_all(&backup_root).with_context(|| {
                    format!(
                        "failed to remove old generated bundle {}",
                        backup_root.display()
                    )
                })?;
            }
            Ok(())
        }
        Err(error) => {
            if had_existing_bundle {
                let restore_result = fs::rename(&backup_root, output_root);
                if let Err(restore_error) = restore_result {
                    return Err(restore_error).with_context(|| {
                        format!(
                            "failed to restore generated bundle {} after swap failure: {}",
                            output_root.display(),
                            error
                        )
                    });
                }
            }
            Err(error).with_context(|| {
                format!(
                    "failed to move generated bundle staging dir {} into place at {}",
                    staging_root.display(),
                    output_root.display()
                )
            })
        }
    }
}

fn unique_bundle_temp_path(output_root: &Path, suffix: &str) -> PathBuf {
    let parent = output_root
        .parent()
        .expect("generated bundle path should have a parent");
    let base_name = output_root
        .file_name()
        .expect("generated bundle path should have a filename")
        .to_string_lossy();
    for attempt in 0..10_000u32 {
        let candidate = parent.join(format!(
            ".{base_name}.{suffix}.{}.{}",
            process::id(),
            attempt
        ));
        if !candidate.exists() {
            return candidate;
        }
    }
    parent.join(format!(".{base_name}.{suffix}.{}", process::id()))
}

pub fn load_manifest(path: &Path) -> Result<AssetManifest> {
    let bytes =
        fs::read(path).with_context(|| format!("failed to read manifest {}", path.display()))?;
    let mut manifest: AssetManifest =
        serde_json::from_slice(&bytes).context("failed to parse asset manifest")?;
    normalise_manifest(&mut manifest)?;
    Ok(manifest)
}

pub fn ensure_generated_bundle_contract(manifest: &AssetManifest) -> Result<()> {
    if manifest.generated_bundle_version != GENERATED_BUNDLE_VERSION {
        bail!(
            "generated bundle version mismatch: expected {}, got {}",
            GENERATED_BUNDLE_VERSION,
            manifest.generated_bundle_version
        );
    }

    if manifest.decoded_levels.is_empty() {
        bail!("generated bundle has no decoded level assets");
    }
    for decoded in &manifest.decoded_levels {
        if decoded.contract_version != DECODED_LEVEL_ASSET_VERSION {
            bail!(
                "decoded level 0x{:02X} asset version mismatch: expected {}, got {}",
                decoded.level_id,
                DECODED_LEVEL_ASSET_VERSION,
                decoded.contract_version
            );
        }
        validate_decoded_asset_file(manifest, &decoded.foreground, "foreground")?;
        validate_decoded_asset_file(manifest, &decoded.block, "block")?;
        validate_decoded_asset_file(manifest, &decoded.background, "background")?;
    }
    Ok(())
}

pub fn resource_level_id(manifest: &AssetManifest, level_id: u8) -> u8 {
    manifest
        .map_order
        .get(usize::from(level_id))
        .copied()
        .unwrap_or(level_id)
}

pub fn resource_level_record(manifest: &AssetManifest, level_id: u8) -> Option<&LevelRecord> {
    let resource_level_id = resource_level_id(manifest, level_id);
    manifest
        .levels
        .iter()
        .find(|level| level.level_id == resource_level_id)
}

pub fn load_level_asset_bundle(manifest: &AssetManifest, level_id: u8) -> Result<LevelAssetBundle> {
    let level = manifest
        .levels
        .iter()
        .find(|level| level.level_id == level_id)
        .ok_or_else(|| anyhow!("level 0x{level_id:02X} not found in asset manifest"))?
        .clone();
    let resource_level = resource_level_record(manifest, level_id)
        .cloned()
        .unwrap_or_else(|| level.clone());
    let header = load_asset(manifest, resource_level.resources.header.as_deref())?;
    let parsed_header = header
        .as_ref()
        .map(|asset| parse_level_header(&asset.bytes))
        .transpose()?
        .or(resource_level.header_metadata.clone());
    let enemy = load_asset(manifest, resource_level.resources.enemy.as_deref())?;
    let parsed_enemy_layout = resource_level
        .enemy_layout_metadata
        .clone()
        .or(level.enemy_layout_metadata.clone())
        .or(enemy
            .as_ref()
            .map(|asset| parse_enemy_layout(&asset.bytes))
            .transpose()?);
    let bgscroll = load_asset(manifest, resource_level.resources.bgscroll.as_deref())?;
    let parsed_bgscroll = bgscroll
        .as_ref()
        .map(|asset| parse_bgscroll_profile(&asset.bytes))
        .transpose()?
        .or(resource_level.bgscroll_metadata.clone());
    let platform_script = load_asset(
        manifest,
        resource_level.resources.platform_script.as_deref(),
    )?;
    let parsed_platform_layout = platform_script
        .as_ref()
        .map(|asset| parse_platform_layout(std::str::from_utf8(&asset.bytes)?))
        .transpose()?
        .or(resource_level.platform_metadata.clone());
    Ok(LevelAssetBundle {
        level_id,
        display_name: level.display_name.clone(),
        title_card: level.title_card.clone(),
        platform_script,
        parsed_platform_layout,
        bgscroll,
        parsed_bgscroll,
        header,
        parsed_header,
        enemy,
        parsed_enemy_layout,
        foreground: load_asset(manifest, resource_level.resources.foreground.as_deref())?,
        block: load_asset(manifest, resource_level.resources.block.as_deref())?,
        background: load_asset(manifest, resource_level.resources.background.as_deref())?,
    })
}

pub fn enemy_definitions(manifest: &AssetManifest) -> Vec<EnemyDefinition> {
    let mut definitions = manifest
        .constants
        .iter()
        .filter_map(|value| {
            let symbolic_name = value.name.strip_prefix("objid_")?;
            let id = u8::try_from(value.value).ok()?;
            Some(EnemyDefinition {
                id,
                constant_name: value.name.clone(),
                symbolic_name: symbolic_name.to_owned(),
                source_path: find_enemy_source_path(manifest, symbolic_name),
            })
        })
        .collect::<Vec<_>>();
    definitions.sort_by_key(|definition| definition.id);
    definitions
}

pub fn find_enemy_definition(manifest: &AssetManifest, selector: &str) -> Option<EnemyDefinition> {
    let selector = selector.trim();
    let by_id = selector
        .strip_prefix("0x")
        .or_else(|| selector.strip_prefix("0X"))
        .or(Some(selector))
        .and_then(|hex| u8::from_str_radix(hex, 16).ok());
    let normalised_selector = normalise_enemy_name(selector);
    enemy_definitions(manifest).into_iter().find(|definition| {
        Some(definition.id) == by_id
            || normalise_enemy_name(&definition.constant_name) == normalised_selector
            || normalise_enemy_name(&definition.symbolic_name) == normalised_selector
    })
}

pub fn parse_level_header(bytes: &[u8]) -> Result<LevelHeader> {
    if bytes.len() < 0x0C {
        bail!(
            "level header was {} bytes, expected at least 12 bytes",
            bytes.len()
        );
    }

    let width_screens = bytes[0];
    let raw_height = bytes[1];
    let raw_level_theme = bytes[2];
    let raw_background_theme = bytes[3];

    Ok(LevelHeader {
        width_screens,
        height_screens: raw_height & 0x3F,
        height_flag_bits: raw_height >> 6,
        level_theme: raw_level_theme & 0x3F,
        murder_wall_flags: raw_level_theme >> 6,
        background_theme: raw_background_theme & 0x0F,
        weather_flags: raw_background_theme >> 4,
        player_start_x: read_u16_be(bytes, 4),
        player_start_y: read_u16_be(bytes, 6),
        flag_x: read_u16_be(bytes, 8),
        flag_y: read_u16_be(bytes, 10),
    })
}

pub fn parse_enemy_layout(bytes: &[u8]) -> Result<EnemyLayout> {
    const HEADER_LEN: usize = 12;
    const OBJECT_BATCH_HEADER_LEN: usize = 2;
    const OBJECT_LEN: usize = 8;

    if bytes.len() < HEADER_LEN + OBJECT_BATCH_HEADER_LEN {
        bail!(
            "enemy layout was {} bytes, expected at least {} bytes",
            bytes.len(),
            HEADER_LEN + OBJECT_BATCH_HEADER_LEN
        );
    }

    let object_batch_flags = bytes[HEADER_LEN];
    let object_count = bytes[HEADER_LEN + 1];
    let expected_len =
        HEADER_LEN + OBJECT_BATCH_HEADER_LEN + usize::from(object_count) * OBJECT_LEN;
    if bytes.len() < expected_len {
        bail!(
            "enemy layout was {} bytes, expected at least {expected_len} bytes for {object_count} objects",
            bytes.len()
        );
    }

    let mut objects = Vec::with_capacity(usize::from(object_count));
    for chunk in bytes[HEADER_LEN + OBJECT_BATCH_HEADER_LEN..expected_len].chunks_exact(OBJECT_LEN)
    {
        objects.push(EnemyLayoutObject {
            object_type: chunk[0],
            flags: chunk[1],
            hit_points_plus_one: read_u16_be(chunk, 2),
            x: read_u16_be(chunk, 4),
            y: read_u16_be(chunk, 6),
        });
    }

    let enemy_type_slots = vec![
        read_u16_be(bytes, 4),
        read_u16_be(bytes, 6),
        read_u16_be(bytes, 8),
    ];
    let lower_bound = read_u16_be(bytes, 2);
    let upper_bound = read_u16_be(bytes, 10);
    let segment = EnemyLayoutSegment {
        lower_bound,
        upper_bound,
        enemy_type_slots: enemy_type_slots.clone(),
        object_batch_flags,
        object_count,
        objects: objects.clone(),
    };

    Ok(EnemyLayout {
        axis_uses_y_position: read_u16_be(bytes, 0) != 0,
        unknown_prefix: read_u32_be(bytes, 0),
        enemy_type_slots,
        unknown_word: upper_bound,
        header_byte: object_batch_flags,
        object_count,
        objects,
        segments: vec![segment],
    })
}

pub fn parse_bgscroll_profile(bytes: &[u8]) -> Result<BgscrollProfile> {
    match bytes.iter().position(|byte| *byte == 0xFF) {
        Some(terminator_index) => Ok(BgscrollProfile {
            layer_speeds: bytes[..terminator_index].to_vec(),
            has_terminator: true,
        }),
        None => Ok(BgscrollProfile {
            layer_speeds: bytes.to_vec(),
            has_terminator: false,
        }),
    }
}

pub fn parse_platform_layout(contents: &str) -> Result<PlatformLayout> {
    let mut placements = Vec::new();
    for (index, line) in contents.lines().enumerate() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with(';') || trimmed.starts_with("dc.w") {
            continue;
        }
        let Some(arguments) = trimmed.strip_prefix("ptfm") else {
            continue;
        };
        let fields = arguments
            .trim()
            .split(',')
            .map(str::trim)
            .collect::<Vec<_>>();
        if fields.len() != 11 {
            bail!(
                "platform layout line {} had {} ptfm fields, expected 11",
                index + 1,
                fields.len()
            );
        }
        placements.push(PlatformPlacement {
            x: fields[0]
                .parse()
                .with_context(|| format!("invalid platform x on line {}", index + 1))?,
            y: fields[1]
                .parse()
                .with_context(|| format!("invalid platform y on line {}", index + 1))?,
            buffer_left: fields[2]
                .parse()
                .with_context(|| format!("invalid platform left buffer on line {}", index + 1))?,
            buffer_right: fields[3]
                .parse()
                .with_context(|| format!("invalid platform right buffer on line {}", index + 1))?,
            buffer_top: fields[4]
                .parse()
                .with_context(|| format!("invalid platform top buffer on line {}", index + 1))?,
            buffer_bottom: fields[5]
                .parse()
                .with_context(|| format!("invalid platform bottom buffer on line {}", index + 1))?,
            kind: fields[6]
                .parse()
                .with_context(|| format!("invalid platform kind on line {}", index + 1))?,
            subtype: fields[7]
                .parse()
                .with_context(|| format!("invalid platform subtype on line {}", index + 1))?,
            h_variant: fields[8]
                .parse()
                .with_context(|| format!("invalid platform h variant on line {}", index + 1))?,
            v_variant: fields[9]
                .parse()
                .with_context(|| format!("invalid platform v variant on line {}", index + 1))?,
            script: parse_platform_script_ref(fields[10]),
        });
    }
    Ok(PlatformLayout { placements })
}

/// Parse the shared `platform_scripts.asm` file into a lookup table.
///
/// Format: blocks of `PlatformScript_XXXX: ; relative offset: YYYY`
/// followed by `ptfm_move duration, x_vel, y_vel` lines, terminated
/// by `dc.w $FFFF` and a loop-back pointer.
pub fn parse_platform_scripts(contents: &str) -> PlatformScriptTable {
    let mut scripts = std::collections::BTreeMap::new();
    let mut current_offset: Option<u16> = None;
    let mut current_commands: Vec<PlatformMoveCommand> = Vec::new();

    for line in contents.lines() {
        let trimmed = line.trim();

        // Detect script label: "PlatformScript_XXXX:   ; relative offset: YYYY"
        if let Some(rest) = trimmed.strip_prefix("PlatformScript_") {
            if let Some(colon_idx) = rest.find(':') {
                // Flush previous script
                if let Some(offset) = current_offset
                    && !current_commands.is_empty()
                {
                    scripts.insert(
                        offset,
                        PlatformScript {
                            commands: std::mem::take(&mut current_commands),
                        },
                    );
                }
                // Parse relative offset from comment or from label hex
                if let Some(comment) = rest.get(colon_idx + 1..) {
                    if let Some(hex_str) = comment.trim().strip_prefix("; relative offset: ") {
                        current_offset = u16::from_str_radix(hex_str.trim(), 16).ok();
                    } else {
                        // Fallback: parse from label name
                        current_offset =
                            u16::from_str_radix(&rest[..colon_idx], 16)
                                .ok()
                                .map(|addr| {
                                    // Convert absolute address to relative (base = 0x3602)
                                    addr.wrapping_sub(0x3602)
                                });
                    }
                }
            }
            continue;
        }

        // Parse ptfm_move line
        if let Some(args) = trimmed.strip_prefix("ptfm_move") {
            let args = args.trim();
            let parts: Vec<&str> = args.split(',').map(|s| s.trim()).collect();
            if parts.len() == 3
                && let (Some(duration), Some(x_vel), Some(y_vel)) = (
                    parse_asm_integer(parts[0]),
                    parse_asm_signed(parts[1]),
                    parse_asm_signed(parts[2]),
                )
            {
                current_commands.push(PlatformMoveCommand {
                    duration: duration as u16,
                    x_velocity: x_vel,
                    y_velocity: y_vel,
                });
            }
        }
    }

    // Flush last script
    if let Some(offset) = current_offset
        && !current_commands.is_empty()
    {
        scripts.insert(
            offset,
            PlatformScript {
                commands: current_commands,
            },
        );
    }

    PlatformScriptTable { scripts }
}

fn parse_asm_integer(s: &str) -> Option<i64> {
    let s = s.trim().trim_start_matches('#');
    if let Some(hex) = s.strip_prefix('$') {
        i64::from_str_radix(hex, 16).ok()
    } else if let Some(hex) = s.strip_prefix("0x") {
        i64::from_str_radix(hex, 16).ok()
    } else {
        s.parse().ok()
    }
}

fn parse_asm_signed(s: &str) -> Option<i32> {
    let s = s.trim().trim_start_matches('#');
    let (neg, s) = if let Some(rest) = s.strip_prefix('-') {
        (true, rest)
    } else {
        (false, s)
    };
    let val = if let Some(hex) = s.strip_prefix('$') {
        i64::from_str_radix(hex, 16).ok()?
    } else if let Some(hex) = s.strip_prefix("0x") {
        i64::from_str_radix(hex, 16).ok()?
    } else {
        s.parse::<i64>().ok()?
    };
    let signed = if neg { -(val as i32) } else { val as i32 };
    Some(signed)
}

fn parse_platform_script_ref(value: &str) -> PlatformScriptRef {
    let relative_offset = value
        .strip_prefix("PlatformScript_")
        .and_then(|rest| rest.strip_suffix("-PlatformScript_BaseAddress"))
        .and_then(|hex| u16::from_str_radix(hex, 16).ok());
    PlatformScriptRef {
        raw: value.to_owned(),
        relative_offset,
    }
}

fn merge_level_catalogues(
    names: BTreeMap<u8, (u8, String)>,
    title_cards: BTreeMap<u8, LevelTitleCard>,
    files: BTreeMap<u8, Vec<String>>,
) -> Vec<LevelRecord> {
    let mut by_level = BTreeMap::new();
    for (level_id, (title_text_id, display_name)) in names {
        by_level.insert(
            level_id,
            LevelRecord {
                level_id,
                title_text_id: Some(title_text_id),
                display_name: Some(display_name),
                title_card: title_cards.get(&level_id).cloned(),
                resource_paths: files.get(&level_id).cloned().unwrap_or_default(),
                resources: level_resources_from_paths(
                    files.get(&level_id).cloned().unwrap_or_default().as_slice(),
                ),
                header_metadata: None,
                enemy_layout_metadata: None,
                bgscroll_metadata: None,
                platform_metadata: None,
            },
        );
    }
    for (level_id, resource_paths) in files {
        by_level.entry(level_id).or_insert(LevelRecord {
            level_id,
            title_text_id: None,
            display_name: None,
            title_card: title_cards.get(&level_id).cloned(),
            resources: level_resources_from_paths(&resource_paths),
            resource_paths,
            header_metadata: None,
            enemy_layout_metadata: None,
            bgscroll_metadata: None,
            platform_metadata: None,
        });
    }
    by_level.into_values().collect()
}

fn normalise_manifest(manifest: &mut AssetManifest) -> Result<()> {
    for level in &mut manifest.levels {
        if level.resources == LevelResources::default() {
            level.resources = level_resources_from_paths(&level.resource_paths);
        }
        if level.header_metadata.is_none()
            && let Some(relative_path) = level.resources.header.as_deref()
        {
            let full_path = manifest.output_root.join("level").join(relative_path);
            if full_path.is_file() {
                let bytes = fs::read(&full_path).with_context(|| {
                    format!("failed to read level header {}", full_path.display())
                })?;
                level.header_metadata = Some(parse_level_header(&bytes)?);
            }
        }
        if level
            .enemy_layout_metadata
            .as_ref()
            .is_none_or(|layout| layout.segments.is_empty())
            && let Some(relative_path) = level.resources.enemy.as_deref()
        {
            let full_path = manifest.output_root.join("level").join(relative_path);
            if full_path.is_file() {
                let bytes = fs::read(&full_path).with_context(|| {
                    format!("failed to read enemy layout {}", full_path.display())
                })?;
                level.enemy_layout_metadata = Some(parse_enemy_layout(&bytes)?);
            }
        }
        if level.bgscroll_metadata.is_none()
            && let Some(relative_path) = level.resources.bgscroll.as_deref()
        {
            let full_path = manifest.output_root.join("level").join(relative_path);
            if full_path.is_file() {
                let bytes = fs::read(&full_path)
                    .with_context(|| format!("failed to read bgscroll {}", full_path.display()))?;
                level.bgscroll_metadata = Some(parse_bgscroll_profile(&bytes)?);
            }
        }
        if level.platform_metadata.is_none()
            && let Some(relative_path) = level.resources.platform_script.as_deref()
        {
            let full_path = manifest.output_root.join("level").join(relative_path);
            if full_path.is_file() {
                let contents = fs::read_to_string(&full_path).with_context(|| {
                    format!("failed to read platform layout {}", full_path.display())
                })?;
                level.platform_metadata = Some(parse_platform_layout(&contents)?);
            }
        }
    }
    Ok(())
}

fn level_resources_from_paths(paths: &[String]) -> LevelResources {
    let mut resources = LevelResources::default();
    for path in paths {
        if path.starts_with("platform/") {
            resources.platform_script = Some(path.clone());
        } else if path.starts_with("bgscroll/") {
            resources.bgscroll = Some(path.clone());
        } else if path.starts_with("header/") {
            resources.header = Some(path.clone());
        } else if path.starts_with("enemy/") {
            resources.enemy = Some(path.clone());
        } else if path.starts_with("foreground/") {
            resources.foreground = Some(path.clone());
        } else if path.starts_with("block/") {
            resources.block = Some(path.clone());
        } else if path.starts_with("background/") {
            resources.background = Some(path.clone());
        }
    }
    resources
}

fn load_asset(
    manifest: &AssetManifest,
    relative_path: Option<&str>,
) -> Result<Option<LoadedAsset>> {
    let Some(relative_path) = relative_path else {
        return Ok(None);
    };
    let full_path = manifest.output_root.join("level").join(relative_path);
    let bytes = fs::read(&full_path)
        .with_context(|| format!("failed to read level asset {}", full_path.display()))?;
    Ok(Some(LoadedAsset {
        relative_path: relative_path.to_owned(),
        bytes,
    }))
}

pub fn load_ingame_asset(manifest: &AssetManifest, relative_path: &str) -> Result<LoadedAsset> {
    let full_path = manifest.output_root.join("ingame").join(relative_path);
    let bytes = fs::read(&full_path)
        .with_context(|| format!("failed to read ingame asset {}", full_path.display()))?;
    Ok(LoadedAsset {
        relative_path: relative_path.to_owned(),
        bytes,
    })
}

#[allow(non_snake_case)]
pub fn load_juggernaut_skull_and_star_art_sub_129CE(
    manifest: &AssetManifest,
) -> Result<[IngameVramUpload; 3]> {
    // ASM sub_129CE (kid.asm:25354): upload two Juggernaut skull frames to
    // VRAM $CDC0/$CDE0, then the star art to VRAM $DF40.
    Ok([
        IngameVramUpload {
            vram_word_address: 0xCDC0,
            asset: load_ingame_asset(manifest, "artunc/Juggernaut_skull_frame_1.bin")?,
        },
        IngameVramUpload {
            vram_word_address: 0xCDE0,
            asset: load_ingame_asset(manifest, "artunc/Juggernaut_skull_frame_2.bin")?,
        },
        IngameVramUpload {
            vram_word_address: 0xDF40,
            asset: load_ingame_asset(manifest, "artunc/Some_kind_of_star.bin")?,
        },
    ])
}

#[allow(non_snake_case)]
pub fn load_hitpoint_display_art_sub_12C24(manifest: &AssetManifest) -> Result<IngameVramUpload> {
    // ASM sub_12C24 (kid.asm:25409): upload Hitpoint_display art to VRAM
    // $45A0.
    Ok(IngameVramUpload {
        vram_word_address: 0x45A0,
        asset: load_ingame_asset(manifest, "artunc/Hitpoint_display.bin")?,
    })
}

#[allow(non_snake_case)]
pub fn title_screen_load_plan_sub_1B2A4(region_byte: u8) -> TitleScreenLoadPlanSub1B2A4 {
    // ASM sub_1B2A4 + loc_1C4B2 (kid.asm:29472-29505, 31197-31222): queue four
    // title-screen VRAM uploads, decode one Enigma tilemap to VRAM $20C1, push
    // the 32-word `unk_1B32C` patch to VRAM $F000, then seed Palette_Buffer
    // with the base `Pal_1C400` copy plus an optional region-specific overlay.
    let region_variant = TitleScreenRegionVariantSub1B2A4::from_region_byte(region_byte);
    let region_art_label = match region_variant {
        TitleScreenRegionVariantSub1B2A4::Default => "byte_26E3D",
        TitleScreenRegionVariantSub1B2A4::Variant1 => "byte_2927C",
        TitleScreenRegionVariantSub1B2A4::Variant2 => "byte_282C1",
    };
    let mut palette_loads = vec![SymbolicPaletteLoadSub1B2A4 {
        destination_word_offset: 0x0000,
        palette_label: "Pal_1C400",
        word_count: 0x0033,
    }];
    if !matches!(region_variant, TitleScreenRegionVariantSub1B2A4::Default) {
        palette_loads.push(SymbolicPaletteLoadSub1B2A4 {
            destination_word_offset: 0x001A,
            palette_label: match region_variant {
                TitleScreenRegionVariantSub1B2A4::Variant1 => "Pal_1C466",
                TitleScreenRegionVariantSub1B2A4::Variant2 => "Pal_1C48C",
                TitleScreenRegionVariantSub1B2A4::Default => unreachable!(),
            },
            word_count: 0x0013,
        });
    }
    TitleScreenLoadPlanSub1B2A4 {
        region_variant,
        art_uploads: [
            SymbolicVramAssetLoadSub1B2A4 {
                vram_word_address: 0x1820,
                asset_label: "byte_2A756",
            },
            SymbolicVramAssetLoadSub1B2A4 {
                vram_word_address: 0x21E0,
                asset_label: region_art_label,
            },
            SymbolicVramAssetLoadSub1B2A4 {
                vram_word_address: 0x6620,
                asset_label: "byte_2A961",
            },
            SymbolicVramAssetLoadSub1B2A4 {
                vram_word_address: 0xFA60,
                asset_label: "ArtComp_1DA86_LettersNumbers",
            },
        ],
        tilemap_load: SymbolicEnigmaTilemapLoadSub1B2A4 {
            vram_word_address: 0x20C1,
            asset_label: "unk_2F7D0",
        },
        tile_patch_words: TITLE_SCREEN_TILE_PATCH_WORDS_SUB_1B2A4,
        palette_loads,
    }
}

#[allow(non_snake_case)]
pub fn title_intro_load_plan_sub_1B850(
    decoded_row_words: &[u16],
) -> Result<TitleIntroLoadPlanSub1B850> {
    // ASM sub_1B850 (kid.asm:30172): queue five title-intro art decompresses,
    // decode `unk_2FDE0` to a 40x28 tilemap at VRAM word address 0 via
    // sub_1C5D0, then decode `unk_2FDCE` and let loc_1B8E4/loc_1B8E6 stamp
    // each of the first 28 words across a full 64-word row at VRAM byte
    // address $E000.
    let decoded_row_words = decoded_row_words.get(..28).ok_or_else(|| {
        anyhow!(
            "sub_1B850 needs 28 decoded row words for loc_1B8E4 but only received {}",
            decoded_row_words.len()
        )
    })?;
    let rows = core::array::from_fn(|row_index| [decoded_row_words[row_index]; 64]);
    Ok(TitleIntroLoadPlanSub1B850 {
        art_uploads: [
            SymbolicVramAssetLoadSub1B850 {
                vram_word_address: 0x1780,
                asset_label: "byte_2D73B",
            },
            SymbolicVramAssetLoadSub1B850 {
                vram_word_address: 0x2740,
                asset_label: "byte_2D71A",
            },
            SymbolicVramAssetLoadSub1B850 {
                vram_word_address: 0x2800,
                asset_label: "byte_2DEBF",
            },
            SymbolicVramAssetLoadSub1B850 {
                vram_word_address: 0x3AC0,
                asset_label: "byte_261D7",
            },
            SymbolicVramAssetLoadSub1B850 {
                vram_word_address: 0xFA60,
                asset_label: "ArtComp_1DA86_LettersNumbers",
            },
        ],
        tilemap_load: SymbolicEnigmaTilemapLoadSub1B850 {
            vram_word_address: 0x0000,
            asset_label: "unk_2FDE0",
            width_words: 40,
            height_words: 28,
        },
        repeated_rows_upload: RepeatedWordRowsVramUploadSub1B850 {
            vram_byte_address: 0xE000,
            rows,
        },
    })
}

#[allow(non_snake_case)]
pub fn title_intro_palette_load_sub_1C4EA() -> SymbolicPaletteBuffer2Load {
    // ASM sub_1C4EA (kid.asm:31227): copy 0x33 words from `Pal_1C28A` into
    // Palette_Buffer_2 starting at word offset 0.
    SymbolicPaletteBuffer2Load {
        destination_word_offset: 0x0000,
        palette_label: "Pal_1C28A",
        word_count: 0x0033,
    }
}

#[allow(non_snake_case)]
pub fn title_intro_palette_load_sub_1C4FE() -> SymbolicPaletteBuffer2Load {
    // ASM sub_1C4FE (kid.asm:31242): copy 0x23 words from `Pal_1C2F0` into
    // Palette_Buffer_2 starting at word offset 0.
    SymbolicPaletteBuffer2Load {
        destination_word_offset: 0x0000,
        palette_label: "Pal_1C2F0",
        word_count: 0x0023,
    }
}

#[allow(non_snake_case)]
pub fn title_character_palette_load_sub_1C512(
    region_byte: u8,
) -> TitleCharacterPaletteLoadSub1C512 {
    // ASM sub_1C512 (kid.asm:31257): copy 0x40 words from `Pal_1C346` into
    // Palette_Buffer_2, then if ($FFFFFC82).w is non-zero overlay 0xE words at
    // byte offset `$62` (word offset `$31`) from `Pal_1C3C6` when the byte is
    // 1 or `Pal_1C3D4` for any other non-zero variant.
    TitleCharacterPaletteLoadSub1C512 {
        base_load: SymbolicPaletteBuffer2Load {
            destination_word_offset: 0x0000,
            palette_label: "Pal_1C346",
            word_count: 0x0040,
        },
        variant_overlay: match region_byte {
            0 => None,
            1 => Some(SymbolicPaletteBuffer2Load {
                destination_word_offset: 0x0031,
                palette_label: "Pal_1C3C6",
                word_count: 0x000E,
            }),
            _ => Some(SymbolicPaletteBuffer2Load {
                destination_word_offset: 0x0031,
                palette_label: "Pal_1C3D4",
                word_count: 0x000E,
            }),
        },
    }
}

#[allow(non_snake_case)]
pub fn title_intro_glyph_write_sub_1C7F6(
    base_vram_word_address: u16,
    base_tile_word: u16,
    column: u16,
    row: u16,
    glyph_index: u16,
) -> TitleIntroGlyphWriteSub1C7F6 {
    // ASM sub_1C7F6 (kid.asm:31455): build the text-plane VRAM word address as
    // `base + row*$80 + column*2`, convert that word address into the VDP
    // control longword, then add the text header's tile base to the glyph
    // index before writing the tile word.
    let vram_word_address = base_vram_word_address
        .wrapping_add(row << 7)
        .wrapping_add(column)
        .wrapping_add(column);
    let high_word = (vram_word_address & 0x3FFF) | 0x4000;
    let low_word = vram_word_address.rotate_left(2) & 0x0003;

    TitleIntroGlyphWriteSub1C7F6 {
        vram_word_address,
        vdp_control_longword: (u32::from(high_word) << 16) | u32::from(low_word),
        tile_word: base_tile_word.wrapping_add(glyph_index),
    }
}

#[allow(non_snake_case)]
pub fn decompress_to_vram_setup_sub_142CC(
    compressed_bytes: &[u8],
    vram_word_address: u16,
) -> Result<DecompressToVramSetupSub142CC> {
    // ASM sub_142CC (kid.asm:27121): convert the caller's VRAM word address
    // into a VDP write-control longword, then read the next big-endian offset
    // and advance `a1` past the two-byte header plus that offset while `a0`
    // keeps pointing at the payload stream that starts immediately after the
    // header. The bit reader and decompression buffer cursors are zeroed here.
    let offset_bytes = compressed_bytes
        .get(..2)
        .context("sub_142CC needs the two-byte command-stream offset header")?;
    let command_stream_delta = usize::from(u16::from_be_bytes([offset_bytes[0], offset_bytes[1]]));
    let high_word = (vram_word_address & 0x3FFF) | 0x4000;
    let low_word = vram_word_address.rotate_left(2) & 0x0003;

    Ok(DecompressToVramSetupSub142CC {
        vram_word_address,
        vdp_control_longword: (u32::from(high_word) << 16) | u32::from(low_word),
        input_stream_offset: 2,
        command_stream_offset: 2 + command_stream_delta,
        bit_reader_word: 0,
        bit_index: 0,
    })
}

#[allow(non_snake_case)]
pub fn ending_sequence_palette_load_sub_1C820(
    region_variant: u16,
) -> EndingSequencePaletteLoadSub1C820 {
    // ASM sub_1C820 (kid.asm:31478): copy the first $10 words from `Pal_1C8AA`
    // into Palette_Buffer, optionally overwrite words $09..$0D with the hard-
    // coded region variant when d1 is non-zero, then continue copying the
    // remaining $30 words into Palette_Buffer+$20.
    EndingSequencePaletteLoadSub1C820 {
        base_loads: [
            SymbolicPaletteBuffer2Load {
                destination_word_offset: 0x0000,
                palette_label: "Pal_1C8AA",
                word_count: 0x0010,
            },
            SymbolicPaletteBuffer2Load {
                destination_word_offset: 0x0010,
                palette_label: "Pal_1C8AA",
                word_count: 0x0030,
            },
        ],
        variant_overlay: if region_variant == 0 {
            None
        } else {
            Some(PaletteWordOverlaySub1C820 {
                destination_word_offset: 0x0009,
                words: [0x0040, 0x00A0, 0xE004, 0x00EE, 0x0EEE],
            })
        },
    }
}

#[allow(non_snake_case)]
pub fn damage_flash_palette_clear_sub_80BC() -> PaletteWordOverlaySub80BC {
    // ASM sub_80BC (kid.asm:8817): fill 12 words at Palette_Buffer+$62 with
    // `$0FFF`.
    PaletteWordOverlaySub80BC {
        destination_word_offset: 0x0031,
        words: [0x0FFF; 12],
    }
}

#[allow(non_snake_case)]
pub fn damage_flash_helmet_palette_load_sub_80D0(
    helmet_id: u16,
) -> Option<SymbolicPaletteLoadSub1B2A4> {
    // ASM sub_80D0 (kid.asm:8833): index `off_80F2` by `Current_Helmet` and
    // copy 12 words from the selected palette into Palette_Buffer+$62.
    const PALETTES: [&str; 10] = [
        "Pal_A1C72",
        "Pal_A1CE4",
        "Pal_A1E1C",
        "Pal_A1DA4",
        "Pal_A1DE0",
        "Pal_A1E58",
        "Pal_A1E94",
        "Pal_A1D68",
        "Pal_A1C8A",
        "Pal_A1D26",
    ];

    PALETTES
        .get(usize::from(helmet_id))
        .copied()
        .map(|palette_label| SymbolicPaletteLoadSub1B2A4 {
            destination_word_offset: 0x0031,
            palette_label,
            word_count: 0x000C,
        })
}

#[allow(non_snake_case)]
pub fn helmet_pose_palette_load_sub_8106(helmet_id: u16) -> Option<SymbolicPaletteLoadSub1B2A4> {
    // ASM sub_8106 (kid.asm:8864): index `off_8128` by `Current_Helmet` and
    // copy 15 words from the selected palette into Palette_Buffer+$62.
    const PALETTES: [&str; 10] = [
        "Pal_A1C72",
        "Pal_A1D08",
        "Pal_A1E3A",
        "Pal_A1DC2",
        "Pal_A1DFE",
        "Pal_A1E76",
        "Pal_A1EB2",
        "Pal_A1D86",
        "Pal_A1CA8",
        "Pal_A1D4A",
    ];

    PALETTES
        .get(usize::from(helmet_id))
        .copied()
        .map(|palette_label| SymbolicPaletteLoadSub1B2A4 {
            destination_word_offset: 0x0031,
            palette_label,
            word_count: 0x000F,
        })
}

#[allow(non_snake_case)]
pub fn option_menu_palette_load_sub_1C896() -> SymbolicPaletteLoadSub1B2A4 {
    // ASM sub_1C896 (kid.asm:31542): copy 0x40 words from `Pal_1C97C`
    // straight into `Palette_Buffer` starting at word offset 0.
    SymbolicPaletteLoadSub1B2A4 {
        destination_word_offset: 0x0000,
        palette_label: "Pal_1C97C",
        word_count: 0x0040,
    }
}

#[allow(non_snake_case)]
pub fn option_menu_tile_split_upload_sub_1CCAE(
    source_words: &[u16],
) -> Result<OptionMenuTileSplitUploadSub1CCAE> {
    // ASM sub_1CCAE + sub_1CD20 + sub_1CD32 (kid.asm:31831-31899): read
    // `$1180` words from VRAM `$5580`, first split each 4-word chunk into
    // low-nibble words (`& $0F0F`) followed by high-nibble words (`& $F0F0`)
    // and upload that to VRAM `$7880`, then repeat the read with the opposite
    // high-first / low-second ordering and upload it back to VRAM `$5580`.
    let source_words = source_words
        .get(..OPTION_MENU_TILE_SPLIT_WORD_COUNT_SUB_1CCAE)
        .ok_or_else(|| {
            anyhow!(
                "sub_1CCAE needs {OPTION_MENU_TILE_SPLIT_WORD_COUNT_SUB_1CCAE:#06X} words but only received {:#06X}",
                source_words.len()
            )
        })?;

    let mut upper_words = Vec::with_capacity(OPTION_MENU_TILE_SPLIT_WORD_COUNT_SUB_1CCAE);
    let mut lower_words = Vec::with_capacity(OPTION_MENU_TILE_SPLIT_WORD_COUNT_SUB_1CCAE);

    for chunk in source_words.chunks_exact(4) {
        upper_words.push(chunk[0] & 0x0F0F);
        upper_words.push(chunk[1] & 0x0F0F);
        upper_words.push(chunk[2] & 0xF0F0);
        upper_words.push(chunk[3] & 0xF0F0);

        lower_words.push(chunk[0] & 0xF0F0);
        lower_words.push(chunk[1] & 0xF0F0);
        lower_words.push(chunk[2] & 0x0F0F);
        lower_words.push(chunk[3] & 0x0F0F);
    }

    Ok(OptionMenuTileSplitUploadSub1CCAE {
        upper_vram_word_address: 0x7880,
        upper_words,
        lower_vram_word_address: 0x5580,
        lower_words,
    })
}

#[allow(non_snake_case)]
pub fn clear_bit15_vram_upload_sub_1CD44(
    source_words: &[u16],
) -> Result<ClearBit15VramUploadSub1CD44> {
    // ASM sub_1CD44 (kid.asm:31902): read `$800` words from VRAM address 0,
    // clear bit 15 on each word via `bclr #$F,d6`, write the masked span back
    // to VRAM address 0, then spin for `$101` `dbf` iterations.
    const WORD_COUNT: usize = 0x0800;

    let source_words = source_words.get(..WORD_COUNT).ok_or_else(|| {
        anyhow!(
            "sub_1CD44 needs {WORD_COUNT:#06X} source words but only received {:#06X}",
            source_words.len()
        )
    })?;

    Ok(ClearBit15VramUploadSub1CD44 {
        read_vram_byte_address: 0x0000,
        write_vram_byte_address: 0x0000,
        masked_words: source_words.iter().map(|word| word & 0x7FFF).collect(),
        delay_loop_iterations: 0x0101,
    })
}

#[allow(non_snake_case)]
pub fn decompression_buffer_rows_upload_sub_1CD88(
    decoded_words: &[u16],
) -> Result<DecompressionBufferRowsUploadSub1CD88> {
    // ASM sub_1CD88 (kid.asm:31931): stream 28 rows from
    // `Decompression_Buffer`, taking 40 sequential decoded words per row and
    // padding the remaining 24 columns with tile word `$94`.
    const ROW_COUNT: usize = 28;
    const SOURCE_WORDS_PER_ROW: usize = 40;
    const REQUIRED_WORDS: usize = ROW_COUNT * SOURCE_WORDS_PER_ROW;

    let decoded_words = decoded_words.get(..REQUIRED_WORDS).ok_or_else(|| {
        anyhow!(
            "sub_1CD88 needs {REQUIRED_WORDS:#06X} decoded words but only received {:#06X}",
            decoded_words.len()
        )
    })?;

    let rows = core::array::from_fn(|row_index| {
        let row_start = row_index * SOURCE_WORDS_PER_ROW;
        core::array::from_fn(|column_index| {
            if column_index < SOURCE_WORDS_PER_ROW {
                decoded_words[row_start + column_index]
            } else {
                0x0094
            }
        })
    });

    Ok(DecompressionBufferRowsUploadSub1CD88 { rows })
}

fn vdp_write_control_longword(vram_byte_address: u16) -> u32 {
    let high_word = (vram_byte_address & 0x3FFF) | 0x4000;
    let low_word = vram_byte_address.rotate_left(2) & 0x0003;
    (u32::from(high_word) << 16) | u32::from(low_word)
}

fn dma_source_register_words(source_address: u32) -> [u16; 3] {
    let shifted = source_address >> 1;
    [
        0x9500 | ((shifted as u16) & 0x00FF),
        0x9600 | (((shifted as u16) >> 8) & 0x00FF),
        0x9700 | (((shifted >> 16) as u16) & 0x00FF),
    ]
}

#[allow(non_snake_case)]
pub fn diamond_hud_dma_transfer_sub_5D4A(
    frame_table_byte_offset: u16,
) -> Result<DiamondHudDmaTransferSub5D4A> {
    // ASM sub_5D4A (kid.asm:5771): use the caller's 4-byte selector inside
    // `off_7B0AC`, read the chosen longword pointer, skip the two-byte header
    // on the pointed resource, then programme a `$200`-byte VRAM DMA to `$DD40`.
    const DIAMOND_FRAME_POINTERS: [(&str, u32); 4] = [
        ("unk_99FCA", 0x0009_9FCA),
        ("unk_9A1CC", 0x0009_A1CC),
        ("unk_9A3CE", 0x0009_A3CE),
        ("unk_9A5D0", 0x0009_A5D0),
    ];

    ensure!(
        frame_table_byte_offset.is_multiple_of(4),
        "sub_5D4A expects a 4-byte selector into `off_7B0AC`, got {frame_table_byte_offset:#06X}",
    );

    let frame_index = usize::from(frame_table_byte_offset >> 2);
    let (asset_label, table_address) = *DIAMOND_FRAME_POINTERS
        .get(frame_index)
        .ok_or_else(|| {
            anyhow!(
                "sub_5D4A selector {frame_table_byte_offset:#06X} falls outside the 4-entry `off_7B0AC` table"
            )
        })?;
    let dma_source_address = table_address + 2;

    Ok(DiamondHudDmaTransferSub5D4A {
        frame_table_byte_offset,
        asset_label,
        dma_source_address,
        dma_length_registers: [0x9300, 0x9401],
        dma_source_registers: dma_source_register_words(dma_source_address),
        vram_byte_address: 0xDD40,
    })
}

#[allow(non_snake_case)]
pub fn plane_line_upload_sub_1CDA8(
    decoded_words: &[u16],
    source_row_index: u16,
    plane_row_index: u16,
    plane_base_vram_byte_address: u16,
) -> Result<PlaneLineUploadSub1CDA8> {
    // ASM sub_1CDA8 (kid.asm:31954): use `d0*$50` as the source row stride
    // inside `Decompression_Buffer`, build a VRAM write command for
    // `($FFFFFB60).w + d1*$80`, then write exactly 40 words from that source
    // row.
    const SOURCE_ROW_STRIDE_WORDS: usize = 0x50;
    const WORDS_PER_LINE: usize = 40;

    let source_word_offset = usize::from(source_row_index) * SOURCE_ROW_STRIDE_WORDS;
    let source_words = decoded_words
        .get(source_word_offset..source_word_offset + WORDS_PER_LINE)
        .ok_or_else(|| {
            anyhow!(
                "sub_1CDA8 needs row {} words [{:#06X}..{:#06X}) but only received {:#06X}",
                source_row_index,
                source_word_offset,
                source_word_offset + WORDS_PER_LINE,
                decoded_words.len()
            )
        })?;

    let vram_byte_address = plane_base_vram_byte_address.wrapping_add(plane_row_index << 7);
    Ok(PlaneLineUploadSub1CDA8 {
        source_word_offset,
        vram_byte_address,
        vdp_control_longword: vdp_write_control_longword(vram_byte_address),
        words: core::array::from_fn(|index| source_words[index]),
    })
}

#[allow(non_snake_case)]
pub fn filled_plane_line_upload_sub_1CDFA(row_index: u16) -> FilledPlaneLineUploadSub1CDFA {
    // ASM sub_1CDFA (kid.asm:31999): build a VRAM write command for
    // `d0*$80 - $2000 + $000C`, then stamp exactly 28 `$94` fill words.
    const WORDS_PER_LINE: usize = 28;

    let vram_byte_address = row_index
        .wrapping_mul(0x0080)
        .wrapping_sub(0x2000)
        .wrapping_add(0x000C);
    FilledPlaneLineUploadSub1CDFA {
        row_index,
        vram_byte_address,
        vdp_control_longword: vdp_write_control_longword(vram_byte_address),
        words: [0x0094; WORDS_PER_LINE],
    }
}

#[allow(non_snake_case)]
pub fn end_sequence_plane_clear_plan_sub_1CDE0() -> EndSequencePlaneClearPlanSub1CDE0 {
    // ASM sub_1CDE0 (kid.asm:31981): refresh the display via sub_1DA24 ten
    // times while filling rows 9 through 28 inclusive with sub_1CDFA.
    EndSequencePlaneClearPlanSub1CDE0 {
        display_refresh_count: 10,
        line_uploads: core::array::from_fn(|index| {
            filled_plane_line_upload_sub_1CDFA(9 + index as u16)
        }),
    }
}

#[allow(non_snake_case)]
pub fn title_panel_line_upload_sub_1D990(
    decoded_words: &[u16],
    row_index: u16,
    panel_flag_fb5c: u8,
) -> Result<PlaneLineUploadSub1D990> {
    // ASM sub_1D990 (kid.asm:33250): use `d7*$1C` as the source row stride
    // inside `Decompression_Buffer`, build a VRAM write command for
    // `((d7+7)*$80 + $1A)`, subtract `$2000` when `($FFFFFB5C).w` is non-zero,
    // then stream exactly 14 words from that row.
    const SOURCE_WORD_STRIDE: usize = 0x1C;
    const WORD_COUNT: usize = 14;

    let source_word_offset = usize::from(row_index) * SOURCE_WORD_STRIDE;
    let source_words = decoded_words
        .get(source_word_offset..source_word_offset + WORD_COUNT)
        .ok_or_else(|| {
            anyhow!(
                "sub_1D990 needs row {} words [{:#06X}..{:#06X}) but only received {:#06X}",
                row_index,
                source_word_offset,
                source_word_offset + WORD_COUNT,
                decoded_words.len()
            )
        })?;

    let base_vram_byte_address = row_index
        .wrapping_add(7)
        .wrapping_mul(0x0080)
        .wrapping_add(0x001A);
    let vram_byte_address = if panel_flag_fb5c != 0 {
        base_vram_byte_address.wrapping_sub(0x2000)
    } else {
        base_vram_byte_address
    };

    Ok(PlaneLineUploadSub1D990 {
        row_index,
        source_word_offset,
        vram_byte_address,
        vdp_control_longword: vdp_write_control_longword(vram_byte_address),
        words: core::array::from_fn(|index| source_words[index]),
    })
}

#[allow(non_snake_case)]
pub fn title_panel_load_plan_sub_1D944(
    panel_selector_with_flags: u16,
    decoded_words: &[u16],
) -> Result<TitlePanelLoadPlanSub1D944> {
    // ASM sub_1D944 (kid.asm:33220): strip bits 15 and 14 into
    // `($FFFFFB5C).w`/`($FFFFFB5E).w`, use the remaining low selector bits to
    // choose one of the seven Enigma tilemaps at `off_1D936`, decode it to
    // `Decompression_Buffer` with base tile `$813E` unless only bit 15 was
    // set, then call sub_1D990 for rows `d7=13..0`.
    const ASSET_LABELS: [&str; 7] = [
        "unk_2E39A",
        "unk_2E436",
        "unk_2E4D6",
        "unk_2E582",
        "unk_2E64A",
        "unk_2EA26",
        "unk_2EA5E",
    ];

    let panel_flag_fb5c = if panel_selector_with_flags & 0x8000 != 0 {
        0xFF
    } else {
        0x00
    };
    let panel_flag_fb5e = if panel_selector_with_flags & 0x4000 != 0 {
        0xFF
    } else {
        0x00
    };
    let selector = (panel_selector_with_flags & 0x3FFF) as usize;
    let asset_label = *ASSET_LABELS.get(selector).ok_or_else(|| {
        anyhow!(
            "sub_1D944 selector {selector:#06X} is outside off_1D936's {} entries",
            ASSET_LABELS.len()
        )
    })?;
    let vram_word_address = if panel_flag_fb5c != 0 && panel_flag_fb5e == 0 {
        0x22DC
    } else {
        0x813E
    };

    Ok(TitlePanelLoadPlanSub1D944 {
        tilemap_decode: SymbolicEnigmaTilemapDecodeSub1D944 {
            asset_label,
            vram_word_address,
            panel_flag_fb5c,
            panel_flag_fb5e,
        },
        line_uploads: core::array::from_fn(|index| {
            let row_index = 13 - index as u16;
            title_panel_line_upload_sub_1D990(decoded_words, row_index, panel_flag_fb5c)
                .expect("sub_1D944 should already have validated the decoded row span")
        }),
    })
}

#[allow(non_snake_case)]
pub fn vdp_data_fill_sub_1DA62() -> VdpDataFillSub1DA62 {
    // ASM sub_1DA62 (kid.asm:33336): write the constant word `$94` to the VDP
    // data port for `$7FF+1` iterations of the `dbf` loop.
    VdpDataFillSub1DA62 {
        word_count: 0x0800,
        fill_word: 0x0094,
    }
}

#[allow(non_snake_case)]
pub fn vram_zero_fill_sub_1DA72() -> VramZeroFillSub1DA72 {
    // ASM sub_1DA72 (kid.asm:33350): point the VDP at VRAM byte `$1280`, then
    // stream 16 zero words via `moveq #$F,d0` / `dbf d0`.
    VramZeroFillSub1DA72 {
        vram_byte_address: 0x1280,
        vdp_control_longword: vdp_write_control_longword(0x1280),
        words: [0; 16],
    }
}

#[allow(non_snake_case)]
pub fn level_layout_address_to_cell_sub_1F8E(
    level_layout_address: u16,
    level_layout_base_address: u16,
    level_width_tiles: u16,
) -> LevelLayoutCellCoordsSub1F8E {
    // ASM sub_1F8E (kid.asm:3769): subtract `Level_Layout` from the caller's
    // row-stride address, divide that byte offset by `Level_width_tiles`, keep
    // the quotient as the row index, then halve the remainder to recover the
    // 2-byte cell column.
    assert_ne!(
        level_width_tiles, 0,
        "sub_1F8E expects a non-zero Level_width_tiles row stride",
    );

    let byte_offset = level_layout_address.wrapping_sub(level_layout_base_address);
    LevelLayoutCellCoordsSub1F8E {
        row_index: byte_offset / level_width_tiles,
        column_index: (byte_offset % level_width_tiles) >> 1,
    }
}

#[allow(non_snake_case)]
pub fn end_sequence_text_render_plan_sub_1D080(
    text_bytes: &[u8],
) -> Result<EndSequenceTextRenderPlanSub1D080> {
    // ASM sub_1D080 (kid.asm:32448): read `(x,y)` from `a4`, convert them to a
    // VRAM byte address at `((y+9)*$80 + x*2) - $2000`, then for each
    // non-zero ASCII letter write one glyph tile word and present three
    // display-refresh frames before advancing the destination by one plane row.
    let (&x, rest) = text_bytes
        .split_first()
        .context("sub_1D080 needs an x coordinate byte before the glyph stream")?;
    let (&y, glyph_bytes) = rest
        .split_first()
        .context("sub_1D080 needs a y coordinate byte before the glyph stream")?;

    let base_vram_byte_address = u16::from(y)
        .wrapping_add(9)
        .wrapping_mul(0x0080)
        .wrapping_add(u16::from(x).wrapping_mul(2))
        .wrapping_sub(0x2000);

    let glyph_uploads = glyph_bytes
        .iter()
        .copied()
        .take_while(|&byte| byte != 0)
        .enumerate()
        .map(|(index, glyph)| {
            let vram_byte_address = base_vram_byte_address.wrapping_add((index as u16) * 0x0080);
            EndSequenceGlyphUploadSub1D080 {
                vram_byte_address,
                vdp_control_longword: vdp_write_control_longword(vram_byte_address),
                glyph_tile_word: u16::from(glyph).wrapping_sub(0x41).wrapping_sub(0x1B24),
            }
        })
        .collect();

    Ok(EndSequenceTextRenderPlanSub1D080 {
        glyph_uploads,
        frames_per_glyph: 3,
    })
}

#[allow(non_snake_case)]
pub fn end_sequence_holo_background_load_plan_sub_1D0EC()
-> EndSequenceHoloBackgroundLoadPlanSub1D0EC {
    // ASM sub_1D0EC (kid.asm:32496): decompress `ArtComp_C65A_HoloBG` to RAM
    // at base tile `$1780` using the fixed `unk_1D118` remap bytes, then
    // decode the paired Enigma tilemap `MapEni_CC0E` to VRAM word `$80BC`.
    EndSequenceHoloBackgroundLoadPlanSub1D0EC {
        art_decode: SymbolicRamAssetDecodeSub1D0EC {
            vram_word_address: 0x1780,
            asset_label: "ArtComp_C65A_HoloBG",
            remap_bytes: [
                0x08, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0A, 0x0B, 0x0C, 0x0D,
                0x0E, 0x0F,
            ],
        },
        tilemap_load: SymbolicEnigmaTilemapLoadSub1B2A4 {
            vram_word_address: 0x80BC,
            asset_label: "MapEni_CC0E",
        },
    }
}

#[allow(non_snake_case)]
pub fn title_screen_character_load_plan_sub_1BF24(
    frame_index: u16,
    region_byte: u8,
) -> Result<TitleScreenCharacterLoadPlanSub1BF24> {
    // ASM sub_1BF24 (kid.asm:30740): select one of eight title-screen
    // character tilemaps, switch the non-zero frames between the Red Stealth /
    // Maniaxe / Juggernaut banks via ($FFFFFC82).w, decode the chosen Enigma
    // tilemap with base tile `$20C1` for frame 0 or `$210F` otherwise, fill
    // the 18x20 staging plane with `$20BC`, then blit it through sub_1C5D0 at
    // plane cell `(x,y)=(#$15,#1)`.
    if frame_index > 7 {
        bail!("sub_1BF24 expects a title-screen frame index between 0 and 7, got {frame_index}");
    }

    const DEFAULT_FRAME_LABELS: [&str; 8] = [
        "unk_2F7D0",
        "unk_2EACE",
        "unk_2EB32",
        "unk_2EBD4",
        "unk_2EC86",
        "unk_2ED62",
        "unk_2EE54",
        "unk_2EF2C",
    ];
    const REGION_1_FRAME_LABELS: [&str; 7] = [
        "unk_2F306",
        "unk_2F374",
        "unk_2F41E",
        "unk_2F4D0",
        "unk_2F592",
        "unk_2F674",
        "unk_2F76A",
    ];
    const REGION_2_FRAME_LABELS: [&str; 7] = [
        "unk_2EF92",
        "unk_2EFFE",
        "unk_2F09C",
        "unk_2F124",
        "unk_2F1B0",
        "unk_2F242",
        "unk_2F2D0",
    ];

    let region_variant = TitleScreenRegionVariantSub1B2A4::from_region_byte(region_byte);
    let asset_label = if frame_index == 0 {
        DEFAULT_FRAME_LABELS[0]
    } else {
        let bank_index = usize::from(frame_index as u8 - 1);
        match region_variant {
            TitleScreenRegionVariantSub1B2A4::Default => {
                DEFAULT_FRAME_LABELS[usize::from(frame_index as u8)]
            }
            TitleScreenRegionVariantSub1B2A4::Variant1 => REGION_1_FRAME_LABELS[bank_index],
            TitleScreenRegionVariantSub1B2A4::Variant2 => REGION_2_FRAME_LABELS[bank_index],
        }
    };

    Ok(TitleScreenCharacterLoadPlanSub1BF24 {
        region_variant,
        frame_index,
        tilemap_load: SymbolicEnigmaTilemapLoadSub1BF24 {
            vram_word_address: if frame_index == 0 { 0x20C1 } else { 0x210F },
            asset_label,
            width_words: 18,
            height_words: 20,
        },
        plane_upload: PlaneRectUploadSub1BF24 {
            plane_cell_x: 0x15,
            plane_cell_y: 0x01,
            width_words: 18,
            height_words: 20,
            fill_word: 0x20BC,
        },
    })
}

#[allow(non_snake_case)]
pub fn intro_sequence_5_frame_load_plan_sub_1C034(
    frame_index: u16,
) -> Result<IntroSequence5FrameLoadPlanSub1C034> {
    // ASM sub_1C034 (kid.asm:30849): select one of eight intro-sequence-5
    // Enigma tilemaps from `off_1C126`, decode it with base tile `$A340`,
    // stage it inside a 16x25 word buffer with the table's byte padding, then
    // stream 25 VDP rows starting at VRAM word `$A340` with the fixed
    // 15/15/..../16/16/15/..../15 row shape from loc_1C09E/loc_1C0B8/loc_1C0D0.
    const FRAME_TABLE: [(&str, u16, u16, u16); 8] = [
        ("unk_2F856", 0x000C, 0x0018, 0x0006),
        ("unk_2F882", 0x000C, 0x0018, 0x0006),
        ("unk_2F8D2", 0x000C, 0x0019, 0x0006),
        ("unk_2F94E", 0x000E, 0x0019, 0x0004),
        ("unk_2F9DE", 0x000F, 0x0019, 0x0000),
        ("unk_2FAA8", 0x000D, 0x0019, 0x0004),
        ("unk_2FC3A", 0x000D, 0x0019, 0x0004),
        ("unk_2FCE4", 0x000D, 0x0019, 0x0004),
    ];

    let Some(&(asset_label, width_words, height_words, left_padding_bytes)) =
        FRAME_TABLE.get(frame_index as usize)
    else {
        bail!(
            "sub_1C034 expects an intro-sequence-5 frame index between 0 and 7, got {frame_index}"
        );
    };

    let vram_rows = core::array::from_fn(|row_index| {
        let row_index_u16 =
            u16::try_from(row_index).expect("sub_1C034 row indices should fit into u16");
        let word_count = if (8..10).contains(&row_index) { 16 } else { 15 };
        VramRowWriteSub1C034 {
            vram_word_address: 0xA340 + row_index_u16 * 0x40,
            source_word_offset: row_index_u16 * 16,
            word_count,
        }
    });

    Ok(IntroSequence5FrameLoadPlanSub1C034 {
        frame_index,
        tilemap_load: SymbolicEnigmaTilemapLoadSub1C034 {
            vram_word_address: 0xA340,
            asset_label,
            width_words,
            height_words,
            left_padding_words: left_padding_bytes / 2,
        },
        vram_rows,
    })
}

#[allow(non_snake_case)]
pub fn intro_sequence_5_panel_load_plan_sub_1C0EE(
    plane_cell_x: u16,
    plane_cell_y: u16,
) -> IntroSequence5PanelLoadPlanSub1C0EE {
    // ASM sub_1C0EE + loc_1C104 (kid.asm:30926): decode `unk_2FDA6` with base
    // tile `$A340`, then hand the resulting 10x10 block to sub_1C5D0 at the
    // caller-supplied plane cell in d4/d5.
    IntroSequence5PanelLoadPlanSub1C0EE {
        tilemap_load: SymbolicEnigmaTilemapLoadSub1C0EE {
            vram_word_address: 0xA340,
            asset_label: "unk_2FDA6",
            width_words: 10,
            height_words: 10,
        },
        plane_upload: PlaneRectUploadSub1C0EE {
            plane_cell_x,
            plane_cell_y,
            width_words: 10,
            height_words: 10,
        },
    }
}

#[allow(non_snake_case)]
pub fn intro_sequence_5_blank_panel_load_plan_sub_1C112(
    plane_cell_x: u16,
    plane_cell_y: u16,
) -> IntroSequence5BlankPanelLoadPlanSub1C112 {
    // ASM sub_1C112 (kid.asm:30943): clear `$5A` words in
    // `Decompression_Buffer`, then branch into loc_1C104 so sub_1C5D0 blits
    // the resulting blank 10x10 panel at the caller-supplied plane cell.
    IntroSequence5BlankPanelLoadPlanSub1C112 {
        cleared_word_count: 0x005A,
        plane_upload: PlaneRectUploadSub1C0EE {
            plane_cell_x,
            plane_cell_y,
            width_words: 10,
            height_words: 10,
        },
    }
}

#[allow(non_snake_case)]
pub fn level_layout_xor_vram_upload_sub_1C180(
    level_layout: &[u8],
    shadow_layout: &[u8],
) -> Result<LevelLayoutXorVramUploadSub1C180> {
    // ASM sub_1C180 (kid.asm:30995): stream `$2C0` longwords to VRAM `$1780`,
    // each word pair being `Level_Layout xor $FFFFB152` in big-endian longword
    // order.
    const LONGWORD_COUNT: usize = 0x02C0;
    const BYTE_COUNT: usize = LONGWORD_COUNT * 4;

    let level_layout = level_layout.get(..BYTE_COUNT).ok_or_else(|| {
        anyhow!(
            "sub_1C180 needs {BYTE_COUNT:#06X} bytes of Level_Layout but only received {:#06X}",
            level_layout.len()
        )
    })?;
    let shadow_layout = shadow_layout.get(..BYTE_COUNT).ok_or_else(|| {
        anyhow!(
            "sub_1C180 needs {BYTE_COUNT:#06X} bytes of shadow layout but only received {:#06X}",
            shadow_layout.len()
        )
    })?;

    let longwords = core::array::from_fn(|index| {
        let offset = index * 4;
        let level_longword = u32::from_be_bytes([
            level_layout[offset],
            level_layout[offset + 1],
            level_layout[offset + 2],
            level_layout[offset + 3],
        ]);
        let shadow_longword = u32::from_be_bytes([
            shadow_layout[offset],
            shadow_layout[offset + 1],
            shadow_layout[offset + 2],
            shadow_layout[offset + 3],
        ]);
        level_longword ^ shadow_longword
    });

    Ok(LevelLayoutXorVramUploadSub1C180 {
        vram_word_address: 0x1780,
        longwords,
    })
}

#[allow(non_snake_case)]
fn mapped_longword_sub_3034A(
    cell: u16,
    row_half_word_offset: usize,
    theme_mappings: &[u16],
    block_mappings: &[u16],
) -> Result<u32> {
    let tile_index = usize::from(cell & 0x00FF);
    let base = tile_index
        .checked_mul(4)
        .and_then(|base| base.checked_add(row_half_word_offset))
        .context("sub_3034A tile mapping offset overflowed")?;
    let mappings = if cell & 0x8000 != 0 {
        block_mappings
    } else {
        theme_mappings
    };
    let (&word_0, &word_1) = mappings
        .get(base)
        .zip(mappings.get(base + 1))
        .context("sub_3034A needs two theme/block mapping words for the selected tile row")?;
    let priority_mask = if cell & 0x0100 != 0 { 0x8000 } else { 0 };
    Ok((u32::from(word_0 | priority_mask) << 16) | u32::from(word_1 | priority_mask))
}

#[allow(non_snake_case)]
fn mapped_word_pair_sub_302F6(
    cell: u16,
    column_half_word_offset: usize,
    theme_mappings: &[u16],
    block_mappings: &[u16],
) -> Result<[u16; 2]> {
    let tile_index = usize::from(cell & 0x00FF);
    let base = tile_index
        .checked_mul(4)
        .and_then(|base| base.checked_add(column_half_word_offset))
        .context("sub_302F6 tile mapping offset overflowed")?;
    let mappings = if cell & 0x8000 != 0 {
        block_mappings
    } else {
        theme_mappings
    };
    let (&word_0, &word_1) = mappings
        .get(base)
        .zip(mappings.get(base + 2))
        .context("sub_302F6 needs two theme/block mapping words for the selected tile column")?;
    let priority_mask = if cell & 0x0100 != 0 { 0x8000 } else { 0 };
    Ok([word_0 | priority_mask, word_1 | priority_mask])
}

fn plane_row_word_address_sub_30194(tile_row: u16, tile_pair_column: u16) -> u16 {
    ((tile_row & 0x001F) << 6).wrapping_add((tile_pair_column & 0x001F) << 1)
}

fn plane_column_word_address_sub_30194(tile_row_even: u16, tile_column: u16) -> u16 {
    ((tile_row_even & 0x001F) << 6).wrapping_add(tile_column & 0x003F)
}

#[allow(non_snake_case)]
fn longwords_from_bytes_sub_303BA(
    bytes: &[u8],
    byte_offset: usize,
    count: usize,
) -> Result<Vec<u32>> {
    let byte_len = count
        .checked_mul(4)
        .context("sub_303BA longword count overflowed")?;
    let end = byte_offset
        .checked_add(byte_len)
        .context("sub_303BA byte range overflowed")?;
    let slice = bytes
        .get(byte_offset..end)
        .context("sub_303BA source bytes are too short for the selected background row")?;

    Ok(slice
        .chunks_exact(4)
        .map(|chunk| u32::from_be_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]))
        .collect())
}

#[allow(non_snake_case)]
pub fn background_row_upload_sub_303BA(
    camera_y_pos: i16,
    previous_camera_chunk: i16,
    map_header_background_rows: &[u8],
    background_is_desert: bool,
    background_layer_0: &[u8],
    background_layer_1: &[u8],
) -> Result<Option<BackgroundRowUploadSub303BA>> {
    // ASM sub_303BA (kid.asm:34007): when `Camera_Y_pos>>5` changes, stream
    // one 32px background row to VRAM. Downward movement uploads row
    // `current+#$1C`; upward movement uploads `current`. Non-Desert themes read
    // row selectors from `1(a0,d0.w)`, Desert adds one extra byte of bias, and
    // row selectors `<#$1C` stream 16 longwords from `$FFFF87B2+row*64` then
    // another 16 from `+$700`, while selectors `>=#$1C` stream both halves from
    // `$FFFF8EB2+row*64`.
    let current_camera_chunk = camera_y_pos >> 5;
    if current_camera_chunk == previous_camera_chunk {
        return Ok(None);
    }

    ensure!(
        current_camera_chunk >= 0,
        "sub_303BA expects a non-negative camera row chunk"
    );

    let streamed_chunk = if current_camera_chunk > previous_camera_chunk {
        current_camera_chunk.wrapping_add(0x001C)
    } else {
        current_camera_chunk
    };
    ensure!(
        streamed_chunk >= 0,
        "sub_303BA expects a non-negative streamed row chunk"
    );

    let row_selector_index = usize::try_from(streamed_chunk)
        .context("sub_303BA streamed row chunk should fit into usize")?
        .checked_add(1 + usize::from(background_is_desert))
        .context("sub_303BA row-selector index overflowed")?;
    let row_selector = usize::from(
        *map_header_background_rows
            .get(row_selector_index)
            .context("sub_303BA row-selector table is missing the streamed chunk")?,
    );
    let row_byte_offset = row_selector
        .checked_mul(64)
        .context("sub_303BA row byte offset overflowed")?;

    let (source_upper, source_lower) = if row_selector < 0x001C {
        (
            background_layer_0,
            background_layer_0
                .get(0x0700..)
                .context("sub_303BA layer-0 background bytes are missing the +$700 second half")?,
        )
    } else {
        (background_layer_1, background_layer_1)
    };

    Ok(Some(BackgroundRowUploadSub303BA {
        vram_word_address: (((streamed_chunk as u16) & 0x001F) << 7) | 0x6000,
        upper_longwords: longwords_from_bytes_sub_303BA(source_upper, row_byte_offset, 16)?,
        lower_longwords: longwords_from_bytes_sub_303BA(source_lower, row_byte_offset, 16)?,
    }))
}

#[allow(non_snake_case)]
pub fn background_scroll_update_sub_304B4(
    current_scroll_pixels: i16,
    background_width: u16,
    moved_left: bool,
) -> Result<BackgroundScrollUpdateSub304B4> {
    // ASM sub_304B4 (kid.asm:34134): convert the new layer scroll position at
    // `4(a0)` into a wrapped background-buffer column and a VDP word offset.
    // Leftward movement uses `current>>3`; rightward movement uses
    // `(current>>3)+#$28` so the newly exposed column comes from the far edge
    // of the 40-cell viewport. The buffer column then wraps by repeatedly
    // subtracting `Background_width`, while the VDP word offset keeps the
    // pre-wrap value masked to 64 columns.
    ensure!(
        background_width != 0,
        "sub_304B4 needs a non-zero Background_width"
    );

    let mut unwrapped_column = current_scroll_pixels >> 3;
    if !moved_left {
        unwrapped_column = unwrapped_column.wrapping_add(0x0028);
    }

    let mut wrapped_column = unwrapped_column;
    let background_width_i16 = i16::try_from(background_width)
        .context("sub_304B4 Background_width should fit into i16")?;
    while wrapped_column >= background_width_i16 {
        wrapped_column = wrapped_column.wrapping_sub(background_width_i16);
    }

    Ok(BackgroundScrollUpdateSub304B4 {
        background_buffer_column: wrapped_column as u16,
        vram_word_offset: (((unwrapped_column as u16) & 0x003F) << 1),
    })
}

#[allow(non_snake_case)]
fn background_tile_word_sub_30744(background_chunk_id: u8) -> u16 {
    // ASM sub_30744 reads `unk_309B8(background_chunk_id*2)`, which is the
    // contiguous word table `$2780..$27FF`.
    0x2780_u16.wrapping_add(u16::from(background_chunk_id))
}

#[allow(non_snake_case)]
pub fn background_tile_uploads_sub_30744(
    camera_y_pos: i16,
    background_width: u16,
    background_row_offsets: &[u16],
    bgscroll_profile: &[u8],
    horiz_scroll_entries: &[BackgroundScrollStateSub30744],
    background_chunk_buffer: &[u8],
) -> Result<Vec<BackgroundTileUploadSub30744>> {
    // ASM sub_30744 (kid.asm:34563): starting at row `Camera_Y_pos>>5`, walk
    // up to 29 bgscroll bytes, stop on `$FF`, skip negative `Horiz_Scroll_Data`
    // entries, and otherwise emit one background tile upload per row using the
    // wrapped VDP row address and the chunk-id lookup from `unk_309B8`.
    ensure!(
        background_width != 0,
        "sub_30744 needs a non-zero Background_width"
    );
    ensure!(
        camera_y_pos >= 0,
        "sub_30744 expects a non-negative Camera_Y_pos"
    );

    let camera_chunk = usize::try_from(camera_y_pos >> 5)
        .context("sub_30744 camera row chunk should fit into usize")?;
    let mut background_row_offset = usize::from(
        *background_row_offsets
            .get(camera_chunk)
            .context("sub_30744 background row-offset table is missing the camera chunk")?,
    );
    let background_width = usize::from(background_width);

    let mut uploads = Vec::new();
    for row_index in 0..=0x001C_u16 {
        let entry_offset = *bgscroll_profile
            .get(camera_chunk + usize::from(row_index))
            .context("sub_30744 bgscroll profile ended before the 29-row window")?;
        if entry_offset == 0xFF {
            break;
        }
        ensure!(
            entry_offset % 8 == 0,
            "sub_30744 bgscroll entries should point at 8-byte Horiz_Scroll_Data slots"
        );

        let entry = *horiz_scroll_entries
            .get(usize::from(entry_offset / 8))
            .context("sub_30744 bgscroll entry points past Horiz_Scroll_Data")?;
        if let Some(background_buffer_column) = entry.background_buffer_column {
            let source_index = background_row_offset
                .checked_add(usize::from(background_buffer_column))
                .context("sub_30744 background source index overflowed")?;
            let background_chunk_id = *background_chunk_buffer
                .get(source_index)
                .context("sub_30744 background chunk buffer is missing the selected source byte")?;
            uploads.push(BackgroundTileUploadSub30744 {
                vram_word_address: 0x6000
                    | (((camera_chunk as u16).wrapping_add(row_index) & 0x001F) << 7)
                    | entry.vram_word_offset,
                tile_word: background_tile_word_sub_30744(background_chunk_id),
                background_chunk_id,
            });
        }

        background_row_offset = background_row_offset
            .checked_add(background_width)
            .context("sub_30744 row advance overflowed")?;
    }

    Ok(uploads)
}

#[allow(non_snake_case)]
pub fn background_row_uploads_sub_307D8(
    camera_y_pos: i16,
    previous_camera_row_chunk: i16,
    background_width: u16,
    background_row_offsets: &[u16],
    bgscroll_profile: &[u8],
    horiz_scroll_entries: &[BackgroundScrollStateSub30744],
    background_chunk_buffer: &[u8],
) -> Result<Vec<BackgroundRowUploadSegmentSub307D8>> {
    // ASM sub_307D8 (kid.asm:34625): when `Camera_Y_pos>>5` changes, upload the
    // newly exposed 40-cell background row. Upward scroll streams the current
    // row chunk; downward scroll streams `current+#$1C`. The routine then walks
    // 40 chunk ids from the selected background row, splitting the upload any
    // time either the background row source wraps by `Background_width` or the
    // plane row address wraps back to its `$6000 | row<<7` base.
    ensure!(
        background_width != 0,
        "sub_307D8 needs a non-zero Background_width"
    );
    ensure!(
        camera_y_pos >= 0,
        "sub_307D8 expects a non-negative Camera_Y_pos"
    );

    let current_camera_row_chunk = camera_y_pos >> 5;
    if current_camera_row_chunk == previous_camera_row_chunk {
        return Ok(Vec::new());
    }

    let streamed_row_chunk = if previous_camera_row_chunk > current_camera_row_chunk {
        current_camera_row_chunk
    } else {
        current_camera_row_chunk.wrapping_add(0x001C)
    };
    ensure!(
        streamed_row_chunk >= 0,
        "sub_307D8 streamed row chunk should stay non-negative"
    );
    let streamed_row_chunk = usize::try_from(streamed_row_chunk)
        .context("sub_307D8 streamed row chunk should fit into usize")?;

    let entry_offset = *bgscroll_profile
        .get(streamed_row_chunk)
        .context("sub_307D8 bgscroll profile is missing the streamed row chunk")?;
    if entry_offset == 0xFF {
        return Ok(Vec::new());
    }
    ensure!(
        entry_offset % 8 == 0,
        "sub_307D8 bgscroll entries should point at 8-byte Horiz_Scroll_Data slots"
    );
    let entry = *horiz_scroll_entries
        .get(usize::from(entry_offset / 8))
        .context("sub_307D8 bgscroll entry points past Horiz_Scroll_Data")?;
    let background_buffer_column = entry
        .background_buffer_column
        .context("sub_307D8 needs a visible Horiz_Scroll_Data entry")?;

    let row_offset = usize::from(
        *background_row_offsets
            .get(streamed_row_chunk)
            .context("sub_307D8 background row-offset table is missing the streamed row")?,
    );
    let background_width = usize::from(background_width);
    let source_row_start = row_offset
        .checked_add(usize::from(background_buffer_column))
        .context("sub_307D8 background source row start overflowed")?;
    let row_base_vram_word_address =
        0x6000 | (((u16::try_from(streamed_row_chunk).unwrap_or_default()) & 0x001F) << 7);
    let mut current_vram_word_address = row_base_vram_word_address | entry.vram_word_offset;
    let mut current_source_index = source_row_start;
    let mut source_remaining_until_wrap = background_width
        .checked_sub(usize::from(background_buffer_column))
        .context("sub_307D8 wrapped source column exceeded Background_width")?;
    let mut vram_remaining_until_wrap = 0x40_usize
        .checked_sub(usize::from(entry.vram_word_offset >> 1))
        .context("sub_307D8 VDP word offset exceeded the 64-cell plane row")?;
    let mut remaining_cells = 0x28_usize;
    let mut segments = Vec::new();

    while remaining_cells != 0 {
        let segment_len = remaining_cells
            .min(source_remaining_until_wrap)
            .min(vram_remaining_until_wrap);
        ensure!(
            segment_len != 0,
            "sub_307D8 segment length unexpectedly reached zero"
        );

        let mut background_chunk_ids = Vec::with_capacity(segment_len);
        let mut tile_words = Vec::with_capacity(segment_len);
        for offset in 0..segment_len {
            let background_chunk_id = *background_chunk_buffer
                .get(current_source_index + offset)
                .context(
                "sub_307D8 background chunk buffer is missing the streamed source byte",
            )?;
            background_chunk_ids.push(background_chunk_id);
            tile_words.push(background_tile_word_sub_30744(background_chunk_id));
        }
        segments.push(BackgroundRowUploadSegmentSub307D8 {
            vram_word_address: current_vram_word_address,
            tile_words,
            background_chunk_ids,
        });

        remaining_cells -= segment_len;
        current_source_index += segment_len;
        source_remaining_until_wrap -= segment_len;
        vram_remaining_until_wrap -= segment_len;

        if remaining_cells == 0 {
            break;
        }
        if source_remaining_until_wrap == 0 {
            current_source_index = row_offset;
            source_remaining_until_wrap = background_width;
        }
        if vram_remaining_until_wrap == 0 {
            current_vram_word_address = row_base_vram_word_address;
            vram_remaining_until_wrap = 0x40;
        } else {
            current_vram_word_address =
                current_vram_word_address.wrapping_add(u16::try_from(segment_len * 2).unwrap_or(0));
        }
    }

    Ok(segments)
}

#[allow(non_snake_case)]
pub fn level_plane_scroll_update_plan_sub_30194(
    inputs: LevelPlaneScrollUpdateInputsSub30194<'_>,
) -> Result<LevelPlaneScrollUpdatePlanSub30194> {
    // ASM sub_30194 (kid.asm:33745): when the camera crosses an 8px tile
    // boundary, stream the newly exposed foreground strip from the row-pointer
    // table at `$FFFF4A04`, using theme mappings for non-negative cells and
    // Block_Mappings for negative cells. Horizontal updates write 21 longwords
    // via sub_3034A, while vertical updates write 15 two-word cell pairs via
    // sub_302F6, splitting the upload when the 64x32 plane wrap is crossed.
    ensure!(
        inputs.level_width_tiles != 0 && inputs.level_width_tiles.is_multiple_of(2),
        "sub_30194 needs a non-zero even Level_width_tiles row stride",
    );

    let level_width_cells = usize::from(inputs.level_width_tiles / 2);
    let camera_column_tiles = inputs.camera_x_pos >> 3;
    let camera_row_tiles = inputs.camera_y_pos >> 3;
    let previous_column_tiles = inputs.previous_camera_x_pos >> 3;
    let previous_row_tiles = inputs.previous_camera_y_pos >> 3;

    let mut horizontal_uploads = Vec::new();
    if camera_row_tiles != previous_row_tiles {
        let mut streamed_row_tiles = camera_row_tiles;
        if camera_row_tiles >= previous_row_tiles {
            streamed_row_tiles = streamed_row_tiles.wrapping_add(0x001C);
            if streamed_row_tiles >= inputs.level_height_tiles as i16 {
                streamed_row_tiles = -1;
            }
        }

        if streamed_row_tiles >= 0 {
            let streamed_row_tiles_u16 = u16::try_from(streamed_row_tiles)
                .expect("sub_30194 keeps streamed rows non-negative");
            let row_pair_index = usize::from(streamed_row_tiles_u16 >> 1);
            let row_half_word_offset = if streamed_row_tiles_u16 & 1 != 0 {
                2
            } else {
                0
            };
            let row_start = usize::from(
                *inputs
                    .level_layout_row_offsets_cells
                    .get(row_pair_index)
                    .context("sub_30194 row-pointer table is missing the streamed row pair")?,
            );
            let source_cell_offset = row_start
                .checked_add(usize::from(
                    u16::try_from(camera_column_tiles >> 1)
                        .context("sub_30194 expects non-negative camera columns")?,
                ))
                .context("sub_30194 horizontal source cell offset overflowed")?;
            let plane_column_tiles = u16::try_from(camera_column_tiles)
                .context("sub_30194 expects non-negative camera columns")?
                & 0x001F;
            let first_count = if plane_column_tiles < 0x000C {
                21
            } else {
                usize::from(0x0020_u16.wrapping_sub(plane_column_tiles))
            };
            let second_count = 21usize.saturating_sub(first_count);

            let mut first_longwords = Vec::with_capacity(first_count);
            for source_index in 0..first_count {
                let &cell = inputs
                    .level_layout_cells
                    .get(source_cell_offset + source_index)
                    .context("sub_30194 horizontal strip ran past Level_Layout cells")?;
                first_longwords.push(mapped_longword_sub_3034A(
                    cell,
                    row_half_word_offset,
                    inputs.theme_mappings,
                    inputs.block_mappings,
                )?);
            }
            horizontal_uploads.push(PlaneLongwordStripUploadSub30194 {
                vram_word_address: plane_row_word_address_sub_30194(
                    streamed_row_tiles_u16,
                    u16::try_from(camera_column_tiles >> 1)
                        .context("sub_30194 expects non-negative camera columns")?,
                ),
                longwords: first_longwords,
            });

            if second_count > 0 {
                let mut second_longwords = Vec::with_capacity(second_count);
                for source_index in first_count..(first_count + second_count) {
                    let &cell = inputs
                        .level_layout_cells
                        .get(source_cell_offset + source_index)
                        .context(
                            "sub_30194 wrapped horizontal strip ran past Level_Layout cells",
                        )?;
                    second_longwords.push(mapped_longword_sub_3034A(
                        cell,
                        row_half_word_offset,
                        inputs.theme_mappings,
                        inputs.block_mappings,
                    )?);
                }
                horizontal_uploads.push(PlaneLongwordStripUploadSub30194 {
                    vram_word_address: plane_row_word_address_sub_30194(streamed_row_tiles_u16, 0),
                    longwords: second_longwords,
                });
            }
        }
    }

    let mut vertical_uploads = Vec::new();
    if camera_column_tiles != previous_column_tiles {
        let mut streamed_column_tiles = camera_column_tiles;
        if camera_column_tiles >= previous_column_tiles {
            streamed_column_tiles = streamed_column_tiles.wrapping_add(0x0028);
            if streamed_column_tiles >= inputs.level_width_tiles as i16 {
                streamed_column_tiles = -1;
            }
        }

        if streamed_column_tiles >= 0 {
            let streamed_column_tiles_u16 = u16::try_from(streamed_column_tiles)
                .expect("sub_30194 keeps streamed columns non-negative");
            let column_half_word_offset = if streamed_column_tiles_u16 & 1 != 0 {
                1
            } else {
                0
            };
            let source_column = usize::from(streamed_column_tiles_u16 >> 1);
            let source_row_even_tiles = u16::try_from(camera_row_tiles & !1)
                .context("sub_30194 expects non-negative camera rows")?;
            let row_pair_index = usize::from(source_row_even_tiles >> 1);
            let row_start = usize::from(
                *inputs
                    .level_layout_row_offsets_cells
                    .get(row_pair_index)
                    .context(
                        "sub_30194 row-pointer table is missing the streamed column start row",
                    )?,
            );
            let source_cell_offset = row_start
                .checked_add(source_column)
                .context("sub_30194 vertical source cell offset overflowed")?;
            let plane_row_pair = (u16::try_from(camera_row_tiles)
                .context("sub_30194 expects non-negative camera rows")?
                >> 1)
                & 0x000F;
            let first_count = if plane_row_pair < 0x0002 {
                15
            } else {
                usize::from(0x0010_u16.wrapping_sub(plane_row_pair))
            };
            let second_count = 15usize.saturating_sub(first_count);

            let mut first_words = Vec::with_capacity(first_count * 2);
            for row_index in 0..first_count {
                let &cell = inputs
                    .level_layout_cells
                    .get(source_cell_offset + row_index * level_width_cells)
                    .context("sub_30194 vertical strip ran past Level_Layout cells")?;
                first_words.extend(mapped_word_pair_sub_302F6(
                    cell,
                    column_half_word_offset,
                    inputs.theme_mappings,
                    inputs.block_mappings,
                )?);
            }
            vertical_uploads.push(PlaneWordStripUploadSub30194 {
                vram_word_address: plane_column_word_address_sub_30194(
                    source_row_even_tiles,
                    streamed_column_tiles_u16,
                ),
                words: first_words,
            });

            if second_count > 0 {
                let mut second_words = Vec::with_capacity(second_count * 2);
                for row_index in first_count..(first_count + second_count) {
                    let &cell = inputs
                        .level_layout_cells
                        .get(source_cell_offset + row_index * level_width_cells)
                        .context("sub_30194 wrapped vertical strip ran past Level_Layout cells")?;
                    second_words.extend(mapped_word_pair_sub_302F6(
                        cell,
                        column_half_word_offset,
                        inputs.theme_mappings,
                        inputs.block_mappings,
                    )?);
                }
                vertical_uploads.push(PlaneWordStripUploadSub30194 {
                    vram_word_address: plane_column_word_address_sub_30194(
                        0,
                        streamed_column_tiles_u16,
                    ),
                    words: second_words,
                });
            }
        }
    }

    Ok(LevelPlaneScrollUpdatePlanSub30194 {
        horizontal_uploads,
        vertical_uploads,
    })
}

#[allow(non_snake_case)]
pub fn mask_level_layout_column_sub_1C1A2(level_layout: &mut [u8], selector: i16) {
    // ASM sub_1C1A2 (kid.asm:31014): increment the signed selector, reject any
    // value outside `0..$3F`, then map that 6-bit column id through
    // `byte_1C1F4`. ASM sub_1C1D2 (kid.asm:31043) applies the resulting nibble
    // mask down 16 rows at a fixed `$20`-byte stride across `Level_Layout`.
    let adjusted_selector = selector.saturating_add(1);
    if !(0..0x40).contains(&adjusted_selector) {
        return;
    }

    let adjusted_selector =
        u16::try_from(adjusted_selector).expect("sub_1C1A2 should keep selectors in range");
    let mut encoded_offset =
        u16::from(COLUMN_MASK_LOOKUP_SUB_1C1A2[usize::from(adjusted_selector >> 2)]);
    if adjusted_selector & 0x0001 != 0 {
        encoded_offset += 0x20;
    }
    if adjusted_selector & 0x0002 != 0 {
        encoded_offset += 0x04;
    }

    apply_level_layout_nibble_mask_sub_1C1D2(level_layout, encoded_offset);
}

#[allow(non_snake_case)]
fn apply_level_layout_nibble_mask_sub_1C1D2(level_layout: &mut [u8], encoded_offset: u16) {
    let byte_mask = if encoded_offset & 0x0001 == 0 {
        0x0F
    } else {
        0xF0
    };
    let base_offset = usize::from(encoded_offset >> 1);

    for row in 0..16usize {
        let offset = base_offset + row * 0x20;
        let Some(cell) = level_layout.get_mut(offset) else {
            break;
        };
        *cell &= byte_mask;
    }
}

#[allow(non_snake_case)]
pub fn thunder_animation_frame_upload_sub_1B41C(
    frame_index: u16,
    art: &[u8],
) -> Result<LongwordVramUploadSub1B41C> {
    // ASM sub_1B41C (kid.asm:29644-29656) multiplies d0 by $80, adds that
    // byte offset to `ArtUnc_2A4D6`, then writes 32 big-endian longwords to
    // VRAM word address $17A0.
    let start = usize::from(frame_index) << 7;
    let end = start
        .checked_add(32 * 4)
        .context("sub_1B41C frame slice overflow")?;
    let bytes = art.get(start..end).ok_or_else(|| {
        anyhow!(
            "sub_1B41C frame {} needs bytes [{start:#06X}..{end:#06X}) but art is only {} bytes",
            frame_index,
            art.len()
        )
    })?;
    let longwords = core::array::from_fn(|index| {
        let offset = index * 4;
        u32::from_be_bytes([
            bytes[offset],
            bytes[offset + 1],
            bytes[offset + 2],
            bytes[offset + 3],
        ])
    });
    Ok(LongwordVramUploadSub1B41C {
        vram_word_address: 0x17A0,
        longwords,
    })
}

#[allow(non_snake_case)]
pub fn title_text_chameleon_tilemap_upload_sub_1B532(
    decoded_words: &[u16],
) -> Result<TitleTextChameleonTilemapUploadSub1B532> {
    // ASM sub_1B532 (kid.asm:29782): clear 8 longwords at VRAM $1780, then copy
    // 28 rows of 64 words from the decoded `unk_2F840` tilemap to VRAM $E000,
    // skipping one source word between rows via `addq.w #2,a4`.
    const ROW_COUNT: usize = 28;
    const SOURCE_STRIDE_WORDS: usize = 65;

    let required_words = ROW_COUNT * SOURCE_STRIDE_WORDS;
    if decoded_words.len() < required_words {
        bail!(
            "sub_1B532 needs at least {required_words} decoded words for 28 source rows, got {}",
            decoded_words.len()
        );
    }

    let rows = core::array::from_fn(|row_index| {
        let row_start = row_index * SOURCE_STRIDE_WORDS;
        core::array::from_fn(|column_index| decoded_words[row_start + column_index])
    });

    Ok(TitleTextChameleonTilemapUploadSub1B532 {
        clear_vram_word_address: 0x1780,
        clear_longword_count: 8,
        tilemap_vram_word_address: 0xE000,
        rows,
    })
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
fn compose_indoor_rubber_block_art_sub_144DA(
    base_art: &[u8],
    rubber_overlay_art: &[u8],
) -> Vec<u8> {
    // ASM sub_144DA (kid.asm:27356) keeps each non-zero nibble from the
    // indoor base block art at $FFFF78B2 and only fills zero nibbles from the
    // decompressed rubber overlay art.
    base_art
        .iter()
        .zip(rubber_overlay_art.iter())
        .map(|(&base, &overlay)| {
            let high = if base & 0xF0 != 0 {
                base & 0xF0
            } else {
                overlay & 0xF0
            };
            let low = if base & 0x0F != 0 {
                base & 0x0F
            } else {
                overlay & 0x0F
            };
            high | low
        })
        .collect()
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
fn vram_read_control_word_sub_14582(vram_word_address: u16) -> u32 {
    // ASM sub_14582 (kid.asm:27442): convert the caller's VRAM word address
    // into a VDP VRAM-read control word, carrying the original address bits
    // 14-15 into the low control word after the longword rotate and clearing
    // the command bits to the VRAM-read form.
    let control_low = (vram_word_address >> 14) & 0x0003;
    let control_high = u32::from(vram_word_address & 0x3FFF) << 16;
    control_high | u32::from(control_low)
}

fn export_decoded_all_levels(manifest: &AssetManifest) -> Result<Vec<DecodedLevelAssets>> {
    let mut results = Vec::new();
    for level in &manifest.levels {
        let has_data = level.resources.foreground.is_some()
            || level.resources.block.is_some()
            || level.resources.background.is_some();
        if !has_data {
            continue;
        }
        let level_id = level.level_id;
        let preview_width = level
            .header_metadata
            .as_ref()
            .map(|h| usize::from(h.width_screens) * 40)
            .unwrap_or(256);
        let output_dir_rel = decoded_level_output_dir(level_id);
        let output_dir = manifest.output_root.join(&output_dir_rel);
        fs::create_dir_all(&output_dir)
            .with_context(|| format!("failed to create {}", output_dir.display()))?;

        let foreground = export_saxman_decoded_asset(
            manifest,
            level.resources.foreground.as_deref(),
            "foreground",
            preview_width,
            level_id,
            &output_dir_rel,
        )?;
        let block = export_raw_level_asset(
            manifest,
            level.resources.block.as_deref(),
            "block",
            preview_width,
            level_id,
            &output_dir_rel,
        )?;
        let background = export_raw_level_asset(
            manifest,
            level.resources.background.as_deref(),
            "background",
            BACKGROUND_PREVIEW_WIDTH_TILES,
            level_id,
            &output_dir_rel,
        )?;

        results.push(DecodedLevelAssets {
            contract_version: DECODED_LEVEL_ASSET_VERSION,
            level_id,
            foreground,
            block,
            background,
        });
    }
    Ok(results)
}

fn export_saxman_decoded_asset(
    manifest: &AssetManifest,
    relative_path: Option<&str>,
    stem: &str,
    preview_width: usize,
    level_id: u8,
    output_dir: &str,
) -> Result<DecodedAssetFile> {
    let relative_path = relative_path
        .ok_or_else(|| anyhow!("level 0x{level_id:02X} is missing {stem} asset path"))?;
    let encoded_path = manifest.output_root.join("level").join(relative_path);
    let decoded_bytes = decode_saxman(
        &fs::read(&encoded_path)
            .with_context(|| format!("failed to read {}", encoded_path.display()))?,
    )?;
    export_decoded_asset_file(
        manifest,
        stem,
        &decoded_bytes,
        preview_width,
        false,
        output_dir,
    )
}

fn export_raw_level_asset(
    manifest: &AssetManifest,
    relative_path: Option<&str>,
    stem: &str,
    preview_width: usize,
    level_id: u8,
    output_dir: &str,
) -> Result<DecodedAssetFile> {
    let relative_path = relative_path
        .ok_or_else(|| anyhow!("level 0x{level_id:02X} is missing {stem} asset path"))?;
    let encoded_path = manifest.output_root.join("level").join(relative_path);
    let raw_bytes = fs::read(&encoded_path)
        .with_context(|| format!("failed to read {}", encoded_path.display()))?;
    export_decoded_asset_file(manifest, stem, &raw_bytes, preview_width, false, output_dir)
}

fn export_decoded_asset_file(
    manifest: &AssetManifest,
    stem: &str,
    decoded_bytes: &[u8],
    preview_width: usize,
    word_preview: bool,
    output_dir: &str,
) -> Result<DecodedAssetFile> {
    let raw_path = manifest
        .output_root
        .join(output_dir)
        .join(format!("{stem}.bin"));
    let preview_path = manifest
        .output_root
        .join(output_dir)
        .join(format!("{stem}.png"));
    fs::write(&raw_path, decoded_bytes)
        .with_context(|| format!("failed to write {}", raw_path.display()))?;
    if word_preview {
        write_word_preview_png(&preview_path, decoded_bytes, preview_width)?;
    } else {
        write_byte_preview_png(&preview_path, decoded_bytes, preview_width)?;
    }

    Ok(DecodedAssetFile {
        relative_path: normalise_relative_path(
            raw_path
                .strip_prefix(&manifest.output_root)
                .expect("decoded asset should stay inside output root"),
        ),
        preview_png_path: normalise_relative_path(
            preview_path
                .strip_prefix(&manifest.output_root)
                .expect("decoded preview should stay inside output root"),
        ),
        size_bytes: decoded_bytes.len() as u64,
        sha1: sha1_hex(decoded_bytes),
    })
}

fn validate_decoded_asset_file(
    manifest: &AssetManifest,
    asset: &DecodedAssetFile,
    label: &str,
) -> Result<()> {
    let asset_path = manifest.output_root.join(&asset.relative_path);
    let preview_path = manifest.output_root.join(&asset.preview_png_path);
    if !asset_path.is_file() {
        bail!("decoded {label} asset is missing: {}", asset_path.display());
    }
    if !preview_path.is_file() {
        bail!(
            "decoded {label} preview is missing: {}",
            preview_path.display()
        );
    }
    let bytes = fs::read(&asset_path)
        .with_context(|| format!("failed to read decoded asset {}", asset_path.display()))?;
    if bytes.len() as u64 != asset.size_bytes {
        bail!(
            "decoded Level 1 {label} size mismatch: expected {}, got {}",
            asset.size_bytes,
            bytes.len()
        );
    }
    let actual_sha1 = sha1_hex(&bytes);
    if actual_sha1 != asset.sha1 {
        bail!(
            "decoded Level 1 {label} sha1 mismatch: expected {}, got {}",
            asset.sha1,
            actual_sha1
        );
    }
    Ok(())
}

fn write_byte_preview_png(path: &Path, bytes: &[u8], width: usize) -> Result<()> {
    let width = width.max(1);
    let height = bytes.len().max(1).div_ceil(width);
    let mut image = GrayImage::new(width as u32, height as u32);
    for (index, byte) in bytes.iter().copied().enumerate() {
        let x = (index % width) as u32;
        let y = (index / width) as u32;
        image.put_pixel(x, y, Luma([byte]));
    }
    image
        .save(path)
        .with_context(|| format!("failed to write {}", path.display()))?;
    Ok(())
}

fn write_word_preview_png(path: &Path, bytes: &[u8], width_words: usize) -> Result<()> {
    let words = bytes
        .chunks_exact(2)
        .map(|chunk| u16::from_be_bytes([chunk[0], chunk[1]]))
        .collect::<Vec<_>>();
    let width_words = width_words.max(1);
    let height = words.len().max(1).div_ceil(width_words);
    let mut image = GrayImage::new(width_words as u32, height as u32);
    for (index, value) in words.iter().copied().enumerate() {
        let x = (index % width_words) as u32;
        let y = (index / width_words) as u32;
        image.put_pixel(x, y, Luma([(value & 0x00FF) as u8]));
    }
    image
        .save(path)
        .with_context(|| format!("failed to write {}", path.display()))?;
    Ok(())
}

pub fn decode_saxman(data: &[u8]) -> Result<Vec<u8>> {
    if data.len() < 4 {
        bail!("Saxman payload is too short");
    }
    let input_offset = usize::from(read_u16_be(data, 0)) + 2;
    if input_offset > data.len() {
        bail!(
            "Saxman input offset {} is past payload length {}",
            input_offset,
            data.len()
        );
    }

    let mut bits = BitReader::new(&data[2..input_offset]);
    let mut input_position = input_offset;
    let mut output = Vec::new();

    loop {
        if bits.read_bit()? == 1 {
            output.push(read_input_byte(data, &mut input_position)?);
            continue;
        }

        if bits.read_bit()? == 0 {
            let count = usize::from(bits.read_bit()?) + 2;
            let distance = usize::from(read_input_byte(data, &mut input_position)?);
            if distance == 0 {
                write_zero_fill(&mut output, count.saturating_sub(1));
            } else {
                copy_from_output(&mut output, distance, count)?;
            }
            continue;
        }

        let high_distance = usize::from(bits.read_bits(3)?);
        let count_code = bits.read_bits(2)?;
        let low_distance = usize::from(read_input_byte(data, &mut input_position)?);
        let count = if count_code == 0b11 {
            usize::from(read_input_byte(data, &mut input_position)?)
        } else {
            usize::from(count_code) + 3
        };
        if count_code == 0b11 && count == 0 {
            break;
        }
        if count_code == 0b11 && count < 6 {
            continue;
        }

        let distance = low_distance + (high_distance << 8);
        if distance == 0 {
            write_zero_fill(&mut output, count.saturating_sub(1));
        } else {
            copy_from_output(&mut output, distance, count)?;
        }
    }

    Ok(output)
}

pub fn decode_enigma(data: &[u8], base_tile_word: u16) -> Result<Vec<u16>> {
    let header = parse_enigma_header_eni_dec(data, base_tile_word)?;
    let mut incrementing_value = header.incrementing_value;
    let common_value = header.common_value;
    let mut bits = EnigmaBitReader::new(header.bitstream);
    bits.refill_word_sub_13618()?;
    let mut output = Vec::new();

    loop {
        let (command, count) = bits.read_prefix_eni_dec()?;
        match command {
            0 | 1 => {
                for _ in 0..=count {
                    output.push(incrementing_value);
                    incrementing_value = incrementing_value.wrapping_add(1);
                }
            }
            2 | 3 => {
                output.extend(std::iter::repeat_n(common_value, count + 1));
            }
            4 => {
                let value = read_inline_value_sub_13588(
                    &mut bits,
                    base_tile_word,
                    header.flag_mask,
                    header.packet_length,
                )?;
                output.extend(std::iter::repeat_n(value, count + 1));
            }
            5 => {
                let mut value = read_inline_value_sub_13588(
                    &mut bits,
                    base_tile_word,
                    header.flag_mask,
                    header.packet_length,
                )?;
                for _ in 0..=count {
                    output.push(value);
                    value = value.wrapping_add(1);
                }
            }
            6 => {
                let mut value = read_inline_value_sub_13588(
                    &mut bits,
                    base_tile_word,
                    header.flag_mask,
                    header.packet_length,
                )?;
                for _ in 0..=count {
                    output.push(value);
                    value = value.wrapping_sub(1);
                }
            }
            7 => {
                if count == 0x0F {
                    break;
                }
                for _ in 0..=count {
                    let value = read_inline_value_sub_13588(
                        &mut bits,
                        base_tile_word,
                        header.flag_mask,
                        header.packet_length,
                    )?;
                    output.push(value);
                }
            }
            _ => unreachable!(),
        }
    }

    Ok(output)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct EnigmaHeaderEniDec<'a> {
    packet_length: usize,
    flag_mask: u8,
    incrementing_value: u16,
    common_value: u16,
    bitstream: &'a [u8],
}

fn parse_enigma_header_eni_dec<'a>(
    data: &'a [u8],
    base_tile_word: u16,
) -> Result<EnigmaHeaderEniDec<'a>> {
    // ASM EniDec (kid.asm:25837): byte 0 is packet length, byte 1 selects the
    // enabled high-flag bits for sub_13588, words 1/2 are base-tile-relative
    // incrementing/common seeds, and the next word seeds d5 for loc_134EA.
    if data.len() < 8 {
        bail!("Enigma payload is too short");
    }
    let packet_length = usize::from(data[0]);
    if packet_length == 0 || packet_length > 16 {
        bail!("Enigma packet length must be between 1 and 16 bits");
    }

    Ok(EnigmaHeaderEniDec {
        packet_length,
        flag_mask: data[1] & 0x1F,
        incrementing_value: read_u16_be(data, 2).wrapping_add(base_tile_word),
        common_value: read_u16_be(data, 4).wrapping_add(base_tile_word),
        bitstream: &data[6..],
    })
}

fn read_inline_value_sub_13588(
    bits: &mut EnigmaBitReader<'_>,
    base_tile_word: u16,
    flag_mask: u8,
    packet_length: usize,
) -> Result<u16> {
    // ASM sub_13588 (kid.asm:25973): start from the caller's base tile word,
    // fold in the enabled high-flag bits from d4, then append the packet bits.
    // loc_13602/loc_13614 either tail into loc_13650 or reload d5/d6 directly
    // from the next 16-bit word; this packet path is not a plain sub_13622 call.
    let flags = read_enigma_flags(bits, flag_mask)?;
    let packet_bits = bits.read_inline_packet_sub_13588(packet_length)?;
    Ok(base_tile_word.wrapping_add(flags).wrapping_add(packet_bits))
}

fn read_enigma_flags(bits: &mut EnigmaBitReader<'_>, flag_mask: u8) -> Result<u16> {
    let mut flags = 0_u16;
    for (mask_bit, flag) in [
        (4_u8, 1_u16 << 15),
        (3_u8, 1_u16 << 14),
        (2_u8, 1_u16 << 13),
        (1_u8, 1_u16 << 12),
        (0_u8, 1_u16 << 11),
    ] {
        // ASM sub_13588 loc_13588-loc_135D2 (kid.asm:25973-26016): enabled
        // flag bits consume buffered bits via `subq.w #1,d6` + `btst d6,d5`
        // and do not tail into loc_13650.
        if flag_mask & (1 << mask_bit) != 0 && bits.read_raw_flag_bit_sub_13588()? == 1 {
            flags |= flag;
        }
    }
    Ok(flags)
}

fn read_input_byte(data: &[u8], input_position: &mut usize) -> Result<u8> {
    let byte = *data
        .get(*input_position)
        .ok_or_else(|| anyhow!("compressed payload ended unexpectedly"))?;
    *input_position += 1;
    Ok(byte)
}

fn copy_from_output(output: &mut Vec<u8>, distance: usize, count: usize) -> Result<()> {
    if distance == 0 {
        bail!("back-reference distance cannot be zero");
    }
    for _ in 0..count {
        let value = output
            .len()
            .checked_sub(distance)
            .and_then(|source_index| output.get(source_index).copied())
            .unwrap_or(0);
        output.push(value);
    }
    Ok(())
}

fn write_zero_fill(output: &mut Vec<u8>, count: usize) {
    output.extend(std::iter::repeat_n(0_u8, count));
}

struct BitReader<'a> {
    bytes: &'a [u8],
    bit_index: usize,
}

impl<'a> BitReader<'a> {
    fn new(bytes: &'a [u8]) -> Self {
        Self {
            bytes,
            bit_index: 0,
        }
    }

    fn read_bit(&mut self) -> Result<u8> {
        let byte = *self
            .bytes
            .get(self.bit_index / 8)
            .ok_or_else(|| anyhow!("compressed bitstream ended unexpectedly"))?;
        let shift = 7 - (self.bit_index % 8);
        self.bit_index += 1;
        Ok((byte >> shift) & 1)
    }

    fn read_bits(&mut self, count: usize) -> Result<u16> {
        let mut value = 0_u16;
        for _ in 0..count {
            value = (value << 1) | u16::from(self.read_bit()?);
        }
        Ok(value)
    }
}

struct EnigmaBitReader<'a> {
    bytes: &'a [u8],
    input_position: usize,
    current_word_d5: u16,
    bits_remaining_d6: u16,
}

impl<'a> EnigmaBitReader<'a> {
    fn new(bytes: &'a [u8]) -> Self {
        Self {
            bytes,
            input_position: 0,
            current_word_d5: 0,
            bits_remaining_d6: 0,
        }
    }

    fn refill_word_sub_13618(&mut self) -> Result<()> {
        // ASM sub_13618 (kid.asm:26062): load the next 16-bit big-endian word
        // into d5 and reset d6 to 16 buffered bits.
        let high_byte = read_input_byte(self.bytes, &mut self.input_position)?;
        let low_byte = read_input_byte(self.bytes, &mut self.input_position)?;
        self.current_word_d5 = (u16::from(high_byte) << 8) | u16::from(low_byte);
        self.bits_remaining_d6 = 16;
        Ok(())
    }

    fn read_prefix_eni_dec(&mut self) -> Result<(usize, usize)> {
        // ASM EniDec loc_134EA (kid.asm:25850): inspect the next 7 bits,
        // downgrade codes below `$40` to six consumed bits, then dispatch via
        // `loc_13560(pc,d1.w)`.
        let prefix = self.peek_bits_sub_1363c(7)?;
        let (command, count, consumed_bits) = if prefix < 0x40 {
            (
                usize::from(prefix >> 4),
                usize::from((prefix >> 1) & 0x0F),
                6,
            )
        } else {
            (usize::from(prefix >> 4), usize::from(prefix & 0x0F), 7)
        };
        if command == 7 && count == 0x0F {
            self.consume_bits_loc_13650_allow_terminator_eof(consumed_bits)?;
        } else {
            self.consume_bits_loc_13650(consumed_bits)?;
        }
        Ok((command, count))
    }

    #[cfg_attr(not(test), allow(dead_code))]
    fn read_raw_flag_bit_sub_13588(&mut self) -> Result<u8> {
        if self.bits_remaining_d6 == 0 {
            bail!("Enigma flag read exceeded buffered bits");
        }
        self.bits_remaining_d6 -= 1;
        Ok(((self.current_word_d5 >> self.bits_remaining_d6) & 1) as u8)
    }

    #[cfg_attr(not(test), allow(dead_code))]
    fn read_inline_packet_sub_13588(&mut self, bit_count: usize) -> Result<u16> {
        if bit_count == 0 || bit_count > 16 {
            bail!("Enigma inline packets must be between 1 and 16 bits");
        }
        if self.bits_remaining_d6 == 0 {
            self.refill_word_sub_13618()?;
        }

        let bits_remaining = usize::from(self.bits_remaining_d6);
        let mask = ENIGMA_BIT_MASKS[bit_count - 1];

        if bits_remaining > bit_count {
            let shift = bits_remaining - bit_count;
            let value = (self.current_word_d5 >> shift) & mask;
            self.consume_bits_loc_13650(bit_count as u16)?;
            return Ok(value);
        }

        let current_tail_mask = ENIGMA_BIT_MASKS[bits_remaining - 1];
        let next_word = self.peek_refill_word_sub_13588()?;

        if bits_remaining == bit_count {
            let value = self.current_word_d5 & current_tail_mask;
            self.current_word_d5 = next_word;
            self.bits_remaining_d6 = 16;
            self.input_position += 2;
            return Ok(value);
        }

        let missing_bits = bit_count - bits_remaining;
        let next_mask = ENIGMA_BIT_MASKS[missing_bits - 1];
        let carried = (self.current_word_d5 & current_tail_mask) << missing_bits;
        let appended = (next_word >> (16 - missing_bits)) & next_mask;
        self.current_word_d5 = next_word;
        self.bits_remaining_d6 = (16 - missing_bits) as u16;
        self.input_position += 2;
        Ok((carried | appended) & mask)
    }

    #[cfg_attr(not(test), allow(dead_code))]
    fn read_bits_sub_13622(&mut self, bit_count: usize) -> Result<u16> {
        // ASM sub_13622 (kid.asm:26074): read up to 16 bits from the current
        // Enigma word buffer, splitting wide reads into a high chunk and a low
        // byte while preserving loc_13650's rolling refill.
        if bit_count == 0 || bit_count > 16 {
            bail!("Enigma bit reads must be between 1 and 16 bits");
        }
        if self.bits_remaining_d6 == 0 {
            self.refill_word_sub_13618()?;
        }

        if bit_count <= 8 {
            return self.read_short_bits_sub_1364c(bit_count as u16);
        }

        let high = self.read_short_bits_sub_1364c((bit_count - 8) as u16)?;
        let low = self.read_short_bits_sub_1364c(8)?;
        Ok((high << 8) | low)
    }

    #[cfg_attr(not(test), allow(dead_code))]
    fn read_short_bits_sub_1364c(&mut self, bit_count: u16) -> Result<u16> {
        // ASM sub_1364C (kid.asm:26105): tail-call sub_1363C to fetch the
        // short value, then shift d0 into a byte count for loc_13650's
        // rolling buffer consumption/refill logic.
        let value = self.peek_bits_sub_1363c(bit_count)?;
        self.consume_bits_loc_13650(bit_count)?;
        Ok(value)
    }

    #[cfg_attr(not(test), allow(dead_code))]
    fn peek_bits_sub_1363c(&self, bit_count: u16) -> Result<u16> {
        // ASM sub_1363C (kid.asm:26091): keep d6 intact, right-shift the
        // buffered word by (d6-d0), then mask the low d0 bits with the ROM's
        // 1/3/7/.../$FFFF lookup table.
        if bit_count == 0 || bit_count > self.bits_remaining_d6 {
            bail!("Enigma short read exceeded buffered bits");
        }
        let shift = self.bits_remaining_d6 - bit_count;
        let mask = ENIGMA_BIT_MASKS[usize::from(bit_count - 1)];
        Ok((self.current_word_d5 >> shift) & mask)
    }

    #[cfg_attr(not(test), allow(dead_code))]
    fn consume_bits_loc_13650(&mut self, bit_count: u16) -> Result<()> {
        self.bits_remaining_d6 -= bit_count;
        if self.bits_remaining_d6 >= 9 {
            return Ok(());
        }

        self.bits_remaining_d6 += 8;
        self.current_word_d5 = (self.current_word_d5 << 8) & 0xFF00;
        let next_byte = read_input_byte(self.bytes, &mut self.input_position)?;
        self.current_word_d5 |= u16::from(next_byte);
        Ok(())
    }

    fn consume_bits_loc_13650_allow_terminator_eof(&mut self, bit_count: u16) -> Result<()> {
        self.bits_remaining_d6 -= bit_count;
        if self.bits_remaining_d6 >= 9 {
            return Ok(());
        }
        if self.bytes.get(self.input_position).is_none() {
            return Ok(());
        }

        self.bits_remaining_d6 += 8;
        self.current_word_d5 = (self.current_word_d5 << 8) & 0xFF00;
        let next_byte = read_input_byte(self.bytes, &mut self.input_position)?;
        self.current_word_d5 |= u16::from(next_byte);
        Ok(())
    }

    fn peek_refill_word_sub_13588(&self) -> Result<u16> {
        let high_byte = *self
            .bytes
            .get(self.input_position)
            .ok_or_else(|| anyhow!("compressed payload ended unexpectedly"))?;
        let low_byte = *self
            .bytes
            .get(self.input_position + 1)
            .ok_or_else(|| anyhow!("compressed payload ended unexpectedly"))?;
        Ok((u16::from(high_byte) << 8) | u16::from(low_byte))
    }
}

fn read_u16_be(bytes: &[u8], offset: usize) -> u16 {
    u16::from_be_bytes([bytes[offset], bytes[offset + 1]])
}

fn read_u32_be(bytes: &[u8], offset: usize) -> u32 {
    u32::from_be_bytes([
        bytes[offset],
        bytes[offset + 1],
        bytes[offset + 2],
        bytes[offset + 3],
    ])
}

fn find_enemy_source_path(manifest: &AssetManifest, symbolic_name: &str) -> Option<String> {
    let wanted = normalise_enemy_name(symbolic_name);
    manifest.enemy_sources.iter().find_map(|asset| {
        let stem = Path::new(&asset.relative_path)
            .file_stem()?
            .to_string_lossy();
        if normalise_enemy_name(&stem) == wanted {
            Some(asset.relative_path.clone())
        } else {
            None
        }
    })
}

fn normalise_enemy_name(value: &str) -> String {
    value
        .chars()
        .filter(|character| character.is_ascii_alphanumeric())
        .map(|character| character.to_ascii_lowercase())
        .collect()
}

fn copy_directory_recursive(source: &Path, dest: &Path) -> Result<()> {
    if !source.exists() {
        return Ok(());
    }
    fs::create_dir_all(dest).with_context(|| format!("failed to create {}", dest.display()))?;
    for entry in
        fs::read_dir(source).with_context(|| format!("failed to read {}", source.display()))?
    {
        let entry = entry?;
        let entry_type = entry.file_type()?;
        let source_path = entry.path();
        let dest_path = dest.join(entry.file_name());
        if entry_type.is_dir() {
            copy_directory_recursive(&source_path, &dest_path)?;
        } else if entry_type.is_file() {
            copy_file(&source_path, &dest_path)?;
        }
    }
    Ok(())
}

fn copy_file(source: &Path, dest: &Path) -> Result<()> {
    if !source.is_file() {
        return Ok(());
    }
    if let Some(parent) = dest.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("failed to create {}", parent.display()))?;
    }
    fs::copy(source, dest).with_context(|| {
        format!(
            "failed to copy asset from {} to {}",
            source.display(),
            dest.display()
        )
    })?;
    Ok(())
}

fn collect_asset_files(root: &Path) -> Result<Vec<AssetFile>> {
    let mut files = Vec::new();
    if !root.exists() {
        return Ok(files);
    }
    collect_asset_files_recursive(root, root, &mut files)?;
    files.sort_by(|left, right| left.relative_path.cmp(&right.relative_path));
    Ok(files)
}

fn collect_asset_files_recursive(
    root: &Path,
    current: &Path,
    files: &mut Vec<AssetFile>,
) -> Result<()> {
    for entry in
        fs::read_dir(current).with_context(|| format!("failed to read {}", current.display()))?
    {
        let entry = entry?;
        let entry_type = entry.file_type()?;
        let path = entry.path();
        if entry_type.is_dir() {
            collect_asset_files_recursive(root, &path, files)?;
            continue;
        }
        if !entry_type.is_file() {
            continue;
        }

        let bytes =
            fs::read(&path).with_context(|| format!("failed to read asset {}", path.display()))?;
        let relative = path
            .strip_prefix(root)
            .map_err(|_| anyhow!("{} is not under {}", path.display(), root.display()))?;
        files.push(AssetFile {
            relative_path: normalise_relative_path(relative),
            size_bytes: bytes.len() as u64,
            sha1: sha1_hex(&bytes),
        });
    }
    Ok(())
}

fn parse_level_names(contents: &str) -> Result<BTreeMap<u8, (u8, String)>> {
    let mut levels = BTreeMap::new();
    for (index, line) in contents.lines().enumerate() {
        let trimmed = line.trim();
        if !trimmed.starts_with("0x") {
            continue;
        }
        let mut parts = trimmed.split_whitespace();
        let level_id = parse_hex_byte(
            parts
                .next()
                .ok_or_else(|| anyhow!("missing level id on line {}", index + 1))?,
        )?;
        let title_text_id = parse_hex_byte(
            parts
                .next()
                .ok_or_else(|| anyhow!("missing title id on line {}", index + 1))?,
        )?;
        let display_name = parts.collect::<Vec<_>>().join(" ");
        if display_name.is_empty() {
            bail!("missing level name on line {}", index + 1);
        }
        levels.insert(level_id, (title_text_id, display_name));
    }
    Ok(levels)
}

fn parse_level_title_cards(contents: &str) -> Result<BTreeMap<u8, LevelTitleCard>> {
    let title_scripts = parse_title_text_scripts(contents)?;
    let layouts = parse_title_text_layouts(contents)?;
    let mut level_title_cards = BTreeMap::new();
    let mut in_table = false;
    for (index, line) in contents.lines().enumerate() {
        let trimmed = line.trim();
        if trimmed.starts_with("AddrTbl_LevelNames:") {
            in_table = true;
            continue;
        }
        if !in_table || trimmed.is_empty() || trimmed.starts_with(';') {
            continue;
        }
        let (statement, comment) = split_comment(trimmed);
        let Some(data) = statement.strip_prefix("levnamhdr") else {
            continue;
        };
        let parts = data.split(',').map(str::trim).collect::<Vec<_>>();
        if parts.len() != 3 {
            bail!("invalid levnamhdr on line {}", index + 1);
        }
        let level_id = comment
            .and_then(parse_level_name_comment_id)
            .ok_or_else(|| anyhow!("missing level id comment on line {}", index + 1))?;
        let script_bytes = title_scripts
            .get(parts[0])
            .cloned()
            .ok_or_else(|| anyhow!("unknown title text {} on line {}", parts[0], index + 1))?;
        let layout_rows = layouts
            .get(parts[1])
            .cloned()
            .ok_or_else(|| anyhow!("unknown title layout {} on line {}", parts[1], index + 1))?;
        let level_number_glyph_count =
            u16::try_from(parse_numeric_value(parts[2])?).map_err(|_| {
                anyhow!(
                    "level number glyph count out of range on line {}: {}",
                    index + 1,
                    parts[2]
                )
            })?;
        let (line_destinations, level_number_destination) =
            split_title_card_layout(layout_rows, level_number_glyph_count, index + 1)?;
        level_title_cards.insert(
            level_id,
            LevelTitleCard {
                script_bytes,
                line_destinations,
                level_number_destination,
                level_number_glyph_count,
            },
        );
    }
    Ok(level_title_cards)
}

fn parse_level_files(contents: &str) -> Result<BTreeMap<u8, Vec<String>>> {
    let mut levels = BTreeMap::new();
    for (index, line) in contents.lines().enumerate() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        let mut parts = trimmed.split_whitespace();
        let level_id = parse_hex_byte(
            parts
                .next()
                .ok_or_else(|| anyhow!("missing level id on line {}", index + 1))?,
        )?;
        let resource_paths = parts.map(str::to_owned).collect::<Vec<_>>();
        levels.insert(level_id, resource_paths);
    }
    Ok(levels)
}

fn parse_title_text_scripts(contents: &str) -> Result<BTreeMap<String, Vec<u8>>> {
    let mut scripts = BTreeMap::new();
    let mut current_label = None::<String>;
    let mut current_bytes = Vec::new();
    for (index, line) in contents.lines().enumerate() {
        let trimmed = line.trim();
        if let Some(label) = parse_asm_label(trimmed) {
            if let Some(previous_label) = current_label.replace(label.to_owned())
                && !current_bytes.is_empty()
            {
                scripts.insert(previous_label, std::mem::take(&mut current_bytes));
            }
            continue;
        }
        let Some(label) = current_label.as_deref() else {
            continue;
        };
        if !label.starts_with("TitleText_") {
            continue;
        }
        let (statement, _) = split_comment(trimmed);
        let Some(data) = statement.strip_prefix("dc.b") else {
            continue;
        };
        current_bytes.extend(parse_dc_b_bytes(data.trim(), index + 1)?);
    }
    if let Some(label) = current_label
        && !current_bytes.is_empty()
    {
        scripts.insert(label, current_bytes);
    }
    Ok(scripts)
}

fn parse_title_text_layouts(contents: &str) -> Result<BTreeMap<String, Vec<TitleCardDestination>>> {
    let mut layouts = BTreeMap::new();
    let mut current_label = None::<String>;
    let mut current_rows = Vec::new();
    for (index, line) in contents.lines().enumerate() {
        let trimmed = line.trim();
        if let Some(label) = parse_asm_label(trimmed) {
            if let Some(previous_label) = current_label.replace(label.to_owned())
                && !current_rows.is_empty()
            {
                layouts.insert(previous_label, std::mem::take(&mut current_rows));
            }
            continue;
        }
        let Some(label) = current_label.as_deref() else {
            continue;
        };
        if !label.starts_with("TitleTextLayout_") {
            continue;
        }
        let (statement, _) = split_comment(trimmed);
        let Some(data) = statement.strip_prefix("dc.w") else {
            continue;
        };
        let coords = data
            .split(',')
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .collect::<Vec<_>>();
        if coords.len() != 2 {
            bail!("invalid title layout row on line {}", index + 1);
        }
        let x = u16::try_from(parse_numeric_value(coords[0])?)
            .map_err(|_| anyhow!("title layout X out of range on line {}", index + 1))?;
        let y = u16::try_from(parse_numeric_value(coords[1])?)
            .map_err(|_| anyhow!("title layout Y out of range on line {}", index + 1))?;
        current_rows.push(TitleCardDestination { x, y });
    }
    if let Some(label) = current_label
        && !current_rows.is_empty()
    {
        layouts.insert(label, current_rows);
    }
    Ok(layouts)
}

fn split_title_card_layout(
    mut layout_rows: Vec<TitleCardDestination>,
    level_number_glyph_count: u16,
    line_number: usize,
) -> Result<(Vec<TitleCardDestination>, Option<TitleCardDestination>)> {
    if level_number_glyph_count == 0 {
        return Ok((layout_rows, None));
    }
    let level_number_destination = layout_rows.pop().ok_or_else(|| {
        anyhow!(
            "missing level number destination for levnamhdr on line {}",
            line_number
        )
    })?;
    Ok((layout_rows, Some(level_number_destination)))
}

fn parse_dc_b_bytes(data: &str, line_number: usize) -> Result<Vec<u8>> {
    let mut bytes = Vec::new();
    for token in data
        .split(',')
        .map(str::trim)
        .filter(|token| !token.is_empty())
    {
        if token.starts_with('"') {
            bytes.extend(parse_asm_byte_string(token, line_number)?);
            continue;
        }
        let value = parse_numeric_value(token)
            .with_context(|| format!("failed to parse dc.b token on line {}", line_number))?;
        let byte = u8::try_from(value)
            .map_err(|_| anyhow!("dc.b byte out of range on line {}: {}", line_number, token))?;
        bytes.push(byte);
    }
    Ok(bytes)
}

fn parse_level_name_comment_id(comment: &str) -> Option<u8> {
    let trimmed = comment.trim();
    parse_asm_integer(trimmed)
        .and_then(|value| u8::try_from(value).ok())
        .or_else(|| u8::from_str_radix(trimmed, 16).ok())
}

fn parse_asm_label(line: &str) -> Option<&str> {
    let (statement, _) = split_comment(line);
    statement.trim().strip_suffix(':')
}

fn parse_asm_byte_string(token: &str, line_number: usize) -> Result<Vec<u8>> {
    if !token.ends_with('"') {
        bail!("unterminated string on line {}", line_number);
    }
    let mut bytes = Vec::new();
    let mut chars = token[1..token.len() - 1].chars().peekable();
    while let Some(ch) = chars.next() {
        if ch != '\\' {
            let byte = u8::try_from(u32::from(ch))
                .map_err(|_| anyhow!("non-byte character in string on line {}", line_number))?;
            bytes.push(byte);
            continue;
        }
        let escaped = chars
            .next()
            .ok_or_else(|| anyhow!("truncated escape on line {}", line_number))?;
        match escaped {
            '0' => bytes.push(0),
            'x' => {
                let high = chars
                    .next()
                    .ok_or_else(|| anyhow!("truncated hex escape on line {}", line_number))?;
                let low = chars
                    .next()
                    .ok_or_else(|| anyhow!("truncated hex escape on line {}", line_number))?;
                let hex = format!("{high}{low}");
                let value = u8::from_str_radix(&hex, 16).with_context(|| {
                    format!("invalid hex escape \\x{hex} on line {}", line_number)
                })?;
                bytes.push(value);
            }
            '\\' => bytes.push(b'\\'),
            '"' => bytes.push(b'"'),
            other => bail!("unsupported escape \\{} on line {}", other, line_number),
        }
    }
    Ok(bytes)
}

fn parse_map_order(contents: &str) -> Result<Vec<u8>> {
    let mut map_order = Vec::new();
    for (index, line) in contents.lines().enumerate() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        let (statement, _) = split_comment(trimmed);
        let Some(data) = statement.strip_prefix("dc.b") else {
            continue;
        };
        for value in data
            .split(',')
            .map(str::trim)
            .filter(|value| !value.is_empty())
        {
            let parsed = parse_numeric_value(value)
                .with_context(|| format!("failed to parse map-order byte on line {}", index + 1))?;
            let level_id = u8::try_from(parsed).map_err(|_| {
                anyhow!(
                    "map-order byte on line {} was out of range for u8: {}",
                    index + 1,
                    parsed
                )
            })?;
            map_order.push(level_id);
        }
    }
    Ok(map_order)
}

fn parse_named_values(contents: &str) -> Result<Vec<NamedValue>> {
    let mut values = Vec::new();
    for line in contents.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with(';') {
            continue;
        }
        let (assignment, comment) = split_comment(trimmed);
        let Some((name, value)) = assignment.split_once('=') else {
            continue;
        };
        let value = match parse_numeric_value(value.trim()) {
            Ok(value) => value,
            Err(_) => continue,
        };
        values.push(NamedValue {
            name: name.trim().to_owned(),
            value,
            comment: comment.map(str::to_owned),
        });
    }
    Ok(values)
}

fn parse_ram_variables(contents: &str) -> Result<Vec<RamVariable>> {
    let mut values = Vec::new();
    for line in contents.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with(';') {
            continue;
        }
        let (assignment, comment) = split_comment(trimmed);
        let Some((name, value)) = assignment.split_once('=') else {
            continue;
        };
        let value = value.trim();
        if !value.starts_with('$') {
            continue;
        }
        values.push(RamVariable {
            name: name.trim().to_owned(),
            address: parse_hex_u32(value)?,
            detail: comment.map(str::to_owned),
        });
    }
    Ok(values)
}

fn split_comment(line: &str) -> (&str, Option<&str>) {
    if let Some((assignment, comment)) = line.split_once(';') {
        (assignment.trim_end(), Some(comment.trim()))
    } else {
        (line, None)
    }
}

fn parse_numeric_value(value: &str) -> Result<i64> {
    if let Some(hex) = value.strip_prefix('$') {
        return i64::from_str_radix(hex, 16).with_context(|| format!("invalid hex value {value}"));
    }
    value
        .parse::<i64>()
        .with_context(|| format!("invalid integer value {value}"))
}

fn parse_hex_byte(value: &str) -> Result<u8> {
    let hex = value
        .strip_prefix("0x")
        .or_else(|| value.strip_prefix("0X"))
        .unwrap_or(value);
    u8::from_str_radix(hex, 16).with_context(|| format!("invalid hex byte {value}"))
}

fn parse_hex_u32(value: &str) -> Result<u32> {
    let hex = value
        .strip_prefix('$')
        .ok_or_else(|| anyhow!("expected $-prefixed address, got {value}"))?;
    u32::from_str_radix(hex, 16).with_context(|| format!("invalid hex address {value}"))
}

fn sha1_hex(bytes: &[u8]) -> String {
    let mut hasher = Sha1::new();
    hasher.update(bytes);
    format!("{:x}", hasher.finalize())
}

fn normalise_relative_path(path: &Path) -> String {
    path.components()
        .map(|component| component.as_os_str().to_string_lossy().into_owned())
        .collect::<Vec<_>>()
        .join("/")
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    const SAXMAN_FIXTURE: &[u8] = &[0x00, 0x02, 0xF4, 0x60, 0x41, 0x42, 0x43, 0x44, 0x00, 0x00];
    const ENIGMA_FIXTURE: &[u8] = &[
        0x05, 0x00, 0x00, 0x01, 0x00, 0x0A, 0x09, 0x18, 0x10, 0xFE, 0x00,
    ];
    const ENIGMA_COMMON_RUN_FIXTURE: &[u8] =
        &[0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x7D, 0xFF, 0xE0, 0x00];
    const REAL_STYLE_BLOCK_FIXTURE: &[u8] = &[
        0x40, 0x76, 0x02, 0x08, 0xA0, 0x80, 0x38, 0x1F, 0x92, 0x08, 0x64, 0x82, 0x81, 0x40, 0x9E,
        0x58, 0x28, 0x96, 0x08, 0x86, 0x02, 0x99, 0x80, 0xA8, 0x68, 0x0B, 0x1C, 0x03, 0x07, 0x00,
        0xD1, 0xC0,
    ];
    const REAL_STYLE_BACKGROUND_FIXTURE: &[u8] = &[
        0x00, 0x0D, 0x00, 0x00, 0x00, 0x2D, 0x00, 0x0D, 0x00, 0x1E, 0x00, 0x2D, 0x00, 0x0D, 0x00,
        0x3C, 0x00, 0x2D, 0x00, 0x0F, 0x00, 0x00, 0x00, 0x29, 0x00, 0x0F, 0x00, 0x24, 0x00, 0x29,
        0x00, 0x0F, 0x00, 0x3C, 0x00, 0x29, 0x00, 0x03, 0x00, 0x00, 0x00, 0x04, 0x00, 0x02, 0x00,
        0x06, 0x00, 0x20, 0x00, 0x02, 0x00, 0x24, 0x00, 0x1F, 0x00, 0x02, 0x00, 0x26, 0x00, 0x0A,
        0x00, 0x06, 0x00, 0x28,
    ];

    fn write_test_bundle_source(
        source_root: &Path,
        foreground_bytes: &[u8],
        block_bytes: &[u8],
        background_bytes: &[u8],
    ) {
        fs::create_dir_all(source_root.join("code/enemy")).expect("enemy dir");
        fs::create_dir_all(source_root.join("doc")).expect("doc dir");
        fs::create_dir_all(source_root.join("ingame/anim")).expect("anim dir");
        fs::create_dir_all(source_root.join("ingame/artcomp")).expect("artcomp dir");
        fs::create_dir_all(source_root.join("ingame/artunc")).expect("artunc dir");
        fs::create_dir_all(source_root.join("ingame/artunc_kid")).expect("kid art dir");
        fs::create_dir_all(source_root.join("ingame/palette_enemy")).expect("enemy palette dir");
        fs::create_dir_all(source_root.join("ingame/palette_kid")).expect("kid palette dir");
        fs::create_dir_all(source_root.join("level")).expect("level dir");
        fs::create_dir_all(source_root.join("level/platform")).expect("platform assets dir");
        fs::create_dir_all(source_root.join("level/bgscroll")).expect("bgscroll assets dir");
        fs::create_dir_all(source_root.join("level/header")).expect("header assets dir");
        fs::create_dir_all(source_root.join("level/enemy")).expect("enemy assets dir");
        fs::create_dir_all(source_root.join("level/foreground")).expect("foreground assets dir");
        fs::create_dir_all(source_root.join("level/block")).expect("block assets dir");
        fs::create_dir_all(source_root.join("level/background")).expect("background assets dir");
        fs::create_dir_all(source_root.join("scenes")).expect("scene dir");
        fs::create_dir_all(source_root.join("sound")).expect("sound dir");
        fs::create_dir_all(source_root.join("theme")).expect("theme dir");

        fs::write(source_root.join("code/enemy/Armadillo.asm"), b"enemy code").expect("enemy");
        fs::write(source_root.join("theme/sample.bin"), b"theme").expect("theme");
        fs::write(source_root.join("scenes/title.bin"), b"scene").expect("scene");
        fs::write(source_root.join("ingame/anim/idle.bin"), b"anim").expect("anim");
        fs::write(
            source_root.join("ingame/artcomp/Blocks.bin"),
            SAXMAN_FIXTURE,
        )
        .expect("blocks");
        fs::write(
            source_root.join("ingame/artcomp/HUD_numbers.bin"),
            SAXMAN_FIXTURE,
        )
        .expect("hud numbers");
        fs::write(source_root.join("ingame/artunc/diamond_1.bin"), [0; 32]).expect("diamond");
        fs::write(
            source_root.join("ingame/palette_enemy/Crystal_1.bin"),
            [0; 30],
        )
        .expect("diamond palette");
        fs::write(
            source_root.join("ingame/palette_enemy/Mini_Hopping_Skull_1.bin"),
            [0; 30],
        )
        .expect("skull palette");
        fs::write(source_root.join("ingame/palette_kid/Kid.bin"), [0; 30]).expect("kid palette");
        fs::write(source_root.join("ingame/artunc_kid/kid.bin"), [0; 32 * 12]).expect("kid idle");
        fs::write(
            source_root.join("ingame/artunc_kid/kid_walk_1.bin"),
            [1; 32 * 12],
        )
        .expect("kid walk");
        fs::write(
            source_root.join("ingame/artunc_kid/kid_jumping_1.bin"),
            [2; 32 * 12],
        )
        .expect("kid jump rise");
        fs::write(
            source_root.join("ingame/artunc_kid/kid_jumping_2.bin"),
            [3; 32 * 12],
        )
        .expect("kid jump fall");
        fs::write(source_root.join("sound/music.bin"), b"sound").expect("sound");
        fs::write(
            source_root.join("level/platform/00.asm"),
            b"\tptfm\t0,0,0,0,0,0,0,0,0,0,PlatformScript_Nothing-PlatformScript_BaseAddress\n\tdc.w\t$FFFF\n",
        )
        .expect("platform");
        fs::write(
            source_root.join("level/bgscroll/00.bin"),
            [0x10, 0x10, 0x40, 0x60, 0xFF],
        )
        .expect("bgscroll");
        fs::write(
            source_root.join("level/header/00.bin"),
            [
                0x06, 0x04, 0x08, 0x18, 0x07, 0x50, 0x00, 0xE0, 0x07, 0x50, 0x03, 0x00,
            ],
        )
        .expect("header");
        fs::write(
            source_root.join("level/enemy/00.bin"),
            [
                0x00, 0x00, 0x00, 0x00, 0xFF, 0xFF, 0x00, 0x01, 0x00, 0x1C, 0x7D, 0x00, 0x80, 0x01,
                0x80, 0x16, 0x00, 0x01, 0x01, 0x20, 0x00, 0xE0,
            ],
        )
        .expect("enemy");
        fs::write(
            source_root.join("level/foreground/00.bin"),
            foreground_bytes,
        )
        .expect("foreground");
        fs::write(source_root.join("level/block/00.bin"), block_bytes).expect("block");
        fs::write(
            source_root.join("level/background/00.bin"),
            background_bytes,
        )
        .expect("background");
        fs::write(
            source_root.join("doc/level_list.txt"),
            "0x00 0x4A Blue Lake Woods 1\n",
        )
        .expect("level names");
        fs::write(
            source_root.join("level/levelnames.asm"),
            r#"; \0   = end of line
TitleText_00:
	dc.b	"blue\0lake\0woods"
	dc.b	$FF

TitleTextLayout_3Lines:
	dc.w	$20, $28
	dc.w	$20, $48
	dc.w	$20, $68
	dc.w	$DC, $50

AddrTbl_LevelNames:   ;1A842
	levnamhdr	TitleText_00, TitleTextLayout_3Lines, 1	;  0
"#,
        )
        .expect("levelnames");
        fs::write(
            source_root.join("level/level_files.txt"),
            "00 platform/00.asm bgscroll/00.bin header/00.bin enemy/00.bin foreground/00.bin block/00.bin background/00.bin\n",
        )
        .expect("level files");
        fs::write(source_root.join("level/maporder.asm"), "\tdc.b\t$00\t; 0\n").expect("map order");
        fs::write(source_root.join("constants.asm"), "Theme_Sky = 1 ; sky\n").expect("constants");
        fs::write(
            source_root.join("variables.asm"),
            "Camera_X_pos = $FFFFF81C ; long (16.16 fixed point)\n",
        )
        .expect("variables");
    }

    fn write_default_test_bundle_source(source_root: &Path) {
        write_test_bundle_source(
            source_root,
            SAXMAN_FIXTURE,
            REAL_STYLE_BLOCK_FIXTURE,
            REAL_STYLE_BACKGROUND_FIXTURE,
        );
    }

    fn write_sub_129ce_test_assets(source_root: &Path) {
        fs::write(
            source_root.join("ingame/artunc/Juggernaut_skull_frame_1.bin"),
            [0x11; 32 * 4],
        )
        .expect("skull frame 1");
        fs::write(
            source_root.join("ingame/artunc/Juggernaut_skull_frame_2.bin"),
            [0x22; 32 * 4],
        )
        .expect("skull frame 2");
        fs::write(
            source_root.join("ingame/artunc/Some_kind_of_star.bin"),
            [0x33; 32 * 4],
        )
        .expect("star");
    }

    fn write_sub_12c24_test_asset(source_root: &Path) {
        fs::write(
            source_root.join("ingame/artunc/Hitpoint_display.bin"),
            [0x44; 32 * 4],
        )
        .expect("hitpoint display");
    }

    #[test]
    fn parses_level_names_and_titles() {
        let parsed =
            parse_level_names("0x00 0x4A Blue Lake Woods 1\n0x01 0x4B Blue Lake Woods 2\n")
                .expect("level names should parse");
        assert_eq!(
            parsed.get(&0x00),
            Some(&(0x4A, String::from("Blue Lake Woods 1")))
        );
        assert_eq!(
            parsed.get(&0x01),
            Some(&(0x4B, String::from("Blue Lake Woods 2")))
        );
    }

    #[test]
    fn parses_level_title_cards_from_levelnames_asm() {
        let parsed = parse_level_title_cards(
            r#"TitleText_00:
	dc.b	"blue\0lake\0woods"
	dc.b	$FF
TitleText_2C:
	dc.b	"\x7C\x84hills\0have\x84eyes"
	dc.b	$FF

TitleTextLayout_3Lines:
	dc.w	$20, $28
	dc.w	$20, $48
	dc.w	$20, $68
	dc.w	$DC, $50
TitleTextLayout_2Lines:
	dc.w	$20, $28
	dc.w	$20, $40
	dc.w	$DC, $3C

AddrTbl_LevelNames:   ;1A842
	levnamhdr	TitleText_00, TitleTextLayout_3Lines, 1	;  0
	levnamhdr	TitleText_2C, TitleTextLayout_2Lines, 0	;  2C
"#,
        )
        .expect("title cards should parse");

        assert_eq!(
            parsed.get(&0x00),
            Some(&LevelTitleCard {
                script_bytes: b"blue\0lake\0woods\xFF".to_vec(),
                line_destinations: vec![
                    TitleCardDestination { x: 0x20, y: 0x28 },
                    TitleCardDestination { x: 0x20, y: 0x48 },
                    TitleCardDestination { x: 0x20, y: 0x68 },
                ],
                level_number_destination: Some(TitleCardDestination { x: 0xDC, y: 0x50 }),
                level_number_glyph_count: 1,
            })
        );
        assert_eq!(
            parsed.get(&0x2C),
            Some(&LevelTitleCard {
                script_bytes: vec![
                    0x7C, 0x84, b'h', b'i', b'l', b'l', b's', 0x00, b'h', b'a', b'v', b'e', 0x84,
                    b'e', b'y', b'e', b's', 0xFF,
                ],
                line_destinations: vec![
                    TitleCardDestination { x: 0x20, y: 0x28 },
                    TitleCardDestination { x: 0x20, y: 0x40 },
                    TitleCardDestination { x: 0xDC, y: 0x3C },
                ],
                level_number_destination: None,
                level_number_glyph_count: 0,
            })
        );
    }

    #[test]
    fn parses_level_files() {
        let parsed = parse_level_files("00 platform/00.asm bgscroll/00.bin header/00.bin\n")
            .expect("level files should parse");
        assert_eq!(
            parsed.get(&0x00),
            Some(&vec![
                String::from("platform/00.asm"),
                String::from("bgscroll/00.bin"),
                String::from("header/00.bin"),
            ])
        );
    }

    #[test]
    fn parses_map_order_bytes() {
        let parsed = parse_map_order("\tdc.b\t$4A\t; 0\n\tdc.b\t$4B, $2D\t; 1, 2\n")
            .expect("map order should parse");

        assert_eq!(parsed, vec![0x4A, 0x4B, 0x2D]);
    }

    #[test]
    fn classifies_level_resources() {
        let resources = level_resources_from_paths(&[
            String::from("platform/00.asm"),
            String::from("bgscroll/00.bin"),
            String::from("header/00.bin"),
            String::from("enemy/00.bin"),
            String::from("foreground/00.bin"),
            String::from("block/00.bin"),
            String::from("background/00.bin"),
        ]);
        assert_eq!(
            resources.platform_script.as_deref(),
            Some("platform/00.asm")
        );
        assert_eq!(resources.bgscroll.as_deref(), Some("bgscroll/00.bin"));
        assert_eq!(resources.header.as_deref(), Some("header/00.bin"));
        assert_eq!(resources.enemy.as_deref(), Some("enemy/00.bin"));
        assert_eq!(resources.foreground.as_deref(), Some("foreground/00.bin"));
        assert_eq!(resources.block.as_deref(), Some("block/00.bin"));
        assert_eq!(resources.background.as_deref(), Some("background/00.bin"));
    }

    #[test]
    fn resolves_enemy_definitions_to_sources() {
        let manifest = AssetManifest {
            generated_bundle_version: GENERATED_BUNDLE_VERSION,
            rom_sha1: String::from("abc123"),
            upstream_commit: String::from("deadbeef"),
            source_root: PathBuf::from("/tmp/source"),
            output_root: PathBuf::from("/tmp/output"),
            map_order: Vec::new(),
            levels: Vec::new(),
            constants: vec![
                NamedValue {
                    name: String::from("objid_Armadillo"),
                    value: 0x04,
                    comment: None,
                },
                NamedValue {
                    name: String::from("objid_TarMonster"),
                    value: 0x05,
                    comment: None,
                },
            ],
            ram_variables: Vec::new(),
            enemy_sources: vec![
                AssetFile {
                    relative_path: String::from("Armadillo.asm"),
                    size_bytes: 123,
                    sha1: String::from("x"),
                },
                AssetFile {
                    relative_path: String::from("Tar_Monster.asm"),
                    size_bytes: 456,
                    sha1: String::from("y"),
                },
            ],
            theme_assets: Vec::new(),
            scene_assets: Vec::new(),
            animation_assets: Vec::new(),
            sound_assets: Vec::new(),
            ingame_assets: Vec::new(),
            decoded_levels: vec![],
        };

        let definitions = enemy_definitions(&manifest);
        assert_eq!(definitions.len(), 2);
        assert_eq!(definitions[0].source_path.as_deref(), Some("Armadillo.asm"));
        assert_eq!(
            definitions[1].source_path.as_deref(),
            Some("Tar_Monster.asm")
        );
        assert_eq!(
            find_enemy_definition(&manifest, "04")
                .as_ref()
                .map(|definition| definition.symbolic_name.as_str()),
            Some("Armadillo")
        );
        assert_eq!(
            find_enemy_definition(&manifest, "tarmonster")
                .as_ref()
                .map(|definition| definition.id),
            Some(0x05)
        );
    }

    #[test]
    fn parses_level_header_fields() {
        let header = parse_level_header(&[
            0x06, 0x84, 0x88, 0x18, 0x07, 0x50, 0x00, 0xE0, 0x07, 0x50, 0x03, 0x00,
        ])
        .expect("header should parse");
        assert_eq!(header.width_screens, 0x06);
        assert_eq!(header.height_screens, 0x04);
        assert_eq!(header.height_flag_bits, 0x02);
        assert_eq!(header.level_theme, 0x08);
        assert_eq!(header.murder_wall_flags, 0x02);
        assert_eq!(header.background_theme, 0x08);
        assert_eq!(header.weather_flags, 0x01);
        assert_eq!(header.player_start_x, 0x0750);
        assert_eq!(header.player_start_y, 0x00E0);
        assert_eq!(header.flag_x, 0x0750);
        assert_eq!(header.flag_y, 0x0300);
    }

    #[test]
    fn parses_enemy_layout_header_and_objects() {
        let layout = parse_enemy_layout(&[
            0x00, 0x00, 0x00, 0x00, 0xFF, 0xFF, 0x00, 0x01, 0x40, 0x1C, 0x7D, 0x00, 0x80, 0x02,
            0x80, 0x16, 0x00, 0x01, 0x01, 0x20, 0x00, 0xE0, 0x81, 0xFF, 0x00, 0x02, 0x03, 0x40,
            0x02, 0x00,
        ])
        .expect("enemy layout should parse");

        assert!(!layout.axis_uses_y_position);
        assert_eq!(layout.unknown_prefix, 0x0000_0000);
        assert_eq!(layout.enemy_type_slots, vec![0xFFFF, 0x0001, 0x401C]);
        assert_eq!(layout.unknown_word, 0x7D00);
        assert_eq!(layout.header_byte, 0x80);
        assert_eq!(layout.object_count, 0x02);
        assert_eq!(layout.segments.len(), 1);
        assert_eq!(layout.segments[0].lower_bound, 0x0000);
        assert_eq!(layout.segments[0].upper_bound, 0x7D00);
        assert_eq!(layout.segments[0].object_batch_flags, 0x80);
        assert_eq!(
            layout.segments[0].enemy_type_slots,
            vec![0xFFFF, 0x0001, 0x401C]
        );
        assert_eq!(layout.objects.len(), 2);
        assert_eq!(layout.objects[0].object_type, 0x80);
        assert_eq!(layout.objects[0].flags, 0x16);
        assert_eq!(layout.objects[0].hit_points_plus_one, 0x0001);
        assert_eq!(layout.objects[0].x, 0x0120);
        assert_eq!(layout.objects[0].y, 0x00E0);
        assert_eq!(layout.objects[1].object_type, 0x81);
        assert_eq!(layout.objects[1].flags, 0xFF);
        assert_eq!(layout.objects[1].hit_points_plus_one, 0x0002);
        assert_eq!(layout.objects[1].x, 0x0340);
        assert_eq!(layout.objects[1].y, 0x0200);
    }

    #[test]
    fn parses_bgscroll_profile_layers() {
        let profile =
            parse_bgscroll_profile(&[0x10, 0x10, 0x40, 0x60, 0xFF]).expect("bgscroll should parse");
        assert_eq!(profile.layer_speeds, vec![0x10, 0x10, 0x40, 0x60]);
        assert!(profile.has_terminator);
    }

    #[test]
    fn parses_unterminated_bgscroll_profile_layers() {
        let profile =
            parse_bgscroll_profile(&[0x10, 0x10, 0x40, 0x60]).expect("bgscroll should parse");
        assert_eq!(profile.layer_speeds, vec![0x10, 0x10, 0x40, 0x60]);
        assert!(!profile.has_terminator);
    }

    #[test]
    fn parses_platform_layout_entries() {
        let layout = parse_platform_layout(
            "\tptfm\t24,25,0,40,0,19,0,2,3,0,PlatformScript_3C3A-PlatformScript_BaseAddress\n\tdc.w\t$FFFF\n",
        )
        .expect("platform layout should parse");
        assert_eq!(layout.placements.len(), 1);
        let placement = &layout.placements[0];
        assert_eq!(placement.x, 24);
        assert_eq!(placement.y, 25);
        assert_eq!(placement.buffer_right, 40);
        assert_eq!(placement.kind, 0);
        assert_eq!(placement.subtype, 2);
        assert_eq!(placement.h_variant, 3);
        assert_eq!(placement.v_variant, 0);
        assert_eq!(
            placement.script.raw,
            "PlatformScript_3C3A-PlatformScript_BaseAddress"
        );
        assert_eq!(placement.script.relative_offset, Some(0x3C3A));
    }

    #[test]
    fn saxman_fixture_decodes_to_expected_bytes() {
        let decoded = decode_saxman(SAXMAN_FIXTURE).expect("fixture should decode");
        assert_eq!(decoded, b"ABCD");
    }

    #[test]
    fn sub_13618_refill_word_loads_big_endian_word_and_resets_bit_count() {
        let mut bits = EnigmaBitReader::new(&[0x12, 0x34, 0x56, 0x78]);
        bits.refill_word_sub_13618()
            .expect("ASM sub_13618 should load the next buffered word");

        assert_eq!(bits.current_word_d5, 0x1234);
        assert_eq!(bits.bits_remaining_d6, 16);
        assert_eq!(bits.input_position, 2);
    }

    #[test]
    fn sub_13622_reads_wide_values_across_the_rolling_refill_boundary() {
        let mut bits = EnigmaBitReader::new(&[0xAB, 0xCD, 0xEF, 0x12]);

        assert_eq!(
            bits.read_bits_sub_13622(12)
                .expect("ASM sub_13622 should split 12-bit reads at 8 bits"),
            0x0ABC
        );
        assert_eq!(bits.current_word_d5, 0xCDEF);
        assert_eq!(bits.bits_remaining_d6, 12);
        assert_eq!(bits.input_position, 3);
        assert_eq!(
            bits.read_bits_sub_13622(4)
                .expect("ASM sub_13622 should keep the post-refill nibble aligned"),
            0x000D
        );
        assert_eq!(bits.current_word_d5, 0xEF12);
        assert_eq!(bits.bits_remaining_d6, 16);
        assert_eq!(bits.input_position, 4);
    }

    #[test]
    fn sub_1363c_peeks_the_low_masked_bits_without_consuming_buffer_state() {
        let bits = EnigmaBitReader {
            bytes: &[],
            input_position: 0,
            current_word_d5: 0b1101_0110_1011_0011,
            bits_remaining_d6: 11,
        };

        assert_eq!(
            bits.peek_bits_sub_1363c(5)
                .expect("ASM sub_1363C should shift by d6-d0 and mask the low d0 bits"),
            0b1_1010
        );
        assert_eq!(bits.current_word_d5, 0b1101_0110_1011_0011);
        assert_eq!(bits.bits_remaining_d6, 11);
        assert_eq!(bits.input_position, 0);
    }

    #[test]
    fn sub_1364c_reads_short_values_and_rolls_the_buffer_forward() {
        let mut bits = EnigmaBitReader {
            bytes: &[0xAB, 0xCD, 0xEF, 0x12],
            input_position: 3,
            current_word_d5: 0xCDEF,
            bits_remaining_d6: 12,
        };

        assert_eq!(
            bits.read_short_bits_sub_1364c(4)
                .expect("ASM sub_1364C should consume the short value and refill via loc_13650"),
            0x000D
        );
        assert_eq!(bits.current_word_d5, 0xEF12);
        assert_eq!(bits.bits_remaining_d6, 16);
        assert_eq!(bits.input_position, 4);
    }

    #[test]
    fn eni_dec_loc_134ea_reads_six_and_seven_bit_prefixes() {
        let mut bits = EnigmaBitReader::new(&[0x7D, 0xFF, 0xE0, 0x00]);
        bits.refill_word_sub_13618()
            .expect("ASM sub_13618 should seed d5 before EniDec loc_134EA runs");

        assert_eq!(
            bits.read_prefix_eni_dec()
                .expect("ASM EniDec loc_134EA should decode the first common-run prefix"),
            (3, 0x0F)
        );
        assert_eq!(bits.bits_remaining_d6, 10);
        assert_eq!(
            bits.read_prefix_eni_dec()
                .expect("ASM EniDec loc_134EA should decode the second common-run prefix"),
            (3, 0x0F)
        );
        assert_eq!(bits.bits_remaining_d6, 12);
        assert_eq!(
            bits.read_prefix_eni_dec()
                .expect("ASM EniDec loc_134EA should decode the 7-bit terminator prefix"),
            (7, 0x0F)
        );
    }

    #[test]
    fn sub_13c0_appends_a_static_sprite_entry_and_advances_the_tile_cursor() {
        let mut state = SpriteBuilderStateSub13C0 {
            number_sprites: 3,
            tile_word_cursor: 0x8200,
            dma_tile_word_cursor: 0x9000,
            next_art_source: 0x0012_3400,
            dma_transfers: Vec::new(),
        };

        let entry = append_sprite_entry_sub_13c0(&mut state, 0x0180, 0x0090, 0x0F, false);

        assert_eq!(
            entry,
            SpriteTableEntrySub13C0 {
                y: 0x0090,
                size: 0x0F,
                link: 4,
                tile_word: 0x8200,
                x: 0x0180,
            }
        );
        assert_eq!(state.number_sprites, 4);
        assert_eq!(state.tile_word_cursor, 0x8210);
        assert_eq!(state.dma_tile_word_cursor, 0x9000);
        assert_eq!(state.next_art_source, 0x0012_3400);
        assert!(state.dma_transfers.is_empty());
    }

    #[test]
    fn sub_13c0_queues_dynamic_art_dma_and_advances_both_cursors() {
        let mut state = SpriteBuilderStateSub13C0 {
            number_sprites: 0,
            tile_word_cursor: 0x6000,
            dma_tile_word_cursor: 0x6000,
            next_art_source: 0x00AB_C000,
            dma_transfers: Vec::new(),
        };

        let entry = append_sprite_entry_sub_13c0(&mut state, 0x0108, 0x00B8, 0x06, true);

        assert_eq!(
            entry,
            SpriteTableEntrySub13C0 {
                y: 0x00B8,
                size: 0x06,
                link: 1,
                tile_word: 0x6000,
                x: 0x0108,
            }
        );
        assert_eq!(state.number_sprites, 1);
        assert_eq!(state.tile_word_cursor, 0x6006);
        assert_eq!(state.dma_tile_word_cursor, 0x6006);
        assert_eq!(state.next_art_source, 0x00AB_C0C0);
        assert_eq!(
            state.dma_transfers,
            vec![SpriteDmaTransferSub13C0 {
                tile_count: 6,
                art_source: 0x00AB_C000,
            }]
        );
    }

    #[test]
    fn sub_13588_combines_enabled_flag_bits_with_packet_bits() {
        let mut bits = EnigmaBitReader::new(&[0xAC, 0x00, 0x00]);
        bits.refill_word_sub_13618()
            .expect("ASM sub_13618 should seed d5 before sub_13588 consumes flags and packet bits");

        assert_eq!(
            read_inline_value_sub_13588(&mut bits, 0x0040, 0b1_0101, 5)
                .expect("ASM sub_13588 should merge flag bits and packet bits into the tile word"),
            0x884C
        );
    }

    #[test]
    fn sub_13588_flag_reads_do_not_trigger_loc_13650_refill() {
        let mut bits = EnigmaBitReader {
            bytes: &[0xE0],
            input_position: 0,
            current_word_d5: 0x0005,
            bits_remaining_d6: 3,
        };

        assert_eq!(
            read_enigma_flags(&mut bits, 0b1_1100)
                .expect("ASM sub_13588 should read enabled flag bits directly from d5"),
            0xA000
        );
        assert_eq!(bits.bits_remaining_d6, 0);
        assert_eq!(bits.input_position, 0);
        assert_eq!(
            bits.read_prefix_eni_dec().unwrap_err().to_string(),
            "Enigma short read exceeded buffered bits"
        );
    }

    #[test]
    fn sub_13588_preserves_prefix_readability_after_a_seven_bit_packet() {
        let mut bits = EnigmaBitReader {
            bytes: &[0xE0, 0x00],
            input_position: 0,
            current_word_d5: 0xAA80,
            bits_remaining_d6: 9,
        };

        assert_eq!(
            read_inline_value_sub_13588(&mut bits, 0, 0, 7)
                .expect("ASM sub_13588 should read the full packet via the rolling refill helper"),
            0x0020
        );
        assert!(
            bits.bits_remaining_d6 >= 9,
            "rolling refill should leave enough buffered bits for the next Enigma prefix"
        );
        assert_eq!(bits.input_position, 1);
        assert_eq!(
            bits.read_prefix_eni_dec()
                .expect("next Enigma prefix should still be readable after a 7-bit inline packet"),
            (1, 0x0E)
        );
    }

    #[test]
    fn sub_13588_reloads_the_next_word_after_a_cross_boundary_packet() {
        let mut bits = EnigmaBitReader {
            bytes: &[0xCA, 0xFE],
            input_position: 0,
            current_word_d5: 0x000B,
            bits_remaining_d6: 4,
        };

        assert_eq!(
            read_inline_value_sub_13588(&mut bits, 0, 0, 7).expect(
                "ASM sub_13588 should splice packet bits from the current tail and next word"
            ),
            0x005E
        );
        assert_eq!(bits.current_word_d5, 0xCAFE);
        assert_eq!(bits.bits_remaining_d6, 13);
        assert_eq!(bits.input_position, 2);
    }

    #[test]
    fn terminator_prefix_allows_eof_without_a_refill_byte() {
        let mut bits = EnigmaBitReader {
            bytes: &[],
            input_position: 0,
            current_word_d5: 0x3F80,
            bits_remaining_d6: 14,
        };

        assert_eq!(
            bits.read_prefix_eni_dec().expect(
                "ASM EniDec terminator should not require a byte past EOF in extracted assets"
            ),
            (7, 0x0F)
        );
        assert_eq!(bits.input_position, 0);
        assert_eq!(bits.bits_remaining_d6, 7);
    }

    #[test]
    fn enigma_inline_packet_crosses_into_the_reloaded_word() {
        let decoded = decode_enigma(
            &[
                0x0A, 0x00, 0x00, 0x00, 0x00, 0x00, // header
                0xE1, 0x55, 0x7F, 0x80,
                0x00, // command 7,count 0; inline 0x2AA; terminator; refill tail byte
            ],
            0,
        )
        .expect("ASM EniDec should finish after an inline packet that crosses the initial word");

        assert_eq!(decoded, vec![0x02AA]);
    }

    #[test]
    fn eni_dec_parses_base_relative_header_fields() {
        let header = parse_enigma_header_eni_dec(ENIGMA_FIXTURE, 0x5F60)
            .expect("ASM EniDec should parse the Enigma header prefix");

        assert_eq!(header.packet_length, 5);
        assert_eq!(header.flag_mask, 0);
        assert_eq!(header.incrementing_value, 0x5F61);
        assert_eq!(header.common_value, 0x5F6A);
        assert_eq!(header.bitstream, &ENIGMA_FIXTURE[6..]);
    }

    #[test]
    fn enigma_fixture_decodes_to_expected_tiles_with_base_tile_offset() {
        let decoded = decode_enigma(ENIGMA_FIXTURE, 0x0040).expect("fixture should decode");
        assert_eq!(
            decoded,
            vec![0x0041, 0x0042, 0x0043, 0x004A, 0x004A, 0x0050]
        );
    }

    #[test]
    fn enigma_common_run_fixture_decodes_to_a_full_plane_row() {
        let decoded = decode_enigma(ENIGMA_COMMON_RUN_FIXTURE, 0).expect("fixture should decode");

        assert_eq!(
            decoded.len(),
            32,
            "ASM EniDec should honour two 16-word common-value runs before the terminator"
        );
        assert!(decoded.iter().all(|&word| word == 0));
    }

    #[test]
    fn sub_144da_prefers_non_zero_base_nibbles() {
        let composed = compose_indoor_rubber_block_art_sub_144DA(
            &[0xF0, 0x0F, 0xA5, 0x00],
            &[0x12, 0x34, 0x67, 0x89],
        );

        assert_eq!(
            composed,
            vec![0xF2, 0x3F, 0xA5, 0x89],
            "ASM sub_144DA should preserve non-zero indoor base nibbles and only fill the empty nibbles from the rubber overlay art"
        );
    }

    #[test]
    fn sub_14582_builds_vram_read_control_word() {
        assert_eq!(
            vram_read_control_word_sub_14582(0x026E),
            0x026E_0000,
            "ASM sub_14582 should emit the VRAM-read control word for word address $026E"
        );
        assert_eq!(
            vram_read_control_word_sub_14582(0x5F60),
            0x1F60_0001,
            "ASM sub_14582 should mask the VRAM word address to 14 bits and carry the original high address bits into the low control word"
        );
    }

    #[test]
    fn export_split_bundle_writes_decoded_level_entries() {
        let source = TempDir::new().expect("temp dir");
        let output = TempDir::new().expect("temp dir");
        let source_root = source.path();
        write_default_test_bundle_source(source_root);

        let manifest = export_split_bundle(source_root, output.path(), "abc123", "deadbeef")
            .expect("bundle export should succeed");

        assert_eq!(manifest.generated_bundle_version, GENERATED_BUNDLE_VERSION);
        assert!(!manifest.decoded_levels.is_empty());
        let decoded_level0 = manifest
            .decoded_levels
            .iter()
            .find(|d| d.level_id == 0x00)
            .expect("decoded level 0x00 should be present");
        assert_eq!(decoded_level0.contract_version, DECODED_LEVEL_ASSET_VERSION);
        assert_eq!(decoded_level0.level_id, 0x00);
        assert_eq!(
            fs::read(output.path().join(&decoded_level0.foreground.relative_path))
                .expect("decoded foreground"),
            b"ABCD"
        );
        assert_eq!(
            fs::read(output.path().join(&decoded_level0.block.relative_path))
                .expect("decoded block"),
            REAL_STYLE_BLOCK_FIXTURE
        );
        assert_eq!(
            fs::read(output.path().join(&decoded_level0.background.relative_path))
                .expect("decoded background"),
            REAL_STYLE_BACKGROUND_FIXTURE
        );
        assert!(
            output
                .path()
                .join(&decoded_level0.foreground.preview_png_path)
                .is_file()
        );
        assert!(
            output
                .path()
                .join(&decoded_level0.block.preview_png_path)
                .is_file()
        );
        assert!(
            output
                .path()
                .join(&decoded_level0.background.preview_png_path)
                .is_file()
        );
        assert_eq!(manifest.rom_sha1, "abc123");
        assert_eq!(manifest.levels.len(), 1);
        assert_eq!(
            manifest.levels[0].resources.platform_script.as_deref(),
            Some("platform/00.asm")
        );
        assert_eq!(
            manifest.levels[0]
                .header_metadata
                .as_ref()
                .map(|header| header.width_screens),
            Some(0x06)
        );
        assert_eq!(
            manifest.levels[0]
                .enemy_layout_metadata
                .as_ref()
                .map(|layout| layout.object_count),
            Some(0x01)
        );
        assert_eq!(
            manifest.levels[0]
                .bgscroll_metadata
                .as_ref()
                .map(|profile| (profile.layer_speeds.clone(), profile.has_terminator)),
            Some((vec![0x10, 0x10, 0x40, 0x60], true))
        );
        assert_eq!(
            manifest.levels[0]
                .platform_metadata
                .as_ref()
                .map(|layout| layout.placements.len()),
            Some(1)
        );
        assert_eq!(manifest.enemy_sources.len(), 1);
        assert_eq!(manifest.theme_assets.len(), 1);
        assert_eq!(manifest.ingame_assets.len(), 11);
        assert!(
            manifest
                .ingame_assets
                .iter()
                .any(|asset| asset.relative_path == "anim/idle.bin")
        );
        assert!(
            manifest
                .ingame_assets
                .iter()
                .any(|asset| asset.relative_path == "artcomp/Blocks.bin")
        );
        assert!(
            manifest
                .ingame_assets
                .iter()
                .any(|asset| asset.relative_path == "artcomp/HUD_numbers.bin")
        );
        assert!(
            manifest
                .ingame_assets
                .iter()
                .any(|asset| asset.relative_path == "artunc/diamond_1.bin")
        );
        assert!(
            manifest
                .ingame_assets
                .iter()
                .any(|asset| asset.relative_path == "palette_enemy/Crystal_1.bin")
        );
        assert!(
            manifest
                .ingame_assets
                .iter()
                .any(|asset| asset.relative_path == "palette_enemy/Mini_Hopping_Skull_1.bin")
        );
        assert!(
            manifest
                .ingame_assets
                .iter()
                .any(|asset| asset.relative_path == "palette_kid/Kid.bin")
        );
        assert!(
            manifest
                .ingame_assets
                .iter()
                .any(|asset| asset.relative_path == "artunc_kid/kid.bin")
        );
        assert!(
            manifest
                .ingame_assets
                .iter()
                .any(|asset| asset.relative_path == "artunc_kid/kid_walk_1.bin")
        );
        assert!(
            manifest
                .ingame_assets
                .iter()
                .any(|asset| asset.relative_path == "artunc_kid/kid_jumping_1.bin")
        );
        assert!(
            manifest
                .ingame_assets
                .iter()
                .any(|asset| asset.relative_path == "artunc_kid/kid_jumping_2.bin")
        );
        assert!(manifest_path(output.path()).is_file());

        let level_bundle =
            load_level_asset_bundle(&manifest, 0x00).expect("level bundle should load");
        assert_eq!(level_bundle.level_id, 0x00);
        assert_eq!(
            level_bundle.display_name.as_deref(),
            Some("Blue Lake Woods 1")
        );
        assert_eq!(
            level_bundle.title_card.as_ref(),
            Some(&LevelTitleCard {
                script_bytes: b"blue\0lake\0woods\xFF".to_vec(),
                line_destinations: vec![
                    TitleCardDestination { x: 0x20, y: 0x28 },
                    TitleCardDestination { x: 0x20, y: 0x48 },
                    TitleCardDestination { x: 0x20, y: 0x68 },
                ],
                level_number_destination: Some(TitleCardDestination { x: 0xDC, y: 0x50 }),
                level_number_glyph_count: 1,
            })
        );
        assert_eq!(
            level_bundle
                .platform_script
                .as_ref()
                .map(|asset| asset.bytes.as_slice()),
            Some(
                &b"\tptfm\t0,0,0,0,0,0,0,0,0,0,PlatformScript_Nothing-PlatformScript_BaseAddress\n\tdc.w\t$FFFF\n"
                    [..]
            )
        );
        assert_eq!(
            level_bundle
                .parsed_header
                .as_ref()
                .map(|header| header.width_screens),
            Some(0x06)
        );
        assert_eq!(
            level_bundle
                .parsed_header
                .as_ref()
                .map(|header| header.flag_y),
            Some(0x0300)
        );
        assert_eq!(
            level_bundle
                .parsed_bgscroll
                .as_ref()
                .map(|profile| (profile.layer_speeds.clone(), profile.has_terminator)),
            Some((vec![0x10, 0x10, 0x40, 0x60], true))
        );
        assert_eq!(
            level_bundle
                .parsed_platform_layout
                .as_ref()
                .map(|layout| layout.placements.len()),
            Some(1)
        );
        assert_eq!(
            level_bundle
                .parsed_enemy_layout
                .as_ref()
                .map(|layout| layout.object_count),
            Some(0x01)
        );
        assert_eq!(
            level_bundle
                .parsed_enemy_layout
                .as_ref()
                .and_then(|layout| layout.objects.first())
                .map(|object| object.object_type),
            Some(0x80)
        );
    }

    #[test]
    fn load_juggernaut_skull_and_star_art_sub_129ce_reads_the_expected_ingame_assets() {
        let source = TempDir::new().expect("temp dir");
        let output = TempDir::new().expect("temp dir");
        write_default_test_bundle_source(source.path());
        write_sub_129ce_test_assets(source.path());
        let manifest = export_split_bundle(source.path(), output.path(), "abc123", "deadbeef")
            .expect("bundle should export");

        let uploads = load_juggernaut_skull_and_star_art_sub_129CE(&manifest)
            .expect("ASM sub_129CE assets should load");

        assert_eq!(
            uploads
                .iter()
                .map(|upload| upload.vram_word_address)
                .collect::<Vec<_>>(),
            vec![0xCDC0, 0xCDE0, 0xDF40]
        );
        assert_eq!(
            uploads
                .iter()
                .map(|upload| upload.asset.relative_path.as_str())
                .collect::<Vec<_>>(),
            vec![
                "artunc/Juggernaut_skull_frame_1.bin",
                "artunc/Juggernaut_skull_frame_2.bin",
                "artunc/Some_kind_of_star.bin",
            ]
        );
        assert_eq!(uploads[0].asset.bytes, vec![0x11; 32 * 4]);
        assert_eq!(uploads[1].asset.bytes, vec![0x22; 32 * 4]);
        assert_eq!(uploads[2].asset.bytes, vec![0x33; 32 * 4]);
    }

    #[test]
    fn load_hitpoint_display_art_sub_12c24_reads_the_expected_ingame_asset() {
        let source = TempDir::new().expect("temp dir");
        let output = TempDir::new().expect("temp dir");
        write_default_test_bundle_source(source.path());
        write_sub_12c24_test_asset(source.path());
        let manifest = export_split_bundle(source.path(), output.path(), "abc123", "deadbeef")
            .expect("bundle should export");

        let upload = load_hitpoint_display_art_sub_12C24(&manifest)
            .expect("ASM sub_12C24 asset should load");

        assert_eq!(upload.vram_word_address, 0x45A0);
        assert_eq!(upload.asset.relative_path, "artunc/Hitpoint_display.bin");
        assert_eq!(upload.asset.bytes, vec![0x44; 32 * 4]);
    }

    #[test]
    fn title_screen_load_plan_sub_1b2a4_uses_default_region_assets_without_overlay_palette() {
        let plan = title_screen_load_plan_sub_1B2A4(0);

        assert_eq!(
            plan.region_variant,
            TitleScreenRegionVariantSub1B2A4::Default
        );
        assert_eq!(
            plan.art_uploads
                .iter()
                .map(|upload| (upload.vram_word_address, upload.asset_label))
                .collect::<Vec<_>>(),
            vec![
                (0x1820, "byte_2A756"),
                (0x21E0, "byte_26E3D"),
                (0x6620, "byte_2A961"),
                (0xFA60, "ArtComp_1DA86_LettersNumbers"),
            ],
            "ASM sub_1B2A4 should keep the default title art at VRAM $21E0 when ($FFFFFC82).w is zero",
        );
        assert_eq!(
            plan.tilemap_load,
            SymbolicEnigmaTilemapLoadSub1B2A4 {
                vram_word_address: 0x20C1,
                asset_label: "unk_2F7D0",
            }
        );
        assert_eq!(
            plan.tile_patch_words,
            TITLE_SCREEN_TILE_PATCH_WORDS_SUB_1B2A4
        );
        assert_eq!(
            plan.palette_loads,
            vec![SymbolicPaletteLoadSub1B2A4 {
                destination_word_offset: 0x0000,
                palette_label: "Pal_1C400",
                word_count: 0x0033,
            }],
            "ASM loc_1C4B2 should copy only the base palette when ($FFFFFC82).w is zero",
        );
    }

    #[test]
    fn title_intro_load_plan_sub_1b850_replays_the_rom_upload_order() {
        let plan = title_intro_load_plan_sub_1B850(&(0u16..28).collect::<Vec<_>>())
            .expect("ASM sub_1B850 should accept the 28 decoded words consumed by loc_1B8E4");

        assert_eq!(
            plan.art_uploads
                .iter()
                .map(|upload| (upload.vram_word_address, upload.asset_label))
                .collect::<Vec<_>>(),
            vec![
                (0x1780, "byte_2D73B"),
                (0x2740, "byte_2D71A"),
                (0x2800, "byte_2DEBF"),
                (0x3AC0, "byte_261D7"),
                (0xFA60, "ArtComp_1DA86_LettersNumbers"),
            ],
            "ASM sub_1B850 should queue the five title-intro art decompresses in ROM order",
        );
        assert_eq!(
            plan.tilemap_load,
            SymbolicEnigmaTilemapLoadSub1B850 {
                vram_word_address: 0x0000,
                asset_label: "unk_2FDE0",
                width_words: 40,
                height_words: 28,
            },
            "ASM sub_1B850 should decode `unk_2FDE0` and hand it to sub_1C5D0 as a 40x28 tilemap at VRAM word address 0",
        );
    }

    #[test]
    fn title_intro_load_plan_sub_1b850_repeats_each_second_tilemap_word_across_a_full_row() {
        let decoded_row_words = (0u16..28).collect::<Vec<_>>();
        let plan = title_intro_load_plan_sub_1B850(&decoded_row_words).expect(
            "ASM sub_1B850 should keep the second tilemap blit when 28 row words are available",
        );

        assert_eq!(plan.repeated_rows_upload.vram_byte_address, 0xE000);
        assert_eq!(plan.repeated_rows_upload.rows.len(), 28);
        assert_eq!(plan.repeated_rows_upload.rows[0], [0; 64]);
        assert_eq!(plan.repeated_rows_upload.rows[1], [1; 64]);
        assert_eq!(plan.repeated_rows_upload.rows[27], [27; 64]);
    }

    #[test]
    fn title_screen_load_plan_sub_1b2a4_switches_art_and_palette_overlay_for_non_zero_region() {
        let region_1 = title_screen_load_plan_sub_1B2A4(1);
        let region_2 = title_screen_load_plan_sub_1B2A4(2);

        assert_eq!(
            region_1.region_variant,
            TitleScreenRegionVariantSub1B2A4::Variant1
        );
        assert_eq!(region_1.art_uploads[1].asset_label, "byte_2927C");
        assert_eq!(
            region_1.palette_loads[1],
            SymbolicPaletteLoadSub1B2A4 {
                destination_word_offset: 0x001A,
                palette_label: "Pal_1C466",
                word_count: 0x0013,
            },
            "ASM loc_1C4B2 should overlay Pal_1C466 at Palette_Buffer+$1A for region byte 1",
        );

        assert_eq!(
            region_2.region_variant,
            TitleScreenRegionVariantSub1B2A4::Variant2
        );
        assert_eq!(region_2.art_uploads[1].asset_label, "byte_282C1");
        assert_eq!(
            region_2.palette_loads[1],
            SymbolicPaletteLoadSub1B2A4 {
                destination_word_offset: 0x001A,
                palette_label: "Pal_1C48C",
                word_count: 0x0013,
            },
            "ASM loc_1C4B2 should switch to Pal_1C48C once ($FFFFFC82).w is non-zero and not 1",
        );
    }

    #[test]
    fn title_intro_palette_load_sub_1c4ea_copies_intro1_wildside_into_palette_buffer_2() {
        assert_eq!(
            title_intro_palette_load_sub_1C4EA(),
            SymbolicPaletteBuffer2Load {
                destination_word_offset: 0x0000,
                palette_label: "Pal_1C28A",
                word_count: 0x0033,
            },
            "ASM sub_1C4EA should seed Palette_Buffer_2 from `Pal_1C28A` for the Intro 1 Wild Side palette",
        );
    }

    #[test]
    fn title_intro_palette_load_sub_1c4fe_copies_intro2_sky_into_palette_buffer_2() {
        assert_eq!(
            title_intro_palette_load_sub_1C4FE(),
            SymbolicPaletteBuffer2Load {
                destination_word_offset: 0x0000,
                palette_label: "Pal_1C2F0",
                word_count: 0x0023,
            },
            "ASM sub_1C4FE should seed Palette_Buffer_2 from `Pal_1C2F0` for the Intro 2 Sky palette",
        );
    }

    #[test]
    fn title_character_palette_load_sub_1c512_copies_the_base_character_palette() {
        assert_eq!(
            title_character_palette_load_sub_1C512(0),
            TitleCharacterPaletteLoadSub1C512 {
                base_load: SymbolicPaletteBuffer2Load {
                    destination_word_offset: 0x0000,
                    palette_label: "Pal_1C346",
                    word_count: 0x0040,
                },
                variant_overlay: None,
            },
            "ASM sub_1C512 should always seed Palette_Buffer_2 from `Pal_1C346` before any region-specific overlay",
        );
    }

    #[test]
    fn title_character_palette_load_sub_1c512_uses_pal_1c3c6_for_region_byte_1() {
        assert_eq!(
            title_character_palette_load_sub_1C512(1).variant_overlay,
            Some(SymbolicPaletteBuffer2Load {
                destination_word_offset: 0x0031,
                palette_label: "Pal_1C3C6",
                word_count: 0x000E,
            }),
            "ASM sub_1C512 overlays `Pal_1C3C6` at Palette_Buffer_2+$62 when ($FFFFFC82).w equals 1",
        );
    }

    #[test]
    fn title_character_palette_load_sub_1c512_uses_pal_1c3d4_for_non_zero_non_one_region_bytes() {
        assert_eq!(
            title_character_palette_load_sub_1C512(2).variant_overlay,
            Some(SymbolicPaletteBuffer2Load {
                destination_word_offset: 0x0031,
                palette_label: "Pal_1C3D4",
                word_count: 0x000E,
            }),
            "ASM sub_1C512 switches to `Pal_1C3D4` for any non-zero region byte other than 1",
        );
    }

    #[test]
    fn title_intro_glyph_write_sub_1c7f6_encodes_the_first_intro_text_glyph() {
        assert_eq!(
            title_intro_glyph_write_sub_1C7F6(0x0000, 0x67D3, 2, 8, u16::from(b'T' - b'A')),
            TitleIntroGlyphWriteSub1C7F6 {
                vram_word_address: 0x0404,
                vdp_control_longword: 0x4404_0000,
                tile_word: 0x67E6,
            },
            "ASM sub_1C7F6 should map IntroText1's first glyph to VRAM word $0404 and tile word $67E6",
        );
    }

    #[test]
    fn title_intro_glyph_write_sub_1c7f6_preserves_the_low_vdp_address_bits() {
        assert_eq!(
            title_intro_glyph_write_sub_1C7F6(0xE000, 0xC7D3, 7, 3, u16::from(b'I' - b'A')),
            TitleIntroGlyphWriteSub1C7F6 {
                vram_word_address: 0xE18E,
                vdp_control_longword: 0x618E_0003,
                tile_word: 0xC7DB,
            },
            "ASM sub_1C7F6 should preserve the rotated low two VRAM address bits in the VDP control longword",
        );
    }

    #[test]
    fn decompress_to_vram_setup_sub_142cc_keeps_the_payload_and_command_stream_split() {
        let setup =
            decompress_to_vram_setup_sub_142CC(&[0x00, 0x06, 0xAA, 0xBB, 0xCC, 0xDD], 0xE18E)
                .expect("ASM sub_142CC should accept the two-byte command-stream offset header");

        assert_eq!(
            setup,
            DecompressToVramSetupSub142CC {
                vram_word_address: 0xE18E,
                vdp_control_longword: 0x618E_0003,
                input_stream_offset: 2,
                command_stream_offset: 8,
                bit_reader_word: 0,
                bit_index: 0,
            },
            "ASM sub_142CC should leave `a0` on the payload bytes, move `a1` by the big-endian offset beyond the two-byte header, and zero the bit-reader state",
        );
    }

    #[test]
    fn decompress_to_vram_setup_sub_142cc_rejects_missing_offset_header() {
        let error = decompress_to_vram_setup_sub_142CC(&[0x34], 0x4000)
            .expect_err("ASM sub_142CC needs both bytes of the command-stream offset");

        assert!(
            error
                .to_string()
                .contains("two-byte command-stream offset header"),
            "the failure should explain that sub_142CC cannot split payload/command streams without the two-byte header",
        );
    }

    #[test]
    fn ending_sequence_palette_load_sub_1c820_copies_the_full_base_palette_without_region_overlay()
    {
        assert_eq!(
            ending_sequence_palette_load_sub_1C820(0),
            EndingSequencePaletteLoadSub1C820 {
                base_loads: [
                    SymbolicPaletteBuffer2Load {
                        destination_word_offset: 0x0000,
                        palette_label: "Pal_1C8AA",
                        word_count: 0x0010,
                    },
                    SymbolicPaletteBuffer2Load {
                        destination_word_offset: 0x0010,
                        palette_label: "Pal_1C8AA",
                        word_count: 0x0030,
                    },
                ],
                variant_overlay: None,
            },
            "ASM sub_1C820 should copy `Pal_1C8AA` into Palette_Buffer in a $10-word block followed by a $30-word block when d1 is zero",
        );
    }

    #[test]
    fn ending_sequence_palette_load_sub_1c820_overlays_the_region_variant_words_when_d1_is_non_zero()
     {
        assert_eq!(
            ending_sequence_palette_load_sub_1C820(1).variant_overlay,
            Some(PaletteWordOverlaySub1C820 {
                destination_word_offset: 0x0009,
                words: [0x0040, 0x00A0, 0xE004, 0x00EE, 0x0EEE],
            }),
            "ASM sub_1C820 should rewind Palette_Buffer by seven words and stamp the five hard-coded variant colours when d1 is non-zero",
        );
    }

    #[test]
    fn damage_flash_palette_clear_sub_80bc_fills_twelve_words_with_fff_at_palette_buffer_62() {
        assert_eq!(
            damage_flash_palette_clear_sub_80BC(),
            PaletteWordOverlaySub80BC {
                destination_word_offset: 0x0031,
                words: [0x0FFF; 12],
            },
            "ASM sub_80BC should fill 12 words at Palette_Buffer+$62 with `$0FFF` before the damage flash restores the helmet palette",
        );
    }

    #[test]
    fn damage_flash_helmet_palette_load_sub_80d0_maps_each_helmet_to_the_12_word_overlay() {
        let expected = [
            "Pal_A1C72",
            "Pal_A1CE4",
            "Pal_A1E1C",
            "Pal_A1DA4",
            "Pal_A1DE0",
            "Pal_A1E58",
            "Pal_A1E94",
            "Pal_A1D68",
            "Pal_A1C8A",
            "Pal_A1D26",
        ];

        for (helmet_id, palette_label) in expected.into_iter().enumerate() {
            assert_eq!(
                damage_flash_helmet_palette_load_sub_80D0(helmet_id as u16),
                Some(SymbolicPaletteLoadSub1B2A4 {
                    destination_word_offset: 0x0031,
                    palette_label,
                    word_count: 0x000C,
                }),
                "ASM sub_80D0 should copy the 12-word helmet palette {palette_label} into Palette_Buffer+$62 for helmet index {helmet_id}",
            );
        }

        assert_eq!(
            damage_flash_helmet_palette_load_sub_80D0(10),
            None,
            "native should reject out-of-range helmet indices instead of inventing a palette past ASM off_80F2",
        );
    }

    #[test]
    fn helmet_pose_palette_load_sub_8106_maps_each_helmet_to_the_15_word_overlay() {
        let expected = [
            "Pal_A1C72",
            "Pal_A1D08",
            "Pal_A1E3A",
            "Pal_A1DC2",
            "Pal_A1DFE",
            "Pal_A1E76",
            "Pal_A1EB2",
            "Pal_A1D86",
            "Pal_A1CA8",
            "Pal_A1D4A",
        ];

        for (helmet_id, palette_label) in expected.into_iter().enumerate() {
            assert_eq!(
                helmet_pose_palette_load_sub_8106(helmet_id as u16),
                Some(SymbolicPaletteLoadSub1B2A4 {
                    destination_word_offset: 0x0031,
                    palette_label,
                    word_count: 0x000F,
                }),
                "ASM sub_8106 should copy the 15-word helmet palette {palette_label} into Palette_Buffer+$62 for helmet index {helmet_id}",
            );
        }

        assert_eq!(
            helmet_pose_palette_load_sub_8106(10),
            None,
            "native should reject out-of-range helmet indices instead of inventing a palette past ASM off_8128",
        );
    }

    #[test]
    fn option_menu_palette_load_sub_1c896_copies_the_full_options_palette() {
        assert_eq!(
            option_menu_palette_load_sub_1C896(),
            SymbolicPaletteLoadSub1B2A4 {
                destination_word_offset: 0x0000,
                palette_label: "Pal_1C97C",
                word_count: 0x0040,
            },
            "ASM sub_1C896 should copy all `$40` words of `Pal_1C97C` into `Palette_Buffer` for the options menu",
        );
    }

    #[test]
    fn option_menu_tile_split_upload_sub_1ccae_splits_each_four_word_chunk_into_low_then_high_nibbles()
     {
        let mut source_words = vec![0u16; OPTION_MENU_TILE_SPLIT_WORD_COUNT_SUB_1CCAE];
        source_words[..8].copy_from_slice(&[
            0x1234, 0xABCD, 0x0F0F, 0xF0F0, 0x2468, 0x1357, 0xAAAA, 0x5555,
        ]);

        let upload = option_menu_tile_split_upload_sub_1CCAE(&source_words)
            .expect("ASM sub_1CCAE should accept the full `$1180`-word VRAM source span");

        assert_eq!(upload.upper_vram_word_address, 0x7880);
        assert_eq!(upload.lower_vram_word_address, 0x5580);
        assert_eq!(
            &upload.upper_words[..8],
            &[
                0x0204, 0x0B0D, 0x0000, 0xF0F0, 0x0408, 0x0307, 0xA0A0, 0x5050
            ],
            "ASM sub_1CCAE/sub_1CD20/sub_1CD32 should emit low-low-high-high masked words into the first VRAM upload",
        );
        assert_eq!(
            &upload.lower_words[..8],
            &[
                0x1030, 0xA0C0, 0x0F0F, 0x0000, 0x2060, 0x1050, 0x0A0A, 0x0505
            ],
            "ASM sub_1CCAE should emit the inverse high-high-low-low ordering into the second VRAM upload",
        );
        assert_eq!(
            upload.upper_words.len(),
            OPTION_MENU_TILE_SPLIT_WORD_COUNT_SUB_1CCAE,
            "sub_1CCAE should preserve the full `$1180`-word span in the first upload",
        );
        assert_eq!(
            upload.lower_words.len(),
            OPTION_MENU_TILE_SPLIT_WORD_COUNT_SUB_1CCAE,
            "sub_1CCAE should preserve the full `$1180`-word span in the second upload",
        );
    }

    #[test]
    fn option_menu_tile_split_upload_sub_1ccae_rejects_short_vram_sources() {
        let error = option_menu_tile_split_upload_sub_1CCAE(&vec![
            0;
            OPTION_MENU_TILE_SPLIT_WORD_COUNT_SUB_1CCAE
                - 1
        ])
        .expect_err("ASM sub_1CCAE should require the full `$1180`-word VRAM source span");

        assert!(
            error.to_string().contains("sub_1CCAE needs 0x1180 words"),
            "the error should describe the exact ROM-sized VRAM source requirement",
        );
    }

    #[test]
    fn clear_bit15_vram_upload_sub_1cd44_masks_an_800_word_vram_span_and_rewrites_it_to_zero() {
        let mut source_words = vec![0u16; 0x0800];
        source_words[..4].copy_from_slice(&[0x8001, 0xFFFF, 0x7FFF, 0x1234]);

        let upload = clear_bit15_vram_upload_sub_1CD44(&source_words)
            .expect("ASM sub_1CD44 should accept the full `$800`-word VRAM span");

        assert_eq!(upload.read_vram_byte_address, 0x0000);
        assert_eq!(upload.write_vram_byte_address, 0x0000);
        assert_eq!(upload.delay_loop_iterations, 0x0101);
        assert_eq!(
            &upload.masked_words[..4],
            &[0x0001, 0x7FFF, 0x7FFF, 0x1234],
            "ASM sub_1CD44 should clear only bit 15 via `bclr #$F,d6` before writing back to VRAM 0",
        );
        assert_eq!(
            upload.masked_words.len(),
            0x0800,
            "ASM sub_1CD44 should preserve the full `$800`-word VRAM transfer span",
        );
    }

    #[test]
    fn clear_bit15_vram_upload_sub_1cd44_rejects_short_vram_spans() {
        let error = clear_bit15_vram_upload_sub_1CD44(&vec![0; 0x07FF])
            .expect_err("ASM sub_1CD44 should require the full `$800`-word VRAM span");

        assert!(
            error
                .to_string()
                .contains("sub_1CD44 needs 0x0800 source words"),
            "the error should describe the exact ROM-sized VRAM source requirement",
        );
    }

    #[test]
    fn decompression_buffer_rows_upload_sub_1cd88_packs_28_rows_of_40_words_plus_24_fill_tiles() {
        let decoded_words = (0u16..(28 * 40) as u16).collect::<Vec<_>>();

        let upload = decompression_buffer_rows_upload_sub_1CD88(&decoded_words)
            .expect("ASM sub_1CD88 should accept the full `28*40` decoded-word span");

        assert_eq!(
            upload.rows[0][..44],
            [
                0x0000, 0x0001, 0x0002, 0x0003, 0x0004, 0x0005, 0x0006, 0x0007, 0x0008, 0x0009,
                0x000A, 0x000B, 0x000C, 0x000D, 0x000E, 0x000F, 0x0010, 0x0011, 0x0012, 0x0013,
                0x0014, 0x0015, 0x0016, 0x0017, 0x0018, 0x0019, 0x001A, 0x001B, 0x001C, 0x001D,
                0x001E, 0x001F, 0x0020, 0x0021, 0x0022, 0x0023, 0x0024, 0x0025, 0x0026, 0x0027,
                0x0094, 0x0094, 0x0094, 0x0094
            ],
            "ASM sub_1CD88 should write 40 sequential `Decompression_Buffer` words then pad the row with `$94`",
        );
        assert_eq!(
            upload.rows[27][0],
            27 * 40,
            "ASM sub_1CD88 should keep advancing `a0` across all 28 rows rather than restarting each row",
        );
        assert!(
            upload
                .rows
                .iter()
                .all(|row| row[40..].iter().all(|&word| word == 0x0094)),
            "ASM sub_1CD88 should stamp `$94` into the trailing 24 columns of every row",
        );
    }

    #[test]
    fn decompression_buffer_rows_upload_sub_1cd88_rejects_short_decoded_spans() {
        let error = decompression_buffer_rows_upload_sub_1CD88(&vec![0; 28 * 40 - 1])
            .expect_err("ASM sub_1CD88 should require all 28 decoded rows");

        assert!(
            error
                .to_string()
                .contains("sub_1CD88 needs 0x0460 decoded words"),
            "the error should describe the exact decoded-word requirement",
        );
    }

    #[test]
    fn plane_line_upload_sub_1cda8_uses_a_50_word_source_stride_and_80_byte_vram_row_stride() {
        let decoded_words = (0u16..0x1000).collect::<Vec<_>>();

        let upload = plane_line_upload_sub_1CDA8(&decoded_words, 0x001B, 0x001F, 0xE000)
            .expect("ASM sub_1CDA8 should accept the requested source row");

        assert_eq!(upload.source_word_offset, 0x0870);
        assert_eq!(upload.vram_byte_address, 0xEF80);
        assert_eq!(upload.vdp_control_longword, 0x6F80_0003);
        assert_eq!(upload.words[0], 0x0870);
        assert_eq!(upload.words[39], 0x0897);
        assert_eq!(
            upload.words.len(),
            40,
            "ASM sub_1CDA8 should stream exactly one 40-word plane row from the decoded buffer",
        );
    }

    #[test]
    fn plane_line_upload_sub_1cda8_rejects_source_rows_without_40_words_available() {
        let error = plane_line_upload_sub_1CDA8(&vec![0; 0x0870 + 39], 0x001B, 0x001F, 0xE000)
            .expect_err("ASM sub_1CDA8 should require 40 source words for the requested row");

        assert!(
            error
                .to_string()
                .contains("sub_1CDA8 needs row 27 words [0x0870..0x0898)"),
            "the error should describe the exact row slice requested by the ASM stride math",
        );
    }

    #[test]
    fn filled_plane_line_upload_sub_1cdfa_targets_row_offsets_from_minus_2000_plus_c() {
        let upload = filled_plane_line_upload_sub_1CDFA(9);

        assert_eq!(upload.row_index, 9);
        assert_eq!(upload.vram_byte_address, 0xE48C);
        assert_eq!(upload.vdp_control_longword, 0x648C_0003);
        assert_eq!(
            upload.words, [0x0094; 28],
            "ASM sub_1CDFA should stamp exactly 28 `$94` fill words into the target row",
        );
    }

    #[test]
    fn end_sequence_plane_clear_plan_sub_1cde0_covers_rows_9_through_28_in_pairs() {
        let plan = end_sequence_plane_clear_plan_sub_1CDE0();

        assert_eq!(
            plan.display_refresh_count, 10,
            "ASM sub_1CDE0 should call sub_1DA24 once per `dbf d1` iteration",
        );
        assert_eq!(
            plan.line_uploads.len(),
            20,
            "ASM sub_1CDE0 should emit two sub_1CDFA row fills across ten loop iterations",
        );
        assert_eq!(
            plan.line_uploads.first().map(|upload| upload.row_index),
            Some(9)
        );
        assert_eq!(
            plan.line_uploads.get(1).map(|upload| upload.row_index),
            Some(10)
        );
        assert_eq!(
            plan.line_uploads.last().map(|upload| upload.row_index),
            Some(28)
        );
        assert!(
            plan.line_uploads
                .windows(2)
                .all(|pair| pair[1].row_index == pair[0].row_index + 1)
        );
    }

    #[test]
    fn title_panel_line_upload_sub_1d990_uses_a_1c_word_source_stride_and_1a_byte_row_base() {
        let decoded_words = (0u16..0x0400).collect::<Vec<_>>();

        let upload = title_panel_line_upload_sub_1D990(&decoded_words, 0x000D, 0x00)
            .expect("ASM sub_1D990 should accept a decoded span covering row 13");

        assert_eq!(upload.row_index, 0x000D);
        assert_eq!(upload.source_word_offset, 0x016C);
        assert_eq!(upload.vram_byte_address, 0x0A1A);
        assert_eq!(upload.vdp_control_longword, 0x4A1A_0000);
        assert_eq!(upload.words[0], 0x016C);
        assert_eq!(upload.words[13], 0x0179);
        assert_eq!(
            upload.words.len(),
            14,
            "ASM sub_1D990 should stream exactly 14 words from the selected `Decompression_Buffer` row",
        );
    }

    #[test]
    fn title_panel_line_upload_sub_1d990_subtracts_2000_from_the_vram_bank_when_fb5c_is_set() {
        let decoded_words = (0u16..0x0400).collect::<Vec<_>>();

        let upload = title_panel_line_upload_sub_1D990(&decoded_words, 0x000D, 0xFF).expect(
            "ASM sub_1D990 should accept the same decoded row when `($FFFFFB5C).w` is non-zero",
        );

        assert_eq!(upload.vram_byte_address, 0xEA1A);
        assert_eq!(upload.vdp_control_longword, 0x6A1A_0003);
        assert_eq!(
            upload.words[0], 0x016C,
            "ASM sub_1D990 should keep the source row unchanged when only the destination VRAM bank flips",
        );
    }

    #[test]
    fn title_panel_line_upload_sub_1d990_rejects_short_decoded_rows() {
        let error = title_panel_line_upload_sub_1D990(&vec![0; 0x016C + 13], 0x000D, 0x00)
            .expect_err("ASM sub_1D990 should require the full 14-word row slice");

        assert!(
            error
                .to_string()
                .contains("sub_1D990 needs row 13 words [0x016C..0x017A)"),
            "the error should describe the exact decoded row range requested by the ASM stride math",
        );
    }

    #[test]
    fn title_panel_load_plan_sub_1d944_selects_the_enigma_asset_flags_and_descending_row_uploads() {
        let decoded_words = (0u16..0x0400).collect::<Vec<_>>();

        let plan = title_panel_load_plan_sub_1D944(0x8004, &decoded_words)
            .expect("ASM sub_1D944 should accept selector 4 with the bit-15 flag set");

        assert_eq!(
            plan.tilemap_decode,
            SymbolicEnigmaTilemapDecodeSub1D944 {
                asset_label: "unk_2E64A",
                vram_word_address: 0x22DC,
                panel_flag_fb5c: 0xFF,
                panel_flag_fb5e: 0x00,
            },
            "ASM sub_1D944 should clear bits 15/14 into `($FFFFFB5C).w` and `($FFFFFB5E).w`, then use the remaining selector in `off_1D936`",
        );
        assert_eq!(plan.line_uploads.len(), 14);
        assert_eq!(
            plan.line_uploads[0].row_index, 13,
            "ASM sub_1D944 should start the `dbf d7` loop at row 13",
        );
        assert_eq!(
            plan.line_uploads[13].row_index, 0,
            "ASM sub_1D944 should finish the `dbf d7` loop at row 0",
        );
        assert_eq!(
            plan.line_uploads[0].vram_byte_address, 0xEA1A,
            "ASM sub_1D944 should pass the bit-15-derived `($FFFFFB5C).w` state through to sub_1D990's VRAM bank select",
        );
        assert_eq!(
            plan.line_uploads[13].vram_byte_address, 0xE39A,
            "ASM sub_1D944 should keep descending the destination rows by `$80` bytes down to row 0",
        );
    }

    #[test]
    fn title_panel_load_plan_sub_1d944_keeps_base_tile_813e_when_bit14_is_set() {
        let decoded_words = vec![0; 14 * 0x1C];

        let plan = title_panel_load_plan_sub_1D944(0x4006, &decoded_words)
            .expect("ASM sub_1D944 should accept selector 6 with only the bit-14 flag set");

        assert_eq!(plan.tilemap_decode.asset_label, "unk_2EA5E");
        assert_eq!(plan.tilemap_decode.vram_word_address, 0x813E);
        assert_eq!(plan.tilemap_decode.panel_flag_fb5c, 0x00);
        assert_eq!(plan.tilemap_decode.panel_flag_fb5e, 0xFF);
    }

    #[test]
    fn title_panel_load_plan_sub_1d944_rejects_selectors_outside_off_1d936() {
        let error = title_panel_load_plan_sub_1D944(7, &vec![0; 14 * 0x1C]).expect_err(
            "ASM sub_1D944 should reject selectors beyond the seven-entry `off_1D936` table",
        );

        assert!(
            error
                .to_string()
                .contains("sub_1D944 selector 0x0007 is outside off_1D936's 7 entries"),
            "the error should describe the exact selector overflow against `off_1D936`",
        );
    }

    #[test]
    fn vdp_data_fill_sub_1da62_repeats_94_for_an_800_word_dbf_span() {
        let fill = vdp_data_fill_sub_1DA62();

        assert_eq!(
            fill.word_count, 0x0800,
            "ASM sub_1DA62 should run the `dbf d0` loop for `$7FF+1` VDP data writes",
        );
        assert_eq!(
            fill.fill_word, 0x0094,
            "ASM sub_1DA62 should write the constant word `$94` on every loop iteration",
        );
    }

    #[test]
    fn vram_zero_fill_sub_1da72_targets_vram_1280_for_16_zero_words() {
        let fill = vram_zero_fill_sub_1DA72();

        assert_eq!(fill.vram_byte_address, 0x1280);
        assert_eq!(fill.vdp_control_longword, 0x5280_0000);
        assert_eq!(
            fill.words.len(),
            16,
            "ASM sub_1DA72 should run the `dbf d0` loop for `$F+1` zero-word writes",
        );
        assert!(
            fill.words.iter().all(|&word| word == 0),
            "ASM sub_1DA72 should clear all 16 VDP data words to zero",
        );
    }

    #[test]
    fn diamond_hud_dma_transfer_sub_5d4a_uses_the_first_off_7b0ac_entry() {
        let transfer = diamond_hud_dma_transfer_sub_5D4A(0)
            .expect("ASM sub_5D4A should accept the first 4-byte selector from `off_7B0AC`");

        assert_eq!(
            transfer,
            DiamondHudDmaTransferSub5D4A {
                frame_table_byte_offset: 0,
                asset_label: "unk_99FCA",
                dma_source_address: 0x0009_9FCC,
                dma_length_registers: [0x9300, 0x9401],
                dma_source_registers: [0x95E6, 0x96CF, 0x9704],
                vram_byte_address: 0xDD40,
            },
            "ASM sub_5D4A should skip the two-byte `dc.w $10` header on `unk_99FCA` and programme a `$200`-byte DMA into VRAM byte `$DD40`",
        );
    }

    #[test]
    fn diamond_hud_dma_transfer_sub_5d4a_uses_the_last_off_7b0ac_entry() {
        let transfer = diamond_hud_dma_transfer_sub_5D4A(0x000C)
            .expect("ASM sub_5D4A should accept the fourth 4-byte selector from `off_7B0AC`");

        assert_eq!(transfer.asset_label, "unk_9A5D0");
        assert_eq!(transfer.dma_source_address, 0x0009_A5D2);
        assert_eq!(transfer.dma_source_registers, [0x95E9, 0x96D2, 0x9704]);
        assert_eq!(transfer.vram_byte_address, 0xDD40);
    }

    #[test]
    fn diamond_hud_dma_transfer_sub_5d4a_rejects_non_longword_selectors() {
        let error = diamond_hud_dma_transfer_sub_5D4A(2)
            .expect_err("ASM sub_5D4A indexes `off_7B0AC` in longwords, not bytes");

        assert!(
            error
                .to_string()
                .contains("expects a 4-byte selector into `off_7B0AC`"),
            "the failure should explain the longword selector requirement from ASM sub_5D4A",
        );
    }

    #[test]
    fn level_layout_address_to_cell_sub_1f8e_splits_row_and_column_from_the_stride_address() {
        let coords = level_layout_address_to_cell_sub_1F8E(0xA77C, 0xA652, 0x0040);

        assert_eq!(
            coords,
            LevelLayoutCellCoordsSub1F8E {
                row_index: 4,
                column_index: 21,
            },
            "ASM sub_1F8E should keep the `divu` quotient as the row index and halve the remainder into the 2-byte cell column",
        );
    }

    #[test]
    fn level_layout_address_to_cell_sub_1f8e_preserves_the_low_address_remainder_before_halving() {
        let coords = level_layout_address_to_cell_sub_1F8E(0xA699, 0xA652, 0x0040);

        assert_eq!(coords.row_index, 1);
        assert_eq!(
            coords.column_index, 3,
            "ASM sub_1F8E should drop only the low byte-address bit via `lsr.w #1,d5` after the divide remainder is swapped down",
        );
    }

    #[test]
    fn end_sequence_text_render_plan_sub_1d080_maps_letters_to_vertical_plane_rows() {
        let plan = end_sequence_text_render_plan_sub_1D080(&[5, 7, b'A', b'Z', 0])
            .expect("ASM sub_1D080 should accept `(x,y)` plus a NUL-terminated glyph stream");

        assert_eq!(
            plan.frames_per_glyph, 3,
            "ASM sub_1D080 should present three frames after each glyph via `dbf d6`",
        );
        assert_eq!(plan.glyph_uploads.len(), 2);
        assert_eq!(plan.glyph_uploads[0].vram_byte_address, 0xE80A);
        assert_eq!(plan.glyph_uploads[0].vdp_control_longword, 0x680A_0003);
        assert_eq!(
            plan.glyph_uploads[0].glyph_tile_word, 0xE4DC,
            "ASM sub_1D080 should map ASCII `A` to tile word `$E4DC` via `subi.w #$41` / `addi.w #-$1B24`",
        );
        assert_eq!(
            plan.glyph_uploads[1].vram_byte_address, 0xE88A,
            "ASM sub_1D080 should advance one plane row (`+$80` bytes) per glyph",
        );
        assert_eq!(plan.glyph_uploads[1].glyph_tile_word, 0xE4F5);
    }

    #[test]
    fn end_sequence_text_render_plan_sub_1d080_rejects_missing_coordinates() {
        let error = end_sequence_text_render_plan_sub_1D080(&[])
            .expect_err("ASM sub_1D080 should require the leading x/y coordinate bytes");

        assert!(
            error
                .to_string()
                .contains("sub_1D080 needs an x coordinate byte"),
            "the error should describe the missing leading coordinate byte",
        );
    }

    #[test]
    fn end_sequence_holo_background_load_plan_sub_1d0ec_uses_the_holobg_art_and_tilemap_pair() {
        let plan = end_sequence_holo_background_load_plan_sub_1D0EC();

        assert_eq!(
            plan.art_decode,
            SymbolicRamAssetDecodeSub1D0EC {
                vram_word_address: 0x1780,
                asset_label: "ArtComp_C65A_HoloBG",
                remap_bytes: [
                    0x08, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0A, 0x0B, 0x0C,
                    0x0D, 0x0E, 0x0F,
                ],
            },
            "ASM sub_1D0EC should decode `ArtComp_C65A_HoloBG` to RAM with the fixed `unk_1D118` remap bytes before the ending sequence uses it",
        );
        assert_eq!(
            plan.tilemap_load,
            SymbolicEnigmaTilemapLoadSub1B2A4 {
                vram_word_address: 0x80BC,
                asset_label: "MapEni_CC0E",
            },
            "ASM sub_1D0EC should then decode `MapEni_CC0E` from the same HoloBG blob to VRAM word `$80BC` via `off_718A`",
        );
    }

    #[test]
    fn title_screen_region_byte_sub_1c278_cycles_between_the_three_title_variants() {
        assert_eq!(
            advance_title_screen_region_byte_sub_1C278(0),
            1,
            "ASM sub_1C278 should advance the default title variant to region byte 1",
        );
        assert_eq!(
            advance_title_screen_region_byte_sub_1C278(1),
            2,
            "ASM sub_1C278 should advance the second title variant to region byte 2",
        );
        assert_eq!(
            advance_title_screen_region_byte_sub_1C278(2),
            0,
            "ASM sub_1C278 should clear ($FFFFFC82).w once the byte reaches 3",
        );
    }

    #[test]
    fn title_screen_character_load_plan_sub_1bf24_keeps_frame_zero_on_the_default_tile_base() {
        let plan = title_screen_character_load_plan_sub_1BF24(0, 0)
            .expect("ASM sub_1BF24 should accept the shared frame-0 title tilemap");

        assert_eq!(
            plan.region_variant,
            TitleScreenRegionVariantSub1B2A4::Default
        );
        assert_eq!(
            plan.tilemap_load,
            SymbolicEnigmaTilemapLoadSub1BF24 {
                vram_word_address: 0x20C1,
                asset_label: "unk_2F7D0",
                width_words: 18,
                height_words: 20,
            },
            "ASM sub_1BF24 should keep frame 0 on `unk_2F7D0` with base tile `$20C1`",
        );
        assert_eq!(
            plan.plane_upload,
            PlaneRectUploadSub1BF24 {
                plane_cell_x: 0x15,
                plane_cell_y: 0x01,
                width_words: 18,
                height_words: 20,
                fill_word: 0x20BC,
            },
            "ASM sub_1BF24 should fill an 18x20 staging rectangle with `$20BC` before sub_1C5D0 blits it at `(#$15,#1)`",
        );
    }

    #[test]
    fn title_screen_character_load_plan_sub_1bf24_switches_non_zero_frames_by_region_bank() {
        let default_plan = title_screen_character_load_plan_sub_1BF24(7, 0)
            .expect("ASM sub_1BF24 should accept the final default-region title frame");
        let region_1_plan = title_screen_character_load_plan_sub_1BF24(7, 1)
            .expect("ASM sub_1BF24 should accept the final region-1 title frame");
        let region_2_plan = title_screen_character_load_plan_sub_1BF24(7, 2)
            .expect("ASM sub_1BF24 should accept the final region-2 title frame");

        assert_eq!(default_plan.tilemap_load.vram_word_address, 0x210F);
        assert_eq!(default_plan.tilemap_load.asset_label, "unk_2EF2C");
        assert_eq!(region_1_plan.tilemap_load.asset_label, "unk_2F76A");
        assert_eq!(region_2_plan.tilemap_load.asset_label, "unk_2F2D0");
    }

    #[test]
    fn intro_sequence_5_frame_load_plan_sub_1c034_replays_the_rom_frame_table() {
        let expectations = [
            ("unk_2F856", 12, 24, 3),
            ("unk_2F882", 12, 24, 3),
            ("unk_2F8D2", 12, 25, 3),
            ("unk_2F94E", 14, 25, 2),
            ("unk_2F9DE", 15, 25, 0),
            ("unk_2FAA8", 13, 25, 2),
            ("unk_2FC3A", 13, 25, 2),
            ("unk_2FCE4", 13, 25, 2),
        ];

        let actual = (0..8)
            .map(|frame_index| {
                let plan = intro_sequence_5_frame_load_plan_sub_1C034(frame_index)
                    .expect("ASM sub_1C034 should accept the 8 intro-sequence-5 frame indices");
                (
                    plan.tilemap_load.asset_label,
                    plan.tilemap_load.width_words,
                    plan.tilemap_load.height_words,
                    plan.tilemap_load.left_padding_words,
                )
            })
            .collect::<Vec<_>>();

        assert_eq!(
            actual, expectations,
            "ASM sub_1C034 should keep the `off_1C126` asset, size, and left-padding table intact",
        );
    }

    #[test]
    fn intro_sequence_5_frame_load_plan_sub_1c034_keeps_the_fixed_25_row_vdp_stream_shape() {
        let plan = intro_sequence_5_frame_load_plan_sub_1C034(4)
            .expect("ASM sub_1C034 should accept the centred 15-word frame");

        assert_eq!(plan.tilemap_load.vram_word_address, 0xA340);
        assert_eq!(plan.vram_rows.len(), 25);
        assert_eq!(
            plan.vram_rows[0],
            VramRowWriteSub1C034 {
                vram_word_address: 0xA340,
                source_word_offset: 0,
                word_count: 15,
            },
            "ASM loc_1C09E starts the row stream at VRAM `$A340` and skips the 16th staged word on the top rows",
        );
        assert_eq!(
            plan.vram_rows[8],
            VramRowWriteSub1C034 {
                vram_word_address: 0xA540,
                source_word_offset: 128,
                word_count: 16,
            },
            "ASM loc_1C0B8 widens the middle band to all 16 staged words",
        );
        assert_eq!(
            plan.vram_rows[24],
            VramRowWriteSub1C034 {
                vram_word_address: 0xA940,
                source_word_offset: 384,
                word_count: 15,
            },
            "ASM loc_1C0D0 returns to 15-word rows for the tail of the upload",
        );
    }

    #[test]
    fn intro_sequence_5_panel_load_plan_sub_1c0ee_reuses_the_shared_10x10_sub_1c5d0_blit() {
        let plan = intro_sequence_5_panel_load_plan_sub_1C0EE(3, 0x13);

        assert_eq!(
            plan.tilemap_load,
            SymbolicEnigmaTilemapLoadSub1C0EE {
                vram_word_address: 0xA340,
                asset_label: "unk_2FDA6",
                width_words: 10,
                height_words: 10,
            },
            "ASM sub_1C0EE should decode `unk_2FDA6` with base tile `$A340` before reusing loc_1C104",
        );
        assert_eq!(
            plan.plane_upload,
            PlaneRectUploadSub1C0EE {
                plane_cell_x: 3,
                plane_cell_y: 0x13,
                width_words: 10,
                height_words: 10,
            },
            "ASM loc_1C104 should hand the decoded panel to sub_1C5D0 as a 10x10 blit at the caller's plane cell",
        );
    }

    #[test]
    fn intro_sequence_5_blank_panel_load_plan_sub_1c112_clears_5a_words_then_reuses_loc_1c104() {
        let plan = intro_sequence_5_blank_panel_load_plan_sub_1C112(4, 0x12);

        assert_eq!(
            plan.cleared_word_count, 0x005A,
            "ASM sub_1C112 should clear exactly `$5A` words in `Decompression_Buffer` before branching to loc_1C104",
        );
        assert_eq!(
            plan.plane_upload,
            PlaneRectUploadSub1C0EE {
                plane_cell_x: 4,
                plane_cell_y: 0x12,
                width_words: 10,
                height_words: 10,
            },
            "ASM sub_1C112 should reuse loc_1C104's shared 10x10 sub_1C5D0 blit at the caller's plane cell",
        );
    }

    #[test]
    fn level_layout_xor_vram_upload_sub_1c180_xors_2c0_big_endian_longwords() {
        let level_layout = (0..(704 * 4))
            .map(|value| u8::try_from(value & 0xFF).expect("fixture byte should fit in u8"))
            .collect::<Vec<_>>();
        let shadow_layout = (0..(704 * 4))
            .map(|value| {
                let masked = (value * 3) & 0xFF;
                u8::try_from(masked).expect("fixture byte should fit in u8")
            })
            .collect::<Vec<_>>();

        let upload = level_layout_xor_vram_upload_sub_1C180(&level_layout, &shadow_layout)
            .expect("ASM sub_1C180 should accept the full `$2C0`-longword layout span");

        assert_eq!(upload.vram_word_address, 0x1780);
        assert_eq!(
            upload.longwords.len(),
            704,
            "ASM sub_1C180 should emit one XORed longword for each of the `$2C0` streamed writes",
        );
        assert_eq!(
            upload.longwords[0], 0x0002_040A,
            "ASM sub_1C180 should treat both RAM sources as big-endian longwords before XORing them",
        );
        assert_eq!(upload.longwords[1], 0x080A_1412);
        assert_eq!(upload.longwords[703], 0x080A_0402);
    }

    #[test]
    fn level_layout_xor_vram_upload_sub_1c180_rejects_short_layout_inputs() {
        let error = level_layout_xor_vram_upload_sub_1C180(&vec![0; 0x0AFF], &vec![0; 0x0B00])
            .expect_err("ASM sub_1C180 should require the full `$2C0`-longword Level_Layout span");

        assert!(
            error
                .to_string()
                .contains("sub_1C180 needs 0x0B00 bytes of Level_Layout"),
            "the error should describe the required ROM-sized Level_Layout byte span",
        );
    }

    #[test]
    fn mask_level_layout_column_sub_1c1a2_keeps_the_low_nibble_for_selector_minus_1() {
        let mut level_layout = vec![0xAB; 16 * 0x20];

        mask_level_layout_column_sub_1C1A2(&mut level_layout, -1);

        for row in 0..16usize {
            assert_eq!(
                level_layout[row * 0x20],
                0x0B,
                "ASM sub_1C1A2/sub_1C1D2 should keep only the low nibble when selector -1 maps to the first column",
            );
        }
        assert_eq!(
            level_layout[1], 0xAB,
            "sub_1C1D2 should only touch the selected byte in each `$20`-byte row",
        );
    }

    #[test]
    fn mask_level_layout_column_sub_1c1a2_keeps_the_high_nibble_for_selector_15() {
        let mut level_layout = vec![0xAB; 16 * 0x20];

        mask_level_layout_column_sub_1C1A2(&mut level_layout, 15);

        for row in 0..16usize {
            assert_eq!(
                level_layout[4 + row * 0x20],
                0xA0,
                "ASM sub_1C1A2 should follow `byte_1C1F4` entry `$09`, and sub_1C1D2 should keep only the high nibble when the encoded offset is odd",
            );
        }
        assert_eq!(level_layout[5], 0xAB);
    }

    #[test]
    fn mask_level_layout_column_sub_1c1a2_ignores_selectors_outside_the_signed_rom_range() {
        let baseline = vec![0xAB; 16 * 0x20];
        let mut negative = baseline.clone();
        let mut too_large = baseline.clone();

        mask_level_layout_column_sub_1C1A2(&mut negative, -2);
        mask_level_layout_column_sub_1C1A2(&mut too_large, 63);

        assert_eq!(
            negative, baseline,
            "ASM sub_1C1A2 should return immediately when the pre-increment selector stays negative",
        );
        assert_eq!(
            too_large, baseline,
            "ASM sub_1C1A2 should return immediately when the incremented selector reaches `$40`",
        );
    }

    #[test]
    fn thunder_animation_frame_upload_sub_1b41c_reads_32_longwords_from_the_selected_frame() {
        let art = (0u16..256)
            .map(|value| u8::try_from(value).expect("fixture byte should fit in u8"))
            .collect::<Vec<_>>();

        let upload = thunder_animation_frame_upload_sub_1B41C(1, &art).expect("frame 1 should fit");

        assert_eq!(upload.vram_word_address, 0x17A0);
        assert_eq!(
            upload.longwords[0], 0x8081_8283,
            "ASM sub_1B41C should advance by $80 bytes per frame before the first move.l",
        );
        assert_eq!(upload.longwords[1], 0x8485_8687);
        assert_eq!(upload.longwords[31], 0xFCFD_FEFF);
    }

    #[test]
    fn thunder_animation_frame_upload_sub_1b41c_rejects_truncated_art() {
        let error = thunder_animation_frame_upload_sub_1B41C(2, &[0; 128])
            .expect_err("frame 2 should require a third $80-byte frame");

        assert!(
            error
                .to_string()
                .contains("sub_1B41C frame 2 needs bytes [0x0100..0x0180)"),
            "the error should describe the exact ASM byte window that was out of range",
        );
    }

    #[test]
    fn sub_1b532_uses_the_title_clear_and_tilemap_vram_windows() {
        let decoded_words = (0u16..(28 * 65) as u16).collect::<Vec<_>>();

        let upload = title_text_chameleon_tilemap_upload_sub_1B532(&decoded_words)
            .expect("ASM sub_1B532 should accept 28 source rows with the skipped trailer word");

        assert_eq!(upload.clear_vram_word_address, 0x1780);
        assert_eq!(
            upload.clear_longword_count, 8,
            "ASM sub_1B532 starts by clearing eight longwords at VRAM $1780"
        );
        assert_eq!(upload.tilemap_vram_word_address, 0xE000);
    }

    #[test]
    fn sub_1b532_skips_one_decoded_word_between_each_uploaded_row() {
        let decoded_words = (0u16..(28 * 65) as u16).collect::<Vec<_>>();

        let upload = title_text_chameleon_tilemap_upload_sub_1B532(&decoded_words)
            .expect("ASM sub_1B532 should map the decoded tilemap into 28 uploaded rows");

        assert_eq!(upload.rows[0][0], 0);
        assert_eq!(upload.rows[0][63], 63);
        assert_eq!(
            upload.rows[1][0], 65,
            "ASM loc_1B566/loc_1B568 skips decoded word 64 before starting row 1"
        );
        assert_eq!(upload.rows[1][63], 128);
        assert_eq!(upload.rows[27][0], 27 * 65);
        assert_eq!(upload.rows[27][63], 27 * 65 + 63);
    }

    #[test]
    fn sub_1b532_rejects_short_decoded_tilemaps() {
        let decoded_words = vec![0_u16; 28 * 65 - 1];

        let error = title_text_chameleon_tilemap_upload_sub_1B532(&decoded_words)
            .expect_err("ASM sub_1B532 needs the skipped trailer word for every source row");

        assert!(
            error
                .to_string()
                .contains("needs at least 1820 decoded words"),
            "short tilemaps should fail with the missing-row-stride detail: {error}"
        );
    }

    #[test]
    fn export_split_bundle_resolves_logical_level_resources_through_map_order() {
        let source = TempDir::new().expect("temp dir");
        let output = TempDir::new().expect("temp dir");
        let source_root = source.path();
        write_default_test_bundle_source(source_root);
        fs::write(
            source_root.join("doc/level_list.txt"),
            "0x00 0x4A Blue Lake Woods 1\n0x4A 0x5E Elsewhere 2\n",
        )
        .expect("level names");
        fs::write(
            source_root.join("level/level_files.txt"),
            "00 platform/00.asm bgscroll/00.bin header/00.bin enemy/00.bin foreground/00.bin block/00.bin background/00.bin\n4A platform/4A.asm bgscroll/4A.bin header/4A.bin enemy/4A.bin foreground/4A.bin block/4A.bin background/4A.bin\n",
        )
        .expect("level files");
        fs::write(source_root.join("level/maporder.asm"), "\tdc.b\t$4A\t; 0\n").expect("map order");
        fs::write(
            source_root.join("level/platform/4A.asm"),
            b"\tptfm\t0,0,0,0,0,0,0,0,0,0,PlatformScript_Nothing-PlatformScript_BaseAddress\n\tdc.w\t$FFFF\n",
        )
        .expect("platform");
        fs::write(
            source_root.join("level/bgscroll/4A.bin"),
            [0x30, 0x50, 0xFF],
        )
        .expect("bgscroll");
        fs::write(
            source_root.join("level/header/4A.bin"),
            [
                0x0A, 0x02, 0x09, 0x09, 0x00, 0x20, 0x01, 0x00, 0x0C, 0x50, 0x00, 0xB0,
            ],
        )
        .expect("header");
        fs::write(
            source_root.join("level/enemy/4A.bin"),
            [
                0x00, 0x00, 0x00, 0x00, 0xFF, 0xFF, 0x00, 0x0C, 0x00, 0x00, 0x7D, 0x00, 0x80, 0x01,
                0x0D, 0x00, 0x00, 0x01, 0x00, 0x60, 0x00, 0x00,
            ],
        )
        .expect("enemy");
        fs::write(source_root.join("level/foreground/4A.bin"), SAXMAN_FIXTURE).expect("foreground");
        fs::write(
            source_root.join("level/block/4A.bin"),
            REAL_STYLE_BLOCK_FIXTURE,
        )
        .expect("block");
        fs::write(
            source_root.join("level/background/4A.bin"),
            REAL_STYLE_BACKGROUND_FIXTURE,
        )
        .expect("background");

        let manifest = export_split_bundle(source_root, output.path(), "abc123", "deadbeef")
            .expect("bundle export should succeed");
        assert_eq!(resource_level_id(&manifest, 0x00), 0x4A);
        let level = manifest
            .levels
            .iter()
            .find(|level| level.level_id == 0x00)
            .expect("logical level 00");
        assert_eq!(level.resources.header.as_deref(), Some("header/00.bin"));
        assert_eq!(
            level
                .header_metadata
                .as_ref()
                .map(|header| header.level_theme),
            Some(0x08)
        );

        let decoded_level0 = manifest
            .decoded_levels
            .iter()
            .find(|d| d.level_id == 0x00)
            .expect("decoded level 0x00");
        assert_eq!(
            fs::read(output.path().join(&decoded_level0.foreground.relative_path))
                .expect("decoded foreground"),
            b"ABCD"
        );

        let level_bundle =
            load_level_asset_bundle(&manifest, 0x00).expect("level bundle should load");
        assert_eq!(
            level_bundle.display_name.as_deref(),
            Some("Blue Lake Woods 1")
        );
        assert_eq!(
            level_bundle.title_card.as_ref(),
            Some(&LevelTitleCard {
                script_bytes: b"blue\0lake\0woods\xFF".to_vec(),
                line_destinations: vec![
                    TitleCardDestination { x: 0x20, y: 0x28 },
                    TitleCardDestination { x: 0x20, y: 0x48 },
                    TitleCardDestination { x: 0x20, y: 0x68 },
                ],
                level_number_destination: Some(TitleCardDestination { x: 0xDC, y: 0x50 }),
                level_number_glyph_count: 1,
            })
        );
        assert_eq!(
            level_bundle
                .parsed_header
                .as_ref()
                .map(|header| (header.level_theme, header.background_theme)),
            Some((0x09, 0x09))
        );
        assert_eq!(
            level_bundle
                .parsed_header
                .as_ref()
                .map(|header| (header.player_start_x, header.flag_x)),
            Some((0x0020, 0x0C50))
        );
        assert_eq!(
            level_bundle
                .parsed_enemy_layout
                .as_ref()
                .map(|layout| layout.enemy_type_slots.clone()),
            Some(vec![0xFFFF, 0x000C, 0x0000])
        );
        assert_eq!(
            level_bundle.parsed_enemy_layout.as_ref().map(|layout| {
                layout
                    .objects
                    .iter()
                    .map(|object| object.object_type)
                    .collect::<Vec<_>>()
            }),
            Some(vec![0x0D])
        );
    }

    #[test]
    fn export_split_bundle_preserves_level_layout_sidecars() {
        let source = TempDir::new().expect("temp dir");
        write_default_test_bundle_source(source.path());
        let output_root = TempDir::new().expect("temp dir");
        let bundle_root = output_root.path().join("bundle");

        export_split_bundle(source.path(), &bundle_root, "abc123", "deadbeef")
            .expect("initial export should succeed");
        fs::create_dir_all(bundle_root.join("level-layout")).expect("level-layout dir");
        fs::write(bundle_root.join("level-layout/00.bin"), [0x70, 0x33]).expect("layout");
        fs::write(
            bundle_root.join("level-layout/00.json"),
            br#"{"data_bytes":2,"width_blocks":1,"height_blocks":1}"#,
        )
        .expect("layout metadata");

        export_split_bundle(source.path(), &bundle_root, "abc123", "deadbeef")
            .expect("second export should preserve sidecars");

        assert_eq!(
            fs::read(bundle_root.join("level-layout/00.bin")).expect("preserved layout"),
            vec![0x70, 0x33]
        );
        assert_eq!(
            fs::read_to_string(bundle_root.join("level-layout/00.json"))
                .expect("preserved metadata"),
            r#"{"data_bytes":2,"width_blocks":1,"height_blocks":1}"#
        );
    }

    #[test]
    fn export_split_bundle_keeps_existing_bundle_when_regeneration_fails() {
        let initial_source = TempDir::new().expect("temp dir");
        write_default_test_bundle_source(initial_source.path());
        let output_root = TempDir::new().expect("temp dir");
        let output_path = output_root.path().join("bundle");

        let original_manifest =
            export_split_bundle(initial_source.path(), &output_path, "abc123", "deadbeef")
                .expect("initial export should succeed");
        let original_manifest_path = manifest_path(&output_path);
        let original_manifest_bytes =
            fs::read(&original_manifest_path).expect("original manifest should exist");

        let failing_source = TempDir::new().expect("temp dir");
        write_test_bundle_source(
            failing_source.path(),
            &[0x00, 0x00],
            REAL_STYLE_BLOCK_FIXTURE,
            ENIGMA_FIXTURE,
        );

        let error = export_split_bundle(failing_source.path(), &output_path, "abc123", "deadbeef")
            .expect_err("regeneration should fail");
        assert!(error.to_string().contains("Saxman"));

        let retained_manifest_bytes =
            fs::read(&original_manifest_path).expect("original manifest should still exist");
        assert_eq!(retained_manifest_bytes, original_manifest_bytes);
        let retained_manifest = load_manifest(&original_manifest_path).expect("manifest reload");
        assert_eq!(retained_manifest, original_manifest);
        assert!(
            output_path
                .join(
                    retained_manifest
                        .decoded_levels
                        .iter()
                        .find(|d| d.level_id == 0x00)
                        .expect("decoded metadata")
                        .foreground
                        .relative_path
                        .as_str()
                )
                .is_file()
        );
    }

    #[test]
    fn parse_platform_scripts_extracts_commands() {
        let asm = "\
PlatformScript_Nothing\t= $10000

PlatformScript_BaseAddress:
PlatformScript_3602:   ; relative offset: 0000
\tptfm_move   $24B, $18000, $0
\tptfm_move   $24B, -$18000, $0
\tdc.w    $FFFF
\tdc.l    PlatformScript_3602
PlatformScript_361C:   ; relative offset: 001A
\tptfm_move   $E1, $0, -$11000
\tptfm_move   $E1, $0, $11000
\tdc.w    $FFFF
\tdc.l    PlatformScript_361C
";
        let table = parse_platform_scripts(asm);
        assert_eq!(table.scripts.len(), 2);

        let script_0 = &table.scripts[&0x0000];
        assert_eq!(script_0.commands.len(), 2);
        assert_eq!(script_0.commands[0].duration, 0x24B);
        assert_eq!(script_0.commands[0].x_velocity, 0x18000);
        assert_eq!(script_0.commands[0].y_velocity, 0);
        assert_eq!(script_0.commands[1].x_velocity, -0x18000);

        let script_1a = &table.scripts[&0x001A];
        assert_eq!(script_1a.commands.len(), 2);
        assert_eq!(script_1a.commands[0].duration, 0xE1);
        assert_eq!(script_1a.commands[0].x_velocity, 0);
        assert_eq!(script_1a.commands[0].y_velocity, -0x11000);
    }

    fn synthetic_mappings_sub_30194(base: u16) -> Vec<u16> {
        let mut mappings = vec![0u16; 0x100 * 4];
        for tile in 0..0x100u16 {
            let offset = usize::from(tile) * 4;
            mappings[offset] = base.wrapping_add(tile.wrapping_mul(4));
            mappings[offset + 1] = base.wrapping_add(tile.wrapping_mul(4)).wrapping_add(1);
            mappings[offset + 2] = base.wrapping_add(tile.wrapping_mul(4)).wrapping_add(2);
            mappings[offset + 3] = base.wrapping_add(tile.wrapping_mul(4)).wrapping_add(3);
        }
        mappings
    }

    fn synthetic_row_offsets_sub_30194(row_pairs: usize, width_cells: usize) -> Vec<u16> {
        (0..row_pairs)
            .map(|row_pair| u16::try_from(row_pair * width_cells).expect("row offset fits in u16"))
            .collect()
    }

    #[test]
    fn sub_30194_builds_the_new_bottom_row_longword_upload_from_theme_and_block_mappings() {
        let theme_mappings = synthetic_mappings_sub_30194(0x1000);
        let block_mappings = synthetic_mappings_sub_30194(0x5000);
        let row_offsets = synthetic_row_offsets_sub_30194(32, 40);
        let mut level_layout_cells = vec![0u16; 32 * 40];
        let streamed_row_start = 14 * 40;
        level_layout_cells[streamed_row_start] = 0x0001;
        level_layout_cells[streamed_row_start + 1] = 0x0102;
        level_layout_cells[streamed_row_start + 2] = 0x8003;

        let plan = level_plane_scroll_update_plan_sub_30194(LevelPlaneScrollUpdateInputsSub30194 {
            camera_x_pos: 0,
            camera_y_pos: 8,
            previous_camera_x_pos: 0,
            previous_camera_y_pos: 0,
            level_layout_row_offsets_cells: &row_offsets,
            level_layout_cells: &level_layout_cells,
            level_width_tiles: 80,
            level_height_tiles: 64,
            theme_mappings: &theme_mappings,
            block_mappings: &block_mappings,
        })
        .expect("ASM sub_30194 should accept a valid row-pointer table and mapping tables");

        assert_eq!(
            plan.vertical_uploads,
            Vec::<PlaneWordStripUploadSub30194>::new()
        );
        assert_eq!(plan.horizontal_uploads.len(), 1);
        assert_eq!(plan.horizontal_uploads[0].vram_word_address, 0x0740);
        assert_eq!(
            &plan.horizontal_uploads[0].longwords[..3],
            &[0x1006_1007, 0x900A_900B, 0x500E_500F],
            "ASM sub_30194/sub_3034A should select the streamed bottom row, set bit 15 for priority-marked theme cells, and switch to Block_Mappings for negative cells",
        );
        assert_eq!(
            plan.horizontal_uploads[0].longwords.len(),
            21,
            "ASM sub_30194 should emit 21 longwords for the newly exposed horizontal strip when the plane row does not wrap",
        );
    }

    #[test]
    fn sub_30194_splits_vertical_strip_uploads_when_the_fg_plane_wraps() {
        let theme_mappings = synthetic_mappings_sub_30194(0x1000);
        let block_mappings = synthetic_mappings_sub_30194(0x5000);
        let row_offsets = synthetic_row_offsets_sub_30194(20, 40);
        let mut level_layout_cells = vec![0u16; 20 * 40];
        let source_column = 32;
        let source_row_start = 2 * 40;
        level_layout_cells[source_row_start + source_column] = 0x0004;
        level_layout_cells[source_row_start + 40 + source_column] = 0x0105;
        level_layout_cells[source_row_start + 14 * 40 + source_column] = 0x8006;

        let plan = level_plane_scroll_update_plan_sub_30194(LevelPlaneScrollUpdateInputsSub30194 {
            camera_x_pos: 24 * 8,
            camera_y_pos: 32,
            previous_camera_x_pos: 23 * 8,
            previous_camera_y_pos: 32,
            level_layout_row_offsets_cells: &row_offsets,
            level_layout_cells: &level_layout_cells,
            level_width_tiles: 80,
            level_height_tiles: 64,
            theme_mappings: &theme_mappings,
            block_mappings: &block_mappings,
        })
        .expect("ASM sub_30194 should split wrapped vertical uploads across the FG plane");

        assert_eq!(
            plan.horizontal_uploads,
            Vec::<PlaneLongwordStripUploadSub30194>::new()
        );
        assert_eq!(plan.vertical_uploads.len(), 2);
        assert_eq!(plan.vertical_uploads[0].vram_word_address, 0x0100);
        assert_eq!(
            &plan.vertical_uploads[0].words[..4],
            &[0x1010, 0x1012, 0x9014, 0x9016],
            "ASM sub_30194/sub_302F6 should stream the left metatile column as two words per row and set bit 15 for priority cells",
        );
        assert_eq!(plan.vertical_uploads[0].words.len(), 28);
        assert_eq!(plan.vertical_uploads[1].vram_word_address, 0x0000);
        assert_eq!(
            plan.vertical_uploads[1].words,
            vec![0x5018, 0x501A],
            "ASM sub_30194 should wrap the fifteenth row write back to the top of the plane while continuing to read the next source row from Block_Mappings",
        );
    }

    #[allow(non_snake_case)]
    fn be_bytes_from_longwords_sub_303BA(longwords: &[u32]) -> Vec<u8> {
        longwords
            .iter()
            .flat_map(|longword| longword.to_be_bytes())
            .collect()
    }

    #[test]
    fn background_row_upload_sub_303ba_streams_the_forward_row_from_layer_0_then_plus_700() {
        let mut map_header_background_rows = vec![0u8; 0x30];
        map_header_background_rows[0x001E] = 5;

        let mut background_layer_0 = vec![0u8; 0x0900];
        let upper_pattern = (0..16)
            .map(|index| 0x1000_0000_u32 + index)
            .collect::<Vec<_>>();
        let lower_pattern = (0..16)
            .map(|index| 0x2000_0000_u32 + index)
            .collect::<Vec<_>>();
        background_layer_0[5 * 64..5 * 64 + 64]
            .copy_from_slice(&be_bytes_from_longwords_sub_303BA(&upper_pattern));
        background_layer_0[0x0700 + 5 * 64..0x0700 + 5 * 64 + 64]
            .copy_from_slice(&be_bytes_from_longwords_sub_303BA(&lower_pattern));

        let upload = background_row_upload_sub_303BA(
            0x0020,
            0,
            &map_header_background_rows,
            false,
            &background_layer_0,
            &[],
        )
        .expect("ASM sub_303BA should accept a valid non-Desert row selector and layer-0 buffer")
        .expect("ASM sub_303BA should emit an upload when the camera row changes");

        assert_eq!(upload.vram_word_address, 0x6E80);
        assert_eq!(
            upload.upper_longwords, upper_pattern,
            "ASM sub_303BA should read the first 16 longwords from `$FFFF87B2 + row*64` for selectors `<#$1C`",
        );
        assert_eq!(
            upload.lower_longwords, lower_pattern,
            "ASM sub_303BA should read the second 16 longwords from `$FFFF87B2 + $700 + row*64` for selectors `<#$1C`",
        );
    }

    #[test]
    fn background_row_upload_sub_303ba_uses_the_desert_bias_and_layer_1_for_high_rows() {
        let mut map_header_background_rows = vec![0u8; 4];
        map_header_background_rows[2] = 0x1C;

        let mut background_layer_1 = vec![0u8; 0x0800];
        let layer_1_pattern = (0..16)
            .map(|index| 0x3000_0000_u32 + index)
            .collect::<Vec<_>>();
        background_layer_1[0x1C * 64..0x1C * 64 + 64]
            .copy_from_slice(&be_bytes_from_longwords_sub_303BA(&layer_1_pattern));

        let upload = background_row_upload_sub_303BA(
            0,
            1,
            &map_header_background_rows,
            true,
            &[],
            &background_layer_1,
        )
        .expect("ASM sub_303BA should accept a valid Desert row selector and layer-1 buffer")
        .expect("ASM sub_303BA should emit an upload when the camera row changes");

        assert_eq!(upload.vram_word_address, 0x6000);
        assert_eq!(
            upload.upper_longwords, layer_1_pattern,
            "ASM sub_303BA should switch Desert themes to `2(a0,d0.w)` and selectors `>=#$1C` to `$FFFF8EB2 + row*64`",
        );
        assert_eq!(upload.lower_longwords, layer_1_pattern);
    }

    #[test]
    fn background_scroll_update_sub_304b4_uses_the_visible_left_edge_when_scrolling_left() {
        let update = background_scroll_update_sub_304B4(16, 40, true)
            .expect("ASM sub_304B4 should accept a valid leftward scroll update");

        assert_eq!(
            update,
            BackgroundScrollUpdateSub304B4 {
                background_buffer_column: 2,
                vram_word_offset: 4,
            },
            "ASM sub_304B4 should use `current>>3` directly when the previous tile column was greater than the new one",
        );
    }

    #[test]
    fn background_scroll_update_sub_304b4_adds_the_40_cell_viewport_when_scrolling_right() {
        let update = background_scroll_update_sub_304B4(16, 40, false)
            .expect("ASM sub_304B4 should accept a valid rightward scroll update");

        assert_eq!(
            update,
            BackgroundScrollUpdateSub304B4 {
                background_buffer_column: 2,
                vram_word_offset: 84,
            },
            "ASM sub_304B4 should add `#$28` before wrapping so the upload targets the newly exposed far column while keeping the pre-wrap VDP offset",
        );
    }

    #[test]
    fn background_tile_uploads_sub_30744_stops_on_ff_after_the_first_visible_row() {
        let mut background_chunk_buffer = vec![0u8; 16];
        background_chunk_buffer[3] = 5;

        let uploads = background_tile_uploads_sub_30744(
            0,
            10,
            &[0],
            &[0, 0xFF],
            &[BackgroundScrollStateSub30744 {
                background_buffer_column: Some(3),
                vram_word_offset: 0x0020,
            }],
            &background_chunk_buffer,
        )
        .expect("ASM sub_30744 should accept a valid first-row bgscroll entry plus terminator");

        assert_eq!(
            uploads,
            vec![BackgroundTileUploadSub30744 {
                vram_word_address: 0x6020,
                tile_word: 0x2785,
                background_chunk_id: 5,
            }],
            "ASM sub_30744 should stop at `$FF` and map the selected chunk id through `unk_309B8`",
        );
    }

    #[test]
    fn background_tile_uploads_sub_30744_advances_rows_across_skipped_scroll_entries() {
        let mut background_chunk_buffer = vec![0u8; 80];
        background_chunk_buffer[64] = 9;

        let uploads = background_tile_uploads_sub_30744(
            0,
            60,
            &[0],
            &[8, 0, 0xFF],
            &[
                BackgroundScrollStateSub30744 {
                    background_buffer_column: Some(4),
                    vram_word_offset: 0x0006,
                },
                BackgroundScrollStateSub30744 {
                    background_buffer_column: None,
                    vram_word_offset: 0x0010,
                },
            ],
            &background_chunk_buffer,
        )
        .expect(
            "ASM sub_30744 should skip hidden rows but keep advancing the background source row",
        );

        assert_eq!(
            uploads,
            vec![BackgroundTileUploadSub30744 {
                vram_word_address: 0x6086,
                tile_word: 0x2789,
                background_chunk_id: 9,
            }],
            "ASM sub_30744 should treat negative `Horiz_Scroll_Data` entries as skips, then upload the next visible row from the advanced background row base",
        );
    }

    #[test]
    fn background_row_uploads_sub_307d8_uses_the_newly_exposed_downward_row_chunk() {
        let mut background_chunk_buffer = vec![0u8; 32 * 40];
        for (index, byte) in background_chunk_buffer[31 * 40..32 * 40]
            .iter_mut()
            .enumerate()
        {
            *byte = u8::try_from(index).expect("row payload should fit into u8");
        }

        let uploads = background_row_uploads_sub_307D8(
            96,
            2,
            40,
            &(0..32)
                .map(|row| u16::try_from(row * 40).unwrap_or(0))
                .collect::<Vec<_>>(),
            &[0; 32],
            &[BackgroundScrollStateSub30744 {
                background_buffer_column: Some(0),
                vram_word_offset: 0x0000,
            }],
            &background_chunk_buffer,
        )
        .expect("ASM sub_307D8 should stream `current+#$1C` when the camera row increases");

        assert_eq!(
            uploads,
            vec![BackgroundRowUploadSegmentSub307D8 {
                vram_word_address: 0x6F80,
                tile_words: (0x2780..=0x27A7).collect(),
                background_chunk_ids: (0u8..=39).collect(),
            }],
            "ASM sub_307D8 should pick row chunk 31 for the 3->31 downward reveal and upload the full 40-cell strip without wrapping",
        );
    }

    #[test]
    fn background_row_uploads_sub_307d8_splits_when_source_and_vdp_rows_wrap() {
        let mut background_chunk_buffer = vec![0u8; 10];
        background_chunk_buffer[..10].copy_from_slice(&[1, 2, 3, 4, 5, 6, 7, 8, 9, 10]);

        let uploads = background_row_uploads_sub_307D8(
            0,
            1,
            10,
            &[0],
            &[0],
            &[BackgroundScrollStateSub30744 {
                background_buffer_column: Some(8),
                vram_word_offset: 0x0078,
            }],
            &background_chunk_buffer,
        )
        .expect(
            "ASM sub_307D8 should split the streamed strip whenever the source row or plane row wraps",
        );

        assert_eq!(
            uploads,
            vec![
                BackgroundRowUploadSegmentSub307D8 {
                    vram_word_address: 0x6078,
                    tile_words: vec![0x2789, 0x278A],
                    background_chunk_ids: vec![9, 10],
                },
                BackgroundRowUploadSegmentSub307D8 {
                    vram_word_address: 0x607C,
                    tile_words: vec![0x2781, 0x2782],
                    background_chunk_ids: vec![1, 2],
                },
                BackgroundRowUploadSegmentSub307D8 {
                    vram_word_address: 0x6000,
                    tile_words: vec![
                        0x2783, 0x2784, 0x2785, 0x2786, 0x2787, 0x2788, 0x2789, 0x278A
                    ],
                    background_chunk_ids: vec![3, 4, 5, 6, 7, 8, 9, 10],
                },
                BackgroundRowUploadSegmentSub307D8 {
                    vram_word_address: 0x6010,
                    tile_words: vec![
                        0x2781, 0x2782, 0x2783, 0x2784, 0x2785, 0x2786, 0x2787, 0x2788, 0x2789,
                        0x278A,
                    ],
                    background_chunk_ids: vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10],
                },
                BackgroundRowUploadSegmentSub307D8 {
                    vram_word_address: 0x6024,
                    tile_words: vec![
                        0x2781, 0x2782, 0x2783, 0x2784, 0x2785, 0x2786, 0x2787, 0x2788, 0x2789,
                        0x278A,
                    ],
                    background_chunk_ids: vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10],
                },
                BackgroundRowUploadSegmentSub307D8 {
                    vram_word_address: 0x6038,
                    tile_words: vec![
                        0x2781, 0x2782, 0x2783, 0x2784, 0x2785, 0x2786, 0x2787, 0x2788
                    ],
                    background_chunk_ids: vec![1, 2, 3, 4, 5, 6, 7, 8],
                },
            ],
            "ASM sub_307D8 should keep reissuing row segments whenever the 10-cell source wraps, even after the plane row has already wrapped back to $6000",
        );
    }
}
