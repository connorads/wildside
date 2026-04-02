#!/usr/bin/env python3

from __future__ import annotations

import json
import subprocess
import sys
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parent.parent
NEXT_TASK_PATH = REPO_ROOT / ".cache/autonomy/source-port-frontier/next-task.json"
NEXT_AGENT_PATH = REPO_ROOT / ".cache/autonomy/source-port-frontier/next-agent.md"


def run_autonomy_loop() -> None:
    result = subprocess.run(
        [
            "cargo",
            "run",
            "-q",
            "-p",
            "kidcli",
            "--",
            "autonomous-loop",
            "--goal",
            "source-port-frontier",
        ],
        cwd=REPO_ROOT,
        check=False,
        capture_output=True,
        text=True,
    )
    if result.returncode != 0:
        sys.stderr.write(result.stdout)
        sys.stderr.write(result.stderr)
        raise SystemExit(result.returncode)


def load_task() -> dict:
    try:
        return json.loads(NEXT_TASK_PATH.read_text(encoding="utf-8"))
    except FileNotFoundError as exc:
        raise SystemExit(f"missing next task package: {NEXT_TASK_PATH}") from exc
    except json.JSONDecodeError as exc:
        raise SystemExit(f"invalid next task package: {exc}") from exc


def bullet_list(items: list[str]) -> str:
    return "\n".join(f"- {item}" for item in items)


def main() -> int:
    run_autonomy_loop()
    task = load_task()
    delegated_goal = task.get("delegated_goal") or "source-port-frontier"
    delegated_target = task.get("delegated_target") or "none"
    commands = task.get("commands") or []
    done_criteria = task.get("done_criteria") or []
    sources = task.get("sources") or []
    implementation_targets = task.get("implementation_targets") or []
    generated_at = task.get("generated_at", "unknown")
    cycle = task.get("cycle", "unknown")
    fingerprint = task.get("progress_fingerprint", "unknown")

    print(
        f"""# Source-Port RL Prompt

Use the live source-port task package for this run. Do not invent a backlog hunt.

## Fresh package

- Regenerated with `cargo run -q -p kidcli -- autonomous-loop --goal source-port-frontier`
- Package: `{NEXT_TASK_PATH.relative_to(REPO_ROOT)}`
- Note: `{NEXT_AGENT_PATH.relative_to(REPO_ROOT)}`
- Generated at: `{generated_at}`
- Cycle: `{cycle}`
- Progress fingerprint: `{fingerprint}`
- Delegated goal: `{delegated_goal}`
- Delegated target: `{delegated_target}`

## Task

- Title: {task["title"]}
- Summary: {task["summary"]}

## Read only these repo artefacts

{bullet_list([str(NEXT_TASK_PATH.relative_to(REPO_ROOT)), str(NEXT_AGENT_PATH.relative_to(REPO_ROOT)), *sources])}

## Commands

{bullet_list(commands)}

## Done criteria

{bullet_list(done_criteria)}

## Implementation targets

{bullet_list(implementation_targets) if implementation_targets else '- None specified; stay inside the delegated lane.'}

## Hard rules

- Work only the delegated lane from the live package above.
- Do not read `docs/source-port/backlog.md`, `docs/source-port/frontier-state.yaml`, or `docs/source-port/tas-roadmap.md` unless this package explicitly lists them in `sources`.
- Do not reopen TAS-frontier work if the live package delegates to `case-queue` or `sweep-parity`.
- If the delegated goal is `sweep-parity`, stay on sweep/release-gate hardening.
- If the delegated goal is `case-queue`, re-verify the selected case before changing code.
- If the package shows no actionable reduced case, do not invent one from stale notes; follow the delegated lane that was generated for this run.
- Use ASM first for gameplay behaviour; use probes only when ASM plus oracle still leave ambiguity.
- Do not make a handoff-only commit.
"""
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
