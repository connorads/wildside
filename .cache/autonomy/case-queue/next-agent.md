---
slug: death_respawn_level1_case_00300_00500
cycle: 34
updated_at: 2026-04-01T01:04:54Z
---

## What Changed

- Ran `cargo run -p kidcli -- autonomous-loop --goal case-queue`; cycle 34 selected `death_respawn_level1_case_00300_00500` from `death-respawn-300-500`.
- Re-verified the selected bundle with `cargo run -p kidcli -- case verify --case death-respawn-300-500`; it still fails.
- Refreshed `cargo run -p kidcli -- case summarise --case death-respawn-300-500` to confirm the current evidence on disk.
- Read the ROM death/respawn bootstrap before considering any gameplay edit: `lose_life` / `Teleport`, `Load_TitleCard`, `loc_1AE2E`, and `Load_InGame` in `.cache/upstream/kid-chameleon-disasm/kid.asm`.
- Tried to tighten the selected case with `cargo run -p kidcli -- case reduce --case death-respawn-300-500`, but it produced no output or new artefacts after about 70s and was killed.
- Did not patch gameplay code. The task package still marks this case `evidence-only`.

## Selected Case

- Case ID: `death_respawn_level1_case_00300_00500`
- Selector: `death-respawn-300-500`
- Autonomy gate: `evidence-only`
- Verify result: `FAIL`
- First bad frame/field after re-verify: frame `300`, `player.x_velocity`

## Why This Case

- The case-queue loop requires taking only the selected case from `.cache/autonomy/case-queue/next-task.json`.
- Cycle 34 selected the already-reduced `300..500` death/respawn slice, so the correct move was to stay on that bundle and avoid reopening broader death/respawn work.
- Because the gate remains `evidence-only`, the session could only re-verify, re-read the ROM path, and attempt to tighten the same case.

## Strongest Evidence

- `cargo run -p kidcli -- case verify --case death-respawn-300-500` still reports `death_respawn_level1_case_00300_00500: FAIL`.
- `.cache/cases/death-respawn-300-500/verify.json` still records the first failure at frame `300` on `player.x_velocity` with native `0` and oracle `none`.
- `.cache/cases/death-respawn-300-500/summary.md` shows the same first failure across all generated branches while `world.game_mode`, `camera`, `player.position`, `player.motion`, `entities`, and `events` still match.
- ASM confirms the expected bootstrap path before this slice starts:
  `lose_life` / `Teleport` clears respawn state and may set `Game_Mode=$0008`;
  `Load_TitleCard` allocates the title-card objects;
  `loc_1AE2E` waits for Start, sets `Game_Mode=$000C`, sets `$FFFFFBCE`, then jumps to `loc_6E2`;
  `Load_InGame` rebuilds the level state, player start, camera, and enemy layout from the map header.

## Exact Next Moves

1. Inspect why `kidcli case reduce --case death-respawn-300-500` makes no visible progress on this selector; until that works, the evidence-only loop cannot tighten this case automatically.
2. After fixing or instrumenting reduction, rerun the reducer on `death-respawn-300-500` and keep only the child case if it preserves the frame-300 `player.x_velocity` failure.
3. Only if a tighter child case still fails and the autonomy gate changes from `evidence-only` should the next session patch gameplay code, using the same ROM labels as the source of truth.

## Verification To Re-run

- `cargo run -p kidcli -- autonomous-loop --goal case-queue`
- `cargo run -p kidcli -- case verify --case death-respawn-300-500`
- `cargo run -p kidcli -- case summarise --case death-respawn-300-500`
- `cargo run -p kidcli -- case reduce --case death-respawn-300-500`

## Dead Ends / Do Not Retry

- Do not patch gameplay code from this handoff alone; the selected case is still `evidence-only`.
- Do not widen back out to `death-respawn-derived` or another death/respawn slice in the same session; cycle 34 selected `death-respawn-300-500`, and the loop says to take the selected case only.
- Do not assume the reducer succeeded just because it started. In this session `cargo run -p kidcli -- case reduce --case death-respawn-300-500` ran for about 70 seconds with no output and no new `.cache/cases/*` artefacts before being killed.
