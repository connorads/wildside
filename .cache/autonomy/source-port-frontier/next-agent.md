---
slug: any-percent-published-frame-8056-demo-1-climb-drift-after-timeout-fix
cycle: 34
updated_at: 2026-03-23T03:18:38Z
---

# Source-Port Frontier Handoff

## What Changed

- Re-ran the demo timeout/title-return ROM path in `kid.asm` and checked oracle RAM directly instead of trusting the previous frame-8004 theory.
- Proved with `oracle-probe` that demo 1 stays in `Game_Mode=$0010` / `Current_LevelID=$0015` through absolute frame `8068` and only flips to mode 0 on frame `8069`.
- Moved `DEMO_1_TIMEOUT_COUNTER_OFFSET_FRAMES` from `3` to `68` so native matches that oracle-backed timeout boundary.
- Added `published_tas_frame_8004_keeps_demo_1_in_live_climb_gameplay` and `published_tas_demo_1_times_out_on_frame_8069`.
- Re-ran the focused demo regressions, the 7968/7969 and `iron_knight_` suites, `verify-tas --frame-start 8002 --frame-end 8010 --checkpoint-interval 1`, `verify-tas --frame-start 6975 --frame-end 9530 --bisect`, `tas-diff --frame-start 8054 --frame-end 8061 --include-physics-trace --include-advisory`, and `mise run verify-slice-fast`.

## Current Model

- The old "native still times out one frame early at 8004" model was wrong. Demo 1 timeout timing is now aligned through the visible handoff: native stays in live gameplay through frame `8068` and exposes the mode-0/title return on frame `8069`, matching oracle.
- The active frontier has moved earlier in the same demo-1 climb sequence: first CRC divergence is now frame `8056`, not the timeout handoff.
- At frame `8056`, oracle Kid has already started drifting left/up (`kid_x_pos=0x03a7d500`, `kid_y_pos=0x0186d680`, non-zero horizontal velocity by frame `8057`), while native stays pinned at `x=0x03a80000` with the old straight-up climb motion.
- Treat this as Iron Knight climb/flip/resume gameplay drift, not timeout/title-return logic.
- Demo 2 still needs its own frozen timeout snapshot rule. Do not merge demo 1 and demo 2 back into one shared timeout rule.
- The frame-7969 Iron Knight climb-resume fix is still holding. Do not reopen it without new oracle evidence.

## Strongest Evidence

- `cargo test -p kid-core published_tas_frame_8004_keeps_demo_1_in_live_climb_gameplay -- --nocapture`
- `cargo test -p kid-core published_tas_demo_1_times_out_on_frame_8069 -- --nocapture`
- `cargo test -p kid-core published_tas_demo_2_freezes_the_timeout_snapshot_on_frame_4966 -- --nocapture`
- `cargo test -p kid-core 7968 -- --nocapture`
- `cargo test -p kid-core 7969 -- --nocapture`
- `cargo test -p kid-core iron_knight_ -- --nocapture`
- `cargo run -p kidcli -- oracle-probe --scenario .cache/tas-imports/any-percent-published/scenario.json --frame-start 7998 --frame-end 8200 --watches 0xFFFFFBCA:u16,0xFFFFFC44:u16`
  - Tail rows show `8068,0x10,0x15` then `8069,0x0,0x0`.
- `cargo run -p kidcli -- verify-tas --scenario .cache/tas-imports/any-percent-published/scenario.json --frame-start 8002 --frame-end 8010 --checkpoint-interval 1`
  - `CRC: all 8 frames match in 8002..8010`.
- `cargo run -p kidcli -- verify-tas --scenario .cache/tas-imports/any-percent-published/scenario.json --frame-start 6975 --frame-end 9530 --bisect`
  - `RESULT: first CRC divergence at frame 8056`.
- `cargo run -p kidcli -- tas-diff --scenario .cache/tas-imports/any-percent-published/scenario.json --frame-start 8054 --frame-end 8061 --include-physics-trace --include-advisory`
  - Frames `8054..8055` are advisory-only.
  - Frame `8056` is the first authoritative miss: native `kid_y_pos=0x01855680`; oracle `kid_y_pos=0x0186d680`.
  - By frame `8057`, oracle also diverges on `kid_x_pos` and `camera_y_pos`, showing the real blocker is climb movement, not timeout exit.
- `mise run verify-slice-fast`

## Exact Next Moves

1. Read the Iron Knight climb/flip routines in `.cache/upstream/kid-chameleon-disasm/kid.asm` that correspond to the native probes `kid_flip_branch_probe`, `kid_flip_follow_on_probe`, and `kid_iron_knight_attach_probe`. Do not reopen `loc_C22` unless new evidence points back to timeout logic.
2. Trace frames `8054..8057` only. Compare native vs oracle for held inputs, wall contact, climb state, and when oracle first injects the leftward release motion that native misses.
3. Fix only that climb drift so the first authoritative divergence moves past frame `8056`, then rerun `tas-diff --frame-start 8054 --frame-end 8061 --include-physics-trace --include-advisory`, the 6975..9530 bisect, and `mise run verify-slice-fast`.

## Verification To Re-run

- `cargo test -p kid-core published_tas_frame_8004_keeps_demo_1_in_live_climb_gameplay -- --nocapture`
- `cargo test -p kid-core published_tas_demo_1_times_out_on_frame_8069 -- --nocapture`
- `cargo test -p kid-core published_tas_demo_2_freezes_the_timeout_snapshot_on_frame_4966 -- --nocapture`
- `cargo test -p kid-core 7968 -- --nocapture`
- `cargo test -p kid-core 7969 -- --nocapture`
- `cargo test -p kid-core iron_knight_ -- --nocapture`
- `cargo run -p kidcli -- tas-diff --scenario .cache/tas-imports/any-percent-published/scenario.json --frame-start 8054 --frame-end 8061 --include-physics-trace --include-advisory`
- `cargo run -p kidcli -- verify-tas --scenario .cache/tas-imports/any-percent-published/scenario.json --frame-start 6975 --frame-end 9530 --bisect`
- `mise run verify-slice-fast`

## Dead Ends / Do Not Retry

- Do not keep iterating the old "8004 timeout/title handoff" theory. Oracle RAM already shows the real timeout boundary is `8068 -> 8069`.
- Do not restore `DEMO_1_TIMEOUT_COUNTER_OFFSET_FRAMES = 3`; that reintroduces the frame-8004 miss immediately.
- Do not collapse demo 1 and demo 2 back into one shared timeout-freeze rule. Demo 2 still needs its frozen threshold snapshot; demo 1 does not.
- Do not reopen the frame-7969 Iron Knight climb-resume work without fresh oracle evidence. The new blocker is later and more specific.
