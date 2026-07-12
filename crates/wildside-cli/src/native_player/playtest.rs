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
use wildside_core::NativeTraceFrame;
use wildside_frame::{DrawCommand, FRAME_HEIGHT, FRAME_WIDTH, Scene, text};

/// Frames a toast confirmation stays on screen (~3 seconds at 60 fps).
const TOAST_HOLD_FRAMES: usize = 180;
/// Issue rows visible at once in the browser window.
const BROWSER_VISIBLE_ROWS: usize = 14;
/// 320px wide frame / 8px glyphs.
const BROWSER_TEXT_COLUMNS: usize = (FRAME_WIDTH as usize) / 8;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum IssueFilter {
    /// Issues targeting the level being played, plus `unknown` targets.
    CurrentLevel,
    All,
}

/// What arming an issue asks the player loop to do. Warp-capable targets start
/// a fresh deterministic segment; everything else pins the Check and waits for
/// the player to reach the target manually.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum PlaytestWarp {
    Level(u8),
    Boot,
}

/// The issue currently under validation: its Check stays pinned on the HUD
/// until a Y/H/G verdict lands.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ArmedIssue {
    pub(crate) issue_id: String,
    pub(crate) target: String,
    pub(crate) check: String,
}

#[derive(Debug, Clone)]
struct ActiveSegmentMeta {
    index: usize,
    slug: String,
    bundle_dir: String,
    start: RecordingStart,
}

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
    browser_open: bool,
    /// Selection index into the *filtered* issue list.
    browser_selection: usize,
    filter: IssueFilter,
    toast: Option<(String, usize)>,
    armed: Option<ArmedIssue>,
    active_segment: Option<ActiveSegmentMeta>,
    segment_counter: usize,
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
            browser_open: false,
            browser_selection: 0,
            filter: IssueFilter::CurrentLevel,
            toast: None,
            armed: None,
            active_segment: None,
            segment_counter: 0,
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

    /// Player-level pause: gameplay must not step (and segments must not
    /// record) while any playtest UI holds the game.
    pub(crate) fn paused(&self) -> bool {
        self.browser_open
    }

    pub(crate) fn browser_open(&self) -> bool {
        self.browser_open
    }

    pub(crate) fn toggle_browser(&mut self, current_level: u8) {
        if self.browser_open {
            self.close_browser();
        } else {
            self.browser_open = true;
            self.clamp_selection(current_level);
        }
    }

    pub(crate) fn close_browser(&mut self) {
        self.browser_open = false;
    }

    pub(crate) fn move_selection(&mut self, delta: i64, current_level: u8) {
        let len = self.filtered_issues(current_level).len();
        if len == 0 {
            self.browser_selection = 0;
            return;
        }
        let current = self.browser_selection.min(len - 1) as i64;
        self.browser_selection = current.saturating_add(delta).clamp(0, len as i64 - 1) as usize;
    }

    pub(crate) fn toggle_filter(&mut self, current_level: u8) {
        self.filter = match self.filter {
            IssueFilter::CurrentLevel => IssueFilter::All,
            IssueFilter::All => IssueFilter::CurrentLevel,
        };
        self.clamp_selection(current_level);
        let label = match self.filter {
            IssueFilter::CurrentLevel => format!("LEVEL {current_level:02X}+UNKNOWN"),
            IssueFilter::All => String::from("ALL"),
        };
        self.show_toast(format!("FILTER: {label}"));
    }

    fn clamp_selection(&mut self, current_level: u8) {
        let len = self.filtered_issues(current_level).len();
        self.browser_selection = self.browser_selection.min(len.saturating_sub(1));
    }

    pub(crate) fn filtered_issues(&self, current_level: u8) -> Vec<&HumanValidationBlocker> {
        self.issues
            .iter()
            .filter(|issue| issue_matches_filter(issue, self.filter, current_level))
            .collect()
    }

    /// Arm the selected issue: close the browser, pin its Check on the HUD,
    /// and tell the loop whether the target warrants a warp + fresh segment.
    pub(crate) fn arm_selected(&mut self, current_level: u8) -> Option<PlaytestWarp> {
        let filtered = self.filtered_issues(current_level);
        let selection = self.browser_selection.min(filtered.len().saturating_sub(1));
        let issue = (*filtered.get(selection)?).clone();
        let warp = if issue.target == "boot" {
            Some(PlaytestWarp::Boot)
        } else {
            issue
                .target
                .strip_prefix("0x")
                .and_then(|hex| u8::from_str_radix(hex, 16).ok())
                .map(PlaytestWarp::Level)
        };
        self.show_toast(format!("ARMED {} [{}]", issue.id, issue.target));
        self.armed = Some(ArmedIssue {
            issue_id: issue.id,
            target: issue.target,
            check: issue.check,
        });
        self.close_browser();
        warp
    }

    pub(crate) fn armed(&self) -> Option<&ArmedIssue> {
        self.armed.as_ref()
    }

    /// Record a Y/H/G verdict for the armed issue at the current segment
    /// position. Returns true when a verdict was recorded (an issue was armed).
    pub(crate) fn record_verdict(&mut self, kind: VerdictKind, frame_index: usize) -> Result<bool> {
        let Some(armed) = self.armed.take() else {
            return Ok(false);
        };
        let label = match kind {
            VerdictKind::ConfirmedFixed => "CONFIRMED FIXED",
            VerdictKind::StillBroken => "STILL BROKEN",
            VerdictKind::Partial => "PARTIAL",
        };
        self.show_toast(format!("{}: {label}", armed.issue_id));
        self.file.verdicts.push(Verdict {
            issue_id: armed.issue_id,
            target: armed.target,
            check: armed.check,
            verdict: kind,
            segment_index: self.current_segment_index(),
            frame_index,
            at_unix_ms: unix_time_ms(),
            note: None,
        });
        self.persist()?;
        Ok(true)
    }

    pub(crate) fn current_segment_index(&self) -> usize {
        self.active_segment
            .as_ref()
            .map_or(self.segment_counter, |segment| segment.index)
    }

    /// Register the next segment and return its absolute bundle directory.
    pub(crate) fn begin_segment(&mut self, start: RecordingStart) -> PathBuf {
        self.segment_counter += 1;
        let index = self.segment_counter;
        let bundle_dir = format!("segments/seg{index:02}");
        let slug = format!("{}-seg{index:02}", self.file.session_slug);
        self.active_segment = Some(ActiveSegmentMeta {
            index,
            slug,
            bundle_dir: bundle_dir.clone(),
            start,
        });
        self.session_dir.join(bundle_dir)
    }

    /// Close out the active segment's bookkeeping after its recorder was
    /// flushed (`saved`) or discarded.
    pub(crate) fn end_segment(&mut self, saved: bool, reason: &str) -> Result<()> {
        let Some(segment) = self.active_segment.take() else {
            return Ok(());
        };
        self.file.segments.push(SegmentRecord {
            index: segment.index,
            slug: segment.slug,
            bundle_dir: segment.bundle_dir,
            start: segment.start,
            saved,
            reason: reason.to_owned(),
        });
        self.persist()
    }

    /// Drop a draft-ID bookmark at the current trace position and return the
    /// absolute path the caller should write the screenshot to.
    pub(crate) fn add_bookmark(
        &mut self,
        trace: &NativeTraceFrame,
        segment_frame_index: usize,
    ) -> Result<PathBuf> {
        let draft_id = format!("PT-{:03}", self.file.bookmarks.len() + 1);
        let screenshot = format!("bookmarks/{draft_id}.png");
        let screenshot_path = self.session_dir.join(&screenshot);
        fs::create_dir_all(self.session_dir.join("bookmarks"))
            .with_context(|| format!("failed to create {}", self.session_dir.display()))?;
        self.show_toast(format!("BOOKMARK {draft_id}"));
        self.file.bookmarks.push(Bookmark {
            draft_id,
            segment_index: self.current_segment_index(),
            frame_index: segment_frame_index,
            level: trace.current_level.0,
            kid_x: trace.kid_x_pos.map(|position| position >> 16),
            kid_y: trace.kid_y_pos.map(|position| position >> 16),
            screenshot,
            at_unix_ms: unix_time_ms(),
        });
        self.persist()?;
        Ok(screenshot_path)
    }

    pub(crate) fn show_toast(&mut self, message: impl Into<String>) {
        self.toast = Some((message.into(), TOAST_HOLD_FRAMES));
    }

    /// Per-frame housekeeping (toast countdown).
    pub(crate) fn tick(&mut self) {
        if let Some((_, remaining)) = self.toast.as_mut() {
            *remaining = remaining.saturating_sub(1);
            if *remaining == 0 {
                self.toast = None;
            }
        }
    }

    /// The overlay to composite over the finished frame, if any.
    pub(crate) fn overlay_scene(&self, current_level: u8) -> Option<Scene> {
        if self.browser_open {
            return Some(issue_browser_scene(
                &self.filtered_issues(current_level),
                self.browser_selection,
                self.filter,
                current_level,
            ));
        }
        let mut commands = Vec::new();
        if let Some(armed) = self.armed.as_ref() {
            commands.extend(armed_hud_commands(armed));
        }
        if let Some((message, _)) = self.toast.as_ref() {
            commands.extend(toast_commands(message));
        }
        if commands.is_empty() {
            return None;
        }
        Some(Scene {
            background: [0, 0, 0],
            commands,
            pause_palette_darken: false,
        })
    }

    #[cfg(test)]
    pub(crate) fn file(&self) -> &PlaytestSessionFile {
        &self.file
    }
}

