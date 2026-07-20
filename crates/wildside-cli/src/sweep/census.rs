//! Sweep-blocker census: joins the blocking state diffs of a sweep report
//! onto per-level enemy rosters so blocking differences can be attributed to
//! enemy families. The report's `StateDiff` records name only global fields
//! (`number_objects`, `kid_y_pos`, ...) with no object type; the census
//! recovers the missing attribution axis mechanically by comparing which
//! families appear in failing versus passing levels.

use super::*;
use wildside_core::layout_object_family_name;
use wildside_data::EnemyLayout;

/// Blocking categories: everything that fails a level. `Advisory` and
/// `OracleLimitation` diffs never block, so they are excluded from the census.
pub(crate) fn is_blocking(category: DivergenceCategory) -> bool {
    matches!(
        category,
        DivergenceCategory::Authoritative
            | DivergenceCategory::Accumulation
            | DivergenceCategory::Rng
    )
}

/// One enemy family present in a level, with its layout-object count.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct RosterEntry {
    pub(crate) family: String,
    pub(crate) count: usize,
}

/// Per-level census record: blocking diffs joined to the enemy roster.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct CensusLevel {
    pub(crate) level_id: u8,
    pub(crate) slug: String,
    pub(crate) passed: bool,
    pub(crate) blocking_diffs: Vec<StateDiff>,
    pub(crate) roster: Vec<RosterEntry>,
    pub(crate) roster_available: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct FieldCount {
    pub(crate) field: String,
    pub(crate) count: usize,
}

/// Family presence across failing vs passing levels. `overrepresentation` is
/// the Laplace-smoothed ratio of the family's failing-level share to its
/// passing-level share: a family common in failures but rare in passes is a
/// suspect worth reading the ASM for.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct FamilyFailureRate {
    pub(crate) family: String,
    pub(crate) failing_levels: usize,
    pub(crate) passing_levels: usize,
    pub(crate) failing_level_ids: Vec<String>,
    /// Total blocking diffs across the failing levels containing this family.
    pub(crate) blocking_diffs_in_failing_levels: usize,
    pub(crate) overrepresentation: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct CensusAggregates {
    pub(crate) field_histogram_blocking_only: Vec<FieldCount>,
    pub(crate) family_failure_rate: Vec<FamilyFailureRate>,
}

/// Census artefact written to `.cache/audit/sweep-census.json`. Vec-of-records
/// throughout (not maps) for stable ordering across runs.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct SweepCensus {
    pub(crate) run_id: String,
    pub(crate) source_report: String,
    pub(crate) levels: Vec<CensusLevel>,
    pub(crate) aggregates: CensusAggregates,
}

/// Family roster for a level's enemy layout. Segments duplicate the top-level
/// object list (each segment carries its own slot table), so iterate segments
/// when present and fall back to the top-level objects otherwise — mirroring
/// how `inspect-level` reads the layout.
pub(crate) fn roster_from_layout(layout: &EnemyLayout) -> Vec<RosterEntry> {
    let mut counts: BTreeMap<String, usize> = BTreeMap::new();
    if layout.segments.is_empty() {
        for object in &layout.objects {
            let family = layout_object_family_name(&layout.enemy_type_slots, object);
            *counts.entry(family).or_default() += 1;
        }
    } else {
        for segment in &layout.segments {
            for object in &segment.objects {
                let family = layout_object_family_name(&segment.enemy_type_slots, object);
                *counts.entry(family).or_default() += 1;
            }
        }
    }
    counts
        .into_iter()
        .map(|(family, count)| RosterEntry { family, count })
        .collect()
}

