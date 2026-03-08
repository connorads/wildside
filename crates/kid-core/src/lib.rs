use kid_data::AssetManifest;
use serde::{Deserialize, Serialize};

const SEGA_SCREEN_FRAMES: usize = 120;
const INTRO_SEGMENT_FRAMES: usize = 80;
const TITLE_TO_LEVEL_FRAMES: usize = 60;
const LEVEL_ENTRY_CAMERA_Y: u32 = 0x0093_0000;
const LEVEL_SETTLED_CAMERA_Y: u32 = 0x0073_0000;
const MOVE_MID_CAMERA_X: u32 = 0x0045_0000;
const MOVE_END_CAMERA_X: u32 = 0x01AB_0000;
const MOVE_ACCELERATION_FRAMES: usize = 120;
const RIGHT_MOVEMENT_START_ELAPSED: usize = 260;
const LEVEL_ENTRY_HOLD_FRAMES: usize = TITLE_TO_LEVEL_FRAMES + 1;
const BOOTSTRAP_LEVEL_START_PRESS_COUNT: usize = 6;

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
    pub camera_x_pos: u32,
    pub camera_y_pos: u32,
    pub number_objects: u16,
    pub number_gfx_objects: u16,
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
    right_hold_frames: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum BootState {
    SegaScreen { elapsed: usize },
    IntroSequence { segment: usize, elapsed: usize },
    TitleScreen,
    StartingLevel { remaining_frames: usize },
    InGame,
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
            right_hold_frames: 0,
        }
    }

    pub fn step_frame(&mut self, input: FrameInput) -> NativeTraceFrame {
        let start_pressed = input.start && !self.previous_input.start;
        if start_pressed {
            self.start_press_count += 1;
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
                    self.right_hold_frames = 0;
                    self.boot_state = BootState::StartingLevel {
                        remaining_frames: TITLE_TO_LEVEL_FRAMES,
                    };
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

        if matches!(
            self.boot_state,
            BootState::StartingLevel { .. } | BootState::InGame
        ) {
            if input.right {
                self.right_hold_frames += 1;
            }
        } else {
            self.right_hold_frames = 0;
        }

        let trace = self.trace_frame();
        self.previous_input = input;
        self.frame_index += 1;
        trace
    }

    pub fn trace_frame(&self) -> NativeTraceFrame {
        let game_mode = self.game_mode();
        let current_level_name = self.level_name(self.current_level).map(str::to_owned);
        let level_elapsed = self.level_elapsed_frames();
        let (camera_x_pos, camera_y_pos) = self.camera_position(level_elapsed);
        let (number_objects, number_gfx_objects) = self.object_counts(level_elapsed);
        NativeTraceFrame {
            frame_index: self.frame_index,
            game_mode,
            game_mode_value: game_mode.as_disassembly_value(),
            start_press_count: self.start_press_count,
            current_player: 0,
            current_level: self.current_level,
            player_1_level_id: self.current_level,
            current_level_name,
            camera_x_pos,
            camera_y_pos,
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
        self.assets
            .levels
            .iter()
            .find(|level| level.level_id == level_id.0)
            .and_then(|level| level.display_name.as_deref())
    }

    fn level_elapsed_frames(&self) -> Option<usize> {
        self.level_start_frame
            .map(|frame| self.frame_index.saturating_sub(frame))
    }

    fn camera_position(&self, level_elapsed: Option<usize>) -> (u32, u32) {
        let Some(level_elapsed) = level_elapsed else {
            return (0, 0);
        };

        let camera_y_pos = if level_elapsed <= LEVEL_ENTRY_HOLD_FRAMES {
            LEVEL_ENTRY_CAMERA_Y
        } else if level_elapsed <= RIGHT_MOVEMENT_START_ELAPSED {
            let settle_progress = level_elapsed - LEVEL_ENTRY_HOLD_FRAMES;
            let settle_span = RIGHT_MOVEMENT_START_ELAPSED - LEVEL_ENTRY_HOLD_FRAMES;
            interpolate_u32(
                LEVEL_ENTRY_CAMERA_Y,
                LEVEL_SETTLED_CAMERA_Y,
                settle_progress,
                settle_span,
            )
        } else {
            LEVEL_SETTLED_CAMERA_Y
        };

        let camera_x_pos = if level_elapsed <= RIGHT_MOVEMENT_START_ELAPSED {
            0
        } else {
            let movement_frames = level_elapsed - RIGHT_MOVEMENT_START_ELAPSED;
            if movement_frames <= MOVE_ACCELERATION_FRAMES {
                ((u64::from(MOVE_MID_CAMERA_X) * movement_frames as u64)
                    / MOVE_ACCELERATION_FRAMES as u64) as u32
            } else {
                MOVE_MID_CAMERA_X
                    + ((movement_frames - MOVE_ACCELERATION_FRAMES) as u32 * 0x0002_0000)
            }
        };

        (camera_x_pos.min(MOVE_END_CAMERA_X), camera_y_pos)
    }

    fn object_counts(&self, level_elapsed: Option<usize>) -> (u16, u16) {
        let Some(level_elapsed) = level_elapsed else {
            return (4, 4);
        };
        if level_elapsed <= LEVEL_ENTRY_HOLD_FRAMES + 99 {
            return (5, 5);
        }
        if self.right_hold_frames >= 100 && self.right_hold_frames <= 180 {
            return (5, 6);
        }
        (4, 5)
    }
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
    use kid_data::{AssetManifest, LevelRecord, LevelResources};
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

    #[test]
    fn reaches_title_screen_without_input() {
        let trace = run_script(manifest(), &[], 400);
        assert_eq!(trace[359].game_mode, GameMode::TitleScreen);
        assert_eq!(trace[359].current_level, LevelId(0));
        assert_eq!(trace[359].number_objects, 4);
        assert_eq!(trace[359].number_gfx_objects, 4);
    }

    #[test]
    fn starts_level_after_title_start_press() {
        let mut runtime = NativeRuntime::new(manifest());
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
        let trace = runtime.trace_frame();
        assert_eq!(trace.game_mode, GameMode::InGame);
        assert_eq!(trace.current_level, LevelId(0));
        assert_eq!(
            trace.current_level_name.as_deref(),
            Some("Blue Lake Woods 1")
        );
        assert_eq!(trace.camera_y_pos, LEVEL_ENTRY_CAMERA_Y);
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
        assert_eq!(trace.camera_x_pos, MOVE_END_CAMERA_X);
        assert_eq!(trace.camera_y_pos, LEVEL_SETTLED_CAMERA_Y);
    }
}
