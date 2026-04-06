use anyhow::{Context, Result, bail};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::Path;

const BACKLOG_PATH: &str = "docs/source-port/backlog.md";

/// Label types extracted from the disassembly.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum LabelKind {
    Sub,
    Loc,
}

/// A single ASM label with its location and citation status.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct AsmLabel {
    pub(crate) label: String,
    pub(crate) kind: LabelKind,
    pub(crate) asm_file: String,
    pub(crate) line_number: usize,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub(crate) cited_in: Vec<String>,
}

/// Coverage data for a single ASM subsystem.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct SubsystemCoverage {
    pub(crate) asm_source: String,
    pub(crate) sub_total: usize,
    pub(crate) sub_cited: usize,
    pub(crate) loc_total: usize,
    pub(crate) loc_cited: usize,
    pub(crate) labels: Vec<AsmLabel>,
}

/// Top-level coverage map.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct CoverageMap {
    pub(crate) generated_at: String,
    pub(crate) summary: CoverageSummary,
    pub(crate) subsystems: BTreeMap<String, SubsystemCoverage>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct CoverageSummary {
    pub(crate) total_sub_labels: usize,
    pub(crate) total_loc_labels: usize,
    pub(crate) sub_cited: usize,
    pub(crate) sub_coverage_pct: f64,
    pub(crate) loc_cited: usize,
    pub(crate) loc_coverage_pct: f64,
}

/// Previously hand-annotated data we want to preserve across --update runs.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
struct AnnotationOverlay {
    #[serde(default)]
    descriptions: BTreeMap<String, String>,
    #[serde(default)]
    priorities: BTreeMap<String, String>,
    #[serde(default)]
    backlog_ids: BTreeMap<String, Vec<String>>,
    #[serde(default)]
    parity_evidence: BTreeMap<String, Vec<String>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
enum CoverageState {
    Mentioned,
    Implemented,
    Tested,
    OracleBacked,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
struct CitationBreakdown {
    implementation: Vec<String>,
    tests: Vec<String>,
    docs: Vec<String>,
    support: Vec<String>,
}

// ── Label extraction ───────────────────────────────────────────────────

/// Extract sub_ and loc_ labels from a single ASM file.
fn extract_labels_from_file(path: &Path, relative: &str) -> Result<Vec<AsmLabel>> {
    let bytes = fs::read(path).with_context(|| format!("failed to read {}", path.display()))?;
    let content = String::from_utf8_lossy(&bytes);
    let mut labels = Vec::new();
    for (i, line) in content.lines().enumerate() {
        let trimmed = line.trim_start();
        let (prefix, kind) = if trimmed.starts_with("sub_") {
            ("sub_", LabelKind::Sub)
        } else if trimmed.starts_with("loc_") {
            ("loc_", LabelKind::Loc)
        } else {
            continue;
        };
        // Label line must end with ':'
        let Some(colon_pos) = trimmed.find(':') else {
            continue;
        };
        let candidate = &trimmed[..colon_pos];
        // Must be prefix + hex digits only
        let hex_part = &candidate[prefix.len()..];
        if hex_part.is_empty() || !hex_part.chars().all(|c| c.is_ascii_hexdigit()) {
            continue;
        }
        labels.push(AsmLabel {
            label: candidate.to_owned(),
            kind,
            asm_file: relative.to_owned(),
            line_number: i + 1,
            cited_in: Vec::new(),
        });
    }
    Ok(labels)
}

/// Extract all labels from the disassembly tree.
fn extract_all_asm_labels(upstream_dir: &Path) -> Result<Vec<AsmLabel>> {
    let mut all_labels = Vec::new();

    // kid.asm (main file)
    let kid_asm = upstream_dir.join("kid.asm");
    if kid_asm.is_file() {
        all_labels.extend(extract_labels_from_file(&kid_asm, "kid.asm")?);
    }

    // code/**/*.asm
    let code_dir = upstream_dir.join("code");
    if code_dir.is_dir() {
        collect_asm_files_recursive(&code_dir, &code_dir, &mut all_labels)?;
    }

    Ok(all_labels)
}

fn collect_asm_files_recursive(base: &Path, dir: &Path, labels: &mut Vec<AsmLabel>) -> Result<()> {
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            collect_asm_files_recursive(base, &path, labels)?;
        } else if path.extension().is_some_and(|ext| ext == "asm") {
            let relative = format!(
                "code/{}",
                path.strip_prefix(base).unwrap_or(&path).display()
            );
            labels.extend(extract_labels_from_file(&path, &relative)?);
        }
    }
    Ok(())
}

