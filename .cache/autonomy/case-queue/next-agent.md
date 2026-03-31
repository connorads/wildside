---
slug: death_respawn_level1_case_00700_00750
cycle: 7
updated_at: 2026-03-31T22:32:50Z
---

## What Changed

- Ran `cargo run -p kidcli -- autonomous-loop --goal case-queue` and took only `death_respawn_level1_case_00700_00750` (`test-case`).
- Re-verified the selected case before changing code; `cargo run -p kidcli -- case verify --case test-case` now returns `PASS`.
- Read ASM `loc_1AE2E` in `.cache/upstream/kid-chameleon-disasm/kid.asm` before considering any gameplay patch. The ROM still shows Start advancing `Game_Mode` to `$000C`.
- No gameplay patch landed. The selected case closed during re-verification, so the remaining work is queue refresh, not another fix attempt on this case.

## Selected Case

- Case ID: `death_respawn_level1_case_00700_00750`
- Selector: `test-case`
- Current verify result: `PASS`
- Current first failure: none

## Why This Case

- Cycle 7 selected it as the autonomy-ready case-queue task.
- It was the highest-ranked reduced authoritative gameplay case in `.cache/cases/` at selection time.
- Re-verification showed the cached failure artefacts were stale for this case, so the focused work was to confirm closure rather than widen scope.

## Strongest Evidence

- `cargo run -p kidcli -- case verify --case test-case` prints `case death_respawn_level1_case_00700_00750: PASS`.
- `.cache/cases/test-case/verify.json` now records `"passed": true` and `"first_failure": null`.
- `.cache/cases/test-case/summary.md` now records `passed: true`.
- Relevant ASM: `.cache/upstream/kid-chameleon-disasm/kid.asm:29037` (`loc_1AE2E`) waits for a Start edge, then writes `Game_Mode=$000C`.

## Exact Next Moves

1. Run `cargo run -p kidcli -- autonomous-loop --goal case-queue` again to refresh the queue after this case closure.
2. Read the refreshed `.cache/autonomy/case-queue/next-task.json`.
3. Take the newly selected case only.
4. Leave `death_respawn_level1_case_00700_00750` closed unless it regresses.

## Verification To Re-run

- `cargo run -p kidcli -- case verify --case test-case`
- `cargo run -p kidcli -- case summarise --case test-case`
- `cargo run -p kidcli -- autonomous-loop --goal case-queue`

## Dead Ends / Do Not Retry

- Do not patch gameplay for this case without first re-running `case verify`; the current selected bundle already passes.
- Do not widen into another case during the same session once the selected case closes on re-verification.
- Do not ignore the ROM transition at `loc_1AE2E`; if this case regresses later, re-check the Start-edge path against ASM before changing gameplay code.
