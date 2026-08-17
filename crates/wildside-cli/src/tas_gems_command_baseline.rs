//! Ratchet gate for GEMS command parity (`gems_command_parity_v1`, #435) on
//! the full published-route TAS run, cloned from `tas_gems_baseline`.
//!
//! This is the authoritative audio gate per the #416 supersession decision:
//! unlike the frozen signature witness, the ratchet is WANTED here — fewer
//! command mismatches and a later first mismatch both rank `Better` (exit 3
//! Improvement prompts a baseline update). The two skew-reconciliation
//! direction counts are pinned exactly as evidence: a real one-frame native
//! scheduling shift converts reconciled pairs into exact pairs (or flips
//! their direction), so any change classifies Regression-for-review rather
//! than silently passing.

use anyhow::{Context, Result, bail};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

use crate::tas_pixel_baseline::{
    Classification, IMPROVEMENT_EXIT_CODE, MetricDirection, MetricRow, frontier_recording,
    metric_lower_better, pareto_classification, print_metric_table, splice_recording_block,
};

const FRONTIER_KEY: &str = "full_expected_gems_command_comparison";

/// Field order mirrors the on-disk block; `--update` serialises this struct
/// directly into it.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
struct GemsCommandBaseline {
    compared_frames: u64,
    native_command_count: u64,
    oracle_command_count: u64,
    matched_exact: u64,
    reconciled_native_earlier: u64,
    reconciled_oracle_earlier: u64,
    mismatch_count: u64,
    first_mismatch_frame: Option<u64>,
    mismatch_digest: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
struct ReportTimedCommand {
    frame_index: u64,
}

#[derive(Debug, Clone, Deserialize)]
struct ReportCommandMismatch {
    native: Option<ReportTimedCommand>,
    oracle: Option<ReportTimedCommand>,
}

/// The `comparison` payload of the report's `gems_command_comparison`
/// block. Extra fields (full streams, mismatch list bodies) are ignored.
#[derive(Debug, Clone, Deserialize)]
struct ReportCommandComparison {
    compared_frames: u64,
    native_command_count: u64,
    oracle_command_count: u64,
    matched_exact: u64,
    reconciled_native_earlier: u64,
    reconciled_oracle_earlier: u64,
    mismatch_count: u64,
    first_mismatch: Option<ReportCommandMismatch>,
    mismatch_digest: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
struct ReportCommandBlock {
    projection: String,
    comparison: ReportCommandComparison,
}

#[derive(Debug, Clone, Deserialize)]
struct VerifyTasReport {
    gems_command_comparison: Option<ReportCommandBlock>,
}

impl ReportCommandComparison {
    fn baseline(&self) -> GemsCommandBaseline {
        GemsCommandBaseline {
            compared_frames: self.compared_frames,
            native_command_count: self.native_command_count,
            oracle_command_count: self.oracle_command_count,
            matched_exact: self.matched_exact,
            reconciled_native_earlier: self.reconciled_native_earlier,
            reconciled_oracle_earlier: self.reconciled_oracle_earlier,
            mismatch_count: self.mismatch_count,
            first_mismatch_frame: self.first_mismatch.as_ref().map(first_mismatch_frame),
            mismatch_digest: self.mismatch_digest.clone(),
        }
    }
}

/// Deterministic frame for a mismatch: the earliest frame either side
/// attributes it to (one side may be absent for insertions).
fn first_mismatch_frame(mismatch: &ReportCommandMismatch) -> u64 {
    match (&mismatch.native, &mismatch.oracle) {
        (Some(native), Some(oracle)) => native.frame_index.min(oracle.frame_index),
        (Some(native), None) => native.frame_index,
        (None, Some(oracle)) => oracle.frame_index,
        (None, None) => 0,
    }
}

/// `None` (no mismatch anywhere) is the best possible first-mismatch frame.
fn metric_first_mismatch(expected: Option<u64>, actual: Option<u64>) -> MetricRow {
    let display =
        |value: Option<u64>| value.map_or_else(|| "none".to_owned(), |frame| frame.to_string());
    let direction = match (expected, actual) {
        (None, None) => MetricDirection::Same,
        (Some(_), None) => MetricDirection::Better,
        (None, Some(_)) => MetricDirection::Worse,
        (Some(expected), Some(actual)) => match actual.cmp(&expected) {
            std::cmp::Ordering::Greater => MetricDirection::Better,
            std::cmp::Ordering::Equal => MetricDirection::Same,
            std::cmp::Ordering::Less => MetricDirection::Worse,
        },
    };
    MetricRow {
        name: "first_mismatch_frame".into(),
        expected: display(expected),
        actual: display(actual),
        direction,
    }
}

/// Ranked metrics: fewer command mismatches and a later first mismatch.
/// Everything else (command counts, matched_exact, the two reconciliation
/// direction counts, digest) is evidence pinned by Match's struct equality:
/// any change with equal ranked metrics is Regression-for-review.
fn classify(
    expected: &GemsCommandBaseline,
    actual: &GemsCommandBaseline,
) -> (Classification, Vec<MetricRow>) {
    let rows = vec![
        metric_lower_better(
            "mismatch_count",
            expected.mismatch_count,
            actual.mismatch_count,
        ),
        metric_first_mismatch(expected.first_mismatch_frame, actual.first_mismatch_frame),
    ];
    let classification = pareto_classification(expected == actual, &rows);
    (classification, rows)
}

/// Hard coverage gate: the comparison must span every frame of the run.
fn enforce_coverage(block: &ReportCommandBlock, total_frames: u64) -> Result<()> {
    if block.projection != crate::gems_ring::GEMS_COMMAND_PARITY_PROJECTION_NAME {
        bail!(
            "GEMS command projection contract changed: expected {}, got {}",
            crate::gems_ring::GEMS_COMMAND_PARITY_PROJECTION_NAME,
            block.projection
        );
    }
    if block.comparison.compared_frames != total_frames {
        bail!(
            "GEMS command comparison does not span the run (compared_frames={} expected {total_frames})",
            block.comparison.compared_frames
        );
    }
    Ok(())
}

fn load_report_block(report_path: &Path) -> Result<ReportCommandBlock> {
    let content = fs::read_to_string(report_path)
        .with_context(|| format!("failed to read {}", report_path.display()))?;
    let report: VerifyTasReport = serde_json::from_str(&content)
        .with_context(|| format!("failed to parse {}", report_path.display()))?;
    report.gems_command_comparison.with_context(|| {
        format!(
            "{} has no gems_command_comparison block; run verify-tas with --compare-gems-commands",
            report_path.display()
        )
    })
}

/// Returns the process exit code: 0 match, 1 regression,
/// [`IMPROVEMENT_EXIT_CODE`] improvement. Coverage/contract failures return
/// `Err` (exit 1 via anyhow) independent of classification.
pub(crate) fn run_tas_gems_command_baseline(
    report_path: &Path,
    frontier_path: &Path,
    slug: &str,
    update: bool,
    force: bool,
) -> Result<i32> {
    let block = load_report_block(report_path)?;
    let frontier_content = fs::read_to_string(frontier_path)
        .with_context(|| format!("failed to read {}", frontier_path.display()))?;
    let frontier: serde_json::Value = serde_json::from_str(&frontier_content)
        .with_context(|| format!("failed to parse {}", frontier_path.display()))?;
    let recording = frontier_recording(&frontier, frontier_path, slug)?;
    let total_frames = recording
        .get("total_frames")
        .and_then(serde_json::Value::as_u64)
        .with_context(|| format!("recording {slug} has no total_frames"))?;
    enforce_coverage(&block, total_frames)?;

    let expected: Option<GemsCommandBaseline> = recording
        .get(FRONTIER_KEY)
        .map(|value| {
            serde_json::from_value(value.clone()).with_context(|| {
                format!(
                    "failed to parse {FRONTIER_KEY} for {slug} in {}",
                    frontier_path.display()
                )
            })
        })
        .transpose()?;
    let actual = block.comparison.baseline();
    if force && !update {
        anyhow::bail!(
            "tas-gems-command-baseline: --force is only meaningful together with --update"
        );
    }
    let Some(expected) = expected else {
        // Creating a missing baseline block is not a regression; the guard
        // below covers only the classified path.
        if update {
            let spliced = splice_recording_block(&frontier_content, slug, FRONTIER_KEY, &actual)?;
            fs::write(frontier_path, &spliced)
                .with_context(|| format!("failed to write {}", frontier_path.display()))?;
            println!(
                "tas-gems-command-baseline: created {FRONTIER_KEY} for {slug} in {}",
                frontier_path.display()
            );
            return Ok(0);
        }
        bail!(
            "{} recording {slug} has no {FRONTIER_KEY} block; baseline it with --update",
            frontier_path.display()
        );
    };
    let (classification, rows) = classify(&expected, &actual);
    crate::tas_pixel_baseline::guard_baseline_update(
        "tas-gems-command-baseline",
        classification,
        &rows,
        update,
        force,
    )?;

    if update {
        let spliced = splice_recording_block(&frontier_content, slug, FRONTIER_KEY, &actual)?;
        if spliced == frontier_content {
            println!(
                "tas-gems-command-baseline: {} already matches the report; nothing to update",
                frontier_path.display()
            );
        } else {
            if classification == Classification::Regression {
                println!(
                    "tas-gems-command-baseline: WARNING — updating the baseline to a REGRESSED state"
                );
            }
            fs::write(frontier_path, &spliced)
                .with_context(|| format!("failed to write {}", frontier_path.display()))?;
            println!(
                "tas-gems-command-baseline: updated {FRONTIER_KEY} for {slug} in {}",
                frontier_path.display()
            );
            print_metric_table(&rows);
        }
        return Ok(0);
    }

    match classification {
        Classification::Match => {
            println!(
                "tas-gems-command-baseline: ok ({} command mismatches, first at {} — matches {})",
                actual.mismatch_count,
                actual
                    .first_mismatch_frame
                    .map_or_else(|| "none".to_owned(), |frame| frame.to_string()),
                frontier_path.display()
            );
            Ok(0)
        }
        Classification::Improvement => {
            println!("tas-gems-command-baseline: FAIL — IMPROVEMENT (ratchet not yet updated)");
            println!(
                "  GEMS command parity moved FORWARD; the committed baseline in {} is stale.",
                frontier_path.display()
            );
            print_metric_table(&rows);
            println!("  Review the delta, then ratchet the baseline with:");
            println!(
                "  cargo run --release -q -p wildside-cli -- tas-gems-command-baseline --report {} --frontier {} --slug {} --update",
                report_path.display(),
                frontier_path.display(),
                slug
            );
            Ok(IMPROVEMENT_EXIT_CODE)
        }
        Classification::Regression => {
            println!("tas-gems-command-baseline: FAIL — REGRESSION");
            println!(
                "  GEMS command parity regressed against the committed baseline in {}.",
                frontier_path.display()
            );
            print_metric_table(&rows);
            if actual.mismatch_digest != expected.mismatch_digest {
                println!(
                    "  mismatch_digest changed: {} -> {}",
                    expected.mismatch_digest.as_deref().unwrap_or("none"),
                    actual.mismatch_digest.as_deref().unwrap_or("none")
                );
            }
            if (
                actual.reconciled_native_earlier,
                actual.reconciled_oracle_earlier,
            ) != (
                expected.reconciled_native_earlier,
                expected.reconciled_oracle_earlier,
            ) {
                println!(
                    "  skew reconciliation changed: native-earlier {} -> {}, oracle-earlier {} -> {} (pinned exactly; a shift here is scheduling drift)",
                    expected.reconciled_native_earlier,
                    actual.reconciled_native_earlier,
                    expected.reconciled_oracle_earlier,
                    actual.reconciled_oracle_earlier
                );
            }
            Ok(1)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tas_pixel_baseline::reindent;

    fn baseline() -> GemsCommandBaseline {
        GemsCommandBaseline {
            compared_frames: 37_759,
            native_command_count: 414,
            oracle_command_count: 437,
            matched_exact: 40,
            reconciled_native_earlier: 3,
            reconciled_oracle_earlier: 1,
            mismatch_count: 419,
            first_mismatch_frame: Some(16),
            mismatch_digest: Some("aec5d5040dd4050565b0055501de47cd087e66e6".to_owned()),
        }
    }

    #[test]
    fn classify_identical_is_match() {
        let (classification, _) = classify(&baseline(), &baseline());
        assert_eq!(classification, Classification::Match);
    }

    #[test]
    fn classify_fewer_mismatches_and_later_first_is_improvement() {
        let mut actual = baseline();
        actual.mismatch_count = 100;
        actual.first_mismatch_frame = Some(900);
        actual.matched_exact = 300;
        actual.mismatch_digest = Some("0000000000000000000000000000000000000000".to_owned());
        let (classification, _) = classify(&baseline(), &actual);
        assert_eq!(classification, Classification::Improvement);
    }

    #[test]
    fn classify_zero_mismatches_is_improvement() {
        let mut actual = baseline();
        actual.mismatch_count = 0;
        actual.first_mismatch_frame = None;
        actual.mismatch_digest = None;
        actual.matched_exact = 410;
        let (classification, _) = classify(&baseline(), &actual);
        assert_eq!(classification, Classification::Improvement);
    }

    #[test]
    fn classify_grown_mismatch_count_is_regression() {
        let mut actual = baseline();
        actual.mismatch_count += 1;
        let (classification, _) = classify(&baseline(), &actual);
        assert_eq!(classification, Classification::Regression);
    }

    #[test]
    fn classify_skew_direction_shift_alone_is_regression() {
        // The masking case from adversarial review: a one-frame native
        // scheduling shift can flip reconciled pairs without touching the
        // ranked metrics. Pinned evidence must catch it.
        let mut actual = baseline();
        actual.reconciled_native_earlier = 1;
        actual.reconciled_oracle_earlier = 3;
        let (classification, rows) = classify(&baseline(), &actual);
        assert_eq!(classification, Classification::Regression);
        assert!(
            rows.iter()
                .all(|row| row.direction == MetricDirection::Same),
            "ranked metrics unchanged; equality pins the evidence"
        );
    }

    #[test]
    fn classify_matched_exact_change_alone_is_regression() {
        let mut actual = baseline();
        actual.matched_exact += 1;
        let (classification, _) = classify(&baseline(), &actual);
        assert_eq!(classification, Classification::Regression);
    }

    fn report_fixture() -> serde_json::Value {
        serde_json::json!({
            "gems_command_comparison": {
                "projection": "gems_command_parity_v1",
                "comparison": {
                    "compared_frames": 37_759,
                    "native_command_count": 414,
                    "oracle_command_count": 437,
                    "matched_exact": 40,
                    "reconciled_skew_commands": 4,
                    "reconciled_native_earlier": 3,
                    "reconciled_oracle_earlier": 1,
                    "mismatch_count": 419,
                    "first_mismatch": {
                        "native": {"frame_index": 36, "command": {"opcode": 11, "args": [1]}},
                        "oracle": {"frame_index": 16, "command": {"opcode": 11, "args": [1]}},
                    },
                    "mismatches": [],
                    "native_commands": [],
                    "oracle_commands": [],
                    "mismatch_digest": "aec5d5040dd4050565b0055501de47cd087e66e6",
                    "native_reinit_frames": 44,
                    "oracle_reinit_frames": 91,
                }
            }
        })
    }

    #[test]
    fn report_loader_extracts_baseline_with_min_first_mismatch_frame() {
        let parsed: VerifyTasReport =
            serde_json::from_value(report_fixture()).expect("extra fields ignored");
        let block = parsed.gems_command_comparison.expect("block present");
        enforce_coverage(&block, 37_759).expect("coverage complete");
        assert_eq!(block.comparison.baseline(), baseline());
    }

    #[test]
    fn coverage_rejects_partial_runs_and_foreign_projections() {
        let parsed: VerifyTasReport =
            serde_json::from_value(report_fixture()).expect("fixture parses");
        let block = parsed.gems_command_comparison.expect("block present");
        assert!(
            enforce_coverage(&block, 40_000)
                .expect_err("short run")
                .to_string()
                .contains("does not span the run")
        );
        let mut wrong = block.clone();
        wrong.projection = "gems_frame_signature_v1".to_owned();
        assert!(
            enforce_coverage(&wrong, 37_759)
                .expect_err("wrong projection")
                .to_string()
                .contains("projection contract")
        );
    }

    fn frontier_fixture() -> String {
        let block = reindent(
            &serde_json::to_string_pretty(&baseline()).expect("serialise baseline"),
            "      ",
        );
        format!(
            r#"{{
  "recordings": {{
    "any-percent-published": {{
      "scenario": "tests/data/tas/any-percent-published/scenario.json",
      "total_frames": 37759,
      "full_expected_gems_command_comparison": {block},
      "status": "verified"
    }}
  }}
}}
"#
        )
    }

    #[test]
    fn update_splices_and_subsequent_run_matches() {
        let dir = tempfile::tempdir().expect("tempdir");
        let frontier_path = dir.path().join("tas_frontier.json");
        fs::write(&frontier_path, frontier_fixture()).expect("write fixture");
        let report_path = dir.path().join("report.json");

        let mut report = report_fixture();
        report["gems_command_comparison"]["comparison"]["mismatch_count"] = serde_json::json!(100);
        report["gems_command_comparison"]["comparison"]["mismatch_digest"] =
            serde_json::json!("feedfacefeedfacefeedfacefeedfacefeedface");
        fs::write(
            &report_path,
            serde_json::to_vec_pretty(&report).expect("report bytes"),
        )
        .expect("write report");

        let code = run_tas_gems_command_baseline(
            &report_path,
            &frontier_path,
            "any-percent-published",
            false,
            false,
        )
        .expect("classification run");
        assert_eq!(code, IMPROVEMENT_EXIT_CODE);

        let code = run_tas_gems_command_baseline(
            &report_path,
            &frontier_path,
            "any-percent-published",
            true,
            false,
        )
        .expect("update run");
        assert_eq!(code, 0);
        let code = run_tas_gems_command_baseline(
            &report_path,
            &frontier_path,
            "any-percent-published",
            false,
            false,
        )
        .expect("post-update run");
        assert_eq!(code, 0, "updated baseline must classify as Match");
    }

    #[test]
    fn regression_exits_one() {
        let dir = tempfile::tempdir().expect("tempdir");
        let frontier_path = dir.path().join("tas_frontier.json");
        fs::write(&frontier_path, frontier_fixture()).expect("write fixture");
        let report_path = dir.path().join("report.json");

        let mut report = report_fixture();
        report["gems_command_comparison"]["comparison"]["mismatch_count"] = serde_json::json!(500);
        fs::write(
            &report_path,
            serde_json::to_vec_pretty(&report).expect("report bytes"),
        )
        .expect("write report");

        let code = run_tas_gems_command_baseline(
            &report_path,
            &frontier_path,
            "any-percent-published",
            false,
            false,
        )
        .expect("regression run");
        assert_eq!(code, 1);
    }
}
