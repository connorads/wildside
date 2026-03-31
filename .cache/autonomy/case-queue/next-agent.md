---
slug: death_respawn_level1_case_00200_00300
cycle: 17
updated_at: 2026-03-31T23:27:03Z
---

## What Changed

- Ran `cargo run -p kidcli -- autonomous-loop --goal case-queue`; cycle 17 selected `death_respawn_level1_case_00200_00300`.
- Re-ran `cargo run -p kidcli -- case verify --case death-respawn-level1-200-300`; the stale frame-200 camera/player drift refreshed to a `world.game_mode` mismatch.
- Read the relevant ROM routines before patching: `Load_TitleCard` plus `loc_1AB26`, `loc_1AC70`, `loc_1AD48`, and `loc_1AE2E` in `.cache/upstream/kid-chameleon-disasm/kid.asm`.
- Patched the native title-card bootstrap so real title-card entries keep the hidden shell alive through frame 200, keep the hidden start-level shell through frame 520, and only expose visible gameplay on frame 521.
- Added focused regressions for the frame-200 hidden bootstrap and the frame-520/frame-521 hand-off.
- Re-verified the selected case and refreshed the summary; the first semantic failure moved later to frame 222 on `player.x_pos`.

## Selected Case

- Case ID: `death_respawn_level1_case_00200_00300`
- Selector: `death-respawn-level1-200-300`
- Autonomy gate: `patch-allowed`
- Current verify result: `FAIL`
- Current first failure: frame 222, `player.x_pos` (`native=4292870144`, `oracle=0`)

## Why This Case

- The Ralph case-queue loop selected this case in cycle 17, so it remained the only permitted focus.
- Fresh verification showed the cached case bundle was stale; the first mismatch was no longer the older frame-200 camera/player drift.
- The ROM title-card path explained why native was exposing gameplay too early: the hidden bootstrap and later visible hand-off are distinct phases, and the case could be narrowed by fixing only that bootstrap timing first.

## Strongest Evidence

- `cargo run -p kidcli -- compare-native-gameplay-window --scenario death-respawn-level1 --frame-start 200 --frame-end 300` originally diverged at frame 200 on hidden bootstrap fields; after the patch it now reports object-count drift at frame 200 and the first live drift later at frame 222.
- `cargo run -p kidcli -- case verify --case death-respawn-level1-200-300` now reports `FAIL`, but the bundle’s `first_failure` moved to frame 222 on `player.x_pos`.
- `.cache/cases/death-respawn-level1-200-300/verify.json` now records:
  `frame_index: 222`, `field: "player.x_pos"`, `native_value: "4292870144"`, `oracle_value: "0"`.
- The new regressions `death_respawn_level_1_keeps_bootstrap_hidden_on_frame_200` and `boot_title_start_stays_hidden_through_frame_520_then_enters_ingame` pass.

## Exact Next Moves

1. Re-run `cargo run -p kidcli -- compare-native-gameplay-window --scenario death-respawn-level1 --frame-start 200 --frame-end 300` and inspect the frame-222 artefacts in `.cache/native-gameplay-oracle-runs/death_respawn_level1_window_00200_00300_step_1`.
2. Read the next relevant ROM path for the hidden start-level shell after `Load_TitleCard`: keep tracing how the title-card shell hands off object counts and Kid visibility between frame 200 and frame 222.
3. Patch only the frame-222 `player.x_pos` frontier, then re-run `cargo run -p kidcli -- case verify --case death-respawn-level1-200-300` and `cargo run -p kidcli -- case summarise --case death-respawn-level1-200-300`.

## Verification To Re-run

- `cargo test -p kid-core death_respawn_level_1_keeps_bootstrap_hidden_on_frame_200 -- --nocapture`
- `cargo test -p kid-core boot_title_start_stays_hidden_through_frame_520_then_enters_ingame -- --nocapture`
- `cargo run -p kidcli -- case verify --case death-respawn-level1-200-300`
- `cargo run -p kidcli -- case summarise --case death-respawn-level1-200-300`

## Dead Ends / Do Not Retry

- Do not reuse the stale case summary that claimed frame 200 `player.x_pos` or `camera.y_pos`; after re-verification that evidence was outdated.
- Do not patch the hidden bootstrap by exposing visible gameplay earlier; the ROM title-card shell still keeps Kid/camera hidden while gameplay mode is already live.
- Do not treat the synthetic no-title-card boot tests as authoritative for this case; the real generated-assets title-card lineage has different timing and object-shell behaviour.
