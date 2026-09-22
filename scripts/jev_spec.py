"""Private JEV inputs. Scenarios retain only ordinary controls and lawful starts."""

from dataclasses import dataclass
import hashlib
import subprocess


# ASM ReadJoypad / constants.asm: controller bits are SACBRLDU.
ACTION_MASKS = {"release": 0, "left": 4, "right": 8, "jump": 16, "left_jump": 20,
                "right_jump": 24, "run_right": 72, "run_right_jump": 88,
                "run_left": 68, "run_left_jump": 84, "start": 128,
                "up": 1, "down": 2, "a": 64, "special": 32,
                "left_special": 36, "right_special": 40, "jump_special": 48}


def discovery_target(value):
    if value is None:
        return None
    if not isinstance(value, dict):
        raise ValueError("target must be an object")
    kind = value.get("kind")
    if kind in ("blw2_teleport", "dragon_breath_death", "object_pressure_candidate"):
        if value.keys() != {"kind"}:
            raise ValueError("registered target takes only kind")
    elif kind == "position":
        if value.keys() != {"kind", "level", "x", "y", "radius", "dwell_frames"}:
            raise ValueError("position needs level, x, y, radius and dwell_frames")
        integer(value["level"], 0, 255, "target level")
        for coordinate in ("x", "y"):
            integer(value[coordinate], -32768, 32767, coordinate)
        integer(value["radius"], 0, 65535, "radius")
        integer(value["dwell_frames"], 1, 3600, "dwell_frames")
    else:
        raise ValueError("unknown discovery target")
    return value.copy()


def integer(value, low, high, name):
    if type(value) is not int or not low <= value <= high:
        raise ValueError(f"{name} must be an integer in {low}..{high}")
    return value


def load_scenario(value):
    fields = {"slug", "script", "frame_count", "checkpoints", "warp_to_level", "settle_oracle_warp",
              "start_position", "start_velocity", "camera_position", "playtest_form",
              "initial_lives", "initial_diamond_count"}
    if not isinstance(value, dict) or value.keys() - fields:
        raise ValueError("unknown scenario fields")
    for key in ("start_position", "start_velocity", "camera_position", "playtest_form",
                "initial_lives", "initial_diamond_count"):
        if value.get(key) is not None:
            raise ValueError(f"ordinary-input scenarios cannot inject {key}")
    script = value.get("script")
    if not isinstance(script, list) or not script:
        raise ValueError("scenario needs ordinary input bytes")
    for mask in script:
        integer(mask, 0, 255, "controller mask")
    if type(value.get("frame_count", len(script))) is not int or value.get("frame_count", len(script)) != len(script):
        raise ValueError("scenario frame_count must equal script length")
    level = value.get("warp_to_level")
    if level is not None:
        integer(level, 0, 255, "warp_to_level")
    if value.get("settle_oracle_warp", False) is not (level is not None):
        raise ValueError("start must be ordinary boot or a settled diagnostic warp")
    checkpoints = value.get("checkpoints", [])
    if not isinstance(checkpoints, list):
        raise ValueError("checkpoints must be an array")
    frames = []
    for checkpoint in checkpoints:
        if not isinstance(checkpoint, dict) or checkpoint.keys() - {"frame_index", "label"}:
            raise ValueError("invalid checkpoint")
        frames.append(integer(checkpoint.get("frame_index"), 0, len(script) - 1, "checkpoint"))
    return level, script, frames


@dataclass(frozen=True)
class RunSpec:
    issue: int
    start_level: int | None = None
    goal: str = ""
    prefix: str | None = None
    max_decisions: int = 600
    actions: tuple[str, ...] = tuple(ACTION_MASKS)
    progress: str = "cells"
    target: dict | None = None
    tail_frames: int = 30
    stop_on_death: bool = True

    @classmethod
    def parse(cls, value):
        if not isinstance(value, dict) or value.keys() - {"issue", "start_level", "goal", "prefix", "max_decisions", "actions", "progress", "target", "tail_frames", "stop_on_death"}:
            raise ValueError("unknown run specification fields")
        issue = integer(value.get("issue"), 1, 65535, "issue")
        level = value.get("start_level")
        if level is not None:
            integer(level, 0, 255, "start_level")
        goal = value.get("goal", "")
        if not isinstance(goal, str):
            raise ValueError("goal must be text")
        prefix = value.get("prefix")
        if prefix is not None and (not isinstance(prefix, str) or not prefix.strip()):
            raise ValueError("prefix must name a scenario file")
        if prefix is not None and "start_level" in value:
            raise ValueError("prefix supplies its own start; omit start_level")
        decisions = integer(value.get("max_decisions", 600), 1, 600, "max_decisions")
        actions = value.get("actions", list(ACTION_MASKS))
        if (not isinstance(actions, list) or not actions or any(not isinstance(a, str) or a not in ACTION_MASKS for a in actions)
                or len(set(actions)) != len(actions) or "release" not in actions):
            raise ValueError("actions must be unique known controls including release")
        progress = value.get("progress", "cells")
        if progress not in ("cells", "rightward"):
            raise ValueError("progress must be cells or rightward")
        target = discovery_target(value.get("target"))
        tail = integer(value.get("tail_frames", 30), 0, 3600, "tail_frames")
        death = value.get("stop_on_death", True)
        if type(death) is not bool:
            raise ValueError("stop_on_death must be boolean")
        return cls(issue, level, goal, prefix, decisions, tuple(actions), progress, target, tail, death)


def assert_source_current(root, identity):
    revision = subprocess.check_output(["git", "rev-parse", "HEAD"], cwd=root, text=True).strip()
    patch = subprocess.check_output(["git", "diff", revision, "--binary"], cwd=root)
    untracked = subprocess.check_output(["git", "ls-files", "--others", "--exclude-standard", "scripts", "crates"], cwd=root)
    if (revision != identity["revision"] or hashlib.sha256(patch).hexdigest() != identity["source_patch_sha256"]
            or untracked.strip()):
        raise ValueError("source changed or contains untracked code; repeat preflight")
