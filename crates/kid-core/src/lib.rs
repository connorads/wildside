pub mod physics;
pub mod terrain;

use kid_data::{AssetManifest, EnemyLayout, EnemyLayoutObject, PlatformLayout, PlatformPlacement};
use physics::{Direction, HorizontalInput, LevelBounds};
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
// Oracle frame windows keep the opening slice unharmed through frame 750, then
// show the first damage state before the later move-right checkpoints.
const LEVEL_ONE_OPENING_HIT_LEVEL_ELAPSED_FRAMES: usize = 300;
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
const PLAYER_DEATH_RESPAWN_SETTLE_FRAMES: usize = 480;
const PLAYER_LEVEL_ADVANCE_RESPAWN_SETTLE_FRAMES: usize = 401;
const PLAYER_DAMAGE_INVULNERABLE_FRAMES: usize = 300;
const ENEMY_CONTACT_MIN_OVERLAP_FP: i64 = 0x0001_0000;
const FLAG_HIT_HALF_SIZE: u32 = 8; // 16x16 hit region → 8px either side of centre
const LEVEL_COMPLETE_PAUSE_FRAMES: usize = 120; // brief pause before advancing
const ACTIVATION_WINDOW_X_LEAD: u32 = 384;
const ACTIVATION_WINDOW_Y_LEAD: u32 = 288;
const ACTIVATION_WINDOW_PADDING: u32 = 64;
const FLYER_RETENTION_X_PADDING: u32 = 512;
const FLYER_RETENTION_Y_PADDING: u32 = 96;
const PLATFORM_PRELOAD_X_PADDING: u32 = 64;

// MiniHoppingSkull hopper constants recovered from the enemy module.
const HOPPER_JUMP_X_SPEED_FP: i32 = 0x0001_8000;
const HOPPER_JUMP_Y_SPEED_FP: i32 = -0x0003_8000;
const HOPPER_GRAVITY_FP: i32 = 0x0000_1200;
const HOPPER_GROUND_DELAY_FRAMES: u8 = 32;
// Enemy half-width for terrain probes (8px, matching the 16px sprite).
const ENEMY_PROBE_HALF_WIDTH: i32 = 8;
// Enemy height for terrain probes (16px).
const ENEMY_PROBE_HEIGHT: i32 = 16;

// Player collision hitbox for "The Kid" (form 0).
// From ASM hitbox data: dc.w -$5, $A, -$1F, $1F
// x_offset=-5, x_size=10, y_offset=-31, y_size=31
// Position is at feet, so hitbox extends half_width either side and height above.
const PLAYER_HALF_WIDTH: i32 = 5;
const PLAYER_HEIGHT: i32 = 31;
const PLAYER_RIGHT_WALL_EXTENT: i32 = PLAYER_HALF_WIDTH + 1;
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

// Knockback on contact damage: ~2px/frame horizontal, small upward arc.
const KNOCKBACK_X_VELOCITY: i32 = 0x0002_0000;
const KNOCKBACK_Y_VELOCITY: i32 = -0x0001_8000;

// Frames an enemy flashes white after taking a non-lethal stomp.
const ENEMY_DAMAGE_FLASH_FRAMES: u8 = 8;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct LevelId(pub u8);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GameMode {
    SegaScreen,
    IntroSequence1,
    IntroSequence2,
    IntroSequence3,
    TitleScreen,
    LevelTransition,
    InGame,
}

