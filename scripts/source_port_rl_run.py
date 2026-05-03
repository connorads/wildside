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
AUTO_RECOVERY_PATH = (
    REPO_ROOT / ".cache/autonomy/source-port-frontier/auto_recovery_attempts.json"
)
ASM_COVERAGE_PATH = REPO_ROOT / "docs/source-port/asm-coverage.yaml"
BACKLOG_PATH = REPO_ROOT / "docs/source-port/backlog.md"
PROMISE_TOKEN = "__PROMISE_RL_DONE__"
DEFAULT_MAX_STALL_STREAK = 3

# Mirrors `TELEMETRY_REFRESH_PATHS` in crates/kidcli/src/autonomy.rs. The Rust
# side rejects emitting RefreshEvidence on stalls now, but if a future gaming
# variant slips past it, this defence-in-depth check catches it before another
# wasted iteration. Keep both lists in sync.
TELEMETRY_REFRESH_PATHS: frozenset[str] = frozenset(
    {
        "docs/source-port/methodology.md",
        "docs/source-port/research-map.md",
        "crates/kidcli/src/autonomy.rs",
        "crates/kidcli/src/release_gate.rs",
        "crates/kidcli/src/sweep/mod.rs",
    }
)
TELEMETRY_REFRESH_STREAK_THRESHOLD = 2

# Mirrors `auto_recovery_command` in crates/kidcli/src/autonomy.rs. Rust
# unit tests pin both shapes so the two sides cannot drift silently.
AUTO_RECOVERY_ARGV: dict[str, list[str]] = {
    "state_sweep": [
        "cargo", "run", "--release", "-p", "kidcli", "--",
        "sweep", "--levels", "0-104", "--tier", "state",
    ],
    "release_gate": ["mise", "run", "release-gate"],
}

MISSING_ARTEFACT_REGEN_HINT = {
    "state_sweep": (
        "cargo run --release -p kidcli -- sweep --levels 0-104 --tier state    "
        "# regenerate the missing state sweep "
        "(auto-recovery will attempt this if the fingerprint hasn't been retried)"
    ),
    "release_gate": (
        "mise run release-gate    "
        "# regenerate the missing release-gate evidence "
        "(auto-recovery will attempt this if the fingerprint hasn't been retried)"
    ),
}


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


def latest_clean_completion(records: list[dict]) -> dict | None:
    for record in reversed(records):
        if (
            record.get("outcome") == "completed"
            and record.get("exit_code") == 0
            and not record.get("dirty_tree")
        ):
            return record
    return None


def consecutive_stalled_repeat_count(records: list[dict]) -> int:
    count = 0
    for record in reversed(records):
        if record.get("outcome") == "stalled_repeat":
            count += 1
            continue
        break
    return count


def stall_streak_threshold() -> int:
    raw = os.environ.get("SOURCE_PORT_MAX_STALL_STREAK")
    if raw is None or raw == "":
        return DEFAULT_MAX_STALL_STREAK
    try:
        value = int(raw)
    except ValueError:
        return DEFAULT_MAX_STALL_STREAK
    return max(value, 1)


def asm_coverage_has_uncovered_sub_labels(path: Path) -> bool:
    """True if `asm-coverage.yaml` reports `sub_coverage_pct` below 100.0.

    Cheap heuristic over a cached file — we don't pull in PyYAML for this.
    """
    try:
        contents = path.read_text(encoding="utf-8")
    except OSError:
        return False
    for line in contents.splitlines():
        stripped = line.strip()
        if stripped.startswith("sub_coverage_pct:"):
            value = stripped.split(":", 1)[1].strip()
            try:
                return float(value) < 100.0
            except ValueError:
                return False
    return False


def backlog_has_unpromoted_open_item(path: Path) -> bool:
    """True if `backlog.md` has an open item (`[ ]` or `[~]`) whose row text
    does NOT include "Root cause identified". The autonomy cascade prefers
    items with that marker, so promoting another one unblocks new work.
    """
    try:
        contents = path.read_text(encoding="utf-8")
    except OSError:
        return False
    for line in contents.splitlines():
        if not line.startswith("|"):
            continue
        if "[ ]" not in line and "[~]" not in line:
            continue
        if "Root cause identified" not in line:
            return True
    return False


