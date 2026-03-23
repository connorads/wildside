---
slug: any-percent-published-frame-8002-demo-1-timeout-handoff-still-early-after-climb-fix
cycle: 31
updated_at: 2026-03-23T02:35:45Z
---

# Source-Port Frontier Handoff

## What Changed

- Split the demo auto-timeout model by recording: demo 1 now subtracts a one-frame timeout-counter offset, while demo 2 keeps the existing 61-frame offset.
- Stopped arming the same-frame frozen timeout snapshot for demo 1; only demo 2 still freezes the threshold frame.
- Added `published_tas_frame_8000_stays_on_live_climb_gameplay` and `published_tas_frame_8001_delays_demo_1_timeout_handoff_by_one_more_climb_step`.
- Re-verified that demo 2 still preserves the frozen timeout snapshot at frame 4966.

## Current Model

- The frame-8000 blocker was real, but it was a shared timeout-model bug, not another Iron Knight climb bug.
- Demo 1 (Knights Isle) stays in live climb gameplay through frames 8000 and 8001 once the timeout counter lags by one frame and the same-frame freeze is disabled for demo 1.
- Demo 2 (Skydragon Castle) still needs the frozen threshold snapshot on frame 4966, so demo 1 and demo 2 cannot share one timeout-freeze rule.
- The first CRC divergence has moved to frame 8002. Native now flips to the mode-0/title handoff there, while oracle still does not.

## Strongest Evidence

- `cargo test -p kid-core published_tas_frame_8000_stays_on_live_climb_gameplay -- --nocapture`
- `cargo test -p kid-core published_tas_frame_8001_delays_demo_1_timeout_handoff_by_one_more_climb_step -- --nocapture`
- `cargo test -p kid-core published_tas_demo_2_freezes_the_timeout_snapshot_on_frame_4966 -- --nocapture`
- `cargo run -p kidcli -- tas-diff --scenario .cache/tas-imports/any-percent-published/scenario.json --frame-start 7998 --frame-end 8002 --include-physics-trace --include-advisory`
  - Frames 7998-8001 are advisory-only; frame 8000 ends at native `kid_y_pos=0x01BA6D00`, frame 8001 ends at `0x01BA1780`, both on `motion=iron_knight_climb_resume`.
- `cargo run -p kidcli -- verify-tas --scenario .cache/tas-imports/any-percent-published/scenario.json --frame-start 6975 --frame-end 9530 --bisect`
  - `RESULT: first CRC divergence at frame 8002`
- `mise run verify-slice-fast`

## Exact Next Moves

1. Read the ROM demo timeout/title-return path again around the demo input compare and demo object wake (`loc_C22` and `loc_119D8` in `kid.asm`) and explain why oracle still avoids the mode-0 handoff on frame 8002.
2. Trace native demo-1 timeout arming across frames 8001-8002 only. Compare `demo_timeout_counter`, `demo_auto_timeout_return_pending`, and the title-return entry against the oracle run. Do not touch Iron Knight climb code.
3. Fix only the remaining demo-1 timeout handoff timing so the first authoritative divergence moves past 8002, then rerun the frame-7998..8002 diff, the 6975..9530 bisect, and `mise run verify-slice-fast`.

## Verification To Re-run

- `cargo test -p kid-core published_tas_frame_8000_stays_on_live_climb_gameplay -- --nocapture`
- `cargo test -p kid-core published_tas_frame_8001_delays_demo_1_timeout_handoff_by_one_more_climb_step -- --nocapture`
- `cargo test -p kid-core published_tas_demo_2_freezes_the_timeout_snapshot_on_frame_4966 -- --nocapture`
- `cargo test -p kid-core 7968 -- --nocapture`
- `cargo test -p kid-core 7969 -- --nocapture`
- `cargo test -p kid-core iron_knight_ -- --nocapture`
- `cargo run -p kidcli -- tas-diff --scenario .cache/tas-imports/any-percent-published/scenario.json --frame-start 7998 --frame-end 8002 --include-physics-trace --include-advisory`
- `cargo run -p kidcli -- verify-tas --scenario .cache/tas-imports/any-percent-published/scenario.json --frame-start 6975 --frame-end 9530 --bisect`
- `mise run verify-slice-fast`

## Dead Ends / Do Not Retry

- Do not collapse demo 1 and demo 2 back into one shared timeout-freeze rule. Demo 2 still needs the frozen threshold frame; demo 1 does not.
- Do not reopen the frame-7969 Iron Knight climb-resume work. That fix is still holding; the live blocker is later and lives in demo timeout/title-return timing.
- Do not restore unconditional `demo_auto_timeout_freeze_gameplay_frame = true`; that reintroduces the frame-8000 freeze immediately.
