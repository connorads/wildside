use super::{
    GameplayScenario, Scenario, compare_native_gameplay, compare_native_oracle,
    prepare_generated_assets, verify_native_bootstrap,
};
use anyhow::{Context, Result, bail};
use clap::ValueEnum;
use kid_data::AssetManifest;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
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
const SOURCE_PORT_BACKLOG_PATH: &str = "docs/source-port/backlog.md";
const SOURCE_PORT_RELEASE_GATE_PATH: &str = ".cache/release-gate/1p-accurate/latest.json";
const SOURCE_PORT_FRONTIER_RUN_HISTORY_PATH: &str =
    ".cache/autonomy/source-port-frontier/run-history.jsonl";
const SOURCE_PORT_RECOVERY_TRACE_PATH: &str =
    ".cache/autonomy/source-port-frontier/recovery-trace.jsonl";
const SOURCE_PORT_RECOVERY_TRACE_ENV: &str = "KC_AUTONOMY_RECOVERY_TRACE";
const SOURCE_PORT_BACKLOG_EVIDENCE_BLOCKER_TARGET: &str =
    "backlog-evidence-blocker:boot-title-start-level1:level_entry@520";
/// Maximum number of gameplay-bug investigation attempts per backlog issue
/// before the selector skips it. The per-issue counter lives in
/// `frontier-state.yaml` under `bug_investigation_attempts`.
const BUG_INVESTIGATION_ATTEMPT_CAP: u32 = 3;
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
    /// ASM-driven gameplay backlog: pick uncovered routines from asm-coverage.yaml.
    #[value(name = "gameplay-backlog")]
    #[serde(rename = "gameplay-backlog")]
    GameplayBacklog,
    /// Proactive verification: loc_ coverage, enemy audits, mutation testing.
    #[value(name = "proactive-verification")]
    #[serde(rename = "proactive-verification")]
    ProactiveVerification,
    /// Gameplay-bug investigation: reproduce, trace ASM, propose fix, mark `[?]`.
    #[value(name = "gameplay-bug-investigation")]
    #[serde(rename = "gameplay-bug-investigation")]
    GameplayBugInvestigation,
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
            Self::GameplayBacklog => "gameplay-backlog",
            Self::ProactiveVerification => "proactive-verification",
            Self::GameplayBugInvestigation => "gameplay-bug-investigation",
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
            Self::GameplayBacklog => repo_root.join("docs/source-port/asm-coverage.yaml"),
            Self::ProactiveVerification => {
                repo_root.join("docs/source-port/proactive-verification-config.yaml")
            }
            Self::GameplayBugInvestigation => repo_root.join(SOURCE_PORT_BACKLOG_PATH),
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
    GameplayBacklogTask,
    ProactiveVerificationTask,
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
            Self::GameplayBacklogTask => "gameplay_backlog_task",
            Self::ProactiveVerificationTask => "proactive_verification_task",
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
    generated_at: String,
    cycle: u32,
    progress_fingerprint: String,
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
    recovery_mode: Option<RecoveryMode>,
    #[serde(skip_serializing_if = "Option::is_none")]
    recovery_step: Option<RecoveryStep>,
    #[serde(skip_serializing_if = "Option::is_none")]
    stalled_repetition_count: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pivot_reason: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    next_fallback_goal: Option<AutonomyGoal>,
    #[serde(skip_serializing_if = "Option::is_none")]
    next_agent_note: Option<NextAgentNoteDirective>,
    #[serde(skip_serializing_if = "Option::is_none")]
    target_routines: Option<Vec<TargetRoutine>>,
    /// True iff the planner has nothing genuinely new to dispatch and the
    /// recovery cascade can only restate work that already landed cleanly.
    /// Runner uses this to short-circuit straight to the operator message
    /// on iteration 1 instead of waiting for `DEFAULT_MAX_STALL_STREAK`
    /// consecutive silent retries.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    definitively_stalled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct TargetRoutine {
    label: String,
    subsystem: String,
    asm_file: String,
    line_number: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
enum RecoveryMode {
    DirtyTree,
    StalledFingerprint,
}

