"""Isolated #372 negative control for the saved Jev route. No model calls."""

import base64
import difflib
import hashlib
import html
import json
import os
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile
import time

ROOT = Path(__file__).resolve().parents[1]
SCENE = Path("crates/wildside-frame/src/scene.rs")
REGRESSION = "prize_block_wobble_sprite_stays_over_low_priority_terrain_sub_1007a_issue372"
ROUTE = Path("tests/data/dynamic-scenarios/issue-372-jev-prize-recovery.json")


def verdict(evidence):
    if not evidence["mutation_verified"]:
        return "execution failed"
    if not all(evidence[key] for key in (
        "encounter_complete", "trace_equal", "state_equal", "captures_complete"
    )) or evidence["fixed_changed_pixels"]:
        return "comparison inconclusive"
    if not evidence["broken_changed_pixels"]:
        return "case did not expose the bug"
    if not evidence["target_changed_pixels"]:
        return "comparison inconclusive"
    return "proof passed"


def restore_defect(source):
    # ASM sub_1007A: restore the pre-5d83d0aa3 scene painter order only.
    begin = "    if filter.block_overrides {\n        let wobble_commands ="
    end = "\n    if !eol_hologram_visible && filter.effects {"
    insertion = "    layer_tags.mark(SceneLayer::Background, commands.len());\n"
    if any(source.count(marker) != 1 for marker in (begin, end, insertion)):
        raise ValueError("historical defect anchors changed; inspect renderer before adapting")
    start = source.index(begin)
    finish = source.index(end, start)
    block = source[start:finish]
    if "push_prize_block_wobble_overrides_sub_1007a(" not in block:
        raise ValueError("prize draw missing from historical defect span")
    shortened = source[:start] + source[finish:]
    return shortened.replace(insertion, insertion + block + "\n", 1)


def read_json(path):
    return json.loads(path.read_text())


def write_json(path, value):
    path.write_text(json.dumps(value, indent=2) + "\n")


