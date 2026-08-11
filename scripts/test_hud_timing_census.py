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
            ],
            "per_frame": [
                {"frame_index": 10, "changed_pixels": 2, "region": "hud_diamond"},
                {"frame_index": 11, "changed_pixels": 4, "region": "hud_diamond"},
            ],
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

        result = CENSUS.build_census(report, pcs, fifos, ["hud_diamond"])
        cluster = result["clusters"][0]

        self.assertEqual(result["frame_count"], 2)
        self.assertEqual(result["zero_z80_wait_both_intervals"], 2)
        self.assertEqual(cluster["frames"][0]["mode_to_update_hud"]["m68k_cycles"], 90)
        self.assertEqual(cluster["frames"][1]["first_fifo_commit"], {"scanline": 20, "pixel": 300})

    def test_default_census_includes_both_hud_timing_clusters(self) -> None:
        report = {
            "clusters": [
                {"key": "hud_diamond", "frame_ranges": [[10, 10]]},
                {"key": "hud_life_icon", "frame_ranges": [[11, 11]]},
            ],
            "per_frame": [
                {"frame_index": 10, "changed_pixels": 2, "region": "hud_diamond"},
                {"frame_index": 11, "changed_pixels": 3, "region": "hud_life_icon"},
            ],
        }
        pcs = []
        fifos = []
        for frame in (10, 11):
            pcs.extend(
                [
                    pc(frame - 1, CENSUS.MODE_LEVEL_PC, 1000 * frame, 0, 20),
                    pc(frame, CENSUS.UPDATE_HUD_PC, 1000 * frame + 90, 0, 30),
                    pc(frame, CENSUS.DRAW_HUD_PC, 1000 * frame + 140, 0, 35),
                ]
            )
            fifos.append({"frame": frame, "first": {"scanline": 20, "pixel": 300}})

        result = CENSUS.build_census(report, pcs, fifos)

        self.assertEqual(
            [cluster["cluster"] for cluster in result["clusters"]],
            ["hud_diamond", "hud_life_icon"],
        )
        self.assertEqual(result["frame_count"], 2)

    def test_rejects_an_incomplete_trace(self) -> None:
        report = {
            "clusters": [{"key": "hud_diamond", "frame_ranges": [[10, 10]]}],
            "per_frame": [
                {"frame_index": 10, "changed_pixels": 2, "region": "hud_diamond"}
            ],
        }

        with self.assertRaisesRegex(ValueError, "frame 10: missing trace record"):
            CENSUS.build_census(report, [], [], ["hud_diamond"])

    def test_manifest_fifo_summaries_split_hud_destination_ranges(self) -> None:
        manifest = {
            "vdp_fifo_commits": [
                {
                    "frame": 10,
                    "commits": [
                        {"target": "vram", "address": 0x4620, "scanline": 20, "pixel": 10},
                        {"target": "vram", "address": 0xDD40, "scanline": 30, "pixel": 20},
                    ],
                }
            ]
        }

        summaries = CENSUS.manifest_fifo_summaries(manifest, CENSUS.DEFAULT_CLUSTERS)

        self.assertEqual(summaries["hud_life_icon"][0]["first"]["address"], 0x4620)
        self.assertEqual(summaries["hud_diamond"][0]["first"]["address"], 0xDD40)


if __name__ == "__main__":
    unittest.main()
