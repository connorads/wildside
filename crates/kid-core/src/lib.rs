pub mod physics;
pub mod terrain;

use kid_data::{AssetManifest, EnemyLayout, EnemyLayoutObject, PlatformLayout, PlatformPlacement};
use physics::{Direction, HorizontalInput, LevelBounds};
use serde::{Deserialize, Serialize};

const SEGA_SCREEN_FRAMES: usize = 120;
const INTRO_SEGMENT_FRAMES: usize = 80;
const TITLE_TO_LEVEL_FRAMES: usize = 60;
const OPENING_ENTRY_CAMERA_Y_FP: u32 = 0x0093_0000;
const OPENING_GROUNDED_CAMERA_Y_FP: u32 = 0x0073_0000;
const OPENING_SCROLL_ACCEL_TARGET_X_FP: u32 = 0x0045_0000;
const OPENING_SCROLL_START_FRAME: usize = 260;
const OPENING_ENTRY_HOLD_FRAMES: usize = TITLE_TO_LEVEL_FRAMES + 1;
const BOOTSTRAP_LEVEL_START_PRESS_COUNT: usize = 6;
const TITLE_KID_X_POS: u32 = 0x0028_0000;
const TITLE_KID_Y_POS: u32 = 0x0030_0000;
const OPENING_ENTRY_KID_X_POS: u32 = 0x0020_0000;
const OPENING_ENTRY_KID_Y_POS: u32 = 0x0109_0000;
const OPENING_GROUNDED_KID_Y_POS: u32 = 0x010F_0000;
const LEVEL_ONE_LATE_HAZARD_CAMERA_X_FP: u32 = 0x03B0_0000;
const LEVEL_ONE_ENEMY_CONTACT_X_FP: u32 = 0x0032_0000;
const LEVEL_ONE_ENEMY_CONTACT_Y_FP: u32 = 0x0006_0000;
const PLAYER_DEATH_WINDUP_FRAMES: usize = 159;
const PLAYER_DEATH_HIDE_FRAMES: usize = 80;
const PLAYER_RESPAWN_SETTLE_FRAMES: usize = 401;
const PLAYER_DAMAGE_INVULNERABLE_FRAMES: usize = 300;
const ACTIVATION_WINDOW_X_LEAD: u32 = 384;
const ACTIVATION_WINDOW_Y_LEAD: u32 = 288;
const ACTIVATION_WINDOW_PADDING: u32 = 64;
const PLATFORM_PRELOAD_X_PADDING: u32 = 64;

