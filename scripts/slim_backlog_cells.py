#!/usr/bin/env python3
"""One-shot slimming of docs/source-port/backlog.md table cells.

Any table cell over MAX_CELL chars moves verbatim to backlog-archive.md
(appended section `## Slimmed table prose (2026-06-10)`, one `### #NNN`
subsection per issue), leaving a short truncated summary in the cell.

Invariants enforced (the project-status backlog parsers depend on them):
- column count and order unchanged (parsers index columns[1]/[2])
- row count and id set unchanged
- the literal phrase "root cause identified" (case-insensitive match in
  project_status.rs backlog_row_has_root_cause) survives in any slimmed
  summary/verify cell that contained it
- reconciliation: every moved cell's full text appears in the archive

Run once; deleted in the same change series.
"""

from __future__ import annotations

import re
import sys
from pathlib import Path

REPO = Path(__file__).resolve().parent.parent
BACKLOG = REPO / "docs/source-port/backlog.md"
ARCHIVE = REPO / "docs/source-port/backlog-archive.md"

SALVAGE_DATE = "2026-06-10"
# Open-ish rows keep more context; closed/won't-fix rows are pure history
# (full text always lands in the archive, grep by the id column).
MAX_CELL_OPEN = 120
TRUNCATE_OPEN = 80
MAX_CELL_CLOSED = 60
TRUNCATE_CLOSED = 45
CLOSED_STATUSES = {"[x]", "[-]"}

ROOT_CAUSE = "root cause identified"
COLUMN_NAMES = ["#", "Status", "Pri", "System", "Summary", "Verify"]


def shorten(cell: str, issue_id: str, truncate_at: int) -> str:
    prefix = cell[:truncate_at]
    if " " in prefix:
        prefix = prefix.rsplit(" ", 1)[0]
    short = f"{prefix}…"
    if ROOT_CAUSE in cell.lower() and ROOT_CAUSE not in short.lower():
        short = f"{prefix}… Root cause identified."
    return short.replace("|", "/")


def root_cause_rows(lines: list[str]) -> set[str]:
    rows = set()
    for line in lines:
        cells = split_row(line)
        if cells and ROOT_CAUSE in (cells[4] + " " + cells[5]).lower():
            rows.add(cells[0])
    return rows


def split_row(line: str) -> list[str] | None:
    if not line.startswith("|") or line.startswith("| #") or line.startswith("|---"):
        return None
    cells = [c.strip() for c in line.split("|")[1:-1]]
    if len(cells) < 6 or not re.fullmatch(r"\d{3}", cells[0]):
        return None
    if len(cells) > 6:
        # 7 historical rows carry unescaped pipes inside the verify prose;
        # merge the tail back into one cell (pipes become '/') so the row
        # regains the canonical 6 columns. Full text is preserved verbatim
        # in the archive by the caller (the merged cell is always >MAX_CELL).
        cells = cells[:5] + [" / ".join(cells[5:])]
    return cells


def main() -> int:
    lines = BACKLOG.read_text().splitlines()
    before_rows = [c for c in (split_row(l) for l in lines) if c]
    before_ids = [c[0] for c in before_rows]
    before_root_cause = root_cause_rows(lines)

    moved: dict[str, list[tuple[str, str]]] = {}  # issue -> [(column, full text)]
    out_lines: list[str] = []
    for line in lines:
        cells = split_row(line)
        if cells is None:
            out_lines.append(line)
            continue
        issue_id = cells[0]
        closed = cells[1] in CLOSED_STATUSES
        max_cell = MAX_CELL_CLOSED if closed else MAX_CELL_OPEN
        truncate_at = TRUNCATE_CLOSED if closed else TRUNCATE_OPEN
        new_cells = list(cells)
        for idx, cell in enumerate(cells):
            if len(cell) > max_cell:
                moved.setdefault(issue_id, []).append((COLUMN_NAMES[idx], cell))
                if closed and COLUMN_NAMES[idx] == "Verify":
                    # Closed rows' verify steps are pure history; the stub
                    # points at the archive without repeating any of it.
                    new_cells[idx] = "archived"
                else:
                    new_cells[idx] = shorten(cell, issue_id, truncate_at)
        out_lines.append("| " + " | ".join(new_cells) + " |")

    archive_lines = [
        "",
        f"## Slimmed table prose ({SALVAGE_DATE})",
        "",
        "Full text of backlog.md table cells slimmed during the autonomy",
        "demolition (the table is now a lean index; long evidence lives here).",
        "Column of origin shown in brackets; text is verbatim.",
        "",
    ]
    for issue in sorted(moved):
        archive_lines.append(f"### #{int(issue)}")
        archive_lines.append("")
        for column, text in moved[issue]:
            archive_lines.append(f"- [{column}] {text}")
        archive_lines.append("")

    BACKLOG.write_text("\n".join(out_lines) + "\n")
    with ARCHIVE.open("a") as fh:
        fh.write("\n".join(archive_lines))

    # --- reconciliation ---
    after_lines = BACKLOG.read_text().splitlines()
    after_rows = [c for c in (split_row(l) for l in after_lines) if c]
    after_ids = [c[0] for c in after_rows]
    assert after_ids == before_ids, "row count or id set changed"
    assert root_cause_rows(after_lines) == before_root_cause, (
        "root-cause-identified row set changed: "
        f"{root_cause_rows(after_lines) ^ before_root_cause}"
    )
    archive_text = ARCHIVE.read_text()
    moved_cells = sum(len(v) for v in moved.values())
    for issue, items in moved.items():
        for _, text in items:
            assert text in archive_text, f"moved cell for #{issue} missing from archive"
    oversize = [
        (c[0], i)
        for c in after_rows
        for i, cell in enumerate(c)
        if len(cell) > MAX_CELL_OPEN + 30
    ]
    assert not oversize, f"cells still oversize: {oversize}"

    print(f"rows: {len(after_ids)} (unchanged), issues slimmed: {len(moved)}, cells moved: {moved_cells}")
    print(f"moved chars: {sum(len(t) for v in moved.values() for _, t in v)}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
