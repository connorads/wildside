"""Private run specifications preserve ordinary inputs and explicit evidence scope."""

import unittest

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


if __name__ == "__main__":
    unittest.main()
