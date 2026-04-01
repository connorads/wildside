---
slug: death_respawn_level1_case_00000_00200
cycle: 43
updated_at: 2026-04-01T02:15:07Z
---

## What Changed

- Ran `cargo run -p kidcli -- autonomous-loop --goal case-queue`; cycle 43 selected the same case `death_respawn_level1_case_00000_00200` (`death-respawn-derived-check`).
- Re-verified the selected case before any code change: `cargo run -p kidcli -- case verify --case death-respawn-derived-check` still fails first on frame 161 at `player.x_pos`.
- Refreshed the case summary with `cargo run -p kidcli -- case summarise --case death-respawn-derived-check`.
- Re-ran `cargo run -p kidcli -- compare-native-gameplay-window --scenario death-respawn-level1 --frame-start 150 --frame-end 170 --checkpoint-interval 1` and inspected the emitted checkpoints under `.cache/native-gameplay-oracle-runs/death_respawn_level1_window_00150_00170_step_1/`.
- Re-read the ROM bootstrap path before considering a fix:
  - `.cache/upstream/kid-chameleon-disasm/kid.asm`: `Title_InputLoop`, `loc_6E2`, `Load_InGame`
  - confirmed `Title_InputLoop` writes `Game_Mode=$0008` then jumps to `loc_6E2`, while `Load_InGame` later rebuilds the in-level shell from `PlayerStart_X_pos` / `PlayerStart_Y_pos`.
- No gameplay code changed this cycle. The new result is a tighter explanation of the failure boundary.

## Selected Case

- Case ID: `death_respawn_level1_case_00000_00200`
- Selector: `death-respawn-derived-check`
- Autonomy gate: `evidence-only`
- Current verify result: `FAIL`

## Why This Case

- The autonomous loop selected the same case again, so the session stayed on that case only.
- The previous handoff assumed a single title-cursor carry window. The refreshed frame window shows the carry is split:
  - oracle clears the traced Kid pose on frame 161
  - oracle still keeps the title-shell object counts (`4` objects / `3` gfx objects) after that
- That means a one-line carry toggle would be wrong. The selected case is now specifically about separating stale player WRAM visibility from the longer-lived title shell during the early `Game_Mode=$0008` bootstrap.

## Strongest Evidence

- `cargo run -p kidcli -- case verify --case death-respawn-derived-check`
  - result: `case death_respawn_level1_case_00000_00200: FAIL`
  - first failure: frame 161, `player.x_pos`, native `2621440`, oracle `0`
- `.cache/cases/death-respawn-derived-check/summary.md`
  - result: case remains `frames: 0..162`
  - first bad field still `player.x_pos`
- `cargo run -p kidcli -- compare-native-gameplay-window --scenario death-respawn-level1 --frame-start 150 --frame-end 170 --checkpoint-interval 1`
  - result: divergence still begins on frame 161 before `level_settle@719`
  - native frame 160 matches oracle: `kid_x_pos=0x0028_0000`, `kid_y_pos=0x0030_0000`, `number_objects=4`, `number_gfx_objects=3`
  - oracle frame 161 clears `kid_x_pos` / `kid_y_pos` to `0`, but still reports `number_objects=4`, `number_gfx_objects=3`
  - native frame 161 still exposes `kid_x_pos=0x0028_0000`, `kid_y_pos=0x0030_0000`
  - native frame 162 drops to `number_objects=0`, `number_gfx_objects=0`, while oracle still reports `4` / `3`
- ASM basis:
  - `Title_InputLoop`: `move.w #8,(Game_Mode).w` then `jmp (j_loc_6E2).w`
  - `loc_6E2`: shared mode-change dispatcher that clears/rebuilds runtime state
  - `Load_InGame`: later reseeds level state from map header spawn data, so the frame-161 issue is still pre-`Load_InGame`

## Exact Next Moves

1. Inspect the native early-start trace path in `crates/kid-core/src/trace.rs`, not generic gameplay physics. The selected case now points at how `BootState::StartingLevel` exposes stale title-shell state during mode `$0008`.
2. Split the concepts currently coupled by `early_start_title_menu_cursor_carry_active()`:
   - traced Kid pose visibility
   - title-shell object-count visibility
3. Re-read `Title_InputLoop -> loc_6E2` before patching. The next fix should preserve the longer-lived title shell while clearing traced Kid pose on frame 161.
4. If patching, verify immediately with the selected case first, then the `150..170` compare window to confirm the failure moves later than frame 161 instead of just switching from `player.x_pos` to `entities` on frame 162.

## Verification To Re-run

- `cargo run -p kidcli -- case verify --case death-respawn-derived-check`
- `cargo run -p kidcli -- case summarise --case death-respawn-derived-check`
- `cargo run -p kidcli -- compare-native-gameplay-window --scenario death-respawn-level1 --frame-start 150 --frame-end 170 --checkpoint-interval 1`

## Dead Ends / Do Not Retry

- Do not reopen the already-fixed frame-136 title-menu cursor spawn regression. That part still matches oracle.
- Do not patch generic player physics or collision for this case. The failure remains in the hidden mode-8 bootstrap shell before `Load_InGame` becomes relevant.
- Do not "fix" frame 161 by shortening the shared carry window globally. Oracle still keeps `number_objects=4` / `number_gfx_objects=3` after Kid pose clears, so a single shared cutoff would immediately break the next frame.
- Do not assume `Load_InGame` owns frame 161. The ROM evidence says this is still the earlier `Title_InputLoop -> loc_6E2` lineage.
