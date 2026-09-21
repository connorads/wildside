"""Case verdicts require an encounter, oracle evidence and a sensitive control."""

import hashlib
from concurrent.futures import ThreadPoolExecutor
import json
from pathlib import Path
import tempfile
import unittest
from unittest.mock import patch

import jev_case
import jev_case_service as service


class VerdictTests(unittest.TestCase):
    def test_distinct_drips_case_configures_level_and_never_proves_threshold_alone(self):
        self.assertEqual(jev_case.CASE_STARTS[502], 0x35)
        self.assertNotEqual(jev_case.LIVE_GOALS[502], jev_case.LIVE_GOALS[327])
        trace = [{"number_objects": 20}, {"number_objects": 21}]
        assessment = jev_case.encounter(502, trace)
        self.assertTrue(assessment["complete"])
        self.assertEqual(assessment["witness_frame"], 1)
        self.assertFalse(assessment["symptom_pass"])

    def test_run_pins_revision_and_runner_before_capture_changes_checkout(self):
        with tempfile.TemporaryDirectory() as directory:
            output = Path(directory)
            runner = output / "runner.py"
            runner.write_bytes(b"original runner")
            revision = [b"before\n"]

            def git(args, **_):
                return revision[0].decode() if args[1] == "rev-parse" else b"original patch"

            def capture(folder, *_):
                bundle = folder / "bundle"
                bundle.mkdir()
                jev_case.write_json(bundle / "trace.json", [{"current_level": 1}])
                jev_case.write_json(bundle / "manifest.json", {
                    "scenario": {"script": [0]}, "rom_sha1": "rom",
                    "script_sha1": "script", "trace_sha1": "trace"})
                # A hook or another commit can change disk state while an
                # already loaded runner and native binary finish their work.
                runner.write_bytes(b"changed runner")
                revision[0] = b"after\n"

            def verify(*_, **__):
                folder = output / "bundle/verify"
                folder.mkdir()
                jev_case.write_json(folder / "report.json", {})

            with patch.object(jev_case, "__file__", str(runner)), \
                    patch.object(jev_case.subprocess, "check_output", side_effect=git), \
                    patch.object(jev_case, "capture", side_effect=capture), \
                    patch.object(jev_case, "command", side_effect=verify):
                report = jev_case.run(output, 327, [0])

            self.assertEqual(report["provenance"]["revision"], "before")
            self.assertEqual(report["provenance"]["runner_sha256"],
                             hashlib.sha256(b"original runner").hexdigest())
            self.assertEqual((output / "source.patch").read_bytes(), b"original patch")

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
        # Oracle recovery may lag native recovery. Preserve the complete input
        # prefix and tail so the native lifecycle cannot shorten its own proof.
        longer_trace = [alive] * 40 + trace + [reload] * 20
        self.assertEqual(jev_case.encounter(370, longer_trace)["window"],
                         [0, len(longer_trace) - 1])
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
        service.persist(self.path, {"nanodollars_upper_bound": service.PRIOR, "attempts": []})
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

    def test_controller_actor_coordinates_use_pixels_and_journal_the_sent_state(self):
        # Saved decision72 placed the Tank at x=13139968 beside Kid x=58.
        # The controller promises pixels, so that Tank must be x=200.5.
        state = {
            "frame": 72, "kid": {"x": 58, "y": 191},
            "enemies": [{"x_pos": 13139968, "y_pos": 224 << 16,
                         "x_velocity": -32768, "y_velocity": None,
                         "hit_points": 2}],
            "projectiles": [{"x_pos": 0xFFFF8000, "y_pos": (191 << 16) + 16384,
                             "x_velocity": 65536, "y_velocity": -32768,
                             "friendly": False}],
        }
        original = json.loads(json.dumps(state))
        sent = []

        def evaluate(observation, key):
            sent.append(observation)
            return self.response()

        service.decision(self.token, state, "test", self.path, evaluate, now=101)

        self.assertEqual(sent[0]["enemies"], [{
            "x_pos": 200.5, "y_pos": 224.0, "x_velocity": -0.5,
            "y_velocity": None, "hit_points": 2}])
        self.assertEqual(sent[0]["projectiles"], [{
            "x_pos": -0.5, "y_pos": 191.25, "x_velocity": 1.0,
            "y_velocity": -0.5, "friendly": False}])
        self.assertEqual(sent[0]["kid"], state["kid"])
        self.assertEqual(state, original)
        journal = json.loads((self.output / "decisions.jsonl").read_text())
        self.assertEqual(journal["state"], sent[0])
        self.assertEqual(self.read()["nanodollars_upper_bound"], service.PRIOR + 4200)

    def test_failed_calls_remain_reserved(self):
        def fail(*_):
            raise TimeoutError()
        with self.assertRaises(TimeoutError):
            service.decision(self.token, {}, "test", self.path, fail, now=101)
        self.assertEqual(self.read()["nanodollars_upper_bound"], service.PRIOR + service.RESERVATION)

    def test_limits_stop_before_dispatch(self):
        ceiling = service.PRIOR + 5_000_000_000
        for change, now in (({"nanodollars_upper_bound": ceiling}, 101), ({}, 1300)):
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


