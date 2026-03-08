mod oracle;
mod player;
mod scenarios;

use anyhow::{Context, Result, anyhow, bail};
use clap::{Parser, Subcommand};
use image::{ColorType, ImageFormat};
use kid_core::{GameMode as NativeGameMode, NativeTraceFrame, run_script as run_native_script};
use kid_data::{
    AssetManifest, export_split_bundle, find_enemy_definition, generated_bundle_dir,
    load_level_asset_bundle, load_manifest, manifest_path as generated_manifest_path,
};
use kid_md::{MdRuntime, probe_rom, probe_rom_with_script, render_frame};
use kid_rom::{RomIdentity, VerifiedRom, find_default_rom, known_sha1, verify_rom};
use oracle::{
    OracleEnvironment, OracleManifest, OracleState, ensure_oracles, run_jgenesis_capture,
};
use player::{PlayOptions, run_player};
use scenarios::{GameplayScenario, Scenario};
use serde::Serialize;
use std::env;
use std::fs;
use std::fs::OpenOptions;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::thread;
use std::time::{Duration, Instant};
use ymfm_sys::Ym2612;

const ASL_REPO_URL: &str = "https://github.com/Macroassembler-AS/asl-releases.git";
const UPSTREAM_URL: &str = "https://github.com/sonicretro/kid-chameleon-disasm";
const UPSTREAM_COMMIT: &str = "a134567";

#[derive(Debug, Parser)]
#[command(name = "kidcli")]
#[command(about = "Contained tooling for the Kid Chameleon PC-port workspace")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    VerifyRom {
        #[arg(long)]
        rom: Option<PathBuf>,
    },
    Play {
        #[arg(long)]
        rom: Option<PathBuf>,
        #[arg(long, default_value_t = false)]
        fullscreen: bool,
        #[arg(long)]
        scale: Option<u32>,
        #[arg(long, default_value_t = false)]
        mute: bool,
    },
    ProbeCpu {
        #[arg(long)]
        rom: Option<PathBuf>,
        #[arg(long, default_value_t = 16)]
        steps: u32,
        #[arg(long)]
        press_start_frame: Vec<usize>,
        #[arg(long)]
        input: Vec<String>,
        #[arg(long, default_value_t = false)]
        summary_only: bool,
        #[arg(long, default_value_t = 64)]
        step_log_limit: usize,
        #[arg(long, default_value_t = 16)]
        trace_log_limit: usize,
    },
    VerifyBoot {
        #[arg(long)]
        rom: Option<PathBuf>,
        #[arg(long, default_value_t = 32)]
        steps: u32,
    },
    VerifyStartup {
        #[arg(long)]
        rom: Option<PathBuf>,
        #[arg(long, default_value_t = 120_000)]
        steps: u32,
    },
    VerifyLevel {
        #[arg(long)]
        rom: Option<PathBuf>,
        #[arg(long, default_value_t = 8_000_000)]
        steps: u32,
    },
    VerifyGameplay {
        #[arg(long)]
        rom: Option<PathBuf>,
        #[arg(long, value_enum)]
        scenario: Option<GameplayScenario>,
        #[arg(long, default_value_t = false)]
        all: bool,
    },
    VerifyAudioDriver {
        #[arg(long)]
        rom: Option<PathBuf>,
        #[arg(long, default_value_t = 8_000_000)]
        steps: u32,
    },
    VerifyAudioOutput {
        #[arg(long)]
        rom: Option<PathBuf>,
        #[arg(long, value_enum, default_value_t = Scenario::BootTitleStartLevel1)]
        scenario: Scenario,
    },
    TraceAudio {
        #[arg(long)]
        rom: Option<PathBuf>,
        #[arg(long, value_enum, default_value_t = Scenario::BootTitleStartLevel1)]
        scenario: Scenario,
    },
    VerifyFrames {
        #[arg(long)]
        rom: Option<PathBuf>,
        #[arg(long, value_enum, default_value_t = Scenario::BootTitleStartLevel1MoveRight5s)]
        scenario: Scenario,
    },
    VerifyOracle {
        #[arg(long)]
        rom: Option<PathBuf>,
        #[arg(long, value_enum, default_value_t = Scenario::BootTitleStartLevel1)]
        scenario: Scenario,
    },
    RenderFrame {
        #[arg(long)]
        rom: Option<PathBuf>,
        #[arg(long, default_value_t = 2_000_000)]
        steps: u32,
        #[arg(long)]
        press_start_frame: Vec<usize>,
        #[arg(long)]
        input: Vec<String>,
        #[arg(long)]
        out: PathBuf,
    },
    DumpAudio {
        #[arg(long)]
        rom: Option<PathBuf>,
        #[arg(long, value_enum, default_value_t = Scenario::BootTitleStartLevel1)]
        scenario: Scenario,
        #[arg(long)]
        out: PathBuf,
    },
    RunSdl {
        #[arg(long)]
        rom: Option<PathBuf>,
        #[arg(long, default_value_t = 3)]
        scale: u32,
        #[arg(long, default_value_t = false)]
        autostart_level: bool,
    },
    BootstrapUpstream {
        #[arg(long)]
        dir: Option<PathBuf>,
    },
    OracleSetup,
    ExtractAssets {
        #[arg(long)]
        rom: Option<PathBuf>,
        #[arg(long)]
        dir: Option<PathBuf>,
    },
    ProbeNative {
        #[arg(long)]
        rom: Option<PathBuf>,
        #[arg(long)]
        dir: Option<PathBuf>,
        #[arg(long, value_enum, default_value_t = Scenario::BootTitleStartLevel1)]
        scenario: Scenario,
    },
    InspectLevel {
        #[arg(long)]
        rom: Option<PathBuf>,
        #[arg(long)]
        dir: Option<PathBuf>,
        #[arg(long)]
        level: String,
    },
    InspectEnemy {
        #[arg(long)]
        rom: Option<PathBuf>,
        #[arg(long)]
        dir: Option<PathBuf>,
        #[arg(long)]
        enemy: String,
    },
    VerifyNativeBootstrap {
        #[arg(long)]
        rom: Option<PathBuf>,
        #[arg(long)]
        dir: Option<PathBuf>,
    },
    CompareNativeOracle {
        #[arg(long)]
        rom: Option<PathBuf>,
        #[arg(long)]
        dir: Option<PathBuf>,
        #[arg(long, value_enum, default_value_t = Scenario::BootTitleStartLevel1)]
        scenario: Scenario,
    },
    PortLoop {
        #[arg(long)]
        rom: Option<PathBuf>,
        #[arg(long)]
        dir: Option<PathBuf>,
        #[arg(long, value_enum, default_value_t = Scenario::BootTitleStartLevel1)]
        scenario: Scenario,
        #[arg(long, default_value_t = 1)]
        iterations: u32,
    },
    ReferenceBuild {
        #[arg(long)]
        rom: Option<PathBuf>,
        #[arg(long)]
        dir: Option<PathBuf>,
        #[arg(long, default_value_t = false)]
        allow_unknown_rom: bool,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Commands::VerifyRom { rom } => {
            let repo_root = current_dir()?;
            let rom_path = resolve_rom_path(&repo_root, rom)?;
            let verified = verify_rom(&rom_path)?;
            print_verified_rom(&verified);
        }
        Commands::Play {
            rom,
            fullscreen,
            scale,
            mute,
        } => {
            let repo_root = current_dir()?;
            run_player(
                &repo_root,
                PlayOptions {
                    rom,
                    fullscreen,
                    scale,
                    mute,
                },
            )?;
        }
        Commands::ProbeCpu {
            rom,
            steps,
            press_start_frame,
            input,
            summary_only,
            step_log_limit,
            trace_log_limit,
        } => {
            let repo_root = current_dir()?;
            let rom_path = resolve_rom_path(&repo_root, rom)?;
            let rom_bytes = fs::read(&rom_path)
                .with_context(|| format!("failed to read ROM from {}", rom_path.display()))?;
            let controller_script = build_controller_script(&press_start_frame, &input)?;
            let probe = probe_rom_with_script(&rom_bytes, steps, &controller_script)?;
            print_probe(
                &rom_path,
                &probe,
                summary_only,
                step_log_limit,
                trace_log_limit,
            );
        }
        Commands::VerifyBoot { rom, steps } => {
            let repo_root = current_dir()?;
            let rom_path = resolve_rom_path(&repo_root, rom)?;
            let verified = verify_rom(&rom_path)?;
            if !matches!(verified.identity, RomIdentity::KnownKidChameleon) {
                bail!("verify-boot currently supports only the known Kid Chameleon ROM dump");
            }
            let rom_bytes = fs::read(&rom_path)
                .with_context(|| format!("failed to read ROM from {}", rom_path.display()))?;
            let probe = probe_rom(&rom_bytes, steps)?;
            verify_boot_probe(&probe)?;
            println!("boot verification passed");
        }
        Commands::VerifyStartup { rom, steps } => {
            let repo_root = current_dir()?;
            let rom_path = resolve_rom_path(&repo_root, rom)?;
            let verified = verify_rom(&rom_path)?;
            if !matches!(verified.identity, RomIdentity::KnownKidChameleon) {
                bail!("verify-startup currently supports only the known Kid Chameleon ROM dump");
            }
            let rom_bytes = fs::read(&rom_path)
                .with_context(|| format!("failed to read ROM from {}", rom_path.display()))?;
            let probe = probe_rom(&rom_bytes, steps)?;
            verify_startup_probe(&probe)?;
            println!("startup verification passed");
        }
        Commands::VerifyLevel { rom, steps } => {
            let repo_root = current_dir()?;
            let rom_path = resolve_rom_path(&repo_root, rom)?;
            let verified = verify_rom(&rom_path)?;
            if !matches!(verified.identity, RomIdentity::KnownKidChameleon) {
                bail!("verify-level currently supports only the known Kid Chameleon ROM dump");
            }
            let rom_bytes = fs::read(&rom_path)
                .with_context(|| format!("failed to read ROM from {}", rom_path.display()))?;
            verify_level_probe(&rom_bytes, steps)?;
            println!("level verification passed");
        }
        Commands::VerifyGameplay { rom, scenario, all } => {
            let repo_root = current_dir()?;
            let rom_path = resolve_rom_path(&repo_root, rom)?;
            let verified = verify_rom(&rom_path)?;
            if !matches!(verified.identity, RomIdentity::KnownKidChameleon) {
                bail!("verify-gameplay currently supports only the known Kid Chameleon ROM dump");
            }
            if all && scenario.is_some() {
                bail!("pass either --scenario or --all to verify-gameplay, not both");
            }
            let scenarios = if all {
                Vec::from([
                    GameplayScenario::PauseResumeLevel1,
                    GameplayScenario::DeathRespawnLevel1,
                    GameplayScenario::ProgressTwoTransitions,
                    GameplayScenario::SoakLevel10m,
                ])
            } else if let Some(scenario) = scenario {
                vec![scenario]
            } else {
                bail!("verify-gameplay requires --scenario <name> or --all");
            };
            let rom_bytes = fs::read(&rom_path)
                .with_context(|| format!("failed to read ROM from {}", rom_path.display()))?;
            for scenario in scenarios {
                verify_gameplay_scenario(&repo_root, &rom_bytes, scenario)?;
                println!("gameplay verification passed: {}", scenario.slug());
            }
        }
        Commands::VerifyAudioDriver { rom, steps } => {
            let repo_root = current_dir()?;
            let rom_path = resolve_rom_path(&repo_root, rom)?;
            let verified = verify_rom(&rom_path)?;
            if !matches!(verified.identity, RomIdentity::KnownKidChameleon) {
                bail!(
                    "verify-audio-driver currently supports only the known Kid Chameleon ROM dump"
                );
            }
            let rom_bytes = fs::read(&rom_path)
                .with_context(|| format!("failed to read ROM from {}", rom_path.display()))?;
            verify_audio_driver_probe(&rom_bytes, steps)?;
            println!("audio-driver verification passed");
        }
        Commands::VerifyAudioOutput { rom, scenario } => {
            let repo_root = current_dir()?;
            let rom_path = resolve_rom_path(&repo_root, rom)?;
            let verified = verify_rom(&rom_path)?;
            if !matches!(verified.identity, RomIdentity::KnownKidChameleon) {
                bail!(
                    "verify-audio-output currently supports only the known Kid Chameleon ROM dump"
                );
            }
            let rom_bytes = fs::read(&rom_path)
                .with_context(|| format!("failed to read ROM from {}", rom_path.display()))?;
            verify_audio_output(&repo_root, &rom_path, &rom_bytes, scenario)?;
            println!("audio-output verification passed");
        }
        Commands::TraceAudio { rom, scenario } => {
            let repo_root = current_dir()?;
            let rom_path = resolve_rom_path(&repo_root, rom)?;
            let verified = verify_rom(&rom_path)?;
            if !matches!(verified.identity, RomIdentity::KnownKidChameleon) {
                bail!("trace-audio currently supports only the known Kid Chameleon ROM dump");
            }
            let rom_bytes = fs::read(&rom_path)
                .with_context(|| format!("failed to read ROM from {}", rom_path.display()))?;
            trace_audio(&repo_root, &rom_bytes, scenario)?;
        }
        Commands::VerifyFrames { rom, scenario } => {
            let repo_root = current_dir()?;
            let rom_path = resolve_rom_path(&repo_root, rom)?;
            let verified = verify_rom(&rom_path)?;
            if !matches!(verified.identity, RomIdentity::KnownKidChameleon) {
                bail!("verify-frames currently supports only the known Kid Chameleon ROM dump");
            }
            let rom_bytes = fs::read(&rom_path)
                .with_context(|| format!("failed to read ROM from {}", rom_path.display()))?;
            verify_frames(&repo_root, &rom_path, &rom_bytes, scenario)?;
            println!("frame verification passed");
        }
        Commands::VerifyOracle { rom, scenario } => {
            let repo_root = current_dir()?;
            let rom_path = resolve_rom_path(&repo_root, rom)?;
            let verified = verify_rom(&rom_path)?;
            if !matches!(verified.identity, RomIdentity::KnownKidChameleon) {
                bail!("verify-oracle currently supports only the known Kid Chameleon ROM dump");
            }
            let rom_bytes = fs::read(&rom_path)
                .with_context(|| format!("failed to read ROM from {}", rom_path.display()))?;
            verify_oracle(&repo_root, &rom_path, &rom_bytes, scenario)?;
            println!("oracle verification passed");
        }
        Commands::RenderFrame {
            rom,
            steps,
            press_start_frame,
            input,
            out,
        } => {
            let repo_root = current_dir()?;
            let rom_path = resolve_rom_path(&repo_root, rom)?;
            let rom_bytes = fs::read(&rom_path)
                .with_context(|| format!("failed to read ROM from {}", rom_path.display()))?;
            let controller_script = build_controller_script(&press_start_frame, &input)?;
            let probe = probe_rom_with_script(&rom_bytes, steps, &controller_script)?;
            let frame = render_frame(&probe);
            if let Some(parent) = out.parent()
                && !parent.as_os_str().is_empty()
            {
                fs::create_dir_all(parent)
                    .with_context(|| format!("failed to create {}", parent.display()))?;
            }
            image::save_buffer_with_format(
                &out,
                &frame.rgb,
                frame.width as u32,
                frame.height as u32,
                ColorType::Rgb8,
                ImageFormat::Png,
            )
            .with_context(|| format!("failed to write {}", out.display()))?;
            println!("rendered frame to {}", out.display());
        }
        Commands::RunSdl {
            rom,
            scale,
            autostart_level,
        } => {
            let repo_root = current_dir()?;
            let rom_path = resolve_rom_path(&repo_root, rom)?;
            let rom_bytes = fs::read(&rom_path)
                .with_context(|| format!("failed to read ROM from {}", rom_path.display()))?;
            run_sdl(&rom_bytes, scale, autostart_level)?;
        }
        Commands::DumpAudio { rom, scenario, out } => {
            let repo_root = current_dir()?;
            let rom_path = resolve_rom_path(&repo_root, rom)?;
            let rom_bytes = fs::read(&rom_path)
                .with_context(|| format!("failed to read ROM from {}", rom_path.display()))?;
            dump_audio(&rom_bytes, scenario, &out)?;
        }
        Commands::BootstrapUpstream { dir } => {
            let repo_root = current_dir()?;
            let upstream_dir = resolve_upstream_dir(&repo_root, dir);
            bootstrap_upstream(&upstream_dir)?;
            println!("upstream ready at {}", upstream_dir.display());
        }
        Commands::OracleSetup => {
            let repo_root = current_dir()?;
            let oracle_env = ensure_oracles(&repo_root)?;
            print_oracle_environment(&oracle_env);
        }
        Commands::ExtractAssets { rom, dir } => {
            let repo_root = current_dir()?;
            let rom_path = resolve_rom_path(&repo_root, rom)?;
            let upstream_dir = resolve_upstream_dir(&repo_root, dir);
            let manifest = prepare_generated_assets(&repo_root, &rom_path, &upstream_dir)?;
            print_asset_manifest(&manifest);
        }
        Commands::ProbeNative { rom, dir, scenario } => {
            let repo_root = current_dir()?;
            let rom_path = resolve_rom_path(&repo_root, rom)?;
            let upstream_dir = resolve_upstream_dir(&repo_root, dir);
            let manifest = prepare_generated_assets(&repo_root, &rom_path, &upstream_dir)?;
            probe_native_scenario(&manifest, scenario)?;
        }
        Commands::InspectLevel { rom, dir, level } => {
            let repo_root = current_dir()?;
            let rom_path = resolve_rom_path(&repo_root, rom)?;
            let upstream_dir = resolve_upstream_dir(&repo_root, dir);
            let manifest = prepare_generated_assets(&repo_root, &rom_path, &upstream_dir)?;
            inspect_level(&manifest, &level)?;
        }
        Commands::InspectEnemy { rom, dir, enemy } => {
            let repo_root = current_dir()?;
            let rom_path = resolve_rom_path(&repo_root, rom)?;
            let upstream_dir = resolve_upstream_dir(&repo_root, dir);
            let manifest = prepare_generated_assets(&repo_root, &rom_path, &upstream_dir)?;
            inspect_enemy(&manifest, &enemy)?;
        }
        Commands::VerifyNativeBootstrap { rom, dir } => {
            let repo_root = current_dir()?;
            let rom_path = resolve_rom_path(&repo_root, rom)?;
            let upstream_dir = resolve_upstream_dir(&repo_root, dir);
            let manifest = prepare_generated_assets(&repo_root, &rom_path, &upstream_dir)?;
            verify_native_bootstrap(&manifest)?;
        }
        Commands::CompareNativeOracle { rom, dir, scenario } => {
            let repo_root = current_dir()?;
            let rom_path = resolve_rom_path(&repo_root, rom)?;
            let upstream_dir = resolve_upstream_dir(&repo_root, dir);
            let manifest = prepare_generated_assets(&repo_root, &rom_path, &upstream_dir)?;
            compare_native_oracle(&repo_root, &rom_path, &manifest, scenario)?;
        }
        Commands::PortLoop {
            rom,
            dir,
            scenario,
            iterations,
        } => {
            let repo_root = current_dir()?;
            let rom_path = resolve_rom_path(&repo_root, rom)?;
            let upstream_dir = resolve_upstream_dir(&repo_root, dir);
            let manifest = prepare_generated_assets(&repo_root, &rom_path, &upstream_dir)?;
            run_port_loop(&repo_root, &rom_path, &manifest, scenario, iterations)?;
        }
        Commands::ReferenceBuild {
            rom,
            dir,
            allow_unknown_rom,
        } => {
            let repo_root = current_dir()?;
            let rom_path = resolve_rom_path(&repo_root, rom)?;
            let verified = verify_rom(&rom_path)?;
            if matches!(verified.identity, RomIdentity::HeaderMatchesKidChameleon)
                && !allow_unknown_rom
            {
                bail!(
                    "ROM header matches Kid Chameleon but SHA-1 differs from the known dump (expected {}). Pass --allow-unknown-rom to continue",
                    known_sha1()
                );
            }
            let upstream_dir = resolve_upstream_dir(&repo_root, dir);
            bootstrap_upstream(&upstream_dir)?;
            run_reference_build(&repo_root, &upstream_dir, &verified)?;
        }
    }
    Ok(())
}