def derive_operator_unblocks(
    task: dict,
    repo_root: Path = REPO_ROOT,
) -> list[str]:
    """Render hints that match the current repo state.

    Order: missing-artefact (most actionable) → all-exhausted PV → uncovered
    asm coverage → backlog promotion → fallback rerun. The runner always
    appends the rerun line so the operator has a no-op safe choice.
    """
    hints: list[str] = []
    missing = task.get("missing_artefact")
    if missing in MISSING_ARTEFACT_REGEN_HINT:
        hints.append(MISSING_ARTEFACT_REGEN_HINT[missing])
    target = task.get("delegated_target") or ""
    if "all-exhausted" in target:
        hints.append(
            "Broaden docs/source-port/proactive-verification-config.yaml — "
            "all PV strategies report exhausted"
        )
    if asm_coverage_has_uncovered_sub_labels(repo_root / "docs/source-port/asm-coverage.yaml"):
        hints.append(
            "cargo run -q -p kidcli -- asm-coverage --update    "
            "# uncovered sub_ labels remain — refreshing may unblock task selection"
        )
    if backlog_has_unpromoted_open_item(repo_root / "docs/source-port/backlog.md"):
        hints.append(
            "Promote a backlog item by adding "
            '"Root cause identified" to its summary in '
            "docs/source-port/backlog.md"
        )
    hints.append(
        "Rerun `rl` to retry from the current state "
        "(safe no-op if nothing else has changed)"
    )
    return hints


def emit_circuit_breaker_message(
    task: dict,
    previous: dict | None,
    streak_after: int,
    threshold: int,
    recent_records: list[dict],
) -> None:
    target = task.get("delegated_target")
    fingerprint = task.get("progress_fingerprint")
    if task.get("definitively_stalled"):
        sys.stderr.write(
            "✗ Definitively stalled: autonomy module signalled the recovery "
            "cascade can only restate work that already landed cleanly.\n"
        )
    else:
        sys.stderr.write(
            "✗ Stall streak circuit breaker tripped: "
            f"{streak_after} consecutive stalled_repeat cycles "
            f"(threshold {threshold}) on the same delegated lane.\n"
        )
    sys.stderr.write(f"  Stuck delegated_target: {target}\n")
    sys.stderr.write(f"  Stuck progress_fingerprint: {fingerprint}\n")
    if previous is not None:
        sys.stderr.write(
            f"  Last clean completion: cycle {previous.get('cycle')} "
            f"({previous.get('delegated_target')})\n"
        )
    if recent_records:
        sys.stderr.write("  Recent cycles (oldest -> newest):\n")
        for record in recent_records:
            sys.stderr.write(
                f"    cycle={record.get('cycle')} outcome={record.get('outcome')} "
                f"target={record.get('delegated_target')}\n"
            )
    hints = derive_operator_unblocks(task)
    sys.stderr.write("  Operator unblocks (try one before relaunching `rl`):\n")
    for index, hint in enumerate(hints, start=1):
        sys.stderr.write(f"    {index}. {hint}\n")


def recent_commit_filesets(count: int) -> list[list[str]]:
    """Return the changed-file lists for the last `count` non-merge commits,
    newest first. Empty list if git is unavailable.
    """
    if count <= 0:
        return []
    delim = "__KID_COMMIT__"
    fmt = f"{delim}%n%H"
    result = subprocess.run(
        [
            "git",
            "log",
            f"-{count}",
            "--no-merges",
            "--name-only",
            f"--format={fmt}",
        ],
        cwd=REPO_ROOT,
        check=False,
        capture_output=True,
        text=True,
    )
    if result.returncode != 0:
        return []
    filesets: list[list[str]] = []
    current: list[str] | None = None
    expecting_sha = False
    for raw_line in result.stdout.splitlines():
        line = raw_line.strip()
        if line == delim:
            if current is not None:
                filesets.append(current)
            current = []
            expecting_sha = True
            continue
        if expecting_sha:
            expecting_sha = False
            continue
        if line and current is not None:
            current.append(line)
    if current is not None:
        filesets.append(current)
    return filesets


def is_telemetry_refresh_fileset(files: list[str]) -> bool:
    if not files:
        return False
    return all(path in TELEMETRY_REFRESH_PATHS for path in files)


def telemetry_refresh_streak() -> int:
    streak = 0
    for files in recent_commit_filesets(5):
        if is_telemetry_refresh_fileset(files):
            streak += 1
        else:
            break
    return streak


