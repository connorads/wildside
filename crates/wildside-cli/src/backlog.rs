//! Shared accessors for the source-port playtest backlog.
//!
//! The backlog is sharded by reader into lane files:
//! - `backlog.md` holds the NOW handoff and the live queue (open `[ ]`/`[~]` rows).
//! - `backlog-awaiting-human.md` holds `[?]`/`[b]` rows that only a human can close.
//! - resolved rows (`[x]`/`[-]`) live in `backlog-archive.md` as history, keyed by `#NNN`.
//!
//! Tools that select or audit work (`project-status`, `asm-coverage`) read the live
//! lanes; resolved history is out of their scope. Reading is layout-tolerant so the file
//! layout stays decoupled from selection: a row is found wherever it lives, and a missing
//! awaiting-human lane is fine (it may not exist yet, or every such row may currently sit
//! in `backlog.md`).

use anyhow::{Context, Result, bail};
use std::collections::BTreeSet;
use std::fs;
use std::path::Path;

/// Primary lane: the NOW handoff and the open queue. Required.
pub(crate) const BACKLOG_PATH: &str = "docs/source-port/backlog.md";
/// Human-validation lane: `[?]`/`[b]` rows. Optional.
pub(crate) const AWAITING_HUMAN_PATH: &str = "docs/source-port/backlog-awaiting-human.md";

/// Cap on the NOW handoff plus its preamble (`## NOW` to the table header). Catches the
/// changelog accretion that bloated the block to ~1350 lines.
const MAX_NOW_LINES: usize = 80;
/// Cap on an open-queue Verify cell. Evidence over this belongs in the archive.
const MAX_OPEN_VERIFY_CHARS: usize = 400;

/// Read every live backlog lane and return their concatenated markdown.
///
/// The primary lane is required (a missing `backlog.md` is an error, as before); the
/// awaiting-human lane is appended when present. Row parsers scan the result without
/// caring which lane a row physically lives in.
pub(crate) fn read_live_lanes(repo_root: &Path) -> Result<String> {
    let primary = repo_root.join(BACKLOG_PATH);
    let mut combined = fs::read_to_string(&primary)
        .with_context(|| format!("failed to read backlog {}", primary.display()))?;
    let awaiting = repo_root.join(AWAITING_HUMAN_PATH);
    if let Ok(extra) = fs::read_to_string(&awaiting) {
        combined.push('\n');
        combined.push_str(&extra);
    }
    Ok(combined)
}

/// One parsed table row, with just the fields the lint inspects.
struct LintRow {
    id: String,
    status: String,
    verify: String,
}

/// Parse `| id | status | pri | system | summary | verify |` rows. Mirrors the
/// filter-empty positional split used by the selection parsers, and ignores any row
/// whose first cell is not a numeric id (so heading/separator/template lines are skipped).
fn parse_rows(contents: &str) -> Vec<LintRow> {
    let mut rows = Vec::new();
    for line in contents.lines() {
        if !line.starts_with('|') || line.contains("| Status |") || line.contains("|-----") {
            continue;
        }
        let cols: Vec<&str> = line
            .split('|')
            .map(str::trim)
            .filter(|c| !c.is_empty())
            .collect();
        if cols.len() < 2 || !cols[0].chars().all(|c| c.is_ascii_digit()) {
            continue;
        }
        rows.push(LintRow {
            id: cols[0].to_owned(),
            status: cols[1].to_owned(),
            verify: cols.get(5).copied().unwrap_or("").to_owned(),
        });
    }
    rows
}

/// Length of the `## NOW` block plus preamble, measured to the table header.
fn now_block_len(backlog: &str) -> Option<usize> {
    let lines: Vec<&str> = backlog.lines().collect();
    let start = lines.iter().position(|l| l.trim_start().starts_with("## NOW"))?;
    let table = lines.iter().position(|l| l.starts_with("| # | Status"))?;
    Some(table.saturating_sub(start))
}

