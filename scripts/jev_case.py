"""Run one private bug case through the existing recording/oracle pipeline."""

import argparse
from dataclasses import asdict, replace
import base64
import html
import json
import os
from pathlib import Path
import subprocess
import difflib
import sys
import tempfile
import time

from jev_audit import ROOT, audit
from jev_proof import command, read_json, sha256, write_json
import jev_case_service
from jev_spec import RunSpec, load_scenario, assert_source_current

CASE_STARTS = {210: None, 254: None, 327: 1, 370: 0, 502: 0x35}


def route(issue):
    if issue in (210, 254):
        return [128 if frame == 2400 or (2700 <= frame <= 2730 and frame % 2 == 0)
                else 0 for frame in range(3300)]
    if issue == 370:
        inputs = [0] * 620
        inputs[500] = 128
        return inputs
    return None


def breath(frame):
    return [p for p in frame.get("active_projectile_details", []) if p["kind"] == "DragonWalkingBreath"]


def encounter(issue, trace):
    if issue == 502:
        witness = next((i for i, frame in enumerate(trace)
                        if frame.get("number_objects", 0) >= 21), None)
        return {"complete": witness is not None, "witness_frame": witness,
                "symptom_pass": False,
                "window": [max(0, (witness if witness is not None else len(trace) - 1) - 30),
                           min(len(trace) - 1, (witness if witness is not None else len(trace) - 1) + 30)],
                "scope": "Drips live Object count threshold witness; range-call parity still required"}
    if issue in (210, 254):
        complete = len(trace) >= 3300 and any(f["game_mode"] == "TitleScreen" for f in trace[:2718])
        return {"complete": complete, "symptom_pass": complete and all(
            trace[i]["game_mode"] == "LevelTransition" for i in (2732, 3000, 3299)),
            "window": [2700, 3299], "scope": "rapid Start does not release early gameplay"}
    if issue == 370:
        fatal = next((i for i in range(1, len(trace)) if trace[i - 1]["number_hitpoints"] > 0
                      and trace[i]["number_hitpoints"] == 0 and breath(trace[i])
                      and "post=projectile_damage/" in trace[i].get("gameplay_debug_fields", {}).get("kid_gameplay_path_probe", "")), None)
        reload_frame = next((i for i in range((fatal or 0) + 1, len(trace))
                            if fatal is not None and trace[i]["number_lives"] < trace[fatal]["number_lives"]
                            and trace[i].get("gameplay_debug_fields", {}).get("kid_gameplay_path_probe", "").startswith("lifecycle=respawning/")), None)
        live_frame = next((i for i in range((reload_frame or 0) + 1, len(trace))
                          if reload_frame is not None and trace[i]["game_mode"] == "InGame"
                          and trace[i].get("gameplay_debug_fields", {}).get("kid_gameplay_path_probe", "").startswith("lifecycle=alive/")), None)
        complete = live_frame is not None and len(trace) > live_frame + 30
        return {"complete": complete, "fatal_frame": fatal, "reload_frame": reload_frame, "live_frame": live_frame,
                "symptom_pass": complete and all(not breath(f) and f["number_hitpoints"] > 0
                    for f in trace[reload_frame:live_frame + 31]),
                "window": [0, len(trace) - 1],
                "scope": "natural breath fatal hit and ordinary life-loss reload, excluding continue"}
    if issue == 327:
        contact = next((i for i, f in enumerate(trace) if f.get("current_level") == 1
                        and abs((f.get("kid_x_pos") or 0) / 65536 - 2503) < 16
                        and abs((f.get("kid_y_pos") or 0) / 65536 - 575) < 16
                        and f.get("kid_on_ground")), None)
        relocated = next((i for i in range((contact or 0) + 1, len(trace)) if contact is not None
                          and trace[i].get("current_level") == 1
                          and abs((trace[i].get("kid_x_pos") or 0) / 65536 - 448) < 24
                          and trace[i].get("teleport_warp_colour") == 238), None)
        complete = relocated is not None and len(trace) > relocated + 30
        return {"complete": complete, "contact_frame": contact, "relocated_frame": relocated,
                "symptom_pass": complete and all(f["game_mode"] == "InGame"
                    for f in trace[contact:relocated + 31]),
                "window": [max(0, contact - 30) if contact is not None else max(0, len(trace) - 60),
                           min(len(trace) - 1, (relocated if relocated is not None else len(trace) - 31) + 30)],
                "scope": "natural BLW2 same-level teleporter, excluding cross-level warp"}
    return None


