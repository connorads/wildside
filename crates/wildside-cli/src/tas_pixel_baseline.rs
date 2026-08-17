//! Ratchet-aware pixel-fidelity gate for the full published-route TAS run.
//!
//! Compares a `verify-tas --compare-pixels` report's `pixel_comparison` block
//! against the slug's `full_expected_pixel_comparison` baseline in
//! `tests/tas_frontier.json`, classifying the delta as Match / Improvement /
//! Regression instead of the old byte-for-byte jq equality (where an
//! improvement failed identically to a regression and the moved frontier had
//! to be rediscovered by hand). On improvement the gate still fails (exit 3)
//! but prints an unmistakable classification plus the exact `--update`
//! command; `--update` splices only the pixel block into the frontier file so
//! the rest stays byte-identical (serde_json would alphabetise the whole file
//! on a round-trip).

use anyhow::{Context, Result, bail};
use serde::{Deserialize, Serialize};
use std::borrow::Cow;
use std::fs;
use std::path::Path;

use crate::tas_pixels::TAS_PIXEL_PROJECTION_NAME;

/// Exit code for "the pixel frontier moved forward; ratchet not yet updated".
/// Distinct from 1 (regression / coverage failure) so callers can tell the
/// two failure modes apart mechanically.
pub(crate) const IMPROVEMENT_EXIT_CODE: i32 = 3;

const FRONTIER_PIXEL_KEY: &str = "full_expected_pixel_comparison";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
struct PixelMismatchEvidence {
    frame_index: u64,
    changed_pixels: u64,
}

/// Field order mirrors the on-disk `full_expected_pixel_comparison` block in
/// the frontier file; `--update` serialises this struct directly into it.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
struct PixelBaseline {
    exact_match_frames: u64,
    mismatch_frames: u64,
    total_changed_pixels: u64,
    mismatch_digest: String,
    first_mismatch: Option<PixelMismatchEvidence>,
    worst_mismatch: Option<PixelMismatchEvidence>,
}

#[derive(Debug, Clone, Deserialize)]
#[cfg_attr(test, derive(Serialize))]
struct ReportFrameCounts {
    requested: u64,
    native: u64,
    oracle: u64,
    compared: u64,
}

/// The `pixel_comparison` block of a verify-tas report. Extra report fields
/// (evidence labels, sha1s, proves_* flags we don't gate on) are ignored by
/// serde's default unknown-field behaviour.
#[derive(Debug, Clone, Deserialize)]
#[cfg_attr(test, derive(Serialize))]
struct ReportPixelComparison {
    projection: String,
    frame_counts: ReportFrameCounts,
    proves_every_frame_in_range: bool,
    exact_match_frames: u64,
    mismatch_frames: u64,
    total_changed_pixels: u64,
    mismatch_digest: String,
    first_mismatch: Option<PixelMismatchEvidence>,
    worst_mismatch: Option<PixelMismatchEvidence>,
}

