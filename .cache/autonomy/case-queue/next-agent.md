---
slug: death_respawn_level1_case_00700_00800
cycle: 10
updated_at: 2026-03-31T22:37:52Z
---

## What Changed

- Ran `cargo run -p kidcli -- autonomous-loop --goal case-queue`; cycle 10 selected `death_respawn_level1_case_00700_00800` via selector `death-respawn-level1-gameplay`.
- Re-verified the selected case first with `cargo run -p kidcli -- case verify --case death-respawn-level1-gameplay`; it returned `case death_respawn_level1_case_00700_00800: PASS`.
- Refreshed the case summary with `cargo run -p kidcli -- case summarise --case death-respawn-level1-gameplay`; the cache now records `passed: true`.
- Read the ROM around `loc_1AE2E` in `.cache/upstream/kid-chameleon-disasm/kid.asm` before considering gameplay changes. No gameplay patch landed because the selected case is already closed.

## Selected Case

- Case ID: `death_respawn_level1_case_00700_00800`
- Selector: `death-respawn-level1-gameplay`
- Autonomy gate: `patch-allowed`
- Current verify result: `PASS`
- Current first failure: none

## Why This Case

- The cycle 10 task package selected this case as the highest-ranked autonomy-executable gameplay case.
- The task package still carried stale failure metadata (`world.game_mode` at frame 700), so the immediate job was to re-check the live case rather than trust the cached failure.
- Re-verification showed the case is already closed, so widening into another case in this session would violate the loop contract.

## Strongest Evidence

- `cargo run -p kidcli -- case verify --case death-respawn-level1-gameplay` prints `case death_respawn_level1_case_00700_00800: PASS`.
- `.cache/cases/death-respawn-level1-gameplay/verify.json` records `"passed": true` and `"first_failure": null`.
- `.cache/cases/death-respawn-level1-gameplay/summary.md` records `passed: true` with matching fields including `world.game_mode`.
- `.cache/upstream/kid-chameleon-disasm/kid.asm:29057` (`loc_1AE2E`) loops on `bclr #Button_Start,(Ctrl_Pressed).w` and then writes `move.w #$C,(Game_Mode).w`.

## Exact Next Moves

1. Run `cargo run -p kidcli -- autonomous-loop --goal case-queue` again to refresh the queue after this confirmed closure.
2. Read the refreshed `.cache/autonomy/case-queue/next-task.json`.
3. Take the newly selected case only.
4. Leave `death_respawn_level1_case_00700_00800` alone unless a future re-verify makes it fail again.

## Verification To Re-run

- `cargo run -p kidcli -- case verify --case death-respawn-level1-gameplay`
- `cargo run -p kidcli -- case summarise --case death-respawn-level1-gameplay`
- `cargo run -p kidcli -- autonomous-loop --goal case-queue`

## Dead Ends / Do Not Retry

- Do not patch gameplay for this case while `case verify` still passes; there is no live failing behaviour to fix.
- Do not widen into `pause_resume_level1_case_00700_00800` or any other open case during this session; the loop says take the selected case only.
- Do not ignore the ROM transition at `loc_1AE2E`; if `world.game_mode` regresses here later, re-check the Start-edge path against ASM before changing native code.
