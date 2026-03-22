---
slug: any-percent-published-frame-7969-knights-isle-native-stays-grounded-while-oracle-lifts-off
cycle: 16
updated_at: 2026-03-22T23:05:04Z
---

# Source-Port Frontier Handoff

## What Changed

- Added `kid_collision_handoff_probe`, a native frame probe captured before any pre-physics consumption in `refresh_opening_slice`.
- The new probe records the carried gameplay lifecycle, the prior frame's first post-motion mutator, and any queued stomp/form/level-skip or player jump-buffer/wall-cling state.
- Tightened trace coverage with `trace_frame_reports_kid_collision_handoff_probe` and `published_tas_frames_7968_7969_report_kid_collision_handoff_probe`.
- Re-ran the focused frame-`7968..7969` TAS diff and confirmed native enters frame `7968` with `prev_post=none`, `pending_stomp=0`, `pending_form=none`, `pending_level_skip=none`, `ground=1`, `jump_buffer=0`, and `wall_cling=none`.

## Current Model

- The active frontier is still frame `7969` in `any-percent-published`, clean through `7968`.
- Native now rules out both obvious special-motion latches and the broader pre-motion carry-style handoff. There is no prior native post-motion consumer feeding the next frame, and no queued stomp, transform, level skip, jump buffer, or wall-cling state waiting to be consumed before physics.
- Inference from the ASM plus the new native probe: the missing lift-off is more likely inside the live movement-state path itself, specifically the state that reaches `Character_CheckCollision -> sub_B084 -> sub_B168` in `kid.asm`, rather than a deferred carry bag that native forgot to apply at frame start.

## Strongest Evidence

- `cargo test -p kid-core trace_frame_reports_kid_collision_handoff_probe -- --nocapture`
  Result: passes.
- `cargo test -p kid-core published_tas_frames_7968_7969_report_kid_collision_handoff_probe -- --nocapture`
  Result: passes; both frames now require `lifecycle=alive/prev_post=none/pending_stomp=0/pending_form=none/pending_level_skip=none/ground=1/jump_buffer=0/wall_cling=none`.
- `cargo run -p kidcli -- tas-diff --scenario .cache/tas-imports/any-percent-published/scenario.json --frame-start 7968 --frame-end 7969 --include-advisory --include-physics-trace --show-object-roster-on-match`
  Result: native frame `7968` prints the new empty carry-state probe while oracle Kid gfx already has `y_velocity=0xFFFFC000`; native still reports `kid_grounded_release_probe ... velocity=grounded_hold ... support=solid@58,28`.
- `.cache/upstream/kid-chameleon-disasm/kid.asm`
  Result: the active jump/walk loop at `loc_A73A` runs `sub_7ACC`, then `Character_CheckCollision`, then `sub_B084`, then `sub_B168`, then `sub_A4EE`; the wall-cling variant at `loc_AA22` also runs `Character_CheckCollision` immediately before `sub_B168`. This reinforces that the next probe should target the live movement-state path, not another queued native carry flag.
- `mise run audit-debug-fields`
  Result: clean.
- `mise run verify-slice-fast`
  Result: passes.

## Exact Next Moves

1. Inspect the ASM movement-state path that reaches `loc_A73A -> loc_A74E` (and `loc_AA22` only if the frame window proves wall-cling relevant) to identify what state transition can give oracle Kid upward `y_velocity` before or around `sub_B168`.
2. Instrument the native equivalent of that live movement-state gate around frames `7968..7969`; do not add another generic carry-state probe unless the new trace points to one.
3. Once that state transition is proven, add the tight regression around frames `7968..7969` and land the gameplay fix.

## Verification To Re-run

- `cargo test -p kid-core trace_frame_reports_kid_collision_handoff_probe -- --nocapture`
- `cargo test -p kid-core published_tas_frames_7968_7969_report_kid_collision_handoff_probe -- --nocapture`
- `cargo test -p kid-core trace_frame_reports_kid_grounded_release_probe -- --nocapture`
- `cargo test -p kid-core published_tas_frames_7968_7969_report_kid_grounded_release_probe -- --nocapture`
- `cargo run -p kidcli -- tas-diff --scenario .cache/tas-imports/any-percent-published/scenario.json --frame-start 7968 --frame-end 7969 --include-advisory --include-physics-trace --show-object-roster-on-match`
- `mise run audit-debug-fields`
- `mise run verify-slice-fast`

## Dead Ends / Do Not Retry

- Do not retry terrain-write theories for cell `58,28`; native still proves the support tile matches the generated layout and the grounded-release scan still sees solid support.
- Do not retry post-motion platform/lift/support handoff theories for this window; `post=none` is still stable in the grounded-release probe.
- Do not retry native goal-touch timing; the goal probe still shows `lifecycle=alive` and `touch=0`.
- Do not retry the obvious special-motion latch theories; `kid_special_motion_probe` still keeps every explicit latch idle.
- Do not retry generic native jump/gravity-gate theories without new evidence; the grounded-release probe still shows `velocity=grounded_hold`.
- Do not add another broad native carry-state probe for this frame window unless the movement-state inspection points back to one; `kid_collision_handoff_probe` is empty on the live frontier frames.
