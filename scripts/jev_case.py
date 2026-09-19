"""Run one private bug case through the existing recording/oracle pipeline."""

import argparse
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
                "window": [max(0, (fatal or 30) - 30), min(len(trace) - 1, (live_frame or len(trace) - 31) + 30)],
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
    raise ValueError("unsupported case")


def verdict(assessment, comparison, sensitivity):
    if not assessment["complete"]:
        return "not exercised"
    if not comparison or comparison.get("native_replay_only") or not all(comparison.get(key) for key in
            ("trace_match", "oracle_state_match", "oracle_frame_match")) or not comparison.get("frame_diffs"):
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


def capture(output, issue, inputs, checkpoints, env, live=False):
    config = output / "input.json"
    write_json(config, {"issue": issue, "output": str(output / "bundle"), "script": inputs,
                        "checkpoints": checkpoints, "live": live})
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


def negative_control(output, issue, inputs, trace, assessment, env):
    relative = Path("crates/wildside-core/src") / ("level.rs" if issue == 370 else "boot.rs")
    source = (ROOT / relative).read_text()
    mutant = mutate(issue, source)
    patch = "".join(difflib.unified_diff(source.splitlines(True), mutant.splitlines(True), fromfile=str(relative), tofile=str(relative)))
    (output / "negative-control.patch").write_text(patch)
    checkout = Path(tempfile.mkdtemp(prefix="wildside-jev-case-")) / "checkout"
    command(["git", "worktree", "add", "--detach", str(checkout), "HEAD"], ROOT, output / "checkout.log")
    delta = subprocess.check_output(["git", "diff", "HEAD", "--binary"], cwd=ROOT)
    (output / "source.patch").write_bytes(delta)
    if delta:
        command(["git", "apply", str(output / "source.patch")], checkout, output / "snapshot.log")
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
                       "script": inputs, "checkpoints": [0, len(inputs) - 1]})
    command(["cargo", "test", "-q", "-p", "wildside-cli", "recording::jev_cases::case_bundle",
             "--", "--ignored", "--exact", "--nocapture"], checkout, output / "negative-capture.log",
            env=env | {"JEV_CASE_INPUT": str(config), "CARGO_TARGET_DIR": str(build)})
    if "1 passed; 0 failed" not in (output / "negative-capture.log").read_text():
        raise RuntimeError("negative-control adapter did not execute")
    broken = read_json(output / "negative-bundle/trace.json")
    observed = encounter(issue, broken)
    prefix = assessment["fatal_frame"] + 1 if issue == 370 else 2718
    prefix_equal = trace[:prefix] == broken[:prefix]
    return {"kind": "synthetic negative control" if issue == 370 else "restored ordinary-runtime Start guard defect",
            "verified": prefix_equal and source != mutant, "same_check_failed": observed["complete"] and not observed["symptom_pass"],
            "prefix_equal_frames": prefix if prefix_equal else 0, "encounter": observed,
            "checkout": str(checkout), "patch_sha256": sha256(output / "negative-control.patch"),
            "trace_sha256": sha256(output / "negative-bundle/trace.json")}


