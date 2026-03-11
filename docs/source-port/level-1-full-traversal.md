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
- Level 1 opening damage now lands on the oracle frame under the run-right gameplay script, and Mini Hopping Skulls now wait longer on the floor and retarget hops toward the kid before launch
- `compare-native-gameplay --scenario pause-resume-level1`: passing on dumped terrain
- `compare-native-gameplay --scenario death-respawn-level1`: passing again after locking the death camera, snapping the synthetic late hazard onto the oracle camera checkpoint, and extending the respawn settle slightly
- Dumped-terrain wall probing no longer treats the landing floor cell as a side wall; the lower-body wall probe now stays above the feet and has a regression test in `kid-core`
- Gameplay movement now latches held left/right/run from the previous frame, removing the frame-720 eagerness against the oracle while keeping jump-edge handling on the current frame
- Diamonds no longer enter `process_stomps`; falling through a Diamond now collects it without applying stomp bounce, and the stomp regressions now run against the static skull fixture instead of the old Diamond placeholder
- Transition parity currently uses an explicit generated `Level_Layout` mode for `pause-resume-level1` and `progress-two-transitions`
- The default runtime still keeps the flat-floor fallback outside those explicit generated-terrain paths

## Immediate blockers

- Real terrain is not yet the default runtime path for all gameplay scenarios
- Generated terrain currently depends on a dumped Level 0 `Level_Layout` cache under `.cache/generated/<rom-sha1>/level-layout/00.bin`
- The remaining blocker is `death-respawn-level1` on dumped terrain; pause and progress now hold there, but death still trails the oracle route before `pre_death@1279`
- After the opening-hit, skull-timing, landing-wall, held-input latch, and diamond-stomp fixes, dumped-terrain death still diverges in kid Y at `pre_death@1279` (`native 0x008F0000`, `oracle 0x00F88900`)
- The false floor-as-wall clamp at the first landing is fixed, but the dumped-terrain route is still ahead of the oracle before the death window and now clips the underside of the overhang around frames 888-904
- The surviving route drift now starts as a smaller kid-X mismatch at frame `743` and is still a jump-arc/high-route problem: startup eagerness and diamond bounce are fixed, but dumped terrain still reaches the overhang late, clips the underside around frames 888-904, and misses the late high-route window before `pre_death@1279`
- Top-level docs were pointing at older slices; this file is now the canonical traversal hand-off

## Useful commands

```bash
mise run verify-slice-fast
cargo test -p kid-core
cargo test -p kidcli
cargo run -p kidcli -- dump-level-layout --level 00
cargo run -p kidcli -- compare-native-gameplay --scenario progress-two-transitions
cargo run -p kidcli -- compare-native-gameplay --scenario pause-resume-level1
cargo run -p kidcli -- compare-native-gameplay --scenario death-respawn-level1
cargo run -p kidcli -- diagnose-native-gameplay-drift --scenario pause-resume-level1 --from-frame 719
cargo run -p kidcli -- autonomous-loop --goal level1-full-traversal
KID_NATIVE_GENERATED_LEVEL_LAYOUT=1 cargo run -p kidcli -- compare-native-gameplay --scenario pause-resume-level1
KID_NATIVE_GENERATED_LEVEL_LAYOUT=1 cargo run -p kidcli -- compare-native-gameplay --scenario death-respawn-level1
KID_NATIVE_GENERATED_LEVEL_LAYOUT=1 cargo run -p kidcli -- probe-native-gameplay --scenario death-respawn-level1
```

## Continuation protocol

1. Read this dossier, the PRD, the goal JSON, and the latest 10 commits.
2. Open `.cache/autonomy/level1-full-traversal/ledger.json` if it exists.
3. Pick the first non-complete capability from the goal contract.
4. Write or tighten the failing test first.
5. Make the smallest runtime change that moves that capability.
6. Rerun `mise run verify-slice-fast`, then the full relevant package tests and compares for that capability.
7. Update this dossier with:
   - what changed
   - what still fails
   - the exact next command to run

Fast gate policy:

- `mise run verify-slice-fast` is the default pre-commit gate for gameplay/runtime work.
- `mise run verify-slice-full` is the slice acceptance gate before calling a traversal/runtime step green.
- If `pause-resume-level1` is red, do not lock in generated-terrain death work from the same timing family.

## Next expected implementation order

1. Keep `death-respawn-level1` on the stable default path, and close the dumped-terrain late jump/late death divergence before moving it over by default.
2. Remove the flat-floor fallback from the default runtime path only after the dumped-terrain path keeps all gameplay comparisons green.
3. Keep `progress-two-transitions` green while moving death onto dumped terrain.
4. Leave the next frontier explicit in tracked docs after each MVP slice.
