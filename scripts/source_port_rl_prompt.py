#!/usr/bin/env python3

from __future__ import annotations

import json
import os
import subprocess
import sys
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parent.parent
NEXT_TASK_PATH = REPO_ROOT / ".cache/autonomy/source-port-frontier/next-task.json"
NEXT_AGENT_PATH = REPO_ROOT / ".cache/autonomy/source-port-frontier/next-agent.md"
RUN_HISTORY_PATH = REPO_ROOT / ".cache/autonomy/source-port-frontier/run-history.jsonl"


def run_autonomy_loop() -> None:
    routines = os.environ.get("SOURCE_PORT_ROUTINES", "3")
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
            "--routines-per-task",
            routines,
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


def current_dirty_preflight(dirty_files: list[str]) -> dict[str, object]:
    env_fingerprint = os.environ.get("SOURCE_PORT_DIRTY_TREE_FINGERPRINT") or None
    env_command = json.loads(os.environ.get("SOURCE_PORT_DIRTY_PREFLIGHT_COMMAND", "null"))
    env_exit_code = json.loads(
        os.environ.get("SOURCE_PORT_DIRTY_PREFLIGHT_EXIT_CODE", "null")
    )
    env_summary = os.environ.get("SOURCE_PORT_DIRTY_PREFLIGHT_SUMMARY")
    if env_fingerprint or env_command is not None or env_summary:
        return {
            "fingerprint": env_fingerprint,
            "command": env_command,
            "exit_code": env_exit_code,
            "summary": env_summary or "unknown",
        }

    command = select_preflight_command(dirty_files)
    if command is None:
        return {
            "fingerprint": None,
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
        "fingerprint": None,
        "command": command,
        "exit_code": result.returncode,
        "summary": summarise_preflight_output(result),
    }


def load_run_history() -> list[dict]:
    if not RUN_HISTORY_PATH.exists():
        return []
    records: list[dict] = []
    for line in RUN_HISTORY_PATH.read_text(encoding="utf-8").splitlines():
        line = line.strip()
        if not line:
            continue
        try:
            records.append(json.loads(line))
        except json.JSONDecodeError:
            continue
    return records


def failed_dirty_preflight_count(records: list[dict], fingerprint: str | None) -> int:
    if not fingerprint:
        return 0
    return sum(
        1
        for record in records
        if record.get("dirty_tree_fingerprint") == fingerprint
        and record.get("preflight_exit_code") not in (None, 0)
    )


