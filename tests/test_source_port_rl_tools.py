import json
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parent.parent

sys.path.insert(0, str(REPO_ROOT / "scripts"))

import source_port_rl_prompt
import source_port_rl_run


class SourcePortRlToolTests(unittest.TestCase):
    def setUp(self) -> None:
        self.temp_dir = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp_dir.cleanup)
        self.repo_root = Path(self.temp_dir.name)

        self.original_run_repo_root = source_port_rl_run.REPO_ROOT
        self.original_run_history_path = source_port_rl_run.RUN_HISTORY_PATH
        self.original_prompt_repo_root = source_port_rl_prompt.REPO_ROOT
        self.original_prompt_history_path = source_port_rl_prompt.RUN_HISTORY_PATH

        source_port_rl_run.REPO_ROOT = self.repo_root
        source_port_rl_run.RUN_HISTORY_PATH = (
            self.repo_root / ".cache/autonomy/source-port-frontier/run-history.jsonl"
        )
        source_port_rl_prompt.REPO_ROOT = self.repo_root
        source_port_rl_prompt.RUN_HISTORY_PATH = (
            self.repo_root / ".cache/autonomy/source-port-frontier/run-history.jsonl"
        )

    def tearDown(self) -> None:
        source_port_rl_run.REPO_ROOT = self.original_run_repo_root
        source_port_rl_run.RUN_HISTORY_PATH = self.original_run_history_path
        source_port_rl_prompt.REPO_ROOT = self.original_prompt_repo_root
        source_port_rl_prompt.RUN_HISTORY_PATH = self.original_prompt_history_path

    def git(self, *args: str) -> subprocess.CompletedProcess[str]:
        return subprocess.run(
            ["git", *args],
            cwd=self.repo_root,
            check=True,
            capture_output=True,
            text=True,
        )

    def test_select_preflight_command_prefers_kid_core_for_local_rust_edits(self) -> None:
        dirty_files = ["crates/kid-core/src/level.rs", "crates/kid-core/src/tests.rs"]

        self.assertEqual(
            source_port_rl_run.select_preflight_command(dirty_files),
            ["cargo", "check", "-p", "kid-core"],
        )
        self.assertEqual(
            source_port_rl_prompt.select_preflight_command(dirty_files),
            ["cargo", "check", "-p", "kid-core"],
        )

    def test_select_preflight_command_uses_workspace_for_broader_rust_edits(self) -> None:
        dirty_files = ["crates/kid-core/src/level.rs", "Cargo.toml"]

        self.assertEqual(
            source_port_rl_run.select_preflight_command(dirty_files),
            ["cargo", "check", "--workspace"],
        )
        self.assertEqual(
            source_port_rl_prompt.select_preflight_command(dirty_files),
            ["cargo", "check", "--workspace"],
        )

    def test_select_preflight_command_skips_non_rust_dirty_files(self) -> None:
        dirty_files = ["docs/source-port/methodology.md"]

        self.assertIsNone(source_port_rl_run.select_preflight_command(dirty_files))
        self.assertIsNone(source_port_rl_prompt.select_preflight_command(dirty_files))

    def test_run_autonomy_loop_skips_bootstrap_when_upstream_exists(self) -> None:
        upstream = (
            self.repo_root / ".cache/upstream/kid-chameleon-disasm/kid.asm"
        )
        upstream.parent.mkdir(parents=True, exist_ok=True)
        upstream.write_text("; fixture\n", encoding="utf-8")
        commands: list[list[str]] = []
        original_run = source_port_rl_prompt.subprocess.run

        def fake_run(
            cmd: list[str], *args: object, **kwargs: object
        ) -> subprocess.CompletedProcess[str]:
            commands.append(cmd)
            return subprocess.CompletedProcess(cmd, 0, stdout="", stderr="")

        source_port_rl_prompt.subprocess.run = fake_run  # type: ignore[assignment]
        try:
            source_port_rl_prompt.run_autonomy_loop()
        finally:
            source_port_rl_prompt.subprocess.run = original_run  # type: ignore[assignment]

        self.assertEqual(len(commands), 2)
        self.assertEqual(commands[0][-2:], ["--", "asm-coverage", "--update"][-2:])
        self.assertNotIn("bootstrap-upstream", commands[0])
        self.assertNotIn("bootstrap-upstream", commands[1])

    def test_run_autonomy_loop_bootstraps_missing_upstream(self) -> None:
        upstream = (
            self.repo_root / ".cache/upstream/kid-chameleon-disasm/kid.asm"
        )
        commands: list[list[str]] = []
        original_run = source_port_rl_prompt.subprocess.run

        def fake_run(
            cmd: list[str], *args: object, **kwargs: object
        ) -> subprocess.CompletedProcess[str]:
            commands.append(cmd)
            if "bootstrap-upstream" in cmd:
                upstream.parent.mkdir(parents=True, exist_ok=True)
                upstream.write_text("; fixture\n", encoding="utf-8")
            return subprocess.CompletedProcess(cmd, 0, stdout="", stderr="")

        source_port_rl_prompt.subprocess.run = fake_run  # type: ignore[assignment]
        try:
            source_port_rl_prompt.run_autonomy_loop()
        finally:
            source_port_rl_prompt.subprocess.run = original_run  # type: ignore[assignment]

        self.assertEqual(len(commands), 3)
        self.assertIn("bootstrap-upstream", commands[0])
        self.assertIn("asm-coverage", commands[1])
        self.assertIn("autonomous-loop", commands[2])

    def test_run_autonomy_loop_stops_when_bootstrap_fails(self) -> None:
        commands: list[list[str]] = []
        original_run = source_port_rl_prompt.subprocess.run

        def fake_run(
            cmd: list[str], *args: object, **kwargs: object
        ) -> subprocess.CompletedProcess[str]:
            commands.append(cmd)
            if "bootstrap-upstream" in cmd:
                return subprocess.CompletedProcess(
                    cmd, 128, stdout="", stderr="clone failed\n"
                )
            raise AssertionError(f"unexpected command after bootstrap failure: {cmd}")

        source_port_rl_prompt.subprocess.run = fake_run  # type: ignore[assignment]
        try:
            with self.assertRaises(SystemExit) as caught:
                source_port_rl_prompt.run_autonomy_loop()
        finally:
            source_port_rl_prompt.subprocess.run = original_run  # type: ignore[assignment]

        self.assertEqual(caught.exception.code, 128)
        self.assertEqual(len(commands), 1)
        self.assertIn("bootstrap-upstream", commands[0])

    def test_dirty_tree_fingerprint_changes_with_diff(self) -> None:
        self.git("init")
        self.git("config", "user.name", "Test User")
        self.git("config", "user.email", "test@example.com")
        tracked = self.repo_root / "crates/kid-core/src/level.rs"
        tracked.parent.mkdir(parents=True, exist_ok=True)
        tracked.write_text("fn before() {}\n", encoding="utf-8")
        self.git("add", ".")
        self.git("commit", "-m", "init")

        tracked.write_text("fn after_one() {}\n", encoding="utf-8")
        first = source_port_rl_run.dirty_tree_fingerprint(["crates/kid-core/src/level.rs"])
        tracked.write_text("fn after_two() {}\n", encoding="utf-8")
        second = source_port_rl_run.dirty_tree_fingerprint(["crates/kid-core/src/level.rs"])

        self.assertIsNotNone(first)
        self.assertIsNotNone(second)
        self.assertNotEqual(first, second)

    def test_append_history_records_preflight_fields(self) -> None:
        source_port_rl_run.append_history(
            task={"cycle": 4, "progress_fingerprint": "lane:test"},
            dirty_files=["crates/kid-core/src/level.rs"],
            outcome="failed",
            exit_code=101,
            preflight={
                "fingerprint": "abcd1234",
                "command": ["cargo", "check", "-p", "kid-core"],
                "exit_code": 101,
                "summary": "error[E0425]: cannot find value",
            },
        )

        record = json.loads(source_port_rl_run.RUN_HISTORY_PATH.read_text(encoding="utf-8"))
        self.assertEqual(record["dirty_tree_fingerprint"], "abcd1234")
        self.assertEqual(record["preflight_command"], ["cargo", "check", "-p", "kid-core"])
        self.assertEqual(record["preflight_exit_code"], 101)
        self.assertIn("cannot find value", record["preflight_summary"])

    def test_failed_dirty_preflight_count_filters_by_fingerprint(self) -> None:
        source_port_rl_prompt.RUN_HISTORY_PATH.parent.mkdir(parents=True, exist_ok=True)
        source_port_rl_prompt.RUN_HISTORY_PATH.write_text(
            "\n".join(
                [
                    json.dumps(
                        {
                            "dirty_tree_fingerprint": "aaaa1111",
                            "preflight_exit_code": 101,
                        }
                    ),
                    json.dumps(
                        {
                            "dirty_tree_fingerprint": "aaaa1111",
                            "preflight_exit_code": 0,
                        }
                    ),
                    json.dumps(
                        {
                            "dirty_tree_fingerprint": "bbbb2222",
                            "preflight_exit_code": 101,
                        }
                    ),
                ]
            )
            + "\n",
            encoding="utf-8",
        )

        records = source_port_rl_prompt.load_run_history()

        self.assertEqual(
            source_port_rl_prompt.failed_dirty_preflight_count(records, "aaaa1111"), 1
        )
        self.assertEqual(
            source_port_rl_prompt.failed_dirty_preflight_count(records, "bbbb2222"), 1
        )
        self.assertEqual(
            source_port_rl_prompt.failed_dirty_preflight_count(records, "cccc3333"), 0
        )


    def test_consecutive_stalled_repeat_count_walks_tail(self) -> None:
        records = [
            {"outcome": "completed"},
            {"outcome": "stalled_repeat"},
            {"outcome": "stalled_repeat"},
            {"outcome": "stalled_repeat"},
        ]
        self.assertEqual(source_port_rl_run.consecutive_stalled_repeat_count(records), 3)

    def test_consecutive_stalled_repeat_count_breaks_on_other_outcomes(self) -> None:
        records = [
            {"outcome": "stalled_repeat"},
            {"outcome": "completed"},
            {"outcome": "stalled_repeat"},
            {"outcome": "stalled_repeat"},
        ]
        # Only the trailing run of stalled_repeat counts; the completed in the middle breaks.
        self.assertEqual(source_port_rl_run.consecutive_stalled_repeat_count(records), 2)

    def test_consecutive_stalled_repeat_count_handles_empty(self) -> None:
        self.assertEqual(source_port_rl_run.consecutive_stalled_repeat_count([]), 0)

    def test_stall_streak_threshold_default_when_unset(self) -> None:
        import os

        original = os.environ.pop("SOURCE_PORT_MAX_STALL_STREAK", None)
        try:
            self.assertEqual(
                source_port_rl_run.stall_streak_threshold(),
                source_port_rl_run.DEFAULT_MAX_STALL_STREAK,
            )
        finally:
            if original is not None:
                os.environ["SOURCE_PORT_MAX_STALL_STREAK"] = original

    def test_stall_streak_threshold_reads_env(self) -> None:
        import os

        original = os.environ.get("SOURCE_PORT_MAX_STALL_STREAK")
        os.environ["SOURCE_PORT_MAX_STALL_STREAK"] = "5"
        try:
            self.assertEqual(source_port_rl_run.stall_streak_threshold(), 5)
        finally:
            if original is None:
                os.environ.pop("SOURCE_PORT_MAX_STALL_STREAK", None)
            else:
                os.environ["SOURCE_PORT_MAX_STALL_STREAK"] = original

    def test_stall_streak_threshold_clamps_to_one(self) -> None:
        import os

        original = os.environ.get("SOURCE_PORT_MAX_STALL_STREAK")
        os.environ["SOURCE_PORT_MAX_STALL_STREAK"] = "0"
        try:
            self.assertEqual(source_port_rl_run.stall_streak_threshold(), 1)
        finally:
            if original is None:
                os.environ.pop("SOURCE_PORT_MAX_STALL_STREAK", None)
            else:
                os.environ["SOURCE_PORT_MAX_STALL_STREAK"] = original

    def test_stall_streak_threshold_invalid_falls_back_to_default(self) -> None:
        import os

        original = os.environ.get("SOURCE_PORT_MAX_STALL_STREAK")
        os.environ["SOURCE_PORT_MAX_STALL_STREAK"] = "not-a-number"
        try:
            self.assertEqual(
                source_port_rl_run.stall_streak_threshold(),
                source_port_rl_run.DEFAULT_MAX_STALL_STREAK,
            )
        finally:
            if original is None:
                os.environ.pop("SOURCE_PORT_MAX_STALL_STREAK", None)
            else:
                os.environ["SOURCE_PORT_MAX_STALL_STREAK"] = original

    def test_definitively_stalled_task_emits_promise_token_on_iteration_1(self) -> None:
        import io
        import os

        # Seed a next-task.json with definitively_stalled=true so the runner
        # sees the autonomy module's "cascade can only restate completed
        # work" signal and short-circuits without waiting for
        # DEFAULT_MAX_STALL_STREAK silent retries.
        next_task_path = (
            self.repo_root / ".cache/autonomy/source-port-frontier/next-task.json"
        )
        next_task_path.parent.mkdir(parents=True, exist_ok=True)
        next_task_path.write_text(
            json.dumps(
                {
                    "cycle": 474,
                    "delegated_target": "state-sweep",
                    "progress_fingerprint": "lane:state-sweep:recovery:refresh_evidence",
                    "definitively_stalled": True,
                }
            ),
            encoding="utf-8",
        )

        # Guard against the prompt subprocess and agent subprocess actually
        # running — the runner should never reach them on this path.
        original_next_task_path = source_port_rl_run.NEXT_TASK_PATH
        original_run = subprocess.run
        original_argv = sys.argv
        original_stdout = sys.stdout
        original_stderr = sys.stderr

        def fake_run(
            cmd: list[str], *args: object, **kwargs: object
        ) -> subprocess.CompletedProcess[str]:
            # The runner calls `git status` for dirty_files; everything else
            # should not execute on the definitively_stalled path.
            if cmd and cmd[0] == "git":
                return subprocess.CompletedProcess(cmd, 0, stdout="", stderr="")
            if cmd and "source_port_rl_prompt" in str(cmd[-1]):
                # Prompt script runs before the short-circuit; mimic success.
                return subprocess.CompletedProcess(cmd, 0, stdout="{}", stderr="")
            raise AssertionError(
                f"runner should not invoke subprocess after "
                f"definitively_stalled short-circuit: {cmd}"
            )

        source_port_rl_run.NEXT_TASK_PATH = next_task_path
        subprocess.run = fake_run  # type: ignore[assignment]
        source_port_rl_run.subprocess.run = fake_run  # type: ignore[assignment]
        sys.argv = ["source_port_rl_run.py"]
        sys.stdout = io.StringIO()
        sys.stderr = io.StringIO()
        # Prevent resolve_agent from looking up a real binary.
        original_resolve = source_port_rl_run.resolve_agent
        source_port_rl_run.resolve_agent = lambda name: ["/bin/true"]
        try:
            exit_code = source_port_rl_run.main()
            stdout_value = sys.stdout.getvalue()
            stderr_value = sys.stderr.getvalue()
        finally:
            source_port_rl_run.NEXT_TASK_PATH = original_next_task_path
            subprocess.run = original_run  # type: ignore[assignment]
            source_port_rl_run.subprocess.run = original_run  # type: ignore[assignment]
            source_port_rl_run.resolve_agent = original_resolve
            sys.argv = original_argv
            sys.stdout = original_stdout
            sys.stderr = original_stderr

        self.assertEqual(exit_code, 0)
        self.assertIn(source_port_rl_run.PROMISE_TOKEN, stdout_value)
        self.assertIn("Definitively stalled", stderr_value)
        self.assertIn("state-sweep", stderr_value)


if __name__ == "__main__":
    unittest.main()
