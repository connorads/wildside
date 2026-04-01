---
slug: death_respawn_level1_case_00720_00900
cycle: 27
updated_at: 2026-04-01T00:39:48Z
---

## What Changed

- Ran `cargo run -p kidcli -- autonomous-loop --goal case-queue`; cycle 27 kept `death_respawn_level1_case_00720_00900` selected.
- Re-verified the selected case with `cargo run -p kidcli -- case verify --case death-respawn-level1-720-900`; it failed first at frame 720 before the patch and passes after it.
- Read the grounded movement ROM path in `.cache/upstream/kid-chameleon-disasm/kid.asm`: `loc_8DFC` and `loc_8E16` both write `MoveID_Standingstill` when grounded movement settles at zero `x_vel`.
- Patched [`crates/kid-core/src/trace.rs`](/home/connor/git/kc/crates/kid-core/src/trace.rs) so `current_character_movement()` now reports standing still on grounded zero-velocity frames before considering walking or slope movement.
- Added a focused regression in [`crates/kid-core/src/tests.rs`](/home/connor/git/kc/crates/kid-core/src/tests.rs) covering frame 720 of `death_respawn_level_1_script()`.

## Selected Case

- Case ID: `death_respawn_level1_case_00720_00900`
- Selector: `death-respawn-level1-720-900`
- Autonomy gate at start: `evidence-only`
- Verify result now: `PASS`
- Gameplay window result now: `PASS`

## Why This Case

- The loop spec required taking only the selected case from `.cache/autonomy/case-queue/next-task.json`.
- The reduced case was already tight: frame 720 matched oracle on position, camera, entities, and events, leaving only `player.movement_state`.
- Native frame-720 capture already had grounded support and zero `x_vel`, so the remaining mismatch was trace derivation, not a wider gameplay state frontier.

## Strongest Evidence

- `cargo run -p kidcli -- case verify --case death-respawn-level1-720-900` now reports `PASS`.
- `cargo run -p kidcli -- compare-native-gameplay-window --scenario death-respawn-level1 --frame-start 720 --frame-end 899 --checkpoint-interval 1` now passes for the full case window.
- `.cache/cases/death-respawn-level1-720-900/summary.md` now records `passed: true`.
- ROM `loc_8DFC` / `loc_8E16` writes `MoveID_Standingstill` on the grounded zero-velocity path, which matches the oracle frame-720 expectation.

## Exact Next Moves

1. Run `cargo run -p kidcli -- autonomous-loop --goal case-queue` again to select the next case now that this one passes.
2. Read the refreshed `.cache/autonomy/case-queue/next-task.json`.
3. Take that selected case only; re-verify it before any code change.

## Verification To Re-run

- `cargo test -p kid-core death_respawn_level_1_reports_standingstill_on_frame_720`
- `cargo run -p kidcli -- case verify --case death-respawn-level1-720-900`
- `cargo run -p kidcli -- case summarise --case death-respawn-level1-720-900`
- `cargo run -p kidcli -- compare-native-gameplay-window --scenario death-respawn-level1 --frame-start 720 --frame-end 899 --checkpoint-interval 1`

## Dead Ends / Do Not Retry

- Do not reopen this case by inferring walking from held left/right input alone on grounded zero-velocity frames; the ROM standing-still write happens before the walking continuation.
- Do not widen this fix into death-shell or respawn-shell gameplay; the selected case resolved entirely in trace movement reporting.
- Do not treat the unrelated `demo_1_reenters_walking_from_standingstill_on_frame_3252` failure from this session as evidence against the fix without investigating that scenario separately; its observed failure was a large frame-3251 position mismatch, not the selected case frontier.
