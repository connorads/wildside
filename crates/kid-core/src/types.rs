use crate::physics::{Direction, LevelBounds};
use crate::terrain;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// Controls the order of physics sub-steps within each frame.
/// Used for data-driven comparison of native vs ASM execution order.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PhysicsOrder {
    /// Legacy native order: gravity (update_vertical_motion) before collisions.
    GravityFirst,
    /// Experimental: position update first, then collisions, then gravity.
    PositionFirst,
    /// ASM-faithful separation matching the exact sub-step order:
    #[default]
    /// 1. update_horizontal_velocity_only()  — sub_B084: x_vel only
    /// 2. apply_jump_and_gravity_vel_only()  — sub_B168: y_vel only
    /// 3. apply_y_velocity_to_position()     — y_pos += y_vel
    /// 4. ceiling + floor collision           — sub_902A (old x_pos, new y_pos)
    /// 5. apply_x_velocity_to_position()     — x_pos += x_vel
    /// 6. wall collision                      — sub_8F26 (new x_pos)
    AsmFaithful,
}

/// Sub-frame physics trace: captures intermediate values at each sub-step
/// within `step_with_order` for transparent debugging of physics divergences.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct PhysicsTrace {
    // Pre/post horizontal motion
    pub pre_horizontal: (u32, i32),
    pub post_horizontal: (u32, i32),
    pub horizontal_step_applied: bool,

    // Pre/post vertical motion
    pub pre_vertical: (u32, i32),
    pub post_vertical: (u32, i32),

    // Gravity specifics
    pub pre_gravity_y_vel: i32,
    pub post_gravity_y_vel: i32,
    pub gravity_applied: bool,
    pub ascending_and_b_held: bool,

    // Collision results
    pub wall_hit: bool,
    pub wall_snap_x: Option<u32>,
    pub ceiling_hit: bool,
    pub floor_hit: bool,
    pub floor_y_px: Option<i32>,
    pub landing_path: Option<LandingPath>,
    pub on_ground_before: bool,
    pub on_ground_after: bool,

    // Jump
    pub jump_initiated: bool,
    pub jump_y_vel: Option<i32>,

    // Final state
    pub final_pos: (u32, u32),
    pub final_vel: (i32, i32),
}

/// Which ASM floor landing path was taken.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum LandingPath {
    /// ASM loc_AAF8: `move.w d7,y_pos(a3)` — keeps sub-pixel.
    PreserveSubPixel,
    /// ASM loc_AB22: `clr.w $20(a3)` — zeroes sub-pixel.
    ZeroSubPixel,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub(crate) enum KidFrameMotionBranch {
    #[default]
    None,
    DemoAutoTimeoutFrozen,
    DemoBootstrapPoseHold,
    DemoBootstrapCameraHandoff,
    DemoBootstrapInitialSettle,
    DemoBootstrapFreefall,
    DemoBootstrapGroundedHold,
    DemoBootstrapGroundedRun,
    TransformHibernating,
    NormalPhysics,
}

