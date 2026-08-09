#!/usr/bin/env python3
"""Downward-only 68000 conformance ratchet for the pinned r68k core (#447).

`verify-cpu-vectors` replays the two published SingleStepTests vector sets
through `r68k-emu` and reports, per opcode file, how many vectors it fails on
state and on cycle count. r68k is a pinned third-party core, so most of that
failure set will not shrink by fixing r68k — the point of the ratchet is that it
may never GROW. A new failure means one of:

- the harness regressed (a change to state injection, comparison, or decoding);
- the r68k pin moved and the new revision is worse; or
- the vector pin moved, which is a different measurement entirely and is
  rejected outright rather than silently absorbed.

Provenance is part of the baseline, not decoration: both vector-repo commits,
the r68k git rev read straight out of `crates/wildside-md/Cargo.toml`, and the
per-file vector cap. If any of them differs from the run, the script refuses to
compare or shrink — the numbers would not be about the same thing.

Modes:
  (default, check): exit non-zero iff any (set, file) fails more vectors than the
                    baseline allows; a subset is reported and exits 0.
  --update:         lower the baseline where the run improved on it. Never
                    raises. A regression still exits non-zero.
  --reseed:         rewrite provenance AND counts wholesale from the run. The
                    deliberate act a provenance change demands — a vector or
                    r68k pin bump is a new measurement, and pretending otherwise
                    would launder a regression through the baseline. Review the
                    diff.

Deliberately NOT an hk pre-commit step and NOT part of `verify-slice-fast`: it
needs ~600 MB of downloaded vectors and minutes of wall time per run, so it is a
periodic gate like `sweep`, invoked as `mise run verify-cpu-vectors-ratchet`.

A run that did not happen must never rewrite the baseline (the lesson from
`failing_test_ratchet.py`, #443): a non-zero exit, unparsable stdout, or a
report without `completed: true` is an error, not an empty failure set.
"""

from __future__ import annotations

import json
import re
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
BASELINE = ROOT / "docs/source-port/cpu-vector-baseline.yaml"
MD_MANIFEST = ROOT / "crates/wildside-md/Cargo.toml"
VECTOR_DIRS = {
    "680x0": ROOT / ".cache/references/singlesteptests-680x0",
    "m68000": ROOT / ".cache/references/singlesteptests-m68000",
}

# `<key>: <value>` inside the provenance block.
PROVENANCE_RE = re.compile(r"^\s{2}(?P<key>\w+):\s*(?P<value>\S+)\s*$", re.MULTILINE)
# The `files:` block: `  <set>/<file>: [<state_fail>, <cycle_fail>]`.
FILES_BLOCK_RE = re.compile(r"^files:\n(?P<body>(?:[ \t]+\S.*\n?)*)", re.MULTILINE)
FILE_ROW_RE = re.compile(
    r"^\s+(?P<key>[^\s:]+):\s*\[\s*(?P<state>\d+)\s*,\s*(?P<cycle>\d+)\s*\]\s*$",
    re.MULTILINE,
)
R68K_REV_RE = re.compile(r'r68k-emu\s*=\s*\{[^}]*\brev\s*=\s*"(?P<rev>[0-9a-f]+)"', re.DOTALL)

Counts = dict[str, tuple[int, int]]


def r68k_rev() -> str:
    match = R68K_REV_RE.search(MD_MANIFEST.read_text())
    if not match:
        print(f"error: no r68k-emu `rev` in {MD_MANIFEST}", file=sys.stderr)
        raise SystemExit(2)
    return match.group("rev")


def vectors_present() -> bool:
    return all(path.is_dir() for path in VECTOR_DIRS.values())


def run_verifier() -> dict | None:
    """Run the verifier and return its report, or `None` if it did not run."""
    proc = subprocess.run(
        [
            "cargo",
            "run",
            "-q",
            "--release",
            "-p",
            "wildside-cli",
            "--",
            "verify-cpu-vectors",
            "--json",
        ],
        cwd=ROOT,
        capture_output=True,
        text=True,
    )
    if proc.returncode != 0:
        print(proc.stderr.strip()[-2000:], file=sys.stderr)
        return None
    try:
        report = json.loads(proc.stdout)
    except json.JSONDecodeError:
        return None
    if not isinstance(report, dict) or report.get("completed") is not True:
        return None
    return report


def run_counts(report: dict) -> Counts:
    counts: Counts = {}
    for entry in report["sets"]:
        label = entry["set"]
        for name, file_report in entry["files"].items():
            state_fail = file_report["vectors"] - file_report["state_pass"]
            cycle_fail = file_report["vectors"] - file_report["cycle_pass"]
            counts[f"{label}/{name}"] = (state_fail, cycle_fail)
    return counts