impl RecoveryMode {
    fn as_str(self) -> &'static str {
        match self {
            Self::DirtyTree => "dirty_tree",
            Self::StalledFingerprint => "stalled_fingerprint",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
enum RecoveryStep {
    FinishDirtyWork,
    ReverifyCurrentLane,
    PivotToNextLane,
    RefreshEvidence,
    OperatorNoteOnly,
}

impl RecoveryStep {
    fn as_str(self) -> &'static str {
        match self {
            Self::FinishDirtyWork => "finish_dirty_work",
            Self::ReverifyCurrentLane => "reverify_current_lane",
            Self::PivotToNextLane => "pivot_to_next_lane",
            Self::RefreshEvidence => "refresh_evidence",
            Self::OperatorNoteOnly => "operator_note_only",
        }
    }
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
    selected_recording: Option<RecordingQueueCandidate>,
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
    frame_count: usize,
    has_transcript: bool,
    has_notes: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum BacklogIssueFocus {
    Playability,
    VisualParity,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum BacklogIssuePhase {
    CaptureEvidence,
    InvestigateReproduced,
}

impl BacklogIssuePhase {
    fn slug(self) -> &'static str {
        match self {
            BacklogIssuePhase::CaptureEvidence => "capture",
            BacklogIssuePhase::InvestigateReproduced => "investigate",
        }
    }
}

#[derive(Debug, Clone)]
struct BacklogIssueCandidate {
    id: String,
    priority: String,
    system: String,
    summary: String,
    verify: String,
    status: String,
    focus: BacklogIssueFocus,
}

#[derive(Debug, Clone)]
struct ReleaseGateFailureCandidate {
    target: String,
    step_name: String,
    detail: String,
}

const HIGH_VALUE_BACKLOG_ISSUE_ORDER: &[&str] =
    &["138", "143", "139", "140", "136", "135", "141", "137"];

#[derive(Debug, Clone)]
struct CaseQueueSnapshot {
    selected_case: Option<CaseQueueCandidate>,
    selected_recording: Option<RecordingQueueCandidate>,
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
    #[serde(default)]
    blocked_reason: Option<String>,
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
    /// Optional `progress_fingerprint` token (e.g.
    /// `proactive-verification:mutation-testing:enemies.rs`). When
    /// present, the cascade skips any candidate task whose fingerprint
    /// matches so the loop stops rediscovering dead ends.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    fingerprint: Option<String>,
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
    /// Debug-field registry entries that have been flagged ready for
    /// promotion from the debug bag to typed state. The proactive
    /// cascade emits a `promote-debug-field` task for any entry whose
    /// `maturity_status` is `ready` so mature candidates preempt the
    /// loc-coverage-drive busywork.
    #[serde(default)]
    candidate_promotions: Vec<CandidatePromotion>,
    /// Per-issue attempt counter for the gameplay-bug investigation lane.
    /// Incremented each time the lane stalls without producing evidence;
    /// once it exceeds `BUG_INVESTIGATION_ATTEMPT_CAP`, the selector skips
    /// the issue until a human re-inspects.
    #[serde(default)]
    bug_investigation_attempts: BTreeMap<String, u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct CandidatePromotion {
    key: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    promotion_target: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    reason: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    maturity_status: Option<String>,
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
                    "cargo run -p kidcli -- autonomous-loop --goal gameplay-backlog".to_owned(),
                    "cargo run -p kidcli -- asm-coverage --uncovered".to_owned(),
                    "mise run verify-slice-fast".to_owned(),
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
    routines_per_task: u32,
) -> Result<()> {
    let goal_state = match goal {
        AutonomyGoal::SourcePortFrontier => GoalState::SourcePortFrontier,
        AutonomyGoal::CaseQueue => GoalState::CaseQueue,
        AutonomyGoal::GameplayBacklog => GoalState::GameplayBacklog,
        AutonomyGoal::ProactiveVerification => GoalState::ProactiveVerification,
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
        let mut cycle = goal_state.evaluate_cycle(repo_root, cycle_number, routines_per_task)?;
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
        if matches!(goal, AutonomyGoal::SweepParity) {
            refresh_source_port_frontier_handoff_from_latest_evidence(
                repo_root,
                routines_per_task,
            )?;
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
        ledger.blocker.clone_from(&cycle.blocker);
        write_json(&ledger_path, &ledger)?;

        if emit_json {
            println!("{}", serde_json::to_string_pretty(&cycle)?);
        } else {
            print_cycle_summary(&cycle);
        }

        if cycle.goal_complete
            || cycle.blocker.is_some()
            || cycle.stalled
            || cycle
                .selected_task
                .as_ref()
                .and_then(|task| task.recovery_mode)
                .is_some()
        {
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
    GameplayBacklog,
    ProactiveVerification,
}

impl GoalState {
    fn title(&self) -> &str {
        match self {
            Self::SpecDriven { spec, .. } => spec.title.as_str(),
            Self::SourcePortFrontier => SOURCE_PORT_FRONTIER_TITLE,
            Self::CaseQueue => SOURCE_PORT_CASE_QUEUE_TITLE,
            Self::GameplayBacklog => "ASM-driven gameplay backlog",
            Self::ProactiveVerification => "Proactive verification",
        }
    }

    fn evaluate_cycle(
        &self,
        repo_root: &Path,
        cycle_number: u32,
        routines_per_task: u32,
    ) -> Result<CycleReport> {
        match self {
            Self::SpecDriven {
                spec,
                rom_path,
                manifest,
            } => evaluate_cycle(repo_root, rom_path, manifest, spec, cycle_number),
            Self::SourcePortFrontier => {
                evaluate_source_port_frontier_cycle(repo_root, cycle_number, routines_per_task)
            }
            Self::CaseQueue => evaluate_case_queue_cycle(repo_root, cycle_number),
            Self::GameplayBacklog => {
                evaluate_gameplay_backlog_cycle(repo_root, cycle_number, routines_per_task)
            }
            Self::ProactiveVerification => {
                evaluate_proactive_verification_cycle(repo_root, cycle_number, routines_per_task)
            }
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

pub(crate) fn refresh_source_port_frontier_handoff_from_latest_evidence(
    repo_root: &Path,
    routines_per_task: u32,
) -> Result<()> {
    refresh_source_port_frontier_handoff(repo_root, routines_per_task).map(|_| ())
}

pub(crate) fn refresh_source_port_frontier_handoff_after_release_gate(
    repo_root: &Path,
    routines_per_task: u32,
) -> Result<()> {
    if let Some(task) = refresh_source_port_frontier_handoff(repo_root, routines_per_task)? {
        record_refresh_evidence_completion(repo_root, &task)?;
        if task.recovery_step == Some(RecoveryStep::RefreshEvidence) {
            refresh_source_port_frontier_handoff(repo_root, routines_per_task)?;
        }
    }
    Ok(())
}

fn refresh_source_port_frontier_handoff(
    repo_root: &Path,
    routines_per_task: u32,
) -> Result<Option<TaskPackage>> {
    let frontier_path = repo_root.join(SOURCE_PORT_FRONTIER_STATE_PATH);
    if !frontier_path.is_file() {
        return Ok(None);
    }
    let next_cycle = load_goal_ledger_if_present(repo_root, AutonomyGoal::SourcePortFrontier)?
        .map_or(1, |ledger| ledger.cycles_run.saturating_add(1));
    let cycle = evaluate_source_port_frontier_cycle(repo_root, next_cycle, routines_per_task)?;
    let Some(task) = cycle.task_package else {
        return Ok(None);
    };
    let next_task_path = repo_root
        .join(".cache/autonomy")
        .join(AutonomyGoal::SourcePortFrontier.slug())
        .join("next-task.json");
    if let Some(parent) = next_task_path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("failed to create {}", parent.display()))?;
    }
    write_json(&next_task_path, &task)?;
    if let (Some(note), Some(frontier)) = (
        task.next_agent_note.as_ref(),
        task.frontier_context.as_ref(),
    ) {
        write_source_port_next_agent_note(repo_root, note, frontier, &task)?;
    }
    Ok(Some(task))
}

fn record_refresh_evidence_completion(repo_root: &Path, task: &TaskPackage) -> Result<()> {
    if task.recovery_step != Some(RecoveryStep::RefreshEvidence) {
        return Ok(());
    }
    let run_history_path = repo_root.join(SOURCE_PORT_FRONTIER_RUN_HISTORY_PATH);
    if let Some(parent) = run_history_path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("failed to create {}", parent.display()))?;
    }
    let mut record = serde_json::Map::new();
    record.insert(
        "cycle".to_owned(),
        serde_json::Value::from(task.cycle as u64),
    );
    if let Some(goal) = task.delegated_goal {
        record.insert(
            "delegated_goal".to_owned(),
            serde_json::Value::from(goal.slug()),
        );
    } else {
        record.insert("delegated_goal".to_owned(), serde_json::Value::Null);
    }
    if let Some(target) = task.delegated_target.as_ref() {
        record.insert(
            "delegated_target".to_owned(),
            serde_json::Value::from(target.as_str()),
        );
    } else {
        record.insert("delegated_target".to_owned(), serde_json::Value::Null);
    }
    record.insert(
        "dirty_files".to_owned(),
        serde_json::Value::Array(Vec::new()),
    );
    record.insert("dirty_tree".to_owned(), serde_json::Value::Bool(false));
    record.insert("dirty_tree_fingerprint".to_owned(), serde_json::Value::Null);
    record.insert("exit_code".to_owned(), serde_json::Value::from(0));
    record.insert("outcome".to_owned(), serde_json::Value::from("completed"));
    record.insert("preflight_command".to_owned(), serde_json::Value::Null);
    record.insert("preflight_exit_code".to_owned(), serde_json::Value::Null);
    record.insert(
        "preflight_summary".to_owned(),
        serde_json::Value::from("release-gate refreshed source-port evidence"),
    );
    record.insert(
        "progress_fingerprint".to_owned(),
        serde_json::Value::from(task.progress_fingerprint.as_str()),
    );
    if let Some(mode) = task.recovery_mode {
        record.insert(
            "recovery_mode".to_owned(),
            serde_json::Value::from(mode.as_str()),
        );
    } else {
        record.insert("recovery_mode".to_owned(), serde_json::Value::Null);
    }
    record.insert(
        "recovery_step".to_owned(),
        serde_json::Value::from(RecoveryStep::RefreshEvidence.as_str()),
    );
    if let Some(count) = task.stalled_repetition_count {
        record.insert(
            "stalled_repetition_count".to_owned(),
            serde_json::Value::from(count as u64),
        );
    } else {
        record.insert(
            "stalled_repetition_count".to_owned(),
            serde_json::Value::Null,
        );
    }
    record.insert(
        "timestamp".to_owned(),
        serde_json::Value::from(autonomy_timestamp()),
    );
    let mut file = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&run_history_path)
        .with_context(|| format!("failed to open {}", run_history_path.display()))?;
    use std::io::Write;
    writeln!(file, "{}", serde_json::Value::Object(record))
        .with_context(|| format!("failed to append {}", run_history_path.display()))?;
    Ok(())
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
            .map_or_else(|| "none".to_owned(), |frame| frame.to_string()),
        candidate.first_bad_field.as_deref().unwrap_or("none"),
        case_queue_gate_status(candidate),
        candidate.frame_end.saturating_sub(candidate.frame_start),
        candidate
            .failure_class
            .map_or("unknown", CaseFailureClass::as_str)
    )
}

fn recording_queue_candidate_progress_marker(recording: &RecordingQueueCandidate) -> String {
    format!(
        "recording:{}:derive:{}f",
        recording.slug, recording.frame_count
    )
}

fn case_queue_progress_fingerprint(
    selected_case: Option<&CaseQueueCandidate>,
    selected_recording: Option<&RecordingQueueCandidate>,
) -> String {
    selected_case
        .map(case_queue_candidate_progress_marker)
        .or_else(|| selected_recording.map(recording_queue_candidate_progress_marker))
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

fn select_recording_queue_candidate(
    repo_root: &Path,
    recordings: &[RecordingQueueCandidate],
) -> Result<(Option<RecordingQueueCandidate>, Option<String>)> {
    let Some(first) = recordings.first().cloned() else {
        return Ok((None, None));
    };
    let previous = load_goal_ledger_if_present(repo_root, AutonomyGoal::CaseQueue)?
        .and_then(|ledger| ledger.last_progress_fingerprint);
    if previous.as_deref() == Some(recording_queue_candidate_progress_marker(&first).as_str()) {
        if let Some(next) = recordings.get(1).cloned() {
            return Ok((
                Some(next.clone()),
                Some(format!(
                    "previous cycle stalled while deriving from recording {}; pivoted to the next untriaged recording {}",
                    first.slug, next.slug
                )),
            ));
        }
        return Ok((
            Some(first.clone()),
            Some(format!(
                "recording {} remains the highest-priority gameplay source and no alternate untriaged recording is available",
                first.slug
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
        delegated_goal.map_or("tas-frontier", AutonomyGoal::slug),
        delegated_target.unwrap_or("none"),
        lane_fingerprint.unwrap_or("none")
    )
}

fn source_port_frontier_sweep_lane_marker(repo_root: &Path) -> String {
    let release_gate_timestamp = latest_release_gate_timestamp(repo_root, "1p-accurate")
        .unwrap_or_else(|| "missing".to_owned());
    let state_sweep_timestamp = latest_state_sweep_evidence(repo_root)
        .and_then(|evidence| evidence.timestamp)
        .unwrap_or_else(|| "missing".to_owned());
    format!("state-sweep:gate-{release_gate_timestamp}:sweep-{state_sweep_timestamp}")
}

fn latest_release_gate_timestamp(repo_root: &Path, target: &str) -> Option<String> {
    let latest_path = repo_root
        .join(".cache/release-gate")
        .join(target)
        .join("latest.json");
    let contents = fs::read_to_string(latest_path).ok()?;
    let report = serde_json::from_str::<serde_json::Value>(&contents).ok()?;
    report
        .get("timestamp")
        .and_then(serde_json::Value::as_str)
        .map(str::to_owned)
}

fn tracked_dirty_files(repo_root: &Path) -> Vec<String> {
    let output = Command::new("git")
        .args(["status", "--short", "--untracked-files=no"])
        .current_dir(repo_root)
        .output();
    let Ok(output) = output else {
        return Vec::new();
    };
    if !output.status.success() {
        return Vec::new();
    }
    String::from_utf8_lossy(&output.stdout)
        .lines()
        .filter_map(|line| line.get(3..).map(str::trim))
        .filter(|path| !path.is_empty())
        .map(ToOwned::to_owned)
        .collect()
}

fn source_port_recent_matching_stall_count(repo_root: &Path, fingerprint: &str) -> u32 {
    let path = repo_root.join(SOURCE_PORT_FRONTIER_RUN_HISTORY_PATH);
    let Ok(contents) = fs::read_to_string(path) else {
        return 0;
    };

    let mut matches = 0;
    for line in contents.lines().rev() {
        let Ok(record) = serde_json::from_str::<serde_json::Value>(line) else {
            continue;
        };
        let same_fingerprint = record
            .get("progress_fingerprint")
            .and_then(serde_json::Value::as_str)
            .is_some_and(|value| value == fingerprint);
        let stalled_repeat = record
            .get("outcome")
            .and_then(serde_json::Value::as_str)
            .is_some_and(|outcome| outcome == "stalled_repeat");
        if same_fingerprint && stalled_repeat {
            matches += 1;
            continue;
        }
        break;
    }
    matches
}

fn source_port_recent_clean_completed_targets(repo_root: &Path) -> BTreeSet<String> {
    let path = repo_root.join(SOURCE_PORT_FRONTIER_RUN_HISTORY_PATH);
    let Ok(contents) = fs::read_to_string(path) else {
        return BTreeSet::new();
    };
    let mut targets = BTreeSet::new();
    for line in contents.lines().rev() {
        let Ok(record) = serde_json::from_str::<serde_json::Value>(line) else {
            continue;
        };
        let clean_completion = record
            .get("outcome")
            .and_then(serde_json::Value::as_str)
            .is_some_and(|outcome| outcome == "completed")
            && record
                .get("exit_code")
                .and_then(serde_json::Value::as_i64)
                .is_some_and(|code| code == 0)
            && !record
                .get("dirty_tree")
                .and_then(serde_json::Value::as_bool)
                .unwrap_or(false);
        if !clean_completion {
            continue;
        }
        if let Some(target) = record
            .get("delegated_target")
            .and_then(serde_json::Value::as_str)
            .filter(|target| !target.is_empty())
        {
            targets.insert(target.to_owned());
            if targets.len() >= 12 {
                break;
            }
        }
    }
    targets
}

/// True iff a recent clean completion (within the last ~12 rows) exposed
/// exactly this `(delegated_target, progress_fingerprint)` pair. Distinct
/// from `source_port_recent_matching_stall_count` which needs trailing
/// `stalled_repeat` rows — this fires on iteration 1, before any stalls
/// accumulate, when the planner would re-emit just-completed work.
fn source_port_repeats_recent_clean_completion(
    repo_root: &Path,
    target: &str,
    fingerprint: &str,
) -> bool {
    let path = repo_root.join(SOURCE_PORT_FRONTIER_RUN_HISTORY_PATH);
    let Ok(contents) = fs::read_to_string(path) else {
        return false;
    };
    let mut examined = 0usize;
    for line in contents.lines().rev() {
        if examined >= 12 {
            break;
        }
        let Ok(record) = serde_json::from_str::<serde_json::Value>(line) else {
            continue;
        };
        examined += 1;
        let clean_completion = record
            .get("outcome")
            .and_then(serde_json::Value::as_str)
            .is_some_and(|outcome| outcome == "completed")
            && record
                .get("exit_code")
                .and_then(serde_json::Value::as_i64)
                .is_some_and(|code| code == 0)
            && !record
                .get("dirty_tree")
                .and_then(serde_json::Value::as_bool)
                .unwrap_or(false);
        if !clean_completion {
            continue;
        }
        let target_matches = record
            .get("delegated_target")
            .and_then(serde_json::Value::as_str)
            .is_some_and(|value| value == target);
        let fingerprint_matches = record
            .get("progress_fingerprint")
            .and_then(serde_json::Value::as_str)
            .is_some_and(|value| value == fingerprint);
        if target_matches && fingerprint_matches {
            return true;
        }
    }
    false
}

fn prepend_commands(existing: &[String], prefix: &[String]) -> Vec<String> {
    prefix
        .iter()
        .cloned()
        .chain(existing.iter().cloned())
        .collect()
}

fn apply_dirty_tree_recovery(
    task: &mut TaskPackage,
    dirty_files: &[String],
    delegated_goal: Option<AutonomyGoal>,
) {
    let detail = format!(
        "tracked git changes exist and must be resolved before the loop can trust a fresh delegated lane: {}",
        dirty_files.join(", ")
    );
    task.summary = format!("Resolve the current tracked work first. {}", task.summary);
    task.done_criteria.insert(
        0,
        "Resolve the tracked git diff as a coherent unit: verify it, commit it if it passes, or deliberately discard only that unit.".to_owned(),
    );
    task.evidence.push(EvidenceItem {
        label: "dirty-tree".to_owned(),
        status: "tracked-changes".to_owned(),
        detail,
        artefact_path: None,
    });
    task.commands = prepend_commands(
        &task.commands,
        &[
            "git status --short".to_owned(),
            "git diff --stat".to_owned(),
        ],
    );
    let additional_targets = dirty_files
        .iter()
        .filter(|path| {
            !task
                .implementation_targets
                .iter()
                .any(|existing| existing == *path)
        })
        .cloned()
        .collect::<Vec<_>>();
    task.implementation_targets.extend(additional_targets);
    task.recovery_mode = Some(RecoveryMode::DirtyTree);
    task.recovery_step = Some(RecoveryStep::FinishDirtyWork);
    task.pivot_reason =
        Some("dirty tracked work takes priority over starting a fresh delegated lane".to_owned());
    task.next_fallback_goal = delegated_goal;
}

#[allow(clippy::too_many_arguments)]
fn emit_recovery_trace(
    repo_root: &Path,
    cycle: u32,
    delegated_goal: Option<AutonomyGoal>,
    delegated_target: Option<&str>,
    base_progress_fingerprint: &str,
    dirty_files_len: usize,
    repeated_stalls: u32,
    chose_branch: &str,
    task: &TaskPackage,
    final_progress_fingerprint: &str,
) {
    if std::env::var(SOURCE_PORT_RECOVERY_TRACE_ENV).is_err() {
        return;
    }
    let path = repo_root.join(SOURCE_PORT_RECOVERY_TRACE_PATH);
    if let Some(parent) = path.parent()
        && fs::create_dir_all(parent).is_err()
    {
        return;
    }
    let record = serde_json::json!({
        "timestamp": autonomy_timestamp(),
        "cycle": cycle,
        "delegated_goal": delegated_goal.map(AutonomyGoal::slug),
        "delegated_target": delegated_target,
        "base_progress_fingerprint": base_progress_fingerprint,
        "dirty_files_len": dirty_files_len,
        "repeated_stalls": repeated_stalls,
        "chose_branch": chose_branch,
        "recovery_mode_out": task.recovery_mode.map(RecoveryMode::as_str),
        "recovery_step_out": task.recovery_step.map(RecoveryStep::as_str),
        "final_progress_fingerprint": final_progress_fingerprint,
    });
    let Ok(line) = serde_json::to_string(&record) else {
        return;
    };
    let _ = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&path)
        .and_then(|mut handle| {
            use std::io::Write;
            writeln!(handle, "{line}")
        });
}

fn apply_stalled_recovery(
    task: &mut TaskPackage,
    fingerprint: &str,
    repeated_stalls: u32,
    recovery_step: RecoveryStep,
    pivot_reason: String,
    next_fallback_goal: Option<AutonomyGoal>,
) {
    task.summary = format!(
        "Recovery mode after {} repeated stalled cycles on the same delegated fingerprint. {}",
        repeated_stalls, task.summary
    );
    task.done_criteria.insert(
        0,
        format!(
            "Break the repeated stall by completing the `{}` recovery step before returning to routine lane work.",
            recovery_step.as_str()
        ),
    );
    task.evidence.push(EvidenceItem {
        label: "stalled-fingerprint".to_owned(),
        status: "repeated".to_owned(),
        detail: format!("{repeated_stalls} repeated stalled cycles for fingerprint {fingerprint}"),
        artefact_path: Some(PathBuf::from(".cache/autonomy/source-port-frontier")),
    });
    task.recovery_mode = Some(RecoveryMode::StalledFingerprint);
    task.recovery_step = Some(recovery_step);
    task.stalled_repetition_count = Some(repeated_stalls);
    task.pivot_reason = Some(pivot_reason);
    task.next_fallback_goal = next_fallback_goal;
}

fn evaluate_source_port_frontier_cycle(
    repo_root: &Path,
    cycle: u32,
    routines_per_task: u32,
) -> Result<CycleReport> {
    let state = load_source_port_frontier_state(repo_root)?;
    let frontier = &state.active_frontier;
    let note = source_port_next_agent_note(repo_root, frontier.slug.as_str());
    let task_ready = matches!(frontier.status.as_str(), "active" | "complete");
    let recent_clean_completed_targets = source_port_recent_clean_completed_targets(repo_root);
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
                Some(case_queue_progress_fingerprint(Some(candidate), None)),
                summary,
            )
        } else if let Some(recording) = queue.selected_recording.as_ref() {
            let task = source_port_frontier_recording_task(frontier, &note, &queue, recording);
            let summary = format!(
                "TAS frontier {} is closed through frame {}; delegated to recording-derived gameplay work for {}.",
                frontier.slug, frontier.authoritative_clean_through, recording.slug
            );
            (
                task,
                Some(AutonomyGoal::CaseQueue),
                Some(recording.slug.clone()),
                Some(recording_queue_candidate_progress_marker(recording)),
                summary,
            )
        } else {
            // No case-queue work. Prioritise ready backlog fixes and live
            // gate blockers before coverage-driven fallback work.
            if let Some(issue) =
                select_ready_backlog_issue_excluding(repo_root, &recent_clean_completed_targets)?
            {
                let target = ready_backlog_issue_progress_marker(&issue);
                let task = source_port_frontier_ready_backlog_issue_task(frontier, &note, &issue);
                let summary = format!(
                    "TAS frontier {} is closed through frame {}; delegated to ready backlog issue #{} ({}).",
                    frontier.slug, frontier.authoritative_clean_through, issue.id, issue.summary
                );
                (
                    task,
                    Some(AutonomyGoal::ProactiveVerification),
                    Some(target.clone()),
                    Some(format!("ready-backlog:{target}")),
                    summary,
                )
            } else if let Some(gate_failure) = select_release_gate_blocker(repo_root) {
                let task = source_port_frontier_release_gate_task(
                    repo_root,
                    frontier,
                    &note,
                    &gate_failure,
                );
                let summary = format!(
                    "TAS frontier {} is closed through frame {}; delegated to failing release-gate step {} ({}).",
                    frontier.slug,
                    frontier.authoritative_clean_through,
                    gate_failure.step_name,
                    gate_failure.target
                );
                (
                    task,
                    Some(AutonomyGoal::SweepParity),
                    Some(gate_failure.step_name.clone()),
                    Some(format!(
                        "release-gate:{}:{}",
                        gate_failure.target, gate_failure.step_name
                    )),
                    summary,
                )
            } else {
                let uncovered = collect_uncovered_sub_labels(repo_root).unwrap_or_default();
                if uncovered.is_empty() {
                    let pv_report =
                        evaluate_proactive_verification_cycle(repo_root, cycle, routines_per_task)?;
                    if let Some(pv_task) = pv_report.task_package {
                        let target = pv_task.progress_fingerprint.clone();
                        let summary = format!(
                            "TAS frontier {} closed; delegated to proactive-verification ({}).",
                            frontier.slug, target
                        );
                        (
                            source_port_frontier_proactive_task(
                                pv_task,
                                frontier,
                                &note,
                                target.clone(),
                                "ready backlog and release-gate blockers are clear; proactive verification has work"
                                    .to_owned(),
                            ),
                            Some(AutonomyGoal::ProactiveVerification),
                            Some(target.clone()),
                            Some(target),
                            summary,
                        )
                    } else if let Some(issue) = select_open_bug_for_investigation(
                        repo_root,
                        &recent_clean_completed_targets,
                        &state.bug_investigation_attempts,
                        BUG_INVESTIGATION_ATTEMPT_CAP,
                    )? {
                        let target = bug_investigation_progress_marker(&issue);
                        let task =
                            source_port_frontier_bug_investigation_task(frontier, &note, &issue);
                        let summary = format!(
                            "TAS frontier {} is closed through frame {}; delegated to gameplay-bug investigation for #{} ({}).",
                            frontier.slug,
                            frontier.authoritative_clean_through,
                            issue.id,
                            issue.summary
                        );
                        (
                            task,
                            Some(AutonomyGoal::GameplayBugInvestigation),
                            Some(target.clone()),
                            Some(format!("bug-investigation:{target}")),
                            summary,
                        )
                    } else if let Some(issue) = select_open_backlog_issue_excluding(
                        repo_root,
                        &recent_clean_completed_targets,
                    )? {
                        let target = backlog_issue_progress_marker(&issue);
                        let task = source_port_frontier_backlog_issue_task(frontier, &note, &issue);
                        let summary = format!(
                            "TAS frontier {} is closed through frame {}; proactive verification exhausted, pivoting to backlog issue #{} ({}).",
                            frontier.slug,
                            frontier.authoritative_clean_through,
                            issue.id,
                            issue.summary
                        );
                        (
                            task,
                            None,
                            Some(target.clone()),
                            Some(format!("backlog-issue:{target}")),
                            summary,
                        )
                    } else if open_backlog_targets_exhausted_by_recent_clean_completion(
                        repo_root,
                        &recent_clean_completed_targets,
                    )? && !recent_clean_completed_targets
                        .contains(SOURCE_PORT_BACKLOG_EVIDENCE_BLOCKER_TARGET)
                    {
                        let task = source_port_frontier_backlog_evidence_blocker_task(
                            frontier,
                            &note,
                            &recent_clean_completed_targets,
                        );
                        let summary = format!(
                            "TAS frontier {} is closed through frame {}; open backlog issue targets were recently clean-completed without closing, so source-port pivots to the shared evidence blocker.",
                            frontier.slug, frontier.authoritative_clean_through
                        );
                        (
                            task,
                            Some(AutonomyGoal::SweepParity),
                            Some(SOURCE_PORT_BACKLOG_EVIDENCE_BLOCKER_TARGET.to_owned()),
                            Some(SOURCE_PORT_BACKLOG_EVIDENCE_BLOCKER_TARGET.to_owned()),
                            summary,
                        )
                    } else {
                        let task = source_port_frontier_sweep_task(repo_root, frontier, &note);
                        let lane_marker = source_port_frontier_sweep_lane_marker(repo_root);
                        let summary = format!(
                            "TAS frontier {} is closed through frame {}; all proactive verification strategies and backlog issues exhausted, so source-port pivots to sweep parity hardening.",
                            frontier.slug, frontier.authoritative_clean_through
                        );
                        (
                            task,
                            Some(AutonomyGoal::SweepParity),
                            Some("state-sweep".to_owned()),
                            Some(lane_marker),
                            summary,
                        )
                    }
                } else {
                    let batch_size = (routines_per_task as usize).min(uncovered.len());
                    let batch = &uncovered[..batch_size];
                    let first_label = &batch[0].1;
                    let task = source_port_frontier_gameplay_backlog_task(
                        frontier,
                        &note,
                        batch,
                        uncovered.len(),
                    );
                    let summary = format!(
                        "TAS frontier {} is closed through frame {}; no actionable case-queue, delegated to gameplay-backlog ({} uncovered sub_ labels, batch of {}, first: {}).",
                        frontier.slug,
                        frontier.authoritative_clean_through,
                        uncovered.len(),
                        batch_size,
                        first_label
                    );
                    (
                        task,
                        Some(AutonomyGoal::GameplayBacklog),
                        Some(first_label.clone()),
                        Some(format!("gameplay-backlog:{first_label}")),
                        summary,
                    )
                }
            }
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
    let base_progress_fingerprint = source_port_frontier_progress_fingerprint(
        frontier,
        delegated_goal,
        delegated_target.as_deref(),
        lane_fingerprint.as_deref(),
    );
    let mut task_package = TaskPackage {
        generated_at: autonomy_timestamp(),
        cycle,
        progress_fingerprint: base_progress_fingerprint.clone(),
        ..task_package
    };
    let dirty_files = tracked_dirty_files(repo_root);
    let mut repeated_stalls =
        source_port_recent_matching_stall_count(repo_root, &base_progress_fingerprint);
    // Mechanism A: iteration 1 must not silently retry work that already
    // landed cleanly. If the candidate `(target, fingerprint)` matches any
    // clean-completion row in the recent run history, lift the recovery
    // trigger into the existing `repeated_stalls >= 2` cascade so the
    // dispatch arms pivot instead of re-emitting byte-identical work.
    if repeated_stalls < 2
        && dirty_files.is_empty()
        && delegated_target.as_deref().is_some_and(|target| {
            source_port_repeats_recent_clean_completion(
                repo_root,
                target,
                &base_progress_fingerprint,
            )
        })
    {
        repeated_stalls = 2;
    }
    // Mechanism C: the sweep-parity lane marker embeds fresh timestamps every
    // cycle, so `repeated_stalls` never accumulates even when the agent is
    // producing back-to-back docs-only "Refresh sweep parity evidence" commits.
    // Inspect git history directly: if the last 3+ consecutive commits touched
    // only the sweep-refresh prose docs, force the stalled-recovery cascade to
    // fire.
    let recent_commits = recent_commit_summaries(repo_root, 5);
    let docs_only_refresh_streak = sweep_refresh_streak(&recent_commits);
    if docs_only_refresh_streak >= 3 {
        repeated_stalls = repeated_stalls.max(2);
    }
    let mut chose_branch: &'static str = "none";
    if !dirty_files.is_empty() {
        chose_branch = "dirty";
        apply_dirty_tree_recovery(&mut task_package, &dirty_files, delegated_goal);
    } else if repeated_stalls >= 2 {
        let repeated_cycles = repeated_stalls + 1;
        match delegated_goal {
            Some(AutonomyGoal::CaseQueue) => {
                // Stalled case-queue: pivot to gameplay-backlog if uncovered routines exist
                let uncovered = collect_uncovered_sub_labels(repo_root).unwrap_or_default();
                if uncovered.is_empty() {
                    // All sub_ covered; try proactive verification before sweep-parity
                    let pv_report =
                        evaluate_proactive_verification_cycle(repo_root, cycle, routines_per_task)?;
                    if let Some(pv_task) = pv_report.task_package {
                        chose_branch = "case_queue_pv";
                        let target = pv_task.progress_fingerprint.clone();
                        task_package = TaskPackage {
                            generated_at: autonomy_timestamp(),
                            cycle,
                            progress_fingerprint: base_progress_fingerprint.clone(),
                            capability: CapabilityId::SourcePortFrontierTask,
                            delegated_goal: Some(AutonomyGoal::ProactiveVerification),
                            delegated_target: Some(target.clone()),
                            recovery_mode: Some(RecoveryMode::StalledFingerprint),
                            recovery_step: Some(RecoveryStep::PivotToNextLane),
                            stalled_repetition_count: Some(repeated_cycles),
                            pivot_reason: Some(format!(
                                "case-queue stalled {repeated_cycles} cycles; all sub_ covered, pivot to proactive-verification ({target})"
                            )),
                            next_fallback_goal: Some(AutonomyGoal::ProactiveVerification),
                            ..source_port_frontier_proactive_task(
                                pv_task,
                                frontier,
                                &note,
                                target,
                                "case-queue stalled and all sub_ labels are covered; proactive verification has work"
                                    .to_owned(),
                            )
                        };
                    } else {
                        chose_branch = "case_queue_sweep";
                        let sweep_task =
                            source_port_frontier_sweep_task(repo_root, frontier, &note);
                        task_package = TaskPackage {
                            generated_at: autonomy_timestamp(),
                            cycle,
                            progress_fingerprint: base_progress_fingerprint.clone(),
                            delegated_goal: Some(AutonomyGoal::SweepParity),
                            delegated_target: Some("state-sweep".to_owned()),
                            recovery_mode: Some(RecoveryMode::StalledFingerprint),
                            recovery_step: Some(RecoveryStep::PivotToNextLane),
                            stalled_repetition_count: Some(repeated_cycles),
                            pivot_reason: Some(format!(
                                "case-queue stalled {repeated_cycles} cycles; all sub_ covered, proactive-verification exhausted, pivot to sweep-parity"
                            )),
                            next_fallback_goal: Some(AutonomyGoal::SweepParity),
                            ..sweep_task
                        };
                    }
                } else {
                    chose_branch = "case_queue_backlog";
                    let batch_size = (routines_per_task as usize).min(uncovered.len());
                    let batch = &uncovered[..batch_size];
                    let first_label = batch[0].1.clone();
                    let backlog_task = source_port_frontier_gameplay_backlog_task(
                        frontier,
                        &note,
                        batch,
                        uncovered.len(),
                    );
                    task_package = TaskPackage {
                        generated_at: autonomy_timestamp(),
                        cycle,
                        progress_fingerprint: base_progress_fingerprint.clone(),
                        delegated_goal: Some(AutonomyGoal::GameplayBacklog),
                        delegated_target: Some(first_label.clone()),
                        recovery_mode: Some(RecoveryMode::StalledFingerprint),
                        recovery_step: Some(RecoveryStep::PivotToNextLane),
                        stalled_repetition_count: Some(repeated_cycles),
                        pivot_reason: Some(format!(
                            "case-queue delegated fingerprint repeated for {repeated_cycles} stalled cycles; pivot to gameplay-backlog (next: {first_label})"
                        )),
                        next_fallback_goal: Some(AutonomyGoal::GameplayBacklog),
                        ..backlog_task
                    };
                }
            }
            Some(AutonomyGoal::SweepParity) => {
                let queue = build_case_queue_snapshot(repo_root)?;
                if let Some(recording) = queue.selected_recording.as_ref() {
                    chose_branch = "sweep_parity_recording";
                    let recording_task =
                        source_port_frontier_recording_task(frontier, &note, &queue, recording);
                    task_package = TaskPackage {
                        generated_at: autonomy_timestamp(),
                        cycle,
                        progress_fingerprint: base_progress_fingerprint.clone(),
                        delegated_goal: Some(AutonomyGoal::CaseQueue),
                        delegated_target: Some(recording.slug.clone()),
                        recovery_mode: Some(RecoveryMode::StalledFingerprint),
                        recovery_step: Some(RecoveryStep::PivotToNextLane),
                        stalled_repetition_count: Some(repeated_cycles),
                        pivot_reason: Some(format!(
                            "sweep-parity delegated fingerprint repeated for {} stalled cycles; pivot to gameplay case derivation from recording {}",
                            repeated_cycles, recording.slug
                        )),
                        next_fallback_goal: Some(AutonomyGoal::CaseQueue),
                        ..recording_task
                    };
                } else if docs_only_refresh_streak >= 3 {
                    // The previous `RefreshEvidence` recovery step is itself
                    // what produced the docs-only commit streak; asking for
                    // another refresh perpetuates the loop. Pivot to
                    // proactive-verification instead, regardless of whether
                    // `evaluate_proactive_verification_cycle` returns a task —
                    // the operator-visible pivot_reason and fallback goal are
                    // what break the loop.
                    chose_branch = "sweep_parity_docs_only_streak";
                    let pv_report =
                        evaluate_proactive_verification_cycle(repo_root, cycle, routines_per_task)?;
                    let pivot_reason = format!(
                        "last {docs_only_refresh_streak} commits were docs-only sweep-parity refreshes; pivot to proactive-verification instead of emitting another release-gate evidence refresh"
                    );
                    if let Some(pv_task) = pv_report.task_package {
                        let target = pv_task.progress_fingerprint.clone();
                        task_package = TaskPackage {
                            generated_at: autonomy_timestamp(),
                            cycle,
                            progress_fingerprint: base_progress_fingerprint.clone(),
                            capability: CapabilityId::SourcePortFrontierTask,
                            delegated_goal: Some(AutonomyGoal::ProactiveVerification),
                            delegated_target: Some(target.clone()),
                            recovery_mode: Some(RecoveryMode::StalledFingerprint),
                            recovery_step: Some(RecoveryStep::PivotToNextLane),
                            stalled_repetition_count: Some(repeated_cycles),
                            pivot_reason: Some(pivot_reason),
                            next_fallback_goal: Some(AutonomyGoal::ProactiveVerification),
                            ..source_port_frontier_proactive_task(
                                pv_task,
                                frontier,
                                &note,
                                target,
                                "sweep-parity refresh loop detected; pivoting to proactive-verification".to_owned(),
                            )
                        };
                    } else {
                        task_package = TaskPackage {
                            generated_at: autonomy_timestamp(),
                            cycle,
                            progress_fingerprint: base_progress_fingerprint.clone(),
                            recovery_mode: Some(RecoveryMode::StalledFingerprint),
                            recovery_step: Some(RecoveryStep::PivotToNextLane),
                            stalled_repetition_count: Some(repeated_cycles),
                            pivot_reason: Some(pivot_reason.clone()),
                            ..source_port_frontier_proactive_exhausted_recovery_task(
                                frontier,
                                &note,
                                docs_only_refresh_streak,
                                pivot_reason,
                            )
                        };
                    }
                } else {
                    chose_branch = "sweep_parity_refresh";
                    apply_stalled_recovery(
                        &mut task_package,
                        &base_progress_fingerprint,
                        repeated_cycles,
                        RecoveryStep::RefreshEvidence,
                        format!(
                            "sweep-parity delegated fingerprint repeated for {repeated_cycles} stalled cycles; narrow the next run to release-gate evidence refresh instead of another broad lane replay"
                        ),
                        Some(AutonomyGoal::SweepParity),
                    );
                }
            }
            _ => {
                // Stalled on a backlog-issue or unknown lane. Try proactive
                // verification (enemy audits, loc_ coverage, mutation testing)
                // before falling back to re-verify.
                let pv_report =
                    evaluate_proactive_verification_cycle(repo_root, cycle, routines_per_task)?;
                if let Some(pv_task) = pv_report.task_package {
                    chose_branch = "catchall_pv";
                    let target = pv_task.progress_fingerprint.clone();
                    task_package = TaskPackage {
                        generated_at: autonomy_timestamp(),
                        cycle,
                        progress_fingerprint: base_progress_fingerprint.clone(),
                        capability: CapabilityId::SourcePortFrontierTask,
                        delegated_goal: Some(AutonomyGoal::ProactiveVerification),
                        delegated_target: Some(target.clone()),
                        recovery_mode: Some(RecoveryMode::StalledFingerprint),
                        recovery_step: Some(RecoveryStep::PivotToNextLane),
                        stalled_repetition_count: Some(repeated_cycles),
                        pivot_reason: Some(format!(
                            "backlog-issue stalled {repeated_cycles} cycles; pivot to proactive-verification ({target})"
                        )),
                        next_fallback_goal: Some(AutonomyGoal::ProactiveVerification),
                        ..source_port_frontier_proactive_task(
                            pv_task,
                            frontier,
                            &note,
                            target,
                            "current source-port lane stalled; proactive verification has work"
                                .to_owned(),
                        )
                    };
                } else {
                    chose_branch = "catchall_reverify";
                    apply_stalled_recovery(
                        &mut task_package,
                        &base_progress_fingerprint,
                        repeated_cycles,
                        RecoveryStep::ReverifyCurrentLane,
                        format!(
                            "delegated fingerprint repeated for {repeated_cycles} stalled cycles; proactive-verification exhausted, re-verify current lane"
                        ),
                        delegated_goal,
                    );
                }
            }
        }
    }
    let progress_fingerprint = if let Some(step) = task_package.recovery_step {
        format!("{}:recovery:{}", base_progress_fingerprint, step.as_str())
    } else {
        base_progress_fingerprint.clone()
    };
    task_package
        .progress_fingerprint
        .clone_from(&progress_fingerprint);
    // Mechanism B: if the recovery cascade could only decorate the same lane
    // (deterministic `:recovery:<step>` suffix that already landed cleanly),
    // mark the package so the runner short-circuits to the operator message
    // on iteration 1 instead of emitting another identical retry.
    if task_package.recovery_mode.is_some()
        && task_package
            .delegated_target
            .as_deref()
            .is_some_and(|target| {
                source_port_repeats_recent_clean_completion(
                    repo_root,
                    target,
                    &progress_fingerprint,
                )
            })
    {
        task_package.definitively_stalled = true;
    }
    emit_recovery_trace(
        repo_root,
        cycle,
        delegated_goal,
        delegated_target.as_deref(),
        &base_progress_fingerprint,
        dirty_files.len(),
        repeated_stalls,
        chose_branch,
        &task_package,
        &progress_fingerprint,
    );
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
        progress_fingerprint,
        capabilities,
    })
}

fn source_port_frontier_context(frontier: &ActiveFrontierState) -> FrontierContext {
    FrontierContext {
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
    }
}

fn source_port_frontier_proactive_task(
    mut task: TaskPackage,
    frontier: &ActiveFrontierState,
    note: &NextAgentNoteDirective,
    target: String,
    delegation_reason: String,
) -> TaskPackage {
    task.capability = CapabilityId::SourcePortFrontierTask;
    task.frontier_context = Some(source_port_frontier_context(frontier));
    task.delegated_goal = Some(AutonomyGoal::ProactiveVerification);
    task.delegated_target = Some(target);
    task.delegation_reason = Some(delegation_reason);
    task.next_agent_note = Some(note.clone());
    task
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
        generated_at: String::new(),
        cycle: 0,
        progress_fingerprint: String::new(),
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
        frontier_context: Some(source_port_frontier_context(frontier)),
        case_queue_context: None,
        delegated_goal: None,
        delegated_target: None,
        delegation_reason: None,
        recovery_mode: None,
        recovery_step: None,
        stalled_repetition_count: None,
        pivot_reason: None,
        next_fallback_goal: None,
        next_agent_note: Some(note.clone()),
        target_routines: None,
        definitively_stalled: false,
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
        generated_at: String::new(),
        cycle: 0,
        progress_fingerprint: String::new(),
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
            selected_recording: None,
            open_cases: queue.open_cases.clone(),
            blocked_cases: queue.blocked_cases.clone(),
            untriaged_recordings: queue.untriaged_recordings.clone(),
        }),
        delegated_goal: Some(AutonomyGoal::CaseQueue),
        delegated_target: Some(candidate.case_id.clone()),
        delegation_reason: Some(delegation_reason),
        recovery_mode: None,
        recovery_step: None,
        stalled_repetition_count: None,
        pivot_reason: None,
        next_fallback_goal: Some(AutonomyGoal::SweepParity),
        next_agent_note: Some(note.clone()),
        target_routines: None,
        definitively_stalled: false,
    }
}

fn source_port_frontier_sweep_task(
    repo_root: &Path,
    frontier: &ActiveFrontierState,
    note: &NextAgentNoteDirective,
) -> TaskPackage {
    let delegation_reason = "the TAS frontier is closed and no actionable reduced case is available, so the next repo-local lane is sweep parity hardening".to_owned();
    let release_gate_latest = latest_release_gate_failure_detail(repo_root, "1p-accurate");
    let release_gate_focus = release_gate_focus_evidence(repo_root, "1p-accurate");
    let release_gate_signal = latest_release_gate_signal(repo_root);
    let state_sweep_signal = latest_state_sweep_signal(repo_root);
    let mut evidence = vec![
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
    ];
    if let Some((detail, artefact_path)) = &release_gate_latest {
        evidence.push(EvidenceItem {
            label: "release-gate-failure".to_owned(),
            status: "failed".to_owned(),
            detail: detail.clone(),
            artefact_path: Some(artefact_path.clone()),
        });
    } else if release_gate_signal.ok {
        evidence.push(EvidenceItem {
            label: "release-gate-status".to_owned(),
            status: "passed".to_owned(),
            detail: release_gate_signal.detail.clone(),
            artefact_path: release_gate_signal.artefact_path.clone(),
        });
    }
    if let Some((detail, artefact_path)) = &release_gate_focus {
        evidence.push(EvidenceItem {
            label: "release-gate-focus".to_owned(),
            status: "correlated".to_owned(),
            detail: detail.clone(),
            artefact_path: Some(artefact_path.clone()),
        });
    }
    evidence.push(EvidenceItem {
        label: "state-sweep-status".to_owned(),
        status: if state_sweep_signal.ok {
            "passed".to_owned()
        } else {
            "failed".to_owned()
        },
        detail: state_sweep_signal.detail.clone(),
        artefact_path: state_sweep_signal.artefact_path.clone(),
    });
    let first_done_criterion = if release_gate_latest.is_some() {
        "Refresh the persisted release-gate evidence in the sweep lane so the next-agent note cites the latest failure and state-sweep correlation."
            .to_owned()
    } else if release_gate_signal.ok {
        "Refresh the persisted sweep-parity evidence so the next-agent note cites the passing release gate and latest state-sweep report."
            .to_owned()
    } else {
        "Refresh the persisted release-gate evidence in the sweep lane so the next-agent note cites the current gate status."
            .to_owned()
    };
    TaskPackage {
        capability: CapabilityId::SourcePortFrontierTask,
        title: "Advance source-port via sweep parity hardening".to_owned(),
        summary: format!(
            "{} is authoritative-clean through frame {}; continue with sweep parity and release-gate hardening instead of reopening TAS work.",
            frontier.recording, frontier.authoritative_clean_through
        ),
        generated_at: String::new(),
        cycle: 0,
        progress_fingerprint: String::new(),
        sources: vec![
            SOURCE_PORT_FRONTIER_STATE_PATH.to_owned(),
            SOURCE_PORT_RESEARCH_MAP_PATH.to_owned(),
            "docs/source-port/methodology.md".to_owned(),
            SOURCE_PORT_RELEASE_GATE_PATH.to_owned(),
        ],
        implementation_targets: vec![
            "crates/kidcli/src/autonomy.rs".to_owned(),
            "crates/kidcli/src/sweep/release_gate.rs".to_owned(),
            "docs/source-port/methodology.md".to_owned(),
            SOURCE_PORT_RESEARCH_MAP_PATH.to_owned(),
        ],
        commands: vec![
            "cargo run -p kidcli -- autonomous-loop --goal sweep-parity".to_owned(),
            "cargo run --release -p kidcli -- sweep --levels 0-104 --tier state".to_owned(),
            "mise run release-gate".to_owned(),
        ],
        done_criteria: vec![
            first_done_criterion,
            "Advance sweep parity, release-gate hardening, or another explicit verification lane with new evidence."
                .to_owned(),
            "Do not reopen TAS-frontier work unless a new authoritative frontier appears.".to_owned(),
        ],
        evidence,
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
        recovery_mode: None,
        recovery_step: None,
        stalled_repetition_count: None,
        pivot_reason: None,
        next_fallback_goal: Some(AutonomyGoal::CaseQueue),
        next_agent_note: Some(note.clone()),
        target_routines: None,
        definitively_stalled: false,
    }
}

fn source_port_frontier_gameplay_backlog_task(
    frontier: &ActiveFrontierState,
    note: &NextAgentNoteDirective,
    routines: &[(String, String, String, usize, Option<String>)],
    total_uncovered: usize,
) -> TaskPackage {
    let (subsystem, label, asm_file, line_number, priority) = &routines[0];
    let batch_size = routines.len();
    let delegation_reason = format!(
        "TAS frontier closed, no actionable case-queue; delegated to gameplay-backlog ({total_uncovered} uncovered sub_ labels)"
    );

    let (title, summary, done_criteria, target_routines) = if batch_size == 1 {
        (
            format!("Implement ASM routine {label} ({subsystem})"),
            format!(
                "{} is authoritative-clean through frame {}; implement {} from {} line {}.",
                frontier.recording,
                frontier.authoritative_clean_through,
                label,
                asm_file,
                line_number
            ),
            vec![
                format!("Native code cites {} with a passing test.", label),
                "verify-slice-fast passes.".to_owned(),
            ],
            None,
        )
    } else {
        let subsystems: Vec<&str> = routines.iter().map(|(s, ..)| s.as_str()).collect();
        let all_same = subsystems.iter().all(|s| *s == subsystems[0]);
        let subsystem_desc = if all_same {
            subsystems[0].to_owned()
        } else {
            "mixed subsystems".to_owned()
        };
        let routine_list: Vec<String> = routines
            .iter()
            .map(|(sub, lbl, file, line, _)| format!("{lbl} ({sub}, {file}:{line})"))
            .collect();
        let tr: Vec<TargetRoutine> = routines
            .iter()
            .map(|(sub, lbl, file, line, _)| TargetRoutine {
                label: lbl.clone(),
                subsystem: sub.clone(),
                asm_file: file.clone(),
                line_number: *line,
            })
            .collect();
        (
            format!("Implement {batch_size} ASM routines ({subsystem_desc})"),
            format!(
                "{} is authoritative-clean through frame {}; implement {} routines: {}.",
                frontier.recording,
                frontier.authoritative_clean_through,
                batch_size,
                routine_list.join(", ")
            ),
            vec![
                format!(
                    "Commit {} routines with passing tests (or fewer if blocked). Each commit cites its ASM label.",
                    batch_size
                ),
                "verify-slice-fast passes after each commit.".to_owned(),
            ],
            Some(tr),
        )
    };

    // Collect unique ASM files for sources
    let mut asm_sources: Vec<String> = routines
        .iter()
        .map(|(_, _, file, ..)| format!(".cache/upstream/kid-chameleon-disasm/{file}"))
        .collect();
    asm_sources.sort();
    asm_sources.dedup();
    let mut sources = asm_sources;
    sources.push("docs/source-port/asm-coverage.yaml".to_owned());
    sources.push("docs/source-port/ralph-gameplay-loop.md".to_owned());

    // Collect unique subsystems for commands
    let mut subsystems: Vec<String> = routines.iter().map(|(s, ..)| s.clone()).collect();
    subsystems.sort();
    subsystems.dedup();
    let mut commands: Vec<String> = subsystems
        .iter()
        .map(|s| format!("kidcli asm-coverage --uncovered --subsystem {s}"))
        .collect();
    commands.push("mise run verify-slice-fast".to_owned());
    commands.push("kidcli asm-coverage --update".to_owned());

    TaskPackage {
        capability: CapabilityId::GameplayBacklogTask,
        title,
        summary,
        generated_at: String::new(),
        cycle: 0,
        progress_fingerprint: String::new(),
        sources,
        implementation_targets: vec![
            "crates/kid-core/src/".to_owned(),
            "crates/kid-data/src/".to_owned(),
        ],
        commands,
        done_criteria,
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
                label: "uncovered-routine".to_owned(),
                status: priority.as_deref().unwrap_or("unranked").to_owned(),
                detail: format!(
                    "{label} in {asm_file} line {line_number} ({total_uncovered} uncovered sub_ remain)"
                ),
                artefact_path: Some(PathBuf::from("docs/source-port/asm-coverage.yaml")),
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
        delegated_goal: Some(AutonomyGoal::GameplayBacklog),
        delegated_target: Some(label.to_owned()),
        delegation_reason: Some(delegation_reason),
        recovery_mode: None,
        recovery_step: None,
        stalled_repetition_count: None,
        pivot_reason: None,
        next_fallback_goal: Some(AutonomyGoal::SweepParity),
        next_agent_note: Some(note.clone()),
        target_routines,
        definitively_stalled: false,
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
    let Ok(contents) = fs::read_to_string(note_path) else {
        return (
            NextAgentNoteStatus::Missing,
            None,
            "no next-agent note present for the active frontier".to_owned(),
        );
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
        .or_else(|| {
            queue
                .selected_recording
                .as_ref()
                .map(|recording| recording.slug.as_str())
        })
        .unwrap_or("no-selected-case");
    let note = case_queue_next_agent_note(repo_root, active_slug);
    let selected_task = queue
        .selected_case
        .as_ref()
        .map(|candidate| case_queue_task(candidate, &queue, &note))
        .or_else(|| {
            queue
                .selected_recording
                .as_ref()
                .map(|recording| case_queue_recording_task(recording, &queue, &note))
        });
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
    } else if let Some(recording) = queue.selected_recording.as_ref() {
        let mut summary = format!(
            "Selected recording {} ({}f) for gameplay case derivation.",
            recording.slug, recording.frame_count
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

    let progress_fingerprint = case_queue_progress_fingerprint(
        queue.selected_case.as_ref(),
        queue.selected_recording.as_ref(),
    );
    let selected_task = selected_task.map(|task| TaskPackage {
        generated_at: autonomy_timestamp(),
        cycle,
        progress_fingerprint: progress_fingerprint.clone(),
        ..task
    });

    Ok(CycleReport {
        goal: AutonomyGoal::CaseQueue.slug().to_owned(),
        cycle,
        goal_complete: false,
        stalled: false,
        task_package: selected_task.clone(),
        selected_task,
        blocker,
        progress_fingerprint,
        capabilities,
    })
}

fn evaluate_gameplay_backlog_cycle(
    repo_root: &Path,
    cycle: u32,
    routines_per_task: u32,
) -> Result<CycleReport> {
    let coverage_path = repo_root.join("docs/source-port/asm-coverage.yaml");
    if !coverage_path.is_file() {
        bail!("asm-coverage.yaml not found; run `kidcli asm-coverage --update` first");
    }

    // Run asm-coverage to find uncovered routines
    let uncovered = collect_uncovered_sub_labels(repo_root)?;

    if uncovered.is_empty() {
        return Ok(CycleReport {
            goal: AutonomyGoal::GameplayBacklog.slug().to_owned(),
            cycle,
            goal_complete: true,
            stalled: false,
            task_package: None,
            selected_task: None,
            blocker: None,
            progress_fingerprint: "all-sub-covered".to_owned(),
            capabilities: vec![CapabilityReport {
                id: CapabilityId::GameplayBacklogTask,
                title: "ASM-driven gameplay backlog".to_owned(),
                status: CapabilityStatus::Complete,
                summary: "All sub_ labels are cited in native code.".to_owned(),
            }],
        });
    }

    let batch_size = (routines_per_task as usize).min(uncovered.len());
    let batch = &uncovered[..batch_size];
    let (subsystem, label, asm_file, line_number, priority) = &batch[0];

    let (title, summary_text, done_criteria, target_routines) = if batch_size == 1 {
        (
            format!("Implement ASM routine {label} ({subsystem})"),
            format!(
                "Uncovered sub_ label {label} in {asm_file} (line {line_number}). Read the ASM, write a failing test, implement, run verify-slice-fast."
            ),
            vec![
                format!("Native code cites {} with a passing test.", label),
                "verify-slice-fast passes.".to_owned(),
                "asm-coverage.yaml updated to reflect the new citation.".to_owned(),
            ],
            None,
        )
    } else {
        let subsystems: Vec<&str> = batch.iter().map(|(s, ..)| s.as_str()).collect();
        let all_same = subsystems.iter().all(|s| *s == subsystems[0]);
        let subsystem_desc = if all_same {
            subsystems[0].to_owned()
        } else {
            "mixed subsystems".to_owned()
        };
        let routine_list: Vec<String> = batch
            .iter()
            .map(|(sub, lbl, file, line, _)| format!("{lbl} ({sub}, {file}:{line})"))
            .collect();
        let tr: Vec<TargetRoutine> = batch
            .iter()
            .map(|(sub, lbl, file, line, _)| TargetRoutine {
                label: lbl.clone(),
                subsystem: sub.clone(),
                asm_file: file.clone(),
                line_number: *line,
            })
            .collect();
        (
            format!("Implement {batch_size} ASM routines ({subsystem_desc})"),
            format!(
                "{} uncovered routines to implement: {}.",
                batch_size,
                routine_list.join(", ")
            ),
            vec![
                format!(
                    "Commit {} routines with passing tests (or fewer if blocked). Each commit cites its ASM label.",
                    batch_size
                ),
                "verify-slice-fast passes after each commit.".to_owned(),
                "asm-coverage.yaml updated to reflect new citations.".to_owned(),
            ],
            Some(tr),
        )
    };

    // Collect unique ASM files for sources
    let mut asm_sources: Vec<String> = batch
        .iter()
        .map(|(_, _, file, ..)| format!(".cache/upstream/kid-chameleon-disasm/{file}"))
        .collect();
    asm_sources.sort();
    asm_sources.dedup();
    let mut sources = asm_sources;
    sources.push("docs/source-port/asm-coverage.yaml".to_owned());
    sources.push("docs/source-port/ralph-gameplay-loop.md".to_owned());

    // Collect unique subsystems for commands
    let mut cmd_subsystems: Vec<String> = batch.iter().map(|(s, ..)| s.clone()).collect();
    cmd_subsystems.sort();
    cmd_subsystems.dedup();
    let mut commands: Vec<String> = cmd_subsystems
        .iter()
        .map(|s| format!("kidcli asm-coverage --uncovered --subsystem {s}"))
        .collect();
    commands.push("mise run verify-slice-fast".to_owned());
    commands.push("kidcli asm-coverage --update".to_owned());

    let task = TaskPackage {
        capability: CapabilityId::GameplayBacklogTask,
        title,
        summary: summary_text,
        generated_at: autonomy_timestamp(),
        cycle,
        progress_fingerprint: String::new(),
        sources,
        implementation_targets: vec![
            "crates/kid-core/src/".to_owned(),
            "crates/kid-data/src/".to_owned(),
        ],
        commands,
        done_criteria,
        evidence: vec![EvidenceItem {
            label: "uncovered-routine".to_owned(),
            status: priority.clone().unwrap_or_else(|| "unranked".to_owned()),
            detail: format!(
                "{} in {} line {} ({} uncovered sub_ labels remain)",
                label,
                asm_file,
                line_number,
                uncovered.len()
            ),
            artefact_path: Some(repo_root.join("docs/source-port/asm-coverage.yaml")),
        }],
        frontier_context: None,
        case_queue_context: None,
        delegated_goal: None,
        delegated_target: None,
        delegation_reason: None,
        recovery_mode: None,
        recovery_step: None,
        stalled_repetition_count: None,
        pivot_reason: None,
        next_fallback_goal: None,
        next_agent_note: None,
        target_routines,
        definitively_stalled: false,
    };

    let progress_fingerprint = format!("gameplay-backlog:{label}");
    let task = TaskPackage {
        progress_fingerprint: progress_fingerprint.clone(),
        ..task
    };

    Ok(CycleReport {
        goal: AutonomyGoal::GameplayBacklog.slug().to_owned(),
        cycle,
        goal_complete: false,
        stalled: false,
        task_package: Some(task.clone()),
        selected_task: Some(task),
        blocker: None,
        progress_fingerprint,
        capabilities: vec![CapabilityReport {
            id: CapabilityId::GameplayBacklogTask,
            title: "ASM-driven gameplay backlog".to_owned(),
            status: CapabilityStatus::Ready,
            summary: format!(
                "{} uncovered sub_ labels; next: {} in {}",
                uncovered.len(),
                label,
                subsystem
            ),
        }],
    })
}

// ---------------------------------------------------------------------------
// Proactive verification config types
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
struct ProactiveVerificationConfig {
    strategies: Vec<ProactiveStrategy>,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "name", rename_all = "kebab-case")]
enum ProactiveStrategy {
    RootCauseBacklog {
        enabled: bool,
        root_cause_marker: String,
    },
    EnemyFamilyAudit {
        enabled: bool,
        priority_order: Vec<String>,
    },
    LocCoverageDrive {
        enabled: bool,
        min_cluster_size: usize,
        #[serde(default)]
        focus_subsystems: Vec<String>,
        /// Cap cite-only commits per autonomy session so this strategy
        /// stops absorbing every cycle once the uncovered queue is stale.
        #[serde(default = "default_loc_coverage_max_per_session")]
        max_per_session: u32,
    },
    MutationTesting {
        enabled: bool,
        output_dir: String,
        targets: Vec<MutationTarget>,
        /// Ordered list of module filenames. Each cycle picks the current
        /// rotation entry via `mutation-pointer.json` and advances on
        /// completion so the cascade never blocks on a single large file.
        #[serde(default)]
        rotation: Vec<String>,
        /// Optional time-box for large files (e.g. enemies.rs enumerates
        /// thousands of candidates). When present, the task passes
        /// `--shard` and `--test-timeout-multiplier` hints.
        #[serde(default)]
        max_candidates_per_cycle: Option<u32>,
    },
}

fn default_loc_coverage_max_per_session() -> u32 {
    2
}

#[derive(Debug, Deserialize)]
struct MutationTarget {
    module: String,
    package: String,
    tier: u8,
}

const PROACTIVE_VERIFICATION_CONFIG_PATH: &str =
    "docs/source-port/proactive-verification-config.yaml";

fn load_proactive_verification_config(repo_root: &Path) -> Result<ProactiveVerificationConfig> {
    let config_path = repo_root.join(PROACTIVE_VERIFICATION_CONFIG_PATH);
    if config_path.is_file() {
        let content = fs::read_to_string(&config_path)
            .with_context(|| format!("failed to read {}", config_path.display()))?;
        serde_yaml::from_str(&content)
            .with_context(|| format!("failed to parse {}", config_path.display()))
    } else {
        // Hardcoded fallback so the goal works without a config file.
        Ok(ProactiveVerificationConfig {
            strategies: vec![
                ProactiveStrategy::RootCauseBacklog {
                    enabled: true,
                    root_cause_marker: "Root cause identified".to_owned(),
                },
                ProactiveStrategy::EnemyFamilyAudit {
                    enabled: true,
                    priority_order: vec![
                        "enemy_tank".to_owned(),
                        "enemy_tank_shooting".to_owned(),
                        "enemy_mini_hopping_skull".to_owned(),
                        "enemy_drips".to_owned(),
                        "enemy_ninja".to_owned(),
                        "enemy_fireball".to_owned(),
                        "enemy_armadillo".to_owned(),
                        "enemy_diamond".to_owned(),
                        "enemy_emo_rock".to_owned(),
                        "enemy_sphere".to_owned(),
                    ],
                },
                ProactiveStrategy::LocCoverageDrive {
                    enabled: true,
                    min_cluster_size: 3,
                    focus_subsystems: Vec::new(),
                    max_per_session: default_loc_coverage_max_per_session(),
                },
                ProactiveStrategy::MutationTesting {
                    enabled: true,
                    output_dir: ".cache/mutants".to_owned(),
                    targets: vec![
                        MutationTarget {
                            module: "enemies.rs".to_owned(),
                            package: "kid-core".to_owned(),
                            tier: 1,
                        },
                        MutationTarget {
                            module: "combat.rs".to_owned(),
                            package: "kid-core".to_owned(),
                            tier: 1,
                        },
                        MutationTarget {
                            module: "blocks.rs".to_owned(),
                            package: "kid-core".to_owned(),
                            tier: 1,
                        },
                    ],
                    rotation: vec![
                        "enemies.rs".to_owned(),
                        "combat.rs".to_owned(),
                        "blocks.rs".to_owned(),
                    ],
                    max_candidates_per_cycle: None,
                },
            ],
        })
    }
}

// ---------------------------------------------------------------------------
// Proactive verification cycle evaluator
// ---------------------------------------------------------------------------

fn evaluate_proactive_verification_cycle(
    repo_root: &Path,
    cycle: u32,
    routines_per_task: u32,
) -> Result<CycleReport> {
    let config = load_proactive_verification_config(repo_root)?;
    let work_counts = refresh_work_type_counts(repo_root).unwrap_or_default();
    let frontier_state = load_source_port_frontier_state(repo_root).ok();
    let rejected_fingerprints = collect_rejected_fingerprints(frontier_state.as_ref());

    for strategy in &config.strategies {
        match strategy {
            ProactiveStrategy::RootCauseBacklog {
                enabled,
                root_cause_marker,
            } if *enabled => {
                if let Some(report) =
                    try_root_cause_backlog_strategy(repo_root, cycle, root_cause_marker)?
                    && !rejected_fingerprints.contains(&report.progress_fingerprint)
                {
                    return Ok(report);
                }
            }
            ProactiveStrategy::EnemyFamilyAudit {
                enabled,
                priority_order,
            } if *enabled => {
                if let Some(report) =
                    try_enemy_family_audit_strategy(repo_root, cycle, priority_order)?
                    && !rejected_fingerprints.contains(&report.progress_fingerprint)
                {
                    return Ok(report);
                }
            }
            ProactiveStrategy::LocCoverageDrive {
                enabled,
                min_cluster_size,
                focus_subsystems,
                max_per_session,
            } if *enabled => {
                // Mature debug-field candidates preempt coverage busywork so
                // the loop does not queue another `Cite ASM loc_` task when
                // there is a ready promotion waiting.
                if let Some(report) = try_candidate_promotion_strategy(
                    cycle,
                    frontier_state.as_ref(),
                    &rejected_fingerprints,
                ) {
                    return Ok(report);
                }
                // Circuit breaker: once this session has already produced
                // max_per_session cite-only commits, skip to the next
                // strategy instead of queuing another `Cite ASM loc_` task.
                if work_counts.cite_only >= *max_per_session {
                    continue;
                }
                if let Some(report) = try_loc_coverage_drive_strategy(
                    repo_root,
                    cycle,
                    routines_per_task,
                    *min_cluster_size,
                    focus_subsystems,
                )? && !rejected_fingerprints.contains(&report.progress_fingerprint)
                {
                    return Ok(report);
                }
            }
            ProactiveStrategy::MutationTesting {
                enabled,
                output_dir,
                targets,
                rotation,
                max_candidates_per_cycle,
            } if *enabled => {
                if let Some(report) = try_mutation_testing_strategy(
                    repo_root,
                    cycle,
                    output_dir,
                    targets,
                    rotation,
                    *max_candidates_per_cycle,
                ) && !rejected_fingerprints.contains(&report.progress_fingerprint)
                {
                    return Ok(report);
                }
            }
            _ => {} // disabled strategies
        }
    }

    // All strategies exhausted — goal complete.
    Ok(CycleReport {
        goal: AutonomyGoal::ProactiveVerification.slug().to_owned(),
        cycle,
        goal_complete: true,
        stalled: false,
        task_package: None,
        selected_task: None,
        blocker: None,
        progress_fingerprint: "proactive-verification:all-exhausted".to_owned(),
        capabilities: vec![CapabilityReport {
            id: CapabilityId::ProactiveVerificationTask,
            title: "Proactive verification".to_owned(),
            status: CapabilityStatus::Complete,
            summary: "All proactive verification strategies exhausted.".to_owned(),
        }],
    })
}

fn collect_rejected_fingerprints(state: Option<&SourcePortFrontierState>) -> BTreeSet<String> {
    state
        .map(|s| {
            s.active_frontier
                .rejected_hypotheses
                .iter()
                .filter_map(|entry| entry.fingerprint.clone())
                .collect()
        })
        .unwrap_or_default()
}

fn try_candidate_promotion_strategy(
    cycle: u32,
    state: Option<&SourcePortFrontierState>,
    rejected_fingerprints: &BTreeSet<String>,
) -> Option<CycleReport> {
    let state = state?;
    let ready = state
        .candidate_promotions
        .iter()
        .find(|entry| entry.maturity_status.as_deref() == Some("ready"))?;

    let target = ready.key.clone();
    let fingerprint = format!("proactive-verification:promote-debug-field:{target}");
    if rejected_fingerprints.contains(&fingerprint) {
        return None;
    }

    let promotion_target = ready
        .promotion_target
        .clone()
        .unwrap_or_else(|| "typed gameplay state".to_owned());
    let reason = ready.reason.clone().unwrap_or_else(|| {
        "debug-field age plus repeated parity use qualify it for promotion".to_owned()
    });

    let task = TaskPackage {
        capability: CapabilityId::ProactiveVerificationTask,
        title: format!("Promote debug field `{target}` to typed state"),
        summary: format!(
            "Debug field `{target}` is ready for promotion to {promotion_target}. Move it from the debug bag into typed gameplay state per the debug-field registry contract, update referencing parity logic, and drop the `candidate` status.",
        ),
        generated_at: String::new(),
        cycle: 0,
        progress_fingerprint: String::new(),
        sources: vec![
            "docs/source-port/debug-field-registry.yaml".to_owned(),
            SOURCE_PORT_FRONTIER_STATE_PATH.to_owned(),
            "CLAUDE.md".to_owned(),
        ],
        implementation_targets: vec![
            "crates/kid-core/src/".to_owned(),
            "crates/kidcli/src/".to_owned(),
        ],
        commands: vec![
            "mise run audit-debug-fields".to_owned(),
            "mise run verify-slice-fast".to_owned(),
        ],
        done_criteria: vec![
            format!("`{target}` removed from `debug_fields`/`gameplay_debug_fields` maps."),
            format!(
                "`{target}` registry entry status changes from `candidate` and the matching `candidate_promotions[]` entry is removed from frontier-state.yaml."
            ),
            "mise run audit-debug-fields exits 0.".to_owned(),
            "verify-slice-fast passes.".to_owned(),
        ],
        evidence: vec![EvidenceItem {
            label: "candidate-promotion".to_owned(),
            status: "ready".to_owned(),
            detail: reason,
            artefact_path: Some(PathBuf::from("docs/source-port/debug-field-registry.yaml")),
        }],
        frontier_context: None,
        case_queue_context: None,
        delegated_goal: None,
        delegated_target: None,
        delegation_reason: None,
        recovery_mode: None,
        recovery_step: None,
        stalled_repetition_count: None,
        pivot_reason: None,
        next_fallback_goal: None,
        next_agent_note: None,
        target_routines: None,
        definitively_stalled: false,
    };

    Some(proactive_verification_report(
        cycle,
        "promote-debug-field",
        &target,
        task,
    ))
}

fn proactive_verification_report(
    cycle: u32,
    strategy_name: &str,
    target: &str,
    task: TaskPackage,
) -> CycleReport {
    let fingerprint = format!("proactive-verification:{strategy_name}:{target}");
    let task = TaskPackage {
        generated_at: autonomy_timestamp(),
        cycle,
        progress_fingerprint: fingerprint.clone(),
        ..task
    };
    CycleReport {
        goal: AutonomyGoal::ProactiveVerification.slug().to_owned(),
        cycle,
        goal_complete: false,
        stalled: false,
        task_package: Some(task.clone()),
        selected_task: Some(task),
        blocker: None,
        progress_fingerprint: fingerprint,
        capabilities: vec![CapabilityReport {
            id: CapabilityId::ProactiveVerificationTask,
            title: "Proactive verification".to_owned(),
            status: CapabilityStatus::Ready,
            summary: format!("Strategy: {strategy_name}, target: {target}"),
        }],
    }
}

fn try_root_cause_backlog_strategy(
    repo_root: &Path,
    cycle: u32,
    root_cause_marker: &str,
) -> Result<Option<CycleReport>> {
    let issues = load_open_backlog_issues(repo_root)?;
    let marker_lower = root_cause_marker.to_ascii_lowercase();
    let ready_issue = issues.iter().find(|issue| {
        issue.summary.to_ascii_lowercase().contains(&marker_lower)
            || issue.verify.to_ascii_lowercase().contains(&marker_lower)
    });

    let Some(issue) = ready_issue else {
        return Ok(None);
    };

    let impl_target = match issue.system.as_str() {
        "enemy" => "crates/kid-core/src/enemies.rs",
        "block" | "platform" => "crates/kid-core/src/blocks.rs",
        "player" => "crates/kid-core/src/kid_main.rs",
        "combat" => "crates/kid-core/src/combat.rs",
        "render" => "crates/kid-core/src/render.rs",
        _ => "crates/kid-core/src/lib.rs",
    };

    let task = TaskPackage {
        capability: CapabilityId::ProactiveVerificationTask,
        title: format!("Fix backlog #{}: {}", issue.id, issue.summary),
        summary: format!(
            "Backlog #{} ({}, {}) has root-cause evidence. Implement the fix, add a regression test citing the ASM label, run verify-slice-fast.",
            issue.id, issue.system, issue.priority
        ),
        generated_at: String::new(),
        cycle: 0,
        progress_fingerprint: String::new(),
        sources: vec![
            SOURCE_PORT_BACKLOG_PATH.to_owned(),
            "docs/source-port/ralph-gameplay-loop.md".to_owned(),
            ".cache/upstream/kid-chameleon-disasm/kid.asm".to_owned(),
        ],
        implementation_targets: vec![impl_target.to_owned()],
        commands: vec![
            "mise run verify-slice-fast".to_owned(),
            "cargo run -p kidcli -- asm-coverage --update".to_owned(),
        ],
        done_criteria: vec![
            format!("Backlog #{} marked [x].", issue.id),
            "verify-slice-fast passes.".to_owned(),
            "Regression test cites ASM label.".to_owned(),
        ],
        evidence: vec![EvidenceItem {
            label: "root-cause-backlog".to_owned(),
            status: "ready".to_owned(),
            detail: format!("#{}: {}", issue.id, issue.summary),
            artefact_path: Some(PathBuf::from(SOURCE_PORT_BACKLOG_PATH)),
        }],
        frontier_context: None,
        case_queue_context: None,
        delegated_goal: None,
        delegated_target: None,
        delegation_reason: None,
        recovery_mode: None,
        recovery_step: None,
        stalled_repetition_count: None,
        pivot_reason: None,
        next_fallback_goal: None,
        next_agent_note: None,
        target_routines: None,
        definitively_stalled: false,
    };

    let target = ready_backlog_issue_progress_marker(issue);
    Ok(Some(proactive_verification_report(
        cycle,
        "root-cause-backlog",
        &target,
        task,
    )))
}

fn try_enemy_family_audit_strategy(
    repo_root: &Path,
    cycle: u32,
    priority_order: &[String],
) -> Result<Option<CycleReport>> {
    use crate::asm_coverage;

    let coverage_path = repo_root.join("docs/source-port/asm-coverage.yaml");
    if !coverage_path.is_file() {
        return Ok(None);
    }
    let content = fs::read_to_string(&coverage_path)?;

    #[derive(Deserialize)]
    struct CovMap {
        subsystems: BTreeMap<String, CovSub>,
    }
    #[derive(Deserialize)]
    struct CovSub {
        loc_cited: usize,
        #[serde(default)]
        asm_source: Option<String>,
        labels: Vec<CovLabel>,
    }
    #[derive(Deserialize)]
    struct CovLabel {
        label: String,
        kind: asm_coverage::LabelKind,
        asm_file: String,
        line_number: usize,
        #[serde(default)]
        cited_in: Vec<String>,
    }

    let map: CovMap = serde_yaml::from_str(&content)?;

    for family in priority_order {
        if let Some(sub) = map.subsystems.get(family)
            && sub.loc_cited == 0
        {
            let asm_file = sub
                .asm_source
                .clone()
                .unwrap_or_else(|| format!("code/enemy/{family}.asm"));
            let target_routines: Vec<TargetRoutine> = sub
                .labels
                .iter()
                .filter(|l| l.kind == asm_coverage::LabelKind::Loc && l.cited_in.is_empty())
                .map(|l| TargetRoutine {
                    label: l.label.clone(),
                    subsystem: family.clone(),
                    asm_file: l.asm_file.clone(),
                    line_number: l.line_number,
                })
                .collect();

            let task = TaskPackage {
                capability: CapabilityId::ProactiveVerificationTask,
                title: format!("Audit enemy family {family} against ASM"),
                summary: format!(
                    "Enemy family {family} has zero loc_ citations. Read the ASM, compare to Rust, write tests for divergences, fix bugs."
                ),
                generated_at: String::new(),
                cycle: 0,
                progress_fingerprint: String::new(),
                sources: vec![
                    format!(".cache/upstream/kid-chameleon-disasm/{}", asm_file),
                    "crates/kid-core/src/enemies.rs".to_owned(),
                    ".agents/skills/m68k-kid-asm/SKILL.md".to_owned(),
                    ".cache/upstream/kid-chameleon-disasm/variables.asm".to_owned(),
                ],
                implementation_targets: vec!["crates/kid-core/src/enemies.rs".to_owned()],
                commands: vec![
                    format!(
                        "cargo run -p kidcli -- asm-coverage --uncovered --subsystem {}",
                        family
                    ),
                    "mise run verify-slice-fast".to_owned(),
                    "cargo run -p kidcli -- asm-coverage --update".to_owned(),
                ],
                done_criteria: vec![
                    format!("At least one loc_ in {} now cited.", family),
                    "verify-slice-fast passes.".to_owned(),
                ],
                evidence: vec![EvidenceItem {
                    label: "enemy-family-audit".to_owned(),
                    status: "zero-loc-coverage".to_owned(),
                    detail: format!(
                        "{}: {} uncovered loc_ labels",
                        family,
                        target_routines.len()
                    ),
                    artefact_path: Some(coverage_path.clone()),
                }],
                frontier_context: None,
                case_queue_context: None,
                delegated_goal: None,
                delegated_target: None,
                delegation_reason: None,
                recovery_mode: None,
                recovery_step: None,
                stalled_repetition_count: None,
                pivot_reason: None,
                next_fallback_goal: None,
                next_agent_note: None,
                target_routines: if target_routines.is_empty() {
                    None
                } else {
                    Some(target_routines)
                },
                definitively_stalled: false,
            };

            return Ok(Some(proactive_verification_report(
                cycle,
                "enemy-family-audit",
                family,
                task,
            )));
        }
    }

    Ok(None)
}

fn try_loc_coverage_drive_strategy(
    repo_root: &Path,
    cycle: u32,
    routines_per_task: u32,
    min_cluster_size: usize,
    focus_subsystems: &[String],
) -> Result<Option<CycleReport>> {
    let uncovered = collect_loc_labels(repo_root, LocCoverageSelection::Uncovered)?;
    if uncovered.is_empty() {
        return try_loc_promotion_strategy(
            repo_root,
            cycle,
            routines_per_task,
            min_cluster_size,
            focus_subsystems,
        );
    }

    // Group by subsystem, pick the one with the most uncovered labels.
    let mut by_subsystem: BTreeMap<&str, Vec<&LocCoverageCandidate>> = BTreeMap::new();
    for entry in &uncovered {
        if focus_subsystems.is_empty() || focus_subsystems.iter().any(|f| f == &entry.subsystem) {
            by_subsystem
                .entry(&entry.subsystem)
                .or_default()
                .push(entry);
        }
    }

    if by_subsystem.is_empty() {
        return Ok(None);
    }

    // Pick subsystem with most uncovered labels.
    let (best_subsystem, best_entries) = by_subsystem
        .iter()
        .max_by_key(|(_, entries)| entries.len())
        .unwrap();

    if best_entries.len() < min_cluster_size {
        return Ok(None);
    }

    let batch_size = (routines_per_task as usize).min(best_entries.len());
    let batch = &best_entries[..batch_size];

    let target_routines: Vec<TargetRoutine> = batch
        .iter()
        .map(|entry| TargetRoutine {
            label: entry.label.clone(),
            subsystem: entry.subsystem.clone(),
            asm_file: entry.asm_file.clone(),
            line_number: entry.line_number,
        })
        .collect();

    let first_label = batch[0].label.clone();

    let task = TaskPackage {
        capability: CapabilityId::ProactiveVerificationTask,
        title: format!("Audit {batch_size} loc_ labels in {best_subsystem}"),
        summary: format!(
            "{} uncovered loc_ labels in {} (total uncovered: {}). Read the ASM for each, compare to native code, write tests.",
            batch_size,
            best_subsystem,
            uncovered.len()
        ),
        generated_at: String::new(),
        cycle: 0,
        progress_fingerprint: String::new(),
        sources: vec![
            format!(".cache/upstream/kid-chameleon-disasm/{}", batch[0].asm_file),
            ".agents/skills/m68k-kid-asm/SKILL.md".to_owned(),
            ".cache/upstream/kid-chameleon-disasm/variables.asm".to_owned(),
        ],
        implementation_targets: vec!["crates/kid-core/src/".to_owned()],
        commands: vec![
            "mise run verify-slice-fast".to_owned(),
            "cargo run -p kidcli -- asm-coverage --update".to_owned(),
        ],
        done_criteria: vec![
            format!(
                "At least {} loc_ labels in {} now cited.",
                batch_size.min(3),
                best_subsystem
            ),
            "verify-slice-fast passes.".to_owned(),
        ],
        evidence: vec![EvidenceItem {
            label: "loc-coverage-drive".to_owned(),
            status: "uncovered".to_owned(),
            detail: format!(
                "{}: {} uncovered loc_ labels, batch of {}",
                best_subsystem,
                best_entries.len(),
                batch_size
            ),
            artefact_path: Some(repo_root.join("docs/source-port/asm-coverage.yaml")),
        }],
        frontier_context: None,
        case_queue_context: None,
        delegated_goal: None,
        delegated_target: None,
        delegation_reason: None,
        recovery_mode: None,
        recovery_step: None,
        stalled_repetition_count: None,
        pivot_reason: None,
        next_fallback_goal: None,
        next_agent_note: None,
        target_routines: Some(target_routines),
        definitively_stalled: false,
    };

    Ok(Some(proactive_verification_report(
        cycle,
        "loc-coverage-drive",
        &first_label,
        task,
    )))
}

fn try_loc_promotion_strategy(
    repo_root: &Path,
    cycle: u32,
    routines_per_task: u32,
    min_cluster_size: usize,
    focus_subsystems: &[String],
) -> Result<Option<CycleReport>> {
    let promotion_candidates = collect_loc_labels(repo_root, LocCoverageSelection::Promotion)?;
    if promotion_candidates.is_empty() {
        return Ok(None);
    }

    let mut by_subsystem: BTreeMap<&str, Vec<&LocCoverageCandidate>> = BTreeMap::new();
    for entry in &promotion_candidates {
        if focus_subsystems.is_empty() || focus_subsystems.iter().any(|f| f == &entry.subsystem) {
            by_subsystem
                .entry(&entry.subsystem)
                .or_default()
                .push(entry);
        }
    }

    if by_subsystem.is_empty() {
        return Ok(None);
    }

    let (best_subsystem, best_entries) = by_subsystem
        .iter()
        .max_by_key(|(_, entries)| entries.len())
        .unwrap();

    if best_entries.len() < min_cluster_size {
        return Ok(None);
    }

    let batch_size = (routines_per_task as usize).min(best_entries.len());
    let batch = &best_entries[..batch_size];
    let first_label = batch[0].label.clone();
    let target_routines: Vec<TargetRoutine> = batch
        .iter()
        .map(|entry| TargetRoutine {
            label: entry.label.clone(),
            subsystem: entry.subsystem.clone(),
            asm_file: entry.asm_file.clone(),
            line_number: entry.line_number,
        })
        .collect();

    let task = TaskPackage {
        capability: CapabilityId::ProactiveVerificationTask,
        title: format!("Promote {batch_size} test-cited loc_ labels in {best_subsystem}"),
        summary: format!(
            "{batch_size} loc_ labels in {best_subsystem} already have test citations but no implementation citations. Read the ASM, add the missing implementation citations or code-path coverage, and refresh asm coverage."
        ),
        generated_at: String::new(),
        cycle: 0,
        progress_fingerprint: String::new(),
        sources: vec![
            "docs/source-port/asm-coverage.yaml".to_owned(),
            format!(".cache/upstream/kid-chameleon-disasm/{}", batch[0].asm_file),
            ".agents/skills/m68k-kid-asm/SKILL.md".to_owned(),
        ],
        implementation_targets: vec!["crates/kid-core/src/".to_owned()],
        commands: vec![
            "mise run verify-slice-fast".to_owned(),
            "cargo run -p kidcli -- asm-coverage --update".to_owned(),
        ],
        done_criteria: vec![
            format!(
                "At least {} test-cited loc_ labels in {} now have implementation citations.",
                batch_size.min(3),
                best_subsystem
            ),
            "verify-slice-fast passes.".to_owned(),
        ],
        evidence: vec![EvidenceItem {
            label: "loc-coverage-promotion".to_owned(),
            status: "test-cited-without-implementation".to_owned(),
            detail: format!(
                "{}: {} test-cited loc_ labels waiting for implementation promotion, batch of {}",
                best_subsystem,
                best_entries.len(),
                batch_size
            ),
            artefact_path: Some(repo_root.join("docs/source-port/asm-coverage.yaml")),
        }],
        frontier_context: None,
        case_queue_context: None,
        delegated_goal: None,
        delegated_target: None,
        delegation_reason: None,
        recovery_mode: None,
        recovery_step: None,
        stalled_repetition_count: None,
        pivot_reason: None,
        next_fallback_goal: None,
        next_agent_note: None,
        target_routines: Some(target_routines),
        definitively_stalled: false,
    };

    Ok(Some(proactive_verification_report(
        cycle,
        "loc-coverage-promotion",
        &first_label,
        task,
    )))
}

fn try_mutation_testing_strategy(
    repo_root: &Path,
    cycle: u32,
    output_dir: &str,
    targets: &[MutationTarget],
    rotation: &[String],
    max_candidates_per_cycle: Option<u32>,
) -> Option<CycleReport> {
    // Build the candidate ordering. When a rotation list is provided the
    // cascade prefers the current rotation entry first so every cycle works
    // on one file at a time; other targets still act as fallback in their
    // original tier order. Without rotation the behaviour matches the
    // previous tier-sorted pass.
    let mut ordered: Vec<&MutationTarget> = Vec::new();
    let mut seen: BTreeSet<&str> = BTreeSet::new();

    if !rotation.is_empty() {
        let pointer_path = repo_root.join(MUTATION_POINTER_PATH);
        let mut pointer = load_mutation_pointer(&pointer_path);
        let mut start_index = pointer.index % rotation.len();

        // Skip past modules that have already reached Complete so the
        // rotation naturally advances at the end of each successful cycle
        // instead of cycling back to finished files.
        for _ in 0..rotation.len() {
            let module = &rotation[start_index];
            let Some(target) = targets.iter().find(|t| &t.module == module) else {
                start_index = (start_index + 1) % rotation.len();
                continue;
            };
            let module_stem = target.module.strip_suffix(".rs").unwrap_or(&target.module);
            let output_root_abs = repo_root.join(output_dir).join(module_stem);
            if matches!(
                inspect_mutation_run(&output_root_abs).status,
                MutationRunStatus::Complete
            ) {
                start_index = (start_index + 1) % rotation.len();
                continue;
            }
            break;
        }

        pointer.index = start_index;
        pointer.last_file.clone_from(&rotation[start_index]);
        pointer.last_run_at = autonomy_timestamp();
        let _ = write_mutation_pointer(&pointer_path, &pointer);

        for offset in 0..rotation.len() {
            let idx = (start_index + offset) % rotation.len();
            let module = &rotation[idx];
            if let Some(target) = targets.iter().find(|t| &t.module == module)
                && seen.insert(target.module.as_str())
            {
                ordered.push(target);
            }
        }
    }

    let mut tier_sorted: Vec<&MutationTarget> = targets.iter().collect();
    tier_sorted.sort_by_key(|t| t.tier);
    for target in tier_sorted {
        if seen.insert(target.module.as_str()) {
            ordered.push(target);
        }
    }

    for target in ordered {
        let module_stem = target.module.strip_suffix(".rs").unwrap_or(&target.module);
        let output_root = PathBuf::from(output_dir).join(module_stem);
        let output_root_abs = repo_root.join(&output_root);
        let run = inspect_mutation_run(&output_root_abs);

        match &run.status {
            MutationRunStatus::NotRun => {
                let task = mutation_run_task(target, &output_root, None, max_candidates_per_cycle);
                return Some(proactive_verification_report(
                    cycle,
                    "mutation-testing",
                    &format!("{module_stem}:run"),
                    task,
                ));
            }
            MutationRunStatus::Incomplete { detail } => {
                let detail_hash = stable_short_hash(detail);
                let task = mutation_run_task(
                    target,
                    &output_root,
                    Some(detail.clone()),
                    max_candidates_per_cycle,
                );
                return Some(proactive_verification_report(
                    cycle,
                    "mutation-testing",
                    &format!("{module_stem}:rerun:{detail_hash}"),
                    task,
                ));
            }
            MutationRunStatus::Survivors { detail } => {
                let detail_hash = stable_short_hash(detail);
                let task = mutation_triage_task(target, &output_root, &run, detail.clone());
                return Some(proactive_verification_report(
                    cycle,
                    "mutation-testing",
                    &format!("{module_stem}:triage:{detail_hash}"),
                    task,
                ));
            }
            MutationRunStatus::Complete => {}
        }
    }

    None
}

const MUTATION_POINTER_PATH: &str = ".cache/autonomy/proactive-verification/mutation-pointer.json";

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
struct MutationPointer {
    #[serde(default)]
    index: usize,
    #[serde(default)]
    last_file: String,
    #[serde(default)]
    last_run_at: String,
}

fn load_mutation_pointer(path: &Path) -> MutationPointer {
    fs::read_to_string(path)
        .ok()
        .and_then(|content| serde_json::from_str(&content).ok())
        .unwrap_or_default()
}

fn write_mutation_pointer(path: &Path, pointer: &MutationPointer) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("failed to create {}", parent.display()))?;
    }
    let json = serde_json::to_string_pretty(pointer)
        .with_context(|| format!("failed to serialise {}", path.display()))?;
    fs::write(path, format!("{json}\n"))
        .with_context(|| format!("failed to write {}", path.display()))?;
    Ok(())
}