fn build_controller_script(
    press_start_frames: &[usize],
    input_specs: &[String],
) -> Result<Vec<u8>> {
    let mut ranges = Vec::new();
    for frame in press_start_frames {
        ranges.push(FrameInputRange {
            start: *frame,
            end: *frame,
            buttons: 0x80,
        });
    }
    for spec in input_specs {
        ranges.push(parse_input_spec(spec)?);
    }

    let Some(max_frame) = ranges.iter().map(|range| range.end).max() else {
        return Ok(Vec::new());
    };
    let mut script = vec![0; max_frame.saturating_add(2)];
    for range in ranges {
        for frame_buttons in script
            .iter_mut()
            .take(range.end.saturating_add(1))
            .skip(range.start)
        {
            *frame_buttons |= range.buttons;
        }
    }
    Ok(script)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct FrameInputRange {
    start: usize,
    end: usize,
    buttons: u8,
}

fn parse_input_spec(spec: &str) -> Result<FrameInputRange> {
    let (frame_spec, button_spec) = spec.split_once(':').ok_or_else(|| {
        anyhow!("input spec must be FRAME:BUTTON or START-END:BUTTON, got {spec}")
    })?;
    let (start, end) = if let Some((start, end)) = frame_spec.split_once('-') {
        let start = start
            .parse::<usize>()
            .with_context(|| format!("invalid input frame {start}"))?;
        let end = end
            .parse::<usize>()
            .with_context(|| format!("invalid input frame {end}"))?;
        if end < start {
            bail!("input frame range must be ascending, got {spec}");
        }
        (start, end)
    } else {
        let frame = frame_spec
            .parse::<usize>()
            .with_context(|| format!("invalid input frame {frame_spec}"))?;
        (frame, frame)
    };
    let buttons = parse_button_mask(button_spec)?;
    Ok(FrameInputRange {
        start,
        end,
        buttons,
    })
}

fn parse_button_mask(spec: &str) -> Result<u8> {
    if let Some(hex) = spec.strip_prefix("0x") {
        return u8::from_str_radix(hex, 16)
            .with_context(|| format!("invalid raw button mask {spec}"));
    }
    if spec.eq_ignore_ascii_case("none") {
        return Ok(0);
    }

    let mut buttons = 0u8;
    for button in spec.split('+') {
        let bit = match button.trim().to_ascii_lowercase().as_str() {
            "up" | "u" => 0,
            "down" | "d" => 1,
            "left" | "l" => 2,
            "right" | "r" => 3,
            "b" => 4,
            "c" => 5,
            "a" => 6,
            "start" | "s" => 7,
            other => bail!("unknown button {other}; use up/down/left/right/a/b/c/start"),
        };
        buttons |= 1 << bit;
    }
    Ok(buttons)
}

fn gameplay_entry_script() -> Result<Vec<u8>> {
    build_controller_script(&[120, 200, 280, 360, 440], &[String::from("460:start")])
}

fn gameplay_move_right_script() -> Result<Vec<u8>> {
    build_controller_script(
        &[120, 200, 280, 360, 440],
        &[String::from("460:start"), String::from("500-650:right")],
    )
}

fn run_sdl(rom_bytes: &[u8], scale: u32, autostart_level: bool) -> Result<()> {
    let mut runtime = MdRuntime::new(rom_bytes)?;
    let autostart_script = if autostart_level {
        Some(gameplay_entry_script()?)
    } else {
        None
    };
    let sdl = sdl2::init().map_err(|error| anyhow!("failed to init SDL: {error}"))?;
    let video = sdl
        .video()
        .map_err(|error| anyhow!("failed to init SDL video: {error}"))?;
    let audio = sdl.audio().ok();
    let game_controller = sdl
        .game_controller()
        .map_err(|error| anyhow!("failed to init SDL game controller subsystem: {error}"))?;
    let window = video
        .window("Kid Chameleon", 320 * scale, 224 * scale)
        .position_centered()
        .build()
        .map_err(|error| anyhow!("failed to create SDL window: {error}"))?;
    let mut canvas = window
        .into_canvas()
        .present_vsync()
        .build()
        .map_err(|error| anyhow!("failed to create SDL canvas: {error}"))?;
    let texture_creator = canvas.texture_creator();
    let mut texture = texture_creator
        .create_texture_streaming(sdl2::pixels::PixelFormatEnum::RGB24, 320, 224)
        .map_err(|error| anyhow!("failed to create SDL texture: {error}"))?;
    let mut events = sdl
        .event_pump()
        .map_err(|error| anyhow!("failed to create SDL event pump: {error}"))?;
    let mut controller = open_first_controller(&game_controller)?;
    let audio_queue = audio
        .as_ref()
        .and_then(|audio| open_audio_queue(audio).transpose())
        .transpose()?;
    let frame_time = Duration::from_micros(16_667);
    let mut frame_index = 0usize;
    const AUDIO_SAMPLE_RATE: u32 = 44_100;
    const AUDIO_FRAME_SAMPLES: usize = 735;
    const AUDIO_QUEUE_BYTES_LOW_WATERMARK: u32 =
        (AUDIO_FRAME_SAMPLES * 2 * 3 * std::mem::size_of::<f32>()) as u32;

    'running: loop {
        let frame_started = Instant::now();
        for event in events.poll_iter() {
            match event {
                sdl2::event::Event::Quit { .. } => break 'running,
                sdl2::event::Event::KeyDown {
                    keycode: Some(sdl2::keyboard::Keycode::Escape),
                    ..
                } => break 'running,
                sdl2::event::Event::ControllerDeviceAdded { .. } => {
                    if controller.is_none() {
                        controller = open_first_controller(&game_controller)?;
                    }
                }
                sdl2::event::Event::ControllerDeviceRemoved { .. } => {
                    controller = controller.filter(|gamepad| gamepad.attached());
                    if controller.is_none() {
                        controller = open_first_controller(&game_controller)?;
                    }
                }
                _ => {}
            }
        }
        let scripted_buttons = autostart_script
            .as_ref()
            .and_then(|script| script.get(frame_index))
            .copied()
            .unwrap_or(0);
        let buttons =
            scripted_buttons | keyboard_buttons(&events) | controller_buttons(controller.as_ref());

        runtime.set_controller_1(buttons);
        runtime.run_until_vblank(500_000);
        let snapshot = runtime.snapshot();
        let frame = render_frame(&snapshot);
        if let Some(audio_queue) = audio_queue.as_ref()
            && audio_queue.size() < AUDIO_QUEUE_BYTES_LOW_WATERMARK
        {
            let samples = runtime.generate_audio_samples(AUDIO_SAMPLE_RATE, AUDIO_FRAME_SAMPLES);
            audio_queue
                .queue_audio(&samples)
                .map_err(|error| anyhow!("failed to queue SDL audio samples: {error}"))?;
        }
        texture
            .update(None, &frame.rgb, frame.width * 3)
            .map_err(|error| anyhow!("failed to update SDL texture: {error}"))?;
        canvas.clear();
        canvas
            .copy(&texture, None, None)
            .map_err(|error| anyhow!("failed to copy SDL texture: {error}"))?;
        canvas.present();

        let elapsed = frame_started.elapsed();
        if elapsed < frame_time {
            thread::sleep(frame_time - elapsed);
        }
        frame_index = frame_index.saturating_add(1);
    }

    Ok(())
}

