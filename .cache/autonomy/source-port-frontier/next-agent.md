---
slug: any-percent-published-frame-9312-native-enters-title-before-oracle-leaves-intro-sequence-3
cycle: 39
updated_at: 2026-03-23T04:19:08Z
---

# Source-Port Frontier Handoff

## What Changed

- Re-read the ROM intro-sequence-3 path in `Load_IntroSequence3`, `sub_1B93E`, `loc_1B96A`, and the `unk_1AF24` / `unk_1AF78` descriptor tables.
- Replaced the single hard-coded intro-sequence-3 pose with a ROM-derived roster selector: native now mirrors the rotating `$FFFFFC83` roster cycle and exposes the third descriptor from the active `unk_1AF78` roster, which is what oracle trace slot 2 (`kid_gfx`) shows.
- Added `published_tas_demo_1_uses_the_second_intro_sequence_3_roster_pose_on_frame_9097`.
- Kept the earlier intro-sequence-3/title regressions green, reran `tas-diff 9094..9100`, reran the `6975..9530` bisect, and reran `mise run verify-slice-fast`.

## Current Model

- The frame-9097 blocker is fixed. Native now matches the oracle's second intro-sequence-3 roster pose at `kid=(0x00700000,0x00CC0000)`.
- The first CRC divergence has moved to frame `9312`.
- This is now an intro-sequence-3 exit-timing frontier: native reaches `game_mode=0x002C` (title) on frame 9312 while oracle is still in `game_mode=0x001C` (intro sequence 3) with the same carried second-roster pose.

## Strongest Evidence

- `.cache/upstream/kid-chameleon-disasm/kid.asm`
  - `Load_IntroSequence3` increments `$FFFFFC83`, masks it through `unk_1AF78`, and allocates the selected gfx descriptors from `unk_1AF24`.
  - `sub_1B93E` / `loc_1B96A` are still the live intro-sequence-3 object coroutine.
  - `sub_1C246` / `loc_1C250` are the ROM's intro skip/title entry helper that writes `Game_Mode=0x002C`.
- `cargo test -p kid-core published_tas_demo_2_exposes_the_first_intro_sequence_3_kid_pose_on_frame_5994 -- --nocapture`
  - still green with the first roster pose.
- `cargo test -p kid-core published_tas_demo_2_carries_intro_sequence_3_pose_into_the_first_title_frames -- --nocapture`
  - still green with the title carry-over path.
- `cargo test -p kid-core published_tas_demo_1_uses_the_second_intro_sequence_3_roster_pose_on_frame_9097 -- --nocapture`
  - green; native now exposes the second roster pose on the correct frame.
- `cargo run -p kidcli -- tas-diff --scenario .cache/tas-imports/any-percent-published/scenario.json --frame-start 9094 --frame-end 9100 --include-advisory`
  - frames `9094..9096`: `MATCH`
  - frames `9097..9099`: advisory-only `number_objects` drift; no authoritative pose mismatch remains.
- `cargo run -p kidcli -- verify-tas --scenario .cache/tas-imports/any-percent-published/scenario.json --frame-start 6975 --frame-end 9530 --bisect`
  - `RESULT: first CRC divergence at frame 9312`
  - native summary: `mode=0x002c level=00 kid=(0x00700000,0x00cc0000)`
- `cargo run -p kidcli -- tas-diff --scenario .cache/tas-imports/any-percent-published/scenario.json --frame-start 9308 --frame-end 9316 --include-advisory`
  - frames `9308..9311`: advisory-only `number_objects` drift
  - frame `9312`: authoritative `game_mode` diff only
    - native `0x002c`
    - oracle `0x001c`
  - oracle still shows the second-roster intro-sequence-3 object roster through at least `9315`.

## Exact Next Moves

1. Read the ROM exit path again before touching native: `sub_1B93E`, `loc_1B96A`, and `sub_1C246` / `loc_1C250`. The new blocker is timing, not pose selection.
2. Compare that ROM timing against native `step_boot_state()` around segment 2 only, especially `elapsed`, `pending_title_skip`, and any timeout-restart intro-sequence-3 return flags that could force `TitleScreen` early.
3. Patch only the intro-sequence-3 -> title timing for this later attract-loop visit, then rerun the focused `9308..9316` diff, the `6975..9530` bisect, and `mise run verify-slice-fast`.

## Verification To Re-run

- `cargo test -p kid-core published_tas_demo_2_exposes_the_first_intro_sequence_3_kid_pose_on_frame_5994 -- --nocapture`
- `cargo test -p kid-core published_tas_demo_2_carries_intro_sequence_3_pose_into_the_first_title_frames -- --nocapture`
- `cargo test -p kid-core published_tas_demo_1_uses_the_second_intro_sequence_3_roster_pose_on_frame_9097 -- --nocapture`
- `cargo run -p kidcli -- tas-diff --scenario .cache/tas-imports/any-percent-published/scenario.json --frame-start 9094 --frame-end 9100 --include-advisory`
- `cargo run -p kidcli -- tas-diff --scenario .cache/tas-imports/any-percent-published/scenario.json --frame-start 9308 --frame-end 9316 --include-advisory`
- `cargo run -p kidcli -- verify-tas --scenario .cache/tas-imports/any-percent-published/scenario.json --frame-start 6975 --frame-end 9530 --bisect`
- `mise run verify-slice-fast`

## Dead Ends / Do Not Retry

- Do not reopen the frame-9097 pose as if the hard-coded `0x00600000,0x00D00000` value were still correct. The ROM roster tables and oracle both prove the later attract loop uses `0x00700000,0x00CC0000`.
- Do not go back to the old demo-timeout snapshot/camera handoff theories. Frames `8069..8070` and the later pose frontier are already clean.
- Do not follow `.cache/autonomy/source-port-frontier/next-task.json` for this cycle. It is stale and still points at the older frame-7969 gameplay frontier.
