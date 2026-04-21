mod frames;
mod gameplay;
mod release_gate;

pub(crate) use frames::{capture_frames, diagnose_frame, golden_capture, golden_diff};
pub(crate) use gameplay::run_gameplay_sweep;
pub(crate) use release_gate::ReleaseGateFocusSummary;

// Re-export submodule items needed by tests in this module
#[cfg(test)]
use frames::{diff_golden_frames, golden_scenarios_for_level};
#[cfg(test)]
use gameplay::{check_jump_assertions, check_walk_right_assertions};
#[cfg(test)]
use release_gate::{
    ReleaseGateFieldFocus, extract_release_gate_failed_field,
    extract_release_gate_scenario_level_id, format_release_gate_focus_summary,
    normalise_release_gate_value,
};

use super::*;
use cache::{OracleCacheKey, load_cached_oracle, store_oracle_cache};
use diff::{annotate_diff, frame_mae, generate_diff_composite, pixel_mismatch_count};
use native_player::render_gameplay_frame_evidence_for_level;
use oracle::{JgenesisBackend, OracleBackend, run_jgenesis_capture_dynamic};
use rayon::prelude::*;
use release_gate::{latest_release_gate_failed_fields, summarise_release_gate_focuses};
use scenarios::{DynamicCheckpoint, DynamicScenario, idle_level_scenario};
#[cfg(test)]
use scenarios::{prize_block_scenario, walk_right_scenario};
use sha1::{Digest, Sha1};

/// Advisory fields — diverge due to architectural differences, not native bugs.
/// Promoted to authoritative (removed from list): score, number_diamonds (0 diffs at 0px),
/// murder_wall_x (oracle-pause exceptions handled in classify_state_diff).
pub(crate) const ADVISORY_FIELDS: &[&str] = &[
    "number_objects",   // native doesn't model system objects (palette rotation, etc.)
    "number_hitpoints", // oracle-pause: idle_tail diverges because oracle is paused
    "number_lives",     // oracle-pause: idle_tail diverges because oracle is paused
    "kid_x_vel",        // capture-timing: native post-frame vs oracle VBlank
    "kid_y_vel",        // capture-timing: native post-frame vs oracle VBlank
];

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, clap::ValueEnum)]
#[serde(rename_all = "snake_case")]
pub(crate) enum FrontierKind {
    Gameplay,
    Enemy,
}

/// Position tolerance in FP16.16 units. 6px = 0x0006_0000 accommodates the ±5px
/// PLAYER_HALF_WIDTH offset from gravity settling differences between native
/// (instant floor snap) and oracle (physics-based settling).
const POSITION_TOLERANCE: u32 = 0; // 0px — exact match required

/// Result tier controls how much data is collected per level.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, clap::ValueEnum)]
#[serde(rename_all = "snake_case")]
pub(crate) enum SweepTier {
    /// State-only: NativeTraceFrame comparison, no frame capture.
    State,
    /// State + checkpoint frame capture + MAE comparison.
    Frames,
    /// Full diagnosis: multi-frame capture + drift detection.
    Full,
    /// Scripted gameplay: run walk/jump scenarios, check self-consistency
    /// assertions on the native trace (no oracle needed).
    Gameplay,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum DivergenceCategory {
    Authoritative,
    Advisory,
    Accumulation,
    Rng,
    OracleLimitation,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct DivergenceSummary {
    pub(crate) authoritative: usize,
    pub(crate) advisory: usize,
    pub(crate) accumulation: usize,
    pub(crate) rng: usize,
    pub(crate) oracle_limitation: usize,
}

impl DivergenceSummary {
    pub(crate) fn increment(&mut self, category: DivergenceCategory) {
        match category {
            DivergenceCategory::Authoritative => self.authoritative += 1,
            DivergenceCategory::Advisory => self.advisory += 1,
            DivergenceCategory::Accumulation => self.accumulation += 1,
            DivergenceCategory::Rng => self.rng += 1,
            DivergenceCategory::OracleLimitation => self.oracle_limitation += 1,
        }
    }

    fn merge(&mut self, other: &Self) {
        self.authoritative += other.authoritative;
        self.advisory += other.advisory;
        self.accumulation += other.accumulation;
        self.rng += other.rng;
        self.oracle_limitation += other.oracle_limitation;
    }

    fn is_empty(&self) -> bool {
        self.authoritative == 0
            && self.advisory == 0
            && self.accumulation == 0
            && self.rng == 0
            && self.oracle_limitation == 0
    }
}

/// Per-level sweep result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct LevelSweepResult {
    pub(crate) level_id: u8,
    pub(crate) slug: String,
    pub(crate) tier: SweepTier,
    pub(crate) passed: bool,
    pub(crate) state_diffs: Vec<StateDiff>,
    pub(crate) mae: Option<f32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) pixel_mismatches: Option<usize>,
    pub(crate) diff_images: Vec<String>,
    pub(crate) error: Option<String>,
    #[serde(default, skip_serializing_if = "DivergenceSummary::is_empty")]
    pub(crate) divergence_summary: DivergenceSummary,
}

/// Field-level state divergence.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct StateDiff {
    pub(crate) checkpoint: String,
    pub(crate) field: String,
    pub(crate) native_value: String,
    pub(crate) oracle_value: String,
    pub(crate) category: DivergenceCategory,
}

/// Top-level sweep report.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct SweepReport {
    pub(crate) timestamp: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) kid_core_hash: Option<String>,
    pub(crate) levels: Vec<LevelSweepResult>,
    pub(crate) passed: usize,
    pub(crate) failed: usize,
    pub(crate) errors: usize,
    #[serde(default, skip_serializing_if = "DivergenceSummary::is_empty")]
    pub(crate) divergence_summary: DivergenceSummary,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub(crate) field_summary: Vec<SweepHistoryFieldEntry>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub(crate) release_gate_focus: Vec<ReleaseGateFocusSummary>,
}

/// Per-level golden snapshot: full NativeTraceFrame at each checkpoint.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct GoldenSnapshot {
    pub(crate) level_id: u8,
    pub(crate) scenario_slug: String,
    pub(crate) kid_core_hash: String,
    pub(crate) frames: Vec<NativeTraceFrame>,
}

/// Field-level diff between two golden snapshots.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct GoldenDiff {
    pub(crate) checkpoint: String,
    pub(crate) field: String,
    pub(crate) old_value: String,
    pub(crate) new_value: String,
}

/// Options controlling sweep behaviour beyond tier and level selection.
#[derive(Debug, Clone, Default)]
pub(crate) struct SweepOptions {
    /// Only populate the oracle cache — skip native runs and comparison.
    pub(crate) warm_cache_only: bool,
    /// Stop the sweep after this many failures (0 = no limit).
    pub(crate) max_failures: usize,
}

