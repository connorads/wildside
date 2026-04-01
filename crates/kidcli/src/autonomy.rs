use super::{
    GameplayScenario, Scenario, compare_native_gameplay, compare_native_oracle,
    prepare_generated_assets, verify_native_bootstrap,
};
use anyhow::{Context, Result};
use clap::ValueEnum;
use kid_data::AssetManifest;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

const LEVEL_ONE_ID: u8 = 0x00;
const PLAYER_CAMERA_TASK_MARKER: &str =
    "Replace the remaining scripted camera/player heuristics with recovered movement logic.";
const ENEMY_TASK_MARKER: &str = "Identify the exact enemy families visible in the first movement slice and map them to `code/enemy/*.asm`.";
const PLATFORM_TASK_MARKER: &str =
    "Trace the platform manager routines that consume `level/platform/00.asm`.";
const COLLISION_TASK_MARKER: &str =
    "Record the first collision and terrain rules needed for the opening walk segment.";
const DAMAGE_TRIGGER_TASK_MARKER: &str = "Recover the Level 1 gameplay damage trigger and death-transition path for the lifecycle slice.";
const README_TRAVERSAL_FRONTIER_MARKER: &str =
    "The current delivery frontier is Level 1 full traversal";
const RESEARCH_MAP_TRAVERSAL_MARKER: &str = "## Current frontier\n\n1. Level 1 full traversal";
const SOURCE_PORT_FRONTIER_STATE_PATH: &str = "docs/source-port/frontier-state.yaml";
const SOURCE_PORT_FRONTIER_NOTE_PATH: &str = ".cache/autonomy/source-port-frontier/next-agent.md";
const SOURCE_PORT_FRONTIER_TITLE: &str = "Source-port frontier";
const SOURCE_PORT_CASE_QUEUE_NOTE_PATH: &str = ".cache/autonomy/case-queue/next-agent.md";
const SOURCE_PORT_CASE_QUEUE_TITLE: &str = "Source-port case queue";
const SOURCE_PORT_CASE_CACHE_PATH: &str = ".cache/cases";
const SOURCE_PORT_RECORDINGS_CACHE_PATH: &str = ".cache/recordings";
const SOURCE_PORT_RESEARCH_MAP_PATH: &str = "docs/source-port/research-map.md";
const SOURCE_PORT_ROADMAP_PATH: &str = "docs/source-port/roadmap.md";
const SOURCE_PORT_WHOLE_GAME_INDEX_PATH: &str = "docs/source-port/whole-game-index.md";
const SOURCE_PORT_RELEASE_GATE_PATH: &str = ".cache/release-gate/1p-accurate/latest.json";
const SOURCE_PORT_FRONTIER_NOTE_REQUIREMENT_SUMMARY: &str = "Before exiting, replace the next-agent note with frontmatter for the active slug plus the required handoff sections.";
const SOURCE_PORT_FRONTIER_NOTE_REQUIRED_SECTIONS: [&str; 6] = [
    "What Changed",
    "Current Model",
    "Strongest Evidence",
    "Exact Next Moves",
    "Verification To Re-run",
    "Dead Ends / Do Not Retry",
];
const SOURCE_PORT_CASE_QUEUE_NOTE_REQUIREMENT_SUMMARY: &str = "Before exiting, replace the case-queue note with frontmatter for the selected case slug plus the required handoff sections.";
const SOURCE_PORT_CASE_QUEUE_NOTE_REQUIRED_SECTIONS: [&str; 7] = [
    "What Changed",
    "Selected Case",
    "Why This Case",
    "Strongest Evidence",
    "Exact Next Moves",
    "Verification To Re-run",
    "Dead Ends / Do Not Retry",
];

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum, Serialize, Deserialize)]
pub(crate) enum AutonomyGoal {
    #[value(name = "level1-opening-slice")]
    #[serde(rename = "level1-opening-slice")]
    Level1OpeningSlice,
    #[value(name = "level1-gameplay-lifecycle")]
    #[serde(rename = "level1-gameplay-lifecycle")]
    Level1GameplayLifecycle,
    #[value(name = "level1-full-traversal")]
    #[serde(rename = "level1-full-traversal")]
    Level1FullTraversal,
    /// Parity for a single level's idle scenario against oracle.
    #[value(name = "level-idle-parity")]
    #[serde(rename = "level-idle-parity")]
    LevelIdleParity,
    /// Sweep parity across a range of levels.
    #[value(name = "sweep-parity")]
    #[serde(rename = "sweep-parity")]
    SweepParity,
    /// Package the live source-port frontier into a Ralph-friendly task handoff.
    #[value(name = "source-port-frontier")]
    #[serde(rename = "source-port-frontier")]
    SourcePortFrontier,
    /// Rank and package reduced semantic gameplay cases for autonomous work.
    #[value(name = "case-queue")]
    #[serde(rename = "case-queue")]
    CaseQueue,
}