const RL_SESSION_PATH: &str = ".cache/autonomy/rl-session.json";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum WorkType {
    CiteOnly,
    DocsOnly,
    Productive,
}

fn classify_commit_subject(subject: &str) -> WorkType {
    let trimmed = subject.trim_start();
    if trimmed.starts_with("Cite ASM loc_") {
        return WorkType::CiteOnly;
    }
    // Common dev-doc conventions: "docs: ...", "docs(area): ...", "Document ..."
    if trimmed.starts_with("docs:")
        || trimmed.starts_with("docs(")
        || trimmed.starts_with("Document ")
    {
        return WorkType::DocsOnly;
    }
    WorkType::Productive
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
struct WorkTypeCounts {
    #[serde(default)]
    cite_only: u32,
    #[serde(default)]
    docs_only: u32,
    #[serde(default)]
    productive: u32,
}

/// Paths whose commits we treat as "sweep-parity evidence refresh" — the
/// autonomy loop's observed dead-end when every other lane is green. A commit
/// that touches only these paths has encoded no new information (they embed
/// `.cache/sweep-runs/<timestamp>/report.json` paths and `cycle=N` counters
/// that change every run). See `sweep_refresh_streak` below.
const SWEEP_REFRESH_DOC_PATHS: &[&str] = &[
    "docs/source-port/methodology.md",
    "docs/source-port/research-map.md",
];

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct RecentCommitSummary {
    pub(crate) sha: String,
    pub(crate) subject: String,
    pub(crate) files: Vec<String>,
}

/// Returns the last `count` commits (newest first) with their changed file
/// list. Used by the handoff note and the dead-end detector to give the next
/// agent — and the loop itself — a picture of whether we're about to repeat
/// ourselves. Silently returns an empty Vec if git is unavailable.
fn recent_commit_summaries(repo_root: &Path, count: usize) -> Vec<RecentCommitSummary> {
    if count == 0 {
        return Vec::new();
    }
    const DELIM: &str = "__KID_COMMIT__";
    let format = format!("{DELIM}%n%h%n%s");
    let output = Command::new("git")
        .args([
            "log",
            &format!("-{count}"),
            "--no-merges",
            "--name-only",
            &format!("--format={format}"),
        ])
        .current_dir(repo_root)
        .output();
    let Ok(output) = output else {
        return Vec::new();
    };
    if !output.status.success() {
        return Vec::new();
    }
    parse_recent_commit_summaries(&String::from_utf8_lossy(&output.stdout))
}

fn parse_recent_commit_summaries(stdout: &str) -> Vec<RecentCommitSummary> {
    const DELIM: &str = "__KID_COMMIT__";
    let mut summaries = Vec::new();
    let mut iter = stdout.lines().peekable();
    while let Some(line) = iter.next() {
        if line.trim() != DELIM {
            continue;
        }
        let Some(sha) = iter.next() else { break };
        let Some(subject) = iter.next() else { break };
        let mut files = Vec::new();
        while let Some(next) = iter.peek() {
            if next.trim() == DELIM {
                break;
            }
            let file_line = iter.next().expect("peek succeeded");
            if file_line.is_empty() {
                continue;
            }
            files.push(file_line.to_owned());
        }
        summaries.push(RecentCommitSummary {
            sha: sha.to_owned(),
            subject: subject.to_owned(),
            files,
        });
    }
    summaries
}

fn is_sweep_refresh_commit(summary: &RecentCommitSummary) -> bool {
    !summary.files.is_empty()
        && summary
            .files
            .iter()
            .all(|path| SWEEP_REFRESH_DOC_PATHS.contains(&path.as_str()))
}

/// Counts consecutive most-recent commits (starting from `summaries[0]`) that
/// are pure sweep-parity evidence refreshes. A streak >= 3 means the loop is
/// spinning; downstream code short-circuits to proactive-verification.
fn sweep_refresh_streak(summaries: &[RecentCommitSummary]) -> usize {
    summaries
        .iter()
        .take_while(|commit| is_sweep_refresh_commit(commit))
        .count()
}

/// Re-walks commits recorded in the rl-session.json session window and
/// updates the work_type_counts field. Returns the updated counters.
/// Silently no-ops if the session file is missing (the rl-loop harness
/// creates it; outside that harness the circuit breaker is effectively
/// disabled, which matches one-off manual invocations).
fn refresh_work_type_counts(repo_root: &Path) -> Result<WorkTypeCounts> {
    let path = repo_root.join(RL_SESSION_PATH);
    let Ok(content) = fs::read_to_string(&path) else {
        return Ok(WorkTypeCounts::default());
    };
    let mut session: serde_json::Value = serde_json::from_str(&content)
        .with_context(|| format!("failed to parse {}", path.display()))?;

    let start_sha = session
        .get("start_commit_sha")
        .and_then(serde_json::Value::as_str)
        .map(str::to_owned);
    let Some(start_sha) = start_sha else {
        return Ok(WorkTypeCounts::default());
    };

    let output = Command::new("git")
        .args(["log", "--format=%s", &format!("{start_sha}..HEAD")])
        .current_dir(repo_root)
        .output();
    let Ok(output) = output else {
        return Ok(WorkTypeCounts::default());
    };
    if !output.status.success() {
        return Ok(WorkTypeCounts::default());
    }

    let mut counts = WorkTypeCounts::default();
    for line in String::from_utf8_lossy(&output.stdout).lines() {
        match classify_commit_subject(line) {
            WorkType::CiteOnly => counts.cite_only += 1,
            WorkType::DocsOnly => counts.docs_only += 1,
            WorkType::Productive => counts.productive += 1,
        }
    }

    if let Some(obj) = session.as_object_mut() {
        obj.insert(
            "work_type_counts".to_owned(),
            serde_json::to_value(&counts).unwrap_or(serde_json::Value::Null),
        );
        if let Ok(serialised) = serde_json::to_string_pretty(&session) {
            let _ = fs::write(&path, format!("{serialised}\n"));
        }
    }

    Ok(counts)
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum MutationRunStatus {
    NotRun,
    Incomplete { detail: String },
    Survivors { detail: String },
    Complete,
}

#[derive(Debug, Clone)]
struct MutationRunInspection {
    outcomes_path: PathBuf,
    missed_path: PathBuf,
    timeout_path: PathBuf,
    status: MutationRunStatus,
}

fn inspect_mutation_run(output_root_abs: &Path) -> MutationRunInspection {
    let run_dir = output_root_abs.join("mutants.out");
    let outcomes_path = run_dir.join("outcomes.json");
    let mutants_path = run_dir.join("mutants.json");
    let missed_path = run_dir.join("missed.txt");
    let timeout_path = run_dir.join("timeout.txt");
    let debug_log_path = run_dir.join("debug.log");

    let status = if !outcomes_path.exists() {
        MutationRunStatus::NotRun
    } else if let Some(detail) = mutation_debug_failure(&debug_log_path) {
        MutationRunStatus::Incomplete { detail }
    } else {
        match mutation_outcome_counts(&outcomes_path, &mutants_path) {
            Ok((candidate_count, outcome_count)) if outcome_count < candidate_count => {
                MutationRunStatus::Incomplete {
                    detail: format!(
                        "partial cargo-mutants results: {outcome_count}/{candidate_count} mutants have outcomes"
                    ),
                }
            }
            Ok(_) => {
                let missed = trimmed_file(&missed_path);
                let timed_out = trimmed_file(&timeout_path);
                if missed.is_empty() && timed_out.is_empty() {
                    MutationRunStatus::Complete
                } else {
                    let missed_count = line_count(&missed);
                    let timeout_count = line_count(&timed_out);
                    let first = missed
                        .lines()
                        .chain(timed_out.lines())
                        .next()
                        .unwrap_or("unknown survivor");
                    MutationRunStatus::Survivors {
                        detail: format!(
                            "{missed_count} missed and {timeout_count} timed-out mutants remain; first: {first}"
                        ),
                    }
                }
            }
            Err(detail) => MutationRunStatus::Incomplete { detail },
        }
    };

    MutationRunInspection {
        outcomes_path,
        missed_path,
        timeout_path,
        status,
    }
}

fn mutation_debug_failure(debug_log_path: &Path) -> Option<String> {
    let content = fs::read_to_string(debug_log_path).ok()?;
    let failure_markers = [
        "No space left on device",
        "Worker thread failed",
        "scenario execution internal error",
        "interrupted",
        "ERROR cargo_mutants",
    ];
    content
        .lines()
        .find(|line| failure_markers.iter().any(|marker| line.contains(marker)))
        .map(|line| line.trim().to_owned())
}

fn mutation_outcome_counts(
    outcomes_path: &Path,
    mutants_path: &Path,
) -> std::result::Result<(usize, usize), String> {
    let outcomes_content = fs::read_to_string(outcomes_path)
        .map_err(|error| format!("failed to read {}: {error}", outcomes_path.display()))?;
    let outcomes: serde_json::Value = serde_json::from_str(&outcomes_content)
        .map_err(|error| format!("failed to parse {}: {error}", outcomes_path.display()))?;
    let outcome_count = outcomes
        .get("outcomes")
        .and_then(serde_json::Value::as_array)
        .ok_or_else(|| format!("{} has no outcomes array", outcomes_path.display()))?
        .iter()
        .filter(|outcome| {
            outcome
                .get("scenario")
                .and_then(|scenario| scenario.get("Mutant"))
                .is_some()
        })
        .count();

    let mutants_content = fs::read_to_string(mutants_path)
        .map_err(|error| format!("failed to read {}: {error}", mutants_path.display()))?;
    let mutants: serde_json::Value = serde_json::from_str(&mutants_content)
        .map_err(|error| format!("failed to parse {}: {error}", mutants_path.display()))?;
    let candidate_count = mutants
        .as_array()
        .ok_or_else(|| format!("{} is not a mutants array", mutants_path.display()))?
        .len();

    Ok((candidate_count, outcome_count))
}

fn trimmed_file(path: &Path) -> String {
    fs::read_to_string(path)
        .map(|content| content.trim().to_owned())
        .unwrap_or_default()
}

fn line_count(content: &str) -> usize {
    content
        .lines()
        .filter(|line| !line.trim().is_empty())
        .count()
}

fn stable_short_hash(content: &str) -> String {
    let mut hash = 0xcbf2_9ce4_8422_2325u64;
    for byte in content.as_bytes() {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
    }
    format!("{hash:016x}")[..8].to_owned()
}

fn mutation_command(target: &MutationTarget, output_root: &Path) -> String {
    format!(
        "cargo mutants -p {} -f {} -j 4 --gitignore=true --output {} -- --lib -- --skip demo_ --skip published_tas --skip snapshot",
        target.package,
        target.module,
        output_root.display()
    )
}

fn mutation_common_sources(target: &MutationTarget) -> Vec<String> {
    vec![
        format!("crates/{}/src/{}", target.package, target.module),
        ".cache/upstream/kid-chameleon-disasm/kid.asm".to_owned(),
        ".agents/skills/m68k-kid-asm/SKILL.md".to_owned(),
    ]
}

fn mutation_common_targets(target: &MutationTarget) -> Vec<String> {
    vec![format!("crates/{}/src/{}", target.package, target.module)]
}

fn mutation_run_task(
    target: &MutationTarget,
    output_root: &Path,
    incomplete_detail: Option<String>,
    max_candidates_per_cycle: Option<u32>,
) -> TaskPackage {
    let command = mutation_command(target, output_root);
    let outcomes_path = output_root.join("mutants.out/outcomes.json");
    let mutants_path = output_root.join("mutants.out/mutants.json");
    let clean_command = format!(
        "rm -rf {}/mutants.out {}/mutants.out.old",
        output_root.display(),
        output_root.display()
    );
    let is_rerun = incomplete_detail.is_some();
    let cycle_cap_note = max_candidates_per_cycle.map(|cap| {
        format!(
            " If the file enumerates more than {cap} candidates, prioritise the first {cap} and leave the rest for the next rotation cycle."
        )
    });

    TaskPackage {
        capability: CapabilityId::ProactiveVerificationTask,
        title: if is_rerun {
            format!("Rerun incomplete mutation testing on {} (tier {})", target.module, target.tier)
        } else {
            format!("Run mutation testing on {} (tier {})", target.module, target.tier)
        },
        summary: {
            let base = if is_rerun {
                format!(
                    "Previous cargo-mutants output for {} is incomplete. Clean the partial output, rerun with gitignore forced on, then triage survivors if any remain.",
                    target.module
                )
            } else {
                format!(
                    "Run cargo-mutants on {} in package {} with gitignore forced on. If it finds surviving mutants, triage one into an ASM-backed code fix or test.",
                    target.module, target.package
                )
            };
            match cycle_cap_note.as_ref() {
                Some(note) => format!("{base}{note}"),
                None => base,
            }
        },
        generated_at: String::new(),
        cycle: 0,
        progress_fingerprint: String::new(),
        sources: mutation_common_sources(target),
        implementation_targets: mutation_common_targets(target),
        commands: if is_rerun {
            vec![clean_command, command, "mise run verify-slice-fast".to_owned()]
        } else {
            vec![command, "mise run verify-slice-fast".to_owned()]
        },
        done_criteria: vec![
            format!(
                "Mutation testing outcomes exist at {}.",
                outcomes_path.display()
            ),
            format!(
                "{} covers every candidate listed in {}.",
                outcomes_path.display(),
                mutants_path.display()
            ),
            "If missed or timed-out mutants remain, convert at least one into an ASM-backed fix or test before committing.".to_owned(),
            "verify-slice-fast passes after any code/test change.".to_owned(),
        ],
        evidence: vec![EvidenceItem {
            label: if is_rerun {
                "mutation-testing-incomplete".to_owned()
            } else {
                "mutation-testing".to_owned()
            },
            status: if is_rerun {
                "incomplete".to_owned()
            } else {
                "not-run".to_owned()
            },
            detail: incomplete_detail.unwrap_or_else(|| {
                format!("{} (tier {}) has no mutation testing results yet", target.module, target.tier)
            }),
            artefact_path: if is_rerun {
                Some(outcomes_path)
            } else {
                None
            },
        }],
        frontier_context: None,
        case_queue_context: None,
        delegated_goal: None,
        delegated_target: None,
        delegation_reason: None,
        recovery_mode: None,
        recovery_step: None,
        stalled_repetition_count: None,
        pivot_reason: None,
        next_fallback_goal: None,
        next_agent_note: None,
        target_routines: None,
        definitively_stalled: false,
    }
}

fn mutation_triage_task(
    target: &MutationTarget,
    output_root: &Path,
    run: &MutationRunInspection,
    detail: String,
) -> TaskPackage {
    TaskPackage {
        capability: CapabilityId::ProactiveVerificationTask,
        title: format!("Triage surviving mutation results for {}", target.module),
        summary: format!(
            "cargo-mutants completed for {}, but survivors remain. Read the ASM for the first surviving mutant and turn it into a bug fix or an exact-value test.",
            target.module
        ),
        generated_at: String::new(),
        cycle: 0,
        progress_fingerprint: String::new(),
        sources: mutation_common_sources(target),
        implementation_targets: mutation_common_targets(target),
        commands: vec![
            format!("sed -n '1,80p' {}", run.missed_path.display()),
            format!("sed -n '1,80p' {}", run.timeout_path.display()),
            mutation_command(target, output_root),
            "mise run verify-slice-fast".to_owned(),
        ],
        done_criteria: vec![
            "At least one surviving or timed-out mutant is killed by an ASM-backed code fix or exact-value test.".to_owned(),
            "The relevant ASM label is cited in the test or implementation comment.".to_owned(),
            "cargo-mutants is rerun for the affected survivor or module.".to_owned(),
            "verify-slice-fast passes.".to_owned(),
        ],
        evidence: vec![EvidenceItem {
            label: "mutation-testing-survivors".to_owned(),
            status: "needs-triage".to_owned(),
            detail,
            artefact_path: Some(run.outcomes_path.clone()),
        }],
        frontier_context: None,
        case_queue_context: None,
        delegated_goal: None,
        delegated_target: None,
        delegation_reason: None,
        recovery_mode: None,
        recovery_step: None,
        stalled_repetition_count: None,
        pivot_reason: None,
        next_fallback_goal: None,
        next_agent_note: None,
        target_routines: None,
        definitively_stalled: false,
    }
}

/// Collect uncovered sub_ labels from asm-coverage, sorted by priority then subsystem gap.
/// Returns: Vec<(subsystem, label, asm_file, line_number, priority)>
fn collect_uncovered_sub_labels(
    repo_root: &Path,
) -> Result<Vec<(String, String, String, usize, Option<String>)>> {
    use crate::asm_coverage;

    let upstream_dir = repo_root.join(".cache/upstream/kid-chameleon-disasm");
    if !upstream_dir.is_dir() {
        bail!(
            "disassembly not found at {}; run `mise run bootstrap-upstream`",
            upstream_dir.display()
        );
    }

    // Re-use the asm_coverage internals by running the tool in-process
    // We invoke the public function in a capture mode via --json --uncovered
    // but since we need structured data, we'll parse the coverage YAML directly.
    let coverage_path = repo_root.join("docs/source-port/asm-coverage.yaml");
    let content = fs::read_to_string(&coverage_path)
        .with_context(|| format!("failed to read {}", coverage_path.display()))?;

    #[derive(Deserialize)]
    struct CovMap {
        subsystems: std::collections::BTreeMap<String, CovSub>,
    }
    #[derive(Deserialize)]
    struct CovSub {
        labels: Vec<CovLabel>,
    }
    #[derive(Deserialize)]
    struct CovLabel {
        label: String,
        kind: asm_coverage::LabelKind,
        asm_file: String,
        line_number: usize,
        #[serde(default)]
        cited_in: Vec<String>,
        #[serde(default)]
        priority: Option<String>,
        #[serde(default)]
        implementation_cited: Option<bool>,
    }

    let map: CovMap =
        serde_yaml::from_str(&content).with_context(|| "failed to parse asm-coverage.yaml")?;
    let backlog_issues = load_open_backlog_issues(repo_root).unwrap_or_default();

    let mut uncovered: Vec<(String, String, String, usize, Option<String>)> = Vec::new();
    for (subsystem, sub) in &map.subsystems {
        for label in &sub.labels {
            let implementation_cited = label
                .implementation_cited
                .unwrap_or(!label.cited_in.is_empty());
            if label.kind == asm_coverage::LabelKind::Sub && !implementation_cited {
                uncovered.push((
                    subsystem.clone(),
                    label.label.clone(),
                    label.asm_file.clone(),
                    label.line_number,
                    label.priority.clone(),
                ));
            }
        }
    }

    // Sort: explicit priority first, then open backlog signals, then subsystem/name.
    uncovered.sort_by(|a, b| {
        fn priority_rank(p: Option<&String>) -> u8 {
            match p.map(String::as_str) {
                Some("high") => 0,
                Some("medium") => 1,
                Some("low") => 2,
                _ => 3,
            }
        }
        fn subsystem_signal_rank(subsystem: &str, issues: &[BacklogIssueCandidate]) -> u8 {
            let has_enemy_signal = issues.iter().any(|issue| issue.system == "enemy");
            let has_core_signal = issues.iter().any(|issue| {
                matches!(
                    issue.system.as_str(),
                    "player" | "block" | "render" | "level" | "camera" | "form" | "menu"
                )
            });
            let matches_backlog_signal = (subsystem.starts_with("enemy_") && has_enemy_signal)
                || (subsystem == "kid_main" && has_core_signal);
            if matches_backlog_signal { 0 } else { 1 }
        }
        priority_rank(a.4.as_ref())
            .cmp(&priority_rank(b.4.as_ref()))
            .then_with(|| {
                subsystem_signal_rank(&a.0, &backlog_issues)
                    .cmp(&subsystem_signal_rank(&b.0, &backlog_issues))
            })
            .then_with(|| a.0.cmp(&b.0))
            .then_with(|| a.1.cmp(&b.1))
    });

    Ok(uncovered)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum LocCoverageSelection {
    Uncovered,
    Promotion,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct LocCoverageCandidate {
    subsystem: String,
    label: String,
    asm_file: String,
    line_number: usize,
    priority: Option<String>,
}

/// Collect loc_ labels for proactive coverage work, sorted by priority then subsystem gap.
fn collect_loc_labels(
    repo_root: &Path,
    selection: LocCoverageSelection,
) -> Result<Vec<LocCoverageCandidate>> {
    use crate::asm_coverage;

    let coverage_path = repo_root.join("docs/source-port/asm-coverage.yaml");
    let content = fs::read_to_string(&coverage_path)
        .with_context(|| format!("failed to read {}", coverage_path.display()))?;

    #[derive(Deserialize)]
    struct CovMap {
        subsystems: std::collections::BTreeMap<String, CovSub>,
    }
    #[derive(Deserialize)]
    struct CovSub {
        labels: Vec<CovLabel>,
    }
    #[derive(Deserialize)]
    struct CovLabel {
        label: String,
        kind: asm_coverage::LabelKind,
        asm_file: String,
        line_number: usize,
        #[serde(default)]
        cited_in: Vec<String>,
        #[serde(default)]
        priority: Option<String>,
        #[serde(default)]
        implementation_cited: Option<bool>,
        #[serde(default)]
        test_cited: Option<bool>,
    }

    let map: CovMap =
        serde_yaml::from_str(&content).with_context(|| "failed to parse asm-coverage.yaml")?;
    let backlog_issues = load_open_backlog_issues(repo_root).unwrap_or_default();

    let mut candidates: Vec<LocCoverageCandidate> = Vec::new();
    for (subsystem, sub) in &map.subsystems {
        for label in &sub.labels {
            let implementation_cited = label
                .implementation_cited
                .unwrap_or(!label.cited_in.is_empty());
            let test_cited = label.test_cited.unwrap_or(false);
            let matches_selection = match selection {
                LocCoverageSelection::Uncovered => !implementation_cited && !test_cited,
                LocCoverageSelection::Promotion => !implementation_cited && test_cited,
            };
            if label.kind == asm_coverage::LabelKind::Loc && matches_selection {
                candidates.push(LocCoverageCandidate {
                    subsystem: subsystem.clone(),
                    label: label.label.clone(),
                    asm_file: label.asm_file.clone(),
                    line_number: label.line_number,
                    priority: label.priority.clone(),
                });
            }
        }
    }

    // Sort: explicit priority first, then open backlog signals, then subsystem/name.
    candidates.sort_by(|a, b| {
        fn priority_rank(p: Option<&String>) -> u8 {
            match p.map(String::as_str) {
                Some("high") => 0,
                Some("medium") => 1,
                Some("low") => 2,
                _ => 3,
            }
        }
        fn subsystem_signal_rank(subsystem: &str, issues: &[BacklogIssueCandidate]) -> u8 {
            let has_enemy_signal = issues.iter().any(|issue| issue.system == "enemy");
            let has_core_signal = issues.iter().any(|issue| {
                matches!(
                    issue.system.as_str(),
                    "player" | "block" | "render" | "level" | "camera" | "form" | "menu"
                )
            });
            let matches_backlog_signal = (subsystem.starts_with("enemy_") && has_enemy_signal)
                || (subsystem == "kid_main" && has_core_signal);
            if matches_backlog_signal { 0 } else { 1 }
        }
        priority_rank(a.priority.as_ref())
            .cmp(&priority_rank(b.priority.as_ref()))
            .then_with(|| {
                subsystem_signal_rank(&a.subsystem, &backlog_issues)
                    .cmp(&subsystem_signal_rank(&b.subsystem, &backlog_issues))
            })
            .then_with(|| a.subsystem.cmp(&b.subsystem))
            .then_with(|| a.label.cmp(&b.label))
    });

    Ok(candidates)
}

fn build_case_queue_snapshot(repo_root: &Path) -> Result<CaseQueueSnapshot> {
    let mut cases = load_case_queue_candidates(repo_root)?;
    apply_case_queue_duplicate_penalties(&mut cases);
    cases.sort_by(case_queue_sort_key);

    let (selected_case, selection_reason) = select_case_queue_candidate(repo_root, &cases)?;
    let selectable_recordings = load_untriaged_recordings(repo_root, &cases)?;
    let (selected_recording, recording_reason) = if selected_case.is_none() {
        select_recording_queue_candidate(repo_root, &selectable_recordings)?
    } else {
        (None, None)
    };

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
    let untriaged_recordings = selectable_recordings;

    Ok(CaseQueueSnapshot {
        selected_case,
        selected_recording,
        selection_reason: selection_reason.or(recording_reason),
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
        let source_blocked = verify.blocked_reason.is_some();
        let autonomy_ready = !source_blocked
            && failure_class == Some(CaseFailureClass::Authoritative)
            && window_len <= 120
            && !verify.likely_subsystems.is_empty()
            && verify.likely_subsystems.len() <= 2;
        let status = if source_blocked {
            CaseQueueStatus::Blocked
        } else {
            case_queue_status(
                failure_class,
                autonomy_ready,
                window_len,
                verify.likely_subsystems.len(),
            )
        };
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
            frame_count: manifest
                .get("scenario")
                .and_then(|scenario| scenario.get("frame_count"))
                .and_then(serde_json::Value::as_u64)
                .unwrap_or(0) as usize,
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
        .map_or(frame_end.saturating_sub(checkpoint_frame), |failure| {
            failure.frame_index.saturating_sub(checkpoint_frame)
        });
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
        .map_or_else(|_| path.to_path_buf(), std::path::Path::to_path_buf)
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
                .map_or_else(|| "unknown".to_owned(), |frame| frame.to_string()),
            candidate.first_bad_field.as_deref().unwrap_or("unknown"),
            if candidate.autonomy_ready {
                "the hybrid gate allows a focused patch"
            } else {
                "this remains evidence-first until the case is tighter"
            }
        ),
        generated_at: String::new(),
        cycle: 0,
        progress_fingerprint: String::new(),
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
                            .map_or("unknown", CaseFailureClass::as_str),
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
            selected_recording: None,
            open_cases: queue.open_cases.clone(),
            blocked_cases: queue.blocked_cases.clone(),
            untriaged_recordings: queue.untriaged_recordings.clone(),
        }),
        delegated_goal: None,
        delegated_target: None,
        delegation_reason: queue.selection_reason.clone(),
        recovery_mode: None,
        recovery_step: None,
        stalled_repetition_count: None,
        pivot_reason: None,
        next_fallback_goal: Some(AutonomyGoal::SweepParity),
        next_agent_note: Some(note.clone()),
        target_routines: None,
        definitively_stalled: false,
    }
}

fn source_port_frontier_proactive_exhausted_recovery_task(
    frontier: &ActiveFrontierState,
    note: &NextAgentNoteDirective,
    docs_only_refresh_streak: usize,
    pivot_reason: String,
) -> TaskPackage {
    TaskPackage {
        capability: CapabilityId::SourcePortFrontierTask,
        title: "Record source-port verification stall".to_owned(),
        summary: format!(
            "{} is authoritative-clean through frame {}; sweep-parity has repeated docs-only refreshes and proactive-verification is currently exhausted, so record the recovery stall before touching sweep evidence docs again.",
            frontier.recording, frontier.authoritative_clean_through
        ),
        generated_at: String::new(),
        cycle: 0,
        progress_fingerprint: String::new(),
        sources: vec![
            SOURCE_PORT_FRONTIER_STATE_PATH.to_owned(),
            SOURCE_PORT_RESEARCH_MAP_PATH.to_owned(),
            "docs/source-port/methodology.md".to_owned(),
            SOURCE_PORT_RELEASE_GATE_PATH.to_owned(),
        ],
        implementation_targets: vec![
            SOURCE_PORT_FRONTIER_STATE_PATH.to_owned(),
            "crates/kidcli/src/autonomy.rs".to_owned(),
        ],
        commands: vec![
            "cargo run -p kidcli -- autonomous-loop --goal proactive-verification".to_owned(),
            "mise run verify-slice-fast".to_owned(),
            "cargo run -p kidcli -- asm-coverage --update".to_owned(),
        ],
        done_criteria: vec![
            "Record the exhausted proactive-verification pivot in docs/source-port/frontier-state.yaml so the next package does not ask for another docs-only sweep refresh.".to_owned(),
            "Do not edit only docs/source-port/methodology.md and docs/source-port/research-map.md.".to_owned(),
            "verify-slice-fast passes.".to_owned(),
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
                label: "docs-only-refresh-streak".to_owned(),
                status: "stalled".to_owned(),
                detail: format!(
                    "last {docs_only_refresh_streak} commits touched only sweep-parity evidence docs"
                ),
                artefact_path: Some(PathBuf::from(SOURCE_PORT_RESEARCH_MAP_PATH)),
            },
            EvidenceItem {
                label: "proactive-verification-status".to_owned(),
                status: "complete".to_owned(),
                detail: "autonomous-loop --goal proactive-verification has no actionable task package".to_owned(),
                artefact_path: Some(PathBuf::from("docs/source-port/proactive-verification-config.yaml")),
            },
            EvidenceItem {
                label: "next-agent-note".to_owned(),
                status: note.status.as_str().to_owned(),
                detail: note.detail.clone(),
                artefact_path: Some(note.path.clone()),
            },
        ],
        frontier_context: Some(source_port_frontier_context(frontier)),
        case_queue_context: None,
        delegated_goal: Some(AutonomyGoal::ProactiveVerification),
        delegated_target: Some("all-exhausted".to_owned()),
        delegation_reason: Some(pivot_reason),
        recovery_mode: None,
        recovery_step: None,
        stalled_repetition_count: None,
        pivot_reason: None,
        next_fallback_goal: Some(AutonomyGoal::SweepParity),
        next_agent_note: Some(note.clone()),
        target_routines: None,
        definitively_stalled: false,
    }
}

fn source_port_frontier_recording_task(
    frontier: &ActiveFrontierState,
    note: &NextAgentNoteDirective,
    queue: &CaseQueueSnapshot,
    recording: &RecordingQueueCandidate,
) -> TaskPackage {
    let initial_frame_end = recording.frame_count.min(120);
    let case_path = PathBuf::from(SOURCE_PORT_CASE_CACHE_PATH).join(&recording.slug);
    let delegation_reason = queue.selection_reason.clone().unwrap_or_else(|| {
        format!(
            "the TAS frontier is already closed and no failing reduced case is ready, so the next concrete lane is deriving gameplay work from recording {}",
            recording.slug
        )
    });
    TaskPackage {
        capability: CapabilityId::SourcePortFrontierTask,
        title: format!(
            "Advance source-port via gameplay recording: {}",
            recording.slug
        ),
        summary: format!(
            "{} is authoritative-clean through frame {}; derive a reduced gameplay case from recording {} before any sweep-only hardening.",
            frontier.recording, frontier.authoritative_clean_through, recording.slug
        ),
        generated_at: String::new(),
        cycle: 0,
        progress_fingerprint: String::new(),
        sources: vec![
            SOURCE_PORT_FRONTIER_STATE_PATH.to_owned(),
            recording.manifest_path.display().to_string(),
            SOURCE_PORT_RESEARCH_MAP_PATH.to_owned(),
            "docs/source-port/methodology.md".to_owned(),
        ],
        implementation_targets: vec![
            case_path.display().to_string(),
            SOURCE_PORT_CASE_CACHE_PATH.to_owned(),
        ],
        commands: vec![
            format!("cargo run -p kidcli -- verify-recording --slug {}", recording.slug),
            format!(
                "cargo run -p kidcli -- case derive --recording-slug {} --frame-start 0 --frame-end {} --out {}",
                recording.slug,
                initial_frame_end,
                case_path.display()
            ),
            format!("cargo run -p kidcli -- case verify --case {}", recording.slug),
            format!("cargo run -p kidcli -- case reduce --case {}", recording.slug),
            format!("cargo run -p kidcli -- case summarise --case {}", recording.slug),
        ],
        done_criteria: vec![
            format!(
                "Convert recording {} into a reduced gameplay case or record an explicit blocked result with fresh evidence.",
                recording.slug
            ),
            "Prefer a failing authoritative window of 120 frames or fewer before reopening broad verification lanes."
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
                artefact_path: Some(recording.manifest_path.clone()),
            },
            EvidenceItem {
                label: "selected-recording".to_owned(),
                status: "derive-case".to_owned(),
                detail: format!(
                    "recording={}, frame_count={}, transcript={}, notes={}",
                    recording.slug, recording.frame_count, recording.has_transcript, recording.has_notes
                ),
                artefact_path: Some(recording.manifest_path.clone()),
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
            selected_case: None,
            selected_recording: Some(recording.clone()),
            open_cases: queue.open_cases.clone(),
            blocked_cases: queue.blocked_cases.clone(),
            untriaged_recordings: queue.untriaged_recordings.clone(),
        }),
        delegated_goal: Some(AutonomyGoal::CaseQueue),
        delegated_target: Some(recording.slug.clone()),
        delegation_reason: Some(delegation_reason),
        recovery_mode: None,
        recovery_step: None,
        stalled_repetition_count: None,
        pivot_reason: None,
        next_fallback_goal: Some(AutonomyGoal::SweepParity),
        next_agent_note: Some(note.clone()),
        target_routines: None,
        definitively_stalled: false,
    }
}

