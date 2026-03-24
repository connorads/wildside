---
slug: any-percent-published-frame-9895-intro-sequence-4-still-times-out-too-early
cycle: 46
updated_at: 2026-03-24T01:35:45Z
---

# Source-Port Frontier Handoff

## What Changed

- Re-read `sub_1B988` / `loc_1B9B2` in `.cache/upstream/kid-chameleon-disasm/kid.asm`.
- Added a timeout-restart intro-sequence-4 duration override to native boot-state handling instead of reusing the shared 80-frame segment timer.
- Added `published_tas_second_timeout_restart_keeps_intro_sequence_4_alive_on_frame_9435` so the old `9434 clean / 9435 exits too early` window stays locked down.
- Verified the old frame-9435 blocker is gone: `tas-diff --frame-start 9432 --frame-end 9438 --include-advisory` is now advisory-only, and `verify-tas --frame-start 6975 --frame-end 9530 --bisect` is fully CRC-clean.

## Current Model

- The timeout-restart intro-sequence-4 segment can no longer use the shared 80-frame default; native now keeps `game_mode=0x0020` and `camera_y_pos=0x00040000` through frame 9435 like oracle.
- The next authoritative blocker after the cleared `6975..9530` slice is frame 9895.
- At frame 9895 native has already advanced to `game_mode=0x0024` with `camera_y_pos=0`, while oracle is still in `game_mode=0x0020` with `camera_y_pos=0x00040000`.
- The remaining gap is still the intro-sequence-4 exit timing, but the simple theory “just add one more frame to the native timer” is wrong; oracle stays in intro sequence 4 well past frame 9896.

## Strongest Evidence

- `.cache/upstream/kid-chameleon-disasm/kid.asm`
  - `sub_1B988` writes `move.w #4,(Camera_Y_pos).w`, allocates the intro text object, stores `#loc_1B9B2,$44(a5)`, then calls `sub_1C204`.
  - `loc_1B9B2` runs `move.w #$21B,d0`, hibernates with `dbf`, then writes `move.w #$24,(Game_Mode).w`.
- Native code
  - `[boot.rs](/home/connor/git/kc/crates/kid-core/src/boot.rs)` now carries a timeout-restart intro-sequence-4 frame override through the boot-state segment scheduler.
  - `[constants.rs](/home/connor/git/kc/crates/kid-core/src/constants.rs)` records the new intro-sequence-4 override and cites `sub_1B988` / `loc_1B9B2`.
  - `[tests.rs](/home/connor/git/kc/crates/kid-core/src/tests.rs)` now locks down the old frame-9435 window.
- Oracle/native diagnostics
  - `cargo run -q -p kidcli -- tas-diff --scenario .cache/tas-imports/any-percent-published/scenario.json --frame-start 9432 --frame-end 9438 --include-advisory`
    - frames `9432..9437`: advisory-only (`number_objects`)
  - `cargo run -q -p kidcli -- verify-tas --scenario .cache/tas-imports/any-percent-published/scenario.json --frame-start 6975 --frame-end 9530 --bisect`
    - `PASS: all 9530 frames match by CRC`
  - `cargo run -q -p kidcli -- verify-tas --scenario .cache/tas-imports/any-percent-published/scenario.json --frame-start 9531 --bisect`
    - `RESULT: first CRC divergence at frame 9895`
  - `cargo run -q -p kidcli -- tas-diff --scenario .cache/tas-imports/any-percent-published/scenario.json --frame-start 9892 --frame-end 9898 --include-advisory`
    - frames `9892..9894`: advisory-only (`number_objects`)
    - frame `9895`: `game_mode native=0x0024 oracle=0x0020`, `camera_y_pos native=0x00000000 oracle=0x00040000`

## Exact Next Moves

1. Read `Load_IntroSequence4`, `sub_1B988`, and `loc_1B9B2` together and pin down when the `loc_1B9B2` countdown actually starts relative to the carried pose clear and delayed `camera_y_pos=4` hand-off; do not assume the timer starts on frame 9355 just because `Game_Mode=$20` is already visible.
2. Re-run `tas-diff --frame-start 9892 --frame-end 9905 --include-advisory` while tracing the native `segment=3 elapsed` value so you can map the first bad frame to the real ROM countdown start, not the guessed boot-state entry frame.
3. Once that threshold is proven, add a focused regression for the `9894 clean / 9895 still intro-sequence-4` window and then re-run `verify-tas --frame-start 9531 --bisect`.

## Verification To Re-run

- `cargo test -p kid-core published_tas_second_timeout_restart_intro_sequence_3_visit_matches_oracle_frame_9312 -- --nocapture`
- `cargo test -p kid-core published_tas_second_timeout_restart_delays_intro_sequence_4_camera_until_sub_1b988 -- --nocapture`
- `cargo test -p kid-core published_tas_second_timeout_restart_keeps_intro_sequence_4_alive_on_frame_9435 -- --nocapture`
- `cargo run -q -p kidcli -- tas-diff --scenario .cache/tas-imports/any-percent-published/scenario.json --frame-start 9432 --frame-end 9438 --include-advisory`
- `cargo run -q -p kidcli -- verify-tas --scenario .cache/tas-imports/any-percent-published/scenario.json --frame-start 6975 --frame-end 9530 --bisect`
- `cargo run -q -p kidcli -- verify-tas --scenario .cache/tas-imports/any-percent-published/scenario.json --frame-start 9531 --bisect`
- `cargo run -q -p kidcli -- tas-diff --scenario .cache/tas-imports/any-percent-published/scenario.json --frame-start 9892 --frame-end 9898 --include-advisory`
- `mise run verify-slice-fast`

## Dead Ends / Do Not Retry

- Do not revert to the shared 80-frame intro timer; that reintroduces the fixed frame-9435 blocker immediately.
- Do not special-case `current_camera_state()` again for frame 9895 without fixing the underlying segment timing; the camera diff is still downstream of the early mode exit.
- Do not assume `move.w #$21B,d0` means “add exactly one more native frame” from the visible `game_mode=0x0020` entry. I tried the naive extra-frame tweak (`0x21D`) and oracle still stayed in intro sequence 4 well beyond frame 9896.
