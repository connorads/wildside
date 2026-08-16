#!/usr/bin/env python3

import importlib.util
import unittest
from pathlib import Path

SCRIPT = Path(__file__).with_name("raster_span_replay.py")
SPEC = importlib.util.spec_from_file_location("raster_span_replay", SCRIPT)
assert SPEC and SPEC.loader
REPLAY = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(REPLAY)


def census_frame_1730() -> dict:
    """Real capture: hud_diamond frame 1730 from the 2026-08-11 HUD census.

    All `pixel` fields are H40 pixel indices (0..419), the unit
    `hud_timing_census.py` emits. Entry pixel 204 spans mclk 1632..1639 on
    scanline 224; 9,282 measured 68000 cycles (x7 mclk) land the advanced
    interval on scanline 243 mclk 832686..832693, intersecting observed
    pixel 204 (mclk 832692..832699): exact with residual 0.
    """
    return {
        "frame": 1730,
        "changed_pixels": 2,
        "mode_entry": {"scanline": 224, "pixel": 204, "refresh_phase": 80},
        "mode_to_update_hud": {
            "m68k_cycles": 9282,
            "z80_wait_cycles": 0,
            "total_wait_cycles": 2058,
        },
        "update_hud": {"scanline": 243, "pixel": 204, "refresh_phase": 20},
        "update_to_draw_hud": {
            "m68k_cycles": 18630,
            "z80_wait_cycles": 0,
            "total_wait_cycles": 290,
        },
        "draw_hud": {"scanline": 19, "pixel": 260, "refresh_phase": 90},
        "first_fifo_commit": {"scanline": 20, "pixel": 380},
    }


def census_frame_1754() -> dict:
    """Real capture: hud_diamond frame 1754 from the 2026-08-11 HUD census.

    Chosen because its advanced interval touches the observed interval's
    trailing edge: exact at the recorded 9,162 cycles, no longer exact at
    9,163 (one 68000 cycle more).
    """
    return {
        "frame": 1754,
        "changed_pixels": 2,
        "mode_entry": {"scanline": 224, "pixel": 205, "refresh_phase": 68},
        "mode_to_update_hud": {
            "m68k_cycles": 9162,
            "z80_wait_cycles": 0,
            "total_wait_cycles": 2056,
        },
        "update_hud": {"scanline": 243, "pixel": 99, "refresh_phase": 28},
        "update_to_draw_hud": {
            "m68k_cycles": 17826,
            "z80_wait_cycles": 0,
            "total_wait_cycles": 278,
        },
        "draw_hud": {"scanline": 17, "pixel": 307, "refresh_phase": 62},
        "first_fifo_commit": {"scanline": 19, "pixel": 12},
    }


def census(*frames: dict, cluster: str = "hud_diamond") -> dict:
    return {"clusters": [{"cluster": cluster, "frames": list(frames)}]}


def replay_row(report: dict, frame: int) -> dict:
    for cluster in report["clusters"]:
        for row in cluster["frames"]:
            if row["frame"] == frame:
                return row
    raise AssertionError(f"frame {frame} missing from replay report")


def synthetic_frame(index: int, z80_wait: int = 0, changed_pixels: int = 1) -> dict:
    # Valid H40 pixel indices only (0..419). Entry pixel 100 spans mclk
    # 800..807; 10 cycles (70 mclk) advance to 870..877 (pixels 108..109),
    # intersecting observed pixel 109 (mclk 872..879): exact.
    return {
        "frame": index,
        "changed_pixels": changed_pixels,
        "mode_entry": {"scanline": 0, "pixel": 100, "refresh_phase": 0},
        "mode_to_update_hud": {
            "m68k_cycles": 10,
            "z80_wait_cycles": z80_wait,
            "total_wait_cycles": z80_wait,
        },
        "update_hud": {"scanline": 0, "pixel": 109, "refresh_phase": 10},
        "update_to_draw_hud": {
            "m68k_cycles": 5,
            "z80_wait_cycles": 0,
            "total_wait_cycles": 0,
        },
        "draw_hud": {"scanline": 0, "pixel": 120, "refresh_phase": 15},
        "first_fifo_commit": {"scanline": 0, "pixel": 130},
    }