/// Enforce the backlog's pit-of-success shape. Returns an error listing every problem so
/// drift fails `verify-slice-fast` rather than rotting silently.
pub(crate) fn run_backlog_lint(repo_root: &Path) -> Result<()> {
    let backlog_path = repo_root.join(BACKLOG_PATH);
    let backlog = fs::read_to_string(&backlog_path)
        .with_context(|| format!("failed to read {}", backlog_path.display()))?;
    let awaiting = fs::read_to_string(repo_root.join(AWAITING_HUMAN_PATH)).unwrap_or_default();

    let mut problems: Vec<String> = Vec::new();

    match now_block_len(&backlog) {
        Some(n) if n > MAX_NOW_LINES => problems.push(format!(
            "NOW block + preamble is {n} lines (max {MAX_NOW_LINES}); move session history to session-log.md"
        )),
        None => problems.push("backlog.md has no `## NOW` block before the table".to_owned()),
        Some(_) => {}
    }
    if backlog.contains("- **Last session**") {
        problems.push(
            "backlog.md contains `- **Last session**`; session history belongs in session-log.md"
                .to_owned(),
        );
    }

    let backlog_rows = parse_rows(&backlog);
    let awaiting_rows = parse_rows(&awaiting);

    for row in &backlog_rows {
        let len = row.verify.chars().count();
        if len > MAX_OPEN_VERIFY_CHARS {
            problems.push(format!(
                "#{} Verify cell is {len} chars (max {MAX_OPEN_VERIFY_CHARS}); move evidence to the archive",
                row.id
            ));
        }
        if !matches!(row.status.as_str(), "[ ]" | "[~]") {
            problems.push(format!(
                "#{} has status {} in backlog.md (open queue); move it to its lane",
                row.id, row.status
            ));
        }
    }
    for row in &awaiting_rows {
        if !matches!(row.status.as_str(), "[?]" | "[b]") {
            problems.push(format!(
                "#{} has status {} in backlog-awaiting-human.md; only [?]/[b] belong there",
                row.id, row.status
            ));
        }
    }
    let backlog_ids: BTreeSet<&str> = backlog_rows.iter().map(|r| r.id.as_str()).collect();
    for row in &awaiting_rows {
        if backlog_ids.contains(row.id.as_str()) {
            problems.push(format!(
                "#{} appears in both backlog.md and backlog-awaiting-human.md",
                row.id
            ));
        }
    }

    if problems.is_empty() {
        println!(
            "backlog-lint: ok ({} open, {} awaiting-human)",
            backlog_rows.len(),
            awaiting_rows.len()
        );
        Ok(())
    } else {
        for problem in &problems {
            eprintln!("backlog-lint: {problem}");
        }
        bail!("{} backlog hygiene problem(s)", problems.len());
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn write(dir: &TempDir, rel: &str, body: &str) {
        let path = dir.path().join(rel);
        fs::create_dir_all(path.parent().unwrap()).expect("parent");
        fs::write(path, body).expect("write");
    }

    #[test]
    fn reads_primary_lane_only_when_awaiting_human_absent() {
        let dir = TempDir::new().expect("temp");
        write(&dir, BACKLOG_PATH, "queue row\n");
        let combined = read_live_lanes(dir.path()).expect("read");
        assert!(combined.contains("queue row"));
    }

    #[test]
    fn concatenates_awaiting_human_lane_when_present() {
        let dir = TempDir::new().expect("temp");
        write(&dir, BACKLOG_PATH, "queue row\n");
        write(&dir, AWAITING_HUMAN_PATH, "awaiting row\n");
        let combined = read_live_lanes(dir.path()).expect("read");
        assert!(combined.contains("queue row"));
        assert!(combined.contains("awaiting row"));
    }

    #[test]
    fn missing_primary_lane_is_an_error() {
        let dir = TempDir::new().expect("temp");
        let error = read_live_lanes(dir.path()).expect_err("missing primary errors");
        assert!(error.to_string().contains("failed to read backlog"));
    }

    const CLEAN_BACKLOG: &str = "\
## NOW

- Focus: working #200

| # | Status | Pri | System | Summary | Verify |
|-----|--------|-----|--------|---------|--------|
| 200 | [ ] | P1 | player | a bug | mise run verify-slice-fast |
| 201 | [~] | P2 | render | another | verify |
";

    #[test]
    fn lint_passes_on_clean_lanes() {
        let dir = TempDir::new().expect("temp");
        write(&dir, BACKLOG_PATH, CLEAN_BACKLOG);
        write(
            &dir,
            AWAITING_HUMAN_PATH,
            "| # | Status | Pri | System | Summary | Verify |\n\
             |-----|--------|-----|--------|---------|--------|\n\
             | 300 | [?] | P1 | audio | proposed | review |\n",
        );
        run_backlog_lint(dir.path()).expect("clean lanes lint clean");
    }

    #[test]
    fn lint_flags_changelog_accretion_in_backlog() {
        let dir = TempDir::new().expect("temp");
        let bloated =
            CLEAN_BACKLOG.replace("- Focus: working #200", "- **Last session** (2026-06-20): ...");
        write(&dir, BACKLOG_PATH, &bloated);
        let error = run_backlog_lint(dir.path()).expect_err("changelog flagged");
        assert!(error.to_string().contains("hygiene problem"));
    }

    #[test]
    fn lint_flags_oversized_now_block() {
        let dir = TempDir::new().expect("temp");
        let filler = "- note\n".repeat(MAX_NOW_LINES + 5);
        let bloated = CLEAN_BACKLOG.replace("- Focus: working #200", &filler);
        write(&dir, BACKLOG_PATH, &bloated);
        run_backlog_lint(dir.path()).expect_err("oversized NOW flagged");
    }

    #[test]
    fn lint_flags_oversized_open_verify_cell() {
        let dir = TempDir::new().expect("temp");
        let big = "x".repeat(MAX_OPEN_VERIFY_CHARS + 1);
        let bloated = CLEAN_BACKLOG.replace("mise run verify-slice-fast", &big);
        write(&dir, BACKLOG_PATH, &bloated);
        run_backlog_lint(dir.path()).expect_err("oversized verify flagged");
    }

    #[test]
    fn lint_flags_status_in_wrong_lane() {
        let dir = TempDir::new().expect("temp");
        // A [?] row left in the open queue must be flagged.
        let misplaced = CLEAN_BACKLOG.replace("| 200 | [ ] |", "| 200 | [?] |");
        write(&dir, BACKLOG_PATH, &misplaced);
        run_backlog_lint(dir.path()).expect_err("misplaced status flagged");
    }

    #[test]
    fn lint_flags_id_in_two_lanes() {
        let dir = TempDir::new().expect("temp");
        write(&dir, BACKLOG_PATH, CLEAN_BACKLOG);
        write(
            &dir,
            AWAITING_HUMAN_PATH,
            "| # | Status | Pri | System | Summary | Verify |\n\
             |-----|--------|-----|--------|---------|--------|\n\
             | 200 | [?] | P1 | player | dup | review |\n",
        );
        run_backlog_lint(dir.path()).expect_err("duplicate id across lanes flagged");
    }
}