fn classify_backlog_issue_focus(system: &str, summary: &str) -> BacklogIssueFocus {
    let system = system.to_ascii_lowercase();
    let summary = summary.to_ascii_lowercase();
    if matches!(system.as_str(), "render" | "menu")
        || summary.contains("graphic")
        || summary.contains("animation")
        || summary.contains("sprite")
        || summary.contains("background")
        || summary.contains("render")
        || summary.contains("intro")
    {
        BacklogIssueFocus::VisualParity
    } else {
        BacklogIssueFocus::Playability
    }
}

fn load_open_backlog_issues(repo_root: &Path) -> Result<Vec<BacklogIssueCandidate>> {
    let backlog_path = repo_root.join(SOURCE_PORT_BACKLOG_PATH);
    let contents = match fs::read_to_string(&backlog_path) {
        Ok(contents) => contents,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(error) => {
            return Err(error)
                .with_context(|| format!("failed to read {}", backlog_path.display()));
        }
    };
    let mut items = Vec::new();
    for line in contents.lines() {
        if !line.starts_with('|') || line.contains("| Status |") || line.contains("|-----") {
            continue;
        }
        let columns = line
            .split('|')
            .map(str::trim)
            .filter(|column| !column.is_empty())
            .collect::<Vec<_>>();
        if columns.len() < 5 {
            continue;
        }
        let status = columns[1];
        let priority = columns[2];
        if !matches!(status, "[ ]" | "[~]") || !matches!(priority, "P0" | "P1" | "P2") {
            continue;
        }
        let system = columns[3].to_owned();
        let summary = columns[4].to_owned();
        let verify = columns.get(5).copied().unwrap_or_default().to_owned();
        items.push(BacklogIssueCandidate {
            id: columns[0].to_owned(),
            priority: priority.to_owned(),
            focus: classify_backlog_issue_focus(&system, &summary),
            system,
            summary,
            verify,
            status: status.to_owned(),
        });
    }
    Ok(items)
}

#[cfg(test)]
fn select_open_backlog_issue(repo_root: &Path) -> Result<Option<BacklogIssueCandidate>> {
    let exhausted_targets = source_port_recent_clean_completed_targets(repo_root);
    select_open_backlog_issue_excluding(repo_root, &exhausted_targets)
}

fn select_open_backlog_issue_excluding(
    repo_root: &Path,
    exhausted_targets: &BTreeSet<String>,
) -> Result<Option<BacklogIssueCandidate>> {
    let mut items = load_open_backlog_issues(repo_root)?;
    items.retain(|issue| !backlog_issue_target_exhausted(issue, exhausted_targets));
    items.sort_by(|left, right| {
        fn priority_rank(priority: &str) -> u8 {
            match priority {
                "P0" => 0,
                "P1" => 1,
                "P2" => 2,
                _ => 3,
            }
        }
        fn focus_rank(focus: BacklogIssueFocus) -> u8 {
            match focus {
                BacklogIssueFocus::Playability => 0,
                BacklogIssueFocus::VisualParity => 1,
            }
        }
        fn preferred_issue_rank(issue_id: &str) -> usize {
            HIGH_VALUE_BACKLOG_ISSUE_ORDER
                .iter()
                .position(|candidate| *candidate == issue_id)
                .unwrap_or(HIGH_VALUE_BACKLOG_ISSUE_ORDER.len())
        }
        fn evidence_rank(issue: &BacklogIssueCandidate) -> u8 {
            let verify = issue.verify.to_ascii_lowercase();
            if verify.contains("native repro status: not reproduced") {
                1
            } else {
                0
            }
        }
        priority_rank(&left.priority)
            .cmp(&priority_rank(&right.priority))
            .then_with(|| evidence_rank(left).cmp(&evidence_rank(right)))
            .then_with(|| preferred_issue_rank(&left.id).cmp(&preferred_issue_rank(&right.id)))
            .then_with(|| focus_rank(left.focus).cmp(&focus_rank(right.focus)))
            .then_with(|| left.id.cmp(&right.id))
    });
    Ok(items.into_iter().next())
}

fn open_backlog_targets_exhausted_by_recent_clean_completion(
    repo_root: &Path,
    exhausted_targets: &BTreeSet<String>,
) -> Result<bool> {
    let items = load_open_backlog_issues(repo_root)?;
    Ok(!items.is_empty()
        && items
            .iter()
            .all(|issue| backlog_issue_target_exhausted(issue, exhausted_targets)))
}

fn backlog_issue_phase(issue: &BacklogIssueCandidate) -> BacklogIssuePhase {
    let verify = issue.verify.to_ascii_lowercase();
    if !verify.contains("not reproduced") && verify.contains("repro status: reproduced") {
        BacklogIssuePhase::InvestigateReproduced
    } else {
        BacklogIssuePhase::CaptureEvidence
    }
}

fn backlog_issue_progress_marker(issue: &BacklogIssueCandidate) -> String {
    format!("{}:{}", issue.id, backlog_issue_phase(issue).slug())
}

fn backlog_issue_target_exhausted(
    issue: &BacklogIssueCandidate,
    exhausted_targets: &BTreeSet<String>,
) -> bool {
    if exhausted_targets.contains(&backlog_issue_progress_marker(issue)) {
        return true;
    }
    matches!(
        backlog_issue_phase(issue),
        BacklogIssuePhase::CaptureEvidence
    ) && exhausted_targets.contains(&issue.id)
}

fn is_ready_backlog_issue(issue: &BacklogIssueCandidate) -> bool {
    let summary = issue.summary.to_ascii_lowercase();
    let verify = issue.verify.to_ascii_lowercase();
    summary.contains("root cause identified") || verify.contains("root cause identified")
}

fn ready_backlog_issue_progress_marker(issue: &BacklogIssueCandidate) -> String {
    format!("{}:ready", issue.id)
}

fn select_ready_backlog_issue_excluding(
    repo_root: &Path,
    exhausted_targets: &BTreeSet<String>,
) -> Result<Option<BacklogIssueCandidate>> {
    let mut items = load_open_backlog_issues(repo_root)?;
    items.retain(is_ready_backlog_issue);
    items.retain(|issue| !exhausted_targets.contains(&ready_backlog_issue_progress_marker(issue)));
    items.sort_by(|left, right| {
        fn priority_rank(priority: &str) -> u8 {
            match priority {
                "P0" => 0,
                "P1" => 1,
                "P2" => 2,
                _ => 3,
            }
        }
        priority_rank(&left.priority)
            .cmp(&priority_rank(&right.priority))
            .then_with(|| left.id.cmp(&right.id))
    });
    Ok(items.into_iter().next())
}

fn source_port_frontier_ready_backlog_issue_task(
    frontier: &ActiveFrontierState,
    note: &NextAgentNoteDirective,
    issue: &BacklogIssueCandidate,
) -> TaskPackage {
    let impl_target = match issue.system.as_str() {
        "enemy" => "crates/kid-core/src/enemies.rs",
        "block" | "platform" => "crates/kid-core/src/blocks.rs",
        "player" | "camera" | "form" => "crates/kid-core/src/lib.rs",
        "render" | "level" => "crates/kid-core/src/level.rs",
        _ => "crates/kid-core/src/lib.rs",
    };
    let mut task = source_port_frontier_backlog_issue_task(frontier, note, issue);
    task.title = format!("Implement ready backlog issue #{}", issue.id);
    task.summary = format!(
        "{} is authoritative-clean through frame {}; backlog issue #{} already has root-cause evidence, so implement it before fallback coverage work.",
        frontier.recording, frontier.authoritative_clean_through, issue.id
    );
    task.implementation_targets = vec![impl_target.to_owned(), SOURCE_PORT_BACKLOG_PATH.to_owned()];
    task.commands = vec![
        "mise run verify-slice-fast".to_owned(),
        "cargo run -p kidcli -- asm-coverage --update".to_owned(),
    ];
    task.done_criteria = vec![
        format!("Backlog issue #{} marked [x].", issue.id),
        "Targeted regression test added for the repaired behaviour.".to_owned(),
        "verify-slice-fast passes.".to_owned(),
    ];
    task.delegated_goal = Some(AutonomyGoal::ProactiveVerification);
    task.delegated_target = Some(ready_backlog_issue_progress_marker(issue));
    task.delegation_reason = Some(
        "root-cause backlog items outrank proactive loc_ coverage once TAS is closed".to_owned(),
    );
    task
}

fn bug_investigation_progress_marker(issue: &BacklogIssueCandidate) -> String {
    format!("{}:investigate-bug", issue.id)
}

fn is_bug_investigation_candidate(issue: &BacklogIssueCandidate) -> bool {
    // Ready items (root cause identified) belong to the ready-implement lane;
    // investigation is for bugs that still need reproduction + tracing.
    if is_ready_backlog_issue(issue) {
        return false;
    }
    let verify = issue.verify.to_ascii_lowercase();
    let summary = issue.summary.to_ascii_lowercase();
    // Skip items tagged `blocked:*` or `stalled:*` — those are for humans.
    !(verify.contains("blocked:")
        || verify.contains("stalled:")
        || summary.contains("blocked:")
        || summary.contains("stalled:"))
}

fn select_open_bug_for_investigation(
    repo_root: &Path,
    exhausted_targets: &BTreeSet<String>,
    attempts: &BTreeMap<String, u32>,
    attempt_cap: u32,
) -> Result<Option<BacklogIssueCandidate>> {
    let mut items = load_open_backlog_issues(repo_root)?;
    items.retain(is_bug_investigation_candidate);
    items.retain(|issue| !exhausted_targets.contains(&bug_investigation_progress_marker(issue)));
    items.retain(|issue| attempts.get(&issue.id).copied().unwrap_or(0) < attempt_cap);
    items.sort_by(|left, right| {
        fn priority_rank(priority: &str) -> u8 {
            match priority {
                "P0" => 0,
                "P1" => 1,
                "P2" => 2,
                _ => 3,
            }
        }
        priority_rank(&left.priority)
            .cmp(&priority_rank(&right.priority))
            .then_with(|| left.id.cmp(&right.id))
    });
    Ok(items.into_iter().next())
}

fn source_port_frontier_bug_investigation_task(
    frontier: &ActiveFrontierState,
    note: &NextAgentNoteDirective,
    issue: &BacklogIssueCandidate,
) -> TaskPackage {
    let impl_target = match issue.system.as_str() {
        "enemy" => "crates/kid-core/src/enemies.rs",
        "block" | "platform" => "crates/kid-core/src/blocks.rs",
        "player" | "camera" | "form" => "crates/kid-core/src/lib.rs",
        "render" | "level" => "crates/kid-core/src/level.rs",
        _ => "crates/kid-core/src/lib.rs",
    };
    let issue_slug = format!("issue-{}", issue.id);
    let level_hint = if issue.summary.to_ascii_lowercase().contains("2nd level") {
        "01"
    } else {
        "00"
    };
    let focus_label = match issue.focus {
        BacklogIssueFocus::Playability => "playability",
        BacklogIssueFocus::VisualParity => "visual-parity",
    };
    let title = format!(
        "Investigate open backlog issue #{} ({})",
        issue.id, focus_label
    );
    let summary = format!(
        "{} is authoritative-clean through frame {}; drive gameplay-bug investigation on backlog issue #{}: {}. Reproduce, trace ASM + oracle, propose a fix, mark the backlog item [?] — never [x].",
        frontier.recording, frontier.authoritative_clean_through, issue.id, issue.summary
    );
    let delegation_reason = format!(
        "gate green and proactive verification exhausted; backlog issue #{} is the highest-priority open bug without root cause, so investigate it instead of meta-hardening",
        issue.id
    );
    TaskPackage {
        capability: CapabilityId::SourcePortFrontierTask,
        title,
        summary,
        generated_at: String::new(),
        cycle: 0,
        progress_fingerprint: String::new(),
        sources: vec![
            SOURCE_PORT_BACKLOG_PATH.to_owned(),
            SOURCE_PORT_FRONTIER_STATE_PATH.to_owned(),
            SOURCE_PORT_RESEARCH_MAP_PATH.to_owned(),
            ".agents/skills/m68k-kid-asm/SKILL.md".to_owned(),
            ".cache/upstream/kid-chameleon-disasm/kid.asm".to_owned(),
        ],
        implementation_targets: vec![
            impl_target.to_owned(),
            SOURCE_PORT_BACKLOG_PATH.to_owned(),
        ],
        commands: vec![
            format!(
                "cargo run --release -p kidcli -- play-native --warp-level {} --settled --record {}-repro",
                level_hint, issue_slug
            ),
            format!(
                "cargo run -p kidcli -- verify-recording --slug {}-repro",
                issue_slug
            ),
            format!(
                "cargo run -p kidcli -- capture-frames --level {} --frames 0,60,120 --output .cache/debug/bug-{}/",
                level_hint, issue.id
            ),
            "cargo run -p kidcli -- compare-native-oracle --scenario boot-title-start-level1"
                .to_owned(),
            "mise run verify-slice-fast".to_owned(),
            "cargo run -q -p kidcli -- asm-coverage --update".to_owned(),
        ],
        done_criteria: vec![
            format!(
                "Backlog issue #{} is marked [?] with an evidence block citing repro scenario path, ASM labels, oracle-parity result, and the proposed-fix commit hash.",
                issue.id
            ),
            "Never promote to [x]; that remains a human decision.".to_owned(),
            "If reproduction fails or ASM/oracle remain ambiguous, append the dead end to the rejected-hypotheses ledger, bump `bug_investigation_attempts` in frontier-state.yaml, and leave status at [~].".to_owned(),
            "Commit must touch either `crates/kid-core/src/**` (fix + regression test) or `docs/source-port/backlog.md` (evidence / status transition); cache-only churn does not count as progress.".to_owned(),
            "verify-slice-fast stays green after any fix.".to_owned(),
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
                label: "bug-investigation".to_owned(),
                status: issue.status.clone(),
                detail: format!(
                    "#{} {} {} {}: {}",
                    issue.id, issue.priority, focus_label, issue.system, issue.summary
                ),
                artefact_path: Some(PathBuf::from(SOURCE_PORT_BACKLOG_PATH)),
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
        delegated_goal: Some(AutonomyGoal::GameplayBugInvestigation),
        delegated_target: Some(bug_investigation_progress_marker(issue)),
        delegation_reason: Some(delegation_reason),
        recovery_mode: None,
        recovery_step: None,
        stalled_repetition_count: None,
        pivot_reason: None,
        next_fallback_goal: Some(AutonomyGoal::SweepParity),
        next_agent_note: Some(note.clone()),
        target_routines: None,
        definitively_stalled: false,
    }
}

fn select_release_gate_blocker(repo_root: &Path) -> Option<ReleaseGateFailureCandidate> {
    #[derive(Deserialize)]
    struct GateReport {
        #[serde(default)]
        passed: bool,
        #[serde(default)]
        steps: Vec<GateStep>,
    }
    #[derive(Deserialize)]
    struct GateStep {
        name: String,
        status: String,
        detail: String,
    }

    for target in ["1p-playable", "1p-accurate"] {
        let path = repo_root
            .join(".cache/release-gate")
            .join(target)
            .join("latest.json");
        let Ok(content) = fs::read_to_string(&path) else {
            continue;
        };
        let Ok(report): Result<GateReport, _> = serde_json::from_str(&content) else {
            continue;
        };
        if report.passed {
            continue;
        }
        if let Some(step) = report
            .steps
            .into_iter()
            .find(|step| step.status.eq_ignore_ascii_case("failed"))
        {
            return Some(ReleaseGateFailureCandidate {
                target: target.to_owned(),
                step_name: step.name,
                detail: step.detail,
            });
        }
    }

    None
}

fn source_port_frontier_release_gate_task(
    repo_root: &Path,
    frontier: &ActiveFrontierState,
    note: &NextAgentNoteDirective,
    gate_failure: &ReleaseGateFailureCandidate,
) -> TaskPackage {
    let gate_command = if gate_failure.target == "1p-playable" {
        "mise run verify-1p-playable"
    } else {
        "mise run release-gate"
    };
    let release_gate_latest = latest_release_gate_failure_detail(repo_root, &gate_failure.target);
    let release_gate_focus = release_gate_focus_evidence(repo_root, &gate_failure.target);
    let (failure_detail, failure_artefact) = release_gate_latest.clone().unwrap_or_else(|| {
        (
            format!("{}: {}", gate_failure.step_name, gate_failure.detail),
            PathBuf::from(format!(
                ".cache/release-gate/{}/latest.json",
                gate_failure.target
            )),
        )
    });
    let mut evidence = vec![
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
            label: "release-gate-failure".to_owned(),
            status: "failed".to_owned(),
            detail: failure_detail,
            artefact_path: Some(failure_artefact),
        },
    ];
    if let Some((detail, artefact_path)) = release_gate_focus {
        evidence.push(EvidenceItem {
            label: "release-gate-focus".to_owned(),
            status: "correlated".to_owned(),
            detail,
            artefact_path: Some(artefact_path),
        });
    }

    TaskPackage {
        capability: CapabilityId::SourcePortFrontierTask,
        title: format!(
            "Repair failing release-gate step {} ({})",
            gate_failure.step_name, gate_failure.target
        ),
        summary: format!(
            "{} is authoritative-clean through frame {}; fix the live {} blocker before proactive coverage work.",
            frontier.recording, frontier.authoritative_clean_through, gate_failure.target
        ),
        generated_at: String::new(),
        cycle: 0,
        progress_fingerprint: String::new(),
        sources: vec![
            SOURCE_PORT_RELEASE_GATE_PATH.to_owned(),
            SOURCE_PORT_BACKLOG_PATH.to_owned(),
            "docs/source-port/research-map.md".to_owned(),
        ],
        implementation_targets: vec!["crates/kid-core/src/".to_owned()],
        commands: vec![
            gate_command.to_owned(),
            "mise run verify-slice-fast".to_owned(),
            "mise run project-status".to_owned(),
        ],
        done_criteria: vec![
            format!(
                "The failing gate step {} is green or replaced by a later blocker with fresher evidence.",
                gate_failure.step_name
            ),
            "verify-slice-fast passes.".to_owned(),
            "project-status no longer leads with the same release-gate blocker.".to_owned(),
        ],
        evidence,
        frontier_context: Some(source_port_frontier_context(frontier)),
        case_queue_context: None,
        delegated_goal: Some(AutonomyGoal::SweepParity),
        delegated_target: Some(gate_failure.step_name.clone()),
        delegation_reason: Some(
            "failing release-gate blockers outrank proactive verification once replay parity is closed"
                .to_owned(),
        ),
        recovery_mode: None,
        recovery_step: None,
        stalled_repetition_count: None,
        pivot_reason: None,
        next_fallback_goal: Some(AutonomyGoal::ProactiveVerification),
        next_agent_note: Some(note.clone()),
        target_routines: None,
        definitively_stalled: false,
    }
}

fn source_port_frontier_backlog_evidence_blocker_task(
    frontier: &ActiveFrontierState,
    note: &NextAgentNoteDirective,
    exhausted_targets: &BTreeSet<String>,
) -> TaskPackage {
    let exhausted_detail = exhausted_targets
        .iter()
        .filter(|target| {
            target
                .chars()
                .next()
                .is_some_and(|first| first.is_ascii_digit())
        })
        .cloned()
        .collect::<Vec<_>>()
        .join(", ");
    let detail = if exhausted_detail.is_empty() {
        "open backlog issue targets were recently completed cleanly but remain open".to_owned()
    } else {
        format!("recent clean completions exhausted open backlog issue targets: {exhausted_detail}")
    };
    let delegation_reason = "open backlog issue packages were recently clean-completed without closing or changing phase, so the loop must repair the shared evidence blocker instead of reselecting the same issue".to_owned();

    TaskPackage {
        capability: CapabilityId::SourcePortFrontierTask,
        title: "Repair backlog evidence blocker: boot-title-start-level1".to_owned(),
        summary: format!(
            "{} is authoritative-clean through frame {}; backlog issue selection is exhausted by recent clean completions, so narrow or fix the shared boot-title-start-level1 evidence blocker before another issue package is selected.",
            frontier.recording, frontier.authoritative_clean_through
        ),
        generated_at: String::new(),
        cycle: 0,
        progress_fingerprint: String::new(),
        sources: vec![
            SOURCE_PORT_BACKLOG_PATH.to_owned(),
            "docs/source-port/methodology.md".to_owned(),
            ".agents/skills/m68k-kid-asm/SKILL.md".to_owned(),
            ".cache/upstream/kid-chameleon-disasm/kid.asm".to_owned(),
        ],
        implementation_targets: vec![
            "crates/kid-core/src/".to_owned(),
            "crates/kidcli/src/".to_owned(),
            SOURCE_PORT_BACKLOG_PATH.to_owned(),
        ],
        commands: vec![
            "cargo run -p kidcli -- compare-native-oracle --scenario boot-title-start-level1"
                .to_owned(),
            "cargo run -p kidcli -- verify-recording --slug issue-141".to_owned(),
            "cargo run -p kidcli -- capture-frames --level 00 --frames 0,60,120 --output .cache/debug/backlog-evidence-blocker/"
                .to_owned(),
            "mise run verify-slice-fast".to_owned(),
        ],
        done_criteria: vec![
            "The boot-title-start-level1 evidence path is green, or the first authoritative blocker is narrowed with concrete native/oracle artefacts.".to_owned(),
            "Any gameplay/render fix cites the ASM label or asset/source path that justifies it.".to_owned(),
            "A targeted regression or backlog note prevents the selector from choosing the same exhausted issue package again without new evidence.".to_owned(),
            "verify-slice-fast passes for code changes.".to_owned(),
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
                label: "recent-clean-completions".to_owned(),
                status: "exhausted".to_owned(),
                detail,
                artefact_path: Some(PathBuf::from(SOURCE_PORT_FRONTIER_RUN_HISTORY_PATH)),
            },
            EvidenceItem {
                label: "next-agent-note".to_owned(),
                status: note.status.as_str().to_owned(),
                detail: note.detail.clone(),
                artefact_path: Some(note.path.clone()),
            },
        ],
        frontier_context: Some(source_port_frontier_context(frontier)),
        case_queue_context: None,
        delegated_goal: Some(AutonomyGoal::SweepParity),
        delegated_target: Some(SOURCE_PORT_BACKLOG_EVIDENCE_BLOCKER_TARGET.to_owned()),
        delegation_reason: Some(delegation_reason),
        recovery_mode: None,
        recovery_step: None,
        stalled_repetition_count: None,
        pivot_reason: Some(
            "recent clean completions exhausted open backlog issue targets".to_owned(),
        ),
        next_fallback_goal: Some(AutonomyGoal::ProactiveVerification),
        next_agent_note: Some(note.clone()),
        target_routines: None,
        definitively_stalled: false,
    }
}