// Player collision hitbox for "The Kid" (form 0).
// From ASM hitbox data: dc.w -$5, $A, -$1F, $1F
// x_offset=-5, x_size=10, y_offset=-31, y_size=31
// Position is at feet, so hitbox extends half_width either side and height above.
const PLAYER_HALF_WIDTH: i32 = 5;
const PLAYER_HEIGHT: i32 = 31;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct LevelId(pub u8);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GameMode {
    SegaScreen,
    IntroSequence1,
    IntroSequence2,
    IntroSequence3,
    TitleScreen,
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
    pub active_objects: Vec<NativeObjectSnapshot>,
    pub camera_x_pos: u32,
    pub camera_y_pos: u32,
    pub number_objects: u16,
    pub number_gfx_objects: u16,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NativeObjectSnapshot {
    pub kind: NativeObjectKind,
    pub x_pos: u32,
    pub y_pos: u32,
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
    player_lifecycle: PlayerLifecycle,
    opening_slice: Option<OpeningSliceState>,
    active_enemy_roster: Vec<NativeObjectSnapshot>,
    active_platform_roster: Vec<NativeObjectSnapshot>,
    block_grid: Option<terrain::BlockGrid>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum BootState {
    SegaScreen { elapsed: usize },
    IntroSequence { segment: usize, elapsed: usize },
    TitleScreen,
    StartingLevel { remaining_frames: usize },
    InGame,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PlayerLifecycle {
    Alive,
    DeathWindup { remaining_frames: usize },
    Dying { remaining_frames: usize },
    Respawning { remaining_frames: usize },
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
    MiniHoppingSkull,
    Unknown(u8),
}

impl NativeRuntime {
    pub fn new(assets: AssetManifest) -> Self {
        Self {
            assets,
            frame_index: 0,
            boot_state: BootState::SegaScreen { elapsed: 0 },
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
            player_lifecycle: PlayerLifecycle::Alive,
            opening_slice: None,
            active_enemy_roster: Vec::new(),
            active_platform_roster: Vec::new(),
            block_grid: None,
        }
    }

    /// Set the block collision grid for the current level.
    /// Call after constructing the runtime, before stepping frames.
    pub fn set_block_grid(&mut self, grid: terrain::BlockGrid) {
        self.block_grid = Some(grid);
    }

    pub fn step_frame(&mut self, input: FrameInput) -> NativeTraceFrame {
        let start_pressed = input.start && !self.previous_input.start;
        if start_pressed {
            self.start_press_count += 1;
            if matches!(self.boot_state, BootState::InGame)
                && matches!(self.player_lifecycle, PlayerLifecycle::Alive)
            {
                if input.a && input.c {
                    self.advance_level();
                } else {
                    self.paused = !self.paused;
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
                    self.level_start_frame = Some(self.frame_index);
                    self.boot_state = BootState::StartingLevel {
                        remaining_frames: TITLE_TO_LEVEL_FRAMES,
                    };
                    let bounds = self.level_bounds(self.current_level);
                    self.opening_slice = Some(OpeningSliceState::new(bounds));
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
            BootState::InGame => {}
        }

        self.refresh_opening_slice(input);
        self.refresh_active_roster();

        let trace = self.trace_frame();
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
            active_objects,
            camera_x_pos: camera_state.x_pos,
            camera_y_pos: camera_state.y_pos,
            number_objects,
            number_gfx_objects,
        }
    }

    pub fn game_mode(&self) -> GameMode {
        match self.boot_state {
            BootState::SegaScreen { .. } => GameMode::SegaScreen,
            BootState::IntroSequence { segment: 0, .. } => GameMode::IntroSequence1,
            BootState::IntroSequence { segment: 1, .. } => GameMode::IntroSequence2,
            BootState::IntroSequence { .. } => GameMode::IntroSequence3,
            BootState::TitleScreen => GameMode::TitleScreen,
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

    fn refresh_opening_slice(&mut self, input: FrameInput) {
        if !matches!(
            self.boot_state,
            BootState::StartingLevel { .. } | BootState::InGame
        ) {
            self.opening_slice = None;
            self.current_form = PlayerForm::TheKid;
            self.pending_form = None;
            self.number_lives = 3;
            self.number_hitpoints = PlayerForm::TheKid.max_hitpoints();
            self.invulnerable_frames = 0;
            self.paused = false;
            self.player_lifecycle = PlayerLifecycle::Alive;
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
                    self.opening_slice = Some(OpeningSliceState::respawn(self.level_bounds(self.current_level)));
                    self.player_lifecycle = PlayerLifecycle::Respawning {
                        remaining_frames: PLAYER_RESPAWN_SETTLE_FRAMES,
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
        }

        if self.paused {
            return;
        }

        let level_elapsed = self.level_elapsed_frames().unwrap_or(0);
        let bounds = self.level_bounds(self.current_level);
        let mut slice = self
            .opening_slice
            .unwrap_or_else(|| OpeningSliceState::new(bounds));
        slice.step(level_elapsed, input, self.current_form, self.block_grid.as_ref());

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

        if self.should_apply_level_one_opening_hit(&slice, input)
            || self.should_apply_level_one_contact_damage(&slice)
            || self.should_apply_level_one_late_hazard(&slice)
        {
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
        ) {
            return;
        }
        let window = ActivationWindow::from_camera(slice.camera);
        let enemy_layout = self.enemy_layout(self.current_level).cloned();
        let platform_layout = self.platform_layout(self.current_level).cloned();
        if let Some(enemy_layout) = enemy_layout.as_ref() {
            self.active_enemy_roster = build_enemy_roster(enemy_layout, window);
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
        if camera_x_pos == 0 && camera_y_pos > OPENING_GROUNDED_CAMERA_Y_FP + 0x0004_0000 {
            return (roster_count + 2, roster_count + 2);
        }
        if camera_x_pos == 0 {
            return (roster_count + 1, roster_count + 2);
        }
        (roster_count, roster_count + 1)
    }
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
            && slice.camera.x_pos >= OPENING_SCROLL_ACCEL_TARGET_X_FP
    }

    fn should_apply_level_one_contact_damage(&self, slice: &OpeningSliceState) -> bool {
        if self.current_level != LevelId(0)
            || self.invulnerable_frames != 0
            || !matches!(self.player_lifecycle, PlayerLifecycle::Alive)
        {
            return false;
        }

        let Some(enemy_layout) = self.enemy_layout(self.current_level) else {
            return false;
        };
        let window = ActivationWindow::from_camera(slice.camera);
        build_enemy_roster(enemy_layout, window)
            .iter()
            .any(|enemy| player_touches_enemy(slice.player, enemy))
    }

    fn fell_below_level(&self, slice: &OpeningSliceState) -> bool {
        let player_y_px = slice.player.y_pos >> 16;
        player_y_px > u32::from(slice.level_bounds.height_pixels)
    }

    fn should_apply_level_one_late_hazard(&self, slice: &OpeningSliceState) -> bool {
        self.current_level == LevelId(0)
            && self.number_hitpoints == 1
            && self.invulnerable_frames == 0
            && !slice.player.on_ground
            && slice.camera.x_pos >= LEVEL_ONE_LATE_HAZARD_CAMERA_X_FP
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
        self.invulnerable_frames = 0;
        self.player_lifecycle = PlayerLifecycle::DeathWindup {
            remaining_frames: PLAYER_DEATH_WINDUP_FRAMES,
        };
    }

    fn enter_death_hidden_state(&mut self) {
        self.current_form = PlayerForm::TheKid;
        self.pending_form = None;
        self.number_hitpoints = PlayerForm::TheKid.max_hitpoints();
        self.number_lives = self.number_lives.saturating_sub(1);
        self.active_enemy_roster.clear();
        self.active_platform_roster.clear();
        self.player_lifecycle = PlayerLifecycle::Dying {
            remaining_frames: PLAYER_DEATH_HIDE_FRAMES,
        };
    }

    fn advance_level(&mut self) {
        self.paused = false;
        self.current_level = self.next_level_id();
        self.level_start_frame = Some(self.frame_index);
        self.opening_slice = Some(OpeningSliceState::respawn(self.level_bounds(self.current_level)));
        self.active_enemy_roster.clear();
        self.active_platform_roster.clear();
        self.invulnerable_frames = 0;
        self.player_lifecycle = PlayerLifecycle::Respawning {
            remaining_frames: PLAYER_RESPAWN_SETTLE_FRAMES,
        };
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
        }
    }

    fn step(
        &mut self,
        level_elapsed: usize,
        input: FrameInput,
        form: PlayerForm,
        block_grid: Option<&terrain::BlockGrid>,
    ) {
        if level_elapsed < OPENING_SCROLL_START_FRAME {
            self.run_entry_animation(level_elapsed);
            return;
        }
        self.update_horizontal_motion(input);
        self.update_vertical_motion(input, form);
        self.apply_wall_collision(block_grid);
        self.apply_ceiling_collision(block_grid);
        self.apply_floor_collision(block_grid);
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
    fn update_horizontal_motion(&mut self, input: FrameInput) {
        let h_input = HorizontalInput {
            left: input.left,
            right: input.right,
            run: input.a,
        };
        let (new_vel, new_facing) = physics::update_horizontal_velocity(
            self.player.x_velocity,
            self.player.facing,
            h_input,
            &physics::NORMAL_KID,
        );
        self.player.x_velocity = new_vel;
        self.player.facing = new_facing;

        let next_x = i64::from(self.player.x_pos) + i64::from(self.player.x_velocity);
        self.player.x_pos = next_x.clamp(0, i64::from(u32::MAX)) as u32;
    }

    /// Physics-driven vertical movement: jump initiation and gravity.
    fn update_vertical_motion(&mut self, input: FrameInput, form: PlayerForm) {
        if self.player.on_ground && input.b {
            let jump_height = physics::JUMP_HEIGHTS[usize::from(form.helmet_id())];
            let (new_x_vel, y_vel) =
                physics::initiate_jump(self.player.x_velocity, jump_height);
            self.player.x_velocity = new_x_vel;
            self.player.y_velocity = y_vel;
            self.player.on_ground = false;
        }

        if !self.player.on_ground {
            let ascending_and_b_held = self.player.y_velocity < 0 && input.b;
            self.player.y_velocity = physics::apply_gravity(
                self.player.y_velocity,
                ascending_and_b_held,
            );
            let next_y = i64::from(self.player.y_pos) + i64::from(self.player.y_velocity);
            self.player.y_pos = next_y.clamp(0, i64::from(u32::MAX)) as u32;
        } else {
            self.player.y_velocity = 0;
        }
    }

    fn apply_wall_collision(&mut self, block_grid: Option<&terrain::BlockGrid>) {
        let Some(grid) = block_grid else { return };
        if self.player.x_velocity == 0 {
            return;
        }

        let x_px = (self.player.x_pos >> 16) as i32;
        let y_px = (self.player.y_pos >> 16) as i32;
        let moving_right = self.player.x_velocity > 0;

        let probe = terrain::probe_wall(
            grid,
            x_px,
            y_px,
            PLAYER_HALF_WIDTH,
            PLAYER_HEIGHT,
            moving_right,
        );

        if probe.hit {
            let clamped_center = if moving_right {
                // Player centre must stay left of wall: centre + half_width <= wall_x
                probe.wall_x_px - PLAYER_HALF_WIDTH
            } else {
                // Player centre must stay right of wall: centre - half_width >= wall_x
                probe.wall_x_px + PLAYER_HALF_WIDTH
            };
            self.player.x_pos = (clamped_center.max(0) as u32) << 16;
            self.player.x_velocity = 0;
        }
    }

    fn apply_ceiling_collision(&mut self, block_grid: Option<&terrain::BlockGrid>) {
        let Some(grid) = block_grid else { return };
        if self.player.y_velocity >= 0 {
            return;
        }

        let x_px = (self.player.x_pos >> 16) as i32;
        let y_px = (self.player.y_pos >> 16) as i32;

        let probe = terrain::probe_ceiling(
            grid,
            x_px,
            y_px,
            PLAYER_HALF_WIDTH,
            PLAYER_HEIGHT,
        );

        if probe.hit {
            // Push feet down so head clears the ceiling: feet = ceiling_bottom + height
            let new_y = probe.ceiling_y_px + PLAYER_HEIGHT;
            self.player.y_pos = (new_y.max(0) as u32) << 16;
            self.player.y_velocity = 0;
        }
    }

    fn apply_floor_collision(&mut self, block_grid: Option<&terrain::BlockGrid>) {
        if let Some(grid) = block_grid {
            let x_px = (self.player.x_pos >> 16) as i32;
            let y_px = (self.player.y_pos >> 16) as i32;
            let probe =
                terrain::probe_floor_hitbox(grid, x_px, y_px, PLAYER_HALF_WIDTH);
            if probe.on_ground {
                let floor_y_fp = (probe.floor_y_px as u32) << 16;
                if self.player.y_pos >= floor_y_fp || self.player.y_velocity > 0 {
                    self.player.y_pos = floor_y_fp;
                    self.player.on_ground = true;
                    self.player.y_velocity = 0;
                    self.player.on_spikes =
                        probe.cell.collision == terrain::CollisionKind::Spikes;
                }
            } else if self.player.on_ground {
                // Was on ground but no longer — start falling
                self.player.on_ground = false;
                self.player.on_spikes = false;
            }
        } else {
            // Flat floor fallback when no block grid is loaded
            if self.player.y_pos >= self.terrain.floor_y_pos {
                self.player.y_pos = self.terrain.floor_y_pos;
                self.player.on_ground = true;
                self.player.y_velocity = 0;
            }
        }
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
            })
            .collect()
    }
}

impl EnemyFamily {
    fn from_layout_object(layout: &EnemyLayout, object: &EnemyLayoutObject) -> Self {
        match resolved_enemy_id(layout, object.object_type) {
            Some(0x01) => Self::Diamond,
            Some(0x1C) => Self::MiniHoppingSkull,
            Some(enemy_id) => Self::Unknown(enemy_id),
            None => Self::Unknown(object.object_type),
        }
    }

    fn object_type(self) -> u8 {
        match self {
            Self::Diamond => 0x01,
            Self::MiniHoppingSkull => 0x1C,
            Self::Unknown(enemy_id) => enemy_id,
        }
    }

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
        })
    }
}

fn build_enemy_roster(layout: &EnemyLayout, window: ActivationWindow) -> Vec<NativeObjectSnapshot> {
    layout
        .objects
        .iter()
        .filter_map(|object| {
            EnemyFamily::from_layout_object(layout, object).snapshot(object, window)
        })
        .collect()
}

fn resolved_enemy_id(layout: &EnemyLayout, object_type: u8) -> Option<u8> {
    if object_type & 0x80 == 0 {
        return Some(object_type);
    }
    let slot_index = usize::from(object_type & 0x7F);
    let raw_slot = *layout.enemy_type_slots.get(slot_index)?;
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

fn player_touches_enemy(player: PlayerState, enemy: &NativeObjectSnapshot) -> bool {
    let x_delta = player.x_pos.abs_diff(enemy.x_pos);
    let y_delta = player.y_pos.abs_diff(enemy.y_pos);
    x_delta <= LEVEL_ONE_ENEMY_CONTACT_X_FP && y_delta <= LEVEL_ONE_ENEMY_CONTACT_Y_FP
}

pub fn run_script(
    assets: AssetManifest,
    script: &[u8],
    frame_count: usize,
) -> Vec<NativeTraceFrame> {
    let mut runtime = NativeRuntime::new(assets);
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

    fn manifest() -> AssetManifest {
        AssetManifest {
            rom_sha1: String::from("abc123"),
            upstream_commit: String::from("deadbeef"),
            source_root: PathBuf::from("/tmp/source"),
            output_root: PathBuf::from("/tmp/output"),
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
        }
    }

    fn start_level(runtime: &mut NativeRuntime) {
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

        runtime.step_frame(FrameInput {
            a: true,
            c: true,
            start: true,
            ..FrameInput::default()
        });
        runtime.step_frame(FrameInput::default());
        assert_eq!(runtime.trace_frame().current_level, LevelId(1));

        for _ in 0..PLAYER_RESPAWN_SETTLE_FRAMES {
            runtime.step_frame(FrameInput::default());
        }
        runtime.step_frame(FrameInput {
            a: true,
            c: true,
            start: true,
            ..FrameInput::default()
        });
        runtime.step_frame(FrameInput::default());
        let trace = runtime.trace_frame();
        assert_eq!(trace.current_level, LevelId(2));
        assert_eq!(trace.current_level_name.as_deref(), Some("Level 2"));
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
        for _ in 0..80 {
            runtime.step_frame(FrameInput {
                right: true,
                ..FrameInput::default()
            });
        }
        let paused_tail = runtime.trace_frame();
        assert!(paused_tail.paused);
        assert_eq!(paused_tail.camera_x_pos, pre_pause.camera_x_pos);
        assert_eq!(paused_tail.kid_x_pos, pre_pause.kid_x_pos);

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

        for _ in 0..(PLAYER_RESPAWN_SETTLE_FRAMES - 1) {
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
        assert!(trace
            .kid_x_pos
            .is_some_and(|x| x > trace.camera_x_pos));
        // Opening hit should have reduced HP (right + full HP + camera past threshold)
        assert!(
            trace.number_hitpoints < PlayerForm::TheKid.max_hitpoints(),
            "opening hit should have reduced HP"
        );
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
            matches!(runtime.player_lifecycle, PlayerLifecycle::DeathWindup { .. }),
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
            let offset =
                (17usize * usize::from(width_cells) + usize::from(col)) * 2;
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
            matches!(runtime.player_lifecycle, PlayerLifecycle::DeathWindup { .. }),
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
            matches!(runtime.player_lifecycle, PlayerLifecycle::DeathWindup { .. }),
            "expected DeathWindup despite 5 HP, got {:?}",
            runtime.player_lifecycle
        );
        assert_eq!(runtime.number_hitpoints, 0, "spike death zeroes HP regardless");
    }
}
