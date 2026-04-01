---
slug: death_respawn_level1_case_00000_00200
cycle: 58
updated_at: 2026-04-01T03:07:56Z
---

## What Changed

- Re-ran `cargo run -p kidcli -- autonomous-loop --goal case-queue`; cycle `58` selected `death_respawn_level1_case_00000_00200` again and kept it evidence-only.
- Re-verified only the selected case with `cargo run -p kidcli -- case verify --case death-respawn-test`; it still fails first on frame `172` at `player.x_pos`.
- Refreshed the reduced artefacts with `cargo run -p kidcli -- case summarise --case death-respawn-test`.
- Re-read the ROM path in `.cache/upstream/kid-chameleon-disasm/kid.asm`: `Title_InputLoop`, `loc_6E2`, `LoadGameModeData`, `Load_InGame`, and the later Kid gfx-object bootstrap at `loc_7526`.
- Tightened the diagnosis: the first bad reduced frame is a hidden Level 1 preload pose `(x=$0128_0000, y=$0100_0000)`, not a player-physics step and not visible title-card worker motion.
- Left gameplay code untouched.

## Selected Case

- Case ID: `death_respawn_level1_case_00000_00200`
- Selector: `death-respawn-test`
- Cycle: `58`
- Current result: `FAIL`
- Current first failure: frame `172`, field `player.x_pos`, native `0`, oracle `19398656`

## Why This Case

- The autonomous loop selected the same case again, so the session stayed on this case only.
- The task package still marks the case `evidence-only`, so the correct scope stayed: re-verify, tighten the explanation, and avoid widening into a symptom patch.
- The reduced window still sits on the opening `Title_InputLoop -> loc_6E2 -> Load_InGame` lineage before the later lose-life respawn shell exists.

## Strongest Evidence

- `cargo run -p kidcli -- autonomous-loop --goal case-queue`
  - cycle `58`
  - status `stalled`
  - selected case still `death_respawn_level1_case_00000_00200`
- `cargo run -p kidcli -- case verify --case death-respawn-test`
  - still `FAIL`
  - first bad frame/field unchanged: frame `172`, `player.x_pos`
- `cargo run -p kidcli -- inspect-level --level 00`
  - Level 1 header start is `player start: 32,256`
  - in fixed-point that is `(0x0020_0000, 0x0100_0000)`, so the oracle reduced case's `(0x0128_0000, 0x0100_0000)` is not the plain header start and the exact owner remains unproven
- `.cache/cases/death-respawn-test/oracle-semantic.json`
  - frames `168..171`: `player=(0,0)`, `camera=(0,0)`, `game_mode=8`, `entities=[]`, `events=[]`
  - frame `172`: `player=(0x0128_0000,0x0100_0000)`, `camera=(0,0)`, `game_mode=8`, `entities=[]`, `events=[]`
- `.cache/cases/death-respawn-test/native-semantic.json`
  - frames `168..172`: `player=(0,0)`, `camera=(0,0)`, `game_mode=8`, `entities=[]`, `events=[]`
  - implication: native still hides the first lone preload pose entirely
- `.cache/upstream/kid-chameleon-disasm/kid.asm`
  - `Title_InputLoop` writes `Game_Mode=$0008` then jumps to `loc_6E2`
  - `loc_6E2` clears VRAM, object slots, and gfx-object slots, then dispatches through `LoadGameModeData`
  - `Load_InGame` writes `PlayerStart_X_pos` and `PlayerStart_Y_pos` from the level header before the later camera snap at `loc_11F48`
  - `loc_7526` later allocates the Kid gfx object and copies `PlayerStart_X_pos/Y_pos` into `x_pos(a3)` / `y_pos(a3)`
  - the reduced oracle shell still shows `camera=(0,0)` and `entities=[]` on frame `172`, so the mismatch is still a hidden preload trace issue rather than visible title-card worker motion
  - the current evidence still does not prove which ROM path owns the oracle's exact `x=$0128` preload coordinate
- `crates/kid-core/src/trace.rs`
  - `early_start_load_ingame_player_state()` still returns `None` for every local elapsed frame `< 60`
  - for Level 1 it also hardcodes `player.x_pos = 0x0020_0000`, which does not match the oracle case's frame-`172` pose

## Exact Next Moves

1. Stay on `death_respawn_level1_case_00000_00200` only; do not widen into later respawn windows or other case IDs.
2. Trace the hidden early-start player pose owner past `Load_InGame`: identify exactly when the runtime should expose the Level 1 preload pose and whether it comes from the level header seed, the later Kid gfx-object bootstrap at `loc_7526`, or another hidden trace source.
3. Map reduced frame `172` onto `StartingLevel.elapsed_frames` with the current native boot timeline before changing any constant or pose helper.
4. If a later session patches this, do not tune only `EARLY_START_LOAD_INGAME_VISIBLE_FRAME`; the Level 1 preload `x_pos` source must also stop using the current synthetic `0x0020_0000` placeholder unless ASM proves that placeholder is correct.
5. Re-run `cargo run -p kidcli -- case verify --case death-respawn-test` after any evidence or gameplay change and stop if the first bad frame moves earlier.

## Verification To Re-run

- `cargo run -p kidcli -- autonomous-loop --goal case-queue`
- `cargo run -p kidcli -- case verify --case death-respawn-test`
- `cargo run -p kidcli -- case summarise --case death-respawn-test`
- `cargo run -p kidcli -- inspect-level --level 00`

## Dead Ends / Do Not Retry

- Do not treat this reduced case as evidence about the later death/respawn bootstrap; frame `172` is still on the opening title-start lineage.
- Do not patch `EARLY_START_LOAD_INGAME_VISIBLE_FRAME` from the symptom alone. The current hard gate explains the miss, but it is not the whole mismatch.
- Do not keep framing this case as generic `player physics, collision`; the strongest current evidence points to hidden boot-trace preload pose exposure.
- Do not assume the frame-`172` pose is one of the visible `loc_1AC70 -> loc_1ACFE` title-card workers; the reduced semantic shell is still empty apart from player coordinates.
- Do not widen into `pause_resume_level1_case_00000_00200` or `death_respawn_level1_case_00300_00500`; this session is only about `death_respawn_level1_case_00000_00200`.
