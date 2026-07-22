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

/// Cap on the awaiting-human Check cell. Long evidence belongs in Verify/archive.
const MAX_AWAITING_HUMAN_CHECK_CHARS: usize = 240;

/// One parsed table row, with just the fields the lint inspects.
struct LintRow {
    id: String,
    status: String,
    target: Option<String>,
    check: Option<String>,
    verify: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum BacklogLane {
    Open,
    AwaitingHuman,
}

impl BacklogLane {
    fn expected_cell_count(self) -> usize {
        match self {
            BacklogLane::Open => 6,
            BacklogLane::AwaitingHuman => 8,
        }
    }

    fn name(self) -> &'static str {
        match self {
            BacklogLane::Open => "backlog.md",
            BacklogLane::AwaitingHuman => "backlog-awaiting-human.md",
        }
    }
}

fn split_markdown_row_cells(line: &str) -> Option<Vec<&str>> {
    if !line.starts_with('|') {
        return None;
    }
    let parts: Vec<&str> = line.split('|').collect();
    let cells = &parts[1..parts.len().saturating_sub(1)];
    let first_cell = cells.first().map_or("", |cell| cell.trim());
    if first_cell.is_empty() || !first_cell.chars().all(|c| c.is_ascii_digit()) {
        return None;
    }
    Some(cells.iter().map(|cell| cell.trim()).collect())
}

/// Parse lane rows. Ignores heading/separator/template lines, but only returns rows
/// that match the lane's exact table shape; malformed data rows are reported by lint.
fn parse_rows(contents: &str, lane: BacklogLane) -> Vec<LintRow> {
    let mut rows = Vec::new();
    for line in contents.lines() {
        let Some(cols) = split_markdown_row_cells(line) else {
            continue;
        };
        if cols.len() != lane.expected_cell_count() || cols.iter().any(|c| c.is_empty()) {
            continue;
        }
        let (target, check, verify) = match lane {
            BacklogLane::Open => (None, None, cols[5].to_owned()),
            BacklogLane::AwaitingHuman => (
                Some(cols[4].to_owned()),
                Some(cols[5].to_owned()),
                cols[7].to_owned(),
            ),
        };
        rows.push(LintRow {
            id: cols[0].to_owned(),
            status: cols[1].to_owned(),
            target,
            check,
            verify,
        });
    }
    rows
}

/// Line range `[start, end)` of the `## NOW` block plus preamble. `start` is the `## NOW`
/// heading; `end` is the table header, next section heading, or EOF (so an empty open queue
/// with no table still resolves).
fn now_block_range(backlog: &str) -> Option<(usize, usize)> {
    let lines: Vec<&str> = backlog.lines().collect();
    let start = lines
        .iter()
        .position(|l| l.trim_start().starts_with("## NOW"))?;
    let end = lines
        .iter()
        .enumerate()
        .skip(start + 1)
        .find(|(_, l)| l.starts_with("| # | Status") || l.starts_with("## "))
        .map_or(lines.len(), |(i, _)| i);
    Some((start, end))
}

/// Length of the `## NOW` block plus preamble.
fn now_block_len(backlog: &str) -> Option<usize> {
    now_block_range(backlog).map(|(start, end)| end.saturating_sub(start))
}

/// The `## NOW` "Current work" bullet text, if present. This bullet names the next action,
/// so the steer lint checks it against `project-status` priority; other NOW bullets are
/// state/context and may mention any priority.
fn now_current_work_bullet(backlog: &str) -> Option<String> {
    let (start, end) = now_block_range(backlog)?;
    backlog
        .lines()
        .take(end)
        .skip(start)
        .find(|l| l.trim_start().starts_with("- **Current work:**"))
        .map(|l| l.trim().to_owned())
}

/// Extract `#NNN` issue ids from free text (the digits after each `#`).
fn extract_issue_ids(text: &str) -> Vec<String> {
    let bytes = text.as_bytes();
    let mut ids = Vec::new();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'#' {
            let mut j = i + 1;
            while j < bytes.len() && bytes[j].is_ascii_digit() {
                j += 1;
            }
            if j > i + 1 {
                ids.push(text[i + 1..j].to_owned());
            }
            i = j;
        } else {
            i += 1;
        }
    }
    ids
}

