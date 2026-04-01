---
slug: death_respawn_level1_case_00200_00400
cycle: 32
updated_at: 2026-04-01T00:56:52Z
---

## What Changed

- Ran `cargo run -p kidcli -- autonomous-loop --goal case-queue` for cycle 32 and treated `.cache/autonomy/case-queue/next-task.json` as the authoritative selector package.
- Re-verified the selected case with `cargo run -p kidcli -- case verify --case death-respawn-200-400`; it now passes, so the prior frame-200 `player.x_pos` failure is no longer reproducible on the current tree.
- Refreshed `cargo run -p kidcli -- case summarise --case death-respawn-200-400`; the case summary now records a fully matching 200..400 window across `world.game_mode`, `camera`, `player.position`, `player.motion`, `entities`, and `events`.
- Read the ROM path named by the existing native regression coverage before considering any gameplay edit: `lose_life -> Teleport -> loc_6E2` for the death/respawn transition, plus `Load_TitleCard` and `loc_1AE2E -> loc_6E2 -> Load_InGame` for the hidden title-card bootstrap and later in-game rebuild that this reduced window now matches.
- Did not patch gameplay code. The selected case cleared during mandatory re-verification, so the only durable change this session is the refreshed handoff note.

## Selected Case

- Case ID: `death_respawn_level1_case_00200_00400`
- Selector: `death-respawn-200-400`
- Autonomy gate: `evidence-only`
- Verify result: `PASS`
- Matching window: `200..400`

## Why This Case

- The case-queue loop says to run the selector, read `.cache/autonomy/case-queue/next-task.json`, and work only the selected case.
- Cycle 32 selected `death_respawn_level1_case_00200_00400`, and the task package flagged the previous handoff note as stale because it still referenced a pause/resume case.
- Because the selector package was evidence-only, the correct first step was re-verification rather than widening scope or patching code.

## Strongest Evidence

- `.cache/cases/death-respawn-200-400/verify.json` now reports `"passed": true` with `first_failure: null`.
- `.cache/cases/death-respawn-200-400/summary.md` now records the whole 200..400 slice as matching on gameplay-critical fields, which satisfies the package’s requirement that the first bad frame/field be removed or moved later with a better explanation.
- ROM `lose_life` / `Teleport` sets `Game_Mode=$0008` and jumps to `loc_6E2`, matching the hidden respawn shell described in native trace comments for this scenario.
- ROM `Load_TitleCard` allocates `loc_1AB26` and `loc_1AE2E`; `loc_1AE2E` waits for Start, writes `Game_Mode=$000C`, sets `$FFFFFBCE`, and jumps to `loc_6E2`, after which `Load_InGame` rebuilds the live level state.
- Existing regression tests in [tests.rs](/home/connor/git/kc/crates/kid-core/src/tests.rs#L527) and [tests.rs](/home/connor/git/kc/crates/kid-core/src/tests.rs#L553) already lock down this exact ROM-backed hand-off: hidden bootstrap still blank at frame 200/222, settled Level 1 spawn visible by frame 300.

## Exact Next Moves

1. Run `cargo run -p kidcli -- autonomous-loop --goal case-queue` again in the next session so the selector can promote a different open case now that `death-respawn-200-400` re-verifies cleanly.
2. If cycle 33 still selects this same case, inspect why the selector artefacts were not refreshed despite `verify.json` showing a pass before doing any gameplay work.
3. Keep death/respawn work scoped to a newly selected case only; do not widen back to the parent `0..500` window unless the selector explicitly returns it.

## Verification To Re-run

- `cargo run -p kidcli -- case verify --case death-respawn-200-400`
- `cargo run -p kidcli -- case summarise --case death-respawn-200-400`
- `cargo test -p kid-core death_respawn_level_1_keeps_bootstrap_hidden_on_frame_200 death_respawn_level_1_matches_load_ingame_handoff_through_frame_300`

## Dead Ends / Do Not Retry

- Do not patch gameplay code for `death-respawn-200-400` without first reproducing a fresh failure; this session’s mandatory re-verify passed.
- Do not ignore the ROM path and patch from trace comments alone. The relevant ground truth is `lose_life` / `Teleport` and `Load_TitleCard` / `loc_1AE2E` / `Load_InGame` in `kid.asm`.
- Do not keep the stale pause/resume handoff note; cycle 32’s selected case is the death/respawn 200..400 window.
