---
slug: death_respawn_level1_case_00000_00200
cycle: 47
updated_at: 2026-04-01T02:30:33Z
---

## What Changed

- Ran `cargo run -p kidcli -- autonomous-loop --goal case-queue`; cycle 47 selected `death_respawn_level1_case_00000_00200` again and kept it evidence-first.
- Re-verified the selected case with `cargo run -p kidcli -- case verify --case death-respawn-test`; it still fails on `player.x_pos`.
- Re-ran `cargo run -p kidcli -- case reduce --case death-respawn-test`; the reducer confirmed the current bundle is already at `173` continuation frames.
- Refreshed the reduced-case summary with `cargo run -p kidcli -- case summarise --case death-respawn-test`.
- Read the ROM respawn path in `.cache/upstream/kid-chameleon-disasm/kid.asm`: `lose_life`, `Teleport`, `loc_6E2`, and `Load_InGame`.
- Made no gameplay code change; the selector remains evidence-only.

## Selected Case

- Case ID: `death_respawn_level1_case_00000_00200`
- Selector: `death-respawn-test`
- Cycle: `47`
- Current result: `FAIL`
- Current first failure: frame `172`, field `player.x_pos`, native `0`, oracle `19398656`

## Why This Case

- The case queue selected this exact case again, so the session stayed on it only.
- The autonomy gate is still `evidence-only`, which means the correct move is to tighten or explain the case rather than widen into adjacent fixes.
- Re-reduction showed the bundle is already minimal enough for the current tooling, so the remaining work is to explain the respawn snap with tighter ASM-backed evidence.

## Strongest Evidence

- `cargo run -p kidcli -- autonomous-loop --goal case-queue`
  - cycle `47`
  - selected case still `death_respawn_level1_case_00000_00200`
  - summary now reports first failure at frame `172`
- `cargo run -p kidcli -- case verify --case death-respawn-test`
  - result: `case death_respawn_level1_case_00000_00200: FAIL`
- `.cache/cases/death-respawn-test/verify.json`
  - `frame_count: 173`
  - `first_failure.frame_index: 172`
  - `first_failure.field: player.x_pos`
  - `first_failure.native_value: "0"`
  - `first_failure.oracle_value: "19398656"`
- `.cache/cases/death-respawn-test/oracle-semantic.json`
  - frames `168..171` keep Kid hidden at `(0, 0)` in both native and oracle
  - frame `172` is the first oracle-only respawn snap to `x_pos=19398656` / `y_pos=16777216`
- `.cache/upstream/kid-chameleon-disasm/kid.asm`
  - `lose_life` decrements lives and jumps into `Teleport`
  - `Teleport` writes `Game_Mode=$0008` and jumps through `loc_B8DE -> loc_6E2`
  - `loc_6E2` clears VRAM/object slots and enters the main game-mode loop
  - `Load_InGame` is the ROM path that rebuilds level state before the player pose is restored

## Exact Next Moves

1. Start from the selected case only: `cargo run -p kidcli -- case verify --case death-respawn-test`.
2. Inspect the native respawn bootstrap around frame `172`, specifically why `PlayerLifecycle::Respawning` in [`trace.rs`](/home/connor/git/kc/crates/kid-core/src/trace.rs) still emits Kid at `(0,0)` when oracle has already landed on the `Load_InGame` snap.
3. Read the exact ROM routine that writes the respawn pose after `Load_InGame`; do not patch from the semantic diff alone.
4. If the queue still marks the selector evidence-only, tighten evidence around that single hand-off rather than widening into the later `player.hitpoints` case.
5. Only patch gameplay once the case is patch-allowed or the evidence isolates the missing native transition to one concrete respawn path.

## Verification To Re-run

- `cargo run -p kidcli -- autonomous-loop --goal case-queue`
- `cargo run -p kidcli -- case verify --case death-respawn-test`
- `cargo run -p kidcli -- case reduce --case death-respawn-test`
- `cargo run -p kidcli -- case summarise --case death-respawn-test`

## Dead Ends / Do Not Retry

- Do not trust the stale frame `136` claim from older task packages; current verification and the queue both place the first miss at frame `172`.
- Do not widen into `pause_resume_level1_case_00000_00200` or the later `death_respawn_level1_case_00300_00500` hitpoints case from this session.
- Do not patch gameplay from observation alone while the selector is still evidence-only.
- Do not assume the failure is generic physics/collision drift; the semantic evidence points to the respawn hand-off where oracle restores Kid’s pose and native does not.