/// Pure aggregation core: join a sweep report with per-level rosters
/// (`None` = roster unavailable for that level) into the census artefact.
pub(crate) fn build_census(
    report: &SweepReport,
    run_id: &str,
    source_report: &str,
    rosters: &BTreeMap<u8, Option<Vec<RosterEntry>>>,
) -> SweepCensus {
    let levels: Vec<CensusLevel> = report
        .levels
        .iter()
        .map(|level| {
            let roster = rosters.get(&level.level_id).cloned().flatten();
            CensusLevel {
                level_id: level.level_id,
                slug: level.slug.clone(),
                passed: level.passed,
                blocking_diffs: level
                    .state_diffs
                    .iter()
                    .filter(|diff| is_blocking(diff.category))
                    .cloned()
                    .collect(),
                roster_available: roster.is_some(),
                roster: roster.unwrap_or_default(),
            }
        })
        .collect();

    let mut field_counts: BTreeMap<String, usize> = BTreeMap::new();
    for level in &levels {
        for diff in &level.blocking_diffs {
            *field_counts.entry(diff.field.clone()).or_default() += 1;
        }
    }
    let mut field_histogram: Vec<FieldCount> = field_counts
        .into_iter()
        .map(|(field, count)| FieldCount { field, count })
        .collect();
    field_histogram.sort_by(|a, b| b.count.cmp(&a.count).then(a.field.cmp(&b.field)));

    // Family failure rate over levels with a roster. A level counts as
    // "failing" here when it has blocking diffs, which matches the sweep's
    // pass/fail verdict for state-tier runs.
    let with_roster: Vec<&CensusLevel> = levels.iter().filter(|l| l.roster_available).collect();
    let total_failing = with_roster.iter().filter(|l| !l.passed).count();
    let total_passing = with_roster.len() - total_failing;
    let mut families: BTreeSet<String> = BTreeSet::new();
    for level in &with_roster {
        for entry in &level.roster {
            families.insert(entry.family.clone());
        }
    }
    let mut family_failure_rate: Vec<FamilyFailureRate> = families
        .into_iter()
        .map(|family| {
            let contains =
                |level: &&&CensusLevel| -> bool { level.roster.iter().any(|e| e.family == family) };
            let failing: Vec<&&CensusLevel> = with_roster
                .iter()
                .filter(|l| !l.passed)
                .filter(contains)
                .collect();
            let passing = with_roster
                .iter()
                .filter(|l| l.passed)
                .filter(contains)
                .count();
            // Laplace smoothing keeps zero-passing families finite and
            // comparable instead of collapsing them all to infinity.
            let failing_share = (failing.len() + 1) as f64 / (total_failing + 1) as f64;
            let passing_share = (passing + 1) as f64 / (total_passing + 1) as f64;
            FamilyFailureRate {
                failing_levels: failing.len(),
                passing_levels: passing,
                failing_level_ids: failing
                    .iter()
                    .map(|l| format!("{:#04x}", l.level_id))
                    .collect(),
                blocking_diffs_in_failing_levels: failing
                    .iter()
                    .map(|l| l.blocking_diffs.len())
                    .sum(),
                overrepresentation: failing_share / passing_share,
                family,
            }
        })
        .collect();
    family_failure_rate.sort_by(|a, b| {
        b.overrepresentation
            .total_cmp(&a.overrepresentation)
            .then(b.failing_levels.cmp(&a.failing_levels))
            .then(a.family.cmp(&b.family))
    });

    SweepCensus {
        run_id: run_id.to_string(),
        source_report: source_report.to_string(),
        levels,
        aggregates: CensusAggregates {
            field_histogram_blocking_only: field_histogram,
            family_failure_rate,
        },
    }
}

