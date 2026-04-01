---
slug: death_respawn_level1_case_00000_00200
cycle: 28
updated_at: 2026-04-01T01:04:00Z
---

## What Changed

- Ran `cargo run -p kidcli -- autonomous-loop --goal case-queue`; cycle 28 selected `death_respawn_level1_case_00000_00200` (`death-respawn-derived-check`).
- Re-verified the selected case with `cargo run -p kidcli -- case verify --case death-respawn-derived-check`; it still fails, but the refreshed case artefacts now move the first failure from stale frame 0 to frame 136.
- Refreshed the case summary with `cargo run -p kidcli -- case summarise --case death-respawn-derived-check`; the live failure is now `player.x_pos` at frame 136 with native `0` versus oracle `2621440` (`0x0028_0000`).
- Read the relevant ROM path in `.cache/upstream/kid-chameleon-disasm/kid.asm`: `loc_1AE2E` only waits for Start, writes `Game_Mode=$000C`, and jumps to `loc_6E2`; `Load_InGame` seeds the gameplay header pose later, so the frame-136 divergence is not explained by the later gameplay rebuild alone.
- Ran `cargo run -p kidcli -- compare-native-gameplay-window --scenario death-respawn-level1 --frame-start 130 --frame-end 140 --checkpoint-interval 1`; oracle exposes Kid at `(x=$28,y=$30)` on frame 136 while native keeps the blank shell through frame 140.
- Did not patch gameplay code. The autonomy gate is still `evidence-only`, and the remaining mismatch is now a tighter title-screen carry/frontier rather than the stale frame-0 report.

## Selected Case

- Case ID: `death_respawn_level1_case_00000_00200`
- Selector: `death-respawn-derived-check`
- Autonomy gate at start: `evidence-only`
- Verify result now: `FAIL`
- First bad frame now: `136`
- First bad field now: `player.x_pos`

## Why This Case

- The loop spec required taking only the selected case from `.cache/autonomy/case-queue/next-task.json`.
- Re-verification materially tightened the case without widening scope: the original frame-0 `none`/`0` mismatch was stale cache, not the live frontier.
- The remaining frame-136 mismatch is early enough to isolate a single lineage and should be resolved before touching later death/respawn windows that duplicate the same hidden-start semantics.

## Strongest Evidence

- `cargo run -p kidcli -- case verify --case death-respawn-derived-check` still fails, but now reports the first failure at frame 136 instead of frame 0.
- `.cache/cases/death-respawn-derived-check/summary.md` now records `first_bad_frame: 136`, `native_value: 0`, and `oracle_value: 2621440`.
- `cargo run -p kidcli -- compare-native-gameplay-window --scenario death-respawn-level1 --frame-start 130 --frame-end 140 --checkpoint-interval 1` reports `kid X (native 00000000, oracle 00280000)` on `frame_00136@136`.
- The compare artefacts in `.cache/native-gameplay-oracle-runs/death_respawn_level1_window_00130_00140_step_1/` show:
- Oracle frame 136: `game_mode=44`, `kid_x_pos=0x0028_0000`, `kid_y_pos=0x0030_0000`, `number_gfx_objects=3`
- Native frame 136: `game_mode=44`, `kid_x_pos=0`, `kid_y_pos=0`, `number_gfx_objects=0`
- ROM `loc_1AE2E` and `Load_InGame` confirm the later gameplay rebuild is a separate step, so the live divergence is now a title-screen carry/visibility problem before the normal settled Level 1 gameplay pose at frame 300.

## Exact Next Moves

1. Re-open the frame-136 lineage in code, not the later respawn shell: inspect the title-screen and early-start visibility branches in [`crates/kid-core/src/trace.rs`](/home/connor/git/kc/crates/kid-core/src/trace.rs) and [`crates/kid-core/src/boot.rs`](/home/connor/git/kc/crates/kid-core/src/boot.rs).
2. Add a focused regression that proves the `death_respawn_level1` route must expose Kid at `(0x0028_0000, 0x0030_0000)` on frame 136 while still on the hidden `$002C` shell before any gameplay patch.
3. Only after that regression is red, patch the matching title-screen carry path and rerun the selected-case verification plus the frame-130..140 gameplay-window compare.

## Verification To Re-run

- `cargo run -p kidcli -- case verify --case death-respawn-derived-check`
- `cargo run -p kidcli -- case summarise --case death-respawn-derived-check`
- `cargo run -p kidcli -- compare-native-gameplay-window --scenario death-respawn-level1 --frame-start 130 --frame-end 140 --checkpoint-interval 1`
- `cargo test -p kid-core death_respawn_level_1_keeps_bootstrap_hidden_on_frame_200 -- --nocapture`
- `cargo test -p kid-core death_respawn_level_1_matches_load_ingame_handoff_through_frame_300 -- --nocapture`

## Dead Ends / Do Not Retry

- Do not treat the stale frame-0 `player.x_pos` report as the live bug; re-verification already moved the frontier to frame 136.
- Do not patch the later `lose_life -> Teleport -> loc_6E2` respawn shell for this case. The selected case lives before `level_settle@719`, and the frame-136 divergence happens under `Game_Mode=$002C`.
- Do not "fix" this by shifting the settled `Load_InGame` frame-300 ramp earlier. Oracle is exposing a different hidden pose first: `(x=$28,y=$30)`, not the later grounded `(x=$20,y=$10F)` gameplay state.
