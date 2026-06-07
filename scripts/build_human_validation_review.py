#!/usr/bin/env python3

from __future__ import annotations

import argparse
import html
import os
from dataclasses import dataclass
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parent.parent
DEFAULT_OUTPUT_DIR = REPO_ROOT / ".cache" / "human-validation"


@dataclass(frozen=True)
class ReviewImage:
    path: Path
    caption: str


@dataclass(frozen=True)
class ReviewAudio:
    path: Path
    caption: str


@dataclass(frozen=True)
class ReviewItem:
    backlog_row: str
    title: str
    verifier: str
    decision: str
    images: tuple[ReviewImage, ...]
    audio: tuple[ReviewAudio, ...] = ()
    transcripts: tuple[Path, ...] = ()
    evidence_files: tuple[Path, ...] = ()
    note: str | None = None


def repo_path(path: str) -> Path:
    return REPO_ROOT / path


# Only include rows that are still open in docs/source-port/backlog.md. Several
# historical validation bundles are stale on current native but their backlog
# rows are closed by stronger tests/evidence, so they must not block this gate.
REVIEW_ITEMS = (
    ReviewItem(
        backlog_row="#238",
        title="Highwater flag transitions",
        verifier=(
            "cargo run -q -p kidcli -- verify-recording "
            "--slug issue-238-highwater-flag-transition-current-20260606; "
            "cargo run -q -p kidcli -- verify-recording "
            "--slug issue-238-highwater-pass2-juggernaut-flag-transition-current-20260606 "
            "--native-only"
        ),
        decision=(
            "Confirm the Highwater flag hand-offs no longer drop Kid into an "
            "instant-death or empty-level state: fresh current recordings reach "
            "Highwater Pass 2 and Under Skull Mountain 1, then survive their "
            "tail checkpoints."
        ),
        images=(
            ReviewImage(
                repo_path(
                    ".cache/recordings/issue-238-highwater-flag-transition-current-20260606/verify/native/000000_flag_touch.png"
                ),
                "Real Highwater Pass 1 flag touch",
            ),
            ReviewImage(
                repo_path(
                    ".cache/recordings/issue-238-highwater-flag-transition-current-20260606/verify/native/001200_transition_window.png"
                ),
                "Title-card transition window after Start",
            ),
            ReviewImage(
                repo_path(
                    ".cache/recordings/issue-238-highwater-flag-transition-current-20260606/verify/native/001800_next_level_entry.png"
                ),
                "Highwater Pass 2 entry after the hand-off",
            ),
            ReviewImage(
                repo_path(
                    ".cache/recordings/issue-238-highwater-flag-transition-current-20260606/verify/native/002399_next_level_survival_tail.png"
                ),
                "Tail checkpoint survives in Highwater Pass 2",
            ),
            ReviewImage(
                repo_path(
                    ".cache/recordings/issue-238-highwater-pass2-juggernaut-flag-transition-current-20260606/verify/native/000000_flag_touch.png"
                ),
                "Juggernaut at the real Highwater Pass 2 flag",
            ),
            ReviewImage(
                repo_path(
                    ".cache/recordings/issue-238-highwater-pass2-juggernaut-flag-transition-current-20260606/verify/native/001800_next_level_entry.png"
                ),
                "Under Skull Mountain 1 entry after Pass 2 hand-off",
            ),
            ReviewImage(
                repo_path(
                    ".cache/recordings/issue-238-highwater-pass2-juggernaut-flag-transition-current-20260606/verify/native/002399_next_level_survival_tail.png"
                ),
                "Tail checkpoint survives in Under Skull Mountain 1",
            ),
        ),
        transcripts=(
            repo_path(
                ".cache/recordings/issue-238-highwater-flag-transition-current-20260606/transcript.txt"
            ),
            repo_path(
                ".cache/recordings/issue-238-highwater-pass2-juggernaut-flag-transition-current-20260606/transcript.txt"
            ),
        ),
        evidence_files=(
            repo_path(
                ".cache/recordings/issue-238-highwater-flag-transition-current-20260606/verify/report.json"
            ),
            repo_path(
                ".cache/recordings/issue-238-highwater-pass2-juggernaut-flag-transition-current-20260606/verify/report.json"
            ),
        ),
        note=(
            "Fresh Pass 1 evidence is trace/oracle-state/oracle-frame clean. "
            "Fresh Pass 2/Juggernaut evidence is trace/oracle-state clean and "
            "native-replay stable; full oracle frame comparison still has visual "
            "pixel drift, so use it as gameplay-transition evidence rather than "
            "pixel-parity closure."
        ),
    ),
    ReviewItem(
        backlog_row="#249",
        title="BLW2 left-wall no-jump probe",
        verifier=(
            "cargo run -q -p kidcli -- verify-recording "
            "--slug issue-249-blw2-left-pit-wall-no-jump-current-20260606 "
            "--native-only; "
            "cargo test -q -p kidcli "
            "issue249_blw2_left_pit_wall_no_jump_does_not_auto_launch_juggernaut "
            "-- --nocapture"
        ),
        decision=(
            "Inspect the reported Blue Lake Woods 2 left-wall contact with no "
            "jump input: current native stays grounded at the wall instead of "
            "auto-launching, but the route still needs a natural repro or richer "
            "oracle-equivalent seed before closing."
        ),
        images=(
            ReviewImage(
                repo_path(
                    ".cache/recordings/issue-249-blw2-left-pit-wall-no-jump-current-20260606/verify/native/000030_left_pit_wall_contact_loc_a8de.png"
                ),
                "First wall-contact checkpoint with no jump input",
            ),
            ReviewImage(
                repo_path(
                    ".cache/recordings/issue-249-blw2-left-pit-wall-no-jump-current-20260606/verify/native/000090_left_pit_wall_hold_loc_aba4.png"
                ),
                "Held against the reported left wall without launch",
            ),
            ReviewImage(
                repo_path(
                    ".cache/recordings/issue-249-blw2-left-pit-wall-no-jump-current-20260606/verify/native/000179_tail.png"
                ),
                "Tail checkpoint remains grounded at the wall",
            ),
        ),
        transcripts=(
            repo_path(
                ".cache/recordings/issue-249-blw2-left-pit-wall-no-jump-current-20260606/transcript.txt"
            ),
        ),
        evidence_files=(
            repo_path(
                ".cache/recordings/issue-249-blw2-left-pit-wall-no-jump-current-20260606/verify/report.json"
            ),
        ),
        note=(
            "This is native-review evidence only. Full oracle verification is "
            "blocked by the direct WRAM start-position seed diverging before "
            "wall logic; see the report for the frame-30 pose mismatch."
        ),
    ),
    ReviewItem(
        backlog_row="#248",
        title="Death respawn Start handoff",
        verifier=(
            "cargo test -q -p kid-core respawn_hold -- --nocapture; "
            "cargo test -q -p kid-core spike_death_triggers_death_and_respawn -- --nocapture; "
            "cargo test -q -p kid-core continue_restores_lives_and_respawns -- --nocapture"
        ),
        decision=(
            "Confirm a non-dragon death no longer feels permanently frozen: "
            "the hold is still black before Start, then gameplay visibly returns "
            "after the observed Start handoff."
        ),
        images=(
            ReviewImage(
                repo_path(
                    ".cache/debug/issue248-start-resume-review/issue-248-death-start-resume_frame_00555.png"
                ),
                "Pre-Start death-respawn hold remains black",
            ),
            ReviewImage(
                repo_path(
                    ".cache/debug/issue248-start-resume-review/issue-248-death-start-resume_frame_00561.png"
                ),
                "Second Start-held hidden frame is still black",
            ),
            ReviewImage(
                repo_path(
                    ".cache/debug/issue248-start-resume-review/issue-248-death-start-resume_frame_00563.png"
                ),
                "Gameplay returns after the Start handoff",
            ),
            ReviewImage(
                repo_path(
                    ".cache/debug/issue248-start-resume-review/issue-248-death-start-resume_frame_00575.png"
                ),
                "Level remains restored with Kid alive",
            ),
        ),
        evidence_files=(
            repo_path(".cache/recordings/issue-248-death-start-resume-settled.json"),
        ),
        note=(
            "Review artefacts are regenerated by `mise run human-validation-review`. "
            "Current trace probe shows frames 560-562 remain in LevelTransition and "
            "frame 563 returns to InGame; remaining #248 debt is title-card art "
            "during the hold plus no-input auto-timeout derivation."
        ),
    ),
    ReviewItem(
        backlog_row="#210",
        title="Intro title and menu return",
        verifier=(
            "cargo run -q -p kidcli -- verify-recording "
            "--slug issue-210-title-menu-tail"
        ),
        decision=(
            "Inspect whether the Start-skipped intro/title path shows readable "
            "title and menu graphics without the earlier green shell or demo "
            "re-entry corruption."
        ),
        images=(
            ReviewImage(
                repo_path(
                    ".cache/recordings/issue-210-title-menu-tail/verify/native/002599_pre_start.png"
                ),
                "IntroSequence6 just before pressing Start",
            ),
            ReviewImage(
                repo_path(
                    ".cache/recordings/issue-210-title-menu-tail/verify/native/002700_menu_wait.png"
                ),
                "Title art restored while waiting for the second Start",
            ),
            ReviewImage(
                repo_path(
                    ".cache/recordings/issue-210-title-menu-tail/verify/native/002720_menu_visible.png"
                ),
                "Readable title menu after the second Start",
            ),
            ReviewImage(
                repo_path(
                    ".cache/recordings/issue-210-title-menu-tail/verify/native/003299_tail.png"
                ),
                "Tail checkpoint remains on the title/menu path",
            ),
        ),
        transcripts=(
            repo_path(".cache/recordings/issue-210-title-menu-tail/transcript.txt"),
        ),
        evidence_files=(
            repo_path(".cache/recordings/issue-210-title-menu-tail/verify/report.json"),
        ),
        note=(
            "Mechanical recheck on 2026-06-06: trace, oracle state, and oracle "
            "frame comparison all pass for this focused Start/menu slice; the "
            "remaining #210 decision is whether a broader manual route still "
            "reproduces the playtest symptoms."
        ),
    ),
    ReviewItem(
        backlog_row="#160",
        title="Native music and sound effects",
        verifier=(
            "cargo run -q -p kidcli -- verify-recording "
            "--slug issue-160-audio-latest-validation --native-only"
        ),
        decision=(
            "Listen for audible native gameplay music/SFX without the title "
            "music carrying under gameplay."
        ),
        images=(),
        audio=(
            ReviewAudio(
                repo_path(
                    ".cache/recordings/issue-160-audio-latest-validation/audio.wav"
                ),
                "Fresh native Level 00 audio capture after the GEMS reset fix",
            ),
        ),
        transcripts=(
            repo_path(".cache/recordings/issue-160-audio-latest-validation/transcript.txt"),
        ),
        evidence_files=(
            repo_path(
                ".cache/recordings/issue-160-audio-latest-validation/audio-events.txt"
            ),
            repo_path(
                ".cache/recordings/issue-160-audio-latest-validation/audio-diagnostics.txt"
            ),
        ),
        note=(
            "Mechanical check on 2026-06-06: the WAV is 10.00s stereo at "
            "44.1 kHz, and the live SDL/GEMS diagnostics report 600 generated "
            "and queued frames, no queue errors, and no skipped full frames."
        ),
    ),
)


