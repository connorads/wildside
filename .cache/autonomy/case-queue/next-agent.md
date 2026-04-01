---
slug: death_respawn_level1_case_00400_00500
cycle: 20
updated_at: 2026-04-01T00:04:02Z
---

## What Changed

- Ran `cargo run -p kidcli -- autonomous-loop --goal case-queue`; cycle 20 selected `death_respawn_level1_case_00400_00500`.
- Re-ran `cargo run -p kidcli -- case verify --case death-respawn-level1-400-500` first; the stored case failed at frame 472 on `player.hitpoints`.
- Read the ROM damage path in `.cache/upstream/kid-chameleon-disasm/kid.asm` (`Character_CheckCollision` / `Check_for_recent_damge`) before any gameplay decision; ASM showed no ambiguity in the damage latch itself.
- Traced the failure to a stale case/scenario bundle, not a gameplay-code bug: `kidcli`'s `DeathRespawnLevel1` scenario was missing the frame-462 `Start` press used by the repo's death/respawn regression script.
- Added that frame-462 `Start` press to `crates/kidcli/src/scenarios.rs`, added a regression test, and regenerated the selected case bundle in place.

## Selected Case

- Case ID: `death_respawn_level1_case_00400_00500`
- Selector: `death-respawn-level1-400-500`
- Autonomy gate: `patch-allowed`
- Current verify result: `PASS`

## Why This Case

- The case-queue loop selected it in cycle 20, so it was the only allowed scope.
- Fresh verification showed the first failure lived inside the setup window for this exact case, so the session stayed on that one bundle.
- The strongest local contradiction was that the repo backlog said this smoke path should miss later, while the stored case still failed early at frame 472.

## Strongest Evidence

- `cargo run -p kidcli -- compare-native-gameplay-window --scenario death-respawn-level1 --frame-start 460 --frame-end 472` now passes.
- `cargo run -p kidcli -- case verify --case death-respawn-level1-400-500` now returns `case death_respawn_level1_case_00400_00500: PASS`.
- `.cache/cases/death-respawn-level1-400-500/scenario.json` now contains `script[462] = 128`, matching the in-repo death/respawn regression script.
- `.cache/cases/death-respawn-level1-400-500/summary.md` now reports `passed: true`.

## Exact Next Moves

1. Re-run `cargo run -p kidcli -- autonomous-loop --goal case-queue`.
2. Take the next selected case only.
3. Re-verify it before patching, and if gameplay changes are needed, read the relevant ASM first.

## Verification To Re-run

- `cargo test -p kidcli 'scenarios::tests::death_respawn_level_1_script_reuses_walk_only_opening_baseline' -- --exact`
- `cargo run -p kidcli -- compare-native-gameplay-window --scenario death-respawn-level1 --frame-start 460 --frame-end 472`
- `cargo run -p kidcli -- case verify --case death-respawn-level1-400-500`
- `cargo run -p kidcli -- case summarise --case death-respawn-level1-400-500`

## Dead Ends / Do Not Retry

- Do not keep debugging `player.hitpoints` at frame 472 as a gameplay/ASM bug for this case; the failing bundle was stale.
- Do not trust the cached `.cache/cases/death-respawn-level1-400-500/scenario.json` if it still shows `script[462] = 0`; regenerate the case bundle from `--gameplay-scenario death-respawn-level1`.
- Do not widen into the later death/respawn camera hand-off work from this green case; that belongs to a later case/frontier.
