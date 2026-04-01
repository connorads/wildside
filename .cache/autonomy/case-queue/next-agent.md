---
slug: death_respawn_level1_case_00720_00900
cycle: 25
updated_at: 2026-04-01T00:28:20Z
---

## What Changed

- Ran `cargo run -p kidcli -- autonomous-loop --goal case-queue`; cycle 25 kept `death_respawn_level1_case_00720_00900` selected.
- Re-verified the selected case before editing with `cargo run -p kidcli -- case verify --case death-respawn-level1-720-900`; the bundle still failed, but the old `world.game_mode` evidence conflicted with a passing direct gameplay window compare.
- Re-read the ROM death shell in `.cache/upstream/kid-chameleon-disasm/kid.asm:14714` through `loc_B8DE`; `Teleport` still writes `Game_Mode = #8` before `jmp (j_loc_6E2).w`, so any real gameplay follow-up still needs ASM, not guesswork.
- Fixed [`crates/kidcli/src/case.rs`](/home/connor/git/kc/crates/kidcli/src/case.rs) so gameplay case derivation reuses `GameplayScenario::to_dynamic_window(...)`, matching the same slicing path used by `compare-native-gameplay-window`.
- Rebuilt the selected case in place with `cargo run -p kidcli -- case derive --gameplay-scenario death-respawn-level1 --frame-start 720 --frame-end 900 --out .cache/cases/death-respawn-level1-720-900`, then refreshed it with `case verify` and `case summarise`.

## Selected Case

- Case ID: `death_respawn_level1_case_00720_00900`
- Selector: `death-respawn-level1-720-900`
- Autonomy gate: `evidence-only`
- Verify result: `FAIL`
- First live failure: frame `720`, `player.movement_state`, native `movement_0002`, oracle `movement_0000`

## Why This Case

- The loop spec says to take only the selected case from `.cache/autonomy/case-queue/next-task.json`.
- The previous handoff had a stale `world.game_mode` failure at frame 720, but a fresh direct compare on the same gameplay window passed, so the useful work here was to tighten the case evidence rather than patch gameplay blindly.
- After the rebuild, the case now isolates a narrower live gameplay mismatch: movement state plus the paired `kid_collision_type` gap at the same first frame.

## Strongest Evidence

- `cargo run -p kidcli -- compare-native-gameplay-window --scenario death-respawn-level1 --frame-start 720 --frame-end 899 --checkpoint-interval 1` passes, proving the old frame-720 `world.game_mode` mismatch was a case-bundling problem, not live gameplay truth.
- `.cache/cases/death-respawn-level1-720-900/verify.json` now records `player.movement_state` as the first failure at frame `720`; `world.game_mode`, `camera`, and `player.position` now match.
- `.cache/cases/death-respawn-level1-720-900/native-semantic.json` and `oracle-semantic.json` both show `world.game_mode = 12` and the same frame-720 pose, but native still reports `movement_0002` while oracle reports `movement_0000`; oracle also still has `kid_collision_type = 0` where native leaves it unset.
- The new regression test `case::tests::gameplay_case_verification_scenario_matches_canonical_window_builder` locks gameplay case derivation to the canonical window slicer.

## Exact Next Moves

1. Inspect the frame-720 movement-state and collision-type sources in native gameplay capture versus oracle gameplay probe before touching gameplay logic; the remaining mismatch may still be normalisation, not engine behaviour.
2. Read the ROM movement / floor-contact path that governs standing-vs-walk state on flat ground before any gameplay patch; the next agent should treat `movement_0000` at this frame as the oracle requirement to explain, not assume.
3. Keep the work scoped to `death_respawn_level1_case_00720_00900`; do not reopen the earlier `world.game_mode` failure unless the refreshed artefacts regress.

## Verification To Re-run

- `cargo test -p kidcli gameplay_case_verification_scenario_matches_canonical_window_builder`
- `cargo run -p kidcli -- case derive --gameplay-scenario death-respawn-level1 --frame-start 720 --frame-end 900 --out .cache/cases/death-respawn-level1-720-900`
- `cargo run -p kidcli -- case verify --case death-respawn-level1-720-900`
- `cargo run -p kidcli -- case summarise --case death-respawn-level1-720-900`
- `cargo run -p kidcli -- compare-native-gameplay-window --scenario death-respawn-level1 --frame-start 720 --frame-end 899 --checkpoint-interval 1`
- `sed -n '14714,14830p' .cache/upstream/kid-chameleon-disasm/kid.asm`

## Dead Ends / Do Not Retry

- Do not patch `Game_Mode` timing for this case from the old bundle; that mismatch was stale case slicing, not current evidence.
- Do not assume the remaining failure is gameplay code yet; native/oracle still differ only in `player.movement_state` and `kid_collision_type`, which may be capture/normalisation.
- Do not widen to other death-respawn windows in this session; the selected case is now tighter and should stay the sole frontier.
