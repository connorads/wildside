#!/usr/bin/env python3
"""Gate commit-message shape: subject line, blank line, and a real body.

A forcing function, not a quality check: it can demand that a body exists,
not that the body actually records the why or the dead ends — loop.md step 7
owns that standard. Merge/Revert/fixup!/squash! subjects are exempt because
their messages are tool-generated.

Usage: check_commit_message.py <commit-msg-file> (the git commit-msg hook in
.hk-hooks/commit-msg). Tests: scripts/test_check_commit_message.py (runs in
verify-slice-fast via the script-tests discovery task).
"""

from __future__ import annotations

import sys
from pathlib import Path

MIN_BODY_CHARS = 60

EXEMPT_PREFIXES = ("Merge ", "Revert ", "fixup!", "squash!")


def message_problems(message: str) -> list[str]:
    lines = [
        line
        for line in message.splitlines()
        if not line.startswith("#")  # commented template/hook lines
    ]
    while lines and not lines[0].strip():
        lines.pop(0)
    if not lines:
        return ["empty commit message"]
    subject = lines[0].strip()
    if subject.startswith(EXEMPT_PREFIXES):
        return []
    problems = []
    if len(lines) > 1 and lines[1].strip():
        problems.append("second line must be blank (subject / blank / body)")
    body_lines = [line for line in lines[2:] if line.strip()]
    if not body_lines:
        problems.append("commit body is missing")
    body_chars = sum(len(line.strip()) for line in body_lines)
    if body_lines and body_chars < MIN_BODY_CHARS:
        problems.append(
            f"commit body is {body_chars} chars (min {MIN_BODY_CHARS}); record the why"
        )
    return problems


def main(argv: list[str]) -> int:
    if len(argv) != 1:
        print("usage: check_commit_message.py <commit-msg-file>", file=sys.stderr)
        return 2
    problems = message_problems(Path(argv[0]).read_text())
    if problems:
        for problem in problems:
            print(f"commit-msg: {problem}", file=sys.stderr)
        print(
            "commit-msg: write subject / blank line / body carrying the why and dead"
            " ends (loop.md step 7); Merge/Revert/fixup!/squash! are exempt",
            file=sys.stderr,
        )
        return 1
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
