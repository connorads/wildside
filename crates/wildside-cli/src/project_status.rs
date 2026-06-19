use anyhow::{Context, Result, bail};
use clap::ValueEnum;
use serde::Serialize;
use std::fs;
use std::io::{self, Write};
use std::path::Path;

const BACKLOG_PATH: &str = "docs/source-port/backlog.md";

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
    ready_to_implement_items: Vec<BacklogItem>,
    top_open_playability_items: Vec<BacklogItem>,
    top_open_visual_items: Vec<BacklogItem>,
    human_validation_blockers: Vec<HumanValidationBlocker>,
    warnings: Vec<String>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
struct GateSignal {
    name: String,
    passed: bool,
    detail: String,
    path: String,
    failing_step: Option<String>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
struct BacklogItem {
    id: String,
    priority: String,
    system: String,
    summary: String,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
struct HumanValidationBlocker {
    id: String,
    status: String,
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
struct GateReport {
    passed: bool,
    #[serde(default)]
    steps: Vec<GateStep>,
}

#[derive(Debug, Clone, serde::Deserialize)]
struct GateStep {
    name: String,
    status: String,
    detail: String,
}

pub(crate) fn run_project_status(
    repo_root: &Path,
    format: ProjectStatusFormat,
    fail_on_human_blockers: bool,
) -> Result<()> {
    let report = build_project_status_report(repo_root)?;
    match format {
        ProjectStatusFormat::Text => print_project_status(&report),
        ProjectStatusFormat::Json => println!("{}", serde_json::to_string_pretty(&report)?),
    }
    if fail_on_human_blockers && !report.human_validation_blockers.is_empty() {
        let _ = io::stdout().flush();
        bail!(
            "{} human validation/repro blocker(s) remain",
            report.human_validation_blockers.len()
        );
    }
    Ok(())
}

fn build_project_status_report(repo_root: &Path) -> Result<ProjectStatusReport> {
    let playable_gate = load_gate_signal(repo_root, "1p-playable");
    let accurate_gate = load_gate_signal(repo_root, "1p-accurate");
    let ready_to_implement_items = load_ready_to_implement_items(repo_root, 5)?;
    let top_open_playability_items = load_top_open_items(repo_root, 5, BacklogFocus::Playability)?;
    let top_open_visual_items = load_top_open_items(repo_root, 5, BacklogFocus::VisualParity)?;
    let human_validation_blockers = load_human_validation_blockers(repo_root)?;

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
    if !human_validation_blockers.is_empty() {
        warnings.push(format!(
            "{} human validation/repro blocker(s) remain; only a human playtest can close [?]/[b] rows",
            human_validation_blockers.len()
        ));
    }

    Ok(ProjectStatusReport {
        headline_gate: playable_gate.name.clone(),
        headline_passed: playable_gate.passed,
        secondary_gates: vec![accurate_gate],
        ready_to_implement_items,
        top_open_playability_items,
        top_open_visual_items,
        human_validation_blockers,
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
                failing_step: None,
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
                failing_step: None,
            };
        }
    };
    let failing_step = report
        .steps
        .iter()
        .find(|step| step.status.eq_ignore_ascii_case("failed"))
        .map(|step| format!("{}: {}", step.name, step.detail));
    GateSignal {
        name: gate_name.to_owned(),
        passed: report.passed,
        detail: if report.passed {
            "latest cached report passed".to_owned()
        } else {
            failing_step
                .clone()
                .unwrap_or_else(|| "latest cached report failed".to_owned())
        },
        path: path.display().to_string(),
        failing_step,
    }
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

fn backlog_row_has_root_cause(summary: &str, verify: &str) -> bool {
    let summary = summary.to_ascii_lowercase();
    let verify = verify.to_ascii_lowercase();
    summary.contains("root cause identified") || verify.contains("root cause identified")
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

fn load_ready_to_implement_items(repo_root: &Path, limit: usize) -> Result<Vec<BacklogItem>> {
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
        if columns.len() < 6 {
            continue;
        }
        let status = columns[1];
        let priority = columns[2];
        if !matches!(status, "[ ]" | "[~]") || !matches!(priority, "P0" | "P1" | "P2") {
            continue;
        }
        if !backlog_row_has_root_cause(columns[4], columns[5]) {
            continue;
        }
        items.push(BacklogItem {
            id: columns[0].to_owned(),
            priority: priority.to_owned(),
            system: columns[3].to_owned(),
            summary: columns[4].to_owned(),
        });
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

fn load_human_validation_blockers(repo_root: &Path) -> Result<Vec<HumanValidationBlocker>> {
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
        if !matches!(status, "[?]" | "[b]") || !matches!(priority, "P0" | "P1" | "P2") {
            continue;
        }
        items.push(HumanValidationBlocker {
            id: columns[0].to_owned(),
            status: status.to_owned(),
            priority: priority.to_owned(),
            system: columns[3].to_owned(),
            summary: columns[4].to_owned(),
        });
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
    println!("  ready to implement now:");
    if report.ready_to_implement_items.is_empty() {
        println!("    - none");
    } else {
        for item in &report.ready_to_implement_items {
            println!(
                "    - #{} {} {}: {}",
                item.id, item.priority, item.system, item.summary
            );
        }
    }
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
    println!("  human validation/repro blockers:");
    if report.human_validation_blockers.is_empty() {
        println!("    - none");
    } else {
        for item in &report.human_validation_blockers {
            println!(
                "    - #{} {} {} {}: {}",
                item.id, item.status, item.priority, item.system, item.summary
            );
        }
        println!("    - review: human playtest required to close [?]/[b] rows");
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
        BacklogFocus, ProjectStatusFormat, build_project_status_report,
        load_human_validation_blockers, load_ready_to_implement_items, load_top_open_items,
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
    fn project_status_extracts_ready_to_implement_items() {
        let temp_dir = TempDir::new().expect("temp dir");
        fs::create_dir_all(temp_dir.path().join("docs/source-port")).expect("docs dir");
        fs::write(
            temp_dir.path().join("docs/source-port/backlog.md"),
            "\
| # | Status | Pri | System | Summary | Verify |
|-----|--------|-----|--------|---------|--------|
| 144 | [ ] | P1 | enemy | Level 2 shooter enemy fires too rapidly. Root cause identified. | verify |
| 145 | [ ] | P1 | render | Level 2 shooter enemy is not rendered correctly. Root cause identified. | verify |
| 146 | [ ] | P1 | level | Needs human repro | no |
",
        )
        .expect("backlog");

        let items = load_ready_to_implement_items(temp_dir.path(), 5).expect("items");
        assert_eq!(items.len(), 2);
        assert_eq!(items[0].id, "144");
        assert_eq!(items[1].id, "145");
    }

    #[test]
    fn project_status_extracts_human_validation_blockers() {
        let temp_dir = TempDir::new().expect("temp dir");
        fs::create_dir_all(temp_dir.path().join("docs/source-port")).expect("docs dir");
        fs::write(
            temp_dir.path().join("docs/source-port/backlog.md"),
            "\
| # | Status | Pri | System | Summary | Verify |
|-----|--------|-----|--------|---------|--------|
| 135 | [?] | P1 | render | Level 1 background needs validation | review |
| 146 | [b] | P1 | level | Level 2 death needs repro | review |
| 147 | [x] | P1 | render | Closed validation row | done |
| 148 | [ ] | P1 | player | Agent-actionable row | fix |
",
        )
        .expect("backlog");

        let items = load_human_validation_blockers(temp_dir.path()).expect("items");
        assert_eq!(items.len(), 2);
        assert_eq!(items[0].id, "135");
        assert_eq!(items[0].status, "[?]");
        assert_eq!(items[1].id, "146");
        assert_eq!(items[1].status, "[b]");
    }

    #[test]
    fn project_status_reports_missing_headline_gate() {
        let temp_dir = TempDir::new().expect("temp dir");
        fs::create_dir_all(temp_dir.path().join("docs/source-port")).expect("docs dir");
        fs::create_dir_all(temp_dir.path().join(".cache/release-gate/1p-accurate"))
            .expect("gate dir");
        fs::write(
            temp_dir.path().join("docs/source-port/backlog.md"),
            "\
| # | Status | Pri | System | Summary | Verify |
|-----|--------|-----|--------|---------|--------|
| 123 | [ ] | P1 | player | core playability bug | verify |
| 135 | [?] | P1 | render | human visual blocker | review |
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
        assert_eq!(report.human_validation_blockers.len(), 1);
        assert!(
            report
                .warnings
                .iter()
                .any(|warning| warning.contains("1p-playable"))
        );
        assert!(
            report
                .warnings
                .iter()
                .any(|warning| warning.contains("human validation/repro blocker"))
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

        run_project_status(temp_dir.path(), ProjectStatusFormat::Text, false).expect("text output");
        run_project_status(temp_dir.path(), ProjectStatusFormat::Json, false).expect("json output");
    }

    #[test]
    fn fail_on_human_blockers_returns_error_after_rendering_status() {
        let temp_dir = TempDir::new().expect("temp dir");
        fs::create_dir_all(temp_dir.path().join("docs/source-port")).expect("docs dir");
        fs::create_dir_all(temp_dir.path().join(".cache/release-gate/1p-playable"))
            .expect("playable gate dir");
        fs::create_dir_all(temp_dir.path().join(".cache/release-gate/1p-accurate"))
            .expect("accurate gate dir");
        fs::write(
            temp_dir.path().join("docs/source-port/backlog.md"),
            "\
| # | Status | Pri | System | Summary | Verify |
|-----|--------|-----|--------|---------|--------|
| 135 | [?] | P1 | render | human visual blocker | review |
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

        let error = run_project_status(temp_dir.path(), ProjectStatusFormat::Text, true)
            .expect_err("human blockers fail completion audit");
        assert!(error.to_string().contains("human validation/repro blocker"));
    }
}
