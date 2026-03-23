---
slug: any-percent-published-frame-8056-demo-1-camera-y-after-iron-knight-release
cycle: 36
updated_at: 2026-03-23T03:39:03Z
---

# Source-Port Frontier Handoff

## What Changed

- Confirmed from ROM `ReadJoypad` (`loc_C22`) that demo playback does not guard the demo-input pointer. `Demo_InputData1` falls straight into `byte_1E6E3`, so late demo 1 reads adjacent ROM bytes instead of clamping to zero.
- Ported that quirk into native with a small demo-1 overflow tail (`0x06, 0xC4, 0xC7, 0xFF, 0x9A, 0x14`) and reused it in both `step_demo_input()` and the transform-parity re-prime path.
- Added the missing Iron Knight opposite-direction wall-release branch: when the overrun byte supplies held left on frame `8056`, native now leaves `iron_knight_climb_resume`, flips facing, and hibernates the release frame instead of forcing another climb step.
- Added focused regressions for the demo-byte overrun and the `8056`/`8057` wall-release window.

## Current Model

- The old gameplay blocker is fixed. Kid now matches oracle through the wall-release handoff:
  - frame `8056`: native preserves `kid_y_pos=0x0186d680` and reports `motion=iron_knight_climb_exit_hibernate`
  - frame `8057`: native matches oracle `kid_x_pos=0x03a7d500`, `kid_y_pos=0x01859680`
- The live frontier is still frame `8056`, but it has moved from Kid gameplay to camera state.
- Focused `tas-diff` now shows the only authoritative mismatch in `8054..8059` is `camera_y_pos`:
  - frame `8056`: native `0x00f10000`, oracle `0x00f00000`
  - frame `8057`: native `0x00f00000`, oracle `0x00ef0000`
  - frame `8058`: native `0x00ef0000`, oracle `0x00ee0000`
- Advisory velocity/object-count drift remains, but Kid pose/control flow is no longer the causal blocker in this slice.

## Strongest Evidence

- `cargo run -p kidcli -- oracle-probe --scenario .cache/tas-imports/any-percent-published/scenario.json --frame-start 8055 --frame-end 8058 --watches 0xFFFFF813:u8,0xFFFFF815:u8,0xFFFFF817:u8,0xFFFFF818:u8,0xFFFFF819:u8`
  - oracle rows prove the late demo window is not idle:
  - frame `8055`: pressed `0x06`, `Ctrl_C_Held=FF`, `Ctrl_Right_Held=FF`
  - frame `8056`: first overflow byte is live in held-state form: `Ctrl_Left_Held=FF`, `Ctrl_Down_Held=FF`
- `.cache/upstream/kid-chameleon-disasm/kid.asm`
  - `ReadJoypad` `loc_C22` increments `(Addr_Current_Demo_Keypress)` with no bounds check.
  - `Demo_InputData1` is followed immediately by `byte_1E6E3` (`scenes/artcomp/Dark_alley_from_intro.bin`), whose first bytes are `06 C4 C7 FF 9A 14`.
  - `loc_A2E2`/`loc_A2F6` exit Iron Knight wall climb on opposite-direction hold and feed `loc_A6F8`, which hibernates the release frame.
- `cargo run -p kidcli -- tas-diff --scenario .cache/tas-imports/any-percent-published/scenario.json --frame-start 8054 --frame-end 8059 --include-physics-trace --include-advisory`
  - frame `8056` now matches oracle Kid pose and reports `motion=iron_knight_climb_exit_hibernate`
  - frame `8057` now matches oracle Kid `x/y`
  - remaining authoritative diff is `camera_y_pos`
- `cargo run -p kidcli -- verify-tas --scenario .cache/tas-imports/any-percent-published/scenario.json --frame-start 6975 --frame-end 9530 --bisect`
  - `RESULT: first CRC divergence at frame 8056`
  - native summary already shows the repaired Kid pose: `kid=(0x03a80000,0x0186d680)`

## Exact Next Moves

1. Read the camera-follow ROM path that runs on the `loc_A6F8` release frame and the first post-release jump frame. Do not reopen Iron Knight climb maths; Kid state is now right.
2. Compare native camera update timing against oracle for frames `8056..8058`. The pattern is a clean one-frame-high camera carry, so look for a preserved y-look / follow offset that should have dropped with the climb exit.
3. Patch camera only, then rerun the focused `8054..8059` diff, the `6975..9530` bisect, and `mise run verify-slice-fast`.

## Verification To Re-run

- `cargo test -p kid-core demo_1_timeout_overrun_uses_the_adjacent_rom_bytes -- --nocapture`
- `cargo test -p kid-core published_tas_frame_8056_hibernates_the_iron_knight_wall_release -- --nocapture`
- `cargo test -p kid-core published_tas_frame_8057_matches_the_oracle_wall_release_carry -- --nocapture`
- `cargo test -p kid-core published_tas_frame_8004_keeps_demo_1_in_live_climb_gameplay -- --nocapture`
- `cargo test -p kid-core published_tas_demo_1_times_out_on_frame_8069 -- --nocapture`
- `cargo run -p kidcli -- tas-diff --scenario .cache/tas-imports/any-percent-published/scenario.json --frame-start 8054 --frame-end 8059 --include-physics-trace --include-advisory`
- `cargo run -p kidcli -- verify-tas --scenario .cache/tas-imports/any-percent-published/scenario.json --frame-start 6975 --frame-end 9530 --bisect`
- `mise run verify-slice-fast`

## Dead Ends / Do Not Retry

- Do not clamp demo-1 input past `DEMO_INPUT_DATA_1` to zero. The ROM overreads into `byte_1E6E3`, and zero-clamping recreates the bad `held=none` branch choice.
- Do not reopen the old `iron_knight_climb_resume` constant tuning. The climb exit was a branch/input issue, not an accel issue.
- Do not trust the imported scenario bytes around `8055..8058` for demo gameplay input. Oracle controller RAM is the source of truth here.
- Do not keep working the Kid pose on this frontier. Kid state now matches through the release carry; the remaining blocker is camera timing.
