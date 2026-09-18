# Jev #372 pilot result

The 2026-09-18 pilot is **inconclusive for #372**. Jev did not hit the selected
prize block. A separate scripted control reached it and showed matching wobble
pixels, but the full oracle comparison failed RNG parity. The awaiting-human
issue remains open. No gameplay code changed.

## Model attempts

| Attempt | Result |
| --- | --- |
| Initial observation format | Jumped too early, held Jump after landing and missed the block. Operator stopped the request helper after 432 frames. Exact replay exposed a harness input-feed error at frame 1. |
| Explicit held buttons and revised instructions | Walked past the target, reversed and died at frame 128 without jumping. All 129 recorded frames replay exactly. |
| Repeat of revised instructions | Produced the same input stream and outcome. All 129 frames replay exactly. |

The initial harness fed settled-recording inputs one frame early. The neutral
control missed that defect; a directional/jump/release replay control reproduced
it and is included in the gameplay regression gate. The corrected live attempts
also reproduce exactly. Their failures measure this controller's performance on
this task, not a game defect.

There were 117 requests, including one interrupted request. The 116 successful
responses reported 472,174 input tokens. Known cost is $0.019831308; including
the interrupted request's maximum reservation, the upper bound is $0.022519308.
Median successful request duration was 695 ms, including helper startup and
network time. No further paid attempts were made.

## Scripted reachability control

The [saved scenario](../../tests/data/dynamic-scenarios/issue-372-forest-prize-natural-control.json)
uses normal controller inputs from the same settled level-0 start. It is
hand-authored and is not a successful Jev episode. Its first jump misses; a
second jump beneath cell `(8, 12)` activates the prize worker.

- Exact native reconstruction passes for all 173 frames.
- The `sub_1007A` prize worker starts at frame 112 and is present through 142.
- All 91 frames from 82 through 172 were captured in native and oracle.
- The 16×36 world-pixel crop at `(128, 176)` has zero changed pixels throughout
  the 31 active wobble frames. Paired full frames and enlarged crops show the P
  block rising/falling visibly over the forest art.
- The full capture has 3,738 changed pixels across 47 frames. The target crop
  differs after the wobble on frames 148, 149, 160, 161, 166, 167 and 172.
  These differences remain evidence; they are not masked out of verification.
- RNG offsets differ from the initial frame, native `0xFFF2` versus oracle
  `0xFFF3`. `verify-recording` rejects oracle state at the first requested
  checkpoint, frame 82. Visual agreement cannot turn that into a fidelity pass.

The RNG result resembles the [replay-exploration preflight failure](replay-exploration-pilot.md),
but a shared root cause is unproven. A later investigation must establish
comparable starts before awarding a bug-validation pass.

## Reproduce and inspect

The [harness guide](jev-playtesting.md) covers the private runner. Reproduce the
successful control without API calls:

```sh
cargo test -p wildside-cli recording::jev_playtest::scripted_case -- --ignored --exact --nocapture
```

The printed directory contains a standard `scripted-control` recording bundle.
Run `verify-recording --bundle <directory>/scripted-control` to obtain the full
state/pixel report. Its oracle-state failure is the recorded negative result,
not a check to weaken.

Local raw evidence is under `.cache/jev-playtest/`:

- `1789697295663`: initial attempt, decisions and interrupted-request reservation.
- `1789697507842`: both corrected model attempts and their exact recordings.
- `1789697657267/scripted-control`: native/oracle images, `verify/report.json`,
  `comparison-contact.png`, `target-contact.png` and pixel-count reports.

The [compact evidence record](jev-playtest-pilot-evidence.json) retains input
hashes, cost, source identity and measurements. Raw cache artefacts are local;
the checked-in scenario reconstructs the control without them.

The pilot does not justify expanding autonomous Jev playtesting. A separate
experiment could test semantic spatial observations against the recorded
misses, while keeping the replay and oracle comparison rules fixed.
