mod autonomy;
mod bootstrap;
mod capture;
mod cheats;
mod compare;
mod native_player;
mod oracle;
mod player;
mod scenarios;
mod verify;

use verify::*;

use bootstrap::*;
use capture::*;
use compare::*;

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
    let effective_resource_level_id = resource_level_id(manifest, level_id);
    if effective_resource_level_id != level_id {
        println!("resource level: 0x{effective_resource_level_id:02X}");
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
    if let Some(bgscroll) = &bundle.parsed_bgscroll {
        let preview = bgscroll
            .layer_speeds
            .iter()
            .take(8)
            .map(|speed| format!("0x{speed:02X}"))
            .collect::<Vec<_>>()
            .join(", ");
        println!(
            "bgscroll: {} layers{}{}",
            bgscroll.layer_speeds.len(),
            if preview.is_empty() {
                String::new()
            } else {
                format!(" [{preview}]")
            },
            if bgscroll.has_terminator {
                String::new()
            } else {
                String::from(" (unterminated split blob)")
            }
        );
    }
    if let Some(platform_layout) = &bundle.parsed_platform_layout {
        let preview = platform_layout
            .placements
            .iter()
            .take(3)
            .map(|placement| {
                format!(
                    "({}, {}) t{} s{} {}",
                    placement.x,
                    placement.y,
                    placement.kind,
                    placement.subtype,
                    placement.script.raw
                )
            })
            .collect::<Vec<_>>()
            .join(", ");
        println!(
            "platforms: {} entries{}",
            platform_layout.placements.len(),
            if preview.is_empty() {
                String::new()
            } else {
                format!(" [{preview}]")
            }
        );
    }
    if let Some(enemy_layout) = &bundle.parsed_enemy_layout {
        println!(
            "enemy layout: {} objects across {} segment{} ({})",
            enemy_layout.object_count,
            enemy_layout.segments.len().max(1),
            if enemy_layout.segments.len() == 1 {
                ""
            } else {
                "s"
            },
            if enemy_layout.axis_uses_y_position {
                "camera-y keyed"
            } else {
                "camera-x keyed"
            }
        );
        let primary_slots = enemy_layout
            .segments
            .first()
            .map(|segment| segment.enemy_type_slots.as_slice())
            .unwrap_or(enemy_layout.enemy_type_slots.as_slice());
        for (slot_index, slot) in primary_slots.iter().enumerate() {
            println!(
                "enemy slot {}: {}",
                slot_index + 1,
                describe_enemy_slot(manifest, *slot)
            );
        }
        if let Some(segment) = enemy_layout.segments.first() {
            println!(
                "enemy segment 1: 0x{:04X}..0x{:04X}, batch 0x{:02X}, {} object{}",
                segment.lower_bound,
                segment.upper_bound,
                segment.object_batch_flags,
                segment.object_count,
                if segment.object_count == 1 { "" } else { "s" }
            );
        } else {
            println!(
                "enemy segment 1: 0x0000..0x{:04X}, batch 0x{:02X}, {} object{}",
                enemy_layout.unknown_word,
                enemy_layout.header_byte,
                enemy_layout.object_count,
                if enemy_layout.object_count == 1 {
                    ""
                } else {
                    "s"
                }
            );
        }
        println!(
            "enemy objects: {}",
            summarise_enemy_object_types(&enemy_layout.objects)
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

fn describe_enemy_slot(manifest: &AssetManifest, raw_slot: u16) -> String {
    if raw_slot == 0xFFFF {
        return String::from("blank");
    }

    let palette_flags = raw_slot >> 14;
    let slot_id = raw_slot & 0x3FFF;
    let enemy_name = u8::try_from(slot_id)
        .ok()
        .and_then(|enemy_id| find_enemy_definition(manifest, &format!("{enemy_id:02X}")))
        .map(|definition| definition.symbolic_name);

    match enemy_name {
        Some(enemy_name) => format!(
            "0x{raw_slot:04X} -> {enemy_name} (kind 0x{slot_id:02X}, palette {palette_flags})"
        ),
        None => format!("0x{raw_slot:04X} (kind 0x{slot_id:04X}, palette {palette_flags})"),
    }
}

fn summarise_enemy_object_types(objects: &[kid_data::EnemyLayoutObject]) -> String {
    if objects.is_empty() {
        return String::from("<none>");
    }

    let mut counts = BTreeMap::new();
    for object in objects {
        *counts.entry(object.object_type).or_insert(0usize) += 1;
    }

    counts
        .into_iter()
        .map(|(object_type, count)| format!("0x{object_type:02X}x{count}"))
        .collect::<Vec<_>>()
        .join(", ")
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
    println!("number hitpoints: {}", probe.number_hitpoints);
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

fn probe_native_scenario(manifest: &AssetManifest, scenario: Scenario) -> Result<()> {
    let script = scenario.script()?;
    let trace = run_native_script_for_scenario(manifest, &script, scenario);
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
    if let (Some(player_start_x), Some(player_start_y)) =
        (final_frame.player_start_x, final_frame.player_start_y)
    {
        println!("player start: {player_start_x},{player_start_y}");
    }
    if let (Some(flag_x), Some(flag_y)) = (final_frame.flag_x, final_frame.flag_y) {
        println!("flag: {flag_x},{flag_y}");
    }
    if let Some(level_enemy_object_count) = final_frame.level_enemy_object_count {
        println!("level enemy objects: {level_enemy_object_count}");
    }
    println!(
        "active enemy objects: {}",
        final_frame.active_enemy_object_count
    );
    println!(
        "active platform objects: {}",
        final_frame.active_platform_object_count
    );
    if let (Some(kid_x_pos), Some(kid_y_pos)) = (final_frame.kid_x_pos, final_frame.kid_y_pos) {
        println!("kid pos: {kid_x_pos:08X},{kid_y_pos:08X}");
    }
    println!("active roster: {}", describe_active_objects(&final_frame));

    for checkpoint in scenario.checkpoints() {
        if let Some(frame) =
            native_trace_at_checkpoint(&trace, checkpoint.label, checkpoint.frame_index)
        {
            println!(
                "checkpoint {}@{}: {:?} level 0x{:02X} {} camera {:08X},{:08X} objects {}/{} active_enemies {} active_platforms {} roster {}",
                checkpoint.label,
                checkpoint.frame_index,
                frame.game_mode,
                frame.current_level.0,
                frame.current_level_name.as_deref().unwrap_or(""),
                frame.camera_x_pos,
                frame.camera_y_pos,
                frame.number_objects,
                frame.number_gfx_objects,
                frame.active_enemy_object_count,
                frame.active_platform_object_count,
                describe_active_objects(frame)
            );
        }
    }

    Ok(())
}

fn probe_native_gameplay_scenario(
    manifest: &AssetManifest,
    scenario: GameplayScenario,
    frame_start: Option<usize>,
    frame_end: Option<usize>,
) -> Result<()> {
    let script = scenario.script()?;
    let trace = run_native_script_for_gameplay(manifest, &script, scenario);
    let final_frame = trace.last().ok_or_else(|| {
        anyhow!(
            "native trace was empty for gameplay scenario {}",
            scenario.slug()
        )
    })?;

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
    println!(
        "lives/hp: {}/{}",
        final_frame.number_lives, final_frame.number_hitpoints
    );
    println!(
        "active enemy objects: {}",
        final_frame.active_enemy_object_count
    );
    println!(
        "active platform objects: {}",
        final_frame.active_platform_object_count
    );
    if let (Some(kid_x_pos), Some(kid_y_pos)) = (final_frame.kid_x_pos, final_frame.kid_y_pos) {
        println!("kid pos: {kid_x_pos:08X},{kid_y_pos:08X}");
    }
    println!(
        "camera: {:08X},{:08X}",
        final_frame.camera_x_pos, final_frame.camera_y_pos
    );
    println!("active roster: {}", describe_active_objects(final_frame));

    for checkpoint in scenario.checkpoints() {
        if let Some(frame) =
            native_trace_at_checkpoint(&trace, checkpoint.label, checkpoint.frame_index)
        {
            println!(
                "checkpoint {}@{}: {:?} level 0x{:02X} {} camera {:08X},{:08X} kid {:08X},{:08X} lives/hp {}/{} objects {}/{} active_enemies {} active_platforms {} roster {}",
                checkpoint.label,
                checkpoint.frame_index,
                frame.game_mode,
                frame.current_level.0,
                frame.current_level_name.as_deref().unwrap_or(""),
                frame.camera_x_pos,
                frame.camera_y_pos,
                frame.kid_x_pos.unwrap_or_default(),
                frame.kid_y_pos.unwrap_or_default(),
                frame.number_lives,
                frame.number_hitpoints,
                frame.number_objects,
                frame.number_gfx_objects,
                frame.active_enemy_object_count,
                frame.active_platform_object_count,
                describe_active_objects(frame)
            );
        }
    }

    if let Some(start) = frame_start {
        let end = frame_end.unwrap_or(start);
        for frame in trace
            .iter()
            .filter(|frame| (start..=end).contains(&frame.frame_index))
        {
            let input = script
                .get(frame.frame_index)
                .copied()
                .map(kid_core::FrameInput::from_mask)
                .unwrap_or_default();
            println!(
                "frame {}: input {} mode {:?} camera {:08X},{:08X} kid {:08X},{:08X} vel {:08X},{:08X} ground {:?} lives/hp {}/{} active_enemies {} active_platforms {} roster {}",
                frame.frame_index,
                describe_frame_input(input),
                frame.game_mode,
                frame.camera_x_pos,
                frame.camera_y_pos,
                frame.kid_x_pos.unwrap_or_default(),
                frame.kid_y_pos.unwrap_or_default(),
                frame.kid_x_velocity.unwrap_or_default() as u32,
                frame.kid_y_velocity.unwrap_or_default() as u32,
                frame.kid_on_ground,
                frame.number_lives,
                frame.number_hitpoints,
                frame.active_enemy_object_count,
                frame.active_platform_object_count,
                describe_active_objects(frame)
            );
        }
    }

    Ok(())
}

fn describe_frame_input(input: kid_core::FrameInput) -> String {
    let mut pressed = Vec::new();
    if input.up {
        pressed.push("U");
    }
    if input.down {
        pressed.push("D");
    }
    if input.left {
        pressed.push("L");
    }
    if input.right {
        pressed.push("R");
    }
    if input.a {
        pressed.push("A");
    }
    if input.b {
        pressed.push("B");
    }
    if input.c {
        pressed.push("C");
    }
    if input.start {
        pressed.push("Start");
    }
    if pressed.is_empty() {
        String::from("<none>")
    } else {
        pressed.join("+")
    }
}

fn describe_active_objects(frame: &NativeTraceFrame) -> String {
    if frame.active_objects.is_empty() {
        return String::from("<none>");
    }

    frame
        .active_objects
        .iter()
        .map(|object| match object.kind {
            kid_core::NativeObjectKind::Player => {
                format!("player@{:08X},{:08X}", object.x_pos, object.y_pos)
            }
            kid_core::NativeObjectKind::Enemy { object_type } => {
                format!(
                    "enemy_{object_type:02X}@{:08X},{:08X}",
                    object.x_pos, object.y_pos
                )
            }
            kid_core::NativeObjectKind::Platform { kind, subtype } => {
                format!(
                    "platform_{kind:02X}_{subtype:02X}@{:08X},{:08X}",
                    object.x_pos, object.y_pos
                )
            }
        })
        .collect::<Vec<_>>()
        .join(", ")
}



fn trace_oracle_enemy_layout(repo_root: &Path, rom_path: &Path, scenario: Scenario) -> Result<()> {
    let env = ensure_oracles(repo_root)?;
    let run_dir = repo_root
        .join(".cache/native-oracle-runs")
        .join(scenario.slug());
    let oracle_dir = run_dir.join("oracle-trace");
    if oracle_dir.exists() {
        fs::remove_dir_all(&oracle_dir)
            .with_context(|| format!("failed to remove {}", oracle_dir.display()))?;
    }
    fs::create_dir_all(&oracle_dir)
        .with_context(|| format!("failed to create {}", oracle_dir.display()))?;

    let trace = run_jgenesis_enemy_layout_trace(&env, rom_path, scenario, &oracle_dir)?;
    let trace_path = run_dir.join("enemy-layout-trace.json");
    fs::create_dir_all(&run_dir)
        .with_context(|| format!("failed to create {}", run_dir.display()))?;
    fs::write(&trace_path, serde_json::to_vec_pretty(&trace)?)
        .with_context(|| format!("failed to write {}", trace_path.display()))?;
    print_enemy_layout_trace_summary(&trace_path, &trace);
    Ok(())
}

fn print_enemy_layout_trace_summary(path: &Path, trace: &OracleEnemyLayoutTraceManifest) {
    println!("scenario: {}", trace.scenario);
    println!("trace: {}", path.display());
    for checkpoint in &trace.checkpoints {
        let slot_summary = checkpoint
            .enemy_type_slots
            .iter()
            .map(|slot| format!("0x{slot:04X}"))
            .collect::<Vec<_>>()
            .join(", ");
        let visible_summary = checkpoint
            .visible_enemies
            .iter()
            .map(|enemy| format!("0x{:02X}@{:06X}", enemy.object_type, enemy.layout_ptr))
            .collect::<Vec<_>>()
            .join(", ");
        println!(
            "{}@{} camera {:08X},{:08X} header {:06X} object {:06X} axis {} slots [{}] upper 0x{:04X} batch 0x{:02X} count {} visible [{}]",
            checkpoint.label,
            checkpoint.frame_index,
            checkpoint.camera_x_pos,
            checkpoint.camera_y_pos,
            checkpoint.header_ptr,
            checkpoint.object_ptr,
            if checkpoint.axis_uses_y_position {
                "y"
            } else {
                "x"
            },
            slot_summary,
            checkpoint.upper_bound,
            checkpoint.object_batch_flags,
            checkpoint.object_count,
            visible_summary,
        );
    }
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











// ---------------------------------------------------------------------------
// Level layout dump — extracts the collision grid from emulator RAM
// ---------------------------------------------------------------------------

/// RAM addresses for level layout extraction (from variables.asm).
const LEVEL_LAYOUT_ADDR: u32 = 0x00FF_A652;
const LEVEL_LAYOUT_SIZE: usize = 0x41A0;
/// Block List RAM: prize/ghost/teleporter/hidden entries (ram_layout.txt).
const BLOCK_LIST_ADDR: u32 = 0x00FF_3B24;
const BLOCK_LIST_SIZE: usize = 0x0898; // FF3B24..FF43BC

fn dump_level_layout(rom_bytes: &[u8], level_id: u8, out_dir: &Path) -> Result<()> {
    let mut runtime = MdRuntime::new(rom_bytes)?;

    // If level_id != 0, set it before the game starts
    if level_id != 0 {
        // We'll set the level after the game has initialised
    }

    // Run through boot + title presses to reach gameplay
    let entry_script = gameplay_entry_script()?;
    for (frame_index, buttons) in entry_script.iter().enumerate() {
        runtime.set_controller_1(*buttons);
        if !runtime.run_until_vblank(500_000) {
            bail!("emulator stalled at frame {frame_index}");
        }
    }

    // Run additional frames to ensure level is fully loaded
    for _ in 0..60 {
        runtime.set_controller_1(0);
        runtime.run_until_vblank(500_000);
    }

    let snapshot = runtime.snapshot();
    if snapshot.game_mode != 0x000C {
        bail!(
            "expected in-level game mode 0x000C, got 0x{:04X}",
            snapshot.game_mode
        );
    }

    // variables.asm addresses:
    //   Level_width_blocks  = $FFFFF89E (cells per row, each 16x16 px)
    //   Level_height_blocks = $FFFFF8A4 (rows)
    let width_blocks = runtime.read_work_ram_word(0x00FF_F89E);
    let height_blocks = runtime.read_work_ram_word(0x00FF_F8A4);
    let layout_data = runtime.read_work_ram_range(LEVEL_LAYOUT_ADDR, LEVEL_LAYOUT_SIZE);
    let block_list_data = runtime.read_work_ram_range(BLOCK_LIST_ADDR, BLOCK_LIST_SIZE);

    fs::create_dir_all(out_dir)
        .with_context(|| format!("failed to create {}", out_dir.display()))?;

    let filename = format!("{:02X}.bin", level_id);
    let layout_path = out_dir.join(&filename);
    fs::write(&layout_path, &layout_data)
        .with_context(|| format!("failed to write {}", layout_path.display()))?;

    // Dump the Block List (prize/ghost/teleporter/hidden entries)
    let block_list_path = out_dir.join(format!("{:02X}.blocklist.bin", level_id));
    fs::write(&block_list_path, &block_list_data)
        .with_context(|| format!("failed to write {}", block_list_path.display()))?;

    // Write metadata alongside
    let meta = serde_json::json!({
        "level_id": level_id,
        "width_blocks": width_blocks,
        "height_blocks": height_blocks,
        "width_pixels": u32::from(width_blocks) * 16,
        "height_pixels": u32::from(height_blocks) * 16,
        "data_bytes": usize::from(width_blocks) * usize::from(height_blocks) * 2,
        "dump_bytes": layout_data.len(),
    });
    let meta_path = out_dir.join(format!("{:02X}.json", level_id));
    fs::write(&meta_path, serde_json::to_string_pretty(&meta)?)
        .with_context(|| format!("failed to write {}", meta_path.display()))?;

    println!(
        "dumped Level_Layout for level 0x{:02X}: {}x{} cells ({}x{} px) to {}",
        level_id,
        width_blocks,
        height_blocks,
        u32::from(width_blocks) * 16,
        u32::from(height_blocks) * 16,
        layout_path.display()
    );
    Ok(())
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
            active_enemies,
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
            murder_wall_x: None,
            active_projectiles: Vec::new(),
            active_geysers: Vec::new(),
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
