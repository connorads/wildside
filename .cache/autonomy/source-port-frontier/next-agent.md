---
slug: any-percent-published-frame-9435-intro-sequence-4-exits-too-early
cycle: 45
updated_at: 2026-03-24T01:11:51Z
---

# Source-Port Frontier Handoff

## What Changed

- Re-read `Load_IntroSequence4` and `sub_1B988` in `.cache/upstream/kid-chameleon-disasm/kid.asm`.
- Added a focused regression for the `9391 -> 9392` camera hand-off: `published_tas_second_timeout_restart_delays_intro_sequence_4_camera_until_sub_1b988`.
- Modelled the timeout-restart intro-sequence-4 camera gap in native trace state with `TIMEOUT_RESTART_INTRO_SEQUENCE_4_CAMERA_HANDOFF_FRAMES = 37`, matching the delayed `sub_1B988` `move.w #4,(Camera_Y_pos).w`.
- Verified the old frame-9392 camera mismatch is gone. `tas-diff --frame-start 9388 --frame-end 9394 --include-advisory` is now authoritative-clean; only advisory `number_objects` drift remains there.

## Current Model

- The frame-9392 blocker is fixed: native now keeps `camera_y_pos=0` through frame 9391, then exposes `camera_y_pos=0x00040000` on frame 9392 exactly like oracle.
- The new first authoritative divergence is frame 9435.
- At frame 9435 native switches to `game_mode=0x0024` (intro sequence 5) and drops `camera_y_pos` back to `0`, while oracle is still in `game_mode=0x0020` (intro sequence 4) with `camera_y_pos=0x00040000`.
- The next blocker is therefore no longer the delayed `sub_1B988` hand-off. It is the intro-sequence-4 duration itself: native still reuses the shared 80-frame segment timer, but ROM `loc_1B9B2` counts down `#$21B`.

## Strongest Evidence

- `.cache/upstream/kid-chameleon-disasm/kid.asm`
  - `Load_IntroSequence4` only allocates an object with `4(a0)=sub_1B988`.
  - `sub_1B988` immediately writes `move.w #4,(Camera_Y_pos).w`, then enters `sub_1C204`.
  - `loc_1B9B2` then runs `move.w #$21B,d0` and hibernates frame-by-frame before changing `Game_Mode` to `$24`.
- Native code
  - `[trace.rs](/home/connor/git/kc/crates/kid-core/src/trace.rs)` now has a timeout-restart intro-sequence-4 camera hand-off helper keyed off the proven frame-9392 threshold, and keeps that camera latch alive into segment 4 while the boot model still says intro sequence 5.
  - `[tests.rs](/home/connor/git/kc/crates/kid-core/src/tests.rs)` now has a regression for the `9391 -> 9392` hand-off.
- Oracle/native diagnostics
  - `cargo run -q -p kidcli -- tas-diff --scenario .cache/tas-imports/any-percent-published/scenario.json --frame-start 9388 --frame-end 9394 --include-advisory`
    - frames `9392..9393`: advisory-only (`number_objects`)
  - `cargo run -q -p kidcli -- verify-tas --scenario .cache/tas-imports/any-percent-published/scenario.json --frame-start 6975 --frame-end 9530 --bisect`
    - `RESULT: first CRC divergence at frame 9435`
  - `cargo run -q -p kidcli -- tas-diff --scenario .cache/tas-imports/any-percent-published/scenario.json --frame-start 9432 --frame-end 9438 --include-advisory`
    - frames `9432..9434`: advisory-only (`number_objects`)
    - frame `9435`: `game_mode native=0x0024 oracle=0x0020`, `camera_y_pos native=0x00000000 oracle=0x00040000`

## Exact Next Moves

1. Read `loc_1B9B2` in `kid.asm` and model intro-sequence-4 duration from that ROM timer, not the shared native 80-frame segment default.
2. Re-run `tas-diff --frame-start 9432 --frame-end 9438 --include-advisory` after the timer fix; frame 9435 should become advisory-only if the duration is correct.
3. Add a focused regression for the `9434 clean / 9435 still intro-sequence-4` window once the timer threshold is proven.

## Verification To Re-run

- `cargo test -p kid-core published_tas_second_timeout_restart_intro_sequence_3_visit_matches_oracle_frame_9312 -- --nocapture`
- `cargo test -p kid-core published_tas_second_timeout_restart_carries_intro_sequence_3_pose_into_intro_sequence_4 -- --nocapture`
- `cargo test -p kid-core published_tas_second_timeout_restart_delays_intro_sequence_4_camera_until_sub_1b988 -- --nocapture`
- `cargo run -q -p kidcli -- tas-diff --scenario .cache/tas-imports/any-percent-published/scenario.json --frame-start 9388 --frame-end 9394 --include-advisory`
- `cargo run -q -p kidcli -- verify-tas --scenario .cache/tas-imports/any-percent-published/scenario.json --frame-start 6975 --frame-end 9530 --bisect`
- `cargo run -q -p kidcli -- tas-diff --scenario .cache/tas-imports/any-percent-published/scenario.json --frame-start 9432 --frame-end 9438 --include-advisory`
- `mise run verify-slice-fast`

## Dead Ends / Do Not Retry

- Do not reintroduce a generic “intro sequence 4 means camera_y_pos=4 immediately” rule; that regresses the now-clean `9355..9391` zero-camera gap.
- Do not revisit `current_camera_state()` special-casing without an elapsed threshold. The ROM delay is real; only the thresholded hand-off matches oracle.
- Do not chase advisory `number_objects` drift in the `9388..9434` window; the live blocker is the early native transition out of intro sequence 4 on frame 9435.
- Do not trust `.cache/autonomy/source-port-frontier/next-task.json`; it is still stale and points at the older frame-7969 Knights Isle frontier.
