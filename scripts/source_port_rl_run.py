#!/usr/bin/env python3

from __future__ import annotations

import json
import subprocess
import sys
from datetime import UTC, datetime
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parent.parent
PROMPT_SCRIPT = REPO_ROOT / "scripts/source_port_rl_prompt.py"
NEXT_TASK_PATH = REPO_ROOT / ".cache/autonomy/source-port-frontier/next-task.json"
RUN_HISTORY_PATH = REPO_ROOT / ".cache/autonomy/source-port-frontier/run-history.jsonl"


def tracked_dirty_files() -> list[str]:
    result = subprocess.run(
        ["git", "status", "--short", "--untracked-files=no"],
        cwd=REPO_ROOT,
        check=False,
        capture_output=True,
        text=True,
    )
    if result.returncode != 0:
        return []
    files: list[str] = []
    for line in result.stdout.splitlines():
        if len(line) >= 4:
            files.append(line[3:].strip())
    return [path for path in files if path]


def load_task() -> dict:
    return json.loads(NEXT_TASK_PATH.read_text(encoding="utf-8"))


def append_history(task: dict, dirty_files: list[str], outcome: str, exit_code: int) -> None:
    RUN_HISTORY_PATH.parent.mkdir(parents=True, exist_ok=True)
    record = {
        "timestamp": datetime.now(UTC).isoformat(),
        "cycle": task.get("cycle"),
        "delegated_goal": task.get("delegated_goal"),
        "delegated_target": task.get("delegated_target"),
        "progress_fingerprint": task.get("progress_fingerprint"),
        "stalled_repetition_count": task.get("stalled_repetition_count"),
        "recovery_mode": task.get("recovery_mode"),
        "recovery_step": task.get("recovery_step"),
        "dirty_tree": bool(dirty_files),
        "dirty_files": dirty_files,
        "outcome": outcome,
        "exit_code": exit_code,
    }
    with RUN_HISTORY_PATH.open("a", encoding="utf-8") as handle:
        handle.write(json.dumps(record, sort_keys=True))
        handle.write("\n")


def main() -> int:
    prompt_result = subprocess.run(
        [sys.executable, str(PROMPT_SCRIPT)],
        cwd=REPO_ROOT,
        check=False,
        capture_output=True,
        text=True,
    )
    if prompt_result.returncode != 0:
        sys.stderr.write(prompt_result.stdout)
        sys.stderr.write(prompt_result.stderr)
        return prompt_result.returncode

    task = load_task()
    dirty_files = tracked_dirty_files()
    cxys = subprocess.run(
        ["/home/connor/.local/bin/cxys", "-"],
        cwd=REPO_ROOT,
        check=False,
        input=prompt_result.stdout,
        text=True,
    )
    append_history(
        task,
        dirty_files,
        outcome="completed" if cxys.returncode == 0 else "failed",
        exit_code=cxys.returncode,
    )
    return cxys.returncode


if __name__ == "__main__":
    raise SystemExit(main())
