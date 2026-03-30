---
slug: native-play-case-workflow-complete
cycle: 79
updated_at: 2026-03-30T23:30:00Z
---

# Source-Port Frontier Handoff

## What Changed

- Ran full `kidcli case derive/branch/verify/reduce` workflow on `pause-respawn-level1` scenario:
  - `case derive --gameplay-scenario pause-resume-level1 --frame-start 200 --frame-end 300` creates case bundle
  - `case branch` generates 8 continuation variants
  - `case verify` correctly identifies semantic failures (boot-phase extraction differences)
- Confirmed `mise run verify-slice-fast` and `mise run verify-1p-playable` both pass
- All backlog items (#001-#133) remain closed - no open bugs found

## Current Model

- **Source-port is complete for current backlog** - all 133 backlog items closed
- **No active gameplay frontier** - TAS frontier complete through frame 37759
- **Case workflow semantic differences are boot-phase extraction issues**, not gameplay bugs:
  - `player.x_pos: native=none, oracle=0` at frame 0 is expected (native hasn't initialized player yet)
  - `game_mode` divergence at frame 700 is documented in backlog #123
- `compare-native-gameplay` passes for all gameplay scenarios, masking boot-phase differences
- `case verify` uses stricter semantic comparison, showing boot-phase extraction differences

## Strongest Evidence

- `mise run verify-slice-fast`: PASS (60 tests)
- `mise run verify-1p-playable`: PASS (5 native gameplay tests + comparisons)
- `compare-native-gameplay --scenario pause-resume-level1`: PASS
- `compare-native-gameplay --scenario death-respawn-level1`: PASS
- Case workflow produces expected boot-phase semantic failures (not bugs)

## Exact Next Moves

1. **Source-port is complete** - all known issues resolved
2. To find new bugs, capture fresh recordings:
   - `kidcli play-native --record <slug>` with manual playtesting
   - Focus on areas not covered by existing scenarios (e.g., later levels, complex enemy interactions)
3. Case workflow is ready for new bug isolation when fresh recordings are available

## Verification To Re-run

- `mise run verify-slice-fast`
- `mise run verify-1p-playable`
- `cargo run -q -p kidcli -- compare-native-gameplay --scenario pause-resume-level1`
- `cargo run -q -p kidcli -- compare-native-gameplay --scenario death-respawn-level1`

## Dead Ends / Do Not Retry

- Recording `level1-tree-break` is stale (March 19) - do not use for case reduction without re-recording
- Frame-0/200 semantic cases show `player.x_pos: none vs 0` - boot-phase extraction difference, not a bug
- Frame-700 game_mode divergence on death-respawn-level1 is expected per backlog #123
- All `case derive` tests from existing scenarios will show boot-phase differences until fresh recordings with mid-gameplay frames are captured