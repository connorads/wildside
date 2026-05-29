#!/usr/bin/env python3
"""Dead-helper census extractor.

Scans kid-core (and optionally kidcli) Rust sources for items carrying
`#[allow(dead_code)]` or `#[cfg_attr(not(test), allow(dead_code))]`, captures the
item name/kind/line, the ASM labels cited in its doc comment (or embedded in its
name), and a crate-wide call-site count split by test vs non-test context.

The `cfg_attr(not(test), ...)` attribute is self-labelling: its presence means the
compiler sees the item as dead in release builds, i.e. NOT wired into the live path.
A non-test call site on such an item is therefore a flag (stale attribute or a call
that is itself dead). Output is JSON for downstream synthesis agents.
"""
import json
import re
import sys
from pathlib import Path

SRC = Path("crates/kid-core/src")
ATTR_RE = re.compile(r"#\[(cfg_attr\(not\(test\),\s*allow\(dead_code\)\)|allow\(dead_code\))\]")
ITEM_RE = re.compile(
    r"^\s*(?:pub(?:\([^)]*\))?\s+)?"
    r"(?P<kind>fn|struct|enum|const|static|impl|trait|type|mod)\s+"
    r"(?P<name>[A-Za-z_][A-Za-z0-9_]*)"
)
ASM_RE = re.compile(
    r"\b(?:loc|sub|off|word|byte|unk|stru)_[0-9A-Fa-f]{3,}\b"
    r"|\bEnemy[0-9A-Fa-f]{2}_[A-Za-z0-9_]+\b"
    r"|\b[A-Z][A-Za-z0-9_]+\.asm:[0-9,]+\b"
)


def doc_block_above(lines, attr_idx):
    """Collect contiguous /// or // doc lines immediately above the attribute(s)."""
    i = attr_idx - 1
    # skip stacked attributes/derives
    while i >= 0 and (lines[i].lstrip().startswith("#[") or lines[i].strip() == ""):
        i -= 1
    doc = []
    while i >= 0 and lines[i].lstrip().startswith("//"):
        doc.append(lines[i].strip().lstrip("/").strip())
        i -= 1
    return list(reversed(doc))


def find_item(lines, attr_idx):
    """Find the first item declaration at/after the attribute (skipping more attrs)."""
    i = attr_idx
    while i < len(lines):
        m = ITEM_RE.match(lines[i])
        if m:
            return m.group("kind"), m.group("name"), i + 1  # 1-based line
        if lines[i].lstrip().startswith("#[") or lines[i].strip() == "":
            i += 1
            continue
        # some non-item line (e.g. macro) — bail
        i += 1
        if i - attr_idx > 6:
            break
    return None, None, None


def build_test_line_mask(text):
    """Return a set of 0-based line indices that fall inside a #[cfg(test)] mod block
    (brace-balanced). Coarse but good enough for call-site classification."""
    lines = text.splitlines()
    mask = set()
    i = 0
    while i < len(lines):
        if re.search(r"#\[cfg\(test\)\]", lines[i]):
            # find the opening brace of the following mod/block
            j = i
            depth = 0
            started = False
            while j < len(lines):
                depth += lines[j].count("{") - lines[j].count("}")
                if "{" in lines[j]:
                    started = True
                if started and depth <= 0:
                    break
                if started:
                    mask.add(j)
                j += 1
            i = j + 1
            continue
        i += 1
    return mask


def main():
    files = sorted(SRC.glob("*.rs"))  # subsystem source files (declaration scan)
    # Call-site search must also see the test tree: tests live in separate files
    # under crates/kid-core/src/tests/*.rs, NOT inline #[cfg(test)] mods. Treat every
    # reference in that directory as a test call.
    test_files = sorted((SRC / "tests").glob("*.rs"))
    search_text = {f: f.read_text() for f in files + test_files}
    all_text = {f: search_text[f] for f in files}
    # A line is test-context if it is in the tests/ dir, or inside an inline cfg(test) mod.
    test_masks = {}
    for f, t in search_text.items():
        if f.parent.name == "tests":
            test_masks[f] = set(range(len(t.splitlines())))  # whole file is tests
        else:
            test_masks[f] = build_test_line_mask(t)

    records = []
    for f in files:
        lines = all_text[f].splitlines()
        for idx, line in enumerate(lines):
            if not ATTR_RE.search(line):
                continue
            attr = "cfg_not_test" if "cfg_attr" in line else "plain"
            kind, name, item_line = find_item(lines, idx)
            if not name:
                continue
            doc = doc_block_above(lines, idx)
            labels = sorted(set(ASM_RE.findall("\n".join(doc))))
            # labels embedded in the item name itself
            name_labels = sorted(set(ASM_RE.findall(name)))
            records.append(
                {
                    "file": f.name,
                    "line": item_line,
                    "kind": kind,
                    "name": name,
                    "attr": attr,
                    "asm_labels": labels,
                    "name_asm_labels": name_labels,
                    "doc_first": doc[0] if doc else "",
                }
            )

    # call-site classification (functions only)
    for r in records:
        if r["kind"] != "fn":
            r["calls_nontest"] = None
            r["calls_test"] = None
            continue
        pat = re.compile(r"\b" + re.escape(r["name"]) + r"\s*\(")
        nontest = test = 0
        for f, t in search_text.items():
            for ln, txt in enumerate(t.splitlines()):
                if ln + 1 == r["line"] and f.name == r["file"]:
                    continue  # skip definition
                if pat.search(txt):
                    if ln in test_masks[f]:
                        test += 1
                    else:
                        nontest += 1
        r["calls_nontest"] = nontest
        r["calls_test"] = test

    out = {
        "records": records,
        "summary": {
            "total": len(records),
            "by_file": {},
            "by_attr": {},
        },
    }
    for r in records:
        out["summary"]["by_file"].setdefault(r["file"], 0)
        out["summary"]["by_file"][r["file"]] += 1
        out["summary"]["by_attr"].setdefault(r["attr"], 0)
        out["summary"]["by_attr"][r["attr"]] += 1

    # wired-flag: cfg_not_test item with a non-test call site = stale/flagged
    flagged = [
        r for r in records
        if r["kind"] == "fn" and r["attr"] == "cfg_not_test" and (r["calls_nontest"] or 0) > 0
    ]
    out["summary"]["flagged_wired_despite_attr"] = len(flagged)

    Path(".cache/audit").mkdir(parents=True, exist_ok=True)
    Path(".cache/audit/dead-helper-census.json").write_text(json.dumps(out, indent=2))
    print(json.dumps(out["summary"], indent=2))


if __name__ == "__main__":
    main()
