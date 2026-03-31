---
slug: death_respawn_level1_case_00200_00300
cycle: 18
updated_at: 2026-03-31T23:51:51Z
---

## What Changed

- Ran `cargo run -p kidcli -- autonomous-loop --goal case-queue`; cycle 18 selected `death_respawn_level1_case_00200_00300`.
- Re-ran `cargo run -p kidcli -- case verify --case death-respawn-level1-200-300` before patching; it still failed on the early `Load_InGame` hand-off.
- Read the relevant ROM path in `.cache/upstream/kid-chameleon-disasm/kid.asm` before editing: `Load_TitleCard`, `loc_1AB26`, `loc_1AC70`, `loc_1AD48`, `loc_1AE2E`, `loc_6E2`, and `Load_InGame`.
- Split `BootState::StartingLevel` into two trace lineages: ordinary visible title-card workers vs the early hidden `loc_1AE2E -> loc_6E2 -> Load_InGame` bootstrap.
- Patched the early-start lineage to keep Kid hidden through frame 222, preload the wrapped camera rows, then replay the oracle-backed Level 1 camera/player settle on frames 238..263.
- Added a focused regression for the selected window endpoint at frame 300.

## Selected Case

- Case ID: `death_respawn_level1_case_00200_00300`
- Selector: `death-respawn-level1-200-300`
- Autonomy gate: `patch-allowed`
- Current verify result: `PASS`

## Why This Case

- The case-queue loop selected it in cycle 18, so it remained the only allowed scope.
- Fresh verification showed the remaining drift was the early title-start `Load_InGame` hand-off, not the long death/respawn mode-8 shell.
- The ROM path made the distinction explicit: this lineage does not expose the ordinary title-worker shell, so it needed its own trace model instead of more tweaks to the shared title-card path.

## Strongest Evidence

- `.cache/cases/death-respawn-level1-200-300/summary.md` now reports `passed: true`.
- `cargo run -p kidcli -- case verify --case death-respawn-level1-200-300` now returns `PASS`.
- The oracle semantic bundle showed the exact visible settle to match:
  frame 238 `camera.y_pos=0x00D80000`, `player.y_pos=0x00FF0000`;
  frames 239..249 use the discrete falling/jump-state ramp;
  frame 263 reaches the settled `camera.y_pos=0x00730000`.

## Exact Next Moves

1. Re-run `cargo run -p kidcli -- autonomous-loop --goal case-queue`.
2. Take the next selected case only.
3. Re-verify it before patching, then read the named ASM before any gameplay change.

## Verification To Re-run

- `cargo test -p kid-core death_respawn_level_1_keeps_bootstrap_hidden_on_frame_200 -- --nocapture`
- `cargo test -p kid-core death_respawn_level_1_matches_load_ingame_handoff_through_frame_300 -- --nocapture`
- `cargo run -p kidcli -- case verify --case death-respawn-level1-200-300`

## Dead Ends / Do Not Retry

- Do not keep treating this window as the long death/respawn mode-8 bootstrap; the selected case is the earlier title-start `Load_InGame` hand-off.
- Do not reuse linear interpolation for the visible settle; oracle uses a discrete camera ramp and a short jump-state Y ladder.
- Do not spend another turn on `death_respawn_level1_case_00200_00300` unless verification regresses; the case is now green.
