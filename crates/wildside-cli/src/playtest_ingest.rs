//! playtest-ingest: turn a playtest session (verdicts, bookmarks, mic
//! narration) into a draft markdown report.
//!
//! The report is DRAFT ONLY: it never edits backlog markdown. A follow-up
//! agent session applies lane moves, archive evidence, and new rows per
//! `docs/source-port/loop.md`, keeping backlog-lint and archive conventions
//! with the agent rather than this tool.

use crate::native_player::{PlaytestSessionFile, PlaytestSyncMark, SyncMarkKind, VerdictKind};
use anyhow::{Context, Result, bail};
use std::fmt::Write as _;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

/// Utterances this close (wall-clock) to a verdict/bookmark are attached to it.
const ATTACH_WINDOW_MS: u64 = 10_000;
/// Gameplay frame rate used to extrapolate past the final sync mark.
const FRAMES_PER_SECOND: u64 = 60;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Utterance {
    pub(crate) start_ms: u64,
    pub(crate) end_ms: u64,
    pub(crate) text: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct AlignedUtterance {
    pub(crate) utterance: Utterance,
    /// Wall-clock range (unix ms) the utterance spans.
    pub(crate) wall_start_ms: u64,
    pub(crate) wall_end_ms: u64,
    pub(crate) segment_index: usize,
    pub(crate) frame_index: usize,
}

pub(crate) fn run_playtest_ingest(
    repo_root: &Path,
    session: &str,
    transcript: Option<&Path>,
    whisper_cmd: Option<&str>,
    out: Option<&Path>,
) -> Result<()> {
    let session_dir = resolve_playtest_session_dir(repo_root, session)?;
    let session_json = session_dir.join("session.json");
    let file: PlaytestSessionFile = serde_json::from_slice(
        &fs::read(&session_json)
            .with_context(|| format!("failed to read {}", session_json.display()))?,
    )
    .with_context(|| format!("failed to parse {}", session_json.display()))?;

    let utterances = match (transcript, whisper_cmd) {
        (Some(path), _) => parse_transcript_file(path)?,
        (None, Some(command)) => {
            let mic = file.mic.as_ref().ok_or_else(|| {
                anyhow::anyhow!("session has no mic recording; --whisper-cmd needs mic.wav")
            })?;
            let wav_path = session_dir.join(&mic.wav_path);
            warn_if_wav_unreadable(&wav_path);
            transcribe_with_command(command, &wav_path)?
        }
        (None, None) if file.mic.is_some() => {
            bail!(
                "session has mic narration; pass --transcript <srt|vtt|json> or --whisper-cmd \"<cmd with {{wav}}>\""
            )
        }
        (None, None) => Vec::new(),
    };

    let mic_started_at_unix_ms = file.mic.as_ref().map_or(0, |mic| mic.started_at_unix_ms);
    let aligned = align_utterances(&utterances, &file.sync_marks, mic_started_at_unix_ms);

    let session_dir_display = session_dir
        .strip_prefix(repo_root)
        .unwrap_or(&session_dir)
        .display()
        .to_string();
    let report = render_playtest_report(&file, &session_dir_display, &aligned);
    let out_path = out.map_or_else(|| session_dir.join("report.md"), Path::to_path_buf);
    fs::write(&out_path, &report)
        .with_context(|| format!("failed to write {}", out_path.display()))?;
    println!("draft report written to {}", out_path.display());
    println!("the backlog was NOT modified; hand the report to an agent session to apply it");
    Ok(())
}

pub(crate) fn resolve_playtest_session_dir(repo_root: &Path, session: &str) -> Result<PathBuf> {
    let candidate = PathBuf::from(session);
    let dir = if candidate.is_dir() {
        candidate
    } else {
        repo_root.join(".cache/playtest").join(session)
    };
    if !dir.join("session.json").is_file() {
        bail!(
            "no playtest session at {} (expected session.json)",
            dir.display()
        );
    }
    Ok(dir)
}

fn warn_if_wav_unreadable(wav_path: &Path) {
    if let Err(error) = hound::WavReader::open(wav_path) {
        eprintln!(
            "warning: {} is unreadable or truncated ({error}); transcription may still work",
            wav_path.display()
        );
    }
}

fn transcribe_with_command(template: &str, wav_path: &Path) -> Result<Vec<Utterance>> {
    let wav = wav_path.display().to_string();
    let mut argv: Vec<String> = template
        .split_whitespace()
        .map(|part| part.replace("{wav}", &wav))
        .collect();
    if !template.contains("{wav}") {
        argv.push(wav);
    }
    let Some((program, args)) = argv.split_first() else {
        bail!("--whisper-cmd is empty");
    };
    let output = Command::new(program)
        .args(args)
        .stdin(Stdio::null())
        .stderr(Stdio::inherit())
        .output()
        .with_context(|| format!("failed to run whisper command: {}", argv.join(" ")))?;
    if !output.status.success() {
        bail!("whisper command failed with {}", output.status);
    }
    let text = String::from_utf8(output.stdout).context("whisper output was not UTF-8")?;
    parse_transcript_text(&text)
}

pub(crate) fn parse_transcript_file(path: &Path) -> Result<Vec<Utterance>> {
    let text =
        fs::read_to_string(path).with_context(|| format!("failed to read {}", path.display()))?;
    match path.extension().and_then(|extension| extension.to_str()) {
        Some("srt") => parse_srt(&text),
        Some("vtt") => parse_vtt(&text),
        Some("json") => parse_whisper_json(&text),
        _ => parse_transcript_text(&text),
    }
}

/// Sniff the transcript format: VTT header, JSON brace, or SRT fallback.
pub(crate) fn parse_transcript_text(text: &str) -> Result<Vec<Utterance>> {
    let trimmed = text.trim_start();
    if trimmed.starts_with("WEBVTT") {
        parse_vtt(text)
    } else if trimmed.starts_with('{') || trimmed.starts_with('[') {
        parse_whisper_json(text)
    } else {
        parse_srt(text)
    }
}

pub(crate) fn parse_srt(text: &str) -> Result<Vec<Utterance>> {
    parse_cue_blocks(text, false)
}

pub(crate) fn parse_vtt(text: &str) -> Result<Vec<Utterance>> {
    parse_cue_blocks(text, true)
}

/// SRT and VTT share the same cue shape: an optional index/id line, a
/// `start --> end` line, then text lines until a blank line.
fn parse_cue_blocks(text: &str, skip_webvtt_header: bool) -> Result<Vec<Utterance>> {
    let mut utterances = Vec::new();
    for block in text.replace("\r\n", "\n").split("\n\n") {
        let mut lines = block
            .lines()
            .filter(|line| !line.trim().is_empty())
            .peekable();
        if skip_webvtt_header
            && lines
                .peek()
                .is_some_and(|line| line.trim_start().starts_with("WEBVTT"))
        {
            continue;
        }
        let Some(timing_line) = lines.clone().find(|line| line.contains("-->")) else {
            continue;
        };
        let (start, end) = parse_cue_timing(timing_line)?;
        let cue_text: String = lines
            .skip_while(|line| !line.contains("-->"))
            .skip(1)
            .collect::<Vec<_>>()
            .join(" ");
        let cue_text = cue_text.trim().to_owned();
        if !cue_text.is_empty() {
            utterances.push(Utterance {
                start_ms: start,
                end_ms: end,
                text: cue_text,
            });
        }
    }
    Ok(utterances)
}

fn parse_cue_timing(line: &str) -> Result<(u64, u64)> {
    let mut parts = line.split("-->");
    let start = parts
        .next()
        .map(str::trim)
        .filter(|part| !part.is_empty())
        .with_context(|| format!("cue timing missing start: {line}"))?;
    let end_part = parts
        .next()
        .map(str::trim)
        .with_context(|| format!("cue timing missing end: {line}"))?;
    // VTT cue settings can follow the end timestamp ("... align:start").
    let end = end_part.split_whitespace().next().unwrap_or(end_part);
    Ok((parse_cue_timestamp(start)?, parse_cue_timestamp(end)?))
}

/// `HH:MM:SS,mmm` (SRT), `HH:MM:SS.mmm`, or `MM:SS.mmm` (VTT).
fn parse_cue_timestamp(value: &str) -> Result<u64> {
    let normalised = value.replace(',', ".");
    let fields: Vec<&str> = normalised.split(':').collect();
    let (hours, minutes, seconds) = match fields.as_slice() {
        [hours, minutes, seconds] => (hours.parse::<u64>()?, minutes.parse::<u64>()?, *seconds),
        [minutes, seconds] => (0, minutes.parse::<u64>()?, *seconds),
        _ => bail!("unrecognised cue timestamp: {value}"),
    };
    let seconds_ms = (seconds.parse::<f64>()? * 1000.0).round() as u64;
    Ok(((hours * 60 + minutes) * 60) * 1000 + seconds_ms)
}

/// whisper.cpp `--output-json` (`transcription[].offsets.{from,to}` in ms) or
/// OpenAI/mlx-whisper (`segments[].{start,end}` in seconds).
pub(crate) fn parse_whisper_json(text: &str) -> Result<Vec<Utterance>> {
    let value: serde_json::Value = serde_json::from_str(text).context("invalid whisper JSON")?;
    let segments = value
        .get("transcription")
        .or_else(|| value.get("segments"))
        .and_then(serde_json::Value::as_array)
        .or_else(|| value.as_array())
        .context("whisper JSON has no transcription/segments array")?;

    segments
        .iter()
        .map(|segment| {
            let text = segment
                .get("text")
                .and_then(serde_json::Value::as_str)
                .unwrap_or_default()
                .trim()
                .to_owned();
            let (start_ms, end_ms) = if let Some(offsets) = segment.get("offsets") {
                (
                    offsets.get("from").and_then(serde_json::Value::as_u64),
                    offsets.get("to").and_then(serde_json::Value::as_u64),
                )
            } else {
                let seconds_to_ms = |key: &str| {
                    segment
                        .get(key)
                        .and_then(serde_json::Value::as_f64)
                        .map(|seconds| (seconds * 1000.0).round() as u64)
                };
                (seconds_to_ms("start"), seconds_to_ms("end"))
            };
            match (start_ms, end_ms) {
                (Some(start_ms), Some(end_ms)) => Ok(Utterance {
                    start_ms,
                    end_ms,
                    text,
                }),
                _ => bail!("whisper JSON segment missing timing: {segment}"),
            }
        })
        .collect()
}

/// Map each utterance's midpoint wall-clock time onto (segment, frame) using
/// the session's sync marks. Pause/Resume marks share a frame index, so
/// interpolation across a pause stays clamped to the paused frame and paused
/// wall time never inflates the estimate.
pub(crate) fn align_utterances(
    utterances: &[Utterance],
    sync_marks: &[PlaytestSyncMark],
    mic_started_at_unix_ms: u64,
) -> Vec<AlignedUtterance> {
    utterances
        .iter()
        .map(|utterance| {
            let wall_start_ms = mic_started_at_unix_ms + utterance.start_ms;
            let wall_end_ms = mic_started_at_unix_ms + utterance.end_ms;
            let midpoint = wall_start_ms + (wall_end_ms - wall_start_ms) / 2;
            let (segment_index, frame_index) = locate_frame(sync_marks, midpoint);
            AlignedUtterance {
                utterance: utterance.clone(),
                wall_start_ms,
                wall_end_ms,
                segment_index,
                frame_index,
            }
        })
        .collect()
}

fn locate_frame(sync_marks: &[PlaytestSyncMark], wall_ms: u64) -> (usize, usize) {
    let left = sync_marks
        .iter()
        .rev()
        .find(|mark| mark.wall_clock_ms <= wall_ms);
    let right = sync_marks.iter().find(|mark| mark.wall_clock_ms >= wall_ms);
    match (left, right) {
        (Some(left), Some(right)) if left.segment_index == right.segment_index => {
            let span_ms = right.wall_clock_ms - left.wall_clock_ms;
            if span_ms == 0 {
                return (left.segment_index, left.frame_index);
            }
            let progress = (wall_ms - left.wall_clock_ms) as f64 / span_ms as f64;
            let frames = right.frame_index.saturating_sub(left.frame_index) as f64;
            (
                left.segment_index,
                left.frame_index + (frames * progress).round() as usize,
            )
        }
        // Across a segment boundary: snap to whichever mark is closer.
        (Some(left), Some(right)) => {
            if wall_ms - left.wall_clock_ms <= right.wall_clock_ms - wall_ms {
                (left.segment_index, left.frame_index)
            } else {
                (right.segment_index, right.frame_index)
            }
        }
        // After the last mark: extrapolate at 60fps unless the session ended paused.
        (Some(left), None) => {
            if left.kind == SyncMarkKind::Pause {
                (left.segment_index, left.frame_index)
            } else {
                let extra_frames =
                    ((wall_ms - left.wall_clock_ms) * FRAMES_PER_SECOND / 1000) as usize;
                (left.segment_index, left.frame_index + extra_frames)
            }
        }
        (None, Some(right)) => (right.segment_index, right.frame_index),
        (None, None) => (0, 0),
    }
}

fn attached_narration(
    aligned: &[AlignedUtterance],
    event_at_unix_ms: u64,
) -> Vec<&AlignedUtterance> {
    aligned
        .iter()
        .filter(|utterance| {
            utterance.wall_start_ms <= event_at_unix_ms + ATTACH_WINDOW_MS
                && utterance.wall_end_ms + ATTACH_WINDOW_MS >= event_at_unix_ms
        })
        .collect()
}

fn format_transcript_offset(ms: u64) -> String {
    format!(
        "{:02}:{:02}.{}",
        ms / 60_000,
        (ms / 1000) % 60,
        (ms % 1000) / 100
    )
}

pub(crate) fn render_playtest_report(
    file: &PlaytestSessionFile,
    session_dir_display: &str,
    aligned: &[AlignedUtterance],
) -> String {
    let mut report = String::new();
    let _ = writeln!(report, "# Playtest report: {} (DRAFT)", file.session_slug);
    let _ = writeln!(report);
    let _ = writeln!(
        report,
        "> DRAFT ONLY - the backlog has NOT been modified. A follow-up agent session"
    );
    let _ = writeln!(
        report,
        "> applies lane moves, archive evidence, and new rows per docs/source-port/loop.md."
    );
    let _ = writeln!(report);
    let _ = writeln!(report, "- Session dir: `{session_dir_display}`");
    let _ = writeln!(report, "- ROM: `{}`", file.rom_sha1);
    if let Some(commit) = file.git_commit.as_deref() {
        let dirty = if file.git_dirty == Some(true) {
            " (dirty)"
        } else {
            ""
        };
        let _ = writeln!(report, "- Engine commit: `{commit}`{dirty}");
    }
    let _ = writeln!(
        report,
        "- Segments: {} recorded, {} saved",
        file.segments.len(),
        file.segments.iter().filter(|segment| segment.saved).count()
    );

    let _ = writeln!(report, "\n## Verdicts ({})", file.verdicts.len());
    if file.verdicts.is_empty() {
        let _ = writeln!(report, "\nNone recorded.");
    }
    for verdict in &file.verdicts {
        let (label, suggestion) = match verdict.verdict {
            VerdictKind::ConfirmedFixed => (
                "CONFIRMED FIXED",
                "promote `[?]` -> `[x]`: move the row to backlog-archive.md with this session as closing evidence",
            ),
            VerdictKind::StillBroken => (
                "STILL BROKEN",
                "return the row to the open queue in backlog.md with this session as reproduction evidence",
            ),
            VerdictKind::Partial => (
                "PARTIAL",
                "keep awaiting-human; append partial evidence under the issue's #NNN in backlog-archive.md",
            ),
        };
        let _ = writeln!(report, "\n### {} - {label}", verdict.issue_id);
        let _ = writeln!(report, "- Target: `{}`", verdict.target);
        let _ = writeln!(report, "- Check: {}", verdict.check);
        let _ = writeln!(
            report,
            "- Position: segment {} frame {}",
            verdict.segment_index, verdict.frame_index
        );
        if let Some(segment) = file
            .segments
            .iter()
            .find(|segment| segment.index == verdict.segment_index)
        {
            if segment.saved {
                let _ = writeln!(
                    report,
                    "- Replay: `cargo run -p wildside-cli -- verify-recording --bundle {session_dir_display}/{}`",
                    segment.bundle_dir
                );
            } else {
                let _ = writeln!(report, "- Replay: segment was not saved");
            }
        }
        let _ = writeln!(report, "- Suggested lane move: {suggestion}");
        write_narration(&mut report, aligned, verdict.at_unix_ms);
    }

    let _ = writeln!(
        report,
        "\n## New issues ({} bookmarks)",
        file.bookmarks.len()
    );
    if file.bookmarks.is_empty() {
        let _ = writeln!(report, "\nNone recorded.");
    }
    for bookmark in &file.bookmarks {
        let _ = writeln!(report, "\n### {}", bookmark.draft_id);
        let position = match (bookmark.kid_x, bookmark.kid_y) {
            (Some(x), Some(y)) => format!(" at Kid ({x}, {y})"),
            _ => String::new(),
        };
        let _ = writeln!(
            report,
            "- Level 0x{:02X}{position}, segment {} frame {}",
            bookmark.level, bookmark.segment_index, bookmark.frame_index
        );
        let _ = writeln!(
            report,
            "- Screenshot: `{session_dir_display}/{}`",
            bookmark.screenshot
        );
        if let Some(segment) = file
            .segments
            .iter()
            .find(|segment| segment.index == bookmark.segment_index)
            .filter(|segment| segment.saved)
        {
            let _ = writeln!(
                report,
                "- Replay: `cargo run -p wildside-cli -- verify-recording --bundle {session_dir_display}/{}`",
                segment.bundle_dir
            );
        }
        write_narration(&mut report, aligned, bookmark.at_unix_ms);
        let _ = writeln!(
            report,
            "- Awaiting-human row skeleton (fill #NNN, system, Check, Summary from narration):"
        );
        let _ = writeln!(
            report,
            "  `| NNN | [b] | P2 | <system> | 0x{:02X} | <overlay-safe check sentence> | <summary> | Playtest {} {} frame {}; screenshot {} |`",
            bookmark.level,
            file.session_slug,
            bookmark.draft_id,
            bookmark.frame_index,
            bookmark.screenshot,
        );
    }

    report
}

fn write_narration(report: &mut String, aligned: &[AlignedUtterance], event_at_unix_ms: u64) {
    let narration = attached_narration(aligned, event_at_unix_ms);
    if narration.is_empty() {
        return;
    }
    let _ = writeln!(report, "- Narration:");
    for utterance in narration {
        let _ = writeln!(
            report,
            "  - [{}-{}] (segment {} frame {}) {}",
            format_transcript_offset(utterance.utterance.start_ms),
            format_transcript_offset(utterance.utterance.end_ms),
            utterance.segment_index,
            utterance.frame_index,
            utterance.utterance.text,
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::native_player::PlaytestSessionFile;

    fn mark(
        kind: SyncMarkKind,
        wall_clock_ms: u64,
        segment_index: usize,
        frame_index: usize,
    ) -> PlaytestSyncMark {
        PlaytestSyncMark {
            kind,
            wall_clock_ms,
            segment_index,
            frame_index,
        }
    }

    #[test]
    fn parses_srt_blocks_with_comma_timestamps() {
        let srt = "1\n00:00:01,000 --> 00:00:02,500\nthe dragon still freezes\n\n2\n00:01:00,000 --> 00:01:03,000\nafter the transformation\nit never resumes\n";
        let utterances = parse_srt(srt).expect("parse srt");
        assert_eq!(
            utterances,
            vec![
                Utterance {
                    start_ms: 1000,
                    end_ms: 2500,
                    text: String::from("the dragon still freezes"),
                },
                Utterance {
                    start_ms: 60_000,
                    end_ms: 63_000,
                    text: String::from("after the transformation it never resumes"),
                },
            ]
        );
    }

    #[test]
    fn parses_vtt_with_header_short_timestamps_and_cue_settings() {
        let vtt = "WEBVTT\n\n00:05.000 --> 00:07.200 align:start\nhand grab looks right\n\n00:01:10.000 --> 00:01:11.000\nsecond cue\n";
        let utterances = parse_vtt(vtt).expect("parse vtt");
        assert_eq!(utterances.len(), 2);
        assert_eq!(utterances[0].start_ms, 5000);
        assert_eq!(utterances[0].end_ms, 7200);
        assert_eq!(utterances[0].text, "hand grab looks right");
        assert_eq!(utterances[1].start_ms, 70_000);
    }

    #[test]
    fn parses_whisper_cpp_and_segments_json() {
        let whisper_cpp =
            r#"{"transcription":[{"offsets":{"from":100,"to":900},"text":" hello "}]}"#;
        let utterances = parse_whisper_json(whisper_cpp).expect("whisper.cpp json");
        assert_eq!(
            utterances,
            vec![Utterance {
                start_ms: 100,
                end_ms: 900,
                text: String::from("hello"),
            }]
        );

        let segments = r#"{"segments":[{"start":1.5,"end":2.25,"text":"kid clips"}]}"#;
        let utterances = parse_whisper_json(segments).expect("segments json");
        assert_eq!(utterances[0].start_ms, 1500);
        assert_eq!(utterances[0].end_ms, 2250);
    }

    #[test]
    fn sniffs_transcript_format_from_content() {
        assert_eq!(
            parse_transcript_text("WEBVTT\n\n00:01.000 --> 00:02.000\nx\n")
                .expect("vtt")
                .len(),
            1
        );
        assert_eq!(
            parse_transcript_text("1\n00:00:01,000 --> 00:00:02,000\nx\n")
                .expect("srt")
                .len(),
            1
        );
        assert_eq!(
            parse_transcript_text(r#"{"segments":[{"start":0.0,"end":1.0,"text":"x"}]}"#)
                .expect("json")
                .len(),
            1
        );
    }

    #[test]
    fn alignment_interpolates_and_clamps_across_a_pause_boundary() {
        // Mic starts at wall 100_000. Marks: seg1 starts at 101_000, plays a
        // second to frame 60, pauses at frame 120 for 10s, resumes, plays on.
        let marks = vec![
            mark(SyncMarkKind::SegmentStart, 101_000, 1, 0),
            mark(SyncMarkKind::Interval, 102_000, 1, 60),
            mark(SyncMarkKind::Pause, 103_000, 1, 120),
            mark(SyncMarkKind::Resume, 113_000, 1, 120),
            mark(SyncMarkKind::Interval, 114_000, 1, 180),
        ];
        let utterances = vec![
            // midpoint 101_500 -> halfway between frames 0 and 60
            Utterance {
                start_ms: 1_000,
                end_ms: 2_000,
                text: String::from("early"),
            },
            // midpoint 108_000 -> inside the pause, clamps to frame 120
            Utterance {
                start_ms: 7_000,
                end_ms: 9_000,
                text: String::from("paused note"),
            },
            // midpoint 113_500 -> halfway between resume(120) and 180
            Utterance {
                start_ms: 13_000,
                end_ms: 14_000,
                text: String::from("late"),
            },
            // midpoint 116_000 -> 2s past the last mark, extrapolates at 60fps
            Utterance {
                start_ms: 15_500,
                end_ms: 16_500,
                text: String::from("tail"),
            },
        ];

        let aligned = align_utterances(&utterances, &marks, 100_000);
        let frames: Vec<(usize, usize)> = aligned
            .iter()
            .map(|utterance| (utterance.segment_index, utterance.frame_index))
            .collect();
        assert_eq!(
            frames,
            vec![(1, 30), (1, 120), (1, 150), (1, 300)],
            "pause marks bound interpolation so paused wall time never inflates frames"
        );
    }

    #[test]
    fn alignment_snaps_to_the_nearer_mark_across_segment_boundaries() {
        let marks = vec![
            mark(SyncMarkKind::Interval, 10_000, 1, 600),
            mark(SyncMarkKind::SegmentStart, 20_000, 2, 0),
        ];
        let aligned = align_utterances(
            &[
                Utterance {
                    start_ms: 11_000,
                    end_ms: 13_000,
                    text: String::from("end of seg1"),
                },
                Utterance {
                    start_ms: 18_000,
                    end_ms: 19_000,
                    text: String::from("start of seg2"),
                },
            ],
            &marks,
            0,
        );
        assert_eq!(aligned[0].segment_index, 1);
        assert_eq!(aligned[0].frame_index, 600);
        assert_eq!(aligned[1].segment_index, 2);
        assert_eq!(aligned[1].frame_index, 0);
    }

    #[test]
    fn report_carries_verdicts_bookmarks_and_draft_banner() {
        let file: PlaytestSessionFile = serde_json::from_str(
            r#"{
                "session_slug": "pt-20260712-090807",
                "started_at_unix": 1783847287,
                "rom_sha1": "28b904",
                "git_commit": "deadbeef",
                "git_dirty": false,
                "segments": [
                    {"index": 1, "slug": "pt-20260712-090807-seg01", "bundle_dir": "segments/seg01",
                     "start": {"warp_level": 3, "settled": false, "record_frames": null},
                     "saved": true, "reason": "exit"}
                ],
                "verdicts": [
                    {"issue_id": "238", "target": "0x03",
                     "check": "Dragons resume patrol after the Kid transformation.",
                     "verdict": "StillBroken", "segment_index": 1, "frame_index": 512,
                     "at_unix_ms": 1783847300000, "note": null}
                ],
                "bookmarks": [
                    {"draft_id": "PT-001", "segment_index": 1, "frame_index": 900, "level": 3,
                     "kid_x": 230, "kid_y": 271, "screenshot": "bookmarks/PT-001.png",
                     "at_unix_ms": 1783847310000}
                ],
                "sync_marks": [],
                "mic": null
            }"#,
        )
        .expect("session fixture");
        let aligned = vec![AlignedUtterance {
            utterance: Utterance {
                start_ms: 12_000,
                end_ms: 14_000,
                text: String::from("dragon freezes right here"),
            },
            wall_start_ms: 1_783_847_299_000,
            wall_end_ms: 1_783_847_301_000,
            segment_index: 1,
            frame_index: 500,
        }];

        let report = render_playtest_report(&file, ".cache/playtest/pt-20260712-090807", &aligned);

        assert!(report.contains("DRAFT ONLY - the backlog has NOT been modified"));
        assert!(report.contains("### 238 - STILL BROKEN"));
        assert!(report.contains(
            "verify-recording --bundle .cache/playtest/pt-20260712-090807/segments/seg01"
        ));
        assert!(report.contains("return the row to the open queue"));
        assert!(report.contains("dragon freezes right here"));
        assert!(report.contains("### PT-001"));
        assert!(report.contains("Level 0x03 at Kid (230, 271), segment 1 frame 900"));
        assert!(report.contains("bookmarks/PT-001.png"));
        assert!(report.contains("| NNN | [b] | P2 |"));
    }
}
