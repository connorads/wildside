---
slug: death_respawn_level1_case_00000_00200
cycle: 53
updated_at: 2026-04-01T03:09:00Z
---

## What Changed

- Re-ran `cargo run -p kidcli -- autonomous-loop --goal case-queue`; cycle `53` selected `death_respawn_level1_case_00000_00200` again and kept it evidence-first.
- Re-verified the selected case with `cargo run -p kidcli -- case verify --case death-respawn-test`; it still fails first on frame `172` at `player.x_pos`.
- Refreshed the reduced-case artefacts with `cargo run -p kidcli -- case summarise --case death-respawn-test`.
- Re-read the ROM path in `.cache/upstream/kid-chameleon-disasm/kid.asm`: `Title_InputLoop`, `loc_6E2`, `Load_TitleCard`, `loc_1AB26`, `loc_1AC70`, `loc_1ACFE`, `loc_1AE2E`, and `Load_InGame`.
- Re-ran the opening regression tests `death_respawn_level_1_exposes_title_menu_cursor_on_frame_136` and `death_respawn_level_1_clears_title_cursor_pose_before_title_shell_counts`; both still pass.
- Tightened the case hypothesis: the refreshed semantic slice contradicts the current “hidden preload workers begin on frame 168” assumption. Oracle stays empty through frame `171` and only exposes a pose on frame `172`.

## Selected Case

- Case ID: `death_respawn_level1_case_00000_00200`
- Selector: `death-respawn-test`
- Cycle: `53`
- Current result: `FAIL`
- Current first failure: frame `172`, field `player.x_pos`, native `0`, oracle `19398656`

## Why This Case

- The autonomous loop selected the same case again, so the session stayed on this case only.
- The task package still has no implementation targets and still describes the work as evidence-first, so the safe scope was: re-verify, tighten the evidence, and leave gameplay code untouched.
- The strongest remaining ambiguity is now narrower than “player physics/collision”. The case shows a hidden mode-`8` window with matching game mode, camera, motion, entities, and events; only the player pose appears one frame later in oracle than native currently models.

## Strongest Evidence

- `cargo run -p kidcli -- autonomous-loop --goal case-queue`
  - cycle `53`
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
  - implication: native still exposes no hidden mode-`8` player shell at the first bad frame
- `.cache/upstream/kid-chameleon-disasm/kid.asm`
  - `Title_InputLoop` writes `Game_Mode=$0008` and jumps straight to `loc_6E2`
  - `Load_TitleCard` is the separate path that allocates `loc_1AB26` and `loc_1AE2E`
  - `loc_1AC70 -> loc_1ACFE` can seed worker poses with `y=$0100`, but only on the title-card object lineage
  - implication: do not assume the early title-start path reaches `Load_TitleCard` before the first bad frame just because the oracle pose has `y=$0100`
- `crates/kid-core/src/trace.rs`
  - `early_start_load_ingame_player_state()` currently stays invisible until local frame `60`
  - `early_start_load_ingame_object_counts()` still assumes the hidden preload shell starts at local frame `30`
  - implication: the current native model already bakes in a stale “frame 168 preload workers” theory that the refreshed reduced case no longer supports
- `cargo test -p kid-core death_respawn_level_1_exposes_title_menu_cursor_on_frame_136 -- --nocapture`
  - `ok`
- `cargo test -p kid-core death_respawn_level_1_clears_title_cursor_pose_before_title_shell_counts -- --nocapture`
  - `ok`

## Exact Next Moves

1. Stay on `death_respawn_level1_case_00000_00200` only; do not widen into later respawn windows or neighbouring case IDs.
2. Trace the exact early-start lineage after `Title_InputLoop -> loc_6E2`: prove from ASM whether this path can reach `Load_TitleCard` before the first visible hand-off, or whether the frame-`172` pose comes from a different hidden bootstrap exposure.
3. Audit the RNG consumption on that lineage before frame `172`. If the oracle pose really matches `loc_1AC70`, the ROM must account for `x=$0128` and `y=$0100` on that exact frame.
4. Before any gameplay patch, replace the stale assumption in the evidence with an ASM-backed local-frame mapping: empty shell through frame `171`, first pose on frame `172`.
5. Re-run `cargo run -p kidcli -- case verify --case death-respawn-test` after any evidence or gameplay change and stop if the first bad frame moves earlier or the opening title-shell tests regress.

## Verification To Re-run

- `cargo run -p kidcli -- autonomous-loop --goal case-queue`
- `cargo run -p kidcli -- case verify --case death-respawn-test`
- `cargo run -p kidcli -- case summarise --case death-respawn-test`
- `cargo test -p kid-core death_respawn_level_1_exposes_title_menu_cursor_on_frame_136 -- --nocapture`
- `cargo test -p kid-core death_respawn_level_1_clears_title_cursor_pose_before_title_shell_counts -- --nocapture`

## Dead Ends / Do Not Retry

- Do not pass the case ID `death_respawn_level1_case_00000_00200` directly to `kidcli case verify`; the command needs the selector bundle `death-respawn-test`.
- Do not reopen the broad “player physics/collision” hypothesis first. The case is already reduced to a hidden mode-`8` pose exposure problem.
- Do not assume the old `EARLY_START_LOAD_INGAME_PRELOAD_OBJECTS_FRAME` story is still supported. The refreshed reduced case stays empty through frame `171`; treat the “preload workers start at frame 168” theory as stale until the ASM proves it.
- Do not treat the frame-`172` pose as proof that the path has already reached `Load_InGame`; the camera is still zero and the current reduced case has no later visible hand-off evidence yet.
- Do not widen into `pause_resume_level1_case_00000_00200` or `death_respawn_level1_case_00300_00500`; this session is only about `death_respawn_level1_case_00000_00200`.
