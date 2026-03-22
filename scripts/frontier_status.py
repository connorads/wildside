#!/usr/bin/env python3

from __future__ import annotations

import json
import sys
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parent.parent
STATE_PATH = REPO_ROOT / "docs/source-port/frontier-state.yaml"
REGISTRY_PATH = REPO_ROOT / "docs/source-port/debug-field-registry.yaml"
SOURCE_PORT_AUTONOMY_DIR = REPO_ROOT / ".cache/autonomy/source-port-frontier"
SOURCE_PORT_NEXT_TASK_PATH = SOURCE_PORT_AUTONOMY_DIR / "next-task.json"
SOURCE_PORT_NEXT_AGENT_NOTE_PATH = SOURCE_PORT_AUTONOMY_DIR / "next-agent.md"


def load_state(path: Path) -> dict:
    try:
        return json.loads(path.read_text(encoding="utf-8"))
    except FileNotFoundError as exc:
        raise SystemExit(f"missing frontier state: {path}") from exc
    except json.JSONDecodeError as exc:
        raise SystemExit(f"invalid frontier state JSON-compatible YAML in {path}: {exc}") from exc


def print_list(items: list[str], indent: str = "  - ") -> None:
    for item in items:
        print(f"{indent}{item}")


def repo_rel(path: Path) -> str:
    try:
        return str(path.relative_to(REPO_ROOT))
    except ValueError:
        return str(path)


def extract_frontmatter_slug(path: Path) -> str | None:
    try:
        contents = path.read_text(encoding="utf-8")
    except FileNotFoundError:
        return None
    lines = iter(contents.splitlines())
    if next(lines, None) != "---":
        return None
    for line in lines:
        if line == "---":
            break
        key, sep, value = line.partition(":")
        if sep and key.strip() == "slug":
            return value.strip()
    return None


def next_agent_note_status(active_slug: str) -> tuple[str, str]:
    if not SOURCE_PORT_NEXT_AGENT_NOTE_PATH.exists():
        return ("missing", "no next-agent note present")
    slug = extract_frontmatter_slug(SOURCE_PORT_NEXT_AGENT_NOTE_PATH)
    if slug is None:
        return ("stale", "note frontmatter is missing `slug`")
    if slug != active_slug:
        return ("stale", f"note slug `{slug}` does not match `{active_slug}`")
    return ("current", f"note slug `{slug}` matches the active frontier")


def main() -> int:
    path = Path(sys.argv[1]).resolve() if len(sys.argv) > 1 else STATE_PATH
    state = load_state(path)
    registry = load_state(REGISTRY_PATH)
    frontier = state["active_frontier"]
    hypothesis = frontier["current_hypothesis"]
    stuck_policy = state["stuck_policy"]
    bootstrap = state["bootstrap"]

    print("Active Frontier")
    print(
        f"  {frontier['recording']} frame {frontier['frontier_frame']} "
        f"({frontier['area']})"
    )
    print(f"  status: {frontier['status']}")
    print(f"  clean through: frame {frontier['authoritative_clean_through']}")
    print()

    print("Blocker")
    print(f"  {frontier['current_blocker']}")
    print()

    print("Current Hypothesis")
    print(f"  {hypothesis['name']}: {hypothesis['reason']}")
    print()

    print("Pre-divergence Windows")
    for window in frontier["pre_divergence_windows"]:
        print(
            f"  frames {window['frame_start']}..{window['frame_end']}: "
            f"{window['reason']}"
        )
    print()

    print("Rejected Hypotheses")
    for item in frontier["rejected_hypotheses"]:
        print(f"  - {item['name']}: {item['reason']}")
    print()

    print("Required Next Evidence")
    print_list(frontier["required_next_evidence"])
    print()

    candidate_promotions = state.get("candidate_promotions", [])
    print("Candidate Promotions")
    if candidate_promotions:
        for item in candidate_promotions:
            print(
                f"  - {item['key']} -> {item['promotion_target']}: {item['reason']}"
            )
    else:
        print("  - none")
    print()

    print("Verification Commands")
    print_list(frontier["verification_commands"])
    print()

    print("Bootstrap")
    print("  read first:")
    print_list(bootstrap["read_first"], indent="    - ")
    print("  commands:")
    print_list(bootstrap["commands"], indent="    - ")
    print()

    note_status, note_detail = next_agent_note_status(frontier["slug"])
    print("Ralph Loop")
    print("  package command:")
    print("    - cargo run -p kidcli -- autonomous-loop --goal source-port-frontier")
    print(f"  package dir: {repo_rel(SOURCE_PORT_AUTONOMY_DIR)}")
    print(f"  next task path: {repo_rel(SOURCE_PORT_NEXT_TASK_PATH)}")
    print(
        "  next-agent note: "
        f"{repo_rel(SOURCE_PORT_NEXT_AGENT_NOTE_PATH)} ({note_status}: {note_detail})"
    )
    print()

    registry_candidates = [
        field for field in registry.get("fields", []) if field.get("status") == "candidate"
    ]
    print("Registry Candidates")
    if registry_candidates:
        for field in registry_candidates:
            target = field.get("promotion_target", "(target not set)")
            print(f"  - {field['key']} -> {target}")
    else:
        print("  - none")
    print()

    print("Stuck Policy")
    print(
        f"  after {stuck_policy['probe_only_commit_threshold']} consecutive "
        "probe/docs-only commits on the same frontier:"
    )
    print(f"  {stuck_policy['rule']}")
    print("  acceptable next moves:")
    print_list(stuck_policy["acceptable_next_moves"], indent="    - ")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
