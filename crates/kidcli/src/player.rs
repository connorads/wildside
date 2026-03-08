use anyhow::{Context, Result, anyhow};
use font8x8::{BASIC_FONTS, UnicodeFonts};
use kid_md::{AudioLevels, MdRuntime, render_frame};
use kid_rom::{RomError, RomIdentity, VerifiedRom, default_rom_candidates, verify_rom};
use sdl2::AudioSubsystem;
use sdl2::audio::AudioQueue;
use sdl2::controller::{Button, GameController};
use sdl2::event::Event;
use sdl2::keyboard::{Keycode, Scancode};
use sdl2::pixels::PixelFormatEnum;
use sdl2::rect::Rect;
use sdl2::render::WindowCanvas;
use sdl2::video::FullscreenType;
use serde::{Deserialize, Serialize};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::thread;
use std::time::{Duration, Instant};

const FRAME_WIDTH: u32 = 320;
const FRAME_HEIGHT: u32 = 224;
const FRAME_TIME: Duration = Duration::from_micros(16_667);
const AUDIO_SAMPLE_RATE: u32 = 44_100;
const AUDIO_FRAME_SAMPLES: usize = 735;
const AUDIO_QUEUE_BYTES_LOW_WATERMARK: u32 =
    (AUDIO_FRAME_SAMPLES * 2 * 3 * std::mem::size_of::<f32>()) as u32;
const PLAYER_CONFIG_VERSION: u32 = 1;
const TOAST_DURATION_FRAMES: u32 = 180;

#[derive(Debug, Clone)]
pub struct PlayOptions {
    pub rom: Option<PathBuf>,
    pub fullscreen: bool,
    pub scale: Option<u32>,
    pub mute: bool,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
enum DisplayScale {
    #[default]
    Auto,
    One,
    Two,
    Three,
    Four,
}

impl DisplayScale {
    fn factor(self, usable_bounds: Rect) -> u32 {
        match self {
            Self::Auto => {
                let width_scale = (usable_bounds.width() / FRAME_WIDTH).max(1);
                let height_scale = (usable_bounds.height() / FRAME_HEIGHT).max(1);
                width_scale.min(height_scale).clamp(1, 4)
            }
            Self::One => 1,
            Self::Two => 2,
            Self::Three => 3,
            Self::Four => 4,
        }
    }

    fn cycle_forward(self) -> Self {
        match self {
            Self::Auto => Self::One,
            Self::One => Self::Two,
            Self::Two => Self::Three,
            Self::Three => Self::Four,
            Self::Four => Self::Auto,
        }
    }

    fn cycle_backward(self) -> Self {
        match self {
            Self::Auto => Self::Four,
            Self::One => Self::Auto,
            Self::Two => Self::One,
            Self::Three => Self::Two,
            Self::Four => Self::Three,
        }
    }

    fn label(self) -> &'static str {
        match self {
            Self::Auto => "Auto",
            Self::One => "1x",
            Self::Two => "2x",
            Self::Three => "3x",
            Self::Four => "4x",
        }
    }

