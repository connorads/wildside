---
slug: death_respawn_level1_case_00000_00200
cycle: 40
updated_at: 2026-04-01T01:51:05Z
---

## What Changed

- Ran `cargo run -p kidcli -- autonomous-loop --goal case-queue`; cycle 40 selected `death_respawn_level1_case_00000_00200` (`death-respawn-derived-check`).
- Re-verified the selected case before any gameplay work; it now fails first on frame 161 at `player.x_pos`.
- Refreshed the case summary after verify so the selected bundle records the shifted first failure at frame 161 instead of the stale frame-136 report.
- Read the relevant ROM path before considering any gameplay change:
  - `.cache/upstream/kid-chameleon-disasm/kid.asm`: `Obj_TitleMenu`, `loc_1BAEA`, `loc_1BB52`, `Title_InputLoop`, `Load_InGame`
- Compared native vs oracle on frames 150..170 and confirmed the broad case now diverges one frame later than the tightened 130..141 fix:
  - native still exposes Kid at the grounded bootstrap pose on frame 161
  - oracle clears Kid to `(0,0)` on frame 161 while `Game_Mode=$0008` still matches
- Did not patch gameplay. The task package marks this selected case `evidence-only`, so the correct move is to tighten/reduce the case before touching runtime behaviour again.

## Selected Case

- Case ID: `death_respawn_level1_case_00000_00200`
- Selector: `death-respawn-derived-check`
- Autonomy gate: `evidence-only`
- Current verify result: `FAIL`

## Why This Case

- Cycle 40 selected this exact `0..200` reduced case, so the session stayed on the chosen case only.
- The prior frame-136 title-menu mismatch is gone. The remaining failure moved to frame 161, which means the selected case now spans beyond the already-fixed title-start hand-off and into the later in-game bootstrap lineage.
- Because the autonomy gate is now `evidence-only`, the value of this session was tightening the explanation of the remaining failure rather than widening into speculative gameplay fixes.

## Strongest Evidence

- `cargo run -p kidcli -- case verify --case death-respawn-derived-check`
  - result: `case death_respawn_level1_case_00000_00200: FAIL`
  - first failure: frame 161, `player.x_pos`, native `2621440`, oracle `0`
- `cargo run -p kidcli -- case summarise --case death-respawn-derived-check`
  - result: selected case summary refreshed with `first_bad_frame: 161`
- `cargo run -p kidcli -- compare-native-gameplay-window --scenario death-respawn-level1 --frame-start 150 --frame-end 170 --checkpoint-interval 1`
  - result: divergence on frame 161, `kid_x_pos` native `0x00280000`, oracle `0x00000000`
- ASM basis:
  - `Obj_TitleMenu loc_1BAEA` and `loc_1BB52` explain the already-fixed earlier title-menu timing
  - `Title_InputLoop` still owns the `Game_Mode=$0008` hand-off
  - `Load_InGame` owns the later level bootstrap that seeds the player pose from `PlayerStart_X_pos` / `PlayerStart_Y_pos` before hidden settle work

## Exact Next Moves

1. Tighten the selected case around frame 161 before any gameplay patch. Use `kidcli case reduce --case death-respawn-derived-check` or derive an equivalent narrower replay-matched window that preserves the frame-161 `player.x_pos` signature.
2. Once the reduced case is stable, inspect the native `BootState::StartingLevel` / `Load_InGame` hand-off against the ROM-owned `loc_6E2 -> Load_InGame` path. The remaining mismatch is no longer the title cursor timing.
3. Only allow a gameplay patch after the reduced case points at the same frame-161 failure and the first responsible bootstrap step is identified from ASM plus oracle.

## Verification To Re-run

- `cargo run -p kidcli -- case verify --case death-respawn-derived-check`
- `cargo run -p kidcli -- case summarise --case death-respawn-derived-check`
- `cargo run -p kidcli -- compare-native-gameplay-window --scenario death-respawn-level1 --frame-start 150 --frame-end 170 --checkpoint-interval 1`
- After tightening the case: re-run verify on that tighter bundle and confirm frame 161 remains the first authoritative failure.

## Dead Ends / Do Not Retry

- Do not reopen the already-fixed frame-136 title-menu work. This selected case now fails later, at frame 161.
- Do not widen into `lose_life`, `Teleport`, or late respawn logic yet. The selected case still sits in the early `Game_Mode=$0008` bootstrap lineage.
- Do not patch player physics or collision from the broad `0..200` case alone. Tighten the evidence first, then patch from the reduced frame-161 case.
