---
slug: case-workflow-semantic-extraction-audit-complete
cycle: 80
updated_at: 2026-03-31T00:15:00Z
---

# Source-Port Frontier Handoff

## What Changed

- Ran full `kidcli case derive/branch/verify/reduce` workflow on `death-respawn-level1` scenario
- Confirmed case workflow correctly identifies semantic extraction differences
- Verified `compare-native-gameplay` passes for all gameplay scenarios
- Confirmed all 133 backlog items remain closed

## Current Model

- **Source-port has no open gameplay bugs** - backlog is clean
- **Case workflow semantic failures are expected** - boot-phase extraction differences between native and oracle semantic state
- **Native exposes velocity/facing/on_ground during title screen** - oracle returns `null` for these fields
- **This is not a bug** - it's a representational difference in semantic state extraction during non-gameplay modes

## Strongest Evidence

- `case derive --gameplay-scenario death-respawn-level1 --frame-start 300 --frame-end 500`: creates case bundle
- `case verify`: FAIL at frame 300 (`player.x_velocity: native=0, oracle=none`)
- `compare-native-gameplay --scenario death-respawn-level1`: PASS (only checks authoritative fields)
- `compare-native-gameplay --scenario pause-resume-level1`: PASS
- `compare-native-gameplay --scenario progress-two-transitions`: PASS
- Both native and oracle show `boot_phase: "title_screen"` at frame 300

## Exact Next Moves

1. **Semantic extraction classification** - decide whether to:
   - Accept as known (document in methodology)
   - Update native extraction to not populate velocity/facing/on_ground during title screen
   - Update `case verify` to mask advisory fields during non-gameplay modes

2. **No backlog action needed** - this is not a gameplay bug, backlog remains closed

## Verification To Re-run

```
mise run verify-slice-fast
mise run verify-1p-playable
cargo run -q -p kidcli -- compare-native-gameplay --scenario pause-resume-level1
cargo run -q -p kidcli -- compare-native-gameplay --scenario death-respawn-level1
cargo run -q -p kidcli -- compare-native-gameplay --scenario progress-two-transitions
cargo run -q -p kidcli -- case verify --case .cache/cases/death-respawn-300-500
```

## Dead Ends / Do Not Retry

- Frame-0/200/300 semantic cases will show `player.x_pos/y_velocity: none vs 0` - boot-phase extraction difference, not a bug
- `case verify` uses stricter semantic comparison than `compare-native-gameplay`
- All `case derive` tests from existing scenarios will show boot-phase differences until fresh recordings with mid-gameplay frames are captured