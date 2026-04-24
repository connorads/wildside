//! Pixel-diff ranker: operator-initiated triage of the worst visual
//! divergences from a frames-tier sweep.
//!
//! Reads `<sweep-run>/report.json`, picks the top N levels ranked by
//! `pixel_mismatches` (tiebreak: `mae`), copies their `native_*.png`
//! and `diff_*.png` artefacts into a flat review directory, and writes
//! a manifest. Filters out levels whose only non-`oracle_limitation`
//! signal is empty — per the recovery plan's gotcha #5, pre-C8 the
//! top-N is otherwise polluted by oracle-pause noise.
//!
//! No autonomy hook — runs on demand, produces PNGs for a human (or a
//! future Claude session with vision) to classify.

use super::*;
use std::cmp::Ordering;

#[derive(Debug, Clone, Serialize)]
pub(crate) struct PixelRankEntry {
    pub(crate) rank: usize,
    pub(crate) level_id: u8,
    pub(crate) slug: String,
    pub(crate) mae: Option<f32>,
    pub(crate) pixel_mismatches: Option<usize>,
    pub(crate) authoritative: usize,
    pub(crate) advisory: usize,
    pub(crate) oracle_limitation: usize,
    pub(crate) native_pngs: Vec<String>,
    pub(crate) diff_pngs: Vec<String>,
    pub(crate) output_dir: String,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct PixelRankManifest {
    pub(crate) source_report: String,
    pub(crate) top_n_requested: usize,
    pub(crate) entries: Vec<PixelRankEntry>,
    pub(crate) skipped_oracle_limitation_only: usize,
    pub(crate) skipped_no_pixel_data: usize,
}

pub(crate) fn run_rank_pixel_diffs(
    report_path: &Path,
    top_n: usize,
    output_dir: &Path,
) -> Result<PixelRankManifest> {
    let report_json = fs::read_to_string(report_path)
        .with_context(|| format!("failed to read sweep report at {}", report_path.display()))?;
    let report: SweepReport = serde_json::from_str(&report_json)
        .with_context(|| format!("failed to parse sweep report at {}", report_path.display()))?;

    let report_dir = report_path.parent().ok_or_else(|| {
        anyhow!(
            "sweep report path has no parent directory: {}",
            report_path.display()
        )
    })?;

    let manifest = build_manifest(
        &report,
        report_path,
        report_dir,
        top_n,
        output_dir,
        copy_png,
    )?;

    fs::create_dir_all(output_dir)
        .with_context(|| format!("failed to create {}", output_dir.display()))?;
    let manifest_path = output_dir.join("manifest.json");
    let manifest_json = serde_json::to_vec_pretty(&manifest)?;
    fs::write(&manifest_path, manifest_json)
        .with_context(|| format!("failed to write {}", manifest_path.display()))?;

    Ok(manifest)
}

type CopyFn = fn(&Path, &Path) -> Result<()>;

fn copy_png(src: &Path, dst: &Path) -> Result<()> {
    fs::copy(src, dst)
        .with_context(|| format!("failed to copy {} to {}", src.display(), dst.display()))?;
    Ok(())
}

fn build_manifest(
    report: &SweepReport,
    report_path: &Path,
    report_dir: &Path,
    top_n: usize,
    output_dir: &Path,
    copy_fn: CopyFn,
) -> Result<PixelRankManifest> {
    let mut skipped_no_pixel_data = 0usize;
    let mut skipped_oracle_limitation_only = 0usize;

    // Filter: must have non-zero pixel mismatches AND at least one state
    // diff that isn't `oracle_limitation` — otherwise it's just oracle-pause
    // noise dressed as a pixel divergence.
    let mut candidates: Vec<&LevelSweepResult> = report
        .levels
        .iter()
        .filter(|r| match r.pixel_mismatches {
            Some(n) if n > 0 => true,
            _ => {
                skipped_no_pixel_data += 1;
                false
            }
        })
        .filter(|r| {
            if has_meaningful_divergence(r) {
                true
            } else {
                skipped_oracle_limitation_only += 1;
                false
            }
        })
        .collect();

    candidates.sort_by(|a, b| {
        let a_px = a.pixel_mismatches.unwrap_or(0);
        let b_px = b.pixel_mismatches.unwrap_or(0);
        b_px.cmp(&a_px)
            .then_with(|| f32_desc(a.mae, b.mae))
            .then_with(|| a.level_id.cmp(&b.level_id))
    });

    let mut entries = Vec::with_capacity(top_n.min(candidates.len()));
    for (idx, result) in candidates.iter().take(top_n).enumerate() {
        let rank = idx + 1;
        let entry_dir_name = format!("{rank:02}_level{:02x}_{}", result.level_id, result.slug);
        let entry_dir = output_dir.join(&entry_dir_name);
        fs::create_dir_all(&entry_dir)
            .with_context(|| format!("failed to create {}", entry_dir.display()))?;

        let level_dir = report_dir
            .join("levels")
            .join(format!("{:02x}", result.level_id));

        let (native_pngs, diff_pngs) = copy_level_pngs(&level_dir, &entry_dir, copy_fn)?;

        entries.push(PixelRankEntry {
            rank,
            level_id: result.level_id,
            slug: result.slug.clone(),
            mae: result.mae,
            pixel_mismatches: result.pixel_mismatches,
            authoritative: result.divergence_summary.authoritative,
            advisory: result.divergence_summary.advisory,
            oracle_limitation: result.divergence_summary.oracle_limitation,
            native_pngs,
            diff_pngs,
            output_dir: entry_dir_name,
        });
    }

    Ok(PixelRankManifest {
        source_report: report_path.display().to_string(),
        top_n_requested: top_n,
        entries,
        skipped_oracle_limitation_only,
        skipped_no_pixel_data,
    })
}

/// Does this level have any divergence signal beyond oracle-pause noise?
/// `number_objects`/`number_hitpoints`/`number_lives` fall under advisory,
/// which already excludes them from the pixel-review stream.
fn has_meaningful_divergence(r: &LevelSweepResult) -> bool {
    r.divergence_summary.authoritative > 0
        || r.divergence_summary.accumulation > 0
        || r.divergence_summary.rng > 0
        || r.state_diffs
            .iter()
            .any(|d| !matches!(d.category, DivergenceCategory::OracleLimitation))
}

fn f32_desc(a: Option<f32>, b: Option<f32>) -> Ordering {
    let a = a.unwrap_or(0.0);
    let b = b.unwrap_or(0.0);
    b.partial_cmp(&a).unwrap_or(Ordering::Equal)
}

fn copy_level_pngs(
    level_dir: &Path,
    entry_dir: &Path,
    copy_fn: CopyFn,
) -> Result<(Vec<String>, Vec<String>)> {
    let mut native_pngs = Vec::new();
    let mut diff_pngs = Vec::new();

    if !level_dir.exists() {
        return Ok((native_pngs, diff_pngs));
    }

    for entry in fs::read_dir(level_dir)
        .with_context(|| format!("failed to read {}", level_dir.display()))?
    {
        let entry = entry?;
        let path = entry.path();
        let file_name = match path.file_name().and_then(|n| n.to_str()) {
            Some(n) => n.to_string(),
            None => continue,
        };
        let is_png = path
            .extension()
            .is_some_and(|ext| ext.eq_ignore_ascii_case("png"));
        if !is_png {
            continue;
        }
        let dest = entry_dir.join(&file_name);
        if file_name.starts_with("native_") {
            copy_fn(&path, &dest)?;
            native_pngs.push(file_name);
        } else if file_name.starts_with("diff_") {
            copy_fn(&path, &dest)?;
            diff_pngs.push(file_name);
        }
    }

    native_pngs.sort();
    diff_pngs.sort();
    Ok((native_pngs, diff_pngs))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn result(
        level_id: u8,
        mismatches: Option<usize>,
        mae: Option<f32>,
        authoritative: usize,
        oracle_limitation: usize,
    ) -> LevelSweepResult {
        let mut state_diffs = Vec::new();
        for _ in 0..authoritative {
            state_diffs.push(StateDiff {
                checkpoint: "level_settle".into(),
                field: "kid_x_pos".into(),
                native_value: "0".into(),
                oracle_value: "1".into(),
                category: DivergenceCategory::Authoritative,
            });
        }
        for _ in 0..oracle_limitation {
            state_diffs.push(StateDiff {
                checkpoint: "level_settle".into(),
                field: "game_paused".into(),
                native_value: "false".into(),
                oracle_value: "true".into(),
                category: DivergenceCategory::OracleLimitation,
            });
        }
        let summary = DivergenceSummary {
            authoritative,
            oracle_limitation,
            ..DivergenceSummary::default()
        };
        LevelSweepResult {
            level_id,
            slug: format!("idle_level_{level_id:02}"),
            tier: SweepTier::Frames,
            passed: false,
            state_diffs,
            mae,
            pixel_mismatches: mismatches,
            diff_images: Vec::new(),
            error: None,
            divergence_summary: summary,
        }
    }

    fn empty_report(levels: Vec<LevelSweepResult>) -> SweepReport {
        SweepReport {
            timestamp: "0".into(),
            kid_core_hash: None,
            levels,
            passed: 0,
            failed: 0,
            errors: 0,
            divergence_summary: DivergenceSummary::default(),
            field_summary: Vec::new(),
            release_gate_focus: Vec::new(),
        }
    }

    #[allow(clippy::unnecessary_wraps)]
    fn noop_copy(_src: &Path, _dst: &Path) -> Result<()> {
        Ok(())
    }

    #[test]
    fn skips_levels_with_no_pixel_data() {
        let report = empty_report(vec![
            result(0, None, None, 0, 0),
            result(1, Some(0), Some(0.0), 0, 0),
            result(2, Some(500), Some(12.5), 1, 0),
        ]);
        let tmp = tempfile::tempdir().unwrap();
        let manifest = build_manifest(
            &report,
            Path::new("report.json"),
            tmp.path(),
            10,
            tmp.path(),
            noop_copy,
        )
        .unwrap();
        assert_eq!(manifest.skipped_no_pixel_data, 2);
        assert_eq!(manifest.entries.len(), 1);
        assert_eq!(manifest.entries[0].level_id, 2);
    }

    #[test]
    fn skips_oracle_limitation_only_levels() {
        // Level with pixel diff but ONLY oracle_limitation state_diffs —
        // the gotcha #5 case. Should be skipped to avoid polluting top-N.
        let report = empty_report(vec![
            result(0, Some(1000), Some(5.0), 0, 8),
            result(1, Some(100), Some(2.0), 1, 0),
        ]);
        let tmp = tempfile::tempdir().unwrap();
        let manifest = build_manifest(
            &report,
            Path::new("report.json"),
            tmp.path(),
            10,
            tmp.path(),
            noop_copy,
        )
        .unwrap();
        assert_eq!(manifest.skipped_oracle_limitation_only, 1);
        assert_eq!(manifest.entries.len(), 1);
        assert_eq!(manifest.entries[0].level_id, 1);
    }

    #[test]
    fn ranks_by_pixel_mismatches_desc_then_mae_desc() {
        let report = empty_report(vec![
            result(0, Some(100), Some(1.0), 1, 0),
            result(1, Some(500), Some(5.0), 1, 0),
            result(2, Some(500), Some(9.0), 1, 0),
            result(3, Some(300), Some(3.0), 1, 0),
        ]);
        let tmp = tempfile::tempdir().unwrap();
        let manifest = build_manifest(
            &report,
            Path::new("report.json"),
            tmp.path(),
            4,
            tmp.path(),
            noop_copy,
        )
        .unwrap();
        let ids: Vec<u8> = manifest.entries.iter().map(|e| e.level_id).collect();
        assert_eq!(ids, vec![2, 1, 3, 0]);
        assert_eq!(manifest.entries[0].rank, 1);
    }

    #[test]
    fn respects_top_n_limit() {
        let report = empty_report(vec![
            result(0, Some(100), Some(1.0), 1, 0),
            result(1, Some(200), Some(2.0), 1, 0),
            result(2, Some(300), Some(3.0), 1, 0),
        ]);
        let tmp = tempfile::tempdir().unwrap();
        let manifest = build_manifest(
            &report,
            Path::new("report.json"),
            tmp.path(),
            2,
            tmp.path(),
            noop_copy,
        )
        .unwrap();
        assert_eq!(manifest.entries.len(), 2);
        assert_eq!(manifest.entries[0].level_id, 2);
        assert_eq!(manifest.entries[1].level_id, 1);
    }

    #[test]
    fn entry_dir_name_is_rank_level_slug() {
        let report = empty_report(vec![result(0x2a, Some(99), Some(1.0), 1, 0)]);
        let tmp = tempfile::tempdir().unwrap();
        let manifest = build_manifest(
            &report,
            Path::new("report.json"),
            tmp.path(),
            1,
            tmp.path(),
            noop_copy,
        )
        .unwrap();
        assert_eq!(manifest.entries[0].output_dir, "01_level2a_idle_level_42");
    }

    #[test]
    fn copies_only_native_and_diff_pngs() {
        let report = empty_report(vec![result(5, Some(50), Some(1.0), 1, 0)]);
        let tmp = tempfile::tempdir().unwrap();
        let levels_dir = tmp.path().join("levels/05");
        fs::create_dir_all(&levels_dir).unwrap();
        fs::write(levels_dir.join("native_level_settle.png"), b"fake").unwrap();
        fs::write(levels_dir.join("native_idle_tail.png"), b"fake").unwrap();
        fs::write(levels_dir.join("diff_level_settle.png"), b"fake").unwrap();
        fs::write(levels_dir.join("oracle_level_settle.png"), b"fake").unwrap();
        fs::write(levels_dir.join("trace.txt"), b"fake").unwrap();

        let out = tempfile::tempdir().unwrap();
        let manifest = build_manifest(
            &report,
            &tmp.path().join("report.json"),
            tmp.path(),
            1,
            out.path(),
            copy_png,
        )
        .unwrap();
        let entry = &manifest.entries[0];
        assert_eq!(entry.native_pngs.len(), 2);
        assert_eq!(entry.diff_pngs.len(), 1);
        let entry_dir = out.path().join(&entry.output_dir);
        assert!(entry_dir.join("native_level_settle.png").exists());
        assert!(entry_dir.join("native_idle_tail.png").exists());
        assert!(entry_dir.join("diff_level_settle.png").exists());
        assert!(!entry_dir.join("oracle_level_settle.png").exists());
        assert!(!entry_dir.join("trace.txt").exists());
    }
}
