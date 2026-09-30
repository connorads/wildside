#!/usr/bin/env python3
"""Fail when a crate clippy.toml drifts from the workspace root clippy.toml.

Clippy reads the nearest clippy.toml and does not merge it with a parent, so a
crate-level file (wildside-core and wildside-render carry purity bans) silently
drops every root key it does not repeat. This gate requires each crate file to
repeat every root key with the same value; crate-only keys are allowed.

Usage: check_clippy_conf_sync.py [repo-root] (the hk `clippy-conf-sync` step).
Tests: scripts/test_check_clippy_conf_sync.py (script-tests discovery task).
"""

from __future__ import annotations

import sys
import tomllib
from pathlib import Path


def drift(root_conf: dict, crate_conf: dict) -> list[str]:
    problems = []
    for key, value in root_conf.items():
        if key not in crate_conf:
            problems.append(f"missing root key `{key}`")
        elif crate_conf[key] != value:
            problems.append(
                f"`{key}` is {crate_conf[key]!r}, root has {value!r}"
            )
    return problems


def check(repo_root: Path) -> list[str]:
    root_path = repo_root / "clippy.toml"
    root_conf = tomllib.loads(root_path.read_text())
    problems = []
    for crate_path in sorted(repo_root.glob("crates/*/clippy.toml")):
        relative = crate_path.relative_to(repo_root)
        crate_conf = tomllib.loads(crate_path.read_text())
        problems += [f"{relative}: {p}" for p in drift(root_conf, crate_conf)]
    return problems


def main(argv: list[str]) -> int:
    repo_root = Path(argv[1]) if len(argv) > 1 else Path(__file__).resolve().parent.parent
    problems = check(repo_root)
    for problem in problems:
        print(problem, file=sys.stderr)
    if problems:
        print(
            "clippy.toml sync: FAIL (a crate clippy.toml replaces the root file; "
            "repeat every root key there)",
            file=sys.stderr,
        )
        return 1
    print("clippy.toml sync: ok")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
