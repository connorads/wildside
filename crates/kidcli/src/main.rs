mod autonomy;
mod bootstrap;
mod cache;
mod capture;
mod cheats;
mod compare;
mod diff;
mod inspect;
mod native_player;
mod oracle;
mod player;
mod scenarios;
mod sweep;
mod verify;

use verify::*;

use bootstrap::*;
use capture::*;
use compare::*;
use inspect::*;

use anyhow::{Context, Result, anyhow, bail};
use autonomy::{AutonomyGoal, run_autonomous_loop};
use clap::{Parser, Subcommand};
use image::{ColorType, ImageFormat};
use kid_core::physics::Direction as NativeDirection;
use kid_core::{GameMode as NativeGameMode, NativeTraceFrame, run_script};
use kid_data::{AssetManifest, find_enemy_definition, load_level_asset_bundle, resource_level_id};
use kid_md::{MdRuntime, probe_rom, probe_rom_with_script, render_frame};
use kid_rom::{RomIdentity, VerifiedRom, find_default_rom, known_sha1, verify_rom};
use native_player::{NativePlayOptions, render_gameplay_frame_evidence, run_native_player};
use oracle::{
    EnemyFacing, OracleEnemyLayoutTraceManifest, OracleEnemyState, OracleEnvironment,
    OracleManifest, OracleState, ensure_oracles, run_jgenesis_capture,
    run_jgenesis_enemy_layout_trace,
};
use player::{PlayOptions, run_player};
use scenarios::{GameplayScenario, Scenario};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::thread;
use std::time::{Duration, Instant};
use ymfm_sys::Ym2612;

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
    /// Launch the emulator-backed player shell.
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
    /// Launch the native Level 1 playtest runtime in release mode.
    PlayNative {
        #[arg(long)]
        rom: Option<PathBuf>,
        #[arg(long)]
        dir: Option<PathBuf>,
        #[arg(long, default_value_t = false)]
        fullscreen: bool,
        #[arg(long)]
        scale: Option<u32>,
        #[arg(long, default_value_t = false)]
        debug_overlay: bool,
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
    VerifyNativeGameplay {
        #[arg(long)]
        rom: Option<PathBuf>,
        #[arg(long)]
        dir: Option<PathBuf>,
        #[arg(long, value_enum, default_value_t = GameplayScenario::PauseResumeLevel1)]
        scenario: GameplayScenario,
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
    BootstrapReferences {
        #[arg(long, value_enum)]
        only: Option<ReferenceName>,
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
    ProbeNativeGameplay {
        #[arg(long)]
        rom: Option<PathBuf>,
        #[arg(long)]
        dir: Option<PathBuf>,
        #[arg(long, value_enum, default_value_t = GameplayScenario::PauseResumeLevel1)]
        scenario: GameplayScenario,
        #[arg(long)]
        frame_start: Option<usize>,
        #[arg(long)]
        frame_end: Option<usize>,
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
    TraceOracleEnemyLayout {
        #[arg(long)]
        rom: Option<PathBuf>,
        #[arg(long, value_enum, default_value_t = Scenario::BootTitleStartLevel1)]
        scenario: Scenario,
    },
    CompareNativeGameplay {
        #[arg(long)]
        rom: Option<PathBuf>,
        #[arg(long)]
        dir: Option<PathBuf>,
        #[arg(long, value_enum, default_value_t = GameplayScenario::PauseResumeLevel1)]
        scenario: GameplayScenario,
    },
    DiagnoseNativeGameplayDrift {
        #[arg(long)]
        rom: Option<PathBuf>,
        #[arg(long)]
        dir: Option<PathBuf>,
        #[arg(long, value_enum, default_value_t = GameplayScenario::PauseResumeLevel1)]
        scenario: GameplayScenario,
        #[arg(long, default_value_t = 719)]
        from_frame: usize,
        #[arg(long, default_value_t = 1)]
        limit: usize,
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
    AutonomousLoop {
        #[arg(long)]
        rom: Option<PathBuf>,
        #[arg(long)]
        dir: Option<PathBuf>,
        #[arg(long, value_enum, default_value_t = AutonomyGoal::Level1OpeningSlice)]
        goal: AutonomyGoal,
        #[arg(long, default_value_t = 1)]
        max_cycles: u32,
        #[arg(long, default_value_t = false)]
        no_resume: bool,
        #[arg(long, default_value_t = false)]
        emit_json: bool,
    },
    ReferenceBuild {
        #[arg(long)]
        rom: Option<PathBuf>,
        #[arg(long)]
        dir: Option<PathBuf>,
        #[arg(long, default_value_t = false)]
        allow_unknown_rom: bool,
    },
    /// Dump the Level_Layout collision grid from emulator RAM after level load.
    DumpLevelLayout {
        #[arg(long)]
        rom: Option<PathBuf>,
        /// Hex level ID (e.g. 00 for Blue Lake Woods 1)
        #[arg(long)]
        level: String,
        /// Output directory (default: .cache/generated/<sha1>/level-layout/)
        #[arg(long)]
        out: Option<PathBuf>,
    },
    /// Sweep levels for native vs oracle parity, producing a JSON report.
    Sweep {
        #[arg(long)]
        rom: Option<PathBuf>,
        #[arg(long)]
        dir: Option<PathBuf>,
        /// Level range: "0-104", "0,5,10", or "0-10"
        #[arg(long, default_value = "0-104")]
        levels: String,
        /// Collection tier: state, frames, or full
        #[arg(long, value_enum, default_value_t = sweep::SweepTier::State)]
        tier: sweep::SweepTier,
        /// Only populate oracle cache — skip native runs and comparison.
        #[arg(long, default_value_t = false)]
        warm_cache_only: bool,
        /// Stop after this many failures (0 = run all levels).
        #[arg(long, default_value_t = 0)]
        max_failures: usize,
        /// Capture golden state snapshots (no oracle needed).
        #[arg(long, default_value_t = false)]
        golden_capture: bool,
        /// Diff current state against golden snapshots (no oracle needed).
        #[arg(long, default_value_t = false)]
        golden_diff: bool,
    },
    /// Deep diagnosis of a single failing level.
    DiagnoseLevel {
        #[arg(long)]
        rom: Option<PathBuf>,
        #[arg(long)]
        dir: Option<PathBuf>,
        /// Hex level ID (e.g. 05)
        #[arg(long)]
        level: String,
    },
    /// Agent-driven parity loop: sweep → diagnose worst → report.
    ParityLoop {
        #[arg(long)]
        rom: Option<PathBuf>,
        #[arg(long)]
        dir: Option<PathBuf>,
        #[arg(long, default_value_t = 1)]
        max_cycles: u32,
    },
    /// Capture native gameplay frames at specified indices as PNGs.
    CaptureFrames {
        #[arg(long)]
        rom: Option<PathBuf>,
        #[arg(long)]
        dir: Option<PathBuf>,
        /// Hex level ID (e.g. 00)
        #[arg(long, default_value = "00")]
        level: String,
        /// Comma-separated frame indices to capture
        #[arg(long, default_value = "30,60,120,240,360")]
        frames: String,
        /// Output directory for PNGs
        #[arg(long, default_value = ".cache/captures")]
        output: PathBuf,
        /// Optional layer filter: only render this layer (background, terrain,
        /// block_overrides, platforms, enemies, player, effects, hud)
        #[arg(long)]
        layer: Option<String>,
    },
    /// Per-layer frame diagnosis: render each layer solo, report pixel contribution.
    DiagnoseFrame {
        #[arg(long)]
        rom: Option<PathBuf>,
        #[arg(long)]
        dir: Option<PathBuf>,
        /// Hex level ID (e.g. 00)
        #[arg(long, default_value = "00")]
        level: String,
        /// Frame index to diagnose (default: settle at frame 60)
        #[arg(long, default_value_t = 60)]
        frame: usize,
        /// Save per-layer PNGs alongside the report
        #[arg(long, default_value_t = false)]
        save_layers: bool,
        /// Output directory
        #[arg(long, default_value = ".cache/diagnose")]
        output: PathBuf,
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
        Commands::PlayNative {
            rom,
            dir,
            fullscreen,
            scale,
            debug_overlay,
        } => {
            let repo_root = current_dir()?;
            let rom_path = resolve_rom_path(&repo_root, rom)?;
            let upstream_dir = resolve_upstream_dir(&repo_root, dir);
            let manifest = prepare_generated_assets(&repo_root, &rom_path, &upstream_dir)?;
            run_native_player(
                &manifest,
                NativePlayOptions {
                    fullscreen,
                    scale,
                    debug_overlay,
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
        Commands::VerifyNativeGameplay { rom, dir, scenario } => {
            let repo_root = current_dir()?;
            let rom_path = resolve_rom_path(&repo_root, rom)?;
            let upstream_dir = resolve_upstream_dir(&repo_root, dir);
            let manifest = prepare_generated_assets(&repo_root, &rom_path, &upstream_dir)?;
            ensure_gameplay_level_layouts(&rom_path, &manifest, scenario)?;
            verify_native_gameplay_scenario(&repo_root, &manifest, scenario)?;
            println!("native gameplay verification passed: {}", scenario.slug());
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
        Commands::BootstrapReferences { only } => {
            let repo_root = current_dir()?;
            let reference_root = resolve_reference_root(&repo_root);
            let references = bootstrap_references(&reference_root, only)?;
            print_bootstrapped_references(&reference_root, &references);
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
        Commands::ProbeNativeGameplay {
            rom,
            dir,
            scenario,
            frame_start,
            frame_end,
        } => {
            let repo_root = current_dir()?;
            let rom_path = resolve_rom_path(&repo_root, rom)?;
            let upstream_dir = resolve_upstream_dir(&repo_root, dir);
            let manifest = prepare_generated_assets(&repo_root, &rom_path, &upstream_dir)?;
            ensure_gameplay_level_layouts(&rom_path, &manifest, scenario)?;
            probe_native_gameplay_scenario(&manifest, scenario, frame_start, frame_end)?;
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
        Commands::TraceOracleEnemyLayout { rom, scenario } => {
            let repo_root = current_dir()?;
            let rom_path = resolve_rom_path(&repo_root, rom)?;
            trace_oracle_enemy_layout(&repo_root, &rom_path, scenario)?;
        }
        Commands::CompareNativeGameplay { rom, dir, scenario } => {
            let repo_root = current_dir()?;
            let rom_path = resolve_rom_path(&repo_root, rom)?;
            let upstream_dir = resolve_upstream_dir(&repo_root, dir);
            let manifest = prepare_generated_assets(&repo_root, &rom_path, &upstream_dir)?;
            ensure_gameplay_level_layouts(&rom_path, &manifest, scenario)?;
            compare_native_gameplay(&repo_root, &rom_path, &manifest, scenario)?;
        }
        Commands::DiagnoseNativeGameplayDrift {
            rom,
            dir,
            scenario,
            from_frame,
            limit,
        } => {
            let repo_root = current_dir()?;
            let rom_path = resolve_rom_path(&repo_root, rom)?;
            let upstream_dir = resolve_upstream_dir(&repo_root, dir);
            let manifest = prepare_generated_assets(&repo_root, &rom_path, &upstream_dir)?;
            ensure_gameplay_level_layouts(&rom_path, &manifest, scenario)?;
            diagnose_native_gameplay_drift(
                &repo_root, &rom_path, &manifest, scenario, from_frame, limit,
            )?;
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
        Commands::AutonomousLoop {
            rom,
            dir,
            goal,
            max_cycles,
            no_resume,
            emit_json,
        } => {
            let repo_root = current_dir()?;
            let rom_path = resolve_rom_path(&repo_root, rom)?;
            let upstream_dir = resolve_upstream_dir(&repo_root, dir);
            run_autonomous_loop(
                &repo_root,
                &rom_path,
                &upstream_dir,
                goal,
                max_cycles,
                !no_resume,
                emit_json,
            )?;
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
        Commands::DumpLevelLayout { rom, level, out } => {
            let repo_root = current_dir()?;
            let rom_path = resolve_rom_path(&repo_root, rom)?;
            let rom_bytes = fs::read(&rom_path)
                .with_context(|| format!("failed to read ROM from {}", rom_path.display()))?;
            let level_id =
                u8::from_str_radix(&level, 16).context("level must be a hex byte (e.g. 00)")?;
            let out_dir = out.unwrap_or_else(|| {
                repo_root
                    .join(".cache/generated")
                    .join(&sha1_hex(&rom_bytes))
                    .join("level-layout")
            });
            dump_level_layout(&rom_bytes, level_id, &out_dir)?;
        }
        Commands::Sweep {
            rom,
            dir,
            levels,
            tier,
            warm_cache_only,
            max_failures,
            golden_capture,
            golden_diff,
        } => {
            let repo_root = current_dir()?;
            let rom_path = resolve_rom_path(&repo_root, rom)?;
            let upstream_dir = resolve_upstream_dir(&repo_root, dir);
            let manifest = prepare_generated_assets(&repo_root, &rom_path, &upstream_dir)?;
            let level_ids = sweep::parse_level_range(&levels)?;
            if golden_capture {
                sweep::golden_capture(&repo_root, &manifest, &level_ids)?;
            } else if golden_diff {
                sweep::golden_diff(&repo_root, &manifest, &level_ids)?;
            } else {
                let options = sweep::SweepOptions {
                    warm_cache_only,
                    max_failures,
                };
                sweep::run_sweep_with_options(
                    &repo_root, &rom_path, &manifest, &level_ids, tier, &options,
                )?;
            }
        }
        Commands::DiagnoseLevel { rom, dir, level } => {
            let repo_root = current_dir()?;
            let rom_path = resolve_rom_path(&repo_root, rom)?;
            let upstream_dir = resolve_upstream_dir(&repo_root, dir);
            let manifest = prepare_generated_assets(&repo_root, &rom_path, &upstream_dir)?;
            let level_id =
                u8::from_str_radix(&level, 16).context("level must be a hex byte (e.g. 05)")?;
            sweep::diagnose_level(&repo_root, &rom_path, &manifest, level_id)?;
        }
        Commands::ParityLoop {
            rom,
            dir,
            max_cycles,
        } => {
            let repo_root = current_dir()?;
            let rom_path = resolve_rom_path(&repo_root, rom)?;
            let upstream_dir = resolve_upstream_dir(&repo_root, dir);
            let manifest = prepare_generated_assets(&repo_root, &rom_path, &upstream_dir)?;
            sweep::run_parity_loop(&repo_root, &rom_path, &manifest, max_cycles)?;
        }
        Commands::CaptureFrames {
            rom,
            dir,
            level,
            frames,
            output,
            layer,
        } => {
            let repo_root = current_dir()?;
            let rom_path = resolve_rom_path(&repo_root, rom)?;
            let upstream_dir = resolve_upstream_dir(&repo_root, dir);
            let manifest = prepare_generated_assets(&repo_root, &rom_path, &upstream_dir)?;
            let level_id =
                u8::from_str_radix(&level, 16).context("level must be a hex byte (e.g. 05)")?;
            let frame_indices: Vec<usize> = frames
                .split(',')
                .map(|s| s.trim().parse::<usize>().context("invalid frame index"))
                .collect::<Result<_>>()?;
            let filter = layer
                .as_deref()
                .map(native_player::LayerFilter::solo)
                .unwrap_or_default();
            sweep::capture_frames(&manifest, level_id, &frame_indices, &output, &filter)?;
        }
        Commands::DiagnoseFrame {
            rom,
            dir,
            level,
            frame,
            save_layers,
            output,
        } => {
            let repo_root = current_dir()?;
            let rom_path = resolve_rom_path(&repo_root, rom)?;
            let upstream_dir = resolve_upstream_dir(&repo_root, dir);
            let manifest = prepare_generated_assets(&repo_root, &rom_path, &upstream_dir)?;
            let level_id =
                u8::from_str_radix(&level, 16).context("level must be a hex byte (e.g. 05)")?;
            sweep::diagnose_frame(&manifest, level_id, frame, save_layers, &output)?;
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

fn resolve_reference_root(repo_root: &Path) -> PathBuf {
    repo_root.join(".cache/references")
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

#[cfg(test)]
mod tests {
    use super::{
        GameplayCapture, GameplayScenario, GameplayState, ReferenceName, analyse_audio_samples,
        audio_envelope_similarity, build_controller_script, capture_native_gameplay_run,
        capture_native_gameplay_scenario, compare_native_gameplay_with,
        compare_native_states_against_oracle, ensure_gameplay_level_layouts_with,
        find_gameplay_live_drifts, gameplay_checkpoint_window, native_trace_to_oracle_state,
        parse_button_mask, parse_input_spec, prepare_generated_assets_with,
        selected_reference_repos,
    };
    use crate::native_player::validate_native_session_assets;
    use crate::oracle::{EnemyFacing, OracleEnemyState, OracleState};
    use kid_core::{GameMode, LevelId, NativeObjectKind, NativeObjectSnapshot, NativeTraceFrame};
    use kid_data::{
        AssetManifest, export_split_bundle, load_manifest, manifest_path as generated_manifest_path,
    };
    use kid_rom::{RomHeader, RomIdentity, VerifiedRom};
    use std::cell::{Cell, RefCell};
    use std::fs;
    use std::path::Path;
    use tempfile::TempDir;

    const SAXMAN_FIXTURE: &[u8] = &[0x00, 0x02, 0xF4, 0x60, 0x41, 0x42, 0x43, 0x44, 0x00, 0x00];
    const ENIGMA_FIXTURE: &[u8] = &[0x05, 0x00, 0x00, 0x01, 0x00, 0x0A, 0x09, 0x18, 0x10, 0xFE];

    fn write_generated_asset_source_fixture(source_root: &Path) {
        fs::create_dir_all(source_root.join("code/enemy")).expect("enemy dir");
        fs::create_dir_all(source_root.join("doc")).expect("doc dir");
        fs::create_dir_all(source_root.join("ingame/anim")).expect("anim dir");
        fs::create_dir_all(source_root.join("ingame/artcomp")).expect("artcomp dir");
        fs::create_dir_all(source_root.join("ingame/artunc")).expect("artunc dir");
        fs::create_dir_all(source_root.join("ingame/artunc_kid")).expect("kid art dir");
        fs::create_dir_all(source_root.join("ingame/palette_enemy")).expect("enemy palette dir");
        fs::create_dir_all(source_root.join("ingame/palette_kid")).expect("kid palette dir");
        fs::create_dir_all(source_root.join("level/platform")).expect("platform dir");
        fs::create_dir_all(source_root.join("level/bgscroll")).expect("bgscroll dir");
        fs::create_dir_all(source_root.join("level/header")).expect("header dir");
        fs::create_dir_all(source_root.join("level/enemy")).expect("enemy assets dir");
        fs::create_dir_all(source_root.join("level/foreground")).expect("foreground dir");
        fs::create_dir_all(source_root.join("level/block")).expect("block dir");
        fs::create_dir_all(source_root.join("level/background")).expect("background dir");
        fs::create_dir_all(source_root.join("scenes")).expect("scene dir");
        fs::create_dir_all(source_root.join("sound")).expect("sound dir");
        fs::create_dir_all(source_root.join("theme")).expect("theme dir");

        fs::write(source_root.join("code/enemy/Armadillo.asm"), b"enemy code").expect("enemy");
        fs::write(source_root.join("theme/sample.bin"), b"theme").expect("theme");
        fs::write(source_root.join("scenes/title.bin"), b"scene").expect("scene");
        fs::write(source_root.join("ingame/anim/idle.bin"), b"anim").expect("anim");
        fs::write(
            source_root.join("ingame/artcomp/Blocks.bin"),
            SAXMAN_FIXTURE,
        )
        .expect("blocks");
        fs::write(
            source_root.join("ingame/artcomp/HUD_numbers.bin"),
            SAXMAN_FIXTURE,
        )
        .expect("hud");
        fs::write(
            source_root.join("ingame/artcomp/Enemy01_Diamond.bin"),
            SAXMAN_FIXTURE,
        )
        .expect("diamond art");
        fs::write(
            source_root.join("ingame/artcomp/Enemy1C_Mini_hopping_skull.bin"),
            SAXMAN_FIXTURE,
        )
        .expect("skull art");
        fs::write(
            source_root.join("ingame/artcomp/Horizontal_platform.bin"),
            SAXMAN_FIXTURE,
        )
        .expect("horizontal platform art");
        fs::write(
            source_root.join("ingame/artcomp/Vertical_platform.bin"),
            SAXMAN_FIXTURE,
        )
        .expect("vertical platform art");
        fs::write(source_root.join("ingame/artunc/diamond_1.bin"), &[0; 32]).expect("diamond");
        fs::write(source_root.join("ingame/artunc_kid/kid.bin"), &[0; 32 * 12]).expect("kid idle");
        fs::write(
            source_root.join("ingame/artunc_kid/kid_walk_1.bin"),
            &[0; 32 * 12],
        )
        .expect("kid walk");
        fs::write(
            source_root.join("ingame/artunc_kid/kid_jumping_1.bin"),
            &[0; 32 * 12],
        )
        .expect("kid jump rise");
        fs::write(
            source_root.join("ingame/artunc_kid/kid_jumping_2.bin"),
            &[0; 32 * 12],
        )
        .expect("kid jump fall");
        fs::write(
            source_root.join("ingame/palette_enemy/Crystal_1.bin"),
            &[0; 30],
        )
        .expect("diamond palette");
        fs::write(
            source_root.join("ingame/palette_enemy/Mini_Hopping_Skull_1.bin"),
            &[0; 30],
        )
        .expect("skull palette");
        fs::write(source_root.join("ingame/palette_kid/Kid.bin"), &[0; 30]).expect("kid palette");
        fs::write(source_root.join("sound/music.bin"), b"sound").expect("sound");
        fs::write(
            source_root.join("level/platform/00.asm"),
            b"\tptfm\t0,0,0,0,0,0,0,0,0,0,PlatformScript_Nothing-PlatformScript_BaseAddress\n\tdc.w\t$FFFF\n",
        )
        .expect("platform");
        fs::write(
            source_root.join("level/bgscroll/00.bin"),
            [0x10, 0x10, 0x40, 0x60, 0xFF],
        )
        .expect("bgscroll");
        fs::write(
            source_root.join("level/header/00.bin"),
            [
                0x06, 0x04, 0x08, 0x18, 0x07, 0x50, 0x00, 0xE0, 0x07, 0x50, 0x03, 0x00,
            ],
        )
        .expect("header");
        fs::write(
            source_root.join("level/enemy/00.bin"),
            [
                0x00, 0x00, 0x00, 0x00, 0xFF, 0xFF, 0x00, 0x01, 0x00, 0x1C, 0x7D, 0x00, 0x80, 0x01,
                0x80, 0x16, 0x00, 0x01, 0x01, 0x20, 0x00, 0xE0,
            ],
        )
        .expect("enemy layout");
        fs::write(source_root.join("level/foreground/00.bin"), SAXMAN_FIXTURE).expect("foreground");
        fs::write(source_root.join("level/block/00.bin"), SAXMAN_FIXTURE).expect("block");
        fs::write(source_root.join("level/background/00.bin"), ENIGMA_FIXTURE).expect("background");
        fs::write(
            source_root.join("doc/level_list.txt"),
            "0x00 0x4A Blue Lake Woods 1\n",
        )
        .expect("level names");
        fs::write(
            source_root.join("level/level_files.txt"),
            "00 platform/00.asm bgscroll/00.bin header/00.bin enemy/00.bin foreground/00.bin block/00.bin background/00.bin\n",
        )
        .expect("level files");
        fs::write(source_root.join("level/maporder.asm"), "\tdc.b\t$00\t; 0\n").expect("map order");
        fs::write(source_root.join("constants.asm"), "Theme_Sky = 1 ; sky\n").expect("constants");
        fs::write(
            source_root.join("variables.asm"),
            "Camera_X_pos = $FFFFF81C ; long (16.16 fixed point)\n",
        )
        .expect("variables");
    }

    fn exported_test_manifest() -> (TempDir, TempDir, AssetManifest) {
        let source_root = TempDir::new().expect("source root");
        let bundle_root = TempDir::new().expect("bundle root");
        write_generated_asset_source_fixture(source_root.path());
        let manifest =
            export_split_bundle(source_root.path(), bundle_root.path(), "abc123", "deadbeef")
                .expect("bundle export");
        (source_root, bundle_root, manifest)
    }

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

    fn oracle_enemy(
        object_type: u8,
        x_pos: u32,
        y_pos: u32,
        x_velocity: i32,
        y_velocity: i32,
        facing: EnemyFacing,
    ) -> OracleEnemyState {
        OracleEnemyState {
            object_type,
            x_pos,
            y_pos,
            x_velocity,
            y_velocity,
            facing,
        }
    }

    fn oracle_state(
        label: &str,
        frame_index: usize,
        active_enemies: Vec<OracleEnemyState>,
    ) -> OracleState {
        OracleState {
            label: label.to_owned(),
            frame_index,
            game_mode: 0x000C,
            current_player: 0,
            number_hitpoints: 2,
            current_helmet_id: 0,
            current_level_id: 0,
            player_1_level_id: 0,
            camera_x_pos: 0x0010_0000,
            camera_y_pos: 0x0008_0000,
            kid_x_pos: 0x0020_0000,
            kid_y_pos: 0x0010_0000,
            number_objects: 3,
            number_gfx_objects: 3,
            game_paused: false,
            active_enemies,
            score: 0,
            number_lives: 3,
            number_diamonds: 0,
            diamond_power_active: false,
            murder_wall_x: None,
        }
    }

    #[test]
    fn native_trace_to_oracle_state_includes_sorted_enemy_snapshots() {
        let frame = NativeTraceFrame {
            frame_index: 810,
            game_mode: GameMode::InGame,
            game_mode_value: 0x000C,
            start_press_count: 6,
            current_player: 0,
            current_level: LevelId(0),
            player_1_level_id: LevelId(0),
            current_level_name: Some(String::from("Blue Lake Woods 1")),
            player_start_x: Some(1872),
            player_start_y: Some(224),
            flag_x: Some(1872),
            flag_y: Some(768),
            level_enemy_object_count: Some(22),
            active_enemy_object_count: 2,
            active_platform_object_count: 0,
            number_lives: 3,
            number_hitpoints: 2,
            current_helmet_id: 0,
            paused: false,
            kid_x_pos: Some(0x0020_0000),
            kid_y_pos: Some(0x0010_0000),
            kid_x_velocity: Some(0),
            kid_y_velocity: Some(0),
            kid_facing: None,
            kid_on_ground: Some(true),
            active_objects: vec![
                NativeObjectSnapshot {
                    kind: NativeObjectKind::Enemy { object_type: 0x1C },
                    x_pos: 0x0030_0000,
                    y_pos: 0x0010_0000,
                    layout_index: Some(7),
                    x_velocity: Some(-0x0001_8000),
                    y_velocity: Some(-0x0002_0000),
                    facing: Some(kid_core::physics::Direction::Left),
                    on_ground: Some(false),
                    hit_points: Some(1),
                    damage_flash: false,
                    palette_variant: 0,
                stationary: false,
                },
                NativeObjectSnapshot {
                    kind: NativeObjectKind::Enemy { object_type: 0x01 },
                    x_pos: 0x0010_0000,
                    y_pos: 0x0010_0000,
                    layout_index: Some(2),
                    x_velocity: Some(0),
                    y_velocity: Some(0),
                    facing: Some(kid_core::physics::Direction::Right),
                    on_ground: Some(true),
                    hit_points: Some(1),
                    damage_flash: false,
                    palette_variant: 0,
                stationary: false,
                },
            ],
            camera_x_pos: 0x0010_0000,
            camera_y_pos: 0x0008_0000,
            number_objects: 3,
            number_gfx_objects: 3,
            level_complete: false,
            game_over: false,
            game_complete: false,
            number_continues: 3,
            invulnerable_frames: 0,
            diamond_power_active: false,
            diamond_count: 0,
            score: 0,
            level_timer_seconds: 180,
            level_damage_taken: false,
            murder_wall_x: None,
            active_projectiles: Vec::new(),
            active_geysers: Vec::new(),
            block_bounces: Vec::new(),
            block_destroy_effects: Vec::new(),
            floating_items: Vec::new(),
            enemy_death_effects: Vec::new(),
            stomped_this_frame: false,
            blocks_hit_this_frame: 0,
            sound_test_track: None,
            sound_test_play_pending: false,
            options_cursor: None,
            two_player: false,
        };

        let state = native_trace_to_oracle_state("enemy_window_open", 810, &frame);

        assert_eq!(state.active_enemies.len(), 2);
        assert_eq!(state.active_enemies[0].object_type, 0x01);
        assert_eq!(state.active_enemies[1].object_type, 0x1C);
        assert_eq!(state.active_enemies[1].x_velocity, -0x0001_8000);
        assert_eq!(state.active_enemies[1].facing, EnemyFacing::Left);
    }

    #[test]
    fn compare_native_states_against_oracle_rejects_enemy_position_divergence() {
        let run_dir = TempDir::new().expect("run dir");
        let native = vec![oracle_state(
            "enemy_window_open",
            810,
            vec![oracle_enemy(
                0x1C,
                0x0030_0000,
                0x0010_0000,
                -0x0001_8000,
                -0x0002_0000,
                EnemyFacing::Left,
            )],
        )];
        let oracle = vec![oracle_state(
            "enemy_window_open",
            810,
            vec![oracle_enemy(
                0x1C,
                0x0050_0000,
                0x0010_0000,
                -0x0001_8000,
                -0x0002_0000,
                EnemyFacing::Left,
            )],
        )];

        let error = compare_native_states_against_oracle(run_dir.path(), &native, &oracle)
            .expect_err("enemy position drift should fail");

        assert!(format!("{error:#}").contains("enemy 1C position"));
    }

    #[test]
    fn compare_native_states_against_oracle_accepts_matching_enemy_snapshots() {
        let run_dir = TempDir::new().expect("run dir");
        let native = vec![oracle_state(
            "enemy_window_open",
            810,
            vec![
                oracle_enemy(0x01, 0x0010_0000, 0x0010_0000, 0, 0, EnemyFacing::Left),
                oracle_enemy(
                    0x1C,
                    0x0030_0000,
                    0x0010_0000,
                    -0x0001_8000,
                    -0x0002_0000,
                    EnemyFacing::Left,
                ),
            ],
        )];

        compare_native_states_against_oracle(run_dir.path(), &native, &native)
            .expect("matching enemy snapshots should pass");
    }

    #[test]
    fn compare_native_states_against_oracle_skips_enemy_compare_when_oracle_has_none() {
        let run_dir = TempDir::new().expect("run dir");
        let native = vec![oracle_state(
            "level_settle",
            719,
            vec![oracle_enemy(
                0x01,
                0x0010_0000,
                0x0010_0000,
                0,
                0,
                EnemyFacing::Left,
            )],
        )];
        let oracle = vec![oracle_state("level_settle", 719, Vec::new())];

        compare_native_states_against_oracle(run_dir.path(), &native, &oracle)
            .expect("missing oracle enemy snapshots should skip the per-enemy compare");
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

    #[test]
    fn selects_all_reference_repos_by_default() {
        let references = selected_reference_repos(None);
        assert_eq!(references.len(), 3);
        assert_eq!(references[0].dir_name, "m68k-instructions-documentation");
        assert_eq!(references[1].dir_name, "gems");
        assert_eq!(references[2].dir_name, "gemsplay");
    }

    #[test]
    fn selects_single_reference_repo() {
        let references = selected_reference_repos(Some(ReferenceName::Gemsplay));
        assert_eq!(references.len(), 1);
        assert_eq!(references[0].dir_name, "gemsplay");
        assert_eq!(references[0].name, "gemsplay");
    }

    #[test]
    fn prepare_generated_assets_rebuilds_when_decoded_version_missing() {
        let repo_root = TempDir::new().expect("repo root");
        let upstream_dir = TempDir::new().expect("upstream");
        let rom_dir = TempDir::new().expect("rom dir");
        let rom_path = rom_dir.path().join("kid.bin");
        fs::write(&rom_path, b"not used").expect("rom placeholder");
        write_generated_asset_source_fixture(upstream_dir.path());

        let verified = VerifiedRom {
            path: rom_path.clone(),
            size: 0x200,
            sha1: String::from("abc123"),
            identity: RomIdentity::HeaderMatchesKidChameleon,
            header: RomHeader {
                console_name: String::from("SEGA MEGA DRIVE"),
                domestic_title: String::from("KID CHAMELEON"),
                overseas_title: String::from("KID CHAMELEON"),
                serial: String::from("GM MK-1010"),
                regions: String::from("JUE"),
            },
        };

        let _manifest = export_split_bundle(
            upstream_dir.path(),
            repo_root.path().join(".cache/generated/abc123").as_path(),
            "abc123",
            "deadbeef",
        )
        .expect("initial export");
        let stale_manifest_path =
            generated_manifest_path(&repo_root.path().join(".cache/generated/abc123"));
        let mut stale_manifest: AssetManifest =
            load_manifest(&stale_manifest_path).expect("stale manifest should load");
        stale_manifest.decoded_level1 = None;
        fs::write(
            &stale_manifest_path,
            serde_json::to_vec_pretty(&stale_manifest).expect("json"),
        )
        .expect("rewrite stale manifest");

        let export_calls = Cell::new(0usize);
        let refreshed = prepare_generated_assets_with(
            repo_root.path(),
            &rom_path,
            upstream_dir.path(),
            |_| Ok(verified.clone()),
            |_| Ok(()),
            |_, _| Ok(()),
            |_| Ok(()),
            |source_root, bundle_dir, sha1, upstream_commit| {
                export_calls.set(export_calls.get() + 1);
                export_split_bundle(source_root, bundle_dir, sha1, upstream_commit)
            },
        )
        .expect("prepare should rebuild");

        assert_eq!(export_calls.get(), 1);
        assert!(refreshed.decoded_level1.is_some());
        let refreshed_manifest =
            load_manifest(&stale_manifest_path).expect("refreshed manifest should load");
        assert!(refreshed_manifest.decoded_level1.is_some());
        let decoded = refreshed_manifest.decoded_level1.expect("decoded metadata");
        assert!(
            repo_root
                .path()
                .join(".cache/generated/abc123")
                .join(decoded.foreground.relative_path)
                .is_file()
        );
    }

    #[test]
    fn prepare_generated_assets_ignores_enemy_layout_trace_sidecar() {
        let repo_root = TempDir::new().expect("repo root");
        let upstream_dir = TempDir::new().expect("upstream");
        let rom_dir = TempDir::new().expect("rom dir");
        let rom_path = rom_dir.path().join("kid.bin");
        fs::write(&rom_path, b"not used").expect("rom placeholder");
        write_generated_asset_source_fixture(upstream_dir.path());

        let verified = VerifiedRom {
            path: rom_path.clone(),
            size: 0x200,
            sha1: String::from("abc123"),
            identity: RomIdentity::HeaderMatchesKidChameleon,
            header: RomHeader {
                console_name: String::from("SEGA MEGA DRIVE"),
                domestic_title: String::from("KID CHAMELEON"),
                overseas_title: String::from("KID CHAMELEON"),
                serial: String::from("GM MK-1010"),
                regions: String::from("JUE"),
            },
        };

        let existing_manifest = export_split_bundle(
            upstream_dir.path(),
            repo_root.path().join(".cache/generated/abc123").as_path(),
            "abc123",
            "deadbeef",
        )
        .expect("initial export");
        let trace_path = repo_root
            .path()
            .join(".cache/native-oracle-runs/boot_title_start_level1/enemy-layout-trace.json");
        fs::create_dir_all(trace_path.parent().expect("trace dir")).expect("trace dir");
        fs::write(&trace_path, b"{ not json").expect("stale trace");

        let export_calls = Cell::new(0usize);
        let prepared = prepare_generated_assets_with(
            repo_root.path(),
            &rom_path,
            upstream_dir.path(),
            |_| Ok(verified.clone()),
            |_| Ok(()),
            |_, _| Ok(()),
            |_| Ok(()),
            |source_root, bundle_dir, sha1, upstream_commit| {
                export_calls.set(export_calls.get() + 1);
                export_split_bundle(source_root, bundle_dir, sha1, upstream_commit)
            },
        )
        .expect("prepare should ignore trace sidecar");

        assert_eq!(export_calls.get(), 0);
        assert_eq!(prepared, existing_manifest);
    }

    #[test]
    fn ensure_gameplay_level_layouts_only_dumps_missing_levels() {
        let bundle_root = TempDir::new().expect("bundle root");
        let layout_dir = bundle_root.path().join("level-layout");
        fs::create_dir_all(&layout_dir).expect("layout dir");
        fs::write(layout_dir.join("00.bin"), [0x60, 0x00]).expect("existing layout");
        fs::write(
            layout_dir.join("00.json"),
            br#"{"width_blocks":200,"height_blocks":28,"data_bytes":2}"#,
        )
        .expect("existing layout metadata");

        let manifest = AssetManifest {
            generated_bundle_version: 7,
            rom_sha1: String::from("test"),
            upstream_commit: String::from("deadbeef"),
            source_root: bundle_root.path().to_path_buf(),
            output_root: bundle_root.path().to_path_buf(),
            map_order: vec![],
            levels: vec![],
            constants: vec![],
            ram_variables: vec![],
            enemy_sources: vec![],
            theme_assets: vec![],
            scene_assets: vec![],
            animation_assets: vec![],
            sound_assets: vec![],
            ingame_assets: vec![],
            decoded_level1: None,
        };

        let dumped_levels = RefCell::new(Vec::new());
        ensure_gameplay_level_layouts_with(
            b"rom",
            &manifest,
            GameplayScenario::ProgressTwoTransitions,
            |_, level_id, out_dir| {
                dumped_levels.borrow_mut().push(level_id);
                fs::write(out_dir.join(format!("{level_id:02X}.bin")), [0x60, 0x00])
                    .expect("dumped layout");
                fs::write(
                    out_dir.join(format!("{level_id:02X}.json")),
                    br#"{"width_blocks":200,"height_blocks":28,"data_bytes":2}"#,
                )
                .expect("dumped layout metadata");
                Ok(())
            },
        )
        .expect("missing layouts should be dumped");

        assert_eq!(*dumped_levels.borrow(), vec![0x01, 0x02]);
        assert!(layout_dir.join("02.bin").is_file());
        assert!(layout_dir.join("02.json").is_file());
    }

    #[test]
    fn play_native_fails_fast_when_required_level1_visuals_are_invalid() {
        let source_root = TempDir::new().expect("source root");
        let bundle_root = TempDir::new().expect("bundle root");
        write_generated_asset_source_fixture(source_root.path());

        let manifest =
            export_split_bundle(source_root.path(), bundle_root.path(), "abc123", "deadbeef")
                .expect("bundle export");
        let error = validate_native_session_assets(&manifest, 0x00)
            .expect_err("missing required visuals should fail startup");

        let message = format!("{error:#}");
        assert!(message.contains("foreground layout"));
        assert!(message.contains("6720"));
        assert!(message.contains("120x56 cells"));
    }

    #[test]
    fn gameplay_regression_compare_native_gameplay_retains_semantic_passes_while_writing_frame_evidence()
     {
        let repo_root = TempDir::new().expect("repo root");
        let capture = GameplayCapture {
            scenario: String::from("pause_resume_level1"),
            frame_count: 4,
            missed_vblank_frames: 0,
            states: vec![
                gameplay_state("level_settle", 0),
                gameplay_state("pre_pause", 1),
                gameplay_state("paused_tail", 2),
                gameplay_state("resumed_tail", 3),
            ],
            checkpoints: vec![
                gameplay_state("level_settle", 0),
                gameplay_state("pre_pause", 1),
                gameplay_state("paused_tail", 2),
                gameplay_state("resumed_tail", 3),
            ],
        };

        compare_native_gameplay_with(
            repo_root.path(),
            GameplayScenario::PauseResumeLevel1,
            || Ok(capture.clone()),
            || Ok(capture.clone()),
            || {
                Ok(vec![(
                    String::from("paused_tail"),
                    2usize,
                    2usize,
                    1usize,
                    vec![1u8, 2, 3, 4, 5, 6],
                )])
            },
        )
        .expect("semantic comparison with frame evidence should pass");

        let run_dir = repo_root
            .path()
            .join(".cache/native-gameplay-oracle-runs")
            .join(GameplayScenario::PauseResumeLevel1.slug());
        assert!(run_dir.join("review/frame-evidence.json").is_file());
        assert!(run_dir.join("review/0002_paused_tail.png").is_file());
        assert!(run_dir.join("native/checkpoints.json").is_file());
        assert!(run_dir.join("oracle/checkpoints.json").is_file());
    }

    #[test]
    fn gameplay_regression_native_gameplay_capture_uses_default_runtime_path() {
        let (_source_root, _bundle_root, manifest) = exported_test_manifest();
        let capture =
            capture_native_gameplay_scenario(&manifest, GameplayScenario::DeathRespawnLevel1)
                .expect("capture");
        assert_eq!(capture.checkpoints.len(), 4);
        assert!(
            capture
                .checkpoints
                .iter()
                .any(|state| state.label == "respawn_tail"),
            "default native gameplay capture should reach the scripted respawn checkpoint"
        );
    }

    #[test]
    fn gameplay_regression_finds_first_live_drift_from_requested_frame() {
        let native = GameplayCapture {
            scenario: String::from("pause_resume_level1"),
            frame_count: 5,
            missed_vblank_frames: 0,
            states: vec![
                gameplay_state("frame_00719", 719),
                gameplay_state("frame_00720", 720),
                gameplay_state("frame_00721", 721),
                gameplay_state("frame_00722", 722),
                gameplay_state("frame_00723", 723),
            ],
            checkpoints: Vec::new(),
        };
        let mut oracle = native.clone();
        oracle.states[1].camera_x_pos = 0x0099_0000;
        oracle.states[3].kid_y_pos = 0x0042_0000;

        let reports =
            find_gameplay_live_drifts(&native, &oracle, GameplayScenario::PauseResumeLevel1, 721);

        assert_eq!(reports.len(), 1);
        assert_eq!(reports[0].frame_index, 722);
        assert_eq!(reports[0].differing_fields, vec!["kid_y_pos"]);
        assert_eq!(
            reports[0].window,
            String::from("level_settle@719 -> pre_pause@839")
        );
    }

    #[test]
    fn gameplay_regression_capture_native_gameplay_run_writes_summaries_without_compare() {
        let repo_root = TempDir::new().expect("repo root");
        let capture = GameplayCapture {
            scenario: String::from("pause_resume_level1"),
            frame_count: 2,
            missed_vblank_frames: 0,
            states: vec![
                gameplay_state("frame_00719", 719),
                gameplay_state("frame_00720", 720),
            ],
            checkpoints: vec![gameplay_state("level_settle", 719)],
        };

        let (run_dir, native, oracle) = capture_native_gameplay_run(
            repo_root.path(),
            GameplayScenario::PauseResumeLevel1,
            || Ok(capture.clone()),
            || Ok(capture.clone()),
        )
        .expect("capture run should succeed");

        assert_eq!(native.states.len(), 2);
        assert_eq!(oracle.states.len(), 2);
        assert!(run_dir.join("native/summary.json").is_file());
        assert!(run_dir.join("oracle/summary.json").is_file());
        assert_eq!(
            gameplay_checkpoint_window(GameplayScenario::PauseResumeLevel1, 920),
            String::from("paused_tail@919 -> resumed_tail@1119")
        );
    }

    fn gameplay_state(label: &str, frame_index: usize) -> GameplayState {
        GameplayState {
            label: label.to_owned(),
            frame_index,
            game_mode: 0x000C,
            current_player: 0,
            number_lives: 3,
            number_hitpoints: 2,
            current_level_id: 0,
            player_1_level_id: 0,
            current_helmet_id: 0,
            camera_x_pos: 0x0010_0000 + (frame_index as u32 * 0x0004_0000),
            camera_y_pos: 0x0008_0000,
            kid_x_pos: 0x0020_0000 + (frame_index as u32 * 0x0004_0000),
            kid_y_pos: 0x0010_0000,
            number_objects: 3,
            number_gfx_objects: 3,
        }
    }
}
