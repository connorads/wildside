---
slug: death_respawn_level1_case_00000_00060
cycle: 12
updated_at: 2026-03-31T22:45:42Z
---

## What Changed

- Ran `cargo run -p kidcli -- autonomous-loop --goal case-queue`; cycle 12 selected `death_respawn_level1_case_00000_00060` via selector `death-respawn-level1-case`.
- Re-verified the selected case with `cargo run -p kidcli -- case verify --case death-respawn-level1-case`; the live failure was no longer the stale cached `player.x_pos` miss.
- Read the relevant ROM paths before editing: the respawn/teleport path in `.cache/upstream/kid-chameleon-disasm/kid.asm` (`lose_life`, `Teleport`, `loc_74E0`, `loc_7508`) and the boot/title initialisation path that seeds the early hidden frames.
- Patched `crates/kidcli/src/capture.rs` so gameplay-case normalisation no longer mixes zeroed hidden-player positions with missing motion state; hidden native trace frames now emit zero `kid_x_vel`, `kid_y_vel`, and standingstill `character_movement`, matching the oracle’s WRAM-backed gameplay probe.
- Added `gameplay_state_from_native_trace_zeroes_hidden_player_motion` and re-ran the selected case. The first live failure moved from frame 0 hidden motion to frame 1 `player.hitpoints`.

## Selected Case

- Case ID: `death_respawn_level1_case_00000_00060`
- Selector: `death-respawn-level1-case`
- Autonomy gate: `patch-allowed`
- Current verify result: `FAIL`
- Current first failure: frame `1`, `player.hitpoints` (`native 2`, `oracle 0`)

## Why This Case

- Cycle 12 ranked this as the highest-priority autonomy-executable case.
- The task package’s stale summary still claimed frame 0 `player.x_pos`; re-verification showed the real live issue was hidden-frame semantic normalisation.
- The selected case starts in the cold-boot/title setup window (`game_mode = 0`), not the later visible respawn bootstrap, so the next investigation must stay on early boot state rather than widening into other death-respawn windows.

## Strongest Evidence

- `cargo run -p kidcli -- case verify --case death-respawn-level1-case` now reports `case death_respawn_level1_case_00000_00060: FAIL`, with `.cache/cases/death-respawn-level1-case/verify.json` recording the first failure as frame `1` `player.hitpoints` (`native 2`, `oracle 0`).
- `.cache/cases/death-respawn-level1-case/native-semantic.json` now shows frame 0 hidden motion aligned with the oracle: `x_pos=0`, `y_pos=0`, `x_velocity=0`, `y_velocity=0`, `movement_state=movement_0000`.
- `.cache/upstream/kid-chameleon-disasm/kid.asm:7648` enters in-level mode `8`, `.cache/upstream/kid-chameleon-disasm/kid.asm:7674` / `:7675` seed Kid from `PlayerStart_X_pos` / `PlayerStart_Y_pos`, and `.cache/upstream/kid-chameleon-disasm/kid.asm:7708` sets `Character_Movement` to standingstill; that justified the hidden-motion normalisation patch.
- `.cache/upstream/kid-chameleon-disasm/kid.asm:14763` (`lose_life`) does `clr.w (Current_Helmet).w` and `move.w #2,(Number_Hitpoints).w`, but the selected case’s first ten frames stay in `game_mode = 0`, so the remaining frame-1 hitpoint mismatch belongs to the earlier boot/title init window, not the respawn bootstrap.

## Exact Next Moves

1. Re-run `cargo run -p kidcli -- case verify --case death-respawn-level1-case` to confirm the first failure is still frame 1 `player.hitpoints`.
2. Read the early `Game_Mode = 0` boot/title initialisation block in `.cache/upstream/kid-chameleon-disasm/kid.asm` before patching again; do not keep tracing the later respawn bootstrap for this case window.
3. Inspect when the oracle flips `Number_Hitpoints` from `0` to `2` during frames `0..9`, then decide whether the fix belongs in native boot-state timing or gameplay-case normalisation for hidden boot frames.
4. Keep the work on `death_respawn_level1_case_00000_00060` only until this case passes or the first failure moves later with evidence.

## Verification To Re-run

- `cargo test -p kidcli gameplay_state_from_native_trace_zeroes_hidden_player_motion -- --nocapture`
- `cargo run -p kidcli -- case verify --case death-respawn-level1-case`
- `cargo run -p kidcli -- case summarise --case death-respawn-level1-case`

## Dead Ends / Do Not Retry

- Do not trust the stale task-package summary that claimed frame 0 `player.x_pos`; the live case moved after re-verification.
- Do not widen into `death_respawn_level1_case_00200_00300` or other open cases; the loop still requires this selected case only.
- Do not keep reasoning from the respawn bootstrap alone. For this window, frames `0..8` are still `game_mode = 0`, so the next fix must account for cold-boot/title timing.