fn issue_matches_filter(
    issue: &HumanValidationBlocker,
    filter: IssueFilter,
    current_level: u8,
) -> bool {
    match filter {
        IssueFilter::All => true,
        IssueFilter::CurrentLevel => {
            issue.target == "unknown" || target_level(&issue.target) == Some(current_level)
        }
    }
}

/// Level a `0xNN` (or `demo:0xNN`) target points at, if any.
fn target_level(target: &str) -> Option<u8> {
    let hex = target
        .strip_prefix("0x")
        .or_else(|| target.strip_prefix("demo:0x"))?;
    u8::from_str_radix(hex, 16).ok()
}

/// Pause-screen issue browser: dim backdrop, windowed issue rows, filter and
/// key-hint lines. `issues` is the already-filtered list.
fn issue_browser_scene(
    issues: &[&HumanValidationBlocker],
    selection: usize,
    filter: IssueFilter,
    current_level: u8,
) -> Scene {
    let mut commands = vec![DrawCommand::Tint {
        colour: [0, 0, 0],
        alpha: 176,
    }];
    let filter_label = match filter {
        IssueFilter::CurrentLevel => format!("LEVEL {current_level:02X}+UNKNOWN"),
        IssueFilter::All => String::from("ALL"),
    };
    commands.push(text(
        8,
        8,
        &format!("PLAYTEST ISSUES  FILTER: {filter_label}"),
        [255, 255, 160],
        1,
    ));

    if issues.is_empty() {
        commands.push(text(
            8,
            32,
            "NO AWAITING-HUMAN ISSUES FOR THIS FILTER",
            [200, 200, 200],
            1,
        ));
    } else {
        let selection = selection.min(issues.len() - 1);
        let window_start = selection.saturating_sub(BROWSER_VISIBLE_ROWS - 1);
        for (row, (index, issue)) in issues
            .iter()
            .enumerate()
            .skip(window_start)
            .take(BROWSER_VISIBLE_ROWS)
            .enumerate()
        {
            let selected = index == selection;
            let marker = if selected { ">" } else { " " };
            let line = format!(
                "{marker} {} {} {} {} {}",
                issue.id, issue.status, issue.priority, issue.target, issue.summary
            );
            let line: String = line.chars().take(BROWSER_TEXT_COLUMNS).collect();
            let colour = if selected {
                [255, 255, 255]
            } else {
                [170, 190, 210]
            };
            commands.push(text(8, 24 + (row as i32) * 10, &line, colour, 1));
        }
        if let Some(issue) = issues.get(selection) {
            let check_chars: Vec<char> = issue.check.chars().collect();
            let first: String = check_chars.iter().take(BROWSER_TEXT_COLUMNS).collect();
            let rest: String = check_chars
                .iter()
                .skip(BROWSER_TEXT_COLUMNS)
                .take(BROWSER_TEXT_COLUMNS)
                .collect();
            commands.push(text(8, 176, &first, [160, 220, 160], 1));
            if !rest.is_empty() {
                commands.push(text(8, 186, &rest, [160, 220, 160], 1));
            }
        }
    }

    commands.push(text(
        8,
        (FRAME_HEIGHT as i32) - 16,
        "UP/DOWN SELECT  L/R FILTER  ENTER ARM  F1 RESUME",
        [140, 140, 140],
        1,
    ));

    Scene {
        background: [0, 0, 0],
        commands,
        pause_palette_darken: false,
    }
}

