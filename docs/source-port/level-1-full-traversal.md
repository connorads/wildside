# Level 1 Full Traversal Dossier

## Scope

This dossier covers the current post-opening frontier:
- real terrain authority on the default native runtime path
- Level 1 start-to-flag traversal
- flag completion and next-level transition parity
- autonomous continuation without relying on chat history

## Read first

- `tasks/prd-level-1-full-traversal.md`
- `docs/source-port/level-1-dossier.md`
- `docs/source-port/level-1-gameplay-lifecycle.md`
- `docs/source-port/level-1-full-traversal-goal.json`
- `.cache/native-gameplay-oracle-runs/progress_two_transitions/`

## Current truth

- Opening-slice autonomy goal: complete
- Gameplay-lifecycle autonomy goal: complete
- Asset-backed Level 1 visuals: landed
- The default runtime still keeps a flat-floor fallback when no `BlockGrid` is loaded
- `compare-native-gameplay --scenario progress-two-transitions` currently fails:
  - checkpoint: `progress_mid@1999`
  - divergence: game mode
  - native: `0x000C`
  - oracle: `0x0008`

## Immediate blockers

- The default runtime path does not yet source real terrain authority from generated Level 1 data
- Level-complete and next-level transition code exists, but the long traversal oracle still diverges
- Top-level docs were pointing at older slices; this file is now the canonical traversal hand-off

## Useful commands

```bash
cargo test -p kid-core
cargo test -p kidcli
cargo run -p kidcli -- compare-native-gameplay --scenario progress-two-transitions
cargo run -p kidcli -- compare-native-gameplay --scenario death-respawn-level1
cargo run -p kidcli -- autonomous-loop --goal level1-full-traversal
```

## Continuation protocol

1. Read this dossier, the PRD, the goal JSON, and the latest 10 commits.
2. Open `.cache/autonomy/level1-full-traversal/ledger.json` if it exists.
3. Pick the first non-complete capability from the goal contract.
4. Write or tighten the failing test first.
5. Make the smallest runtime change that moves that capability.
6. Rerun only the narrow verification set for that capability, then the full relevant package tests.
7. Update this dossier with:
   - what changed
   - what still fails
   - the exact next command to run

## Next expected implementation order

1. Remove the flat-floor fallback from the default runtime path.
2. Re-run `progress-two-transitions` and inspect the first remaining divergence.
3. Tighten traversal/transition state until the gameplay comparison passes.
4. Leave the next frontier explicit in tracked docs.
