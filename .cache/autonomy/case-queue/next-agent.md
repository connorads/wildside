---
slug: death_respawn_level1_case_00000_00200
cycle: 41
updated_at: 2026-04-01T02:07:09Z
---

## What Changed

- Ran `cargo run -p kidcli -- autonomous-loop --goal case-queue`; cycle 41 selected the same case `death_respawn_level1_case_00000_00200` (`death-respawn-derived-check`).
- Re-verified the selected case before any gameplay work; it still fails first on frame 161 at `player.x_pos`.
- Read the relevant ROM bootstrap path before touching anything gameplay-facing:
  - `.cache/upstream/kid-chameleon-disasm/kid.asm`: `loc_6E2`, `Load_InGame`, `Obj_TitleMenu`, `Title_InputLoop`
- Confirmed with `compare-native-gameplay-window` that frame 161 diverges before `level_settle@719`:
  - native still exposes Kid at `0x00280000`
  - oracle has already cleared Kid to `0x00000000`
  - `world.game_mode`, camera, and player motion still match
- Tightened the selected case bundle from `0..200` down to `0..162` while preserving the exact first failure signature on frame 161.
- Fixed a `kidcli case reduce` no-op span bug that could loop forever once the continuation script was already mostly zeroed; this was required to finish reducing the selected case.

## Selected Case

- Case ID: `death_respawn_level1_case_00000_00200`
- Selector: `death-respawn-derived-check`
- Autonomy gate: `evidence-only`
- Current verify result: `FAIL`

## Why This Case

- Cycle 41 selected this exact case again, so the session stayed on the chosen case only.
- The earlier title-menu failure is gone; the remaining failure sits in the later `Game_Mode=$0008` bootstrap lineage just before the visible `Load_InGame` settle work.
- Because the task is still `evidence-only`, the right move was to make the selected replay smaller and cleaner, not to widen into speculative player-physics patches.

## Strongest Evidence

- `cargo run -p kidcli -- case verify --case death-respawn-derived-check`
  - result: `case death_respawn_level1_case_00000_00200: FAIL`
  - first failure: frame 161, `player.x_pos`, native `2621440`, oracle `0`
- `.cache/cases/death-respawn-derived-check/manifest.json`
  - result: selected bundle is now `frames: 0..162`
  - surviving non-zero inputs are only four `Start` taps at frames `33`, `77`, `135`, and `137`
- `cargo run -p kidcli -- case summarise --case death-respawn-derived-check`
  - result: refreshed summary now reports `frames: 0..162`
  - matching fields: `world.game_mode`, `camera`, `player.motion`, `entities`, `events`
- `cargo run -p kidcli -- compare-native-gameplay-window --scenario death-respawn-level1 --frame-start 150 --frame-end 170 --checkpoint-interval 1`
  - result: divergence on frame 161, `kid_x_pos` native `0x00280000`, oracle `0x00000000`
- ASM basis:
  - `loc_6E2` is the mode-reset + main-loop entry that hands control into `Load_InGame`
  - `Load_InGame` seeds `PlayerStart_X_pos` / `PlayerStart_Y_pos`, then snaps camera and runs hidden settle/bootstrap work before normal gameplay becomes visible
  - `Title_InputLoop` still owns the `Game_Mode=$0008` transition into that later bootstrap path

## Exact Next Moves

1. Inspect the native `BootState::StartingLevel` / opening-slice hand-off against the ROM-owned `loc_6E2 -> Load_InGame` bootstrap. The reduced case now isolates the remaining failure to the frame immediately before the settled bootstrap pose appears.
2. Trace why native still keeps Kid grounded at `0x0028_0000` on frame 161 while oracle clears Kid to zero even though `world.game_mode`, camera, and player motion still match.
3. If a gameplay patch is needed later, keep it scoped to the bootstrap/respawn hand-off only. Do not reopen the already-fixed title-menu timing.

## Verification To Re-run

- `cargo run -p kidcli -- case verify --case death-respawn-derived-check`
- `cargo run -p kidcli -- case summarise --case death-respawn-derived-check`
- `cargo run -p kidcli -- compare-native-gameplay-window --scenario death-respawn-level1 --frame-start 150 --frame-end 170 --checkpoint-interval 1`
- `cargo test -p kidcli reduction_skips_spans_that_are_already_zero`

## Dead Ends / Do Not Retry

- Do not reopen the already-fixed frame-136 title-menu work. This selected case now fails later, at frame 161.
- Do not widen into `lose_life`, `Teleport`, or late respawn gameplay logic yet. The reduced case still sits in the early `Game_Mode=$0008` bootstrap lineage.
- Do not rerun the pre-fix reducer binary logic mentally; the selected case only finished reducing after skipping no-op zero spans.
- Do not patch player physics or collision from the old `0..200` bundle. Use the reduced `0..162` bundle as the evidence base.
