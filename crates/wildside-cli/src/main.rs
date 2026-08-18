#![allow(
    clippy::too_many_arguments,
    clippy::type_complexity,
    clippy::doc_lazy_continuation
)]
mod asm_coverage;
mod backlog;
mod bootstrap;
mod cache;
mod capture;
mod case;
mod cheats;
mod compare;
mod cpu_vectors;
mod diff;
mod gems_census;
mod gems_ring;
mod gems_span;
mod hypothesis;
mod inspect;
mod native_player;
mod oracle;
mod player;
mod playtest_ingest;
mod project_status;
mod ram_diff;
mod raster_span;
mod recording;
mod release_gate;
mod rust_source_context;
mod scenarios;
mod sweep;
mod tas;
mod tas_diff;
mod tas_gems;
mod tas_gems_baseline;
mod tas_gems_command_baseline;
mod tas_pixel_baseline;
mod tas_pixels;
mod verify;
mod view_tas;
mod visual_triage;

use verify::*;

use bootstrap::*;
use capture::*;
use case::{CaseCommand, run_case_command};
use compare::*;
use inspect::*;

use anyhow::{Context, Result, anyhow, bail};
use clap::{Parser, Subcommand, ValueEnum};
use image::{ColorType, ImageFormat};
use native_player::{
    NativePlayOptions, NativeScriptedRoute, PlaytestOptions, load_playtest_issues,
    playtest_session_slug, render_dynamic_gameplay_frame_evidence, render_gameplay_frame_evidence,
    run_native_player, unix_time_ms,
};
use oracle::{
    EnemyFacing, OracleEnemyState, OracleEnvironment, OracleManifest, OracleState, ensure_oracles,
    run_jgenesis_capture, run_jgenesis_capture_dynamic,
};
use player::{PlayOptions, run_player};
use project_status::{ProjectStatusFormat, run_project_status};
use recording::{
    RecordingRouteTarget, RecordingTargetCell, RecordingTargetPoint,
    resolve_recording_bundle_input, validate_recording_slug, verify_recording,
};
use release_gate::{ReleaseGateTarget, run_release_gate};
use scenarios::{
    DynamicCheckpoint, DynamicScenario, GameplayScenario, NativeDynamicRunOptions, Scenario,
    diamond_power_circle_audit_scenario, diamond_power_invulnerability_audit_scenario,
    diamond_power_samurai_haze_audit_scenario, diamond_power_snake_audit_scenario,
    diamond_power_wall_audit_scenario, enemy_spawn_audit_scenario,
    highwater_flag_floor_contact_audit_scenario, highwater_flag_transition_audit_scenario,
    highwater_pass2_grow_shooter_touch_audit_scenario,
    highwater_pass2_juggernaut_flag_transition_audit_scenario,
    highwater_shooter_barrier_audit_scenario, highwater_water_priority_audit_scenario,
    idle_level_scenario, issue_147_wall_side_jump_audit_scenario,
    issue_213_drips_delayed_b_drop_audit_scenario, issue_213_drips_metal_block_audit_scenario,
    issue_216_prize_audio_audit_scenario, issue_249_blw2_left_pit_wall_jump_audit_scenario,
    issue_249_blw2_left_pit_wall_kid_no_jump_audit_scenario,
    issue_249_blw2_left_pit_wall_no_jump_audit_scenario,
    issue_249_blw2_left_pit_wall_red_stealth_no_jump_audit_scenario,
    issue_249_juggernaut_wall_jump_audit_scenario,
    issue_249_juggernaut_wall_no_jump_audit_scenario, issue_249_wall_no_jump_audit_scenario,
    issue_260_dlb_tail_b_row32_bounce_scenario, issue_260_drips_b_jump_climb_seed_scenario,
    issue_260_drips_cleaner_prefix_audit_scenario, issue_260_drips_drop_contact_probe_scenario,
    issue_260_drips_early_jump_close_probe_scenario,
    issue_260_drips_iron_knight_climb_probe_scenario, issue_260_gr_pulse600_scenario,
    issue_271_level00_dragon_stomp_damage_audit_scenario,
    issue_271_level01_rock_tank_stomp_route_audit_scenario,
    issue_271_level01_rock_tank_stomp_sharp_audit_scenario, lava_weather_audit_scenario,
    level_00_slope_right_jump_probe_scenario, level_00_slope_right_probe_iron_knight_scenario,
    level_00_slope_right_probe_juggernaut_scenario,
    level_00_slope_right_probe_red_stealth_scenario, level_00_slope_right_probe_scenario,
    prize_block_scenario, run_native_dynamic_scenario, stomp_enemy_scenario,
    walk_right_iron_knight_scenario, walk_right_juggernaut_scenario,
    walk_right_red_stealth_scenario, walk_right_scenario,
};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use std::env;
use std::fmt::Write as _;
use std::fs;
use std::path::{Path, PathBuf};
use std::thread;
use std::time::{Duration, Instant};
use wildside_core::physics::Direction as NativeDirection;
use wildside_core::{
    ActiveProjectileKind, FrameInput, GameMode as NativeGameMode, LevelId, NativeRuntime,
    NativeTraceFrame, PlayerForm, run_script,
};
use wildside_data::{
    AssetManifest, find_enemy_definition, load_level_asset_bundle, pack_runtime_bundle_blob,
    resource_level_id,
};
use wildside_md::{
    GemsAudioEngine, GemsAudioWrite, GemsAudioWriteKind, MdRuntime, probe_rom,
    probe_rom_with_script, render_frame,
};
use wildside_rom::{RomIdentity, VerifiedRom, find_default_rom, known_sha1, verify_rom};
use ymfm_sys::Ym2612;

#[derive(Clone, Copy, Debug, ValueEnum)]
enum TargetProjectileKind {
    DripsDrop,
    DripsSurfaceCrawler,
    ShooterBlock,
}

impl From<TargetProjectileKind> for ActiveProjectileKind {
    fn from(value: TargetProjectileKind) -> Self {
        match value {
            TargetProjectileKind::DripsDrop => ActiveProjectileKind::DripsDrop,
            TargetProjectileKind::DripsSurfaceCrawler => ActiveProjectileKind::DripsSurfaceCrawler,
            TargetProjectileKind::ShooterBlock => ActiveProjectileKind::ShooterBlock,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, ValueEnum)]
enum TargetProjectileDirection {
    Left,
    Right,
    Up,
    Down,
    Stationary,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, ValueEnum)]
enum TargetBlockEventKind {
    Bounce,
    Destroy,
}

