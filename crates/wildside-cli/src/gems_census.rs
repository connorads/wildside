//! GEMS mismatch census (#434): cluster the full classified outcome corpus
//! from `verify-tas --compare-gems --gems-mismatch-report` so the 36k-frame
//! timing-debt tail decomposes into a handful of reviewable shapes instead
//! of one opaque count. Cluster key: classification + the sorted
//! diverging-field tuple + the bank-address pair when the bank itself
//! diverges. Contiguous frame runs are summarised per cluster because the
//! debt is expected to be long phase-locked stretches, not scattered noise;
//! run structure is what selects the bounded replay spans for #437.

use anyhow::{Context, Result, bail};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::Path;
use wildside_core::GemsFrameSignature;

/// Input: the JSON written by `write_gems_mismatch_report` (tas.rs). Owned
/// deserialization types keep the census decoupled from `tas_gems`
/// internals (`diverging_fields` are `&'static str` there).
#[derive(Debug, Clone, Deserialize)]
pub(crate) struct GemsMismatchReport {
    pub(crate) slug: String,
    pub(crate) projection: String,
    pub(crate) compared_frames: usize,
    pub(crate) counts: GemsCorpusCounts,
    pub(crate) records: Vec<GemsCorpusRecord>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct GemsCorpusCounts {
    pub(crate) genuine: usize,
    pub(crate) oracle_limitation: usize,
    pub(crate) advisory: usize,
}

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct GemsCorpusRecord {
    pub(crate) frame_index: usize,
    pub(crate) classification: String,
    pub(crate) diverging_fields: Vec<String>,
    pub(crate) native: Option<GemsFrameSignature>,
    pub(crate) oracle: Option<GemsFrameSignature>,
}

/// One census cluster. `example_frames` is capped ([`EXAMPLE_FRAME_CAP`]);
/// `runs` summarises the contiguous frame-index structure.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct GemsCensusCluster {
    pub(crate) classification: String,
    pub(crate) diverging_fields: Vec<String>,
    /// `Some((native, oracle))` bank pair when `bank_address` diverges —
    /// distinct bank pairs are distinct hardware states, so they cluster
    /// separately.
    pub(crate) bank_pair: Option<(u32, u32)>,
    pub(crate) record_count: usize,
    pub(crate) first_frame: usize,
    pub(crate) last_frame: usize,
    pub(crate) run_count: usize,
    pub(crate) longest_run: usize,
    pub(crate) example_frames: Vec<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct GemsCensus {
    pub(crate) slug: String,
    pub(crate) projection: String,
    pub(crate) source_report: String,
    pub(crate) compared_frames: usize,
    pub(crate) counts: GemsCorpusCounts,
    pub(crate) clusters: Vec<GemsCensusCluster>,
}

pub(crate) const EXAMPLE_FRAME_CAP: usize = 10;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct ClusterKey {
    classification: String,
    diverging_fields: Vec<String>,
    bank_pair: Option<(u32, u32)>,
}

fn cluster_key(record: &GemsCorpusRecord) -> ClusterKey {
    let mut diverging_fields = record.diverging_fields.clone();
    diverging_fields.sort_unstable();
    let bank_pair = record
        .diverging_fields
        .iter()
        .any(|field| field == "bank_address")
        .then(|| {
            (
                record.native.map_or(0, |s| s.bank_address),
                record.oracle.map_or(0, |s| s.bank_address),
            )
        });
    ClusterKey {
        classification: record.classification.clone(),
        diverging_fields,
        bank_pair,
    }
}

/// Contiguous ascending runs in a sorted frame list: `(run_count, longest)`.
fn run_structure(frames: &[usize]) -> (usize, usize) {
    let mut run_count = 0usize;
    let mut longest = 0usize;
    let mut current = 0usize;
    let mut previous: Option<usize> = None;
    for &frame in frames {
        match previous {
            Some(prev) if frame == prev + 1 => current += 1,
            _ => {
                run_count += 1;
                current = 1;
            }
        }
        longest = longest.max(current);
        previous = Some(frame);
    }
    (run_count, longest)
}

/// Pure aggregation core. Fails when the record classes stop reconciling
/// with the report's own counts. Note the independent collector-drift
/// guard lives in `write_gems_mismatch_report` (corpus classes vs the
/// comparison's own counters); this check only rejects corrupted or
/// foreign report files, since a writer-produced report derives both
/// operands from the same records.
pub(crate) fn build_gems_census(
    report: &GemsMismatchReport,
    source_report: &str,
) -> Result<GemsCensus> {
    let class_count = |class: &str| {
        report
            .records
            .iter()
            .filter(|record| record.classification == class)
            .count()
    };
    let observed = GemsCorpusCounts {
        genuine: class_count("genuine"),
        oracle_limitation: class_count("oracle_limitation"),
        advisory: class_count("advisory"),
    };
    if observed != report.counts {
        bail!(
            "census reconciliation failed: records {observed:?} vs report counts {:?}",
            report.counts
        );
    }

    let mut grouped: BTreeMap<ClusterKey, Vec<usize>> = BTreeMap::new();
    for record in &report.records {
        grouped
            .entry(cluster_key(record))
            .or_default()
            .push(record.frame_index);
    }

    let mut clusters: Vec<GemsCensusCluster> = grouped
        .into_iter()
        .map(|(key, mut frames)| {
            frames.sort_unstable();
            let (run_count, longest_run) = run_structure(&frames);
            GemsCensusCluster {
                classification: key.classification,
                diverging_fields: key.diverging_fields,
                bank_pair: key.bank_pair,
                record_count: frames.len(),
                first_frame: *frames.first().expect("non-empty cluster"),
                last_frame: *frames.last().expect("non-empty cluster"),
                run_count,
                longest_run,
                example_frames: frames.iter().copied().take(EXAMPLE_FRAME_CAP).collect(),
            }
        })
        .collect();
    // Largest clusters first; key order breaks ties deterministically.
    clusters.sort_by(|a, b| {
        b.record_count.cmp(&a.record_count).then_with(|| {
            (&a.classification, &a.diverging_fields, a.bank_pair).cmp(&(
                &b.classification,
                &b.diverging_fields,
                b.bank_pair,
            ))
        })
    });

    Ok(GemsCensus {
        slug: report.slug.clone(),
        projection: report.projection.clone(),
        source_report: source_report.to_owned(),
        compared_frames: report.compared_frames,
        counts: report.counts,
        clusters,
    })
}

/// Shell runner: read the corpus report, build the census, write the
/// artefact, and print the top clusters.
pub(crate) fn run_gems_census(report_path: &Path, output_path: &Path) -> Result<()> {
    let content = std::fs::read_to_string(report_path)
        .with_context(|| format!("failed to read {}", report_path.display()))?;
    let report: GemsMismatchReport = serde_json::from_str(&content)
        .with_context(|| format!("failed to parse {}", report_path.display()))?;
    let census = build_gems_census(&report, &report_path.display().to_string())?;

    if let Some(parent) = output_path.parent()
        && !parent.as_os_str().is_empty()
    {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("failed to create {}", parent.display()))?;
    }
    std::fs::write(output_path, serde_json::to_vec_pretty(&census)?)
        .with_context(|| format!("failed to write {}", output_path.display()))?;

