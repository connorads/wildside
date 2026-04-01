---
slug: death_respawn_level1_case_00000_00200
cycle: 54
updated_at: 2026-04-01T02:55:01Z
---

## What Changed

- Re-ran `cargo run -p kidcli -- autonomous-loop --goal case-queue`; cycle `54` selected `death_respawn_level1_case_00000_00200` again and kept it evidence-only.
- Re-verified the selected case with `cargo run -p kidcli -- case verify --case death-respawn-test`; it still fails first on frame `172` at `player.x_pos`.
- Refreshed the reduced-case artefacts with `cargo run -p kidcli -- case summarise --case death-respawn-test`.
- Re-read the ROM path in `.cache/upstream/kid-chameleon-disasm/kid.asm`: `Title_InputLoop`, `loc_6E2`, `Load_TitleCard`, `loc_1AB26`, `loc_1AC70`, `loc_1ACFE`, `loc_1AE2E`, and `Load_InGame`.
- Left gameplay code untouched. The task package still marks this case evidence-first, and the ASM read does not yet justify a bootstrap timing patch.

## Selected Case

- Case ID: `death_respawn_level1_case_00000_00200`
- Selector: `death-respawn-test`
- Cycle: `54`
- Current result: `FAIL`
- Current first failure: frame `172`, field `player.x_pos`, native `0`, oracle `19398656`

## Why This Case

- The autonomous loop selected the same case again, so the session stayed on this case only.
- The task package still has no implementation targets and still classifies the work as evidence-only, so the safe scope was: re-verify, refresh the reduced artefacts, and tighten the ASM-backed constraint on the hidden mode-`8` bootstrap.
- The key unresolved question is not generic physics or collision. It is which ROM lineage first exposes the hidden frame-`172` pose at `(x=$0128_0000, y=$0100_0000)` while `game_mode=$0008` and camera remain zero.

## Strongest Evidence

- `cargo run -p kidcli -- autonomous-loop --goal case-queue`
  - cycle `54`
  - status `stalled`
  - selected case still `death_respawn_level1_case_00000_00200`
- `cargo run -p kidcli -- case verify --case death-respawn-test`
  - still `FAIL`
  - first bad frame/field unchanged: frame `172`, `player.x_pos`
- `.cache/cases/death-respawn-test/oracle-semantic.json`
  - frames `166..171`: `player=(0,0)`, `camera=(0,0)`, `game_mode=8`
  - frame `172`: `player=(0x0128_0000,0x0100_0000)`, `camera=(0,0)`, `game_mode=8`
- `.cache/cases/death-respawn-test/native-semantic.json`
  - frames `166..172`: `player=(0,0)`, `camera=(0,0)`, `game_mode=8`
  - implication: native still withholds the first hidden bootstrap pose one frame too long
- `.cache/upstream/kid-chameleon-disasm/kid.asm`
  - `Title_InputLoop` writes `Game_Mode=$0008`, sets `($FFFFFBCE)`, and jumps straight to `loc_6E2`
  - `Load_TitleCard` is a separate loader that allocates `loc_1AB26` and `loc_1AE2E`
  - `loc_1AB26` can allocate `loc_1AC70`, and `loc_1ACFE` writes the title-card letter worker positions including fixed `y=$0100` cases
  - implication: the frame-`172` pose numerically resembles the title-card worker path, but the early title-start lineage does not obviously pass through `Load_TitleCard`; treat that resemblance as a clue, not proof

## Exact Next Moves

1. Stay on `death_respawn_level1_case_00000_00200` only; do not widen into later respawn windows or neighbouring case IDs.
2. Trace the exact `Game_Mode=$0008` dispatch after `Title_InputLoop -> loc_6E2` and prove which loader/object lineage can emit the first non-zero player pose on frame `172`.
3. If the frame-`172` pose really comes from title-card-style worker coordinates, identify the call path or shared setup that explains how the early-start lineage reaches those values without first going through visible title-card flow.
4. Before any gameplay patch, map native local elapsed frames to oracle frames `161..172` with an ASM-backed explanation for when the hidden player shell should first exist.
5. Re-run `cargo run -p kidcli -- case verify --case death-respawn-test` after any evidence or gameplay change and stop if the first bad frame moves earlier.

## Verification To Re-run

- `cargo run -p kidcli -- autonomous-loop --goal case-queue`
- `cargo run -p kidcli -- case verify --case death-respawn-test`
- `cargo run -p kidcli -- case summarise --case death-respawn-test`

## Dead Ends / Do Not Retry

- Do not pass the case ID `death_respawn_level1_case_00000_00200` directly to `kidcli case verify`; the command needs the selector bundle `death-respawn-test`.
- Do not widen into `pause_resume_level1_case_00000_00200` or `death_respawn_level1_case_00300_00500`; this session is only about `death_respawn_level1_case_00000_00200`.
- Do not patch `EARLY_START_LOAD_INGAME_VISIBLE_FRAME` or related bootstrap constants from the frame-`172` symptom alone. The ASM still does not explain the hidden pose source.
- Do not assume the frame-`172` `(x=$0128_0000, y=$0100_0000)` pose proves the path already ran `Load_TitleCard`; `Title_InputLoop` jumps straight to `loc_6E2`, so that link still needs proof.
