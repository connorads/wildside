---
slug: any-percent-published-frame-7969-knights-isle-native-stays-grounded-while-oracle-lifts-off
cycle: 20
updated_at: 2026-03-22T23:42:09Z
---

# Source-Port Frontier Handoff

## What Changed

- Added a focused `kid_wall_cling_release_probe` at the native `loc_AA9C` mirror in `OpeningSliceState::step_with_order`, immediately after floor/ceiling resolution and before the two wall-cling live-release helpers run.
- Registered the new exploratory debug key and added both a unit formatter regression and a published-TAS regression for frames `7968..7969`.
- Re-ran the new published probe regression, the existing `published_tas_frames_7968_7969_report_kid_*` probe suite, `tas-diff` for frames `7968..7969`, `mise run audit-debug-fields`, and `mise run verify-slice-fast`.

## Current Model

- The active frontier is still frame `7969` in `any-percent-published`, clean through `7968`.
- The new probe shows native reaches the `loc_AA9C` mirror with `held=right`, `probe_dir=right(held)`, `wall_cling=none`, `sticky=R0L0`, `ground=1`, `y_vel=0`, and no vertical displacement (`y=447=>447`).
- Native still sees the proven forward wall contact at `cell=59,27`, and the new top-edge probe also hits solid terrain at `cell=59,26`.
- Inference from the new probe plus earlier grounded-release/collision-handoff/same-frame-stomp/block-breaker probes: frame `7969` is not missing an existing native wall-cling live-release. Native never carries wall-cling state into the live window, so both `sub_ADB8` and `sub_AE6E` stay cold because the prerequisite state is absent.

## Strongest Evidence

- `crates/kid-core/src/movement.rs`
  Result: `record_wall_cling_release_probe` captures the native `loc_AA9C` mirror inputs and the forward/top-edge wall contacts before the release helpers run.
- `crates/kid-core/src/tests.rs`
  Result: `trace_frame_reports_kid_wall_cling_release_probe` and `published_tas_frames_7968_7969_report_kid_wall_cling_release_probe` lock the new probe format and the frame-7968/7969 findings in place.
- `cargo test -p kid-core published_tas_frames_7968_7969_report_kid_ -- --nocapture`
  Result: all six frame-7968/7969 probe regressions pass, including the new wall-cling live-release probe.
- `cargo run -p kidcli -- tas-diff --scenario .cache/tas-imports/any-percent-published/scenario.json --frame-start 7968 --frame-end 7969 --include-advisory --include-physics-trace --show-object-roster-on-match`
  Result: frame `7968` now reports `kid_wall_cling_release_probe = held=right/probe_dir=right(held)/wall_cling=none/sticky=R0L0/ground=1/y_vel=0x00000000/y=447=>447/branch=no_wall_cling_state/sub_ADB8=wall_cling_missing/sub_AE6E=wall_cling_missing/wall=hit=1/x=944/cell=59,27 collision=solid block=None has_block=0/top_edge=hit=1/x=944/cell=59,26 collision=solid block=None has_block=0`.
- `mise run audit-debug-fields`
  Result: clean after registering `kid_wall_cling_release_probe`.
- `mise run verify-slice-fast`
  Result: passes.

## Exact Next Moves

1. Inspect the remaining post-`loc_A94E` jump/live loop after the new `loc_AA9C` probe, concentrating on native-equivalent consumers that can inject oracle `y_velocity=0xFFFFC000` without stomp, goal touch, block-breaker state, or carried wall-cling release state.
2. Add the next focused probe at that post-`loc_A94E` consumer boundary rather than broadening the current wall-cling instrumentation.
3. Do not implement a gameplay fix yet unless that next probe exposes a concrete native state path for the oracle lift-off.

## Verification To Re-run

- `cargo test -p kid-core trace_frame_reports_kid_wall_cling_release_probe -- --nocapture`
- `cargo test -p kid-core published_tas_frames_7968_7969_report_kid_wall_cling_release_probe -- --nocapture`
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
- Do not retry wall-cling live-release as the primary explanation; the new `kid_wall_cling_release_probe` shows frames `7968..7969` never carry wall-cling state into the `loc_AA9C` mirror.
- Do not rely on the broad `cargo test -p kid-core trace_frame_reports_kid_ -- --nocapture` filter as a frontier signal; it still trips the unrelated existing `trace_frame_reports_kid_object_support_candidates` assertion.
