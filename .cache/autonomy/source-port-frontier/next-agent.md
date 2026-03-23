---
slug: any-percent-published-frame-9097-intro-sequence-3-pose-still-uses-the-old-title-coordinates
cycle: 38
updated_at: 2026-03-23T04:07:25Z
---

# Source-Port Frontier Handoff

## What Changed

- Re-read the ROM demo-timeout handoff in `ReadJoypad`: once `cmpi.w #$3F2,d7` fails, the ROM skips `loc_C22`, writes `Game_Mode=0`, clears demo mode, resets `Current_LevelID`, and jumps through `loc_6E2`.
- Fixed native demo 1 timeout ordering so the first visible mode-0 frame replays the already-buffered gameplay step before the timeout handoff becomes visible.
- Kept the preserved demo snapshot inert during the title rebuild so the timeout handoff no longer clears the carried gameplay roster/counts on `8069..8070`.
- Added a focused regression for the oracle-backed timeout snapshot on frames `8069..8070`.
- Updated the stale generic demo 1 timeout unit test to use the oracle-backed `DEMO_1_TIMEOUT_COUNTER_OFFSET_FRAMES`.
- Re-ran the focused demo-timeout tests, `tas-diff 8068..8071`, `verify-tas --frame-start 6975 --frame-end 9530 --bisect`, and `mise run verify-slice-fast`.

## Current Model

- The `8069..8070` demo-timeout snapshot blocker is fixed.
- Native now matches the oracle on the first visible mode-0 handoff frames, including the carried Kid pose, camera, and preserved object count.
- The first CRC divergence for the published TAS slice has moved to frame `9097`.
- This is now an intro-sequence-3/title-pose frontier: native still uses the old `intro_sequence_3_pose()` coordinates `kid=(0x00600000,0x00d00000)` while the oracle wants `kid=(0x00700000,0x00cc0000)` as `game_mode=0x001c` becomes visible, then the oracle starts exposing the matching title glyph objects on `9098`.

## Strongest Evidence

- `.cache/upstream/kid-chameleon-disasm/kid.asm`
  - `ReadJoypad` at `loc_B98` / `loc_C22`: once the timeout compare hits `$3F2`, the ROM skips demo-byte fetch, writes `Game_Mode=0`, clears demo mode, resets `Current_LevelID`, and jumps through `loc_6E2`.
  - `Load_IntroSequence3`, `sub_1B93E`, and `loc_1B96A`: this is the ROM path for the visible `game_mode=0x001c` intro-sequence-3 pose.
- `cargo run -p kidcli -- tas-diff --scenario .cache/tas-imports/any-percent-published/scenario.json --frame-start 8068 --frame-end 8071 --include-advisory`
  - frame `8069`: `MATCH`
  - frame `8070`: `MATCH`
- `cargo run -p kidcli -- verify-tas --scenario .cache/tas-imports/any-percent-published/scenario.json --frame-start 6975 --frame-end 9530 --bisect`
  - `RESULT: first CRC divergence at frame 9097`
  - native summary: `mode=0x001c level=00 kid=(0x00600000,0x00d00000)`
- `cargo run -p kidcli -- tas-diff --scenario .cache/tas-imports/any-percent-published/scenario.json --frame-start 9094 --frame-end 9100 --include-advisory`
  - frames `9094..9096`: `MATCH`
  - frame `9097` authoritative diffs:
    - `kid_x_pos`: native `0x00600000`, oracle `0x00700000`
    - `kid_y_pos`: native `0x00d00000`, oracle `0x00cc0000`
  - frame `9098` keeps the same pose mismatch and oracle starts exposing the matching title glyph roster (`0x30` objects).

## Exact Next Moves

1. Read the ROM intro-sequence-3/title path: `Load_IntroSequence3`, `sub_1B93E`, `loc_1B96A`, plus the helper that seeds the visible Kid/title glyph positions. Do not reopen the demo-timeout handoff; `8069..8070` is clean now.
2. Compare that ROM pose/loader path against native `PlayerState::intro_sequence_3_pose()` and the intro/title trace path to explain why native still uses `0x00600000,0x00d00000` while oracle wants `0x00700000,0x00cc0000` on `9097`.
3. Patch only the intro-sequence-3/title pose path, then rerun the focused `9094..9100` diff, the `6975..9530` bisect, and `mise run verify-slice-fast`.

## Verification To Re-run

- `cargo test -p kid-core demo_auto_ends_after_max_frames -- --nocapture`
- `cargo test -p kid-core demo_2_auto_end_waits_for_the_oracle_backed_timeout_counter -- --nocapture`
- `cargo test -p kid-core demo_1_timeout_overrun_uses_the_adjacent_rom_bytes -- --nocapture`
- `cargo test -p kid-core published_tas_frame_8004_keeps_demo_1_in_live_climb_gameplay -- --nocapture`
- `cargo test -p kid-core published_tas_demo_1 -- --nocapture`
- `cargo run -p kidcli -- tas-diff --scenario .cache/tas-imports/any-percent-published/scenario.json --frame-start 8068 --frame-end 8071 --include-advisory`
- `cargo run -p kidcli -- tas-diff --scenario .cache/tas-imports/any-percent-published/scenario.json --frame-start 9094 --frame-end 9100 --include-advisory`
- `cargo run -p kidcli -- verify-tas --scenario .cache/tas-imports/any-percent-published/scenario.json --frame-start 6975 --frame-end 9530 --bisect`
- `mise run verify-slice-fast`

## Dead Ends / Do Not Retry

- Do not reopen the old wall-release camera carry or the `8069` timeout snapshot ordering/count work. That frontier is clean.
- Do not “fix” the new `9097` blocker by changing the demo-timeout carry or the demo 2 timeout-restart path. The new divergence happens later in intro sequence 3.
- Do not keep the old hard-coded intro-sequence-3 pose unquestioned. The oracle now proves that the visible `0x001c` pose on this frontier is not `0x00600000,0x00d00000`.
