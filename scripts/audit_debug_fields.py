#!/usr/bin/env python3

from __future__ import annotations

import json
import re
import subprocess
import sys
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parent.parent
REGISTRY_PATH = REPO_ROOT / "docs/source-port/debug-field-registry.yaml"
FRONTIER_STATE_PATH = REPO_ROOT / "docs/source-port/frontier-state.yaml"
SCAN_GLOBS = (
    "crates/**/*.rs",
    "scripts/**/*.py",
)
KEY_PATTERN = re.compile(
    r"\b(?:debug_fields|gameplay_debug_fields)\.insert\(\s*\"([^\"]+)\"(?:\.into\(\))?"
)


def load_jsonish(path: Path) -> dict:
    try:
        return json.loads(path.read_text(encoding="utf-8"))
    except FileNotFoundError as exc:
        raise SystemExit(f"missing file: {path}") from exc
    except json.JSONDecodeError as exc:
        raise SystemExit(f"invalid JSON-compatible YAML in {path}: {exc}") from exc


def discover_emitted_keys() -> set[str]:
    keys: set[str] = set()
    for pattern in SCAN_GLOBS:
        for path in REPO_ROOT.glob(pattern):
            if not path.is_file():
                continue
            text = path.read_text(encoding="utf-8")
            keys.update(KEY_PATTERN.findall(text))
    return keys


def commit_distance(first_added_commit: str) -> int | None:
    if not re.fullmatch(r"[0-9a-f]{7,40}", first_added_commit):
        return None
    result = subprocess.run(
        ["git", "rev-list", "--count", f"{first_added_commit}..HEAD"],
        cwd=REPO_ROOT,
        capture_output=True,
        text=True,
        check=False,
    )
    if result.returncode != 0:
        return None
    try:
        return int(result.stdout.strip())
    except ValueError:
        return None


def main() -> int:
    registry = load_jsonish(REGISTRY_PATH)
    frontier_state = load_jsonish(FRONTIER_STATE_PATH)
    defaults = registry.get("defaults", {})
    warn_after = int(defaults.get("warn_after_commits", 2))
    fail_after = int(defaults.get("fail_after_commits", 4))
    fields = registry.get("fields", [])
    registered = {field["key"]: field for field in fields}
    emitted = discover_emitted_keys()

    problems: list[str] = []
    warnings: list[str] = []

    unregistered = sorted(emitted - registered.keys())
    if unregistered:
        problems.extend(f"unregistered emitted debug key: {key}" for key in unregistered)

    for key, field in sorted(registered.items()):
        status = field.get("status", "exploratory")
        if status != "candidate":
            continue
        distance = commit_distance(str(field.get("first_added_commit", "")))
        if distance is None:
            warnings.append(
                f"candidate key {key}: age unknown (set first_added_commit after the first commit that carries it)"
            )
            continue
        if distance >= fail_after:
            problems.append(
                f"candidate key {key}: {distance} commits old, promote or add an exemption note"
            )
        elif distance >= warn_after:
            warnings.append(
                f"candidate key {key}: {distance} commits old, consider promotion to typed state"
            )

    candidate_promotions = frontier_state.get("candidate_promotions", [])
    for item in candidate_promotions:
        key = item.get("key")
        if key not in registered:
            problems.append(f"frontier-state candidate promotion references unknown key: {key}")

    print("Debug Field Audit")
    print(f"  emitted keys: {len(emitted)}")
    print(f"  registered keys: {len(registered)}")
    if emitted:
        print("  emitted:")
        for key in sorted(emitted):
            print(f"    - {key}")
    if candidate_promotions:
        print("  frontier candidate promotions:")
        for item in candidate_promotions:
            print(f"    - {item['key']}: {item['reason']}")

    for warning in warnings:
        print(f"warning: {warning}")

    if problems:
        for problem in problems:
            print(f"error: {problem}")
        return 1

    print("  audit: clean")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
