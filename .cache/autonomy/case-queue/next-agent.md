---
slug: death_respawn_level1_case_00000_00200
cycle: 46
updated_at: 2026-04-01T02:26:54Z
---

## What Changed

- Ran `cargo run -p kidcli -- autonomous-loop --goal case-queue`; cycle 46 selected `death_respawn_level1_case_00000_00200` again and kept it evidence-first.
- Re-verified the selected case with `cargo run -p kidcli -- case verify --case death-respawn-test`; it still fails, so the task package is live.
- Read the ROM respawn hand-off in `.cache/upstream/kid-chameleon-disasm/kid.asm`: `lose_life`, `Teleport`, `loc_6E2`, and `Load_InGame` before considering any gameplay change.
- Tightened the selected case instead of patching gameplay by running `cargo run -p kidcli -- case reduce --case death-respawn-test`; the bundle now preserves the same failure in `173` frames instead of `200`.
- Refreshed the reduced-case summary with `cargo run -p kidcli -- case summarise --case death-respawn-test`.
- Made no gameplay code change; the autonomy gate for this selector remains evidence-only.

## Selected Case

- Case ID: `death_respawn_level1_case_00000_00200`
- Selector: `death-respawn-test`
- Cycle: `46`
- Current result: `FAIL`
- Current first failure: frame `172`, field `player.x_pos`, native `0`, oracle `19398656`

## Why This Case

- The loop selected this exact case, so the session stayed on it only.
- The task package marked it `evidence-only`, which means tighten the case rather than widen into adjacent fixes.
- Re-verification confirmed the bug still reproduces, but not at the stale frame `136`; reducing the case gave a smaller and more current bundle for the next investigation.

## Strongest Evidence

- `cargo run -p kidcli -- case verify --case death-respawn-test`
  - result: `case death_respawn_level1_case_00000_00200: FAIL`
- `.cache/cases/death-respawn-test/verify.json`
  - `frame_count: 173`
  - `first_failure.frame_index: 172`
  - `first_failure.field: player.x_pos`
  - `first_failure.native_value: "0"`
  - `first_failure.oracle_value: "19398656"`
- `.cache/cases/death-respawn-test/summary.md`
  - `frames: 0..173`
  - `matching_fields: world.game_mode, camera, player.motion, entities, events`
- `.cache/upstream/kid-chameleon-disasm/kid.asm`
  - `lose_life` decrements lives, resets hitpoints, then enters `Teleport`
  - `Teleport` sets `Game_Mode=$0008` and jumps through `loc_6E2`
  - `Load_InGame` reloads level state and player start positions after that mode reset

## Exact Next Moves

1. Start from the tightened case only: `cargo run -p kidcli -- case verify --case death-respawn-test`.
2. Compare native and oracle semantic state around frame `172`, especially why native still exposes `player.x_pos=0` after the `loc_6E2 -> Load_InGame` hand-off.
3. Derive or inspect a tighter window around the surviving failure if the gate still says evidence-only.
4. Read the exact ROM path that rebuilds Kid's gameplay object after `Load_InGame`; do not patch from observation alone.
5. Only move to gameplay code once the queue promotes this selector to patch-allowed or the failure is tight enough to justify a focused fix.

## Verification To Re-run

- `cargo run -p kidcli -- autonomous-loop --goal case-queue`
- `cargo run -p kidcli -- case verify --case death-respawn-test`
- `cargo run -p kidcli -- case summarise --case death-respawn-test`

## Dead Ends / Do Not Retry

- Do not trust the stale `first_bad_frame: 136` claim from older task packages; current verification moves the first failure to frame `172`.
- Do not widen into pause/resume or the later hitpoints case from this session.
- Do not patch gameplay from the reduced case alone while the selector is still evidence-only.
