---
slug: pause_resume_level1_case_00000_00500
cycle: 35
updated_at: 2026-04-01T01:10:30Z
---

## What Changed

- Ran `cargo run -p kidcli -- autonomous-loop --goal case-queue`; cycle 35 selected `pause_resume_level1_case_00000_00500` from `pause-resume-derived`.
- Re-verified the selected bundle with `cargo run -p kidcli -- case verify --case pause-resume-derived`; it still fails.
- Refreshed `cargo run -p kidcli -- case summarise --case pause-resume-derived` so the on-disk summary matches the current verify result.
- Read the pause/resume ROM path before considering any gameplay edit: `loc_74B0`/`loc_74C8` (Start -> set `Game_Paused`), `sub_6874`, `Game_Paused_Loop`, and `loc_6C3E` in `.cache/upstream/kid-chameleon-disasm/kid.asm`.
- Tried `cargo run -p kidcli -- case reduce --case pause-resume-derived`, but like the earlier evidence-only session it ran for minutes with no visible progress and produced no new reduced artefacts.
- Did not patch gameplay code. The selected case still needs tighter evidence first.

## Selected Case

- Case ID: `pause_resume_level1_case_00000_00500`
- Selector: `pause-resume-derived`
- Autonomy gate: `evidence-only`
- Verify result: `FAIL`
- First bad frame/field after re-verify: frame `136`, `player.x_pos`

## Why This Case

- The loop selected this exact case from `.cache/autonomy/case-queue/next-task.json`, so the session stayed on `pause-resume-derived` only.
- The failure still happens inside the broad `0..500` bundle, and the existing reduced child `pause_resume_level1_case_00000_00200` already proves the same first failure persists at frame `136`.
- Because the gate remains `evidence-only`, the right move was to re-verify, tighten the explanation, and avoid any gameplay patch that would guess past the ROM.

## Strongest Evidence

- `cargo run -p kidcli -- case verify --case pause-resume-derived` still reports `pause_resume_level1_case_00000_00500: FAIL`.
- `.cache/cases/pause-resume-derived/verify.json` records the first failure at frame `136` on `player.x_pos` with native `0` and oracle `2621440`.
- `.cache/cases/pause-resume-derived/native-semantic.json` stays at `player.x_pos = 0` through at least frame `180`, while `.cache/cases/pause-resume-derived/oracle-semantic.json` has Kid spawned at frame `136` and flips `world.game_mode` from `44` to `8` at frame `138`.
- The already-reduced child `.cache/cases/pause-resume-derived-0/summary.md` (`0..200`) keeps the same first bad frame/field, so the selected `0..500` parent is broader than the current evidence needs.
- ASM says the pause path is driven by `loc_74C8` (`st (Game_Paused).w`) and then `sub_6874` / `Game_Paused_Loop`; the selected case’s first semantic mismatch appears before that path is isolated well enough to justify a gameplay fix.

## Exact Next Moves

1. Fix or instrument `kidcli case reduce --case pause-resume-derived` so it can finish on this selector instead of spinning for minutes with no visible progress.
2. Once reduction works, keep the selected case family only and replace the broad `0..500` parent with the tightest child that still reproduces frame `136` on `player.x_pos`.
3. If the tighter child still fails, determine whether the first mismatch is still a bootstrap/spawn issue rather than pause semantics by re-checking frames `136..140` before any gameplay patch.
4. Only patch gameplay after the tighter case is in place and the evidence stops pointing at pre-pause bootstrap ambiguity; if that happens, re-read the same ROM labels first.

## Verification To Re-run

- `cargo run -p kidcli -- autonomous-loop --goal case-queue`
- `cargo run -p kidcli -- case verify --case pause-resume-derived`
- `cargo run -p kidcli -- case summarise --case pause-resume-derived`
- `cargo run -p kidcli -- case reduce --case pause-resume-derived`

## Dead Ends / Do Not Retry

- Do not patch gameplay code from this handoff alone; the case is still `evidence-only`.
- Do not widen to another pause/resume or death/respawn case in the same session; cycle 35 selected `pause-resume-derived`.
- Do not assume the selected parent case is the tightest available evidence. The `pause_resume_level1_case_00000_00200` child already exists and still fails at the same frame.
- Do not describe this as a frame-0 failure. Re-verification still puts the first mismatch at frame `136`.