def is_clean_repeat(task: dict, dirty_files: list[str]) -> tuple[bool, dict | None]:
    if dirty_files:
        return (False, None)
    delegated_target = task.get("delegated_target")
    progress_fingerprint = task.get("progress_fingerprint")
    if not delegated_target or not progress_fingerprint:
        return (False, None)
    previous = latest_clean_completion(load_run_history())
    if previous is None:
        return (False, None)
    return (
        previous.get("delegated_target") == delegated_target
        and previous.get("progress_fingerprint") == progress_fingerprint,
        previous,
    )


def load_auto_recovery_attempts(path: Path = AUTO_RECOVERY_PATH) -> dict[str, dict]:
    try:
        contents = path.read_text(encoding="utf-8")
    except OSError:
        return {}
    try:
        data = json.loads(contents)
    except json.JSONDecodeError:
        return {}
    if not isinstance(data, dict):
        return {}
    return data


def should_auto_recover(fingerprint: str, path: Path = AUTO_RECOVERY_PATH) -> bool:
    entries = load_auto_recovery_attempts(path)
    entry = entries.get(fingerprint, {})
    return int(entry.get("attempts", 0) or 0) == 0


def record_auto_recovery_attempt(
    fingerprint: str,
    timestamp: str,
    path: Path = AUTO_RECOVERY_PATH,
) -> None:
    """Atomic write: serialise to .tmp + rename. A crash between write and
    rename leaves the existing JSON intact, mirroring the Rust helper.
    """
    path.parent.mkdir(parents=True, exist_ok=True)
    entries = load_auto_recovery_attempts(path)
    entry = entries.get(fingerprint, {"attempts": 0, "last_at": ""})
    entries[fingerprint] = {
        "attempts": int(entry.get("attempts", 0) or 0) + 1,
        "last_at": timestamp,
    }
    tmp_path = path.with_suffix(".json.tmp")
    tmp_path.write_text(json.dumps(entries, indent=2, sort_keys=True), encoding="utf-8")
    tmp_path.replace(path)


def append_auto_recovered_history(task: dict, missing: str) -> None:
    RUN_HISTORY_PATH.parent.mkdir(parents=True, exist_ok=True)
    record = {
        "timestamp": datetime.now(UTC).isoformat(),
        "cycle": task.get("cycle"),
        "delegated_goal": task.get("delegated_goal"),
        "delegated_target": task.get("delegated_target"),
        "progress_fingerprint": task.get("progress_fingerprint"),
        "missing_artefact": missing,
        "outcome": "auto_recovered",
        "exit_code": 0,
        "dirty_tree": False,
    }
    with RUN_HISTORY_PATH.open("a", encoding="utf-8") as handle:
        handle.write(json.dumps(record, sort_keys=True))
        handle.write("\n")


