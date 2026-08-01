#!/usr/bin/env python3
"""Gate "the disassembly is truth": assemble it and byte-compare the reference ROM.

Every fidelity claim in this repo is cited against
`.cache/upstream/kid-chameleon-disasm`, so the disassembly being an exact
description of the shipped ROM is a load-bearing assumption. This script turns
that assumption into a gate: it assembles the upstream source with the AS
(Macroassembler AS) toolchain and byte-compares the result against the ROM the
rest of the tooling reads. Any differing byte is a red gate.

Pipeline (all steps idempotent, so it is safe to run on demand):

1. Resolve the reference ROM the same way `wildside_rom::default_rom_candidates`
   does: `roms/kid.bin`, then `kid.bin`, then `kc.md`. `--rom` overrides.
2. Run `wildside reference-build`, which bootstraps the pinned disassembly
   checkout, builds the AS toolchain into `.cache/tooling/asl-releases`, runs
   upstream `split.py` to regenerate the `incbin` chunks, assembles `kid.asm`
   and links `kid_built.bin`.
3. Byte-compare `kid_built.bin` against the reference ROM, coalescing differing
   bytes into readable regions.
4. On a mismatch only, re-assemble with `-g map` and attribute every region to
   its nearest preceding ASM label and source line, so the offsets point at
   source rather than at raw addresses.

Upstream quirks this works around:

- `build.sh` ships prebuilt Linux ELF binaries and `exit 1`s on the *successful*
  path, so it cannot be trusted as a build oracle. `reference-build` falls back
  to the AS toolchain built from source in `.cache/tooling`, which is the path
  used on macOS.
- `p2bin` emits one "overlapping memory allocation!" warning per overlapping
  section, hundreds of lines per run. Build output is captured and only shown on
  failure or under `--verbose`.
- Generating the `-g map` debug map costs ~20x the plain assemble (~9s versus
  ~0.4s), so it is only paid when there is a mismatch to localise.

There is deliberately no known-mismatch allowlist: the pinned upstream commit
assembles bit-perfect, so every byte difference is a real regression (a stale
`split.py` run, a patched checkout, or a non-canonical ROM) and must be
diagnosed rather than masked.
"""

from __future__ import annotations

import argparse
import bisect
import os
import subprocess
import sys
from dataclasses import dataclass, field
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parent.parent
DISASM_DIR = REPO_ROOT / ".cache/upstream/kid-chameleon-disasm"
BUILT_ROM = DISASM_DIR / "kid_built.bin"
MAP_PATH = DISASM_DIR / "kid.map"
NATIVE_ASL_DIR = REPO_ROOT / ".cache/tooling/asl-releases"
BUNDLED_ASL_DIR = DISASM_DIR / "build/linux"
ROM_CANDIDATES = ("roms/kid.bin", "kid.bin", "kc.md")

# Differing bytes separated by fewer than this many matching bytes are reported
# as one region; a one-instruction edit otherwise shreds into several ranges.
COALESCE_GAP = 16
DEFAULT_MAX_REGIONS = 40
PREVIEW_BYTES = 16


@dataclass(frozen=True)
class Region:
    """Half-open `[start, end)` run of differing ROM offsets."""

    start: int
    end: int

    @property
    def length(self) -> int:
        return self.end - self.start


@dataclass
class SourceMap:
    """Address lookup tables parsed from an AS `-g map` debug map."""

    labels: list[tuple[int, str]] = field(default_factory=list)
    lines: list[tuple[int, str, int]] = field(default_factory=list)
    label_addresses: list[int] = field(default_factory=list)
    line_addresses: list[int] = field(default_factory=list)

    def sort(self) -> None:
        """Order both tables by address and cache the bisect key lists."""
        self.labels.sort()
        self.lines.sort()
        self.label_addresses = [address for address, _ in self.labels]
        self.line_addresses = [address for address, _, _ in self.lines]

    def describe(self, offset: int) -> str:
        """Name the nearest preceding label and source line for a ROM offset."""
        parts = []
        label = _nearest(self.labels, self.label_addresses, offset)
        if label is not None:
            address, name = label
            delta = offset - address
            parts.append(name if delta == 0 else f"{name}+0x{delta:X}")
        line = _nearest(self.lines, self.line_addresses, offset)
        if line is not None:
            _, source, number = line
            parts.append(f"{source}:{number}")
        return "  ".join(parts) if parts else "<unattributed>"


