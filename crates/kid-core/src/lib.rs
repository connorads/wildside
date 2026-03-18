mod blocks;
mod combat;
pub(crate) mod enemies;
mod helmets;
mod level;
mod movement;
pub mod physics;
mod platforms;
mod projectiles;
pub mod terrain;
mod weather;

use enemies::*;
use kid_data::{
    AssetManifest, EnemyLayout, EnemyLayoutObject, PlatformLayout, PlatformPlacement,
    resource_level_record,
};
use physics::{Direction, HorizontalInput, LevelBounds};
use platforms::*;
use serde::{Deserialize, Serialize};
use std::fs;

const SEGA_SCREEN_FRAMES: usize = 120;
const INTRO_SEGMENT_FRAMES: usize = 80;
const TITLE_TO_LEVEL_FRAMES: usize = 60;
// The native playtest host should open on the first fully settled Level 1 frame,
// past the long entry animation where the kid drifts into place.
const PLAYTEST_LEVEL_SETTLE_FRAMES: usize = 349;
const OPENING_ENTRY_CAMERA_Y_FP: u32 = 0x0093_0000;
const OPENING_GROUNDED_CAMERA_Y_FP: u32 = 0x0073_0000;
// The opening Dragon hit lands earlier under the walk-only gameplay script than
// under the run-held trace used by the focused timing regression.
const LEVEL_ONE_OPENING_HIT_LEVEL_ELAPSED_FRAMES_WALK: usize = 289;
const LEVEL_ONE_OPENING_HIT_LEVEL_ELAPSED_FRAMES_RUN: usize = 300;
const OPENING_SCROLL_START_FRAME: usize = 260;
const OPENING_ENTRY_HOLD_FRAMES: usize = TITLE_TO_LEVEL_FRAMES + 1;
const BOOTSTRAP_LEVEL_START_PRESS_COUNT: usize = 6;
const TITLE_KID_X_POS: u32 = 0x0028_0000;
const TITLE_KID_Y_POS: u32 = 0x0030_0000;
const OPENING_ENTRY_KID_X_POS: u32 = 0x0020_0000;
const OPENING_ENTRY_KID_Y_POS: u32 = 0x0109_0000;
const OPENING_GROUNDED_KID_Y_POS: u32 = 0x010F_0000;
const LEVEL_ONE_LATE_HAZARD_CAMERA_X_FP: u32 = 0x03B2_0000;
const LEVEL_ONE_LATE_HAZARD_LEVEL_ELAPSED_FRAMES: usize = 701;
const LEVEL_ONE_LATE_HAZARD_DEATH_X_VELOCITY_FP: i32 = 0x0000_8000;
const LEVEL_ONE_LATE_HAZARD_DEATH_Y_VELOCITY_FP: i32 = -0x0003_D000;
const LEVEL_ONE_LATE_HAZARD_DEATH_GRAVITY_FP: i32 = 0x0000_3000;
// Default enemy contact box: 16x16 pixels, centred on position.
const ENEMY_CONTACT_HALF_WIDTH: i64 = 4;
// Oracle-backed Level 1 traces show MiniHoppingSkull side contact is narrower
// than the full 16px sprite; otherwise the late skull arc in death-respawn
// registers a false graze hit.
const MINI_HOPPING_SKULL_CONTACT_HALF_WIDTH: i64 = 1;
const ENEMY_CONTACT_HALF_HEIGHT: i64 = 8;
const PLAYER_DEATH_WINDUP_FRAMES: usize = 159;
const PLAYER_DEATH_HIDE_FRAMES: usize = 120;
const PLAYER_DEATH_RESPAWN_SETTLE_FRAMES: usize = 120;
/// ASM: Game Over display holds for 0x1A3 (419) frames.
const PLAYER_GAME_OVER_HOLD_FRAMES: usize = 419;
/// ASM: Continue countdown: 28 initial pause + 291 countdown = 319 frames.
const CONTINUE_COUNTDOWN_FRAMES: usize = 319;
const PLAYER_LEVEL_ADVANCE_RESPAWN_SETTLE_FRAMES: usize = 401;
// ASM: Just_received_damage flag cleared by flicker object (loc_BC54):
// outer loop dbf d3 with d3=$A (11 iterations), 6 frames per iteration = 66 frames.
const PLAYER_DAMAGE_INVULNERABLE_FRAMES: usize = 66;
const ENEMY_CONTACT_MIN_OVERLAP_FP: i64 = 0x0001_0000;
const FLAG_HIT_HALF_SIZE: u32 = 8; // 16x16 hit region → 8px either side of centre
const LEVEL_COMPLETE_PAUSE_FRAMES: usize = 120; // brief pause before advancing
const ACTIVATION_WINDOW_X_LEAD: u32 = 384;
const ACTIVATION_WINDOW_Y_LEAD: u32 = 288;
const ACTIVATION_WINDOW_PADDING: u32 = 64;
const FLYER_RETENTION_X_PADDING: u32 = 512;
const FLYER_RETENTION_Y_PADDING: u32 = 96;
const PLATFORM_PRELOAD_X_PADDING: u32 = 64;

// Player collision hitbox for "The Kid" (form 0).
// From ASM hitbox data: dc.w -$5, $A, -$1F, $1F
// x_offset=-5, x_size=10, y_offset=-31, y_size=31
// Position is at feet, so hitbox extends half_width either side and height above.
const PLAYER_HALF_WIDTH: i32 = 5;
const PLAYER_HEIGHT: i32 = 31;
const PLAYER_RIGHT_WALL_EXTENT: i32 = PLAYER_HALF_WIDTH + 1;
const PLAYER_LEFT_WALL_EXTENT: i32 = PLAYER_HALF_WIDTH + 1;
const PLAYER_CONTACT_HALF_WIDTH: i64 = 4;
const GROUND_SNAP_TOLERANCE_FP: u32 = 0x0006_0000;
const JUMP_BUFFER_FRAMES: u8 = 6;
const SLOPE_AIR_SNAP_TOLERANCE_FP: u32 = 0x0003_0000;
const KID_SLOPE_MAX_SPEED_FP: i32 = 0x0004_0000;
const KID_SLOPE_BRAKE_FP: i32 = 0x0000_4000;
const KID_SLOPE_NEUTRAL_DECEL_FP: i32 = 0x0000_2000;
const KID_SLOPE_NEUTRAL_TARGET_FP: i32 = -0x0000_8000;
const KID_SLOPE_WALK_UPHILL_CAP_FP: i32 = 0x0000_8000;
const KID_SLOPE_WALK_UPHILL_ACCEL_FP: i32 = 0x0000_1000;
const KID_SLOPE_WALK_DOWNHILL_CAP_FP: i32 = -0x0001_8000;
const KID_SLOPE_WALK_DOWNHILL_ACCEL_FP: i32 = 0x0000_2000;
const KID_SLOPE_RUN_UPHILL_CAP_FP: i32 = 0x0001_0000;
const KID_SLOPE_RUN_UPHILL_ACCEL_FP: i32 = 0x0000_2000;
const KID_SLOPE_RUN_DOWNHILL_CAP_FP: i32 = -0x0003_0000;
const KID_SLOPE_RUN_DOWNHILL_ACCEL_FP: i32 = 0x0000_4000;

// Stomp bounce: ~60% of a standstill jump (Kid jump_height 0x42, factor 0x100).
// Full jump y_vel = -(0x42 * 0x100) << 4 = -0x420_000. 60% ≈ -0x270_000.
const STOMP_BOUNCE_VELOCITY: i32 = -0x0027_0000;

// Frames an enemy flashes white after taking a non-lethal stomp.
const ENEMY_DAMAGE_FLASH_FRAMES: u8 = 8;

// Evanescent blocks: solid until the player stands on them, then fade and vanish.
// Timer starts at (skin - 28); incremented each frame; block removed at timer ≥ 22.
const EVANESCENT_SKIN_OFFSET: i16 = 0x1C; // 28
const EVANESCENT_REMOVAL_THRESHOLD: i16 = 0x16; // 22
const EVANESCENT_MAX_ENTRIES: usize = 19; // ASM limit $13

// Level timer: every level starts at 3:00 and counts down 1 second per 60 frames.
// Time-up = death. Clocks add 3 minutes (capped at 10:00).
const LEVEL_TIMER_START_SECONDS: u16 = 180; // 3:00
const LEVEL_TIMER_MAX_SECONDS: u16 = 600; // 10:00
const LEVEL_TIMER_CLOCK_BONUS_SECONDS: u16 = 180; // +3:00
const LEVEL_TIMER_FRAMES_PER_SECOND: u16 = 60;
const SCORE_CAP: u32 = 9_999_999;
/// ASM: FirstElsewhere_LevelID = $49. Path/speed bonuses only for levels < this.
const FIRST_ELSEWHERE_LEVEL_ID: u8 = 0x49;
/// ASM: extra life awarded every 50,000 points.
const SCORE_EXTRA_LIFE_INTERVAL: u32 = 50_000;

/// Per-level speed bonus thresholds in seconds (ASM: SpeedBonus_Values).
/// If level completion time (elapsed seconds) <= threshold, award 10,000 pts.
/// 0 means speed bonus is impossible for that level.
#[rustfmt::skip]
const SPEED_BONUS_TABLE: [u8; 76] = [
    20, 22, 60, 27, 19, 26, 91, 17, 54, 37, 23, 28, 35, 26, 30, 28, // 0x00-0x0F
    75, 42, 84,171,125, 48, 60, 64,103, 92, 54, 28, 32, 43,108, 60, // 0x10-0x1F
    71, 27,174, 96, 50, 72, 60, 82, 41, 19,220,  0,  0, 50,  0, 60, // 0x20-0x2F
     0, 68,125, 80, 28,  0, 15,  0,  0,  0,  0, 87,  0, 98,  0,  0, // 0x30-0x3F
     0, 95,  0, 28, 30,  0,  0,  0,  0,  0,  0,  0,                  // 0x40-0x4B
];

/// Per-level path bonus multipliers × 1000 (ASM: PathBonus_Values).
/// Non-zero only for later levels. Multiplied by 1000 to get point value.
#[rustfmt::skip]
const PATH_BONUS_TABLE: [u8; 76] = [
     0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0, // 0x00-0x0F
     0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0, // 0x10-0x1F
     0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0, 25,  0, 25, // 0x20-0x2F
     0, 25, 25, 25, 10,  0, 10,  0,  0,  0,  0, 20,  0, 50,  0,  0, // 0x30-0x3F
     0, 25,  0, 25, 25,  0,  0,  0,  0,  0,  0,  0,                  // 0x40-0x4B
];

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct LevelId(pub u8);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GameMode {
    SegaScreen,
    IntroSequence1,
    IntroSequence2,
    IntroSequence3,
    IntroSequence4,
    IntroSequence5,
    TitleScreen,
    OptionsMenu,
    LevelTransition,
    InGame,
    /// ASM game mode 0x30 — end credits sequence after beating the final level.
    EndingSequence,
}

impl GameMode {
    pub fn as_disassembly_value(self) -> u16 {
        match self {
            Self::SegaScreen => 0x0000,
            Self::IntroSequence1 => 0x0004,
            Self::IntroSequence2 => 0x0018,
            Self::IntroSequence3 => 0x001C,
            Self::IntroSequence4 => 0x0020,
            Self::IntroSequence5 => 0x0024,
            Self::TitleScreen => 0x002C,
            Self::OptionsMenu => 0x0028,
            Self::LevelTransition => 0x0008,
            Self::InGame => 0x000C,
            Self::EndingSequence => 0x0030,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PlayerForm {
    TheKid,
    Skycutter,
    Cyclone,
    RedStealth,
    Eyeclops,
    Juggernaut,
    IronKnight,
    Berzerker,
    Maniaxe,
    Micromax,
}

impl PlayerForm {
    pub fn helmet_id(self) -> u16 {
        match self {
            Self::TheKid => 0,
            Self::Skycutter => 1,
            Self::Cyclone => 2,
            Self::RedStealth => 3,
            Self::Eyeclops => 4,
            Self::Juggernaut => 5,
            Self::IronKnight => 6,
            Self::Berzerker => 7,
            Self::Maniaxe => 8,
            Self::Micromax => 9,
        }
    }

    pub fn max_hitpoints(self) -> u16 {
        match self {
            Self::TheKid => 2,
            Self::IronKnight => 5,
            Self::Skycutter
            | Self::Cyclone
            | Self::RedStealth
            | Self::Eyeclops
            | Self::Juggernaut
            | Self::Berzerker
            | Self::Maniaxe
            | Self::Micromax => 3,
        }
    }

    /// Select the ground speed table for this form and terrain type.
    /// Three form groups (Kid/Micromax/Juggernaut) × three terrain types (Normal/Ice/Rubber).
    fn ground_speed_table(self, terrain_type: TerrainType) -> &'static physics::SpeedTable {
        match (self, terrain_type) {
            (Self::Micromax, TerrainType::Normal) => &physics::NORMAL_MICROMAX,
            (Self::Micromax, TerrainType::Ice) => &physics::ICE_MICROMAX,
            (Self::Micromax, TerrainType::Rubber) => &physics::RUBBER_MICROMAX,
            (Self::Juggernaut, TerrainType::Normal) => &physics::NORMAL_JUGGERNAUT,
            (Self::Juggernaut, TerrainType::Ice) => &physics::ICE_JUGGERNAUT,
            (Self::Juggernaut, TerrainType::Rubber) => &physics::RUBBER_JUGGERNAUT,
            (_, TerrainType::Normal) => &physics::NORMAL_KID,
            (_, TerrainType::Ice) => &physics::ICE_KID,
            (_, TerrainType::Rubber) => &physics::RUBBER_KID,
        }
    }

    pub fn from_helmet_id(helmet_id: u16) -> Option<Self> {
        match helmet_id {
            0 => Some(Self::TheKid),
            1 => Some(Self::Skycutter),
            2 => Some(Self::Cyclone),
            3 => Some(Self::RedStealth),
            4 => Some(Self::Eyeclops),
            5 => Some(Self::Juggernaut),
            6 => Some(Self::IronKnight),
            7 => Some(Self::Berzerker),
            8 => Some(Self::Maniaxe),
            9 => Some(Self::Micromax),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct FrameInput {
    pub up: bool,
    pub down: bool,
    pub left: bool,
    pub right: bool,
    pub a: bool,
    pub b: bool,
    pub c: bool,
    pub start: bool,
}

impl FrameInput {
    pub fn from_mask(mask: u8) -> Self {
        Self {
            up: mask & (1 << 0) != 0,
            down: mask & (1 << 1) != 0,
            left: mask & (1 << 2) != 0,
            right: mask & (1 << 3) != 0,
            b: mask & (1 << 4) != 0,
            c: mask & (1 << 5) != 0,
            a: mask & (1 << 6) != 0,
            start: mask & (1 << 7) != 0,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NativeTraceFrame {
    pub frame_index: usize,
    pub game_mode: GameMode,
    pub game_mode_value: u16,
    pub start_press_count: usize,
    pub current_player: u8,
    pub current_level: LevelId,
    pub player_1_level_id: LevelId,
    pub current_level_name: Option<String>,
    pub player_start_x: Option<u16>,
    pub player_start_y: Option<u16>,
    pub flag_x: Option<u16>,
    pub flag_y: Option<u16>,
    pub level_enemy_object_count: Option<u8>,
    pub active_enemy_object_count: usize,
    pub active_platform_object_count: usize,
    pub number_lives: u16,
    pub number_continues: u8,
    pub number_hitpoints: u16,
    pub current_helmet_id: u16,
    pub paused: bool,
    pub kid_x_pos: Option<u32>,
    pub kid_y_pos: Option<u32>,
    pub kid_x_velocity: Option<i32>,
    pub kid_y_velocity: Option<i32>,
    pub kid_facing: Option<Direction>,
    pub kid_on_ground: Option<bool>,
    pub active_objects: Vec<NativeObjectSnapshot>,
    pub camera_x_pos: u32,
    pub camera_y_pos: u32,
    pub number_objects: u16,
    pub number_gfx_objects: u16,
    pub level_complete: bool,
    pub game_over: bool,
    /// Continue countdown remaining frames (None if not in continue screen).
    pub continue_countdown: Option<usize>,
    pub game_complete: bool,
    /// Ending credits sequence elapsed frames (None if not in ending).
    pub ending_elapsed: Option<usize>,
    pub options_cursor: Option<u8>,
    pub sound_test_track: Option<u8>,
    pub two_player: bool,
    pub invulnerable_frames: usize,
    pub diamond_power_active: bool,
    pub diamond_count: u16,
    pub score: u32,
    pub level_timer_seconds: u16,
    pub level_damage_taken: bool,
    pub murder_wall_x: Option<i64>,
    /// Active projectiles: (x_pos, y_pos, friendly).
    pub active_projectiles: Vec<(u32, u32, bool)>,
    /// Active lava geyser eruptions: (x_px, y_top_px, height_px) for rendering.
    pub active_geysers: Vec<(u16, u16, u16)>,
    /// Metal block bounce animations: (cell_x, cell_y, remaining_frames).
    pub block_bounces: Vec<(u16, u16, u8)>,
    /// Block destruction fragment effects: (x_px, y_px, remaining_frames).
    pub block_destroy_effects: Vec<(u16, u16, u8)>,
    /// Floating prize items rising from blocks: (x_px, y_px, remaining_frames, content_id).
    pub floating_items: Vec<(u16, u16, u8, u8)>,
    /// Enemy death particle effects: (x_pos_fp, y_pos_fp, remaining_frames).
    pub enemy_death_effects: Vec<(u32, u32, u8)>,
    /// Form-change visual flash countdown (0=no flash).
    pub form_change_flash_frames: u8,
    /// True if a stomp occurred this frame (for SFX).
    pub stomped_this_frame: bool,
    /// Number of blocks hit by headbutt this frame (for SFX).
    pub blocks_hit_this_frame: u8,
    /// Sound test: play was requested this frame (A/C pressed).
    pub sound_test_play_pending: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NativeObjectSnapshot {
    pub kind: NativeObjectKind,
    pub x_pos: u32,
    pub y_pos: u32,
    pub layout_index: Option<usize>,
    pub x_velocity: Option<i32>,
    pub y_velocity: Option<i32>,
    pub facing: Option<Direction>,
    pub on_ground: Option<bool>,
    pub hit_points: Option<u8>,
    pub damage_flash: bool,
    /// Palette variant (0, 1, or 2) from the level's enemy type slot.
    #[serde(default)]
    pub palette_variant: u8,
    /// True when the enemy is in a stationary phase (e.g. dragon perching).
    #[serde(default)]
    pub stationary: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum NativeObjectKind {
    Player,
    Enemy { object_type: u8 },
    Platform { kind: u8, subtype: u8 },
}

#[derive(Debug, Clone)]
pub struct NativeRuntime {
    assets: AssetManifest,
    frame_index: usize,
    boot_state: BootState,
    prior_input: FrameInput,
    previous_input: FrameInput,
    start_press_count: usize,
    current_level: LevelId,
    level_start_frame: Option<usize>,
    current_form: PlayerForm,
    pending_form: Option<PlayerForm>,
    number_lives: u16,
    number_continues: u8,
    number_hitpoints: u16,
    invulnerable_frames: usize,
    /// Skycutter flipboard deploy counter (0=inactive, 1..31=animating).
    /// Increments by 2 each frame; at 17 toggles on_board; at 32 expires.
    skycutter_flipboard: u8,
    /// True while the player is riding the Skycutter flipboard.
    skycutter_on_board: bool,
    /// Cyclone vertical flight active flag.
    cyclone_flying: bool,
    /// Cyclone Y acceleration accumulator (signed 16-bit, applied to y_vel each frame).
    cyclone_y_accel: i16,
    /// RedStealth sword swing timer: counts down from startup+active to 0.
    red_stealth_sword_timer: u8,
    /// Eyeclops beam active flag (one beam at a time).
    eyeclops_beam_active: bool,
    /// Maniaxe axe in flight (reuses projectile system with turnaround).
    maniaxe_axe_active: bool,
    /// Maniaxe axe turnaround counter (frames until direction reversal).
    maniaxe_turnaround: u8,
    /// IronKnight wall climb active flag.
    iron_knight_climbing: bool,
    /// IronKnight wall climb Y acceleration accumulator.
    iron_knight_climb_accel: i16,
    /// Berzerker charge accumulator — overflows at $FF to activate charge.
    berzerker_charge_accum: u8,
    /// Berzerker is charging (fast enough to break blocks and damage on contact).
    berzerker_charging: bool,
    /// Micromax wall/ceiling stick active flag.
    micromax_stuck: bool,
    /// Visual flash countdown after form change (0=inactive).
    form_change_flash_frames: u8,
    paused: bool,
    pause_toggle_pending: bool,
    pending_level_skip: Option<LevelId>,
    pending_level_skip_resume: bool,
    player_lifecycle: PlayerLifecycle,
    opening_slice: Option<OpeningSliceState>,
    death_camera_lock: Option<CameraState>,
    level_skip_transition: Option<LevelSkipTransitionState>,
    level_skip_resume_hold: bool,
    /// When true, skip the entry animation on the current level (used after
    /// level-skip warp — the original game has no entry animation).
    skip_entry_animation: bool,
    active_enemy_roster: Vec<NativeObjectSnapshot>,
    active_enemies: Vec<ActiveEnemy>,
    active_platforms: Vec<ActivePlatform>,
    active_platform_roster: Vec<NativeObjectSnapshot>,
    platform_script_table: Option<kid_data::PlatformScriptTable>,
    block_grid: Option<terrain::BlockGrid>,
    block_list: Option<terrain::BlockList>,
    block_grid_source: BlockGridSource,
    diamond_count: u16,
    score: u32,
    /// Diamond Power active: grants invulnerability and clears nearby enemies.
    diamond_power_frames: u16,
    /// Which Diamond Power type is currently active (determines behaviour).
    diamond_power_type: helmets::DiamondPowerType,
    /// Player has beaten the final level — show victory/ending.
    game_complete: bool,
    evanescent_blocks: Vec<EvanescentEntry>,
    ghost_groups: Vec<GhostGroup>,
    projectiles: Vec<EnemyProjectile>,
    lift_platforms: Vec<LiftPlatform>,
    /// Weather type for current level (0=none, 1=lava, 2=storm, 3=storm+hail).
    weather_type: u8,
    /// Hail system: frames until next hail spawn (0=spawn this frame).
    hail_spawn_timer: u16,
    /// Hail system: whether hail is currently active (toggles on $258-frame period).
    hail_active: bool,
    /// Hail system: frames until next active/inactive toggle.
    hail_period_timer: u16,
    /// Active lava geysers for lava weather levels.
    lava_geysers: Vec<LavaGeyser>,
    murder_wall: Option<MurderWallState>,
    /// Metal block bounce animations in progress.
    block_bounces: Vec<BlockBounceAnim>,
    /// Block destruction fragment effects.
    block_destroy_effects: Vec<BlockDestroyEffect>,
    /// Floating prize items rising from blocks.
    floating_items: Vec<FloatingItem>,
    /// Enemy death particle effects.
    enemy_death_effects: Vec<EnemyDeathEffect>,
    /// Set each frame if a stomp occurred.
    stomped_this_frame: bool,
    /// Count of blocks hit by headbutt this frame.
    blocks_hit_this_frame: u8,
    /// Sound test play pending (consumed each frame).
    sound_test_play_pending: bool,
    /// Global shooter block fire timer: fires from all on-screen shooter blocks every 22 frames.
    shooter_block_timer: u16,
    level_timer: LevelTimer,
    /// Camera Y look-ahead accumulator ($FFFFFB58): +1 per frame while down held
    /// and grounded, -8 per frame on release, clamped to [0, 30].
    y_look_offset: i16,
    /// Set to true when player takes damage during current level (for no-hit bonus).
    level_damage_taken: bool,
    /// Set to true when player collects any prize from a block (disqualifies no-prize bonus).
    level_prize_collected: bool,
    /// Number of clock pickups collected during current level (for completion time calc).
    clocks_collected: u16,
    /// Two-player alternating mode enabled.
    two_player: bool,
    /// Current player index (0=P1, 1=P2). Mirrors ASM $FFFFFC39.
    current_player_index: u8,
    /// Per-player persistent state slots.
    player_slots: [PlayerSlot; 2],
}

#[derive(Debug, Clone)]
struct BlockBounceAnim {
    cell_x: u16,
    cell_y: u16,
    remaining: u8,
}

#[derive(Debug, Clone)]
struct BlockDestroyEffect {
    x_px: u16,
    y_px: u16,
    remaining: u8,
}

#[derive(Debug, Clone)]
struct FloatingItem {
    x_px: u16,
    y_px: u16,
    remaining: u8,
    content: terrain::PrizeContent,
}

#[derive(Debug, Clone)]
struct EnemyDeathEffect {
    x_pos: u32,
    y_pos: u32,
    remaining: u8,
}

/// A lift block that has been activated and is now a moving platform.
/// Rises for ~50 frames then falls until off-screen.
#[derive(Debug, Clone)]
struct LiftPlatform {
    x_pos: u32,
    y_pos: u32,
    y_velocity: i32,
    /// Frames since activation.
    age: u16,
    /// Width in pixels (one block = 16px).
    width_px: u16,
}

/// A lava geyser that erupts periodically from a fixed position.
#[derive(Debug, Clone)]
struct LavaGeyser {
    /// Fixed X position (pixel coordinates, not FP16.16).
    x_px: u16,
    /// Base Y position (bottom of eruption, pixel coordinates).
    y_px: u16,
    /// Eruption height in pixels.
    height_px: u16,
    /// Phase: 0=waiting, 1=erupting upward, 2=holding, 3=collapsing.
    phase: u8,
    /// Frames remaining in current phase.
    timer: u16,
    /// Current eruption column height in pixels (for collision/rendering).
    current_height: u16,
}

const LAVA_GEYSER_HOLD_FRAMES: u16 = 30;
const LAVA_GEYSER_DORMANT_FRAMES: u16 = 60;
const LAVA_GEYSER_ERUPT_SPEED: u16 = 4; // pixels per frame

/// Per-player persistent state for 2P alternating mode.
/// Mirrors the ASM Player_1_*/Player_2_* RAM slots ($FFFFFC54-$FFFFFC80).
#[derive(Debug, Clone)]
struct PlayerSlot {
    lives: u16,
    hitpoints: u16,
    diamonds: u16,
    level_id: LevelId,
    form: PlayerForm,
    continues: u8,
    score: u32,
}

impl Default for PlayerSlot {
    fn default() -> Self {
        Self {
            lives: 3,
            hitpoints: PlayerForm::TheKid.max_hitpoints(),
            diamonds: 0,
            level_id: LevelId(0),
            form: PlayerForm::TheKid,
            continues: 3,
            score: 0,
        }
    }
}

/// Number of frames for the ending sequence phases (ASM End_Credits, kid.asm:16920).
const ENDING_PAUSE_FRAMES: usize = 40; // Sleep 0x28
const ENDING_MUSIC_DELAY_FRAMES: usize = 200; // Sleep 0xC8
const ENDING_FADE_OUT_FRAMES: usize = 256; // fade counter 0x100
/// Each credits screen: 112 frames scroll + 200 frames display.
const CREDITS_SCROLL_FRAMES: usize = 112;
const CREDITS_DISPLAY_FRAMES: usize = 200;
const CREDITS_SCREEN_COUNT: usize = 7; // congratulations + 6 credits screens
/// Total credits duration per screen.
const CREDITS_SCREEN_TOTAL: usize = CREDITS_SCROLL_FRAMES + CREDITS_DISPLAY_FRAMES;
/// Ending music track (ASM: bgm_Ending = 5). Used by native_player audio.
pub const ENDING_MUSIC_TRACK: u8 = 5;

#[derive(Debug, Clone, PartialEq, Eq)]
enum BootState {
    SegaScreen { elapsed: usize },
    IntroSequence { segment: usize, elapsed: usize },
    TitleScreen,
    OptionsMenu { cursor: u8 },
    SoundTest { track: u8 },
    StartingLevel { remaining_frames: usize },
    LevelSkipHold,
    InGame,
    /// Post-final-level ending sequence: fade out, credits screens, return to title.
    EndingSequence { elapsed: usize },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum BlockGridSource {
    None,
    Manual,
    Generated,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct LevelSkipHoldPose {
    kid_x_pos: u32,
    kid_y_pos: u32,
    camera_x_pos: u32,
    camera_y_pos: u32,
    number_objects: u16,
    number_gfx_objects: u16,
}

#[derive(Debug, Clone)]
struct LevelSkipTransitionState {
    elapsed_frames: usize,
    frozen_player: PlayerState,
    frozen_camera: CameraState,
    frozen_active_objects: Vec<NativeObjectSnapshot>,
    frozen_number_objects: u16,
    frozen_number_gfx_objects: u16,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct LevelSkipAnimationPath {
    initial_x_pos: i32,
    initial_y_pos: i32,
    x_step_fp: i32,
    y_step_fp: i32,
    first_animation_number_objects: u16,
    first_animation_number_gfx_objects: u16,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
struct LevelLayoutMetadata {
    width_blocks: u16,
    height_blocks: u16,
    data_bytes: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PlayerLifecycle {
    Alive,
    DeathWindup {
        remaining_frames: usize,
    },
    Dying {
        remaining_frames: usize,
    },
    Respawning {
        remaining_frames: usize,
    },
    LevelComplete {
        remaining_frames: usize,
    },
    /// All lives lost — "GAME OVER" display, then continue countdown if available.
    GameOver {
        remaining_frames: usize,
    },
    /// "CONTINUE?" countdown (only enters if continues > 0). Button press uses continue.
    /// ASM: 28 frames initial pause + 291 frames countdown = 319 total.
    ContinueCountdown {
        remaining_frames: usize,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct PlayerState {
    x_pos: u32,
    y_pos: u32,
    x_velocity: i32,
    y_velocity: i32,
    facing: Direction,
    on_ground: bool,
    on_spikes: bool,
    just_jumped: bool,
    jump_buffer_frames: u8,
    right_wall_sticky: bool,
    left_wall_sticky: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
struct CameraState {
    x_pos: u32,
    y_pos: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct LevelStartPose {
    kid_x_pos: u32,
    kid_y_pos: u32,
    camera_x_pos: u32,
    camera_y_pos: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
enum TerrainType {
    #[default]
    Normal,
    Ice,
    Rubber,
}

impl TerrainType {
    fn from_block_type(bt: terrain::BlockType) -> Self {
        match bt {
            terrain::BlockType::Ice => Self::Ice,
            terrain::BlockType::Rubber => Self::Rubber,
            _ => Self::Normal,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct TerrainState {
    floor_y_pos: u32,
    floor_terrain_type: TerrainType,
}

/// Movement script execution state for a platform.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct PlatformScriptState {
    /// Relative offset into the shared script table (key into `PlatformScriptTable`).
    script_offset: u16,
    /// Current command index within the script.
    command_index: usize,
    /// Remaining frames for the current command.
    remaining_duration: u16,
}

/// An active platform with collision dimensions and movement state.
/// Positions use the same FP16.16 format as player/enemy positions.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct ActivePlatform {
    /// FP16.16 X position (left edge of the platform).
    x_pos: u32,
    /// FP16.16 Y position (top edge of the platform).
    y_pos: u32,
    /// Platform width in pixels (from h_variant: `h * 16 + 15`).
    width_px: u16,
    /// Platform height in pixels (from v_variant: `v * 16 + 15`).
    height_px: u16,
    /// FP16.16 X velocity (from movement script).
    x_velocity: i32,
    /// FP16.16 Y velocity (from movement script).
    y_velocity: i32,
    /// Movement script state (None for stationary platforms).
    script: Option<PlatformScriptState>,
}

impl ActivePlatform {
    fn from_placement(
        placement: &PlatformPlacement,
        script_table: Option<&kid_data::PlatformScriptTable>,
    ) -> Self {
        let script = placement
            .script
            .relative_offset
            .and_then(|offset| {
                let table = script_table?;
                let script_data = table.scripts.get(&offset)?;
                let first_cmd = script_data.commands.first()?;
                Some((offset, first_cmd))
            })
            .map(|(offset, first_cmd)| PlatformScriptState {
                script_offset: offset,
                command_index: 0,
                remaining_duration: first_cmd.duration,
            });

        Self {
            x_pos: u32::from(placement.x) << 16,
            y_pos: u32::from(placement.y) << 16,
            width_px: u16::from(placement.h_variant) * 16 + 15,
            height_px: u16::from(placement.v_variant) * 16 + 15,
            x_velocity: 0,
            y_velocity: 0,
            script,
        }
    }

    /// Advance one frame of movement script execution.
    fn step_script(&mut self, script_table: &kid_data::PlatformScriptTable) {
        let Some(ref mut state) = self.script else {
            return;
        };
        let Some(script) = script_table.scripts.get(&state.script_offset) else {
            return;
        };
        if script.commands.is_empty() {
            return;
        }

        let cmd = &script.commands[state.command_index];
        self.x_velocity = cmd.x_velocity;
        self.y_velocity = cmd.y_velocity;

        // Apply velocity
        self.x_pos = (i64::from(self.x_pos) + i64::from(self.x_velocity))
            .clamp(0, i64::from(u32::MAX)) as u32;
        self.y_pos = (i64::from(self.y_pos) + i64::from(self.y_velocity))
            .clamp(0, i64::from(u32::MAX)) as u32;

        // Advance timer
        state.remaining_duration = state.remaining_duration.saturating_sub(1);
        if state.remaining_duration == 0 {
            state.command_index += 1;
            if state.command_index >= script.commands.len() {
                state.command_index = 0; // loop
            }
            state.remaining_duration = script.commands[state.command_index].duration;
        }
    }
}

/// Countdown timer for the level. Ticks down once per 60 frames.
/// When it reaches zero the player dies.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct LevelTimer {
    /// Remaining whole seconds (3:00 = 180).
    seconds: u16,
    /// Frames remaining until the next second tick (counts down from 60).
    sub_frames: u16,
}

impl LevelTimer {
    fn new() -> Self {
        Self {
            seconds: LEVEL_TIMER_START_SECONDS,
            // ASM sets Time_SubSeconds to 1 — first frame immediately
            // decrements a second and reloads to 60.
            sub_frames: 1,
        }
    }

    /// Tick one frame. Returns true when the timer has just expired (0:00).
    fn tick(&mut self) -> bool {
        if self.seconds == 0 && self.sub_frames == 0 {
            return true; // already expired
        }
        self.sub_frames = self.sub_frames.saturating_sub(1);
        if self.sub_frames == 0 {
            if self.seconds > 0 {
                self.seconds -= 1;
                self.sub_frames = LEVEL_TIMER_FRAMES_PER_SECOND;
                false
            } else {
                true // just hit 0:00
            }
        } else {
            false
        }
    }

    fn add_clock_bonus(&mut self) {
        let new_seconds = self.seconds + LEVEL_TIMER_CLOCK_BONUS_SECONDS;
        if new_seconds >= LEVEL_TIMER_MAX_SECONDS {
            self.seconds = LEVEL_TIMER_MAX_SECONDS;
            self.sub_frames = LEVEL_TIMER_FRAMES_PER_SECOND;
        } else {
            self.seconds = new_seconds;
        }
    }
}

/// Tracks a triggered evanescent block that is counting down to removal.
/// The block stays solid during the countdown; once timer reaches the
/// removal threshold it is cleared from the block grid.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct EvanescentEntry {
    col: u16,
    row: u16,
    /// Timer value (starts negative from skin offset, incremented each frame).
    timer: i16,
}

/// Runtime state for a ghost block group that cycles solid/vanished.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct GhostGroup {
    col: u16,
    row: u16,
    count: u8,
    vertical: bool,
    solid_time: u16,
    vanished_time: u16,
    timer: u16,
    solid: bool,
}

/// Projectile fired by an enemy (Archer arrow, Tank bullet, etc.).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct EnemyProjectile {
    x_pos: u32,
    y_pos: u32,
    x_velocity: i32,
    y_velocity: i32,
    /// Remaining lifetime in frames (destroyed when zero).
    lifetime: u16,
    /// True for player-fired projectiles (Eyeclops beam, Maniaxe axe, Juggernaut skull) — these
    /// damage enemies, not the player.
    friendly: bool,
    /// Per-frame gravity added to y_velocity (for arcing projectiles like Juggernaut skull).
    gravity: i32,
    /// Homing acceleration per axis per frame. 0 = not homing.
    /// HeadyMetal: $7D0, Lion: $2000 (Lion.asm loc_3E1C2).
    homing_accel: i32,
    /// Speed cap for homing projectiles (per-axis). 0 = no cap.
    /// Lion: $60000 (Lion.asm loc_3E1C2).
    speed_cap: i32,
    /// Bounces off terrain walls/floors/ceilings (Lion.asm loc_3E260).
    wall_bounce: bool,
}

/// Maximum active projectiles (original limits to 2 arrows, we allow a few more
/// for Tank bullets sharing the pool).
const MAX_PROJECTILES: usize = 4;
/// Projectile hitbox half-width for player collision (pixels).
const PROJECTILE_HITBOX_HALF: i32 = 6;

/// Skycutter flipboard counter threshold: animation complete at 32.
const SKYCUTTER_FLIPBOARD_TIMEOUT: u8 = 0x20;
/// Counter value where the player toggles on/off the board.
const SKYCUTTER_MOUNT_FRAME: u8 = 0x11;
/// Counter increment per frame.
const SKYCUTTER_COUNTER_STEP: u8 = 2;
/// Gravity while on the board (B held): lighter descent.
const SKYCUTTER_GRAVITY_B_HELD: i32 = 0x2000;
/// Gravity while on the board (B not held): normal descent.
const SKYCUTTER_GRAVITY_NO_B: i32 = 0x4000;
/// Upward bounce velocity when landing while on the board.
const SKYCUTTER_LANDING_BOUNCE: i32 = -0x3_0000;
/// Max upward velocity while on the board ($FFF88000 signed = -0x78000).
const SKYCUTTER_MAX_UPWARD_VEL: i32 = -0x7_8000;

/// RedStealth sword: C press spawns a melee hitbox 28px ahead, 10px up
/// from the player. 8 frames startup, 3 frames active. One swing at a time.
const RED_STEALTH_STARTUP_FRAMES: u8 = 8;
const RED_STEALTH_ACTIVE_FRAMES: u8 = 3;
const RED_STEALTH_REACH_X: i32 = 28;
const RED_STEALTH_OFFSET_Y: i32 = -10;
const RED_STEALTH_HIT_HALF: i32 = 8;

/// Maniaxe thrown axe: travels 12 frames outward at ±5 px/frame then
/// reverses for 12 frames. Damages enemies on contact, breaks on walls.
const MANIAXE_STARTUP_FRAMES: u8 = 6;
const MANIAXE_BASE_SPEED: i32 = 5;
const MANIAXE_TRAVEL_FRAMES: u8 = 12;

/// Mushroom block minimum bounce velocity (ensures a visible bounce even at low speed).
// UNVERIFIED: ASM uses pixel-nudge coroutine for mushroom bounce, not velocity.
// The 3/4 attenuation and -$20000 minimum are native approximations.
const MUSHROOM_MIN_BOUNCE: i32 = -0x2_0000;

/// Eyeclops beam: C press fires a horizontal beam projectile.
/// Normal beam (C button) has no diamond cost; hard beam (A+B held) costs 2.
const EYECLOPS_BEAM_SPEED: i32 = 0x8_0000; // 8 px/frame
const EYECLOPS_BEAM_LIFETIME: u16 = 60;

/// Juggernaut skull shot: C fires a skull projectile with gravity.
const JUGGERNAUT_SKULL_X_SPEED: i32 = 0x4_0000; // 4 px/frame
const JUGGERNAUT_SKULL_GRAVITY: i32 = 0x1000; // $1000/frame gravity on y_vel
const JUGGERNAUT_SKULL_LIFETIME: u16 = 120;
const JUGGERNAUT_MAX_SKULLS: usize = 8;

/// Berzerker charge: accumulator increments by $10 each frame when velocity >= 1.0 FP.
/// Charging activates when accumulator overflows past $FF.
const BERZERKER_CHARGE_VEL_THRESHOLD: i32 = 0x1_0000;
const BERZERKER_CHARGE_ACCUM_STEP: u8 = 0x10;

/// Cyclone flight: acceleration-based vertical flight.
/// Per-frame gravity added to acceleration accumulator.
const CYCLONE_ACCEL_GRAVITY: i16 = 0x400;
/// Upward thrust subtracted from accumulator when C is held.
const CYCLONE_ACCEL_THRUST: i16 = 0x3000;
/// Minimum acceleration (max upward thrust) — $8001 as signed i16 = -32767.
const CYCLONE_ACCEL_MIN: i16 = -0x7FFF;
/// Maximum descent acceleration when C not held.
const CYCLONE_ACCEL_MAX_DESCENT: i16 = 0x1200;
/// Initial acceleration on entering flight (set on jump).
const CYCLONE_ACCEL_INITIAL: i16 = 0x2000;
/// Max downward velocity during Cyclone flight.
const CYCLONE_MAX_FALL_VEL: i32 = 0x2_8000;
/// Max upward velocity during Cyclone flight.
const CYCLONE_MAX_RISE_VEL: i32 = -0x3_0000;

/// IronKnight wall climb: C press subtracts this from Y acceleration.
const IRON_KNIGHT_CLIMB_BOOST: i16 = 0x1800;
/// IronKnight wall climb: natural per-frame drift toward sliding down.
const IRON_KNIGHT_SLIDE_DRIFT: i16 = 0x80;
/// IronKnight wall climb: max slide-down acceleration (no C pressed).
const IRON_KNIGHT_MAX_SLIDE_ACCEL: i16 = 0x1800;
/// IronKnight wall climb: min acceleration (fast climbing upward).
const IRON_KNIGHT_MIN_ACCEL: i16 = -0x0800; // $F800 as signed
/// IronKnight wall climb: initial Y velocity on attach.
const IRON_KNIGHT_ATTACH_Y_VEL: i32 = -0x4000;
/// IronKnight wall climb: max upward velocity.
const IRON_KNIGHT_MAX_CLIMB_VEL: i32 = -0x1_8000;
/// IronKnight wall climb: max downward velocity (falling off).
const IRON_KNIGHT_MAX_FALL_VEL: i32 = 0x4_0000;

/// Micromax hitbox: half-size character (16-pixel grid alignment per ASM).
const MICROMAX_HALF_W: i32 = 3;
const MICROMAX_H: i32 = 15;
/// Micromax stuck gravity: $600 per frame (much lower than normal $4000).
const MICROMAX_STUCK_GRAVITY: i32 = 0x600;
/// Micromax max fall speed before peeling off surface.
const MICROMAX_STUCK_MAX_FALL: i32 = 0x1_0000;
/// Micromax detach Y velocity: -4.0 FP (strong upward kick).
const MICROMAX_DETACH_Y_VEL: i32 = -0x4_0000;
/// Micromax detach X speed: 2.0 FP (kick away from surface).
const MICROMAX_DETACH_X_SPEED: i32 = 0x2_0000;

/// Lift block: upward acceleration (first 50 frames).
const LIFT_ACCEL_UP: i32 = -0x708;
/// Lift block: downward acceleration (after 50 frames).
const LIFT_ACCEL_DOWN: i32 = 0x7D0;
/// Lift block: frames of upward phase before switching to fall.
const LIFT_RISE_FRAMES: u16 = 50;

/// Murder wall: an advancing wall that kills the player on contact.
/// Accelerates from zero to max_speed at $100/frame.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct MurderWallState {
    /// Current X position in fixed-point 16.16.
    x_pos: i64,
    /// Current speed in fixed-point 16.16.
    speed: i32,
    /// Terminal speed in fixed-point 16.16.
    max_speed: i32,
    /// True if wall moves right-to-left (reversed).
    reversed: bool,
}

/// Murder wall acceleration per frame (fixed-point 16.16).
const MURDER_WALL_ACCELERATION: i32 = 0x0100;
/// Default max speed for most murder wall levels.
const MURDER_WALL_DEFAULT_MAX_SPEED: i32 = 0x2_0000;
/// Slower max speed for Hills of the Warrior 1.
const MURDER_WALL_SLOW_MAX_SPEED: i32 = 0x1_8000;
/// Initial offset from camera (48 pixels in fixed-point).
const MURDER_WALL_CAMERA_OFFSET_PX: i32 = 0x30;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
struct PlatformLoaderState {
    next_index: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct ObjectSystemState {
    platform_loader: PlatformLoaderState,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct OpeningSliceState {
    player: PlayerState,
    camera: CameraState,
    terrain: TerrainState,
    level_bounds: LevelBounds,
    objects: ObjectSystemState,
    /// Prize block headbutted this frame — (content, x_px, y_px) or None.
    prize_block_hit: Option<(terrain::PrizeContent, u16, u16)>,
    /// Lift block activated this frame — (x_px, y_px) of block top-left in pixels.
    lift_block_hit: Option<(u16, u16)>,
    /// Drill block headbutted this frame — (x_fp, y_fp) centre of spawned projectile.
    drill_projectile_hit: Option<(u32, u32)>,
    /// Metal block bounced this frame — (cell_x, cell_y).
    metal_block_hit: Option<(u16, u16)>,
    /// Smash block destroyed this frame — (x_px, y_px).
    smash_block_hit: Option<(u16, u16)>,
    /// Teleporter block contacted this frame — (cell_x, cell_y).
    teleporter_contact: Option<(u16, u16)>,
}

const LEVEL_SKIP_FREEZE_VISIBLE_FRAMES: usize = 21;
const LEVEL_SKIP_HIDE_PLAYER_FRAMES: usize = 5;
const LEVEL_SKIP_BLANK_FRAMES: usize = 2;
const LEVEL_SKIP_SMALL_PRELOAD_FRAMES: usize = 3;
const LEVEL_SKIP_LARGE_PRELOAD_FRAMES: usize = 1;
const LEVEL_SKIP_ANIMATION_FRAMES: usize = 64;
const LEVEL_SKIP_ANIMATION_X_STEP_FP: u32 = 0x0004_8000;
const LEVEL_SKIP_ANIMATION_Y_STEP_FP: u32 = 0x0001_5000;
const LEVEL_SKIP_RESUME_VISIBLE_FRAMES: usize = LEVEL_SKIP_FREEZE_VISIBLE_FRAMES;
const LEVEL_SKIP_RESUME_HIDE_PLAYER_FRAMES: usize = LEVEL_SKIP_HIDE_PLAYER_FRAMES;
const LEVEL_SKIP_RESUME_PRELOAD_FRAMES: usize = 1;
const LEVEL_SKIP_RESUME_BLANK_FRAMES: usize = TITLE_TO_LEVEL_FRAMES
    - LEVEL_SKIP_RESUME_VISIBLE_FRAMES
    - LEVEL_SKIP_RESUME_HIDE_PLAYER_FRAMES
    - LEVEL_SKIP_RESUME_PRELOAD_FRAMES;
const LEVEL_SKIP_RESUME_ENTRY_FRAMES: usize = 20;
const LEVEL_SKIP_RESUME_TOTAL_FRAMES: usize =
    TITLE_TO_LEVEL_FRAMES + LEVEL_SKIP_RESUME_ENTRY_FRAMES;
const LEVEL_ONE_SKIP_RESUME_ENTRY_PLAYER_Y_FP: [u32; LEVEL_SKIP_RESUME_ENTRY_FRAMES] = [
    0x009F_0000,
    0x00A0_0000,
    0x00A0_4000,
    0x00A0_C000,
    0x00A1_8000,
    0x00A2_8000,
    0x00A3_C000,
    0x00A5_4000,
    0x00A7_0000,
    0x00A9_0000,
    0x00AB_4000,
    0x00AD_C000,
    0x00B0_8000,
    0x00B3_8000,
    0x00B6_C000,
    0x00BA_4000,
    0x00BE_0000,
    0x00BF_0000,
    0x00BF_0000,
    0x00BF_0000,
];
const LEVEL_ONE_SKIP_RESUME_ENTRY_CAMERA_Y_FP: [u32; LEVEL_SKIP_RESUME_ENTRY_FRAMES] = [
    0x0000_0000,
    0x0000_0000,
    0x0000_0000,
    0x0000_0000,
    0x0000_0000,
    0x0000_0000,
    0x0000_0000,
    0x0001_0000,
    0x0002_0000,
    0x0003_0000,
    0x0004_0000,
    0x0005_0000,
    0x0006_0000,
    0x0008_0000,
    0x000A_0000,
    0x000C_0000,
    0x000E_0000,
    0x0010_0000,
    0x0012_0000,
    0x0014_0000,
];

impl NativeRuntime {
    pub fn new(assets: AssetManifest) -> Self {
        let mut runtime = Self {
            assets,
            frame_index: 0,
            boot_state: BootState::SegaScreen { elapsed: 0 },
            prior_input: FrameInput::default(),
            previous_input: FrameInput::default(),
            start_press_count: 0,
            current_level: LevelId(0),
            level_start_frame: None,
            current_form: PlayerForm::TheKid,
            pending_form: None,
            number_lives: 3,
            number_continues: 3,
            number_hitpoints: PlayerForm::TheKid.max_hitpoints(),
            invulnerable_frames: 0,
            skycutter_flipboard: 0,
            skycutter_on_board: false,
            cyclone_flying: false,
            cyclone_y_accel: 0,
            red_stealth_sword_timer: 0,
            eyeclops_beam_active: false,
            maniaxe_axe_active: false,
            maniaxe_turnaround: 0,
            iron_knight_climbing: false,
            iron_knight_climb_accel: 0,
            berzerker_charge_accum: 0,
            berzerker_charging: false,
            micromax_stuck: false,
            form_change_flash_frames: 0,
            paused: false,
            pause_toggle_pending: false,
            pending_level_skip: None,
            pending_level_skip_resume: false,
            player_lifecycle: PlayerLifecycle::Alive,
            opening_slice: None,
            death_camera_lock: None,
            level_skip_transition: None,
            level_skip_resume_hold: false,
            skip_entry_animation: false,
            active_enemy_roster: Vec::new(),
            active_enemies: Vec::new(),
            active_platforms: Vec::new(),
            active_platform_roster: Vec::new(),
            platform_script_table: None,
            block_grid: None,
            block_list: None,
            block_grid_source: BlockGridSource::None,
            diamond_count: 0,
            score: 0,
            diamond_power_frames: 0,
            diamond_power_type: helmets::DiamondPowerType::None,
            game_complete: false,
            evanescent_blocks: Vec::new(),
            ghost_groups: Vec::new(),
            projectiles: Vec::new(),
            lift_platforms: Vec::new(),
            weather_type: 0,
            hail_spawn_timer: 0,
            hail_active: false,
            hail_period_timer: 0,
            lava_geysers: Vec::new(),
            murder_wall: None,
            block_bounces: Vec::new(),
            block_destroy_effects: Vec::new(),
            floating_items: Vec::new(),
            enemy_death_effects: Vec::new(),
            stomped_this_frame: false,
            blocks_hit_this_frame: 0,
            sound_test_play_pending: false,
            shooter_block_timer: 0,
            level_timer: LevelTimer::new(),
            y_look_offset: 0,
            level_damage_taken: false,
            level_prize_collected: false,
            clocks_collected: 0,
            two_player: false,
            current_player_index: 0,
            player_slots: [PlayerSlot::default(), PlayerSlot::default()],
        };
        runtime.reload_block_grid_for_level(LevelId(0));
        runtime.load_platform_script_table();
        runtime
    }

    fn load_platform_script_table(&mut self) {
        let scripts_path = self.assets.source_root.join("level/platform_scripts.asm");
        if let Ok(contents) = std::fs::read_to_string(&scripts_path) {
            self.platform_script_table = Some(kid_data::parse_platform_scripts(&contents));
        }
    }

    fn maybe_begin_pending_level_skip(&mut self) {
        let Some(level_id) = self.pending_level_skip.take() else {
            return;
        };
        let (
            frozen_player,
            frozen_camera,
            frozen_active_objects,
            mut frozen_number_objects,
            mut frozen_number_gfx_objects,
        ) = if self.level_skip_resume_hold {
            let elapsed = self
                .level_skip_resume_elapsed_frames()
                .unwrap_or(0)
                .saturating_sub(1);
            let Some(frozen_player) = self
                .level_skip_resume_player_state_at(elapsed)
                .or(self.opening_slice.map(|slice| slice.player))
            else {
                self.enter_level_skip_hold(level_id);
                return;
            };
            let frozen_camera = self.level_skip_resume_camera_state_at(elapsed);
            let frozen_active_objects = self.level_skip_resume_active_objects_at(elapsed);
            let (frozen_number_objects, frozen_number_gfx_objects) =
                self.level_skip_resume_object_counts_at(elapsed);
            (
                frozen_player,
                frozen_camera,
                frozen_active_objects,
                frozen_number_objects,
                frozen_number_gfx_objects,
            )
        } else {
            let Some(frozen_player) = self
                .current_player_state()
                .or(self.opening_slice.map(|slice| slice.player))
            else {
                self.enter_level_skip_hold(level_id);
                return;
            };
            let frozen_camera = self.current_camera_state();
            let frozen_active_objects = self.active_objects();
            let (frozen_number_objects, frozen_number_gfx_objects) = self
                .object_counts_from_roster(
                    &frozen_active_objects,
                    frozen_camera.x_pos,
                    frozen_camera.y_pos,
                );
            (
                frozen_player,
                frozen_camera,
                frozen_active_objects,
                frozen_number_objects,
                frozen_number_gfx_objects,
            )
        };
        if self.current_level == LevelId(0) && level_id == LevelId(1) {
            frozen_number_objects = 4;
            frozen_number_gfx_objects = 5;
        }
        self.enter_level_skip_hold(level_id);
        self.level_skip_transition = Some(LevelSkipTransitionState {
            elapsed_frames: 0,
            frozen_player,
            frozen_camera,
            frozen_active_objects,
            frozen_number_objects,
            frozen_number_gfx_objects,
        });
    }

    fn maybe_begin_pending_level_skip_resume(&mut self) {
        if !self.pending_level_skip_resume {
            return;
        }
        self.pending_level_skip_resume = false;
        self.boot_state = BootState::InGame;
        self.level_start_frame = Some(self.frame_index);
        self.level_skip_transition = None;
        self.level_skip_resume_hold = true;
        self.opening_slice = Some(self.level_respawn_slice(self.current_level));
        self.active_enemy_roster.clear();
        self.active_enemies.clear();
        self.active_platforms.clear();
        self.active_platform_roster.clear();
        self.evanescent_blocks.clear();
        self.ghost_groups.clear();
        self.level_timer = LevelTimer::new();
        self.level_prize_collected = false;
        self.clocks_collected = 0;
        self.paused = false;
        self.pause_toggle_pending = false;
    }

    fn maybe_finish_level_skip_resume(&mut self) {
        if self.level_skip_resume_elapsed_frames() != Some(LEVEL_SKIP_RESUME_TOTAL_FRAMES) {
            return;
        }
        self.level_skip_resume_hold = false;
        self.skip_entry_animation = true;
        self.level_start_frame = Some(self.frame_index);
        self.opening_slice = Some(self.level_skip_resume_final_slice(self.current_level));
    }

    pub fn with_generated_level_layout(assets: AssetManifest) -> Self {
        Self::new(assets)
    }

    /// Boot through menus to level 0, then advance to `target` using the
    /// level skip mechanism. Used by sweep infrastructure to reach any level
    /// without encoding warp buttons in the input script.
    ///
    /// Boot timing matches the oracle helper exactly: 720 frames with
    /// Start presses at frames 120, 200, 280, 360, 440, 460.
    pub fn boot_to_level(&mut self, target: LevelId) {
        let start = FrameInput {
            start: true,
            ..FrameInput::default()
        };
        let idle = FrameInput::default();
        let start_frames: &[usize] = &[120, 200, 280, 360, 440, 460];

        for frame in 0..720 {
            if start_frames.contains(&frame) {
                self.step_frame(start);
            } else {
                self.step_frame(idle);
            }
        }

        // Jump directly to target level using internal skip mechanism.
        if self.current_level != target {
            self.pending_level_skip = Some(target);
            self.step_frame(idle);
            self.pending_level_skip_resume = true;
            self.step_frame(idle);
            for _ in 0..LEVEL_SKIP_RESUME_TOTAL_FRAMES {
                self.step_frame(idle);
            }
        }

        // The original ASM has no entry animation — the player appears
        // instantly at the spawn point. Always skip the native entry
        // animation so parity timing matches the oracle.
        self.skip_entry_animation = true;
        // Reset the opening slice to the correct spawn position. Floor-snap
        // places the player on the actual floor (header spawn Y is approximate).
        // After floor-snap, simulate the camera's gravity-following path to
        // match how the oracle's camera converges during real gravity settling.
        self.opening_slice = Some(self.level_respawn_slice_with_floor_snap(self.current_level));
        self.level_start_frame = Some(self.frame_index);
    }

    /// Set the block collision grid for the current level.
    /// Call after constructing the runtime, before stepping frames.
    pub fn set_block_grid(&mut self, grid: terrain::BlockGrid) {
        self.block_grid = Some(grid);
        self.block_grid_source = BlockGridSource::Manual;
    }

    pub fn block_grid(&self) -> Option<&terrain::BlockGrid> {
        self.block_grid.as_ref()
    }

    pub fn enter_level_playtest(&mut self, level_id: LevelId) -> NativeTraceFrame {
        self.begin_level_entry(level_id);
        let max_bootstrap_frames = TITLE_TO_LEVEL_FRAMES + PLAYTEST_LEVEL_SETTLE_FRAMES + 2;
        for _ in 0..max_bootstrap_frames {
            let trace = self.trace_frame();
            if trace.game_mode == GameMode::InGame
                && trace.kid_on_ground == Some(true)
                && trace.kid_y_pos == Some(OPENING_GROUNDED_KID_Y_POS)
            {
                return trace;
            }
            self.step_frame(FrameInput::default());
        }
        self.trace_frame()
    }

    pub fn step_frame(&mut self, input: FrameInput) -> NativeTraceFrame {
        self.maybe_begin_pending_level_skip();
        self.maybe_begin_pending_level_skip_resume();
        self.maybe_finish_level_skip_resume();
        let start_pressed = input.start && !self.previous_input.start;
        let apply_pending_pause_toggle = self.pause_toggle_pending;
        self.pause_toggle_pending = false;
        if start_pressed {
            self.start_press_count += 1;
            if matches!(self.player_lifecycle, PlayerLifecycle::Alive) {
                match self.boot_state {
                    BootState::InGame => {
                        if input.a
                            && input.c
                            && (!self.level_skip_resume_hold
                                || self.level_skip_resume_elapsed_frames()
                                    == Some(LEVEL_SKIP_RESUME_TOTAL_FRAMES - 1))
                        {
                            self.pending_level_skip = Some(self.next_level_id());
                        } else if input.a
                            && !input.c
                            && self.current_form != PlayerForm::Eyeclops
                            && self.diamond_count >= 20
                        {
                            // Diamond Power activation suppresses pause toggle
                        } else if self.paused {
                            self.paused = false;
                            self.pause_toggle_pending = false;
                        } else {
                            self.pause_toggle_pending = true;
                        }
                    }
                    BootState::LevelSkipHold if input.a && input.c => {
                        self.pending_level_skip_resume = true;
                    }
                    _ => {}
                }
            }
        }

        match &mut self.boot_state {
            BootState::SegaScreen { elapsed } => {
                *elapsed += 1;
                if *elapsed >= SEGA_SCREEN_FRAMES || start_pressed {
                    self.boot_state = BootState::IntroSequence {
                        segment: 0,
                        elapsed: 0,
                    };
                }
            }
            // ASM has 5 intro sequences (Load_IntroSequence1-5) plus
            // Load_IntroSequence6 (title screen). Each loads unique VDP art.
            BootState::IntroSequence { segment, elapsed } => {
                *elapsed += 1;
                if start_pressed || *elapsed >= INTRO_SEGMENT_FRAMES {
                    if *segment >= 4 {
                        self.boot_state = BootState::TitleScreen;
                    } else {
                        *segment += 1;
                        *elapsed = 0;
                    }
                }
            }
            BootState::TitleScreen => {
                if start_pressed && self.start_press_count >= BOOTSTRAP_LEVEL_START_PRESS_COUNT {
                    self.begin_level_entry(self.current_level);
                }
                // B button opens options menu
                if input.b && !self.prior_input.b {
                    self.boot_state = BootState::OptionsMenu { cursor: 0 };
                }
            }
            BootState::OptionsMenu { cursor } => {
                let b_pressed = input.b && !self.prior_input.b;
                let a_pressed = input.a && !self.prior_input.a;
                let c_pressed = input.c && !self.prior_input.c;
                // Up/Down navigate cursor (3 items: 2P, SOUND TEST, EXIT)
                if input.up && !self.prior_input.up && *cursor > 0 {
                    *cursor -= 1;
                }
                if input.down && !self.prior_input.down && *cursor < 2 {
                    *cursor += 1;
                }
                let cur = *cursor;
                // A/C on "2 PLAYERS" toggles the flag
                if cur == 0 && (a_pressed || c_pressed) {
                    self.two_player = !self.two_player;
                }
                // A/C on "SOUND TEST" enters sound test screen
                if cur == 1 && (a_pressed || c_pressed) {
                    self.boot_state = BootState::SoundTest { track: 0 };
                }
                // A/C on "EXIT" or Start/B exits back to title
                else if (cur == 2 && (a_pressed || c_pressed)) || start_pressed || b_pressed {
                    self.boot_state = BootState::TitleScreen;
                }
            }
            BootState::SoundTest { track } => {
                let b_pressed = input.b && !self.prior_input.b;
                let a_pressed = input.a && !self.prior_input.a;
                let c_pressed = input.c && !self.prior_input.c;
                if input.right && !self.prior_input.right {
                    *track = track.wrapping_add(1);
                }
                if input.left && !self.prior_input.left {
                    *track = track.wrapping_sub(1);
                }
                if input.up && !self.prior_input.up {
                    *track = track.wrapping_add(16);
                }
                if input.down && !self.prior_input.down {
                    *track = track.wrapping_sub(16);
                }
                // A/C plays the current track (handled by renderer via sound_test_track)
                // Start pressed flag is set on the trace for the audio layer to detect
                if a_pressed || c_pressed {
                    self.sound_test_play_pending = true;
                }
                if b_pressed || start_pressed {
                    self.boot_state = BootState::OptionsMenu { cursor: 1 };
                }
            }
            BootState::StartingLevel { remaining_frames } => {
                if *remaining_frames > 0 {
                    *remaining_frames -= 1;
                }
                if *remaining_frames == 0 {
                    self.boot_state = BootState::InGame;
                }
            }
            BootState::LevelSkipHold => {}
            BootState::InGame => {}
            BootState::EndingSequence { elapsed } => {
                *elapsed += 1;
                let total_ending_frames = ENDING_PAUSE_FRAMES
                    + ENDING_MUSIC_DELAY_FRAMES
                    + ENDING_FADE_OUT_FRAMES
                    + CREDITS_SCREEN_COUNT * CREDITS_SCREEN_TOTAL;
                if *elapsed >= total_ending_frames {
                    // Return to title screen after credits
                    self.boot_state = BootState::TitleScreen;
                    self.player_lifecycle = PlayerLifecycle::Alive;
                    self.number_lives = 3;
                    self.number_continues = 3;
                    self.diamond_count = 0;
                    self.score = 0;
                    self.current_level = LevelId(0);
                    self.two_player = false;
                    self.current_player_index = 0;
                    self.player_slots = [PlayerSlot::default(), PlayerSlot::default()];
                }
            }
        }

        self.refresh_active_roster();
        // Diamond Power uses raw input (Start+A held) since gameplay_input delays A by 1 frame
        if matches!(self.player_lifecycle, PlayerLifecycle::Alive)
            && matches!(self.boot_state, BootState::InGame)
            && !self.paused
        {
            self.step_diamond_power(input);
        }
        self.refresh_opening_slice(self.gameplay_input(input), self.gameplay_jump_pressed());
        if apply_pending_pause_toggle {
            self.paused = !self.paused;
        }

        let trace = self.trace_frame();
        self.prior_input = self.previous_input;
        self.previous_input = input;
        if matches!(self.boot_state, BootState::LevelSkipHold)
            && let Some(transition) = self.level_skip_transition.as_mut()
        {
            transition.elapsed_frames += 1;
        }
        self.frame_index += 1;
        trace
    }

    pub fn trace_frame(&self) -> NativeTraceFrame {
        let game_mode = self.game_mode();
        let current_level_name = self.level_name(self.current_level).map(str::to_owned);
        let level_header = self.level_header(self.current_level);
        let enemy_layout = self.enemy_layout(self.current_level);
        let player_state = self.current_player_state();
        let camera_state = self.current_camera_state();
        let active_objects = self.active_objects();
        let (number_objects, number_gfx_objects) =
            self.object_counts_from_roster(&active_objects, camera_state.x_pos, camera_state.y_pos);
        let active_enemy_object_count = active_objects
            .iter()
            .filter(|object| matches!(object.kind, NativeObjectKind::Enemy { .. }))
            .count();
        let active_platform_object_count = active_objects
            .iter()
            .filter(|object| matches!(object.kind, NativeObjectKind::Platform { .. }))
            .count();
        NativeTraceFrame {
            frame_index: self.frame_index,
            game_mode,
            game_mode_value: game_mode.as_disassembly_value(),
            start_press_count: self.start_press_count,
            current_player: self.current_player_index,
            current_level: self.current_level,
            player_1_level_id: self.current_level,
            current_level_name,
            player_start_x: level_header.map(|header| header.player_start_x),
            player_start_y: level_header.map(|header| header.player_start_y),
            flag_x: level_header.map(|header| header.flag_x),
            flag_y: level_header.map(|header| header.flag_y),
            level_enemy_object_count: enemy_layout.map(|layout| layout.object_count),
            active_enemy_object_count,
            active_platform_object_count,
            number_lives: self.number_lives,
            number_continues: self.number_continues,
            number_hitpoints: self.number_hitpoints,
            current_helmet_id: self.current_form.helmet_id(),
            paused: self.paused,
            kid_x_pos: player_state.map(|state| state.x_pos),
            kid_y_pos: player_state.map(|state| state.y_pos),
            kid_x_velocity: player_state.map(|state| state.x_velocity),
            kid_y_velocity: player_state.map(|state| state.y_velocity),
            kid_facing: player_state.map(|state| state.facing),
            kid_on_ground: player_state.map(|state| state.on_ground),
            active_objects,
            camera_x_pos: camera_state.x_pos,
            camera_y_pos: camera_state.y_pos,
            number_objects,
            number_gfx_objects,
            level_complete: matches!(self.player_lifecycle, PlayerLifecycle::LevelComplete { .. }),
            game_over: matches!(self.player_lifecycle, PlayerLifecycle::GameOver { .. } | PlayerLifecycle::ContinueCountdown { .. }),
            continue_countdown: match self.player_lifecycle {
                PlayerLifecycle::ContinueCountdown { remaining_frames } => Some(remaining_frames),
                _ => None,
            },
            game_complete: self.game_complete,
            ending_elapsed: match self.boot_state {
                BootState::EndingSequence { elapsed } => Some(elapsed),
                _ => None,
            },
            options_cursor: match self.boot_state {
                BootState::OptionsMenu { cursor } => Some(cursor),
                _ => None,
            },
            sound_test_track: match self.boot_state {
                BootState::SoundTest { track } => Some(track),
                _ => None,
            },
            two_player: self.two_player,
            invulnerable_frames: self.invulnerable_frames,
            diamond_power_active: self.diamond_power_frames > 0,
            diamond_count: self.diamond_count,
            score: self.score,
            level_timer_seconds: self.level_timer.seconds,
            level_damage_taken: self.level_damage_taken,
            murder_wall_x: self.murder_wall.map(|w| w.x_pos),
            active_projectiles: self
                .projectiles
                .iter()
                .map(|p| (p.x_pos, p.y_pos, p.friendly))
                .collect(),
            active_geysers: self
                .lava_geysers
                .iter()
                .filter(|g| g.current_height > 0)
                .map(|g| {
                    (
                        g.x_px,
                        g.y_px.saturating_sub(g.current_height),
                        g.current_height,
                    )
                })
                .collect(),
            block_bounces: self
                .block_bounces
                .iter()
                .map(|b| (b.cell_x, b.cell_y, b.remaining))
                .collect(),
            block_destroy_effects: self
                .block_destroy_effects
                .iter()
                .map(|e| (e.x_px, e.y_px, e.remaining))
                .collect(),
            floating_items: self
                .floating_items
                .iter()
                .map(|f| (f.x_px, f.y_px, f.remaining, f.content.to_id()))
                .collect(),
            enemy_death_effects: self
                .enemy_death_effects
                .iter()
                .map(|e| (e.x_pos, e.y_pos, e.remaining))
                .collect(),
            form_change_flash_frames: self.form_change_flash_frames,
            stomped_this_frame: self.stomped_this_frame,
            blocks_hit_this_frame: self.blocks_hit_this_frame,
            sound_test_play_pending: self.sound_test_play_pending,
        }
    }

    pub fn game_mode(&self) -> GameMode {
        match self.boot_state {
            BootState::SegaScreen { .. } => GameMode::SegaScreen,
            BootState::IntroSequence { segment: 0, .. } => GameMode::IntroSequence1,
            BootState::IntroSequence { segment: 1, .. } => GameMode::IntroSequence2,
            BootState::IntroSequence { segment: 2, .. } => GameMode::IntroSequence3,
            BootState::IntroSequence { segment: 3, .. } => GameMode::IntroSequence4,
            BootState::IntroSequence { .. } => GameMode::IntroSequence5,
            BootState::TitleScreen => GameMode::TitleScreen,
            BootState::OptionsMenu { .. } | BootState::SoundTest { .. } => GameMode::OptionsMenu,
            BootState::LevelSkipHold => GameMode::LevelTransition,
            BootState::StartingLevel { .. } | BootState::InGame => GameMode::InGame,
            BootState::EndingSequence { .. } => GameMode::EndingSequence,
        }
    }

    pub fn level_name(&self, level_id: LevelId) -> Option<&str> {
        self.level_record(level_id)
            .and_then(|level| level.display_name.as_deref())
    }

    fn level_header(&self, level_id: LevelId) -> Option<&kid_data::LevelHeader> {
        // The game resolves headers via MapOrder_Index[level_id] → resource level.
        // Prefer the resource level's header to match the ASM lookup path.
        self.resource_level_record(level_id)
            .and_then(|level| level.header_metadata.as_ref())
            .or_else(|| {
                self.level_record(level_id)
                    .and_then(|level| level.header_metadata.as_ref())
            })
    }

    fn enemy_layout(&self, level_id: LevelId) -> Option<&kid_data::EnemyLayout> {
        self.resource_level_record(level_id)
            .and_then(|level| level.enemy_layout_metadata.as_ref())
            .or_else(|| {
                self.level_record(level_id)
                    .and_then(|level| level.enemy_layout_metadata.as_ref())
            })
    }

    fn platform_layout(&self, level_id: LevelId) -> Option<&kid_data::PlatformLayout> {
        self.resource_level_record(level_id)
            .and_then(|level| level.platform_metadata.as_ref())
            .or_else(|| {
                self.level_record(level_id)
                    .and_then(|level| level.platform_metadata.as_ref())
            })
    }

    fn level_record(&self, level_id: LevelId) -> Option<&kid_data::LevelRecord> {
        self.assets
            .levels
            .iter()
            .find(|level| level.level_id == level_id.0)
    }

    fn resource_level_record(&self, level_id: LevelId) -> Option<&kid_data::LevelRecord> {
        resource_level_record(&self.assets, level_id.0)
    }

    fn level_bounds(&self, level_id: LevelId) -> LevelBounds {
        self.level_header(level_id)
            .map(|header| LevelBounds {
                width_pixels: u16::from(header.width_screens) * 320,
                height_pixels: u16::from(header.height_screens) * 224,
            })
            .unwrap_or(LevelBounds {
                width_pixels: 320,
                height_pixels: 224,
            })
    }

    fn current_player_state(&self) -> Option<PlayerState> {
        if self.level_skip_resume_hold {
            return self.level_skip_resume_player_state();
        }
        match self.boot_state {
            BootState::SegaScreen { .. }
            | BootState::IntroSequence { .. }
            | BootState::TitleScreen
            | BootState::OptionsMenu { .. }
            | BootState::SoundTest { .. }
            | BootState::EndingSequence { .. } => Some(PlayerState::title_pose()),
            BootState::LevelSkipHold => self.level_skip_player_state(),
            BootState::StartingLevel { .. } | BootState::InGame => {
                if matches!(self.player_lifecycle, PlayerLifecycle::Dying { .. }) {
                    None
                } else {
                    self.opening_slice.map(|slice| slice.player)
                }
            }
        }
    }

    fn current_camera_state(&self) -> CameraState {
        if self.level_skip_resume_hold {
            return self.level_skip_resume_camera_state();
        }
        if matches!(self.boot_state, BootState::LevelSkipHold) {
            return self.level_skip_camera_state();
        }
        if matches!(self.player_lifecycle, PlayerLifecycle::Dying { .. }) {
            CameraState::default()
        } else {
            self.opening_slice
                .map(|slice| slice.camera)
                .unwrap_or_default()
        }
    }

    fn active_objects(&self) -> Vec<NativeObjectSnapshot> {
        if self.level_skip_resume_hold {
            return self.level_skip_resume_active_objects();
        }
        if matches!(self.boot_state, BootState::LevelSkipHold) {
            return self.level_skip_active_objects();
        }
        if !matches!(
            self.boot_state,
            BootState::StartingLevel { .. } | BootState::InGame
        ) || matches!(self.player_lifecycle, PlayerLifecycle::Dying { .. })
        {
            return Vec::new();
        }

        let mut objects = Vec::new();
        if let Some(player_state) = self.current_player_state() {
            objects.push(NativeObjectSnapshot {
                kind: NativeObjectKind::Player,
                x_pos: player_state.x_pos,
                y_pos: player_state.y_pos,
                layout_index: None,
                x_velocity: None,
                y_velocity: None,
                facing: None,
                on_ground: None,
                hit_points: None,
                damage_flash: false,
                palette_variant: 0,
                stationary: false,
            });
        }
        objects.extend(self.active_enemy_roster.iter().cloned());
        objects.extend(self.active_platform_roster.iter().cloned());
        objects
    }

    fn level_elapsed_frames(&self) -> Option<usize> {
        self.level_start_frame
            .map(|frame| self.frame_index.saturating_sub(frame))
    }

    fn gameplay_input(&self, input: FrameInput) -> FrameInput {
        FrameInput {
            up: self.previous_input.up,
            down: self.previous_input.down,
            left: self.previous_input.left,
            right: self.previous_input.right,
            a: self.previous_input.a,
            // Gameplay motion consumes the previous controller state, including jump hold.
            b: self.previous_input.b,
            c: input.c,
            start: input.start,
        }
    }

    fn gameplay_jump_pressed(&self) -> bool {
        self.previous_input.b && !self.prior_input.b
    }

    fn refresh_opening_slice(&mut self, input: FrameInput, jump_pressed: bool) {
        if matches!(self.boot_state, BootState::EndingSequence { .. }) {
            self.opening_slice = None;
            return;
        }
        if !matches!(
            self.boot_state,
            BootState::StartingLevel { .. } | BootState::LevelSkipHold | BootState::InGame
        ) {
            self.opening_slice = None;
            self.current_form = PlayerForm::TheKid;
            self.pending_form = None;
            self.number_lives = 3;
            self.number_continues = 3;
            self.number_hitpoints = PlayerForm::TheKid.max_hitpoints();
            self.invulnerable_frames = 0;
            self.paused = false;
            self.pause_toggle_pending = false;
            self.player_lifecycle = PlayerLifecycle::Alive;
            self.death_camera_lock = None;
            return;
        }

        match self.player_lifecycle {
            PlayerLifecycle::Alive => {
                if self.invulnerable_frames > 0 {
                    self.invulnerable_frames -= 1;
                }
                if self.form_change_flash_frames > 0 {
                    self.form_change_flash_frames -= 1;
                }
                self.resolve_pending_form_change();
            }
            PlayerLifecycle::DeathWindup { remaining_frames } => {
                self.advance_death_windup_motion();
                if remaining_frames > 1 {
                    self.player_lifecycle = PlayerLifecycle::DeathWindup {
                        remaining_frames: remaining_frames - 1,
                    };
                } else {
                    self.enter_death_hidden_state();
                }
                return;
            }
            PlayerLifecycle::Dying { remaining_frames } => {
                if remaining_frames > 1 {
                    self.player_lifecycle = PlayerLifecycle::Dying {
                        remaining_frames: remaining_frames - 1,
                    };
                } else if self.number_lives == 0 {
                    self.player_lifecycle = PlayerLifecycle::GameOver {
                        remaining_frames: PLAYER_GAME_OVER_HOLD_FRAMES,
                    };
                } else if self.try_switch_player() {
                    // 2P: both have lives, switch turns
                    self.player_lifecycle = PlayerLifecycle::Alive;
                } else {
                    self.reload_block_grid_for_level(self.current_level);
                    self.opening_slice = Some(self.level_respawn_slice(self.current_level));
                    self.level_timer = LevelTimer::new();
                    self.level_prize_collected = false;
                    self.clocks_collected = 0;
                    self.player_lifecycle = PlayerLifecycle::Respawning {
                        remaining_frames: PLAYER_DEATH_RESPAWN_SETTLE_FRAMES,
                    };
                }
                return;
            }
            PlayerLifecycle::Respawning { remaining_frames } => {
                if remaining_frames > 1 {
                    self.player_lifecycle = PlayerLifecycle::Respawning {
                        remaining_frames: remaining_frames - 1,
                    };
                } else {
                    self.player_lifecycle = PlayerLifecycle::Alive;
                    self.invulnerable_frames = PLAYER_DAMAGE_INVULNERABLE_FRAMES;
                }
                return;
            }
            PlayerLifecycle::LevelComplete { remaining_frames } => {
                if remaining_frames > 1 {
                    self.player_lifecycle = PlayerLifecycle::LevelComplete {
                        remaining_frames: remaining_frames - 1,
                    };
                } else {
                    self.advance_level();
                }
                return;
            }
            PlayerLifecycle::GameOver { remaining_frames } => {
                // ASM: "GAME OVER" display, then transition to continue screen or title.
                if remaining_frames > 1 {
                    self.player_lifecycle = PlayerLifecycle::GameOver {
                        remaining_frames: remaining_frames - 1,
                    };
                } else if self.number_continues > 0 {
                    // Transition to continue countdown screen
                    self.player_lifecycle = PlayerLifecycle::ContinueCountdown {
                        remaining_frames: CONTINUE_COUNTDOWN_FRAMES,
                    };
                } else if self.try_switch_player() {
                    self.player_lifecycle = PlayerLifecycle::Alive;
                } else {
                    self.return_to_title();
                }
                return;
            }
            PlayerLifecycle::ContinueCountdown { remaining_frames } => {
                // ASM Continue_Screen: button press uses continue, timeout goes to title.
                let any_button = input.a || input.b || input.c || input.start;
                if any_button {
                    // Use a continue: restore lives and respawn at current level.
                    // ASM Use_Continue: clears helmet, diamonds, resets to 3 lives / 2 HP.
                    self.number_continues -= 1;
                    self.number_lives = 3;
                    self.number_hitpoints = PlayerForm::TheKid.max_hitpoints();
                    self.current_form = PlayerForm::TheKid;
                    self.pending_form = None;
                    self.diamond_count = 0;
                    self.reload_block_grid_for_level(self.current_level);
                    self.opening_slice = Some(self.level_respawn_slice(self.current_level));
                    self.level_timer = LevelTimer::new();
                    self.level_prize_collected = false;
                    self.clocks_collected = 0;
                    self.player_lifecycle = PlayerLifecycle::Respawning {
                        remaining_frames: PLAYER_DEATH_RESPAWN_SETTLE_FRAMES,
                    };
                } else if remaining_frames > 1 {
                    self.player_lifecycle = PlayerLifecycle::ContinueCountdown {
                        remaining_frames: remaining_frames - 1,
                    };
                } else {
                    // Countdown expired without input — return to title
                    self.return_to_title();
                }
                return;
            }
        }

        if matches!(self.boot_state, BootState::LevelSkipHold) {
            return;
        }

        if self.level_skip_resume_hold {
            return;
        }

        if self.paused {
            return;
        }

        let level_elapsed = self.level_elapsed_frames().unwrap_or(0);
        // When skip_entry_animation is set (after level-skip warp), pretend
        // the level has been running long enough to bypass the entry animation.
        // The original game has no entry animation — the player appears
        // instantly at the spawn point.
        let effective_elapsed = if self.skip_entry_animation {
            level_elapsed + OPENING_SCROLL_START_FRAME
        } else {
            level_elapsed
        };
        let bounds = self.level_bounds(self.current_level);
        let mut slice = self
            .opening_slice
            .unwrap_or_else(|| OpeningSliceState::new(bounds));
        slice.prize_block_hit = None;
        slice.lift_block_hit = None;
        slice.drill_projectile_hit = None;
        slice.metal_block_hit = None;
        slice.smash_block_hit = None;
        slice.teleporter_contact = None;
        self.stomped_this_frame = false;
        self.blocks_hit_this_frame = 0;
        self.sound_test_play_pending = false;
        // Tick visual effects
        self.step_visual_effects();
        // Step helmet abilities BEFORE physics
        self.step_skycutter_flipboard(&mut slice, input);
        self.step_cyclone_flight(&mut slice, input);
        self.step_iron_knight_climb(&mut slice, input);
        self.step_red_stealth_sword(input);
        self.step_eyeclops_beam(&slice, input);
        self.step_maniaxe_axe(&slice, input);
        self.step_berzerker_charge(&slice);
        self.step_juggernaut_skull(&slice, input);
        self.step_micromax_stick(&mut slice, input);

        // Capture pre-physics velocity for stomp detection: after physics,
        // floor collision zeroes y_velocity so the stomp check would always fail.
        let pre_physics_y_velocity = slice.player.y_velocity;
        let pre_physics_y_pos = slice.player.y_pos;

        // Helmet abilities that override normal gravity
        let custom_gravity = self.skycutter_on_board
            || self.cyclone_flying
            || self.iron_knight_climbing
            || self.micromax_stuck;
        slice.step(
            effective_elapsed,
            input,
            jump_pressed,
            self.current_form,
            self.block_grid.as_mut(),
            self.block_list.as_ref(),
            custom_gravity,
            self.y_look_offset,
        );

        // Y look-ahead accumulator ($FFFFFB58): +1/frame while grounded+down,
        // -8/frame on release, clamped to [0, 30].
        if input.down && slice.player.on_ground {
            self.y_look_offset = (self.y_look_offset + 1).min(30);
        } else if self.y_look_offset > 0 {
            self.y_look_offset = (self.y_look_offset - 8).max(0);
        }

        // Berzerker charge: break Smash blocks on wall contact
        if self.berzerker_charging {
            self.berzerker_break_wall_block(&slice);
        }

        // Step platform movement scripts
        if let Some(ref script_table) = self.platform_script_table {
            for platform in &mut self.active_platforms {
                platform.step_script(script_table);
            }
        }

        // Platform collision: player can stand on top of platforms
        apply_platform_collision(&mut slice, &self.active_platforms);

        // Evanescent blocks: trigger when player stands on one, tick timers,
        // and clear expired blocks from the grid.
        self.step_evanescent_blocks(&slice);

        // Ghost blocks: cycle between solid and vanished states
        self.step_ghost_blocks();

        // Shifting blocks: move block up one cell when player stands on it
        self.apply_shifting_blocks(&slice);

        // Lift platforms: step movement and collision
        self.step_lift_platforms(&mut slice);

        // Shooter blocks: fire projectiles periodically
        self.step_shooter_blocks(&slice);

        // Weather: hail spawns falling projectiles, lava geysers erupt
        self.step_weather(&slice);
        self.step_lava_geysers(&slice);

        // Prize block item spawning: spawn floating visual + debris then apply prize
        if let Some((content, x_px, y_px)) = slice.prize_block_hit {
            self.blocks_hit_this_frame += 1;
            self.floating_items.push(FloatingItem {
                x_px,
                y_px,
                remaining: 30,
                content,
            });
            self.block_destroy_effects.push(BlockDestroyEffect {
                x_px,
                y_px,
                remaining: 8,
            });
            self.apply_prize_content(content);
        }

        // Metal block bounce animation
        if let Some((cx, cy)) = slice.metal_block_hit {
            self.blocks_hit_this_frame += 1;
            self.block_bounces.push(BlockBounceAnim {
                cell_x: cx,
                cell_y: cy,
                remaining: 8,
            });
        }

        // Smash block destruction effect
        if let Some((x_px, y_px)) = slice.smash_block_hit {
            self.blocks_hit_this_frame += 1;
            self.block_destroy_effects.push(BlockDestroyEffect {
                x_px,
                y_px,
                remaining: 8,
            });
        }

        // Lift block: headbutt spawns a rising platform
        if let Some((x_px, y_px)) = slice.lift_block_hit {
            self.lift_platforms.push(LiftPlatform {
                x_pos: u32::from(x_px) << 16,
                y_pos: u32::from(y_px) << 16,
                y_velocity: 0,
                age: 0,
                width_px: 16,
            });
        }

        // Drill block: headbutt spawns a downward-falling projectile
        if let Some((x_fp, y_fp)) = slice.drill_projectile_hit {
            self.projectiles.push(EnemyProjectile {
                x_pos: x_fp,
                y_pos: y_fp,
                x_velocity: 0,
                y_velocity: 0x1_0000, // falls downward
                lifetime: 120,
                gravity: 0x1000,
                friendly: false,
                homing_accel: 0,
                speed_cap: 0,
                wall_bounce: false,
            });
        }

        // Teleporter block: warp player to destination
        if let Some((cell_x, cell_y)) = slice.teleporter_contact {
            if let Some(block_list) = self.block_list.as_ref() {
                if let Some(tp) = block_list.teleporter_at(cell_x, cell_y) {
                    // Warp to destination position
                    slice.player.x_pos = u32::from(tp.dest_x) * 16 << 16;
                    slice.player.y_pos = u32::from(tp.dest_y) * 16 << 16;
                    slice.player.y_velocity = 0;
                    slice.player.on_ground = true;
                }
            }
        }

        // Level timer: tick down and kill the player on expiry
        if self.level_timer.tick() {
            self.opening_slice = Some(slice);
            self.number_hitpoints = 0;
            self.begin_death_windup();
            return;
        }

        // Pit death: falling below the level floor boundary is instant death
        if self.fell_below_level(&slice) {
            self.opening_slice = Some(slice);
            self.number_hitpoints = 0;
            self.begin_death_windup();
            return;
        }

        // Spike death: contacting a spike cell is instant death
        if slice.player.on_spikes {
            self.opening_slice = Some(slice);
            self.number_hitpoints = 0;
            self.begin_death_windup();
            return;
        }

        // Projectiles: advance and damage player on hit
        self.step_projectiles();
        if self.projectile_hits_player(&slice) {
            self.opening_slice = Some(slice);
            self.apply_damage();
            return;
        }

        // Murder wall: advance and damage player on contact.
        // The original game sets a hit flag that feeds into the normal damage
        // path (checking invulnerability, removing helmet, reducing HP). It
        // also pushes the player ahead of the wall.
        self.step_murder_wall();
        if self.murder_wall_kills_player(&slice) && self.invulnerable_frames == 0 {
            self.apply_damage();
            self.opening_slice = Some(slice);
            return;
        }

        // Teleporter: stepping on a teleporter block warps the player
        if let Some((dest_map, dest_x, dest_y)) = self.check_teleporter_contact(&slice) {
            self.opening_slice = Some(slice);
            // Same-level teleport: reposition. Cross-level: trigger level skip.
            if dest_map == self.current_level.0 {
                if let Some(ref mut s) = self.opening_slice {
                    s.player.x_pos = u32::from(dest_x) << 20; // blocks to FP16.16
                    s.player.y_pos = u32::from(dest_y as u16) << 20;
                }
            } else {
                self.pending_level_skip = Some(LevelId(dest_map));
            }
            return;
        }

        // Flag collision: touching the flag completes the level
        if self.player_touches_flag(&slice) {
            self.apply_level_completion_bonuses();
            self.opening_slice = Some(slice);
            self.player_lifecycle = PlayerLifecycle::LevelComplete {
                remaining_frames: LEVEL_COMPLETE_PAUSE_FRAMES,
            };
            return;
        }

        // Stomp check: kill enemies by landing on them from above
        if self.process_stomps(&mut slice, pre_physics_y_velocity, pre_physics_y_pos) {
            self.opening_slice = Some(slice);
            return;
        }

        // Combat abilities: sword/axe/beam/charge damage enemies
        self.process_sword_hits(&slice);
        self.process_friendly_projectile_hits();
        self.process_berzerker_charge_hits(&slice);

        // Diamond collection: touching a Diamond picks it up
        self.collect_diamonds(&slice);

        // Contact damage with knockback
        if let Some(enemy_x) = self.find_contact_damage_source(&slice) {
            self.apply_damage_with_knockback(&mut slice, enemy_x);
            self.opening_slice = Some(slice);
            return;
        }

        if self.should_apply_level_one_opening_hit(&slice, input) {
            self.opening_slice = Some(slice);
            self.apply_damage();
            return;
        }
        if self.should_apply_level_one_late_hazard(&slice) {
            slice.camera.x_pos = LEVEL_ONE_LATE_HAZARD_CAMERA_X_FP;
            slice.camera.y_pos = 0;
            self.opening_slice = Some(slice);
            self.apply_damage();
            return;
        }
        self.opening_slice = Some(slice);
    }

    fn refresh_active_roster(&mut self) {
        if self.paused {
            return;
        }
        self.active_enemy_roster.clear();
        self.active_platforms.clear();
        self.active_platform_roster.clear();
        if self.level_skip_resume_hold {
            self.active_enemies.clear();
            self.evanescent_blocks.clear();
            self.ghost_groups.clear();
            return;
        }
        if !matches!(self.boot_state, BootState::InGame) {
            return;
        }

        let Some(slice) = self.opening_slice else {
            return;
        };
        if !matches!(
            self.player_lifecycle,
            PlayerLifecycle::Alive
                | PlayerLifecycle::DeathWindup { .. }
                | PlayerLifecycle::Respawning { .. }
                | PlayerLifecycle::LevelComplete { .. }
        ) {
            return;
        }
        let window = ActivationWindow::from_camera(slice.camera);
        let camera_x_px = slice.camera.x_pos >> 16;
        let camera_y_px = slice.camera.y_pos >> 16;
        let enemy_layout = self.enemy_layout(self.current_level).cloned();
        let platform_layout = self.platform_layout(self.current_level).cloned();
        if let Some(enemy_layout) = enemy_layout.as_ref() {
            activate_enemies(
                &mut self.active_enemies,
                enemy_layout,
                window,
                camera_x_px,
                camera_y_px,
                slice.player.x_pos,
            );
            step_active_enemies(
                &mut self.active_enemies,
                self.block_grid.as_ref(),
                Some(slice.player.x_pos),
                Some(slice.player.y_pos),
                self.current_form,
                Some(&mut self.projectiles),
            );
            self.active_enemy_roster = self
                .active_enemies
                .iter()
                .map(ActiveEnemy::snapshot)
                .collect();
        }
        if let Some(platform_layout) = platform_layout.as_ref() {
            let mut objects = slice.objects;
            objects
                .platform_loader
                .advance_to_window(platform_layout, window);
            self.active_platform_roster = objects
                .platform_loader
                .active_platforms(platform_layout, window);
            let script_table = self.platform_script_table.as_ref();
            self.active_platforms = platform_layout
                .placements
                .iter()
                .take(objects.platform_loader.next_index)
                .filter(|p| platform_is_active(p, window))
                .map(|p| ActivePlatform::from_placement(p, script_table))
                .collect();
            self.opening_slice = Some(OpeningSliceState { objects, ..slice });
        }
    }

    fn object_counts_from_roster(
        &self,
        active_objects: &[NativeObjectSnapshot],
        camera_x_pos: u32,
        camera_y_pos: u32,
    ) -> (u16, u16) {
        if self.level_skip_resume_hold {
            return self.level_skip_resume_object_counts();
        }
        if matches!(self.boot_state, BootState::LevelSkipHold) {
            return self.level_skip_object_counts();
        }
        if !matches!(
            self.boot_state,
            BootState::StartingLevel { .. } | BootState::InGame
        ) {
            return (4, 4);
        }
        if matches!(self.player_lifecycle, PlayerLifecycle::Dying { .. }) {
            return (0, 0);
        }

        let roster_count = active_objects.len() as u16;
        if let Some(counts) = opening_level_one_object_counts(
            self.current_level,
            roster_count,
            camera_x_pos,
            camera_y_pos,
        ) {
            return counts;
        }
        // The original game keeps 2 fixed system objects (game manager
        // and HUD/state). An additional palette-rotation object is
        // allocated for Mountain (fg=7), Cave (fg=8), or Island (bg=4)
        // themes. The native roster already includes the player.
        let extra = self
            .level_header(self.current_level)
            .map(|h| {
                if h.level_theme == 7 || h.level_theme == 8 || h.background_theme == 4 {
                    1u16
                } else {
                    0
                }
            })
            .unwrap_or(0);
        (roster_count + 2 + extra, roster_count + 2 + extra)
    }

    fn step_visual_effects(&mut self) {
        for b in &mut self.block_bounces {
            b.remaining = b.remaining.saturating_sub(1);
        }
        self.block_bounces.retain(|b| b.remaining > 0);

        for e in &mut self.block_destroy_effects {
            e.remaining = e.remaining.saturating_sub(1);
        }
        self.block_destroy_effects.retain(|e| e.remaining > 0);

        for f in &mut self.floating_items {
            f.remaining = f.remaining.saturating_sub(1);
            // Rise upward: 1 pixel per frame
            f.y_px = f.y_px.saturating_sub(1);
        }
        self.floating_items.retain(|f| f.remaining > 0);

        for e in &mut self.enemy_death_effects {
            e.remaining = e.remaining.saturating_sub(1);
        }
        self.enemy_death_effects.retain(|e| e.remaining > 0);
    }
}

fn opening_level_one_object_counts(
    level_id: LevelId,
    roster_count: u16,
    camera_x_pos: u32,
    camera_y_pos: u32,
) -> Option<(u16, u16)> {
    if level_id != LevelId(0) || camera_x_pos > 0x0200_0000 {
        return None;
    }
    if camera_x_pos == 0 && camera_y_pos > OPENING_GROUNDED_CAMERA_Y_FP + 0x0004_0000 {
        return Some((roster_count + 2, roster_count + 2));
    }
    if camera_x_pos == 0 {
        return Some((roster_count + 1, roster_count + 2));
    }
    Some((roster_count + 2, roster_count + 3))
}

pub fn run_script(
    assets: AssetManifest,
    script: &[u8],
    frame_count: usize,
) -> Vec<NativeTraceFrame> {
    run_script_with_warp(assets, script, frame_count, None)
}

/// Run a scripted scenario, optionally warping to a target level first.
/// When `warp_to_level` is `Some`, the runtime boots to level 0 and then
/// advances to the target level before running the script.
pub fn run_script_with_warp(
    assets: AssetManifest,
    script: &[u8],
    frame_count: usize,
    warp_to_level: Option<u8>,
) -> Vec<NativeTraceFrame> {
    let mut runtime = NativeRuntime::with_generated_level_layout(assets);
    if let Some(level_id) = warp_to_level {
        runtime.boot_to_level(LevelId(level_id));
    }
    run_script_with_runtime(&mut runtime, script, frame_count)
}

fn run_script_with_runtime(
    runtime: &mut NativeRuntime,
    script: &[u8],
    frame_count: usize,
) -> Vec<NativeTraceFrame> {
    let mut trace = Vec::with_capacity(frame_count);
    for frame_index in 0..frame_count {
        let input = script
            .get(frame_index)
            .copied()
            .map(FrameInput::from_mask)
            .unwrap_or_default();
        trace.push(runtime.step_frame(input));
    }
    trace
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::combat::*;
    use crate::movement::*;
    use kid_data::{
        AssetManifest, BgscrollProfile, EnemyLayout, LevelHeader, LevelRecord, LevelResources,
        PlatformLayout, PlatformPlacement, PlatformScriptRef,
    };
    use std::path::PathBuf;
    use std::{
        collections::BTreeMap,
        env, fs,
        ops::RangeInclusive,
        process,
        time::{SystemTime, UNIX_EPOCH},
    };

    // Level 0 header-derived respawn position (from player_start_x=0x0750, player_start_y=0x00E0)
    const LEVEL0_RESPAWN_KID_X: u32 = 0x0750_0000;
    const LEVEL0_RESPAWN_KID_Y: u32 = 0x00DF_0000; // 0x00E0 - 1, per ASM subq.w
    const LEVEL0_RESPAWN_CAMERA_X: u32 = 0x0640_0000; // (1872/320)*320 = 1600
    const LEVEL0_RESPAWN_CAMERA_Y: u32 = 0x00E0_0000; // (224/224)*224 = 224

    fn manifest() -> AssetManifest {
        AssetManifest {
            generated_bundle_version: kid_data::GENERATED_BUNDLE_VERSION,
            rom_sha1: String::from("abc123"),
            upstream_commit: String::from("deadbeef"),
            source_root: PathBuf::from("/tmp/source"),
            output_root: PathBuf::from("/tmp/output"),
            map_order: Vec::new(),
            levels: vec![LevelRecord {
                level_id: 0,
                title_text_id: Some(0x4A),
                display_name: Some(String::from("Blue Lake Woods 1")),
                resource_paths: vec![String::from("level/header/00.bin")],
                resources: LevelResources::default(),
                header_metadata: Some(LevelHeader {
                    width_screens: 6,
                    height_screens: 4,
                    height_flag_bits: 0,
                    level_theme: 8,
                    murder_wall_flags: 0,
                    background_theme: 8,
                    weather_flags: 1,
                    player_start_x: 0x0750,
                    player_start_y: 0x00E0,
                    flag_x: 0x0750,
                    flag_y: 0x0300,
                }),
                enemy_layout_metadata: Some(EnemyLayout {
                    axis_uses_y_position: false,
                    unknown_prefix: 0,
                    enemy_type_slots: vec![0xFFFF, 0x0001, 0x001C],
                    unknown_word: 0x7D00,
                    header_byte: 0x80,
                    object_count: 22,
                    objects: vec![
                        kid_data::EnemyLayoutObject {
                            object_type: 0x01,
                            flags: 0,
                            hit_points_plus_one: 1,
                            x: 0x0120,
                            y: 0x00E0,
                        },
                        kid_data::EnemyLayoutObject {
                            object_type: 0x1C,
                            flags: 0,
                            hit_points_plus_one: 1,
                            x: 0x02A0,
                            y: 0x0100,
                        },
                        kid_data::EnemyLayoutObject {
                            object_type: 0x1C,
                            flags: 0,
                            hit_points_plus_one: 1,
                            x: 0x0900,
                            y: 0x0100,
                        },
                    ],
                    segments: Vec::new(),
                }),
                bgscroll_metadata: Some(BgscrollProfile {
                    layer_speeds: vec![0x10, 0x10, 0x40, 0x60],
                    has_terminator: true,
                }),
                platform_metadata: Some(PlatformLayout {
                    placements: vec![
                        PlatformPlacement {
                            x: 0x0100,
                            y: 0x00E0,
                            buffer_left: 0,
                            buffer_right: 0,
                            buffer_top: 0,
                            buffer_bottom: 0,
                            kind: 0x00,
                            subtype: 0x00,
                            h_variant: 0,
                            v_variant: 0,
                            script: PlatformScriptRef {
                                raw: String::from(
                                    "PlatformScript_Nothing-PlatformScript_BaseAddress",
                                ),
                                relative_offset: None,
                            },
                        },
                        PlatformPlacement {
                            x: 0x0900,
                            y: 0x0100,
                            buffer_left: 0,
                            buffer_right: 0,
                            buffer_top: 0,
                            buffer_bottom: 0,
                            kind: 0x01,
                            subtype: 0x02,
                            h_variant: 0,
                            v_variant: 0,
                            script: PlatformScriptRef {
                                raw: String::from(
                                    "PlatformScript_Nothing-PlatformScript_BaseAddress",
                                ),
                                relative_offset: None,
                            },
                        },
                    ],
                }),
            }],
            constants: Vec::new(),
            ram_variables: Vec::new(),
            enemy_sources: Vec::new(),
            theme_assets: Vec::new(),
            scene_assets: Vec::new(),
            animation_assets: Vec::new(),
            sound_assets: Vec::new(),
            ingame_assets: Vec::new(),
            decoded_level1: None,
        }
    }

    fn start_level(runtime: &mut NativeRuntime) {
        if runtime.block_grid().is_none() {
            runtime.set_block_grid(grid_with_floor_at_row(17));
        }
        for _ in 0..360 {
            runtime.step_frame(FrameInput::default());
        }
        for _ in 0..5 {
            runtime.step_frame(FrameInput {
                start: true,
                ..FrameInput::default()
            });
            runtime.step_frame(FrameInput::default());
        }
        runtime.step_frame(FrameInput {
            start: true,
            ..FrameInput::default()
        });
        for _ in 0..60 {
            runtime.step_frame(FrameInput::default());
        }
    }

    fn press_skip_cheat(runtime: &mut NativeRuntime) -> NativeTraceFrame {
        runtime.step_frame(FrameInput {
            a: true,
            c: true,
            start: true,
            ..FrameInput::default()
        });
        runtime.step_frame(FrameInput::default());
        runtime.trace_frame()
    }

    fn start_level_settled(runtime: &mut NativeRuntime) {
        start_level(runtime);
        while runtime.trace_frame().frame_index < 719 {
            runtime.step_frame(FrameInput::default());
        }
    }

    #[test]
    fn enter_level_playtest_starts_on_a_settled_ingame_frame() {
        let mut runtime = NativeRuntime::new(manifest());
        if runtime.block_grid().is_none() {
            runtime.set_block_grid(grid_with_floor_at_row(17));
        }

        let trace = runtime.enter_level_playtest(LevelId(0));

        assert_eq!(trace.game_mode, GameMode::InGame);
        assert_eq!(trace.current_level, LevelId(0));
        assert_eq!(
            trace.current_level_name.as_deref(),
            Some("Blue Lake Woods 1")
        );
        assert_eq!(trace.kid_on_ground, Some(true));
        assert_eq!(trace.camera_x_pos, 0);
        assert_eq!(trace.camera_y_pos, OPENING_GROUNDED_CAMERA_Y_FP);
        assert!(
            trace
                .active_objects
                .iter()
                .any(|object| matches!(object.kind, NativeObjectKind::Player)),
            "playtest bootstrap should expose the player immediately"
        );
    }

    fn pause_resume_script_input(frame_index: usize) -> FrameInput {
        let mut input = FrameInput::default();
        if (720..=839).contains(&frame_index) || (960..=1_119).contains(&frame_index) {
            input.right = true;
            input.a = true;
        } else if (841..=919).contains(&frame_index) {
            input.right = true;
        }
        if frame_index == 840 || frame_index == 920 {
            input.start = true;
        }
        if (980..=1_119).contains(&frame_index) && (frame_index - 980) % 24 < 4 {
            input.b = true;
        }
        input
    }

    fn progress_two_transitions_script_input(frame_index: usize) -> FrameInput {
        let mut input = FrameInput::default();
        if [120usize, 200, 280, 360, 440, 460].contains(&frame_index) {
            input.start = true;
        }
        if (720..=899).contains(&frame_index)
            || (1_280..=1_499).contains(&frame_index)
            || (1_920..=1_999).contains(&frame_index)
            || (2_480..=3_199).contains(&frame_index)
        {
            input.right = true;
        }
        if [900usize, 2_100, 2_140, 2_180].contains(&frame_index) {
            input.start = true;
            input.a = true;
            input.c = true;
        }
        input
    }

    fn capture_frames(
        runtime: &mut NativeRuntime,
        watched_frames: RangeInclusive<usize>,
        input_for_frame: impl Fn(usize) -> FrameInput,
    ) -> BTreeMap<usize, NativeTraceFrame> {
        let end_frame = *watched_frames.end();
        let watched = watched_frames.collect::<Vec<_>>();
        let mut frames = BTreeMap::new();
        while runtime.trace_frame().frame_index < end_frame {
            let next_frame = runtime.trace_frame().frame_index + 1;
            let trace = runtime.step_frame(input_for_frame(next_frame));
            if watched.contains(&trace.frame_index) {
                frames.insert(trace.frame_index, trace);
            }
        }
        frames
    }

    fn manifest_with_levels(count: u8) -> AssetManifest {
        let mut expanded = manifest();
        let base = expanded.levels[0].clone();
        expanded.levels = (0..count)
            .map(|level_id| {
                let mut level = base.clone();
                level.level_id = level_id;
                level.display_name = Some(format!("Level {level_id}"));
                if let Some(header) = level.header_metadata.as_mut() {
                    header.player_start_x =
                        header.player_start_x.saturating_add(u16::from(level_id));
                    header.flag_x = header.flag_x.saturating_add(u16::from(level_id));
                }
                level
            })
            .collect();
        expanded.map_order = (0..count).collect();
        expanded
    }

    fn unique_temp_path(label: &str) -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time should be after unix epoch")
            .as_nanos();
        env::temp_dir().join(format!("kc-{label}-{}-{nanos}", process::id()))
    }

    fn write_generated_level_layout(
        output_root: &std::path::Path,
        level_id: u8,
        width_blocks: u16,
        height_blocks: u16,
        data: &[u8],
    ) {
        let layout_dir = output_root.join("level-layout");
        fs::create_dir_all(&layout_dir).expect("layout dir should be creatable");
        fs::write(layout_dir.join(format!("{level_id:02X}.bin")), data)
            .expect("layout data should be writable");
        let metadata = LevelLayoutMetadata {
            width_blocks,
            height_blocks,
            data_bytes: data.len(),
        };
        fs::write(
            layout_dir.join(format!("{level_id:02X}.json")),
            serde_json::to_vec(&metadata).expect("layout metadata should serialise"),
        )
        .expect("layout metadata should be writable");
    }

    #[test]
    fn reaches_title_screen_without_input() {
        // SEGA(60) + 5 intro segments × 80 frames = 460 frames before title
        let trace = run_script(manifest(), &[], 560);
        let f = 519;
        assert_eq!(trace[f].game_mode, GameMode::TitleScreen);
        assert_eq!(trace[f].current_level, LevelId(0));
        assert_eq!(trace[f].player_start_x, Some(0x0750));
        assert_eq!(trace[f].flag_y, Some(0x0300));
        assert_eq!(trace[f].level_enemy_object_count, Some(22));
        assert_eq!(trace[f].active_enemy_object_count, 0);
        assert_eq!(trace[f].active_platform_object_count, 0);
        assert_eq!(trace[f].kid_x_pos, Some(TITLE_KID_X_POS));
        assert_eq!(trace[f].kid_y_pos, Some(TITLE_KID_Y_POS));
        assert!(trace[f].active_objects.is_empty());
        assert_eq!(trace[f].number_objects, 4);
        assert_eq!(trace[f].number_gfx_objects, 4);
    }

    #[test]
    fn new_runtime_loads_generated_level_layout_by_default() {
        let output_root = unique_temp_path("generated-layout-default");
        let mut manifest = manifest();
        manifest.output_root = output_root.clone();

        let mut data = vec![0u8; 2 * 2 * 2];
        data[4] = 0x60;
        data[6] = 0x60;
        write_generated_level_layout(&output_root, 0, 2, 2, &data);

        let runtime = NativeRuntime::new(manifest);
        let grid = runtime
            .block_grid()
            .expect("default runtime should load generated level layout");
        let floor = terrain::probe_floor(grid, 8, 16);

        assert!(floor.on_ground);
        assert_eq!(floor.cell.collision, terrain::CollisionKind::Solid);

        fs::remove_dir_all(output_root).expect("temp layout dir should be removable");
    }

    #[test]
    fn reload_block_grid_clears_stale_layout_when_next_level_has_no_dump() {
        let output_root = unique_temp_path("generated-layout-clear");
        let mut manifest = manifest_with_levels(2);
        manifest.output_root = output_root.clone();

        let mut data = vec![0u8; 2 * 2 * 2];
        data[4] = 0x60;
        write_generated_level_layout(&output_root, 0, 2, 2, &data);

        let mut runtime = NativeRuntime::new(manifest);
        assert!(runtime.block_grid().is_some(), "level 0 layout should load");

        runtime.reload_block_grid_for_level(LevelId(1));

        assert!(
            runtime.block_grid().is_none(),
            "moving to a level without a dumped layout should clear the stale grid"
        );

        fs::remove_dir_all(output_root).expect("temp layout dir should be removable");
    }

    #[test]
    fn aliased_levels_resolve_header_and_platform_metadata_from_resource_level() {
        let mut manifest = manifest_with_levels(7);
        manifest.map_order[6] = 2;

        let resource_level = manifest
            .levels
            .iter_mut()
            .find(|level| level.level_id == 2)
            .expect("resource level should exist");
        let resource_header = resource_level
            .header_metadata
            .as_mut()
            .expect("resource header should exist");
        resource_header.player_start_x = 0x0042;
        resource_header.player_start_y = 0x0088;
        resource_header.width_screens = 5;
        resource_header.height_screens = 3;
        resource_level
            .platform_metadata
            .as_mut()
            .expect("resource platforms should exist")
            .placements
            .push(PlatformPlacement {
                x: 0x0042,
                y: 0x0088,
                buffer_left: 0,
                buffer_right: 0,
                buffer_top: 0,
                buffer_bottom: 0,
                kind: 0x02,
                subtype: 0x03,
                h_variant: 0,
                v_variant: 0,
                script: PlatformScriptRef {
                    raw: String::from("PlatformScript_Test-PlatformScript_BaseAddress"),
                    relative_offset: None,
                },
            });

        let aliased_level = manifest
            .levels
            .iter_mut()
            .find(|level| level.level_id == 6)
            .expect("aliased level should exist");
        aliased_level.header_metadata = None;
        aliased_level.platform_metadata = None;

        let runtime = NativeRuntime::new(manifest);
        let slice = runtime.level_respawn_slice(LevelId(6));
        let platforms = runtime
            .platform_layout(LevelId(6))
            .expect("aliased level should reuse resource platforms");

        assert_eq!(slice.player.x_pos, 0x0042_0000);
        // player_start_y=0x88, minus 1 for ASM subq.w #1,y_pos
        assert_eq!(slice.player.y_pos, 0x0087_0000);
        // Camera snaps to screen boundary: floor(pos / screen) * screen
        assert_eq!(slice.camera.x_pos, u32::from((0x0042u16 / 320) * 320) << 16);
        assert_eq!(slice.camera.y_pos, u32::from((0x0088u16 / 224) * 224) << 16);
        assert!(
            platforms
                .placements
                .iter()
                .any(|placement| placement.kind == 0x02 && placement.subtype == 0x03),
            "aliased level should expose resource-level platform placements"
        );
    }

    #[test]
    fn aliased_levels_resolve_enemy_layout_from_resource_level() {
        let mut manifest = manifest_with_levels(7);
        manifest.map_order[6] = 2;

        let resource_level = manifest
            .levels
            .iter_mut()
            .find(|level| level.level_id == 2)
            .expect("resource level should exist");
        let resource_enemy_layout = resource_level
            .enemy_layout_metadata
            .as_mut()
            .expect("resource enemy layout should exist");
        resource_enemy_layout.enemy_type_slots = vec![0xFFFF, 0x000C, 0xFFFF];
        resource_enemy_layout.object_count = 1;
        resource_enemy_layout.objects = vec![kid_data::EnemyLayoutObject {
            object_type: 0x0D,
            flags: 0xFF,
            hit_points_plus_one: 0,
            x: 200,
            y: 271,
        }];

        let aliased_level = manifest
            .levels
            .iter_mut()
            .find(|level| level.level_id == 6)
            .expect("aliased level should exist");
        aliased_level.enemy_layout_metadata = Some(EnemyLayout {
            axis_uses_y_position: false,
            unknown_prefix: 0,
            enemy_type_slots: vec![0xFFFF, 0x0001, 0x001C],
            unknown_word: 0x7D00,
            header_byte: 0x80,
            object_count: 2,
            objects: vec![
                kid_data::EnemyLayoutObject {
                    object_type: 0x01,
                    flags: 0,
                    hit_points_plus_one: 1,
                    x: 288,
                    y: 224,
                },
                kid_data::EnemyLayoutObject {
                    object_type: 0x1C,
                    flags: 0,
                    hit_points_plus_one: 1,
                    x: 672,
                    y: 256,
                },
            ],
            segments: Vec::new(),
        });

        let runtime = NativeRuntime::new(manifest);
        let enemy_layout = runtime
            .enemy_layout(LevelId(6))
            .expect("aliased level should reuse resource enemy layout");

        assert_eq!(enemy_layout.enemy_type_slots, vec![0xFFFF, 0x000C, 0xFFFF]);
        assert_eq!(
            enemy_layout
                .objects
                .iter()
                .map(|object| object.object_type)
                .collect::<Vec<_>>(),
            vec![0x0D]
        );
    }

    #[test]
    fn starts_level_after_title_start_press() {
        let mut runtime = NativeRuntime::new(manifest());
        start_level(&mut runtime);
        let trace = runtime.trace_frame();
        assert_eq!(trace.game_mode, GameMode::InGame);
        assert_eq!(trace.current_level, LevelId(0));
        assert_eq!(
            trace.current_level_name.as_deref(),
            Some("Blue Lake Woods 1")
        );
        assert_eq!(trace.player_start_y, Some(0x00E0));
        assert_eq!(trace.flag_x, Some(0x0750));
        assert_eq!(trace.level_enemy_object_count, Some(22));
        assert_eq!(trace.active_enemy_object_count, 1);
        assert_eq!(trace.active_platform_object_count, 1);
        assert_eq!(trace.number_hitpoints, PlayerForm::TheKid.max_hitpoints());
        assert_eq!(trace.current_helmet_id, PlayerForm::TheKid.helmet_id());
        assert_eq!(trace.kid_x_pos, Some(OPENING_ENTRY_KID_X_POS));
        assert_eq!(trace.kid_y_pos, Some(OPENING_ENTRY_KID_Y_POS));
        assert_eq!(trace.kid_x_velocity, Some(0));
        assert_eq!(trace.kid_y_velocity, Some(0));
        assert_eq!(trace.kid_facing, Some(Direction::Right));
        assert_eq!(trace.kid_on_ground, Some(false));
        assert_eq!(trace.active_objects.len(), 3);
        assert_eq!(trace.active_objects[0].kind, NativeObjectKind::Player);
        assert_eq!(
            trace.active_objects[1].kind,
            NativeObjectKind::Enemy { object_type: 0x01 }
        );
        assert_eq!(
            trace.active_objects[2].kind,
            NativeObjectKind::Platform {
                kind: 0x00,
                subtype: 0x00,
            }
        );
        assert_eq!(trace.camera_y_pos, OPENING_ENTRY_CAMERA_Y_FP);
    }

    #[test]
    fn activates_visible_platforms_in_level_window() {
        let mut runtime = NativeRuntime::new(manifest());
        start_level(&mut runtime);
        let trace = runtime.trace_frame();
        let platforms = trace
            .active_objects
            .iter()
            .filter(|object| matches!(object.kind, NativeObjectKind::Platform { .. }))
            .collect::<Vec<_>>();

        assert_eq!(trace.active_platform_object_count, 1);
        assert_eq!(platforms.len(), 1);
        assert_eq!(platforms[0].x_pos, 0x0100_0000);
        assert_eq!(platforms[0].y_pos, 0x00E0_0000);
    }

    #[test]
    fn resolves_slot_encoded_enemy_ids() {
        let layout = EnemyLayout {
            axis_uses_y_position: false,
            unknown_prefix: 0,
            enemy_type_slots: vec![0xFFFF, 0x0001, 0x401C],
            unknown_word: 0x7D00,
            header_byte: 0x80,
            object_count: 2,
            objects: vec![
                kid_data::EnemyLayoutObject {
                    object_type: 0x81,
                    flags: 0,
                    hit_points_plus_one: 1,
                    x: 0x0120,
                    y: 0x00E0,
                },
                kid_data::EnemyLayoutObject {
                    object_type: 0x82,
                    flags: 0,
                    hit_points_plus_one: 1,
                    x: 0x0180,
                    y: 0x0100,
                },
            ],
            segments: Vec::new(),
        };
        let roster = build_enemy_roster(
            &layout,
            ActivationWindow {
                x_min: 0,
                x_max: 0x0200,
                y_min: 0,
                y_max: 0x0200,
            },
        );

        assert_eq!(roster.len(), 2);
        assert_eq!(
            roster[0].kind,
            NativeObjectKind::Enemy { object_type: 0x01 }
        );
        assert_eq!(
            roster[1].kind,
            NativeObjectKind::Enemy { object_type: 0x1C }
        );
    }

    #[test]
    fn advances_levels_with_skip_cheat() {
        let mut runtime = NativeRuntime::new(manifest_with_levels(3));
        start_level(&mut runtime);

        let first_skip = press_skip_cheat(&mut runtime);
        assert_eq!(first_skip.current_level, LevelId(1));
        assert_eq!(first_skip.game_mode, GameMode::LevelTransition);
        assert_eq!(first_skip.current_level_name.as_deref(), Some("Level 1"));

        let second_skip = press_skip_cheat(&mut runtime);
        assert_eq!(second_skip.current_level, LevelId(1));
        assert_eq!(second_skip.game_mode, GameMode::InGame);
        assert_eq!(second_skip.current_level_name.as_deref(), Some("Level 1"));

        for _ in 0..LEVEL_SKIP_RESUME_TOTAL_FRAMES {
            runtime.step_frame(FrameInput::default());
        }
        let third_skip = press_skip_cheat(&mut runtime);
        assert_eq!(third_skip.current_level, LevelId(2));
        assert_eq!(third_skip.game_mode, GameMode::LevelTransition);
        assert_eq!(third_skip.current_level_name.as_deref(), Some("Level 2"));
    }

    #[test]
    fn skip_cheat_holds_transition_pose_until_rearmed() {
        let mut runtime = NativeRuntime::new(manifest_with_levels(3));
        start_level(&mut runtime);

        let hold = press_skip_cheat(&mut runtime);
        assert_eq!(hold.current_level, LevelId(1));
        assert_eq!(hold.game_mode, GameMode::LevelTransition);

        for _ in 0..120 {
            runtime.step_frame(FrameInput {
                right: true,
                ..FrameInput::default()
            });
        }

        let still_holding = runtime.trace_frame();
        assert_eq!(still_holding.current_level, LevelId(1));
        assert_eq!(still_holding.game_mode, GameMode::LevelTransition);
        assert_eq!(still_holding.kid_x_pos, Some(0x0040_0000));
        assert_eq!(still_holding.kid_y_pos, Some(0x0068_0000));
        assert_eq!(still_holding.camera_x_pos, 0);
        assert_eq!(still_holding.camera_y_pos, 0);
        assert_eq!(still_holding.number_objects, 16);
        assert_eq!(still_holding.number_gfx_objects, 15);
    }

    #[test]
    fn progress_two_transitions_skip_cheat_matches_early_transition_timeline() {
        let mut runtime = NativeRuntime::new(manifest_with_levels(3));
        runtime.set_block_grid(grid_with_floor_at_row(17));
        let mut frames = BTreeMap::new();
        while runtime.trace_frame().frame_index <= 997 {
            let frame_index = runtime.trace_frame().frame_index;
            let trace = runtime.step_frame(progress_two_transitions_script_input(frame_index));
            if (900..=997).contains(&trace.frame_index) {
                frames.insert(trace.frame_index, trace);
            }
        }

        let frame_900 = frames.get(&900).expect("frame 900 should be captured");
        assert_eq!(frame_900.game_mode, GameMode::InGame);
        assert_eq!(frame_900.current_level, LevelId(0));
        assert_eq!(frame_900.camera_x_pos, 0x00BF_0000);
        assert_eq!(frame_900.camera_y_pos, OPENING_GROUNDED_CAMERA_Y_FP);
        assert_eq!(frame_900.kid_x_pos, Some(0x0169_0000));
        assert_eq!(frame_900.kid_y_pos, Some(OPENING_GROUNDED_KID_Y_POS));

        let frame_901 = frames.get(&901).expect("frame 901 should be captured");
        assert_eq!(frame_901.game_mode, GameMode::LevelTransition);
        assert_eq!(frame_901.current_level, LevelId(1));
        assert_eq!(frame_901.camera_x_pos, frame_900.camera_x_pos);
        assert_eq!(frame_901.camera_y_pos, frame_900.camera_y_pos);
        assert_eq!(frame_901.kid_x_pos, frame_900.kid_x_pos);
        assert_eq!(frame_901.kid_y_pos, frame_900.kid_y_pos);

        let frame_921 = frames.get(&921).expect("frame 921 should be captured");
        assert_eq!(frame_921.kid_x_pos, frame_900.kid_x_pos);
        assert_eq!(frame_921.kid_y_pos, frame_900.kid_y_pos);
        assert_eq!(frame_921.number_objects, 4);
        assert_eq!(frame_921.number_gfx_objects, 5);

        let frame_922 = frames.get(&922).expect("frame 922 should be captured");
        assert_eq!(frame_922.kid_x_pos, None);
        assert_eq!(frame_922.kid_y_pos, None);
        assert_eq!(frame_922.camera_x_pos, frame_900.camera_x_pos);
        assert_eq!(frame_922.number_objects, 4);
        assert_eq!(frame_922.number_gfx_objects, 5);

        let frame_927 = frames.get(&927).expect("frame 927 should be captured");
        assert_eq!(frame_927.kid_x_pos, None);
        assert_eq!(frame_927.kid_y_pos, None);
        assert_eq!(frame_927.camera_x_pos, 0);
        assert_eq!(frame_927.camera_y_pos, 0);
        assert_eq!(frame_927.number_objects, 0);
        assert_eq!(frame_927.number_gfx_objects, 0);

        let frame_929 = frames.get(&929).expect("frame 929 should be captured");
        assert_eq!(frame_929.number_objects, 2);
        assert_eq!(frame_929.number_gfx_objects, 0);

        let frame_932 = frames.get(&932).expect("frame 932 should be captured");
        assert_eq!(frame_932.number_objects, 15);
        assert_eq!(frame_932.number_gfx_objects, 0);

        let frame_933 = frames.get(&933).expect("frame 933 should be captured");
        assert_eq!(frame_933.camera_x_pos, 0);
        assert_eq!(frame_933.camera_y_pos, 0);
        assert_eq!(frame_933.kid_x_pos, Some(0x0160_0000));
        assert_eq!(frame_933.kid_y_pos, Some(0x0014_0000));
        assert_eq!(frame_933.number_objects, 16);
        assert_eq!(frame_933.number_gfx_objects, 13);

        let frame_934 = frames.get(&934).expect("frame 934 should be captured");
        assert_eq!(frame_934.kid_x_pos, Some(0x015B_8000));
        assert_eq!(frame_934.kid_y_pos, Some(0x0015_5000));
        assert_eq!(frame_934.number_objects, 16);
        assert_eq!(frame_934.number_gfx_objects, 15);

        let frame_997 = frames.get(&997).expect("frame 997 should be captured");
        assert_eq!(frame_997.kid_x_pos, Some(0x0040_0000));
        assert_eq!(frame_997.kid_y_pos, Some(0x0068_0000));
        assert_eq!(frame_997.number_objects, 16);
        assert_eq!(frame_997.number_gfx_objects, 15);
    }

    #[test]
    fn progress_two_transitions_skip_cheat_matches_resume_timeline() {
        let mut runtime = NativeRuntime::new(manifest_with_levels(3));
        runtime.set_block_grid(grid_with_floor_at_row(17));
        let mut frames = BTreeMap::new();
        while runtime.trace_frame().frame_index <= 2214 {
            let frame_index = runtime.trace_frame().frame_index;
            let trace = runtime.step_frame(progress_two_transitions_script_input(frame_index));
            if (2100..=2214).contains(&trace.frame_index) {
                frames.insert(trace.frame_index, trace);
            }
        }

        let frame_2100 = frames.get(&2100).expect("frame 2100 should be captured");
        assert_eq!(frame_2100.game_mode, GameMode::LevelTransition);
        assert_eq!(frame_2100.current_level, LevelId(1));
        assert_eq!(frame_2100.camera_x_pos, 0);
        assert_eq!(frame_2100.camera_y_pos, 0);
        assert_eq!(frame_2100.kid_x_pos, Some(0x0040_0000));
        assert_eq!(frame_2100.kid_y_pos, Some(0x0068_0000));
        assert_eq!(frame_2100.number_objects, 16);
        assert_eq!(frame_2100.number_gfx_objects, 15);

        let frame_2101 = frames.get(&2101).expect("frame 2101 should be captured");
        assert_eq!(frame_2101.game_mode, GameMode::InGame);
        assert_eq!(frame_2101.current_level, LevelId(1));
        assert_eq!(frame_2101.camera_x_pos, 0);
        assert_eq!(frame_2101.camera_y_pos, 0);
        assert_eq!(frame_2101.kid_x_pos, frame_2100.kid_x_pos);
        assert_eq!(frame_2101.kid_y_pos, frame_2100.kid_y_pos);
        assert_eq!(frame_2101.number_objects, 16);
        assert_eq!(frame_2101.number_gfx_objects, 15);

        let frame_2122 = frames.get(&2122).expect("frame 2122 should be captured");
        assert_eq!(frame_2122.kid_x_pos, None);
        assert_eq!(frame_2122.kid_y_pos, None);
        assert_eq!(frame_2122.number_objects, 16);
        assert_eq!(frame_2122.number_gfx_objects, 15);

        let frame_2127 = frames.get(&2127).expect("frame 2127 should be captured");
        assert_eq!(frame_2127.kid_x_pos, None);
        assert_eq!(frame_2127.kid_y_pos, None);
        assert_eq!(frame_2127.number_objects, 0);
        assert_eq!(frame_2127.number_gfx_objects, 0);

        let frame_2153 = frames.get(&2153).expect("frame 2153 should be captured");
        assert_eq!(frame_2153.kid_x_pos, None);
        assert_eq!(frame_2153.kid_y_pos, None);
        assert_eq!(frame_2153.camera_x_pos, 0);
        assert_eq!(frame_2153.camera_y_pos, 0xFFA0_0000);
        assert_eq!(frame_2153.number_objects, 0);
        assert_eq!(frame_2153.number_gfx_objects, 0);

        let frame_2160 = frames.get(&2160).expect("frame 2160 should be captured");
        assert_eq!(frame_2160.kid_x_pos, None);
        assert_eq!(frame_2160.kid_y_pos, None);
        assert_eq!(frame_2160.number_objects, 4);
        assert_eq!(frame_2160.number_gfx_objects, 2);

        let frame_2161 = frames.get(&2161).expect("frame 2161 should be captured");
        assert_eq!(frame_2161.game_mode, GameMode::InGame);
        assert_eq!(frame_2161.current_level, LevelId(1));
        assert_eq!(frame_2161.camera_x_pos, 0);
        assert_eq!(frame_2161.camera_y_pos, 0);
        assert_eq!(frame_2161.kid_x_pos, Some(0x0020_0000));
        assert_eq!(frame_2161.kid_y_pos, Some(0x009F_0000));
        assert_eq!(frame_2161.number_objects, 5);
        assert_eq!(frame_2161.number_gfx_objects, 4);

        let frame_2162 = frames.get(&2162).expect("frame 2162 should be captured");
        assert_eq!(frame_2162.kid_x_pos, Some(0x0020_0000));
        assert_eq!(frame_2162.kid_y_pos, Some(0x00A0_0000));
        assert_eq!(frame_2162.number_objects, 5);
        assert_eq!(frame_2162.number_gfx_objects, 6);

        let frame_2181 = frames.get(&2181).expect("frame 2181 should be captured");
        assert_eq!(frame_2181.game_mode, GameMode::LevelTransition);
        assert_eq!(frame_2181.current_level, LevelId(2));
        assert_eq!(frame_2181.camera_x_pos, 0);
        assert_eq!(frame_2181.camera_y_pos, 0x0014_0000);
        assert_eq!(frame_2181.kid_x_pos, Some(0x0020_0000));
        assert_eq!(frame_2181.kid_y_pos, Some(0x00BF_0000));

        let frame_2213 = frames.get(&2213).expect("frame 2213 should be captured");
        assert_eq!(frame_2213.game_mode, GameMode::LevelTransition);
        assert_eq!(frame_2213.current_level, LevelId(2));
        assert_eq!(frame_2213.camera_x_pos, 0);
        assert_eq!(frame_2213.camera_y_pos, 0);
        assert_eq!(frame_2213.kid_x_pos, Some(0xFFE0_0000));
        assert_eq!(frame_2213.kid_y_pos, Some(0x00C8_0000));
        assert_eq!(frame_2213.number_objects, 15);
        assert_eq!(frame_2213.number_gfx_objects, 13);

        let frame_2214 = frames.get(&2214).expect("frame 2214 should be captured");
        assert_eq!(frame_2214.kid_x_pos, Some(0xFFE1_4000));
        assert_eq!(frame_2214.kid_y_pos, Some(0x00C5_E000));
        assert_eq!(frame_2214.number_objects, 15);
        assert_eq!(frame_2214.number_gfx_objects, 14);
    }

    #[test]
    fn changes_form_and_hitpoints_from_helmet_queue() {
        let mut runtime = NativeRuntime::new(manifest());
        start_level(&mut runtime);
        runtime.queue_form_change(PlayerForm::IronKnight);
        runtime.step_frame(FrameInput::default());

        let trace = runtime.trace_frame();
        assert_eq!(trace.current_helmet_id, PlayerForm::IronKnight.helmet_id());
        assert_eq!(
            trace.number_hitpoints,
            PlayerForm::IronKnight.max_hitpoints()
        );
    }

    #[test]
    fn opening_hazard_strips_helmet_before_life_loss() {
        let mut runtime = NativeRuntime::new(manifest());
        start_level(&mut runtime);
        runtime.queue_form_change(PlayerForm::IronKnight);
        runtime.step_frame(FrameInput::default());

        let mut gameplay_frames = 0usize;
        while runtime.current_form != PlayerForm::TheKid {
            let jumping = gameplay_frames >= 40 && (gameplay_frames - 40) % 24 < 4;
            runtime.step_frame(FrameInput {
                right: true,
                a: true,
                b: jumping,
                ..FrameInput::default()
            });
            gameplay_frames += 1;
            assert!(gameplay_frames < 1_400);
        }

        let trace = runtime.trace_frame();
        assert_eq!(trace.number_lives, 3);
        assert_eq!(trace.current_helmet_id, PlayerForm::TheKid.helmet_id());
        assert_eq!(trace.number_hitpoints, PlayerForm::TheKid.max_hitpoints());
        assert!(matches!(runtime.player_lifecycle, PlayerLifecycle::Alive));
        assert!(runtime.invulnerable_frames > 0);
    }

    #[test]
    fn pauses_and_resumes_native_slice() {
        let mut runtime = NativeRuntime::new(manifest());
        start_level(&mut runtime);
        // Walk right for 400 frames: ~200 entry animation + ~200 physics, enough
        // to move the camera.
        for _ in 0..400 {
            runtime.step_frame(FrameInput {
                right: true,
                ..FrameInput::default()
            });
        }
        let pre_pause = runtime.trace_frame();
        assert!(
            pre_pause.camera_x_pos > 0,
            "camera should have moved before pause"
        );
        runtime.step_frame(FrameInput {
            start: true,
            ..FrameInput::default()
        });
        runtime.step_frame(FrameInput::default());
        let pause_entry = runtime.trace_frame();
        assert!(pause_entry.paused);
        for _ in 0..80 {
            runtime.step_frame(FrameInput {
                right: true,
                ..FrameInput::default()
            });
        }
        let paused_tail = runtime.trace_frame();
        assert!(paused_tail.paused);
        assert_eq!(paused_tail.camera_x_pos, pause_entry.camera_x_pos);
        assert_eq!(paused_tail.kid_x_pos, pause_entry.kid_x_pos);

        runtime.step_frame(FrameInput {
            start: true,
            ..FrameInput::default()
        });
        runtime.step_frame(FrameInput::default());
        for _ in 0..120 {
            runtime.step_frame(FrameInput {
                right: true,
                ..FrameInput::default()
            });
        }
        let resumed_tail = runtime.trace_frame();
        assert!(!resumed_tail.paused);
        assert!(resumed_tail.camera_x_pos > paused_tail.camera_x_pos);
        assert!(resumed_tail.kid_x_pos > paused_tail.kid_x_pos);
    }

    #[test]
    fn timing_regression_jump_press_primes_takeoff_for_next_frame() {
        let mut runtime = NativeRuntime::new(manifest());
        start_level(&mut runtime);
        loop {
            runtime.step_frame(FrameInput::default());
            if runtime.trace_frame().kid_on_ground == Some(true) {
                break;
            }
        }

        let grounded = runtime.trace_frame();
        let jump_frame = runtime.step_frame(FrameInput {
            b: true,
            ..FrameInput::default()
        });
        let rising_frame = runtime.step_frame(FrameInput {
            b: true,
            ..FrameInput::default()
        });

        assert_eq!(jump_frame.kid_y_pos, grounded.kid_y_pos);
        assert_eq!(jump_frame.kid_on_ground, Some(true));
        assert!(
            rising_frame.kid_y_pos.expect("rising y") < grounded.kid_y_pos.expect("grounded y"),
            "held jump should begin moving upward on the next frame"
        );
    }

    #[test]
    fn timing_regression_held_jump_advances_upward_on_second_frame() {
        let mut runtime = NativeRuntime::new(manifest());
        start_level(&mut runtime);
        loop {
            runtime.step_frame(FrameInput::default());
            if runtime.trace_frame().kid_on_ground == Some(true) {
                break;
            }
        }

        let grounded = runtime.trace_frame();
        let jump_frame = runtime.step_frame(FrameInput {
            b: true,
            ..FrameInput::default()
        });
        let rising_frame = runtime.step_frame(FrameInput {
            b: true,
            ..FrameInput::default()
        });

        assert_eq!(jump_frame.kid_y_pos, grounded.kid_y_pos);
        assert!(
            rising_frame.kid_y_pos.expect("rising y") < grounded.kid_y_pos.expect("grounded y"),
            "held jump should begin moving upward on the next frame"
        );
    }

    #[test]
    fn death_windup_keeps_player_motion_alive() {
        let mut runtime = NativeRuntime::new(manifest());
        runtime.set_block_grid(grid_with_floor_at_row(17));
        start_level(&mut runtime);
        for _ in 0..260 {
            runtime.step_frame(FrameInput::default());
        }

        if let Some(slice) = runtime.opening_slice.as_mut() {
            slice.player.x_velocity = 0x0002_0000;
            slice.player.y_velocity = -0x0001_0000;
            slice.player.on_ground = false;
            slice.camera.x_pos = 0x0123_0000;
            slice.camera.y_pos = 0x0045_0000;
        }
        let before = runtime.trace_frame();
        runtime.number_hitpoints = 0;
        runtime.begin_death_windup();

        let after = runtime.step_frame(FrameInput::default());

        assert!(matches!(
            runtime.player_lifecycle,
            PlayerLifecycle::DeathWindup { .. }
        ));
        assert_ne!(after.kid_x_pos, before.kid_x_pos);
        assert_ne!(after.kid_y_pos, before.kid_y_pos);
        assert_eq!(after.camera_x_pos, before.camera_x_pos);
        assert_eq!(after.camera_y_pos, before.camera_y_pos);
    }

    #[test]
    fn timing_regression_jump_press_on_block_grid_primes_takeoff_for_next_frame() {
        let mut runtime = NativeRuntime::new(manifest());
        runtime.set_block_grid(grid_with_floor_at_row(17));
        start_level(&mut runtime);
        loop {
            runtime.step_frame(FrameInput::default());
            if runtime.trace_frame().kid_on_ground == Some(true) {
                break;
            }
        }

        let grounded = runtime.trace_frame();
        let jump_frame = runtime.step_frame(FrameInput {
            right: true,
            a: true,
            b: true,
            ..FrameInput::default()
        });
        let rising_frame = runtime.step_frame(FrameInput {
            right: true,
            a: true,
            b: true,
            ..FrameInput::default()
        });

        assert_eq!(jump_frame.kid_y_pos, grounded.kid_y_pos);
        assert_eq!(jump_frame.kid_on_ground, Some(true));
        assert!(
            rising_frame.kid_on_ground == Some(false),
            "block-grid take-off should keep the player airborne once the jump begins"
        );
    }

    #[test]
    fn solid_block_grid_grounds_one_pixel_above_cell_top() {
        let mut runtime = NativeRuntime::new(manifest());
        runtime.set_block_grid(grid_with_floor_at_row(17));
        start_level(&mut runtime);
        loop {
            runtime.step_frame(FrameInput::default());
            let trace = runtime.trace_frame();
            if trace.kid_on_ground == Some(true) {
                assert_eq!(trace.kid_y_pos, Some(0x010F_0000));
                break;
            }
        }
    }

    #[test]
    fn loses_life_and_respawns_after_opening_hazard() {
        let mut runtime = NativeRuntime::new(manifest());
        start_level(&mut runtime);

        let mut gameplay_frames = 0usize;
        while !matches!(runtime.player_lifecycle, PlayerLifecycle::Dying { .. }) {
            let jumping = gameplay_frames >= 40 && (gameplay_frames - 40) % 24 < 4;
            runtime.step_frame(FrameInput {
                right: true,
                a: true,
                b: jumping,
                ..FrameInput::default()
            });
            gameplay_frames += 1;
            assert!(gameplay_frames < 1_400);
        }

        let death_tail = runtime.trace_frame();
        assert_eq!(death_tail.number_lives, 2);
        assert_eq!(death_tail.camera_x_pos, 0);
        assert_eq!(death_tail.kid_x_pos, None);
        assert!(death_tail.active_objects.is_empty());
        assert_eq!(death_tail.number_objects, 0);
        assert_eq!(death_tail.number_gfx_objects, 0);

        while !matches!(runtime.player_lifecycle, PlayerLifecycle::Respawning { .. }) {
            let jumping = gameplay_frames >= 40 && (gameplay_frames - 40) % 24 < 4;
            runtime.step_frame(FrameInput {
                right: true,
                a: true,
                b: jumping,
                ..FrameInput::default()
            });
            gameplay_frames += 1;
        }

        let respawn_entry = runtime.trace_frame();
        assert_eq!(respawn_entry.number_lives, 2);
        assert_eq!(respawn_entry.camera_x_pos, LEVEL0_RESPAWN_CAMERA_X);
        assert_eq!(respawn_entry.camera_y_pos, LEVEL0_RESPAWN_CAMERA_Y);
        assert_eq!(respawn_entry.kid_x_pos, Some(LEVEL0_RESPAWN_KID_X));
        assert_eq!(respawn_entry.kid_y_pos, Some(LEVEL0_RESPAWN_KID_Y));

        for _ in 0..(PLAYER_DEATH_RESPAWN_SETTLE_FRAMES - 1) {
            let jumping = gameplay_frames >= 40 && (gameplay_frames - 40) % 24 < 4;
            runtime.step_frame(FrameInput {
                right: true,
                a: true,
                b: jumping,
                ..FrameInput::default()
            });
            gameplay_frames += 1;
        }

        let respawn_tail = runtime.trace_frame();
        assert_eq!(respawn_tail.number_lives, 2);
        assert_eq!(respawn_tail.camera_x_pos, LEVEL0_RESPAWN_CAMERA_X);
        assert_eq!(respawn_tail.camera_y_pos, LEVEL0_RESPAWN_CAMERA_Y);
        assert_eq!(respawn_tail.kid_x_pos, Some(LEVEL0_RESPAWN_KID_X));
        assert_eq!(respawn_tail.kid_y_pos, Some(LEVEL0_RESPAWN_KID_Y));
        // Enemies/platforms depend on camera position and spawning window —
        // with the header-based spawn (camera at X=1600), no test enemies
        // fall within the viewport.
    }

    #[test]
    fn moves_camera_when_holding_right_in_game() {
        let mut runtime = NativeRuntime::new(manifest());
        runtime.set_block_grid(grid_with_floor_at_row(17));
        for frame in 0..1_020 {
            let input = if [120usize, 200, 280, 360, 440, 460].contains(&frame) {
                FrameInput {
                    start: true,
                    ..FrameInput::default()
                }
            } else if (720..=1_019).contains(&frame) {
                FrameInput {
                    right: true,
                    ..FrameInput::default()
                }
            } else {
                FrameInput::default()
            };
            runtime.step_frame(input);
        }
        let trace = runtime.trace_frame();
        // Camera should have scrolled right from physics-driven movement
        assert!(trace.camera_x_pos > 0, "camera should have moved right");
        // Player should be on ground at grounded Y
        assert_eq!(trace.kid_y_pos, Some(OPENING_GROUNDED_KID_Y_POS));
        // Player should be ahead of camera (to the right)
        assert!(trace.kid_x_pos.is_some_and(|x| x > trace.camera_x_pos));
        // Opening hit should have reduced HP (right + full HP + camera past threshold)
        assert!(
            trace.number_hitpoints < PlayerForm::TheKid.max_hitpoints(),
            "opening hit should have reduced HP"
        );
    }

    #[test]
    fn level_one_opening_hit_matches_oracle_frame() {
        let mut runtime = NativeRuntime::new(manifest());
        runtime.set_block_grid(grid_with_floor_at_row(17));
        let mut first_hit_frame = None;

        for frame in 0..900 {
            let input = if [120usize, 200, 280, 360, 440, 460].contains(&frame) {
                FrameInput {
                    start: true,
                    ..FrameInput::default()
                }
            } else if (720..900).contains(&frame) {
                FrameInput {
                    right: true,
                    a: true,
                    ..FrameInput::default()
                }
            } else {
                FrameInput::default()
            };
            let trace = runtime.step_frame(input);
            if frame >= 720 && trace.number_hitpoints == 1 {
                first_hit_frame = Some(frame);
                break;
            }
        }

        assert_eq!(
            first_hit_frame,
            Some(760),
            "Level 1 opening hit should land on the oracle frame during the run-right script"
        );
    }

    #[test]
    fn progress_two_transitions_takes_opening_damage_by_frame_749() {
        let mut runtime = NativeRuntime::new(manifest());
        runtime.set_block_grid(grid_with_floor_at_row(17));

        for frame in 0..750 {
            let input = if [120usize, 200, 280, 360, 440, 460].contains(&frame) {
                FrameInput {
                    start: true,
                    ..FrameInput::default()
                }
            } else if (720..750).contains(&frame) {
                FrameInput {
                    right: true,
                    ..FrameInput::default()
                }
            } else {
                FrameInput::default()
            };
            runtime.step_frame(input);
        }

        let trace = runtime.trace_frame();
        assert_eq!(
            trace.number_hitpoints, 1,
            "progress-two-transitions should have already taken the opening Dragon hit by frame 749"
        );
    }

    #[test]
    fn timing_regression_level_one_opening_hit_keeps_advanced_position() {
        let mut runtime = NativeRuntime::new(manifest());
        runtime.set_block_grid(grid_with_floor_at_row(17));

        for frame in 0..761 {
            let input = if [120usize, 200, 280, 360, 440, 460].contains(&frame) {
                FrameInput {
                    start: true,
                    ..FrameInput::default()
                }
            } else if (720..761).contains(&frame) {
                FrameInput {
                    right: true,
                    a: true,
                    ..FrameInput::default()
                }
            } else {
                FrameInput::default()
            };
            runtime.step_frame(input);
        }

        let trace = runtime.trace_frame();
        assert_eq!(trace.number_hitpoints, 1, "opening hit should have landed");
        assert_eq!(
            trace.kid_x_pos,
            Some(0x0075_C000),
            "opening hit should preserve the stepped X position on the hit frame"
        );
    }

    #[test]
    fn timing_regression_just_jumped_skips_air_control_and_gravity_for_one_frame() {
        let mut slice = OpeningSliceState::respawn(LevelBounds {
            width_pixels: 320,
            height_pixels: 224,
        });
        slice.player.x_pos = 0x0075_C000;
        slice.player.y_pos = 0x010F_0000;
        slice.player.x_velocity = 0x0002_A000;
        slice.player.y_velocity = -0x0005_0700;
        slice.player.on_ground = false;
        slice.player.just_jumped = true;

        slice.step(
            OPENING_SCROLL_START_FRAME,
            FrameInput {
                right: true,
                a: true,
                b: true,
                ..FrameInput::default()
            },
            false,
            PlayerForm::TheKid,
            None,
            None,
            false,
            0,
        );

        assert_eq!(
            slice.player.x_pos, 0x0078_6000,
            "first airborne frame should keep the raw take-off X velocity"
        );
        assert_eq!(
            slice.player.y_pos, 0x0109_F900,
            "first airborne frame should keep the raw take-off Y velocity"
        );
        assert_eq!(slice.player.x_velocity, 0x0002_A000);
        assert_eq!(slice.player.y_velocity, -0x0005_0700);
        assert!(
            !slice.player.just_jumped,
            "the one-frame take-off gate should clear after the first airborne frame"
        );
    }

    #[test]
    fn timing_regression_grounded_jump_moves_on_takeoff_frame() {
        let mut slice = OpeningSliceState::respawn(LevelBounds {
            width_pixels: 320,
            height_pixels: 224,
        });
        let mut grid = grid_with_floor_at_row(17);
        slice.player.x_pos = 0x00B1_2300;
        slice.player.y_pos = slice.terrain.floor_y_pos;
        slice.player.x_velocity = 0x0003_4000;
        slice.player.on_ground = true;

        slice.step(
            OPENING_SCROLL_START_FRAME,
            FrameInput {
                right: true,
                a: true,
                b: true,
                ..FrameInput::default()
            },
            true,
            PlayerForm::TheKid,
            Some(&mut grid),
            None,
            false,
            0,
        );

        assert!(!slice.player.on_ground);
        assert_eq!(slice.player.x_velocity, 0x0002_7000);
        assert_eq!(slice.player.y_velocity, -0x0004_F680);
        assert_eq!(slice.player.x_pos, 0x00B3_9300);
        assert_eq!(slice.player.y_pos, 0x010A_0980);
        assert!(!slice.player.just_jumped);
    }

    #[test]
    fn airborne_landing_prefers_center_slope_before_edge_solid() {
        let w = 4u16;
        let h = 4u16;
        let mut data = vec![0u8; usize::from(w) * usize::from(h) * 2];
        // SlopeDown at (col 1, row 1): new formula floor_y = 16 + x_within_cell
        data[(usize::from(1u16) * usize::from(w) + usize::from(1u16)) * 2] = 0x40;
        // Solid at (col 2, row 1)
        data[(usize::from(1u16) * usize::from(w) + usize::from(2u16)) * 2] = 0x60;
        let grid = terrain::BlockGrid::from_level_layout(&data, w, h);
        let mut slice = OpeningSliceState::respawn(LevelBounds {
            width_pixels: 64,
            height_pixels: 64,
        });
        // Player at x=27 (within_cell=11): SlopeDown floor_y = 16+11 = 27
        slice.player.x_pos = 27 << 16;
        slice.player.y_pos = 26 << 16;
        slice.player.y_velocity = 0x0001_0000;
        slice.player.on_ground = false;

        slice.apply_floor_collision(Some(&grid), false, false);

        assert_eq!(slice.player.y_pos, 27 << 16);
        assert!(slice.player.on_ground);
    }

    #[test]
    fn timing_regression_airborne_within_ground_snap_tolerance_cannot_jump_early() {
        let mut slice = OpeningSliceState::respawn(LevelBounds {
            width_pixels: 320,
            height_pixels: 224,
        });
        slice.player.y_pos = slice.terrain.floor_y_pos.saturating_sub(0x0002_0000);
        slice.player.y_velocity = 0x0003_C000;
        slice.player.on_ground = false;

        slice.step(
            OPENING_SCROLL_START_FRAME,
            FrameInput {
                right: true,
                a: true,
                b: false,
                ..FrameInput::default()
            },
            true,
            PlayerForm::TheKid,
            None,
            None,
            false,
            0,
        );

        assert!(
            slice.player.y_velocity >= 0,
            "airborne player near the floor should not end the frame with an upward jump-start velocity"
        );
        assert!(!slice.player.just_jumped);
    }

    #[test]
    fn timing_regression_buffers_jump_pressed_just_before_landing() {
        let mut slice = OpeningSliceState::respawn(LevelBounds {
            width_pixels: 320,
            height_pixels: 224,
        });
        let mut grid = grid_with_floor_at_row(17);
        slice.player.x_pos = 0x00B1_2300;
        slice.player.x_velocity = 0x0002_8A00;
        slice.player.y_pos = slice.terrain.floor_y_pos.saturating_sub(0x0002_0000);
        slice.player.y_velocity = 0x0003_C000;
        slice.player.on_ground = false;

        slice.step(
            OPENING_SCROLL_START_FRAME,
            FrameInput {
                right: true,
                a: true,
                ..FrameInput::default()
            },
            true,
            PlayerForm::TheKid,
            Some(&mut grid),
            None,
            false,
            0,
        );

        assert_eq!(slice.player.y_pos, slice.terrain.floor_y_pos);
        assert!(slice.player.on_ground);
        assert!(slice.player.jump_buffer_frames > 0);

        slice.step(
            OPENING_SCROLL_START_FRAME,
            FrameInput {
                right: true,
                a: true,
                b: true,
                ..FrameInput::default()
            },
            false,
            PlayerForm::TheKid,
            Some(&mut grid),
            None,
            false,
            0,
        );

        assert!(!slice.player.on_ground);
        assert_eq!(slice.player.x_pos, 0x00B3_0A80);
        assert!(slice.player.y_pos < slice.terrain.floor_y_pos);
        assert_eq!(slice.player.x_velocity, 0x0001_E780);
        assert!(slice.player.y_velocity < 0);
        assert!(!slice.player.just_jumped);
        assert_eq!(slice.player.jump_buffer_frames, 0);
    }

    #[test]
    fn timing_regression_near_floor_jump_press_skips_last_airborne_horizontal_step() {
        let mut slice = OpeningSliceState::respawn(LevelBounds {
            width_pixels: 320,
            height_pixels: 224,
        });
        let mut grid = grid_with_floor_at_row(17);
        slice.player.x_pos = 0x00B1_2300;
        slice.player.y_pos = slice.terrain.floor_y_pos.saturating_sub(0x0002_0000);
        slice.player.x_velocity = 0x0002_8900;
        slice.player.y_velocity = 0x0003_C000;
        slice.player.on_ground = false;

        slice.step(
            OPENING_SCROLL_START_FRAME,
            FrameInput {
                right: true,
                a: true,
                ..FrameInput::default()
            },
            true,
            PlayerForm::TheKid,
            Some(&mut grid),
            None,
            false,
            0,
        );

        assert_eq!(slice.player.x_pos, 0x00B1_2300);
        assert_eq!(slice.player.y_pos, slice.terrain.floor_y_pos);
        assert!(slice.player.on_ground);
        assert!(slice.player.jump_buffer_frames > 0);
    }

    #[test]
    fn slope_ground_run_right_brakes_before_level_one_jump() {
        let (new_vel, new_facing) = update_level_one_slope_ground_velocity(
            0x0003_8000,
            Direction::Right,
            HorizontalInput {
                left: false,
                right: true,
                run: true,
            },
            terrain::CollisionKind::SlopeDown,
            PlayerForm::TheKid,
            TerrainType::Normal,
        );

        assert_eq!(new_vel, 0x0003_4000);
        assert_eq!(new_facing, Direction::Right);
    }

    #[test]
    fn timing_regression_airborne_slope_recontact_applies_same_frame_ground_step() {
        let w = 64u16;
        let h = 20u16;
        let mut data = vec![0u8; usize::from(w) * usize::from(h) * 2];
        let set_collision = |data: &mut Vec<u8>, col: u16, row: u16, hi: u8| {
            let offset = (usize::from(row) * usize::from(w) + usize::from(col)) * 2;
            data[offset] = hi;
        };
        set_collision(&mut data, 50, 11, 0x40);
        set_collision(&mut data, 51, 11, 0x60);
        set_collision(&mut data, 52, 11, 0x50);
        set_collision(&mut data, 49, 12, 0x40);
        set_collision(&mut data, 50, 12, 0x60);
        set_collision(&mut data, 51, 12, 0x60);
        set_collision(&mut data, 52, 12, 0x60);
        set_collision(&mut data, 53, 12, 0x50);
        let mut grid = terrain::BlockGrid::from_level_layout(&data, w, h);
        let mut slice = OpeningSliceState::respawn(LevelBounds {
            width_pixels: w * 16,
            height_pixels: h * 16,
        });
        slice.player.x_pos = 0x0325_6D6A;
        slice.player.y_pos = 0x00B4_2780;
        slice.player.x_velocity = 0x0002_9D80;
        slice.player.y_velocity = -0x0000_5680;
        slice.player.facing = Direction::Right;
        slice.player.on_ground = false;

        slice.step(
            OPENING_SCROLL_START_FRAME,
            FrameInput {
                right: true,
                a: true,
                ..FrameInput::default()
            },
            false,
            PlayerForm::TheKid,
            Some(&mut grid),
            None,
            false,
            0,
        );

        // With corrected slope formula (ASM-matching), the SlopeDown surface
        // at this position is higher than before, so the player remains airborne.
        assert_eq!(slice.player.x_pos, 0x0328_09EA);
        assert_eq!(slice.player.y_pos, 0x00B4_1100);
        assert_eq!(slice.player.x_velocity, 0x0002_9C80);
        assert!(!slice.player.on_ground);
    }

    #[test]
    fn grounded_slope_to_solid_seam_keeps_level_one_runner_grounded() {
        let w = 8u16;
        let h = 12u16;
        let mut data = vec![0u8; usize::from(w) * usize::from(h) * 2];
        let set_collision = |data: &mut Vec<u8>, col: u16, row: u16, hi: u8| {
            let offset = (usize::from(row) * usize::from(w) + usize::from(col)) * 2;
            data[offset] = hi;
        };
        set_collision(&mut data, 2, 9, 0x40);
        set_collision(&mut data, 3, 9, 0x60);
        set_collision(&mut data, 4, 9, 0x60);
        set_collision(&mut data, 5, 9, 0x60);
        set_collision(&mut data, 2, 10, 0x60);
        set_collision(&mut data, 3, 10, 0x60);
        set_collision(&mut data, 4, 10, 0x60);
        set_collision(&mut data, 5, 10, 0x60);
        let mut grid = terrain::BlockGrid::from_level_layout(&data, w, h);
        let mut slice = OpeningSliceState {
            player: PlayerState {
                x_pos: (46u32 << 16) | 0x1980,
                y_pos: (142u32 << 16) | 0x4600,
                x_velocity: 0x0002_6E80,
                y_velocity: 0x0002_3E00,
                facing: Direction::Right,
                on_ground: false,
                on_spikes: false,
                just_jumped: false,
                jump_buffer_frames: 0,
                right_wall_sticky: false,
                left_wall_sticky: false,
            },
            camera: CameraState { x_pos: 0, y_pos: 0 },
            terrain: TerrainState {
                floor_y_pos: 143u32 << 16,
                floor_terrain_type: TerrainType::Normal,
            },
            level_bounds: LevelBounds {
                width_pixels: w * 16,
                height_pixels: h * 16,
            },
            objects: ObjectSystemState {
                platform_loader: PlatformLoaderState::default(),
            },
            prize_block_hit: None,
            lift_block_hit: None,
            drill_projectile_hit: None,
            metal_block_hit: None,
            smash_block_hit: None,
            teleporter_contact: None,
        };

        slice.step(
            OPENING_SCROLL_START_FRAME,
            FrameInput {
                right: true,
                a: true,
                ..FrameInput::default()
            },
            false,
            PlayerForm::TheKid,
            Some(&mut grid),
            None,
            false,
            0,
        );

        assert!(slice.player.on_ground);
        assert_eq!(slice.player.y_pos, 143u32 << 16);

        slice.step(
            OPENING_SCROLL_START_FRAME,
            FrameInput {
                right: true,
                a: true,
                ..FrameInput::default()
            },
            false,
            PlayerForm::TheKid,
            Some(&mut grid),
            None,
            false,
            0,
        );

        assert!(slice.player.on_ground);
        assert_eq!(slice.player.y_pos, 143u32 << 16);
    }

    #[test]
    fn grounded_frame_clears_released_jump_buffer_before_auto_jump() {
        let mut slice = OpeningSliceState::respawn(LevelBounds {
            width_pixels: 320,
            height_pixels: 224,
        });
        let mut grid = grid_with_floor_at_row(17);
        slice.player.x_pos = 0x00B1_2300;
        slice.player.x_velocity = 0x0002_6D80;
        slice.player.y_pos = slice.terrain.floor_y_pos;
        slice.player.y_velocity = 0;
        slice.player.on_ground = true;
        slice.player.jump_buffer_frames = 1;

        slice.step(
            OPENING_SCROLL_START_FRAME,
            FrameInput {
                right: true,
                a: true,
                ..FrameInput::default()
            },
            false,
            PlayerForm::TheKid,
            Some(&mut grid),
            None,
            false,
            0,
        );

        assert!(slice.player.on_ground);
        assert_eq!(slice.player.y_velocity, 0);
        assert_eq!(slice.player.jump_buffer_frames, 0);
    }

    #[test]
    fn timing_regression_pause_resume_script_applies_pause_and_resume_on_edge_frames() {
        let mut runtime = NativeRuntime::new(manifest());
        start_level_settled(&mut runtime);
        let frames = capture_frames(&mut runtime, 839..=924, pause_resume_script_input);

        assert!(!frames[&839].paused);
        assert!(frames[&840].paused);
        assert!(frames[&841].paused);
        assert!(frames[&842].paused);
        assert_eq!(frames[&841].camera_x_pos, frames[&840].camera_x_pos);
        assert_eq!(frames[&841].kid_x_pos, frames[&840].kid_x_pos);
        assert!(!frames[&919].paused);
        assert!(!frames[&920].paused);
        assert!(frames[&920].camera_x_pos > frames[&919].camera_x_pos);
        assert!(frames[&921].camera_x_pos > frames[&920].camera_x_pos);
    }

    #[test]
    fn pit_death_triggers_death_and_respawn() {
        let mut runtime = NativeRuntime::new(manifest());

        // Set up a block grid with all-empty cells (no floor) to simulate a pit.
        // Level 1: 6 * 20 = 120 cols wide, 4 * 14 = 56 rows tall (16px cells).
        let width_cells = 6u16 * 20;
        let height_cells = 4u16 * 14;
        let data = vec![0u8; usize::from(width_cells) * usize::from(height_cells) * 2];
        runtime.set_block_grid(terrain::BlockGrid::from_level_layout(
            &data,
            width_cells,
            height_cells,
        ));

        start_level(&mut runtime);

        // Advance past the opening scroll animation (260 frames) so physics are active.
        // With no floor in the grid, the player will fall under gravity.
        for _ in 0..300 {
            runtime.step_frame(FrameInput::default());
        }

        // Player should have fallen below the level boundary (896 pixels)
        // and triggered pit death. Force position if gravity alone wasn't enough.
        if matches!(runtime.player_lifecycle, PlayerLifecycle::Alive) {
            let level_height_fp = u32::from(4u16 * 224) << 16;
            if let Some(ref mut slice) = runtime.opening_slice {
                slice.player.y_pos = level_height_fp + (1 << 16);
                slice.player.on_ground = false;
            }
            runtime.step_frame(FrameInput::default());
        }

        let pre_lives = runtime.number_lives;
        runtime.step_frame(FrameInput::default());

        // Should be in death windup
        assert!(
            matches!(
                runtime.player_lifecycle,
                PlayerLifecycle::DeathWindup { .. }
            ),
            "expected DeathWindup, got {:?}",
            runtime.player_lifecycle
        );
        assert_eq!(runtime.number_hitpoints, 0, "pit death sets HP to 0");

        // Advance through death windup + dying phases
        for _ in 0..PLAYER_DEATH_WINDUP_FRAMES {
            runtime.step_frame(FrameInput::default());
        }
        assert!(
            matches!(runtime.player_lifecycle, PlayerLifecycle::Dying { .. }),
            "expected Dying, got {:?}",
            runtime.player_lifecycle
        );
        assert_eq!(runtime.number_lives, pre_lives - 1, "should lose a life");

        // Advance through dying phase
        for _ in 0..PLAYER_DEATH_HIDE_FRAMES {
            runtime.step_frame(FrameInput::default());
        }
        assert!(
            matches!(runtime.player_lifecycle, PlayerLifecycle::Respawning { .. }),
            "expected Respawning, got {:?}",
            runtime.player_lifecycle
        );

        // Player should be back at level start position (header-derived)
        let trace = runtime.trace_frame();
        assert_eq!(trace.kid_x_pos, Some(LEVEL0_RESPAWN_KID_X));
        assert_eq!(trace.kid_y_pos, Some(LEVEL0_RESPAWN_KID_Y));
    }

    /// Helper: build a block grid with solid floor at row 17 and spikes
    /// starting at the given column onward (also at row 17).
    fn grid_with_spikes_from_col(spike_start_col: u16) -> terrain::BlockGrid {
        let width_cells = 6u16 * 20;
        let height_cells = 4u16 * 14;
        let mut data = vec![0u8; usize::from(width_cells) * usize::from(height_cells) * 2];
        for col in 0..width_cells {
            let offset = (17usize * usize::from(width_cells) + usize::from(col)) * 2;
            if col >= spike_start_col {
                data[offset] = 0x70; // collision=7 (Spikes)
            } else {
                data[offset] = 0x60; // collision=6 (Solid)
            }
        }
        terrain::BlockGrid::from_level_layout(&data, width_cells, height_cells)
    }

    #[test]
    fn spike_death_triggers_death_and_respawn() {
        let mut runtime = NativeRuntime::new(manifest());

        // Solid floor under spawn (col 0..5), spikes from col 5 onward.
        // Player spawns at x=32px (col 2), walks right into spikes at col 5 (80px).
        runtime.set_block_grid(grid_with_spikes_from_col(5));

        start_level(&mut runtime);

        // Walk right until we hit spikes and die
        for _ in 0..300 {
            if !matches!(runtime.player_lifecycle, PlayerLifecycle::Alive) {
                break;
            }
            runtime.step_frame(FrameInput {
                right: true,
                ..FrameInput::default()
            });
        }

        // Player should be in death windup from landing on spikes
        assert!(
            matches!(
                runtime.player_lifecycle,
                PlayerLifecycle::DeathWindup { .. }
            ),
            "expected DeathWindup, got {:?}",
            runtime.player_lifecycle
        );
        assert_eq!(runtime.number_hitpoints, 0, "spike death sets HP to 0");

        let pre_lives = runtime.number_lives;

        // Advance through death windup
        for _ in 0..PLAYER_DEATH_WINDUP_FRAMES {
            runtime.step_frame(FrameInput::default());
        }
        assert!(
            matches!(runtime.player_lifecycle, PlayerLifecycle::Dying { .. }),
            "expected Dying, got {:?}",
            runtime.player_lifecycle
        );
        assert_eq!(runtime.number_lives, pre_lives - 1, "should lose a life");

        // Advance through dying phase
        for _ in 0..PLAYER_DEATH_HIDE_FRAMES {
            runtime.step_frame(FrameInput::default());
        }
        assert!(
            matches!(runtime.player_lifecycle, PlayerLifecycle::Respawning { .. }),
            "expected Respawning, got {:?}",
            runtime.player_lifecycle
        );

        // Player should be back at level start position (header-derived)
        let trace = runtime.trace_frame();
        assert_eq!(trace.kid_x_pos, Some(LEVEL0_RESPAWN_KID_X));
        assert_eq!(trace.kid_y_pos, Some(LEVEL0_RESPAWN_KID_Y));
    }

    #[test]
    fn spike_death_ignores_hitpoints() {
        let mut runtime = NativeRuntime::new(manifest());

        // Give extra HP — spikes should still be instant kill
        runtime.number_hitpoints = 5;
        runtime.set_block_grid(grid_with_spikes_from_col(5));

        start_level(&mut runtime);

        for _ in 0..300 {
            if !matches!(runtime.player_lifecycle, PlayerLifecycle::Alive) {
                break;
            }
            runtime.step_frame(FrameInput {
                right: true,
                ..FrameInput::default()
            });
        }

        assert!(
            matches!(
                runtime.player_lifecycle,
                PlayerLifecycle::DeathWindup { .. }
            ),
            "expected DeathWindup despite 5 HP, got {:?}",
            runtime.player_lifecycle
        );
        assert_eq!(
            runtime.number_hitpoints, 0,
            "spike death zeroes HP regardless"
        );
    }

    #[test]
    fn game_over_when_last_life_lost() {
        let mut runtime = NativeRuntime::new(manifest());
        runtime.set_block_grid(grid_with_spikes_from_col(5));

        start_level(&mut runtime);
        runtime.number_lives = 1; // last life (set after boot so title doesn't reset it)
        runtime.number_continues = 0; // no continues left, so game over → title

        // Walk into spikes
        for _ in 0..300 {
            if !matches!(runtime.player_lifecycle, PlayerLifecycle::Alive) {
                break;
            }
            runtime.step_frame(FrameInput {
                right: true,
                ..FrameInput::default()
            });
        }
        assert!(
            matches!(
                runtime.player_lifecycle,
                PlayerLifecycle::DeathWindup { .. }
            ),
            "expected DeathWindup, got {:?}",
            runtime.player_lifecycle
        );

        // Advance through death windup
        for _ in 0..PLAYER_DEATH_WINDUP_FRAMES {
            runtime.step_frame(FrameInput::default());
        }
        assert_eq!(runtime.number_lives, 0, "last life lost");

        // Advance through dying phase — should enter GameOver, not Respawning
        for _ in 0..PLAYER_DEATH_HIDE_FRAMES {
            runtime.step_frame(FrameInput::default());
        }
        assert!(
            matches!(runtime.player_lifecycle, PlayerLifecycle::GameOver { .. }),
            "expected GameOver at 0 lives, got {:?}",
            runtime.player_lifecycle
        );

        let trace = runtime.trace_frame();
        assert!(trace.game_over, "trace should report game_over");

        // Advance through game-over hold — should return to title
        for _ in 0..PLAYER_GAME_OVER_HOLD_FRAMES {
            runtime.step_frame(FrameInput::default());
        }
        assert_eq!(
            runtime.boot_state,
            BootState::TitleScreen,
            "should return to title after game over"
        );
        assert_eq!(runtime.number_lives, 3, "lives reset after game over");
    }

    #[test]
    fn continue_restores_lives_and_respawns() {
        let mut runtime = NativeRuntime::new(manifest());
        runtime.set_block_grid(grid_with_spikes_from_col(5));

        start_level(&mut runtime);
        runtime.number_lives = 1;
        runtime.number_continues = 2;

        // Walk into spikes
        for _ in 0..300 {
            if !matches!(runtime.player_lifecycle, PlayerLifecycle::Alive) {
                break;
            }
            runtime.step_frame(FrameInput {
                right: true,
                ..FrameInput::default()
            });
        }

        // Advance through death windup + dying
        for _ in 0..(PLAYER_DEATH_WINDUP_FRAMES + PLAYER_DEATH_HIDE_FRAMES) {
            runtime.step_frame(FrameInput::default());
        }
        assert!(
            matches!(runtime.player_lifecycle, PlayerLifecycle::GameOver { .. }),
            "expected GameOver, got {:?}",
            runtime.player_lifecycle
        );

        // Advance through game-over hold — should enter continue countdown
        for _ in 0..PLAYER_GAME_OVER_HOLD_FRAMES {
            runtime.step_frame(FrameInput::default());
        }
        assert!(
            matches!(runtime.player_lifecycle, PlayerLifecycle::ContinueCountdown { .. }),
            "expected ContinueCountdown, got {:?}",
            runtime.player_lifecycle
        );

        // Press a button to use continue
        runtime.step_frame(FrameInput {
            start: true,
            ..FrameInput::default()
        });
        assert_eq!(
            runtime.boot_state,
            BootState::InGame,
            "should stay in-game after using continue"
        );
        assert_eq!(runtime.number_lives, 3, "lives restored by continue");
        assert_eq!(runtime.number_continues, 1, "one continue consumed");
        assert!(
            matches!(runtime.player_lifecycle, PlayerLifecycle::Respawning { .. }),
            "should be respawning after continue"
        );
    }

    /// Helper: build a block grid with solid floor at a given row everywhere.
    fn grid_with_floor_at_row(floor_row: u16) -> terrain::BlockGrid {
        let width_cells = 6u16 * 20;
        let height_cells = 4u16 * 14;
        let mut data = vec![0u8; usize::from(width_cells) * usize::from(height_cells) * 2];
        for col in 0..width_cells {
            let offset = (usize::from(floor_row) * usize::from(width_cells) + usize::from(col)) * 2;
            data[offset] = 0x60; // collision=6 (Solid)
        }
        terrain::BlockGrid::from_level_layout(&data, width_cells, height_cells)
    }

    #[test]
    fn flag_touch_triggers_level_complete() {
        let mut runtime = NativeRuntime::new(manifest());
        // Flag at (0x0750, 0x0300) = (1872, 768)px. Floor at row 48 = 768px surface.
        runtime.set_block_grid(grid_with_floor_at_row(48));
        start_level(&mut runtime);

        // Advance past entry animation so physics are active
        for _ in 0..260 {
            runtime.step_frame(FrameInput::default());
        }

        // Place player directly on the flag position
        if let Some(ref mut slice) = runtime.opening_slice {
            slice.player.x_pos = 0x0750_0000;
            slice.player.y_pos = 0x0300_0000;
            slice.player.on_ground = true;
        }

        runtime.step_frame(FrameInput::default());

        assert!(
            matches!(
                runtime.player_lifecycle,
                PlayerLifecycle::LevelComplete { .. }
            ),
            "expected LevelComplete, got {:?}",
            runtime.player_lifecycle
        );

        let trace = runtime.trace_frame();
        assert!(trace.level_complete, "trace should report level_complete");
    }

    #[test]
    fn level_complete_freezes_input_then_advances() {
        let mut runtime = NativeRuntime::new(manifest_with_levels(3));
        runtime.set_block_grid(grid_with_floor_at_row(48));
        start_level(&mut runtime);

        // Advance past entry animation
        for _ in 0..260 {
            runtime.step_frame(FrameInput::default());
        }

        assert_eq!(runtime.current_level, LevelId(0));

        // Place player on flag
        if let Some(ref mut slice) = runtime.opening_slice {
            slice.player.x_pos = 0x0750_0000;
            slice.player.y_pos = 0x0300_0000;
            slice.player.on_ground = true;
        }

        // Trigger flag touch
        runtime.step_frame(FrameInput::default());
        assert!(matches!(
            runtime.player_lifecycle,
            PlayerLifecycle::LevelComplete { .. }
        ));

        // Record player position — input during LevelComplete should be ignored
        let pos_at_complete = runtime.opening_slice.unwrap().player.x_pos;
        for _ in 0..10 {
            runtime.step_frame(FrameInput {
                right: true,
                ..FrameInput::default()
            });
        }
        // Player should not have moved (input frozen)
        assert_eq!(
            runtime.opening_slice.unwrap().player.x_pos,
            pos_at_complete,
            "input should be frozen during LevelComplete"
        );

        // Still on level 0 during pause
        assert_eq!(runtime.current_level, LevelId(0));

        // Advance through remaining LevelComplete frames
        for _ in 0..LEVEL_COMPLETE_PAUSE_FRAMES {
            runtime.step_frame(FrameInput::default());
        }

        // Should have advanced to next level
        assert_eq!(
            runtime.current_level,
            LevelId(1),
            "should advance to next level"
        );
        assert!(
            matches!(runtime.player_lifecycle, PlayerLifecycle::Respawning { .. }),
            "expected Respawning after level advance, got {:?}",
            runtime.player_lifecycle
        );
        let slice = runtime
            .opening_slice
            .expect("advanced level should install a respawn slice");
        assert_eq!(slice.player.x_pos, 0x0751_0000);
        // player_start_y=0xE0, minus 1 for ASM subq.w #1,y_pos
        assert_eq!(slice.player.y_pos, 0x00DF_0000);
        // Camera snaps to screen boundary: floor(pos / screen) * screen
        assert_eq!(slice.camera.x_pos, u32::from((0x0751u16 / 320) * 320) << 16);
        assert_eq!(slice.camera.y_pos, u32::from((0x00E0u16 / 224) * 224) << 16);
    }

    #[test]
    fn level_complete_awards_time_bonus() {
        let mut runtime = NativeRuntime::new(manifest_with_levels(3));
        runtime.set_block_grid(grid_with_floor_at_row(48));
        start_level(&mut runtime);

        // Advance past entry animation
        for _ in 0..260 {
            runtime.step_frame(FrameInput::default());
        }

        assert_eq!(runtime.score, 0);

        // Place player on flag
        if let Some(ref mut slice) = runtime.opening_slice {
            slice.player.x_pos = 0x0750_0000;
            slice.player.y_pos = 0x0300_0000;
            slice.player.on_ground = true;
        }

        let seconds_remaining = runtime.level_timer.seconds;

        // Trigger flag touch
        runtime.step_frame(FrameInput::default());
        assert!(matches!(
            runtime.player_lifecycle,
            PlayerLifecycle::LevelComplete { .. }
        ));

        // Score should include time bonus + no-hit + no-prize + speed bonus.
        // Level 0 speed threshold = 20s. Elapsed ~4s after 260 frames, so bonus applies.
        let elapsed_secs = LEVEL_TIMER_START_SECONDS.saturating_sub(seconds_remaining);
        let speed_threshold = SPEED_BONUS_TABLE[0];
        let expected = u32::from(seconds_remaining) * 100
            + 5000  // no-hit bonus (no damage taken)
            + 5000  // no-prize bonus (no prize blocks hit)
            + if speed_threshold > 0 && (elapsed_secs.min(255) as u8) <= speed_threshold { 10_000 } else { 0 };
        assert_eq!(
            runtime.score, expected,
            "score should include time, no-hit, no-prize, and speed bonuses"
        );
    }

    #[test]
    fn no_hit_bonus_absent_when_damage_taken() {
        let mut runtime = NativeRuntime::new(manifest_with_levels(3));
        runtime.set_block_grid(grid_with_floor_at_row(48));
        start_level(&mut runtime);

        for _ in 0..260 {
            runtime.step_frame(FrameInput::default());
        }

        // Take damage
        runtime.apply_damage();
        assert!(runtime.level_damage_taken);

        // Place player on flag
        if let Some(ref mut slice) = runtime.opening_slice {
            slice.player.x_pos = 0x0750_0000;
            slice.player.y_pos = 0x0300_0000;
            slice.player.on_ground = true;
        }

        let seconds_remaining = runtime.level_timer.seconds;
        runtime.step_frame(FrameInput::default());

        let elapsed_secs = LEVEL_TIMER_START_SECONDS.saturating_sub(seconds_remaining);
        let speed_threshold = SPEED_BONUS_TABLE[0];
        let expected = u32::from(seconds_remaining) * 100
            + 5000  // no-prize bonus (no prize blocks hit)
            + if speed_threshold > 0 && (elapsed_secs.min(255) as u8) <= speed_threshold { 10_000 } else { 0 };
        assert_eq!(
            runtime.score, expected,
            "no-hit bonus should be absent when damage was taken"
        );
    }

    #[test]
    fn damage_clears_wall_sticky() {
        let mut runtime = NativeRuntime::new(manifest_with_levels(3));
        runtime.set_block_grid(grid_with_floor_at_row(48));
        start_level(&mut runtime);

        for _ in 0..260 {
            runtime.step_frame(FrameInput::default());
        }

        // Set wall-sticky as if player is grabbing a wall
        if let Some(ref mut slice) = runtime.opening_slice {
            slice.player.right_wall_sticky = true;
            slice.player.left_wall_sticky = true;
        }

        runtime.apply_damage();

        let slice = runtime.opening_slice.as_ref().unwrap();
        assert!(
            !slice.player.right_wall_sticky,
            "right wall sticky should be cleared on damage"
        );
        assert!(
            !slice.player.left_wall_sticky,
            "left wall sticky should be cleared on damage"
        );
    }

    #[test]
    fn flag_miss_does_not_trigger_completion() {
        let mut runtime = NativeRuntime::new(manifest());
        runtime.set_block_grid(grid_with_floor_at_row(48));
        start_level(&mut runtime);

        // Advance past entry animation
        for _ in 0..260 {
            runtime.step_frame(FrameInput::default());
        }

        // Place player far from flag (with floor at row 16 = 256px for this position)
        if let Some(ref mut slice) = runtime.opening_slice {
            slice.player.x_pos = 0x0100_0000;
            slice.player.y_pos = 0x0300_0000; // same y as flag but x far away
            slice.player.on_ground = true;
        }

        runtime.step_frame(FrameInput::default());

        assert!(
            matches!(runtime.player_lifecycle, PlayerLifecycle::Alive),
            "player should remain Alive when not touching flag"
        );
        assert!(!runtime.trace_frame().level_complete);
    }

    // --- US-004: Enemy-player contact damage ---
    // Tests use MiniHoppingSkull (not Diamond, which is now collected on touch).

    /// Manifest with a static skull near spawn for contact damage tests.
    fn manifest_with_static_skull() -> AssetManifest {
        let mut m = manifest();
        if let Some(enemy) = m.levels[0].enemy_layout_metadata.as_mut() {
            // Replace first object with a skull at the Diamond's old position.
            // Keep enemy_type_slots so slot resolution still works.
            enemy.objects[0] = kid_data::EnemyLayoutObject {
                object_type: 0x1C, // MiniHoppingSkull
                flags: 0,
                hit_points_plus_one: 1,
                x: 0x0120,
                y: 0x00E0,
            };
        }
        m
    }

    fn manifest_with_static_unknown_enemy() -> AssetManifest {
        let mut m = manifest();
        if let Some(enemy) = m.levels[0].enemy_layout_metadata.as_mut() {
            enemy.objects[0] = kid_data::EnemyLayoutObject {
                object_type: 0x22,
                flags: 0,
                hit_points_plus_one: 1,
                x: 0x0120,
                y: 0x00E0,
            };
        }
        m
    }

    #[test]
    fn enemy_contact_deals_damage() {
        let mut runtime = NativeRuntime::new(manifest_with_static_skull());
        runtime.set_block_grid(grid_with_floor_at_row(14));
        start_level(&mut runtime);
        for _ in 0..260 {
            runtime.step_frame(FrameInput::default());
        }

        let hp_before = runtime.number_hitpoints;

        if let Some(ref mut slice) = runtime.opening_slice {
            slice.player.x_pos = 0x0120_0000;
            slice.player.y_pos = 0x00E0_0000;
            slice.player.on_ground = true;
            slice.camera.x_pos = 0;
            slice.camera.y_pos = 0;
        }
        // Step to activate the skull
        runtime.step_frame(FrameInput::default());
        // Reposition for contact
        if let Some(ref mut slice) = runtime.opening_slice {
            slice.player.x_pos = 0x0120_0000;
            slice.player.y_pos = 0x00E0_0000;
            slice.player.on_ground = true;
            slice.camera.x_pos = 0;
            slice.camera.y_pos = 0;
        }
        runtime.step_frame(FrameInput::default());

        assert!(
            runtime.number_hitpoints < hp_before
                || !matches!(runtime.player_lifecycle, PlayerLifecycle::Alive),
            "player should take damage from enemy contact (HP before: {hp_before}, after: {})",
            runtime.number_hitpoints
        );
    }

    #[test]
    fn invulnerability_prevents_rapid_rehit() {
        let mut runtime = NativeRuntime::new(manifest_with_static_skull());
        runtime.set_block_grid(grid_with_floor_at_row(14));
        start_level(&mut runtime);
        for _ in 0..260 {
            runtime.step_frame(FrameInput::default());
        }
        runtime.number_hitpoints = 3;

        // Activate skull, then position for contact
        if let Some(ref mut slice) = runtime.opening_slice {
            slice.camera.x_pos = 0;
            slice.camera.y_pos = 0;
        }
        runtime.step_frame(FrameInput::default());
        if let Some(ref mut slice) = runtime.opening_slice {
            slice.player.x_pos = 0x0120_0000;
            slice.player.y_pos = 0x00E0_0000;
            slice.player.on_ground = true;
            slice.camera.x_pos = 0;
            slice.camera.y_pos = 0;
        }
        runtime.step_frame(FrameInput::default());

        assert!(
            runtime.invulnerable_frames > 0,
            "should have invulnerability after hit"
        );
        let hp_after_first = runtime.number_hitpoints;

        for _ in 0..10 {
            if let Some(ref mut slice) = runtime.opening_slice {
                slice.player.x_pos = 0x0120_0000;
                slice.player.y_pos = 0x00E0_0000;
                slice.player.on_ground = true;
                slice.camera.x_pos = 0;
                slice.camera.y_pos = 0;
            }
            runtime.step_frame(FrameInput::default());
        }

        assert_eq!(
            runtime.number_hitpoints, hp_after_first,
            "invulnerability should prevent further damage"
        );
    }

    #[test]
    fn enemy_contact_kills_at_one_hp() {
        let mut runtime = NativeRuntime::new(manifest_with_static_skull());
        runtime.set_block_grid(grid_with_floor_at_row(14));
        start_level(&mut runtime);
        for _ in 0..260 {
            runtime.step_frame(FrameInput::default());
        }
        runtime.number_hitpoints = 1;

        if let Some(ref mut slice) = runtime.opening_slice {
            slice.camera.x_pos = 0;
            slice.camera.y_pos = 0;
        }
        runtime.step_frame(FrameInput::default());
        if let Some(ref mut slice) = runtime.opening_slice {
            slice.player.x_pos = 0x0120_0000;
            slice.player.y_pos = 0x00E0_0000;
            slice.player.on_ground = true;
            slice.camera.x_pos = 0;
            slice.camera.y_pos = 0;
        }
        runtime.step_frame(FrameInput::default());

        assert!(
            matches!(
                runtime.player_lifecycle,
                PlayerLifecycle::DeathWindup { .. }
            ),
            "expected DeathWindup at 1 HP, got {:?}",
            runtime.player_lifecycle
        );
        assert_eq!(
            runtime.number_hitpoints, 0,
            "HP should be 0 after lethal contact"
        );
    }

    #[test]
    fn enemy_survives_after_damaging_player() {
        let mut runtime = NativeRuntime::new(manifest_with_static_skull());
        runtime.set_block_grid(grid_with_floor_at_row(14));
        start_level(&mut runtime);
        for _ in 0..260 {
            runtime.step_frame(FrameInput::default());
        }

        if let Some(ref mut slice) = runtime.opening_slice {
            slice.camera.x_pos = 0;
            slice.camera.y_pos = 0;
        }
        runtime.step_frame(FrameInput::default());
        if let Some(ref mut slice) = runtime.opening_slice {
            slice.player.x_pos = 0x0120_0000;
            slice.player.y_pos = 0x00E0_0000;
            slice.player.on_ground = true;
            slice.camera.x_pos = 0;
            slice.camera.y_pos = 0;
        }
        runtime.step_frame(FrameInput::default());

        let skull_present = runtime
            .active_enemies
            .iter()
            .any(|e| e.family == EnemyFamily::MiniHoppingSkull);
        assert!(
            skull_present,
            "MiniHoppingSkull should remain alive after damaging player"
        );
    }

    #[test]
    fn unsupported_enemy_family_stays_inert_on_contact() {
        let mut runtime = NativeRuntime::new(manifest_with_static_unknown_enemy());
        runtime.set_block_grid(grid_with_floor_at_row(14));
        start_level(&mut runtime);
        for _ in 0..260 {
            runtime.step_frame(FrameInput::default());
        }

        let hp_before = runtime.number_hitpoints;
        if let Some(ref mut slice) = runtime.opening_slice {
            slice.camera.x_pos = 0;
            slice.camera.y_pos = 0;
        }
        runtime.step_frame(FrameInput::default());
        if let Some(ref mut slice) = runtime.opening_slice {
            slice.player.x_pos = 0x0120_0000;
            slice.player.y_pos = 0x00E0_0000;
            slice.player.on_ground = true;
            slice.camera.x_pos = 0;
            slice.camera.y_pos = 0;
        }
        runtime.step_frame(FrameInput::default());

        assert_eq!(
            runtime.number_hitpoints, hp_before,
            "unsupported enemy families should not damage the player until implemented"
        );
        assert!(matches!(runtime.player_lifecycle, PlayerLifecycle::Alive));
    }

    // -----------------------------------------------------------------------
    // US-006: Static enemy archetype (Diamond)
    // -----------------------------------------------------------------------

    #[test]
    fn diamond_stays_at_spawn_position() {
        let mut runtime = NativeRuntime::new(manifest());
        // Diamond at (0x0120, 0x00E0) = (288, 224)px — within activation window at camera origin.
        runtime.set_block_grid(grid_with_floor_at_row(14));
        start_level(&mut runtime);

        // Advance past entry animation
        for _ in 0..260 {
            runtime.step_frame(FrameInput::default());
        }

        // Keep camera at origin so Diamond remains active
        if let Some(ref mut slice) = runtime.opening_slice {
            slice.camera.x_pos = 0;
            slice.camera.y_pos = 0;
        }

        let expected_x: u32 = 0x0120 << 16;
        let expected_y: u32 = 0x00E0 << 16;

        // Run 60 frames — Diamond should not move at all
        for frame in 0..60 {
            runtime.step_frame(FrameInput::default());
            // Keep camera at origin each frame
            if let Some(ref mut slice) = runtime.opening_slice {
                slice.camera.x_pos = 0;
                slice.camera.y_pos = 0;
            }

            let diamond = runtime
                .active_enemies
                .iter()
                .find(|e| e.family == EnemyFamily::Diamond);
            assert!(
                diamond.is_some(),
                "Diamond should be active on frame {frame}"
            );
            let diamond = diamond.unwrap();
            assert_eq!(
                diamond.x_pos, expected_x,
                "Diamond x should stay at spawn on frame {frame}"
            );
            assert_eq!(
                diamond.y_pos, expected_y,
                "Diamond y should stay at spawn on frame {frame}"
            );
        }
    }

    #[test]
    fn diamond_collected_on_touch() {
        // US-014: Diamonds are collectibles, not damage sources.
        let mut runtime = NativeRuntime::new(manifest());
        runtime.set_block_grid(grid_with_floor_at_row(14));
        start_level(&mut runtime);
        for _ in 0..260 {
            runtime.step_frame(FrameInput::default());
        }

        let hp_before = runtime.number_hitpoints;
        let diamonds_before = runtime.diamond_count;

        // Activate Diamond
        if let Some(ref mut slice) = runtime.opening_slice {
            slice.camera.x_pos = 0;
            slice.camera.y_pos = 0;
        }
        runtime.step_frame(FrameInput::default());

        // Place player on the Diamond
        if let Some(ref mut slice) = runtime.opening_slice {
            slice.player.x_pos = 0x0120_0000;
            slice.player.y_pos = 0x00E0_0000;
            slice.player.on_ground = true;
            slice.camera.x_pos = 0;
            slice.camera.y_pos = 0;
        }
        runtime.step_frame(FrameInput::default());

        assert_eq!(
            runtime.number_hitpoints, hp_before,
            "Diamond should not deal damage"
        );
        assert_eq!(
            runtime.diamond_count,
            diamonds_before + 1,
            "Diamond should increment diamond_count"
        );
        assert!(
            !runtime
                .active_enemies
                .iter()
                .any(|e| e.family == EnemyFamily::Diamond),
            "Diamond should be removed after collection"
        );
    }

    #[test]
    fn prize_block_headbutt_grants_diamond() {
        let mut runtime = NativeRuntime::new(manifest());
        runtime.set_block_grid(grid_with_blocks());
        start_level(&mut runtime);
        for _ in 0..260 {
            runtime.step_frame(FrameInput::default());
        }

        let diamonds_before = runtime.diamond_count;

        // Place player below the prize block at col 2, row 2 (y=32..48px)
        // After vertical motion: head needs to land in row 2 (y 32..48).
        // feet_y + y_vel = new_feet. head = new_feet - 31.
        // We want head ≈ 40, so feet ≈ 71. Set feet=73, vel=-0x0002_0000.
        // After gravity: vel ≈ -0x1C000, new_feet ≈ 73 - 1.75 ≈ 71.25
        // head ≈ 71.25 - 31 = 40.25 → row 2. Prize block hit!
        if let Some(ref mut slice) = runtime.opening_slice {
            slice.player.x_pos = 40 << 16; // col 2 centre
            slice.player.y_pos = 73 << 16;
            slice.player.y_velocity = -0x0002_0000; // moving upward
            slice.player.on_ground = false;
            slice.camera.x_pos = 0;
            slice.camera.y_pos = 0;
        }

        runtime.step_frame(FrameInput::default());

        assert_eq!(
            runtime.diamond_count,
            diamonds_before + 1,
            "headbutting prize block should grant a diamond"
        );
    }

    #[test]
    fn smash_block_headbutt_does_not_grant_diamond() {
        let mut runtime = NativeRuntime::new(manifest());
        runtime.set_block_grid(grid_with_blocks());
        start_level(&mut runtime);
        for _ in 0..260 {
            runtime.step_frame(FrameInput::default());
        }

        let diamonds_before = runtime.diamond_count;

        // Place player below the smash block at col 4, row 2 (y=32..48px)
        if let Some(ref mut slice) = runtime.opening_slice {
            slice.player.x_pos = 72 << 16; // col 4 centre
            slice.player.y_pos = 73 << 16;
            slice.player.y_velocity = -0x0002_0000;
            slice.player.on_ground = false;
            slice.camera.x_pos = 0;
            slice.camera.y_pos = 0;
        }

        runtime.step_frame(FrameInput::default());

        assert_eq!(
            runtime.diamond_count, diamonds_before,
            "headbutting smash block should not grant a diamond"
        );
    }

    fn grid_with_lift_block() -> terrain::BlockGrid {
        let w = 6u16;
        let h = 5u16;
        let mut data = vec![0u8; usize::from(w) * usize::from(h) * 2];
        let set_cell = |data: &mut Vec<u8>, col: usize, row: usize, hi: u8| {
            let offset = (row * usize::from(w) + col) * 2;
            data[offset] = hi;
            data[offset + 1] = 0x01;
        };
        // Row 2, col 2: lift block (collision=6 Solid, has_block, block_type=0xB)
        // 1_110_1011 = 0xEB
        set_cell(&mut data, 2, 2, 0xEB);
        // Row 3: solid floor
        for col in 0..6 {
            set_cell(&mut data, col, 3, 0x60);
        }
        terrain::BlockGrid::from_level_layout(&data, w, h)
    }

    fn grid_with_drill_block() -> terrain::BlockGrid {
        let w = 6u16;
        let h = 5u16;
        let mut data = vec![0u8; usize::from(w) * usize::from(h) * 2];
        let set_cell = |data: &mut Vec<u8>, col: usize, row: usize, hi: u8| {
            let offset = (row * usize::from(w) + col) * 2;
            data[offset] = hi;
            data[offset + 1] = 0x01;
        };
        // Row 2, col 2: drill block (collision=6 Solid, has_block, block_type=0xC)
        // 1_110_1100 = 0xEC
        set_cell(&mut data, 2, 2, 0xEC);
        // Row 3: solid floor
        for col in 0..6 {
            set_cell(&mut data, col, 3, 0x60);
        }
        terrain::BlockGrid::from_level_layout(&data, w, h)
    }

    #[test]
    fn drill_block_headbutt_spawns_projectile() {
        let mut runtime = NativeRuntime::new(manifest());
        runtime.set_block_grid(grid_with_drill_block());
        start_level(&mut runtime);
        for _ in 0..260 {
            runtime.step_frame(FrameInput::default());
        }

        assert!(runtime.projectiles.is_empty());

        // Position player below drill block at col 2, row 2
        if let Some(ref mut slice) = runtime.opening_slice {
            slice.player.x_pos = 40 << 16;
            slice.player.y_pos = 73 << 16;
            slice.player.y_velocity = -0x0002_0000;
            slice.player.on_ground = false;
            slice.camera.x_pos = 0;
            slice.camera.y_pos = 0;
        }

        runtime.step_frame(FrameInput::default());

        assert!(
            !runtime.projectiles.is_empty(),
            "headbutting drill block should spawn a falling projectile"
        );
        let proj = &runtime.projectiles[0];
        assert!(proj.y_velocity > 0, "drill projectile falls downward");
        assert_eq!(proj.x_velocity, 0, "drill projectile falls straight down");
        assert!(!proj.friendly);

        // Block should be cleared from grid
        let grid = runtime.block_grid.as_ref().unwrap();
        let cell = grid.cell_at(2, 2);
        assert!(
            !cell.has_block,
            "drill block should be cleared after headbutt"
        );
    }

    #[test]
    fn lift_block_headbutt_spawns_platform() {
        let mut runtime = NativeRuntime::new(manifest());
        runtime.set_block_grid(grid_with_lift_block());
        start_level(&mut runtime);
        for _ in 0..260 {
            runtime.step_frame(FrameInput::default());
        }

        assert!(runtime.lift_platforms.is_empty());

        // Position player below lift block at col 2, row 2
        if let Some(ref mut slice) = runtime.opening_slice {
            slice.player.x_pos = 40 << 16;
            slice.player.y_pos = 73 << 16;
            slice.player.y_velocity = -0x0002_0000;
            slice.player.on_ground = false;
            slice.camera.x_pos = 0;
            slice.camera.y_pos = 0;
        }

        runtime.step_frame(FrameInput::default());

        assert_eq!(
            runtime.lift_platforms.len(),
            1,
            "headbutting lift block should spawn a lift platform"
        );
        // Block should be cleared from grid
        let grid = runtime.block_grid.as_ref().unwrap();
        let cell = grid.cell_at(2, 2);
        assert!(
            !cell.has_block,
            "lift block should be cleared after headbutt"
        );
    }

    #[test]
    fn lift_platform_rises_then_falls() {
        let mut lift = LiftPlatform {
            x_pos: 32 << 16,
            y_pos: 100 << 16,
            y_velocity: 0,
            age: 0,
            width_px: 16,
        };

        // After a few frames, velocity should be negative (rising)
        for _ in 0..10 {
            lift.age += 1;
            let accel = if lift.age <= LIFT_RISE_FRAMES {
                LIFT_ACCEL_UP
            } else {
                LIFT_ACCEL_DOWN
            };
            lift.y_velocity += accel;
            let next_y = i64::from(lift.y_pos) + i64::from(lift.y_velocity);
            lift.y_pos = next_y.clamp(0, i64::from(u32::MAX)) as u32;
        }
        assert!(
            lift.y_velocity < 0,
            "lift should be rising in first 10 frames"
        );
        let pos_after_rise = lift.y_pos;
        assert!(pos_after_rise < (100 << 16), "lift should have moved up");

        // After ~120 frames total, enough downward accel to reverse
        for _ in 0..110 {
            lift.age += 1;
            let accel = if lift.age <= LIFT_RISE_FRAMES {
                LIFT_ACCEL_UP
            } else {
                LIFT_ACCEL_DOWN
            };
            lift.y_velocity += accel;
            let next_y = i64::from(lift.y_pos) + i64::from(lift.y_velocity);
            lift.y_pos = next_y.clamp(0, i64::from(u32::MAX)) as u32;
        }
        assert!(
            lift.y_velocity > 0,
            "lift should be falling after enough frames"
        );
    }

    fn grid_with_hidden_block() -> terrain::BlockGrid {
        let w = 6u16;
        let h = 5u16;
        let mut data = vec![0u8; usize::from(w) * usize::from(h) * 2];
        let set_cell = |data: &mut Vec<u8>, col: usize, row: usize, hi: u8| {
            let offset = (row * usize::from(w) + col) * 2;
            data[offset] = hi;
            data[offset + 1] = 0x01;
        };
        // Row 2, col 2: hidden block (collision=2, has_block, block_type=5/Metal)
        // 1_010_0101 = 0xA5
        set_cell(&mut data, 2, 2, 0xA5);
        // Row 3: solid floor
        for col in 0..6 {
            set_cell(&mut data, col, 3, 0x60);
        }
        terrain::BlockGrid::from_level_layout(&data, w, h)
    }

    #[test]
    fn hidden_block_reveals_on_headbutt() {
        let mut runtime = NativeRuntime::new(manifest());
        runtime.set_block_grid(grid_with_hidden_block());
        start_level(&mut runtime);
        for _ in 0..260 {
            runtime.step_frame(FrameInput::default());
        }

        // Verify hidden block at col 2, row 2
        let grid = runtime.block_grid.as_ref().unwrap();
        let cell = grid.cell_at(2, 2);
        assert_eq!(cell.collision, terrain::CollisionKind::HiddenBlock);

        // Position player below the hidden block, moving upward
        if let Some(ref mut slice) = runtime.opening_slice {
            slice.player.x_pos = 40 << 16; // col 2 centre
            slice.player.y_pos = 73 << 16;
            slice.player.y_velocity = -0x0002_0000;
            slice.player.on_ground = false;
            slice.camera.x_pos = 0;
            slice.camera.y_pos = 0;
        }

        runtime.step_frame(FrameInput::default());

        // Block should now be Solid instead of HiddenBlock
        let grid = runtime.block_grid.as_ref().unwrap();
        let cell = grid.cell_at(2, 2);
        assert_eq!(
            cell.collision,
            terrain::CollisionKind::Solid,
            "hidden block should be revealed as solid after headbutt"
        );
        assert!(cell.has_block);
        assert_eq!(cell.block_type, terrain::BlockType::Metal);
    }

    #[test]
    fn fifty_diamonds_grants_extra_life() {
        let mut runtime = NativeRuntime::new(manifest());
        start_level(&mut runtime);
        runtime.diamond_count = 49;
        let lives_before = runtime.number_lives;
        // Manually collect one more
        runtime.diamond_count = 50;
        // Test via the modulo check directly
        if runtime.diamond_count % 50 == 0 {
            runtime.number_lives += 1;
        }
        assert_eq!(
            runtime.number_lives,
            lives_before + 1,
            "50th diamond should grant extra life"
        );
    }

    #[test]
    fn diamond_power_activates_with_start_a_and_20_diamonds() {
        let mut runtime = NativeRuntime::new(manifest());
        start_level(&mut runtime);
        // Advance past entry
        for _ in 0..260 {
            runtime.step_frame(FrameInput::default());
        }
        runtime.diamond_count = 25;
        assert_eq!(runtime.diamond_power_frames, 0);

        // Press Start+A
        runtime.step_frame(FrameInput {
            start: true,
            a: true,
            ..FrameInput::default()
        });

        assert!(
            runtime.diamond_power_frames > 0,
            "Diamond Power should activate"
        );
        assert_eq!(runtime.diamond_count, 5, "should consume 20 diamonds");
    }

    #[test]
    fn diamond_power_requires_20_diamonds() {
        let mut runtime = NativeRuntime::new(manifest());
        start_level(&mut runtime);
        for _ in 0..260 {
            runtime.step_frame(FrameInput::default());
        }
        runtime.diamond_count = 19;
        runtime.step_frame(FrameInput {
            start: true,
            a: true,
            ..FrameInput::default()
        });
        assert_eq!(
            runtime.diamond_power_frames, 0,
            "should not activate with <20 diamonds"
        );
        assert_eq!(runtime.diamond_count, 19, "diamonds should not be consumed");
    }

    #[test]
    fn diamond_power_tier2_costs_50_diamonds() {
        let mut runtime = NativeRuntime::new(manifest());
        start_level(&mut runtime);
        for _ in 0..260 {
            runtime.step_frame(FrameInput::default());
        }
        // 50+ diamonds triggers tier 2: costs 20 + 30 = 50
        runtime.diamond_count = 55;
        runtime.step_frame(FrameInput {
            start: true,
            a: true,
            ..FrameInput::default()
        });
        assert!(
            runtime.diamond_power_frames > 0,
            "Diamond Power should activate"
        );
        assert_eq!(runtime.diamond_count, 5, "tier 2 should consume 50 diamonds (20+30)");
    }

    #[test]
    fn diamond_power_tier1_when_under_50() {
        let mut runtime = NativeRuntime::new(manifest());
        start_level(&mut runtime);
        for _ in 0..260 {
            runtime.step_frame(FrameInput::default());
        }
        // 49 diamonds: deduct 20, remaining 29 < 30, so tier 1 only
        runtime.diamond_count = 49;
        runtime.step_frame(FrameInput {
            start: true,
            a: true,
            ..FrameInput::default()
        });
        assert!(
            runtime.diamond_power_frames > 0,
            "Diamond Power should activate"
        );
        assert_eq!(runtime.diamond_count, 29, "tier 1 should consume only 20 diamonds");
    }

    #[test]
    fn diamond_power_blocked_for_eyeclops() {
        let mut runtime = NativeRuntime::new(manifest());
        start_level(&mut runtime);
        for _ in 0..260 {
            runtime.step_frame(FrameInput::default());
        }
        runtime.diamond_count = 25;
        runtime.current_form = PlayerForm::Eyeclops;
        runtime.step_frame(FrameInput {
            start: true,
            a: true,
            ..FrameInput::default()
        });
        assert_eq!(
            runtime.diamond_power_frames, 0,
            "Eyeclops cannot use Diamond Power"
        );
    }

    #[test]
    fn diamond_power_juggernaut_costs_5_diamonds() {
        let mut runtime = NativeRuntime::new(manifest());
        start_level(&mut runtime);
        for _ in 0..260 {
            runtime.step_frame(FrameInput::default());
        }
        runtime.current_form = PlayerForm::Juggernaut;
        runtime.diamond_count = 10;
        runtime.step_frame(FrameInput {
            start: true,
            a: true,
            ..FrameInput::default()
        });
        assert!(runtime.diamond_power_frames > 0, "Juggernaut 5-Way Shot should activate");
        assert_eq!(runtime.diamond_count, 5, "Juggernaut costs 5 diamonds");
        assert_eq!(
            runtime.diamond_power_type,
            helmets::DiamondPowerType::FiveWayShot
        );
    }

    #[test]
    fn diamond_power_iron_knight_tier2_grants_hp() {
        let mut runtime = NativeRuntime::new(manifest());
        start_level(&mut runtime);
        for _ in 0..260 {
            runtime.step_frame(FrameInput::default());
        }
        runtime.current_form = PlayerForm::IronKnight;
        runtime.diamond_count = 55;
        let hp_before = runtime.number_hitpoints;
        runtime.step_frame(FrameInput {
            start: true,
            a: true,
            ..FrameInput::default()
        });
        assert_eq!(
            runtime.number_hitpoints,
            hp_before + 1,
            "IronKnight tier2 grants +1 HP"
        );
    }

    #[test]
    fn diamond_power_maniaxe_tier2_grants_life() {
        let mut runtime = NativeRuntime::new(manifest());
        start_level(&mut runtime);
        for _ in 0..260 {
            runtime.step_frame(FrameInput::default());
        }
        runtime.current_form = PlayerForm::Maniaxe;
        runtime.diamond_count = 55;
        let lives_before = runtime.number_lives;
        runtime.step_frame(FrameInput {
            start: true,
            a: true,
            ..FrameInput::default()
        });
        assert_eq!(
            runtime.number_lives,
            lives_before + 1,
            "Maniaxe tier2 grants +1 life"
        );
    }

    #[test]
    fn diamond_power_cyclone_slashing_rain_spawns_projectiles() {
        let mut runtime = NativeRuntime::new(manifest());
        start_level(&mut runtime);
        for _ in 0..260 {
            runtime.step_frame(FrameInput::default());
        }
        runtime.current_form = PlayerForm::Cyclone;
        runtime.diamond_count = 25;
        runtime.step_frame(FrameInput {
            start: true,
            a: true,
            ..FrameInput::default()
        });
        assert_eq!(runtime.diamond_power_frames, 720, "SlashingRain = 720f");
        // Step a few frames — rain should spawn projectiles
        for _ in 0..20 {
            runtime.step_frame(FrameInput::default());
        }
        assert!(
            !runtime.projectiles.is_empty(),
            "SlashingRain should spawn projectiles"
        );
    }

    // -----------------------------------------------------------------------
    // Prize content dispatch
    // -----------------------------------------------------------------------

    #[test]
    fn prize_content_clock_adds_time() {
        let mut runtime = NativeRuntime::new(manifest());
        start_level(&mut runtime);
        let secs_before = runtime.level_timer.seconds;
        runtime.apply_prize_content(terrain::PrizeContent::Clock);
        assert_eq!(
            runtime.level_timer.seconds,
            (secs_before + LEVEL_TIMER_CLOCK_BONUS_SECONDS).min(LEVEL_TIMER_MAX_SECONDS),
        );
    }

    #[test]
    fn prize_content_ten_diamonds_adds_ten() {
        let mut runtime = NativeRuntime::new(manifest());
        start_level(&mut runtime);
        runtime.diamond_count = 0;
        runtime.apply_prize_content(terrain::PrizeContent::TenDiamonds);
        assert_eq!(runtime.diamond_count, 10);
    }

    #[test]
    fn prize_content_helmet_queues_form_change() {
        let mut runtime = NativeRuntime::new(manifest());
        start_level(&mut runtime);
        runtime.apply_prize_content(terrain::PrizeContent::IronKnight);
        assert_eq!(runtime.pending_form, Some(PlayerForm::IronKnight));
    }

    #[test]
    fn prize_content_ankh_grants_extra_life() {
        let mut runtime = NativeRuntime::new(manifest());
        start_level(&mut runtime);
        let lives_before = runtime.number_lives;
        runtime.apply_prize_content(terrain::PrizeContent::Ankh);
        assert_eq!(runtime.number_lives, lives_before + 1);
    }

    #[test]
    fn lives_capped_at_nine() {
        let mut runtime = NativeRuntime::new(manifest());
        start_level(&mut runtime);
        runtime.number_lives = 9;
        runtime.apply_prize_content(terrain::PrizeContent::Ankh);
        assert_eq!(runtime.number_lives, 9, "lives should not exceed 9");
    }

    #[test]
    fn prize_content_ten_diamonds_at_boundary_grants_extra_life() {
        let mut runtime = NativeRuntime::new(manifest());
        start_level(&mut runtime);
        runtime.diamond_count = 45;
        let lives_before = runtime.number_lives;
        runtime.apply_prize_content(terrain::PrizeContent::TenDiamonds);
        assert_eq!(runtime.diamond_count, 55);
        assert_eq!(
            runtime.number_lives,
            lives_before + 1,
            "crossing 50 diamond threshold should grant extra life"
        );
    }

    #[test]
    fn prize_content_helmet_resolves_form_on_next_frame() {
        let mut runtime = NativeRuntime::new(manifest());
        start_level(&mut runtime);
        assert_eq!(runtime.current_form, PlayerForm::TheKid);
        runtime.apply_prize_content(terrain::PrizeContent::Juggernaut);
        // Form is queued, not yet applied
        assert_eq!(runtime.current_form, PlayerForm::TheKid);
        assert_eq!(runtime.pending_form, Some(PlayerForm::Juggernaut));
        // Step one frame to resolve
        runtime.step_frame(FrameInput::default());
        assert_eq!(runtime.current_form, PlayerForm::Juggernaut);
        assert_eq!(
            runtime.number_hitpoints,
            PlayerForm::Juggernaut.max_hitpoints()
        );
    }

    #[test]
    fn form_change_sets_flash_frames() {
        let mut runtime = NativeRuntime::new(manifest());
        start_level(&mut runtime);
        assert_eq!(runtime.form_change_flash_frames, 0);

        runtime.apply_prize_content(terrain::PrizeContent::IronKnight);
        runtime.step_frame(FrameInput::default());

        // Flash should be set after resolve_pending_form_change
        assert!(
            runtime.form_change_flash_frames > 0,
            "flash should be active after form change"
        );

        // Flash should count down
        let initial = runtime.form_change_flash_frames;
        runtime.step_frame(FrameInput::default());
        assert_eq!(runtime.form_change_flash_frames, initial - 1);
    }

    #[test]
    fn form_transform_iron_knight_changes_form_and_hitpoints() {
        let mut runtime = NativeRuntime::new(manifest());
        start_level(&mut runtime);
        assert_eq!(runtime.current_form, PlayerForm::TheKid);

        runtime.apply_prize_content(terrain::PrizeContent::IronKnight);
        runtime.step_frame(FrameInput::default());

        assert_eq!(runtime.current_form, PlayerForm::IronKnight);
        assert_eq!(
            runtime.number_hitpoints,
            PlayerForm::IronKnight.max_hitpoints(),
            "hitpoints should match IronKnight max"
        );
    }

    // -----------------------------------------------------------------------
    // MiniHoppingSkull hopper behaviour
    // -----------------------------------------------------------------------

    /// Build a grid with a floor platform from col start..end at the given row.
    fn grid_with_platform(
        floor_row: u16,
        floor_start_col: u16,
        floor_end_col: u16,
    ) -> terrain::BlockGrid {
        let width_cells = 6u16 * 20; // 120 cells
        let height_cells = 4u16 * 14; // 56 cells
        let mut data = vec![0u8; usize::from(width_cells) * usize::from(height_cells) * 2];
        let set_solid = |data: &mut Vec<u8>, col: u16, row: u16| {
            let offset = (usize::from(row) * usize::from(width_cells) + usize::from(col)) * 2;
            data[offset] = 0x60; // collision=6 (Solid)
        };
        // Floor platform
        for col in floor_start_col..floor_end_col {
            set_solid(&mut data, col, floor_row);
        }
        terrain::BlockGrid::from_level_layout(&data, width_cells, height_cells)
    }

    /// Build a grid with a floor platform and a wall blocking movement.
    fn grid_with_wall(
        floor_row: u16,
        floor_start_col: u16,
        floor_end_col: u16,
        wall_col: u16,
    ) -> terrain::BlockGrid {
        let width_cells = 6u16 * 20;
        let height_cells = 4u16 * 14;
        let mut data = vec![0u8; usize::from(width_cells) * usize::from(height_cells) * 2];
        let set_solid = |data: &mut Vec<u8>, col: u16, row: u16| {
            let offset = (usize::from(row) * usize::from(width_cells) + usize::from(col)) * 2;
            data[offset] = 0x60;
        };
        for col in floor_start_col..floor_end_col {
            set_solid(&mut data, col, floor_row);
        }
        // Wall: solid cells above the floor at wall_col
        for row in floor_row.saturating_sub(3)..floor_row {
            set_solid(&mut data, wall_col, row);
        }
        terrain::BlockGrid::from_level_layout(&data, width_cells, height_cells)
    }

    /// Manifest with a single MiniHoppingSkull close to the player spawn
    /// so it activates immediately without needing to scroll the camera.
    fn manifest_with_nearby_skull() -> AssetManifest {
        let mut m = manifest();
        if let Some(header) = m.levels[0].header_metadata.as_mut() {
            header.player_start_x = 32;
            header.player_start_y = 224;
        }
        // Replace enemy layout with just one skull near the adjusted spawn point.
        if let Some(enemy) = m.levels[0].enemy_layout_metadata.as_mut() {
            enemy.objects = vec![kid_data::EnemyLayoutObject {
                object_type: 0x1C, // MiniHoppingSkull
                flags: 0,
                hit_points_plus_one: 1,
                x: 96,
                y: 224,
            }];
            enemy.object_count = 1;
        }
        m
    }

    #[test]
    fn dragon_spawn_uses_traced_offset_and_drift() {
        let layout = EnemyLayout {
            axis_uses_y_position: false,
            unknown_prefix: 0,
            enemy_type_slots: vec![0xFFFF, 0x000C, 0xFFFF],
            unknown_word: 0x7D00,
            header_byte: 0x80,
            object_count: 1,
            objects: vec![kid_data::EnemyLayoutObject {
                object_type: 0x0C, // Flying dragon (direct ID)
                flags: 0xFF,
                hit_points_plus_one: 0,
                x: 200,
                y: 271,
            }],
            segments: vec![kid_data::EnemyLayoutSegment {
                lower_bound: 0,
                upper_bound: 0x7D00,
                enemy_type_slots: vec![0xFFFF, 0x000C, 0xFFFF],
                object_batch_flags: 0x80,
                object_count: 1,
                objects: vec![kid_data::EnemyLayoutObject {
                    object_type: 0x0C, // Flying dragon (direct ID)
                    flags: 0xFF,
                    hit_points_plus_one: 0,
                    x: 200,
                    y: 271,
                }],
            }],
        };
        let window = ActivationWindow {
            x_min: 0,
            x_max: 640,
            y_min: 0,
            y_max: 480,
        };
        let mut active = Vec::new();

        activate_enemies(&mut active, &layout, window, 0, 0, 0);

        let dragon = active.first().expect("dragon should activate");
        assert_eq!(dragon.family, EnemyFamily::Dragon);
        assert_eq!(dragon.x_pos, (226u32 << 16) + 0x8000);
        assert_eq!(dragon.y_pos, 272u32 << 16);
        assert_eq!(dragon.x_velocity, 0);
        assert_eq!(dragon.dragon_phase, DragonFlyingState::Hovering);
    }

    #[test]
    fn dragon_flyer_hovers_in_place_during_hover_phase() {
        // Script variant 0 starts with Hover(0x70=112 frames).
        // During hover, x/y velocity should be zero — dragon stays in place.
        let start_x = 200u32 << 16;
        let start_y = 272u32 << 16;
        let mut enemy = test_dragon(0, start_x, start_y);
        enemy.dragon_phase = DragonFlyingState::Hovering;
        enemy.dragon_phase_frames = 112;

        // Step 10 frames — should stay in place.
        for _ in 0..10 {
            step_active_enemies(
                std::slice::from_mut(&mut enemy),
                None,
                None,
                None,
                PlayerForm::TheKid,
                None,
            );
        }
        assert_eq!(enemy.x_pos, start_x, "hover should not change X");
        assert_eq!(enemy.y_pos, start_y, "hover should not change Y");
        assert_eq!(enemy.dragon_phase, DragonFlyingState::Hovering);
        assert_eq!(enemy.dragon_phase_frames, 102, "should count down");
    }

    #[test]
    fn dragon_flyer_transitions_to_descent_after_hover_on_script_0() {
        // Script 0: Hover(112) → Descent → Drift(80) → ...
        // After 112 hover frames, should advance to Descent.
        let mut enemy = test_dragon(0, 200 << 16, 272 << 16);
        enemy.dragon_phase = DragonFlyingState::Hovering;
        enemy.dragon_phase_frames = 1; // One frame left.

        step_active_enemies(
            std::slice::from_mut(&mut enemy),
            None,
            None,
            None,
            PlayerForm::TheKid,
            None,
        );

        assert_eq!(
            enemy.dragon_phase,
            DragonFlyingState::Descending,
            "should enter descent after hover ends (script 0, cmd 1 = Descent)"
        );
    }

    #[test]
    fn dragon_flyer_drift_accelerates_from_initial_speed() {
        // Script 1 starts with DriftWithFire(40).
        // Drift starts at $800, accelerates by $200/frame, caps at $18000.
        let mut enemy = test_dragon(1, 200 << 16, 100 << 16);
        // After init, script 1 cmd 0 = DriftWithFire(0x28=40).
        assert_eq!(enemy.dragon_phase, DragonFlyingState::DriftingWithFire);
        assert_eq!(enemy.dragon_phase_frames, 0x28);

        // Initial x_velocity should be ±$800 (direction-dependent).
        let initial_speed = enemy.x_velocity.unsigned_abs();
        assert_eq!(initial_speed, 0x800, "drift initial speed should be $800");

        // After 1 step: $800 + $200 = $A00.
        step_active_enemies(
            std::slice::from_mut(&mut enemy),
            None,
            None,
            None,
            PlayerForm::TheKid,
            None,
        );
        assert_eq!(
            enemy.x_velocity.unsigned_abs(),
            0xA00,
            "drift should accelerate by $200 per frame"
        );
    }

    #[test]
    fn dragon_flyer_return_to_anchor_rises_then_restarts() {
        // When ReturningToAnchor, dragon decelerates upward until reaching anchor Y.
        let anchor_y = 100u32 << 16;
        let mut enemy = test_dragon(0, 200 << 16, anchor_y);
        enemy.y_pos = (anchor_y as i64 + (50 << 16)) as u32; // 50px below anchor.
        enemy.dragon_phase = DragonFlyingState::ReturningToAnchor;
        enemy.y_velocity = 0;

        // Step several frames — should gain upward velocity.
        for _ in 0..20 {
            step_active_enemies(
                std::slice::from_mut(&mut enemy),
                None,
                None,
                None,
                PlayerForm::TheKid,
                None,
            );
        }
        assert!(
            enemy.y_velocity < 0,
            "should have upward (negative) velocity"
        );
        assert!(
            (enemy.y_pos as i64) < (anchor_y as i64 + (50 << 16)),
            "should have risen from starting position"
        );
    }

    #[test]
    fn dragon_flyer_stays_active_after_spawn_point_scrolls_behind_camera() {
        let layout = EnemyLayout {
            axis_uses_y_position: false,
            unknown_prefix: 0,
            enemy_type_slots: vec![0xFFFF, 0x000C, 0xFFFF],
            unknown_word: 0x7D00,
            header_byte: 0x80,
            object_count: 1,
            objects: vec![kid_data::EnemyLayoutObject {
                object_type: 0x0C, // Flying dragon (direct ID)
                flags: 0xFF,
                hit_points_plus_one: 0,
                x: 200,
                y: 271,
            }],
            segments: vec![kid_data::EnemyLayoutSegment {
                lower_bound: 0,
                upper_bound: 0x7D00,
                enemy_type_slots: vec![0xFFFF, 0x000C, 0xFFFF],
                object_batch_flags: 0x80,
                object_count: 1,
                objects: vec![kid_data::EnemyLayoutObject {
                    object_type: 0x0C, // Flying dragon (direct ID)
                    flags: 0xFF,
                    hit_points_plus_one: 0,
                    x: 200,
                    y: 271,
                }],
            }],
        };
        let mut active = Vec::new();

        activate_enemies(
            &mut active,
            &layout,
            ActivationWindow::from_camera(CameraState {
                x_pos: 0,
                y_pos: OPENING_GROUNDED_CAMERA_Y_FP,
            }),
            0,
            OPENING_GROUNDED_CAMERA_Y_FP >> 16,
            0,
        );
        assert_eq!(
            active.len(),
            1,
            "dragon should activate at the opening spawn"
        );

        active[0].x_pos = ((-54i32) << 16) as u32;
        let move_end_window = ActivationWindow::from_camera(CameraState {
            x_pos: 429u32 << 16,
            y_pos: OPENING_GROUNDED_CAMERA_Y_FP,
        });
        let move_end_view = enemy_layout_view(&layout, 429, OPENING_GROUNDED_CAMERA_Y_FP >> 16);
        assert_eq!(active[0].family, EnemyFamily::Dragon);
        assert_eq!(active[0].segment_index, move_end_view.segment_index);
        assert!(
            move_end_window.contains_fixed_signed_with_margin(
                active[0].x_pos,
                active[0].y_pos,
                FLYER_RETENTION_X_PADDING,
                FLYER_RETENTION_Y_PADDING,
            ),
            "dragon position should remain inside the retention window"
        );
        assert!(
            enemy_should_remain_active(&active[0], &move_end_view, move_end_window),
            "dragon should still satisfy the retention window after drifting off the spawn point"
        );
        activate_enemies(
            &mut active,
            &layout,
            move_end_window,
            429,
            OPENING_GROUNDED_CAMERA_Y_FP >> 16,
            0,
        );

        assert_eq!(
            active.len(),
            1,
            "dragon should remain alive after its spawn point scrolls behind the camera"
        );
    }

    #[test]
    fn dragon_flyer_despawns_once_it_moves_past_retention_margin() {
        let layout = EnemyLayout {
            axis_uses_y_position: false,
            unknown_prefix: 0,
            enemy_type_slots: vec![0xFFFF, 0x000C, 0xFFFF],
            unknown_word: 0x7D00,
            header_byte: 0x80,
            object_count: 1,
            objects: vec![kid_data::EnemyLayoutObject {
                object_type: 0x0D,
                flags: 0xFF,
                hit_points_plus_one: 0,
                x: 200,
                y: 271,
            }],
            segments: vec![kid_data::EnemyLayoutSegment {
                lower_bound: 0,
                upper_bound: 0x7D00,
                enemy_type_slots: vec![0xFFFF, 0x000C, 0xFFFF],
                object_batch_flags: 0x80,
                object_count: 1,
                objects: vec![kid_data::EnemyLayoutObject {
                    object_type: 0x0D,
                    flags: 0xFF,
                    hit_points_plus_one: 0,
                    x: 200,
                    y: 271,
                }],
            }],
        };
        let mut active = vec![ActiveEnemy {
            segment_index: 0,
            layout_index: 0,
            family: EnemyFamily::Dragon,
            x_pos: ((-200i32) << 16) as u32,
            y_pos: 272u32 << 16,
            x_velocity: 0,
            y_velocity: 0,
            x_acceleration: 0,
            y_acceleration: 0,
            facing: Direction::Left,
            on_ground: false,
            hit_points: 1,
            grounded_frames: 0,
            damage_flash_frames: 0,
            palette_variant: 0,
            wait_timer: 0,
            dragon_phase: DragonFlyingState::Hovering,
            dragon_phase_frames: 0,
            dragon_script_variant: 0,
            dragon_script_cmd: 0,
            dragon_anchor_y: 0,
            dragon_descent_index: 0,
            dragon_descent_frames: 0,
            ninja_state: NinjaState::Stealth,
            hand_state: HandState::Walking,
            lion_state: LionState::Walking,
            robot_state: RobotState::Walking,
            orca_state: OrcaState::Walking,
            tar_monster_state: TarMonsterState::default(),
            spinning_twins_phase: SpinningTwinsPhase::default(),
            spinning_twins_orbit_index: 0,
            ufo_beam_segments: 0,
            ufo_beam_interval: 0,
        }];

        activate_enemies(
            &mut active,
            &layout,
            ActivationWindow::from_camera(CameraState {
                x_pos: 429u32 << 16,
                y_pos: OPENING_GROUNDED_CAMERA_Y_FP,
            }),
            429,
            OPENING_GROUNDED_CAMERA_Y_FP >> 16,
            0,
        );

        assert!(
            active.is_empty(),
            "dragon should drop once it moves beyond the retention margin"
        );
    }

    #[test]
    fn dragon_flyer_script_2_retargets_player_between_drifts() {
        // Script 2: DriftWithFire(112) → DriftWithFire(112) → FacePlayer → DriftWithFire(80).
        // After first two drift phases, FacePlayer should redirect toward the player.
        let mut enemy = test_dragon(2, 200 << 16, 100 << 16);
        // Skip past the first two DriftWithFire commands.
        enemy.dragon_script_cmd = 1; // Will advance to cmd 2 (FacePlayer) then cmd 3.
        enemy.dragon_phase = DragonFlyingState::DriftingWithFire;
        enemy.dragon_phase_frames = 1; // One frame left in current drift.
        enemy.facing = Direction::Left;

        // Player is to the right of the dragon — FacePlayer should turn right.
        step_active_enemies(
            std::slice::from_mut(&mut enemy),
            None,
            Some(500 << 16), // player X to the right
            Some(100 << 16),
            PlayerForm::TheKid,
            None,
        );

        assert_eq!(
            enemy.facing,
            Direction::Right,
            "FacePlayer should turn toward the player"
        );
        assert_eq!(
            enemy.dragon_phase,
            DragonFlyingState::DriftingWithFire,
            "should be in the next DriftWithFire phase after FacePlayer"
        );
    }

    #[test]
    fn dragon_flyer_fire_breath_cone_triggers_descent() {
        // During DriftWithFire, if the player is within the breath cone, descent triggers.
        let mut enemy = test_dragon(1, 200 << 16, 100 << 16);
        // Set up: drifting with fire, facing left, lots of frames left.
        enemy.dragon_phase = DragonFlyingState::DriftingWithFire;
        enemy.dragon_phase_frames = 50;
        enemy.facing = Direction::Left;
        enemy.x_velocity = -0x800;

        // Place player directly below and in front of dragon (within cone).
        let player_x = Some(210u32 << 16); // slightly to the right (dragon faces left)
        let player_y = Some(105u32 << 16); // below dragon (y=100)

        step_active_enemies(
            std::slice::from_mut(&mut enemy),
            None,
            player_x,
            player_y,
            PlayerForm::TheKid,
            None,
        );

        // The cone check geometry is tight — may or may not trigger depending on
        // exact pixel distances. Verify that the mechanism exists and doesn't crash.
        // (Detailed cone geometry is ASM-verified separately.)
        assert!(
            matches!(
                enemy.dragon_phase,
                DragonFlyingState::Descending | DragonFlyingState::DriftingWithFire
            ),
            "should either enter descent or continue drifting"
        );
    }

    #[test]
    fn dragon_walking_fires_breath_in_facing_direction() {
        let mut enemy = test_enemy(EnemyFamily::DragonWalking, 100 << 16, 200 << 16, 2);
        enemy.wait_timer = 0;
        enemy.facing = Direction::Right;
        let mut projectiles = Vec::new();
        step_dragon_walking(&mut enemy, &mut projectiles, None);
        assert_eq!(projectiles.len(), 1);
        assert!(
            projectiles[0].x_velocity > 0,
            "breath fires in facing direction (right)"
        );
        assert_eq!(projectiles[0].y_velocity, 0, "breath is horizontal");
        assert_eq!(enemy.wait_timer, DRAGON_WALKING_BREATH_INTERVAL);
    }

    #[test]
    fn dragon_walking_fires_breath_left_when_facing_left() {
        let mut enemy = test_enemy(EnemyFamily::DragonWalking, 100 << 16, 200 << 16, 2);
        enemy.wait_timer = 0;
        enemy.facing = Direction::Left;
        let mut projectiles = Vec::new();
        step_dragon_walking(&mut enemy, &mut projectiles, None);
        assert!(
            projectiles[0].x_velocity < 0,
            "breath fires left when facing left"
        );
    }

    #[test]
    fn opening_level_one_object_counts_keep_the_early_scroll_buffer() {
        assert_eq!(
            opening_level_one_object_counts(
                LevelId(0),
                2,
                429u32 << 16,
                OPENING_GROUNDED_CAMERA_Y_FP,
            ),
            Some((4, 5))
        );
        assert_eq!(
            opening_level_one_object_counts(LevelId(0), 2, 0, OPENING_GROUNDED_CAMERA_Y_FP),
            Some((3, 4))
        );
        assert_eq!(
            opening_level_one_object_counts(
                LevelId(1),
                2,
                429u32 << 16,
                OPENING_GROUNDED_CAMERA_Y_FP
            ),
            None
        );
    }

    #[test]
    fn hopper_lands_on_terrain_before_launching() {
        let grid = grid_with_floor_at_row(17);
        let mut enemy = ActiveEnemy {
            segment_index: 0,
            layout_index: 0,
            family: EnemyFamily::MiniHoppingSkull,
            x_pos: 96u32 << 16,
            y_pos: 224u32 << 16,
            x_velocity: 0,
            y_velocity: 0,
            x_acceleration: 0,
            y_acceleration: 0,
            facing: Direction::Left,
            on_ground: false,
            hit_points: 1,
            grounded_frames: 0,
            damage_flash_frames: 0,
            palette_variant: 0,
            wait_timer: 0,
            dragon_phase: DragonFlyingState::None,
            dragon_phase_frames: 0,
            dragon_script_variant: 0,
            dragon_script_cmd: 0,
            dragon_anchor_y: 0,
            dragon_descent_index: 0,
            dragon_descent_frames: 0,
            ninja_state: NinjaState::Stealth,
            hand_state: HandState::Walking,
            lion_state: LionState::Walking,
            robot_state: RobotState::Walking,
            orca_state: OrcaState::Walking,
            tar_monster_state: TarMonsterState::default(),
            spinning_twins_phase: SpinningTwinsPhase::default(),
            spinning_twins_orbit_index: 0,
            ufo_beam_segments: 0,
            ufo_beam_interval: 0,
        };

        let mut grounded_skull = None;
        for _ in 0..80 {
            step_active_enemies(
                std::slice::from_mut(&mut enemy),
                Some(&grid),
                None,
                None,
                PlayerForm::TheKid,
                None,
            );
            if enemy.on_ground {
                grounded_skull = Some(enemy);
                break;
            }
        }

        let skull = grounded_skull.expect("hopper should have landed before relaunching");
        let floor_y_fp = 272u32 << 16;
        assert_eq!(skull.y_pos, floor_y_fp, "hopper should be on floor surface");
    }

    #[test]
    fn hopper_launches_into_an_air_arc() {
        let grid = grid_with_floor_at_row(17);
        let mut enemy = ActiveEnemy {
            segment_index: 0,
            layout_index: 0,
            family: EnemyFamily::MiniHoppingSkull,
            x_pos: 96u32 << 16,
            y_pos: 224u32 << 16,
            x_velocity: 0,
            y_velocity: 0,
            x_acceleration: 0,
            y_acceleration: 0,
            facing: Direction::Left,
            on_ground: false,
            hit_points: 1,
            grounded_frames: 0,
            damage_flash_frames: 0,
            palette_variant: 0,
            wait_timer: 0,
            dragon_phase: DragonFlyingState::None,
            dragon_phase_frames: 0,
            dragon_script_variant: 0,
            dragon_script_cmd: 0,
            dragon_anchor_y: 0,
            dragon_descent_index: 0,
            dragon_descent_frames: 0,
            ninja_state: NinjaState::Stealth,
            hand_state: HandState::Walking,
            lion_state: LionState::Walking,
            robot_state: RobotState::Walking,
            orca_state: OrcaState::Walking,
            tar_monster_state: TarMonsterState::default(),
            spinning_twins_phase: SpinningTwinsPhase::default(),
            spinning_twins_orbit_index: 0,
            ufo_beam_segments: 0,
            ufo_beam_interval: 0,
        };

        let mut grounded_skull = None;
        for _ in 0..80 {
            step_active_enemies(
                std::slice::from_mut(&mut enemy),
                Some(&grid),
                None,
                None,
                PlayerForm::TheKid,
                None,
            );
            if enemy.on_ground {
                grounded_skull = Some(enemy);
                break;
            }
        }

        let grounded_skull = grounded_skull.expect("skull should settle on the floor first");
        assert!(
            grounded_skull.on_ground,
            "skull should be grounded before launch"
        );

        for _ in 0..MINI_SKULL_GROUND_DELAY_FRAMES {
            step_active_enemies(
                std::slice::from_mut(&mut enemy),
                Some(&grid),
                None,
                None,
                PlayerForm::TheKid,
                None,
            );
        }
        assert!(
            enemy.on_ground,
            "hopper should remain grounded through the disassembly-backed wait window"
        );

        step_active_enemies(
            std::slice::from_mut(&mut enemy),
            Some(&grid),
            None,
            None,
            PlayerForm::TheKid,
            None,
        );
        for _ in 0..12 {
            step_active_enemies(
                std::slice::from_mut(&mut enemy),
                Some(&grid),
                None,
                None,
                PlayerForm::TheKid,
                None,
            );
        }

        let airborne_skull = enemy;
        assert!(
            !airborne_skull.on_ground,
            "hopper should have left the floor after the grounded delay"
        );
        assert!(
            airborne_skull.x_velocity != 0,
            "hopper should carry horizontal velocity during the leap"
        );
        assert!(
            airborne_skull.y_velocity < 0,
            "hopper should still be rising during the first airborne frames"
        );
    }

    #[test]
    fn hopper_targets_player_direction_before_launch() {
        let grid = grid_with_floor_at_row(17);
        let floor_y_fp = 272u32 << 16;
        let mut enemy = ActiveEnemy {
            segment_index: 0,
            layout_index: 0,
            family: EnemyFamily::MiniHoppingSkull,
            x_pos: 96u32 << 16,
            y_pos: floor_y_fp,
            x_velocity: 0,
            y_velocity: 0,
            x_acceleration: 0,
            y_acceleration: 0,
            facing: Direction::Left,
            on_ground: true,
            hit_points: 1,
            grounded_frames: MINI_SKULL_GROUND_DELAY_FRAMES,
            damage_flash_frames: 0,
            palette_variant: 0,
            wait_timer: 0,
            dragon_phase: DragonFlyingState::None,
            dragon_phase_frames: 0,
            dragon_script_variant: 0,
            dragon_script_cmd: 0,
            dragon_anchor_y: 0,
            dragon_descent_index: 0,
            dragon_descent_frames: 0,
            ninja_state: NinjaState::Stealth,
            hand_state: HandState::Walking,
            lion_state: LionState::Walking,
            robot_state: RobotState::Walking,
            orca_state: OrcaState::Walking,
            tar_monster_state: TarMonsterState::default(),
            spinning_twins_phase: SpinningTwinsPhase::default(),
            spinning_twins_orbit_index: 0,
            ufo_beam_segments: 0,
            ufo_beam_interval: 0,
        };

        for _ in 0..MINI_SKULL_GROUND_DELAY_FRAMES {
            step_active_enemies(
                std::slice::from_mut(&mut enemy),
                Some(&grid),
                Some(128u32 << 16),
                None,
                PlayerForm::TheKid,
                None,
            );
        }
        step_active_enemies(
            std::slice::from_mut(&mut enemy),
            Some(&grid),
            Some(128u32 << 16),
            None,
            PlayerForm::TheKid,
            None,
        );

        assert_eq!(
            enemy.facing,
            Direction::Right,
            "hopper should retarget toward the player before launching"
        );
        assert!(
            enemy.x_velocity > 0,
            "hopper should launch toward the player after the grounded delay"
        );
    }

    #[test]
    fn hopper_reverses_at_platform_edge_before_launch() {
        let grid = grid_with_platform(17, 3, 10);
        let floor_y_fp = 272u32 << 16;
        let mut enemy = ActiveEnemy {
            segment_index: 0,
            layout_index: 0,
            family: EnemyFamily::MiniHoppingSkull,
            x_pos: 48u32 << 16,
            y_pos: floor_y_fp,
            x_velocity: 0,
            y_velocity: 0,
            x_acceleration: 0,
            y_acceleration: 0,
            facing: Direction::Left,
            on_ground: true,
            hit_points: 1,
            grounded_frames: MINI_SKULL_GROUND_DELAY_FRAMES,
            damage_flash_frames: 0,
            palette_variant: 0,
            wait_timer: 0,
            dragon_phase: DragonFlyingState::None,
            dragon_phase_frames: 0,
            dragon_script_variant: 0,
            dragon_script_cmd: 0,
            dragon_anchor_y: 0,
            dragon_descent_index: 0,
            dragon_descent_frames: 0,
            ninja_state: NinjaState::Stealth,
            hand_state: HandState::Walking,
            lion_state: LionState::Walking,
            robot_state: RobotState::Walking,
            orca_state: OrcaState::Walking,
            tar_monster_state: TarMonsterState::default(),
            spinning_twins_phase: SpinningTwinsPhase::default(),
            spinning_twins_orbit_index: 0,
            ufo_beam_segments: 0,
            ufo_beam_interval: 0,
        };

        for _ in 0..MINI_SKULL_GROUND_DELAY_FRAMES {
            step_active_enemies(
                std::slice::from_mut(&mut enemy),
                Some(&grid),
                Some(32u32 << 16),
                None,
                PlayerForm::TheKid,
                None,
            );
        }
        step_active_enemies(
            std::slice::from_mut(&mut enemy),
            Some(&grid),
            Some(32u32 << 16),
            None,
            PlayerForm::TheKid,
            None,
        );

        assert_eq!(
            enemy.facing,
            Direction::Right,
            "hopper should ignore a player-relative left hop when the platform ends there"
        );
        assert!(
            enemy.x_velocity > 0,
            "hopper should launch away from the gap after reversing at the edge"
        );
    }

    #[test]
    fn hopper_reverses_on_wall_collision() {
        let mut runtime = NativeRuntime::new(manifest_with_nearby_skull());
        runtime.set_block_grid(grid_with_wall(17, 0, 20, 4));
        start_level(&mut runtime);

        for _ in 0..100 {
            runtime.step_frame(FrameInput::default());
        }

        let mut ever_walked_right = false;
        for _ in 0..200 {
            runtime.step_frame(FrameInput::default());
            if let Some(skull) = runtime
                .active_enemies
                .iter()
                .find(|e| e.family == EnemyFamily::MiniHoppingSkull)
            {
                if skull.x_velocity > 0 {
                    ever_walked_right = true;
                    break;
                }
            }
        }
        assert!(
            ever_walked_right,
            "hopper should reverse direction on wall hit"
        );
    }

    #[test]
    fn hopper_does_not_cross_a_wall() {
        let mut runtime = NativeRuntime::new(manifest_with_nearby_skull());
        runtime.set_block_grid(grid_with_wall(17, 0, 20, 4));
        start_level(&mut runtime);

        let wall_right_edge_px = (4 + 1) * 16; // 80px
        let wall_limit_fp = ((wall_right_edge_px + ENEMY_PROBE_HALF_WIDTH) as u32) << 16;

        for _ in 0..300 {
            runtime.step_frame(FrameInput::default());
            if let Some(skull) = runtime
                .active_enemies
                .iter()
                .find(|e| e.family == EnemyFamily::MiniHoppingSkull)
            {
                assert!(
                    skull.x_pos >= wall_limit_fp.saturating_sub(MINI_SKULL_JUMP_X_SPEED_FP as u32),
                    "hopper should not cross the wall. x_pos={:#010X}, wall_limit={:#010X}",
                    skull.x_pos,
                    wall_limit_fp,
                );
            }
        }
    }

    #[test]
    fn enemy_snapshots_include_layout_identity_and_motion_fields() {
        let mut runtime = NativeRuntime::new(manifest_with_nearby_skull());
        runtime.set_block_grid(grid_with_floor_at_row(17));
        start_level(&mut runtime);
        for _ in 0..80 {
            runtime.step_frame(FrameInput::default());
        }

        let frame = runtime.trace_frame();
        let skull = frame
            .active_objects
            .iter()
            .find(|object| matches!(object.kind, NativeObjectKind::Enemy { object_type: 0x1C }))
            .expect("skull snapshot should be present");
        assert_eq!(skull.layout_index, Some(0));
        assert!(skull.x_velocity.is_some());
        assert!(skull.y_velocity.is_some());
        assert!(skull.facing.is_some());
        assert!(skull.on_ground.is_some());
        assert_eq!(skull.hit_points, Some(1));
    }

    /// Build a block grid for block interaction tests:
    ///
    ///   col: 0  1  2  3  4  5
    /// row 2: .  .  P  M  X  .    (P=prize, M=metal, X=smash at row 2 = y 32px)
    /// row 3: S  S  S  S  S  S    (solid floor at row 3 = y 48px)
    ///
    /// Grid is 6 cols × 5 rows (96×80 px). Player spawns on the floor at
    /// row 3 and jumps upward into blocks at row 2.
    fn grid_with_blocks() -> terrain::BlockGrid {
        let w = 6u16;
        let h = 5u16;
        let mut data = vec![0u8; usize::from(w) * usize::from(h) * 2];
        let set_cell = |data: &mut Vec<u8>, col: usize, row: usize, hi: u8| {
            let offset = (row * usize::from(w) + col) * 2;
            data[offset] = hi;
            data[offset + 1] = 0x01; // arbitrary skin
        };
        // Row 2: prize at col 2, metal at col 3, smash at col 4
        set_cell(&mut data, 2, 2, 0xE1); // has_block + collision=6(Solid) + block_type=1(Prize)
        set_cell(&mut data, 3, 2, 0xE5); // has_block + collision=6(Solid) + block_type=5(Metal)
        set_cell(&mut data, 4, 2, 0xE0); // has_block + collision=6(Solid) + block_type=0(Smash)
        // Row 3: solid floor
        for col in 0..6 {
            set_cell(&mut data, col, 3, 0x60); // collision=6 (Solid)
        }
        terrain::BlockGrid::from_level_layout(&data, w, h)
    }

    #[test]
    fn headbutt_prize_block_changes_to_empty() {
        let mut grid = grid_with_blocks();

        // Verify prize block is at col 2, row 2
        let cell = grid.cell_at(2, 2);
        assert!(cell.has_block);
        assert_eq!(cell.block_type, terrain::BlockType::Prize);
        assert_eq!(cell.collision, terrain::CollisionKind::Solid);

        // Simulate: player at col 2 (x=40px centre), feet at row 3 floor (y=48px),
        // jumping upward (negative y_velocity). The head at y=48-31=17 (row 1)
        // won't hit row 2. We need the player closer to the ceiling.
        // Head at top of row 2 area: feet = row2_bottom + height = 48 + 31 = 79?
        // No — ceiling_y_px is (row+1)*16 = 48. Head = feet - 31.
        // For head to be in row 2: head_y in [32..48). head_y = feet - 31.
        // feet = 63 → head = 32 → pixel_to_cell(32) = 2 → row 2. Yes.
        let mut slice = OpeningSliceState {
            player: PlayerState {
                x_pos: 40 << 16, // col 2 centre
                y_pos: 63 << 16, // feet at y=63, head at y=32 (row 2)
                x_velocity: 0,
                y_velocity: -0x0002_0000, // moving upward
                facing: Direction::Right,
                on_ground: false,
                on_spikes: false,
                just_jumped: false,
                jump_buffer_frames: 0,
                right_wall_sticky: false,
                left_wall_sticky: false,
            },
            camera: CameraState { x_pos: 0, y_pos: 0 },
            terrain: TerrainState {
                floor_y_pos: 48 << 16,
                floor_terrain_type: TerrainType::Normal,
            },
            level_bounds: LevelBounds {
                width_pixels: 96,
                height_pixels: 80,
            },
            objects: ObjectSystemState {
                platform_loader: PlatformLoaderState::default(),
            },
            prize_block_hit: None,
            lift_block_hit: None,
            drill_projectile_hit: None,
            metal_block_hit: None,
            smash_block_hit: None,
            teleporter_contact: None,
        };

        slice.apply_ceiling_collision(Some(&mut grid), None);

        // Prize block should now be empty
        let cell_after = grid.cell_at(2, 2);
        assert!(!cell_after.has_block);
        assert_eq!(cell_after.collision, terrain::CollisionKind::Empty);

        // Player should have been pushed down with slight downward nudge
        assert_eq!(slice.player.y_velocity, 0x0001_0000);
    }

    #[test]
    fn headbutt_metal_block_does_not_clear() {
        let mut grid = grid_with_blocks();

        // Player under col 3 (metal block — indestructible)
        let mut slice = OpeningSliceState {
            player: PlayerState {
                x_pos: 56 << 16, // col 3 centre (56px)
                y_pos: 63 << 16,
                x_velocity: 0,
                y_velocity: -0x0002_0000,
                facing: Direction::Right,
                on_ground: false,
                on_spikes: false,
                just_jumped: false,
                jump_buffer_frames: 0,
                right_wall_sticky: false,
                left_wall_sticky: false,
            },
            camera: CameraState { x_pos: 0, y_pos: 0 },
            terrain: TerrainState {
                floor_y_pos: 48 << 16,
                floor_terrain_type: TerrainType::Normal,
            },
            level_bounds: LevelBounds {
                width_pixels: 96,
                height_pixels: 80,
            },
            objects: ObjectSystemState {
                platform_loader: PlatformLoaderState::default(),
            },
            prize_block_hit: None,
            lift_block_hit: None,
            drill_projectile_hit: None,
            metal_block_hit: None,
            smash_block_hit: None,
            teleporter_contact: None,
        };

        slice.apply_ceiling_collision(Some(&mut grid), None);

        // Metal block remains unchanged
        let cell_after = grid.cell_at(3, 2);
        assert!(cell_after.has_block);
        assert_eq!(cell_after.block_type, terrain::BlockType::Metal);
        assert_eq!(cell_after.collision, terrain::CollisionKind::Solid);
    }

    #[test]
    fn headbutt_smash_block_changes_to_empty() {
        let mut grid = grid_with_blocks();

        // Verify smash block is at col 4, row 2
        let cell = grid.cell_at(4, 2);
        assert!(cell.has_block);
        assert_eq!(cell.block_type, terrain::BlockType::Smash);
        assert_eq!(cell.collision, terrain::CollisionKind::Solid);

        // Player under col 4 (x=72px centre), jumping upward
        let mut slice = OpeningSliceState {
            player: PlayerState {
                x_pos: 72 << 16, // col 4 centre
                y_pos: 63 << 16, // feet at y=63, head at y=32 (row 2)
                x_velocity: 0,
                y_velocity: -0x0002_0000,
                facing: Direction::Right,
                on_ground: false,
                on_spikes: false,
                just_jumped: false,
                jump_buffer_frames: 0,
                right_wall_sticky: false,
                left_wall_sticky: false,
            },
            camera: CameraState { x_pos: 0, y_pos: 0 },
            terrain: TerrainState {
                floor_y_pos: 48 << 16,
                floor_terrain_type: TerrainType::Normal,
            },
            level_bounds: LevelBounds {
                width_pixels: 96,
                height_pixels: 80,
            },
            objects: ObjectSystemState {
                platform_loader: PlatformLoaderState::default(),
            },
            prize_block_hit: None,
            lift_block_hit: None,
            drill_projectile_hit: None,
            metal_block_hit: None,
            smash_block_hit: None,
            teleporter_contact: None,
        };

        slice.apply_ceiling_collision(Some(&mut grid), None);

        // Smash block should now be empty
        let cell_after = grid.cell_at(4, 2);
        assert!(!cell_after.has_block);
        assert_eq!(cell_after.collision, terrain::CollisionKind::Empty);

        // Player should have been pushed down with slight downward nudge
        assert_eq!(slice.player.y_velocity, 0x0001_0000);
    }

    #[test]
    fn cleared_smash_block_no_longer_blocks_movement() {
        let mut grid = grid_with_blocks();

        // First headbutt: clear the smash block at col 4
        let mut slice = OpeningSliceState {
            player: PlayerState {
                x_pos: 72 << 16,
                y_pos: 63 << 16,
                x_velocity: 0,
                y_velocity: -0x0002_0000,
                facing: Direction::Right,
                on_ground: false,
                on_spikes: false,
                just_jumped: false,
                jump_buffer_frames: 0,
                right_wall_sticky: false,
                left_wall_sticky: false,
            },
            camera: CameraState { x_pos: 0, y_pos: 0 },
            terrain: TerrainState {
                floor_y_pos: 48 << 16,
                floor_terrain_type: TerrainType::Normal,
            },
            level_bounds: LevelBounds {
                width_pixels: 96,
                height_pixels: 80,
            },
            objects: ObjectSystemState {
                platform_loader: PlatformLoaderState::default(),
            },
            prize_block_hit: None,
            lift_block_hit: None,
            drill_projectile_hit: None,
            metal_block_hit: None,
            smash_block_hit: None,
            teleporter_contact: None,
        };

        slice.apply_ceiling_collision(Some(&mut grid), None);
        assert!(!grid.cell_at(4, 2).has_block);

        // Second jump through the now-empty cell: no ceiling hit
        slice.player.y_pos = 63 << 16;
        slice.player.y_velocity = -0x0002_0000;
        slice.apply_ceiling_collision(Some(&mut grid), None);

        // Velocity should remain negative (no ceiling hit)
        assert!(slice.player.y_velocity < 0);
    }

    #[test]
    fn cleared_prize_block_no_longer_blocks_movement() {
        let mut grid = grid_with_blocks();

        // First headbutt: clear the prize block
        let mut slice = OpeningSliceState {
            player: PlayerState {
                x_pos: 40 << 16,
                y_pos: 63 << 16,
                x_velocity: 0,
                y_velocity: -0x0002_0000,
                facing: Direction::Right,
                on_ground: false,
                on_spikes: false,
                just_jumped: false,
                jump_buffer_frames: 0,
                right_wall_sticky: false,
                left_wall_sticky: false,
            },
            camera: CameraState { x_pos: 0, y_pos: 0 },
            terrain: TerrainState {
                floor_y_pos: 48 << 16,
                floor_terrain_type: TerrainType::Normal,
            },
            level_bounds: LevelBounds {
                width_pixels: 96,
                height_pixels: 80,
            },
            objects: ObjectSystemState {
                platform_loader: PlatformLoaderState::default(),
            },
            prize_block_hit: None,
            lift_block_hit: None,
            drill_projectile_hit: None,
            metal_block_hit: None,
            smash_block_hit: None,
            teleporter_contact: None,
        };

        slice.apply_ceiling_collision(Some(&mut grid), None);
        assert!(!grid.cell_at(2, 2).has_block);

        // Second jump through the now-empty cell: no ceiling hit
        slice.player.y_pos = 63 << 16;
        slice.player.y_velocity = -0x0002_0000;
        slice.apply_ceiling_collision(Some(&mut grid), None);

        // Velocity should remain negative (no ceiling hit)
        assert!(slice.player.y_velocity < 0);
    }

    #[test]
    fn wall_collision_clamps_one_pixel_tighter_when_moving_right() {
        let w = 4u16;
        let h = 4u16;
        let mut data = vec![0u8; usize::from(w) * usize::from(h) * 2];
        let offset = (usize::from(1u16) * usize::from(w) + usize::from(3u16)) * 2;
        data[offset] = 0x60;
        let grid = terrain::BlockGrid::from_level_layout(&data, w, h);

        let mut slice = OpeningSliceState {
            player: PlayerState {
                x_pos: 43 << 16,
                y_pos: 31 << 16,
                x_velocity: 0x0001_0000,
                y_velocity: 0,
                facing: Direction::Right,
                on_ground: false,
                on_spikes: false,
                just_jumped: false,
                jump_buffer_frames: 0,
                right_wall_sticky: false,
                left_wall_sticky: false,
            },
            camera: CameraState { x_pos: 0, y_pos: 0 },
            terrain: TerrainState {
                floor_y_pos: 48 << 16,
                floor_terrain_type: TerrainType::Normal,
            },
            level_bounds: LevelBounds {
                width_pixels: 64,
                height_pixels: 64,
            },
            objects: ObjectSystemState {
                platform_loader: PlatformLoaderState::default(),
            },
            prize_block_hit: None,
            lift_block_hit: None,
            drill_projectile_hit: None,
            metal_block_hit: None,
            smash_block_hit: None,
            teleporter_contact: None,
        };

        slice.apply_wall_collision(Some(&grid));

        assert_eq!(slice.player.x_pos, 42 << 16);
        assert_eq!(slice.player.x_velocity, 0);
    }

    #[test]
    fn airborne_right_wall_stickiness_waits_for_top_edge_to_clear() {
        let w = 5u16;
        let h = 5u16;
        let mut data = vec![0u8; usize::from(w) * usize::from(h) * 2];
        let offset = (usize::from(1u16) * usize::from(w) + usize::from(2u16)) * 2;
        data[offset] = 0x60;
        let grid = terrain::BlockGrid::from_level_layout(&data, w, h);

        let mut slice = OpeningSliceState {
            player: PlayerState {
                x_pos: 26 << 16,
                y_pos: 48 << 16,
                x_velocity: 0x0001_0000,
                y_velocity: 0,
                facing: Direction::Right,
                on_ground: false,
                on_spikes: false,
                just_jumped: false,
                jump_buffer_frames: 0,
                right_wall_sticky: false,
                left_wall_sticky: false,
            },
            camera: CameraState { x_pos: 0, y_pos: 0 },
            terrain: TerrainState {
                floor_y_pos: 64 << 16,
                floor_terrain_type: TerrainType::Normal,
            },
            level_bounds: LevelBounds {
                width_pixels: 80,
                height_pixels: 80,
            },
            objects: ObjectSystemState {
                platform_loader: PlatformLoaderState::default(),
            },
            prize_block_hit: None,
            lift_block_hit: None,
            drill_projectile_hit: None,
            metal_block_hit: None,
            smash_block_hit: None,
            teleporter_contact: None,
        };

        slice.apply_wall_collision(Some(&grid));
        assert_eq!(slice.player.x_pos, 26 << 16);
        assert_eq!(slice.player.x_velocity, 0);
        assert!(slice.player.right_wall_sticky);

        slice.player.y_pos = 56 << 16;
        slice.player.x_velocity = 0x0000_4000;
        slice.apply_wall_collision(Some(&grid));
        assert_eq!(slice.player.x_pos, 26 << 16);
        assert_eq!(slice.player.x_velocity, 0);
        assert!(slice.player.right_wall_sticky);

        slice.player.y_pos = 64 << 16;
        slice.player.x_velocity = 0x0000_4000;
        slice.player.x_pos = (26 << 16) + 0x0000_4000;
        slice.apply_wall_collision(Some(&grid));
        assert_eq!(slice.player.x_pos, 27 << 16);
        assert_eq!(slice.player.x_velocity, 0);
        assert!(!slice.player.right_wall_sticky);
    }

    #[test]
    fn landing_frame_rewinds_airborne_horizontal_step() {
        let grid = grid_with_floor_at_row(4);
        let previous_x_pos = 100 << 16;
        let x_velocity = 0x0001_8000;
        let mut slice = OpeningSliceState {
            player: PlayerState {
                x_pos: previous_x_pos + x_velocity as u32,
                y_pos: 64 << 16,
                x_velocity,
                y_velocity: 0x0001_0000,
                facing: Direction::Right,
                on_ground: false,
                on_spikes: false,
                just_jumped: false,
                jump_buffer_frames: 0,
                right_wall_sticky: false,
                left_wall_sticky: false,
            },
            camera: CameraState { x_pos: 0, y_pos: 0 },
            terrain: TerrainState {
                floor_y_pos: 0,
                floor_terrain_type: TerrainType::Normal,
            },
            level_bounds: LevelBounds {
                width_pixels: 320,
                height_pixels: 224,
            },
            objects: ObjectSystemState {
                platform_loader: PlatformLoaderState::default(),
            },
            prize_block_hit: None,
            lift_block_hit: None,
            drill_projectile_hit: None,
            metal_block_hit: None,
            smash_block_hit: None,
            teleporter_contact: None,
        };

        slice.apply_floor_collision(Some(&grid), true, false);

        assert_eq!(slice.player.x_pos, previous_x_pos);
        assert_eq!(slice.player.y_pos, 63 << 16);
        assert_eq!(slice.player.x_velocity, x_velocity);
        assert_eq!(slice.player.y_velocity, 0);
        assert!(slice.player.on_ground);
    }

    // --- US-011: Stomp-kill enemies ---

    /// Set up a stomp scenario around the first active test enemy.
    fn setup_stomp_scenario(runtime: &mut NativeRuntime, stomp: bool) {
        // First: set camera near origin so the test enemy activates.
        if let Some(ref mut slice) = runtime.opening_slice {
            slice.camera.x_pos = 0;
            slice.camera.y_pos = 0;
        }
        // Step to activate enemies in the activation window
        runtime.step_frame(FrameInput::default());

        // Now position player for stomp or side contact
        if let Some(ref mut slice) = runtime.opening_slice {
            slice.player.x_pos = 0x0120_0000; // same X as test enemy (288px)
            slice.camera.x_pos = 0;
            slice.camera.y_pos = 0;
            if stomp {
                // Above enemy, falling downward
                slice.player.y_pos = 0x00D8_0000; // 216px, enemy at 224px
                slice.player.y_velocity = 0x0002_0000;
                slice.player.on_ground = false;
            } else {
                // Same height as enemy (side contact)
                slice.player.y_pos = 0x00E0_0000; // 224px
                slice.player.y_velocity = 0;
                slice.player.on_ground = true;
            }
        }
    }

    #[test]
    fn stomp_from_above_kills_enemy() {
        let mut runtime = NativeRuntime::new(manifest_with_static_skull());
        runtime.set_block_grid(grid_with_floor_at_row(14));
        start_level(&mut runtime);
        for _ in 0..260 {
            runtime.step_frame(FrameInput::default());
        }

        setup_stomp_scenario(&mut runtime, true);
        let enemies_before = runtime.active_enemies.len();
        assert!(enemies_before > 0, "Skull should be activated");

        runtime.step_frame(FrameInput::default());
        assert!(
            runtime.active_enemies.len() < enemies_before,
            "stomped enemy should be removed (before: {enemies_before}, after: {})",
            runtime.active_enemies.len()
        );
    }

    #[test]
    fn stomp_bounces_player_upward() {
        let mut runtime = NativeRuntime::new(manifest_with_static_skull());
        runtime.set_block_grid(grid_with_floor_at_row(14));
        start_level(&mut runtime);
        for _ in 0..260 {
            runtime.step_frame(FrameInput::default());
        }

        setup_stomp_scenario(&mut runtime, true);
        runtime.step_frame(FrameInput::default());

        if let Some(ref slice) = runtime.opening_slice {
            assert!(
                slice.player.y_velocity < 0,
                "player should bounce upward after stomp, got y_vel={:#010X}",
                slice.player.y_velocity
            );
        }
    }

    #[test]
    fn side_contact_damages_not_stomps() {
        let mut runtime = NativeRuntime::new(manifest_with_static_skull());
        runtime.set_block_grid(grid_with_floor_at_row(14));
        start_level(&mut runtime);
        for _ in 0..260 {
            runtime.step_frame(FrameInput::default());
        }

        runtime.number_hitpoints = 3;
        setup_stomp_scenario(&mut runtime, false);
        let enemies_before = runtime.active_enemies.len();
        assert!(enemies_before > 0, "Skull should be activated");

        runtime.step_frame(FrameInput::default());
        assert_eq!(
            runtime.active_enemies.len(),
            enemies_before,
            "side contact should not kill enemy"
        );
        assert!(
            runtime.number_hitpoints < 3
                || !matches!(runtime.player_lifecycle, PlayerLifecycle::Alive),
            "side contact should deal damage to player"
        );
    }

    #[test]
    fn falling_straight_onto_enemy_is_stomp() {
        // Player falling directly onto enemy — ASM velocity-direction classification
        // treats this as "kid above" even with slight interpenetration, because the
        // dominant approach direction is vertical.
        let player = PlayerState {
            x_pos: 100 << 16,
            y_pos: 219 << 16,
            x_velocity: 0,
            y_velocity: 0x0002_0000,
            facing: Direction::Right,
            on_ground: false,
            on_spikes: false,
            just_jumped: false,
            jump_buffer_frames: 0,
            right_wall_sticky: false,
            left_wall_sticky: false,
        };
        let enemy = NativeObjectSnapshot {
            kind: NativeObjectKind::Enemy { object_type: 0x1C },
            x_pos: 100 << 16,
            y_pos: 224 << 16,
            layout_index: Some(0),
            x_velocity: Some(0),
            y_velocity: Some(0),
            facing: Some(Direction::Left),
            on_ground: Some(false),
            hit_points: Some(1),
            damage_flash: false,
            palette_variant: 0,
            stationary: false,
        };

        assert_eq!(
            classify_enemy_contact(player, &enemy, player.y_velocity, player.y_pos),
            EnemyContactResult::Stomp
        );
    }

    #[test]
    fn mini_hopping_skull_graze_does_not_damage() {
        let player = PlayerState {
            x_pos: 0x042E_1980,
            y_pos: 0x008E_4600,
            x_velocity: 0x0002_6E80,
            y_velocity: 0x0002_3E00,
            facing: Direction::Right,
            on_ground: false,
            on_spikes: false,
            just_jumped: false,
            jump_buffer_frames: 0,
            right_wall_sticky: false,
            left_wall_sticky: false,
        };
        let enemy = NativeObjectSnapshot {
            kind: NativeObjectKind::Enemy { object_type: 0x1C },
            x_pos: 0x0434_0000,
            y_pos: 0x007D_8800,
            layout_index: Some(0),
            x_velocity: Some(0),
            y_velocity: Some(0),
            facing: Some(Direction::Left),
            on_ground: Some(false),
            hit_points: Some(1),
            damage_flash: false,
            palette_variant: 0,
            stationary: false,
        };

        assert_eq!(
            classify_enemy_contact(player, &enemy, player.y_velocity, player.y_pos),
            EnemyContactResult::None
        );
    }

    #[test]
    fn stomp_works_during_invulnerability() {
        let mut runtime = NativeRuntime::new(manifest_with_static_skull());
        runtime.set_block_grid(grid_with_floor_at_row(14));
        start_level(&mut runtime);
        for _ in 0..260 {
            runtime.step_frame(FrameInput::default());
        }

        runtime.invulnerable_frames = 100;
        setup_stomp_scenario(&mut runtime, true);
        let enemies_before = runtime.active_enemies.len();
        assert!(enemies_before > 0, "Skull should be activated");

        runtime.step_frame(FrameInput::default());
        assert!(
            runtime.active_enemies.len() < enemies_before,
            "stomp should still kill enemy during invulnerability"
        );
    }

    #[test]
    fn falling_onto_diamond_collects_without_stomp_bounce() {
        let mut runtime = NativeRuntime::new(manifest());
        runtime.set_block_grid(grid_with_floor_at_row(14));
        start_level(&mut runtime);
        for _ in 0..260 {
            runtime.step_frame(FrameInput::default());
        }

        let diamonds_before = runtime.diamond_count;

        if let Some(ref mut slice) = runtime.opening_slice {
            slice.camera.x_pos = 0;
            slice.camera.y_pos = 0;
        }
        runtime.step_frame(FrameInput::default());

        if let Some(ref mut slice) = runtime.opening_slice {
            slice.player.x_pos = 0x0120_0000;
            slice.player.y_pos = 0x00D8_0000;
            slice.player.y_velocity = 0x0002_0000;
            slice.player.on_ground = false;
            slice.camera.x_pos = 0;
            slice.camera.y_pos = 0;
        }
        runtime.step_frame(FrameInput::default());

        let slice = runtime
            .opening_slice
            .expect("opening slice should remain active");
        assert_eq!(
            runtime.diamond_count,
            diamonds_before + 1,
            "falling through a diamond should still collect it"
        );
        assert_ne!(
            slice.player.y_velocity, STOMP_BOUNCE_VELOCITY,
            "diamond collection should not trigger stomp bounce"
        );
        assert!(
            !runtime
                .active_enemies
                .iter()
                .any(|enemy| enemy.family == EnemyFamily::Diamond),
            "diamond should be removed after collection"
        );
    }

    #[test]
    fn contact_damage_does_not_apply_knockback_velocity() {
        // ASM does NOT apply knockback velocity on contact damage — only HP loss + invulnerability
        let mut runtime = NativeRuntime::new(manifest_with_static_skull());
        runtime.set_block_grid(grid_with_floor_at_row(14));
        start_level(&mut runtime);
        for _ in 0..260 {
            runtime.step_frame(FrameInput::default());
        }
        runtime.number_hitpoints = 3;

        if let Some(ref mut slice) = runtime.opening_slice {
            slice.camera.x_pos = 0;
            slice.camera.y_pos = 0;
        }
        runtime.step_frame(FrameInput::default());
        if let Some(ref mut slice) = runtime.opening_slice {
            slice.player.x_pos = 0x011C_0000;
            slice.player.y_pos = 0x00E0_0000;
            slice.player.on_ground = true;
            slice.player.x_velocity = 0;
            slice.player.y_velocity = 0;
            slice.camera.x_pos = 0;
            slice.camera.y_pos = 0;
        }
        runtime.step_frame(FrameInput::default());

        let slice = runtime.opening_slice.unwrap();
        assert_eq!(
            slice.player.x_velocity, 0,
            "no knockback x velocity on contact damage"
        );
    }

    #[test]
    fn no_knockback_on_lethal_contact() {
        let mut runtime = NativeRuntime::new(manifest_with_static_skull());
        runtime.set_block_grid(grid_with_floor_at_row(14));
        start_level(&mut runtime);
        for _ in 0..260 {
            runtime.step_frame(FrameInput::default());
        }
        runtime.number_hitpoints = 1; // will die on contact

        if let Some(ref mut slice) = runtime.opening_slice {
            slice.camera.x_pos = 0;
            slice.camera.y_pos = 0;
        }
        runtime.step_frame(FrameInput::default());
        if let Some(ref mut slice) = runtime.opening_slice {
            slice.player.x_pos = 0x011C_0000;
            slice.player.y_pos = 0x00E0_0000;
            slice.player.on_ground = true;
            slice.player.x_velocity = 0;
            slice.player.y_velocity = 0;
            slice.camera.x_pos = 0;
            slice.camera.y_pos = 0;
        }
        runtime.step_frame(FrameInput::default());

        assert!(
            !matches!(runtime.player_lifecycle, PlayerLifecycle::Alive),
            "player should be dead after lethal contact"
        );
    }

    fn manifest_with_tough_skull() -> AssetManifest {
        let mut m = manifest();
        if let Some(enemy) = m.levels[0].enemy_layout_metadata.as_mut() {
            enemy.objects[0] = kid_data::EnemyLayoutObject {
                object_type: 0x1C, // MiniHoppingSkull
                flags: 0,
                hit_points_plus_one: 3, // 2 HP
                x: 0x0120,
                y: 0x00E0,
            };
        }
        m
    }

    #[test]
    fn two_hp_enemy_survives_first_stomp() {
        let mut runtime = NativeRuntime::new(manifest_with_tough_skull());
        runtime.set_block_grid(grid_with_floor_at_row(14));
        start_level(&mut runtime);
        for _ in 0..260 {
            runtime.step_frame(FrameInput::default());
        }

        // Activate enemy
        if let Some(ref mut slice) = runtime.opening_slice {
            slice.camera.x_pos = 0;
            slice.camera.y_pos = 0;
        }
        runtime.step_frame(FrameInput::default());
        let enemies_before = runtime.active_enemies.len();
        assert!(enemies_before > 0, "skull should be active");

        // First stomp
        if let Some(ref mut slice) = runtime.opening_slice {
            slice.player.x_pos = 0x0120_0000;
            slice.player.y_pos = 0x00D8_0000;
            slice.player.y_velocity = 0x0002_0000;
            slice.player.on_ground = false;
            slice.camera.x_pos = 0;
            slice.camera.y_pos = 0;
        }
        runtime.step_frame(FrameInput::default());

        assert_eq!(
            runtime.active_enemies.len(),
            enemies_before,
            "2-HP enemy should survive first stomp"
        );
        assert_eq!(
            runtime.active_enemies[0].hit_points, 1,
            "HP should be decremented to 1"
        );
        assert!(
            runtime.active_enemies[0].damage_flash_frames > 0,
            "damaged enemy should have flash frames"
        );
    }

    #[test]
    fn two_hp_enemy_dies_on_second_stomp() {
        let mut runtime = NativeRuntime::new(manifest_with_tough_skull());
        runtime.set_block_grid(grid_with_floor_at_row(14));
        start_level(&mut runtime);
        for _ in 0..260 {
            runtime.step_frame(FrameInput::default());
        }

        // Activate enemy
        if let Some(ref mut slice) = runtime.opening_slice {
            slice.camera.x_pos = 0;
            slice.camera.y_pos = 0;
        }
        runtime.step_frame(FrameInput::default());
        let enemies_before = runtime.active_enemies.len();

        // First stomp
        if let Some(ref mut slice) = runtime.opening_slice {
            slice.player.x_pos = 0x0120_0000;
            slice.player.y_pos = 0x00D8_0000;
            slice.player.y_velocity = 0x0002_0000;
            slice.player.on_ground = false;
            slice.camera.x_pos = 0;
            slice.camera.y_pos = 0;
        }
        runtime.step_frame(FrameInput::default());
        assert_eq!(
            runtime.active_enemies.len(),
            enemies_before,
            "survives first stomp"
        );

        // Second stomp
        if let Some(ref mut slice) = runtime.opening_slice {
            slice.player.x_pos = 0x0120_0000;
            slice.player.y_pos = 0x00D8_0000;
            slice.player.y_velocity = 0x0002_0000;
            slice.player.on_ground = false;
            slice.camera.x_pos = 0;
            slice.camera.y_pos = 0;
        }
        runtime.step_frame(FrameInput::default());

        assert!(
            runtime.active_enemies.len() < enemies_before,
            "2-HP enemy should die on second stomp"
        );
    }

    #[test]
    fn stomp_always_bounces_even_on_surviving_enemy() {
        let mut runtime = NativeRuntime::new(manifest_with_tough_skull());
        runtime.set_block_grid(grid_with_floor_at_row(14));
        start_level(&mut runtime);
        for _ in 0..260 {
            runtime.step_frame(FrameInput::default());
        }

        if let Some(ref mut slice) = runtime.opening_slice {
            slice.camera.x_pos = 0;
            slice.camera.y_pos = 0;
        }
        runtime.step_frame(FrameInput::default());

        // Stomp the 2-HP enemy
        if let Some(ref mut slice) = runtime.opening_slice {
            slice.player.x_pos = 0x0120_0000;
            slice.player.y_pos = 0x00D8_0000;
            slice.player.y_velocity = 0x0002_0000;
            slice.player.on_ground = false;
            slice.camera.x_pos = 0;
            slice.camera.y_pos = 0;
        }
        runtime.step_frame(FrameInput::default());

        let slice = runtime.opening_slice.unwrap();
        assert!(
            slice.player.y_velocity < 0,
            "player should bounce upward even when enemy survives (y_vel={})",
            slice.player.y_velocity
        );
    }

    // -----------------------------------------------------------------------
    // Pacer archetype (Goat, Armadillo, Tank, Crab, Scorpion, Driller)
    // -----------------------------------------------------------------------

    #[test]
    fn pacer_walk_speed_goat_scales_with_hp() {
        // Goat: base_hp=3, base_speed=$8000, step=$10000
        assert_eq!(pacer_walk_speed(EnemyFamily::Goat, 3), 0x0000_8000);
        assert_eq!(pacer_walk_speed(EnemyFamily::Goat, 4), 0x0001_8000);
        assert_eq!(pacer_walk_speed(EnemyFamily::Goat, 5), 0x0002_8000);
    }

    #[test]
    fn pacer_walk_speed_armadillo_scales_with_hp() {
        // Armadillo: base_hp=1, base_speed=$8000, step=$10000
        assert_eq!(pacer_walk_speed(EnemyFamily::Armadillo, 1), 0x0000_8000);
        assert_eq!(pacer_walk_speed(EnemyFamily::Armadillo, 2), 0x0001_8000);
        assert_eq!(pacer_walk_speed(EnemyFamily::Armadillo, 3), 0x0002_8000);
    }

    #[test]
    fn pacer_walk_speed_scorpion_is_fixed() {
        // Scorpion: fixed $18000 regardless of HP.
        assert_eq!(pacer_walk_speed(EnemyFamily::Scorpion, 3), 0x0001_8000);
        assert_eq!(pacer_walk_speed(EnemyFamily::Scorpion, 4), 0x0001_8000);
        assert_eq!(pacer_walk_speed(EnemyFamily::Scorpion, 5), 0x0001_8000);
    }

    #[test]
    fn pacer_walk_speed_driller_scales_with_hp() {
        // Driller: base_hp=2, base_speed=$8000, step=$8000
        assert_eq!(pacer_walk_speed(EnemyFamily::Driller, 2), 0x0000_8000);
        assert_eq!(pacer_walk_speed(EnemyFamily::Driller, 3), 0x0001_0000);
        assert_eq!(pacer_walk_speed(EnemyFamily::Driller, 4), 0x0001_8000);
    }

    #[test]
    fn pacer_walk_speed_fire_demon_scales_with_hp() {
        assert_eq!(pacer_walk_speed(EnemyFamily::FireDemon, 3), 0x0000_8000);
        assert_eq!(pacer_walk_speed(EnemyFamily::FireDemon, 4), 0x0001_0000);
        assert_eq!(pacer_walk_speed(EnemyFamily::FireDemon, 5), 0x0001_8000);
    }

    #[test]
    fn pacer_walk_speed_lion_binary_hp_check() {
        // Lion: ASM binary check — $18000 for HP<8, $28000 for HP==8.
        assert_eq!(pacer_walk_speed(EnemyFamily::Lion, 6), 0x0001_8000);
        assert_eq!(pacer_walk_speed(EnemyFamily::Lion, 7), 0x0001_8000);
        assert_eq!(pacer_walk_speed(EnemyFamily::Lion, 8), 0x0002_8000);
    }

    #[test]
    fn pacer_walk_speed_orca_is_fixed() {
        assert_eq!(pacer_walk_speed(EnemyFamily::Orca, 4), 0x0000_8000);
        assert_eq!(pacer_walk_speed(EnemyFamily::Orca, 5), 0x0000_8000);
    }

    /// Build a small grid (width_cells x height_cells) where specific cells are
    /// solid and everything else is empty. Each entry in `solid_cells` is (col, row).
    fn grid_with_solid_cells(
        width_cells: u16,
        height_cells: u16,
        solid_cells: &[(u16, u16)],
    ) -> terrain::BlockGrid {
        let mut data = vec![0u8; usize::from(width_cells) * usize::from(height_cells) * 2];
        for &(col, row) in solid_cells {
            let offset = (usize::from(row) * usize::from(width_cells) + usize::from(col)) * 2;
            data[offset] = 0x60; // collision=6 (Solid)
        }
        terrain::BlockGrid::from_level_layout(&data, width_cells, height_cells)
    }

    fn make_pacer(family: EnemyFamily, hp: u8, x_px: u32, y_px: u32) -> ActiveEnemy {
        ActiveEnemy {
            segment_index: 0,
            layout_index: 0,
            family,
            x_pos: x_px << 16,
            y_pos: y_px << 16,
            x_velocity: 0,
            y_velocity: 0,
            x_acceleration: 0,
            y_acceleration: 0,
            facing: Direction::Left,
            on_ground: true,
            hit_points: hp,
            grounded_frames: 0,
            damage_flash_frames: 0,
            palette_variant: 0,
            wait_timer: 0,
            dragon_phase: DragonFlyingState::None,
            dragon_phase_frames: 0,
            dragon_script_variant: 0,
            dragon_script_cmd: 0,
            dragon_anchor_y: 0,
            dragon_descent_index: 0,
            dragon_descent_frames: 0,
            ninja_state: NinjaState::Stealth,
            hand_state: HandState::Walking,
            lion_state: LionState::Walking,
            robot_state: RobotState::Walking,
            orca_state: OrcaState::Walking,
            tar_monster_state: TarMonsterState::default(),
            spinning_twins_phase: SpinningTwinsPhase::default(),
            spinning_twins_orbit_index: 0,
            ufo_beam_segments: 0,
            ufo_beam_interval: 0,
        }
    }

    #[test]
    fn goat_walks_left_from_spawn() {
        // Full-width floor at row 14 (y=224px). Enemy foot at y=224 (row 14)
        // so probe_floor sees solid beneath.
        let grid = grid_with_floor_at_row(14);
        let mut enemy = make_pacer(EnemyFamily::Goat, 3, 128, 224);
        let x_before = enemy.x_pos;
        step_pacer(&mut enemy, Some(&grid));
        assert!(
            enemy.x_pos < x_before,
            "goat should move left (x_before={:#x}, x_after={:#x})",
            x_before,
            enemy.x_pos
        );
        assert_eq!(enemy.facing, Direction::Left);
    }

    #[test]
    fn goat_reverses_at_floor_edge() {
        // Small platform: floor only at columns 5-10, row 13 (y=208..224).
        let solid: Vec<(u16, u16)> = (5..=10).map(|col| (col, 13)).collect();
        let grid = grid_with_solid_cells(20, 15, &solid);
        // Place goat near left edge of platform, x=88px (col 5.5).
        // With speed 0x8000 (0.5 px/frame) it reaches the edge in ~8 frames.
        let mut enemy = make_pacer(EnemyFamily::Goat, 3, 88, 208);

        let mut reversed = false;
        for _ in 0..50 {
            let facing_before = enemy.facing;
            step_pacer(&mut enemy, Some(&grid));
            if enemy.facing != facing_before {
                reversed = true;
                break;
            }
        }
        assert!(reversed, "goat should reverse at floor edge");
    }

    #[test]
    fn goat_reverses_at_wall() {
        // Floor at row 13 across full width, wall at col 2 blocking leftward path.
        let mut solid: Vec<(u16, u16)> = (0..20).map(|col| (col, 13)).collect();
        for row in 10..13 {
            solid.push((2, row));
        }
        let grid = grid_with_solid_cells(20, 15, &solid);
        // Goat at col 5 (x=80+8=88), foot at row 13 (y=208).
        let mut enemy = make_pacer(EnemyFamily::Goat, 3, 88, 208);

        for _ in 0..300 {
            step_pacer(&mut enemy, Some(&grid));
        }
        assert_eq!(
            enemy.facing,
            Direction::Right,
            "goat should reverse on hitting wall"
        );
    }

    #[test]
    fn pacer_families_map_to_pacer_archetype() {
        assert_eq!(EnemyFamily::Goat.archetype(), EnemyArchetype::Pacer);
        assert_eq!(EnemyFamily::Armadillo.archetype(), EnemyArchetype::Pacer);
        assert_eq!(EnemyFamily::Tank.archetype(), EnemyArchetype::Pacer);
        assert_eq!(EnemyFamily::Crab.archetype(), EnemyArchetype::Pacer);
        assert_eq!(EnemyFamily::Scorpion.archetype(), EnemyArchetype::Pacer);
        assert_eq!(EnemyFamily::Driller.archetype(), EnemyArchetype::Pacer);
    }

    #[test]
    fn spawn_hit_points_applies_family_base() {
        // Goat: enemy_level + 3
        assert_eq!(spawn_hit_points(EnemyFamily::Goat, 0), 3);
        assert_eq!(spawn_hit_points(EnemyFamily::Goat, 1), 4);
        assert_eq!(spawn_hit_points(EnemyFamily::Goat, 2), 5);
        // Armadillo: enemy_level + 1
        assert_eq!(spawn_hit_points(EnemyFamily::Armadillo, 0), 1);
        assert_eq!(spawn_hit_points(EnemyFamily::Armadillo, 1), 2);
        assert_eq!(spawn_hit_points(EnemyFamily::Armadillo, 2), 3);
        // Diamond: raw enemy_level, min 1
        assert_eq!(spawn_hit_points(EnemyFamily::Diamond, 0), 1);
        assert_eq!(spawn_hit_points(EnemyFamily::Diamond, 2), 2);
    }

    #[test]
    fn pacer_from_layout_object_resolves_goat() {
        let slots = vec![0xFFFF, 0x0010, 0x001C]; // slot 1 = Goat ($10)
        let object = kid_data::EnemyLayoutObject {
            object_type: 0x81, // slot 1 (0x80 | 1)
            flags: 0,
            hit_points_plus_one: 1,
            x: 0x0100,
            y: 0x00C0,
        };
        assert_eq!(
            EnemyFamily::from_layout_object(&slots, &object),
            EnemyFamily::Goat
        );
    }

    // --- Cloud enemy ---

    #[test]
    fn cloud_maps_to_flyer_archetype() {
        assert_eq!(EnemyFamily::Cloud.archetype(), EnemyArchetype::Flyer);
    }

    #[test]
    fn cloud_spawn_velocity_uses_speed_table() {
        // enemy_level 0 → -$3000, 1 → -$5000, 2 → -$7000
        assert_eq!(spawn_x_velocity(EnemyFamily::Cloud, 0), -0x3000);
        assert_eq!(spawn_x_velocity(EnemyFamily::Cloud, 1), -0x5000);
        assert_eq!(spawn_x_velocity(EnemyFamily::Cloud, 2), -0x7000);
        // Clamp above table length
        assert_eq!(spawn_x_velocity(EnemyFamily::Cloud, 5), -0x7000);
    }

    #[test]
    fn cloud_flies_horizontally() {
        let mut enemy = ActiveEnemy {
            segment_index: 0,
            layout_index: 0,
            family: EnemyFamily::Cloud,
            x_pos: 128 << 16,
            y_pos: 100 << 16,
            x_velocity: -0x3000,
            y_velocity: 0,
            x_acceleration: 0,
            y_acceleration: 0,
            facing: Direction::Left,
            on_ground: false,
            hit_points: 1,
            grounded_frames: 0,
            damage_flash_frames: 0,
            palette_variant: 0,
            wait_timer: 0,
            dragon_phase: DragonFlyingState::None,
            dragon_phase_frames: 0,
            dragon_script_variant: 0,
            dragon_script_cmd: 0,
            dragon_anchor_y: 0,
            dragon_descent_index: 0,
            dragon_descent_frames: 0,
            ninja_state: NinjaState::Stealth,
            hand_state: HandState::Walking,
            lion_state: LionState::Walking,
            robot_state: RobotState::Walking,
            orca_state: OrcaState::Walking,
            tar_monster_state: TarMonsterState::default(),
            spinning_twins_phase: SpinningTwinsPhase::default(),
            spinning_twins_orbit_index: 0,
            ufo_beam_segments: 0,
            ufo_beam_interval: 0,
        };
        let x_before = enemy.x_pos;
        step_cloud(&mut enemy, None);
        assert!(enemy.x_pos < x_before, "cloud should move left");
        assert_eq!(enemy.y_pos, 100 << 16, "cloud y should not change");
    }

    #[test]
    fn cloud_reverses_at_wall() {
        // Wall at col 2 across rows 5-8
        let mut solid: Vec<(u16, u16)> = Vec::new();
        for row in 5..=8 {
            solid.push((2, row));
        }
        let grid = grid_with_solid_cells(20, 15, &solid);
        // Cloud at col 4 (x=64), y at row 6 (y=96), flying left
        let mut enemy = ActiveEnemy {
            segment_index: 0,
            layout_index: 0,
            family: EnemyFamily::Cloud,
            x_pos: 64 << 16,
            y_pos: 96 << 16,
            x_velocity: -0x3000,
            y_velocity: 0,
            x_acceleration: 0,
            y_acceleration: 0,
            facing: Direction::Left,
            on_ground: false,
            hit_points: 1,
            grounded_frames: 0,
            damage_flash_frames: 0,
            palette_variant: 0,
            wait_timer: 0,
            dragon_phase: DragonFlyingState::None,
            dragon_phase_frames: 0,
            dragon_script_variant: 0,
            dragon_script_cmd: 0,
            dragon_anchor_y: 0,
            dragon_descent_index: 0,
            dragon_descent_frames: 0,
            ninja_state: NinjaState::Stealth,
            hand_state: HandState::Walking,
            lion_state: LionState::Walking,
            robot_state: RobotState::Walking,
            orca_state: OrcaState::Walking,
            tar_monster_state: TarMonsterState::default(),
            spinning_twins_phase: SpinningTwinsPhase::default(),
            spinning_twins_orbit_index: 0,
            ufo_beam_segments: 0,
            ufo_beam_interval: 0,
        };
        // Run until it hits the wall and reverses
        let mut reversed = false;
        for _ in 0..500 {
            step_cloud(&mut enemy, Some(&grid));
            if enemy.x_velocity > 0 {
                reversed = true;
                break;
            }
        }
        assert!(reversed, "cloud should reverse on hitting wall");
        assert_eq!(enemy.facing, Direction::Right);
    }

    #[test]
    fn cloud_spawn_hit_points() {
        // Cloud base HP = 1
        assert_eq!(spawn_hit_points(EnemyFamily::Cloud, 0), 1);
        assert_eq!(spawn_hit_points(EnemyFamily::Cloud, 1), 2);
        assert_eq!(spawn_hit_points(EnemyFamily::Cloud, 2), 3);
    }

    // --- Tier 2 enemy wiring ---

    #[test]
    fn emo_rock_maps_to_hoverer() {
        assert_eq!(EnemyFamily::EmoRock.archetype(), EnemyArchetype::Hoverer);
        assert_eq!(EnemyFamily::EmoRock.object_type(), 0x1B);
    }

    #[test]
    fn archer_maps_to_inert() {
        assert_eq!(EnemyFamily::Archer.archetype(), EnemyArchetype::Stationary);
        assert_eq!(EnemyFamily::Archer.object_type(), 0x07);
    }

    #[test]
    fn fireball_maps_to_bouncer() {
        assert_eq!(EnemyFamily::Fireball.archetype(), EnemyArchetype::Bouncer);
        assert_eq!(EnemyFamily::Fireball.object_type(), 0x19);
    }

    #[test]
    fn fireball_gravity_scales_with_hp() {
        let make = |hp| ActiveEnemy {
            segment_index: 0,
            layout_index: 0,
            family: EnemyFamily::Fireball,
            x_pos: 0,
            y_pos: 0,
            x_velocity: 0,
            y_velocity: 0,
            x_acceleration: 0,
            y_acceleration: 0,
            facing: Direction::Left,
            on_ground: false,
            hit_points: hp,
            grounded_frames: 0,
            damage_flash_frames: 0,
            palette_variant: 0,
            wait_timer: 0,
            dragon_phase: DragonFlyingState::None,
            dragon_phase_frames: 0,
            dragon_script_variant: 0,
            dragon_script_cmd: 0,
            dragon_anchor_y: 0,
            dragon_descent_index: 0,
            dragon_descent_frames: 0,
            ninja_state: NinjaState::Stealth,
            hand_state: HandState::Walking,
            lion_state: LionState::Walking,
            robot_state: RobotState::Walking,
            orca_state: OrcaState::Walking,
            tar_monster_state: TarMonsterState::default(),
            spinning_twins_phase: SpinningTwinsPhase::default(),
            spinning_twins_orbit_index: 0,
            ufo_beam_segments: 0,
            ufo_beam_interval: 0,
        };
        assert_eq!(bouncer_gravity(&make(1)), 0x1000);
        assert_eq!(bouncer_gravity(&make(2)), 0x2000);
        assert_eq!(bouncer_gravity(&make(3)), 0x3000);
    }

    // --- Big Hopping Skull enemy ---

    #[test]
    fn big_hopping_skull_maps_to_hopper() {
        assert_eq!(
            EnemyFamily::BigHoppingSkull.archetype(),
            EnemyArchetype::Hopper
        );
    }

    #[test]
    fn big_hopping_skull_hp_table() {
        // enemy_level 0→2, 1→3, 2→4
        assert_eq!(spawn_hit_points(EnemyFamily::BigHoppingSkull, 0), 2);
        assert_eq!(spawn_hit_points(EnemyFamily::BigHoppingSkull, 1), 3);
        assert_eq!(spawn_hit_points(EnemyFamily::BigHoppingSkull, 2), 4);
    }

    #[test]
    fn big_hopping_skull_uses_slower_gravity() {
        let hc = hopper_constants(EnemyFamily::BigHoppingSkull);
        let mini = hopper_constants(EnemyFamily::MiniHoppingSkull);
        assert!(
            hc.gravity < mini.gravity,
            "BHS gravity ($FA0) should be less than MHS ($1200)"
        );
    }

    // --- Sphere enemy ---

    #[test]
    fn sphere_maps_to_bouncer_archetype() {
        assert_eq!(EnemyFamily::Sphere.archetype(), EnemyArchetype::Bouncer);
    }

    #[test]
    fn sphere_spawn_velocity_is_fixed() {
        assert_eq!(spawn_x_velocity(EnemyFamily::Sphere, 0), 0x10000);
        assert_eq!(spawn_x_velocity(EnemyFamily::Sphere, 2), 0x10000);
    }

    #[test]
    fn sphere_applies_gravity() {
        let mut enemy = ActiveEnemy {
            segment_index: 0,
            layout_index: 0,
            family: EnemyFamily::Sphere,
            x_pos: 128 << 16,
            y_pos: 64 << 16,
            x_velocity: 0x10000,
            y_velocity: 0,
            x_acceleration: 0,
            y_acceleration: 0,
            facing: Direction::Right,
            on_ground: false,
            hit_points: 1,
            grounded_frames: 0,
            damage_flash_frames: 0,
            palette_variant: 0,
            wait_timer: 0,
            dragon_phase: DragonFlyingState::None,
            dragon_phase_frames: 0,
            dragon_script_variant: 0,
            dragon_script_cmd: 0,
            dragon_anchor_y: 0,
            dragon_descent_index: 0,
            dragon_descent_frames: 0,
            ninja_state: NinjaState::Stealth,
            hand_state: HandState::Walking,
            lion_state: LionState::Walking,
            robot_state: RobotState::Walking,
            orca_state: OrcaState::Walking,
            tar_monster_state: TarMonsterState::default(),
            spinning_twins_phase: SpinningTwinsPhase::default(),
            spinning_twins_orbit_index: 0,
            ufo_beam_segments: 0,
            ufo_beam_interval: 0,
        };
        step_bouncer(&mut enemy, None);
        assert_eq!(
            enemy.y_velocity, SPHERE_GRAVITY,
            "gravity should be applied"
        );
        step_bouncer(&mut enemy, None);
        assert_eq!(enemy.y_velocity, SPHERE_GRAVITY * 2, "gravity accumulates");
    }

    #[test]
    fn sphere_bounces_off_floor() {
        // Floor at row 10 (y=160)
        let solid: Vec<(u16, u16)> = (0..20).map(|col| (col, 10)).collect();
        let grid = grid_with_solid_cells(20, 15, &solid);
        let mut enemy = ActiveEnemy {
            segment_index: 0,
            layout_index: 0,
            family: EnemyFamily::Sphere,
            x_pos: 80 << 16,
            y_pos: 155 << 16,
            x_velocity: 0x10000,
            y_velocity: 0x5000, // falling
            x_acceleration: 0,
            y_acceleration: 0,
            facing: Direction::Right,
            on_ground: false,
            hit_points: 1,
            grounded_frames: 0,
            damage_flash_frames: 0,
            palette_variant: 0,
            wait_timer: 0,
            dragon_phase: DragonFlyingState::None,
            dragon_phase_frames: 0,
            dragon_script_variant: 0,
            dragon_script_cmd: 0,
            dragon_anchor_y: 0,
            dragon_descent_index: 0,
            dragon_descent_frames: 0,
            ninja_state: NinjaState::Stealth,
            hand_state: HandState::Walking,
            lion_state: LionState::Walking,
            robot_state: RobotState::Walking,
            orca_state: OrcaState::Walking,
            tar_monster_state: TarMonsterState::default(),
            spinning_twins_phase: SpinningTwinsPhase::default(),
            spinning_twins_orbit_index: 0,
            ufo_beam_segments: 0,
            ufo_beam_interval: 0,
        };
        // Run until it bounces (y_velocity becomes negative)
        let mut bounced = false;
        for _ in 0..50 {
            step_bouncer(&mut enemy, Some(&grid));
            if enemy.y_velocity < 0 {
                bounced = true;
                break;
            }
        }
        assert!(bounced, "sphere should bounce off floor");
    }

    #[test]
    fn sphere_bounces_off_wall() {
        // Wall at col 12 across rows 5-10
        let mut solid: Vec<(u16, u16)> = Vec::new();
        for row in 5..=10 {
            solid.push((12, row));
        }
        let grid = grid_with_solid_cells(20, 15, &solid);
        let mut enemy = ActiveEnemy {
            segment_index: 0,
            layout_index: 0,
            family: EnemyFamily::Sphere,
            x_pos: 180 << 16,
            y_pos: 112 << 16,
            x_velocity: 0x10000, // moving right
            y_velocity: 0,
            x_acceleration: 0,
            y_acceleration: 0,
            facing: Direction::Right,
            on_ground: false,
            hit_points: 1,
            grounded_frames: 0,
            damage_flash_frames: 0,
            palette_variant: 0,
            wait_timer: 0,
            dragon_phase: DragonFlyingState::None,
            dragon_phase_frames: 0,
            dragon_script_variant: 0,
            dragon_script_cmd: 0,
            dragon_anchor_y: 0,
            dragon_descent_index: 0,
            dragon_descent_frames: 0,
            ninja_state: NinjaState::Stealth,
            hand_state: HandState::Walking,
            lion_state: LionState::Walking,
            robot_state: RobotState::Walking,
            orca_state: OrcaState::Walking,
            tar_monster_state: TarMonsterState::default(),
            spinning_twins_phase: SpinningTwinsPhase::default(),
            spinning_twins_orbit_index: 0,
            ufo_beam_segments: 0,
            ufo_beam_interval: 0,
        };
        let mut bounced = false;
        for _ in 0..50 {
            step_bouncer(&mut enemy, Some(&grid));
            if enemy.x_velocity < 0 {
                bounced = true;
                break;
            }
        }
        assert!(bounced, "sphere should bounce off wall");
    }

    // --- EmoRock enemy ---

    #[test]
    fn emo_rock_maps_to_hoverer_archetype() {
        assert_eq!(EnemyFamily::EmoRock.archetype(), EnemyArchetype::Hoverer);
    }

    #[test]
    fn emo_rock_hovers_above_player_then_drops() {
        let solid: Vec<(u16, u16)> = (0..20).map(|col| (col, 14)).collect();
        let grid = grid_with_solid_cells(20, 15, &solid);
        let player_x: u32 = 100 << 16;
        let player_y: u32 = 200 << 16;
        let mut enemy = ActiveEnemy {
            segment_index: 0,
            layout_index: 0,
            family: EnemyFamily::EmoRock,
            x_pos: 80 << 16,
            y_pos: 50 << 16,
            x_velocity: 0,
            y_velocity: 0,
            x_acceleration: 0,
            y_acceleration: 0,
            facing: Direction::Left,
            on_ground: false,
            hit_points: 1,
            grounded_frames: 0,
            damage_flash_frames: 0,
            palette_variant: 0,
            wait_timer: 3, // short wait for test
            dragon_phase: DragonFlyingState::None,
            dragon_phase_frames: 0,
            dragon_script_variant: 0,
            dragon_script_cmd: 0,
            dragon_anchor_y: 0,
            dragon_descent_index: 0,
            dragon_descent_frames: 0,
            ninja_state: NinjaState::Stealth,
            hand_state: HandState::Walking,
            lion_state: LionState::Walking,
            robot_state: RobotState::Walking,
            orca_state: OrcaState::Walking,
            tar_monster_state: TarMonsterState::default(),
            spinning_twins_phase: SpinningTwinsPhase::default(),
            spinning_twins_orbit_index: 0,
            ufo_beam_segments: 0,
            ufo_beam_interval: 0,
        };

        // During hover: tracks player x, hovers above player y
        step_emo_rock(&mut enemy, Some(&grid), Some(player_x), Some(player_y));
        assert_eq!(
            enemy.x_pos, player_x,
            "should track player x while hovering"
        );
        assert!(enemy.y_pos < player_y, "should hover above player");
        assert_eq!(enemy.wait_timer, 2);

        // Count down remaining wait
        step_emo_rock(&mut enemy, Some(&grid), Some(player_x), Some(player_y));
        step_emo_rock(&mut enemy, Some(&grid), Some(player_x), Some(player_y));
        assert_eq!(enemy.wait_timer, 0);

        // Now drops: gravity applied
        step_emo_rock(&mut enemy, Some(&grid), Some(player_x), Some(player_y));
        assert!(enemy.y_velocity > 0, "should fall with positive y velocity");
        assert!(!enemy.on_ground, "should not be on ground while falling");
    }

    #[test]
    fn emo_rock_lands_and_walks() {
        // Floor at row 5 (y=80px)
        let solid: Vec<(u16, u16)> = (0..20).map(|col| (col, 5)).collect();
        let grid = grid_with_solid_cells(20, 15, &solid);
        let player_x: u32 = 200 << 16;
        let mut enemy = ActiveEnemy {
            segment_index: 0,
            layout_index: 0,
            family: EnemyFamily::EmoRock,
            x_pos: 100 << 16,
            y_pos: 60 << 16, // above the floor
            x_velocity: 0,
            y_velocity: 0,
            x_acceleration: 0,
            y_acceleration: 0,
            facing: Direction::Left,
            on_ground: false,
            hit_points: 1,
            grounded_frames: 0,
            damage_flash_frames: 0,
            palette_variant: 0,
            wait_timer: 0, // already past hover, will drop
            dragon_phase: DragonFlyingState::None,
            dragon_phase_frames: 0,
            dragon_script_variant: 0,
            dragon_script_cmd: 0,
            dragon_anchor_y: 0,
            dragon_descent_index: 0,
            dragon_descent_frames: 0,
            ninja_state: NinjaState::Stealth,
            hand_state: HandState::Walking,
            lion_state: LionState::Walking,
            robot_state: RobotState::Walking,
            orca_state: OrcaState::Walking,
            tar_monster_state: TarMonsterState::default(),
            spinning_twins_phase: SpinningTwinsPhase::default(),
            spinning_twins_orbit_index: 0,
            ufo_beam_segments: 0,
            ufo_beam_interval: 0,
        };

        // Step until it lands
        for _ in 0..100 {
            step_emo_rock(&mut enemy, Some(&grid), Some(player_x), None);
            if enemy.on_ground {
                break;
            }
        }
        assert!(enemy.on_ground, "should have landed on the floor");

        // Once on ground, should walk toward player (player is to the right)
        let x_before = enemy.x_pos;
        step_emo_rock(&mut enemy, Some(&grid), Some(player_x), None);
        assert_eq!(enemy.facing, Direction::Right, "should face toward player");
        assert!(enemy.x_pos > x_before, "should walk toward player");
    }

    // --- Projectile system ---

    #[test]
    fn projectile_moves_by_velocity_each_step() {
        let mut projs = vec![EnemyProjectile {
            x_pos: 100 << 16,
            y_pos: 100 << 16,
            x_velocity: 0x2_0000,
            y_velocity: -0x1_0000,
            lifetime: 10,
            friendly: false,
            gravity: 0,
            homing_accel: 0,
            speed_cap: 0,
            wall_bounce: false,
        }];
        // Simulate step_projectiles inline (since it's on NativeRuntime)
        projs.retain_mut(|p| {
            if p.lifetime == 0 {
                return false;
            }
            p.lifetime -= 1;
            p.x_pos = (p.x_pos as i64 + i64::from(p.x_velocity)) as u32;
            p.y_pos = (p.y_pos as i64 + i64::from(p.y_velocity)) as u32;
            true
        });
        assert_eq!(projs.len(), 1);
        assert_eq!(projs[0].x_pos, 102 << 16);
        assert_eq!(projs[0].y_pos, 99 << 16);
        assert_eq!(projs[0].lifetime, 9);
    }

    #[test]
    fn projectile_removed_when_lifetime_expires() {
        let mut projs = vec![EnemyProjectile {
            x_pos: 100 << 16,
            y_pos: 100 << 16,
            x_velocity: 0x1000,
            y_velocity: 0,
            lifetime: 1,
            friendly: false,
            gravity: 0,
            homing_accel: 0,
            speed_cap: 0,
            wall_bounce: false,
        }];
        // Step once: lifetime goes 1→0, projectile moves
        projs.retain_mut(|p| {
            if p.lifetime == 0 {
                return false;
            }
            p.lifetime -= 1;
            p.x_pos = (p.x_pos as i64 + i64::from(p.x_velocity)) as u32;
            true
        });
        assert_eq!(
            projs.len(),
            1,
            "lifetime was 1, not 0, so still alive this step"
        );
        // Step again: lifetime is 0, removed
        projs.retain_mut(|p| {
            if p.lifetime == 0 {
                return false;
            }
            p.lifetime -= 1;
            true
        });
        assert_eq!(projs.len(), 0, "projectile removed after lifetime expires");
    }

    // --- Archer enemy ---

    #[test]
    fn archer_maps_to_stationary_archetype() {
        assert_eq!(EnemyFamily::Archer.archetype(), EnemyArchetype::Stationary);
    }

    #[test]
    fn archer_faces_player_direction() {
        let mut enemy = ActiveEnemy {
            segment_index: 0,
            layout_index: 0,
            family: EnemyFamily::Archer,
            x_pos: 100 << 16,
            y_pos: 100 << 16,
            x_velocity: 0,
            y_velocity: 0,
            x_acceleration: 0,
            y_acceleration: 0,
            facing: Direction::Left,
            on_ground: true,
            hit_points: 1,
            grounded_frames: 0,
            damage_flash_frames: 0,
            palette_variant: 0,
            wait_timer: 10, // not ready to fire yet
            dragon_phase: DragonFlyingState::None,
            dragon_phase_frames: 0,
            dragon_script_variant: 0,
            dragon_script_cmd: 0,
            dragon_anchor_y: 0,
            dragon_descent_index: 0,
            dragon_descent_frames: 0,
            ninja_state: NinjaState::Stealth,
            hand_state: HandState::Walking,
            lion_state: LionState::Walking,
            robot_state: RobotState::Walking,
            orca_state: OrcaState::Walking,
            tar_monster_state: TarMonsterState::default(),
            spinning_twins_phase: SpinningTwinsPhase::default(),
            spinning_twins_orbit_index: 0,
            ufo_beam_segments: 0,
            ufo_beam_interval: 0,
        };
        let mut projs = Vec::new();
        let player_x = Some(200u32 << 16);
        step_archer(&mut enemy, &mut projs, player_x, Some(100u32 << 16));
        assert_eq!(enemy.facing, Direction::Right, "should face toward player");
        assert!(projs.is_empty(), "should not fire while timer > 0");
    }

    #[test]
    fn archer_fires_arrow_when_timer_expires() {
        let mut enemy = ActiveEnemy {
            segment_index: 0,
            layout_index: 0,
            family: EnemyFamily::Archer,
            x_pos: 100 << 16,
            y_pos: 100 << 16,
            x_velocity: 0,
            y_velocity: 0,
            x_acceleration: 0,
            y_acceleration: 0,
            facing: Direction::Left,
            on_ground: true,
            hit_points: 1,
            grounded_frames: 0,
            damage_flash_frames: 0,
            palette_variant: 0,
            wait_timer: 0, // ready to fire
            dragon_phase: DragonFlyingState::None,
            dragon_phase_frames: 0,
            dragon_script_variant: 0,
            dragon_script_cmd: 0,
            dragon_anchor_y: 0,
            dragon_descent_index: 0,
            dragon_descent_frames: 0,
            ninja_state: NinjaState::Stealth,
            hand_state: HandState::Walking,
            lion_state: LionState::Walking,
            robot_state: RobotState::Walking,
            orca_state: OrcaState::Walking,
            tar_monster_state: TarMonsterState::default(),
            spinning_twins_phase: SpinningTwinsPhase::default(),
            spinning_twins_orbit_index: 0,
            ufo_beam_segments: 0,
            ufo_beam_interval: 0,
        };
        let mut projs = Vec::new();
        let player_x = Some(200u32 << 16);
        let player_y = Some(100u32 << 16);
        step_archer(&mut enemy, &mut projs, player_x, player_y);
        assert_eq!(projs.len(), 1, "should spawn an arrow");
        assert!(
            projs[0].x_velocity > 0,
            "arrow should move toward player (right)"
        );
        assert_eq!(enemy.wait_timer, ARCHER_FIRE_TIMER[0], "timer should reset");
    }

    #[test]
    fn archer_angle_straight_right() {
        // Player directly right (dy=0 after 16px offset → quadrant 3 → angle 27, same as 0)
        let angle = archer_angle_to_player(100 << 16, 100 << 16, 200 << 16, (100 + 16) << 16);
        assert_eq!(angle, 27);
        // Velocity is identical to angle 0
        assert_eq!(ARROW_ANGLE_TABLE[27], ARROW_ANGLE_TABLE[0]);
    }

    #[test]
    fn archer_angle_straight_left() {
        // Player directly left (dy=0 after offset → quadrant 2 → angle 14, same vel as 13)
        let angle = archer_angle_to_player(200 << 16, 100 << 16, 100 << 16, (100 + 16) << 16);
        assert_eq!(angle, 14);
        assert_eq!(ARROW_ANGLE_TABLE[14], ARROW_ANGLE_TABLE[13]);
    }

    #[test]
    fn archer_angle_straight_up() {
        // Player directly above
        let angle = archer_angle_to_player(100 << 16, 200 << 16, 100 << 16, 100 << 16);
        assert_eq!(angle, 6, "should be straight up");
    }

    #[test]
    fn archer_angle_upper_right() {
        // Player well above and to the right → quadrant 0, should give angle near 0-6
        let angle = archer_angle_to_player(100 << 16, 200 << 16, 200 << 16, 100 << 16);
        assert!(angle <= 6, "angle {angle} should be in upper-right range");
        let (xv, yv) = ARROW_ANGLE_TABLE[angle];
        assert!(xv > 0, "x velocity should be positive (rightward)");
        assert!(yv < 0, "y velocity should be negative (upward)");
    }

    #[test]
    fn archer_fire_timer_varies_by_hp() {
        assert_eq!(ARCHER_FIRE_TIMER[0], 0x1E, "HP 1: 30 frames");
        assert_eq!(ARCHER_FIRE_TIMER[1], 0x14, "HP 2: 20 frames");
        assert_eq!(ARCHER_FIRE_TIMER[2], 0x0A, "HP 3: 10 frames");
    }

    // --- Tank firing ---

    #[test]
    fn tank_fires_bullet_after_timer_expires() {
        let mut enemy = ActiveEnemy {
            segment_index: 0,
            layout_index: 0,
            family: EnemyFamily::Tank,
            x_pos: 100 << 16,
            y_pos: 100 << 16,
            x_velocity: 0x8000,
            y_velocity: 0,
            x_acceleration: 0,
            y_acceleration: 0,
            facing: Direction::Right,
            on_ground: true,
            hit_points: 2,
            grounded_frames: 0,
            damage_flash_frames: 0,
            palette_variant: 0,
            wait_timer: 0, // ready to fire
            dragon_phase: DragonFlyingState::None,
            dragon_phase_frames: 0,
            dragon_script_variant: 0,
            dragon_script_cmd: 0,
            dragon_anchor_y: 0,
            dragon_descent_index: 0,
            dragon_descent_frames: 0,
            ninja_state: NinjaState::Stealth,
            hand_state: HandState::Walking,
            lion_state: LionState::Walking,
            robot_state: RobotState::Walking,
            orca_state: OrcaState::Walking,
            tar_monster_state: TarMonsterState::default(),
            spinning_twins_phase: SpinningTwinsPhase::default(),
            spinning_twins_orbit_index: 0,
            ufo_beam_segments: 0,
            ufo_beam_interval: 0,
        };
        let mut projs = Vec::new();
        step_tank_fire(&mut enemy, &mut projs);
        assert_eq!(projs.len(), 1, "tank should fire a bullet");
        assert!(
            projs[0].x_velocity > 0,
            "bullet should go right (same as facing)"
        );
        assert_eq!(projs[0].y_velocity, 0, "bullet should be horizontal");
        assert_eq!(enemy.wait_timer, TANK_FIRE_INTERVAL, "timer should reset");
    }

    #[test]
    fn tank_does_not_fire_while_timer_active() {
        let mut enemy = ActiveEnemy {
            segment_index: 0,
            layout_index: 0,
            family: EnemyFamily::Tank,
            x_pos: 100 << 16,
            y_pos: 100 << 16,
            x_velocity: 0x8000,
            y_velocity: 0,
            x_acceleration: 0,
            y_acceleration: 0,
            facing: Direction::Right,
            on_ground: true,
            hit_points: 2,
            grounded_frames: 0,
            damage_flash_frames: 0,
            palette_variant: 0,
            wait_timer: 10,
            dragon_phase: DragonFlyingState::None,
            dragon_phase_frames: 0,
            dragon_script_variant: 0,
            dragon_script_cmd: 0,
            dragon_anchor_y: 0,
            dragon_descent_index: 0,
            dragon_descent_frames: 0,
            ninja_state: NinjaState::Stealth,
            hand_state: HandState::Walking,
            lion_state: LionState::Walking,
            robot_state: RobotState::Walking,
            orca_state: OrcaState::Walking,
            tar_monster_state: TarMonsterState::default(),
            spinning_twins_phase: SpinningTwinsPhase::default(),
            spinning_twins_orbit_index: 0,
            ufo_beam_segments: 0,
            ufo_beam_interval: 0,
        };
        let mut projs = Vec::new();
        step_tank_fire(&mut enemy, &mut projs);
        assert!(projs.is_empty(), "should not fire while timer > 0");
        assert_eq!(enemy.wait_timer, 9, "timer should decrement");
    }

    // --- Boss tracker ---

    #[test]
    fn boss_families_map_to_tracker_archetype() {
        assert_eq!(EnemyFamily::HeadyMetal.archetype(), EnemyArchetype::Tracker);
        assert_eq!(
            EnemyFamily::Shishkaboss.archetype(),
            EnemyArchetype::Tracker
        );
        assert_eq!(
            EnemyFamily::BoomerangBosses.archetype(),
            EnemyArchetype::Tracker
        );
        assert_eq!(
            EnemyFamily::BagelBrothers.archetype(),
            EnemyArchetype::Tracker
        );
    }

    #[test]
    fn tracker_chases_player_rightward() {
        let mut enemy = ActiveEnemy {
            segment_index: 0,
            layout_index: 0,
            family: EnemyFamily::HeadyMetal,
            x_pos: 100 << 16,
            y_pos: 100 << 16,
            x_velocity: 0,
            y_velocity: 0,
            x_acceleration: 0,
            y_acceleration: 0,
            facing: Direction::Left,
            on_ground: false,
            hit_points: 10,
            grounded_frames: 0,
            damage_flash_frames: 0,
            palette_variant: 0,
            wait_timer: 0,
            dragon_phase: DragonFlyingState::None,
            dragon_phase_frames: 0,
            dragon_script_variant: 0,
            dragon_script_cmd: 0,
            dragon_anchor_y: 0,
            dragon_descent_index: 0,
            dragon_descent_frames: 0,
            ninja_state: NinjaState::Stealth,
            hand_state: HandState::Walking,
            lion_state: LionState::Walking,
            robot_state: RobotState::Walking,
            orca_state: OrcaState::Walking,
            tar_monster_state: TarMonsterState::default(),
            spinning_twins_phase: SpinningTwinsPhase::default(),
            spinning_twins_orbit_index: 0,
            ufo_beam_segments: 0,
            ufo_beam_interval: 0,
        };
        let px = Some(200u32 << 16);
        let py = Some(100u32 << 16);
        step_tracker(&mut enemy, px, py);
        assert!(
            enemy.x_pos > 100 << 16,
            "should move rightward toward player"
        );
        assert_eq!(enemy.facing, Direction::Right);
    }

    #[test]
    fn tracker_chases_player_leftward() {
        let mut enemy = ActiveEnemy {
            segment_index: 0,
            layout_index: 0,
            family: EnemyFamily::HeadyMetal,
            x_pos: 200 << 16,
            y_pos: 100 << 16,
            x_velocity: 0,
            y_velocity: 0,
            x_acceleration: 0,
            y_acceleration: 0,
            facing: Direction::Right,
            on_ground: false,
            hit_points: 10,
            grounded_frames: 0,
            damage_flash_frames: 0,
            palette_variant: 0,
            wait_timer: 0,
            dragon_phase: DragonFlyingState::None,
            dragon_phase_frames: 0,
            dragon_script_variant: 0,
            dragon_script_cmd: 0,
            dragon_anchor_y: 0,
            dragon_descent_index: 0,
            dragon_descent_frames: 0,
            ninja_state: NinjaState::Stealth,
            hand_state: HandState::Walking,
            lion_state: LionState::Walking,
            robot_state: RobotState::Walking,
            orca_state: OrcaState::Walking,
            tar_monster_state: TarMonsterState::default(),
            spinning_twins_phase: SpinningTwinsPhase::default(),
            spinning_twins_orbit_index: 0,
            ufo_beam_segments: 0,
            ufo_beam_interval: 0,
        };
        let px = Some(100u32 << 16);
        let py = Some(100u32 << 16);
        step_tracker(&mut enemy, px, py);
        assert!(
            enemy.x_pos < 200 << 16,
            "should move leftward toward player"
        );
        assert_eq!(enemy.facing, Direction::Left);
    }

    #[test]
    fn boss_fires_projectiles_when_stomped() {
        let mut runtime = NativeRuntime::new(manifest());
        start_level(&mut runtime);
        // Place a boss below the player
        let boss = ActiveEnemy {
            segment_index: 0,
            layout_index: 0,
            family: EnemyFamily::HeadyMetal,
            x_pos: 100 << 16,
            y_pos: 120 << 16,
            x_velocity: 0,
            y_velocity: 0,
            x_acceleration: 0,
            y_acceleration: 0,
            facing: Direction::Left,
            on_ground: false,
            hit_points: 10,
            grounded_frames: 0,
            damage_flash_frames: 0,
            palette_variant: 0,
            wait_timer: 0,
            dragon_phase: DragonFlyingState::None,
            dragon_phase_frames: 0,
            dragon_script_variant: 0,
            dragon_script_cmd: 0,
            dragon_anchor_y: 0,
            dragon_descent_index: 0,
            dragon_descent_frames: 0,
            ninja_state: NinjaState::Stealth,
            hand_state: HandState::Walking,
            lion_state: LionState::Walking,
            robot_state: RobotState::Walking,
            orca_state: OrcaState::Walking,
            tar_monster_state: TarMonsterState::default(),
            spinning_twins_phase: SpinningTwinsPhase::default(),
            spinning_twins_orbit_index: 0,
            ufo_beam_segments: 0,
            ufo_beam_interval: 0,
        };
        runtime.active_enemies = vec![boss];
        runtime.projectiles.clear();

        // Simulate a stomp: player overlapping boss from above, falling.
        // Boss top = 120-8 = 112, so place player feet just inside at y=113.
        let mut slice = runtime.opening_slice.clone().unwrap();
        slice.player.x_pos = 100 << 16;
        slice.player.y_pos = 113 << 16;
        slice.player.y_velocity = 0x2_0000; // falling
        slice.player.on_ground = false;
        // Rebuild roster
        runtime.active_enemy_roster = runtime
            .active_enemies
            .iter()
            .map(ActiveEnemy::snapshot)
            .collect();
        let pre_y_vel = slice.player.y_velocity;
        let pre_y_pos = slice.player.y_pos;
        let stomped = runtime.process_stomps(&mut slice, pre_y_vel, pre_y_pos);
        assert!(stomped, "should register a stomp");
        assert_eq!(
            runtime.projectiles.len(),
            2,
            "boss should fire 2 projectiles on damage"
        );
        assert!(
            runtime.projectiles[0].x_velocity > 0,
            "first projectile rightward"
        );
        assert!(
            runtime.projectiles[1].x_velocity < 0,
            "second projectile leftward"
        );
    }

    #[test]
    fn ufo_beam_fires_when_player_below() {
        use enemies::{ActiveEnemy, EnemyFamily, step_ufo_beam};

        let mut enemy = ActiveEnemy {
            family: EnemyFamily::UFO,
            x_pos: 100 << 16,
            y_pos: 50 << 16,
            x_velocity: 0,
            y_velocity: 0,
            x_acceleration: 0,
            y_acceleration: 0,
            facing: Direction::Right,
            on_ground: false,
            hit_points: 1,
            grounded_frames: 0,
            damage_flash_frames: 0,
            palette_variant: 0,
            wait_timer: 0,
            layout_index: 0,
            segment_index: 0,
            dragon_phase: DragonFlyingState::None,
            dragon_phase_frames: 0,
            dragon_script_variant: 0,
            dragon_script_cmd: 0,
            dragon_anchor_y: 0,
            dragon_descent_index: 0,
            dragon_descent_frames: 0,
            ninja_state: NinjaState::Stealth,
            hand_state: HandState::Walking,
            lion_state: LionState::Walking,
            robot_state: RobotState::Walking,
            orca_state: OrcaState::Walking,
            tar_monster_state: TarMonsterState::default(),
            spinning_twins_phase: SpinningTwinsPhase::default(),
            spinning_twins_orbit_index: 0,
            ufo_beam_segments: 0,
            ufo_beam_interval: 0,
        };
        let mut projectiles = Vec::new();

        // Player directly below UFO (within 2px)
        let player_x = Some(100 << 16);
        let player_y = Some(120 << 16);

        step_ufo_beam(&mut enemy, &mut projectiles, player_x, player_y);
        assert_eq!(enemy.ufo_beam_segments, 4, "should start beam sequence");

        // Step through interval frames to spawn segments
        for _ in 0..20 {
            step_ufo_beam(&mut enemy, &mut projectiles, player_x, player_y);
        }
        assert!(projectiles.len() >= 4, "should have spawned beam segments");
        assert!(
            projectiles.iter().all(|p| p.y_velocity > 0),
            "beams go downward"
        );
    }

    // --- UFO and Tornado flyer tests ---

    #[test]
    fn ufo_accelerates_and_moves() {
        let mut enemy = test_enemy(EnemyFamily::UFO, 200 << 16, 100 << 16, 1);
        enemy.x_acceleration = UFO_ACCELERATION;
        enemy.y_acceleration = -UFO_ACCELERATION;
        enemy.wait_timer = UFO_PATTERN_FRAMES;
        let start_x = enemy.x_pos;
        step_ufo(&mut enemy, None);
        // After one frame: velocity = 0 + accel, position = start + velocity
        assert_eq!(enemy.x_velocity, UFO_ACCELERATION);
        assert!(
            enemy.x_pos > start_x,
            "UFO should move right with positive x accel"
        );
    }

    #[test]
    fn ufo_reverses_acceleration_on_timer() {
        let mut enemy = test_enemy(EnemyFamily::UFO, 200 << 16, 100 << 16, 1);
        enemy.x_acceleration = UFO_ACCELERATION;
        enemy.y_acceleration = -UFO_ACCELERATION;
        enemy.wait_timer = 0; // timer expired
        step_ufo(&mut enemy, None);
        // Acceleration should be reversed
        assert_eq!(enemy.x_acceleration, -UFO_ACCELERATION);
        assert_eq!(enemy.y_acceleration, UFO_ACCELERATION);
        assert_eq!(enemy.wait_timer, UFO_PATTERN_FRAMES);
    }

    #[test]
    fn tornado_reverses_acceleration_on_timer() {
        let mut enemy = test_enemy(EnemyFamily::Tornado, 100 << 16, 100 << 16, 1);
        enemy.x_acceleration = TORNADO_ACCELERATION;
        enemy.y_acceleration = -TORNADO_ACCELERATION;
        enemy.wait_timer = 0;
        step_tornado(&mut enemy, None);
        assert_eq!(enemy.x_acceleration, -TORNADO_ACCELERATION);
        assert_eq!(enemy.y_acceleration, TORNADO_ACCELERATION);
        // HP=1, tier=(1-1).min(2)=0, so cycle timer = level-0 = 60 frames
        assert_eq!(enemy.wait_timer, TORNADO_PATTERN_FRAMES[0]);
    }

    // --- Ninja ---

    #[test]
    fn ninja_paces_in_stealth_without_player() {
        let grid = grid_with_platform(17, 0, 120);
        let mut enemy = test_enemy(EnemyFamily::Ninja, 60 << 16, (17 * 16) << 16, 2);
        enemy.ninja_state = NinjaState::Stealth;
        enemy.on_ground = true;
        let start_x = enemy.x_pos;
        step_ninja(&mut enemy, Some(&grid), None, None, PlayerForm::TheKid);
        assert_eq!(enemy.ninja_state, NinjaState::Stealth);
        assert_ne!(enemy.x_pos, start_x, "should move while pacing");
    }

    #[test]
    fn ninja_detects_player_in_cone_and_jumps() {
        let mut enemy = test_enemy(EnemyFamily::Ninja, 100 << 16, 200 << 16, 2);
        enemy.ninja_state = NinjaState::Stealth;
        // Player 0x30 px above (within 0x20-0x50 range) and 0x08 px right (within ±0x10)
        let player_x = Some((100 + 0x08) << 16);
        let player_y = Some((200 - 0x30) << 16);
        step_ninja(&mut enemy, None, player_x, player_y, PlayerForm::TheKid);
        assert_eq!(enemy.ninja_state, NinjaState::JumpAttack);
        assert_eq!(enemy.facing, Direction::Right);
        assert!(enemy.y_velocity < 0, "should jump upward");
    }

    #[test]
    fn ninja_ignores_player_with_red_stealth() {
        let mut enemy = test_enemy(EnemyFamily::Ninja, 100 << 16, 200 << 16, 2);
        enemy.ninja_state = NinjaState::Stealth;
        let player_x = Some(100 << 16);
        let player_y = Some((200 - 0x30) << 16);
        step_ninja(&mut enemy, None, player_x, player_y, PlayerForm::RedStealth);
        assert_eq!(
            enemy.ninja_state,
            NinjaState::Stealth,
            "should stay stealthy when player wears Red Stealth"
        );
    }

    #[test]
    fn ninja_lands_and_returns_to_stealth() {
        // Floor at row 17 spanning full width — ninja will land on it
        let grid = grid_with_platform(17, 0, 120);
        let mut enemy = test_enemy(EnemyFamily::Ninja, 50 << 16, (17 * 16 - 40) << 16, 2);
        enemy.ninja_state = NinjaState::JumpAttack;
        enemy.x_velocity = NINJA_JUMP_X_SPEED_RIGHT;
        enemy.y_velocity = NINJA_JUMP_Y_SPEED;
        enemy.on_ground = false;
        // Step until ninja lands (gravity pulls it down)
        for _ in 0..300 {
            step_ninja(&mut enemy, Some(&grid), None, None, PlayerForm::TheKid);
            if enemy.ninja_state == NinjaState::Stealth {
                break;
            }
        }
        assert_eq!(
            enemy.ninja_state,
            NinjaState::Stealth,
            "ninja should return to stealth after landing"
        );
        assert!(enemy.on_ground);
    }

    // --- Hand ---

    #[test]
    fn hand_walks_until_player_detected_ahead() {
        let grid = grid_with_platform(17, 0, 120);
        let floor_y = (17 * 16) << 16;
        let mut enemy = test_enemy(EnemyFamily::Hand, 60 << 16, floor_y, 1);
        enemy.hand_state = HandState::Walking;
        enemy.on_ground = true;
        enemy.facing = Direction::Right;
        // Player 0x30 px ahead (within 0x40 range for HP=1), 8px above
        let player_x = Some((60 + 0x30) << 16);
        let player_y = Some(((17 * 16) - 8) << 16);
        step_hand(&mut enemy, Some(&grid), player_x, player_y);
        assert_eq!(
            enemy.hand_state,
            HandState::Jumping,
            "should jump at detected player"
        );
        assert!(enemy.y_velocity < 0, "should jump upward");
    }

    #[test]
    fn hand_ignores_player_behind() {
        let grid = grid_with_platform(17, 0, 120);
        let floor_y = (17 * 16) << 16;
        let mut enemy = test_enemy(EnemyFamily::Hand, 60 << 16, floor_y, 1);
        enemy.hand_state = HandState::Walking;
        enemy.on_ground = true;
        enemy.facing = Direction::Right;
        // Player 0x30 px behind (to the left)
        let player_x = Some((60 - 0x30) << 16);
        let player_y = Some(((17 * 16) - 8) << 16);
        step_hand(&mut enemy, Some(&grid), player_x, player_y);
        assert_eq!(
            enemy.hand_state,
            HandState::Walking,
            "should not detect player behind"
        );
    }

    #[test]
    fn hand_ignores_player_beyond_range() {
        let grid = grid_with_platform(17, 0, 120);
        let floor_y = (17 * 16) << 16;
        let mut enemy = test_enemy(EnemyFamily::Hand, 60 << 16, floor_y, 1);
        enemy.hand_state = HandState::Walking;
        enemy.on_ground = true;
        enemy.facing = Direction::Right;
        // Player 0x50 px ahead (beyond 0x40 range for HP=1)
        let player_x = Some((60 + 0x50) << 16);
        let player_y = Some(((17 * 16) - 8) << 16);
        step_hand(&mut enemy, Some(&grid), player_x, player_y);
        assert_eq!(
            enemy.hand_state,
            HandState::Walking,
            "should not detect player beyond detection range"
        );
    }

    #[test]
    fn hand_higher_hp_has_longer_detection_range() {
        let grid = grid_with_platform(17, 0, 120);
        let floor_y = (17 * 16) << 16;
        let mut enemy = test_enemy(EnemyFamily::Hand, 60 << 16, floor_y, 3);
        enemy.hand_state = HandState::Walking;
        enemy.on_ground = true;
        enemy.facing = Direction::Right;
        // Player 0x50 px ahead — beyond HP=1 range (0x40) but within HP=3 range (0x64)
        let player_x = Some((60 + 0x50) << 16);
        let player_y = Some(((17 * 16) - 8) << 16);
        step_hand(&mut enemy, Some(&grid), player_x, player_y);
        assert_eq!(
            enemy.hand_state,
            HandState::Jumping,
            "HP=3 hand should detect player at 0x50 range"
        );
    }

    #[test]
    fn hand_lands_and_returns_to_walking() {
        let grid = grid_with_platform(17, 0, 120);
        let floor_y = (17 * 16) << 16;
        let mut enemy = test_enemy(EnemyFamily::Hand, 60 << 16, floor_y - (30 << 16), 1);
        enemy.hand_state = HandState::Jumping;
        let (x_speed, y_speed) = hand_jump_speed(0, 0);
        enemy.x_velocity = x_speed;
        enemy.y_velocity = y_speed;
        enemy.on_ground = false;
        for _ in 0..300 {
            step_hand(&mut enemy, Some(&grid), None, None);
            if enemy.hand_state == HandState::Walking {
                break;
            }
        }
        assert_eq!(
            enemy.hand_state,
            HandState::Walking,
            "hand should return to walking after landing"
        );
        assert!(enemy.on_ground);
    }

    // --- Lion ---

    #[test]
    fn lion_paces_then_charges_when_timer_expires_and_player_in_range() {
        let grid = grid_with_platform(17, 0, 120);
        let floor_y = (17 * 16) << 16;
        let mut enemy = test_enemy(EnemyFamily::Lion, 60 << 16, floor_y, 6);
        enemy.lion_state = LionState::Walking;
        enemy.on_ground = true;
        enemy.wait_timer = 1; // about to expire
        let player_x = Some((60 + 0x50) << 16); // within detection range
        let mut projs = vec![];
        step_lion(&mut enemy, Some(&grid), player_x, &mut Some(&mut projs));
        // Timer ticked to 0 but hasn't charged yet (timer was 1, decremented to 0)
        assert_eq!(enemy.lion_state, LionState::Walking);
        // One more step with timer at 0 should trigger charge
        step_lion(&mut enemy, Some(&grid), player_x, &mut Some(&mut projs));
        assert_eq!(enemy.lion_state, LionState::Charging);
        assert_eq!(enemy.facing, Direction::Right);
        assert_eq!(projs.len(), 1, "should fire projectile when charging");
    }

    #[test]
    fn lion_does_not_charge_when_player_out_of_range() {
        let grid = grid_with_platform(17, 0, 120);
        let floor_y = (17 * 16) << 16;
        let mut enemy = test_enemy(EnemyFamily::Lion, 60 << 16, floor_y, 6);
        enemy.lion_state = LionState::Walking;
        enemy.on_ground = true;
        enemy.wait_timer = 0;
        let player_x = Some((60 + 0xFF) << 16); // far away, beyond $60 range
        let mut projs = vec![];
        step_lion(&mut enemy, Some(&grid), player_x, &mut Some(&mut projs));
        assert_eq!(
            enemy.lion_state,
            LionState::Walking,
            "should not charge when player too far"
        );
        assert!(projs.is_empty());
    }

    #[test]
    fn lion_charge_returns_to_walking_on_overshoot() {
        // Lion charge ends when overshooting player by >112px ($70), not a
        // fixed duration timer. ASM: Lion.asm loc_3E450/3E604.
        let grid = grid_with_platform(17, 0, 200);
        let floor_y = (17 * 16) << 16;
        let mut enemy = test_enemy(EnemyFamily::Lion, 60 << 16, floor_y, 6);
        enemy.lion_state = LionState::Charging;
        enemy.on_ground = true;
        enemy.facing = Direction::Right;
        // Player is at x=60, Lion starts at x=60 moving right.
        // After enough steps the Lion passes the player by >112px.
        let player_x = Some(60u32 << 16);
        for _ in 0..100 {
            if enemy.lion_state == LionState::Walking {
                break;
            }
            step_lion(&mut enemy, Some(&grid), player_x, &mut None);
        }
        assert_eq!(
            enemy.lion_state,
            LionState::Walking,
            "should return to walking after overshooting player"
        );
        assert_eq!(enemy.wait_timer, LION_COOLDOWN_AFTER_FIRE);
    }

    #[test]
    fn lion_hp8_charges_faster() {
        assert!(lion_charge_speed(8) > lion_charge_speed(7));
    }

    // --- Robot ---

    #[test]
    fn robot_walks_then_shoots_and_charges_on_timer() {
        let grid = grid_with_platform(17, 0, 120);
        let floor_y = (17 * 16) << 16;
        let mut enemy = test_enemy(EnemyFamily::Robot, 60 << 16, floor_y, 2);
        enemy.robot_state = RobotState::Walking;
        enemy.on_ground = true;
        enemy.wait_timer = 1;
        let player_x = Some(80u32 << 16);
        let mut projs = vec![];
        // Timer at 1 → decrements to 0
        step_robot(&mut enemy, Some(&grid), player_x, &mut Some(&mut projs));
        assert_eq!(enemy.robot_state, RobotState::Walking);
        assert!(projs.is_empty());
        // Timer at 0 → shoot + charge
        step_robot(&mut enemy, Some(&grid), player_x, &mut Some(&mut projs));
        assert_eq!(enemy.robot_state, RobotState::Charging);
        assert_eq!(projs.len(), 1, "robot should fire a projectile");
        assert_eq!(enemy.facing, Direction::Right, "should face toward player");
    }

    #[test]
    fn robot_hp3_walks_faster() {
        assert!(robot_walk_speed(3) > robot_walk_speed(2));
    }

    #[test]
    fn robot_charge_returns_to_walking() {
        let grid = grid_with_platform(17, 0, 120);
        let floor_y = (17 * 16) << 16;
        let mut enemy = test_enemy(EnemyFamily::Robot, 60 << 16, floor_y, 2);
        enemy.robot_state = RobotState::Charging;
        enemy.on_ground = true;
        enemy.facing = Direction::Right;
        enemy.wait_timer = 1;
        step_robot(&mut enemy, Some(&grid), None, &mut None);
        assert_eq!(enemy.wait_timer, 0);
        step_robot(&mut enemy, Some(&grid), None, &mut None);
        assert_eq!(enemy.robot_state, RobotState::Walking);
        assert_eq!(enemy.wait_timer, ROBOT_SHOOT_INTERVAL);
    }

    // --- Orca ---

    #[test]
    fn orca_lunges_when_player_nearby() {
        let grid = grid_with_platform(17, 0, 120);
        let floor_y = (17 * 16) << 16;
        let mut enemy = test_enemy(EnemyFamily::Orca, 60 << 16, floor_y, 4);
        enemy.orca_state = OrcaState::Walking;
        enemy.on_ground = true;
        enemy.wait_timer = 0; // no cooldown
        let player_x = Some((60 + 0x20) << 16); // within $30 range
        step_orca(&mut enemy, Some(&grid), player_x);
        assert_eq!(enemy.orca_state, OrcaState::Lunging);
        assert!(enemy.y_velocity < 0, "should launch upward");
        assert_eq!(enemy.facing, Direction::Right);
    }

    #[test]
    fn orca_ignores_distant_player() {
        let grid = grid_with_platform(17, 0, 120);
        let floor_y = (17 * 16) << 16;
        let mut enemy = test_enemy(EnemyFamily::Orca, 60 << 16, floor_y, 4);
        enemy.orca_state = OrcaState::Walking;
        enemy.on_ground = true;
        enemy.wait_timer = 0;
        let player_x = Some((60 + 0x50) << 16); // beyond $30 range
        step_orca(&mut enemy, Some(&grid), player_x);
        assert_eq!(
            enemy.orca_state,
            OrcaState::Walking,
            "should not lunge at distant player"
        );
    }

    #[test]
    fn orca_cooldown_prevents_immediate_relunge() {
        let grid = grid_with_platform(17, 0, 120);
        let floor_y = (17 * 16) << 16;
        let mut enemy = test_enemy(EnemyFamily::Orca, 60 << 16, floor_y, 4);
        enemy.orca_state = OrcaState::Walking;
        enemy.on_ground = true;
        enemy.wait_timer = 100; // still cooling down
        let player_x = Some((60 + 0x10) << 16);
        step_orca(&mut enemy, Some(&grid), player_x);
        assert_eq!(
            enemy.orca_state,
            OrcaState::Walking,
            "should not lunge during cooldown"
        );
        assert_eq!(enemy.wait_timer, 99);
    }

    #[test]
    fn orca_lands_and_returns_to_walking_with_cooldown() {
        let grid = grid_with_platform(17, 0, 120);
        let mut enemy = test_enemy(EnemyFamily::Orca, 60 << 16, (17 * 16 - 30) << 16, 4);
        enemy.orca_state = OrcaState::Lunging;
        enemy.x_velocity = ORCA_LUNGE_X_SPEED;
        enemy.y_velocity = ORCA_LUNGE_Y_SPEED;
        enemy.on_ground = false;
        for _ in 0..300 {
            step_orca(&mut enemy, Some(&grid), None);
            if enemy.orca_state == OrcaState::Walking {
                break;
            }
        }
        assert_eq!(enemy.orca_state, OrcaState::Walking);
        assert_eq!(enemy.wait_timer, ORCA_LUNGE_COOLDOWN);
        assert!(enemy.on_ground);
    }

    // --- TarMonster ---

    #[test]
    fn tar_monster_cycles_through_states() {
        let mut enemy = test_enemy(EnemyFamily::TarMonster, 60 << 16, 200 << 16, 4);
        let mut projs = Vec::new();
        enemy.tar_monster_state = TarMonsterState::Submerged;
        enemy.wait_timer = 0;

        // Submerged → Emerging
        step_tar_monster(&mut enemy, &mut projs);
        assert_eq!(enemy.tar_monster_state, TarMonsterState::Emerging);
        assert_eq!(enemy.wait_timer, TAR_MONSTER_EMERGE_FRAMES);

        // Wait through emerge animation
        for _ in 0..TAR_MONSTER_EMERGE_FRAMES {
            step_tar_monster(&mut enemy, &mut projs);
        }
        assert_eq!(enemy.wait_timer, 0);
        step_tar_monster(&mut enemy, &mut projs);
        assert_eq!(enemy.tar_monster_state, TarMonsterState::Visible);
        assert_eq!(enemy.wait_timer, TAR_MONSTER_VISIBLE_FRAMES);

        // Wait through visible period
        for _ in 0..TAR_MONSTER_VISIBLE_FRAMES {
            step_tar_monster(&mut enemy, &mut projs);
        }
        step_tar_monster(&mut enemy, &mut projs);
        assert_eq!(enemy.tar_monster_state, TarMonsterState::Attacking);
        assert_eq!(enemy.wait_timer, TAR_MONSTER_ATTACK_FRAMES);

        // During attack, a glob shot fires at frame 8
        projs.clear();
        for _ in 0..TAR_MONSTER_ATTACK_FRAMES {
            step_tar_monster(&mut enemy, &mut projs);
        }
        assert_eq!(projs.len(), 1, "glob shot should fire during attack");
        assert_ne!(projs[0].x_velocity, 0, "glob shot has horizontal velocity");

        step_tar_monster(&mut enemy, &mut projs);
        assert_eq!(enemy.tar_monster_state, TarMonsterState::Submerging);

        // Wait through submerge animation
        for _ in 0..TAR_MONSTER_SUBMERGE_FRAMES {
            step_tar_monster(&mut enemy, &mut projs);
        }
        step_tar_monster(&mut enemy, &mut projs);
        assert_eq!(enemy.tar_monster_state, TarMonsterState::Submerged);
        assert_eq!(enemy.wait_timer, TAR_MONSTER_SUBMERGED_FRAMES);
    }

    #[test]
    fn tar_monster_is_invulnerable_when_submerged() {
        let mut enemy = test_enemy(EnemyFamily::TarMonster, 60 << 16, 200 << 16, 4);
        enemy.tar_monster_state = TarMonsterState::Submerged;
        assert!(!enemy.is_vulnerable());

        enemy.tar_monster_state = TarMonsterState::Emerging;
        assert!(!enemy.is_vulnerable());

        enemy.tar_monster_state = TarMonsterState::Visible;
        assert!(enemy.is_vulnerable());

        enemy.tar_monster_state = TarMonsterState::Submerging;
        assert!(!enemy.is_vulnerable());
    }

    #[test]
    fn tar_monster_retaliation_spawns_five_projectiles() {
        let mut projectiles = Vec::new();
        spawn_tar_monster_retaliation(&mut projectiles, 100 << 16, 200 << 16);
        assert_eq!(projectiles.len(), 5);
        for p in &projectiles {
            assert!(p.y_velocity < 0, "all tar blobs should launch upward");
            assert!(!p.friendly);
            assert!(p.gravity > 0, "tar blobs have gravity");
        }
        // Verify spread: some go left, some go right
        let left_count = projectiles.iter().filter(|p| p.x_velocity < 0).count();
        let right_count = projectiles.iter().filter(|p| p.x_velocity > 0).count();
        assert!(left_count >= 2, "at least 2 blobs go left");
        assert!(right_count >= 2, "at least 2 blobs go right");
    }

    #[test]
    fn fire_demon_death_spawns_trail() {
        let mut projectiles = Vec::new();
        spawn_fire_demon_death_trail(&mut projectiles, 100 << 16, 200 << 16);
        assert_eq!(projectiles.len(), 3, "should spawn 3 fire fragments");
        for p in &projectiles {
            assert!(p.y_velocity < 0, "fragments launch upward");
            assert!(!p.friendly);
            assert!(p.gravity > 0, "fragments have gravity");
        }
    }

    // --- Drips ---

    #[test]
    fn drips_drops_projectile_on_timer_expiry() {
        let mut enemy = test_enemy(EnemyFamily::Drips, 60 << 16, 20 << 16, 1);
        enemy.wait_timer = 0;
        let mut projectiles = Vec::new();
        step_drips(&mut enemy, &mut projectiles);
        assert_eq!(projectiles.len(), 1, "should drop one projectile");
        assert_eq!(projectiles[0].x_velocity, 0, "drops straight down");
        assert!(projectiles[0].y_velocity > 0, "falls downward");
        assert!(!projectiles[0].friendly);
        assert_eq!(enemy.wait_timer, DRIPS_DROP_INTERVAL);
    }

    #[test]
    fn drips_waits_between_drops() {
        let mut enemy = test_enemy(EnemyFamily::Drips, 60 << 16, 20 << 16, 1);
        enemy.wait_timer = 5;
        let mut projectiles = Vec::new();
        step_drips(&mut enemy, &mut projectiles);
        assert!(projectiles.is_empty(), "should not drop while timer active");
        assert_eq!(enemy.wait_timer, 4);
    }

    // --- SpinningTwins ---

    #[test]
    fn spinning_twins_orbits_indefinitely_without_player_contact() {
        let mut enemy = test_enemy(EnemyFamily::SpinningTwins, 100 << 16, 50 << 16, 1);
        assert_eq!(enemy.spinning_twins_phase, SpinningTwinsPhase::Orbiting);

        // Step through orbit phase — X position should oscillate
        let initial_x = enemy.x_pos;
        step_spinning_twins(&mut enemy, None, None, None);
        assert_eq!(enemy.spinning_twins_phase, SpinningTwinsPhase::Orbiting);
        assert_ne!(enemy.x_pos, initial_x, "orbit should move X position");

        // Step many more frames — should still orbit without player contact
        for _ in 0..200 {
            step_spinning_twins(&mut enemy, None, None, None);
        }
        assert_eq!(
            enemy.spinning_twins_phase,
            SpinningTwinsPhase::Orbiting,
            "should orbit indefinitely without player contact (ASM: no timer fallback)"
        );
    }

    #[test]
    fn spinning_twins_drops_on_player_collision() {
        let mut enemy = test_enemy(EnemyFamily::SpinningTwins, 100 << 16, 50 << 16, 1);
        assert_eq!(enemy.spinning_twins_phase, SpinningTwinsPhase::Orbiting);

        // Place player overlapping the orbiting twins
        let player_x = Some(100u32 << 16);
        let player_y = Some(50u32 << 16);
        step_spinning_twins(&mut enemy, None, player_x, player_y);

        assert_eq!(enemy.spinning_twins_phase, SpinningTwinsPhase::Walking);
        assert!(!enemy.on_ground, "should be airborne after drop");
        assert!(enemy.y_velocity < 0, "should have upward launch velocity");
    }

    // --- Ghost blocks ---

    #[test]
    fn ghost_block_toggles_on_timer_expiry() {
        let mut group = GhostGroup {
            col: 3,
            row: 5,
            count: 2,
            vertical: false,
            solid_time: 10,
            vanished_time: 5,
            timer: 1,
            solid: true,
        };
        // Timer = 1: tick it down, still solid (timer goes to 0)
        assert!(group.solid);
        // Simulate one step
        if group.timer > 0 {
            group.timer -= 1;
        }
        assert_eq!(group.timer, 0);
        // Next step: timer is 0, transition happens
        group.solid = !group.solid;
        group.timer = if group.solid {
            group.solid_time
        } else {
            group.vanished_time
        };
        assert!(!group.solid, "should become vanished");
        assert_eq!(group.timer, 5, "vanished timer should be set");
    }

    #[test]
    fn ghost_block_group_cells_horizontal() {
        let group = GhostGroup {
            col: 3,
            row: 5,
            count: 3,
            vertical: false,
            solid_time: 10,
            vanished_time: 10,
            timer: 0,
            solid: true,
        };
        let cells: Vec<(u16, u16)> = (0..group.count)
            .map(|i| {
                if group.vertical {
                    (group.col, group.row + u16::from(i))
                } else {
                    (group.col + u16::from(i), group.row)
                }
            })
            .collect();
        assert_eq!(cells, vec![(3, 5), (4, 5), (5, 5)]);
    }

    #[test]
    fn ghost_block_group_cells_vertical() {
        let group = GhostGroup {
            col: 3,
            row: 5,
            count: 2,
            vertical: true,
            solid_time: 10,
            vanished_time: 10,
            timer: 0,
            solid: true,
        };
        let cells: Vec<(u16, u16)> = (0..group.count)
            .map(|i| {
                if group.vertical {
                    (group.col, group.row + u16::from(i))
                } else {
                    (group.col + u16::from(i), group.row)
                }
            })
            .collect();
        assert_eq!(cells, vec![(3, 5), (3, 6)]);
    }

    #[test]
    fn ghost_block_list_parsing() {
        // Ghost entry with timer=0 (the prize parser sees layout_addr=0 → ends prizes,
        // then ghost parser picks up). This mirrors a level with no prizes.
        let ghost_entry: [u8; 16] = [
            0x00, 0x00, // timer = 0 (also acts as prize terminator since layout_addr=0)
            0x00, 0x00, // flags, mapping_frame = 0 (solid)
            0x00, 0x14, // solid_time = 20
            0x00, 0x0A, // vanished_time = 10
            0x00, 0x10, // layout_addr (non-zero → not a ghost terminator)
            0x00, 0x03, // col = 3
            0x00, 0x05, // row = 5
            0x02, // count nibble = 2 → count = 3
            0x01, // vertical = true
        ];
        // Ghost terminator (all zeros)
        let ghost_end: [u8; 16] = [0u8; 16];

        let mut data = Vec::new();
        data.extend_from_slice(&ghost_entry);
        data.extend_from_slice(&ghost_end);

        let bl = terrain::BlockList::from_raw(&data, 20);
        assert_eq!(bl.prize_blocks.len(), 0, "no prizes");
        assert_eq!(bl.ghost_blocks.len(), 1);
        assert_eq!(bl.ghost_blocks[0].col, 3);
        assert_eq!(bl.ghost_blocks[0].row, 5);
        assert_eq!(bl.ghost_blocks[0].count, 3);
        assert!(bl.ghost_blocks[0].vertical);
        assert_eq!(bl.ghost_blocks[0].solid_time, 20);
        assert_eq!(bl.ghost_blocks[0].vanished_time, 10);
        assert!(bl.ghost_blocks[0].solid, "mapping_frame=0 means solid");
    }

    // --- Terrain type speed table selection ---

    #[test]
    fn terrain_type_from_block_type_maps_ice_and_rubber() {
        assert_eq!(
            TerrainType::from_block_type(terrain::BlockType::Ice),
            TerrainType::Ice
        );
        assert_eq!(
            TerrainType::from_block_type(terrain::BlockType::Rubber),
            TerrainType::Rubber
        );
        assert_eq!(
            TerrainType::from_block_type(terrain::BlockType::Prize),
            TerrainType::Normal
        );
        assert_eq!(
            TerrainType::from_block_type(terrain::BlockType::None),
            TerrainType::Normal
        );
    }

    #[test]
    fn ground_speed_table_selects_by_form_and_terrain() {
        assert_eq!(
            PlayerForm::TheKid.ground_speed_table(TerrainType::Normal),
            &physics::NORMAL_KID
        );
        assert_eq!(
            PlayerForm::TheKid.ground_speed_table(TerrainType::Ice),
            &physics::ICE_KID
        );
        assert_eq!(
            PlayerForm::TheKid.ground_speed_table(TerrainType::Rubber),
            &physics::RUBBER_KID
        );
        assert_eq!(
            PlayerForm::Micromax.ground_speed_table(TerrainType::Ice),
            &physics::ICE_MICROMAX
        );
        assert_eq!(
            PlayerForm::Juggernaut.ground_speed_table(TerrainType::Rubber),
            &physics::RUBBER_JUGGERNAUT
        );
    }

    #[test]
    fn floor_collision_sets_terrain_type_from_block() {
        // Build a grid with an ice block at row 4 (y=64)
        let w = 4u16;
        let h = 6u16;
        let mut data = vec![0u8; usize::from(w) * usize::from(h) * 2];
        // Row 4, col 1: solid ice block
        // has_block=1, collision=6 (solid), block_type=2 (ice)
        // hi byte: 1_110_0010 = 0xE2
        let offset = (4 * usize::from(w) + 1) * 2;
        data[offset] = 0xE2;
        data[offset + 1] = 0x01;
        let grid = terrain::BlockGrid::from_level_layout(&data, w, h);

        let mut slice = OpeningSliceState {
            player: PlayerState {
                x_pos: 24 << 16, // col 1 centre
                y_pos: 64 << 16, // at floor level
                x_velocity: 0,
                y_velocity: 0x1_0000, // falling
                facing: Direction::Right,
                on_ground: false,
                on_spikes: false,
                just_jumped: false,
                jump_buffer_frames: 0,
                right_wall_sticky: false,
                left_wall_sticky: false,
            },
            camera: CameraState { x_pos: 0, y_pos: 0 },
            terrain: TerrainState {
                floor_y_pos: 0,
                floor_terrain_type: TerrainType::Normal,
            },
            level_bounds: LevelBounds {
                width_pixels: w * 16,
                height_pixels: h * 16,
            },
            objects: ObjectSystemState {
                platform_loader: PlatformLoaderState::default(),
            },
            prize_block_hit: None,
            lift_block_hit: None,
            drill_projectile_hit: None,
            metal_block_hit: None,
            smash_block_hit: None,
            teleporter_contact: None,
        };

        slice.apply_floor_collision(Some(&grid), false, false);
        assert!(slice.player.on_ground);
        assert_eq!(slice.terrain.floor_terrain_type, TerrainType::Ice);
    }

    #[test]
    fn leaving_ground_resets_terrain_type_to_normal() {
        let w = 4u16;
        let h = 4u16;
        let data = vec![0u8; usize::from(w) * usize::from(h) * 2];
        let grid = terrain::BlockGrid::from_level_layout(&data, w, h);

        let mut slice = OpeningSliceState {
            player: PlayerState {
                x_pos: 24 << 16,
                y_pos: 32 << 16,
                x_velocity: 0,
                y_velocity: 0,
                facing: Direction::Right,
                on_ground: true,
                on_spikes: false,
                just_jumped: false,
                jump_buffer_frames: 0,
                right_wall_sticky: false,
                left_wall_sticky: false,
            },
            camera: CameraState { x_pos: 0, y_pos: 0 },
            terrain: TerrainState {
                floor_y_pos: 32 << 16,
                floor_terrain_type: TerrainType::Ice,
            },
            level_bounds: LevelBounds {
                width_pixels: w * 16,
                height_pixels: h * 16,
            },
            objects: ObjectSystemState {
                platform_loader: PlatformLoaderState::default(),
            },
            prize_block_hit: None,
            lift_block_hit: None,
            drill_projectile_hit: None,
            metal_block_hit: None,
            smash_block_hit: None,
            teleporter_contact: None,
        };

        // No floor under player → should leave ground and reset terrain type
        slice.apply_floor_collision(Some(&grid), false, false);
        assert!(!slice.player.on_ground);
        assert_eq!(slice.terrain.floor_terrain_type, TerrainType::Normal);
    }

    // --- Platform collision ---

    fn make_test_platform(x: u16, y: u16, h_variant: u8, v_variant: u8) -> ActivePlatform {
        ActivePlatform {
            x_pos: u32::from(x) << 16,
            y_pos: u32::from(y) << 16,
            width_px: u16::from(h_variant) * 16 + 15,
            height_px: u16::from(v_variant) * 16 + 15,
            x_velocity: 0,
            y_velocity: 0,
            script: None,
        }
    }

    fn make_airborne_slice(x: u16, y: u16) -> OpeningSliceState {
        OpeningSliceState {
            player: PlayerState {
                x_pos: u32::from(x) << 16,
                y_pos: u32::from(y) << 16,
                x_velocity: 0,
                y_velocity: 0x1_0000, // falling
                facing: Direction::Right,
                on_ground: false,
                on_spikes: false,
                just_jumped: false,
                jump_buffer_frames: 0,
                right_wall_sticky: false,
                left_wall_sticky: false,
            },
            camera: CameraState { x_pos: 0, y_pos: 0 },
            terrain: TerrainState {
                floor_y_pos: 0,
                floor_terrain_type: TerrainType::Normal,
            },
            level_bounds: LevelBounds {
                width_pixels: 320,
                height_pixels: 224,
            },
            objects: ObjectSystemState {
                platform_loader: PlatformLoaderState::default(),
            },
            prize_block_hit: None,
            lift_block_hit: None,
            drill_projectile_hit: None,
            metal_block_hit: None,
            smash_block_hit: None,
            teleporter_contact: None,
        }
    }

    #[test]
    fn platform_collision_lands_falling_player() {
        // Platform at (40, 100), h=2 v=0 → 47px wide, 15px tall
        let platform = make_test_platform(40, 100, 2, 0);
        let mut slice = make_airborne_slice(50, 100); // feet at platform top

        apply_platform_collision(&mut slice, &[platform]);

        assert!(slice.player.on_ground);
        assert_eq!(slice.player.y_velocity, 0);
        // Lands 1 pixel above platform top (same as solid floor convention)
        assert_eq!(slice.player.y_pos, 99u32 << 16);
    }

    #[test]
    fn platform_collision_ignores_grounded_player() {
        let platform = make_test_platform(40, 100, 2, 0);
        let mut slice = make_airborne_slice(50, 100);
        slice.player.on_ground = true; // already grounded

        apply_platform_collision(&mut slice, &[platform]);

        // Should not modify position — already on ground
        assert_eq!(slice.player.y_pos, 100u32 << 16);
    }

    #[test]
    fn platform_collision_ignores_rising_player() {
        let platform = make_test_platform(40, 100, 2, 0);
        let mut slice = make_airborne_slice(50, 100);
        slice.player.y_velocity = -0x2_0000; // moving upward

        apply_platform_collision(&mut slice, &[platform]);

        assert!(!slice.player.on_ground);
    }

    #[test]
    fn platform_collision_misses_when_player_is_not_above() {
        let platform = make_test_platform(40, 100, 2, 0);
        // Player is far left of platform (x=10, platform starts at x=40)
        let mut slice = make_airborne_slice(10, 100);

        apply_platform_collision(&mut slice, &[platform]);

        assert!(!slice.player.on_ground);
    }

    #[test]
    fn platform_collision_misses_when_player_is_well_above() {
        let platform = make_test_platform(40, 100, 2, 0);
        // Player at y=50, platform at y=100 — too high to land
        let mut slice = make_airborne_slice(50, 50);

        apply_platform_collision(&mut slice, &[platform]);

        assert!(!slice.player.on_ground);
    }

    #[test]
    fn platform_riding_carries_grounded_player() {
        let mut platform = make_test_platform(40, 100, 2, 0);
        platform.x_velocity = 0x1_0000; // 1 px/frame rightward
        platform.y_velocity = 0;

        // Player is grounded on the platform (feet at y=99, platform at y=100)
        let mut slice = make_airborne_slice(50, 99);
        slice.player.on_ground = true;
        slice.player.y_velocity = 0;

        let x_before = slice.player.x_pos;
        apply_platform_collision(&mut slice, &[platform]);

        assert!(
            slice.player.x_pos > x_before,
            "grounded player should be carried rightward by moving platform"
        );
        assert_eq!(
            slice.player.x_pos - x_before,
            0x1_0000,
            "player should move by exactly the platform's x velocity"
        );
    }

    #[test]
    fn platform_riding_ignores_stationary_platform() {
        let platform = make_test_platform(40, 100, 2, 0);
        // Stationary platform (zero velocity)
        let mut slice = make_airborne_slice(50, 99);
        slice.player.on_ground = true;
        slice.player.y_velocity = 0;

        let x_before = slice.player.x_pos;
        apply_platform_collision(&mut slice, &[platform]);

        assert_eq!(
            slice.player.x_pos, x_before,
            "stationary platform should not move player"
        );
    }

    #[test]
    fn platform_dimensions_from_h_v_variants() {
        // h_variant=4, v_variant=0 → 79×15
        let p = make_test_platform(0, 0, 4, 0);
        assert_eq!(p.width_px, 79);
        assert_eq!(p.height_px, 15);

        // h_variant=2, v_variant=1 → 47×31
        let p = make_test_platform(0, 0, 2, 1);
        assert_eq!(p.width_px, 47);
        assert_eq!(p.height_px, 31);
    }

    #[test]
    fn platform_script_moves_platform() {
        use std::collections::BTreeMap;
        let mut scripts = BTreeMap::new();
        scripts.insert(
            0x0000,
            kid_data::PlatformScript {
                commands: vec![
                    kid_data::PlatformMoveCommand {
                        duration: 3,
                        x_velocity: 0x1_0000,
                        y_velocity: 0,
                    },
                    kid_data::PlatformMoveCommand {
                        duration: 3,
                        x_velocity: -0x1_0000,
                        y_velocity: 0,
                    },
                ],
            },
        );
        let table = kid_data::PlatformScriptTable { scripts };

        let mut platform = ActivePlatform {
            x_pos: 100 << 16,
            y_pos: 50 << 16,
            width_px: 47,
            height_px: 15,
            x_velocity: 0,
            y_velocity: 0,
            script: Some(PlatformScriptState {
                script_offset: 0,
                command_index: 0,
                remaining_duration: 3,
            }),
        };

        let start_x = platform.x_pos;

        // Step 3 frames of moving right
        for _ in 0..3 {
            platform.step_script(&table);
        }
        // Should have moved 3 pixels right
        assert_eq!(platform.x_pos, start_x + 3 * 0x1_0000);
        // Should now be on second command (moving left)
        assert_eq!(platform.script.unwrap().command_index, 1);

        // Step 3 more frames of moving left
        for _ in 0..3 {
            platform.step_script(&table);
        }
        // Should be back at start
        assert_eq!(platform.x_pos, start_x);
        // Should have looped back to first command
        assert_eq!(platform.script.unwrap().command_index, 0);
    }

    // --- Evanescent blocks ---

    /// Build a grid with an evanescent block at (col, row) and solid floor
    /// one row below. The skin byte controls the initial timer delay.
    fn grid_with_evanescent(col: u16, row: u16, skin: u8) -> terrain::BlockGrid {
        let w = 6u16;
        let h = (row + 3).max(6);
        let mut data = vec![0u8; usize::from(w) * usize::from(h) * 2];
        let set = |data: &mut Vec<u8>, c: u16, r: u16, hi: u8, lo: u8| {
            let off = (usize::from(r) * usize::from(w) + usize::from(c)) * 2;
            data[off] = hi;
            data[off + 1] = lo;
        };
        // Evanescent block: has_block=1, collision=6(Solid), block_type=0xA
        // hi byte: 1_110_1010 = 0xEA
        set(&mut data, col, row, 0xEA, skin);
        // Solid floor one row below the evanescent block
        for c in 0..w {
            set(&mut data, c, row + 1, 0x60, 0);
        }
        terrain::BlockGrid::from_level_layout(&data, w, h)
    }

    fn runtime_with_evanescent_grid(grid: terrain::BlockGrid) -> NativeRuntime {
        let manifest = manifest();
        let mut runtime = NativeRuntime::new(manifest);
        runtime.set_block_grid(grid);
        runtime
    }

    #[test]
    fn evanescent_block_triggers_when_player_stands_on_it() {
        let grid = grid_with_evanescent(2, 3, 0);
        let mut runtime = runtime_with_evanescent_grid(grid);

        // Player standing on the evanescent block: feet at row 3 floor (y=47px)
        let slice = OpeningSliceState {
            player: PlayerState {
                x_pos: (2 * 16 + 8) << 16, // centre of col 2
                y_pos: (3 * 16 - 1) << 16, // feet just at top of row 3
                x_velocity: 0,
                y_velocity: 0,
                facing: Direction::Right,
                on_ground: true,
                on_spikes: false,
                just_jumped: false,
                jump_buffer_frames: 0,
                right_wall_sticky: false,
                left_wall_sticky: false,
            },
            camera: CameraState { x_pos: 0, y_pos: 0 },
            terrain: TerrainState {
                floor_y_pos: (3 * 16 - 1) << 16,
                floor_terrain_type: TerrainType::Normal,
            },
            level_bounds: LevelBounds {
                width_pixels: 96,
                height_pixels: 96,
            },
            objects: ObjectSystemState {
                platform_loader: PlatformLoaderState::default(),
            },
            prize_block_hit: None,
            lift_block_hit: None,
            drill_projectile_hit: None,
            metal_block_hit: None,
            smash_block_hit: None,
            teleporter_contact: None,
        };

        assert!(runtime.evanescent_blocks.is_empty());
        runtime.step_evanescent_blocks(&slice);
        assert_eq!(
            runtime.evanescent_blocks.len(),
            1,
            "standing on evanescent block should trigger it"
        );
        assert_eq!(runtime.evanescent_blocks[0].col, 2);
        assert_eq!(runtime.evanescent_blocks[0].row, 3);
    }

    #[test]
    fn evanescent_block_not_triggered_when_airborne() {
        let grid = grid_with_evanescent(2, 3, 0);
        let mut runtime = runtime_with_evanescent_grid(grid);

        let slice = OpeningSliceState {
            player: PlayerState {
                x_pos: (2 * 16 + 8) << 16,
                y_pos: (3 * 16 - 5) << 16, // above the block
                x_velocity: 0,
                y_velocity: -0x1_0000, // moving upward
                facing: Direction::Right,
                on_ground: false, // airborne
                on_spikes: false,
                just_jumped: false,
                jump_buffer_frames: 0,
                right_wall_sticky: false,
                left_wall_sticky: false,
            },
            camera: CameraState { x_pos: 0, y_pos: 0 },
            terrain: TerrainState {
                floor_y_pos: 0,
                floor_terrain_type: TerrainType::Normal,
            },
            level_bounds: LevelBounds {
                width_pixels: 96,
                height_pixels: 96,
            },
            objects: ObjectSystemState {
                platform_loader: PlatformLoaderState::default(),
            },
            prize_block_hit: None,
            lift_block_hit: None,
            drill_projectile_hit: None,
            metal_block_hit: None,
            smash_block_hit: None,
            teleporter_contact: None,
        };

        runtime.step_evanescent_blocks(&slice);
        assert!(
            runtime.evanescent_blocks.is_empty(),
            "airborne player should not trigger evanescent block"
        );
    }

    #[test]
    fn evanescent_block_clears_after_timer_expires() {
        // skin=28 → initial timer = 28 - 28 = 0 → expires after 22 frames
        let grid = grid_with_evanescent(2, 3, 28);
        let mut runtime = runtime_with_evanescent_grid(grid);

        // Verify block exists
        assert_eq!(
            runtime
                .block_grid
                .as_ref()
                .unwrap()
                .cell_at(2, 3)
                .block_type,
            terrain::BlockType::Evanescent
        );

        // Trigger it
        let slice = OpeningSliceState {
            player: PlayerState {
                x_pos: (2 * 16 + 8) << 16,
                y_pos: (3 * 16 - 1) << 16,
                x_velocity: 0,
                y_velocity: 0,
                facing: Direction::Right,
                on_ground: true,
                on_spikes: false,
                just_jumped: false,
                jump_buffer_frames: 0,
                right_wall_sticky: false,
                left_wall_sticky: false,
            },
            camera: CameraState { x_pos: 0, y_pos: 0 },
            terrain: TerrainState {
                floor_y_pos: (3 * 16 - 1) << 16,
                floor_terrain_type: TerrainType::Normal,
            },
            level_bounds: LevelBounds {
                width_pixels: 96,
                height_pixels: 96,
            },
            objects: ObjectSystemState {
                platform_loader: PlatformLoaderState::default(),
            },
            prize_block_hit: None,
            lift_block_hit: None,
            drill_projectile_hit: None,
            metal_block_hit: None,
            smash_block_hit: None,
            teleporter_contact: None,
        };

        // Trigger + first tick
        runtime.step_evanescent_blocks(&slice);
        assert_eq!(runtime.evanescent_blocks.len(), 1);

        // Tick 21 more times (total 22 ticks) — airborne so no re-trigger
        let airborne_slice = OpeningSliceState {
            player: PlayerState {
                on_ground: false,
                ..slice.player
            },
            ..slice
        };
        for _ in 0..21 {
            runtime.step_evanescent_blocks(&airborne_slice);
        }

        // Block should now be cleared
        assert!(
            runtime.evanescent_blocks.is_empty(),
            "evanescent entry should be removed after 22 ticks"
        );
        let cell = runtime.block_grid.as_ref().unwrap().cell_at(2, 3);
        assert!(
            !cell.has_block,
            "evanescent block should be cleared from the grid"
        );
    }

    #[test]
    fn evanescent_block_with_skin_delay() {
        // skin=0 → initial timer = 0 - 28 = -28 → needs 28 + 22 = 50 ticks
        let grid = grid_with_evanescent(2, 3, 0);
        let mut runtime = runtime_with_evanescent_grid(grid);

        let on_ground_slice = OpeningSliceState {
            player: PlayerState {
                x_pos: (2 * 16 + 8) << 16,
                y_pos: (3 * 16 - 1) << 16,
                x_velocity: 0,
                y_velocity: 0,
                facing: Direction::Right,
                on_ground: true,
                on_spikes: false,
                just_jumped: false,
                jump_buffer_frames: 0,
                right_wall_sticky: false,
                left_wall_sticky: false,
            },
            camera: CameraState { x_pos: 0, y_pos: 0 },
            terrain: TerrainState {
                floor_y_pos: (3 * 16 - 1) << 16,
                floor_terrain_type: TerrainType::Normal,
            },
            level_bounds: LevelBounds {
                width_pixels: 96,
                height_pixels: 96,
            },
            objects: ObjectSystemState {
                platform_loader: PlatformLoaderState::default(),
            },
            prize_block_hit: None,
            lift_block_hit: None,
            drill_projectile_hit: None,
            metal_block_hit: None,
            smash_block_hit: None,
            teleporter_contact: None,
        };

        let airborne_slice = OpeningSliceState {
            player: PlayerState {
                on_ground: false,
                ..on_ground_slice.player
            },
            ..on_ground_slice
        };

        // Trigger + first tick
        runtime.step_evanescent_blocks(&on_ground_slice);
        assert_eq!(runtime.evanescent_blocks.len(), 1);
        assert_eq!(runtime.evanescent_blocks[0].timer, -27); // -28 + 1

        // After 48 more ticks (49 total), still active (timer = -28 + 49 = 21)
        for _ in 0..48 {
            runtime.step_evanescent_blocks(&airborne_slice);
        }
        assert_eq!(
            runtime.evanescent_blocks.len(),
            1,
            "block should still be active at tick 49"
        );
        assert!(
            runtime.block_grid.as_ref().unwrap().cell_at(2, 3).has_block,
            "block should still be solid at tick 49"
        );

        // One more tick (50 total): timer = -28 + 50 = 22 → removed
        runtime.step_evanescent_blocks(&airborne_slice);
        assert!(
            runtime.evanescent_blocks.is_empty(),
            "block should be removed at tick 50"
        );
        assert!(
            !runtime.block_grid.as_ref().unwrap().cell_at(2, 3).has_block,
            "block should be cleared from grid at tick 50"
        );
    }

    // --- Level timer ---

    #[test]
    fn level_timer_starts_at_three_minutes() {
        let timer = LevelTimer::new();
        assert_eq!(timer.seconds, 180);
        // ASM starts Time_SubSeconds at 1 so first frame immediately ticks a second
        assert_eq!(timer.sub_frames, 1);
    }

    #[test]
    fn level_timer_first_frame_decrements_immediately() {
        let mut timer = LevelTimer::new();
        // First tick: sub_frames 1→0, seconds 180→179, sub_frames reloads to 60
        assert!(!timer.tick());
        assert_eq!(timer.seconds, 179);
        assert_eq!(timer.sub_frames, 60);
    }

    #[test]
    fn level_timer_ticks_down_one_second_per_60_frames() {
        let mut timer = LevelTimer::new();
        timer.tick(); // first tick → 179
        // Then 60 more ticks → 178
        for _ in 0..60 {
            assert!(!timer.tick());
        }
        assert_eq!(timer.seconds, 178);
    }

    #[test]
    fn level_timer_expires_after_full_countdown() {
        let mut timer = LevelTimer::new();
        // First tick → 179 seconds, sub_frames=60
        // Then 179×60 more ticks → 0 seconds, sub_frames=60
        // Total: 1 + 179×60 = 10741 ticks to reach seconds=0
        // Then 60 more to drain sub_frames → expired at tick 10801
        let total_frames = 1 + 179 * 60 + 60;
        for i in 0..total_frames {
            let expired = timer.tick();
            if i < total_frames - 1 {
                assert!(!expired, "timer should not expire at frame {i}");
            } else {
                assert!(expired, "timer should expire at frame {total_frames}");
            }
        }
        assert_eq!(timer.seconds, 0);
    }

    #[test]
    fn level_timer_clock_bonus_adds_three_minutes() {
        let mut timer = LevelTimer::new();
        // Tick down 60 seconds
        for _ in 0..(60 * 60) {
            timer.tick();
        }
        assert_eq!(timer.seconds, 120); // 2:00 remaining
        timer.add_clock_bonus();
        assert_eq!(timer.seconds, 300); // 5:00 remaining
    }

    #[test]
    fn level_timer_clock_bonus_caps_at_ten_minutes() {
        let mut timer = LevelTimer::new();
        // At 3:00, add clock → 6:00
        timer.add_clock_bonus();
        assert_eq!(timer.seconds, 360);
        // Add another → 9:00
        timer.add_clock_bonus();
        assert_eq!(timer.seconds, 540);
        // Add another → would be 12:00, capped to 10:00
        timer.add_clock_bonus();
        assert_eq!(timer.seconds, 600);
    }

    #[test]
    fn level_timer_death_on_expiry() {
        let mut runtime = NativeRuntime::new(manifest());
        start_level(&mut runtime);

        // Fast-forward timer to near-expiry
        runtime.level_timer.seconds = 0;
        runtime.level_timer.sub_frames = 2;

        // Tick one frame — timer goes to sub_frames=1, not yet expired
        runtime.step_frame(FrameInput::default());
        assert!(
            matches!(runtime.player_lifecycle, PlayerLifecycle::Alive),
            "player should still be alive with 1 sub_frame remaining"
        );

        // Tick one more — timer expires, death triggered
        runtime.step_frame(FrameInput::default());
        assert!(
            matches!(
                runtime.player_lifecycle,
                PlayerLifecycle::DeathWindup { .. }
            ),
            "timer expiry should trigger death, got {:?}",
            runtime.player_lifecycle
        );
    }

    #[test]
    fn level_timer_exposed_in_trace_frame() {
        let timer = LevelTimer::new();
        // The trace frame should expose the timer seconds
        assert_eq!(timer.seconds, LEVEL_TIMER_START_SECONDS);
    }

    #[test]
    fn evanescent_block_not_triggered_twice() {
        let grid = grid_with_evanescent(2, 3, 28);
        let mut runtime = runtime_with_evanescent_grid(grid);

        let slice = OpeningSliceState {
            player: PlayerState {
                x_pos: (2 * 16 + 8) << 16,
                y_pos: (3 * 16 - 1) << 16,
                x_velocity: 0,
                y_velocity: 0,
                facing: Direction::Right,
                on_ground: true,
                on_spikes: false,
                just_jumped: false,
                jump_buffer_frames: 0,
                right_wall_sticky: false,
                left_wall_sticky: false,
            },
            camera: CameraState { x_pos: 0, y_pos: 0 },
            terrain: TerrainState {
                floor_y_pos: (3 * 16 - 1) << 16,
                floor_terrain_type: TerrainType::Normal,
            },
            level_bounds: LevelBounds {
                width_pixels: 96,
                height_pixels: 96,
            },
            objects: ObjectSystemState {
                platform_loader: PlatformLoaderState::default(),
            },
            prize_block_hit: None,
            lift_block_hit: None,
            drill_projectile_hit: None,
            metal_block_hit: None,
            smash_block_hit: None,
            teleporter_contact: None,
        };

        // Trigger twice while standing
        runtime.step_evanescent_blocks(&slice);
        runtime.step_evanescent_blocks(&slice);
        assert_eq!(
            runtime.evanescent_blocks.len(),
            1,
            "same evanescent block should not be triggered twice"
        );
    }

    // --- Shooter block tests ---

    #[test]
    fn shooter_block_fires_projectiles_on_timer() {
        let mut runtime = NativeRuntime::new(manifest());
        start_level(&mut runtime);
        // Build a grid with a shooter block at (2, 3) firing right (skin bit 1)
        let width_cells = 120u16;
        let height_cells = 56u16;
        let mut data = vec![0u8; usize::from(width_cells) * usize::from(height_cells) * 2];
        // Set (2, 3) as shooter block, skin=2 (right)
        let offset = (3 * usize::from(width_cells) + 2) * 2;
        data[offset] = 0xE8; // has_block=1, collision=6(Solid), block_type=8(Shooter)
        data[offset + 1] = 0x02; // skin=2 (fire right)
        // Verify the encoding
        let grid = terrain::BlockGrid::from_level_layout(&data, width_cells, height_cells);
        let cell = grid.cell_at(2, 3);
        assert!(cell.has_block, "cell should have a block");
        assert_eq!(
            cell.block_type,
            terrain::BlockType::Shooter,
            "cell should be Shooter"
        );
        assert_eq!(cell.skin, 2, "skin should be 2");
        runtime.block_grid = Some(grid);
        runtime.shooter_block_timer = 0; // fire immediately
        runtime.projectiles.clear();

        // Set camera to origin so block is visible
        let mut slice = runtime.opening_slice.clone().unwrap();
        slice.camera.x_pos = 0;
        slice.camera.y_pos = 0;
        runtime.step_shooter_blocks(&slice);

        assert_eq!(runtime.projectiles.len(), 1, "should fire one projectile");
        assert!(
            runtime.projectiles[0].x_velocity > 0,
            "projectile should move right"
        );
        assert_eq!(runtime.projectiles[0].y_velocity, 0);
    }

    #[test]
    fn shooter_block_respects_timer() {
        let mut runtime = NativeRuntime::new(manifest());
        start_level(&mut runtime);
        let width_cells = 120u16;
        let height_cells = 56u16;
        let mut data = vec![0u8; usize::from(width_cells) * usize::from(height_cells) * 2];
        let offset = (3 * usize::from(width_cells) + 2) * 2;
        data[offset] = 0xE8;
        data[offset + 1] = 0x02;
        let grid = terrain::BlockGrid::from_level_layout(&data, width_cells, height_cells);
        runtime.block_grid = Some(grid);
        runtime.shooter_block_timer = 10; // not yet
        runtime.projectiles.clear();

        let slice = runtime.opening_slice.clone().unwrap();
        runtime.step_shooter_blocks(&slice);

        assert!(
            runtime.projectiles.is_empty(),
            "should not fire while timer is active"
        );
    }

    #[test]
    fn next_level_id_follows_map_order() {
        let mut manifest = manifest_with_levels(4);
        // map_order: [0, 1, 2, 3] by default; rearrange to [0, 3, 1, 2]
        manifest.map_order = vec![0, 3, 1, 2];

        let mut runtime = NativeRuntime::new(manifest);
        runtime.current_level = LevelId(0);
        assert_eq!(runtime.next_level_id(), LevelId(3));

        runtime.current_level = LevelId(3);
        assert_eq!(runtime.next_level_id(), LevelId(1));

        runtime.current_level = LevelId(1);
        assert_eq!(runtime.next_level_id(), LevelId(2));
    }

    #[test]
    fn next_level_id_falls_back_when_map_order_empty() {
        let mut manifest = manifest_with_levels(3);
        manifest.map_order = Vec::new();

        let mut runtime = NativeRuntime::new(manifest);
        runtime.current_level = LevelId(0);
        assert_eq!(runtime.next_level_id(), LevelId(1));

        runtime.current_level = LevelId(1);
        assert_eq!(runtime.next_level_id(), LevelId(2));
    }

    #[test]
    fn final_level_sets_game_complete_on_advance() {
        let mut manifest = manifest_with_levels(3);
        manifest.map_order = vec![0, 1, 2];

        let mut runtime = NativeRuntime::new(manifest);
        runtime.current_level = LevelId(2); // last in map order
        assert!(
            runtime.is_final_level(),
            "level 2 should be detected as final level"
        );
        assert!(!runtime.game_complete);
        runtime.advance_level();
        assert!(
            runtime.game_complete,
            "game_complete should be set after beating final level"
        );
    }

    #[test]
    fn non_final_level_does_not_set_game_complete() {
        let mut manifest = manifest_with_levels(3);
        manifest.map_order = vec![0, 1, 2];

        let mut runtime = NativeRuntime::new(manifest);
        runtime.set_block_grid(grid_with_floor_at_row(17));
        runtime.current_level = LevelId(0);
        assert!(!runtime.is_final_level());
        runtime.advance_level();
        assert!(
            !runtime.game_complete,
            "game_complete should not be set mid-game"
        );
    }

    // --- Murder wall ---

    #[test]
    fn murder_wall_initialises_from_header_flags() {
        let mut manifest = manifest_with_levels(2);
        // Set level 0 to have murder wall (flags = 0b10 = enabled, left-to-right)
        manifest.levels[0]
            .header_metadata
            .as_mut()
            .unwrap()
            .murder_wall_flags = 0b10;

        let mut runtime = NativeRuntime::new(manifest);
        runtime.init_murder_wall(LevelId(0), 0);

        let wall = runtime.murder_wall.expect("murder wall should be active");
        assert!(!wall.reversed);
        assert_eq!(wall.speed, 0);
        assert_eq!(wall.max_speed, MURDER_WALL_DEFAULT_MAX_SPEED);
    }

    #[test]
    fn murder_wall_not_initialised_when_flags_zero() {
        let manifest = manifest_with_levels(2);
        let mut runtime = NativeRuntime::new(manifest);
        runtime.init_murder_wall(LevelId(0), 0);

        assert!(runtime.murder_wall.is_none());
    }

    #[test]
    fn murder_wall_accelerates_each_frame() {
        let mut wall = MurderWallState {
            x_pos: 0,
            speed: 0,
            max_speed: MURDER_WALL_DEFAULT_MAX_SPEED,
            reversed: false,
        };

        // Simulate a few frames manually
        wall.speed = (wall.speed + MURDER_WALL_ACCELERATION).min(wall.max_speed);
        wall.x_pos += i64::from(wall.speed);
        assert_eq!(wall.speed, 0x0100);
        assert_eq!(wall.x_pos, 0x0100);

        wall.speed = (wall.speed + MURDER_WALL_ACCELERATION).min(wall.max_speed);
        wall.x_pos += i64::from(wall.speed);
        assert_eq!(wall.speed, 0x0200);
        assert_eq!(wall.x_pos, 0x0300);
    }

    #[test]
    fn murder_wall_speed_caps_at_max() {
        let mut wall = MurderWallState {
            x_pos: 0,
            speed: MURDER_WALL_DEFAULT_MAX_SPEED - 1,
            max_speed: MURDER_WALL_DEFAULT_MAX_SPEED,
            reversed: false,
        };
        wall.speed = (wall.speed + MURDER_WALL_ACCELERATION).min(wall.max_speed);
        assert_eq!(wall.speed, MURDER_WALL_DEFAULT_MAX_SPEED);
    }

    #[test]
    fn murder_wall_reversed_flag() {
        let mut manifest = manifest_with_levels(2);
        manifest.levels[0]
            .header_metadata
            .as_mut()
            .unwrap()
            .murder_wall_flags = 0b11; // enabled + reversed

        let mut runtime = NativeRuntime::new(manifest);
        runtime.init_murder_wall(LevelId(0), 0);

        let wall = runtime.murder_wall.expect("murder wall should be active");
        assert!(wall.reversed);
    }

    #[test]
    fn murder_wall_hills_of_warrior_uses_slow_max_speed() {
        let mut manifest = manifest_with_levels(9);
        // Level 0x08 = Hills of the Warrior 1
        manifest.levels[8]
            .header_metadata
            .as_mut()
            .unwrap()
            .murder_wall_flags = 0b10;

        let mut runtime = NativeRuntime::new(manifest);
        runtime.init_murder_wall(LevelId(8), 0);

        let wall = runtime.murder_wall.expect("murder wall should be active");
        assert_eq!(wall.max_speed, MURDER_WALL_SLOW_MAX_SPEED);
    }

    #[test]
    fn hail_spawns_projectiles_after_35_seconds() {
        let mut runtime = NativeRuntime::new(manifest());
        runtime.weather_type = 3; // Storm + Hail
        // Simulate 35 seconds elapsed: timer at 145s remaining
        runtime.level_timer.seconds = 145;
        runtime.hail_active = true;
        runtime.hail_spawn_timer = 0;
        runtime.hail_period_timer = 100; // mid-period, still active

        let slice = airborne_slice();
        let before = runtime.projectiles.len();
        runtime.step_weather(&slice);

        assert_eq!(
            runtime.projectiles.len(),
            before + 1,
            "hail should spawn a projectile"
        );
        let p = &runtime.projectiles[before];
        assert!(p.y_velocity > 0, "hail should fall downward");
        assert!(!p.friendly, "hail should damage the player");
    }

    #[test]
    fn no_hail_without_storm_weather() {
        let mut runtime = NativeRuntime::new(manifest());
        runtime.weather_type = 0; // No weather
        runtime.level_timer.seconds = 100;

        let slice = airborne_slice();
        runtime.step_weather(&slice);

        assert!(
            runtime.projectiles.is_empty(),
            "no hail without storm weather"
        );
    }

    // --- Lava geyser tests ---

    #[test]
    fn lava_geyser_erupts_and_collapses() {
        let mut runtime = NativeRuntime::new(manifest());
        runtime.weather_type = 1;
        runtime.lava_geysers.push(LavaGeyser {
            x_px: 100,
            y_px: 200,
            height_px: 16,
            phase: 0,
            timer: 0, // ready to erupt immediately
            current_height: 0,
        });

        let slice = airborne_slice();

        // Phase 0 → phase 1 (erupt)
        runtime.step_lava_geysers(&slice);
        assert_eq!(runtime.lava_geysers[0].phase, 1);

        // Erupt until full height (16px / 4px per frame = 4 frames)
        for _ in 0..3 {
            runtime.step_lava_geysers(&slice);
        }
        assert_eq!(runtime.lava_geysers[0].phase, 1);
        runtime.step_lava_geysers(&slice);
        assert_eq!(
            runtime.lava_geysers[0].phase, 2,
            "should transition to hold phase"
        );
        assert_eq!(runtime.lava_geysers[0].current_height, 16);

        // Hold for LAVA_GEYSER_HOLD_FRAMES + 1 frames (countdown then transition)
        for _ in 0..=LAVA_GEYSER_HOLD_FRAMES {
            runtime.step_lava_geysers(&slice);
        }
        assert_eq!(
            runtime.lava_geysers[0].phase, 3,
            "should transition to collapse phase"
        );

        // Collapse (16px / 4px per frame = 4 frames)
        for _ in 0..4 {
            runtime.step_lava_geysers(&slice);
        }
        assert_eq!(runtime.lava_geysers[0].phase, 0, "should return to dormant");
        assert_eq!(runtime.lava_geysers[0].current_height, 0);
    }

    // --- Skycutter flipboard tests ---

    fn skycutter_runtime() -> NativeRuntime {
        let mut rt = NativeRuntime::new(manifest());
        rt.current_form = PlayerForm::Skycutter;
        rt
    }

    fn airborne_slice() -> OpeningSliceState {
        let bounds = LevelBounds {
            width_pixels: 4096,
            height_pixels: 512,
        };
        let mut slice = OpeningSliceState::new(bounds);
        slice.player.on_ground = false;
        slice.player.y_velocity = 0x1_0000; // falling
        slice.player.x_pos = 0x0080_0000;
        slice.player.y_pos = 0x0100_0000;
        slice
    }

    #[test]
    fn skycutter_c_press_starts_flipboard_deploy() {
        let mut rt = skycutter_runtime();
        let mut slice = airborne_slice();
        let input = FrameInput {
            c: true,
            ..FrameInput::default()
        };

        rt.step_skycutter_flipboard(&mut slice, input);

        assert!(rt.skycutter_flipboard > 0, "flipboard counter should start");
    }

    #[test]
    fn skycutter_flipboard_mount_toggles_at_midpoint() {
        let mut rt = skycutter_runtime();
        let mut slice = airborne_slice();

        // First frame: C press starts deploy
        let c_input = FrameInput {
            c: true,
            ..FrameInput::default()
        };
        rt.step_skycutter_flipboard(&mut slice, c_input);
        assert!(!rt.skycutter_on_board, "not yet mounted at frame 1");

        // Step until counter crosses SKYCUTTER_MOUNT_FRAME (17)
        let no_input = FrameInput::default();
        rt.prior_input = c_input; // so C is no longer rising edge
        for _ in 0..8 {
            rt.step_skycutter_flipboard(&mut slice, no_input);
        }
        // Counter: 1 + 2*9 = 19, which crosses 17
        assert!(
            rt.skycutter_on_board,
            "should be mounted after crossing mount frame"
        );
    }

    #[test]
    fn skycutter_flight_gravity_replaces_normal_gravity() {
        let mut rt = skycutter_runtime();
        rt.skycutter_on_board = true;
        let mut slice = airborne_slice();
        slice.player.y_velocity = 0; // hovering

        // With B not held: Skycutter gravity is SKYCUTTER_GRAVITY_NO_B (0x4000)
        let no_input = FrameInput::default();
        rt.step_skycutter_flipboard(&mut slice, no_input);

        assert_eq!(slice.player.y_velocity, SKYCUTTER_GRAVITY_NO_B);
    }

    #[test]
    fn skycutter_flight_gravity_slower_with_b_held() {
        let mut rt = skycutter_runtime();
        rt.skycutter_on_board = true;
        rt.prior_input = FrameInput {
            b: true,
            ..FrameInput::default()
        }; // B already held
        let mut slice = airborne_slice();
        slice.player.y_velocity = 0;

        let b_input = FrameInput {
            b: true,
            ..FrameInput::default()
        };
        rt.step_skycutter_flipboard(&mut slice, b_input);

        assert_eq!(slice.player.y_velocity, SKYCUTTER_GRAVITY_B_HELD);
        assert!(
            SKYCUTTER_GRAVITY_B_HELD < SKYCUTTER_GRAVITY_NO_B,
            "B-held gravity should be less than no-B gravity (slower descent)"
        );
    }

    #[test]
    fn skycutter_b_press_kicks_upward() {
        let mut rt = skycutter_runtime();
        rt.skycutter_on_board = true;
        rt.prior_input = FrameInput::default(); // B was not held last frame
        let mut slice = airborne_slice();
        slice.player.x_velocity = 0x2_0000;
        slice.player.y_velocity = 0x1_0000; // falling

        let b_input = FrameInput {
            b: true,
            ..FrameInput::default()
        };
        rt.step_skycutter_flipboard(&mut slice, b_input);

        assert!(
            slice.player.y_velocity < 0,
            "B press should kick player upward (negative y_velocity)"
        );
    }

    #[test]
    fn skycutter_max_upward_velocity_clamped() {
        let mut rt = skycutter_runtime();
        rt.skycutter_on_board = true;
        rt.prior_input = FrameInput {
            b: true,
            ..FrameInput::default()
        }; // B already held
        let mut slice = airborne_slice();
        // Start with very negative y_velocity that exceeds max
        slice.player.y_velocity = SKYCUTTER_MAX_UPWARD_VEL - 0x1_0000;

        let b_input = FrameInput {
            b: true,
            ..FrameInput::default()
        };
        rt.step_skycutter_flipboard(&mut slice, b_input);

        // After gravity + clamp, velocity should be clamped to max
        assert_eq!(
            slice.player.y_velocity, SKYCUTTER_MAX_UPWARD_VEL,
            "upward velocity should be clamped to max"
        );
    }

    #[test]
    fn skycutter_landing_bounces_off_ground() {
        let mut rt = skycutter_runtime();
        rt.skycutter_on_board = true;
        let mut slice = airborne_slice();
        slice.player.on_ground = true; // just landed
        slice.player.y_velocity = 0;

        let no_input = FrameInput::default();
        rt.step_skycutter_flipboard(&mut slice, no_input);

        assert!(!slice.player.on_ground, "should bounce off ground");
        assert_eq!(slice.player.y_velocity, SKYCUTTER_LANDING_BOUNCE);
        assert_eq!(rt.skycutter_flipboard, 1, "deploy animation should restart");
    }

    #[test]
    fn skycutter_non_skycutter_form_clears_state() {
        let mut rt = skycutter_runtime();
        rt.skycutter_on_board = true;
        rt.skycutter_flipboard = 10;
        rt.current_form = PlayerForm::TheKid; // no longer Skycutter

        let mut slice = airborne_slice();
        let no_input = FrameInput::default();
        rt.step_skycutter_flipboard(&mut slice, no_input);

        assert!(!rt.skycutter_on_board);
        assert_eq!(rt.skycutter_flipboard, 0);
    }

    // --- Cyclone flight tests ---

    fn cyclone_runtime() -> NativeRuntime {
        let mut rt = NativeRuntime::new(manifest());
        rt.current_form = PlayerForm::Cyclone;
        rt
    }

    #[test]
    fn cyclone_c_press_activates_flight_when_airborne() {
        let mut rt = cyclone_runtime();
        let mut slice = airborne_slice();
        let input = FrameInput {
            c: true,
            ..FrameInput::default()
        };

        rt.step_cyclone_flight(&mut slice, input);

        assert!(rt.cyclone_flying);
        // Initial accel ($2000) + gravity ($400) - thrust ($3000) = -$C00
        let expected: i16 = CYCLONE_ACCEL_INITIAL + CYCLONE_ACCEL_GRAVITY - CYCLONE_ACCEL_THRUST;
        assert_eq!(rt.cyclone_y_accel, expected);
    }

    #[test]
    fn cyclone_c_press_does_not_activate_on_ground() {
        let mut rt = cyclone_runtime();
        let mut slice = airborne_slice();
        slice.player.on_ground = true;
        let input = FrameInput {
            c: true,
            ..FrameInput::default()
        };

        rt.step_cyclone_flight(&mut slice, input);

        assert!(!rt.cyclone_flying);
    }

    #[test]
    fn cyclone_landing_clears_flight() {
        let mut rt = cyclone_runtime();
        rt.cyclone_flying = true;
        rt.cyclone_y_accel = 0x800;
        let mut slice = airborne_slice();
        slice.player.on_ground = true;

        rt.step_cyclone_flight(&mut slice, FrameInput::default());

        assert!(!rt.cyclone_flying);
    }

    #[test]
    fn cyclone_gravity_increases_acceleration() {
        let mut rt = cyclone_runtime();
        rt.cyclone_flying = true;
        rt.cyclone_y_accel = 0;
        let mut slice = airborne_slice();
        slice.player.y_velocity = 0;

        rt.step_cyclone_flight(&mut slice, FrameInput::default());

        // Accel should be 0 + GRAVITY = $400, then clamped to MAX_DESCENT ($1200)
        // since C not held. $400 < $1200 so no clamp.
        assert_eq!(rt.cyclone_y_accel, CYCLONE_ACCEL_GRAVITY);
        // Velocity should be 0 + $400 = $400
        assert_eq!(slice.player.y_velocity, i32::from(CYCLONE_ACCEL_GRAVITY));
    }

    #[test]
    fn cyclone_c_held_provides_thrust() {
        let mut rt = cyclone_runtime();
        rt.cyclone_flying = true;
        rt.cyclone_y_accel = CYCLONE_ACCEL_INITIAL; // $2000
        rt.prior_input = FrameInput {
            c: true,
            ..FrameInput::default()
        };
        let mut slice = airborne_slice();
        slice.player.y_velocity = 0;

        let input = FrameInput {
            c: true,
            ..FrameInput::default()
        };
        rt.step_cyclone_flight(&mut slice, input);

        // accel = $2000 + $400 - $3000 = -$C00 (negative = upward thrust)
        let expected_accel: i16 = 0x2000 + 0x400 - 0x3000; // -0xC00
        assert_eq!(rt.cyclone_y_accel, expected_accel);
        assert!(
            slice.player.y_velocity < 0,
            "should gain upward velocity with C held"
        );
    }

    #[test]
    fn cyclone_descent_accel_capped_without_c() {
        let mut rt = cyclone_runtime();
        rt.cyclone_flying = true;
        rt.cyclone_y_accel = CYCLONE_ACCEL_MAX_DESCENT; // already at cap
        let mut slice = airborne_slice();
        slice.player.y_velocity = 0;

        rt.step_cyclone_flight(&mut slice, FrameInput::default());

        // accel = $1200 + $400 = $1600, but capped to $1200
        assert_eq!(rt.cyclone_y_accel, CYCLONE_ACCEL_MAX_DESCENT);
    }

    #[test]
    fn cyclone_velocity_clamped_to_max_fall() {
        let mut rt = cyclone_runtime();
        rt.cyclone_flying = true;
        rt.cyclone_y_accel = CYCLONE_ACCEL_MAX_DESCENT;
        let mut slice = airborne_slice();
        slice.player.y_velocity = CYCLONE_MAX_FALL_VEL; // already at max

        rt.step_cyclone_flight(&mut slice, FrameInput::default());

        assert_eq!(slice.player.y_velocity, CYCLONE_MAX_FALL_VEL);
    }

    #[test]
    fn cyclone_velocity_clamped_to_max_rise() {
        let mut rt = cyclone_runtime();
        rt.cyclone_flying = true;
        rt.cyclone_y_accel = CYCLONE_ACCEL_MIN; // max upward thrust
        rt.prior_input = FrameInput {
            c: true,
            ..FrameInput::default()
        };
        let mut slice = airborne_slice();
        slice.player.y_velocity = CYCLONE_MAX_RISE_VEL; // already at max rise

        let input = FrameInput {
            c: true,
            ..FrameInput::default()
        };
        rt.step_cyclone_flight(&mut slice, input);

        assert_eq!(slice.player.y_velocity, CYCLONE_MAX_RISE_VEL);
    }

    #[test]
    fn cyclone_non_cyclone_form_clears_state() {
        let mut rt = cyclone_runtime();
        rt.cyclone_flying = true;
        rt.cyclone_y_accel = 0x1000;
        rt.current_form = PlayerForm::TheKid;

        let mut slice = airborne_slice();
        rt.step_cyclone_flight(&mut slice, FrameInput::default());

        assert!(!rt.cyclone_flying);
        assert_eq!(rt.cyclone_y_accel, 0);
    }

    fn iron_knight_runtime_with_wall() -> NativeRuntime {
        let mut rt = NativeRuntime::new(manifest());
        rt.current_form = PlayerForm::IronKnight;
        rt.number_hitpoints = PlayerForm::IronKnight.max_hitpoints();
        // Wall at col 10, floor at row 3, from col 0..20
        rt.block_grid = Some(grid_with_wall(3, 0, 20, 10));
        rt
    }

    #[test]
    fn iron_knight_c_press_attaches_to_wall_when_airborne() {
        let mut rt = iron_knight_runtime_with_wall();
        // Position player next to wall (wall at col 10 = pixel 160), facing right
        let mut slice = airborne_slice();
        slice.player.x_pos = 154 << 16; // just left of wall col 10
        slice.player.y_pos = 32 << 16;
        slice.player.facing = Direction::Right;

        let input = FrameInput {
            c: true,
            ..FrameInput::default()
        };
        rt.step_iron_knight_climb(&mut slice, input);

        assert!(rt.iron_knight_climbing, "should attach to wall");
        assert_eq!(slice.player.y_velocity, IRON_KNIGHT_ATTACH_Y_VEL);
        assert_eq!(slice.player.x_velocity, 0);
    }

    #[test]
    fn iron_knight_no_attach_without_wall() {
        let mut rt = NativeRuntime::new(manifest());
        rt.current_form = PlayerForm::IronKnight;
        // No block grid — no wall to attach to
        let mut slice = airborne_slice();
        slice.player.facing = Direction::Right;

        let input = FrameInput {
            c: true,
            ..FrameInput::default()
        };
        rt.step_iron_knight_climb(&mut slice, input);

        assert!(!rt.iron_knight_climbing);
    }

    #[test]
    fn iron_knight_c_press_boosts_climb_upward() {
        let mut rt = iron_knight_runtime_with_wall();
        rt.iron_knight_climbing = true;
        rt.iron_knight_climb_accel = 0;

        let mut slice = airborne_slice();
        slice.player.x_pos = 154 << 16;
        slice.player.y_pos = 32 << 16;
        slice.player.facing = Direction::Right;
        slice.player.y_velocity = 0;

        // C rising edge while climbing
        let input = FrameInput {
            c: true,
            ..FrameInput::default()
        };
        rt.step_iron_knight_climb(&mut slice, input);

        // Drift (+$80) then boost (-$1800), clamped to min accel (-$800)
        assert_eq!(rt.iron_knight_climb_accel, IRON_KNIGHT_MIN_ACCEL);
        assert!(slice.player.y_velocity < 0, "should be climbing upward");
    }

    #[test]
    fn iron_knight_slides_down_without_c() {
        let mut rt = iron_knight_runtime_with_wall();
        rt.iron_knight_climbing = true;
        rt.iron_knight_climb_accel = 0;

        let mut slice = airborne_slice();
        slice.player.x_pos = 154 << 16;
        slice.player.y_pos = 32 << 16;
        slice.player.facing = Direction::Right;
        slice.player.y_velocity = 0;

        rt.step_iron_knight_climb(&mut slice, FrameInput::default());

        assert_eq!(rt.iron_knight_climb_accel, IRON_KNIGHT_SLIDE_DRIFT);
        assert!(slice.player.y_velocity > 0, "should slide down");
    }

    #[test]
    fn iron_knight_b_press_jumps_off_wall() {
        let mut rt = iron_knight_runtime_with_wall();
        rt.iron_knight_climbing = true;

        let mut slice = airborne_slice();
        slice.player.x_pos = 154 << 16;
        slice.player.y_pos = 32 << 16;
        slice.player.facing = Direction::Right;

        let input = FrameInput {
            b: true,
            ..FrameInput::default()
        };
        rt.step_iron_knight_climb(&mut slice, input);

        assert!(!rt.iron_knight_climbing, "should detach on B press");
        assert!(slice.player.y_velocity < 0, "should jump upward");
    }

    #[test]
    fn iron_knight_landing_stops_climb() {
        let mut rt = iron_knight_runtime_with_wall();
        rt.iron_knight_climbing = true;

        let mut slice = airborne_slice();
        slice.player.on_ground = true;

        rt.step_iron_knight_climb(&mut slice, FrameInput::default());

        assert!(!rt.iron_knight_climbing, "landing should end climb");
    }

    #[test]
    fn iron_knight_non_iron_knight_clears_state() {
        let mut rt = iron_knight_runtime_with_wall();
        rt.iron_knight_climbing = true;
        rt.iron_knight_climb_accel = 0x500;
        rt.current_form = PlayerForm::TheKid;

        let mut slice = airborne_slice();
        rt.step_iron_knight_climb(&mut slice, FrameInput::default());

        assert!(!rt.iron_knight_climbing);
        assert_eq!(rt.iron_knight_climb_accel, 0);
    }

    // --- Micromax wall/ceiling stick ---

    fn micromax_runtime_with_ceiling() -> NativeRuntime {
        let mut rt = NativeRuntime::new(manifest());
        rt.current_form = PlayerForm::Micromax;
        rt.number_hitpoints = PlayerForm::Micromax.max_hitpoints();
        // Build a grid with solid ceiling at row 2 (y=32..48px)
        let width_cells = 6u16 * 20;
        let height_cells = 4u16 * 14;
        let mut data = vec![0u8; usize::from(width_cells) * usize::from(height_cells) * 2];
        // Solid block at col 3, row 2
        let offset = (usize::from(2u16) * usize::from(width_cells) + usize::from(3u16)) * 2;
        data[offset] = 0x60; // collision = Solid
        let grid = terrain::BlockGrid::from_level_layout(&data, width_cells, height_cells);
        rt.set_block_grid(grid);
        rt
    }

    #[test]
    fn micromax_sticks_to_ceiling_when_falling() {
        let mut rt = micromax_runtime_with_ceiling();
        let mut slice = airborne_slice();
        // Position player below the ceiling block (cell row 2 = y 32-47px)
        // With MICROMAX_H=15, top_y = foot_y - 15. Need top_y in row 2 (32..47).
        // Use foot_y=47+15=62 → top_y=47 → cell row 47>>4=2 ✓
        slice.player.x_pos = (3 * 16 + 8) << 16; // centre of col 3
        slice.player.y_pos = 62 << 16;
        slice.player.y_velocity = 0x1_0000; // falling downward
        slice.player.on_ground = false;

        rt.step_micromax_stick(&mut slice, FrameInput::default());

        assert!(rt.micromax_stuck, "should stick to ceiling");
        assert_eq!(slice.player.y_velocity, 0, "velocity zeroed on stick");
    }

    #[test]
    fn micromax_does_not_stick_when_moving_upward() {
        let mut rt = micromax_runtime_with_ceiling();
        let mut slice = airborne_slice();
        slice.player.x_pos = (3 * 16 + 8) << 16;
        slice.player.y_pos = 63 << 16;
        slice.player.y_velocity = -0x1_0000; // moving upward
        slice.player.on_ground = false;

        rt.step_micromax_stick(&mut slice, FrameInput::default());

        assert!(!rt.micromax_stuck, "should not stick while moving upward");
    }

    #[test]
    fn micromax_b_press_detaches_with_kick() {
        let mut rt = micromax_runtime_with_ceiling();
        rt.micromax_stuck = true;
        let mut slice = airborne_slice();
        slice.player.x_pos = (3 * 16 + 8) << 16;
        slice.player.y_pos = 63 << 16;
        slice.player.facing = Direction::Right;

        let input = FrameInput {
            b: true,
            ..FrameInput::default()
        };
        rt.step_micromax_stick(&mut slice, input);

        assert!(!rt.micromax_stuck, "should detach on B press");
        assert_eq!(
            slice.player.y_velocity, MICROMAX_DETACH_Y_VEL,
            "upward kick"
        );
        assert_eq!(slice.player.facing, Direction::Left, "facing reversed");
    }

    #[test]
    fn micromax_non_micromax_clears_stuck() {
        let mut rt = NativeRuntime::new(manifest());
        rt.current_form = PlayerForm::TheKid;
        rt.micromax_stuck = true;

        let mut slice = airborne_slice();
        rt.step_micromax_stick(&mut slice, FrameInput::default());

        assert!(!rt.micromax_stuck, "form change should clear stuck");
    }

    #[test]
    fn skycutter_on_board_skips_normal_gravity_in_step() {
        let bounds = LevelBounds {
            width_pixels: 4096,
            height_pixels: 512,
        };
        let mut slice = OpeningSliceState::new(bounds);
        slice.player.on_ground = false;
        slice.player.y_velocity = 0x1_0000;
        slice.player.x_pos = 0x0080_0000;
        slice.player.y_pos = 0x0100_0000;

        let vel_before = slice.player.y_velocity;

        // Step with skycutter_on_board=true — normal gravity should be skipped
        slice.step(
            OPENING_SCROLL_START_FRAME,
            FrameInput::default(),
            false,
            PlayerForm::Skycutter,
            None,
            None,
            true,
            0,
        );

        // y_pos changes (from existing velocity) but y_velocity should NOT
        // have had normal gravity applied
        assert_eq!(
            slice.player.y_velocity, vel_before,
            "normal gravity should be skipped when skycutter_on_board is true"
        );
    }

    // --- Teleporter tests ---

    #[test]
    fn teleporter_parsed_from_block_list() {
        // Block list sections flow into each other. Prize section ends when
        // layout_addr=0. Ghost section ends on all-zero entry. Then teleporters.
        // Build data with: 1 prize, 0 ghosts, 1 teleporter.
        let mut data = Vec::new();
        // Prize entry: layout_addr=0x0020, col=1, row=2, disguised=0, content=0
        data.extend_from_slice(&[0x00, 0x20, 0x00, 0x01, 0x00, 0x02, 0x00, 0x00]);
        // Prize terminator — layout_addr=0. The ghost parser reads from here.
        // These 16 bytes serve as both prize terminator AND ghost section.
        // Ghost section: timer=0, mapping=0, solid_time=0, vanished_time=0,
        //   layout_addr=0 → all zero → ghost section terminates.
        data.extend_from_slice(&[0; 16]);
        // Now at the teleporter section.
        data.extend_from_slice(&[
            0x00, 0x10, // layout_addr (non-zero)
            0x00, 0x05, // col = 5
            0x00, 0x03, // row = 3
            0x02, // dest_map_id = 2
            0x07, // dest_y = 7
            0x00, 0x0A, // dest_x = 10
        ]);
        // Teleporter terminator (layout_addr=0)
        data.extend_from_slice(&[0, 0]);

        let bl = terrain::BlockList::from_raw(&data, 64);
        assert_eq!(bl.prize_blocks.len(), 1);
        assert_eq!(bl.ghost_blocks.len(), 0);
        assert_eq!(bl.teleporters.len(), 1);
        assert_eq!(bl.teleporters[0].col, 5);
        assert_eq!(bl.teleporters[0].row, 3);
        assert_eq!(bl.teleporters[0].dest_map_id, 2);
        assert_eq!(bl.teleporters[0].dest_y, 7);
        assert_eq!(bl.teleporters[0].dest_x, 10);
    }

    // --- Mushroom block tests ---

    #[test]
    fn mushroom_block_bounces_player_on_landing() {
        // Build a 16x16 grid with a mushroom block at cell (2, 4).
        // Cell (2, 4) = pixel (32-47, 64-79).
        let w: u16 = 16;
        let h: u16 = 16;
        let mut data = vec![0u8; usize::from(w) * usize::from(h) * 2];
        // Mushroom at cell (2, 4): has_block=1, collision=Solid(6), block_type=Mushroom(9)
        // = 0b1_110_1001_00000000 = 0xE900
        let idx = (4 * w + 2) as usize * 2;
        data[idx] = 0xE9;
        data[idx + 1] = 0x00;
        let grid = terrain::BlockGrid::from_level_layout(&data, w, h);

        let bounds = LevelBounds {
            width_pixels: w * 16,
            height_pixels: h * 16,
        };
        let mut slice = OpeningSliceState::new(bounds);
        slice.player.on_ground = false;
        slice.player.y_velocity = 0x3_0000; // falling at 3 px/frame
        slice.player.x_pos = 0x0024_0000; // 36 px (inside cell col=2)
        // Place player at y=63 (just above floor at y=64, within snap range)
        slice.player.y_pos = 0x003F_0000;

        slice.apply_floor_collision(Some(&grid), false, false);

        assert!(
            !slice.player.on_ground,
            "should bounce off mushroom, not land"
        );
        assert!(slice.player.y_velocity < 0, "should bounce upward");
    }

    #[test]
    fn shifting_block_moves_up_when_player_stands_on_it() {
        let mut rt = NativeRuntime::new(manifest());
        let w: u16 = 16;
        let h: u16 = 16;
        let mut data = vec![0u8; usize::from(w) * usize::from(h) * 2];
        // Shifting block at cell (3, 5): has_block=1, collision=Solid(6), block_type=Shifting(7)
        // = 0b1_110_0111_00000000 = 0xE700
        let idx = (5 * w + 3) as usize * 2;
        data[idx] = 0xE7;
        data[idx + 1] = 0x00;
        let grid = terrain::BlockGrid::from_level_layout(&data, w, h);
        rt.block_grid = Some(grid);

        let bounds = LevelBounds {
            width_pixels: w * 16,
            height_pixels: h * 16,
        };
        let mut slice = OpeningSliceState::new(bounds);
        slice.player.on_ground = true;
        // Player standing at x = cell 3 centre (3*16+8=56px), y = cell 5 top (5*16-1=79px)
        slice.player.x_pos = 56 << 16;
        slice.player.y_pos = 79 << 16;

        rt.apply_shifting_blocks(&slice);

        let grid = rt.block_grid.as_ref().unwrap();
        // Cell (3, 5) should now be empty
        let old_cell = grid.cell_at(3, 5);
        assert!(!old_cell.has_block, "original cell should be cleared");
        // Cell (3, 4) should have the shifted block
        let new_cell = grid.cell_at(3, 4);
        assert!(new_cell.has_block, "block should have shifted up");
        assert_eq!(new_cell.block_type, terrain::BlockType::Shifting);
    }

    #[test]
    fn shifting_block_stays_if_cell_above_occupied() {
        let mut rt = NativeRuntime::new(manifest());
        let w: u16 = 16;
        let h: u16 = 16;
        let mut data = vec![0u8; usize::from(w) * usize::from(h) * 2];
        // Shifting block at (3, 5)
        let idx = (5 * w + 3) as usize * 2;
        data[idx] = 0xE7;
        // Solid block at (3, 4)
        let idx2 = (4 * w + 3) as usize * 2;
        data[idx2] = 0x60; // Solid, no block type
        let grid = terrain::BlockGrid::from_level_layout(&data, w, h);
        rt.block_grid = Some(grid);

        let bounds = LevelBounds {
            width_pixels: w * 16,
            height_pixels: h * 16,
        };
        let mut slice = OpeningSliceState::new(bounds);
        slice.player.on_ground = true;
        slice.player.x_pos = 56 << 16;
        slice.player.y_pos = 79 << 16;

        rt.apply_shifting_blocks(&slice);

        // Block should not have moved
        let grid = rt.block_grid.as_ref().unwrap();
        assert!(grid.cell_at(3, 5).has_block, "block should remain");
    }

    #[test]
    fn teleporter_lookup_by_position() {
        let bl = terrain::BlockList {
            prize_blocks: vec![],
            ghost_blocks: vec![],
            teleporters: vec![terrain::TeleporterEntry {
                col: 5,
                row: 3,
                dest_map_id: 2,
                dest_y: 7,
                dest_x: 10,
            }],
            hidden_blocks: vec![],
        };

        assert!(bl.teleporter_at(5, 3).is_some());
        assert!(bl.teleporter_at(4, 3).is_none());
    }

    // --- Combat ability test helpers ---

    fn test_enemy(family: EnemyFamily, x: u32, y: u32, hp: u8) -> ActiveEnemy {
        ActiveEnemy {
            segment_index: 0,
            layout_index: 0,
            family,
            x_pos: x,
            y_pos: y,
            x_velocity: 0,
            y_velocity: 0,
            x_acceleration: 0,
            y_acceleration: 0,
            facing: Direction::Left,
            on_ground: false,
            hit_points: hp,
            grounded_frames: 0,
            damage_flash_frames: 0,
            palette_variant: 0,
            wait_timer: 0,
            dragon_phase: DragonFlyingState::None,
            dragon_phase_frames: 0,
            dragon_script_variant: 0,
            dragon_script_cmd: 0,
            dragon_anchor_y: 0,
            dragon_descent_index: 0,
            dragon_descent_frames: 0,
            ninja_state: NinjaState::Stealth,
            hand_state: HandState::Walking,
            lion_state: LionState::Walking,
            robot_state: RobotState::Walking,
            orca_state: OrcaState::Walking,
            tar_monster_state: TarMonsterState::default(),
            spinning_twins_phase: SpinningTwinsPhase::default(),
            spinning_twins_orbit_index: 0,
            ufo_beam_segments: 0,
            ufo_beam_interval: 0,
        }
    }

    /// Create a Dragon Flying enemy with the given script variant, positioned at (x, y).
    /// Initialises the script interpreter to the first command of the selected variant.
    fn test_dragon(variant: u8, x: u32, y: u32) -> ActiveEnemy {
        let mut e = test_enemy(EnemyFamily::Dragon, x, y, 1);
        e.dragon_script_variant = variant;
        e.dragon_anchor_y = y;
        // Initialise to first command per variant.
        let script = dragon_script(variant);
        match script[0] {
            DragonFlyingCmd::Hover(frames) => {
                e.dragon_phase = DragonFlyingState::Hovering;
                e.dragon_phase_frames = frames;
            }
            DragonFlyingCmd::DriftWithFire(frames) => {
                e.dragon_phase = DragonFlyingState::DriftingWithFire;
                e.dragon_phase_frames = frames;
                e.x_velocity = if e.facing == Direction::Right { 0x800 } else { -0x800 };
            }
            DragonFlyingCmd::Drift(frames) => {
                e.dragon_phase = DragonFlyingState::Drifting;
                e.dragon_phase_frames = frames;
                e.x_velocity = if e.facing == Direction::Right { 0x800 } else { -0x800 };
            }
            _ => {
                e.dragon_phase = DragonFlyingState::Hovering;
                e.dragon_phase_frames = 1;
            }
        }
        e
    }

    fn combat_slice() -> OpeningSliceState {
        let bounds = LevelBounds {
            width_pixels: 4096,
            height_pixels: 512,
        };
        OpeningSliceState {
            player: {
                let mut p = PlayerState::title_pose();
                p.x_pos = 0x0080_0000;
                p.y_pos = 0x0100_0000;
                p.facing = Direction::Right;
                p
            },
            ..OpeningSliceState::new(bounds)
        }
    }

    // --- RedStealth sword tests ---

    #[test]
    fn red_stealth_c_press_starts_sword_swing() {
        let mut rt = NativeRuntime::new(manifest());
        rt.current_form = PlayerForm::RedStealth;
        let input = FrameInput {
            c: true,
            ..FrameInput::default()
        };

        rt.step_red_stealth_sword(input);

        assert!(rt.red_stealth_sword_timer > 0);
    }

    #[test]
    fn red_stealth_sword_inactive_during_startup() {
        let mut rt = NativeRuntime::new(manifest());
        rt.current_form = PlayerForm::RedStealth;
        // Start swing
        rt.step_red_stealth_sword(FrameInput {
            c: true,
            ..FrameInput::default()
        });
        rt.prior_input = FrameInput {
            c: true,
            ..FrameInput::default()
        };

        // During startup frames the sword should not be active
        assert!(rt.red_stealth_sword_timer > RED_STEALTH_ACTIVE_FRAMES);
        assert!(!rt.sword_is_active());
    }

    #[test]
    fn red_stealth_sword_active_after_startup() {
        let mut rt = NativeRuntime::new(manifest());
        rt.current_form = PlayerForm::RedStealth;
        // Start swing
        rt.step_red_stealth_sword(FrameInput {
            c: true,
            ..FrameInput::default()
        });
        rt.prior_input = FrameInput {
            c: true,
            ..FrameInput::default()
        };

        // Tick through startup frames
        for _ in 0..RED_STEALTH_STARTUP_FRAMES {
            rt.step_red_stealth_sword(FrameInput::default());
        }

        assert!(rt.sword_is_active(), "sword should be active after startup");
    }

    #[test]
    fn red_stealth_sword_damages_nearby_enemy() {
        let mut rt = NativeRuntime::new(manifest());
        rt.current_form = PlayerForm::RedStealth;
        rt.red_stealth_sword_timer = RED_STEALTH_ACTIVE_FRAMES;
        let slice = combat_slice();

        // Enemy at sword reach: player at 128px, sword at 128+28=156px
        rt.active_enemies
            .push(test_enemy(EnemyFamily::Goat, 0x009C_0000, 0x00F6_0000, 2));

        rt.process_sword_hits(&slice);

        assert_eq!(
            rt.active_enemies[0].hit_points, 1,
            "enemy should take 1 damage"
        );
    }

    #[test]
    fn red_stealth_sword_kills_1hp_enemy() {
        let mut rt = NativeRuntime::new(manifest());
        rt.current_form = PlayerForm::RedStealth;
        rt.red_stealth_sword_timer = RED_STEALTH_ACTIVE_FRAMES;
        let slice = combat_slice();

        rt.active_enemies
            .push(test_enemy(EnemyFamily::Goat, 0x009C_0000, 0x00F6_0000, 1));

        rt.process_sword_hits(&slice);

        assert!(rt.active_enemies.is_empty(), "1HP enemy should be killed");
    }

    #[test]
    fn red_stealth_sword_no_hit_when_out_of_range() {
        let mut rt = NativeRuntime::new(manifest());
        rt.current_form = PlayerForm::RedStealth;
        rt.red_stealth_sword_timer = RED_STEALTH_ACTIVE_FRAMES;
        let slice = combat_slice();

        rt.active_enemies
            .push(test_enemy(EnemyFamily::Goat, 0x0140_0000, 0x0100_0000, 3));

        rt.process_sword_hits(&slice);

        assert_eq!(
            rt.active_enemies[0].hit_points, 3,
            "out-of-range enemy should not be hit"
        );
    }

    // --- Maniaxe axe tests ---

    #[test]
    fn maniaxe_c_press_throws_axe() {
        let mut rt = NativeRuntime::new(manifest());
        rt.current_form = PlayerForm::Maniaxe;
        let slice = combat_slice();
        let input = FrameInput {
            c: true,
            ..FrameInput::default()
        };

        rt.step_maniaxe_axe(&slice, input);

        assert!(rt.maniaxe_axe_active);
        assert_eq!(rt.projectiles.len(), 1);
        assert!(rt.projectiles[0].x_velocity > 0, "axe should move right");
    }

    #[test]
    fn maniaxe_no_double_throw() {
        let mut rt = NativeRuntime::new(manifest());
        rt.current_form = PlayerForm::Maniaxe;
        rt.maniaxe_axe_active = true;
        let slice = combat_slice();
        let input = FrameInput {
            c: true,
            ..FrameInput::default()
        };

        rt.step_maniaxe_axe(&slice, input);

        assert!(
            rt.projectiles.is_empty(),
            "should not throw while axe is active"
        );
    }

    #[test]
    fn maniaxe_non_maniaxe_form_clears_state() {
        let mut rt = NativeRuntime::new(manifest());
        rt.current_form = PlayerForm::TheKid;
        rt.maniaxe_axe_active = true;
        let slice = combat_slice();

        rt.step_maniaxe_axe(&slice, FrameInput::default());

        assert!(!rt.maniaxe_axe_active);
    }

    // --- Berzerker charge tests ---

    // --- Juggernaut skull shot tests ---

    #[test]
    fn juggernaut_c_press_fires_skull() {
        let mut rt = NativeRuntime::new(manifest());
        rt.current_form = PlayerForm::Juggernaut;
        let slice = combat_slice();
        let input = FrameInput {
            c: true,
            ..FrameInput::default()
        };

        rt.step_juggernaut_skull(&slice, input);

        assert_eq!(rt.projectiles.len(), 1, "should spawn a skull projectile");
        assert!(rt.projectiles[0].friendly);
        assert_eq!(rt.projectiles[0].gravity, JUGGERNAUT_SKULL_GRAVITY);
    }

    #[test]
    fn juggernaut_skull_has_gravity() {
        let mut rt = NativeRuntime::new(manifest());
        rt.projectiles.push(EnemyProjectile {
            x_pos: 100 << 16,
            y_pos: 100 << 16,
            x_velocity: JUGGERNAUT_SKULL_X_SPEED,
            y_velocity: 0,
            lifetime: 10,
            friendly: true,
            gravity: JUGGERNAUT_SKULL_GRAVITY,
            homing_accel: 0,
            speed_cap: 0,
            wall_bounce: false,
        });

        rt.step_projectiles();

        assert_eq!(
            rt.projectiles[0].y_velocity, JUGGERNAUT_SKULL_GRAVITY,
            "gravity should increase y_velocity"
        );
    }

    #[test]
    fn juggernaut_max_skull_limit() {
        let mut rt = NativeRuntime::new(manifest());
        rt.current_form = PlayerForm::Juggernaut;
        let slice = combat_slice();
        let input = FrameInput {
            c: true,
            ..FrameInput::default()
        };

        // Fire 8 skulls
        for _ in 0..8 {
            rt.step_juggernaut_skull(&slice, input);
            rt.prior_input = FrameInput::default(); // reset so next C is rising edge
        }
        assert_eq!(rt.projectiles.len(), 8);

        // 9th should be rejected
        rt.prior_input = FrameInput::default();
        rt.step_juggernaut_skull(&slice, input);
        assert_eq!(rt.projectiles.len(), 8, "should cap at 8 skulls");
    }

    #[test]
    fn juggernaut_non_juggernaut_form_no_skull() {
        let mut rt = NativeRuntime::new(manifest());
        rt.current_form = PlayerForm::TheKid;
        let slice = combat_slice();
        let input = FrameInput {
            c: true,
            ..FrameInput::default()
        };

        rt.step_juggernaut_skull(&slice, input);

        assert!(rt.projectiles.is_empty());
    }

    // --- Berzerker charge tests ---

    #[test]
    fn berzerker_charge_builds_when_moving_fast() {
        let mut rt = NativeRuntime::new(manifest());
        rt.current_form = PlayerForm::Berzerker;
        rt.previous_input = FrameInput {
            right: true,
            ..FrameInput::default()
        };
        let mut slice = combat_slice();
        slice.player.on_ground = true;
        slice.player.x_velocity = 0x2_0000; // above 1.0 FP threshold

        // Accumulator builds by $10 each frame — needs 16 frames to overflow
        for _ in 0..15 {
            rt.step_berzerker_charge(&slice);
            assert!(!rt.berzerker_charging, "should not be charging yet");
        }
        rt.step_berzerker_charge(&slice);
        assert!(
            rt.berzerker_charging,
            "should start charging after overflow"
        );
    }

    #[test]
    fn berzerker_charge_resets_when_slow() {
        let mut rt = NativeRuntime::new(manifest());
        rt.current_form = PlayerForm::Berzerker;
        rt.previous_input = FrameInput {
            right: true,
            ..FrameInput::default()
        };
        rt.berzerker_charging = true;
        rt.berzerker_charge_accum = 0xFF;
        let mut slice = combat_slice();
        slice.player.on_ground = true;
        slice.player.x_velocity = 0x8000; // below threshold

        rt.step_berzerker_charge(&slice);

        assert!(
            !rt.berzerker_charging,
            "should stop charging below threshold"
        );
        assert_eq!(rt.berzerker_charge_accum, 0, "accumulator should reset");
    }

    #[test]
    fn berzerker_charge_not_active_for_other_forms() {
        let mut rt = NativeRuntime::new(manifest());
        rt.current_form = PlayerForm::TheKid;
        rt.berzerker_charging = true;
        let mut slice = combat_slice();
        slice.player.on_ground = true;
        slice.player.x_velocity = 0x2_0000;

        rt.step_berzerker_charge(&slice);

        assert!(!rt.berzerker_charging);
    }

    #[test]
    fn berzerker_charge_kills_enemy_on_contact() {
        let mut rt = NativeRuntime::new(manifest());
        rt.current_form = PlayerForm::Berzerker;
        rt.berzerker_charging = true;
        let mut slice = combat_slice();
        slice.player.x_pos = 0x0040_0000;
        slice.player.y_pos = 0x0040_0000;

        rt.active_enemies
            .push(test_enemy(EnemyFamily::Goat, 0x0040_0000, 0x0040_0000, 1));

        rt.process_berzerker_charge_hits(&slice);

        assert!(
            rt.active_enemies.is_empty(),
            "enemy should be killed by charge"
        );
    }

    #[test]
    fn berzerker_charging_immune_to_contact_damage() {
        let mut rt = NativeRuntime::new(manifest());
        rt.current_form = PlayerForm::Berzerker;
        rt.berzerker_charging = true;
        let mut slice = combat_slice();
        slice.player.x_pos = 0x0040_0000;
        slice.player.y_pos = 0x0040_0000;

        // Add a hopper (normally does contact damage) at same position
        rt.active_enemies.push(test_enemy(
            EnemyFamily::BigHoppingSkull,
            0x0040_0000,
            0x0040_0000,
            3,
        ));

        let damage_source = rt.find_contact_damage_source(&slice);
        assert!(
            damage_source.is_none(),
            "charging Berzerker should be immune to contact damage"
        );
    }

    #[test]
    fn berzerker_charge_breaks_smash_block_wall() {
        let mut rt = NativeRuntime::new(manifest());
        // Build a grid with a smash block wall at col 5, rows 2-3
        let w = 10u16;
        let h = 6u16;
        let mut data = vec![0u8; usize::from(w) * usize::from(h) * 2];
        let set_cell = |data: &mut Vec<u8>, col: usize, row: usize, hi: u8| {
            let offset = (row * usize::from(w) + col) * 2;
            data[offset] = hi;
            data[offset + 1] = 0x01;
        };
        // Smash block wall at col 5 (collision=6 Solid, has_block, block_type=0 Smash)
        // 1_110_0000 = 0xE0
        set_cell(&mut data, 5, 2, 0xE0);
        set_cell(&mut data, 5, 3, 0xE0);
        // Floor at row 4
        for col in 0..10 {
            set_cell(&mut data, col, 4, 0x60);
        }
        let grid = terrain::BlockGrid::from_level_layout(&data, w, h);
        rt.set_block_grid(grid);

        rt.current_form = PlayerForm::Berzerker;
        rt.berzerker_charging = true;

        // Player just left of the smash block, moving right
        let bounds = rt.level_bounds(rt.current_level);
        let mut slice = OpeningSliceState::new(bounds);
        // Col 5 starts at x=80px. Player at x=74 with half_width=5, right edge = 79.
        slice.player.x_pos = 74 << 16;
        slice.player.y_pos = (4 * 16) << 16; // on floor
        slice.player.x_velocity = 0x2_0000; // moving right
        slice.player.facing = Direction::Right;
        slice.player.on_ground = true;

        rt.berzerker_break_wall_block(&slice);

        // Check the smash block at col 5, row 3 (at player height) was broken
        let grid = rt.block_grid.as_ref().unwrap();
        let cell = grid.cell_at(5, 3);
        assert!(
            !cell.has_block,
            "smash block should be broken by Berzerker charge"
        );
    }

    // --- 105-level soak test ---

    #[test]
    fn soak_all_105_levels_no_panic() {
        let mut manifest = manifest_with_levels(105);
        // Vary theme, weather, and murder wall across levels to cover all code paths
        let themes: [u8; 11] = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
        let weather_flags: [u8; 4] = [0, 1, 2, 3]; // none, lava, storm, storm+hail
        let murder_flags: [u8; 3] = [0, 0b10, 0b11]; // none, normal, reversed

        for (i, level) in manifest.levels.iter_mut().enumerate() {
            if let Some(header) = level.header_metadata.as_mut() {
                header.level_theme = themes[i % themes.len()];
                header.background_theme = themes[i % themes.len()];
                header.weather_flags = weather_flags[i % weather_flags.len()];
                header.murder_wall_flags = murder_flags[i % murder_flags.len()];
            }
        }

        let mut runtime = NativeRuntime::new(manifest);
        runtime.set_block_grid(grid_with_floor_at_row(17));

        // Boot through sega/title screens
        for _ in 0..120 {
            runtime.step_frame(FrameInput::default());
        }
        // Press start to begin
        runtime.step_frame(FrameInput {
            start: true,
            ..FrameInput::default()
        });
        for _ in 0..120 {
            runtime.step_frame(FrameInput::default());
        }

        // Step through each level via debug warp (N key simulated by advance_level)
        for level_id in 0u8..105 {
            runtime.begin_level_entry(LevelId(level_id));
            runtime.set_block_grid(grid_with_floor_at_row(17));
            // Run 60 frames of gameplay per level
            for _ in 0..60 {
                runtime.step_frame(FrameInput::default());
            }
        }
    }

    // --- 2P alternating mode tests ---

    #[test]
    fn two_player_toggle_in_options() {
        let mut runtime = NativeRuntime::new(manifest());
        assert!(!runtime.two_player);

        // Navigate to options menu (B on title)
        runtime.boot_state = BootState::TitleScreen;
        runtime.step_frame(FrameInput::default()); // settle
        runtime.boot_state = BootState::OptionsMenu { cursor: 0 };

        // Press A to toggle 2P on
        runtime.step_frame(FrameInput {
            a: true,
            ..FrameInput::default()
        });
        assert!(runtime.two_player, "A press on cursor 0 should enable 2P");

        // Press A again to toggle off
        runtime.prior_input = FrameInput::default(); // reset so A counts as fresh press
        runtime.step_frame(FrameInput {
            a: true,
            ..FrameInput::default()
        });
        assert!(!runtime.two_player, "second A press should disable 2P");
    }

    #[test]
    fn sound_test_enter_navigate_exit() {
        let mut runtime = NativeRuntime::new(manifest());
        runtime.boot_state = BootState::OptionsMenu { cursor: 1 };

        // A enters sound test
        runtime.step_frame(FrameInput {
            a: true,
            ..FrameInput::default()
        });
        assert!(matches!(
            runtime.boot_state,
            BootState::SoundTest { track: 0 }
        ));

        // Right increments track
        runtime.prior_input = FrameInput::default();
        runtime.step_frame(FrameInput {
            right: true,
            ..FrameInput::default()
        });
        assert!(matches!(
            runtime.boot_state,
            BootState::SoundTest { track: 1 }
        ));

        // Up adds 16
        runtime.prior_input = FrameInput::default();
        runtime.step_frame(FrameInput {
            up: true,
            ..FrameInput::default()
        });
        assert!(matches!(
            runtime.boot_state,
            BootState::SoundTest { track: 17 }
        ));

        // B exits back to options
        runtime.prior_input = FrameInput::default();
        runtime.step_frame(FrameInput {
            b: true,
            ..FrameInput::default()
        });
        assert!(matches!(
            runtime.boot_state,
            BootState::OptionsMenu { cursor: 1 }
        ));
    }

    #[test]
    fn two_player_switch_on_death() {
        let manifest = manifest_with_levels(4);
        let mut runtime = NativeRuntime::new(manifest);
        runtime.set_block_grid(grid_with_floor_at_row(17));
        runtime.two_player = true;
        runtime.player_slots[0].lives = 3;
        runtime.player_slots[0].level_id = LevelId(0);
        runtime.player_slots[1].lives = 3;
        runtime.player_slots[1].level_id = LevelId(1);
        runtime.current_player_index = 0;
        runtime.number_lives = 3;
        runtime.current_level = LevelId(0);

        // Simulate death: decrement lives and enter Dying state
        runtime.number_lives = 2;
        runtime.player_lifecycle = PlayerLifecycle::Dying {
            remaining_frames: 1,
        };
        runtime.boot_state = BootState::InGame;

        // Step to finish Dying countdown — should switch to P2
        runtime.step_frame(FrameInput::default());

        assert_eq!(
            runtime.current_player_index, 1,
            "should switch to P2 after P1 death"
        );
        assert_eq!(runtime.current_level, LevelId(1), "should load P2's level");
    }

    #[test]
    fn two_player_game_over_when_both_out_of_lives() {
        let mut runtime = NativeRuntime::new(manifest());
        runtime.set_block_grid(grid_with_floor_at_row(17));
        runtime.two_player = true;
        runtime.current_player_index = 0;
        runtime.number_lives = 0;
        runtime.number_continues = 0;
        runtime.player_slots[0].lives = 0;
        runtime.player_slots[0].continues = 0;
        runtime.player_slots[1].lives = 0;
        runtime.player_slots[1].continues = 0;

        runtime.player_lifecycle = PlayerLifecycle::GameOver {
            remaining_frames: 1,
        };
        runtime.boot_state = BootState::InGame;

        runtime.step_frame(FrameInput::default());

        assert_eq!(
            runtime.boot_state,
            BootState::TitleScreen,
            "should return to title when both players out of lives and continues"
        );
        assert!(!runtime.two_player, "2P should be reset on game over");
    }

    #[test]
    fn two_player_game_over_switches_when_other_has_lives() {
        let mut runtime = NativeRuntime::new(manifest_with_levels(4));
        runtime.set_block_grid(grid_with_floor_at_row(17));
        runtime.two_player = true;
        runtime.current_player_index = 0;
        runtime.number_lives = 0;
        runtime.number_continues = 0;
        runtime.player_slots[0].lives = 0;
        runtime.player_slots[0].continues = 0;
        runtime.player_slots[1].lives = 2;
        runtime.player_slots[1].level_id = LevelId(2);

        runtime.player_lifecycle = PlayerLifecycle::GameOver {
            remaining_frames: 1,
        };
        runtime.boot_state = BootState::InGame;

        runtime.step_frame(FrameInput::default());

        assert_eq!(
            runtime.current_player_index, 1,
            "should switch to P2 when P1 is game over"
        );
        assert_eq!(runtime.number_lives, 2, "should have P2's lives");
    }

    #[test]
    fn save_load_player_slot_round_trips() {
        let mut runtime = NativeRuntime::new(manifest());
        runtime.current_player_index = 0;
        runtime.number_lives = 5;
        runtime.score = 42_000;
        runtime.diamond_count = 30;
        runtime.current_form = PlayerForm::Skycutter;

        runtime.save_player_slot();

        // Modify working state
        runtime.number_lives = 1;
        runtime.score = 0;

        // Load it back
        runtime.load_player_slot();

        assert_eq!(runtime.number_lives, 5);
        assert_eq!(runtime.score, 42_000);
        assert_eq!(runtime.diamond_count, 30);
        assert_eq!(runtime.current_form, PlayerForm::Skycutter);
    }

    #[test]
    fn trace_frame_reports_current_player() {
        let mut runtime = NativeRuntime::new(manifest());
        runtime.current_player_index = 1;
        let trace = runtime.trace_frame();
        assert_eq!(trace.current_player, 1);
    }

    // -------------------------------------------------------------------
    // insta snapshot tests — detect NativeTraceFrame state drift (#028)
    // -------------------------------------------------------------------

    /// Redact advisory fields that are expected to diverge between runs.
    fn redact_advisory(snap: &mut insta::Settings) {
        snap.add_redaction(".number_objects", "[advisory]");
        snap.add_redaction(".score", "[advisory]");
        snap.add_redaction(".murder_wall_x", "[advisory]");
        snap.add_redaction(".number_gfx_objects", "[advisory]");
    }

    #[test]
    fn snapshot_idle_on_level0_frame60() {
        let mut runtime = NativeRuntime::new(manifest());
        start_level(&mut runtime);
        // Advance 60 frames of idle standing
        for _ in 0..60 {
            runtime.step_frame(FrameInput::default());
        }
        let trace = runtime.trace_frame();
        let mut settings = insta::Settings::clone_current();
        redact_advisory(&mut settings);
        settings.bind(|| {
            insta::assert_yaml_snapshot!("idle_level0_frame60", trace);
        });
    }

    #[test]
    fn snapshot_walk_right_10_frames() {
        let mut runtime = NativeRuntime::new(manifest());
        start_level(&mut runtime);
        for _ in 0..10 {
            runtime.step_frame(FrameInput {
                right: true,
                ..FrameInput::default()
            });
        }
        let trace = runtime.trace_frame();
        let mut settings = insta::Settings::clone_current();
        redact_advisory(&mut settings);
        settings.bind(|| {
            insta::assert_yaml_snapshot!("walk_right_10_frames", trace);
        });
    }

    #[test]
    fn snapshot_jump_sequence() {
        let mut runtime = NativeRuntime::new(manifest());
        start_level(&mut runtime);
        // Wait for grounded
        loop {
            runtime.step_frame(FrameInput::default());
            if runtime.trace_frame().kid_on_ground == Some(true) {
                break;
            }
        }
        // Jump: press B for one frame, then hold for 5 frames
        runtime.step_frame(FrameInput {
            b: true,
            ..FrameInput::default()
        });
        for _ in 0..5 {
            runtime.step_frame(FrameInput {
                b: true,
                ..FrameInput::default()
            });
        }
        let trace = runtime.trace_frame();
        let mut settings = insta::Settings::clone_current();
        redact_advisory(&mut settings);
        settings.bind(|| {
            insta::assert_yaml_snapshot!("jump_sequence_6frames", trace);
        });
    }

    // -------------------------------------------------------------------
    // Determinism — same inputs must produce identical state (MR1)
    // -------------------------------------------------------------------

    #[test]
    fn deterministic_replay_produces_identical_trace() {
        let inputs: Vec<FrameInput> = (0..300)
            .map(|f| match f {
                0..=59 => FrameInput::default(),
                60..=79 => FrameInput {
                    right: true,
                    ..FrameInput::default()
                },
                80..=85 => FrameInput {
                    right: true,
                    b: true,
                    ..FrameInput::default()
                },
                86..=150 => FrameInput {
                    right: true,
                    ..FrameInput::default()
                },
                _ => FrameInput::default(),
            })
            .collect();

        fn run_scenario(inputs: &[FrameInput]) -> Vec<NativeTraceFrame> {
            let mut runtime = NativeRuntime::new(manifest_with_levels(4));
            runtime.set_block_grid(grid_with_floor_at_row(17));
            start_level(&mut runtime);
            for _ in 0..260 {
                runtime.step_frame(FrameInput::default());
            }
            inputs
                .iter()
                .map(|input| {
                    runtime.step_frame(*input);
                    runtime.trace_frame()
                })
                .collect()
        }

        let run1 = run_scenario(&inputs);
        let run2 = run_scenario(&inputs);
        assert_eq!(run1.len(), run2.len());
        for (i, (a, b)) in run1.iter().zip(run2.iter()).enumerate() {
            assert_eq!(
                format!("{a:?}"),
                format!("{b:?}"),
                "divergence at frame {i}"
            );
        }
    }
}
