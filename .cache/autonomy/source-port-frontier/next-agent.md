---
slug: any-percent-published-frame-9932-intro-sequence-4-still-times-out-before-sub-1c204-finishes
cycle: 47
updated_at: 2026-03-24T01:50:07Z
---

# Source-Port Frontier Handoff

## What Changed

- Re-read `Load_IntroSequence4`, `sub_1B988`, `loc_1B9B2`, and then the shared helper `sub_1C204` in `.cache/upstream/kid-chameleon-disasm/kid.asm`.
- Changed `[constants.rs](/home/connor/git/kc/crates/kid-core/src/constants.rs)` so the timeout-restart intro-sequence-4 override no longer counts only the raw `loc_1B9B2` `0x21C` DBF loop from frame 9355; it now also includes the proven 37-frame delay before `sub_1B988` becomes visible.
- Added `[tests.rs](/home/connor/git/kc/crates/kid-core/src/tests.rs)` regression `published_tas_second_timeout_restart_keeps_intro_sequence_4_alive_on_frame_9895`.
- Verified the old frame-9895 blocker is gone and the frontier moved forward.

## Current Model

- The previous native bug was real: segment 3 started timing from the visible `Game_Mode=$20` hand-off on frame 9355, but the ROM does not even run `sub_1B988` until the delayed camera hand-off window that first appears on frame 9392.
- Adding that 37-frame delay shifts the first CRC divergence from frame 9895 to frame 9932 exactly, so the fix is directionally correct.
- The remaining gap is now the `sub_1C204` fade-in that `sub_1B988` calls before jumping into `loc_1B9B2`. `sub_1C204` hibernates 17 frames (`$FFFFF876 += 0x10` until it becomes `> 0x100`) before it dispatches `$44(a5)`, so the `0x21C` DBF loop still starts later than the first visible `camera_y_pos=4` frame.
- The close-up around frame 9946 shows oracle stays in intro sequence 4 through frame 9948, then reaches `game_mode=0x0024` on frame 9949 while still keeping `camera_y_pos=0x00040000`. That means the next fix is likely two-part: finish the missing `sub_1C204` delay, then keep the timeout-restart camera carry alive into intro sequence 5 instead of clearing it on the segment-3 -> segment-4 transition.

## Strongest Evidence

- `.cache/upstream/kid-chameleon-disasm/kid.asm`
  - `Load_IntroSequence4` only allocates the `sub_1B988` worker object.
  - `sub_1B988` writes `move.w #4,(Camera_Y_pos).w`, stores `#loc_1B9B2,$44(a5)`, then calls `sub_1C204`.
  - `sub_1C204` runs a fade loop: `Hibernate_Object_1Frame`, `addi.w #$10,($FFFFF876).w`, `cmpi.w #$100,($FFFFF876).w`, and only when that value is `> $100` does it `move.l $44(a5),a0` / `jmp (a0)`.
  - `loc_1B9B2` then runs `move.w #$21B,d0` / `dbf d0,loc_1B9B6` before `move.w #$24,(Game_Mode).w`.
- Native/oracle diagnostics
  - `cargo run -q -p kidcli -- tas-diff --scenario .cache/tas-imports/any-percent-published/scenario.json --frame-start 9892 --frame-end 9905 --include-advisory`
    - frames `9892..9904`: advisory-only (`number_objects`)
  - `cargo run -q -p kidcli -- verify-tas --scenario .cache/tas-imports/any-percent-published/scenario.json --frame-start 9531 --bisect`
    - `RESULT: first CRC divergence at frame 9932`
  - `cargo run -q -p kidcli -- tas-diff --scenario .cache/tas-imports/any-percent-published/scenario.json --frame-start 9929 --frame-end 9936 --include-advisory`
    - frames `9929..9931`: advisory-only (`number_objects`)
    - frame `9932`: `game_mode native=0x0024 oracle=0x0020`, `camera_y_pos native=0x00000000 oracle=0x00040000`
  - `cargo run -q -p kidcli -- tas-diff --scenario .cache/tas-imports/any-percent-published/scenario.json --frame-start 9946 --frame-end 9953 --include-advisory`
    - frames `9946..9948`: still `game_mode native=0x0024 oracle=0x0020`
    - frame `9949`: `game_mode` aligns, but `camera_y_pos native=0x00000000 oracle=0x00040000`
- Native code
  - `[constants.rs](/home/connor/git/kc/crates/kid-core/src/constants.rs)` now models the proven 37-frame `sub_1B988` hand-off delay before the `0x21C` loop.
  - `[tests.rs](/home/connor/git/kc/crates/kid-core/src/tests.rs)` locks down the `9894 clean / 9895 still intro-sequence-4` window.

## Exact Next Moves

1. Re-read `sub_1B988`, `sub_1C204`, and `loc_1B9B2` together and model the exact `sub_1C204` delay in native terms; do not start the `0x21C` loop from the first visible `camera_y_pos=4` frame.
2. Add a focused regression for the next proven boundary: `9948` still `GameMode::IntroSequence4`, `9949` reaches `GameMode::IntroSequence5` but keeps `camera_y_pos=0x0004_0000`.
3. Adjust the timeout-restart intro-sequence-4/5 hand-off so the segment-3 timer includes `sub_1C204`, and do not clear the timeout-restart intro-4 camera carry before the intro-sequence-5 window has finished with it. Then re-run the frame-9531 bisect.

## Verification To Re-run

- `cargo test -p kid-core published_tas_second_timeout_restart_delays_intro_sequence_4_camera_until_sub_1b988 -- --nocapture`
- `cargo test -p kid-core published_tas_second_timeout_restart_keeps_intro_sequence_4_alive_on_frame_9435 -- --nocapture`
- `cargo test -p kid-core published_tas_second_timeout_restart_keeps_intro_sequence_4_alive_on_frame_9895 -- --nocapture`
- `cargo run -q -p kidcli -- tas-diff --scenario .cache/tas-imports/any-percent-published/scenario.json --frame-start 9892 --frame-end 9905 --include-advisory`
- `cargo run -q -p kidcli -- verify-tas --scenario .cache/tas-imports/any-percent-published/scenario.json --frame-start 9531 --bisect`
- `cargo run -q -p kidcli -- tas-diff --scenario .cache/tas-imports/any-percent-published/scenario.json --frame-start 9929 --frame-end 9936 --include-advisory`
- `cargo run -q -p kidcli -- tas-diff --scenario .cache/tas-imports/any-percent-published/scenario.json --frame-start 9946 --frame-end 9953 --include-advisory`
- `mise run verify-slice-fast`

## Dead Ends / Do Not Retry

- Do not revert to the shared 80-frame intro timer; that reintroduces the already-fixed frame-9435 blocker immediately.
- Do not count only the raw `0x21C` `loc_1B9B2` loop from frame 9355; that was the original frame-9895 bug.
- Do not assume the first visible `camera_y_pos=4` frame means `loc_1B9B2` already started. `sub_1C204` sits in between and adds more delay.
- Do not paper over frame 9932 or frame 9949 with trace-only camera special-cases. The ROM order is now clear enough to fix in the boot-state model.