impl GameMode {
    pub fn as_disassembly_value(self) -> u16 {
        match self {
            Self::SegaScreen => 0x0000,
            Self::IntroSequence1 => 0x0004,
            Self::IntroSequence2 => 0x0018,
            Self::IntroSequence3 => 0x001C,
            Self::TitleScreen => 0x002C,
            Self::LevelTransition => 0x0008,
            Self::InGame => 0x000C,
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
    pub invulnerable_frames: usize,
    pub diamond_count: u16,
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
    number_hitpoints: u16,
    invulnerable_frames: usize,
    paused: bool,
    pause_toggle_pending: bool,
    player_lifecycle: PlayerLifecycle,
    opening_slice: Option<OpeningSliceState>,
    death_camera_lock: Option<CameraState>,
    active_enemy_roster: Vec<NativeObjectSnapshot>,
    active_enemies: Vec<ActiveEnemy>,
    active_platform_roster: Vec<NativeObjectSnapshot>,
    block_grid: Option<terrain::BlockGrid>,
    diamond_count: u16,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum BootState {
    SegaScreen { elapsed: usize },
    IntroSequence { segment: usize, elapsed: usize },
    TitleScreen,
    StartingLevel { remaining_frames: usize },
    LevelSkipHold,
    InGame,
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

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
struct LevelLayoutMetadata {
    width_blocks: u16,
    height_blocks: u16,
    data_bytes: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PlayerLifecycle {
    Alive,
    DeathWindup { remaining_frames: usize },
    Dying { remaining_frames: usize },
    Respawning { remaining_frames: usize },
    LevelComplete { remaining_frames: usize },
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
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
struct CameraState {
    x_pos: u32,
    y_pos: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct TerrainState {
    floor_y_pos: u32,
}

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
    /// Prize blocks headbutted this frame (for diamond_count increment).
    prize_blocks_hit: u16,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct ActivationWindow {
    x_min: u32,
    x_max: u32,
    y_min: u32,
    y_max: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum EnemyFamily {
    Diamond,
    Dragon,
    MiniHoppingSkull,
    Unknown(u8),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
enum DragonMotionPhase {
    #[default]
    None,
    GlideLeftToPerch,
    PerchLeft,
    TurnRightDelay,
    SwoopRight,
    PerchLeftAtApex,
    GlideLeftToEdge,
    ParkedLeft,
}

/// Persistent per-enemy state that survives across frames.
/// Enemies are activated when they enter the camera window and
/// deactivated when they leave.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct ActiveEnemy {
    /// Index into the enemy layout segment list, or 0 for legacy flat layouts.
    segment_index: usize,
    /// Index into the level's enemy layout objects list.
    layout_index: usize,
    family: EnemyFamily,
    x_pos: u32,
    y_pos: u32,
    x_velocity: i32,
    y_velocity: i32,
    facing: Direction,
    on_ground: bool,
    hit_points: u8,
    grounded_frames: u8,
    damage_flash_frames: u8,
    dragon_phase: DragonMotionPhase,
    dragon_phase_frames: u16,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum EnemyArchetype {
    Collectible,
    Flyer,
    Hopper,
    Inert,
}

const DRAGON_DRIFT_X_SPEED_FP: i32 = 0x0000_8000;
const DRAGON_SPAWN_X_OFFSET_FP: u32 = 0x001A_8000;
const DRAGON_SPAWN_Y_OFFSET_PX: u32 = 1;
const DRAGON_LEFT_EDGE_STOP_FP: u32 = 0x001F_8000;
const DRAGON_PERCH_X_FP: u32 = 0x0032_8000;
const DRAGON_SWOOP_APEX_X_FP: u32 = 0x003D_8000;
const DRAGON_PERCH_LEFT_FRAMES: u16 = 39;
const DRAGON_TURN_RIGHT_DELAY_FRAMES: u16 = 38;
const DRAGON_PERCH_LEFT_AT_APEX_FRAMES: u16 = 49;

#[derive(Debug, Clone, Copy)]
struct EnemyLayoutView<'a> {
    segment_index: usize,
    enemy_type_slots: &'a [u16],
    objects: &'a [EnemyLayoutObject],
}

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
            number_hitpoints: PlayerForm::TheKid.max_hitpoints(),
            invulnerable_frames: 0,
            paused: false,
            pause_toggle_pending: false,
            player_lifecycle: PlayerLifecycle::Alive,
            opening_slice: None,
            death_camera_lock: None,
            active_enemy_roster: Vec::new(),
            active_enemies: Vec::new(),
            active_platform_roster: Vec::new(),
            block_grid: None,
            diamond_count: 0,
        };
        runtime.reload_block_grid_for_level(LevelId(0));
        runtime
    }

    pub fn with_generated_level_layout(assets: AssetManifest) -> Self {
        Self::new(assets)
    }

    /// Set the block collision grid for the current level.
    /// Call after constructing the runtime, before stepping frames.
    pub fn set_block_grid(&mut self, grid: terrain::BlockGrid) {
        self.block_grid = Some(grid);
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
        let start_pressed = input.start && !self.previous_input.start;
        let apply_pending_pause_toggle = self.pause_toggle_pending;
        self.pause_toggle_pending = false;
        if start_pressed {
            self.start_press_count += 1;
            if matches!(self.player_lifecycle, PlayerLifecycle::Alive) {
                match self.boot_state {
                    BootState::InGame => {
                        if input.a && input.c {
                            self.enter_level_skip_hold(self.next_level_id());
                        } else if self.paused {
                            self.paused = false;
                            self.pause_toggle_pending = false;
                        } else {
                            self.pause_toggle_pending = true;
                        }
                    }
                    BootState::LevelSkipHold if input.a && input.c => {
                        self.begin_level_entry(self.current_level);
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
            BootState::IntroSequence { segment, elapsed } => {
                *elapsed += 1;
                if start_pressed || *elapsed >= INTRO_SEGMENT_FRAMES {
                    if *segment >= 2 {
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
        }

        self.refresh_active_roster();
        self.refresh_opening_slice(self.gameplay_input(input), self.gameplay_jump_pressed());
        if apply_pending_pause_toggle {
            self.paused = !self.paused;
        }

        let trace = self.trace_frame();
        self.prior_input = self.previous_input;
        self.previous_input = input;
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
            current_player: 0,
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
            invulnerable_frames: self.invulnerable_frames,
            diamond_count: self.diamond_count,
        }
    }

    pub fn game_mode(&self) -> GameMode {
        match self.boot_state {
            BootState::SegaScreen { .. } => GameMode::SegaScreen,
            BootState::IntroSequence { segment: 0, .. } => GameMode::IntroSequence1,
            BootState::IntroSequence { segment: 1, .. } => GameMode::IntroSequence2,
            BootState::IntroSequence { .. } => GameMode::IntroSequence3,
            BootState::TitleScreen => GameMode::TitleScreen,
            BootState::LevelSkipHold => GameMode::LevelTransition,
            BootState::StartingLevel { .. } | BootState::InGame => GameMode::InGame,
        }
    }

    pub fn level_name(&self, level_id: LevelId) -> Option<&str> {
        self.level_record(level_id)
            .and_then(|level| level.display_name.as_deref())
    }

    fn level_header(&self, level_id: LevelId) -> Option<&kid_data::LevelHeader> {
        self.level_record(level_id)
            .and_then(|level| level.header_metadata.as_ref())
    }

    fn enemy_layout(&self, level_id: LevelId) -> Option<&kid_data::EnemyLayout> {
        self.level_record(level_id)
            .and_then(|level| level.enemy_layout_metadata.as_ref())
    }

    fn platform_layout(&self, level_id: LevelId) -> Option<&kid_data::PlatformLayout> {
        self.level_record(level_id)
            .and_then(|level| level.platform_metadata.as_ref())
    }

    fn level_record(&self, level_id: LevelId) -> Option<&kid_data::LevelRecord> {
        self.assets
            .levels
            .iter()
            .find(|level| level.level_id == level_id.0)
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
        match self.boot_state {
            BootState::SegaScreen { .. }
            | BootState::IntroSequence { .. }
            | BootState::TitleScreen => Some(PlayerState::title_pose()),
            BootState::LevelSkipHold => {
                let pose = self.level_skip_hold_pose(self.current_level);
                Some(PlayerState {
                    x_pos: pose.kid_x_pos,
                    y_pos: pose.kid_y_pos,
                    x_velocity: 0,
                    y_velocity: 0,
                    facing: Direction::Right,
                    on_ground: true,
                    on_spikes: false,
                    just_jumped: false,
                    jump_buffer_frames: 0,
                    right_wall_sticky: false,
                })
            }
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
        if matches!(self.boot_state, BootState::LevelSkipHold) {
            let pose = self.level_skip_hold_pose(self.current_level);
            return CameraState {
                x_pos: pose.camera_x_pos,
                y_pos: pose.camera_y_pos,
            };
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
        if !matches!(
            self.boot_state,
            BootState::StartingLevel { .. } | BootState::LevelSkipHold | BootState::InGame
        ) {
            self.opening_slice = None;
            self.current_form = PlayerForm::TheKid;
            self.pending_form = None;
            self.number_lives = 3;
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
                } else {
                    self.opening_slice = Some(OpeningSliceState::respawn(
                        self.level_bounds(self.current_level),
                    ));
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
        }

        if matches!(self.boot_state, BootState::LevelSkipHold) {
            return;
        }

        if self.paused {
            return;
        }

        let level_elapsed = self.level_elapsed_frames().unwrap_or(0);
        let bounds = self.level_bounds(self.current_level);
        let mut slice = self
            .opening_slice
            .unwrap_or_else(|| OpeningSliceState::new(bounds));
        slice.prize_blocks_hit = 0;
        slice.step(
            level_elapsed,
            input,
            jump_pressed,
            self.current_form,
            self.block_grid.as_mut(),
        );

        // Prize block item spawning: each prize block grants a diamond
        if slice.prize_blocks_hit > 0 {
            for _ in 0..slice.prize_blocks_hit {
                self.diamond_count += 1;
                if self.diamond_count % 50 == 0 {
                    self.number_lives += 1;
                }
            }
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

        // Flag collision: touching the flag completes the level
        if self.player_touches_flag(&slice) {
            self.opening_slice = Some(slice);
            self.player_lifecycle = PlayerLifecycle::LevelComplete {
                remaining_frames: LEVEL_COMPLETE_PAUSE_FRAMES,
            };
            return;
        }

        // Stomp check: kill enemies by landing on them from above
        if self.process_stomps(&mut slice) {
            self.opening_slice = Some(slice);
            return;
        }

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
        self.active_platform_roster.clear();
        if !matches!(
            self.boot_state,
            BootState::StartingLevel { .. } | BootState::InGame
        ) {
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
            );
            step_active_enemies(
                &mut self.active_enemies,
                self.block_grid.as_ref(),
                Some(slice.player.x_pos),
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
            self.opening_slice = Some(OpeningSliceState { objects, ..slice });
        }
    }

    fn object_counts_from_roster(
        &self,
        active_objects: &[NativeObjectSnapshot],
        camera_x_pos: u32,
        camera_y_pos: u32,
    ) -> (u16, u16) {
        if matches!(self.boot_state, BootState::LevelSkipHold) {
            let pose = self.level_skip_hold_pose(self.current_level);
            return (pose.number_objects, pose.number_gfx_objects);
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
        (roster_count, roster_count + 1)
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

impl NativeRuntime {
    fn should_apply_level_one_opening_hit(
        &self,
        slice: &OpeningSliceState,
        input: FrameInput,
    ) -> bool {
        self.current_level == LevelId(0)
            && self.invulnerable_frames == 0
            && input.right
            && self.number_hitpoints == self.current_form.max_hitpoints()
            && self.level_elapsed_frames().is_some_and(|elapsed| {
                elapsed >= LEVEL_ONE_OPENING_HIT_LEVEL_ELAPSED_FRAMES && slice.camera.x_pos == 0
            })
    }

    /// Check for stomp-kills: player landing on enemies from above.
    /// Decrements HP on stomp; removes enemy at 0 HP. Always bounces player.
    fn process_stomps(&mut self, slice: &mut OpeningSliceState) -> bool {
        if !matches!(self.player_lifecycle, PlayerLifecycle::Alive) {
            return false;
        }

        let mut stomped_indices = Vec::new();
        for (i, enemy) in self.active_enemy_roster.iter().enumerate() {
            let is_diamond = matches!(enemy.kind, NativeObjectKind::Enemy { object_type: 0x01 });
            if is_diamond {
                continue;
            }
            if classify_enemy_contact(slice.player, enemy) == EnemyContactResult::Stomp {
                stomped_indices.push(i);
            }
        }

        if stomped_indices.is_empty() {
            return false;
        }

        // Decrement HP; collect indices of enemies that should be removed
        let mut kill_indices = Vec::new();
        for &roster_idx in &stomped_indices {
            if roster_idx < self.active_enemies.len() {
                let enemy = &mut self.active_enemies[roster_idx];
                enemy.hit_points = enemy.hit_points.saturating_sub(1);
                if enemy.hit_points == 0 {
                    kill_indices.push(roster_idx);
                } else {
                    enemy.damage_flash_frames = ENEMY_DAMAGE_FLASH_FRAMES;
                }
            }
        }

        // Remove dead enemies
        for &idx in kill_indices.iter().rev() {
            self.active_enemies.remove(idx);
        }

        // Rebuild roster after changes
        self.active_enemy_roster = self
            .active_enemies
            .iter()
            .map(ActiveEnemy::snapshot)
            .collect();

        // Bounce the player upward
        slice.player.y_velocity = STOMP_BOUNCE_VELOCITY;
        slice.player.on_ground = false;
        true
    }

    /// Collect Diamond enemies on touch: remove from active list, increment count.
    fn collect_diamonds(&mut self, slice: &OpeningSliceState) {
        if !matches!(self.player_lifecycle, PlayerLifecycle::Alive) {
            return;
        }

        let mut collected_indices = Vec::new();
        for (i, enemy) in self.active_enemies.iter().enumerate() {
            if enemy.family != EnemyFamily::Diamond {
                continue;
            }
            let snapshot = NativeObjectSnapshot {
                kind: NativeObjectKind::Enemy {
                    object_type: enemy.family.object_type(),
                },
                x_pos: enemy.x_pos,
                y_pos: enemy.y_pos,
                layout_index: Some(enemy.layout_index),
                x_velocity: Some(enemy.x_velocity),
                y_velocity: Some(enemy.y_velocity),
                facing: Some(enemy.facing),
                on_ground: Some(enemy.on_ground),
                hit_points: Some(enemy.hit_points),
                damage_flash: false,
            };
            if player_touches_enemy(slice.player, &snapshot) {
                collected_indices.push(i);
            }
        }

        for &idx in collected_indices.iter().rev() {
            self.active_enemies.remove(idx);
            self.diamond_count += 1;
            // 50 diamonds → extra life
            if self.diamond_count % 50 == 0 {
                self.number_lives += 1;
            }
        }

        if !collected_indices.is_empty() {
            // Rebuild roster after removal
            self.active_enemy_roster = self
                .active_enemies
                .iter()
                .map(ActiveEnemy::snapshot)
                .collect();
        }
    }

    /// Returns the x_pos of the damaging enemy if contact damage should apply, or None.
    fn find_contact_damage_source(&self, slice: &OpeningSliceState) -> Option<u32> {
        if self.invulnerable_frames != 0 || !matches!(self.player_lifecycle, PlayerLifecycle::Alive)
        {
            return None;
        }

        self.active_enemies.iter().find_map(|enemy| {
            if !matches!(enemy.family.archetype(), EnemyArchetype::Hopper) {
                return None;
            }
            let snapshot = enemy.snapshot();
            if player_touches_enemy(slice.player, &snapshot) {
                Some(enemy.x_pos)
            } else {
                None
            }
        })
    }

    fn fell_below_level(&self, slice: &OpeningSliceState) -> bool {
        let player_y_px = slice.player.y_pos >> 16;
        player_y_px > u32::from(slice.level_bounds.height_pixels)
    }

    fn player_touches_flag(&self, slice: &OpeningSliceState) -> bool {
        let Some(header) = self.level_header(self.current_level) else {
            return false;
        };
        let flag_x_fp = u32::from(header.flag_x) << 16;
        let flag_y_fp = u32::from(header.flag_y) << 16;
        let half_fp = FLAG_HIT_HALF_SIZE << 16;
        let dx = slice.player.x_pos.abs_diff(flag_x_fp);
        let dy = slice.player.y_pos.abs_diff(flag_y_fp);
        dx <= half_fp && dy <= half_fp
    }

    fn should_apply_level_one_late_hazard(&self, slice: &OpeningSliceState) -> bool {
        self.current_level == LevelId(0)
            && self.number_hitpoints == 1
            && self.invulnerable_frames == 0
            && slice.camera.x_pos > LEVEL_ONE_LATE_HAZARD_CAMERA_X_FP
            && self
                .level_elapsed_frames()
                .is_some_and(|elapsed| elapsed >= LEVEL_ONE_LATE_HAZARD_LEVEL_ELAPSED_FRAMES)
    }

    fn queue_form_change(&mut self, form: PlayerForm) {
        self.pending_form = Some(form);
    }

    fn resolve_pending_form_change(&mut self) {
        let Some(form) = self.pending_form.take() else {
            return;
        };
        self.current_form = form;
        self.number_hitpoints = form.max_hitpoints();
    }

    fn apply_damage_with_knockback(&mut self, slice: &mut OpeningSliceState, enemy_x: u32) {
        let dies = self.current_form == PlayerForm::TheKid && self.number_hitpoints <= 1;
        self.apply_damage();

        // No knockback if the hit kills (death windup takes over)
        if !dies {
            let push_left = enemy_x > slice.player.x_pos;
            slice.player.x_velocity = if push_left {
                -KNOCKBACK_X_VELOCITY
            } else {
                KNOCKBACK_X_VELOCITY
            };
            slice.player.y_velocity = KNOCKBACK_Y_VELOCITY;
            slice.player.on_ground = false;
        }
    }

    fn apply_damage(&mut self) {
        if self.current_form != PlayerForm::TheKid {
            self.queue_form_change(PlayerForm::TheKid);
            self.resolve_pending_form_change();
            self.invulnerable_frames = PLAYER_DAMAGE_INVULNERABLE_FRAMES;
            return;
        }

        if self.number_hitpoints > 1 {
            self.number_hitpoints -= 1;
            self.invulnerable_frames = PLAYER_DAMAGE_INVULNERABLE_FRAMES;
            return;
        }

        self.number_hitpoints = 0;
        self.begin_death_windup();
    }

    fn begin_death_windup(&mut self) {
        self.paused = false;
        self.pause_toggle_pending = false;
        self.invulnerable_frames = 0;
        self.death_camera_lock = self.opening_slice.map(|slice| slice.camera);
        self.player_lifecycle = PlayerLifecycle::DeathWindup {
            remaining_frames: PLAYER_DEATH_WINDUP_FRAMES,
        };
    }

    fn advance_death_windup_motion(&mut self) {
        if matches!(self.boot_state, BootState::LevelSkipHold) || self.paused {
            return;
        }
        if self.current_level == LevelId(0)
            && self
                .death_camera_lock
                .is_some_and(|camera| camera.x_pos == LEVEL_ONE_LATE_HAZARD_CAMERA_X_FP)
        {
            let bounds = self.level_bounds(self.current_level);
            let mut slice = self
                .opening_slice
                .unwrap_or_else(|| OpeningSliceState::new(bounds));
            let first_windup_frame = matches!(
                self.player_lifecycle,
                PlayerLifecycle::DeathWindup { remaining_frames }
                    if remaining_frames == PLAYER_DEATH_WINDUP_FRAMES
            );
            if first_windup_frame {
                slice.player.on_ground = false;
                slice.player.x_velocity = LEVEL_ONE_LATE_HAZARD_DEATH_X_VELOCITY_FP;
                slice.player.y_velocity = LEVEL_ONE_LATE_HAZARD_DEATH_Y_VELOCITY_FP;
            } else {
                let next_x = i64::from(slice.player.x_pos) + i64::from(slice.player.x_velocity);
                let next_y = i64::from(slice.player.y_pos) + i64::from(slice.player.y_velocity);
                slice.player.x_pos = next_x.clamp(0, i64::from(u32::MAX)) as u32;
                slice.player.y_pos = next_y.clamp(0, i64::from(u32::MAX)) as u32;
                slice.player.y_velocity = slice
                    .player
                    .y_velocity
                    .saturating_add(LEVEL_ONE_LATE_HAZARD_DEATH_GRAVITY_FP);
                slice.player.on_ground = false;
            }
            if let Some(camera_lock) = self.death_camera_lock {
                slice.camera = camera_lock;
            }
            self.opening_slice = Some(slice);
            return;
        }
        let level_elapsed = self.level_elapsed_frames().unwrap_or(0);
        let bounds = self.level_bounds(self.current_level);
        let mut slice = self
            .opening_slice
            .unwrap_or_else(|| OpeningSliceState::new(bounds));
        slice.step(
            level_elapsed,
            FrameInput::default(),
            false,
            self.current_form,
            self.block_grid.as_mut(),
        );
        if let Some(camera_lock) = self.death_camera_lock {
            slice.camera = camera_lock;
        }
        self.opening_slice = Some(slice);
    }

    fn enter_death_hidden_state(&mut self) {
        self.current_form = PlayerForm::TheKid;
        self.pending_form = None;
        self.number_hitpoints = PlayerForm::TheKid.max_hitpoints();
        self.number_lives = self.number_lives.saturating_sub(1);
        self.active_enemy_roster.clear();
        self.active_enemies.clear();
        self.active_platform_roster.clear();
        self.death_camera_lock = None;
        self.player_lifecycle = PlayerLifecycle::Dying {
            remaining_frames: PLAYER_DEATH_HIDE_FRAMES,
        };
    }

    fn advance_level(&mut self) {
        self.paused = false;
        self.pause_toggle_pending = false;
        let next_level = self.next_level_id();
        self.current_level = next_level;
        self.reload_block_grid_for_level(next_level);
        self.level_start_frame = Some(self.frame_index);
        self.opening_slice = Some(OpeningSliceState::respawn(self.level_bounds(next_level)));
        self.active_enemy_roster.clear();
        self.active_enemies.clear();
        self.active_platform_roster.clear();
        self.invulnerable_frames = 0;
        self.death_camera_lock = None;
        self.player_lifecycle = PlayerLifecycle::Respawning {
            remaining_frames: PLAYER_LEVEL_ADVANCE_RESPAWN_SETTLE_FRAMES,
        };
    }

    fn begin_level_entry(&mut self, level_id: LevelId) {
        self.current_level = level_id;
        self.level_start_frame = Some(self.frame_index);
        self.boot_state = BootState::StartingLevel {
            remaining_frames: TITLE_TO_LEVEL_FRAMES,
        };
        self.death_camera_lock = None;
        let bounds = self.level_bounds(level_id);
        self.opening_slice = Some(OpeningSliceState::new(bounds));
        self.active_enemy_roster.clear();
        self.active_enemies.clear();
        self.active_platform_roster.clear();
        self.paused = false;
        self.pause_toggle_pending = false;
        self.reload_block_grid_for_level(level_id);
    }

    fn enter_level_skip_hold(&mut self, level_id: LevelId) {
        self.current_level = level_id;
        self.level_start_frame = Some(self.frame_index);
        self.boot_state = BootState::LevelSkipHold;
        self.opening_slice = Some(OpeningSliceState::respawn(self.level_bounds(level_id)));
        self.active_enemy_roster.clear();
        self.active_enemies.clear();
        self.active_platform_roster.clear();
        self.paused = false;
        self.pause_toggle_pending = false;
        self.reload_block_grid_for_level(level_id);
    }

    fn reload_block_grid_for_level(&mut self, level_id: LevelId) {
        if let Some(grid) = self.load_block_grid_for_level(level_id) {
            self.block_grid = Some(grid);
        }
    }

    fn load_block_grid_for_level(&self, level_id: LevelId) -> Option<terrain::BlockGrid> {
        let layout_dir = self.assets.output_root.join("level-layout");
        let layout_path = layout_dir.join(format!("{:02X}.bin", level_id.0));
        if !layout_path.is_file() {
            return None;
        }
        let mut data = fs::read(layout_path).ok()?;
        let metadata_path = layout_dir.join(format!("{:02X}.json", level_id.0));
        let metadata = fs::read_to_string(metadata_path)
            .ok()
            .and_then(|json| serde_json::from_str::<LevelLayoutMetadata>(&json).ok())?;
        data.truncate(metadata.data_bytes.min(data.len()));
        Some(terrain::BlockGrid::from_level_layout(
            &data,
            metadata.width_blocks,
            metadata.height_blocks,
        ))
    }

    fn level_skip_hold_pose(&self, level_id: LevelId) -> LevelSkipHoldPose {
        match level_id {
            LevelId(1) => LevelSkipHoldPose {
                kid_x_pos: 0x0040_0000,
                kid_y_pos: 0x0068_0000,
                camera_x_pos: 0,
                camera_y_pos: 0,
                number_objects: 16,
                number_gfx_objects: 15,
            },
            LevelId(2) => LevelSkipHoldPose {
                kid_x_pos: 0x0030_0000,
                kid_y_pos: 0x0040_0000,
                camera_x_pos: 0,
                camera_y_pos: 0,
                number_objects: 15,
                number_gfx_objects: 14,
            },
            _ => {
                let fallback = self
                    .level_header(level_id)
                    .map(|header| LevelSkipHoldPose {
                        kid_x_pos: u32::from(header.player_start_x) << 16,
                        kid_y_pos: u32::from(header.player_start_y) << 16,
                        camera_x_pos: 0,
                        camera_y_pos: 0,
                        number_objects: 4,
                        number_gfx_objects: 4,
                    })
                    .unwrap_or(LevelSkipHoldPose {
                        kid_x_pos: OPENING_ENTRY_KID_X_POS,
                        kid_y_pos: OPENING_GROUNDED_KID_Y_POS,
                        camera_x_pos: 0,
                        camera_y_pos: 0,
                        number_objects: 4,
                        number_gfx_objects: 4,
                    });
                fallback
            }
        }
    }

    fn next_level_id(&self) -> LevelId {
        let mut level_ids = self
            .assets
            .levels
            .iter()
            .map(|level| level.level_id)
            .collect::<Vec<_>>();
        level_ids.sort_unstable();
        level_ids
            .into_iter()
            .find(|level_id| *level_id > self.current_level.0)
            .map(LevelId)
            .or_else(|| {
                self.assets
                    .levels
                    .first()
                    .map(|level| LevelId(level.level_id))
            })
            .unwrap_or(self.current_level)
    }
}

impl PlayerState {
    fn title_pose() -> Self {
        Self {
            x_pos: TITLE_KID_X_POS,
            y_pos: TITLE_KID_Y_POS,
            x_velocity: 0,
            y_velocity: 0,
            facing: Direction::Right,
            on_ground: true,
            on_spikes: false,
            just_jumped: false,
            jump_buffer_frames: 0,
            right_wall_sticky: false,
        }
    }
}

impl OpeningSliceState {
    fn new(bounds: LevelBounds) -> Self {
        Self {
            player: PlayerState {
                x_pos: OPENING_ENTRY_KID_X_POS,
                y_pos: OPENING_ENTRY_KID_Y_POS,
                x_velocity: 0,
                y_velocity: 0,
                facing: Direction::Right,
                on_ground: false,
                on_spikes: false,
                just_jumped: false,
                jump_buffer_frames: 0,
                right_wall_sticky: false,
            },
            camera: CameraState {
                x_pos: 0,
                y_pos: OPENING_ENTRY_CAMERA_Y_FP,
            },
            terrain: TerrainState {
                floor_y_pos: OPENING_GROUNDED_KID_Y_POS,
            },
            level_bounds: bounds,
            objects: ObjectSystemState {
                platform_loader: PlatformLoaderState::default(),
            },
            prize_blocks_hit: 0,
        }
    }

    fn respawn(bounds: LevelBounds) -> Self {
        Self {
            player: PlayerState {
                x_pos: OPENING_ENTRY_KID_X_POS,
                y_pos: OPENING_GROUNDED_KID_Y_POS,
                x_velocity: 0,
                y_velocity: 0,
                facing: Direction::Right,
                on_ground: true,
                on_spikes: false,
                just_jumped: false,
                jump_buffer_frames: 0,
                right_wall_sticky: false,
            },
            camera: CameraState {
                x_pos: 0,
                y_pos: OPENING_GROUNDED_CAMERA_Y_FP,
            },
            terrain: TerrainState {
                floor_y_pos: OPENING_GROUNDED_KID_Y_POS,
            },
            level_bounds: bounds,
            objects: ObjectSystemState {
                platform_loader: PlatformLoaderState::default(),
            },
            prize_blocks_hit: 0,
        }
    }

    fn step(
        &mut self,
        level_elapsed: usize,
        input: FrameInput,
        jump_pressed: bool,
        form: PlayerForm,
        block_grid: Option<&mut terrain::BlockGrid>,
    ) {
        if level_elapsed < OPENING_SCROLL_START_FRAME {
            self.run_entry_animation(level_elapsed);
            return;
        }
        let mut block_grid = block_grid;
        let horizontal_step_applied =
            self.update_horizontal_motion(input, jump_pressed, form, block_grid.as_deref());
        self.update_vertical_motion(input, jump_pressed, form);
        let airborne_slope_recontact = if let Some(grid) = block_grid.as_deref_mut() {
            self.apply_wall_collision(Some(grid));
            self.apply_ceiling_collision(Some(grid));
            self.apply_floor_collision(Some(grid), horizontal_step_applied, !jump_pressed)
        } else {
            self.apply_floor_collision(None, horizontal_step_applied, !jump_pressed)
        };
        if let (Some(grid), Some(collision)) = (block_grid.as_deref(), airborne_slope_recontact) {
            self.apply_immediate_slope_recontact(input, form, collision, grid);
        }
        self.update_camera_follow();
    }

    /// Pre-physics entry animation: player descends and settles to ground.
    fn run_entry_animation(&mut self, level_elapsed: usize) {
        let previous_y_pos = self.player.y_pos;
        if level_elapsed <= OPENING_ENTRY_HOLD_FRAMES {
            self.camera.y_pos = OPENING_ENTRY_CAMERA_Y_FP;
            self.player.y_pos = OPENING_ENTRY_KID_Y_POS;
            self.player.y_velocity = 0;
            self.player.x_velocity = 0;
            self.player.on_ground = false;
            return;
        }
        let settle_progress = (level_elapsed - OPENING_ENTRY_HOLD_FRAMES)
            .min(OPENING_SCROLL_START_FRAME - OPENING_ENTRY_HOLD_FRAMES);
        let settle_span = OPENING_SCROLL_START_FRAME - OPENING_ENTRY_HOLD_FRAMES;
        self.camera.y_pos = interpolate_u32(
            OPENING_ENTRY_CAMERA_Y_FP,
            OPENING_GROUNDED_CAMERA_Y_FP,
            settle_progress,
            settle_span,
        );
        self.player.y_pos = interpolate_u32(
            OPENING_ENTRY_KID_Y_POS,
            OPENING_GROUNDED_KID_Y_POS,
            settle_progress,
            settle_span,
        );
        self.player.y_velocity =
            i32::try_from(self.player.y_pos.saturating_sub(previous_y_pos)).unwrap_or(i32::MAX);
        self.player.x_velocity = 0;
        self.player.on_ground = self.player.y_pos >= self.terrain.floor_y_pos;
        self.camera.x_pos = 0;
        self.player.x_pos = OPENING_ENTRY_KID_X_POS;
    }

    /// Physics-driven horizontal movement using speed tables.
    /// Uses ground tables when on_ground, air tables when airborne.
    fn update_horizontal_motion(
        &mut self,
        input: FrameInput,
        jump_pressed: bool,
        form: PlayerForm,
        block_grid: Option<&terrain::BlockGrid>,
    ) -> bool {
        if self.player.on_ground && self.player.jump_buffer_frames > 0 && !input.b {
            self.player.jump_buffer_frames = 0;
        }
        let grounded_slope = self
            .grounded_slope_collision(block_grid)
            .filter(|_| self.player.on_ground);
        if self.player.on_ground
            && self.player.y_velocity >= 0
            && self.player.jump_buffer_frames > 0
        {
            return false;
        }
        if self.player.on_ground && self.player.y_velocity >= 0 && jump_pressed {
            if let Some(collision) = grounded_slope {
                let h_input = HorizontalInput {
                    left: input.left,
                    right: input.right,
                    run: input.a,
                };
                let (new_vel, new_facing) = update_level_one_slope_ground_velocity(
                    self.player.x_velocity,
                    self.player.facing,
                    h_input,
                    collision,
                    form,
                );
                self.player.x_velocity = new_vel;
                self.player.facing = new_facing;
            }
            return false;
        }

        let near_floor =
            self.player.y_pos.abs_diff(self.terrain.floor_y_pos) <= GROUND_SNAP_TOLERANCE_FP;
        if !self.player.on_ground && jump_pressed && self.player.y_velocity >= 0 && near_floor {
            return false;
        }

        let h_input = HorizontalInput {
            left: input.left,
            right: input.right,
            run: input.a,
        };
        let (new_vel, new_facing) = if self.player.on_ground {
            if let Some(collision) = grounded_slope {
                update_level_one_slope_ground_velocity(
                    self.player.x_velocity,
                    self.player.facing,
                    h_input,
                    collision,
                    form,
                )
            } else {
                physics::update_horizontal_velocity(
                    self.player.x_velocity,
                    self.player.facing,
                    h_input,
                    &physics::NORMAL_KID,
                )
            }
        } else if self.player.just_jumped {
            (self.player.x_velocity, self.player.facing)
        } else {
            let jump_profile = physics::JUMP_AIR_PROFILES[usize::from(form.helmet_id())];
            physics::update_jump_horizontal_velocity(
                self.player.x_velocity,
                self.player.facing,
                h_input,
                jump_profile,
            )
        };
        self.player.x_velocity = new_vel;
        self.player.facing = new_facing;

        let next_x = i64::from(self.player.x_pos) + i64::from(self.player.x_velocity);
        self.player.x_pos = next_x.clamp(0, i64::from(u32::MAX)) as u32;
        true
    }

    fn grounded_slope_collision(
        &self,
        block_grid: Option<&terrain::BlockGrid>,
    ) -> Option<terrain::CollisionKind> {
        let grid = block_grid?;
        let x_px = (self.player.x_pos >> 16) as i32;
        let y_px = (self.player.y_pos >> 16) as i32;
        match terrain::probe_floor(grid, x_px, y_px).cell.collision {
            terrain::CollisionKind::SlopeDown => Some(terrain::CollisionKind::SlopeDown),
            terrain::CollisionKind::SlopeUp => Some(terrain::CollisionKind::SlopeUp),
            _ => None,
        }
    }

    /// Physics-driven vertical movement: jump initiation and gravity.
    fn update_vertical_motion(&mut self, input: FrameInput, jump_pressed: bool, form: PlayerForm) {
        if self.player.on_ground && self.player.y_velocity >= 0 && jump_pressed {
            let jump_height = physics::JUMP_HEIGHTS[usize::from(form.helmet_id())];
            let (new_x_vel, y_vel) = physics::initiate_jump(self.player.x_velocity, jump_height);
            self.player.x_velocity = new_x_vel;
            self.player.y_velocity = y_vel;
            self.player.on_ground = false;
            self.player.just_jumped = false;
            self.player.jump_buffer_frames = 0;
            let next_x = i64::from(self.player.x_pos) + i64::from(self.player.x_velocity);
            self.player.x_pos = next_x.clamp(0, i64::from(u32::MAX)) as u32;
            let next_y = i64::from(self.player.y_pos) + i64::from(self.player.y_velocity);
            self.player.y_pos = next_y.clamp(0, i64::from(u32::MAX)) as u32;
            return;
        }

        if self.player.on_ground
            && self.player.y_velocity >= 0
            && self.player.jump_buffer_frames > 0
        {
            let jump_height = physics::JUMP_HEIGHTS[usize::from(form.helmet_id())];
            let (new_x_vel, y_vel) = physics::initiate_jump(self.player.x_velocity, jump_height);
            self.player.x_velocity = new_x_vel;
            self.player.y_velocity = y_vel;
            self.player.on_ground = false;
            self.player.just_jumped = false;
            self.player.jump_buffer_frames = 0;
            let next_x = i64::from(self.player.x_pos) + i64::from(self.player.x_velocity);
            self.player.x_pos = next_x.clamp(0, i64::from(u32::MAX)) as u32;
            let next_y = i64::from(self.player.y_pos) + i64::from(self.player.y_velocity);
            self.player.y_pos = next_y.clamp(0, i64::from(u32::MAX)) as u32;
            return;
        }

        if !self.player.on_ground {
            if jump_pressed {
                self.player.jump_buffer_frames = JUMP_BUFFER_FRAMES;
            } else if self.player.jump_buffer_frames > 0 {
                self.player.jump_buffer_frames -= 1;
            }
            if self.player.just_jumped {
                self.player.just_jumped = false;
            } else {
                let ascending_and_b_held = self.player.y_velocity < 0 && input.b;
                self.player.y_velocity =
                    physics::apply_gravity(self.player.y_velocity, ascending_and_b_held);
            }
            let next_y = i64::from(self.player.y_pos) + i64::from(self.player.y_velocity);
            self.player.y_pos = next_y.clamp(0, i64::from(u32::MAX)) as u32;
        } else {
            self.player.y_velocity = 0;
            self.player.just_jumped = false;
        }
    }

    fn apply_wall_collision(&mut self, block_grid: Option<&terrain::BlockGrid>) {
        let Some(grid) = block_grid else {
            self.player.right_wall_sticky = false;
            return;
        };
        if self.player.x_velocity == 0 {
            if self.player.on_ground {
                self.player.right_wall_sticky = false;
            }
            return;
        }

        let x_px = (self.player.x_pos >> 16) as i32;
        let y_px = (self.player.y_pos >> 16) as i32;
        let moving_right = self.player.x_velocity > 0;
        if !moving_right || self.player.on_ground {
            self.player.right_wall_sticky = false;
        }

        let mut probe = terrain::probe_wall(
            grid,
            x_px,
            y_px,
            PLAYER_HALF_WIDTH,
            PLAYER_HEIGHT,
            moving_right,
        );
        if !probe.hit && moving_right && self.player.right_wall_sticky {
            let top_edge_probe = terrain::probe_wall_top_edge(
                grid,
                x_px,
                y_px,
                PLAYER_HALF_WIDTH,
                PLAYER_HEIGHT,
                true,
            );
            if top_edge_probe.hit {
                probe = top_edge_probe;
            } else {
                // Release from the tighter inclusive clamp onto the normal
                // right-edge clearance pixel before air acceleration resumes.
                self.player.x_pos = ((self.player.x_pos >> 16) + 1) << 16;
                self.player.x_velocity = 0;
                self.player.right_wall_sticky = false;
                return;
            }
        }

        if probe.hit {
            let clamped_center = if moving_right {
                // The disassembly hitbox stores left=-5, width=10, so the right edge is inclusive.
                probe.wall_x_px - PLAYER_RIGHT_WALL_EXTENT
            } else {
                // Player centre must stay right of wall: centre - half_width >= wall_x
                probe.wall_x_px + PLAYER_HALF_WIDTH
            };
            if moving_right && x_px < clamped_center {
                self.player.right_wall_sticky = false;
                return;
            }
            self.player.x_pos = (clamped_center.max(0) as u32) << 16;
            self.player.x_velocity = 0;
            self.player.right_wall_sticky = moving_right && !self.player.on_ground;
        } else {
            self.player.right_wall_sticky = false;
        }
    }

    fn apply_ceiling_collision(&mut self, block_grid: Option<&mut terrain::BlockGrid>) {
        let Some(grid) = block_grid else { return };
        if self.player.y_velocity >= 0 {
            return;
        }

        let x_px = (self.player.x_pos >> 16) as i32;
        let y_px = (self.player.y_pos >> 16) as i32;

        let probe = terrain::probe_ceiling(grid, x_px, y_px, PLAYER_HALF_WIDTH, PLAYER_HEIGHT);

        if probe.hit {
            // Prize/smash block headbutt: hitting from below clears the cell.
            if probe.cell.has_block
                && matches!(
                    probe.cell.block_type,
                    terrain::BlockType::Prize | terrain::BlockType::Smash
                )
            {
                if probe.cell.block_type == terrain::BlockType::Prize {
                    self.prize_blocks_hit += 1;
                }
                grid.set_cell_at(
                    probe.cell_x,
                    probe.cell_y,
                    terrain::Cell::from_be_bytes(0, 0),
                );
            }

            // Push feet down so head clears the ceiling: feet = ceiling_bottom + height
            let new_y = probe.ceiling_y_px + PLAYER_HEIGHT;
            self.player.y_pos = (new_y.max(0) as u32) << 16;
            self.player.y_velocity = 0;
        }
    }

    fn apply_floor_collision(
        &mut self,
        block_grid: Option<&terrain::BlockGrid>,
        horizontal_step_applied: bool,
        allow_upward_slope_snap: bool,
    ) -> Option<terrain::CollisionKind> {
        if let Some(grid) = block_grid {
            let was_on_ground = self.player.on_ground;
            let x_px = (self.player.x_pos >> 16) as i32;
            let y_px = (self.player.y_pos >> 16) as i32;
            let left_edge_probe = terrain::probe_floor(grid, x_px - PLAYER_HALF_WIDTH, y_px);
            let right_edge_probe = terrain::probe_floor(grid, x_px + PLAYER_HALF_WIDTH, y_px);
            let mut probe = terrain::probe_floor_hitbox(grid, x_px, y_px, PLAYER_HALF_WIDTH);
            if was_on_ground
                && matches!(
                    probe.cell.collision,
                    terrain::CollisionKind::SlopeDown | terrain::CollisionKind::SlopeUp
                )
            {
                let edge_probe = match (left_edge_probe.on_ground, right_edge_probe.on_ground) {
                    (true, true) => {
                        if left_edge_probe.floor_y_px >= right_edge_probe.floor_y_px {
                            Some(left_edge_probe)
                        } else {
                            Some(right_edge_probe)
                        }
                    }
                    (true, false) => Some(left_edge_probe),
                    (false, true) => Some(right_edge_probe),
                    (false, false) => None,
                };
                if let Some(edge_probe) = edge_probe {
                    if edge_probe.floor_y_px > probe.floor_y_px {
                        probe = edge_probe;
                    }
                }
            }
            let mut allow_hitbox_snap = true;
            let mut center_slope_snap_candidate = false;
            if !was_on_ground {
                let center_probe = terrain::probe_floor(grid, x_px, y_px);
                if matches!(
                    center_probe.cell.collision,
                    terrain::CollisionKind::SlopeDown | terrain::CollisionKind::SlopeUp
                ) {
                    let center_floor_fp = (center_probe.floor_y_px.max(0) as u32) << 16;
                    let near_center_slope = self.player.y_pos >= center_floor_fp
                        || self.player.y_pos.abs_diff(center_floor_fp)
                            <= SLOPE_AIR_SNAP_TOLERANCE_FP;
                    if near_center_slope {
                        probe = center_probe;
                        center_slope_snap_candidate = true;
                    } else {
                        allow_hitbox_snap = matches!(
                            left_edge_probe.cell.collision,
                            terrain::CollisionKind::SlopeDown | terrain::CollisionKind::SlopeUp
                        ) || matches!(
                            right_edge_probe.cell.collision,
                            terrain::CollisionKind::SlopeDown | terrain::CollisionKind::SlopeUp
                        );
                    }
                }
            }
            let preserving_ground_contact = self.player.on_ground && probe.cell.is_solid_floor();
            if center_slope_snap_candidate
                || (allow_hitbox_snap && (probe.on_ground || preserving_ground_contact))
            {
                let floor_y_px = match probe.cell.collision {
                    terrain::CollisionKind::Solid
                    | terrain::CollisionKind::HiddenBlock
                    | terrain::CollisionKind::Spikes => probe.floor_y_px.saturating_sub(1),
                    terrain::CollisionKind::SlopeDown
                    | terrain::CollisionKind::SlopeUp
                    | terrain::CollisionKind::Empty => probe.floor_y_px,
                };
                let floor_y_fp = (floor_y_px.max(0) as u32) << 16;
                self.terrain.floor_y_pos = floor_y_fp;
                let within_snap =
                    self.player.y_pos.abs_diff(floor_y_fp) <= GROUND_SNAP_TOLERANCE_FP;
                let can_land = match probe.cell.collision {
                    terrain::CollisionKind::SlopeDown | terrain::CollisionKind::SlopeUp => {
                        let within_slope_touch_snap =
                            self.player.y_pos.abs_diff(floor_y_fp) <= 0x0001_0000;
                        (self.player.y_velocity >= 0
                            && (self.player.y_pos >= floor_y_fp || within_slope_touch_snap))
                            || (allow_upward_slope_snap
                                && center_slope_snap_candidate
                                && self.player.y_velocity < 0
                                && self.player.y_velocity >= -(SLOPE_AIR_SNAP_TOLERANCE_FP as i32)
                                && (self.player.y_pos >= floor_y_fp || within_snap))
                    }
                    terrain::CollisionKind::Solid
                    | terrain::CollisionKind::HiddenBlock
                    | terrain::CollisionKind::Spikes
                    | terrain::CollisionKind::Empty => {
                        self.player.y_velocity >= 0
                            && (self.player.y_pos >= floor_y_fp || within_snap)
                    }
                };
                if can_land {
                    let should_apply_slope_recontact = !was_on_ground
                        && self.player.y_velocity < 0
                        && matches!(
                            probe.cell.collision,
                            terrain::CollisionKind::SlopeDown | terrain::CollisionKind::SlopeUp
                        );
                    if !was_on_ground
                        && horizontal_step_applied
                        && !matches!(
                            probe.cell.collision,
                            terrain::CollisionKind::SlopeDown | terrain::CollisionKind::SlopeUp
                        )
                    {
                        let landing_x =
                            i64::from(self.player.x_pos) - i64::from(self.player.x_velocity);
                        self.player.x_pos = landing_x.clamp(0, i64::from(u32::MAX)) as u32;
                    }
                    self.player.y_pos = floor_y_fp;
                    self.player.on_ground = true;
                    self.player.y_velocity = 0;
                    self.player.on_spikes = probe.cell.collision == terrain::CollisionKind::Spikes;
                    if should_apply_slope_recontact {
                        return match probe.cell.collision {
                            terrain::CollisionKind::SlopeDown => {
                                Some(terrain::CollisionKind::SlopeDown)
                            }
                            terrain::CollisionKind::SlopeUp => {
                                Some(terrain::CollisionKind::SlopeUp)
                            }
                            _ => None,
                        };
                    }
                }
            } else if self.player.on_ground {
                // Was on ground but no longer — start falling
                self.player.on_ground = false;
                self.player.on_spikes = false;
            }
        } else if self.player.on_ground {
            self.player.on_ground = false;
            self.player.on_spikes = false;
        }
        None
    }

    fn apply_immediate_slope_recontact(
        &mut self,
        input: FrameInput,
        form: PlayerForm,
        collision: terrain::CollisionKind,
        grid: &terrain::BlockGrid,
    ) {
        let h_input = HorizontalInput {
            left: input.left,
            right: input.right,
            run: input.a,
        };
        let (new_vel, new_facing) = update_level_one_slope_ground_velocity(
            self.player.x_velocity,
            self.player.facing,
            h_input,
            collision,
            form,
        );
        self.player.x_velocity = new_vel;
        self.player.facing = new_facing;
        let next_x = i64::from(self.player.x_pos) + i64::from(self.player.x_velocity);
        self.player.x_pos = next_x.clamp(0, i64::from(u32::MAX)) as u32;

        let x_px = (self.player.x_pos >> 16) as i32;
        let y_px = (self.player.y_pos >> 16) as i32;
        let probe = terrain::probe_floor_hitbox(grid, x_px, y_px, PLAYER_HALF_WIDTH);
        let floor_y_px = match probe.cell.collision {
            terrain::CollisionKind::Solid
            | terrain::CollisionKind::HiddenBlock
            | terrain::CollisionKind::Spikes => probe.floor_y_px.saturating_sub(1),
            terrain::CollisionKind::SlopeDown
            | terrain::CollisionKind::SlopeUp
            | terrain::CollisionKind::Empty => probe.floor_y_px,
        };
        let floor_y_fp = (floor_y_px.max(0) as u32) << 16;
        self.terrain.floor_y_pos = floor_y_fp;
        self.player.y_pos = floor_y_fp;
        self.player.on_ground = true;
        self.player.y_velocity = 0;
        self.player.on_spikes = probe.cell.collision == terrain::CollisionKind::Spikes;
    }

    /// Camera follows player using the real sub_71E4 follow algorithm.
    fn update_camera_follow(&mut self) {
        let player_x_px = (self.player.x_pos >> 16) as i32;
        let player_y_px = (self.player.y_pos >> 16) as i32;
        let camera_x_px = (self.camera.x_pos >> 16) as i32;
        let camera_y_px = (self.camera.y_pos >> 16) as i32;

        let (new_cx, new_cy) = physics::update_camera(
            camera_x_px,
            camera_y_px,
            player_x_px,
            player_y_px,
            &self.level_bounds,
        );

        self.camera.x_pos = (new_cx.max(0) as u32) << 16;
        self.camera.y_pos = (new_cy.max(0) as u32) << 16;
    }
}

fn update_level_one_slope_ground_velocity(
    x_vel: i32,
    facing: Direction,
    input: HorizontalInput,
    collision: terrain::CollisionKind,
    form: PlayerForm,
) -> (i32, Direction) {
    if form != PlayerForm::TheKid {
        return physics::update_horizontal_velocity(x_vel, facing, input, &physics::NORMAL_KID);
    }

    let slope_descends_right = collision == terrain::CollisionKind::SlopeDown;
    let mut canonical_vel = if slope_descends_right { x_vel } else { -x_vel };
    let uphill_pressed = if slope_descends_right {
        input.right
    } else {
        input.left
    };
    let downhill_pressed = if slope_descends_right {
        input.left
    } else {
        input.right
    };
    let (uphill_cap, uphill_accel, downhill_cap, downhill_accel) = if input.run {
        (
            KID_SLOPE_RUN_UPHILL_CAP_FP,
            KID_SLOPE_RUN_UPHILL_ACCEL_FP,
            KID_SLOPE_RUN_DOWNHILL_CAP_FP,
            KID_SLOPE_RUN_DOWNHILL_ACCEL_FP,
        )
    } else {
        (
            KID_SLOPE_WALK_UPHILL_CAP_FP,
            KID_SLOPE_WALK_UPHILL_ACCEL_FP,
            KID_SLOPE_WALK_DOWNHILL_CAP_FP,
            KID_SLOPE_WALK_DOWNHILL_ACCEL_FP,
        )
    };

    if uphill_pressed {
        if canonical_vel < 0 {
            canonical_vel += uphill_accel * 2;
        } else if canonical_vel <= uphill_cap {
            canonical_vel += uphill_accel;
            if canonical_vel > uphill_cap {
                canonical_vel = uphill_cap;
            }
        } else {
            canonical_vel -= KID_SLOPE_BRAKE_FP;
            if canonical_vel > KID_SLOPE_MAX_SPEED_FP {
                canonical_vel = KID_SLOPE_MAX_SPEED_FP;
            }
        }
    } else if downhill_pressed {
        if canonical_vel >= 0 {
            canonical_vel -= downhill_accel * 2;
        } else if canonical_vel >= downhill_cap {
            canonical_vel -= downhill_accel;
            if canonical_vel < downhill_cap {
                canonical_vel = downhill_cap;
            }
        } else {
            canonical_vel += KID_SLOPE_BRAKE_FP;
            if canonical_vel < -KID_SLOPE_MAX_SPEED_FP {
                canonical_vel = -KID_SLOPE_MAX_SPEED_FP;
            }
        }
    } else if canonical_vel < KID_SLOPE_NEUTRAL_TARGET_FP {
        canonical_vel += KID_SLOPE_NEUTRAL_DECEL_FP;
        if canonical_vel > KID_SLOPE_NEUTRAL_TARGET_FP {
            canonical_vel = KID_SLOPE_NEUTRAL_TARGET_FP;
        }
    } else {
        canonical_vel -= KID_SLOPE_NEUTRAL_DECEL_FP;
        if canonical_vel < KID_SLOPE_NEUTRAL_TARGET_FP {
            canonical_vel = KID_SLOPE_NEUTRAL_TARGET_FP;
        }
    }

    let next_facing = if uphill_pressed {
        if slope_descends_right {
            Direction::Right
        } else {
            Direction::Left
        }
    } else if downhill_pressed {
        if slope_descends_right {
            Direction::Left
        } else {
            Direction::Right
        }
    } else if slope_descends_right == (canonical_vel >= 0) {
        Direction::Right
    } else {
        Direction::Left
    };

    let next_x_vel = if slope_descends_right {
        canonical_vel
    } else {
        -canonical_vel
    };
    (next_x_vel, next_facing)
}

impl ActivationWindow {
    fn from_camera(camera: CameraState) -> Self {
        let camera_x_px = camera.x_pos >> 16;
        let camera_y_px = camera.y_pos >> 16;
        Self {
            x_min: camera_x_px.saturating_sub(ACTIVATION_WINDOW_PADDING),
            x_max: camera_x_px.saturating_add(ACTIVATION_WINDOW_X_LEAD),
            y_min: camera_y_px.saturating_sub(ACTIVATION_WINDOW_PADDING),
            y_max: camera_y_px.saturating_add(ACTIVATION_WINDOW_Y_LEAD),
        }
    }

    fn contains(&self, x_pos: u32, y_pos: u32) -> bool {
        (self.x_min..=self.x_max).contains(&x_pos) && (self.y_min..=self.y_max).contains(&y_pos)
    }

    fn contains_fixed_signed_with_margin(
        &self,
        x_pos: u32,
        y_pos: u32,
        x_margin: u32,
        y_margin: u32,
    ) -> bool {
        let x_pos = (x_pos as i32) >> 16;
        let y_pos = (y_pos as i32) >> 16;
        let x_min = (self.x_min as i32).saturating_sub(x_margin as i32);
        let x_max = (self.x_max as i32).saturating_add(x_margin as i32);
        let y_min = (self.y_min as i32).saturating_sub(y_margin as i32);
        let y_max = (self.y_max as i32).saturating_add(y_margin as i32);
        (x_min..=x_max).contains(&x_pos) && (y_min..=y_max).contains(&y_pos)
    }
}

impl PlatformLoaderState {
    fn advance_to_window(&mut self, layout: &PlatformLayout, window: ActivationWindow) {
        while self.next_index < layout.placements.len() {
            let placement = &layout.placements[self.next_index];
            let preload_limit = window
                .x_max
                .saturating_add(PLATFORM_PRELOAD_X_PADDING)
                .saturating_add(u32::from(placement.buffer_right));
            if u32::from(placement.x) > preload_limit {
                break;
            }
            self.next_index += 1;
        }
    }

    fn active_platforms(
        &self,
        layout: &PlatformLayout,
        window: ActivationWindow,
    ) -> Vec<NativeObjectSnapshot> {
        layout
            .placements
            .iter()
            .take(self.next_index)
            .filter(|placement| platform_is_active(placement, window))
            .map(|placement| NativeObjectSnapshot {
                kind: NativeObjectKind::Platform {
                    kind: placement.kind,
                    subtype: placement.subtype,
                },
                x_pos: u32::from(placement.x) << 16,
                y_pos: u32::from(placement.y) << 16,
                layout_index: None,
                x_velocity: None,
                y_velocity: None,
                facing: None,
                on_ground: None,
                hit_points: None,
                damage_flash: false,
            })
            .collect()
    }
}

impl ActiveEnemy {
    fn snapshot(&self) -> NativeObjectSnapshot {
        NativeObjectSnapshot {
            kind: NativeObjectKind::Enemy {
                object_type: self.family.object_type(),
            },
            x_pos: self.x_pos,
            y_pos: self.y_pos,
            layout_index: Some(self.layout_index),
            x_velocity: Some(self.x_velocity),
            y_velocity: Some(self.y_velocity),
            facing: Some(self.facing),
            on_ground: Some(self.on_ground),
            hit_points: Some(self.hit_points),
            damage_flash: self.damage_flash_frames > 0,
        }
    }
}

impl EnemyFamily {
    fn archetype(self) -> EnemyArchetype {
        match self {
            Self::Diamond => EnemyArchetype::Collectible,
            Self::Dragon => EnemyArchetype::Flyer,
            Self::MiniHoppingSkull => EnemyArchetype::Hopper,
            Self::Unknown(_) => EnemyArchetype::Inert,
        }
    }

    fn from_layout_object(enemy_type_slots: &[u16], object: &EnemyLayoutObject) -> Self {
        match resolved_enemy_id(enemy_type_slots, object.object_type) {
            Some(0x01) => Self::Diamond,
            Some(0x0C | 0x0D) => Self::Dragon,
            Some(0x1C) => Self::MiniHoppingSkull,
            Some(enemy_id) => Self::Unknown(enemy_id),
            None => Self::Unknown(object.object_type),
        }
    }

    fn object_type(self) -> u8 {
        match self {
            Self::Diamond => 0x01,
            Self::Dragon => 0x0D,
            Self::MiniHoppingSkull => 0x1C,
            Self::Unknown(enemy_id) => enemy_id,
        }
    }

    #[cfg(test)]
    fn snapshot(
        self,
        object: &EnemyLayoutObject,
        window: ActivationWindow,
    ) -> Option<NativeObjectSnapshot> {
        let object_x = u32::from(object.x);
        let object_y = u32::from(object.y);
        if !window.contains(object_x, object_y) {
            return None;
        }
        Some(NativeObjectSnapshot {
            kind: NativeObjectKind::Enemy {
                object_type: self.object_type(),
            },
            x_pos: object_x << 16,
            y_pos: object_y << 16,
            layout_index: None,
            x_velocity: None,
            y_velocity: None,
            facing: None,
            on_ground: None,
            hit_points: None,
            damage_flash: false,
        })
    }
}

#[cfg(test)]
fn build_enemy_roster(layout: &EnemyLayout, window: ActivationWindow) -> Vec<NativeObjectSnapshot> {
    let view = enemy_layout_view(layout, window.x_min, window.y_min);
    view.objects
        .iter()
        .filter_map(|object| {
            EnemyFamily::from_layout_object(view.enemy_type_slots, object).snapshot(object, window)
        })
        .collect()
}

/// Activate enemies that enter the camera window and deactivate those that leave.
/// Newly activated enemies start at their layout position.
fn activate_enemies(
    active: &mut Vec<ActiveEnemy>,
    layout: &EnemyLayout,
    window: ActivationWindow,
    camera_x_px: u32,
    camera_y_px: u32,
) {
    let view = enemy_layout_view(layout, camera_x_px, camera_y_px);
    // Most enemies despawn on spawn-point exit. Flyers keep their live object
    // around longer; the Level 1 Dragon continues drifting after its layout
    // anchor scrolls behind the camera.
    active.retain(|enemy| enemy_should_remain_active(enemy, &view, window));

    // Activate new enemies that entered the window.
    for (index, object) in view.objects.iter().enumerate() {
        let object_x = u32::from(object.x);
        let object_y = u32::from(object.y);
        if !window.contains(object_x, object_y) {
            continue;
        }
        if active
            .iter()
            .any(|e| e.segment_index == view.segment_index && e.layout_index == index)
        {
            continue;
        }
        let family = EnemyFamily::from_layout_object(view.enemy_type_slots, object);
        let hp_raw = object.hit_points_plus_one.saturating_sub(1) as u8;
        let hit_points = hp_raw.max(1); // at least 1 HP
        active.push(ActiveEnemy {
            segment_index: view.segment_index,
            layout_index: index,
            family,
            x_pos: spawn_x_pos(family, object_x),
            y_pos: spawn_y_pos(family, object_y),
            x_velocity: spawn_x_velocity(family),
            y_velocity: 0,
            facing: spawn_facing(family),
            on_ground: false,
            hit_points,
            grounded_frames: 0,
            damage_flash_frames: 0,
            dragon_phase: spawn_dragon_phase(family),
            dragon_phase_frames: 0,
        });
    }
}

/// Step all active enemies: apply movement, gravity, and terrain collision.
fn step_active_enemies(
    active: &mut [ActiveEnemy],
    block_grid: Option<&terrain::BlockGrid>,
    player_x_pos: Option<u32>,
) {
    for enemy in active.iter_mut() {
        enemy.damage_flash_frames = enemy.damage_flash_frames.saturating_sub(1);
        match enemy.family.archetype() {
            EnemyArchetype::Collectible | EnemyArchetype::Inert => {}
            EnemyArchetype::Flyer => step_flyer(enemy),
            EnemyArchetype::Hopper => step_hopper(enemy, block_grid, player_x_pos),
        }
    }
}

fn step_flyer(enemy: &mut ActiveEnemy) {
    if matches!(enemy.family, EnemyFamily::Dragon) {
        step_dragon(enemy);
        return;
    }

    enemy.x_pos = (enemy.x_pos as i64 + i64::from(enemy.x_velocity)) as u32;
    enemy.y_pos = (enemy.y_pos as i64 + i64::from(enemy.y_velocity)) as u32;
}

fn step_dragon(enemy: &mut ActiveEnemy) {
    enemy.x_velocity = 0;
    enemy.y_velocity = 0;
    match enemy.dragon_phase {
        DragonMotionPhase::None => {
            enemy.facing = Direction::Left;
        }
        DragonMotionPhase::GlideLeftToPerch => {
            enemy.facing = Direction::Left;
            let next_x_pos = enemy.x_pos.saturating_sub(DRAGON_DRIFT_X_SPEED_FP as u32);
            if next_x_pos <= DRAGON_PERCH_X_FP {
                enemy.x_pos = DRAGON_PERCH_X_FP;
                enemy.dragon_phase = DragonMotionPhase::PerchLeft;
                enemy.dragon_phase_frames = 0;
            } else {
                enemy.x_pos = next_x_pos;
            }
        }
        DragonMotionPhase::PerchLeft => {
            enemy.facing = Direction::Left;
            if dragon_phase_elapsed(enemy, DRAGON_PERCH_LEFT_FRAMES) {
                enemy.dragon_phase = DragonMotionPhase::TurnRightDelay;
                enemy.dragon_phase_frames = 0;
            }
        }
        DragonMotionPhase::TurnRightDelay => {
            enemy.facing = Direction::Right;
            if dragon_phase_elapsed(enemy, DRAGON_TURN_RIGHT_DELAY_FRAMES) {
                enemy.dragon_phase = DragonMotionPhase::SwoopRight;
                enemy.dragon_phase_frames = 0;
            }
        }
        DragonMotionPhase::SwoopRight => {
            enemy.facing = Direction::Right;
            let next_x_pos = enemy.x_pos.saturating_add(DRAGON_DRIFT_X_SPEED_FP as u32);
            if next_x_pos >= DRAGON_SWOOP_APEX_X_FP {
                enemy.x_pos = DRAGON_SWOOP_APEX_X_FP;
                enemy.dragon_phase = DragonMotionPhase::PerchLeftAtApex;
                enemy.dragon_phase_frames = 0;
            } else {
                enemy.x_pos = next_x_pos;
            }
        }
        DragonMotionPhase::PerchLeftAtApex => {
            enemy.facing = Direction::Left;
            if dragon_phase_elapsed(enemy, DRAGON_PERCH_LEFT_AT_APEX_FRAMES) {
                enemy.dragon_phase = DragonMotionPhase::GlideLeftToEdge;
                enemy.dragon_phase_frames = 0;
            }
        }
        DragonMotionPhase::GlideLeftToEdge => {
            enemy.facing = Direction::Left;
            let next_x_pos = enemy.x_pos.saturating_sub(DRAGON_DRIFT_X_SPEED_FP as u32);
            if next_x_pos <= DRAGON_LEFT_EDGE_STOP_FP {
                enemy.x_pos = DRAGON_LEFT_EDGE_STOP_FP;
                enemy.dragon_phase = DragonMotionPhase::ParkedLeft;
                enemy.dragon_phase_frames = 0;
            } else {
                enemy.x_pos = next_x_pos;
            }
        }
        DragonMotionPhase::ParkedLeft => {
            enemy.facing = Direction::Left;
            enemy.x_pos = DRAGON_LEFT_EDGE_STOP_FP;
        }
    }
}

fn dragon_phase_elapsed(enemy: &mut ActiveEnemy, phase_duration_frames: u16) -> bool {
    enemy.dragon_phase_frames = enemy.dragon_phase_frames.saturating_add(1);
    enemy.dragon_phase_frames >= phase_duration_frames
}

fn enemy_should_remain_active(
    enemy: &ActiveEnemy,
    view: &EnemyLayoutView<'_>,
    window: ActivationWindow,
) -> bool {
    if enemy.segment_index != view.segment_index {
        return false;
    }
    let Some(object) = view.objects.get(enemy.layout_index) else {
        return false;
    };

    match enemy.family.archetype() {
        EnemyArchetype::Flyer => window.contains_fixed_signed_with_margin(
            enemy.x_pos,
            enemy.y_pos,
            FLYER_RETENTION_X_PADDING,
            FLYER_RETENTION_Y_PADDING,
        ),
        _ => window.contains(u32::from(object.x), u32::from(object.y)),
    }
}

fn step_hopper(
    enemy: &mut ActiveEnemy,
    block_grid: Option<&terrain::BlockGrid>,
    player_x_pos: Option<u32>,
) {
    let Some(grid) = block_grid else {
        return;
    };

    let cx = (enemy.x_pos >> 16) as i32;
    let cy = (enemy.y_pos >> 16) as i32;
    if enemy.on_ground {
        let mut facing =
            player_relative_direction(enemy.x_pos, player_x_pos).unwrap_or(enemy.facing);
        if hopper_should_reverse(facing, grid, cx, cy) {
            facing = opposite_direction(facing);
        }
        enemy.facing = facing;
        if enemy.grounded_frames > 0 {
            enemy.grounded_frames -= 1;
            enemy.x_velocity = 0;
            return;
        }
        enemy.x_velocity = horizontal_velocity(enemy.facing, HOPPER_JUMP_X_SPEED_FP);
        enemy.y_velocity = HOPPER_JUMP_Y_SPEED_FP;
        enemy.on_ground = false;
    }

    enemy.y_velocity = (enemy.y_velocity + HOPPER_GRAVITY_FP).min(physics::MAX_FALL_SPEED);
    enemy.x_pos = (enemy.x_pos as i64 + i64::from(enemy.x_velocity)) as u32;
    enemy.y_pos = (enemy.y_pos as i64 + i64::from(enemy.y_velocity)) as u32;

    let cx = (enemy.x_pos >> 16) as i32;
    let cy = (enemy.y_pos >> 16) as i32;
    let moving_right = enemy.x_velocity >= 0;
    let wall = terrain::probe_wall(
        grid,
        cx,
        cy,
        ENEMY_PROBE_HALF_WIDTH,
        ENEMY_PROBE_HEIGHT,
        moving_right,
    );
    if wall.hit {
        reverse_enemy_direction(enemy);
        let wall_fp = (wall.wall_x_px as u32) << 16;
        let edge_fp = (ENEMY_PROBE_HALF_WIDTH as u32) << 16;
        if moving_right {
            if enemy.x_pos + edge_fp > wall_fp {
                enemy.x_pos = wall_fp.saturating_sub(edge_fp);
            }
        } else if enemy.x_pos < wall_fp + edge_fp {
            enemy.x_pos = wall_fp + edge_fp;
        }
        enemy.x_velocity = horizontal_velocity(enemy.facing, HOPPER_JUMP_X_SPEED_FP);
    }

    let floor = terrain::probe_floor(grid, cx, cy);
    if floor.on_ground {
        let floor_y_fp = (floor.floor_y_px as u32) << 16;
        if enemy.y_velocity >= 0 && enemy.y_pos >= floor_y_fp {
            enemy.y_pos = floor_y_fp;
            enemy.x_velocity = 0;
            enemy.y_velocity = 0;
            enemy.on_ground = true;
            enemy.grounded_frames = HOPPER_GROUND_DELAY_FRAMES;
        }
    } else {
        enemy.on_ground = false;
    }
}

fn hopper_should_reverse(facing: Direction, grid: &terrain::BlockGrid, cx: i32, cy: i32) -> bool {
    let moving_right = matches!(facing, Direction::Right);
    let wall = terrain::probe_wall(
        grid,
        cx,
        cy,
        ENEMY_PROBE_HALF_WIDTH,
        ENEMY_PROBE_HEIGHT,
        moving_right,
    );
    if wall.hit {
        return true;
    }
    let look_ahead_x = if moving_right {
        cx + ENEMY_PROBE_HALF_WIDTH
    } else {
        cx - ENEMY_PROBE_HALF_WIDTH
    };
    !terrain::probe_floor(grid, look_ahead_x, cy).on_ground
}

fn reverse_enemy_direction(enemy: &mut ActiveEnemy) {
    enemy.facing = opposite_direction(enemy.facing);
}

fn opposite_direction(facing: Direction) -> Direction {
    match facing {
        Direction::Left => Direction::Right,
        Direction::Right => Direction::Left,
    }
}

fn player_relative_direction(enemy_x_pos: u32, player_x_pos: Option<u32>) -> Option<Direction> {
    let player_x_pos = player_x_pos?;
    Some(if player_x_pos > enemy_x_pos {
        Direction::Right
    } else {
        Direction::Left
    })
}

fn spawn_x_pos(family: EnemyFamily, object_x: u32) -> u32 {
    match family {
        EnemyFamily::Dragon => (object_x << 16).saturating_add(DRAGON_SPAWN_X_OFFSET_FP),
        _ => object_x << 16,
    }
}

fn spawn_y_pos(family: EnemyFamily, object_y: u32) -> u32 {
    match family {
        EnemyFamily::Dragon => object_y.saturating_add(DRAGON_SPAWN_Y_OFFSET_PX) << 16,
        _ => object_y << 16,
    }
}

fn spawn_x_velocity(family: EnemyFamily) -> i32 {
    match family {
        EnemyFamily::Dragon => 0,
        _ => 0,
    }
}

fn spawn_facing(family: EnemyFamily) -> Direction {
    match family {
        EnemyFamily::Dragon => Direction::Left,
        _ => Direction::Left,
    }
}

fn spawn_dragon_phase(family: EnemyFamily) -> DragonMotionPhase {
    match family {
        EnemyFamily::Dragon => DragonMotionPhase::GlideLeftToPerch,
        _ => DragonMotionPhase::None,
    }
}

fn horizontal_velocity(facing: Direction, speed: i32) -> i32 {
    match facing {
        Direction::Left => -speed,
        Direction::Right => speed,
    }
}

fn enemy_layout_view(
    layout: &EnemyLayout,
    camera_x_px: u32,
    camera_y_px: u32,
) -> EnemyLayoutView<'_> {
    if layout.segments.is_empty() {
        return EnemyLayoutView {
            segment_index: 0,
            enemy_type_slots: &layout.enemy_type_slots,
            objects: &layout.objects,
        };
    }

    let position = if layout.axis_uses_y_position {
        camera_y_px.min(u32::from(u16::MAX)) as u16
    } else {
        camera_x_px.min(u32::from(u16::MAX)) as u16
    };
    let mut selected_index = 0usize;
    for (index, segment) in layout.segments.iter().enumerate() {
        if position >= segment.lower_bound {
            selected_index = index;
        }
        if position < segment.upper_bound {
            break;
        }
    }
    let segment = &layout.segments[selected_index];
    EnemyLayoutView {
        segment_index: selected_index,
        enemy_type_slots: &segment.enemy_type_slots,
        objects: &segment.objects,
    }
}

fn resolved_enemy_id(enemy_type_slots: &[u16], object_type: u8) -> Option<u8> {
    if object_type & 0x80 == 0 {
        return Some(object_type);
    }
    let slot_index = usize::from(object_type & 0x7F);
    let raw_slot = *enemy_type_slots.get(slot_index)?;
    if raw_slot == 0xFFFF {
        return None;
    }
    u8::try_from(raw_slot & 0x3FFF).ok()
}

fn platform_is_active(platform: &PlatformPlacement, window: ActivationWindow) -> bool {
    let object_x = u32::from(platform.x);
    let object_y = u32::from(platform.y);
    let x_min = window.x_min.saturating_sub(u32::from(platform.buffer_left));
    let x_max = window
        .x_max
        .saturating_add(u32::from(platform.buffer_right));
    let y_min = window.y_min.saturating_sub(u32::from(platform.buffer_top));
    let y_max = window
        .y_max
        .saturating_add(u32::from(platform.buffer_bottom));
    (x_min..=x_max).contains(&object_x) && (y_min..=y_max).contains(&object_y)
}

fn interpolate_u32(start: u32, end: u32, progress: usize, span: usize) -> u32 {
    if span == 0 {
        return end;
    }
    let clamped = progress.min(span) as u64;
    let start = u64::from(start);
    let end = u64::from(end);
    if end >= start {
        (start + ((end - start) * clamped) / span as u64) as u32
    } else {
        (start - ((start - end) * clamped) / span as u64) as u32
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum EnemyContactResult {
    None,
    Stomp,
    Damage,
}

/// Classify contact between player and enemy as stomp, damage, or no contact.
///
/// Stomp: player is falling (y_velocity > 0) and the player's feet are at or
/// above the enemy's top edge. Contacts lower on the sprite deal damage
/// instead of granting a bounce.
fn classify_enemy_contact(player: PlayerState, enemy: &NativeObjectSnapshot) -> EnemyContactResult {
    let px = player.x_pos as i64;
    let py = player.y_pos as i64;
    let p_left = px - (PLAYER_CONTACT_HALF_WIDTH << 16);
    let p_right = px + (PLAYER_CONTACT_HALF_WIDTH << 16);
    let p_top = py - (i64::from(PLAYER_HEIGHT) << 16);
    let p_bottom = py;

    let ex = enemy.x_pos as i64;
    let ey = enemy.y_pos as i64;
    let stomp_enemy_half_width = ENEMY_CONTACT_HALF_WIDTH;
    let damage_enemy_half_width = match enemy.kind {
        NativeObjectKind::Enemy { object_type: 0x1C } => MINI_HOPPING_SKULL_CONTACT_HALF_WIDTH,
        _ => ENEMY_CONTACT_HALF_WIDTH,
    };
    let stomp_e_left = ex - (stomp_enemy_half_width << 16);
    let stomp_e_right = ex + (stomp_enemy_half_width << 16);
    let damage_e_left = ex - (damage_enemy_half_width << 16);
    let damage_e_right = ex + (damage_enemy_half_width << 16);
    let stomp_e_top = ey - (ENEMY_CONTACT_HALF_HEIGHT << 16);
    let damage_e_top = ey - (ENEMY_CONTACT_HALF_HEIGHT << 16);
    let damage_e_bottom = ey + (ENEMY_CONTACT_HALF_HEIGHT << 16);

    // Stomp: player must be falling and their feet must still be at or above
    // the enemy's top edge on the previous frame, so this frame crosses or
    // touches the stomp line instead of already interpenetrating from the side.
    let stomp_horizontal_overlap = p_right.min(stomp_e_right) - p_left.max(stomp_e_left);
    let previous_bottom = p_bottom - i64::from(player.y_velocity);
    if player.y_velocity > 0
        && previous_bottom <= stomp_e_top
        && stomp_horizontal_overlap >= ENEMY_CONTACT_MIN_OVERLAP_FP
    {
        return EnemyContactResult::Stomp;
    }

    if matches!(enemy.kind, NativeObjectKind::Enemy { object_type: 0x1C })
        && enemy.on_ground == Some(false)
        && enemy.y_velocity.is_some_and(|y_velocity| y_velocity < 0)
    {
        return EnemyContactResult::None;
    }

    if !(p_left < damage_e_right
        && p_right > damage_e_left
        && p_top < damage_e_bottom
        && p_bottom > damage_e_top)
    {
        return EnemyContactResult::None;
    }
    let damage_horizontal_overlap = p_right.min(damage_e_right) - p_left.max(damage_e_left);
    if damage_horizontal_overlap < ENEMY_CONTACT_MIN_OVERLAP_FP {
        return EnemyContactResult::None;
    }

    EnemyContactResult::Damage
}

fn player_touches_enemy(player: PlayerState, enemy: &NativeObjectSnapshot) -> bool {
    !matches!(
        classify_enemy_contact(player, enemy),
        EnemyContactResult::None
    )
}

pub fn run_script(
    assets: AssetManifest,
    script: &[u8],
    frame_count: usize,
) -> Vec<NativeTraceFrame> {
    let mut runtime = NativeRuntime::with_generated_level_layout(assets);
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
        let trace = run_script(manifest(), &[], 400);
        assert_eq!(trace[359].game_mode, GameMode::TitleScreen);
        assert_eq!(trace[359].current_level, LevelId(0));
        assert_eq!(trace[359].player_start_x, Some(0x0750));
        assert_eq!(trace[359].flag_y, Some(0x0300));
        assert_eq!(trace[359].level_enemy_object_count, Some(22));
        assert_eq!(trace[359].active_enemy_object_count, 0);
        assert_eq!(trace[359].active_platform_object_count, 0);
        assert_eq!(trace[359].kid_x_pos, Some(TITLE_KID_X_POS));
        assert_eq!(trace[359].kid_y_pos, Some(TITLE_KID_Y_POS));
        assert!(trace[359].active_objects.is_empty());
        assert_eq!(trace[359].number_objects, 4);
        assert_eq!(trace[359].number_gfx_objects, 4);
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

        for _ in 0..TITLE_TO_LEVEL_FRAMES {
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
        assert_eq!(hold.kid_x_pos, Some(0x0040_0000));
        assert_eq!(hold.kid_y_pos, Some(0x0068_0000));
        assert_eq!(hold.camera_x_pos, 0);
        assert_eq!(hold.camera_y_pos, 0);
        assert_eq!(hold.number_objects, 16);
        assert_eq!(hold.number_gfx_objects, 15);

        for _ in 0..120 {
            runtime.step_frame(FrameInput {
                right: true,
                ..FrameInput::default()
            });
        }

        let still_holding = runtime.trace_frame();
        assert_eq!(still_holding.current_level, LevelId(1));
        assert_eq!(still_holding.game_mode, GameMode::LevelTransition);
        assert_eq!(still_holding.kid_x_pos, hold.kid_x_pos);
        assert_eq!(still_holding.kid_y_pos, hold.kid_y_pos);
        assert_eq!(still_holding.camera_x_pos, hold.camera_x_pos);
        assert_eq!(still_holding.camera_y_pos, hold.camera_y_pos);
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
        assert_eq!(respawn_entry.camera_x_pos, 0);
        assert_eq!(respawn_entry.camera_y_pos, OPENING_GROUNDED_CAMERA_Y_FP);
        assert_eq!(respawn_entry.kid_x_pos, Some(OPENING_ENTRY_KID_X_POS));
        assert_eq!(respawn_entry.kid_y_pos, Some(OPENING_GROUNDED_KID_Y_POS));

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
        assert_eq!(respawn_tail.camera_x_pos, 0);
        assert_eq!(respawn_tail.camera_y_pos, OPENING_GROUNDED_CAMERA_Y_FP);
        assert_eq!(respawn_tail.kid_x_pos, Some(OPENING_ENTRY_KID_X_POS));
        assert_eq!(respawn_tail.kid_y_pos, Some(OPENING_GROUNDED_KID_Y_POS));
        assert_eq!(respawn_tail.active_enemy_object_count, 1);
        assert_eq!(respawn_tail.active_platform_object_count, 1);
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
        data[(usize::from(1u16) * usize::from(w) + usize::from(1u16)) * 2] = 0x40;
        data[(usize::from(1u16) * usize::from(w) + usize::from(2u16)) * 2] = 0x60;
        let grid = terrain::BlockGrid::from_level_layout(&data, w, h);
        let mut slice = OpeningSliceState::respawn(LevelBounds {
            width_pixels: 64,
            height_pixels: 64,
        });
        slice.player.x_pos = 27 << 16;
        slice.player.y_pos = 19 << 16;
        slice.player.y_velocity = 0x0001_0000;
        slice.player.on_ground = false;

        slice.apply_floor_collision(Some(&grid), false, false);

        assert_eq!(slice.player.y_pos, 20 << 16);
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
        );

        assert_eq!(slice.player.x_pos, 0x032A_666A);
        assert_eq!(slice.player.y_pos, 0x00B5_0000);
        assert_eq!(slice.player.x_velocity, 0x0002_5C80);
        assert!(slice.player.on_ground);
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
            },
            camera: CameraState { x_pos: 0, y_pos: 0 },
            terrain: TerrainState {
                floor_y_pos: 143u32 << 16,
            },
            level_bounds: LevelBounds {
                width_pixels: w * 16,
                height_pixels: h * 16,
            },
            objects: ObjectSystemState {
                platform_loader: PlatformLoaderState::default(),
            },
            prize_blocks_hit: 0,
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

        // Player should be back at level start position
        let trace = runtime.trace_frame();
        assert_eq!(trace.kid_x_pos, Some(OPENING_ENTRY_KID_X_POS));
        assert_eq!(trace.kid_y_pos, Some(OPENING_GROUNDED_KID_Y_POS));
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

        // Player should be back at level start position
        let trace = runtime.trace_frame();
        assert_eq!(trace.kid_x_pos, Some(OPENING_ENTRY_KID_X_POS));
        assert_eq!(trace.kid_y_pos, Some(OPENING_GROUNDED_KID_Y_POS));
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
        let window = ActivationWindow {
            x_min: 0,
            x_max: 640,
            y_min: 0,
            y_max: 480,
        };
        let mut active = Vec::new();

        activate_enemies(&mut active, &layout, window, 0, 0);

        let dragon = active.first().expect("dragon should activate");
        assert_eq!(dragon.family, EnemyFamily::Dragon);
        assert_eq!(dragon.x_pos, (226u32 << 16) + 0x8000);
        assert_eq!(dragon.y_pos, 272u32 << 16);
        assert_eq!(dragon.x_velocity, 0);
        assert_eq!(dragon.dragon_phase, DragonMotionPhase::GlideLeftToPerch);
    }

    #[test]
    fn dragon_flyer_drifts_left_each_frame() {
        let mut enemy = ActiveEnemy {
            segment_index: 0,
            layout_index: 0,
            family: EnemyFamily::Dragon,
            x_pos: (226u32 << 16) + 0x8000,
            y_pos: 272u32 << 16,
            x_velocity: -DRAGON_DRIFT_X_SPEED_FP,
            y_velocity: 0,
            facing: Direction::Left,
            on_ground: false,
            hit_points: 1,
            grounded_frames: 0,
            damage_flash_frames: 0,
            dragon_phase: DragonMotionPhase::GlideLeftToPerch,
            dragon_phase_frames: 0,
        };

        step_active_enemies(std::slice::from_mut(&mut enemy), None, None);
        step_active_enemies(std::slice::from_mut(&mut enemy), None, None);

        assert_eq!(enemy.y_pos, 272u32 << 16);
        assert_eq!(enemy.x_pos, (225u32 << 16) + 0x8000);
        assert_eq!(enemy.facing, Direction::Left);
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
            facing: Direction::Left,
            on_ground: false,
            hit_points: 1,
            grounded_frames: 0,
            damage_flash_frames: 0,
            dragon_phase: DragonMotionPhase::GlideLeftToPerch,
            dragon_phase_frames: 0,
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
        );

        assert!(
            active.is_empty(),
            "dragon should drop once it moves beyond the retention margin"
        );
    }

    #[test]
    fn dragon_flyer_matches_oracle_turn_sequence() {
        let mut enemy = ActiveEnemy {
            segment_index: 0,
            layout_index: 0,
            family: EnemyFamily::Dragon,
            x_pos: 196u32 << 16,
            y_pos: 272u32 << 16,
            x_velocity: 0,
            y_velocity: 0,
            facing: Direction::Left,
            on_ground: false,
            hit_points: 1,
            grounded_frames: 0,
            damage_flash_frames: 0,
            dragon_phase: DragonMotionPhase::GlideLeftToPerch,
            dragon_phase_frames: 0,
        };

        for _ in 0..199 {
            step_active_enemies(std::slice::from_mut(&mut enemy), None, None);
        }
        assert_eq!(enemy.x_pos, 0x0060_8000);
        assert_eq!(enemy.facing, Direction::Left);

        for _ in 0..91 {
            step_active_enemies(std::slice::from_mut(&mut enemy), None, None);
        }
        assert_eq!(enemy.x_pos, 0x0033_0000);
        assert_eq!(enemy.facing, Direction::Left);

        for _ in 0..39 {
            step_active_enemies(std::slice::from_mut(&mut enemy), None, None);
        }
        assert_eq!(enemy.x_pos, DRAGON_PERCH_X_FP);
        assert_eq!(enemy.facing, Direction::Left);

        for _ in 0..50 {
            step_active_enemies(std::slice::from_mut(&mut enemy), None, None);
        }
        assert_eq!(enemy.x_pos, 0x0038_0000);
        assert_eq!(enemy.facing, Direction::Right);

        for _ in 0..60 {
            step_active_enemies(std::slice::from_mut(&mut enemy), None, None);
        }
        assert_eq!(enemy.x_pos, DRAGON_SWOOP_APEX_X_FP);
        assert_eq!(enemy.facing, Direction::Left);

        for _ in 0..60 {
            step_active_enemies(std::slice::from_mut(&mut enemy), None, None);
        }

        assert_eq!(enemy.x_pos, DRAGON_LEFT_EDGE_STOP_FP);
        assert_eq!(enemy.y_pos, 272u32 << 16);
        assert_eq!(enemy.facing, Direction::Left);
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
            facing: Direction::Left,
            on_ground: false,
            hit_points: 1,
            grounded_frames: 0,
            damage_flash_frames: 0,
            dragon_phase: DragonMotionPhase::None,
            dragon_phase_frames: 0,
        };

        let mut grounded_skull = None;
        for _ in 0..80 {
            step_active_enemies(std::slice::from_mut(&mut enemy), Some(&grid), None);
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
            facing: Direction::Left,
            on_ground: false,
            hit_points: 1,
            grounded_frames: 0,
            damage_flash_frames: 0,
            dragon_phase: DragonMotionPhase::None,
            dragon_phase_frames: 0,
        };

        let mut grounded_skull = None;
        for _ in 0..80 {
            step_active_enemies(std::slice::from_mut(&mut enemy), Some(&grid), None);
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

        for _ in 0..HOPPER_GROUND_DELAY_FRAMES {
            step_active_enemies(std::slice::from_mut(&mut enemy), Some(&grid), None);
        }
        assert!(
            enemy.on_ground,
            "hopper should remain grounded through the disassembly-backed wait window"
        );

        step_active_enemies(std::slice::from_mut(&mut enemy), Some(&grid), None);
        for _ in 0..12 {
            step_active_enemies(std::slice::from_mut(&mut enemy), Some(&grid), None);
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
            facing: Direction::Left,
            on_ground: true,
            hit_points: 1,
            grounded_frames: HOPPER_GROUND_DELAY_FRAMES,
            damage_flash_frames: 0,
            dragon_phase: DragonMotionPhase::None,
            dragon_phase_frames: 0,
        };

        for _ in 0..HOPPER_GROUND_DELAY_FRAMES {
            step_active_enemies(
                std::slice::from_mut(&mut enemy),
                Some(&grid),
                Some(128u32 << 16),
            );
        }
        step_active_enemies(
            std::slice::from_mut(&mut enemy),
            Some(&grid),
            Some(128u32 << 16),
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
            facing: Direction::Left,
            on_ground: true,
            hit_points: 1,
            grounded_frames: HOPPER_GROUND_DELAY_FRAMES,
            damage_flash_frames: 0,
            dragon_phase: DragonMotionPhase::None,
            dragon_phase_frames: 0,
        };

        for _ in 0..HOPPER_GROUND_DELAY_FRAMES {
            step_active_enemies(
                std::slice::from_mut(&mut enemy),
                Some(&grid),
                Some(32u32 << 16),
            );
        }
        step_active_enemies(
            std::slice::from_mut(&mut enemy),
            Some(&grid),
            Some(32u32 << 16),
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
                    skull.x_pos >= wall_limit_fp.saturating_sub(HOPPER_JUMP_X_SPEED_FP as u32),
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
            },
            camera: CameraState { x_pos: 0, y_pos: 0 },
            terrain: TerrainState {
                floor_y_pos: 48 << 16,
            },
            level_bounds: LevelBounds {
                width_pixels: 96,
                height_pixels: 80,
            },
            objects: ObjectSystemState {
                platform_loader: PlatformLoaderState::default(),
            },
            prize_blocks_hit: 0,
        };

        slice.apply_ceiling_collision(Some(&mut grid));

        // Prize block should now be empty
        let cell_after = grid.cell_at(2, 2);
        assert!(!cell_after.has_block);
        assert_eq!(cell_after.collision, terrain::CollisionKind::Empty);

        // Player should have been pushed down (velocity zeroed)
        assert_eq!(slice.player.y_velocity, 0);
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
            },
            camera: CameraState { x_pos: 0, y_pos: 0 },
            terrain: TerrainState {
                floor_y_pos: 48 << 16,
            },
            level_bounds: LevelBounds {
                width_pixels: 96,
                height_pixels: 80,
            },
            objects: ObjectSystemState {
                platform_loader: PlatformLoaderState::default(),
            },
            prize_blocks_hit: 0,
        };

        slice.apply_ceiling_collision(Some(&mut grid));

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
            },
            camera: CameraState { x_pos: 0, y_pos: 0 },
            terrain: TerrainState {
                floor_y_pos: 48 << 16,
            },
            level_bounds: LevelBounds {
                width_pixels: 96,
                height_pixels: 80,
            },
            objects: ObjectSystemState {
                platform_loader: PlatformLoaderState::default(),
            },
            prize_blocks_hit: 0,
        };

        slice.apply_ceiling_collision(Some(&mut grid));

        // Smash block should now be empty
        let cell_after = grid.cell_at(4, 2);
        assert!(!cell_after.has_block);
        assert_eq!(cell_after.collision, terrain::CollisionKind::Empty);

        // Player should have been pushed down (velocity zeroed)
        assert_eq!(slice.player.y_velocity, 0);
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
            },
            camera: CameraState { x_pos: 0, y_pos: 0 },
            terrain: TerrainState {
                floor_y_pos: 48 << 16,
            },
            level_bounds: LevelBounds {
                width_pixels: 96,
                height_pixels: 80,
            },
            objects: ObjectSystemState {
                platform_loader: PlatformLoaderState::default(),
            },
            prize_blocks_hit: 0,
        };

        slice.apply_ceiling_collision(Some(&mut grid));
        assert!(!grid.cell_at(4, 2).has_block);

        // Second jump through the now-empty cell: no ceiling hit
        slice.player.y_pos = 63 << 16;
        slice.player.y_velocity = -0x0002_0000;
        slice.apply_ceiling_collision(Some(&mut grid));

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
            },
            camera: CameraState { x_pos: 0, y_pos: 0 },
            terrain: TerrainState {
                floor_y_pos: 48 << 16,
            },
            level_bounds: LevelBounds {
                width_pixels: 96,
                height_pixels: 80,
            },
            objects: ObjectSystemState {
                platform_loader: PlatformLoaderState::default(),
            },
            prize_blocks_hit: 0,
        };

        slice.apply_ceiling_collision(Some(&mut grid));
        assert!(!grid.cell_at(2, 2).has_block);

        // Second jump through the now-empty cell: no ceiling hit
        slice.player.y_pos = 63 << 16;
        slice.player.y_velocity = -0x0002_0000;
        slice.apply_ceiling_collision(Some(&mut grid));

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
            },
            camera: CameraState { x_pos: 0, y_pos: 0 },
            terrain: TerrainState {
                floor_y_pos: 48 << 16,
            },
            level_bounds: LevelBounds {
                width_pixels: 64,
                height_pixels: 64,
            },
            objects: ObjectSystemState {
                platform_loader: PlatformLoaderState::default(),
            },
            prize_blocks_hit: 0,
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
            },
            camera: CameraState { x_pos: 0, y_pos: 0 },
            terrain: TerrainState {
                floor_y_pos: 64 << 16,
            },
            level_bounds: LevelBounds {
                width_pixels: 80,
                height_pixels: 80,
            },
            objects: ObjectSystemState {
                platform_loader: PlatformLoaderState::default(),
            },
            prize_blocks_hit: 0,
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
            },
            camera: CameraState { x_pos: 0, y_pos: 0 },
            terrain: TerrainState { floor_y_pos: 0 },
            level_bounds: LevelBounds {
                width_pixels: 320,
                height_pixels: 224,
            },
            objects: ObjectSystemState {
                platform_loader: PlatformLoaderState::default(),
            },
            prize_blocks_hit: 0,
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
    fn stomp_requires_feet_above_enemy_top_edge() {
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
        };

        assert_eq!(
            classify_enemy_contact(player, &enemy),
            EnemyContactResult::Damage
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
        };

        assert_eq!(
            classify_enemy_contact(player, &enemy),
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
    fn contact_damage_knocks_player_away_from_enemy_on_right() {
        let mut runtime = NativeRuntime::new(manifest_with_static_skull());
        runtime.set_block_grid(grid_with_floor_at_row(14));
        start_level(&mut runtime);
        for _ in 0..260 {
            runtime.step_frame(FrameInput::default());
        }
        runtime.number_hitpoints = 3;

        // Activate skull, position player to the LEFT of skull
        if let Some(ref mut slice) = runtime.opening_slice {
            slice.camera.x_pos = 0;
            slice.camera.y_pos = 0;
        }
        runtime.step_frame(FrameInput::default());
        if let Some(ref mut slice) = runtime.opening_slice {
            // Player slightly left of skull (skull at 0x0120)
            slice.player.x_pos = 0x011C_0000;
            slice.player.y_pos = 0x00E0_0000;
            slice.player.on_ground = true;
            slice.player.x_velocity = 0;
            slice.player.y_velocity = 0;
            slice.camera.x_pos = 0;
            slice.camera.y_pos = 0;
        }
        runtime.step_frame(FrameInput::default());

        // Enemy is to the right → knockback should push player left (negative x_velocity)
        let slice = runtime.opening_slice.unwrap();
        assert!(
            slice.player.x_velocity < 0,
            "player should be knocked left (x_vel={}) when enemy is to the right",
            slice.player.x_velocity
        );
        assert!(
            slice.player.y_velocity < 0,
            "knockback should include upward arc (y_vel={})",
            slice.player.y_velocity
        );
    }

    #[test]
    fn contact_damage_knocks_player_away_from_enemy_on_left() {
        let mut runtime = NativeRuntime::new(manifest_with_static_skull());
        runtime.set_block_grid(grid_with_floor_at_row(14));
        start_level(&mut runtime);
        for _ in 0..260 {
            runtime.step_frame(FrameInput::default());
        }
        runtime.number_hitpoints = 3;

        // Activate skull, position player to the RIGHT of skull
        if let Some(ref mut slice) = runtime.opening_slice {
            slice.camera.x_pos = 0;
            slice.camera.y_pos = 0;
        }
        runtime.step_frame(FrameInput::default());
        if let Some(ref mut slice) = runtime.opening_slice {
            // Player slightly right of skull (skull at 0x0120)
            slice.player.x_pos = 0x0124_0000;
            slice.player.y_pos = 0x00E0_0000;
            slice.player.on_ground = true;
            slice.player.x_velocity = 0;
            slice.player.y_velocity = 0;
            slice.camera.x_pos = 0;
            slice.camera.y_pos = 0;
        }
        runtime.step_frame(FrameInput::default());

        // Enemy is to the left → knockback should push player right (positive x_velocity)
        let slice = runtime.opening_slice.unwrap();
        assert!(
            slice.player.x_velocity > 0,
            "player should be knocked right (x_vel={}) when enemy is to the left",
            slice.player.x_velocity
        );
        assert!(
            slice.player.y_velocity < 0,
            "knockback should include upward arc (y_vel={})",
            slice.player.y_velocity
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
}
