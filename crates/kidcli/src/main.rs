#![allow(
    clippy::too_many_arguments,
    clippy::type_complexity,
    clippy::doc_lazy_continuation
)]
mod asm_coverage;
mod autonomy;
mod bootstrap;
mod cache;
mod capture;
mod case;
mod cheats;
mod compare;
mod diff;
mod hypothesis;
mod inspect;
mod native_player;
mod oracle;
mod player;
mod project_status;
mod ram_diff;
mod recording;
mod release_gate;
mod scenarios;
mod sweep;
mod tas;
mod tas_diff;
mod tui_debugger;
mod verify;
mod view_tas;

use verify::*;

use bootstrap::*;
use capture::*;
use case::{CaseCommand, run_case_command};
use compare::*;
use inspect::*;

use anyhow::{Context, Result, anyhow, bail};
use autonomy::{AutonomyGoal, run_autonomous_loop};
use clap::{Parser, Subcommand, ValueEnum};
use image::{ColorType, ImageFormat};
use kid_core::physics::Direction as NativeDirection;
use kid_core::{
    ActiveProjectileKind, FrameInput, GameMode as NativeGameMode, LevelId, NativeRuntime,
    NativeTraceFrame, PlayerForm, run_script,
};
use kid_data::{AssetManifest, find_enemy_definition, load_level_asset_bundle, resource_level_id};
use kid_md::{MdRuntime, probe_rom, probe_rom_with_script, render_frame};
use kid_rom::{RomIdentity, VerifiedRom, find_default_rom, known_sha1, verify_rom};
use native_player::{
    NativePlayOptions, render_dynamic_gameplay_frame_evidence, render_gameplay_frame_evidence,
    run_native_player,
};
use oracle::{
    EnemyFacing, OracleEnemyLayoutTraceManifest, OracleEnemyState, OracleEnvironment,
    OracleManifest, OracleState, ensure_oracles, run_jgenesis_capture,
    run_jgenesis_enemy_layout_trace,
};
use player::{PlayOptions, run_player};
use project_status::{ProjectStatusFormat, run_project_status};
use recording::{resolve_recording_bundle_input, validate_recording_slug, verify_recording};
use release_gate::{ReleaseGateTarget, run_release_gate};
use scenarios::{
    DynamicScenario, GameplayScenario, NativeDynamicRunOptions, Scenario,
    diamond_power_circle_audit_scenario, diamond_power_invulnerability_audit_scenario,
    diamond_power_samurai_haze_audit_scenario, diamond_power_snake_audit_scenario,
    diamond_power_wall_audit_scenario, enemy_spawn_audit_scenario,
    highwater_flag_transition_audit_scenario, highwater_pass2_grow_shooter_touch_audit_scenario,
    highwater_pass2_juggernaut_flag_transition_audit_scenario,
    highwater_shooter_barrier_audit_scenario, highwater_water_priority_audit_scenario,
    idle_level_scenario, issue_147_wall_side_jump_audit_scenario,
    issue_213_drips_delayed_b_drop_audit_scenario, issue_213_drips_metal_block_audit_scenario,
    issue_216_prize_audio_audit_scenario, issue_249_blw2_left_pit_wall_jump_audit_scenario,
    issue_249_blw2_left_pit_wall_no_jump_audit_scenario,
    issue_249_juggernaut_wall_jump_audit_scenario,
    issue_249_juggernaut_wall_no_jump_audit_scenario, issue_249_wall_no_jump_audit_scenario,
    lava_weather_audit_scenario, level_00_slope_right_probe_iron_knight_scenario,
    level_00_slope_right_probe_juggernaut_scenario,
    level_00_slope_right_probe_red_stealth_scenario, level_00_slope_right_probe_scenario,
    prize_block_scenario, run_native_dynamic_scenario, stomp_enemy_scenario,
    walk_right_iron_knight_scenario, walk_right_juggernaut_scenario,
    walk_right_red_stealth_scenario, walk_right_scenario,
};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::thread;
use std::time::{Duration, Instant};
use ymfm_sys::Ym2612;

#[derive(Clone, Copy, Debug, ValueEnum)]
enum TargetProjectileKind {
    DripsDrop,
    DripsSurfaceCrawler,
}

impl From<TargetProjectileKind> for ActiveProjectileKind {
    fn from(value: TargetProjectileKind) -> Self {
        match value {
            TargetProjectileKind::DripsDrop => ActiveProjectileKind::DripsDrop,
            TargetProjectileKind::DripsSurfaceCrawler => ActiveProjectileKind::DripsSurfaceCrawler,
        }
    }
}

#[derive(Debug, Parser)]
#[command(name = "kidcli")]
#[command(about = "Contained tooling for the Kid Chameleon PC-port workspace")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
enum RecordScenarioKind {
    /// idle for 5 seconds — captures pure level-settle state.
    Idle,
    /// walk right (+ periodic jumps) for 5 seconds.
    WalkRight,
    /// walk right as Red Stealth/Samurai for natural form animation probes.
    WalkRightRedStealth,
    /// walk right as Iron Knight for natural form animation probes.
    WalkRightIronKnight,
    /// walk right as Juggernaut/Rhino for natural form animation probes.
    WalkRightJuggernaut,
    /// walk right and jump under prize blocks.
    PrizeBlock,
    /// start under the Level 0 prize cluster to audit block/diamond SFX without damage.
    Issue216PrizeAudioAudit,
    /// start on Blue Lake Woods 1's first `$4000` slope and hold Right.
    Level00SlopeRightProbe,
    /// start Red Stealth/Samurai on Blue Lake Woods 1's first `$4000` slope and hold Right.
    Level00SlopeRightProbeRedStealth,
    /// start Iron Knight on Blue Lake Woods 1's first `$4000` slope and hold Right.
    Level00SlopeRightProbeIronKnight,
    /// start Juggernaut/Rhino on Blue Lake Woods 1's first `$4000` slope and hold Right.
    Level00SlopeRightProbeJuggernaut,
    /// run + aggressive jumps to maximise enemy stomps.
    StompEnemy,
    /// idle 10 s in-arena to sample enemy roster across the spawn window —
    /// pair with `--level 0x33` to probe HeadyMetal (kind 0x20) on Plethora 1.
    EnemySpawnAuditHeadyMetal,
    /// idle in a settled lava level to sample special-effect object workers.
    LavaWeatherAudit,
    /// seed 20 diamonds and hold Start+A to audit The Kid's Circle of Doom.
    DiamondPowerCircleAudit,
    /// seed 50 diamonds and hold Start+A to audit Berzerker's Wall of Death.
    DiamondPowerWallAudit,
    /// seed 20 diamonds and hold Start+A to audit Micromax's MiniSnake.
    DiamondPowerSnakeAudit,
    /// seed 20 diamonds and hold Start+A to audit Skycutter's Invulnerability.
    DiamondPowerInvulnerabilityAudit,
    /// seed 20 diamonds and hold Start+A to audit Red Stealth's Samurai Haze.
    DiamondPowerSamuraiHazeAudit,
    /// drive Berzerker through Highwater Pass 1's first shooter/barrier stack.
    HighwaterShooterBarrierAudit,
    /// walk the Level 02 spawn route to the first water/foreground overlap.
    HighwaterWaterPriorityAudit,
    /// start at Highwater Pass 1's flag and audit the normal next-level hand-off.
    HighwaterFlagTransitionAudit,
    /// start at Highwater Pass 2's flag as Juggernaut and audit the normal level-4 hand-off.
    HighwaterPass2JuggernautFlagTransitionAudit,
    /// start above Highwater Pass 2's first `$E919` touch-grown shooter block.
    HighwaterPass2GrowShooterTouchAudit,
    /// drive Red Stealth into a Level 02 wall and jump (#147 wall-grab "zoom to top" audit).
    Issue147WallSideJumpAudit,
    /// preserve the current #155/#213 direct Drips/metal-block blocker probe.
    Issue213DripsMetalBlockAudit,
    /// delayed Level 01 B-jump that activates Drips drops before the fall tail (#155/#213).
    Issue213DripsDelayedBDropAudit,
    /// drive The Kid into a Level 02 wall without jump/climb input (#249).
    Issue249WallNoJumpAudit,
    /// drive Juggernaut/Rhino into a Level 02 wall without jump/climb input (#249).
    Issue249JuggernautWallNoJumpAudit,
    /// drive Juggernaut/Rhino into a Level 02 wall while pulsing jump (#249).
    Issue249JuggernautWallJumpAudit,
    /// drive Juggernaut/Rhino into Blue Lake Woods 2's left pit wall without jump (#249).
    Issue249Blw2LeftPitWallNoJumpAudit,
    /// drive Juggernaut/Rhino into Blue Lake Woods 2's left pit wall while pulsing jump (#249).
    Issue249Blw2LeftPitWallJumpAudit,
}