/// Resolve which sweep run to census: explicit `--run` id, else the `latest`
/// symlink, else the highest numeric run directory.
pub(crate) fn resolve_run_dir(sweep_runs_dir: &Path, run: Option<&str>) -> Result<PathBuf> {
    if let Some(run_id) = run {
        let dir = sweep_runs_dir.join(run_id);
        if !dir.join("report.json").is_file() {
            bail!(
                "sweep run {run_id} has no report.json under {}",
                dir.display()
            );
        }
        return Ok(dir);
    }
    let latest = sweep_runs_dir.join("latest");
    if latest.join("report.json").is_file() {
        return Ok(latest.canonicalize().unwrap_or(latest));
    }
    let mut numeric_runs: Vec<(u64, PathBuf)> = Vec::new();
    for entry in fs::read_dir(sweep_runs_dir)
        .with_context(|| format!("failed to read {}", sweep_runs_dir.display()))?
    {
        let entry = entry?;
        let name = entry.file_name();
        if let Ok(id) = name.to_string_lossy().parse::<u64>() {
            let dir = entry.path();
            if dir.join("report.json").is_file() {
                numeric_runs.push((id, dir));
            }
        }
    }
    numeric_runs
        .into_iter()
        .max_by_key(|(id, _)| *id)
        .map(|(_, dir)| dir)
        .ok_or_else(|| {
            anyhow!(
                "no sweep run with a report.json under {} — run `mise run sweep` first",
                sweep_runs_dir.display()
            )
        })
}