    println!(
        "gems-census: {} clusters over {} records ({} genuine, {} limitation, {} advisory) -> {}",
        census.clusters.len(),
        census.counts.genuine + census.counts.oracle_limitation + census.counts.advisory,
        census.counts.genuine,
        census.counts.oracle_limitation,
        census.counts.advisory,
        output_path.display()
    );
    for cluster in census.clusters.iter().take(10) {
        println!(
            "  {:>6} frames  {} [{}]{} runs={} longest={} span={}..{}",
            cluster.record_count,
            cluster.classification,
            cluster.diverging_fields.join(","),
            cluster
                .bank_pair
                .map_or_else(String::new, |(native, oracle)| format!(
                    " bank={native:#x}/{oracle:#x}"
                )),
            cluster.run_count,
            cluster.longest_run,
            cluster.first_frame,
            cluster.last_frame,
        );
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn signature(bank_address: u32) -> GemsFrameSignature {
        GemsFrameSignature {
            audio_ram_crc32: 1,
            bank_address,
            z80_pc: 2,
            z80_mclk_counter: 3,
            z80_wait_mclk_cycles: 4,
            pending_m68k_wait_cycles: 5,
            ym2612_mclk_counter: 6,
            status_bits: 0,
        }
    }

    fn record(
        frame_index: usize,
        classification: &str,
        fields: &[&str],
        banks: Option<(u32, u32)>,
    ) -> GemsCorpusRecord {
        GemsCorpusRecord {
            frame_index,
            classification: classification.to_owned(),
            diverging_fields: fields.iter().map(|&f| f.to_owned()).collect(),
            native: Some(signature(banks.map_or(0, |(native, _)| native))),
            oracle: Some(signature(banks.map_or(0, |(_, oracle)| oracle))),
        }
    }

    fn report(records: Vec<GemsCorpusRecord>, counts: GemsCorpusCounts) -> GemsMismatchReport {
        GemsMismatchReport {
            slug: "any-percent-published".to_owned(),
            projection: "gems_frame_signature_v1".to_owned(),
            compared_frames: 100,
            counts,
            records,
        }
    }

    #[test]
    fn clusters_by_sorted_fields_and_bank_pair_with_run_structure() {
        let records = vec![
            // One field-shape across frames 1,2,3 and 10 => 2 runs.
            record(3, "genuine", &["audio_ram_crc32", "z80_pc"], None),
            record(1, "genuine", &["z80_pc", "audio_ram_crc32"], None),
            record(2, "genuine", &["audio_ram_crc32", "z80_pc"], None),
            record(10, "genuine", &["audio_ram_crc32", "z80_pc"], None),
            // Same fields, bank diverges with two distinct pairs => two
            // separate clusters.
            record(20, "genuine", &["bank_address"], Some((0xE000, 0xA000))),
            record(21, "genuine", &["bank_address"], Some((0xE000, 0xA000))),
            record(30, "genuine", &["bank_address"], Some((0xC000, 0xA000))),
            // Same fields as the first shape but a different class stays a
            // separate cluster.
            record(5, "oracle_limitation", &["z80_pc"], None),
            record(6, "advisory", &["z80_mclk_counter"], None),
        ];
        let counts = GemsCorpusCounts {
            genuine: 7,
            oracle_limitation: 1,
            advisory: 1,
        };
        let census = build_gems_census(&report(records, counts), "src").expect("census");

        assert_eq!(census.clusters.len(), 5);
        let top = &census.clusters[0];
        assert_eq!(top.record_count, 4);
        assert_eq!(top.diverging_fields, vec!["audio_ram_crc32", "z80_pc"]);
        assert_eq!((top.run_count, top.longest_run), (2, 3));
        assert_eq!((top.first_frame, top.last_frame), (1, 10));
        assert_eq!(top.example_frames, vec![1, 2, 3, 10]);
        let banked: Vec<_> = census
            .clusters
            .iter()
            .filter(|cluster| cluster.bank_pair.is_some())
            .collect();
        assert_eq!(banked.len(), 2, "distinct bank pairs cluster separately");
    }

    #[test]
    fn reconciliation_failure_is_an_error() {
        let records = vec![record(1, "genuine", &["z80_pc"], None)];
        let counts = GemsCorpusCounts {
            genuine: 2,
            oracle_limitation: 0,
            advisory: 0,
        };
        let error =
            build_gems_census(&report(records, counts), "src").expect_err("count drift must fail");
        assert!(error.to_string().contains("reconciliation failed"));
    }

    #[test]
    fn example_frames_are_capped() {
        let records: Vec<_> = (0..25)
            .map(|frame| record(frame, "genuine", &["z80_pc"], None))
            .collect();
        let counts = GemsCorpusCounts {
            genuine: 25,
            oracle_limitation: 0,
            advisory: 0,
        };
        let census = build_gems_census(&report(records, counts), "src").expect("census");
        assert_eq!(census.clusters.len(), 1);
        assert_eq!(census.clusters[0].example_frames.len(), EXAMPLE_FRAME_CAP);
        assert_eq!(census.clusters[0].record_count, 25);
        assert_eq!(
            (census.clusters[0].run_count, census.clusters[0].longest_run),
            (1, 25)
        );
    }
}