def comparable(comparison, frames=()):
    return bool(comparison and not comparison.get("native_replay_only")
                and all(comparison.get(key) for key in
                        ("trace_match", "oracle_state_match", "oracle_frame_match"))
                and comparison.get("frame_diffs")
                and set(frames) <= {row.get("frame_index") for row in comparison["frame_diffs"]})



def preflight_frames(length, checkpoints):
    return sorted({0, *checkpoints, *range(max(0, length - 30), length)})

def verdict(assessment, comparison, sensitivity):
    if assessment is None:
        return "not evaluated"
    if not assessment["complete"]:
        return "not exercised"
    if not comparable(comparison):
        return "comparison inconclusive"
    if not assessment["symptom_pass"]:
        return "symptom failed"
    if not sensitivity or not sensitivity.get("verified") or not sensitivity.get("same_check_failed"):
        return "negative control required"
    return "proven check"


def render_report(output, report):
    images = []
    for frame in report.get("comparison", {}).get("frame_diffs", []):
        label = frame["label"]
        pair = {}
        for side in ("native", "oracle"):
            width = 6 if side == "native" else 4
            path = output / "bundle/verify" / side / f"{frame['frame_index']:0{width}}_{label}.png"
            if path.is_file():
                pair[side] = "data:image/png;base64," + base64.b64encode(path.read_bytes()).decode()
        if len(pair) == 2:
            images.append({"frame": frame["frame_index"], **pair})
    document = '<!doctype html><html lang="en"><meta charset="utf-8"><title>Jev case evidence</title>'
    document += '<style>body{font:16px system-ui;max-width:1000px;margin:2rem auto}img{width:45%;image-rendering:pixelated}pre{white-space:pre-wrap}</style>'
    summary = report.copy()
    if "comparison" in summary:
        comparison = summary["comparison"]
        summary["comparison"] = {key: value for key, value in comparison.items()
                                 if key not in ("frame_diffs", "checkpoint_state_deltas")}
        summary["comparison"]["capture_count"] = len(comparison.get("frame_diffs", []))
        summary["comparison"]["changed_pixels"] = sum(frame.get("changed_pixel_count", 0) for frame in comparison.get("frame_diffs", []))
    document += f'<h1>{html.escape(report["verdict"])}</h1><pre>{html.escape(json.dumps(summary, indent=2))}</pre>'
    document += '<p id="frame"></p><input id="slider" type="range" min="0" value="0"><div><img id="native" alt="Native"><img id="oracle" alt="Oracle"></div>'
    document += '<script>const rows=' + json.dumps(images).replace('<', '\\u003c') + ';const slider=document.getElementById("slider");slider.max=Math.max(0,rows.length-1);function show(){const r=rows[Number(slider.value)];if(!r)return;document.getElementById("frame").textContent="Frame "+r.frame;for(const side of ["native","oracle"])document.getElementById(side).src=r[side];}slider.addEventListener("input",show);show();</script></html>'
    (output / "report.html").write_text(document)


def capture(output, spec, inputs, checkpoints, env, live=False):
    issue = spec.issue
    config = output / "input.json"
    write_json(config, {"issue": issue, "output": str(output / "bundle"), "script": inputs,
                        "checkpoints": checkpoints, "live": live, "start_level": spec.start_level,
                        "goal": spec.goal, "max_decisions": spec.max_decisions,
                        "actions": spec.actions, "progress": spec.progress, "target": spec.target,
                        "tail_frames": spec.tail_frames, "stop_on_death": spec.stop_on_death})
    command(["cargo", "test", "-q", "-p", "wildside-cli", "recording::jev_cases::case_bundle",
             "--", "--ignored", "--exact", "--nocapture"], ROOT, output / "capture.log",
            env=env | {"JEV_CASE_INPUT": str(config)})
    if "1 passed; 0 failed" not in (output / "capture.log").read_text():
        raise RuntimeError("case adapter did not execute")


