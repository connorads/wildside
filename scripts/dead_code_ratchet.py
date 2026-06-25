#!/usr/bin/env python3
"""Downward-only dead-code marker ratchet — the #32 scoreboard.

The `#[cfg_attr(not(test), allow(dead_code))]` markers are not junk and not
missing features: each is a faithful, unit-tested, per-`loc_XXXX` ASM model that
is simply not yet wired into the live path — the "faithful-but-dark shadow".
Incorporating the rest of the port means promoting those models to first-class
(replacing the consolidated/approximate code) and stripping the now-stale marker.
This ratchet is the meter for that work: it pins the marker count at a baseline
that may only ever fall.

- A new bare marker (fresh dark code) raises the count above the baseline and
  trips the gate — wire the model into the live path, or keep a genuinely
  test-only helper behind `#[cfg(test)]` instead.
- Stripping a marker (a model went live, or was test-only) lowers the count;
  `--update` lowers the baseline to match. It never raises it.

The count is the literal attribute across `crates/**/*.rs`, scanned recursively
— critical after the enemies.rs per-family split (#40) scattered dark code into
`crates/wildside-core/src/enemies/<family>.rs`.

Modes:
  (default, check): exit non-zero iff count > baseline; otherwise report
                    `<= baseline` and exit 0 (an improvement is not a failure).
  --update (fix):   lower the baseline to the current count when it has fallen;
                    never raise it. A regression (count > baseline) still exits
                    non-zero so the pre-commit `fix` step blocks the commit.

The hk pre-commit `dead-code-ratchet` step runs `--update` as its `fix` with
`check_first = false`, so an improvement re-stages the lowered baseline
automatically (precedent: cargo-fmt). It is its own step, not part of
verify-slice-fast.
"""

from __future__ import annotations

import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
CRATES = ROOT / "crates"
BASELINE = ROOT / "docs/source-port/dead-code-baseline.yaml"

# Matches the canonical `#[cfg_attr(not(test), allow(dead_code))]` and tolerates
# whitespace drift after the comma. This is the dark-shadow marker only; a plain
# `#[allow(dead_code)]` is a different (broader) suppression and is not counted.
ATTR_RE = re.compile(r"#\[cfg_attr\(not\(test\),\s*allow\(dead_code\)\)\]")
# `max_count:` line in the YAML baseline; rewritten in place so the header
# comment survives a ratchet-down.
MAX_COUNT_RE = re.compile(r"^(?P<pre>\s*max_count\s*:\s*)(?P<n>\d+)(?P<post>.*)$", re.MULTILINE)


def count_markers() -> int:
    total = 0
    for path in sorted(CRATES.rglob("*.rs")):
        total += len(ATTR_RE.findall(path.read_text()))
    return total


def read_baseline() -> int:
    text = BASELINE.read_text()
    match = MAX_COUNT_RE.search(text)
    if not match:
        print(f"error: {BASELINE} has no `max_count:` line", file=sys.stderr)
        raise SystemExit(2)
    return int(match.group("n"))


def write_baseline(new_count: int) -> None:
    text = BASELINE.read_text()
    new_text = MAX_COUNT_RE.sub(rf"\g<pre>{new_count}\g<post>", text, count=1)
    BASELINE.write_text(new_text)


def main(argv: list[str]) -> int:
    update = "--update" in argv[1:]
    count = count_markers()
    baseline = read_baseline()

    if count > baseline:
        print(
            f"error: dead-code markers rose to {count} (baseline {baseline}). "
            "A new `#[cfg_attr(not(test), allow(dead_code))]` is fresh dark code "
            "— wire the model into the live path, or keep a test-only helper "
            "behind `#[cfg(test)]`. The ratchet only falls.",
            file=sys.stderr,
        )
        return 1

    if update and count < baseline:
        write_baseline(count)
        print(f"dead-code ratchet lowered: {baseline} -> {count}")
        return 0

    print(f"dead-code markers: {count} (<= baseline {baseline})")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
