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
RESERVATION = 64_000 * 42
MASKS = {"release": 0, "left": 4, "right": 8, "jump": 16, "left_jump": 20,
         "right_jump": 24, "run_right": 40, "run_right_jump": 56,
         "run_left": 36, "run_left_jump": 52, "start": 128}


def persist(path, value):
    with tempfile.NamedTemporaryFile(mode="w", dir=path.parent, delete=False) as temporary:
        json.dump(value, temporary)
        temporary.flush()
        os.fsync(temporary.fileno())
    os.replace(temporary.name, path)


@contextmanager
def ledger(path=LEDGER):
    path.parent.mkdir(parents=True, exist_ok=True)
    with path.with_suffix(".lock").open("a") as lock:
        fcntl.flock(lock, fcntl.LOCK_EX)
        value = json.loads(path.read_text()) if path.exists() else {"nanodollars_upper_bound": PRIOR, "attempts": []}
        if not PRIOR <= value["nanodollars_upper_bound"] <= LIMIT:
            raise ValueError("invalid budget ledger")
        yield value


def begin_attempt(issue, output, path=LEDGER, now=None):
    now = time.time() if now is None else now
    with ledger(path) as value:
        attempts = [a for a in value["attempts"] if a["issue"] == issue]
        if len(attempts) >= 3 or attempts and now >= attempts[0]["started"] + 1200:
            raise ValueError("case collection limit reached")
        token = uuid.uuid4().hex
        value["attempts"].append({"issue": issue, "token": token, "started": now,
                                  "deadline": (attempts[0]["started"] if attempts else now) + 1200,
                                  "requests": 0, "output": str(output)})
        persist(path, value)
        return token


def choose(state, key):
    request = {"model": MODEL, "state": state, "questions": {"action": {"type": "choice",
        "instructions": "Choose ordinary controls for six game frames to achieve goal. Coordinates are pixels, right/down positive. B jumps; release B before a new jump. A runs. Momentum persists. Start releases a visible title card but pauses live gameplay. Hold still on the teleporter for 90 frames. Use nearby terrain to find a higher route if blocked; never change game state.",
        "criteria": {name: name.replace("_", " ") for name in MASKS}}}}
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
        attempt = next(a for a in value["attempts"] if a["token"] == token)
        if now >= attempt["deadline"] or attempt["requests"] >= 600:
            raise ValueError("case collection limit reached")
        if value["nanodollars_upper_bound"] + RESERVATION > LIMIT:
            raise ValueError("cumulative API budget exhausted")
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
        if response.get("model") != MODEL or answer.get("type") != "choice" or answer.get("choice") not in MASKS:
            raise ValueError("invalid controller answer")
        with (Path(attempt["output"]) / "decisions.jsonl").open("a") as journal:
            journal.write(json.dumps({"state": state, "response": response}) + "\n")
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
