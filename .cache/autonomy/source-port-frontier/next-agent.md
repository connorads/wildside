---
slug: any-percent-published-frame-9392-intro-sequence-4-camera-y-lags-oracle
cycle: 44
updated_at: 2026-03-24T01:01:08Z
---

# Source-Port Frontier Handoff

## What Changed

- Re-read `Load_IntroSequence4`, `sub_1B988`, `sub_1C204`, `loc_1C7A0`, and the surrounding intro loaders in `kid.asm`.
- Confirmed `Load_IntroSequence4` does not itself write `Camera_Y_pos`; it only allocates an object whose first routine is `sub_1B988`.
- Re-ran `cargo test -p kid-core published_tas_second_timeout_restart_carries_intro_sequence_3_pose_into_intro_sequence_4 -- --nocapture` and `cargo run -q -p kidcli -- tas-diff --scenario .cache/tas-imports/any-percent-published/scenario.json --frame-start 9308 --frame-end 9395 --include-advisory`.
- Proved the naive fix is wrong: keying intro-sequence-4 camera directly off native `BootState::IntroSequence { segment: 3 }` would move the camera change back to frame 9355 and regress the clean `9355..9391` window.
- Note: `next-task.json` is stale and still points at the older frame-7969 Knights Isle frontier. This handoff note is the correct fast-path context.

## Current Model

- The frame-9355 intro-sequence-4 carry bug is fixed. Native now matches oracle on authoritative fields through frame 9381 while the old `loc_1B96A` roster remains visible.
- Frame 9382 now clears the carried pose on both sides; from 9382 through 9391 the window is authoritative-clean apart from advisory `number_objects`.
- The live blocker is still frame 9392: native reports `camera_y_pos=0x00000000`, while oracle has `camera_y_pos=0x00040000`.
- `loc_1B96A` setting `Game_Mode=$20` on frame 9355 is not the same event as the visible `Camera_Y_pos=4` hand-off. The ROM only reaches that camera write when the `Load_IntroSequence4`-allocated object eventually runs `sub_1B988`.
- The missing native behaviour is therefore a late intro-sequence-4 subphase or dispatch delay, not a plain “segment 3 means camera=4” rule in `current_camera_state()`.

## Strongest Evidence

- `.cache/upstream/kid-chameleon-disasm/kid.asm`
  - `loc_1B96A` writes `Game_Mode=$20` after the intro-sequence-3 timeout.
  - `Load_IntroSequence4` only allocates an object with `4(a0)=sub_1B988`; it does not write `Camera_Y_pos` itself.
  - `sub_1B988` is the routine that finally writes `Camera_Y_pos=4`, allocates the intro-4 text object, then enters `sub_1C204`.
  - `loc_1BA28` / `loc_1BA5C` do not explain the 9355..9391 zero-camera window; the missing timing sits before or at the first `sub_1B988` dispatch.
- `cargo run -q -p kidcli -- tas-diff --scenario .cache/tas-imports/any-percent-published/scenario.json --frame-start 9308 --frame-end 9395 --include-advisory`
  - frames `9355..9381`: advisory-only (`number_objects`)
  - frame `9382`: advisory-only, carried pose gone on both sides
  - frames `9382..9391`: authoritative-clean apart from advisory `number_objects`
  - frame `9392`: first authoritative diff is `camera_y_pos` (`0x00000000` native vs `0x00040000` oracle)
- The rejected native patch model was: “if boot state is intro sequence 4/5, force `camera_y_pos=4`”.
  - That would have made frame `9355` diverge immediately, contradicting the oracle-clean `9355..9391` window.

## Exact Next Moves

1. Trace why `Load_IntroSequence4`'s `move.l #sub_1B988,4(a0)` does not surface as visible `Camera_Y_pos=4` until frame 9392: inspect the intro loader / object-dispatch timing, not just `current_camera_state()`.
2. Model that delayed `sub_1B988` hand-off in native boot progression without disturbing the fixed `9355..9381` pose carry or the clean `9382..9391` zero-camera window.
3. Add a focused regression for the `9381 clean / 9392 camera_y_pos=4` window once the exact camera timing is proven.

## Verification To Re-run

- `cargo test -p kid-core published_tas_second_timeout_restart_intro_sequence_3_visit_matches_oracle_frame_9312 -- --nocapture`
- `cargo test -p kid-core published_tas_second_timeout_restart_carries_intro_sequence_3_pose_into_intro_sequence_4 -- --nocapture`
- `cargo run -q -p kidcli -- tas-diff --scenario .cache/tas-imports/any-percent-published/scenario.json --frame-start 9308 --frame-end 9395 --include-advisory`
- `cargo run -q -p kidcli -- verify-tas --scenario .cache/tas-imports/any-percent-published/scenario.json --frame-start 6975 --frame-end 9530 --bisect`
- `mise run verify-slice-fast`

## Dead Ends / Do Not Retry

- Do not widen the intro-sequence-4 carry into a generic rule. Repo evidence still supports a timeout-restart-specific carry only.
- Do not special-case `current_camera_state()` off native intro-sequence segment `3` or `4`; that collapses the ROM's delayed `sub_1B988` dispatch and would regress frames `9355..9391`.
- Do not revisit the old intro-sequence-3 timer tweak. The 299-frame override is still correct and no longer the live blocker.
- Do not chase advisory `number_objects` drift in this window; the live frontier is now camera timing on frame 9392.
- Do not trust `next-task.json` over this handoff note until the machine task package is refreshed.
