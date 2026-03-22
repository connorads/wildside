---
slug: any-percent-published-frame-7969-knights-isle-native-stays-grounded-while-oracle-lifts-off
cycle: 21
updated_at: 2026-03-22T23:52:24Z
---

# Source-Port Frontier Handoff

## What Changed

- Added a focused `kid_iron_knight_attach_probe` around the native `step_iron_knight_climb` attach gate, capturing the raw C-edge, grounded/climbing state, forward wall contact, candidate status, and whether the handler actually injected `IRON_KNIGHT_ATTACH_Y_VEL`.
- Registered the new exploratory debug key and added both a unit formatter regression and a published-TAS regression for frames `7968..7969`.
- Re-ran the new published probe regression, the existing `published_tas_frames_7968_7969_report_kid_` suite, `tas-diff` for frames `7968..7969`, `mise run audit-debug-fields`, and `mise run verify-slice-fast`.

## Current Model

- The active frontier is still frame `7969` in `any-percent-published`, clean through `7968`.
- The new probe proves the exact native Iron Knight attach injector is present but still cold in the live window: frame `7968` carries a raw `C` edge, the forward wall contact is still solid at `cell=59,27`, but Kid is grounded (`ground=1`) so the attach candidate stays `0` and the handler leaves `y_velocity` flat at `0x00000000`.
- Inference from the new probe plus the earlier special-motion probe: oracle `kid_gfx y_velocity=0xFFFFC000` matches native `IRON_KNIGHT_ATTACH_Y_VEL`, but the native attach path is still blocked by the grounded gate rather than by missing wall contact or missing button input.

## Strongest Evidence

- `crates/kid-core/src/lib.rs`
  Result: `note_kid_iron_knight_attach_probe` records the pre-physics Iron Knight attach gate immediately around `step_iron_knight_climb`.
- `crates/kid-core/src/trace.rs`
  Result: `kid_iron_knight_attach_probe_debug` formats the raw C-edge, grounded/climbing transition, y-velocity window, wall contact, candidate bit, and attach bit.
- `crates/kid-core/src/tests.rs`
  Result: `trace_frame_reports_kid_iron_knight_attach_probe` and `published_tas_frames_7968_7969_report_kid_iron_knight_attach_probe` lock the probe format and the frame-7968/7969 findings in place.
- `cargo run -p kidcli -- tas-diff --scenario .cache/tas-imports/any-percent-published/scenario.json --frame-start 7968 --frame-end 7969 --include-advisory --include-physics-trace --show-object-roster-on-match`
  Result: frame `7968` now reports `kid_iron_knight_attach_probe = c_press=1/ground=1/climbing=0=>0/facing=right/y_vel=0x00000000->0x00000000/wall=hit=1/x=944/cell=59,27 collision=solid block=None has_block=0/candidate=0/attach=0`.
- `mise run audit-debug-fields`
  Result: clean after registering `kid_iron_knight_attach_probe`.
- `mise run verify-slice-fast`
  Result: passes.

## Exact Next Moves

1. Inspect the remaining post-`kid_flip` / `sub_AF10` consumers after the new attach probe, concentrating on grounded branches that still see the raw `C` edge on frame `7968` but do not require carried wall-cling state, stomp, goal touch, or block-breaker state.
2. Add the next focused probe at that post-`kid_flip` branch-selection boundary rather than broadening the new Iron Knight attach instrumentation.
3. Do not implement a gameplay fix yet unless that next probe exposes a concrete native state path for the oracle lift-off.

## Verification To Re-run

- `cargo test -p kid-core trace_frame_reports_kid_iron_knight_attach_probe -- --nocapture`
- `cargo test -p kid-core published_tas_frames_7968_7969_report_kid_iron_knight_attach_probe -- --nocapture`
- `cargo test -p kid-core published_tas_frames_7968_7969_report_kid_ -- --nocapture`
- `cargo run -p kidcli -- tas-diff --scenario .cache/tas-imports/any-percent-published/scenario.json --frame-start 7968 --frame-end 7969 --include-advisory --include-physics-trace --show-object-roster-on-match`
- `mise run audit-debug-fields`
- `mise run verify-slice-fast`

## Dead Ends / Do Not Retry

- Do not retry terrain-write theories for cell `58,28`; the support tile still matches the generated layout.
- Do not retry generic post-motion support/platform/lift handoff theories; the existing carry-state probes are still empty.
- Do not retry native goal-touch timing; the goal probe is still `lifecycle=alive` with `touch=0`.
- Do not retry broad special-motion latch theories; the existing special-motion probe is still idle in the live window.
- Do not retry same-frame stomp timing as the primary explanation unless a new movement-state probe later exposes a concrete stomp candidate.
- Do not retry ROM `sub_AF7A` / Iron Knight block-breaker as the primary explanation unless a later probe finds non-zero pre-physics horizontal speed or a block-bearing forward wall in the live window.
- Do not retry wall-cling live-release as the primary explanation; the `kid_wall_cling_release_probe` shows frames `7968..7969` never carry wall-cling state into the `loc_AA9C` mirror.
- Do not retry Iron Knight wall-attach as the primary explanation; the new `kid_iron_knight_attach_probe` shows frame `7968` does carry a raw `C` edge, but Kid is still grounded so the candidate stays `0` and no `-0x4000` attach velocity is injected.
- Do not rely on the broad `cargo test -p kid-core trace_frame_reports_kid_ -- --nocapture` filter as a frontier signal; it still trips the unrelated existing `trace_frame_reports_kid_object_support_candidates` assertion.
