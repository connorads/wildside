---
slug: death_respawn_level1_case_00000_00200
cycle: 45
updated_at: 2026-04-01T03:04:55Z
---

## What Changed

- Ran `cargo run -p kidcli -- autonomous-loop --goal case-queue`; cycle 45 selected `death_respawn_level1_case_00000_00200` again and still labelled it evidence-first.
- Re-verified the selected case before any code change with `cargo run -p kidcli -- case verify --case death-respawn-derived-check`; the case now passes.
- Refreshed the reduced-case summary with `cargo run -p kidcli -- case summarise --case death-respawn-derived-check`; the summary now reports `frames: 0..162` and `passed: true`.
- Read the ROM labels tied to the earlier title/death-respawn investigation in `.cache/upstream/kid-chameleon-disasm/kid.asm`: `Title_InputLoop`, `loc_6E2`, and `Load_InGame`.
- Made no gameplay change and no code change because the selected case no longer reproduces.

## Selected Case

- Case ID: `death_respawn_level1_case_00000_00200`
- Selector used for re-verification: `death-respawn-derived-check`
- Autonomy cycle selection summary: first bad frame `136`, field `player.x_pos`
- Current verify result: `PASS`

## Why This Case

- The loop selected this exact case in cycle 45, so the session stayed on it only.
- The loop rules require re-verification before any widening or gameplay patch.
- Re-verification invalidated the stale task-package failure, so the right action this session was to capture that fact rather than patch adjacent behaviour.

## Strongest Evidence

- `cargo run -p kidcli -- case verify --case death-respawn-derived-check`
  - result: `case death_respawn_level1_case_00000_00200: PASS`
- `.cache/cases/death-respawn-derived-check/summary.md`
  - `passed: true`
  - `frames: 0..162`
  - matching fields include `player.position` and `player.motion`
- `.cache/upstream/kid-chameleon-disasm/kid.asm`
  - `Title_InputLoop` remains the title-mode input loop
  - `loc_6E2` remains the mode-reset path
  - `Load_InGame` remains the later gameplay hand-off

## Exact Next Moves

1. Start by running `cargo run -p kidcli -- autonomous-loop --goal case-queue` again to refresh the queue after this stale pass result.
2. Read the newly emitted `.cache/autonomy/case-queue/next-task.json`.
3. Take only that newly selected case.
4. Re-verify it before any gameplay change.
5. Read the relevant ASM labels before patching if the new case still reproduces.

## Verification To Re-run

- `cargo run -p kidcli -- case verify --case death-respawn-derived-check`
- `cargo run -p kidcli -- case summarise --case death-respawn-derived-check`
- `cargo run -p kidcli -- autonomous-loop --goal case-queue`

## Dead Ends / Do Not Retry

- Do not patch `death_respawn_level1_case_00000_00200` again unless `case verify` turns red first.
- Do not widen into the pause/resume twin case from this session; the loop selected only the death-respawn case.
- Do not trust the stale `first_bad_frame: 136` task package without re-verification; cycle 45 proved the reduced case can already pass.