#[derive(Debug, Subcommand)]
enum Commands {
    /// Semantic case-bundle workflow for branching and reduction.
    Case {
        #[command(subcommand)]
        command: CaseCommand,
    },
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
    /// Record a recording bundle from a scripted scenario without an
    /// interactive session. Bypasses the play-native --settled --record
    /// trap that has stalled multiple [~]/[?] backlog repros (#135, #136,
    /// #137, #140, #143) by warping straight to the level and saving the
    /// manifest deterministically.
    RecordScenario {
        /// Recording slug — bundle is written to .cache/recordings/<slug>/.
        #[arg(long)]
        slug: String,
        /// Scenario kind.
        #[arg(long, value_enum, default_value_t = RecordScenarioKind::WalkRight)]
        scenario: RecordScenarioKind,
        /// Level to warp into.
        #[arg(long)]
        level: u8,
        /// Override the default scenario frame count (post-settle frames).
        #[arg(long)]
        walk_frames: Option<usize>,
        /// Optional ROM path (defaults to repo-root resolution).
        #[arg(long)]
        rom: Option<PathBuf>,
        /// Optional upstream-dir path.
        #[arg(long)]
        dir: Option<PathBuf>,
    },
    /// Record a native review bundle from a DynamicScenario JSON.
    RecordScenarioFile {
        /// Recording slug — bundle is written to .cache/recordings/<slug>/.
        #[arg(long)]
        slug: String,
        /// Path to the DynamicScenario JSON to replay.
        #[arg(long)]
        scenario: PathBuf,
        /// Optional ROM path (defaults to repo-root resolution).
        #[arg(long)]
        rom: Option<PathBuf>,
        /// Optional upstream-dir path.
        #[arg(long)]
        dir: Option<PathBuf>,
    },
    /// Probe a scripted recording scenario without writing a recording bundle.
    ProbeRecordScenario {
        /// Scenario kind.
        #[arg(long, value_enum, default_value_t = RecordScenarioKind::WalkRight)]
        scenario: RecordScenarioKind,
        /// Level to warp into.
        #[arg(long)]
        level: u8,
        /// Override the default scenario frame count (post-settle frames).
        #[arg(long)]
        walk_frames: Option<usize>,
        /// Overlay input ranges, e.g. `60-600:right+c` or `120:b`.
        #[arg(long)]
        input: Vec<String>,
        #[arg(long, default_value_t = false)]
        trace_physics: bool,
        /// Include verbose gameplay debug fields in frame-range output.
        #[arg(long, default_value_t = false)]
        debug_fields: bool,
        /// Summarise activation windows for one enemy object type, e.g. 16 for Drips.
        #[arg(long, value_parser = parse_level_id)]
        target_enemy: Option<u8>,
        /// Summarise typed projectile windows, e.g. drips-drop or drips-surface-crawler.
        #[arg(long)]
        target_projectile: Option<TargetProjectileKind>,
        /// Summarise live player movement bounds and first transition frame.
        #[arg(long)]
        player_motion: bool,
        /// Print scenario/checkpoint/target summaries without per-frame trace rows.
        #[arg(long, default_value_t = false)]
        summary_only: bool,
        #[arg(long)]
        frame_start: Option<usize>,
        #[arg(long)]
        frame_end: Option<usize>,
        /// Optional ROM path (defaults to repo-root resolution).
        #[arg(long)]
        rom: Option<PathBuf>,
        /// Optional upstream-dir path.
        #[arg(long)]
        dir: Option<PathBuf>,
    },
    /// Probe a DynamicScenario JSON without writing a recording bundle.
    ProbeRecordScenarioFile {
        /// Path to the DynamicScenario JSON to replay.
        #[arg(long)]
        scenario: PathBuf,
        /// Overlay input ranges, e.g. `60-600:right+c` or `120:b`.
        #[arg(long)]
        input: Vec<String>,
        #[arg(long, default_value_t = false)]
        trace_physics: bool,
        /// Include verbose gameplay debug fields in frame-range output.
        #[arg(long, default_value_t = false)]
        debug_fields: bool,
        /// Summarise activation windows for one enemy object type, e.g. 16 for Drips.
        #[arg(long, value_parser = parse_level_id)]
        target_enemy: Option<u8>,
        /// Summarise typed projectile windows, e.g. drips-drop or drips-surface-crawler.
        #[arg(long)]
        target_projectile: Option<TargetProjectileKind>,
        /// Summarise live player movement bounds and first transition frame.
        #[arg(long)]
        player_motion: bool,
        /// Print scenario/checkpoint/target summaries without per-frame trace rows.
        #[arg(long, default_value_t = false)]
        summary_only: bool,
        #[arg(long)]
        frame_start: Option<usize>,
        #[arg(long)]
        frame_end: Option<usize>,
        /// Optional ROM path (defaults to repo-root resolution).
        #[arg(long)]
        rom: Option<PathBuf>,
        /// Optional upstream-dir path.
        #[arg(long)]
        dir: Option<PathBuf>,
    },
    /// Launch the native runtime. Defaults to the full boot/title/game flow.
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
        #[arg(long)]
        warp_level: Option<u8>,
        #[arg(long, default_value_t = false)]
        settled: bool,
        #[arg(long)]
        record: Option<String>,
        /// Stop, save, and exit after this many recorded native frames.
        #[arg(long)]
        record_frames: Option<usize>,
        #[arg(long)]
        record_notes: Option<PathBuf>,
        /// Replay a DynamicScenario JSON through the live SDL player loop.
        #[arg(long)]
        scenario: Option<PathBuf>,
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
        #[arg(long)]
        frame_start: Option<usize>,
        #[arg(long)]
        frame_end: Option<usize>,
    },
    ProbeNativeGameplay {
        #[arg(long)]
        rom: Option<PathBuf>,
        #[arg(long)]
        dir: Option<PathBuf>,
        #[arg(long, value_enum, default_value_t = GameplayScenario::PauseResumeLevel1)]
        scenario: GameplayScenario,
        #[arg(long, default_value_t = false)]
        trace_physics: bool,
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
    InspectRespawnSettle {
        #[arg(long)]
        rom: Option<PathBuf>,
        #[arg(long)]
        dir: Option<PathBuf>,
        #[arg(long)]
        level: String,
        #[arg(long, default_value_t = 180)]
        steps: usize,
    },
    InspectSpawnProbe {
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
    CompareNativeGameplayWindow {
        #[arg(long)]
        rom: Option<PathBuf>,
        #[arg(long)]
        dir: Option<PathBuf>,
        #[arg(long, value_enum, default_value_t = GameplayScenario::PauseResumeLevel1)]
        scenario: GameplayScenario,
        #[arg(long)]
        frame_start: usize,
        #[arg(long)]
        frame_end: usize,
        #[arg(long, default_value_t = 1)]
        checkpoint_interval: usize,
    },
    VerifyRecording {
        #[arg(long)]
        rom: Option<PathBuf>,
        #[arg(long)]
        dir: Option<PathBuf>,
        #[arg(long)]
        slug: Option<String>,
        #[arg(long)]
        bundle: Option<PathBuf>,
        /// Only verify deterministic native replay; skip oracle state/pixel comparison.
        #[arg(long, default_value_t = false)]
        native_only: bool,
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
        #[arg(long, default_value_t = 0)]
        context_window: usize,
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
        #[arg(long, default_value_t = 1)]
        routines_per_task: u32,
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
    /// Dump an enemy family's sprite tile sheet to a PNG grid (debug; tile
    /// grid index = tile_base_index). Helps locate sub-sprite tile offsets.
    DumpEnemySheet {
        #[arg(long)]
        rom: Option<PathBuf>,
        #[arg(long)]
        dir: Option<PathBuf>,
        /// Hex level ID providing the palette/theme context (e.g. 01)
        #[arg(long)]
        level: String,
        /// Hex enemy object type (e.g. 0A for Rock Tank)
        #[arg(long)]
        enemy: String,
        /// Output PNG path
        #[arg(long)]
        output: PathBuf,
        /// Optional: render one assembled sub-sprite frame instead of the tile
        /// grid. Format: BASE,WIDTH,HEIGHT in hex (e.g. 20,1e,14 for the
        /// RockTank turret). Shows transparency as magenta.
        #[arg(long)]
        frame: Option<String>,
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
        /// Collection tier: state, frames, full, or gameplay
        #[arg(long, value_enum, default_value_t = sweep::SweepTier::State)]
        tier: sweep::SweepTier,
        /// Only populate oracle cache — skip native runs and comparison.
        #[arg(long, default_value_t = false)]
        warm_cache_only: bool,
        /// Stop after this many failures (0 = run all levels).
        #[arg(long, default_value_t = 0)]
        max_failures: usize,
        /// Record first per-frame authoritative CRC mismatch per level in report.json.
        #[arg(long, default_value_t = false)]
        checksum: bool,
        /// Capture golden state snapshots (no oracle needed).
        #[arg(long, default_value_t = false)]
        golden_capture: bool,
        /// Diff current state against golden snapshots (no oracle needed).
        #[arg(long, default_value_t = false)]
        golden_diff: bool,
    },
    /// Run the canonical release gate for a completion target.
    ReleaseGate {
        #[arg(long)]
        rom: Option<PathBuf>,
        #[arg(long)]
        dir: Option<PathBuf>,
        #[arg(long, value_enum, default_value_t = ReleaseGateTarget::OnePlayerAccurate)]
        target: ReleaseGateTarget,
        /// Stop the sweep phase after this many failures (0 = run all levels).
        #[arg(long, default_value_t = 0)]
        max_failures: usize,
    },
    /// Summarise the current repo health with 1P playability first.
    ProjectStatus {
        #[arg(long, value_enum, default_value_t = ProjectStatusFormat::Text)]
        format: ProjectStatusFormat,
        /// Exit non-zero when human validation/repro blockers remain.
        #[arg(long, default_value_t = false)]
        fail_on_human_blockers: bool,
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
    /// Render NATIVE engine frames while replaying a recording's DynamicScenario
    /// (per-frame inputs + warp/settle). Native counterpart to `render-frame`
    /// (which renders the oracle ROM); use both to compare native vs oracle
    /// sprites on a scripted route (e.g. the #211 slope-walk render probe).
    CaptureScenarioFrames {
        #[arg(long)]
        rom: Option<PathBuf>,
        #[arg(long)]
        dir: Option<PathBuf>,
        /// Path to the DynamicScenario JSON (e.g. a recording's scenario.json)
        #[arg(long)]
        scenario: PathBuf,
        /// Overlay input ranges, e.g. `60-600:right+c` or `120:b`.
        #[arg(long)]
        input: Vec<String>,
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
    /// Verify a TAS scenario against the oracle (native vs emulator comparison).
    VerifyTas {
        #[arg(long)]
        rom: Option<PathBuf>,
        #[arg(long)]
        dir: Option<PathBuf>,
        /// Path to the DynamicScenario JSON (from import-tas)
        #[arg(long)]
        scenario: PathBuf,
        /// Only compare from this frame onwards
        #[arg(long)]
        frame_start: Option<usize>,
        /// Only compare up to this frame
        #[arg(long)]
        frame_end: Option<usize>,
        /// Override checkpoint interval (regenerates checkpoints within the slice)
        #[arg(long)]
        checkpoint_interval: Option<usize>,
        /// Binary-search for the first divergent frame within the range
        #[arg(long, default_value_t = false)]
        bisect: bool,
        /// Which frontier to optimise for during verification/bisection
        #[arg(long, value_enum, default_value_t = sweep::FrontierKind::Gameplay)]
        frontier_kind: sweep::FrontierKind,
    },
    /// Side-by-side native vs oracle comparison for TAS debugging.
    TasDiff {
        #[arg(long)]
        rom: Option<PathBuf>,
        #[arg(long)]
        dir: Option<PathBuf>,
        /// Path to the DynamicScenario JSON
        #[arg(long)]
        scenario: PathBuf,
        #[arg(long)]
        frame_start: Option<usize>,
        #[arg(long)]
        frame_end: Option<usize>,
        /// Only show frames with divergences
        #[arg(long, default_value_t = false)]
        only_divergent: bool,
        /// Include advisory field diffs in output
        #[arg(long, default_value_t = false)]
        include_advisory: bool,
        /// Show PhysicsTrace sub-step values at divergent frames
        #[arg(long, default_value_t = false)]
        include_physics_trace: bool,
        /// Print object rosters on matching frames as well as divergent ones
        #[arg(long, default_value_t = false)]
        show_object_roster_on_match: bool,
        /// Generate 3-panel visual diff PNGs at divergent frames
        #[arg(long, default_value_t = false)]
        visual: bool,
        /// Which frontier to optimise for during diff classification
        #[arg(long, value_enum, default_value_t = sweep::FrontierKind::Gameplay)]
        frontier_kind: sweep::FrontierKind,
    },
    /// Import a BK2 TAS recording as a DynamicScenario JSON.
    ImportTas {
        /// Path to the .bk2 file
        #[arg(long)]
        bk2: PathBuf,
        /// Output directory (default: .cache/tas-imports/<slug>/)
        #[arg(long)]
        out: Option<PathBuf>,
        /// Frames between checkpoints (default: 300 ≈ 5s at 60fps)
        #[arg(long, default_value_t = 300)]
        checkpoint_interval: usize,
        /// Override the auto-detected slug
        #[arg(long)]
        slug: Option<String>,
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
    /// Interactive TUI for comparing native vs oracle state frame-by-frame.
    DebugTui {
        #[arg(long)]
        rom: Option<PathBuf>,
        #[arg(long)]
        dir: Option<PathBuf>,
        /// Path to a DynamicScenario JSON
        #[arg(long)]
        scenario: PathBuf,
        /// Only run up to this frame
        #[arg(long)]
        frame_end: Option<usize>,
    },
    /// SDL viewer for stepping a TAS scenario with native, oracle, and diff panels.
    ViewTas {
        #[arg(long)]
        rom: Option<PathBuf>,
        #[arg(long)]
        dir: Option<PathBuf>,
        /// Path to a DynamicScenario JSON
        #[arg(long)]
        scenario: PathBuf,
        /// First frame to include in the loaded window
        #[arg(long)]
        frame_start: Option<usize>,
        /// Stop before this frame
        #[arg(long)]
        frame_end: Option<usize>,
        /// Load the full scenario even when it exceeds the default viewer limit
        #[arg(long, default_value_t = false)]
        whole_run: bool,
        /// Diff classification for state comparisons
        #[arg(long, value_enum, default_value_t = sweep::FrontierKind::Gameplay)]
        frontier_kind: sweep::FrontierKind,
    },
    /// Probe oracle RAM at arbitrary addresses across a frame range.
    OracleProbe {
        #[arg(long)]
        rom: Option<PathBuf>,
        #[arg(long)]
        dir: Option<PathBuf>,
        /// Path to the DynamicScenario JSON
        #[arg(long)]
        scenario: PathBuf,
        /// First frame to include in output
        #[arg(long)]
        frame_start: Option<usize>,
        /// Last frame to include in output
        #[arg(long)]
        frame_end: Option<usize>,
        /// Memory watches: ADDRESS:TYPE pairs (e.g. 0xFF24FE:u32,0xFF2502:i32)
        #[arg(long)]
        watches: Option<String>,
        /// Frames at which to dump full 64KB WRAM (e.g. 954,955,956)
        #[arg(long)]
        wram_dump_frames: Option<String>,
        /// Frames at which to dump full 64KB VRAM (e.g. 1080)
        #[arg(long)]
        vram_dump_frames: Option<String>,
        /// Frames at which to dump oracle CRAM/palette memory (e.g. 1380)
        #[arg(long)]
        cram_dump_frames: Option<String>,
        /// Frames at which to dump oracle VSRAM/scroll memory (e.g. 1380)
        #[arg(long)]
        vsram_dump_frames: Option<String>,
        /// Frames at which to dump native synthetic VDP VRAM for comparison.
        #[arg(long)]
        native_vram_dump_frames: Option<String>,
        /// Frames at which to dump native synthetic VDP CRAM for comparison.
        #[arg(long)]
        native_cram_dump_frames: Option<String>,
        /// Frames at which to dump native synthetic VDP VSRAM for comparison.
        #[arg(long)]
        native_vsram_dump_frames: Option<String>,
        /// Oracle VDP pixel samples as X/Y pairs (e.g. 0/169,126/169)
        #[arg(long)]
        vdp_pixel_samples: Option<String>,
        /// Conditional filter: only show frames matching predicate (e.g. "0xFF2502>=0")
        /// Multiple --when flags are ANDed. Ops: =, !=, <, >, <=, >=
        #[arg(long)]
        when: Vec<String>,
        /// Capture M68K CPU registers (PC, D0-D7, A0-A6, SR) at each checkpoint
        #[arg(long, default_value_t = false)]
        capture_cpu_state: bool,
        /// Capture hidden oracle target-level settle rows before scenario frame 0
        #[arg(long, default_value_t = false)]
        capture_settle_trace: bool,
    },
    /// Compare sweep results between three physics orderings.
    SweepCompare {
        #[arg(long)]
        rom: Option<PathBuf>,
        #[arg(long)]
        dir: Option<PathBuf>,
        /// Level range: "0-104", "0,5,10", or "0-10"
        #[arg(long, default_value = "0-68")]
        levels: String,
        /// Collection tier
        #[arg(long, value_enum, default_value_t = sweep::SweepTier::State)]
        tier: sweep::SweepTier,
    },
    /// Rank the top-N pixel-diff checkpoints from a frames-tier sweep for
    /// visual triage. Copies native_*.png and diff_*.png into an output
    /// directory with a manifest, ordered by worst mismatch first.
    ///
    /// Filters out levels whose only state divergences are `oracle_limitation`
    /// — otherwise the top-N is polluted by oracle-pause noise. Requires a
    /// frames-tier sweep (state-tier runs have no pixel metrics).
    RankPixelDiffs {
        /// Path to a sweep report.json. Defaults to `.cache/sweep-runs/latest/report.json`.
        #[arg(long)]
        report: Option<PathBuf>,
        /// Number of top-ranked checkpoints to copy.
        #[arg(long, default_value_t = 10)]
        top: usize,
        /// Output directory.
        #[arg(long, default_value = ".cache/pixel-review/latest")]
        output: PathBuf,
    },
    /// Diff two WRAM dump files byte-by-byte, highlighting changed addresses.
    RamDiff {
        /// First WRAM dump file
        file_a: PathBuf,
        /// Second WRAM dump file
        file_b: PathBuf,
        /// Cross-reference known RAM addresses from variables.asm
        #[arg(long, default_value_t = false)]
        annotate: bool,
    },
    /// Test a hypothesis TOML file: run oracle, evaluate assertions, log result.
    TestHypothesis {
        #[arg(long)]
        rom: Option<PathBuf>,
        #[arg(long)]
        dir: Option<PathBuf>,
        /// Path to hypothesis TOML file
        hypothesis: PathBuf,
    },
    /// Cross-reference ASM labels against native code citations.
    AsmCoverage {
        /// Regenerate docs/source-port/asm-coverage.yaml (preserves hand annotations).
        #[arg(long, default_value_t = false)]
        update: bool,
        /// Filter to subsystems matching this substring (e.g. "enemy").
        #[arg(long)]
        subsystem: Option<String>,
        /// List only uncovered sub_ labels.
        #[arg(long, default_value_t = false)]
        uncovered: bool,
        /// Machine-readable JSON output.
        #[arg(long, default_value_t = false)]
        json: bool,
        /// Fail (exit non-zero) if any label in REQUIRED_LABELS lacks a
        /// native source citation. Use in CI/pre-commit to prevent
        /// regressions on attack/animation/slope routines.
        #[arg(long, default_value_t = false)]
        check_required: bool,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Commands::Case { command } => {
            let repo_root = current_dir()?;
            run_case_command(
                &repo_root,
                command,
                |rom| resolve_rom_path(&repo_root, rom),
                |dir| resolve_upstream_dir(&repo_root, dir),
            )?;
        }
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
                &PlayOptions {
                    rom,
                    fullscreen,
                    scale,
                    mute,
                },
            )?;
        }
        Commands::RecordScenario {
            slug,
            scenario,
            level,
            walk_frames,
            rom,
            dir,
        } => {
            let repo_root = current_dir()?;
            let rom_path = resolve_rom_path(&repo_root, rom)?;
            let upstream_dir = resolve_upstream_dir(&repo_root, dir);
            let manifest = prepare_generated_assets(&repo_root, &rom_path, &upstream_dir)?;
            let dynamic = build_record_scenario(scenario, level, walk_frames);
            if let Some(level) = dynamic.warp_to_level {
                ensure_level_layouts(&rom_path, &manifest, &[level])?;
            }
            let start = recording_start_for_dynamic_scenario(&dynamic);
            let trace = recording::run_recorded_trace(&manifest, start, &dynamic)?;
            let record_to = repo_root
                .join(".cache/recordings")
                .join(recording::validate_recording_slug(&slug)?);
            let recording =
                recording::build_recording(&manifest, &slug, start, dynamic.script.clone(), trace)?;
            // Replace the auto-generated walk_right/idle scenario inside the
            // recording with the explicit warp-driven scenario, so verify-
            // recording replays the same script the agent just executed.
            let mut recording = recording;
            recording.manifest.scenario = recording_scenario_manifest(dynamic, start);
            recording.manifest.record_command =
                record_scenario_command(&slug, scenario, level, walk_frames);
            recording::save_recording_with_artifacts(
                &record_to,
                &recording,
                &native_player::native_audio_artifacts_for_trace(&manifest, &recording.trace),
            )?;
            println!(
                "wrote recording bundle to {}",
                record_to
                    .strip_prefix(&repo_root)
                    .unwrap_or(&record_to)
                    .display()
            );
        }
        Commands::RecordScenarioFile {
            slug,
            scenario,
            rom,
            dir,
        } => {
            let repo_root = current_dir()?;
            let rom_path = resolve_rom_path(&repo_root, rom)?;
            let upstream_dir = resolve_upstream_dir(&repo_root, dir);
            let manifest = prepare_generated_assets(&repo_root, &rom_path, &upstream_dir)?;
            let dynamic = load_record_scenario_file(&scenario)?;
            if let Some(level) = dynamic.warp_to_level {
                ensure_level_layouts(&rom_path, &manifest, &[level])?;
            }
            let start = recording_start_for_dynamic_scenario(&dynamic);
            let trace = recording::run_recorded_trace(&manifest, start, &dynamic)?;
            let record_to = repo_root
                .join(".cache/recordings")
                .join(recording::validate_recording_slug(&slug)?);
            let recording =
                recording::build_recording(&manifest, &slug, start, dynamic.script.clone(), trace)?;
            let mut recording = recording;
            recording.manifest.scenario = recording_scenario_manifest(dynamic, start);
            recording.manifest.record_command = record_scenario_file_command(&slug, &scenario);
            recording::save_recording_with_artifacts(
                &record_to,
                &recording,
                &native_player::native_audio_artifacts_for_trace(&manifest, &recording.trace),
            )?;
            println!(
                "wrote recording bundle to {}",
                record_to
                    .strip_prefix(&repo_root)
                    .unwrap_or(&record_to)
                    .display()
            );
        }
        Commands::ProbeRecordScenario {
            scenario,
            level,
            walk_frames,
            input,
            trace_physics,
            debug_fields,
            target_enemy,
            target_projectile,
            player_motion,
            summary_only,
            frame_start,
            frame_end,
            rom,
            dir,
        } => {
            let repo_root = current_dir()?;
            let rom_path = resolve_rom_path(&repo_root, rom)?;
            let upstream_dir = resolve_upstream_dir(&repo_root, dir);
            let manifest = prepare_generated_assets(&repo_root, &rom_path, &upstream_dir)?;
            let dynamic = apply_dynamic_scenario_input_overlays(
                build_record_scenario(scenario, level, walk_frames),
                &input,
            )?;
            if let Some(level) = dynamic.warp_to_level {
                ensure_level_layouts(&rom_path, &manifest, &[level])?;
            }
            probe_dynamic_record_scenario(
                &manifest,
                &dynamic,
                trace_physics,
                debug_fields,
                target_enemy,
                target_projectile.map(Into::into),
                player_motion,
                summary_only,
                frame_start,
                frame_end,
            )?;
        }
        Commands::ProbeRecordScenarioFile {
            scenario,
            input,
            trace_physics,
            debug_fields,
            target_enemy,
            target_projectile,
            player_motion,
            summary_only,
            frame_start,
            frame_end,
            rom,
            dir,
        } => {
            let repo_root = current_dir()?;
            let rom_path = resolve_rom_path(&repo_root, rom)?;
            let upstream_dir = resolve_upstream_dir(&repo_root, dir);
            let manifest = prepare_generated_assets(&repo_root, &rom_path, &upstream_dir)?;
            let dynamic = apply_dynamic_scenario_input_overlays(
                load_record_scenario_file(&scenario)?,
                &input,
            )?;
            if let Some(level) = dynamic.warp_to_level {
                ensure_level_layouts(&rom_path, &manifest, &[level])?;
            }
            probe_dynamic_record_scenario(
                &manifest,
                &dynamic,
                trace_physics,
                debug_fields,
                target_enemy,
                target_projectile.map(Into::into),
                player_motion,
                summary_only,
                frame_start,
                frame_end,
            )?;
        }
        Commands::PlayNative {
            rom,
            dir,
            fullscreen,
            scale,
            debug_overlay,
            warp_level,
            settled,
            record,
            record_frames,
            record_notes,
            scenario,
        } => {
            let repo_root = current_dir()?;
            let rom_path = resolve_rom_path(&repo_root, rom)?;
            let upstream_dir = resolve_upstream_dir(&repo_root, dir);
            let manifest = prepare_generated_assets(&repo_root, &rom_path, &upstream_dir)?;
            // Native level entry depends on generated `level-layout` dumps for
            // floor snap/collision. Preflight the early boot path here so
            // manual play does not open into a misleading loading-screen stall.
            if let Some(level_id) = warp_level {
                ensure_level_layouts(&rom_path, &manifest, &[level_id])?;
            } else {
                ensure_gameplay_level_layouts(
                    &rom_path,
                    &manifest,
                    GameplayScenario::ProgressTwoTransitions,
                )?;
            }
            let record_to = record
                .as_deref()
                .map(validate_recording_slug)
                .transpose()?
                .map(|slug| repo_root.join(".cache/recordings").join(slug));
            let record_notes = match (&record_to, record_notes.as_deref()) {
                (None, Some(_)) => bail!("--record-notes requires --record <slug>"),
                (_, Some(path)) => Some(
                    fs::read_to_string(path)
                        .with_context(|| format!("failed to read {}", path.display()))?,
                ),
                (_, None) => None,
            };
            match (&record_to, record_frames) {
                (None, Some(_)) => bail!("--record-frames requires --record <slug>"),
                (Some(_), Some(0)) => bail!("--record-frames must be greater than zero"),
                _ => {}
            }
            let scripted_scenario = scenario
                .as_deref()
                .map(load_record_scenario_file)
                .transpose()?;
            let record_frames = match (&record_to, record_frames, &scripted_scenario) {
                (Some(_), None, Some(dynamic)) => Some(dynamic.frame_count),
                _ => record_frames,
            };
            run_native_player(
                &manifest,
                &NativePlayOptions {
                    fullscreen,
                    scale,
                    debug_overlay,
                    warp_level,
                    settled,
                    record_to,
                    record_frames,
                    record_notes,
                    scripted_input: scripted_scenario.map(|dynamic| dynamic.script),
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
                    GameplayScenario::DeathRespawnStartResumeLevel1,
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
            let upstream_dir = resolve_upstream_dir(&repo_root, None);
            bootstrap_upstream(&upstream_dir)?;
            let verified = verify_rom(&rom_path)?;
            if !matches!(verified.identity, RomIdentity::KnownKidChameleon) {
                bail!(
                    "verify-audio-output currently supports only the known Kid Chameleon ROM dump"
                );
            }
            let manifest = prepare_generated_assets(&repo_root, &rom_path, &upstream_dir)?;
            let rom_bytes = fs::read(&rom_path)
                .with_context(|| format!("failed to read ROM from {}", rom_path.display()))?;
            verify_audio_output(&repo_root, &rom_path, &rom_bytes, &manifest, scenario)?;
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
            let upstream_dir = resolve_upstream_dir(&repo_root, None);
            bootstrap_upstream(&upstream_dir)?;
            let verified = verify_rom(&rom_path)?;
            if !matches!(verified.identity, RomIdentity::KnownKidChameleon) {
                bail!("verify-frames currently supports only the known Kid Chameleon ROM dump");
            }
            let manifest = prepare_generated_assets(&repo_root, &rom_path, &upstream_dir)?;
            let rom_bytes = fs::read(&rom_path)
                .with_context(|| format!("failed to read ROM from {}", rom_path.display()))?;
            verify_frames(&repo_root, &rom_path, &rom_bytes, &manifest, scenario)?;
            println!("frame verification passed");
        }
        Commands::VerifyOracle { rom, scenario } => {
            let repo_root = current_dir()?;
            let rom_path = resolve_rom_path(&repo_root, rom)?;
            let upstream_dir = resolve_upstream_dir(&repo_root, None);
            bootstrap_upstream(&upstream_dir)?;
            let verified = verify_rom(&rom_path)?;
            if !matches!(verified.identity, RomIdentity::KnownKidChameleon) {
                bail!("verify-oracle currently supports only the known Kid Chameleon ROM dump");
            }
            let manifest = prepare_generated_assets(&repo_root, &rom_path, &upstream_dir)?;
            let rom_bytes = fs::read(&rom_path)
                .with_context(|| format!("failed to read ROM from {}", rom_path.display()))?;
            verify_oracle(&repo_root, &rom_path, &rom_bytes, &manifest, scenario)?;
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
        Commands::ProbeNative {
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
            probe_native_scenario(&manifest, scenario, frame_start, frame_end)?;
        }
        Commands::ProbeNativeGameplay {
            rom,
            dir,
            scenario,
            trace_physics,
            frame_start,
            frame_end,
        } => {
            let repo_root = current_dir()?;
            let rom_path = resolve_rom_path(&repo_root, rom)?;
            let upstream_dir = resolve_upstream_dir(&repo_root, dir);
            let manifest = prepare_generated_assets(&repo_root, &rom_path, &upstream_dir)?;
            ensure_gameplay_level_layouts(&rom_path, &manifest, scenario)?;
            probe_native_gameplay_scenario(
                &manifest,
                scenario,
                trace_physics,
                frame_start,
                frame_end,
            )?;
        }
        Commands::InspectLevel { rom, dir, level } => {
            let repo_root = current_dir()?;
            let rom_path = resolve_rom_path(&repo_root, rom)?;
            let upstream_dir = resolve_upstream_dir(&repo_root, dir);
            let manifest = prepare_generated_assets(&repo_root, &rom_path, &upstream_dir)?;
            inspect_level(&manifest, &level)?;
        }
        Commands::InspectRespawnSettle {
            rom,
            dir,
            level,
            steps,
        } => {
            let repo_root = current_dir()?;
            let rom_path = resolve_rom_path(&repo_root, rom)?;
            let upstream_dir = resolve_upstream_dir(&repo_root, dir);
            let manifest = prepare_generated_assets(&repo_root, &rom_path, &upstream_dir)?;
            inspect_respawn_settle(&manifest, &level, steps)?;
        }
        Commands::InspectSpawnProbe { rom, dir, level } => {
            let repo_root = current_dir()?;
            let rom_path = resolve_rom_path(&repo_root, rom)?;
            let upstream_dir = resolve_upstream_dir(&repo_root, dir);
            let manifest = prepare_generated_assets(&repo_root, &rom_path, &upstream_dir)?;
            inspect_spawn_probe(&manifest, &level)?;
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
        Commands::CompareNativeGameplayWindow {
            rom,
            dir,
            scenario,
            frame_start,
            frame_end,
            checkpoint_interval,
        } => {
            let repo_root = current_dir()?;
            let rom_path = resolve_rom_path(&repo_root, rom)?;
            let upstream_dir = resolve_upstream_dir(&repo_root, dir);
            let manifest = prepare_generated_assets(&repo_root, &rom_path, &upstream_dir)?;
            ensure_gameplay_level_layouts(&rom_path, &manifest, scenario)?;
            compare_native_gameplay_window(
                &repo_root,
                &rom_path,
                &manifest,
                scenario,
                frame_start,
                frame_end,
                checkpoint_interval,
            )?;
        }
        Commands::VerifyRecording {
            rom,
            dir,
            slug,
            bundle,
            native_only,
        } => {
            let repo_root = current_dir()?;
            let rom_path = resolve_rom_path(&repo_root, rom)?;
            let upstream_dir = resolve_upstream_dir(&repo_root, dir);
            let manifest = prepare_generated_assets(&repo_root, &rom_path, &upstream_dir)?;
            let record_to =
                resolve_recording_bundle_input(&repo_root, slug.as_deref(), bundle.as_deref())?;
            verify_recording(&repo_root, &rom_path, &manifest, &record_to, native_only)?;
        }
        Commands::DiagnoseNativeGameplayDrift {
            rom,
            dir,
            scenario,
            from_frame,
            limit,
            context_window,
        } => {
            let repo_root = current_dir()?;
            let rom_path = resolve_rom_path(&repo_root, rom)?;
            let upstream_dir = resolve_upstream_dir(&repo_root, dir);
            let manifest = prepare_generated_assets(&repo_root, &rom_path, &upstream_dir)?;
            ensure_gameplay_level_layouts(&rom_path, &manifest, scenario)?;
            diagnose_native_gameplay_drift(
                &repo_root,
                &rom_path,
                &manifest,
                scenario,
                from_frame,
                limit,
                context_window,
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
            routines_per_task,
        } => {
            let repo_root = current_dir()?;
            let (rom_path, upstream_dir) = if matches!(
                goal,
                AutonomyGoal::SourcePortFrontier
                    | AutonomyGoal::CaseQueue
                    | AutonomyGoal::GameplayBacklog
                    | AutonomyGoal::ProactiveVerification
            ) {
                (None, None)
            } else {
                let rom_path = resolve_rom_path(&repo_root, rom)?;
                let upstream_dir = resolve_upstream_dir(&repo_root, dir);
                (Some(rom_path), Some(upstream_dir))
            };
            run_autonomous_loop(
                &repo_root,
                rom_path.as_deref(),
                upstream_dir.as_deref(),
                goal,
                max_cycles,
                !no_resume,
                emit_json,
                routines_per_task,
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
                    .join(sha1_hex(&rom_bytes))
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
            checksum,
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
                    checksum,
                };
                sweep::run_sweep_with_options(
                    &repo_root, &rom_path, &manifest, &level_ids, tier, &options,
                )?;
            }
        }
        Commands::ReleaseGate {
            rom,
            dir,
            target,
            max_failures,
        } => {
            let repo_root = current_dir()?;
            let rom_path = resolve_rom_path(&repo_root, rom)?;
            let upstream_dir = resolve_upstream_dir(&repo_root, dir);
            let manifest = prepare_generated_assets(&repo_root, &rom_path, &upstream_dir)?;
            run_release_gate(&repo_root, &rom_path, &manifest, target, max_failures)?;
        }
        Commands::ProjectStatus {
            format,
            fail_on_human_blockers,
        } => {
            let repo_root = current_dir()?;
            run_project_status(&repo_root, format, fail_on_human_blockers)?;
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
            sweep::capture_frames(&manifest, level_id, &frame_indices, &output, filter)?;
        }
        Commands::CaptureScenarioFrames {
            rom,
            dir,
            scenario,
            input,
            frames,
            output,
            layer,
        } => {
            let repo_root = current_dir()?;
            let rom_path = resolve_rom_path(&repo_root, rom)?;
            let upstream_dir = resolve_upstream_dir(&repo_root, dir);
            let manifest = prepare_generated_assets(&repo_root, &rom_path, &upstream_dir)?;
            let frame_indices: Vec<usize> = frames
                .split(',')
                .map(|s| s.trim().parse::<usize>().context("invalid frame index"))
                .collect::<Result<_>>()?;
            let filter = layer
                .as_deref()
                .map(native_player::LayerFilter::solo)
                .unwrap_or_default();
            let dynamic = apply_dynamic_scenario_input_overlays(
                load_record_scenario_file(&scenario)?,
                &input,
            )?;
            if let Some(level) = dynamic.warp_to_level {
                ensure_level_layouts(&rom_path, &manifest, &[level])?;
            }
            sweep::capture_scenario_frames(&manifest, &dynamic, &frame_indices, &output, filter)?;
        }
        Commands::DumpEnemySheet {
            rom,
            dir,
            level,
            enemy,
            output,
            frame,
        } => {
            let repo_root = current_dir()?;
            let rom_path = resolve_rom_path(&repo_root, rom)?;
            let upstream_dir = resolve_upstream_dir(&repo_root, dir);
            let manifest = prepare_generated_assets(&repo_root, &rom_path, &upstream_dir)?;
            let level_id =
                u8::from_str_radix(&level, 16).context("level must be a hex byte (e.g. 01)")?;
            let enemy_type =
                u8::from_str_radix(&enemy, 16).context("enemy must be a hex byte (e.g. 0A)")?;
            let assembled_frame = frame
                .as_deref()
                .map(|spec| -> Result<(u16, u16, u16)> {
                    let parts: Vec<&str> = spec.split(',').map(str::trim).collect();
                    if parts.len() != 3 {
                        bail!("--frame must be BASE,WIDTH,HEIGHT in hex (e.g. 20,1e,14)");
                    }
                    Ok((
                        u16::from_str_radix(parts[0], 16).context("frame base")?,
                        u16::from_str_radix(parts[1], 16).context("frame width")?,
                        u16::from_str_radix(parts[2], 16).context("frame height")?,
                    ))
                })
                .transpose()?;
            sweep::dump_enemy_sheet(&manifest, level_id, enemy_type, &output, assembled_frame)?;
        }
        Commands::VerifyTas {
            rom,
            dir,
            scenario,
            frame_start,
            frame_end,
            checkpoint_interval,
            bisect,
            frontier_kind,
        } => {
            let repo_root = current_dir()?;
            let rom_path = resolve_rom_path(&repo_root, rom)?;
            let upstream_dir = resolve_upstream_dir(&repo_root, dir);
            let manifest = prepare_generated_assets(&repo_root, &rom_path, &upstream_dir)?;
            if bisect {
                tas::bisect_tas(
                    &repo_root,
                    &rom_path,
                    &manifest,
                    &scenario,
                    frame_start,
                    frame_end,
                    frontier_kind,
                )?;
            } else {
                tas::verify_tas(
                    &repo_root,
                    &rom_path,
                    &manifest,
                    &scenario,
                    frame_start,
                    frame_end,
                    checkpoint_interval,
                    frontier_kind,
                )?;
            }
        }
        Commands::TasDiff {
            rom,
            dir,
            scenario,
            frame_start,
            frame_end,
            only_divergent,
            include_advisory,
            include_physics_trace,
            show_object_roster_on_match,
            visual,
            frontier_kind,
        } => {
            let repo_root = current_dir()?;
            let rom_path = resolve_rom_path(&repo_root, rom)?;
            let upstream_dir = resolve_upstream_dir(&repo_root, dir);
            let manifest = prepare_generated_assets(&repo_root, &rom_path, &upstream_dir)?;
            tas_diff::run_tas_diff(
                &repo_root,
                &rom_path,
                &manifest,
                &scenario,
                frame_start,
                frame_end,
                only_divergent,
                include_advisory,
                include_physics_trace,
                show_object_roster_on_match,
                visual,
                frontier_kind,
            )?;
        }
        Commands::ImportTas {
            bk2,
            out,
            checkpoint_interval,
            slug,
        } => {
            let repo_root = current_dir()?;
            let mut import = tas::parse_bk2(&bk2)?;
            if let Some(slug_override) = slug {
                import.slug = slug_override;
            }
            let scenario = tas::bk2_to_scenario(&import, checkpoint_interval);
            let out_dir =
                out.unwrap_or_else(|| repo_root.join(".cache/tas-imports").join(&scenario.slug));
            fs::create_dir_all(&out_dir)
                .with_context(|| format!("failed to create {}", out_dir.display()))?;
            let scenario_file = out_dir.join("scenario.json");
            let json = serde_json::to_vec_pretty(&scenario)?;
            fs::write(&scenario_file, &json)
                .with_context(|| format!("failed to write {}", scenario_file.display()))?;
            let duration_secs = import.total_frames as f64 / 60.0;
            println!("imported TAS: {}", import.slug);
            println!("  frames: {}", import.total_frames);
            println!(
                "  duration: {duration_secs:.1}s ({:.1}m)",
                duration_secs / 60.0
            );
            println!("  checkpoints: {}", scenario.checkpoints.len());
            println!("  scenario: {}", scenario_file.display());
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
        Commands::DebugTui {
            rom,
            dir,
            scenario,
            frame_end,
        } => {
            let repo_root = current_dir()?;
            let rom_path = resolve_rom_path(&repo_root, rom)?;
            let upstream_dir = resolve_upstream_dir(&repo_root, dir);
            let manifest = prepare_generated_assets(&repo_root, &rom_path, &upstream_dir)?;
            let dyn_scenario = view_tas::load_dynamic_scenario(&scenario)?;
            let end = frame_end
                .unwrap_or(dyn_scenario.frame_count)
                .min(dyn_scenario.frame_count);
            let comparison = view_tas::prepare_tas_comparison_run(
                &repo_root,
                &rom_path,
                &manifest,
                &dyn_scenario,
                0,
                end,
                "debug-tui",
                sweep::FrontierKind::Gameplay,
            )?;

            println!(
                "Launching TUI ({} native frames, {} oracle checkpoints, {} CRCs)...",
                comparison.native_trace.len(),
                comparison.oracle.states.len(),
                comparison.oracle.frame_crcs.len()
            );

            let debugger =
                tui_debugger::TuiDebugger::new(comparison.native_trace, &comparison.oracle);
            debugger.run()?;
        }
        Commands::ViewTas {
            rom,
            dir,
            scenario,
            frame_start,
            frame_end,
            whole_run,
            frontier_kind,
        } => {
            let repo_root = current_dir()?;
            let rom_path = resolve_rom_path(&repo_root, rom)?;
            let upstream_dir = resolve_upstream_dir(&repo_root, dir);
            let manifest = prepare_generated_assets(&repo_root, &rom_path, &upstream_dir)?;
            view_tas::run_view_tas(
                &repo_root,
                &rom_path,
                &manifest,
                &scenario,
                frame_start,
                frame_end,
                whole_run,
                frontier_kind,
            )?;
        }
        Commands::OracleProbe {
            rom,
            dir,
            scenario,
            frame_start,
            frame_end,
            watches,
            wram_dump_frames,
            vram_dump_frames,
            cram_dump_frames,
            vsram_dump_frames,
            native_vram_dump_frames,
            native_cram_dump_frames,
            native_vsram_dump_frames,
            vdp_pixel_samples,
            when,
            capture_cpu_state,
            capture_settle_trace,
        } => {
            let repo_root = current_dir()?;
            let rom_path = resolve_rom_path(&repo_root, rom)?;
            let upstream_dir = resolve_upstream_dir(&repo_root, dir);
            let manifest = prepare_generated_assets(&repo_root, &rom_path, &upstream_dir)?;

            // Load and optionally slice the scenario
            let raw_json = fs::read_to_string(&scenario)
                .with_context(|| format!("failed to read {}", scenario.display()))?;
            let mut dyn_scenario: scenarios::DynamicScenario = serde_json::from_str(&raw_json)?;

            // Override frame_count if frame_end is specified
            if let Some(end) = frame_end {
                dyn_scenario.frame_count = end + 1;
            }

            // Generate checkpoints for every frame in the range
            let start = frame_start.unwrap_or(0);
            let end = frame_end.unwrap_or(dyn_scenario.frame_count.saturating_sub(1));
            dyn_scenario.checkpoints = (start..=end)
                .map(|f| scenarios::DynamicCheckpoint {
                    label: format!("f{f}"),
                    frame_index: f,
                })
                .collect();

            // Write the sliced scenario
            let run_dir = repo_root.join(".cache/oracle-probe");
            fs::create_dir_all(&run_dir)?;
            let scenario_file = run_dir.join("scenario.json");
            fs::write(&scenario_file, serde_json::to_vec_pretty(&dyn_scenario)?)?;

            // Run oracle with memory watches
            let env = crate::oracle::ensure_oracles(&repo_root)?;
            let oracle_dir = run_dir.join("oracle");
            fs::create_dir_all(&oracle_dir)?;

            let options = crate::oracle::OracleRunOptions {
                skip_frames_before: frame_start,
                memory_watches: watches.clone(),
                wram_dump_frames: wram_dump_frames.clone(),
                vram_dump_frames: vram_dump_frames.clone(),
                cram_dump_frames: cram_dump_frames.clone(),
                vsram_dump_frames: vsram_dump_frames.clone(),
                vdp_pixel_samples: vdp_pixel_samples.clone(),
                capture_cpu_state,
                capture_settle_trace,
            };
            let oracle_manifest = crate::oracle::run_jgenesis_capture_dynamic_with_options(
                &env,
                &rom_path,
                &scenario_file,
                &oracle_dir,
                &options,
            )?;
            let oracle_manifest_path = oracle_dir.join("manifest.json");
            fs::write(
                &oracle_manifest_path,
                serde_json::to_vec_pretty(&oracle_manifest)?,
            )
            .with_context(|| {
                format!(
                    "failed to write oracle probe manifest {}",
                    oracle_manifest_path.display()
                )
            })?;

            // Parse --when predicates
            let predicates = parse_when_predicates(&when)?;

            let watch_rows_in_range: Vec<_> = oracle_manifest
                .memory_watches
                .iter()
                .filter(|watch_map| oracle_probe_watch_row_in_range(watch_map, start, end))
                .collect();

            // Output memory watches as CSV
            if !watch_rows_in_range.is_empty() {
                // Collect all column names (excluding "frame")
                let mut columns: Vec<String> = Vec::new();
                for watch_map in &watch_rows_in_range {
                    for key in watch_map.keys() {
                        if key != "frame" && !columns.contains(key) {
                            columns.push(key.clone());
                        }
                    }
                }
                columns.sort();

                // Print header
                print!("frame");
                for col in &columns {
                    print!(",{col}");
                }
                println!();

                // Print rows (filtered by --when predicates)
                let mut shown = 0usize;
                for watch_map in &watch_rows_in_range {
                    if !predicates.is_empty()
                        && !predicates.iter().all(|pred| pred.evaluate(watch_map))
                    {
                        continue;
                    }
                    let frame = watch_map.get("frame").copied().unwrap_or(-1);
                    print!("{frame}");
                    for col in &columns {
                        let val = watch_map.get(col).copied().unwrap_or(0);
                        print!(",0x{val:X}");
                    }
                    println!();
                    shown += 1;
                }
                if !predicates.is_empty() {
                    eprintln!(
                        "  --when filter: {shown}/{} rows shown",
                        watch_rows_in_range.len()
                    );
                }
            }

            if !oracle_manifest.settle_trace.is_empty() {
                println!(
                    "settle_phase,settle_step,game_mode,current_level_id,paused,camera_x_pos,camera_y_pos,kid_x_pos,kid_y_pos,kid_x_vel,kid_y_vel,number_objects"
                );
                for row in &oracle_manifest.settle_trace {
                    println!(
                        "{},{},0x{:04X},0x{:04X},{},0x{:08X},0x{:08X},0x{:08X},0x{:08X},0x{:08X},0x{:08X},{}",
                        row.phase,
                        row.step,
                        row.game_mode,
                        row.current_level_id,
                        row.paused,
                        row.camera_x_pos,
                        row.camera_y_pos,
                        row.kid_x_pos,
                        row.kid_y_pos,
                        row.kid_x_vel as u32,
                        row.kid_y_vel as u32,
                        row.number_objects,
                    );
                }
            }

            // Report WRAM dumps
            if wram_dump_frames.is_some() {
                eprintln!("WRAM dumps written to {}", oracle_dir.display());
            }
            if vram_dump_frames.is_some() {
                eprintln!("VRAM dumps written to {}", oracle_dir.display());
            }
            if cram_dump_frames.is_some() {
                eprintln!("CRAM dumps written to {}", oracle_dir.display());
            }
            if vsram_dump_frames.is_some() {
                eprintln!("VSRAM dumps written to {}", oracle_dir.display());
            }
            if let Some(frames) = native_vram_dump_frames.as_deref() {
                let frames = parse_frame_list_cli(frames)?;
                write_native_vdp_dumps_for_dynamic_scenario(
                    &run_dir.join("native"),
                    &manifest,
                    &dyn_scenario,
                    &frames,
                    NativeVdpDumpArea::Vram,
                )?;
                eprintln!(
                    "Native VRAM dumps written to {}",
                    run_dir.join("native").display()
                );
            }
            if let Some(frames) = native_cram_dump_frames.as_deref() {
                let frames = parse_frame_list_cli(frames)?;
                write_native_vdp_dumps_for_dynamic_scenario(
                    &run_dir.join("native"),
                    &manifest,
                    &dyn_scenario,
                    &frames,
                    NativeVdpDumpArea::Cram,
                )?;
                eprintln!(
                    "Native CRAM dumps written to {}",
                    run_dir.join("native").display()
                );
            }
            if let Some(frames) = native_vsram_dump_frames.as_deref() {
                let frames = parse_frame_list_cli(frames)?;
                write_native_vdp_dumps_for_dynamic_scenario(
                    &run_dir.join("native"),
                    &manifest,
                    &dyn_scenario,
                    &frames,
                    NativeVdpDumpArea::Vsram,
                )?;
                eprintln!(
                    "Native VSRAM dumps written to {}",
                    run_dir.join("native").display()
                );
            }

            for row in oracle_manifest
                .vdp_pixel_samples
                .iter()
                .filter(|row| (start..=end).contains(&row.frame))
            {
                println!("vdp_sample_frame={},{}", row.frame, row.samples);
            }

            // Also output checkpoint states for the range
            let state_count = oracle_manifest.states.len();
            eprintln!(
                "Oracle probe complete: {} state checkpoints, {} watch rows, {} VDP sample rows",
                state_count,
                watch_rows_in_range.len(),
                oracle_manifest
                    .vdp_pixel_samples
                    .iter()
                    .filter(|row| (start..=end).contains(&row.frame))
                    .count()
            );

            // Display CPU registers if captured
            if capture_cpu_state {
                for state in &oracle_manifest.states {
                    if let Some(ref regs) = state.cpu_registers {
                        println!(
                            "frame {:>5}  PC={:#010x}  SR={:#06x}",
                            state.frame_index, regs.pc, regs.status_register
                        );
                        println!(
                            "  D0-D3: {:#010x} {:#010x} {:#010x} {:#010x}",
                            regs.data_registers[0],
                            regs.data_registers[1],
                            regs.data_registers[2],
                            regs.data_registers[3]
                        );
                        println!(
                            "  D4-D7: {:#010x} {:#010x} {:#010x} {:#010x}",
                            regs.data_registers[4],
                            regs.data_registers[5],
                            regs.data_registers[6],
                            regs.data_registers[7]
                        );
                        println!(
                            "  A0-A3: {:#010x} {:#010x} {:#010x} {:#010x}",
                            regs.address_registers[0],
                            regs.address_registers[1],
                            regs.address_registers[2],
                            regs.address_registers[3]
                        );
                        println!(
                            "  A4-A6: {:#010x} {:#010x} {:#010x}",
                            regs.address_registers[4],
                            regs.address_registers[5],
                            regs.address_registers[6]
                        );
                    }
                }
            }
        }
        Commands::SweepCompare {
            rom,
            dir,
            levels,
            tier,
        } => {
            let repo_root = current_dir()?;
            let rom_path = resolve_rom_path(&repo_root, rom)?;
            let upstream_dir = resolve_upstream_dir(&repo_root, dir);
            let manifest = prepare_generated_assets(&repo_root, &rom_path, &upstream_dir)?;
            let level_ids = sweep::parse_level_range(&levels)?;

            sweep::run_sweep_compare(&repo_root, &rom_path, &manifest, &level_ids, tier)?;
        }
        Commands::RankPixelDiffs {
            report,
            top,
            output,
        } => {
            let repo_root = current_dir()?;
            let report_path =
                report.unwrap_or_else(|| repo_root.join(".cache/sweep-runs/latest/report.json"));
            let manifest = sweep::run_rank_pixel_diffs(&report_path, top, &output)?;
            println!(
                "ranked {} entries (skipped {} oracle-limitation-only, {} without pixel data)",
                manifest.entries.len(),
                manifest.skipped_oracle_limitation_only,
                manifest.skipped_no_pixel_data
            );
            for entry in &manifest.entries {
                println!(
                    "  #{:02} level {:02x} {} — {} mismatches (mae {:.1})",
                    entry.rank,
                    entry.level_id,
                    entry.slug,
                    entry.pixel_mismatches.unwrap_or(0),
                    entry.mae.unwrap_or(0.0),
                );
            }
            println!("manifest written to {}/manifest.json", output.display());
        }
        Commands::RamDiff {
            file_a,
            file_b,
            annotate,
        } => {
            ram_diff::run_ram_diff(&file_a, &file_b, annotate)?;
        }
        Commands::TestHypothesis {
            rom,
            dir,
            hypothesis,
        } => {
            let repo_root = current_dir()?;
            let rom_path = resolve_rom_path(&repo_root, rom)?;
            let upstream_dir = resolve_upstream_dir(&repo_root, dir);
            let manifest = prepare_generated_assets(&repo_root, &rom_path, &upstream_dir)?;
            hypothesis::test_hypothesis(&repo_root, &rom_path, &manifest, &hypothesis)?;
        }
        Commands::AsmCoverage {
            update,
            subsystem,
            uncovered,
            json,
            check_required,
        } => {
            let repo_root = current_dir()?;
            asm_coverage::run_asm_coverage(
                &repo_root,
                update,
                subsystem.as_deref(),
                uncovered,
                json,
                check_required,
            )?;
        }
    }
    Ok(())
}

fn parse_frame_list_cli(spec: &str) -> Result<Vec<usize>> {
    if spec.trim().is_empty() {
        return Ok(Vec::new());
    }
    spec.split(',')
        .map(|part| {
            let trimmed = part.trim();
            trimmed
                .parse::<usize>()
                .with_context(|| format!("invalid frame number: {trimmed}"))
        })
        .collect()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum NativeVdpDumpArea {
    Vram,
    Cram,
    Vsram,
}

fn write_native_vdp_dumps_for_dynamic_scenario(
    out_dir: &Path,
    manifest: &AssetManifest,
    scenario: &DynamicScenario,
    frames: &[usize],
    area: NativeVdpDumpArea,
) -> Result<()> {
    if frames.is_empty() {
        return Ok(());
    }
    fs::create_dir_all(out_dir)
        .with_context(|| format!("failed to create {}", out_dir.display()))?;

    let mut runtime = NativeRuntime::with_generated_level_layout(manifest.clone());
    if let Some(level_id) = scenario.warp_to_level {
        let level_id = LevelId(level_id);
        runtime.boot_to_level(level_id);
        if scenario.settle_oracle_warp {
            runtime.settle_recording_oracle_warp_live_window(level_id);
        }
    }
    if let Some(position) = scenario.start_position {
        runtime.set_player_probe_position(position.x, position.y);
    }
    if let Some(position) = scenario.camera_position {
        runtime.set_camera_probe_position(position.x, position.y);
    }
    if let Some(helmet_id) = scenario.playtest_form {
        let form = PlayerForm::from_helmet_id(helmet_id)
            .ok_or_else(|| anyhow!("invalid playtest_form helmet id {helmet_id}"))?;
        runtime.set_playtest_form(form);
    }
    if let Some(count) = scenario.initial_diamond_count {
        runtime.set_probe_diamond_count(count);
    }

    let last_frame = frames
        .iter()
        .copied()
        .max()
        .expect("empty frames handled above");
    for frame_index in 0..=last_frame {
        let input = scenario
            .script
            .get(frame_index)
            .copied()
            .map(FrameInput::from_mask)
            .unwrap_or_default();
        runtime.step_frame(input);
        if frames.contains(&frame_index) {
            let vdp = runtime
                .vdp_state()
                .ok_or_else(|| anyhow!("native frame {frame_index} has no synthetic VDP state"))?;
            let (prefix, bytes) = match area {
                NativeVdpDumpArea::Vram => ("vram", vdp.vram.as_slice()),
                NativeVdpDumpArea::Cram => ("cram", vdp.cram.as_slice()),
                NativeVdpDumpArea::Vsram => ("vsram", vdp.vsram.as_slice()),
            };
            let dump_path = out_dir.join(format!("{prefix}_frame_{frame_index}.bin"));
            fs::write(&dump_path, bytes)
                .with_context(|| format!("failed to write {}", dump_path.display()))?;
        }
    }

    Ok(())
}

// ---------------------------------------------------------------------------
// --when predicate parsing for oracle-probe conditional filtering
// ---------------------------------------------------------------------------

#[derive(Debug)]
enum WhenOp {
    Eq,
    Ne,
    Lt,
    Gt,
    Le,
    Ge,
}

#[derive(Debug)]
struct WhenPredicate {
    /// Watch column name (e.g. "0xFF24FE_u32")
    address_key: String,
    op: WhenOp,
    value: i64,
}

impl WhenPredicate {
    fn evaluate(&self, watch_map: &std::collections::HashMap<String, i64>) -> bool {
        let actual = watch_map.get(&self.address_key).copied().unwrap_or(0);
        match self.op {
            WhenOp::Eq => actual == self.value,
            WhenOp::Ne => actual != self.value,
            WhenOp::Lt => actual < self.value,
            WhenOp::Gt => actual > self.value,
            WhenOp::Le => actual <= self.value,
            WhenOp::Ge => actual >= self.value,
        }
    }
}

fn parse_when_predicates(specs: &[String]) -> Result<Vec<WhenPredicate>> {
    // Order matters: check two-char ops before single-char
    let op_tokens: &[(&str, fn() -> WhenOp)] = &[
        ("!=", || WhenOp::Ne),
        (">=", || WhenOp::Ge),
        ("<=", || WhenOp::Le),
        ("=", || WhenOp::Eq),
        (">", || WhenOp::Gt),
        ("<", || WhenOp::Lt),
    ];
    let mut predicates = Vec::new();
    for spec in specs {
        let (addr, mk_op, val_str) = op_tokens
            .iter()
            .find_map(|(tok, mk)| {
                let (left, right) = spec.split_once(tok)?;
                Some((left.trim(), *mk, right.trim()))
            })
            .ok_or_else(|| {
                anyhow!("invalid --when predicate: '{spec}' (expected ADDRESS OP VALUE)")
            })?;
        let address_key = addr.replace(':', "_");
        let value = if let Some(hex) = val_str
            .strip_prefix("0x")
            .or_else(|| val_str.strip_prefix("0X"))
        {
            i64::from_str_radix(hex, 16).with_context(|| format!("invalid hex value: {val_str}"))?
        } else {
            val_str
                .parse::<i64>()
                .with_context(|| format!("invalid value: {val_str}"))?
        };
        predicates.push(WhenPredicate {
            address_key,
            op: mk_op(),
            value,
        });
    }
    Ok(predicates)
}

fn oracle_probe_watch_row_in_range(
    watch_map: &std::collections::HashMap<String, i64>,
    start: usize,
    end: usize,
) -> bool {
    let Some(frame) = watch_map.get("frame").copied() else {
        return false;
    };
    let Ok(frame) = usize::try_from(frame) else {
        return false;
    };
    (start..=end).contains(&frame)
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

fn apply_dynamic_scenario_input_overlays(
    mut scenario: DynamicScenario,
    input_specs: &[String],
) -> Result<DynamicScenario> {
    for spec in input_specs {
        let range = parse_input_spec(spec)?;
        let required_len = range.end.saturating_add(2);
        if scenario.script.len() < required_len {
            scenario.script.resize(required_len, 0);
        }
        scenario.frame_count = scenario.frame_count.max(range.end.saturating_add(1));
        for frame_buttons in scenario
            .script
            .iter_mut()
            .take(range.end.saturating_add(1))
            .skip(range.start)
        {
            if range.buttons == 0 {
                *frame_buttons = 0;
            } else {
                *frame_buttons |= range.buttons;
            }
        }
    }
    Ok(scenario)
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

fn build_record_scenario(
    kind: RecordScenarioKind,
    level: u8,
    walk_frames: Option<usize>,
) -> DynamicScenario {
    match kind {
        RecordScenarioKind::Idle => idle_level_scenario(level, walk_frames),
        RecordScenarioKind::WalkRight => walk_right_scenario(level, walk_frames.unwrap_or(300)),
        RecordScenarioKind::WalkRightRedStealth => {
            walk_right_red_stealth_scenario(level, walk_frames.unwrap_or(700))
        }
        RecordScenarioKind::WalkRightIronKnight => {
            walk_right_iron_knight_scenario(level, walk_frames.unwrap_or(700))
        }
        RecordScenarioKind::WalkRightJuggernaut => {
            walk_right_juggernaut_scenario(level, walk_frames.unwrap_or(700))
        }
        RecordScenarioKind::PrizeBlock => prize_block_scenario(level),
        RecordScenarioKind::Issue216PrizeAudioAudit => issue_216_prize_audio_audit_scenario(),
        RecordScenarioKind::Level00SlopeRightProbe => level_00_slope_right_probe_scenario(),
        RecordScenarioKind::Level00SlopeRightProbeRedStealth => {
            level_00_slope_right_probe_red_stealth_scenario()
        }
        RecordScenarioKind::Level00SlopeRightProbeIronKnight => {
            level_00_slope_right_probe_iron_knight_scenario()
        }
        RecordScenarioKind::Level00SlopeRightProbeJuggernaut => {
            level_00_slope_right_probe_juggernaut_scenario()
        }
        RecordScenarioKind::StompEnemy => stomp_enemy_scenario(level, walk_frames.unwrap_or(360)),
        RecordScenarioKind::EnemySpawnAuditHeadyMetal => enemy_spawn_audit_scenario(0x20, level),
        RecordScenarioKind::LavaWeatherAudit => lava_weather_audit_scenario(level),
        RecordScenarioKind::DiamondPowerCircleAudit => diamond_power_circle_audit_scenario(level),
        RecordScenarioKind::DiamondPowerWallAudit => diamond_power_wall_audit_scenario(level),
        RecordScenarioKind::DiamondPowerSnakeAudit => diamond_power_snake_audit_scenario(level),
        RecordScenarioKind::DiamondPowerInvulnerabilityAudit => {
            diamond_power_invulnerability_audit_scenario(level)
        }
        RecordScenarioKind::DiamondPowerSamuraiHazeAudit => {
            diamond_power_samurai_haze_audit_scenario(level)
        }
        RecordScenarioKind::HighwaterShooterBarrierAudit => {
            highwater_shooter_barrier_audit_scenario()
        }
        RecordScenarioKind::HighwaterWaterPriorityAudit => {
            highwater_water_priority_audit_scenario()
        }
        RecordScenarioKind::HighwaterFlagTransitionAudit => {
            highwater_flag_transition_audit_scenario()
        }
        RecordScenarioKind::HighwaterPass2JuggernautFlagTransitionAudit => {
            highwater_pass2_juggernaut_flag_transition_audit_scenario()
        }
        RecordScenarioKind::HighwaterPass2GrowShooterTouchAudit => {
            highwater_pass2_grow_shooter_touch_audit_scenario()
        }
        RecordScenarioKind::Issue147WallSideJumpAudit => issue_147_wall_side_jump_audit_scenario(),
        RecordScenarioKind::Issue213DripsMetalBlockAudit => {
            issue_213_drips_metal_block_audit_scenario()
        }
        RecordScenarioKind::Issue213DripsDelayedBDropAudit => {
            issue_213_drips_delayed_b_drop_audit_scenario()
        }
        RecordScenarioKind::Issue249WallNoJumpAudit => issue_249_wall_no_jump_audit_scenario(),
        RecordScenarioKind::Issue249JuggernautWallNoJumpAudit => {
            issue_249_juggernaut_wall_no_jump_audit_scenario()
        }
        RecordScenarioKind::Issue249JuggernautWallJumpAudit => {
            issue_249_juggernaut_wall_jump_audit_scenario()
        }
        RecordScenarioKind::Issue249Blw2LeftPitWallNoJumpAudit => {
            issue_249_blw2_left_pit_wall_no_jump_audit_scenario()
        }
        RecordScenarioKind::Issue249Blw2LeftPitWallJumpAudit => {
            issue_249_blw2_left_pit_wall_jump_audit_scenario()
        }
    }
}

fn load_record_scenario_file(path: &Path) -> Result<DynamicScenario> {
    let scenario_json =
        fs::read_to_string(path).with_context(|| format!("failed to read {}", path.display()))?;
    serde_json::from_str(&scenario_json).context("invalid scenario JSON")
}

fn recording_scenario_manifest(
    mut scenario: DynamicScenario,
    start: recording::RecordingStart,
) -> DynamicScenario {
    if start.settled && start.warp_level.is_some() {
        scenario.settle_oracle_warp = true;
    }
    scenario
}

fn recording_start_for_dynamic_scenario(scenario: &DynamicScenario) -> recording::RecordingStart {
    recording::RecordingStart {
        warp_level: scenario.warp_to_level,
        settled: scenario.warp_to_level.is_some(),
        record_frames: None,
    }
}

fn record_scenario_command(
    slug: &str,
    kind: RecordScenarioKind,
    level: u8,
    walk_frames: Option<usize>,
) -> String {
    use std::fmt::Write;

    let scenario = match kind {
        RecordScenarioKind::Idle => "idle",
        RecordScenarioKind::WalkRight => "walk-right",
        RecordScenarioKind::WalkRightRedStealth => "walk-right-red-stealth",
        RecordScenarioKind::WalkRightIronKnight => "walk-right-iron-knight",
        RecordScenarioKind::WalkRightJuggernaut => "walk-right-juggernaut",
        RecordScenarioKind::PrizeBlock => "prize-block",
        RecordScenarioKind::Issue216PrizeAudioAudit => "issue216-prize-audio-audit",
        RecordScenarioKind::Level00SlopeRightProbe => "level00-slope-right-probe",
        RecordScenarioKind::Level00SlopeRightProbeRedStealth => {
            "level00-slope-right-probe-red-stealth"
        }
        RecordScenarioKind::Level00SlopeRightProbeIronKnight => {
            "level00-slope-right-probe-iron-knight"
        }
        RecordScenarioKind::Level00SlopeRightProbeJuggernaut => {
            "level00-slope-right-probe-juggernaut"
        }
        RecordScenarioKind::StompEnemy => "stomp-enemy",
        RecordScenarioKind::EnemySpawnAuditHeadyMetal => "enemy-spawn-audit-heady-metal",
        RecordScenarioKind::LavaWeatherAudit => "lava-weather-audit",
        RecordScenarioKind::DiamondPowerCircleAudit => "diamond-power-circle-audit",
        RecordScenarioKind::DiamondPowerWallAudit => "diamond-power-wall-audit",
        RecordScenarioKind::DiamondPowerSnakeAudit => "diamond-power-snake-audit",
        RecordScenarioKind::DiamondPowerInvulnerabilityAudit => {
            "diamond-power-invulnerability-audit"
        }
        RecordScenarioKind::DiamondPowerSamuraiHazeAudit => "diamond-power-samurai-haze-audit",
        RecordScenarioKind::HighwaterShooterBarrierAudit => "highwater-shooter-barrier-audit",
        RecordScenarioKind::HighwaterWaterPriorityAudit => "highwater-water-priority-audit",
        RecordScenarioKind::HighwaterFlagTransitionAudit => "highwater-flag-transition-audit",
        RecordScenarioKind::HighwaterPass2JuggernautFlagTransitionAudit => {
            "highwater-pass2-juggernaut-flag-transition-audit"
        }
        RecordScenarioKind::HighwaterPass2GrowShooterTouchAudit => {
            "highwater-pass2-grow-shooter-touch-audit"
        }
        RecordScenarioKind::Issue147WallSideJumpAudit => "issue147-wall-side-jump-audit",
        RecordScenarioKind::Issue213DripsMetalBlockAudit => "issue213-drips-metal-block-audit",
        RecordScenarioKind::Issue213DripsDelayedBDropAudit => "issue213-drips-delayed-b-drop-audit",
        RecordScenarioKind::Issue249WallNoJumpAudit => "issue249-wall-no-jump-audit",
        RecordScenarioKind::Issue249JuggernautWallNoJumpAudit => {
            "issue249-juggernaut-wall-no-jump-audit"
        }
        RecordScenarioKind::Issue249JuggernautWallJumpAudit => {
            "issue249-juggernaut-wall-jump-audit"
        }
        RecordScenarioKind::Issue249Blw2LeftPitWallNoJumpAudit => {
            "issue249-blw2-left-pit-wall-no-jump-audit"
        }
        RecordScenarioKind::Issue249Blw2LeftPitWallJumpAudit => {
            "issue249-blw2-left-pit-wall-jump-audit"
        }
    };
    let mut command = format!(
        "cargo run --release -p kidcli -- record-scenario --slug {slug} --scenario {scenario} --level {level:02X}"
    );
    if let Some(frames) = walk_frames {
        let _ = write!(command, " --walk-frames {frames}");
    }
    command
}

fn record_scenario_file_command(slug: &str, scenario: &Path) -> String {
    format!(
        "cargo run --release -p kidcli -- record-scenario-file --slug {slug} --scenario {}",
        scenario.display()
    )
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
    let audio_queue = audio.as_ref().and_then(open_audio_queue);
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
                    controller = controller.filter(sdl2::controller::GameController::attached);
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
        if let Some(remaining) = frame_time.checked_sub(elapsed) {
            thread::sleep(remaining);
        }
        frame_index = frame_index.saturating_add(1);
    }

    Ok(())
}

fn open_audio_queue(audio: &sdl2::AudioSubsystem) -> Option<sdl2::audio::AudioQueue<f32>> {
    let desired = sdl2::audio::AudioSpecDesired {
        freq: Some(44_100),
        channels: Some(2),
        samples: Some(1024),
    };
    let Ok(queue) = audio.open_queue::<f32, _>(None, &desired) else {
        return None;
    };
    queue.resume();
    Some(queue)
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
        DynamicScenario, GameplayCapture, GameplayScenario, GameplayState, PlayerForm,
        RecordScenarioKind, ReferenceName, analyse_audio_samples,
        apply_dynamic_scenario_input_overlays, audio_envelope_similarity, build_controller_script,
        build_record_scenario, capture_native_gameplay_run, capture_native_gameplay_scenario,
        compare_native_gameplay_with, compare_native_states_against_oracle,
        ensure_gameplay_level_layouts_with, ensure_level_layouts_with, find_gameplay_live_drifts,
        gameplay_checkpoint_window, native_trace_to_oracle_state, oracle_probe_watch_row_in_range,
        parse_button_mask, parse_input_spec, prepare_generated_assets_with,
        record_scenario_command, record_scenario_file_command, recording_scenario_manifest,
        recording_start_for_dynamic_scenario, selected_reference_repos,
    };
    use crate::native_player::validate_native_session_assets;
    use crate::oracle::{EnemyFacing, OracleEnemyState, OracleState};
    use crate::recording;
    use crate::scenarios::{
        DynamicStartPosition, NativeDynamicRunOptions, run_native_dynamic_scenario,
    };
    use crate::verify::verify_soak_level_10m;
    use kid_core::{GameMode, LevelId, NativeObjectKind, NativeObjectSnapshot, NativeTraceFrame};
    use kid_data::{
        AssetManifest, export_split_bundle, load_manifest, manifest_path as generated_manifest_path,
    };
    use kid_rom::{RomHeader, RomIdentity, VerifiedRom};
    use std::cell::{Cell, RefCell};
    use std::fs;
    use std::path::{Path, PathBuf};
    use tempfile::TempDir;

    const SAXMAN_FIXTURE: &[u8] = &[0x00, 0x02, 0xF4, 0x60, 0x41, 0x42, 0x43, 0x44, 0x00, 0x00];
    const ENIGMA_FIXTURE: &[u8] = &[0x05, 0x00, 0x00, 0x01, 0x00, 0x0A, 0x09, 0x18, 0x10, 0xFE];

    fn cached_generated_manifest() -> Option<AssetManifest> {
        let repo_root = if let Ok(override_root) = std::env::var("KID_REPO_ROOT")
            && !override_root.is_empty()
        {
            PathBuf::from(override_root)
        } else {
            let crate_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
            crate_dir
                .parent()
                .and_then(Path::parent)
                .expect("kidcli crate should live under <repo>/crates/kidcli")
                .to_path_buf()
        };
        let manifest_path = repo_root
            .join(".cache/generated/28b904000b2863b6760531807760b571f1a5fc1d/asset-manifest.json");
        if !manifest_path.is_file() {
            eprintln!(
                "skipping cache-backed kidcli test: generated asset manifest missing at {}",
                manifest_path.display()
            );
            return None;
        }
        Some(load_manifest(&manifest_path).expect("load cached generated asset manifest"))
    }

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
        fs::write(source_root.join("ingame/artunc/diamond_1.bin"), [0; 32]).expect("diamond");
        fs::write(source_root.join("ingame/artunc_kid/kid.bin"), [0; 32 * 12]).expect("kid idle");
        fs::write(
            source_root.join("ingame/artunc_kid/kid_walk_1.bin"),
            [0; 32 * 12],
        )
        .expect("kid walk");
        fs::write(
            source_root.join("ingame/artunc_kid/kid_jumping_1.bin"),
            [0; 32 * 12],
        )
        .expect("kid jump rise");
        fs::write(
            source_root.join("ingame/artunc_kid/kid_jumping_2.bin"),
            [0; 32 * 12],
        )
        .expect("kid jump fall");
        fs::write(
            source_root.join("ingame/palette_enemy/Crystal_1.bin"),
            [0; 30],
        )
        .expect("diamond palette");
        fs::write(
            source_root.join("ingame/palette_enemy/Mini_Hopping_Skull_1.bin"),
            [0; 30],
        )
        .expect("skull palette");
        fs::write(source_root.join("ingame/palette_kid/Kid.bin"), [0; 30]).expect("kid palette");
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
            source_root.join("level/levelnames.asm"),
            r#"TitleText_00:
	dc.b	"blue\0lake\0woods"
	dc.b	$FF

TitleTextLayout_3Lines:
	dc.w	$20, $28
	dc.w	$20, $48
	dc.w	$20, $68
	dc.w	$DC, $50

AddrTbl_LevelNames:
	levnamhdr	TitleText_00, TitleTextLayout_3Lines, 1	;  0
"#,
        )
        .expect("levelnames");
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

    #[test]
    fn dynamic_record_scenario_input_overlays_or_buttons_into_existing_script() {
        let scenario = DynamicScenario {
            slug: String::from("test-overlay"),
            script: vec![0, 0, 1 << 3, 1 << 3],
            frame_count: 4,
            checkpoints: Vec::new(),
            start_position: None,
            camera_position: None,
            playtest_form: None,
            initial_diamond_count: None,
            warp_to_level: Some(0x01),
            settle_oracle_warp: false,
        };

        let scenario = apply_dynamic_scenario_input_overlays(scenario, &[String::from("1-2:b")])
            .expect("overlay should apply");

        assert_eq!(scenario.script[1], 1 << 4);
        assert_eq!(scenario.script[2], (1 << 3) | (1 << 4));
        assert_eq!(scenario.frame_count, 4);
    }

    #[test]
    fn dynamic_record_scenario_input_overlays_extend_script_and_frame_count() {
        let scenario = DynamicScenario {
            slug: String::from("test-overlay-extend"),
            script: vec![0, 0],
            frame_count: 2,
            checkpoints: Vec::new(),
            start_position: None,
            camera_position: None,
            playtest_form: None,
            initial_diamond_count: None,
            warp_to_level: Some(0x01),
            settle_oracle_warp: false,
        };

        let scenario =
            apply_dynamic_scenario_input_overlays(scenario, &[String::from("5:right+c")])
                .expect("overlay should extend");

        assert!(scenario.script.len() >= 7);
        assert_eq!(scenario.frame_count, 6);
        assert_eq!(scenario.script[5], (1 << 3) | (1 << 5));
    }

    #[test]
    fn dynamic_record_scenario_input_none_clears_existing_script_range() {
        let scenario = DynamicScenario {
            slug: String::from("test-overlay-clear"),
            script: vec![1 << 3, 1 << 3, (1 << 3) | (1 << 4), 1 << 2],
            frame_count: 4,
            checkpoints: Vec::new(),
            start_position: None,
            camera_position: None,
            playtest_form: None,
            initial_diamond_count: None,
            warp_to_level: Some(0x01),
            settle_oracle_warp: false,
        };

        let scenario = apply_dynamic_scenario_input_overlays(scenario, &[String::from("1-2:none")])
            .expect("none overlay should clear scripted buttons");

        assert_eq!(scenario.script, vec![1 << 3, 0, 0, 1 << 2]);
        assert_eq!(scenario.frame_count, 4);
    }

    #[test]
    fn oracle_probe_watch_rows_respect_requested_frame_range() {
        let mut before = std::collections::HashMap::new();
        before.insert(String::from("frame"), 959);

        let mut start = std::collections::HashMap::new();
        start.insert(String::from("frame"), 960);

        let mut end = std::collections::HashMap::new();
        end.insert(String::from("frame"), 962);

        let mut after = std::collections::HashMap::new();
        after.insert(String::from("frame"), 963);

        assert!(!oracle_probe_watch_row_in_range(&before, 960, 962));
        assert!(oracle_probe_watch_row_in_range(&start, 960, 962));
        assert!(oracle_probe_watch_row_in_range(&end, 960, 962));
        assert!(!oracle_probe_watch_row_in_range(&after, 960, 962));
    }

    #[test]
    fn oracle_probe_watch_rows_reject_missing_or_negative_frames() {
        let missing = std::collections::HashMap::new();
        let mut negative = std::collections::HashMap::new();
        negative.insert(String::from("frame"), -1);

        assert!(!oracle_probe_watch_row_in_range(&missing, 0, 10));
        assert!(!oracle_probe_watch_row_in_range(&negative, 0, 10));
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
            collision_type: None,
            collision_source: None,
            layout_index: None,
            layout_ptr: None,
            parent_object_ptr: None,
            gfx_slot_index: None,
            on_ground: None,
            current_hp: None,
            debug_fields: std::collections::BTreeMap::new(),
        }
    }

    fn oracle_state(
        label: &str,
        frame_index: usize,
        active_enemies: Vec<OracleEnemyState>,
    ) -> OracleState {
        oracle_state_with_game_mode(label, frame_index, 0x000C, active_enemies)
    }

    fn oracle_state_with_game_mode(
        label: &str,
        frame_index: usize,
        game_mode: u16,
        active_enemies: Vec<OracleEnemyState>,
    ) -> OracleState {
        OracleState {
            label: label.to_owned(),
            frame_index,
            game_mode,
            boot_phase: None,
            boot_phase_elapsed: None,
            title_screen_ready: false,
            main_menu_option: None,
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
            kid_x_vel: None,
            kid_y_vel: None,
            character_movement: None,
            just_received_damage: None,
            kid_collision_type: None,
            kid_collision_source: None,
            kid_x_direction: None,
            invulnerable_frames: None,
            currently_transforming: None,
            rng_offset: None,
            level_special_effects: None,
            gameplay_debug_fields: std::collections::BTreeMap::new(),
            cpu_registers: None,
        }
    }

    #[test]
    fn native_trace_to_oracle_state_includes_sorted_enemy_snapshots() {
        let frame = NativeTraceFrame {
            frame_index: 810,
            game_mode: GameMode::InGame,
            game_mode_value: 0x000C,
            start_press_count: 6,
            boot_phase: None,
            boot_phase_elapsed: None,
            title_screen_ready: false,
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
            extra_hitpoint_slots: 0,
            current_helmet_id: 0,
            red_stealth_sword_timer: 0,
            red_stealth_airborne_slash_counter: 0,
            paused: false,
            kid_x_pos: Some(0x0020_0000),
            kid_y_pos: Some(0x0010_0000),
            kid_x_velocity: Some(0),
            kid_y_velocity: Some(0),
            character_movement: None,
            kid_facing: None,
            kid_on_ground: Some(true),
            walk_visual_accumulator: 0,
            hud_diamond_frame_offset: None,
            berzerker_charging: false,
            berzerker_charge_pose: false,
            berzerker_walk_pose: false,
            ledge_snap_cooldown: 0,
            kid_on_slope: 0,
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
                    shared_walker_state: None,
                    shared_walker_patrol_timer: None,
                    shared_walker_patrol_short_next: None,
                    walk_phase: None,
                    wait_timer: None,
                    debug_fields: std::collections::BTreeMap::new(),
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
                    shared_walker_state: None,
                    shared_walker_patrol_timer: None,
                    shared_walker_patrol_short_next: None,
                    walk_phase: None,
                    wait_timer: None,
                    debug_fields: std::collections::BTreeMap::new(),
                },
            ],
            active_enemy_details: Vec::new(),
            camera_x_pos: 0x0010_0000,
            camera_y_pos: 0x0008_0000,
            number_objects: 3,
            number_gfx_objects: 3,
            rng_offset: None,
            level_complete: false,
            level_complete_elapsed_frames: 0,
            level_complete_score_rows_elapsed_frames: 0,
            level_complete_score_counter_elapsed_frames: 0,
            level_complete_score_counter_drained: false,
            level_complete_initial_score: 0,
            level_complete_score_rows_started: 0,
            level_completion_time: 0,
            level_skip_cheat_enabled: 0,
            game_over: false,
            game_over_elapsed_frames: 0,
            continue_countdown: None,
            continue_countdown_elapsed_frames: 0,
            game_complete: false,
            ending_elapsed: None,
            number_continues: 3,
            invulnerable_frames: 0,
            kid_is_invulnerable: false,
            just_received_damage: false,
            diamond_power_active: false,
            diamond_count: 0,
            score: 0,
            level_timer_seconds: 180,
            level_damage_taken: false,
            level_prize_collected: false,
            level_speed_bonus_awarded: false,
            level_path_bonus_amount: 0,
            murder_wall_x: None,
            active_projectiles: Vec::new(),
            active_projectile_details: Vec::new(),
            diamond_power_sprites: Vec::new(),
            active_geysers: Vec::new(),
            block_bounces: Vec::new(),
            block_destroy_effects: Vec::new(),
            block_debris_pieces: Vec::new(),
            prize_block_spawn_flashes: Vec::new(),
            floating_items: Vec::new(),
            enemy_death_effects: Vec::new(),
            stomped_this_frame: false,
            death_windup_elapsed: 0,
            form_change_flash_frames: 0,
            blocks_hit_this_frame: 0,
            sound_test_track: None,
            sound_test_play_pending: false,
            gameplay_debug_fields: std::collections::BTreeMap::new(),
            physics_trace: None,
            options_cursor: None,
            played_sound_effect_ids: Vec::new(),
            two_player: false,
            options_two_player_two_controllers: false,
            options_controls_scheme: 0,
            options_speed_button_fast: false,
        };

        let state = native_trace_to_oracle_state("enemy_window_open", 810, &frame);

        assert_eq!(state.active_enemies.len(), 2);
        assert_eq!(state.active_enemies[0].object_type, 0x01);
        assert_eq!(state.active_enemies[1].object_type, 0x1C);
        assert_eq!(state.active_enemies[1].x_velocity, -0x0001_8000);
        assert_eq!(state.active_enemies[1].facing, EnemyFacing::Left);
        assert_eq!(state.character_movement, None);
        assert_eq!(state.just_received_damage, Some(false));

        let mut hidden_frame = frame.clone();
        hidden_frame.kid_x_pos = None;
        hidden_frame.kid_y_pos = None;
        hidden_frame.kid_x_velocity = None;
        hidden_frame.kid_y_velocity = None;
        hidden_frame.character_movement = None;
        let hidden_state = native_trace_to_oracle_state("title", 360, &hidden_frame);
        assert_eq!(hidden_state.kid_x_pos, 0);
        assert_eq!(hidden_state.kid_y_pos, 0);
        assert_eq!(hidden_state.kid_x_vel, Some(0));
        assert_eq!(hidden_state.kid_y_vel, Some(0));
        assert_eq!(
            hidden_state.character_movement,
            Some(0),
            "ASM WRAM keeps hidden boot/title motion mirrors as standingstill; native oracle-state export should match the gameplay capture normalisation",
        );
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
    fn compare_native_states_against_oracle_rejects_enemy_layout_index_divergence() {
        let run_dir = TempDir::new().expect("run dir");
        let mut native_enemy = oracle_enemy(
            0x1C,
            0x0030_0000,
            0x0010_0000,
            -0x0001_8000,
            -0x0002_0000,
            EnemyFacing::Left,
        );
        native_enemy.layout_index = Some(7);
        let mut oracle_enemy = oracle_enemy(
            0x1C,
            0x0030_0000,
            0x0010_0000,
            -0x0001_8000,
            -0x0002_0000,
            EnemyFacing::Left,
        );
        oracle_enemy.layout_index = Some(2);

        let native = vec![oracle_state("enemy_window_open", 810, vec![native_enemy])];
        let oracle = vec![oracle_state("enemy_window_open", 810, vec![oracle_enemy])];

        let error = compare_native_states_against_oracle(run_dir.path(), &native, &oracle)
            .expect_err("enemy layout index drift should fail");

        assert!(format!("{error:#}").contains("layout index"));
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
    fn compare_native_states_against_oracle_ignores_intro_gfx_object_snapshots() {
        let run_dir = TempDir::new().expect("run dir");
        let native = vec![oracle_state_with_game_mode(
            "title",
            360,
            GameMode::IntroSequence1.as_disassembly_value(),
            Vec::new(),
        )];
        let oracle = vec![oracle_state_with_game_mode(
            "title",
            360,
            GameMode::IntroSequence1.as_disassembly_value(),
            vec![oracle_enemy(
                0x30,
                0x0000_0000,
                0x0000_0000,
                0,
                0,
                EnemyFacing::Left,
            )],
        )];

        compare_native_states_against_oracle(run_dir.path(), &native, &oracle)
            .expect("ASM Load_IntroSequence1 title gfx workers are not gameplay enemy snapshots");
    }

    #[test]
    fn compare_native_states_against_oracle_still_rejects_gameplay_enemy_count() {
        let run_dir = TempDir::new().expect("run dir");
        let native = vec![oracle_state("enemy_window_open", 810, Vec::new())];
        let oracle = vec![oracle_state(
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

        let error = compare_native_states_against_oracle(run_dir.path(), &native, &oracle)
            .expect_err("gameplay enemy count drift should still fail");

        assert!(format!("{error:#}").contains("enemy count"));
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
        stale_manifest.decoded_levels = vec![];
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
        assert!(!refreshed.decoded_levels.is_empty());
        let refreshed_manifest =
            load_manifest(&stale_manifest_path).expect("refreshed manifest should load");
        assert!(!refreshed_manifest.decoded_levels.is_empty());
        let decoded = refreshed_manifest
            .decoded_levels
            .iter()
            .find(|d| d.level_id == 0x00)
            .expect("decoded metadata");
        assert!(
            repo_root
                .path()
                .join(".cache/generated/abc123")
                .join(&decoded.foreground.relative_path)
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
            generated_bundle_version: 8,
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
            decoded_levels: vec![],
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
    fn ensure_gameplay_level_layouts_skips_known_oracle_warp_broken_levels() {
        let bundle_root = TempDir::new().expect("bundle root");
        let layout_dir = bundle_root.path().join("level-layout");
        fs::create_dir_all(&layout_dir).expect("layout dir");
        for level_id in [
            0x00u8, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x0A, 0x0B, 0x0C, 0x0D, 0x0E,
            0x0F, 0x10, 0x11, 0x12, 0x13, 0x14, 0x15, 0x16, 0x17, 0x18, 0x19, 0x1A, 0x1B, 0x1C,
            0x1D, 0x1E, 0x1F,
        ] {
            fs::write(layout_dir.join(format!("{level_id:02X}.bin")), [0x60, 0x00])
                .expect("existing layout");
            fs::write(
                layout_dir.join(format!("{level_id:02X}.json")),
                br#"{"width_blocks":200,"height_blocks":28,"data_bytes":2}"#,
            )
            .expect("existing layout metadata");
        }

        let manifest = AssetManifest {
            generated_bundle_version: 8,
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
            decoded_levels: vec![],
        };

        let dumped_levels = RefCell::new(Vec::new());
        ensure_gameplay_level_layouts_with(
            b"rom",
            &manifest,
            GameplayScenario::SoakLevel10m,
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
        .expect("oracle-warp-broken level should be skipped");

        assert!(dumped_levels.borrow().is_empty());
        assert!(!layout_dir.join("09.bin").exists());
        assert!(!layout_dir.join("09.json").exists());
    }

    #[test]
    fn ensure_level_layouts_only_dumps_requested_missing_levels() {
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
            generated_bundle_version: 8,
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
            decoded_levels: vec![],
        };

        let dumped_levels = RefCell::new(Vec::new());
        ensure_level_layouts_with(b"rom", &manifest, &[0x00, 0x02], |_, level_id, out_dir| {
            dumped_levels.borrow_mut().push(level_id);
            fs::write(out_dir.join(format!("{level_id:02X}.bin")), [0x60, 0x00])
                .expect("dumped layout");
            fs::write(
                out_dir.join(format!("{level_id:02X}.json")),
                br#"{"width_blocks":200,"height_blocks":28,"data_bytes":2}"#,
            )
            .expect("dumped layout metadata");
            Ok(())
        })
        .expect("missing requested layouts should be dumped");

        assert_eq!(*dumped_levels.borrow(), vec![0x02]);
        assert!(layout_dir.join("02.bin").is_file());
        assert!(layout_dir.join("02.json").is_file());
        assert!(!layout_dir.join("01.bin").exists());
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
        assert_eq!(capture.checkpoints.len(), 5);
        assert!(
            capture
                .checkpoints
                .iter()
                .any(|state| state.label == "respawn_bootstrap_tail"),
            "default native gameplay capture should reach the late hidden respawn checkpoint"
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

    #[test]
    fn soak_level_10m_verifier_uses_captured_stable_gameplay_not_stale_checkpoint() {
        let mut stale_level_settle = gameplay_state("level_settle", 719);
        stale_level_settle.game_mode = GameMode::LevelTransition.as_disassembly_value();

        let mut states = Vec::new();
        for level in 0..8u16 {
            let mut state = gameplay_state(&format!("frame_{:05}", 720 + usize::from(level)), 720);
            state.current_level_id = level;
            state.player_1_level_id = level;
            states.push(state);
        }
        let mut soak_end = states.last().cloned().expect("states");
        soak_end.label = String::from("soak_end");

        let capture = GameplayCapture {
            scenario: String::from("native_soak_level_10m"),
            frame_count: 800,
            missed_vblank_frames: 0,
            states,
            checkpoints: vec![stale_level_settle, soak_end],
        };

        verify_soak_level_10m(&capture)
            .expect("stable captured gameplay should satisfy the soak gate");
    }

    #[test]
    fn soak_level_10m_verifier_rejects_transition_only_level_coverage() {
        let mut states = Vec::new();
        for level in 0..8u16 {
            let mut state =
                gameplay_state(&format!("transition_{level}"), 720 + usize::from(level));
            state.game_mode = GameMode::LevelTransition.as_disassembly_value();
            state.current_level_id = level;
            state.player_1_level_id = level;
            states.push(state);
        }

        let mut soak_end = gameplay_state("soak_end", 800);
        soak_end.current_level_id = 7;
        soak_end.player_1_level_id = 7;

        let capture = GameplayCapture {
            scenario: String::from("native_soak_level_10m"),
            frame_count: 800,
            missed_vblank_frames: 0,
            states,
            checkpoints: vec![soak_end],
        };

        let error = verify_soak_level_10m(&capture)
            .expect_err("transition-only level churn must not satisfy the soak gate");
        assert!(
            error
                .to_string()
                .contains("never reached stable in-level gameplay"),
            "{error:#}"
        );
    }

    #[test]
    fn gameplay_regression_verify_death_respawn_accepts_hidden_bootstrap_at_script_end() {
        let capture = GameplayCapture {
            scenario: String::from("death_respawn_level1"),
            frame_count: 7_501,
            missed_vblank_frames: 0,
            states: vec![
                GameplayState {
                    label: String::from("respawn_tail"),
                    frame_index: 1_919,
                    game_mode: 0x0008,
                    current_player: 0,
                    number_lives: 2,
                    number_hitpoints: 2,
                    current_level_id: 0,
                    player_1_level_id: 0,
                    current_helmet_id: 0,
                    camera_x_pos: 0,
                    camera_y_pos: 0,
                    kid_x_pos: 0x0040_0000,
                    kid_y_pos: 0x0068_0000,
                    kid_x_vel: Some(0),
                    kid_y_vel: Some(0),
                    character_movement: Some(0),
                    kid_collision_type: Some(0),
                    invulnerable_frames: Some(0),
                    just_received_damage: Some(false),
                    kid_ground_probe: None,
                    kid_grounded_release_probe: None,
                    kid_jump_gravity_probe: None,
                    number_objects: 15,
                    number_gfx_objects: 14,
                },
                GameplayState {
                    label: String::from("respawn_bootstrap_tail"),
                    frame_index: 7_500,
                    game_mode: 0x0008,
                    current_player: 0,
                    number_lives: 2,
                    number_hitpoints: 2,
                    current_level_id: 0,
                    player_1_level_id: 0,
                    current_helmet_id: 0,
                    camera_x_pos: 0,
                    camera_y_pos: 0,
                    kid_x_pos: 0x0040_0000,
                    kid_y_pos: 0x0068_0000,
                    kid_x_vel: Some(0),
                    kid_y_vel: Some(0),
                    character_movement: Some(0),
                    kid_collision_type: Some(0),
                    invulnerable_frames: Some(0),
                    just_received_damage: Some(false),
                    kid_ground_probe: None,
                    kid_grounded_release_probe: None,
                    kid_jump_gravity_probe: None,
                    number_objects: 15,
                    number_gfx_objects: 14,
                },
            ],
            checkpoints: vec![
                GameplayState {
                    label: String::from("level_settle"),
                    frame_index: 719,
                    number_lives: 3,
                    number_hitpoints: 1,
                    game_mode: 0x000C,
                    ..gameplay_state("level_settle", 719)
                },
                GameplayState {
                    label: String::from("pre_death"),
                    frame_index: 1_680,
                    number_lives: 3,
                    number_hitpoints: 1,
                    game_mode: 0x000C,
                    ..gameplay_state("pre_death", 1_680)
                },
                GameplayState {
                    label: String::from("death_tail"),
                    frame_index: 1_762,
                    number_lives: 2,
                    number_hitpoints: 2,
                    game_mode: 0x0008,
                    ..gameplay_state("death_tail", 1_762)
                },
                GameplayState {
                    label: String::from("respawn_tail"),
                    frame_index: 1_919,
                    game_mode: 0x0008,
                    number_lives: 2,
                    number_hitpoints: 2,
                    camera_x_pos: 0,
                    camera_y_pos: 0,
                    kid_x_pos: 0x0040_0000,
                    kid_y_pos: 0x0068_0000,
                    kid_x_vel: Some(0),
                    kid_y_vel: Some(0),
                    character_movement: Some(0),
                    kid_collision_type: Some(0),
                    invulnerable_frames: Some(0),
                    just_received_damage: Some(false),
                    kid_ground_probe: None,
                    kid_grounded_release_probe: None,
                    kid_jump_gravity_probe: None,
                    number_objects: 15,
                    number_gfx_objects: 14,
                    ..gameplay_state("respawn_tail", 1_919)
                },
                GameplayState {
                    label: String::from("respawn_bootstrap_tail"),
                    frame_index: 9_000,
                    game_mode: 0x0008,
                    number_lives: 2,
                    number_hitpoints: 2,
                    camera_x_pos: 0,
                    camera_y_pos: 0,
                    kid_x_pos: 0x0040_0000,
                    kid_y_pos: 0x0068_0000,
                    kid_x_vel: Some(0),
                    kid_y_vel: Some(0),
                    character_movement: Some(0),
                    kid_collision_type: Some(0),
                    invulnerable_frames: Some(0),
                    just_received_damage: Some(false),
                    kid_ground_probe: None,
                    kid_grounded_release_probe: None,
                    kid_jump_gravity_probe: None,
                    number_objects: 15,
                    number_gfx_objects: 14,
                    ..gameplay_state("respawn_bootstrap_tail", 9_000)
                },
            ],
        };

        crate::verify::verify_death_respawn_level_1(&capture)
            .expect("extended death/respawn smoke path should still end on the hidden bootstrap");
    }

    #[test]
    fn gameplay_regression_compare_dynamic_gameplay_with_uses_custom_window_slug() {
        let repo_root = TempDir::new().expect("repo root");
        let capture = GameplayCapture {
            scenario: String::from("death_respawn_level1_window_01216_01220_step_1"),
            frame_count: 5,
            missed_vblank_frames: 0,
            states: vec![
                gameplay_state("frame_01216", 1_216),
                gameplay_state("frame_01217", 1_217),
                gameplay_state("frame_01218", 1_218),
                gameplay_state("frame_01219", 1_219),
                gameplay_state("frame_01220", 1_220),
            ],
            checkpoints: vec![
                gameplay_state("frame_01216", 1_216),
                gameplay_state("frame_01217", 1_217),
                gameplay_state("frame_01218", 1_218),
                gameplay_state("frame_01219", 1_219),
                gameplay_state("frame_01220", 1_220),
            ],
        };

        crate::compare::compare_dynamic_gameplay_with(
            repo_root.path(),
            "death_respawn_level1_window_01216_01220_step_1",
            GameplayScenario::DeathRespawnLevel1,
            || Ok(capture.clone()),
            || Ok(capture.clone()),
            || Ok(Vec::new()),
        )
        .expect("dynamic gameplay comparison should pass");

        let run_dir = repo_root
            .path()
            .join(".cache/native-gameplay-oracle-runs")
            .join("death_respawn_level1_window_01216_01220_step_1");
        assert!(run_dir.join("native/checkpoints.json").is_file());
        assert!(run_dir.join("oracle/checkpoints.json").is_file());
    }

    #[test]
    fn gameplay_regression_dynamic_death_window_uses_hidden_object_tolerance() {
        let tolerance = crate::compare::gameplay_checkpoint_tolerance(
            GameplayScenario::DeathRespawnLevel1,
            "frame_01217",
        );
        assert_eq!(tolerance.number_objects, 8);
        assert_eq!(tolerance.number_gfx_objects, 8);
        assert_eq!(tolerance.kid_y, crate::capture::CAMERA_POSITION_TOLERANCE);
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
            kid_x_vel: Some(0x0002_0000),
            kid_y_vel: Some(0),
            character_movement: Some(0x0002),
            kid_collision_type: Some(0),
            invulnerable_frames: Some(0),
            just_received_damage: Some(false),
            kid_ground_probe: None,
            kid_grounded_release_probe: None,
            kid_jump_gravity_probe: None,
            number_objects: 3,
            number_gfx_objects: 3,
        }
    }

    #[test]
    fn build_record_scenario_warps_idle_to_requested_level() {
        let scenario = build_record_scenario(RecordScenarioKind::Idle, 0x07, Some(120));
        assert_eq!(scenario.warp_to_level, Some(0x07));
        assert!(scenario.frame_count >= 120, "idle frame count tracks input");
        assert!(
            scenario.script.iter().all(|&b| b == 0),
            "idle scenario must hold no buttons"
        );
    }

    #[test]
    fn build_record_scenario_walk_right_holds_right_during_walk_phase() {
        let scenario = build_record_scenario(RecordScenarioKind::WalkRight, 0x00, Some(120));
        assert_eq!(scenario.warp_to_level, Some(0x00));
        // Settle is 60 frames; after settle, the script must drive Right.
        let post_settle: Vec<u8> = scenario.script[60..(60 + 120)].to_vec();
        let right_held = post_settle.iter().filter(|&&b| b & (1 << 3) != 0).count();
        assert!(
            right_held > post_settle.len() / 2,
            "walk_right scenario must hold Right for the majority of post-settle frames \
             ({} of {})",
            right_held,
            post_settle.len()
        );
    }

    #[test]
    fn build_record_scenario_walk_right_form_routes_use_natural_level_path() {
        let cases = [
            (
                RecordScenarioKind::WalkRightRedStealth,
                "walk_right_red_stealth_00",
                PlayerForm::RedStealth.helmet_id(),
            ),
            (
                RecordScenarioKind::WalkRightIronKnight,
                "walk_right_iron_knight_00",
                PlayerForm::IronKnight.helmet_id(),
            ),
            (
                RecordScenarioKind::WalkRightJuggernaut,
                "walk_right_juggernaut_00",
                PlayerForm::Juggernaut.helmet_id(),
            ),
        ];

        for (kind, slug, helmet_id) in cases {
            let scenario = build_record_scenario(kind, 0x00, Some(700));

            assert_eq!(scenario.slug, slug);
            assert_eq!(scenario.warp_to_level, Some(0x00));
            assert_eq!(scenario.playtest_form, Some(helmet_id));
            assert_eq!(scenario.frame_count, 760);
            assert_eq!(scenario.start_position, None);
            assert_eq!(scenario.script[60] & (1 << 3), 1 << 3);
            assert_eq!(scenario.script[90] & (1 << 4), 1 << 4);
        }
    }

    #[test]
    fn build_record_scenario_kinds_warp_to_distinct_levels() {
        // Sanity: every kind targets the level it was given, not a default.
        for level in [0x00u8, 0x05u8, 0x10u8] {
            for kind in [
                RecordScenarioKind::Idle,
                RecordScenarioKind::WalkRight,
                RecordScenarioKind::PrizeBlock,
                RecordScenarioKind::StompEnemy,
                RecordScenarioKind::LavaWeatherAudit,
                RecordScenarioKind::DiamondPowerCircleAudit,
                RecordScenarioKind::DiamondPowerWallAudit,
                RecordScenarioKind::DiamondPowerSnakeAudit,
            ] {
                let scenario = build_record_scenario(kind, level, None);
                assert_eq!(
                    scenario.warp_to_level,
                    Some(level),
                    "{kind:?} at level {level:#04x} must warp"
                );
            }
        }
    }

    #[test]
    fn build_record_scenario_level_00_slope_probe_uses_named_window() {
        let scenario =
            build_record_scenario(RecordScenarioKind::Level00SlopeRightProbe, 0x05, None);

        assert_eq!(scenario.slug, "level_00_slope_right_probe");
        assert_eq!(
            scenario.warp_to_level,
            Some(0x00),
            "named Level 00 slope probe must not accidentally follow the generic --level argument"
        );
        assert!(scenario.settle_oracle_warp);
        assert_eq!(
            scenario.script.first(),
            Some(&(1 << 3)),
            "ASM loc_9D22/loc_9F18 consume held Right while slope collision is active"
        );
        assert_eq!(scenario.playtest_form, None);
        assert_eq!(
            scenario
                .start_position
                .map(|position| (position.x, position.y)),
            Some((760, 231))
        );
    }

    #[test]
    fn build_record_scenario_level_00_slope_probe_form_variants_use_same_window() {
        let cases = [
            (
                RecordScenarioKind::Level00SlopeRightProbeRedStealth,
                "level_00_slope_right_probe_red_stealth",
                PlayerForm::RedStealth.helmet_id(),
            ),
            (
                RecordScenarioKind::Level00SlopeRightProbeIronKnight,
                "level_00_slope_right_probe_iron_knight",
                PlayerForm::IronKnight.helmet_id(),
            ),
            (
                RecordScenarioKind::Level00SlopeRightProbeJuggernaut,
                "level_00_slope_right_probe_juggernaut",
                PlayerForm::Juggernaut.helmet_id(),
            ),
        ];

        for (kind, slug, helmet_id) in cases {
            let scenario = build_record_scenario(kind, 0x05, None);

            assert_eq!(scenario.slug, slug);
            assert_eq!(scenario.warp_to_level, Some(0x00));
            assert!(scenario.settle_oracle_warp);
            assert_eq!(scenario.playtest_form, Some(helmet_id));
            assert_eq!(
                scenario
                    .start_position
                    .map(|position| (position.x, position.y)),
                Some((760, 231)),
                "form probes must reuse the same ASM GfxObjects_Collision_UpSlope window"
            );
            assert_eq!(scenario.script.first(), Some(&(1 << 3)));
        }
    }

    #[test]
    fn build_record_scenario_lava_weather_audit_uses_named_settled_window() {
        let scenario = build_record_scenario(RecordScenarioKind::LavaWeatherAudit, 0x05, None);

        assert_eq!(scenario.slug, "lava_weather_audit_05");
        assert_eq!(scenario.warp_to_level, Some(0x05));
        assert!(scenario.settle_oracle_warp);
        assert_eq!(scenario.frame_count, 180);
    }

    #[test]
    fn build_record_scenario_diamond_power_audit_seeds_activation_probe() {
        let scenario =
            build_record_scenario(RecordScenarioKind::DiamondPowerCircleAudit, 0x00, None);

        assert_eq!(scenario.slug, "diamond_power_circle_audit_00");
        assert_eq!(scenario.playtest_form, Some(0));
        assert_eq!(scenario.initial_diamond_count, Some(20));
        assert_eq!(scenario.script[0], (1 << 7) | (1 << 6));
    }

    #[test]
    fn build_record_scenario_diamond_power_wall_audit_seeds_berzerker_tier2_probe() {
        let scenario = build_record_scenario(RecordScenarioKind::DiamondPowerWallAudit, 0x00, None);

        assert_eq!(scenario.slug, "diamond_power_wall_audit_00");
        assert_eq!(scenario.playtest_form, Some(7));
        assert_eq!(scenario.initial_diamond_count, Some(50));
        assert_eq!(scenario.script[0], (1 << 7) | (1 << 6));
    }

    #[test]
    fn build_record_scenario_diamond_power_snake_audit_seeds_micromax_tier1_probe() {
        let scenario =
            build_record_scenario(RecordScenarioKind::DiamondPowerSnakeAudit, 0x00, None);

        assert_eq!(scenario.slug, "diamond_power_snake_audit_00");
        assert_eq!(scenario.playtest_form, Some(9));
        assert_eq!(scenario.initial_diamond_count, Some(20));
        assert_eq!(scenario.script[0], (1 << 7) | (1 << 6));
    }

    #[test]
    fn build_record_scenario_diamond_power_invulnerability_audit_seeds_skycutter_tier1_probe() {
        let scenario = build_record_scenario(
            RecordScenarioKind::DiamondPowerInvulnerabilityAudit,
            0x02,
            None,
        );

        assert_eq!(scenario.slug, "diamond_power_invulnerability_audit_02");
        assert_eq!(scenario.playtest_form, Some(1));
        assert_eq!(scenario.initial_diamond_count, Some(20));
        assert_eq!(scenario.script[0], (1 << 7) | (1 << 6));
    }

    #[test]
    fn build_record_scenario_diamond_power_samurai_haze_audit_seeds_red_stealth_tier1_probe() {
        let scenario =
            build_record_scenario(RecordScenarioKind::DiamondPowerSamuraiHazeAudit, 0x02, None);

        assert_eq!(scenario.slug, "diamond_power_samurai_haze_audit_02");
        assert_eq!(scenario.playtest_form, Some(3));
        assert_eq!(scenario.initial_diamond_count, Some(20));
        assert_eq!(scenario.script[0], (1 << 7) | (1 << 6));
    }

    #[test]
    fn build_record_scenario_highwater_shooter_barrier_audit_drives_berzerker_route() {
        let scenario =
            build_record_scenario(RecordScenarioKind::HighwaterShooterBarrierAudit, 0x00, None);

        assert_eq!(scenario.slug, "highwater_shooter_barrier_audit_02");
        assert_eq!(
            scenario.warp_to_level,
            Some(0x02),
            "the named #214 probe must target Highwater Pass 1 regardless of the generic --level argument"
        );
        assert!(scenario.settle_oracle_warp);
        assert_eq!(
            scenario.playtest_form,
            Some(7),
            "ASM loc_9564 Berzerker charge hitbox is required for the first live shooter/barrier route"
        );
        assert!(
            scenario.script[..scenario.frame_count]
                .iter()
                .all(|mask| mask & (1 << 3) != 0),
            "the #214 route holds Right from the settled Level 02 spawn"
        );
        assert!(
            scenario
                .checkpoints
                .iter()
                .any(|checkpoint| checkpoint.label == "barrier_clear_check"),
            "recording evidence needs a durable checkpoint after loc_F02A should clear the ice barrier"
        );
        assert!(
            scenario.checkpoints.iter().any(|checkpoint| {
                checkpoint.label == "first_shooter_objects_visible_loc_ec76"
                    && checkpoint.frame_index == 92
            }),
            "oracle RAM probe shows ShooterObject_RAM first populated at frame 92, so #214 evidence must sample the first visible loc_EC76 bullets"
        );
        assert!(
            scenario.checkpoints.iter().any(|checkpoint| {
                checkpoint.label == "shooter_cascade_window_loc_f04a"
                    && checkpoint.frame_index == 104
            }),
            "the #214 route needs a checkpoint after loc_F04A cascades through the upper Highwater shooter cells"
        );
    }

    #[test]
    fn build_record_scenario_highwater_water_priority_audit_samples_overlap_route() {
        let scenario =
            build_record_scenario(RecordScenarioKind::HighwaterWaterPriorityAudit, 0x00, None);

        assert_eq!(scenario.slug, "highwater_water_priority_audit_02");
        assert_eq!(
            scenario.warp_to_level,
            Some(0x02),
            "the #215 water/foreground probe must target Highwater Pass 1 regardless of the generic --level argument"
        );
        assert!(scenario.settle_oracle_warp);
        assert_eq!(
            scenario.playtest_form,
            Some(7),
            "ASM loc_9564 Berzerker charge hitbox is required to break Highwater Pass 1's early smash-column stopper before sampling #215 water/foreground frames"
        );
        assert!(
            scenario.script[..scenario.frame_count]
                .iter()
                .all(|mask| mask & (1 << 3) != 0),
            "the #215 route holds Right from the settled Level 02 spawn"
        );
        assert_eq!(
            scenario.frame_count, 1_860,
            "the route needs to reach the first visible Mountain water hole from the settled Level 02 spawn"
        );
        assert!(
            scenario.checkpoints.iter().any(|checkpoint| {
                checkpoint.label == "waterfall_overlap_loc_300ba" && checkpoint.frame_index == 960
            }),
            "recording evidence needs a durable checkpoint where Transfer_ScrollDataToVRAM loc_300BA has latched the waterfall overlap camera"
        );
        assert!(
            scenario.checkpoints.iter().any(|checkpoint| {
                checkpoint.label == "waterfall_palette_tail_loc_1223c"
                    && checkpoint.frame_index == 1_859
            }),
            "the #215 route must keep a late palette-phase sample for RotatePalette_Lava_Water loc_1223C"
        );
    }

    #[test]
    fn build_record_scenario_highwater_flag_transition_audit_starts_at_real_flag() {
        let scenario =
            build_record_scenario(RecordScenarioKind::HighwaterFlagTransitionAudit, 0x00, None);

        assert_eq!(scenario.slug, "highwater_flag_transition_audit_02");
        assert_eq!(
            scenario.warp_to_level,
            Some(0x02),
            "the #238 transition probe must target Highwater Pass 1 regardless of the generic --level argument"
        );
        assert!(scenario.settle_oracle_warp);
        assert_eq!(
            scenario
                .start_position
                .map(|position| (position.x, position.y)),
            Some((912, 144)),
            "inspect-level reports Highwater Pass 1's flag at (912, 144)"
        );
        assert!(
            scenario
                .checkpoints
                .iter()
                .any(|checkpoint| checkpoint.label == "next_level_survival_tail"),
            "issue #238 needs evidence after the normal level-complete hand-off, not just flag touch"
        );
    }

    #[test]
    fn build_record_scenario_highwater_pass2_juggernaut_flag_transition_audit_starts_at_real_flag()
    {
        let scenario = build_record_scenario(
            RecordScenarioKind::HighwaterPass2JuggernautFlagTransitionAudit,
            0x00,
            None,
        );

        assert_eq!(
            scenario.slug,
            "highwater_pass2_juggernaut_flag_transition_audit_03"
        );
        assert_eq!(
            scenario.warp_to_level,
            Some(0x03),
            "the latest level-4 instant-death audit must target Highwater Pass 2 regardless of the generic --level argument"
        );
        assert!(scenario.settle_oracle_warp);
        assert_eq!(
            scenario
                .start_position
                .map(|position| (position.x, position.y)),
            Some((1_552, 1_040)),
            "inspect-level --level 03 reports Highwater Pass 2's flag at (1552, 1040)"
        );
        assert_eq!(
            scenario
                .camera_position
                .map(|position| (position.x, position.y)),
            Some((1_280, 896)),
            "the camera is clamped to native Level 0x03 max X before the transition"
        );
        assert_eq!(
            scenario.playtest_form,
            Some(5),
            "latest playtest reported the level 03 -> 04 death after carrying Rhino/Juggernaut"
        );
        assert!(
            scenario
                .checkpoints
                .iter()
                .any(|checkpoint| checkpoint.label == "next_level_survival_tail"),
            "the audit needs evidence after Level 0x04 becomes playable, not just flag touch"
        );
    }

    #[test]
    fn build_record_scenario_highwater_pass2_grow_shooter_touch_audit_starts_at_real_e919_cell() {
        let scenario = build_record_scenario(
            RecordScenarioKind::HighwaterPass2GrowShooterTouchAudit,
            0x00,
            None,
        );

        assert_eq!(scenario.slug, "highwater_pass2_grow_shooter_touch_audit_03");
        assert_eq!(
            scenario.warp_to_level,
            Some(0x03),
            "issue #250 needs the real Highwater Pass 2 level, not the generic --level argument"
        );
        assert!(scenario.settle_oracle_warp);
        assert_eq!(
            scenario
                .start_position
                .map(|position| (position.x, position.y)),
            Some((328, 143)),
            "level 0x03 decoded `$E919` touch-shooter cells start at grid col 20 row 9"
        );
        assert_eq!(
            scenario.playtest_form,
            Some(5),
            "latest playtest reported the grow/shooter confusion while carrying Rhino/Juggernaut"
        );
        assert!(
            scenario
                .checkpoints
                .iter()
                .any(|checkpoint| checkpoint.label == "worker_wake_window_sub_114f4"),
            "the audit must sample the delayed loc_11430/sub_114F4 worker window"
        );
    }

    #[test]
    fn build_record_scenario_issue249_wall_no_jump_audit_holds_right_without_jump() {
        let scenario =
            build_record_scenario(RecordScenarioKind::Issue249WallNoJumpAudit, 0x00, None);

        assert_eq!(scenario.slug, "issue_249_wall_no_jump_audit_02");
        assert_eq!(
            scenario.warp_to_level,
            Some(0x02),
            "issue #249 repro uses the known Highwater Pass 1 wall route"
        );
        assert!(scenario.settle_oracle_warp);
        assert_eq!(
            scenario.playtest_form, None,
            "base Kid isolates the loc_ABA4 jump-edge pull-up gate before helmet-specific variants"
        );
        const RIGHT: u8 = 1 << 3;
        const JUMP: u8 = 1 << 4;
        assert_eq!(scenario.script[60], RIGHT);
        assert!(
            !scenario.script.iter().any(|buttons| buttons & JUMP != 0),
            "the audit must reproduce wall contact without any jump edge"
        );
    }

    #[test]
    fn build_record_scenario_issue213_drips_metal_block_audit_preserves_direct_probe() {
        let scenario =
            build_record_scenario(RecordScenarioKind::Issue213DripsMetalBlockAudit, 0x00, None);

        assert_eq!(scenario.slug, "issue_213_drips_metal_block_audit_01");
        assert_eq!(
            scenario.warp_to_level,
            Some(0x01),
            "the named #155/#213 route targets Blue Lake Woods 2 regardless of the generic --level argument"
        );
        assert!(scenario.settle_oracle_warp);
        assert_eq!(
            scenario.start_position,
            Some(DynamicStartPosition { x: 568, y: 575 })
        );
        assert_eq!(
            scenario.camera_position,
            Some(DynamicStartPosition { x: 240, y: 352 })
        );
        assert_eq!(scenario.script[339], 0);
        assert_eq!(
            scenario.script[340] & (1 << 4),
            1 << 4,
            "B is held over the current oracle-clean jump window"
        );
        assert_eq!(scenario.script[376], 0);
        assert!(
            scenario
                .checkpoints
                .iter()
                .any(|checkpoint| checkpoint.label == "contact_window"
                    && checkpoint.frame_index == 460),
            "preserve the old Drips/RockTank contact sampling point while a natural route is built"
        );
    }

    #[test]
    fn build_record_scenario_issue249_juggernaut_wall_no_jump_audit_uses_rhino_form() {
        let scenario = build_record_scenario(
            RecordScenarioKind::Issue249JuggernautWallNoJumpAudit,
            0x00,
            None,
        );

        assert_eq!(scenario.slug, "issue_249_juggernaut_wall_no_jump_audit_02");
        assert_eq!(scenario.warp_to_level, Some(0x02));
        assert_eq!(
            scenario.playtest_form,
            Some(5),
            "the latest playtest mentioned Rhino/Juggernaut wall cheating"
        );
        const JUMP: u8 = 1 << 4;
        assert!(
            scenario.script.iter().all(|buttons| buttons & JUMP == 0),
            "the no-jump audit must not produce a B edge"
        );
    }

    #[test]
    fn build_record_scenario_issue249_juggernaut_wall_jump_audit_pulses_jump() {
        let scenario = build_record_scenario(
            RecordScenarioKind::Issue249JuggernautWallJumpAudit,
            0x00,
            None,
        );

        assert_eq!(scenario.slug, "issue_249_juggernaut_wall_jump_audit_02");
        assert_eq!(scenario.warp_to_level, Some(0x02));
        assert_eq!(scenario.playtest_form, Some(5));
        const RIGHT: u8 = 1 << 3;
        const JUMP: u8 = 1 << 4;
        assert_eq!(scenario.script[120] & (RIGHT | JUMP), RIGHT | JUMP);
        assert_eq!(
            scenario.script[122] & JUMP,
            0,
            "jump release creates the next rising edge"
        );
    }

    #[test]
    fn build_record_scenario_issue249_blw2_left_pit_wall_no_jump_targets_reported_pit() {
        let scenario = build_record_scenario(
            RecordScenarioKind::Issue249Blw2LeftPitWallNoJumpAudit,
            0x00,
            None,
        );

        assert_eq!(
            scenario.slug,
            "issue_249_blw2_left_pit_wall_no_jump_audit_01"
        );
        assert_eq!(
            scenario.warp_to_level,
            Some(0x01),
            "the reported rubber-block drop pit is in Blue Lake Woods 2, not the older Highwater route"
        );
        assert_eq!(
            scenario.start_position,
            Some(DynamicStartPosition { x: 352, y: 192 }),
            "block-grid scan pins the left pit wall at x=320, y=160..576; use a top-shaft seed to avoid the lower rubber/bridge bounce"
        );
        assert_eq!(
            scenario.camera_position,
            Some(DynamicStartPosition { x: 240, y: 80 })
        );
        assert_eq!(scenario.playtest_form, Some(5));
        const LEFT: u8 = 1 << 2;
        const JUMP: u8 = 1 << 4;
        assert_eq!(scenario.script[60], LEFT);
        assert!(
            scenario.script.iter().all(|buttons| buttons & JUMP == 0),
            "the no-jump audit must isolate any automatic wall launch"
        );
        assert!(
            scenario
                .checkpoints
                .iter()
                .any(|checkpoint| checkpoint.label == "left_pit_wall_hold_loc_aba4"),
            "ASM loc_ABA4 is the pull-up gate this scenario is meant to audit"
        );
    }

    #[test]
    fn issue249_blw2_left_pit_wall_no_jump_does_not_auto_launch_juggernaut() {
        let Some(manifest) = cached_generated_manifest() else {
            return;
        };
        let scenario = build_record_scenario(
            RecordScenarioKind::Issue249Blw2LeftPitWallNoJumpAudit,
            0x00,
            None,
        );

        let trace = run_native_dynamic_scenario(
            manifest,
            &scenario,
            NativeDynamicRunOptions {
                settled_warp: false,
                trace_physics: true,
            },
        )
        .expect("native BLW2 left-pit wall probe should run");

        assert_eq!(trace.len(), scenario.frame_count);
        let frame = |index: usize| {
            trace
                .get(index)
                .unwrap_or_else(|| panic!("missing native scenario frame {index}"))
        };
        let contact = frame(30);
        let hold = frame(90);
        let tail = frame(179);

        // ASM kid_flip -> loc_ABA4 gates the ledge pull-up to The Kid.  The
        // reported no-jump Rhino/Juggernaut wall contact must therefore stay
        // out of the ledge snap path unless a separate form-specific path is
        // found.
        for checkpoint in [contact, hold, tail] {
            assert_eq!(checkpoint.current_level, LevelId(0x01));
            assert_eq!(checkpoint.current_helmet_id, 5);
            assert_eq!(checkpoint.kid_y_velocity, Some(0));
            assert_eq!(checkpoint.ledge_snap_cooldown, 0);
            assert_eq!(checkpoint.kid_on_ground, Some(true));
        }
        assert_eq!(contact.kid_y_pos, Some(0x00A0_0000));
        assert_eq!(hold.kid_y_pos, contact.kid_y_pos);
        assert_eq!(tail.kid_y_pos, contact.kid_y_pos);
        let collision_probe = tail
            .gameplay_debug_fields
            .get("kid_collision_handoff_probe")
            .expect("tail collision handoff probe");
        assert!(
            collision_probe.contains("jump_buffer=0")
                && collision_probe.contains("wall_cling=none"),
            "no-jump BLW2 left-pit wall probe should not carry jump buffer or wall cling: {collision_probe}"
        );
    }

    #[test]
    fn build_record_scenario_issue249_blw2_left_pit_wall_jump_pulses_jump() {
        let scenario = build_record_scenario(
            RecordScenarioKind::Issue249Blw2LeftPitWallJumpAudit,
            0x00,
            None,
        );

        assert_eq!(scenario.slug, "issue_249_blw2_left_pit_wall_jump_audit_01");
        assert_eq!(scenario.warp_to_level, Some(0x01));
        assert_eq!(scenario.playtest_form, Some(5));
        const LEFT: u8 = 1 << 2;
        const JUMP: u8 = 1 << 4;
        assert_eq!(scenario.script[12] & (LEFT | JUMP), LEFT | JUMP);
        assert_eq!(
            scenario.script[14] & JUMP,
            0,
            "jump release creates the next rising edge"
        );
    }

    #[test]
    fn record_scenario_command_preserves_scripted_provenance() {
        let command = record_scenario_command(
            "issue-136-prize-emergence",
            RecordScenarioKind::PrizeBlock,
            0x00,
            Some(420),
        );

        assert_eq!(
            command,
            "cargo run --release -p kidcli -- record-scenario --slug issue-136-prize-emergence --scenario prize-block --level 00 --walk-frames 420"
        );
    }

    #[test]
    fn record_scenario_command_exposes_issue_216_prize_audio_audit() {
        let command = record_scenario_command(
            "issue-216-prize-audio-audit",
            RecordScenarioKind::Issue216PrizeAudioAudit,
            0x00,
            None,
        );

        assert_eq!(
            command,
            "cargo run --release -p kidcli -- record-scenario --slug issue-216-prize-audio-audit --scenario issue216-prize-audio-audit --level 00"
        );
    }

    #[test]
    fn record_scenario_command_exposes_issue213_drips_metal_block_audit() {
        let command = record_scenario_command(
            "issue-213-drips-metal-block-audit",
            RecordScenarioKind::Issue213DripsMetalBlockAudit,
            0x01,
            None,
        );

        assert_eq!(
            command,
            "cargo run --release -p kidcli -- record-scenario --slug issue-213-drips-metal-block-audit --scenario issue213-drips-metal-block-audit --level 01"
        );
    }

    #[test]
    fn record_scenario_command_exposes_form_walk_routes() {
        let command = record_scenario_command(
            "issue-211-samurai-natural-slope",
            RecordScenarioKind::WalkRightRedStealth,
            0x00,
            Some(700),
        );

        assert_eq!(
            command,
            "cargo run --release -p kidcli -- record-scenario --slug issue-211-samurai-natural-slope --scenario walk-right-red-stealth --level 00 --walk-frames 700"
        );
    }

    #[test]
    fn recording_start_for_dynamic_scenario_uses_power_on_for_tas_files() {
        let scenario = DynamicScenario {
            slug: "kc_any_percent_2024".into(),
            script: vec![0, 8, 8],
            frame_count: 3,
            checkpoints: Vec::new(),
            start_position: None,
            camera_position: None,
            playtest_form: None,
            initial_diamond_count: None,
            warp_to_level: None,
            settle_oracle_warp: false,
        };

        let start = recording_start_for_dynamic_scenario(&scenario);

        assert_eq!(start.warp_level, None);
        assert!(
            !start.settled,
            "power-on TAS scenarios must replay boot/title state, not a settled level shortcut"
        );
    }

    #[test]
    fn recording_start_for_dynamic_scenario_settles_warped_files() {
        let scenario = DynamicScenario {
            slug: "issue-143-heavy-man-block-break".into(),
            script: vec![0, 8, 8],
            frame_count: 3,
            checkpoints: Vec::new(),
            start_position: None,
            camera_position: None,
            playtest_form: None,
            initial_diamond_count: None,
            warp_to_level: Some(0x15),
            settle_oracle_warp: false,
        };

        let start = recording_start_for_dynamic_scenario(&scenario);

        assert_eq!(start.warp_level, Some(0x15));
        assert!(
            start.settled,
            "warp scenarios run after the level-load prelude and should start settled"
        );
    }

    #[test]
    fn recording_scenario_manifest_marks_settled_warp_for_standalone_oracle_replay() {
        let scenario = DynamicScenario {
            slug: "walk_right_00".into(),
            script: vec![0, 8, 8],
            frame_count: 3,
            checkpoints: Vec::new(),
            start_position: None,
            camera_position: None,
            playtest_form: None,
            initial_diamond_count: None,
            warp_to_level: Some(0x00),
            settle_oracle_warp: false,
        };
        let start = recording::RecordingStart {
            warp_level: Some(0x00),
            settled: true,
            record_frames: None,
        };

        let manifest_scenario = recording_scenario_manifest(scenario, start);

        assert!(
            manifest_scenario.settle_oracle_warp,
            "record-scenario exports scenario.json for direct verify-tas use, so warped settled recordings must carry their oracle bootstrap"
        );
    }

    #[test]
    fn record_scenario_file_command_preserves_json_provenance() {
        let command = record_scenario_file_command(
            "issue-143-tas-route",
            Path::new("tests/data/tas/kc_any_percent_2024/scenario.json"),
        );

        assert_eq!(
            command,
            "cargo run --release -p kidcli -- record-scenario-file --slug issue-143-tas-route --scenario tests/data/tas/kc_any_percent_2024/scenario.json"
        );
    }
}
