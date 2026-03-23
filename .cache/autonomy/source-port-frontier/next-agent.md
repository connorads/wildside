---
slug: any-percent-published-frame-9312-native-enters-title-before-oracle-leaves-intro-sequence-3
cycle: 40
updated_at: 2026-03-23T04:35:30Z
---

# Source-Port Frontier Handoff

## What Changed

- Re-read `sub_1B93E`, `loc_1B96A`, and `sub_1C246` / `loc_1C250` in the ROM, then compared them against the published TAS inputs around both intro-sequence-3 title-return windows.
- Added `published_tas_only_presses_start_on_the_first_timeout_restart_intro_sequence_3_visit`.
- Updated the native timeout-restart intro-sequence-3 comments to record the corrected model: frame 6209 is not a proven natural timeout.

## Current Model

- The old “timeout-restart intro sequence 3 naturally returns to title on frame 6209” model is wrong.
- The first timeout-restart visit reaches title because the published TAS presses Start on frame 6207; `sub_1C246` is the ROM path that jumps straight to title from intro sequence 3.
- The later frontier at frame 9312 has no Start input in the surrounding window, so native’s direct intro-sequence-3 -> title shortcut is unjustified there.
- The remaining open question is what the no-Start natural path does after the second intro-sequence-3 visit: longer hold inside segment 2, or a later intro4/intro5 handoff before title.

## Strongest Evidence

- `.cache/upstream/kid-chameleon-disasm/kid.asm`
  - `sub_1B93E` / `loc_1B96A` only write `Game_Mode=$20` on natural completion.
  - `sub_1C246` is the only path here that writes `Game_Mode=$2C` directly from intro sequence 3, and it is gated by Start (`bclr #7,(Ctrl_1_Pressed).w`).
- `python` over `.cache/tas-imports/any-percent-published/scenario.json`
  - frame `6207 = 128` (`Start`)
  - frames `9308..9316` contain no Start bit
- `cargo run -p kidcli -- tas-diff --scenario .cache/tas-imports/any-percent-published/scenario.json --frame-start 6206 --frame-end 6210 --include-advisory`
  - oracle and native both reach title on frame `6209`, consistent with the frame-6207 Start press and the deferred title-return path
- `cargo run -p kidcli -- tas-diff --scenario .cache/tas-imports/any-percent-published/scenario.json --frame-start 9312 --frame-end 9350 --include-advisory`
  - oracle stays in `game_mode=0x001C` with the second-roster intro-sequence-3 object roster throughout the no-Start window while native is already in title

## Exact Next Moves

1. Patch `step_boot_state()` so the timeout-restart intro-sequence-3 direct-title path is Start-driven only; do not treat the segment-2 override itself as a natural title return.
2. After that patch, rerun `tas-diff 9312..9350` to find the next true no-Start frontier and determine whether oracle stays in intro sequence 3 longer or advances into intro sequence 4/5.
3. Once the no-Start natural path is clear, add a focused regression around the first post-9312 authoritative frame and rerun the slice verification set.

## Verification To Re-run

- `cargo test -p kid-core published_tas_only_presses_start_on_the_first_timeout_restart_intro_sequence_3_visit -- --nocapture`
- `cargo test -p kid-core published_tas_demo_2_carries_intro_sequence_3_pose_into_the_first_title_frames -- --nocapture`
- `cargo test -p kid-core published_tas_demo_1_uses_the_second_intro_sequence_3_roster_pose_on_frame_9097 -- --nocapture`
- `cargo run -p kidcli -- tas-diff --scenario .cache/tas-imports/any-percent-published/scenario.json --frame-start 6206 --frame-end 6210 --include-advisory`
- `cargo run -p kidcli -- tas-diff --scenario .cache/tas-imports/any-percent-published/scenario.json --frame-start 9094 --frame-end 9100 --include-advisory`
- `cargo run -p kidcli -- tas-diff --scenario .cache/tas-imports/any-percent-published/scenario.json --frame-start 9312 --frame-end 9350 --include-advisory`
- `cargo run -p kidcli -- verify-tas --scenario .cache/tas-imports/any-percent-published/scenario.json --frame-start 6975 --frame-end 9530 --bisect`
- `mise run verify-slice-fast`

## Dead Ends / Do Not Retry

- Do not keep treating frame 6209 as evidence that the timeout-restart intro-sequence-3 timer naturally returns to title. That window is Start-driven.
- Do not use the first timeout-restart visit to justify the later no-Start frame-9312 title return.
- Do not reopen the frame-9097 pose-selection work; the second-roster pose is already correct.