fn open_audio_queue(audio: &sdl2::AudioSubsystem) -> Result<Option<sdl2::audio::AudioQueue<f32>>> {
    let desired = sdl2::audio::AudioSpecDesired {
        freq: Some(44_100),
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

const AUDIO_SAMPLE_RATE: u32 = 44_100;
const AUDIO_FRAME_SAMPLES: usize = 735;
const AUDIO_ANALYSIS_EPSILON: f32 = 1e-6;
const CAMERA_POSITION_TOLERANCE: u32 = 0x0004_0000;

#[derive(Debug, Clone, PartialEq)]
struct AudioAnalysis {
    peak_left: f32,
    peak_right: f32,
    change_count_left: usize,
    change_count_right: usize,
    first_dynamic_frame: Option<usize>,
    tail_change_count_left: usize,
    tail_change_count_right: usize,
    rms_windows: Vec<f32>,
}

#[derive(Debug, Clone)]
struct ScenarioCapture {
    samples: Vec<f32>,
    frames: Vec<CapturedFrame>,
    states: Vec<CapturedState>,
    audio_trace: AudioTrace,
}

#[derive(Debug, Clone, Serialize)]
struct GameplayCapture {
    scenario: String,
    frame_count: usize,
    missed_vblank_frames: usize,
    states: Vec<GameplayState>,
    checkpoints: Vec<GameplayState>,
}

#[derive(Debug, Clone)]
struct CapturedFrame {
    label: String,
    frame_index: usize,
    width: usize,
    height: usize,
    rgb: Vec<u8>,
    sha1: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct CapturedState {
    label: String,
    frame_index: usize,
    game_mode: u16,
    current_player: u8,
    current_level_id: u16,
    player_1_level_id: u16,
    camera_x_pos: u32,
    camera_y_pos: u32,
    number_objects: u16,
    number_gfx_objects: u16,
}

#[derive(Debug, Clone, Serialize)]
struct GameplayState {
    label: String,
    frame_index: usize,
    game_mode: u16,
    current_player: u8,
    number_lives: u16,
    current_level_id: u16,
    player_1_level_id: u16,
    current_helmet_id: u16,
    camera_x_pos: u32,
    camera_y_pos: u32,
    kid_x_pos: u32,
    kid_y_pos: u32,
    number_objects: u16,
    number_gfx_objects: u16,
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct FrameDiff {
    mean_absolute_error: f32,
    max_channel_delta: u8,
}

#[derive(Debug, Clone, Serialize)]
struct AudioTrace {
    scenario: String,
    sample_rate: u32,
    frame_samples: usize,
    z80_reset_count: usize,
    first_ym2612_write_frame: Option<usize>,
    first_ym2612_write_sample: Option<usize>,
    first_ym2612_key_on_frame: Option<usize>,
    first_ym2612_key_on_sample: Option<usize>,
    first_psg_write_frame: Option<usize>,
    first_psg_write_sample: Option<usize>,
    first_non_silent_sample: Option<usize>,
    first_significant_audio_sample: Option<usize>,
    peak_left: f32,
    peak_right: f32,
    frames: Vec<AudioTraceFrame>,
}

#[derive(Debug, Clone, Serialize)]
struct AudioTraceFrame {
    frame_index: usize,
    z80_reset_count: usize,
    ym2612_write_count: usize,
    ym2612_key_on_count: usize,
    psg_write_count: usize,
    peak_left: f32,
    peak_right: f32,
}

fn capture_scenario(rom_bytes: &[u8], scenario: Scenario) -> Result<ScenarioCapture> {
    let mut runtime = MdRuntime::new(rom_bytes)?;
    let script = scenario.script()?;
    let checkpoints = scenario.checkpoints();
    let mut checkpoint_index = 0usize;
    let mut samples = Vec::with_capacity(scenario.frame_count() * AUDIO_FRAME_SAMPLES * 2);
    let mut frames = Vec::with_capacity(checkpoints.len());
    let mut states = Vec::with_capacity(checkpoints.len());
    let mut audio_frames = Vec::with_capacity(scenario.frame_count());
    let mut first_ym2612_write_frame = None;
    let mut first_ym2612_key_on_frame = None;
    let mut first_psg_write_frame = None;
    let mut previous_z80_reset_count = 0usize;
    let mut previous_ym2612_write_count = 0usize;
    let mut previous_ym2612_key_on_count = 0usize;
    let mut previous_psg_write_count = 0usize;

    for frame_index in 0..scenario.frame_count() {
        let buttons = script.get(frame_index).copied().unwrap_or(0);
        runtime.set_controller_1(buttons);
        runtime.run_until_vblank(500_000);
        let snapshot = runtime.snapshot();
        while checkpoint_index < checkpoints.len()
            && checkpoints[checkpoint_index].frame_index == frame_index
        {
            let checkpoint = checkpoints[checkpoint_index];
            let rendered = render_frame(&snapshot);
            frames.push(CapturedFrame {
                label: checkpoint.label.to_string(),
                frame_index,
                sha1: sha1_hex(&rendered.rgb),
                width: rendered.width,
                height: rendered.height,
                rgb: rendered.rgb,
            });
            states.push(CapturedState {
                label: checkpoint.label.to_string(),
                frame_index,
                game_mode: snapshot.game_mode,
                current_player: snapshot.current_player,
                current_level_id: snapshot.current_level_id,
                player_1_level_id: snapshot.player_1_level_id,
                camera_x_pos: snapshot.camera_x_pos,
                camera_y_pos: snapshot.camera_y_pos,
                number_objects: snapshot.number_objects,
                number_gfx_objects: snapshot.number_gfx_objects,
            });
            checkpoint_index += 1;
        }
        let z80_reset_count = snapshot.z80_reset_count;
        let ym2612_write_count = snapshot.ym2612_write_count;
        let ym2612_key_on_count = snapshot.ym2612_key_on_count;
        let psg_write_count = snapshot.psg_write_count;
        if first_ym2612_write_frame.is_none() && ym2612_write_count > 0 {
            first_ym2612_write_frame = Some(frame_index);
        }
        if first_ym2612_key_on_frame.is_none() && ym2612_key_on_count > 0 {
            first_ym2612_key_on_frame = Some(frame_index);
        }
        if first_psg_write_frame.is_none() && psg_write_count > 0 {
            first_psg_write_frame = Some(frame_index);
        }

        let z80_reset_delta = z80_reset_count.saturating_sub(previous_z80_reset_count);
        let ym2612_write_delta = ym2612_write_count.saturating_sub(previous_ym2612_write_count);
        let ym2612_key_on_delta = ym2612_key_on_count.saturating_sub(previous_ym2612_key_on_count);
        let psg_write_delta = psg_write_count.saturating_sub(previous_psg_write_count);
        previous_z80_reset_count = z80_reset_count;
        previous_ym2612_write_count = ym2612_write_count;
        previous_ym2612_key_on_count = ym2612_key_on_count;
        previous_psg_write_count = psg_write_count;

        let frame_samples = runtime.generate_audio_samples(AUDIO_SAMPLE_RATE, AUDIO_FRAME_SAMPLES);
        let (peak_left, peak_right) = stereo_peak(&frame_samples);
        audio_frames.push(AudioTraceFrame {
            frame_index,
            z80_reset_count: z80_reset_delta,
            ym2612_write_count: ym2612_write_delta,
            ym2612_key_on_count: ym2612_key_on_delta,
            psg_write_count: psg_write_delta,
            peak_left,
            peak_right,
        });
        samples.extend(frame_samples);
    }

    let audio_analysis = analyse_audio_samples(&samples, AUDIO_SAMPLE_RATE);
    let first_non_silent_sample = first_non_silent_sample(&samples);
    let audio_trace = AudioTrace {
        scenario: scenario.slug().to_string(),
        sample_rate: AUDIO_SAMPLE_RATE,
        frame_samples: AUDIO_FRAME_SAMPLES,
        z80_reset_count: previous_z80_reset_count,
        first_ym2612_write_sample: first_ym2612_write_frame
            .map(|frame| frame * AUDIO_FRAME_SAMPLES),
        first_ym2612_write_frame,
        first_ym2612_key_on_sample: first_ym2612_key_on_frame
            .map(|frame| frame * AUDIO_FRAME_SAMPLES),
        first_ym2612_key_on_frame,
        first_psg_write_sample: first_psg_write_frame.map(|frame| frame * AUDIO_FRAME_SAMPLES),
        first_psg_write_frame,
        first_non_silent_sample,
        first_significant_audio_sample: audio_analysis.first_dynamic_frame,
        peak_left: audio_analysis.peak_left,
        peak_right: audio_analysis.peak_right,
        frames: audio_frames,
    };

    Ok(ScenarioCapture {
        samples,
        frames,
        states,
        audio_trace,
    })
}

fn capture_gameplay_scenario(
    rom_bytes: &[u8],
    scenario: GameplayScenario,
) -> Result<GameplayCapture> {
    let mut runtime = MdRuntime::new(rom_bytes)?;
    let script = scenario.script()?;
    let checkpoints = scenario.checkpoints();
    let mut checkpoint_index = 0usize;
    let mut states = Vec::with_capacity(scenario.frame_count());
    let mut captured_checkpoints = Vec::with_capacity(checkpoints.len());
    let mut missed_vblank_frames = 0usize;

    for frame_index in 0..scenario.frame_count() {
        if scenario.uses_level_skip_cheat() && frame_index >= 720 {
            runtime.set_level_skip_cheat_enabled(true);
        }
        let buttons = script.get(frame_index).copied().unwrap_or(0);
        runtime.set_controller_1(buttons);
        if !runtime.run_until_vblank(500_000) {
            missed_vblank_frames += 1;
        }
        let snapshot = runtime.snapshot();
        let state = gameplay_state_from_probe(frame_index, &snapshot);
        while checkpoint_index < checkpoints.len()
            && checkpoints[checkpoint_index].frame_index == frame_index
        {
            let checkpoint = checkpoints[checkpoint_index];
            captured_checkpoints.push(GameplayState {
                label: checkpoint.label.to_string(),
                ..state.clone()
            });
            checkpoint_index += 1;
        }
        states.push(state);
    }

    Ok(GameplayCapture {
        scenario: scenario.slug().to_string(),
        frame_count: scenario.frame_count(),
        missed_vblank_frames,
        states,
        checkpoints: captured_checkpoints,
    })
}

fn gameplay_state_from_probe(frame_index: usize, snapshot: &kid_md::CpuProbe) -> GameplayState {
    GameplayState {
        label: format!("frame_{frame_index:05}"),
        frame_index,
        game_mode: snapshot.game_mode,
        current_player: snapshot.current_player,
        number_lives: snapshot.number_lives,
        current_level_id: snapshot.current_level_id,
        player_1_level_id: snapshot.player_1_level_id,
        current_helmet_id: snapshot.current_helmet_id,
        camera_x_pos: snapshot.camera_x_pos,
        camera_y_pos: snapshot.camera_y_pos,
        kid_x_pos: snapshot.kid_x_pos,
        kid_y_pos: snapshot.kid_y_pos,
        number_objects: snapshot.number_objects,
        number_gfx_objects: snapshot.number_gfx_objects,
    }
}

fn analyse_audio_samples(samples: &[f32], sample_rate: u32) -> AudioAnalysis {
    let mut peak_left = 0.0f32;
    let mut peak_right = 0.0f32;
    let mut change_count_left = 0usize;
    let mut change_count_right = 0usize;
    let mut first_dynamic_frame = None;
    let mut previous_left: Option<f32> = None;
    let mut previous_right: Option<f32> = None;
    let total_frames = samples.len() / 2;
    let tail_window_frames = (sample_rate as usize) * 3;
    let tail_window_start = total_frames.saturating_sub(tail_window_frames);
    let mut tail_change_count_left = 0usize;
    let mut tail_change_count_right = 0usize;
    let rms_window_frames = (sample_rate / 4).max(1) as usize;
    let onset_window_frames = (sample_rate / 20).max(1) as usize;
    let mut rms_windows = Vec::new();
    let mut rms_accumulator = 0.0f64;
    let mut rms_frames = 0usize;
    let mut onset_activity_windows = Vec::new();

    for (frame_index, frame) in samples.chunks_exact(2).enumerate() {
        let left = frame[0];
        let right = frame[1];
        peak_left = peak_left.max(left.abs());
        peak_right = peak_right.max(right.abs());
        if let Some(previous) = previous_left
            && (left - previous).abs() > AUDIO_ANALYSIS_EPSILON
        {
            change_count_left += 1;
            if frame_index >= tail_window_start {
                tail_change_count_left += 1;
            }
            if first_dynamic_frame.is_none() {
                first_dynamic_frame = Some(frame_index);
            }
        }
        if let Some(previous) = previous_right
            && (right - previous).abs() > AUDIO_ANALYSIS_EPSILON
        {
            change_count_right += 1;
            if frame_index >= tail_window_start {
                tail_change_count_right += 1;
            }
            if first_dynamic_frame.is_none() {
                first_dynamic_frame = Some(frame_index);
            }
        }
        previous_left = Some(left);
        previous_right = Some(right);
        rms_accumulator += f64::from((left * left + right * right) * 0.5);
        rms_frames += 1;
        if rms_frames == rms_window_frames {
            rms_windows.push((rms_accumulator / rms_frames as f64).sqrt() as f32);
            rms_accumulator = 0.0;
            rms_frames = 0;
        }
    }

    if rms_frames != 0 {
        rms_windows.push((rms_accumulator / rms_frames as f64).sqrt() as f32);
    }

    for window in samples.chunks_exact(onset_window_frames * 2) {
        let mut left_mean = 0.0f64;
        let mut right_mean = 0.0f64;
        let mut channel_frames = 0usize;
        for frame in window.chunks_exact(2) {
            left_mean += f64::from(frame[0]);
            right_mean += f64::from(frame[1]);
            channel_frames += 1;
        }
        if channel_frames == 0 {
            continue;
        }
        left_mean /= channel_frames as f64;
        right_mean /= channel_frames as f64;
        let mut variance = 0.0f64;
        for frame in window.chunks_exact(2) {
            let left_delta = f64::from(frame[0]) - left_mean;
            let right_delta = f64::from(frame[1]) - right_mean;
            variance += (left_delta * left_delta + right_delta * right_delta) * 0.5;
        }
        onset_activity_windows.push((variance / channel_frames as f64).sqrt() as f32);
    }

    if first_dynamic_frame.is_some() {
        let peak_activity = onset_activity_windows
            .iter()
            .copied()
            .fold(0.0f32, f32::max);
        let activity_threshold = (peak_activity * 0.15).max(5e-4);
        first_dynamic_frame = onset_activity_windows
            .windows(2)
            .position(|window| window[0] >= activity_threshold && window[1] >= activity_threshold)
            .map(|index| index * onset_window_frames);
    }

    AudioAnalysis {
        peak_left,
        peak_right,
        change_count_left,
        change_count_right,
        first_dynamic_frame,
        tail_change_count_left,
        tail_change_count_right,
        rms_windows,
    }
}

fn stereo_peak(samples: &[f32]) -> (f32, f32) {
    let mut peak_left = 0.0f32;
    let mut peak_right = 0.0f32;
    for frame in samples.chunks_exact(2) {
        peak_left = peak_left.max(frame[0].abs());
        peak_right = peak_right.max(frame[1].abs());
    }
    (peak_left, peak_right)
}

fn first_non_silent_sample(samples: &[f32]) -> Option<usize> {
    samples.chunks_exact(2).position(|frame| {
        frame[0].abs() > AUDIO_ANALYSIS_EPSILON || frame[1].abs() > AUDIO_ANALYSIS_EPSILON
    })
}

fn dump_audio(rom_bytes: &[u8], scenario: Scenario, out: &Path) -> Result<()> {
    let capture = capture_scenario(rom_bytes, scenario)?;
    write_wav(out, AUDIO_SAMPLE_RATE, &capture.samples)?;
    println!("wrote audio to {}", out.display());
    Ok(())
}

fn write_wav(out: &Path, sample_rate: u32, samples: &[f32]) -> Result<()> {
    if let Some(parent) = out.parent()
        && !parent.as_os_str().is_empty()
    {
        fs::create_dir_all(parent)
            .with_context(|| format!("failed to create {}", parent.display()))?;
    }

    let spec = hound::WavSpec {
        channels: 2,
        sample_rate,
        bits_per_sample: 32,
        sample_format: hound::SampleFormat::Float,
    };
    let mut writer = hound::WavWriter::create(out, spec)
        .with_context(|| format!("failed to create {}", out.display()))?;
    for sample in samples {
        writer
            .write_sample(sample.clamp(-1.0, 1.0))
            .with_context(|| format!("failed to write {}", out.display()))?;
    }
    writer
        .finalize()
        .with_context(|| format!("failed to finalise {}", out.display()))?;
    Ok(())
}

fn read_wav_f32(path: &Path) -> Result<Vec<f32>> {
    let mut reader = hound::WavReader::open(path)
        .with_context(|| format!("failed to open {}", path.display()))?;
    let spec = reader.spec();
    if spec.channels != 2 {
        bail!(
            "expected stereo WAV from {}, got {}",
            path.display(),
            spec.channels
        );
    }
    match spec.sample_format {
        hound::SampleFormat::Float => reader
            .samples::<f32>()
            .collect::<Result<Vec<_>, _>>()
            .with_context(|| format!("failed to read {}", path.display())),
        hound::SampleFormat::Int => {
            let scale = if spec.bits_per_sample <= 16 {
                f32::from(i16::MAX)
            } else {
                i32::MAX as f32
            };
            reader
                .samples::<i32>()
                .map(|sample| sample.map(|value| value as f32 / scale))
                .collect::<Result<Vec<_>, _>>()
                .with_context(|| format!("failed to read {}", path.display()))
        }
    }
}

fn sha1_hex(bytes: &[u8]) -> String {
    use sha1::Digest;

    let mut hasher = sha1::Sha1::new();
    hasher.update(bytes);
    let digest = hasher.finalize();
    let mut hex = String::with_capacity(digest.len() * 2);
    for byte in digest {
        hex.push_str(&format!("{byte:02x}"));
    }
    hex
}

fn open_first_controller(
    game_controller: &sdl2::GameControllerSubsystem,
) -> Result<Option<sdl2::controller::GameController>> {
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

fn keyboard_buttons(events: &sdl2::EventPump) -> u8 {
    let keyboard = events.keyboard_state();
    let mut buttons = 0u8;
    if keyboard.is_scancode_pressed(sdl2::keyboard::Scancode::Up) {
        buttons |= 1 << 0;
    }
    if keyboard.is_scancode_pressed(sdl2::keyboard::Scancode::Down) {
        buttons |= 1 << 1;
    }
    if keyboard.is_scancode_pressed(sdl2::keyboard::Scancode::Left) {
        buttons |= 1 << 2;
    }
    if keyboard.is_scancode_pressed(sdl2::keyboard::Scancode::Right) {
        buttons |= 1 << 3;
    }
    if keyboard.is_scancode_pressed(sdl2::keyboard::Scancode::Z) {
        buttons |= 1 << 4;
    }
    if keyboard.is_scancode_pressed(sdl2::keyboard::Scancode::X) {
        buttons |= 1 << 5;
    }
    if keyboard.is_scancode_pressed(sdl2::keyboard::Scancode::C) {
        buttons |= 1 << 6;
    }
    if keyboard.is_scancode_pressed(sdl2::keyboard::Scancode::Return) {
        buttons |= 1 << 7;
    }
    buttons
}

fn controller_buttons(controller: Option<&sdl2::controller::GameController>) -> u8 {
    let Some(controller) = controller else {
        return 0;
    };

    use sdl2::controller::Button;

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

fn current_dir() -> Result<PathBuf> {
    env::current_dir().context("failed to determine current directory")
}

fn resolve_rom_path(repo_root: &Path, supplied: Option<PathBuf>) -> Result<PathBuf> {
    match supplied {
        Some(path) => Ok(path),
        None => find_default_rom(repo_root).ok_or_else(|| {
            anyhow!(
                "no ROM found. Checked {}",
                kid_rom::default_rom_candidates(repo_root)
                    .iter()
                    .map(|path| path.display().to_string())
                    .collect::<Vec<_>>()
                    .join(", ")
            )
        }),
    }
}

fn resolve_upstream_dir(repo_root: &Path, supplied: Option<PathBuf>) -> PathBuf {
    supplied.unwrap_or_else(|| repo_root.join(".cache/upstream/kid-chameleon-disasm"))
}

fn print_verified_rom(verified: &VerifiedRom) {
    println!("rom: {}", verified.path.display());
    println!("size: {} bytes", verified.size);
    println!("sha1: {}", verified.sha1);
    println!("identity: {}", verified.identity);
    println!("console: {}", verified.header.console_name);
    println!("domestic title: {}", verified.header.domestic_title);
    println!("overseas title: {}", verified.header.overseas_title);
    println!("serial: {}", verified.header.serial);
    println!("regions: {}", verified.header.regions);
}

fn print_asset_manifest(manifest: &AssetManifest) {
    println!("bundle: {}", manifest.output_root.display());
    println!("rom sha1: {}", manifest.rom_sha1);
    println!("upstream commit: {}", manifest.upstream_commit);
    println!("levels: {}", manifest.levels.len());
    println!("constants: {}", manifest.constants.len());
    println!("ram variables: {}", manifest.ram_variables.len());
    println!("enemy sources: {}", manifest.enemy_sources.len());
    println!("theme assets: {}", manifest.theme_assets.len());
    println!("scene assets: {}", manifest.scene_assets.len());
    println!("animation assets: {}", manifest.animation_assets.len());
    println!("sound assets: {}", manifest.sound_assets.len());
    if let Some(level) = manifest.levels.iter().find(|level| level.level_id == 0) {
        let display_name = level.display_name.as_deref().unwrap_or("<unnamed>");
        println!("level 00: {display_name}");
    }
}

fn inspect_level(manifest: &AssetManifest, level_spec: &str) -> Result<()> {
    let level_id = parse_level_id(level_spec)?;
    let level = manifest
        .levels
        .iter()
        .find(|level| level.level_id == level_id)
        .ok_or_else(|| anyhow!("level 0x{level_id:02X} not found in asset manifest"))?;
    let bundle = load_level_asset_bundle(manifest, level_id)?;

    println!("level: 0x{:02X}", level_id);
    if let Some(name) = &level.display_name {
        println!("name: {name}");
    }
    if let Some(title_text_id) = level.title_text_id {
        println!("title text id: 0x{:02X}", title_text_id);
    }
    println!("resource paths: {}", level.resource_paths.join(", "));
    if let Some(header) = &bundle.parsed_header {
        println!(
            "header: size {}x{} screens, theme 0x{:02X}, bg theme 0x{:02X}, weather 0x{:X}, murder wall 0x{:X}",
            header.width_screens,
            header.height_screens,
            header.level_theme,
            header.background_theme,
            header.weather_flags,
            header.murder_wall_flags
        );
        println!(
            "player start: {},{}  flag: {},{}",
            header.player_start_x, header.player_start_y, header.flag_x, header.flag_y
        );
    }
    print_loaded_asset("platform", bundle.platform_script.as_ref());
    print_loaded_asset("bgscroll", bundle.bgscroll.as_ref());
    print_loaded_asset("header", bundle.header.as_ref());
    print_loaded_asset("enemy", bundle.enemy.as_ref());
    print_loaded_asset("foreground", bundle.foreground.as_ref());
    print_loaded_asset("block", bundle.block.as_ref());
    print_loaded_asset("background", bundle.background.as_ref());
    Ok(())
}

fn print_loaded_asset(label: &str, asset: Option<&kid_data::LoadedAsset>) {
    match asset {
        Some(asset) => println!(
            "{label}: {} ({} bytes)",
            asset.relative_path,
            asset.bytes.len()
        ),
        None => println!("{label}: <none>"),
    }
}

fn parse_level_id(value: &str) -> Result<u8> {
    let trimmed = value.trim();
    let hex = trimmed
        .strip_prefix("0x")
        .or_else(|| trimmed.strip_prefix("0X"))
        .unwrap_or(trimmed);
    u8::from_str_radix(hex, 16)
        .with_context(|| format!("level id must be hex in the range 00-FF, got {value}"))
}

fn inspect_enemy(manifest: &AssetManifest, selector: &str) -> Result<()> {
    let enemy = find_enemy_definition(manifest, selector)
        .ok_or_else(|| anyhow!("enemy {selector} not found in asset manifest"))?;
    println!("enemy id: 0x{:02X}", enemy.id);
    println!("constant: {}", enemy.constant_name);
    println!("name: {}", enemy.symbolic_name);
    match &enemy.source_path {
        Some(path) => println!("source: {path}"),
        None => println!("source: <none>"),
    }
    Ok(())
}

fn print_probe(
    rom_path: &Path,
    probe: &kid_md::CpuProbe,
    summary_only: bool,
    step_log_limit: usize,
    trace_log_limit: usize,
) {
    println!("rom: {}", rom_path.display());
    println!("reset sp: 0x{:08X}", probe.reset_sp);
    println!("reset pc: 0x{:08X}", probe.reset_pc);
    println!("final sp: 0x{:08X}", probe.final_sp);
    println!("final pc: 0x{:08X}", probe.final_pc);
    println!("synthetic vblanks: {}", probe.synthetic_vblanks);
    println!("v-int counter: {}", probe.v_int_counter);
    println!("v-int done: 0x{:02X}", probe.v_int_done);
    println!("game mode: 0x{:04X}", probe.game_mode);
    println!("demo mode flag: 0x{:02X}", probe.demo_mode_flag);
    println!("current player: 0x{:02X}", probe.current_player);
    println!("number lives: {}", probe.number_lives);
    println!("current level id: 0x{:04X}", probe.current_level_id);
    println!("player 1 level id: 0x{:04X}", probe.player_1_level_id);
    println!("current helmet id: 0x{:04X}", probe.current_helmet_id);
    println!("camera x: 0x{:08X}", probe.camera_x_pos);
    println!("camera y: 0x{:08X}", probe.camera_y_pos);
    println!("kid x: 0x{:08X}", probe.kid_x_pos);
    println!("kid y: 0x{:08X}", probe.kid_y_pos);
    println!("number objects: {}", probe.number_objects);
    println!("number gfx objects: {}", probe.number_gfx_objects);
    println!("z80 interrupt mode: {}", probe.z80_interrupt_mode);
    println!("z80 reset count: {}", probe.z80_reset_count);
    println!("vram nonzero bytes: {}", probe.vram_nonzero_bytes);
    println!("cram nonzero bytes: {}", probe.cram_nonzero_bytes);
    println!("vsram nonzero bytes: {}", probe.vsram_nonzero_bytes);
    println!("ym2612 writes: {}", probe.ym2612_write_count);
    println!("ym2612 key-on writes: {}", probe.ym2612_key_on_count);
    println!("psg writes: {}", probe.psg_write_count);
    let ym_register_writes = decode_ym2612_register_writes(&probe.ym2612_writes);
    if !ym_register_writes.is_empty() {
        let key_on_count = ym_register_writes
            .iter()
            .filter(|write| write.register == 0x0028)
            .count();
        println!(
            "ym2612 register writes: {}",
            format_ym2612_register_writes(&ym_register_writes)
        );
        println!("ym2612 key-on writes: {key_on_count}");
        println!(
            "ym2612 replay is audible: {}",
            ym2612_register_writes_are_audible(&ym_register_writes)
        );
    } else if !probe.ym2612_writes.is_empty() {
        println!(
            "ym2612 write samples: {}",
            format_ym2612_writes(&probe.ym2612_writes)
        );
    }
    if !probe.psg_writes.is_empty() {
        println!(
            "psg write samples: {}",
            format_u8_samples(&probe.psg_writes)
        );
    }
    if !probe.banked_68k_reads.is_empty() {
        println!(
            "banked 68k read samples: {}",
            format_banked_68k_accesses(&probe.banked_68k_reads)
        );
    }
    if !probe.banked_68k_writes.is_empty() {
        println!(
            "banked 68k write samples: {}",
            format_banked_68k_accesses(&probe.banked_68k_writes)
        );
    }

    if !summary_only {
        for step in probe.steps.iter().take(step_log_limit) {
            println!(
                "step {:03}: pc 0x{:08X} -> 0x{:08X} ({} cycles)",
                step.index, step.pc_before, step.pc_after, step.cycles
            );
        }
        if probe.steps.len() > step_log_limit {
            println!(
                "step log truncated: showing {} of {} steps",
                step_log_limit,
                probe.steps.len()
            );
        }
    }

    println!("mmio/trace entries: {}", probe.accesses.len());
    if !summary_only {
        for access in probe.accesses.iter().take(trace_log_limit) {
            println!(
                "trace {:?}: 0x{:08X} = 0x{:X}",
                access.kind, access.address, access.value
            );
        }
        if probe.accesses.len() > trace_log_limit {
            println!(
                "trace log truncated: showing {} of {} entries",
                trace_log_limit,
                probe.accesses.len()
            );
        }
    }

    if !probe.vdp_control_writes.is_empty() {
        println!(
            "vdp control writes: {}",
            format_u16_samples(&probe.vdp_control_writes)
        );
    }
    if !probe.vdp_data_writes.is_empty() {
        println!(
            "vdp data writes: {}",
            format_u16_samples(&probe.vdp_data_writes)
        );
    }
}

fn format_u16_samples(values: &[u16]) -> String {
    const EDGE: usize = 8;

    if values.len() <= EDGE * 2 {
        return format!("{values:?}");
    }

    let head = values
        .iter()
        .take(EDGE)
        .map(|value| format!("0x{value:04X}"))
        .collect::<Vec<_>>()
        .join(", ");
    let tail = values
        .iter()
        .skip(values.len() - EDGE)
        .map(|value| format!("0x{value:04X}"))
        .collect::<Vec<_>>()
        .join(", ");
    format!("[{head}, ..., {tail}] ({} total)", values.len())
}

fn format_u8_samples(values: &[u8]) -> String {
    const EDGE: usize = 8;

    if values.len() <= EDGE * 2 {
        return values
            .iter()
            .map(|value| format!("0x{value:02X}"))
            .collect::<Vec<_>>()
            .join(", ");
    }

    let head = values
        .iter()
        .take(EDGE)
        .map(|value| format!("0x{value:02X}"))
        .collect::<Vec<_>>()
        .join(", ");
    let tail = values
        .iter()
        .skip(values.len() - EDGE)
        .map(|value| format!("0x{value:02X}"))
        .collect::<Vec<_>>()
        .join(", ");
    format!("[{head}, ..., {tail}] ({} total)", values.len())
}

fn format_ym2612_writes(values: &[(u16, u8)]) -> String {
    const EDGE: usize = 8;

    let render = |slice: &[(u16, u8)]| {
        slice
            .iter()
            .map(|(port, value)| format!("(0x{port:04X}, 0x{value:02X})"))
            .collect::<Vec<_>>()
            .join(", ")
    };

    if values.len() <= EDGE * 2 {
        return render(values);
    }

    let head = render(&values[..EDGE]);
    let tail = render(&values[values.len() - EDGE..]);
    format!("[{head}, ..., {tail}] ({} total)", values.len())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Ym2612RegisterWrite {
    register: u16,
    value: u8,
}

fn decode_ym2612_register_writes(values: &[(u16, u8)]) -> Vec<Ym2612RegisterWrite> {
    let mut latched = [None, None];
    let mut decoded = Vec::new();

    for (port, value) in values {
        match port & 0x0003 {
            0 => latched[0] = Some(u16::from(*value)),
            2 => latched[1] = Some(0x0100 | u16::from(*value)),
            1 => {
                if let Some(register) = latched[0] {
                    decoded.push(Ym2612RegisterWrite {
                        register,
                        value: *value,
                    });
                }
            }
            3 => {
                if let Some(register) = latched[1] {
                    decoded.push(Ym2612RegisterWrite {
                        register,
                        value: *value,
                    });
                }
            }
            _ => {}
        }
    }

    decoded
}

fn format_ym2612_register_writes(values: &[Ym2612RegisterWrite]) -> String {
    const EDGE: usize = 8;

    let render = |slice: &[Ym2612RegisterWrite]| {
        slice
            .iter()
            .map(|write| format!("(0x{:04X} <- 0x{:02X})", write.register, write.value))
            .collect::<Vec<_>>()
            .join(", ")
    };

    if values.len() <= EDGE * 2 {
        return render(values);
    }

    let head = render(&values[..EDGE]);
    let tail = render(&values[values.len() - EDGE..]);
    format!("[{head}, ..., {tail}] ({} total)", values.len())
}

fn format_banked_68k_accesses(values: &[(u32, u8)]) -> String {
    const EDGE: usize = 8;

    let render = |slice: &[(u32, u8)]| {
        slice
            .iter()
            .map(|(address, value)| format!("(0x{address:06X}, 0x{value:02X})"))
            .collect::<Vec<_>>()
            .join(", ")
    };

    if values.len() <= EDGE * 2 {
        return render(values);
    }

    let head = render(&values[..EDGE]);
    let tail = render(&values[values.len() - EDGE..]);
    format!("[{head}, ..., {tail}] ({} total)", values.len())
}

fn ym2612_register_writes_are_audible(values: &[Ym2612RegisterWrite]) -> bool {
    if values.is_empty() {
        return false;
    }

    let mut chip = Ym2612::default();
    for write in values {
        match write.register {
            0x0000..=0x00FF => {
                chip.write_port(0, write.register as u8);
                chip.write_port(1, write.value);
            }
            0x0100..=0x01FF => {
                chip.write_port(2, (write.register & 0x00FF) as u8);
                chip.write_port(3, write.value);
            }
            _ => {}
        }
    }

    let mut left = [0i32; 512];
    let mut right = [0i32; 512];
    chip.generate(&mut left, &mut right);
    left.iter().any(|sample| *sample != left[0]) || right.iter().any(|sample| *sample != right[0])
}

fn bootstrap_upstream(dir: &Path) -> Result<()> {
    if !dir.exists() {
        let parent = dir
            .parent()
            .ok_or_else(|| anyhow!("upstream directory has no parent: {}", dir.display()))?;
        fs::create_dir_all(parent)
            .with_context(|| format!("failed to create {}", parent.display()))?;
        run_command(
            Command::new("git").arg("clone").arg(UPSTREAM_URL).arg(dir),
            None,
        )?;
    }

    run_command(Command::new("git").arg("fetch").arg("--all"), Some(dir))?;
    run_command(
        Command::new("git")
            .arg("checkout")
            .arg("--detach")
            .arg(UPSTREAM_COMMIT),
        Some(dir),
    )?;
    Ok(())
}

fn prepare_generated_assets(
    repo_root: &Path,
    rom_path: &Path,
    upstream_dir: &Path,
) -> Result<AssetManifest> {
    let _lock = acquire_prepare_lock(repo_root)?;
    let verified = verify_rom(rom_path)?;
    let bundle_dir = generated_bundle_dir(repo_root, &verified.sha1);
    let bundle_manifest_path = generated_manifest_path(&bundle_dir);
    if bundle_manifest_path.is_file() {
        return load_manifest(&bundle_manifest_path);
    }

    bootstrap_upstream(upstream_dir)?;
    sync_rom_into_upstream(rom_path, upstream_dir)?;
    run_command(Command::new("python3").arg("split.py"), Some(upstream_dir))?;
    export_split_bundle(upstream_dir, &bundle_dir, &verified.sha1, UPSTREAM_COMMIT)
}

fn acquire_prepare_lock(repo_root: &Path) -> Result<PrepareLock> {
    let lock_dir = repo_root.join(".cache/generated");
    fs::create_dir_all(&lock_dir)
        .with_context(|| format!("failed to create {}", lock_dir.display()))?;
    let lock_path = lock_dir.join(".prepare.lock");
    let deadline = Instant::now() + Duration::from_secs(30);
    loop {
        match OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&lock_path)
        {
            Ok(_) => return Ok(PrepareLock { path: lock_path }),
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {
                if Instant::now() >= deadline {
                    bail!(
                        "timed out waiting for asset prepare lock at {}",
                        lock_path.display()
                    );
                }
                thread::sleep(Duration::from_millis(200));
            }
            Err(error) => {
                return Err(error)
                    .with_context(|| format!("failed to open {}", lock_path.display()));
            }
        }
    }
}

fn sync_rom_into_upstream(rom_path: &Path, upstream_dir: &Path) -> Result<()> {
    let upstream_rom = upstream_dir.join("kid.bin");
    fs::copy(rom_path, &upstream_rom).with_context(|| {
        format!(
            "failed to copy ROM from {} to {}",
            rom_path.display(),
            upstream_rom.display()
        )
    })?;
    Ok(())
}

fn run_reference_build(
    repo_root: &Path,
    upstream_dir: &Path,
    verified: &VerifiedRom,
) -> Result<()> {
    sync_rom_into_upstream(&verified.path, upstream_dir)?;
    run_command(Command::new("python3").arg("split.py"), Some(upstream_dir))?;
    build_reference_rom(repo_root, upstream_dir)?;

    let rebuilt = verify_rom(&upstream_dir.join("kid_built.bin"))
        .context("rebuilt ROM did not pass Kid Chameleon header verification")?;

    println!("source sha1:  {}", verified.sha1);
    println!("rebuilt sha1: {}", rebuilt.sha1);
    if verified.sha1 != rebuilt.sha1 {
        bail!("reference build completed but ROM is not bit-perfect");
    }

    println!("reference build is bit-perfect");
    Ok(())
}

fn probe_native_scenario(manifest: &AssetManifest, scenario: Scenario) -> Result<()> {
    let script = scenario.script()?;
    let trace = run_native_script(manifest.clone(), &script, scenario.frame_count());
    let final_frame = trace
        .last()
        .ok_or_else(|| anyhow!("native trace was empty for scenario {}", scenario.slug()))?;

    println!("scenario: {}", scenario.slug());
    println!("frames: {}", trace.len());
    println!(
        "final mode: {:?} (0x{:04X})",
        final_frame.game_mode, final_frame.game_mode_value
    );
    println!("current level: 0x{:02X}", final_frame.current_level.0);
    if let Some(level_name) = &final_frame.current_level_name {
        println!("current level name: {level_name}");
    }

    for checkpoint in scenario.checkpoints() {
        if let Some(frame) =
            native_trace_at_checkpoint(&trace, checkpoint.label, checkpoint.frame_index)
        {
            println!(
                "checkpoint {}@{}: {:?} level 0x{:02X} {} camera {:08X},{:08X} objects {}/{}",
                checkpoint.label,
                checkpoint.frame_index,
                frame.game_mode,
                frame.current_level.0,
                frame.current_level_name.as_deref().unwrap_or(""),
                frame.camera_x_pos,
                frame.camera_y_pos,
                frame.number_objects,
                frame.number_gfx_objects
            );
        }
    }

    Ok(())
}

fn verify_native_bootstrap(manifest: &AssetManifest) -> Result<()> {
    let scenario = Scenario::BootTitleStartLevel1;
    let script = scenario.script()?;
    let trace = run_native_script(manifest.clone(), &script, scenario.frame_count());
    let title_frame = trace
        .get(359)
        .ok_or_else(|| anyhow!("native trace never reached title checkpoint"))?;
    if title_frame.game_mode != NativeGameMode::TitleScreen {
        bail!(
            "native bootstrap diverged before title: expected {:?}, got {:?}",
            NativeGameMode::TitleScreen,
            title_frame.game_mode
        );
    }

    let final_frame = trace
        .last()
        .ok_or_else(|| anyhow!("native bootstrap trace was empty"))?;
    if final_frame.game_mode != NativeGameMode::InGame {
        bail!(
            "native bootstrap did not reach gameplay: expected {:?}, got {:?}",
            NativeGameMode::InGame,
            final_frame.game_mode
        );
    }
    if final_frame.current_level.0 != 0 {
        bail!(
            "native bootstrap loaded the wrong first level: expected 0x00, got {:?}",
            final_frame.current_level.0
        );
    }

    println!("native bootstrap verification passed");
    Ok(())
}

fn compare_native_oracle(
    repo_root: &Path,
    rom_path: &Path,
    manifest: &AssetManifest,
    scenario: Scenario,
) -> Result<()> {
    let run_dir = repo_root
        .join(".cache/native-oracle-runs")
        .join(scenario.slug());
    let native_dir = run_dir.join("native");
    let oracle_dir = run_dir.join("oracle");
    if native_dir.exists() {
        fs::remove_dir_all(&native_dir)
            .with_context(|| format!("failed to remove {}", native_dir.display()))?;
    }
    if oracle_dir.exists() {
        fs::remove_dir_all(&oracle_dir)
            .with_context(|| format!("failed to remove {}", oracle_dir.display()))?;
    }
    fs::create_dir_all(&native_dir)
        .with_context(|| format!("failed to create {}", native_dir.display()))?;
    fs::create_dir_all(&oracle_dir)
        .with_context(|| format!("failed to create {}", oracle_dir.display()))?;

    let script = scenario.script()?;
    let native_trace = run_native_script(manifest.clone(), &script, scenario.frame_count());
    let native_states = capture_native_checkpoint_states(&native_trace, scenario)?;
    fs::write(
        native_dir.join("state.json"),
        serde_json::to_vec_pretty(&native_states)?,
    )
    .with_context(|| {
        format!(
            "failed to write {}",
            native_dir.join("state.json").display()
        )
    })?;

    let env = ensure_oracles(repo_root)?;
    let oracle = run_jgenesis_capture(&env, rom_path, scenario, &oracle_dir)?;
    fs::write(
        oracle_dir.join("manifest.json"),
        serde_json::to_vec_pretty(&oracle)?,
    )
    .with_context(|| {
        format!(
            "failed to write {}",
            oracle_dir.join("manifest.json").display()
        )
    })?;

    compare_native_states_against_oracle(&run_dir, &native_states, &oracle.states)?;
    println!(
        "native-oracle checkpoint comparison passed for {}",
        scenario.slug()
    );
    Ok(())
}

fn run_port_loop(
    repo_root: &Path,
    rom_path: &Path,
    manifest: &AssetManifest,
    scenario: Scenario,
    iterations: u32,
) -> Result<()> {
    let total_iterations = iterations.max(1);
    for iteration in 1..=total_iterations {
        println!(
            "port loop iteration {iteration}/{total_iterations} for {}",
            scenario.slug()
        );
        verify_native_bootstrap(manifest)?;
        compare_native_oracle(repo_root, rom_path, manifest, scenario)?;
    }
    Ok(())
}

fn capture_native_checkpoint_states(
    trace: &[NativeTraceFrame],
    scenario: Scenario,
) -> Result<Vec<OracleState>> {
    scenario
        .checkpoints()
        .iter()
        .map(|checkpoint| {
            let frame = native_trace_at_checkpoint(trace, checkpoint.label, checkpoint.frame_index)
                .ok_or_else(|| {
                    anyhow!(
                        "missing native checkpoint {}@{}",
                        checkpoint.label,
                        checkpoint.frame_index
                    )
                })?;
            Ok(native_trace_to_oracle_state(
                checkpoint.label,
                checkpoint.frame_index,
                frame,
            ))
        })
        .collect()
}

fn native_trace_at_checkpoint<'a>(
    trace: &'a [NativeTraceFrame],
    checkpoint_label: &str,
    checkpoint_frame_index: usize,
) -> Option<&'a NativeTraceFrame> {
    if checkpoint_label == "title" {
        return checkpoint_frame_index
            .checked_sub(1)
            .and_then(|index| trace.get(index))
            .or_else(|| trace.get(checkpoint_frame_index));
    }
    trace.get(checkpoint_frame_index).or_else(|| {
        checkpoint_frame_index
            .checked_sub(1)
            .and_then(|index| trace.get(index))
    })
}

fn native_trace_to_oracle_state(
    label: &str,
    frame_index: usize,
    frame: &NativeTraceFrame,
) -> OracleState {
    OracleState {
        label: label.to_owned(),
        frame_index,
        game_mode: frame.game_mode_value,
        current_player: frame.current_player,
        current_level_id: u16::from(frame.current_level.0),
        player_1_level_id: u16::from(frame.player_1_level_id.0),
        camera_x_pos: frame.camera_x_pos,
        camera_y_pos: frame.camera_y_pos,
        number_objects: frame.number_objects,
        number_gfx_objects: frame.number_gfx_objects,
    }
}

fn compare_native_states_against_oracle(
    run_dir: &Path,
    native_states: &[OracleState],
    oracle_states: &[OracleState],
) -> Result<()> {
    if native_states.len() != oracle_states.len() {
        bail!(
            "native captured {} states but oracle captured {}; artefacts in {}",
            native_states.len(),
            oracle_states.len(),
            run_dir.display()
        );
    }

    for (native, oracle) in native_states.iter().zip(oracle_states) {
        if native.label != oracle.label || native.frame_index != oracle.frame_index {
            bail!(
                "state checkpoint mismatch: native {}@{} vs oracle {}@{}",
                native.label,
                native.frame_index,
                oracle.label,
                oracle.frame_index
            );
        }
        if native.game_mode != oracle.game_mode {
            bail!(
                "state {}@{} diverged in game mode (native {:04X}, oracle {:04X}); artefacts in {}",
                native.label,
                native.frame_index,
                native.game_mode,
                oracle.game_mode,
                run_dir.display()
            );
        }
        if native.current_player != oracle.current_player {
            bail!(
                "state {}@{} diverged in current player (native {:02X}, oracle {:02X}); artefacts in {}",
                native.label,
                native.frame_index,
                native.current_player,
                oracle.current_player,
                run_dir.display()
            );
        }
        if native.current_level_id != oracle.current_level_id
            || native.player_1_level_id != oracle.player_1_level_id
        {
            bail!(
                "state {}@{} diverged in level ids (native current {:04X} / p1 {:04X}, oracle current {:04X} / p1 {:04X}); artefacts in {}",
                native.label,
                native.frame_index,
                native.current_level_id,
                native.player_1_level_id,
                oracle.current_level_id,
                oracle.player_1_level_id,
                run_dir.display()
            );
        }

        let camera_delta_x = native.camera_x_pos.abs_diff(oracle.camera_x_pos);
        let camera_delta_y = native.camera_y_pos.abs_diff(oracle.camera_y_pos);
        if camera_delta_x > CAMERA_POSITION_TOLERANCE || camera_delta_y > CAMERA_POSITION_TOLERANCE
        {
            bail!(
                "state {}@{} diverged in camera position (native {:08X},{:08X}; oracle {:08X},{:08X}); artefacts in {}",
                native.label,
                native.frame_index,
                native.camera_x_pos,
                native.camera_y_pos,
                oracle.camera_x_pos,
                oracle.camera_y_pos,
                run_dir.display()
            );
        }
        if native.number_objects.abs_diff(oracle.number_objects) > 1
            || native
                .number_gfx_objects
                .abs_diff(oracle.number_gfx_objects)
                > 1
        {
            bail!(
                "state {}@{} diverged in object counts (native {} / {}, oracle {} / {}); artefacts in {}",
                native.label,
                native.frame_index,
                native.number_objects,
                native.number_gfx_objects,
                oracle.number_objects,
                oracle.number_gfx_objects,
                run_dir.display()
            );
        }
    }

    Ok(())
}

fn build_reference_rom(repo_root: &Path, upstream_dir: &Path) -> Result<()> {
    let bundled = Command::new("sh")
        .arg("build.sh")
        .current_dir(upstream_dir)
        .status();
    if let Ok(status) = bundled
        && status.success()
        && upstream_dir.join("kid_built.bin").is_file()
    {
        return Ok(());
    }

    let native_tools = ensure_native_asl_tools(repo_root)?;
    run_command(
        Command::new(&native_tools.asl)
            .env("AS_MSGPATH", &native_tools.root)
            .arg("-i")
            .arg(".")
            .arg("-xx")
            .arg("-c")
            .arg("-A")
            .arg("-U")
            .arg("kid.asm"),
        Some(upstream_dir),
    )?;
    run_command(
        Command::new(&native_tools.p2bin)
            .arg("-l")
            .arg("0")
            .arg("-r")
            .arg("0x-0x")
            .arg("kid.p")
            .arg("kid_built.bin"),
        Some(upstream_dir),
    )?;
    Ok(())
}

fn ensure_native_asl_tools(repo_root: &Path) -> Result<NativeAslTools> {
    let source_dir = repo_root.join(".cache/tooling/asl-releases");
    if !source_dir.exists() {
        let parent = source_dir
            .parent()
            .ok_or_else(|| anyhow!("tooling directory has no parent: {}", source_dir.display()))?;
        fs::create_dir_all(parent)
            .with_context(|| format!("failed to create {}", parent.display()))?;
        run_command(
            Command::new("git")
                .arg("clone")
                .arg("--depth")
                .arg("1")
                .arg(ASL_REPO_URL)
                .arg(&source_dir),
            None,
        )?;
    }

    run_command(
        Command::new("git")
            .arg("fetch")
            .arg("origin")
            .arg("upstream:upstream"),
        Some(&source_dir),
    )?;
    run_command(
        Command::new("git").arg("checkout").arg("upstream"),
        Some(&source_dir),
    )?;

    let makefile = capture_stdout(
        Command::new("git")
            .arg("show")
            .arg("origin/main:Makefile.def")
            .current_dir(&source_dir),
    )
    .context("failed to load ASL Makefile.def from origin/main")?;
    fs::write(source_dir.join("Makefile.def"), makefile)
        .with_context(|| format!("failed to write {}/Makefile.def", source_dir.display()))?;

    let asl = source_dir.join("asl");
    let p2bin = source_dir.join("p2bin");
    if !asl.is_file() || !p2bin.is_file() {
        run_command(Command::new("make").arg("-j4"), Some(&source_dir))?;
    }

    Ok(NativeAslTools {
        root: source_dir,
        asl,
        p2bin,
    })
}

fn capture_stdout(command: &mut Command) -> Result<Vec<u8>> {
    let output = command.stderr(Stdio::inherit()).output().with_context(|| {
        format!(
            "failed to execute {}",
            render_command(command, Path::new("."))
        )
    })?;
    if !output.status.success() {
        bail!(
            "command failed: {}",
            render_command(command, Path::new("."))
        );
    }
    Ok(output.stdout)
}

fn run_command(command: &mut Command, cwd: Option<&Path>) -> Result<()> {
    if let Some(cwd) = cwd {
        command.current_dir(cwd);
    }

    command.stdin(Stdio::null());
    command.stdout(Stdio::inherit());
    command.stderr(Stdio::inherit());

    let status = command.status().with_context(|| {
        format!(
            "failed to execute command: {}",
            render_command(command, cwd.unwrap_or_else(|| Path::new(".")))
        )
    })?;

    if !status.success() {
        bail!(
            "command failed: {}",
            render_command(command, cwd.unwrap_or_else(|| Path::new(".")))
        );
    }
    Ok(())
}

struct NativeAslTools {
    root: PathBuf,
    asl: PathBuf,
    p2bin: PathBuf,
}

struct PrepareLock {
    path: PathBuf,
}

impl Drop for PrepareLock {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.path);
    }
}

fn verify_boot_probe(probe: &kid_md::CpuProbe) -> Result<()> {
    let expected_vdp_prefix = [0x8004, 0x8114, 0x8230, 0x833C];
    if probe.reset_sp != 0xFFFF_F7FE {
        bail!("unexpected reset SP: 0x{:08X}", probe.reset_sp);
    }
    if probe.reset_pc != 0x0000_04B0 {
        bail!("unexpected reset PC: 0x{:08X}", probe.reset_pc);
    }
    if probe.vdp_control_writes.len() < expected_vdp_prefix.len() {
        bail!("not enough VDP control writes observed during boot");
    }
    if probe.vdp_control_writes[..expected_vdp_prefix.len()] != expected_vdp_prefix {
        bail!(
            "unexpected VDP init prefix: {:?}",
            &probe.vdp_control_writes[..expected_vdp_prefix.len()]
        );
    }
    let saw_port_a = probe
        .accesses
        .iter()
        .any(|access| access.address == 0x00A1_0008 && access.value == 0);
    let saw_port_b = probe
        .accesses
        .iter()
        .any(|access| access.address == 0x00A1_000A && access.value == 0);
    let saw_port_c = probe
        .accesses
        .iter()
        .any(|access| access.address == 0x00A1_000C && access.value == 0);
    if !(saw_port_a && saw_port_b && saw_port_c) {
        bail!("expected cold-boot port control reads were not observed");
    }
    Ok(())
}

fn verify_startup_probe(probe: &kid_md::CpuProbe) -> Result<()> {
    verify_boot_probe(probe)?;

    if probe.final_pc < 0x0000_0700 {
        bail!(
            "startup probe did not reach the later startup path; final PC was 0x{:08X}",
            probe.final_pc
        );
    }

    let expected_game_program_writes = [0x8104, 0x8F02, 0xC000, 0x0000, 0x4000, 0x0010];
    if !contains_u16_subsequence(&probe.vdp_control_writes, &expected_game_program_writes) {
        bail!(
            "startup probe missed the expected GameProgram VDP setup writes: {}",
            format_u16_samples(&probe.vdp_control_writes)
        );
    }

    if probe.synthetic_vblanks == 0 || probe.v_int_counter == 0 {
        bail!(
            "startup probe never serviced a synthetic VBlank interrupt (synthetic_vblanks={}, v_int_counter={})",
            probe.synthetic_vblanks,
            probe.v_int_counter
        );
    }

    Ok(())
}

fn verify_level_probe(rom_bytes: &[u8], steps: u32) -> Result<()> {
    let entry_probe = probe_rom_with_script(rom_bytes, steps, &gameplay_entry_script()?)?;
    if entry_probe.game_mode != 0x000C {
        bail!(
            "expected in-level game mode 0x000C after scripted title skips, got 0x{:04X}",
            entry_probe.game_mode
        );
    }
    if entry_probe.camera_y_pos == 0 {
        bail!("expected non-zero level camera Y after gameplay entry");
    }
    if entry_probe.number_gfx_objects == 0 {
        bail!("expected active gameplay gfx objects after level entry");
    }

    let movement_probe = probe_rom_with_script(rom_bytes, steps, &gameplay_move_right_script()?)?;
    if movement_probe.game_mode != 0x000C {
        bail!(
            "expected to remain in level mode during scripted movement, got 0x{:04X}",
            movement_probe.game_mode
        );
    }
    if movement_probe.camera_x_pos <= entry_probe.camera_x_pos {
        bail!(
            "expected camera X to advance during scripted movement, got baseline 0x{:08X} and moved 0x{:08X}",
            entry_probe.camera_x_pos,
            movement_probe.camera_x_pos
        );
    }

    Ok(())
}

fn verify_gameplay_scenario(
    repo_root: &Path,
    rom_bytes: &[u8],
    scenario: GameplayScenario,
) -> Result<()> {
    let capture = capture_gameplay_scenario(rom_bytes, scenario)?;
    write_gameplay_artifacts(repo_root, &capture)?;
    match scenario {
        GameplayScenario::PauseResumeLevel1 => verify_pause_resume_level_1(&capture),
        GameplayScenario::DeathRespawnLevel1 => verify_death_respawn_level_1(&capture),
        GameplayScenario::ProgressTwoTransitions => verify_progress_two_transitions(&capture),
        GameplayScenario::SoakLevel10m => verify_soak_level_10m(&capture),
    }
}

fn write_gameplay_artifacts(repo_root: &Path, capture: &GameplayCapture) -> Result<()> {
    let out_dir = repo_root
        .join(".cache/gameplay-runs")
        .join(&capture.scenario);
    fs::create_dir_all(&out_dir)
        .with_context(|| format!("failed to create {}", out_dir.display()))?;
    fs::write(
        out_dir.join("summary.json"),
        serde_json::to_vec_pretty(capture)?,
    )
    .with_context(|| format!("failed to write {}", out_dir.join("summary.json").display()))?;
    fs::write(
        out_dir.join("checkpoints.json"),
        serde_json::to_vec_pretty(&capture.checkpoints)?,
    )
    .with_context(|| {
        format!(
            "failed to write {}",
            out_dir.join("checkpoints.json").display()
        )
    })?;
    Ok(())
}

fn verify_pause_resume_level_1(capture: &GameplayCapture) -> Result<()> {
    const PAUSE_DRIFT_TOLERANCE: u32 = 0x0010_0000;

    if capture.missed_vblank_frames != 0 {
        bail!(
            "pause_resume_level1 missed {} vblank frames; artefacts in .cache/gameplay-runs/{}",
            capture.missed_vblank_frames,
            capture.scenario
        );
    }
    let level_settle = gameplay_checkpoint(capture, "level_settle")?;
    let pre_pause = gameplay_checkpoint(capture, "pre_pause")?;
    let paused_tail = gameplay_checkpoint(capture, "paused_tail")?;
    let resumed_tail = gameplay_checkpoint(capture, "resumed_tail")?;

    if level_settle.game_mode != 0x000C || pre_pause.game_mode != 0x000C {
        bail!("pause_resume_level1 never stabilised in gameplay mode before pause");
    }
    if pre_pause.camera_x_pos <= level_settle.camera_x_pos
        || pre_pause.kid_x_pos <= level_settle.kid_x_pos
    {
        bail!("pause_resume_level1 did not advance before the pause point");
    }
    if paused_tail.camera_x_pos.abs_diff(pre_pause.camera_x_pos) > PAUSE_DRIFT_TOLERANCE
        || paused_tail.kid_x_pos.abs_diff(pre_pause.kid_x_pos) > PAUSE_DRIFT_TOLERANCE
    {
        bail!("pause_resume_level1 kept advancing while paused");
    }
    if resumed_tail.camera_x_pos <= paused_tail.camera_x_pos
        || resumed_tail.kid_x_pos <= paused_tail.kid_x_pos
    {
        bail!("pause_resume_level1 did not resume movement after unpausing");
    }

    Ok(())
}

fn verify_death_respawn_level_1(capture: &GameplayCapture) -> Result<()> {
    if capture.missed_vblank_frames != 0 {
        bail!(
            "death_respawn_level1 missed {} vblank frames; artefacts in .cache/gameplay-runs/{}",
            capture.missed_vblank_frames,
            capture.scenario
        );
    }
    let level_settle = gameplay_checkpoint(capture, "level_settle")?;
    let pre_death = gameplay_checkpoint(capture, "pre_death")?;
    let death_tail = gameplay_checkpoint(capture, "death_tail")?;
    let respawn_tail = gameplay_checkpoint(capture, "respawn_tail")?;
    if level_settle.game_mode != 0x000C || pre_death.game_mode != 0x000C {
        bail!("death_respawn_level1 never stabilised in gameplay before the death window");
    }
    if death_tail.number_lives >= pre_death.number_lives {
        bail!(
            "death_respawn_level1 did not lose a life (before {}, after {})",
            pre_death.number_lives,
            death_tail.number_lives
        );
    }
    if respawn_tail.game_mode != 0x000C {
        bail!(
            "death_respawn_level1 did not return to gameplay mode after death: 0x{:04X}",
            respawn_tail.game_mode
        );
    }
    if respawn_tail.camera_x_pos == 0 && respawn_tail.kid_x_pos == 0 {
        bail!("death_respawn_level1 respawned into an invalid position");
    }

    Ok(())
}

fn verify_progress_two_transitions(capture: &GameplayCapture) -> Result<()> {
    if capture.missed_vblank_frames != 0 {
        bail!(
            "progress_two_transitions missed {} vblank frames; artefacts in .cache/gameplay-runs/{}",
            capture.missed_vblank_frames,
            capture.scenario
        );
    }
    let level_settle = gameplay_checkpoint(capture, "level_settle")?;
    let run_end = gameplay_checkpoint(capture, "run_end")?;
    if level_settle.game_mode != 0x000C {
        bail!("progress_two_transitions never reached stable in-level gameplay");
    }
    let distinct_levels = distinct_level_sequence(&capture.states);
    if distinct_levels.len() < 3 {
        bail!(
            "progress_two_transitions saw only {:?}; expected at least three distinct level ids",
            distinct_levels
        );
    }
    if run_end.number_objects == 0 || run_end.number_gfx_objects == 0 {
        bail!("progress_two_transitions ended without active gameplay objects");
    }
    if run_end.number_lives == 0 {
        bail!("progress_two_transitions exhausted all lives before the run finished");
    }
    if run_end.current_level_id != *distinct_levels.last().unwrap_or(&run_end.current_level_id) {
        bail!(
            "progress_two_transitions ended on level {:04X} after visiting {:?}",
            run_end.current_level_id,
            distinct_levels
        );
    }

    Ok(())
}

fn verify_soak_level_10m(capture: &GameplayCapture) -> Result<()> {
    if capture.missed_vblank_frames != 0 {
        bail!(
            "soak_level_10m missed {} vblank frames; artefacts in .cache/gameplay-runs/{}",
            capture.missed_vblank_frames,
            capture.scenario
        );
    }
    let soak_end = gameplay_checkpoint(capture, "soak_end")?;
    if gameplay_checkpoint(capture, "level_settle")?.game_mode != 0x000C {
        bail!("soak_level_10m never reached stable in-level gameplay");
    }
    if soak_end.game_mode == 0 || soak_end.number_objects == 0 || soak_end.number_gfx_objects == 0 {
        bail!("soak_level_10m ended in an invalid gameplay state");
    }
    if soak_end.number_lives == 0 {
        bail!("soak_level_10m exhausted all lives");
    }
    let distinct_levels = distinct_level_sequence(&capture.states);
    if distinct_levels.len() < 8 {
        bail!(
            "soak_level_10m visited only {:?}; expected broad transition coverage",
            distinct_levels
        );
    }

    Ok(())
}

fn gameplay_checkpoint<'a>(capture: &'a GameplayCapture, label: &str) -> Result<&'a GameplayState> {
    capture
        .checkpoints
        .iter()
        .find(|state| state.label == label)
        .ok_or_else(|| anyhow!("missing gameplay checkpoint {label}"))
}

