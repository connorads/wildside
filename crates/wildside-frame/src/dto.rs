//! Draw-command DTOs, diagnostics, and rasteriser constants.

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
