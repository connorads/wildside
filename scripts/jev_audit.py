"""Read-only readiness catalogue joined to the authoritative live backlog."""

import argparse
from collections import Counter
import json
from pathlib import Path
import re

ROOT = Path(__file__).resolve().parents[1]
READINESS = {"needs_route", "needs_comparison", "human_device", "inactive_path", "ready_replay"}


def live_rows(root):
    rows = {}
    for name in ("backlog.md", "backlog-awaiting-human.md"):
        for line in (root / "docs/source-port" / name).read_text().splitlines():
            cells = [cell.strip() for cell in line.strip().strip("|").split("|")]
            if not cells or not cells[0].isdigit():
                continue
            issue = int(cells[0])
            if cells[1] == "[f]":
                continue
            if len(cells) not in (6, 8) or cells[1] not in ("[ ]", "[~]", "[?]", "[b]"):
                raise ValueError(f"unexpected live backlog row #{issue}")
            if issue in rows:
                raise ValueError(f"duplicate backlog issue #{issue}")
            rows[issue] = {"status": cells[1], "priority": cells[2], "system": cells[3],
                           "target": cells[4] if len(cells) == 8 else "unknown"}
    return rows


def audit(root=ROOT):
    backlog = live_rows(root)
    catalogue = json.loads((root / "docs/source-port/jev-cases.json").read_text())
    seen = set()
    result = []
    for entry in catalogue:
        issue = entry["issue"]
        if type(issue) is not int or issue in seen:
            raise ValueError(f"invalid or duplicate catalogue issue {issue}")
        seen.add(issue)
        if issue not in backlog:
            raise ValueError(f"stale catalogue issue #{issue}")
        if entry.get("readiness") not in READINESS:
            raise ValueError(f"invalid readiness for #{issue}")
        for field in ("active_path", "route", "next_action", "scope"):
            if not isinstance(entry.get(field), str) or not entry[field].strip():
                raise ValueError(f"missing {field} for #{issue}")
        if not entry.get("evidence"):
            raise ValueError(f"missing evidence for #{issue}")
        for reference in entry["evidence"]:
            path = re.split(r"[#:]", reference, maxsplit=1)[0]
            relative = Path(path)
            if relative.is_absolute() or ".." in relative.parts or not (root / relative).is_file():
                raise ValueError(f"missing or invalid evidence for #{issue}: {reference}")
        result.append(entry | backlog[issue])
    if missing := backlog.keys() - seen:
        raise ValueError(f"missing catalogue issues: {sorted(missing)}")
    return sorted(result, key=lambda row: (row["priority"], row["issue"]))


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--json", action="store_true")
    args = parser.parse_args()
    try:
        rows = audit()
    except (ValueError, KeyError, OSError) as error:
        parser.exit(1, f"audit failed: {error}\n")
    if args.json:
        print(json.dumps(rows, indent=2))
    else:
        print(f"{len(rows)} unresolved issues; frozen lanes excluded. Readiness is not a proof verdict.")
        print(", ".join(f"{count} {name}" for name, count in sorted(Counter(r["readiness"] for r in rows).items())))
        for row in rows:
            print(f"#{row['issue']} {row['status']} {row['readiness']}: {row['next_action']}")


if __name__ == "__main__":
    main()