fn distinct_level_sequence(states: &[GameplayState]) -> Vec<u16> {
    let mut levels = Vec::new();
    for state in states {
        if levels.last().copied() != Some(state.current_level_id) {
            levels.push(state.current_level_id);
        }
    }
    levels
}

fn verify_audio_driver_probe(rom_bytes: &[u8], steps: u32) -> Result<()> {
    let probe = probe_rom_with_script(rom_bytes, steps, &gameplay_entry_script()?)?;
    if probe.ym2612_write_count == 0 {
        bail!("expected the Z80 sound path to issue YM2612 writes by level entry");
    }
    if probe.psg_write_count == 0 {
        bail!("expected the Z80 sound path to issue PSG writes by level entry");
    }

    Ok(())
}

struct OracleComparison {
    run_dir: PathBuf,
    native: ScenarioCapture,
    oracle: OracleManifest,
}

fn print_oracle_environment(env: &OracleEnvironment) {
    println!("jgenesis repo: {}", env.jgenesis_dir.display());
    println!("jgenesis helper: {}", env.jgenesis_helper_bin.display());
    println!("blastem repo: {}", env.blastem_dir.display());
    match &env.blastem_binary {
        Some(path) => println!("blastem binary: {}", path.display()),
        None => println!(
            "blastem binary: not built (install libsdl2-dev libglew-dev libgl1-mesa-dev to enable local builds)"
        ),
    }
}

