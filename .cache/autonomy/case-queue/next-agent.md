---
slug: death_respawn_level1_case_00000_00200
cycle: 49
updated_at: 2026-04-01T02:36:50Z
---

## What Changed

- Ran `cargo run -p kidcli -- autonomous-loop --goal case-queue`; cycle `49` selected `death_respawn_level1_case_00000_00200` again and still marked it evidence-first.
- Re-verified the selected case with `cargo run -p kidcli -- case verify --case death-respawn-test`; it still fails at frame `172` on `player.x_pos`.
- Refreshed the reduced-case summary with `cargo run -p kidcli -- case summarise --case death-respawn-test`.
- Tried `cargo run -p kidcli -- case reduce --case death-respawn-test`; it could not shrink the bundle below `173` continuation frames while preserving the same frame-172 `player.x_pos` failure.
- Re-read the ROM respawn path in `.cache/upstream/kid-chameleon-disasm/kid.asm`: `lose_life`, `Teleport`, `loc_B8DE -> loc_6E2`, and `Load_InGame`.
- Traced the native visibility gate in [`boot.rs`](/home/connor/git/kc/crates/kid-core/src/boot.rs) and [`trace.rs`](/home/connor/git/kc/crates/kid-core/src/trace.rs): respawn always seeds `PlayerLifecycle::Respawning { remaining_frames: PLAYER_DEATH_RESPAWN_SETTLE_FRAMES }`, and trace keeps the whole lifecycle in hidden mode-8 handling.
- Cross-checked the existing full-scenario respawn tests in [`tests.rs`](/home/connor/git/kc/crates/kid-core/src/tests.rs); they still encode a hidden Level 1 bootstrap long after the reduced bundle's local frame `172`.
- Made no gameplay code change; the case is still evidence-only.

## Selected Case

- Case ID: `death_respawn_level1_case_00000_00200`
- Selector: `death-respawn-test`
- Cycle: `49`
- Current result: `FAIL`
- Current first failure: frame `172`, field `player.x_pos`, native `0`, oracle `19398656`

## Why This Case

- The autonomous selector chose the same case again, so the session stayed on this case only.
- The task package still says `evidence-only`, which rules out speculative gameplay patches.
- The strongest remaining gap is no longer "which subsystem?"; it is the narrower timing question of why the visible `Load_InGame` pose appears in the reduced case while native still exposes the long shared respawn shell.

## Strongest Evidence

- `cargo run -p kidcli -- autonomous-loop --goal case-queue`
  - cycle `49`
  - status `stalled`
  - selected case still `death_respawn_level1_case_00000_00200`
- `cargo run -p kidcli -- case verify --case death-respawn-test`
  - result: `case death_respawn_level1_case_00000_00200: FAIL`
- `cargo run -p kidcli -- case reduce --case death-respawn-test`
  - result: `reduced case death_respawn_level1_case_00000_00200 to 173 continuation frames`
  - implication: this case is already minimal for that exact semantic signature; more progress needs better explanation, not another tail trim
- `.cache/cases/death-respawn-test/verify.json`
  - `frame_count: 173`
  - `first_failure.frame_index: 172`
  - `first_failure.field: player.x_pos`
  - `first_failure.native_value: "0"`
  - `first_failure.oracle_value: "19398656"`
- `.cache/cases/death-respawn-test/oracle-semantic.json`
  - frames `168..171`: Kid stays hidden at `(0,0)` with `game_mode=8`
  - frame `172`: oracle snaps to `x_pos=19398656` / `y_pos=16777216` while camera stays `(0,0)`
- `.cache/upstream/kid-chameleon-disasm/kid.asm`
  - `Teleport` writes `Game_Mode=$0008`
  - `loc_B8DE` jumps to `loc_6E2`
  - `Load_InGame` reads the level header, writes `PlayerStart_X_pos` / `PlayerStart_Y_pos`, and then snaps `Camera_X_pos` / `Camera_Y_pos` from those start positions
- [`crates/kid-core/src/boot.rs`](/home/connor/git/kc/crates/kid-core/src/boot.rs)
  - `PlayerLifecycle::Dying` falls into `reset_current_level_runtime_state_for_respawn()`
  - native then seeds `PlayerLifecycle::Respawning { remaining_frames: PLAYER_DEATH_RESPAWN_SETTLE_FRAMES }`
  - `PlayerLifecycle::Respawning` only decrements the shared settle timer; it has no earlier visible `Load_InGame` return for this case window
- [`crates/kid-core/src/trace.rs`](/home/connor/git/kc/crates/kid-core/src/trace.rs)
  - `current_player_state()` prefers `death_respawn_bootstrap_player_state()` during respawn
  - `current_camera_state()` returns cleared camera for all respawn lifecycle frames
- [`crates/kid-core/src/tests.rs`](/home/connor/git/kc/crates/kid-core/src/tests.rs)
  - existing tests `death_respawn_level_1_keeps_respawn_bootstrap_hidden_on_frame_1791` and `death_respawn_level_1_uses_hidden_respawn_bootstrap_path_through_frame_1855` encode a much later hidden bootstrap window on the full scenario
  - that is the current contradiction: the reduced case wants the visible snap by local frame `172`, but the full-scenario trace tests still insist the shared bootstrap remains hidden much later

## Exact Next Moves

1. Re-run `cargo run -p kidcli -- case verify --case death-respawn-test` to keep the local frame-172 failure pinned before any patch.
2. Explain the reduction/full-scenario contradiction before changing gameplay: map local case frame `172` back onto the full `death_respawn_level_1_script()` timeline and confirm whether the reduced case is legitimately isolating a different lineage or whether the semantic case is over-collapsing the hidden shell.
3. If the contradiction survives that mapping, inspect the case-bundle derivation/reduction assumptions rather than patching `PlayerLifecycle::Respawning` from contradictory evidence.
4. Read the exact ROM path that restores Kid's visible pose after `Load_InGame`; do not infer it from the semantic snapshot alone.
5. Only patch gameplay after the earlier visible hand-off is justified against both ASM and the full-scenario trace/tests.

## Verification To Re-run

- `cargo run -p kidcli -- autonomous-loop --goal case-queue`
- `cargo run -p kidcli -- case verify --case death-respawn-test`
- `cargo run -p kidcli -- case summarise --case death-respawn-test`

## Dead Ends / Do Not Retry

- Do not widen into `pause_resume_level1_case_00000_00200` or `death_respawn_level1_case_00300_00500`; this session is only about `death_respawn_level1_case_00000_00200`.
- Do not patch from `oracle-semantic.json` alone. The visible frame-172 snap must be reconciled with the ROM path and the existing full-scenario respawn tests first.
- Do not assume the previous "player physics/collision" subsystem hint is the real cause. The tighter hypothesis is a respawn visibility/timing gate inside the shared `PlayerLifecycle::Respawning` path.
- Do not re-run `case reduce` expecting an even smaller bundle. Cycle `49` already confirmed this bundle is minimal for the current failure signature.
- Do not add new debug fields yet. The current gap is still explainable with the existing case artefacts, ASM, and native trace/lifecycle code.
