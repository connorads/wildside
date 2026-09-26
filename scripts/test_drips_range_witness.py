"""The range witness must pair a Drips call with its actual ROM branch."""

import unittest

from drips_range_witness import CALLS, CLOSE, ENTRY, NORMAL, classify, summarise


def hit(pc, frame=7, a3=0xFFFF2578):
    return {"pc": pc, "frame": frame, "address_registers": [0, 0, 0, a3]}


class DripsRangeWitnessTests(unittest.TestCase):
    def test_exact_call_entry_branch_sequence(self):
        rows = [hit(0x32054), hit(0x33460), hit(ENTRY), hit(NORMAL),
                hit(0x3351A, a3=0xFFFF265C), hit(ENTRY, a3=0xFFFF265C),
                hit(CLOSE, a3=0xFFFF265C)]
        self.assertEqual([call["branch"] for call in classify(rows)], ["normal", "close"])
        self.assertEqual(classify(rows)[1]["object_a3"], 0xFFFF265C)

    def test_rejects_missing_branch_or_wrong_object(self):
        call = hit(min(CALLS))
        for rows in ([call], [call, hit(ENTRY)],
                     [call, hit(ENTRY, a3=0xFFFF265C)],
                     [call, hit(ENTRY), hit(CLOSE, frame=8)]):
            with self.subTest(rows=rows), self.assertRaises(ValueError):
                classify(rows)

    def test_requires_both_object_threshold_crossings(self):
        calls = []
        for frame, branch in [(10, NORMAL), (11, CLOSE), (12, CLOSE), (13, NORMAL)]:
            calls.extend(classify([
                hit(min(CALLS), frame=frame),
                hit(ENTRY, frame=frame),
                hit(branch, frame=frame),
            ]))

        summary = summarise(calls)
        self.assertEqual(summary["transitions"], ["normal->close", "close->normal"])
        self.assertTrue(summary["interleaved_threshold_witness"])

    def test_one_close_call_is_not_an_interleaved_witness(self):
        calls = classify([
            hit(min(CALLS), frame=10), hit(ENTRY, frame=10), hit(CLOSE, frame=10),
        ])
        self.assertFalse(summarise(calls)["interleaved_threshold_witness"])


if __name__ == "__main__":
    unittest.main()