fn verify_audio_output(
    repo_root: &Path,
    rom_path: &Path,
    rom_bytes: &[u8],
    scenario: Scenario,
) -> Result<()> {
    let comparison = capture_oracle_pair(repo_root, rom_path, rom_bytes, scenario)?;
    compare_audio_against_oracle(&comparison)
}

fn trace_audio(repo_root: &Path, rom_bytes: &[u8], scenario: Scenario) -> Result<()> {
    let run_dir = repo_root.join(".cache/oracle-runs").join(scenario.slug());
    let native_dir = run_dir.join("native");
    fs::create_dir_all(&native_dir)
        .with_context(|| format!("failed to create {}", native_dir.display()))?;
    let capture = capture_scenario(rom_bytes, scenario)?;
    write_native_artifacts(&native_dir, &capture)?;
    let trace_path = native_dir.join("audio-trace.json");
    println!("wrote audio trace to {}", trace_path.display());
    println!("z80 reset count: {}", capture.audio_trace.z80_reset_count);
    if let Some(onset) = capture.audio_trace.first_significant_audio_sample {
        println!(
            "first significant audio sample: {} ({:.3}s)",
            onset,
            onset as f32 / AUDIO_SAMPLE_RATE as f32
        );
    } else {
        println!("first significant audio sample: none");
    }
    println!(
        "first YM2612 write frame/sample: {:?}/{:?}",
        capture.audio_trace.first_ym2612_write_frame, capture.audio_trace.first_ym2612_write_sample
    );
    println!(
        "first YM2612 key-on frame/sample: {:?}/{:?}",
        capture.audio_trace.first_ym2612_key_on_frame,
        capture.audio_trace.first_ym2612_key_on_sample
    );
    println!(
        "first PSG write frame/sample: {:?}/{:?}",
        capture.audio_trace.first_psg_write_frame, capture.audio_trace.first_psg_write_sample
    );
    Ok(())
}

