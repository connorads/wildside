#!/usr/bin/env python3

import importlib.util
import unittest
from pathlib import Path

SCRIPT = Path(__file__).with_name("check_commit_message.py")
SPEC = importlib.util.spec_from_file_location("check_commit_message", SCRIPT)
assert SPEC and SPEC.loader
CHECK = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(CHECK)


class CommitMessageShapeTests(unittest.TestCase):
    def assert_accepted(self, message: str) -> None:
        self.assertEqual(CHECK.message_problems(message), [])

    def assert_rejected(self, message: str, fragment: str) -> None:
        problems = CHECK.message_problems(message)
        self.assertTrue(
            any(fragment in problem for problem in problems),
            f"expected a problem containing {fragment!r}, got {problems!r}",
        )

    def test_subject_blank_body_is_accepted(self) -> None:
        self.assert_accepted(
            "fix(x): subject\n\n"
            "A body long enough to explain the why of this change in detail.\n"
        )

    def test_subject_only_is_rejected(self) -> None:
        self.assert_rejected("fix(x): subject only\n", "body is missing")

    def test_missing_blank_separator_is_rejected(self) -> None:
        self.assert_rejected(
            "fix(x): subject\nbody crammed straight onto the second line here\n",
            "second line must be blank",
        )

    def test_short_body_is_rejected(self) -> None:
        self.assert_rejected("fix(x): subject\n\nshort\n", "min 60")

    def test_tool_generated_subjects_are_exempt(self) -> None:
        self.assert_accepted("Merge branch 'main' into feature\n")
        self.assert_accepted('Revert "an earlier change"\n')
        self.assert_accepted("fixup! fix(x): subject\n")
        self.assert_accepted("squash! fix(x): subject\n")

    def test_comment_lines_are_ignored(self) -> None:
        self.assert_rejected(
            "fix(x): subject\n\n# only a template comment\n", "body is missing"
        )

    def test_empty_message_is_rejected(self) -> None:
        self.assert_rejected("\n\n", "empty commit message")


if __name__ == "__main__":
    unittest.main()
