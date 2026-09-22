"""Private run specifications preserve ordinary inputs and explicit evidence scope."""

import unittest
import hashlib
import tempfile
from pathlib import Path
from unittest.mock import patch
import jev_spec

from jev_spec import RunSpec, load_scenario
import jev_case


class SpecificationTests(unittest.TestCase):
    def test_new_issue_can_collect_without_a_proof_evaluator(self):
        spec = RunSpec.parse({"issue": 344, "start_level": 0, "goal": "Stomp a walking Dragon"})
        self.assertEqual(spec.issue, 344)
        self.assertEqual(spec.start_level, 0)
        self.assertEqual(jev_case.verdict(None, {"trace_match": True}, None), "not evaluated")

    def test_spec_rejects_unknown_fields_and_invalid_identity(self):
        for value in ({"issue": True}, {"issue": -1}, {"issue": 344, "start_level": 256},
                      {"issue": 344, "teleport": True}):
            with self.subTest(value=value), self.assertRaises(ValueError):
                RunSpec.parse(value)

    def test_ordinary_scenario_retains_inputs_start_and_checkpoints(self):
        scenario = {"script": [16, 16, 0], "frame_count": 3, "warp_to_level": 2,
                    "settle_oracle_warp": True,
                    "checkpoints": [{"frame_index": 1, "label": "held-jump"}]}
        self.assertEqual(load_scenario(scenario), (2, [16, 16, 0], [1]))

    def test_injected_or_ambiguous_scenarios_are_rejected(self):
        base = {"script": [0], "frame_count": 1}
        for key, value in (("start_position", {"x": 1, "y": 2}), ("start_velocity", {}),
                           ("camera_position", {}), ("playtest_form", 0),
                           ("initial_lives", 0), ("initial_diamond_count", 0),
                           ("frame_count", 2), ("script", [True]), ("typo", 1),
                           ("warp_to_level", 1), ("checkpoints", [{"frame_index": 1}])):
            with self.subTest(key=key), self.assertRaises(ValueError):
                load_scenario(base | {key: value})

    def test_green_replay_is_distinct_from_proof(self):
        comparison = {"trace_match": True, "oracle_state_match": True,
                      "oracle_frame_match": True, "frame_diffs": [{"frame_index": 0}]}
        self.assertTrue(jev_case.comparable(comparison))
        self.assertFalse(jev_case.comparable(comparison | {"frame_diffs": []}))
        self.assertFalse(jev_case.comparable(comparison | {"native_replay_only": True}))
        self.assertEqual(jev_case.verdict(None, comparison, None), "not evaluated")

    def test_prefix_spec_and_dense_handoff_keep_existing_checkpoints(self):
        spec = RunSpec.parse({"issue": 344, "goal": "Reach the Dragon", "prefix": "route.json", "max_decisions": 2})
        self.assertEqual(spec.prefix, "route.json")
        self.assertEqual(spec.max_decisions, 2)
        self.assertEqual(jev_case.preflight_frames(40, [2, 8]), [0, 2, 8, *range(10, 40)])
        self.assertEqual(jev_case.preflight_frames(1, []), [0])
        with self.assertRaises(ValueError):
            RunSpec.parse({"issue": 344, "prefix": "route.json", "start_level": 0})
        with self.assertRaises(ValueError):
            RunSpec.parse({"issue": 344, "max_decisions": 601})

    def test_preflight_requires_every_declared_checkpoint(self):
        comparison = {"trace_match": True, "oracle_state_match": True, "oracle_frame_match": True,
                      "frame_diffs": [{"frame_index": 0}, {"frame_index": 5}]}
        self.assertTrue(jev_case.comparable(comparison, [0, 5]))
        self.assertFalse(jev_case.comparable(comparison, [0, 4, 5]))

    def test_source_guard_rejects_revision_patch_and_untracked_code_changes(self):
        identity = {"revision": "original", "source_patch_sha256": hashlib.sha256(b"patch").hexdigest()}
        for revision, diff, untracked, valid in (("original", b"patch", b"", True),
                ("changed", b"patch", b"", False), ("original", b"different", b"", False),
                ("original", b"patch", b"scripts/new.py", False)):
            with patch.object(jev_spec.subprocess, "check_output", side_effect=[revision, diff, untracked]):
                if valid:
                    jev_spec.assert_source_current(Path("."), identity)
                else:
                    with self.assertRaises(ValueError):
                        jev_spec.assert_source_current(Path("."), identity)

    def test_red_or_incomplete_preflight_never_opens_a_paid_attempt(self):
        for comparison in ({}, {"trace_match": True, "oracle_state_match": True,
                              "oracle_frame_match": True, "frame_diffs": [{"frame_index": 0}]}):
            with tempfile.TemporaryDirectory() as directory:
                output = Path(directory)
                def capture(folder, spec, inputs, frames, env):
                    self.assertNotIn("TYPESAFE_API_KEY", env)
                    self.assertEqual(inputs, [16, 0])
                    self.assertEqual(frames, [0, 1])
                with patch.object(jev_case, "assert_source_current"), \
                     patch.object(jev_case, "capture", side_effect=capture), \
                     patch.object(jev_case, "command"), \
                     patch.object(jev_case, "read_json", return_value=comparison), \
                     patch.object(jev_case.jev_case_service, "begin_attempt") as paid:
                    with self.assertRaisesRegex(RuntimeError, "blocked"):
                        jev_case.run(output, RunSpec(344, 0, "Stomp Dragon"), [16, 0], "discover")
                    paid.assert_not_called()

    def test_discovery_policy_rejects_unknown_controls_and_unbounded_waits(self):
        spec = RunSpec.parse({"issue": 344, "actions": ["release", "up", "special"],
                              "target": {"kind": "position", "level": 0, "x": 32, "y": 191,
                                         "radius": 16, "dwell_frames": 90}})
        self.assertEqual(spec.progress, "cells")
        self.assertEqual(spec.tail_frames, 30)
        self.assertEqual(spec.actions, ("release", "up", "special"))
        for change in ({"actions": ["teleport"]}, {"actions": ["right"]},
                       {"progress": "anything"}, {"stop_on_death": 1},
                       {"target": {"kind": "position", "level": 0, "x": 32, "y": 191,
                                   "radius": 16, "dwell_frames": 0}},
                       {"target": {"kind": "unknown"}}, {"tail_frames": -1}):
            with self.subTest(change=change), self.assertRaises(ValueError):
                RunSpec.parse({"issue": 344} | change)


if __name__ == "__main__":
    unittest.main()