fn verify_frames(
    repo_root: &Path,
    rom_path: &Path,
    rom_bytes: &[u8],
    scenario: Scenario,
) -> Result<()> {
    let comparison = capture_oracle_pair(repo_root, rom_path, rom_bytes, scenario)?;
    compare_frames_against_oracle(&comparison)
}

fn verify_oracle(
    repo_root: &Path,
    rom_path: &Path,
    rom_bytes: &[u8],
    scenario: Scenario,
) -> Result<()> {
    let comparison = capture_oracle_pair(repo_root, rom_path, rom_bytes, scenario)?;
    compare_audio_against_oracle(&comparison)?;
    compare_frames_against_oracle(&comparison)
}

fn capture_oracle_pair(
    repo_root: &Path,
    rom_path: &Path,
    rom_bytes: &[u8],
    scenario: Scenario,
) -> Result<OracleComparison> {
    let env = ensure_oracles(repo_root)?;
    let run_dir = repo_root.join(".cache/oracle-runs").join(scenario.slug());
    let native_dir = run_dir.join("native");
    let oracle_dir = run_dir.join("oracle");
    if native_dir.exists() {
        fs::remove_dir_all(&native_dir)
            .with_context(|| format!("failed to remove {}", native_dir.display()))?;
    }
    if oracle_dir.exists() {
        fs::remove_dir_all(&oracle_dir)
            .with_context(|| format!("failed to remove {}", oracle_dir.display()))?;
    }
    fs::create_dir_all(&native_dir)
        .with_context(|| format!("failed to create {}", native_dir.display()))?;
    fs::create_dir_all(&oracle_dir)
        .with_context(|| format!("failed to create {}", oracle_dir.display()))?;

    let native = capture_scenario(rom_bytes, scenario)?;
    write_native_artifacts(&native_dir, &native)?;
    let oracle = run_jgenesis_capture(&env, rom_path, scenario, &oracle_dir)?;
    fs::write(
        oracle_dir.join("manifest.json"),
        serde_json::to_vec_pretty(&oracle)?,
    )
    .with_context(|| {
        format!(
            "failed to write {}",
            oracle_dir.join("manifest.json").display()
        )
    })?;

    Ok(OracleComparison {
        run_dir,
        native,
        oracle,
    })
}