/// Imperative shell: resolve the run, load the report, join rosters (unless
/// `manifest` is `None`), print the tables, and write the JSON artefact.
pub(crate) fn run_sweep_census(
    repo_root: &Path,
    manifest: Option<&AssetManifest>,
    run: Option<&str>,
    output: Option<&Path>,
) -> Result<()> {
    let run_dir = resolve_run_dir(&repo_root.join(".cache/sweep-runs"), run)?;
    let run_id = run_dir.file_name().map_or_else(
        || run_dir.display().to_string(),
        |n| n.to_string_lossy().into_owned(),
    );
    let report_path = run_dir.join("report.json");
    let report: SweepReport = serde_json::from_str(
        &fs::read_to_string(&report_path)
            .with_context(|| format!("failed to read {}", report_path.display()))?,
    )
    .with_context(|| format!("failed to parse {}", report_path.display()))?;

    // Print the run identity up front so a mis-targeted default (e.g. a
    // `latest` symlink pointing at a single-level run) is obvious.
    println!(
        "sweep census: run {run_id} — {} levels ({} passed, {} failed)",
        report.levels.len(),
        report.passed,
        report.failed
    );

    let mut rosters: BTreeMap<u8, Option<Vec<RosterEntry>>> = BTreeMap::new();
    if let Some(manifest) = manifest {
        for level in &report.levels {
            // A level without an enemy layout resource (e.g. 0x41 The Crypt)
            // has an EMPTY roster — a real "not enemy-related" signal — while
            // a bundle that fails to load has NO roster and is excluded from
            // the family table.
            let roster = load_level_asset_bundle(manifest, level.level_id)
                .ok()
                .map(|bundle| {
                    bundle
                        .parsed_enemy_layout
                        .as_ref()
                        .map(roster_from_layout)
                        .unwrap_or_default()
                });
            if roster.is_none() {
                println!(
                    "  (no enemy roster for level {:#04x} — asset bundle failed to load)",
                    level.level_id
                );
            }
            rosters.insert(level.level_id, roster);
        }
    } else {
        println!("  (--no-roster: family attribution skipped, field histogram only)");
        for level in &report.levels {
            rosters.insert(level.level_id, None);
        }
    }

    let census = build_census(
        &report,
        &run_id,
        &report_path.display().to_string(),
        &rosters,
    );

    let blocking_total: usize = census.levels.iter().map(|l| l.blocking_diffs.len()).sum();
    println!("blocking diffs: {blocking_total}\n");

    println!("failing levels (blocking diffs, roster):");
    let mut failing: Vec<&CensusLevel> = census.levels.iter().filter(|l| !l.passed).collect();
    failing.sort_by_key(|l| std::cmp::Reverse(l.blocking_diffs.len()));
    for level in &failing {
        let roster = if level.roster_available {
            level
                .roster
                .iter()
                .map(|e| format!("{}x{}", e.family, e.count))
                .collect::<Vec<_>>()
                .join(", ")
        } else {
            "<roster unavailable>".to_string()
        };
        println!(
            "  {:#04x} {:<28} {:>3}  [{}]",
            level.level_id,
            level.slug,
            level.blocking_diffs.len(),
            roster
        );
    }

    println!("\nblocking-field histogram:");
    for entry in &census.aggregates.field_histogram_blocking_only {
        println!("  {:>4}  {}", entry.count, entry.field);
    }

    if manifest.is_some() {
        println!(
            "\nfamily failure rate (failing/passing levels containing family, sorted by overrepresentation):"
        );
        for row in &census.aggregates.family_failure_rate {
            println!(
                "  {:<20} fail {:>2}  pass {:>2}  blockers {:>3}  ratio {:>5.2}  [{}]",
                row.family,
                row.failing_levels,
                row.passing_levels,
                row.blocking_diffs_in_failing_levels,
                row.overrepresentation,
                row.failing_level_ids.join(", ")
            );
        }
    }

    let output_path = output.map_or_else(
        || repo_root.join(".cache/audit/sweep-census.json"),
        Path::to_path_buf,
    );
    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("failed to create {}", parent.display()))?;
    }
    fs::write(&output_path, serde_json::to_string_pretty(&census)?)
        .with_context(|| format!("failed to write {}", output_path.display()))?;
    println!("\ncensus written to {}", output_path.display());
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn diff(field: &str, category: DivergenceCategory) -> StateDiff {
        StateDiff {
            checkpoint: "idle_tail".to_string(),
            field: field.to_string(),
            native_value: "1".to_string(),
            oracle_value: "2".to_string(),
            category,
        }
    }

    fn level(level_id: u8, passed: bool, state_diffs: Vec<StateDiff>) -> LevelSweepResult {
        LevelSweepResult {
            level_id,
            slug: format!("level_{level_id:02x}"),
            tier: SweepTier::State,
            passed,
            state_diffs,
            mae: None,
            pixel_mismatches: None,
            diff_images: Vec::new(),
            error: None,
            divergence_summary: DivergenceSummary::default(),
            checksum_mismatch: None,
            total_checkpoints: 4,
            masked_checkpoints: 0,
        }
    }

    fn roster(entries: &[(&str, usize)]) -> Vec<RosterEntry> {
        entries
            .iter()
            .map(|(family, count)| RosterEntry {
                family: (*family).to_string(),
                count: *count,
            })
            .collect()
    }

    #[test]
    fn blocking_classification_covers_all_categories() {
        let cases = [
            (DivergenceCategory::Authoritative, true),
            (DivergenceCategory::Accumulation, true),
            (DivergenceCategory::Rng, true),
            (DivergenceCategory::Advisory, false),
            (DivergenceCategory::OracleLimitation, false),
        ];
        for (category, expected) in cases {
            assert_eq!(
                is_blocking(category),
                expected,
                "{category:?} blocking classification"
            );
        }
    }

    #[test]
    fn build_census_aggregates_blocking_diffs_and_ranks_suspect_family() {
        let report = SweepReport {
            timestamp: "0".to_string(),
            kid_core_hash: None,
            levels: vec![
                level(
                    0x01,
                    false,
                    vec![
                        diff("kid_y_pos", DivergenceCategory::Authoritative),
                        diff("number_objects", DivergenceCategory::Authoritative),
                        diff("number_objects", DivergenceCategory::OracleLimitation),
                    ],
                ),
                level(
                    0x02,
                    false,
                    vec![
                        diff("number_objects", DivergenceCategory::Accumulation),
                        diff("rng_offset", DivergenceCategory::Rng),
                        diff("kid_x_pos", DivergenceCategory::Advisory),
                    ],
                ),
                level(0x03, true, vec![]),
            ],
            passed: 1,
            unverified: 0,
            failed: 2,
            errors: 0,
            divergence_summary: DivergenceSummary::default(),
            field_summary: Vec::new(),
            release_gate_focus: Vec::new(),
        };
        let rosters: BTreeMap<u8, Option<Vec<RosterEntry>>> = BTreeMap::from([
            (0x01, Some(roster(&[("Tank", 2), ("Goat", 1)]))),
            (0x02, Some(roster(&[("Tank", 1)]))),
            (0x03, Some(roster(&[("Goat", 3)]))),
        ]);

        let census = build_census(&report, "12345", "report.json", &rosters);

        // Advisory + oracle_limitation diffs are excluded.
        let blocking_total: usize = census.levels.iter().map(|l| l.blocking_diffs.len()).sum();
        assert_eq!(blocking_total, 4);
        assert_eq!(
            census.aggregates.field_histogram_blocking_only,
            vec![
                FieldCount {
                    field: "number_objects".to_string(),
                    count: 2
                },
                FieldCount {
                    field: "kid_y_pos".to_string(),
                    count: 1
                },
                FieldCount {
                    field: "rng_offset".to_string(),
                    count: 1
                },
            ]
        );

        // Tank is in both failing levels and no passing level: top suspect.
        // Goat is in one failing and one passing level: ranked below.
        let families: Vec<&str> = census
            .aggregates
            .family_failure_rate
            .iter()
            .map(|r| r.family.as_str())
            .collect();
        assert_eq!(families, vec!["Tank", "Goat"]);
        let tank = &census.aggregates.family_failure_rate[0];
        assert_eq!(tank.failing_levels, 2);
        assert_eq!(tank.passing_levels, 0);
        assert_eq!(tank.blocking_diffs_in_failing_levels, 4);
        assert_eq!(tank.failing_level_ids, vec!["0x01", "0x02"]);
        let goat = &census.aggregates.family_failure_rate[1];
        assert!(tank.overrepresentation > goat.overrepresentation);
    }

    #[test]
    fn build_census_marks_levels_without_roster() {
        let report = SweepReport {
            timestamp: "0".to_string(),
            kid_core_hash: None,
            levels: vec![level(
                0x05,
                false,
                vec![diff("kid_y_pos", DivergenceCategory::Authoritative)],
            )],
            passed: 0,
            unverified: 0,
            failed: 1,
            errors: 0,
            divergence_summary: DivergenceSummary::default(),
            field_summary: Vec::new(),
            release_gate_focus: Vec::new(),
        };
        let rosters: BTreeMap<u8, Option<Vec<RosterEntry>>> = BTreeMap::from([(0x05, None)]);
        let census = build_census(&report, "1", "r.json", &rosters);
        assert!(!census.levels[0].roster_available);
        assert!(census.levels[0].roster.is_empty());
        // No level has a roster, so the family table is empty rather than
        // built from a biased subset.
        assert!(census.aggregates.family_failure_rate.is_empty());
    }

    #[test]
    fn resolve_run_dir_prefers_explicit_then_latest_then_highest_numeric() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let runs = tmp.path();
        for id in ["100", "250", "90"] {
            let dir = runs.join(id);
            fs::create_dir_all(&dir).unwrap();
            fs::write(dir.join("report.json"), "{}").unwrap();
        }
        // A numerically-highest dir without a report is skipped.
        fs::create_dir_all(runs.join("999")).unwrap();

        // Explicit run wins.
        assert_eq!(
            resolve_run_dir(runs, Some("100")).unwrap(),
            runs.join("100")
        );
        // Explicit run without a report errors.
        assert!(resolve_run_dir(runs, Some("999")).is_err());
        assert!(resolve_run_dir(runs, Some("missing")).is_err());

        // No latest symlink: highest numeric run with a report.
        assert_eq!(resolve_run_dir(runs, None).unwrap(), runs.join("250"));

        // latest symlink wins over the numeric fallback.
        #[cfg(unix)]
        {
            std::os::unix::fs::symlink(runs.join("100"), runs.join("latest")).unwrap();
            assert_eq!(
                resolve_run_dir(runs, None)
                    .unwrap()
                    .file_name()
                    .unwrap()
                    .to_string_lossy(),
                "100"
            );
        }
    }
}
