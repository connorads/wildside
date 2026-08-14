#!/usr/bin/env python3
"""Run the fast core suite with a reviewed exact-name exclusion set (#444)."""

from __future__ import annotations

import argparse
import json
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
BASELINE = ROOT / "docs/source-port/core-test-exclusions.json"
LEGACY_FILTERS = ("demo_", "published_tas", "snapshot")


def listed_tests() -> list[str]:
    result = subprocess.run(
        ["cargo", "test", "-q", "-p", "wildside-core", "--lib", "--", "--list"],
        cwd=ROOT,
        check=True,
        text=True,
        capture_output=True,
    )
    return [
        line.removesuffix(": test")
        for line in result.stdout.splitlines()
        if line.endswith(": test")
    ]


def pattern_exclusions(tests: list[str]) -> list[str]:
    return sorted(
        name for name in tests if any(pattern in name for pattern in LEGACY_FILTERS)
    )


def load_baseline() -> list[str]:
    payload = json.loads(BASELINE.read_text())
    exclusions = payload.get("excluded_tests")
    if not isinstance(exclusions, list) or not all(
        isinstance(name, str) for name in exclusions
    ):
        raise ValueError(f"invalid excluded_tests in {BASELINE}")
    if exclusions != sorted(set(exclusions)):
        raise ValueError(f"excluded_tests must be sorted and unique in {BASELINE}")
    return exclusions


def validate_exact_filters(tests: list[str], exclusions: list[str]) -> None:
    for excluded in exclusions:
        matches = [name for name in tests if excluded in name]
        if matches != [excluded]:
            raise ValueError(
                f"libtest filter {excluded!r} matches {matches!r}; exclusions must select exactly one full test name"
            )


def write_baseline(exclusions: list[str]) -> None:
    payload = {
        "schema": "wildside_core_test_exclusions_v1",
        "reason": "Slow demo and published-TAS routes plus three insta snapshots excluded from verify-slice-fast; exact names prevent substring-darkened regressions (#444).",
        "excluded_tests": exclusions,
    }
    BASELINE.write_text(json.dumps(payload, indent=2) + "\n")


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--update", action="store_true")
    args = parser.parse_args()

    tests = listed_tests()
    current = pattern_exclusions(tests)
    if args.update:
        validate_exact_filters(tests, current)
        write_baseline(current)
        print(f"core test exclusions: wrote {len(current)} exact names to {BASELINE}")
        return 0

    reviewed = load_baseline()
    added = sorted(set(current) - set(reviewed))
    stale = sorted(set(reviewed) - set(current))
    if added or stale:
        if added:
            print("new pattern-excluded tests require review:", file=sys.stderr)
            print("\n".join(f"  + {name}" for name in added), file=sys.stderr)
        if stale:
            print("stale reviewed exclusions require removal:", file=sys.stderr)
            print("\n".join(f"  - {name}" for name in stale), file=sys.stderr)
        return 1

    validate_exact_filters(tests, reviewed)
    command = ["cargo", "test", "-q", "-p", "wildside-core", "--lib", "--"]
    for name in reviewed:
        command.extend(("--skip", name))
    print(f"core test exclusions: running with {len(reviewed)} reviewed exact names")
    return subprocess.run(command, cwd=ROOT).returncode


if __name__ == "__main__":
    raise SystemExit(main())