fn compare_audio_against_oracle(comparison: &OracleComparison) -> Result<()> {
    let native_analysis = analyse_audio_samples(&comparison.native.samples, AUDIO_SAMPLE_RATE);
    let oracle_samples = read_wav_f32(&comparison.oracle.audio_path)?;
    let oracle_analysis = analyse_audio_samples(&oracle_samples, comparison.oracle.sample_rate);
    let trace_path = comparison.run_dir.join("native/audio-trace.json");

    if native_analysis.change_count_left == 0 && native_analysis.change_count_right == 0 {
        bail!(
            "native audio remained constant across {}; trace {}",
            comparison.run_dir.display(),
            trace_path.display()
        );
    }
    if native_analysis.peak_left < 0.01 && native_analysis.peak_right < 0.01 {
        bail!(
            "native audio peak was too low (left {:.4}, right {:.4}); trace {}",
            native_analysis.peak_left,
            native_analysis.peak_right,
            trace_path.display()
        );
    }
    let native_onset = native_analysis
        .first_dynamic_frame
        .ok_or_else(|| anyhow!("native audio never became dynamic"))?;
    let oracle_onset = oracle_analysis
        .first_dynamic_frame
        .ok_or_else(|| anyhow!("oracle audio never became dynamic"))?;
    let onset_delta_seconds =
        native_onset.abs_diff(oracle_onset) as f32 / comparison.oracle.sample_rate as f32;
    if onset_delta_seconds > 0.5 {
        bail!(
            "audio onset drift was {:.3}s (native {:.3}s @ sample {}, oracle {:.3}s @ sample {}); native peaks {:.4}/{:.4}; artefacts in {}; trace {}",
            onset_delta_seconds,
            native_onset as f32 / AUDIO_SAMPLE_RATE as f32,
            native_onset,
            oracle_onset as f32 / comparison.oracle.sample_rate as f32,
            oracle_onset,
            native_analysis.peak_left,
            native_analysis.peak_right,
            comparison.run_dir.display(),
            trace_path.display()
        );
    }
    if native_analysis.tail_change_count_left == 0 && native_analysis.tail_change_count_right == 0 {
        bail!(
            "native audio tail stayed constant; artefacts in {}; trace {}",
            comparison.run_dir.display(),
            trace_path.display()
        );
    }
    let similarity = audio_envelope_similarity(
        &native_analysis.rms_windows,
        &oracle_analysis.rms_windows,
        native_onset,
        oracle_onset,
        comparison.oracle.sample_rate,
    );
    if similarity < 0.90 {
        bail!(
            "audio envelope similarity {:.3} was below 0.90; native peaks {:.4}/{:.4}; artefacts in {}; trace {}",
            similarity,
            native_analysis.peak_left,
            native_analysis.peak_right,
            comparison.run_dir.display(),
            trace_path.display()
        );
    }

    Ok(())
}

