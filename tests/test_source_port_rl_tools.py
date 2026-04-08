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


if __name__ == "__main__":
    unittest.main()
