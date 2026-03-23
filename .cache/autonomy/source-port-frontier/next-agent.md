---
slug: any-percent-published-frame-8004-demo-1-timeout-handoff-still-early-after-third-lag-fix
cycle: 33
updated_at: 2026-03-23T02:57:00Z
---

# Source-Port Frontier Handoff

## What Changed

- Re-read the ROM timeout path at `loc_C22` and the demo Start-abort coroutine at `loc_119D8`; the timeout compare is still the stored-counter check before increment, and the Start-abort helper is still a separate one-frame hibernate path.
- Added `published_tas_frame_8003_keeps_demo_1_in_live_climb_gameplay`.
- Moved `DEMO_1_TIMEOUT_COUNTER_OFFSET_FRAMES` from 2 to 3 so demo 1 stays live through frame 8003.
- Re-ran the focused demo timeout regressions, the 7968/7969 and `iron_knight_` suites, `tas-diff --frame-start 7998 --frame-end 8004 --include-physics-trace --include-advisory`, `verify-tas --frame-start 8002 --frame-end 8010 --checkpoint-interval 1`, `verify-tas --frame-start 6975 --frame-end 9530 --bisect`, and `mise run verify-slice-fast`.

## Current Model

- Demo 1 needed a third timeout-counter lag frame. With that fix, native now stays in live climb gameplay through frame 8003.
- The live blocker is still demo-1 timeout/title-return timing only. Native flips into the mode-0/title handoff on frame 8004, while oracle still does not.
- The frame-7969 Iron Knight climb-resume fix is still holding; do not reopen it.
- Demo 2 still needs its own timeout snapshot handling. Do not merge demo 1 and demo 2 back into one shared timeout rule.

## Strongest Evidence

- `cargo test -p kid-core published_tas_frame_8003_keeps_demo_1_in_live_climb_gameplay -- --nocapture`
- `cargo test -p kid-core published_tas_demo_2_freezes_the_timeout_snapshot_on_frame_4966 -- --nocapture`
- `cargo test -p kid-core 7968 -- --nocapture`
- `cargo test -p kid-core 7969 -- --nocapture`
- `cargo test -p kid-core iron_knight_ -- --nocapture`
- `cargo run -p kidcli -- tas-diff --scenario .cache/tas-imports/any-percent-published/scenario.json --frame-start 7998 --frame-end 8004 --include-physics-trace --include-advisory`
  - Frames 7998..8003 are advisory-only (`number_objects`, `kid_y_vel`); frame 8004 is the first authoritative miss: native `game_mode=0x0000`, `current_level_id=0x0000`, `kid_y_pos=0x01B95E00`; oracle stays on `game_mode=0x0010`, `current_level_id=0x0015`, `kid_y_pos=0x01B8FB00`.
- `cargo run -p kidcli -- verify-tas --scenario .cache/tas-imports/any-percent-published/scenario.json --frame-start 8002 --frame-end 8010 --checkpoint-interval 1`
  - `CRC divergence at frame 8004`
- `cargo run -p kidcli -- verify-tas --scenario .cache/tas-imports/any-percent-published/scenario.json --frame-start 6975 --frame-end 9530 --bisect`
  - `RESULT: first CRC divergence at frame 8004`
- `mise run verify-slice-fast`

## Exact Next Moves

1. Read the ROM demo timeout/title-return path again around `loc_C22` and the title entry it reaches after `j_loc_6E2` in `kid.asm`, and explain why oracle still avoids the visible mode-0 handoff on frame 8004.
2. Trace native demo-1 timeout arming across frames 8003-8004 only. Compare `frame_counter`, the derived demo timeout counter, `demo_auto_timeout_return_pending`, and the first visible title-return frame against oracle. Do not touch Iron Knight climb code.
3. Fix only the remaining demo-1 timeout handoff timing so the first authoritative divergence moves past 8004, then rerun `verify-tas --frame-start 8002 --frame-end 8010 --checkpoint-interval 1`, the 6975..9530 bisect, and `mise run verify-slice-fast`.

## Verification To Re-run

- `cargo test -p kid-core published_tas_frame_8000_stays_on_live_climb_gameplay -- --nocapture`
- `cargo test -p kid-core published_tas_frame_8001_delays_demo_1_timeout_handoff_by_one_more_climb_step -- --nocapture`
- `cargo test -p kid-core published_tas_frame_8002_keeps_demo_1_in_live_climb_gameplay -- --nocapture`
- `cargo test -p kid-core published_tas_frame_8003_keeps_demo_1_in_live_climb_gameplay -- --nocapture`
- `cargo test -p kid-core published_tas_demo_2_freezes_the_timeout_snapshot_on_frame_4966 -- --nocapture`
- `cargo test -p kid-core 7968 -- --nocapture`
- `cargo test -p kid-core 7969 -- --nocapture`
- `cargo test -p kid-core iron_knight_ -- --nocapture`
- `cargo run -p kidcli -- tas-diff --scenario .cache/tas-imports/any-percent-published/scenario.json --frame-start 7998 --frame-end 8004 --include-physics-trace --include-advisory`
- `cargo run -p kidcli -- verify-tas --scenario .cache/tas-imports/any-percent-published/scenario.json --frame-start 8002 --frame-end 8010 --checkpoint-interval 1`
- `cargo run -p kidcli -- verify-tas --scenario .cache/tas-imports/any-percent-published/scenario.json --frame-start 6975 --frame-end 9530 --bisect`
- `mise run verify-slice-fast`

## Dead Ends / Do Not Retry

- Do not collapse demo 1 and demo 2 back into one shared timeout-freeze rule. Demo 2 still needs its frozen threshold snapshot; demo 1 does not.
- Do not reopen the frame-7969 Iron Knight climb-resume work. That fix is still holding; the live blocker is later and lives in demo timeout/title-return timing.
- Do not restore `DEMO_1_TIMEOUT_COUNTER_OFFSET_FRAMES = 2`; that reintroduces the frame-8003 authoritative miss immediately.
- Do not assume gameplay-mode `tas-diff` alone will expose the remaining timeout/title-return miss. The authoritative `game_mode` / `current_level_id` break is clearest in checkpointed `verify-tas`.
