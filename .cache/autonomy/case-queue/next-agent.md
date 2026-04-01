---
slug: death_respawn_level1_case_00000_00200
cycle: 29
updated_at: 2026-04-01T00:49:25Z
---

## What Changed

- Ran `cargo run -p kidcli -- autonomous-loop --goal case-queue`; cycle 29 kept case ID `death_respawn_level1_case_00000_00200` but switched the selected selector to `death-respawn-test`.
- Re-verified the selected case with `cargo run -p kidcli -- case verify --case death-respawn-test`; the live first failure is `player.x_pos` at frame 136, not the stale frame-0 `none` vs `0` report.
- Re-ran `cargo run -p kidcli -- case summarise --case death-respawn-test` after verify completed so `.cache/cases/death-respawn-test/summary.md` now matches `verify.json`.
- Read the relevant ROM path in `.cache/upstream/kid-chameleon-disasm/kid.asm`: `loc_1AE2E` only waits for Start, writes `Game_Mode=$000C`, and jumps to `loc_6E2`; `Load_InGame` is a later rebuild, so it does not explain the earlier frame-136 hidden-shell mismatch by itself.
- Reproduced the same failure with `cargo run -p kidcli -- compare-native-gameplay-window --scenario death-respawn-level1 --frame-start 130 --frame-end 140 --checkpoint-interval 1`; oracle exposes Kid at `(x=$28,y=$30)` on frame 136 while native still shows the blank shell.
- Did not patch gameplay code. The task package is still `evidence-only`, so this session only tightened the selected case and refreshed the handoff.

## Selected Case

- Case ID: `death_respawn_level1_case_00000_00200`
- Selector: `death-respawn-test`
- Autonomy gate: `evidence-only`
- Verify result: `FAIL`
- First bad frame: `136`
- First bad field: `player.x_pos`

## Why This Case

- The loop spec required taking only the selector from the refreshed `.cache/autonomy/case-queue/next-task.json`.
- Re-verification showed the selected case was already tighter than the task summary claimed; frame 0 was stale metadata, not the live frontier.
- Frame 136 is the earliest authoritative gameplay drift in this selected case, so it is the right place to resume without widening into later respawn windows.

## Strongest Evidence

- `.cache/cases/death-respawn-test/verify.json` now records `frame_index: 136`, `field: player.x_pos`, native `0`, oracle `2621440`.
- `.cache/cases/death-respawn-test/native-semantic.json` and `.cache/cases/death-respawn-test/oracle-semantic.json` agree through frame 135; the first semantic drift is frame 136 `player.x_pos` (`0` vs `0x0028_0000`).
- `cargo run -p kidcli -- compare-native-gameplay-window --scenario death-respawn-level1 --frame-start 130 --frame-end 140 --checkpoint-interval 1` fails at `frame_00136@136` with `kid X (native 00000000, oracle 00280000)`.
- The compare artefacts in `.cache/native-gameplay-oracle-runs/death_respawn_level1_window_00130_00140_step_1/` show oracle frame 136 at `kid_x_pos=0x0028_0000`, `kid_y_pos=0x0030_0000`, while native stays at zero for both.
- ROM `loc_1AE2E` and `Load_InGame` show the visible Start transition and later gameplay rebuild are distinct stages; the unresolved bug is the earlier hidden pose/visibility carry, not the settled respawn bootstrap.

## Exact Next Moves

1. Inspect the frame-136 lineage in [`trace.rs`](/home/connor/git/kc/crates/kid-core/src/trace.rs) and [`boot.rs`](/home/connor/git/kc/crates/kid-core/src/boot.rs), focusing on the hidden title-start / death-respawn carry before the later `Load_InGame` settle path.
2. Add a focused regression in [`tests.rs`](/home/connor/git/kc/crates/kid-core/src/tests.rs) proving that `death_respawn_level1` must expose Kid at `(0x0028_0000, 0x0030_0000)` on frame 136 while still before `level_settle@719`.
3. Only after that regression is red, patch the matching hidden-shell carry path and rerun the selected-case verification plus the frame-130..140 gameplay-window compare.

## Verification To Re-run

- `cargo run -p kidcli -- case verify --case death-respawn-test`
- `cargo run -p kidcli -- case summarise --case death-respawn-test`
- `cargo run -p kidcli -- compare-native-gameplay-window --scenario death-respawn-level1 --frame-start 130 --frame-end 140 --checkpoint-interval 1`
- `cargo test -p kid-core death_respawn_level_1_keeps_bootstrap_hidden_on_frame_200 -- --nocapture`
- `cargo test -p kid-core death_respawn_level_1_matches_load_ingame_handoff_through_frame_300 -- --nocapture`

## Dead Ends / Do Not Retry

- Do not trust the stale task summary that still says frame 0; the selected case now verifies to frame 136.
- Do not run `case summarise` in parallel with the first `case verify` if you need the refreshed summary immediately; summarise can race ahead and leave `summary.md` stale even though `verify.json` is correct.
- Do not patch the later `lose_life -> Teleport -> loc_6E2` respawn settle path for this case. The selected case diverges earlier, before the later grounded Level 1 pose at frame 300.
- Do not "fix" this by pulling the settled `Load_InGame` pose earlier. Oracle first exposes a different hidden pose `(x=$28,y=$30)`, not the later grounded `(x=$20,y=$10F)` respawn state.
