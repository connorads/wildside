#!/usr/bin/env python3

import importlib.util
import unittest
from pathlib import Path

SCRIPT = Path(__file__).with_name("hud_timing_census.py")
SPEC = importlib.util.spec_from_file_location("hud_timing_census", SCRIPT)
assert SPEC and SPEC.loader
CENSUS = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(CENSUS)


def pc(frame: int, address: int, cycles: int, z80: int, wait: int) -> dict:
    return {
        "frame": frame,
        "pc": address,
        "m68k_cycles_before_instruction": cycles,
        "m68k_refresh_counter_before_instruction": cycles % 128,
        "z80_68k_wait_cpu_cycles_total": z80,
        "m68k_wait_cpu_cycles_total": wait,
        "scanline": frame,
        "pixel": cycles % 420,
    }


class HudTimingCensusTests(unittest.TestCase):
    def test_joins_previous_frame_mode_entry_to_hud_fifo_batch(self) -> None:
        report = {
            "clusters": [
                {"key": "hud_diamond", "frame_ranges": [[10, 11]]},
            ]
        }
        pcs = []
        fifos = []
        for frame, z80 in ((10, 0), (11, 7)):
            pcs.extend(
                [
                    pc(frame - 1, CENSUS.MODE_LEVEL_PC, 1000 * frame, z80, 20),
                    pc(frame, CENSUS.UPDATE_HUD_PC, 1000 * frame + 90, z80, 30),
                    pc(frame, CENSUS.DRAW_HUD_PC, 1000 * frame + 140, z80, 35),
                ]
            )
            fifos.append({"frame": frame, "first": {"scanline": 20, "pixel": 300}})

        result = CENSUS.build_census(report, pcs, fifos)

        self.assertEqual(result["frame_count"], 2)
        self.assertEqual(result["zero_z80_wait_both_intervals"], 2)
        self.assertEqual(result["frames"][0]["mode_to_update_hud"]["m68k_cycles"], 90)
        self.assertEqual(result["frames"][1]["first_fifo_commit"], {"scanline": 20, "pixel": 300})

    def test_rejects_an_incomplete_trace(self) -> None:
        report = {"clusters": [{"key": "hud_diamond", "frame_ranges": [[10, 10]]}]}

        with self.assertRaisesRegex(ValueError, "frame 10: missing trace record"):
            CENSUS.build_census(report, [], [])


if __name__ == "__main__":
    unittest.main()
