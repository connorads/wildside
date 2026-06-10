#!/usr/bin/env python3
"""One-shot salvage of docs/source-port/frontier-state.yaml (JSON despite the
extension) before its deletion.

Partition rule: any entry in the salvage lists whose text mentions a backlog
issue (`#NNN`) is appended to docs/source-port/backlog-archive.md under a
single new section with one `### #NNN` subsection per issue (the archive has
no per-issue headings — grep-by-`#NNN` access works either way). Everything
else, plus the frontier bookkeeping scalars, goes to
docs/source-port/tas-era-ledger.md (historical, never loop-read).

Asserts per-list reconciliation (in == archived + ledgered) and that the
#290 frame_7800 full-TAS evidence lands under #290. Run once; deleted in the
same change series.
"""

from __future__ import annotations

import json
import re
import sys
from pathlib import Path

REPO = Path(__file__).resolve().parent.parent
FRONTIER = REPO / "docs/source-port/frontier-state.yaml"
ARCHIVE = REPO / "docs/source-port/backlog-archive.md"
LEDGER = REPO / "docs/source-port/tas-era-ledger.md"

SALVAGE_DATE = "2026-06-10"

# Lists (and scalar prose fields treated as one-entry lists) eligible for
# per-issue archive salvage.
SALVAGE_KEYS = [
    "rejected_hypotheses",
    "required_next_evidence",
    "authoritative_diffs",
    "advisory_diffs",
    "current_hypothesis",
    "current_blocker",
    "pre_divergence_windows",
    "verification_commands",
]

ISSUE_RE = re.compile(r"#(\d{1,3})\b")


def entry_text(entry) -> str:
    if isinstance(entry, str):
        return entry
    return json.dumps(entry, ensure_ascii=False)


def first_issue(text: str) -> str | None:
    match = ISSUE_RE.search(text)
    if not match:
        return None
    return f"{int(match.group(1)):03d}"


def main() -> int:
    state = json.loads(FRONTIER.read_text())
    frontier = state["active_frontier"]

    archived: dict[str, list[tuple[str, str]]] = {}  # issue -> [(key, text)]
    ledgered: dict[str, list[str]] = {}  # key -> [text]
    counts: dict[str, tuple[int, int, int]] = {}  # key -> (in, archived, ledgered)

    for key in SALVAGE_KEYS:
        value = frontier.get(key)
        if value is None:
            counts[key] = (0, 0, 0)
            continue
        entries = value if isinstance(value, list) else [value]
        n_arch = 0
        n_ledg = 0
        for entry in entries:
            text = entry_text(entry)
            issue = first_issue(text)
            if issue is not None:
                archived.setdefault(issue, []).append((key, text))
                n_arch += 1
            else:
                ledgered.setdefault(key, []).append(text)
                n_ledg += 1
        counts[key] = (len(entries), n_arch, n_ledg)
        assert len(entries) == n_arch + n_ledg, f"reconciliation failed for {key}"

    # Investigation attempt counters worth keeping (>=2 attempts = real signal).
    attempts = state.get("bug_investigation_attempts", {})
    for issue_id, count in sorted(attempts.items(), key=lambda kv: int(kv[0])):
        if int(count) >= 2:
            padded = f"{int(issue_id):03d}"
            archived.setdefault(padded, []).append(
                (
                    "bug_investigation_attempts",
                    f"{count} structured investigation attempts were logged against "
                    f"#{int(issue_id)} in the retired frontier-state attempt counter "
                    f"before {SALVAGE_DATE}; treat further attempts as 'instrument "
                    f"before another gameplay patch' territory.",
                )
            )

    # --- backlog-archive.md: one appended section, per-issue subsections ---
    lines = [
        "",
        f"## Salvaged frontier-state evidence ({SALVAGE_DATE})",
        "",
        "`docs/source-port/frontier-state.yaml` (the TAS-era frontier ledger) was",
        "retired in the autonomy demolition. Entries below are the issue-tagged",
        "evidence salvaged from it, grouped by backlog id; the untagged remainder",
        "lives in `docs/source-port/tas-era-ledger.md`. Source key shown in",
        "brackets. Nothing is re-attributed: text is verbatim.",
        "",
    ]
    for issue in sorted(archived):
        lines.append(f"### #{int(issue)}")
        lines.append("")
        for key, text in archived[issue]:
            lines.append(f"- [{key}] {text}")
        lines.append("")

    with ARCHIVE.open("a") as fh:
        fh.write("\n".join(lines))

    # --- tas-era-ledger.md: everything untagged + frontier bookkeeping ---
    ledger_lines = [
        f"# TAS-era frontier ledger (salvaged {SALVAGE_DATE})",
        "",
        "Historical record of the retired `frontier-state.yaml` — the working",
        "state of the published any-percent TAS verification frontier. The",
        "frontier closed replay-matched; issue-tagged evidence moved to",
        "`backlog-archive.md` (grep `#NNN`). This file is never loop-read.",
        "",
        "## Frontier bookkeeping",
        "",
        "```json",
        json.dumps(
            {
                "version": state.get("version"),
                "updated_at": state.get("updated_at"),
                "scope": state.get("scope"),
                "candidate_promotions": state.get("candidate_promotions"),
                "bug_investigation_attempts": attempts,
                "active_frontier_scalars": {
                    k: v
                    for k, v in frontier.items()
                    if k not in SALVAGE_KEYS
                },
            },
            indent=2,
        ),
        "```",
        "",
    ]
    for key in ["stuck_policy", "next_actions", "bootstrap"]:
        if key in state:
            ledger_lines.append(f"## {key}")
            ledger_lines.append("")
            ledger_lines.append("```json")
            ledger_lines.append(json.dumps(state[key], indent=2))
            ledger_lines.append("```")
            ledger_lines.append("")
    for key in SALVAGE_KEYS:
        if key not in ledgered:
            continue
        ledger_lines.append(f"## {key} (no issue tag)")
        ledger_lines.append("")
        for text in ledgered[key]:
            ledger_lines.append(f"- {text}")
        ledger_lines.append("")

    LEDGER.write_text("\n".join(ledger_lines))

    # --- reconciliation report + assertions ---
    print(f"{'key':<28} {'in':>4} {'archived':>9} {'ledgered':>9}")
    for key, (n_in, n_arch, n_ledg) in counts.items():
        print(f"{key:<28} {n_in:>4} {n_arch:>9} {n_ledg:>9}")

    archive_text = ARCHIVE.read_text()
    section_290 = archive_text.split("### #290", 1)
    assert len(section_290) == 2, "#290 section missing from backlog-archive.md"
    tail_290 = section_290[1].split("\n### ", 1)[0]
    assert "frame_7800" in tail_290, "frame_7800 evidence did not land under #290"
    print("OK: frame_7800 evidence is under #290 in backlog-archive.md")
    return 0


if __name__ == "__main__":
    sys.exit(main())
