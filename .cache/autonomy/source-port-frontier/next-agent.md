---
slug: any-percent-published-frame-9392-intro-sequence-4-camera-y-lags-oracle
cycle: 43
updated_at: 2026-03-24T00:51:47Z
---

# Source-Port Frontier Handoff

## What Changed

- Re-read `Load_IntroSequence4`, `sub_1B988`, and `loc_1B96A` in `kid.asm`.
- Added a narrow timeout-restart intro-sequence-4 carry flag so native keeps the second `Load_IntroSequence3` roster pose visible after `loc_1B96A` flips `Game_Mode` to `$20`.
- Replaced the old frame-9355 `Game_Mode` assertion with `published_tas_second_timeout_restart_carries_intro_sequence_3_pose_into_intro_sequence_4`, which pins frames 9355, 9381, and 9382 against oracle.
- Re-ran the focused regressions, `tas-diff 9308..9395`, `verify-tas --frame-start 6975 --frame-end 9530 --bisect`, and `mise run verify-slice-fast`.
- Note: `next-task.json` is stale and still points at the older frame-7969 Knights Isle frontier. This handoff note is the correct fast-path context.

## Current Model

- The frame-9355 intro-sequence-4 carry bug is fixed. Native now matches oracle on authoritative fields through frame 9381 while the old `loc_1B96A` roster remains visible.
- Frame 9382 now clears the carried pose on both sides; from 9382 through 9391 the window is authoritative-clean apart from advisory `number_objects`.
- The new first authoritative divergence is frame 9392: native still reports `camera_y_pos=0x00000000`, while oracle has `camera_y_pos=0x00040000`.
- The remaining blocker is therefore the early intro-sequence-4 camera hand-off, not pose carry.

## Strongest Evidence

- `.cache/upstream/kid-chameleon-disasm/kid.asm`
  - `loc_1B96A` writes `Game_Mode=$20` after the intro-sequence-3 timeout.
  - `sub_1B988` starts intro sequence 4 by writing `Camera_Y_pos=4` and installing the intro-4 text object.
- `cargo run -q -p kidcli -- tas-diff --scenario .cache/tas-imports/any-percent-published/scenario.json --frame-start 9308 --frame-end 9395 --include-advisory`
  - frames `9355..9381`: advisory-only (`number_objects`)
  - frame `9382`: advisory-only, carried pose gone on both sides
  - frames `9382..9391`: authoritative-clean apart from advisory `number_objects`
  - frame `9392`: first authoritative diff is `camera_y_pos` (`0x00000000` native vs `0x00040000` oracle)
- `cargo run -q -p kidcli -- verify-tas --scenario .cache/tas-imports/any-percent-published/scenario.json --frame-start 6975 --frame-end 9530 --bisect`
  - first CRC divergence moved from frame `9355` to frame `9392`

## Exact Next Moves

1. Read the intro-sequence-4 camera path around `sub_1B988`, `sub_1C204`, and any immediate follow-on labels that can delay when `Camera_Y_pos=4` becomes visible in the rendered frame stream.
2. Compare that ROM timing against native intro-sequence-4 camera handling in `current_camera_state()` / boot-state progression; patch the frame-9392 camera hand-off without disturbing the fixed 9355..9381 pose carry.
3. Add a focused regression for the `9381 clean / 9392 camera_y_pos=4` window once the exact camera timing is proven.

## Verification To Re-run

- `cargo test -p kid-core published_tas_second_timeout_restart_intro_sequence_3_visit_matches_oracle_frame_9312 -- --nocapture`
- `cargo test -p kid-core published_tas_second_timeout_restart_carries_intro_sequence_3_pose_into_intro_sequence_4 -- --nocapture`
- `cargo run -q -p kidcli -- tas-diff --scenario .cache/tas-imports/any-percent-published/scenario.json --frame-start 9308 --frame-end 9395 --include-advisory`
- `cargo run -q -p kidcli -- verify-tas --scenario .cache/tas-imports/any-percent-published/scenario.json --frame-start 6975 --frame-end 9530 --bisect`
- `mise run verify-slice-fast`

## Dead Ends / Do Not Retry

- Do not widen the intro-sequence-4 carry into a generic rule. Repo evidence still supports a timeout-restart-specific carry only.
- Do not revisit the old intro-sequence-3 timer tweak. The 299-frame override is still correct and no longer the live blocker.
- Do not chase advisory `number_objects` drift in this window; the live frontier is now camera timing on frame 9392.
- Do not trust `next-task.json` over this handoff note until the machine task package is refreshed.
