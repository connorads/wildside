---
slug: death_respawn_level1_case_00720_00900
cycle: 26
updated_at: 2026-04-01T00:35:16Z
---

## What Changed

- Ran `cargo run -p kidcli -- autonomous-loop --goal case-queue`; cycle 26 kept `death_respawn_level1_case_00720_00900` selected and still marked it `evidence-only`.
- Re-verified the selected case with `cargo run -p kidcli -- case verify --case death-respawn-level1-720-900`; it still fails first at frame 720 on `player.movement_state`.
- Read the grounded movement ASM in `.cache/upstream/kid-chameleon-disasm/kid.asm:10188` and `.cache/upstream/kid-chameleon-disasm/kid.asm:10212`; `loc_8DFC` and `loc_8E16` both write `MoveID_Standingstill` when grounded movement settles with zero horizontal motion.
- Tightened gameplay evidence in [`crates/kidcli/src/compare.rs`](/home/connor/git/kc/crates/kidcli/src/compare.rs) so gameplay checkpoint comparison and live-drift reporting now include `character_movement`.
- Added regression tests proving checkpoint comparison and drift reporting surface `character_movement` mismatches.
- Re-ran `cargo run -p kidcli -- compare-native-gameplay-window --scenario death-respawn-level1 --frame-start 720 --frame-end 899 --checkpoint-interval 1`; it now fails on the same frame/field as the selected case instead of falsely passing.

## Selected Case

- Case ID: `death_respawn_level1_case_00720_00900`
- Selector: `death-respawn-level1-720-900`
- Autonomy gate: `evidence-only`
- Verify result: `FAIL`
- First live failure: frame `720`, `player.movement_state`, native `movement_0002`, oracle `movement_0000`

## Why This Case

- The loop spec says to take only the selected case from `.cache/autonomy/case-queue/next-task.json`.
- The main confusion on the previous cycle was that direct gameplay-window comparison passed while the reduced case failed.
- This cycle resolved that contradiction: the gameplay comparer was not checking `character_movement`, so the case was tighter than the live comparison tooling.

## Strongest Evidence

- `cargo run -p kidcli -- case verify --case death-respawn-level1-720-900` still fails with `player.movement_state` at frame 720.
- `cargo run -p kidcli -- compare-native-gameplay-window --scenario death-respawn-level1 --frame-start 720 --frame-end 899 --checkpoint-interval 1` now fails with the same `frame_00720@720` `character movement` mismatch and reports the first live drift at frame 720.
- `.cache/cases/death-respawn-level1-720-900/native-semantic.json` still records native `movement_0002`; `.cache/cases/death-respawn-level1-720-900/oracle-semantic.json` records oracle `movement_0000`.
- ASM `.cache/upstream/kid-chameleon-disasm/kid.asm:10199` (`loc_8DFC`) and `.cache/upstream/kid-chameleon-disasm/kid.asm:10229` (`loc_8E16`) show the grounded zero-x-velocity path writing `MoveID_Standingstill`, which matches the oracle expectation to explain next.

## Exact Next Moves

1. Inspect the native frame-720 gameplay trace around the grounded movement transition and find why `current_character_movement()` still returns walking while the ROM path has already settled to standing still.
2. Read the surrounding ROM path from `loc_8DFC` through `loc_8E4E` together with the corresponding native grounded-movement code before changing gameplay.
3. Add one focused `kid-core` regression test for the frame-720 standing-still expectation before patching gameplay logic.

## Verification To Re-run

- `cargo test -p kidcli compare_gameplay_checkpoints_reports_character_movement_drift`
- `cargo test -p kidcli gameplay_differing_fields_reports_character_movement_drift`
- `cargo test -p kidcli gameplay_case_verification_scenario_matches_canonical_window_builder`
- `cargo run -p kidcli -- case verify --case death-respawn-level1-720-900`
- `cargo run -p kidcli -- case summarise --case death-respawn-level1-720-900`
- `cargo run -p kidcli -- compare-native-gameplay-window --scenario death-respawn-level1 --frame-start 720 --frame-end 899 --checkpoint-interval 1`

## Dead Ends / Do Not Retry

- Do not trust an old `compare-native-gameplay-window` pass on this case; before this cycle it ignored `character_movement`.
- Do not widen the work to `kid_collision_type` yet; native gameplay capture still leaves that field unset, so it is not ready to drive pass/fail parity for this case.
- Do not patch death-shell or respawn-shell logic from this evidence; the remaining authoritative mismatch is earlier, at grounded movement frame 720.