#[derive(Debug, Parser)]
#[command(name = "wildside")]
#[command(
    about = "Contained tooling for the Wildside workspace (a Kid Chameleon reimplementation)"
)]
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
    /// start on Blue Lake Woods 1's first `$4000` slope, hold Right, then jump.
    Level00SlopeRightJumpProbe,
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
    /// walk into Highwater Pass 1's flag from floor level (#267).
    HighwaterFlagFloorContactAudit,
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
    /// cleaner Level 01 Drips prefix with two crawler sightings and live tail (#213/#260).
    Issue260DripsCleanerPrefixAudit,
    /// cleaner Level 01 Drips prefix extended by a live B-jump climb seed (#260).
    Issue260DripsBJumpClimbSeed,
    /// Iron Knight variant of the cleaner Level 01 Drips prefix for wall-climb route search (#260).
    Issue260DripsIronKnightClimbProbe,
    /// close early-jump Level 01 Drips route that stops before the death tail (#260).
    Issue260DripsEarlyJumpCloseProbe,
    /// refreshed Level 01 Drips tail route that makes native overlap the drop (#260).
    Issue260DripsDropContactProbe,
    /// oracle-comparable row-32 metal-block bounce near-miss (#260).
    Issue260DlbTailBRow32Bounce,
    /// oracle-comparable late DripsDrop/row-32 near-miss (#260).
    Issue260GrPulse600,
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
    /// drive The Kid into Blue Lake Woods 2's left pit wall without jump (#249/#270).
    Issue249Blw2LeftPitWallKidNoJumpAudit,
    /// drive Red Stealth/Samurai into Blue Lake Woods 2's left pit wall without jump (#249/#270).
    Issue249Blw2LeftPitWallRedStealthNoJumpAudit,
    /// preserve the current Level 00 Dragon side-graze damage route before sharpening #271.
    Issue271Level00DragonStompDamageAudit,
    /// preserve the current Level 01 RockTank stomp route miss before sharpening #271.
    Issue271Level01RockTankStompRouteAudit,
    /// sharpened Level 01 RockTank route that reaches the stomp classifier (#271).
    Issue271Level01RockTankStompSharpAudit,
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
        #[arg(long, value_parser = parse_level_id)]
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
        /// Overlay input ranges, e.g. `60-600:right+c` or `120:b`.
        #[arg(long)]
        input: Vec<String>,
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
        #[arg(long, value_parser = parse_level_id)]
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
        /// Summarise closest healthy Kid approach to a projectile moving in a direction, formatted kind:direction.
        #[arg(long, value_parser = parse_batch_target_projectile_direction)]
        target_projectile_direction: Vec<BatchTargetProjectileDirection>,
        /// Summarise live player movement bounds and first transition frame.
        #[arg(long)]
        player_motion: bool,
        /// Summarise closest live Kid approach to an x,y pixel target.
        #[arg(long, value_parser = parse_target_point)]
        target_point: Vec<TargetPoint>,
        /// Summarise closest live Kid approach to a 16x16 level cell, formatted col,row.
        #[arg(long, value_parser = parse_target_cell)]
        target_cell: Vec<TargetCell>,
        /// Summarise closest live Kid hitbox approach to a 16x16 level cell.
        #[arg(long, value_parser = parse_target_cell)]
        target_cell_hitbox: Vec<TargetCell>,
        /// Summarise closest projectile approach to a 16x16 level cell, formatted kind:col,row.
        #[arg(long, value_parser = parse_batch_target_projectile_cell)]
        target_projectile_cell: Vec<BatchTargetProjectileCell>,
        /// Summarise same-frame healthy Kid hitbox and projectile approach to a 16x16 level cell, formatted kind:col,row.
        #[arg(long, value_parser = parse_batch_target_projectile_cell)]
        target_projectile_cell_conjunction: Vec<BatchTargetProjectileCell>,
        /// Summarise exact block bounce/destroy events at a 16x16 level cell, formatted kind:col,row.
        #[arg(long, value_parser = parse_batch_target_block_event)]
        target_block_event: Vec<BatchTargetBlockEvent>,
        /// Summarise best native Kid contact classification against one enemy object type.
        #[arg(long, value_parser = parse_level_id)]
        target_enemy_contact: Option<u8>,
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
        /// Summarise closest healthy Kid approach to a projectile moving in a direction, formatted kind:direction.
        #[arg(long, value_parser = parse_batch_target_projectile_direction)]
        target_projectile_direction: Vec<BatchTargetProjectileDirection>,
        /// Summarise live player movement bounds and first transition frame.
        #[arg(long)]
        player_motion: bool,
        /// Summarise closest live Kid approach to an x,y pixel target.
        #[arg(long, value_parser = parse_target_point)]
        target_point: Vec<TargetPoint>,
        /// Summarise closest live Kid approach to a 16x16 level cell, formatted col,row.
        #[arg(long, value_parser = parse_target_cell)]
        target_cell: Vec<TargetCell>,
        /// Summarise closest live Kid hitbox approach to a 16x16 level cell.
        #[arg(long, value_parser = parse_target_cell)]
        target_cell_hitbox: Vec<TargetCell>,
        /// Summarise closest projectile approach to a 16x16 level cell, formatted kind:col,row.
        #[arg(long, value_parser = parse_batch_target_projectile_cell)]
        target_projectile_cell: Vec<BatchTargetProjectileCell>,
        /// Summarise same-frame healthy Kid hitbox and projectile approach to a 16x16 level cell, formatted kind:col,row.
        #[arg(long, value_parser = parse_batch_target_projectile_cell)]
        target_projectile_cell_conjunction: Vec<BatchTargetProjectileCell>,
        /// Summarise exact block bounce/destroy events at a 16x16 level cell, formatted kind:col,row.
        #[arg(long, value_parser = parse_batch_target_block_event)]
        target_block_event: Vec<BatchTargetBlockEvent>,
        /// Summarise best native Kid contact classification against one enemy object type.
        #[arg(long, value_parser = parse_level_id)]
        target_enemy_contact: Option<u8>,
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
    /// Probe many DynamicScenario input variants in one native process.
    ProbeRecordScenarioFileBatch {
        /// Path to the base DynamicScenario JSON to replay.
        #[arg(long)]
        scenario: PathBuf,
        /// Candidate overlay, formatted `label=START-END:buttons,START-END:buttons`.
        #[arg(long)]
        candidate: Vec<String>,
        /// Generate candidate overlays from a moving input window, formatted `LABEL=START-END:DURATION:STEP:BUTTONS`.
        #[arg(long, value_parser = parse_batch_probe_window)]
        sweep_window: Vec<BatchProbeWindow>,
        /// Refuse to run if generated sweep windows expand beyond this count.
        #[arg(long, default_value_t = 5_000)]
        max_generated: usize,
        /// Score candidates by closest healthy Kid approach to these x,y pixel targets.
        #[arg(long, value_parser = parse_target_point)]
        target_point: Vec<TargetPoint>,
        /// Score candidates by closest healthy Kid approach to these 16x16 level cells.
        #[arg(long, value_parser = parse_target_cell)]
        target_cell: Vec<TargetCell>,
        /// Score candidates by closest healthy Kid hitbox approach to these 16x16 level cells.
        #[arg(long, value_parser = parse_target_cell)]
        target_cell_hitbox: Vec<TargetCell>,
        /// Score candidates by closest healthy Kid approach to typed projectiles.
        #[arg(long)]
        target_projectile: Vec<TargetProjectileKind>,
        /// Score candidates by closest healthy Kid approach to a projectile moving in a direction, formatted kind:direction.
        #[arg(long, value_parser = parse_batch_target_projectile_direction)]
        target_projectile_direction: Vec<BatchTargetProjectileDirection>,
        /// Score candidates by closest projectile approach to a 16x16 level cell, formatted kind:col,row.
        #[arg(long, value_parser = parse_batch_target_projectile_cell)]
        target_projectile_cell: Vec<BatchTargetProjectileCell>,
        /// Score candidates by same-frame healthy Kid hitbox and projectile approach to a 16x16 level cell, formatted kind:col,row.
        #[arg(long, value_parser = parse_batch_target_projectile_cell)]
        target_projectile_cell_conjunction: Vec<BatchTargetProjectileCell>,
        /// Score candidates by exact block event at a 16x16 level cell, formatted kind:col,row.
        #[arg(long, value_parser = parse_batch_target_block_event)]
        target_block_event: Vec<BatchTargetBlockEvent>,
        /// Score candidates by best native Kid contact classification against enemy object types.
        #[arg(long, value_parser = parse_level_id)]
        target_enemy_contact: Vec<u8>,
        /// Ignore earlier frames when scoring closest target approach.
        #[arg(long, default_value_t = 0)]
        score_frame_start: usize,
        /// Print at most this many ranked candidates.
        #[arg(long, default_value_t = 20)]
        limit: usize,
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
        /// Stop, save, and exit after first stomp/damage contact with a record-target enemy.
        #[arg(long, default_value_t = false)]
        record_stop_on_target_enemy_contact: bool,
        /// Frames to keep after --record-stop-on-target-enemy-contact triggers.
        #[arg(long, default_value_t = 30)]
        record_contact_tail_frames: usize,
        #[arg(long)]
        record_notes: Option<PathBuf>,
        /// Stable label for the route target in the saved recording bundle.
        #[arg(long)]
        record_target_label: Option<String>,
        /// Target level for the route being recorded.
        #[arg(long, value_parser = parse_level_id)]
        record_target_level: Option<u8>,
        /// Route target point in level pixels, formatted x,y.
        #[arg(long, value_parser = parse_target_point)]
        record_target_point: Vec<TargetPoint>,
        /// Route target 16x16 level cell, formatted col,row.
        #[arg(long, value_parser = parse_target_cell)]
        record_target_cell: Vec<TargetCell>,
        /// Route target 16x16 level cell for Kid-hitbox approach checks.
        #[arg(long, value_parser = parse_target_cell)]
        record_target_cell_hitbox: Vec<TargetCell>,
        /// Route target enemy object type.
        #[arg(long, value_parser = parse_level_id)]
        record_target_enemy: Vec<u8>,
        /// Frontier class to use in the generated TAS bisect command.
        #[arg(long, value_enum, default_value_t = sweep::FrontierKind::Enemy)]
        record_frontier_kind: sweep::FrontierKind,
        /// Replay a DynamicScenario JSON through the live SDL player loop.
        #[arg(long)]
        scenario: Option<PathBuf>,
        /// Playtest mode: F1 issue browser fed from the awaiting-human lane,
        /// verdict hotkeys, and segmented session recording under
        /// .cache/playtest/. Mutually exclusive with --record and --scenario.
        #[arg(long, default_value_t = false)]
        playtest: bool,
        /// Record session-long mic narration to <session>/mic.wav (requires
        /// --playtest). Default command: ffmpeg avfoundation, mono 16 kHz.
        #[arg(long, default_value_t = false)]
        mic: bool,
        /// Custom mic capture command; `{wav}` is replaced with the WAV path.
        #[arg(long)]
        mic_command: Option<String>,
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
    /// Enumerate GEMS SFX ids and measure standalone native driver output.
    AudioAudit {
        #[arg(long)]
        rom: Option<PathBuf>,
        #[arg(long)]
        dir: Option<PathBuf>,
        /// Audit one sound id, e.g. 0x41. Defaults to every sfx_* constant.
        #[arg(long, value_parser = parse_level_id)]
        sound_id: Option<u8>,
        /// Frames to render after starting each sequence.
        #[arg(long, default_value_t = 180)]
        frames: usize,
        /// Host sample rate for generated waveform metrics.
        #[arg(long, default_value_t = 44_100)]
        sample_rate: u32,
        /// Output JSON instead of the compact table.
        #[arg(long, default_value_t = false)]
        json: bool,
    },
    VerifyNativeAudioOutputFile {
        #[arg(long)]
        rom: Option<PathBuf>,
        #[arg(long)]
        dir: Option<PathBuf>,
        /// Path to the DynamicScenario JSON to compare against the oracle.
        #[arg(long)]
        scenario: PathBuf,
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
    /// Shallow-clone the two SingleStepTests 68000 vector repositories into
    /// .cache/references (#447). ~600 MB, so deliberately not part of
    /// bootstrap-references; a re-run at the pinned SHA costs no network.
    BootstrapCpuVectors,
    /// Validate the pinned r68k-emu 68000 core against the SingleStepTests
    /// vectors (#447). Primary verdict is final state (registers, stack
    /// pointers, PC, SR, memory); cycle counts and the data-bus sequence are
    /// reported as a secondary, informational tier.
    VerifyCpuVectors {
        /// Restrict to one published set (default: both).
        #[arg(long, value_enum)]
        set: Option<cpu_vectors::vector::CpuVectorSet>,
        /// Keep only the first N vectors of each opcode file. A prefix, not a
        /// random sample, so a capped run stays reproducible.
        #[arg(long)]
        per_file_limit: Option<usize>,
        /// Print the machine-readable report to stdout instead of the summary.
        #[arg(long, default_value_t = false)]
        json: bool,
    },
    OracleSetup,
    ExtractAssets {
        #[arg(long)]
        rom: Option<PathBuf>,
        #[arg(long)]
        dir: Option<PathBuf>,
    },
    /// Pack the extracted generated bundle into a single blob the web build
    /// fetches at startup (#19/#21). Ships decoded assets only — private
    /// (Tailscale/local) delivery per the decision on #16.
    PackWebBundle {
        #[arg(long)]
        rom: Option<PathBuf>,
        #[arg(long)]
        dir: Option<PathBuf>,
        #[arg(long, default_value = "web/bundle.wsbndl")]
        output: PathBuf,
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
    CompareCachedNativeGameplay {
        #[arg(long, value_enum, default_value_t = GameplayScenario::PauseResumeLevel1)]
        scenario: GameplayScenario,
        #[arg(long)]
        run_slug: String,
    },
    /// Emit a draft markdown report for a playtest session (verdicts,
    /// bookmarks, aligned mic narration). Never edits the backlog.
    PlaytestIngest {
        /// Session slug (under .cache/playtest/) or a session directory path.
        #[arg(long)]
        session: String,
        /// Pre-made transcript export (SRT, VTT, or whisper JSON).
        #[arg(long)]
        transcript: Option<PathBuf>,
        /// Local transcription command printing SRT/VTT/JSON to stdout;
        /// `{wav}` is replaced with the session's mic.wav path.
        #[arg(long)]
        whisper_cmd: Option<String>,
        /// Report path (default: <session>/report.md).
        #[arg(long)]
        out: Option<PathBuf>,
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
        /// Also compare RNG offset/ring at each checkpoint (#442). Opt-in:
        /// RNG diffs are blocking, so enabling this by default would move the
        /// 1p-accurate gate.
        #[arg(long, default_value_t = false)]
        include_rng: bool,
        /// Capture golden state snapshots (no oracle needed).
        #[arg(long, default_value_t = false)]
        golden_capture: bool,
        /// Diff current state against golden snapshots (no oracle needed).
        #[arg(long, default_value_t = false)]
        golden_diff: bool,
    },
    /// Classify sweep-blocking differences by enemy family: join the blocking
    /// (authoritative/accumulation/rng) state diffs of a sweep report onto
    /// per-level enemy rosters, rank families by overrepresentation in
    /// failing levels, and write the census to .cache/audit/sweep-census.json.
    SweepCensus {
        #[arg(long)]
        rom: Option<PathBuf>,
        #[arg(long)]
        dir: Option<PathBuf>,
        /// Sweep run id under .cache/sweep-runs (default: the latest symlink,
        /// else the highest numeric run with a report.json)
        #[arg(long)]
        run: Option<String>,
        /// Output JSON path (default: .cache/audit/sweep-census.json)
        #[arg(long)]
        output: Option<PathBuf>,
        /// Skip the ROM/asset roster join (field histogram only, no ROM needed)
        #[arg(long, default_value_t = false)]
        no_roster: bool,
    },
    /// Measure the oracle helper's hidden settled-warp shell per level (#442):
    /// re-run the settled-warp bootstrap with the settle trace captured and
    /// derive how many emulator frames run between the helper's first
    /// target-level poll and scenario frame 0, plus a frame-0 RNG/state
    /// witness. Cache-bypassing and helper-edit-free, so the oracle cache is
    /// untouched. Writes .cache/audit/oracle-settle-census.json.
    OracleSettleCensus {
        #[arg(long)]
        rom: Option<PathBuf>,
        /// Level range: "0-104", "0,5,10", or "0-10"
        #[arg(long, default_value = "0-104")]
        levels: String,
        /// Output JSON path (default: .cache/audit/oracle-settle-census.json)
        #[arg(long)]
        output: Option<PathBuf>,
        /// Skip the oracle runs and reload a previous census JSON artefact
        /// (pair with --shell-table to reformat it as the checked-in table)
        #[arg(long)]
        from_json: Option<PathBuf>,
        /// Also write the checked-in YAML shell table (provenance header +
        /// staleness-guarded schema) to this path
        #[arg(long)]
        shell_table: Option<PathBuf>,
        /// Persist each measured level's full settle rows (RNG ring, kid
        /// kinematics, object-slot dump per step) as settle-trace.json in the
        /// level's run dir — the #442 shell-content evidence
        #[arg(long)]
        dump_trace: bool,
    },
    /// Cluster the classified GEMS outcome corpus from
    /// `verify-tas --compare-gems --gems-mismatch-report` by classification,
    /// diverging-field shape, and bank pair, and write the census to
    /// .cache/audit/gems-census.json (#434).
    GemsCensus {
        /// Corpus report path (default:
        /// .cache/tas-verify/any-percent-published/gems-mismatch-report.json)
        #[arg(long)]
        report: Option<PathBuf>,
        /// Output JSON path (default: .cache/audit/gems-census.json)
        #[arg(long)]
        output: Option<PathBuf>,
    },
    /// Decode the 68k→Z80 GEMS mailbox-ring command stream from an
    /// oracle-probe --capture-gems-state manifest and print the decoded
    /// events plus an opcode census (#435). Fails when any frame decodes
    /// uncleanly (unknown opcode or ring overflow).
    GemsRingDecode {
        /// Oracle manifest with per-frame audio-RAM dumps (default:
        /// .cache/oracle-probe/oracle/manifest.json)
        #[arg(long)]
        manifest: Option<PathBuf>,
        /// Optional JSON dump of the decoded per-frame events
        #[arg(long)]
        output: Option<PathBuf>,
    },
    /// Replay a captured ORACLE 68k→Z80 stimulus span into the native GEMS
    /// timing engine, seeded at a ROM reinit anchor, and compare
    /// timing-emergent signature fields per frame (#437). Produces a
    /// measured-debt report — deliberately NOT a CI gate.
    GemsSpanReplay {
        #[arg(long)]
        rom: Option<PathBuf>,
        /// Oracle manifest with gems_stimuli + pc-trace anchor checkpoint +
        /// per-frame signatures (default: .cache/oracle-probe/oracle/manifest.json)
        #[arg(long)]
        manifest: Option<PathBuf>,
        /// Frame of the ROM reinit whose $E14C8 pc-trace checkpoint seeds
        /// the replay phases
        #[arg(long)]
        anchor_frame: usize,
        /// First compared frame (must be after the anchor frame)
        #[arg(long)]
        frame_start: usize,
        /// Last compared frame (inclusive)
        #[arg(long)]
        frame_end: usize,
        /// Sensitivity probe: add this many 68k cycles to every stimulus
        /// stamp (captured stamps are instruction-OPENING; jgenesis lands
        /// pending writes at instruction close, typically ~16 cycles later)
        #[arg(long, default_value_t = 0)]
        stamp_skew_cycles: u64,
        /// Report JSON (default: .cache/audit/gems-span-replay-<start>-<end>.json)
        #[arg(long)]
        output: Option<PathBuf>,
    },
    /// Record the frozen GEMS span's engine stimuli, replay them into a
    /// fresh engine, and demand bit-identical per-frame checkpoints (#436).
    /// GO proves the engine's public-method seam is its complete input
    /// surface; NO-GO names the first diverging frame and fields.
    GemsReplayProbe {
        #[arg(long)]
        rom: Option<PathBuf>,
        #[arg(long)]
        dir: Option<PathBuf>,
        /// TAS scenario driving the span (default:
        /// tests/data/tas/any-percent-published/scenario.json)
        #[arg(long)]
        scenario: Option<PathBuf>,
        /// Probe report JSON (default: .cache/audit/gems-replay-probe.json)
        #[arg(long)]
        output: Option<PathBuf>,
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
    /// Enforce backlog hygiene: NOW size, lean open-queue Verify cells, lane consistency.
    BacklogLint,
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
        /// Render through the native scene compositor instead of raw VDP capture.
        #[arg(long)]
        scene: bool,
    },
    /// Render ORACLE ROM frames while replaying a DynamicScenario.
    ///
    /// Complements `capture-scenario-frames`, which renders the native engine.
    /// Unlike `render-frame`, this honours DynamicScenario warp/settle/probe
    /// fields and is suitable for manual title/menu or settled gameplay routes.
    CaptureOracleScenarioFrames {
        #[arg(long)]
        rom: Option<PathBuf>,
        /// Path to the DynamicScenario JSON (e.g. a recording's scenario.json)
        #[arg(long)]
        scenario: PathBuf,
        /// Overlay input ranges, e.g. `60-600:right+c` or `120:b`.
        #[arg(long)]
        input: Vec<String>,
        /// Comma-separated frame indices to capture
        #[arg(long, default_value = "30,60,120,240,360")]
        frames: String,
        /// Output directory for PNGs and oracle manifest
        #[arg(long, default_value = ".cache/captures-oracle")]
        output: PathBuf,
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
        /// Compare the complete initialised ROM RNG ring and offset at checkpoints
        #[arg(long, default_value_t = false)]
        include_rng: bool,
        /// Render and compare native/oracle RGB pixels at every requested checkpoint
        #[arg(long, default_value_t = false)]
        compare_pixels: bool,
        /// Compare per-frame GEMS/Z80 timing signatures across the whole run
        /// (`gems_frame_signature_v1`)
        #[arg(long, default_value_t = false)]
        compare_gems: bool,
        /// Compare the decoded 68k→Z80 GEMS command streams per frame
        /// (`gems_command_parity_v1`; requires --compare-gems)
        #[arg(long, default_value_t = false)]
        compare_gems_commands: bool,
        /// Write a standalone per-frame pixel-mismatch + cluster report JSON to
        /// this path (requires --compare-pixels; not supported with --bisect)
        #[arg(long)]
        pixel_mismatch_report: Option<PathBuf>,
        /// Write the full classified GEMS outcome corpus (genuine /
        /// oracle-limitation / advisory records) JSON to this path
        /// (requires --compare-gems; not supported with --bisect)
        #[arg(long)]
        gems_mismatch_report: Option<PathBuf>,
    },
    /// Cluster a TAS pixel-mismatch corpus by root cause: replay the route,
    /// sample per-pixel draw provenance at each divergent frame, and rank
    /// signatures (divergence class + layer + command kind + palette line).
    VisualTriage {
        #[arg(long)]
        rom: Option<PathBuf>,
        #[arg(long)]
        dir: Option<PathBuf>,
        /// Path to the DynamicScenario JSON the mismatch report was built
        /// from (e.g. .cache/tas-verify/<slug>/scenario.json)
        #[arg(long)]
        scenario: PathBuf,
        /// Pixel mismatch report from verify-tas --pixel-mismatch-report
        /// (default: .cache/tas-verify/<slug>/mismatch-report.json)
        #[arg(long)]
        mismatch_report: Option<PathBuf>,
        /// Only triage divergent frames from this frame onwards
        #[arg(long)]
        frame_start: Option<usize>,
        /// Only triage divergent frames before this frame
        #[arg(long)]
        frame_end: Option<usize>,
        /// Clusters to print and write diff composites for
        #[arg(long, default_value_t = 20)]
        top: usize,
        /// Regenerate the cached oracle PNG captures before triaging
        #[arg(long, default_value_t = false)]
        rerun_oracle: bool,
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
        /// Print compact native/oracle RNG offsets and skip verbose diffs
        #[arg(long, default_value_t = false)]
        rng_trace: bool,
        /// Print compact Drips controller/crawler state and skip verbose diffs
        #[arg(long, default_value_t = false)]
        drips_trace: bool,
        /// Print compact Kid/player state and skip verbose diffs
        #[arg(long, default_value_t = false)]
        player_trace: bool,
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
        /// Frames for oracle VDP FIFO enqueue and commit records with scanline/pixel timing.
        #[arg(long)]
        vdp_fifo_commit_frames: Option<String>,
        /// Inclusive VDP FIFO address range to retain (e.g. 0xDD40-0xDF3F).
        #[arg(long)]
        vdp_fifo_commit_address_range: Option<String>,
        /// Print only count and first/last records for each filtered FIFO frame.
        #[arg(long, default_value_t = false)]
        vdp_fifo_summary: bool,
        /// Hexadecimal M68K PCs or even inclusive ranges to trace before execution
        /// (e.g. 0x74A,0xE12,0x44DC or 0xE12-0xE58).
        #[arg(long)]
        m68k_pc_trace: Option<String>,
        /// Frames whose 68k→Z80 GEMS stimulus rows (RAM writes under BUSACK,
        /// BUSREQ/RESET bits, m68k-cycle stamped) are retained (#437 spans).
        #[arg(long)]
        gems_stimulus_frames: Option<String>,
        /// Attach GEMS audio-RAM and timing checkpoints to selected M68K PC rows.
        #[arg(long, default_value_t = false)]
        capture_gems_state: bool,
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
        /// Skip per-frame PNG/state checkpoint captures; keep trace/FIFO records.
        /// Long trace-only probes (e.g. the HUD timing census) otherwise write
        /// tens of thousands of PNG and state files they never read.
        #[arg(long, default_value_t = false)]
        no_frame_captures: bool,
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
    /// Replay the bounded HUD CPU/raster census as measurement evidence
    /// (#409, ADR 0003). Never a CI gate; shares the engine's H40 divider.
    RasterSpanReplay {
        /// HUD timing census JSON (from `mise run hud-timing-census`)
        #[arg(long)]
        census: PathBuf,
        /// HUD cluster to replay; repeat to select both (default: both)
        #[arg(long)]
        cluster: Vec<String>,
        /// Report JSON (default: stdout)
        #[arg(long)]
        output: Option<PathBuf>,
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
        /// Regenerate .cache/asm-coverage.yaml (untracked; preserves hand annotations).
        #[arg(long, default_value_t = false)]
        update: bool,
        /// Filter to subsystems matching this substring (e.g. "enemy").
        #[arg(long)]
        subsystem: Option<String>,
        /// List ordinary uncovered labels and uncited emulated GEMS sub_ labels.
        #[arg(long, default_value_t = false)]
        uncovered: bool,
        /// Machine-readable JSON output.
        #[arg(long, default_value_t = false)]
        json: bool,
        /// Fail (exit non-zero) if any label in REQUIRED_LABELS lacks a
        /// non-test core/data source citation. verify-slice-fast uses this to
        /// prevent regressions on attack/animation/slope routines.
        #[arg(long, default_value_t = false)]
        check_required: bool,
        /// Fail (exit non-zero) if any ASM label newly becomes test-only debt
        /// outside the grandfathered baseline.
        #[arg(long, default_value_t = false)]
        check_no_new_test_only: bool,
    },
    /// Ratchet-aware pixel-fidelity gate: compare a verify-tas pixel report
    /// against the committed frontier baseline. Exit 0 on match, 1 on
    /// regression (or coverage failure), 3 on improvement (baseline stale;
    /// re-run with --update after review).
    TasPixelBaseline {
        /// Path to the verify-tas report.json (must include pixel_comparison)
        #[arg(long)]
        report: PathBuf,
        /// Path to the frontier baseline file
        #[arg(long, default_value = "tests/tas_frontier.json")]
        frontier: PathBuf,
        /// Recording slug whose full_expected_pixel_comparison block to check
        #[arg(long)]
        slug: String,
        /// Splice the report's pixel comparison into the frontier baseline
        /// (surgical: only that block changes; byte-identical writes skipped)
        #[arg(long, default_value_t = false)]
        update: bool,
        /// Allow --update to accept a REGRESSED report (deliberate,
        /// reviewed baseline lowering; refused otherwise)
        #[arg(long, default_value_t = false)]
        force: bool,
    },
    /// Ratchet-aware GEMS/Z80 audio-timing gate: compare a verify-tas
    /// --compare-gems report against the committed frontier baseline. Exit 0
    /// on match, 1 on regression (or coverage failure), 3 on improvement
    /// (baseline stale; re-run with --update after review).
    TasGemsBaseline {
        /// Path to the verify-tas report.json (must include gems_frame_comparison)
        #[arg(long)]
        report: PathBuf,
        /// Path to the frontier baseline file
        #[arg(long, default_value = "tests/tas_frontier.json")]
        frontier: PathBuf,
        /// Recording slug whose full_expected_gems_frame_comparison block to check
        #[arg(long)]
        slug: String,
        /// Splice the report's GEMS comparison into the frontier baseline
        /// (surgical: only that block changes; byte-identical writes skipped)
        #[arg(long, default_value_t = false)]
        update: bool,
        /// Allow --update to accept a REGRESSED report (deliberate,
        /// reviewed baseline lowering; refused otherwise)
        #[arg(long, default_value_t = false)]
        force: bool,
    },
    /// Ratchet gate for GEMS command parity (`gems_command_parity_v1`, the
    /// authoritative audio gate per #416/#435): compare a verify-tas
    /// --compare-gems-commands report against the committed frontier
    /// baseline. Exit 0 on match, 1 on regression (or coverage failure), 3
    /// on improvement (baseline stale; re-run with --update after review).
    TasGemsCommandBaseline {
        /// Path to the verify-tas report.json (must include gems_command_comparison)
        #[arg(long)]
        report: PathBuf,
        /// Path to the frontier baseline file
        #[arg(long, default_value = "tests/tas_frontier.json")]
        frontier: PathBuf,
        /// Recording slug whose full_expected_gems_command_comparison block to check
        #[arg(long)]
        slug: String,
        /// Splice the report's command comparison into the frontier baseline
        /// (surgical: only that block changes; byte-identical writes skipped)
        #[arg(long, default_value_t = false)]
        update: bool,
        /// Allow --update to accept a REGRESSED report (deliberate,
        /// reviewed baseline lowering; refused otherwise)
        #[arg(long, default_value_t = false)]
        force: bool,
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
            input,
            rom,
            dir,
        } => {
            let repo_root = current_dir()?;
            let rom_path = resolve_rom_path(&repo_root, rom)?;
            let upstream_dir = resolve_upstream_dir(&repo_root, dir);
            let manifest = prepare_generated_assets(&repo_root, &rom_path, &upstream_dir)?;
            let dynamic = load_record_scenario_file(&scenario)?;
            let dynamic = apply_dynamic_scenario_input_overlays(dynamic, &input)?;
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
            recording.manifest.record_command =
                record_scenario_file_command(&slug, &scenario, &input);
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
            target_projectile_direction,
            player_motion,
            target_point,
            target_cell,
            target_cell_hitbox,
            target_projectile_cell,
            target_projectile_cell_conjunction,
            target_block_event,
            target_enemy_contact,
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
                None,
                trace_physics,
                debug_fields,
                target_enemy,
                target_projectile.map(Into::into),
                &target_projectile_direction,
                player_motion,
                &target_point,
                &target_cell,
                &target_cell_hitbox,
                &projectile_cell_targets(&target_projectile_cell),
                &projectile_cell_targets(&target_projectile_cell_conjunction),
                &target_block_event,
                target_enemy_contact,
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
            target_projectile_direction,
            player_motion,
            target_point,
            target_cell,
            target_cell_hitbox,
            target_projectile_cell,
            target_projectile_cell_conjunction,
            target_block_event,
            target_enemy_contact,
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
                Some(&scenario),
                trace_physics,
                debug_fields,
                target_enemy,
                target_projectile.map(Into::into),
                &target_projectile_direction,
                player_motion,
                &target_point,
                &target_cell,
                &target_cell_hitbox,
                &projectile_cell_targets(&target_projectile_cell),
                &projectile_cell_targets(&target_projectile_cell_conjunction),
                &target_block_event,
                target_enemy_contact,
                summary_only,
                frame_start,
                frame_end,
            )?;
        }
        Commands::ProbeRecordScenarioFileBatch {
            scenario,
            candidate,
            sweep_window,
            max_generated,
            target_point,
            target_cell,
            target_cell_hitbox,
            target_projectile,
            target_projectile_direction,
            target_projectile_cell,
            target_projectile_cell_conjunction,
            target_block_event,
            target_enemy_contact,
            score_frame_start,
            limit,
            rom,
            dir,
        } => {
            let repo_root = current_dir()?;
            let rom_path = resolve_rom_path(&repo_root, rom)?;
            let upstream_dir = resolve_upstream_dir(&repo_root, dir);
            let manifest = prepare_generated_assets(&repo_root, &rom_path, &upstream_dir)?;
            let base = load_record_scenario_file(&scenario)?;
            if target_point.is_empty()
                && target_cell.is_empty()
                && target_cell_hitbox.is_empty()
                && target_projectile.is_empty()
                && target_projectile_direction.is_empty()
                && target_projectile_cell.is_empty()
                && target_projectile_cell_conjunction.is_empty()
                && target_block_event.is_empty()
                && target_enemy_contact.is_empty()
            {
                bail!(
                    "at least one --target-point, --target-cell, --target-cell-hitbox, --target-projectile, --target-projectile-cell, --target-projectile-cell-conjunction, --target-block-event, or --target-enemy-contact is required for batch scoring"
                );
            }
            if let Some(level) = base.warp_to_level {
                ensure_level_layouts(&rom_path, &manifest, &[level])?;
            }
            probe_dynamic_record_scenario_batch(
                &manifest,
                &base,
                &candidate,
                &sweep_window,
                max_generated,
                &target_point,
                &target_cell,
                &target_cell_hitbox,
                &target_projectile
                    .iter()
                    .copied()
                    .map(Into::into)
                    .collect::<Vec<_>>(),
                &target_projectile_direction,
                &target_projectile_cell,
                &target_projectile_cell_conjunction,
                &target_block_event,
                &target_enemy_contact,
                score_frame_start,
                limit,
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
            record_stop_on_target_enemy_contact,
            record_contact_tail_frames,
            record_notes,
            record_target_label,
            record_target_level,
            record_target_point,
            record_target_cell,
            record_target_cell_hitbox,
            record_target_enemy,
            record_frontier_kind,
            scenario,
            playtest,
            mic,
            mic_command,
        } => {
            let repo_root = current_dir()?;
            let rom_path = resolve_rom_path(&repo_root, rom)?;
            let upstream_dir = resolve_upstream_dir(&repo_root, dir);
            let manifest = prepare_generated_assets(&repo_root, &rom_path, &upstream_dir)?;
            if playtest {
                if record.is_some() {
                    bail!(
                        "--playtest and --record are mutually exclusive; playtest manages its own segment recordings"
                    );
                }
                if scenario.is_some() {
                    bail!(
                        "--playtest and --scenario are mutually exclusive; playtest needs live input"
                    );
                }
            } else if mic || mic_command.is_some() {
                bail!("--mic/--mic-command require --playtest");
            }
            let scripted_scenario = scenario
                .as_deref()
                .map(load_record_scenario_file)
                .transpose()?;
            let effective_warp_level = warp_level.or_else(|| {
                scripted_scenario
                    .as_ref()
                    .and_then(|dynamic| dynamic.warp_to_level)
            });
            let effective_settled = settled
                || scripted_scenario
                    .as_ref()
                    .is_some_and(|dynamic| dynamic.settle_oracle_warp);
            ensure_play_native_level_layouts(&rom_path, &manifest, effective_warp_level)?;
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
            let record_frames = match (&record_to, record_frames, &scripted_scenario) {
                (Some(_), None, Some(dynamic)) => Some(dynamic.frame_count),
                _ => record_frames,
            };
            let record_route_target = play_native_record_route_target(
                record_target_label,
                record_target_level,
                &record_target_point,
                &record_target_cell,
                &record_target_cell_hitbox,
                &record_target_enemy,
            )?;
            if record_to.is_none() && record_route_target.is_some() {
                bail!("--record-target-* options require --record <slug>");
            }
            if record_stop_on_target_enemy_contact {
                if record_to.is_none() {
                    bail!("--record-stop-on-target-enemy-contact requires --record <slug>");
                }
                if record_route_target
                    .as_ref()
                    .is_none_or(|target| target.enemies.is_empty())
                {
                    bail!(
                        "--record-stop-on-target-enemy-contact requires at least one --record-target-enemy"
                    );
                }
            }
            let playtest_options = playtest.then(|| {
                let session_slug = playtest_session_slug(unix_time_ms() / 1000);
                PlaytestOptions {
                    session_dir: repo_root.join(".cache/playtest").join(session_slug),
                    issues: load_playtest_issues(&repo_root),
                    mic_command: mic_command
                        .or_else(|| mic.then(|| native_player::DEFAULT_MIC_COMMAND.to_owned())),
                }
            });
            run_native_player(
                &manifest,
                &NativePlayOptions {
                    fullscreen,
                    scale,
                    debug_overlay,
                    warp_level: effective_warp_level,
                    settled: effective_settled,
                    record_to,
                    record_frames,
                    record_stop_on_target_enemy_contact,
                    record_contact_tail_frames,
                    record_notes,
                    record_route_target,
                    record_frontier_kind: Some(frontier_kind_arg(record_frontier_kind).to_owned()),
                    scripted_route: scripted_scenario.map(NativeScriptedRoute::from),
                    playtest: playtest_options,
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
        Commands::AudioAudit {
            rom,
            dir,
            sound_id,
            frames,
            sample_rate,
            json,
        } => {
            let repo_root = current_dir()?;
            let rom_path = resolve_rom_path(&repo_root, rom)?;
            let upstream_dir = resolve_upstream_dir(&repo_root, dir);
            bootstrap_upstream(&upstream_dir)?;
            let verified = verify_rom(&rom_path)?;
            if !matches!(verified.identity, RomIdentity::KnownKidChameleon) {
                bail!("audio-audit currently supports only the known Kid Chameleon ROM dump");
            }
            let rom_bytes = fs::read(&rom_path)
                .with_context(|| format!("failed to read ROM from {}", rom_path.display()))?;
            let manifest = prepare_generated_assets(&repo_root, &rom_path, &upstream_dir)?;
            run_audio_audit(
                &repo_root,
                &upstream_dir,
                &manifest,
                &rom_bytes,
                sound_id,
                frames,
                sample_rate,
                json,
            )?;
        }
        Commands::VerifyNativeAudioOutputFile { rom, dir, scenario } => {
            let repo_root = current_dir()?;
            let rom_path = resolve_rom_path(&repo_root, rom)?;
            let upstream_dir = resolve_upstream_dir(&repo_root, dir);
            let verified = verify_rom(&rom_path)?;
            if !matches!(verified.identity, RomIdentity::KnownKidChameleon) {
                bail!(
                    "verify-native-audio-output-file currently supports only the known Kid Chameleon ROM dump"
                );
            }
            let manifest = prepare_generated_assets(&repo_root, &rom_path, &upstream_dir)?;
            let dynamic = load_record_scenario_file(&scenario)?;
            if let Some(level) = dynamic.warp_to_level {
                ensure_level_layouts(&rom_path, &manifest, &[level])?;
            }
            verify_native_audio_output_file(&repo_root, &rom_path, &manifest, &dynamic, &scenario)?;
            println!("native audio-output verification passed: {}", dynamic.slug);
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
        Commands::BootstrapCpuVectors => {
            let repo_root = current_dir()?;
            let reference_root = resolve_reference_root(&repo_root);
            let references = bootstrap_cpu_vector_repos(&reference_root)?;
            print_bootstrapped_references(&reference_root, &references);
        }
        Commands::VerifyCpuVectors {
            set,
            per_file_limit,
            json,
        } => {
            let repo_root = current_dir()?;
            let reference_root = resolve_reference_root(&repo_root);
            let sets = set.map_or_else(
                || cpu_vectors::vector::CpuVectorSet::ALL.to_vec(),
                |set| vec![set],
            );
            cpu_vectors::ensure_vectors_present(&reference_root, &sets)?;
            let report = cpu_vectors::verify_cpu_vectors(&reference_root, &sets, per_file_limit)?;
            let path = cpu_vectors::write_report(&repo_root, &report)?;
            if json {
                println!("{}", serde_json::to_string_pretty(&report)?);
            } else {
                print!("{}", cpu_vectors::format_report(&report));
                println!("report: {}", path.display());
            }
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
        Commands::PackWebBundle { rom, dir, output } => {
            let repo_root = current_dir()?;
            let rom_path = resolve_rom_path(&repo_root, rom)?;
            let upstream_dir = resolve_upstream_dir(&repo_root, dir);
            let manifest = prepare_generated_assets(&repo_root, &rom_path, &upstream_dir)?;
            let blob = pack_runtime_bundle_blob(&manifest.output_root)?;
            if let Some(parent) = output.parent() {
                fs::create_dir_all(parent)
                    .with_context(|| format!("failed to create {}", parent.display()))?;
            }
            fs::write(&output, &blob)
                .with_context(|| format!("failed to write {}", output.display()))?;
            println!(
                "packed {} -> {} ({:.1} MiB)",
                manifest.output_root.display(),
                output.display(),
                blob.len() as f64 / (1024.0 * 1024.0)
            );
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
        Commands::CompareCachedNativeGameplay { scenario, run_slug } => {
            let repo_root = current_dir()?;
            compare_cached_native_gameplay_run(&repo_root, &run_slug, scenario)?;
        }
        Commands::PlaytestIngest {
            session,
            transcript,
            whisper_cmd,
            out,
        } => {
            let repo_root = current_dir()?;
            playtest_ingest::run_playtest_ingest(
                &repo_root,
                &session,
                transcript.as_deref(),
                whisper_cmd.as_deref(),
                out.as_deref(),
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
            include_rng,
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
                    include_rng,
                };
                sweep::run_sweep_with_options(
                    &repo_root, &rom_path, &manifest, &level_ids, tier, &options,
                )?;
            }
        }
        Commands::GemsRingDecode { manifest, output } => {
            let repo_root = current_dir()?;
            let manifest_path = manifest
                .unwrap_or_else(|| repo_root.join(".cache/oracle-probe/oracle/manifest.json"));
            gems_ring::run_gems_ring_decode(&manifest_path, output.as_deref())?;
        }
        Commands::GemsSpanReplay {
            rom,
            manifest,
            anchor_frame,
            frame_start,
            frame_end,
            stamp_skew_cycles,
            output,
        } => {
            let repo_root = current_dir()?;
            let rom_path = resolve_rom_path(&repo_root, rom)?;
            let manifest_path = manifest
                .unwrap_or_else(|| repo_root.join(".cache/oracle-probe/oracle/manifest.json"));
            gems_span::run_gems_span_replay(
                &repo_root,
                &rom_path,
                &manifest_path,
                anchor_frame,
                frame_start,
                frame_end,
                stamp_skew_cycles,
                output.as_deref(),
            )?;
        }
        Commands::GemsReplayProbe {
            rom,
            dir,
            scenario,
            output,
        } => {
            let repo_root = current_dir()?;
            let rom_path = resolve_rom_path(&repo_root, rom)?;
            let upstream_dir = resolve_upstream_dir(&repo_root, dir);
            let manifest = prepare_generated_assets(&repo_root, &rom_path, &upstream_dir)?;
            let scenario_path = scenario.unwrap_or_else(|| {
                repo_root.join("tests/data/tas/any-percent-published/scenario.json")
            });
            tas_gems::run_gems_replay_probe(
                &repo_root,
                &rom_path,
                &manifest,
                &scenario_path,
                output.as_deref(),
            )?;
        }
        Commands::GemsCensus { report, output } => {
            let repo_root = current_dir()?;
            let report_path = report.unwrap_or_else(|| {
                repo_root.join(".cache/tas-verify/any-percent-published/gems-mismatch-report.json")
            });
            let output_path =
                output.unwrap_or_else(|| repo_root.join(".cache/audit/gems-census.json"));
            gems_census::run_gems_census(&report_path, &output_path)?;
        }
        Commands::SweepCensus {
            rom,
            dir,
            run,
            output,
            no_roster,
        } => {
            let repo_root = current_dir()?;
            let manifest = if no_roster {
                None
            } else {
                let rom_path = resolve_rom_path(&repo_root, rom)?;
                let upstream_dir = resolve_upstream_dir(&repo_root, dir);
                Some(prepare_generated_assets(
                    &repo_root,
                    &rom_path,
                    &upstream_dir,
                )?)
            };
            sweep::run_sweep_census(
                &repo_root,
                manifest.as_ref(),
                run.as_deref(),
                output.as_deref(),
            )?;
        }
        Commands::OracleSettleCensus {
            rom,
            levels,
            output,
            from_json,
            shell_table,
            dump_trace,
        } => {
            let repo_root = current_dir()?;
            let rom_path = resolve_rom_path(&repo_root, rom)?;
            let level_ids = sweep::parse_level_range(&levels)?;
            sweep::run_oracle_settle_census(
                &repo_root,
                &rom_path,
                &level_ids,
                output.as_deref(),
                from_json.as_deref(),
                shell_table.as_deref(),
                dump_trace,
            )?;
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
        Commands::BacklogLint => {
            let repo_root = current_dir()?;
            backlog::run_backlog_lint(&repo_root)?;
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
            scene,
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
            sweep::capture_scenario_frames(
                &manifest,
                &dynamic,
                &frame_indices,
                &output,
                filter,
                scene,
            )?;
        }
        Commands::CaptureOracleScenarioFrames {
            rom,
            scenario,
            input,
            frames,
            output,
        } => {
            let repo_root = current_dir()?;
            let rom_path = resolve_rom_path(&repo_root, rom)?;
            let env = ensure_oracles(&repo_root)?;
            let frame_indices: Vec<usize> = frames
                .split(',')
                .map(|s| s.trim().parse::<usize>().context("invalid frame index"))
                .collect::<Result<_>>()?;
            let dynamic = dynamic_scenario_with_frame_checkpoints(
                apply_dynamic_scenario_input_overlays(
                    load_record_scenario_file(&scenario)?,
                    &input,
                )?,
                &frame_indices,
            );
            fs::create_dir_all(&output)
                .with_context(|| format!("failed to create {}", output.display()))?;
            let scenario_path = output.join("oracle-frame-scenario.json");
            fs::write(&scenario_path, serde_json::to_vec_pretty(&dynamic)?)
                .with_context(|| format!("failed to write {}", scenario_path.display()))?;
            let skip_frames_before = frame_indices.iter().min().copied();
            let oracle = run_jgenesis_capture_dynamic(
                &env,
                &rom_path,
                &scenario_path,
                &output,
                skip_frames_before,
            )?;
            fs::write(
                output.join("manifest.json"),
                serde_json::to_vec_pretty(&oracle)?,
            )
            .with_context(|| {
                format!("failed to write {}", output.join("manifest.json").display())
            })?;
            for frame in &oracle.frames {
                println!("  saved {}", frame.png_path.display());
            }
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
            include_rng,
            compare_pixels,
            compare_gems,
            compare_gems_commands,
            pixel_mismatch_report,
            gems_mismatch_report,
        } => {
            let repo_root = current_dir()?;
            let rom_path = resolve_rom_path(&repo_root, rom)?;
            let upstream_dir = resolve_upstream_dir(&repo_root, dir);
            let manifest = prepare_generated_assets(&repo_root, &rom_path, &upstream_dir)?;
            if pixel_mismatch_report.is_some() && !compare_pixels {
                bail!("--pixel-mismatch-report requires --compare-pixels");
            }
            if gems_mismatch_report.is_some() && !compare_gems {
                bail!("--gems-mismatch-report requires --compare-gems");
            }
            if compare_gems_commands && !compare_gems {
                bail!("--compare-gems-commands requires --compare-gems");
            }
            if bisect {
                if compare_pixels {
                    bail!("--compare-pixels is not supported with --bisect");
                }
                if compare_gems {
                    bail!("--compare-gems is not supported with --bisect");
                }
                if pixel_mismatch_report.is_some() {
                    bail!("--pixel-mismatch-report is not supported with --bisect");
                }
                if gems_mismatch_report.is_some() {
                    bail!("--gems-mismatch-report is not supported with --bisect");
                }
                if compare_gems_commands {
                    bail!("--compare-gems-commands is not supported with --bisect");
                }
                tas::bisect_tas(
                    &repo_root,
                    &rom_path,
                    &manifest,
                    &scenario,
                    frame_start,
                    frame_end,
                    frontier_kind,
                    include_rng,
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
                    include_rng,
                    compare_pixels,
                    compare_gems,
                    compare_gems_commands,
                    pixel_mismatch_report.as_deref(),
                    gems_mismatch_report.as_deref(),
                )?;
            }
        }
        Commands::VisualTriage {
            rom,
            dir,
            scenario,
            mismatch_report,
            frame_start,
            frame_end,
            top,
            rerun_oracle,
        } => {
            let repo_root = current_dir()?;
            let rom_path = resolve_rom_path(&repo_root, rom)?;
            let upstream_dir = resolve_upstream_dir(&repo_root, dir);
            let manifest = prepare_generated_assets(&repo_root, &rom_path, &upstream_dir)?;
            visual_triage::run_visual_triage(
                &repo_root,
                &rom_path,
                &manifest,
                &scenario,
                mismatch_report.as_deref(),
                frame_start,
                frame_end,
                top,
                rerun_oracle,
            )?;
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
            rng_trace,
            drips_trace,
            player_trace,
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
                rng_trace,
                drips_trace,
                player_trace,
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
            vdp_fifo_commit_frames,
            vdp_fifo_commit_address_range,
            vdp_fifo_summary,
            m68k_pc_trace,
            gems_stimulus_frames,
            capture_gems_state,
            when,
            capture_cpu_state,
            capture_settle_trace,
            no_frame_captures,
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

            // Generate checkpoints for every frame in the range. Checkpoints
            // are what drive the oracle helper's per-frame PNG/state capture;
            // trace and FIFO records are drained independently, so a
            // trace-only probe can skip them entirely.
            let start = frame_start.unwrap_or(0);
            let end = frame_end.unwrap_or(dyn_scenario.frame_count.saturating_sub(1));
            dyn_scenario.checkpoints = if no_frame_captures {
                Vec::new()
            } else {
                (start..=end)
                    .map(|f| scenarios::DynamicCheckpoint {
                        label: format!("f{f}"),
                        frame_index: f,
                    })
                    .collect()
            };

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
                vdp_fifo_commit_frames: vdp_fifo_commit_frames.clone(),
                vdp_fifo_commit_address_range: vdp_fifo_commit_address_range.clone(),
                m68k_pc_trace: m68k_pc_trace.clone(),
                gems_stimulus_frames: gems_stimulus_frames.clone(),
                capture_gems_state,
                // Ring snapshots (#435) ride the signature capture, so gems
                // state dumps also carry the decoded-command cross-check
                // input.
                capture_gems_frame_signature: capture_gems_state,
                capture_cpu_state,
                capture_settle_trace,
                capture_rng_stream: false,
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
                    "settle_phase,settle_step,game_mode,current_level_id,rng_offset,rng_last_byte,rng_ram,paused,camera_x_pos,camera_y_pos,kid_x_pos,kid_y_pos,kid_x_vel,kid_y_vel,number_objects,background_scroll_loc_3050a,background_row_offsets_loc_30744,background_source_window_loc_30744,background_source_absolute_loc_30744,background_source_rows_loc_12848,background_plane_b_rows_loc_307d8,background_plane_b_spans_loc_307d8"
                );
                for row in &oracle_manifest.settle_trace {
                    println!(
                        "{},{},0x{:04X},0x{:04X},0x{:04X},0x{:02X},{},{},0x{:08X},0x{:08X},0x{:08X},0x{:08X},0x{:08X},0x{:08X},{},{},{},{},{},{},{},{}",
                        row.phase,
                        row.step,
                        row.game_mode,
                        row.current_level_id,
                        row.rng_offset,
                        row.rng_last_byte,
                        csv_field(&hex_bytes(&row.rng_ram)),
                        row.paused,
                        row.camera_x_pos,
                        row.camera_y_pos,
                        row.kid_x_pos,
                        row.kid_y_pos,
                        row.kid_x_vel as u32,
                        row.kid_y_vel as u32,
                        row.number_objects,
                        csv_field(&row.background_scroll_loc_3050a),
                        csv_field(&row.background_row_offsets_loc_30744),
                        csv_field(&row.background_source_window_loc_30744),
                        csv_field(&row.background_source_absolute_loc_30744),
                        csv_field(&row.background_source_rows_loc_12848),
                        csv_field(&row.background_plane_b_rows_loc_307d8),
                        csv_field(&row.background_plane_b_spans_loc_307d8),
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
            for row in oracle_manifest
                .vdp_fifo_commits
                .iter()
                .filter(|row| (start..=end).contains(&row.frame))
            {
                if vdp_fifo_summary {
                    println!(
                        "vdp_fifo_summary={}",
                        serde_json::to_string(&crate::oracle::summarise_vdp_fifo_commits(row))?
                    );
                } else {
                    println!(
                        "vdp_fifo_frame={},{}",
                        row.frame,
                        serde_json::to_string(&row.commits)?
                    );
                }
            }
            for row in oracle_manifest
                .vdp_fifo_enqueues
                .iter()
                .filter(|row| (start..=end).contains(&row.frame))
            {
                if vdp_fifo_summary {
                    println!(
                        "vdp_fifo_enqueue_summary={}",
                        serde_json::to_string(&crate::oracle::summarise_vdp_fifo_commits(row))?
                    );
                } else {
                    println!(
                        "vdp_fifo_enqueue_frame={},{}",
                        row.frame,
                        serde_json::to_string(&row.commits)?
                    );
                }
            }
            for row in oracle_manifest
                .m68k_pc_trace
                .iter()
                .filter(|row| (start..=end).contains(&row.frame))
            {
                println!("m68k_pc_trace={}", serde_json::to_string(row)?);
            }

            // Also output checkpoint states for the range
            let state_count = oracle_manifest.states.len();
            eprintln!(
                "Oracle probe complete: {} state checkpoints, {} watch rows, {} VDP sample rows, {} FIFO commit rows",
                state_count,
                watch_rows_in_range.len(),
                oracle_manifest
                    .vdp_pixel_samples
                    .iter()
                    .filter(|row| (start..=end).contains(&row.frame))
                    .count(),
                oracle_manifest
                    .vdp_fifo_commits
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
        Commands::RasterSpanReplay {
            census,
            cluster,
            output,
        } => {
            raster_span::run_raster_span_replay(&census, &cluster, output.as_deref())?;
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
            check_no_new_test_only,
        } => {
            let repo_root = current_dir()?;
            asm_coverage::run_asm_coverage(
                &repo_root,
                update,
                subsystem.as_deref(),
                uncovered,
                json,
                check_required,
                check_no_new_test_only,
            )?;
        }
        Commands::TasPixelBaseline {
            report,
            frontier,
            slug,
            update,
            force,
        } => {
            let code = tas_pixel_baseline::run_tas_pixel_baseline(
                &report, &frontier, &slug, update, force,
            )?;
            if code != 0 {
                std::process::exit(code);
            }
        }
        Commands::TasGemsBaseline {
            report,
            frontier,
            slug,
            update,
            force,
        } => {
            let code =
                tas_gems_baseline::run_tas_gems_baseline(&report, &frontier, &slug, update, force)?;
            if code != 0 {
                std::process::exit(code);
            }
        }
        Commands::TasGemsCommandBaseline {
            report,
            frontier,
            slug,
            update,
            force,
        } => {
            let code = tas_gems_command_baseline::run_tas_gems_command_baseline(
                &report, &frontier, &slug, update, force,
            )?;
            if code != 0 {
                std::process::exit(code);
            }
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
    if let Some(count) = scenario.initial_lives {
        runtime.set_probe_lives_count(count);
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

fn csv_field(value: &str) -> String {
    if value.contains([',', '"', '\n', '\r']) {
        format!("\"{}\"", value.replace('"', "\"\""))
    } else {
        value.to_owned()
    }
}

fn hex_bytes(bytes: &[u8]) -> String {
    bytes.iter().fold(
        String::with_capacity(bytes.len().saturating_mul(2)),
        |mut output, byte| {
            write!(output, "{byte:02X}").expect("writing to String cannot fail");
            output
        },
    )
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

#[derive(Debug, Clone, PartialEq, Eq)]
struct BatchProbeCandidate {
    label: String,
    input_specs: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct BatchProbeWindow {
    label: String,
    start_min: usize,
    start_max: usize,
    duration: usize,
    step: usize,
    button_spec: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct BatchTargetProjectileCell {
    kind: ActiveProjectileKind,
    cell: TargetCell,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct BatchTargetProjectileDirection {
    kind: ActiveProjectileKind,
    direction: TargetProjectileDirection,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct BatchTargetBlockEvent {
    kind: TargetBlockEventKind,
    cell: TargetCell,
}

fn parse_batch_target_projectile_cell(spec: &str) -> Result<BatchTargetProjectileCell> {
    let (kind, cell) = spec
        .split_once(':')
        .ok_or_else(|| anyhow!("target projectile cell must be KIND:COL,ROW, got {spec}"))?;
    let kind = <TargetProjectileKind as ValueEnum>::from_str(kind.trim(), true)
        .map_err(|message| anyhow!(message))?;
    Ok(BatchTargetProjectileCell {
        kind: kind.into(),
        cell: parse_target_cell(cell)?,
    })
}

fn parse_batch_target_projectile_direction(spec: &str) -> Result<BatchTargetProjectileDirection> {
    let (kind, direction) = spec
        .split_once(':')
        .ok_or_else(|| anyhow!("target projectile direction must be KIND:DIRECTION, got {spec}"))?;
    let kind = <TargetProjectileKind as ValueEnum>::from_str(kind.trim(), true)
        .map_err(|message| anyhow!(message))?;
    let direction = TargetProjectileDirection::from_str(direction.trim(), true)
        .map_err(|message| anyhow!(message))?;
    Ok(BatchTargetProjectileDirection {
        kind: kind.into(),
        direction,
    })
}

fn projectile_cell_targets(
    targets: &[BatchTargetProjectileCell],
) -> Vec<(ActiveProjectileKind, TargetCell)> {
    targets
        .iter()
        .map(|target| (target.kind, target.cell))
        .collect()
}

fn parse_batch_target_block_event(spec: &str) -> Result<BatchTargetBlockEvent> {
    let (kind, cell) = spec
        .split_once(':')
        .ok_or_else(|| anyhow!("target block event must be KIND:COL,ROW, got {spec}"))?;
    let kind =
        TargetBlockEventKind::from_str(kind.trim(), true).map_err(|message| anyhow!(message))?;
    Ok(BatchTargetBlockEvent {
        kind,
        cell: parse_target_cell(cell)?,
    })
}

fn parse_batch_probe_candidate(spec: &str) -> Result<BatchProbeCandidate> {
    let (label, input_spec) = spec
        .split_once('=')
        .ok_or_else(|| anyhow!("batch candidate must be LABEL=INPUT[,INPUT...], got {spec}"))?;
    let label = label.trim();
    if label.is_empty() {
        bail!("batch candidate label must not be empty");
    }
    let input_specs = input_spec
        .split(',')
        .map(str::trim)
        .filter(|part| !part.is_empty())
        .map(|part| {
            parse_input_spec(part)?;
            Ok(part.to_string())
        })
        .collect::<Result<Vec<_>>>()?;
    if input_specs.is_empty() {
        bail!("batch candidate {label} must include at least one input overlay");
    }
    Ok(BatchProbeCandidate {
        label: label.to_string(),
        input_specs,
    })
}

fn parse_batch_probe_window(spec: &str) -> Result<BatchProbeWindow> {
    let (label, range_spec) = spec.split_once('=').ok_or_else(|| {
        anyhow!("batch sweep window must be LABEL=START-END:DURATION:STEP:BUTTONS, got {spec}")
    })?;
    let label = label.trim();
    if label.is_empty() {
        bail!("batch sweep window label must not be empty");
    }
    let mut parts = range_spec.split(':').map(str::trim);
    let frame_range = parts
        .next()
        .ok_or_else(|| anyhow!("batch sweep window {label} is missing START-END"))?;
    let duration = parts
        .next()
        .ok_or_else(|| anyhow!("batch sweep window {label} is missing DURATION"))?
        .parse::<usize>()
        .with_context(|| format!("invalid duration in batch sweep window {label}"))?;
    let step = parts
        .next()
        .ok_or_else(|| anyhow!("batch sweep window {label} is missing STEP"))?
        .parse::<usize>()
        .with_context(|| format!("invalid step in batch sweep window {label}"))?;
    let button_spec = parts
        .next()
        .ok_or_else(|| anyhow!("batch sweep window {label} is missing BUTTONS"))?;
    if parts.next().is_some() {
        bail!("batch sweep window {label} has too many ':' fields");
    }
    let (start_min, start_max) = frame_range
        .split_once('-')
        .ok_or_else(|| anyhow!("batch sweep window {label} range must be START-END"))?;
    let start_min = start_min
        .parse::<usize>()
        .with_context(|| format!("invalid start frame in batch sweep window {label}"))?;
    let start_max = start_max
        .parse::<usize>()
        .with_context(|| format!("invalid end frame in batch sweep window {label}"))?;
    if start_max < start_min {
        bail!("batch sweep window {label} range must be ascending");
    }
    if duration == 0 {
        bail!("batch sweep window {label} duration must be positive");
    }
    if step == 0 {
        bail!("batch sweep window {label} step must be positive");
    }
    parse_button_mask(button_spec)
        .with_context(|| format!("invalid buttons in batch sweep window {label}"))?;
    Ok(BatchProbeWindow {
        label: label.to_string(),
        start_min,
        start_max,
        duration,
        step,
        button_spec: button_spec.to_string(),
    })
}

fn batch_probe_window_specs(window: &BatchProbeWindow) -> Vec<(String, String)> {
    (window.start_min..=window.start_max)
        .step_by(window.step)
        .map(|start| {
            let end = start.saturating_add(window.duration).saturating_sub(1);
            (
                format!("{}{}", window.label, start),
                format!("{start}-{end}:{}", window.button_spec),
            )
        })
        .collect()
}

fn generated_batch_probe_candidates(
    windows: &[BatchProbeWindow],
    max_generated: usize,
) -> Result<Vec<BatchProbeCandidate>> {
    if windows.is_empty() {
        return Ok(Vec::new());
    }
    let choices = windows
        .iter()
        .map(batch_probe_window_specs)
        .collect::<Vec<_>>();
    let generated_count = choices.iter().try_fold(1usize, |acc, choice| {
        acc.checked_mul(choice.len())
            .ok_or_else(|| anyhow!("batch sweep windows overflowed candidate-count calculation"))
    })?;
    if generated_count > max_generated {
        bail!(
            "batch sweep windows would generate {generated_count} candidates, above --max-generated {max_generated}"
        );
    }

    let mut candidates = vec![BatchProbeCandidate {
        label: String::new(),
        input_specs: Vec::new(),
    }];
    for choice in choices {
        let mut next = Vec::with_capacity(candidates.len() * choice.len());
        for candidate in &candidates {
            for (label_part, input_spec) in &choice {
                let label = if candidate.label.is_empty() {
                    label_part.clone()
                } else {
                    format!("{}_{}", candidate.label, label_part)
                };
                let mut input_specs = candidate.input_specs.clone();
                input_specs.push(input_spec.clone());
                next.push(BatchProbeCandidate { label, input_specs });
            }
        }
        candidates = next;
    }
    Ok(candidates)
}

#[derive(Debug)]
struct BatchProbeResult {
    label: String,
    final_mode: NativeGameMode,
    final_mode_value: u16,
    lives: u16,
    hitpoints: u16,
    helmet_id: u16,
    sound_ids: Vec<u8>,
    first_target_score: Option<i64>,
    target_summaries: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct BlockEventMatch {
    replay_frame: usize,
}

fn sorted_batch_probe_results(mut results: Vec<BatchProbeResult>) -> Vec<BatchProbeResult> {
    results.sort_by(|left, right| {
        left.first_target_score
            .unwrap_or(i64::MAX)
            .cmp(&right.first_target_score.unwrap_or(i64::MAX))
            .then_with(|| left.label.cmp(&right.label))
    });
    results
}

fn target_block_event_from_frame(
    trace: &[NativeTraceFrame],
    kind: TargetBlockEventKind,
    target_cell: TargetCell,
    score_frame_start: usize,
) -> Option<BlockEventMatch> {
    trace
        .iter()
        .enumerate()
        .skip(score_frame_start)
        .find(|(_replay_frame, frame)| match kind {
            TargetBlockEventKind::Bounce => {
                frame
                    .block_bounces
                    .iter()
                    .any(|&(col, row, _remaining, _y_offset)| {
                        i64::from(col) == target_cell.col && i64::from(row) == target_cell.row
                    })
            }
            TargetBlockEventKind::Destroy => {
                frame
                    .block_destroy_effects
                    .iter()
                    .any(|&(x_px, y_px, _remaining, _direction)| {
                        i64::from(x_px / 16) == target_cell.col
                            && i64::from(y_px / 16) == target_cell.row
                    })
            }
        })
        .map(|(replay_frame, _frame)| BlockEventMatch { replay_frame })
}

fn probe_dynamic_record_scenario_batch(
    manifest: &AssetManifest,
    base: &DynamicScenario,
    candidate_specs: &[String],
    sweep_windows: &[BatchProbeWindow],
    max_generated: usize,
    target_points: &[TargetPoint],
    target_cells: &[TargetCell],
    target_cell_hitboxes: &[TargetCell],
    target_projectiles: &[ActiveProjectileKind],
    target_projectile_directions: &[BatchTargetProjectileDirection],
    target_projectile_cells: &[BatchTargetProjectileCell],
    target_projectile_cell_conjunctions: &[BatchTargetProjectileCell],
    target_block_events: &[BatchTargetBlockEvent],
    target_enemy_contacts: &[u8],
    score_frame_start: usize,
    limit: usize,
) -> Result<()> {
    let mut candidates = candidate_specs
        .iter()
        .map(|spec| parse_batch_probe_candidate(spec))
        .collect::<Result<Vec<_>>>()?;
    candidates.extend(generated_batch_probe_candidates(
        sweep_windows,
        max_generated,
    )?);
    if candidates.is_empty() {
        bail!("at least one --candidate or --sweep-window is required");
    }

    let mut results = Vec::with_capacity(candidates.len());
    for candidate in candidates {
        let mut dynamic =
            apply_dynamic_scenario_input_overlays(base.clone(), &candidate.input_specs)?;
        dynamic.slug = format!("{}__{}", base.slug, candidate.label);
        let trace = run_native_dynamic_scenario(
            manifest.clone(),
            &dynamic,
            NativeDynamicRunOptions {
                settled_warp: false,
                trace_physics: false,
            },
        )?;
        let final_frame = trace.last().ok_or_else(|| {
            anyhow!(
                "native trace was empty for batch candidate {}",
                candidate.label
            )
        })?;
        let mut first_target_score = None;
        let mut target_summaries = Vec::with_capacity(
            target_points.len()
                + target_cells.len()
                + target_cell_hitboxes.len()
                + target_projectiles.len()
                + target_projectile_directions.len()
                + target_projectile_cells.len()
                + target_projectile_cell_conjunctions.len()
                + target_block_events.len()
                + target_enemy_contacts.len(),
        );
        for (index, target_point) in target_points.iter().copied().enumerate() {
            let summary = target_point_approach_from_frame(
                &trace,
                target_point,
                true,
                score_frame_start,
            ).map_or_else(|| {
                    format!(
                        "target {},{} healthy <none>",
                        target_point.x_px, target_point.y_px
                    )
                }, |approach| {
                    if index == 0 {
                        first_target_score = Some(approach.manhattan_px);
                    }
                    format!(
                        "target {},{} healthy dx_px {:+} dy_px {:+} manhattan_px {} @{} kid {:08X},{:08X}",
                        target_point.x_px,
                        target_point.y_px,
                        approach.dx_px,
                        approach.dy_px,
                        approach.manhattan_px,
                        approach.replay_frame,
                        approach.kid_x,
                        approach.kid_y,
                    )
                });
            target_summaries.push(summary);
        }
        for (index, target_cell) in target_cells.iter().copied().enumerate() {
            let summary = target_cell_approach_from_frame(
                &trace,
                target_cell,
                true,
                score_frame_start,
            ).map_or_else(|| format!("cell {},{} healthy <none>", target_cell.col, target_cell.row), |approach| {
                if target_points.is_empty() && index == 0 {
                    first_target_score = Some(approach.manhattan_px);
                }
                format!(
                    "cell {},{} healthy dx_px {:+} dy_px {:+} manhattan_px {} @{} kid {:08X},{:08X}",
                    target_cell.col,
                    target_cell.row,
                    approach.dx_px,
                    approach.dy_px,
                    approach.manhattan_px,
                    approach.replay_frame,
                    approach.kid_x,
                    approach.kid_y,
                )
            });
            target_summaries.push(summary);
        }
        for (index, target_cell) in target_cell_hitboxes.iter().copied().enumerate() {
            let summary = target_cell_hitbox_approach_from_frame(
                &trace,
                target_cell,
                true,
                score_frame_start,
            ).map_or_else(|| {
                format!(
                    "cell hitbox {},{} healthy <none>",
                    target_cell.col, target_cell.row
                )
            }, |approach| {
                if target_points.is_empty() && target_cells.is_empty() && index == 0 {
                    first_target_score = Some(approach.manhattan_px);
                }
                format!(
                    "cell hitbox {},{} healthy dx_px {:+} dy_px {:+} manhattan_px {} @{} kid {:08X},{:08X}",
                    target_cell.col,
                    target_cell.row,
                    approach.dx_px,
                    approach.dy_px,
                    approach.manhattan_px,
                    approach.replay_frame,
                    approach.kid_x,
                    approach.kid_y,
                )
            });
            target_summaries.push(summary);
        }
        for (index, target_projectile) in target_projectiles.iter().copied().enumerate() {
            let summary = target_projectile_kid_approach_from_frame(
                &trace,
                target_projectile,
                true,
                score_frame_start,
            )
            .map_or_else(
                || format!("projectile {target_projectile:?} healthy kid <none>"),
                |approach| {
                    if target_points.is_empty()
                        && target_cells.is_empty()
                        && target_cell_hitboxes.is_empty()
                        && index == 0
                    {
                        first_target_score = Some(approach.manhattan_px);
                    }
                    format!(
                        "projectile {:?} healthy kid dx_px {:+} dy_px {:+} manhattan_px {} @{}",
                        target_projectile,
                        approach.dx_px,
                        approach.dy_px,
                        approach.manhattan_px,
                        approach.replay_frame,
                    )
                },
            );
            target_summaries.push(summary);
        }
        for (index, target_projectile_direction) in
            target_projectile_directions.iter().copied().enumerate()
        {
            let summary = target_projectile_directional_kid_approach_from_frame(
                &trace,
                target_projectile_direction.kind,
                target_projectile_direction.direction,
                true,
                score_frame_start,
            ).map_or_else(|| {
                format!(
                    "projectile {:?} direction {:?} healthy kid <none>",
                    target_projectile_direction.kind, target_projectile_direction.direction
                )
            }, |approach| {
                if target_points.is_empty()
                    && target_cells.is_empty()
                    && target_cell_hitboxes.is_empty()
                    && target_projectiles.is_empty()
                    && index == 0
                {
                    first_target_score = Some(approach.manhattan_px);
                }
                format!(
                    "projectile {:?} direction {:?} healthy kid dx_px {:+} dy_px {:+} manhattan_px {} @{} projectile {:08X},{:08X} vel {:08X},{:08X}",
                    target_projectile_direction.kind,
                    target_projectile_direction.direction,
                    approach.dx_px,
                    approach.dy_px,
                    approach.manhattan_px,
                    approach.replay_frame,
                    approach.projectile_x,
                    approach.projectile_y,
                    approach.projectile_x_velocity,
                    approach.projectile_y_velocity,
                )
            });
            target_summaries.push(summary);
        }
        for (index, target_projectile_cell) in target_projectile_cells.iter().copied().enumerate() {
            let summary = target_projectile_cell_approach_from_frame(
                &trace,
                target_projectile_cell.kind,
                target_projectile_cell.cell,
                score_frame_start,
            ).map_or_else(|| {
                format!(
                    "projectile {:?} cell {},{} <none>",
                    target_projectile_cell.kind,
                    target_projectile_cell.cell.col,
                    target_projectile_cell.cell.row,
                )
            }, |approach| {
                if target_points.is_empty()
                    && target_cells.is_empty()
                    && target_cell_hitboxes.is_empty()
                    && target_projectiles.is_empty()
                    && target_projectile_directions.is_empty()
                    && index == 0
                {
                    first_target_score = Some(approach.manhattan_px);
                }
                format!(
                    "projectile {:?} cell {},{} dx_px {:+} dy_px {:+} manhattan_px {} @{} projectile {:08X},{:08X}",
                    target_projectile_cell.kind,
                    target_projectile_cell.cell.col,
                    target_projectile_cell.cell.row,
                    approach.dx_px,
                    approach.dy_px,
                    approach.manhattan_px,
                    approach.replay_frame,
                    approach.projectile_x,
                    approach.projectile_y,
                )
            });
            target_summaries.push(summary);
        }
        for (index, target_projectile_cell) in target_projectile_cell_conjunctions
            .iter()
            .copied()
            .enumerate()
        {
            let summary = target_projectile_cell_conjunction_from_frame(
                &trace,
                target_projectile_cell.kind,
                target_projectile_cell.cell,
                score_frame_start,
            ).map_or_else(|| {
                format!(
                    "projectile {:?} cell conjunction {},{} <none>",
                    target_projectile_cell.kind,
                    target_projectile_cell.cell.col,
                    target_projectile_cell.cell.row,
                )
            }, |approach| {
                if target_points.is_empty()
                    && target_cells.is_empty()
                    && target_cell_hitboxes.is_empty()
                    && target_projectiles.is_empty()
                    && target_projectile_directions.is_empty()
                    && target_projectile_cells.is_empty()
                    && index == 0
                {
                    first_target_score = Some(approach.manhattan_px);
                }
                format!(
                    "projectile {:?} cell conjunction {},{} kid_dx_px {:+} kid_dy_px {:+} projectile_dx_px {:+} projectile_dy_px {:+} manhattan_px {} @{} kid {:08X},{:08X} projectile {:08X},{:08X}",
                    target_projectile_cell.kind,
                    target_projectile_cell.cell.col,
                    target_projectile_cell.cell.row,
                    approach.kid_dx_px,
                    approach.kid_dy_px,
                    approach.projectile_dx_px,
                    approach.projectile_dy_px,
                    approach.manhattan_px,
                    approach.replay_frame,
                    approach.kid_x,
                    approach.kid_y,
                    approach.projectile_x,
                    approach.projectile_y,
                )
            });
            target_summaries.push(summary);
        }
        for (index, target_block_event) in target_block_events.iter().copied().enumerate() {
            let summary = target_block_event_from_frame(
                &trace,
                target_block_event.kind,
                target_block_event.cell,
                score_frame_start,
            )
            .map_or_else(
                || {
                    format!(
                        "block {:?} cell {},{} <none>",
                        target_block_event.kind,
                        target_block_event.cell.col,
                        target_block_event.cell.row,
                    )
                },
                |event| {
                    if target_points.is_empty()
                        && target_cells.is_empty()
                        && target_cell_hitboxes.is_empty()
                        && target_projectiles.is_empty()
                        && target_projectile_directions.is_empty()
                        && target_projectile_cells.is_empty()
                        && target_projectile_cell_conjunctions.is_empty()
                        && index == 0
                    {
                        first_target_score = Some(0);
                    }
                    format!(
                        "block {:?} cell {},{} @{}",
                        target_block_event.kind,
                        target_block_event.cell.col,
                        target_block_event.cell.row,
                        event.replay_frame,
                    )
                },
            );
            target_summaries.push(summary);
        }
        for (index, object_type) in target_enemy_contacts.iter().copied().enumerate() {
            let summary = target_enemy_contact_from_frame(&trace, object_type, score_frame_start)
                .map_or_else(
                    || format!("enemy contact 0x{object_type:02X} <none>"),
                    |contact| {
                        if target_points.is_empty()
                            && target_cells.is_empty()
                            && target_cell_hitboxes.is_empty()
                            && target_projectiles.is_empty()
                            && target_projectile_directions.is_empty()
                            && target_projectile_cells.is_empty()
                            && target_projectile_cell_conjunctions.is_empty()
                            && target_block_events.is_empty()
                            && index == 0
                        {
                            first_target_score = Some(contact.score_px);
                        }
                        format!(
                            "enemy contact 0x{object_type:02X} result {} gate {} score_px {} @{} same_frame_stomp={}",
                            contact.result,
                            contact.gate,
                            contact.score_px,
                            contact.replay_frame,
                            contact.same_frame_stomp,
                        )
                    },
                );
            target_summaries.push(summary);
        }
        results.push(BatchProbeResult {
            label: candidate.label,
            final_mode: final_frame.game_mode,
            final_mode_value: final_frame.game_mode_value,
            lives: final_frame.number_lives,
            hitpoints: final_frame.number_hitpoints,
            helmet_id: final_frame.current_helmet_id,
            sound_ids: batch_probe_sound_ids(&trace),
            first_target_score,
            target_summaries,
        });
    }

    for result in sorted_batch_probe_results(results).into_iter().take(limit) {
        println!(
            "candidate {}: final {:?} (0x{:04X}) lives/hp {}/{} helmet {} sounds {} score {} | {}",
            result.label,
            result.final_mode,
            result.final_mode_value,
            result.lives,
            result.hitpoints,
            result.helmet_id,
            format_sound_id_list(&result.sound_ids),
            result
                .first_target_score
                .map_or_else(|| String::from("<none>"), |score| score.to_string()),
            result.target_summaries.join(" | "),
        );
    }

    Ok(())
}

fn batch_probe_sound_ids(trace: &[NativeTraceFrame]) -> Vec<u8> {
    trace
        .iter()
        .flat_map(|frame| frame.played_sound_effect_ids.iter().copied())
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect()
}

fn format_sound_id_list(sound_ids: &[u8]) -> String {
    if sound_ids.is_empty() {
        return String::from("[]");
    }
    let ids = sound_ids
        .iter()
        .map(|id| format!("0x{id:02X}"))
        .collect::<Vec<_>>()
        .join(",");
    format!("[{ids}]")
}

fn dynamic_scenario_with_frame_checkpoints(
    mut scenario: DynamicScenario,
    frame_indices: &[usize],
) -> DynamicScenario {
    let mut frames = frame_indices.to_vec();
    frames.sort_unstable();
    frames.dedup();
    if let Some(last_frame) = frames.last().copied() {
        let required_len = last_frame.saturating_add(1);
        scenario.frame_count = scenario.frame_count.max(required_len);
        if scenario.script.len() < required_len {
            scenario.script.resize(required_len, 0);
        }
    }
    scenario.checkpoints = frames
        .into_iter()
        .map(|frame_index| DynamicCheckpoint {
            label: format!("frame_{frame_index:05}"),
            frame_index,
        })
        .collect();
    scenario
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
        RecordScenarioKind::Level00SlopeRightJumpProbe => {
            level_00_slope_right_jump_probe_scenario()
        }
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
        RecordScenarioKind::HighwaterFlagFloorContactAudit => {
            highwater_flag_floor_contact_audit_scenario()
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
        RecordScenarioKind::Issue260DripsCleanerPrefixAudit => {
            issue_260_drips_cleaner_prefix_audit_scenario()
        }
        RecordScenarioKind::Issue260DripsBJumpClimbSeed => {
            issue_260_drips_b_jump_climb_seed_scenario()
        }
        RecordScenarioKind::Issue260DripsIronKnightClimbProbe => {
            issue_260_drips_iron_knight_climb_probe_scenario()
        }
        RecordScenarioKind::Issue260DripsEarlyJumpCloseProbe => {
            issue_260_drips_early_jump_close_probe_scenario()
        }
        RecordScenarioKind::Issue260DripsDropContactProbe => {
            issue_260_drips_drop_contact_probe_scenario()
        }
        RecordScenarioKind::Issue260DlbTailBRow32Bounce => {
            issue_260_dlb_tail_b_row32_bounce_scenario()
        }
        RecordScenarioKind::Issue260GrPulse600 => issue_260_gr_pulse600_scenario(),
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
        RecordScenarioKind::Issue249Blw2LeftPitWallKidNoJumpAudit => {
            issue_249_blw2_left_pit_wall_kid_no_jump_audit_scenario()
        }
        RecordScenarioKind::Issue249Blw2LeftPitWallRedStealthNoJumpAudit => {
            issue_249_blw2_left_pit_wall_red_stealth_no_jump_audit_scenario()
        }
        RecordScenarioKind::Issue271Level00DragonStompDamageAudit => {
            issue_271_level00_dragon_stomp_damage_audit_scenario()
        }
        RecordScenarioKind::Issue271Level01RockTankStompRouteAudit => {
            issue_271_level01_rock_tank_stomp_route_audit_scenario()
        }
        RecordScenarioKind::Issue271Level01RockTankStompSharpAudit => {
            issue_271_level01_rock_tank_stomp_sharp_audit_scenario()
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

fn play_native_record_route_target(
    label: Option<String>,
    level: Option<u8>,
    points: &[TargetPoint],
    cells: &[TargetCell],
    cell_hitboxes: &[TargetCell],
    enemies: &[u8],
) -> Result<Option<RecordingRouteTarget>> {
    let has_target = label.is_some()
        || level.is_some()
        || !points.is_empty()
        || !cells.is_empty()
        || !cell_hitboxes.is_empty()
        || !enemies.is_empty();
    if !has_target {
        return Ok(None);
    }
    if let Some(label) = &label {
        validate_recording_slug(label)
            .with_context(|| "--record-target-label uses the same safe characters as slugs")?;
    }
    Ok(Some(RecordingRouteTarget {
        label,
        level,
        points: points
            .iter()
            .map(|point| RecordingTargetPoint {
                x_px: point.x_px,
                y_px: point.y_px,
            })
            .collect(),
        cells: cells
            .iter()
            .map(|cell| RecordingTargetCell {
                col: cell.col,
                row: cell.row,
            })
            .collect(),
        cell_hitboxes: cell_hitboxes
            .iter()
            .map(|cell| RecordingTargetCell {
                col: cell.col,
                row: cell.row,
            })
            .collect(),
        enemies: enemies.to_vec(),
    }))
}

fn frontier_kind_arg(kind: sweep::FrontierKind) -> &'static str {
    match kind {
        sweep::FrontierKind::Gameplay => "gameplay",
        sweep::FrontierKind::Enemy => "enemy",
        sweep::FrontierKind::Gems => "gems",
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
        RecordScenarioKind::Level00SlopeRightJumpProbe => "level00-slope-right-jump-probe",
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
        RecordScenarioKind::HighwaterFlagFloorContactAudit => "highwater-flag-floor-contact-audit",
        RecordScenarioKind::HighwaterPass2JuggernautFlagTransitionAudit => {
            "highwater-pass2-juggernaut-flag-transition-audit"
        }
        RecordScenarioKind::HighwaterPass2GrowShooterTouchAudit => {
            "highwater-pass2-grow-shooter-touch-audit"
        }
        RecordScenarioKind::Issue147WallSideJumpAudit => "issue147-wall-side-jump-audit",
        RecordScenarioKind::Issue213DripsMetalBlockAudit => "issue213-drips-metal-block-audit",
        RecordScenarioKind::Issue213DripsDelayedBDropAudit => "issue213-drips-delayed-b-drop-audit",
        RecordScenarioKind::Issue260DripsCleanerPrefixAudit => {
            "issue260-drips-cleaner-prefix-audit"
        }
        RecordScenarioKind::Issue260DripsBJumpClimbSeed => "issue260-drips-b-jump-climb-seed",
        RecordScenarioKind::Issue260DripsIronKnightClimbProbe => {
            "issue260-drips-iron-knight-climb-probe"
        }
        RecordScenarioKind::Issue260DripsEarlyJumpCloseProbe => {
            "issue260-drips-early-jump-close-probe"
        }
        RecordScenarioKind::Issue260DripsDropContactProbe => "issue260-drips-drop-contact-probe",
        RecordScenarioKind::Issue260DlbTailBRow32Bounce => "issue260-dlb-tail-b-row32-bounce",
        RecordScenarioKind::Issue260GrPulse600 => "issue260-gr-pulse600",
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
        RecordScenarioKind::Issue249Blw2LeftPitWallKidNoJumpAudit => {
            "issue249-blw2-left-pit-wall-kid-no-jump-audit"
        }
        RecordScenarioKind::Issue249Blw2LeftPitWallRedStealthNoJumpAudit => {
            "issue249-blw2-left-pit-wall-red-stealth-no-jump-audit"
        }
        RecordScenarioKind::Issue271Level00DragonStompDamageAudit => {
            "issue271-level00-dragon-stomp-damage-audit"
        }
        RecordScenarioKind::Issue271Level01RockTankStompRouteAudit => {
            "issue271-level01-rock-tank-stomp-route-audit"
        }
        RecordScenarioKind::Issue271Level01RockTankStompSharpAudit => {
            "issue271-level01-rock-tank-stomp-sharp-audit"
        }
    };
    let mut command = format!(
        "cargo run --release -p wildside-cli -- record-scenario --slug {slug} --scenario {scenario} --level {level:02X}"
    );
    if let Some(frames) = walk_frames {
        let _ = write!(command, " --walk-frames {frames}");
    }
    command
}

fn record_scenario_file_command(slug: &str, scenario: &Path, input_specs: &[String]) -> String {
    let mut command = format!(
        "cargo run --release -p wildside-cli -- record-scenario-file --slug {slug} --scenario {}",
        scenario.display()
    );
    for input in input_specs {
        use std::fmt::Write;

        let _ = write!(command, " --input {input}");
    }
    command
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

#[derive(Debug, Serialize)]
struct AudioAuditRow {
    id: u8,
    name: String,
    chip_type: String,
    channels: Vec<String>,
    native_dur_frames: usize,
    native_dur_ms: u32,
    native_rms: f32,
    native_peak: f32,
    peak_freq_hz: Option<u32>,
    looping: bool,
    ym2612_writes: usize,
    psg_writes: usize,
    oracle_dur_frames: Option<usize>,
    oracle_dur_ms: Option<u32>,
    oracle_rms: Option<f32>,
    oracle_peak: Option<f32>,
    oracle_peak_freq_hz: Option<u32>,
    oracle_looping: Option<bool>,
    duration_delta_frames: Option<i32>,
    oracle_status: String,
}

#[derive(Debug, Default)]
struct AudioChipActivity {
    ym2612_writes: usize,
    psg_writes: usize,
    uses_dac: bool,
    uses_fm: bool,
}

fn run_audio_audit(
    repo_root: &Path,
    upstream_dir: &Path,
    manifest: &AssetManifest,
    rom_bytes: &[u8],
    sound_id: Option<u8>,
    frames: usize,
    sample_rate: u32,
    json: bool,
) -> Result<()> {
    if frames == 0 {
        bail!("audio-audit requires --frames > 0");
    }
    if sample_rate == 0 {
        bail!("audio-audit requires --sample-rate > 0");
    }

    let sound_dir = manifest.output_root.join("sound");
    let driver = read_sound_asset(&sound_dir, "GEMS_Sounddriver.bin")?;
    let instruments = read_sound_asset(&sound_dir, "instruments.bin")?;
    let envelopes = read_sound_asset(&sound_dir, "envelopes.bin")?;
    let sequences = read_sound_asset(&sound_dir, "sequences.bin")?;
    let samples = read_sound_asset(&sound_dir, "samples.bin")?;
    let names = parse_sfx_constants(&upstream_dir.join("constants.asm"))?;

    let ids: Vec<u8> = if let Some(id) = sound_id {
        vec![id]
    } else {
        names.keys().copied().collect()
    };

    let mut rows = Vec::with_capacity(ids.len());
    for id in ids {
        let name = names
            .get(&id)
            .map_or_else(|| format!("sfx_{id:02X}"), |labels| labels.join("|"));
        rows.push(audit_one_sound_id(
            id,
            name,
            frames,
            sample_rate,
            &driver,
            &instruments,
            &envelopes,
            &sequences,
            &samples,
            rom_bytes,
        ));
    }

    if json {
        println!("{}", serde_json::to_string_pretty(&rows)?);
    } else {
        println!("id   type   ndur odur diff  nrms     orms     loop  writes  name  oracle");
        for row in &rows {
            let oracle_dur = row
                .oracle_dur_frames
                .map_or_else(|| "-".to_string(), |dur| dur.to_string());
            let delta = row
                .duration_delta_frames
                .map_or_else(|| "-".to_string(), |delta| delta.to_string());
            let oracle_rms = row
                .oracle_rms
                .map_or_else(|| "-".to_string(), |rms| format!("{rms:.5}"));
            println!(
                "{:02X}   {:5}  {:3}  {:>3} {:>4}  {:.5}  {:>7}  {:5} {:>3}/{:<3} {}  {}",
                row.id,
                row.chip_type,
                row.native_dur_frames,
                oracle_dur,
                delta,
                row.native_rms,
                oracle_rms,
                row.looping,
                row.ym2612_writes,
                row.psg_writes,
                row.name,
                row.oracle_status,
            );
        }
        let asset_path = sound_dir
            .strip_prefix(repo_root)
            .unwrap_or(sound_dir.as_path());
        println!("assets: {}", asset_path.display());
    }
    Ok(())
}

fn read_sound_asset(sound_dir: &Path, filename: &str) -> Result<Vec<u8>> {
    let path = sound_dir.join(filename);
    fs::read(&path).with_context(|| format!("failed to read sound asset {}", path.display()))
}

fn parse_sfx_constants(path: &Path) -> Result<BTreeMap<u8, Vec<String>>> {
    let text = fs::read_to_string(path)
        .with_context(|| format!("failed to read ASM constants from {}", path.display()))?;
    let mut names: BTreeMap<u8, Vec<String>> = BTreeMap::new();
    for raw_line in text.lines() {
        let line = raw_line.split(';').next().unwrap_or_default().trim();
        let Some((name, value)) = line.split_once('=') else {
            continue;
        };
        let name = name.trim();
        if !name.starts_with("sfx_") {
            continue;
        }
        let value = value.trim();
        let id = if let Some(hex) = value.strip_prefix('$') {
            u8::from_str_radix(hex, 16)
        } else {
            value.parse()
        };
        let id = id.with_context(|| format!("failed to parse {name} sound id `{value}`"))?;
        names.entry(id).or_default().push(name.to_string());
    }
    Ok(names)
}

fn audit_one_sound_id(
    id: u8,
    name: String,
    frames: usize,
    sample_rate: u32,
    driver: &[u8],
    instruments: &[u8],
    envelopes: &[u8],
    sequences: &[u8],
    samples: &[u8],
    rom_bytes: &[u8],
) -> AudioAuditRow {
    let mut engine = GemsAudioEngine::new(driver, instruments, envelopes, sequences, samples);
    engine.start_sequence(id);
    let samples_per_frame = ((u64::from(sample_rate) + 30) / 60).max(1) as usize;
    let mut activity = AudioChipActivity::default();
    let native = render_standalone_gems_audit(
        &mut engine,
        frames,
        sample_rate,
        samples_per_frame,
        Some(&mut activity),
    );
    let oracle = render_rom_gems_audit(rom_bytes, id, frames, sample_rate, samples_per_frame);
    let (chip_type, channels) = summarise_chip_activity(&activity);
    let duration_delta_frames = oracle
        .as_ref()
        .ok()
        .map(|oracle| native.dur_frames as i32 - oracle.dur_frames as i32);
    let oracle_metrics = oracle.as_ref().ok();

    AudioAuditRow {
        id,
        name,
        chip_type,
        channels,
        native_dur_frames: native.dur_frames,
        native_dur_ms: native.dur_ms,
        native_rms: native.rms,
        native_peak: native.peak,
        peak_freq_hz: native.peak_freq_hz,
        looping: native.looping,
        ym2612_writes: activity.ym2612_writes,
        psg_writes: activity.psg_writes,
        oracle_dur_frames: oracle_metrics.map(|metrics| metrics.dur_frames),
        oracle_dur_ms: oracle_metrics.map(|metrics| metrics.dur_ms),
        oracle_rms: oracle_metrics.map(|metrics| metrics.rms),
        oracle_peak: oracle_metrics.map(|metrics| metrics.peak),
        oracle_peak_freq_hz: oracle_metrics.and_then(|metrics| metrics.peak_freq_hz),
        oracle_looping: oracle_metrics.map(|metrics| metrics.looping),
        duration_delta_frames,
        oracle_status: oracle
            .as_ref()
            .map_or_else(std::string::ToString::to_string, |_| "ok".to_string()),
    }
}

#[derive(Debug, Clone)]
struct AudioAuditMetrics {
    dur_frames: usize,
    dur_ms: u32,
    rms: f32,
    peak: f32,
    peak_freq_hz: Option<u32>,
    looping: bool,
}

fn render_standalone_gems_audit(
    engine: &mut GemsAudioEngine,
    frames: usize,
    sample_rate: u32,
    samples_per_frame: usize,
    mut activity: Option<&mut AudioChipActivity>,
) -> AudioAuditMetrics {
    let mut all_samples = Vec::with_capacity(frames * samples_per_frame * 2);
    let mut audible_frames = Vec::with_capacity(frames);
    for _ in 0..frames {
        let writes = engine.step_frame_with_audio_writes();
        if let Some(activity) = activity.as_deref_mut() {
            classify_audio_writes(&writes, activity);
        }
        let frame_samples = engine.generate_samples_at_rate(sample_rate, samples_per_frame);
        let frame_rms = rms(&frame_samples);
        audible_frames.push(frame_rms > AUDIBLE_RMS);
        all_samples.extend(frame_samples);
    }
    audio_audit_metrics(frames, sample_rate, &audible_frames, &all_samples)
}

fn render_rom_gems_audit(
    rom_bytes: &[u8],
    sound_id: u8,
    frames: usize,
    sample_rate: u32,
    samples_per_frame: usize,
) -> Result<AudioAuditMetrics> {
    let mut engine = GemsAudioEngine::from_kid_rom(rom_bytes)?;
    engine.start_sequence(sound_id);
    Ok(render_standalone_gems_audit(
        &mut engine,
        frames,
        sample_rate,
        samples_per_frame,
        None,
    ))
}

const AUDIBLE_RMS: f32 = 0.0005;

fn audio_audit_metrics(
    frames: usize,
    sample_rate: u32,
    audible_frames: &[bool],
    samples: &[f32],
) -> AudioAuditMetrics {
    let native_dur_frames = audible_frames
        .iter()
        .rposition(|audible| *audible)
        .map_or(0, |idx| idx + 1);
    let tail = (frames / 10).clamp(6, 30).min(frames);
    let looping = tail != 0
        && audible_frames[frames - tail..]
            .iter()
            .any(|audible| *audible);
    let native_rms = rms(samples);
    let native_peak = samples
        .iter()
        .fold(0.0_f32, |peak, sample| peak.max(sample.abs()));
    AudioAuditMetrics {
        dur_frames: native_dur_frames,
        dur_ms: ((native_dur_frames as u64 * 1000) / 60) as u32,
        rms: native_rms,
        peak: native_peak,
        peak_freq_hz: estimate_zero_crossing_frequency(samples, sample_rate),
        looping,
    }
}

fn classify_audio_writes(writes: &[GemsAudioWrite], activity: &mut AudioChipActivity) {
    let mut ym_addr = [0u8; 2];
    for write in writes {
        match write.kind {
            GemsAudioWriteKind::Psg => {
                activity.psg_writes += 1;
            }
            GemsAudioWriteKind::Ym2612 => {
                activity.ym2612_writes += 1;
                match write.port {
                    0 | 2 => ym_addr[usize::from(write.port / 2)] = write.value,
                    1 | 3 => {
                        let addr = ym_addr[usize::from(write.port / 2)];
                        if addr == 0x2A || addr == 0x2B {
                            activity.uses_dac = true;
                        } else {
                            activity.uses_fm = true;
                        }
                    }
                    _ => {}
                }
            }
        }
    }
}

fn summarise_chip_activity(activity: &AudioChipActivity) -> (String, Vec<String>) {
    let mut channels = Vec::new();
    if activity.uses_fm {
        channels.push("fm".to_string());
    }
    if activity.uses_dac {
        channels.push("dac".to_string());
    }
    if activity.psg_writes != 0 {
        channels.push("psg".to_string());
    }
    let chip_type = match (
        activity.uses_fm,
        activity.uses_dac,
        activity.psg_writes != 0,
    ) {
        (false, false, false) => "silent",
        (true, false, false) => "fm",
        (false, true, false) => "dac",
        (false, false, true) => "psg",
        _ => "mixed",
    };
    (chip_type.to_string(), channels)
}

fn rms(samples: &[f32]) -> f32 {
    if samples.is_empty() {
        return 0.0;
    }
    let sum_squares: f64 = samples
        .iter()
        .map(|sample| {
            let sample = f64::from(*sample);
            sample * sample
        })
        .sum();
    (sum_squares / samples.len() as f64).sqrt() as f32
}

fn estimate_zero_crossing_frequency(samples: &[f32], sample_rate: u32) -> Option<u32> {
    if samples.len() < 4 {
        return None;
    }
    let mut previous = 0.0_f32;
    let mut crossings = 0usize;
    let mut mono_samples = 0usize;
    for stereo in samples.chunks_exact(2) {
        let mono = (stereo[0] + stereo[1]) * 0.5;
        if mono.abs() < 0.001 {
            mono_samples += 1;
            continue;
        }
        if previous != 0.0 && mono.signum() != previous.signum() {
            crossings += 1;
        }
        previous = mono;
        mono_samples += 1;
    }
    if crossings < 2 || mono_samples == 0 {
        return None;
    }
    let seconds = mono_samples as f64 / f64::from(sample_rate);
    Some(((crossings as f64 / 2.0) / seconds).round() as u32)
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
                wildside_rom::default_rom_candidates(repo_root)
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
        AudioChipActivity, BatchProbeResult, BatchTargetBlockEvent, BatchTargetProjectileCell,
        BatchTargetProjectileDirection, DynamicCheckpoint, DynamicScenario, GameplayCapture,
        GameplayScenario, GameplayState, PlayerForm, RecordScenarioKind, ReferenceName,
        TargetBlockEventKind, TargetCell, TargetPoint, TargetProjectileDirection,
        analyse_audio_samples, apply_dynamic_scenario_input_overlays, audio_envelope_similarity,
        batch_probe_sound_ids, batch_probe_window_specs, build_controller_script,
        build_record_scenario, capture_native_gameplay_run, capture_native_gameplay_scenario,
        classify_audio_writes, compare_native_gameplay_with, compare_native_states_against_oracle,
        dynamic_scenario_with_frame_checkpoints, ensure_gameplay_level_layouts_with,
        ensure_level_layouts_with, ensure_play_native_level_layouts_with,
        find_gameplay_live_drifts, format_sound_id_list, frontier_kind_arg,
        gameplay_checkpoint_window, generated_batch_probe_candidates, native_trace_to_oracle_state,
        oracle_probe_watch_row_in_range, parse_batch_probe_candidate, parse_batch_probe_window,
        parse_batch_target_block_event, parse_batch_target_projectile_cell,
        parse_batch_target_projectile_direction, parse_button_mask, parse_input_spec,
        parse_level_id, parse_sfx_constants, play_native_record_route_target,
        prepare_generated_assets_with, record_scenario_command, record_scenario_file_command,
        recording_scenario_manifest, recording_start_for_dynamic_scenario,
        selected_reference_repos, sorted_batch_probe_results, summarise_chip_activity,
        target_block_event_from_frame,
    };
    use crate::native_player::validate_native_session_assets;
    use crate::oracle::{EnemyFacing, OracleEnemyState, OracleState};
    use crate::recording;
    use crate::scenarios::{
        DynamicStartPosition, NativeDynamicRunOptions, run_native_dynamic_scenario,
    };
    use crate::verify::verify_soak_level_10m;
    use std::cell::{Cell, RefCell};
    use std::fs;
    use std::path::{Path, PathBuf};
    use tempfile::TempDir;
    use wildside_core::{
        ActiveProjectileKind, GameMode, GameMode as NativeGameMode, LevelId, NativeObjectKind,
        NativeObjectSnapshot, NativeTraceFrame,
    };
    use wildside_data::{
        AssetManifest, export_split_bundle, load_manifest, manifest_path as generated_manifest_path,
    };
    use wildside_md::{GemsAudioWrite, GemsAudioWriteKind};
    use wildside_rom::{RomHeader, RomIdentity, VerifiedRom};

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
                .expect("wildside-cli crate should live under <repo>/crates/wildside-cli")
                .to_path_buf()
        };
        let manifest_path = repo_root
            .join(".cache/generated/28b904000b2863b6760531807760b571f1a5fc1d/asset-manifest.json");
        if !manifest_path.is_file() {
            eprintln!(
                "skipping cache-backed wildside-cli test: generated asset manifest missing at {}",
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
        fs::create_dir_all(source_root.join("ingame/palette")).expect("palette dir");
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
        fs::write(source_root.join("ingame/palette/Murder_wall.bin"), [0; 16])
            .expect("murder wall palette");
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
    fn parses_batch_probe_candidate_specs() {
        let candidate = parse_batch_probe_candidate("tail=478-524:left,525-529:none,530-540:b")
            .expect("candidate should parse");

        assert_eq!(candidate.label, "tail");
        assert_eq!(
            candidate.input_specs,
            vec!["478-524:left", "525-529:none", "530-540:b"]
        );
    }

    #[test]
    fn parses_batch_probe_window_specs() {
        let window =
            parse_batch_probe_window("turn=430-440:20:5:b+right").expect("window should parse");

        assert_eq!(window.label, "turn");
        assert_eq!(window.start_min, 430);
        assert_eq!(window.start_max, 440);
        assert_eq!(window.duration, 20);
        assert_eq!(window.step, 5);
        assert_eq!(window.button_spec, "b+right");
        assert_eq!(
            batch_probe_window_specs(&window),
            vec![
                (String::from("turn430"), String::from("430-449:b+right")),
                (String::from("turn435"), String::from("435-454:b+right")),
                (String::from("turn440"), String::from("440-459:b+right")),
            ]
        );
    }

    #[test]
    fn generates_batch_probe_candidates_from_window_products() {
        let windows = vec![
            parse_batch_probe_window("a=10-20:5:10:left").expect("first window"),
            parse_batch_probe_window("b=30-40:3:10:none").expect("second window"),
        ];

        let candidates =
            generated_batch_probe_candidates(&windows, 10).expect("candidates should generate");

        assert_eq!(candidates.len(), 4);
        assert_eq!(candidates[0].label, "a10_b30");
        assert_eq!(
            candidates[0].input_specs,
            vec![String::from("10-14:left"), String::from("30-32:none")]
        );
        assert_eq!(candidates[3].label, "a20_b40");
        assert_eq!(
            generated_batch_probe_candidates(&windows, 3)
                .expect_err("limit should reject generated products")
                .to_string(),
            "batch sweep windows would generate 4 candidates, above --max-generated 3"
        );
    }

    #[test]
    fn rejects_batch_probe_candidate_without_valid_input() {
        let err = parse_batch_probe_candidate("bad=not-a-range").expect_err("invalid candidate");

        assert!(
            err.to_string().contains("input spec must be"),
            "unexpected error: {err}"
        );
    }

    #[test]
    fn ranks_batch_probe_results_by_first_target_then_label() {
        let ranked = sorted_batch_probe_results(vec![
            BatchProbeResult {
                label: String::from("z-no-score"),
                final_mode: NativeGameMode::InGame,
                final_mode_value: 0x000C,
                lives: 3,
                hitpoints: 1,
                helmet_id: 0,
                sound_ids: Vec::new(),
                first_target_score: None,
                target_summaries: Vec::new(),
            },
            BatchProbeResult {
                label: String::from("b"),
                final_mode: NativeGameMode::InGame,
                final_mode_value: 0x000C,
                lives: 3,
                hitpoints: 1,
                helmet_id: 0,
                sound_ids: Vec::new(),
                first_target_score: Some(4),
                target_summaries: Vec::new(),
            },
            BatchProbeResult {
                label: String::from("a"),
                final_mode: NativeGameMode::InGame,
                final_mode_value: 0x000C,
                lives: 3,
                hitpoints: 1,
                helmet_id: 0,
                sound_ids: Vec::new(),
                first_target_score: Some(4),
                target_summaries: Vec::new(),
            },
            BatchProbeResult {
                label: String::from("best"),
                final_mode: NativeGameMode::InGame,
                final_mode_value: 0x000C,
                lives: 3,
                hitpoints: 1,
                helmet_id: 0,
                sound_ids: Vec::new(),
                first_target_score: Some(2),
                target_summaries: Vec::new(),
            },
        ]);

        assert_eq!(
            ranked
                .iter()
                .map(|result| result.label.as_str())
                .collect::<Vec<_>>(),
            vec!["best", "a", "b", "z-no-score"]
        );
    }

    #[test]
    fn batch_probe_sound_ids_reports_unique_sorted_sfx_ids() {
        let mut trace = vec![NativeTraceFrame::default(); 3];
        trace[0].played_sound_effect_ids = vec![0x2E, 0x17];
        trace[1].played_sound_effect_ids = vec![0x17, 0x41];

        assert_eq!(batch_probe_sound_ids(&trace), vec![0x17, 0x2E, 0x41]);
        assert_eq!(
            format_sound_id_list(&[0x17, 0x2E, 0x41]),
            "[0x17,0x2E,0x41]"
        );
        assert_eq!(format_sound_id_list(&[]), "[]");
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
            start_velocity: None,
            camera_position: None,
            playtest_form: None,
            initial_diamond_count: None,
            initial_lives: None,
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
            start_velocity: None,
            camera_position: None,
            playtest_form: None,
            initial_diamond_count: None,
            initial_lives: None,
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
    fn dynamic_frame_capture_checkpoints_are_sorted_unique_and_extend_script() {
        let scenario = DynamicScenario {
            slug: String::from("test-frame-capture"),
            script: vec![0, 1 << 3],
            frame_count: 2,
            checkpoints: vec![DynamicCheckpoint {
                label: String::from("old"),
                frame_index: 1,
            }],
            start_position: None,
            start_velocity: None,
            camera_position: None,
            playtest_form: None,
            initial_diamond_count: None,
            initial_lives: None,
            warp_to_level: None,
            settle_oracle_warp: false,
        };

        let scenario = dynamic_scenario_with_frame_checkpoints(scenario, &[5, 2, 5]);

        assert_eq!(scenario.frame_count, 6);
        assert_eq!(scenario.script.len(), 6);
        assert_eq!(scenario.script[1], 1 << 3);
        assert_eq!(
            scenario.checkpoints,
            vec![
                DynamicCheckpoint {
                    label: String::from("frame_00002"),
                    frame_index: 2,
                },
                DynamicCheckpoint {
                    label: String::from("frame_00005"),
                    frame_index: 5,
                },
            ]
        );
    }

    #[test]
    fn dynamic_record_scenario_input_none_clears_existing_script_range() {
        let scenario = DynamicScenario {
            slug: String::from("test-overlay-clear"),
            script: vec![1 << 3, 1 << 3, (1 << 3) | (1 << 4), 1 << 2],
            frame_count: 4,
            checkpoints: Vec::new(),
            start_position: None,
            start_velocity: None,
            camera_position: None,
            playtest_form: None,
            initial_diamond_count: None,
            initial_lives: None,
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
            two_player: false,
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
            rng_last_byte: None,
            rng_ring: None,
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
            time_frames_loc_3038a: 0,
            background_scroll_hidden_wrap_display_phase_loc_3050a: false,
            transform_hscroll_owner_loc_8fe: false,
            transform_auto_scroll_uses_previous_word_loc_7e90: false,
            island_shore_post_visible_dma_latch_loc_1202a: false,
            floating_items_transferred_loc_901: Vec::new(),
            hud_dma_events_sub_44dc: Vec::new(),
            hud_raster_upload_timings_sub_44dc: Vec::new(),
            mode_level_vblank_work_before_sub_e12: None,
            mode_level_work_before_hud_sub_44dc: None,
            mode_level_work_after_hud_sub_44dc: None,
            mode_level_tail_work_loc_892: None,
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
            cyclone_flying: false,
            cyclone_grounded_sprite_source: None,
            paused: false,
            pause_menu_option: 0,
            kid_x_pos: Some(0x0020_0000),
            kid_y_pos: Some(0x0010_0000),
            kid_x_velocity: Some(0),
            kid_y_velocity: Some(0),
            airborne_visual_mapping_sub_b270:
                wildside_core::ModeLevelAirborneVisualMappingPathSubB270::Unclassified,
            character_movement: None,
            kid_facing: None,
            kid_on_ground: Some(true),
            walk_visual_accumulator: 0,
            iron_knight_climbing: false,
            iron_knight_climb_pose_accum: 0,
            hud_diamond_frame_offset: None,
            hud_diamond_vram_page_loc_5d4a: None,
            transform_resident_hud_diamond_vram_page_loc_5d4a: None,
            hud_timer_sprite_seconds_loc_bee8: None,
            hud_diamond_count_loc_bdc0: None,
            hud_life_frame_offset: None,
            hud_life_icon_vram_page_loc_4586: None,
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
                    facing: Some(wildside_core::physics::Direction::Left),
                    on_ground: Some(false),
                    hit_points: Some(1),
                    damage_flash: false,
                    palette_variant: 0,
                    stationary: false,
                    contact_phase: None,
                    shared_walker_state: None,
                    shared_walker_patrol_timer: None,
                    shared_walker_patrol_short_next: None,
                    walk_phase: None,
                    sprite_animation_phase: None,
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
                    facing: Some(wildside_core::physics::Direction::Right),
                    on_ground: Some(true),
                    hit_points: Some(1),
                    damage_flash: false,
                    palette_variant: 0,
                    stationary: false,
                    contact_phase: None,
                    shared_walker_state: None,
                    shared_walker_patrol_timer: None,
                    shared_walker_patrol_short_next: None,
                    walk_phase: None,
                    sprite_animation_phase: None,
                    wait_timer: None,
                    debug_fields: std::collections::BTreeMap::new(),
                },
            ],
            active_enemy_details: Vec::new(),
            camera_x_pos: 0x0010_0000,
            camera_y_pos: 0x0008_0000,
            resident_gameplay_vdp_camera_loc_300ba: None,
            resident_gameplay_vdp_terminal_scroll_table_loc_28c6: false,
            number_objects: 3,
            number_gfx_objects: 3,
            rng_offset: None,
            rng_last_byte: None,
            rng_ring: None,
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
            level_timer_sub_frames: 0,
            level_damage_taken: false,
            level_prize_collected: false,
            level_speed_bonus_awarded: false,
            level_path_bonus_amount: 0,
            murder_wall_x: None,
            active_projectiles: Vec::new(),
            active_projectile_details: Vec::new(),
            diamond_power_sprites: Vec::new(),
            eyeclops_hard_beam_object: None,
            active_geysers: Vec::new(),
            lava_geyser_script_templates_loc_147ba: Vec::new(),
            island_shore_cycle_active: false,
            island_shore_cycle_state_loc_1202a: None,
            island_shore_vram_frames_loc_1202a: None,
            transform_resident_island_shore_vram_frames_loc_1202a: None,
            island_shore_dma_uploads_loc_1202a: Vec::new(),
            lava_geyser_art_uploads_loc_14c6a: Vec::new(),
            rotate_palette_lava_water_state_loc_1223c: None,
            lava_water_palette_words_loc_1223c: None,
            lava_geyser_palette_words_loc_14b2a: None,
            storm_pattern_upload_loc_14d32: None,
            storm_scroll_transfer_loc_30110: None,
            storm_shared_art_upload_loc_14d8a: None,
            special_effect_unknown_art_upload_loc_14df6: None,
            special_effect_unknown_plane_fill_loc_14e42: None,
            storm_falling_slots_sub_61ca: Vec::new(),
            storm_impact_slots_sub_61ca: Vec::new(),
            storm_sprites_sub_61ca: Vec::new(),
            block_bounces: Vec::new(),
            block_bounce_details: Vec::new(),
            solid_ghost_transitions_sub_f730: Vec::new(),
            block_destroy_effects: Vec::new(),
            block_debris_pieces: Vec::new(),
            prize_block_worker_tails_sub_1007a: Vec::new(),
            prize_block_spawn_flashes: Vec::new(),
            kid_projectile_hit_sparks: Vec::new(),
            floating_items: Vec::new(),
            prize_diamond_hud_flights_loc_fe3e: Vec::new(),
            prize_diamond_hud_flight_transferred_loc_fe3e: false,
            clock_prize_animation_swaps_loc_10796: Vec::new(),
            continue_prize_animation_swaps_loc_1080a: Vec::new(),
            enemy_death_effects: Vec::new(),
            heady_metal_death_splits: Vec::new(),
            heady_metal_body_pieces_sub_37abc: Vec::new(),
            stomped_this_frame: false,
            death_windup_elapsed: 0,
            form_change_flash_frames: 0,
            hud_transform_target_helmet_id: None,
            block_destroy_ice_effects: Vec::new(),
            transform_source_helmet_id_loc_7ca6: None,
            transform_source_hitpoints_loc_bdc0: None,
            blocks_hit_this_frame: 0,
            sound_test_track: None,
            sound_test_play_pending: false,
            gameplay_debug_fields: std::collections::BTreeMap::new(),
            physics_trace: None,
            options_cursor: None,
            played_sound_effect_ids: Vec::new(),
            played_sound_commands: Vec::new(),
            played_bgm_ids: Vec::new(),
            played_music_speed_updates: Vec::new(),
            teleport_warp_colour: 0,
            level_entry_fade_progress_sub_73d0: None,
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
        fs::write(layout_dir.join("00.blocklist.bin"), []).expect("existing block list");

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
                fs::write(out_dir.join(format!("{level_id:02X}.blocklist.bin")), [])
                    .expect("dumped block list");
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
            fs::write(layout_dir.join(format!("{level_id:02X}.blocklist.bin")), [])
                .expect("existing block list");
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
                fs::write(out_dir.join(format!("{level_id:02X}.blocklist.bin")), [])
                    .expect("dumped block list");
                Ok(())
            },
        )
        .expect("oracle-warp-broken level should be skipped");

        assert!(dumped_levels.borrow().is_empty());
        assert!(!layout_dir.join("09.bin").exists());
        assert!(!layout_dir.join("09.json").exists());
    }

    #[test]
    fn ensure_play_native_level_layouts_warms_debug_skip_targets() {
        let bundle_root = TempDir::new().expect("bundle root");
        let layout_dir = bundle_root.path().join("level-layout");
        fs::create_dir_all(&layout_dir).expect("layout dir");
        for level_id in 0x00u8..=0x02 {
            fs::write(layout_dir.join(format!("{level_id:02X}.bin")), [0x60, 0x00])
                .expect("existing layout");
            fs::write(
                layout_dir.join(format!("{level_id:02X}.json")),
                br#"{"width_blocks":200,"height_blocks":28,"data_bytes":2}"#,
            )
            .expect("existing layout metadata");
            fs::write(layout_dir.join(format!("{level_id:02X}.blocklist.bin")), [])
                .expect("existing block list");
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
        ensure_play_native_level_layouts_with(b"rom", &manifest, None, |_, level_id, out_dir| {
            dumped_levels.borrow_mut().push(level_id);
            fs::write(out_dir.join(format!("{level_id:02X}.bin")), [0x60, 0x00])
                .expect("dumped layout");
            fs::write(
                out_dir.join(format!("{level_id:02X}.json")),
                br#"{"width_blocks":200,"height_blocks":28,"data_bytes":2}"#,
            )
            .expect("dumped layout metadata");
            fs::write(out_dir.join(format!("{level_id:02X}.blocklist.bin")), [])
                .expect("dumped block list");
            Ok(())
        })
        .expect("play-native should preflight debug-skip layout targets");

        let dumped_levels = dumped_levels.borrow();
        assert!(
            dumped_levels.contains(&0x04),
            "play-native N/P should preflight Under Skull Mountain 1 layout data"
        );
        assert!(
            dumped_levels.contains(&0x07),
            "play-native N/P should preflight Isle of the Lion Lord layout data"
        );
        assert!(
            dumped_levels.contains(&0x1B),
            "play-native N/P should preflight Skydragon Castle layout data"
        );
        assert!(
            !dumped_levels.contains(&0x09),
            "known oracle-warp-broken levels should still be skipped during broad preflight"
        );
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
        fs::write(layout_dir.join("00.blocklist.bin"), []).expect("existing block list");

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
            fs::write(out_dir.join(format!("{level_id:02X}.blocklist.bin")), [])
                .expect("dumped block list");
            Ok(())
        })
        .expect("missing requested layouts should be dumped");

        assert_eq!(*dumped_levels.borrow(), vec![0x02]);
        assert!(layout_dir.join("02.bin").is_file());
        assert!(layout_dir.join("02.json").is_file());
        assert!(layout_dir.join("02.blocklist.bin").is_file());
        assert!(!layout_dir.join("01.bin").exists());
    }

    #[test]
    fn ensure_level_layouts_redumps_when_block_list_is_missing() {
        let bundle_root = TempDir::new().expect("bundle root");
        let layout_dir = bundle_root.path().join("level-layout");
        fs::create_dir_all(&layout_dir).expect("layout dir");
        fs::write(layout_dir.join("14.bin"), [0x60, 0x00]).expect("existing layout");
        fs::write(
            layout_dir.join("14.json"),
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
        ensure_level_layouts_with(b"rom", &manifest, &[0x14], |_, level_id, out_dir| {
            dumped_levels.borrow_mut().push(level_id);
            fs::write(out_dir.join(format!("{level_id:02X}.bin")), [0x60, 0x00])
                .expect("dumped layout");
            fs::write(
                out_dir.join(format!("{level_id:02X}.json")),
                br#"{"width_blocks":200,"height_blocks":28,"data_bytes":2}"#,
            )
            .expect("dumped layout metadata");
            fs::write(
                out_dir.join(format!("{level_id:02X}.blocklist.bin")),
                [0x00, 0x00],
            )
            .expect("dumped block list");
            Ok(())
        })
        .expect("missing block list should redump the layout bundle");

        assert_eq!(*dumped_levels.borrow(), vec![0x14]);
        assert!(layout_dir.join("14.blocklist.bin").is_file());
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
    fn gameplay_regression_playtest_segment_replays_deterministically() {
        // A playtest segment starts from a fresh warp (playtest_warp_session)
        // and must replay bit-identically through the verify-recording native
        // path from its saved `{warp_to_level}` scenario. A drifting frame
        // here means paused/live-only state leaked into the segment.
        let Some(manifest) = cached_generated_manifest() else {
            return;
        };
        let options = crate::NativePlayOptions {
            fullscreen: false,
            scale: None,
            debug_overlay: false,
            warp_level: None,
            settled: false,
            record_to: None,
            record_frames: None,
            record_stop_on_target_enemy_contact: false,
            record_contact_tail_frames: 30,
            record_notes: None,
            record_route_target: None,
            record_frontier_kind: None,
            scripted_route: None,
            playtest: None,
        };
        let (_assets, mut runtime) =
            crate::native_player::playtest_warp_session(&manifest, &options, 0x00)
                .expect("playtest warp session");

        let mut script = vec![0u8; 30];
        for buttons in script.iter_mut().skip(5).take(12) {
            *buttons = 1 << 3; // hold Right, as a live playtester would
        }
        let live_trace: Vec<_> = script
            .iter()
            .map(|&buttons| runtime.step_frame(wildside_core::FrameInput::from_mask(buttons)))
            .collect();

        let start = crate::recording::RecordingStart {
            warp_level: Some(0x00),
            settled: false,
            record_frames: None,
        };
        let recording_bundle = crate::recording::build_recording(
            &manifest,
            "pt-test-seg01",
            start,
            script,
            live_trace.clone(),
        )
        .expect("build segment recording");
        let replay = crate::recording::run_recorded_trace(
            &manifest,
            start,
            &recording_bundle.manifest.scenario,
        )
        .expect("native replay");

        assert_eq!(replay.len(), live_trace.len());
        for (frame_index, (live, replayed)) in live_trace.iter().zip(&replay).enumerate() {
            assert_eq!(
                live, replayed,
                "playtest segment frame {frame_index} must replay identically"
            );
        }
    }

    #[test]
    fn gameplay_regression_native_gameplay_capture_uses_default_runtime_path() {
        let (_source_root, _bundle_root, manifest) = exported_test_manifest();
        let capture =
            capture_native_gameplay_scenario(&manifest, GameplayScenario::DeathRespawnLevel1)
                .expect("capture");
        assert_eq!(capture.checkpoints.len(), 11);
        assert!(
            capture
                .checkpoints
                .iter()
                .any(|state| state.label == "respawn_bootstrap_tail_26000"),
            "default native gameplay capture should reach the extended hidden respawn checkpoint"
        );
    }

    #[test]
    fn gameplay_regression_pause_resume_native_capture_carries_title_start_loc_1bbde_1ae2e() {
        let (_source_root, _bundle_root, manifest) = exported_test_manifest();
        let capture =
            capture_native_gameplay_scenario(&manifest, GameplayScenario::PauseResumeLevel1)
                .expect("capture");
        let level_settle = capture
            .checkpoints
            .iter()
            .find(|state| state.label == "level_settle")
            .expect("level_settle checkpoint");
        let pre_pause = capture
            .checkpoints
            .iter()
            .find(|state| state.label == "pre_pause")
            .expect("pre_pause checkpoint");
        let paused_tail = capture
            .checkpoints
            .iter()
            .find(|state| state.label == "paused_tail")
            .expect("paused_tail checkpoint");

        // ASM Load_TitleCard / loc_1AE2E keeps the cold-boot route in mode
        // $0008 through frame 719; the direct harness releases it on the next
        // input frame so pre_pause still samples live Mode_Level.
        assert_eq!(level_settle.game_mode, 0x0008);
        assert_eq!(pre_pause.game_mode, 0x000C);
        assert!(!pre_pause.game_paused);
        assert_eq!(paused_tail.game_mode, 0x000C);
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
    fn gameplay_regression_live_drift_skips_shared_level_transition_shells() {
        let mut native_shell = gameplay_state("frame_00922", 922);
        native_shell.game_mode = GameMode::LevelTransition.as_disassembly_value();
        native_shell.camera_y_pos = 0x0073_0000;
        native_shell.character_movement = Some(0);
        let mut oracle_shell = native_shell.clone();
        oracle_shell.camera_y_pos = 0;
        oracle_shell.character_movement = Some(2);

        let mut native_live = gameplay_state("frame_00933", 933);
        native_live.game_mode = GameMode::InGame.as_disassembly_value();
        let mut oracle_still_shell = native_live.clone();
        oracle_still_shell.game_mode = GameMode::LevelTransition.as_disassembly_value();

        let native = GameplayCapture {
            scenario: String::from("progress_two_transitions"),
            frame_count: 934,
            missed_vblank_frames: 0,
            states: vec![native_shell, native_live],
            checkpoints: Vec::new(),
        };
        let oracle = GameplayCapture {
            scenario: String::from("progress_two_transitions"),
            frame_count: 934,
            missed_vblank_frames: 0,
            states: vec![oracle_shell, oracle_still_shell],
            checkpoints: Vec::new(),
        };

        let reports = find_gameplay_live_drifts(
            &native,
            &oracle,
            GameplayScenario::ProgressTwoTransitions,
            900,
        );

        assert_eq!(reports.len(), 1);
        assert_eq!(reports[0].frame_index, 933);
        assert_eq!(reports[0].differing_fields, vec!["game_mode"]);
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
                    game_paused: false,
                    pause_menu_option: 0,
                    options_controls_scheme: 0,
                    options_speed_button_fast: false,
                    invulnerable_frames: Some(0),
                    just_received_damage: Some(false),
                    rng_offset: None,
                    rng_ring: None,
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
                    game_paused: false,
                    pause_menu_option: 0,
                    options_controls_scheme: 0,
                    options_speed_button_fast: false,
                    invulnerable_frames: Some(0),
                    just_received_damage: Some(false),
                    rng_offset: None,
                    rng_ring: None,
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
            game_paused: false,
            pause_menu_option: 0,
            options_controls_scheme: 0,
            options_speed_button_fast: false,
            invulnerable_frames: Some(0),
            just_received_damage: Some(false),
            rng_offset: None,
            rng_ring: None,
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
    fn build_record_scenario_level_00_slope_jump_probe_uses_named_window() {
        let scenario =
            build_record_scenario(RecordScenarioKind::Level00SlopeRightJumpProbe, 0x05, None);

        assert_eq!(scenario.slug, "level_00_slope_right_jump_probe");
        assert_eq!(
            scenario.warp_to_level,
            Some(0x00),
            "named Level 00 slope jump probe must not accidentally follow the generic --level argument"
        );
        assert!(scenario.settle_oracle_warp);
        assert_eq!(
            scenario.script[0], 0,
            "explicit probe starts need one neutral frame before native/oracle input comparison"
        );
        assert_eq!(
            scenario.script[13],
            (1 << 3) | (1 << 4),
            "ASM loc_9F18 should see a held-Right B press on the active slope window"
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
    fn build_record_scenario_highwater_flag_floor_contact_audit_walks_into_flag() {
        let scenario = build_record_scenario(
            RecordScenarioKind::HighwaterFlagFloorContactAudit,
            0x00,
            None,
        );

        assert_eq!(scenario.slug, "highwater_flag_floor_contact_audit_02");
        assert_eq!(
            scenario.warp_to_level,
            Some(0x02),
            "the #267 floor-contact probe must target Highwater Pass 1 regardless of the generic --level argument"
        );
        assert!(scenario.settle_oracle_warp);
        assert_eq!(
            scenario
                .start_position
                .map(|position| (position.x, position.y)),
            Some((888, 144)),
            "the #267 route starts left of Highwater Pass 1's flag at (912, 144) so contact comes from walking into the pole"
        );
        assert_eq!(
            scenario.script.first().copied(),
            Some(1 << 3),
            "the #267 route must begin by holding Right, not by spawning inside the flag hitbox"
        );
        assert_eq!(
            scenario.script.get(90).copied(),
            Some(0),
            "the #267 route releases Right after the approach window so the level-complete hand-off can settle"
        );
        assert!(
            scenario.checkpoints.iter().any(|checkpoint| {
                checkpoint.label == "floor_flag_contact_window_loc_31d7a"
                    && checkpoint.frame_index == 45
            }),
            "issue #267 needs a durable checkpoint around the ROM flag-floor-contact window"
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
    fn build_record_scenario_issue271_stomp_audits_ignore_generic_level_arg() {
        let dragon = build_record_scenario(
            RecordScenarioKind::Issue271Level00DragonStompDamageAudit,
            0x7F,
            None,
        );
        let tank = build_record_scenario(
            RecordScenarioKind::Issue271Level01RockTankStompRouteAudit,
            0x7F,
            None,
        );

        assert_eq!(
            dragon.slug,
            "issue_271_level00_dragon_stomp_damage_audit_00"
        );
        assert_eq!(
            dragon.warp_to_level,
            Some(0x00),
            "the named #271 Dragon audit preserves today's Level 00 side-graze route"
        );
        assert!(
            dragon.checkpoints.iter().any(|checkpoint| checkpoint.label
                == "dragon_graze_damage_window_loc_1c76"
                && checkpoint.frame_index == 137),
            "ASM GfxObjects_CollisionKid loc_1C76 side-contact window stays named until a sharper stomp route exists"
        );

        assert_eq!(
            tank.slug,
            "issue_271_level01_rock_tank_stomp_route_audit_01"
        );
        assert_eq!(
            tank.warp_to_level,
            Some(0x01),
            "the named #271 Tank audit preserves today's Level 01 miss route"
        );
        assert!(
            tank.checkpoints
                .iter()
                .any(|checkpoint| checkpoint.label == "closest_tank_pass_miss"
                    && checkpoint.frame_index == 171),
            "current generic Level 01 route misses the Tank collision box and must be sharpened before gameplay changes"
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
        assert!(
            scenario.script[..60].iter().all(|&buttons| buttons == 0),
            "the direct BLW2 no-jump wall probe must settle before holding LEFT"
        );
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
    // Pinned regression for #249 (no-jump wall contact must not inject upward
    // velocity). Backlog #291 re-derived the old failing tail sample: by then
    // the route has walked off support, so the loc_ABA4 wall-contact invariant
    // belongs on the contact/hold window only.
    fn gameplay_regression_issue249_blw2_left_pit_wall_no_jump_does_not_auto_launch_juggernaut() {
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

        // ASM kid_flip -> loc_ABA4 gates the ledge pull-up to The Kid. The
        // reported no-jump Rhino/Juggernaut wall contact must therefore stay
        // out of the ledge snap path unless a separate form-specific path is
        // found.  The tail frame is allowed to be airborne: the route can walk
        // off support after the hold window, so grounded/zero-velocity is not
        // the invariant this regression is meant to protect.
        for checkpoint in [contact, hold] {
            assert_eq!(checkpoint.current_level, LevelId(0x01));
            assert_eq!(checkpoint.current_helmet_id, 5);
            assert_eq!(checkpoint.ledge_snap_cooldown, 0);
            assert!(
                checkpoint.kid_y_velocity.unwrap_or_default() >= 0,
                "no-jump BLW2 left-pit wall probe should not inject upward velocity: frame {} y_vel={:?}",
                checkpoint.frame_index,
                checkpoint.kid_y_velocity
            );
        }
        assert_eq!(tail.current_level, LevelId(0x01));
        assert_eq!(tail.current_helmet_id, 5);
        assert_eq!(tail.ledge_snap_cooldown, 0);
        assert!(
            contact
                .kid_y_pos
                .is_some_and(|y| (0x009F_0000..=0x00A0_0000).contains(&y)),
            "contact frame should stay near the reported shelf without an upward launch: {:?}",
            contact.kid_y_pos
        );
        assert!(
            hold.kid_y_pos <= contact.kid_y_pos.map(|y| y + 0x0001_0000),
            "hold frame should remain near the contact shelf without an upward launch: contact={:?} hold={:?}",
            contact.kid_y_pos,
            hold.kid_y_pos
        );
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
    fn issue249_blw2_left_pit_wall_no_jump_does_not_auto_launch_kid_or_samurai() {
        let Some(manifest) = cached_generated_manifest() else {
            return;
        };

        for (kind, expected_helmet) in [
            (RecordScenarioKind::Issue249Blw2LeftPitWallKidNoJumpAudit, 0),
            (
                RecordScenarioKind::Issue249Blw2LeftPitWallRedStealthNoJumpAudit,
                3,
            ),
        ] {
            let scenario = build_record_scenario(kind, 0x00, None);
            let trace = run_native_dynamic_scenario(
                manifest.clone(),
                &scenario,
                NativeDynamicRunOptions {
                    settled_warp: false,
                    trace_physics: true,
                },
            )
            .expect("native BLW2 Kid/Samurai left-wall probe should run");

            for index in [30, 90, 179] {
                let checkpoint = trace
                    .get(index)
                    .unwrap_or_else(|| panic!("missing native scenario frame {index}"));
                assert_eq!(checkpoint.current_level, LevelId(0x01));
                assert_eq!(checkpoint.current_helmet_id, expected_helmet);
                assert_eq!(
                    checkpoint.ledge_snap_cooldown, 0,
                    "ASM kid_flip -> loc_ABA4 must not enter ledge snap without a B edge"
                );

                let collision_probe = checkpoint
                    .gameplay_debug_fields
                    .get("kid_collision_handoff_probe")
                    .expect("collision handoff probe");
                assert!(
                    collision_probe.contains("jump_buffer=0"),
                    "no-jump BLW2 probe should not carry a stale jump buffer at frame {index}: {collision_probe}"
                );

                let special_probe = checkpoint
                    .gameplay_debug_fields
                    .get("kid_special_motion_probe")
                    .expect("special motion probe");
                assert!(
                    special_probe.contains("iron_knight_climbing=0")
                        && special_probe.contains("skycutter_on_board=0")
                        && special_probe.contains("cyclone_flying=0"),
                    "no-jump BLW2 probe should not enter a helmet climb/fly path at frame {index}: {special_probe}"
                );
            }
        }
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
            "cargo run --release -p wildside-cli -- record-scenario --slug issue-136-prize-emergence --scenario prize-block --level 00 --walk-frames 420"
        );
    }

    #[test]
    fn record_scenario_command_hex_level_is_parseable() {
        let command = record_scenario_command(
            "issue280-bloody-swamp",
            RecordScenarioKind::Idle,
            0x3E,
            None,
        );
        let level = command
            .split_whitespace()
            .skip_while(|part| *part != "--level")
            .nth(1)
            .expect("record command should include a level argument");

        assert_eq!(
            parse_level_id(level).expect("printed record-scenario level should parse"),
            0x3E
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
            "cargo run --release -p wildside-cli -- record-scenario --slug issue-216-prize-audio-audit --scenario issue216-prize-audio-audit --level 00"
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
            "cargo run --release -p wildside-cli -- record-scenario --slug issue-213-drips-metal-block-audit --scenario issue213-drips-metal-block-audit --level 01"
        );
    }

    #[test]
    fn record_scenario_command_exposes_issue260_drips_cleaner_prefix_audit() {
        let command = record_scenario_command(
            "issue-260-drips-cleaner-prefix-audit",
            RecordScenarioKind::Issue260DripsCleanerPrefixAudit,
            0x01,
            None,
        );

        assert_eq!(
            command,
            "cargo run --release -p wildside-cli -- record-scenario --slug issue-260-drips-cleaner-prefix-audit --scenario issue260-drips-cleaner-prefix-audit --level 01"
        );
    }

    #[test]
    fn record_scenario_command_exposes_issue260_drips_b_jump_climb_seed() {
        let command = record_scenario_command(
            "issue-260-drips-b-jump-climb-seed",
            RecordScenarioKind::Issue260DripsBJumpClimbSeed,
            0x01,
            None,
        );

        assert_eq!(
            command,
            "cargo run --release -p wildside-cli -- record-scenario --slug issue-260-drips-b-jump-climb-seed --scenario issue260-drips-b-jump-climb-seed --level 01"
        );
    }

    #[test]
    fn record_scenario_command_exposes_issue260_drips_early_jump_close_probe() {
        let command = record_scenario_command(
            "issue-260-drips-early-jump-close-probe",
            RecordScenarioKind::Issue260DripsEarlyJumpCloseProbe,
            0x01,
            None,
        );

        assert_eq!(
            command,
            "cargo run --release -p wildside-cli -- record-scenario --slug issue-260-drips-early-jump-close-probe --scenario issue260-drips-early-jump-close-probe --level 01"
        );
    }

    #[test]
    fn record_scenario_command_exposes_issue260_dlb_tail_b_row32_bounce() {
        let command = record_scenario_command(
            "issue-260-dlb-tail-b-row32-bounce",
            RecordScenarioKind::Issue260DlbTailBRow32Bounce,
            0x01,
            None,
        );

        assert_eq!(
            command,
            "cargo run --release -p wildside-cli -- record-scenario --slug issue-260-dlb-tail-b-row32-bounce --scenario issue260-dlb-tail-b-row32-bounce --level 01"
        );
    }

    #[test]
    fn record_scenario_command_exposes_issue260_gr_pulse600() {
        let command = record_scenario_command(
            "issue-260-gr-pulse600",
            RecordScenarioKind::Issue260GrPulse600,
            0x01,
            None,
        );

        assert_eq!(
            command,
            "cargo run --release -p wildside-cli -- record-scenario --slug issue-260-gr-pulse600 --scenario issue260-gr-pulse600 --level 01"
        );
    }

    #[test]
    fn record_scenario_command_exposes_issue271_stomp_audits() {
        let dragon_command = record_scenario_command(
            "issue-271-level00-dragon-stomp-damage-audit",
            RecordScenarioKind::Issue271Level00DragonStompDamageAudit,
            0x00,
            None,
        );
        let tank_command = record_scenario_command(
            "issue-271-level01-rock-tank-stomp-route-audit",
            RecordScenarioKind::Issue271Level01RockTankStompRouteAudit,
            0x01,
            None,
        );
        let sharp_tank_command = record_scenario_command(
            "issue-271-level01-rock-tank-stomp-sharp-audit",
            RecordScenarioKind::Issue271Level01RockTankStompSharpAudit,
            0x01,
            None,
        );

        assert_eq!(
            dragon_command,
            "cargo run --release -p wildside-cli -- record-scenario --slug issue-271-level00-dragon-stomp-damage-audit --scenario issue271-level00-dragon-stomp-damage-audit --level 00"
        );
        assert_eq!(
            tank_command,
            "cargo run --release -p wildside-cli -- record-scenario --slug issue-271-level01-rock-tank-stomp-route-audit --scenario issue271-level01-rock-tank-stomp-route-audit --level 01"
        );
        assert_eq!(
            sharp_tank_command,
            "cargo run --release -p wildside-cli -- record-scenario --slug issue-271-level01-rock-tank-stomp-sharp-audit --scenario issue271-level01-rock-tank-stomp-sharp-audit --level 01"
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
            "cargo run --release -p wildside-cli -- record-scenario --slug issue-211-samurai-natural-slope --scenario walk-right-red-stealth --level 00 --walk-frames 700"
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
            start_velocity: None,
            camera_position: None,
            playtest_form: None,
            initial_diamond_count: None,
            initial_lives: None,
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
            start_velocity: None,
            camera_position: None,
            playtest_form: None,
            initial_diamond_count: None,
            initial_lives: None,
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
            start_velocity: None,
            camera_position: None,
            playtest_form: None,
            initial_diamond_count: None,
            initial_lives: None,
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
    fn play_native_record_route_target_collects_named_route_metadata() {
        let target = play_native_record_route_target(
            Some(String::from("usm3-0x06-slope")),
            Some(0x06),
            &[TargetPoint {
                x_px: 2848,
                y_px: 512,
            }],
            &[TargetCell { col: 178, row: 32 }],
            &[TargetCell { col: 179, row: 32 }],
            &[0x0A],
        )
        .expect("record target")
        .expect("target metadata");

        assert_eq!(target.label.as_deref(), Some("usm3-0x06-slope"));
        assert_eq!(target.level, Some(0x06));
        assert_eq!(target.points[0].x_px, 2848);
        assert_eq!(target.cells[0].col, 178);
        assert_eq!(target.cell_hitboxes[0].col, 179);
        assert_eq!(target.enemies, vec![0x0A]);
    }

    #[test]
    fn play_native_record_route_target_omits_empty_metadata() {
        let target = play_native_record_route_target(None, None, &[], &[], &[], &[])
            .expect("empty target metadata");

        assert_eq!(target, None);
    }

    #[test]
    fn play_native_record_route_target_accepts_each_single_field_metadata() {
        let label = play_native_record_route_target(
            Some(String::from("slope-route")),
            None,
            &[],
            &[],
            &[],
            &[],
        )
        .expect("label target")
        .expect("label metadata");
        assert_eq!(label.label.as_deref(), Some("slope-route"));

        let level = play_native_record_route_target(None, Some(0x06), &[], &[], &[], &[])
            .expect("level target")
            .expect("level metadata");
        assert_eq!(level.level, Some(0x06));

        let point = play_native_record_route_target(
            None,
            None,
            &[TargetPoint {
                x_px: 2848,
                y_px: 512,
            }],
            &[],
            &[],
            &[],
        )
        .expect("point target")
        .expect("point metadata");
        assert_eq!(point.points.len(), 1);
        assert_eq!(point.points[0].x_px, 2848);
        assert_eq!(point.points[0].y_px, 512);

        let cell = play_native_record_route_target(
            None,
            None,
            &[],
            &[TargetCell { col: 178, row: 32 }],
            &[],
            &[],
        )
        .expect("cell target")
        .expect("cell metadata");
        assert_eq!(cell.cells.len(), 1);
        assert_eq!(cell.cells[0].col, 178);
        assert_eq!(cell.cells[0].row, 32);

        let hitbox = play_native_record_route_target(
            None,
            None,
            &[],
            &[],
            &[TargetCell { col: 179, row: 32 }],
            &[],
        )
        .expect("cell-hitbox target")
        .expect("cell-hitbox metadata");
        assert_eq!(hitbox.cell_hitboxes.len(), 1);
        assert_eq!(hitbox.cell_hitboxes[0].col, 179);
        assert_eq!(hitbox.cell_hitboxes[0].row, 32);

        let enemy = play_native_record_route_target(None, None, &[], &[], &[], &[0x0A])
            .expect("enemy target")
            .expect("enemy metadata");
        assert_eq!(enemy.enemies, vec![0x0A]);
    }

    #[test]
    fn play_native_record_route_target_rejects_shell_awkward_labels() {
        let error = play_native_record_route_target(
            Some(String::from("USM3 slope")),
            None,
            &[],
            &[],
            &[],
            &[],
        )
        .expect_err("labels with spaces should be rejected");

        assert!(
            format!("{error:#}").contains("--record-target-label"),
            "target labels are echoed into record_command, so they need slug-like characters"
        );
    }

    #[test]
    fn frontier_kind_arg_matches_clap_value_names() {
        assert_eq!(
            frontier_kind_arg(crate::sweep::FrontierKind::Enemy),
            "enemy"
        );
        assert_eq!(
            frontier_kind_arg(crate::sweep::FrontierKind::Gameplay),
            "gameplay"
        );
        assert_eq!(frontier_kind_arg(crate::sweep::FrontierKind::Gems), "gems");
    }

    #[test]
    fn record_scenario_file_command_preserves_json_provenance() {
        let command = record_scenario_file_command(
            "issue-143-tas-route",
            Path::new("tests/data/tas/kc_any_percent_2024/scenario.json"),
            &[],
        );

        assert_eq!(
            command,
            "cargo run --release -p wildside-cli -- record-scenario-file --slug issue-143-tas-route --scenario tests/data/tas/kc_any_percent_2024/scenario.json"
        );
    }

    #[test]
    fn record_scenario_file_command_preserves_input_overlays() {
        let command = record_scenario_file_command(
            "issue-260-pre390",
            Path::new(".cache/recordings/issue-260/scenario.json"),
            &[
                String::from("390-440:none"),
                String::from("481-510:right+b"),
            ],
        );

        assert_eq!(
            command,
            "cargo run --release -p wildside-cli -- record-scenario-file --slug issue-260-pre390 --scenario .cache/recordings/issue-260/scenario.json --input 390-440:none --input 481-510:right+b"
        );
    }

    #[test]
    fn batch_target_projectile_cell_parser_accepts_kind_and_cell() {
        assert_eq!(
            parse_batch_target_projectile_cell("drips-surface-crawler:36,32")
                .expect("projectile-cell target should parse"),
            BatchTargetProjectileCell {
                kind: ActiveProjectileKind::DripsSurfaceCrawler,
                cell: TargetCell { col: 36, row: 32 },
            }
        );
        assert!(
            parse_batch_target_projectile_cell("drips-surface-crawler").is_err(),
            "batch projectile-cell targets must include both projectile kind and cell"
        );
    }

    #[test]
    fn batch_target_projectile_direction_parser_accepts_kind_and_direction() {
        assert_eq!(
            parse_batch_target_projectile_direction("shooter-block:left")
                .expect("projectile-direction target should parse"),
            BatchTargetProjectileDirection {
                kind: ActiveProjectileKind::ShooterBlock,
                direction: TargetProjectileDirection::Left,
            }
        );
        assert!(
            parse_batch_target_projectile_direction("shooter-block").is_err(),
            "batch projectile-direction targets must include both projectile kind and direction"
        );
    }

    #[test]
    fn batch_target_block_event_parser_accepts_kind_and_cell() {
        assert_eq!(
            parse_batch_target_block_event("bounce:37,32")
                .expect("block-event target should parse"),
            BatchTargetBlockEvent {
                kind: TargetBlockEventKind::Bounce,
                cell: TargetCell { col: 37, row: 32 },
            }
        );
        assert!(
            parse_batch_target_block_event("destroy").is_err(),
            "batch block-event targets must include both event kind and cell"
        );
    }

    #[test]
    fn target_block_event_finds_bounce_or_destroy_at_cell() {
        let mut trace = vec![NativeTraceFrame::default(); 13];
        trace[10].frame_index = 1110;
        trace[10].block_bounces = vec![(37, 32, 31, 0)];
        trace[12].frame_index = 1112;
        trace[12].block_destroy_effects = vec![(37 * 16, 32 * 16, 2, 1)];

        assert_eq!(
            target_block_event_from_frame(
                &trace,
                TargetBlockEventKind::Bounce,
                TargetCell { col: 37, row: 32 },
                0,
            )
            .map(|event| event.replay_frame),
            Some(10)
        );
        assert_eq!(
            target_block_event_from_frame(
                &trace,
                TargetBlockEventKind::Destroy,
                TargetCell { col: 37, row: 32 },
                11,
            )
            .map(|event| event.replay_frame),
            Some(12)
        );
        assert!(
            target_block_event_from_frame(
                &trace,
                TargetBlockEventKind::Bounce,
                TargetCell { col: 37, row: 32 },
                11,
            )
            .is_none(),
            "score-frame-start should filter earlier block events"
        );
    }

    #[test]
    fn audio_audit_parses_duplicate_sfx_constants_from_asm() {
        let temp = TempDir::new().expect("tempdir");
        let constants = temp.path().join("constants.asm");
        fs::write(
            &constants,
            "sfx_UFO_shoots = $3A ; duplicate\nsfx_Robot_shoots = $3A\nmusic_Level_1 = $06\n",
        )
        .expect("constants fixture");

        let names = parse_sfx_constants(&constants).expect("sfx constants should parse");

        assert_eq!(
            names.get(&0x3A).expect("duplicate id should exist"),
            &vec!["sfx_UFO_shoots".to_string(), "sfx_Robot_shoots".to_string()],
            "audio-audit must preserve duplicate constants.asm SFX labels such as the shared $3A shooter cue"
        );
        assert!(
            !names.values().flatten().any(|name| name == "music_Level_1"),
            "audio-audit enumerates sfx_* ids, not BGM constants"
        );
    }

    #[test]
    fn audio_audit_classifies_fm_dac_and_psg_writes() {
        let mut activity = AudioChipActivity::default();
        classify_audio_writes(
            &[
                GemsAudioWrite {
                    z80_cycles: 1,
                    kind: GemsAudioWriteKind::Ym2612,
                    port: 0,
                    value: 0x2A,
                },
                GemsAudioWrite {
                    z80_cycles: 2,
                    kind: GemsAudioWriteKind::Ym2612,
                    port: 1,
                    value: 0x80,
                },
                GemsAudioWrite {
                    z80_cycles: 3,
                    kind: GemsAudioWriteKind::Ym2612,
                    port: 0,
                    value: 0x28,
                },
                GemsAudioWrite {
                    z80_cycles: 4,
                    kind: GemsAudioWriteKind::Ym2612,
                    port: 1,
                    value: 0xF0,
                },
                GemsAudioWrite {
                    z80_cycles: 5,
                    kind: GemsAudioWriteKind::Psg,
                    port: 0,
                    value: 0x9F,
                },
            ],
            &mut activity,
        );

        let (chip_type, channels) = summarise_chip_activity(&activity);
        assert_eq!(chip_type, "mixed");
        assert_eq!(channels, vec!["fm", "dac", "psg"]);
        assert_eq!(activity.ym2612_writes, 4);
        assert_eq!(activity.psg_writes, 1);
    }
}
