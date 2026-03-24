---
slug: any-percent-published-frame-16228-intro-sequence-5-kid-y-high
cycle: 59
updated_at: 2026-03-24T05:02:01Z
---

# Source-Port Frontier Handoff

## What Changed

- Fixed the frame-13188 hidden demo-bootstrap camera frontier.
- The bug was not the shared demo-1 cold-boot constant. A new regression proved only the Start-skip intro-return title path should expose the wrapped hidden row on demo frame 60.
- Added Start-skip intro-return latches in `NativeRuntime` so `Obj_TitleMenu loc_1BB02 -> Load_DemoPlay -> Load_InGame` can expose the oracle-backed `camera_y_pos=0xff200000` without corrupting the cold-boot Knights Isle bootstrap row.
- Added `published_tas_intro_return_demo_1_uses_the_oracle_hidden_camera_wrap_on_frame_13188`.

## Current Model

- The frame-13188 frontier is now authoritative-clean; `tas-diff --frame-start 13184 --frame-end 13192 --include-advisory` shows only the pre-existing advisory `number_objects` drift on frames 13187..13191.
- The next first authoritative mismatch is frame `16228`.
- Current evidence says frame 16228 is a one-frame `kid_y_pos` mismatch on `mode=0x0024`, `level=0x00`:
  - Native frame 16228: `kid_y_pos=0x00000000`
  - Oracle frame 16228: `kid_y_pos=0x00e00000`
- Frames 16224..16227 are clean apart from advisory `number_objects` drift on 16227, and frames 16229..16231 rejoin immediately on authoritative state.
- `mode=0x0024` strongly suggests intro sequence 5, but that remains an inference until the exact ASM write path is re-read.

## Strongest Evidence

- ASM
  - `.cache/upstream/kid-chameleon-disasm/kid.asm`
    - `sub_1B8F4` / `loc_1B920` poll `sub_1C246` on the intro-sequence-1 Start-skip path.
    - `sub_1C246` returns through the title shell instead of the cold-boot demo bootstrap.
    - `Obj_TitleMenu loc_1BB02` flips to `Game_Mode=$10`, then `Load_DemoPlay` / `Load_InGame` own the hidden demo bootstrap rows.
- Native
  - `crates/kid-core/src/boot.rs`
    - Start-skip intro returns now latch a dedicated next-demo hidden-camera wrap instead of mutating the shared cold-boot path.
  - `crates/kid-core/src/trace.rs`
    - `current_camera_state()` exposes `0xff200000` only for demo-1 frame 60 when the Start-skip intro-return latch is active.
  - `crates/kid-core/src/tests.rs`
    - `published_tas_intro_return_demo_1_uses_the_oracle_hidden_camera_wrap_on_frame_13188` locks the fixed path down.
- Verification
  - `cargo test -q -p kid-core published_tas_intro_return_demo_1_uses_the_oracle_hidden_camera_wrap_on_frame_13188 -- --nocapture`
  - `cargo test -q -p kid-core demo_1_bootstrap_keeps_camera_hidden_through_frame_2537 -- --nocapture`
  - `cargo test -q -p kid-core published_tas_intro_sequence_1_start_skip_keeps_kid_hidden_on_frame_12430 -- --nocapture`
  - `cargo run -q -p kidcli -- tas-diff --scenario .cache/tas-imports/any-percent-published/scenario.json --frame-start 13184 --frame-end 13192 --include-advisory`
    - frame 13188 no longer diverges authoritatively
  - `cargo run -q -p kidcli -- verify-tas --scenario .cache/tas-imports/any-percent-published/scenario.json --frame-start 13189 --bisect`
    - first CRC divergence moved to frame `16228`
  - `cargo run -q -p kidcli -- tas-diff --scenario .cache/tas-imports/any-percent-published/scenario.json --frame-start 16224 --frame-end 16232 --include-advisory`
    - frame 16228 diverges on `kid_y_pos` only (`native=0x00000000`, `oracle=0x00e00000`)
  - `mise run verify-slice-fast`

## Exact Next Moves

1. Read the `Game_Mode=$24` ASM path around frame 16228 before touching native code. Confirm whether this is indeed intro sequence 5 and identify the exact write that keeps oracle `kid_y_pos=0x00e00000` for one more frame.
2. Explain why native blanks `kid_y_pos` to zero on frame 16228 while oracle still exposes `0x00e00000`.
3. Add a focused regression for frame 16228 once the write path is proven, then fix the pose/visibility hand-off.

## Verification To Re-run

- `cargo test -q -p kid-core published_tas_intro_return_demo_1_uses_the_oracle_hidden_camera_wrap_on_frame_13188 -- --nocapture`
- `cargo test -q -p kid-core demo_1_bootstrap_keeps_camera_hidden_through_frame_2537 -- --nocapture`
- `cargo test -q -p kid-core published_tas_intro_sequence_1_start_skip_keeps_kid_hidden_on_frame_12430 -- --nocapture`
- `cargo run -q -p kidcli -- tas-diff --scenario .cache/tas-imports/any-percent-published/scenario.json --frame-start 13184 --frame-end 13192 --include-advisory`
- `cargo run -q -p kidcli -- verify-tas --scenario .cache/tas-imports/any-percent-published/scenario.json --frame-start 13189 --bisect`
- `cargo run -q -p kidcli -- tas-diff --scenario .cache/tas-imports/any-percent-published/scenario.json --frame-start 16224 --frame-end 16232 --include-advisory`
- `mise run verify-slice-fast`

## Dead Ends / Do Not Retry

- Do not change `demo::demo_1_hidden_camera_y(60)` globally. Cold-boot demo 1 still needs the existing wrapped `0xff000000` row, and `demo_1_bootstrap_keeps_camera_hidden_through_frame_2537` proves it.
- Do not reopen the cleared frame-13188 frontier as a generic demo bootstrap bug. The fix is specific to the Start-skip intro-return title path.
- Do not assume frame 16228 is another hidden camera issue. The current diff is `kid_y_pos`, not `camera_y_pos`, and it needs fresh ASM before any native patch.
