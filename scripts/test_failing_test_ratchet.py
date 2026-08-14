#!/usr/bin/env python3
"""Black-box contract tests for `failing_test_ratchet.py` (#443).

The ratchet's one fatal failure mode is fail-open: reading a run that DID NOT
HAPPEN (compile error, harness abort, signal) as "zero failures" and letting the
hk `--update` fix step rewrite the baseline to empty. These tests pin the
fail-closed contract from the outside: each test copies the shipped script into
a temp repo-shaped tree (`scripts/`, `docs/source-port/`, `.cache/generated/`),
plants a fake `cargo` on PATH that emits a chosen stdout/stderr/exit status, and
asserts on the script's exit code, stderr, and the baseline file's bytes. No
internals are imported; the subprocess CLI is the contract under test.

Run: python3 scripts/test_failing_test_ratchet.py
"""

from __future__ import annotations

import os
import shutil
import stat
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path

SCRIPT = Path(__file__).with_name("failing_test_ratchet.py")

BASELINE_FIXTURE = """\
# Fixture baseline: header comment must survive a ratchet-down.
tests:
  - sweep::grandfathered_alpha
  - sweep::grandfathered_beta
"""

# libtest's real shape: an inline FAILED line, a first `failures:` block with
# per-test stdout dumps, then the flat second `failures:` list the parser reads.
LIBTEST_ONE_GRANDFATHERED_FAILURE = """\
running 3 tests
test sweep::grandfathered_alpha ... FAILED
test sweep::other ... ok
test sweep::third ... ok

failures:

---- sweep::grandfathered_alpha stdout ----
thread panicked at 'boom'

failures:
    sweep::grandfathered_alpha

test result: FAILED. 2 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out
"""

LIBTEST_ALL_GREEN = """\
running 3 tests
test sweep::grandfathered_alpha ... ok
test sweep::other ... ok
test sweep::third ... ok

test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
"""

LIBTEST_NOVEL_FAILURE = LIBTEST_ONE_GRANDFATHERED_FAILURE.replace(
    "sweep::grandfathered_alpha", "sweep::novel_regression"
)


