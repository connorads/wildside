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

use anyhow::{Context, Result};
use std::fs;
use std::path::Path;

/// Primary lane: the NOW handoff and the open queue. Required.
pub(crate) const BACKLOG_PATH: &str = "docs/source-port/backlog.md";
/// Human-validation lane: `[?]`/`[b]` rows. Optional.
pub(crate) const AWAITING_HUMAN_PATH: &str = "docs/source-port/backlog-awaiting-human.md";

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
}
