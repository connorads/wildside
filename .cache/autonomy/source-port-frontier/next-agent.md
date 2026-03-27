---
slug: any-percent-published-tail-clean-through-end
cycle: 76
updated_at: 2026-03-27T09:05:12Z
---

# Source-Port Frontier Handoff

## What Changed

- Took the next live non-tail fidelity item from the published-TAS handoff: backlog `#037` level `0x52`.
- Tightened [`crates/kid-core/src/level.rs`](/home/connor/git/kc/crates/kid-core/src/level.rs) so the spawn-support fallback now allows the stair-step continuation that level `0x52` exposes while still rejecting the older isolated diagonal-support case.
- Added `respawn_settle_level_52_uses_stair_step_support_path` in [`crates/kid-core/src/tests.rs`](/home/connor/git/kc/crates/kid-core/src/tests.rs) and refreshed [`docs/source-port/backlog.md`](/home/connor/git/kc/docs/source-port/backlog.md) plus [`docs/source-port/research-map.md`](/home/connor/git/kc/docs/source-port/research-map.md): `0x52` is no longer part of the live `#037` tail.

## Current Model

- The published any-percent tail is still complete through frame `37759`; no TAS successor frontier reopened.
- `#037` remains open, but only for the shared camera-convergence residue on levels `0x24` / `0x51`.
- Level `0x52` now takes the oracle-backed stair-step support path immediately: `inspect-respawn-settle --level 52 --steps 180` lands on `kid_y=0x014F0000` with `wide_support=true` on step 1, so the old support-path miss is gone.
- Level `0x29` remains locked down by its cached-manifest regression test.

## Strongest Evidence

- [`crates/kid-core/src/tests.rs`](/home/connor/git/kc/crates/kid-core/src/tests.rs)
  - `respawn_settle_level_29_matches_oracle_wall_nudged_spawn_x`
  - `respawn_settle_level_52_uses_stair_step_support_path`
- Local inspection
  - `cargo run -q -p kidcli -- inspect-respawn-settle --level 52 --steps 180`
  - step 1 now reports `wide_support=true` and keeps `kid_y=0x014F0000`
- Tracker updates
  - [`docs/source-port/backlog.md`](/home/connor/git/kc/docs/source-port/backlog.md)
  - [`docs/source-port/research-map.md`](/home/connor/git/kc/docs/source-port/research-map.md)

## Exact Next Moves

1. Re-read the init-settle camera path around [`crates/kid-core/src/movement.rs`](/home/connor/git/kc/crates/kid-core/src/movement.rs) and [`crates/kid-core/src/physics.rs`](/home/connor/git/kc/crates/kid-core/src/physics.rs) for the remaining `#037` levels only.
2. Use `cargo run -q -p kidcli -- inspect-respawn-settle --level 24 --steps 180` and `cargo run -q -p kidcli -- inspect-respawn-settle --level 51 --steps 180` to compare the last few camera-follow steps against the cached tracker values.
3. Fix `0x24` / `0x51` together as one camera-convergence change, not as separate level-specific tweaks.

## Verification To Re-run

- `cargo test -q -p kid-core respawn_settle_level_29_matches_oracle_wall_nudged_spawn_x -- --nocapture`
- `cargo test -q -p kid-core respawn_settle_level_52_uses_stair_step_support_path -- --nocapture`
- `cargo run -q -p kidcli -- inspect-respawn-settle --level 52 --steps 180`
- `cargo run -q -p kidcli -- inspect-respawn-settle --level 24 --steps 180`
- `cargo run -q -p kidcli -- inspect-respawn-settle --level 51 --steps 180`
- `mise run verify-slice-fast`

## Dead Ends / Do Not Retry

- Do not reopen the published any-percent tail; that frontier is still complete through frame `37759`.
- Do not keep treating level `0x29` as an unfixed slope-settle bug unless the new regression test fails.
- Do not reopen the `0x52` support-path hypothesis unless `respawn_settle_level_52_uses_stair_step_support_path` or `inspect-respawn-settle --level 52` regresses.
- Do not trust the generic `sweep` oracle output that currently falls back to the paused boot state for these isolated settle checks; use the focused cached-manifest test plus `inspect-respawn-settle` while narrowing `#037`.
