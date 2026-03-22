#!/usr/bin/env python3

from __future__ import annotations

import json
import sys
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parent.parent
STATE_PATH = REPO_ROOT / "docs/source-port/frontier-state.yaml"


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


def main() -> int:
    path = Path(sys.argv[1]).resolve() if len(sys.argv) > 1 else STATE_PATH
    state = load_state(path)
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

    print("Verification Commands")
    print_list(frontier["verification_commands"])
    print()

    print("Bootstrap")
    print("  read first:")
    print_list(bootstrap["read_first"], indent="    - ")
    print("  commands:")
    print_list(bootstrap["commands"], indent="    - ")
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
