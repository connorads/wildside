# PRD: Level 1 Full Traversal

## Goal

Turn the completed opening-slice, gameplay-lifecycle, and graphics work into one coherent Level 1 traversal milestone: real terrain on the default native path, flag reachability, and clean next-level transition parity against the oracle.

## Current state

| System | Status |
|--------|--------|
| Opening-slice boot/title/load/oracle checkpoints | Done |
| Pause/resume and death/respawn lifecycle parity | Partial; pause is green again, death still overruns the late-hazard route |
| Asset-backed native Level 1 visuals | Done |
| Flag touch and level advance code path | Done in `kid-core` tests |
| `progress-two-transitions` gameplay scenario | Passing with explicit generated `Level_Layout` loading |
| Default native runtime terrain authority | Incomplete; still falls back to flat floor outside the generated-terrain path |
| Full traversal oracle parity | Partial; progress scenario passes, but dumped terrain is not yet the default path for pause/death scenarios |

## Scope

### 1. Terrain on the default runtime path

- Remove the remaining flat-floor fallback from the default Level 1 runtime path
- Drive collision and grounded state from real Level 1 terrain data rather than test-only grid injection
- Use dumped `Level_Layout` data, not raw `block.bin`, as the terrain authority input
- Keep the terrain logic pure inside `kid-core`

### 2. Traversal and transition parity

- Use the existing gameplay comparison harness to validate start-to-transition traversal
- Make `compare-native-gameplay --scenario progress-two-transitions` pass
- Keep pause/resume green and recover death/respawn parity after the jump-air fix
- Move pause/death comparisons onto the dumped-terrain path once their route parity holds

### 3. Tracked source of truth

- Keep this PRD, the traversal dossier, and the traversal goal JSON as the canonical resume points
- Update top-level docs so they point at traversal completion rather than older frontiers
- Do not depend on untracked scratchpads for essential context

## Execution protocol

- Use TDD by default: write the failing test or failing comparison expectation first
- Land the smallest coherent runtime change, then rerun the narrowest relevant verification
- Commit after each MVP slice: docs/goal alignment, terrain authority, traversal parity
- If compacted, resume from:
  - this PRD
  - `docs/source-port/level-1-full-traversal.md`
  - `docs/source-port/level-1-full-traversal-goal.json`
  - the latest commits
  - `.cache/autonomy/level1-full-traversal/ledger.json`
- Record blockers with the exact failing command and artefact path, then continue with the next unblocked capability

## Acceptance criteria

- The default native Level 1 runtime no longer depends on the flat-floor fallback
- `compare-native-gameplay --scenario progress-two-transitions` passes
- `compare-native-gameplay --scenario pause-resume-level1` stays green, and `death-respawn-level1` returns to green, then both stay green on the real-terrain path
- Docs clearly state that the current frontier is Level 1 full traversal
- The autonomy loop can evaluate `--goal level1-full-traversal` and produce the next unresolved task package

## Verification

Keep green:

```bash
cargo test -p kid-core
cargo test -p kidcli
cargo test -p kid-render
```

Scenario checks:

```bash
cargo run -p kidcli -- dump-level-layout --level 00
cargo run -p kidcli -- compare-native-oracle --scenario boot-title-start-level1
cargo run -p kidcli -- compare-native-oracle --scenario boot-title-start-level1-move-right5s
cargo run -p kidcli -- compare-native-gameplay --scenario pause-resume-level1
cargo run -p kidcli -- compare-native-gameplay --scenario death-respawn-level1
cargo run -p kidcli -- compare-native-gameplay --scenario progress-two-transitions
cargo run -p kidcli -- autonomous-loop --goal level1-full-traversal
```

## Out of scope

- Whole-game level-order completion beyond what `progress-two-transitions` already exercises
- New renderer architecture work
- Broad enemy-family fidelity beyond traversal blockers
- Audio/presentation polish
