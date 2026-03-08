use kid_data::AssetManifest;
use serde::{Deserialize, Serialize};

const SEGA_SCREEN_FRAMES: usize = 120;
const INTRO_SEGMENT_FRAMES: usize = 80;
const TITLE_TO_LEVEL_FRAMES: usize = 60;

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
    pub current_level: Option<LevelId>,
    pub current_level_name: Option<String>,
}

#[derive(Debug, Clone)]
pub struct NativeRuntime {
    assets: AssetManifest,
    frame_index: usize,
    boot_state: BootState,
    previous_input: FrameInput,
    start_press_count: usize,
    current_level: Option<LevelId>,
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
            current_level: None,
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
                if start_pressed {
                    self.current_level = Some(LevelId(0));
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

        let trace = self.trace_frame();
        self.previous_input = input;
        self.frame_index += 1;
        trace
    }

    pub fn trace_frame(&self) -> NativeTraceFrame {
        let game_mode = self.game_mode();
        let current_level_name = self
            .current_level
            .and_then(|level_id| self.level_name(level_id))
            .map(str::to_owned);
        NativeTraceFrame {
            frame_index: self.frame_index,
            game_mode,
            game_mode_value: game_mode.as_disassembly_value(),
            start_press_count: self.start_press_count,
            current_level: self.current_level,
            current_level_name,
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
    use kid_data::{AssetManifest, LevelRecord};
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
    }

    #[test]
    fn starts_level_after_title_start_press() {
        let mut runtime = NativeRuntime::new(manifest());
        for _ in 0..360 {
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
        assert_eq!(trace.current_level, Some(LevelId(0)));
        assert_eq!(
            trace.current_level_name.as_deref(),
            Some("Blue Lake Woods 1")
        );
    }
}
