---
slug: any-percent-published-frame-22700-demo-timeout-title-pose-return
cycle: 68
updated_at: 2026-03-27T03:14:12Z
---

# Source-Port Frontier Handoff

## What Changed

- Fixed the frame-21837 timeout-restart demo-1 bootstrap delay by pushing the effective demo gameplay start one frame later for the `IntroSequence6` lineage in [`boot.rs`](/home/connor/git/kc/crates/kid-core/src/boot.rs), [`lib.rs`](/home/connor/git/kc/crates/kid-core/src/lib.rs), and [`trace.rs`](/home/connor/git/kc/crates/kid-core/src/trace.rs).
- Added [`published_tas_intro_sequence_6_keeps_demo_1_bootstrap_hidden_on_frame_21837`](/home/connor/git/kc/crates/kid-core/src/tests.rs), locking the oracle-backed hidden frame (`camera_y_pos=0x00E00000`, Kid hidden) before the visible bootstrap starts on frame 21838.
- Re-ran the focused bootstrap regressions, `tas-diff --frame-start 21834 --frame-end 21842 --include-advisory`, `verify-tas --frame-start 21765 --bisect`, and `mise run verify-slice-fast`; the first authoritative divergence now moves to frame 22700.

## Current Model

- ASM `loc_1BA5C` enters `Load_IntroSequence6`, and the later `Obj_TitleMenu loc_1BB02 -> Load_DemoPlay -> Load_InGame` hand-off preserves a distinct hidden demo-1 wrap for the timeout-restart lineage that came from intro sequence 5.
- Native now matches the later one-frame bootstrap delay as well and stays authoritative-clean through frame 22699.
- The next blocker opens when demo 1 times out back to title mode: on frames 22700..22701 native already flips to `mode=0x002C` / `level=0x00`, but Kid stays hidden at `(0,0)` while oracle restores the standard title pose at `(0x002E0000,0x008C0000)`.

## Strongest Evidence

- ASM
  - `.cache/upstream/kid-chameleon-disasm/kid.asm`
    - `loc_1BA5C` flips into `Load_IntroSequence6`.
    - `loc_1BB02` flips `Game_Mode` to `$10`, then `Load_DemoPlay` immediately calls `Load_InGame`.
    - `Load_InGame` seeds `Camera_Y_pos` from `PlayerStart_Y_pos` and starts the normal level bootstrap.
    - The next investigation target is the demo timeout return back into `Game_Mode=$2C`, where oracle restores the title pose for two frames before the later title helper hides Kid again.
- Native
  - [`boot.rs`](/home/connor/git/kc/crates/kid-core/src/boot.rs)
    - `demo_gameplay_start_frame()` now delays only the timeout-restart `IntroSequence6` demo-1 bootstrap by one startup frame.
  - [`lib.rs`](/home/connor/git/kc/crates/kid-core/src/lib.rs)
    - The demo bootstrap motion branch selection now uses that delayed start frame, so the hidden warm-up and first visible pose stay aligned.
  - [`trace.rs`](/home/connor/git/kc/crates/kid-core/src/trace.rs)
    - Trace visibility, hidden camera selection, and bootstrap object counts now use the same delayed start frame for the `IntroSequence6` lineage.
  - [`tests.rs`](/home/connor/git/kc/crates/kid-core/src/tests.rs)
    - `published_tas_intro_sequence_6_keeps_demo_1_bootstrap_hidden_on_frame_21837` passes.
- Oracle / parity
  - `cargo run -q -p kidcli -- verify-tas --scenario .cache/tas-imports/any-percent-published/scenario.json --frame-start 21765 --bisect`
    - First CRC divergence is now frame 22700.
  - `cargo run -q -p kidcli -- tas-diff --scenario .cache/tas-imports/any-percent-published/scenario.json --frame-start 21834 --frame-end 21842 --include-advisory`
    - Frames 21834..21842 now differ only on advisory `number_objects`; the old frame-21837 pose/camera break is cleared.
  - `cargo run -q -p kidcli -- tas-diff --scenario .cache/tas-imports/any-percent-published/scenario.json --frame-start 22696 --frame-end 22704 --include-advisory`
    - Frames 22696..22699 and 22702..22703 match. Frames 22700..22701 are the new first authoritative break: `kid_x_pos native=0x00000000 oracle=0x002E0000`, `kid_y_pos native=0x00000000 oracle=0x008C0000`.

## Exact Next Moves

1. Read the demo-1 timeout return path back into `Game_Mode=$2C` and identify which ASM routine restores the standard title pose on frames 22700..22701 before Kid blanks again.
2. Inspect the native title-return path after demo timeout and prove why Kid remains hidden at `(0,0)` for two frames instead of exposing the standard title pose.
3. Add one focused regression for frames 22700..22701 only after the exact title-pose write path is proven.

## Verification To Re-run

- `cargo test -q -p kid-core published_tas_intro_sequence_6_enters_demo_play_on_frame_21765 -- --nocapture`
- `cargo test -q -p kid-core published_tas_intro_sequence_6_uses_demo_1_hidden_camera_wrap_on_frame_21825 -- --nocapture`
- `cargo test -q -p kid-core published_tas_intro_sequence_6_keeps_demo_1_bootstrap_hidden_on_frame_21837 -- --nocapture`
- `cargo test -q -p kid-core demo_1_bootstrap_keeps_camera_hidden_through_frame_2537 -- --nocapture`
- `cargo test -q -p kid-core published_tas_intro_return_demo_1_uses_the_oracle_hidden_camera_wrap_on_frame_13188 -- --nocapture`
- `cargo run -q -p kidcli -- tas-diff --scenario .cache/tas-imports/any-percent-published/scenario.json --frame-start 21834 --frame-end 21842 --include-advisory`
- `cargo run -q -p kidcli -- tas-diff --scenario .cache/tas-imports/any-percent-published/scenario.json --frame-start 22696 --frame-end 22704 --include-advisory`
- `cargo run -q -p kidcli -- verify-tas --scenario .cache/tas-imports/any-percent-published/scenario.json --frame-start 21765 --bisect`
- `mise run verify-slice-fast`

## Dead Ends / Do Not Retry

- Do not reopen the old cold-boot demo-1 wrap. `demo_1_bootstrap_keeps_camera_hidden_through_frame_2537` still expects `0xFF000000` on the cold-boot path.
- Do not reuse the Start-skip intro-return wrap. That path is already covered by `published_tas_intro_return_demo_1_uses_the_oracle_hidden_camera_wrap_on_frame_13188` and still expects `0xFF200000`.
- Do not revert to a trace-only visibility mask. The frame-21837 fix needed the effective gameplay start delayed by one frame, not just a late hide in `trace.rs`.
