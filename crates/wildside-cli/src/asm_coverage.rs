use crate::backlog::read_live_lanes;
use crate::rust_source_context::{RustCodeContext, RustSourceContexts};
use anyhow::{Context, Result, bail};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::Path;

const TEST_ONLY_BASELINE_PATH: &str = "docs/source-port/asm-coverage-test-only-baseline.yaml";

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
    #[serde(skip_serializing_if = "Option::is_none")]
    coverage_state: Option<CoverageState>,
    #[serde(skip_serializing_if = "Option::is_none")]
    implementation_cited: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    test_cited: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    emulated_gems: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    citation_breakdown: Option<CitationBreakdown>,
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
    #[serde(default)]
    pub(crate) total_labels: usize,
    #[serde(default)]
    pub(crate) reachable_labels: usize,
    #[serde(default)]
    pub(crate) test_only_debt_labels: usize,
    #[serde(default)]
    /// Ordinary labels with no scanned production/test citation. Emulated GEMS
    /// M68k<->Z80 wrappers are counted in `emulated_gems_labels` instead.
    pub(crate) uncovered_labels: usize,
    #[serde(default)]
    pub(crate) emulated_gems_labels: usize,
    #[serde(default)]
    pub(crate) non_executable_data_alias_labels: usize,
    #[serde(default)]
    /// Backward-compatible alias for `uncovered_labels` after the GEMS split.
    pub(crate) non_emulated_uncovered_labels: usize,
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
    #[serde(alias = "implemented")]
    Reachable,
    #[serde(alias = "tested")]
    TestOnlyDebt,
    #[serde(alias = "mentioned")]
    Uncovered,
    EmulatedGems,
    NonExecutableDataAlias,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
