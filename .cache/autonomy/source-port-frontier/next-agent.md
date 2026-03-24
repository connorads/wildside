---
slug: any-percent-published-frame-11549-demo-2-cyclone-flight-y-high
cycle: 53
updated_at: 2026-03-24T03:39:11Z
---

# Source-Port Frontier Handoff

## What Changed

- Re-read the timeout-restart title-to-demo ASM around `Obj_TitleMenu` / `loc_1BB02` and `Load_InGame` to confirm the live blocker was in the hidden demo bootstrap, not in the reused title wait.
- Added `published_tas_second_timeout_restart_uses_oracle_demo_camera_preload_on_frame_11043` to lock the 11042/11043/11044 camera settle window.
- Fixed the demo-2 hidden preload camera row so frame 11043 now uses the oracle-backed `0x02200000` row before the `Load_InGame` hold resumes on 11044.
- Re-ran the timeout-restart regressions, both oracle slices, the bisect from frame 9531, and `mise run verify-slice-fast`.
- The first authoritative mismatch moved from frame 11043 to frame 11549.

## Current Model

- The frame-11043 blocker was a one-frame hidden demo bootstrap camera preload mismatch, not another title-shell timing bug.
- `Obj_TitleMenu`'s `loc_1BB02` hand-off is now clean through the hidden preload and respawn hold: 11038..11048 fully match oracle.
- The next live blocker is frame 11549 in Skydragon Castle demo gameplay. Native keeps Cyclone one pixel higher on the first authoritative break:
  - native `camera_y_pos=0x00730000`, oracle `0x00720000`
  - native `kid_y_pos=0x010F0000`, oracle `0x010E9A00`
- Native probes already show `helmet=0x0002`, `cyclone_flying=1`, and `custom_gravity=1`, so the next likely cause is Cyclone's flying Y-acceleration / post-flight vertical hand-off rather than title/demo bootstrap state.
- `number_objects` drift in this window is still advisory and comes from prize-diamond / floating-item state; do not chase that before the Cyclone Y path.

## Strongest Evidence

- ASM
  - `.cache/upstream/kid-chameleon-disasm/kid.asm`
    - `Obj_TitleMenu` / `loc_1BB02`: `move.w #$10,(Game_Mode).w`
    - `Load_InGame`: respawn camera snap initialises the normal demo hold on the following frame
    - `loc_B214`..`loc_B26C`: Cyclone flying path updates `Cyclone_YAcceleration` and clamps `y_vel(a3)`
- Native/oracle diagnostics
  - `cargo test -q -p kid-core published_tas_second_timeout_restart_ -- --nocapture`
    - 12 tests pass
  - `cargo run -q -p kidcli -- tas-diff --scenario .cache/tas-imports/any-percent-published/scenario.json --frame-start 10982 --frame-end 10994 --include-advisory`
    - frames `10982..10993`: `MATCH`
  - `cargo run -q -p kidcli -- tas-diff --scenario .cache/tas-imports/any-percent-published/scenario.json --frame-start 11038 --frame-end 11048 --include-advisory`
    - frames `11038..11047`: `MATCH`
  - `cargo run -q -p kidcli -- verify-tas --scenario .cache/tas-imports/any-percent-published/scenario.json --frame-start 9531 --bisect`
    - `RESULT: first CRC divergence at frame 11549`
  - `cargo run -q -p kidcli -- tas-diff --scenario .cache/tas-imports/any-percent-published/scenario.json --frame-start 11548 --frame-end 11551 --include-advisory`
    - frame `11548`: advisory-only `number_objects`
    - frame `11549`: first authoritative break on `camera_y_pos` (`0x00730000` vs `0x00720000`) and `kid_y_pos` (`0x010F0000` vs `0x010E9A00`)
    - native gameplay probes on `11549` show `cyclone_flying=1`, `custom_gravity=1`, `Cyclone_YAcceleration=-26112`
  - `mise run verify-slice-fast`
    - passes

## Exact Next Moves

1. Read the Cyclone flying ASM first: `.cache/upstream/kid-chameleon-disasm/kid.asm` around `loc_B214`..`loc_B26C`, then read the surrounding jump/custom-gravity path that feeds it.
2. Add a focused regression for the 11548/11549/11550 window so the first authoritative Cyclone Y mismatch is explicit in `crates/kid-core/src/tests.rs`.
3. Fix the Cyclone demo-flight vertical hand-off so frame 11549 matches oracle, then re-run the bisect from frame 9531.

## Verification To Re-run

- `cargo test -q -p kid-core published_tas_second_timeout_restart_ -- --nocapture`
- `cargo run -q -p kidcli -- tas-diff --scenario .cache/tas-imports/any-percent-published/scenario.json --frame-start 10982 --frame-end 10994 --include-advisory`
- `cargo run -q -p kidcli -- tas-diff --scenario .cache/tas-imports/any-percent-published/scenario.json --frame-start 11038 --frame-end 11048 --include-advisory`
- `cargo run -q -p kidcli -- tas-diff --scenario .cache/tas-imports/any-percent-published/scenario.json --frame-start 11548 --frame-end 11551 --include-advisory`
- `cargo run -q -p kidcli -- verify-tas --scenario .cache/tas-imports/any-percent-published/scenario.json --frame-start 9531 --bisect`
- `mise run verify-slice-fast`

## Dead Ends / Do Not Retry

- Do not reopen the timeout-restart title-arm/timer theory for this frontier. The 10982..10994 and 11038..11048 slices are now clean.
- Do not move the preload frame itself earlier or later. Frame 11042 already matches; only the hidden preload row value on 11043 was wrong.
- Do not chase `number_objects` at frame 11549 before fixing the Cyclone Y path. That object-count drift is advisory and already starts one frame earlier.
