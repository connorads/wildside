---
slug: any-percent-published-frame-7969-knights-isle-native-stays-grounded-while-oracle-lifts-off
cycle: 24
updated_at: 2026-03-23T00:25:17Z
---

# Source-Port Frontier Handoff

## What Changed

- Added a focused `kid_jump_gravity_probe` around the native `apply_jump_and_gravity_vel_only(...)` call and registered the new exploratory debug field.
- Added formatter coverage plus a published-TAS regression for frames `7968..7969`.
- Re-ran the targeted `kid_flip_*` and new `kid_jump_gravity_probe` tests, the broader `published_tas_frames_7968_7969_report_kid_` suite, `tas-diff` for frames `7968..7969`, `mise run audit-debug-fields`, and `mise run verify-slice-fast`.

## Current Model

- The active frontier is still frame `7969` in `any-percent-published`, clean through `7968`.
- The shared `kid_flip` fallthrough is still correct through the `loc_AA22` mirror: `c_press=1`, wall scan hit, grounded Iron Knight state, and no post-motion mutator.
- The new probe narrows the blocker further: on frame `7968`, the native `apply_jump_and_gravity_vel_only(...)` call itself takes `branch=grounded_hold` with `jump=0`, `jump_buffer=0=>0`, `just_jumped=0=>0`, `b_held=0`, `custom_gravity=0`, `y_vel=0x00000000->0x00000000`, and `ground=1=>1`.
- Inference from the disassembly plus the new probe: the missing oracle lift-off is not explained by branch selection, wall-cling release, floor/wall handling, or a local gravity tweak inside the native call. The next likely gap is an omitted pre-`sub_B168` state write or carry-in condition that should already exist before the jump/gravity routine runs.

## Strongest Evidence

- `crates/kid-core/src/movement.rs`
  Result: `apply_jump_and_gravity_vel_only(...)` now records the exact native gate branch plus pre/post `y_vel`, grounded, jump-buffer, and `just_jumped` state.
- `crates/kid-core/src/trace.rs`
  Result: `kid_jump_gravity_probe_debug()` emits a compact summary for live-frame inspection.
- `crates/kid-core/src/tests.rs`
  Result: `trace_frame_reports_kid_jump_gravity_probe` and `published_tas_frames_7968_7969_report_kid_jump_gravity_probe` lock the new probe into the frontier suite.
- `cargo run -p kidcli -- tas-diff --scenario .cache/tas-imports/any-percent-published/scenario.json --frame-start 7968 --frame-end 7969 --include-advisory --include-physics-trace --show-object-roster-on-match`
  Result: frame `7968` now reports `kid_jump_gravity_probe = branch=grounded_hold/jump=0/jump_buffer=0=>0/just_jumped=0=>0/b_held=0/custom_gravity=0/y_vel=0x00000000->0x00000000/ground=1=>1`, while oracle `kid_gfx` is already at `y_velocity=0xffffc000`.
- `mise run audit-debug-fields`
  Result: clean after registering `kid_jump_gravity_probe`.
- `mise run verify-slice-fast`
  Result: passes.

## Exact Next Moves

1. Inspect the ROM path immediately before and into `loc_AA22` / `sub_B168`, concentrating on what state write or carry-in condition makes oracle enter the routine with lift-off already armed; do not revisit wall-cling, floor, or later post-motion systems.
2. Compare native equivalents for the pre-`sub_B168` setup that still remains unprobed in this frontier, especially any `Character_Movement = Jump`, `Character_CheckCollision`, or other carry-in state that can feed `y_vel` before the shared jump/gravity call.
3. Do not implement a gameplay fix yet unless that narrower pre-`sub_B168` inspection exposes the concrete missing native state write or gate.

## Verification To Re-run

- `cargo test -p kid-core trace_frame_reports_kid_flip_branch_probe -- --nocapture`
- `cargo test -p kid-core trace_frame_reports_kid_flip_follow_on_probe -- --nocapture`
- `cargo test -p kid-core trace_frame_reports_kid_jump_gravity_probe -- --nocapture`
- `cargo test -p kid-core published_tas_frames_7968_7969_report_kid_flip_branch_probe -- --nocapture`
- `cargo test -p kid-core published_tas_frames_7968_7969_report_kid_flip_follow_on_probe -- --nocapture`
- `cargo test -p kid-core published_tas_frames_7968_7969_report_kid_jump_gravity_probe -- --nocapture`
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
- Do not retry Iron Knight wall-attach as the primary explanation; the `kid_iron_knight_attach_probe` shows frame `7968` does carry a raw `C` edge, but Kid is still grounded so the candidate stays `0` and no `-0x4000` attach velocity is injected.
- Do not retry "missing raw C-edge at kid_flip" as the primary explanation; the `kid_flip_branch_probe` still shows `c_press=1`, `wall_scan=1`, and `branch=c_pressed_fallthrough`.
- Do not retry `sub_B270`, floor collision, wall collision, or later post-motion systems as primary lift-off causes unless `kid_flip_follow_on_probe` stops reporting the flat `x_vel=0/y_vel=0/ground=1/wall_cling=none` stage sequence.
- Do not retry fixing `apply_jump_and_gravity_vel_only(...)` in isolation based only on the new `grounded_hold` result; the probe shows the native call enters with no jump, no jump-buffer carry, no `just_jumped`, no held-B low-gravity gate, and no custom-gravity override, so the missing lift-off is upstream of the call.
- Do not rely on the broad `cargo test -p kid-core trace_frame_reports_kid_ -- --nocapture` filter as a frontier signal; it still trips the unrelated existing `trace_frame_reports_kid_object_support_candidates` assertion.
