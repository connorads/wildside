---
slug: pause_resume_level1_case_00000_00200
cycle: 30
updated_at: 2026-04-01T00:52:32Z
---

## What Changed

- Ran `cargo run -p kidcli -- autonomous-loop --goal case-queue`; cycle 30 selected `pause_resume_level1_case_00000_00200` with selector `pause-resume-derived-0`.
- Re-verified the selected case with `cargo run -p kidcli -- case verify --case pause-resume-derived-0`; the live first failure is `player.x_pos` at frame 136, not the stale frame-0 report in the task summary.
- Re-ran `cargo run -p kidcli -- case summarise --case pause-resume-derived-0` so `.cache/cases/pause-resume-derived-0/summary.md` matches the live verify result.
- Reproduced the same failure with `cargo run -p kidcli -- compare-native-gameplay-window --scenario pause-resume-level1 --frame-start 130 --frame-end 140 --checkpoint-interval 1`; oracle exposes Kid at `(x=$28,y=$30)` on frame 136 while native still shows the blank shell.
- Read the relevant ROM handoff in `.cache/upstream/kid-chameleon-disasm/kid.asm`: `Obj_TitleMenu loc_1BB02` flips to `Game_Mode=$10` and jumps to `loc_6E2`, then `Load_InGame` performs the later rebuild. `sub_6874`'s gameplay pause loop is not on the failing frame-136 path.
- Did not patch gameplay code. The task package is still `evidence-only`, so this session only tightened evidence and refreshed the handoff.

## Selected Case

- Case ID: `pause_resume_level1_case_00000_00200`
- Selector: `pause-resume-derived-0`
- Autonomy gate: `evidence-only`
- Verify result: `FAIL`
- First bad frame: `136`
- First bad field: `player.x_pos`

## Why This Case

- The loop spec required taking only the selector from the refreshed `.cache/autonomy/case-queue/next-task.json`.
- Re-verification showed the selected case was already tighter than the task summary claimed; frame 0 was stale metadata, not the live frontier.
- Although the source scenario is named pause/resume, this 0..200 case fails before the later in-game pause input window, so the useful work here is narrowing the title-to-level bootstrap drift without widening into later gameplay.

## Strongest Evidence

- `.cache/cases/pause-resume-derived-0/verify.json` now records `frame_index: 136`, `field: player.x_pos`, native `0`, oracle `2621440`.
- `.cache/cases/pause-resume-derived-0/summary.md` now matches that verify result: first bad frame `136`, first bad field `player.x_pos`.
- `cargo run -p kidcli -- compare-native-gameplay-window --scenario pause-resume-level1 --frame-start 130 --frame-end 140 --checkpoint-interval 1` fails at `frame_00136@136` with `kid X (native 00000000, oracle 00280000)`.
- The compare artefacts in `.cache/native-gameplay-oracle-runs/pause_resume_level1_window_00130_00140_step_1/` show oracle frame 136 at `kid_x_pos=0x0028_0000`, `kid_y_pos=0x0030_0000`, while native stays at zero for both.
- The same compare window shows this is not the gameplay pause loop yet: both sides remain on `game_mode=44` through frame 137, and only oracle starts exposing the hidden bootstrap pose before the later mode-8 handoff.
- ROM `Obj_TitleMenu loc_1BB02` and `Load_InGame` own this early lineage; `sub_6874` only handles later in-game pause/unpause and should not be patched for this case.

## Exact Next Moves

1. Inspect the frame-136 lineage in [`trace.rs`](/home/connor/git/kc/crates/kid-core/src/trace.rs) and [`boot.rs`](/home/connor/git/kc/crates/kid-core/src/boot.rs), focusing on the title-menu to `Load_InGame` carry that should expose the hidden `(0x0028_0000, 0x0030_0000)` pose before mode 8.
2. Add a focused regression in [`tests.rs`](/home/connor/git/kc/crates/kid-core/src/tests.rs) proving that `pause_resume_level1` must expose Kid at `(0x0028_0000, 0x0030_0000)` on frame 136 while still before the later gameplay pause inputs at frame 840.
3. Only after that regression is red, patch the matching bootstrap carry path and rerun the selected-case verification plus the frame-130..140 gameplay-window compare.

## Verification To Re-run

- `cargo run -p kidcli -- case verify --case pause-resume-derived-0`
- `cargo run -p kidcli -- case summarise --case pause-resume-derived-0`
- `cargo run -p kidcli -- compare-native-gameplay-window --scenario pause-resume-level1 --frame-start 130 --frame-end 140 --checkpoint-interval 1`

## Dead Ends / Do Not Retry

- Do not trust the stale task summary that still says frame 0; the selected case now verifies to frame 136.
- Do not treat this as an in-game pause-menu bug. The selected 0..200 window fails long before the later pause/unpause Start presses in the scenario.
- Do not patch `sub_6874` / `Game_Paused_Loop` for this case. The failing lineage is `Obj_TitleMenu loc_1BB02 -> Load_InGame`, not the gameplay pause loop.
- Do not run `case summarise` before the fresh `case verify` if you need the updated summary immediately; the summary can remain stale even when `verify.json` is already correct.