def render_target_routines(task: dict) -> str:
    routines = task.get("target_routines")
    if not routines or len(routines) == 0:
        return ""
    lines = [f"## Routines to implement ({len(routines)})", ""]
    for i, r in enumerate(routines, 1):
        lines.append(
            f"{i}. `{r['label']}` — {r['subsystem']} — {r['asm_file']}:{r['line_number']}"
        )
    lines.append("")
    return "\n".join(lines)


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
    recovery_mode = task.get("recovery_mode")
    recovery_step = task.get("recovery_step")
    stalled_repetition_count = task.get("stalled_repetition_count")
    pivot_reason = task.get("pivot_reason")
    next_fallback_goal = task.get("next_fallback_goal")
    dirty_files = tracked_dirty_files()
    preflight = current_dirty_preflight(dirty_files)
    preflight_command = preflight.get("command")
    failed_preflight_repeats = failed_dirty_preflight_count(
        load_run_history(), preflight.get("fingerprint")
    )

    recovery_lines = []
    if recovery_mode:
        recovery_lines.extend(
            [
                "## Recovery state",
                "",
                f"- Recovery mode: `{recovery_mode}`",
                f"- Recovery step: `{recovery_step or 'none'}`",
            ]
        )
        if stalled_repetition_count is not None:
            recovery_lines.append(
                f"- Repeated stalled cycles on this lane: `{stalled_repetition_count}`"
            )
        if pivot_reason:
            recovery_lines.append(f"- Pivot reason: {pivot_reason}")
        if next_fallback_goal:
            recovery_lines.append(f"- Next fallback goal: `{next_fallback_goal}`")
        recovery_lines.append("")

    dirty_lines = [
        "## Git state",
        "",
        f"- Tracked dirty tree: `{'yes' if dirty_files else 'no'}`",
    ]
    if dirty_files:
        dirty_lines.append("- Dirty files:")
        dirty_lines.extend(f"  - `{path}`" for path in dirty_files)
        dirty_lines.append(
            "- Start by resolving the current tracked work before changing lanes."
        )
        if preflight_command is None:
            dirty_lines.append("- Dirty-tree preflight: skipped")
        else:
            dirty_lines.append(
                f"- Dirty-tree preflight command: `{' '.join(preflight_command)}`"
            )
            dirty_lines.append(
                f"- Dirty-tree preflight exit: `{preflight.get('exit_code')}`"
            )
        dirty_lines.append(f"- Dirty-tree preflight summary: {preflight.get('summary')}")
    dirty_lines.append("")

    dirty_recovery_lines: list[str] = []
    if (
        dirty_files
        and preflight.get("command") is not None
        and preflight.get("exit_code") not in (None, 0)
    ):
        dirty_recovery_lines.extend(
            [
                "## Dirty-tree compile recovery",
                "",
                "- The tracked diff is currently red. Do not start new lane work until this unit is repaired, stashed, or deliberately discarded.",
            ]
        )
        if failed_preflight_repeats > 0:
            dirty_recovery_lines.append(
                f"- Same dirty fingerprint has already failed preflight `{failed_preflight_repeats}` prior run(s). Escalate recovery instead of replaying the lane."
            )
        dirty_recovery_lines.extend(
            [
                "- Recovery commands to run first:",
                *[f"  - `git diff -- {path}`" for path in dirty_files],
                f"  - `{' '.join(preflight_command)}`",
                "  - If the diff is valid, repair and verify it as one coherent unit.",
                "  - If the diff is a dead end, stash it or discard only that unit before resuming lane work.",
                "",
            ]
        )

    task_heading = "## Task"
    if recovery_mode == "dirty_tree":
        task_heading = "## Immediate task"
    elif recovery_mode:
        task_heading = "## Recovery task"

    routines_section = render_target_routines(task)

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

{chr(10).join(recovery_lines)}{chr(10).join(dirty_lines)}{chr(10).join(dirty_recovery_lines)}{task_heading}

- Title: {task["title"]}
- Summary: {task["summary"]}

{routines_section}## Read only these repo artefacts

{bullet_list([str(NEXT_TASK_PATH.relative_to(REPO_ROOT)), str(NEXT_AGENT_PATH.relative_to(REPO_ROOT)), *sources])}

## Commands

{bullet_list(commands)}

## Done criteria

{bullet_list(done_criteria)}

## Implementation targets

{bullet_list(implementation_targets) if implementation_targets else '- None specified; stay inside the delegated lane.'}

## Commit gate (mandatory)

Every completed routine MUST be committed before the session ends.
After verify-slice-fast passes:
1. `kidcli asm-coverage --update`
2. `git add <changed files>`
3. `git commit` citing the ASM label
Commit each routine individually. If you finish your turn with uncommitted tracked changes, the session has FAILED.

## Hard rules

- Work only the delegated lane from the live package above.
- If the git tree is dirty, resolve that tracked work first; do not jump to a fresh lane until the dirty unit is verified, committed, or intentionally discarded.
- Do not read `docs/source-port/backlog.md`, `docs/source-port/frontier-state.yaml`, or `docs/source-port/tas-roadmap.md` unless this package explicitly lists them in `sources`.
- Do not reopen TAS-frontier work if the live package delegates to `case-queue` or `sweep-parity`.
- If the delegated goal is `sweep-parity`, stay on sweep/release-gate hardening, but only because the live package found no gameplay-producing lane for this run.
- If the delegated goal is `case-queue` and the delegated target is a reduced case, re-verify that case before changing code.
- If the delegated goal is `case-queue` and the delegated target is a recording slug, derive and reduce a failing gameplay case from that recording before doing any sweep-only work.
- Do not choose `sweep-parity` or another verification-only lane while the live package is delegating to recording-derived gameplay work.
- If `recovery_mode` is present, complete that recovery step before resuming routine lane work.
- Use ASM first for gameplay behaviour; use probes only when ASM plus oracle still leave ambiguity.
- Do not make a handoff-only commit.
"""
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
