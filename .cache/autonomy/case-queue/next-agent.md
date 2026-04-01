---
slug: death_respawn_level1_case_00000_00200
cycle: 52
updated_at: 2026-04-01T02:50:11Z
---

## What Changed

- Re-ran `cargo run -p kidcli -- autonomous-loop --goal case-queue`; cycle `52` selected `death_respawn_level1_case_00000_00200` again and kept it `evidence-only`.
- Re-verified the selected case with the selector bundle command `cargo run -p kidcli -- case verify --case death-respawn-test`; it still fails first on frame `172` at `player.x_pos`.
- Refreshed the reduced-case artefacts with `cargo run -p kidcli -- case summarise --case death-respawn-test`.
- Re-read the ROM paths in `.cache/upstream/kid-chameleon-disasm/kid.asm`: `Obj_TitleMenu`, `Title_InputLoop`, `Load_TitleCard`, `loc_1AB26`, `loc_1AC70`, `loc_1AD2A`, `loc_1AE2E`, `loc_6E2`, and `Load_InGame`.
- Re-ran the opening regression tests `death_respawn_level_1_exposes_title_menu_cursor_on_frame_136` and `death_respawn_level_1_clears_title_cursor_pose_before_title_shell_counts`; both still pass, so the opening title-shell model before frame `168` remains consistent.

## Selected Case

- Case ID: `death_respawn_level1_case_00000_00200`
- Selector: `death-respawn-test`
- Cycle: `52`
- Current result: `FAIL`
- Current first failure: frame `172`, field `player.x_pos`, native `0`, oracle `19398656`

## Why This Case

- The autonomous loop selected the same case again, so the session stayed on this case only.
- The task package still marks it `evidence-only`, so the correct scope was: re-verify, tighten the evidence, and leave gameplay code untouched.
- The strongest remaining ambiguity is no longer “player physics/collision” in the abstract. The failure window is inside the hidden mode-`8` title/bootstrap path, after the cursor shell has cleared but before any visible `Load_InGame` hand-off.

## Strongest Evidence

- `cargo run -p kidcli -- autonomous-loop --goal case-queue`
  - cycle `52`
  - status `stalled`
  - selected case still `death_respawn_level1_case_00000_00200`
- `cargo run -p kidcli -- case verify --case death-respawn-test`
  - still `FAIL`
  - first bad frame/field unchanged: frame `172`, `player.x_pos`
  - note: `kidcli case verify` does not accept the case ID here; it needs the selector bundle `death-respawn-test`
- `.cache/cases/death-respawn-test/oracle-semantic.json`
  - frames `166..171`: `player=(0,0)`, `camera=(0,0)`, `game_mode=8`
  - frame `172`: `player=(0x0128_0000,0x0100_0000)`, `camera=(0,0)`, `game_mode=8`
- `.cache/cases/death-respawn-test/native-semantic.json`
  - frames `166..172`: `player=(0,0)`, `camera=(0,0)`, `game_mode=8`
  - implication: native is not exposing any transient title/bootstrap worker at the first bad frame
- `.cache/upstream/kid-chameleon-disasm/kid.asm`
  - `Title_InputLoop` writes `Game_Mode=$0008` and jumps through `loc_6E2`
  - `Load_TitleCard` allocates `loc_1AB26` and `loc_1AE2E`
  - `loc_1AC70 -> loc_1ACFE` can seed worker poses with `y=$0100` and randomised `x=((rand_byte+$3E)&$FFFC)`
  - implication: oracle frame `172` still fits a transient title-card worker shell better than the later `Load_InGame` settled spawn path
- `cargo test -p kid-core death_respawn_level_1_exposes_title_menu_cursor_on_frame_136 -- --nocapture`
  - `ok`
- `cargo test -p kid-core death_respawn_level_1_clears_title_cursor_pose_before_title_shell_counts -- --nocapture`
  - `ok`

## Exact Next Moves

1. Stay on `death_respawn_level1_case_00000_00200` only; do not widen into later respawn windows or neighbouring case IDs.
2. Trace the early title-start lineage from `Title_InputLoop` through `loc_6E2` and confirm whether `Load_TitleCard` actually runs on this path before the frame-`172` exposure.
3. Inspect the exact RNG consumption between the frame-`137` Start press and oracle frame `172`; the candidate worker pose implies `loc_1AC70`, but the correct `x=$0128` seed still needs ROM-backed RNG accounting.
4. If the ROM confirms a transient title-card worker is visible on this lineage, add a narrow trace/bootstrap fix for that hidden shell only. Do not reuse the generic `starting_level_title_workers` flow without proof.
5. Re-run `cargo run -p kidcli -- case verify --case death-respawn-test` after any change and stop if the first bad frame moves earlier or the mode-`8` opening tests regress.

## Verification To Re-run

- `cargo run -p kidcli -- autonomous-loop --goal case-queue`
- `cargo run -p kidcli -- case verify --case death-respawn-test`
- `cargo run -p kidcli -- case summarise --case death-respawn-test`
- `cargo test -p kid-core death_respawn_level_1_exposes_title_menu_cursor_on_frame_136 -- --nocapture`
- `cargo test -p kid-core death_respawn_level_1_clears_title_cursor_pose_before_title_shell_counts -- --nocapture`

## Dead Ends / Do Not Retry

- Do not pass the case ID `death_respawn_level1_case_00000_00200` directly to `kidcli case verify`; the command needs the selector bundle `death-respawn-test`.
- Do not reopen the broad “player physics/collision” hypothesis first. The first bad frame sits in hidden mode `8` with camera and motion still matching.
- Do not treat frame `172` as the later visible `Load_InGame` settled spawn without more proof; the oracle pose shape still aligns better with `loc_1AC70`.
- Do not widen into `pause_resume_level1_case_00000_00200` or `death_respawn_level1_case_00300_00500`; this session is only about `death_respawn_level1_case_00000_00200`.