def sha256(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def command(args, cwd, log, *, env=None, allowed=(0,)):
    with log.open("wb") as output:
        result = subprocess.run(args, cwd=cwd, env=env, stdout=output,
                                stderr=subprocess.STDOUT, timeout=1800)
    if result.returncode not in allowed:
        raise RuntimeError(f"command failed ({result.returncode}); see {log}")
    return result.returncode


def rgb(path):
    data = subprocess.check_output(["magick", str(path), "-alpha", "off", "-depth", "8", "rgb:-"])
    if len(data) != 320 * 224 * 3:
        raise ValueError(f"unexpected frame dimensions: {path}")
    return data


def pixel_counts(left, right, rectangle):
    full = target = 0
    x0, y0, width, height = rectangle
    for offset in range(0, len(left), 3):
        if left[offset:offset + 3] != right[offset:offset + 3]:
            full += 1
            pixel = offset // 3
            x, y = pixel % 320, pixel // 320
            target += x0 <= x < x0 + width and y0 <= y < y0 + height
    return full, target


def inspect_comparison(output, assessment):
    fixed, broken = output / "fixed", output / "broken"
    reports = [read_json(path / "verify/report.json") for path in (fixed, broken)]
    traces = [read_json(path / "verify/native/trace.json") for path in (fixed, broken)]
    scenario = read_json(fixed / "scenario.json")
    first, last = assessment["first_hit_frame"], assessment["last_animation_frame"]
    frames = [c["frame_index"] for c in scenario["checkpoints"]]
    complete = (first is not None and last is not None and
                frames == list(range(max(0, first - 30), last + 31)))
    rows = []
    for checkpoint in scenario["checkpoints"]:
        frame, label = checkpoint["frame_index"], checkpoint["label"]
        images = {
            "fixed": fixed / "verify/native" / f"{frame:06}_{label}.png",
            "broken": broken / "verify/native" / f"{frame:06}_{label}.png",
            "oracle": fixed / "verify/oracle" / f"{frame:04}_{label}.png",
        }
        if not all(path.is_file() for path in images.values()):
            complete = False
            continue
        trace = traces[0][frame]
        # ASM sub_1007A: include the 16px block and its complete vertical wobble.
        col, row = assessment["target_cell"]
        rect = (col * 16 - (trace["camera_x_pos"] >> 16),
                row * 16 - 16 - (trace["camera_y_pos"] >> 16), 16, 36)
        pixels = {name: rgb(path) for name, path in images.items()}
        fixed_count, _ = pixel_counts(pixels["fixed"], pixels["oracle"], rect)
        broken_count, target_count = pixel_counts(pixels["broken"], pixels["oracle"], rect)
        contrast_count, _ = pixel_counts(pixels["broken"], pixels["fixed"], rect)
        rows.append({"frame": frame, "fixed_changed_pixels": fixed_count,
                     "broken_changed_pixels": broken_count,
                     "target_changed_pixels": target_count if first is not None and last is not None and first <= frame <= last else 0,
                     "broken_vs_fixed_pixels": contrast_count,
                     "images": {name: str(path.relative_to(output)) for name, path in images.items()}})
    evidence = {
        "mutation_verified": True,
        "encounter_complete": assessment["complete_window"] and complete,
        "trace_equal": traces[0] == traces[1] and all(r["trace_match"] for r in reports),
        "state_equal": all(r["oracle_state_match"] for r in reports),
        "captures_complete": complete and all(
            [d["frame_index"] for d in report["frame_diffs"]] == frames for report in reports),
        **{key: sum(row[key] for row in rows) for key in (
            "fixed_changed_pixels", "broken_changed_pixels", "target_changed_pixels")},
    }
    return evidence, rows


def render_report(output, report):
    payload = []
    for row in report.get("frames", []):
        images = {name: "data:image/png;base64," + base64.b64encode(
            (output / path).read_bytes()).decode() for name, path in row["images"].items()}
        payload.append({**row, "images": images})
    summary = {key: value for key, value in report.items() if key != "frames"}
    document = '''<!doctype html><html lang="en"><meta charset="utf-8">
<title>Jev prize-block proof</title><style>
body{font:16px system-ui;max-width:1200px;margin:2rem auto;padding:0 1rem;background:#131920;color:#eef3f8}
.grid{display:flex;gap:12px;flex-wrap:wrap}figure{margin:0}img{width:320px;image-rendering:pixelated}
pre{white-space:pre-wrap;overflow-wrap:anywhere;background:#202b36;padding:1rem}input{width:100%}
</style><h1>Jev prize-block proof</h1><h2>VERDICT</h2><p>NOTE</p>
<p>Identical saved Jev inputs. The restored defect is isolated from the working checkout.</p>
<label>Checkpoint <input id="frame" type="range" min="0" value="0"></label><p id="details"></p>
<div class="grid"><figure><figcaption>Broken build</figcaption><img id="broken" alt="Broken build frame"></figure>
<figure><figcaption>Fixed build</figcaption><img id="fixed" alt="Fixed build frame"></figure>
<figure><figcaption>Emulator</figcaption><img id="oracle" alt="Emulator frame"></figure></div>
<h2>Evidence</h2><pre>SUMMARY</pre><script>
const rows=PAYLOAD, slider=document.getElementById('frame');
slider.max=Math.max(0,rows.length-1);slider.disabled=!rows.length;
function show(){if(!rows.length)return;const r=rows[Number(slider.value)];
for(const name of ['broken','fixed','oracle'])document.getElementById(name).src=r.images[name];
document.getElementById('details').textContent=`Frame ${r.frame}: broken ${r.broken_changed_pixels}, fixed ${r.fixed_changed_pixels}, target ${r.target_changed_pixels} changed pixels`;}
slider.addEventListener('input',show);show();</script></html>'''
    document = document.replace("VERDICT", html.escape(report["verdict"]))
    document = document.replace("NOTE", html.escape(report.get("note", "")))
    document = document.replace("SUMMARY", html.escape(json.dumps(summary, indent=2)))
    document = document.replace("PAYLOAD", json.dumps(payload).replace("<", "\\u003c"))
    (output / "report.html").write_text(document)


def run(output):
    if not shutil.which("magick"):
        raise RuntimeError("ImageMagick (magick) is required for pixel comparison")
    env = os.environ.copy()
    env.pop("TYPESAFE_API_KEY", None)
    env["CARGO_TARGET_DIR"] = str(ROOT / "target")
    source = (ROOT / SCENE).read_text()
    mutant = restore_defect(source)
    patch = "".join(difflib.unified_diff(source.splitlines(True), mutant.splitlines(True),
                                       fromfile=str(SCENE), tofile=str(SCENE)))
    (output / "defect.patch").write_text(patch)
    source_delta = subprocess.check_output(["git", "diff", "HEAD", "--binary"], cwd=ROOT)
    (output / "source.patch").write_bytes(source_delta)
    metadata = {
        "revision": subprocess.check_output(["git", "rev-parse", "HEAD"], cwd=ROOT, text=True).strip(),
        "source_patch_sha256": sha256(output / "source.patch"),
        "defect_patch_sha256": sha256(output / "defect.patch"),
        "historical_fix": "5d83d0aa370113ce8ea65c44a4e4b07b976d64f5",
        "scenario_sha256": sha256(ROOT / ROUTE),
        "runner_sha256": sha256(Path(__file__)),
        "api_requests": 0, "api_cost": 0,
    }
    write_json(output / "provenance.json", metadata)
    print("Checking fixed renderer regression", flush=True)
    command(["cargo", "test", "-q", "-p", "wildside-frame", REGRESSION, "--", "--nocapture"],
            ROOT, output / "fixed-regression.log", env=env)
    if "1 passed; 0 failed" not in (output / "fixed-regression.log").read_text():
        raise RuntimeError("fixed renderer regression was not executed")
    bundle_env = env | {"JEV_PROOF_OUTPUT": str(output / "fixed")}
    command(["cargo", "test", "-q", "-p", "wildside-cli", "recording::jev_playtest::proof_bundle",
             "--", "--ignored", "--exact", "--nocapture"], ROOT, output / "bundle.log", env=bundle_env)
    print("Comparing fixed recording with emulator", flush=True)
    command(["cargo", "run", "-q", "-p", "wildside-cli", "--", "verify-recording", "--bundle", str(output / "fixed")],
            ROOT, output / "fixed-verify.log", env=env, allowed=(0, 1))
    fixed_manifest = read_json(output / "fixed/manifest.json")
    metadata.update({key: fixed_manifest[key] for key in ("rom_sha1", "script_sha1", "trace_sha1")})
    write_json(output / "provenance.json", metadata)
    checkout = Path(tempfile.mkdtemp(prefix="wildside-jev-proof-")) / "checkout"
    command(["git", "worktree", "add", "--detach", str(checkout), metadata["revision"]],
            ROOT, output / "checkout.log")
    metadata["isolated_checkout"] = str(checkout)
    write_json(output / "provenance.json", metadata)
    if source_delta:
        command(["git", "apply", str(output / "source.patch")], checkout, output / "snapshot.log")
    (checkout / SCENE).write_text(mutant)
    for name in (".cache", "roms"):
        (checkout / name).symlink_to(ROOT / name, target_is_directory=True)
    build = checkout.parent / "target"
    # APFS clones reuse dependency artefacts without sharing writable build files.
    if sys.platform == "darwin":
        command(["/bin/cp", "-cR", str(ROOT / "target"), str(build)], ROOT, output / "clone-build.log")
    broken_env = env | {"CARGO_TARGET_DIR": str(build)}
    print("Checking restored historical defect in isolated checkout", flush=True)
    status = command(["cargo", "test", "-q", "-p", "wildside-frame", REGRESSION, "--", "--nocapture"],
                     checkout, output / "broken-regression.log", env=broken_env, allowed=(0, 101))
    log = (output / "broken-regression.log").read_text()
    if status != 101 or f"{REGRESSION} --- FAILED" not in log or "1 failed" not in log:
        raise RuntimeError("restored defect did not fail the expected renderer assertion")
    shutil.copytree(output / "fixed", output / "broken", ignore=shutil.ignore_patterns("verify", "native"))
    print("Comparing broken recording with emulator", flush=True)
    command(["cargo", "run", "-q", "-p", "wildside-cli", "--", "verify-recording", "--bundle", str(output / "broken")],
            checkout, output / "broken-verify.log", env=broken_env, allowed=(0, 1))
    assessment = read_json(output / "fixed/assessment.json")
    evidence, rows = inspect_comparison(output, assessment)
    return {"verdict": verdict(evidence), "evidence": evidence, "encounter": assessment,
            "provenance": metadata, "frames": rows,
            "jev_decisions": read_json(ROOT / "docs/source-port/jev-successful-play-evidence.json"),
            "note": "The restored historical defect belongs to the scene renderer. Current gameplay recordings use the VDP renderer. No paid attempts are made to overcome an inactive rendering path."}


def main():
    output = ROOT / ".cache/jev-proof" / str(time.time_ns())
    output.mkdir(parents=True)
    print(f"Jev proof evidence: {output}", flush=True)
    try:
        report = run(output)
    except (OSError, ValueError, KeyError, TypeError, RuntimeError, subprocess.SubprocessError) as error:
        report = {"verdict": "execution failed", "note": str(error), "frames": []}
    write_json(output / "report.json", report)
    render_report(output, report)
    print(f"{report['verdict']}: {output / 'report.html'}", flush=True)
    return 0 if report["verdict"] == "proof passed" else 1


if __name__ == "__main__":
    sys.exit(main())