def _nearest(entries: list, addresses: list[int], offset: int):
    """Greatest entry whose address is <= `offset`, or None below the first."""
    index = bisect.bisect_right(addresses, offset) - 1
    if index < 0:
        return None
    return entries[index]


def resolve_rom(explicit: Path | None) -> Path:
    """Locate the reference ROM, mirroring the Rust default-candidate order."""
    if explicit is not None:
        if not explicit.is_file():
            raise SystemExit(f"ROM not found at {explicit}")
        return explicit
    for candidate in ROM_CANDIDATES:
        path = REPO_ROOT / candidate
        if path.is_file():
            return path
    listed = ", ".join(ROM_CANDIDATES)
    raise SystemExit(f"no reference ROM found; drop one at one of: {listed}")


def run_reference_build(rom: Path, allow_unknown_rom: bool, verbose: bool) -> None:
    """Bootstrap the disassembly plus AS toolchain and link `kid_built.bin`.

    A non-zero exit is not fatal here: `reference-build` also fails on a SHA-1
    mismatch, and that is exactly the case this script exists to localise. A
    missing `kid_built.bin` is fatal, because then there is nothing to compare.
    """
    BUILT_ROM.unlink(missing_ok=True)
    command = [
        "cargo",
        "run",
        "-q",
        "-p",
        "wildside-cli",
        "--",
        "reference-build",
        "--rom",
        str(rom),
    ]
    if allow_unknown_rom:
        command.append("--allow-unknown-rom")
    completed = subprocess.run(
        command,
        cwd=REPO_ROOT,
        capture_output=True,
        text=True,
        check=False,
    )
    if verbose or not BUILT_ROM.is_file():
        sys.stdout.write(completed.stdout)
        sys.stderr.write(completed.stderr)
    if not BUILT_ROM.is_file():
        raise SystemExit(
            f"assembly produced no {BUILT_ROM.relative_to(REPO_ROOT)} "
            f"(reference-build exited {completed.returncode})"
        )


def diff_regions(
    reference: bytes, built: bytes, gap: int = COALESCE_GAP
) -> list[Region]:
    """Coalesce differing offsets into regions, merging runs closer than `gap`.

    The all-equal case short-circuits on the C-level `bytes` comparison; only a
    real mismatch pays the per-byte scan.
    """
    if reference == built:
        return []
    shared = min(len(reference), len(built))
    regions: list[Region] = []
    for offset in range(shared):
        if reference[offset] == built[offset]:
            continue
        if regions and offset - regions[-1].end < gap:
            regions[-1] = Region(regions[-1].start, offset + 1)
        else:
            regions.append(Region(offset, offset + 1))
    if len(reference) != len(built):
        regions.append(Region(shared, max(len(reference), len(built))))
    return regions


def ensure_source_map(verbose: bool) -> SourceMap | None:
    """Re-assemble with `-g map` and parse it; None when no assembler is found."""
    assembler, msg_path = _find_assembler()
    if assembler is None or msg_path is None:
        print(
            "no AS assembler found; reporting raw offsets without label attribution",
            file=sys.stderr,
        )
        return None
    environment = dict(os.environ)
    environment["AS_MSGPATH"] = str(msg_path)
    command = [
        str(assembler),
        "-i",
        ".",
        "-xx",
        "-c",
        "-A",
        "-U",
        "-g",
        "map",
        "kid.asm",
    ]
    completed = subprocess.run(
        command,
        cwd=DISASM_DIR,
        env=environment,
        capture_output=True,
        text=True,
        check=False,
    )
    if verbose:
        sys.stdout.write(completed.stdout)
        sys.stderr.write(completed.stderr)
    if completed.returncode != 0 or not MAP_PATH.is_file():
        print(
            "failed to generate an AS debug map; reporting raw offsets", file=sys.stderr
        )
        return None
    return parse_source_map(MAP_PATH.read_text(encoding="utf-8", errors="replace"))


def _find_assembler() -> tuple[Path | None, Path | None]:
    """Prefer the toolchain built from source; fall back to upstream's ELF binary."""
    native = NATIVE_ASL_DIR / "asl"
    if native.is_file():
        return native, NATIVE_ASL_DIR
    bundled = BUNDLED_ASL_DIR / "asl"
    if bundled.is_file():
        return bundled, BUNDLED_ASL_DIR / "msg"
    return None, None


