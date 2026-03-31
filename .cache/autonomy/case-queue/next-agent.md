---
slug: death_respawn_level1_case_00000_00060
cycle: 13
updated_at: 2026-03-31T22:52:40Z
---

## What Changed

- Ran `cargo run -p kidcli -- autonomous-loop --goal case-queue`; cycle 13 selected `death_respawn_level1_case_00000_00060` again via selector `death-respawn-level1-case`.
- Re-verified the selected case before editing and confirmed the live failure was frame 1 `player.hitpoints` (`native 2`, `oracle 0`).
- Read the ROM boot/init path in `.cache/upstream/kid-chameleon-disasm/kid.asm` before patching: `loc_65C` seeds `Player_1_*` / `Player_2_*`, `sub_8A4` copies the active player slot into live `Number_*`, and `loc_12EDE` is the later `move.w #4,(Game_Mode).w` transition.
- Patched `crates/kid-core/src/trace.rs` so trace output keeps `number_lives`, `number_hitpoints`, and `current_helmet_id` cold only for the oracle-backed early Sega-screen window instead of leaking progression immediately or masking the whole mode-0 phase.
- Added kid-core regressions covering both sides of that boundary: the second Sega-screen frame stays cold, and frame 9 already exposes live progression while `game_mode` is still `0`.
- Re-ran the selected case. The early semantic drift is gone; the first failure moved to frame 33 `world.game_mode` (`native 0`, `oracle 4`).

## Selected Case

- Case ID: `death_respawn_level1_case_00000_00060`
- Selector: `death-respawn-level1-case`
- Autonomy gate: `patch-allowed`
- Current verify result: `FAIL`
- Current first failure: frame `33`, `world.game_mode` (`native 0`, `oracle 4`)

## Why This Case

- Cycle 13 kept this case as the top autonomy-executable item.
- The stale early boot semantic mismatch was small, local, and backed by both oracle data and ROM init routines, so it was worth fixing before widening into later boot/start timing.
- After the fix, the remaining failure is no longer HUD-state noise; it is a cleaner mode-transition boundary.

## Strongest Evidence

- `cargo run -p kidcli -- case verify --case death-respawn-level1-case` now writes `.cache/cases/death-respawn-level1-case/verify.json` with first failure frame `33` on `world.game_mode` (`native 0`, `oracle 4`).
- `.cache/cases/death-respawn-level1-case/native-semantic.json` and `oracle-semantic.json` now agree on the early counter boundary: frames `0..8` keep `number_lives=0` / `hitpoints=0`, then frame `9` flips to `number_lives=3` / `hitpoints=2` while `game_mode` remains `0`.
- `.cache/upstream/kid-chameleon-disasm/kid.asm:807` / `:810` (`loc_65C`) seed `Player_1_Lives` and `Player_1_Hitpoints`, `.cache/upstream/kid-chameleon-disasm/kid.asm:964` (`sub_8A4`) copies the active player slot into live `Number_Lives` onward, and `.cache/upstream/kid-chameleon-disasm/kid.asm:25546` (`loc_12EDE`) performs `move.w #4,(Game_Mode).w`.
- Focused verification passed: `first_boot_frame_keeps_gameplay_state_cold`, `sega_screen_frames_keep_gameplay_state_cold_until_title_transition`, `sega_screen_bootstrap_exposes_live_progression_by_frame_nine`, `early_title_frames_stay_inert_until_boot_ready`, and `gameplay_state_from_native_trace_zeroes_hidden_player_motion`.

## Exact Next Moves

1. Re-run `cargo run -p kidcli -- case verify --case death-respawn-level1-case` and confirm the first failure is still frame 33 `world.game_mode`.
2. Read the ROM path around `.cache/upstream/kid-chameleon-disasm/kid.asm:25546` (`loc_12EDE`) and the preceding Sega logo loop to map exactly when the case’s input should trigger the `Game_Mode = 4` jump.
3. Compare native trace around frames `30..35` against oracle output to decide whether the remaining bug is in boot/start timing or in how the native boot state maps Start presses onto the deferred mode switch.
4. Keep work on `death_respawn_level1_case_00000_00060` only until the frame-33 mode drift is removed or moved later with equally specific evidence.

## Verification To Re-run

- `cargo test -p kid-core first_boot_frame_keeps_gameplay_state_cold -- --nocapture`
- `cargo test -p kid-core sega_screen_frames_keep_gameplay_state_cold_until_title_transition -- --nocapture`
- `cargo test -p kid-core sega_screen_bootstrap_exposes_live_progression_by_frame_nine -- --nocapture`
- `cargo test -p kid-core early_title_frames_stay_inert_until_boot_ready -- --nocapture`
- `cargo test -p kidcli gameplay_state_from_native_trace_zeroes_hidden_player_motion -- --nocapture`
- `cargo run -p kidcli -- case verify --case death-respawn-level1-case`
- `cargo run -p kidcli -- case summarise --case death-respawn-level1-case`

## Dead Ends / Do Not Retry

- Do not retry the old “mask the whole Sega-screen phase” trace fix. Oracle counters become live on frame 9 even though `game_mode` is still `0`.
- Do not go back to the stale frame-1 `player.hitpoints` report. That mismatch is fixed.
- Do not widen into other reduced/open cases. The loop still requires this selected case only.
