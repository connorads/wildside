#!/usr/bin/env python3
"""Replay bounded HUD raster spans from the census' measured CPU timeline.

This is an ADR 0002 measurement tool, never a CI oracle. It tests whether the
captured cumulative 68000 cycle span reproduces the captured H40 beam and
classifies the asynchronous wait surface that a native frame-local scheduler
must own. It does not emit values consumed by the game or pixel verifier.
"""

from __future__ import annotations

import argparse
import json
import sys
from pathlib import Path
from typing import Any, Iterable

H40_MCLK_PER_SCANLINE = 3420
NTSC_SCANLINES = 262
MCLK_PER_M68K_CYCLE = 7
DEFAULT_CLUSTERS = ("hud_diamond", "hud_life_icon")


def beam_mclk(position: dict[str, Any]) -> int:
    return position["scanline"] * H40_MCLK_PER_SCANLINE + position["pixel"]


def advance_beam(position: dict[str, Any], m68k_cycles: int) -> dict[str, int]:
    frame_mclk = H40_MCLK_PER_SCANLINE * NTSC_SCANLINES
    advanced = (beam_mclk(position) + m68k_cycles * MCLK_PER_M68K_CYCLE) % frame_mclk
    return {
        "scanline": advanced // H40_MCLK_PER_SCANLINE,
        "pixel": advanced % H40_MCLK_PER_SCANLINE,
    }


def select_witnesses(clusters: list[dict[str, Any]]) -> list[dict[str, Any]]:
    by_name = {cluster["cluster"]: cluster for cluster in clusters}
    diamond = by_name.get("hud_diamond", {}).get("frames", [])
    life = by_name.get("hud_life_icon", {}).get("frames", [])
    witnesses: list[dict[str, Any]] = []

    wait_free = next(
        (
            frame
            for frame in diamond
            if frame["mode_to_update_hud"]["z80_wait_cycles"] == 0
            and frame["update_to_draw_hud"]["z80_wait_cycles"] == 0
        ),
        None,
    )
    if wait_free is not None:
        witnesses.append({"kind": "first_wait_free_diamond", "frame": wait_free["frame"]})

    asynchronous = next(
        (
            frame
            for frame in diamond
            if frame["mode_to_update_hud"]["z80_wait_cycles"] != 0
            or frame["update_to_draw_hud"]["z80_wait_cycles"] != 0
        ),
        None,
    )
    if asynchronous is not None:
        witnesses.append(
            {"kind": "first_asynchronous_wait_diamond", "frame": asynchronous["frame"]}
        )

    if life:
        worst_life = max(life, key=lambda frame: (frame["changed_pixels"], -frame["frame"]))
        witnesses.append(
            {
                "kind": "worst_life_icon",
                "frame": worst_life["frame"],
                "changed_pixels": worst_life["changed_pixels"],
            }
        )
    return witnesses


def replay_census(
    census: dict[str, Any], cluster_keys: Iterable[str] = DEFAULT_CLUSTERS
) -> dict[str, Any]:
    selected = set(cluster_keys)
    clusters = [cluster for cluster in census["clusters"] if cluster["cluster"] in selected]
    if not clusters:
        raise ValueError(f"no selected clusters found: {sorted(selected)}")

    output_clusters = []
    exact_cpu_spans = 0
    total_frames = 0
    for cluster in clusters:
        rows = []
        for frame in cluster["frames"]:
            replayed_update = advance_beam(
                frame["mode_entry"], frame["mode_to_update_hud"]["m68k_cycles"]
            )
            observed_update = {
                "scanline": frame["update_hud"]["scanline"],
                "pixel": frame["update_hud"]["pixel"],
            }
            cpu_span_exact = replayed_update == observed_update
            exact_cpu_spans += int(cpu_span_exact)
            total_frames += 1
            first_commit = frame["first_fifo_commit"]
            fifo_latency_mclk = (
                beam_mclk(first_commit) - beam_mclk(observed_update)
            ) % (H40_MCLK_PER_SCANLINE * NTSC_SCANLINES)
            rows.append(
                {
                    "frame": frame["frame"],
                    "wait_class": (
                        "wait_free"
                        if frame["mode_to_update_hud"]["z80_wait_cycles"] == 0
                        and frame["update_to_draw_hud"]["z80_wait_cycles"] == 0
                        else "asynchronous_z80_wait"
                    ),
                    "replayed_update_hud": replayed_update,
                    "observed_update_hud": observed_update,
                    "cpu_span_exact": cpu_span_exact,
                    "first_fifo_commit": first_commit,
                    "fifo_latency_mclk_from_update_hud": fifo_latency_mclk,
                }
            )
        output_clusters.append({"cluster": cluster["cluster"], "frames": rows})

    return {
        "schema_version": 1,
        "measurement_only": True,
        "clusters": output_clusters,
        "frame_count": total_frames,
        "exact_cpu_span_count": exact_cpu_spans,
        "all_cpu_spans_exact": exact_cpu_spans == total_frames,
        "witnesses": select_witnesses(clusters),
    }


def parse_args(argv: list[str]) -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--census", type=Path, required=True)
    parser.add_argument(
        "--cluster",
        action="append",
        choices=DEFAULT_CLUSTERS,
        help="HUD cluster to replay; repeat to select both (default: both)",
    )
    parser.add_argument("--output", type=Path)
    return parser.parse_args(argv)


def main(argv: list[str]) -> int:
    args = parse_args(argv)
    try:
        report = replay_census(
            json.loads(args.census.read_text()), args.cluster or DEFAULT_CLUSTERS
        )
    except (KeyError, OSError, ValueError) as error:
        print(f"error: {error}", file=sys.stderr)
        return 1
    rendered = json.dumps(report, indent=2) + "\n"
    if args.output:
        args.output.parent.mkdir(parents=True, exist_ok=True)
        args.output.write_text(rendered)
    else:
        print(rendered, end="")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
