---
slug: death_respawn_level1_case_00600_00750
cycle: 23
updated_at: 2026-04-01T00:14:32Z
---

## What Changed

- Ran `cargo run -p kidcli -- autonomous-loop --goal case-queue`; cycle 23 reported a `720..900` summary on stdout, but `.cache/autonomy/case-queue/next-task.json` still selected `death_respawn_level1_case_00600_00750`.
- Re-ran the selected case exactly as packaged: `cargo run -p kidcli -- case verify --case death-respawn-level1-600-750`.
- Refreshed the selected case summary with `cargo run -p kidcli -- case summarise --case death-respawn-level1-600-750`.
- Read the ROM death / teleport path in `.cache/upstream/kid-chameleon-disasm/kid.asm` before considering gameplay work.
- Did not patch gameplay. The selected window still fails at its checkpoint on `player.x_pos`, so the respawn-shell `world.game_mode` theory remains stale metadata, not a live bug in this case.

## Selected Case

- Case ID: `death_respawn_level1_case_00600_00750`
- Selector: `death-respawn-level1-600-750`
- Autonomy gate: `evidence-only`
- Verify result: `FAIL`
- First live failure: frame `600`, `player.x_pos` native `4096000`, oracle `2097152`

## Why This Case

- The loop spec says to work only the selected case from `.cache/autonomy/case-queue/next-task.json`.
- Even though `autonomous-loop` stdout mentioned `death_respawn_level1_case_00720_00900`, the task package on disk still pointed at `600..750`, so widening to the later case would have violated the selected-case rule.
- Re-verification was required because the packaged summary still claimed `world.game_mode` as the first failure.

## Strongest Evidence

- `cargo run -p kidcli -- case verify --case death-respawn-level1-600-750` returns `case death_respawn_level1_case_00600_00750: FAIL`.
- `.cache/cases/death-respawn-level1-600-750/verify.json` now records `player.x_pos` as the first failure at frame `600`; `world.game_mode` is listed under `matching_fields`.
- `.cache/cases/death-respawn-level1-600-750/summary.md` matches the refreshed verify report.
- ASM `.cache/upstream/kid-chameleon-disasm/kid.asm:14772` (`Teleport`) sets `Game_Mode` to `#8` only after the death / teleport shell runs, so a frame-600 checkpoint mismatch is upstream of that shell.

## Exact Next Moves

1. Keep this case evidence-only; do not patch `Death` / `Teleport` for the `600..750` window.
2. Fix or re-queue the earlier source of the inherited `player.x_pos` drift before returning to this case.
3. Refresh case-queue packaging so `autonomous-loop` stdout, `next-task.json`, and the case summary agree on the selected case and first failing field.

## Verification To Re-run

- `cargo run -p kidcli -- autonomous-loop --goal case-queue`
- `cargo run -p kidcli -- case verify --case death-respawn-level1-600-750`
- `cargo run -p kidcli -- case summarise --case death-respawn-level1-600-750`
- `sed -n '14714,14820p' .cache/upstream/kid-chameleon-disasm/kid.asm`

## Dead Ends / Do Not Retry

- Do not debug `world.game_mode` for this case unless a fresh verify report says it fails again; the live verify output says it matches.
- Do not widen this session to `death_respawn_level1_case_00720_00900` just because stdout mentioned it; the packaged selected case was still `600..750`.
- Do not patch the respawn shell from this case alone; the checkpoint is already wrong on `player.x_pos` at frame `600`.
