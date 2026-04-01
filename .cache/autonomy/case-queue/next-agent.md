---
slug: death_respawn_level1_case_00000_00200
cycle: 50
updated_at: 2026-04-01T03:08:00Z
---

## What Changed

- Ran `cargo run -p kidcli -- autonomous-loop --goal case-queue`; cycle `50` selected `death_respawn_level1_case_00000_00200` again and still marked it evidence-first.
- Re-verified the selected case with `cargo run -p kidcli -- case verify --case death-respawn-test`; it still fails at frame `172` on `player.x_pos`.
- Refreshed the reduced-case summary with `cargo run -p kidcli -- case summarise --case death-respawn-test`.
- Re-read the ROM hand-off path in `.cache/upstream/kid-chameleon-disasm/kid.asm`: `Teleport` sets `Game_Mode=$0008`, returns through `loc_B8DE -> loc_6E2`, and `Load_InGame` later writes `PlayerStart_X_pos` / `PlayerStart_Y_pos` before snapping camera from those start positions.
- Corrected the working hypothesis: this reduced case is not inside the later Level 1 death-respawn shell. Its manifest starts at `frame_start=0`, and oracle frame `172` still shows `number_lives=3`, `current_level_id=0`, `camera=(0,0)`, and the title/bootstrap path rather than the post-death branch.
- Cross-checked the existing title/bootstrap tests in [`tests.rs`](/home/connor/git/kc/crates/kid-core/src/tests.rs): frame `136` already exposes the title-menu cursor pose `(x=$28,y=$30)`, frame `138` enters mode `$0008`, and frame `161` clears that cursor pose before later bootstrap phases continue.
- Made no gameplay code change; the evidence-only case still needs a tighter title/bootstrap explanation before any patch.

## Selected Case

- Case ID: `death_respawn_level1_case_00000_00200`
- Selector: `death-respawn-test`
- Cycle: `50`
- Current result: `FAIL`
- Current first failure: frame `172`, field `player.x_pos`, native `0`, oracle `19398656`

## Why This Case

- The autonomous selector chose the same case again, so the session stayed on this case only.
- The task package still says `evidence-only`, so the job was to tighten the explanation rather than patch blindly.
- The main surprise is durable and important: the case name inherits the parent scenario slug `death_respawn_level1`, but this reduced window is actually a boot/title bootstrap slice. Treating it as respawn evidence sends follow-up work to the wrong subsystem.

## Strongest Evidence

- `cargo run -p kidcli -- autonomous-loop --goal case-queue`
  - cycle `50`
  - status `stalled`
  - selected case still `death_respawn_level1_case_00000_00200`
- `.cache/cases/death-respawn-test/manifest.json`
  - `frame_start: 0`
  - `checkpoint_frame: 0`
  - `scenario.slug: death_respawn_level1_window_00000_00172_step_1`
  - implication: this reduced case begins at boot, not at the later death-respawn segment
- `.cache/cases/death-respawn-test/oracle-semantic.json`
  - frames `161..171`: `player.x_pos=0`, `player.y_pos=0`, `world.game_mode=8`, `number_lives=3`, `current_level_id=0`
  - frame `172`: oracle snaps to `x_pos=19398656` (`$0128_0000`) and `y_pos=16777216` (`$0100_0000`) while camera stays `(0,0)` and lives remain `3`
  - implication: this is still title/bootstrap state, not a post-death Level 1 respawn
- [`tests.rs`](/home/connor/git/kc/crates/kid-core/src/tests.rs#L19952)
  - `death_respawn_level_1_exposes_title_menu_cursor_on_frame_136` already documents the title-menu cursor hand-off from ASM `Obj_TitleMenu loc_1BB52`
- [`tests.rs`](/home/connor/git/kc/crates/kid-core/src/tests.rs#L20034)
  - `death_respawn_level_1_clears_title_cursor_pose_before_title_shell_counts` already documents the mode-8 bootstrap clearing sequence through frames `160..168`
- `.cache/upstream/kid-chameleon-disasm/kid.asm`
  - `Teleport` only explains a later mode-8 hand-off
  - `Load_InGame` is where `PlayerStart_X_pos` / `PlayerStart_Y_pos` and camera snaps are written
  - neither label justifies treating this frame-172 boot/title case as respawn evidence

## Exact Next Moves

1. Stay on `death_respawn_level1_case_00000_00200` only; do not widen back into later respawn windows.
2. Read the title/bootstrap ASM around `Obj_TitleMenu loc_1BAEA`, `loc_1BB52`, and the early `loc_6E2` mode-8 bootstrap path that runs between frames `136..172`.
3. Map oracle frame `172` to the exact title/bootstrap routine that produces `(x=$128,y=$100)` with `camera=(0,0)` and `number_lives=3`.
4. Tighten the case explanation or reduction around that title/bootstrap transition first. Only patch gameplay if the ROM path and reduced-case evidence both point at the same missing native state.

## Verification To Re-run

- `cargo run -p kidcli -- autonomous-loop --goal case-queue`
- `cargo run -p kidcli -- case verify --case death-respawn-test`
- `cargo run -p kidcli -- case summarise --case death-respawn-test`

## Dead Ends / Do Not Retry

- Do not reason from the parent scenario slug alone. `death_respawn_level1_case_00000_00200` is a boot/title slice despite the scenario name.
- Do not patch `PlayerLifecycle::Respawning` from this case. The selected window never reaches the later respawn shell.
- Do not re-use the old “player physics/collision” or “respawn visibility gate” hypothesis until the title/bootstrap frame-172 pose is mapped to the ROM.
- Do not widen into `pause_resume_level1_case_00000_00200` or `death_respawn_level1_case_00300_00500`; this session is only about `death_respawn_level1_case_00000_00200`.
