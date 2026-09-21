"""Private paid controller with a persistent, locked cumulative spend ledger."""

from contextlib import contextmanager
import fcntl
import json
import os
from pathlib import Path
import signal
import sys
import tempfile
import time
import urllib.request
import uuid

from jev_request import ENDPOINT, MODEL, deadline

LEDGER = Path(__file__).resolve().parents[1] / ".cache/jev-cases/budget.json"
PRIOR = 24_954_048
LIMIT = 1_000_000_000
CAMPAIGN_ID = "single-player-completion-2026-09-20"
CAMPAIGN_ALLOWANCE = 5_000_000_000
RESERVATION = 64_000 * 42
MASKS = {"release": 0, "left": 4, "right": 8, "jump": 16, "left_jump": 20,
         "right_jump": 24, "run_right": 72, "run_right_jump": 88,
         "run_left": 68, "run_left_jump": 84, "start": 128}


QUESTION = {
    "type": "choice",
    "instructions": "Choose ordinary controls for six game frames to achieve the stated goal. Positions are pixels and velocities are pixels per frame, right/down positive. B jumps; release B before a new jump. A runs. Momentum persists. Use terrain to find a higher route if blocked. Use Start to leave LevelTransition title cards or resume a paused game. Do not repeat an action that leaves Kid stationary when the goal requires movement. Never change game state.",
    "criteria": {
        "release": "Release held buttons before another Start or jump, or wait still when the stated goal requires it.",
        "left": "Unpaused InGame: move left slowly to backtrack or adjust position.",
        "right": "Unpaused InGame: move right slowly for precise positioning.",
        "jump": "Unpaused InGame: jump vertically. Start a jump only when B is released; holding B sustains an existing jump.",
        "left_jump": "Unpaused InGame: jump towards a nearby ledge on the left. Release B before a new jump.",
        "right_jump": "Unpaused InGame: jump towards a nearby ledge on the right. Release B before a new jump.",
        "run_right": "Unpaused InGame only: run right on clear ground. This cannot leave a title card.",
        "run_right_jump": "Unpaused InGame only: run and jump right to clear a gap or enemy. Release B before a new jump.",
        "run_left": "Unpaused InGame only: run left to backtrack or gain a run-up.",
        "run_left_jump": "Unpaused InGame only: run and jump left to reach a higher route. Release B before a new jump.",
        "start": "Press Start when mode is LevelTransition and HP is positive to leave the title card, or when paused is true to resume. If Start is already held, release it first."
    }
}


def question_for_state(state):
    question = QUESTION.copy()
    question["criteria"] = QUESTION["criteria"].copy()
    if not state.get("allow_wait") and not any(state.get("held_buttons", {}).values()):
        question["criteria"].pop("release")
    if state.get("mode") != "LevelTransition" and not state.get("paused"):
        question["criteria"].pop("start")
    if state.get("stalled_forward_decisions", 0) >= 3:
        question["criteria"].pop("right")
        question["criteria"].pop("run_right")
    return question


def persist(path, value):
    with tempfile.NamedTemporaryFile(mode="w", dir=path.parent, delete=False) as temporary:
        json.dump(value, temporary)
        temporary.flush()
        os.fsync(temporary.fileno())
    os.replace(temporary.name, path)


def _budget_bounds(value):
    if "campaign" not in value:
        return PRIOR, LIMIT
    campaign = value["campaign"]
    if not isinstance(campaign, dict) or campaign.get("id") != CAMPAIGN_ID:
        raise ValueError("invalid campaign allocation")
    opening = campaign.get("opening_nanodollars_upper_bound")
    if (type(opening) is not int or not PRIOR <= opening <= LIMIT
            or campaign.get("allowance_nanodollars") != CAMPAIGN_ALLOWANCE):
        raise ValueError("invalid campaign allocation")
    return opening, opening + CAMPAIGN_ALLOWANCE


@contextmanager
def ledger(path=LEDGER):
    path.parent.mkdir(parents=True, exist_ok=True)
    with path.with_suffix(".lock").open("a") as lock:
        fcntl.flock(lock, fcntl.LOCK_EX)
        if not path.exists():
            raise ValueError("budget ledger is missing; restore historical accounting")
        value = json.loads(path.read_text())
        floor, ceiling = _budget_bounds(value)
        if (type(value["nanodollars_upper_bound"]) is not int
                or not floor <= value["nanodollars_upper_bound"] <= ceiling):
            raise ValueError("invalid budget ledger")
        yield value


