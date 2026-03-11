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
- `compare-native-gameplay --scenario progress-two-transitions`: passing
- Jump take-off no longer re-grounds on the first held-button frame, solid terrain grounds one pixel above the cell top, and player jump-air control now uses the disassembly-backed per-helmet tables
- `compare-native-gameplay --scenario pause-resume-level1`: passing again
- `compare-native-gameplay --scenario death-respawn-level1`: currently diverging at `pre_death@1279` in camera X; native hits `hp=0` too far to the right on the late-hazard path
- Transition parity currently uses an explicit generated `Level_Layout` mode for `progress-two-transitions`
- The default runtime still keeps the flat-floor fallback outside that explicit generated-terrain path

## Immediate blockers

- Real terrain is not yet the default runtime path for all gameplay scenarios
- Generated terrain currently depends on a dumped Level 0 `Level_Layout` cache under `.cache/generated/<rom-sha1>/level-layout/00.bin`
- The remaining blocker is the long death route: native overruns the late-hazard path before `pre_death@1279`
- Top-level docs were pointing at older slices; this file is now the canonical traversal hand-off

## Useful commands

```bash
cargo test -p kid-core
cargo test -p kidcli
cargo run -p kidcli -- dump-level-layout --level 00
cargo run -p kidcli -- compare-native-gameplay --scenario progress-two-transitions
cargo run -p kidcli -- compare-native-gameplay --scenario pause-resume-level1
cargo run -p kidcli -- compare-native-gameplay --scenario death-respawn-level1
cargo run -p kidcli -- autonomous-loop --goal level1-full-traversal
KID_NATIVE_GENERATED_LEVEL_LAYOUT=1 cargo run -p kidcli -- compare-native-gameplay --scenario pause-resume-level1
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

1. Close the remaining `death-respawn-level1` late-hazard overrun on the long route.
2. Re-run `death-respawn-level1` on dumped terrain and fix the first traversal divergence that remains there.
3. Remove the flat-floor fallback from the default runtime path only after the dumped-terrain path keeps all gameplay comparisons green.
4. Leave the next frontier explicit in tracked docs.
