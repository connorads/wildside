//! Frozen regression witness for GEMS/Z80 audio-timing state on the full
//! published-route TAS run, cloned from the pixel ratchet
//! (`tas_pixel_baseline`).
//!
//! Compares a `verify-tas --compare-gems` report's `gems_frame_comparison`
//! block against the slug's `full_expected_gems_frame_comparison` baseline in
//! `tests/tas_frontier.json`, classifying the delta as Match / Improvement /
//! Regression. The per-frame signature frontier is FROZEN (#416): the only
//! metric that can rank `Better` is `mismatch_count` (down = better).
//! `first_mismatch_frame` is an asymmetric guard — moving it EARLIER blocks
//! as Regression (frames inside the defended span regressed) even when the
//! total count dropped, while moving it later is never rewarded.
//! Command-level audio correctness is owned by the command-parity gate
//! (`gems_command_parity_v1`, #435). On improvement the gate fails (exit 3)
//! and prints the exact `--update` command; `--update` splices only the GEMS
//! block into the frontier file.

use anyhow::{Context, Result, bail};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

use crate::tas_pixel_baseline::{
    Classification, IMPROVEMENT_EXIT_CODE, MetricDirection, MetricRow, frontier_recording,
    metric_lower_better, pareto_classification, print_metric_table, splice_recording_block,
};

const FRONTIER_GEMS_KEY: &str = "full_expected_gems_frame_comparison";

/// Field order mirrors the on-disk `full_expected_gems_frame_comparison`
/// block; `--update` serialises this struct directly into it.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
struct GemsBaseline {
    compared_frames: u64,
    mismatch_count: u64,
    oracle_limitation_count: u64,
    advisory_only_frames: u64,
    mismatch_digest: Option<String>,
    first_mismatch_frame: Option<u64>,
}

#[derive(Debug, Clone, Deserialize)]
#[cfg_attr(test, derive(Serialize))]
struct ReportFrameCounts {
    requested: u64,
    native: u64,
    oracle: u64,
}

#[derive(Debug, Clone, Deserialize)]
#[cfg_attr(test, derive(Serialize))]
struct ReportGemsFirstMismatch {
    frame_index: u64,
}

/// The `gems_frame_comparison` block of a verify-tas report. Extra fields
/// (per-field mismatch evidence, proves_* flags we don't gate on) are ignored
/// by serde's default unknown-field behaviour.
#[derive(Debug, Clone, Deserialize)]
#[cfg_attr(test, derive(Serialize))]
struct ReportGemsComparison {
    projection: String,
    frame_counts: ReportFrameCounts,
    start_frame: Option<u64>,
    compared_frames: u64,
    mismatch_count: u64,
    oracle_limitation_count: u64,
    advisory_only_frames: u64,
    mismatch_digest: Option<String>,
    first_mismatch: Option<ReportGemsFirstMismatch>,
}