struct CitationBreakdown {
    implementation: Vec<String>,
    tests: Vec<String>,
    docs: Vec<String>,
    support: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct NativeCitationSite {
    path: String,
    line_number: usize,
    context: RustCodeContext,
}

#[derive(Debug, Clone, Default)]
struct NativeCitationIndex {
    files_by_label: BTreeMap<String, BTreeSet<String>>,
    breakdown_by_label: BTreeMap<String, CitationBreakdown>,
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
            coverage_state: None,
            implementation_cited: None,
            test_cited: None,
            emulated_gems: None,
            citation_breakdown: None,
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
fn extract_native_citations(repo_root: &Path) -> Result<NativeCitationIndex> {
    let mut sites_by_label: BTreeMap<String, BTreeSet<NativeCitationSite>> = BTreeMap::new();
    let search_dirs = [
        repo_root.join("crates/wildside-core/src"),
        repo_root.join("crates/wildside-data/src"),
        repo_root.join("crates/wildside-md/src"),
    ];
    for dir in &search_dirs {
        if dir.is_dir() {
            scan_dir_for_citations(dir, repo_root, &mut sites_by_label)?;
        }
    }
    Ok(index_native_citation_sites(sites_by_label))
}

fn scan_dir_for_citations(
    dir: &Path,
    repo_root: &Path,
    sites_by_label: &mut BTreeMap<String, BTreeSet<NativeCitationSite>>,
) -> Result<()> {
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            scan_dir_for_citations(&path, repo_root, sites_by_label)?;
        } else if path.extension().is_some_and(|ext| ext == "rs") {
            let relative = path
                .strip_prefix(repo_root)
                .unwrap_or(&path)
                .display()
                .to_string();
            let content = fs::read_to_string(&path)?;
            for site in scan_rust_file_for_citations(&relative, &content) {
                sites_by_label
                    .entry(site.label)
                    .or_default()
                    .insert(site.citation);
            }
        }
    }
    Ok(())
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct LabelCitationSite {
    label: String,
    citation: NativeCitationSite,
}

fn scan_rust_file_for_citations(relative_path: &str, content: &str) -> Vec<LabelCitationSite> {
    let contexts = RustSourceContexts::analyse(content);
    let path_is_test = path_is_test_file(relative_path);
    let mut sites = Vec::new();
    // Find all sub_XXXX and loc_XXXX references. Citations intentionally include
    // comments: ASM comments are the auditable source-port link.
    for (line_index, line) in content.lines().enumerate() {
        let line_number = line_index + 1;
        let context = if path_is_test {
            RustCodeContext::Test
        } else {
            contexts.context_for_line(line_number)
        };
        let mut rest = line;
        while let Some(idx) = [rest.find("sub_"), rest.find("loc_")]
            .into_iter()
            .flatten()
            .min()
        {
            let prefix = &rest[idx..idx + 4];
            let after = &rest[idx + 4..];
            let hex_len = after.chars().take_while(char::is_ascii_hexdigit).count();
            if hex_len > 0 {
                sites.push(LabelCitationSite {
                    label: format!("{}{}", prefix, &after[..hex_len]),
                    citation: NativeCitationSite {
                        path: relative_path.to_owned(),
                        line_number,
                        context,
                    },
                });
            }
            rest = &rest[idx + 4 + hex_len..];
        }
    }
    sites
}

fn index_native_citation_sites(
    sites_by_label: BTreeMap<String, BTreeSet<NativeCitationSite>>,
) -> NativeCitationIndex {
    let mut index = NativeCitationIndex::default();
    for (label, sites) in sites_by_label {
        let mut breakdown = CitationBreakdown::default();
        for site in sites {
            index
                .files_by_label
                .entry(label.clone())
                .or_default()
                .insert(site.path.clone());
            match classify_native_citation_site(&site) {
                CitationBucket::Implementation => {
                    push_unique_sorted(&mut breakdown.implementation, site.path);
                }
                CitationBucket::Tests => {
                    push_unique_sorted(&mut breakdown.tests, site.path);
                }
                CitationBucket::Docs => {
                    push_unique_sorted(&mut breakdown.docs, site.path);
                }
                CitationBucket::Support => {
                    push_unique_sorted(&mut breakdown.support, site.path);
                }
            }
        }
        index.breakdown_by_label.insert(label, breakdown);
    }
    index
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CitationBucket {
    Implementation,
    Tests,
    Docs,
    Support,
}

fn classify_native_citation_site(site: &NativeCitationSite) -> CitationBucket {
    if path_is_test_file(&site.path) || site.context == RustCodeContext::Test {
        CitationBucket::Tests
    } else if std::path::Path::new(&site.path)
        .extension()
        .is_some_and(|ext| ext.eq_ignore_ascii_case("md"))
    {
        CitationBucket::Docs
    } else if site.path.contains("/src/") {
        CitationBucket::Implementation
    } else {
        CitationBucket::Support
    }
}

fn path_is_test_file(path: &str) -> bool {
    path.ends_with("/tests.rs") || path.contains("/tests/")
}

fn push_unique_sorted(values: &mut Vec<String>, value: String) {
    match values.binary_search(&value) {
        Ok(_) => {}
        Err(index) => values.insert(index, value),
    }
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

#[derive(Debug, Clone)]
struct LabelCoverageClassification {
    state: CoverageState,
    breakdown: CitationBreakdown,
    implementation_cited: bool,
    test_cited: bool,
    emulated_gems: bool,
}

fn classify_label_coverage(
    label: &str,
    citation_breakdowns: &BTreeMap<String, CitationBreakdown>,
    cited_in: &[String],
) -> LabelCoverageClassification {
    let breakdown = citation_breakdown_for_label(citation_breakdowns, label, cited_in);
    let implementation_cited = !breakdown.implementation.is_empty();
    let test_cited = !breakdown.tests.is_empty();
    let emulated_gems = is_emulated_gems_label(label);
    let state = if is_non_executable_data_alias_label(label) {
        CoverageState::NonExecutableDataAlias
    } else if implementation_cited {
        CoverageState::Reachable
    } else if test_cited {
        CoverageState::TestOnlyDebt
    } else if emulated_gems {
        CoverageState::EmulatedGems
    } else {
        CoverageState::Uncovered
    };

    LabelCoverageClassification {
        state,
        breakdown,
        implementation_cited,
        test_cited,
        emulated_gems,
    }
}

fn is_emulated_gems_label(label: &str) -> bool {
    label_address(label).is_some_and(|address| (0xE133C..=0xE15E8).contains(&address))
}

fn is_non_executable_data_alias_label(label: &str) -> bool {
    // ASM CollisionSize_Index[612..614] uses loc_32454 as filler alongside
    // Data_Index[612..614]'s Pal_A2100/A210E/A211C pointers. Those palette
    // entries can never become a GfxObject sprite link, so the bytes at this
    // address are not an executable Tar Monster path or a collision row that
    // native gameplay can select.
    matches!(label, "loc_32454")
}

fn label_address(label: &str) -> Option<u32> {
    label
        .strip_prefix("sub_")
        .or_else(|| label.strip_prefix("loc_"))
        .and_then(|hex| u32::from_str_radix(hex, 16).ok())
}

fn appears_in_uncovered_report(label: &AsmLabel) -> bool {
    label.coverage_state == Some(CoverageState::Uncovered)
        || (label.kind == LabelKind::Sub
            && label.coverage_state == Some(CoverageState::EmulatedGems))
}

fn build_coverage_map(
    asm_labels: Vec<AsmLabel>,
    citations: &BTreeMap<String, BTreeSet<String>>,
    citation_breakdowns: &BTreeMap<String, CitationBreakdown>,
) -> CoverageMap {
    let mut subsystems: BTreeMap<String, Vec<AsmLabel>> = BTreeMap::new();

    for mut label in asm_labels {
        if let Some(files) = citations.get(&label.label) {
            label.cited_in = files.iter().cloned().collect();
        }
        let classification =
            classify_label_coverage(&label.label, citation_breakdowns, &label.cited_in);
        label.coverage_state = Some(classification.state);
        label.implementation_cited = Some(classification.implementation_cited);
        label.test_cited = Some(classification.test_cited);
        label.emulated_gems = Some(classification.emulated_gems);
        label.citation_breakdown = Some(classification.breakdown);
        let subsystem = classify_subsystem(&label.asm_file);
        subsystems.entry(subsystem).or_default().push(label);
    }

    let mut total_sub = 0usize;
    let mut total_loc = 0usize;
    let mut cited_sub = 0usize;
    let mut cited_loc = 0usize;
    let mut reachable_labels = 0usize;
    let mut test_only_debt_labels = 0usize;
    let mut uncovered_labels = 0usize;
    let mut emulated_gems_labels = 0usize;
    let mut non_executable_data_alias_labels = 0usize;

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
        for label in &labels {
            match label.coverage_state {
                Some(CoverageState::Reachable) => reachable_labels += 1,
                Some(CoverageState::TestOnlyDebt) => test_only_debt_labels += 1,
                Some(CoverageState::Uncovered) => uncovered_labels += 1,
                Some(CoverageState::EmulatedGems) => emulated_gems_labels += 1,
                Some(CoverageState::NonExecutableDataAlias) => {
                    non_executable_data_alias_labels += 1;
                }
                None => uncovered_labels += 1,
            }
        }

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
            total_labels: total_sub + total_loc,
            reachable_labels,
            test_only_debt_labels,
            uncovered_labels,
            emulated_gems_labels,
            non_executable_data_alias_labels,
            non_emulated_uncovered_labels: uncovered_labels,
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

/// Return `text` with every top-level `generated_at:` line stripped so two YAML
/// documents can be compared for semantic equality without the timestamp churn.
fn yaml_without_generated_at(text: &str) -> String {
    text.lines()
        .filter(|line| !line.trim_start().starts_with("generated_at:"))
        .collect::<Vec<_>>()
        .join("\n")
}

/// Pull the first top-level `generated_at:` value out of a YAML document, trimmed.
fn extract_generated_at(text: &str) -> Option<String> {
    text.lines().find_map(|line| {
        let trimmed = line.trim_start();
        trimmed
            .strip_prefix("generated_at:")
            .map(|v| v.trim().trim_matches('"').to_owned())
    })
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
    emulated_gems: Option<bool>,
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
                    coverage_state: l.coverage_state,
                    implementation_cited: l.implementation_cited,
                    test_cited: l.test_cited,
                    emulated_gems: l.emulated_gems,
                    citation_breakdown: l.citation_breakdown,
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
        } else if std::path::Path::new(citation)
            .extension()
            .is_some_and(|ext| ext.eq_ignore_ascii_case("md"))
        {
            breakdown.docs.push(citation.clone());
        } else if citation.contains("/src/") {
            breakdown.implementation.push(citation.clone());
        } else {
            breakdown.support.push(citation.clone());
        }
    }
    breakdown
}

fn citation_breakdown_for_label(
    citation_breakdowns: &BTreeMap<String, CitationBreakdown>,
    label: &str,
    cited_in: &[String],
) -> CitationBreakdown {
    citation_breakdowns
        .get(label)
        .cloned()
        .unwrap_or_else(|| classify_citations(cited_in))
}

#[cfg(test)]
fn coverage_state_for_label(
    citation_breakdowns: &BTreeMap<String, CitationBreakdown>,
    _overlay: &AnnotationOverlay,
    label: &str,
    cited_in: &[String],
) -> CoverageState {
    classify_label_coverage(label, citation_breakdowns, cited_in).state
}

fn load_open_backlog_signal_counts(repo_root: &Path) -> BTreeMap<String, usize> {
    let Ok(contents) = read_live_lanes(repo_root) else {
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
        if !matches!(status, "[ ]" | "[~]") || !matches!(priority, "P0" | "P1" | "P2" | "P3") {
            continue;
        }
        *counts.entry(columns[3].to_owned()).or_insert(0) += 1;
    }
    counts
}

// ── Required-labels gate ───────────────────────────────────────────────
//
// Labels listed here MUST stay cited from native source files. The gate
// is preventative: every label below is currently covered, so the gate
// passes today. It exists to catch the regression where someone deletes
// the only citation, leaving a load-bearing ROM routine undocumented in
// the native code.
//
// Add a label here only when there is a backlog item or expected fidelity
// regression that hinges on a specific routine being recovered. The
// reason field is shown when the gate fails — make it useful for the
// person who has to restore the citation.

#[derive(Debug, Clone, Copy)]
pub(crate) struct RequiredLabel {
    pub(crate) label: &'static str,
    pub(crate) reason: &'static str,
}

pub(crate) const REQUIRED_LABELS: &[RequiredLabel] = &[
    // Slope-traversal facing flips (backlog #137 — moonwalking up slopes).
    RequiredLabel {
        label: "loc_9E88",
        reason: "SlopeDown facing branch (sf x_direction). #137 moonwalk regression.",
    },
    RequiredLabel {
        label: "loc_9EA4",
        reason: "SlopeUp facing branch (st x_direction). #137 moonwalk regression.",
    },
    RequiredLabel {
        label: "loc_9D22",
        reason: "Slope grid lookup feeding the SlopeDown/SlopeUp branches. #137.",
    },
    // Sprite direction selection during gameplay.
    RequiredLabel {
        label: "loc_A13A",
        reason: "Sprite direction selection — drives kid_x_direction every frame.",
    },
    // Slope-tracking math used by the renderer for column tilt.
    RequiredLabel {
        label: "sub_902A",
        reason: "Slope tracking helper (sub_9386 callee). Sprite tilt parity.",
    },
    RequiredLabel {
        label: "sub_9386",
        reason: "Slope direction probe ($FFFFFA26 producer). Renderer tilt parity.",
    },
    // Per-helmet attack routines that gate the rest of the combat ports.
    RequiredLabel {
        label: "sub_868E",
        reason: "Red Stealth sword swing. Removing this citation hides attack regressions.",
    },
    RequiredLabel {
        label: "loc_8ABC",
        reason: "Maniaxe axe-throw branch. Combat parity for helmet 08.",
    },
    RequiredLabel {
        label: "sub_AF7A",
        reason: "Iron Knight block-breaker (#143). Helmet attack + block destroy.",
    },
];

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum RequiredLabelStatus {
    /// Label is cited from at least one source file — gate passes for it.
    Covered,
    /// Label is in the disassembly but no source file cites it.
    Uncovered,
    /// Label is not present in the disassembly map at all (renamed,
    /// removed, or extractor missed it).
    NotFound,
}

#[derive(Debug, Clone)]
pub(crate) struct RequiredLabelReport {
    pub(crate) required: RequiredLabel,
    pub(crate) status: RequiredLabelStatus,
}

#[derive(Debug, Clone, Deserialize)]
struct TestOnlyBaseline {
    version: u32,
    labels: BTreeSet<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct TestOnlyRatchetReport {
    current_count: usize,
    baseline_count: usize,
    new_labels: Vec<String>,
    burned_down_labels: Vec<String>,
}

pub(crate) fn check_required_labels(map: &CoverageMap) -> Vec<RequiredLabelReport> {
    REQUIRED_LABELS
        .iter()
        .map(|required| {
            let mut status = RequiredLabelStatus::NotFound;
            for sub in map.subsystems.values() {
                if let Some(label) = sub.labels.iter().find(|l| l.label == required.label) {
                    status = if label.cited_in.is_empty() {
                        RequiredLabelStatus::Uncovered
                    } else {
                        RequiredLabelStatus::Covered
                    };
                    break;
                }
            }
            RequiredLabelReport {
                required: *required,
                status,
            }
        })
        .collect()
}

fn test_only_debt_labels(map: &CoverageMap) -> BTreeSet<String> {
    map.subsystems
        .values()
        .flat_map(|subsystem| subsystem.labels.iter())
        .filter(|label| label.coverage_state == Some(CoverageState::TestOnlyDebt))
        .map(|label| label.label.clone())
        .collect()
}

fn load_test_only_baseline(repo_root: &Path) -> Result<TestOnlyBaseline> {
    let path = repo_root.join(TEST_ONLY_BASELINE_PATH);
    let content =
        fs::read_to_string(&path).with_context(|| format!("failed to read {}", path.display()))?;
    let baseline: TestOnlyBaseline = serde_yaml::from_str(&content)
        .with_context(|| format!("failed to parse {}", path.display()))?;
    if baseline.version != 1 {
        bail!(
            "{} has unsupported version {}; expected 1",
            TEST_ONLY_BASELINE_PATH,
            baseline.version
        );
    }
    Ok(baseline)
}

fn compare_test_only_baseline(
    map: &CoverageMap,
    baseline: &TestOnlyBaseline,
) -> TestOnlyRatchetReport {
    let current = test_only_debt_labels(map);
    let new_labels = current.difference(&baseline.labels).cloned().collect();
    let burned_down_labels = baseline.labels.difference(&current).cloned().collect();
    TestOnlyRatchetReport {
        current_count: current.len(),
        baseline_count: baseline.labels.len(),
        new_labels,
        burned_down_labels,
    }
}

fn enforce_no_new_test_only_labels(report: &TestOnlyRatchetReport) -> Result<()> {
    if !report.new_labels.is_empty() {
        bail!(
            "{} new ASM label(s) are test-only debt; wire them into production or explicitly update {} after review",
            report.new_labels.len(),
            TEST_ONLY_BASELINE_PATH
        )
    }

    if !report.burned_down_labels.is_empty() {
        bail!(
            "{} test-only ASM label(s) are no longer debt but still listed in {}; \
             remove them from the baseline so the ratchet cannot silently regress",
            report.burned_down_labels.len(),
            TEST_ONLY_BASELINE_PATH
        )
    }

    Ok(())
}

// ── Public entry point ─────────────────────────────────────────────────

pub(crate) fn run_asm_coverage(
    repo_root: &Path,
    update: bool,
    subsystem_filter: Option<&str>,
    uncovered_only: bool,
    json_output: bool,
    check_required: bool,
    check_no_new_test_only: bool,
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
    let map = build_coverage_map(
        asm_labels,
        &citations.files_by_label,
        &citations.breakdown_by_label,
    );

    if update {
        // Generated regenerable state lives under .cache (untracked), not in docs:
        // the 2.2MB dump churned across thousands of commits and conflated label
        // citation with verified fidelity. (#38)
        let output_path = repo_root.join(".cache/asm-coverage.yaml");
        let overlay = load_existing_annotations(&output_path);
        let mut annotated = apply_annotations(map.clone(), &overlay);
        let header = "# Auto-generated by: wildside asm-coverage --update\n# Re-run to refresh. Hand-annotated description/priority fields are preserved.\n\n";
        // Keep the existing `generated_at` timestamp when no label/citation content changed.
        // Otherwise every `asm-coverage --update` invocation would dirty the tree, pinning the
        // autonomy loop in recovery mode even when nothing of substance moved.
        if let Ok(existing) = fs::read_to_string(&output_path) {
            let candidate = format!("{header}{}", serde_yaml::to_string(&annotated)?);
            if yaml_without_generated_at(&existing) == yaml_without_generated_at(&candidate)
                && let Some(prev_ts) = extract_generated_at(&existing)
            {
                annotated.generated_at = prev_ts;
            }
        }
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
        .filter(|(name, _)| subsystem_filter.is_none_or(|f| name.contains(f)))
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
                        .filter(|label| appears_in_uncovered_report(label))
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
        "labels: {} reachable / {} test-only debt / {} ordinary uncovered / {} emulated GEMS / {} non-executable data alias",
        map.summary.reachable_labels,
        map.summary.test_only_debt_labels,
        map.summary.uncovered_labels,
        map.summary.emulated_gems_labels,
        map.summary.non_executable_data_alias_labels
    );
    println!(
        "legacy citation presence: sub_ {}/{} ({:.1}%), loc_ {}/{} ({:.1}%)",
        map.summary.sub_cited,
        map.summary.total_sub_labels,
        map.summary.sub_coverage_pct,
        map.summary.loc_cited,
        map.summary.total_loc_labels,
        map.summary.loc_coverage_pct
    );
    println!(
        "interpretation: reachable means scanned non-test implementation citation; test-only ports are fidelity debt; emulated GEMS wrappers and non-executable data aliases are out of native hand-port scope"
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
        if uncovered_only && !sub.labels.iter().any(appears_in_uncovered_report) {
            continue;
        }
        println!(
            "{}: sub {}/{}, loc {}/{}  [{}]",
            name, sub.sub_cited, sub.sub_total, sub.loc_cited, sub.loc_total, sub.asm_source
        );

        if uncovered_only {
            for label in &sub.labels {
                if appears_in_uncovered_report(label) {
                    let suffix = if label.coverage_state == Some(CoverageState::EmulatedGems) {
                        " [emulated GEMS out-of-scope]"
                    } else {
                        ""
                    };
                    println!("  - {} (line {}){}", label.label, label.line_number, suffix);
                }
            }
        }
    }

    if check_required {
        let reports = check_required_labels(&map);
        let issues: Vec<&RequiredLabelReport> = reports
            .iter()
            .filter(|r| !matches!(r.status, RequiredLabelStatus::Covered))
            .collect();
        if issues.is_empty() {
            println!("\nrequired-labels gate: {} labels covered", reports.len());
        } else {
            println!("\nrequired-labels gate: FAIL ({} issues)", issues.len());
            for report in &issues {
                let status = match report.status {
                    RequiredLabelStatus::Uncovered => "uncovered",
                    RequiredLabelStatus::NotFound => "not in disasm",
                    RequiredLabelStatus::Covered => unreachable!(),
                };
                println!(
                    "  - {} ({}): {}",
                    report.required.label, status, report.required.reason
                );
            }
            bail!(
                "{} required ASM label(s) missing scanned non-test implementation citations — restore them in production source",
                issues.len()
            );
        }
    }

    if check_no_new_test_only {
        let baseline = load_test_only_baseline(repo_root)?;
        let report = compare_test_only_baseline(&map, &baseline);
        if !report.new_labels.is_empty() {
            println!(
                "\ntest-only ratchet: FAIL ({} new test-only label(s); current {}, baseline {})",
                report.new_labels.len(),
                report.current_count,
                report.baseline_count
            );
            for label in &report.new_labels {
                println!("  - {label}");
            }
        }
        if !report.burned_down_labels.is_empty() {
            println!(
                "\ntest-only ratchet: FAIL ({} burned-down label(s) still in baseline; current {}, baseline {})",
                report.burned_down_labels.len(),
                report.current_count,
                report.baseline_count
            );
            for label in &report.burned_down_labels {
                println!("  - {label}");
            }
        }
        enforce_no_new_test_only_labels(&report)?;
        println!(
            "\ntest-only ratchet: ok ({} current, {} baseline)",
            report.current_count, report.baseline_count
        );
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_label(label: &str, kind: LabelKind, cited: &[&str]) -> AsmLabel {
        AsmLabel {
            label: label.to_owned(),
            kind,
            asm_file: "kid.asm".to_owned(),
            line_number: 1,
            cited_in: cited.iter().map(|s| (*s).to_owned()).collect(),
            coverage_state: None,
            implementation_cited: None,
            test_cited: None,
            emulated_gems: None,
            citation_breakdown: None,
        }
    }

    fn make_label_with_state(label: &str, state: CoverageState) -> AsmLabel {
        let mut label = make_label(label, LabelKind::Loc, &[]);
        label.coverage_state = Some(state);
        label
    }

    fn make_subsystem(labels: Vec<AsmLabel>) -> SubsystemCoverage {
        SubsystemCoverage {
            asm_source: "kid.asm".to_owned(),
            sub_total: labels.iter().filter(|l| l.kind == LabelKind::Sub).count(),
            sub_cited: labels
                .iter()
                .filter(|l| l.kind == LabelKind::Sub && !l.cited_in.is_empty())
                .count(),
            loc_total: labels.iter().filter(|l| l.kind == LabelKind::Loc).count(),
            loc_cited: labels
                .iter()
                .filter(|l| l.kind == LabelKind::Loc && !l.cited_in.is_empty())
                .count(),
            labels,
        }
    }

    fn coverage_map_with(labels: Vec<AsmLabel>) -> CoverageMap {
        let mut subsystems = BTreeMap::new();
        subsystems.insert("test".to_owned(), make_subsystem(labels));
        CoverageMap {
            generated_at: "0s".to_owned(),
            summary: CoverageSummary {
                total_sub_labels: 0,
                total_loc_labels: 0,
                sub_cited: 0,
                sub_coverage_pct: 100.0,
                loc_cited: 0,
                loc_coverage_pct: 100.0,
                total_labels: 0,
                reachable_labels: 0,
                test_only_debt_labels: 0,
                uncovered_labels: 0,
                emulated_gems_labels: 0,
                non_executable_data_alias_labels: 0,
                non_emulated_uncovered_labels: 0,
            },
            subsystems,
        }
    }

    fn native_site(path: &str, context: RustCodeContext) -> NativeCitationSite {
        NativeCitationSite {
            path: path.to_owned(),
            line_number: 1,
            context,
        }
    }

    #[test]
    fn check_required_labels_reports_covered_when_all_cited() {
        let labels = REQUIRED_LABELS
            .iter()
            .map(|r| {
                make_label(
                    r.label,
                    LabelKind::Loc,
                    &["crates/wildside-core/src/movement.rs"],
                )
            })
            .collect();
        let map = coverage_map_with(labels);

        let reports = check_required_labels(&map);
        assert_eq!(reports.len(), REQUIRED_LABELS.len());
        for report in reports {
            assert_eq!(report.status, RequiredLabelStatus::Covered);
        }
    }

    #[test]
    fn check_required_labels_flags_uncovered_label() {
        // Only one required label is in the map, with no citation.
        let target = REQUIRED_LABELS.first().expect("REQUIRED_LABELS nonempty");
        let labels = vec![make_label(target.label, LabelKind::Loc, &[])];
        let map = coverage_map_with(labels);

        let reports = check_required_labels(&map);
        let target_report = reports
            .iter()
            .find(|r| r.required.label == target.label)
            .expect("target label report present");
        assert_eq!(target_report.status, RequiredLabelStatus::Uncovered);
    }

    #[test]
    fn check_required_labels_flags_missing_label() {
        // No labels in the map at all — every required label should report
        // NotFound rather than Covered, so the gate fails loudly when the
        // disasm is missing or the extractor regresses.
        let map = coverage_map_with(Vec::new());

        let reports = check_required_labels(&map);
        for report in reports {
            assert_eq!(report.status, RequiredLabelStatus::NotFound);
        }
    }

    #[test]
    fn check_required_labels_are_covered_in_real_repo() {
        let repo_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .and_then(|path| path.parent())
            .expect("wildside-cli crate should live under crates/");
        let upstream_dir = repo_root.join(".cache/upstream/kid-chameleon-disasm");
        assert!(
            upstream_dir.is_dir(),
            "disassembly fixture missing at {}; run `mise run bootstrap-upstream`",
            upstream_dir.display()
        );

        let asm_labels =
            extract_all_asm_labels(&upstream_dir).expect("real disassembly labels parse");
        let citations = extract_native_citations(repo_root).expect("real native citations scan");
        let map = build_coverage_map(
            asm_labels,
            &citations.files_by_label,
            &citations.breakdown_by_label,
        );
        let reports = check_required_labels(&map);
        let issues = reports
            .iter()
            .filter(|report| !matches!(report.status, RequiredLabelStatus::Covered))
            .map(|report| format!("{}: {:?}", report.required.label, report.status))
            .collect::<Vec<_>>();

        assert_eq!(reports.len(), REQUIRED_LABELS.len());
        assert!(
            issues.is_empty(),
            "required ASM labels must stay cited in the real repo: {}",
            issues.join(", ")
        );
    }

    #[test]
    fn test_only_ratchet_allows_grandfathered_labels() {
        let baseline = TestOnlyBaseline {
            version: 1,
            labels: BTreeSet::from(["loc_1111".to_owned(), "loc_2222".to_owned()]),
        };
        let map = coverage_map_with(vec![
            make_label_with_state("loc_1111", CoverageState::TestOnlyDebt),
            make_label_with_state("loc_2222", CoverageState::TestOnlyDebt),
            make_label_with_state("loc_3333", CoverageState::Reachable),
        ]);

        let report = compare_test_only_baseline(&map, &baseline);

        assert_eq!(report.current_count, 2);
        assert_eq!(report.baseline_count, 2);
        assert!(report.new_labels.is_empty());
        assert!(report.burned_down_labels.is_empty());
        enforce_no_new_test_only_labels(&report).expect("in-sync baseline should pass");
    }

    #[test]
    fn test_only_ratchet_requires_baseline_shrink_after_burn_down() {
        let baseline = TestOnlyBaseline {
            version: 1,
            labels: BTreeSet::from(["loc_1111".to_owned(), "loc_2222".to_owned()]),
        };
        let map = coverage_map_with(vec![
            make_label_with_state("loc_1111", CoverageState::TestOnlyDebt),
            make_label_with_state("loc_3333", CoverageState::Reachable),
        ]);

        let report = compare_test_only_baseline(&map, &baseline);

        assert_eq!(report.current_count, 1);
        assert_eq!(report.baseline_count, 2);
        assert!(report.new_labels.is_empty());
        assert_eq!(report.burned_down_labels, vec!["loc_2222"]);
        let err = enforce_no_new_test_only_labels(&report)
            .expect_err("stale baseline entries must fail the ratchet");
        assert!(
            err.to_string().contains(TEST_ONLY_BASELINE_PATH),
            "error should point at the baseline file to shrink: {err}"
        );
    }

    #[test]
    fn test_only_ratchet_reports_new_test_only_debt() {
        let baseline = TestOnlyBaseline {
            version: 1,
            labels: BTreeSet::from(["loc_1111".to_owned()]),
        };
        let map = coverage_map_with(vec![
            make_label_with_state("loc_1111", CoverageState::TestOnlyDebt),
            make_label_with_state("loc_2222", CoverageState::TestOnlyDebt),
        ]);

        let report = compare_test_only_baseline(&map, &baseline);

        assert_eq!(report.current_count, 2);
        assert_eq!(report.baseline_count, 1);
        assert_eq!(report.new_labels, vec!["loc_2222"]);
        assert!(enforce_no_new_test_only_labels(&report).is_err());
    }

    #[test]
    fn test_only_baseline_deserialises_label_set() {
        let baseline: TestOnlyBaseline = serde_yaml::from_str(
            "
version: 1
labels:
  - loc_2222
  - loc_1111
",
        )
        .expect("baseline parses");

        assert_eq!(baseline.version, 1);
        assert_eq!(
            baseline.labels,
            BTreeSet::from(["loc_1111".to_owned(), "loc_2222".to_owned()])
        );
    }

    #[test]
    fn classify_citations_separates_implementation_and_tests() {
        let cited_in = vec![
            "crates/wildside-core/src/level.rs".to_owned(),
            "crates/wildside-core/src/tests.rs".to_owned(),
            "docs/source-port/backlog.md".to_owned(),
        ];

        let breakdown = classify_citations(&cited_in);

        assert_eq!(
            breakdown.implementation,
            vec!["crates/wildside-core/src/level.rs"]
        );
        assert_eq!(breakdown.tests, vec!["crates/wildside-core/src/tests.rs"]);
        assert_eq!(breakdown.docs, vec!["docs/source-port/backlog.md"]);
    }

    #[test]
    fn classify_native_citation_site_separates_path_and_context_test_cases() {
        assert_eq!(
            classify_native_citation_site(&native_site(
                "crates/wildside-core/src/level.rs",
                RustCodeContext::Production
            )),
            CitationBucket::Implementation
        );
        assert_eq!(
            classify_native_citation_site(&native_site(
                "crates/wildside-core/src/blocks.rs",
                RustCodeContext::Test
            )),
            CitationBucket::Tests
        );
        assert_eq!(
            classify_native_citation_site(&native_site(
                "crates/wildside-core/src/tests.rs",
                RustCodeContext::Production
            )),
            CitationBucket::Tests
        );
        assert_eq!(
            classify_native_citation_site(&native_site(
                "crates/wildside-core/tests/fixtures.rs",
                RustCodeContext::Production
            )),
            CitationBucket::Tests
        );
    }

    #[test]
    fn scan_rust_file_classifies_blocks_rs_trap_sites_by_cfg_context() {
        let source = include_str!("../../wildside-core/src/blocks.rs");
        let sites = scan_rust_file_for_citations("crates/wildside-core/src/blocks.rs", source);
        let has_context = |label: &str, context: RustCodeContext| {
            sites
                .iter()
                .any(|site| site.label == label && site.citation.context == context)
        };

        assert!(has_context("loc_10228", RustCodeContext::Production));
        assert!(has_context("loc_FF92", RustCodeContext::Production));
        assert!(has_context("loc_100F6", RustCodeContext::Test));
    }

    #[test]
    fn native_citation_index_allows_same_file_to_be_both_prod_and_test() {
        let sites_by_label = BTreeMap::from([(
            "loc_100F6".to_owned(),
            BTreeSet::from([
                NativeCitationSite {
                    path: "crates/wildside-core/src/blocks.rs".to_owned(),
                    line_number: 1692,
                    context: RustCodeContext::Production,
                },
                NativeCitationSite {
                    path: "crates/wildside-core/src/blocks.rs".to_owned(),
                    line_number: 5698,
                    context: RustCodeContext::Test,
                },
            ]),
        )]);

        let index = index_native_citation_sites(sites_by_label);
        let breakdown = index
            .breakdown_by_label
            .get("loc_100F6")
            .expect("breakdown present");

        assert_eq!(
            breakdown.implementation,
            vec!["crates/wildside-core/src/blocks.rs"]
        );
        assert_eq!(breakdown.tests, vec!["crates/wildside-core/src/blocks.rs"]);
        assert_eq!(
            index
                .files_by_label
                .get("loc_100F6")
                .expect("file list present")
                .iter()
                .cloned()
                .collect::<Vec<_>>(),
            vec!["crates/wildside-core/src/blocks.rs"]
        );
    }

    #[test]
    fn coverage_state_uses_reachability_not_oracle_backing() {
        let cited_in = vec![
            "crates/wildside-core/src/level.rs".to_owned(),
            "crates/wildside-core/src/tests.rs".to_owned(),
        ];
        let mut overlay = AnnotationOverlay::default();
        overlay.parity_evidence.insert(
            "loc_1234".to_owned(),
            vec!["verify-tas --frame-start 10 --frame-end 20".to_owned()],
        );

        assert_eq!(
            coverage_state_for_label(&BTreeMap::new(), &overlay, "loc_1234", &cited_in),
            CoverageState::Reachable
        );
    }

    #[test]
    fn coverage_state_separates_reachable_test_only_uncovered_and_out_of_scope_labels() {
        let citation_breakdowns = BTreeMap::from([
            (
                "loc_1111".to_owned(),
                CitationBreakdown {
                    implementation: vec!["crates/wildside-core/src/level.rs".to_owned()],
                    tests: vec!["crates/wildside-core/src/tests.rs".to_owned()],
                    docs: Vec::new(),
                    support: Vec::new(),
                },
            ),
            (
                "loc_2222".to_owned(),
                CitationBreakdown {
                    implementation: Vec::new(),
                    tests: vec!["crates/wildside-core/src/tests.rs".to_owned()],
                    docs: Vec::new(),
                    support: Vec::new(),
                },
            ),
            (
                "sub_E13DA".to_owned(),
                CitationBreakdown {
                    implementation: vec!["crates/wildside-md/src/lib.rs".to_owned()],
                    tests: Vec::new(),
                    docs: Vec::new(),
                    support: Vec::new(),
                },
            ),
        ]);
        let overlay = AnnotationOverlay::default();

        assert_eq!(
            coverage_state_for_label(
                &citation_breakdowns,
                &overlay,
                "loc_1111",
                &["crates/wildside-core/src/level.rs".to_owned()]
            ),
            CoverageState::Reachable
        );
        assert_eq!(
            coverage_state_for_label(
                &citation_breakdowns,
                &overlay,
                "loc_2222",
                &["crates/wildside-core/src/tests.rs".to_owned()]
            ),
            CoverageState::TestOnlyDebt
        );
        assert_eq!(
            coverage_state_for_label(&citation_breakdowns, &overlay, "loc_3333", &[]),
            CoverageState::Uncovered
        );
        assert_eq!(
            coverage_state_for_label(
                &citation_breakdowns,
                &overlay,
                "sub_E13DA",
                &["crates/wildside-md/src/lib.rs".to_owned()]
            ),
            CoverageState::Reachable
        );
        assert_eq!(
            coverage_state_for_label(&citation_breakdowns, &overlay, "sub_E1416", &[]),
            CoverageState::EmulatedGems
        );
        assert_eq!(
            coverage_state_for_label(&citation_breakdowns, &overlay, "loc_32454", &[]),
            CoverageState::NonExecutableDataAlias
        );
    }

    #[test]
    fn uncovered_report_includes_loc_gaps_and_out_of_scope_gems_labels() {
        let uncovered_loc = make_label_with_state("loc_3333", CoverageState::Uncovered);
        let test_only_loc = make_label_with_state("loc_3AA4C", CoverageState::TestOnlyDebt);
        let mut gems_sub = make_label_with_state("sub_E1416", CoverageState::EmulatedGems);
        gems_sub.kind = LabelKind::Sub;
        let gems_loc = make_label_with_state("loc_E136C", CoverageState::EmulatedGems);

        assert!(appears_in_uncovered_report(&uncovered_loc));
        assert!(!appears_in_uncovered_report(&test_only_loc));
        assert!(appears_in_uncovered_report(&gems_sub));
        assert!(!appears_in_uncovered_report(&gems_loc));
    }

    #[test]
    fn coverage_summary_counts_reachable_test_only_uncovered_and_gems() {
        let labels = vec![
            make_label(
                "loc_1111",
                LabelKind::Loc,
                &["crates/wildside-core/src/level.rs"],
            ),
            make_label(
                "loc_2222",
                LabelKind::Loc,
                &["crates/wildside-core/src/tests.rs"],
            ),
            make_label("loc_3333", LabelKind::Loc, &[]),
            make_label(
                "sub_E13DA",
                LabelKind::Sub,
                &["crates/wildside-md/src/lib.rs"],
            ),
            make_label("sub_E1416", LabelKind::Sub, &[]),
        ];
        let citation_breakdowns = BTreeMap::from([
            (
                "loc_1111".to_owned(),
                CitationBreakdown {
                    implementation: vec!["crates/wildside-core/src/level.rs".to_owned()],
                    tests: Vec::new(),
                    docs: Vec::new(),
                    support: Vec::new(),
                },
            ),
            (
                "loc_2222".to_owned(),
                CitationBreakdown {
                    implementation: Vec::new(),
                    tests: vec!["crates/wildside-core/src/tests.rs".to_owned()],
                    docs: Vec::new(),
                    support: Vec::new(),
                },
            ),
            (
                "sub_E13DA".to_owned(),
                CitationBreakdown {
                    implementation: vec!["crates/wildside-md/src/lib.rs".to_owned()],
                    tests: Vec::new(),
                    docs: Vec::new(),
                    support: Vec::new(),
                },
            ),
        ]);

        let map = build_coverage_map(labels, &BTreeMap::new(), &citation_breakdowns);

        assert_eq!(map.summary.reachable_labels, 2);
        assert_eq!(map.summary.test_only_debt_labels, 1);
        assert_eq!(map.summary.uncovered_labels, 1);
        assert_eq!(map.summary.emulated_gems_labels, 1);
        assert_eq!(map.summary.non_executable_data_alias_labels, 0);
        assert_eq!(map.summary.non_emulated_uncovered_labels, 1);
        assert_eq!(
            map.subsystems["kid_main"].labels[3].coverage_state,
            Some(CoverageState::Reachable)
        );
        assert_eq!(
            map.subsystems["kid_main"].labels[4].coverage_state,
            Some(CoverageState::EmulatedGems)
        );
    }
}
