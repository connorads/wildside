---
slug: any-percent-published-tail-clean-through-end
cycle: 76
updated_at: 2026-03-27T10:15:00Z
---

# Source-Port Frontier Handoff

## What Changed

- Finished the remaining non-tail fidelity follow-on from the published-TAS handoff: backlog `#037` on levels `0x24` / `0x51`.
- Updated [`crates/kid-core/src/level.rs`](/home/connor/git/kc/crates/kid-core/src/level.rs) so `level_respawn_slice_with_floor_snap()` applies the final hidden `sub_71E4`-backed camera carry when the settle stalls at the exact `target-camera == 2px` residue.
- Added `respawn_settle_level_24_matches_oracle_camera_convergence` and `respawn_settle_level_51_matches_oracle_camera_convergence` in [`crates/kid-core/src/tests.rs`](/home/connor/git/kc/crates/kid-core/src/tests.rs), and refreshed [`docs/source-port/backlog.md`](/home/connor/git/kc/docs/source-port/backlog.md), [`docs/source-port/research-map.md`](/home/connor/git/kc/docs/source-port/research-map.md), and [`docs/source-port/tas-roadmap.md`](/home/connor/git/kc/docs/source-port/tas-roadmap.md): `#037` is now closed.

## Current Model

- The published any-percent tail is still complete through frame `37759`; no TAS successor frontier reopened.
- `#037` is now done: levels `0x24` and `0x51` finish on the oracle-backed `camera_y=$036E0000` / `$028E0000`, while levels `0x29` and `0x52` stay locked down by their focused regressions.
- The next incomplete TAS work is the live `kc_any_percent_2024` pause-snapshot frontier on frame `3414`.

## Strongest Evidence

- [`crates/kid-core/src/tests.rs`](/home/connor/git/kc/crates/kid-core/src/tests.rs)
  - `respawn_settle_level_29_matches_oracle_wall_nudged_spawn_x`
  - `respawn_settle_level_52_uses_stair_step_support_path`
  - `respawn_settle_level_24_matches_oracle_camera_convergence`
  - `respawn_settle_level_51_matches_oracle_camera_convergence`
- Local inspection
  - `cargo run -q -p kidcli -- inspect-respawn-settle --level 24 --steps 180`
  - `cargo run -q -p kidcli -- inspect-respawn-settle --level 51 --steps 180`
  - the final traced settle frame now lands at `camera_y=$036E0000` / `$028E0000`
- Tracker updates
  - [`docs/source-port/backlog.md`](/home/connor/git/kc/docs/source-port/backlog.md)
  - [`docs/source-port/research-map.md`](/home/connor/git/kc/docs/source-port/research-map.md)
  - [`docs/source-port/tas-roadmap.md`](/home/connor/git/kc/docs/source-port/tas-roadmap.md)

## Exact Next Moves

1. Re-read the frame-3414 pause path in `.cache/upstream/kid-chameleon-disasm/kid.asm` before changing native code; stay on the pause snapshot / `Game_Paused` hand-off rather than reopening published-tail or respawn-settle work.
2. Run `cargo run -q -p kidcli -- tas-diff --scenario .cache/tas-imports/kc_any_percent_2024/scenario.json --frame-start 3412 --frame-end 3416 --include-advisory --show-object-roster-on-match` and `cargo run -q -p kidcli -- verify-tas --scenario .cache/tas-imports/kc_any_percent_2024/scenario.json --frame-start 3400 --frame-end 3500 --bisect` to restate the live pause-snapshot miss.
3. Fix the `kc_any_percent_2024` frame-3414 pause snapshot as one engine-level change, then lock it down with a focused regression test and `mise run verify-slice-fast`.

## Verification To Re-run

- `cargo test -q -p kid-core respawn_settle_level_24_matches_oracle_camera_convergence -- --nocapture`
- `cargo test -q -p kid-core respawn_settle_level_51_matches_oracle_camera_convergence -- --nocapture`
- `cargo test -q -p kid-core respawn_settle_level_29_matches_oracle_wall_nudged_spawn_x -- --nocapture`
- `cargo test -q -p kid-core respawn_settle_level_52_uses_stair_step_support_path -- --nocapture`
- `cargo run -q -p kidcli -- inspect-respawn-settle --level 24 --steps 180`
- `cargo run -q -p kidcli -- inspect-respawn-settle --level 51 --steps 180`
- `mise run verify-slice-fast`

## Dead Ends / Do Not Retry

- Do not reopen the published any-percent tail; that frontier is still complete through frame `37759`.
- Do not reopen `#037` unless one of the four focused respawn-settle regressions fails.
- Do not treat the `inspect-respawn-settle` mid-loop `camera_y=$036D/$028D` frames as the returned settle result; the final hidden carry is applied after the loop and is now reflected in the last traced frame only.
- Do not trust the generic `sweep` oracle output that currently falls back to the paused boot state for isolated settle checks; use the focused regressions plus `inspect-respawn-settle`.
