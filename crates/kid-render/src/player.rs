use crate::{Palette, TileSheet};
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SpriteFrame {
    pub origin_x: i16,
    pub origin_y: i16,
    pub width_px: u16,
    pub height_px: u16,
    pub tile_base_index: u16,
    pub tile_sheet: TileSheet,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AnimatedSpriteVisual {
    pub palette: Palette,
    /// Additional palette variants (indexed by palette_variant - 1).
    pub extra_palettes: Vec<Palette>,
    pub frames: Vec<SpriteFrame>,
    pub frame_duration: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlayerTransformStep {
    pub duration_frames: usize,
    pub frame: Option<SpriteFrame>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlayerFormVisuals {
    pub palette: Palette,
    pub idle: SpriteFrame,
    pub walk: Vec<SpriteFrame>,
    /// ASM stru_8B6A: pre-drawn tilted sprite frames for slope walking.
    /// Right-facing set (kid_slope_7..12). Empty if not loaded.
    pub slope_walk: Vec<SpriteFrame>,
    /// Left-facing slope walk frames (kid_slope_1..6). Empty if not loaded.
    pub slope_walk_left: Vec<SpriteFrame>,
    pub jump_rise: SpriteFrame,
    pub jump_fall: SpriteFrame,
    pub transform_palette: Option<Palette>,
    pub transform_steps: Vec<PlayerTransformStep>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlayerVisuals {
    pub forms: HashMap<u16, PlayerFormVisuals>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FlagVisuals {
    pub palette: Palette,
    pub pole_bottom: SpriteFrame,
    pub waving: Vec<SpriteFrame>,
    pub frame_duration: usize,
}

/// Enemy sprite visuals keyed by object_type (0x00..0x22).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EnemyVisuals {
    pub families: std::collections::HashMap<u8, AnimatedSpriteVisual>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FloatingPrizeVisuals {
    pub diamond: AnimatedSpriteVisual,
    /// Helmet prize visuals keyed by prize content id (0x02..0x0A).
    pub helmets: HashMap<u8, AnimatedSpriteVisual>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlatformVisuals {
    pub palette: Palette,
    pub horizontal: SpriteFrame,
    pub vertical: SpriteFrame,
}
