# Jev prize-block playtest

Campaign discovery uses [the case runner and shared ledger](jev-cases.md).
This prize-block pilot documents its separate experiment.

This private experiment asks Jev to hit the forest prize block at cell `(8, 12)`
in level `0x00`, then retains the `sub_1007A` animation for independent review of
#372. The controller reads internal state; visual assessment uses native and
oracle images. A successful hit is not a visual pass.

The runtime pauses during each decision and applies six frames of normal
controller input. The input journal uses the standard settled-recording
one-frame feed delay; it records the controller script, not immediate runtime
arguments. Attempts stop on death or exit from gameplay. The test-only runner shares recording initialisation and
rendering with `verify-recording`. It changes no public interface or stored
recording format. It does not assess real-time latency or audio devices.

## Run

The ROM, generated assets and oracle must be available. First run the offline
control, which proves exact native reconstruction and that observations do not
advance the simulation:

```sh
cargo test -p wildside-cli gameplay_regression_jev_recording_feed -- --nocapture
```

Provide `TYPESAFE_API_KEY` in the environment, then run the paid pilot explicitly:

```sh
cargo test -p wildside-cli recording::jev_playtest::pilot -- --ignored --exact --nocapture
```

The private Python helper uses the standard library and the pinned
`jev-1.13.0` model. No package installation is needed. Credentials are excluded
from requests saved as evidence and from error messages.

Limits are three attempts, 600 decisions and 3,600 simulated frames per attempt,
20 minutes of decision collection, and $1 across the run. Each request reserves
the model's maximum 64,000 input tokens at $0.042 per million before dispatch.
Successful responses reconcile that reservation against reported usage; failed
or timed-out requests retain it. No automatic retries occur. The HTTP helper
has a 12-second total deadline. Allow a further ten minutes for comparison.
For a revised experiment, carry previous spend through `JEV_PRIOR_NANODOLLARS`
and restrict remaining attempts with `JEV_ATTEMPTS` (1..3); neither can increase
the $1 ceiling. One dollar is 1,000,000,000 nanodollars.

## Evidence and verdict

Each run writes a fresh `.cache/jev-playtest/<run>/` directory containing
model decisions, applied-input journals, budget accounting and standard recording
bundles. Exact native replay is required before rendering. The animation capture
starts 30 frames before the selected prize worker first appears and ends 30
frames after it disappears. An interrupted capture remains incomplete.

Replay the standard bundle with `verify-recording --bundle <path>`. Compare the
same scenario with `capture-oracle-scenario-frames`, using every checkpoint
frame. Inspect chronological full-frame images and enlarged target crops beside
the native sequence. Check that the background actually overlaps the wobble area.
Earlier state divergence prevents attributing later pixel differences to #372.

Report one of:

- **Bug reproduced**: the selected interaction has the reported visual defect
  in native, while the comparable oracle sequence does not.
- **Pass on this route**: the interaction and background overlap are confirmed,
  and the relevant native animation matches the oracle.
- **Not exercised**: the selected interaction never occurs.
- **Inconclusive**: replay, state comparability, capture completeness or visual
  evidence is insufficient.

The runner records `awaiting_visual_review`; it cannot award a visual pass.
Keep human-validation rows open. Promote useful input scenarios into
`tests/data/dynamic-scenarios/` so later checks require no model calls.

## Choice of approach

Existing scripts are the cheapest repeatable control but cannot adapt their
route. Jev plus structured state fits the native traces and recording tools;
independent frame review compensates for its inability to see missing sprites.
A screenshot-driven player would test visual perception directly but needs a
different model and introduces perception errors. This bounded internal harness
keeps the selected approach removable without engine or recording migrations.

API and model constraints: [HTTP API](https://docs.typesafe.ai/api),
[model limits and pricing](https://docs.typesafe.ai/models),
[text-only state](https://docs.typesafe.ai/concepts/state).

## Pilot evidence

The initial live attempt (`1789697295663`) did not exercise the target. Jev
jumped early and held Jump after landing. After 73 requests, the operator
terminated its helper; the partial 432-frame recording was preserved. Its
API cost upper bound was $0.01278774, including the interrupted request.

Exact replay rejected that recording at frame 1. The harness omitted the
settled recording adapter's one-frame input feed delay; the neutral control
could not discriminate this error. The control therefore includes directional,
jump and release inputs. This attempt supplies no evidence about #372.

## Successful model-generated route

[The successful-run report](jev-successful-play.md) records a complete Jev
attempt, independent emulator comparison and an offline regression fixture.
The controller receives compact target relations and movement descriptions.
Its target-distance bands are task-specific hints; they do not predict collision
or simulate candidate actions. Jev still selects every controller batch.

## Historical-defect comparison

`mise run jev-proof` compares the saved Jev route with an isolated restoration
of #372's scene-rendering defect. The [runner guide](jev-proof.md) explains the
verdicts; the [measured result](jev-proof-result.md) records why this case does
not expose that defect through the active VDP renderer. The command requires
no model credentials and does not treat two green builds as a successful proof.
