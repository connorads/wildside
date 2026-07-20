#!/usr/bin/env python3
"""Downward-only failing-test ratchet for the ungated `wildside-cli` suite (#417).

`verify-slice-fast` gates only `wildside-core --lib` + `wildside-cli
gameplay_regression_*`, and there is no CI. The rest of the `wildside-cli` test
suite (`native_player`, `inspect`, `sweep`, `rust_source_context`) is manual-only
via `mise run test`, so regressions there rotted silently into a large committed
failure set. This ratchet stops the bleeding: it pins the *named set* of currently
failing test IDs at a baseline that may only ever shrink.

- A newly failing test (one not in the baseline) trips the gate — fix it, or, if
  it is a test written ahead of implementation, mark it `#[ignore]` with a
  re-derivation note per AGENTS.md. The set only shrinks.
- Fixing a grandfathered test removes it from the current set; `--update` lowers
  the baseline to match. It never grows the set.

A *named set* (not a bare count, unlike `dead_code_ratchet.py`) is deliberate:
grandfathered tests stay auditable, and a fix-one/break-another swap cannot mask
itself behind a stable count.

Modes:
  (default, check): exit non-zero iff any current failure is outside the baseline;
                    otherwise report the subset and exit 0 (an improvement is not
                    a failure).
  --update (fix):   drop fixed tests from the baseline when the set has shrunk;
                    a new regression (outside baseline) still exits non-zero so
                    the pre-commit `fix` step blocks the commit.

The hk pre-commit `failing-test-ratchet` step runs `--update` as its `fix` with
`check_first = false`, so a fix re-stages the shrunk baseline automatically
(precedent: `dead-code-ratchet`). It is its own step, not part of
verify-slice-fast.

Asset-absent machines (fresh clones without `mise run extract-assets`) can't
produce a stable failure set — the suite guards on the extracted asset bundle —
so the ratchet skips (returns 0, never shrinks) when no `.cache/generated/*`
bundle exists. This mirrors `check-hooks.sh` skipping under CI.
"""

from __future__ import annotations

import re
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
BASELINE = ROOT / "docs/source-port/failing-test-baseline.yaml"
GENERATED = ROOT / ".cache/generated"

# The `tests:` list region of the YAML baseline; rewritten in place so the header
# comment survives a ratchet-down (mirrors dead_code_ratchet's MAX_COUNT_RE).
TESTS_BLOCK_RE = re.compile(r"^tests:\n(?P<body>(?:[ \t]*-[ \t].*\n?)*)", re.MULTILINE)
BASELINE_ITEM_RE = re.compile(r"^[ \t]*-[ \t]+(?P<id>\S+)\s*$", re.MULTILINE)


def current_failures() -> set[str]:
    """Run the `wildside-cli` suite and parse the final `failures:` summary block.

    libtest emits `failures:` twice: first before the per-test stdout dumps, then
    again as the flat, alphabetically-sorted list of `    <test_id>` lines right
    before the `test result:` line. We parse the *last* `failures:` block — the
    flat list — which is deterministic regardless of parallel interleaving (the
    inline `test NAME ... FAILED` lines are not, and vary by verbosity).

    `wildside-cli` builds exactly one test binary (`unittests src/main.rs`), so
    there is a single `test result:` line and the parse is unambiguous.
    """
    proc = subprocess.run(
        ["cargo", "test", "-p", "wildside-cli", "--no-fail-fast"],
        cwd=ROOT,
        capture_output=True,
        text=True,
    )
    # cargo prints test output to stdout; captured-output ordering is preserved.
    lines = proc.stdout.splitlines()
    last_header = None
    for index, line in enumerate(lines):
        if line == "failures:":
            last_header = index
    if last_header is None:
        return set()

    failures: set[str] = set()
    for line in lines[last_header + 1 :]:
        if line.startswith("test result:"):
            break
        stripped = line.strip()
        if stripped:
            failures.add(stripped)
    return failures


def read_baseline() -> set[str]:
    text = BASELINE.read_text()
    match = TESTS_BLOCK_RE.search(text)
    if not match:
        print(f"error: {BASELINE} has no `tests:` list", file=sys.stderr)
        raise SystemExit(2)
    return {m.group("id") for m in BASELINE_ITEM_RE.finditer(match.group("body"))}


def write_baseline(tests: set[str]) -> None:
    text = BASELINE.read_text()
    body = "".join(f"  - {test}\n" for test in sorted(tests))
    new_text = TESTS_BLOCK_RE.sub(lambda _m: f"tests:\n{body}", text, count=1)
    BASELINE.write_text(new_text)


def assets_present() -> bool:
    return any(GENERATED.glob("*/asset-manifest.json"))


def main(argv: list[str]) -> int:
    update = "--update" in argv[1:]

    if not assets_present():
        print(
            "failing-test ratchet: skipped (no `.cache/generated/*` asset bundle; "
            "run `mise run extract-assets`). The suite guards on the bundle, so no "
            "stable failure set exists here.",
        )
        return 0

    current = current_failures()
    baseline = read_baseline()

    new_failures = current - baseline
    if new_failures:
        print(
            f"error: {len(new_failures)} wildside-cli test(s) fail outside the "
            f"reviewed baseline. Fix them, or `#[ignore]` a test written ahead of "
            f"implementation with a re-derivation note (AGENTS.md). The ratchet "
            f"only shrinks:",
            file=sys.stderr,
        )
        for test in sorted(new_failures):
            print(f"    {test}", file=sys.stderr)
        return 1

    fixed = baseline - current
    if update and fixed:
        write_baseline(current)
        print(f"failing-test ratchet lowered: {len(baseline)} -> {len(current)}")
        return 0

    print(f"failing tests: {len(current)} (subset of baseline {len(baseline)})")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