def run(output, issue, inputs, live=False):
    env = os.environ.copy()
    (output / "source.patch").write_bytes(subprocess.check_output(["git", "diff", "HEAD", "--binary"], cwd=ROOT))
    if live:
        preflight = output / "preflight"
        preflight.mkdir()
        offline_env = env.copy()
        offline_env.pop("TYPESAFE_API_KEY", None)
        capture(preflight, issue, [0] * 6, [0, 5], offline_env)
        command(["cargo", "run", "-q", "-p", "wildside-cli", "--", "verify-recording", "--bundle",
                 str(preflight / "bundle")], ROOT, preflight / "verify.log", env=offline_env, allowed=(0, 1))
        check = read_json(preflight / "bundle/verify/report.json")
        if check.get("native_replay_only") or not all(check.get(key) for key in
                ("trace_match", "oracle_state_match", "oracle_frame_match")):
            raise RuntimeError("live discovery blocked by starting-state oracle comparison; see preflight/verify.log")
        env["JEV_CASE_TOKEN"] = jev_case_service.begin_attempt(issue, output)
    else:
        env.pop("TYPESAFE_API_KEY", None)
    checkpoints = [0] if live else [0, len(inputs) - 1]
    capture(output, issue, inputs, checkpoints, env, live)
    env.pop("TYPESAFE_API_KEY", None)
    trace = read_json(output / "bundle/trace.json")
    assessment = encounter(issue, trace)
    begin, end = assessment["window"]
    # Preserve the whole event window, including prefix checkpoints, not just favourable frames.
    frames = sorted({0, *range(begin, end + 1)})
    manifest = read_json(output / "bundle/manifest.json")
    manifest["scenario"]["checkpoints"] = [{"frame_index": frame, "label": f"case-{frame}"} for frame in frames]
    write_json(output / "bundle/manifest.json", manifest)
    write_json(output / "bundle/scenario.json", manifest["scenario"])
    command(["cargo", "run", "-q", "-p", "wildside-cli", "--", "verify-recording",
             "--bundle", str(output / "bundle")], ROOT, output / "verify.log", env=env, allowed=(0, 1))
    comparison = read_json(output / "bundle/verify/report.json")
    sensitivity = negative_control(output, issue, manifest["scenario"]["script"], trace, assessment, env) if (
        issue in (210, 254, 370) and assessment["complete"] and assessment["symptom_pass"]) else None
    return {"issue": issue, "verdict": verdict(assessment, comparison, sensitivity), "encounter": assessment,
            "comparison": comparison, "negative_control": sensitivity,
            "provenance": {"revision": subprocess.check_output(["git", "rev-parse", "HEAD"], cwd=ROOT, text=True).strip(),
                "scenario_sha256": sha256(output / "bundle/scenario.json"),
                "source_patch_sha256": sha256(output / "source.patch"), "runner_sha256": sha256(Path(__file__)),
                "rom_sha1": manifest["rom_sha1"], "script_sha1": manifest["script_sha1"],
                "trace_sha1": manifest["trace_sha1"], "live_discovery": live,
                "budget": read_json(jev_case_service.LEDGER) if live else None}}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--issue", type=int, required=True)
    parser.add_argument("--script", type=Path, help="private JSON array of ordinary controller masks")
    parser.add_argument("--live", action="store_true", help="explicitly permit bounded paid discovery for #327")
    args = parser.parse_args()
    entries = {row["issue"]: row for row in audit()}
    if args.issue not in entries:
        parser.error("issue is not in the unresolved catalogue")
    if args.live and (args.issue != 327 or args.script):
        parser.error("live discovery starts naturally at BLW2; no unverified prefix is accepted")
    if args.live and not os.environ.get("TYPESAFE_API_KEY"):
        parser.error("TYPESAFE_API_KEY is required for --live")
    inputs = [] if args.live else read_json(args.script) if args.script else route(args.issue)
    if inputs is None:
        parser.exit(1, entries[args.issue]["next_action"] + "\n")
    if not isinstance(inputs, list) or not (0 if args.live else 1) <= len(inputs) <= 3600 or any(type(x) is not int or not 0 <= x <= 255 for x in inputs):
        parser.error("script must contain 1..3600 byte masks")
    output = ROOT / ".cache/jev-cases" / str(time.time_ns())
    output.mkdir(parents=True)
    print(f"Case evidence: {output}", flush=True)
    try:
        report = run(output, args.issue, inputs, args.live)
    except (ValueError, OSError, RuntimeError, subprocess.SubprocessError) as error:
        report = {"issue": args.issue, "verdict": "execution failed", "error": str(error)}
    write_json(output / "report.json", report)
    render_report(output, report)
    print(report["verdict"], flush=True)
    return 0 if report["verdict"] == "proven check" else 1


if __name__ == "__main__":
    raise SystemExit(main())
