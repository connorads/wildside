#!/usr/bin/env python3
"""Downward-only named failure ratchet for the five non-green workspace suites (#445)."""

from __future__ import annotations

import re
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
BASELINE = ROOT / "docs/source-port/workspace-failing-test-baseline.yaml"
GENERATED = ROOT / ".cache/generated"
PACKAGES = (
    "wildside-md",
    "wildside-frame",
    "wildside-data",
    "wildside-render",
    "wildside-web",
)
TESTS_BLOCK_RE = re.compile(r"^tests:\n(?P<body>(?:[ \t]*-[ \t].*\n?)*)", re.MULTILINE)
BASELINE_ITEM_RE = re.compile(r"^[ \t]*-[ \t]+(?P<id>\S+)\s*$", re.MULTILINE)


def parse_libtest_failures(package: str, stdout: str) -> set[str] | None:
    """Parse all completed libtest targets, using each target's final flat failure list."""
    lines = stdout.splitlines()
    result_indexes = [i for i, line in enumerate(lines) if line.startswith("test result:")]
    if not result_indexes:
        return None

    failures: set[str] = set()
    start = 0
    for result_index in result_indexes:
        segment = lines[start:result_index]
        headers = [i for i, line in enumerate(segment) if line == "failures:"]
        if headers:
            for line in segment[headers[-1] + 1 :]:
                test_id = line.strip()
                if test_id:
                    failures.add(f"{package}::{test_id}")
        start = result_index + 1
    return failures


def current_failures() -> set[str] | None:
    failures: set[str] = set()
    for package in PACKAGES:
        proc = subprocess.run(
            ["cargo", "test", "-p", package, "--no-fail-fast"],
            cwd=ROOT,
            capture_output=True,
            text=True,
        )
        if proc.returncode not in (0, 101):
            print(f"{package}: cargo test exit status {proc.returncode}", file=sys.stderr)
            return None
        parsed = parse_libtest_failures(package, proc.stdout)
        if parsed is None:
            print(f"{package}: no completed `test result:` target", file=sys.stderr)
            for line in proc.stderr.splitlines()[-20:]:
                print(f"    {line}", file=sys.stderr)
            return None
        failures.update(parsed)
    return failures


def read_baseline() -> set[str]:
    match = TESTS_BLOCK_RE.search(BASELINE.read_text())
    if not match:
        raise SystemExit(f"error: {BASELINE} has no `tests:` list")
    return {m.group("id") for m in BASELINE_ITEM_RE.finditer(match.group("body"))}


def write_baseline(tests: set[str]) -> None:
    text = BASELINE.read_text()
    body = "".join(f"  - {test}\n" for test in sorted(tests))
    BASELINE.write_text(TESTS_BLOCK_RE.sub(lambda _m: f"tests:\n{body}", text, count=1))


def main(argv: list[str]) -> int:
    if not any(GENERATED.glob("*/asset-manifest.json")):
        print("workspace failing-test ratchet: skipped (no generated asset bundle)")
        return 0

    current = current_failures()
    if current is None:
        print("error: a workspace test suite did not run; refusing to compare", file=sys.stderr)
        return 1
    baseline = read_baseline()
    novel = current - baseline
    if novel:
        print("error: workspace tests fail outside the reviewed baseline:", file=sys.stderr)
        for test in sorted(novel):
            print(f"    {test}", file=sys.stderr)
        return 1

    fixed = baseline - current
    if "--update" in argv[1:] and fixed:
        write_baseline(current)
        print(f"workspace failing-test ratchet lowered: {len(baseline)} -> {len(current)}")
    else:
        print(f"workspace failing tests: {len(current)} (subset of baseline {len(baseline)})")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
