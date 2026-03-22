---
slug: any-percent-published-frame-7969-knights-isle-native-stays-grounded-while-oracle-lifts-off
cycle: 14
updated_at: 2026-03-22T22:48:16Z
---

# Source-Port Frontier Handoff

## What Changed

- Extended `kid_grounded_release_probe` so the debug string now compares the live support cell against the generated level-layout baseline for the same `(col,row)`.
- Tightened `published_tas_frames_7968_7969_report_kid_grounded_release_probe` to require that provenance string in the live frame-7968/7969 window.
- Re-ran the narrowed TAS window and confirmed the support tile under Kid is still the generated `solid/None/skin=0x82` terrain cell.

## Current Model

- The active frontier is still frame `7969` in `any-percent-published`, clean through `7968`.
- Native is not standing on a dynamically mutated support tile at cell `58,28`. The grounded-release probe now proves the live cell matches the generated level-layout baseline exactly: `grid=generated/current[solid,None,skin=0x82]/baseline[solid,None,skin=0x82]/same=1`.
- That removes terrain writes as the likely cause for this window. The remaining blocker is an earlier player/support-state handoff or alternate collision/support path that oracle uses before native's simple grounded-release scan runs.

## Strongest Evidence

- `cargo test -p kid-core trace_frame_reports_kid_grounded_release_probe -- --nocapture`
  Result: passes.
- `cargo test -p kid-core published_tas_frames_7968_7969_report_kid_grounded_release_probe -- --nocapture`
  Result: passes; frames `7968` and `7969` now require the provenance suffix.
- `cargo run -p kidcli -- tas-diff --scenario .cache/tas-imports/any-percent-published/scenario.json --frame-start 7968 --frame-end 7969 --include-advisory --include-physics-trace --show-object-roster-on-match`
  Result: frame `7968` prints `kid_grounded_release_probe: ... /terrain=grid=generated/current[solid,None,skin=0x82]/baseline[solid,None,skin=0x82]/same=1` while oracle has already seeded upward `kid_gfx` velocity `0xFFFFC000`.
- `od -An -tx1 -j 9076 -N 2 .cache/generated/28b904000b2863b6760531807760b571f1a5fc1d/level-layout/15.bin`
  Result: `60 82`, matching native's traced `solid/None/skin=0x82` cell at `(58,28)`.
- `mise run verify-slice-fast`
  Result: passes.

## Exact Next Moves

1. Trace the player-side state/collision path that can invalidate support before or instead of the grounded-release terrain scan, now that cell `58,28` is proven to be pristine generated terrain.
2. Focus on pre-scan support-source swaps or state carried by Kid / Kid-gfx / object collision words around frames `7968..7969`; do not spend another probe on block-grid mutation for cell `58,28`.
3. Once that earlier handoff path is proven, add the tight regression around frames `7968..7969` and land the gameplay fix.

## Verification To Re-run

- `cargo test -p kid-core trace_frame_reports_kid_grounded_release_probe -- --nocapture`
- `cargo test -p kid-core published_tas_frames_7968_7969_report_kid_grounded_release_probe -- --nocapture`
- `cargo run -p kidcli -- tas-diff --scenario .cache/tas-imports/any-percent-published/scenario.json --frame-start 7968 --frame-end 7969 --include-advisory --include-physics-trace --show-object-roster-on-match`
- `mise run verify-slice-fast`

## Dead Ends / Do Not Retry

- Do not spend another probe proving terrain writes changed cell `58,28`; native now proves the live support tile still matches generated layout bytes `0x60 0x82`.
- Do not retry post-motion platform/lift/support handoff theories for this window; `kid_grounded_release_probe` still shows `post=none` through the live window.
- Do not retry native goal-touch timing; `kid_goal_probe` still shows `lifecycle=alive` and `touch=0`.
- Do not retry the obvious special-motion latch theories; `kid_special_motion_probe` keeps them idle across the window.
- Do not retry the explicit pre-physics mutator layer in `refresh_opening_slice`; `kid_pre_physics_probe` still shows no native Kid state change before `step_with_order(...)`.