class RasterSpanReplayUnitsTests(unittest.TestCase):
    """Pin the H40 pixel-index/mclk units of the replay (#409).

    The census records beam positions as H40 pixel indices (0..419);
    3,420 mclk per scanline. A pixel index is an mclk *interval*
    (8/9/10 mclk wide under the divider in wildside-core vdp.rs
    `h40_scanline_mclk_to_pixel`), never an mclk offset.
    """

    def test_frame_1730_replays_exact_with_zero_residual(self) -> None:
        report = REPLAY.replay_census(census(census_frame_1730()))

        self.assertEqual(report["schema_version"], 2)
        self.assertEqual(report["pixel_units"], "h40_pixel_index")
        self.assertTrue(report["all_cpu_spans_exact"])
        row = replay_row(report, 1730)
        self.assertTrue(row["cpu_span_exact"])
        self.assertEqual(row["residual_mclk"], 0)
        self.assertEqual(row["entry_quantisation_mclk"], 8)
        self.assertEqual(row["replayed_update_hud"], {"scanline": 243, "pixel": 204})

    def test_frame_1730_discriminates_at_one_cpu_cycle(self) -> None:
        frame = census_frame_1730()
        frame["mode_to_update_hud"]["m68k_cycles"] -= 1

        row = replay_row(REPLAY.replay_census(census(frame)), 1730)

        self.assertFalse(row["cpu_span_exact"])
        # Advanced interval 832679..832686 stops 6 mclk short of observed
        # 832692..832699; positive residual = replay behind the beam.
        self.assertEqual(row["residual_mclk"], 6)

    def test_frame_1730_rejects_ten_cycle_overshoot(self) -> None:
        frame = census_frame_1730()
        frame["mode_to_update_hud"]["m68k_cycles"] += 10

        row = replay_row(REPLAY.replay_census(census(frame)), 1730)

        self.assertFalse(row["cpu_span_exact"])
        # Advanced interval 832756..832763 overshoots observed 832692..832699
        # by 57 mclk; negative residual = replay past the beam.
        self.assertEqual(row["residual_mclk"], -57)

    def test_frame_1754_flips_at_plus_one_cpu_cycle(self) -> None:
        exact = replay_row(
            REPLAY.replay_census(census(census_frame_1754())), 1754
        )
        self.assertTrue(exact["cpu_span_exact"])
        self.assertEqual(exact["residual_mclk"], 0)

        perturbed_frame = census_frame_1754()
        perturbed_frame["mode_to_update_hud"]["m68k_cycles"] += 1
        perturbed = replay_row(REPLAY.replay_census(census(perturbed_frame)), 1754)
        self.assertFalse(perturbed["cpu_span_exact"])
        self.assertEqual(perturbed["residual_mclk"], -2)

    def test_fifo_latency_converts_pixel_indices_to_mclk(self) -> None:
        row = replay_row(REPLAY.replay_census(census(census_frame_1730())), 1730)

        # update_hud {243, 204} -> mclk 832692; first_fifo_commit {20, 380}
        # -> mclk 71460 (pixel 380 sits in the HSYNC divider region, mclk
        # 3060 into the scanline, NOT 380). Wrapped distance = 134808.
        # Adding the pixel index as mclk would give 133556.
        self.assertEqual(row["fifo_latency_mclk_from_update_hud"], 134808)


class RasterSpanReplayTests(unittest.TestCase):
    def test_classifies_waits_and_selects_witnesses(self) -> None:
        report = REPLAY.replay_census(
            {
                "clusters": [
                    {
                        "cluster": "hud_diamond",
                        "frames": [synthetic_frame(10), synthetic_frame(11, 11)],
                    },
                    {
                        "cluster": "hud_life_icon",
                        "frames": [
                            synthetic_frame(12),
                            synthetic_frame(13, changed_pixels=3),
                        ],
                    },
                ]
            }
        )

        self.assertTrue(report["all_cpu_spans_exact"])
        self.assertEqual(report["frame_count"], 4)
        self.assertEqual(
            [row["wait_class"] for row in report["clusters"][0]["frames"]],
            ["wait_free", "asynchronous_z80_wait"],
        )
        self.assertEqual(
            report["witnesses"],
            [
                {"kind": "first_wait_free_diamond", "frame": 10},
                {"kind": "first_asynchronous_wait_diamond", "frame": 11},
                {"kind": "worst_life_icon", "frame": 13, "changed_pixels": 3},
            ],
        )

    def test_rejects_missing_selected_clusters(self) -> None:
        with self.assertRaisesRegex(ValueError, "no selected clusters"):
            REPLAY.replay_census({"clusters": []})

    def test_rejects_out_of_range_pixel_index(self) -> None:
        frame = synthetic_frame(10)
        frame["mode_entry"]["pixel"] = 3400  # mclk offset, not a pixel index

        with self.assertRaisesRegex(ValueError, "pixel index"):
            REPLAY.replay_census(census(frame))


if __name__ == "__main__":
    unittest.main()
