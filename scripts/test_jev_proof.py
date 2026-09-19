"""Verdicts must distinguish a detected defect from two passing builds."""

import copy
import json
import unittest
from pathlib import Path
import tempfile

import jev_proof


class VerdictTests(unittest.TestCase):
    def setUp(self):
        self.evidence = {
            "mutation_verified": True,
            "encounter_complete": True,
            "trace_equal": True,
            "state_equal": True,
            "captures_complete": True,
            "fixed_changed_pixels": 0,
            "broken_changed_pixels": 12,
            "target_changed_pixels": 12,
        }

    def test_requires_broken_and_fixed_contrast(self):
        self.assertEqual(jev_proof.verdict(self.evidence), "proof passed")
        self.evidence.update(broken_changed_pixels=0, target_changed_pixels=0)
        self.assertEqual(jev_proof.verdict(self.evidence), "case did not expose the bug")

    def test_missing_or_drifting_evidence_cannot_pass(self):
        for key in ("encounter_complete", "trace_equal", "state_equal", "captures_complete"):
            evidence = copy.copy(self.evidence)
            evidence[key] = False
            with self.subTest(key=key):
                self.assertEqual(jev_proof.verdict(evidence), "comparison inconclusive")

    def test_fixed_differences_and_changes_outside_target_are_inconclusive(self):
        for change in ({"fixed_changed_pixels": 1}, {"target_changed_pixels": 0}):
            self.assertEqual(jev_proof.verdict(self.evidence | change), "comparison inconclusive")

    def test_ineffective_mutation_is_execution_failure(self):
        self.evidence["mutation_verified"] = False
        self.assertEqual(jev_proof.verdict(self.evidence), "execution failed")

    def test_target_counts_do_not_include_unrelated_pixels(self):
        fixed = bytes(320 * 224 * 3)
        broken = bytearray(fixed)
        broken[0] = 1
        broken[(20 * 320 + 10) * 3] = 1
        self.assertEqual(jev_proof.pixel_counts(fixed, bytes(broken), (10, 20, 16, 36)), (2, 1))

    def test_report_embeds_images_and_escapes_diagnostic_text(self):
        with tempfile.TemporaryDirectory() as directory:
            output = Path(directory)
            (output / "frame.png").write_bytes(b"png evidence")
            report = {"verdict": "comparison inconclusive", "note": "<script>bad()</script>",
                      "frames": [{"frame": 376, "fixed_changed_pixels": 0,
                                  "broken_changed_pixels": 1, "target_changed_pixels": 1,
                                  "images": {name: "frame.png" for name in ("fixed", "broken", "oracle")}}]}
            jev_proof.render_report(output, report)
            document = (output / "report.html").read_text()
            self.assertIn("data:image/png;base64,", document)
            self.assertNotIn("<script>bad()</script>", document)
            self.assertIn("comparison inconclusive", document)

    def test_missing_capture_files_are_inconclusive(self):
        with tempfile.TemporaryDirectory() as directory:
            output = Path(directory)
            checkpoints = [{"frame_index": frame, "label": f"jev-{frame}"} for frame in range(31)]
            report = {"trace_match": True, "oracle_state_match": True, "frame_diffs": checkpoints}
            for side in ("fixed", "broken"):
                native = output / side / "verify/native"
                native.mkdir(parents=True)
                (native / "trace.json").write_text(json.dumps([{}] * 31))
                (native.parent / "report.json").write_text(json.dumps(report))
            (output / "fixed/scenario.json").write_text(json.dumps({"checkpoints": checkpoints}))
            evidence, rows = jev_proof.inspect_comparison(output, {
                "first_hit_frame": 0, "last_animation_frame": 0,
                "complete_window": True, "target_cell": [8, 12]})
            self.assertFalse(evidence["captures_complete"])
            self.assertEqual(rows, [])
            self.assertEqual(jev_proof.verdict(evidence), "comparison inconclusive")

    def test_mutation_refuses_unrecognised_source(self):
        with self.assertRaises(ValueError):
            jev_proof.restore_defect("unrelated source")


if __name__ == "__main__":
    unittest.main()