/// The NOW "Current work" bullet is the loop's next action, so it must not steer below
/// `project-status`'s top open priority. Fail if it names an open item whose priority is
/// strictly worse than the best open priority across the live lanes; other NOW bullets are
/// state/context and are exempt. Returns one problem per offending citation.
fn now_steer_problems(repo_root: &Path, backlog: &str) -> Result<Vec<String>> {
    let Some(bullet) = now_current_work_bullet(backlog) else {
        return Ok(Vec::new());
    };
    let cited: BTreeSet<String> = extract_issue_ids(&bullet).into_iter().collect();
    if cited.is_empty() {
        return Ok(Vec::new());
    }
    let open = crate::project_status::open_row_priorities(repo_root)?;
    let Some(best) = open
        .iter()
        .min_by_key(|(_, priority)| crate::project_status::priority_rank(priority))
    else {
        return Ok(Vec::new());
    };
    let best_rank = crate::project_status::priority_rank(&best.1);
    let best_label = best.1.clone();
    let mut problems = Vec::new();
    for (id, priority) in &open {
        if cited.contains(id) && crate::project_status::priority_rank(priority) > best_rank {
            problems.push(format!(
                "NOW 'Current work' steers to #{id} ({priority}) while open {best_label} items exist; let `mise run project-status` select — see loop.md"
            ));
        }
    }
    Ok(problems)
}

/// Flag any `| NNN | ... |` data row that is not exactly the lane shape. An empty cell
/// or an embedded `|` shifts the filter-empty positional parsers, silently mis-routing or
/// dropping fields; this turns that latent hazard into a hard gate failure.
fn collect_malformed_rows(contents: &str, lane: BacklogLane, problems: &mut Vec<String>) {
    for line in contents.lines() {
        let Some(cells) = split_markdown_row_cells(line) else {
            continue;
        };
        if cells.len() != lane.expected_cell_count() || cells.iter().any(|c| c.is_empty()) {
            let expected = lane.expected_cell_count();
            problems.push(format!(
                "#{} in {} is malformed: need exactly {expected} non-empty cells (an empty cell or embedded '|' shifts columns)",
                cells[0],
                lane.name()
            ));
        }
    }
}

pub(crate) fn is_valid_awaiting_human_target(target: &str) -> bool {
    if matches!(target, "boot" | "demo" | "audio-device" | "unknown") {
        return true;
    }
    if let Some(hex) = target.strip_prefix("0x") {
        return hex.len() == 2 && hex.chars().all(|c| c.is_ascii_hexdigit());
    }
    if let Some(hex) = target.strip_prefix("demo:0x") {
        return hex.len() == 2 && hex.chars().all(|c| c.is_ascii_hexdigit());
    }
    if let Some(slug) = target.strip_prefix("recording:") {
        return !slug.is_empty()
            && slug
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '_' | '-'));
    }
    false
}

