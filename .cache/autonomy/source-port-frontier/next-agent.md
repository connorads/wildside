---
slug: native-play-case-workflow-investigation
cycle: 78
updated_at: 2026-03-30T23:20:00Z
---

# Source-Port Frontier Handoff

## What Changed

- Investigated native-play recording `level1-tree-break` - it's stale (March 19) and drifted from current codebase
- Tested `kidcli case derive/branch/verify/reduce` workflow end-to-end with `death-respawn-level1` and `pause-resume-level1` scenarios
- Confirmed `compare-native-gameplay --scenario death-respawn-level1` and `compare-native-gameplay --scenario pause-resume-level1` both pass
- Confirmed `mise run verify-slice-fast` passes (27 + 33 tests = 60 total)

## Current Model

- No active gameplay frontier. The TAS frontier `any-percent-published` is complete through frame 37759
- All backlog items (#001-#133) are closed ([x] or [x] FIXED or [-] won't fix)
- The case workflow correctly identifies semantic state differences:
  - Frame 700 `game_mode` divergence (native=12 InGame, oracle=8 LevelTransition) is expected per #123 death/respawn bootstrap timing
  - Oracle correctly stays in hidden bootstrap (0x08) while native was exposing gameplay (0x0C) earlier - this is documented in the backlog

## Strongest Evidence

- `mise run verify-slice-fast`: PASS (60 tests)
- `cargo run -q -p kidcli -- compare-native-gameplay --scenario death-respawn-level1`: PASS
- `cargo run -q -p kidcli -- compare-native-gameplay --scenario pause-resume-level1`: PASS
- Case workflow demonstrates:
  - `kidcli case derive --gameplay-scenario death-respawn-level1 --frame-start 700 --frame-end 800`: creates semantic case bundle
  - `kidcli case branch --case <path>`: generates continuation variants
  - `kidcli case verify --case <path>`: identifies semantic failures
  - Frame-700 game_mode divergence is known/expected behavior

## Exact Next Moves

1. The source-port appears complete for current backlog. Any new work should:
   - Come from fresh native-play recordings capturing real gameplay bugs
   - Use `kidcli play-native --record <slug>` to capture new issues
   - Use `kidcli case derive/branch/verify/reduce` workflow to isolate minimal semantic cases
2. If continuing source-port work, check for:
   - New regression in `mise run verify-slice-fast`
   - New divergence in `cargo run -p kidcli -- verify-tas --scenario tests/data/tas/any-percent-published/scenario.json --frame-start 23250 --bisect`
   - Fresh `play-native --record` sessions with actual gameplay bugs

## Verification To Re-run

- `mise run verify-slice-fast`
- `cargo run -q -p kidcli -- compare-native-gameplay --scenario death-respawn-level1`
- `cargo run -q -p kidcli -- compare-native-gameplay --scenario pause-resume-level1`
- `mise run verify-1p-playable`

## Dead Ends / Do Not Retry

- Recording `level1-tree-break` is stale (March 19) - do not use for case reduction without re-recording
- Frame-0 semantic case shows null vs 0 player position - this is boot-phase extraction difference, not a gameplay bug
- Frame-700 game_mode divergence on death-respawn-level1 is expected per backlog #123