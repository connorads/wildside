---
slug: any-percent-published-frame-8003-demo-1-timeout-handoff-still-early-after-second-lag-fix
cycle: 32
updated_at: 2026-03-23T02:47:41Z
---

# Source-Port Frontier Handoff

## What Changed

- Re-read the ROM timeout path at `loc_C22` and the demo Start-abort coroutine at `loc_119D8`; the key detail is still the pre-increment compare in `ReadJoypad`, not the coroutine wake-up path.
- Corrected the Knights Isle timeout model again: demo 1 now subtracts a two-frame timeout-counter offset instead of one.
- Added `published_tas_frame_8002_keeps_demo_1_in_live_climb_gameplay`.
- Re-ran the focused demo timeout regressions, the 7968/7969 and `iron_knight_` suites, `tas-diff --frame-start 7998 --frame-end 8002 --include-physics-trace --include-advisory`, `verify-tas --frame-start 8002 --frame-end 8010 --checkpoint-interval 1`, `verify-tas --frame-start 6975 --frame-end 9530 --bisect`, and `mise run verify-slice-fast`.

## Current Model

- Demo 1 needed a second timeout-counter lag frame because `loc_C22` compares the stored counter before incrementing it; the frame where the counter first reaches `$03F2` is still live gameplay, and the mode-0 handoff is only visible on the following frame.
- With the two-frame lag, demo 1 now stays in live climb gameplay through frame 8002. Native flips to the mode-0/title handoff on frame 8003, while oracle still does not.
- Demo 2 (Skydragon Castle) still needs the frozen threshold snapshot on frame 4966, so demo 1 and demo 2 still cannot share one timeout-freeze rule.
- The live blocker remains demo-1 timeout/title-return timing only; the frame-7969 Iron Knight climb-resume fix is still holding.

## Strongest Evidence

- `cargo test -p kid-core published_tas_frame_8000_stays_on_live_climb_gameplay -- --nocapture`
- `cargo test -p kid-core published_tas_frame_8001_delays_demo_1_timeout_handoff_by_one_more_climb_step -- --nocapture`
- `cargo test -p kid-core published_tas_frame_8002_keeps_demo_1_in_live_climb_gameplay -- --nocapture`
- `cargo test -p kid-core published_tas_demo_2_freezes_the_timeout_snapshot_on_frame_4966 -- --nocapture`
- `cargo run -p kidcli -- verify-tas --scenario .cache/tas-imports/any-percent-published/scenario.json --frame-start 8002 --frame-end 8010 --checkpoint-interval 1`
  - `frame_8002` is now advisory-only (`number_objects`, `kid_y_vel`), while `frame_8003` is the first authoritative miss: native `game_mode=0x0000`, `current_level_id=0x0000`, `kid_y_pos=0x01B9BD00`; oracle stays on `game_mode=0x0010`, `current_level_id=0x0015`, `kid_y_pos=0x01B95E00`.
- `cargo run -p kidcli -- tas-diff --scenario .cache/tas-imports/any-percent-published/scenario.json --frame-start 7998 --frame-end 8002 --include-physics-trace --include-advisory`
  - Frames 7998-8001 remain advisory-only and stay on `motion=iron_knight_climb_resume`; gameplay-mode `tas-diff` does not show the mode-0 handoff once native leaves gameplay, so use checkpointed `verify-tas` for this frontier.
- `cargo run -p kidcli -- verify-tas --scenario .cache/tas-imports/any-percent-published/scenario.json --frame-start 6975 --frame-end 9530 --bisect`
  - `RESULT: first CRC divergence at frame 8003`
- `mise run verify-slice-fast`

## Exact Next Moves

1. Read the ROM demo timeout/title-return path again around the demo input compare and demo object wake (`loc_C22` and `loc_119D8` in `kid.asm`) and explain why oracle still avoids the mode-0 handoff on frame 8003.
2. Trace native demo-1 timeout arming across frames 8002-8003 only. Compare `demo_timeout_counter`, `demo_auto_timeout_return_pending`, and the title-return entry against the oracle run. Do not touch Iron Knight climb code.
3. Fix only the remaining demo-1 timeout handoff timing so the first authoritative divergence moves past 8003, then rerun `verify-tas --frame-start 8002 --frame-end 8010 --checkpoint-interval 1`, the 6975..9530 bisect, and `mise run verify-slice-fast`.

## Verification To Re-run

- `cargo test -p kid-core published_tas_frame_8000_stays_on_live_climb_gameplay -- --nocapture`
- `cargo test -p kid-core published_tas_frame_8001_delays_demo_1_timeout_handoff_by_one_more_climb_step -- --nocapture`
- `cargo test -p kid-core published_tas_frame_8002_keeps_demo_1_in_live_climb_gameplay -- --nocapture`
- `cargo test -p kid-core published_tas_demo_2_freezes_the_timeout_snapshot_on_frame_4966 -- --nocapture`
- `cargo test -p kid-core 7968 -- --nocapture`
- `cargo test -p kid-core 7969 -- --nocapture`
- `cargo test -p kid-core iron_knight_ -- --nocapture`
- `cargo run -p kidcli -- tas-diff --scenario .cache/tas-imports/any-percent-published/scenario.json --frame-start 7998 --frame-end 8002 --include-physics-trace --include-advisory`
- `cargo run -p kidcli -- verify-tas --scenario .cache/tas-imports/any-percent-published/scenario.json --frame-start 8002 --frame-end 8010 --checkpoint-interval 1`
- `cargo run -p kidcli -- verify-tas --scenario .cache/tas-imports/any-percent-published/scenario.json --frame-start 6975 --frame-end 9530 --bisect`
- `mise run verify-slice-fast`

## Dead Ends / Do Not Retry

- Do not collapse demo 1 and demo 2 back into one shared timeout-freeze rule. Demo 2 still needs the frozen threshold frame; demo 1 does not.
- Do not reopen the frame-7969 Iron Knight climb-resume work. That fix is still holding; the live blocker is later and lives in demo timeout/title-return timing.
- Do not restore unconditional `demo_auto_timeout_freeze_gameplay_frame = true`; that reintroduces the frame-8000 freeze immediately.
- Do not assume gameplay-mode `tas-diff` alone will expose the remaining timeout/title-return miss. Once native flips out of gameplay, the authoritative `game_mode` / `current_level_id` break is only obvious in checkpointed `verify-tas`.
