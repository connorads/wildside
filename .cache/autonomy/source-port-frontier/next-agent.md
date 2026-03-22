---
slug: any-percent-published-frame-7969-knights-isle-native-stays-grounded-while-oracle-lifts-off
cycle: 19
updated_at: 2026-03-22T23:30:09Z
---

# Source-Port Frontier Handoff

## What Changed

- Added a focused `kid_iron_knight_block_breaker_probe` derived from the ROM `sub_AF7A` read: it reports the pre-physics Iron Knight speed gate plus the forward wall probe's cell/block state.
- Registered the new exploratory debug key and added trace plus published-TAS regressions for frames `7968..7969`.
- Re-ran the focused TAS diff, `mise run audit-debug-fields`, the new regressions, and `mise run verify-slice-fast`.

## Current Model

- The active frontier is still frame `7969` in `any-percent-published`, clean through `7968`.
- Re-reading `loc_A74E -> sub_A4EE` surfaced `sub_AF7A` as the only clear Iron Knight-specific upward-velocity source in the jump/live loop that does not require `Jump_On_Enemy`.
- The new native probe rules that path out for the live window: frame `7968` has `x_vel=0`, so the ROM speed gate is cold, and the forward wall contact at cell `59,27` is a plain solid tile with `block=None` and `has_block=0`, not a block-bearing wall.
- Inference from the new probe plus the earlier grounded-release/collision-handoff/same-frame-stomp probes: the frame-7969 lift-off mismatch is still happening before any known native stomp, goal, special-motion, grounded-release, or Iron Knight block-breaker state becomes active.

## Strongest Evidence

- `crates/kid-core/src/trace.rs`
  Result: `kid_iron_knight_block_breaker_probe` emits the ROM-derived Iron Knight speed gate and forward wall/block contact from the pre-physics Kid snapshot.
- `crates/kid-core/src/tests.rs`
  Result: `trace_frame_reports_kid_iron_knight_block_breaker_probe` proves the new probe reports a positive candidate when speed and a block-bearing wall align.
- `cargo test -p kid-core published_tas_frames_7968_7969_report_kid_iron_knight_block_breaker_probe -- --nocapture`
  Result: frames `7968` and `7969` both keep `x_vel=0x00000000`, `speed_gate=0`, and `candidate=0`.
- `cargo run -p kidcli -- tas-diff --scenario .cache/tas-imports/any-percent-published/scenario.json --frame-start 7968 --frame-end 7969 --include-advisory --include-physics-trace --show-object-roster-on-match`
  Result: frame `7968` now shows `kid_iron_knight_block_breaker_probe = helmet=0x0006/facing=right/x_vel=0x00000000/speed_gate=0/wall=hit=1/x=944/cell=59,27 collision=solid block=None has_block=0/candidate=0` while oracle Kid gfx still has `y_velocity=0xFFFFC000`.
- `mise run audit-debug-fields`
  Result: clean after registering `kid_iron_knight_block_breaker_probe`.
- `mise run verify-slice-fast`
  Result: passes.

## Exact Next Moves

1. Instrument the native wall-cling/live-release mirrors of ROM `loc_AA9C`, `sub_ADB8`, and `sub_AE6E` around frames `7968..7969`, using the now-proven forward wall contact at cell `59,27` to see whether the release gates are cold or missing state.
2. If those release probes stay cold, inspect the remaining post-`sub_A4EE` jump-loop path after `loc_A94E` for a native-equivalent consumer that can inject upward velocity without stomp, goal touch, or block-breaker state.
3. Do not implement a gameplay fix yet unless a new probe exposes a concrete native state path for the oracle `y_velocity=0xFFFFC000` lift-off.

## Verification To Re-run

- `cargo test -p kid-core trace_frame_reports_kid_iron_knight_block_breaker_probe -- --nocapture`
- `cargo test -p kid-core published_tas_frames_7968_7969_report_kid_iron_knight_block_breaker_probe -- --nocapture`
- `cargo test -p kid-core published_tas_frames_7968_7969_report_kid_collision_handoff_probe -- --nocapture`
- `cargo test -p kid-core published_tas_frames_7968_7969_report_kid_grounded_release_probe -- --nocapture`
- `cargo test -p kid-core published_tas_frames_7968_7969_report_kid_same_frame_stomp_probe -- --nocapture`
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