impl KidFrameMotionBranch {
    pub(crate) fn as_debug_label(self) -> &'static str {
        match self {
            Self::None => "none",
            Self::DemoAutoTimeoutFrozen => "demo_auto_timeout_frozen",
            Self::DemoBootstrapPoseHold => "demo_bootstrap_pose_hold",
            Self::DemoBootstrapCameraHandoff => "demo_bootstrap_camera_handoff",
            Self::DemoBootstrapInitialSettle => "demo_bootstrap_initial_settle",
            Self::DemoBootstrapFreefall => "demo_bootstrap_freefall",
            Self::DemoBootstrapGroundedHold => "demo_bootstrap_grounded_hold",
            Self::DemoBootstrapGroundedRun => "demo_bootstrap_grounded_run",
            Self::TransformHibernating => "transform_hibernating",
            Self::NormalPhysics => "normal_physics",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub(crate) enum KidFramePostMotionMutator {
    #[default]
    None,
    PlatformCollision,
    LiftPlatform,
    LevelTimerDeath,
    PitDeath,
    SpikeDeath,
    ProjectileDamage,
    MurderWallDamage,
    Teleporter,
    FlagTouch,
    StompBounce,
    ContactDamage,
    LevelOneOpeningHit,
    LevelOneLateHazard,
}

impl KidFramePostMotionMutator {
    pub(crate) fn as_debug_label(self) -> &'static str {
        match self {
            Self::None => "none",
            Self::PlatformCollision => "platform_collision",
            Self::LiftPlatform => "lift_platform",
            Self::LevelTimerDeath => "level_timer_death",
            Self::PitDeath => "pit_death",
            Self::SpikeDeath => "spike_death",
            Self::ProjectileDamage => "projectile_damage",
            Self::MurderWallDamage => "murder_wall_damage",
            Self::Teleporter => "teleporter",
            Self::FlagTouch => "flag_touch",
            Self::StompBounce => "stomp_bounce",
            Self::ContactDamage => "contact_damage",
            Self::LevelOneOpeningHit => "level_one_opening_hit",
            Self::LevelOneLateHazard => "level_one_late_hazard",
        }
    }
}

pub const GENERIC_ENEMY_DEATH_EFFECT_FRAMES: u16 = 10;
pub const HEADYMETAL_DEATH_EFFECT_FRAMES: u16 = 0x01C2;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SpawnProbeInspection {
    pub kid_x_pos: u32,
    pub kid_y_pos: u32,
    pub hitbox_probe: terrain::FloorProbe,
    pub span_probe: terrain::FloorProbe,
}

/// Block interaction state: evanescent, ghost, shooter, and lift blocks.
#[derive(Debug, Clone, Default)]
pub(crate) struct BlockInteractionState {
    pub(crate) evanescent_blocks: Vec<EvanescentEntry>,
    pub(crate) ghost_groups: Vec<GhostGroup>,
    pub(crate) lift_platforms: Vec<LiftPlatform>,
    /// Global shooter block fire timer: fires every 22 frames.
    pub(crate) shooter_block_timer: u16,
    /// Prize blocks headbutted from below: ASM sub_1007A owns the full
    /// lifecycle — keep the cell Solid (0x6F), run the bounce coroutine,
    /// then clear the tile and spawn the prize object.
    pub(crate) deferred_ceiling_clears: Vec<DeferredCeilingClear>,
}

/// Weather system state for the current level.
#[derive(Debug, Clone, Default)]
pub(crate) struct WeatherState {
    /// Weather type (0=none, 1=lava, 2=storm, 3=storm+hail).
    pub(crate) weather_type: u8,
    /// Hail system: frames until next hail spawn (0=spawn this frame).
    pub(crate) hail_spawn_timer: u16,
    /// Hail system: whether hail is currently active (toggles on $258-frame period).
    pub(crate) hail_active: bool,
    /// Hail system: frames until next active/inactive toggle.
    pub(crate) hail_period_timer: u16,
    /// Active lava geysers for lava weather levels.
    pub(crate) lava_geysers: Vec<LavaGeyser>,
}

/// Per-helmet ability state, reset on form change or death.
#[derive(Debug, Clone, Default)]
pub(crate) struct HelmetState {
    /// Skycutter flipboard deploy counter (0=inactive, 1..31=animating).
    pub(crate) skycutter_flipboard: u8,
    pub(crate) skycutter_on_board: bool,
    pub(crate) cyclone_flying: bool,
    pub(crate) cyclone_y_accel: i16,
    pub(crate) red_stealth_sword_timer: u8,
    pub(crate) eyeclops_beam_active: bool,
    pub(crate) maniaxe_axe_active: bool,
    pub(crate) maniaxe_turnaround: u8,
    pub(crate) iron_knight_climbing: bool,
    pub(crate) iron_knight_climb_accel: i16,
    pub(crate) berzerker_charge_accum: u8,
    pub(crate) berzerker_charging: bool,
    pub(crate) micromax_stuck: bool,
}

impl HelmetState {
    pub(crate) fn reset(&mut self) {
        *self = Self::default();
    }
}

/// Player progression: form, lives, HP, diamonds, score, and combat state.
#[derive(Debug, Clone)]
pub(crate) struct PlayerProgression {
    pub(crate) current_form: PlayerForm,
    pub(crate) pending_form: Option<PlayerForm>,
    pub(crate) number_lives: u16,
    pub(crate) number_continues: u8,
    pub(crate) number_hitpoints: u16,
    pub(crate) invulnerable_frames: usize,
    /// Visual flash countdown after form change (0=inactive).
    pub(crate) form_change_flash_frames: u8,
    pub(crate) diamond_count: u16,
    pub(crate) score: u32,
    /// Pending half-width change from form transform.
    /// ASM: during Kid_Transform, the new half-width is set via sub_B41C,
    /// and the Hibernate frame resolves any wall overlap. Non-zero = need fixup.
    pub(crate) form_width_delta: i32,
    /// Diamond Power active: grants invulnerability and clears nearby enemies.
    pub(crate) diamond_power_frames: u16,
    /// Which Diamond Power type is currently active (determines behaviour).
    pub(crate) diamond_power_type: super::helmets::DiamondPowerType,
}

impl Default for PlayerProgression {
    fn default() -> Self {
        Self {
            current_form: PlayerForm::TheKid,
            pending_form: None,
            number_lives: 3,
            number_continues: 3,
            number_hitpoints: PlayerForm::TheKid.max_hitpoints(),
            invulnerable_frames: 0,
            form_change_flash_frames: 0,
            diamond_count: 0,
            score: 0,
            diamond_power_frames: 0,
            diamond_power_type: super::helmets::DiamondPowerType::None,
            form_width_delta: 0,
        }
    }
}

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
    /// ASM game mode 0x10 — title screen demo playback.
    DemoPlay,
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
            Self::OptionsMenu => 0x0014,
            Self::LevelTransition => 0x0008,
            Self::InGame => 0x000C,
            Self::DemoPlay => 0x0010,
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

