---
slug: any-percent-published-frame-7969-knights-isle-native-stays-grounded-while-oracle-lifts-off
cycle: 23
updated_at: 2026-03-23T00:13:12Z
---

# Source-Port Frontier Handoff

## What Changed

- Added a focused `kid_flip_follow_on_probe` inside the native shared post-`kid_flip` movement shell, recording the entry, press-away, jump/gravity, floor, and wall stages before later post-motion systems run.
- Registered the new exploratory debug key and added both a formatter regression and a published-TAS regression for frames `7968..7969`.
- Re-ran the new probe tests, the existing `kid_flip_branch_probe` tests, the broader `published_tas_frames_7968_7969_report_kid_` suite, `tas-diff` for frames `7968..7969`, `mise run audit-debug-fields`, and `mise run verify-slice-fast`.

## Current Model

- The active frontier is still frame `7969` in `any-percent-published`, clean through `7968`.
- Frame `7968` still reaches the shared `kid_flip` fan-out with `c_press=1`, `wall_scan=1`, grounded Iron Knight state, and `branch=c_pressed_fallthrough`.
- The new follow-on probe narrows the live blocker further: once native enters the shared `loc_AA22` mirror, every recorded stage stays flat on frame `7968` (`x_vel=0`, `y_vel=0`, `ground=1`, `wall_cling=none`) through press-away, jump/gravity, floor, and wall handling.
- Inference from the new probe plus the earlier branch/attach probes: the missing oracle lift-off is no longer inside `sub_B270`, wall-cling release, floor collision, wall collision, or any later post-motion consumer. The next likely gap is the native precondition or omitted side effect that should feed the shared jump/gravity path before or at `apply_jump_and_gravity_vel_only(...)`.

## Strongest Evidence

- `crates/kid-core/src/movement.rs`
  Result: `step_with_order(...)` now snapshots the shared post-fallthrough follow-on stages around the native `loc_AA22` mirror.
- `crates/kid-core/src/lib.rs`
  Result: `note_kid_flip_branch_probe(...)` now stores both the shared branch-selection result and the staged follow-on probe for the same frame window.
- `crates/kid-core/src/trace.rs`
  Result: `kid_flip_follow_on_probe_debug()` formats the stage-by-stage velocity, grounded, and wall-cling state.
- `crates/kid-core/src/tests.rs`
  Result: `trace_frame_reports_kid_flip_follow_on_probe` and `published_tas_frames_7968_7969_report_kid_flip_follow_on_probe` lock the new probe in place.
- `cargo run -p kidcli -- tas-diff --scenario .cache/tas-imports/any-percent-published/scenario.json --frame-start 7968 --frame-end 7969 --include-advisory --include-physics-trace --show-object-roster-on-match`
  Result: frame `7968` now reports `kid_flip_follow_on_probe = branch=c_pressed_fallthrough/motion=normal_physics/post=none/entry[x_vel=0x00000000/y_vel=0x00000000/ground=1/wall_cling=none]/press_away[...] /jump_gravity[...] /floor[...] /wall[...]`, while oracle `kid_gfx` already carries `y_velocity=0xffffc000`.
- `mise run audit-debug-fields`
  Result: clean after registering `kid_flip_follow_on_probe`.
- `mise run verify-slice-fast`
  Result: passes.

## Exact Next Moves

1. Inspect why the native shared jump/gravity stage still stays at `grounded_hold` on frame `7968`, concentrating on the setup immediately before and inside `apply_jump_and_gravity_vel_only(...)` rather than revisiting branch-selection, wall-cling, or post-motion systems.
2. Cross-check the shared `loc_AA22` setup against the disassembly for any omitted state write or precondition equivalent to the ROM’s jump-movement / platform-clear handoff before `sub_B168`.
3. Do not implement a gameplay fix yet unless that narrower jump/gravity inspection exposes the concrete missing native state write or gate.

## Verification To Re-run

- `cargo test -p kid-core trace_frame_reports_kid_flip_branch_probe -- --nocapture`
- `cargo test -p kid-core trace_frame_reports_kid_flip_follow_on_probe -- --nocapture`
- `cargo test -p kid-core published_tas_frames_7968_7969_report_kid_flip_branch_probe -- --nocapture`
- `cargo test -p kid-core published_tas_frames_7968_7969_report_kid_flip_follow_on_probe -- --nocapture`
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
- Do not rely on the broad `cargo test -p kid-core trace_frame_reports_kid_ -- --nocapture` filter as a frontier signal; it still trips the unrelated existing `trace_frame_reports_kid_object_support_candidates` assertion.
