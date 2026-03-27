---
slug: any-percent-published-frame-21837-demo-bootstrap-pose-release
cycle: 67
updated_at: 2026-03-27T02:57:53Z
---

# Source-Port Frontier Handoff

## What Changed

- Fixed the frame-21825 timeout-restart demo-1 hidden camera wrap by carrying an `IntroSequence6`-specific demo bootstrap lineage into [`boot.rs`](/home/connor/git/kc/crates/kid-core/src/boot.rs) and [`trace.rs`](/home/connor/git/kc/crates/kid-core/src/trace.rs).
- Added [`DEMO_1_INTRO_SEQUENCE_6_CAMERA_WRAP_Y_FP`](/home/connor/git/kc/crates/kid-core/src/demo.rs) and [`published_tas_intro_sequence_6_uses_demo_1_hidden_camera_wrap_on_frame_21825`](/home/connor/git/kc/crates/kid-core/src/tests.rs), locking the oracle-backed `camera_y_pos=0xFD200000` on frame 21825.
- Re-ran the focused camera-wrap regressions, `tas-diff --frame-start 21820 --frame-end 21830 --include-advisory`, `verify-tas --frame-start 21765 --bisect`, and `mise run verify-slice-fast`; the first authoritative divergence now moves to frame 21837.

## Current Model

- ASM `loc_1BA5C` enters `Load_IntroSequence6`, and the later `Obj_TitleMenu loc_1BB02 -> Load_DemoPlay -> Load_InGame` hand-off preserves a distinct hidden demo-1 wrap for the timeout-restart lineage that came from intro sequence 5.
- Native now matches that hidden wrap on frame 21825 and stays authoritative-clean through frame 21836.
- The next blocker opens when the demo bootstrap starts surfacing gameplay state: native still holds the demo bootstrap pose one frame too long on frame 21837, so the camera climbs eight pixels too early and Kid appears at `(0x00100000,0x00FF0000)` while oracle still keeps `(kid_x_pos,kid_y_pos)=(0,0)`.

## Strongest Evidence

- ASM
  - `.cache/upstream/kid-chameleon-disasm/kid.asm`
    - `loc_1BA5C` flips into `Load_IntroSequence6`.
    - `loc_1BB02` flips `Game_Mode` to `$10`, then `Load_DemoPlay` immediately calls `Load_InGame`.
    - `Load_InGame` seeds `Camera_Y_pos` from `PlayerStart_Y_pos` and starts the normal level bootstrap.
- Native
  - [`boot.rs`](/home/connor/git/kc/crates/kid-core/src/boot.rs)
    - `begin_demo_play()` now remembers when demo playback came from `IntroSequence6` with the carried intro-sequence-5 lineage bit.
  - [`trace.rs`](/home/connor/git/kc/crates/kid-core/src/trace.rs)
    - Demo-1 hidden camera selection now distinguishes cold boot, Start-skip intro return, and `IntroSequence6`.
  - [`tests.rs`](/home/connor/git/kc/crates/kid-core/src/tests.rs)
    - `published_tas_intro_sequence_6_uses_demo_1_hidden_camera_wrap_on_frame_21825` passes.
- Oracle / parity
  - `cargo run -q -p kidcli -- tas-diff --scenario .cache/tas-imports/any-percent-published/scenario.json --frame-start 21820 --frame-end 21830 --include-advisory`
    - Frames 21824..21829 now differ only on advisory `number_objects`.
  - `cargo run -q -p kidcli -- verify-tas --scenario .cache/tas-imports/any-percent-published/scenario.json --frame-start 21765 --bisect`
    - First CRC divergence is now frame 21837.
  - `cargo run -q -p kidcli -- tas-diff --scenario .cache/tas-imports/any-percent-published/scenario.json --frame-start 21834 --frame-end 21842 --include-advisory`
    - Frame 21837 is the first authoritative break: `camera_y_pos native=0x00D80000 oracle=0x00E00000`, `kid_x_pos native=0x00100000 oracle=0x00000000`, `kid_y_pos native=0x00FF0000 oracle=0x00000000`.

## Exact Next Moves

1. Read the first visible demo-bootstrap path after `loc_1BB02 -> Load_DemoPlay -> Load_InGame` and trace why native releases the hidden demo bootstrap pose on frame 21837 while oracle keeps Kid hidden for one more frame.
2. Inspect the native demo bootstrap hold logic that currently reports `kid_gameplay_path_probe=demo_bootstrap_pose_hold` on frame 21837 and line it up with the ROM/bootstrap timing.
3. Add one focused regression for frame 21837 only after the exact pose/camera release is proven.

## Verification To Re-run

- `cargo test -q -p kid-core published_tas_intro_sequence_6_enters_demo_play_on_frame_21765 -- --nocapture`
- `cargo test -q -p kid-core published_tas_intro_sequence_6_uses_demo_1_hidden_camera_wrap_on_frame_21825 -- --nocapture`
- `cargo test -q -p kid-core demo_1_bootstrap_keeps_camera_hidden_through_frame_2537 -- --nocapture`
- `cargo test -q -p kid-core published_tas_intro_return_demo_1_uses_the_oracle_hidden_camera_wrap_on_frame_13188 -- --nocapture`
- `cargo run -q -p kidcli -- tas-diff --scenario .cache/tas-imports/any-percent-published/scenario.json --frame-start 21820 --frame-end 21830 --include-advisory`
- `cargo run -q -p kidcli -- tas-diff --scenario .cache/tas-imports/any-percent-published/scenario.json --frame-start 21834 --frame-end 21842 --include-advisory`
- `cargo run -q -p kidcli -- verify-tas --scenario .cache/tas-imports/any-percent-published/scenario.json --frame-start 21765 --bisect`
- `mise run verify-slice-fast`

## Dead Ends / Do Not Retry

- Do not reopen the old cold-boot demo-1 wrap. `demo_1_bootstrap_keeps_camera_hidden_through_frame_2537` still expects `0xFF000000` on the cold-boot path.
- Do not reuse the Start-skip intro-return wrap. That path is already covered by `published_tas_intro_return_demo_1_uses_the_oracle_hidden_camera_wrap_on_frame_13188` and still expects `0xFF200000`.
- Do not patch another shared hidden-camera magic constant. The next blocker is no longer the hidden wrap; it is the first visible demo-bootstrap pose/camera release at frame 21837.