    fn from_cli(scale: u32) -> Option<Self> {
        match scale {
            1 => Some(Self::One),
            2 => Some(Self::Two),
            3 => Some(Self::Three),
            4 => Some(Self::Four),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct PlayerSettings {
    display_scale: DisplayScale,
    fullscreen: bool,
    master_volume: u8,
    music_volume: u8,
    sfx_volume: u8,
    mute: bool,
    bindings: KeyboardBindings,
}

impl Default for PlayerSettings {
    fn default() -> Self {
        Self {
            display_scale: DisplayScale::Auto,
            fullscreen: false,
            master_volume: 100,
            music_volume: 100,
            sfx_volume: 100,
            mute: false,
            bindings: KeyboardBindings::default(),
        }
    }
}

impl PlayerSettings {
    fn apply_cli_overrides(&mut self, options: &PlayOptions) {
        if let Some(scale) = options.scale.and_then(DisplayScale::from_cli) {
            self.display_scale = scale;
        }
        if options.fullscreen {
            self.fullscreen = true;
        }
        if options.mute {
            self.mute = true;
        }
    }

    fn audio_levels(&self) -> AudioLevels {
        let master = if self.mute {
            0.0
        } else {
            f32::from(self.master_volume) / 100.0
        };
        AudioLevels {
            master,
            // Alpha approximation: GEMS music is mostly YM2612, and PSG carries simple SFX/noise.
            ym2612: f32::from(self.music_volume) / 100.0,
            psg: f32::from(self.sfx_volume) / 100.0,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct PlayerSettingsFile {
    #[serde(default = "default_config_version")]
    version: u32,
    #[serde(default)]
    display: DisplaySettingsFile,
    #[serde(default)]
    audio: AudioSettingsFile,
    #[serde(default)]
    keyboard: KeyboardBindingsFile,
}

fn default_config_version() -> u32 {
    PLAYER_CONFIG_VERSION
}

impl Default for PlayerSettingsFile {
    fn default() -> Self {
        Self {
            version: PLAYER_CONFIG_VERSION,
            display: DisplaySettingsFile::default(),
            audio: AudioSettingsFile::default(),
            keyboard: KeyboardBindingsFile::default(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct DisplaySettingsFile {
    #[serde(default)]
    scale: DisplayScale,
    #[serde(default)]
    fullscreen: bool,
}

impl Default for DisplaySettingsFile {
    fn default() -> Self {
        Self {
            scale: DisplayScale::Auto,
            fullscreen: false,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct AudioSettingsFile {
    #[serde(default = "default_hundred")]
    master_volume: u8,
    #[serde(default = "default_hundred")]
    music_volume: u8,
    #[serde(default = "default_hundred")]
    sfx_volume: u8,
    #[serde(default)]
    mute: bool,
}

impl Default for AudioSettingsFile {
    fn default() -> Self {
        Self {
            master_volume: 100,
            music_volume: 100,
            sfx_volume: 100,
            mute: false,
        }
    }
}

fn default_hundred() -> u8 {
    100
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct KeyboardBindingsFile {
    #[serde(default = "default_key_up")]
    up: String,
    #[serde(default = "default_key_down")]
    down: String,
    #[serde(default = "default_key_left")]
    left: String,
    #[serde(default = "default_key_right")]
    right: String,
    #[serde(default = "default_key_a")]
    a: String,
    #[serde(default = "default_key_b")]
    b: String,
    #[serde(default = "default_key_c")]
    c: String,
    #[serde(default = "default_key_start")]
    start: String,
    #[serde(default = "default_key_pause")]
    pause: String,
}

impl Default for KeyboardBindingsFile {
    fn default() -> Self {
        Self {
            up: default_key_up(),
            down: default_key_down(),
            left: default_key_left(),
            right: default_key_right(),
            a: default_key_a(),
            b: default_key_b(),
            c: default_key_c(),
            start: default_key_start(),
            pause: default_key_pause(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct KeyboardBindings {
    up: Keycode,
    down: Keycode,
    left: Keycode,
    right: Keycode,
    a: Keycode,
    b: Keycode,
    c: Keycode,
    start: Keycode,
    pause: Keycode,
}

impl Default for KeyboardBindings {
    fn default() -> Self {
        Self {
            up: Keycode::Up,
            down: Keycode::Down,
            left: Keycode::Left,
            right: Keycode::Right,
            a: Keycode::C,
            b: Keycode::Z,
            c: Keycode::X,
            start: Keycode::Return,
            pause: Keycode::Escape,
        }
    }
}

impl KeyboardBindings {
    fn to_file(&self) -> KeyboardBindingsFile {
        KeyboardBindingsFile {
            up: self.up.name(),
            down: self.down.name(),
            left: self.left.name(),
            right: self.right.name(),
            a: self.a.name(),
            b: self.b.name(),
            c: self.c.name(),
            start: self.start.name(),
            pause: self.pause.name(),
        }
    }

    fn from_file(file: KeyboardBindingsFile) -> Self {
        let defaults = Self::default();
        Self {
            up: parse_key_name(&file.up).unwrap_or(defaults.up),
            down: parse_key_name(&file.down).unwrap_or(defaults.down),
            left: parse_key_name(&file.left).unwrap_or(defaults.left),
            right: parse_key_name(&file.right).unwrap_or(defaults.right),
            a: parse_key_name(&file.a).unwrap_or(defaults.a),
            b: parse_key_name(&file.b).unwrap_or(defaults.b),
            c: parse_key_name(&file.c).unwrap_or(defaults.c),
            start: parse_key_name(&file.start).unwrap_or(defaults.start),
            pause: parse_key_name(&file.pause).unwrap_or(defaults.pause),
        }
    }

    fn key_for(&self, action: BindingAction) -> Keycode {
        match action {
            BindingAction::Up => self.up,
            BindingAction::Down => self.down,
            BindingAction::Left => self.left,
            BindingAction::Right => self.right,
            BindingAction::A => self.a,
            BindingAction::B => self.b,
            BindingAction::C => self.c,
            BindingAction::Start => self.start,
            BindingAction::Pause => self.pause,
        }
    }

    fn set_key(&mut self, action: BindingAction, key: Keycode) -> Result<()> {
        if let Some(conflict) = BindingAction::ALL
            .iter()
            .copied()
            .find(|candidate| *candidate != action && self.key_for(*candidate) == key)
        {
            return Err(anyhow!(
                "{} is already bound to {}",
                key.name(),
                conflict.label()
            ));
        }
        match action {
            BindingAction::Up => self.up = key,
            BindingAction::Down => self.down = key,
            BindingAction::Left => self.left = key,
            BindingAction::Right => self.right = key,
            BindingAction::A => self.a = key,
            BindingAction::B => self.b = key,
            BindingAction::C => self.c = key,
            BindingAction::Start => self.start = key,
            BindingAction::Pause => self.pause = key,
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum BindingAction {
    Up,
    Down,
    Left,
    Right,
    A,
    B,
    C,
    Start,
    Pause,
}

impl BindingAction {
    const ALL: [Self; 9] = [
        Self::Up,
        Self::Down,
        Self::Left,
        Self::Right,
        Self::A,
        Self::B,
        Self::C,
        Self::Start,
        Self::Pause,
    ];

    fn label(self) -> &'static str {
        match self {
            Self::Up => "Bind Up",
            Self::Down => "Bind Down",
            Self::Left => "Bind Left",
            Self::Right => "Bind Right",
            Self::A => "Bind A",
            Self::B => "Bind B",
            Self::C => "Bind C",
            Self::Start => "Bind Start",
            Self::Pause => "Bind Pause",
        }
    }

    fn button_mask(self) -> Option<u8> {
        match self {
            Self::Up => Some(1 << 0),
            Self::Down => Some(1 << 1),
            Self::Left => Some(1 << 2),
            Self::Right => Some(1 << 3),
            Self::B => Some(1 << 4),
            Self::C => Some(1 << 5),
            Self::A => Some(1 << 6),
            Self::Start => Some(1 << 7),
            Self::Pause => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum OptionsOrigin {
    MainMenu,
    Pause,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ScreenState {
    MainMenu {
        selected: usize,
    },
    Options {
        origin: OptionsOrigin,
        selected: usize,
        capture: Option<BindingAction>,
    },
    Playing,
    Paused {
        selected: usize,
    },
    Error,
}

#[derive(Debug, Clone)]
struct StatusToast {
    text: String,
    frames_remaining: u32,
}

#[derive(Debug, Clone)]
struct ErrorView {
    title: String,
    lines: Vec<String>,
}

#[derive(Debug, Clone)]
enum LaunchState {
    Ready {
        verified: VerifiedRom,
        rom_bytes: Vec<u8>,
    },
    Error(ErrorView),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum OptionsItem {
    DisplayScale,
    Fullscreen,
    MasterVolume,
    MusicVolume,
    SfxVolume,
    Mute,
    Bind(BindingAction),
    Back,
}

const OPTIONS_ITEMS: [OptionsItem; 16] = [
    OptionsItem::DisplayScale,
    OptionsItem::Fullscreen,
    OptionsItem::MasterVolume,
    OptionsItem::MusicVolume,
    OptionsItem::SfxVolume,
    OptionsItem::Mute,
    OptionsItem::Bind(BindingAction::Up),
    OptionsItem::Bind(BindingAction::Down),
    OptionsItem::Bind(BindingAction::Left),
    OptionsItem::Bind(BindingAction::Right),
    OptionsItem::Bind(BindingAction::A),
    OptionsItem::Bind(BindingAction::B),
    OptionsItem::Bind(BindingAction::C),
    OptionsItem::Bind(BindingAction::Start),
    OptionsItem::Bind(BindingAction::Pause),
    OptionsItem::Back,
];

pub fn run_player(repo_root: &Path, options: PlayOptions) -> Result<()> {
    let (mut settings, settings_warning) = load_settings()?;
    settings.apply_cli_overrides(&options);
    let launch = resolve_launch_state(repo_root, options.rom.clone())?;

    let sdl = sdl2::init().map_err(|error| anyhow!("failed to init SDL: {error}"))?;
    let video = sdl
        .video()
        .map_err(|error| anyhow!("failed to init SDL video: {error}"))?;
    let audio = sdl.audio().ok();
    let game_controller = sdl
        .game_controller()
        .map_err(|error| anyhow!("failed to init SDL game controller subsystem: {error}"))?;

    let usable_bounds = video
        .display_usable_bounds(0)
        .unwrap_or_else(|_| Rect::new(0, 0, 1280, 720));
    let initial_scale = settings.display_scale.factor(usable_bounds);
    let window = video
        .window(
            "Kid Chameleon",
            FRAME_WIDTH * initial_scale,
            FRAME_HEIGHT * initial_scale,
        )
        .position_centered()
        .resizable()
        .build()
        .map_err(|error| anyhow!("failed to create SDL window: {error}"))?;
    let mut canvas = window
        .into_canvas()
        .present_vsync()
        .build()
        .map_err(|error| anyhow!("failed to create SDL canvas: {error}"))?;
    canvas
        .set_logical_size(FRAME_WIDTH, FRAME_HEIGHT)
        .map_err(|error| anyhow!("failed to set SDL logical size: {error}"))?;
    canvas
        .set_integer_scale(true)
        .map_err(|error| anyhow!("failed to enable SDL integer scaling: {error}"))?;
    apply_window_settings(&mut canvas, &video, &settings)?;

    let texture_creator = canvas.texture_creator();
    let mut texture = texture_creator
        .create_texture_streaming(PixelFormatEnum::RGB24, FRAME_WIDTH, FRAME_HEIGHT)
        .map_err(|error| anyhow!("failed to create SDL texture: {error}"))?;
    let mut events = sdl
        .event_pump()
        .map_err(|error| anyhow!("failed to create SDL event pump: {error}"))?;
    let mut controller = open_first_controller(&game_controller)?;
    let audio_queue = audio
        .as_ref()
        .and_then(|audio| open_audio_queue(audio).transpose())
        .transpose()?;

    if let Some(warning) = &settings_warning {
        eprintln!("{warning}");
    }

    let mut status_toast = settings_warning.map(|warning| StatusToast {
        text: warning,
        frames_remaining: TOAST_DURATION_FRAMES,
    });
    let mut runtime = match &launch {
        LaunchState::Ready { rom_bytes, .. } => Some(MdRuntime::new(rom_bytes)?),
        LaunchState::Error(_) => None,
    };
    if let Some(runtime) = runtime.as_mut() {
        runtime.set_audio_levels(settings.audio_levels());
    }
    if audio_queue.is_none() && runtime.is_some() {
        status_toast = Some(StatusToast {
            text: String::from("Audio unavailable: running silently"),
            frames_remaining: TOAST_DURATION_FRAMES,
        });
    }

    let mut screen = match &launch {
        LaunchState::Ready { .. } => ScreenState::MainMenu { selected: 0 },
        LaunchState::Error(_) => ScreenState::Error,
    };
    let mut last_frame = vec![0; (FRAME_WIDTH * FRAME_HEIGHT * 3) as usize];

    'running: loop {
        let frame_started = Instant::now();
        let mut confirm = false;
        let mut back = false;
        let mut menu_up = false;
        let mut menu_down = false;
        let mut menu_left = false;
        let mut menu_right = false;
        let mut toggle_fullscreen = false;
        let mut pause_pressed = false;

        for event in events.poll_iter() {
            match event {
                Event::Quit { .. } => break 'running,
                Event::ControllerDeviceAdded { .. } => {
                    if controller.is_none() {
                        controller = open_first_controller(&game_controller)?;
                    }
                }
                Event::ControllerDeviceRemoved { .. } => {
                    controller = controller.filter(|gamepad| gamepad.attached());
                    if controller.is_none() {
                        controller = open_first_controller(&game_controller)?;
                    }
                }
                Event::KeyDown {
                    keycode: Some(Keycode::F11),
                    repeat: false,
                    ..
                } => {
                    toggle_fullscreen = true;
                }
                Event::KeyDown {
                    keycode: Some(keycode),
                    repeat: false,
                    ..
                } => match screen {
                    ScreenState::Options {
                        capture: Some(action),
                        ..
                    } => {
                        if keycode == Keycode::Escape {
                            status_toast = Some(StatusToast {
                                text: String::from("Rebind cancelled"),
                                frames_remaining: TOAST_DURATION_FRAMES / 2,
                            });
                            if let ScreenState::Options { capture, .. } = &mut screen {
                                *capture = None;
                            }
                        } else if let Err(error) = settings.bindings.set_key(action, keycode) {
                            status_toast = Some(StatusToast {
                                text: error.to_string(),
                                frames_remaining: TOAST_DURATION_FRAMES,
                            });
                        } else {
                            save_settings(&settings)?;
                            status_toast = Some(StatusToast {
                                text: format!("{} set to {}", action.label(), keycode.name()),
                                frames_remaining: TOAST_DURATION_FRAMES / 2,
                            });
                            if let ScreenState::Options { capture, .. } = &mut screen {
                                *capture = None;
                            }
                        }
                    }
                    _ => {
                        if keycode == Keycode::Escape {
                            match screen {
                                ScreenState::Playing => pause_pressed = true,
                                _ => back = true,
                            }
                        } else if keycode == Keycode::Return || keycode == Keycode::Space {
                            confirm = true;
                        } else if keycode == Keycode::Up {
                            menu_up = true;
                        } else if keycode == Keycode::Down {
                            menu_down = true;
                        } else if keycode == Keycode::Left {
                            menu_left = true;
                        } else if keycode == Keycode::Right {
                            menu_right = true;
                        } else if keycode == settings.bindings.pause {
                            pause_pressed = true;
                        }
                    }
                },
                Event::ControllerButtonDown { button, .. } => match button {
                    Button::DPadUp => menu_up = true,
                    Button::DPadDown => menu_down = true,
                    Button::DPadLeft => menu_left = true,
                    Button::DPadRight => menu_right = true,
                    Button::A | Button::Start => confirm = true,
                    Button::B | Button::Back => back = true,
                    _ => {}
                },
                _ => {}
            }
        }

        if toggle_fullscreen {
            settings.fullscreen = !settings.fullscreen;
            apply_window_settings(&mut canvas, &video, &settings)?;
            save_settings(&settings)?;
            status_toast = Some(StatusToast {
                text: format!(
                    "Fullscreen {}",
                    if settings.fullscreen {
                        "enabled"
                    } else {
                        "disabled"
                    }
                ),
                frames_remaining: TOAST_DURATION_FRAMES / 2,
            });
        }

        match screen {
            ScreenState::MainMenu { ref mut selected } => {
                if menu_up {
                    *selected = selected.saturating_sub(1);
                }
                if menu_down {
                    *selected = (*selected + 1).min(2);
                }
                if confirm {
                    match *selected {
                        0 => screen = ScreenState::Playing,
                        1 => {
                            screen = ScreenState::Options {
                                origin: OptionsOrigin::MainMenu,
                                selected: 0,
                                capture: None,
                            }
                        }
                        2 => break 'running,
                        _ => {}
                    }
                }
                if back {
                    break 'running;
                }
            }
            ScreenState::Playing => {
                if pause_pressed {
                    screen = ScreenState::Paused { selected: 0 };
                }
            }
            ScreenState::Paused { ref mut selected } => {
                if menu_up {
                    *selected = selected.saturating_sub(1);
                }
                if menu_down {
                    *selected = (*selected + 1).min(4);
                }
                if confirm {
                    match *selected {
                        0 => screen = ScreenState::Playing,
                        1 => {
                            runtime = match &launch {
                                LaunchState::Ready { rom_bytes, .. } => {
                                    Some(MdRuntime::new(rom_bytes)?)
                                }
                                LaunchState::Error(_) => None,
                            };
                            if let Some(runtime) = runtime.as_mut() {
                                runtime.set_audio_levels(settings.audio_levels());
                            }
                            screen = ScreenState::Playing;
                            status_toast = Some(StatusToast {
                                text: String::from("Run restarted"),
                                frames_remaining: TOAST_DURATION_FRAMES / 2,
                            });
                        }
                        2 => {
                            runtime = match &launch {
                                LaunchState::Ready { rom_bytes, .. } => {
                                    Some(MdRuntime::new(rom_bytes)?)
                                }
                                LaunchState::Error(_) => None,
                            };
                            if let Some(runtime) = runtime.as_mut() {
                                runtime.set_audio_levels(settings.audio_levels());
                            }
                            screen = ScreenState::MainMenu { selected: 0 };
                        }
                        3 => {
                            screen = ScreenState::Options {
                                origin: OptionsOrigin::Pause,
                                selected: 0,
                                capture: None,
                            }
                        }
                        4 => break 'running,
                        _ => {}
                    }
                }
                if back || pause_pressed {
                    screen = ScreenState::Playing;
                }
            }
            ScreenState::Options {
                origin,
                ref mut selected,
                capture,
            } => {
                if capture.is_none() {
                    if menu_up {
                        *selected = selected.saturating_sub(1);
                    }
                    if menu_down {
                        *selected = (*selected + 1).min(OPTIONS_ITEMS.len().saturating_sub(1));
                    }

                    let mut changed = false;
                    match OPTIONS_ITEMS[*selected] {
                        OptionsItem::DisplayScale => {
                            if menu_left {
                                settings.display_scale = settings.display_scale.cycle_backward();
                                changed = true;
                            }
                            if menu_right || confirm {
                                settings.display_scale = settings.display_scale.cycle_forward();
                                changed = true;
                            }
                            if changed {
                                apply_window_settings(&mut canvas, &video, &settings)?;
                            }
                        }
                        OptionsItem::Fullscreen => {
                            if menu_left || menu_right || confirm {
                                settings.fullscreen = !settings.fullscreen;
                                apply_window_settings(&mut canvas, &video, &settings)?;
                                changed = true;
                            }
                        }
                        OptionsItem::MasterVolume => {
                            changed =
                                adjust_volume(&mut settings.master_volume, menu_left, menu_right);
                        }
                        OptionsItem::MusicVolume => {
                            changed =
                                adjust_volume(&mut settings.music_volume, menu_left, menu_right);
                        }
                        OptionsItem::SfxVolume => {
                            changed =
                                adjust_volume(&mut settings.sfx_volume, menu_left, menu_right);
                        }
                        OptionsItem::Mute => {
                            if menu_left || menu_right || confirm {
                                settings.mute = !settings.mute;
                                changed = true;
                            }
                        }
                        OptionsItem::Bind(action) => {
                            if confirm {
                                screen = ScreenState::Options {
                                    origin,
                                    selected: *selected,
                                    capture: Some(action),
                                };
                            }
                        }
                        OptionsItem::Back => {
                            if confirm {
                                screen = match origin {
                                    OptionsOrigin::MainMenu => {
                                        ScreenState::MainMenu { selected: 1 }
                                    }
                                    OptionsOrigin::Pause => ScreenState::Paused { selected: 3 },
                                };
                            }
                        }
                    }

                    if changed {
                        if let Some(runtime) = runtime.as_mut() {
                            runtime.set_audio_levels(settings.audio_levels());
                        }
                        save_settings(&settings)?;
                        status_toast = Some(StatusToast {
                            text: String::from("Settings saved"),
                            frames_remaining: TOAST_DURATION_FRAMES / 2,
                        });
                    }
                    if back {
                        screen = match origin {
                            OptionsOrigin::MainMenu => ScreenState::MainMenu { selected: 1 },
                            OptionsOrigin::Pause => ScreenState::Paused { selected: 3 },
                        };
                    }
                }
            }
            ScreenState::Error => {
                if confirm || back {
                    break 'running;
                }
            }
        }

        let should_step_runtime = matches!(
            screen,
            ScreenState::MainMenu { .. }
                | ScreenState::Playing
                | ScreenState::Options {
                    origin: OptionsOrigin::MainMenu,
                    ..
                }
        );
        if should_step_runtime {
            if let Some(runtime) = runtime.as_mut() {
                let buttons = match screen {
                    ScreenState::Playing => {
                        keyboard_buttons(&events, &settings.bindings)
                            | controller_buttons(controller.as_ref())
                    }
                    _ => 0,
                };
                runtime.set_controller_1(buttons);
                runtime.run_until_vblank(500_000);
                let snapshot = runtime.snapshot();
                let frame = render_frame(&snapshot);
                last_frame = frame.rgb;
                if let Some(audio_queue) = audio_queue.as_ref()
                    && audio_queue.size() < AUDIO_QUEUE_BYTES_LOW_WATERMARK
                {
                    let samples =
                        runtime.generate_audio_samples(AUDIO_SAMPLE_RATE, AUDIO_FRAME_SAMPLES);
                    audio_queue
                        .queue_audio(&samples)
                        .map_err(|error| anyhow!("failed to queue SDL audio samples: {error}"))?;
                }
            }
        } else if let Some(audio_queue) = audio_queue.as_ref() {
            audio_queue.clear();
        }

        let mut composed = last_frame.clone();
        apply_overlay(
            &mut composed,
            &settings,
            &launch,
            &screen,
            status_toast.as_ref(),
        );
        texture
            .update(None, &composed, (FRAME_WIDTH as usize) * 3)
            .map_err(|error| anyhow!("failed to update SDL texture: {error}"))?;
        canvas.clear();
        canvas
            .copy(&texture, None, None)
            .map_err(|error| anyhow!("failed to copy SDL texture: {error}"))?;
        canvas.present();

        if let Some(toast) = &mut status_toast {
            toast.frames_remaining = toast.frames_remaining.saturating_sub(1);
            if toast.frames_remaining == 0 {
                status_toast = None;
            }
        }

        let elapsed = frame_started.elapsed();
        if elapsed < FRAME_TIME {
            thread::sleep(FRAME_TIME - elapsed);
        }
    }

    Ok(())
}

fn apply_overlay(
    rgb: &mut [u8],
    settings: &PlayerSettings,
    launch: &LaunchState,
    screen: &ScreenState,
    toast: Option<&StatusToast>,
) {
    if !matches!(screen, ScreenState::Playing) {
        dim_frame(rgb);
    }
    match screen {
        ScreenState::MainMenu { selected } => {
            draw_panel(rgb, 28, 20, 264, 120, [16, 22, 32], [205, 196, 173]);
            draw_text(rgb, 44, 36, "Kid Chameleon", [255, 240, 196], 2);
            if let LaunchState::Ready { verified, .. } = launch {
                draw_text(
                    rgb,
                    44,
                    58,
                    &rom_identity_label(verified),
                    [160, 214, 255],
                    1,
                );
            }
            let items = ["Start Game", "Options", "Quit"];
            draw_menu_items(rgb, 44, 82, &items, *selected);
            draw_text(
                rgb,
                44,
                124,
                "Enter start  Esc quit  F11 fullscreen",
                [190, 190, 190],
                1,
            );
        }
        ScreenState::Paused { selected } => {
            draw_panel(rgb, 42, 26, 236, 132, [18, 20, 28], [205, 196, 173]);
            draw_text(rgb, 58, 42, "Paused", [255, 240, 196], 2);
            let items = [
                "Resume",
                "Restart Run",
                "Return To Title",
                "Options",
                "Quit To Desktop",
            ];
            draw_menu_items(rgb, 58, 76, &items, *selected);
        }
        ScreenState::Options {
            origin: _,
            selected,
            capture,
        } => {
            draw_panel(rgb, 12, 10, 296, 196, [14, 18, 24], [205, 196, 173]);
            draw_text(rgb, 24, 22, "Options", [255, 240, 196], 2);
            let mut y = 48;
            for (index, item) in OPTIONS_ITEMS.iter().enumerate() {
                let (label, value) = option_label_value(*item, settings, *capture);
                let colour = if index == *selected {
                    [255, 240, 196]
                } else {
                    [220, 220, 220]
                };
                draw_text(rgb, 24, y, &label, colour, 1);
                if !value.is_empty() {
                    draw_text(rgb, 174, y, &value, [160, 214, 255], 1);
                }
                y += 10;
            }
            if let Some(action) = capture {
                draw_text(
                    rgb,
                    24,
                    188,
                    &format!("Press a key for {} (Esc cancels)", action.label()),
                    [255, 188, 140],
                    1,
                );
            } else {
                draw_text(
                    rgb,
                    24,
                    188,
                    "Left/Right change  Enter select  Esc back",
                    [190, 190, 190],
                    1,
                );
            }
        }
        ScreenState::Error => {
            draw_panel(rgb, 20, 28, 280, 140, [28, 12, 12], [255, 208, 160]);
            if let LaunchState::Error(error) = launch {
                draw_text(rgb, 36, 44, &error.title, [255, 220, 180], 2);
                let mut y = 76;
                for line in &error.lines {
                    draw_text(rgb, 36, y, line, [235, 235, 235], 1);
                    y += 10;
                }
                draw_text(
                    rgb,
                    36,
                    152,
                    "Press Enter or Esc to quit",
                    [220, 220, 220],
                    1,
                );
            }
        }
        ScreenState::Playing => {}
    }

    if let Some(toast) = toast {
        draw_panel(rgb, 24, 198, 272, 18, [24, 24, 24], [160, 214, 255]);
        draw_text(rgb, 32, 203, &toast.text, [230, 230, 230], 1);
    }
}

fn option_label_value(
    item: OptionsItem,
    settings: &PlayerSettings,
    capture: Option<BindingAction>,
) -> (String, String) {
    match item {
        OptionsItem::DisplayScale => (
            String::from("Display Scale"),
            settings.display_scale.label().into(),
        ),
        OptionsItem::Fullscreen => (
            String::from("Fullscreen"),
            if settings.fullscreen { "On" } else { "Off" }.into(),
        ),
        OptionsItem::MasterVolume => (
            String::from("Master Volume"),
            format!("{}%", settings.master_volume),
        ),
        OptionsItem::MusicVolume => (
            String::from("Music Volume"),
            format!("{}%", settings.music_volume),
        ),
        OptionsItem::SfxVolume => (
            String::from("SFX Volume"),
            format!("{}%", settings.sfx_volume),
        ),
        OptionsItem::Mute => (
            String::from("Mute"),
            if settings.mute { "On" } else { "Off" }.into(),
        ),
        OptionsItem::Bind(action) => {
            let suffix = if capture == Some(action) {
                " <waiting>"
            } else {
                ""
            };
            (
                action.label().to_string(),
                format!("{}{}", settings.bindings.key_for(action).name(), suffix),
            )
        }
        OptionsItem::Back => (String::from("Back"), String::new()),
    }
}

fn rom_identity_label(verified: &VerifiedRom) -> String {
    format!(
        "{}  {}",
        verified.header.domestic_title,
        match verified.identity {
            RomIdentity::KnownKidChameleon => "(known dump)",
            RomIdentity::HeaderMatchesKidChameleon => "(header-only match)",
        }
    )
}

fn dim_frame(rgb: &mut [u8]) {
    for channel in rgb {
        *channel = (*channel / 3).saturating_add(12);
    }
}

fn draw_menu_items(rgb: &mut [u8], x: i32, y: i32, items: &[&str], selected: usize) {
    let mut row = y;
    for (index, item) in items.iter().enumerate() {
        let colour = if index == selected {
            [255, 240, 196]
        } else {
            [220, 220, 220]
        };
        let prefix = if index == selected { "> " } else { "  " };
        draw_text(rgb, x, row, &format!("{prefix}{item}"), colour, 1);
        row += 14;
    }
}

fn draw_panel(
    rgb: &mut [u8],
    x: i32,
    y: i32,
    width: i32,
    height: i32,
    fill: [u8; 3],
    border: [u8; 3],
) {
    fill_rect(rgb, x, y, width, height, fill);
    fill_rect(rgb, x, y, width, 2, border);
    fill_rect(rgb, x, y + height - 2, width, 2, border);
    fill_rect(rgb, x, y, 2, height, border);
    fill_rect(rgb, x + width - 2, y, 2, height, border);
}

fn draw_text(rgb: &mut [u8], x: i32, y: i32, text: &str, colour: [u8; 3], scale: i32) {
    let mut cursor_x = x;
    for ch in text.chars() {
        if ch == '\n' {
            cursor_x = x;
            continue;
        }
        draw_glyph(rgb, cursor_x, y, ch, colour, scale);
        cursor_x += 8 * scale;
    }
}

fn draw_glyph(rgb: &mut [u8], x: i32, y: i32, ch: char, colour: [u8; 3], scale: i32) {
    let Some(bitmap) = BASIC_FONTS.get(ch) else {
        return;
    };
    for (row, bits) in bitmap.iter().enumerate() {
        for column in 0..8 {
            if bits & (1 << column) == 0 {
                continue;
            }
            fill_rect(
                rgb,
                x + (column * scale),
                y + ((row as i32) * scale),
                scale,
                scale,
                colour,
            );
        }
    }
}

fn fill_rect(rgb: &mut [u8], x: i32, y: i32, width: i32, height: i32, colour: [u8; 3]) {
    for py in y.max(0)..(y + height).min(FRAME_HEIGHT as i32) {
        for px in x.max(0)..(x + width).min(FRAME_WIDTH as i32) {
            let index = ((py as u32 * FRAME_WIDTH + px as u32) * 3) as usize;
            rgb[index..index + 3].copy_from_slice(&colour);
        }
    }
}

fn resolve_launch_state(repo_root: &Path, supplied: Option<PathBuf>) -> Result<LaunchState> {
    if let Some(path) = supplied {
        return match verify_rom(&path) {
            Ok(verified) => match verified.identity {
                RomIdentity::KnownKidChameleon => Ok(LaunchState::Ready {
                    rom_bytes: fs::read(&path)
                        .with_context(|| format!("failed to read ROM from {}", path.display()))?,
                    verified,
                }),
                RomIdentity::HeaderMatchesKidChameleon => Ok(LaunchState::Error(ErrorView {
                    title: String::from("Unsupported Kid Chameleon dump"),
                    lines: vec![
                        format!("File: {}", path.display()),
                        String::from(
                            "The header matches, but the SHA-1 is not the supported dump.",
                        ),
                        String::from("Use the known Kid Chameleon ROM dump for this alpha build."),
                    ],
                })),
            },
            Err(error) => Ok(LaunchState::Error(error_view_from_rom_error(
                Some(path),
                repo_root,
                error,
            ))),
        };
    }

    if let Some(path) = default_rom_candidates(repo_root)
        .into_iter()
        .find(|candidate| candidate.is_file())
    {
        return resolve_launch_state(repo_root, Some(path));
    }

    Ok(LaunchState::Error(ErrorView {
        title: String::from("ROM not found"),
        lines: default_rom_candidates(repo_root)
            .iter()
            .map(|path| format!("Checked: {}", path.display()))
            .collect(),
    }))
}

fn error_view_from_rom_error(
    supplied: Option<PathBuf>,
    repo_root: &Path,
    error: RomError,
) -> ErrorView {
    match error {
        RomError::Read { path, source } => ErrorView {
            title: String::from("ROM unreadable"),
            lines: vec![
                format!("File: {}", path.display()),
                format!("Reason: {source}"),
            ],
        },
        RomError::TooSmall => ErrorView {
            title: String::from("ROM is too small"),
            lines: vec![
                supplied
                    .map(|path| format!("File: {}", path.display()))
                    .unwrap_or_else(|| String::from("The detected ROM is incomplete.")),
                String::from("Expected a full Mega Drive ROM image."),
            ],
        },
        RomError::UnexpectedHeader => ErrorView {
            title: String::from("Wrong game or unsupported ROM"),
            lines: if let Some(path) = supplied {
                vec![
                    format!("File: {}", path.display()),
                    String::from("The ROM header does not match Kid Chameleon."),
                ]
            } else {
                default_rom_candidates(repo_root)
                    .iter()
                    .map(|path| format!("Checked: {}", path.display()))
                    .collect()
            },
        },
    }
}

fn load_settings() -> Result<(PlayerSettings, Option<String>)> {
    let path = config_path()?;
    let contents = match fs::read_to_string(&path) {
        Ok(contents) => contents,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            return Ok((PlayerSettings::default(), None));
        }
        Err(error) => {
            return Ok((
                PlayerSettings::default(),
                Some(format!("Config unreadable, using defaults: {error}")),
            ));
        }
    };

    let file = match toml::from_str::<PlayerSettingsFile>(&contents) {
        Ok(file) => file,
        Err(error) => {
            return Ok((
                PlayerSettings::default(),
                Some(format!("Config invalid, using defaults: {error}")),
            ));
        }
    };
    Ok((
        PlayerSettings {
            display_scale: file.display.scale,
            fullscreen: file.display.fullscreen,
            master_volume: file.audio.master_volume.min(100),
            music_volume: file.audio.music_volume.min(100),
            sfx_volume: file.audio.sfx_volume.min(100),
            mute: file.audio.mute,
            bindings: KeyboardBindings::from_file(file.keyboard),
        },
        None,
    ))
}

fn save_settings(settings: &PlayerSettings) -> Result<()> {
    let path = config_path()?;
    let parent = path
        .parent()
        .ok_or_else(|| anyhow!("config path has no parent: {}", path.display()))?;
    fs::create_dir_all(parent).with_context(|| format!("failed to create {}", parent.display()))?;
    let file = PlayerSettingsFile {
        version: PLAYER_CONFIG_VERSION,
        display: DisplaySettingsFile {
            scale: settings.display_scale,
            fullscreen: settings.fullscreen,
        },
        audio: AudioSettingsFile {
            master_volume: settings.master_volume,
            music_volume: settings.music_volume,
            sfx_volume: settings.sfx_volume,
            mute: settings.mute,
        },
        keyboard: settings.bindings.to_file(),
    };
    let toml = toml::to_string_pretty(&file).context("failed to serialise config")?;
    fs::write(&path, toml).with_context(|| format!("failed to write {}", path.display()))
}

fn config_path() -> Result<PathBuf> {
    if let Some(xdg) = env::var_os("XDG_CONFIG_HOME") {
        return Ok(PathBuf::from(xdg).join("kid-chameleon/config.toml"));
    }
    if let Some(home) = env::var_os("HOME") {
        return Ok(PathBuf::from(home).join(".config/kid-chameleon/config.toml"));
    }
    Err(anyhow!(
        "could not determine config path; set XDG_CONFIG_HOME or HOME"
    ))
}

fn default_key_up() -> String {
    Keycode::Up.name()
}
fn default_key_down() -> String {
    Keycode::Down.name()
}
fn default_key_left() -> String {
    Keycode::Left.name()
}
fn default_key_right() -> String {
    Keycode::Right.name()
}
fn default_key_a() -> String {
    Keycode::C.name()
}
fn default_key_b() -> String {
    Keycode::Z.name()
}
fn default_key_c() -> String {
    Keycode::X.name()
}
fn default_key_start() -> String {
    Keycode::Return.name()
}
fn default_key_pause() -> String {
    Keycode::Escape.name()
}

fn parse_key_name(name: &str) -> Option<Keycode> {
    Keycode::from_name(name)
}

fn adjust_volume(value: &mut u8, left: bool, right: bool) -> bool {
    let old = *value;
    if left {
        *value = value.saturating_sub(5);
    }
    if right {
        *value = value.saturating_add(5).min(100);
    }
    *value != old
}

fn apply_window_settings(
    canvas: &mut WindowCanvas,
    video: &sdl2::VideoSubsystem,
    settings: &PlayerSettings,
) -> Result<()> {
    if settings.fullscreen {
        canvas
            .window_mut()
            .set_fullscreen(FullscreenType::Desktop)
            .map_err(|error| anyhow!("failed to enable fullscreen: {error}"))?;
    } else {
        canvas
            .window_mut()
            .set_fullscreen(FullscreenType::Off)
            .map_err(|error| anyhow!("failed to disable fullscreen: {error}"))?;
        let usable_bounds = video
            .display_usable_bounds(0)
            .unwrap_or_else(|_| Rect::new(0, 0, 1280, 720));
        let scale = settings.display_scale.factor(usable_bounds);
        canvas
            .window_mut()
            .set_size(FRAME_WIDTH * scale, FRAME_HEIGHT * scale)
            .map_err(|error| anyhow!("failed to resize SDL window: {error}"))?;
        canvas.window_mut().set_position(
            sdl2::video::WindowPos::Centered,
            sdl2::video::WindowPos::Centered,
        );
    }
    Ok(())
}

fn open_audio_queue(audio: &AudioSubsystem) -> Result<Option<AudioQueue<f32>>> {
    let desired = sdl2::audio::AudioSpecDesired {
        freq: Some(AUDIO_SAMPLE_RATE as i32),
        channels: Some(2),
        samples: Some(1024),
    };
    let queue = match audio.open_queue::<f32, _>(None, &desired) {
        Ok(queue) => queue,
        Err(_) => return Ok(None),
    };
    queue.resume();
    Ok(Some(queue))
}

fn open_first_controller(
    game_controller: &sdl2::GameControllerSubsystem,
) -> Result<Option<GameController>> {
    let joystick_count = game_controller
        .num_joysticks()
        .map_err(|error| anyhow!("failed to enumerate SDL controllers: {error}"))?;
    for index in 0..joystick_count {
        if !game_controller.is_game_controller(index) {
            continue;
        }
        let controller = game_controller
            .open(index)
            .map_err(|error| anyhow!("failed to open SDL controller {index}: {error}"))?;
        return Ok(Some(controller));
    }
    Ok(None)
}

fn keyboard_buttons(events: &sdl2::EventPump, bindings: &KeyboardBindings) -> u8 {
    let keyboard = events.keyboard_state();
    let mut buttons = 0u8;
    for action in BindingAction::ALL {
        let Some(mask) = action.button_mask() else {
            continue;
        };
        let keycode = bindings.key_for(action);
        let Some(scancode) = Scancode::from_keycode(keycode) else {
            continue;
        };
        if keyboard.is_scancode_pressed(scancode) {
            buttons |= mask;
        }
    }
    buttons
}

fn controller_buttons(controller: Option<&GameController>) -> u8 {
    let Some(controller) = controller else {
        return 0;
    };

    let mut buttons = 0u8;
    if controller.button(Button::DPadUp) {
        buttons |= 1 << 0;
    }
    if controller.button(Button::DPadDown) {
        buttons |= 1 << 1;
    }
    if controller.button(Button::DPadLeft) {
        buttons |= 1 << 2;
    }
    if controller.button(Button::DPadRight) {
        buttons |= 1 << 3;
    }
    if controller.button(Button::A) {
        buttons |= 1 << 4;
    }
    if controller.button(Button::B) {
        buttons |= 1 << 5;
    }
    if controller.button(Button::X) {
        buttons |= 1 << 6;
    }
    if controller.button(Button::Start) {
        buttons |= 1 << 7;
    }
    buttons
}

#[cfg(test)]
mod tests {
    use super::{
        BindingAction, DisplayScale, KeyboardBindings, PlayerSettings, PlayerSettingsFile,
        adjust_volume,
    };

    #[test]
    fn keyboard_rebind_rejects_conflicts() {
        let mut bindings = KeyboardBindings::default();
        let error = bindings
            .set_key(BindingAction::A, bindings.b)
            .expect_err("conflicting key should be rejected");
        assert!(error.to_string().contains("already bound"));
    }

    #[test]
    fn settings_file_round_trip_preserves_defaults() {
        let settings = PlayerSettings::default();
        let file = PlayerSettingsFile {
            version: 1,
            display: super::DisplaySettingsFile {
                scale: settings.display_scale,
                fullscreen: settings.fullscreen,
            },
            audio: super::AudioSettingsFile {
                master_volume: settings.master_volume,
                music_volume: settings.music_volume,
                sfx_volume: settings.sfx_volume,
                mute: settings.mute,
            },
            keyboard: settings.bindings.to_file(),
        };
        let encoded = toml::to_string_pretty(&file).expect("settings should encode");
        let decoded =
            toml::from_str::<PlayerSettingsFile>(&encoded).expect("settings should decode");
        assert_eq!(decoded.display.scale, DisplayScale::Auto);
        assert_eq!(decoded.audio.master_volume, 100);
    }

    #[test]
    fn volume_adjustment_clamps_to_valid_range() {
        let mut value = 100;
        assert!(!adjust_volume(&mut value, false, true));
        assert_eq!(value, 100);
        assert!(adjust_volume(&mut value, true, false));
        assert_eq!(value, 95);
    }
}
