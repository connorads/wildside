use anyhow::{Context, Result};
use clap::ValueEnum;
use serde::Serialize;
use std::fs;
use std::path::Path;

const BACKLOG_PATH: &str = "docs/source-port/backlog.md";
const FRONTIER_STATE_PATH: &str = "docs/source-port/frontier-state.yaml";
const NEXT_AGENT_NOTE_PATH: &str = ".cache/autonomy/source-port-frontier/next-agent.md";

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub(crate) enum ProjectStatusFormat {
    Text,
    Json,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub(crate) struct ProjectStatusReport {
    headline_gate: String,
    headline_passed: bool,
    secondary_gates: Vec<GateSignal>,
    tas_frontier: TasFrontierStatus,
    top_open_playability_items: Vec<BacklogItem>,
    top_open_visual_items: Vec<BacklogItem>,
    warnings: Vec<String>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
struct GateSignal {
    name: String,
    passed: bool,
    detail: String,
    path: String,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
struct TasFrontierStatus {
    slug: String,
    status: String,
    recording: String,
    authoritative_clean_through: u32,
    note_status: String,
    summary: String,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
struct BacklogItem {
    id: String,
    priority: String,
    system: String,
    summary: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum BacklogFocus {
    Playability,
    VisualParity,
}

#[derive(Debug, Clone, serde::Deserialize)]
struct FrontierState {
    active_frontier: ActiveFrontierState,
}

#[derive(Debug, Clone, serde::Deserialize)]
struct ActiveFrontierState {
    slug: String,
    status: String,
    recording: String,
    authoritative_clean_through: u32,
    area: String,
}

#[derive(Debug, Clone, serde::Deserialize)]
struct GateReport {
    passed: bool,
}

pub(crate) fn run_project_status(repo_root: &Path, format: ProjectStatusFormat) -> Result<()> {
    let report = build_project_status_report(repo_root)?;
    match format {
        ProjectStatusFormat::Text => print_project_status(&report),
        ProjectStatusFormat::Json => println!("{}", serde_json::to_string_pretty(&report)?),
    }
    Ok(())
}

fn build_project_status_report(repo_root: &Path) -> Result<ProjectStatusReport> {
    let playable_gate = load_gate_signal(repo_root, "1p-playable");
    let accurate_gate = load_gate_signal(repo_root, "1p-accurate");
    let tas_frontier = load_tas_frontier(repo_root)?;
    let top_open_playability_items = load_top_open_items(repo_root, 5, BacklogFocus::Playability)?;
    let top_open_visual_items = load_top_open_items(repo_root, 5, BacklogFocus::VisualParity)?;

    let mut warnings = Vec::new();
    if !playable_gate.passed {
        warnings.push(format!(
            "headline gate `{}` is missing or failing: {}",
            playable_gate.name, playable_gate.detail
        ));
    }
    if !accurate_gate.passed {
        warnings.push(format!(
            "supporting gate `{}` is missing or failing: {}",
            accurate_gate.name, accurate_gate.detail
        ));
    }
    if tas_frontier.note_status != "current" {
        warnings.push(format!(
            "source-port frontier handoff note is {}",
            tas_frontier.note_status
        ));
    }
    if tas_frontier.status == "complete" {
        warnings.push(
            "TAS frontier is closed; treat it as replay-matched evidence, not overall completion"
                .to_owned(),
        );
    }

    Ok(ProjectStatusReport {
        headline_gate: playable_gate.name.clone(),
        headline_passed: playable_gate.passed,
        secondary_gates: vec![accurate_gate],
        tas_frontier,
        top_open_playability_items,
        top_open_visual_items,
        warnings,
    })
}

fn load_gate_signal(repo_root: &Path, gate_name: &str) -> GateSignal {
    let path = repo_root
        .join(".cache/release-gate")
        .join(gate_name)
        .join("latest.json");
    let bytes = match fs::read(&path) {
        Ok(bytes) => bytes,
        Err(error) => {
            return GateSignal {
                name: gate_name.to_owned(),
                passed: false,
                detail: format!("missing gate artefact: {error}"),
                path: path.display().to_string(),
            };
        }
    };
    let report: GateReport = match serde_json::from_slice(&bytes) {
        Ok(report) => report,
        Err(error) => {
            return GateSignal {
                name: gate_name.to_owned(),
                passed: false,
                detail: format!("failed to parse gate artefact: {error}"),
                path: path.display().to_string(),
            };
        }
    };
    GateSignal {
        name: gate_name.to_owned(),
        passed: report.passed,
        detail: if report.passed {
            "latest cached report passed".to_owned()
        } else {
            "latest cached report failed".to_owned()
        },
        path: path.display().to_string(),
    }
}

fn load_tas_frontier(repo_root: &Path) -> Result<TasFrontierStatus> {
    let path = repo_root.join(FRONTIER_STATE_PATH);
    let bytes = fs::read(&path)
        .with_context(|| format!("failed to read frontier state {}", path.display()))?;
    let state: FrontierState = serde_json::from_slice(&bytes)
        .with_context(|| format!("failed to parse frontier state {}", path.display()))?;
    let note_path = repo_root.join(NEXT_AGENT_NOTE_PATH);
    let note_status = next_agent_note_status(&note_path, &state.active_frontier.slug);
    let summary = if state.active_frontier.status == "complete" {
        format!(
            "{} is replay-matched through frame {}; broader 1P work remains open",
            state.active_frontier.recording, state.active_frontier.authoritative_clean_through
        )
    } else {
        state.active_frontier.area.clone()
    };
    Ok(TasFrontierStatus {
        slug: state.active_frontier.slug,
        status: state.active_frontier.status,
        recording: state.active_frontier.recording,
        authoritative_clean_through: state.active_frontier.authoritative_clean_through,
        note_status,
        summary,
    })
}

fn next_agent_note_status(note_path: &Path, active_slug: &str) -> String {
    let contents = match fs::read_to_string(note_path) {
        Ok(contents) => contents,
        Err(_) => return "missing".to_owned(),
    };
    match extract_frontmatter_slug(&contents).as_deref() {
        Some(slug) if slug == active_slug => "current".to_owned(),
        Some(_) => "stale".to_owned(),
        None => "stale".to_owned(),
    }
}

fn extract_frontmatter_slug(contents: &str) -> Option<String> {
    let mut lines = contents.lines();
    if lines.next()? != "---" {
        return None;
    }
    for line in lines {
        if line == "---" {
            break;
        }
        if let Some((key, value)) = line.split_once(':')
            && key.trim() == "slug"
        {
            return Some(value.trim().to_owned());
        }
    }
    None
}

fn classify_backlog_focus(system: &str, summary: &str) -> BacklogFocus {
    let system = system.to_ascii_lowercase();
    let summary = summary.to_ascii_lowercase();
    if matches!(system.as_str(), "render" | "menu")
        || summary.contains("graphic")
        || summary.contains("animation")
        || summary.contains("sprite")
        || summary.contains("background")
        || summary.contains("render")
        || summary.contains("intro")
    {
        BacklogFocus::VisualParity
    } else {
        BacklogFocus::Playability
    }
}

fn load_top_open_items(
    repo_root: &Path,
    limit: usize,
    focus: BacklogFocus,
) -> Result<Vec<BacklogItem>> {
    let path = repo_root.join(BACKLOG_PATH);
    let contents = fs::read_to_string(&path)
        .with_context(|| format!("failed to read backlog {}", path.display()))?;
    let mut items = Vec::new();
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
        let system = columns[3];
        let summary = columns[4];
        if classify_backlog_focus(system, summary) != focus {
            continue;
        }
        items.push(BacklogItem {
            id: columns[0].to_owned(),
            priority: priority.to_owned(),
            system: system.to_owned(),
            summary: summary.to_owned(),
        });
        if items.len() >= limit {
            continue;
        }
    }
    items.sort_by(|left, right| {
        fn priority_rank(priority: &str) -> u8 {
            match priority {
                "P0" => 0,
                "P1" => 1,
                "P2" => 2,
                _ => 3,
            }
        }
        priority_rank(&left.priority)
            .cmp(&priority_rank(&right.priority))
            .then_with(|| left.id.cmp(&right.id))
    });
    items.truncate(limit);
    Ok(items)
}

fn print_project_status(report: &ProjectStatusReport) {
    println!("Project Status");
    println!(
        "  headline: {} ({})",
        report.headline_gate,
        if report.headline_passed {
            "passed"
        } else {
            "missing/failing"
        }
    );
    println!("  supporting gates:");
    for gate in &report.secondary_gates {
        println!(
            "    - {}: {}",
            gate.name,
            if gate.passed {
                "passed"
            } else {
                "missing/failing"
            }
        );
    }
    println!("  TAS frontier:");
    println!(
        "    - {} [{}] clean through frame {}",
        report.tas_frontier.recording,
        report.tas_frontier.status,
        report.tas_frontier.authoritative_clean_through
    );
    println!("    - handoff note: {}", report.tas_frontier.note_status);
    println!("    - summary: {}", report.tas_frontier.summary);
    println!("  top open playability risks:");
    if report.top_open_playability_items.is_empty() {
        println!("    - none");
    } else {
        for item in &report.top_open_playability_items {
            println!(
                "    - #{} {} {}: {}",
                item.id, item.priority, item.system, item.summary
            );
        }
    }
    println!("  top open visual risks:");
    if report.top_open_visual_items.is_empty() {
        println!("    - none");
    } else {
        for item in &report.top_open_visual_items {
            println!(
                "    - #{} {} {}: {}",
                item.id, item.priority, item.system, item.summary
            );
        }
    }
    println!("  warnings:");
    if report.warnings.is_empty() {
        println!("    - none");
    } else {
        for warning in &report.warnings {
            println!("    - {warning}");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{
        BacklogFocus, ProjectStatusFormat, build_project_status_report, load_top_open_items,
        run_project_status,
    };
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn backlog_parser_prefers_open_high_priority_items() {
        let temp_dir = TempDir::new().expect("temp dir");
        fs::create_dir_all(temp_dir.path().join("docs/source-port")).expect("docs dir");
        fs::write(
            temp_dir.path().join("docs/source-port/backlog.md"),
            "\
| # | Status | Pri | System | Summary | Verify |
|-----|--------|-----|--------|---------|--------|
| 001 | [x] | P0 | foo | closed item | no |
| 002 | [ ] | P2 | foo | low priority | no |
| 003 | [~] | P1 | player | important risk | no |
| 004 | [ ] | P0 | camera | urgent bug | no |
| 005 | [ ] | P1 | render | wrong intro graphics | no |
",
        )
        .expect("backlog");

        let items =
            load_top_open_items(temp_dir.path(), 5, BacklogFocus::Playability).expect("items");
        assert_eq!(items.len(), 3);
        assert_eq!(items[0].id, "004");
        assert_eq!(items[1].id, "003");
        assert_eq!(items[2].id, "002");

        let visual_items =
            load_top_open_items(temp_dir.path(), 5, BacklogFocus::VisualParity).expect("items");
        assert_eq!(visual_items.len(), 1);
        assert_eq!(visual_items[0].id, "005");
    }

    #[test]
    fn project_status_reports_missing_headline_gate() {
        let temp_dir = TempDir::new().expect("temp dir");
        fs::create_dir_all(temp_dir.path().join("docs/source-port")).expect("docs dir");
        fs::create_dir_all(temp_dir.path().join(".cache/release-gate/1p-accurate"))
            .expect("gate dir");
        fs::write(
            temp_dir.path().join("docs/source-port/frontier-state.yaml"),
            r#"{"active_frontier":{"slug":"frontier","status":"complete","recording":"any-percent-published","authoritative_clean_through":37759,"area":"closed frontier"}}"#,
        )
        .expect("frontier");
        fs::write(
            temp_dir.path().join("docs/source-port/backlog.md"),
            "\
| # | Status | Pri | System | Summary | Verify |
|-----|--------|-----|--------|---------|--------|
| 123 | [ ] | P1 | player | core playability bug | verify |
",
        )
        .expect("backlog");
        fs::write(
            temp_dir
                .path()
                .join(".cache/release-gate/1p-accurate/latest.json"),
            r#"{"passed":true}"#,
        )
        .expect("accurate gate");

        let report = build_project_status_report(temp_dir.path()).expect("report");
        assert_eq!(report.headline_gate, "1p-playable");
        assert!(!report.headline_passed);
        assert_eq!(report.secondary_gates[0].name, "1p-accurate");
        assert_eq!(report.top_open_playability_items.len(), 1);
        assert!(
            report
                .warnings
                .iter()
                .any(|warning| warning.contains("1p-playable"))
        );
    }

    #[test]
    fn text_and_json_status_commands_run() {
        let temp_dir = TempDir::new().expect("temp dir");
        fs::create_dir_all(temp_dir.path().join("docs/source-port")).expect("docs dir");
        fs::create_dir_all(temp_dir.path().join(".cache/release-gate/1p-playable"))
            .expect("playable gate dir");
        fs::create_dir_all(temp_dir.path().join(".cache/release-gate/1p-accurate"))
            .expect("accurate gate dir");
        fs::create_dir_all(temp_dir.path().join(".cache/autonomy/source-port-frontier"))
            .expect("autonomy dir");
        fs::write(
            temp_dir.path().join("docs/source-port/frontier-state.yaml"),
            r#"{"active_frontier":{"slug":"frontier","status":"active","recording":"any-percent-published","authoritative_clean_through":100,"area":"active"}} "#,
        )
        .expect("frontier");
        fs::write(
            temp_dir
                .path()
                .join(".cache/autonomy/source-port-frontier/next-agent.md"),
            "---\nslug: frontier\n---\n",
        )
        .expect("note");
        fs::write(
            temp_dir.path().join("docs/source-port/backlog.md"),
            "\
| # | Status | Pri | System | Summary | Verify |
|-----|--------|-----|--------|---------|--------|
| 122 | [ ] | P1 | render | level intro graphics wrong | verify |
| 123 | [ ] | P1 | player | core playability bug | verify |
",
        )
        .expect("backlog");
        fs::write(
            temp_dir
                .path()
                .join(".cache/release-gate/1p-playable/latest.json"),
            r#"{"passed":true}"#,
        )
        .expect("playable gate");
        fs::write(
            temp_dir
                .path()
                .join(".cache/release-gate/1p-accurate/latest.json"),
            r#"{"passed":true}"#,
        )
        .expect("accurate gate");

        run_project_status(temp_dir.path(), ProjectStatusFormat::Text).expect("text output");
        run_project_status(temp_dir.path(), ProjectStatusFormat::Json).expect("json output");
    }
}
