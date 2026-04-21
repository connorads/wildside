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
}

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
    crate::autonomy::refresh_source_port_frontier_handoff_after_release_gate(repo_root, 1)
        .context("failed to refresh source-port frontier handoff from release-gate evidence")?;

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
    for scenario in [
        GameplayScenario::PauseResumeLevel1,
        GameplayScenario::ProgressTwoTransitions,
        GameplayScenario::DeathRespawnLevel1,
    ] {
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

    for scenario in [
        GameplayScenario::PauseResumeLevel1,
        GameplayScenario::DeathRespawnLevel1,
    ] {
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

    for (name, args) in [
        (
            "kid-core:slope-ground",
            &["test", "-q", "-p", "kid-core", "--", "slope_ground_"][..],
        ),
        (
            "kid-core:slope-seams",
            &[
                "test",
                "-q",
                "-p",
                "kid-core",
                "walking_on_slope_up_y_tracks_diagonal_every_frame",
            ][..],
        ),
        (
            "kid-core:slope-seams-down",
            &[
                "test",
                "-q",
                "-p",
                "kid-core",
                "walking_on_slope_down_y_tracks_diagonal_every_frame",
            ][..],
        ),
        (
            "kid-core:slope-solid-transition",
            &[
                "test",
                "-q",
                "-p",
                "kid-core",
                "solid_to_slope_up_transition_stays_grounded",
            ][..],
        ),
        (
            "kid-core:slope-grounded-seam",
            &[
                "test",
                "-q",
                "-p",
                "kid-core",
                "grounded_slope_to_solid_seam_keeps_level_one_runner_grounded",
            ][..],
        ),
        (
            "kid-core:rubber-block",
            &["test", "-q", "-p", "kid-core", "--", "rubber_block_"][..],
        ),
        (
            "kid-core:metal-bounce",
            &[
                "test",
                "-q",
                "-p",
                "kid-core",
                "iron_knight_landing_on_metal_block_bounces_without_destroying_it",
            ][..],
        ),
    ] {
        run_release_step(report, name, || run_cargo_step(repo_root, args))?;
    }

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

    for scenario in [
        GameplayScenario::PauseResumeLevel1,
        GameplayScenario::DeathRespawnLevel1,
        GameplayScenario::ProgressTwoTransitions,
    ] {
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

    for scenario in [
        Scenario::BootTitleStartLevel1,
        Scenario::BootTitleStartLevel1MoveRight5s,
    ] {
        let step_name = format!("frame_oracle:{}", scenario.slug());
        run_release_step(report, &step_name, || {
            verify_frames(repo_root, rom_path, &rom_bytes, manifest, scenario)?;
            Ok(format!("frame compare passed for {}", scenario.slug()))
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
    match sweep::run_sweep_with_options(
        repo_root,
        rom_path,
        manifest,
        &levels,
        sweep::SweepTier::State,
        &sweep::SweepOptions {
            warm_cache_only: false,
            max_failures,
        },
    ) {
        Ok(sweep_report) => {
            report.sweep_summary = Some(sweep_report.divergence_summary.clone());
            if sweep_report.failed > 0 || sweep_report.errors > 0 {
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
    let detail = stderr
        .lines()
        .rev()
        .find(|line| !line.trim().is_empty())
        .or_else(|| stdout.lines().rev().find(|line| !line.trim().is_empty()))
        .unwrap_or("command failed without output");
    bail!("`cargo {}` failed: {detail}", args.join(" "));
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

    fn sweep_report(timestamp: &str, failed: usize, errors: usize) -> sweep::SweepReport {
        sweep::SweepReport {
            timestamp: timestamp.to_owned(),
            kid_core_hash: None,
            levels: Vec::new(),
            passed: 69,
            failed,
            errors,
            divergence_summary: sweep::DivergenceSummary::default(),
            field_summary: Vec::new(),
            release_gate_focus: Vec::new(),
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
}
