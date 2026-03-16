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
}

impl AutonomyGoal {
    pub(crate) fn slug(self) -> &'static str {
        match self {
            Self::Level1OpeningSlice => "level1-opening-slice",
            Self::Level1GameplayLifecycle => "level1-gameplay-lifecycle",
            Self::Level1FullTraversal => "level1-full-traversal",
            Self::LevelIdleParity => "level-idle-parity",
            Self::SweepParity => "sweep-parity",
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
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct EvidenceItem {
    label: String,
    status: String,
    detail: String,
    artefact_path: Option<PathBuf>,
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
    rom_path: &Path,
    upstream_dir: &Path,
    goal: AutonomyGoal,
    max_cycles: u32,
    resume: bool,
    emit_json: bool,
) -> Result<()> {
    let spec = load_goal_spec(&goal.spec_path(repo_root))?;
    let manifest = prepare_generated_assets(repo_root, rom_path, upstream_dir)?;
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
            title: spec.title.clone(),
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
        let mut cycle = evaluate_cycle(repo_root, rom_path, &manifest, &spec, cycle_number)?;
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
        if let Some(task) = &cycle.selected_task {
            write_json(&run_dir.join("next-task.json"), task)?;
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
        CapabilityId, CapabilityStatus, CheckOutcome, GoalSpec, RepositorySignals, TaskPackage,
        detect_blocker, evaluate_capabilities, fingerprint_for_capabilities, select_next_task,
    };
    use std::collections::BTreeMap;
    use std::path::{Path, PathBuf};

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
}
