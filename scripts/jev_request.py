"""One private playtest decision over stdin/stdout; credentials stay in the environment."""

import json
import os
import signal
import sys
import urllib.error
import urllib.request

ENDPOINT = "https://api.typesafe.ai/v1/systemone"
MODEL = "jev-1.13.0"
CHOICES = {
    "release": "Release all buttons. Brake naturally, or wait after hitting the target.",
    "left": "Hold Left; accelerate or brake towards the left.",
    "right": "Hold Right; accelerate or brake towards the right.",
    "jump": "Hold B (jump), without horizontal input.",
    "left_jump": "Hold Left and B (jump).",
    "right_jump": "Hold Right and B (jump).",
}


def evaluate(state, key, *, endpoint=ENDPOINT):
    if not key:
        raise ValueError("TYPESAFE_API_KEY is missing")
    request = {
        "model": MODEL,
        "state": state,
        "questions": {"action": {
            "type": "choice",
            "instructions": (
                "Choose controls for the next 6 simulation frames to reach the stated goal. "
                "Use current position, velocity, nearby terrain and recent decisions. "
                "Walk underneath the target before jumping; do not jump early from the start. "
                "When jump_requires_release is true, choose release, left or right first. "
                "Holding B extends a jump; release B "
                "before trying another jump. Horizontal momentum persists after release. "
                "Do not try to finish the level. Once target_hit is true, release controls."
            ),
            "criteria": CHOICES,
        }},
    }
    message = urllib.request.Request(endpoint, data=json.dumps(request).encode(), headers={
        "Authorization": f"Bearer {key}", "Content-Type": "application/json",
    })
    # No retries: the Rust runner reserves spend before each invocation.
    with urllib.request.urlopen(message, timeout=10) as response:
        body = response.read(1_000_001)
    if len(body) > 1_000_000:
        raise ValueError("response exceeds private adapter limit")
    return json.loads(body)


def deadline(*_):
    raise TimeoutError("request deadline reached")


def main():
    signal.signal(signal.SIGALRM, deadline)
    signal.alarm(12)
    try:
        result = evaluate(json.load(sys.stdin), os.environ.get("TYPESAFE_API_KEY", ""))
        print(json.dumps(result))
        return 0
    except urllib.error.HTTPError as error:
        print(f"TypeSafe HTTP {error.code}", file=sys.stderr)
    except (ValueError, OSError, TimeoutError):
        print("TypeSafe request failed; credentials and response body withheld", file=sys.stderr)
    finally:
        signal.alarm(0)
    return 1


if __name__ == "__main__":
    sys.exit(main())
