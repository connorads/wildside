---
slug: any-percent-published-frame-10007-intro-sequence-5-native-still-hides-kid-after-camera-clears
cycle: 49
updated_at: 2026-03-24T02:16:00Z
---

# Source-Port Frontier Handoff

## What Changed

- Re-read `Load_IntroSequence5`, `loc_1BA28`, `loc_1BCD0`, `sub_1B222`, and `sub_1B988` in `.cache/upstream/kid-chameleon-disasm/kid.asm`.
- Corrected the prior model: `Load_IntroSequence5` does **not** call `sub_1B222`; the only visible `sub_1B222` callers here are `Load_IntroSequence6` and `GfxObjects_Collision_BottomBoundary4`.
- Added `[tests.rs](/home/connor/git/kc/crates/kid-core/src/tests.rs)` regression `published_tas_second_timeout_restart_clears_intro_sequence_5_camera_on_frame_9980`.
- Narrowed the timeout-restart intro-sequence-5 camera carry in `[trace.rs](/home/connor/git/kc/crates/kid-core/src/trace.rs)` to the oracle-backed 31-frame window after the `sub_1B988` hand-off.
- Verified the old frame-9980 camera blocker is gone; the first CRC divergence now moves to frame 10007.

## Current Model

- The old bug was native overextending the `sub_1B988` `Camera_Y_pos=4` carry across all of intro sequence 5.
- ASM now rules out the previous `sub_1B222` explanation: `Load_IntroSequence5` only allocates `loc_1BA28` and `loc_1BCD0`, so the frame-9980 fix is a carry-expiry boundary, not a missing `sub_1B222` call.
- Oracle-backed traces keep the carried camera through frame 9979 and clear it on frame 9980 while `game_mode` stays `0x0024` (`IntroSequence5`).
- The new frontier is frame 10007. Camera now matches, but native still has no Kid/title objects visible while oracle has already raised `kid_y_pos` to `0x00E0_0000` with higher object count.
- The next causal path likely lives inside the intro-sequence-5 workers `loc_1BA28` / `loc_1BCD0` and their spawned objects (`loc_1BC36`, `loc_1BE3C`, `loc_1BE88`), not in the old camera carry.

## Strongest Evidence

- `.cache/upstream/kid-chameleon-disasm/kid.asm`
  - `Load_IntroSequence5` allocates `#loc_1BA28` and `#loc_1BCD0`, then returns.
  - `sub_1B222` is called at `Load_IntroSequence6` and `GfxObjects_Collision_BottomBoundary4`, not from `Load_IntroSequence5`.
  - `sub_1B988` still provides the earlier `move.w #4,(Camera_Y_pos).w` hand-off.
- Native/oracle diagnostics
  - `cargo test -q -p kid-core published_tas_second_timeout_restart_ -- --nocapture`
    - all 7 focused timeout-restart regressions pass
  - `cargo run -q -p kidcli -- tas-diff --scenario .cache/tas-imports/any-percent-published/scenario.json --frame-start 9946 --frame-end 9953 --include-advisory`
    - frames `9946..9952`: advisory-only (`number_objects`)
  - `cargo run -q -p kidcli -- tas-diff --scenario .cache/tas-imports/any-percent-published/scenario.json --frame-start 9976 --frame-end 9984 --include-advisory`
    - frames `9976..9979`: advisory-only (`number_objects`)
    - frames `9980..9983`: `MATCH`
  - `cargo run -q -p kidcli -- verify-tas --scenario .cache/tas-imports/any-percent-published/scenario.json --frame-start 9531 --bisect`
    - `RESULT: first CRC divergence at frame 10007`
  - `cargo run -q -p kidcli -- tas-diff --scenario .cache/tas-imports/any-percent-published/scenario.json --frame-start 10003 --frame-end 10011 --include-advisory`
    - frames `10003..10005`: `MATCH`
    - frame `10006`: advisory-only (`number_objects`)
    - frame `10007`: `kid_y_pos native=0x00000000 oracle=0x00e00000`, plus advisory `number_objects native=0 oracle=4`

## Exact Next Moves

1. Read `loc_1BA28`, `loc_1BC36`, `loc_1BCD0`, `loc_1BE3C`, and `loc_1BE88` together and map which intro-sequence-5 worker first makes Kid/title objects visible around frame 10007.
2. Add a focused regression for the new boundary: frame `10006` still matches Kid hidden, frame `10007` should keep `GameMode::IntroSequence5` but raise `kid_y_pos` to `0x00E0_0000`.
3. Fix the missing intro-sequence-5 object/Kid visibility hand-off in native, then re-run the frame-9531 bisect.

## Verification To Re-run

- `cargo test -q -p kid-core published_tas_second_timeout_restart_ -- --nocapture`
- `cargo run -q -p kidcli -- tas-diff --scenario .cache/tas-imports/any-percent-published/scenario.json --frame-start 9946 --frame-end 9953 --include-advisory`
- `cargo run -q -p kidcli -- tas-diff --scenario .cache/tas-imports/any-percent-published/scenario.json --frame-start 9976 --frame-end 9984 --include-advisory`
- `cargo run -q -p kidcli -- tas-diff --scenario .cache/tas-imports/any-percent-published/scenario.json --frame-start 10003 --frame-end 10011 --include-advisory`
- `cargo run -q -p kidcli -- verify-tas --scenario .cache/tas-imports/any-percent-published/scenario.json --frame-start 9928 --frame-end 9953 --checkpoint-interval 1`
- `cargo run -q -p kidcli -- verify-tas --scenario .cache/tas-imports/any-percent-published/scenario.json --frame-start 9531 --bisect`
- `mise run verify-slice-fast`

## Dead Ends / Do Not Retry

- Do not assume `Load_IntroSequence5` calls `sub_1B222`; it does not in `kid.asm`.
- Do not re-extend the carried `sub_1B988` camera across all of intro sequence 5; oracle clears it on frame 9980.
- Do not reopen the old frame-9949 boundary by clearing the carry on the intro-sequence-4 -> intro-sequence-5 transition; frames 9949 and 9950 still require `camera_y_pos=0x0004_0000`.
- Do not add more debug fields for the old camera theory. The next answer is in the intro-sequence-5 worker ASM.