/// Armed-issue HUD pinned to the top of the frame: issue id, target, verdict
/// key hints, and the Check sentence across one or two lines.
fn armed_hud_commands(armed: &ArmedIssue) -> Vec<DrawCommand> {
    let check_chars: Vec<char> = armed.check.chars().collect();
    let first: String = check_chars.iter().take(BROWSER_TEXT_COLUMNS).collect();
    let rest: String = check_chars
        .iter()
        .skip(BROWSER_TEXT_COLUMNS)
        .take(BROWSER_TEXT_COLUMNS)
        .collect();
    let height = if rest.is_empty() { 24 } else { 34 };
    let mut commands = vec![
        DrawCommand::Rect {
            x: 0,
            y: 0,
            width: FRAME_WIDTH as i32,
            height,
            colour: [16, 24, 40],
        },
        text(
            4,
            3,
            &format!(
                "{} [{}]  Y FIXED H BROKEN G PARTIAL",
                armed.issue_id, armed.target
            ),
            [255, 255, 160],
            1,
        ),
        text(4, 13, &first, [160, 220, 160], 1),
    ];
    if !rest.is_empty() {
        commands.push(text(4, 23, &rest, [160, 220, 160], 1));
    }
    commands
}

/// Write a 320x224 RGB24 frame as the bookmark screenshot.
pub(crate) fn write_playtest_screenshot(path: &Path, rgb: &[u8]) -> Result<()> {
    image::save_buffer_with_format(
        path,
        rgb,
        FRAME_WIDTH,
        FRAME_HEIGHT,
        image::ColorType::Rgb8,
        image::ImageFormat::Png,
    )
    .with_context(|| format!("failed to write {}", path.display()))
}

