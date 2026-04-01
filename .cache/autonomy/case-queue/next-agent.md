---
slug: death_respawn_level1_case_00600_00750
cycle: 22
updated_at: 2026-04-01T00:12:54Z
---

## What Changed

- Ran `cargo run -p kidcli -- autonomous-loop --goal case-queue`; cycle 22 selected `death_respawn_level1_case_00600_00750`.
- Re-ran verification with the actual selector from `.cache/autonomy/case-queue/next-task.json`: `cargo run -p kidcli -- case verify --case death-respawn-level1-600-750`.
- Refreshed the stored summary with `cargo run -p kidcli -- case summarise --case death-respawn-level1-600-750`.
- Read the ROM death / teleport path in `.cache/upstream/kid-chameleon-disasm/kid.asm` before considering gameplay work. `lose_life` sets `Game_Mode` to `$0008` only after the death tumble completes, so a frame-600 mismatch is upstream of the hidden respawn bootstrap.
- Did not patch gameplay. The selected window now re-verifies as an inherited checkpoint failure, not an isolated respawn transition bug.

## Selected Case

- Case ID: `death_respawn_level1_case_00600_00750`
- Selector: `death-respawn-level1-600-750`
- Autonomy gate: `evidence-only`
- Current verify result: `FAIL`
- Current first failure: frame `600`, `player.x_pos` native `4096000`, oracle `2097152`

## Why This Case

- Cycle 22 selected this exact case, so it was the only allowed scope.
- Re-verification was required because the packaged task summary still pointed at `world.game_mode`.
- The refreshed failure shows this window inherits an older player-position drift at its checkpoint instead of isolating a new death / respawn transition.

## Strongest Evidence

- `cargo run -p kidcli -- case verify --case death-respawn-level1-600-750` returns `case death_respawn_level1_case_00600_00750: FAIL`.
- `.cache/cases/death-respawn-level1-600-750/verify.json` now records `player.x_pos` as the first failure at frame `600`.
- `.cache/cases/death-respawn-level1-600-750/summary.md` now matches the refreshed verify report.
- ASM `Death` -> `lose_life` -> `Teleport` in `.cache/upstream/kid-chameleon-disasm/kid.asm` shows `Game_Mode=$0008` is a later shell hand-off, so the old `world.game_mode` report was stale for this case.

## Exact Next Moves

1. Treat this case as inherited evidence only; do not patch the respawn shell from the `600..750` window.
2. If queue work continues, prefer the earlier `death_respawn_level1` player-position case that actually introduces the drift, or refresh queue packaging so selected-case summaries are regenerated before handoff.
3. Re-run `cargo run -p kidcli -- autonomous-loop --goal case-queue` only after accepting that this case has been reclassified as a checkpoint-inherited `player.x_pos` failure.

## Verification To Re-run

- `cargo run -p kidcli -- case verify --case death-respawn-level1-600-750`
- `cargo run -p kidcli -- case summarise --case death-respawn-level1-600-750`
- `sed -n '14714,14826p' .cache/upstream/kid-chameleon-disasm/kid.asm`

## Dead Ends / Do Not Retry

- Do not use the underscore case ID with `kidcli case verify`; the command takes the selector directory name (`death-respawn-level1-600-750`).
- Do not debug `world.game_mode` for this case unless a fresh verify report says it fails again; that was stale case metadata.
- Do not widen this into a gameplay fix for `Death` / `Teleport`; the selected window is already wrong on `player.x_pos` at its first frame.