impl ReportPixelComparison {
    fn baseline(&self) -> PixelBaseline {
        PixelBaseline {
            exact_match_frames: self.exact_match_frames,
            mismatch_frames: self.mismatch_frames,
            total_changed_pixels: self.total_changed_pixels,
            mismatch_digest: self.mismatch_digest.clone(),
            first_mismatch: self.first_mismatch.clone(),
            worst_mismatch: self.worst_mismatch.clone(),
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
struct VerifyTasReport {
    pixel_comparison: Option<ReportPixelComparison>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Classification {
    Match,
    Improvement,
    Regression,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum MetricDirection {
    Better,
    Same,
    Worse,
}

#[derive(Debug, Clone)]
pub(crate) struct MetricRow {
    /// `Cow` so per-region rows can carry computed keys while the fixed
    /// metrics stay allocation-free literals.
    pub(crate) name: Cow<'static, str>,
    pub(crate) expected: String,
    pub(crate) actual: String,
    pub(crate) direction: MetricDirection,
}

fn direction_marker(direction: MetricDirection) -> &'static str {
    match direction {
        MetricDirection::Better => "better",
        MetricDirection::Same => "same",
        MetricDirection::Worse => "WORSE",
    }
}

pub(crate) fn metric_higher_better(
    name: impl Into<Cow<'static, str>>,
    expected: u64,
    actual: u64,
) -> MetricRow {
    MetricRow {
        name: name.into(),
        expected: expected.to_string(),
        actual: actual.to_string(),
        direction: match actual.cmp(&expected) {
            std::cmp::Ordering::Greater => MetricDirection::Better,
            std::cmp::Ordering::Equal => MetricDirection::Same,
            std::cmp::Ordering::Less => MetricDirection::Worse,
        },
    }
}

pub(crate) fn metric_lower_better(
    name: impl Into<Cow<'static, str>>,
    expected: u64,
    actual: u64,
) -> MetricRow {
    MetricRow {
        name: name.into(),
        expected: expected.to_string(),
        actual: actual.to_string(),
        direction: match actual.cmp(&expected) {
            std::cmp::Ordering::Less => MetricDirection::Better,
            std::cmp::Ordering::Equal => MetricDirection::Same,
            std::cmp::Ordering::Greater => MetricDirection::Worse,
        },
    }
}

/// Pareto classification shared by the ratchet gates: the caller supplies
/// baseline equality (Match detection) and the ranked metric rows. Evidence
/// fields such as digests are deliberately NOT metrics — a pure reshuffle of
/// the same totals cannot count as an improvement. Mixed win/lose trades
/// classify as Regression.
pub(crate) fn pareto_classification(baseline_matches: bool, rows: &[MetricRow]) -> Classification {
    if baseline_matches {
        return Classification::Match;
    }
    let any_worse = rows
        .iter()
        .any(|row| row.direction == MetricDirection::Worse);
    let any_better = rows
        .iter()
        .any(|row| row.direction == MetricDirection::Better);
    if !any_worse && any_better {
        Classification::Improvement
    } else {
        Classification::Regression
    }
}

/// `None` (no mismatch at all) is the best possible value for both optional
/// metrics; `map_value` picks which evidence field the metric ranks on.
fn metric_optional(
    name: impl Into<Cow<'static, str>>,
    expected: Option<&PixelMismatchEvidence>,
    actual: Option<&PixelMismatchEvidence>,
    map_value: fn(&PixelMismatchEvidence) -> u64,
    higher_better: bool,
) -> MetricRow {
    let display = |evidence: Option<&PixelMismatchEvidence>| {
        evidence.map_or_else(|| "none".to_owned(), |e| map_value(e).to_string())
    };
    let direction = match (expected, actual) {
        (None, None) => MetricDirection::Same,
        (Some(_), None) => MetricDirection::Better,
        (None, Some(_)) => MetricDirection::Worse,
        (Some(e), Some(a)) => {
            let (e, a) = (map_value(e), map_value(a));
            match (a.cmp(&e), higher_better) {
                (std::cmp::Ordering::Equal, _) => MetricDirection::Same,
                (std::cmp::Ordering::Greater, true) | (std::cmp::Ordering::Less, false) => {
                    MetricDirection::Better
                }
                _ => MetricDirection::Worse,
            }
        }
    };
    MetricRow {
        name: name.into(),
        expected: display(expected),
        actual: display(actual),
        direction,
    }
}

/// Pareto classification over the five ratchet metrics. The digest is
/// deliberately NOT a metric (digest-neutral): it participates only in Match
/// detection, so a pure reshuffle of the same totals cannot count as an
/// improvement. Mixed win/lose trades classify as Regression.
fn classify(expected: &PixelBaseline, actual: &PixelBaseline) -> (Classification, Vec<MetricRow>) {
    let rows = vec![
        metric_higher_better(
            "exact_match_frames",
            expected.exact_match_frames,
            actual.exact_match_frames,
        ),
        metric_lower_better(
            "mismatch_frames",
            expected.mismatch_frames,
            actual.mismatch_frames,
        ),
        metric_lower_better(
            "total_changed_pixels",
            expected.total_changed_pixels,
            actual.total_changed_pixels,
        ),
        metric_optional(
            "first_mismatch_frame",
            expected.first_mismatch.as_ref(),
            actual.first_mismatch.as_ref(),
            |evidence| evidence.frame_index,
            true,
        ),
        metric_optional(
            "worst_mismatch_pixels",
            expected.worst_mismatch.as_ref(),
            actual.worst_mismatch.as_ref(),
            |evidence| evidence.changed_pixels,
            false,
        ),
    ];
    let classification = pareto_classification(expected == actual, &rows);
    (classification, rows)
}

/// Hard coverage gate, independent of ratchet classification: the report must
/// prove every frame of the whole run under the expected projection, or the
/// baseline comparison is meaningless.
fn enforce_pixel_coverage(pixel: &ReportPixelComparison, total_frames: u64) -> Result<()> {
    if pixel.projection != TAS_PIXEL_PROJECTION_NAME {
        bail!(
            "pixel projection contract changed: expected {TAS_PIXEL_PROJECTION_NAME}, got {}",
            pixel.projection
        );
    }
    if !pixel.proves_every_frame_in_range {
        bail!("pixel evidence does not prove every frame in range");
    }
    let counts = &pixel.frame_counts;
    if counts.requested != total_frames
        || counts.native != total_frames
        || counts.oracle != total_frames
        || counts.compared != total_frames
    {
        bail!(
            "pixel coverage incomplete (requested={} native={} oracle={} compared={} expected={total_frames})",
            counts.requested,
            counts.native,
            counts.oracle,
            counts.compared
        );
    }
    Ok(())
}

fn load_report_pixels(report_path: &Path) -> Result<ReportPixelComparison> {
    let content = fs::read_to_string(report_path)
        .with_context(|| format!("failed to read {}", report_path.display()))?;
    let report: VerifyTasReport = serde_json::from_str(&content)
        .with_context(|| format!("failed to parse {}", report_path.display()))?;
    report.pixel_comparison.with_context(|| {
        format!(
            "{} has no pixel_comparison block; run verify-tas with --compare-pixels",
            report_path.display()
        )
    })
}

pub(crate) fn frontier_recording<'a>(
    frontier: &'a serde_json::Value,
    frontier_path: &Path,
    slug: &str,
) -> Result<&'a serde_json::Value> {
    frontier
        .get("recordings")
        .and_then(|recordings| recordings.get(slug))
        .with_context(|| {
            format!(
                "{} has no recordings entry for slug {slug}",
                frontier_path.display()
            )
        })
}

fn load_expected_baseline(
    recording: &serde_json::Value,
    frontier_path: &Path,
    slug: &str,
) -> Result<PixelBaseline> {
    let block = recording.get(FRONTIER_PIXEL_KEY).with_context(|| {
        format!(
            "{} recording {slug} has no {FRONTIER_PIXEL_KEY} block",
            frontier_path.display()
        )
    })?;
    serde_json::from_value(block.clone()).with_context(|| {
        format!(
            "failed to parse {FRONTIER_PIXEL_KEY} for {slug} in {}",
            frontier_path.display()
        )
    })
}

/// Index of the `}` matching the `{` at `open`, skipping braces inside JSON
/// strings (string-aware counter).
fn match_brace(text: &str, open: usize) -> Result<usize> {
    let mut depth = 0usize;
    let mut in_string = false;
    let mut escaped = false;
    for (offset, byte) in text.as_bytes()[open..].iter().enumerate() {
        if in_string {
            if escaped {
                escaped = false;
            } else if *byte == b'\\' {
                escaped = true;
            } else if *byte == b'"' {
                in_string = false;
            }
            continue;
        }
        match byte {
            b'"' => in_string = true,
            b'{' => depth += 1,
            b'}' => {
                depth -= 1;
                if depth == 0 {
                    return Ok(open + offset);
                }
            }
            _ => {}
        }
    }
    bail!("unbalanced braces from offset {open}");
}

/// Re-indent pretty JSON (2-space, first line unindented) so nested lines sit
/// under a key at `indent`.
pub(crate) fn reindent(pretty: &str, indent: &str) -> String {
    let mut out = String::new();
    for (index, line) in pretty.lines().enumerate() {
        if index > 0 {
            out.push('\n');
            out.push_str(indent);
        }
        out.push_str(line);
    }
    out
}

/// Splice a serialised baseline into the slug's `block_key` block, leaving
/// every other byte of the frontier file untouched. Deliberately NOT a
/// `serde_json::Value` round-trip: without the `preserve_order` feature that
/// would alphabetise the whole file.
pub(crate) fn splice_recording_block(
    original: &str,
    slug: &str,
    block_key: &str,
    actual: &impl Serialize,
) -> Result<String> {
    let slug_key = format!("\"{slug}\"");
    let slug_pos = original
        .find(&slug_key)
        .with_context(|| format!("frontier file has no {slug_key} key"))?;
    let key = format!("\"{block_key}\"");
    let key_pos = original[slug_pos..]
        .find(&key)
        .map(|offset| slug_pos + offset)
        .with_context(|| format!("no {key} block after {slug_key}"))?;
    let line_start = original[..key_pos].rfind('\n').map_or(0, |pos| pos + 1);
    let indent = &original[line_start..key_pos];
    if !indent.chars().all(|c| c == ' ') {
        bail!("unexpected non-space indent before {key}");
    }
    let brace_pos = original[key_pos..]
        .find('{')
        .map(|offset| key_pos + offset)
        .with_context(|| format!("no opening brace after {key}"))?;
    let block_end = match_brace(original, brace_pos)?;
    let pretty = serde_json::to_string_pretty(actual)?;
    Ok(format!(
        "{}{}{}",
        &original[..brace_pos],
        reindent(&pretty, indent),
        &original[block_end + 1..]
    ))
}

fn splice_pixel_baseline(original: &str, slug: &str, actual: &PixelBaseline) -> Result<String> {
    splice_recording_block(original, slug, FRONTIER_PIXEL_KEY, actual)
}

/// One update guard shared by all three TAS ratchets. `--force` is only
/// meaningful with `--update`, and a regressed report may only be baselined
/// under an explicit `--update --force`: an accidental `--update` after a
/// regressed run must not quietly lower the bar behind a printed warning.
/// The caller bails before any write, so the frontier file stays untouched.
pub(crate) fn guard_baseline_update(
    gate: &str,
    classification: Classification,
    rows: &[MetricRow],
    update: bool,
    force: bool,
) -> Result<()> {
    if force && !update {
        bail!("{gate}: --force is only meaningful together with --update");
    }
    if update && !force && classification == Classification::Regression {
        println!("{gate}: REFUSING --update — the report REGRESSES the committed baseline");
        print_metric_table(rows);
        println!(
            "  review the regression; to deliberately accept it, re-run with --update --force"
        );
        bail!("{gate}: refusing to ratchet a regression without --force");
    }
    Ok(())
}

pub(crate) fn print_metric_table(rows: &[MetricRow]) {
    let name_width = rows
        .iter()
        .map(|row| row.name.len())
        .max()
        .unwrap_or(0)
        .max(22);
    for row in rows {
        println!(
            "  {:<name_width$} {:>10} -> {:>10}  [{}]",
            row.name,
            row.expected,
            row.actual,
            direction_marker(row.direction)
        );
    }
}

/// Returns the process exit code: 0 match, 1 regression,
/// [`IMPROVEMENT_EXIT_CODE`] improvement. Coverage/contract failures return
/// `Err` (exit 1 via anyhow) independent of classification.
pub(crate) fn run_tas_pixel_baseline(
    report_path: &Path,
    frontier_path: &Path,
    slug: &str,
    update: bool,
    force: bool,
) -> Result<i32> {
    let pixel = load_report_pixels(report_path)?;
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

    enforce_pixel_coverage(&pixel, total_frames)?;
    let actual = pixel.baseline();
    let (classification, rows) = classify(&expected, &actual);
    guard_baseline_update("tas-pixel-baseline", classification, &rows, update, force)?;

    if update {
        let spliced = splice_pixel_baseline(&frontier_content, slug, &actual)?;
        if spliced == frontier_content {
            println!(
                "tas-pixel-baseline: {} already matches the report; nothing to update",
                frontier_path.display()
            );
        } else {
            if classification == Classification::Regression {
                println!(
                    "tas-pixel-baseline: WARNING — updating the baseline to a REGRESSED frontier"
                );
            }
            fs::write(frontier_path, &spliced)
                .with_context(|| format!("failed to write {}", frontier_path.display()))?;
            println!(
                "tas-pixel-baseline: updated {FRONTIER_PIXEL_KEY} for {slug} in {}",
                frontier_path.display()
            );
            print_metric_table(&rows);
        }
        return Ok(0);
    }

    match classification {
        Classification::Match => {
            println!(
                "tas-pixel-baseline: ok ({} exact, {} divergent, {} changed pixels — matches {})",
                actual.exact_match_frames,
                actual.mismatch_frames,
                actual.total_changed_pixels,
                frontier_path.display()
            );
            Ok(0)
        }
        Classification::Improvement => {
            println!("tas-pixel-baseline: FAIL — IMPROVEMENT (ratchet not yet updated)");
            println!(
                "  The pixel frontier moved FORWARD; the committed baseline in {} is stale.",
                frontier_path.display()
            );
            print_metric_table(&rows);
            println!("  Review the delta, then ratchet the baseline with:");
            println!(
                "  cargo run --release -q -p wildside-cli -- tas-pixel-baseline --report {} --frontier {} --slug {} --update",
                report_path.display(),
                frontier_path.display(),
                slug
            );
            Ok(IMPROVEMENT_EXIT_CODE)
        }
        Classification::Regression => {
            println!("tas-pixel-baseline: FAIL — REGRESSION");
            println!(
                "  Pixel fidelity regressed against the committed baseline in {}.",
                frontier_path.display()
            );
            print_metric_table(&rows);
            if actual.mismatch_digest != expected.mismatch_digest {
                println!(
                    "  mismatch_digest changed: {} -> {}",
                    expected.mismatch_digest, actual.mismatch_digest
                );
            }
            Ok(1)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn evidence(frame_index: u64, changed_pixels: u64) -> PixelMismatchEvidence {
        PixelMismatchEvidence {
            frame_index,
            changed_pixels,
        }
    }

    fn baseline() -> PixelBaseline {
        PixelBaseline {
            exact_match_frames: 24310,
            mismatch_frames: 13449,
            total_changed_pixels: 8408626,
            mismatch_digest: "aec5d5040dd4050565b0055501de47cd087e66e6".to_owned(),
            first_mismatch: Some(evidence(963, 59)),
            worst_mismatch: Some(evidence(22511, 2595)),
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
    fn classify_clean_improvement() {
        let mut actual = baseline();
        actual.exact_match_frames += 100;
        actual.mismatch_frames -= 100;
        actual.total_changed_pixels -= 50_000;
        actual.first_mismatch = Some(evidence(1200, 40));
        actual.worst_mismatch = Some(evidence(22511, 2000));
        actual.mismatch_digest = "0000000000000000000000000000000000000000".to_owned();
        let (classification, _) = classify(&baseline(), &actual);
        assert_eq!(classification, Classification::Improvement);
    }

    #[test]
    fn classify_total_pixels_up_is_regression() {
        let mut actual = baseline();
        actual.total_changed_pixels += 1;
        let (classification, rows) = classify(&baseline(), &actual);
        assert_eq!(classification, Classification::Regression);
        assert_eq!(
            rows.iter()
                .filter(|row| row.direction == MetricDirection::Worse)
                .count(),
            1
        );
    }

    #[test]
    fn classify_mixed_trade_first_later_but_total_up_is_regression() {
        // First mismatch moved later (better) but total pixels grew (worse):
        // mixed win/lose trades are NOT improvements.
        let mut actual = baseline();
        actual.first_mismatch = Some(evidence(2000, 59));
        actual.total_changed_pixels += 10;
        let (classification, _) = classify(&baseline(), &actual);
        assert_eq!(classification, Classification::Regression);
    }

    #[test]
    fn classify_first_later_with_more_pixels_at_first_frame_is_improvement() {
        // first_mismatch.changed_pixels is NOT a ratchet metric; only the
        // frame index is. A later first mismatch that happens to be larger
        // still improves when exact frames rise and total pixels fall.
        let mut actual = baseline();
        actual.exact_match_frames += 5;
        actual.mismatch_frames -= 5;
        actual.total_changed_pixels -= 300;
        actual.first_mismatch = Some(evidence(1500, 500));
        let (classification, _) = classify(&baseline(), &actual);
        assert_eq!(classification, Classification::Improvement);
    }

    #[test]
    fn classify_digest_only_change_is_regression() {
        // Digest-neutral: a reshuffle with identical totals is not a Match
        // (evidence changed) and has no strictly-better metric, so it fails.
        let mut actual = baseline();
        actual.mismatch_digest = "1111111111111111111111111111111111111111".to_owned();
        let (classification, _) = classify(&baseline(), &actual);
        assert_eq!(classification, Classification::Regression);
    }

    #[test]
    fn classify_no_mismatches_at_all_is_improvement() {
        let actual = PixelBaseline {
            exact_match_frames: 37759,
            mismatch_frames: 0,
            total_changed_pixels: 0,
            mismatch_digest: "da39a3ee5e6b4b0d3255bfef95601890afd80709".to_owned(),
            first_mismatch: None,
            worst_mismatch: None,
        };
        let (classification, _) = classify(&baseline(), &actual);
        assert_eq!(classification, Classification::Improvement);
    }

    #[test]
    fn report_loader_ignores_extra_fields() {
        let report = serde_json::json!({
            "slug": "any-percent-published",
            "unrelated": {"stuff": true},
            "pixel_comparison": {
                "projection": "source_port_visible_rgb24_v1",
                "frame_range": [0, 37759],
                "frame_counts": {
                    "requested": 37759,
                    "native": 37759,
                    "oracle": 37759,
                    "compared": 37759
                },
                "proves_requested_frame_pixels": true,
                "proves_every_frame_in_range": true,
                "proves_whole_state": false,
                "exact_match_frames": 24310,
                "mismatch_frames": 13449,
                "total_changed_pixels": 8408626,
                "mismatch_digest": "aec5d5040dd4050565b0055501de47cd087e66e6",
                "first_mismatch": {
                    "frame_index": 963,
                    "label": "frame_963",
                    "changed_pixels": 59,
                    "native_sha1": "aa",
                    "oracle_sha1": "bb"
                },
                "worst_mismatch": {
                    "frame_index": 22511,
                    "label": "frame_22511",
                    "changed_pixels": 2595,
                    "native_sha1": "cc",
                    "oracle_sha1": "dd"
                }
            }
        });
        let parsed: VerifyTasReport =
            serde_json::from_value(report).expect("extra fields are ignored");
        let pixel = parsed.pixel_comparison.expect("block present");
        assert_eq!(pixel.baseline(), baseline());
        enforce_pixel_coverage(&pixel, 37759).expect("coverage complete");
    }

    fn coverage_fixture() -> ReportPixelComparison {
        ReportPixelComparison {
            projection: TAS_PIXEL_PROJECTION_NAME.to_owned(),
            frame_counts: ReportFrameCounts {
                requested: 100,
                native: 100,
                oracle: 100,
                compared: 100,
            },
            proves_every_frame_in_range: true,
            exact_match_frames: 100,
            mismatch_frames: 0,
            total_changed_pixels: 0,
            mismatch_digest: String::new(),
            first_mismatch: None,
            worst_mismatch: None,
        }
    }

    #[test]
    fn coverage_checks_hard_fail() {
        enforce_pixel_coverage(&coverage_fixture(), 100).expect("clean fixture passes");

        let mut wrong_projection = coverage_fixture();
        wrong_projection.projection = "other_projection_v9".to_owned();
        assert!(
            enforce_pixel_coverage(&wrong_projection, 100)
                .expect_err("wrong projection")
                .to_string()
                .contains("projection contract")
        );

        let mut sampled = coverage_fixture();
        sampled.proves_every_frame_in_range = false;
        assert!(
            enforce_pixel_coverage(&sampled, 100)
                .expect_err("sampled coverage")
                .to_string()
                .contains("every frame")
        );

        let mut short = coverage_fixture();
        short.frame_counts.compared = 99;
        assert!(
            enforce_pixel_coverage(&short, 100)
                .expect_err("short coverage")
                .to_string()
                .contains("coverage incomplete")
        );

        assert!(
            enforce_pixel_coverage(&coverage_fixture(), 101)
                .expect_err("total_frames mismatch")
                .to_string()
                .contains("coverage incomplete")
        );
    }

    /// Minimal frontier fixture shaped like tests/tas_frontier.json (nested
    /// recordings, 2-space indent, sibling recordings before and after).
    fn frontier_fixture() -> String {
        let baseline = baseline();
        let block = reindent(
            &serde_json::to_string_pretty(&baseline).expect("serialise baseline"),
            "      ",
        );
        format!(
            r#"{{
  "recordings": {{
    "before-slug": {{
      "total_frames": 10,
      "status": "verified"
    }},
    "any-percent-published": {{
      "scenario": "tests/data/tas/any-percent-published/scenario.json",
      "total_frames": 37759,
      "full_expected_pixel_comparison": {block},
      "status": "verified"
    }},
    "after-slug": {{
      "total_frames": 20,
      "status": "verified"
    }}
  }}
}}
"#
        )
    }

    #[test]
    fn splice_replaces_only_the_pixel_block() {
        let original = frontier_fixture();
        let mut actual = baseline();
        actual.exact_match_frames += 7;
        actual.mismatch_frames -= 7;
        actual.total_changed_pixels -= 123;
        actual.first_mismatch = Some(evidence(1042, 12));
        actual.mismatch_digest = "feedfacefeedfacefeedfacefeedfacefeedface".to_owned();

        let spliced =
            splice_pixel_baseline(&original, "any-percent-published", &actual).expect("splice");

        let original_value: serde_json::Value = serde_json::from_str(&original).expect("original");
        let spliced_value: serde_json::Value =
            serde_json::from_str(&spliced).expect("spliced stays valid JSON");
        let path = |value: &serde_json::Value| {
            value["recordings"]["any-percent-published"][FRONTIER_PIXEL_KEY].clone()
        };
        assert_eq!(
            path(&spliced_value),
            serde_json::to_value(&actual).expect("actual as value")
        );
        assert_ne!(path(&spliced_value), path(&original_value));

        // Everything outside the block is byte-identical: prefix up to the
        // opening brace and suffix after the closing brace.
        let key_pos = original.find(FRONTIER_PIXEL_KEY).expect("key");
        let brace_pos = key_pos + original[key_pos..].find('{').expect("brace");
        let block_end = match_brace(&original, brace_pos).expect("balanced");
        assert!(spliced.starts_with(&original[..brace_pos]));
        assert!(spliced.ends_with(&original[block_end + 1..]));

        // Sibling recordings untouched.
        assert_eq!(
            spliced_value["recordings"]["before-slug"],
            original_value["recordings"]["before-slug"]
        );
        assert_eq!(
            spliced_value["recordings"]["after-slug"],
            original_value["recordings"]["after-slug"]
        );
    }

    #[test]
    fn splice_is_byte_identical_when_actual_matches_disk() {
        let original = frontier_fixture();
        let spliced =
            splice_pixel_baseline(&original, "any-percent-published", &baseline()).expect("splice");
        assert_eq!(spliced, original, "write-guard input must be byte-stable");
    }

    #[test]
    fn splice_matches_real_frontier_block_format() {
        // The committed tests/tas_frontier.json block must round-trip
        // byte-identically through the splicer, or --update would churn the
        // file the first time it runs on a genuine improvement.
        let repo_root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .and_then(Path::parent)
            .expect("workspace root");
        let frontier_path = repo_root.join("tests/tas_frontier.json");
        let original = fs::read_to_string(&frontier_path).expect("read committed frontier");
        let frontier: serde_json::Value = serde_json::from_str(&original).expect("parse frontier");
        let expected: PixelBaseline = serde_json::from_value(
            frontier["recordings"]["any-percent-published"][FRONTIER_PIXEL_KEY].clone(),
        )
        .expect("committed baseline parses");
        let spliced = splice_pixel_baseline(&original, "any-percent-published", &expected)
            .expect("splice committed file");
        assert_eq!(spliced, original);
    }

    fn report_from(comparison: &ReportPixelComparison, dir: &Path) -> std::path::PathBuf {
        let report_path = dir.join("report.json");
        let report = serde_json::json!({
            "pixel_comparison": serde_json::to_value(comparison).expect("fixture value"),
        });
        fs::write(
            &report_path,
            serde_json::to_vec_pretty(&report).expect("report bytes"),
        )
        .expect("write report");
        report_path
    }

    #[test]
    fn update_refuses_regression_without_force_and_leaves_file_untouched() {
        let dir = tempfile::tempdir().expect("tempdir");
        let frontier_path = dir.path().join("tas_frontier.json");
        fs::write(&frontier_path, frontier_fixture()).expect("write fixture");
        let original = fs::read(&frontier_path).expect("original bytes");

        let mut regressed = coverage_fixture();
        regressed.frame_counts = ReportFrameCounts {
            requested: 37759,
            native: 37759,
            oracle: 37759,
            compared: 37759,
        };
        let base = baseline();
        regressed.exact_match_frames = base.exact_match_frames;
        regressed.mismatch_frames = base.mismatch_frames;
        regressed.total_changed_pixels = base.total_changed_pixels + 1;
        regressed.mismatch_digest = base.mismatch_digest.clone();
        regressed.first_mismatch = base.first_mismatch.clone();
        regressed.worst_mismatch = base.worst_mismatch.clone();
        let report_path = report_from(&regressed, dir.path());

        let error = run_tas_pixel_baseline(
            &report_path,
            &frontier_path,
            "any-percent-published",
            true,
            false,
        )
        .expect_err("regression --update without --force must refuse");
        assert!(error.to_string().contains("--force"));
        assert_eq!(
            fs::read(&frontier_path).expect("after"),
            original,
            "refused update must not touch the frontier file"
        );

        // --update --force deliberately accepts the regressed frontier.
        let code = run_tas_pixel_baseline(
            &report_path,
            &frontier_path,
            "any-percent-published",
            true,
            true,
        )
        .expect("forced update");
        assert_eq!(code, 0);
        assert_ne!(fs::read(&frontier_path).expect("after force"), original);
        let code = run_tas_pixel_baseline(
            &report_path,
            &frontier_path,
            "any-percent-published",
            false,
            false,
        )
        .expect("post-force run");
        assert_eq!(code, 0, "forced baseline must classify as Match");
    }

    #[test]
    fn force_without_update_bails() {
        let dir = tempfile::tempdir().expect("tempdir");
        let frontier_path = dir.path().join("tas_frontier.json");
        fs::write(&frontier_path, frontier_fixture()).expect("write fixture");
        let mut matching = coverage_fixture();
        matching.frame_counts = ReportFrameCounts {
            requested: 37759,
            native: 37759,
            oracle: 37759,
            compared: 37759,
        };
        let base = baseline();
        matching.exact_match_frames = base.exact_match_frames;
        matching.mismatch_frames = base.mismatch_frames;
        matching.total_changed_pixels = base.total_changed_pixels;
        matching.mismatch_digest = base.mismatch_digest.clone();
        matching.first_mismatch = base.first_mismatch.clone();
        matching.worst_mismatch = base.worst_mismatch.clone();
        let report_path = report_from(&matching, dir.path());

        let error = run_tas_pixel_baseline(
            &report_path,
            &frontier_path,
            "any-percent-published",
            false,
            true,
        )
        .expect_err("--force without --update is a usage error");
        assert!(error.to_string().contains("--update"));
    }

    #[test]
    fn update_writes_spliced_file_and_write_guard_skips_matching() {
        let dir = tempfile::tempdir().expect("tempdir");
        let frontier_path = dir.path().join("tas_frontier.json");
        fs::write(&frontier_path, frontier_fixture()).expect("write fixture");

        let mut improved = coverage_fixture();
        improved.frame_counts = ReportFrameCounts {
            requested: 37759,
            native: 37759,
            oracle: 37759,
            compared: 37759,
        };
        improved.exact_match_frames = 24400;
        improved.mismatch_frames = 13359;
        improved.total_changed_pixels = 8_300_000;
        improved.mismatch_digest = "feedfacefeedfacefeedfacefeedfacefeedface".to_owned();
        improved.first_mismatch = Some(evidence(1042, 12));
        improved.worst_mismatch = Some(evidence(22511, 2595));
        let report_path = dir.path().join("report.json");
        let report = serde_json::json!({
            "pixel_comparison": serde_json::to_value(&improved).expect("fixture value"),
        });
        fs::write(
            &report_path,
            serde_json::to_vec_pretty(&report).expect("report bytes"),
        )
        .expect("write report");

        // Without --update: improvement exit code.
        let code = run_tas_pixel_baseline(
            &report_path,
            &frontier_path,
            "any-percent-published",
            false,
            false,
        )
        .expect("classification run");
        assert_eq!(code, IMPROVEMENT_EXIT_CODE);

        // With --update: splices and subsequently matches.
        let code = run_tas_pixel_baseline(
            &report_path,
            &frontier_path,
            "any-percent-published",
            true,
            false,
        )
        .expect("update run");
        assert_eq!(code, 0);
        let updated = fs::read_to_string(&frontier_path).expect("read updated");
        let updated_value: serde_json::Value = serde_json::from_str(&updated).expect("valid JSON");
        assert_eq!(
            updated_value["recordings"]["any-percent-published"][FRONTIER_PIXEL_KEY]["total_changed_pixels"],
            serde_json::json!(8_300_000u64)
        );

        let code = run_tas_pixel_baseline(
            &report_path,
            &frontier_path,
            "any-percent-published",
            false,
            false,
        )
        .expect("post-update run");
        assert_eq!(code, 0, "updated baseline must classify as Match");

        // Write-guard: a second --update leaves the file byte-identical.
        let before = fs::read_to_string(&frontier_path).expect("before");
        let code = run_tas_pixel_baseline(
            &report_path,
            &frontier_path,
            "any-percent-published",
            true,
            false,
        )
        .expect("no-op update");
        assert_eq!(code, 0);
        let after = fs::read_to_string(&frontier_path).expect("after");
        assert_eq!(before, after);
    }
}