def rel_from_output(path: Path, output_dir: Path) -> str:
    return html.escape(os.path.relpath(path, output_dir))


def rel_from_repo(path: Path) -> str:
    return html.escape(str(path.relative_to(REPO_ROOT)))


def validate_inputs() -> list[Path]:
    missing: list[Path] = []
    for item in REVIEW_ITEMS:
        for image in item.images:
            if not image.path.is_file():
                missing.append(image.path)
        for audio in item.audio:
            if not audio.path.is_file():
                missing.append(audio.path)
        for transcript in item.transcripts:
            if not transcript.is_file():
                missing.append(transcript)
        for evidence_file in item.evidence_files:
            if not evidence_file.is_file():
                missing.append(evidence_file)
    return missing


def render_item(item: ReviewItem, output_dir: Path) -> str:
    image_html = []
    for image in item.images:
        src = rel_from_output(image.path, output_dir)
        image_html.append(
            "<figure>"
            f'<a href="{src}"><img src="{src}" alt="{html.escape(image.caption)}"></a>'
            f"<figcaption>{html.escape(image.caption)}"
            f"<br><code>{rel_from_repo(image.path)}</code></figcaption>"
            "</figure>"
        )

    audio_html = []
    for audio in item.audio:
        src = rel_from_output(audio.path, output_dir)
        audio_html.append(
            "<figure>"
            f'<audio controls preload="metadata" src="{src}"></audio>'
            f"<figcaption>{html.escape(audio.caption)}"
            f"<br><code>{rel_from_repo(audio.path)}</code></figcaption>"
            "</figure>"
        )

    transcript_links = []
    for transcript in item.transcripts:
        href = rel_from_output(transcript, output_dir)
        transcript_links.append(
            f'<li><a href="{href}"><code>{rel_from_repo(transcript)}</code></a></li>'
        )

    evidence_links = []
    for evidence_file in item.evidence_files:
        href = rel_from_output(evidence_file, output_dir)
        evidence_links.append(
            f'<li><a href="{href}"><code>{rel_from_repo(evidence_file)}</code></a></li>'
        )

    note = f"<p><strong>Note:</strong> {html.escape(item.note)}</p>" if item.note else ""
    transcripts = (
        f"<h3>Transcript</h3><ul>{''.join(transcript_links)}</ul>"
        if transcript_links
        else ""
    )
    evidence = (
        f"<h3>Evidence files</h3><ul>{''.join(evidence_links)}</ul>"
        if evidence_links
        else ""
    )

    return f"""
<section>
  <h2>{html.escape(item.backlog_row)} {html.escape(item.title)}</h2>
  <p><strong>Verifier:</strong> <code>{html.escape(item.verifier)}</code></p>
  <p><strong>Decision needed:</strong> {html.escape(item.decision)}</p>
  {note}
  <div class="images">
    {''.join(image_html)}
  </div>
  <div class="audio">
    {''.join(audio_html)}
  </div>
  {transcripts}
  {evidence}
</section>
"""


