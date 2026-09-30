#!/usr/bin/env python3

import importlib.util
import tempfile
import unittest
from pathlib import Path

SCRIPT = Path(__file__).with_name("check_clippy_conf_sync.py")
SPEC = importlib.util.spec_from_file_location("check_clippy_conf_sync", SCRIPT)
assert SPEC and SPEC.loader
CHECK = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(CHECK)

ROOT = 'too-many-arguments-threshold = 7\ndisallowed-names = ["foo"]\n'


class ClippyConfSyncTests(unittest.TestCase):
    def check(self, crate_conf: str) -> list[str]:
        with tempfile.TemporaryDirectory() as tmp:
            repo = Path(tmp)
            (repo / "clippy.toml").write_text(ROOT)
            crate = repo / "crates" / "demo"
            crate.mkdir(parents=True)
            (crate / "clippy.toml").write_text(crate_conf)
            return CHECK.check(repo)

    def test_repeated_keys_plus_crate_only_keys_pass(self) -> None:
        self.assertEqual(
            self.check(ROOT + 'disallowed-types = [{ path = "std::fs::File" }]\n'), []
        )

    def test_missing_root_key_fails(self) -> None:
        problems = self.check('disallowed-names = ["foo"]\n')
        self.assertEqual(len(problems), 1)
        self.assertIn("missing root key `too-many-arguments-threshold`", problems[0])
        self.assertIn("crates/demo/clippy.toml", problems[0])

    def test_changed_root_value_fails(self) -> None:
        problems = self.check(
            'too-many-arguments-threshold = 9\ndisallowed-names = ["foo"]\n'
        )
        self.assertEqual(len(problems), 1)
        self.assertIn("`too-many-arguments-threshold` is 9, root has 7", problems[0])

    def test_no_crate_files_passes(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            repo = Path(tmp)
            (repo / "clippy.toml").write_text(ROOT)
            self.assertEqual(CHECK.check(repo), [])

    def test_live_repo_is_in_sync(self) -> None:
        self.assertEqual(CHECK.check(SCRIPT.resolve().parent.parent), [])


if __name__ == "__main__":
    unittest.main()