def mutate(issue, source):
    if issue == 370:
        marker = "    pub(crate) fn reset_current_level_runtime_state_for_respawn(&mut self) {"
        if source.count(marker) != 1:
            raise ValueError("respawn boundary changed")
        begin = source.index(marker)
        end = source.index("\n    pub", begin + len(marker))
        section = source[begin:end]
        if section.count("        self.projectiles.clear();") != 1:
            raise ValueError("projectile reset changed")
        return source[:begin] + section.replace("        self.projectiles.clear();", "        // Synthetic negative control: retain pre-reload projectiles.") + source[end:]
    if issue in (210, 254):
        marker = "enter_gameplay = if title_menu_vdp_carry_active {"
        if source.count(marker) != 1:
            raise ValueError("ordinary Start carry guard absent or changed")
        return source.replace(marker, "enter_gameplay = if title_menu_vdp_carry_active && recorded_title_replay {", 1)
    raise ValueError("no verified negative-control boundary for this case")


def negative_control(output, issue, inputs, trace, assessment, env, revision):
    relative = Path("crates/wildside-core/src") / ("level.rs" if issue == 370 else "boot.rs")
    checkout = Path(tempfile.mkdtemp(prefix="wildside-jev-case-")) / "checkout"
    command(["git", "worktree", "add", "--detach", str(checkout), revision], ROOT, output / "checkout.log")
    if (output / "source.patch").stat().st_size:
        command(["git", "apply", str(output / "source.patch")], checkout, output / "snapshot.log")
    source = (checkout / relative).read_text()
    mutant = mutate(issue, source)
    patch = "".join(difflib.unified_diff(source.splitlines(True), mutant.splitlines(True), fromfile=str(relative), tofile=str(relative)))
    (output / "negative-control.patch").write_text(patch)
    (checkout / relative).write_text(mutant)
    for name in (".cache", "roms"):
        (checkout / name).symlink_to(ROOT / name, target_is_directory=True)
    build = checkout.parent / "target"
    if sys.platform == "darwin":
        source_build = Path(env.get("CARGO_TARGET_DIR", ROOT / "target")).resolve()
        command(["/bin/cp", "-cR", str(source_build), str(build)], ROOT, output / "clone-build.log")
        # Workspace build-wrap executables retain their original absolute paths.
        # Cargo rebuilds them inside this isolated checkout with sandboxing intact.
        command(["cargo", "clean", "-p", "wildside-cli", "-p", "ymfm-sys", "--target-dir", str(build)],
                checkout, output / "rebuild-sandbox-wrappers.log", env=env)
    config = output / "negative-input.json"
    write_json(config, {"issue": issue, "output": str(output / "negative-bundle"),
                       "script": inputs, "checkpoints": [0, len(inputs) - 1],
                       "start_level": CASE_STARTS[issue]})
    command(["cargo", "test", "-q", "-p", "wildside-cli", "recording::jev_cases::case_bundle",
             "--", "--ignored", "--exact", "--nocapture"], checkout, output / "negative-capture.log",
            env=env | {"JEV_CASE_INPUT": str(config), "CARGO_TARGET_DIR": str(build)})
    if "1 passed; 0 failed" not in (output / "negative-capture.log").read_text():
        raise RuntimeError("negative-control adapter did not execute")
    broken = read_json(output / "negative-bundle/trace.json")
    observed = encounter(issue, broken)
    # The #210/#254 control restores the ordinary-runtime Start carry defect.
    # Both traces stay identical through frame 2705; frame 2706 is the first
    # causal divergence, when the mutant releases gameplay instead of entering
    # the title-card worker shell. Prefix verification ends at that boundary;
    # later equality would include the defect that the control must expose.
    prefix = assessment["fatal_frame"] + 1 if issue == 370 else 2706
    prefix_equal = trace[:prefix] == broken[:prefix]
    return {"kind": "synthetic negative control" if issue == 370 else "restored ordinary-runtime Start guard defect",
            "verified": prefix_equal and source != mutant, "same_check_failed": observed["complete"] and not observed["symptom_pass"],
            "prefix_equal_frames": prefix if prefix_equal else 0, "encounter": observed,
            "checkout": str(checkout), "patch_sha256": sha256(output / "negative-control.patch"),
            "trace_sha256": sha256(output / "negative-bundle/trace.json")}