def run_provenance(report: dict) -> dict[str, str]:
    limit = report.get("per_file_limit")
    provenance = {
        "per_file_limit": "all" if limit is None else str(limit),
        "r68k_rev": r68k_rev(),
    }
    for entry in report["sets"]:
        key = "harte_commit" if entry["set"] == "680x0" else "mame_commit"
        provenance[key] = entry["commit"] or "unknown"
    return provenance


def read_baseline() -> tuple[dict[str, str], Counts]:
    text = BASELINE.read_text()
    files_block = FILES_BLOCK_RE.search(text)
    if not files_block:
        print(f"error: {BASELINE} has no `files:` block", file=sys.stderr)
        raise SystemExit(2)
    counts: Counts = {
        match.group("key"): (int(match.group("state")), int(match.group("cycle")))
        for match in FILE_ROW_RE.finditer(files_block.group("body"))
    }
    provenance = {
        match.group("key"): match.group("value")
        for match in PROVENANCE_RE.finditer(text[: files_block.start()])
    }
    return provenance, counts


def write_baseline(counts: Counts, provenance: dict[str, str] | None = None) -> None:
    text = BASELINE.read_text()
    if provenance is not None:
        for key, value in provenance.items():
            text = re.sub(
                rf"^(\s{{2}}{re.escape(key)}:\s*)\S+\s*$",
                rf"\g<1>{value}",
                text,
                count=1,
                flags=re.MULTILINE,
            )
    body = "".join(
        f"  {key}: [{state}, {cycle}]\n" for key, (state, cycle) in sorted(counts.items())
    )
    BASELINE.write_text(FILES_BLOCK_RE.sub(lambda _m: f"files:\n{body}", text, count=1))


def check_provenance(baseline: dict[str, str], current: dict[str, str]) -> bool:
    drift = {
        key: (baseline.get(key), value)
        for key, value in current.items()
        if baseline.get(key) != value
    }
    if not drift:
        return True
    print(
        "error: the run does not measure what the baseline recorded, so it can "
        "neither be compared nor shrunk. If the change is intended, re-seed "
        "deliberately with `--reseed` and review the diff:",
        file=sys.stderr,
    )
    for key, (want, got) in sorted(drift.items()):
        print(f"    {key}: baseline {want} != run {got}", file=sys.stderr)
    return False


def main(argv: list[str]) -> int:
    update = "--update" in argv[1:]
    reseed = "--reseed" in argv[1:]

    if not vectors_present():
        print(
            "cpu-vector ratchet: skipped (no SingleStepTests checkouts under "
            ".cache/references; run `mise run bootstrap-cpu-vectors`). A run "
            "that cannot happen must never shrink the baseline."
        )
        return 0

    report = run_verifier()
    if report is None:
        print(
            "error: verify-cpu-vectors did not run to completion (non-zero exit, "
            "unparsable output, or no `completed` marker); refusing to compare "
            "or shrink the baseline against a run that did not happen",
            file=sys.stderr,
        )
        return 1

    current = run_counts(report)
    provenance = run_provenance(report)

    if reseed:
        write_baseline(current, provenance)
        print(
            f"cpu-vector baseline RESEEDED from this run: {len(current)} opcode files, "
            "provenance rewritten. Review the diff — this discards the previous "
            "downward-only history."
        )
        return 0

    baseline_provenance, baseline = read_baseline()
    if not check_provenance(baseline_provenance, provenance):
        return 1

    regressions = []
    for key, (state, cycle) in sorted(current.items()):
        allowed = baseline.get(key)
        if allowed is None:
            regressions.append(f"{key}: not in the baseline ({state} state, {cycle} cycle fails)")
            continue
        if state > allowed[0]:
            regressions.append(f"{key}: state fails {allowed[0]} -> {state}")
        if cycle > allowed[1]:
            regressions.append(f"{key}: cycle fails {allowed[1]} -> {cycle}")
    for key in sorted(set(baseline) - set(current)):
        regressions.append(f"{key}: in the baseline but absent from the run")

    if regressions:
        print(
            f"error: {len(regressions)} 68000 conformance regression(s) outside the "
            "reviewed baseline. The ratchet only shrinks:",
            file=sys.stderr,
        )
        for line in regressions:
            print(f"    {line}", file=sys.stderr)
        return 1

    improvements = {key for key, value in current.items() if value != baseline[key]}
    if update and improvements:
        write_baseline(current)
        print(f"cpu-vector ratchet lowered on {len(improvements)} opcode file(s)")
        return 0

    state_total = sum(state for state, _ in current.values())
    cycle_total = sum(cycle for _, cycle in current.values())
    print(
        f"68000 conformance: {state_total} state and {cycle_total} cycle failures "
        f"across {len(current)} opcode files (within baseline)"
        + (f"; {len(improvements)} file(s) improved, re-run with --update" if improvements else "")
    )
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
