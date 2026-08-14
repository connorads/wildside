#!/usr/bin/env python3
"""Parser contracts for the multi-target workspace failure ratchet (#445)."""

import importlib.util
import unittest
from pathlib import Path

SCRIPT = Path(__file__).with_name("workspace_failing_test_ratchet.py")
SPEC = importlib.util.spec_from_file_location("workspace_ratchet", SCRIPT)
assert SPEC and SPEC.loader
MODULE = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(MODULE)


class ParseLibtestFailuresTests(unittest.TestCase):
    def test_parses_failures_from_multiple_targets(self) -> None:
        output = """\
failures:
---- tests::one stdout ----
panic
failures:
    tests::one
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
failures:
---- parity::two stdout ----
panic
failures:
    parity::two
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out
"""
        self.assertEqual(
            MODULE.parse_libtest_failures("wildside-web", output),
            {"wildside-web::tests::one", "wildside-web::parity::two"},
        )

    def test_green_multi_target_output_is_empty(self) -> None:
        output = "\n".join(
            [
                "test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out",
                "test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out",
            ]
        )
        self.assertEqual(MODULE.parse_libtest_failures("wildside-web", output), set())

    def test_missing_completed_target_fails_closed(self) -> None:
        self.assertIsNone(MODULE.parse_libtest_failures("wildside-web", "running 10 tests"))


if __name__ == "__main__":
    unittest.main()