def run(output, spec, inputs, operation="check", checkpoints=()):
    issue = spec.issue
    live = operation == "discover"
    write_json(output / "run-spec.json", asdict(spec))
    env = os.environ.copy()
    revision = subprocess.check_output(["git", "rev-parse", "HEAD"], cwd=ROOT, text=True).strip()
    (output / "source.patch").write_bytes(subprocess.check_output(["git", "diff", revision, "--binary"], cwd=ROOT))
    # The runner and compiled process outlive temporary hook stashes and commits.
    # The negative control must reconstruct this same source, not a later HEAD.
    source_provenance = {"revision": revision,
                         "source_patch_sha256": sha256(output / "source.patch"),
                         "runner_sha256": sha256(Path(__file__))}
    write_json(output / "source.json", source_provenance)
    if live:
        assert_source_current(ROOT, source_provenance)
        write_json(output / "prefix.json", {"warp_to_level": spec.start_level,
                   "settle_oracle_warp": spec.start_level is not None, "script": inputs,
                   "frame_count": len(inputs), "checkpoints": [{"frame_index": f, "label": f"prefix-{f}"} for f in checkpoints]})
        if inputs:
            resolved = asdict(spec) | {"prefix": "prefix.json"}
            resolved.pop("start_level")
            write_json(output / "run-spec.json", resolved)
        source_provenance["prefix_sha256"] = sha256(output / "prefix.json")
        preflight = output / "preflight"
        preflight.mkdir()
        offline_env = env.copy()
        offline_env.pop("TYPESAFE_API_KEY", None)
        preflight_inputs = inputs or [0] * 6
        expected_frames = preflight_frames(len(preflight_inputs), checkpoints)
        capture(preflight, spec, preflight_inputs, expected_frames, offline_env)
        command(["cargo", "run", "-q", "-p", "wildside-cli", "--", "verify-recording", "--bundle",
                 str(preflight / "bundle")], ROOT, preflight / "verify.log", env=offline_env, allowed=(0, 1))
        check = read_json(preflight / "bundle/verify/report.json")
        if not comparable(check, expected_frames):
            raise RuntimeError("live discovery blocked by starting-state oracle comparison; see preflight/verify.log")
        source_provenance["preflight_frames"] = expected_frames
        source_provenance["rom_sha1"] = read_json(preflight / "bundle/manifest.json")["rom_sha1"]
        assert_source_current(ROOT, source_provenance)
        env["JEV_SOURCE_IDENTITY"] = str(output / "source.json")
        env["JEV_EXPECTED_ROM_SHA1"] = source_provenance["rom_sha1"]
        env["JEV_CASE_TOKEN"] = jev_case_service.begin_attempt(issue, output)
    else:
        env.pop("TYPESAFE_API_KEY", None)
    checkpoints = sorted({0, *checkpoints}) if live else sorted({0, len(inputs) - 1, *checkpoints})
    capture(output, spec, inputs, checkpoints, env, live)
    env.pop("TYPESAFE_API_KEY", None)
    if live:
        assert_source_current(ROOT, source_provenance)
    trace = read_json(output / "bundle/trace.json")
    collection = read_json(output / "collection.json") if live else None
    assessment = encounter(issue, trace) if operation == "check" else None
    begin, end = assessment["window"] if assessment else (max(0, len(trace) - 60), len(trace) - 1)
    if collection and collection.get("target_witness_frame") is not None:
        dwell = spec.target.get("dwell_frames", 1) if spec.target else 1
        begin = max(0, collection["target_witness_frame"] - dwell + 1 - 30)
    # Preserve the whole event window, including prefix checkpoints, not just favourable frames.
    frames = sorted({*checkpoints, *range(begin, end + 1)})
    manifest = read_json(output / "bundle/manifest.json")
    if live and manifest["rom_sha1"] != source_provenance["rom_sha1"]:
        raise ValueError("ROM changed after preflight")
    manifest["scenario"]["checkpoints"] = [{"frame_index": frame, "label": f"case-{frame}"} for frame in frames]
    write_json(output / "bundle/manifest.json", manifest)
    write_json(output / "bundle/scenario.json", manifest["scenario"])
    command(["cargo", "run", "-q", "-p", "wildside-cli", "--", "verify-recording",
             "--bundle", str(output / "bundle")], ROOT, output / "verify.log", env=env, allowed=(0, 1))
    comparison = read_json(output / "bundle/verify/report.json")
    sensitivity = negative_control(output, issue, manifest["scenario"]["script"], trace, assessment, env, revision) if (
        assessment and issue in (210, 254, 370) and assessment["complete"] and assessment["symptom_pass"]) else None
    proof = verdict(assessment, comparison, sensitivity)
    if assessment and assessment["complete"] and not comparable(comparison, frames):
        proof = "comparison inconclusive"
    result = proof if operation == "check" else "replay matched" if comparable(comparison, frames) else "comparison inconclusive"
    if live:
        if collection["stop_reason"] == "request_failed_or_limit":
            result = "collection interrupted"
        elif collection["suffix_frames"] == 0:
            result = "no inputs collected"
        else:
            result = "route collected" if comparable(comparison, frames) else "comparison inconclusive"
    return {"issue": issue, "operation": operation, "verdict": result, "proof": proof,
            "collection": collection,
            "encounter": assessment,
            "comparison": comparison, "negative_control": sensitivity,
            "provenance": {**source_provenance,
                "scenario_sha256": sha256(output / "bundle/scenario.json"),
                "rom_sha1": manifest["rom_sha1"], "script_sha1": manifest["script_sha1"],
                "trace_sha1": manifest["trace_sha1"], "live_discovery": live,
                "budget": read_json(jev_case_service.LEDGER) if live else None}}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    commands = parser.add_subparsers(dest="operation", required=True)
    discover = commands.add_parser("discover", help="explicitly permit paid route collection")
    discover.add_argument("--spec", type=Path, required=True)
    replay = commands.add_parser("replay", help="compare an ordinary-input scenario without paid calls")
    replay.add_argument("--scenario", type=Path, required=True)
    check = commands.add_parser("check", help="apply a registered issue assertion and negative control")
    check.add_argument("--issue", type=int, required=True)
    check.add_argument("--scenario", type=Path)
    args = parser.parse_args()
    checkpoints = []
    try:
        if args.operation == "discover":
            spec = RunSpec.parse(read_json(args.spec))
            entries = {row["issue"]: row for row in audit()}
            if spec.issue not in entries:
                raise ValueError("discovery issue must be in the unresolved catalogue")
            if entries[spec.issue]["readiness"] in ("inactive_path", "human_device"):
                raise ValueError("establish an active objective path before discovery")
            if not spec.goal.strip():
                raise ValueError("discovery needs a goal")
            if not os.environ.get("TYPESAFE_API_KEY"):
                raise ValueError("TYPESAFE_API_KEY is required for discovery")
            inputs = []
            if spec.prefix:
                level, inputs, checkpoints = load_scenario(read_json(args.spec.parent / spec.prefix))
                spec = replace(spec, start_level=level)
        elif args.scenario:
            level, inputs, checkpoints = load_scenario(read_json(args.scenario))
            spec = RunSpec(args.issue if args.operation == "check" else 0, level)
            if args.operation == "check" and spec.issue in CASE_STARTS and level != CASE_STARTS[spec.issue]:
                raise ValueError("scenario start does not match the registered proof case")
        else:
            if args.issue not in CASE_STARTS or route(args.issue) is None:
                raise ValueError("this case needs --scenario")
            spec = RunSpec(args.issue, CASE_STARTS[args.issue])
            inputs = route(args.issue)
    except (ValueError, OSError) as error:
        parser.error(str(error))
    output = ROOT / ".cache/jev-cases" / str(time.time_ns())
    output.mkdir(parents=True)
    print(f"Case evidence: {output}", flush=True)
    try:
        report = run(output, spec, inputs, args.operation, checkpoints)
    except (ValueError, OSError, RuntimeError, subprocess.SubprocessError) as error:
        report = {"issue": spec.issue, "verdict": "execution failed", "proof": "not evaluated", "error": str(error)}
        if (output / "collection.json").exists():
            report["collection"] = read_json(output / "collection.json")
    write_json(output / "report.json", report)
    render_report(output, report)
    print(report["verdict"], flush=True)
    return 0 if report["verdict"] in ("proven check", "replay matched", "route collected") else 1


if __name__ == "__main__":
    raise SystemExit(main())