impl ReportGemsComparison {
    fn baseline(&self) -> GemsBaseline {
        GemsBaseline {
            compared_frames: self.compared_frames,
            mismatch_count: self.mismatch_count,
            oracle_limitation_count: self.oracle_limitation_count,
            advisory_only_frames: self.advisory_only_frames,
            mismatch_digest: self.mismatch_digest.clone(),
            first_mismatch_frame: self
                .first_mismatch
                .as_ref()
                .map(|mismatch| mismatch.frame_index),
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
struct VerifyTasReport {
    gems_frame_comparison: Option<ReportGemsComparison>,
}

/// Asymmetric freeze guard (#416): an EARLIER first mismatch means frames
/// inside the defended span regressed and must block even when the total
/// mismatch count dropped, but a later first mismatch is never `Better` —
/// the frozen frontier pays no advancement reward.
fn metric_first_mismatch_guard(expected: Option<u64>, actual: Option<u64>) -> MetricRow {
    let display =
        |value: Option<u64>| value.map_or_else(|| "none".to_owned(), |frame| frame.to_string());
    let direction = match (expected, actual) {
        // A mismatch appearing where none existed is a regression; a
        // mismatch vanishing or moving later earns Same (mismatch_count
        // alone decides Improvement).
        (None, Some(_)) => MetricDirection::Worse,
        (Some(expected), Some(actual)) if actual < expected => MetricDirection::Worse,
        _ => MetricDirection::Same,
    };
    MetricRow {
        name: "first_mismatch_frame",
        expected: display(expected),
        actual: display(actual),
        direction,
    }
}

/// The only metric that can rank `Better` is `mismatch_count` (fewer
/// mismatched frames). The frontier is frozen (#416): `first_mismatch_frame`
/// is a guard that can only hold (`Same`) or block (`Worse`), and the
/// digest, limitation, and advisory counts are evidence that participates in
/// Match detection but never ranks. A run that only moves the first mismatch
/// later (or reshuffles content with identical totals) fails as Regression
/// for review rather than passing as an improvement.
fn classify(expected: &GemsBaseline, actual: &GemsBaseline) -> (Classification, Vec<MetricRow>) {
    let rows = vec![
        metric_lower_better(
            "mismatch_count",
            expected.mismatch_count,
            actual.mismatch_count,
        ),
        metric_first_mismatch_guard(expected.first_mismatch_frame, actual.first_mismatch_frame),
    ];
    let classification = pareto_classification(expected == actual, &rows);
    (classification, rows)
}

/// Hard coverage gate, independent of ratchet classification: the report must
/// carry a signature for every frame of the whole run on both sides under the
/// expected projection, or the baseline comparison is meaningless.
fn enforce_gems_coverage(gems: &ReportGemsComparison, total_frames: u64) -> Result<()> {
    if gems.projection != wildside_core::GEMS_FRAME_SIGNATURE_PROJECTION_NAME {
        bail!(
            "GEMS projection contract changed: expected {}, got {}",
            wildside_core::GEMS_FRAME_SIGNATURE_PROJECTION_NAME,
            gems.projection
        );
    }
    let counts = &gems.frame_counts;
    if counts.requested != total_frames
        || counts.native != total_frames
        || counts.oracle != total_frames
    {
        bail!(
            "GEMS coverage incomplete (requested={} native={} oracle={} expected={total_frames})",
            counts.requested,
            counts.native,
            counts.oracle
        );
    }
    if gems.start_frame != Some(0) || gems.compared_frames != total_frames {
        bail!(
            "GEMS comparison does not span the run (start_frame={:?} compared_frames={} expected 0..{total_frames})",
            gems.start_frame,
            gems.compared_frames
        );
    }
    Ok(())
}

fn load_report_gems(report_path: &Path) -> Result<ReportGemsComparison> {
    let content = fs::read_to_string(report_path)
        .with_context(|| format!("failed to read {}", report_path.display()))?;
    let report: VerifyTasReport = serde_json::from_str(&content)
        .with_context(|| format!("failed to parse {}", report_path.display()))?;
    report.gems_frame_comparison.with_context(|| {
        format!(
            "{} has no gems_frame_comparison block; run verify-tas with --compare-gems",
            report_path.display()
        )
    })
}

fn load_expected_baseline(
    recording: &serde_json::Value,
    frontier_path: &Path,
    slug: &str,
) -> Result<GemsBaseline> {
    let block = recording.get(FRONTIER_GEMS_KEY).with_context(|| {
        format!(
            "{} recording {slug} has no {FRONTIER_GEMS_KEY} block",
            frontier_path.display()
        )
    })?;
    serde_json::from_value(block.clone()).with_context(|| {
        format!(
            "failed to parse {FRONTIER_GEMS_KEY} for {slug} in {}",
            frontier_path.display()
        )
    })
}

/// Returns the process exit code: 0 match, 1 regression,
/// [`IMPROVEMENT_EXIT_CODE`] improvement. Coverage/contract failures return
/// `Err` (exit 1 via anyhow) independent of classification.
pub(crate) fn run_tas_gems_baseline(
    report_path: &Path,
    frontier_path: &Path,
    slug: &str,
    update: bool,
) -> Result<i32> {
    let gems = load_report_gems(report_path)?;
    let frontier_content = fs::read_to_string(frontier_path)
        .with_context(|| format!("failed to read {}", frontier_path.display()))?;
    let frontier: serde_json::Value = serde_json::from_str(&frontier_content)
        .with_context(|| format!("failed to parse {}", frontier_path.display()))?;
    let recording = frontier_recording(&frontier, frontier_path, slug)?;
    let total_frames = recording
        .get("total_frames")
        .and_then(serde_json::Value::as_u64)
        .with_context(|| format!("recording {slug} has no total_frames"))?;
    let expected = load_expected_baseline(recording, frontier_path, slug)?;

    enforce_gems_coverage(&gems, total_frames)?;
    let actual = gems.baseline();
    let (classification, rows) = classify(&expected, &actual);

    if update {
        let spliced = splice_recording_block(&frontier_content, slug, FRONTIER_GEMS_KEY, &actual)?;
        if spliced == frontier_content {
            println!(
                "tas-gems-baseline: {} already matches the report; nothing to update",
                frontier_path.display()
            );
        } else {
            if classification == Classification::Regression {
                println!(
                    "tas-gems-baseline: WARNING — updating the baseline to a REGRESSED frontier"
                );
            }
            fs::write(frontier_path, &spliced)
                .with_context(|| format!("failed to write {}", frontier_path.display()))?;
            println!(
                "tas-gems-baseline: updated {FRONTIER_GEMS_KEY} for {slug} in {}",
                frontier_path.display()
            );
            print_metric_table(&rows);
        }
        return Ok(0);
    }

    match classification {
        Classification::Match => {
            println!(
                "tas-gems-baseline: ok ({} mismatched frames, first at {} — matches {})",
                actual.mismatch_count,
                actual
                    .first_mismatch_frame
                    .map_or_else(|| "none".to_owned(), |frame| frame.to_string()),
                frontier_path.display()
            );
            Ok(0)
        }
        Classification::Improvement => {
            println!("tas-gems-baseline: FAIL — IMPROVEMENT (ratchet not yet updated)");
            println!(
                "  GEMS signature mismatch count DROPPED; the committed baseline in {} is stale.",
                frontier_path.display()
            );
            print_metric_table(&rows);
            println!("  Review the delta, then ratchet the baseline with:");
            println!(
                "  cargo run --release -q -p wildside-cli -- tas-gems-baseline --report {} --frontier {} --slug {} --update",
                report_path.display(),
                frontier_path.display(),
                slug
            );
            Ok(IMPROVEMENT_EXIT_CODE)
        }
        Classification::Regression => {
            println!("tas-gems-baseline: FAIL — REGRESSION");
            println!(
                "  GEMS audio-timing parity regressed against the committed baseline in {}.",
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
            Ok(1)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tas_pixel_baseline::reindent;

    fn baseline() -> GemsBaseline {
        GemsBaseline {
            compared_frames: 37_759,
            mismatch_count: 37_000,
            oracle_limitation_count: 4,
            advisory_only_frames: 100,
            mismatch_digest: Some("aec5d5040dd4050565b0055501de47cd087e66e6".to_owned()),
            first_mismatch_frame: Some(0),
        }
    }

    fn report_fixture() -> ReportGemsComparison {
        ReportGemsComparison {
            projection: wildside_core::GEMS_FRAME_SIGNATURE_PROJECTION_NAME.to_owned(),
            frame_counts: ReportFrameCounts {
                requested: 37_759,
                native: 37_759,
                oracle: 37_759,
            },
            start_frame: Some(0),
            compared_frames: 37_759,
            mismatch_count: 37_000,
            oracle_limitation_count: 4,
            advisory_only_frames: 100,
            mismatch_digest: Some("aec5d5040dd4050565b0055501de47cd087e66e6".to_owned()),
            first_mismatch: Some(ReportGemsFirstMismatch { frame_index: 0 }),
        }
    }

    #[test]
    fn classify_identical_is_match() {
        let (classification, rows) = classify(&baseline(), &baseline());
        assert_eq!(classification, Classification::Match);
        assert!(
            rows.iter()
                .all(|row| row.direction == MetricDirection::Same)
        );
    }

    #[test]
    fn classify_fewer_mismatches_is_improvement() {
        let mut actual = baseline();
        actual.mismatch_count -= 1_000;
        actual.first_mismatch_frame = Some(1_709);
        actual.mismatch_digest = Some("0000000000000000000000000000000000000000".to_owned());
        let (classification, _) = classify(&baseline(), &actual);
        assert_eq!(classification, Classification::Improvement);
    }

    #[test]
    fn classify_later_first_mismatch_alone_is_regression_not_improvement() {
        // Frontier frozen (#416): advancing first_mismatch_frame without
        // changing mismatch_count earns Regression-for-review, not the exit-3
        // Improvement reward the retired per-frame ratchet paid out.
        let mut actual = baseline();
        actual.first_mismatch_frame = Some(1_709);
        let (classification, rows) = classify(&baseline(), &actual);
        assert_eq!(classification, Classification::Regression);
        let guard = rows
            .iter()
            .find(|row| row.name == "first_mismatch_frame")
            .expect("guard row present");
        assert_eq!(
            guard.direction,
            MetricDirection::Same,
            "a later first mismatch must never rank Better"
        );
    }

    #[test]
    fn classify_fewer_mismatches_but_earlier_first_mismatch_is_regression() {
        // The freeze guard: fixing many late-frame mismatches while breaking
        // a frame inside the defended span (first mismatch moves EARLIER)
        // must block as Regression, not be rewarded for the net count drop.
        let mut expected = baseline();
        expected.first_mismatch_frame = Some(923);
        let mut actual = expected.clone();
        actual.mismatch_count -= 1_000;
        actual.first_mismatch_frame = Some(400);
        let (classification, _) = classify(&expected, &actual);
        assert_eq!(classification, Classification::Regression);
    }

    #[test]
    fn classify_new_mismatch_where_none_existed_is_regression() {
        let mut expected = baseline();
        expected.mismatch_count = 0;
        expected.first_mismatch_frame = None;
        expected.mismatch_digest = None;
        let mut actual = expected.clone();
        actual.first_mismatch_frame = Some(10);
        let (classification, _) = classify(&expected, &actual);
        assert_eq!(classification, Classification::Regression);
    }

    #[test]
    fn classify_no_mismatch_at_all_is_improvement() {
        let actual = GemsBaseline {
            compared_frames: 37_759,
            mismatch_count: 0,
            oracle_limitation_count: 4,
            advisory_only_frames: 100,
            mismatch_digest: None,
            first_mismatch_frame: None,
        };
        let (classification, _) = classify(&baseline(), &actual);
        assert_eq!(classification, Classification::Improvement);
    }

    #[test]
    fn classify_earlier_first_mismatch_is_regression() {
        let mut expected = baseline();
        expected.first_mismatch_frame = Some(1_709);
        let mut actual = expected.clone();
        actual.first_mismatch_frame = Some(3);
        let (classification, _) = classify(&expected, &actual);
        assert_eq!(classification, Classification::Regression);
    }

    #[test]
    fn classify_grown_mismatch_count_is_regression() {
        // A worsened run must still exit 1 even when the (evidence-only)
        // first mismatch moved later.
        let mut actual = baseline();
        actual.first_mismatch_frame = Some(200);
        actual.mismatch_count += 10;
        let (classification, _) = classify(&baseline(), &actual);
        assert_eq!(classification, Classification::Regression);
    }

    #[test]
    fn classify_digest_only_change_is_regression() {
        let mut actual = baseline();
        actual.mismatch_digest = Some("1111111111111111111111111111111111111111".to_owned());
        let (classification, _) = classify(&baseline(), &actual);
        assert_eq!(classification, Classification::Regression);
    }

    #[test]
    fn coverage_checks_hard_fail() {
        enforce_gems_coverage(&report_fixture(), 37_759).expect("clean fixture passes");

        let mut wrong_projection = report_fixture();
        wrong_projection.projection = "other_projection_v9".to_owned();
        assert!(
            enforce_gems_coverage(&wrong_projection, 37_759)
                .expect_err("wrong projection")
                .to_string()
                .contains("projection contract")
        );

        let mut short = report_fixture();
        short.frame_counts.oracle = 37_758;
        assert!(
            enforce_gems_coverage(&short, 37_759)
                .expect_err("short capture")
                .to_string()
                .contains("coverage incomplete")
        );

        let mut late_start = report_fixture();
        late_start.start_frame = Some(1);
        late_start.compared_frames = 37_758;
        assert!(
            enforce_gems_coverage(&late_start, 37_759)
                .expect_err("late start")
                .to_string()
                .contains("does not span the run")
        );
    }

    #[test]
    fn report_loader_ignores_extra_fields() {
        let report = serde_json::json!({
            "slug": "any-percent-published",
            "gems_frame_comparison": {
                "projection": "gems_frame_signature_v1",
                "proves_ym_writes": false,
                "proves_sample_audio": false,
                "proves_intra_frame_ordering": false,
                "advisory_fields": ["z80_mclk_counter", "ym2612_mclk_counter"],
                "coverage": {"route_frames": 37759, "modelled_frames": 0},
                "frame_counts": {"requested": 37759, "native": 37759, "oracle": 37759},
                "start_frame": 0,
                "compared_frames": 37759,
                "mismatch_count": 37000,
                "oracle_limitation_count": 4,
                "advisory_only_frames": 100,
                "mismatch_digest": "aec5d5040dd4050565b0055501de47cd087e66e6",
                "first_mismatch": {
                    "frame_index": 0,
                    "diverging_fields": ["audio_ram_crc32"],
                    "native": {"audio_ram_crc32": 1},
                    "oracle": {"audio_ram_crc32": 2}
                }
            }
        });
        let parsed: VerifyTasReport =
            serde_json::from_value(report).expect("extra fields are ignored");
        let gems = parsed.gems_frame_comparison.expect("block present");
        assert_eq!(gems.baseline(), baseline());
        enforce_gems_coverage(&gems, 37_759).expect("coverage complete");
    }

    /// Minimal frontier fixture shaped like tests/tas_frontier.json.
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
      "full_expected_gems_frame_comparison": {block},
      "status": "verified"
    }}
  }}
}}
"#
        )
    }

    #[test]
    fn update_writes_spliced_file_and_write_guard_skips_matching() {
        let dir = tempfile::tempdir().expect("tempdir");
        let frontier_path = dir.path().join("tas_frontier.json");
        fs::write(&frontier_path, frontier_fixture()).expect("write fixture");

        let mut improved = report_fixture();
        improved.mismatch_count = 36_000;
        improved.first_mismatch = Some(ReportGemsFirstMismatch { frame_index: 1_709 });
        improved.mismatch_digest = Some("feedfacefeedfacefeedfacefeedfacefeedface".to_owned());
        let report_path = dir.path().join("report.json");
        let report = serde_json::json!({
            "gems_frame_comparison": serde_json::to_value(&improved).expect("fixture value"),
        });
        fs::write(
            &report_path,
            serde_json::to_vec_pretty(&report).expect("report bytes"),
        )
        .expect("write report");

        // Without --update: improvement exit code.
        let code =
            run_tas_gems_baseline(&report_path, &frontier_path, "any-percent-published", false)
                .expect("classification run");
        assert_eq!(code, IMPROVEMENT_EXIT_CODE);

        // With --update: splices and subsequently matches.
        let code =
            run_tas_gems_baseline(&report_path, &frontier_path, "any-percent-published", true)
                .expect("update run");
        assert_eq!(code, 0);
        let code =
            run_tas_gems_baseline(&report_path, &frontier_path, "any-percent-published", false)
                .expect("post-update run");
        assert_eq!(code, 0, "updated baseline must classify as Match");

        // Write-guard: a second --update leaves the file byte-identical.
        let before = fs::read_to_string(&frontier_path).expect("before");
        let code =
            run_tas_gems_baseline(&report_path, &frontier_path, "any-percent-published", true)
                .expect("no-op update");
        assert_eq!(code, 0);
        let after = fs::read_to_string(&frontier_path).expect("after");
        assert_eq!(before, after);
    }

    #[test]
    fn regression_exits_one_with_digest_note() {
        let dir = tempfile::tempdir().expect("tempdir");
        let frontier_path = dir.path().join("tas_frontier.json");
        fs::write(&frontier_path, frontier_fixture()).expect("write fixture");

        let mut regressed = report_fixture();
        regressed.mismatch_count += 5;
        regressed.mismatch_digest = Some("feedfacefeedfacefeedfacefeedfacefeedface".to_owned());
        let report_path = dir.path().join("report.json");
        fs::write(
            &report_path,
            serde_json::to_vec_pretty(&serde_json::json!({
                "gems_frame_comparison": serde_json::to_value(&regressed).expect("fixture"),
            }))
            .expect("report bytes"),
        )
        .expect("write report");

        let code =
            run_tas_gems_baseline(&report_path, &frontier_path, "any-percent-published", false)
                .expect("regression run");
        assert_eq!(code, 1);
    }
}
