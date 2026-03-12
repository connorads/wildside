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
- `play-native` is the primary Level 1 path; it now renders background, decoded foreground terrain, live block overrides, sprites, and HUD directly from the generated bundle
- `verify-native-bootstrap`: passing
- `verify-native-gameplay --scenario pause-resume-level1`: passing
- `verify-native-gameplay --scenario death-respawn-level1`: passing
- `compare-native-gameplay --scenario pause-resume-level1`: passing on the default runtime path
- `compare-native-gameplay --scenario death-respawn-level1`: passing on the default runtime path
- `compare-native-gameplay --scenario progress-two-transitions`: passing on the default runtime path
- Jump take-off no longer re-grounds on the first held-button frame, solid terrain grounds one pixel above the cell top, and player jump-air control now uses the disassembly-backed per-helmet tables
- Level 1 opening damage now lands on the oracle frame under the run-right gameplay script, and Mini Hopping Skulls now wait longer on the floor and retarget hops toward the kid before launch
- The default `NativeRuntime::new` path now reloads generated `Level_Layout` terrain on construction, so real terrain is the default runtime authority for Level 1
- Dumped-terrain wall probing no longer treats the landing floor cell as a side wall; the lower-body wall probe now stays above the feet and has a regression test in `kid-core`
- Gameplay movement now latches held left/right/run from the previous frame, removing the frame-720 eagerness against the oracle while keeping jump-edge handling on the current frame
- Diamonds no longer enter `process_stomps`; falling through a Diamond now collects it without applying stomp bounce, and the stomp regressions now run against the static skull fixture instead of the old Diamond placeholder
- `cargo run -p kidcli -- autonomous-loop --goal level1-full-traversal` now reports `status: complete`

## Closure state

- Generated terrain currently depends on a dumped Level 0 `Level_Layout` cache under `.cache/generated/<rom-sha1>/level-layout/00.bin`
- Traversal closure no longer has an open runtime blocker in-repo: default terrain authority is real, the three gameplay compares are green, and the autonomy contract closes
- Oracle compares remain useful as optional differential checks, not the primary sign-off path for whether Level 1 is playable
- The next explicit frontier should move back to post-traversal Level 1 fidelity work rather than more traversal plumbing

## Useful commands

```bash
cargo run --release -p kidcli -- play-native
cargo run -p kidcli -- verify-native-bootstrap
cargo run -p kidcli -- verify-native-gameplay --scenario pause-resume-level1
cargo run -p kidcli -- verify-native-gameplay --scenario death-respawn-level1
mise run verify-slice-fast
cargo test -p kid-core
cargo test -p kidcli
# optional differential checks
cargo run -p kidcli -- compare-native-gameplay --scenario progress-two-transitions
cargo run -p kidcli -- compare-native-gameplay --scenario pause-resume-level1
cargo run -p kidcli -- compare-native-gameplay --scenario death-respawn-level1
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
- If any traversal compare reopens, use the gameplay compare artefacts first before changing the default runtime path again.

## Next expected implementation order

1. Treat this traversal dossier as closed unless one of the default-path gameplay compares reopens.
2. Move the tracked frontier back to post-traversal Level 1 fidelity work, starting with the enemy-family gaps still visible in Blue Lake Woods 1.
3. Keep the default generated-terrain runtime path and the traversal autonomy goal green while deeper fidelity work lands.