fn source_port_frontier_backlog_issue_task(
    frontier: &ActiveFrontierState,
    note: &NextAgentNoteDirective,
    issue: &BacklogIssueCandidate,
) -> TaskPackage {
    let issue_slug = format!("issue-{}", issue.id);
    let phase = backlog_issue_phase(issue);
    let focus_label = match issue.focus {
        BacklogIssueFocus::Playability => "playability",
        BacklogIssueFocus::VisualParity => "visual-parity",
    };
    let level_hint = if issue.summary.to_ascii_lowercase().contains("2nd level") {
        "01"
    } else {
        "00"
    };
    let (title, summary, sources, implementation_targets, commands, done_criteria, reason) =
        match phase {
            BacklogIssuePhase::CaptureEvidence => (
                format!(
                    "Capture canonical evidence for backlog issue #{} ({})",
                    issue.id, focus_label
                ),
                format!(
                    "{} is authoritative-clean through frame {}; capture a reproducible {} bundle for backlog issue #{}: {}.",
                    frontier.recording,
                    frontier.authoritative_clean_through,
                    focus_label,
                    issue.id,
                    issue.summary
                ),
                vec![
                    SOURCE_PORT_BACKLOG_PATH.to_owned(),
                    SOURCE_PORT_RESEARCH_MAP_PATH.to_owned(),
                ],
                vec![SOURCE_PORT_BACKLOG_PATH.to_owned()],
                vec![
                    format!("cargo run -p kidcli -- play-native --record {}", issue_slug),
                    format!("cargo run -p kidcli -- verify-recording --slug {}", issue_slug),
                    format!(
                        "cargo run -p kidcli -- capture-frames --level {} --frames 0,60,120 --output .cache/debug/{}/",
                        level_hint, issue_slug
                    ),
                    "cargo run -p kidcli -- compare-native-oracle --scenario boot-title-start-level1"
                        .to_owned(),
                ],
                vec![
                    format!(
                        "Backlog issue #{} cites a recording slug or explicit frame-capture artefact path.",
                        issue.id
                    ),
                    "The repro is classified as playability or visual-parity with concrete next verification commands.".to_owned(),
                    "If no repro is found, the backlog item records that negative result explicitly instead of staying a vague report.".to_owned(),
                ],
                "the TAS frontier is already closed and there is no reduced case or untriaged recording, so the next highest-value move is to turn a top open backlog report into canonical evidence".to_owned(),
            ),
            BacklogIssuePhase::InvestigateReproduced => (
                format!(
                    "Investigate reproduced backlog issue #{} ({})",
                    issue.id, focus_label
                ),
                format!(
                    "{} is authoritative-clean through frame {}; backlog issue #{} already has reproduced {} evidence, so read ASM/native code and either fix it or narrow the root cause.",
                    frontier.recording,
                    frontier.authoritative_clean_through,
                    issue.id,
                    focus_label
                ),
                vec![
                    SOURCE_PORT_BACKLOG_PATH.to_owned(),
                    SOURCE_PORT_RESEARCH_MAP_PATH.to_owned(),
                    ".agents/skills/m68k-kid-asm/SKILL.md".to_owned(),
                    ".cache/upstream/kid-chameleon-disasm/kid.asm".to_owned(),
                ],
                vec![
                    SOURCE_PORT_BACKLOG_PATH.to_owned(),
                    "crates/kid-core/src/".to_owned(),
                ],
                vec![
                    format!("cargo run -p kidcli -- verify-recording --slug {}", issue_slug),
                    format!(
                        "cargo run -p kidcli -- capture-frames --level {} --frames 0,60,120 --output .cache/debug/{}/",
                        level_hint, issue_slug
                    ),
                    "cargo run -p kidcli -- compare-native-oracle --scenario boot-title-start-level1"
                        .to_owned(),
                    "mise run verify-slice-fast".to_owned(),
                ],
                vec![
                    format!(
                        "Backlog issue #{} is either fixed and marked [x], or its root cause / blocker is narrowed with concrete artefacts.",
                        issue.id
                    ),
                    "Any gameplay/render fix cites the ASM label or asset/source path that justifies it.".to_owned(),
                    "verify-slice-fast passes for code changes.".to_owned(),
                ],
                "the TAS frontier is closed and this backlog issue already has reproduced evidence, so the next loop should investigate or fix it instead of recapturing the same evidence".to_owned(),
            ),
        };
    TaskPackage {
        capability: CapabilityId::SourcePortFrontierTask,
        title,
        summary,
        generated_at: String::new(),
        cycle: 0,
        progress_fingerprint: String::new(),
        sources,
        implementation_targets,
        commands,
        done_criteria,
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
                label: "backlog-issue".to_owned(),
                status: issue.status.clone(),
                detail: format!(
                    "#{} {} {} {}: {}",
                    issue.id, issue.priority, focus_label, issue.system, issue.summary
                ),
                artefact_path: Some(PathBuf::from(SOURCE_PORT_BACKLOG_PATH)),
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
        delegated_target: Some(backlog_issue_progress_marker(issue)),
        delegation_reason: Some(reason),
        recovery_mode: None,
        recovery_step: None,
        stalled_repetition_count: None,
        pivot_reason: None,
        next_fallback_goal: Some(AutonomyGoal::GameplayBacklog),
        next_agent_note: Some(note.clone()),
        target_routines: None,
        definitively_stalled: false,
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

fn case_queue_recording_task(
    recording: &RecordingQueueCandidate,
    queue: &CaseQueueSnapshot,
    note: &NextAgentNoteDirective,
) -> TaskPackage {
    let initial_frame_end = recording.frame_count.min(120);
    let case_selector = recording.slug.clone();
    let case_path = PathBuf::from(SOURCE_PORT_CASE_CACHE_PATH).join(&case_selector);
    let delegation_reason = queue.selection_reason.clone().unwrap_or_else(|| {
        format!(
            "no failing reduced case is currently available, so derive the next gameplay case from recording {}",
            recording.slug
        )
    });
    TaskPackage {
        capability: CapabilityId::SourcePortCaseQueueTask,
        title: format!(
            "Derive reduced gameplay case from recording: {}",
            recording.slug
        ),
        summary: format!(
            "No failing reduced case is currently selected; mine recording {} into a reduced authoritative case before any sweep-only work.",
            recording.slug
        ),
        generated_at: String::new(),
        cycle: 0,
        progress_fingerprint: String::new(),
        sources: vec![
            recording.manifest_path.display().to_string(),
            SOURCE_PORT_RESEARCH_MAP_PATH.to_owned(),
            "docs/source-port/methodology.md".to_owned(),
        ],
        implementation_targets: vec![
            case_path.display().to_string(),
            SOURCE_PORT_CASE_CACHE_PATH.to_owned(),
        ],
        commands: vec![
            format!("cargo run -p kidcli -- verify-recording --slug {}", recording.slug),
            format!(
                "cargo run -p kidcli -- case derive --recording-slug {} --frame-start 0 --frame-end {} --out {}",
                recording.slug,
                initial_frame_end,
                case_path.display()
            ),
            format!("cargo run -p kidcli -- case verify --case {}", case_selector),
            format!("cargo run -p kidcli -- case reduce --case {}", case_selector),
            format!("cargo run -p kidcli -- case summarise --case {}", case_selector),
        ],
        done_criteria: vec![
            format!(
                "Turn recording {} into a locally re-checkable reduced gameplay case or capture an explicit blocked result explaining why reduction failed.",
                recording.slug
            ),
            "Prefer an authoritative failing window of 120 frames or fewer; if the first derived window passes, tighten or move the window instead of pivoting to sweep."
                .to_owned(),
            "Leave behind a failing reduced case, or explicit evidence that this recording did not yield one."
                .to_owned(),
        ],
        evidence: vec![
            EvidenceItem {
                label: "selected-recording".to_owned(),
                status: "derive-case".to_owned(),
                detail: format!(
                    "recording={}, frame_count={}, transcript={}, notes={}",
                    recording.slug, recording.frame_count, recording.has_transcript, recording.has_notes
                ),
                artefact_path: Some(recording.manifest_path.clone()),
            },
            EvidenceItem {
                label: "autonomy-gate".to_owned(),
                status: "gameplay-first".to_owned(),
                detail: delegation_reason.clone(),
                artefact_path: Some(recording.manifest_path.clone()),
            },
            EvidenceItem {
                label: "next-agent-note".to_owned(),
                status: note.status.as_str().to_owned(),
                detail: note.detail.clone(),
                artefact_path: Some(note.path.clone()),
            },
        ],
        frontier_context: None,
        case_queue_context: Some(CaseQueueContext {
            selected_case: None,
            selected_recording: Some(recording.clone()),
            open_cases: queue.open_cases.clone(),
            blocked_cases: queue.blocked_cases.clone(),
            untriaged_recordings: queue.untriaged_recordings.clone(),
        }),
        delegated_goal: None,
        delegated_target: None,
        delegation_reason: queue.selection_reason.clone(),
        recovery_mode: None,
        recovery_step: None,
        stalled_repetition_count: None,
        pivot_reason: None,
        next_fallback_goal: Some(AutonomyGoal::SweepParity),
        next_agent_note: Some(note.clone()),
        target_routines: None,
        definitively_stalled: false,
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
    let progress_fingerprint = fingerprint_for_capabilities(&capabilities);
    let selected_task = select_next_task(spec, &capabilities, &signals).map(|task| TaskPackage {
        generated_at: autonomy_timestamp(),
        cycle,
        progress_fingerprint: progress_fingerprint.clone(),
        ..task
    });
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
        progress_fingerprint,
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
        let timestamp = report
            .get("timestamp")
            .and_then(serde_json::Value::as_str)
            .unwrap_or("unknown");
        let report_path = existing_timestamped_release_gate_report_path(&latest_path, timestamp);
        let report_detail = report_path
            .as_ref()
            .map(|path| format!(", report={}", path.display()))
            .unwrap_or_default();
        CheckOutcome::passing(
            format!("latest release gate passed (timestamp {timestamp}{report_detail})"),
            Some(report_path.unwrap_or(latest_path)),
        )
    } else {
        let failed_steps = release_gate_failed_step_details(&report);
        let detail = format_release_gate_failed_step_summary(&failed_steps)
            .unwrap_or_else(|| "latest release gate did not pass".to_owned());
        let detail = correlate_release_gate_failure_with_state_sweep(repo_root, &detail)
            .map(|correlation| format!("{detail}; {correlation}"))
            .unwrap_or(detail);
        CheckOutcome::failing(detail, Some(latest_path))
    }
}

fn existing_timestamped_release_gate_report_path(
    latest_path: &Path,
    timestamp: &str,
) -> Option<PathBuf> {
    timestamped_release_gate_report_path(latest_path, timestamp).filter(|path| path.is_file())
}

fn timestamped_release_gate_report_path(latest_path: &Path, timestamp: &str) -> Option<PathBuf> {
    if timestamp == "unknown" {
        return None;
    }
    latest_path
        .parent()
        .map(|parent| parent.join(timestamp).join("report.json"))
}

fn release_gate_failed_step_details(report: &serde_json::Value) -> Vec<String> {
    report
        .get("steps")
        .and_then(serde_json::Value::as_array)
        .map(|steps| {
            steps
                .iter()
                .filter(|step| {
                    step.get("status")
                        .and_then(serde_json::Value::as_str)
                        .is_some_and(|status| status == "failed")
                })
                .filter_map(|step| {
                    let name = step.get("name")?.as_str()?;
                    let detail = step.get("detail")?.as_str()?;
                    Some(format!("{name}: {detail}"))
                })
                .collect()
        })
        .unwrap_or_default()
}

fn format_release_gate_failed_step_summary(failed_steps: &[String]) -> Option<String> {
    let first_step = failed_steps.first()?;
    let remaining_count = failed_steps.len().saturating_sub(1);
    if remaining_count == 0 {
        return Some(first_step.clone());
    }

    let preview = failed_steps
        .iter()
        .skip(1)
        .take(3)
        .map(|step| {
            step.split_once(": ")
                .map_or(step.as_str(), |(name, _)| name)
                .to_owned()
        })
        .collect::<Vec<_>>();
    let omitted_count = remaining_count.saturating_sub(preview.len());
    let omitted_suffix = if omitted_count == 0 {
        String::new()
    } else {
        format!(", +{omitted_count} more")
    };
    let plural = if remaining_count == 1 { "" } else { "s" };
    Some(format!(
        "{first_step}; plus {remaining_count} more failed step{plural}: {}{omitted_suffix}",
        preview.join(", ")
    ))
}

fn correlate_release_gate_failure_with_state_sweep(
    repo_root: &Path,
    release_gate_detail: &str,
) -> Option<String> {
    let failed_focus = extract_release_gate_failed_focus("1p-accurate", release_gate_detail)?;
    let failed_field = extract_release_gate_failed_field(release_gate_detail)?;
    if let Some((release_gate_focus_summary, report_path, timestamp)) =
        latest_state_sweep_focus_summary(repo_root, &failed_focus, &failed_field)
    {
        let freshness = timestamp
            .as_deref()
            .map(|timestamp| format!("timestamp={timestamp}, "))
            .unwrap_or_default();
        return Some(format!(
            "latest state sweep reports: {release_gate_focus_summary} ({freshness}report={report_path})"
        ));
    }
    let latest = latest_state_sweep_history_entry(repo_root)?;
    let report_path = latest
        .get("report_path")
        .and_then(serde_json::Value::as_str)
        .unwrap_or("unknown");
    let timestamp = latest.get("timestamp").and_then(serde_json::Value::as_str);
    let field_entry = latest
        .get("field_summary")
        .and_then(serde_json::Value::as_array)
        .and_then(|entries| {
            entries.iter().find(|entry| {
                entry
                    .get("field")
                    .and_then(serde_json::Value::as_str)
                    .is_some_and(|field| field == failed_field)
            })
        })?;
    let total_diffs = field_entry
        .get("total_diffs")
        .and_then(serde_json::Value::as_u64)
        .unwrap_or(0);
    let levels = field_entry
        .get("levels")
        .and_then(serde_json::Value::as_u64)
        .unwrap_or(0);
    let categories = format_divergence_summary_value(field_entry.get("divergence_summary"))?;
    let freshness = timestamp
        .map(|timestamp| format!("timestamp={timestamp}, "))
        .unwrap_or_default();
    Some(format!(
        "latest state sweep classifies `{failed_field}` as {categories} across {levels} levels ({freshness}{total_diffs} diffs, report={report_path})"
    ))
}

fn latest_state_sweep_history_entry(repo_root: &Path) -> Option<serde_json::Value> {
    let history_path = repo_root.join(".cache/metrics/sweep-history.jsonl");
    let contents = fs::read_to_string(&history_path).ok()?;
    contents.lines().rev().find_map(|line| {
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
    })
}

#[derive(Debug, Clone)]
struct StateSweepEvidence {
    source_path: PathBuf,
    report_path: String,
    timestamp: Option<String>,
    payload: serde_json::Value,
}

fn latest_state_sweep_report_entry(repo_root: &Path) -> Option<StateSweepEvidence> {
    let sweep_runs_dir = repo_root.join(".cache/sweep-runs");
    let mut latest: Option<StateSweepEvidence> = None;
    for entry in fs::read_dir(&sweep_runs_dir).ok()? {
        let entry = entry.ok()?;
        let path = entry.path();
        if !entry.file_type().ok()?.is_dir() {
            continue;
        }
        let timestamp = path.file_name()?.to_str()?.to_owned();
        let report_path = path.join("report.json");
        let contents = fs::read_to_string(&report_path).ok()?;
        let payload = serde_json::from_str::<serde_json::Value>(&contents).ok()?;
        if !is_state_sweep_payload(&payload) {
            continue;
        }
        let timestamp = payload
            .get("timestamp")
            .and_then(serde_json::Value::as_str)
            .map_or(timestamp, str::to_owned);
        let candidate = StateSweepEvidence {
            source_path: report_path.clone(),
            report_path: report_path.to_string_lossy().into_owned(),
            timestamp: Some(timestamp),
            payload,
        };
        if latest.as_ref().is_none_or(|current| {
            state_sweep_timestamp_key(&candidate) > state_sweep_timestamp_key(current)
        }) {
            latest = Some(candidate);
        }
    }
    latest
}

fn latest_state_sweep_evidence(repo_root: &Path) -> Option<StateSweepEvidence> {
    let history = latest_state_sweep_history_entry(repo_root).map(|payload| {
        let report_path = payload
            .get("report_path")
            .and_then(serde_json::Value::as_str)
            .unwrap_or("unknown")
            .to_owned();
        let source_path = PathBuf::from(&report_path);
        let source_path = if source_path.is_file() {
            source_path
        } else {
            repo_root.join(".cache/metrics/sweep-history.jsonl")
        };
        StateSweepEvidence {
            source_path,
            report_path,
            timestamp: payload
                .get("timestamp")
                .and_then(serde_json::Value::as_str)
                .map(str::to_owned),
            payload,
        }
    });
    let report = latest_state_sweep_report_entry(repo_root);
    match (history, report) {
        (Some(history), Some(report)) => {
            if state_sweep_timestamp_key(&report) >= state_sweep_timestamp_key(&history) {
                Some(report)
            } else {
                Some(history)
            }
        }
        (Some(history), None) => Some(history),
        (None, Some(report)) => Some(report),
        (None, None) => None,
    }
}

fn is_state_sweep_payload(value: &serde_json::Value) -> bool {
    if value
        .get("tier")
        .and_then(serde_json::Value::as_str)
        .is_some_and(|tier| tier == "state")
    {
        return true;
    }
    value
        .get("levels")
        .and_then(serde_json::Value::as_array)
        .and_then(|levels| levels.first())
        .and_then(|level| level.get("tier"))
        .and_then(serde_json::Value::as_str)
        .is_some_and(|tier| tier == "state")
}

fn state_sweep_timestamp_key(entry: &StateSweepEvidence) -> u64 {
    entry
        .timestamp
        .as_deref()
        .and_then(|timestamp| timestamp.parse::<u64>().ok())
        .unwrap_or(0)
}

fn latest_state_sweep_focus_summary(
    repo_root: &Path,
    failed_focus: &ReleaseGateFocusLookup,
    failed_field: &str,
) -> Option<(String, String, Option<String>)> {
    let latest = latest_state_sweep_evidence(repo_root)?;
    let summary = latest
        .payload
        .get("release_gate_focus")
        .and_then(serde_json::Value::as_array)
        .and_then(|entries| {
            entries
                .iter()
                .find(|entry| release_gate_focus_entry_matches(entry, failed_focus))
                .or_else(|| {
                    entries.iter().find(|entry| {
                        release_gate_focus_entry_is_legacy(entry)
                            && entry
                                .get("field")
                                .and_then(serde_json::Value::as_str)
                                .is_some_and(|field| field == failed_field)
                    })
                })
                .and_then(|entry| {
                    entry
                        .get("summary")
                        .and_then(serde_json::Value::as_str)
                        .map(str::to_owned)
                })
        })?;
    Some((summary, latest.report_path, latest.timestamp))
}

fn latest_release_gate_failure_detail(repo_root: &Path, target: &str) -> Option<(String, PathBuf)> {
    let latest_path = repo_root
        .join(".cache/release-gate")
        .join(target)
        .join("latest.json");
    let contents = fs::read_to_string(&latest_path).ok()?;
    let report = serde_json::from_str::<serde_json::Value>(&contents).ok()?;
    if report
        .get("passed")
        .and_then(serde_json::Value::as_bool)
        .unwrap_or(false)
    {
        return None;
    }
    let failed_steps = release_gate_failed_step_details(&report);
    let detail = format_release_gate_failed_step_summary(&failed_steps)?;
    let timestamp = report
        .get("timestamp")
        .and_then(serde_json::Value::as_str)
        .unwrap_or("unknown");
    let report_path = timestamped_release_gate_report_path(&latest_path, timestamp);
    let freshness = report_path.as_ref().map_or_else(
        || format!(" (timestamp {timestamp})"),
        |report_path| format!(" (timestamp {timestamp}, report={})", report_path.display()),
    );
    let artefact_path = report_path
        .filter(|report_path| report_path.is_file())
        .unwrap_or(latest_path);
    Some((format!("{detail}{freshness}"), artefact_path))
}

fn release_gate_focus_evidence(repo_root: &Path, target: &str) -> Option<(String, PathBuf)> {
    let (detail, _) = latest_release_gate_failure_detail(repo_root, target)?;
    let failed_focus = extract_release_gate_failed_focus(target, &detail)?;
    let failed_field = extract_release_gate_failed_field(&detail)?;
    let (summary, report_path, timestamp) =
        latest_state_sweep_focus_summary(repo_root, &failed_focus, &failed_field)?;
    let freshness = timestamp.as_deref().map_or_else(
        || format!(" (report={report_path})"),
        |timestamp| format!(" (timestamp {timestamp}, report={report_path})"),
    );
    Some((format!("{summary}{freshness}"), PathBuf::from(report_path)))
}

fn extract_release_gate_failed_field(detail: &str) -> Option<String> {
    let (_, tail) = detail.split_once(" diverged in ")?;
    let raw_field = tail
        .split(['(', ';'])
        .next()
        .map(str::trim)
        .filter(|field| !field.is_empty())?;
    let mut field = String::new();
    let mut previous_was_separator = false;
    for ch in raw_field.chars() {
        if ch.is_ascii_alphanumeric() {
            field.push(ch.to_ascii_lowercase());
            previous_was_separator = false;
        } else if !previous_was_separator {
            field.push('_');
            previous_was_separator = true;
        }
    }
    let field = field.trim_matches('_').to_owned();
    if field.is_empty() { None } else { Some(field) }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ReleaseGateFocusLookup {
    target: String,
    step_name: String,
    checkpoint: Option<String>,
    field: String,
    native_value: Option<String>,
    oracle_value: Option<String>,
}

fn extract_release_gate_failed_focus(target: &str, detail: &str) -> Option<ReleaseGateFocusLookup> {
    let (step_name, step_detail) = detail.split_once(": ")?;
    let field = extract_release_gate_failed_field(step_detail)?;
    let checkpoint = extract_release_gate_failed_checkpoint(step_detail);
    let (native_value, oracle_value) = extract_release_gate_failed_values(step_detail)
        .map_or((None, None), |(native, oracle)| {
            (Some(native), Some(oracle))
        });
    Some(ReleaseGateFocusLookup {
        target: target.to_owned(),
        step_name: step_name.to_owned(),
        checkpoint,
        field,
        native_value,
        oracle_value,
    })
}

fn extract_release_gate_failed_checkpoint(detail: &str) -> Option<String> {
    let (_, tail) = detail.split_once("state ")?;
    let checkpoint = tail
        .split_once(" diverged in ")
        .map(|(checkpoint, _)| checkpoint.trim())
        .filter(|checkpoint| !checkpoint.is_empty())?;
    Some(checkpoint.to_owned())
}

fn normalise_release_gate_value(raw: &str) -> String {
    let trimmed = raw.trim();
    if let Some(hex) = trimmed.strip_prefix("0x") {
        return format!("0x{}", hex.to_ascii_lowercase());
    }
    if looks_like_fixed_width_hex(trimmed) {
        format!("0x{}", trimmed.to_ascii_lowercase())
    } else {
        trimmed.to_ascii_lowercase()
    }
}

fn looks_like_fixed_width_hex(value: &str) -> bool {
    !value.is_empty()
        && value.chars().all(|ch| ch.is_ascii_hexdigit())
        && (value
            .chars()
            .any(|ch| ch.is_ascii_hexdigit() && !ch.is_ascii_digit())
            || (value.len() > 1 && value.starts_with('0')))
}

fn extract_release_gate_failed_values(detail: &str) -> Option<(String, String)> {
    let (_, tail) = detail.split_once("(native ")?;
    let (native_raw, tail) = tail.split_once(", oracle ")?;
    let (oracle_raw, _) = tail.split_once(')')?;
    Some((
        normalise_release_gate_value(native_raw),
        normalise_release_gate_value(oracle_raw),
    ))
}

fn release_gate_focus_entry_matches(
    entry: &serde_json::Value,
    focus: &ReleaseGateFocusLookup,
) -> bool {
    let field_matches = entry
        .get("field")
        .and_then(serde_json::Value::as_str)
        .is_some_and(|field| field == focus.field);
    if !field_matches {
        return false;
    }

    let target_matches = entry
        .get("target")
        .and_then(serde_json::Value::as_str)
        .is_some_and(|target| target == focus.target);
    let step_matches = entry
        .get("step_name")
        .and_then(serde_json::Value::as_str)
        .is_some_and(|step_name| step_name == focus.step_name);
    let checkpoint_matches = match (
        entry.get("checkpoint").and_then(serde_json::Value::as_str),
        focus.checkpoint.as_deref(),
    ) {
        (Some(entry_checkpoint), Some(focus_checkpoint)) => entry_checkpoint == focus_checkpoint,
        (None, None) => true,
        _ => false,
    };
    let native_matches = match (
        entry
            .get("native_value")
            .and_then(serde_json::Value::as_str),
        focus.native_value.as_deref(),
    ) {
        (Some(entry_native), Some(focus_native)) => {
            normalise_release_gate_value(entry_native) == focus_native
        }
        (None, None) => true,
        _ => false,
    };
    let oracle_matches = match (
        entry
            .get("oracle_value")
            .and_then(serde_json::Value::as_str),
        focus.oracle_value.as_deref(),
    ) {
        (Some(entry_oracle), Some(focus_oracle)) => {
            normalise_release_gate_value(entry_oracle) == focus_oracle
        }
        (None, None) => true,
        _ => false,
    };

    target_matches && step_matches && checkpoint_matches && native_matches && oracle_matches
}

fn release_gate_focus_entry_is_legacy(entry: &serde_json::Value) -> bool {
    [
        "target",
        "step_name",
        "checkpoint",
        "native_value",
        "oracle_value",
    ]
    .iter()
    .all(|key| entry.get(*key).is_none())
}

fn format_divergence_summary_value(summary: Option<&serde_json::Value>) -> Option<String> {
    let summary = summary?;
    let mut parts = Vec::new();
    for (key, label) in [
        ("authoritative", "authoritative"),
        ("advisory", "advisory"),
        ("accumulation", "accumulation"),
        ("rng", "rng"),
        ("oracle_limitation", "oracle_limitations"),
    ] {
        let count = summary
            .get(key)
            .and_then(serde_json::Value::as_u64)
            .unwrap_or(0);
        if count > 0 {
            parts.push(format!("{label}={count}"));
        }
    }
    if parts.is_empty() {
        None
    } else {
        Some(parts.join(", "))
    }
}

fn latest_state_sweep_signal(repo_root: &Path) -> CheckOutcome {
    let history_path = repo_root.join(".cache/metrics/sweep-history.jsonl");
    let latest = latest_state_sweep_evidence(repo_root);
    let Some(latest) = latest else {
        return CheckOutcome::failing(
            "no persisted state-tier sweep artefact found in sweep history or sweep-runs cache",
            Some(history_path),
        );
    };
    let passed = latest
        .payload
        .get("failed")
        .and_then(serde_json::Value::as_u64)
        .unwrap_or(1)
        == 0
        && latest
            .payload
            .get("errors")
            .and_then(serde_json::Value::as_u64)
            .unwrap_or(1)
            == 0;
    let report_path = &latest.report_path;
    let failed = latest
        .payload
        .get("failed")
        .and_then(serde_json::Value::as_u64)
        .unwrap_or(0);
    let errors = latest
        .payload
        .get("errors")
        .and_then(serde_json::Value::as_u64)
        .unwrap_or(0);
    let advisory = latest
        .payload
        .get("divergence_summary")
        .and_then(|summary| summary.get("advisory"))
        .and_then(serde_json::Value::as_u64)
        .unwrap_or(0);
    let oracle_limitations = latest
        .payload
        .get("divergence_summary")
        .and_then(|summary| summary.get("oracle_limitation"))
        .and_then(serde_json::Value::as_u64)
        .unwrap_or(0);
    let timestamp_detail = latest
        .timestamp
        .as_deref()
        .map(|timestamp| format!("timestamp {timestamp}, "))
        .unwrap_or_default();
    if passed {
        CheckOutcome::passing(
            format!(
                "latest state sweep passed ({timestamp_detail}failed=0, errors=0, advisory={advisory}, oracle_limitations={oracle_limitations}, report={report_path})"
            ),
            Some(latest.source_path),
        )
    } else {
        CheckOutcome::failing(
            format!(
                "latest state sweep reported failures ({timestamp_detail}failed={failed}, errors={errors}, report={report_path})"
            ),
            Some(latest.source_path),
        )
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
        CapabilityId::GameplayBacklogTask => (
            false,
            "The gameplay backlog task is generated from asm-coverage.yaml.".to_owned(),
        ),
        CapabilityId::ProactiveVerificationTask => (
            false,
            "The proactive verification task is generated from the strategy cascade.".to_owned(),
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
        summary: capability.summary.clone(),
        generated_at: String::new(),
        cycle: 0,
        progress_fingerprint: String::new(),
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
        recovery_mode: None,
        recovery_step: None,
        stalled_repetition_count: None,
        pivot_reason: None,
        next_fallback_goal: None,
        next_agent_note: None,
        target_routines: None,
        definitively_stalled: false,
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
        CapabilityId::GameplayBacklogTask => Vec::new(),
        CapabilityId::ProactiveVerificationTask => Vec::new(),
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
    let recent_commits = recent_commit_summaries(repo_root, 5);
    fs::write(
        &note_path,
        render_source_port_next_agent_note(frontier, task, &recent_commits),
    )
    .with_context(|| format!("failed to write {}", note_path.display()))
}

fn render_source_port_next_agent_note(
    frontier: &FrontierContext,
    task: &TaskPackage,
    recent_commits: &[RecentCommitSummary],
) -> String {
    let closed_frontier = frontier.status == "complete";
    let recovery_summary = match (
        task.recovery_mode,
        task.recovery_step,
        task.stalled_repetition_count,
        task.pivot_reason.as_deref(),
    ) {
        (Some(mode), Some(step), Some(repetitions), Some(reason)) => Some(format!(
            "Recovery mode `{}` / step `{}` after {} repeated stalled cycles: {}",
            mode.as_str(),
            step.as_str(),
            repetitions,
            reason
        )),
        (Some(mode), Some(step), _, Some(reason)) => Some(format!(
            "Recovery mode `{}` / step `{}`: {}",
            mode.as_str(),
            step.as_str(),
            reason
        )),
        (Some(mode), Some(step), Some(repetitions), None) => Some(format!(
            "Recovery mode `{}` / step `{}` after {} repeated stalled cycles.",
            mode.as_str(),
            step.as_str(),
            repetitions
        )),
        (Some(mode), Some(step), None, None) => Some(format!(
            "Recovery mode `{}` / step `{}`.",
            mode.as_str(),
            step.as_str()
        )),
        _ => None,
    };
    let release_gate_failure = task
        .evidence
        .iter()
        .find(|item| item.label == "release-gate-failure")
        .map(|item| item.detail.as_str());
    let release_gate_focus = task
        .evidence
        .iter()
        .find(|item| item.label == "release-gate-focus")
        .map(|item| item.detail.as_str());
    let release_gate_status = task
        .evidence
        .iter()
        .find(|item| item.label == "release-gate-status")
        .map(|item| item.detail.as_str());
    let state_sweep_status = task
        .evidence
        .iter()
        .find(|item| item.label == "state-sweep-status")
        .map(|item| item.detail.as_str());
    let exact_next_moves = if closed_frontier {
        let mut moves = Vec::new();
        if let Some(summary) = recovery_summary.as_ref() {
            moves.push(summary.clone());
        }
        moves.push(format!(
            "Do not reopen TAS-frontier work; continue through delegated lane `{}`.",
            task.delegated_goal
                .map_or("source-port-frontier", AutonomyGoal::slug)
        ));
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
    let strongest_evidence = {
        let mut evidence = vec![format!(
            "Hypothesis: {}: {}",
            frontier.current_hypothesis.name, frontier.current_hypothesis.reason
        )];
        if let Some(detail) = release_gate_failure {
            evidence.push(format!("Release-gate failure: {detail}"));
        }
        if let Some(detail) = release_gate_focus {
            evidence.push(format!("Latest state-sweep correlation: {detail}"));
        }
        if let Some(detail) = release_gate_status {
            evidence.push(format!("Release-gate status: {detail}"));
        }
        if let Some(detail) = state_sweep_status {
            evidence.push(format!("Latest state-sweep status: {detail}"));
        }
        evidence
            .iter()
            .map(|step| format!("- {step}"))
            .collect::<Vec<_>>()
            .join("\n")
    };
    let recent_streak = sweep_refresh_streak(recent_commits);
    let self_check = if recent_streak >= 3 {
        format!(
            "- STOP: the last {recent_streak} consecutive commits touched only docs/source-port/methodology.md + docs/source-port/research-map.md. Do NOT produce another commit of the same shape. Pivot to proactive-verification (`cargo run -p kidcli -- autonomous-loop --goal proactive-verification`) or record the stall in docs/source-port/frontier-state.yaml before touching those docs again."
        )
    } else {
        "- Read the Recent Commits block below before starting. If the last 3+ consecutive commits touched only docs/source-port/methodology.md and docs/source-port/research-map.md, do NOT produce another commit of the same shape — pivot to proactive-verification or record the stall in docs/source-port/frontier-state.yaml.".to_owned()
    };
    let recent_commits_block = if recent_commits.is_empty() {
        "- (git log unavailable)".to_owned()
    } else {
        recent_commits
            .iter()
            .map(|commit| {
                let files = if commit.files.is_empty() {
                    "(no file paths reported)".to_owned()
                } else {
                    commit.files.join(", ")
                };
                format!("- {} {} — {}", commit.sha, commit.subject, files)
            })
            .collect::<Vec<_>>()
            .join("\n")
    };
    format!(
        "\
---
slug: {slug}
updated_at: {updated_at}
status: {status}
---
# Self-Check Before Starting
{self_check}

# Recent Commits
{recent_commits_block}

# What Changed
- {what_changed}

# Current Model
- Recording: {recording}
- Area: {area}
- Authoritative clean through: frame {clean_through}
- Current blocker: {blocker}

# Strongest Evidence
{strongest_evidence}

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
            let delegated_lane = format!(
                "The TAS frontier is closed through frame {}; the next loop is delegated to {}{}.",
                frontier.authoritative_clean_through,
                task.delegated_goal
                    .map_or("source-port-frontier", AutonomyGoal::slug),
                task.delegated_target
                    .as_deref()
                    .map(|target| format!(" ({target})"))
                    .unwrap_or_default()
            );
            recovery_summary
                .as_ref()
                .map(|summary| format!("{delegated_lane} {summary}"))
                .unwrap_or(delegated_lane)
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
        strongest_evidence = strongest_evidence,
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
        if let Some(mode) = task.recovery_mode {
            println!("recovery mode: {}", mode.as_str());
        }
        if let Some(step) = task.recovery_step {
            println!("recovery step: {}", step.as_str());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{
        CapabilityId, CapabilityStatus, CaseFailureClass, CaseQueueStatus, CheckOutcome, GoalSpec,
        NextAgentNoteStatus, RecentCommitSummary, RepositorySignals, SOURCE_PORT_CASE_CACHE_PATH,
        SOURCE_PORT_CASE_QUEUE_NOTE_PATH, SOURCE_PORT_FRONTIER_NOTE_PATH,
        SOURCE_PORT_FRONTIER_STATE_PATH, SOURCE_PORT_RECORDINGS_CACHE_PATH,
        SOURCE_PORT_RELEASE_GATE_PATH, SOURCE_PORT_RESEARCH_MAP_PATH, SOURCE_PORT_ROADMAP_PATH,
        SOURCE_PORT_WHOLE_GAME_INDEX_PATH, TaskPackage, build_case_queue_snapshot, detect_blocker,
        evaluate_capabilities, evaluate_case_queue_cycle, evaluate_source_port_frontier_cycle,
        fingerprint_for_capabilities, is_sweep_refresh_commit, latest_release_gate_failure_detail,
        latest_release_gate_signal, latest_state_sweep_signal, load_source_port_frontier_state,
        normalise_release_gate_value, parse_recent_commit_summaries,
        render_source_port_next_agent_note, select_next_task, source_port_frontier_sweep_task,
        source_port_next_agent_note, sweep_refresh_streak,
    };
    use std::collections::BTreeMap;
    use std::fs;
    use std::path::{Path, PathBuf};
    use std::process::Command;
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
            generated_at: "0".to_owned(),
            cycle: 0,
            progress_fingerprint: "test".to_owned(),
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
            recovery_mode: None,
            recovery_step: None,
            stalled_repetition_count: None,
            pivot_reason: None,
            next_fallback_goal: None,
            next_agent_note: None,
            target_routines: None,
            definitively_stalled: false,
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

    #[test]
    fn selected_task_summary_uses_live_capability_status() {
        let spec: GoalSpec = serde_json::from_str(
            r#"
            {
              "goal_id": "sweep-parity",
              "title": "Whole-game sweep parity",
              "summary": "Test goal",
              "capabilities": [
                {
                  "id": "release_gate_green",
                  "title": "Release gate green",
                  "summary": "Static summary that should not leak into the selected task",
                  "prerequisites": [],
                  "sources": ["src/main.rs"],
                  "implementation_targets": ["src/main.rs"],
                  "commands": ["mise run release-gate"],
                  "done_criteria": ["release gate passes"]
                }
              ]
            }
            "#,
        )
        .expect("goal spec should parse");
        let mut signals = base_signals();
        signals.release_gate = CheckOutcome::failing(
            "frame_oracle:boot_title_start_level1: state title@360 diverged in game mode",
            None,
        );

        let capabilities = evaluate_capabilities(&spec, Path::new("."), &signals);
        let task = select_next_task(&spec, &capabilities, &signals).expect("task should exist");

        assert_eq!(task.capability, CapabilityId::ReleaseGateGreen);
        assert!(
            task.summary
                .contains("frame_oracle:boot_title_start_level1"),
            "task summary should describe the live failing evidence"
        );
        assert!(
            !task
                .summary
                .contains("Static summary that should not leak into the selected task"),
            "task summary should not reuse static goal-spec prose"
        );
    }

    #[test]
    fn release_gate_signal_reports_failed_step_detail() {
        let temp_dir = TempDir::new().expect("temp dir should exist");
        let gate_dir = temp_dir.path().join(".cache/release-gate/1p-accurate");
        fs::create_dir_all(&gate_dir).expect("gate dir should exist");
        fs::write(
            gate_dir.join("latest.json"),
            r#"
            {
              "timestamp": "1775119879",
              "target": "1p-accurate",
              "passed": false,
              "steps": [
                {
                  "name": "frame_oracle:boot_title_start_level1",
                  "status": "failed",
                  "detail": "state title@360 diverged in game mode (native 000C, oracle 002C)"
                }
              ]
            }
            "#,
        )
        .expect("gate report should write");

        let outcome = latest_release_gate_signal(temp_dir.path());
        assert!(!outcome.ok);
        assert!(
            outcome
                .detail
                .contains("frame_oracle:boot_title_start_level1"),
            "signal should include the failed step name"
        );
        assert!(
            outcome.detail.contains("title@360"),
            "signal should include the failed step detail"
        );
    }

    #[test]
    fn release_gate_signal_uses_latest_json_when_timestamped_report_is_missing() {
        let temp_dir = TempDir::new().expect("temp dir should exist");
        let gate_dir = temp_dir.path().join(".cache/release-gate/1p-accurate");
        fs::create_dir_all(&gate_dir).expect("gate dir should exist");
        let latest_path = gate_dir.join("latest.json");
        fs::write(
            &latest_path,
            r#"
            {
              "timestamp": "1776704737",
              "target": "1p-accurate",
              "passed": true,
              "steps": []
            }
            "#,
        )
        .expect("gate report should write");

        let outcome = latest_release_gate_signal(temp_dir.path());
        assert!(outcome.ok);
        assert!(
            outcome.detail.contains("timestamp 1776704737"),
            "signal should still preserve the release-gate timestamp"
        );
        assert!(
            !outcome.detail.contains("1776704737/report.json"),
            "signal should not cite a missing timestamped report"
        );
        assert_eq!(
            outcome.artefact_path.as_deref(),
            Some(latest_path.as_path())
        );
    }

    #[test]
    fn release_gate_value_normalisation_preserves_decimal_counts() {
        assert_eq!(normalise_release_gate_value("000C"), "0x000c");
        assert_eq!(normalise_release_gate_value("0x002C"), "0x002c");
        assert_eq!(normalise_release_gate_value("3"), "3");
        assert_eq!(normalise_release_gate_value("120"), "120");
    }

    #[test]
    fn release_gate_failure_evidence_prefers_timestamped_report_when_present() {
        let temp_dir = TempDir::new().expect("temp dir should exist");
        let gate_dir = temp_dir.path().join(".cache/release-gate/1p-accurate");
        let report_dir = gate_dir.join("1775119879");
        fs::create_dir_all(&report_dir).expect("timestamped gate dir should exist");
        fs::write(
            gate_dir.join("latest.json"),
            r#"
            {
              "timestamp": "1775119879",
              "target": "1p-accurate",
              "passed": false,
              "steps": [
                {
                  "name": "frame_oracle:boot_title_start_level1",
                  "status": "failed",
                  "detail": "state title@360 diverged in game mode (native 000C, oracle 002C)"
                }
              ]
            }
            "#,
        )
        .expect("latest gate report should write");
        fs::write(report_dir.join("report.json"), "{}\n")
            .expect("timestamped gate report should write");

        let (detail, artefact_path) =
            latest_release_gate_failure_detail(temp_dir.path(), "1p-accurate")
                .expect("failed gate detail should load");

        assert!(
            detail.contains("frame_oracle:boot_title_start_level1") && detail.contains("report="),
            "failure detail should still name the failed step and timestamped report"
        );
        assert_eq!(
            artefact_path,
            report_dir.join("report.json"),
            "failure evidence should anchor to the immutable timestamped gate report"
        );
    }

    #[test]
    fn release_gate_failure_evidence_mentions_additional_failed_steps() {
        let temp_dir = TempDir::new().expect("temp dir should exist");
        let gate_dir = temp_dir.path().join(".cache/release-gate/1p-accurate");
        fs::create_dir_all(&gate_dir).expect("gate dir should exist");
        fs::write(
            gate_dir.join("latest.json"),
            r#"
            {
              "timestamp": "1775119880",
              "target": "1p-accurate",
              "passed": false,
              "steps": [
                {
                  "name": "gameplay_compare:pause_resume_level1",
                  "status": "failed",
                  "detail": "gameplay state level_settle@719 diverged in game mode (native 0008, oracle 000C)"
                },
                {
                  "name": "frame_oracle:boot_title_start_level1",
                  "status": "failed",
                  "detail": "state title@360 diverged in camera y pos (native 0000, oracle 0010)"
                }
              ]
            }
            "#,
        )
        .expect("gate report should write");

        let outcome = latest_release_gate_signal(temp_dir.path());
        assert!(!outcome.ok);
        assert!(
            outcome
                .detail
                .contains("gameplay_compare:pause_resume_level1"),
            "signal should keep the first failed step as the correlation anchor"
        );
        assert!(
            outcome
                .detail
                .contains("plus 1 more failed step: frame_oracle:boot_title_start_level1"),
            "signal should not hide additional failed release-gate steps"
        );

        let (detail, _) = latest_release_gate_failure_detail(temp_dir.path(), "1p-accurate")
            .expect("failure detail should load");
        assert!(
            detail.contains("plus 1 more failed step: frame_oracle:boot_title_start_level1"),
            "source-port handoff evidence should carry the extra failed step count"
        );
    }

    #[test]
    fn release_gate_signal_correlates_failed_field_with_state_sweep_summary() {
        let temp_dir = TempDir::new().expect("temp dir should exist");
        let gate_dir = temp_dir.path().join(".cache/release-gate/1p-accurate");
        let metrics_dir = temp_dir.path().join(".cache/metrics");
        fs::create_dir_all(&gate_dir).expect("gate dir should exist");
        fs::create_dir_all(&metrics_dir).expect("metrics dir should exist");
        fs::write(
            gate_dir.join("latest.json"),
            r#"
            {
              "timestamp": "1775168489",
              "target": "1p-accurate",
              "passed": false,
              "steps": [
                {
                  "name": "frame_oracle:boot_title_start_level1",
                  "status": "failed",
                  "detail": "state title@360 diverged in game mode (native 000C, oracle 002C)"
                }
              ]
            }
            "#,
        )
        .expect("gate report should write");
        fs::write(
            metrics_dir.join("sweep-history.jsonl"),
            r#"{"timestamp":"1775168490","tier":"state","levels_requested":69,"passed":69,"failed":0,"errors":0,"divergence_summary":{"authoritative":0,"advisory":320,"accumulation":0,"rng":0,"oracle_limitation":648},"report_path":"/tmp/sweep-report.json","field_summary":[{"field":"game_mode","total_diffs":15,"levels":15,"divergence_summary":{"authoritative":0,"advisory":0,"accumulation":0,"rng":0,"oracle_limitation":15}}],"release_gate_focus":[{"field":"game_mode","summary":"1p-accurate failing field game_mode=15 diffs across 15 levels for boot_title_start_level1 @ title@360 [oracle_limitations=15] [includes scenario level 0x00] [excludes checkpoint title@360] [excludes checkpoint family title] checkpoints=[idle_tail=15] (field-level coverage only; exact pair evidence lives in release_gate_focus, and the release-gate scenario still needs its own proof) [exact pair 0x000c->0x002c unseen in state sweep; exact-level and checkpoint correlation absent]"}]}
"#,
        )
        .expect("sweep history should write");

        let outcome = latest_release_gate_signal(temp_dir.path());
        assert!(!outcome.ok);
        assert!(
            outcome
                .detail
                .contains("frame_oracle:boot_title_start_level1"),
            "signal should still include the failed step name"
        );
        assert!(
            outcome
                .detail
                .contains("latest state sweep reports: 1p-accurate failing field game_mode=15 diffs across 15 levels"),
            "signal should prefer the richer release-gate focus summary when it is present"
        );
        assert!(
            outcome.detail.contains("report=/tmp/sweep-report.json"),
            "signal should include the sweep report path"
        );
    }

    #[test]
    fn release_gate_signal_preserves_exact_pair_checkpoint_miss_context() {
        let temp_dir = TempDir::new().expect("temp dir should exist");
        let gate_dir = temp_dir.path().join(".cache/release-gate/1p-accurate");
        let metrics_dir = temp_dir.path().join(".cache/metrics");
        fs::create_dir_all(&gate_dir).expect("gate dir should exist");
        fs::create_dir_all(&metrics_dir).expect("metrics dir should exist");
        fs::write(
            gate_dir.join("latest.json"),
            r#"
            {
              "timestamp": "1775366549",
              "target": "1p-accurate",
              "passed": false,
              "steps": [
                {
                  "name": "gameplay_compare:pause_resume_level1",
                  "status": "failed",
                  "detail": "gameplay state level_settle@719 diverged in game mode (native 0008, oracle 000C)"
                }
              ]
            }
            "#,
        )
        .expect("gate report should write");
        fs::write(
            metrics_dir.join("sweep-history.jsonl"),
            r#"{"timestamp":"1775366423","tier":"state","levels_requested":69,"passed":69,"failed":0,"errors":0,"divergence_summary":{"authoritative":0,"advisory":191,"accumulation":0,"rng":0,"oracle_limitation":668},"report_path":"/tmp/sweep-report.json","field_summary":[{"field":"game_mode","total_diffs":13,"levels":12,"level_ids":["0x00","0x09","0x32","0x33","0x45","0x48","0x50","0x54","0x57","0x61","0x62","0x63"],"checkpoints":["idle_tail","level_settle"],"checkpoint_counts":[{"checkpoint":"idle_tail","diffs":12},{"checkpoint":"level_settle","diffs":1}],"divergence_summary":{"authoritative":0,"advisory":0,"accumulation":0,"rng":0,"oracle_limitation":13}}],"release_gate_focus":[{"field":"game_mode","summary":"1p-accurate failing field game_mode=13 diffs across 12 levels for pause_resume_level1 @ level_settle@719 [oracle_limitations=13] [0x00, 0x09, 0x32, 0x33, 0x45, +7 more] [includes scenario level 0x00] [excludes checkpoint level_settle@719] [includes checkpoint family level_settle] checkpoints=[idle_tail=12, level_settle=1] (field-level coverage only; exact pair evidence lives in release_gate_focus, and the release-gate scenario still needs its own proof) [exact pair 0x0008->0x000c seen on levels [0x09, 0x32, 0x33, 0x45, 0x48, +6 more] checkpoints=[idle_tail=11]; excludes scenario level 0x00; excludes checkpoint level_settle@719; excludes checkpoint family level_settle]"}]}
"#,
        )
        .expect("sweep history should write");

        let outcome = latest_release_gate_signal(temp_dir.path());
        assert!(!outcome.ok);
        assert!(
            outcome
                .detail
                .contains("gameplay_compare:pause_resume_level1"),
            "signal should still include the failed gameplay step"
        );
        assert!(
            outcome.detail.contains("excludes scenario level 0x00"),
            "signal should preserve the exact-pair scenario-level miss from release_gate_focus"
        );
        assert!(
            outcome
                .detail
                .contains("excludes checkpoint family level_settle"),
            "signal should preserve the exact-pair checkpoint-family miss from release_gate_focus"
        );
    }

    #[test]
    fn release_gate_signal_prefers_exact_focus_entry_over_field_only_match() {
        let temp_dir = TempDir::new().expect("temp dir should exist");
        let gate_dir = temp_dir.path().join(".cache/release-gate/1p-accurate");
        let metrics_dir = temp_dir.path().join(".cache/metrics");
        fs::create_dir_all(&gate_dir).expect("gate dir should exist");
        fs::create_dir_all(&metrics_dir).expect("metrics dir should exist");
        fs::write(
            gate_dir.join("latest.json"),
            r#"
            {
              "timestamp": "1775367135",
              "target": "1p-accurate",
              "passed": false,
              "steps": [
                {
                  "name": "gameplay_compare:pause_resume_level1",
                  "status": "failed",
                  "detail": "gameplay state level_settle@719 diverged in game mode (native 0008, oracle 000C)"
                }
              ]
            }
            "#,
        )
        .expect("gate report should write");
        fs::write(
            metrics_dir.join("sweep-history.jsonl"),
            r#"{"timestamp":"1775367561","tier":"state","levels_requested":69,"passed":69,"failed":0,"errors":0,"divergence_summary":{"authoritative":0,"advisory":191,"accumulation":0,"rng":0,"oracle_limitation":668},"report_path":"/tmp/sweep-report.json","field_summary":[{"field":"game_mode","total_diffs":13,"levels":12,"level_ids":["0x00","0x09","0x32","0x33","0x45","0x48","0x50","0x54","0x57","0x61","0x62","0x63"],"checkpoints":["idle_tail","level_settle"],"checkpoint_counts":[{"checkpoint":"idle_tail","diffs":12},{"checkpoint":"level_settle","diffs":1}],"divergence_summary":{"authoritative":0,"advisory":0,"accumulation":0,"rng":0,"oracle_limitation":13}}],"release_gate_focus":[{"target":"1p-playable","step_name":"gameplay_compare:progress_two_transitions","checkpoint":"level_settle@719","field":"game_mode","native_value":"0x0008","oracle_value":"0x000c","summary":"wrong focus summary should not win"},{"target":"1p-accurate","step_name":"gameplay_compare:pause_resume_level1","checkpoint":"level_settle@719","field":"game_mode","native_value":"0x0008","oracle_value":"0x000c","summary":"right focus summary should win"}]}
"#,
        )
        .expect("sweep history should write");

        let outcome = latest_release_gate_signal(temp_dir.path());
        assert!(!outcome.ok);
        assert!(
            outcome.detail.contains("right focus summary should win"),
            "signal should prefer the exact target/step/checkpoint match"
        );
        assert!(
            !outcome
                .detail
                .contains("wrong focus summary should not win"),
            "signal should not stop at the first field-only match"
        );
    }

    #[test]
    fn release_gate_signal_ignores_mismatched_modern_focus_entries() {
        let temp_dir = TempDir::new().expect("temp dir should exist");
        let gate_dir = temp_dir.path().join(".cache/release-gate/1p-accurate");
        let metrics_dir = temp_dir.path().join(".cache/metrics");
        fs::create_dir_all(&gate_dir).expect("gate dir should exist");
        fs::create_dir_all(&metrics_dir).expect("metrics dir should exist");
        fs::write(
            gate_dir.join("latest.json"),
            r#"
            {
              "timestamp": "1775367135",
              "target": "1p-accurate",
              "passed": false,
              "steps": [
                {
                  "name": "gameplay_compare:pause_resume_level1",
                  "status": "failed",
                  "detail": "gameplay state level_settle@719 diverged in game mode (native 0008, oracle 000C)"
                }
              ]
            }
            "#,
        )
        .expect("gate report should write");
        fs::write(
            metrics_dir.join("sweep-history.jsonl"),
            r#"{"timestamp":"1775367561","tier":"state","levels_requested":69,"passed":69,"failed":0,"errors":0,"divergence_summary":{"authoritative":0,"advisory":191,"accumulation":0,"rng":0,"oracle_limitation":668},"report_path":"/tmp/sweep-report.json","field_summary":[{"field":"game_mode","total_diffs":13,"levels":12,"level_ids":["0x00"],"checkpoints":["idle_tail"],"checkpoint_counts":[{"checkpoint":"idle_tail","diffs":12}],"divergence_summary":{"authoritative":0,"advisory":0,"accumulation":0,"rng":0,"oracle_limitation":13}}],"release_gate_focus":[{"target":"1p-playable","step_name":"gameplay_compare:progress_two_transitions","checkpoint":"level_settle@719","field":"game_mode","native_value":"0x0008","oracle_value":"0x000c","summary":"wrong modern focus summary should not win"}]}
"#,
        )
        .expect("sweep history should write");

        let outcome = latest_release_gate_signal(temp_dir.path());
        assert!(!outcome.ok);
        assert!(
            !outcome
                .detail
                .contains("wrong modern focus summary should not win"),
            "modern focus entries must match target/step/checkpoint before reuse"
        );
        assert!(
            outcome
                .detail
                .contains("latest state sweep classifies `game_mode`"),
            "signal should fall back to field-summary classification when only mismatched modern focus entries exist"
        );
    }

    #[test]
    fn state_sweep_signal_reports_advisory_and_oracle_limitation_counts() {
        let temp_dir = TempDir::new().expect("temp dir should exist");
        let metrics_dir = temp_dir.path().join(".cache/metrics");
        fs::create_dir_all(&metrics_dir).expect("metrics dir should exist");
        fs::write(
            metrics_dir.join("sweep-history.jsonl"),
            r#"{"timestamp":"1775120772","tier":"state","levels_requested":69,"passed":69,"failed":0,"errors":0,"divergence_summary":{"authoritative":0,"advisory":320,"accumulation":0,"rng":0,"oracle_limitation":648},"report_path":"/tmp/sweep-report.json"}
"#,
        )
        .expect("sweep history should write");

        let outcome = latest_state_sweep_signal(temp_dir.path());
        assert!(outcome.ok);
        assert!(
            outcome.detail.contains("advisory=320"),
            "signal should include advisory counts"
        );
        assert!(
            outcome.detail.contains("oracle_limitations=648"),
            "signal should include oracle limitation counts"
        );
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
        fs::write(
            temp_dir.path().join("docs/source-port/methodology.md"),
            "# Methodology\n",
        )
        .expect("methodology doc should write");
        fs::write(
            temp_dir.path().join("docs/source-port/research-map.md"),
            "# Research Map\n",
        )
        .expect("research map should write");
        fs::write(
            temp_dir.path().join("docs/source-port/asm-coverage.yaml"),
            "generated_at: now\nsummary: { total_sub_labels: 0, total_loc_labels: 0, sub_cited: 0, sub_coverage_pct: 0.0, loc_cited: 0, loc_coverage_pct: 0.0 }\nsubsystems: {}\n",
        )
        .expect("asm coverage should write");
        Command::new("git")
            .args(["init"])
            .current_dir(temp_dir.path())
            .status()
            .expect("git init should run");
        Command::new("git")
            .args(["config", "user.email", "test@example.com"])
            .current_dir(temp_dir.path())
            .status()
            .expect("git email config should run");
        Command::new("git")
            .args(["config", "user.name", "Test User"])
            .current_dir(temp_dir.path())
            .status()
            .expect("git name config should run");
        Command::new("git")
            .args(["add", "."])
            .current_dir(temp_dir.path())
            .status()
            .expect("git add should run");
        Command::new("git")
            .args(["commit", "-m", "init"])
            .current_dir(temp_dir.path())
            .status()
            .expect("git commit should run");
        temp_dir
    }

    fn commit_repo(repo_root: &Path, message: &str) {
        Command::new("git")
            .args(["add", "."])
            .current_dir(repo_root)
            .status()
            .expect("git add should run");
        Command::new("git")
            .args(["commit", "-m", message])
            .current_dir(repo_root)
            .status()
            .expect("git commit should run");
    }

    fn mutation_target(module: &str, tier: u8) -> super::MutationTarget {
        super::MutationTarget {
            module: module.to_owned(),
            package: "kid-core".to_owned(),
            tier,
        }
    }

    fn write_mutation_run(
        repo_root: &Path,
        module_stem: &str,
        candidate_count: usize,
        outcome_count: usize,
        missed: &str,
        timed_out: &str,
        debug_log: &str,
    ) {
        let run_dir = repo_root
            .join(".cache/mutants")
            .join(module_stem)
            .join("mutants.out");
        fs::create_dir_all(&run_dir).expect("mutation run dir should exist");

        let mutants = (0..candidate_count)
            .map(|index| format!(r#"{{"name":"mutant-{index}"}}"#))
            .collect::<Vec<_>>()
            .join(",");
        fs::write(run_dir.join("mutants.json"), format!("[{mutants}]\n"))
            .expect("mutants.json should write");

        let outcomes = (0..outcome_count)
            .map(|index| {
                format!(
                    r#"{{"scenario":{{"Mutant":{{"name":"mutant-{index}"}}}},"summary":"CaughtMutant"}}"#
                )
            })
            .collect::<Vec<_>>()
            .join(",");
        fs::write(
            run_dir.join("outcomes.json"),
            format!(r#"{{"outcomes":[{{"scenario":"Baseline","summary":"Success"}}{comma}{outcomes}]}}"#, comma = if outcomes.is_empty() { "" } else { "," }),
        )
        .expect("outcomes.json should write");

        fs::write(run_dir.join("missed.txt"), missed).expect("missed.txt should write");
        fs::write(run_dir.join("timeout.txt"), timed_out).expect("timeout.txt should write");
        fs::write(run_dir.join("debug.log"), debug_log).expect("debug.log should write");
    }

    fn write_complete_default_mutation_runs(repo_root: &Path) {
        for module_stem in ["enemies", "combat", "blocks"] {
            write_mutation_run(repo_root, module_stem, 1, 1, "", "", "");
        }
    }

    fn write_clean_completion(repo_root: &Path, cycle: u32, delegated_target: &str) {
        let run_dir = repo_root.join(".cache/autonomy/source-port-frontier");
        fs::create_dir_all(&run_dir).expect("run history dir should exist");
        fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(run_dir.join("run-history.jsonl"))
            .and_then(|mut file| {
                use std::io::Write;
                writeln!(
                    file,
                    r#"{{"cycle":{cycle},"delegated_target":"{delegated_target}","outcome":"completed","exit_code":0,"dirty_tree":false}}"#
                )
            })
            .expect("run history should write");
    }

    fn write_clean_completion_with_fingerprint(
        repo_root: &Path,
        cycle: u32,
        delegated_target: &str,
        fingerprint: &str,
    ) {
        let run_dir = repo_root.join(".cache/autonomy/source-port-frontier");
        fs::create_dir_all(&run_dir).expect("run history dir should exist");
        fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(run_dir.join("run-history.jsonl"))
            .and_then(|mut file| {
                use std::io::Write;
                writeln!(
                    file,
                    r#"{{"cycle":{cycle},"delegated_target":"{delegated_target}","progress_fingerprint":"{fingerprint}","outcome":"completed","exit_code":0,"dirty_tree":false}}"#
                )
            })
            .expect("run history should write");
    }

    fn write_stalled_repeat(
        repo_root: &Path,
        cycle: u32,
        delegated_target: &str,
        fingerprint: &str,
    ) {
        let run_dir = repo_root.join(".cache/autonomy/source-port-frontier");
        fs::create_dir_all(&run_dir).expect("run history dir should exist");
        fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(run_dir.join("run-history.jsonl"))
            .and_then(|mut file| {
                use std::io::Write;
                writeln!(
                    file,
                    r#"{{"cycle":{cycle},"delegated_target":"{delegated_target}","progress_fingerprint":"{fingerprint}","outcome":"stalled_repeat","exit_code":1,"dirty_tree":false}}"#
                )
            })
            .expect("run history should write");
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

    fn write_blocked_case(
        repo_root: &Path,
        selector: &str,
        source_json: &str,
        blocked_reason: &str,
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
  "first_failure": null,
  "likely_subsystems": ["recording verification"],
  "blocked_reason": "{blocked_reason}"
}}
"#
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
            format!(
                r#"{{
  "slug":"{slug}",
  "scenario": {{
    "frame_count": 180
  }}
}}"#
            ),
        )
        .expect("recording manifest should write");
        if with_transcript {
            fs::write(recording_dir.join("transcript.txt"), "trace transcript\n")
                .expect("transcript should write");
        }
    }

    fn write_backlog(repo_root: &Path, rows: &str) {
        fs::write(
            repo_root.join(super::SOURCE_PORT_BACKLOG_PATH),
            format!(
                "| # | Status | Pri | System | Summary | Verify |\n|-----|--------|-----|--------|---------|--------|\n{rows}"
            ),
        )
        .expect("backlog should write");
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
            evaluate_source_port_frontier_cycle(temp_dir.path(), 1, 1).expect("cycle should build");
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
        write_complete_default_mutation_runs(temp_dir.path());

        let cycle =
            evaluate_source_port_frontier_cycle(temp_dir.path(), 1, 1).expect("cycle should build");

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
        assert_eq!(task_package.cycle, 1);
        assert_eq!(
            task_package.progress_fingerprint,
            cycle.progress_fingerprint
        );
        assert!(!task_package.generated_at.is_empty());
    }

    #[test]
    fn source_port_frontier_sweep_task_carries_release_gate_evidence_into_note() {
        let temp_dir = source_port_repo();
        let gate_dir = temp_dir.path().join(".cache/release-gate/1p-accurate");
        let metrics_dir = temp_dir.path().join(".cache/metrics");
        fs::create_dir_all(temp_dir.path().join("docs/source-port"))
            .expect("docs dir should exist");
        fs::create_dir_all(&gate_dir).expect("gate dir should exist");
        fs::create_dir_all(&metrics_dir).expect("metrics dir should exist");
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
            gate_dir.join("latest.json"),
            r#"{
  "timestamp": "1775368044",
  "target": "1p-accurate",
  "passed": false,
  "steps": [
    {
      "name": "gameplay_compare:pause_resume_level1",
      "status": "failed",
      "detail": "gameplay state level_settle@719 diverged in game mode (native 0008, oracle 000C)"
    }
  ]
}
"#,
        )
        .expect("release gate latest should write");
        fs::write(
            metrics_dir.join("sweep-history.jsonl"),
            r#"{"timestamp":"1775367006","tier":"state","levels_requested":69,"passed":69,"failed":0,"errors":0,"divergence_summary":{"authoritative":0,"advisory":191,"accumulation":0,"rng":0,"oracle_limitation":668},"report_path":"/tmp/sweep-report.json","field_summary":[{"field":"game_mode","total_diffs":13,"levels":12,"level_ids":["0x00","0x09","0x32","0x33","0x45","0x48","0x50","0x54","0x57","0x61","0x62","0x63"],"checkpoints":["idle_tail","level_settle"],"checkpoint_counts":[{"checkpoint":"idle_tail","diffs":12},{"checkpoint":"level_settle","diffs":1}],"divergence_summary":{"authoritative":0,"advisory":0,"accumulation":0,"rng":0,"oracle_limitation":13}}],"release_gate_focus":[{"target":"1p-accurate","step_name":"gameplay_compare:pause_resume_level1","checkpoint":"level_settle@719","field":"game_mode","native_value":"0x0008","oracle_value":"0x000c","summary":"1p-accurate failing field game_mode=13 diffs across 12 levels for pause_resume_level1 @ level_settle@719 [oracle_limitations=13] [0x00, 0x09, 0x32, 0x33, 0x45, +7 more] [includes scenario level 0x00] [excludes checkpoint level_settle@719] [includes checkpoint family level_settle] checkpoints=[idle_tail=12, level_settle=1] (field-level coverage only; exact pair evidence lives in release_gate_focus, and the release-gate scenario still needs its own proof) [exact pair 0x0008->0x000c seen on levels [0x09, 0x32, 0x33, 0x45, 0x48, +6 more] checkpoints=[idle_tail=11]; excludes scenario level 0x00; excludes checkpoint level_settle@719; excludes checkpoint family level_settle]"}]}
"#,
        )
        .expect("sweep history should write");

        let cycle =
            evaluate_source_port_frontier_cycle(temp_dir.path(), 1, 1).expect("cycle should build");
        let task = cycle.selected_task.expect("selected task should exist");
        let frontier = task
            .frontier_context
            .as_ref()
            .expect("frontier context should exist");
        let rendered_note = render_source_port_next_agent_note(frontier, &task, &[]);

        assert!(
            task.evidence
                .iter()
                .any(|item| item.label == "release-gate-failure"
                    && item.detail.contains("gameplay_compare:pause_resume_level1")
                    && item.detail.contains("timestamp 1775368044")),
            "task should carry the live release-gate failure detail"
        );
        assert!(
            task.evidence
                .iter()
                .any(|item| item.label == "release-gate-focus"
                    && item.detail.contains("exact pair 0x0008->0x000c")
                    && item.detail.contains("timestamp 1775367006")),
            "task should carry the correlated state-sweep focus summary"
        );
        assert!(
            rendered_note.contains("Release-gate failure: gameplay_compare:pause_resume_level1"),
            "next-agent note should surface the live gate failure"
        );
        assert!(
            rendered_note.contains("timestamp 1775368044"),
            "next-agent note should surface the refreshed release-gate timestamp"
        );
        assert!(
            rendered_note.contains("Latest state-sweep correlation: 1p-accurate failing field game_mode=13 diffs across 12 levels"),
            "next-agent note should surface the persisted sweep correlation"
        );
        assert!(
            rendered_note.contains("timestamp 1775367006"),
            "next-agent note should surface the refreshed state-sweep timestamp"
        );
    }

    #[test]
    fn source_port_frontier_sweep_task_carries_green_release_gate_evidence_into_note() {
        let temp_dir = source_port_repo();
        let gate_dir = temp_dir.path().join(".cache/release-gate/1p-accurate");
        let metrics_dir = temp_dir.path().join(".cache/metrics");
        fs::create_dir_all(temp_dir.path().join("docs/source-port"))
            .expect("docs dir should exist");
        fs::create_dir_all(&gate_dir).expect("gate dir should exist");
        fs::create_dir_all(&metrics_dir).expect("metrics dir should exist");
        let gate_report_dir = gate_dir.join("1776704737");
        fs::create_dir_all(&gate_report_dir).expect("timestamped gate dir should exist");
        fs::write(gate_report_dir.join("report.json"), "{}")
            .expect("timestamped gate report should write");
        write_source_port_frontier(temp_dir.path(), false, "complete");
        fs::write(
            gate_dir.join("latest.json"),
            r#"{
  "timestamp": "1776704737",
  "target": "1p-accurate",
  "passed": true,
  "steps": [
    {
      "name": "state_sweep:0-104",
      "status": "passed",
      "detail": "all 69 requested levels passed the state sweep"
    }
  ],
  "sweep_summary": {
    "authoritative": 0,
    "advisory": 178,
    "accumulation": 0,
    "rng": 0,
    "oracle_limitation": 662
  }
}
"#,
        )
        .expect("release gate latest should write");
        fs::write(
            metrics_dir.join("sweep-history.jsonl"),
            r#"{"timestamp":"1776704792","tier":"state","levels_requested":69,"passed":69,"failed":0,"errors":0,"divergence_summary":{"authoritative":0,"advisory":178,"accumulation":0,"rng":0,"oracle_limitation":662},"report_path":"/tmp/sweep-report-green.json","field_summary":[],"release_gate_focus":[]}
"#,
        )
        .expect("sweep history should write");

        let frontier = load_source_port_frontier_state(temp_dir.path())
            .expect("frontier state should load")
            .active_frontier;
        let note = source_port_next_agent_note(temp_dir.path(), frontier.slug.as_str());
        let task = source_port_frontier_sweep_task(temp_dir.path(), &frontier, &note);
        let rendered_note = render_source_port_next_agent_note(
            task.frontier_context
                .as_ref()
                .expect("frontier context should exist"),
            &task,
            &[],
        );

        assert!(
            task.evidence
                .iter()
                .any(|item| item.label == "release-gate-status"
                    && item.status == "passed"
                    && item.detail.contains("timestamp 1776704737")
                    && item
                        .detail
                        .contains(".cache/release-gate/1p-accurate/1776704737/report.json")
                    && item.artefact_path.as_ref().is_some_and(|path| path
                        .ends_with(".cache/release-gate/1p-accurate/1776704737/report.json"))),
            "task should carry the timestamped passing release-gate artefact"
        );
        assert!(
            task.evidence
                .iter()
                .any(|item| item.label == "state-sweep-status"
                    && item.status == "passed"
                    && item.detail.contains("timestamp 1776704792")
                    && item.detail.contains("advisory=178")
                    && item.detail.contains("oracle_limitations=662")),
            "task should carry the latest passing state-sweep artefact"
        );
        assert!(
            task.done_criteria.iter().any(|criterion| criterion
                .contains("passing release gate and latest state-sweep report")),
            "green gate tasks should ask for refreshed green-lane evidence"
        );
        assert!(
            rendered_note.contains(
                "Release-gate status: latest release gate passed (timestamp 1776704737, report="
            ),
            "next-agent note should surface the timestamped green release gate report"
        );
        assert!(
            rendered_note.contains("Latest state-sweep status: latest state sweep passed"),
            "next-agent note should surface the latest state-sweep report"
        );
    }

    #[test]
    fn source_port_frontier_green_gate_prefers_newer_state_sweep_report_when_history_lags() {
        let temp_dir = source_port_repo();
        let gate_dir = temp_dir.path().join(".cache/release-gate/1p-accurate");
        let metrics_dir = temp_dir.path().join(".cache/metrics");
        let fresh_sweep_dir = temp_dir.path().join(".cache/sweep-runs/1776800896");
        fs::create_dir_all(&gate_dir).expect("gate dir should exist");
        fs::create_dir_all(&metrics_dir).expect("metrics dir should exist");
        fs::create_dir_all(&fresh_sweep_dir).expect("fresh sweep dir should exist");
        write_source_port_frontier(temp_dir.path(), false, "complete");
        fs::write(
            gate_dir.join("latest.json"),
            r#"{
  "timestamp": "1776800861",
  "target": "1p-accurate",
  "passed": true,
  "steps": [
    {
      "name": "state_sweep:0-104",
      "status": "passed",
      "detail": "all 69 requested levels passed the state sweep"
    }
  ],
  "sweep_summary": {
    "authoritative": 0,
    "advisory": 178,
    "accumulation": 0,
    "rng": 0,
    "oracle_limitation": 662
  }
}
"#,
        )
        .expect("release gate latest should write");
        fs::write(
            metrics_dir.join("sweep-history.jsonl"),
            r#"{"timestamp":"1776800852","tier":"state","levels_requested":69,"passed":69,"failed":0,"errors":0,"divergence_summary":{"authoritative":0,"advisory":177,"accumulation":0,"rng":0,"oracle_limitation":661},"report_path":"/tmp/stale-sweep-report.json","field_summary":[],"release_gate_focus":[]}
"#,
        )
        .expect("stale sweep history should write");
        fs::write(
            fresh_sweep_dir.join("report.json"),
            r#"{
  "timestamp": "1776800896",
  "kid_core_hash": "deadbeef",
  "levels": [
    {
      "level_id": 0,
      "slug": "idle_level_00",
      "tier": "state",
      "passed": true,
      "state_diffs": [],
      "mae": null,
      "diff_images": [],
      "error": null,
      "divergence_summary": {
        "authoritative": 0,
        "advisory": 0,
        "accumulation": 0,
        "rng": 0,
        "oracle_limitation": 0
      }
    }
  ],
  "passed": 69,
  "failed": 0,
  "errors": 0,
  "divergence_summary": {
    "authoritative": 0,
    "advisory": 178,
    "accumulation": 0,
    "rng": 0,
    "oracle_limitation": 662
  },
  "field_summary": [],
  "release_gate_focus": []
}
"#,
        )
        .expect("fresh sweep report should write");

        let frontier = load_source_port_frontier_state(temp_dir.path())
            .expect("frontier state should load")
            .active_frontier;
        let note = source_port_next_agent_note(temp_dir.path(), frontier.slug.as_str());
        let task = source_port_frontier_sweep_task(temp_dir.path(), &frontier, &note);
        let rendered_note = render_source_port_next_agent_note(
            task.frontier_context
                .as_ref()
                .expect("frontier context should exist"),
            &task,
            &[],
        );

        assert!(
            task.evidence
                .iter()
                .any(|item| item.label == "state-sweep-status"
                    && item.status == "passed"
                    && item.detail.contains("timestamp 1776800896")
                    && item.detail.contains("advisory=178")
                    && item.detail.contains("oracle_limitations=662")
                    && item
                        .artefact_path
                        .as_ref()
                        .is_some_and(|path| path == &fresh_sweep_dir.join("report.json"))),
            "green sweep handoff should prefer the newer direct report over stale history"
        );
        assert!(
            rendered_note.contains("timestamp 1776800896")
                && rendered_note.contains(".cache/sweep-runs/1776800896/report.json"),
            "next-agent note should cite the newer direct sweep report"
        );
        assert!(
            !rendered_note.contains("1776800852") && !rendered_note.contains("stale-sweep-report"),
            "next-agent note should not reuse stale sweep-history evidence"
        );
    }

    #[test]
    fn sweep_parity_refresh_updates_source_port_handoff_note() {
        let temp_dir = source_port_repo();
        let gate_dir = temp_dir.path().join(".cache/release-gate/1p-accurate");
        let metrics_dir = temp_dir.path().join(".cache/metrics");
        fs::create_dir_all(&gate_dir).expect("gate dir should exist");
        fs::create_dir_all(&metrics_dir).expect("metrics dir should exist");
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
            gate_dir.join("latest.json"),
            r#"{
  "timestamp": "1775369194",
  "target": "1p-accurate",
  "passed": false,
  "steps": [
    {
      "name": "gameplay_compare:pause_resume_level1",
      "status": "failed",
      "detail": "gameplay state level_settle@719 diverged in game mode (native 0008, oracle 000C)"
    }
  ]
}
"#,
        )
        .expect("release gate latest should write");
        fs::write(
            metrics_dir.join("sweep-history.jsonl"),
            r#"{"timestamp":"1775369195","tier":"state","levels_requested":69,"passed":69,"failed":0,"errors":0,"divergence_summary":{"authoritative":0,"advisory":191,"accumulation":0,"rng":0,"oracle_limitation":668},"report_path":"/tmp/sweep-report.json","field_summary":[{"field":"game_mode","total_diffs":13,"levels":12,"level_ids":["0x00","0x09","0x32","0x33","0x45","0x48","0x50","0x54","0x57","0x61","0x62","0x63"],"checkpoints":["idle_tail","level_settle"],"checkpoint_counts":[{"checkpoint":"idle_tail","diffs":12},{"checkpoint":"level_settle","diffs":1}],"divergence_summary":{"authoritative":0,"advisory":0,"accumulation":0,"rng":0,"oracle_limitation":13}}],"release_gate_focus":[{"target":"1p-accurate","step_name":"gameplay_compare:pause_resume_level1","checkpoint":"level_settle@719","field":"game_mode","native_value":"0x0008","oracle_value":"0x000c","summary":"1p-accurate failing field game_mode=13 diffs across 12 levels for pause_resume_level1 @ level_settle@719 [oracle_limitations=13] [0x00, 0x09, 0x32, 0x33, 0x45, +7 more] [includes scenario level 0x00] [excludes checkpoint level_settle@719] [includes checkpoint family level_settle] checkpoints=[idle_tail=12, level_settle=1] (field-level coverage only; exact pair evidence lives in release_gate_focus, and the release-gate scenario still needs its own proof) [exact pair 0x0008->0x000c seen on levels [0x09, 0x32, 0x33, 0x45, 0x48, +6 more] checkpoints=[idle_tail=11]; excludes scenario level 0x00; excludes checkpoint level_settle@719; excludes checkpoint family level_settle]"}]}
"#,
        )
        .expect("sweep history should write");

        super::refresh_source_port_frontier_handoff_from_latest_evidence(temp_dir.path(), 1)
            .expect("frontier handoff refresh should succeed");

        let note = fs::read_to_string(temp_dir.path().join(SOURCE_PORT_FRONTIER_NOTE_PATH))
            .expect("next-agent note should write");
        assert!(
            note.contains("timestamp 1775369194"),
            "refreshed note should include the latest release-gate timestamp"
        );
        assert!(
            note.contains("timestamp 1775369195"),
            "refreshed note should include the latest state-sweep timestamp"
        );
    }

    #[test]
    fn release_gate_refresh_records_source_port_refresh_evidence_completion() {
        let temp_dir = source_port_repo();
        let gate_dir = temp_dir.path().join(".cache/release-gate/1p-accurate");
        let metrics_dir = temp_dir.path().join(".cache/metrics");
        fs::create_dir_all(&gate_dir).expect("gate dir should exist");
        fs::create_dir_all(&metrics_dir).expect("metrics dir should exist");
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
            gate_dir.join("latest.json"),
            r#"{
  "timestamp": "1775369194",
  "target": "1p-accurate",
  "passed": true,
  "steps": [
    {
      "name": "state_sweep:0-104",
      "status": "passed",
      "detail": "all 69 requested levels passed the state sweep"
    }
  ],
  "sweep_summary": {
    "authoritative": 0,
    "advisory": 178,
    "accumulation": 0,
    "rng": 0,
    "oracle_limitation": 662
  }
}
"#,
        )
        .expect("release gate latest should write");
        fs::write(
            metrics_dir.join("sweep-history.jsonl"),
            r#"{"timestamp":"1775369195","tier":"state","levels_requested":69,"passed":69,"failed":0,"errors":0,"divergence_summary":{"authoritative":0,"advisory":178,"accumulation":0,"rng":0,"oracle_limitation":662},"report_path":"/tmp/sweep-report-green.json","field_summary":[],"release_gate_focus":[]}
"#,
        )
        .expect("sweep history should write");
        write_complete_default_mutation_runs(temp_dir.path());
        commit_repo(temp_dir.path(), "prepare clean recovery fixture");

        let baseline =
            evaluate_source_port_frontier_cycle(temp_dir.path(), 1, 1).expect("cycle should build");
        let baseline_task = baseline.selected_task.expect("selected task should exist");
        let stalled_fingerprint = baseline_task.progress_fingerprint.clone();
        write_stalled_repeat(temp_dir.path(), 1, "state-sweep", &stalled_fingerprint);
        write_stalled_repeat(temp_dir.path(), 2, "state-sweep", &stalled_fingerprint);

        super::refresh_source_port_frontier_handoff_after_release_gate(temp_dir.path(), 1)
            .expect("release-gate handoff refresh should record completion");

        let run_history = fs::read_to_string(
            temp_dir
                .path()
                .join(super::SOURCE_PORT_FRONTIER_RUN_HISTORY_PATH),
        )
        .expect("run history should exist");
        let latest_record = run_history
            .lines()
            .last()
            .expect("history should have a row");
        let latest_record: serde_json::Value =
            serde_json::from_str(latest_record).expect("history row should parse");
        assert_eq!(latest_record["outcome"], "completed");
        assert_eq!(latest_record["recovery_step"], "refresh_evidence");
        assert!(
            latest_record["progress_fingerprint"]
                .as_str()
                .is_some_and(|fingerprint| fingerprint.contains(":recovery:refresh_evidence")),
            "completion should record the recovery fingerprint"
        );
        let refreshed_task = fs::read_to_string(
            temp_dir
                .path()
                .join(".cache/autonomy/source-port-frontier/next-task.json"),
        )
        .expect("next task should be refreshed after completion");
        assert!(
            !refreshed_task.contains("\"recovery_step\": \"refresh_evidence\""),
            "handoff should be regenerated after recording the refresh completion"
        );

        let next_cycle =
            evaluate_source_port_frontier_cycle(temp_dir.path(), 4, 1).expect("cycle should build");
        let next_task = next_cycle
            .selected_task
            .expect("selected task should exist");
        assert_eq!(
            next_task.recovery_step, None,
            "the refresh completion should break the stale stalled-repeat chain"
        );
        assert_eq!(
            next_task.delegated_goal,
            Some(super::AutonomyGoal::SweepParity)
        );
    }

    #[test]
    fn source_port_frontier_prefers_newer_state_sweep_report_when_history_lags() {
        let temp_dir = source_port_repo();
        let gate_dir = temp_dir.path().join(".cache/release-gate/1p-accurate");
        let metrics_dir = temp_dir.path().join(".cache/metrics");
        let fresh_sweep_dir = temp_dir.path().join(".cache/sweep-runs/1775371457");
        fs::create_dir_all(&gate_dir).expect("gate dir should exist");
        fs::create_dir_all(&metrics_dir).expect("metrics dir should exist");
        fs::create_dir_all(&fresh_sweep_dir).expect("fresh sweep dir should exist");
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
            gate_dir.join("latest.json"),
            r#"{
  "timestamp": "1775371457",
  "target": "1p-accurate",
  "passed": false,
  "steps": [
    {
      "name": "gameplay_compare:pause_resume_level1",
      "status": "failed",
      "detail": "gameplay state level_settle@719 diverged in game mode (native 0008, oracle 000C)"
    }
  ]
}
"#,
        )
        .expect("release gate latest should write");
        fs::write(
            metrics_dir.join("sweep-history.jsonl"),
            r#"{"timestamp":"1775371156","tier":"state","levels_requested":69,"passed":69,"failed":0,"errors":0,"divergence_summary":{"authoritative":0,"advisory":191,"accumulation":0,"rng":0,"oracle_limitation":668},"report_path":"/tmp/stale-report.json","field_summary":[{"field":"game_mode","total_diffs":13,"levels":12,"level_ids":["0x00"],"checkpoints":["idle_tail"],"checkpoint_counts":[{"checkpoint":"idle_tail","diffs":12}],"divergence_summary":{"authoritative":0,"advisory":0,"accumulation":0,"rng":0,"oracle_limitation":13}}],"release_gate_focus":[{"target":"1p-accurate","step_name":"gameplay_compare:pause_resume_level1","checkpoint":"level_settle@719","field":"game_mode","native_value":"0x0008","oracle_value":"0x000c","summary":"stale summary"}]}
"#,
        )
        .expect("stale sweep history should write");
        fs::write(
            fresh_sweep_dir.join("report.json"),
            "{\n  \"timestamp\": \"1775371457\",\n  \"kid_core_hash\": \"deadbeef\",\n  \"levels\": [{\"level_id\": 0, \"slug\": \"idle_level_00\", \"tier\": \"state\", \"passed\": true, \"state_diffs\": [], \"mae\": null, \"diff_images\": [], \"error\": null, \"divergence_summary\": {\"authoritative\": 0, \"advisory\": 0, \"accumulation\": 0, \"rng\": 0, \"oracle_limitation\": 0}}],\n  \"passed\": 69,\n  \"failed\": 0,\n  \"errors\": 0,\n  \"divergence_summary\": {\"authoritative\": 0, \"advisory\": 191, \"accumulation\": 0, \"rng\": 0, \"oracle_limitation\": 668},\n  \"field_summary\": [{\"field\": \"game_mode\", \"total_diffs\": 13, \"levels\": 12, \"level_ids\": [\"0x00\"], \"checkpoints\": [\"idle_tail\", \"level_settle\"], \"checkpoint_counts\": [{\"checkpoint\": \"idle_tail\", \"diffs\": 12}, {\"checkpoint\": \"level_settle\", \"diffs\": 1}], \"divergence_summary\": {\"authoritative\": 0, \"advisory\": 0, \"accumulation\": 0, \"rng\": 0, \"oracle_limitation\": 13}}],\n  \"release_gate_focus\": [{\"target\": \"1p-accurate\", \"step_name\": \"gameplay_compare:pause_resume_level1\", \"checkpoint\": \"level_settle@719\", \"field\": \"game_mode\", \"native_value\": \"0x0008\", \"oracle_value\": \"0x000c\", \"summary\": \"fresh summary from report\"}]\n}\n",
        )
        .expect("fresh sweep report should write");

        let cycle =
            evaluate_source_port_frontier_cycle(temp_dir.path(), 1, 1).expect("cycle should build");
        let task = cycle.selected_task.expect("selected task should exist");
        let rendered_note = render_source_port_next_agent_note(
            task.frontier_context
                .as_ref()
                .expect("frontier context should exist"),
            &task,
            &[],
        );

        assert!(
            task.evidence
                .iter()
                .any(|item| item.label == "release-gate-focus"
                    && item.detail.contains("fresh summary from report")
                    && item.detail.contains("timestamp 1775371457")
                    && item
                        .artefact_path
                        .as_ref()
                        .is_some_and(|path| path == &fresh_sweep_dir.join("report.json"))),
            "task should prefer the newer persisted sweep report when history is stale"
        );
        assert!(
            rendered_note.contains("fresh summary from report"),
            "next-agent note should surface the newer sweep report summary"
        );
    }

    #[test]
    fn source_port_frontier_prefers_state_sweep_report_when_history_matches_timestamp() {
        let temp_dir = source_port_repo();
        let gate_dir = temp_dir.path().join(".cache/release-gate/1p-accurate");
        let metrics_dir = temp_dir.path().join(".cache/metrics");
        let sweep_dir = temp_dir.path().join(".cache/sweep-runs/1775371457");
        fs::create_dir_all(&gate_dir).expect("gate dir should exist");
        fs::create_dir_all(&metrics_dir).expect("metrics dir should exist");
        fs::create_dir_all(&sweep_dir).expect("sweep dir should exist");
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
            gate_dir.join("latest.json"),
            r#"{
  "timestamp": "1775371457",
  "target": "1p-accurate",
  "passed": true,
  "steps": [
    {
      "name": "state_sweep:0-104",
      "status": "passed",
      "detail": "all 69 requested levels passed the state sweep"
    }
  ],
  "sweep_summary": {
    "authoritative": 0,
    "advisory": 178,
    "accumulation": 0,
    "rng": 0,
    "oracle_limitation": 662
  }
}
"#,
        )
        .expect("release gate latest should write");
        fs::write(
            metrics_dir.join("sweep-history.jsonl"),
            r#"{"timestamp":"1775371457","tier":"state","levels_requested":69,"passed":69,"failed":0,"errors":0,"divergence_summary":{"authoritative":0,"advisory":178,"accumulation":0,"rng":0,"oracle_limitation":662},"report_path":"/tmp/history-sweep-report.json","field_summary":[],"release_gate_focus":[]}
"#,
        )
        .expect("sweep history should write");
        fs::write(
            sweep_dir.join("report.json"),
            r#"{
  "timestamp": "1775371457",
  "kid_core_hash": "deadbeef",
  "levels": [
    {
      "level_id": 0,
      "slug": "idle_level_00",
      "tier": "state",
      "passed": true,
      "state_diffs": [],
      "mae": null,
      "diff_images": [],
      "error": null,
      "divergence_summary": {
        "authoritative": 0,
        "advisory": 0,
        "accumulation": 0,
        "rng": 0,
        "oracle_limitation": 0
      }
    }
  ],
  "passed": 69,
  "failed": 0,
  "errors": 0,
  "divergence_summary": {
    "authoritative": 0,
    "advisory": 178,
    "accumulation": 0,
    "rng": 0,
    "oracle_limitation": 662
  },
  "field_summary": [],
  "release_gate_focus": []
}
"#,
        )
        .expect("sweep report should write");

        let frontier = load_source_port_frontier_state(temp_dir.path())
            .expect("frontier state should load")
            .active_frontier;
        let note = source_port_next_agent_note(temp_dir.path(), frontier.slug.as_str());
        let task = source_port_frontier_sweep_task(temp_dir.path(), &frontier, &note);

        assert!(
            task.evidence
                .iter()
                .any(|item| item.label == "state-sweep-status"
                    && item.status == "passed"
                    && item
                        .artefact_path
                        .as_ref()
                        .is_some_and(|path| path == &sweep_dir.join("report.json"))),
            "task should cite the direct state-sweep report when history mirrors the same run"
        );
    }

    #[test]
    fn source_port_frontier_orders_state_sweep_reports_by_payload_timestamp() {
        let temp_dir = source_port_repo();
        let gate_dir = temp_dir.path().join(".cache/release-gate/1p-accurate");
        let metrics_dir = temp_dir.path().join(".cache/metrics");
        let stale_copied_sweep_dir = temp_dir.path().join(".cache/sweep-runs/9999999999");
        fs::create_dir_all(&gate_dir).expect("gate dir should exist");
        fs::create_dir_all(&metrics_dir).expect("metrics dir should exist");
        fs::create_dir_all(&stale_copied_sweep_dir).expect("sweep dir should exist");
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
            gate_dir.join("latest.json"),
            r#"{
  "timestamp": "1775371458",
  "target": "1p-accurate",
  "passed": true,
  "steps": [
    {
      "name": "state_sweep:0-104",
      "status": "passed",
      "detail": "all 69 requested levels passed the state sweep"
    }
  ],
  "sweep_summary": {
    "authoritative": 0,
    "advisory": 178,
    "accumulation": 0,
    "rng": 0,
    "oracle_limitation": 662
  }
}
"#,
        )
        .expect("release gate latest should write");
        fs::write(
            metrics_dir.join("sweep-history.jsonl"),
            r#"{"timestamp":"1775371458","tier":"state","levels_requested":69,"passed":69,"failed":0,"errors":0,"divergence_summary":{"authoritative":0,"advisory":178,"accumulation":0,"rng":0,"oracle_limitation":662},"report_path":"/tmp/fresh-history-report.json","field_summary":[],"release_gate_focus":[]}
"#,
        )
        .expect("sweep history should write");
        fs::write(
            stale_copied_sweep_dir.join("report.json"),
            r#"{
  "timestamp": "1775371000",
  "kid_core_hash": "deadbeef",
  "levels": [
    {
      "level_id": 0,
      "slug": "idle_level_00",
      "tier": "state",
      "passed": true,
      "state_diffs": [],
      "mae": null,
      "diff_images": [],
      "error": null,
      "divergence_summary": {
        "authoritative": 0,
        "advisory": 0,
        "accumulation": 0,
        "rng": 0,
        "oracle_limitation": 0
      }
    }
  ],
  "passed": 69,
  "failed": 0,
  "errors": 0,
  "divergence_summary": {
    "authoritative": 0,
    "advisory": 999,
    "accumulation": 0,
    "rng": 0,
    "oracle_limitation": 999
  },
  "field_summary": [],
  "release_gate_focus": []
}
"#,
        )
        .expect("stale copied sweep report should write");

        let frontier = load_source_port_frontier_state(temp_dir.path())
            .expect("frontier state should load")
            .active_frontier;
        let note = source_port_next_agent_note(temp_dir.path(), frontier.slug.as_str());
        let task = source_port_frontier_sweep_task(temp_dir.path(), &frontier, &note);

        assert!(
            task.evidence
                .iter()
                .any(|item| item.label == "state-sweep-status"
                    && item.status == "passed"
                    && item.detail.contains("timestamp 1775371458")
                    && item.detail.contains("advisory=178")
                    && item.detail.contains("oracle_limitations=662")
                    && item
                        .artefact_path
                        .as_ref()
                        .is_some_and(|path| path
                            == &temp_dir.path().join(".cache/metrics/sweep-history.jsonl"))),
            "fresh history should beat a stale copied report in a newer-looking cache directory"
        );
    }

    #[test]
    fn source_port_frontier_skips_green_case_queue_bundles() {
        let temp_dir = source_port_repo();
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
        write_case(
            temp_dir.path(),
            "green-case",
            r#"{"kind":"gameplay_scenario","slug":"pause_resume_level1"}"#,
            "player.hitpoints",
            472,
            &["combat"],
            0,
            60,
        );
        fs::write(
            temp_dir
                .path()
                .join(SOURCE_PORT_CASE_CACHE_PATH)
                .join("green-case")
                .join("verify.json"),
            "{\n  \"passed\": true,\n  \"first_failure\": null,\n  \"likely_subsystems\": [\"none\"]\n}\n",
        )
        .expect("green verify should write");
        write_complete_default_mutation_runs(temp_dir.path());

        let cycle =
            evaluate_source_port_frontier_cycle(temp_dir.path(), 1, 1).expect("cycle should build");
        let task = cycle.selected_task.expect("selected task should exist");

        assert_eq!(task.delegated_goal, Some(super::AutonomyGoal::SweepParity));
        assert_eq!(task.delegated_target.as_deref(), Some("state-sweep"));
        assert!(task.summary.contains("sweep parity"));
    }

    #[test]
    fn sweep_parity_stalled_fingerprint_pivots_to_refresh_evidence_recovery() {
        // Reproduce the cycle 455-462 stall pattern from production: TAS frontier
        // closed, no case-queue or PV work, dispatch falls through to
        // state-sweep on the SweepParity lane. After two consecutive stalled
        // repeats on the resulting fingerprint, the SweepParity arm of the
        // stall-recovery dispatch must promote the task package to a recovery
        // package, otherwise the runner would spin until something else
        // (e.g. ASM coverage) advances the lane fingerprint.
        let temp_dir = source_port_repo();
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
        write_complete_default_mutation_runs(temp_dir.path());
        // Commit the fixture writes so the cycle does not see them as a dirty
        // tree (which would short-circuit into DirtyTree recovery).
        commit_repo(temp_dir.path(), "fixture state");

        // First run: nothing stalled yet. Capture the baseline fingerprint
        // without hard-coding it so the test stays robust against fingerprint
        // formatting changes.
        let baseline =
            evaluate_source_port_frontier_cycle(temp_dir.path(), 1, 1).expect("cycle should build");
        let baseline_task = baseline.selected_task.expect("selected task should exist");
        assert_eq!(
            baseline_task.delegated_goal,
            Some(super::AutonomyGoal::SweepParity),
            "fixture must dispatch on SweepParity"
        );
        assert_eq!(
            baseline_task.delegated_target.as_deref(),
            Some("state-sweep"),
            "fixture must target state-sweep"
        );
        assert!(
            baseline_task.recovery_mode.is_none(),
            "first run has no stall history yet"
        );
        let stalled_fingerprint = baseline_task.progress_fingerprint.clone();

        // Seed two consecutive stalled_repeat rows on the baseline fingerprint.
        write_stalled_repeat(temp_dir.path(), 1, "state-sweep", &stalled_fingerprint);
        write_stalled_repeat(temp_dir.path(), 2, "state-sweep", &stalled_fingerprint);

        // Third cycle: repeated_stalls = 2, recovery dispatch must fire on the
        // SweepParity arm. With no case-queue recording in the fixture the
        // arm falls through to apply_stalled_recovery + RefreshEvidence.
        let cycle =
            evaluate_source_port_frontier_cycle(temp_dir.path(), 3, 1).expect("cycle should build");
        let task = cycle.selected_task.expect("selected task should exist");
        assert_eq!(
            task.recovery_mode,
            Some(super::RecoveryMode::StalledFingerprint),
            "SweepParity arm must mark the package as recovery after two stalled cycles"
        );
        assert_eq!(
            task.recovery_step,
            Some(super::RecoveryStep::RefreshEvidence),
            "SweepParity arm without a recording should pivot to refresh-evidence"
        );
        assert_eq!(task.stalled_repetition_count, Some(3));
        assert!(
            task.progress_fingerprint
                .contains(":recovery:refresh_evidence"),
            "fingerprint should advance with the recovery suffix so the runner stops counting it as the same lane: {}",
            task.progress_fingerprint
        );
    }

    #[test]
    fn clean_repeat_of_recent_completion_triggers_recovery_on_iteration_1() {
        // Reproduce the user's "2 silent iterations wasted" pattern: the
        // planner would dispatch byte-identical work because the last clean
        // completion on this lane used exactly the same (target, fingerprint)
        // pair. No stalled_repeat rows have accumulated yet. Mechanism A must
        // lift the recovery trigger on iteration 1 instead of waiting for the
        // runner's `repeated_stalls >= 2` cascade to fire after two silent
        // retries.
        let temp_dir = source_port_repo();
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
        write_complete_default_mutation_runs(temp_dir.path());
        commit_repo(temp_dir.path(), "fixture state");

        // Capture the baseline fingerprint that the planner would emit.
        let baseline =
            evaluate_source_port_frontier_cycle(temp_dir.path(), 1, 1).expect("cycle should build");
        let baseline_task = baseline.selected_task.expect("selected task should exist");
        assert_eq!(
            baseline_task.delegated_target.as_deref(),
            Some("state-sweep"),
        );
        assert!(baseline_task.recovery_mode.is_none());
        let plain_fingerprint = baseline_task.progress_fingerprint.clone();

        // Seed exactly one clean-completion row with that (target, fingerprint)
        // pair — no stalled_repeat rows. This is the cycle 466/470 state the
        // user hit: the last run was clean, the planner reselects the same
        // lane.
        write_clean_completion_with_fingerprint(
            temp_dir.path(),
            10,
            "state-sweep",
            &plain_fingerprint,
        );

        // Iteration 1 on the next `rl` call: Mechanism A must fire without
        // waiting for two silent stalled_repeat retries.
        let cycle = evaluate_source_port_frontier_cycle(temp_dir.path(), 11, 1)
            .expect("cycle should build");
        let task = cycle.selected_task.expect("selected task should exist");
        assert_eq!(
            task.recovery_mode,
            Some(super::RecoveryMode::StalledFingerprint),
            "clean repeat must trigger the recovery cascade on iteration 1"
        );
        assert!(
            task.progress_fingerprint.contains(":recovery:"),
            "recovery suffix should be applied: {}",
            task.progress_fingerprint
        );
    }

    #[test]
    fn recovery_suffix_repeating_clean_completion_marks_definitively_stalled() {
        // Mechanism B: the recovery cascade's decoration is deterministic
        // (`:recovery:<step>`). If both the plain fingerprint AND the
        // suffixed fingerprint already appear as recent clean completions,
        // the cascade can only restate work that already landed. Mark the
        // task so the runner short-circuits to the operator message on
        // iteration 1 instead of looping through another three silent
        // retries.
        let temp_dir = source_port_repo();
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
        write_complete_default_mutation_runs(temp_dir.path());
        commit_repo(temp_dir.path(), "fixture state");

        let baseline =
            evaluate_source_port_frontier_cycle(temp_dir.path(), 1, 1).expect("cycle should build");
        let baseline_task = baseline.selected_task.expect("selected task should exist");
        let plain_fingerprint = baseline_task.progress_fingerprint.clone();
        let suffixed_fingerprint = format!("{plain_fingerprint}:recovery:refresh_evidence");

        // Seed both: the plain fingerprint AND the recovery-suffixed
        // fingerprint as recent clean completions. The first row triggers
        // Mechanism A. The second means the recovery suffix also repeats
        // completed work, so Mechanism B must mark the task.
        write_clean_completion_with_fingerprint(
            temp_dir.path(),
            10,
            "state-sweep",
            &plain_fingerprint,
        );
        write_clean_completion_with_fingerprint(
            temp_dir.path(),
            11,
            "state-sweep",
            &suffixed_fingerprint,
        );

        let cycle = evaluate_source_port_frontier_cycle(temp_dir.path(), 12, 1)
            .expect("cycle should build");
        let task = cycle.selected_task.expect("selected task should exist");
        assert_eq!(
            task.progress_fingerprint, suffixed_fingerprint,
            "recovery cascade should land on the same suffixed fingerprint"
        );
        assert!(
            task.definitively_stalled,
            "task should be marked definitively stalled when the recovery suffix also repeats a recent clean completion"
        );
    }

    #[test]
    fn sweep_parity_fingerprint_advances_when_gate_evidence_refreshes() {
        let temp_dir = source_port_repo();
        let gate_dir = temp_dir.path().join(".cache/release-gate/1p-accurate");
        let metrics_dir = temp_dir.path().join(".cache/metrics");
        fs::create_dir_all(&gate_dir).expect("release gate dir should exist");
        fs::create_dir_all(&metrics_dir).expect("metrics dir should exist");
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
            gate_dir.join("latest.json"),
            r#"{"timestamp":"1776000000","passed":true}"#,
        )
        .expect("release gate should write");
        fs::write(
            metrics_dir.join("sweep-history.jsonl"),
            r#"{"timestamp":"1776000001","tier":"state","levels_requested":69,"passed":69,"failed":0,"errors":0,"divergence_summary":{"authoritative":0,"advisory":178,"accumulation":0,"rng":0,"oracle_limitation":662},"report_path":"/tmp/sweep-report-old.json","field_summary":[],"release_gate_focus":[]}
"#,
        )
        .expect("sweep history should write");
        write_complete_default_mutation_runs(temp_dir.path());
        commit_repo(temp_dir.path(), "fixture state");

        let baseline =
            evaluate_source_port_frontier_cycle(temp_dir.path(), 1, 1).expect("cycle should build");
        let baseline_task = baseline.selected_task.expect("selected task should exist");
        assert!(
            baseline_task
                .progress_fingerprint
                .contains("state-sweep:gate-1776000000:sweep-1776000001"),
            "sweep lane fingerprint should include persisted evidence freshness: {}",
            baseline_task.progress_fingerprint
        );
        write_stalled_repeat(
            temp_dir.path(),
            1,
            "state-sweep",
            &baseline_task.progress_fingerprint,
        );
        write_stalled_repeat(
            temp_dir.path(),
            2,
            "state-sweep",
            &baseline_task.progress_fingerprint,
        );

        fs::write(
            gate_dir.join("latest.json"),
            r#"{"timestamp":"1776000100","passed":true}"#,
        )
        .expect("release gate should refresh");
        fs::write(
            metrics_dir.join("sweep-history.jsonl"),
            r#"{"timestamp":"1776000101","tier":"state","levels_requested":69,"passed":69,"failed":0,"errors":0,"divergence_summary":{"authoritative":0,"advisory":178,"accumulation":0,"rng":0,"oracle_limitation":662},"report_path":"/tmp/sweep-report-new.json","field_summary":[],"release_gate_focus":[]}
"#,
        )
        .expect("sweep history should refresh");
        commit_repo(temp_dir.path(), "refresh sweep evidence");

        let refreshed =
            evaluate_source_port_frontier_cycle(temp_dir.path(), 3, 1).expect("cycle should build");
        let refreshed_task = refreshed.selected_task.expect("selected task should exist");
        assert_eq!(
            refreshed_task.recovery_step, None,
            "fresh release-gate and state-sweep evidence should break the old stalled fingerprint"
        );
        assert!(
            refreshed_task
                .progress_fingerprint
                .contains("state-sweep:gate-1776000100:sweep-1776000101"),
            "refreshed fingerprint should carry new evidence timestamps: {}",
            refreshed_task.progress_fingerprint
        );
    }

    #[test]
    fn source_port_frontier_prioritises_dirty_tree_recovery() {
        let temp_dir = source_port_repo();
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
        fs::write(
            temp_dir.path().join("docs/source-port/methodology.md"),
            "dirty change\n",
        )
        .expect("dirty methodology change should write");

        let cycle =
            evaluate_source_port_frontier_cycle(temp_dir.path(), 1, 1).expect("cycle should build");
        let task = cycle.selected_task.expect("selected task should exist");

        assert_eq!(task.recovery_mode, Some(super::RecoveryMode::DirtyTree));
        assert_eq!(
            task.recovery_step,
            Some(super::RecoveryStep::FinishDirtyWork)
        );
        assert!(
            task.summary
                .contains("Resolve the current tracked work first")
        );
        assert!(
            task.evidence
                .iter()
                .any(|item| item.label == "dirty-tree" && item.detail.contains("methodology.md"))
        );
    }

    #[test]
    fn source_port_frontier_prioritises_ready_backlog_before_loc_coverage() {
        let temp_dir = source_port_repo();
        write_source_port_frontier(temp_dir.path(), false, "complete");
        fs::write(
            temp_dir.path().join(super::SOURCE_PORT_BACKLOG_PATH),
            "\
| # | Status | Pri | System | Summary | Verify |
|-----|--------|-----|--------|---------|--------|
| 144 | [ ] | P1 | enemy | Level 2 shooter enemy fires too rapidly. Root cause identified. | verify |
",
        )
        .expect("backlog should write");
        fs::write(
            temp_dir.path().join("docs/source-port/asm-coverage.yaml"),
            "generated_at: now\nsummary: { total_sub_labels: 0, total_loc_labels: 1, sub_cited: 0, sub_coverage_pct: 0.0, loc_cited: 0, loc_coverage_pct: 0.0 }\nsubsystems:\n  kid_main:\n    asm_source: kid.asm\n    sub_total: 0\n    sub_cited: 0\n    loc_total: 1\n    loc_cited: 0\n    labels:\n      - { label: loc_1295E, kind: loc, asm_file: kid.asm, line_number: 1, cited_in: [] }\n",
        )
        .expect("coverage should write");
        commit_repo(temp_dir.path(), "prepare ready backlog repo");

        let cycle =
            evaluate_source_port_frontier_cycle(temp_dir.path(), 1, 1).expect("cycle should build");
        let task = cycle.selected_task.expect("selected task should exist");

        assert_eq!(task.delegated_target.as_deref(), Some("144:ready"));
        assert!(
            task.title.contains("Implement ready backlog issue"),
            "ready backlog issue should beat loc coverage"
        );
        assert!(
            cycle
                .progress_fingerprint
                .contains("ready-backlog:144:ready")
        );
    }

    #[test]
    fn open_backlog_selection_skips_fresh_negative_repro_before_high_value_issue() {
        let temp_dir = source_port_repo();
        fs::write(
            temp_dir.path().join(super::SOURCE_PORT_BACKLOG_PATH),
            "\
| # | Status | Pri | System | Summary | Verify |
|-----|--------|-----|--------|---------|--------|
| 137 | [ ] | P1 | player | Kid walks backwards while climbing slopes | Native repro status: not reproduced. Fresh negative baseline recorded. |
| 138 | [ ] | P1 | player | Enemy contact appears to apply double damage because invulnerability is too short | Take consecutive hits, confirm post-hit invulnerability timing matches oracle |
",
        )
        .expect("backlog should write");

        let issue = super::select_open_backlog_issue(temp_dir.path())
            .expect("selection should succeed")
            .expect("issue should be selected");

        assert_eq!(issue.id, "138");
    }

    #[test]
    fn open_backlog_selection_moves_past_completed_negative_evidence_targets() {
        let temp_dir = source_port_repo();
        fs::write(
            temp_dir.path().join(super::SOURCE_PORT_BACKLOG_PATH),
            "\
| # | Status | Pri | System | Summary | Verify |
|-----|--------|-----|--------|---------|--------|
| 137 | [ ] | P1 | player | Kid walks backwards while climbing slopes | Native repro status: not reproduced. Fresh negative baseline recorded. |
| 138 | [ ] | P1 | player | Enemy contact appears to apply double damage because invulnerability is too short | Native repro status: not reproduced. Fresh negative baseline recorded. |
| 139 | [ ] | P1 | render | Bouncy blocks still render like spikes | Compare rubber block tiles and state art against oracle frames |
| 143 | [ ] | P1 | block | Heavy Man's first helmet causes repeated block-bounce instead of the ROM's smooth single-hit break | Native repro status: not reproduced. Fresh negative baseline recorded. |
",
        )
        .expect("backlog should write");

        let issue = super::select_open_backlog_issue(temp_dir.path())
            .expect("selection should succeed")
            .expect("issue should be selected");

        assert_eq!(issue.id, "139");
    }

    #[test]
    fn open_backlog_selection_treats_legacy_clean_issue_id_as_capture_exhausted() {
        let temp_dir = source_port_repo();
        fs::write(
            temp_dir.path().join(super::SOURCE_PORT_BACKLOG_PATH),
            "\
| # | Status | Pri | System | Summary | Verify |
|-----|--------|-----|--------|---------|--------|
| 138 | [ ] | P1 | player | Enemy contact appears to apply double damage because invulnerability is too short | Take consecutive hits, confirm post-hit invulnerability timing matches oracle |
| 141 | [ ] | P1 | render | Intro and level-intro graphics still do not match the original game | Native/oracle repro status: reproduced for boot/intro visual parity. |
",
        )
        .expect("backlog should write");
        write_clean_completion(temp_dir.path(), 217, "138");

        let issue = super::select_open_backlog_issue(temp_dir.path())
            .expect("selection should succeed")
            .expect("issue should be selected");

        assert_eq!(issue.id, "141");
        assert_eq!(
            super::backlog_issue_progress_marker(&issue),
            "141:investigate"
        );
    }

    #[test]
    #[allow(unsafe_code)]
    fn recovery_trace_emits_jsonl_record_when_env_var_set() {
        let temp_dir = source_port_repo();
        write_source_port_frontier(temp_dir.path(), false, "complete");
        write_backlog(
            temp_dir.path(),
            "| 141 | [ ] | P1 | render | Intro and level-intro graphics still do not match the original game | Native/oracle repro status: reproduced for boot/intro visual parity. |\n",
        );
        write_complete_default_mutation_runs(temp_dir.path());

        let trace_path = temp_dir.path().join(super::SOURCE_PORT_RECOVERY_TRACE_PATH);
        assert!(!trace_path.exists());

        // SAFETY: tests are single-process; the env toggle is read by the cycle
        // we are about to invoke, then verified once via the on-disk trace file.
        unsafe {
            std::env::set_var(super::SOURCE_PORT_RECOVERY_TRACE_ENV, "1");
        }
        let result = evaluate_source_port_frontier_cycle(temp_dir.path(), 99, 1);
        unsafe {
            std::env::remove_var(super::SOURCE_PORT_RECOVERY_TRACE_ENV);
        }
        result.expect("cycle should build");

        let raw = fs::read_to_string(&trace_path).expect("trace file should exist");
        let line = raw.lines().next().expect("trace should have a line");
        let record: serde_json::Value =
            serde_json::from_str(line).expect("trace line should parse as JSON");
        for key in [
            "timestamp",
            "cycle",
            "delegated_goal",
            "delegated_target",
            "base_progress_fingerprint",
            "dirty_files_len",
            "repeated_stalls",
            "chose_branch",
            "recovery_mode_out",
            "recovery_step_out",
            "final_progress_fingerprint",
        ] {
            assert!(
                record.get(key).is_some(),
                "trace record missing field {key}: {record}"
            );
        }
        assert_eq!(record["cycle"], serde_json::json!(99));
        // No prior stalled_repeat history → recovery dispatch not entered.
        assert_eq!(record["chose_branch"], serde_json::json!("none"));
        assert_eq!(record["repeated_stalls"], serde_json::json!(0));
    }

    #[test]
    fn reproduced_backlog_issue_switches_from_capture_to_investigation_phase() {
        let temp_dir = source_port_repo();
        write_source_port_frontier(temp_dir.path(), false, "complete");
        write_backlog(
            temp_dir.path(),
            "| 141 | [ ] | P1 | render | Intro and level-intro graphics still do not match the original game | Native/oracle repro status: reproduced for boot/intro visual parity. |\n",
        );
        write_complete_default_mutation_runs(temp_dir.path());
        // Exhaust the gameplay-bug investigation lane so the old
        // capture-vs-investigate fallback fires in this test.
        write_clean_completion(temp_dir.path(), 0, "141:investigate-bug");

        let cycle =
            evaluate_source_port_frontier_cycle(temp_dir.path(), 1, 1).expect("cycle should build");
        let task = cycle.selected_task.expect("selected task should exist");

        assert_eq!(task.delegated_target.as_deref(), Some("141:investigate"));
        assert!(task.title.contains("Investigate reproduced backlog issue"));
        assert!(
            !task
                .commands
                .iter()
                .any(|command| command.contains("play-native --record issue-141")),
            "reproduced evidence should not launch the same capture command again"
        );
        assert!(
            cycle
                .progress_fingerprint
                .contains("backlog-issue:141:investigate")
        );
    }

    #[test]
    fn source_port_frontier_skips_recent_clean_completed_backlog_target() {
        let temp_dir = source_port_repo();
        write_source_port_frontier(temp_dir.path(), false, "complete");
        write_backlog(
            temp_dir.path(),
            "\
| 141 | [ ] | P1 | render | Intro and level-intro graphics still do not match the original game | Native/oracle repro status: reproduced for boot/intro visual parity. |
| 139 | [ ] | P1 | render | Bouncy blocks still render like spikes | Compare rubber block tiles and state art against oracle frames |
",
        );
        write_complete_default_mutation_runs(temp_dir.path());
        write_clean_completion(temp_dir.path(), 41, "141:investigate");
        // Exhaust the gameplay-bug investigation lane so the old
        // open-backlog fallback fires in this test.
        write_clean_completion(temp_dir.path(), 39, "141:investigate-bug");
        write_clean_completion(temp_dir.path(), 40, "139:investigate-bug");

        let cycle = evaluate_source_port_frontier_cycle(temp_dir.path(), 42, 1)
            .expect("cycle should build");
        let task = cycle.selected_task.expect("selected task should exist");

        assert_ne!(task.delegated_target.as_deref(), Some("141:investigate"));
        assert_eq!(task.delegated_target.as_deref(), Some("139:capture"));
        assert!(
            cycle
                .progress_fingerprint
                .contains("backlog-issue:139:capture")
        );
    }

    #[test]
    fn source_port_frontier_pivots_when_backlog_targets_are_recently_completed() {
        let temp_dir = source_port_repo();
        write_source_port_frontier(temp_dir.path(), false, "complete");
        write_backlog(
            temp_dir.path(),
            "\
| 141 | [ ] | P1 | render | Intro and level-intro graphics still do not match the original game | Native/oracle repro status: reproduced for boot/intro visual parity. |
",
        );
        write_complete_default_mutation_runs(temp_dir.path());
        write_clean_completion(temp_dir.path(), 41, "141:investigate");
        // Exhaust the gameplay-bug investigation lane so the old
        // evidence-blocker fallback fires in this test.
        write_clean_completion(temp_dir.path(), 40, "141:investigate-bug");

        let cycle = evaluate_source_port_frontier_cycle(temp_dir.path(), 42, 1)
            .expect("cycle should build");
        let task = cycle.selected_task.expect("selected task should exist");

        assert_eq!(
            task.delegated_target.as_deref(),
            Some(super::SOURCE_PORT_BACKLOG_EVIDENCE_BLOCKER_TARGET)
        );
        assert!(
            task.title.contains("Repair backlog evidence blocker"),
            "selector should emit useful blocker work instead of reselecting the clean-completed issue"
        );
        assert!(
            cycle
                .progress_fingerprint
                .contains(super::SOURCE_PORT_BACKLOG_EVIDENCE_BLOCKER_TARGET)
        );
    }

    #[test]
    fn source_port_frontier_promotes_clean_narrowed_backlog_to_ready_work() {
        let temp_dir = source_port_repo();
        write_source_port_frontier(temp_dir.path(), false, "complete");
        write_backlog(
            temp_dir.path(),
            "\
| 141 | [ ] | P1 | render | Intro and level-intro graphics still do not match the original game | Root cause identified: native IntroSequence2/4/5 VDP loading currently reuses ASM sub_1B850; split loading to follow Load_IntroSequence2, Load_IntroSequence4, and Load_IntroSequence5. Native/oracle repro status: reproduced for boot/intro visual parity. |
",
        );
        write_complete_default_mutation_runs(temp_dir.path());
        write_clean_completion(temp_dir.path(), 41, "141:investigate");

        let cycle = evaluate_source_port_frontier_cycle(temp_dir.path(), 42, 1)
            .expect("cycle should build");
        let task = cycle.selected_task.expect("selected task should exist");

        assert_eq!(task.delegated_target.as_deref(), Some("141:ready"));
        assert!(
            task.title.contains("Implement ready backlog issue #141"),
            "narrowed root-cause evidence should promote to implementation instead of the shared evidence blocker"
        );
        assert!(
            cycle
                .progress_fingerprint
                .contains("ready-backlog:141:ready")
        );
    }

    #[test]
    fn source_port_frontier_ready_work_ignores_legacy_clean_raw_issue_id() {
        let temp_dir = source_port_repo();
        write_source_port_frontier(temp_dir.path(), false, "complete");
        write_backlog(
            temp_dir.path(),
            "\
| 141 | [ ] | P1 | render | Intro and level-intro graphics still do not match the original game | Root cause identified: native IntroSequence2/4/5 VDP loading currently reuses ASM sub_1B850; split loading to follow Load_IntroSequence2, Load_IntroSequence4, and Load_IntroSequence5. |
",
        );
        write_clean_completion(temp_dir.path(), 321, "141");

        let cycle = evaluate_source_port_frontier_cycle(temp_dir.path(), 322, 1)
            .expect("cycle should build");
        let task = cycle.selected_task.expect("selected task should exist");

        assert_eq!(task.delegated_target.as_deref(), Some("141:ready"));
        assert!(task.title.contains("Implement ready backlog issue #141"));
    }

    #[test]
    fn proactive_root_cause_strategy_reads_verify_column() {
        let temp_dir = source_port_repo();
        write_backlog(
            temp_dir.path(),
            "\
| 141 | [ ] | P1 | render | Intro and level-intro graphics still do not match the original game | Root cause identified: native IntroSequence2/4/5 VDP loading currently reuses ASM sub_1B850; split loading to follow Load_IntroSequence2, Load_IntroSequence4, and Load_IntroSequence5. |
",
        );

        let report = super::evaluate_proactive_verification_cycle(temp_dir.path(), 7, 1)
            .expect("cycle should build");
        let task = report.selected_task.expect("selected task should exist");

        assert_eq!(
            report.progress_fingerprint,
            "proactive-verification:root-cause-backlog:141:ready"
        );
        assert!(task.title.contains("Fix backlog #141"));
    }

    #[test]
    fn source_port_frontier_prioritises_release_gate_blocker_before_proactive_verification() {
        let temp_dir = source_port_repo();
        write_source_port_frontier(temp_dir.path(), false, "complete");
        fs::create_dir_all(temp_dir.path().join(".cache/release-gate/1p-playable"))
            .expect("gate dir should exist");
        fs::write(
            temp_dir
                .path()
                .join(".cache/release-gate/1p-playable/latest.json"),
            r#"{
  "passed": false,
  "steps": [
    {
      "name": "native_gameplay:death_respawn_level1",
      "status": "failed",
      "detail": "death_respawn_level1 did not lose a life"
    }
  ]
}"#,
        )
        .expect("gate should write");
        commit_repo(temp_dir.path(), "prepare release gate blocker repo");

        let cycle =
            evaluate_source_port_frontier_cycle(temp_dir.path(), 1, 1).expect("cycle should build");
        let task = cycle.selected_task.expect("selected task should exist");

        assert!(
            task.title.contains("Repair failing release-gate step"),
            "release gate blocker should beat proactive verification"
        );
        assert!(
            task.evidence
                .iter()
                .any(|item| item.label == "release-gate-failure"),
            "task should surface the failing gate evidence"
        );
    }

    #[test]
    fn source_port_frontier_proactive_delegation_carries_handoff_context() {
        let temp_dir = source_port_repo();
        write_source_port_frontier(temp_dir.path(), false, "complete");

        let cycle =
            evaluate_source_port_frontier_cycle(temp_dir.path(), 1, 1).expect("cycle should build");
        let task = cycle.selected_task.expect("selected task should exist");

        assert_eq!(task.capability, super::CapabilityId::SourcePortFrontierTask);
        assert_eq!(
            task.delegated_goal,
            Some(super::AutonomyGoal::ProactiveVerification)
        );
        assert!(
            task.title.contains("mutation testing"),
            "source-port should delegate to the proactive mutation lane"
        );
        assert!(
            task.frontier_context.is_some(),
            "delegated proactive task should still carry source-port frontier context"
        );
        assert!(
            task.next_agent_note.is_some(),
            "delegated proactive task should refresh the source-port handoff note"
        );
    }

    #[test]
    fn source_port_frontier_enters_recovery_after_repeated_sweep_stalls() {
        let temp_dir = source_port_repo();
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
        commit_repo(temp_dir.path(), "prepare clean sweep parity repo");
        write_complete_default_mutation_runs(temp_dir.path());
        let run_dir = temp_dir.path().join(".cache/autonomy/source-port-frontier");
        fs::create_dir_all(&run_dir).expect("run dir should exist");
        fs::write(
            run_dir.join("cycle-9999.json"),
            r#"{"progress_fingerprint":"stale-newer-cycle-file","stalled":false}"#,
        )
        .expect("stale cycle file should write");
        let baseline =
            evaluate_source_port_frontier_cycle(temp_dir.path(), 1, 1).expect("cycle should build");
        let baseline_task = baseline.selected_task.expect("selected task should exist");
        write_stalled_repeat(
            temp_dir.path(),
            1,
            "state-sweep",
            &baseline_task.progress_fingerprint,
        );
        write_stalled_repeat(
            temp_dir.path(),
            2,
            "state-sweep",
            &baseline_task.progress_fingerprint,
        );

        let cycle =
            evaluate_source_port_frontier_cycle(temp_dir.path(), 3, 1).expect("cycle should build");
        let task = cycle.selected_task.expect("selected task should exist");
        let frontier = task
            .frontier_context
            .as_ref()
            .expect("frontier context should exist");
        let rendered_note = render_source_port_next_agent_note(frontier, &task, &[]);

        assert_eq!(
            task.recovery_mode,
            Some(super::RecoveryMode::StalledFingerprint)
        );
        assert_eq!(
            task.recovery_step,
            Some(super::RecoveryStep::RefreshEvidence)
        );
        assert_eq!(task.stalled_repetition_count, Some(3));
        assert!(
            cycle
                .progress_fingerprint
                .contains("recovery:refresh_evidence")
        );
        assert!(
            rendered_note.contains(
                "Recovery mode `stalled_fingerprint` / step `refresh_evidence` after 3 repeated stalled cycles"
            ),
            "recovery note should surface the stalled recovery context"
        );
        assert!(
            rendered_note.contains("narrow the next run to release-gate evidence refresh instead of another broad lane replay"),
            "recovery note should surface the stalled recovery pivot reason"
        );
    }

    #[test]
    fn docs_only_sweep_refresh_streak_pivots_to_proactive_exhaustion_record() {
        let temp_dir = source_port_repo();
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
        write_complete_default_mutation_runs(temp_dir.path());
        commit_repo(temp_dir.path(), "prepare clean sweep parity repo");

        for index in 0..3 {
            fs::write(
                temp_dir.path().join("docs/source-port/methodology.md"),
                format!("# Methodology\n\nrefresh {index}\n"),
            )
            .expect("methodology doc should write");
            fs::write(
                temp_dir.path().join("docs/source-port/research-map.md"),
                format!("# Research Map\n\nrefresh {index}\n"),
            )
            .expect("research map should write");
            commit_repo(temp_dir.path(), "docs: refresh sweep parity evidence");
        }

        let cycle =
            evaluate_source_port_frontier_cycle(temp_dir.path(), 5, 1).expect("cycle should build");
        let task = cycle.selected_task.expect("selected task should exist");
        let frontier = task
            .frontier_context
            .as_ref()
            .expect("frontier context should exist");
        let rendered_note = render_source_port_next_agent_note(frontier, &task, &[]);

        assert_eq!(
            task.recovery_mode,
            Some(super::RecoveryMode::StalledFingerprint)
        );
        assert_eq!(
            task.recovery_step,
            Some(super::RecoveryStep::PivotToNextLane)
        );
        assert_eq!(
            task.delegated_goal,
            Some(super::AutonomyGoal::ProactiveVerification),
            "docs-only sweep loops should stop delegating another sweep refresh when proactive-verification is exhausted"
        );
        assert_eq!(task.delegated_target.as_deref(), Some("all-exhausted"));
        assert!(
            task.implementation_targets
                .iter()
                .any(|path| path == SOURCE_PORT_FRONTIER_STATE_PATH),
            "exhausted pivot should ask the agent to record the stall in frontier-state"
        );
        assert!(
            !task
                .implementation_targets
                .iter()
                .all(|path| super::SWEEP_REFRESH_DOC_PATHS.contains(&path.as_str())),
            "exhausted pivot must not be another docs-only sweep refresh"
        );
        assert!(
            rendered_note
                .contains("Target `all-exhausted` for the next concrete source-port step."),
            "handoff should make the exhausted proactive target visible"
        );
        assert!(
            rendered_note.contains("Record the exhausted proactive-verification pivot"),
            "handoff should explain why the lane is no longer another sweep refresh"
        );
    }

    #[test]
    fn collect_loc_labels_separates_uncovered_from_test_only_promotion() {
        let temp_dir = source_port_repo();
        fs::write(
            temp_dir.path().join("docs/source-port/asm-coverage.yaml"),
            "generated_at: now\nsummary: { total_sub_labels: 0, total_loc_labels: 2, sub_cited: 0, sub_coverage_pct: 0.0, loc_cited: 1, loc_coverage_pct: 50.0 }\nsubsystems:\n  kid_main:\n    asm_source: kid.asm\n    sub_total: 0\n    sub_cited: 0\n    loc_total: 2\n    loc_cited: 1\n    labels:\n      - { label: loc_1111, kind: loc, asm_file: kid.asm, line_number: 1, cited_in: [\"crates/kid-core/src/tests.rs\"], implementation_cited: false, test_cited: true }\n      - { label: loc_2222, kind: loc, asm_file: kid.asm, line_number: 2, cited_in: [], implementation_cited: false, test_cited: false }\n",
        )
        .expect("coverage should write");

        let uncovered =
            super::collect_loc_labels(temp_dir.path(), super::LocCoverageSelection::Uncovered)
                .expect("uncovered labels should load");
        let promotion =
            super::collect_loc_labels(temp_dir.path(), super::LocCoverageSelection::Promotion)
                .expect("promotion labels should load");

        assert_eq!(uncovered.len(), 1);
        assert_eq!(uncovered[0].label, "loc_2222");
        assert_eq!(promotion.len(), 1);
        assert_eq!(promotion[0].label, "loc_1111");
    }

    #[test]
    fn loc_coverage_drive_promotes_test_only_labels_when_main_queue_is_empty() {
        let temp_dir = source_port_repo();
        fs::write(
            temp_dir.path().join("docs/source-port/asm-coverage.yaml"),
            "generated_at: now\nsummary: { total_sub_labels: 0, total_loc_labels: 1, sub_cited: 0, sub_coverage_pct: 0.0, loc_cited: 1, loc_coverage_pct: 100.0 }\nsubsystems:\n  kid_main:\n    asm_source: kid.asm\n    sub_total: 0\n    sub_cited: 0\n    loc_total: 1\n    loc_cited: 1\n    labels:\n      - { label: loc_1BBD6, kind: loc, asm_file: kid.asm, line_number: 30463, cited_in: [\"crates/kid-core/src/tests.rs\"], implementation_cited: false, test_cited: true }\n",
        )
        .expect("coverage should write");

        let report =
            super::try_loc_coverage_drive_strategy(temp_dir.path(), 7, 3, 1, &Vec::<String>::new())
                .expect("strategy should succeed")
                .expect("promotion report should exist");
        let task = report.selected_task.expect("selected task should exist");

        assert!(
            task.title
                .contains("Promote 1 test-cited loc_ labels in kid_main")
        );
        assert_eq!(
            report.progress_fingerprint,
            "proactive-verification:loc-coverage-promotion:loc_1BBD6"
        );
        assert!(
            task.evidence
                .iter()
                .any(|item| item.label == "loc-coverage-promotion")
        );
    }

    #[test]
    fn mutation_testing_strategy_uses_cargo_mutants_out_path_and_forces_gitignore() {
        let temp_dir = source_port_repo();
        let targets = vec![mutation_target("enemies.rs", 1)];

        let report = super::try_mutation_testing_strategy(
            temp_dir.path(),
            4,
            ".cache/mutants",
            &targets,
            &Vec::<String>::new(),
            None,
        )
        .expect("mutation testing task should exist");
        let task = report.selected_task.expect("selected task should exist");

        assert_eq!(
            report.progress_fingerprint,
            "proactive-verification:mutation-testing:enemies:run"
        );
        assert!(
            task.commands
                .iter()
                .any(|command| command.contains("--gitignore=true")
                    && command.contains("--output .cache/mutants/enemies")),
            "generated cargo-mutants command should force gitignore and use the module output dir"
        );
        assert!(
            task.done_criteria
                .iter()
                .any(|criterion| criterion
                    .contains(".cache/mutants/enemies/mutants.out/outcomes.json")),
            "done criteria should name cargo-mutants' real outcomes path"
        );
    }

    #[test]
    fn mutation_testing_strategy_reruns_incomplete_mutants_out() {
        let temp_dir = source_port_repo();
        write_mutation_run(
            temp_dir.path(),
            "enemies",
            10,
            2,
            "",
            "",
            "ERROR cargo_mutants::lab: Worker thread failed: No space left on device\n",
        );
        let targets = vec![mutation_target("enemies.rs", 1)];

        let report = super::try_mutation_testing_strategy(
            temp_dir.path(),
            4,
            ".cache/mutants",
            &targets,
            &Vec::<String>::new(),
            None,
        )
        .expect("rerun task should exist");
        let task = report.selected_task.expect("selected task should exist");

        assert!(
            report
                .progress_fingerprint
                .starts_with("proactive-verification:mutation-testing:enemies:rerun:"),
            "incomplete results should get a rerun-specific fingerprint"
        );
        assert!(task.title.contains("Rerun incomplete mutation testing"));
        assert!(
            task.commands
                .iter()
                .any(|command| command.contains("rm -rf .cache/mutants/enemies/mutants.out")),
            "rerun task should clean partial cargo-mutants output first"
        );
        assert!(
            task.commands
                .iter()
                .any(|command| command.contains("--gitignore=true")),
            "rerun command should force gitignore handling"
        );
        assert!(
            task.evidence
                .iter()
                .any(|item| item.label == "mutation-testing-incomplete"
                    && item.status == "incomplete"
                    && item.detail.contains("No space left on device")),
            "task should surface the interrupted cargo-mutants evidence"
        );
    }

    #[test]
    fn mutation_testing_strategy_triages_survivors_from_complete_run() {
        let temp_dir = source_port_repo();
        write_mutation_run(
            temp_dir.path(),
            "enemies",
            2,
            2,
            "crates/kid-core/src/enemies.rs:48:19: replace > with < in ninja_walk_speed_override_loc_3d59a_and_3d5aa\n",
            "",
            "",
        );
        let targets = vec![mutation_target("enemies.rs", 1)];

        let report = super::try_mutation_testing_strategy(
            temp_dir.path(),
            4,
            ".cache/mutants",
            &targets,
            &Vec::<String>::new(),
            None,
        )
        .expect("triage task should exist");
        let task = report.selected_task.expect("selected task should exist");

        assert!(
            report
                .progress_fingerprint
                .starts_with("proactive-verification:mutation-testing:enemies:triage:"),
            "survivors should get a triage-specific fingerprint"
        );
        assert!(task.title.contains("Triage surviving mutation results"));
        assert!(
            task.commands
                .iter()
                .any(|command| command.contains("missed.txt")),
            "triage task should point the agent at missed mutants"
        );
        assert!(
            task.evidence
                .iter()
                .any(|item| item.label == "mutation-testing-survivors"
                    && item.status == "needs-triage"
                    && item.detail.contains("1 missed")),
            "task should summarise remaining survivors"
        );
    }

    #[test]
    fn mutation_testing_strategy_skips_complete_modules_without_survivors() {
        let temp_dir = source_port_repo();
        write_mutation_run(temp_dir.path(), "enemies", 1, 1, "", "", "");
        let targets = vec![
            mutation_target("enemies.rs", 1),
            mutation_target("combat.rs", 1),
        ];

        let report = super::try_mutation_testing_strategy(
            temp_dir.path(),
            4,
            ".cache/mutants",
            &targets,
            &Vec::<String>::new(),
            None,
        )
        .expect("next module task should exist");
        let task = report.selected_task.expect("selected task should exist");

        assert_eq!(
            report.progress_fingerprint,
            "proactive-verification:mutation-testing:combat:run"
        );
        assert!(
            task.title.contains("combat.rs"),
            "complete modules with no survivors should be skipped"
        );
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
    fn case_queue_blocks_recording_cases_with_source_drift() {
        let temp_dir = source_port_repo();
        write_blocked_case(
            temp_dir.path(),
            "recording-case",
            r#"{"kind":"recording","path":".cache/recordings/tree/tree","slug":"tree-break"}"#,
            "source recording replay drifted at frame 0",
            0,
            60,
        );

        let queue = build_case_queue_snapshot(temp_dir.path()).expect("queue should build");

        assert!(queue.selected_case.is_none());
        assert_eq!(queue.blocked_cases.len(), 1);
        assert_eq!(queue.blocked_cases[0].case_id, "recording-case");
        assert_eq!(queue.blocked_cases[0].status, CaseQueueStatus::Blocked);
        assert_eq!(queue.blocked_cases[0].failure_class, None);
    }

    #[test]
    fn case_queue_lists_untriaged_recordings() {
        let temp_dir = source_port_repo();
        write_recording(temp_dir.path(), "level1-tree-break", true);

        let queue = build_case_queue_snapshot(temp_dir.path()).expect("queue should build");
        assert_eq!(queue.untriaged_recordings.len(), 1);
        assert_eq!(queue.untriaged_recordings[0].slug, "level1-tree-break");
        assert_eq!(queue.untriaged_recordings[0].frame_count, 180);
        assert!(queue.untriaged_recordings[0].has_transcript);
    }

    #[test]
    fn case_queue_selects_recording_when_no_failing_case_exists() {
        let temp_dir = source_port_repo();
        write_recording(temp_dir.path(), "level1-tree-break", true);

        let queue = build_case_queue_snapshot(temp_dir.path()).expect("queue should build");

        assert!(queue.selected_case.is_none());
        let recording = queue
            .selected_recording
            .expect("selected recording should exist");
        assert_eq!(recording.slug, "level1-tree-break");
        assert_eq!(recording.frame_count, 180);
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
            evaluate_source_port_frontier_cycle(temp_dir.path(), 1, 1).expect("cycle should build");
        let task = cycle.selected_task.expect("selected task should exist");

        assert_eq!(task.delegated_goal, Some(super::AutonomyGoal::CaseQueue));
        assert_eq!(task.delegated_target.as_deref(), Some("recording-case"));
        assert!(task.title.contains("recording-case"));
        assert!(cycle.progress_fingerprint.contains("case-queue"));
    }

    #[test]
    fn source_port_frontier_delegates_to_recording_derivation_before_sweep() {
        let temp_dir = source_port_repo();
        write_source_port_frontier(temp_dir.path(), false, "complete");
        write_recording(temp_dir.path(), "level1-tree-break", true);

        let cycle =
            evaluate_source_port_frontier_cycle(temp_dir.path(), 1, 1).expect("cycle should build");
        let task = cycle.selected_task.expect("selected task should exist");

        assert_eq!(task.delegated_goal, Some(super::AutonomyGoal::CaseQueue));
        assert_eq!(task.delegated_target.as_deref(), Some("level1-tree-break"));
        assert!(task.title.contains("level1-tree-break"));
        assert!(task.summary.contains("reduced gameplay case"));
        assert!(
            task.commands
                .iter()
                .any(|command| command.contains("case derive --recording-slug level1-tree-break"))
        );
        assert!(cycle.progress_fingerprint.contains("case-queue"));
    }

    #[test]
    fn source_port_frontier_delegates_to_backlog_issue_before_asm_fallback() {
        let temp_dir = source_port_repo();
        write_source_port_frontier(temp_dir.path(), false, "complete");
        write_backlog(
            temp_dir.path(),
            "| 146 | [ ] | P0 | level | Dying in Level 2 can leave the game stuck on the level-loading screen | verify |\n",
        );
        write_complete_default_mutation_runs(temp_dir.path());
        // Exhaust the gameplay-bug investigation lane so the older capture-evidence lane fires.
        write_clean_completion(temp_dir.path(), 0, "146:investigate-bug");

        let cycle =
            evaluate_source_port_frontier_cycle(temp_dir.path(), 1, 1).expect("cycle should build");
        let task = cycle.selected_task.expect("selected task should exist");

        assert_eq!(task.delegated_target.as_deref(), Some("146:capture"));
        assert!(task.title.contains("backlog issue #146"));
        assert!(
            task.commands
                .iter()
                .any(|command| command.contains("play-native --record issue-146"))
        );
        assert!(
            cycle
                .progress_fingerprint
                .contains("backlog-issue:146:capture")
        );
    }

    #[test]
    fn source_port_frontier_selects_gameplay_bug_investigation_when_pv_exhausted() {
        // With the gate green, PV exhausted (mutation runs complete), and at
        // least one open [ ] bug on the backlog, the loop must delegate to
        // gameplay-bug investigation instead of sweep-parity meta-work.
        let temp_dir = source_port_repo();
        write_source_port_frontier(temp_dir.path(), false, "complete");
        write_backlog(
            temp_dir.path(),
            "| 135 | [ ] | P1 | level | Level 1 background graphics drift | verify |\n",
        );
        write_complete_default_mutation_runs(temp_dir.path());

        let cycle =
            evaluate_source_port_frontier_cycle(temp_dir.path(), 1, 1).expect("cycle should build");
        let task = cycle.selected_task.expect("selected task should exist");

        assert_eq!(
            task.delegated_goal,
            Some(super::AutonomyGoal::GameplayBugInvestigation)
        );
        assert_ne!(task.delegated_goal, Some(super::AutonomyGoal::SweepParity));
        assert_eq!(
            task.delegated_target.as_deref(),
            Some("135:investigate-bug")
        );
        assert!(task.title.contains("#135"));
        assert!(
            cycle
                .progress_fingerprint
                .contains("bug-investigation:135:investigate-bug"),
            "fingerprint should reference bug-investigation lane, got: {}",
            cycle.progress_fingerprint
        );
    }

    #[test]
    fn gameplay_bug_investigation_selector_excludes_closed_blocked_stalled_items() {
        // [?] and [x] never reach the parser (parser only accepts [ ] / [~]);
        // [~] items with verify tagged `blocked:` or `stalled:` are filtered by
        // is_bug_investigation_candidate. Only the plain [ ] item survives.
        let temp_dir = source_port_repo();
        write_backlog(
            temp_dir.path(),
            "\
| 135 | [x] | P1 | level | Closed already | verify |\n\
| 136 | [?] | P1 | level | Awaiting human validation | verify |\n\
| 137 | [~] | P1 | level | Blocked bug | blocked:needs-human-repro |\n\
| 138 | [~] | P1 | level | Stalled bug | stalled:agent-blocked |\n\
| 139 | [ ] | P1 | level | Bouncy blocks wrong | verify |\n",
        );

        let exhausted = std::collections::BTreeSet::new();
        let attempts = std::collections::BTreeMap::new();
        let picked = super::select_open_bug_for_investigation(
            temp_dir.path(),
            &exhausted,
            &attempts,
            super::BUG_INVESTIGATION_ATTEMPT_CAP,
        )
        .expect("selector should run");

        let issue = picked.expect("plain open bug should be selected");
        assert_eq!(issue.id, "139");
    }

    #[test]
    fn gameplay_bug_investigation_selector_skips_exhausted_targets() {
        // When the issue's investigate-bug target has already been clean-
        // completed this window, the selector must skip it so we pivot away
        // instead of looping on the same handoff.
        let temp_dir = source_port_repo();
        write_backlog(
            temp_dir.path(),
            "| 135 | [ ] | P1 | level | Level 1 background graphics drift | verify |\n",
        );

        let mut exhausted = std::collections::BTreeSet::new();
        exhausted.insert("135:investigate-bug".to_owned());
        let attempts = std::collections::BTreeMap::new();
        let picked = super::select_open_bug_for_investigation(
            temp_dir.path(),
            &exhausted,
            &attempts,
            super::BUG_INVESTIGATION_ATTEMPT_CAP,
        )
        .expect("selector should run");

        assert!(picked.is_none(), "exhausted target must be skipped");
    }

    #[test]
    fn gameplay_bug_investigation_selector_skips_items_at_attempt_cap() {
        // Issues whose attempt counter has reached the cap are considered
        // agent-blocked and excluded until a human re-inspects.
        let temp_dir = source_port_repo();
        write_backlog(
            temp_dir.path(),
            "| 135 | [ ] | P1 | level | Level 1 background graphics drift | verify |\n",
        );

        let exhausted = std::collections::BTreeSet::new();
        let mut attempts = std::collections::BTreeMap::new();
        attempts.insert("135".to_owned(), super::BUG_INVESTIGATION_ATTEMPT_CAP);
        let picked = super::select_open_bug_for_investigation(
            temp_dir.path(),
            &exhausted,
            &attempts,
            super::BUG_INVESTIGATION_ATTEMPT_CAP,
        )
        .expect("selector should run");

        assert!(picked.is_none(), "issue at attempt cap must be skipped");
    }

    #[test]
    fn gameplay_bug_investigation_selector_sorts_p0_before_p1_before_p2() {
        // Priority drives selection order: P0 beats P1 beats P2. Tie-break by
        // id so behaviour is deterministic.
        let temp_dir = source_port_repo();
        write_backlog(
            temp_dir.path(),
            "\
| 138 | [ ] | P2 | level | Lower priority bug | verify |\n\
| 137 | [ ] | P1 | level | Mid priority bug | verify |\n\
| 136 | [ ] | P0 | level | Top priority bug | verify |\n",
        );

        let exhausted = std::collections::BTreeSet::new();
        let attempts = std::collections::BTreeMap::new();
        let picked = super::select_open_bug_for_investigation(
            temp_dir.path(),
            &exhausted,
            &attempts,
            super::BUG_INVESTIGATION_ATTEMPT_CAP,
        )
        .expect("selector should run");

        let issue = picked.expect("highest-priority bug should be selected");
        assert_eq!(issue.id, "136");
        assert_eq!(issue.priority, "P0");
    }

    #[test]
    fn source_port_frontier_falls_through_to_sweep_parity_when_no_open_bugs() {
        // No open [ ] / [~] bugs → gameplay-bug lane yields nothing, loop must
        // preserve existing sweep-parity / evidence-blocker fallback behaviour.
        let temp_dir = source_port_repo();
        write_source_port_frontier(temp_dir.path(), false, "complete");
        write_backlog(
            temp_dir.path(),
            "| 999 | [x] | P1 | level | Everything already closed | verify |\n",
        );
        write_complete_default_mutation_runs(temp_dir.path());

        let cycle =
            evaluate_source_port_frontier_cycle(temp_dir.path(), 1, 1).expect("cycle should build");
        let task = cycle.selected_task.expect("selected task should exist");

        assert_ne!(
            task.delegated_goal,
            Some(super::AutonomyGoal::GameplayBugInvestigation),
            "no open bugs → must not route to investigation lane"
        );
        assert_eq!(task.delegated_goal, Some(super::AutonomyGoal::SweepParity));
        assert!(
            !cycle.progress_fingerprint.contains("bug-investigation:"),
            "fingerprint should not mention bug-investigation when no open bugs exist, got: {}",
            cycle.progress_fingerprint
        );
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
        assert_eq!(task.cycle, 1);
        assert_eq!(task.progress_fingerprint, cycle.progress_fingerprint);
        assert!(!task.generated_at.is_empty());
    }

    fn fake_commit(sha: &str, subject: &str, files: &[&str]) -> RecentCommitSummary {
        RecentCommitSummary {
            sha: sha.to_owned(),
            subject: subject.to_owned(),
            files: files.iter().map(|s| (*s).to_owned()).collect(),
        }
    }

    #[test]
    fn parse_recent_commit_summaries_parses_multiple_commits() {
        let stdout = "\
__KID_COMMIT__
abc1234
First commit
docs/source-port/methodology.md
docs/source-port/research-map.md

__KID_COMMIT__
def5678
Second commit
crates/kidcli/src/main.rs
";
        let parsed = parse_recent_commit_summaries(stdout);
        assert_eq!(parsed.len(), 2);
        assert_eq!(parsed[0].sha, "abc1234");
        assert_eq!(parsed[0].subject, "First commit");
        assert_eq!(
            parsed[0].files,
            vec![
                "docs/source-port/methodology.md".to_string(),
                "docs/source-port/research-map.md".to_string(),
            ]
        );
        assert_eq!(parsed[1].sha, "def5678");
        assert_eq!(
            parsed[1].files,
            vec!["crates/kidcli/src/main.rs".to_string()]
        );
    }

    #[test]
    fn parse_recent_commit_summaries_handles_commit_with_no_files() {
        // `git log --name-only` for an empty commit emits only the DELIM/sha/subject.
        let stdout = "\
__KID_COMMIT__
abc1234
Empty commit
";
        let parsed = parse_recent_commit_summaries(stdout);
        assert_eq!(parsed.len(), 1);
        assert!(parsed[0].files.is_empty());
    }

    #[test]
    fn is_sweep_refresh_commit_detects_docs_only_refresh() {
        let commit = fake_commit(
            "abc1234",
            "Refresh sweep parity evidence",
            &[
                "docs/source-port/methodology.md",
                "docs/source-port/research-map.md",
            ],
        );
        assert!(is_sweep_refresh_commit(&commit));
    }

    #[test]
    fn is_sweep_refresh_commit_allows_single_sweep_doc() {
        let commit = fake_commit(
            "abc1234",
            "docs: nudge methodology",
            &["docs/source-port/methodology.md"],
        );
        assert!(is_sweep_refresh_commit(&commit));
    }

    #[test]
    fn is_sweep_refresh_commit_rejects_mixed_files() {
        let commit = fake_commit(
            "abc1234",
            "docs: refresh plus code",
            &[
                "docs/source-port/methodology.md",
                "crates/kidcli/src/main.rs",
            ],
        );
        assert!(!is_sweep_refresh_commit(&commit));
    }

    #[test]
    fn is_sweep_refresh_commit_rejects_empty_file_list() {
        // Empty file list (e.g. git merge or empty commit) must not be classified
        // as a sweep refresh — otherwise the detector fires spuriously.
        let commit = fake_commit("abc1234", "Merge commit", &[]);
        assert!(!is_sweep_refresh_commit(&commit));
    }

    #[test]
    fn sweep_refresh_streak_counts_consecutive_refreshes() {
        let commits = vec![
            fake_commit(
                "a",
                "Refresh sweep parity evidence",
                &["docs/source-port/methodology.md"],
            ),
            fake_commit(
                "b",
                "Refresh sweep parity evidence",
                &[
                    "docs/source-port/methodology.md",
                    "docs/source-port/research-map.md",
                ],
            ),
            fake_commit(
                "c",
                "Refresh sweep parity evidence",
                &["docs/source-port/research-map.md"],
            ),
            fake_commit("d", "Real code change", &["crates/kidcli/src/main.rs"]),
        ];
        assert_eq!(sweep_refresh_streak(&commits), 3);
    }

    #[test]
    fn sweep_refresh_streak_stops_at_first_non_refresh() {
        let commits = vec![
            fake_commit(
                "a",
                "Refresh sweep parity evidence",
                &["docs/source-port/methodology.md"],
            ),
            fake_commit("b", "Real code change", &["crates/kidcli/src/main.rs"]),
            fake_commit(
                "c",
                "Refresh sweep parity evidence",
                &["docs/source-port/methodology.md"],
            ),
        ];
        // Only the most-recent run counts; the older refreshes are not part of
        // the current streak.
        assert_eq!(sweep_refresh_streak(&commits), 1);
    }

    #[test]
    fn sweep_refresh_streak_returns_zero_when_no_recent_refreshes() {
        let commits = vec![
            fake_commit("a", "Real code change", &["crates/kidcli/src/main.rs"]),
            fake_commit(
                "b",
                "Refresh sweep parity evidence",
                &["docs/source-port/methodology.md"],
            ),
        ];
        assert_eq!(sweep_refresh_streak(&commits), 0);
    }

    #[test]
    fn render_source_port_next_agent_note_embeds_recent_commits_block() {
        let frontier = super::FrontierContext {
            slug: "test-frontier".to_owned(),
            recording: "rec".to_owned(),
            scenario: "scn".to_owned(),
            area: "area".to_owned(),
            status: "complete".to_owned(),
            frontier_frame: 0,
            authoritative_clean_through: 0,
            current_blocker: "none".to_owned(),
            current_hypothesis: super::HypothesisSummary {
                name: "h".to_owned(),
                reason: "r".to_owned(),
                fingerprint: None,
            },
            rejected_hypotheses: vec![],
            required_next_evidence: vec![],
            verification_commands: vec![],
        };
        let task = super::TaskPackage {
            capability: CapabilityId::SourcePortFrontierTask,
            title: "t".to_owned(),
            summary: "s".to_owned(),
            generated_at: "0".to_owned(),
            cycle: 1,
            progress_fingerprint: "fp".to_owned(),
            sources: vec![],
            implementation_targets: vec![],
            commands: vec![],
            done_criteria: vec![],
            evidence: vec![],
            next_agent_note: None,
            frontier_context: Some(frontier.clone()),
            case_queue_context: None,
            delegated_goal: None,
            delegated_target: None,
            delegation_reason: None,
            recovery_mode: None,
            recovery_step: None,
            stalled_repetition_count: None,
            pivot_reason: None,
            next_fallback_goal: None,
            target_routines: None,
            definitively_stalled: false,
        };
        let commits = vec![
            fake_commit(
                "a",
                "Refresh sweep parity evidence",
                &["docs/source-port/methodology.md"],
            ),
            fake_commit(
                "b",
                "Refresh sweep parity evidence",
                &["docs/source-port/research-map.md"],
            ),
            fake_commit(
                "c",
                "Refresh sweep parity evidence",
                &[
                    "docs/source-port/methodology.md",
                    "docs/source-port/research-map.md",
                ],
            ),
        ];
        let rendered = render_source_port_next_agent_note(&frontier, &task, &commits);
        assert!(
            rendered.contains("# Recent Commits"),
            "expected Recent Commits section: {rendered}"
        );
        assert!(
            rendered.contains("- a Refresh sweep parity evidence"),
            "expected first commit summary: {rendered}"
        );
        assert!(
            rendered.contains("STOP: the last 3 consecutive commits"),
            "expected streak STOP warning: {rendered}"
        );
    }

    #[test]
    fn render_source_port_next_agent_note_uses_soft_self_check_when_no_streak() {
        let frontier = super::FrontierContext {
            slug: "test".to_owned(),
            recording: "rec".to_owned(),
            scenario: "scn".to_owned(),
            area: "area".to_owned(),
            status: "complete".to_owned(),
            frontier_frame: 0,
            authoritative_clean_through: 0,
            current_blocker: "none".to_owned(),
            current_hypothesis: super::HypothesisSummary {
                name: "h".to_owned(),
                reason: "r".to_owned(),
                fingerprint: None,
            },
            rejected_hypotheses: vec![],
            required_next_evidence: vec![],
            verification_commands: vec![],
        };
        let task = super::TaskPackage {
            capability: CapabilityId::SourcePortFrontierTask,
            title: "t".to_owned(),
            summary: "s".to_owned(),
            generated_at: "0".to_owned(),
            cycle: 1,
            progress_fingerprint: "fp".to_owned(),
            sources: vec![],
            implementation_targets: vec![],
            commands: vec![],
            done_criteria: vec![],
            evidence: vec![],
            next_agent_note: None,
            frontier_context: Some(frontier.clone()),
            case_queue_context: None,
            delegated_goal: None,
            delegated_target: None,
            delegation_reason: None,
            recovery_mode: None,
            recovery_step: None,
            stalled_repetition_count: None,
            pivot_reason: None,
            next_fallback_goal: None,
            target_routines: None,
            definitively_stalled: false,
        };
        let rendered = render_source_port_next_agent_note(&frontier, &task, &[]);
        assert!(rendered.contains("# Self-Check Before Starting"));
        assert!(rendered.contains("last 3+ consecutive commits"));
        assert!(!rendered.contains("STOP:"));
    }
}