class CampaignBudgetTests(unittest.TestCase):
    def setUp(self):
        self.directory = tempfile.TemporaryDirectory()
        self.addCleanup(self.directory.cleanup)
        self.output = Path(self.directory.name)
        self.path = self.output / "budget.json"
        self.opening = 176_526_462
        self.historical = [{"issue": 327, "token": f"historical-{i}",
                            "started": 100, "deadline": 1300, "requests": 600,
                            "output": "historical"} for i in range(3)]
        service.persist(self.path, {"nanodollars_upper_bound": self.opening,
                                    "attempts": self.historical})

    def read(self):
        return json.loads(self.path.read_text())

    def response(self, *_):
        return {"model": service.MODEL, "usage": {"input_tokens": 100},
                "answers": {"action": {"type": "choice", "choice": "release"}}}

    def test_campaign_retains_history_and_starts_new_attempt_window(self):
        token = service.begin_attempt(327, self.output, self.path, now=10_000)
        value = self.read()
        self.assertEqual(value["attempts"][:3], self.historical)
        self.assertEqual(value["nanodollars_upper_bound"], self.opening)
        self.assertEqual(value["campaign"], {
            "id": "single-player-completion-2026-09-20",
            "opening_nanodollars_upper_bound": self.opening,
            "allowance_nanodollars": 5_000_000_000})
        attempt = value["attempts"][-1]
        self.assertEqual(attempt["token"], token)
        self.assertEqual(attempt["deadline"], 11_200)
        self.assertEqual(attempt["campaign_id"], value["campaign"]["id"])

    def test_restarts_and_new_cases_do_not_replenish_campaign_allowance(self):
        for issue, now in [(327, 10_000), (370, 10_010), (327, 10_020)]:
            token = service.begin_attempt(issue, self.output, self.path, now=now)
            service.decision(token, {}, "test", self.path, self.response, now=now + 1)
        value = self.read()
        self.assertEqual(value["campaign"]["opening_nanodollars_upper_bound"], self.opening)
        self.assertEqual(value["nanodollars_upper_bound"], self.opening + 3 * 4200)
        self.assertEqual(value["attempts"][-1]["deadline"], 11_200)

    def test_additional_allowance_stops_before_dispatch_and_retains_failed_reservation(self):
        token = service.begin_attempt(327, self.output, self.path, now=10_000)
        value = self.read()
        ceiling = self.opening + 5_000_000_000
        value["nanodollars_upper_bound"] = ceiling - service.RESERVATION
        service.persist(self.path, value)

        def interrupted(*_):
            raise TimeoutError()

        with self.assertRaises(TimeoutError):
            service.decision(token, {}, "test", self.path, interrupted, now=10_001)
        self.assertEqual(self.read()["nanodollars_upper_bound"], ceiling)
        restarted = service.begin_attempt(370, self.output, self.path, now=10_002)
        with self.assertRaisesRegex(ValueError, "budget exhausted"):
            service.decision(restarted, {}, "test", self.path,
                             lambda *_: self.fail("dispatched after exhaustion"), now=10_003)
        self.assertEqual(self.read()["nanodollars_upper_bound"], ceiling)

    def test_historical_tokens_cannot_charge_campaign(self):
        service.begin_attempt(327, self.output, self.path, now=10_000)
        with self.assertRaisesRegex(ValueError, "campaign"):
            service.decision("historical-0", {}, "test", self.path,
                             lambda *_: self.fail("historical token dispatched"), now=101)
        self.assertEqual(self.read()["nanodollars_upper_bound"], self.opening)

    def test_campaign_attempt_limit_survives_restart(self):
        for now in (10_000, 10_001, 10_002):
            service.begin_attempt(327, self.output, self.path, now=now)
        with self.assertRaisesRegex(ValueError, "collection limit"):
            service.begin_attempt(327, self.output, self.path, now=10_003)
        with self.assertRaisesRegex(ValueError, "collection limit"):
            service.begin_attempt(327, self.output, self.path, now=20_000)
        self.assertEqual(len(self.read()["attempts"]), 6)

    def test_concurrent_starts_share_one_allocation_and_attempt_limit(self):
        def start(_):
            try:
                return service.begin_attempt(327, self.output, self.path, now=10_000)
            except ValueError:
                return None

        with ThreadPoolExecutor(max_workers=4) as pool:
            tokens = list(pool.map(start, range(4)))
        self.assertEqual(sum(token is not None for token in tokens), 3)
        value = self.read()
        self.assertEqual(len(value["attempts"]), 6)
        self.assertEqual(value["campaign"]["opening_nanodollars_upper_bound"], self.opening)
        self.assertEqual(value["nanodollars_upper_bound"], self.opening)

    def test_concurrent_failed_requests_cannot_exceed_remaining_credit(self):
        token = service.begin_attempt(327, self.output, self.path, now=10_000)
        ceiling = self.opening + 5_000_000_000
        value = self.read()
        value["nanodollars_upper_bound"] = ceiling - 2 * service.RESERVATION
        service.persist(self.path, value)

        def interrupted(*_):
            raise TimeoutError()

        def request(_):
            try:
                service.decision(token, {}, "test", self.path, interrupted, now=10_001)
            except (ValueError, TimeoutError) as error:
                return type(error)
            self.fail("interrupted request unexpectedly completed")

        with ThreadPoolExecutor(max_workers=4) as pool:
            outcomes = list(pool.map(request, range(4)))
        self.assertEqual(outcomes.count(TimeoutError), 2)
        self.assertEqual(outcomes.count(ValueError), 2)
        self.assertEqual(self.read()["nanodollars_upper_bound"], ceiling)
        self.assertEqual(self.read()["attempts"][-1]["requests"], 2)

    def test_invalid_allocation_never_dispatches_or_reallocates(self):
        token = service.begin_attempt(327, self.output, self.path, now=10_000)
        original = self.read()
        for campaign in [None, {}, original["campaign"] | {"id": "another-campaign"},
                         original["campaign"] | {"allowance_nanodollars": 10_000_000_000},
                         original["campaign"] | {"opening_nanodollars_upper_bound": self.opening + 1}]:
            with self.subTest(campaign=campaign):
                invalid = original | {"campaign": campaign}
                service.persist(self.path, invalid)
                with self.assertRaises(ValueError):
                    service.decision(token, {}, "test", self.path,
                                     lambda *_: self.fail("invalid allocation dispatched"), now=10_001)
                with self.assertRaises(ValueError):
                    service.begin_attempt(370, self.output, self.path, now=10_002)
                self.assertEqual(self.read(), invalid)

    def test_missing_ledger_cannot_restart_campaign_credit(self):
        missing = self.output / "missing.json"
        with self.assertRaisesRegex(ValueError, "ledger.*missing"):
            service.begin_attempt(327, self.output, missing, now=10_000)
        with self.assertRaisesRegex(ValueError, "ledger.*missing"):
            service.decision("historical-0", {}, "test", missing,
                             lambda *_: self.fail("missing ledger dispatched"), now=10_001)
        self.assertFalse(missing.exists())


if __name__ == "__main__":
    unittest.main()
