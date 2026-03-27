---
slug: any-percent-published-frame-21142-intro-sequence-6-camera-ramp
cycle: 66
updated_at: 2026-03-27T01:00:00Z
---

# Source-Port Frontier Handoff

## What Changed

- Fixed the frame-20926 frontier by teaching the timeout-restart `IntroSequence6` trace to expose the first wrapped hidden camera row in [`crates/kid-core/src/trace.rs`](/home/connor/git/kc/crates/kid-core/src/trace.rs).
- Added [`published_tas_intro_sequence_6_wraps_hidden_camera_on_frame_20926`](/home/connor/git/kc/crates/kid-core/src/tests.rs), which locks the oracle-backed `camera_y_pos=0xFFF00000` hand-off on frame 20926 and confirms the row persists on 20927.
- Re-ran the nearby mode-$28 regressions plus `verify-slice-fast`; `verify-tas --frame-start 20895 --bisect` now moves the first CRC divergence to frame 21142.

## Current Model

- ASM [`Load_IntroSequence6`](.cache/upstream/kid-chameleon-disasm/kid.asm) seeds `($FFFFA656).w` with `$FFD00000` and immediately calls [`sub_1B222`](.cache/upstream/kid-chameleon-disasm/kid.asm), so the early mode-$28 camera is driven by the hidden title-camera accumulator rather than the generic cleared title camera.
- Native now matches the first wrapped row, but it still treats that wrapped value as a flat hold.
- Oracle continues ramping the same hidden camera accumulator during early mode `$28`: frame 21142 is `0xFFF10000`, then `0xFFF20000`, `0xFFF30000`, and so on.

## Strongest Evidence

- ASM
  - `.cache/upstream/kid-chameleon-disasm/kid.asm`
    - `Load_IntroSequence6` writes `move.l #$FFD00000,($FFFFA656).w` and calls `sub_1B222`.
    - `sub_1B222` copies `($FFFFA656).w` into `Camera_Y_pos`.
    - `GfxObjects_Collision_BottomBoundary4` repeatedly updates `($FFFFA656).w` and re-calls `sub_1B222`.
- Native
  - [`crates/kid-core/src/constants.rs`](/home/connor/git/kc/crates/kid-core/src/constants.rs)
    - `TIMEOUT_RESTART_INTRO_SEQUENCE_6_CAMERA_WRAP_ELAPSED_FRAMES` now records the oracle-backed first wrapped row at local mode-$28 frame 32.
  - [`crates/kid-core/src/trace.rs`](/home/connor/git/kc/crates/kid-core/src/trace.rs)
    - `intro_sequence_6_uses_timeout_restart_hidden_camera_wrap()` now exposes `camera_y_pos=0xFFF00000` for the timeout-restart `IntroSequence6` lineage once that wrapped row becomes visible.
  - [`crates/kid-core/src/tests.rs`](/home/connor/git/kc/crates/kid-core/src/tests.rs)
    - `published_tas_intro_sequence_6_wraps_hidden_camera_on_frame_20926` passes.
- Oracle / parity
  - `cargo run -q -p kidcli -- tas-diff --scenario .cache/tas-imports/any-percent-published/scenario.json --frame-start 20922 --frame-end 20930 --include-advisory`
    - Frames 20926..20929 now differ only on advisory `number_objects`.
  - `cargo run -q -p kidcli -- verify-tas --scenario .cache/tas-imports/any-percent-published/scenario.json --frame-start 20895 --bisect`
    - First CRC divergence is now frame 21142.
  - `cargo run -q -p kidcli -- tas-diff --scenario .cache/tas-imports/any-percent-published/scenario.json --frame-start 21136 --frame-end 21148 --include-advisory`
    - Frames 21142..21147 diverge on `camera_y_pos`: native stays `0xFFF00000`, oracle ramps `0xFFF10000`, `0xFFF20000`, `0xFFF30000`, `0xFFF40000`, `0xFFF50000`, `0xFFF60000`.

## Exact Next Moves

1. Read `Load_IntroSequence6`, `sub_1B222`, `sub_1B43A`, and `GfxObjects_Collision_BottomBoundary4` again and work out the exact per-frame `($FFFFA656).w` update that starts surfacing at frame 21142.
2. Replace the flat wrapped-camera hold with an accumulator-backed ramp for the timeout-restart `IntroSequence6` lineage.
3. Add one focused regression for frame 21142 only after the exact first post-wrap increment is proven.

## Verification To Re-run

- `cargo test -q -p kid-core published_tas_intro_return_timeout_revisit_enters_intro_sequence_6_on_frame_20894 -- --nocapture`
- `cargo test -q -p kid-core published_tas_intro_return_timeout_revisit_stays_in_intro_sequence_5_on_frame_20835 -- --nocapture`
- `cargo test -q -p kid-core published_tas_intro_sequence_6_clears_carried_slot_2_pose_on_frame_20912 -- --nocapture`
- `cargo test -q -p kid-core published_tas_intro_sequence_6_wraps_hidden_camera_on_frame_20926 -- --nocapture`
- `cargo run -q -p kidcli -- tas-diff --scenario .cache/tas-imports/any-percent-published/scenario.json --frame-start 20922 --frame-end 20930 --include-advisory`
- `cargo run -q -p kidcli -- tas-diff --scenario .cache/tas-imports/any-percent-published/scenario.json --frame-start 21136 --frame-end 21148 --include-advisory`
- `cargo run -q -p kidcli -- verify-tas --scenario .cache/tas-imports/any-percent-published/scenario.json --frame-start 20895 --bisect`
- `mise run verify-slice-fast`

## Dead Ends / Do Not Retry

- Do not reopen the frame-20926 blank-camera hold. The first wrapped row is now proven and covered by a focused regression.
- Do not treat this as another player visibility issue. Frames 21142..21147 diverge only on `camera_y_pos`; Kid stays hidden.
- Do not patch a second magic constant without tracing the `($FFFFA656).w` accumulator. The next mismatch is a ramp, not a one-off row.
