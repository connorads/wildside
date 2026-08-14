#!/usr/bin/env python3
"""Contract tests for the exact-name core test exclusions (#444)."""

import importlib.util
import unittest
from pathlib import Path

SCRIPT = Path(__file__).with_name("core_test_exclusions.py")
SPEC = importlib.util.spec_from_file_location("core_test_exclusions", SCRIPT)
assert SPEC and SPEC.loader
MODULE = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(MODULE)


class CoreTestExclusionsTests(unittest.TestCase):
    def test_legacy_patterns_produce_sorted_unique_names(self) -> None:
        tests = [
            "tests::ordinary",
            "tests::published_tas_route",
            "tests::demo_route",
            "tests::snapshot_real",
            "tests::published_tas_demo_route",
        ]
        self.assertEqual(
            MODULE.pattern_exclusions(tests),
            [
                "tests::demo_route",
                "tests::published_tas_demo_route",
                "tests::published_tas_route",
                "tests::snapshot_real",
            ],
        )

    def test_exact_filter_validation_rejects_substring_collisions(self) -> None:
        with self.assertRaisesRegex(ValueError, "must select exactly one"):
            MODULE.validate_exact_filters(
                ["tests::demo_route", "tests::demo_route_extended"],
                ["tests::demo_route"],
            )


if __name__ == "__main__":
    unittest.main()