impl AutonomyGoal {
    pub(crate) fn slug(self) -> &'static str {
        match self {
            Self::Level1OpeningSlice => "level1-opening-slice",
            Self::Level1GameplayLifecycle => "level1-gameplay-lifecycle",
            Self::Level1FullTraversal => "level1-full-traversal",
            Self::LevelIdleParity => "level-idle-parity",
            Self::SweepParity => "sweep-parity",
            Self::SourcePortFrontier => "source-port-frontier",
            Self::CaseQueue => "case-queue",
        }
    }

    fn spec_path(self, repo_root: &Path) -> PathBuf {
        match self {
            Self::Level1OpeningSlice => repo_root.join("docs/source-port/level-1-goal.json"),
            Self::Level1GameplayLifecycle => {
                repo_root.join("docs/source-port/level-1-gameplay-lifecycle-goal.json")
            }
            Self::Level1FullTraversal => {
                repo_root.join("docs/source-port/level-1-full-traversal-goal.json")
            }
            Self::LevelIdleParity | Self::SweepParity => {
                // Sweep-based goals derive capabilities from the sweep report
                // rather than a static spec file.
                repo_root.join("docs/source-port/sweep-parity-goal.json")
            }
            Self::SourcePortFrontier => repo_root.join(SOURCE_PORT_FRONTIER_STATE_PATH),
            Self::CaseQueue => repo_root.join(SOURCE_PORT_CASE_CACHE_PATH),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum CapabilityId {
    BootTitleFlow,
    LevelLoadAndSpawn,
    LevelSettle,
    PlayerCameraMovement,
    OpeningScroll,
    OpeningCollision,
    PlatformActivation,
    EnemyActivation,
    PauseResumeOracle,
    DamageTriggerResearch,
    DeathRespawnParity,
    GoalLoopCompletion,
    TraversalDocsAligned,
    TerrainRuntimeAuthority,
    ProgressTwoTransitionsOracle,
    TraversalGoalLoopCompletion,
    ReleaseGateGreen,
    StateSweepGreen,
    SourcePortFrontierTask,
    SourcePortCaseQueueTask,
}

impl CapabilityId {
    fn as_str(self) -> &'static str {
        match self {
            Self::BootTitleFlow => "boot_title_flow",
            Self::LevelLoadAndSpawn => "level_load_and_spawn",
            Self::LevelSettle => "level_settle",
            Self::PlayerCameraMovement => "player_camera_movement",
            Self::OpeningScroll => "opening_scroll",
            Self::OpeningCollision => "opening_collision",
            Self::PlatformActivation => "platform_activation",
            Self::EnemyActivation => "enemy_activation",
            Self::PauseResumeOracle => "pause_resume_oracle",
            Self::DamageTriggerResearch => "damage_trigger_research",
            Self::DeathRespawnParity => "death_respawn_parity",
            Self::GoalLoopCompletion => "goal_loop_completion",
            Self::TraversalDocsAligned => "traversal_docs_aligned",
            Self::TerrainRuntimeAuthority => "terrain_runtime_authority",
            Self::ProgressTwoTransitionsOracle => "progress_two_transitions_oracle",
            Self::TraversalGoalLoopCompletion => "traversal_goal_loop_completion",
            Self::ReleaseGateGreen => "release_gate_green",
            Self::StateSweepGreen => "state_sweep_green",
            Self::SourcePortFrontierTask => "source_port_frontier_task",
            Self::SourcePortCaseQueueTask => "source_port_case_queue_task",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum CapabilityStatus {
    Pending,
    Ready,
    Complete,
    Blocked,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct GoalSpec {
    goal_id: String,
    title: String,
    summary: String,
    capabilities: Vec<CapabilitySpec>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct CapabilitySpec {
    id: CapabilityId,
    title: String,
    summary: String,
    prerequisites: Vec<CapabilityId>,
    sources: Vec<String>,
    implementation_targets: Vec<String>,
    commands: Vec<String>,
    done_criteria: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct LoopLedger {
    goal: String,
    title: String,
    cycles_run: u32,
    complete: bool,
    stalled: bool,
    last_task: Option<CapabilityId>,
    last_progress_fingerprint: Option<String>,
    last_cycle_path: Option<PathBuf>,
    capabilities: Vec<CapabilitySnapshot>,
    blocker: Option<BlockerReport>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct CapabilitySnapshot {
    id: CapabilityId,
    title: String,
    status: CapabilityStatus,
    summary: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct CycleReport {
    goal: String,
    cycle: u32,
    goal_complete: bool,
    stalled: bool,
    selected_task: Option<TaskPackage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    task_package: Option<TaskPackage>,
    blocker: Option<BlockerReport>,
    capabilities: Vec<CapabilityReport>,
    progress_fingerprint: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct CapabilityReport {
    id: CapabilityId,
    title: String,
    status: CapabilityStatus,
    summary: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct TaskPackage {
    capability: CapabilityId,
    title: String,
    summary: String,
    sources: Vec<String>,
    implementation_targets: Vec<String>,
    commands: Vec<String>,
    done_criteria: Vec<String>,
    evidence: Vec<EvidenceItem>,
    #[serde(skip_serializing_if = "Option::is_none")]
    frontier_context: Option<FrontierContext>,
    #[serde(skip_serializing_if = "Option::is_none")]
    case_queue_context: Option<CaseQueueContext>,
    #[serde(skip_serializing_if = "Option::is_none")]
    delegated_goal: Option<AutonomyGoal>,
    #[serde(skip_serializing_if = "Option::is_none")]
    delegated_target: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    delegation_reason: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    next_agent_note: Option<NextAgentNoteDirective>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct EvidenceItem {
    label: String,
    status: String,
    detail: String,
    artefact_path: Option<PathBuf>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct FrontierContext {
    slug: String,
    recording: String,
    scenario: String,
    area: String,
    status: String,
    frontier_frame: u32,
    authoritative_clean_through: u32,
    current_blocker: String,
    current_hypothesis: HypothesisSummary,
    rejected_hypotheses: Vec<HypothesisSummary>,
    required_next_evidence: Vec<String>,
    verification_commands: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct CaseQueueContext {
    selected_case: Option<CaseQueueCandidate>,
    open_cases: Vec<CaseQueueCandidate>,
    blocked_cases: Vec<CaseQueueCandidate>,
    untriaged_recordings: Vec<RecordingQueueCandidate>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct CaseQueueCandidate {
    case_id: String,
    selector: String,
    status: CaseQueueStatus,
    source_kind: String,
    source_slug: String,
    evidence_tier: String,
    frame_start: usize,
    frame_end: usize,
    checkpoint_frame: usize,
    checkpoint_label: String,
    first_bad_frame: Option<usize>,
    first_bad_field: Option<String>,
    failure_class: Option<CaseFailureClass>,
    likely_subsystems: Vec<String>,
    summary_path: PathBuf,
    manifest_path: PathBuf,
    verify_path: PathBuf,
    score: i64,
    autonomy_ready: bool,
    duplicate_of: Option<String>,
    rank_reason: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
enum CaseQueueStatus {
    Open,
    Reduced,
    Qualified,
    AutonomyExecutable,
    Blocked,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
enum CaseFailureClass {
    Authoritative,
    Advisory,
    Visual,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct RecordingQueueCandidate {
    slug: String,
    manifest_path: PathBuf,
    has_transcript: bool,
    has_notes: bool,
}

#[derive(Debug, Clone)]
struct CaseQueueSnapshot {
    selected_case: Option<CaseQueueCandidate>,
    selection_reason: Option<String>,
    open_cases: Vec<CaseQueueCandidate>,
    blocked_cases: Vec<CaseQueueCandidate>,
    untriaged_recordings: Vec<RecordingQueueCandidate>,
}

#[derive(Debug, Clone, Deserialize)]
struct CaseManifestView {
    case_id: String,
    source: CaseSourceView,
    evidence_tier: String,
    frame_start: usize,
    frame_end: usize,
    checkpoint_frame: usize,
    checkpoint_label: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
enum CaseSourceView {
    ScenarioPath { path: String },
    GameplayScenario { slug: String },
    Recording { path: String, slug: String },
}

#[derive(Debug, Clone, Deserialize)]
struct CaseVerifyReportView {
    passed: bool,
    #[serde(default)]
    first_failure: Option<CaseFailureView>,
    #[serde(default)]
    likely_subsystems: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
struct CaseFailureView {
    frame_index: usize,
    field: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct HypothesisSummary {
    name: String,
    reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct NextAgentNoteDirective {
    path: PathBuf,
    status: NextAgentNoteStatus,
    detail: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    frontmatter_slug: Option<String>,
    requirement_summary: String,
    required_sections: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
enum NextAgentNoteStatus {
    Missing,
    Stale,
    Current,
}

impl NextAgentNoteStatus {
    fn as_str(self) -> &'static str {
        match self {
            Self::Missing => "missing",
            Self::Stale => "stale",
            Self::Current => "current",
        }
    }
}

impl CaseQueueStatus {
    fn as_str(self) -> &'static str {
        match self {
            Self::Open => "open",
            Self::Reduced => "reduced",
            Self::Qualified => "qualified",
            Self::AutonomyExecutable => "autonomy_executable",
            Self::Blocked => "blocked",
        }
    }
}

impl CaseFailureClass {
    fn as_str(self) -> &'static str {
        match self {
            Self::Authoritative => "authoritative",
            Self::Advisory => "advisory",
            Self::Visual => "visual",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct SourcePortFrontierState {
    updated_at: String,
    active_frontier: ActiveFrontierState,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ActiveFrontierState {
    slug: String,
    recording: String,
    scenario: String,
    area: String,
    status: String,
    frontier_frame: u32,
    authoritative_clean_through: u32,
    current_blocker: String,
    current_hypothesis: HypothesisSummary,
    rejected_hypotheses: Vec<HypothesisSummary>,
    required_next_evidence: Vec<String>,
    verification_commands: Vec<String>,
    #[serde(default)]
    task_package: Option<SourcePortTaskPackageSpec>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct SourcePortTaskPackageSpec {
    title: String,
    summary: String,
    sources: Vec<String>,
    implementation_targets: Vec<String>,
    commands: Vec<String>,
    done_criteria: Vec<String>,
}

impl ActiveFrontierState {
    fn task_package_spec(&self) -> SourcePortTaskPackageSpec {
        self.task_package
            .clone()
            .unwrap_or_else(|| self.synthetic_task_package())
    }

    fn synthetic_task_package(&self) -> SourcePortTaskPackageSpec {
        let is_active = self.status == "active";
        SourcePortTaskPackageSpec {
            title: if is_active {
                format!("Advance source-port frontier: {}", self.area)
            } else {
                "Advance source-port beyond the closed TAS frontier".to_owned()
            },
            summary: if is_active {
                format!(
                    "Investigate `{}` for `{}` and advance the authoritative clean window past frame {}.",
                    self.area, self.recording, self.frontier_frame
                )
            } else {
                format!(
                    "`{}` is authoritative-clean through frame {}, but the source-port is not complete. Prioritise the broader remaining-work queue: oracle sweep hardening, wider 1P verification, and other open risk areas before declaring completion.",
                    self.recording, self.authoritative_clean_through
                )
            },
            sources: vec![
                SOURCE_PORT_FRONTIER_STATE_PATH.to_owned(),
                SOURCE_PORT_FRONTIER_NOTE_PATH.to_owned(),
                "docs/source-port/tas-roadmap.md".to_owned(),
                SOURCE_PORT_RESEARCH_MAP_PATH.to_owned(),
                SOURCE_PORT_ROADMAP_PATH.to_owned(),
                SOURCE_PORT_WHOLE_GAME_INDEX_PATH.to_owned(),
                SOURCE_PORT_RELEASE_GATE_PATH.to_owned(),
            ],
            implementation_targets: if is_active {
                vec![SOURCE_PORT_FRONTIER_STATE_PATH.to_owned()]
            } else {
                vec![
                    "crates/kidcli/src/sweep.rs".to_owned(),
                    "docs/source-port/methodology.md".to_owned(),
                    SOURCE_PORT_RESEARCH_MAP_PATH.to_owned(),
                    "docs/source-port/backlog.md".to_owned(),
                ]
            },
            commands: if is_active {
                self.verification_commands.clone()
            } else {
                vec![
                    "cargo run -p kidcli -- autonomous-loop --goal sweep-parity".to_owned(),
                    "cargo run --release -p kidcli -- sweep --levels 0-104 --tier state".to_owned(),
                    "mise run release-gate".to_owned(),
                ]
            },
            done_criteria: if is_active {
                vec![
                    "The live authoritative mismatch is reproduced and explained.".to_owned(),
                    "The compact frontier state is updated with the current hypothesis and next evidence."
                        .to_owned(),
                ]
            } else {
                vec![
                    "The next source-port task is chosen from the broader remaining-work queue instead of declaring the project complete."
                        .to_owned(),
                    "Oracle hardening, wider 1P verification, or another explicit open risk area is advanced with new evidence."
                        .to_owned(),
                ]
            },
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct BlockerReport {
    capability: CapabilityId,
    reason: String,
    missing_paths: Vec<PathBuf>,
}

#[derive(Debug, Clone)]
struct RepositorySignals {
    native_bootstrap: CheckOutcome,
    boot_oracle: CheckOutcome,
    move_oracle: CheckOutcome,
    pause_oracle: CheckOutcome,
    death_oracle: CheckOutcome,
    progress_two_transitions_oracle: CheckOutcome,
    release_gate: CheckOutcome,
    state_sweep: CheckOutcome,
    has_level_one_header: bool,
    has_level_one_start_and_flag: bool,
    enemy_layout_count: Option<u8>,
    platform_layout_count: Option<usize>,
    scripted_level_one_movement_present: bool,
    flat_floor_fallback_present: bool,
    traversal_docs_aligned: bool,
    dossier_open_markers: BTreeMap<CapabilityId, bool>,
}

#[derive(Debug, Clone)]
struct CheckOutcome {
    ok: bool,
    detail: String,
    artefact_path: Option<PathBuf>,
}

impl CheckOutcome {
    fn passing(detail: impl Into<String>, artefact_path: Option<PathBuf>) -> Self {
        Self {
            ok: true,
            detail: detail.into(),
            artefact_path,
        }
    }

    fn failing(detail: impl Into<String>, artefact_path: Option<PathBuf>) -> Self {
        Self {
            ok: false,
            detail: detail.into(),
            artefact_path,
        }
    }
}

pub(crate) fn run_autonomous_loop(
    repo_root: &Path,
    rom_path: Option<&Path>,
    upstream_dir: Option<&Path>,
    goal: AutonomyGoal,
    max_cycles: u32,
    resume: bool,
    emit_json: bool,
) -> Result<()> {
    let goal_state = match goal {
        AutonomyGoal::SourcePortFrontier => GoalState::SourcePortFrontier,
        AutonomyGoal::CaseQueue => GoalState::CaseQueue,
        _ => {
            let rom_path = rom_path.context("autonomy goal requires a ROM path")?;
            let upstream_dir = upstream_dir.context("autonomy goal requires an upstream dir")?;
            GoalState::SpecDriven {
                spec: load_goal_spec(&goal.spec_path(repo_root))?,
                rom_path: rom_path.to_path_buf(),
                manifest: Box::new(prepare_generated_assets(repo_root, rom_path, upstream_dir)?),
            }
        }
    };
    let run_dir = repo_root.join(".cache/autonomy").join(goal.slug());
    let evidence_dir = run_dir.join("evidence");
    fs::create_dir_all(&evidence_dir)
        .with_context(|| format!("failed to create {}", evidence_dir.display()))?;

    let ledger_path = run_dir.join("ledger.json");
    let mut ledger = if resume && ledger_path.is_file() {
        load_ledger(&ledger_path)?
    } else {
        LoopLedger {
            goal: goal.slug().to_owned(),
            title: goal_state.title().to_owned(),
            cycles_run: 0,
            complete: false,
            stalled: false,
            last_task: None,
            last_progress_fingerprint: None,
            last_cycle_path: None,
            capabilities: Vec::new(),
            blocker: None,
        }
    };

    let cycle_limit = if max_cycles == 0 {
        u32::MAX
    } else {
        max_cycles.max(1)
    };
    for _ in 0..cycle_limit {
        let cycle_number = ledger.cycles_run + 1;
        let mut cycle = goal_state.evaluate_cycle(repo_root, cycle_number)?;
        cycle.stalled = ledger
            .last_progress_fingerprint
            .as_deref()
            .is_some_and(|previous| previous == cycle.progress_fingerprint);

        let cycle_path = run_dir.join(format!("cycle-{cycle_number:04}.json"));
        write_json(&cycle_path, &cycle)?;
        write_json(
            &evidence_dir.join(format!("cycle-{cycle_number:04}.json")),
            &cycle,
        )?;
        let next_task_path = run_dir.join("next-task.json");
        if let Some(task) = &cycle.task_package {
            write_json(&next_task_path, task)?;
            if let (Some(note), Some(frontier)) = (
                task.next_agent_note.as_ref(),
                task.frontier_context.as_ref(),
            ) {
                write_source_port_next_agent_note(repo_root, note, frontier, task)?;
            }
        } else if next_task_path.exists() {
            fs::remove_file(&next_task_path)
                .with_context(|| format!("failed to remove {}", next_task_path.display()))?;
        }

        ledger.cycles_run = cycle_number;
        ledger.complete = cycle.goal_complete;
        ledger.stalled = cycle.stalled;
        ledger.last_task = cycle.selected_task.as_ref().map(|task| task.capability);
        ledger.last_progress_fingerprint = Some(cycle.progress_fingerprint.clone());
        ledger.last_cycle_path = Some(cycle_path);
        ledger.capabilities = cycle
            .capabilities
            .iter()
            .map(|capability| CapabilitySnapshot {
                id: capability.id,
                title: capability.title.clone(),
                status: capability.status,
                summary: capability.summary.clone(),
            })
            .collect();
        ledger.blocker = cycle.blocker.clone();
        write_json(&ledger_path, &ledger)?;

        if emit_json {
            println!("{}", serde_json::to_string_pretty(&cycle)?);
        } else {
            print_cycle_summary(&cycle);
        }

        if cycle.goal_complete || cycle.blocker.is_some() || cycle.stalled {
            break;
        }
    }

    Ok(())
}

enum GoalState {
    SpecDriven {
        spec: GoalSpec,
        rom_path: PathBuf,
        manifest: Box<AssetManifest>,
    },
    SourcePortFrontier,
    CaseQueue,
}

impl GoalState {
    fn title(&self) -> &str {
        match self {
            Self::SpecDriven { spec, .. } => spec.title.as_str(),
            Self::SourcePortFrontier => SOURCE_PORT_FRONTIER_TITLE,
            Self::CaseQueue => SOURCE_PORT_CASE_QUEUE_TITLE,
        }
    }

    fn evaluate_cycle(&self, repo_root: &Path, cycle_number: u32) -> Result<CycleReport> {
        match self {
            Self::SpecDriven {
                spec,
                rom_path,
                manifest,
            } => evaluate_cycle(repo_root, rom_path, manifest, spec, cycle_number),
            Self::SourcePortFrontier => {
                evaluate_source_port_frontier_cycle(repo_root, cycle_number)
            }
            Self::CaseQueue => evaluate_case_queue_cycle(repo_root, cycle_number),
        }
    }
}

fn load_goal_spec(path: &Path) -> Result<GoalSpec> {
    let bytes =
        fs::read(path).with_context(|| format!("failed to read goal spec {}", path.display()))?;
    serde_json::from_slice(&bytes)
        .with_context(|| format!("failed to parse goal spec {}", path.display()))
}

fn load_ledger(path: &Path) -> Result<LoopLedger> {
    let bytes = fs::read(path).with_context(|| format!("failed to read {}", path.display()))?;
    serde_json::from_slice(&bytes)
        .with_context(|| format!("failed to parse ledger {}", path.display()))
}

fn load_source_port_frontier_state(repo_root: &Path) -> Result<SourcePortFrontierState> {
    let path = repo_root.join(SOURCE_PORT_FRONTIER_STATE_PATH);
    let bytes = fs::read(&path)
        .with_context(|| format!("failed to read frontier state {}", path.display()))?;
    serde_json::from_slice(&bytes)
        .with_context(|| format!("failed to parse frontier state {}", path.display()))
}

fn goal_ledger_path(repo_root: &Path, goal: AutonomyGoal) -> PathBuf {
    repo_root
        .join(".cache/autonomy")
        .join(goal.slug())
        .join("ledger.json")
}

fn load_goal_ledger_if_present(repo_root: &Path, goal: AutonomyGoal) -> Result<Option<LoopLedger>> {
    let path = goal_ledger_path(repo_root, goal);
    if !path.is_file() {
        return Ok(None);
    }
    load_ledger(&path).map(Some)
}

fn case_queue_gate_status(candidate: &CaseQueueCandidate) -> &'static str {
    if candidate.autonomy_ready {
        "patch-allowed"
    } else {
        "evidence-only"
    }
}

fn case_queue_candidate_progress_marker(candidate: &CaseQueueCandidate) -> String {
    format!(
        "{}:{}:{}:{}:{}:{}:{}",
        candidate.case_id,
        candidate.status.as_str(),
        candidate
            .first_bad_frame
            .map(|frame| frame.to_string())
            .unwrap_or_else(|| "none".to_owned()),
        candidate.first_bad_field.as_deref().unwrap_or("none"),
        case_queue_gate_status(candidate),
        candidate.frame_end.saturating_sub(candidate.frame_start),
        candidate
            .failure_class
            .map(|class| class.as_str())
            .unwrap_or("unknown")
    )
}

fn case_queue_progress_fingerprint(selected_case: Option<&CaseQueueCandidate>) -> String {
    selected_case
        .map(case_queue_candidate_progress_marker)
        .unwrap_or_else(|| "none".to_owned())
}

fn select_case_queue_candidate(
    repo_root: &Path,
    cases: &[CaseQueueCandidate],
) -> Result<(Option<CaseQueueCandidate>, Option<String>)> {
    let actionable: Vec<_> = cases
        .iter()
        .filter(|candidate| {
            candidate.autonomy_ready
                || (candidate.failure_class == Some(CaseFailureClass::Authoritative)
                    && candidate.duplicate_of.is_none())
        })
        .cloned()
        .collect();
    let Some(first) = actionable.first().cloned() else {
        return Ok((None, None));
    };
    let previous = load_goal_ledger_if_present(repo_root, AutonomyGoal::CaseQueue)?
        .and_then(|ledger| ledger.last_progress_fingerprint);
    if previous.as_deref() == Some(case_queue_candidate_progress_marker(&first).as_str()) {
        if let Some(next) = actionable.get(1).cloned() {
            return Ok((
                Some(next.clone()),
                Some(format!(
                    "previous cycle stalled on {}; pivoted to the next ranked case {}",
                    first.case_id, next.case_id
                )),
            ));
        }
        return Ok((
            Some(first.clone()),
            Some(format!(
                "{} remains the highest-ranked case and no alternate actionable case is available",
                first.case_id
            )),
        ));
    }
    Ok((Some(first), None))
}

fn source_port_frontier_progress_fingerprint(
    frontier: &ActiveFrontierState,
    delegated_goal: Option<AutonomyGoal>,
    delegated_target: Option<&str>,
    lane_fingerprint: Option<&str>,
) -> String {
    format!(
        "{}:{}:{}:{}:{}:{}:{}",
        frontier.slug,
        frontier.status,
        frontier.frontier_frame,
        frontier.authoritative_clean_through,
        delegated_goal
            .map(AutonomyGoal::slug)
            .unwrap_or("tas-frontier"),
        delegated_target.unwrap_or("none"),
        lane_fingerprint.unwrap_or("none")
    )
}

fn evaluate_source_port_frontier_cycle(repo_root: &Path, cycle: u32) -> Result<CycleReport> {
    let state = load_source_port_frontier_state(repo_root)?;
    let frontier = &state.active_frontier;
    let note = source_port_next_agent_note(repo_root, frontier.slug.as_str());
    let task_ready = matches!(frontier.status.as_str(), "active" | "complete");
    let (task_package, delegated_goal, delegated_target, lane_fingerprint, summary) = if frontier
        .status
        == "active"
    {
        let task = source_port_frontier_task(frontier, &note);
        let summary = format!(
            "Active frontier {} at frame {} (clean through {})",
            frontier.slug, frontier.frontier_frame, frontier.authoritative_clean_through
        );
        (
            task,
            None,
            Some(frontier.slug.clone()),
            Some(format!(
                "{}:{}",
                frontier.current_hypothesis.name, frontier.current_blocker
            )),
            summary,
        )
    } else if frontier.status == "complete" {
        let queue = build_case_queue_snapshot(repo_root)?;
        if let Some(candidate) = queue.selected_case.as_ref() {
            let task = source_port_frontier_case_queue_task(frontier, &note, &queue, candidate);
            let summary = format!(
                "TAS frontier {} is closed through frame {}; delegated to case-queue for {}.",
                frontier.slug, frontier.authoritative_clean_through, candidate.case_id
            );
            (
                task,
                Some(AutonomyGoal::CaseQueue),
                Some(candidate.case_id.clone()),
                Some(case_queue_progress_fingerprint(Some(candidate))),
                summary,
            )
        } else {
            let task = source_port_frontier_sweep_task(frontier, &note);
            let summary = format!(
                "TAS frontier {} is closed through frame {}; no actionable case-queue case exists, so source-port pivots to sweep parity hardening.",
                frontier.slug, frontier.authoritative_clean_through
            );
            (
                task,
                Some(AutonomyGoal::SweepParity),
                Some("state-sweep".to_owned()),
                Some("state-sweep".to_owned()),
                summary,
            )
        }
    } else {
        let task = source_port_frontier_task(frontier, &note);
        let summary = format!(
            "Active frontier {} is no longer marked active (status={})",
            frontier.slug, frontier.status
        );
        (
            task,
            None,
            Some(frontier.slug.clone()),
            Some(frontier.status.clone()),
            summary,
        )
    };
    let selected_task = task_ready.then(|| task_package.clone());
    let blocker = selected_task
        .as_ref()
        .and_then(|task| detect_blocker(repo_root, task));
    let capabilities = vec![CapabilityReport {
        id: CapabilityId::SourcePortFrontierTask,
        title: task_package.title.clone(),
        status: if task_ready {
            CapabilityStatus::Ready
        } else {
            CapabilityStatus::Complete
        },
        summary,
    }];

    Ok(CycleReport {
        goal: AutonomyGoal::SourcePortFrontier.slug().to_owned(),
        cycle,
        goal_complete: !task_ready,
        stalled: false,
        task_package: Some(task_package),
        selected_task,
        blocker,
        progress_fingerprint: source_port_frontier_progress_fingerprint(
            frontier,
            delegated_goal,
            delegated_target.as_deref(),
            lane_fingerprint.as_deref(),
        ),
        capabilities,
    })
}

fn source_port_frontier_task(
    frontier: &ActiveFrontierState,
    note: &NextAgentNoteDirective,
) -> TaskPackage {
    let task_package = frontier.task_package_spec();
    TaskPackage {
        capability: CapabilityId::SourcePortFrontierTask,
        title: task_package.title,
        summary: task_package.summary,
        sources: task_package.sources,
        implementation_targets: task_package.implementation_targets,
        commands: task_package.commands,
        done_criteria: task_package.done_criteria,
        evidence: vec![
            EvidenceItem {
                label: "active-frontier".to_owned(),
                status: frontier.status.clone(),
                detail: format!(
                    "{} frame {} (clean through {})",
                    frontier.recording,
                    frontier.frontier_frame,
                    frontier.authoritative_clean_through
                ),
                artefact_path: Some(PathBuf::from(SOURCE_PORT_FRONTIER_STATE_PATH)),
            },
            EvidenceItem {
                label: "current-hypothesis".to_owned(),
                status: "active".to_owned(),
                detail: format!(
                    "{}: {}",
                    frontier.current_hypothesis.name, frontier.current_hypothesis.reason
                ),
                artefact_path: Some(PathBuf::from(SOURCE_PORT_FRONTIER_STATE_PATH)),
            },
            EvidenceItem {
                label: "next-agent-note".to_owned(),
                status: note.status.as_str().to_owned(),
                detail: note.detail.clone(),
                artefact_path: Some(note.path.clone()),
            },
        ],
        frontier_context: Some(FrontierContext {
            slug: frontier.slug.clone(),
            recording: frontier.recording.clone(),
            scenario: frontier.scenario.clone(),
            area: frontier.area.clone(),
            status: frontier.status.clone(),
            frontier_frame: frontier.frontier_frame,
            authoritative_clean_through: frontier.authoritative_clean_through,
            current_blocker: frontier.current_blocker.clone(),
            current_hypothesis: frontier.current_hypothesis.clone(),
            rejected_hypotheses: frontier.rejected_hypotheses.clone(),
            required_next_evidence: frontier.required_next_evidence.clone(),
            verification_commands: frontier.verification_commands.clone(),
        }),
        case_queue_context: None,
        delegated_goal: None,
        delegated_target: None,
        delegation_reason: None,
        next_agent_note: Some(note.clone()),
    }
}

fn source_port_frontier_case_queue_task(
    frontier: &ActiveFrontierState,
    note: &NextAgentNoteDirective,
    queue: &CaseQueueSnapshot,
    candidate: &CaseQueueCandidate,
) -> TaskPackage {
    let delegation_reason = queue.selection_reason.clone().unwrap_or_else(|| {
        format!(
            "the TAS frontier is already closed, so the next concrete lane is the highest-ranked reduced case {}",
            candidate.case_id
        )
    });
    TaskPackage {
        capability: CapabilityId::SourcePortFrontierTask,
        title: format!("Advance source-port via case queue: {}", candidate.case_id),
        summary: format!(
            "{} is authoritative-clean through frame {}; work the reduced case {} instead of reopening the TAS frontier.",
            frontier.recording, frontier.authoritative_clean_through, candidate.case_id
        ),
        sources: vec![
            SOURCE_PORT_FRONTIER_STATE_PATH.to_owned(),
            candidate.manifest_path.display().to_string(),
            candidate.verify_path.display().to_string(),
            candidate.summary_path.display().to_string(),
            SOURCE_PORT_RESEARCH_MAP_PATH.to_owned(),
        ],
        implementation_targets: Vec::new(),
        commands: vec![
            format!(
                "cargo run -p kidcli -- case verify --case {}",
                candidate.selector
            ),
            format!(
                "cargo run -p kidcli -- case summarise --case {}",
                candidate.selector
            ),
        ],
        done_criteria: vec![
            format!(
                "Advance or retire {} without reopening closed TAS-frontier work.",
                candidate.case_id
            ),
            "Use the reduced case evidence to land a code change, tighter probe, or an explicit blocked result."
                .to_owned(),
        ],
        evidence: vec![
            EvidenceItem {
                label: "active-frontier".to_owned(),
                status: frontier.status.clone(),
                detail: format!(
                    "{} clean through frame {}",
                    frontier.slug, frontier.authoritative_clean_through
                ),
                artefact_path: Some(PathBuf::from(SOURCE_PORT_FRONTIER_STATE_PATH)),
            },
            EvidenceItem {
                label: "delegated-lane".to_owned(),
                status: "case-queue".to_owned(),
                detail: delegation_reason.clone(),
                artefact_path: Some(candidate.summary_path.clone()),
            },
            EvidenceItem {
                label: "selected-case".to_owned(),
                status: candidate.status.as_str().to_owned(),
                detail: candidate.rank_reason.clone(),
                artefact_path: Some(candidate.verify_path.clone()),
            },
            EvidenceItem {
                label: "next-agent-note".to_owned(),
                status: note.status.as_str().to_owned(),
                detail: note.detail.clone(),
                artefact_path: Some(note.path.clone()),
            },
        ],
        frontier_context: Some(FrontierContext {
            slug: frontier.slug.clone(),
            recording: frontier.recording.clone(),
            scenario: frontier.scenario.clone(),
            area: frontier.area.clone(),
            status: frontier.status.clone(),
            frontier_frame: frontier.frontier_frame,
            authoritative_clean_through: frontier.authoritative_clean_through,
            current_blocker: frontier.current_blocker.clone(),
            current_hypothesis: frontier.current_hypothesis.clone(),
            rejected_hypotheses: frontier.rejected_hypotheses.clone(),
            required_next_evidence: frontier.required_next_evidence.clone(),
            verification_commands: frontier.verification_commands.clone(),
        }),
        case_queue_context: Some(CaseQueueContext {
            selected_case: Some(candidate.clone()),
            open_cases: queue.open_cases.clone(),
            blocked_cases: queue.blocked_cases.clone(),
            untriaged_recordings: queue.untriaged_recordings.clone(),
        }),
        delegated_goal: Some(AutonomyGoal::CaseQueue),
        delegated_target: Some(candidate.case_id.clone()),
        delegation_reason: Some(delegation_reason),
        next_agent_note: Some(note.clone()),
    }
}

fn source_port_frontier_sweep_task(
    frontier: &ActiveFrontierState,
    note: &NextAgentNoteDirective,
) -> TaskPackage {
    let delegation_reason = "the TAS frontier is closed and no actionable reduced case is available, so the next repo-local lane is sweep parity hardening".to_owned();
    TaskPackage {
        capability: CapabilityId::SourcePortFrontierTask,
        title: "Advance source-port via sweep parity hardening".to_owned(),
        summary: format!(
            "{} is authoritative-clean through frame {}; continue with sweep parity and release-gate hardening instead of reopening TAS work.",
            frontier.recording, frontier.authoritative_clean_through
        ),
        sources: vec![
            SOURCE_PORT_FRONTIER_STATE_PATH.to_owned(),
            SOURCE_PORT_RESEARCH_MAP_PATH.to_owned(),
            "docs/source-port/methodology.md".to_owned(),
            SOURCE_PORT_RELEASE_GATE_PATH.to_owned(),
        ],
        implementation_targets: vec![
            "crates/kidcli/src/sweep.rs".to_owned(),
            "docs/source-port/methodology.md".to_owned(),
            SOURCE_PORT_RESEARCH_MAP_PATH.to_owned(),
        ],
        commands: vec![
            "cargo run -p kidcli -- autonomous-loop --goal sweep-parity".to_owned(),
            "cargo run --release -p kidcli -- sweep --levels 0-104 --tier state".to_owned(),
            "mise run release-gate".to_owned(),
        ],
        done_criteria: vec![
            "Advance sweep parity, release-gate hardening, or another explicit verification lane with new evidence."
                .to_owned(),
            "Do not reopen TAS-frontier work unless a new authoritative frontier appears.".to_owned(),
        ],
        evidence: vec![
            EvidenceItem {
                label: "active-frontier".to_owned(),
                status: frontier.status.clone(),
                detail: format!(
                    "{} clean through frame {}",
                    frontier.slug, frontier.authoritative_clean_through
                ),
                artefact_path: Some(PathBuf::from(SOURCE_PORT_FRONTIER_STATE_PATH)),
            },
            EvidenceItem {
                label: "delegated-lane".to_owned(),
                status: "sweep-parity".to_owned(),
                detail: delegation_reason.clone(),
                artefact_path: Some(PathBuf::from(SOURCE_PORT_RESEARCH_MAP_PATH)),
            },
            EvidenceItem {
                label: "next-agent-note".to_owned(),
                status: note.status.as_str().to_owned(),
                detail: note.detail.clone(),
                artefact_path: Some(note.path.clone()),
            },
        ],
        frontier_context: Some(FrontierContext {
            slug: frontier.slug.clone(),
            recording: frontier.recording.clone(),
            scenario: frontier.scenario.clone(),
            area: frontier.area.clone(),
            status: frontier.status.clone(),
            frontier_frame: frontier.frontier_frame,
            authoritative_clean_through: frontier.authoritative_clean_through,
            current_blocker: frontier.current_blocker.clone(),
            current_hypothesis: frontier.current_hypothesis.clone(),
            rejected_hypotheses: frontier.rejected_hypotheses.clone(),
            required_next_evidence: frontier.required_next_evidence.clone(),
            verification_commands: frontier.verification_commands.clone(),
        }),
        case_queue_context: None,
        delegated_goal: Some(AutonomyGoal::SweepParity),
        delegated_target: Some("state-sweep".to_owned()),
        delegation_reason: Some(delegation_reason),
        next_agent_note: Some(note.clone()),
    }
}

fn source_port_next_agent_note(repo_root: &Path, active_slug: &str) -> NextAgentNoteDirective {
    let path = PathBuf::from(SOURCE_PORT_FRONTIER_NOTE_PATH);
    let note_path = repo_root.join(&path);
    let (status, frontmatter_slug, detail) = note_status_for_frontier(&note_path, active_slug);
    NextAgentNoteDirective {
        path,
        status,
        detail,
        frontmatter_slug,
        requirement_summary: SOURCE_PORT_FRONTIER_NOTE_REQUIREMENT_SUMMARY.to_owned(),
        required_sections: SOURCE_PORT_FRONTIER_NOTE_REQUIRED_SECTIONS
            .iter()
            .map(|section| (*section).to_owned())
            .collect(),
    }
}

fn note_status_for_frontier(
    note_path: &Path,
    active_slug: &str,
) -> (NextAgentNoteStatus, Option<String>, String) {
    let contents = match fs::read_to_string(note_path) {
        Ok(contents) => contents,
        Err(_) => {
            return (
                NextAgentNoteStatus::Missing,
                None,
                "no next-agent note present for the active frontier".to_owned(),
            );
        }
    };
    let frontmatter_slug = extract_frontmatter_slug(&contents);
    match frontmatter_slug.as_deref() {
        Some(slug) if slug == active_slug => (
            NextAgentNoteStatus::Current,
            Some(slug.to_owned()),
            format!("note slug `{slug}` matches the active frontier"),
        ),
        Some(slug) => (
            NextAgentNoteStatus::Stale,
            Some(slug.to_owned()),
            format!("note slug `{slug}` does not match active frontier `{active_slug}`"),
        ),
        None => (
            NextAgentNoteStatus::Stale,
            None,
            "next-agent note is missing `slug` frontmatter".to_owned(),
        ),
    }
}

fn extract_frontmatter_slug(contents: &str) -> Option<String> {
    let mut lines = contents.lines();
    if lines.next()? != "---" {
        return None;
    }
    for line in lines {
        if line == "---" {
            break;
        }
        if let Some((key, value)) = line.split_once(':')
            && key.trim() == "slug"
        {
            return Some(value.trim().to_owned());
        }
    }
    None
}

fn evaluate_case_queue_cycle(repo_root: &Path, cycle: u32) -> Result<CycleReport> {
    let queue = build_case_queue_snapshot(repo_root)?;
    let active_slug = queue
        .selected_case
        .as_ref()
        .map(|candidate| candidate.case_id.as_str())
        .unwrap_or("no-selected-case");
    let note = case_queue_next_agent_note(repo_root, active_slug);
    let selected_task = queue
        .selected_case
        .as_ref()
        .map(|candidate| case_queue_task(candidate, &queue, &note));
    let blocker = if let Some(task) = selected_task.as_ref() {
        detect_blocker(repo_root, task)
    } else {
        Some(BlockerReport {
            capability: CapabilityId::SourcePortCaseQueueTask,
            reason: "no reduced authoritative case is available in .cache/cases".to_owned(),
            missing_paths: vec![repo_root.join(SOURCE_PORT_CASE_CACHE_PATH)],
        })
    };
    let status = if selected_task.is_some() {
        CapabilityStatus::Ready
    } else {
        CapabilityStatus::Blocked
    };
    let summary = if let Some(candidate) = queue.selected_case.as_ref() {
        let mut summary = format!(
            "Selected {} ({}, {} -> {:?}) from the semantic case queue.",
            candidate.case_id,
            candidate.status.as_str(),
            candidate.source_kind,
            candidate.first_bad_field.as_deref().unwrap_or("no-failure")
        );
        if let Some(reason) = queue.selection_reason.as_deref() {
            summary.push(' ');
            summary.push_str(reason);
            summary.push('.');
        }
        summary
    } else {
        "No qualifying semantic case bundle is available yet; derive or reduce a case before patch work."
            .to_owned()
    };
    let capabilities = vec![CapabilityReport {
        id: CapabilityId::SourcePortCaseQueueTask,
        title: "Source-port case queue".to_owned(),
        status,
        summary,
    }];

    Ok(CycleReport {
        goal: AutonomyGoal::CaseQueue.slug().to_owned(),
        cycle,
        goal_complete: false,
        stalled: false,
        task_package: selected_task.clone(),
        selected_task,
        blocker,
        progress_fingerprint: case_queue_progress_fingerprint(queue.selected_case.as_ref()),
        capabilities,
    })
}

fn build_case_queue_snapshot(repo_root: &Path) -> Result<CaseQueueSnapshot> {
    let mut cases = load_case_queue_candidates(repo_root)?;
    apply_case_queue_duplicate_penalties(&mut cases);
    cases.sort_by(case_queue_sort_key);

    let (selected_case, selection_reason) = select_case_queue_candidate(repo_root, &cases)?;

    let open_cases = cases
        .iter()
        .filter(|candidate| candidate.status != CaseQueueStatus::Blocked)
        .take(8)
        .cloned()
        .collect();
    let blocked_cases = cases
        .iter()
        .filter(|candidate| candidate.status == CaseQueueStatus::Blocked)
        .take(8)
        .cloned()
        .collect();
    let untriaged_recordings = load_untriaged_recordings(repo_root, &cases)?;

    Ok(CaseQueueSnapshot {
        selected_case,
        selection_reason,
        open_cases,
        blocked_cases,
        untriaged_recordings,
    })
}

fn load_case_queue_candidates(repo_root: &Path) -> Result<Vec<CaseQueueCandidate>> {
    let cases_dir = repo_root.join(SOURCE_PORT_CASE_CACHE_PATH);
    let mut candidates = Vec::new();
    let entries = match fs::read_dir(&cases_dir) {
        Ok(entries) => entries,
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => return Ok(candidates),
        Err(err) => {
            return Err(err).with_context(|| format!("failed to read {}", cases_dir.display()));
        }
    };
    for entry in entries {
        let entry = entry?;
        let path = entry.path();
        if !path.is_dir() || path.file_name().and_then(|name| name.to_str()) == Some(".tmp") {
            continue;
        }
        let manifest_path = path.join("manifest.json");
        let verify_path = path.join("verify.json");
        let summary_path = path.join("summary.md");
        if !manifest_path.is_file() || !verify_path.is_file() {
            continue;
        }
        let manifest: CaseManifestView = serde_json::from_slice(&fs::read(&manifest_path)?)
            .with_context(|| format!("failed to parse {}", manifest_path.display()))?;
        let verify: CaseVerifyReportView = serde_json::from_slice(&fs::read(&verify_path)?)
            .with_context(|| format!("failed to parse {}", verify_path.display()))?;
        if verify.passed {
            continue;
        }
        let (source_kind, source_slug) = case_source_details(&manifest.source);
        let first_failure = verify.first_failure.as_ref();
        let failure_class = first_failure.map(|failure| classify_case_failure(&failure.field));
        let window_len = manifest.frame_end.saturating_sub(manifest.frame_start);
        let autonomy_ready = failure_class == Some(CaseFailureClass::Authoritative)
            && window_len <= 120
            && !verify.likely_subsystems.is_empty()
            && verify.likely_subsystems.len() <= 2;
        let status = case_queue_status(
            failure_class,
            autonomy_ready,
            window_len,
            verify.likely_subsystems.len(),
        );
        let score = case_queue_score(
            &manifest.source,
            failure_class,
            manifest.checkpoint_frame,
            manifest.frame_end,
            first_failure,
            verify.likely_subsystems.len(),
        );
        candidates.push(CaseQueueCandidate {
            case_id: manifest.case_id.clone(),
            selector: path
                .file_name()
                .and_then(|name| name.to_str())
                .unwrap_or(manifest.case_id.as_str())
                .to_owned(),
            status,
            source_kind: source_kind.to_owned(),
            source_slug,
            evidence_tier: manifest.evidence_tier,
            frame_start: manifest.frame_start,
            frame_end: manifest.frame_end,
            checkpoint_frame: manifest.checkpoint_frame,
            checkpoint_label: manifest.checkpoint_label,
            first_bad_frame: first_failure.map(|failure| failure.frame_index),
            first_bad_field: first_failure.map(|failure| failure.field.clone()),
            failure_class,
            likely_subsystems: verify.likely_subsystems,
            summary_path: repo_relative_path(repo_root, &summary_path),
            manifest_path: repo_relative_path(repo_root, &manifest_path),
            verify_path: repo_relative_path(repo_root, &verify_path),
            score,
            autonomy_ready,
            duplicate_of: None,
            rank_reason: String::new(),
        });
    }
    for candidate in &mut candidates {
        candidate.rank_reason = rank_reason(candidate);
    }
    Ok(candidates)
}

fn load_untriaged_recordings(
    repo_root: &Path,
    cases: &[CaseQueueCandidate],
) -> Result<Vec<RecordingQueueCandidate>> {
    let recordings_dir = repo_root.join(SOURCE_PORT_RECORDINGS_CACHE_PATH);
    let mut out = Vec::new();
    let entries = match fs::read_dir(&recordings_dir) {
        Ok(entries) => entries,
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => return Ok(out),
        Err(err) => {
            return Err(err)
                .with_context(|| format!("failed to read {}", recordings_dir.display()));
        }
    };
    for entry in entries {
        let entry = entry?;
        let root = entry.path();
        if !root.is_dir() {
            continue;
        }
        let bundle_root = nested_single_directory_or_self(&root)?;
        let manifest_path = bundle_root.join("manifest.json");
        if !manifest_path.is_file() {
            continue;
        }
        let bytes = fs::read(&manifest_path)?;
        let manifest: serde_json::Value = serde_json::from_slice(&bytes)
            .with_context(|| format!("failed to parse {}", manifest_path.display()))?;
        let slug = manifest
            .get("slug")
            .and_then(|slug| slug.as_str())
            .unwrap_or_else(|| {
                bundle_root
                    .file_name()
                    .and_then(|name| name.to_str())
                    .unwrap_or("unknown-recording")
            })
            .to_owned();
        let already_covered = cases
            .iter()
            .any(|candidate| candidate.source_kind == "recording" && candidate.source_slug == slug);
        if already_covered {
            continue;
        }
        out.push(RecordingQueueCandidate {
            slug,
            manifest_path: repo_relative_path(repo_root, &manifest_path),
            has_transcript: bundle_root.join("transcript.txt").is_file(),
            has_notes: bundle_root.join("notes.txt").is_file(),
        });
    }
    out.sort_by(|left, right| left.slug.cmp(&right.slug));
    Ok(out)
}

fn nested_single_directory_or_self(path: &Path) -> Result<PathBuf> {
    let entries: Vec<_> = fs::read_dir(path)?
        .collect::<Result<Vec<_>, _>>()
        .with_context(|| format!("failed to inspect {}", path.display()))?;
    let nested_dirs: Vec<_> = entries
        .into_iter()
        .map(|entry| entry.path())
        .filter(|candidate| candidate.is_dir())
        .collect();
    Ok(if nested_dirs.len() == 1 {
        nested_dirs[0].clone()
    } else {
        path.to_path_buf()
    })
}

fn case_source_details(source: &CaseSourceView) -> (&'static str, String) {
    match source {
        CaseSourceView::ScenarioPath { path } => ("scenario_path", path.clone()),
        CaseSourceView::GameplayScenario { slug } => ("gameplay_scenario", slug.clone()),
        CaseSourceView::Recording { path, slug } => (
            "recording",
            if slug.is_empty() {
                path.clone()
            } else {
                slug.clone()
            },
        ),
    }
}

fn classify_case_failure(field: &str) -> CaseFailureClass {
    if [
        "world.score",
        "world.number_lives",
        "world.diamond_count",
        "world.number_objects",
        "world.murder_wall_x",
    ]
    .iter()
    .any(|needle| field.contains(needle))
    {
        CaseFailureClass::Advisory
    } else if ["sprite", "animation", "render", "palette"]
        .iter()
        .any(|needle| field.contains(needle))
    {
        CaseFailureClass::Visual
    } else {
        CaseFailureClass::Authoritative
    }
}

fn case_queue_status(
    failure_class: Option<CaseFailureClass>,
    autonomy_ready: bool,
    window_len: usize,
    subsystem_count: usize,
) -> CaseQueueStatus {
    match failure_class {
        Some(CaseFailureClass::Advisory | CaseFailureClass::Visual) | None => {
            CaseQueueStatus::Blocked
        }
        Some(CaseFailureClass::Authoritative) if autonomy_ready => {
            CaseQueueStatus::AutonomyExecutable
        }
        Some(CaseFailureClass::Authoritative) if window_len <= 120 && subsystem_count <= 2 => {
            CaseQueueStatus::Qualified
        }
        Some(CaseFailureClass::Authoritative) if window_len <= 240 => CaseQueueStatus::Reduced,
        Some(CaseFailureClass::Authoritative) => CaseQueueStatus::Open,
    }
}

fn case_queue_score(
    source: &CaseSourceView,
    failure_class: Option<CaseFailureClass>,
    checkpoint_frame: usize,
    frame_end: usize,
    first_failure: Option<&CaseFailureView>,
    subsystem_count: usize,
) -> i64 {
    let source_weight = match source {
        CaseSourceView::Recording { .. } => 500,
        CaseSourceView::GameplayScenario { .. } => 400,
        CaseSourceView::ScenarioPath { .. } => 300,
    };
    let severity = match failure_class {
        Some(CaseFailureClass::Authoritative) => 300,
        Some(CaseFailureClass::Visual) => 100,
        Some(CaseFailureClass::Advisory) | None => 0,
    };
    let frames_to_failure = first_failure
        .map(|failure| failure.frame_index.saturating_sub(checkpoint_frame))
        .unwrap_or(frame_end.saturating_sub(checkpoint_frame));
    let locality = match frames_to_failure {
        0..=60 => 120,
        61..=120 => 80,
        121..=240 => 40,
        _ => 10,
    };
    let clarity = match subsystem_count {
        1 => 120,
        2 => 80,
        3 => 30,
        0 => 0,
        _ => 10,
    };
    source_weight + severity + locality + clarity
}

fn apply_case_queue_duplicate_penalties(cases: &mut [CaseQueueCandidate]) {
    cases.sort_by(case_queue_sort_key);
    let mut seen: Vec<(String, String, usize, String)> = Vec::new();
    for candidate in cases.iter_mut() {
        let Some(field) = candidate.first_bad_field.as_ref() else {
            continue;
        };
        if let Some((winner, _source_slug, checkpoint, _prior_field)) =
            seen.iter()
                .find(|(_, source_slug, checkpoint, prior_field)| {
                    prior_field == field
                        && *source_slug == candidate.source_slug
                        && candidate.checkpoint_frame.abs_diff(*checkpoint) <= 30
                })
        {
            candidate.score -= 75;
            candidate.duplicate_of = Some(winner.clone());
            candidate.rank_reason = format!(
                "{}; duplicate-of={} @{}",
                candidate.rank_reason, winner, checkpoint
            );
        } else {
            seen.push((
                candidate.case_id.clone(),
                candidate.source_slug.clone(),
                candidate.checkpoint_frame,
                field.clone(),
            ));
        }
        candidate.rank_reason = rank_reason(candidate);
    }
}

fn case_queue_sort_key(
    left: &CaseQueueCandidate,
    right: &CaseQueueCandidate,
) -> std::cmp::Ordering {
    right
        .score
        .cmp(&left.score)
        .then_with(|| left.frame_end.cmp(&right.frame_end))
        .then_with(|| left.case_id.cmp(&right.case_id))
}

fn rank_reason(candidate: &CaseQueueCandidate) -> String {
    let mut parts = vec![format!("source={}", candidate.source_kind)];
    if let Some(classification) = candidate.failure_class {
        parts.push(format!("failure={}", classification.as_str()));
    }
    if let Some(field) = candidate.first_bad_field.as_deref() {
        parts.push(format!("field={field}"));
    }
    parts.push(format!(
        "window={}f",
        candidate.frame_end - candidate.frame_start
    ));
    if candidate.autonomy_ready {
        parts.push("autonomy-ready".to_owned());
    }
    if let Some(duplicate_of) = candidate.duplicate_of.as_deref() {
        parts.push(format!("duplicate-of={duplicate_of}"));
    }
    parts.join(", ")
}

fn repo_relative_path(repo_root: &Path, path: &Path) -> PathBuf {
    path.strip_prefix(repo_root)
        .map(|relative| relative.to_path_buf())
        .unwrap_or_else(|_| path.to_path_buf())
}

fn case_queue_task(
    candidate: &CaseQueueCandidate,
    queue: &CaseQueueSnapshot,
    note: &NextAgentNoteDirective,
) -> TaskPackage {
    let verification_commands = vec![
        format!(
            "cargo run -p kidcli -- case verify --case {}",
            candidate.selector
        ),
        format!(
            "cargo run -p kidcli -- case summarise --case {}",
            candidate.selector
        ),
    ];
    let mut done_criteria = vec![
        "The selected case is re-verified before investigation.".to_owned(),
        "The first bad semantic frame/field is either removed or moved later with a better explanation.".to_owned(),
    ];
    if candidate.autonomy_ready {
        done_criteria.push(
            "If a fix lands, keep the case autonomy-executable and locally re-checkable."
                .to_owned(),
        );
    }
    TaskPackage {
        capability: CapabilityId::SourcePortCaseQueueTask,
        title: format!("Advance source-port case: {}", candidate.case_id),
        summary: format!(
            "{} from {} first fails on frame {} at {}; {}.",
            candidate.case_id,
            candidate.source_slug,
            candidate
                .first_bad_frame
                .map(|frame| frame.to_string())
                .unwrap_or_else(|| "unknown".to_owned()),
            candidate.first_bad_field.as_deref().unwrap_or("unknown"),
            if candidate.autonomy_ready {
                "the hybrid gate allows a focused patch"
            } else {
                "this remains evidence-first until the case is tighter"
            }
        ),
        sources: vec![
            candidate.manifest_path.display().to_string(),
            candidate.verify_path.display().to_string(),
            candidate.summary_path.display().to_string(),
            SOURCE_PORT_RESEARCH_MAP_PATH.to_owned(),
            "docs/source-port/methodology.md".to_owned(),
        ],
        implementation_targets: Vec::new(),
        commands: verification_commands,
        done_criteria,
        evidence: {
            let mut evidence = vec![
                EvidenceItem {
                    label: "selected-case".to_owned(),
                    status: candidate.status.as_str().to_owned(),
                    detail: candidate.rank_reason.clone(),
                    artefact_path: Some(candidate.summary_path.clone()),
                },
                EvidenceItem {
                    label: "autonomy-gate".to_owned(),
                    status: case_queue_gate_status(candidate).to_owned(),
                    detail: format!(
                        "failure_class={}, subsystems={}, window={}f",
                        candidate
                            .failure_class
                            .map(|class| class.as_str())
                            .unwrap_or("unknown"),
                        candidate.likely_subsystems.len(),
                        candidate.frame_end - candidate.frame_start
                    ),
                    artefact_path: Some(candidate.verify_path.clone()),
                },
                EvidenceItem {
                    label: "next-agent-note".to_owned(),
                    status: note.status.as_str().to_owned(),
                    detail: note.detail.clone(),
                    artefact_path: Some(note.path.clone()),
                },
            ];
            if let Some(reason) = queue.selection_reason.as_deref() {
                evidence.push(EvidenceItem {
                    label: "lane-pivot".to_owned(),
                    status: "active".to_owned(),
                    detail: reason.to_owned(),
                    artefact_path: Some(candidate.summary_path.clone()),
                });
            }
            evidence
        },
        frontier_context: None,
        case_queue_context: Some(CaseQueueContext {
            selected_case: Some(candidate.clone()),
            open_cases: queue.open_cases.clone(),
            blocked_cases: queue.blocked_cases.clone(),
            untriaged_recordings: queue.untriaged_recordings.clone(),
        }),
        delegated_goal: None,
        delegated_target: None,
        delegation_reason: queue.selection_reason.clone(),
        next_agent_note: Some(note.clone()),
    }
}

fn case_queue_next_agent_note(repo_root: &Path, active_slug: &str) -> NextAgentNoteDirective {
    let path = PathBuf::from(SOURCE_PORT_CASE_QUEUE_NOTE_PATH);
    let note_path = repo_root.join(&path);
    let (status, frontmatter_slug, detail) = note_status_for_frontier(&note_path, active_slug);
    let detail = detail
        .replace("active frontier", "selected case")
        .replace("for the active frontier", "for the selected case");
    NextAgentNoteDirective {
        path,
        status,
        detail,
        frontmatter_slug,
        requirement_summary: SOURCE_PORT_CASE_QUEUE_NOTE_REQUIREMENT_SUMMARY.to_owned(),
        required_sections: SOURCE_PORT_CASE_QUEUE_NOTE_REQUIRED_SECTIONS
            .iter()
            .map(|section| (*section).to_owned())
            .collect(),
    }
}

fn evaluate_cycle(
    repo_root: &Path,
    rom_path: &Path,
    manifest: &AssetManifest,
    spec: &GoalSpec,
    cycle: u32,
) -> Result<CycleReport> {
    let signals = collect_repository_signals(repo_root, rom_path, manifest)?;
    let capabilities = evaluate_capabilities(spec, repo_root, &signals);
    let selected_task = select_next_task(spec, &capabilities, &signals);
    let blocker = selected_task
        .as_ref()
        .and_then(|task| detect_blocker(repo_root, task))
        .or_else(|| blocked_capability(spec, repo_root, &capabilities));
    let goal_complete = capabilities
        .iter()
        .all(|capability| capability.status == CapabilityStatus::Complete);
    Ok(CycleReport {
        goal: spec.goal_id.clone(),
        cycle,
        goal_complete,
        stalled: false,
        task_package: selected_task.clone(),
        selected_task,
        blocker,
        progress_fingerprint: fingerprint_for_capabilities(&capabilities),
        capabilities,
    })
}

fn collect_repository_signals(
    repo_root: &Path,
    rom_path: &Path,
    manifest: &AssetManifest,
) -> Result<RepositorySignals> {
    let native_bootstrap = run_check(
        repo_root
            .join(".cache/native-oracle-runs")
            .join(Scenario::BootTitleStartLevel1.slug()),
        || verify_native_bootstrap(manifest),
    );
    let boot_oracle = run_check(
        repo_root
            .join(".cache/native-oracle-runs")
            .join(Scenario::BootTitleStartLevel1.slug()),
        || {
            compare_native_oracle(
                repo_root,
                rom_path,
                manifest,
                Scenario::BootTitleStartLevel1,
            )
        },
    );
    let move_oracle = run_check(
        repo_root
            .join(".cache/native-oracle-runs")
            .join(Scenario::BootTitleStartLevel1MoveRight5s.slug()),
        || {
            compare_native_oracle(
                repo_root,
                rom_path,
                manifest,
                Scenario::BootTitleStartLevel1MoveRight5s,
            )
        },
    );
    let pause_oracle = run_check(
        repo_root
            .join(".cache/native-gameplay-oracle-runs")
            .join(GameplayScenario::PauseResumeLevel1.slug()),
        || {
            compare_native_gameplay(
                repo_root,
                rom_path,
                manifest,
                GameplayScenario::PauseResumeLevel1,
            )
        },
    );
    let death_oracle = run_check(
        repo_root
            .join(".cache/native-gameplay-oracle-runs")
            .join(GameplayScenario::DeathRespawnLevel1.slug()),
        || {
            compare_native_gameplay(
                repo_root,
                rom_path,
                manifest,
                GameplayScenario::DeathRespawnLevel1,
            )
        },
    );
    let progress_two_transitions_oracle = run_check(
        repo_root
            .join(".cache/native-gameplay-oracle-runs")
            .join(GameplayScenario::ProgressTwoTransitions.slug()),
        || {
            compare_native_gameplay(
                repo_root,
                rom_path,
                manifest,
                GameplayScenario::ProgressTwoTransitions,
            )
        },
    );

    let level_one = manifest
        .levels
        .iter()
        .find(|level| level.level_id == LEVEL_ONE_ID);
    let has_level_one_header = level_one
        .and_then(|level| level.header_metadata.as_ref())
        .is_some();
    let has_level_one_start_and_flag = level_one
        .and_then(|level| level.header_metadata.as_ref())
        .is_some_and(|header| {
            header.player_start_x > 0
                && header.player_start_y > 0
                && header.flag_x > 0
                && header.flag_y > 0
        });
    let enemy_layout_count = level_one
        .and_then(|level| level.enemy_layout_metadata.as_ref())
        .map(|layout| layout.object_count);
    let platform_layout_count = level_one
        .and_then(|level| level.platform_metadata.as_ref())
        .map(|layout| layout.placements.len());

    let kid_core_source = fs::read_to_string(repo_root.join("crates/kid-core/src/lib.rs"))
        .with_context(|| "failed to read crates/kid-core/src/lib.rs".to_owned())?;
    let dossier = fs::read_to_string(repo_root.join("docs/source-port/level-1-dossier.md"))
        .with_context(|| "failed to read docs/source-port/level-1-dossier.md".to_owned())?;
    let gameplay_dossier =
        fs::read_to_string(repo_root.join("docs/source-port/level-1-gameplay-lifecycle.md"))
            .with_context(|| {
                "failed to read docs/source-port/level-1-gameplay-lifecycle.md".to_owned()
            })?;
    let readme = fs::read_to_string(repo_root.join("README.md"))
        .with_context(|| "failed to read README.md".to_owned())?;
    let research_map = fs::read_to_string(repo_root.join("docs/source-port/research-map.md"))
        .with_context(|| "failed to read docs/source-port/research-map.md".to_owned())?;

    let mut dossier_open_markers = BTreeMap::new();
    dossier_open_markers.insert(
        CapabilityId::PlayerCameraMovement,
        dossier.contains(PLAYER_CAMERA_TASK_MARKER),
    );
    dossier_open_markers.insert(
        CapabilityId::EnemyActivation,
        dossier.contains(ENEMY_TASK_MARKER),
    );
    dossier_open_markers.insert(
        CapabilityId::PlatformActivation,
        dossier.contains(PLATFORM_TASK_MARKER),
    );
    dossier_open_markers.insert(
        CapabilityId::OpeningCollision,
        dossier.contains(COLLISION_TASK_MARKER),
    );
    dossier_open_markers.insert(
        CapabilityId::DamageTriggerResearch,
        gameplay_dossier.contains(DAMAGE_TRIGGER_TASK_MARKER),
    );

    Ok(RepositorySignals {
        native_bootstrap,
        boot_oracle,
        move_oracle,
        pause_oracle,
        death_oracle,
        progress_two_transitions_oracle,
        release_gate: latest_release_gate_signal(repo_root),
        state_sweep: latest_state_sweep_signal(repo_root),
        has_level_one_header,
        has_level_one_start_and_flag,
        enemy_layout_count,
        platform_layout_count,
        scripted_level_one_movement_present: kid_core_source.contains("fn scripted_player_state")
            || (kid_core_source.contains("const MOVE_MID_CAMERA_X")
                && kid_core_source.contains("const FOLLOW_CAMERA_KID_X_OFFSET")),
        flat_floor_fallback_present: kid_core_source
            .contains("Flat floor fallback when no block grid is loaded")
            || kid_core_source.contains("self.apply_floor_collision(None);"),
        traversal_docs_aligned: repo_root
            .join("tasks/prd-level-1-full-traversal.md")
            .is_file()
            && repo_root
                .join("docs/source-port/level-1-full-traversal.md")
                .is_file()
            && repo_root
                .join("docs/source-port/level-1-full-traversal-goal.json")
                .is_file()
            && readme.contains(README_TRAVERSAL_FRONTIER_MARKER)
            && research_map.contains(RESEARCH_MAP_TRAVERSAL_MARKER),
        dossier_open_markers,
    })
}

fn latest_release_gate_signal(repo_root: &Path) -> CheckOutcome {
    let latest_path = repo_root
        .join(".cache/release-gate")
        .join("1p-accurate")
        .join("latest.json");
    let bytes = match fs::read(&latest_path) {
        Ok(bytes) => bytes,
        Err(error) => {
            return CheckOutcome::failing(
                format!("missing release gate artefact: {error}"),
                Some(latest_path),
            );
        }
    };
    let report: serde_json::Value = match serde_json::from_slice(&bytes) {
        Ok(report) => report,
        Err(error) => {
            return CheckOutcome::failing(
                format!("failed to parse release gate artefact: {error}"),
                Some(latest_path),
            );
        }
    };
    if report
        .get("passed")
        .and_then(serde_json::Value::as_bool)
        .unwrap_or(false)
    {
        CheckOutcome::passing("latest release gate passed", Some(latest_path))
    } else {
        CheckOutcome::failing("latest release gate did not pass", Some(latest_path))
    }
}

fn latest_state_sweep_signal(repo_root: &Path) -> CheckOutcome {
    let history_path = repo_root.join(".cache/metrics/sweep-history.jsonl");
    let contents = match fs::read_to_string(&history_path) {
        Ok(contents) => contents,
        Err(error) => {
            return CheckOutcome::failing(
                format!("missing sweep history artefact: {error}"),
                Some(history_path),
            );
        }
    };
    let latest = contents.lines().rev().find_map(|line| {
        let parsed = serde_json::from_str::<serde_json::Value>(line).ok()?;
        if parsed
            .get("tier")
            .and_then(serde_json::Value::as_str)
            .is_some_and(|tier| tier == "state")
        {
            Some(parsed)
        } else {
            None
        }
    });
    let Some(latest) = latest else {
        return CheckOutcome::failing(
            "no state-tier sweep run found in sweep history",
            Some(history_path),
        );
    };
    let passed = latest
        .get("failed")
        .and_then(serde_json::Value::as_u64)
        .unwrap_or(1)
        == 0
        && latest
            .get("errors")
            .and_then(serde_json::Value::as_u64)
            .unwrap_or(1)
            == 0;
    if passed {
        CheckOutcome::passing("latest state sweep passed", Some(history_path))
    } else {
        CheckOutcome::failing("latest state sweep reported failures", Some(history_path))
    }
}

fn run_check<F>(artefact_path: PathBuf, check: F) -> CheckOutcome
where
    F: FnOnce() -> Result<()>,
{
    match check() {
        Ok(()) => CheckOutcome::passing("passed", Some(artefact_path)),
        Err(error) => CheckOutcome::failing(error.to_string(), Some(artefact_path)),
    }
}

fn evaluate_capabilities(
    spec: &GoalSpec,
    repo_root: &Path,
    signals: &RepositorySignals,
) -> Vec<CapabilityReport> {
    let mut reports = Vec::with_capacity(spec.capabilities.len());
    let mut status_by_id = BTreeMap::new();

    for capability in &spec.capabilities {
        let prerequisites_complete = capability.prerequisites.iter().all(|prerequisite| {
            status_by_id.get(prerequisite) == Some(&CapabilityStatus::Complete)
        });
        let (complete, summary) = capability_completion_summary(capability.id, signals);
        let status = if complete {
            CapabilityStatus::Complete
        } else if !prerequisites_complete {
            CapabilityStatus::Pending
        } else if capability
            .sources
            .iter()
            .map(|source| repo_root.join(source))
            .all(|path| path.exists())
        {
            CapabilityStatus::Ready
        } else {
            CapabilityStatus::Blocked
        };
        status_by_id.insert(capability.id, status);
        reports.push(CapabilityReport {
            id: capability.id,
            title: capability.title.clone(),
            status,
            summary,
        });
    }

    reports
}

fn capability_completion_summary(
    capability: CapabilityId,
    signals: &RepositorySignals,
) -> (bool, String) {
    match capability {
        CapabilityId::BootTitleFlow => (
            signals.native_bootstrap.ok,
            format!(
                "verify-native-bootstrap {}",
                if signals.native_bootstrap.ok {
                    "passes"
                } else {
                    "fails"
                }
            ),
        ),
        CapabilityId::LevelLoadAndSpawn => (
            signals.boot_oracle.ok
                && signals.has_level_one_header
                && signals.has_level_one_start_and_flag,
            if signals.boot_oracle.ok {
                "Level 1 header metadata and spawn/flag positions are present and the level-entry oracle comparison passes.".to_owned()
            } else {
                "Level 1 load/spawn metadata exists, but the level-entry oracle comparison still diverges.".to_owned()
            },
        ),
        CapabilityId::LevelSettle => (
            signals.boot_oracle.ok,
            if signals.boot_oracle.ok {
                "The settle checkpoint matches the oracle for boot-title-start-level1.".to_owned()
            } else {
                format!(
                    "The settle checkpoint still diverges: {}",
                    signals.boot_oracle.detail
                )
            },
        ),
        CapabilityId::PlayerCameraMovement => (
            !signals.scripted_level_one_movement_present,
            if signals.scripted_level_one_movement_present {
                "Static scan still finds scripted Level 1 movement/camera heuristics in kid-core."
                    .to_owned()
            } else {
                "Static scan no longer finds the scripted Level 1 movement/camera heuristics."
                    .to_owned()
            },
        ),
        CapabilityId::OpeningScroll => (
            signals.move_oracle.ok && !signals.scripted_level_one_movement_present,
            if signals.move_oracle.ok && !signals.scripted_level_one_movement_present {
                "The first-scroll scenario matches the oracle without the scripted movement path."
                    .to_owned()
            } else if !signals.move_oracle.ok {
                format!(
                    "The first-scroll scenario still diverges: {}",
                    signals.move_oracle.detail
                )
            } else {
                "The movement oracle passes, but the scripted Level 1 movement path is still present.".to_owned()
            },
        ),
        CapabilityId::OpeningCollision => (
            !signals
                .dossier_open_markers
                .get(&CapabilityId::OpeningCollision)
                .copied()
                .unwrap_or(true),
            if signals
                .dossier_open_markers
                .get(&CapabilityId::OpeningCollision)
                .copied()
                .unwrap_or(true)
            {
                "The dossier still lists the opening collision/terrain research task as open."
                    .to_owned()
            } else {
                "The dossier no longer marks the opening collision/terrain task as open.".to_owned()
            },
        ),
        CapabilityId::PlatformActivation => (
            signals.platform_layout_count.unwrap_or(0) > 0
                && !signals
                    .dossier_open_markers
                    .get(&CapabilityId::PlatformActivation)
                    .copied()
                    .unwrap_or(true),
            if signals.platform_layout_count.unwrap_or(0) == 0 {
                "No Level 1 platform placements are available in the generated bundle.".to_owned()
            } else if signals
                .dossier_open_markers
                .get(&CapabilityId::PlatformActivation)
                .copied()
                .unwrap_or(true)
            {
                "The dossier still lists the Level 1 platform-manager tracing task as open."
                    .to_owned()
            } else {
                "Level 1 platform placements exist and the platform-manager task is closed in the dossier.".to_owned()
            },
        ),
        CapabilityId::EnemyActivation => (
            signals.enemy_layout_count.unwrap_or(0) > 0
                && !signals
                    .dossier_open_markers
                    .get(&CapabilityId::EnemyActivation)
                    .copied()
                    .unwrap_or(true),
            if signals.enemy_layout_count.unwrap_or(0) == 0 {
                "No Level 1 enemy objects are available in the generated bundle.".to_owned()
            } else if signals
                .dossier_open_markers
                .get(&CapabilityId::EnemyActivation)
                .copied()
                .unwrap_or(true)
            {
                "The dossier still lists the first visible enemy-family mapping task as open."
                    .to_owned()
            } else {
                "Level 1 enemy objects exist and the enemy-family mapping task is closed in the dossier.".to_owned()
            },
        ),
        CapabilityId::PauseResumeOracle => (
            signals.pause_oracle.ok,
            if signals.pause_oracle.ok {
                "The pause/resume gameplay checkpoints match the oracle capture.".to_owned()
            } else {
                format!(
                    "The pause/resume gameplay checkpoints still diverge: {}",
                    signals.pause_oracle.detail
                )
            },
        ),
        CapabilityId::DamageTriggerResearch => (
            !signals
                .dossier_open_markers
                .get(&CapabilityId::DamageTriggerResearch)
                .copied()
                .unwrap_or(true),
            if signals
                .dossier_open_markers
                .get(&CapabilityId::DamageTriggerResearch)
                .copied()
                .unwrap_or(true)
            {
                "The gameplay lifecycle dossier still marks the Level 1 damage-trigger task as open."
                    .to_owned()
            } else {
                "The gameplay lifecycle dossier no longer marks the Level 1 damage-trigger task as open.".to_owned()
            },
        ),
        CapabilityId::DeathRespawnParity => (
            signals.death_oracle.ok,
            if signals.death_oracle.ok {
                "The death/respawn gameplay checkpoints match the oracle capture.".to_owned()
            } else {
                format!(
                    "The death/respawn gameplay checkpoints still diverge: {}",
                    signals.death_oracle.detail
                )
            },
        ),
        CapabilityId::GoalLoopCompletion => (
            signals.pause_oracle.ok
                && signals.death_oracle.ok
                && !signals
                    .dossier_open_markers
                    .get(&CapabilityId::DamageTriggerResearch)
                    .copied()
                    .unwrap_or(true),
            if signals.pause_oracle.ok
                && signals.death_oracle.ok
                && !signals
                    .dossier_open_markers
                    .get(&CapabilityId::DamageTriggerResearch)
                    .copied()
                    .unwrap_or(true)
            {
                "The gameplay lifecycle slice contract is fully satisfied.".to_owned()
            } else {
                "The gameplay lifecycle slice still has unresolved lifecycle work.".to_owned()
            },
        ),
        CapabilityId::TraversalDocsAligned => (
            signals.traversal_docs_aligned,
            if signals.traversal_docs_aligned {
                "Tracked docs and autonomy inputs point at Level 1 full traversal as the active frontier.".to_owned()
            } else {
                "Tracked docs are not yet aligned on the Level 1 full traversal frontier."
                    .to_owned()
            },
        ),
        CapabilityId::TerrainRuntimeAuthority => (
            !signals.flat_floor_fallback_present,
            if signals.flat_floor_fallback_present {
                "Static scan still finds the default flat-floor fallback in kid-core.".to_owned()
            } else {
                "Static scan no longer finds the default flat-floor fallback in kid-core."
                    .to_owned()
            },
        ),
        CapabilityId::ProgressTwoTransitionsOracle => (
            signals.progress_two_transitions_oracle.ok,
            if signals.progress_two_transitions_oracle.ok {
                "The progress-two-transitions gameplay comparison matches the oracle.".to_owned()
            } else {
                format!(
                    "The progress-two-transitions gameplay comparison still diverges: {}",
                    signals.progress_two_transitions_oracle.detail
                )
            },
        ),
        CapabilityId::TraversalGoalLoopCompletion => (
            signals.traversal_docs_aligned
                && !signals.flat_floor_fallback_present
                && signals.progress_two_transitions_oracle.ok,
            if signals.traversal_docs_aligned
                && !signals.flat_floor_fallback_present
                && signals.progress_two_transitions_oracle.ok
            {
                "The full traversal slice contract is fully satisfied.".to_owned()
            } else {
                "The full traversal slice still has unresolved traversal work.".to_owned()
            },
        ),
        CapabilityId::ReleaseGateGreen => (
            signals.release_gate.ok,
            if signals.release_gate.ok {
                "The latest release-gate artefact reports a passing 1p-accurate gate.".to_owned()
            } else {
                format!(
                    "The latest release-gate artefact is missing or failing: {}",
                    signals.release_gate.detail
                )
            },
        ),
        CapabilityId::StateSweepGreen => (
            signals.state_sweep.ok,
            if signals.state_sweep.ok {
                "The latest state-tier sweep artefact reports no failures or errors.".to_owned()
            } else {
                format!(
                    "The latest state-tier sweep artefact is missing or failing: {}",
                    signals.state_sweep.detail
                )
            },
        ),
        CapabilityId::SourcePortFrontierTask => (
            false,
            "The live source-port frontier task package is generated from frontier-state.yaml."
                .to_owned(),
        ),
        CapabilityId::SourcePortCaseQueueTask => (
            false,
            "The source-port case queue task package is generated from reduced case bundles."
                .to_owned(),
        ),
    }
}

fn select_next_task(
    spec: &GoalSpec,
    capabilities: &[CapabilityReport],
    signals: &RepositorySignals,
) -> Option<TaskPackage> {
    let capability = capabilities
        .iter()
        .find(|capability| capability.status == CapabilityStatus::Ready)?;
    let spec_capability = spec
        .capabilities
        .iter()
        .find(|candidate| candidate.id == capability.id)?;
    Some(TaskPackage {
        capability: capability.id,
        title: spec_capability.title.clone(),
        summary: spec_capability.summary.clone(),
        sources: spec_capability.sources.clone(),
        implementation_targets: spec_capability.implementation_targets.clone(),
        commands: spec_capability.commands.clone(),
        done_criteria: spec_capability.done_criteria.clone(),
        evidence: evidence_for_capability(capability.id, signals),
        frontier_context: None,
        case_queue_context: None,
        delegated_goal: None,
        delegated_target: None,
        delegation_reason: None,
        next_agent_note: None,
    })
}

fn evidence_for_capability(
    capability: CapabilityId,
    signals: &RepositorySignals,
) -> Vec<EvidenceItem> {
    match capability {
        CapabilityId::BootTitleFlow => vec![check_evidence(
            "verify-native-bootstrap",
            &signals.native_bootstrap,
        )],
        CapabilityId::LevelLoadAndSpawn | CapabilityId::LevelSettle => vec![
            check_evidence("compare-native-oracle:boot-title-start-level1", &signals.boot_oracle),
            EvidenceItem {
                label: "level-1-manifest".to_owned(),
                status: if signals.has_level_one_header {
                    "present".to_owned()
                } else {
                    "missing".to_owned()
                },
                detail: format!(
                    "header={} start_and_flag={}",
                    signals.has_level_one_header, signals.has_level_one_start_and_flag
                ),
                artefact_path: None,
            },
        ],
        CapabilityId::PlayerCameraMovement => vec![EvidenceItem {
            label: "kid-core-static-scan".to_owned(),
            status: if signals.scripted_level_one_movement_present {
                "heuristics-present".to_owned()
            } else {
                "heuristics-removed".to_owned()
            },
            detail: "Checks for `scripted_player_state` and hard-coded Level 1 camera-follow constants.".to_owned(),
            artefact_path: Some(PathBuf::from("crates/kid-core/src/lib.rs")),
        }],
        CapabilityId::OpeningScroll => vec![
            check_evidence(
                "compare-native-oracle:boot-title-start-level1-move-right5s",
                &signals.move_oracle,
            ),
            EvidenceItem {
                label: "kid-core-static-scan".to_owned(),
                status: if signals.scripted_level_one_movement_present {
                    "heuristics-present".to_owned()
                } else {
                    "heuristics-removed".to_owned()
                },
                detail: "Opening-scroll parity is only complete once the scripted movement path is gone.".to_owned(),
                artefact_path: Some(PathBuf::from("crates/kid-core/src/lib.rs")),
            },
        ],
        CapabilityId::OpeningCollision => vec![EvidenceItem {
            label: "level-1-dossier".to_owned(),
            status: if signals
                .dossier_open_markers
                .get(&CapabilityId::OpeningCollision)
                .copied()
                .unwrap_or(false)
            {
                "task-open".to_owned()
            } else {
                "task-closed".to_owned()
            },
            detail: "Tracks whether the opening collision/terrain task remains open in the dossier.".to_owned(),
            artefact_path: Some(PathBuf::from("docs/source-port/level-1-dossier.md")),
        }],
        CapabilityId::PlatformActivation => vec![EvidenceItem {
            label: "level-1-platform-layout".to_owned(),
            status: format!("placements={}", signals.platform_layout_count.unwrap_or(0)),
            detail: "Uses generated Level 1 platform placement count plus dossier task state.".to_owned(),
            artefact_path: Some(PathBuf::from("docs/source-port/level-1-dossier.md")),
        }],
        CapabilityId::EnemyActivation => vec![EvidenceItem {
            label: "level-1-enemy-layout".to_owned(),
            status: format!("objects={}", signals.enemy_layout_count.unwrap_or(0)),
            detail: "Uses generated Level 1 enemy object count plus dossier task state.".to_owned(),
            artefact_path: Some(PathBuf::from("docs/source-port/level-1-dossier.md")),
        }],
        CapabilityId::PauseResumeOracle => vec![check_evidence(
            "compare-native-gameplay:pause-resume-level1",
            &signals.pause_oracle,
        )],
        CapabilityId::DamageTriggerResearch => vec![EvidenceItem {
            label: "level-1-gameplay-lifecycle-dossier".to_owned(),
            status: if signals
                .dossier_open_markers
                .get(&CapabilityId::DamageTriggerResearch)
                .copied()
                .unwrap_or(false)
            {
                "task-open".to_owned()
            } else {
                "task-closed".to_owned()
            },
            detail: "Tracks whether the gameplay lifecycle damage-trigger task remains open in the dossier.".to_owned(),
            artefact_path: Some(PathBuf::from("docs/source-port/level-1-gameplay-lifecycle.md")),
        }],
        CapabilityId::DeathRespawnParity => vec![check_evidence(
            "compare-native-gameplay:death-respawn-level1",
            &signals.death_oracle,
        )],
        CapabilityId::GoalLoopCompletion => vec![
            check_evidence("compare-native-gameplay:pause-resume-level1", &signals.pause_oracle),
            check_evidence("compare-native-gameplay:death-respawn-level1", &signals.death_oracle),
        ],
        CapabilityId::TraversalDocsAligned => vec![EvidenceItem {
            label: "tracked-traversal-docs".to_owned(),
            status: if signals.traversal_docs_aligned {
                "aligned".to_owned()
            } else {
                "stale".to_owned()
            },
            detail: "Checks for the tracked traversal PRD/dossier/goal plus aligned README and research-map frontier text.".to_owned(),
            artefact_path: Some(PathBuf::from("docs/source-port/level-1-full-traversal.md")),
        }],
        CapabilityId::TerrainRuntimeAuthority => vec![EvidenceItem {
            label: "kid-core-flat-floor-scan".to_owned(),
            status: if signals.flat_floor_fallback_present {
                "fallback-present".to_owned()
            } else {
                "fallback-removed".to_owned()
            },
            detail: "Static scan for the default flat-floor fallback in the runtime path.".to_owned(),
            artefact_path: Some(PathBuf::from("crates/kid-core/src/lib.rs")),
        }],
        CapabilityId::ProgressTwoTransitionsOracle => vec![check_evidence(
            "compare-native-gameplay:progress-two-transitions",
            &signals.progress_two_transitions_oracle,
        )],
        CapabilityId::TraversalGoalLoopCompletion => vec![
            EvidenceItem {
                label: "tracked-traversal-docs".to_owned(),
                status: if signals.traversal_docs_aligned {
                    "aligned".to_owned()
                } else {
                    "stale".to_owned()
                },
                detail: "Checks whether tracked docs provide the traversal resume path.".to_owned(),
                artefact_path: Some(PathBuf::from("docs/source-port/level-1-full-traversal.md")),
            },
            EvidenceItem {
                label: "kid-core-flat-floor-scan".to_owned(),
                status: if signals.flat_floor_fallback_present {
                    "fallback-present".to_owned()
                } else {
                    "fallback-removed".to_owned()
                },
                detail: "Traversal closure requires the default runtime path to stop using the flat-floor fallback.".to_owned(),
                artefact_path: Some(PathBuf::from("crates/kid-core/src/lib.rs")),
            },
            check_evidence(
                "compare-native-gameplay:progress-two-transitions",
                &signals.progress_two_transitions_oracle,
            ),
        ],
        CapabilityId::ReleaseGateGreen => {
            vec![check_evidence("release-gate:1p-accurate", &signals.release_gate)]
        }
        CapabilityId::StateSweepGreen => {
            vec![check_evidence("sweep:state", &signals.state_sweep)]
        }
        CapabilityId::SourcePortFrontierTask => Vec::new(),
        CapabilityId::SourcePortCaseQueueTask => Vec::new(),
    }
}

fn check_evidence(label: &str, check: &CheckOutcome) -> EvidenceItem {
    EvidenceItem {
        label: label.to_owned(),
        status: if check.ok {
            "passed".to_owned()
        } else {
            "failed".to_owned()
        },
        detail: check.detail.clone(),
        artefact_path: check.artefact_path.clone(),
    }
}

fn blocked_capability(
    spec: &GoalSpec,
    repo_root: &Path,
    capabilities: &[CapabilityReport],
) -> Option<BlockerReport> {
    let blocked = capabilities
        .iter()
        .find(|capability| capability.status == CapabilityStatus::Blocked)?;
    let spec_capability = spec
        .capabilities
        .iter()
        .find(|candidate| candidate.id == blocked.id)?;
    let missing_paths = spec_capability
        .sources
        .iter()
        .map(|source| repo_root.join(source))
        .filter(|path| !path.exists())
        .collect::<Vec<_>>();
    Some(BlockerReport {
        capability: blocked.id,
        reason: "required local source material is missing".to_owned(),
        missing_paths,
    })
}

fn detect_blocker(repo_root: &Path, task: &TaskPackage) -> Option<BlockerReport> {
    let missing_paths = task
        .sources
        .iter()
        .map(|source| repo_root.join(source))
        .filter(|path| !path.exists())
        .collect::<Vec<_>>();
    if missing_paths.is_empty() {
        return None;
    }
    Some(BlockerReport {
        capability: task.capability,
        reason: "required local source material is missing".to_owned(),
        missing_paths,
    })
}

fn fingerprint_for_capabilities(capabilities: &[CapabilityReport]) -> String {
    capabilities
        .iter()
        .map(|capability| format!("{}:{:?}", capability.id.as_str(), capability.status))
        .collect::<Vec<_>>()
        .join("|")
}

fn write_json<T>(path: &Path, value: &T) -> Result<()>
where
    T: Serialize,
{
    fs::write(path, serde_json::to_vec_pretty(value)?)
        .with_context(|| format!("failed to write {}", path.display()))
}

fn write_source_port_next_agent_note(
    repo_root: &Path,
    note: &NextAgentNoteDirective,
    frontier: &FrontierContext,
    task: &TaskPackage,
) -> Result<()> {
    let note_path = repo_root.join(&note.path);
    if let Some(parent) = note_path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("failed to create {}", parent.display()))?;
    }
    fs::write(
        &note_path,
        render_source_port_next_agent_note(frontier, task),
    )
    .with_context(|| format!("failed to write {}", note_path.display()))
}

fn render_source_port_next_agent_note(frontier: &FrontierContext, task: &TaskPackage) -> String {
    let closed_frontier = frontier.status == "complete";
    let exact_next_moves = if closed_frontier {
        let mut moves = vec![format!(
            "Do not reopen TAS-frontier work; continue through delegated lane `{}`.",
            task.delegated_goal
                .map(AutonomyGoal::slug)
                .unwrap_or("source-port-frontier")
        )];
        if let Some(target) = task.delegated_target.as_deref() {
            moves.push(format!(
                "Target `{target}` for the next concrete source-port step."
            ));
        }
        if let Some(reason) = task.delegation_reason.as_deref() {
            moves.push(reason.to_owned());
        }
        moves.extend(task.done_criteria.iter().cloned());
        moves
    } else {
        frontier.required_next_evidence.clone()
    };
    let verification_commands = if closed_frontier {
        task.commands.clone()
    } else {
        frontier.verification_commands.clone()
    };
    format!(
        "\
---
slug: {slug}
updated_at: {updated_at}
status: {status}
---
# What Changed
- {what_changed}

# Current Model
- Recording: {recording}
- Area: {area}
- Authoritative clean through: frame {clean_through}
- Current blocker: {blocker}

# Strongest Evidence
- Hypothesis: {hypothesis}: {reason}

# Exact Next Moves
{exact_next_moves}

# Verification To Re-run
{verification_commands}

# Dead Ends / Do Not Retry
- Re-open TAS continuation only if a new authoritative mismatch exists.
",
        slug = frontier.slug,
        updated_at = autonomy_timestamp(),
        status = frontier.status,
        what_changed = if closed_frontier {
            format!(
                "The TAS frontier is closed through frame {}; the next loop is delegated to {}{}.",
                frontier.authoritative_clean_through,
                task.delegated_goal
                    .map(AutonomyGoal::slug)
                    .unwrap_or("source-port-frontier"),
                task.delegated_target
                    .as_deref()
                    .map(|target| format!(" ({target})"))
                    .unwrap_or_default()
            )
        } else {
            format!(
                "The active TAS frontier remains `{}` at frame {}.",
                frontier.slug, frontier.frontier_frame
            )
        },
        recording = frontier.recording,
        area = frontier.area,
        clean_through = frontier.authoritative_clean_through,
        blocker = frontier.current_blocker,
        hypothesis = frontier.current_hypothesis.name,
        reason = frontier.current_hypothesis.reason,
        exact_next_moves = exact_next_moves
            .iter()
            .map(|step| format!("- {step}"))
            .collect::<Vec<_>>()
            .join("\n"),
        verification_commands = verification_commands
            .iter()
            .map(|command| format!("- {command}"))
            .collect::<Vec<_>>()
            .join("\n"),
    )
}

fn autonomy_timestamp() -> String {
    let seconds = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    seconds.to_string()
}

fn print_cycle_summary(cycle: &CycleReport) {
    println!("autonomy goal: {}", cycle.goal);
    println!("cycle: {}", cycle.cycle);
    if cycle.goal_complete {
        println!("status: complete");
        return;
    }
    if let Some(blocker) = &cycle.blocker {
        println!(
            "status: blocked on {} ({})",
            blocker.capability.as_str(),
            blocker.reason
        );
        return;
    }
    if cycle.stalled {
        println!("status: stalled");
    } else {
        println!("status: incomplete");
    }
    if let Some(task) = &cycle.selected_task {
        println!("next task: {}", task.capability.as_str());
        println!("task summary: {}", task.summary);
    }
}

#[cfg(test)]
mod tests {
    use super::{
        CapabilityId, CapabilityStatus, CaseFailureClass, CaseQueueStatus, CheckOutcome, GoalSpec,
        NextAgentNoteStatus, RepositorySignals, SOURCE_PORT_CASE_CACHE_PATH,
        SOURCE_PORT_CASE_QUEUE_NOTE_PATH, SOURCE_PORT_FRONTIER_NOTE_PATH,
        SOURCE_PORT_FRONTIER_STATE_PATH, SOURCE_PORT_RECORDINGS_CACHE_PATH,
        SOURCE_PORT_RELEASE_GATE_PATH, SOURCE_PORT_RESEARCH_MAP_PATH, SOURCE_PORT_ROADMAP_PATH,
        SOURCE_PORT_WHOLE_GAME_INDEX_PATH, TaskPackage, build_case_queue_snapshot, detect_blocker,
        evaluate_capabilities, evaluate_case_queue_cycle, evaluate_source_port_frontier_cycle,
        fingerprint_for_capabilities, load_source_port_frontier_state, select_next_task,
        source_port_next_agent_note,
    };
    use std::collections::BTreeMap;
    use std::fs;
    use std::path::{Path, PathBuf};
    use tempfile::TempDir;

    fn goal_spec() -> GoalSpec {
        serde_json::from_str(
            r#"
            {
              "goal_id": "level1-opening-slice",
              "title": "Level 1 opening slice",
              "summary": "Test goal",
              "capabilities": [
                {
                  "id": "boot_title_flow",
                  "title": "Boot/title",
                  "summary": "Boot/title",
                  "prerequisites": [],
                  "sources": ["src/main.rs"],
                  "implementation_targets": ["../kid-core/src/lib.rs"],
                  "commands": ["cargo run -p kidcli -- verify-native-bootstrap"],
                  "done_criteria": ["bootstrap passes"]
                },
                {
                  "id": "level_load_and_spawn",
                  "title": "Level load",
                  "summary": "Level load",
                  "prerequisites": ["boot_title_flow"],
                  "sources": ["src/main.rs"],
                  "implementation_targets": ["../kid-core/src/lib.rs"],
                  "commands": ["cargo run -p kidcli -- compare-native-oracle --scenario boot-title-start-level1"],
                  "done_criteria": ["entry passes"]
                },
                {
                  "id": "level_settle",
                  "title": "Settle",
                  "summary": "Settle",
                  "prerequisites": ["level_load_and_spawn"],
                  "sources": ["src/main.rs"],
                  "implementation_targets": ["../kid-core/src/lib.rs"],
                  "commands": ["cargo run -p kidcli -- compare-native-oracle --scenario boot-title-start-level1"],
                  "done_criteria": ["settle passes"]
                },
                {
                  "id": "player_camera_movement",
                  "title": "Player/camera",
                  "summary": "Player/camera",
                  "prerequisites": ["level_settle"],
                  "sources": ["../kid-core/src/lib.rs"],
                  "implementation_targets": ["../kid-core/src/lib.rs"],
                  "commands": ["cargo test -p kid-core"],
                  "done_criteria": ["remove scripted path"]
                },
                {
                  "id": "opening_scroll",
                  "title": "Scroll",
                  "summary": "Scroll",
                  "prerequisites": ["player_camera_movement"],
                  "sources": ["src/main.rs"],
                  "implementation_targets": ["../kid-core/src/lib.rs"],
                  "commands": ["cargo run -p kidcli -- compare-native-oracle --scenario boot-title-start-level1-move-right5s"],
                  "done_criteria": ["scroll passes"]
                },
                {
                  "id": "opening_collision",
                  "title": "Collision",
                  "summary": "Collision",
                  "prerequisites": ["opening_scroll"],
                  "sources": ["src/main.rs"],
                  "implementation_targets": ["../kid-core/src/lib.rs"],
                  "commands": ["cargo test -p kid-core"],
                  "done_criteria": ["collision task closed"]
                },
                {
                  "id": "platform_activation",
                  "title": "Platforms",
                  "summary": "Platforms",
                  "prerequisites": ["opening_collision"],
                  "sources": ["src/main.rs"],
                  "implementation_targets": ["../kid-core/src/lib.rs"],
                  "commands": ["cargo run -p kidcli -- inspect-level --level 00"],
                  "done_criteria": ["platform task closed"]
                },
                {
                  "id": "enemy_activation",
                  "title": "Enemies",
                  "summary": "Enemies",
                  "prerequisites": ["opening_collision"],
                  "sources": ["src/main.rs"],
                  "implementation_targets": ["../kid-core/src/lib.rs"],
                  "commands": ["cargo run -p kidcli -- inspect-enemy --enemy 01"],
                  "done_criteria": ["enemy task closed"]
                }
              ]
            }
            "#,
        )
        .expect("goal spec should parse")
    }

    fn base_signals() -> RepositorySignals {
        let mut dossier_open_markers = BTreeMap::new();
        dossier_open_markers.insert(CapabilityId::OpeningCollision, true);
        dossier_open_markers.insert(CapabilityId::PlatformActivation, true);
        dossier_open_markers.insert(CapabilityId::EnemyActivation, true);
        dossier_open_markers.insert(CapabilityId::PlayerCameraMovement, true);
        dossier_open_markers.insert(CapabilityId::DamageTriggerResearch, true);
        RepositorySignals {
            native_bootstrap: CheckOutcome::passing("passed", None),
            boot_oracle: CheckOutcome::passing("passed", None),
            move_oracle: CheckOutcome::passing("passed", None),
            pause_oracle: CheckOutcome::passing("passed", None),
            death_oracle: CheckOutcome::passing("passed", None),
            progress_two_transitions_oracle: CheckOutcome::passing("passed", None),
            release_gate: CheckOutcome::passing("passed", None),
            state_sweep: CheckOutcome::passing("passed", None),
            has_level_one_header: true,
            has_level_one_start_and_flag: true,
            enemy_layout_count: Some(22),
            platform_layout_count: Some(1),
            scripted_level_one_movement_present: true,
            flat_floor_fallback_present: true,
            traversal_docs_aligned: false,
            dossier_open_markers,
        }
    }

    #[test]
    fn selects_player_camera_before_enemy_and_platform_work() {
        let spec = goal_spec();
        let signals = base_signals();
        let capabilities = evaluate_capabilities(&spec, Path::new("."), &signals);

        let task = select_next_task(&spec, &capabilities, &signals).expect("task should exist");
        assert_eq!(task.capability, CapabilityId::PlayerCameraMovement);
    }

    #[test]
    fn passing_scenarios_are_not_enough_while_scripted_movement_remains() {
        let spec = goal_spec();
        let signals = base_signals();
        let capabilities = evaluate_capabilities(&spec, Path::new("."), &signals);

        let movement = capabilities
            .iter()
            .find(|capability| capability.id == CapabilityId::PlayerCameraMovement)
            .expect("player/camera capability should exist");
        let scroll = capabilities
            .iter()
            .find(|capability| capability.id == CapabilityId::OpeningScroll)
            .expect("opening-scroll capability should exist");

        assert_eq!(movement.status, CapabilityStatus::Ready);
        assert_eq!(scroll.status, CapabilityStatus::Pending);
    }

    #[test]
    fn resumes_at_next_unresolved_capability() {
        let spec = goal_spec();
        let mut signals = base_signals();
        signals.scripted_level_one_movement_present = false;
        signals.move_oracle = CheckOutcome::failing("still diverged", None);
        signals
            .dossier_open_markers
            .insert(CapabilityId::PlayerCameraMovement, false);

        let capabilities = evaluate_capabilities(&spec, Path::new("."), &signals);
        let task = select_next_task(&spec, &capabilities, &signals).expect("task should exist");
        assert_eq!(task.capability, CapabilityId::OpeningScroll);
    }

    #[test]
    fn detects_missing_local_sources_as_blockers() {
        let task = TaskPackage {
            capability: CapabilityId::PlayerCameraMovement,
            title: "Player/camera".to_owned(),
            summary: "Player/camera".to_owned(),
            sources: vec!["missing/source.asm".to_owned()],
            implementation_targets: Vec::new(),
            commands: Vec::new(),
            done_criteria: Vec::new(),
            evidence: Vec::new(),
            frontier_context: None,
            case_queue_context: None,
            delegated_goal: None,
            delegated_target: None,
            delegation_reason: None,
            next_agent_note: None,
        };

        let blocker =
            detect_blocker(Path::new("/tmp/does-not-exist"), &task).expect("blocker expected");
        assert_eq!(blocker.capability, CapabilityId::PlayerCameraMovement);
        assert_eq!(
            blocker.missing_paths,
            vec![PathBuf::from("/tmp/does-not-exist/missing/source.asm")]
        );
    }

    #[test]
    fn fingerprint_changes_when_capability_status_changes() {
        let mut first = vec![super::CapabilityReport {
            id: CapabilityId::BootTitleFlow,
            title: "Boot".to_owned(),
            status: CapabilityStatus::Ready,
            summary: "ready".to_owned(),
        }];
        let second = vec![super::CapabilityReport {
            id: CapabilityId::BootTitleFlow,
            title: "Boot".to_owned(),
            status: CapabilityStatus::Complete,
            summary: "done".to_owned(),
        }];

        assert_ne!(
            fingerprint_for_capabilities(&first),
            fingerprint_for_capabilities(&second)
        );

        first[0].status = CapabilityStatus::Complete;
        assert_eq!(
            fingerprint_for_capabilities(&first),
            fingerprint_for_capabilities(&second)
        );
    }

    #[test]
    fn full_traversal_resumes_at_terrain_after_docs_align() {
        let spec: GoalSpec = serde_json::from_str(
            r#"
            {
              "goal_id": "level1-full-traversal",
              "title": "Level 1 full traversal",
              "summary": "Test goal",
              "capabilities": [
                {
                  "id": "traversal_docs_aligned",
                  "title": "Docs",
                  "summary": "Docs",
                  "prerequisites": [],
                  "sources": ["src/main.rs"],
                  "implementation_targets": ["README.md"],
                  "commands": ["cargo run -p kidcli -- autonomous-loop --goal level1-full-traversal"],
                  "done_criteria": ["docs aligned"]
                },
                {
                  "id": "terrain_runtime_authority",
                  "title": "Terrain",
                  "summary": "Terrain",
                  "prerequisites": ["traversal_docs_aligned"],
                  "sources": ["../kid-core/src/lib.rs"],
                  "implementation_targets": ["../kid-core/src/lib.rs"],
                  "commands": ["cargo run -p kidcli -- compare-native-gameplay --scenario progress-two-transitions"],
                  "done_criteria": ["flat-floor fallback removed"]
                },
                {
                  "id": "progress_two_transitions_oracle",
                  "title": "Transitions",
                  "summary": "Transitions",
                  "prerequisites": ["terrain_runtime_authority"],
                  "sources": ["src/main.rs"],
                  "implementation_targets": ["../kid-core/src/lib.rs"],
                  "commands": ["cargo run -p kidcli -- compare-native-gameplay --scenario progress-two-transitions"],
                  "done_criteria": ["transitions pass"]
                }
              ]
            }
            "#,
        )
        .expect("goal spec should parse");

        let mut signals = base_signals();
        signals.traversal_docs_aligned = true;
        let capabilities = evaluate_capabilities(&spec, Path::new("."), &signals);

        let task = select_next_task(&spec, &capabilities, &signals).expect("task should exist");
        assert_eq!(task.capability, CapabilityId::TerrainRuntimeAuthority);
    }

    fn source_port_repo() -> TempDir {
        let temp_dir = TempDir::new().expect("temp dir should exist");
        fs::create_dir_all(temp_dir.path().join("docs/source-port"))
            .expect("docs dir should exist");
        fs::create_dir_all(temp_dir.path().join("crates/kid-core/src"))
            .expect("source dir should exist");
        fs::create_dir_all(temp_dir.path().join(SOURCE_PORT_CASE_CACHE_PATH))
            .expect("case cache should exist");
        fs::create_dir_all(temp_dir.path().join(SOURCE_PORT_RECORDINGS_CACHE_PATH))
            .expect("recording cache should exist");
        fs::write(
            temp_dir.path().join("crates/kid-core/src/boot.rs"),
            "// boot placeholder\n",
        )
        .expect("boot source should write");
        fs::write(
            temp_dir.path().join("crates/kid-core/src/trace.rs"),
            "// trace placeholder\n",
        )
        .expect("trace source should write");
        fs::write(
            temp_dir.path().join("crates/kid-core/src/tests.rs"),
            "// tests placeholder\n",
        )
        .expect("tests source should write");
        temp_dir
    }

    fn write_case(
        repo_root: &Path,
        selector: &str,
        source_json: &str,
        first_bad_field: &str,
        first_bad_frame: usize,
        likely_subsystems: &[&str],
        frame_start: usize,
        frame_end: usize,
    ) {
        let case_dir = repo_root.join(SOURCE_PORT_CASE_CACHE_PATH).join(selector);
        fs::create_dir_all(&case_dir).expect("case dir should exist");
        fs::write(
            case_dir.join("manifest.json"),
            format!(
                r#"{{
  "case_id": "{selector}",
  "source": {source_json},
  "evidence_tier": "branch-matched",
  "frame_start": {frame_start},
  "frame_end": {frame_end},
  "checkpoint_frame": {frame_start},
  "checkpoint_label": "frame_{frame_start:05}"
}}
"#
            ),
        )
        .expect("manifest should write");
        fs::write(
            case_dir.join("verify.json"),
            format!(
                r#"{{
  "passed": false,
  "first_failure": {{
    "frame_index": {first_bad_frame},
    "field": "{first_bad_field}"
  }},
  "likely_subsystems": [{}]
}}
"#,
                likely_subsystems
                    .iter()
                    .map(|subsystem| format!("\"{subsystem}\""))
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
        )
        .expect("verify should write");
        fs::write(case_dir.join("summary.md"), "case summary\n").expect("summary should write");
    }

    fn write_recording(repo_root: &Path, slug: &str, with_transcript: bool) {
        let recording_dir = repo_root
            .join(SOURCE_PORT_RECORDINGS_CACHE_PATH)
            .join(slug)
            .join(slug);
        fs::create_dir_all(&recording_dir).expect("recording dir should exist");
        fs::write(
            recording_dir.join("manifest.json"),
            format!(r#"{{"slug":"{slug}"}}"#),
        )
        .expect("recording manifest should write");
        if with_transcript {
            fs::write(recording_dir.join("transcript.txt"), "trace transcript\n")
                .expect("transcript should write");
        }
    }

    fn write_source_port_frontier(repo_root: &Path, include_task_package: bool, status: &str) {
        let task_package = if include_task_package {
            r#",
    "task_package": {
      "title": "Fix the handoff",
      "summary": "Advance the source-port frontier.",
      "sources": [
        "docs/source-port/frontier-state.yaml",
        "crates/kid-core/src/boot.rs"
      ],
      "implementation_targets": [
        "crates/kid-core/src/boot.rs"
      ],
      "commands": [
        "mise run frontier-status"
      ],
      "done_criteria": [
        "The focused handoff window is green."
      ]
    }"#
        } else {
            ""
        };
        let contents = format!(
            r#"{{
  "updated_at": "2026-03-22",
  "active_frontier": {{
    "slug": "test-frontier",
    "recording": "any-percent-published",
    "scenario": "tests/data/tas/any-percent-published/scenario.json",
    "area": "title handoff",
    "status": "{status}",
    "frontier_frame": 6906,
    "authoritative_clean_through": 6905,
    "current_blocker": "Native lags title->demo by one frame.",
    "current_hypothesis": {{
      "name": "handoff late",
      "reason": "Oracle resumes gameplay one frame earlier."
    }},
    "rejected_hypotheses": [
      {{
        "name": "hide window late",
        "reason": "Already fixed at frame 6358."
      }}
    ],
    "required_next_evidence": [
      "Probe 6904..6908"
    ],
    "verification_commands": [
      "cargo test -p kid-core published_tas_demo_2 -- --nocapture"
    ]{task_package}
  }}
}}
"#
        );
        fs::write(repo_root.join(SOURCE_PORT_FRONTIER_STATE_PATH), contents)
            .expect("frontier state should write");
    }

    #[test]
    fn source_port_frontier_synthesises_missing_task_package() {
        let temp_dir = source_port_repo();
        write_source_port_frontier(temp_dir.path(), false, "active");

        let state =
            load_source_port_frontier_state(temp_dir.path()).expect("frontier state should load");
        let task_package = state.active_frontier.task_package_spec();

        assert_eq!(
            task_package.title,
            "Advance source-port frontier: title handoff"
        );
        assert!(task_package.summary.contains("frame 6906"));
    }

    #[test]
    fn source_port_frontier_cycle_marks_missing_note() {
        let temp_dir = source_port_repo();
        write_source_port_frontier(temp_dir.path(), true, "active");

        let cycle =
            evaluate_source_port_frontier_cycle(temp_dir.path(), 1).expect("cycle should build");
        let task = cycle.selected_task.expect("task should be selected");
        let note = task.next_agent_note.expect("note directive should exist");

        assert_eq!(task.capability, CapabilityId::SourcePortFrontierTask);
        assert_eq!(note.status, NextAgentNoteStatus::Missing);
        assert!(task.frontier_context.is_some());
    }

    #[test]
    fn source_port_frontier_cycle_marks_current_note_when_slug_matches() {
        let temp_dir = source_port_repo();
        write_source_port_frontier(temp_dir.path(), true, "active");
        let note_path = temp_dir.path().join(SOURCE_PORT_FRONTIER_NOTE_PATH);
        fs::create_dir_all(note_path.parent().expect("note dir")).expect("note dir should exist");
        fs::write(
            &note_path,
            "---\nslug: test-frontier\ncycle: 1\nupdated_at: 2026-03-22T18:00:00Z\n---\n# What Changed\n",
        )
        .expect("note should write");

        let note = source_port_next_agent_note(temp_dir.path(), "test-frontier");
        assert_eq!(note.status, NextAgentNoteStatus::Current);
        assert_eq!(note.frontmatter_slug.as_deref(), Some("test-frontier"));
    }

    #[test]
    fn source_port_frontier_cycle_keeps_broader_source_port_work_open_when_tas_frontier_closes() {
        let temp_dir = source_port_repo();
        fs::create_dir_all(temp_dir.path().join("docs/source-port"))
            .expect("docs dir should exist");
        fs::create_dir_all(temp_dir.path().join(".cache/release-gate/1p-accurate"))
            .expect("release gate dir should exist");
        write_source_port_frontier(temp_dir.path(), false, "complete");
        fs::write(
            temp_dir.path().join(SOURCE_PORT_RESEARCH_MAP_PATH),
            "# Research Map\n\n## Remaining work\n- Oracle sweep hardening\n",
        )
        .expect("research map should write");
        fs::write(
            temp_dir.path().join(SOURCE_PORT_ROADMAP_PATH),
            "# Source-Port Roadmap\n\nThe game is not complete.\n",
        )
        .expect("roadmap should write");
        fs::write(
            temp_dir.path().join(SOURCE_PORT_WHOLE_GAME_INDEX_PATH),
            "# Whole-Game Index\n\n## Explicitly open areas\n- Wider 1P gate\n",
        )
        .expect("whole-game index should write");
        fs::write(
            temp_dir.path().join(SOURCE_PORT_RELEASE_GATE_PATH),
            "{\"passed\": true}\n",
        )
        .expect("release gate should write");

        let cycle =
            evaluate_source_port_frontier_cycle(temp_dir.path(), 1).expect("cycle should build");

        assert!(!cycle.goal_complete);
        let selected_task = cycle.selected_task.expect("task should still be selected");
        let task_package = cycle
            .task_package
            .expect("task package should still be emitted");
        assert_eq!(cycle.capabilities.len(), 1);
        assert_eq!(
            cycle.capabilities[0].title,
            "Advance source-port via sweep parity hardening"
        );
        assert_eq!(cycle.capabilities[0].status, CapabilityStatus::Ready);
        assert_eq!(
            selected_task.title,
            "Advance source-port via sweep parity hardening"
        );
        assert_eq!(
            task_package.delegated_goal,
            Some(super::AutonomyGoal::SweepParity)
        );
        assert_eq!(
            task_package.delegated_target.as_deref(),
            Some("state-sweep")
        );
        assert!(task_package.summary.contains("sweep parity"));
    }

    #[test]
    fn case_queue_prefers_recording_backed_authoritative_case() {
        let temp_dir = source_port_repo();
        write_case(
            temp_dir.path(),
            "recording-case",
            r#"{"kind":"recording","path":".cache/recordings/tree/tree","slug":"tree-break"}"#,
            "player.on_ground",
            48,
            &["collision"],
            0,
            60,
        );
        write_case(
            temp_dir.path(),
            "scenario-case",
            r#"{"kind":"gameplay_scenario","slug":"pause_resume_level1"}"#,
            "player.on_ground",
            90,
            &["collision", "player physics"],
            0,
            140,
        );

        let queue = build_case_queue_snapshot(temp_dir.path()).expect("queue should build");
        let selected = queue.selected_case.expect("selected case should exist");

        assert_eq!(selected.case_id, "recording-case");
        assert_eq!(selected.status, CaseQueueStatus::AutonomyExecutable);
        assert_eq!(
            selected.failure_class,
            Some(CaseFailureClass::Authoritative)
        );
    }

    #[test]
    fn case_queue_blocks_advisory_only_cases() {
        let temp_dir = source_port_repo();
        write_case(
            temp_dir.path(),
            "advisory-case",
            r#"{"kind":"gameplay_scenario","slug":"death_respawn_level1"}"#,
            "world.score",
            32,
            &["player physics"],
            0,
            80,
        );

        let queue = build_case_queue_snapshot(temp_dir.path()).expect("queue should build");
        assert!(queue.selected_case.is_none());
        assert_eq!(queue.blocked_cases.len(), 1);
        assert_eq!(queue.blocked_cases[0].status, CaseQueueStatus::Blocked);
        assert_eq!(
            queue.blocked_cases[0].failure_class,
            Some(CaseFailureClass::Advisory)
        );
    }

    #[test]
    fn case_queue_lists_untriaged_recordings() {
        let temp_dir = source_port_repo();
        write_recording(temp_dir.path(), "level1-tree-break", true);

        let queue = build_case_queue_snapshot(temp_dir.path()).expect("queue should build");
        assert_eq!(queue.untriaged_recordings.len(), 1);
        assert_eq!(queue.untriaged_recordings[0].slug, "level1-tree-break");
        assert!(queue.untriaged_recordings[0].has_transcript);
    }

    #[test]
    fn source_port_frontier_delegates_to_case_queue_when_case_exists() {
        let temp_dir = source_port_repo();
        write_source_port_frontier(temp_dir.path(), false, "complete");
        write_case(
            temp_dir.path(),
            "recording-case",
            r#"{"kind":"recording","path":".cache/recordings/tree/tree","slug":"tree-break"}"#,
            "player.on_ground",
            48,
            &["collision"],
            0,
            60,
        );

        let cycle =
            evaluate_source_port_frontier_cycle(temp_dir.path(), 1).expect("cycle should build");
        let task = cycle.selected_task.expect("selected task should exist");

        assert_eq!(task.delegated_goal, Some(super::AutonomyGoal::CaseQueue));
        assert_eq!(task.delegated_target.as_deref(), Some("recording-case"));
        assert!(task.title.contains("recording-case"));
        assert!(cycle.progress_fingerprint.contains("case-queue"));
    }

    #[test]
    fn case_queue_pivots_after_repeating_the_same_case() {
        let temp_dir = source_port_repo();
        write_case(
            temp_dir.path(),
            "recording-case",
            r#"{"kind":"recording","path":".cache/recordings/tree/tree","slug":"tree-break"}"#,
            "player.on_ground",
            48,
            &["collision"],
            0,
            60,
        );
        write_case(
            temp_dir.path(),
            "scenario-case",
            r#"{"kind":"gameplay_scenario","slug":"pause_resume_level1"}"#,
            "player.on_ground",
            52,
            &["collision"],
            0,
            70,
        );
        let ledger_dir = temp_dir.path().join(".cache/autonomy/case-queue");
        fs::create_dir_all(&ledger_dir).expect("ledger dir should exist");
        fs::write(
            ledger_dir.join("ledger.json"),
            r#"{
  "goal": "case-queue",
  "title": "Source-port case queue",
  "cycles_run": 7,
  "complete": false,
  "stalled": true,
  "last_task": "source_port_case_queue_task",
  "last_progress_fingerprint": "recording-case:autonomy_executable:48:player.on_ground:patch-allowed:60:authoritative",
  "last_cycle_path": null,
  "capabilities": [],
  "blocker": null
}
"#,
        )
        .expect("ledger should write");

        let queue = build_case_queue_snapshot(temp_dir.path()).expect("queue should build");
        let selected = queue.selected_case.expect("selected case should exist");

        assert_eq!(selected.case_id, "scenario-case");
        assert!(
            queue
                .selection_reason
                .as_deref()
                .is_some_and(|reason| reason.contains("pivoted"))
        );
    }

    #[test]
    fn case_queue_progress_ignores_note_state_changes() {
        let temp_dir = source_port_repo();
        write_case(
            temp_dir.path(),
            "recording-case",
            r#"{"kind":"recording","path":".cache/recordings/tree/tree","slug":"tree-break"}"#,
            "player.on_ground",
            48,
            &["collision"],
            0,
            60,
        );
        let stale_note_path = temp_dir.path().join(SOURCE_PORT_CASE_QUEUE_NOTE_PATH);
        fs::create_dir_all(stale_note_path.parent().expect("note dir"))
            .expect("note dir should exist");
        fs::write(
            &stale_note_path,
            "---\nslug: wrong-case\ncycle: 1\nupdated_at: 2026-03-31T20:00:00Z\n---\n# What Changed\n",
        )
        .expect("stale note should write");
        let stale =
            evaluate_case_queue_cycle(temp_dir.path(), 1).expect("stale cycle should build");

        fs::write(
            &stale_note_path,
            "---\nslug: recording-case\ncycle: 2\nupdated_at: 2026-03-31T20:05:00Z\n---\n# What Changed\n",
        )
        .expect("current note should write");
        let current =
            evaluate_case_queue_cycle(temp_dir.path(), 2).expect("current cycle should build");

        assert_eq!(stale.progress_fingerprint, current.progress_fingerprint);
    }

    #[test]
    fn case_queue_cycle_marks_current_note_when_case_slug_matches() {
        let temp_dir = source_port_repo();
        write_case(
            temp_dir.path(),
            "recording-case",
            r#"{"kind":"recording","path":".cache/recordings/tree/tree","slug":"tree-break"}"#,
            "player.on_ground",
            48,
            &["collision"],
            0,
            60,
        );
        let note_path = temp_dir.path().join(SOURCE_PORT_CASE_QUEUE_NOTE_PATH);
        fs::create_dir_all(note_path.parent().expect("note dir")).expect("note dir should exist");
        fs::write(
            &note_path,
            "---\nslug: recording-case\ncycle: 1\nupdated_at: 2026-03-31T20:00:00Z\n---\n# What Changed\n",
        )
        .expect("note should write");

        let cycle = evaluate_case_queue_cycle(temp_dir.path(), 1).expect("cycle should build");
        let task = cycle.selected_task.expect("selected task should exist");
        let note = task.next_agent_note.expect("note should exist");

        assert_eq!(task.capability, CapabilityId::SourcePortCaseQueueTask);
        assert!(task.case_queue_context.is_some());
        assert_eq!(note.status, NextAgentNoteStatus::Current);
        assert_eq!(note.frontmatter_slug.as_deref(), Some("recording-case"));
    }
}