    /// Per-form collision half-width from ASM unk_B408 table (kid.asm:14307).
    /// Used by sub_8F26 (wall collision) and GfxObjects_Collision.
    pub(crate) fn half_width(self) -> i32 {
        match self {
            Self::TheKid => 5,
            Self::Skycutter => 10,
            Self::Juggernaut => 23,
            Self::Cyclone
            | Self::RedStealth
            | Self::Eyeclops
            | Self::IronKnight
            | Self::Berzerker
            | Self::Maniaxe
            | Self::Micromax => 7,
        }
    }

    /// Ceiling-collision height from ASM sub_902A/sub_DAA6.
    /// Most forms use a 31px head probe; Juggernaut and Micromax use 15px.
    pub(crate) fn ceiling_probe_height(self) -> i32 {
        match self {
            Self::Juggernaut | Self::Micromax => 15,
            _ => 31,
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
    pub(crate) fn ground_speed_table(
        self,
        terrain_type: TerrainType,
    ) -> &'static crate::physics::SpeedTable {
        match (self, terrain_type) {
            (Self::Micromax, TerrainType::Normal) => &crate::physics::NORMAL_MICROMAX,
            (Self::Micromax, TerrainType::Ice) => &crate::physics::ICE_MICROMAX,
            (Self::Micromax, TerrainType::Rubber) => &crate::physics::RUBBER_MICROMAX,
            (Self::Juggernaut, TerrainType::Normal) => &crate::physics::NORMAL_JUGGERNAUT,
            (Self::Juggernaut, TerrainType::Ice) => &crate::physics::ICE_JUGGERNAUT,
            (Self::Juggernaut, TerrainType::Rubber) => &crate::physics::RUBBER_JUGGERNAUT,
            (_, TerrainType::Normal) => &crate::physics::NORMAL_KID,
            (_, TerrainType::Ice) => &crate::physics::ICE_KID,
            (_, TerrainType::Rubber) => &crate::physics::RUBBER_KID,
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

    pub fn to_mask(&self) -> u8 {
        let mut mask = 0u8;
        if self.up {
            mask |= 1 << 0;
        }
        if self.down {
            mask |= 1 << 1;
        }
        if self.left {
            mask |= 1 << 2;
        }
        if self.right {
            mask |= 1 << 3;
        }
        if self.b {
            mask |= 1 << 4;
        }
        if self.c {
            mask |= 1 << 5;
        }
        if self.a {
            mask |= 1 << 6;
        }
        if self.start {
            mask |= 1 << 7;
        }
        mask
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NativeTraceFrame {
    pub frame_index: usize,
    pub game_mode: GameMode,
    pub game_mode_value: u16,
    pub start_press_count: usize,
    pub boot_phase: Option<String>,
    pub boot_phase_elapsed: Option<usize>,
    pub title_screen_ready: bool,
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
    pub enemy_death_effects: Vec<(u32, u32, u16)>,
    /// Form-change visual flash countdown (0=no flash).
    pub form_change_flash_frames: u8,
    /// True if a stomp occurred this frame (for SFX).
    pub stomped_this_frame: bool,
    /// Number of blocks hit by headbutt this frame (for SFX).
    pub blocks_hit_this_frame: u8,
    /// Sound test: play was requested this frame (A/C pressed).
    pub sound_test_play_pending: bool,
    /// Frontier-specific frame debug fields. Start exploratory here; promote
    /// recurring decision-making fields into typed members.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub gameplay_debug_fields: BTreeMap<String, String>,
    /// Sub-frame physics trace (populated when `trace_physics` is enabled).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub physics_trace: Option<PhysicsTrace>,
}

impl NativeTraceFrame {
    /// CRC32 of authoritative fields — used for fast divergence detection.
    /// Excludes advisory fields (number_objects, number_hitpoints, number_lives, murder_wall_x).
    pub fn authoritative_crc32(&self) -> u32 {
        let mut hasher = crc32fast::Hasher::new();
        hasher.update(&self.game_mode_value.to_le_bytes());
        hasher.update(&(self.current_level.0 as u16).to_le_bytes());
        hasher.update(&self.camera_x_pos.to_le_bytes());
        hasher.update(&self.camera_y_pos.to_le_bytes());
        hasher.update(&self.kid_x_pos.unwrap_or(0).to_le_bytes());
        hasher.update(&self.kid_y_pos.unwrap_or(0).to_le_bytes());
        // Velocity excluded: capture-timing differs between native (post-frame)
        // and oracle (VBlank), making velocity a lagging indicator. Positions
        // are the authoritative ground truth for gameplay parity.
        hasher.update(&self.current_helmet_id.to_le_bytes());
        hasher.update(&self.score.to_le_bytes());
        hasher.update(&self.diamond_count.to_le_bytes());
        hasher.update(&[self.diamond_power_active as u8]);
        hasher.update(&self.current_player.to_le_bytes());
        // player_1_level_id excluded: demo-mode timing artefact (native sets it
        // when demo level loads, oracle sets it at a different VBlank boundary).
        hasher.finalize()
    }
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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub shared_walker_state: Option<i16>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub shared_walker_patrol_timer: Option<u16>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub shared_walker_patrol_short_next: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub walk_phase: Option<u8>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub wait_timer: Option<u16>,
    /// Frontier-specific object debug fields. Start exploratory here; promote
    /// recurring decision-making fields into typed members.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub debug_fields: BTreeMap<String, String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum NativeObjectKind {
    Player,
    Enemy { object_type: u8 },
    Platform { kind: u8, subtype: u8 },
}

#[derive(Debug, Clone)]
pub(crate) struct BlockBounceAnim {
    pub(crate) cell_x: u16,
    pub(crate) cell_y: u16,
    pub(crate) remaining: u8,
}

#[derive(Debug, Clone)]
pub(crate) struct BlockDestroyEffect {
    pub(crate) x_px: u16,
    pub(crate) y_px: u16,
    pub(crate) remaining: u8,
}

/// A floating prize item spawned from a prize block hit.
/// ASM: sub_1007A runs the block-bounce coroutine, then spawns
/// Prize_Init -> PrizeDiamond_Init (kid.asm:21604-21679).
/// Item spawns at block center, falls downward with accumulating velocity,
/// collected on GfxObjects_Collision contact with player.
#[derive(Debug, Clone)]
pub(crate) struct FloatingItem {
    /// FP16.16 X position (initialized to block_center_x << 16).
    pub(crate) x_pos: u32,
    /// FP16.16 Y position (initialized to block_y << 16).
    pub(crate) y_pos: i64,
    /// FP16.16 Y velocity (accumulates by +$1388 per frame = downward).
    pub(crate) y_velocity: i32,
    /// Frames remaining (ASM: 500 = $1F4). Only counts down once active.
    pub(crate) remaining: u16,
    /// Delay frames before the item becomes active after object creation.
    /// Oracle-backed TAS probing for demo 0x1B shows the spawned prize object
    /// is already active on its first native frame, so this remains `0`.
    pub(crate) spawn_delay: u16,
    /// Source column from the level layout address used by the original
    /// object. Prize items consult this fixed column while falling.
    pub(crate) level_col: u16,
    /// Current row pointer used by the ASM prize/helmet fall routines.
    pub(crate) level_row: u16,
    /// ASM keeps a one-row grace flag so the prize can emerge from its source
    /// tile before later solid rows stop the fall.
    pub(crate) first_row_grace: bool,
    pub(crate) content: terrain::PrizeContent,
    /// Whether the player has collected this item by touching it.
    pub(crate) collected: bool,
}

#[derive(Debug, Clone)]
pub(crate) struct EnemyDeathEffect {
    pub(crate) x_pos: u32,
    pub(crate) y_pos: u32,
    pub(crate) remaining: u16,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RespawnSettleFrame {
    pub step_index: usize,
    pub kid_x_pos: u32,
    pub kid_y_pos: u32,
    pub kid_x_velocity: i32,
    pub kid_y_velocity: i32,
    pub kid_on_ground: bool,
    pub kid_on_spikes: bool,
    pub camera_x_pos: u32,
    pub camera_y_pos: u32,
    pub floor_y_pos: u32,
    pub wide_support_applied: bool,
}

/// A lift block that has been activated and is now a moving platform.
/// ASM coroutine (sub_113C0): rise 2px/frame × 8, erase block, fall 1px/frame × 16, delete.
#[derive(Debug, Clone)]
pub(crate) struct LiftPlatform {
    pub(crate) x_pos: u32,
    pub(crate) y_pos: u32,
    pub(crate) y_velocity: i32,
    /// Frames since activation.
    pub(crate) age: u16,
    /// Width in pixels (one block = 16px).
    pub(crate) width_px: u16,
    /// Grid column of the source block (for erasing after rise phase).
    pub(crate) source_col: u16,
    /// Grid row of the source block.
    pub(crate) source_row: u16,
    /// Whether the source block has been erased from the grid.
    pub(crate) block_erased: bool,
}

/// A lava geyser that erupts periodically from a fixed position.
#[derive(Debug, Clone)]
pub(crate) struct LavaGeyser {
    /// Fixed X position (pixel coordinates, not FP16.16).
    pub(crate) x_px: u16,
    /// Base Y position (bottom of eruption, pixel coordinates).
    pub(crate) y_px: u16,
    /// Eruption height in pixels.
    pub(crate) height_px: u16,
    /// Phase: 0=waiting, 1=erupting upward, 2=holding, 3=collapsing.
    pub(crate) phase: u8,
    /// Frames remaining in current phase.
    pub(crate) timer: u16,
    /// Current eruption column height in pixels (for collision/rendering).
    pub(crate) current_height: u16,
}

/// Per-player persistent state for 2P alternating mode.
/// Mirrors the ASM Player_1_*/Player_2_* RAM slots ($FFFFFC54-$FFFFFC80).
#[derive(Debug, Clone)]
pub(crate) struct PlayerSlot {
    pub(crate) lives: u16,
    pub(crate) hitpoints: u16,
    pub(crate) diamonds: u16,
    pub(crate) level_id: LevelId,
    pub(crate) form: PlayerForm,
    pub(crate) continues: u8,
    pub(crate) score: u32,
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum BootState {
    SegaScreen {
        elapsed: usize,
    },
    IntroSequence {
        segment: usize,
        elapsed: usize,
        /// Countdown for deferred Start→title skip. The original game's
        /// intro objects read Ctrl_1_Pressed (set by VBlank interrupt),
        /// giving a 2-frame delay between button press and visible state
        /// change: 1 frame for VBlank interrupt → game code, 1 frame for
        /// jmp loc_6E2 → Pal_FadeOut → next FrameRendered.
        pending_title_skip: u8,
    },
    TitleScreen {
        cursor: u8,
    },
    OptionsMenu {
        cursor: u8,
    },
    SoundTest {
        track: u8,
    },
    StartingLevel {
        remaining_frames: usize,
    },
    LevelSkipHold,
    InGame,
    /// Demo playback: auto-plays a recorded input sequence on a demo level.
    /// `frame_counter` counts VBlanks (0..1010), `data_index` indexes into the
    /// demo byte array, advancing every 2 frames.
    DemoPlay {
        frame_counter: u16,
        data_index: usize,
        /// false = Demo_InputData1 (Knights Isle), true = Demo_InputData2 (Skydragon Castle)
        use_demo_2: bool,
    },
    /// Post-final-level ending sequence: fade out, credits screens, return to title.
    EndingSequence {
        elapsed: usize,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum BlockGridSource {
    None,
    Manual,
    Generated,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct LevelSkipHoldPose {
    pub(crate) kid_x_pos: u32,
    pub(crate) kid_y_pos: u32,
    pub(crate) camera_x_pos: u32,
    pub(crate) camera_y_pos: u32,
    pub(crate) number_objects: u16,
    pub(crate) number_gfx_objects: u16,
}

#[derive(Debug, Clone)]
pub(crate) struct LevelSkipTransitionState {
    pub(crate) elapsed_frames: usize,
    pub(crate) frozen_player: PlayerState,
    pub(crate) frozen_camera: CameraState,
    pub(crate) frozen_active_objects: Vec<NativeObjectSnapshot>,
    pub(crate) frozen_number_objects: u16,
    pub(crate) frozen_number_gfx_objects: u16,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct LevelSkipAnimationPath {
    pub(crate) initial_x_pos: i32,
    pub(crate) initial_y_pos: i32,
    pub(crate) x_step_fp: i32,
    pub(crate) y_step_fp: i32,
    pub(crate) first_animation_number_objects: u16,
    pub(crate) first_animation_number_gfx_objects: u16,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub(crate) struct LevelLayoutMetadata {
    pub(crate) width_blocks: u16,
    pub(crate) height_blocks: u16,
    pub(crate) data_bytes: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum PlayerLifecycle {
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
pub(crate) struct PlayerState {
    pub(crate) x_pos: u32,
    pub(crate) y_pos: u32,
    pub(crate) x_velocity: i32,
    pub(crate) y_velocity: i32,
    pub(crate) facing: Direction,
    pub(crate) on_ground: bool,
    pub(crate) on_spikes: bool,
    pub(crate) just_jumped: bool,
    pub(crate) jump_buffer_frames: u8,
    pub(crate) right_wall_sticky: bool,
    pub(crate) left_wall_sticky: bool,
    /// Post-ledge-snap pull-up animation cooldown: suppresses all physics for N
    /// frames. ASM: stru_8BB4 = 7 anim frames × 3 VBlanks = 21 frames, driven
    /// by Hibernate_UntilAnimFinished (kid.asm:1982).
    pub(crate) ledge_snap_cooldown: u8,
    /// ASM $FFFFFA72: wall side during wall-cling. Suppresses h_accel (sub_B084)
    /// while active. Set on airborne wall hit (loc_A8DE), cleared on landing,
    /// pressing away, or damage. Some(Right) = wall on right ($FFFFFA72=0),
    /// Some(Left) = wall on left ($FFFFFA72=0xFF).
    pub(crate) wall_cling_direction: Option<Direction>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub(crate) struct CameraState {
    pub(crate) x_pos: u32,
    pub(crate) y_pos: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct LevelStartPose {
    pub(crate) kid_x_pos: u32,
    pub(crate) kid_y_pos: u32,
    pub(crate) camera_x_pos: u32,
    pub(crate) camera_y_pos: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub(crate) enum TerrainType {
    #[default]
    Normal,
    Ice,
    Rubber,
}

impl TerrainType {
    pub(crate) fn from_block_type(bt: terrain::BlockType) -> Self {
        match bt {
            terrain::BlockType::Ice => Self::Ice,
            terrain::BlockType::Rubber => Self::Rubber,
            _ => Self::Normal,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct TerrainState {
    pub(crate) floor_y_pos: u32,
    pub(crate) floor_terrain_type: TerrainType,
}

/// Movement script execution state for a platform.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct PlatformScriptState {
    /// Relative offset into the shared script table (key into `PlatformScriptTable`).
    pub(crate) script_offset: u16,
    /// Current command index within the script.
    pub(crate) command_index: usize,
    /// Remaining frames for the current command.
    pub(crate) remaining_duration: u16,
}

/// An active platform with collision dimensions and movement state.
/// Positions use the same FP16.16 format as player/enemy positions.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct ActivePlatform {
    /// FP16.16 X position (left edge of the platform).
    pub(crate) x_pos: u32,
    /// FP16.16 Y position (top edge of the platform).
    pub(crate) y_pos: u32,
    /// Platform width in pixels (from h_variant: `h * 16 + 15`).
    pub(crate) width_px: u16,
    /// Platform height in pixels (from v_variant: `v * 16 + 15`).
    pub(crate) height_px: u16,
    /// FP16.16 X velocity (from movement script).
    pub(crate) x_velocity: i32,
    /// FP16.16 Y velocity (from movement script).
    pub(crate) y_velocity: i32,
    /// Movement script state (None for stationary platforms).
    pub(crate) script: Option<PlatformScriptState>,
}

impl ActivePlatform {
    pub(crate) fn from_placement(
        placement: &kid_data::PlatformPlacement,
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
    pub(crate) fn step_script(&mut self, script_table: &kid_data::PlatformScriptTable) {
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
pub(crate) struct LevelTimer {
    /// Remaining whole seconds (3:00 = 180).
    pub(crate) seconds: u16,
    /// Frames remaining until the next second tick (counts down from 60).
    pub(crate) sub_frames: u16,
}

impl LevelTimer {
    pub(crate) fn new() -> Self {
        Self {
            seconds: super::LEVEL_TIMER_START_SECONDS,
            // ASM sets Time_SubSeconds to 1 — first frame immediately
            // decrements a second and reloads to 60.
            sub_frames: 1,
        }
    }

    /// Tick one frame. Returns true when the timer has just expired (0:00).
    pub(crate) fn tick(&mut self) -> bool {
        if self.seconds == 0 && self.sub_frames == 0 {
            return true; // already expired
        }
        self.sub_frames = self.sub_frames.saturating_sub(1);
        if self.sub_frames == 0 {
            if self.seconds > 0 {
                self.seconds -= 1;
                self.sub_frames = super::LEVEL_TIMER_FRAMES_PER_SECOND;
                false
            } else {
                true // just hit 0:00
            }
        } else {
            false
        }
    }

    pub(crate) fn add_clock_bonus(&mut self) {
        let new_seconds = self.seconds + super::LEVEL_TIMER_CLOCK_BONUS_SECONDS;
        if new_seconds >= super::LEVEL_TIMER_MAX_SECONDS {
            self.seconds = super::LEVEL_TIMER_MAX_SECONDS;
            self.sub_frames = super::LEVEL_TIMER_FRAMES_PER_SECOND;
        } else {
            self.seconds = new_seconds;
        }
    }
}

/// Tracks a triggered evanescent block that is counting down to removal.
/// The block stays solid during the countdown; once timer reaches the
/// removal threshold it is cleared from the block grid.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct EvanescentEntry {
    pub(crate) col: u16,
    pub(crate) row: u16,
    /// Timer value (starts negative from skin offset, incremented each frame).
    pub(crate) timer: i16,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum DeferredPrizeBlockPhase {
    Deleted,
    Restored,
    PendingClear,
}

/// ASM sub_1007A prize-block lifecycle: after a headbutt the block stays Solid
/// (0x6F), then briefly restores the original prize cell, then clears and
/// creates the prize object.
#[derive(Debug, Clone, Copy)]
pub(crate) struct DeferredCeilingClear {
    pub(crate) col: u16,
    pub(crate) row: u16,
    pub(crate) content: terrain::PrizeContent,
    pub(crate) restore_cell: terrain::Cell,
    pub(crate) deleted_frames_remaining: u16,
    pub(crate) phase: DeferredPrizeBlockPhase,
}

/// Runtime state for a ghost block group that cycles solid/vanished.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct GhostGroup {
    pub(crate) col: u16,
    pub(crate) row: u16,
    pub(crate) count: u8,
    pub(crate) vertical: bool,
    pub(crate) solid_time: u16,
    pub(crate) vanished_time: u16,
    pub(crate) timer: u16,
    pub(crate) solid: bool,
}

/// Projectile fired by an enemy (Archer arrow, Tank bullet, etc.).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct EnemyProjectile {
    pub(crate) x_pos: u32,
    pub(crate) y_pos: u32,
    pub(crate) x_velocity: i32,
    pub(crate) y_velocity: i32,
    /// Remaining lifetime in frames (destroyed when zero).
    pub(crate) lifetime: u16,
    /// True for player-fired projectiles (Eyeclops beam, Maniaxe axe, Juggernaut skull) — these
    /// damage enemies, not the player.
    pub(crate) friendly: bool,
    /// Per-frame gravity added to y_velocity (for arcing projectiles like Juggernaut skull).
    pub(crate) gravity: i32,
    /// Homing acceleration per axis per frame. 0 = not homing.
    /// HeadyMetal: $7D0, Lion: $2000 (Lion.asm loc_3E1C2).
    pub(crate) homing_accel: i32,
    /// Speed cap for homing projectiles (per-axis). 0 = no cap.
    /// Lion: $60000 (Lion.asm loc_3E1C2).
    pub(crate) speed_cap: i32,
    /// Bounces off terrain walls/floors/ceilings (Lion.asm loc_3E260).
    pub(crate) wall_bounce: bool,
}

/// Murder wall: an advancing wall that kills the player on contact.
/// Accelerates from zero to max_speed at $100/frame.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct MurderWallState {
    /// Current X position in fixed-point 16.16.
    pub(crate) x_pos: i64,
    /// Current speed in fixed-point 16.16.
    pub(crate) speed: i32,
    /// Terminal speed in fixed-point 16.16.
    pub(crate) max_speed: i32,
    /// True if wall moves right-to-left (reversed).
    pub(crate) reversed: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub(crate) struct PlatformLoaderState {
    pub(crate) next_index: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct ObjectSystemState {
    pub(crate) platform_loader: PlatformLoaderState,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct OpeningSliceState {
    pub(crate) player: PlayerState,
    pub(crate) camera: CameraState,
    pub(crate) terrain: TerrainState,
    pub(crate) level_bounds: LevelBounds,
    pub(crate) objects: ObjectSystemState,
    /// Prize block headbutted this frame — (content, x_px, y_px) or None.
    pub(crate) prize_block_hit: Option<(terrain::PrizeContent, u16, u16)>,
    /// Lift block activated this frame — (x_px, y_px) of block top-left in pixels.
    pub(crate) lift_block_hit: Option<(u16, u16)>,
    /// Drill block headbutted this frame — (x_fp, y_fp) centre of spawned projectile.
    pub(crate) drill_projectile_hit: Option<(u32, u32)>,
    /// Metal block bounced this frame — (cell_x, cell_y).
    pub(crate) metal_block_hit: Option<(u16, u16)>,
    /// Smash block destroyed this frame — (x_px, y_px).
    pub(crate) smash_block_hit: Option<(u16, u16)>,
    /// Teleporter block contacted this frame — (cell_x, cell_y).
    pub(crate) teleporter_contact: Option<(u16, u16)>,
    /// Prize block ceiling hit: queue the sub_1007A lifecycle for this cell.
    pub(crate) ceiling_prize_clear: Option<(u16, u16, terrain::Cell)>,
    /// Pre-gravity y_vel snapshot for floor landing sub-pixel logic.
    /// ASM tests y_vel before sub_B168 (gravity); native applies gravity first.
    pub(crate) pre_gravity_y_vel: i32,
    /// Sub-frame physics trace (populated when `trace_physics` is enabled).
    pub(crate) physics_trace: Option<PhysicsTrace>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn frame_input_mask_round_trip_all_values() {
        for mask in 0..=u8::MAX {
            let input = FrameInput::from_mask(mask);
            assert_eq!(
                input.to_mask(),
                mask,
                "round-trip failed for mask 0x{mask:02X}"
            );
        }
    }

    #[test]
    fn frame_input_to_mask_specific_buttons() {
        let input = FrameInput {
            up: true,
            down: false,
            left: true,
            right: false,
            b: false,
            c: true,
            a: false,
            start: true,
        };
        assert_eq!(input.to_mask(), 0b1010_0101);
    }

    #[test]
    fn frame_input_to_mask_empty() {
        assert_eq!(FrameInput::default().to_mask(), 0);
    }
}
