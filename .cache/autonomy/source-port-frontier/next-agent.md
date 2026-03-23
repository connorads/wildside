---
slug: any-percent-published-frame-8069-demo-timeout-handoff-preserves-the-wrong-snapshot
cycle: 37
updated_at: 2026-03-23T03:47:34Z
---

# Source-Port Frontier Handoff

## What Changed

- Re-read ROM `loc_A6F8` and `sub_71E4`: the Iron Knight opposite-direction wall release hibernates the frame, but it still runs camera follow before `Hibernate_Object_1Frame`.
- Ported that release-frame camera follow into the native `IronKnightClimbStep::ExitHibernate` branch.
- Added a focused regression for the oracle-backed wall-release camera carry on frames `8056..8058`.
- Re-ran the focused wall-release tests, `tas-diff 8054..8059`, `verify-tas --frame-start 6975 --frame-end 9530 --bisect`, and `mise run verify-slice-fast`.

## Current Model

- The `8056..8058` camera-only blocker is fixed.
- Focused `tas-diff --frame-start 8054 --frame-end 8059 --include-physics-trace --include-advisory` now has no authoritative mismatches; only advisory object-count/velocity drift remains in that window.
- The first CRC divergence for the published TAS slice has moved to frame `8069`.
- This is now a demo-timeout / mode handoff frontier, not a wall-release gameplay frontier.

## Strongest Evidence

- `.cache/upstream/kid-chameleon-disasm/kid.asm`
  - `loc_A6F8` clears climb/platform state, calls `sub_71E4`, then hibernates the frame.
  - `sub_71E4` updates both `Camera_X_pos` and `Camera_Y_pos` from the current Kid pose before returning.
- `cargo run -p kidcli -- tas-diff --scenario .cache/tas-imports/any-percent-published/scenario.json --frame-start 8054 --frame-end 8059 --include-physics-trace --include-advisory`
  - frame `8056`: native now matches oracle `camera_y_pos=0x00f00000`
  - frame `8057`: native now matches oracle `camera_y_pos=0x00ef0000`
  - frame `8058`: native now matches oracle `camera_y_pos=0x00ee0000`
- `cargo run -p kidcli -- verify-tas --scenario .cache/tas-imports/any-percent-published/scenario.json --frame-start 6975 --frame-end 9530 --bisect`
  - `RESULT: first CRC divergence at frame 8069`
  - native summary: `mode=0x0000 level=00 kid=(0x0399d280,0x0186b680)`
- `cargo run -p kidcli -- tas-diff --scenario .cache/tas-imports/any-percent-published/scenario.json --frame-start 8068 --frame-end 8071 --include-advisory`
  - frame `8068` is advisory-only
  - frame `8069` authoritative diffs:
    - `camera_x_pos`: native `0x03000000`, oracle `0x02ff0000`
    - `kid_x_pos`: native `0x0399d280`, oracle `0x03982c80`
    - `kid_y_pos`: native `0x0186b680`, oracle `0x01883680`
  - native has only 4 objects while oracle still has 9 and keeps the Kid gfx object live

## Exact Next Moves

1. Read the ROM timeout handoff path that triggers when demo 1 times out on frame `8069`. Do not reopen the wall-release camera code; that slice is now authoritative-clean.
2. Compare the ROM handoff’s preserved gameplay snapshot against native on `8069..8070`, especially why native jumps to `camera_x_pos=0x03000000` / `kid=(0x0399d280,0x0186b680)` while oracle preserves `camera_x_pos=0x02ff0000` / `kid=(0x03982c80,0x01883680)` and keeps the extra objects alive.
3. Patch only the timeout handoff, then rerun the focused `8068..8071` diff, the `6975..9530` bisect, and `mise run verify-slice-fast`.

## Verification To Re-run

- `cargo test -p kid-core published_tas_frame_8056_hibernates_the_iron_knight_wall_release -- --nocapture`
- `cargo test -p kid-core published_tas_frame_8057_matches_the_oracle_wall_release_carry -- --nocapture`
- `cargo test -p kid-core published_tas_frames_8056_to_8058_match_the_oracle_wall_release_camera_carry -- --nocapture`
- `cargo test -p kid-core demo_1_timeout_overrun_uses_the_adjacent_rom_bytes -- --nocapture`
- `cargo test -p kid-core published_tas_frame_8004_keeps_demo_1_in_live_climb_gameplay -- --nocapture`
- `cargo test -p kid-core published_tas_demo_1_times_out_on_frame_8069 -- --nocapture`
- `cargo run -p kidcli -- tas-diff --scenario .cache/tas-imports/any-percent-published/scenario.json --frame-start 8054 --frame-end 8059 --include-physics-trace --include-advisory`
- `cargo run -p kidcli -- tas-diff --scenario .cache/tas-imports/any-percent-published/scenario.json --frame-start 8068 --frame-end 8071 --include-advisory`
- `cargo run -p kidcli -- verify-tas --scenario .cache/tas-imports/any-percent-published/scenario.json --frame-start 6975 --frame-end 9530 --bisect`
- `mise run verify-slice-fast`

## Dead Ends / Do Not Retry

- Do not reopen the old wall-release branch/input work. `8056..8058` is no longer the blocker.
- Do not chase `y_look_offset` / `$FFFFFB58` for this frontier. Oracle watch rows keep it at `0` through `8054..8058`; that was not the cause of the camera carry.
- Do not remove the release-frame camera follow from the `ExitHibernate` branch. ROM `loc_A6F8` calls `sub_71E4` before hibernation.
- Do not treat the `8069` object-count drift as purely advisory. Native dropping to 4 objects is part of the wrong timeout snapshot/handoff state, not harmless noise.