// ── Citation extraction ────────────────────────────────────────────────

/// Scan native Rust source for ASM label references.
/// Returns a map: label → set of files citing it.
fn extract_native_citations(repo_root: &Path) -> Result<BTreeMap<String, BTreeSet<String>>> {
    let mut citations: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    let search_dirs = [
        repo_root.join("crates/kid-core/src"),
        repo_root.join("crates/kid-data/src"),
    ];
    for dir in &search_dirs {
        if dir.is_dir() {
            scan_dir_for_citations(dir, repo_root, &mut citations)?;
        }
    }
    Ok(citations)
}

fn scan_dir_for_citations(
    dir: &Path,
    repo_root: &Path,
    citations: &mut BTreeMap<String, BTreeSet<String>>,
) -> Result<()> {
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            scan_dir_for_citations(&path, repo_root, citations)?;
        } else if path.extension().is_some_and(|ext| ext == "rs") {
            let relative = path
                .strip_prefix(repo_root)
                .unwrap_or(&path)
                .display()
                .to_string();
            let content = fs::read_to_string(&path)?;
            // Find all sub_XXXX and loc_XXXX references
            for line in content.lines() {
                let mut rest = line;
                while let Some(idx) = rest.find("sub_").or_else(|| rest.find("loc_")) {
                    let prefix = &rest[idx..idx + 4];
                    let after = &rest[idx + 4..];
                    let hex_len = after.chars().take_while(|c| c.is_ascii_hexdigit()).count();
                    if hex_len > 0 {
                        let label = format!("{}{}", prefix, &after[..hex_len]);
                        citations.entry(label).or_default().insert(relative.clone());
                    }
                    rest = &rest[idx + 4 + hex_len..];
                }
            }
        }
    }
    Ok(())
}

// ── Subsystem classification ───────────────────────────────────────────

fn classify_subsystem(asm_file: &str) -> String {
    if asm_file.starts_with("code/enemy/") {
        let name = asm_file
            .strip_prefix("code/enemy/")
            .unwrap_or(asm_file)
            .strip_suffix(".asm")
            .unwrap_or(asm_file);
        format!("enemy_{}", name.to_lowercase().replace(' ', "_"))
    } else if asm_file == "kid.asm" {
        "kid_main".to_owned()
    } else if asm_file.starts_with("code/") {
        let parts: Vec<&str> = asm_file
            .strip_prefix("code/")
            .unwrap_or(asm_file)
            .split('/')
            .collect();
        if let Some(dir) = parts.first() {
            dir.strip_suffix(".asm")
                .unwrap_or(dir)
                .to_lowercase()
                .replace(' ', "_")
        } else {
            "other".to_owned()
        }
    } else {
        "other".to_owned()
    }
}

// ── Coverage map construction ──────────────────────────────────────────

fn build_coverage_map(
    asm_labels: Vec<AsmLabel>,
    citations: &BTreeMap<String, BTreeSet<String>>,
) -> CoverageMap {
    let mut subsystems: BTreeMap<String, Vec<AsmLabel>> = BTreeMap::new();

    for mut label in asm_labels {
        if let Some(files) = citations.get(&label.label) {
            label.cited_in = files.iter().cloned().collect();
        }
        let subsystem = classify_subsystem(&label.asm_file);
        subsystems.entry(subsystem).or_default().push(label);
    }

    let mut total_sub = 0usize;
    let mut total_loc = 0usize;
    let mut cited_sub = 0usize;
    let mut cited_loc = 0usize;

    let mut subsystem_map: BTreeMap<String, SubsystemCoverage> = BTreeMap::new();
    for (name, labels) in subsystems {
        let asm_source = labels
            .first()
            .map(|l| l.asm_file.clone())
            .unwrap_or_default();
        let sub_total = labels.iter().filter(|l| l.kind == LabelKind::Sub).count();
        let sub_cited = labels
            .iter()
            .filter(|l| l.kind == LabelKind::Sub && !l.cited_in.is_empty())
            .count();
        let loc_total = labels.iter().filter(|l| l.kind == LabelKind::Loc).count();
        let loc_cited = labels
            .iter()
            .filter(|l| l.kind == LabelKind::Loc && !l.cited_in.is_empty())
            .count();

        total_sub += sub_total;
        total_loc += loc_total;
        cited_sub += sub_cited;
        cited_loc += loc_cited;

        subsystem_map.insert(
            name,
            SubsystemCoverage {
                asm_source,
                sub_total,
                sub_cited,
                loc_total,
                loc_cited,
                labels,
            },
        );
    }

    let sub_pct = if total_sub > 0 {
        (cited_sub as f64 / total_sub as f64) * 100.0
    } else {
        0.0
    };
    let loc_pct = if total_loc > 0 {
        (cited_loc as f64 / total_loc as f64) * 100.0
    } else {
        0.0
    };

    CoverageMap {
        generated_at: chrono_timestamp(),
        summary: CoverageSummary {
            total_sub_labels: total_sub,
            total_loc_labels: total_loc,
            sub_cited: cited_sub,
            sub_coverage_pct: (sub_pct * 10.0).round() / 10.0,
            loc_cited: cited_loc,
            loc_coverage_pct: (loc_pct * 10.0).round() / 10.0,
        },
        subsystems: subsystem_map,
    }
}