def parse_source_map(text: str) -> SourceMap:
    """Parse the AS map's `File`/`line:address` blocks and CODE symbol table.

    The map has two halves: per-file `line:address` pair lists under
    `Segment CODE`, then `Symbols in Segment <name>` tables of
    `name  Int  <hex>  ...`. Only CODE symbols are ROM addresses; the NOTHING
    segment holds equates and RAM offsets, which would mis-attribute regions.
    """
    source_map = SourceMap()
    current_file: str | None = None
    symbol_segment: str | None = None
    for raw in text.splitlines():
        line = raw.strip()
        if not line:
            continue
        if line.startswith("Symbols in Segment "):
            symbol_segment = line.removeprefix("Symbols in Segment ").strip()
            current_file = None
            continue
        if symbol_segment is not None:
            if symbol_segment != "CODE":
                continue
            fields = line.split()
            if len(fields) < 3 or fields[1] != "Int":
                continue
            try:
                source_map.labels.append((int(fields[2], 16), fields[0]))
            except ValueError:
                continue
            continue
        if line.startswith("Segment "):
            current_file = None
            continue
        if line.startswith("File "):
            current_file = _relative_source(line.removeprefix("File ").strip())
            continue
        if current_file is None:
            continue
        for pair in line.split():
            number, separator, address = pair.partition(":")
            if not separator:
                continue
            try:
                source_map.lines.append((int(address, 16), current_file, int(number)))
            except ValueError:
                continue
    source_map.sort()
    return source_map


def _relative_source(name: str) -> str:
    """Shorten AS's absolute include paths to disassembly-relative ones."""
    path = Path(name)
    if path.is_absolute():
        try:
            return str(path.relative_to(DISASM_DIR))
        except ValueError:
            return str(path)
    return name


def _preview(data: bytes, region: Region) -> str:
    window = data[region.start : min(region.end, region.start + PREVIEW_BYTES)]
    text = " ".join(f"{byte:02X}" for byte in window)
    return f"{text} ..." if region.length > PREVIEW_BYTES else text


def report(
    regions: list[Region],
    reference: bytes,
    built: bytes,
    source_map: SourceMap | None,
    max_regions: int,
) -> None:
    """Print each mismatching region with its ASM attribution and byte preview."""
    differing = sum(region.length for region in regions)
    print(
        f"MISMATCH: {len(regions)} region(s), {differing} byte(s) differ "
        f"(reference {len(reference)} B, built {len(built)} B)"
    )
    for region in regions[:max_regions]:
        location = source_map.describe(region.start) if source_map else ""
        header = f"  0x{region.start:06X}-0x{region.end - 1:06X} ({region.length} B)"
        print(f"{header}  {location}".rstrip())
        print(f"      reference {_preview(reference, region)}")
        print(f"      built     {_preview(built, region)}")
    if len(regions) > max_regions:
        print(f"  ... {len(regions) - max_regions} further region(s) suppressed")


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    parser.add_argument(
        "--rom",
        type=Path,
        help="reference ROM (default: roms/kid.bin, kid.bin, then kc.md)",
    )
    parser.add_argument(
        "--skip-build",
        action="store_true",
        help="compare the existing kid_built.bin without re-assembling",
    )
    parser.add_argument(
        "--allow-unknown-rom",
        action="store_true",
        help="allow a header-compatible ROM whose SHA-1 is not the known dump",
    )
    parser.add_argument(
        "--max-regions",
        type=int,
        default=DEFAULT_MAX_REGIONS,
        help=f"cap reported mismatch regions (default: {DEFAULT_MAX_REGIONS})",
    )
    parser.add_argument(
        "--verbose",
        action="store_true",
        help="show assembler and reference-build output even on success",
    )
    args = parser.parse_args(argv)

    rom = resolve_rom(args.rom)
    if args.skip_build:
        if not BUILT_ROM.is_file():
            raise SystemExit(f"--skip-build needs an existing {BUILT_ROM}")
    else:
        run_reference_build(rom, args.allow_unknown_rom, args.verbose)

    reference = rom.read_bytes()
    built = BUILT_ROM.read_bytes()
    regions = diff_regions(reference, built)
    if not regions:
        print(
            f"bit-perfect: {len(built)} bytes assembled from "
            f"{DISASM_DIR.relative_to(REPO_ROOT)} match {rom.relative_to(REPO_ROOT)}"
        )
        return 0

    report(regions, reference, built, ensure_source_map(args.verbose), args.max_regions)
    return 1


if __name__ == "__main__":
    sys.exit(main())
