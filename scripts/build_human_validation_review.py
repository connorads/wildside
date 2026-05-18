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
class ReviewItem:
    backlog_row: str
    title: str
    verifier: str
    decision: str
    images: tuple[ReviewImage, ...]
    transcripts: tuple[Path, ...] = ()
    note: str | None = None


def repo_path(path: str) -> Path:
    return REPO_ROOT / path


REVIEW_ITEMS = (
    ReviewItem(
        backlog_row="#135",
        title="Level 1 background",
        verifier=(
            "cargo run -p kidcli -- verify-recording "
            "--slug issue-135-level1-background --native-only"
        ),
        decision=(
            "Settled Blue Lake Woods 1 background matches the original game "
            "closely enough for the current 1P phase."
        ),
        images=(
            ReviewImage(
                repo_path(
                    ".cache/recordings/issue-135-level1-background/verify/native/"
                    "000000_start.png"
                ),
                "Settled start frame used for background validation",
            ),
        ),
        transcripts=(
            repo_path(".cache/recordings/issue-135-level1-background/transcript.txt"),
        ),
    ),
    ReviewItem(
        backlog_row="#136",
        title="Prize emergence",
        verifier=(
            "cargo run -p kidcli -- verify-recording "
            "--slug issue-136-prize-emergence --native-only"
        ),
        decision=(
            "Diamond/helmet emergence uses the expected item graphics and does "
            "not show the old incorrect block/item art."
        ),
        images=(
            ReviewImage(
                repo_path(
                    ".cache/recordings/issue-136-prize-emergence/verify/native/"
                    "000150_post_first_jump.png"
                ),
                "After first prize-block jump",
            ),
            ReviewImage(
                repo_path(
                    ".cache/recordings/issue-136-prize-emergence/verify/native/"
                    "000270_post_second_jump.png"
                ),
                "After second prize-block jump",
            ),
            ReviewImage(
                repo_path(
                    ".cache/recordings/issue-136-prize-emergence/verify/native/"
                    "000359_scenario_end.png"
                ),
                "Scenario end with spawned prize",
            ),
        ),
        transcripts=(
            repo_path(".cache/recordings/issue-136-prize-emergence/transcript.txt"),
        ),
    ),
    ReviewItem(
        backlog_row="#138",
        title="Enemy contact re-hit window",
        verifier=(
            "cargo run -p kidcli -- verify-recording "
            "--slug issue-138-enemy-contact --native-only"
        ),
        decision=(
            "Two damage events 66 frames apart are accepted as ROM-faithful "
            "behaviour rather than a native double-damage bug."
        ),
        images=(
            ReviewImage(
                repo_path(
                    ".cache/recordings/issue-138-enemy-contact/verify/native/"
                    "000300_frame_000300.png"
                ),
                "After first damage event",
            ),
            ReviewImage(
                repo_path(
                    ".cache/recordings/issue-138-enemy-contact/verify/native/"
                    "000360_frame_000360.png"
                ),
                "Just after second damage event",
            ),
        ),
        transcripts=(repo_path(".cache/recordings/issue-138-enemy-contact/transcript.txt"),),
    ),
    ReviewItem(
        backlog_row="#139",
        title="Rubber block art",
        verifier=(
            "cargo run -p kidcli -- verify-recording "
            "--slug issue-139-rubber-block-window --native-only"
        ),
        decision="Visible rubber blocks use rubber-block art, not spike art.",
        images=(
            ReviewImage(
                repo_path(
                    ".cache/recordings/issue-139-rubber-block-window/verify/native/"
                    "000699_walk_end.png"
                ),
                "Rubber block window at walk end",
            ),
        ),
        transcripts=(
            repo_path(".cache/recordings/issue-139-rubber-block-window/transcript.txt"),
        ),
    ),
    ReviewItem(
        backlog_row="#140",
        title="Prize block lifecycle art",
        verifier=(
            "cargo run -p kidcli -- verify-recording "
            "--slug issue-140-prize-block-window --native-only"
        ),
        decision=(
            "Prize block visible/disguised/deleted states use the expected "
            "block art during the hit lifecycle."
        ),
        images=(
            ReviewImage(
                repo_path(
                    ".cache/recordings/issue-140-prize-block-window/verify/native/"
                    "000150_post_first_jump.png"
                ),
                "Post first jump",
            ),
            ReviewImage(
                repo_path(
                    ".cache/recordings/issue-140-prize-block-window/verify/native/"
                    "000239_pre_second_jump.png"
                ),
                "Before second jump",
            ),
            ReviewImage(
                repo_path(
                    ".cache/recordings/issue-140-prize-block-window/verify/native/"
                    "000359_scenario_end.png"
                ),
                "Scenario end",
            ),
        ),
        transcripts=(
            repo_path(".cache/recordings/issue-140-prize-block-window/transcript.txt"),
        ),
    ),
    ReviewItem(
        backlog_row="#143",
        title="Heavy Man / Iron Knight block break",
        verifier=(
            "cargo run -p kidcli -- verify-recording "
            "--slug issue-143-level27-ik-smash-probe-5 --native-only"
        ),
        decision=(
            "Accept the scripted Iron Knight route as sufficient playability "
            "evidence, or record a preferred live Heavy Man route."
        ),
        images=(
            ReviewImage(
                repo_path(
                    ".cache/recordings/issue-143-level27-ik-smash-probe-5/verify/native/"
                    "001100_smash_window.png"
                ),
                "Iron Knight block-break route checkpoint",
            ),
        ),
        transcripts=(
            repo_path(".cache/recordings/issue-143-level27-ik-smash-probe-5/transcript.txt"),
        ),
        note=(
            "Trace evidence: Iron Knight is collected at frame 171 and block "
            "destroy effects are emitted from frame 655 onward."
        ),
    ),
    ReviewItem(
        backlog_row="#146",
        title="Level 2 death/loading hang",
        verifier=(
            "cargo run -q -p kidcli -- verify-recording "
            "--slug issue-146-level2-death --native-only"
        ),
        decision=(
            "Accept the scripted Level 2 death transition as non-blocking for "
            "the current 1P phase, or provide a different Level 2 death route "
            "that still hangs."
        ),
        images=(
            ReviewImage(
                repo_path(
                    ".cache/recordings/issue-146-level2-death/verify/native/"
                    "001859_stomp_end.png"
                ),
                "Scripted Level 2 death transition checkpoint",
            ),
            ReviewImage(
                repo_path(
                    ".cache/recordings/issue-146-level2-stomp-long-probe/verify/native/"
                    "009059_stomp_end.png"
                ),
                "Long Level 2 death probe recovery checkpoint",
            ),
        ),
        transcripts=(
            repo_path(".cache/recordings/issue-146-level2-death/transcript.txt"),
            repo_path(".cache/recordings/issue-146-level2-stomp-long-probe/transcript.txt"),
        ),
        note=(
            "Scripted route takes damage at frames 814 and 1272, enters "
            "LevelTransition at frame 1347, and the longer 9000-frame probe "
            "recovers to InGame at frame 8589."
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
