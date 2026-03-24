---
slug: any-percent-published-frame-11549-demo-2-cyclone-flight-y-high
cycle: 54
updated_at: 2026-03-24T04:05:00Z
---

# Source-Port Frontier Handoff

## What Changed

- Re-read the Cyclone flight ASM in `.cache/upstream/kid-chameleon-disasm/kid.asm` around `sub_B168` `loc_B214..loc_B26C`.
- Re-ran `tas-diff` for frames `11548..11550` to confirm the first authoritative break is still `camera_y_pos=0x00730000 -> 0x00720000` and `kid_y_pos=0x010F0000 -> 0x010E9A00` on frame `11549`.
- Added ignored regression `published_tas_second_timeout_restart_keeps_the_oracle_backed_cyclone_flight_arc_on_frame_11549` in `crates/kid-core/src/tests.rs` so the live 11548/11549/11550 Cyclone window is explicit in the test suite without locking in the current bug.
- Verified the focused timeout-restart test slice and `mise run verify-slice-fast`; no runtime behaviour changed this cycle.
- Noted a context hazard: `.cache/autonomy/source-port-frontier/next-task.json` is stale and still points at the older frame-7969 Knights Isle frontier. Follow this note, not that JSON, until the task package is regenerated.

## Current Model

- The live blocker is still frame 11549 in Skydragon Castle demo gameplay. Native keeps Cyclone one pixel higher on the first authoritative break:
  - native `camera_y_pos=0x00730000`, oracle `0x00720000`
  - native `kid_y_pos=0x010F0000`, oracle `0x010E9A00`
- `sub_B168` `loc_B214..loc_B26C` is the relevant ROM path once `Cyclone_flying` is set: it increments the 16-bit `Cyclone_YAcceleration` by `$400`, optionally subtracts `$3000` on a consumed C-edge, stores the wrapped word back, sign-extends it into `d7`, adds it to `y_vel(a3)`, then clamps velocity to `[-$30000, $28000]`.
- Native probes already show `helmet=0x0002`, `cyclone_flying=1`, and `custom_gravity=1`, so the next likely cause is still Cyclone's flying Y-acceleration / post-flight vertical hand-off rather than title/demo bootstrap state.
- `number_objects` drift in this window is still advisory and comes from prize-diamond / floating-item state; do not chase that before the Cyclone Y path.

## Strongest Evidence

- ASM
  - `.cache/upstream/kid-chameleon-disasm/kid.asm`
    - `sub_B168` `loc_B214`..`loc_B26C`: Cyclone flying path updates `Cyclone_YAcceleration`, integrates it into `y_vel(a3)`, then clamps to `#-$30000 .. #$28000`
- Native/oracle diagnostics
  - `cargo run -q -p kidcli -- tas-diff --scenario .cache/tas-imports/any-percent-published/scenario.json --frame-start 10982 --frame-end 10994 --include-advisory`
    - frames `10982..10993`: `MATCH`
  - `cargo run -q -p kidcli -- tas-diff --scenario .cache/tas-imports/any-percent-published/scenario.json --frame-start 11038 --frame-end 11048 --include-advisory`
    - frames `11038..11047`: `MATCH`
  - `cargo run -q -p kidcli -- tas-diff --scenario .cache/tas-imports/any-percent-published/scenario.json --frame-start 11548 --frame-end 11551 --include-advisory`
    - frame `11548`: advisory-only `number_objects`
    - frame `11549`: first authoritative break on `camera_y_pos` (`0x00730000` vs `0x00720000`) and `kid_y_pos` (`0x010F0000` vs `0x010E9A00`)
    - frame `11550`: drift continues to `camera_y_pos` (`0x00720000` vs `0x00710000`) and `kid_y_pos` (`0x010E9E00` vs `0x010DD200`)
    - native gameplay probes on `11548..11550` show `cyclone_flying=1`, `custom_gravity=1`, and `Cyclone_YAcceleration=-27136 -> -26112 -> -25088`
  - `cargo test -q -p kid-core published_tas_second_timeout_restart_ -- --nocapture`
    - passes; the new frame-11549 oracle parity test is ignored as intended until the fix lands
  - `mise run verify-slice-fast`
    - passes

## Exact Next Moves

1. Fix the native Cyclone flight vertical hand-off so `published_tas_second_timeout_restart_keeps_the_oracle_backed_cyclone_flight_arc_on_frame_11549` can be unignored and pass. Start in `crates/kid-core/src/helmets.rs` and the surrounding post-physics path that feeds `step_cyclone_flight()`.
2. Re-read the surrounding player path that runs after `sub_B168` on the same frame to find where native freezes at `kid_y_pos=0x010F0000` even though `y_vel` has already advanced to `0xFFFF9A00`.
3. Once the fix lands, unignore the new frame-11549 regression, rerun the 11548..11550 `tas-diff`, then rerun the bisect from frame `9531`.

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
- Do not follow `.cache/autonomy/source-port-frontier/next-task.json` for this session. It is stale and still points at the old frame-7969 frontier.