def render_html(output_dir: Path) -> str:
    items_html = "\n".join(render_item(item, output_dir) for item in REVIEW_ITEMS)
    return f"""<!doctype html>
<html lang="en">
<head>
  <meta charset="utf-8">
  <title>Kid Chameleon Human Validation Review</title>
  <style>
    body {{
      color: #1d2327;
      font-family: system-ui, -apple-system, BlinkMacSystemFont, "Segoe UI", sans-serif;
      line-height: 1.45;
      margin: 32px auto;
      max-width: 1180px;
      padding: 0 24px 48px;
    }}
    h1, h2, h3 {{ line-height: 1.2; }}
    section {{
      border-top: 1px solid #c8d0d8;
      margin-top: 28px;
      padding-top: 20px;
    }}
    code {{
      background: #eef2f5;
      border-radius: 4px;
      padding: 1px 4px;
    }}
    .images {{
      display: grid;
      gap: 16px;
      grid-template-columns: repeat(auto-fit, minmax(300px, 1fr));
      margin: 18px 0;
    }}
    figure {{ margin: 0; }}
    audio {{ width: 100%; }}
    img {{
      border: 1px solid #9aa7b1;
      image-rendering: pixelated;
      max-width: 100%;
      width: 100%;
    }}
    figcaption {{
      color: #4d5b66;
      font-size: 0.92rem;
      margin-top: 6px;
    }}
  </style>
</head>
<body>
  <h1>Kid Chameleon Human Validation Review</h1>
  <p>Generated from tracked review definitions. Run <code>mise run verify-human-validation</code> before using this page so every linked bundle has a fresh native replay.</p>
  <p>This page is for native replay determinism and human judgement only. It is not an oracle-parity report; settled-recording oracle bootstrap limitations make the remaining rows unsuitable for automatic closure.</p>
  <p>Output: <code>{html.escape(str(output_dir.relative_to(REPO_ROOT)))}/index.html</code></p>
  {items_html}
</body>
</html>
"""


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Build an HTML review index for remaining human validation bundles."
    )
    parser.add_argument(
        "--output-dir",
        type=Path,
        default=DEFAULT_OUTPUT_DIR,
        help="directory for index.html (default: .cache/human-validation)",
    )
    return parser.parse_args()


def main() -> None:
    args = parse_args()
    output_dir = args.output_dir
    if not output_dir.is_absolute():
        output_dir = REPO_ROOT / output_dir

    missing = validate_inputs()
    if missing:
        print("missing required review artefacts:")
        for path in missing:
            print(f"  - {rel_from_repo(path)}")
        raise SystemExit(1)

    output_dir.mkdir(parents=True, exist_ok=True)
    index_path = output_dir / "index.html"
    index_path.write_text(render_html(output_dir), encoding="utf-8")
    print(f"wrote {rel_from_repo(index_path)}")


if __name__ == "__main__":
    main()
