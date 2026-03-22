---
slug: any-percent-published-frame-7969-knights-isle-native-stays-grounded-while-oracle-lifts-off
cycle: 15
updated_at: 2026-03-22T22:56:23Z
---

# Source-Port Frontier Handoff

## What Changed

- Extended `kid_grounded_release_probe` so it now reports the native pre-physics velocity gate chosen after the grounded-release scan: `jump`, `jump_buffer`, `velocity`, `custom_gravity`, and `wall_cling`.
- Tightened `trace_frame_reports_kid_grounded_release_probe` and `published_tas_frames_7968_7969_report_kid_grounded_release_probe` to require that new branch summary.
- Re-ran the frame-`7968..7969` TAS diff and confirmed native enters the window with `velocity=grounded_hold`, `jump=0`, `jump_buffer=0`, `custom_gravity=0`, and `wall_cling=none`.

## Current Model

- The active frontier is still frame `7969` in `any-percent-published`, clean through `7968`.
- Native is not merely missing a terrain mutation or a native jump input. On frame `7968`, the existing grounded-release scan still sees pristine generated solid support at cell `58,28`, and the newly extended probe now proves the immediate native velocity path after that scan is the inert grounded branch: `velocity=grounded_hold`.
- That means native has no player-side lift-off queued through the current `apply_jump_and_gravity_vel_only` gate. The missing behaviour is now more likely a pre-movement state handoff that should have changed Kid's state before the grounded-hold branch runs. Inference from the ASM: the next place to inspect is the `Character_CheckCollision` / pre-motion carry state ahead of `sub_B168`, not another terrain scan or obvious jump/special latch.

## Strongest Evidence

- `cargo test -p kid-core trace_frame_reports_kid_grounded_release_probe -- --nocapture`
  Result: passes.
- `cargo test -p kid-core published_tas_frames_7968_7969_report_kid_grounded_release_probe -- --nocapture`
  Result: passes; both frames now require `jump=0/jump_buffer=0/velocity=grounded_hold/custom_gravity=0/wall_cling=none`.
- `cargo run -p kidcli -- tas-diff --scenario .cache/tas-imports/any-percent-published/scenario.json --frame-start 7968 --frame-end 7969 --include-advisory --include-physics-trace --show-object-roster-on-match`
  Result: frame `7968` still prints `support=solid@58,28 ... same=1`, and now also prints `jump=0/jump_buffer=0/velocity=grounded_hold/custom_gravity=0/wall_cling=none` while oracle `kid_gfx` already has upward `y_velocity=0xFFFFC000`.
- `.cache/upstream/kid-chameleon-disasm/kid.asm`
  Result: the live gameplay loop calls `Character_CheckCollision`, then `sub_B084`, then `sub_B168`, then `sub_902A`; `sub_B168` itself only injects lift-off through jump/custom-gravity paths. This supports the new model that the missing handoff is earlier than native's grounded-hold branch.
- `mise run verify-slice-fast`
  Result: passes.

## Exact Next Moves

1. Trace the native pre-motion collision/state handoff that runs before the grounded-hold `sub_B168` equivalent, starting with whatever should correspond to ASM `Character_CheckCollision` or other pending Kid/object carry state in the frame-`7968..7969` window.
2. Compare oracle `kid_collision_source` / `kid_collision_type` carry state against native pending collision-style state around frames `7967..7969`; the new evidence says the lift-off is not coming from native's current jump/gravity gate.
3. Once that earlier handoff is proven, add the tight regression around frames `7968..7969` and land the gameplay fix.

## Verification To Re-run

- `cargo test -p kid-core trace_frame_reports_kid_grounded_release_probe -- --nocapture`
- `cargo test -p kid-core published_tas_frames_7968_7969_report_kid_grounded_release_probe -- --nocapture`
- `cargo run -p kidcli -- tas-diff --scenario .cache/tas-imports/any-percent-published/scenario.json --frame-start 7968 --frame-end 7969 --include-advisory --include-physics-trace --show-object-roster-on-match`
- `mise run verify-slice-fast`

## Dead Ends / Do Not Retry

- Do not spend another probe proving terrain writes changed cell `58,28`; native still proves the live support tile matches generated layout bytes `0x60 0x82`.
- Do not retry post-motion platform/lift/support handoff theories for this window; `kid_grounded_release_probe` still shows `post=none`.
- Do not retry native goal-touch timing; `kid_goal_probe` still shows `lifecycle=alive` and `touch=0`.
- Do not retry the obvious special-motion latch theories; `kid_special_motion_probe` still keeps every explicit special-motion latch idle.
- Do not retry the generic native jump/gravity gate without new evidence; the extended grounded-release probe now proves the live path is `velocity=grounded_hold` with no jump, buffered jump, custom gravity, or wall-cling release queued.
