//! SDL-free playtest session controller.
//!
//! Playtest mode turns `play-native` into an issue-validation harness: the
//! awaiting-human backlog lane feeds an in-game issue browser, verdicts and
//! bookmarks land in a crash-safe `session.json`, and gameplay is captured as
//! standard recording bundles (one per deterministic segment). Everything here
//! is SDL-free so browser/HUD logic and the session contract stay unit-testable;
//! `native_player/mod.rs` owns the event pump and key mapping.

use crate::backlog::is_valid_awaiting_human_target;
use crate::project_status::{HumanValidationBlocker, load_human_validation_blockers};
use crate::recording::RecordingStart;
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

/// Session-level playtest inputs resolved by `main.rs` (repo paths and backlog
/// IO stay out of the SDL loop).
#[derive(Debug, Clone)]
pub struct PlaytestOptions {
    pub session_dir: PathBuf,
    pub issues: Vec<HumanValidationBlocker>,
}

/// The `session.json` contract consumed by `playtest-ingest` and follow-up
/// agent sessions. Rewritten wholesale on every verdict/bookmark so a crash
/// loses at most the in-flight event.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub(crate) struct PlaytestSessionFile {
    pub(crate) session_slug: String,
    pub(crate) started_at_unix: u64,
    pub(crate) rom_sha1: String,
    #[serde(default)]
    pub(crate) git_commit: Option<String>,
    #[serde(default)]
    pub(crate) git_dirty: Option<bool>,
    #[serde(default)]
    pub(crate) segments: Vec<SegmentRecord>,
    #[serde(default)]
    pub(crate) verdicts: Vec<Verdict>,
    #[serde(default)]
    pub(crate) bookmarks: Vec<Bookmark>,
    #[serde(default)]
    pub(crate) sync_marks: Vec<SyncMark>,
    #[serde(default)]
    pub(crate) mic: Option<MicInfo>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub(crate) struct SegmentRecord {
    pub(crate) index: usize,
    pub(crate) slug: String,
    /// Bundle directory relative to the session dir (`segments/segNN`).
    pub(crate) bundle_dir: String,
    pub(crate) start: RecordingStart,
    pub(crate) saved: bool,
    /// Why the segment ended (`issue-warp`, `debug-next`, `exit`, ...).
    pub(crate) reason: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) enum VerdictKind {
    ConfirmedFixed,
    StillBroken,
    Partial,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub(crate) struct Verdict {
    pub(crate) issue_id: String,
    pub(crate) target: String,
    pub(crate) check: String,
    pub(crate) verdict: VerdictKind,
    pub(crate) segment_index: usize,
    pub(crate) frame_index: usize,
    pub(crate) at_unix_ms: u64,
    #[serde(default)]
    pub(crate) note: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub(crate) struct Bookmark {
    /// Draft issue id (`PT-001`); a follow-up agent session assigns real `#NNN`s.
    pub(crate) draft_id: String,
    pub(crate) segment_index: usize,
    pub(crate) frame_index: usize,
    pub(crate) level: u8,
    /// Kid position in level pixels (trace fixed-point `>> 16` already applied).
    pub(crate) kid_x: Option<u32>,
    pub(crate) kid_y: Option<u32>,
    /// Screenshot path relative to the session dir (`bookmarks/PT-001.png`).
    pub(crate) screenshot: String,
    pub(crate) at_unix_ms: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) enum SyncMarkKind {
    SegmentStart,
    Pause,
    Resume,
    Interval,
}

/// Frame↔wall-clock correspondence for aligning the mic transcript. Pause and
/// Resume marks bound interpolation so paused wall time does not inflate the
/// frame estimate.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct SyncMark {
    pub(crate) kind: SyncMarkKind,
    pub(crate) wall_clock_ms: u64,
    pub(crate) segment_index: usize,
    pub(crate) frame_index: usize,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub(crate) struct MicInfo {
    pub(crate) command: String,
    /// WAV path relative to the session dir (`mic.wav`).
    pub(crate) wav_path: String,
    pub(crate) sample_rate: u32,
    pub(crate) channels: u16,
    pub(crate) started_at_unix_ms: u64,
}

/// Live playtest session state owned by the player loop.
pub(crate) struct PlaytestSession {
    session_dir: PathBuf,
    file: PlaytestSessionFile,
    issues: Vec<HumanValidationBlocker>,
}

impl PlaytestSession {
    /// Create the session directory and write the initial `session.json`.
    pub(crate) fn create(
        options: &PlaytestOptions,
        rom_sha1: String,
        git_commit: Option<String>,
        git_dirty: Option<bool>,
    ) -> Result<Self> {
        let session_dir = options.session_dir.clone();
        let session_slug = session_dir
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("playtest")
            .to_owned();
        fs::create_dir_all(&session_dir)
            .with_context(|| format!("failed to create {}", session_dir.display()))?;
        let session = Self {
            session_dir,
            file: PlaytestSessionFile {
                session_slug,
                started_at_unix: unix_time_ms() / 1000,
                rom_sha1,
                git_commit,
                git_dirty,
                ..PlaytestSessionFile::default()
            },
            issues: options.issues.clone(),
        };
        session.persist()?;
        Ok(session)
    }

    /// Rewrite `session.json` in full (crash-safe incremental contract).
    pub(crate) fn persist(&self) -> Result<()> {
        let path = self.session_dir.join("session.json");
        fs::write(&path, serde_json::to_vec_pretty(&self.file)?)
            .with_context(|| format!("failed to write {}", path.display()))
    }

    pub(crate) fn session_dir(&self) -> &Path {
        &self.session_dir
    }

    pub(crate) fn issues(&self) -> &[HumanValidationBlocker] {
        &self.issues
    }

    #[cfg(test)]
    pub(crate) fn file(&self) -> &PlaytestSessionFile {
        &self.file
    }
}

/// Load the awaiting-human lane for the in-game browser, skipping rows whose
/// Target would not round-trip through the warp/arm table.
pub(crate) fn load_playtest_issues(repo_root: &Path) -> Vec<HumanValidationBlocker> {
    load_human_validation_blockers(repo_root)
        .into_iter()
        .filter(|issue| is_valid_awaiting_human_target(&issue.target))
        .collect()
}

/// Session slug scheme: `pt-YYYYMMDD-HHMMSS` (UTC).
pub(crate) fn playtest_session_slug(unix_secs: u64) -> String {
    let (year, month, day) = civil_from_unix_days(unix_secs / 86_400);
    let secs_of_day = unix_secs % 86_400;
    format!(
        "pt-{year:04}{month:02}{day:02}-{:02}{:02}{:02}",
        secs_of_day / 3600,
        (secs_of_day % 3600) / 60,
        secs_of_day % 60
    )
}

/// Days-since-epoch → civil date (Howard Hinnant's `civil_from_days`, valid for
/// the unsigned post-1970 range used here).
fn civil_from_unix_days(days: u64) -> (u64, u64, u64) {
    let days = days + 719_468;
    let era = days / 146_097;
    let day_of_era = days % 146_097;
    let year_of_era =
        (day_of_era - day_of_era / 1460 + day_of_era / 36_524 - day_of_era / 146_096) / 365;
    let year = year_of_era + era * 400;
    let day_of_year = day_of_era - (365 * year_of_era + year_of_era / 4 - year_of_era / 100);
    let mp = (5 * day_of_year + 2) / 153;
    let day = day_of_year - (153 * mp + 2) / 5 + 1;
    let month = if mp < 10 { mp + 3 } else { mp - 9 };
    (if month <= 2 { year + 1 } else { year }, month, day)
}

pub(crate) fn unix_time_ms() -> u64 {
    use std::time::SystemTime;
    SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_issue(id: &str, target: &str) -> HumanValidationBlocker {
        HumanValidationBlocker {
            id: id.to_owned(),
            status: String::from("[?]"),
            priority: String::from("P1"),
            system: String::from("enemies"),
            target: target.to_owned(),
            check: String::from("Dragons resume patrol after the Kid transformation."),
            summary: String::from("dragon patrol freeze"),
        }
    }

    #[test]
    fn playtest_session_slug_formats_utc_timestamp() {
        // 2026-07-12 09:08:07 UTC
        assert_eq!(playtest_session_slug(1_783_847_287), "pt-20260712-090807");
        assert_eq!(playtest_session_slug(0), "pt-19700101-000000");
        // Leap-year boundary: 2024-02-29 23:59:59 UTC
        assert_eq!(playtest_session_slug(1_709_251_199), "pt-20240229-235959");
    }

    #[test]
    fn session_file_round_trips_through_json() {
        let file = PlaytestSessionFile {
            session_slug: String::from("pt-20260712-090807"),
            started_at_unix: 1_783_933_687,
            rom_sha1: String::from("abc123"),
            git_commit: Some(String::from("deadbeef")),
            git_dirty: Some(false),
            segments: vec![SegmentRecord {
                index: 1,
                slug: String::from("pt-20260712-090807-seg01"),
                bundle_dir: String::from("segments/seg01"),
                start: RecordingStart {
                    warp_level: Some(0x03),
                    settled: false,
                    record_frames: None,
                },
                saved: true,
                reason: String::from("issue-warp"),
            }],
            verdicts: vec![Verdict {
                issue_id: String::from("#238"),
                target: String::from("0x03"),
                check: String::from("Dragons resume patrol after the Kid transformation."),
                verdict: VerdictKind::StillBroken,
                segment_index: 1,
                frame_index: 512,
                at_unix_ms: 1_783_933_700_123,
                note: None,
            }],
            bookmarks: vec![Bookmark {
                draft_id: String::from("PT-001"),
                segment_index: 1,
                frame_index: 900,
                level: 0x03,
                kid_x: Some(230),
                kid_y: Some(271),
                screenshot: String::from("bookmarks/PT-001.png"),
                at_unix_ms: 1_783_933_712_456,
            }],
            sync_marks: vec![SyncMark {
                kind: SyncMarkKind::SegmentStart,
                wall_clock_ms: 1_783_933_690_000,
                segment_index: 1,
                frame_index: 0,
            }],
            mic: Some(MicInfo {
                command: String::from("ffmpeg -f avfoundation -i :0 ..."),
                wav_path: String::from("mic.wav"),
                sample_rate: 16_000,
                channels: 1,
                started_at_unix_ms: 1_783_933_689_500,
            }),
        };

        let json = serde_json::to_string_pretty(&file).expect("serialise session file");
        let parsed: PlaytestSessionFile = serde_json::from_str(&json).expect("parse session file");
        assert_eq!(parsed, file);
    }

    #[test]
    fn session_file_tolerates_missing_optional_sections() {
        let parsed: PlaytestSessionFile =
            serde_json::from_str(r#"{"session_slug":"pt-x","started_at_unix":1,"rom_sha1":"abc"}"#)
                .expect("parse minimal session file");
        assert!(parsed.segments.is_empty());
        assert!(parsed.mic.is_none());
    }

    #[test]
    fn create_writes_initial_session_json() {
        let root = tempfile::tempdir().expect("tempdir");
        let session_dir = root.path().join("pt-20260712-090807");
        let options = PlaytestOptions {
            session_dir: session_dir.clone(),
            issues: vec![sample_issue("#238", "0x03")],
        };

        let session = PlaytestSession::create(
            &options,
            String::from("romsha"),
            Some(String::from("deadbeef")),
            Some(true),
        )
        .expect("create session");

        assert_eq!(session.issues().len(), 1);
        assert_eq!(session.session_dir(), session_dir.as_path());
        let written = fs::read_to_string(session_dir.join("session.json")).expect("session.json");
        let parsed: PlaytestSessionFile =
            serde_json::from_str(&written).expect("parse session.json");
        assert_eq!(parsed.session_slug, "pt-20260712-090807");
        assert_eq!(parsed.rom_sha1, "romsha");
        assert_eq!(parsed.git_commit.as_deref(), Some("deadbeef"));
        assert_eq!(session.file(), &parsed);
    }
}
