---
slug: death_respawn_level1_case_00000_00200
cycle: 63
updated_at: 2026-04-01T03:21:36Z
---

## What Changed

- Re-ran `cargo run -p kidcli -- autonomous-loop --goal case-queue`; cycle `63` selected `death_respawn_level1_case_00000_00200` again and kept it `stalled`.
- Re-verified only the selected case with `cargo run -p kidcli -- case verify --case death-respawn-test`; it still fails on frame `172` at `player.x_pos`.
- Re-ran `cargo run -p kidcli -- case summarise --case death-respawn-test` to confirm the reduced case artefacts are current.
- Re-read the ROM bootstrap path in `.cache/upstream/kid-chameleon-disasm/kid.asm`: `Title_InputLoop`, `loc_6E2`, `Load_InGame`, `loc_11BE4`, `loc_11F48`, and `loc_7526`.
- Left gameplay code untouched. The task package is still evidence-only and the ROM proof for oracle frame `172` `player.x_pos = 0x0128_0000` is still missing.

## Selected Case

- Case ID: `death_respawn_level1_case_00000_00200`
- Selector: `death-respawn-test`
- Cycle: `63`
- Current result: `FAIL`
- Current first failure: frame `172`, field `player.x_pos`, native `0`, oracle `19398656` (`0x0128_0000`)

## Why This Case

- The case-queue loop selected the same case again, so the session stayed on this case only.
- The task package still marks it evidence-only, so the correct scope was re-verification plus tighter evidence, not a speculative gameplay patch.
- The reduced shell still isolates a single hidden preload pose before visible gameplay, object population, or event drift appears.

## Strongest Evidence

- `cargo run -p kidcli -- autonomous-loop --goal case-queue`
  - cycle `63`
  - status `stalled`
  - selected case still `death_respawn_level1_case_00000_00200`
- `cargo run -p kidcli -- case verify --case death-respawn-test`
  - still `FAIL`
  - first bad frame/field unchanged: frame `172`, `player.x_pos`
- `.cache/cases/death-respawn-test/native-semantic.json` vs `.cache/cases/death-respawn-test/oracle-semantic.json`
  - frames `168..171`: both sides still show `game_mode=8`, `camera=(0,0)`, `entities=[]`, `events=[]`, `player=(0,0)`
  - frame `172`: oracle alone jumps to `player=(0x0128_0000,0x0100_0000)` while camera and events still stay at zero
  - implication: the first authoritative mismatch is still a lone hidden Kid preload pose, not camera drift or spawned-object drift
- `.cache/upstream/kid-chameleon-disasm/kid.asm`
  - `Title_InputLoop` writes `Game_Mode=$0008` and jumps to `loc_6E2`
  - `loc_6E2` clears VRAM/object slots and dispatches via `j_LoadGameModeData`
  - `Load_InGame` at `loc_11BE4` seeds `PlayerStart_X_pos/Y_pos` from the map header
  - `Load_InGame` at `loc_11F48` derives camera preload from the same start coordinates
  - `loc_7526` copies `PlayerStart_X_pos/Y_pos` into Kid's gfx object and then subtracts `1` from `y_pos`
  - implication: the ROM path re-read this session still proves the Level 1 header seed `x=$0020`, `y=$0100`, but still does not explain the oracle-only hidden `x=$0128`
- `crates/kid-core/src/trace.rs`
  - `early_start_load_ingame_player_state()` still returns `None` before `EARLY_START_LOAD_INGAME_VISIBLE_FRAME`
  - once visible, Level 1 still hardcodes `player.x_pos = 0x0020_0000`
  - implication: native still withholds the hidden preload entirely, and the visible fallback still matches the header seed rather than the oracle-only hidden pose

## Exact Next Moves

1. Stay on `death_respawn_level1_case_00000_00200` only; do not widen into later respawn windows or other case IDs.
2. Map reduced frame `172` onto the exact `BootState::StartingLevel { elapsed_frames, uses_load_ingame_bootstrap }` timeline so the hidden trace omission is pinned to a specific bootstrap frame.
3. Trace the ROM path after `loc_7526` and the first collision/settle writes that can touch Kid's `x_pos(a3)` before anything becomes visible; the missing proof is the source of oracle `x=$0128`.
4. If a later session patches this, do not tune only `EARLY_START_LOAD_INGAME_VISIBLE_FRAME`; prove the `x` source from ASM or tighter oracle correlation first.
5. Re-run `cargo run -p kidcli -- case verify --case death-respawn-test` after any evidence or gameplay change and stop if the first bad frame moves earlier.

## Verification To Re-run

- `cargo run -p kidcli -- autonomous-loop --goal case-queue`
- `cargo run -p kidcli -- case verify --case death-respawn-test`
- `cargo run -p kidcli -- case summarise --case death-respawn-test`

## Dead Ends / Do Not Retry

- Do not pass the raw case ID to `kidcli case verify`; the CLI expects the selector `death-respawn-test`.
- Do not patch `EARLY_START_LOAD_INGAME_VISIBLE_FRAME` from the symptom alone; visibility timing does not explain the wrong hidden `x` source.
- Do not assume the Level 1 header seed solves the case; the ROM bootstrap path read this session still yields `PlayerStart_X_pos = 0x0020`, not oracle `0x0128`.
- Do not widen into `pause_resume_level1_case_00000_00200` or `death_respawn_level1_case_00300_00500`; this session is only about `death_respawn_level1_case_00000_00200`.
