#!/usr/bin/env python3
"""One-shot splitter: backlog.md -> table-only index + cold evidence archive.

Why: docs/source-port/backlog.md grew to ~7.4k lines / ~143k tokens, dominated by
evidence prose, while the only machine-consumed payload is the 273-row issue table
(four parsers scan `|`-prefixed lines; none read the prose). This splits the file
into a lean table-only working index and an append-only `backlog-archive.md` keyed
by `#NNN`, WITHOUT touching any table row (parser-compatibility is the hard gate).

The partition is lossless and proven mechanically (see assertions below):
  - KEPT-IN-PLACE: title + intro + legend (lines 1..LEGEND_END) stay in backlog.md.
  - TABLE stream:  every `|`-prefixed line is table content (header/sep/row). Rows
                   are parsed, sorted by 3-digit id, de-fragmented under one header.
  - ARCHIVE stream: every other line after the legend goes to the archive verbatim,
                   in original document order (no re-attribution).

Run from repo root: `python scripts/split_backlog.py`.
Re-runnable: it reads backlog.md and overwrites both outputs deterministically.
Kept in-tree as the auditable record of how the partition was produced.
"""

from __future__ import annotations

import re
import sys
from pathlib import Path

BACKLOG = Path("docs/source-port/backlog.md")
ARCHIVE = Path("docs/source-port/backlog-archive.md")

# Legend ends at the last instructional bullet, just before the first dated
# "Live playtest note" log block. Anchor on content, not a magic line number,
# so the script survives edits above the table.
FIRST_ARCHIVE_ANCHOR = "Live playtest note"

TABLE_HEADER = "| # | Status | Pri | System | Summary | Verify |"
TABLE_SEP = "|-----|--------|-----|--------|---------|--------|"

ROW_ID_RE = re.compile(r"^\|\s*(\d{3})\s*\|")

ARCHIVE_POINTER = [
    "",
    "> **Evidence lives in [`backlog-archive.md`](./backlog-archive.md).**",
    "> This file is the table index only. Per-issue investigation notes, follow-ups,",
    "> rejected hypotheses, and dated playtest history are cold storage — grep the",
    "> archive by `#NNN` (e.g. `grep -n '#238' backlog-archive.md`) for an issue's",
    "> full history. When closing an issue, flip its status here and append closing",
    "> evidence to the archive.",
    "",
]

ARCHIVE_HEADER = [
    "# Playtest Backlog — Evidence Archive",
    "",
    "Append-only cold storage for backlog evidence. The live index is",
    "[`backlog.md`](./backlog.md); this file holds the investigation prose that used",
    "to be interleaved with the table: follow-ups, notes, rejected hypotheses,",
    "investigation write-ups, and dated playtest logs.",
    "",
    "Keyed by issue id — grep `#NNN` (e.g. `grep -n '#238' backlog-archive.md`) to",
    "recover an issue's full history. Order is the original `backlog.md` document",
    "order at split time; nothing here is re-attributed or deduplicated.",
    "",
    "---",
    "",
]


def main() -> int:
    src = BACKLOG.read_text().splitlines(keepends=False)

    # Locate the legend/archive boundary by content anchor.
    legend_end = next(
        (i for i, ln in enumerate(src) if ln.startswith(FIRST_ARCHIVE_ANCHOR)),
        None,
    )
    if legend_end is None:
        print(f"FATAL: anchor {FIRST_ARCHIVE_ANCHOR!r} not found", file=sys.stderr)
        return 1

    kept = src[:legend_end]
    tail = src[legend_end:]

    # Trim trailing blank lines from the kept intro/legend so the pointer/table
    # sit with predictable spacing.
    while kept and kept[-1].strip() == "":
        kept.pop()

    table_rows: list[str] = []
    archive_lines: list[str] = []
    table_line_count = 0  # header + sep + rows, for the conservation tally

    for ln in tail:
        if ln.startswith("|"):
            table_line_count += 1
            if ln == TABLE_HEADER or ln.startswith("|---") or ln.startswith("|-"):
                continue  # header/separator dropped; we re-emit canonical ones
            m = ROW_ID_RE.match(ln)
            if not m:
                print(f"FATAL: pipe line is not a valid row: {ln!r}", file=sys.stderr)
                return 1
            table_rows.append(ln)
        else:
            archive_lines.append(ln)

    # ---- Reconciliation check 2: no row lost ---------------------------------
    ids = [ROW_ID_RE.match(r).group(1) for r in table_rows]
    if len(ids) != len(set(ids)):
        dupes = sorted({i for i in ids if ids.count(i) > 1})
        print(f"FATAL: duplicate row ids: {dupes}", file=sys.stderr)
        return 1
    expected = {f"{n:03d}" for n in range(1, 274)}
    if set(ids) != expected:
        missing = sorted(expected - set(ids))
        extra = sorted(set(ids) - expected)
        print(f"FATAL: id set mismatch missing={missing} extra={extra}", file=sys.stderr)
        return 1

    # Sort by numeric id; row text is preserved verbatim (no edits).
    table_rows.sort(key=lambda r: int(ROW_ID_RE.match(r).group(1)))

    # ---- Assemble outputs ----------------------------------------------------
    new_backlog = (
        kept
        + ARCHIVE_POINTER
        + [TABLE_HEADER, TABLE_SEP]
        + table_rows
        + [""]
    )
    new_archive = ARCHIVE_HEADER + archive_lines
    # Normalise trailing blank lines to a single newline.
    while new_archive and new_archive[-1].strip() == "":
        new_archive.pop()

    # ---- Reconciliation check 3: no evidence lost (byte-for-byte) ------------
    # archive_lines (pre-header) must equal source-tail minus all pipe lines.
    expected_archive = [ln for ln in tail if not ln.startswith("|")]
    if archive_lines != expected_archive:
        print("FATAL: archive stream != non-pipe tail lines", file=sys.stderr)
        return 1

    # ---- Reconciliation check 1: line conservation --------------------------
    # Every input line is emitted exactly once across the two streams:
    #   kept(intro/legend, before blank trim) + table pipe lines + archive lines
    accounted = legend_end + table_line_count + len(archive_lines)
    if accounted != len(src):
        print(
            f"FATAL: line conservation failed: "
            f"kept={legend_end} table={table_line_count} "
            f"archive={len(archive_lines)} sum={accounted} src={len(src)}",
            file=sys.stderr,
        )
        return 1

    BACKLOG.write_text("\n".join(new_backlog) + "\n")
    ARCHIVE.write_text("\n".join(new_archive) + "\n")

    print(
        f"OK  src={len(src)}  kept={legend_end}  rows={len(table_rows)}  "
        f"table_pipe_lines={table_line_count}  archive={len(archive_lines)}"
    )
    print(f"  backlog.md  -> {len(new_backlog)} lines")
    print(f"  backlog-archive.md -> {len(new_archive)} lines")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
