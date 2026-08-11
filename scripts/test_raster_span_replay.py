#!/usr/bin/env python3

import importlib.util
import unittest
from pathlib import Path

SCRIPT = Path(__file__).with_name("raster_span_replay.py")
SPEC = importlib.util.spec_from_file_location("raster_span_replay", SCRIPT)
assert SPEC and SPEC.loader
REPLAY = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(REPLAY)


def frame(index: int, z80_wait: int = 0, changed_pixels: int = 1) -> dict:
    return {
        "frame": index,
        "changed_pixels": changed_pixels,
        "mode_entry": {"scanline": 261, "pixel": 3400, "refresh_phase": 0},
        "mode_to_update_hud": {
            "m68k_cycles": 10,
            "z80_wait_cycles": z80_wait,
            "total_wait_cycles": z80_wait,
        },
        "update_hud": {"scanline": 0, "pixel": 50, "refresh_phase": 10},
        "update_to_draw_hud": {
            "m68k_cycles": 5,
            "z80_wait_cycles": 0,
            "total_wait_cycles": 0,
        },
        "draw_hud": {"scanline": 0, "pixel": 85, "refresh_phase": 15},
        "first_fifo_commit": {"scanline": 0, "pixel": 100},
    }


class RasterSpanReplayTests(unittest.TestCase):
    def test_replays_wrapped_h40_cpu_span_and_classifies_waits(self) -> None:
        census = {
            "clusters": [
                {"cluster": "hud_diamond", "frames": [frame(10), frame(11, 11)]},
                {
                    "cluster": "hud_life_icon",
                    "frames": [frame(12), frame(13, changed_pixels=3)],
                },
            ]
        }

        result = REPLAY.replay_census(census)

        self.assertTrue(result["all_cpu_spans_exact"])
        self.assertEqual(result["frame_count"], 4)
        self.assertEqual(
            result["witnesses"],
            [
                {"kind": "first_wait_free_diamond", "frame": 10},
                {"kind": "first_asynchronous_wait_diamond", "frame": 11},
                {"kind": "worst_life_icon", "frame": 13, "changed_pixels": 3},
            ],
        )
        self.assertEqual(
            result["clusters"][0]["frames"][0]["fifo_latency_mclk_from_update_hud"],
            50,
        )

    def test_rejects_missing_selected_clusters(self) -> None:
        with self.assertRaisesRegex(ValueError, "no selected clusters"):
            REPLAY.replay_census({"clusters": []})


if __name__ == "__main__":
    unittest.main()