fn compare_frames_against_oracle(comparison: &OracleComparison) -> Result<()> {
    compare_states_against_oracle(comparison)?;

    if comparison.native.frames.len() != comparison.oracle.frames.len() {
        bail!(
            "native captured {} frames but oracle captured {}; artefacts in {}",
            comparison.native.frames.len(),
            comparison.oracle.frames.len(),
            comparison.run_dir.display()
        );
    }

    for (native, oracle) in comparison
        .native
        .frames
        .iter()
        .zip(&comparison.oracle.frames)
    {
        if native.label != oracle.label || native.frame_index != oracle.frame_index {
            bail!(
                "frame checkpoint mismatch: native {}@{} vs oracle {}@{}",
                native.label,
                native.frame_index,
                oracle.label,
                oracle.frame_index
            );
        }
        if native.label == "level_entry" {
            continue;
        }
        let oracle_rgb = load_png_rgb(&oracle.png_path)?;
        let diff = compare_rgb_frames(&native.rgb, &oracle_rgb)?;
        let mae_limit = match native.label.as_str() {
            "title" => 16.0,
            "level_entry" | "level_settle" => 28.0,
            _ => 32.0,
        };
        if diff.mean_absolute_error > mae_limit {
            bail!(
                "frame {}@{} diverged too far (MAE {:.2}, max delta {}, native {}, oracle {}); artefacts in {}",
                native.label,
                native.frame_index,
                diff.mean_absolute_error,
                diff.max_channel_delta,
                native.sha1,
                oracle.sha1,
                comparison.run_dir.display()
            );
        }
    }

    Ok(())
}

fn compare_states_against_oracle(comparison: &OracleComparison) -> Result<()> {
    if comparison.native.states.len() != comparison.oracle.states.len() {
        bail!(
            "native captured {} states but oracle captured {}; artefacts in {}",
            comparison.native.states.len(),
            comparison.oracle.states.len(),
            comparison.run_dir.display()
        );
    }

    for (native, oracle) in comparison
        .native
        .states
        .iter()
        .zip(&comparison.oracle.states)
    {
        if native.label != oracle.label || native.frame_index != oracle.frame_index {
            bail!(
                "state checkpoint mismatch: native {}@{} vs oracle {}@{}",
                native.label,
                native.frame_index,
                oracle.label,
                oracle.frame_index
            );
        }
        if native.game_mode != oracle.game_mode {
            bail!(
                "state {}@{} diverged in game mode (native {:04X}, oracle {:04X}); artefacts in {}",
                native.label,
                native.frame_index,
                native.game_mode,
                oracle.game_mode,
                comparison.run_dir.display()
            );
        }
        if native.current_player != oracle.current_player {
            bail!(
                "state {}@{} diverged in current player (native {:02X}, oracle {:02X}); artefacts in {}",
                native.label,
                native.frame_index,
                native.current_player,
                oracle.current_player,
                comparison.run_dir.display()
            );
        }
        if native.current_level_id != oracle.current_level_id
            || native.player_1_level_id != oracle.player_1_level_id
        {
            bail!(
                "state {}@{} diverged in level ids (native current {:04X} / p1 {:04X}, oracle current {:04X} / p1 {:04X}); artefacts in {}",
                native.label,
                native.frame_index,
                native.current_level_id,
                native.player_1_level_id,
                oracle.current_level_id,
                oracle.player_1_level_id,
                comparison.run_dir.display()
            );
        }
        if native.label == "level_entry" {
            continue;
        }
        let camera_delta_x = native.camera_x_pos.abs_diff(oracle.camera_x_pos);
        let camera_delta_y = native.camera_y_pos.abs_diff(oracle.camera_y_pos);
        if camera_delta_x > CAMERA_POSITION_TOLERANCE || camera_delta_y > CAMERA_POSITION_TOLERANCE
        {
            bail!(
                "state {}@{} diverged in camera position (native {:08X},{:08X}; oracle {:08X},{:08X}); artefacts in {}",
                native.label,
                native.frame_index,
                native.camera_x_pos,
                native.camera_y_pos,
                oracle.camera_x_pos,
                oracle.camera_y_pos,
                comparison.run_dir.display()
            );
        }
        if native.number_objects.abs_diff(oracle.number_objects) > 8
            || native
                .number_gfx_objects
                .abs_diff(oracle.number_gfx_objects)
                > 8
        {
            bail!(
                "state {}@{} diverged in object counts (native {} / {}, oracle {} / {}); artefacts in {}",
                native.label,
                native.frame_index,
                native.number_objects,
                native.number_gfx_objects,
                oracle.number_objects,
                oracle.number_gfx_objects,
                comparison.run_dir.display()
            );
        }
    }

    Ok(())
}

fn write_native_artifacts(out_dir: &Path, capture: &ScenarioCapture) -> Result<()> {
    write_wav(
        &out_dir.join("audio.wav"),
        AUDIO_SAMPLE_RATE,
        &capture.samples,
    )?;
    fs::write(
        out_dir.join("audio-trace.json"),
        serde_json::to_vec_pretty(&capture.audio_trace)?,
    )
    .with_context(|| {
        format!(
            "failed to write {}",
            out_dir.join("audio-trace.json").display()
        )
    })?;
    fs::write(
        out_dir.join("state.json"),
        serde_json::to_vec_pretty(
            &capture
                .states
                .iter()
                .map(native_state_to_oracle_state)
                .collect::<Vec<_>>(),
        )?,
    )
    .with_context(|| format!("failed to write {}", out_dir.join("state.json").display()))?;
    for frame in &capture.frames {
        let path = out_dir.join(format!("{:04}_{}.png", frame.frame_index, frame.label));
        image::save_buffer_with_format(
            &path,
            &frame.rgb,
            frame.width as u32,
            frame.height as u32,
            ColorType::Rgb8,
            ImageFormat::Png,
        )
        .with_context(|| format!("failed to write {}", path.display()))?;
    }
    Ok(())
}

fn native_state_to_oracle_state(state: &CapturedState) -> OracleState {
    OracleState {
        label: state.label.clone(),
        frame_index: state.frame_index,
        game_mode: state.game_mode,
        current_player: state.current_player,
        current_level_id: state.current_level_id,
        player_1_level_id: state.player_1_level_id,
        camera_x_pos: state.camera_x_pos,
        camera_y_pos: state.camera_y_pos,
        number_objects: state.number_objects,
        number_gfx_objects: state.number_gfx_objects,
    }
}

fn load_png_rgb(path: &Path) -> Result<Vec<u8>> {
    let image = image::open(path)
        .with_context(|| format!("failed to open oracle frame {}", path.display()))?;
    Ok(image.to_rgb8().into_raw())
}

fn compare_rgb_frames(native_rgb: &[u8], oracle_rgb: &[u8]) -> Result<FrameDiff> {
    if native_rgb.len() != oracle_rgb.len() {
        bail!(
            "frame size mismatch: native {} bytes vs oracle {} bytes",
            native_rgb.len(),
            oracle_rgb.len()
        );
    }

    let mut total_delta = 0u64;
    let mut max_channel_delta = 0u8;
    for (native, oracle) in native_rgb.iter().zip(oracle_rgb) {
        let delta = native.abs_diff(*oracle);
        total_delta += u64::from(delta);
        max_channel_delta = max_channel_delta.max(delta);
    }
    Ok(FrameDiff {
        mean_absolute_error: total_delta as f32 / native_rgb.len() as f32,
        max_channel_delta,
    })
}

fn audio_envelope_similarity(
    native: &[f32],
    oracle: &[f32],
    native_onset_frame: usize,
    oracle_onset_frame: usize,
    sample_rate: u32,
) -> f32 {
    let window_frames = (sample_rate / 4).max(1) as usize;
    let native_start = native_onset_frame / window_frames;
    let oracle_start = oracle_onset_frame / window_frames;
    let mut best_similarity = 0.0f32;

    for lag in -1isize..=1 {
        let (shifted_native_start, shifted_oracle_start) = if lag >= 0 {
            (native_start.saturating_add(lag as usize), oracle_start)
        } else {
            (native_start, oracle_start.saturating_add((-lag) as usize))
        };
        let overlapping = native
            .len()
            .saturating_sub(shifted_native_start)
            .min(oracle.len().saturating_sub(shifted_oracle_start));
        if overlapping < 2 {
            continue;
        }

        let native_slice = &native[shifted_native_start..shifted_native_start + overlapping];
        let oracle_slice = &oracle[shifted_oracle_start..shifted_oracle_start + overlapping];
        let mut numerator = 0.0f32;
        let mut native_energy = 0.0f32;
        let mut oracle_energy = 0.0f32;
        for (native_value, oracle_value) in native_slice.iter().zip(oracle_slice) {
            numerator += native_value * oracle_value;
            native_energy += native_value * native_value;
            oracle_energy += oracle_value * oracle_value;
        }
        if native_energy <= f32::EPSILON || oracle_energy <= f32::EPSILON {
            continue;
        }
        let similarity = numerator / (native_energy.sqrt() * oracle_energy.sqrt());
        best_similarity = best_similarity.max(similarity);
    }

    best_similarity
}

fn contains_u16_subsequence(haystack: &[u16], needle: &[u16]) -> bool {
    if needle.is_empty() {
        return true;
    }

    haystack
        .windows(needle.len())
        .any(|window| window == needle)
}

fn render_command(command: &Command, cwd: &Path) -> String {
    let program = command.get_program().to_string_lossy();
    let args = command
        .get_args()
        .map(|arg| arg.to_string_lossy().into_owned())
        .collect::<Vec<_>>()
        .join(" ");
    format!("(cd {} && {} {})", cwd.display(), program, args)
}

#[cfg(test)]
mod tests {
    use super::{
        analyse_audio_samples, audio_envelope_similarity, build_controller_script,
        parse_button_mask, parse_input_spec,
    };

    #[test]
    fn parses_single_frame_input_spec() {
        let range = parse_input_spec("460:start").expect("input spec should parse");
        assert_eq!(range.start, 460);
        assert_eq!(range.end, 460);
        assert_eq!(range.buttons, 0x80);
    }

    #[test]
    fn parses_ranged_input_spec() {
        let range = parse_input_spec("500-650:right+b").expect("input spec should parse");
        assert_eq!(range.start, 500);
        assert_eq!(range.end, 650);
        assert_eq!(range.buttons, (1 << 3) | (1 << 4));
    }

    #[test]
    fn parses_button_masks() {
        assert_eq!(
            parse_button_mask("up+start").expect("mask should parse"),
            0x81
        );
        assert_eq!(
            parse_button_mask("0x88").expect("raw mask should parse"),
            0x88
        );
        assert_eq!(parse_button_mask("none").expect("none should parse"), 0);
    }

    #[test]
    fn builds_combined_controller_script() {
        let script = build_controller_script(&[120], &[String::from("122-123:right")])
            .expect("script should build");
        assert_eq!(script[120], 0x80);
        assert_eq!(script[122], 1 << 3);
        assert_eq!(script[123], 1 << 3);
    }

    #[test]
    fn detects_constant_audio() {
        let analysis = analyse_audio_samples(&[0.1, 0.1, 0.1, 0.1, 0.1, 0.1], 44_100);
        assert_eq!(analysis.change_count_left, 0);
        assert_eq!(analysis.change_count_right, 0);
        assert_eq!(analysis.peak_left, 0.1);
        assert_eq!(analysis.peak_right, 0.1);
        assert_eq!(analysis.first_dynamic_frame, None);
    }

    #[test]
    fn detects_changing_audio() {
        let analysis = analyse_audio_samples(&[0.1, 0.1, 0.2, 0.1, 0.2, -0.3], 44_100);
        assert!(analysis.change_count_left > 0);
        assert!(analysis.change_count_right > 0);
        assert_eq!(analysis.peak_left, 0.2);
        assert_eq!(analysis.peak_right, 0.3);
        assert_eq!(analysis.first_dynamic_frame, None);
    }

    #[test]
    fn ignores_dc_offset_until_signal_activity() {
        let mut samples = Vec::new();
        for _ in 0..4_410 {
            samples.extend_from_slice(&[0.01, 0.01]);
        }
        for index in 0..4_410 {
            let sample = if index & 1 == 0 { 0.2 } else { -0.2 };
            samples.extend_from_slice(&[sample, -sample]);
        }

        let analysis = analyse_audio_samples(&samples, 44_100);
        assert_eq!(analysis.first_dynamic_frame, Some(4_410));
    }

    #[test]
    fn envelope_similarity_prefers_positive_energy_alignment() {
        let native = [0.06, 0.04, 0.05, 0.04, 0.06, 0.05];
        let oracle = [0.04, 0.06, 0.05, 0.05, 0.05, 0.04];

        let similarity = audio_envelope_similarity(&native, &oracle, 0, 0, 4);
        assert!(
            similarity > 0.95,
            "expected high similarity for comparable positive envelopes, got {similarity}"
        );
    }
}
