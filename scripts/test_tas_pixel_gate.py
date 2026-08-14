#!/usr/bin/env python3
"""Prove the real TAS pixel ratchet rejects a deliberate regression (#446)."""

from __future__ import annotations

import copy
import json
import subprocess
import tempfile
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
SLUG = "any-percent-published"


def run_gate(report: Path, frontier: Path) -> subprocess.CompletedProcess[str]:
    return subprocess.run(
        [
            "cargo",
            "run",
            "-q",
            "-p",
            "wildside-cli",
            "--",
            "tas-pixel-baseline",
            "--report",
            str(report),
            "--frontier",
            str(frontier),
            "--slug",
            SLUG,
        ],
        cwd=ROOT,
        text=True,
        capture_output=True,
    )


def report_for(baseline: dict[str, object], total_frames: int) -> dict[str, object]:
    return {
        "pixel_comparison": {
            "projection": "source_port_visible_rgb24_v1",
            "frame_counts": {
                "requested": total_frames,
                "native": total_frames,
                "oracle": total_frames,
                "compared": total_frames,
            },
            "proves_every_frame_in_range": True,
            **baseline,
        }
    }


def main() -> int:
    frontier_source = ROOT / "tests/tas_frontier.json"
    frontier_payload = json.loads(frontier_source.read_text())
    recording = frontier_payload["recordings"][SLUG]
    baseline = recording["full_expected_pixel_comparison"]
    total_frames = recording["total_frames"]

    with tempfile.TemporaryDirectory(prefix="wildside-pixel-gate-") as temp:
        directory = Path(temp)
        frontier = directory / "tas_frontier.json"
        frontier.write_bytes(frontier_source.read_bytes())
        original_frontier = frontier.read_bytes()
        report = directory / "report.json"

        report.write_text(json.dumps(report_for(baseline, total_frames)))
        green = run_gate(report, frontier)
        if green.returncode != 0 or "tas-pixel-baseline: ok" not in green.stdout:
            raise RuntimeError(
                f"pixel gate did not accept its baseline ({green.returncode}):\n{green.stdout}\n{green.stderr}"
            )

        regressed = copy.deepcopy(baseline)
        regressed["total_changed_pixels"] += 1
        report.write_text(json.dumps(report_for(regressed, total_frames)))
        red = run_gate(report, frontier)
        if red.returncode != 1 or "FAIL — REGRESSION" not in red.stdout:
            raise RuntimeError(
                f"pixel gate did not reject deliberate perturbation ({red.returncode}):\n{red.stdout}\n{red.stderr}"
            )
        if frontier.read_bytes() != original_frontier:
            raise RuntimeError("read-only pixel gate mutated the frontier fixture")

    print("TAS pixel gate self-check: baseline green; +1 changed pixel rejected")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
