---
slug: death_respawn_level1_case_00000_00200
cycle: 42
updated_at: 2026-04-01T02:11:11Z
---

## What Changed

- Ran `cargo run -p kidcli -- autonomous-loop --goal case-queue`; cycle 42 selected the same case `death_respawn_level1_case_00000_00200` (`death-respawn-derived-check`).
- Re-verified the selected case before any gameplay work; it still fails first on frame 161 at `player.x_pos`.
- Re-read the ROM bootstrap path before changing anything gameplay-facing:
  - `.cache/upstream/kid-chameleon-disasm/kid.asm`: `loc_6E2`, `Load_InGame`, `Obj_TitleMenu`, `Title_InputLoop`
- Re-ran `cargo run -p kidcli -- case reduce --case death-respawn-derived-check`; the reducer plateaued at the same 162-frame continuation, so this case is presently as tight as the tool can make it without a code fix.
- Re-ran `compare-native-gameplay-window` for frames `150..170`; the first drift still happens on frame 161 before `level_settle@719`.
- No gameplay code changed this cycle. The autonomy gate is still `evidence-only`, and the new work is a cleaner plateau report for the same selected case.

## Selected Case

- Case ID: `death_respawn_level1_case_00000_00200`
- Selector: `death-respawn-derived-check`
- Autonomy gate: `evidence-only`
- Current verify result: `FAIL`

## Why This Case

- The case queue selected this exact case again, so the session stayed on that case only.
- The remaining failure is no longer in title-menu entry timing; it is the next bootstrap hand-off frame after `Title_InputLoop` writes `Game_Mode=$0008` and jumps through `loc_6E2`.
- Because reduction now plateaus at `0..162`, the best progress is to preserve the tighter evidence and point the next session at the specific native hand-off code that still over-exposes Kid on frame 161.

## Strongest Evidence

- `cargo run -p kidcli -- case verify --case death-respawn-derived-check`
  - result: `case death_respawn_level1_case_00000_00200: FAIL`
  - first failure: frame 161, `player.x_pos`, native `2621440`, oracle `0`
- `cargo run -p kidcli -- case reduce --case death-respawn-derived-check`
  - result: `reduced case death_respawn_level1_case_00000_00200 to 162 continuation frames`
  - outcome: no further shrink beyond the existing `0..162` bundle
- `.cache/cases/death-respawn-derived-check/manifest.json`
  - result: selected bundle remains `frames: 0..162`
  - surviving non-zero inputs are still only four `Start` taps at frames `33`, `77`, `135`, and `137`
- `cargo run -p kidcli -- compare-native-gameplay-window --scenario death-respawn-level1 --frame-start 150 --frame-end 170 --checkpoint-interval 1`
  - result: divergence on frame 161 before `level_settle@719`
  - native keeps Kid at `kid_x_pos=0x00280000`
  - oracle has already cleared Kid to `kid_x_pos=0x00000000`
  - `world.game_mode`, camera, and player motion still match
- ASM basis:
  - `Title_InputLoop` writes `Game_Mode=$0008`, sets `$FFFFFBCE`, then jumps to `loc_6E2`
  - `loc_6E2` clears VRAM/object state and dispatches through the main loop
  - `Load_InGame` seeds `PlayerStart_X_pos` / `PlayerStart_Y_pos`, snaps camera from those values, then runs hidden bootstrap/setup work before visible gameplay

## Exact Next Moves

1. Inspect the native `BootState::StartingLevel` hand-off in `crates/kid-core/src/boot.rs` against the ROM-owned `Title_InputLoop -> loc_6E2 -> Load_InGame` sequence. The next fix, if allowed, should live there rather than in generic movement/collision code.
2. Trace why native still exposes the title-menu cursor X position (`0x0028_0000`) on frame 161 even though oracle has already cleared Kid to zero on that same frame.
3. If a later cycle upgrades this case from `evidence-only`, patch only the bootstrap/respawn hand-off and immediately re-run the selected case plus the `150..170` gameplay window compare.

## Verification To Re-run

- `cargo run -p kidcli -- case verify --case death-respawn-derived-check`
- `cargo run -p kidcli -- case summarise --case death-respawn-derived-check`
- `cargo run -p kidcli -- case reduce --case death-respawn-derived-check`
- `cargo run -p kidcli -- compare-native-gameplay-window --scenario death-respawn-level1 --frame-start 150 --frame-end 170 --checkpoint-interval 1`

## Dead Ends / Do Not Retry

- Do not reopen the already-fixed frame-136 title-menu cursor regression. This selected case now fails later, at frame 161.
- Do not widen into generic player physics or collision. The compare window still says the first drift happens before `level_settle@719`.
- Do not expect another reducer pass to shrink this bundle further without changing behaviour; cycle 42 re-ran the reducer and it plateaued at the same 162-frame continuation.
- Do not jump ahead to late death/respawn routines yet. This case is still about the early `Game_Mode=$0008` bootstrap lineage selected by the queue.
