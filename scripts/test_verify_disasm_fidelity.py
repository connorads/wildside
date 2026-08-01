#!/usr/bin/env python3

import importlib.util
import sys
import unittest
from pathlib import Path

SCRIPT = Path(__file__).with_name("verify_disasm_fidelity.py")
SPEC = importlib.util.spec_from_file_location("verify_disasm_fidelity", SCRIPT)
assert SPEC and SPEC.loader
FIDELITY = importlib.util.module_from_spec(SPEC)
# Register before exec: @dataclass resolves annotations through sys.modules, and
# fails on an unregistered module.
sys.modules[SPEC.name] = FIDELITY
SPEC.loader.exec_module(FIDELITY)

# Shape of an AS `-g map` debug map: per-file `line:address` pair lists under a
# segment header, then one symbol table per segment. Only CODE symbols are ROM
# addresses; NOTHING holds equates and RAM offsets. AS writes include paths
# absolute, which the parser shortens to disassembly-relative.
MAP = f"""Segment CODE
File kid.asm
   69:00000000    70:00000004    71:00000008
File {FIDELITY.DISASM_DIR}/code/enemy/Dragon.asm
   28:0003CD70    29:0003CD74

Symbols in Segment NOTHING
x_pos                                 Int    1C                        -1  1 0

Symbols in Segment CODE
EntryPoint                            Int    0                         -1  1 0
loc_3CD70                             Int    3CD70                     -1  1 0
"""


class DiffRegionTests(unittest.TestCase):
    def test_identical_images_report_no_regions(self) -> None:
        self.assertEqual(FIDELITY.diff_regions(b"kid", b"kid"), [])

    def test_adjacent_differences_coalesce_into_one_region(self) -> None:
        regions = FIDELITY.diff_regions(b"\x00\x01\x02\x03", b"\x00\xff\xfe\x03", gap=4)

        self.assertEqual(regions, [FIDELITY.Region(1, 3)])
        self.assertEqual(regions[0].length, 2)

    def test_differences_beyond_the_gap_stay_separate(self) -> None:
        reference = bytes(8)
        built = bytearray(8)
        built[0] = 0xFF
        built[7] = 0xFF

        self.assertEqual(
            FIDELITY.diff_regions(reference, bytes(built), gap=4),
            [FIDELITY.Region(0, 1), FIDELITY.Region(7, 8)],
        )

    def test_size_mismatch_reports_the_unmatched_tail(self) -> None:
        self.assertEqual(
            FIDELITY.diff_regions(b"kid!", b"kid"), [FIDELITY.Region(3, 4)]
        )


class SourceMapTests(unittest.TestCase):
    def setUp(self) -> None:
        self.source_map = FIDELITY.parse_source_map(MAP)

    def test_parses_code_symbols_and_ignores_other_segments(self) -> None:
        self.assertEqual(
            self.source_map.labels,
            [(0x0, "EntryPoint"), (0x3CD70, "loc_3CD70")],
        )

    def test_attributes_an_exact_label_hit_with_its_source_line(self) -> None:
        self.assertEqual(
            self.source_map.describe(0x3CD70),
            "loc_3CD70  code/enemy/Dragon.asm:28",
        )

    def test_attributes_an_interior_offset_to_the_preceding_label(self) -> None:
        self.assertEqual(
            self.source_map.describe(0x3CD72),
            "loc_3CD70+0x2  code/enemy/Dragon.asm:28",
        )

    def test_offsets_below_every_entry_are_unattributed(self) -> None:
        empty = FIDELITY.parse_source_map("")

        self.assertEqual(empty.describe(0x10), "<unattributed>")


if __name__ == "__main__":
    unittest.main()
