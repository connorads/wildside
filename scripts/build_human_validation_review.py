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
    note: str | None = None


def repo_path(path: str) -> Path:
    return REPO_ROOT / path


# Only include rows that are still open in docs/source-port/backlog.md. Several
# historical validation bundles are stale on current native but their backlog
# rows are closed by stronger tests/evidence, so they must not block this gate.
REVIEW_ITEMS = (
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
        note=(
            "Mechanical check on 2026-05-25: the WAV is 10.00s stereo at "
            "44.1 kHz, RMS 0.052095, peak 0.367125, with all 100 0.1s "
            "windows above 0.001 RMS."
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

    note = f"<p><strong>Note:</strong> {html.escape(item.note)}</p>" if item.note else ""

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
  <h3>Transcript</h3>
  <ul>{''.join(transcript_links)}</ul>
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
