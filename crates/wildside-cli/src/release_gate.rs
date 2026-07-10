use super::*;
use clap::ValueEnum;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::process::Command;

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum, Serialize, Deserialize)]
pub(crate) enum ReleaseGateTarget {
    #[value(name = "1p-playable")]
    #[serde(rename = "1p-playable")]
    OnePlayerPlayable,
    #[value(name = "1p-accurate")]
    #[serde(rename = "1p-accurate")]
    OnePlayerAccurate,
}

impl ReleaseGateTarget {
    pub(crate) fn slug(self) -> &'static str {
        match self {
            Self::OnePlayerPlayable => "1p-playable",
            Self::OnePlayerAccurate => "1p-accurate",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
enum ReleaseGateStepStatus {
    Passed,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct ReleaseGateStep {
    name: String,
    status: ReleaseGateStepStatus,
    detail: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct ReleaseGateReport {
    timestamp: String,
    target: String,
    passed: bool,
    steps: Vec<ReleaseGateStep>,
    #[serde(skip_serializing_if = "Option::is_none")]
    sweep_summary: Option<sweep::DivergenceSummary>,
    // Sweep parity: persist the wildside-core source SHA1 the embedded state
    // sweep ran against. Without this, the gate report cites a passing
    // outcome but the operator has to chase the linked sweep-runs report
    // to verify the gate ran on the live tree. Carrying the hash in the
    // gate artefact lets `latest_release_gate_signal` self-cite source
    // freshness and drift in the same drift-aware way the state-sweep
    // signal already does. Optional so 1p-playable runs (which never
    // invoke the state sweep) and pre-field reports stay readable.
    #[serde(skip_serializing_if = "Option::is_none")]
    kid_core_hash: Option<String>,
    // Sweep parity: persist the breadth (level count) the embedded state
    // sweep ran across. The state-sweep signal already cites `levels=N`
    // alongside the divergence breakdown, but the release-gate signal
    // historically had no equivalent — `passed=true` plus
    // `sweep_summary=authoritative=0` proved the divergence threshold
    // without proving the breadth, so a gate that swept only a handful
    // of levels could pass green and look identical to a whole-game
    // gate. Carrying the requested level count in the gate artefact
    // lets `latest_release_gate_signal` cite breadth in the same shape
    // the state-sweep signal does. Set whether the sweep passes or
    // fails so the failing branch also self-cites breadth. Optional so
    // 1p-playable runs and pre-field reports stay readable.
    #[serde(skip_serializing_if = "Option::is_none")]
    levels_requested: Option<usize>,
    // Sweep parity: persist a fingerprint of the dominant divergence
    // field — the name and total_diffs of `field_summary[0]` from the
    // embedded sweep, which is the field with the most diffs across
    // the most levels (deterministic via total_diffs desc, levels
    // desc, name asc). `sweep_summary` already carries the per-category
    // totals, but a green gate with `oracle_limitation=840` says nothing
    // about whether the underlying field shape is unchanged. If a future
    // run flips the dominant field (e.g. `score` overtakes
    // `camera_y_pos`), that's a meaningful state-shape regression that
    // the existing signal would mask. Persisting the top-field name
    // alongside its diff count lets `latest_release_gate_signal`
    // self-cite the fingerprint so downstream status readers carry it
    // forward without chasing the linked sweep-runs report. Optional
    // so 1p-playable runs (no sweep) and pre-field reports stay
    // readable, and skipped when the embedded sweep recorded zero
    // diffs (no dominant field exists).
    #[serde(skip_serializing_if = "Option::is_none")]
    top_divergence_field: Option<TopDivergenceField>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct TopDivergenceField {
    pub(crate) field: String,
    pub(crate) total_diffs: usize,
    // Sweep parity: persist the breadth (distinct level count) the
    // dominant field diverged across. `total_diffs` alone does not
    // distinguish a regression that concentrates 200 diffs on five
    // levels from a baseline that spreads 200 diffs across all 69
    // levels — the gate signal would render the same `top_field`
    // fingerprint in both cases. Persisting `levels` lets the signal
    // surface the breadth half of the dominant-field shape so a
    // shift in concentration is visible from the gate signal
    // alone. Optional so pre-field reports stay readable.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) levels: Option<usize>,
    // Sweep parity: persist the breadth (distinct checkpoint count)
    // the dominant field diverged across. `levels` already discloses
    // the level-axis breadth, but two regressions can spread the same
    // diff total across the same level count while occupying very
    // different checkpoint shapes — e.g. 136 diffs on 68 levels at a
    // single `level_settle` checkpoint vs the same total scattered
    // across `idle_tail`, `level_settle`, and `pre_warp`. Persisting
    // the checkpoint count lets the gate signal surface that
    // checkpoint-axis shift in the same shape the level-axis already
    // does. Optional so pre-checkpoint reports stay readable.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) checkpoints: Option<usize>,
}

const NATIVE_GAMEPLAY_SMOKE_SCENARIOS: &[GameplayScenario] = &[
    GameplayScenario::PauseResumeLevel1,
    GameplayScenario::ProgressTwoTransitions,
    GameplayScenario::SoakLevel10m,
];

const STATIC_NATIVE_GAMEPLAY_COMPARE_SCENARIOS: &[GameplayScenario] =
    &[GameplayScenario::PauseResumeLevel1];

const FULL_BOOT_GAMEPLAY_VERIFY_SCENARIOS: &[GameplayScenario] = &[
    GameplayScenario::ProgressTwoTransitions,
    GameplayScenario::DeathRespawnLevel1,
    GameplayScenario::DeathRespawnStartResumeLevel1,
];

const SOURCE_PORT_VISUAL_CHECKPOINTS: usize = 4;
const SOURCE_PORT_VISUAL_MAX_MAE: f32 = 0.0;
const SOURCE_PORT_VISUAL_MAX_PIXEL_MISMATCHES: usize = 0;

pub(crate) fn run_release_gate(
    repo_root: &Path,
    rom_path: &Path,
    manifest: &AssetManifest,
    target: ReleaseGateTarget,
    max_failures: usize,
) -> Result<()> {
    let timestamp = unix_timestamp();
    let (run_dir, report_path) = release_gate_report_paths(repo_root, target, &timestamp);
    fs::create_dir_all(&run_dir)
        .with_context(|| format!("failed to create {}", run_dir.display()))?;

    let mut report = ReleaseGateReport {
        timestamp,
        target: target.slug().to_string(),
        passed: false,
        steps: Vec::new(),
        sweep_summary: None,
        kid_core_hash: None,
        levels_requested: None,
        top_divergence_field: None,
    };

    let result = match target {
        ReleaseGateTarget::OnePlayerPlayable => {
            run_one_player_playable_gate(repo_root, rom_path, manifest, &mut report)
        }
        ReleaseGateTarget::OnePlayerAccurate => {
            run_one_player_accurate_gate(repo_root, rom_path, manifest, max_failures, &mut report)
        }
    };

    report.passed = result.is_ok();
    fs::write(&report_path, serde_json::to_vec_pretty(&report)?)
        .with_context(|| format!("failed to write {}", report_path.display()))?;

    let latest_path = repo_root
        .join(".cache/release-gate")
        .join(target.slug())
        .join("latest.json");
    fs::copy(&report_path, &latest_path)
        .with_context(|| format!("failed to update {}", latest_path.display()))?;

    match result {
        Ok(()) => {
            println!(
                "release gate passed for {}: {}",
                target.slug(),
                report_path.display()
            );
            Ok(())
        }
        Err(error) => bail!(
            "release gate failed for {}: {error:#}. report: {}",
            target.slug(),
            report_path.display()
        ),
    }
}

fn run_one_player_playable_gate(
    repo_root: &Path,
    rom_path: &Path,
    manifest: &AssetManifest,
    report: &mut ReleaseGateReport,
) -> Result<()> {
    let rom_bytes = fs::read(rom_path)
        .with_context(|| format!("failed to read ROM from {}", rom_path.display()))?;

    for &scenario in NATIVE_GAMEPLAY_SMOKE_SCENARIOS {
        ensure_gameplay_level_layouts(rom_path, manifest, scenario)?;
        let step_name = format!("native_gameplay:{}", scenario.slug());
        run_release_step(report, &step_name, || {
            verify_native_gameplay_scenario(repo_root, manifest, scenario)?;
            Ok(format!(
                "native gameplay smoke passed for {}",
                scenario.slug()
            ))
        })?;
    }

    for &scenario in STATIC_NATIVE_GAMEPLAY_COMPARE_SCENARIOS {
        ensure_gameplay_level_layouts(rom_path, manifest, scenario)?;
        let step_name = format!("gameplay_compare:{}", scenario.slug());
        run_release_step(report, &step_name, || {
            compare_native_gameplay(repo_root, rom_path, manifest, scenario)?;
            Ok(format!(
                "native gameplay compare passed for {}",
                scenario.slug()
            ))
        })?;
    }

    for &scenario in FULL_BOOT_GAMEPLAY_VERIFY_SCENARIOS {
        let step_name = format!("gameplay_verify:{}", scenario.slug());
        run_release_step(report, &step_name, || {
            verify_gameplay_scenario(repo_root, &rom_bytes, scenario)?;
            Ok(format!(
                "full gameplay verification passed for {}",
                scenario.slug()
            ))
        })?;
    }

    for (name, args) in [
        (
            "wildside-core:slope-ground",
            &["test", "-q", "-p", "wildside-core", "--", "slope_ground_"][..],
        ),
        (
            "wildside-core:slope-seams",
            &[
                "test",
                "-q",
                "-p",
                "wildside-core",
                "walking_on_slope_up_y_tracks_diagonal_every_frame",
            ][..],
        ),
        (
            "wildside-core:slope-seams-down",
            &[
                "test",
                "-q",
                "-p",
                "wildside-core",
                "walking_on_slope_down_y_tracks_diagonal_every_frame",
            ][..],
        ),
        (
            "wildside-core:slope-solid-transition",
            &[
                "test",
                "-q",
                "-p",
                "wildside-core",
                "solid_to_slope_up_transition_stays_grounded",
            ][..],
        ),
        (
            "wildside-core:slope-grounded-seam",
            &[
                "test",
                "-q",
                "-p",
                "wildside-core",
                "grounded_slope_to_solid_seam_keeps_level_one_runner_grounded",
            ][..],
        ),
        (
            "wildside-core:rubber-block",
            &["test", "-q", "-p", "wildside-core", "--", "rubber_block_"][..],
        ),
        (
            "wildside-core:metal-bounce",
            &[
                "test",
                "-q",
                "-p",
                "wildside-core",
                "iron_knight_landing_on_metal_block_bounces_without_destroying_it",
            ][..],
        ),
        (
            "wildside-core:highwater-barrier",
            &[
                "test",
                "-q",
                "-p",
                "wildside-core",
                "highwater_pass_1_berzerker_live_charge_from_spawn_clears_first_barrier_loc_eae4_f02a",
            ][..],
        ),
        (
            "wildside-cli:highwater-shooter-render",
            &[
                "test",
                "-q",
                "-p",
                "wildside-cli",
                "highwater_berzerker_live_charge_from_spawn_renders_spikes_and_clears_barrier_loc_ef62_f02a",
            ][..],
        ),
        (
            "wildside-core:rocktank-shooting",
            &[
                "test",
                "-q",
                "-p",
                "wildside-core",
                "blue_lake_woods_2_live_rock_tank_shooting_spawns_tank_bullet_loc_3b730",
            ][..],
        ),
        (
            "wildside-cli:rocktank-bullet-render",
            &[
                "test",
                "-q",
                "-p",
                "wildside-cli",
                "blue_lake_woods_2_live_tank_bullet_renders_rom_child_sprite_loc_3b730",
            ][..],
        ),
        (
            "wildside-render:rocktank-palette",
            &[
                "test",
                "-q",
                "-p",
                "wildside-render",
                "forest_enemy_palette_keeps_borrowed_background_slots_after_art_remap",
            ][..],
        ),
        (
            "wildside-core:drips-collision",
            &[
                "test",
                "-q",
                "-p",
                "wildside-core",
                "drips_drop_floor_collision_uses_object_bottom_loc_3345c",
            ][..],
        ),
        (
            "wildside-core:drips-pool",
            &[
                "test",
                "-q",
                "-p",
                "wildside-core",
                "drips_owned_drop_advances_pool_and_reseeds_parent_only_on_splash_loc_33488",
            ][..],
        ),
        (
            "wildside-cli:drips-render",
            &["test", "-q", "-p", "wildside-cli", "drips_"][..],
        ),
    ] {
        run_release_step(report, name, || run_cargo_step(repo_root, args))?;
    }

    let gameplay_levels = sweep::parse_level_range("0-104")?;
    run_release_step(report, "gameplay_sweep:0-104", || {
        let gameplay_report = sweep::run_sweep_with_options(
            repo_root,
            rom_path,
            manifest,
            &gameplay_levels,
            sweep::SweepTier::Gameplay,
            &sweep::SweepOptions::default(),
        )?;
        if sweep_report_has_failures(gameplay_report.failed, gameplay_report.errors) {
            bail!(
                "gameplay sweep reported {} failed levels and {} errors",
                gameplay_report.failed,
                gameplay_report.errors
            );
        }
        Ok(format!(
            "native gameplay self-consistency sweep passed on {} levels",
            gameplay_report.passed
        ))
    })?;

    Ok(())
}

fn run_one_player_accurate_gate(
    repo_root: &Path,
    rom_path: &Path,
    manifest: &AssetManifest,
    max_failures: usize,
    report: &mut ReleaseGateReport,
) -> Result<()> {
    let rom_bytes = fs::read(rom_path)
        .with_context(|| format!("failed to read ROM from {}", rom_path.display()))?;

    run_release_step(report, "native_bootstrap", || {
        verify_native_bootstrap(manifest)?;
        Ok("boot/title/native gameplay bootstrap matches expected flow".to_string())
    })?;

    for &scenario in STATIC_NATIVE_GAMEPLAY_COMPARE_SCENARIOS {
        ensure_gameplay_level_layouts(rom_path, manifest, scenario)?;
        let step_name = format!("gameplay_compare:{}", scenario.slug());
        run_release_step(report, &step_name, || {
            compare_native_gameplay(repo_root, rom_path, manifest, scenario)?;
            Ok(format!(
                "native gameplay compare passed for {}",
                scenario.slug()
            ))
        })?;
    }

    for &scenario in FULL_BOOT_GAMEPLAY_VERIFY_SCENARIOS {
        let step_name = format!("gameplay_verify:{}", scenario.slug());
        run_release_step(report, &step_name, || {
            verify_gameplay_scenario(repo_root, &rom_bytes, scenario)?;
            Ok(format!(
                "full gameplay verification passed for {}",
                scenario.slug()
            ))
        })?;
    }

    let visual_run_dir = repo_root
        .join(".cache/release-gate")
        .join(report.target.as_str())
        .join(report.timestamp.as_str())
        .join("source-port-visual");
    run_release_step(report, "source_port_frame_oracle:level_00_early", || {
        let result = sweep::run_source_port_visual_release_probe(
            repo_root,
            rom_path,
            manifest,
            &visual_run_dir,
        )?;
        assess_source_port_visual_evidence(&result)
    })?;

    for scenario in [
        Scenario::BootTitleStartLevel1,
        Scenario::BootTitleStartLevel1MoveRight5s,
    ] {
        let step_name = format!("frame_oracle:{}", scenario.slug());
        run_release_step(report, &step_name, || {
            verify_frames(repo_root, rom_path, &rom_bytes, manifest, scenario)?;
            Ok(format!(
                "ROM VDP emulator compare passed for {} (wildside-md vs jgenesis; not source-port renderer evidence)",
                scenario.slug()
            ))
        })?;
    }

    for scenario in [Scenario::BootTitleStartLevel1, Scenario::BootTitleIdle10s] {
        let step_name = format!("audio_oracle:{}", scenario.slug());
        run_release_step(report, &step_name, || {
            verify_audio_output(repo_root, rom_path, &rom_bytes, manifest, scenario)?;
            Ok(format!("audio compare passed for {}", scenario.slug()))
        })?;
    }

    let levels = sweep::parse_level_range("0-104")?;
    // Sweep parity: persist the requested breadth before the sweep runs
    // so both passing and failing gate branches surface `levels=N` in
    // their signal. Mirrors the state-sweep signal which already cites
    // breadth via `levels_requested` (or the fresh-report `levels`
    // array length) regardless of the sweep outcome.
    report.levels_requested = Some(levels.len());
    match sweep::run_sweep_with_options(
        repo_root,
        rom_path,
        manifest,
        &levels,
        sweep::SweepTier::State,
        &sweep::SweepOptions {
            warm_cache_only: false,
            max_failures,
            checksum: false,
        },
    ) {
        Ok(sweep_report) => {
            report.sweep_summary = Some(sweep_report.divergence_summary.clone());
            // Capture the wildside-core source hash the sweep ran against —
            // populated whether the sweep passed or failed so a failing
            // gate also self-cites the source it ran on. The signal layer
            // compares this against the live tree and surfaces drift.
            report.kid_core_hash.clone_from(&sweep_report.kid_core_hash);
            // Sweep parity: capture the dominant divergence field so the
            // gate report carries a per-field fingerprint alongside the
            // per-category `sweep_summary`. `field_summary` is sorted by
            // total_diffs desc, levels desc, name asc (see
            // `summarise_field_diffs` in sweep/mod.rs), so the first
            // entry is the deterministic winner. Skipped when the sweep
            // recorded zero diffs (no dominant field exists), so a
            // pristine gate report stays free of placeholder values.
            report.top_divergence_field =
                sweep_report
                    .field_summary
                    .first()
                    .map(|entry| TopDivergenceField {
                        field: entry.field.clone(),
                        total_diffs: entry.total_diffs,
                        // Persist breadth so the gate signal can render the
                        // dominant-field fingerprint as `name:diffs across
                        // N levels` — concentration shifts (e.g. 200 diffs
                        // on 5 levels vs 200 spread across 69) are
                        // otherwise hidden behind an unchanged top_field.
                        levels: Some(entry.levels),
                        // Persist the checkpoint-axis breadth alongside the
                        // level-axis one so the gate signal can render
                        // `across N levels in M checkpoints`. Two regressions
                        // can share the same level count while diverging at
                        // very different checkpoint sets; without the
                        // checkpoint count, the dominant-field fingerprint
                        // would render identically in both cases.
                        checkpoints: Some(entry.checkpoints.len()),
                    });
            if sweep_report_has_failures(sweep_report.failed, sweep_report.errors) {
                let detail = state_sweep_failed_detail(repo_root, &sweep_report);
                report.steps.push(ReleaseGateStep {
                    name: "state_sweep:0-104".to_string(),
                    status: ReleaseGateStepStatus::Failed,
                    detail: detail.clone(),
                });
                bail!("{detail}");
            }
            report.steps.push(ReleaseGateStep {
                name: "state_sweep:0-104".to_string(),
                status: ReleaseGateStepStatus::Passed,
                detail: state_sweep_passed_detail(repo_root, levels.len(), &sweep_report),
            });
        }
        Err(error) => {
            report.steps.push(ReleaseGateStep {
                name: "state_sweep:0-104".to_string(),
                status: ReleaseGateStepStatus::Failed,
                detail: format!("{error:#}"),
            });
            return Err(error);
        }
    }

    Ok(())
}

fn assess_source_port_visual_evidence(result: &sweep::LevelSweepResult) -> Result<String> {
    if result.level_id != 0x00 || result.slug != "release_visual_level_00_early" {
        bail!(
            "unexpected source-port visual probe identity: level={:#04x}, slug={}",
            result.level_id,
            result.slug
        );
    }
    if let Some(error) = &result.error {
        bail!("source-port visual probe error: {error}");
    }
    if !result.passed {
        bail!("source-port visual probe did not pass its sweep checks");
    }
    if result.total_checkpoints != SOURCE_PORT_VISUAL_CHECKPOINTS {
        bail!(
            "source-port visual probe produced {} checkpoints, expected {}",
            result.total_checkpoints,
            SOURCE_PORT_VISUAL_CHECKPOINTS
        );
    }
    if result.masked_checkpoints > 0 {
        bail!(
            "source-port visual probe left {} of {} checkpoints masked",
            result.masked_checkpoints,
            result.total_checkpoints
        );
    }
    let summary = &result.divergence_summary;
    if !result.state_diffs.is_empty()
        || summary.authoritative > 0
        || summary.advisory > 0
        || summary.accumulation > 0
        || summary.rng > 0
        || summary.oracle_limitation > 0
    {
        bail!(
            "source-port visual probe state diverged: authoritative={}, advisory={}, accumulation={}, rng={}, oracle_limitation={}",
            summary.authoritative,
            summary.advisory,
            summary.accumulation,
            summary.rng,
            summary.oracle_limitation
        );
    }

    let mae = result
        .mae
        .ok_or_else(|| anyhow!("source-port visual probe is missing frame MAE"))?;
    let pixel_mismatches = result
        .pixel_mismatches
        .ok_or_else(|| anyhow!("source-port visual probe is missing changed-pixel evidence"))?;
    if result.diff_images.len() != SOURCE_PORT_VISUAL_CHECKPOINTS {
        bail!(
            "source-port visual probe produced {} diff images for {} checkpoints",
            result.diff_images.len(),
            SOURCE_PORT_VISUAL_CHECKPOINTS
        );
    }
    if mae > SOURCE_PORT_VISUAL_MAX_MAE {
        bail!(
            "source-port visual probe worst MAE {mae:.3} exceeds {SOURCE_PORT_VISUAL_MAX_MAE:.3} baseline"
        );
    }
    if pixel_mismatches > SOURCE_PORT_VISUAL_MAX_PIXEL_MISMATCHES {
        bail!(
            "source-port visual probe {pixel_mismatches} changed pixels exceed {SOURCE_PORT_VISUAL_MAX_PIXEL_MISMATCHES} baseline"
        );
    }

    let total_checkpoints = result.total_checkpoints;
    let diff_dir = result
        .diff_images
        .first()
        .and_then(|path| Path::new(path).parent())
        .map_or_else(|| "unknown".to_string(), |path| path.display().to_string());
    Ok(format!(
        "{total_checkpoints} unmasked source-port checkpoints passed with exact state; worst MAE {mae:.3}, {pixel_mismatches} changed pixels (baselines {SOURCE_PORT_VISUAL_MAX_MAE:.3}/{SOURCE_PORT_VISUAL_MAX_PIXEL_MISMATCHES}; diffs={diff_dir})"
    ))
}

fn state_sweep_passed_detail(
    repo_root: &Path,
    levels_requested: usize,
    sweep_report: &sweep::SweepReport,
) -> String {
    format!(
        "all {} requested levels passed the state sweep (report={})",
        levels_requested,
        state_sweep_report_path(repo_root, sweep_report).display()
    )
}

fn state_sweep_failed_detail(repo_root: &Path, sweep_report: &sweep::SweepReport) -> String {
    format!(
        "state sweep reported {} failed levels and {} errors (report={})",
        sweep_report.failed,
        sweep_report.errors,
        state_sweep_report_path(repo_root, sweep_report).display()
    )
}

fn state_sweep_report_path(repo_root: &Path, sweep_report: &sweep::SweepReport) -> PathBuf {
    repo_root
        .join(".cache/sweep-runs")
        .join(&sweep_report.timestamp)
        .join("report.json")
}

fn sweep_report_has_failures(failed: usize, errors: usize) -> bool {
    failed > 0 || errors > 0
}

fn run_cargo_step(repo_root: &Path, args: &[&str]) -> Result<String> {
    let output = Command::new("cargo")
        .args(args)
        .current_dir(repo_root)
        .output()
        .with_context(|| format!("failed to run `cargo {}`", args.join(" ")))?;
    if output.status.success() {
        return Ok(format!("`cargo {}` passed", args.join(" ")));
    }
    let stderr = String::from_utf8_lossy(&output.stderr);
    let stdout = String::from_utf8_lossy(&output.stdout);
    let detail =
        last_nonempty_output_line(&stdout, &stderr).unwrap_or("command failed without output");
    bail!("`cargo {}` failed: {detail}", args.join(" "));
}

fn last_nonempty_output_line<'a>(stdout: &'a str, stderr: &'a str) -> Option<&'a str> {
    stderr
        .lines()
        .rev()
        .find(|line| !line.trim().is_empty())
        .or_else(|| stdout.lines().rev().find(|line| !line.trim().is_empty()))
}

fn run_release_step<F>(report: &mut ReleaseGateReport, name: &str, step_fn: F) -> Result<()>
where
    F: FnOnce() -> Result<String>,
{
    match step_fn() {
        Ok(detail) => {
            report.steps.push(ReleaseGateStep {
                name: name.to_string(),
                status: ReleaseGateStepStatus::Passed,
                detail,
            });
            Ok(())
        }
        Err(error) => {
            report.steps.push(ReleaseGateStep {
                name: name.to_string(),
                status: ReleaseGateStepStatus::Failed,
                detail: format!("{error:#}"),
            });
            Err(error)
        }
    }
}

fn unix_timestamp() -> String {
    use std::time::SystemTime;
    let since_epoch = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap_or_default();
    format!("{}", since_epoch.as_secs())
}

fn release_gate_report_paths(
    repo_root: &Path,
    target: ReleaseGateTarget,
    timestamp: &str,
) -> (PathBuf, PathBuf) {
    let run_dir = repo_root
        .join(".cache/release-gate")
        .join(target.slug())
        .join(timestamp);
    let report_path = run_dir.join("report.json");
    (run_dir, report_path)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use wildside_data::AssetManifest;

    fn sweep_report(timestamp: &str, failed: usize, errors: usize) -> sweep::SweepReport {
        sweep::SweepReport {
            timestamp: timestamp.to_owned(),
            kid_core_hash: None,
            levels: Vec::new(),
            passed: 69,
            unverified: 0,
            failed,
            errors,
            divergence_summary: sweep::DivergenceSummary::default(),
            field_summary: Vec::new(),
            release_gate_focus: Vec::new(),
        }
    }

    fn source_port_visual_result() -> sweep::LevelSweepResult {
        sweep::LevelSweepResult {
            level_id: 0x00,
            slug: "release_visual_level_00_early".to_string(),
            tier: sweep::SweepTier::Full,
            passed: true,
            state_diffs: Vec::new(),
            mae: Some(0.0),
            pixel_mismatches: Some(0),
            diff_images: (0..4).map(|index| format!("diff_{index}.png")).collect(),
            error: None,
            divergence_summary: sweep::DivergenceSummary::default(),
            checksum_mismatch: None,
            total_checkpoints: 4,
            masked_checkpoints: 0,
        }
    }

    fn empty_manifest(root: &Path) -> AssetManifest {
        AssetManifest {
            generated_bundle_version: 0,
            rom_sha1: String::new(),
            upstream_commit: String::new(),
            source_root: root.to_path_buf(),
            output_root: root.to_path_buf(),
            map_order: Vec::new(),
            levels: Vec::new(),
            constants: Vec::new(),
            ram_variables: Vec::new(),
            enemy_sources: Vec::new(),
            theme_assets: Vec::new(),
            scene_assets: Vec::new(),
            animation_assets: Vec::new(),
            sound_assets: Vec::new(),
            ingame_assets: Vec::new(),
            decoded_levels: Vec::new(),
        }
    }

    #[test]
    fn state_sweep_step_details_cite_persisted_report_path() {
        let repo_root = Path::new("/tmp/kc");
        let report = sweep_report("1776793954", 0, 0);

        assert_eq!(
            state_sweep_passed_detail(repo_root, 69, &report),
            "all 69 requested levels passed the state sweep (report=/tmp/kc/.cache/sweep-runs/1776793954/report.json)"
        );

        let report = sweep_report("1776794000", 2, 1);
        assert_eq!(
            state_sweep_failed_detail(repo_root, &report),
            "state sweep reported 2 failed levels and 1 errors (report=/tmp/kc/.cache/sweep-runs/1776794000/report.json)"
        );
    }

    #[test]
    fn source_port_visual_evidence_accepts_complete_baseline() {
        let detail = assess_source_port_visual_evidence(&source_port_visual_result())
            .expect("complete visual evidence should pass");

        assert!(detail.contains("4 unmasked source-port checkpoints"));
        assert!(detail.contains("worst MAE 0.000"));
        assert!(detail.contains("0 changed pixels"));

        let mut exact_boundary = source_port_visual_result();
        exact_boundary.mae = Some(SOURCE_PORT_VISUAL_MAX_MAE);
        assess_source_port_visual_evidence(&exact_boundary)
            .expect("the versioned MAE boundary should pass");
    }

    #[test]
    fn source_port_visual_evidence_fails_closed_when_incomplete() {
        let mut result = source_port_visual_result();
        result.mae = None;
        assert!(
            assess_source_port_visual_evidence(&result)
                .expect_err("missing MAE should fail")
                .to_string()
                .contains("missing frame MAE")
        );

        let mut result = source_port_visual_result();
        result.masked_checkpoints = 1;
        assert!(
            assess_source_port_visual_evidence(&result)
                .expect_err("masked checkpoint should fail")
                .to_string()
                .contains("1 of 4 checkpoints masked")
        );

        let mut result = source_port_visual_result();
        result.diff_images.pop();
        assert!(
            assess_source_port_visual_evidence(&result)
                .expect_err("missing diff artefact should fail")
                .to_string()
                .contains("3 diff images for 4 checkpoints")
        );
    }

    #[test]
    fn source_port_visual_evidence_rejects_state_or_pixel_regressions() {
        for (level_id, slug) in [
            (0x01, "release_visual_level_00_early"),
            (0x00, "unexpected_visual_probe"),
        ] {
            let mut result = source_port_visual_result();
            result.level_id = level_id;
            result.slug = slug.to_string();
            assert!(
                assess_source_port_visual_evidence(&result)
                    .expect_err("wrong probe identity should fail")
                    .to_string()
                    .contains("unexpected source-port visual probe identity")
            );
        }

        let mut result = source_port_visual_result();
        result.state_diffs.push(sweep::StateDiff {
            checkpoint: "frame_030".to_string(),
            field: "kid_x_pos".to_string(),
            native_value: "1".to_string(),
            oracle_value: "2".to_string(),
            category: sweep::DivergenceCategory::Authoritative,
        });
        assert!(
            assess_source_port_visual_evidence(&result)
                .expect_err("detailed state divergence should fail")
                .to_string()
                .contains("state diverged")
        );

        let mut result = source_port_visual_result();
        result.divergence_summary.authoritative = 1;
        assert!(
            assess_source_port_visual_evidence(&result)
                .expect_err("state divergence should fail")
                .to_string()
                .contains("authoritative=1")
        );

        for category in [
            sweep::DivergenceCategory::Advisory,
            sweep::DivergenceCategory::Accumulation,
            sweep::DivergenceCategory::Rng,
            sweep::DivergenceCategory::OracleLimitation,
        ] {
            let mut result = source_port_visual_result();
            match category {
                sweep::DivergenceCategory::Authoritative => unreachable!(),
                sweep::DivergenceCategory::Advisory => {
                    result.divergence_summary.advisory = 1;
                }
                sweep::DivergenceCategory::Accumulation => {
                    result.divergence_summary.accumulation = 1;
                }
                sweep::DivergenceCategory::Rng => {
                    result.divergence_summary.rng = 1;
                }
                sweep::DivergenceCategory::OracleLimitation => {
                    result.divergence_summary.oracle_limitation = 1;
                }
            }
            assert!(
                assess_source_port_visual_evidence(&result)
                    .expect_err("every state divergence category should fail")
                    .to_string()
                    .contains("state diverged"),
                "category {category:?} should be rejected"
            );
        }

        let mut result = source_port_visual_result();
        result.mae = Some(0.001);
        assert!(
            assess_source_port_visual_evidence(&result)
                .expect_err("MAE regression should fail")
                .to_string()
                .contains("exceeds 0.000 baseline")
        );

        let mut result = source_port_visual_result();
        result.pixel_mismatches = Some(1);
        assert!(
            assess_source_port_visual_evidence(&result)
                .expect_err("changed-pixel regression should fail")
                .to_string()
                .contains("exceed 0 baseline")
        );
    }

    #[test]
    fn sweep_report_has_failures_when_failed_or_error_counts_are_nonzero() {
        assert!(!sweep_report_has_failures(0, 0));
        assert!(sweep_report_has_failures(1, 0));
        assert!(sweep_report_has_failures(0, 1));
        assert!(sweep_report_has_failures(1, 1));
    }

    #[test]
    fn run_release_step_records_pass_and_failure_details() {
        let mut report = ReleaseGateReport {
            timestamp: "1776793954".to_string(),
            target: ReleaseGateTarget::OnePlayerPlayable.slug().to_string(),
            passed: false,
            steps: Vec::new(),
            sweep_summary: None,
            kid_core_hash: None,
            levels_requested: None,
            top_divergence_field: None,
        };

        run_release_step(&mut report, "pass-step", || Ok("step passed".to_string()))
            .expect("passing step should succeed");
        let error = run_release_step(&mut report, "fail-step", || anyhow::bail!("inner failure"))
            .expect_err("failing step should propagate the error");

        assert_eq!(error.to_string(), "inner failure");
        assert_eq!(report.steps.len(), 2);
        assert_eq!(report.steps[0].name, "pass-step");
        assert!(matches!(
            report.steps[0].status,
            ReleaseGateStepStatus::Passed
        ));
        assert_eq!(report.steps[0].detail, "step passed");
        assert_eq!(report.steps[1].name, "fail-step");
        assert!(matches!(
            report.steps[1].status,
            ReleaseGateStepStatus::Failed
        ));
        assert_eq!(report.steps[1].detail, "inner failure");
    }

    #[test]
    fn run_cargo_step_reports_success_and_last_error_line() {
        let repo_root = Path::new(env!("CARGO_MANIFEST_DIR"));

        assert_eq!(
            run_cargo_step(repo_root, &["--version"]).expect("cargo --version should pass"),
            "`cargo --version` passed"
        );

        let error = run_cargo_step(repo_root, &["__wildside_missing_subcommand__"])
            .expect_err("unknown cargo subcommand should fail")
            .to_string();
        assert!(error.starts_with("`cargo __wildside_missing_subcommand__` failed:"));
        assert!(
            error.len() > "`cargo __wildside_missing_subcommand__` failed:".len(),
            "unexpected cargo error detail: {error}"
        );
    }

    #[test]
    fn last_nonempty_output_line_prefers_stderr_and_skips_blank_lines() {
        assert_eq!(
            last_nonempty_output_line("stdout detail\n\n", "\nstderr detail\n\n"),
            Some("stderr detail")
        );
        assert_eq!(
            last_nonempty_output_line("stdout detail\n\n", "\n \t\n"),
            Some("stdout detail")
        );
        assert_eq!(last_nonempty_output_line("\n", " \n\t"), None);
    }

    #[test]
    fn unix_timestamp_is_decimal_epoch_seconds() {
        let timestamp = unix_timestamp();

        assert!(!timestamp.is_empty());
        assert!(timestamp.bytes().all(|byte| byte.is_ascii_digit()));
        assert!(
            timestamp.parse::<u64>().expect("timestamp should parse") > 1_700_000_000,
            "timestamp should be a modern Unix epoch second"
        );
    }

    #[test]
    fn run_release_gate_writes_failed_latest_report_when_rom_is_missing() {
        let temp = TempDir::new().expect("tempdir");
        let manifest = empty_manifest(temp.path());
        let missing_rom = temp.path().join("missing.bin");

        for target in [
            ReleaseGateTarget::OnePlayerPlayable,
            ReleaseGateTarget::OnePlayerAccurate,
        ] {
            let error = run_release_gate(temp.path(), &missing_rom, &manifest, target, 1)
                .expect_err("missing ROM should fail the release gate")
                .to_string();

            assert!(error.contains(&format!("release gate failed for {}", target.slug())));
            assert!(error.contains("failed to read ROM"));
            let latest_path = temp
                .path()
                .join(".cache/release-gate")
                .join(target.slug())
                .join("latest.json");
            let latest = fs::read_to_string(&latest_path).expect("latest report should be written");
            let report: ReleaseGateReport =
                serde_json::from_str(&latest).expect("latest report should deserialize");
            assert_eq!(report.target, target.slug());
            assert!(!report.passed);
            assert!(report.steps.is_empty());
        }
    }

    #[test]
    fn release_gate_report_path_uses_report_timestamp() {
        let repo_root = Path::new("/tmp/kc");

        let (run_dir, report_path) = release_gate_report_paths(
            repo_root,
            ReleaseGateTarget::OnePlayerAccurate,
            "1776796430",
        );

        assert_eq!(
            run_dir,
            PathBuf::from("/tmp/kc/.cache/release-gate/1p-accurate/1776796430")
        );
        assert_eq!(
            report_path,
            PathBuf::from("/tmp/kc/.cache/release-gate/1p-accurate/1776796430/report.json")
        );
    }

    #[test]
    fn death_respawn_routes_use_full_boot_gameplay_verification() {
        assert!(!NATIVE_GAMEPLAY_SMOKE_SCENARIOS.contains(&GameplayScenario::DeathRespawnLevel1));
        assert!(
            !NATIVE_GAMEPLAY_SMOKE_SCENARIOS
                .contains(&GameplayScenario::DeathRespawnStartResumeLevel1)
        );
        assert!(
            !STATIC_NATIVE_GAMEPLAY_COMPARE_SCENARIOS
                .contains(&GameplayScenario::DeathRespawnLevel1)
        );
        assert!(
            !STATIC_NATIVE_GAMEPLAY_COMPARE_SCENARIOS
                .contains(&GameplayScenario::DeathRespawnStartResumeLevel1)
        );
        assert!(
            FULL_BOOT_GAMEPLAY_VERIFY_SCENARIOS.contains(&GameplayScenario::DeathRespawnLevel1)
        );
        assert!(
            FULL_BOOT_GAMEPLAY_VERIFY_SCENARIOS
                .contains(&GameplayScenario::DeathRespawnStartResumeLevel1)
        );
    }

    #[test]
    fn progress_transition_route_uses_full_boot_gameplay_verification() {
        assert!(
            NATIVE_GAMEPLAY_SMOKE_SCENARIOS.contains(&GameplayScenario::ProgressTwoTransitions)
        );
        assert!(
            !STATIC_NATIVE_GAMEPLAY_COMPARE_SCENARIOS
                .contains(&GameplayScenario::ProgressTwoTransitions)
        );
        assert!(
            FULL_BOOT_GAMEPLAY_VERIFY_SCENARIOS.contains(&GameplayScenario::ProgressTwoTransitions)
        );
    }
}
