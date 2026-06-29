//! Draw-command DTOs, diagnostics, and rasteriser constants.

use std::sync::Arc;

use serde::{Deserialize, Serialize};
use wildside_render::{Palette, TileRef, TileSheet};

/// Native framebuffer width in pixels. `render_vdp_into` and the rasteriser
/// hard-code 320×224, matching the Mega Drive H40 display.
pub const FRAME_WIDTH: u32 = 320;
/// Native framebuffer height in pixels.
pub const FRAME_HEIGHT: u32 = 224;

/// VRAM tile index where decoded foreground (Plane A) art begins.
pub const FOREGROUND_TILE_VRAM_BASE: u16 = 0x00BC;
/// VRAM tile index where decoded block art begins.
pub const BLOCK_TILE_VRAM_BASE: u16 = 0x0220;
/// VRAM tile index where the HUD glyph art begins.
pub const HUD_TILE_VRAM_BASE: u16 = 0x06BA;
/// VRAM tile index where the HUD diamond-counter art begins.
pub const HUD_DIAMOND_TILE_VRAM_BASE: u16 = 0x06EA;

/// Backdrop colour for the rasterised frame before any draw command runs.
pub const SKY_COLOUR: [u8; 3] = [28, 42, 56];
/// Degraded-asset fallback colour for an enemy rect (see [`enemy_family_colour`]).
pub const ENEMY_COLOUR: [u8; 3] = [220, 90, 88];
/// Flash colour overlaid on an enemy during its hit-flash window.
pub const ENEMY_FLASH_COLOUR: [u8; 3] = [255, 255, 255];
/// Mega Drive palette line size: 16 entries, of which index 0 is transparent,
/// leaving 15 visible colours per line.
pub const PALETTE_LINE_COLOUR_COUNT: usize = 15;

/// Degraded-asset fallback colour for an enemy rect when its ROM sprite
/// art is not loaded. The previous per-family hand-picked palette
/// (orange-red Fire Demon, steel-grey Robot, military-green Tank, etc.)
/// was a hand-invented "visually distinguishable" set with no ROM
/// citation — the original game's per-family colours come from each
/// family's `Pal_*` block (e.g. `Pal_A2382` for Heady Metal). Until
/// those palettes flow through `level_visuals.enemies.families`, render
/// a single consistent placeholder so missing-asset states do not claim
/// per-family hues that the ROM does not specify.
pub fn enemy_family_colour(_object_type: u8) -> [u8; 3] {
    ENEMY_COLOUR
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Scene {
    pub background: [u8; 3],
    pub commands: Vec<DrawCommand>,
    pub pause_palette_darken: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DrawCommand {
    Rect {
        x: i32,
        y: i32,
        width: i32,
        height: i32,
        colour: [u8; 3],
    },
    Tint {
        colour: [u8; 3],
        alpha: u8,
    },
    PaletteFadeSub73D0 {
        target_word: u16,
        fade_progress: u16,
    },
    ClearPriorityRect {
        x: i32,
        y: i32,
        width: i32,
        height: i32,
    },
    Text {
        x: i32,
        y: i32,
        text: String,
        colour: [u8; 3],
        scale: i32,
    },
    #[allow(dead_code)]
    IndexedTile {
        x: i32,
        y: i32,
        tile_sheet: Arc<TileSheet>,
        tile: TileRef,
        palette: Arc<Palette>,
    },
    #[allow(dead_code)]
    IndexedTileRow {
        x: i32,
        y: i32,
        tile_sheet: Arc<TileSheet>,
        tile: TileRef,
        palette: Arc<Palette>,
        row: u8,
    },
    #[allow(dead_code)]
    IndexedTileSlice {
        x: i32,
        y: i32,
        tile_sheet: Arc<TileSheet>,
        tile: TileRef,
        palette: Arc<Palette>,
        row: u8,
        start_column: u8,
        width: u8,
    },
    #[allow(dead_code)]
    Sprite {
        x: i32,
        y: i32,
        tile_sheet: Arc<TileSheet>,
        palette: Arc<Palette>,
        tiles: Vec<SpriteTile>,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SpriteTile {
    pub offset_x: i32,
    pub offset_y: i32,
    pub tile: TileRef,
}

#[derive(Debug, Clone)]
pub struct ResolvedSceneTile {
    pub tile_sheet: Arc<TileSheet>,
    pub tile: TileRef,
    pub palette: Arc<Palette>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NativeSpriteTileDiagnostic {
    pub offset_x: i32,
    pub offset_y: i32,
    pub tile_index: u16,
    pub palette_line: u8,
    pub priority: bool,
    pub flip_x: bool,
    pub flip_y: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NativeEolScoreHardwareSpriteDiagnostic {
    pub command_index: usize,
    pub chunk_index: usize,
    pub x: i32,
    pub y: i32,
    pub width: u16,
    pub height: u16,
    pub first_tile_index: u16,
    pub min_x: i32,
    pub min_y: i32,
    pub max_x: i32,
    pub max_y: i32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NativeEolScoreSpriteDiagnostic {
    pub command_index: usize,
    pub x: i32,
    pub y: i32,
    pub min_x: i32,
    pub min_y: i32,
    pub max_x: i32,
    pub max_y: i32,
    pub tile_count: usize,
    pub tiles: Vec<NativeSpriteTileDiagnostic>,
    #[serde(default)]
    pub hardware_sprites: Vec<NativeEolScoreHardwareSpriteDiagnostic>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FinalScenePixelDiagnostic {
    pub x: usize,
    pub y: usize,
    pub final_rgb: [u8; 3],
    pub final_high_priority: bool,
    pub writes: Vec<FinalScenePixelWrite>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FinalScenePixelWrite {
    pub command_index: usize,
    pub kind: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub origin_x: Option<i32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub origin_y: Option<i32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub width: Option<i32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub height: Option<i32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tile_index: Option<u16>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub palette_line: Option<u8>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub palette_index: Option<u8>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_x: Option<u8>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_y: Option<u8>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rgb: Option<[u8; 3]>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub priority: Option<bool>,
    pub prior_high_priority: bool,
    pub accepted: bool,
}