#[derive(Debug, Clone, Serialize)]
struct SweepHistoryEntry {
    timestamp: String,
    tier: SweepTier,
    levels_requested: usize,
    passed: usize,
    failed: usize,
    errors: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    kid_core_hash: Option<String>,
    divergence_summary: DivergenceSummary,
    report_path: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    field_summary: Vec<SweepHistoryFieldEntry>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    release_gate_focus: Vec<ReleaseGateFocusSummary>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    per_level: Vec<SweepHistoryLevelEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct SweepHistoryFieldEntry {
    field: String,
    total_diffs: usize,
    levels: usize,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    level_ids: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    checkpoints: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    checkpoint_counts: Vec<SweepHistoryCheckpointCount>,
    #[serde(default, skip_serializing_if = "DivergenceSummary::is_empty")]
    divergence_summary: DivergenceSummary,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct SweepHistoryCheckpointCount {
    checkpoint: String,
    diffs: usize,
}

#[derive(Debug, Clone, Serialize)]
struct SweepHistoryLevelEntry {
    level: String,
    passed: bool,
    blocking: bool,
    authoritative: usize,
    advisory: usize,
    accumulation: usize,
    rng: usize,
    oracle_limitation: usize,
    fields: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
struct LevelSweepTelemetry {
    commit: String,
    timestamp: String,
    level: String,
    tier: SweepTier,
    authoritative: usize,
    advisory: usize,
    accumulation: usize,
    rng: usize,
    oracle_limitation: usize,
    fields: Vec<String>,
    passed: bool,
}

/// Kid Chameleon level IDs are BCD-encoded: 0x00-0x09, 0x10-0x19, …, 0x60-0x68.
/// Returns true if `id` is a valid BCD level ID.
fn is_valid_level_id(id: u8) -> bool {
    let lo = id & 0x0F;
    let hi = id >> 4;
    lo <= 9 && hi <= 6 && id <= 0x68
}

fn is_oracle_limitation_only(result: &LevelSweepResult) -> bool {
    result.divergence_summary.oracle_limitation > 0
        && result.divergence_summary.authoritative == 0
        && result.divergence_summary.accumulation == 0
        && result.divergence_summary.rng == 0
        && result.mae.is_none_or(|mae| mae < 32.0)
}

fn has_blocking_divergences(summary: &DivergenceSummary) -> bool {
    summary.authoritative > 0 || summary.accumulation > 0 || summary.rng > 0
}

fn level_result_status(r: &LevelSweepResult) -> &'static str {
    let has_advisory = r.divergence_summary.advisory > 0;
    let has_oracle_limitation = r.divergence_summary.oracle_limitation > 0;

    if has_blocking_divergences(&r.divergence_summary) {
        "FAIL"
    } else {
        match (has_advisory, has_oracle_limitation) {
            (true, true) => "NON-BLOCKING",
            (true, false) => "ADVISORY",
            (false, true) => "ORACLE LIMITATION",
            (false, false) => "PASS",
        }
    }
}

fn format_divergence_summary(summary: &DivergenceSummary) -> String {
    let mut parts = Vec::new();

    let blocking = summary.authoritative + summary.accumulation + summary.rng;
    if blocking > 0 {
        parts.push(format!("{blocking} blocking diffs"));
    }
    if summary.advisory > 0 {
        parts.push(format!("{} advisory diffs", summary.advisory));
    }
    if summary.oracle_limitation > 0 {
        parts.push(format!("{} oracle limitations", summary.oracle_limitation));
    }

    if parts.is_empty() {
        String::new()
    } else {
        format!(" ({})", parts.join(", "))
    }
}

fn format_field_category_summary(summary: &DivergenceSummary) -> String {
    let mut parts = Vec::new();
    if summary.authoritative > 0 {
        parts.push(format!("authoritative={}", summary.authoritative));
    }
    if summary.advisory > 0 {
        parts.push(format!("advisory={}", summary.advisory));
    }
    if summary.accumulation > 0 {
        parts.push(format!("accumulation={}", summary.accumulation));
    }
    if summary.rng > 0 {
        parts.push(format!("rng={}", summary.rng));
    }
    if summary.oracle_limitation > 0 {
        parts.push(format!("oracle_limitations={}", summary.oracle_limitation));
    }

    if parts.is_empty() {
        String::new()
    } else {
        format!(" [{}]", parts.join(", "))
    }
}

fn format_field_level_preview(level_ids: &[String]) -> String {
    if level_ids.is_empty() {
        return String::new();
    }

    const PREVIEW_LIMIT: usize = 5;
    let preview = level_ids
        .iter()
        .take(PREVIEW_LIMIT)
        .cloned()
        .collect::<Vec<_>>()
        .join(", ");
    if level_ids.len() <= PREVIEW_LIMIT {
        format!(" [{preview}]")
    } else {
        format!(
            " [{preview}, +{} more]",
            level_ids.len().saturating_sub(PREVIEW_LIMIT)
        )
    }
}

fn format_checkpoint_count_preview(checkpoints: &[SweepHistoryCheckpointCount]) -> String {
    if checkpoints.is_empty() {
        return String::new();
    }

    const PREVIEW_LIMIT: usize = 3;
    let preview = checkpoints
        .iter()
        .take(PREVIEW_LIMIT)
        .map(|entry| format!("{}={}", entry.checkpoint, entry.diffs))
        .collect::<Vec<_>>()
        .join(", ");
    if checkpoints.len() <= PREVIEW_LIMIT {
        format!(" checkpoints=[{preview}]")
    } else {
        format!(
            " checkpoints=[{preview}, +{} more]",
            checkpoints.len().saturating_sub(PREVIEW_LIMIT)
        )
    }
}

// Release-gate types and functions: see release_gate.rs

/// Convert a raw `sweep_single_level` result into a finalised `LevelSweepResult`,
/// returning `(result, is_failure)` for fast-fail counting.
fn finalise_level_result(
    level_id: u8,
    tier: SweepTier,
    result: Result<LevelSweepResult>,
) -> (LevelSweepResult, bool) {
    match result {
        Ok(r) => {
            let failed = !r.passed && !is_oracle_limitation_only(&r);
            (r, failed)
        }
        Err(e) => {
            let r = LevelSweepResult {
                level_id,
                slug: format!("idle_level_{level_id:02x}"),
                tier,
                passed: false,
                state_diffs: Vec::new(),
                mae: None,
                pixel_mismatches: None,
                diff_images: Vec::new(),
                error: Some(format!("{e:#}")),
                divergence_summary: divergence_summary_with(DivergenceCategory::Authoritative),
            };
            (r, true)
        }
    }
}

/// Print the status line for a completed level sweep result.
fn print_level_result(r: &LevelSweepResult) {
    if let Some(ref err) = r.error {
        println!("  level {:#04x}: ERROR: {err}", r.level_id);
        return;
    }
    let status = level_result_status(r);
    let mae_str = r.mae.map(|v| format!(" MAE={v:.1}")).unwrap_or_default();
    let px_str = r
        .pixel_mismatches
        .map(|v| format!(" px_diff={v}"))
        .unwrap_or_default();
    let diff_count = format_divergence_summary(&r.divergence_summary);
    println!(
        "  level {:#04x}: {status}{mae_str}{px_str}{diff_count}",
        r.level_id
    );
}

/// Parse a level range spec like "0-104", "0,5,10", or "0-10".
/// Invalid BCD level IDs (0x0A-0x0F, 0x1A-0x1F, etc.) are filtered out.
pub(crate) fn parse_level_range(spec: &str) -> Result<Vec<u8>> {
    let mut levels = Vec::new();
    for part in spec.split(',') {
        let part = part.trim();
        if let Some((start, end)) = part.split_once('-') {
            let start: u8 = start.trim().parse().context("invalid level range start")?;
            let end: u8 = end.trim().parse().context("invalid level range end")?;
            if end < start {
                bail!("level range end {end} < start {start}");
            }
            levels.extend((start..=end).filter(|&id| is_valid_level_id(id)));
        } else {
            let level: u8 = part.parse().context("invalid level id")?;
            if is_valid_level_id(level) {
                levels.push(level);
            }
        }
    }
    levels.sort_unstable();
    levels.dedup();
    Ok(levels)
}

/// Run a sweep across the specified levels.
pub(crate) fn run_sweep(
    repo_root: &Path,
    rom_path: &Path,
    manifest: &AssetManifest,
    levels: &[u8],
    tier: SweepTier,
) -> Result<SweepReport> {
    run_sweep_with_options(
        repo_root,
        rom_path,
        manifest,
        levels,
        tier,
        &SweepOptions::default(),
    )
}

/// Run a sweep with additional options (warm-cache-only, fast-fail).
pub(crate) fn run_sweep_with_options(
    repo_root: &Path,
    rom_path: &Path,
    manifest: &AssetManifest,
    levels: &[u8],
    tier: SweepTier,
    options: &SweepOptions,
) -> Result<SweepReport> {
    // Gameplay tier: self-consistency only, no oracle needed.
    if tier == SweepTier::Gameplay {
        return Ok(run_gameplay_sweep(manifest, levels));
    }

    let rom_bytes = fs::read(rom_path)
        .with_context(|| format!("failed to read ROM from {}", rom_path.display()))?;

    let env = ensure_oracles(repo_root)?;
    let backend: Box<dyn OracleBackend> = Box::new(JgenesisBackend { env: env.clone() });

    // Warm-cache-only: populate oracle cache in parallel, skip native runs.
    if options.warm_cache_only {
        return Ok(warm_oracle_cache(
            &rom_bytes, rom_path, repo_root, &env, levels,
        ));
    }

    let timestamp = chrono_timestamp();
    let run_dir = repo_root.join(".cache/sweep-runs").join(&timestamp);
    fs::create_dir_all(&run_dir)
        .with_context(|| format!("failed to create {}", run_dir.display()))?;

    // Symlink latest → this run
    let latest_link = repo_root.join(".cache/sweep-runs/latest");
    let _ = fs::remove_file(&latest_link);
    #[cfg(unix)]
    std::os::unix::fs::symlink(&run_dir, &latest_link).ok();

    let results = if options.max_failures > 0 {
        // Sequential with fast-fail
        let mut results = Vec::with_capacity(levels.len());
        let mut failure_count: usize = 0;

        for &level_id in levels {
            println!("sweep level {level_id:#04x}...");
            let result = sweep_single_level(
                repo_root, rom_path, &rom_bytes, manifest, &*backend, &run_dir, level_id, tier,
            );
            let (r, failed) = finalise_level_result(level_id, tier, result);
            print_level_result(&r);
            if failed {
                failure_count += 1;
            }
            results.push(r);

            if failure_count >= options.max_failures {
                println!(
                    "\nfast-fail: stopping after {failure_count} failure(s) (--max-failures {})",
                    options.max_failures
                );
                break;
            }
        }
        results
    } else {
        // Parallel execution — run all levels concurrently via rayon
        let parallel_results: Vec<(u8, Result<LevelSweepResult>)> = levels
            .par_iter()
            .map(|&level_id| {
                println!("sweep level {level_id:#04x}...");
                let result = sweep_single_level(
                    repo_root, rom_path, &rom_bytes, manifest, &*backend, &run_dir, level_id, tier,
                );
                (level_id, result)
            })
            .collect();

        let mut results = Vec::with_capacity(parallel_results.len());
        for (level_id, result) in parallel_results {
            let (r, _failed) = finalise_level_result(level_id, tier, result);
            print_level_result(&r);
            results.push(r);
        }
        results
    };

    let passed = results.iter().filter(|r| r.passed).count();
    let failed = results
        .iter()
        .filter(|r| !r.passed && r.error.is_none() && !is_oracle_limitation_only(r))
        .count();
    let errors = results
        .iter()
        .filter(|r| r.error.is_some() && !is_oracle_limitation_only(r))
        .count();
    let divergence_summary = summarise_results(&results);
    let field_summary = summarise_field_diffs(&results);

    let source_hash = kid_core_source_hash(repo_root).ok();
    let release_gate_focus = summarise_release_gate_focuses(
        &results,
        &field_summary,
        &latest_release_gate_failed_fields(repo_root),
    );
    let report = SweepReport {
        timestamp,
        kid_core_hash: source_hash,
        levels: results,
        passed,
        failed,
        errors,
        divergence_summary,
        field_summary,
        release_gate_focus,
    };

    // Emit per-level telemetry to JSONL
    let commit_hash = std::process::Command::new("git")
        .args(["rev-parse", "--short", "HEAD"])
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map_or_else(|| "unknown".into(), |s| s.trim().to_string());
    let telemetry_path = repo_root.join(".cache/metrics/sweep-telemetry.jsonl");
    if let Ok(mut telemetry_file) = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&telemetry_path)
    {
        use std::io::Write;
        for result in &report.levels {
            let mut fields: Vec<String> =
                result.state_diffs.iter().map(|d| d.field.clone()).collect();
            fields.sort_unstable();
            fields.dedup();
            let entry = LevelSweepTelemetry {
                commit: commit_hash.clone(),
                timestamp: report.timestamp.clone(),
                level: format!("{:#04x}", result.level_id),
                tier,
                authoritative: result.divergence_summary.authoritative,
                advisory: result.divergence_summary.advisory,
                accumulation: result.divergence_summary.accumulation,
                rng: result.divergence_summary.rng,
                oracle_limitation: result.divergence_summary.oracle_limitation,
                fields,
                passed: result.passed,
            };
            if let Ok(line) = serde_json::to_string(&entry) {
                let _ = writeln!(telemetry_file, "{line}");
            }
        }
    }

    // Write report JSON
    let report_path = run_dir.join("report.json");
    fs::write(&report_path, serde_json::to_vec_pretty(&report)?)
        .with_context(|| format!("failed to write {}", report_path.display()))?;
    append_sweep_history(
        repo_root,
        &SweepHistoryEntry {
            timestamp: report.timestamp.clone(),
            tier,
            levels_requested: levels.len(),
            passed: report.passed,
            failed: report.failed,
            errors: report.errors,
            kid_core_hash: report.kid_core_hash.clone(),
            divergence_summary: report.divergence_summary.clone(),
            report_path: report_path.to_string_lossy().into_owned(),
            field_summary: report.field_summary.clone(),
            release_gate_focus: report.release_gate_focus.clone(),
            per_level: report
                .levels
                .iter()
                .map(|result| {
                    let mut fields: Vec<String> =
                        result.state_diffs.iter().map(|d| d.field.clone()).collect();
                    fields.sort_unstable();
                    fields.dedup();
                    SweepHistoryLevelEntry {
                        level: format!("{:#04x}", result.level_id),
                        passed: result.passed,
                        blocking: has_blocking_divergences(&result.divergence_summary),
                        authoritative: result.divergence_summary.authoritative,
                        advisory: result.divergence_summary.advisory,
                        accumulation: result.divergence_summary.accumulation,
                        rng: result.divergence_summary.rng,
                        oracle_limitation: result.divergence_summary.oracle_limitation,
                        fields,
                    }
                })
                .collect(),
        },
    )?;

    let divergence_summary = format_divergence_summary(&report.divergence_summary);
    println!(
        "\nsweep complete: {passed} passed, {failed} failed, {errors} errors{divergence_summary}"
    );
    if !report.field_summary.is_empty() {
        let summary = report
            .field_summary
            .iter()
            .take(5)
            .map(|entry| {
                format!(
                    "{}={} diffs across {} levels{}{}{}",
                    entry.field,
                    entry.total_diffs,
                    entry.levels,
                    format_field_category_summary(&entry.divergence_summary),
                    format_field_level_preview(&entry.level_ids),
                    format_checkpoint_count_preview(&entry.checkpoint_counts)
                )
            })
            .collect::<Vec<_>>()
            .join(", ");
        println!("top diff fields: {summary}");
    }
    if !report.release_gate_focus.is_empty() {
        let summary = report
            .release_gate_focus
            .iter()
            .map(|focus| focus.summary.clone())
            .collect::<Vec<_>>()
            .join(", ");
        println!("release-gate focus: {summary}");
    }
    println!("report: {}", report_path.display());

    Ok(report)
}

fn sweep_single_level(
    repo_root: &Path,
    rom_path: &Path,
    rom_bytes: &[u8],
    manifest: &AssetManifest,
    backend: &dyn OracleBackend,
    run_dir: &Path,
    level_id: u8,
    requested_tier: SweepTier,
) -> Result<LevelSweepResult> {
    let scenario = idle_level_scenario(level_id, None);
    let level_dir = run_dir.join(format!("levels/{level_id:02x}"));
    fs::create_dir_all(&level_dir)
        .with_context(|| format!("failed to create {}", level_dir.display()))?;

    // Run native (with warp if specified)
    let native_trace = kid_core::run_script_with_warp(
        manifest.clone(),
        &scenario.script,
        scenario.frame_count,
        scenario.warp_to_level,
    );

    // Extract native state at checkpoints
    let native_states = extract_checkpoint_states(&native_trace, &scenario.checkpoints);

    // Run oracle (with caching)
    let cache_key = OracleCacheKey::from_scenario(rom_bytes, &scenario, backend.name());
    let oracle = if let Some(cached) = load_cached_oracle(&cache_key, repo_root) {
        cached
    } else {
        // Write scenario file for oracle
        let scenario_file = level_dir.join("scenario.json");
        fs::write(&scenario_file, serde_json::to_vec_pretty(&scenario)?)
            .with_context(|| format!("failed to write {}", scenario_file.display()))?;

        let oracle_dir = level_dir.join("oracle");
        fs::create_dir_all(&oracle_dir)?;
        let oracle = backend.run_scenario(rom_path, &scenario_file, &oracle_dir, None)?;
        let _ = store_oracle_cache(&cache_key, repo_root, &oracle, &oracle_dir);
        oracle
    };

    // Detect broken oracle warps: if the oracle's game_mode is clearly invalid
    // (e.g. 0x4e73 instead of 0x0000-0x0014), the warp failed and oracle data
    // is garbage. Report as error rather than native failure.
    let oracle_warp_broken = oracle.states.iter().any(|s| s.game_mode > 0x0014);
    if oracle_warp_broken {
        return Ok(LevelSweepResult {
            level_id,
            slug: format!("idle_level_{level_id:02x}"),
            tier: requested_tier,
            passed: false,
            state_diffs: Vec::new(),
            mae: None,
            pixel_mismatches: None,
            diff_images: Vec::new(),
            error: Some(format!(
                "oracle warp broken: game_mode={:#06x}",
                oracle.states.first().map_or(0, |s| s.game_mode)
            )),
            divergence_summary: divergence_summary_with(DivergenceCategory::OracleLimitation),
        });
    }

    // Compare states
    let mut state_diffs = compare_sweep_states(&native_states, &oracle.states);
    // Build set of checkpoint labels where the native player died during idle
    // but the oracle survived. Detected by: (a) native HP=0, or (b) native
    // lives decreased between checkpoints (died and respawned before capture).
    let first_native_lives = native_states.first().map(|s| s.number_lives);
    let death_gap_checkpoints: std::collections::HashSet<&str> = native_states
        .iter()
        .zip(oracle.states.iter())
        .filter(|(n, o)| {
            (n.number_hitpoints == 0 && o.number_hitpoints > 0)
                || first_native_lives.is_some_and(|first| n.number_lives < first)
        })
        .map(|(n, _)| n.label.as_str())
        .collect();
    // Detect oracle pause: oracle game is paused (in-game pause menu) or
    // stuck on title card (game_mode=0x0008). Position/mode diffs are
    // meaningless because the oracle never ran gameplay.
    let oracle_paused_checkpoints: std::collections::HashSet<&str> = oracle
        .states
        .iter()
        .filter(|s| s.game_paused || s.game_mode == 0x0008)
        .map(|s| s.label.as_str())
        .collect();
    reclassify_state_diffs(
        &mut state_diffs,
        &death_gap_checkpoints,
        &oracle_paused_checkpoints,
    );
    let divergence_summary = summarise_state_diffs(&state_diffs);
    let state_passed = !has_blocking_divergences(&divergence_summary);

    // Tier 1 (state-only): done unless escalating
    let effective_tier = if !state_passed && requested_tier == SweepTier::State {
        SweepTier::Frames // auto-escalate on failure
    } else {
        requested_tier
    };

    let mut mae_val = None;
    let mut pixel_mismatches_val = None;
    let mut diff_images = Vec::new();

    if effective_tier == SweepTier::Frames || effective_tier == SweepTier::Full {
        // Capture native frames at checkpoints and compare with oracle PNGs
        let (best_mae, best_pixel_mismatches) = compare_checkpoint_frames(
            manifest,
            &native_trace,
            &scenario,
            &oracle,
            &level_dir,
            level_id,
            &oracle_paused_checkpoints,
        )?;
        mae_val = best_mae;
        pixel_mismatches_val = best_pixel_mismatches;

        // Collect diff image paths
        for entry in fs::read_dir(&level_dir)? {
            let entry = entry?;
            let name = entry.file_name().to_string_lossy().to_string();
            if name.starts_with("diff_")
                && std::path::Path::new(&name)
                    .extension()
                    .is_some_and(|ext| ext.eq_ignore_ascii_case("png"))
            {
                diff_images.push(entry.path().to_string_lossy().to_string());
            }
        }
    }

    // Goal: 0 MAE (pixel-perfect). Current threshold kept at 32 for compatibility
    // with oracle-pause artefacts. Use pixel_mismatches for strict comparison.
    let passed = state_passed && mae_val.is_none_or(|v| v < 32.0);

    Ok(LevelSweepResult {
        level_id,
        slug: scenario.slug,
        tier: effective_tier,
        passed,
        state_diffs,
        mae: mae_val,
        pixel_mismatches: pixel_mismatches_val,
        diff_images,
        error: None,
        divergence_summary,
    })
}

fn reclassify_state_diffs(
    diffs: &mut [StateDiff],
    death_gap_checkpoints: &std::collections::HashSet<&str>,
    oracle_paused_checkpoints: &std::collections::HashSet<&str>,
) {
    for diff in diffs {
        diff.category = classify_state_diff(diff, death_gap_checkpoints, oracle_paused_checkpoints);
    }
}

pub(crate) fn summarise_state_diffs(diffs: &[StateDiff]) -> DivergenceSummary {
    let mut summary = DivergenceSummary::default();
    for diff in diffs {
        summary.increment(diff.category);
    }
    summary
}

fn classify_state_diff(
    diff: &StateDiff,
    death_gap_checkpoints: &std::collections::HashSet<&str>,
    oracle_paused_checkpoints: &std::collections::HashSet<&str>,
) -> DivergenceCategory {
    if ADVISORY_FIELDS.contains(&diff.field.as_str()) {
        return DivergenceCategory::Advisory;
    }

    if death_gap_checkpoints.contains(diff.checkpoint.as_str())
        && matches!(
            diff.field.as_str(),
            "kid_x_pos" | "kid_y_pos" | "camera_x_pos" | "camera_y_pos"
        )
    {
        return DivergenceCategory::Advisory;
    }

    // When the oracle is paused (stuck in transition or pause menu), its
    // entire game state is unreliable — not just positions. Classify all
    // remaining diffs as oracle limitation.
    if oracle_paused_checkpoints.contains(diff.checkpoint.as_str()) {
        return DivergenceCategory::OracleLimitation;
    }

    DivergenceCategory::Authoritative
}

fn divergence_summary_with(category: DivergenceCategory) -> DivergenceSummary {
    let mut summary = DivergenceSummary::default();
    summary.increment(category);
    summary
}

fn summarise_results(results: &[LevelSweepResult]) -> DivergenceSummary {
    let mut summary = DivergenceSummary::default();
    for result in results {
        summary.merge(&result.divergence_summary);
    }
    summary
}

fn summarise_field_diffs(results: &[LevelSweepResult]) -> Vec<SweepHistoryFieldEntry> {
    let mut totals: std::collections::BTreeMap<String, usize> = std::collections::BTreeMap::new();
    let mut levels: std::collections::BTreeMap<String, std::collections::BTreeSet<u8>> =
        std::collections::BTreeMap::new();
    let mut checkpoints: std::collections::BTreeMap<String, std::collections::BTreeSet<String>> =
        std::collections::BTreeMap::new();
    let mut checkpoint_counts: std::collections::BTreeMap<
        String,
        std::collections::BTreeMap<String, usize>,
    > = std::collections::BTreeMap::new();
    let mut divergence_summaries: std::collections::BTreeMap<String, DivergenceSummary> =
        std::collections::BTreeMap::new();

    for result in results {
        for diff in &result.state_diffs {
            *totals.entry(diff.field.clone()).or_default() += 1;
            levels
                .entry(diff.field.clone())
                .or_default()
                .insert(result.level_id);
            checkpoints
                .entry(diff.field.clone())
                .or_default()
                .insert(diff.checkpoint.clone());
            *checkpoint_counts
                .entry(diff.field.clone())
                .or_default()
                .entry(diff.checkpoint.clone())
                .or_default() += 1;
            divergence_summaries
                .entry(diff.field.clone())
                .or_default()
                .increment(diff.category);
        }
    }

    let mut summary = totals
        .into_iter()
        .map(|(field, total_diffs)| SweepHistoryFieldEntry {
            levels: levels
                .get(&field)
                .map_or(0, std::collections::BTreeSet::len),
            level_ids: levels
                .get(&field)
                .map(|ids| {
                    ids.iter()
                        .map(|level_id| format!("{level_id:#04x}"))
                        .collect()
                })
                .unwrap_or_default(),
            checkpoints: checkpoints
                .get(&field)
                .map(|labels| labels.iter().cloned().collect())
                .unwrap_or_default(),
            checkpoint_counts: checkpoint_counts
                .remove(&field)
                .map(|counts| {
                    let mut entries = counts
                        .into_iter()
                        .map(|(checkpoint, diffs)| SweepHistoryCheckpointCount {
                            checkpoint,
                            diffs,
                        })
                        .collect::<Vec<_>>();
                    entries.sort_by(|left, right| {
                        right
                            .diffs
                            .cmp(&left.diffs)
                            .then_with(|| left.checkpoint.cmp(&right.checkpoint))
                    });
                    entries
                })
                .unwrap_or_default(),
            divergence_summary: divergence_summaries.remove(&field).unwrap_or_default(),
            field,
            total_diffs,
        })
        .collect::<Vec<_>>();
    summary.sort_by(|left, right| {
        right
            .total_diffs
            .cmp(&left.total_diffs)
            .then_with(|| right.levels.cmp(&left.levels))
            .then_with(|| left.field.cmp(&right.field))
    });
    summary
}

/// Compare per-frame CRC arrays, returning the index of the first mismatch.
pub(crate) fn find_first_crc_mismatch(native_crcs: &[u32], oracle_crcs: &[u32]) -> Option<usize> {
    native_crcs
        .iter()
        .zip(oracle_crcs.iter())
        .position(|(n, o)| n != o)
}

/// Extract NativeTraceFrame fields at checkpoint frame indices, returning
/// OracleState-compatible structs for comparison.
pub(crate) fn extract_checkpoint_states(
    trace: &[NativeTraceFrame],
    checkpoints: &[DynamicCheckpoint],
) -> Vec<OracleState> {
    checkpoints
        .iter()
        .filter_map(|cp| {
            let frame = trace.get(cp.frame_index)?;
            let main_menu_option = match frame.game_mode {
                kid_core::GameMode::TitleScreen => frame.options_cursor,
                kid_core::GameMode::OptionsMenu => frame.options_cursor,
                _ => None,
            };
            Some(OracleState {
                label: cp.label.clone(),
                frame_index: cp.frame_index,
                game_mode: frame.game_mode_value,
                boot_phase: frame.boot_phase.clone(),
                boot_phase_elapsed: frame.boot_phase_elapsed,
                title_screen_ready: frame.title_screen_ready,
                main_menu_option,
                current_player: frame.current_player,
                number_hitpoints: frame.number_hitpoints,
                current_helmet_id: frame.current_helmet_id,
                current_level_id: frame.current_level.0 as u16,
                player_1_level_id: frame.player_1_level_id.0 as u16,
                camera_x_pos: frame.camera_x_pos,
                camera_y_pos: frame.camera_y_pos,
                kid_x_pos: frame.kid_x_pos.unwrap_or(0),
                kid_y_pos: frame.kid_y_pos.unwrap_or(0),
                number_objects: frame.number_objects,
                number_gfx_objects: frame.number_gfx_objects,
                game_paused: frame.paused,
                active_enemies: crate::compare::active_enemy_states_from_native_frame(frame),
                score: frame.score,
                number_lives: frame.number_lives,
                number_diamonds: frame.diamond_count,
                diamond_power_active: frame.diamond_power_active,
                murder_wall_x: frame.murder_wall_x.map(|x| x as i32),
                kid_x_vel: frame.kid_x_velocity,
                kid_y_vel: frame.kid_y_velocity,
                character_movement: frame.character_movement,
                just_received_damage: Some(frame.just_received_damage),
                kid_collision_type: None,
                kid_collision_source: None,
                invulnerable_frames: Some(frame.invulnerable_frames as u8),
                currently_transforming: Some(frame.form_change_flash_frames > 0),
                rng_offset: None,            // native uses different RNG system
                level_special_effects: None, // native doesn't track this separately
                gameplay_debug_fields: frame.gameplay_debug_fields.clone(),
                cpu_registers: None,
            })
        })
        .collect()
}

/// Compare native vs oracle state at each checkpoint, returning diffs.
pub(crate) fn compare_sweep_states(
    native_states: &[OracleState],
    oracle_states: &[OracleState],
) -> Vec<StateDiff> {
    compare_sweep_states_for_frontier(native_states, oracle_states, FrontierKind::Gameplay)
}

pub(crate) fn compare_sweep_states_for_frontier(
    native_states: &[OracleState],
    oracle_states: &[OracleState],
    frontier_kind: FrontierKind,
) -> Vec<StateDiff> {
    let mut diffs = Vec::new();

    for (native, oracle) in native_states.iter().zip(oracle_states.iter()) {
        let cp = &native.label;
        if native.game_mode != oracle.game_mode {
            diffs.push(StateDiff {
                checkpoint: cp.clone(),
                field: "game_mode".into(),
                native_value: format!("{:#06x}", native.game_mode),
                oracle_value: format!("{:#06x}", oracle.game_mode),
                category: DivergenceCategory::Authoritative,
            });
        }
        if native.current_level_id != oracle.current_level_id {
            diffs.push(StateDiff {
                checkpoint: cp.clone(),
                field: "current_level_id".into(),
                native_value: format!("{:#06x}", native.current_level_id),
                oracle_value: format!("{:#06x}", oracle.current_level_id),
                category: DivergenceCategory::Authoritative,
            });
        }
        let cam_x_delta = native.camera_x_pos.abs_diff(oracle.camera_x_pos);
        if cam_x_delta > POSITION_TOLERANCE {
            diffs.push(StateDiff {
                checkpoint: cp.clone(),
                field: "camera_x_pos".into(),
                native_value: format!("{:#010x}", native.camera_x_pos),
                oracle_value: format!("{:#010x}", oracle.camera_x_pos),
                category: DivergenceCategory::Authoritative,
            });
        }
        let cam_y_delta = native.camera_y_pos.abs_diff(oracle.camera_y_pos);
        if cam_y_delta > POSITION_TOLERANCE {
            diffs.push(StateDiff {
                checkpoint: cp.clone(),
                field: "camera_y_pos".into(),
                native_value: format!("{:#010x}", native.camera_y_pos),
                oracle_value: format!("{:#010x}", oracle.camera_y_pos),
                category: DivergenceCategory::Authoritative,
            });
        }
        if native.number_objects != oracle.number_objects {
            diffs.push(StateDiff {
                checkpoint: cp.clone(),
                field: "number_objects".into(),
                native_value: format!("{}", native.number_objects),
                oracle_value: format!("{}", oracle.number_objects),
                category: DivergenceCategory::Authoritative,
            });
        }
        if native.number_hitpoints != oracle.number_hitpoints {
            diffs.push(StateDiff {
                checkpoint: cp.clone(),
                field: "number_hitpoints".into(),
                native_value: format!("{}", native.number_hitpoints),
                oracle_value: format!("{}", oracle.number_hitpoints),
                category: DivergenceCategory::Authoritative,
            });
        }
        if native.kid_x_pos.abs_diff(oracle.kid_x_pos) > POSITION_TOLERANCE {
            diffs.push(StateDiff {
                checkpoint: cp.clone(),
                field: "kid_x_pos".into(),
                native_value: format!("{:#010x}", native.kid_x_pos),
                oracle_value: format!("{:#010x}", oracle.kid_x_pos),
                category: DivergenceCategory::Authoritative,
            });
        }
        if native.kid_y_pos.abs_diff(oracle.kid_y_pos) > POSITION_TOLERANCE {
            diffs.push(StateDiff {
                checkpoint: cp.clone(),
                field: "kid_y_pos".into(),
                native_value: format!("{:#010x}", native.kid_y_pos),
                oracle_value: format!("{:#010x}", oracle.kid_y_pos),
                category: DivergenceCategory::Authoritative,
            });
        }
        if native.score != oracle.score {
            diffs.push(StateDiff {
                checkpoint: cp.clone(),
                field: "score".into(),
                native_value: format!("{}", native.score),
                oracle_value: format!("{}", oracle.score),
                category: DivergenceCategory::Authoritative,
            });
        }
        if native.number_lives != oracle.number_lives {
            diffs.push(StateDiff {
                checkpoint: cp.clone(),
                field: "number_lives".into(),
                native_value: format!("{}", native.number_lives),
                oracle_value: format!("{}", oracle.number_lives),
                category: DivergenceCategory::Authoritative,
            });
        }
        if native.number_diamonds != oracle.number_diamonds {
            diffs.push(StateDiff {
                checkpoint: cp.clone(),
                field: "number_diamonds".into(),
                native_value: format!("{}", native.number_diamonds),
                oracle_value: format!("{}", oracle.number_diamonds),
                category: DivergenceCategory::Authoritative,
            });
        }
        if native.current_helmet_id != oracle.current_helmet_id {
            diffs.push(StateDiff {
                checkpoint: cp.clone(),
                field: "current_helmet_id".into(),
                native_value: format!("{}", native.current_helmet_id),
                oracle_value: format!("{}", oracle.current_helmet_id),
                category: DivergenceCategory::Authoritative,
            });
        }
        if native.diamond_power_active != oracle.diamond_power_active {
            diffs.push(StateDiff {
                checkpoint: cp.clone(),
                field: "diamond_power_active".into(),
                native_value: format!("{}", native.diamond_power_active),
                oracle_value: format!("{}", oracle.diamond_power_active),
                category: DivergenceCategory::Authoritative,
            });
        }
        if native.murder_wall_x != oracle.murder_wall_x {
            diffs.push(StateDiff {
                checkpoint: cp.clone(),
                field: "murder_wall_x".into(),
                native_value: format!("{:?}", native.murder_wall_x),
                oracle_value: format!("{:?}", oracle.murder_wall_x),
                category: DivergenceCategory::Authoritative,
            });
        }
        // Extended fields — velocity comparison (authoritative when both available)
        if let (Some(n_xv), Some(o_xv)) = (native.kid_x_vel, oracle.kid_x_vel)
            && n_xv != o_xv
        {
            diffs.push(StateDiff {
                checkpoint: cp.clone(),
                field: "kid_x_vel".into(),
                native_value: format!("{n_xv:#010x}"),
                oracle_value: format!("{o_xv:#010x}"),
                category: DivergenceCategory::Authoritative,
            });
        }
        if let (Some(n_yv), Some(o_yv)) = (native.kid_y_vel, oracle.kid_y_vel)
            && n_yv != o_yv
        {
            diffs.push(StateDiff {
                checkpoint: cp.clone(),
                field: "kid_y_vel".into(),
                native_value: format!("{n_yv:#010x}"),
                oracle_value: format!("{o_yv:#010x}"),
                category: DivergenceCategory::Authoritative,
            });
        }
        if let (Some(n_inv), Some(o_inv)) = (native.invulnerable_frames, oracle.invulnerable_frames)
            && n_inv != o_inv
        {
            diffs.push(StateDiff {
                checkpoint: cp.clone(),
                field: "invulnerable_frames".into(),
                native_value: format!("{n_inv}"),
                oracle_value: format!("{o_inv}"),
                category: DivergenceCategory::Authoritative,
            });
        }
        if let (Some(n_transform), Some(o_transform)) =
            (native.currently_transforming, oracle.currently_transforming)
            && n_transform != o_transform
        {
            diffs.push(StateDiff {
                checkpoint: cp.clone(),
                field: "currently_transforming".into(),
                native_value: format!("{n_transform}"),
                oracle_value: format!("{o_transform}"),
                category: DivergenceCategory::Authoritative,
            });
        }
        if let (Some(n_movement), Some(o_movement)) =
            (native.character_movement, oracle.character_movement)
            && n_movement != o_movement
        {
            diffs.push(StateDiff {
                checkpoint: cp.clone(),
                field: "character_movement".into(),
                native_value: format!("{n_movement:#06x}"),
                oracle_value: format!("{o_movement:#06x}"),
                category: DivergenceCategory::Authoritative,
            });
        }
        if let (Some(n_damage), Some(o_damage)) =
            (native.just_received_damage, oracle.just_received_damage)
            && n_damage != o_damage
        {
            diffs.push(StateDiff {
                checkpoint: cp.clone(),
                field: "just_received_damage".into(),
                native_value: format!("{n_damage}"),
                oracle_value: format!("{o_damage}"),
                category: DivergenceCategory::Authoritative,
            });
        }
        if native.game_paused != oracle.game_paused {
            diffs.push(StateDiff {
                checkpoint: cp.clone(),
                field: "game_paused".into(),
                native_value: format!("{}", native.game_paused),
                oracle_value: format!("{}", oracle.game_paused),
                category: DivergenceCategory::Authoritative,
            });
        }
        if matches!(frontier_kind, FrontierKind::Enemy)
            && native.active_enemies != oracle.active_enemies
        {
            diffs.push(StateDiff {
                checkpoint: cp.clone(),
                field: "enemy_roster".into(),
                native_value: summarise_enemy_states(&native.active_enemies),
                oracle_value: summarise_enemy_states(&oracle.active_enemies),
                category: DivergenceCategory::Authoritative,
            });
        }
    }

    diffs
}

fn summarise_enemy_states(enemies: &[OracleEnemyState]) -> String {
    enemies
        .iter()
        .map(|enemy| {
            format!(
                "{:02X}:{:08X},{:08X}:{:?}:li={:?}:lp={:?}:g={:?}",
                enemy.object_type,
                enemy.x_pos,
                enemy.y_pos,
                enemy.facing,
                enemy.layout_index,
                enemy.layout_ptr,
                enemy.on_ground
            )
        })
        .collect::<Vec<_>>()
        .join(";")
}

/// Capture native frames at checkpoint indices and compare against oracle PNGs.
/// Returns (worst MAE, worst pixel mismatch count) across checkpoints.
fn compare_checkpoint_frames(
    manifest: &AssetManifest,
    native_trace: &[NativeTraceFrame],
    scenario: &DynamicScenario,
    oracle: &OracleManifest,
    level_dir: &Path,
    level_id: u8,
    oracle_paused_checkpoints: &std::collections::HashSet<&str>,
) -> Result<(Option<f32>, Option<usize>)> {
    let mut worst_mae: Option<f32> = None;
    let mut worst_pixel_mismatches: Option<usize> = None;

    // Convert DynamicCheckpoints to GameplayCheckpoints for the renderer.
    let gameplay_checkpoints: Vec<scenarios::GameplayCheckpoint> = scenario
        .checkpoints
        .iter()
        .map(|cp| scenarios::GameplayCheckpoint {
            label: Box::leak(cp.label.clone().into_boxed_str()),
            frame_index: cp.frame_index,
        })
        .collect();

    // Use the existing headless renderer to produce real native frames.
    // Falls back to black frames if level visuals aren't available.
    let native_frames = render_gameplay_frame_evidence_for_level(
        manifest,
        native_trace,
        &gameplay_checkpoints,
        level_id,
    )
    .unwrap_or_else(|e| {
        eprintln!(
            "  warning: level {level_id:#04x} frame render failed ({e:#}), using black frames"
        );
        gameplay_checkpoints
            .iter()
            .map(|cp| {
                (
                    cp.label.to_string(),
                    cp.frame_index,
                    320,
                    224,
                    vec![0u8; 320 * 224 * 3],
                )
            })
            .collect()
    });

    for (native_evidence, oracle_frame) in native_frames.iter().zip(oracle.frames.iter()) {
        if oracle_paused_checkpoints.contains(oracle_frame.label.as_str()) {
            continue;
        }
        let (_label, _frame_idx, w, h, native_rgb) = native_evidence;
        let oracle_rgb = load_png_rgb(&oracle_frame.png_path)?;
        let mae = frame_mae(native_rgb, &oracle_rgb);
        let mismatched_pixels = pixel_mismatch_count(native_rgb, &oracle_rgb);

        worst_mae = Some(worst_mae.map_or(mae, |w: f32| w.max(mae)));
        worst_pixel_mismatches =
            Some(worst_pixel_mismatches.map_or(mismatched_pixels, |w| w.max(mismatched_pixels)));

        if mismatched_pixels > 0 {
            eprintln!(
                "  {}: {mismatched_pixels} mismatched pixels (MAE={mae:.1})",
                oracle_frame.label
            );
        }

        // Generate diff composite
        let (cw, ch, mut composite) =
            generate_diff_composite(native_rgb, &oracle_rgb, *w as u32, *h as u32);
        annotate_diff(&mut composite, cw, level_id, &oracle_frame.label, mae);

        let diff_path = level_dir.join(format!("diff_{}.png", oracle_frame.label));
        image::save_buffer_with_format(
            &diff_path,
            &composite,
            cw,
            ch,
            image::ColorType::Rgb8,
            image::ImageFormat::Png,
        )
        .with_context(|| format!("failed to write {}", diff_path.display()))?;

        // Also save native frame for direct comparison
        let native_path = level_dir.join(format!("native_{}.png", oracle_frame.label));
        image::save_buffer_with_format(
            &native_path,
            native_rgb,
            *w as u32,
            *h as u32,
            image::ColorType::Rgb8,
            image::ImageFormat::Png,
        )
        .with_context(|| format!("failed to write {}", native_path.display()))?;
    }

    Ok((worst_mae, worst_pixel_mismatches))
}

/// Deep diagnosis of a single level: run with Frames tier, print findings.
pub(crate) fn diagnose_level(
    repo_root: &Path,
    rom_path: &Path,
    manifest: &AssetManifest,
    level_id: u8,
) -> Result<()> {
    let report = run_sweep(
        repo_root,
        rom_path,
        manifest,
        &[level_id],
        SweepTier::Frames,
    )?;
    let result = report
        .levels
        .first()
        .ok_or_else(|| anyhow!("no result for level {level_id:#04x}"))?;

    println!("\nLevel {level_id:#04x} diagnosis:");
    if result.passed {
        println!("  PASSED — no divergence detected");
        return Ok(());
    }

    if let Some(ref err) = result.error {
        println!("  ERROR: {err}");
        return Ok(());
    }

    for diff in &result.state_diffs {
        println!(
            "  State divergence at {}: {} native={} oracle={}",
            diff.checkpoint, diff.field, diff.native_value, diff.oracle_value
        );
    }

    if let Some(mae) = result.mae {
        let pass_fail = if mae < 32.0 { "PASS" } else { "FAIL" };
        println!("  Frame MAE: {mae:.1} ({pass_fail}, threshold: 32.0)");
    }

    for path in &result.diff_images {
        println!("  Diff image: {path}");
    }

    // Suggest investigation areas based on which fields diverged
    let diverged_fields: Vec<&str> = result
        .state_diffs
        .iter()
        .map(|d| d.field.as_str())
        .collect();
    println!("\n  Suggested investigation:");
    if diverged_fields.contains(&"number_objects") {
        println!("    - Enemy spawning: check enemy layout for level {level_id:#04x}");
        println!("      → code/enemy/*.asm, kidcli inspect-enemy");
    }
    if diverged_fields.contains(&"camera_x_pos") || diverged_fields.contains(&"camera_y_pos") {
        println!("    - Camera position: check camera follow/scroll logic");
        println!("      → camera routines in kid.asm");
    }
    if diverged_fields.contains(&"kid_x_pos") || diverged_fields.contains(&"kid_y_pos") {
        println!("    - Player position: check physics/movement code");
        println!("      → kid-core physics.rs, terrain.rs");
    }
    if diverged_fields.contains(&"game_mode") {
        println!("    - Game mode: check state machine transitions");
        println!("      → NativeRuntime state machine in kid-core");
    }
    if diverged_fields.contains(&"current_level_id") {
        println!("    - Level ID: check level transition / warp logic");
    }

    Ok(())
}

/// Agent-driven parity loop: sweep → find worst → diagnose → report.
pub(crate) fn run_parity_loop(
    repo_root: &Path,
    rom_path: &Path,
    manifest: &AssetManifest,
    max_cycles: u32,
) -> Result<()> {
    let all_levels: Vec<u8> = (0..=104).collect();
    let source_hash = kid_core_source_hash(repo_root).unwrap_or_else(|_| "unknown".into());

    for cycle in 1..=max_cycles {
        println!("=== parity loop cycle {cycle}/{max_cycles} ===");
        println!("kid-core source hash: {source_hash}\n");

        // Tier 1: state-only sweep
        let report = run_sweep(repo_root, rom_path, manifest, &all_levels, SweepTier::State)?;

        if report.failed == 0 && report.errors == 0 {
            println!("\nAll {0} levels passed!", report.passed);
            return Ok(());
        }

        // Find worst failure (most state diffs)
        let worst = report
            .levels
            .iter()
            .filter(|r| !r.passed)
            .max_by_key(|r| r.state_diffs.len());

        if let Some(worst) = worst {
            println!(
                "\nWorst failure: level {:#04x} ({} state diffs)",
                worst.level_id,
                worst.state_diffs.len()
            );
            diagnose_level(repo_root, rom_path, manifest, worst.level_id)?;
        }

        println!(
            "\nCycle {cycle} summary: {} passed, {} failed, {} errors",
            report.passed, report.failed, report.errors
        );
    }

    Ok(())
}

/// Populate oracle cache for all requested levels in parallel, skip native comparison.
fn warm_oracle_cache(
    rom_bytes: &[u8],
    rom_path: &Path,
    repo_root: &Path,
    env: &OracleEnvironment,
    levels: &[u8],
) -> SweepReport {
    let timestamp = chrono_timestamp();
    let mut cached = 0usize;
    let mut ran = 0usize;
    let mut errored = 0usize;

    // Group levels into batches for parallel oracle runs.
    // Oracle processes are I/O-heavy (emulator), so parallelism helps.
    let batch_size = std::thread::available_parallelism()
        .map(std::num::NonZero::get)
        .unwrap_or(4)
        .min(8); // cap at 8 to avoid thrashing

    for chunk in levels.chunks(batch_size) {
        let handles: Vec<_> = chunk
            .iter()
            .map(|&level_id| {
                let scenario = idle_level_scenario(level_id, None);
                let cache_key = OracleCacheKey::from_scenario(rom_bytes, &scenario, "jgenesis");

                // Already cached?
                if load_cached_oracle(&cache_key, repo_root).is_some() {
                    return None;
                }

                let rom = rom_path.to_owned();
                let root = repo_root.to_owned();
                let env = env.clone();

                Some((level_id, scenario, cache_key, rom, root, env))
            })
            .collect();

        let mut threads = Vec::new();
        for item in handles {
            match item {
                None => cached += 1,
                Some((level_id, scenario, cache_key, rom, root, env)) => {
                    threads.push(thread::spawn(move || {
                        let tmp = root.join(format!(".cache/warm-cache-tmp/{level_id:02x}"));
                        fs::create_dir_all(&tmp).ok();

                        let scenario_file = tmp.join("scenario.json");
                        fs::write(
                            &scenario_file,
                            serde_json::to_vec_pretty(&scenario).unwrap_or_default(),
                        )
                        .ok();

                        let oracle_dir = tmp.join("oracle");
                        fs::create_dir_all(&oracle_dir).ok();

                        match run_jgenesis_capture_dynamic(
                            &env,
                            &rom,
                            &scenario_file,
                            &oracle_dir,
                            None,
                        ) {
                            Ok(oracle) => {
                                let _ = store_oracle_cache(&cache_key, &root, &oracle, &oracle_dir);
                                println!("  warm {level_id:#04x}: cached");
                                Ok(())
                            }
                            Err(e) => {
                                println!("  warm {level_id:#04x}: ERROR {e:#}");
                                Err(e)
                            }
                        }
                    }));
                }
            }
        }

        for handle in threads {
            match handle.join() {
                Ok(Ok(())) => ran += 1,
                _ => errored += 1,
            }
        }
    }

    println!(
        "\nwarm-cache complete: {cached} already cached, {ran} newly cached, {errored} errors"
    );

    SweepReport {
        timestamp,
        kid_core_hash: None,
        levels: Vec::new(),
        passed: cached + ran,
        failed: 0,
        errors: errored,
        divergence_summary: DivergenceSummary::default(),
        field_summary: Vec::new(),
        release_gate_focus: Vec::new(),
    }
}

/// Compute a SHA-1 hash of kid-core source files.
/// Used for incremental native: if the hash hasn't changed since the last sweep,
/// native re-runs can be skipped (the game logic is deterministic).
pub(crate) fn kid_core_source_hash(repo_root: &Path) -> Result<String> {
    let src_dir = repo_root.join("crates/kid-core/src");
    let mut paths: Vec<PathBuf> = Vec::new();
    collect_rs_files(&src_dir, &mut paths)?;
    paths.sort();

    let mut hasher = Sha1::new();
    for path in &paths {
        hasher.update(
            path.strip_prefix(repo_root)
                .unwrap_or(path)
                .to_string_lossy()
                .as_bytes(),
        );
        hasher.update(b"\0");
        hasher.update(&fs::read(path).with_context(|| format!("reading {}", path.display()))?);
        hasher.update(b"\0");
    }
    Ok(format!("{:x}", hasher.finalize()))
}

fn collect_rs_files(dir: &Path, out: &mut Vec<PathBuf>) -> Result<()> {
    if !dir.is_dir() {
        return Ok(());
    }
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            collect_rs_files(&path, out)?;
        } else if path.extension().is_some_and(|e| e == "rs") {
            out.push(path);
        }
    }
    Ok(())
}

