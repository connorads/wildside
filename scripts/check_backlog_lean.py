#!/usr/bin/env python3
"""Pre-commit check: docs/source-port/backlog.md must stay a lean index.

The backlog is the loop's entry point — a session reads it whole to orient.
It was once 315KB of accreted prose; the 2026-06-10 autonomy demolition
slimmed it and this check stops the regrowth. Long evidence belongs in
backlog-archive.md under the issue's `#NNN`, not in table cells.

Fails (non-zero) if any table-row cell exceeds MAX_CELL chars, or the file
exceeds MAX_FILE bytes.
"""

from __future__ import annotations

import sys
from pathlib import Path

BACKLOG = Path(__file__).resolve().parent.parent / "docs/source-port/backlog.md"
MAX_CELL = 250
# The cell cap is the real prose guard (regrowth happens KB-per-row, not
# row-count). The file cap backstops it: ~40KB at the 2026-06-10 slim with
# 290 rows, plus headroom for ~30 organically-filed rows. When it trips,
# archive closed rows' remaining prose — do not raise the cap.
MAX_FILE = 48_000


def main() -> int:
    text = BACKLOG.read_text()
    problems: list[str] = []

    size = len(text.encode())
    if size > MAX_FILE:
        problems.append(
            f"backlog.md is {size} bytes (max {MAX_FILE}) — move prose to "
            "backlog-archive.md under the issue's #NNN"
        )

    for line_no, line in enumerate(text.splitlines(), 1):
        if not line.startswith("|"):
            continue
        for cell in line.split("|")[1:-1]:
            cell = cell.strip()
            if len(cell) > MAX_CELL:
                row_id = line.split("|")[1].strip()
                problems.append(
                    f"line {line_no} (row {row_id}): cell is {len(cell)} chars "
                    f"(max {MAX_CELL}) — move the prose to backlog-archive.md "
                    f"under #{row_id} and leave a short summary"
                )

    if problems:
        for problem in problems:
            print(f"error: {problem}", file=sys.stderr)
        return 1

    print(f"backlog.md lean: {size} bytes, all cells <= {MAX_CELL} chars")
    return 0


if __name__ == "__main__":
    sys.exit(main())
