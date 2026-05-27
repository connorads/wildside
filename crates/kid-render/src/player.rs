use crate::{Palette, TileRef, TileSheet};
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
    /// ASM `off_9688`: Berzerker head-forward charge/run frames.
    /// Empty for forms without a separate charge table.
    pub charge: Vec<SpriteFrame>,
    /// ASM stru_8B6A: pre-drawn tilted sprite frames for slope walking.
    /// Right-facing set (kid_slope_7..12). Empty if not loaded.
    pub slope_walk: Vec<SpriteFrame>,
    /// Left-facing slope walk frames (kid_slope_1..6). Empty if not loaded.
    pub slope_walk_left: Vec<SpriteFrame>,
    pub jump_rise: SpriteFrame,
    pub jump_fall: SpriteFrame,
    /// ASM `Death` (kid.asm:14749) installs `unk_A3FF8` / `kid_death.bin`
    /// for the visible tumble before `lose_life`.
    pub death: Option<SpriteFrame>,
    /// ASM `stru_8BB4`: The Kid wall-grab pull-up / rollover animation.
    pub pull_up: Vec<SpriteFrame>,
    /// ASM `stru_8B3C` / `unk_B3520..unk_B3CB2`: Red Stealth grounded sword slash.
    pub slash_ground: Vec<SpriteFrame>,
    /// ASM `sub_B270` / `unk_B40DE..unk_B4870`: Red Stealth airborne downward slash.
    pub slash_air: Vec<SpriteFrame>,
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MurderWallVisuals {
    pub palette: Palette,
    pub tile_sheet: TileSheet,
    /// ASM Murderwall loc_140D2 uploads `Murder_wall.bin` to VRAM byte
    /// offset `$5F60`, which is tile `$2FB`. Sprite words from `sub_5E02`
    /// are absolute VRAM tile indices, so native subtracts this before
    /// indexing the extracted sheet.
    pub vram_base_tile_index: u16,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LavaVisuals {
    pub palette: Palette,
    pub tile_sheet: TileSheet,
    /// ASM Init_SpecialEffect_Lava loc_14C6A uploads `Lava.bin` to the same
    /// `$5F60` VRAM destination used by murder wall, i.e. tile `$2FB`.
    pub vram_base_tile_index: u16,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EolScoreVisuals {
    pub palette: Palette,
    pub bonus: SpriteFrame,
    pub continues: SpriteFrame,
    pub no_hit: SpriteFrame,
    pub no_prize: SpriteFrame,
    pub path: SpriteFrame,
    pub score: SpriteFrame,
    pub seconds: SpriteFrame,
    pub speed: SpriteFrame,
    pub time: SpriteFrame,
    pub under: SpriteFrame,
    pub digits: Vec<SpriteFrame>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EolHologramVisuals {
    pub palette: Palette,
    pub tile_sheet: TileSheet,
    pub rows: Vec<Vec<TileRef>>,
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
    /// Simple icon prize visuals keyed by prize content id (0x0B..0x0D).
    pub icons: HashMap<u8, AnimatedSpriteVisual>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlatformVisuals {
    pub palette: Palette,
    pub horizontal: SpriteFrame,
    pub vertical: SpriteFrame,
}
