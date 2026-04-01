---
slug: death_respawn_level1_case_00000_00200
cycle: 57
updated_at: 2026-04-01T03:04:28Z
---

## What Changed

- Re-ran `cargo run -p kidcli -- autonomous-loop --goal case-queue`; cycle `57` selected `death_respawn_level1_case_00000_00200` again and kept it evidence-first.
- Re-verified the selected case with `cargo run -p kidcli -- case verify --case death-respawn-test`; it still fails first on frame `172` at `player.x_pos`.
- Refreshed the reduced artefacts with `cargo run -p kidcli -- case summarise --case death-respawn-test`.
- Re-read the ROM path in `.cache/upstream/kid-chameleon-disasm/kid.asm`: `Title_InputLoop`, `loc_6E2`, `Load_TitleCard`, `loc_1AB26`, `loc_1AC70`, `loc_1ACFE`, `loc_1AE2E`, and `Load_InGame`.
- Confirmed the current native failure is the early title-start bootstrap path, not the later lose-life respawn shell: `Title_InputLoop` writes `Game_Mode=$0008` and jumps to `loc_6E2`, `loc_6E2` clears object/gfx slots before the mode dispatch, and `crates/kid-core/src/trace.rs` still hard-gates `early_start_load_ingame_player_state()` until local elapsed `60` even though the oracle reduced case first exposes a hidden Level 1 pose on local elapsed `34`.
- Left gameplay code untouched.

## Selected Case

- Case ID: `death_respawn_level1_case_00000_00200`
- Selector: `death-respawn-test`
- Cycle: `57`
- Current result: `FAIL`
- Current first failure: frame `172`, field `player.x_pos`, native `0`, oracle `19398656`

## Why This Case

- The autonomous loop selected the same case again, so the session stayed on this case only.
- The task package still marks the case evidence-first, so the safe scope stayed: re-verify, refresh artefacts, and narrow the explanation with ROM-backed evidence.
- The case name is misleading at this reduced window: frames `0..172` are still the opening `Title_InputLoop -> loc_6E2 -> Load_InGame` lineage before the later death/respawn path ever begins.

## Strongest Evidence

- `cargo run -p kidcli -- autonomous-loop --goal case-queue`
  - cycle `57`
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
  - implication: native still hides the first lone Level 1 preload pose one frame too long and still treats the whole early mode-`8` window as fully blank
- `.cache/upstream/kid-chameleon-disasm/kid.asm`
  - `Title_InputLoop` writes `Game_Mode=$0008` then jumps straight to `loc_6E2`
  - `loc_6E2` resets VRAM/object slots before `LoadGameModeData` dispatch
  - `Load_TitleCard` allocates `loc_1AB26` and `loc_1AE2E`, `loc_1AB26` allocates the visible title-card worker swarm via `loc_1AC70`, and `loc_1ACFE` only animates those workers toward the title text coordinates
  - `loc_1AE2E` just waits for Start, writes `Game_Mode=$000C`, and jumps back to `loc_6E2`; this excerpt still does not prove who writes the reduced case's hidden frame-`172` pose
  - `Load_InGame` seeds `PlayerStart_X/Y` from the level header, but the reduced oracle shell stays semantically empty when the lone pose first appears
- `crates/kid-core/src/trace.rs`
  - `early_start_load_ingame_player_state()` returns `None` for every local elapsed frame `< 60`
  - this hard gate explains why native cannot expose the oracle pose on reduced frame `172` (local elapsed `34`) even before considering the exact ROM owner

## Exact Next Moves

1. Stay on `death_respawn_level1_case_00000_00200` only; do not widen into later respawn windows or other case IDs.
2. Prove the frame-`172` hidden pose owner in the ROM, not by inference: trace the exact post-`loc_6E2` mode dispatch and identify which path can write only `player.x/y` while camera, entities, and events stay empty.
3. Map reduced frame `172` onto the native local `StartingLevel.elapsed_frames` timeline and keep the distinction explicit: this is early title-start bootstrap, not the later `lose_life -> Teleport -> loc_6E2` path.
4. Only after the ROM proves the pose source, decide whether `early_start_load_ingame_player_state()` should expose an earlier hidden preload pose, or whether another hidden worker/object path owns that frame instead.
5. Re-run `cargo run -p kidcli -- case verify --case death-respawn-test` after any evidence or gameplay change and stop if the first bad frame moves earlier.

## Verification To Re-run

- `cargo run -p kidcli -- autonomous-loop --goal case-queue`
- `cargo run -p kidcli -- case verify --case death-respawn-test`
- `cargo run -p kidcli -- case summarise --case death-respawn-test`

## Dead Ends / Do Not Retry

- Do not treat this reduced case as evidence about the later death/respawn bootstrap; frame `172` is still on the opening title-start lineage.
- Do not patch `EARLY_START_LOAD_INGAME_VISIBLE_FRAME` from the symptom alone. The current hard gate explains the miss, but the ROM still does not prove that the lone pose belongs to `Load_InGame` itself.
- Do not assume the frame-`172` `(x=$0128_0000, y=$0100_0000)` pose proves visible `Load_TitleCard` workers. `loc_1AC70 -> loc_1ACFE` clearly drives title-card letters, but the reduced shell stays semantically empty, so that resemblance is still only a clue.
- Do not widen into `pause_resume_level1_case_00000_00200` or `death_respawn_level1_case_00300_00500`; this session is only about `death_respawn_level1_case_00000_00200`.
