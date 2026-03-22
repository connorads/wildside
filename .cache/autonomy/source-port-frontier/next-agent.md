---
slug: any-percent-published-frame-7969-knights-isle-native-stays-grounded-while-oracle-lifts-off
cycle: 18
updated_at: 2026-03-22T23:18:46Z
---

# Source-Port Frontier Handoff

## What Changed

- Added a dedicated `kid_same_frame_stomp_probe` at the narrowest native point that matches the assigned question: immediately after `step_with_order(...)` finishes and before any post-motion mutator runs.
- Registered the new exploratory debug key in `docs/source-port/debug-field-registry.yaml` and added focused trace/published-TAS regressions for frames `7968..7969`.
- Re-ran the targeted TAS diff for frame `7968` plus the new regressions and `mise run verify-slice-fast`.

## Current Model

- The active frontier is still frame `7969` in `any-percent-published`, clean through `7968`.
- The new same-frame stomp probe stayed empty across the `7968..7969` window: native has no stomp-eligible enemy overlap at the closest pre-post-motion point, even though it preserves the ROM-style saved anchor (`0x03a80000,0x01bf0000`) and stays at zero vertical velocity there.
- That materially weakens the previous stomp-timing hypothesis. The blocker is less likely to be “native found the stomp too late” and more likely to be a secondary movement-state path that the current probes still do not expose, such as a wall-cling/live release path or an Iron Knight-specific transition that happens without a stomp candidate.
- Inference from the new probe plus the existing grounded-release/collision-handoff probes: the frame-7969 lift-off mismatch is still happening before any known native carry-state, grounded-release, special-motion latch, or same-frame stomp candidate becomes visible.

## Strongest Evidence

- `crates/kid-core/src/lib.rs`
  Result: `kid_same_frame_stomp_probe` is captured immediately after `slice.step_with_order(...)` and before any post-motion consumer runs.
- `crates/kid-core/src/combat.rs`
  Result: the probe reuses the same stomp classification gate as `process_stomps()` (`classify_enemy_contact_with_half_width(...)`) instead of inventing a parallel heuristic.
- `cargo run -p kidcli -- tas-diff --scenario .cache/tas-imports/any-percent-published/scenario.json --frame-start 7968 --frame-end 7969 --include-advisory --include-physics-trace --show-object-roster-on-match`
  Result: frame `7968` reports `kid_same_frame_stomp_probe = candidate=0/enemy=none/pre_y_vel=0x00000000/post_y_vel=0x00000000/anchor=0x03a80000,0x01bf0000`.
- `cargo test -p kid-core published_tas_frames_7968_7969_report_kid_same_frame_stomp_probe -- --nocapture`
  Result: both frames `7968` and `7969` keep `candidate=0/enemy=none` at the new probe point.
- `cargo test -p kid-core trace_frame_reports_kid_same_frame_stomp_probe -- --nocapture`
  Result: the trace output emits the new probe with enemy identity, velocity window, and saved anchor.
- `mise run audit-debug-fields`
  Result: clean after registering `kid_same_frame_stomp_probe`.

## Exact Next Moves

1. Return to the secondary movement-state paths from the earlier handoff: instrument wall-cling/live release and Iron Knight-specific transition state around frames `7968..7969`, now that the same-frame stomp probe stayed empty.
2. Re-read the ROM jump/live-state path around `loc_A74E -> sub_A4EE` for movement-side upward-velocity sources that do not require `Jump_On_Enemy`, then mirror the closest missing native state in a focused probe.
3. Do not implement a gameplay fix yet unless a new probe identifies a concrete native state path that explains the lift-off.

## Verification To Re-run

- `cargo test -p kid-core trace_frame_reports_kid_same_frame_stomp_probe -- --nocapture`
- `cargo test -p kid-core published_tas_frames_7968_7969_report_kid_same_frame_stomp_probe -- --nocapture`
- `cargo test -p kid-core trace_frame_reports_kid_collision_handoff_probe -- --nocapture`
- `cargo test -p kid-core published_tas_frames_7968_7969_report_kid_collision_handoff_probe -- --nocapture`
- `cargo run -p kidcli -- tas-diff --scenario .cache/tas-imports/any-percent-published/scenario.json --frame-start 7968 --frame-end 7969 --include-advisory --include-physics-trace --show-object-roster-on-match`
- `mise run audit-debug-fields`
- `mise run verify-slice-fast`

## Dead Ends / Do Not Retry

- Do not retry terrain-write theories for cell `58,28`; the support tile still matches the generated layout.
- Do not retry generic post-motion support/platform/lift handoff theories; the existing carry-state probes are still empty.
- Do not retry native goal-touch timing; the goal probe is still `lifecycle=alive` with `touch=0`.
- Do not retry broad special-motion latch theories; the existing special-motion probe is still idle in the live window.
- Do not retry same-frame stomp timing as the primary explanation unless a new movement-state probe later exposes a concrete stomp candidate. The new pre-post-motion stomp probe stays empty on both frames `7968` and `7969`.
