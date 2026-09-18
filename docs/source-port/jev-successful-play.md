# Jev-generated prize-block regression

Jev completed the target interaction in live run `1789702385770` on 2026-09-18.
It chose every six-frame controller batch from the settled level start. There
was no scripted prefix, action override or state teleport during play.

The observation describes target distance and direction, movement and held
buttons. It omits the entire-level prize list and nearby raw tile dump. The
prompt encourages jumping near the target instead of walking directly beneath
it first. These are task-specific controller hints, not evidence of general
platform-game competence. The six action choices remain unchanged.

Jev first overshot, bounced above the prize row, walked off it, then returned
underneath. It eventually released and pressed Jump again. The target prize
worker appeared at zero-based frame 376 and remained through 406. Capture ended
at 436, retaining the complete 30-frame trailing window. Kid retained both hit
points. The scripted control reaches the same target at frame 112.

The run made 73 successful API requests using 57,970 input tokens. Its cost was
$0.002434740; the cumulative pilot upper bound is $0.024954048. Median request
latency was 699 ms with simulation paused. This is one successful attempt,
not a measured success rate. Earlier unsuccessful runs also used the older
bootstrap, so they are not a controlled comparison of observation formats.

## Independent verdict

`verify-recording` passes for the 437-frame bundle. Native replay is exact.
The oracle state comparison passes and all 91 full-frame captures, frames
346-436, match pixel for pixel. The oracle emitted no enemy snapshots, so
per-enemy state comparison was skipped; full-frame comparison still covers
visible enemies.

Paired images at 370, 376, 382, 390, 400 and 406 were inspected. The moving
prize overlaps the forest background and matches the original throughout the
captured animation. Verdict: **pass on this route**. This does not reproduce
#372 or establish that every route is fixed; the human-validation row stays
open. The earlier scripted control's seven sprite differences remain separate
unresolved evidence.

## Repeat without Jev

The [saved input scenario](../../tests/data/dynamic-scenarios/issue-372-jev-prize-recovery.json)
contains the complete controller stream. A normal CLI regression test requires
the same hit window, survival and exact replay, without credentials or network:

```sh
cargo test -p wildside-cli gameplay_regression_jev_discovered_prize_route_sub_1007a
```

This test requires the local ROM and generated assets, like the existing
recording-feed regression. It protects route reachability and replay; fresh
pixel fidelity still requires the oracle comparison.

[Machine-readable evidence](jev-successful-play-evidence.json) preserves the
73 decisions, input/trace hashes, cost and comparison summary. Full local
artefacts are under `.cache/jev-playtest/1789702385770/attempt-1`, including
`verify/report.json` and `comparison-contact.png`. The recorded revision is
`02475f5c1` with the observation/prompt edits uncommitted at capture time.

The [run guide](jev-playtesting.md) describes new paid attempts. Carry
`JEV_PRIOR_NANODOLLARS=24954048` into further attempts sharing the pilot budget.

## Verification

All five Jev harness tests and both HTTP-adapter tests pass. `verify-slice-fast`
passes, including 7,406 core tests, CLI regressions, lint and the existing TAS
frontier checks. The parallel harness tests exposed millisecond output-directory
collisions; run directory names use nanosecond timestamps.
