#!/usr/bin/env python3
"""Replay bounded HUD raster spans from the census' measured CPU timeline.

This is a measurement tool, never a CI oracle. It tests whether the captured
cumulative 68000 cycle span reproduces the captured H40 beam and classifies
the asynchronous wait surface that a native frame-local scheduler must own.
It does not emit values consumed by the game or pixel verifier.

Units: every `pixel` field in the census is an H40 **pixel index** (0..419),
the unit `hud_timing_census.py` records. A pixel index is an mclk *interval*
(8, 9, or 10 mclk wide under the H40 divider), never an mclk offset into the
3,420-mclk scanline. The replay therefore advances the entry interval by the
exact measured cycle count and asks whether it intersects the observed
interval: `cpu_span_exact` is interval intersection, `residual_mclk` the
signed gap when it misses (positive = replay stopped short of the observed
beam, negative = replay overshot it).
"""

from __future__ import annotations

import argparse
import json
import sys
from pathlib import Path
from typing import Any, Iterable

H40_MCLK_PER_SCANLINE = 3420
H40_PIXELS_PER_SCANLINE = 420
NTSC_SCANLINES = 262
FRAME_MCLK = H40_MCLK_PER_SCANLINE * NTSC_SCANLINES
MCLK_PER_M68K_CYCLE = 7
DEFAULT_CLUSTERS = ("hud_diamond", "hud_life_icon")

# H40 divider, ported from wildside-core `h40_scanline_mclk_to_pixel`
# (crates/wildside-core/src/vdp.rs, itself from jgenesis
# `scanline_mclk_to_pixel_h40`): mclk/8 outside HSYNC, with the documented
# 8/10/9/10 divider pattern during HSYNC.
_INTERNAL_H_JUMP = 0x1C9 - 0x16D
_HSYNC_START_MCLK = (0x1CC - _INTERNAL_H_JUMP) * 8
_HSYNC_PATTERN_MCLK = 8 + 7 * 10 + 2 * 9 + 7 * 10
_HSYNC_END_MCLK = _HSYNC_START_MCLK + 2 * _HSYNC_PATTERN_MCLK


def _scanline_mclk_to_pixel(scanline_mclk: int) -> int:
    if scanline_mclk < _HSYNC_START_MCLK:
        return scanline_mclk // 8
    if scanline_mclk < _HSYNC_END_MCLK:
        hsync_mclk = scanline_mclk - _HSYNC_START_MCLK
        pattern_mclk = hsync_mclk % _HSYNC_PATTERN_MCLK
        if pattern_mclk <= 7:
            pattern_pixel = 0
        elif pattern_mclk <= 77:
            pattern_pixel = 1 + (pattern_mclk - 8) // 10
        elif pattern_mclk <= 95:
            pattern_pixel = 8 + (pattern_mclk - 78) // 9
        else:
            pattern_pixel = 10 + (pattern_mclk - 96) // 10
        base = 0x1CC - _INTERNAL_H_JUMP
        if hsync_mclk < _HSYNC_PATTERN_MCLK:
            return base + pattern_pixel
        return base + 17 + pattern_pixel
    return 0x1CC - _INTERNAL_H_JUMP + 34 + (scanline_mclk - _HSYNC_END_MCLK) // 8


def _pixel_mclk_bounds() -> dict[int, tuple[int, int]]:
    bounds: dict[int, tuple[int, int]] = {}
    for mclk in range(H40_MCLK_PER_SCANLINE):
        pixel = _scanline_mclk_to_pixel(mclk)
        lo, hi = bounds.get(pixel, (mclk, mclk))
        bounds[pixel] = (min(lo, mclk), max(hi, mclk))
    return bounds


PIXEL_MCLK_BOUNDS = _pixel_mclk_bounds()


def beam_mclk_interval(position: dict[str, Any]) -> tuple[int, int]:
    """Frame-absolute mclk interval a captured (scanline, pixel index) spans."""
    pixel = position["pixel"]
    if pixel not in PIXEL_MCLK_BOUNDS:
        raise ValueError(
            f"pixel index out of range 0..{H40_PIXELS_PER_SCANLINE - 1}: {pixel}"
        )
    lo, hi = PIXEL_MCLK_BOUNDS[pixel]
    base = position["scanline"] * H40_MCLK_PER_SCANLINE
    return base + lo, base + hi


def mclk_to_position(frame_mclk: int) -> dict[str, int]:
    wrapped = frame_mclk % FRAME_MCLK
    return {
        "scanline": wrapped // H40_MCLK_PER_SCANLINE,
        "pixel": _scanline_mclk_to_pixel(wrapped % H40_MCLK_PER_SCANLINE),
    }


def replay_cpu_span(
    entry: dict[str, Any], observed: dict[str, Any], m68k_cycles: int
) -> dict[str, Any]:
    entry_lo, entry_hi = beam_mclk_interval(entry)
    advance = m68k_cycles * MCLK_PER_M68K_CYCLE
    advanced_lo, advanced_hi = entry_lo + advance, entry_hi + advance

    observed_lo, observed_hi = beam_mclk_interval(observed)
    # The observed beam recurs every frame; compare against the occurrence
    # nearest the advanced interval so multi-frame spans replay correctly.
    frames_ahead = round(
        ((advanced_lo + advanced_hi) - (observed_lo + observed_hi)) / (2 * FRAME_MCLK)
    )
    observed_lo += frames_ahead * FRAME_MCLK
    observed_hi += frames_ahead * FRAME_MCLK

    if advanced_hi < observed_lo:
        residual = observed_lo - advanced_hi
    elif advanced_lo > observed_hi:
        residual = observed_hi - advanced_lo
    else:
        residual = 0
    return {
        "replayed_update_hud": mclk_to_position(advanced_hi),
        "cpu_span_exact": residual == 0,
        "residual_mclk": residual,
        "entry_quantisation_mclk": entry_hi - entry_lo + 1,
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
            observed_update = {
                "scanline": frame["update_hud"]["scanline"],
                "pixel": frame["update_hud"]["pixel"],
            }
            span = replay_cpu_span(
                frame["mode_entry"],
                observed_update,
                frame["mode_to_update_hud"]["m68k_cycles"],
            )
            exact_cpu_spans += int(span["cpu_span_exact"])
            total_frames += 1
            first_commit = frame["first_fifo_commit"]
            fifo_latency_mclk = (
                beam_mclk_interval(first_commit)[0]
                - beam_mclk_interval(observed_update)[0]
            ) % FRAME_MCLK
            rows.append(
                {
                    "frame": frame["frame"],
                    "wait_class": (
                        "wait_free"
                        if frame["mode_to_update_hud"]["z80_wait_cycles"] == 0
                        and frame["update_to_draw_hud"]["z80_wait_cycles"] == 0
                        else "asynchronous_z80_wait"
                    ),
                    "observed_update_hud": observed_update,
                    "first_fifo_commit": first_commit,
                    "fifo_latency_mclk_from_update_hud": fifo_latency_mclk,
                    **span,
                }
            )
        output_clusters.append({"cluster": cluster["cluster"], "frames": rows})

    return {
        "schema_version": 2,
        "pixel_units": "h40_pixel_index",
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