fn chrono_timestamp() -> String {
    use std::time::SystemTime;
    let since_epoch = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap_or_default();
    format!("{}", since_epoch.as_secs())
}

// Frame capture/diagnosis/golden functions: see frames.rs
// Gameplay sweep scenarios: see gameplay.rs

fn append_sweep_history(repo_root: &Path, entry: &SweepHistoryEntry) -> Result<()> {
    let metrics_dir = repo_root.join(".cache/metrics");
    fs::create_dir_all(&metrics_dir)
        .with_context(|| format!("failed to create {}", metrics_dir.display()))?;
    let history_path = metrics_dir.join("sweep-history.jsonl");
    let mut line = serde_json::to_string(&entry)?;
    line.push('\n');
    let mut file = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&history_path)
        .with_context(|| format!("failed to open {}", history_path.display()))?;
    use std::io::Write;
    file.write_all(line.as_bytes())
        .with_context(|| format!("failed to append {}", history_path.display()))?;
    Ok(())
}

/// Run the sweep with both physics orderings and compare results.
pub(crate) fn run_sweep_compare(
    repo_root: &Path,
    rom_path: &Path,
    manifest: &AssetManifest,
    levels: &[u8],
    _tier: SweepTier,
) -> Result<()> {
    use kid_core::PhysicsOrder;

    let rom_bytes = fs::read(rom_path)
        .with_context(|| format!("failed to read ROM from {}", rom_path.display()))?;

    let env = ensure_oracles(repo_root)?;
    let backend: Box<dyn OracleBackend> = Box::new(JgenesisBackend { env: env.clone() });

    println!(
        "{:<8} {:<30} {:<16} {:<16} {:<16} {:<10}",
        "level", "name", "gravity_first", "pos_first", "asm_faithful", "verdict"
    );
    println!("{}", "-".repeat(96));

    for &level_id in levels {
        let scenario = idle_level_scenario(level_id, None);
        let level_dir = repo_root
            .join(".cache/sweep-compare")
            .join(format!("{level_id:02x}"));
        fs::create_dir_all(&level_dir)?;

        // Run oracle once (shared between both orderings)
        let cache_key = OracleCacheKey::from_scenario(&rom_bytes, &scenario, backend.name());
        let oracle = if let Some(cached) = load_cached_oracle(&cache_key, repo_root) {
            cached
        } else {
            let scenario_file = level_dir.join("scenario.json");
            fs::write(&scenario_file, serde_json::to_vec_pretty(&scenario)?)?;
            let oracle_dir = level_dir.join("oracle");
            fs::create_dir_all(&oracle_dir)?;
            let oracle = backend.run_scenario(rom_path, &scenario_file, &oracle_dir, None)?;
            let _ = store_oracle_cache(&cache_key, repo_root, &oracle, &oracle_dir);
            oracle
        };

        // Run native with GravityFirst (current)
        let current_trace = kid_core::run_script_with_physics_order(
            manifest.clone(),
            &scenario.script,
            scenario.frame_count,
            scenario.warp_to_level,
            PhysicsOrder::GravityFirst,
        );
        let current_states = extract_checkpoint_states(&current_trace, &scenario.checkpoints);
        let current_diffs_vec = compare_sweep_states(&current_states, &oracle.states);
        let current_auth = summarise_state_diffs(&current_diffs_vec).authoritative;

        // Run native with PositionFirst
        let pos_first_trace = kid_core::run_script_with_physics_order(
            manifest.clone(),
            &scenario.script,
            scenario.frame_count,
            scenario.warp_to_level,
            PhysicsOrder::PositionFirst,
        );
        let pos_first_states = extract_checkpoint_states(&pos_first_trace, &scenario.checkpoints);
        let pos_first_diffs_vec = compare_sweep_states(&pos_first_states, &oracle.states);
        let pos_first_auth = summarise_state_diffs(&pos_first_diffs_vec).authoritative;

        // Run native with AsmFaithful
        let asm_trace = kid_core::run_script_with_physics_order(
            manifest.clone(),
            &scenario.script,
            scenario.frame_count,
            scenario.warp_to_level,
            PhysicsOrder::AsmFaithful,
        );
        let asm_states = extract_checkpoint_states(&asm_trace, &scenario.checkpoints);
        let asm_diffs_vec = compare_sweep_states(&asm_states, &oracle.states);
        let asm_auth = summarise_state_diffs(&asm_diffs_vec).authoritative;

        let verdict = if asm_auth < current_auth && asm_auth <= pos_first_auth {
            "ASM_BEST"
        } else if pos_first_auth < current_auth && pos_first_auth < asm_auth {
            "POS_BEST"
        } else if current_auth <= pos_first_auth && current_auth <= asm_auth {
            if current_auth == 0 {
                "PERFECT"
            } else {
                "GRAV_BEST"
            }
        } else {
            "same"
        };

        let slug = format!("idle_level_{level_id:02x}");
        println!(
            "0x{level_id:02X}    {slug:<30} {current_auth:<16} {pos_first_auth:<16} {asm_auth:<16} {verdict}"
        );
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_level_range_single() {
        assert_eq!(parse_level_range("5").unwrap(), vec![5]);
    }

    #[test]
    fn parse_level_range_range() {
        assert_eq!(parse_level_range("0-4").unwrap(), vec![0, 1, 2, 3, 4]);
    }

    #[test]
    fn parse_level_range_csv() {
        assert_eq!(parse_level_range("0,5,9").unwrap(), vec![0, 5, 9]);
    }

    #[test]
    fn parse_level_range_mixed() {
        // 0x10-0x12 are valid BCD; 0x0A-0x0F are filtered out from the range
        assert_eq!(
            parse_level_range("0-2,5,16-18").unwrap(),
            vec![0, 1, 2, 5, 16, 17, 18]
        );
    }

    #[test]
    fn parse_level_range_filters_invalid_bcd() {
        // Decimal 10-15 = 0x0A-0x0F, all invalid BCD
        assert_eq!(parse_level_range("8-18").unwrap(), vec![8, 9, 16, 17, 18]);
    }

    #[test]
    fn parse_level_range_deduplicates() {
        assert_eq!(parse_level_range("1,1,2,2").unwrap(), vec![1, 2]);
    }

    #[test]
    fn parse_level_range_reversed_errors() {
        assert!(parse_level_range("5-3").is_err());
    }

    #[test]
    fn compare_sweep_states_detects_game_mode_diff() {
        let native = vec![OracleState {
            label: "settle".into(),
            frame_index: 780,
            game_mode: 0x0010,
            boot_phase: None,
            boot_phase_elapsed: None,
            title_screen_ready: false,
            main_menu_option: None,
            current_player: 0,
            number_hitpoints: 3,
            current_helmet_id: 0,
            current_level_id: 0,
            player_1_level_id: 0,
            camera_x_pos: 0,
            camera_y_pos: 0,
            kid_x_pos: 0,
            kid_y_pos: 0,
            number_objects: 0,
            number_gfx_objects: 0,
            game_paused: false,
            active_enemies: Vec::new(),
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
            invulnerable_frames: None,
            currently_transforming: None,
            rng_offset: None,
            level_special_effects: None,
            gameplay_debug_fields: std::collections::BTreeMap::new(),
            cpu_registers: None,
        }];
        let mut oracle = native.clone();
        oracle[0].game_mode = 0x0020;

        let diffs = compare_sweep_states(&native, &oracle);
        assert_eq!(diffs.len(), 1);
        assert_eq!(diffs[0].field, "game_mode");
    }

    #[test]
    fn compare_sweep_states_exact_match_produces_no_diffs() {
        let native = vec![OracleState {
            label: "settle".into(),
            frame_index: 780,
            game_mode: 0x0010,
            boot_phase: None,
            boot_phase_elapsed: None,
            title_screen_ready: false,
            main_menu_option: None,
            current_player: 0,
            number_hitpoints: 3,
            current_helmet_id: 0,
            current_level_id: 0,
            player_1_level_id: 0,
            camera_x_pos: 0x0100_0000,
            camera_y_pos: 0,
            kid_x_pos: 0,
            kid_y_pos: 0,
            number_objects: 5,
            number_gfx_objects: 3,
            game_paused: false,
            active_enemies: Vec::new(),
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
            invulnerable_frames: None,
            currently_transforming: None,
            rng_offset: None,
            level_special_effects: None,
            gameplay_debug_fields: std::collections::BTreeMap::new(),
            cpu_registers: None,
        }];
        let oracle = native.clone();

        let diffs = compare_sweep_states(&native, &oracle);
        assert!(diffs.is_empty());
    }

    #[test]
    fn enemy_frontier_detects_enemy_roster_drift_before_gameplay_drift() {
        let enemy = OracleEnemyState {
            object_type: 0x0D,
            x_pos: 0x035E_8000,
            y_pos: 0x0270_0000,
            x_velocity: 0,
            y_velocity: 0,
            facing: EnemyFacing::Left,
            collision_type: None,
            collision_source: None,
            layout_index: Some(4),
            layout_ptr: Some(0x00FF_A100),
            parent_object_ptr: Some(0x00FF_9120),
            gfx_slot_index: Some(3),
            on_ground: Some(true),
            debug_fields: std::collections::BTreeMap::new(),
        };
        let native = vec![OracleState {
            label: "settle".into(),
            frame_index: 780,
            game_mode: 0x0010,
            boot_phase: None,
            boot_phase_elapsed: None,
            title_screen_ready: false,
            main_menu_option: None,
            current_player: 0,
            number_hitpoints: 3,
            current_helmet_id: 0,
            current_level_id: 0,
            player_1_level_id: 0,
            camera_x_pos: 0x0100_0000,
            camera_y_pos: 0,
            kid_x_pos: 0,
            kid_y_pos: 0,
            number_objects: 5,
            number_gfx_objects: 3,
            game_paused: false,
            active_enemies: vec![enemy.clone()],
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
            invulnerable_frames: None,
            currently_transforming: None,
            rng_offset: None,
            level_special_effects: None,
            gameplay_debug_fields: std::collections::BTreeMap::new(),
            cpu_registers: None,
        }];
        let oracle = vec![OracleState {
            active_enemies: Vec::new(),
            ..native[0].clone()
        }];

        let gameplay_diffs =
            compare_sweep_states_for_frontier(&native, &oracle, FrontierKind::Gameplay);
        assert!(gameplay_diffs.is_empty());

        let enemy_diffs = compare_sweep_states_for_frontier(&native, &oracle, FrontierKind::Enemy);
        assert_eq!(enemy_diffs.len(), 1);
        assert_eq!(enemy_diffs[0].field, "enemy_roster");
    }

    #[test]
    fn sweep_options_default_is_no_limits() {
        let opts = SweepOptions::default();
        assert!(!opts.warm_cache_only);
        assert_eq!(opts.max_failures, 0);
    }

    #[test]
    fn kid_core_source_hash_is_deterministic() {
        // Both calls with the same source should produce the same hash.
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .parent()
            .unwrap()
            .to_owned();
        let h1 = kid_core_source_hash(&root).unwrap();
        let h2 = kid_core_source_hash(&root).unwrap();
        assert_eq!(h1, h2);
        assert_eq!(h1.len(), 40); // SHA-1 hex
    }

    #[test]
    fn walk_right_assertions_pass_when_x_increases() {
        use kid_core::{GameMode, LevelId, NativeTraceFrame};
        let scenario = walk_right_scenario(0x00, 300);
        let mut trace: Vec<NativeTraceFrame> = (0..=scenario.frame_count)
            .map(|i| NativeTraceFrame {
                frame_index: i,
                game_mode: GameMode::InGame,
                game_mode_value: GameMode::InGame.as_disassembly_value(),
                start_press_count: 0,
                boot_phase: None,
                boot_phase_elapsed: None,
                title_screen_ready: false,
                current_player: 0,
                current_level: LevelId(0),
                player_1_level_id: LevelId(0),
                current_level_name: None,
                player_start_x: None,
                player_start_y: None,
                flag_x: None,
                flag_y: None,
                level_enemy_object_count: None,
                active_enemy_object_count: 0,
                active_platform_object_count: 0,
                number_lives: 3,
                number_continues: 3,
                number_hitpoints: 2,
                current_helmet_id: 0,
                paused: false,
                // X increases over time
                kid_x_pos: Some(0x0020_0000 + (i as u32) * 0x0001_0000),
                kid_y_pos: Some(0x00E0_0000),
                kid_x_velocity: Some(0),
                kid_y_velocity: Some(0),
                character_movement: None,
                kid_facing: None,
                kid_on_ground: Some(true),
                kid_on_slope: 0,
                active_objects: Vec::new(),
                camera_x_pos: 0,
                camera_y_pos: 0,
                number_objects: 0,
                number_gfx_objects: 0,
                rng_offset: None,
                level_complete: false,
                game_over: false,
                continue_countdown: None,
                game_complete: false,
                ending_elapsed: None,
                options_cursor: None,
                two_player: false,
                options_two_player_two_controllers: false,
                options_controls_scheme: 0,
                options_speed_button_fast: false,
                invulnerable_frames: 0,
                just_received_damage: false,
                diamond_power_active: false,
                diamond_count: 0,
                score: 0,
                level_timer_seconds: 180,
                level_damage_taken: false,
                murder_wall_x: None,
                active_projectiles: Vec::new(),
                active_projectile_details: Vec::new(),
                active_geysers: Vec::new(),
                block_bounces: Vec::new(),
                block_destroy_effects: Vec::new(),
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
            })
            .collect();
        let diffs = check_walk_right_assertions(&trace, &scenario);
        assert!(diffs.is_empty(), "expected no diffs but got: {diffs:?}");

        // Now make X decrease at walk_end checkpoint — should fail
        let walk_end_idx = scenario.frame_count.saturating_sub(1);
        trace[walk_end_idx].kid_x_pos = Some(0x0010_0000);
        let diffs = check_walk_right_assertions(&trace, &scenario);
        assert_eq!(diffs.len(), 1);
        assert_eq!(diffs[0].field, "kid_x_pos_monotonic");
    }

    #[test]
    fn jump_assertions_pass_when_vy_goes_negative() {
        use kid_core::{GameMode, LevelId, NativeTraceFrame};
        let scenario = prize_block_scenario(0x00);
        let mut trace: Vec<NativeTraceFrame> = (0..=scenario.frame_count)
            .map(|i| NativeTraceFrame {
                frame_index: i,
                game_mode: GameMode::InGame,
                game_mode_value: GameMode::InGame.as_disassembly_value(),
                start_press_count: 0,
                boot_phase: None,
                boot_phase_elapsed: None,
                title_screen_ready: false,
                current_player: 0,
                current_level: LevelId(0),
                player_1_level_id: LevelId(0),
                current_level_name: None,
                player_start_x: None,
                player_start_y: None,
                flag_x: None,
                flag_y: None,
                level_enemy_object_count: None,
                active_enemy_object_count: 0,
                active_platform_object_count: 0,
                number_lives: 3,
                number_continues: 3,
                number_hitpoints: 2,
                current_helmet_id: 0,
                paused: false,
                kid_x_pos: Some(0x0020_0000),
                kid_y_pos: Some(0x00E0_0000),
                kid_x_velocity: Some(0),
                kid_y_velocity: Some(0),
                character_movement: None,
                kid_facing: None,
                kid_on_ground: Some(true),
                kid_on_slope: 0,
                active_objects: Vec::new(),
                camera_x_pos: 0,
                camera_y_pos: 0,
                number_objects: 0,
                number_gfx_objects: 0,
                rng_offset: None,
                level_complete: false,
                game_over: false,
                continue_countdown: None,
                game_complete: false,
                ending_elapsed: None,
                options_cursor: None,
                two_player: false,
                options_two_player_two_controllers: false,
                options_controls_scheme: 0,
                options_speed_button_fast: false,
                invulnerable_frames: 0,
                just_received_damage: false,
                diamond_power_active: false,
                diamond_count: 0,
                score: 0,
                level_timer_seconds: 180,
                level_damage_taken: false,
                murder_wall_x: None,
                active_projectiles: Vec::new(),
                active_projectile_details: Vec::new(),
                active_geysers: Vec::new(),
                block_bounces: Vec::new(),
                block_destroy_effects: Vec::new(),
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
            })
            .collect();

        // Find when the first B press happens
        let first_b = scenario
            .script
            .iter()
            .enumerate()
            .skip(60)
            .find(|&(_, &b)| b & (1 << 4) != 0)
            .map(|(i, _)| i)
            .unwrap();

        // Set negative velocity and airborne a few frames after jump
        for offset in 2..=5 {
            if let Some(f) = trace.get_mut(first_b + offset) {
                f.kid_y_velocity = Some(-0x0004_0000);
                f.kid_on_ground = Some(false);
            }
        }

        let diffs = check_jump_assertions(&trace, &scenario);
        assert!(diffs.is_empty(), "expected no diffs but got: {diffs:?}");
    }

    #[test]
    fn jump_assertions_fail_when_player_stays_grounded() {
        use kid_core::{GameMode, LevelId, NativeTraceFrame};
        let scenario = prize_block_scenario(0x00);
        // All frames: grounded, vy=0
        let trace: Vec<NativeTraceFrame> = (0..=scenario.frame_count)
            .map(|i| NativeTraceFrame {
                frame_index: i,
                game_mode: GameMode::InGame,
                game_mode_value: GameMode::InGame.as_disassembly_value(),
                start_press_count: 0,
                boot_phase: None,
                boot_phase_elapsed: None,
                title_screen_ready: false,
                current_player: 0,
                current_level: LevelId(0),
                player_1_level_id: LevelId(0),
                current_level_name: None,
                player_start_x: None,
                player_start_y: None,
                flag_x: None,
                flag_y: None,
                level_enemy_object_count: None,
                active_enemy_object_count: 0,
                active_platform_object_count: 0,
                number_lives: 3,
                number_continues: 3,
                number_hitpoints: 2,
                current_helmet_id: 0,
                paused: false,
                kid_x_pos: Some(0x0020_0000),
                kid_y_pos: Some(0x00E0_0000),
                kid_x_velocity: Some(0),
                kid_y_velocity: Some(0),
                character_movement: None,
                kid_facing: None,
                kid_on_ground: Some(true),
                kid_on_slope: 0,
                active_objects: Vec::new(),
                camera_x_pos: 0,
                camera_y_pos: 0,
                number_objects: 0,
                number_gfx_objects: 0,
                rng_offset: None,
                level_complete: false,
                game_over: false,
                continue_countdown: None,
                game_complete: false,
                ending_elapsed: None,
                options_cursor: None,
                two_player: false,
                options_two_player_two_controllers: false,
                options_controls_scheme: 0,
                options_speed_button_fast: false,
                invulnerable_frames: 0,
                just_received_damage: false,
                diamond_power_active: false,
                diamond_count: 0,
                score: 0,
                level_timer_seconds: 180,
                level_damage_taken: false,
                murder_wall_x: None,
                active_projectiles: Vec::new(),
                active_projectile_details: Vec::new(),
                active_geysers: Vec::new(),
                block_bounces: Vec::new(),
                block_destroy_effects: Vec::new(),
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
            })
            .collect();

        let diffs = check_jump_assertions(&trace, &scenario);
        // When player never gets negative vy, we skip the assertion entirely —
        // the generic scenario doesn't know level geometry, so a blocked jump
        // (e.g. low ceiling) is expected on some levels.
        assert_eq!(diffs.len(), 0, "blocked jump should be skipped, not failed");
    }

    #[test]
    fn compare_sweep_states_detects_large_camera_drift() {
        let native = vec![OracleState {
            label: "settle".into(),
            frame_index: 780,
            game_mode: 0x0010,
            boot_phase: None,
            boot_phase_elapsed: None,
            title_screen_ready: false,
            main_menu_option: None,
            current_player: 0,
            number_hitpoints: 3,
            current_helmet_id: 0,
            current_level_id: 0,
            player_1_level_id: 0,
            camera_x_pos: 0x0100_0000,
            camera_y_pos: 0,
            kid_x_pos: 0,
            kid_y_pos: 0,
            number_objects: 5,
            number_gfx_objects: 3,
            game_paused: false,
            active_enemies: Vec::new(),
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
            invulnerable_frames: None,
            currently_transforming: None,
            rng_offset: None,
            level_special_effects: None,
            gameplay_debug_fields: std::collections::BTreeMap::new(),
            cpu_registers: None,
        }];
        let mut oracle = native.clone();
        oracle[0].camera_x_pos = 0x0110_0000;

        let diffs = compare_sweep_states(&native, &oracle);
        assert_eq!(diffs.len(), 1);
        assert_eq!(diffs[0].field, "camera_x_pos");
    }

    #[test]
    fn paused_oracle_position_diff_is_an_oracle_limitation() {
        let diff = StateDiff {
            checkpoint: "idle_tail".into(),
            field: "camera_y_pos".into(),
            native_value: "0x00830000".into(),
            oracle_value: "0x00b30000".into(),
            category: DivergenceCategory::Authoritative,
        };
        let death_gap_checkpoints = std::collections::HashSet::<&str>::new();
        let oracle_paused_checkpoints = std::collections::HashSet::from(["idle_tail"]);

        let category =
            classify_state_diff(&diff, &death_gap_checkpoints, &oracle_paused_checkpoints);

        assert_eq!(category, DivergenceCategory::OracleLimitation);
    }

    #[test]
    fn paused_oracle_non_position_diff_is_also_oracle_limitation() {
        // When the oracle is paused, ALL fields should be oracle_limitation,
        // not just positions — the oracle's entire state is unreliable.
        for field in ["game_paused", "current_level_id", "just_received_damage"] {
            let diff = StateDiff {
                checkpoint: "level_settle".into(),
                field: field.into(),
                native_value: "true".into(),
                oracle_value: "false".into(),
                category: DivergenceCategory::Authoritative,
            };
            let death_gap_checkpoints = std::collections::HashSet::<&str>::new();
            let oracle_paused_checkpoints = std::collections::HashSet::from(["level_settle"]);

            let category =
                classify_state_diff(&diff, &death_gap_checkpoints, &oracle_paused_checkpoints);

            assert_eq!(
                category,
                DivergenceCategory::OracleLimitation,
                "field '{field}' should be oracle limitation when oracle is paused"
            );
        }
    }

    #[test]
    fn oracle_limitation_only_allows_advisory_diffs() {
        let result = LevelSweepResult {
            level_id: 0x45,
            slug: "idle_level_45".into(),
            tier: SweepTier::Frames,
            passed: false,
            state_diffs: Vec::new(),
            mae: None,
            pixel_mismatches: None,
            diff_images: Vec::new(),
            error: None,
            divergence_summary: DivergenceSummary {
                authoritative: 0,
                advisory: 3,
                accumulation: 0,
                rng: 0,
                oracle_limitation: 4,
            },
        };

        assert!(is_oracle_limitation_only(&result));
    }

    #[test]
    fn oracle_limitation_summary_is_non_blocking() {
        let summary = DivergenceSummary {
            authoritative: 0,
            advisory: 3,
            accumulation: 0,
            rng: 0,
            oracle_limitation: 4,
        };

        assert!(!has_blocking_divergences(&summary));
    }

    #[test]
    fn authoritative_summary_is_blocking() {
        let summary = DivergenceSummary {
            authoritative: 1,
            advisory: 0,
            accumulation: 0,
            rng: 0,
            oracle_limitation: 12,
        };

        assert!(has_blocking_divergences(&summary));
    }

    #[test]
    fn mixed_advisory_and_oracle_limitations_are_reported_as_non_blocking() {
        let result = LevelSweepResult {
            level_id: 0x45,
            slug: "idle_level_45".into(),
            tier: SweepTier::State,
            passed: true,
            state_diffs: Vec::new(),
            mae: None,
            pixel_mismatches: None,
            diff_images: Vec::new(),
            error: None,
            divergence_summary: DivergenceSummary {
                authoritative: 0,
                advisory: 3,
                accumulation: 0,
                rng: 0,
                oracle_limitation: 4,
            },
        };

        assert_eq!(level_result_status(&result), "NON-BLOCKING");
    }

    #[test]
    fn divergence_summary_lists_all_non_blocking_categories() {
        let summary = DivergenceSummary {
            authoritative: 0,
            advisory: 3,
            accumulation: 0,
            rng: 0,
            oracle_limitation: 4,
        };

        assert_eq!(
            format_divergence_summary(&summary),
            " (3 advisory diffs, 4 oracle limitations)"
        );
    }

    #[test]
    fn append_sweep_history_persists_per_level_categories() {
        let temp_dir = tempfile::tempdir().expect("tempdir");
        let entry = SweepHistoryEntry {
            timestamp: "1775118265".into(),
            tier: SweepTier::State,
            levels_requested: 2,
            passed: 2,
            failed: 0,
            errors: 0,
            kid_core_hash: Some("deadbeef".into()),
            divergence_summary: DivergenceSummary {
                authoritative: 0,
                advisory: 3,
                accumulation: 0,
                rng: 0,
                oracle_limitation: 5,
            },
            report_path: "/tmp/report.json".into(),
            field_summary: vec![SweepHistoryFieldEntry {
                field: "game_mode".into(),
                total_diffs: 3,
                levels: 2,
                level_ids: vec!["0x00".into(), "0x01".into()],
                checkpoints: vec!["idle_tail".into()],
                checkpoint_counts: vec![SweepHistoryCheckpointCount {
                    checkpoint: "idle_tail".into(),
                    diffs: 3,
                }],
                divergence_summary: DivergenceSummary {
                    authoritative: 0,
                    advisory: 3,
                    accumulation: 0,
                    rng: 0,
                    oracle_limitation: 0,
                },
            }],
            release_gate_focus: vec![ReleaseGateFocusSummary {
                target: "1p-accurate".into(),
                step_name: "frame_oracle:boot_title_start_level1".into(),
                checkpoint: Some("title@360".into()),
                field: "game_mode".into(),
                native_value: Some("0x000c".into()),
                oracle_value: Some("0x002c".into()),
                scenario_level_id: Some("0x00".into()),
                matched_in_state_sweep: true,
                scenario_level_matched_in_state_sweep: Some(true),
                checkpoint_matched_in_state_sweep: Some(false),
                checkpoint_family_matched_in_state_sweep: Some(false),
                exact_value_pair_matched_in_state_sweep: Some(false),
                exact_value_pair_levels: Some(0),
                exact_value_pair_level_ids: Vec::new(),
                exact_value_pair_scenario_level_matched_in_state_sweep: Some(false),
                exact_value_pair_checkpoint_matched_in_state_sweep: Some(false),
                exact_value_pair_checkpoint_family_matched_in_state_sweep: Some(false),
                total_diffs: Some(3),
                levels: Some(2),
                level_ids: vec!["0x00".into(), "0x01".into()],
                matched_checkpoints: vec![SweepHistoryCheckpointCount {
                    checkpoint: "idle_tail".into(),
                    diffs: 3,
                }],
                exact_value_pair_checkpoints: Vec::new(),
                divergence_summary: DivergenceSummary {
                    authoritative: 0,
                    advisory: 3,
                    accumulation: 0,
                    rng: 0,
                    oracle_limitation: 0,
                },
                summary: "1p-accurate failing field game_mode=3 diffs across 2 levels for boot_title_start_level1 @ title@360 [advisory=3] [0x00, 0x01] [includes scenario level 0x00] [excludes checkpoint title@360] [excludes checkpoint family title] checkpoints=[idle_tail=3] (field-level coverage only; exact pair evidence lives in release_gate_focus, and the release-gate scenario still needs its own proof) [exact pair 0x000c->0x002c absent from latest state sweep; same field class, different concrete miss]".into(),
            }],
            per_level: vec![SweepHistoryLevelEntry {
                level: "0x00".into(),
                passed: true,
                blocking: false,
                authoritative: 0,
                advisory: 1,
                accumulation: 0,
                rng: 0,
                oracle_limitation: 2,
                fields: vec!["game_mode".into(), "number_objects".into()],
            }],
        };

        append_sweep_history(temp_dir.path(), &entry).expect("append history");

        let history_path = temp_dir.path().join(".cache/metrics/sweep-history.jsonl");
        let line = std::fs::read_to_string(&history_path).expect("history jsonl");
        let value: serde_json::Value =
            serde_json::from_str(line.lines().next().expect("history line")).expect("json line");

        assert_eq!(value["per_level"][0]["level"], "0x00");
        assert_eq!(value["per_level"][0]["blocking"], false);
        assert_eq!(value["field_summary"][0]["field"], "game_mode");
        assert_eq!(value["field_summary"][0]["total_diffs"], 3);
        assert_eq!(value["field_summary"][0]["levels"], 2);
        assert_eq!(value["field_summary"][0]["level_ids"][0], "0x00");
        assert_eq!(value["field_summary"][0]["level_ids"][1], "0x01");
        assert_eq!(value["field_summary"][0]["checkpoints"][0], "idle_tail");
        assert_eq!(
            value["field_summary"][0]["checkpoint_counts"][0]["checkpoint"],
            "idle_tail"
        );
        assert_eq!(
            value["field_summary"][0]["checkpoint_counts"][0]["diffs"],
            3
        );
        assert_eq!(value["release_gate_focus"][0]["field"], "game_mode");
        assert_eq!(value["release_gate_focus"][0]["checkpoint"], "title@360");
        assert_eq!(value["release_gate_focus"][0]["native_value"], "0x000c");
        assert_eq!(value["release_gate_focus"][0]["oracle_value"], "0x002c");
        assert_eq!(
            value["release_gate_focus"][0]["matched_checkpoints"][0]["checkpoint"],
            "idle_tail"
        );
        assert_eq!(
            value["release_gate_focus"][0]["checkpoint_matched_in_state_sweep"],
            false
        );
        assert_eq!(
            value["release_gate_focus"][0]["checkpoint_family_matched_in_state_sweep"],
            false
        );
        assert_eq!(
            value["release_gate_focus"][0]["exact_value_pair_matched_in_state_sweep"],
            false
        );
        assert_eq!(value["release_gate_focus"][0]["exact_value_pair_levels"], 0);
        assert!(value["release_gate_focus"][0]["exact_value_pair_level_ids"].is_null());
        assert_eq!(
            value["release_gate_focus"][0]["exact_value_pair_scenario_level_matched_in_state_sweep"],
            false
        );
        assert_eq!(
            value["release_gate_focus"][0]["exact_value_pair_checkpoint_matched_in_state_sweep"],
            false
        );
        assert_eq!(
            value["release_gate_focus"][0]["exact_value_pair_checkpoint_family_matched_in_state_sweep"],
            false
        );
        assert_eq!(value["per_level"][0]["fields"][0], "game_mode");
        assert_eq!(value["per_level"][0]["fields"][1], "number_objects");
    }

    #[test]
    fn sweep_report_serialises_release_gate_focus() {
        let report = SweepReport {
            timestamp: "1775172102".into(),
            kid_core_hash: Some("deadbeef".into()),
            levels: Vec::new(),
            passed: 69,
            failed: 0,
            errors: 0,
            divergence_summary: DivergenceSummary {
                authoritative: 0,
                advisory: 320,
                accumulation: 0,
                rng: 0,
                oracle_limitation: 648,
            },
            field_summary: vec![SweepHistoryFieldEntry {
                field: "game_mode".into(),
                total_diffs: 15,
                levels: 15,
                level_ids: vec!["0x00".into(), "0x09".into()],
                checkpoints: vec!["idle_tail".into(), "title@360".into()],
                checkpoint_counts: vec![
                    SweepHistoryCheckpointCount {
                        checkpoint: "idle_tail".into(),
                        diffs: 10,
                    },
                    SweepHistoryCheckpointCount {
                        checkpoint: "title@360".into(),
                        diffs: 5,
                    },
                ],
                divergence_summary: DivergenceSummary {
                    authoritative: 0,
                    advisory: 0,
                    accumulation: 0,
                    rng: 0,
                    oracle_limitation: 15,
                },
            }],
            release_gate_focus: vec![ReleaseGateFocusSummary {
                target: "1p-accurate".into(),
                step_name: "frame_oracle:boot_title_start_level1".into(),
                checkpoint: Some("title@360".into()),
                field: "game_mode".into(),
                native_value: Some("0x000c".into()),
                oracle_value: Some("0x002c".into()),
                scenario_level_id: Some("0x00".into()),
                matched_in_state_sweep: true,
                scenario_level_matched_in_state_sweep: Some(true),
                checkpoint_matched_in_state_sweep: Some(true),
                checkpoint_family_matched_in_state_sweep: Some(true),
                exact_value_pair_matched_in_state_sweep: Some(true),
                exact_value_pair_levels: Some(1),
                exact_value_pair_level_ids: vec!["0x00".into()],
                exact_value_pair_scenario_level_matched_in_state_sweep: Some(true),
                exact_value_pair_checkpoint_matched_in_state_sweep: Some(true),
                exact_value_pair_checkpoint_family_matched_in_state_sweep: Some(true),
                total_diffs: Some(15),
                levels: Some(15),
                level_ids: vec!["0x00".into(), "0x09".into()],
                matched_checkpoints: vec![
                    SweepHistoryCheckpointCount {
                        checkpoint: "idle_tail".into(),
                        diffs: 10,
                    },
                    SweepHistoryCheckpointCount {
                        checkpoint: "title@360".into(),
                        diffs: 5,
                    },
                ],
                exact_value_pair_checkpoints: vec![SweepHistoryCheckpointCount {
                    checkpoint: "title@360".into(),
                    diffs: 5,
                }],
                divergence_summary: DivergenceSummary {
                    authoritative: 0,
                    advisory: 0,
                    accumulation: 0,
                    rng: 0,
                    oracle_limitation: 15,
                },
                summary: "1p-accurate failing field game_mode=15 diffs across 15 levels for boot_title_start_level1 @ title@360 [oracle_limitations=15] [0x00, 0x09] [includes scenario level 0x00] [includes checkpoint title@360] [includes checkpoint family title] checkpoints=[idle_tail=10, title@360=5] (field-level coverage only; exact pair evidence lives in release_gate_focus, and the release-gate scenario still needs its own proof) [exact pair 0x000c->0x002c seen on levels [0x00] checkpoints=[title@360=5]; includes scenario level 0x00; includes checkpoint title@360; includes checkpoint family title]".into(),
            }],
        };

        let value = serde_json::to_value(&report).expect("serialise report");

        assert_eq!(value["release_gate_focus"][0]["field"], "game_mode");
        assert_eq!(value["release_gate_focus"][0]["checkpoint"], "title@360");
        assert_eq!(value["release_gate_focus"][0]["native_value"], "0x000c");
        assert_eq!(value["release_gate_focus"][0]["oracle_value"], "0x002c");
        assert_eq!(value["release_gate_focus"][0]["scenario_level_id"], "0x00");
        assert_eq!(
            value["release_gate_focus"][0]["scenario_level_matched_in_state_sweep"],
            true
        );
        assert_eq!(
            value["release_gate_focus"][0]["checkpoint_matched_in_state_sweep"],
            true
        );
        assert_eq!(
            value["release_gate_focus"][0]["checkpoint_family_matched_in_state_sweep"],
            true
        );
        assert_eq!(
            value["release_gate_focus"][0]["exact_value_pair_matched_in_state_sweep"],
            true
        );
        assert_eq!(value["release_gate_focus"][0]["exact_value_pair_levels"], 1);
        assert_eq!(
            value["release_gate_focus"][0]["exact_value_pair_level_ids"][0],
            "0x00"
        );
        assert_eq!(
            value["release_gate_focus"][0]["exact_value_pair_scenario_level_matched_in_state_sweep"],
            true
        );
        assert_eq!(
            value["release_gate_focus"][0]["exact_value_pair_checkpoint_matched_in_state_sweep"],
            true
        );
        assert_eq!(
            value["release_gate_focus"][0]["exact_value_pair_checkpoint_family_matched_in_state_sweep"],
            true
        );
        assert_eq!(
            value["release_gate_focus"][0]["divergence_summary"]["oracle_limitation"],
            15
        );
    }

    #[test]
    fn summarise_field_diffs_orders_by_frequency_then_coverage() {
        let results = vec![
            LevelSweepResult {
                level_id: 0x00,
                slug: "idle_level_00".into(),
                tier: SweepTier::State,
                passed: true,
                state_diffs: vec![
                    StateDiff {
                        checkpoint: "level_settle".into(),
                        field: "camera_y_pos".into(),
                        native_value: "1".into(),
                        oracle_value: "2".into(),
                        category: DivergenceCategory::OracleLimitation,
                    },
                    StateDiff {
                        checkpoint: "idle_tail".into(),
                        field: "camera_y_pos".into(),
                        native_value: "3".into(),
                        oracle_value: "4".into(),
                        category: DivergenceCategory::OracleLimitation,
                    },
                    StateDiff {
                        checkpoint: "idle_tail".into(),
                        field: "game_mode".into(),
                        native_value: "0008".into(),
                        oracle_value: "0010".into(),
                        category: DivergenceCategory::Advisory,
                    },
                ],
                mae: None,
                pixel_mismatches: None,
                diff_images: Vec::new(),
                error: None,
                divergence_summary: DivergenceSummary::default(),
            },
            LevelSweepResult {
                level_id: 0x01,
                slug: "idle_level_01".into(),
                tier: SweepTier::State,
                passed: true,
                state_diffs: vec![
                    StateDiff {
                        checkpoint: "level_settle".into(),
                        field: "camera_y_pos".into(),
                        native_value: "5".into(),
                        oracle_value: "6".into(),
                        category: DivergenceCategory::OracleLimitation,
                    },
                    StateDiff {
                        checkpoint: "idle_tail".into(),
                        field: "number_objects".into(),
                        native_value: "2".into(),
                        oracle_value: "4".into(),
                        category: DivergenceCategory::Advisory,
                    },
                ],
                mae: None,
                pixel_mismatches: None,
                diff_images: Vec::new(),
                error: None,
                divergence_summary: DivergenceSummary::default(),
            },
        ];

        let summary = summarise_field_diffs(&results);

        assert_eq!(
            summary,
            vec![
                SweepHistoryFieldEntry {
                    field: "camera_y_pos".into(),
                    total_diffs: 3,
                    levels: 2,
                    level_ids: vec!["0x00".into(), "0x01".into()],
                    checkpoints: vec!["idle_tail".into(), "level_settle".into()],
                    checkpoint_counts: vec![
                        SweepHistoryCheckpointCount {
                            checkpoint: "level_settle".into(),
                            diffs: 2,
                        },
                        SweepHistoryCheckpointCount {
                            checkpoint: "idle_tail".into(),
                            diffs: 1,
                        },
                    ],
                    divergence_summary: DivergenceSummary {
                        authoritative: 0,
                        advisory: 0,
                        accumulation: 0,
                        rng: 0,
                        oracle_limitation: 3,
                    },
                },
                SweepHistoryFieldEntry {
                    field: "game_mode".into(),
                    total_diffs: 1,
                    levels: 1,
                    level_ids: vec!["0x00".into()],
                    checkpoints: vec!["idle_tail".into()],
                    checkpoint_counts: vec![SweepHistoryCheckpointCount {
                        checkpoint: "idle_tail".into(),
                        diffs: 1,
                    }],
                    divergence_summary: DivergenceSummary {
                        authoritative: 0,
                        advisory: 1,
                        accumulation: 0,
                        rng: 0,
                        oracle_limitation: 0,
                    },
                },
                SweepHistoryFieldEntry {
                    field: "number_objects".into(),
                    total_diffs: 1,
                    levels: 1,
                    level_ids: vec!["0x01".into()],
                    checkpoints: vec!["idle_tail".into()],
                    checkpoint_counts: vec![SweepHistoryCheckpointCount {
                        checkpoint: "idle_tail".into(),
                        diffs: 1,
                    }],
                    divergence_summary: DivergenceSummary {
                        authoritative: 0,
                        advisory: 1,
                        accumulation: 0,
                        rng: 0,
                        oracle_limitation: 0,
                    },
                },
            ]
        );
    }

    #[test]
    fn extracts_release_gate_failed_field_from_detail() {
        let detail = "state title@360 diverged in game mode (native 000C, oracle 002C); artefacts in /tmp/run";

        assert_eq!(
            extract_release_gate_failed_field(detail).as_deref(),
            Some("game_mode")
        );
    }

    #[test]
    fn normalises_release_gate_values_without_turning_decimal_counts_into_hex() {
        assert_eq!(normalise_release_gate_value("000C"), "0x000c");
        assert_eq!(normalise_release_gate_value("0x002C"), "0x002c");
        assert_eq!(normalise_release_gate_value("3"), "3");
        assert_eq!(normalise_release_gate_value("120"), "120");
    }

    #[test]
    fn extracts_release_gate_scenario_level_id_from_step_name() {
        assert_eq!(
            extract_release_gate_scenario_level_id("frame_oracle:boot_title_start_level1")
                .as_deref(),
            Some("0x00")
        );
        assert_eq!(
            extract_release_gate_scenario_level_id("frame_oracle:boot_title_start_level10")
                .as_deref(),
            Some("0x09")
        );
        assert_eq!(
            extract_release_gate_scenario_level_id("frame_oracle:boot_title_idle_10s"),
            None
        );
    }

    #[test]
    fn latest_release_gate_failed_fields_reads_latest_reports() {
        let temp_dir = tempfile::tempdir().expect("tempdir");
        let gate_dir = temp_dir.path().join(".cache/release-gate/1p-accurate");
        std::fs::create_dir_all(&gate_dir).expect("create gate dir");
        std::fs::write(
            gate_dir.join("latest.json"),
            r#"{
  "target": "1p-accurate",
  "passed": false,
  "steps": [
    {
      "name": "frame_oracle:boot_title_start_level1",
      "status": "failed",
      "detail": "state title@360 diverged in game mode (native 000C, oracle 002C); artefacts in /tmp/run"
    }
  ]
}"#,
        )
        .expect("write report");

        assert_eq!(
            latest_release_gate_failed_fields(temp_dir.path()),
            vec![ReleaseGateFieldFocus {
                target: "1p-accurate".into(),
                step_name: "frame_oracle:boot_title_start_level1".into(),
                checkpoint: Some("title@360".into()),
                field: "game_mode".into(),
                native_value: Some("0x000c".into()),
                oracle_value: Some("0x002c".into()),
                scenario_level_id: Some("0x00".into()),
            }]
        );
    }

    #[test]
    fn release_gate_focus_summary_uses_field_histogram_entry() {
        let focus = ReleaseGateFieldFocus {
            target: "1p-accurate".into(),
            step_name: "frame_oracle:boot_title_start_level1".into(),
            checkpoint: Some("title@360".into()),
            field: "game_mode".into(),
            native_value: Some("0x000c".into()),
            oracle_value: Some("0x002c".into()),
            scenario_level_id: Some("0x00".into()),
        };
        let field_summary = vec![SweepHistoryFieldEntry {
            field: "game_mode".into(),
            total_diffs: 15,
            levels: 15,
            level_ids: vec![
                "0x09".into(),
                "0x32".into(),
                "0x33".into(),
                "0x50".into(),
                "0x57".into(),
                "0x5a".into(),
            ],
            checkpoints: vec!["idle_tail".into(), "level_settle".into()],
            checkpoint_counts: vec![
                SweepHistoryCheckpointCount {
                    checkpoint: "idle_tail".into(),
                    diffs: 9,
                },
                SweepHistoryCheckpointCount {
                    checkpoint: "level_settle".into(),
                    diffs: 6,
                },
            ],
            divergence_summary: DivergenceSummary {
                authoritative: 0,
                advisory: 0,
                accumulation: 0,
                rng: 0,
                oracle_limitation: 15,
            },
        }];

        assert_eq!(
            format_release_gate_focus_summary(&field_summary, &focus),
            "1p-accurate failing field game_mode=15 diffs across 15 levels for boot_title_start_level1 @ title@360 [oracle_limitations=15] [0x09, 0x32, 0x33, 0x50, 0x57, +1 more] [excludes scenario level 0x00] [excludes checkpoint title@360] [excludes checkpoint family title] checkpoints=[idle_tail=9, level_settle=6] (field-level coverage only; exact pair evidence lives in release_gate_focus, and the release-gate scenario still needs its own proof)"
        );
    }

    #[test]
    fn release_gate_focus_summary_persists_matched_histogram_metadata() {
        let focuses = vec![ReleaseGateFieldFocus {
            target: "1p-accurate".into(),
            step_name: "frame_oracle:boot_title_start_level1".into(),
            checkpoint: Some("title@360".into()),
            field: "game_mode".into(),
            native_value: Some("0x000c".into()),
            oracle_value: Some("0x002c".into()),
            scenario_level_id: Some("0x00".into()),
        }];
        let field_summary = vec![SweepHistoryFieldEntry {
            field: "game_mode".into(),
            total_diffs: 15,
            levels: 15,
            level_ids: vec!["0x00".into(), "0x09".into()],
            checkpoints: vec!["idle_tail".into(), "title@360".into()],
            checkpoint_counts: vec![
                SweepHistoryCheckpointCount {
                    checkpoint: "idle_tail".into(),
                    diffs: 10,
                },
                SweepHistoryCheckpointCount {
                    checkpoint: "title@360".into(),
                    diffs: 5,
                },
            ],
            divergence_summary: DivergenceSummary {
                authoritative: 0,
                advisory: 0,
                accumulation: 0,
                rng: 0,
                oracle_limitation: 15,
            },
        }];

        assert_eq!(
            summarise_release_gate_focuses(&[], &field_summary, &focuses),
            vec![ReleaseGateFocusSummary {
                target: "1p-accurate".into(),
                step_name: "frame_oracle:boot_title_start_level1".into(),
                checkpoint: Some("title@360".into()),
                field: "game_mode".into(),
                native_value: Some("0x000c".into()),
                oracle_value: Some("0x002c".into()),
                scenario_level_id: Some("0x00".into()),
                matched_in_state_sweep: true,
                scenario_level_matched_in_state_sweep: Some(true),
                checkpoint_matched_in_state_sweep: Some(true),
                checkpoint_family_matched_in_state_sweep: Some(true),
                exact_value_pair_matched_in_state_sweep: Some(false),
                exact_value_pair_levels: Some(0),
                exact_value_pair_level_ids: Vec::new(),
                exact_value_pair_scenario_level_matched_in_state_sweep: Some(false),
                exact_value_pair_checkpoint_matched_in_state_sweep: Some(false),
                exact_value_pair_checkpoint_family_matched_in_state_sweep: Some(false),
                total_diffs: Some(15),
                levels: Some(15),
                level_ids: vec!["0x00".into(), "0x09".into()],
                matched_checkpoints: vec![
                    SweepHistoryCheckpointCount {
                        checkpoint: "idle_tail".into(),
                        diffs: 10,
                    },
                    SweepHistoryCheckpointCount {
                        checkpoint: "title@360".into(),
                        diffs: 5,
                    },
                ],
                exact_value_pair_checkpoints: Vec::new(),
                divergence_summary: DivergenceSummary {
                    authoritative: 0,
                    advisory: 0,
                    accumulation: 0,
                    rng: 0,
                    oracle_limitation: 15,
                },
                summary: "1p-accurate failing field game_mode=15 diffs across 15 levels for boot_title_start_level1 @ title@360 [oracle_limitations=15] [0x00, 0x09] [includes scenario level 0x00] [includes checkpoint title@360] [includes checkpoint family title] checkpoints=[idle_tail=10, title@360=5] (field-level coverage only; exact pair evidence lives in release_gate_focus, and the release-gate scenario still needs its own proof) [exact pair 0x000c->0x002c absent from latest state sweep; same field class, different concrete miss]".into(),
            }]
        );
    }

    #[test]
    fn release_gate_focus_summary_marks_unmatched_fields() {
        let focuses = vec![ReleaseGateFieldFocus {
            target: "1p-accurate".into(),
            step_name: "frame_oracle:boot_title_start_level1".into(),
            checkpoint: Some("title@360".into()),
            field: "boot_phase_elapsed".into(),
            native_value: Some("120".into()),
            oracle_value: Some("150".into()),
            scenario_level_id: Some("0x00".into()),
        }];

        assert_eq!(
            summarise_release_gate_focuses(&[], &[], &focuses),
            vec![ReleaseGateFocusSummary {
                target: "1p-accurate".into(),
                step_name: "frame_oracle:boot_title_start_level1".into(),
                checkpoint: Some("title@360".into()),
                field: "boot_phase_elapsed".into(),
                native_value: Some("120".into()),
                oracle_value: Some("150".into()),
                scenario_level_id: Some("0x00".into()),
                matched_in_state_sweep: false,
                scenario_level_matched_in_state_sweep: Some(false),
                checkpoint_matched_in_state_sweep: Some(false),
                checkpoint_family_matched_in_state_sweep: Some(false),
                exact_value_pair_matched_in_state_sweep: Some(false),
                exact_value_pair_levels: Some(0),
                exact_value_pair_level_ids: Vec::new(),
                exact_value_pair_scenario_level_matched_in_state_sweep: Some(false),
                exact_value_pair_checkpoint_matched_in_state_sweep: Some(false),
                exact_value_pair_checkpoint_family_matched_in_state_sweep: Some(false),
                total_diffs: None,
                levels: None,
                level_ids: Vec::new(),
                matched_checkpoints: Vec::new(),
                exact_value_pair_checkpoints: Vec::new(),
                divergence_summary: DivergenceSummary::default(),
                summary: "1p-accurate failing field boot_phase_elapsed for boot_title_start_level1 @ title@360 not present in latest state sweep field_summary [exact pair 120->150 absent from latest state sweep; same field class, different concrete miss]".into(),
            }]
        );
    }

    #[test]
    fn release_gate_focus_summary_marks_exact_value_pair_matches() {
        let focuses = vec![ReleaseGateFieldFocus {
            target: "1p-accurate".into(),
            step_name: "frame_oracle:boot_title_start_level1".into(),
            checkpoint: Some("title@360".into()),
            field: "game_mode".into(),
            native_value: Some("0x000c".into()),
            oracle_value: Some("0x002c".into()),
            scenario_level_id: Some("0x00".into()),
        }];
        let field_summary = vec![SweepHistoryFieldEntry {
            field: "game_mode".into(),
            total_diffs: 2,
            levels: 1,
            level_ids: vec!["0x00".into()],
            checkpoints: vec!["title@360".into()],
            checkpoint_counts: vec![SweepHistoryCheckpointCount {
                checkpoint: "title@360".into(),
                diffs: 2,
            }],
            divergence_summary: DivergenceSummary {
                authoritative: 0,
                advisory: 0,
                accumulation: 0,
                rng: 0,
                oracle_limitation: 2,
            },
        }];
        let results = vec![LevelSweepResult {
            level_id: 0x00,
            slug: "idle_level_00".into(),
            tier: SweepTier::State,
            passed: true,
            state_diffs: vec![
                StateDiff {
                    checkpoint: "title@360".into(),
                    field: "game_mode".into(),
                    native_value: "0x000c".into(),
                    oracle_value: "0x002c".into(),
                    category: DivergenceCategory::OracleLimitation,
                },
                StateDiff {
                    checkpoint: "title@360".into(),
                    field: "game_mode".into(),
                    native_value: "0x000c".into(),
                    oracle_value: "0x002c".into(),
                    category: DivergenceCategory::OracleLimitation,
                },
            ],
            mae: None,
            pixel_mismatches: None,
            diff_images: Vec::new(),
            error: None,
            divergence_summary: DivergenceSummary {
                authoritative: 0,
                advisory: 0,
                accumulation: 0,
                rng: 0,
                oracle_limitation: 2,
            },
        }];

        assert_eq!(
            summarise_release_gate_focuses(&results, &field_summary, &focuses),
            vec![ReleaseGateFocusSummary {
                target: "1p-accurate".into(),
                step_name: "frame_oracle:boot_title_start_level1".into(),
                checkpoint: Some("title@360".into()),
                field: "game_mode".into(),
                native_value: Some("0x000c".into()),
                oracle_value: Some("0x002c".into()),
                scenario_level_id: Some("0x00".into()),
                matched_in_state_sweep: true,
                scenario_level_matched_in_state_sweep: Some(true),
                checkpoint_matched_in_state_sweep: Some(true),
                checkpoint_family_matched_in_state_sweep: Some(true),
                exact_value_pair_matched_in_state_sweep: Some(true),
                exact_value_pair_levels: Some(1),
                exact_value_pair_level_ids: vec!["0x00".into()],
                exact_value_pair_scenario_level_matched_in_state_sweep: Some(true),
                exact_value_pair_checkpoint_matched_in_state_sweep: Some(true),
                exact_value_pair_checkpoint_family_matched_in_state_sweep: Some(true),
                total_diffs: Some(2),
                levels: Some(1),
                level_ids: vec!["0x00".into()],
                matched_checkpoints: vec![SweepHistoryCheckpointCount {
                    checkpoint: "title@360".into(),
                    diffs: 2,
                }],
                exact_value_pair_checkpoints: vec![SweepHistoryCheckpointCount {
                    checkpoint: "title@360".into(),
                    diffs: 2,
                }],
                divergence_summary: DivergenceSummary {
                    authoritative: 0,
                    advisory: 0,
                    accumulation: 0,
                    rng: 0,
                    oracle_limitation: 2,
                },
                summary: "1p-accurate failing field game_mode=2 diffs across 1 levels for boot_title_start_level1 @ title@360 [oracle_limitations=2] [0x00] [includes scenario level 0x00] [includes checkpoint title@360] [includes checkpoint family title] checkpoints=[title@360=2] (field-level coverage only; exact pair evidence lives in release_gate_focus, and the release-gate scenario still needs its own proof) [exact pair 0x000c->0x002c seen on levels [0x00] checkpoints=[title@360=2]; includes scenario level 0x00; includes checkpoint title@360; includes checkpoint family title]".into(),
            }]
        );
    }

    #[test]
    fn release_gate_focus_summary_marks_exact_pair_checkpoint_misses() {
        let focuses = vec![ReleaseGateFieldFocus {
            target: "1p-accurate".into(),
            step_name: "gameplay_compare:pause_resume_level1".into(),
            checkpoint: Some("level_settle@719".into()),
            field: "game_mode".into(),
            native_value: Some("0x0008".into()),
            oracle_value: Some("0x000c".into()),
            scenario_level_id: Some("0x00".into()),
        }];
        let field_summary = vec![SweepHistoryFieldEntry {
            field: "game_mode".into(),
            total_diffs: 13,
            levels: 12,
            level_ids: vec![
                "0x00".into(),
                "0x09".into(),
                "0x32".into(),
                "0x33".into(),
                "0x45".into(),
                "0x48".into(),
                "0x50".into(),
                "0x54".into(),
                "0x57".into(),
                "0x61".into(),
                "0x62".into(),
                "0x63".into(),
            ],
            checkpoints: vec!["idle_tail".into(), "level_settle".into()],
            checkpoint_counts: vec![
                SweepHistoryCheckpointCount {
                    checkpoint: "idle_tail".into(),
                    diffs: 12,
                },
                SweepHistoryCheckpointCount {
                    checkpoint: "level_settle".into(),
                    diffs: 1,
                },
            ],
            divergence_summary: DivergenceSummary {
                authoritative: 0,
                advisory: 0,
                accumulation: 0,
                rng: 0,
                oracle_limitation: 13,
            },
        }];
        let results = vec![LevelSweepResult {
            level_id: 0x00,
            slug: "idle_level_00".into(),
            tier: SweepTier::State,
            passed: true,
            state_diffs: vec![StateDiff {
                checkpoint: "idle_tail".into(),
                field: "game_mode".into(),
                native_value: "0x0008".into(),
                oracle_value: "0x000c".into(),
                category: DivergenceCategory::OracleLimitation,
            }],
            mae: None,
            pixel_mismatches: None,
            diff_images: Vec::new(),
            error: None,
            divergence_summary: DivergenceSummary {
                authoritative: 0,
                advisory: 0,
                accumulation: 0,
                rng: 0,
                oracle_limitation: 1,
            },
        }];

        assert_eq!(
            summarise_release_gate_focuses(&results, &field_summary, &focuses),
            vec![ReleaseGateFocusSummary {
                target: "1p-accurate".into(),
                step_name: "gameplay_compare:pause_resume_level1".into(),
                checkpoint: Some("level_settle@719".into()),
                field: "game_mode".into(),
                native_value: Some("0x0008".into()),
                oracle_value: Some("0x000c".into()),
                scenario_level_id: Some("0x00".into()),
                matched_in_state_sweep: true,
                scenario_level_matched_in_state_sweep: Some(true),
                checkpoint_matched_in_state_sweep: Some(false),
                checkpoint_family_matched_in_state_sweep: Some(true),
                exact_value_pair_matched_in_state_sweep: Some(true),
                exact_value_pair_levels: Some(1),
                exact_value_pair_level_ids: vec!["0x00".into()],
                exact_value_pair_scenario_level_matched_in_state_sweep: Some(true),
                exact_value_pair_checkpoint_matched_in_state_sweep: Some(false),
                exact_value_pair_checkpoint_family_matched_in_state_sweep: Some(false),
                total_diffs: Some(13),
                levels: Some(12),
                level_ids: vec![
                    "0x00".into(),
                    "0x09".into(),
                    "0x32".into(),
                    "0x33".into(),
                    "0x45".into(),
                    "0x48".into(),
                    "0x50".into(),
                    "0x54".into(),
                    "0x57".into(),
                    "0x61".into(),
                    "0x62".into(),
                    "0x63".into(),
                ],
                matched_checkpoints: vec![
                    SweepHistoryCheckpointCount {
                        checkpoint: "idle_tail".into(),
                        diffs: 12,
                    },
                    SweepHistoryCheckpointCount {
                        checkpoint: "level_settle".into(),
                        diffs: 1,
                    },
                ],
                exact_value_pair_checkpoints: vec![SweepHistoryCheckpointCount {
                    checkpoint: "idle_tail".into(),
                    diffs: 1,
                }],
                divergence_summary: DivergenceSummary {
                    authoritative: 0,
                    advisory: 0,
                    accumulation: 0,
                    rng: 0,
                    oracle_limitation: 13,
                },
                summary: "1p-accurate failing field game_mode=13 diffs across 12 levels for pause_resume_level1 @ level_settle@719 [oracle_limitations=13] [0x00, 0x09, 0x32, 0x33, 0x45, +7 more] [includes scenario level 0x00] [excludes checkpoint level_settle@719] [includes checkpoint family level_settle] checkpoints=[idle_tail=12, level_settle=1] (field-level coverage only; exact pair evidence lives in release_gate_focus, and the release-gate scenario still needs its own proof) [exact pair 0x0008->0x000c seen on levels [0x00] checkpoints=[idle_tail=1]; includes scenario level 0x00; excludes checkpoint level_settle@719; excludes checkpoint family level_settle]".into(),
            }]
        );
    }

    #[test]
    fn release_gate_focus_summary_marks_exact_pair_on_other_levels() {
        let focuses = vec![ReleaseGateFieldFocus {
            target: "1p-accurate".into(),
            step_name: "gameplay_compare:pause_resume_level1".into(),
            checkpoint: Some("level_settle@719".into()),
            field: "game_mode".into(),
            native_value: Some("0x0008".into()),
            oracle_value: Some("0x000c".into()),
            scenario_level_id: Some("0x00".into()),
        }];
        let field_summary = vec![SweepHistoryFieldEntry {
            field: "game_mode".into(),
            total_diffs: 13,
            levels: 12,
            level_ids: vec![
                "0x00".into(),
                "0x09".into(),
                "0x32".into(),
                "0x33".into(),
                "0x45".into(),
                "0x48".into(),
                "0x50".into(),
                "0x54".into(),
                "0x57".into(),
                "0x61".into(),
                "0x62".into(),
                "0x63".into(),
            ],
            checkpoints: vec!["idle_tail".into(), "level_settle".into()],
            checkpoint_counts: vec![
                SweepHistoryCheckpointCount {
                    checkpoint: "idle_tail".into(),
                    diffs: 12,
                },
                SweepHistoryCheckpointCount {
                    checkpoint: "level_settle".into(),
                    diffs: 1,
                },
            ],
            divergence_summary: DivergenceSummary {
                authoritative: 0,
                advisory: 0,
                accumulation: 0,
                rng: 0,
                oracle_limitation: 13,
            },
        }];
        let results = vec![
            0x09, 0x32, 0x33, 0x45, 0x48, 0x50, 0x54, 0x57, 0x61, 0x62, 0x63,
        ]
        .into_iter()
        .map(|level_id| LevelSweepResult {
            level_id,
            slug: format!("idle_level_{level_id:02x}"),
            tier: SweepTier::State,
            passed: true,
            state_diffs: vec![StateDiff {
                checkpoint: "idle_tail".into(),
                field: "game_mode".into(),
                native_value: "0x0008".into(),
                oracle_value: "0x000c".into(),
                category: DivergenceCategory::OracleLimitation,
            }],
            mae: None,
            pixel_mismatches: None,
            diff_images: Vec::new(),
            error: None,
            divergence_summary: DivergenceSummary {
                authoritative: 0,
                advisory: 0,
                accumulation: 0,
                rng: 0,
                oracle_limitation: 1,
            },
        })
        .collect::<Vec<_>>();

        assert_eq!(
            summarise_release_gate_focuses(&results, &field_summary, &focuses),
            vec![ReleaseGateFocusSummary {
                target: "1p-accurate".into(),
                step_name: "gameplay_compare:pause_resume_level1".into(),
                checkpoint: Some("level_settle@719".into()),
                field: "game_mode".into(),
                native_value: Some("0x0008".into()),
                oracle_value: Some("0x000c".into()),
                scenario_level_id: Some("0x00".into()),
                matched_in_state_sweep: true,
                scenario_level_matched_in_state_sweep: Some(true),
                checkpoint_matched_in_state_sweep: Some(false),
                checkpoint_family_matched_in_state_sweep: Some(true),
                exact_value_pair_matched_in_state_sweep: Some(true),
                exact_value_pair_levels: Some(11),
                exact_value_pair_level_ids: vec![
                    "0x09".into(),
                    "0x32".into(),
                    "0x33".into(),
                    "0x45".into(),
                    "0x48".into(),
                    "0x50".into(),
                    "0x54".into(),
                    "0x57".into(),
                    "0x61".into(),
                    "0x62".into(),
                    "0x63".into(),
                ],
                exact_value_pair_scenario_level_matched_in_state_sweep: Some(false),
                exact_value_pair_checkpoint_matched_in_state_sweep: Some(false),
                exact_value_pair_checkpoint_family_matched_in_state_sweep: Some(false),
                total_diffs: Some(13),
                levels: Some(12),
                level_ids: vec![
                    "0x00".into(),
                    "0x09".into(),
                    "0x32".into(),
                    "0x33".into(),
                    "0x45".into(),
                    "0x48".into(),
                    "0x50".into(),
                    "0x54".into(),
                    "0x57".into(),
                    "0x61".into(),
                    "0x62".into(),
                    "0x63".into(),
                ],
                matched_checkpoints: vec![
                    SweepHistoryCheckpointCount {
                        checkpoint: "idle_tail".into(),
                        diffs: 12,
                    },
                    SweepHistoryCheckpointCount {
                        checkpoint: "level_settle".into(),
                        diffs: 1,
                    },
                ],
                exact_value_pair_checkpoints: vec![SweepHistoryCheckpointCount {
                    checkpoint: "idle_tail".into(),
                    diffs: 11,
                }],
                divergence_summary: DivergenceSummary {
                    authoritative: 0,
                    advisory: 0,
                    accumulation: 0,
                    rng: 0,
                    oracle_limitation: 13,
                },
                summary: "1p-accurate failing field game_mode=13 diffs across 12 levels for pause_resume_level1 @ level_settle@719 [oracle_limitations=13] [0x00, 0x09, 0x32, 0x33, 0x45, +7 more] [includes scenario level 0x00] [excludes checkpoint level_settle@719] [includes checkpoint family level_settle] checkpoints=[idle_tail=12, level_settle=1] (field-level coverage only; exact pair evidence lives in release_gate_focus, and the release-gate scenario still needs its own proof) [exact pair 0x0008->0x000c seen on levels [0x09, 0x32, 0x33, 0x45, 0x48, +6 more] checkpoints=[idle_tail=11]; excludes scenario level 0x00; excludes checkpoint level_settle@719; excludes checkpoint family level_settle]".into(),
            }]
        );
    }

    #[test]
    fn high_frame_mae_is_not_oracle_limitation_only() {
        let result = LevelSweepResult {
            level_id: 0x45,
            slug: "idle_level_45".into(),
            tier: SweepTier::Frames,
            passed: false,
            state_diffs: Vec::new(),
            mae: Some(42.0),
            pixel_mismatches: None,
            diff_images: Vec::new(),
            error: None,
            divergence_summary: DivergenceSummary {
                authoritative: 0,
                advisory: 0,
                accumulation: 0,
                rng: 0,
                oracle_limitation: 4,
            },
        };

        assert!(!is_oracle_limitation_only(&result));
    }

    #[test]
    fn golden_diff_detects_changed_field() {
        use kid_core::{GameMode, LevelId, NativeTraceFrame};
        let checkpoints = vec![DynamicCheckpoint {
            label: "settle".into(),
            frame_index: 0,
        }];
        let make_frame = |score: u32| NativeTraceFrame {
            frame_index: 0,
            game_mode: GameMode::InGame,
            game_mode_value: GameMode::InGame.as_disassembly_value(),
            start_press_count: 0,
            boot_phase: None,
            boot_phase_elapsed: None,
            title_screen_ready: false,
            current_player: 0,
            current_level: LevelId(0),
            player_1_level_id: LevelId(0),
            current_level_name: None,
            player_start_x: None,
            player_start_y: None,
            flag_x: None,
            flag_y: None,
            level_enemy_object_count: None,
            active_enemy_object_count: 0,
            active_platform_object_count: 0,
            number_lives: 3,
            number_continues: 3,
            number_hitpoints: 2,
            current_helmet_id: 0,
            paused: false,
            kid_x_pos: Some(0x0020_0000),
            kid_y_pos: Some(0x00E0_0000),
            kid_x_velocity: Some(0),
            kid_y_velocity: Some(0),
            character_movement: None,
            kid_facing: None,
            kid_on_ground: Some(true),
            kid_on_slope: 0,
            active_objects: Vec::new(),
            camera_x_pos: 0,
            camera_y_pos: 0,
            number_objects: 0,
            number_gfx_objects: 0,
            rng_offset: None,
            level_complete: false,
            game_over: false,
            continue_countdown: None,
            game_complete: false,
            ending_elapsed: None,
            options_cursor: None,
            two_player: false,
            options_two_player_two_controllers: false,
            options_controls_scheme: 0,
            options_speed_button_fast: false,
            invulnerable_frames: 0,
            just_received_damage: false,
            diamond_power_active: false,
            diamond_count: 0,
            score,
            level_timer_seconds: 180,
            level_damage_taken: false,
            murder_wall_x: None,
            active_projectiles: Vec::new(),
            active_projectile_details: Vec::new(),
            active_geysers: Vec::new(),
            block_bounces: Vec::new(),
            block_destroy_effects: Vec::new(),
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
        };

        let old = vec![make_frame(0)];
        let new = vec![make_frame(100)];
        let diffs = diff_golden_frames("idle_level_00", &old, &new, &checkpoints);
        assert_eq!(diffs.len(), 1);
        assert_eq!(diffs[0].field, "score");
        assert!(diffs[0].old_value.contains('0'));
        assert!(diffs[0].new_value.contains("100"));
    }

    #[test]
    fn golden_diff_reports_nothing_for_identical_frames() {
        use kid_core::{GameMode, LevelId, NativeTraceFrame};
        let checkpoints = vec![DynamicCheckpoint {
            label: "settle".into(),
            frame_index: 0,
        }];
        let frame = NativeTraceFrame {
            frame_index: 0,
            game_mode: GameMode::InGame,
            game_mode_value: GameMode::InGame.as_disassembly_value(),
            start_press_count: 0,
            boot_phase: None,
            boot_phase_elapsed: None,
            title_screen_ready: false,
            current_player: 0,
            current_level: LevelId(0),
            player_1_level_id: LevelId(0),
            current_level_name: None,
            player_start_x: None,
            player_start_y: None,
            flag_x: None,
            flag_y: None,
            level_enemy_object_count: None,
            active_enemy_object_count: 0,
            active_platform_object_count: 0,
            number_lives: 3,
            number_continues: 3,
            number_hitpoints: 2,
            current_helmet_id: 0,
            paused: false,
            kid_x_pos: Some(0x0020_0000),
            kid_y_pos: Some(0x00E0_0000),
            kid_x_velocity: Some(0),
            kid_y_velocity: Some(0),
            character_movement: None,
            kid_facing: None,
            kid_on_ground: Some(true),
            kid_on_slope: 0,
            active_objects: Vec::new(),
            camera_x_pos: 0,
            camera_y_pos: 0,
            number_objects: 0,
            number_gfx_objects: 0,
            rng_offset: None,
            level_complete: false,
            game_over: false,
            continue_countdown: None,
            game_complete: false,
            ending_elapsed: None,
            options_cursor: None,
            two_player: false,
            options_two_player_two_controllers: false,
            options_controls_scheme: 0,
            options_speed_button_fast: false,
            invulnerable_frames: 0,
            just_received_damage: false,
            diamond_power_active: false,
            diamond_count: 0,
            score: 0,
            level_timer_seconds: 180,
            level_damage_taken: false,
            murder_wall_x: None,
            active_projectiles: Vec::new(),
            active_projectile_details: Vec::new(),
            active_geysers: Vec::new(),
            block_bounces: Vec::new(),
            block_destroy_effects: Vec::new(),
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
        };

        let old = vec![frame.clone()];
        let new = vec![frame];
        let diffs = diff_golden_frames("idle_level_00", &old, &new, &checkpoints);
        assert!(diffs.is_empty(), "expected no diffs but got: {diffs:?}");
    }

    #[test]
    fn golden_scenarios_produces_four_scenarios() {
        let scenarios = golden_scenarios_for_level(0x00);
        assert_eq!(scenarios.len(), 4);
        assert!(scenarios[0].slug.starts_with("idle_level_"));
        assert!(scenarios[1].slug.starts_with("walk_right_"));
        assert!(scenarios[2].slug.starts_with("prize_block_"));
        assert!(scenarios[3].slug.starts_with("stomp_enemy_"));
    }
}
