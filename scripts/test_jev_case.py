"""Case verdicts require an encounter, oracle evidence and a sensitive control."""

import json
from pathlib import Path
import tempfile
import unittest

import jev_case
import jev_case_service as service


class VerdictTests(unittest.TestCase):
    def test_missing_teleporter_keeps_only_a_diagnostic_tail(self):
        ordinary = dict(current_level=1, kid_x_pos=32 << 16, kid_y_pos=191 << 16)
        trace = [ordinary] * 3600
        self.assertEqual(jev_case.encounter(327, trace)["window"], [3540, 3599])
        trace[100] = dict(current_level=1, kid_x_pos=2503 << 16,
                          kid_y_pos=575 << 16, kid_on_ground=True)
        trace[250] = dict(current_level=1, kid_x_pos=448 << 16, teleport_warp_colour=238)
        for index in range(100, 281):
            trace[index] = trace[index] | {"game_mode": "InGame"}
        self.assertEqual(jev_case.encounter(327, trace)["window"], [70, 280])
        self.assertTrue(jev_case.encounter(327, trace)["complete"])

    def test_running_controls_press_a_without_c(self):
        self.assertEqual(set(service.QUESTION["criteria"]), set(service.MASKS))
        # FrameInput uses A=0x40, B=0x10 and C=0x20.
        for name, direction in (("run_right", 8), ("run_left", 4)):
            with self.subTest(name=name):
                self.assertEqual(service.MASKS[name], 0x40 | direction)
                self.assertEqual(service.MASKS[name + "_jump"], 0x40 | 0x10 | direction)

    def test_control_removes_only_the_respawn_projectile_clear(self):
        source = """outside.clear();
    pub(crate) fn reset_current_level_runtime_state_for_respawn(&mut self) {
        self.projectiles.clear();
        self.enemies.clear();
    }
    pub fn another(&mut self) {
        self.projectiles.clear();
    }
"""
        mutant = jev_case.mutate(370, source)
        self.assertEqual(mutant.count("self.projectiles.clear();"), 1)
        self.assertIn("self.enemies.clear();", mutant)
        with self.assertRaises(ValueError):
            jev_case.mutate(370, "unrecognised source")

    def test_all_three_evidence_layers_are_required(self):
        assessment = {"complete": True, "symptom_pass": True}
        comparison = dict(trace_match=True, oracle_state_match=True, oracle_frame_match=True,
                          native_replay_only=False, frame_diffs=[{}])
        control = dict(verified=True, same_check_failed=True)
        self.assertEqual(jev_case.verdict(assessment, comparison, control), "proven check")
        self.assertEqual(jev_case.verdict(assessment, comparison, None), "negative control required")
        self.assertEqual(jev_case.verdict(assessment | {"complete": False}, comparison, control), "not exercised")
        for field in ("trace_match", "oracle_state_match", "oracle_frame_match", "frame_diffs"):
            with self.subTest(field=field):
                self.assertEqual(jev_case.verdict(assessment, comparison | {field: False}, control), "comparison inconclusive")
        self.assertEqual(jev_case.verdict(assessment, comparison | {"native_replay_only": True}, control), "comparison inconclusive")

    def test_dragon_case_requires_fatal_projectile_then_reload_and_clean_tail(self):
        alive = dict(game_mode="InGame", number_lives=3, number_hitpoints=2,
                     gameplay_debug_fields={"kid_gameplay_path_probe": "lifecycle=alive/motion=normal_physics/post=none/paused=0"})
        fatal = alive | {"number_hitpoints": 0, "active_projectile_details": [{"kind": "DragonWalkingBreath"}],
                         "gameplay_debug_fields": {"kid_gameplay_path_probe": "lifecycle=death_windup/motion=normal_physics/post=projectile_damage/paused=0"}}
        shell = alive | {"game_mode": "LevelTransition", "number_lives": 2,
                         "gameplay_debug_fields": {"kid_gameplay_path_probe": "lifecycle=respawning/motion=none/post=none/paused=0"}}
        reload = alive | {"number_lives": 2}
        trace = [alive, fatal, shell] + [reload] * 31
        self.assertTrue(jev_case.encounter(370, trace)["symptom_pass"])
        self.assertFalse(jev_case.encounter(370, trace[:3])["complete"])
        self.assertFalse(jev_case.encounter(370, [alive, fatal] + [shell | {"game_mode": "InGame"}] * 100)["complete"])
        trace[3] = reload | {"active_projectile_details": fatal["active_projectile_details"]}
        self.assertFalse(jev_case.encounter(370, trace)["symptom_pass"])
        trace[1] = fatal | {"gameplay_debug_fields": {}}
        self.assertFalse(jev_case.encounter(370, trace)["complete"])

    def test_report_embeds_actual_numbered_capture_paths(self):
        with tempfile.TemporaryDirectory() as directory:
            output = Path(directory)
            for side in ("native", "oracle"):
                folder = output / "bundle/verify" / side
                folder.mkdir(parents=True)
                width = 6 if side == "native" else 4
                (folder / f"{302:0{width}}_case-302.png").write_bytes(b"capture")
            jev_case.render_report(output, {"verdict": "<not a pass>", "comparison": {
                "frame_diffs": [{"label": "case-302", "frame_index": 302}]}})
            document = (output / "report.html").read_text()
            self.assertEqual(document.count("data:image/png;base64,"), 2)
            self.assertNotIn("<not a pass>", document)


class BudgetTests(unittest.TestCase):
    def setUp(self):
        self.directory = tempfile.TemporaryDirectory()
        self.addCleanup(self.directory.cleanup)
        self.output = Path(self.directory.name)
        self.path = self.output / "budget.json"
        self.token = service.begin_attempt(327, self.output, self.path, now=100)

    def response(self, *_):
        return {"model": service.MODEL, "usage": {"input_tokens": 100},
                "answers": {"action": {"type": "choice", "choice": "right_jump"}}}

    def read(self):
        return json.loads(self.path.read_text())

    def test_restarts_preserve_prior_spend_and_usage_reconciles(self):
        self.assertEqual(service.decision(self.token, {}, "test", self.path, self.response, now=101), 24)
        self.assertEqual(self.read()["nanodollars_upper_bound"], service.PRIOR + 4200)
        service.begin_attempt(327, self.output, self.path, now=102)
        self.assertEqual(self.read()["nanodollars_upper_bound"], service.PRIOR + 4200)

    def test_failed_calls_remain_reserved(self):
        def fail(*_):
            raise TimeoutError()
        with self.assertRaises(TimeoutError):
            service.decision(self.token, {}, "test", self.path, fail, now=101)
        self.assertEqual(self.read()["nanodollars_upper_bound"], service.PRIOR + service.RESERVATION)

    def test_limits_stop_before_dispatch(self):
        for change, now in (({"nanodollars_upper_bound": service.LIMIT}, 101), ({}, 1300)):
            with self.subTest(change=change):
                value = self.read() | change
                service.persist(self.path, value)
                with self.assertRaises(ValueError):
                    service.decision(self.token, {}, "test", self.path, lambda *_: self.fail("dispatched"), now=now)
        for _ in range(2):
            service.begin_attempt(327, self.output, self.path, now=102)
        with self.assertRaises(ValueError):
            service.begin_attempt(327, self.output, self.path, now=103)

    def test_invalid_usage_never_releases_reservation(self):
        with self.assertRaises(ValueError):
            service.decision(self.token, {}, "test", self.path, lambda *_: {}, now=101)
        self.assertEqual(self.read()["nanodollars_upper_bound"], service.PRIOR + service.RESERVATION)


if __name__ == "__main__":
    unittest.main()
