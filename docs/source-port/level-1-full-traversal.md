# Level 1 Full Traversal Dossier (ARCHIVED)

> Traversal parity achieved. All traversal scenarios pass oracle comparison. See `research-map.md` for remaining work.

## Scope

This dossier covered the post-opening frontier:

- real terrain authority on the default native runtime path
- Level 1 start-to-flag traversal
- flag completion and next-level transition parity
- autonomous continuation without relying on chat history

## Read first

- `docs/source-port/level-1-dossier.md`
- `docs/source-port/level-1-gameplay-lifecycle.md`
- `docs/source-port/level-1-full-traversal-goal.json`
- `.cache/native-gameplay-oracle-runs/progress_two_transitions/`

## Current truth

- Opening-slice autonomy goal: complete
- Gameplay-lifecycle autonomy goal: complete
- Asset-backed Level 1 visuals: landed
- `play-native` is the primary Level 1 path. It renders background, decoded foreground terrain, live block overrides, sprites, and HUD directly from the generated bundle.
- `verify-native-bootstrap`: passing
- `verify-native-gameplay --scenario pause-resume-level1`: passing
- `verify-native-gameplay --scenario death-respawn-level1`: passing
- `compare-native-gameplay --scenario pause-resume-level1`: passing on the default runtime path
- `compare-native-gameplay --scenario death-respawn-level1`: passing on the default runtime path
- `compare-native-gameplay --scenario progress-two-transitions`: passing on the default runtime path
- Jump take-off no longer re-grounds on the first held-button frame, solid terrain grounds one pixel above the cell top, and player jump-air control uses the disassembly-backed per-helmet tables.
- Level 1 opening damage lands on the oracle frame under the run-right gameplay script, and Mini Hopping Skulls wait longer on the floor and retarget hops toward the kid before launch.
- The long `progress-two-transitions` gameplay script also drops the first Level 1 HP on the oracle frame under walk-only input (`frame 749`), so the opening Dragon hit no longer drifts between checkpoints.
- The default `NativeRuntime::new` path reloads generated `Level_Layout` terrain on construction, so real terrain is the default runtime authority for Level 1.
- Dumped-terrain wall probing no longer treats the landing floor cell as a side wall. The lower-body wall probe stays above the feet and has a regression test in `wildside-core`.
- Gameplay movement latches held left/right/run from the previous frame, removing the frame-720 eagerness against the oracle while keeping jump-edge handling on the current frame.
- Diamonds no longer enter `process_stomps`. Falling through a Diamond collects it without applying stomp bounce, and the stomp regressions run against the static skull fixture instead of the old Diamond placeholder.
- Level-skip choreography matches the oracle through both `progress-two-transitions` skips: the first hold sequence freezes/hides/preloads on the native frames, the second skip resumes through the oracle-timed Level 1 hold/blank/entry window, and the Level 2 transition uses its own traced animation arc.
- The traversal goal contract reported `status: complete` (evaluator since retired with the orchestrator, 2026-06-10).

## Closure state

- Generated terrain depends on a dumped Level 0 `Level_Layout` cache under `.cache/generated/<rom-sha1>/level-layout/00.bin`.
- Traversal closure has no open runtime blocker in-repo: default terrain authority is real, the three gameplay compares are green, and the autonomy contract closes.
- Oracle compares remain useful as optional differential checks, not the primary sign-off path for whether Level 1 is playable.
- `diagnose-native-gameplay-drift --scenario progress-two-transitions --from-frame 719` reports no live drift.
- The next explicit frontier should return to post-traversal Level 1 fidelity work rather than more traversal plumbing.

## Useful commands

```bash
cargo run --release -p wildside-cli -- play-native
cargo run -p wildside-cli -- verify-native-bootstrap
cargo run -p wildside-cli -- verify-native-gameplay --scenario pause-resume-level1
cargo run -p wildside-cli -- verify-native-gameplay --scenario death-respawn-level1
mise run verify-slice-fast
cargo test -p wildside-core
cargo test -p wildside-cli
# optional differential checks
cargo run -p wildside-cli -- compare-native-gameplay --scenario progress-two-transitions
cargo run -p wildside-cli -- compare-native-gameplay --scenario pause-resume-level1
cargo run -p wildside-cli -- compare-native-gameplay --scenario death-respawn-level1
```

## Continuation protocol

1. Read this dossier, the PRD, the goal JSON, and the latest 10 commits.
2. Open `.cache/autonomy/level1-full-traversal/ledger.json` if it exists.
3. Pick the first non-complete capability from the goal contract.
4. Write or tighten the failing test first.
5. Make the smallest runtime change that moves that capability.
6. Rerun `mise run verify-slice-fast`, then the full relevant package tests and compares for that capability.
7. Update this dossier with what changed, what still fails, and the exact next command to run.

Fast gate policy:

- `mise run verify-slice-fast` is the pre-commit gate for gameplay/runtime work.
- For the oracle differential on these scenarios, run `wildside compare-native-gameplay --scenario <name>` directly (pause-resume-level1, progress-two-transitions, death-respawn-level1) before calling a traversal/runtime step green.
- If any traversal compare reopens, use the gameplay compare artefacts first before changing the default runtime path again.

## Next expected implementation order

1. Treat this traversal dossier as closed unless one of the default-path gameplay compares reopens.
2. Move the tracked frontier back to post-traversal Level 1 fidelity work, starting with the enemy-family gaps still visible in Blue Lake Woods 1.
3. Keep the default generated-terrain runtime path and the traversal autonomy goal green while deeper fidelity work lands.
