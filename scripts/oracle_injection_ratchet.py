#!/usr/bin/env python3
"""Oracle-injection ratchet with a rolling frontier window — the #406 meter.

An "oracle injection" is a teleport: the engine's own clock is discarded and
replaced with a measured oracle value.

    self.engine.advance_ntsc_master_clocks(FRAME_993_SUB_E12_CYCLES * 7
                                           - self.engine.master_clock())

Each one is a hole in the persistent CPU/VDP/GEMS scheduler. The scheduler's
payoff — a runtime seed at frame 1706 — needs the clock to be cycle-continuous
from boot, so an injected frame contributes evidence but not continuity.

A plain downward-only count is unusable here: typing the next frame legitimately
adds one. Two numbers instead:

  frontier_frame        highest frame a teleport is guarded on; may only rise.
  max_injections_below  teleports guarded on frames below the rolling window
                        `frontier_frame - WINDOW_FRAMES`; may only fall.

Frames inside the window are the working set and are exempt, so typing the next
frame with a fresh teleport stays legal. Advancing the frontier rolls the window
forward, pulling older frames into the counted set — so continuing the march
costs retiring the injections it leaves behind. That is the point: it makes the
march's rate measurable instead of free.

Attribution: a teleport belongs to the innermost enclosing `power_on_frame == N`
or `matches!(self.power_on_frame, A | B | ...)` guard (the largest N in a
multi-frame guard). Teleports under no frame guard are boot/title injections and
always count as below the window.

Modes:
  (default, check): exit non-zero iff the below-window count exceeds the
                    baseline; an improvement is reported, not failed.
  --update (fix):   raise `frontier_frame` to the observed frontier and lower
                    `max_injections_below` to the current count when it has
                    fallen. Neither number ever moves the other way. A
                    regression still exits non-zero so the pre-commit `fix`
                    step blocks the commit.

Wired as its own hk pre-commit step (`check_first = false`, mirroring
dead-code-ratchet), not as part of verify-slice-fast: it is a static scan, so it
needs no build and no `.cache` assets.
"""

from __future__ import annotations

import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
CRATES = ROOT / "crates"
BASELINE = ROOT / "docs/source-port/oracle-injection-baseline.yaml"

# Frames within this many of the frontier are the working set and are exempt.
# Wide enough that typing a frame and settling it over the next few sessions is
# not immediately penalised; narrow enough that the cost lands within a week of
# march at one frame per session.
WINDOW_FRAMES = 10

# The teleport shape: discard the engine clock and jump to a measured oracle
# cycle. rustfmt wraps this across lines, so the match is DOTALL. Both the
# `self.engine` and bare-`engine` receivers appear; the constant may be a named
# const or a literal.
TELEPORT_RE = re.compile(
    r"advance_ntsc_master_clocks\(\s*"
    r"[A-Za-z0-9_]+\s*\*\s*7\s*-\s*"
    r"[A-Za-z0-9_.]+\.master_clock\(\)\s*,?\s*\)",
    re.S,
)
GUARD_EQ_RE = re.compile(r"power_on_frame\s*==\s*(\d+)")
GUARD_MATCHES_RE = re.compile(r"matches!\(\s*self\.power_on_frame\s*,([^)]*)\)", re.S)

FRONTIER_RE = re.compile(r"^(?P<pre>\s*frontier_frame\s*:\s*)(?P<n>\d+)(?P<post>.*)$", re.M)
MAX_BELOW_RE = re.compile(
    r"^(?P<pre>\s*max_injections_below\s*:\s*)(?P<n>\d+)(?P<post>.*)$", re.M
)