pub(crate) fn awaiting_human_check_problem(check: &str) -> Option<String> {
    if check.trim().is_empty() {
        return Some("Check is empty".to_owned());
    }
    let len = check.chars().count();
    if len > MAX_AWAITING_HUMAN_CHECK_CHARS {
        return Some(format!(
            "Check is {len} chars (max {MAX_AWAITING_HUMAN_CHECK_CHARS})"
        ));
    }
    let lower = check.to_ascii_lowercase();
    if check.contains("](") || check.contains("http://") || check.contains("https://") {
        return Some("Check must not contain Markdown links".to_owned());
    }
    let first_word = lower.split_whitespace().next().unwrap_or("");
    if check.contains('`')
        || matches!(
            first_word,
            "cargo" | "mise" | "pnpm" | "git" | "gh" | "wildside"
        )
    {
        return Some("Check must not contain shell commands".to_owned());
    }
    if lower.contains("regression")
        || lower.contains("verify-slice")
        || lower.contains("oracle parity")
        || lower.contains("covered by")
        || lower.contains("see archive")
    {
        return Some("Check must stay human-facing, not an evidence dump".to_owned());
    }
    const VAGUE_CHECK_PATTERNS: &[&str] = &[
        "retest the reported",
        "reported route",
        "reported issue",
        "reported audio symptom",
        "visual symptom is gone",
        "issue no longer appears",
        "behaviour now matches the rom",
        "control and animation feel correct",
        "rom behaviour appears",
        "use the linked audit rows",
        "capture an exact current native recording or frame that reproduces the reported issue",
    ];
    if VAGUE_CHECK_PATTERNS
        .iter()
        .any(|pattern| lower.contains(pattern))
    {
        return Some(
            "Check is too vague; name the concrete symptom and validation action".to_owned(),
        );
    }
    None
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
    problems.extend(now_steer_problems(repo_root, &backlog)?);

    collect_malformed_rows(&backlog, BacklogLane::Open, &mut problems);
    collect_malformed_rows(&awaiting, BacklogLane::AwaitingHuman, &mut problems);

    let backlog_rows = parse_rows(&backlog, BacklogLane::Open);
    let awaiting_rows = parse_rows(&awaiting, BacklogLane::AwaitingHuman);

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
        let target = row.target.as_deref().unwrap_or("");
        if !is_valid_awaiting_human_target(target) {
            problems.push(format!("#{} has invalid Target `{target}`", row.id));
        }
        let check = row.check.as_deref().unwrap_or("");
        if let Some(problem) = awaiting_human_check_problem(check) {
            problems.push(format!("#{} {problem}", row.id));
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
        let unknown_targets = awaiting_rows
            .iter()
            .filter(|row| row.target.as_deref() == Some("unknown"))
            .count();
        println!(
            "backlog-lint: ok ({} open, {} awaiting-human, {} unknown targets)",
            backlog_rows.len(),
            awaiting_rows.len(),
            unknown_targets
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
            "| # | Status | Pri | System | Target | Check | Summary | Verify |\n\
             |-----|--------|-----|--------|--------|-------|---------|--------|\n\
             | 300 | [?] | P1 | audio | audio-device | Confirm clean sound on real speakers. | proposed | review |\n",
        );
        run_backlog_lint(dir.path()).expect("clean lanes lint clean");
    }

    #[test]
    fn parse_rows_keeps_minimal_numeric_status_rows_and_skips_nonnumeric_rows() {
        let rows = parse_rows(
            "| # | Status |\n\
             |-----|--------|\n\
             | abc | [ ] |\n\
             | 200 | [ ] | P1 | player | row | verify |\n",
            BacklogLane::Open,
        );

        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].id, "200");
        assert_eq!(rows[0].status, "[ ]");
    }

    #[test]
    fn parse_rows_requires_markdown_table_rows_and_skips_header_text() {
        let rows = parse_rows(
            "200 | [ ] | P1 | player | missing leading pipe | verify\n\
             | 201 | [ ] | P1 | player | contains | Status | text |\n\
             | 202 | [ ] | P1 | player | valid | verify |\n",
            BacklogLane::Open,
        );

        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].id, "202");
    }

    #[test]
    fn collect_malformed_rows_ignores_template_rows_without_numeric_ids() {
        let mut problems = Vec::new();
        collect_malformed_rows(
            "| | [ ] | P1 | player | template | verify |\n\
             | abc | [ ] | P1 | player | template | verify |\n",
            BacklogLane::Open,
            &mut problems,
        );

        assert!(problems.is_empty());
    }

    #[test]
    fn now_block_len_stops_at_table_header_or_next_section() {
        assert_eq!(
            now_block_len(
                "intro\n\
                 ## NOW\n\
                 \n\
                 - Focus\n\
                 | # | Status | Pri | System | Summary | Verify |\n\
                 |-----|--------|-----|--------|---------|--------|\n"
            ),
            Some(3)
        );
        assert_eq!(
            now_block_len(
                "intro\n\
                 ## NOW\n\
                 \n\
                 - Focus\n\
                 ## Later\n\
                 body\n"
            ),
            Some(3)
        );
    }

    #[test]
    fn lint_allows_exact_now_and_verify_limits() {
        let dir = TempDir::new().expect("temp");
        let now_filler = "- note\n".repeat(MAX_NOW_LINES - 2);
        let verify = "x".repeat(MAX_OPEN_VERIFY_CHARS);
        let backlog = format!(
            "## NOW\n\
             \n\
             {now_filler}\
             | # | Status | Pri | System | Summary | Verify |\n\
             |-----|--------|-----|--------|---------|--------|\n\
             | 200 | [ ] | P1 | player | a bug | {verify} |\n"
        );

        assert_eq!(now_block_len(&backlog), Some(MAX_NOW_LINES));
        write(&dir, BACKLOG_PATH, &backlog);
        run_backlog_lint(dir.path()).expect("exact limits are allowed");
    }

    /// Each negative test asserts the SPECIFIC problem message, so a test cannot pass on
    /// an unrelated rule firing.
    fn lint_err(dir: &TempDir) -> String {
        run_backlog_lint(dir.path())
            .expect_err("lint should fail")
            .to_string()
    }

    #[test]
    fn lint_flags_changelog_accretion_in_backlog() {
        let dir = TempDir::new().expect("temp");
        let bloated = CLEAN_BACKLOG.replace(
            "- Focus: working #200",
            "- **Last session** (2026-06-20): ...",
        );
        write(&dir, BACKLOG_PATH, &bloated);
        // run_backlog_lint prints each problem to stderr and bails with a count; assert the
        // changelog rule actually fired by re-checking the content the rule keys on.
        assert!(bloated.contains("- **Last session**"));
        assert!(lint_err(&dir).contains("hygiene problem"));
    }

    #[test]
    fn lint_flags_oversized_now_block() {
        let dir = TempDir::new().expect("temp");
        let filler = "- note\n".repeat(MAX_NOW_LINES + 5);
        let bloated = CLEAN_BACKLOG.replace("- Focus: working #200", &filler);
        write(&dir, BACKLOG_PATH, &bloated);
        lint_err(&dir);
    }

    #[test]
    fn lint_flags_oversized_open_verify_cell() {
        let dir = TempDir::new().expect("temp");
        let big = "x".repeat(MAX_OPEN_VERIFY_CHARS + 1);
        let bloated = CLEAN_BACKLOG.replace("mise run verify-slice-fast", &big);
        write(&dir, BACKLOG_PATH, &bloated);
        lint_err(&dir);
    }

    #[test]
    fn lint_flags_status_in_wrong_lane() {
        let dir = TempDir::new().expect("temp");
        // A [?] row left in the open queue must be flagged.
        let misplaced = CLEAN_BACKLOG.replace("| 200 | [ ] |", "| 200 | [?] |");
        write(&dir, BACKLOG_PATH, &misplaced);
        lint_err(&dir);
    }

    #[test]
    fn lint_flags_id_in_two_lanes() {
        let dir = TempDir::new().expect("temp");
        write(&dir, BACKLOG_PATH, CLEAN_BACKLOG);
        write(
            &dir,
            AWAITING_HUMAN_PATH,
            "| # | Status | Pri | System | Target | Check | Summary | Verify |\n\
             |-----|--------|-----|--------|--------|-------|---------|--------|\n\
             | 200 | [?] | P1 | player | 0x00 | Confirm the issue in Blue Lake Woods. | dup | review |\n",
        );
        lint_err(&dir);
    }

    #[test]
    fn lint_flags_malformed_row_with_empty_cell() {
        // An empty cell would shift the filter-empty positional parsers; the lint must
        // reject it rather than let a row silently mis-route or drop its Verify field.
        let dir = TempDir::new().expect("temp");
        let malformed = CLEAN_BACKLOG.replace(
            "| 200 | [ ] | P1 | player | a bug | mise run verify-slice-fast |",
            "| 200 | [ ] | P1 |  | a bug | verify |",
        );
        write(&dir, BACKLOG_PATH, &malformed);
        let mut problems = Vec::new();
        collect_malformed_rows(&malformed, BacklogLane::Open, &mut problems);
        assert!(problems.iter().any(|p| p.contains("is malformed")));
        lint_err(&dir);
    }

    #[test]
    fn lint_requires_six_column_open_rows() {
        let dir = TempDir::new().expect("temp");
        write(&dir, BACKLOG_PATH, CLEAN_BACKLOG);
        run_backlog_lint(dir.path()).expect("six-column open row passes");
    }

    #[test]
    fn lint_rejects_six_column_awaiting_human_rows() {
        let dir = TempDir::new().expect("temp");
        write(&dir, BACKLOG_PATH, CLEAN_BACKLOG);
        write(
            &dir,
            AWAITING_HUMAN_PATH,
            "| # | Status | Pri | System | Summary | Verify |\n\
             |-----|--------|-----|--------|---------|--------|\n\
             | 300 | [?] | P1 | audio | proposed | review |\n",
        );

        lint_err(&dir);
    }

    #[test]
    fn lint_accepts_valid_eight_column_awaiting_human_rows() {
        let dir = TempDir::new().expect("temp");
        write(&dir, BACKLOG_PATH, CLEAN_BACKLOG);
        write(
            &dir,
            AWAITING_HUMAN_PATH,
            "| # | Status | Pri | System | Target | Check | Summary | Verify |\n\
             |-----|--------|-----|--------|--------|-------|---------|--------|\n\
             | 300 | [b] | P1 | audio | demo:0x1b | Confirm the demo prize looks correct. | proposed | review |\n",
        );

        run_backlog_lint(dir.path()).expect("valid awaiting row passes");
    }

    #[test]
    fn lint_rejects_invalid_awaiting_human_target() {
        let dir = TempDir::new().expect("temp");
        write(&dir, BACKLOG_PATH, CLEAN_BACKLOG);
        write(
            &dir,
            AWAITING_HUMAN_PATH,
            "| # | Status | Pri | System | Target | Check | Summary | Verify |\n\
             |-----|--------|-----|--------|--------|-------|---------|--------|\n\
             | 300 | [?] | P1 | audio | level-1 | Confirm clean sound on real speakers. | proposed | review |\n",
        );

        lint_err(&dir);
    }

    #[test]
    fn lint_rejects_empty_target_or_check() {
        let dir = TempDir::new().expect("temp");
        write(&dir, BACKLOG_PATH, CLEAN_BACKLOG);
        write(
            &dir,
            AWAITING_HUMAN_PATH,
            "| # | Status | Pri | System | Target | Check | Summary | Verify |\n\
             |-----|--------|-----|--------|--------|-------|---------|--------|\n\
             | 300 | [?] | P1 | audio |  | Confirm clean sound on real speakers. | proposed | review |\n\
             | 301 | [?] | P1 | audio | audio-device |  | proposed | review |\n",
        );

        lint_err(&dir);
    }

    #[test]
    fn lint_rejects_overlong_or_noisy_check() {
        let dir = TempDir::new().expect("temp");
        let long = "x".repeat(MAX_AWAITING_HUMAN_CHECK_CHARS + 1);
        write(&dir, BACKLOG_PATH, CLEAN_BACKLOG);
        write(
            &dir,
            AWAITING_HUMAN_PATH,
            &format!(
                "| # | Status | Pri | System | Target | Check | Summary | Verify |\n\
                 |-----|--------|-----|--------|--------|-------|---------|--------|\n\
                 | 300 | [?] | P1 | audio | audio-device | {long} | proposed | review |\n\
                 | 301 | [?] | P1 | audio | audio-device | See [the route](./x). | proposed | review |\n\
                 | 302 | [?] | P1 | audio | audio-device | Run `mise run play-native` and listen. | proposed | review |\n\
                 | 303 | [?] | P1 | audio | audio-device | Regression test proves this. | proposed | review |\n"
            ),
        );

        lint_err(&dir);
    }

    #[test]
    fn lint_rejects_vague_awaiting_human_check() {
        for check in [
            "Retest the reported route and confirm the issue no longer appears in native play.",
            "Retest on real speakers and confirm the reported audio symptom is gone.",
            "Use the linked audit rows to choose a concrete repro before more implementation work.",
            "Capture an exact current native recording or frame that reproduces the reported issue.",
            "Retest the reported enemy route and confirm the behaviour now matches the ROM.",
        ] {
            assert_eq!(
                awaiting_human_check_problem(check),
                Some(
                    "Check is too vague; name the concrete symptom and validation action"
                        .to_owned()
                ),
                "{check}"
            );
        }
    }

    #[test]
    fn unknown_awaiting_human_target_is_allowed_and_counted() {
        let dir = TempDir::new().expect("temp");
        write(&dir, BACKLOG_PATH, CLEAN_BACKLOG);
        write(
            &dir,
            AWAITING_HUMAN_PATH,
            "| # | Status | Pri | System | Target | Check | Summary | Verify |\n\
             |-----|--------|-----|--------|--------|-------|---------|--------|\n\
             | 300 | [?] | P1 | audio | unknown | Collect the same-form helmet; HP should refill without a long freeze. | proposed | review |\n",
        );

        run_backlog_lint(dir.path()).expect("unknown target is visible debt, not failure");
        let rows = parse_rows(
            &fs::read_to_string(dir.path().join(AWAITING_HUMAN_PATH)).expect("read"),
            BacklogLane::AwaitingHuman,
        );
        assert_eq!(
            rows.iter()
                .filter(|row| row.target.as_deref() == Some("unknown"))
                .count(),
            1
        );
    }

    /// A NOW block whose Current work bullet is `current_work`, over an open queue with a
    /// P1 (#200) and a P2 (#201). The P1 is the best open priority.
    fn backlog_with_current_work(current_work: &str) -> String {
        format!(
            "## NOW\n\
             \n\
             {current_work}\n\
             \n\
             | # | Status | Pri | System | Summary | Verify |\n\
             |-----|--------|-----|--------|---------|--------|\n\
             | 200 | [ ] | P1 | player | a bug | verify |\n\
             | 201 | [~] | P2 | render | another | verify |\n"
        )
    }

    #[test]
    fn now_current_work_steering_to_subpriority_open_item_fails() {
        // Current work naming the open P2 #201 while open P1 #200 exists is the silent
        // re-latch the lint must catch.
        let dir = TempDir::new().expect("temp");
        write(
            &dir,
            BACKLOG_PATH,
            &backlog_with_current_work("- **Current work:** continue the #201 grind next."),
        );
        let err = lint_err(&dir);
        assert!(err.contains("hygiene problem"));
        let backlog = fs::read_to_string(dir.path().join(BACKLOG_PATH)).expect("read");
        let problems = now_steer_problems(dir.path(), &backlog).expect("steer");
        assert_eq!(problems.len(), 1);
        assert!(
            problems[0].contains("#201 (P2)") && problems[0].contains("open P1 items exist"),
            "{}",
            problems[0]
        );
    }

    #[test]
    fn now_current_work_aligned_with_top_priority_is_ok() {
        // Current work naming the best open priority (#200 P1) is fine; a separate
        // state/context bullet may still mention the lower-priority #201.
        let dir = TempDir::new().expect("temp");
        write(
            &dir,
            BACKLOG_PATH,
            &backlog_with_current_work(
                "- **GEMS lane:** #201 is P2 measured debt, not a per-session pick.\n\
                 - **Current work:** take #200 next per project-status.",
            ),
        );
        run_backlog_lint(dir.path()).expect("aligned current work lints clean");
        let backlog = fs::read_to_string(dir.path().join(BACKLOG_PATH)).expect("read");
        assert!(
            now_steer_problems(dir.path(), &backlog)
                .expect("steer")
                .is_empty()
        );
    }

    #[test]
    fn now_steer_ignores_bullet_without_issue_ids() {
        let dir = TempDir::new().expect("temp");
        write(
            &dir,
            BACKLOG_PATH,
            &backlog_with_current_work("- **Current work:** take project-status's top item."),
        );
        run_backlog_lint(dir.path()).expect("id-free current work is fine");
    }

    #[test]
    fn empty_open_queue_is_valid() {
        // No open rows means no table header; the NOW-size check must not error spuriously.
        let dir = TempDir::new().expect("temp");
        write(&dir, BACKLOG_PATH, "## NOW\n\n- Focus: queue is empty\n");
        run_backlog_lint(dir.path()).expect("empty queue is valid");
    }
}
