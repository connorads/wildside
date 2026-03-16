mod autonomy;
mod cheats;
mod native_player;
mod oracle;
mod player;
mod scenarios;

use anyhow::{Context, Result, anyhow, bail};
use autonomy::{AutonomyGoal, run_autonomous_loop};
use clap::{Parser, Subcommand, ValueEnum};
use image::{ColorType, ImageFormat};
use kid_core::physics::Direction as NativeDirection;
use kid_core::{GameMode as NativeGameMode, NativeTraceFrame, run_script};
use kid_data::{
    AssetManifest, ensure_generated_bundle_contract, export_split_bundle, find_enemy_definition,
    generated_bundle_dir, load_level_asset_bundle, load_manifest,
    manifest_path as generated_manifest_path, resource_level_id,
};
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
use std::fs::OpenOptions;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::thread;
use std::time::{Duration, Instant};
use ymfm_sys::Ym2612;

const ASL_REPO_URL: &str = "https://github.com/Macroassembler-AS/asl-releases.git";
const UPSTREAM_URL: &str = "https://github.com/sonicretro/kid-chameleon-disasm";
const UPSTREAM_COMMIT: &str = "a134567";
const M68K_REFERENCE_URL: &str = "https://github.com/prb28/m68k-instructions-documentation";
const M68K_REFERENCE_COMMIT: &str = "89744462bde926313aa273dde2109b680ed11c8a";
const GEMS_REFERENCE_URL: &str = "https://github.com/realmonster/GEMS";
const GEMS_REFERENCE_COMMIT: &str = "a09a734532231fb8a1170e55f018d8511329562e";
const GEMSPLAY_REFERENCE_URL: &str = "https://github.com/ValleyBell/GEMSPlay";
const GEMSPLAY_REFERENCE_COMMIT: &str = "8b2e490942e9e8c0cc8d6f9132f15c5b8507a5db";

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
enum ReferenceName {
    M68k,
    Gems,
    Gemsplay,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct ReferenceRepo {
    name: &'static str,
    dir_name: &'static str,
    url: &'static str,
    commit: &'static str,
    purpose: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct BootstrappedReference {
    name: &'static str,
    path: PathBuf,
    commit: &'static str,
    purpose: &'static str,
}

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

#[derive(Debug, Clone, Serialize, Deserialize)]
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
    number_hitpoints: u16,
    current_helmet_id: u16,
    current_level_id: u16,
    player_1_level_id: u16,
    camera_x_pos: u32,
    camera_y_pos: u32,
    kid_x_pos: u32,
    kid_y_pos: u32,
    number_objects: u16,
    number_gfx_objects: u16,
    active_enemies: Vec<OracleEnemyState>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct GameplayState {
    label: String,
    frame_index: usize,
    game_mode: u16,
    current_player: u8,
    number_lives: u16,
    number_hitpoints: u16,
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

#[derive(Debug, Clone, PartialEq, Eq)]
struct GameplayDriftReport {
    frame_index: usize,
    window: String,
    differing_fields: Vec<&'static str>,
    native: GameplayState,
    oracle: GameplayState,
}

type GameplayFrameEvidence = (String, usize, usize, usize, Vec<u8>);

#[derive(Debug, Clone, Serialize)]
struct GameplayFrameEvidenceRecord {
    label: String,
    frame_index: usize,
    png_path: String,
    sha1: String,
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
                number_hitpoints: snapshot.number_hitpoints,
                current_helmet_id: snapshot.current_helmet_id,
                current_level_id: snapshot.current_level_id,
                player_1_level_id: snapshot.player_1_level_id,
                camera_x_pos: snapshot.camera_x_pos,
                camera_y_pos: snapshot.camera_y_pos,
                kid_x_pos: snapshot.kid_x_pos,
                kid_y_pos: snapshot.kid_y_pos,
                number_objects: snapshot.number_objects,
                number_gfx_objects: snapshot.number_gfx_objects,
                active_enemies: Vec::new(),
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
        number_hitpoints: snapshot.number_hitpoints,
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

fn capture_native_gameplay_scenario(
    manifest: &AssetManifest,
    scenario: GameplayScenario,
) -> Result<GameplayCapture> {
    if !matches!(
        scenario,
        GameplayScenario::PauseResumeLevel1
            | GameplayScenario::DeathRespawnLevel1
            | GameplayScenario::ProgressTwoTransitions
            | GameplayScenario::SoakLevel10m
    ) {
        bail!(
            "verify-native-gameplay does not support scenario {}",
            scenario.slug()
        );
    }

    let script = scenario.script()?;
    let checkpoints = scenario.checkpoints();
    let trace = run_native_script_for_gameplay(manifest, &script, scenario);
    let mut checkpoint_index = 0usize;
    let mut states = Vec::with_capacity(trace.len());
    let mut captured_checkpoints = Vec::with_capacity(checkpoints.len());

    for snapshot in trace {
        let state = gameplay_state_from_native_trace(&snapshot);
        while checkpoint_index < checkpoints.len()
            && checkpoints[checkpoint_index].frame_index == snapshot.frame_index
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
        scenario: format!("native_{}", scenario.slug()),
        frame_count: scenario.frame_count(),
        missed_vblank_frames: 0,
        states,
        checkpoints: captured_checkpoints,
    })
}

fn gameplay_state_from_native_trace(snapshot: &NativeTraceFrame) -> GameplayState {
    GameplayState {
        label: format!("frame_{:05}", snapshot.frame_index),
        frame_index: snapshot.frame_index,
        game_mode: snapshot.game_mode_value,
        current_player: snapshot.current_player,
        number_lives: snapshot.number_lives,
        number_hitpoints: snapshot.number_hitpoints,
        current_level_id: u16::from(snapshot.current_level.0),
        player_1_level_id: u16::from(snapshot.player_1_level_id.0),
        current_helmet_id: snapshot.current_helmet_id,
        camera_x_pos: snapshot.camera_x_pos,
        camera_y_pos: snapshot.camera_y_pos,
        kid_x_pos: snapshot.kid_x_pos.unwrap_or(0),
        kid_y_pos: snapshot.kid_y_pos.unwrap_or(0),
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

fn print_bootstrapped_references(reference_root: &Path, references: &[BootstrappedReference]) {
    println!("reference root: {}", reference_root.display());
    for reference in references {
        println!(
            "{}: {} @ {}",
            reference.name,
            reference.path.display(),
            reference.commit
        );
        println!("purpose: {}", reference.purpose);
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

fn bootstrap_references(
    reference_root: &Path,
    only: Option<ReferenceName>,
) -> Result<Vec<BootstrappedReference>> {
    fs::create_dir_all(reference_root)
        .with_context(|| format!("failed to create {}", reference_root.display()))?;

    let mut references = Vec::new();
    for reference in selected_reference_repos(only) {
        let path = reference_root.join(reference.dir_name);
        bootstrap_pinned_repo(&path, reference.url, reference.commit)?;
        references.push(BootstrappedReference {
            name: reference.name,
            path,
            commit: reference.commit,
            purpose: reference.purpose,
        });
    }
    Ok(references)
}

fn selected_reference_repos(only: Option<ReferenceName>) -> &'static [ReferenceRepo] {
    const M68K: ReferenceRepo = ReferenceRepo {
        name: "m68k",
        dir_name: "m68k-instructions-documentation",
        url: M68K_REFERENCE_URL,
        commit: M68K_REFERENCE_COMMIT,
        purpose: "Primary 68k instruction and directive reference",
    };
    const GEMS: ReferenceRepo = ReferenceRepo {
        name: "gems",
        dir_name: "gems",
        url: GEMS_REFERENCE_URL,
        commit: GEMS_REFERENCE_COMMIT,
        purpose: "Primary GEMS tooling and driver research reference",
    };
    const GEMSPLAY: ReferenceRepo = ReferenceRepo {
        name: "gemsplay",
        dir_name: "gemsplay",
        url: GEMSPLAY_REFERENCE_URL,
        commit: GEMSPLAY_REFERENCE_COMMIT,
        purpose: "Practical GEMS playback and inspection reference",
    };
    const ALL: &[ReferenceRepo] = &[M68K, GEMS, GEMSPLAY];
    const ONLY_M68K: &[ReferenceRepo] = &[M68K];
    const ONLY_GEMS: &[ReferenceRepo] = &[GEMS];
    const ONLY_GEMSPLAY: &[ReferenceRepo] = &[GEMSPLAY];

    match only {
        None => ALL,
        Some(ReferenceName::M68k) => ONLY_M68K,
        Some(ReferenceName::Gems) => ONLY_GEMS,
        Some(ReferenceName::Gemsplay) => ONLY_GEMSPLAY,
    }
}

fn bootstrap_pinned_repo(dir: &Path, url: &str, commit: &str) -> Result<()> {
    if !dir.exists() {
        let parent = dir
            .parent()
            .ok_or_else(|| anyhow!("reference directory has no parent: {}", dir.display()))?;
        fs::create_dir_all(parent)
            .with_context(|| format!("failed to create {}", parent.display()))?;
        run_command(Command::new("git").arg("clone").arg(url).arg(dir), None)?;
    }

    run_command(
        Command::new("git").arg("fetch").arg("--all").arg("--tags"),
        Some(dir),
    )?;
    run_command(
        Command::new("git")
            .arg("checkout")
            .arg("--detach")
            .arg(commit),
        Some(dir),
    )?;
    Ok(())
}

fn prepare_generated_assets(
    repo_root: &Path,
    rom_path: &Path,
    upstream_dir: &Path,
) -> Result<AssetManifest> {
    prepare_generated_assets_with(
        repo_root,
        rom_path,
        upstream_dir,
        |path| Ok(verify_rom(path)?),
        bootstrap_upstream,
        sync_rom_into_upstream,
        |dir| run_command(Command::new("python3").arg("split.py"), Some(dir)),
        export_split_bundle,
    )
}

fn ensure_gameplay_level_layouts(
    rom_path: &Path,
    manifest: &AssetManifest,
    scenario: GameplayScenario,
) -> Result<()> {
    let rom_bytes = fs::read(rom_path)
        .with_context(|| format!("failed to read ROM from {}", rom_path.display()))?;
    ensure_gameplay_level_layouts_with(&rom_bytes, manifest, scenario, dump_level_layout)
}

fn ensure_gameplay_level_layouts_with<FDump>(
    rom_bytes: &[u8],
    manifest: &AssetManifest,
    scenario: GameplayScenario,
    dump_level_layout_fn: FDump,
) -> Result<()>
where
    FDump: Fn(&[u8], u8, &Path) -> Result<()>,
{
    let (start_level, end_level) = scenario.required_level_layout_range();
    let layout_dir = manifest.output_root.join("level-layout");
    let missing_levels = (start_level..=end_level)
        .filter(|level_id| {
            let data_path = layout_dir.join(format!("{level_id:02X}.bin"));
            let metadata_path = layout_dir.join(format!("{level_id:02X}.json"));
            !(data_path.is_file() && metadata_path.is_file())
        })
        .collect::<Vec<_>>();
    if missing_levels.is_empty() {
        return Ok(());
    }

    for level_id in missing_levels {
        dump_level_layout_fn(rom_bytes, level_id, &layout_dir)?;
    }
    Ok(())
}

fn prepare_generated_assets_with<FVerify, FBootstrap, FSync, FSplit, FExport>(
    repo_root: &Path,
    rom_path: &Path,
    upstream_dir: &Path,
    verify_rom_fn: FVerify,
    bootstrap_upstream_fn: FBootstrap,
    sync_rom_fn: FSync,
    split_fn: FSplit,
    export_bundle_fn: FExport,
) -> Result<AssetManifest>
where
    FVerify: FnOnce(&Path) -> Result<VerifiedRom>,
    FBootstrap: FnOnce(&Path) -> Result<()>,
    FSync: FnOnce(&Path, &Path) -> Result<()>,
    FSplit: FnOnce(&Path) -> Result<()>,
    FExport: FnOnce(&Path, &Path, &str, &str) -> Result<AssetManifest>,
{
    let _lock = acquire_prepare_lock(repo_root)?;
    let verified = verify_rom_fn(rom_path)?;
    let bundle_dir = generated_bundle_dir(repo_root, &verified.sha1);
    let bundle_manifest_path = generated_manifest_path(&bundle_dir);
    if bundle_manifest_path.is_file() {
        let manifest = load_manifest(&bundle_manifest_path)?;
        if ensure_generated_bundle_contract(&manifest).is_ok() {
            return Ok(manifest);
        }
    }

    bootstrap_upstream_fn(upstream_dir)?;
    sync_rom_fn(rom_path, upstream_dir)?;
    split_fn(upstream_dir)?;
    export_bundle_fn(upstream_dir, &bundle_dir, &verified.sha1, UPSTREAM_COMMIT)
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

fn verify_native_bootstrap(manifest: &AssetManifest) -> Result<()> {
    let scenario = Scenario::BootTitleStartLevel1;
    let script = scenario.script()?;
    let trace = run_native_script_for_scenario(manifest, &script, scenario);
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
    let native_trace = run_native_script_for_scenario(manifest, &script, scenario);
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

fn compare_native_gameplay(
    repo_root: &Path,
    rom_path: &Path,
    manifest: &AssetManifest,
    scenario: GameplayScenario,
) -> Result<()> {
    let rom_bytes = fs::read(rom_path)
        .with_context(|| format!("failed to read ROM from {}", rom_path.display()))?;
    compare_native_gameplay_with(
        repo_root,
        scenario,
        || capture_gameplay_scenario(&rom_bytes, scenario),
        || capture_native_gameplay_scenario(manifest, scenario),
        || capture_native_gameplay_frame_evidence(manifest, scenario),
    )?;
    println!("native gameplay comparison passed for {}", scenario.slug());
    Ok(())
}

fn diagnose_native_gameplay_drift(
    repo_root: &Path,
    rom_path: &Path,
    manifest: &AssetManifest,
    scenario: GameplayScenario,
    from_frame: usize,
    limit: usize,
) -> Result<()> {
    let rom_bytes = fs::read(rom_path)
        .with_context(|| format!("failed to read ROM from {}", rom_path.display()))?;
    let (run_dir, native_capture, oracle_capture) = capture_native_gameplay_run(
        repo_root,
        scenario,
        || capture_gameplay_scenario(&rom_bytes, scenario),
        || capture_native_gameplay_scenario(manifest, scenario),
    )?;
    let reports = find_gameplay_live_drifts(&native_capture, &oracle_capture, scenario, from_frame);
    let reports = reports.into_iter().take(limit.max(1)).collect::<Vec<_>>();

    println!("scenario: {}", scenario.slug());
    println!("run dir: {}", run_dir.display());
    if reports.is_empty() {
        println!("no live drift from frame {from_frame}");
        return Ok(());
    }

    for report in reports {
        println!(
            "frame {} [{}] fields: {}",
            report.frame_index,
            report.window,
            report.differing_fields.join(", ")
        );
        for field in &report.differing_fields {
            println!(
                "  {field}: native {} oracle {}",
                gameplay_field_value(&report.native, field),
                gameplay_field_value(&report.oracle, field)
            );
        }
    }
    Ok(())
}

fn capture_native_gameplay_run<FO, FN>(
    repo_root: &Path,
    scenario: GameplayScenario,
    oracle_capture_fn: FO,
    native_capture_fn: FN,
) -> Result<(PathBuf, GameplayCapture, GameplayCapture)>
where
    FO: FnOnce() -> Result<GameplayCapture>,
    FN: FnOnce() -> Result<GameplayCapture>,
{
    let run_dir = repo_root
        .join(".cache/native-gameplay-oracle-runs")
        .join(scenario.slug());
    let native_dir = run_dir.join("native");
    let oracle_dir = run_dir.join("oracle");
    let review_dir = run_dir.join("review");
    if native_dir.exists() {
        fs::remove_dir_all(&native_dir)
            .with_context(|| format!("failed to remove {}", native_dir.display()))?;
    }
    if oracle_dir.exists() {
        fs::remove_dir_all(&oracle_dir)
            .with_context(|| format!("failed to remove {}", oracle_dir.display()))?;
    }
    if review_dir.exists() {
        fs::remove_dir_all(&review_dir)
            .with_context(|| format!("failed to remove {}", review_dir.display()))?;
    }
    fs::create_dir_all(&native_dir)
        .with_context(|| format!("failed to create {}", native_dir.display()))?;
    fs::create_dir_all(&oracle_dir)
        .with_context(|| format!("failed to create {}", oracle_dir.display()))?;

    let oracle_capture = oracle_capture_fn()?;
    write_gameplay_capture(&oracle_dir, &oracle_capture)?;

    let native_capture = native_capture_fn()?;
    write_gameplay_capture(&native_dir, &native_capture)?;

    Ok((run_dir, native_capture, oracle_capture))
}

fn compare_native_gameplay_with<FO, FN, FE>(
    repo_root: &Path,
    scenario: GameplayScenario,
    oracle_capture_fn: FO,
    native_capture_fn: FN,
    frame_evidence_fn: FE,
) -> Result<()>
where
    FO: FnOnce() -> Result<GameplayCapture>,
    FN: FnOnce() -> Result<GameplayCapture>,
    FE: FnOnce() -> Result<Vec<GameplayFrameEvidence>>,
{
    let (run_dir, native_capture, oracle_capture) =
        capture_native_gameplay_run(repo_root, scenario, oracle_capture_fn, native_capture_fn)?;
    compare_gameplay_checkpoints_against_oracle(
        &run_dir,
        scenario,
        &native_capture.checkpoints,
        &oracle_capture.checkpoints,
    )?;
    let frame_evidence = frame_evidence_fn()?;
    write_gameplay_frame_evidence(&run_dir.join("review"), &frame_evidence)?;
    Ok(())
}

fn capture_native_gameplay_frame_evidence(
    manifest: &AssetManifest,
    scenario: GameplayScenario,
) -> Result<Vec<GameplayFrameEvidence>> {
    let script = scenario.script()?;
    let trace = run_native_script_for_gameplay(manifest, &script, scenario);
    render_gameplay_frame_evidence(manifest, &trace, scenario.checkpoints())
}

fn run_native_script_for_scenario(
    manifest: &AssetManifest,
    script: &[u8],
    scenario: Scenario,
) -> Vec<NativeTraceFrame> {
    run_script(manifest.clone(), script, scenario.frame_count())
}

fn run_native_script_for_gameplay(
    manifest: &AssetManifest,
    script: &[u8],
    scenario: GameplayScenario,
) -> Vec<NativeTraceFrame> {
    run_script(manifest.clone(), script, scenario.frame_count())
}

#[derive(Debug, Clone, Copy)]
struct GameplayCheckpointTolerance {
    camera_x: u32,
    camera_y: u32,
    kid_x: u32,
    kid_y: u32,
    number_objects: u16,
    number_gfx_objects: u16,
}

fn compare_gameplay_checkpoints_against_oracle(
    run_dir: &Path,
    scenario: GameplayScenario,
    native_states: &[GameplayState],
    oracle_states: &[GameplayState],
) -> Result<()> {
    if native_states.len() != oracle_states.len() {
        bail!(
            "native captured {} gameplay checkpoints but oracle captured {}; artefacts in {}",
            native_states.len(),
            oracle_states.len(),
            run_dir.display()
        );
    }

    for (native, oracle) in native_states.iter().zip(oracle_states) {
        if native.label != oracle.label || native.frame_index != oracle.frame_index {
            bail!(
                "gameplay checkpoint mismatch: native {}@{} vs oracle {}@{}",
                native.label,
                native.frame_index,
                oracle.label,
                oracle.frame_index
            );
        }
        if native.game_mode != oracle.game_mode {
            bail!(
                "gameplay state {}@{} diverged in game mode (native {:04X}, oracle {:04X}); artefacts in {}",
                native.label,
                native.frame_index,
                native.game_mode,
                oracle.game_mode,
                run_dir.display()
            );
        }
        if native.current_player != oracle.current_player {
            bail!(
                "gameplay state {}@{} diverged in current player (native {:02X}, oracle {:02X}); artefacts in {}",
                native.label,
                native.frame_index,
                native.current_player,
                oracle.current_player,
                run_dir.display()
            );
        }
        if native.number_lives != oracle.number_lives {
            bail!(
                "gameplay state {}@{} diverged in lives (native {}, oracle {}); artefacts in {}",
                native.label,
                native.frame_index,
                native.number_lives,
                oracle.number_lives,
                run_dir.display()
            );
        }
        if native.number_hitpoints != oracle.number_hitpoints
            || native.current_helmet_id != oracle.current_helmet_id
        {
            bail!(
                "gameplay state {}@{} diverged in form state (native hp {} helmet {:04X}, oracle hp {} helmet {:04X}); artefacts in {}",
                native.label,
                native.frame_index,
                native.number_hitpoints,
                native.current_helmet_id,
                oracle.number_hitpoints,
                oracle.current_helmet_id,
                run_dir.display()
            );
        }
        if native.current_level_id != oracle.current_level_id
            || native.player_1_level_id != oracle.player_1_level_id
        {
            bail!(
                "gameplay state {}@{} diverged in level ids (native current {:04X} / p1 {:04X}, oracle current {:04X} / p1 {:04X}); artefacts in {}",
                native.label,
                native.frame_index,
                native.current_level_id,
                native.player_1_level_id,
                oracle.current_level_id,
                oracle.player_1_level_id,
                run_dir.display()
            );
        }

        let tolerance = gameplay_checkpoint_tolerance(scenario, &native.label);
        compare_u32_delta(
            run_dir,
            "camera X",
            native,
            native.camera_x_pos,
            oracle.camera_x_pos,
            tolerance.camera_x,
        )?;
        compare_u32_delta(
            run_dir,
            "camera Y",
            native,
            native.camera_y_pos,
            oracle.camera_y_pos,
            tolerance.camera_y,
        )?;
        compare_u32_delta(
            run_dir,
            "kid X",
            native,
            native.kid_x_pos,
            oracle.kid_x_pos,
            tolerance.kid_x,
        )?;
        compare_u32_delta(
            run_dir,
            "kid Y",
            native,
            native.kid_y_pos,
            oracle.kid_y_pos,
            tolerance.kid_y,
        )?;
        compare_u16_delta(
            run_dir,
            "object counts",
            native,
            native.number_objects,
            oracle.number_objects,
            tolerance.number_objects,
        )?;
        compare_u16_delta(
            run_dir,
            "gfx object counts",
            native,
            native.number_gfx_objects,
            oracle.number_gfx_objects,
            tolerance.number_gfx_objects,
        )?;
    }

    Ok(())
}

fn gameplay_checkpoint_tolerance(
    scenario: GameplayScenario,
    label: &str,
) -> GameplayCheckpointTolerance {
    match (scenario, label) {
        (GameplayScenario::PauseResumeLevel1, "resumed_tail") => GameplayCheckpointTolerance {
            camera_x: 0x0020_0000,
            camera_y: 0x0060_0000,
            kid_x: 0x0020_0000,
            kid_y: 0x0060_0000,
            number_objects: 6,
            number_gfx_objects: 6,
        },
        (GameplayScenario::PauseResumeLevel1, _) => GameplayCheckpointTolerance {
            camera_x: 0x0020_0000,
            camera_y: CAMERA_POSITION_TOLERANCE,
            kid_x: 0x0020_0000,
            kid_y: CAMERA_POSITION_TOLERANCE,
            number_objects: 6,
            number_gfx_objects: 6,
        },
        (GameplayScenario::DeathRespawnLevel1, "pre_death") => GameplayCheckpointTolerance {
            camera_x: 0x0030_0000,
            camera_y: 0x0060_0000,
            kid_x: 0x0030_0000,
            kid_y: 0x0060_0000,
            number_objects: 8,
            number_gfx_objects: 8,
        },
        _ => GameplayCheckpointTolerance {
            camera_x: CAMERA_POSITION_TOLERANCE,
            camera_y: CAMERA_POSITION_TOLERANCE,
            kid_x: CAMERA_POSITION_TOLERANCE,
            kid_y: CAMERA_POSITION_TOLERANCE,
            number_objects: 1,
            number_gfx_objects: 1,
        },
    }
}

fn compare_u32_delta(
    run_dir: &Path,
    field: &str,
    native: &GameplayState,
    native_value: u32,
    oracle_value: u32,
    tolerance: u32,
) -> Result<()> {
    if native_value.abs_diff(oracle_value) > tolerance {
        bail!(
            "gameplay state {}@{} diverged in {} (native {:08X}, oracle {:08X}); artefacts in {}",
            native.label,
            native.frame_index,
            field,
            native_value,
            oracle_value,
            run_dir.display()
        );
    }
    Ok(())
}

fn compare_u16_delta(
    run_dir: &Path,
    field: &str,
    native: &GameplayState,
    native_value: u16,
    oracle_value: u16,
    tolerance: u16,
) -> Result<()> {
    if native_value.abs_diff(oracle_value) > tolerance {
        bail!(
            "gameplay state {}@{} diverged in {} (native {}, oracle {}); artefacts in {}",
            native.label,
            native.frame_index,
            field,
            native_value,
            oracle_value,
            run_dir.display()
        );
    }
    Ok(())
}

fn find_gameplay_live_drifts(
    native_capture: &GameplayCapture,
    oracle_capture: &GameplayCapture,
    scenario: GameplayScenario,
    from_frame: usize,
) -> Vec<GameplayDriftReport> {
    native_capture
        .states
        .iter()
        .zip(&oracle_capture.states)
        .filter(|(native, oracle)| {
            native.frame_index >= from_frame && native.frame_index == oracle.frame_index
        })
        .filter_map(|(native, oracle)| {
            let differing_fields = gameplay_differing_fields(native, oracle);
            if differing_fields.is_empty() {
                None
            } else {
                Some(GameplayDriftReport {
                    frame_index: native.frame_index,
                    window: gameplay_checkpoint_window(scenario, native.frame_index),
                    differing_fields,
                    native: native.clone(),
                    oracle: oracle.clone(),
                })
            }
        })
        .collect()
}

fn gameplay_differing_fields(native: &GameplayState, oracle: &GameplayState) -> Vec<&'static str> {
    let mut fields = Vec::new();
    if native.game_mode != oracle.game_mode {
        fields.push("game_mode");
    }
    if native.current_player != oracle.current_player {
        fields.push("current_player");
    }
    if native.number_lives != oracle.number_lives {
        fields.push("number_lives");
    }
    if native.number_hitpoints != oracle.number_hitpoints {
        fields.push("number_hitpoints");
    }
    if native.current_level_id != oracle.current_level_id {
        fields.push("current_level_id");
    }
    if native.player_1_level_id != oracle.player_1_level_id {
        fields.push("player_1_level_id");
    }
    if native.current_helmet_id != oracle.current_helmet_id {
        fields.push("current_helmet_id");
    }
    if native.camera_x_pos.abs_diff(oracle.camera_x_pos) > CAMERA_POSITION_TOLERANCE {
        fields.push("camera_x_pos");
    }
    if native.camera_y_pos.abs_diff(oracle.camera_y_pos) > CAMERA_POSITION_TOLERANCE {
        fields.push("camera_y_pos");
    }
    if native.kid_x_pos.abs_diff(oracle.kid_x_pos) > CAMERA_POSITION_TOLERANCE {
        fields.push("kid_x_pos");
    }
    if native.kid_y_pos.abs_diff(oracle.kid_y_pos) > CAMERA_POSITION_TOLERANCE {
        fields.push("kid_y_pos");
    }
    fields
}

fn gameplay_checkpoint_window(scenario: GameplayScenario, frame_index: usize) -> String {
    let checkpoints = scenario.checkpoints();
    let previous = checkpoints
        .iter()
        .rev()
        .find(|checkpoint| checkpoint.frame_index <= frame_index);
    let next = checkpoints
        .iter()
        .find(|checkpoint| checkpoint.frame_index >= frame_index);
    match (previous, next) {
        (Some(previous), Some(next)) if previous.frame_index == next.frame_index => {
            format!("{}@{}", previous.label, previous.frame_index)
        }
        (Some(previous), Some(next)) => {
            format!(
                "{}@{} -> {}@{}",
                previous.label, previous.frame_index, next.label, next.frame_index
            )
        }
        (Some(previous), None) => format!("after {}@{}", previous.label, previous.frame_index),
        (None, Some(next)) => format!("before {}@{}", next.label, next.frame_index),
        (None, None) => String::from("no checkpoints"),
    }
}

fn gameplay_field_value(state: &GameplayState, field: &str) -> String {
    match field {
        "game_mode" => format!("{:04X}", state.game_mode),
        "current_player" => state.current_player.to_string(),
        "number_lives" => state.number_lives.to_string(),
        "number_hitpoints" => state.number_hitpoints.to_string(),
        "current_level_id" => format!("{:04X}", state.current_level_id),
        "player_1_level_id" => format!("{:04X}", state.player_1_level_id),
        "current_helmet_id" => format!("{:04X}", state.current_helmet_id),
        "camera_x_pos" => format!("{:08X}", state.camera_x_pos),
        "camera_y_pos" => format!("{:08X}", state.camera_y_pos),
        "kid_x_pos" => format!("{:08X}", state.kid_x_pos),
        "kid_y_pos" => format!("{:08X}", state.kid_y_pos),
        "number_objects" => state.number_objects.to_string(),
        "number_gfx_objects" => state.number_gfx_objects.to_string(),
        _ => String::from("<unknown>"),
    }
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
        number_hitpoints: frame.number_hitpoints,
        current_helmet_id: frame.current_helmet_id,
        current_level_id: u16::from(frame.current_level.0),
        player_1_level_id: u16::from(frame.player_1_level_id.0),
        camera_x_pos: frame.camera_x_pos,
        camera_y_pos: frame.camera_y_pos,
        kid_x_pos: frame.kid_x_pos.unwrap_or(0),
        kid_y_pos: frame.kid_y_pos.unwrap_or(0),
        number_objects: frame.number_objects,
        number_gfx_objects: frame.number_gfx_objects,
        active_enemies: active_enemy_states_from_native_frame(frame),
    }
}

fn active_enemy_states_from_native_frame(frame: &NativeTraceFrame) -> Vec<OracleEnemyState> {
    let mut enemies = frame
        .active_objects
        .iter()
        .filter_map(|object| match object.kind {
            kid_core::NativeObjectKind::Enemy { object_type } => Some(OracleEnemyState {
                object_type,
                x_pos: object.x_pos,
                y_pos: object.y_pos,
                x_velocity: object.x_velocity.unwrap_or(0),
                y_velocity: object.y_velocity.unwrap_or(0),
                facing: enemy_facing_from_native(object.facing),
            }),
            _ => None,
        })
        .collect::<Vec<_>>();
    sort_enemy_states(&mut enemies);
    enemies
}

fn enemy_facing_from_native(facing: Option<NativeDirection>) -> EnemyFacing {
    match facing.unwrap_or(NativeDirection::Left) {
        NativeDirection::Left => EnemyFacing::Left,
        NativeDirection::Right => EnemyFacing::Right,
    }
}

fn sort_enemy_states(enemies: &mut [OracleEnemyState]) {
    enemies.sort_by_key(|enemy| (enemy.object_type, enemy.x_pos, enemy.y_pos));
}

fn compare_native_states_against_oracle(
    run_dir: &Path,
    native_states: &[OracleState],
    oracle_states: &[OracleState],
) -> Result<()> {
    const ENEMY_POSITION_TOLERANCE: u32 = 0x0010_0000;
    const ENEMY_VELOCITY_TOLERANCE: i32 = 0x0001_0000;
    let compare_enemy_snapshots = oracle_states
        .iter()
        .any(|state| !state.active_enemies.is_empty());
    let native_enemy_labels = native_states
        .iter()
        .filter(|state| !state.active_enemies.is_empty())
        .map(|state| format!("{}@{}", state.label, state.frame_index))
        .collect::<Vec<_>>();

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
        if native.number_hitpoints != oracle.number_hitpoints
            || native.current_helmet_id != oracle.current_helmet_id
        {
            bail!(
                "state {}@{} diverged in form state (native hp {} helmet {:04X}, oracle hp {} helmet {:04X}); artefacts in {}",
                native.label,
                native.frame_index,
                native.number_hitpoints,
                native.current_helmet_id,
                oracle.number_hitpoints,
                oracle.current_helmet_id,
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
        let kid_delta_x = native.kid_x_pos.abs_diff(oracle.kid_x_pos);
        let kid_delta_y = native.kid_y_pos.abs_diff(oracle.kid_y_pos);
        if kid_delta_x > CAMERA_POSITION_TOLERANCE || kid_delta_y > CAMERA_POSITION_TOLERANCE {
            bail!(
                "state {}@{} diverged in kid position (native {:08X},{:08X}; oracle {:08X},{:08X}); artefacts in {}",
                native.label,
                native.frame_index,
                native.kid_x_pos,
                native.kid_y_pos,
                oracle.kid_x_pos,
                oracle.kid_y_pos,
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
        if compare_enemy_snapshots {
            compare_enemy_snapshots_against_oracle(
                run_dir,
                native.label.as_str(),
                native.frame_index,
                &native.active_enemies,
                &oracle.active_enemies,
                ENEMY_POSITION_TOLERANCE,
                ENEMY_VELOCITY_TOLERANCE,
            )?;
        }
    }

    if !compare_enemy_snapshots {
        if native_enemy_labels.is_empty() {
            eprintln!(
                "warning: oracle captured no enemy snapshots; skipped per-enemy comparison for {}",
                run_dir.display()
            );
        } else {
            eprintln!(
                "warning: oracle captured no Diamond/MiniHoppingSkull snapshots, but native captured enemy snapshots at {}; skipped per-enemy comparison for {}",
                native_enemy_labels.join(", "),
                run_dir.display()
            );
        }
    }

    Ok(())
}

fn compare_enemy_snapshots_against_oracle(
    run_dir: &Path,
    label: &str,
    frame_index: usize,
    native_enemies: &[OracleEnemyState],
    oracle_enemies: &[OracleEnemyState],
    position_tolerance: u32,
    velocity_tolerance: i32,
) -> Result<()> {
    if native_enemies.len() != oracle_enemies.len() {
        bail!(
            "state {}@{} diverged in enemy count (native {}, oracle {}); artefacts in {}",
            label,
            frame_index,
            native_enemies.len(),
            oracle_enemies.len(),
            run_dir.display()
        );
    }

    for (native, oracle) in native_enemies.iter().zip(oracle_enemies) {
        if native.object_type != oracle.object_type {
            bail!(
                "state {}@{} diverged in enemy type ordering (native {:02X}, oracle {:02X}); artefacts in {}",
                label,
                frame_index,
                native.object_type,
                oracle.object_type,
                run_dir.display()
            );
        }
        if native.x_pos.abs_diff(oracle.x_pos) > position_tolerance
            || native.y_pos.abs_diff(oracle.y_pos) > position_tolerance
        {
            bail!(
                "state {}@{} diverged in enemy {:02X} position (native {:08X},{:08X}; oracle {:08X},{:08X}); artefacts in {}",
                label,
                frame_index,
                native.object_type,
                native.x_pos,
                native.y_pos,
                oracle.x_pos,
                oracle.y_pos,
                run_dir.display()
            );
        }
        if (native.x_velocity - oracle.x_velocity).abs() > velocity_tolerance
            || (native.y_velocity - oracle.y_velocity).abs() > velocity_tolerance
        {
            bail!(
                "state {}@{} diverged in enemy {:02X} velocity (native {},{}; oracle {},{}); artefacts in {}",
                label,
                frame_index,
                native.object_type,
                native.x_velocity,
                native.y_velocity,
                oracle.x_velocity,
                oracle.y_velocity,
                run_dir.display()
            );
        }
        if native.facing != oracle.facing {
            bail!(
                "state {}@{} diverged in enemy {:02X} facing (native {:?}, oracle {:?}); artefacts in {}",
                label,
                frame_index,
                native.object_type,
                native.facing,
                oracle.facing,
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

fn verify_native_gameplay_scenario(
    repo_root: &Path,
    manifest: &AssetManifest,
    scenario: GameplayScenario,
) -> Result<()> {
    let capture = capture_native_gameplay_scenario(manifest, scenario)?;
    write_native_gameplay_artifacts(repo_root, &capture)?;
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
    write_gameplay_capture(&out_dir, capture)
}

fn write_native_gameplay_artifacts(repo_root: &Path, capture: &GameplayCapture) -> Result<()> {
    let out_dir = repo_root
        .join(".cache/native-gameplay-runs")
        .join(&capture.scenario);
    write_gameplay_capture(&out_dir, capture)
}

fn write_gameplay_capture(out_dir: &Path, capture: &GameplayCapture) -> Result<()> {
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

fn write_gameplay_frame_evidence(out_dir: &Path, frames: &[GameplayFrameEvidence]) -> Result<()> {
    fs::create_dir_all(out_dir)
        .with_context(|| format!("failed to create {}", out_dir.display()))?;
    let mut records = Vec::with_capacity(frames.len());
    for (label, frame_index, width, height, rgb) in frames {
        let png_name = format!("{frame_index:04}_{label}.png");
        let path = out_dir.join(&png_name);
        image::save_buffer_with_format(
            &path,
            rgb,
            *width as u32,
            *height as u32,
            ColorType::Rgb8,
            ImageFormat::Png,
        )
        .with_context(|| format!("failed to write {}", path.display()))?;
        records.push(GameplayFrameEvidenceRecord {
            label: label.clone(),
            frame_index: *frame_index,
            png_path: png_name,
            sha1: sha1_hex(rgb),
        });
    }
    fs::write(
        out_dir.join("frame-evidence.json"),
        serde_json::to_vec_pretty(&records)?,
    )
    .with_context(|| {
        format!(
            "failed to write {}",
            out_dir.join("frame-evidence.json").display()
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
        let kid_delta_x = native.kid_x_pos.abs_diff(oracle.kid_x_pos);
        let kid_delta_y = native.kid_y_pos.abs_diff(oracle.kid_y_pos);
        if kid_delta_x > CAMERA_POSITION_TOLERANCE || kid_delta_y > CAMERA_POSITION_TOLERANCE {
            bail!(
                "state {}@{} diverged in kid position (native {:08X},{:08X}; oracle {:08X},{:08X}); artefacts in {}",
                native.label,
                native.frame_index,
                native.kid_x_pos,
                native.kid_y_pos,
                oracle.kid_x_pos,
                oracle.kid_y_pos,
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
        number_hitpoints: state.number_hitpoints,
        current_helmet_id: state.current_helmet_id,
        current_level_id: state.current_level_id,
        player_1_level_id: state.player_1_level_id,
        camera_x_pos: state.camera_x_pos,
        camera_y_pos: state.camera_y_pos,
        kid_x_pos: state.kid_x_pos,
        kid_y_pos: state.kid_y_pos,
        number_objects: state.number_objects,
        number_gfx_objects: state.number_gfx_objects,
        active_enemies: state.active_enemies.clone(),
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
