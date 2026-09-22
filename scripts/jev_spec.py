"""Private JEV inputs. Scenarios retain only ordinary controls and lawful starts."""

from dataclasses import dataclass
import hashlib
import subprocess


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

    @classmethod
    def parse(cls, value):
        if not isinstance(value, dict) or value.keys() - {"issue", "start_level", "goal", "prefix", "max_decisions"}:
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
        return cls(issue, level, goal, prefix, decisions)


def assert_source_current(root, identity):
    revision = subprocess.check_output(["git", "rev-parse", "HEAD"], cwd=root, text=True).strip()
    patch = subprocess.check_output(["git", "diff", revision, "--binary"], cwd=root)
    untracked = subprocess.check_output(["git", "ls-files", "--others", "--exclude-standard", "scripts", "crates"], cwd=root)
    if (revision != identity["revision"] or hashlib.sha256(patch).hexdigest() != identity["source_patch_sha256"]
            or untracked.strip()):
        raise ValueError("source changed or contains untracked code; repeat preflight")
