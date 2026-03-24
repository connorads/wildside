---
slug: any-percent-published-frame-9980-intro-sequence-5-native-keeps-timeout-restart-camera-carry-too-long
cycle: 48
updated_at: 2026-03-24T02:03:00Z
---

# Source-Port Frontier Handoff

## What Changed

- Re-read `sub_1B988`, `sub_1C204`, and `loc_1B9B2` in `.cache/upstream/kid-chameleon-disasm/kid.asm`.
- Updated the timeout-restart intro-sequence-4 override so native includes the proven 17-frame `sub_1C204` fade delay before `loc_1B9B2` starts.
- Kept the timeout-restart `Camera_Y_pos=4` carry alive across the intro-sequence-4 -> intro-sequence-5 hand-off instead of clearing it immediately.
- Added `[tests.rs](/home/connor/git/kc/crates/kid-core/src/tests.rs)` regression `published_tas_second_timeout_restart_carries_camera_into_intro_sequence_5_on_frame_9949`.
- Verified the old frame-9932 blocker is gone; the first CRC divergence now moves to frame 9980.

## Current Model

- The previous native bug was twofold: segment 3 still skipped the 17-frame `sub_1C204` fade delay after `sub_1B988`, and native also dropped the carried `Camera_Y_pos=4` on the intro-sequence-4 -> intro-sequence-5 transition.
- That part is now fixed. Oracle-facing checks show frames 9946..9952 are advisory-only, and frame 9949 now lines up as `GameMode::IntroSequence5` with `camera_y_pos=0x0004_0000`.
- The new frontier is frame 9980. Native still keeps `camera_y_pos=0x0004_0000` in intro sequence 5 while oracle has already cleared to `0x00000000`.
- The next causal path is `Load_IntroSequence5` plus `sub_1B222`: intro sequence 5 appears to replace the carried timeout-restart camera with `($FFFFA656).w`, and native is now holding the carry past that ROM-backed boundary.

## Strongest Evidence

- `.cache/upstream/kid-chameleon-disasm/kid.asm`
  - `sub_1B988` writes `move.w #4,(Camera_Y_pos).w`, stores `#loc_1B9B2,$44(a5)`, then calls `sub_1C204`.
  - `sub_1C204` loops on `Hibernate_Object_1Frame`, `addi.w #$10,($FFFFF876).w`, `cmpi.w #$100,($FFFFF876).w`, and only when that value is `> $100` does it jump through `$44(a5)`. That is a 17-frame delay.
  - `loc_1B9B2` then runs `move.w #$21B,d0` / `dbf d0,loc_1B9B6` before `move.w #$24,(Game_Mode).w`.
  - `Load_IntroSequence5` sets up the next intro scene, and `sub_1B222` writes `move.w ($FFFFA656).w,(Camera_Y_pos).w`.
- Native/oracle diagnostics
  - `cargo test -p kid-core published_tas_second_timeout_restart_ -- --nocapture`
    - all 6 focused intro-sequence timeout-restart regressions pass
  - `cargo run -q -p kidcli -- verify-tas --scenario .cache/tas-imports/any-percent-published/scenario.json --frame-start 9531 --bisect`
    - `RESULT: first CRC divergence at frame 9980`
  - `cargo run -q -p kidcli -- tas-diff --scenario .cache/tas-imports/any-percent-published/scenario.json --frame-start 9946 --frame-end 9953 --include-advisory`
    - frames `9946..9952`: advisory-only (`number_objects`)
  - `cargo run -q -p kidcli -- tas-diff --scenario .cache/tas-imports/any-percent-published/scenario.json --frame-start 9976 --frame-end 9984 --include-advisory`
    - frames `9976..9979`: advisory-only (`number_objects`)
    - frame `9980`: `camera_y_pos native=0x00040000 oracle=0x00000000`
  - `cargo run -q -p kidcli -- verify-tas --scenario .cache/tas-imports/any-percent-published/scenario.json --frame-start 9928 --frame-end 9953 --checkpoint-interval 1`
    - `CRC: all 25 frames match in 9928..9953`
    - use this plus the narrow `9946..9953` diff rather than over-reading the noisy wide `9929..9953` diff window

## Exact Next Moves

1. Re-read `Load_IntroSequence5` and `sub_1B222` together and pin down the exact ROM frame where intro sequence 5 replaces the carried `Camera_Y_pos=4` with `($FFFFA656).w`.
2. Add a focused regression for the new boundary: frame `9979` still keeps `camera_y_pos=0x0004_0000`, frame `9980` clears to `0x0000_0000` while staying in intro sequence 5.
3. Adjust the timeout-restart intro-sequence-5 camera setup so the carry expires on the ROM-backed `Load_IntroSequence5` / `sub_1B222` boundary, then re-run the frame-9531 bisect.

## Verification To Re-run

- `cargo test -p kid-core published_tas_second_timeout_restart_ -- --nocapture`
- `cargo run -q -p kidcli -- verify-tas --scenario .cache/tas-imports/any-percent-published/scenario.json --frame-start 9531 --bisect`
- `cargo run -q -p kidcli -- tas-diff --scenario .cache/tas-imports/any-percent-published/scenario.json --frame-start 9946 --frame-end 9953 --include-advisory`
- `cargo run -q -p kidcli -- tas-diff --scenario .cache/tas-imports/any-percent-published/scenario.json --frame-start 9976 --frame-end 9984 --include-advisory`
- `cargo run -q -p kidcli -- verify-tas --scenario .cache/tas-imports/any-percent-published/scenario.json --frame-start 9928 --frame-end 9953 --checkpoint-interval 1`
- `mise run verify-slice-fast`

## Dead Ends / Do Not Retry

- Do not drop back to counting only the raw `loc_1B9B2` `0x21C` DBF loop from the visible `Camera_Y_pos=4` frame; that reintroduces the already-fixed frame-9932 blocker.
- Do not clear the timeout-restart camera carry on the intro-sequence-4 -> intro-sequence-5 transition; that reintroduces the old frame-9949 mismatch immediately.
- Do not add more debug fields for this frontier. The next answer is already in `Load_IntroSequence5` / `sub_1B222`.
- Do not over-index on the wide `9929..9953` `tas-diff` window; use the checkpoint-interval verification plus the narrow `9946..9953` and `9976..9984` windows instead.