class RatchetHarness(unittest.TestCase):
    """Temp repo tree + fake `cargo`; every test runs the script black-box."""

    def setUp(self) -> None:
        self.root = Path(tempfile.mkdtemp(prefix="ratchet-test-"))
        self.addCleanup(shutil.rmtree, self.root, ignore_errors=True)
        (self.root / "scripts").mkdir()
        shutil.copy(SCRIPT, self.root / "scripts" / SCRIPT.name)
        self.baseline = self.root / "docs" / "source-port" / "failing-test-baseline.yaml"
        self.baseline.parent.mkdir(parents=True)
        self.baseline.write_text(BASELINE_FIXTURE)
        manifest = self.root / ".cache" / "generated" / "fixture" / "asset-manifest.json"
        manifest.parent.mkdir(parents=True)
        manifest.write_text("{}")
        self.bin = self.root / "bin"
        self.bin.mkdir()

    def fake_cargo(self, stdout: str, exit_code: int, stderr: str = "") -> None:
        cargo = self.bin / "cargo"
        script = "#!/bin/sh\n"
        if stdout:
            script += f"cat <<'WILDSIDE_EOF'\n{stdout}WILDSIDE_EOF\n"
        if stderr:
            script += f"cat >&2 <<'WILDSIDE_EOF'\n{stderr}WILDSIDE_EOF\n"
        script += f"exit {exit_code}\n"
        cargo.write_text(script)
        cargo.chmod(cargo.stat().st_mode | stat.S_IXUSR)

    def run_ratchet(self, *args: str) -> subprocess.CompletedProcess[str]:
        env = dict(os.environ, PATH=f"{self.bin}{os.pathsep}{os.environ['PATH']}")
        return subprocess.run(
            [sys.executable, str(self.root / "scripts" / SCRIPT.name), *args],
            capture_output=True,
            text=True,
            env=env,
        )

    # -- did-not-run runs must fail closed ------------------------------------

    def test_compile_error_is_not_an_empty_failure_set(self) -> None:
        # cargo exits 101 on its own build errors too; stdout has no
        # `test result:` line, so the run did not happen.
        self.fake_cargo("", 101, stderr="error[E0308]: mismatched types\n")

        proc = self.run_ratchet()

        self.assertEqual(proc.returncode, 1)
        self.assertIn("did not run", proc.stderr)
        self.assertEqual(self.baseline.read_text(), BASELINE_FIXTURE)

    def test_update_on_a_compile_error_never_rewrites_the_baseline(self) -> None:
        self.fake_cargo("", 101, stderr="error[E0308]: mismatched types\n")

        proc = self.run_ratchet("--update")

        self.assertEqual(proc.returncode, 1)
        self.assertIn("did not run", proc.stderr)
        self.assertEqual(self.baseline.read_text(), BASELINE_FIXTURE)

    def test_signal_killed_harness_is_did_not_run(self) -> None:
        # A run truncated mid-suite: partial stdout, no `test result:` line,
        # non-libtest exit status.
        self.fake_cargo("running 3 tests\ntest sweep::other ... ok\n", 143)

        proc = self.run_ratchet("--update")

        self.assertEqual(proc.returncode, 1)
        self.assertIn("did not run", proc.stderr)
        self.assertEqual(self.baseline.read_text(), BASELINE_FIXTURE)

    def test_unexpected_exit_code_with_result_line_is_still_did_not_run(self) -> None:
        # Exit statuses other than 0/101 are a harness problem even when stdout
        # looks complete; trust the status, not the text.
        self.fake_cargo(LIBTEST_ALL_GREEN, 143)

        proc = self.run_ratchet("--update")

        self.assertEqual(proc.returncode, 1)
        self.assertEqual(self.baseline.read_text(), BASELINE_FIXTURE)

    def test_did_not_run_reports_the_cargo_exit_status(self) -> None:
        self.fake_cargo("", 101, stderr="error[E0308]: mismatched types\n")

        proc = self.run_ratchet()

        self.assertIn("exit status 101", proc.stderr)
        self.assertIn("error[E0308]", proc.stderr)

    # -- runs that happened ---------------------------------------------------

    def test_green_run_reports_subset_and_keeps_baseline(self) -> None:
        self.fake_cargo(LIBTEST_ALL_GREEN, 0)

        proc = self.run_ratchet()

        self.assertEqual(proc.returncode, 0)
        self.assertIn("failing tests: 0 (subset of baseline 2)", proc.stdout)
        self.assertEqual(self.baseline.read_text(), BASELINE_FIXTURE)

    def test_update_shrinks_baseline_to_current_failures_only(self) -> None:
        self.fake_cargo(LIBTEST_ONE_GRANDFATHERED_FAILURE, 101)

        proc = self.run_ratchet("--update")

        self.assertEqual(proc.returncode, 0)
        self.assertIn("lowered: 2 -> 1", proc.stdout)
        text = self.baseline.read_text()
        self.assertIn("header comment must survive", text)
        self.assertIn("  - sweep::grandfathered_alpha\n", text)
        self.assertNotIn("grandfathered_beta", text)

    def test_novel_failure_blocks_and_keeps_baseline_even_under_update(self) -> None:
        self.fake_cargo(LIBTEST_NOVEL_FAILURE, 101)

        proc = self.run_ratchet("--update")

        self.assertEqual(proc.returncode, 1)
        self.assertIn("sweep::novel_regression", proc.stderr)
        self.assertEqual(self.baseline.read_text(), BASELINE_FIXTURE)

    def test_two_test_result_lines_refuse_the_single_binary_parse(self) -> None:
        two_targets = LIBTEST_ALL_GREEN + "\n" + LIBTEST_ONE_GRANDFATHERED_FAILURE
        self.fake_cargo(two_targets, 101)

        proc = self.run_ratchet("--update")

        self.assertEqual(proc.returncode, 2)
        self.assertIn("single-binary parse assumption", proc.stderr)
        self.assertEqual(self.baseline.read_text(), BASELINE_FIXTURE)

    def test_missing_asset_bundle_skips_without_touching_baseline(self) -> None:
        shutil.rmtree(self.root / ".cache")
        self.fake_cargo("", 101)

        proc = self.run_ratchet("--update")

        self.assertEqual(proc.returncode, 0)
        self.assertIn("skipped", proc.stdout)
        self.assertEqual(self.baseline.read_text(), BASELINE_FIXTURE)


if __name__ == "__main__":
    unittest.main()