fn chrono_timestamp() -> String {
    // Simple UTC timestamp without pulling in chrono
    use std::time::SystemTime;
    let now = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap_or_default();
    format!("{}s", now.as_secs())
}

// ── YAML I/O with annotation preservation ──────────────────────────────

/// Load existing coverage YAML and extract hand-annotated fields.
fn load_existing_annotations(path: &Path) -> AnnotationOverlay {
    let Ok(content) = fs::read_to_string(path) else {
        return AnnotationOverlay::default();
    };
    // Parse the YAML to extract description and priority annotations
    // that were hand-added. We look for patterns in the YAML structure.
    let Ok(existing): Result<CoverageMapWithAnnotations, _> = serde_yaml::from_str(&content) else {
        return AnnotationOverlay::default();
    };
    let mut overlay = AnnotationOverlay::default();
    for subsystem in existing.subsystems.values() {
        for label in &subsystem.labels {
            if let Some(desc) = &label.description {
                overlay
                    .descriptions
                    .insert(label.label.clone(), desc.clone());
            }
            if let Some(pri) = &label.priority {
                overlay.priorities.insert(label.label.clone(), pri.clone());
            }
            if !label.backlog_ids.is_empty() {
                overlay
                    .backlog_ids
                    .insert(label.label.clone(), label.backlog_ids.clone());
            }
            if !label.parity_evidence.is_empty() {
                overlay
                    .parity_evidence
                    .insert(label.label.clone(), label.parity_evidence.clone());
            }
        }
    }
    overlay
}

