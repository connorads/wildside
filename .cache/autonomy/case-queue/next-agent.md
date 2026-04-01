---
slug: death_respawn_level1_case_00000_00500
cycle: 33
updated_at: 2026-04-01T01:00:59Z
---

## What Changed

- Ran `cargo run -p kidcli -- autonomous-loop --goal case-queue`; cycle 33 selected `death_respawn_level1_case_00000_00500` from `death-respawn-derived`.
- Re-verified the selected bundle with `cargo run -p kidcli -- case verify --case death-respawn-derived`; it still fails.
- Refreshed `cargo run -p kidcli -- case summarise --case death-respawn-derived` to confirm the current evidence on disk.
- Read the ROM death/respawn bootstrap before considering any gameplay edit: `lose_life` / `Teleport`, `Load_TitleCard`, `loc_1AE2E`, and `Load_InGame` in `.cache/upstream/kid-chameleon-disasm/kid.asm`.
- Did not patch gameplay code. The task package marks this case `evidence-only`.

## Selected Case

- Case ID: `death_respawn_level1_case_00000_00500`
- Selector: `death-respawn-derived`
- Autonomy gate: `evidence-only`
- Verify result: `FAIL`
- First bad frame/field after re-verify: frame `136`, `player.x_pos`

## Why This Case

- The case-queue loop requires taking only the selected case from `.cache/autonomy/case-queue/next-task.json`.
- Cycle 33 selected the parent `0..500` death/respawn bundle again, even though the live summary no longer matches the stale frame-0 evidence quoted in the task package.
- Because the gate is `evidence-only`, the correct move was to re-verify, tighten evidence, and avoid widening into a gameplay patch.

## Strongest Evidence

- `cargo run -p kidcli -- case verify --case death-respawn-derived` still reports the selected case as failing.
- `.cache/cases/death-respawn-derived/summary.md` now records the current first failure as frame `136` on `player.x_pos` with native `0` vs oracle `2621440`; the stale frame-0 claim in `.cache/autonomy/case-queue/next-task.json` is no longer accurate.
- `.cache/cases/death-respawn-level1-0-100/summary.md` already passes, so the failure window is tighter than the selected `0..500` package suggests.
- ASM confirms the expected bootstrap path:
  `lose_life` / `Teleport` clears state, may set `Game_Mode=$0008`, then jumps to `loc_6E2`;
  `Load_TitleCard` allocates the title-card objects;
  `loc_1AE2E` waits for Start, sets `Game_Mode=$000C`, sets `$FFFFFBCE`, then jumps to `loc_6E2`;
  `Load_InGame` later rebuilds player start state from the map header.

## Exact Next Moves

1. Refresh or regenerate the case-queue package so the selected case reflects the current first failure at frame `136` instead of the stale frame-0 evidence.
2. If the queue still selects this same bundle, prefer a tighter reduced case around the live failing region rather than reopening gameplay changes on the broad `0..500` slice.
3. Only after the case package is tightened and still fails should the next session patch gameplay code, using the same ROM path as ground truth.

## Verification To Re-run

- `cargo run -p kidcli -- autonomous-loop --goal case-queue`
- `cargo run -p kidcli -- case verify --case death-respawn-derived`
- `cargo run -p kidcli -- case summarise --case death-respawn-derived`

## Dead Ends / Do Not Retry

- Do not call `cargo run -p kidcli -- case verify --case death_respawn_level1_case_00000_00500`; `kidcli case verify` expects the selector (`death-respawn-derived`), not the case ID.
- Do not patch gameplay code from this handoff alone; the selected case is still `evidence-only`.
- Do not trust the stale frame-0 failure recorded in `.cache/autonomy/case-queue/next-task.json` without re-verifying the bundle.
- Do not sit on `cargo run -p kidcli -- case reduce --case death-respawn-derived` indefinitely. In this session it ran for more than a minute at full CPU with no new artefacts before being killed.
