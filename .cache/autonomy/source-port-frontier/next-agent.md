---
slug: any-percent-published-frame-7969-knights-isle-native-stays-grounded-while-oracle-lifts-off
cycle: 25
updated_at: 2026-03-23T00:33:42Z
---

# Source-Port Frontier Handoff

## What Changed

- Extended `kid_collision_handoff_probe` so it now covers the closest native pre-`sub_B168` collision-bounce surrogate: `form_flash`, pre-physics stomp candidate/enemy, and the saved anchor.
- Updated the trace/unit regression and the published-TAS `7968..7969` regression to lock the new handoff fields.
- Re-ran the focused collision-handoff tests, the broader published-TAS `7968..7969` frontier probe suite, `mise run audit-debug-fields`, and `mise run verify-slice-fast`.

## Current Model

- The active frontier is still frame `7969` in `any-percent-published`, clean through `7968`.
- ROM `loc_AA22` still looks like the critical pre-`sub_B168` shell: it forces `Character_Movement=Jump`, then runs `Character_CheckCollision` before the shared jump/gravity routine.
- The new native handoff probe narrows the gap further: on frames `7968..7969`, native reaches the pre-physics boundary with `pending_form=none`, `form_flash=0`, `stomp_candidate=0`, `stomp_enemy=none`, and anchor `0x03a80000,0x01bf0000`.
- Inference from the ROM path plus the new probe: the nearest native surrogates for `sub_7ACC` and `Jump_On_Enemy` are cold in the live window, so the missing oracle lift-off is still upstream of `apply_jump_and_gravity_vel_only(...)`, but it is no longer well explained by a deferred stomp carry or transform/helmet-change handoff.

## Strongest Evidence

- `crates/kid-core/src/lib.rs`
  Result: `begin_kid_collision_handoff_probe(...)` now records the native pre-physics stomp candidate/enemy, form-flash latch, and saved anchor as the closest mirror of ROM's `Character_CheckCollision`/`Jump_On_Enemy` carry-in before `sub_B168`.
- `crates/kid-core/src/trace.rs`
  Result: `kid_collision_handoff_probe` now emits `form_flash`, `stomp_candidate`, `stomp_enemy`, and `anchor` in one compact summary.
- `crates/kid-core/src/tests.rs`
  Result: both the trace-unit test and the published-TAS `7968..7969` regression now assert `form_flash=0`, `stomp_candidate=0`, `stomp_enemy=none`, and anchor `0x03a80000,0x01bf0000`.
- `cargo test -p kid-core kid_collision_handoff_probe -- --nocapture`
  Result: passes.
- `cargo test -p kid-core published_tas_frames_7968_7969_report_kid_ -- --nocapture`
  Result: passes all 10 frontier probe regressions.
- `cargo run -p kidcli -- tas-diff --scenario .cache/tas-imports/any-percent-published/scenario.json --frame-start 7968 --frame-end 7969 --include-advisory --include-physics-trace --show-object-roster-on-match`
  Result: oracle still shows frame `7968` `0x30(kid_gfx)` with `y_velocity=0xffffc000`, while native now reports `kid_collision_handoff_probe = lifecycle=alive/prev_post=none/pending_stomp=0/pending_form=none/pending_level_skip=none/form_flash=0/ground=1/jump_buffer=0/wall_cling=none/stomp_candidate=0/stomp_enemy=none/anchor=0x03a80000,0x01bf0000`.
- `mise run audit-debug-fields`
  Result: clean after updating the collision-handoff field meaning in the registry.
- `mise run verify-slice-fast`
  Result: passes.

## Exact Next Moves

1. Inspect ROM writers of `collision_type(a3)`, `object_meta(a3)`, and `$FFFFFA6A` that can feed `Character_CheckCollision` immediately before `loc_AA22` in the frame-`7968` window; compare them to native systems that could arm an equivalent carry before physics.
2. Compare the ROM state write `Character_Movement=Jump` at `loc_AA22` against native pre-physics state: determine whether a missing native jump-mode latch or consumer exists before `apply_jump_and_gravity_vel_only(...)`, rather than another gravity/floor issue.
3. Do not implement a gameplay fix yet unless that narrower pre-`sub_B168` inspection exposes the concrete upstream write or consumer that should already have armed the oracle's `y_vel=-0x00004000`.

## Verification To Re-run

- `cargo test -p kid-core kid_collision_handoff_probe -- --nocapture`
- `cargo test -p kid-core published_tas_frames_7968_7969_report_kid_ -- --nocapture`
- `cargo run -p kidcli -- tas-diff --scenario .cache/tas-imports/any-percent-published/scenario.json --frame-start 7968 --frame-end 7969 --include-advisory --include-physics-trace --show-object-roster-on-match`
- `mise run audit-debug-fields`
- `mise run verify-slice-fast`

## Dead Ends / Do Not Retry

- Do not retry terrain-write theories for cell `58,28`; the support tile still matches the generated layout.
- Do not retry generic post-motion support/platform/lift handoff theories; the existing carry-state probes are still empty.
- Do not retry native goal-touch timing; the goal probe is still `lifecycle=alive` with `touch=0`.
- Do not retry broad special-motion latch theories; the existing special-motion probe is still idle in the live window.
- Do not retry same-frame stomp timing as the primary explanation unless a later probe finds a concrete positive pre-`sub_B168` stomp/collision candidate; the new collision-handoff probe still reports `stomp_candidate=0`.
- Do not retry transform/helmet-change handoff as the primary explanation unless a later probe finds a non-zero pending form or flash latch in the live window; the new collision-handoff probe still reports `pending_form=none` and `form_flash=0`.
- Do not retry wall-cling live-release, Iron Knight attach, `sub_B270`, floor collision, wall collision, or `apply_jump_and_gravity_vel_only(...)` in isolation as the primary lift-off cause unless a new upstream carry probe stops reporting the current flat grounded state.
