use super::*;
use clap::ValueEnum;
use serde::{Deserialize, Serialize};
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
    let run_dir = repo_root
        .join(".cache/release-gate")
        .join(target.slug())
        .join(unix_timestamp());
    fs::create_dir_all(&run_dir)
        .with_context(|| format!("failed to create {}", run_dir.display()))?;

    let mut report = ReleaseGateReport {
        timestamp: unix_timestamp(),
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
    let report_path = run_dir.join("report.json");
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
            verify_frames(repo_root, rom_path, &rom_bytes, scenario)?;
            Ok(format!("frame compare passed for {}", scenario.slug()))
        })?;
    }

    for scenario in [Scenario::BootTitleStartLevel1, Scenario::BootTitleIdle10s] {
        let step_name = format!("audio_oracle:{}", scenario.slug());
        run_release_step(report, &step_name, || {
            verify_audio_output(repo_root, rom_path, &rom_bytes, scenario)?;
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
                report.steps.push(ReleaseGateStep {
                    name: "state_sweep:0-104".to_string(),
                    status: ReleaseGateStepStatus::Failed,
                    detail: format!(
                        "state sweep reported {} failed levels and {} errors",
                        sweep_report.failed, sweep_report.errors
                    ),
                });
                bail!(
                    "state sweep reported {} failed levels and {} errors",
                    sweep_report.failed,
                    sweep_report.errors
                );
            }
            report.steps.push(ReleaseGateStep {
                name: "state_sweep:0-104".to_string(),
                status: ReleaseGateStepStatus::Passed,
                detail: format!(
                    "all {} requested levels passed the state sweep",
                    levels.len()
                ),
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
