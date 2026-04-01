---
slug: death_respawn_level1_case_00000_00200
cycle: 55
updated_at: 2026-04-01T03:03:30Z
---

## What Changed

- Re-ran `cargo run -p kidcli -- autonomous-loop --goal case-queue`; cycle `55` selected `death_respawn_level1_case_00000_00200` again and kept it evidence-only.
- Re-verified the selected case with `cargo run -p kidcli -- case verify --case death-respawn-test`; it still fails first on frame `172` at `player.x_pos`.
- Refreshed the reduced-case artefacts with `cargo run -p kidcli -- case summarise --case death-respawn-test`.
- Re-read the ROM path in `.cache/upstream/kid-chameleon-disasm/kid.asm`: `Title_InputLoop`, `loc_6E2`, `Load_TitleCard`, `loc_1AB26`, `loc_1AC70`, `loc_1ACFE`, `loc_1AE2E`, and `Load_InGame`.
- Left gameplay code untouched. The selected task package still has no implementation targets, and the ASM still does not prove which hidden mode-`8` lineage writes the first non-zero Level 1 pose on frame `172`.

## Selected Case

- Case ID: `death_respawn_level1_case_00000_00200`
- Selector: `death-respawn-test`
- Cycle: `55`
- Current result: `FAIL`
- Current first failure: frame `172`, field `player.x_pos`, native `0`, oracle `19398656`

## Why This Case

- The autonomous loop selected the same case again, so the session stayed on this case only.
- The task package still marks the case `evidence-only`, so the safe scope was: re-verify, refresh the reduced artefacts, and tighten the ASM-backed explanation for the hidden frame-`172` pose.
- The refined question is narrower than generic `Load_InGame` timing. Oracle keeps `game_mode=$0008`, camera zero, and empty entity/event summaries through frame `171`, then exposes only the player pose on frame `172`.

## Strongest Evidence

- `cargo run -p kidcli -- autonomous-loop --goal case-queue`
  - cycle `55`
  - status `stalled`
  - selected case still `death_respawn_level1_case_00000_00200`
- `cargo run -p kidcli -- case verify --case death-respawn-test`
  - still `FAIL`
  - first bad frame/field unchanged: frame `172`, `player.x_pos`
- `.cache/cases/death-respawn-test/oracle-semantic.json`
  - frames `161..171`: `player=(0,0)`, `camera=(0,0)`, `game_mode=8`, `entities=[]`, `events=[]`
  - frame `172`: `player=(0x0128_0000,0x0100_0000)`, `camera=(0,0)`, `game_mode=8`, `entities=[]`, `events=[]`
- `.cache/cases/death-respawn-test/native-semantic.json`
  - frames `161..172`: `player=(0,0)`, `camera=(0,0)`, `game_mode=8`, `entities=[]`, `events=[]`
  - implication: native still withholds the first hidden player shell one frame too long even though the semantic shell is otherwise still empty
- `.cache/upstream/kid-chameleon-disasm/kid.asm`
  - `Title_InputLoop` writes `Game_Mode=$0008`, sets `($FFFFFBCE)`, and jumps straight to `loc_6E2`
  - `Load_TitleCard` separately allocates `loc_1AB26` and `loc_1AE2E`
  - `loc_1ACFE` can place title-card workers at fixed `y=$0100` destinations, but the early title-start lineage still does not obviously pass through `Load_TitleCard`
  - implication: the frame-`172` `(x=$0128_0000, y=$0100_0000)` pose still resembles a title-card worker destination more than the later `Load_InGame` settle path, but the call path remains unproven

## Exact Next Moves

1. Stay on `death_respawn_level1_case_00000_00200` only; do not widen into other case IDs or later respawn windows.
2. Trace the exact mode-`8` dispatch after `Title_InputLoop -> loc_6E2` and prove which loader or object lineage can expose only `player.x/y` while camera and semantic shell stay empty.
3. Check whether the frame-`172` `(x=$0128_0000, y=$0100_0000)` pose comes from a reused title-card worker destination, a transient WRAM write during `Load_InGame`, or another hidden object path.
4. Before any gameplay patch, map native local elapsed frames onto oracle frames `166..172` with an ASM-backed explanation for why the first hidden player pose should appear exactly there.
5. Re-run `cargo run -p kidcli -- case verify --case death-respawn-test` after any evidence or gameplay change and stop if the first bad frame moves earlier.

## Verification To Re-run

- `cargo run -p kidcli -- autonomous-loop --goal case-queue`
- `cargo run -p kidcli -- case verify --case death-respawn-test`
- `cargo run -p kidcli -- case summarise --case death-respawn-test`

## Dead Ends / Do Not Retry

- Do not pass the case ID `death_respawn_level1_case_00000_00200` directly to `kidcli case verify`; the command needs the selector bundle `death-respawn-test`.
- Do not widen into `pause_resume_level1_case_00000_00200` or `death_respawn_level1_case_00300_00500`; this session is only about `death_respawn_level1_case_00000_00200`.
- Do not patch `EARLY_START_LOAD_INGAME_VISIBLE_FRAME` or related bootstrap constants from the frame-`172` symptom alone. The ASM still does not explain the hidden pose source.
- Do not assume the frame-`172` `(x=$0128_0000, y=$0100_0000)` pose proves the path ran visible `Load_TitleCard`; `Title_InputLoop` still jumps straight to `loc_6E2`, so that resemblance remains a clue, not proof.
