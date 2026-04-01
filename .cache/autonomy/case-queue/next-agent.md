---
slug: death_respawn_level1_case_00000_00200
cycle: 44
updated_at: 2026-04-01T02:21:10Z
---

## What Changed

- Ran `cargo run -p kidcli -- autonomous-loop --goal case-queue`; cycle 44 selected `death_respawn_level1_case_00000_00200` again (`death-respawn-derived-check`).
- Re-verified the selected case before editing; it still failed first on frame 161 at `player.x_pos`.
- Re-ran the `150..170` gameplay window compare and confirmed the exact oracle timing: title-menu cursor pose visible through frame 160, hidden on frame 161, title shell counts `4/3` through frame 165, `0/0` on frames 166..167, then `2/0` from frame 168.
- Read the ROM path in `.cache/upstream/kid-chameleon-disasm/kid.asm` before changing gameplay code:
  - `Title_InputLoop` writes `Game_Mode=$0008`
  - `loc_6E2` clears and rebuilds runtime state
  - `Load_InGame` remains the later visible hand-off
- Split the native early-start trace timing in `crates/kid-core/src/trace.rs`:
  - title cursor pose carry
  - title shell object-count carry
  - later hidden preload object-count window
- Added a regression test in `crates/kid-core/src/tests.rs` covering frames 160, 161, 165, 166, and 168 for the death-respawn Level 1 script.
- Refreshed the case summary after verification; the selected case now passes.

## Selected Case

- Case ID: `death_respawn_level1_case_00000_00200`
- Selector: `death-respawn-derived-check`
- Autonomy gate at selection time: `evidence-only`
- Current verify result: `PASS`

## Why This Case

- The selector chose the same case in cycle 44, so the session stayed on that one case only.
- The previous handoff had already narrowed the real bug: native treated the title-menu cursor pose and title shell counts as one shared carry window during early mode `$0008`.
- The re-run compare window made the fix concrete enough to patch without widening scope.

## Strongest Evidence

- `cargo run -p kidcli -- case verify --case death-respawn-derived-check`
  - before patch: `FAIL` at frame 161, `player.x_pos`
  - after patch: `PASS`
- `cargo run -p kidcli -- compare-native-gameplay-window --scenario death-respawn-level1 --frame-start 150 --frame-end 170 --checkpoint-interval 1`
  - before patch: diverged at frame 161 (`kid_x_pos`, `kid_y_pos`)
  - after patch: comparison passed for frames `150..=170`
- Regression tests:
  - `cargo test -p kid-core death_respawn_level_1_clears_title_cursor_pose_before_title_shell_counts`
  - `cargo test -p kid-core death_respawn_level_1_exposes_title_menu_cursor_on_frame_136`

## Exact Next Moves

1. Start the next session by running `cargo run -p kidcli -- autonomous-loop --goal case-queue` again.
2. Take only the newly selected case from the refreshed task package.
3. Re-verify that selected case before touching code.
4. Read the relevant ASM labels before any gameplay patch, even if the next case looks adjacent.

## Verification To Re-run

- `cargo run -p kidcli -- case verify --case death-respawn-derived-check`
- `cargo run -p kidcli -- case summarise --case death-respawn-derived-check`
- `cargo run -p kidcli -- compare-native-gameplay-window --scenario death-respawn-level1 --frame-start 150 --frame-end 170 --checkpoint-interval 1`
- `cargo test -p kid-core death_respawn_level_1_clears_title_cursor_pose_before_title_shell_counts`
- `cargo test -p kid-core death_respawn_level_1_exposes_title_menu_cursor_on_frame_136`

## Dead Ends / Do Not Retry

- Do not revert to one shared early-start title carry window. Oracle clearly splits pose visibility from shell-object visibility.
- Do not move this fix into generic player physics or collision; the resolved bug was trace timing during `Title_InputLoop -> loc_6E2`.
- Do not assume `Load_InGame` explains frames 161..168. The winning fix stayed in the pre-`Load_InGame` mode-8 bootstrap schedule.
