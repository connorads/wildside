---
slug: pause_resume_level1_case_00000_00200
cycle: 31
updated_at: 2026-04-01T00:54:47Z
---

## What Changed

- Ran `cargo run -p kidcli -- autonomous-loop --goal case-queue` for cycle 31, then followed the loop spec by treating `.cache/autonomy/case-queue/next-task.json` as the authoritative selector package.
- Re-verified the selected case with `cargo run -p kidcli -- case verify --case pause-resume-derived-0`; it still fails at frame 136 on `player.x_pos`.
- Refreshed `cargo run -p kidcli -- case summarise --case pause-resume-derived-0`; the case summary still matches the live frame-136 failure.
- Read the relevant ROM path in `.cache/upstream/kid-chameleon-disasm/kid.asm`: `Obj_TitleMenu loc_1BB02` flips `Game_Mode` to `$10` and jumps into `loc_6E2`, whose main-loop reset then enters `Load_InGame`; `sub_6874` / `Game_Paused_Loop` are not on this early failing path.
- Did not patch gameplay code. The task package remains `evidence-only`, and this pass did not produce a tighter ROM-backed fix than the existing bootstrap hypothesis.

## Selected Case

- Case ID: `pause_resume_level1_case_00000_00200`
- Selector: `pause-resume-derived-0`
- Autonomy gate: `evidence-only`
- Verify result: `FAIL`
- First bad frame: `136`
- First bad field: `player.x_pos`

## Why This Case

- The loop spec says to run the selector, then read `.cache/autonomy/case-queue/next-task.json`, then work only that selected case.
- Cycle 31's terminal summary mentioned `death_respawn_level1_case_00200_00400`, but the refreshed task package still selects `pause_resume_level1_case_00000_00200`.
- Because the package is the explicit machine task artefact and already carries this case's selector, sources, and gate, this session stayed on `pause-resume-derived-0` only.

## Strongest Evidence

- `.cache/cases/pause-resume-derived-0/verify.json` still records `frame_index: 136`, `field: player.x_pos`, native `0`, oracle `2621440`.
- `.cache/cases/pause-resume-derived-0/summary.md` matches that live verify result: first bad frame `136`, first bad field `player.x_pos`.
- ROM `Obj_TitleMenu loc_1BB02` is the title-to-gameplay hand-off for this lineage: it writes `Game_Mode=$10`, sets `$FFFFFBCE`, and jumps to `loc_6E2`.
- ROM `loc_6E2` clears VRAM/object slots, reloads mode data, and re-enters the main loop before `Load_InGame` runs, so this case is bootstrap/load-in-game work, not the later pause loop.
- Native code already models this path in [`trace.rs`](/home/connor/git/kc/crates/kid-core/src/trace.rs) and [`level.rs`](/home/connor/git/kc/crates/kid-core/src/level.rs), but the selected case still never exposes the expected Level 1 pose by frame 136.

## Exact Next Moves

1. Reproduce the frame-136 gap with a tighter ROM-backed probe around the early title-start lineage, not the later pause inputs, and keep the scope inside `pause-resume-derived-0`.
2. Inspect the native frame timing for `BootState::StartingLevel { uses_load_ingame_bootstrap: true }` against the ROM path `loc_1BB02 -> loc_6E2 -> Load_InGame`, especially why the current trace helper only starts exposing the Level 1 bootstrap pose much later than frame 136.
3. If that timing mismatch can be proven with a focused failing regression, add the test first, then patch only the bootstrap exposure logic and rerun selected-case verification.

## Verification To Re-run

- `cargo run -p kidcli -- case verify --case pause-resume-derived-0`
- `cargo run -p kidcli -- case summarise --case pause-resume-derived-0`

## Dead Ends / Do Not Retry

- Do not switch to the `death_respawn...` case just because the cycle-31 stdout summary mentioned it; the task package for this run still selects `pause-resume-derived-0`.
- Do not patch `sub_6874` / `Game_Paused_Loop` for this case. The selected 0..200 window fails on the title-to-Load-InGame bootstrap path before the gameplay pause logic matters.
- Do not widen back out to the 0..500 parent case while this reduced 0..200 case remains the selected package.
- Do not treat the existing native `trace.rs` comments as proof that the frame timing is right; the live verify result shows the current bootstrap exposure still misses the oracle by frame 136.