def guard_frames_by_line(source: str) -> tuple[list[int | None], int]:
    """Innermost enclosing frame guard for each line, plus the closing brace depth.

    Brace depth is counted over raw text, so the returned depth is a sanity
    signal: a non-zero final depth means braces inside strings or comments have
    skewed the scan and the attribution cannot be trusted.
    """
    per_line: list[int | None] = []
    open_guards: list[tuple[int, int]] = []  # (depth outside the block, frame)
    depth = 0
    for line in source.split("\n"):
        matched = GUARD_MATCHES_RE.search(line)
        if matched:
            frames = [int(n) for n in re.findall(r"\d+", matched.group(1))]
        else:
            frames = [int(n) for n in GUARD_EQ_RE.findall(line)]

        opens, closes = line.count("{"), line.count("}")
        depth += opens - closes
        while open_guards and open_guards[-1][0] >= depth:
            open_guards.pop()
        if frames and opens > closes:
            open_guards.append((depth - 1, max(frames)))
        per_line.append(open_guards[-1][1] if open_guards else None)
    return per_line, depth


def scan() -> dict[int | None, int]:
    """Teleport counts keyed by attributed frame; `None` means no frame guard."""
    counts: dict[int | None, int] = {}
    for path in sorted(CRATES.rglob("*.rs")):
        source = path.read_text()
        if not TELEPORT_RE.search(source):
            continue
        per_line, final_depth = guard_frames_by_line(source)
        if final_depth != 0:
            print(
                f"error: {path} brace scan ended at depth {final_depth}; frame "
                "attribution is unreliable",
                file=sys.stderr,
            )
            raise SystemExit(2)
        for match in TELEPORT_RE.finditer(source):
            line = source.count("\n", 0, match.start())
            frame = per_line[line]
            counts[frame] = counts.get(frame, 0) + 1
    return counts


def below_window(counts: dict[int | None, int], frontier: int) -> int:
    cutoff = frontier - WINDOW_FRAMES
    return sum(n for frame, n in counts.items() if frame is None or frame < cutoff)


def read_baseline() -> tuple[int, int]:
    text = BASELINE.read_text()
    frontier, max_below = FRONTIER_RE.search(text), MAX_BELOW_RE.search(text)
    if not frontier or not max_below:
        print(
            f"error: {BASELINE} needs both `frontier_frame:` and "
            "`max_injections_below:` lines",
            file=sys.stderr,
        )
        raise SystemExit(2)
    return int(frontier.group("n")), int(max_below.group("n"))


def write_baseline(frontier: int, max_below: int) -> None:
    text = BASELINE.read_text()
    text = FRONTIER_RE.sub(rf"\g<pre>{frontier}\g<post>", text, count=1)
    text = MAX_BELOW_RE.sub(rf"\g<pre>{max_below}\g<post>", text, count=1)
    BASELINE.write_text(text)


def main(argv: list[str]) -> int:
    update = "--update" in argv[1:]
    counts = scan()
    observed = max((f for f in counts if f is not None), default=0)
    baseline_frontier, baseline_max = read_baseline()
    # The frontier only ever rises, so a scan that sees fewer typed frames than
    # the baseline (a retired frame, a partial checkout) must not roll the
    # window backwards and silently pardon injections it already counted.
    frontier = max(baseline_frontier, observed)
    count = below_window(counts, frontier)

    if count > baseline_max:
        print(
            f"error: oracle injections below frame {frontier - WINDOW_FRAMES} rose to "
            f"{count} (baseline {baseline_max}, frontier {frontier}). Advancing the "
            "frontier rolls older frames into the counted set — retire their teleports "
            "(let the persistent scheduler reach the frame) rather than leaving the "
            "clock discarded there. The ratchet only falls.",
            file=sys.stderr,
        )
        return 1

    if update and (frontier > baseline_frontier or count < baseline_max):
        write_baseline(frontier, min(count, baseline_max))
        print(
            f"oracle-injection ratchet: frontier {baseline_frontier} -> {frontier}, "
            f"below-window {baseline_max} -> {min(count, baseline_max)}"
        )
        return 0

    print(
        f"oracle injections: {count} below frame {frontier - WINDOW_FRAMES} "
        f"(<= baseline {baseline_max}); frontier {frontier}"
    )
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
