#!/usr/bin/env python3

from __future__ import annotations

import hashlib
import json
import os
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


def dirty_tree_fingerprint(dirty_files: list[str]) -> str | None:
    if not dirty_files:
        return None
    result = subprocess.run(
        ["git", "diff", "--", *dirty_files],
        cwd=REPO_ROOT,
        check=False,
        capture_output=True,
        text=True,
    )
    payload = "\n".join(dirty_files)
    if result.returncode == 0:
        payload = f"{payload}\n{result.stdout}"
    return hashlib.sha256(payload.encode("utf-8")).hexdigest()[:16]


def is_rust_related(path: str) -> bool:
    name = Path(path).name
    return path.endswith(".rs") or name in {"Cargo.toml", "Cargo.lock", "build.rs"}


def is_kid_core_scoped_rust_change(path: str) -> bool:
    name = Path(path).name
    return path.startswith("crates/kid-core/") and (
        path.endswith(".rs") or name in {"Cargo.toml", "build.rs"}
    )


def select_preflight_command(dirty_files: list[str]) -> list[str] | None:
    rust_related = [path for path in dirty_files if is_rust_related(path)]
    if not rust_related:
        return None
    if all(is_kid_core_scoped_rust_change(path) for path in rust_related):
        return ["cargo", "check", "-p", "kid-core"]
    return ["cargo", "check", "--workspace"]


def summarise_preflight_output(result: subprocess.CompletedProcess[str]) -> str:
    for stream in (result.stderr, result.stdout):
        for line in stream.splitlines():
            stripped = line.strip()
            if stripped:
                return stripped[:240]
    return f"exit {result.returncode}"


def run_dirty_tree_preflight(dirty_files: list[str]) -> dict[str, object]:
    command = select_preflight_command(dirty_files)
    if command is None:
        return {
            "fingerprint": dirty_tree_fingerprint(dirty_files),
            "command": None,
            "exit_code": None,
            "summary": "skipped: dirty files are not Rust/Cargo inputs",
        }

    result = subprocess.run(
        command,
        cwd=REPO_ROOT,
        check=False,
        capture_output=True,
        text=True,
    )
    return {
        "fingerprint": dirty_tree_fingerprint(dirty_files),
        "command": command,
        "exit_code": result.returncode,
        "summary": summarise_preflight_output(result),
    }


def append_history(
    task: dict,
    dirty_files: list[str],
    outcome: str,
    exit_code: int,
    preflight: dict[str, object],
) -> None:
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
        "dirty_tree_fingerprint": preflight.get("fingerprint"),
        "preflight_command": preflight.get("command"),
        "preflight_exit_code": preflight.get("exit_code"),
        "preflight_summary": preflight.get("summary"),
        "outcome": outcome,
        "exit_code": exit_code,
    }
    with RUN_HISTORY_PATH.open("a", encoding="utf-8") as handle:
        handle.write(json.dumps(record, sort_keys=True))
        handle.write("\n")


def which(name: str) -> str:
    import shutil

    path = shutil.which(name)
    if path is None:
        raise SystemExit(f"agent command not found: {name}")
    return path


def resolve_agent(name: str | None) -> list[str]:
    name = name or "cxys"
    return [which(name), "-"]


def main() -> int:
    agent_name = sys.argv[1] if len(sys.argv) > 1 else None
    dirty_before = tracked_dirty_files()
    preflight = run_dirty_tree_preflight(dirty_before)
    prompt_env = os.environ.copy()
    prompt_env["SOURCE_PORT_DIRTY_TREE_FINGERPRINT"] = str(
        preflight.get("fingerprint") or ""
    )
    prompt_env["SOURCE_PORT_DIRTY_PREFLIGHT_COMMAND"] = json.dumps(
        preflight.get("command")
    )
    prompt_env["SOURCE_PORT_DIRTY_PREFLIGHT_EXIT_CODE"] = json.dumps(
        preflight.get("exit_code")
    )
    prompt_env["SOURCE_PORT_DIRTY_PREFLIGHT_SUMMARY"] = str(preflight.get("summary") or "")

    prompt_result = subprocess.run(
        [sys.executable, str(PROMPT_SCRIPT)],
        cwd=REPO_ROOT,
        check=False,
        capture_output=True,
        env=prompt_env,
        text=True,
    )
    if prompt_result.returncode != 0:
        sys.stderr.write(prompt_result.stdout)
        sys.stderr.write(prompt_result.stderr)
        return prompt_result.returncode

    agent_cmd = resolve_agent(agent_name)
    task = load_task()
    cxys = subprocess.run(
        agent_cmd,
        cwd=REPO_ROOT,
        check=False,
        input=prompt_result.stdout,
        text=True,
    )
    dirty_after = tracked_dirty_files()

    if cxys.returncode == 0 and dirty_after:
        outcome = "uncommitted"
        sys.stderr.write(
            f"⚠ Agent exited 0 but left uncommitted tracked files: {dirty_after}\n"
        )
    elif cxys.returncode == 0:
        outcome = "completed"
    else:
        outcome = "failed"

    append_history(
        task,
        dirty_before,
        outcome=outcome,
        exit_code=cxys.returncode,
        preflight=preflight,
    )
    return 1 if outcome == "uncommitted" else cxys.returncode


if __name__ == "__main__":
    raise SystemExit(main())
