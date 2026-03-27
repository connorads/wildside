---
slug: any-percent-published-tail-clean-through-end
cycle: 76
updated_at: 2026-03-27T08:54:44Z
---

# Source-Port Frontier Handoff

## What Changed

- Took the next live non-tail fidelity item from the published-TAS handoff: backlog `#037`.
- Added `respawn_settle_level_29_matches_oracle_wall_nudged_spawn_x` in [`crates/kid-core/src/tests.rs`](/home/connor/git/kc/crates/kid-core/src/tests.rs) so the cached manifest now locks level `0x29` to the oracle-backed settle result `kid_x=$02A50000`, `camera_x=$02090000`.
- Narrowed the tracker text in [`docs/source-port/backlog.md`](/home/connor/git/kc/docs/source-port/backlog.md) and [`docs/source-port/research-map.md`](/home/connor/git/kc/docs/source-port/research-map.md): level `0x29` is no longer part of the live `#037` tail.

## Current Model

- The published any-percent tail is still complete through frame `37759`; no TAS successor frontier reopened.
- `#037` remains open, but only for the real residual settle debt:
  - level `0x52` support-path mismatch
  - levels `0x24` / `0x51` `camera_y` off by 1px
- Level `0x29` was stale tracker debt. The current code path already reaches the oracle-backed wall-nudged spawn X; the missing piece was a regression test so future settle refactors do not reopen it.

## Strongest Evidence

- [`crates/kid-core/src/tests.rs`](/home/connor/git/kc/crates/kid-core/src/tests.rs)
  - `respawn_settle_level_29_matches_oracle_wall_nudged_spawn_x`
- Local inspection
  - `cargo run -q -p kidcli -- inspect-respawn-settle --level 29 --steps 80`
  - final state settles at `kid_x=0x02A50000`, `camera_x=0x02090000`
- Tracker updates
  - [`docs/source-port/backlog.md`](/home/connor/git/kc/docs/source-port/backlog.md)
  - [`docs/source-port/research-map.md`](/home/connor/git/kc/docs/source-port/research-map.md)

## Exact Next Moves

1. Re-read the init-settle path around [`crates/kid-core/src/level.rs`](/home/connor/git/kc/crates/kid-core/src/level.rs) and the floor/camera code in [`crates/kid-core/src/movement.rs`](/home/connor/git/kc/crates/kid-core/src/movement.rs) for the remaining `#037` levels only.
2. Use `cargo run -q -p kidcli -- inspect-respawn-settle --level 52 --steps 180` and `cargo run -q -p kidcli -- inspect-spawn-probe --level 52` to isolate why the current support guard still misses the shallower oracle-backed landing path.
3. After `0x52`, tackle the shared `0x24` / `0x51` 1px `camera_y` residue as a single camera-convergence fix, not two level-specific tweaks.

## Verification To Re-run

- `cargo test -q -p kid-core respawn_settle_level_29_matches_oracle_wall_nudged_spawn_x -- --nocapture`
- `cargo run -q -p kidcli -- inspect-respawn-settle --level 29 --steps 80`
- `cargo run -q -p kidcli -- inspect-respawn-settle --level 52 --steps 180`
- `cargo run -q -p kidcli -- inspect-spawn-probe --level 52`
- `mise run verify-slice-fast`

## Dead Ends / Do Not Retry

- Do not reopen the published any-percent tail; that frontier is still complete through frame `37759`.
- Do not keep treating level `0x29` as an unfixed slope-settle bug unless the new regression test fails.
- Do not trust the generic `sweep` oracle output that currently falls back to the paused boot state for these isolated settle checks; use the focused cached-manifest test plus `inspect-respawn-settle` while narrowing `#037`.