def attempt_auto_recovery(task: dict, dirty_at_launch: list[str]) -> int | None:
    """Returns 0 on successful auto-recovery (next iteration retries),
    1 if the operator must intervene (one-attempt cap reached), or None
    if no auto-recovery is applicable and the caller should continue with
    its existing flow.
    """
    missing = task.get("missing_artefact")
    if missing not in AUTO_RECOVERY_ARGV:
        return None
    if dirty_at_launch:
        # Don't run cargo on a dirty tree. Fall through to operator message.
        sys.stderr.write(
            f"⚠ Skipping auto-recovery for missing {missing}: tracked tree is dirty.\n"
        )
        return None
    fingerprint = task.get("progress_fingerprint") or ""
    if not should_auto_recover(fingerprint):
        sys.stderr.write(
            f"✗ Auto-recovery already attempted for fingerprint {fingerprint}; "
            "manual intervention needed.\n"
        )
        return 1
    argv = AUTO_RECOVERY_ARGV[missing]
    sys.stderr.write("\n")
    sys.stderr.write("=" * 72 + "\n")
    sys.stderr.write(f"  Auto-recovery: regenerating missing {missing}\n")
    sys.stderr.write(f"  Command: {' '.join(argv)}\n")
    sys.stderr.write("  Sweep typically takes 2-3 min; release-gate ~1 min.\n")
    sys.stderr.write("=" * 72 + "\n")
    sys.stderr.flush()
    record_auto_recovery_attempt(fingerprint, datetime.now(UTC).isoformat())
    result = subprocess.run(argv, cwd=REPO_ROOT, check=False)
    if result.returncode != 0:
        sys.stderr.write(
            f"✗ Auto-recovery command exited {result.returncode}; manual intervention needed.\n"
        )
        return 1
    append_auto_recovered_history(task, missing)
    sys.stderr.write("✓ Auto-recovery succeeded; next `rl` iteration will retry.\n")
    return 0


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

    # Defence-in-depth: if the last 2+ commits in the rl session touched only
    # telemetry-refresh paths (docs and/or gate-signal renderers), the agent
    # has been gaming the loop and the autonomy module may not have caught
    # the new variant yet. Hard-stop with PROMISE_TOKEN so the operator
    # decides whether to pivot or rest. The Rust-side fix in autonomy.rs
    # should mean this never fires in practice — it limits damage.
    streak = telemetry_refresh_streak()
    if streak >= TELEMETRY_REFRESH_STREAK_THRESHOLD and not dirty_before:
        sys.stderr.write(
            f"✗ Telemetry-refresh streak detected: last {streak} commits "
            "touched only telemetry-refresh paths "
            "(docs/source-port/methodology.md, "
            "docs/source-port/research-map.md, "
            "crates/kidcli/src/autonomy.rs, "
            "crates/kidcli/src/release_gate.rs, "
            "crates/kidcli/src/sweep/mod.rs).\n"
        )
        sys.stderr.write(
            "  Pivot or rest — adding another gate-signal field that cites "
            "itself isn't engine progress.\n"
        )
        sys.stderr.write(
            "  Try: cargo run -p kidcli -- autonomous-loop "
            "--goal proactive-verification\n"
        )
        sys.stdout.write(PROMISE_TOKEN + "\n")
        sys.stdout.flush()
        return 0

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
    dirty_at_launch = tracked_dirty_files()

    # Auto-recovery for known-regenerable missing artefacts. Bounded to one
    # attempt per fingerprint; dirty trees skip the auto-run. On success the
    # prompt script is re-invoked so `next-task.json` reflects post-recovery
    # state, avoiding a spurious is_clean_repeat trip on the next iteration.
    auto_recovery_outcome = attempt_auto_recovery(task, dirty_at_launch)
    if auto_recovery_outcome == 0:
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
        return 0
    if auto_recovery_outcome == 1:
        # One-attempt cap reached or recovery failed: emit operator hints.
        previous = latest_clean_completion(load_run_history())
        emit_circuit_breaker_message(
            task,
            previous,
            streak_after=0,
            threshold=0,
            recent_records=[],
        )
        sys.stdout.write(PROMISE_TOKEN + "\n")
        sys.stdout.flush()
        return 0

    if task.get("definitively_stalled"):
        # Autonomy module signalled that the recovery cascade can only
        # restate work that already completed cleanly. Skip the
        # DEFAULT_MAX_STALL_STREAK wait and hand control back to the
        # operator on iteration 1 with the same unblock hints.
        previous = latest_clean_completion(load_run_history())
        emit_circuit_breaker_message(
            task,
            previous,
            streak_after=0,
            threshold=0,
            recent_records=[],
        )
        sys.stdout.write(PROMISE_TOKEN + "\n")
        sys.stdout.flush()
        return 0
    repeated, previous = is_clean_repeat(task, dirty_at_launch)
    if repeated:
        sys.stderr.write(
            "✗ No-progress guard: fresh package selected the same delegated target "
            "and progress fingerprint "
            f"as the last clean completed run ({task.get('delegated_target')}).\n"
        )
        sys.stderr.write(
            "  Refresh coverage or inspect task selection before launching another agent.\n"
        )
        if previous is not None:
            sys.stderr.write(f"  Previous cycle: {previous.get('cycle')}\n")
            sys.stderr.write(
                f"  Previous fingerprint: {previous.get('progress_fingerprint')}\n"
            )
        history_before = load_run_history()
        threshold = stall_streak_threshold()
        streak_after = consecutive_stalled_repeat_count(history_before) + 1
        append_history(
            task,
            dirty_at_launch,
            outcome="stalled_repeat",
            exit_code=1,
            preflight=preflight,
        )
        if streak_after >= threshold:
            recent = (history_before + [{"cycle": task.get("cycle"),
                                         "outcome": "stalled_repeat",
                                         "delegated_target": task.get("delegated_target")}])[-streak_after:]
            emit_circuit_breaker_message(task, previous, streak_after, threshold, recent)
            sys.stdout.write(PROMISE_TOKEN + "\n")
            sys.stdout.flush()
            return 0
        return 1

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
        dirty_at_launch,
        outcome=outcome,
        exit_code=cxys.returncode,
        preflight=preflight,
    )
    return 1 if outcome == "uncommitted" else cxys.returncode


if __name__ == "__main__":
    raise SystemExit(main())
