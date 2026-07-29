#!/usr/bin/env python3
"""Join TAS HUD pixel mismatches to oracle CPU and VDP timing traces."""

from __future__ import annotations

import argparse
import json
import sys
from collections import Counter
from pathlib import Path
from typing import Any, Iterable

MODE_LEVEL_PC = 0x7C8
UPDATE_HUD_PC = 0xE12
DRAW_HUD_PC = 0x44DC


def json_records(path: Path, prefix: str = "") -> Iterable[dict[str, Any]]:
    for line_number, raw_line in enumerate(path.read_text().splitlines(), 1):
        line = raw_line.strip()
        if prefix:
            if not line.startswith(prefix):
                continue
            line = line.removeprefix(prefix)
        if not line:
            continue
        try:
            yield json.loads(line)
        except json.JSONDecodeError as error:
            raise ValueError(f"{path}:{line_number}: invalid JSON: {error}") from error


def mismatch_frames(report: dict[str, Any], cluster_key: str) -> list[int]:
    clusters = [cluster for cluster in report["clusters"] if cluster["key"] == cluster_key]
    if len(clusters) != 1:
        raise ValueError(f"expected one {cluster_key!r} cluster, found {len(clusters)}")
    return [
        frame
        for start, end in clusters[0]["frame_ranges"]
        for frame in range(start, end + 1)
    ]


def unique_index(
    records: Iterable[dict[str, Any]], key: tuple[str, ...], description: str
) -> dict[tuple[int, ...], dict[str, Any]]:
    result: dict[tuple[int, ...], dict[str, Any]] = {}
    for record in records:
        record_key = tuple(record[field] for field in key)
        if record_key in result:
            raise ValueError(f"duplicate {description} record for {record_key}")
        result[record_key] = record
    return result


def delta(after: dict[str, Any], before: dict[str, Any], field: str) -> int:
    return after[field] - before[field]


def build_census(
    report: dict[str, Any],
    pc_records: Iterable[dict[str, Any]],
    fifo_records: Iterable[dict[str, Any]],
) -> dict[str, Any]:
    frames = mismatch_frames(report, "hud_diamond")
    pcs = unique_index(pc_records, ("frame", "pc"), "PC")
    fifos = unique_index(fifo_records, ("frame",), "FIFO")
    rows = []
    for frame in frames:
        try:
            mode = pcs[(frame - 1, MODE_LEVEL_PC)]
            update = pcs[(frame, UPDATE_HUD_PC)]
            draw = pcs[(frame, DRAW_HUD_PC)]
            fifo = fifos[(frame,)]
        except KeyError as error:
            raise ValueError(f"frame {frame}: missing trace record {error.args[0]}") from error

        rows.append(
            {
                "frame": frame,
                "mode_entry": {
                    "scanline": mode["scanline"],
                    "pixel": mode["pixel"],
                    "refresh_phase": mode["m68k_refresh_counter_before_instruction"],
                },
                "mode_to_update_hud": {
                    "m68k_cycles": delta(update, mode, "m68k_cycles_before_instruction"),
                    "z80_wait_cycles": delta(update, mode, "z80_68k_wait_cpu_cycles_total"),
                    "total_wait_cycles": delta(update, mode, "m68k_wait_cpu_cycles_total"),
                },
                "update_to_draw_hud": {
                    "m68k_cycles": delta(draw, update, "m68k_cycles_before_instruction"),
                    "z80_wait_cycles": delta(draw, update, "z80_68k_wait_cpu_cycles_total"),
                    "total_wait_cycles": delta(draw, update, "m68k_wait_cpu_cycles_total"),
                },
                "first_fifo_commit": {
                    "scanline": fifo["first"]["scanline"],
                    "pixel": fifo["first"]["pixel"],
                },
            }
        )

    mode_z80 = Counter(row["mode_to_update_hud"]["z80_wait_cycles"] for row in rows)
    draw_z80 = Counter(row["update_to_draw_hud"]["z80_wait_cycles"] for row in rows)
    zero_both = sum(
        row["mode_to_update_hud"]["z80_wait_cycles"] == 0
        and row["update_to_draw_hud"]["z80_wait_cycles"] == 0
        for row in rows
    )
    return {
        "cluster": "hud_diamond",
        "frame_count": len(rows),
        "zero_z80_wait_both_intervals": zero_both,
        "mode_to_update_hud_z80_wait_counts": dict(sorted(mode_z80.items())),
        "update_to_draw_hud_z80_wait_counts": dict(sorted(draw_z80.items())),
        "frames": rows,
    }


def parse_args(argv: list[str]) -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--mismatch-report", type=Path, required=True)
    parser.add_argument("--pc-trace", type=Path, required=True)
    parser.add_argument("--fifo-trace", type=Path, required=True)
    parser.add_argument("--output", type=Path)
    return parser.parse_args(argv)


def main(argv: list[str]) -> int:
    args = parse_args(argv)
    try:
        census = build_census(
            json.loads(args.mismatch_report.read_text()),
            json_records(args.pc_trace, "m68k_pc_trace="),
            json_records(args.fifo_trace, "vdp_fifo_enqueue_summary="),
        )
    except (KeyError, OSError, ValueError) as error:
        print(f"error: {error}", file=sys.stderr)
        return 1
    rendered = json.dumps(census, indent=2) + "\n"
    if args.output:
        args.output.parent.mkdir(parents=True, exist_ok=True)
        args.output.write_text(rendered)
    else:
        print(rendered, end="")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
