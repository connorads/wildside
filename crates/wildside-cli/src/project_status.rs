use crate::backlog::{AWAITING_HUMAN_PATH, read_live_lanes};
use anyhow::{Result, bail};
use clap::ValueEnum;
use serde::Serialize;
use std::fmt::Write as _;
use std::fs;
use std::io::{self, Write};
use std::path::Path;

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
    target: String,
    check: String,
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
        ProjectStatusFormat::Text => {
            print_project_status(&report);
        }
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
    let human_validation_blockers = load_human_validation_blockers(repo_root);

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
    let unknown_targets = human_validation_blockers
        .iter()
        .filter(|item| item.target == "unknown")
        .count();
    if unknown_targets > 0 {
        warnings.push(format!(
            "{unknown_targets} human validation/repro blocker(s) have unknown targets"
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
    let text = format!("{summary} {verify}").to_ascii_lowercase();
    // The documented marker is "root cause identified"; also accept the natural variant
    // "root-caused" so rows do not have to use the exact phrase to be picked up.
    text.contains("root cause identified") || text.contains("root-caused")
}

fn load_top_open_items(
    repo_root: &Path,
    limit: usize,
    focus: BacklogFocus,
) -> Result<Vec<BacklogItem>> {
    let contents = read_live_lanes(repo_root)?;
    let mut items = Vec::new();
    for line in contents.lines() {
        if !line.starts_with('|') {
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
        // Every open row is selectable; P3 ranks last so P0-P2 always come first, but P3
        // is no longer permanently invisible to the CLI-driven loop.
        if !matches!(status, "[ ]" | "[~]") || !matches!(priority, "P0" | "P1" | "P2" | "P3") {
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
    let contents = read_live_lanes(repo_root)?;
    let mut items = Vec::new();
    for line in contents.lines() {
        if !line.starts_with('|') {
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
        if !matches!(status, "[ ]" | "[~]") || !matches!(priority, "P0" | "P1" | "P2" | "P3") {
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

fn load_human_validation_blockers(repo_root: &Path) -> Vec<HumanValidationBlocker> {
    let contents =
        fs::read_to_string(repo_root.join(AWAITING_HUMAN_PATH)).unwrap_or_else(|_| String::new());
    let mut items = Vec::new();
    for line in contents.lines() {
        if !line.starts_with('|') {
            continue;
        }
        let columns = line
            .split('|')
            .map(str::trim)
            .filter(|column| !column.is_empty())
            .collect::<Vec<_>>();
        if columns.len() != 8 {
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
            target: columns[4].to_owned(),
            check: columns[5].to_owned(),
            summary: columns[6].to_owned(),
        });
    }
    items.sort_by(|left, right| {
        fn priority_rank(priority: &str) -> u8 {
            match priority {
                "P0" => 0,
                "P1" => 1,
                _ => 2,
            }
        }
        priority_rank(&left.priority)
            .cmp(&priority_rank(&right.priority))
            .then_with(|| left.id.cmp(&right.id))
    });
    items
}

fn print_project_status(report: &ProjectStatusReport) -> String {
    let output = render_project_status(report);
    print!("{output}");
    output
}

fn render_project_status(report: &ProjectStatusReport) -> String {
    let mut output = String::new();
    let _ = writeln!(output, "Project Status");
    let _ = writeln!(
        output,
        "  headline: {} ({})",
        report.headline_gate,
        if report.headline_passed {
            "passed"
        } else {
            "missing/failing"
        }
    );
    let _ = writeln!(output, "  supporting gates:");
    for gate in &report.secondary_gates {
        let _ = writeln!(
            output,
            "    - {}: {}",
            gate.name,
            if gate.passed {
                "passed"
            } else {
                "missing/failing"
            }
        );
    }
    let _ = writeln!(output, "  ready to implement now:");
    if report.ready_to_implement_items.is_empty() {
        let _ = writeln!(output, "    - none");
    } else {
        for item in &report.ready_to_implement_items {
            let _ = writeln!(
                output,
                "    - #{} {} {}: {}",
                item.id, item.priority, item.system, item.summary
            );
        }
    }
    let _ = writeln!(output, "  top open playability risks:");
    if report.top_open_playability_items.is_empty() {
        let _ = writeln!(output, "    - none");
    } else {
        for item in &report.top_open_playability_items {
            let _ = writeln!(
                output,
                "    - #{} {} {}: {}",
                item.id, item.priority, item.system, item.summary
            );
        }
    }
    let _ = writeln!(output, "  top open visual risks:");
    if report.top_open_visual_items.is_empty() {
        let _ = writeln!(output, "    - none");
    } else {
        for item in &report.top_open_visual_items {
            let _ = writeln!(
                output,
                "    - #{} {} {}: {}",
                item.id, item.priority, item.system, item.summary
            );
        }
    }
    let _ = writeln!(output, "  human validation/repro blockers:");
    if report.human_validation_blockers.is_empty() {
        let _ = writeln!(output, "    - none");
    } else {
        for item in &report.human_validation_blockers {
            let _ = writeln!(
                output,
                "    - #{} {} {} {} [{}]: {}",
                item.id, item.status, item.priority, item.system, item.target, item.check
            );
        }
        let _ = writeln!(
            output,
            "    - how to use: run `mise run play-native`; Target says where to play, Check says what to confirm"
        );
        let _ = writeln!(
            output,
            "    - review: human playtest required to close [?]/[b] rows"
        );
    }
    let _ = writeln!(output, "  warnings:");
    if report.warnings.is_empty() {
        let _ = writeln!(output, "    - none");
    } else {
        for warning in &report.warnings {
            let _ = writeln!(output, "    - {warning}");
        }
    }
    output
}

#[cfg(test)]
mod tests {
    use super::{
        BacklogFocus, BacklogItem, GateSignal, HumanValidationBlocker, ProjectStatusFormat,
        ProjectStatusReport, build_project_status_report, classify_backlog_focus,
        load_human_validation_blockers, load_ready_to_implement_items, load_top_open_items,
        print_project_status, render_project_status, run_project_status,
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
    fn visual_focus_classifier_covers_system_and_summary_markers() {
        for (system, summary) in [
            ("render", "ordinary row"),
            ("menu", "ordinary row"),
            ("level", "graphic corruption"),
            ("level", "animation phase drift"),
            ("enemy", "sprite frame wrong"),
            ("level", "background seam"),
            ("player", "render ordering"),
            ("level", "intro boot story"),
        ] {
            assert_eq!(
                classify_backlog_focus(system, summary),
                BacklogFocus::VisualParity,
                "{system}/{summary} should be visual"
            );
        }
        assert_eq!(
            classify_backlog_focus("player", "collision route root-caused"),
            BacklogFocus::Playability
        );
    }

    #[test]
    fn open_item_parser_skips_non_rows_headers_and_malformed_rows() {
        let temp_dir = TempDir::new().expect("temp dir");
        fs::create_dir_all(temp_dir.path().join("docs/source-port")).expect("docs dir");
        fs::write(
            temp_dir.path().join("docs/source-port/backlog.md"),
            "\
not a table row
| # | Status | Pri | System | Summary | Verify |
|-----|--------|-----|--------|---------|--------|
| 001 | [ ] | P0 | player |
| 002 | [ ] | P0 | player | selectable row |
",
        )
        .expect("backlog");

        let items =
            load_top_open_items(temp_dir.path(), 5, BacklogFocus::Playability).expect("items");
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].id, "002");
    }

    #[test]
    fn open_item_priority_ranking_keeps_p2_ahead_of_p3() {
        let temp_dir = TempDir::new().expect("temp dir");
        fs::create_dir_all(temp_dir.path().join("docs/source-port")).expect("docs dir");
        fs::write(
            temp_dir.path().join("docs/source-port/backlog.md"),
            "\
| # | Status | Pri | System | Summary | Verify |
|-----|--------|-----|--------|---------|--------|
| 020 | [ ] | P3 | player | p3 row sorts last | verify |
| 030 | [ ] | P2 | player | p2 row sorts before p3 | verify |
| 010 | [ ] | P1 | player | p1 row sorts first | verify |
",
        )
        .expect("backlog");

        let items =
            load_top_open_items(temp_dir.path(), 5, BacklogFocus::Playability).expect("items");
        assert_eq!(
            items
                .iter()
                .map(|item| item.id.as_str())
                .collect::<Vec<_>>(),
            ["010", "030", "020"]
        );
    }

    #[test]
    fn open_p3_rows_are_selectable_ranked_last() {
        // P3 open rows must be reachable via the CLI-driven loop (not permanently filtered
        // out), but always after P0-P2.
        let temp_dir = TempDir::new().expect("temp dir");
        fs::create_dir_all(temp_dir.path().join("docs/source-port")).expect("docs dir");
        fs::write(
            temp_dir.path().join("docs/source-port/backlog.md"),
            "\
| # | Status | Pri | System | Summary | Verify |
|-----|--------|-----|--------|---------|--------|
| 010 | [ ] | P3 | foo | polish item | no |
| 011 | [ ] | P1 | foo | core risk | no |
",
        )
        .expect("backlog");

        let items =
            load_top_open_items(temp_dir.path(), 5, BacklogFocus::Playability).expect("items");
        assert_eq!(items.len(), 2);
        assert_eq!(items[0].id, "011"); // P1 first
        assert_eq!(items[1].id, "010"); // P3 last but present
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
    fn ready_items_require_open_status_supported_priority_and_root_cause_marker() {
        let temp_dir = TempDir::new().expect("temp dir");
        fs::create_dir_all(temp_dir.path().join("docs/source-port")).expect("docs dir");
        fs::write(
            temp_dir.path().join("docs/source-port/backlog.md"),
            "\
| # | Status | Pri | System | Summary | Verify |
|-----|--------|-----|--------|---------|--------|
| 010 | [x] | P0 | enemy | closed root cause identified | verify |
| 011 | [ ] | PX | enemy | invalid priority root cause identified | verify |
| 012 | [ ] | P2 | enemy | no cause marker | verify |
| 013 | [ ] | P2 | enemy |
| 020 | [~] | P3 | enemy | p3 ready | root-caused in verify |
| 021 | [ ] | P2 | enemy | p2 ready root cause identified | verify |
| 022 | [ ] | P1 | enemy | p1 ready | root-caused |
| 023 | [ ] | P0 | enemy | p0 ready root cause identified | verify |
",
        )
        .expect("backlog");

        let items = load_ready_to_implement_items(temp_dir.path(), 10).expect("items");
        assert_eq!(
            items
                .iter()
                .map(|item| item.id.as_str())
                .collect::<Vec<_>>(),
            ["023", "022", "021", "020"]
        );
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
| 147 | [x] | P1 | render | Closed validation row | done |
| 148 | [ ] | P1 | player | Agent-actionable row | fix |
",
        )
        .expect("backlog");
        fs::write(
            temp_dir
                .path()
                .join("docs/source-port/backlog-awaiting-human.md"),
            "\
| # | Status | Pri | System | Target | Check | Summary | Verify |
|-----|--------|-----|--------|--------|-------|---------|--------|
| 135 | [?] | P1 | render | 0x00 | Confirm the level background looks correct. | Level 1 background needs validation | review |
| 146 | [b] | P1 | level | 0x01 | Reproduce the death route and capture the frame. | Level 2 death needs repro | review |
",
        )
        .expect("awaiting");

        let items = load_human_validation_blockers(temp_dir.path());
        assert_eq!(items.len(), 2);
        assert_eq!(items[0].id, "135");
        assert_eq!(items[0].status, "[?]");
        assert_eq!(items[0].target, "0x00");
        assert_eq!(
            items[0].check,
            "Confirm the level background looks correct."
        );
        assert_eq!(items[1].id, "146");
        assert_eq!(items[1].status, "[b]");
    }

    #[test]
    fn human_blocker_parser_skips_malformed_rows_and_ranks_p0_to_p2() {
        let temp_dir = TempDir::new().expect("temp dir");
        fs::create_dir_all(temp_dir.path().join("docs/source-port")).expect("docs dir");
        fs::write(
            temp_dir.path().join("docs/source-port/backlog.md"),
            "\
| # | Status | Pri | System | Summary | Verify |
|-----|--------|-----|--------|---------|--------|
| 100 | [ ] | P1 | player | open row | verify |
",
        )
        .expect("backlog");
        fs::write(
            temp_dir
                .path()
                .join("docs/source-port/backlog-awaiting-human.md"),
            "\
not a table row
| # | Status | Pri | System | Target | Check | Summary | Verify |
|-----|--------|-----|--------|--------|-------|---------|--------|
| 010 | [?] | P1 | render |
| 011 | [?] | P3 | render | 0x01 | Confirm it. | unsupported blocker priority | review |
| 010 | [b] | P2 | level | 0x01 | Confirm the p2 blocker. | p2 blocker | review |
| 020 | [?] | P1 | render | 0x00 | Confirm the p1 blocker. | p1 blocker | review |
| 030 | [b] | P0 | audio | audio-device | Confirm the p0 blocker. | p0 blocker | review |
",
        )
        .expect("awaiting");

        let items = load_human_validation_blockers(temp_dir.path());
        assert_eq!(
            items
                .iter()
                .map(|item| item.id.as_str())
                .collect::<Vec<_>>(),
            ["030", "020", "010"]
        );
    }

    #[test]
    fn selection_spans_queue_and_awaiting_human_lanes() {
        // The loop selects from the open queue (backlog.md) while the human-validation
        // lane ([?]/[b]) lives in backlog-awaiting-human.md. project-status must read both
        // so sharding by lane does not silently hide blockers from completion-audit.
        let temp_dir = TempDir::new().expect("temp dir");
        fs::create_dir_all(temp_dir.path().join("docs/source-port")).expect("docs dir");
        fs::write(
            temp_dir.path().join("docs/source-port/backlog.md"),
            "\
| # | Status | Pri | System | Summary | Verify |
|-----|--------|-----|--------|---------|--------|
| 200 | [ ] | P1 | player | open queue row | fix |
",
        )
        .expect("backlog");
        fs::write(
            temp_dir
                .path()
                .join("docs/source-port/backlog-awaiting-human.md"),
            "\
| # | Status | Pri | System | Target | Check | Summary | Verify |
|-----|--------|-----|--------|--------|-------|---------|--------|
| 201 | [?] | P0 | render | 0x00 | Confirm the sharded blocker. | sharded blocker | review |
| 202 | [b] | P1 | level | 0x01 | Reproduce the sharded blocker. | sharded repro blocker | review |
",
        )
        .expect("awaiting-human");

        let blockers = load_human_validation_blockers(temp_dir.path());
        assert_eq!(blockers.len(), 2);
        assert_eq!(blockers[0].id, "201"); // P0 sorts ahead of P1
        assert_eq!(blockers[1].id, "202");

        let open =
            load_top_open_items(temp_dir.path(), 5, BacklogFocus::Playability).expect("open");
        assert_eq!(open.len(), 1);
        assert_eq!(open[0].id, "200");
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
",
        )
        .expect("backlog");
        fs::write(
            temp_dir
                .path()
                .join("docs/source-port/backlog-awaiting-human.md"),
            "\
| # | Status | Pri | System | Target | Check | Summary | Verify |
|-----|--------|-----|--------|--------|-------|---------|--------|
| 135 | [?] | P1 | render | 0x00 | Confirm the visual blocker. | human visual blocker | review |
",
        )
        .expect("awaiting");
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
            !report
                .warnings
                .iter()
                .any(|warning| warning.contains("1p-accurate")),
            "passing supporting gate should not warn: {:?}",
            report.warnings
        );
        assert!(
            report
                .warnings
                .iter()
                .any(|warning| warning.contains("human validation/repro blocker"))
        );
    }

    #[test]
    fn render_project_status_includes_all_sections_and_review_hint() {
        let report = ProjectStatusReport {
            headline_gate: "1p-playable".to_owned(),
            headline_passed: true,
            secondary_gates: vec![GateSignal {
                name: "1p-accurate".to_owned(),
                passed: false,
                detail: "state_sweep failed".to_owned(),
                path: "/tmp/report.json".to_owned(),
                failing_step: Some("state_sweep: failed".to_owned()),
            }],
            ready_to_implement_items: vec![BacklogItem {
                id: "010".to_owned(),
                priority: "P0".to_owned(),
                system: "enemy".to_owned(),
                summary: "ready row".to_owned(),
            }],
            top_open_playability_items: vec![BacklogItem {
                id: "011".to_owned(),
                priority: "P1".to_owned(),
                system: "player".to_owned(),
                summary: "play row".to_owned(),
            }],
            top_open_visual_items: vec![BacklogItem {
                id: "012".to_owned(),
                priority: "P1".to_owned(),
                system: "render".to_owned(),
                summary: "visual row".to_owned(),
            }],
            human_validation_blockers: vec![HumanValidationBlocker {
                id: "013".to_owned(),
                status: "[?]".to_owned(),
                priority: "P0".to_owned(),
                system: "audio".to_owned(),
                target: "audio-device".to_owned(),
                check: "Confirm the audio row.".to_owned(),
                summary: "human row".to_owned(),
            }],
            warnings: vec!["warning row".to_owned()],
        };

        let text = render_project_status(&report);

        assert!(text.contains("Project Status"));
        assert!(text.contains("headline: 1p-playable (passed)"));
        assert!(text.contains("1p-accurate: missing/failing"));
        assert!(text.contains("#010 P0 enemy: ready row"));
        assert!(text.contains("#011 P1 player: play row"));
        assert!(text.contains("#012 P1 render: visual row"));
        assert!(text.contains("#013 [?] P0 audio [audio-device]: Confirm the audio row."));
        assert!(text.contains("how to use: run `mise run play-native`"));
        assert!(text.contains("review: human playtest required to close [?]/[b] rows"));
        assert!(text.contains("warning row"));
        assert_eq!(print_project_status(&report), text);
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
",
        )
        .expect("backlog");
        fs::write(
            temp_dir
                .path()
                .join("docs/source-port/backlog-awaiting-human.md"),
            "\
| # | Status | Pri | System | Target | Check | Summary | Verify |
|-----|--------|-----|--------|--------|-------|---------|--------|
| 135 | [?] | P1 | render | 0x00 | Confirm the visual blocker. | human visual blocker | review |
",
        )
        .expect("awaiting");
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

        run_project_status(temp_dir.path(), ProjectStatusFormat::Text, false)
            .expect("human blockers should not fail unless explicitly requested");
    }

    #[test]
    fn project_status_warns_for_unknown_human_targets() {
        let temp_dir = TempDir::new().expect("temp dir");
        fs::create_dir_all(temp_dir.path().join("docs/source-port")).expect("docs dir");
        fs::write(
            temp_dir.path().join("docs/source-port/backlog.md"),
            "\
| # | Status | Pri | System | Summary | Verify |
|-----|--------|-----|--------|---------|--------|
",
        )
        .expect("backlog");
        fs::write(
            temp_dir
                .path()
                .join("docs/source-port/backlog-awaiting-human.md"),
            "\
| # | Status | Pri | System | Target | Check | Summary | Verify |
|-----|--------|-----|--------|--------|-------|---------|--------|
| 135 | [?] | P1 | render | unknown | Confirm the reported issue is fixed. | human visual blocker | review |
",
        )
        .expect("awaiting");

        let report = build_project_status_report(temp_dir.path()).expect("report");
        assert!(
            report
                .warnings
                .iter()
                .any(|warning| warning.contains("unknown targets"))
        );
    }
}