def begin_attempt(issue, output, path=LEDGER, now=None):
    now = time.time() if now is None else now
    with ledger(path) as value:
        value.setdefault("campaign", {
            "id": CAMPAIGN_ID,
            "opening_nanodollars_upper_bound": value["nanodollars_upper_bound"],
            "allowance_nanodollars": CAMPAIGN_ALLOWANCE})
        attempts = [a for a in value["attempts"]
                    if a["issue"] == issue and a.get("campaign_id") == CAMPAIGN_ID]
        if len(attempts) >= 3 or attempts and now >= attempts[0]["started"] + 1200:
            raise ValueError("case collection limit reached")
        token = uuid.uuid4().hex
        value["attempts"].append({"issue": issue, "token": token, "started": now,
                                  "campaign_id": CAMPAIGN_ID,
                                  "deadline": (attempts[0]["started"] if attempts else now) + 1200,
                                  "requests": 0, "output": str(output)})
        persist(path, value)
        return token


def _controller_state_in_pixels(state):
    observation = state.copy()
    for group in ("enemies", "projectiles"):
        if group not in state:
            continue
        actors = []
        for actor in state[group]:
            converted = actor.copy()
            for field in ("x_pos", "y_pos", "x_velocity", "y_velocity"):
                value = actor.get(field)
                if value is not None:
                    # Native actor snapshots use signed 16.16 values, with
                    # position bits stored as u32. Kid/terrain already use pixels.
                    signed = value - (1 << 32) if value >= 1 << 31 else value
                    converted[field] = signed / 65536
            actors.append(converted)
        observation[group] = actors
    return observation


def choose(state, key):
    request = {"model": MODEL, "state": state, "questions": {"action": question_for_state(state)}}
    message = urllib.request.Request(ENDPOINT, data=json.dumps(request).encode(), headers={
        "Authorization": f"Bearer {key}", "Content-Type": "application/json"})
    with urllib.request.urlopen(message, timeout=10) as response:
        body = response.read(1_000_001)
    if len(body) > 1_000_000:
        raise ValueError("oversized response")
    return json.loads(body)


def decision(token, state, key, path=LEDGER, evaluate=choose, now=None):
    if not key:
        raise ValueError("TYPESAFE_API_KEY is required")
    now = time.time() if now is None else now
    with ledger(path) as value:
        attempt = next((a for a in value["attempts"] if a["token"] == token), None)
        if ("campaign" not in value or attempt is None
                or attempt.get("campaign_id") != CAMPAIGN_ID):
            raise ValueError("attempt is outside the active campaign")
        if now >= attempt["deadline"] or attempt["requests"] >= 600:
            raise ValueError("case collection limit reached")
        _, ceiling = _budget_bounds(value)
        if value["nanodollars_upper_bound"] + RESERVATION > ceiling:
            raise ValueError("cumulative API budget exhausted")
        state = _controller_state_in_pixels(state)
        value["nanodollars_upper_bound"] += RESERVATION
        attempt["requests"] += 1
        # Charge before dispatch. Crashes/timeouts retain the full reservation.
        persist(path, value)
        response = evaluate(state, key)
        tokens = response.get("usage", {}).get("input_tokens")
        if type(tokens) is not int or not 0 <= tokens <= 64_000:
            raise ValueError("invalid usage")
        value["nanodollars_upper_bound"] -= RESERVATION - tokens * 42
        persist(path, value)
        answer = response.get("answers", {}).get("action", {})
        if response.get("model") != MODEL or answer.get("type") != "choice" or answer.get("choice") not in question_for_state(state)["criteria"]:
            raise ValueError("invalid controller answer")
        with (Path(attempt["output"]) / "decisions.jsonl").open("a") as journal:
            journal.write(json.dumps({"state": state, "question": question_for_state(state), "response": response}) + "\n")
        return MASKS[answer["choice"]]


def main():
    signal.signal(signal.SIGALRM, deadline)
    signal.alarm(12)
    try:
        print(decision(os.environ["JEV_CASE_TOKEN"], json.load(sys.stdin), os.environ.get("TYPESAFE_API_KEY", "")))
    except Exception:
        # API exception text may contain request details. Retain accounting, not secrets.
        print("Jev decision failed; reservation retained unless usage was reconciled", file=sys.stderr)
        return 1
    finally:
        signal.alarm(0)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
