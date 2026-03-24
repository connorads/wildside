---
slug: any-percent-published-frame-17346-title-demo-start-one-frame-early
cycle: 62
updated_at: 2026-03-24T06:04:40Z
---

# Source-Port Frontier Handoff

## What Changed

- Fixed the frame-16798 title-return frontier by keeping the later Start-skip intro-return lineage hidden for one extra title rebuild frame before restoring the standard title pose.
- Added `published_tas_intro_return_title_keeps_kid_hidden_through_frame_16798`.
- Preserved the older timeout-restart intro-sequence-5 title-return timing that already restores the standard title pose on frame 10438.

## Current Model

- The old frame-16798 theory was correct: native restored the standard title pose one frame too early on the later Start-skip intro-return lineage.
- ASM `sub_1C246` still jumps directly from intro sequence 5 to `Game_Mode=$2C`; the difference is not another intro-sequence timer, it is the later title rebuild window after that shared hand-off.
- Oracle keeps the later lineage in `Game_Mode=$2C` with `slot2_x/y = 0` through frame 16798, then restores the standard title pose `(x=$2E,y=$8C)` on frame 16799.
- Native now matches through frame 17345.
- `verify-tas --frame-start 16650 --bisect` now lands at frame `17346`.
- The new first authoritative break is a one-frame mode transition: native reports `game_mode=0x0010` while oracle is still `0x002C` on frame 17346, then they rejoin on 17347.

## Strongest Evidence

- ASM
  - `.cache/upstream/kid-chameleon-disasm/kid.asm`
    - `sub_1C246` writes `move.w #$2C,(Game_Mode).w` on the Start-driven intro return.
    - `Obj_TitleMenu` / `loc_1BAEA` starts `move.w #$280,d0`, clears `Ctrl_Pressed`, hibernates one frame per loop, and only `loc_1BB02` flips to `move.w #$10,(Game_Mode).w`.
- Oracle
  - `cargo run -q -p kidcli -- oracle-probe --scenario .cache/tas-imports/any-percent-published/scenario.json --frame-start 16796 --frame-end 16800 --watches 0xFFFFFBCA:u16,0x00FF24FA:u16,0x00FF24FE:u16`
    - frame 16798 keeps `Game_Mode=0x2C` with `slot2_x=0`, `slot2_y=0`
    - frame 16799 keeps `Game_Mode=0x2C` and restores the standard title pose `(0x2E,0x8C)`
  - `cargo run -q -p kidcli -- tas-diff --scenario .cache/tas-imports/any-percent-published/scenario.json --frame-start 17342 --frame-end 17350 --include-advisory`
    - frame 17346 diverges only on `game_mode` (`native=0x0010`, `oracle=0x002C`)
- Native
  - `crates/kid-core/src/boot.rs`
    - carries the later intro-return lineage bit into the timeout-restart intro-sequence-5 title return
  - `crates/kid-core/src/trace.rs`
    - adds a lineage-specific one-frame extension to the intro-sequence-5 title hidden window
  - `crates/kid-core/src/tests.rs`
    - adds `published_tas_intro_return_title_keeps_kid_hidden_through_frame_16798`
- Verification
  - `cargo test -q -p kid-core published_tas_intro_return_title_keeps_kid_hidden_through_frame_16798 -- --nocapture`
  - `cargo test -q -p kid-core published_tas_intro_sequence_5_waits_for_the_frame_16648_start_press -- --nocapture`
  - `cargo test -q -p kid-core published_tas_second_timeout_restart_carries_intro_sequence_5_pose_into_title -- --nocapture`
  - `cargo run -q -p kidcli -- tas-diff --scenario .cache/tas-imports/any-percent-published/scenario.json --frame-start 16794 --frame-end 16802 --include-advisory`
  - `cargo run -q -p kidcli -- verify-tas --scenario .cache/tas-imports/any-percent-published/scenario.json --frame-start 16650 --bisect`
    - first CRC divergence moved to frame `17346`
  - `mise run verify-slice-fast`

## Exact Next Moves

1. Read `Obj_TitleMenu` in `.cache/upstream/kid-chameleon-disasm/kid.asm`, especially `loc_1BAEA` and `loc_1BB02`, before touching native code again.
2. Explain why native reaches `GameMode::DemoPlay (0x0010)` one frame early on 17346 while oracle still stays in `Game_Mode=$2C` until 17347. Do not assume the cause is the same hidden-pose gate just fixed.
3. Add one focused regression for frame 17346 only after ASM + the narrow diff agree on the exact title-idle/demo-start fencepost.

## Verification To Re-run

- `cargo test -q -p kid-core published_tas_intro_return_title_keeps_kid_hidden_through_frame_16798 -- --nocapture`
- `cargo test -q -p kid-core published_tas_intro_sequence_5_waits_for_the_frame_16648_start_press -- --nocapture`
- `cargo test -q -p kid-core published_tas_second_timeout_restart_carries_intro_sequence_5_pose_into_title -- --nocapture`
- `cargo run -q -p kidcli -- tas-diff --scenario .cache/tas-imports/any-percent-published/scenario.json --frame-start 16794 --frame-end 16802 --include-advisory`
- `cargo run -q -p kidcli -- tas-diff --scenario .cache/tas-imports/any-percent-published/scenario.json --frame-start 17342 --frame-end 17350 --include-advisory`
- `cargo run -q -p kidcli -- verify-tas --scenario .cache/tas-imports/any-percent-published/scenario.json --frame-start 16650 --bisect`
- `mise run verify-slice-fast`

## Dead Ends / Do Not Retry

- Do not shift `TIMEOUT_RESTART_TITLE_INTRO_SEQUENCE_5_HIDDEN_FRAMES` globally. The plain timeout-restart path already matches frame 10438.
- Do not reopen intro-sequence-5 slot-2 visibility timing for the frame-16798 frontier. The authoritative break was after `sub_1C246` had already entered `Game_Mode=$2C`.
- Do not assume the new frame-17346 frontier is another pose-visibility issue. The narrow diff shows only `game_mode`, so start with the title idle/demo-start ASM instead.