/// Transient confirmation banner at the bottom of the frame.
fn toast_commands(message: &str) -> Vec<DrawCommand> {
    let message: String = message.chars().take(BROWSER_TEXT_COLUMNS - 2).collect();
    vec![
        DrawCommand::Rect {
            x: 0,
            y: (FRAME_HEIGHT as i32) - 14,
            width: FRAME_WIDTH as i32,
            height: 14,
            colour: [24, 32, 48],
        },
        text(8, (FRAME_HEIGHT as i32) - 11, &message, [255, 255, 200], 1),
    ]
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

    fn session_with_issues(
        issues: Vec<HumanValidationBlocker>,
    ) -> (PlaytestSession, tempfile::TempDir) {
        let root = tempfile::tempdir().expect("tempdir");
        let options = PlaytestOptions {
            session_dir: root.path().join("pt-test"),
            issues,
        };
        let session = PlaytestSession::create(&options, String::from("romsha"), None, None)
            .expect("create session");
        (session, root)
    }

    fn scene_text(scene: &Scene) -> Vec<String> {
        scene
            .commands
            .iter()
            .filter_map(|command| match command {
                DrawCommand::Text { text, .. } => Some(text.clone()),
                _ => None,
            })
            .collect()
    }

    #[test]
    fn current_level_filter_keeps_matching_level_and_unknown_targets() {
        let (session, _root) = session_with_issues(vec![
            sample_issue("238", "0x03"),
            sample_issue("240", "0x11"),
            sample_issue("241", "unknown"),
            sample_issue("242", "demo:0x03"),
            sample_issue("243", "boot"),
        ]);

        let filtered = session.filtered_issues(0x03);
        let ids: Vec<&str> = filtered.iter().map(|issue| issue.id.as_str()).collect();
        assert_eq!(
            ids,
            vec!["238", "241", "242"],
            "current-level filter keeps 0xNN/demo:0xNN targets for the played level plus unknown"
        );

        let mut session = session;
        session.toggle_filter(0x03);
        assert_eq!(
            session.filtered_issues(0x03).len(),
            5,
            "ALL keeps every issue"
        );
    }

    #[test]
    fn browser_scene_lists_issues_and_marks_selection() {
        let (mut session, _root) = session_with_issues(vec![
            sample_issue("238", "0x03"),
            sample_issue("241", "unknown"),
        ]);
        session.toggle_browser(0x03);
        assert!(session.paused(), "open browser pauses the player");
        session.move_selection(1, 0x03);

        let scene = session.overlay_scene(0x03).expect("browser scene");
        let lines = scene_text(&scene);
        assert!(lines[0].contains("FILTER: LEVEL 03+UNKNOWN"));
        assert!(
            lines.iter().any(|line| line.starts_with("  238")),
            "unselected row keeps a blank marker: {lines:?}"
        );
        assert!(
            lines.iter().any(|line| line.starts_with("> 241")),
            "selected row carries the > marker: {lines:?}"
        );
        assert!(
            lines
                .iter()
                .any(|line| line.contains("Dragons resume patrol")),
            "selected issue pins its Check sentence: {lines:?}"
        );
    }

    #[test]
    fn browser_scene_shows_placeholder_for_empty_lane() {
        let (mut session, _root) = session_with_issues(Vec::new());
        session.toggle_browser(0x00);
        session.move_selection(1, 0x00);

        let scene = session.overlay_scene(0x00).expect("browser scene");
        let lines = scene_text(&scene);
        assert!(
            lines
                .iter()
                .any(|line| line.contains("NO AWAITING-HUMAN ISSUES")),
            "empty lane shows a placeholder: {lines:?}"
        );
    }

    #[test]
    fn selection_clamps_when_filter_shrinks_the_list() {
        let (mut session, _root) = session_with_issues(vec![
            sample_issue("238", "0x03"),
            sample_issue("240", "0x11"),
            sample_issue("241", "0x11"),
        ]);
        session.toggle_browser(0x11);
        session.move_selection(5, 0x11);
        assert_eq!(session.filtered_issues(0x11).len(), 2);

        // ALL grows the list; back to current-level shrinks it again.
        session.toggle_filter(0x11);
        session.move_selection(5, 0x11);
        session.toggle_filter(0x11);

        let scene = session.overlay_scene(0x11).expect("browser scene");
        let lines = scene_text(&scene);
        assert!(
            lines.iter().any(|line| line.starts_with("> 241")),
            "selection clamps to the last filtered row: {lines:?}"
        );
    }

    #[test]
    fn toast_shows_after_browser_close_and_expires() {
        let (mut session, _root) = session_with_issues(vec![sample_issue("238", "0x03")]);
        session.toggle_browser(0x03);
        session.toggle_filter(0x03);
        session.close_browser();
        assert!(!session.paused());

        let scene = session.overlay_scene(0x03).expect("toast scene");
        let lines = scene_text(&scene);
        assert!(lines.iter().any(|line| line.contains("FILTER: ALL")));

        for _ in 0..TOAST_HOLD_FRAMES {
            session.tick();
        }
        assert!(
            session.overlay_scene(0x03).is_none(),
            "toast expires after its hold frames"
        );
    }

    fn segment_start(warp_level: Option<u8>) -> RecordingStart {
        RecordingStart {
            warp_level,
            settled: false,
            record_frames: None,
        }
    }

    #[test]
    fn arming_pins_check_and_reports_warp_action() {
        let (mut session, _root) = session_with_issues(vec![
            sample_issue("238", "0x03"),
            sample_issue("240", "boot"),
            sample_issue("241", "demo:0x05"),
        ]);
        session.toggle_browser(0x03);
        session.toggle_filter(0x03); // ALL, so every target is reachable

        assert_eq!(session.arm_selected(0x03), Some(PlaytestWarp::Level(0x03)));
        assert!(!session.browser_open(), "arming closes the browser");
        assert_eq!(
            session.armed().map(|armed| armed.issue_id.as_str()),
            Some("238")
        );

        session.toggle_browser(0x03);
        session.move_selection(1, 0x03);
        assert_eq!(session.arm_selected(0x03), Some(PlaytestWarp::Boot));

        session.toggle_browser(0x03);
        session.move_selection(2, 0x03);
        assert_eq!(
            session.arm_selected(0x03),
            None,
            "demo targets arm without warping"
        );
        assert_eq!(
            session.armed().map(|armed| armed.issue_id.as_str()),
            Some("241")
        );

        let scene = session.overlay_scene(0x03).expect("armed HUD");
        let lines = scene_text(&scene);
        assert!(
            lines
                .iter()
                .any(|line| line.contains("241") && line.contains("Y FIXED")),
            "armed HUD pins the issue id and verdict hints: {lines:?}"
        );
        assert!(
            lines
                .iter()
                .any(|line| line.contains("Dragons resume patrol")),
            "armed HUD pins the Check sentence: {lines:?}"
        );
    }

    #[test]
    fn verdicts_record_segment_position_and_disarm() {
        let (mut session, _root) = session_with_issues(vec![sample_issue("238", "0x03")]);
        let bundle_dir = session.begin_segment(segment_start(Some(0x03)));
        assert!(bundle_dir.ends_with("segments/seg01"));
        session.toggle_browser(0x03);
        session.arm_selected(0x03);

        assert!(
            session
                .record_verdict(VerdictKind::StillBroken, 42)
                .expect("record verdict")
        );
        assert!(session.armed().is_none(), "verdict disarms the issue");
        assert!(
            !session
                .record_verdict(VerdictKind::ConfirmedFixed, 43)
                .expect("no-op without armed issue"),
            "verdicts need an armed issue"
        );

        let verdict = session.file().verdicts.first().expect("stored verdict");
        assert_eq!(verdict.issue_id, "238");
        assert_eq!(verdict.verdict, VerdictKind::StillBroken);
        assert_eq!(verdict.segment_index, 1);
        assert_eq!(verdict.frame_index, 42);

        let written =
            fs::read_to_string(session.session_dir().join("session.json")).expect("session.json");
        let parsed: PlaytestSessionFile = serde_json::from_str(&written).expect("parse");
        assert_eq!(parsed.verdicts.len(), 1, "verdicts persist immediately");
    }

    #[test]
    fn segment_bookkeeping_numbers_and_persists_segments() {
        let (mut session, _root) = session_with_issues(Vec::new());
        let first = session.begin_segment(segment_start(None));
        assert!(first.ends_with("segments/seg01"));
        assert_eq!(session.current_segment_index(), 1);
        session.end_segment(true, "issue-warp").expect("end seg01");

        let second = session.begin_segment(segment_start(Some(0x05)));
        assert!(second.ends_with("segments/seg02"));
        session.end_segment(false, "exit").expect("end seg02");

        let segments = &session.file().segments;
        assert_eq!(segments.len(), 2);
        assert_eq!(segments[0].index, 1);
        assert!(segments[0].saved);
        assert_eq!(segments[0].reason, "issue-warp");
        assert_eq!(segments[0].bundle_dir, "segments/seg01");
        assert!(segments[0].slug.ends_with("-seg01"));
        assert_eq!(segments[1].index, 2);
        assert!(!segments[1].saved);
        assert_eq!(segments[1].start.warp_level, Some(0x05));
    }

    #[test]
    fn bookmarks_capture_trace_position_with_pixel_coordinates() {
        let (mut session, _root) = session_with_issues(Vec::new());
        session.begin_segment(segment_start(Some(0x03)));
        let trace = NativeTraceFrame {
            current_level: wildside_core::LevelId(0x03),
            kid_x_pos: Some(230 << 16),
            kid_y_pos: Some((271 << 16) | 0x8000), // sub-pixel bits must drop
            ..NativeTraceFrame::default()
        };
        let screenshot_path = session.add_bookmark(&trace, 512).expect("bookmark");
        assert!(screenshot_path.ends_with("bookmarks/PT-001.png"));

        let second = session
            .add_bookmark(&NativeTraceFrame::default(), 600)
            .expect("second bookmark");
        assert!(
            second.ends_with("bookmarks/PT-002.png"),
            "draft ids increment"
        );

        let bookmark = session.file().bookmarks.first().expect("stored bookmark");
        assert_eq!(bookmark.draft_id, "PT-001");
        assert_eq!(bookmark.segment_index, 1);
        assert_eq!(bookmark.frame_index, 512);
        assert_eq!(bookmark.level, 0x03);
        assert_eq!(bookmark.kid_x, Some(230));
        assert_eq!(
            bookmark.kid_y,
            Some(271),
            "16.16 fixed-point shifts to pixels"
        );
        assert_eq!(bookmark.screenshot, "bookmarks/PT-001.png");

        let written =
            fs::read_to_string(session.session_dir().join("session.json")).expect("session.json");
        let parsed: PlaytestSessionFile = serde_json::from_str(&written).expect("parse");
        assert_eq!(parsed.bookmarks.len(), 2, "bookmarks persist immediately");
    }

    #[test]
    fn screenshot_writer_produces_readable_png() {
        let root = tempfile::tempdir().expect("tempdir");
        let path = root.path().join("PT-001.png");
        let mut rgb = vec![0u8; (FRAME_WIDTH * FRAME_HEIGHT * 3) as usize];
        rgb[0] = 200; // distinctive top-left pixel
        write_playtest_screenshot(&path, &rgb).expect("write png");

        let image = image::open(&path).expect("readable png").to_rgb8();
        assert_eq!(image.dimensions(), (FRAME_WIDTH, FRAME_HEIGHT));
        assert_eq!(image.get_pixel(0, 0).0, [200, 0, 0]);
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