/// Extended label struct that includes optional annotation fields.
#[derive(Debug, Clone, Serialize, Deserialize)]
struct AsmLabelAnnotated {
    label: String,
    kind: LabelKind,
    asm_file: String,
    line_number: usize,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    cited_in: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    priority: Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    backlog_ids: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    parity_evidence: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    coverage_state: Option<CoverageState>,
    #[serde(skip_serializing_if = "Option::is_none")]
    implementation_cited: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    test_cited: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    oracle_backed: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    citation_breakdown: Option<CitationBreakdown>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct SubsystemCoverageAnnotated {
    asm_source: String,
    sub_total: usize,
    sub_cited: usize,
    loc_total: usize,
    loc_cited: usize,
    labels: Vec<AsmLabelAnnotated>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct CoverageMapWithAnnotations {
    generated_at: String,
    summary: CoverageSummary,
    subsystems: BTreeMap<String, SubsystemCoverageAnnotated>,
}

fn apply_annotations(map: CoverageMap, overlay: &AnnotationOverlay) -> CoverageMapWithAnnotations {
    let subsystems = map
        .subsystems
        .into_iter()
        .map(|(name, sub)| {
            let labels = sub
                .labels
                .into_iter()
                .map(|l| AsmLabelAnnotated {
                    backlog_ids: overlay
                        .backlog_ids
                        .get(&l.label)
                        .cloned()
                        .unwrap_or_default(),
                    parity_evidence: overlay
                        .parity_evidence
                        .get(&l.label)
                        .cloned()
                        .unwrap_or_default(),
                    coverage_state: coverage_state_for_citations(&l.cited_in, overlay, &l.label),
                    implementation_cited: Some(has_implementation_citation(&l.cited_in)),
                    test_cited: Some(has_test_citation(&l.cited_in)),
                    oracle_backed: Some(is_oracle_backed(overlay, &l.label)),
                    citation_breakdown: Some(classify_citations(&l.cited_in)),
                    description: overlay.descriptions.get(&l.label).cloned(),
                    priority: overlay.priorities.get(&l.label).cloned(),
                    label: l.label,
                    kind: l.kind,
                    asm_file: l.asm_file,
                    line_number: l.line_number,
                    cited_in: l.cited_in,
                })
                .collect();
            (
                name,
                SubsystemCoverageAnnotated {
                    asm_source: sub.asm_source,
                    sub_total: sub.sub_total,
                    sub_cited: sub.sub_cited,
                    loc_total: sub.loc_total,
                    loc_cited: sub.loc_cited,
                    labels,
                },
            )
        })
        .collect();
    CoverageMapWithAnnotations {
        generated_at: map.generated_at,
        summary: map.summary,
        subsystems,
    }
}

fn classify_citations(cited_in: &[String]) -> CitationBreakdown {
    let mut breakdown = CitationBreakdown::default();
    for citation in cited_in {
        if citation.ends_with("/tests.rs") || citation.contains("/tests/") {
            breakdown.tests.push(citation.clone());
        } else if citation.ends_with(".md") {
            breakdown.docs.push(citation.clone());
        } else if citation.contains("/src/") {
            breakdown.implementation.push(citation.clone());
        } else {
            breakdown.support.push(citation.clone());
        }
    }
    breakdown
}

fn has_implementation_citation(cited_in: &[String]) -> bool {
    !classify_citations(cited_in).implementation.is_empty()
}

fn has_test_citation(cited_in: &[String]) -> bool {
    !classify_citations(cited_in).tests.is_empty()
}

fn is_oracle_backed(overlay: &AnnotationOverlay, label: &str) -> bool {
    overlay
        .parity_evidence
        .get(label)
        .is_some_and(|entries| !entries.is_empty())
}

fn coverage_state_for_citations(
    cited_in: &[String],
    overlay: &AnnotationOverlay,
    label: &str,
) -> Option<CoverageState> {
    if is_oracle_backed(overlay, label) {
        Some(CoverageState::OracleBacked)
    } else if has_test_citation(cited_in) {
        Some(CoverageState::Tested)
    } else if has_implementation_citation(cited_in) {
        Some(CoverageState::Implemented)
    } else if !cited_in.is_empty() {
        Some(CoverageState::Mentioned)
    } else {
        None
    }
}

fn load_open_backlog_signal_counts(repo_root: &Path) -> BTreeMap<String, usize> {
    let path = repo_root.join(BACKLOG_PATH);
    let Ok(contents) = fs::read_to_string(&path) else {
        return BTreeMap::new();
    };
    let mut counts = BTreeMap::new();
    for line in contents.lines() {
        if !line.starts_with('|') || line.contains("| Status |") || line.contains("|-----") {
            continue;
        }
        let columns = line
            .split('|')
            .map(str::trim)
            .filter(|column| !column.is_empty())
            .collect::<Vec<_>>();
        if columns.len() < 5 {
            continue;
        }
        let status = columns[1];
        let priority = columns[2];
        if !matches!(status, "[ ]" | "[~]") || !matches!(priority, "P0" | "P1" | "P2") {
            continue;
        }
        *counts.entry(columns[3].to_owned()).or_insert(0) += 1;
    }
    counts
}

// ── Public entry point ─────────────────────────────────────────────────

pub(crate) fn run_asm_coverage(
    repo_root: &Path,
    update: bool,
    subsystem_filter: Option<&str>,
    uncovered_only: bool,
    json_output: bool,
) -> Result<()> {
    let upstream_dir = repo_root.join(".cache/upstream/kid-chameleon-disasm");
    if !upstream_dir.is_dir() {
        bail!(
            "disassembly not found at {}; run `mise run bootstrap-upstream`",
            upstream_dir.display()
        );
    }

    let asm_labels = extract_all_asm_labels(&upstream_dir)?;
    let citations = extract_native_citations(repo_root)?;
    let map = build_coverage_map(asm_labels, &citations);

    if update {
        let output_path = repo_root.join("docs/source-port/asm-coverage.yaml");
        let overlay = load_existing_annotations(&output_path);
        let annotated = apply_annotations(map.clone(), &overlay);
        let header = "# Auto-generated by: kidcli asm-coverage --update\n# Re-run to refresh. Hand-annotated description/priority fields are preserved.\n\n";
        let yaml = serde_yaml::to_string(&annotated)?;
        fs::write(&output_path, format!("{header}{yaml}"))?;
        println!(
            "Updated {}",
            output_path
                .strip_prefix(repo_root)
                .unwrap_or(&output_path)
                .display()
        );
    }

    // Apply filters for display
    let subsystems: BTreeMap<&str, &SubsystemCoverage> = map
        .subsystems
        .iter()
        .filter(|(name, _)| subsystem_filter.map(|f| name.contains(f)).unwrap_or(true))
        .map(|(name, sub)| (name.as_str(), sub))
        .collect();

    if json_output {
        let output = if uncovered_only {
            let uncovered: BTreeMap<&str, Vec<&AsmLabel>> = subsystems
                .iter()
                .filter_map(|(&name, sub)| {
                    let uncov: Vec<&AsmLabel> = sub
                        .labels
                        .iter()
                        .filter(|l| l.kind == LabelKind::Sub && l.cited_in.is_empty())
                        .collect();
                    if uncov.is_empty() {
                        None
                    } else {
                        Some((name, uncov))
                    }
                })
                .collect();
            serde_json::to_string_pretty(&uncovered)?
        } else {
            serde_json::to_string_pretty(&map)?
        };
        println!("{output}");
        return Ok(());
    }

    // Text summary
    println!("ASM Coverage Report");
    println!("═══════════════════");
    println!(
        "sub_ labels: {}/{} cited ({:.1}%)",
        map.summary.sub_cited, map.summary.total_sub_labels, map.summary.sub_coverage_pct
    );
    println!(
        "loc_ labels: {}/{} cited ({:.1}%)",
        map.summary.loc_cited, map.summary.total_loc_labels, map.summary.loc_coverage_pct
    );
    println!(
        "interpretation: label citation tracks recovered/discussed ASM, not player-visible fidelity"
    );
    let backlog_signals = load_open_backlog_signal_counts(repo_root);
    if !backlog_signals.is_empty() {
        let summary = backlog_signals
            .iter()
            .map(|(system, count)| format!("{system}={count}"))
            .collect::<Vec<_>>()
            .join(", ");
        println!("open backlog signals: {summary}");
    }
    println!();

    // Per-subsystem breakdown
    let mut sorted_subsystems: Vec<_> = subsystems.into_iter().collect();
    sorted_subsystems.sort_by(|a, b| {
        // Sort by coverage gap (most uncovered sub_ first)
        let gap_a = a.1.sub_total.saturating_sub(a.1.sub_cited);
        let gap_b = b.1.sub_total.saturating_sub(b.1.sub_cited);
        gap_b.cmp(&gap_a).then_with(|| a.0.cmp(b.0))
    });

    for (name, sub) in &sorted_subsystems {
        if uncovered_only && sub.sub_cited == sub.sub_total {
            continue;
        }
        println!(
            "{}: sub {}/{}, loc {}/{}  [{}]",
            name, sub.sub_cited, sub.sub_total, sub.loc_cited, sub.loc_total, sub.asm_source
        );

        if uncovered_only {
            for label in &sub.labels {
                if label.kind == LabelKind::Sub && label.cited_in.is_empty() {
                    println!("  - {} (line {})", label.label, label.line_number);
                }
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classify_citations_separates_implementation_and_tests() {
        let cited_in = vec![
            "crates/kid-core/src/level.rs".to_owned(),
            "crates/kid-core/src/tests.rs".to_owned(),
            "docs/source-port/backlog.md".to_owned(),
        ];

        let breakdown = classify_citations(&cited_in);

        assert_eq!(
            breakdown.implementation,
            vec!["crates/kid-core/src/level.rs"]
        );
        assert_eq!(breakdown.tests, vec!["crates/kid-core/src/tests.rs"]);
        assert_eq!(breakdown.docs, vec!["docs/source-port/backlog.md"]);
    }

    #[test]
    fn coverage_state_prefers_oracle_backed_evidence() {
        let cited_in = vec![
            "crates/kid-core/src/level.rs".to_owned(),
            "crates/kid-core/src/tests.rs".to_owned(),
        ];
        let mut overlay = AnnotationOverlay::default();
        overlay.parity_evidence.insert(
            "loc_1234".to_owned(),
            vec!["verify-tas --frame-start 10 --frame-end 20".to_owned()],
        );

        assert_eq!(
            coverage_state_for_citations(&cited_in, &overlay, "loc_1234"),
            Some(CoverageState::OracleBacked)
        );
    }
}
