use crate::bootstrap::prepare_generated_assets;
use crate::capture::{
    GameplayState, capture_dynamic_gameplay_scenario, capture_native_dynamic_gameplay_scenario,
};
use crate::oracle::{
    OracleEnemyState, OracleManifest, OracleState, ensure_oracles, run_jgenesis_capture_dynamic,
};
use crate::recording::{
    NativeRecording, load_recording, oracle_scenario_for_recording, run_recorded_trace,
};
use crate::scenarios::{DynamicCheckpoint, DynamicScenario, GameplayScenario};
use anyhow::{Context, Result, anyhow, bail};
use clap::{Subcommand, ValueEnum};
use kid_core::{NativeObjectKind, NativeObjectSnapshot, NativeTraceFrame};
use kid_data::AssetManifest;
use serde::{Deserialize, Serialize};
use sha1::{Digest, Sha1};
use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write as _;
use std::fs;
use std::path::{Path, PathBuf};

const CASE_BUNDLE_VERSION: u32 = 1;
const MASK_LEFT: u8 = 1 << 2;
const MASK_RIGHT: u8 = 1 << 3;
const MASK_B: u8 = 1 << 4;
const MASK_C: u8 = 1 << 5;
const MASK_A: u8 = 1 << 6;

#[derive(Debug, Subcommand)]
pub(crate) enum CaseCommand {
    /// Derive a semantic case bundle from a scenario, gameplay scenario, or recording.
    Derive {
        #[arg(long)]
        rom: Option<PathBuf>,
        #[arg(long)]
        dir: Option<PathBuf>,
        #[arg(long)]
        scenario: Option<PathBuf>,
        #[arg(long, value_enum)]
        gameplay_scenario: Option<GameplayScenario>,
        #[arg(long)]
        recording_slug: Option<String>,
        #[arg(long)]
        recording_bundle: Option<PathBuf>,
        #[arg(long)]
        frame_start: usize,
        #[arg(long)]
        frame_end: usize,
        #[arg(long)]
        out: Option<PathBuf>,
    },
    /// Generate branched continuations from an existing case bundle.
    Branch {
        #[arg(long)]
        case: String,
        #[arg(long, default_value_t = 8)]
        count: usize,
        #[arg(long)]
        seed: Option<u64>,
        #[arg(long)]
        continuation_frames: Option<usize>,
    },
    /// Verify or re-verify a case bundle against native + oracle semantic state.
    Verify {
        #[arg(long)]
        rom: Option<PathBuf>,
        #[arg(long)]
        dir: Option<PathBuf>,
        #[arg(long)]
        case: String,
    },
    /// Reduce a failing continuation while preserving the first semantic failure.
    Reduce {
        #[arg(long)]
        rom: Option<PathBuf>,
        #[arg(long)]
        dir: Option<PathBuf>,
        #[arg(long)]
        case: String,
    },
    /// Rebuild the markdown summary for an already-verified case.
    Summarise {
        #[arg(long)]
        case: String,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ValueEnum)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum EvidenceTier {
    Stubbed,
    Boots,
    ReplayMatched,
    BranchMatched,
    MechanicSwept,
    HumanPlayStable,
}

impl EvidenceTier {
    fn label(self) -> &'static str {
        match self {
            Self::Stubbed => "stubbed",
            Self::Boots => "boots",
            Self::ReplayMatched => "replay-matched",
            Self::BranchMatched => "branch-matched",
            Self::MechanicSwept => "mechanic-swept",
            Self::HumanPlayStable => "human-play-stable",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub(crate) enum CaseSource {
    ScenarioPath { path: String },
    GameplayScenario { slug: String },
    Recording { path: String, slug: String },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct SemanticPlayerState {
    pub(crate) x_pos: Option<u32>,
    pub(crate) y_pos: Option<u32>,
    pub(crate) x_velocity: Option<i32>,
    pub(crate) y_velocity: Option<i32>,
    pub(crate) facing: Option<String>,
    pub(crate) on_ground: Option<bool>,
    pub(crate) movement_state: Option<String>,
    pub(crate) current_form: Option<u16>,
    pub(crate) hitpoints: Option<u16>,
    pub(crate) invulnerable_frames: Option<u16>,
    pub(crate) collision_type: Option<u16>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct SemanticCameraState {
    pub(crate) x_pos: u32,
    pub(crate) y_pos: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct SemanticWorldState {
    pub(crate) game_mode: u16,
    pub(crate) current_level_id: u16,
    pub(crate) player_1_level_id: u16,
    pub(crate) current_player: u8,
    pub(crate) score: Option<u32>,
    pub(crate) number_lives: Option<u16>,
    pub(crate) diamond_count: Option<u16>,
    pub(crate) paused: bool,
    pub(crate) boot_phase: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct SemanticEntity {
    pub(crate) identity: String,
    pub(crate) category: String,
    pub(crate) object_type: Option<u8>,
    pub(crate) x_pos: u32,
    pub(crate) y_pos: u32,
    pub(crate) x_velocity: Option<i32>,
    pub(crate) y_velocity: Option<i32>,
    pub(crate) facing: Option<String>,
    pub(crate) on_ground: Option<bool>,
    pub(crate) hit_points: Option<u16>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct SemanticGameState {
    pub(crate) frame_index: usize,
    pub(crate) player: SemanticPlayerState,
    pub(crate) camera: SemanticCameraState,
    pub(crate) world: SemanticWorldState,
    pub(crate) entities: Vec<SemanticEntity>,
    pub(crate) events: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct CaseFailure {
    pub(crate) frame_index: usize,
    pub(crate) field: String,
    pub(crate) native_value: String,
    pub(crate) oracle_value: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct CaseVerifyReport {
    pub(crate) case_id: String,
    pub(crate) passed: bool,
    pub(crate) frame_count: usize,
    pub(crate) first_failure: Option<CaseFailure>,
    pub(crate) matching_fields: Vec<String>,
    pub(crate) likely_subsystems: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) blocked_reason: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct BranchSpec {
    pub(crate) slug: String,
    pub(crate) seed: u64,
    pub(crate) continuation_frames: usize,
    pub(crate) actions: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct CaseBundleManifest {
    pub(crate) version: u32,
    pub(crate) case_id: String,
    pub(crate) source: CaseSource,
    pub(crate) evidence_tier: EvidenceTier,
    pub(crate) scenario_slug: String,
    pub(crate) frame_start: usize,
    pub(crate) frame_end: usize,
    pub(crate) checkpoint_frame: usize,
    pub(crate) checkpoint_label: String,
    pub(crate) scenario: DynamicScenario,
    pub(crate) prefix_script: Vec<u8>,
    pub(crate) continuation_script: Vec<u8>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub(crate) branches: Vec<BranchSpec>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct LoadedCaseBundle {
    dir: PathBuf,
    manifest: CaseBundleManifest,
    native_states: Vec<SemanticGameState>,
    oracle_states: Vec<SemanticGameState>,
    verify: CaseVerifyReport,
}

#[derive(Debug, Clone)]
struct BranchRng {
    state: u64,
}

impl BranchRng {
    fn new(seed: u64) -> Self {
        Self {
            state: seed.wrapping_add(0x9E37_79B9_7F4A_7C15),
        }
    }

    fn next_u32(&mut self) -> u32 {
        self.state = self.state.wrapping_mul(6364136223846793005).wrapping_add(1);
        (self.state >> 32) as u32
    }

    fn next_inclusive(&mut self, min: usize, max: usize) -> usize {
        if min >= max {
            return min;
        }
        min + (self.next_u32() as usize % (max - min + 1))
    }
}

pub(crate) fn run_case_command(
    repo_root: &Path,
    command: CaseCommand,
    resolve_rom_path: impl Fn(Option<PathBuf>) -> Result<PathBuf> + Clone,
    resolve_upstream_dir: impl Fn(Option<PathBuf>) -> PathBuf + Clone,
) -> Result<()> {
    match command {
        CaseCommand::Derive {
            rom,
            dir,
            scenario,
            gameplay_scenario,
            recording_slug,
            recording_bundle,
            frame_start,
            frame_end,
            out,
        } => {
            let rom_path = resolve_rom_path.clone()(rom)?;
            let upstream_dir = resolve_upstream_dir.clone()(dir);
            let manifest = prepare_generated_assets(repo_root, &rom_path, &upstream_dir)?;
            derive_case_bundle(
                repo_root,
                &rom_path,
                &manifest,
                DeriveOptions {
                    scenario,
                    gameplay_scenario,
                    recording_slug,
                    recording_bundle,
                    frame_start,
                    frame_end,
                    out,
                },
            )?;
        }
        CaseCommand::Branch {
            case,
            count,
            seed,
            continuation_frames,
        } => {
            let case_dir = resolve_case_dir(repo_root, &case)?;
            generate_case_branches(&case_dir, count, seed, continuation_frames)?;
        }
        CaseCommand::Verify { rom, dir, case } => {
            let rom_path = resolve_rom_path.clone()(rom)?;
            let upstream_dir = resolve_upstream_dir.clone()(dir);
            let manifest = prepare_generated_assets(repo_root, &rom_path, &upstream_dir)?;
            let case_dir = resolve_case_dir(repo_root, &case)?;
            verify_case_bundle(repo_root, &rom_path, &manifest, &case_dir)?;
        }
        CaseCommand::Reduce { rom, dir, case } => {
            let rom_path = resolve_rom_path.clone()(rom)?;
            let upstream_dir = resolve_upstream_dir.clone()(dir);
            let manifest = prepare_generated_assets(repo_root, &rom_path, &upstream_dir)?;
            let case_dir = resolve_case_dir(repo_root, &case)?;
            reduce_case_bundle(repo_root, &rom_path, &manifest, &case_dir)?;
        }
        CaseCommand::Summarise { case } => {
            let case_dir = resolve_case_dir(repo_root, &case)?;
            let loaded = load_case_bundle(&case_dir)?;
            write_case_summary(&loaded.dir, &loaded.manifest, &loaded.verify)?;
            println!("summary refreshed: {}", loaded.dir.display());
        }
    }
    Ok(())
}

#[derive(Debug, Clone)]
struct DeriveOptions {
    scenario: Option<PathBuf>,
    gameplay_scenario: Option<GameplayScenario>,
    recording_slug: Option<String>,
    recording_bundle: Option<PathBuf>,
    frame_start: usize,
    frame_end: usize,
    out: Option<PathBuf>,
}

fn derive_case_bundle(
    repo_root: &Path,
    rom_path: &Path,
    manifest: &AssetManifest,
    options: DeriveOptions,
) -> Result<()> {
    let source_count = usize::from(options.scenario.is_some())
        + usize::from(options.gameplay_scenario.is_some())
        + usize::from(options.recording_slug.is_some())
        + usize::from(options.recording_bundle.is_some());
    if source_count != 1 {
        bail!(
            "pass exactly one of --scenario, --gameplay-scenario, --recording-slug, or --recording-bundle"
        );
    }
    let (source, scenario) = if let Some(path) = options.scenario.as_ref() {
        (
            CaseSource::ScenarioPath {
                path: path.display().to_string(),
            },
            load_dynamic_scenario(path)?,
        )
    } else if let Some(gameplay) = options.gameplay_scenario {
        (
            CaseSource::GameplayScenario {
                slug: gameplay.slug().to_string(),
            },
            gameplay.to_dynamic(),
        )
    } else {
        let record_dir = if let Some(slug) = options.recording_slug.as_deref() {
            repo_root.join(".cache/recordings").join(slug)
        } else {
            options
                .recording_bundle
                .clone()
                .ok_or_else(|| anyhow!("missing recording source"))?
        };
        let recording = load_recording(&record_dir)?;
        (
            CaseSource::Recording {
                path: record_dir.display().to_string(),
                slug: recording.manifest.slug,
            },
            recording.manifest.scenario,
        )
    };

    anyhow::ensure!(
        options.frame_start < options.frame_end,
        "frame_start {} must be < frame_end {}",
        options.frame_start,
        options.frame_end
    );
    anyhow::ensure!(
        options.frame_end <= scenario.frame_count,
        "frame_end {} exceeds scenario frame count {}",
        options.frame_end,
        scenario.frame_count
    );

    let case_id = format!(
        "{}_case_{:05}_{:05}",
        scenario.slug, options.frame_start, options.frame_end
    );
    let out_dir = options
        .out
        .unwrap_or_else(|| repo_root.join(".cache/cases").join(&case_id));
    fs::create_dir_all(&out_dir)
        .with_context(|| format!("failed to create {}", out_dir.display()))?;

    let per_frame = build_case_verification_scenario(
        &source,
        &scenario,
        options.frame_start,
        options.frame_end,
    )?;
    let verify = run_case_verification(repo_root, rom_path, manifest, &per_frame, &source)?;

    let manifest_doc = CaseBundleManifest {
        version: CASE_BUNDLE_VERSION,
        case_id: case_id.clone(),
        source,
        evidence_tier: EvidenceTier::ReplayMatched,
        scenario_slug: scenario.slug.clone(),
        frame_start: options.frame_start,
        frame_end: options.frame_end,
        checkpoint_frame: options.frame_start,
        checkpoint_label: format!("frame_{:05}", options.frame_start),
        prefix_script: per_frame.scenario.script[..options.frame_start].to_vec(),
        continuation_script: per_frame.scenario.script[options.frame_start..options.frame_end]
            .to_vec(),
        scenario: per_frame.scenario.clone(),
        branches: Vec::new(),
    };
    persist_case_bundle(&out_dir, &manifest_doc, &verify)?;
    println!("case bundle written to {}", out_dir.display());
    Ok(())
}

fn generate_case_branches(
    case_dir: &Path,
    count: usize,
    seed: Option<u64>,
    continuation_frames: Option<usize>,
) -> Result<()> {
    let mut loaded = load_case_bundle(case_dir)?;
    let base_frames = continuation_frames.unwrap_or(loaded.manifest.continuation_script.len());
    let seed = seed.unwrap_or_else(|| deterministic_seed(&loaded.manifest.case_id));
    let mut rng = BranchRng::new(seed);
    let branches_dir = case_dir.join("branches");
    fs::create_dir_all(&branches_dir)
        .with_context(|| format!("failed to create {}", branches_dir.display()))?;

    loaded.manifest.branches.clear();
    for index in 0..count {
        let branch_seed = seed.wrapping_add(index as u64);
        let (script, actions) = synthesize_branch_script(&mut rng, base_frames);
        let slug = format!("branch_{index:02}");
        let branch_manifest = CaseBundleManifest {
            version: loaded.manifest.version,
            case_id: format!("{}_{}", loaded.manifest.case_id, slug),
            source: loaded.manifest.source.clone(),
            evidence_tier: EvidenceTier::BranchMatched,
            scenario_slug: loaded.manifest.scenario_slug.clone(),
            frame_start: loaded.manifest.frame_start,
            frame_end: loaded.manifest.frame_start + script.len(),
            checkpoint_frame: loaded.manifest.checkpoint_frame,
            checkpoint_label: loaded.manifest.checkpoint_label.clone(),
            scenario: build_branch_scenario(&loaded.manifest, &script),
            prefix_script: loaded.manifest.prefix_script.clone(),
            continuation_script: script.clone(),
            branches: Vec::new(),
        };
        let branch_dir = branches_dir.join(&slug);
        fs::create_dir_all(&branch_dir)
            .with_context(|| format!("failed to create {}", branch_dir.display()))?;
        let branch_spec = BranchSpec {
            slug: slug.clone(),
            seed: branch_seed,
            continuation_frames: script.len(),
            actions,
        };
        fs::write(
            branch_dir.join("manifest.json"),
            serde_json::to_vec_pretty(&branch_manifest)?,
        )
        .with_context(|| {
            format!(
                "failed to write {}",
                branch_dir.join("manifest.json").display()
            )
        })?;
        fs::write(
            branch_dir.join("scenario.json"),
            serde_json::to_vec_pretty(&branch_manifest.scenario)?,
        )
        .with_context(|| {
            format!(
                "failed to write {}",
                branch_dir.join("scenario.json").display()
            )
        })?;
        fs::write(
            branch_dir.join("branch.json"),
            serde_json::to_vec_pretty(&branch_spec)?,
        )
        .with_context(|| {
            format!(
                "failed to write {}",
                branch_dir.join("branch.json").display()
            )
        })?;
        loaded.manifest.branches.push(branch_spec);
    }

    fs::write(
        case_dir.join("manifest.json"),
        serde_json::to_vec_pretty(&loaded.manifest)?,
    )
    .with_context(|| {
        format!(
            "failed to write {}",
            case_dir.join("manifest.json").display()
        )
    })?;
    println!(
        "generated {} branches under {}",
        count,
        branches_dir.display()
    );
    Ok(())
}

fn verify_case_bundle(
    repo_root: &Path,
    rom_path: &Path,
    manifest: &AssetManifest,
    case_dir: &Path,
) -> Result<()> {
    let loaded = canonicalise_case_manifest(load_case_manifest(case_dir)?)?;
    let verify = run_case_verification(
        repo_root,
        rom_path,
        manifest,
        &VerificationScenario {
            scenario: loaded.scenario.clone(),
            case_id: loaded.case_id.clone(),
        },
        &loaded.source,
    )?;
    persist_case_bundle(case_dir, &loaded, &verify)?;
    println!(
        "case {}: {}",
        loaded.case_id,
        if verify.verify.passed { "PASS" } else { "FAIL" }
    );
    Ok(())
}

fn reduce_case_bundle(
    repo_root: &Path,
    rom_path: &Path,
    manifest: &AssetManifest,
    case_dir: &Path,
) -> Result<()> {
    let mut loaded = canonicalise_case_manifest(load_case_manifest(case_dir)?)?;
    let initial = run_case_verification(
        repo_root,
        rom_path,
        manifest,
        &VerificationScenario {
            scenario: loaded.scenario.clone(),
            case_id: loaded.case_id.clone(),
        },
        &loaded.source,
    )?;
    if let Some(blocked_reason) = initial.verify.blocked_reason.as_ref() {
        bail!("case {} is blocked: {blocked_reason}", loaded.case_id);
    }
    let signature = initial.verify.first_failure.clone().ok_or_else(|| {
        anyhow!(
            "case {} is already passing; nothing to reduce",
            loaded.case_id
        )
    })?;

    let mut working = loaded.continuation_script.clone();
    // First shrink the tail to the failure frame.
    let minimum_frames = signature
        .frame_index
        .saturating_sub(loaded.frame_start)
        .saturating_add(1);
    if working.len() > minimum_frames {
        let candidate = working[..minimum_frames].to_vec();
        if preserves_failure_signature(
            repo_root, rom_path, manifest, &loaded, &candidate, &signature,
        )? {
            working = candidate;
        }
    }

    // Then zero redundant spans from the end back towards the start.
    let mut span = (working.len() / 2).max(1);
    while span > 0 {
        let mut changed = false;
        let mut index = working.len();
        while index > 0 {
            let start = index.saturating_sub(span);
            let mut candidate = working.clone();
            let mut zeroed_any_input = false;
            for slot in candidate.iter_mut().take(index).skip(start) {
                if *slot != 0 {
                    *slot = 0;
                    zeroed_any_input = true;
                }
            }
            if !zeroed_any_input {
                if start == 0 {
                    break;
                }
                index = start;
                continue;
            }
            if preserves_failure_signature(
                repo_root, rom_path, manifest, &loaded, &candidate, &signature,
            )? {
                working = candidate;
                changed = true;
                index = working.len();
                continue;
            }
            if start == 0 {
                break;
            }
            index = start;
        }
        if !changed {
            span /= 2;
        }
    }

    loaded.continuation_script.clone_from(&working);
    loaded.frame_end = loaded.frame_start + working.len();
    loaded.scenario = build_branch_scenario(&loaded, &working);
    let verify = run_case_verification(
        repo_root,
        rom_path,
        manifest,
        &VerificationScenario {
            scenario: loaded.scenario.clone(),
            case_id: loaded.case_id.clone(),
        },
        &loaded.source,
    )?;
    persist_case_bundle(case_dir, &loaded, &verify)?;
    println!(
        "reduced case {} to {} continuation frames",
        loaded.case_id,
        working.len()
    );
    Ok(())
}

fn preserves_failure_signature(
    repo_root: &Path,
    rom_path: &Path,
    manifest: &AssetManifest,
    loaded: &CaseBundleManifest,
    continuation_script: &[u8],
    signature: &CaseFailure,
) -> Result<bool> {
    let scenario = build_branch_scenario(loaded, continuation_script);
    let verify = run_case_verification(
        repo_root,
        rom_path,
        manifest,
        &VerificationScenario {
            scenario,
            case_id: loaded.case_id.clone(),
        },
        &loaded.source,
    )?;
    Ok(verify.verify.first_failure.as_ref() == Some(signature))
}

#[derive(Debug, Clone)]
struct VerificationScenario {
    scenario: DynamicScenario,
    case_id: String,
}

#[derive(Debug, Clone)]
struct VerificationOutput {
    native_states: Vec<SemanticGameState>,
    oracle_states: Vec<SemanticGameState>,
    verify: CaseVerifyReport,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum VerificationBackend {
    OracleHelper,
    GameplayCapture,
}

fn run_case_verification(
    repo_root: &Path,
    rom_path: &Path,
    manifest: &AssetManifest,
    verification: &VerificationScenario,
    source: &CaseSource,
) -> Result<VerificationOutput> {
    let run_dir = repo_root
        .join(".cache/cases")
        .join(".tmp")
        .join(&verification.case_id);
    fs::create_dir_all(&run_dir)
        .with_context(|| format!("failed to create {}", run_dir.display()))?;
    let scenario_path = run_dir.join("scenario.json");
    fs::write(
        &scenario_path,
        serde_json::to_vec_pretty(&verification.scenario)?,
    )
    .with_context(|| format!("failed to write {}", scenario_path.display()))?;

    let (native_states, oracle_states) = match verification_backend_for_source(source) {
        VerificationBackend::OracleHelper => {
            let native_trace = if matches!(source, CaseSource::Recording { .. }) {
                let recording = recording_from_source(source)?;
                let replay_trace = run_recorded_trace(
                    manifest,
                    recording.manifest.start,
                    &verification.scenario.script,
                )?;
                if let Some(blocked_reason) =
                    recording_replay_drift_reason(&recording, &replay_trace)
                {
                    let native_states = extract_semantic_native_states(
                        &replay_trace,
                        &verification.scenario.checkpoints,
                    )?;
                    let frame_count = native_states.len();
                    return Ok(VerificationOutput {
                        native_states,
                        oracle_states: Vec::new(),
                        verify: CaseVerifyReport {
                            case_id: verification.case_id.clone(),
                            passed: false,
                            frame_count,
                            first_failure: None,
                            matching_fields: Vec::new(),
                            likely_subsystems: vec![String::from("recording verification")],
                            blocked_reason: Some(blocked_reason),
                        },
                    });
                }
                replay_trace
            } else {
                kid_core::run_script_with_warp(
                    manifest.clone(),
                    &verification.scenario.script,
                    verification.scenario.frame_count,
                    verification.scenario.warp_to_level,
                )
            };

            let env = ensure_oracles(repo_root)?;
            let oracle_dir = run_dir.join("oracle");
            fs::create_dir_all(&oracle_dir)
                .with_context(|| format!("failed to create {}", oracle_dir.display()))?;
            let oracle = if matches!(source, CaseSource::Recording { .. }) {
                let recording = recording_from_source(source)?;
                let oracle_scenario =
                    oracle_scenario_for_case_recording(&recording, &verification.scenario);
                let oracle_scenario_path = run_dir.join("oracle-scenario.json");
                fs::write(
                    &oracle_scenario_path,
                    serde_json::to_vec_pretty(&oracle_scenario.0)?,
                )
                .with_context(|| format!("failed to write {}", oracle_scenario_path.display()))?;
                let oracle = run_jgenesis_capture_dynamic(
                    &env,
                    rom_path,
                    &oracle_scenario_path,
                    &oracle_dir,
                    None,
                )?;
                normalise_recording_oracle_capture(oracle, oracle_scenario.1)
            } else {
                run_jgenesis_capture_dynamic(&env, rom_path, &scenario_path, &oracle_dir, None)?
            };

            (
                extract_semantic_native_states(&native_trace, &verification.scenario.checkpoints)?,
                extract_semantic_oracle_states(&oracle),
            )
        }
        VerificationBackend::GameplayCapture => {
            let gameplay = gameplay_scenario_from_source(source)?;
            let rom_bytes = fs::read(rom_path)
                .with_context(|| format!("failed to read ROM from {}", rom_path.display()))?;
            let oracle_capture = capture_dynamic_gameplay_scenario(
                &rom_bytes,
                &verification.scenario,
                gameplay.uses_level_skip_cheat(),
            )?;
            let native_capture =
                capture_native_dynamic_gameplay_scenario(manifest, &verification.scenario);
            (
                extract_semantic_gameplay_states(&native_capture.checkpoints),
                extract_semantic_gameplay_states(&oracle_capture.checkpoints),
            )
        }
    };
    let verify = if let Some(blocked_reason) =
        recording_oracle_bootstrap_block_reason(source, &native_states, &oracle_states)?
    {
        CaseVerifyReport {
            case_id: verification.case_id.clone(),
            passed: false,
            frame_count: native_states.len(),
            first_failure: None,
            matching_fields: Vec::new(),
            likely_subsystems: vec![String::from("recording verification")],
            blocked_reason: Some(blocked_reason),
        }
    } else {
        compare_semantic_states(verification.case_id.clone(), &native_states, &oracle_states)?
    };
    Ok(VerificationOutput {
        native_states,
        oracle_states,
        verify,
    })
}

fn recording_from_source(source: &CaseSource) -> Result<crate::recording::NativeRecording> {
    match source {
        CaseSource::Recording { path, .. } => load_recording(Path::new(path)),
        CaseSource::ScenarioPath { .. } | CaseSource::GameplayScenario { .. } => {
            bail!("case source is not a recording")
        }
    }
}

fn recording_replay_drift_reason(
    recording: &NativeRecording,
    replay: &[NativeTraceFrame],
) -> Option<String> {
    for (index, (expected, actual)) in recording.trace.iter().zip(replay).enumerate() {
        if expected != actual {
            return Some(format!(
                "source recording replay drifted at frame {index}; re-verify or re-record `{}` before treating this case as gameplay-authoritative",
                recording.manifest.slug
            ));
        }
    }
    if replay.len() > recording.trace.len() {
        return Some(format!(
            "source recording replay length drifted (recorded {}, replay {}); re-verify or re-record `{}` before treating this case as gameplay-authoritative",
            recording.trace.len(),
            replay.len(),
            recording.manifest.slug
        ));
    }
    None
}

fn recording_oracle_bootstrap_block_reason(
    source: &CaseSource,
    native_states: &[SemanticGameState],
    oracle_states: &[SemanticGameState],
) -> Result<Option<String>> {
    let recording = match source {
        CaseSource::Recording { .. } => recording_from_source(source)?,
        CaseSource::ScenarioPath { .. } | CaseSource::GameplayScenario { .. } => return Ok(None),
    };
    if !recording.manifest.start.settled {
        return Ok(None);
    }

    Ok(settled_recording_oracle_bootstrap_block_reason(
        &recording,
        native_states,
        oracle_states,
    ))
}

fn settled_recording_oracle_bootstrap_block_reason(
    recording: &NativeRecording,
    native_states: &[SemanticGameState],
    oracle_states: &[SemanticGameState],
) -> Option<String> {
    let native = native_states.first()?;
    let oracle = oracle_states.first()?;
    let expected_level = recording.manifest.start.warp_level.map(u16::from);
    if expected_level == Some(native.world.current_level_id)
        && native.world.current_level_id != oracle.world.current_level_id
        && oracle.world.current_level_id == 0
        && oracle.world.paused
    {
        return Some(format!(
            "settled recording oracle bootstrap stayed paused on Level 00 while native reached Level {:02X}; fix settled-recording oracle bootstrap before treating `{}` as gameplay-authoritative",
            native.world.current_level_id, recording.manifest.slug
        ));
    }
    if expected_level == Some(native.world.current_level_id)
        && native.world.game_mode == 0x000c
        && oracle.world.current_level_id == native.world.current_level_id
        && oracle.world.game_mode == 0x0008
        && oracle.world.boot_phase.as_deref() == Some("starting_level")
    {
        return Some(format!(
            "settled recording oracle bootstrap stayed on the Level {:02X} title card while native reached gameplay; fix settled-recording oracle bootstrap before treating `{}` as gameplay-authoritative",
            native.world.current_level_id, recording.manifest.slug
        ));
    }

    None
}

fn oracle_scenario_for_case_recording(
    recording: &crate::recording::NativeRecording,
    scenario: &DynamicScenario,
) -> (DynamicScenario, usize) {
    let mut manifest = recording.manifest.clone();
    manifest.scenario = scenario.clone();
    oracle_scenario_for_recording(&crate::recording::NativeRecording {
        manifest,
        trace: Vec::new(),
        legacy_block_destroy_direction_frames: BTreeSet::default(),
    })
}

fn normalise_recording_oracle_capture(
    oracle: OracleManifest,
    prefix_frames: usize,
) -> OracleManifest {
    if prefix_frames == 0 {
        return oracle;
    }

    let mut oracle = oracle;
    for frame in &mut oracle.frames {
        frame.frame_index = frame.frame_index.saturating_sub(prefix_frames);
    }
    for state in &mut oracle.states {
        state.frame_index = state.frame_index.saturating_sub(prefix_frames);
    }
    oracle
}

fn extract_semantic_native_states(
    trace: &[NativeTraceFrame],
    checkpoints: &[DynamicCheckpoint],
) -> Result<Vec<SemanticGameState>> {
    let by_frame = trace
        .iter()
        .map(|frame| (frame.frame_index, frame))
        .collect::<BTreeMap<_, _>>();
    let mut states = Vec::with_capacity(checkpoints.len());
    let mut previous: Option<SemanticGameState> = None;
    for checkpoint in checkpoints {
        let snapshot = by_frame
            .get(&checkpoint.frame_index)
            .ok_or_else(|| anyhow!("missing native frame {}", checkpoint.frame_index))?;
        let semantic = semantic_from_native(snapshot, previous.as_ref());
        previous = Some(semantic.clone());
        states.push(semantic);
    }
    Ok(states)
}

fn extract_semantic_oracle_states(oracle: &OracleManifest) -> Vec<SemanticGameState> {
    let mut states = oracle.states.clone();
    states.sort_by_key(|state| state.frame_index);
    let mut out = Vec::with_capacity(states.len());
    let mut previous: Option<SemanticGameState> = None;
    for state in states {
        let semantic = semantic_from_oracle(&state, previous.as_ref());
        previous = Some(semantic.clone());
        out.push(semantic);
    }
    out
}

fn extract_semantic_gameplay_states(states: &[GameplayState]) -> Vec<SemanticGameState> {
    let mut out = Vec::with_capacity(states.len());
    let mut previous: Option<SemanticGameState> = None;
    for state in states {
        let semantic = semantic_from_gameplay_state(state, previous.as_ref());
        previous = Some(semantic.clone());
        out.push(semantic);
    }
    out
}

fn semantic_from_native(
    snapshot: &NativeTraceFrame,
    previous: Option<&SemanticGameState>,
) -> SemanticGameState {
    let sega_screen_player_hidden = snapshot.boot_phase.as_deref() == Some("sega_screen");
    let player = SemanticPlayerState {
        // ASM Load_SegaScreen only drives the logo object, but oracle WRAM
        // still exposes Kid's cleared position/movement words as zero during
        // the boot shell. Scenario-path cases compare semantic state, so
        // normalise that hidden Sega-screen window instead of preserving
        // `None` from the native trace.
        x_pos: snapshot
            .kid_x_pos
            .or(sega_screen_player_hidden.then_some(0)),
        y_pos: snapshot
            .kid_y_pos
            .or(sega_screen_player_hidden.then_some(0)),
        x_velocity: snapshot.kid_x_velocity,
        y_velocity: snapshot.kid_y_velocity,
        facing: snapshot
            .kid_facing
            .map(|facing| format!("{facing:?}").to_lowercase()),
        on_ground: snapshot.kid_on_ground,
        movement_state: snapshot
            .character_movement
            .map(|movement| format!("movement_{movement:04x}"))
            .or(sega_screen_player_hidden.then_some(String::from("movement_0000"))),
        // ASM loc_65C/sub_8A4 seed progression before the main loop, but
        // Load_SegaScreen still has no gameplay player. Case semantics hide
        // those bookkeeping slots until the boot shell leaves the Sega logo.
        current_form: Some(if sega_screen_player_hidden {
            0
        } else {
            snapshot.current_helmet_id
        }),
        hitpoints: Some(if sega_screen_player_hidden {
            0
        } else {
            snapshot.number_hitpoints
        }),
        invulnerable_frames: Some(snapshot.invulnerable_frames as u16),
        collision_type: snapshot
            .gameplay_debug_fields
            .get("kid_collision_type")
            .and_then(|value| value.parse::<u16>().ok()),
    };
    let camera = SemanticCameraState {
        x_pos: snapshot.camera_x_pos,
        y_pos: snapshot.camera_y_pos,
    };
    let world = SemanticWorldState {
        game_mode: snapshot.game_mode_value,
        current_level_id: u16::from(snapshot.current_level.0),
        player_1_level_id: u16::from(snapshot.player_1_level_id.0),
        current_player: snapshot.current_player,
        score: Some(snapshot.score),
        number_lives: Some(if sega_screen_player_hidden {
            0
        } else {
            snapshot.number_lives
        }),
        diamond_count: Some(snapshot.diamond_count),
        paused: snapshot.paused,
        boot_phase: snapshot.boot_phase.clone(),
    };
    let entities = snapshot
        .active_objects
        .iter()
        .filter_map(semantic_entity_from_native)
        .collect::<Vec<_>>();
    let events = semantic_events(previous, &player, &entities);
    SemanticGameState {
        frame_index: snapshot.frame_index,
        player,
        camera,
        world,
        entities,
        events,
    }
}

fn semantic_from_gameplay_state(
    state: &GameplayState,
    previous: Option<&SemanticGameState>,
) -> SemanticGameState {
    let player = SemanticPlayerState {
        x_pos: Some(state.kid_x_pos),
        y_pos: Some(state.kid_y_pos),
        x_velocity: state.kid_x_vel,
        y_velocity: state.kid_y_vel,
        facing: None,
        on_ground: None,
        movement_state: state
            .character_movement
            .map(|movement| format!("movement_{movement:04x}")),
        current_form: Some(state.current_helmet_id),
        hitpoints: Some(state.number_hitpoints),
        invulnerable_frames: state.invulnerable_frames.map(u16::from),
        collision_type: state.kid_collision_type,
    };
    let camera = SemanticCameraState {
        x_pos: state.camera_x_pos,
        y_pos: state.camera_y_pos,
    };
    let world = SemanticWorldState {
        game_mode: state.game_mode,
        current_level_id: state.current_level_id,
        player_1_level_id: state.player_1_level_id,
        current_player: state.current_player,
        score: None,
        number_lives: Some(state.number_lives),
        diamond_count: None,
        paused: false,
        boot_phase: None,
    };
    let entities = Vec::new();
    let events = semantic_events(previous, &player, &entities);
    SemanticGameState {
        frame_index: state.frame_index,
        player,
        camera,
        world,
        entities,
        events,
    }
}

fn verification_backend_for_source(source: &CaseSource) -> VerificationBackend {
    match source {
        CaseSource::GameplayScenario { .. } => VerificationBackend::GameplayCapture,
        CaseSource::ScenarioPath { .. } | CaseSource::Recording { .. } => {
            VerificationBackend::OracleHelper
        }
    }
}

fn gameplay_scenario_from_source(source: &CaseSource) -> Result<GameplayScenario> {
    match source {
        CaseSource::GameplayScenario { slug } => gameplay_scenario_from_slug(slug)
            .ok_or_else(|| anyhow!("unknown gameplay scenario slug `{slug}` in case manifest")),
        CaseSource::ScenarioPath { .. } | CaseSource::Recording { .. } => {
            bail!("case source is not a gameplay scenario")
        }
    }
}

fn gameplay_scenario_from_slug(slug: &str) -> Option<GameplayScenario> {
    match slug {
        "pause_resume_level1" => Some(GameplayScenario::PauseResumeLevel1),
        "death_respawn_level1" => Some(GameplayScenario::DeathRespawnLevel1),
        "progress_two_transitions" => Some(GameplayScenario::ProgressTwoTransitions),
        "soak_level_10m" => Some(GameplayScenario::SoakLevel10m),
        _ => None,
    }
}

fn semantic_from_oracle(
    state: &OracleState,
    previous: Option<&SemanticGameState>,
) -> SemanticGameState {
    let player = SemanticPlayerState {
        x_pos: Some(state.kid_x_pos),
        y_pos: Some(state.kid_y_pos),
        x_velocity: state.kid_x_vel,
        y_velocity: state.kid_y_vel,
        facing: None,
        on_ground: None,
        movement_state: state
            .character_movement
            .map(|movement| format!("movement_{movement:04x}")),
        current_form: Some(state.current_helmet_id),
        hitpoints: Some(state.number_hitpoints),
        invulnerable_frames: state.invulnerable_frames.map(u16::from),
        collision_type: state.kid_collision_type,
    };
    let camera = SemanticCameraState {
        x_pos: state.camera_x_pos,
        y_pos: state.camera_y_pos,
    };
    let world = SemanticWorldState {
        game_mode: state.game_mode,
        current_level_id: state.current_level_id,
        player_1_level_id: state.player_1_level_id,
        current_player: state.current_player,
        score: Some(state.score),
        number_lives: Some(state.number_lives),
        diamond_count: Some(state.number_diamonds),
        paused: state.game_paused,
        boot_phase: state.boot_phase.clone(),
    };
    let entities = state
        .active_enemies
        .iter()
        .map(semantic_entity_from_oracle)
        .collect::<Vec<_>>();
    let events = semantic_events(previous, &player, &entities);
    SemanticGameState {
        frame_index: state.frame_index,
        player,
        camera,
        world,
        entities,
        events,
    }
}

fn semantic_entity_from_native(snapshot: &NativeObjectSnapshot) -> Option<SemanticEntity> {
    match snapshot.kind {
        NativeObjectKind::Player => None,
        NativeObjectKind::Enemy { object_type } => Some(SemanticEntity {
            identity: format!(
                "enemy:{object_type:02x}:{}:{:08x}:{:08x}",
                snapshot.layout_index.unwrap_or(usize::MAX),
                snapshot.x_pos,
                snapshot.y_pos
            ),
            category: String::from("enemy"),
            object_type: Some(object_type),
            x_pos: snapshot.x_pos,
            y_pos: snapshot.y_pos,
            x_velocity: snapshot.x_velocity,
            y_velocity: snapshot.y_velocity,
            facing: snapshot
                .facing
                .map(|facing| format!("{facing:?}").to_lowercase()),
            on_ground: snapshot.on_ground,
            hit_points: snapshot.hit_points.map(u16::from),
        }),
        NativeObjectKind::Platform { kind, subtype } => Some(SemanticEntity {
            identity: format!(
                "platform:{kind:02x}:{subtype:02x}:{}:{:08x}:{:08x}",
                snapshot.layout_index.unwrap_or(usize::MAX),
                snapshot.x_pos,
                snapshot.y_pos
            ),
            category: String::from("platform"),
            object_type: Some(kind),
            x_pos: snapshot.x_pos,
            y_pos: snapshot.y_pos,
            x_velocity: snapshot.x_velocity,
            y_velocity: snapshot.y_velocity,
            facing: snapshot
                .facing
                .map(|facing| format!("{facing:?}").to_lowercase()),
            on_ground: snapshot.on_ground,
            hit_points: snapshot.hit_points.map(u16::from),
        }),
    }
}

fn semantic_entity_from_oracle(enemy: &OracleEnemyState) -> SemanticEntity {
    SemanticEntity {
        identity: format!(
            "enemy:{:02x}:{}:{:08x}:{:08x}:{:08x}",
            enemy.object_type,
            enemy.layout_index.unwrap_or(usize::MAX),
            enemy.layout_ptr.unwrap_or(u32::MAX),
            enemy.x_pos,
            enemy.y_pos
        ),
        category: String::from("enemy"),
        object_type: Some(enemy.object_type),
        x_pos: enemy.x_pos,
        y_pos: enemy.y_pos,
        x_velocity: Some(enemy.x_velocity),
        y_velocity: Some(enemy.y_velocity),
        facing: Some(format!("{:?}", enemy.facing).to_lowercase()),
        on_ground: enemy.on_ground,
        hit_points: None,
    }
}

fn semantic_events(
    previous: Option<&SemanticGameState>,
    player: &SemanticPlayerState,
    entities: &[SemanticEntity],
) -> Vec<String> {
    let mut events = Vec::new();
    if let Some(previous) = previous {
        if previous.player.on_ground == Some(true) && player.on_ground == Some(false) {
            events.push(String::from("jump_started"));
        }
        if previous.player.on_ground == Some(false) && player.on_ground == Some(true) {
            events.push(String::from("landed"));
        }
        if let (Some(prev), Some(curr)) = (previous.player.hitpoints, player.hitpoints)
            && curr < prev
        {
            events.push(String::from("took_hit"));
        }
        if entities.len() > previous.entities.len() {
            events.push(String::from("spawned_entity"));
        }
        if entities.len() < previous.entities.len() {
            events.push(String::from("despawned_entity"));
        }
        if previous.player.current_form != player.current_form {
            events.push(String::from("transform_changed"));
        }
        if previous.world.current_level_id != playerless_world(previous).current_level_id {
            events.push(String::from("level_transition"));
        }
    }
    events
}

fn playerless_world(state: &SemanticGameState) -> &SemanticWorldState {
    &state.world
}

fn compare_semantic_states(
    case_id: String,
    native_states: &[SemanticGameState],
    oracle_states: &[SemanticGameState],
) -> Result<CaseVerifyReport> {
    anyhow::ensure!(
        native_states.len() == oracle_states.len(),
        "native captured {} states but oracle captured {}",
        native_states.len(),
        oracle_states.len()
    );
    let matching_fields = semantic_matching_fields(native_states, oracle_states);

    for (native, oracle) in native_states.iter().zip(oracle_states) {
        anyhow::ensure!(
            native.frame_index == oracle.frame_index,
            "mismatched frame indices {} vs {}",
            native.frame_index,
            oracle.frame_index
        );

        if let Some(failure) = first_semantic_failure(native, oracle) {
            let likely_subsystems = classify_subsystems(&failure.field);
            return Ok(CaseVerifyReport {
                case_id,
                passed: false,
                frame_count: native_states.len(),
                first_failure: Some(failure),
                matching_fields,
                likely_subsystems,
                blocked_reason: None,
            });
        }
    }

    Ok(CaseVerifyReport {
        case_id,
        passed: true,
        frame_count: native_states.len(),
        first_failure: None,
        matching_fields,
        likely_subsystems: vec![String::from("none")],
        blocked_reason: None,
    })
}

fn semantic_matching_fields(
    native_states: &[SemanticGameState],
    oracle_states: &[SemanticGameState],
) -> Vec<String> {
    let mut matching_fields = Vec::new();

    if native_states
        .iter()
        .zip(oracle_states)
        .all(|(native, oracle)| native.world.game_mode == oracle.world.game_mode)
    {
        matching_fields.push(String::from("world.game_mode"));
    }
    if native_states
        .iter()
        .zip(oracle_states)
        .all(|(native, oracle)| {
            native.camera.x_pos == oracle.camera.x_pos && native.camera.y_pos == oracle.camera.y_pos
        })
    {
        matching_fields.push(String::from("camera"));
    }
    if native_states
        .iter()
        .zip(oracle_states)
        .all(|(native, oracle)| {
            native.player.x_pos == oracle.player.x_pos && native.player.y_pos == oracle.player.y_pos
        })
    {
        matching_fields.push(String::from("player.position"));
    }
    if native_states
        .iter()
        .zip(oracle_states)
        .all(|(native, oracle)| {
            native.player.x_velocity == oracle.player.x_velocity
                && native.player.y_velocity == oracle.player.y_velocity
                && native.player.on_ground == oracle.player.on_ground
                && native.player.movement_state == oracle.player.movement_state
                && native.player.current_form == oracle.player.current_form
                && native.player.hitpoints == oracle.player.hitpoints
        })
    {
        matching_fields.push(String::from("player.motion"));
    }
    if native_states
        .iter()
        .zip(oracle_states)
        .all(|(native, oracle)| native.entities == oracle.entities)
    {
        matching_fields.push(String::from("entities"));
    }
    if native_states
        .iter()
        .zip(oracle_states)
        .all(|(native, oracle)| native.events == oracle.events)
    {
        matching_fields.push(String::from("events"));
    }

    matching_fields
}

fn first_semantic_failure(
    native: &SemanticGameState,
    oracle: &SemanticGameState,
) -> Option<CaseFailure> {
    compare_field(
        native.frame_index,
        "world.game_mode",
        &native.world.game_mode,
        &oracle.world.game_mode,
    )
    .or_else(|| {
        compare_field(
            native.frame_index,
            "camera.x_pos",
            &native.camera.x_pos,
            &oracle.camera.x_pos,
        )
    })
    .or_else(|| {
        compare_field(
            native.frame_index,
            "camera.y_pos",
            &native.camera.y_pos,
            &oracle.camera.y_pos,
        )
    })
    .or_else(|| {
        compare_opt_field(
            native.frame_index,
            "player.x_pos",
            native.player.x_pos,
            oracle.player.x_pos,
        )
    })
    .or_else(|| {
        compare_opt_field(
            native.frame_index,
            "player.y_pos",
            native.player.y_pos,
            oracle.player.y_pos,
        )
    })
    .or_else(|| {
        compare_opt_field(
            native.frame_index,
            "player.x_velocity",
            native.player.x_velocity,
            oracle.player.x_velocity,
        )
    })
    .or_else(|| {
        compare_opt_field(
            native.frame_index,
            "player.y_velocity",
            native.player.y_velocity,
            oracle.player.y_velocity,
        )
    })
    .or_else(|| {
        compare_opt_field(
            native.frame_index,
            "player.on_ground",
            native.player.on_ground,
            oracle.player.on_ground,
        )
    })
    .or_else(|| {
        compare_opt_field(
            native.frame_index,
            "player.movement_state",
            native.player.movement_state.clone(),
            oracle.player.movement_state.clone(),
        )
    })
    .or_else(|| {
        compare_opt_field(
            native.frame_index,
            "player.current_form",
            native.player.current_form,
            oracle.player.current_form,
        )
    })
    .or_else(|| {
        compare_opt_field(
            native.frame_index,
            "player.hitpoints",
            native.player.hitpoints,
            oracle.player.hitpoints,
        )
    })
    .or_else(|| compare_vec_field(native.frame_index, "events", &native.events, &oracle.events))
    .or_else(|| compare_entities(native.frame_index, &native.entities, &oracle.entities))
}

fn compare_field<T: ToString + PartialEq>(
    frame_index: usize,
    field: &str,
    native: &T,
    oracle: &T,
) -> Option<CaseFailure> {
    if native == oracle {
        None
    } else {
        Some(CaseFailure {
            frame_index,
            field: field.to_string(),
            native_value: native.to_string(),
            oracle_value: oracle.to_string(),
        })
    }
}

fn compare_opt_field<T: ToString + PartialEq>(
    frame_index: usize,
    field: &str,
    native: Option<T>,
    oracle: Option<T>,
) -> Option<CaseFailure> {
    if native == oracle {
        None
    } else {
        Some(CaseFailure {
            frame_index,
            field: field.to_string(),
            native_value: format_option(native),
            oracle_value: format_option(oracle),
        })
    }
}

fn compare_vec_field<T: Serialize + PartialEq>(
    frame_index: usize,
    field: &str,
    native: &[T],
    oracle: &[T],
) -> Option<CaseFailure> {
    if native == oracle {
        None
    } else {
        Some(CaseFailure {
            frame_index,
            field: field.to_string(),
            native_value: serde_json::to_string(native).unwrap_or_else(|_| String::from("[]")),
            oracle_value: serde_json::to_string(oracle).unwrap_or_else(|_| String::from("[]")),
        })
    }
}

fn compare_entities(
    frame_index: usize,
    native: &[SemanticEntity],
    oracle: &[SemanticEntity],
) -> Option<CaseFailure> {
    if native == oracle {
        return None;
    }
    Some(CaseFailure {
        frame_index,
        field: String::from("entities"),
        native_value: serde_json::to_string(native).unwrap_or_else(|_| String::from("[]")),
        oracle_value: serde_json::to_string(oracle).unwrap_or_else(|_| String::from("[]")),
    })
}

fn classify_subsystems(field: &str) -> Vec<String> {
    if field.starts_with("camera.") {
        vec![String::from("camera")]
    } else if field.starts_with("player.on_ground")
        || field.starts_with("player.x_")
        || field.starts_with("player.y_")
    {
        vec![String::from("player physics"), String::from("collision")]
    } else if field.starts_with("player.current_form") || field.starts_with("player.hitpoints") {
        vec![String::from("combat"), String::from("state machine")]
    } else if field == "entities" {
        vec![
            String::from("spawn order"),
            String::from("entity lifecycle"),
        ]
    } else if field == "events" {
        vec![
            String::from("gameplay handoff"),
            String::from("state machine"),
        ]
    } else {
        vec![String::from("gameplay")]
    }
}

fn persist_case_bundle(
    case_dir: &Path,
    manifest: &CaseBundleManifest,
    verify: &VerificationOutput,
) -> Result<()> {
    fs::create_dir_all(case_dir)
        .with_context(|| format!("failed to create {}", case_dir.display()))?;
    fs::write(
        case_dir.join("manifest.json"),
        serde_json::to_vec_pretty(manifest)?,
    )
    .with_context(|| {
        format!(
            "failed to write {}",
            case_dir.join("manifest.json").display()
        )
    })?;
    fs::write(
        case_dir.join("scenario.json"),
        serde_json::to_vec_pretty(&manifest.scenario)?,
    )
    .with_context(|| {
        format!(
            "failed to write {}",
            case_dir.join("scenario.json").display()
        )
    })?;
    fs::write(
        case_dir.join("native-semantic.json"),
        serde_json::to_vec_pretty(&verify.native_states)?,
    )
    .with_context(|| {
        format!(
            "failed to write {}",
            case_dir.join("native-semantic.json").display()
        )
    })?;
    fs::write(
        case_dir.join("oracle-semantic.json"),
        serde_json::to_vec_pretty(&verify.oracle_states)?,
    )
    .with_context(|| {
        format!(
            "failed to write {}",
            case_dir.join("oracle-semantic.json").display()
        )
    })?;
    fs::write(
        case_dir.join("verify.json"),
        serde_json::to_vec_pretty(&verify.verify)?,
    )
    .with_context(|| format!("failed to write {}", case_dir.join("verify.json").display()))?;
    write_case_summary(case_dir, manifest, &verify.verify)?;
    Ok(())
}

fn write_case_summary(
    case_dir: &Path,
    manifest: &CaseBundleManifest,
    verify: &CaseVerifyReport,
) -> Result<()> {
    let mut summary = String::new();
    writeln!(&mut summary, "case_id: {}", manifest.case_id).ok();
    writeln!(
        &mut summary,
        "evidence_tier: {}",
        manifest.evidence_tier.label()
    )
    .ok();
    writeln!(&mut summary, "checkpoint: {}", manifest.checkpoint_label).ok();
    writeln!(
        &mut summary,
        "frames: {}..{}",
        manifest.frame_start, manifest.frame_end
    )
    .ok();
    writeln!(&mut summary, "passed: {}", verify.passed).ok();
    if let Some(failure) = verify.first_failure.as_ref() {
        writeln!(&mut summary, "first_bad_frame: {}", failure.frame_index).ok();
        writeln!(&mut summary, "first_bad_field: {}", failure.field).ok();
        writeln!(&mut summary, "native_value: {}", failure.native_value).ok();
        writeln!(&mut summary, "oracle_value: {}", failure.oracle_value).ok();
        writeln!(
            &mut summary,
            "likely_subsystems: {}",
            verify.likely_subsystems.join(", ")
        )
        .ok();
    }
    if let Some(blocked_reason) = verify.blocked_reason.as_ref() {
        writeln!(&mut summary, "blocked_reason: {blocked_reason}").ok();
    }
    writeln!(
        &mut summary,
        "matching_fields: {}",
        verify.matching_fields.join(", ")
    )
    .ok();
    if !manifest.branches.is_empty() {
        writeln!(&mut summary, "branches:").ok();
        for branch in &manifest.branches {
            writeln!(
                &mut summary,
                "- {} seed={} frames={} actions={}",
                branch.slug,
                branch.seed,
                branch.continuation_frames,
                branch.actions.join(" | ")
            )
            .ok();
        }
    }
    fs::write(case_dir.join("summary.md"), summary)
        .with_context(|| format!("failed to write {}", case_dir.join("summary.md").display()))?;
    Ok(())
}

fn load_case_manifest(case_dir: &Path) -> Result<CaseBundleManifest> {
    let bytes = fs::read(case_dir.join("manifest.json")).with_context(|| {
        format!(
            "failed to read {}",
            case_dir.join("manifest.json").display()
        )
    })?;
    serde_json::from_slice(&bytes).with_context(|| {
        format!(
            "failed to parse {}",
            case_dir.join("manifest.json").display()
        )
    })
}

fn load_case_bundle(case_dir: &Path) -> Result<LoadedCaseBundle> {
    let manifest = load_case_manifest(case_dir)?;
    let native_states = fs::read(case_dir.join("native-semantic.json"))
        .ok()
        .map(|bytes| serde_json::from_slice(&bytes))
        .transpose()?
        .unwrap_or_default();
    let oracle_states = fs::read(case_dir.join("oracle-semantic.json"))
        .ok()
        .map(|bytes| serde_json::from_slice(&bytes))
        .transpose()?
        .unwrap_or_default();
    let verify = fs::read(case_dir.join("verify.json"))
        .ok()
        .map(|bytes| serde_json::from_slice(&bytes))
        .transpose()?
        .unwrap_or(CaseVerifyReport {
            case_id: manifest.case_id.clone(),
            passed: false,
            frame_count: 0,
            first_failure: None,
            matching_fields: Vec::new(),
            likely_subsystems: Vec::new(),
            blocked_reason: None,
        });
    Ok(LoadedCaseBundle {
        dir: case_dir.to_path_buf(),
        manifest,
        native_states,
        oracle_states,
        verify,
    })
}

fn resolve_case_dir(repo_root: &Path, selector: &str) -> Result<PathBuf> {
    let path = PathBuf::from(selector);
    if path.exists() {
        return Ok(path);
    }
    let cache_path = repo_root.join(".cache/cases").join(selector);
    if cache_path.exists() {
        return Ok(cache_path);
    }
    bail!("case bundle not found: {selector}")
}

fn load_dynamic_scenario(path: &Path) -> Result<DynamicScenario> {
    let scenario_json =
        fs::read_to_string(path).with_context(|| format!("failed to read {}", path.display()))?;
    serde_json::from_str(&scenario_json).context("invalid scenario JSON")
}

fn build_case_verification_scenario(
    source: &CaseSource,
    base_scenario: &DynamicScenario,
    frame_start: usize,
    frame_end: usize,
) -> Result<VerificationScenario> {
    let scenario = match source {
        CaseSource::GameplayScenario { .. } => {
            let gameplay = gameplay_scenario_from_source(source)?;
            // Keep gameplay cases on the same window builder used by
            // `compare-native-gameplay-window` so reduced evidence and direct
            // gameplay-oracle comparisons slice identical timelines.
            gameplay.to_dynamic_window(frame_start, frame_end.saturating_sub(1), 1)?
        }
        CaseSource::ScenarioPath { .. } | CaseSource::Recording { .. } => {
            let mut sliced = base_scenario.clone();
            sliced.script.truncate(frame_end);
            sliced.frame_count = frame_end;
            sliced.checkpoints = (frame_start..frame_end)
                .map(|frame_index| DynamicCheckpoint {
                    label: format!("frame_{frame_index:05}"),
                    frame_index,
                })
                .collect();
            sliced
        }
    };
    Ok(VerificationScenario {
        case_id: format!(
            "{}_case_{frame_start:05}_{frame_end:05}",
            base_scenario.slug
        ),
        scenario,
    })
}

fn canonicalise_case_manifest(mut manifest: CaseBundleManifest) -> Result<CaseBundleManifest> {
    if !matches!(manifest.source, CaseSource::GameplayScenario { .. }) {
        return Ok(manifest);
    }

    let verification = build_case_verification_scenario(
        &manifest.source,
        &manifest.scenario,
        manifest.frame_start,
        manifest.frame_end,
    )?;
    manifest.scenario = verification.scenario;
    manifest.checkpoint_frame = manifest.frame_start;
    manifest.checkpoint_label = format!("frame_{:05}", manifest.frame_start);
    manifest.prefix_script = manifest.scenario.script[..manifest.frame_start].to_vec();
    manifest.continuation_script =
        manifest.scenario.script[manifest.frame_start..manifest.frame_end].to_vec();
    Ok(manifest)
}

fn build_branch_scenario(
    manifest: &CaseBundleManifest,
    continuation_script: &[u8],
) -> DynamicScenario {
    let mut script = manifest.prefix_script.clone();
    script.extend_from_slice(continuation_script);
    let frame_end = manifest.frame_start + continuation_script.len();
    DynamicScenario {
        slug: format!("{}_branch", manifest.scenario_slug),
        frame_count: frame_end,
        script,
        checkpoints: (manifest.frame_start..frame_end)
            .map(|frame_index| DynamicCheckpoint {
                label: format!("frame_{frame_index:05}"),
                frame_index,
            })
            .collect(),
        warp_to_level: manifest.scenario.warp_to_level,
    }
}

fn synthesize_branch_script(rng: &mut BranchRng, frame_count: usize) -> (Vec<u8>, Vec<String>) {
    let mut script = vec![0; frame_count];
    let mut frame = 0usize;
    let mut actions = Vec::new();
    while frame < frame_count {
        let choice = rng.next_inclusive(0, 6);
        match choice {
            0 => {
                let len = rng.next_inclusive(6, 18).min(frame_count - frame);
                for slot in script.iter_mut().skip(frame).take(len) {
                    *slot |= MASK_RIGHT;
                }
                actions.push(format!("hold right {len}"));
                frame += len;
            }
            1 => {
                let len = rng.next_inclusive(6, 18).min(frame_count - frame);
                for slot in script.iter_mut().skip(frame).take(len) {
                    *slot |= MASK_LEFT;
                }
                actions.push(format!("hold left {len}"));
                frame += len;
            }
            2 => {
                let len = rng.next_inclusive(2, 4).min(frame_count - frame);
                for slot in script.iter_mut().skip(frame).take(len) {
                    *slot |= MASK_B;
                }
                actions.push(format!("tap jump {len}"));
                frame += len;
            }
            3 => {
                let delay = rng.next_inclusive(2, 10).min(frame_count - frame);
                actions.push(format!("delay {delay}"));
                frame += delay;
                let len = rng.next_inclusive(2, 4).min(frame_count - frame);
                for slot in script.iter_mut().skip(frame).take(len) {
                    *slot |= MASK_B;
                }
                actions.push(format!("then jump {len}"));
                frame += len;
            }
            4 => {
                let len = rng.next_inclusive(4, 10).min(frame_count - frame);
                for slot in script.iter_mut().skip(frame).take(len) {
                    *slot |= 1 << 1;
                }
                actions.push(format!("crouch {len}"));
                frame += len;
            }
            5 => {
                let len = rng.next_inclusive(4, 12).min(frame_count - frame);
                actions.push(format!("no-op {len}"));
                frame += len;
            }
            _ => {
                let len = rng.next_inclusive(4, 12).min(frame_count - frame);
                for (offset, slot) in script.iter_mut().skip(frame).take(len).enumerate() {
                    *slot |= if offset % 2 == 0 {
                        MASK_LEFT
                    } else {
                        MASK_RIGHT
                    };
                }
                actions.push(format!("alternate dir {len}"));
                frame += len;
            }
        }
        if frame < frame_count && rng.next_inclusive(0, 5) == 0 {
            script[frame] |= MASK_A | MASK_C;
            actions.push(String::from("interact tap 1"));
            frame += 1;
        }
    }
    (script, actions)
}

fn deterministic_seed(label: &str) -> u64 {
    let mut hasher = Sha1::new();
    hasher.update(label.as_bytes());
    let digest = hasher.finalize();
    let bytes: [u8; 8] = digest[..8].try_into().unwrap_or([0; 8]);
    u64::from_be_bytes(bytes)
}

fn format_option<T: ToString>(value: Option<T>) -> String {
    value.map_or_else(|| String::from("none"), |value| value.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_case_manifest() -> CaseBundleManifest {
        CaseBundleManifest {
            version: CASE_BUNDLE_VERSION,
            case_id: String::from("sample_case"),
            source: CaseSource::GameplayScenario {
                slug: String::from("pause_resume_level1"),
            },
            evidence_tier: EvidenceTier::ReplayMatched,
            scenario_slug: String::from("sample"),
            frame_start: 10,
            frame_end: 20,
            checkpoint_frame: 10,
            checkpoint_label: String::from("frame_00010"),
            scenario: DynamicScenario {
                slug: String::from("sample"),
                script: vec![0; 20],
                frame_count: 20,
                checkpoints: vec![],
                warp_to_level: None,
            },
            prefix_script: vec![0; 10],
            continuation_script: vec![0; 10],
            branches: Vec::new(),
        }
    }

    #[test]
    fn synthesize_branch_script_is_seeded_and_non_empty() {
        let mut rng_a = BranchRng::new(42);
        let mut rng_b = BranchRng::new(42);
        let (script_a, actions_a) = synthesize_branch_script(&mut rng_a, 24);
        let (script_b, actions_b) = synthesize_branch_script(&mut rng_b, 24);
        assert_eq!(script_a, script_b);
        assert_eq!(actions_a, actions_b);
        assert!(actions_a.iter().all(|action| !action.is_empty()));
    }

    #[test]
    fn build_branch_scenario_replaces_only_continuation_window() {
        let manifest = sample_case_manifest();
        let continuation = vec![MASK_RIGHT; 4];
        let scenario = build_branch_scenario(&manifest, &continuation);
        assert_eq!(scenario.frame_count, 14);
        assert_eq!(scenario.script.len(), 14);
        assert_eq!(&scenario.script[10..], &[MASK_RIGHT; 4]);
    }

    #[test]
    fn compare_semantic_states_reports_first_failure_and_subsystem() {
        let native = SemanticGameState {
            frame_index: 12,
            player: SemanticPlayerState {
                x_pos: Some(10),
                y_pos: Some(20),
                x_velocity: Some(1),
                y_velocity: Some(0),
                facing: None,
                on_ground: Some(true),
                movement_state: Some(String::from("movement_0000")),
                current_form: Some(0),
                hitpoints: Some(3),
                invulnerable_frames: Some(0),
                collision_type: None,
            },
            camera: SemanticCameraState { x_pos: 5, y_pos: 7 },
            world: SemanticWorldState {
                game_mode: 0x000c,
                current_level_id: 0,
                player_1_level_id: 0,
                current_player: 0,
                score: Some(0),
                number_lives: Some(3),
                diamond_count: Some(0),
                paused: false,
                boot_phase: None,
            },
            entities: vec![],
            events: vec![],
        };
        let mut oracle = native.clone();
        oracle.player.y_velocity = Some(-1);
        let report =
            compare_semantic_states(String::from("sample"), &[native], &[oracle]).expect("report");
        let failure = report.first_failure.expect("first failure");
        assert_eq!(failure.field, "player.y_velocity");
        assert_eq!(
            report.likely_subsystems,
            vec!["player physics", "collision"]
        );
    }

    #[test]
    fn compare_semantic_states_only_reports_categories_that_match() {
        let native = SemanticGameState {
            frame_index: 720,
            player: SemanticPlayerState {
                x_pos: Some(10),
                y_pos: Some(20),
                x_velocity: Some(1),
                y_velocity: Some(0),
                facing: None,
                on_ground: Some(true),
                movement_state: Some(String::from("movement_0000")),
                current_form: Some(0),
                hitpoints: Some(3),
                invulnerable_frames: Some(0),
                collision_type: None,
            },
            camera: SemanticCameraState { x_pos: 5, y_pos: 7 },
            world: SemanticWorldState {
                game_mode: 0x000c,
                current_level_id: 0,
                player_1_level_id: 0,
                current_player: 0,
                score: Some(0),
                number_lives: Some(3),
                diamond_count: Some(0),
                paused: false,
                boot_phase: None,
            },
            entities: vec![],
            events: vec![],
        };
        let mut oracle = native.clone();
        oracle.world.game_mode = 0x0008;
        let report =
            compare_semantic_states(String::from("sample"), &[native], &[oracle]).expect("report");

        assert_eq!(
            report.matching_fields,
            vec![
                "camera",
                "player.position",
                "player.motion",
                "entities",
                "events"
            ]
        );
    }

    #[test]
    fn evidence_tier_labels_match_repo_language() {
        assert_eq!(EvidenceTier::ReplayMatched.label(), "replay-matched");
        assert_eq!(EvidenceTier::HumanPlayStable.label(), "human-play-stable");
    }

    #[test]
    fn reduction_skips_spans_that_are_already_zero() {
        let working = vec![0, MASK_RIGHT, 0, 0];
        let start = 2;
        let index = 4;
        let mut candidate = working.clone();
        let mut zeroed_any_input = false;
        for slot in candidate.iter_mut().take(index).skip(start) {
            if *slot != 0 {
                *slot = 0;
                zeroed_any_input = true;
            }
        }

        assert!(!zeroed_any_input);
        assert_eq!(candidate, working);
    }

    #[test]
    fn gameplay_case_sources_use_gameplay_capture_backend() {
        let source = CaseSource::GameplayScenario {
            slug: String::from("death_respawn_level1"),
        };
        assert_eq!(
            verification_backend_for_source(&source),
            VerificationBackend::GameplayCapture
        );
        assert_eq!(
            gameplay_scenario_from_source(&source).expect("known gameplay slug"),
            GameplayScenario::DeathRespawnLevel1
        );
    }

    #[test]
    fn gameplay_case_verification_scenario_matches_canonical_window_builder() {
        let source = CaseSource::GameplayScenario {
            slug: String::from("death_respawn_level1"),
        };
        let base = GameplayScenario::DeathRespawnLevel1.to_dynamic();

        let verification = build_case_verification_scenario(&source, &base, 720, 900)
            .expect("verification scenario");
        let expected = GameplayScenario::DeathRespawnLevel1
            .to_dynamic_window(720, 899, 1)
            .expect("canonical window");

        assert_eq!(
            verification.case_id,
            "death_respawn_level1_case_00720_00900"
        );
        assert_eq!(verification.scenario.frame_count, expected.frame_count);
        assert_eq!(verification.scenario.checkpoints, expected.checkpoints);
        assert_eq!(verification.scenario.script, expected.script);
    }

    #[test]
    fn canonicalise_gameplay_case_manifest_repairs_stale_window_payload() {
        let mut manifest = sample_case_manifest();
        manifest.source = CaseSource::GameplayScenario {
            slug: String::from("death_respawn_level1"),
        };
        manifest.scenario_slug = String::from("death_respawn_level1");
        manifest.frame_start = 600;
        manifest.frame_end = 750;
        manifest.checkpoint_frame = 600;
        manifest.checkpoint_label = String::from("frame_00600");
        manifest.scenario = DynamicScenario {
            slug: String::from("stale"),
            script: vec![0; 750],
            frame_count: 750,
            checkpoints: vec![DynamicCheckpoint {
                label: String::from("frame_00600"),
                frame_index: 600,
            }],
            warp_to_level: None,
        };
        manifest.prefix_script = vec![0; 10];
        manifest.continuation_script = vec![0; 10];

        let repaired = canonicalise_case_manifest(manifest).expect("canonical gameplay manifest");
        let expected = GameplayScenario::DeathRespawnLevel1
            .to_dynamic_window(600, 749, 1)
            .expect("canonical gameplay window");

        assert_eq!(repaired.scenario, expected);
        assert_eq!(repaired.prefix_script, expected.script[..600].to_vec());
        assert_eq!(
            repaired.continuation_script,
            expected.script[600..750].to_vec()
        );
    }

    #[test]
    fn scenario_path_sources_stay_on_oracle_helper_backend() {
        let source = CaseSource::ScenarioPath {
            path: String::from("fixtures/sample.json"),
        };
        assert_eq!(
            verification_backend_for_source(&source),
            VerificationBackend::OracleHelper
        );
    }

    #[test]
    fn recording_case_verification_scenario_stays_on_recording_timeline() {
        let source = CaseSource::Recording {
            path: String::from(".cache/recordings/level1-tree-break"),
            slug: String::from("level1-tree-break"),
        };
        let base = DynamicScenario {
            slug: String::from("level1-tree-break"),
            script: vec![1; 180],
            frame_count: 180,
            checkpoints: vec![DynamicCheckpoint {
                label: String::from("start"),
                frame_index: 0,
            }],
            warp_to_level: Some(0),
        };

        let verification = build_case_verification_scenario(&source, &base, 0, 120)
            .expect("verification scenario");

        assert_eq!(verification.case_id, "level1-tree-break_case_00000_00120");
        assert_eq!(verification.scenario.frame_count, 120);
        assert_eq!(verification.scenario.script.len(), 120);
        assert_eq!(verification.scenario.checkpoints[0].frame_index, 0);
        assert_eq!(verification.scenario.checkpoints[119].frame_index, 119);
    }

    #[test]
    fn recording_replay_drift_reason_blocks_stale_recording_cases() {
        let recording = NativeRecording {
            manifest: crate::recording::NativeRecordingManifest {
                bundle_version: 4,
                slug: String::from("level1-tree-break"),
                recorded_at: String::from("123"),
                start: crate::recording::RecordingStart {
                    warp_level: Some(0),
                    settled: true,
                    record_frames: None,
                },
                rom_sha1: String::new(),
                script_sha1: String::new(),
                trace_sha1: String::new(),
                host_os: String::new(),
                host_arch: String::new(),
                git_commit: None,
                git_dirty: None,
                record_command: String::new(),
                verify_command: String::new(),
                artifacts: Vec::new(),
                scenario: DynamicScenario {
                    slug: String::from("level1-tree-break"),
                    script: vec![0],
                    frame_count: 1,
                    checkpoints: vec![DynamicCheckpoint {
                        label: String::from("start"),
                        frame_index: 0,
                    }],
                    warp_to_level: Some(0),
                },
            },
            trace: vec![NativeTraceFrame::default()],
            legacy_block_destroy_direction_frames: BTreeSet::default(),
        };
        let replay_frame = NativeTraceFrame {
            game_mode_value: 0x000c,
            ..NativeTraceFrame::default()
        };
        let reason =
            recording_replay_drift_reason(&recording, &[replay_frame]).expect("drift reason");

        assert!(reason.contains("source recording replay drifted at frame 0"));
        assert!(reason.contains("level1-tree-break"));
    }

    #[test]
    fn recording_replay_drift_reason_accepts_case_prefix() {
        let recording = NativeRecording {
            manifest: crate::recording::NativeRecordingManifest {
                bundle_version: 4,
                slug: String::from("level1-tree-break"),
                recorded_at: String::from("123"),
                start: crate::recording::RecordingStart {
                    warp_level: Some(0),
                    settled: true,
                    record_frames: None,
                },
                rom_sha1: String::new(),
                script_sha1: String::new(),
                trace_sha1: String::new(),
                host_os: String::new(),
                host_arch: String::new(),
                git_commit: None,
                git_dirty: None,
                record_command: String::new(),
                verify_command: String::new(),
                artifacts: Vec::new(),
                scenario: DynamicScenario {
                    slug: String::from("level1-tree-break"),
                    script: vec![0, 0],
                    frame_count: 2,
                    checkpoints: vec![DynamicCheckpoint {
                        label: String::from("start"),
                        frame_index: 0,
                    }],
                    warp_to_level: Some(0),
                },
            },
            trace: vec![NativeTraceFrame::default(), NativeTraceFrame::default()],
            legacy_block_destroy_direction_frames: BTreeSet::default(),
        };

        let reason = recording_replay_drift_reason(&recording, &[NativeTraceFrame::default()]);

        assert_eq!(reason, None);
    }

    #[test]
    fn settled_recording_oracle_bootstrap_block_reason_catches_level_zero_pause() {
        let recording = NativeRecording {
            manifest: crate::recording::NativeRecordingManifest {
                bundle_version: 4,
                slug: String::from("knights-isle-probe"),
                recorded_at: String::from("123"),
                start: crate::recording::RecordingStart {
                    warp_level: Some(0x15),
                    settled: true,
                    record_frames: None,
                },
                rom_sha1: String::new(),
                script_sha1: String::new(),
                trace_sha1: String::new(),
                host_os: String::new(),
                host_arch: String::new(),
                git_commit: None,
                git_dirty: None,
                record_command: String::new(),
                verify_command: String::new(),
                artifacts: Vec::new(),
                scenario: DynamicScenario {
                    slug: String::from("knights-isle-probe"),
                    script: vec![0],
                    frame_count: 1,
                    checkpoints: vec![DynamicCheckpoint {
                        label: String::from("start"),
                        frame_index: 0,
                    }],
                    warp_to_level: Some(0x15),
                },
            },
            trace: vec![NativeTraceFrame::default()],
            legacy_block_destroy_direction_frames: BTreeSet::default(),
        };
        let native = semantic_state_at_level(0x15, false);
        let oracle = semantic_state_at_level(0x00, true);

        let reason =
            settled_recording_oracle_bootstrap_block_reason(&recording, &[native], &[oracle])
                .expect("blocked reason");

        assert!(reason.contains("stayed paused on Level 00"));
        assert!(reason.contains("Level 15"));
        assert!(reason.contains("knights-isle-probe"));
    }

    #[test]
    fn settled_recording_oracle_bootstrap_block_reason_catches_target_title_card() {
        let recording = NativeRecording {
            manifest: crate::recording::NativeRecordingManifest {
                bundle_version: 4,
                slug: String::from("level2-walk-right"),
                recorded_at: String::from("123"),
                start: crate::recording::RecordingStart {
                    warp_level: Some(0x01),
                    settled: true,
                    record_frames: None,
                },
                rom_sha1: String::new(),
                script_sha1: String::new(),
                trace_sha1: String::new(),
                host_os: String::new(),
                host_arch: String::new(),
                git_commit: None,
                git_dirty: None,
                record_command: String::new(),
                verify_command: String::new(),
                artifacts: Vec::new(),
                scenario: DynamicScenario {
                    slug: String::from("level2-walk-right"),
                    script: vec![0],
                    frame_count: 1,
                    checkpoints: vec![DynamicCheckpoint {
                        label: String::from("start"),
                        frame_index: 0,
                    }],
                    warp_to_level: Some(0x01),
                },
            },
            trace: vec![NativeTraceFrame::default()],
            legacy_block_destroy_direction_frames: BTreeSet::default(),
        };
        let native = semantic_state_at_level(0x01, false);
        let mut oracle = semantic_state_at_level(0x01, true);
        oracle.world.game_mode = 0x0008;
        oracle.world.boot_phase = Some(String::from("starting_level"));

        let reason =
            settled_recording_oracle_bootstrap_block_reason(&recording, &[native], &[oracle])
                .expect("blocked reason");

        assert!(reason.contains("stayed on the Level 01 title card"));
        assert!(reason.contains("level2-walk-right"));
    }

    fn semantic_state_at_level(level: u16, paused: bool) -> SemanticGameState {
        SemanticGameState {
            frame_index: 0,
            player: SemanticPlayerState {
                x_pos: Some(0),
                y_pos: Some(0),
                x_velocity: Some(0),
                y_velocity: Some(0),
                facing: None,
                on_ground: None,
                movement_state: Some(String::from("movement_0000")),
                current_form: Some(0),
                hitpoints: Some(2),
                invulnerable_frames: Some(0),
                collision_type: None,
            },
            camera: SemanticCameraState { x_pos: 0, y_pos: 0 },
            world: SemanticWorldState {
                game_mode: 0x000c,
                current_level_id: level,
                player_1_level_id: level,
                current_player: 0,
                score: Some(0),
                number_lives: Some(3),
                diamond_count: Some(0),
                paused,
                boot_phase: None,
            },
            entities: Vec::new(),
            events: Vec::new(),
        }
    }

    #[test]
    fn semantic_from_gameplay_state_keeps_shared_gameplay_fields() {
        let state = GameplayState {
            label: String::from("frame_00719"),
            frame_index: 719,
            game_mode: 0x000c,
            current_player: 0,
            number_lives: 3,
            number_hitpoints: 1,
            current_level_id: 0,
            player_1_level_id: 0,
            current_helmet_id: 0,
            camera_x_pos: 0x0045_0000,
            camera_y_pos: 0x0073_0000,
            kid_x_pos: 0x00ef_0000,
            kid_y_pos: 0x010f_0000,
            kid_x_vel: Some(0x0002_0000),
            kid_y_vel: Some(0),
            character_movement: Some(2),
            kid_collision_type: Some(0),
            invulnerable_frames: Some(0),
            just_received_damage: Some(false),
            kid_ground_probe: None,
            kid_grounded_release_probe: None,
            kid_jump_gravity_probe: None,
            number_objects: 4,
            number_gfx_objects: 5,
        };

        let semantic = semantic_from_gameplay_state(&state, None);
        assert_eq!(semantic.frame_index, 719);
        assert_eq!(semantic.world.game_mode, 0x000c);
        assert_eq!(semantic.world.number_lives, Some(3));
        assert_eq!(semantic.player.x_pos, Some(0x00ef_0000));
        assert_eq!(
            semantic.player.movement_state.as_deref(),
            Some("movement_0002")
        );
        assert_eq!(semantic.player.collision_type, Some(0));
        assert!(semantic.entities.is_empty());
    }

    #[test]
    fn semantic_from_native_zero_fills_hidden_sega_screen_player_fields() {
        let trace = NativeTraceFrame {
            boot_phase: Some(String::from("sega_screen")),
            ..NativeTraceFrame::default()
        };

        let semantic = semantic_from_native(&trace, None);

        assert_eq!(semantic.player.x_pos, Some(0));
        assert_eq!(semantic.player.y_pos, Some(0));
        assert_eq!(
            semantic.player.movement_state.as_deref(),
            Some("movement_0000")
        );
        assert_eq!(semantic.player.current_form, Some(0));
        assert_eq!(semantic.player.hitpoints, Some(0));
        assert_eq!(semantic.world.number_lives, Some(0));
    }
}
