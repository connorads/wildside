---
slug: any-percent-published-post-16659-next-frontier-pending-bisect
cycle: 61
updated_at: 2026-03-24T05:47:20Z
---

# Source-Port Frontier Handoff

## What Changed

- Fixed the frame-16633 intro-sequence-5 frontier by keeping the Start-skip intro-return lineage in `Game_Mode=$24` for one extra fade window.
- Added `published_tas_intro_sequence_5_waits_for_the_frame_16648_start_press`.
- Updated the timeout-restart intro-sequence-5 timer so this lineage no longer expires to title before the real frame-16648 Start press can arm `sub_1C246`.

## Current Model

- The old frame-16633 theory was wrong. Native was not reaching `finish_timeout_restart_intro_sequence_5_title_return()` early.
- ASM `loc_1BA5C` only leaves intro sequence 5 via natural timeout or `sub_1C246`.
- The published TAS has no Start input on frames 16620..16647. The real Start press is frame 16648.
- Oracle stays in `Game_Mode=$24` with `loc_1BCD0` slot 2 at `(x=0,y=$E0)` through frame 16649, then the shared 2-frame `sub_1C246` deferral reaches title on frame 16650.
- Native had been expiring the timeout-restart copy of intro sequence 5 seventeen frames too early on the Start-skip intro-return lineage. Holding that lineage for one extra fade window clears the old authoritative break.
- `tas-diff --frame-start 16629 --frame-end 16660 --include-advisory` is now authoritative-clean through frame 16659; only advisory `number_objects` drift remains there.
- `verify-tas --frame-start 13189 --bisect` now lands at frame `16798`.

## Strongest Evidence

- ASM
  - `.cache/upstream/kid-chameleon-disasm/kid.asm`
    - `loc_1BA28` calls `sub_1C204` before dispatching to `loc_1BA5C`.
    - `loc_1BA5C` is only `Hibernate_Object_1Frame` + `sub_1C246` polling + DBF loop.
    - `sub_1C246` writes `move.w #$2C,(Game_Mode).w` only on a latched Start press.
- Oracle
  - `cargo run -q -p kidcli -- oracle-probe --scenario .cache/tas-imports/any-percent-published/scenario.json --frame-start 16633 --frame-end 16645 --watches 0xFFFFFBCA:u16,0x00FF24FA:u16,0x00FF24FE:u16`
    - frames 16633..16645 keep `Game_Mode=0x24`, `slot2_x=0`, `slot2_y=0xE0`
  - `jq '.inputs[16646:16651]' .cache/tas-imports/any-percent-published/scenario.json`
    - `[0,0,128,0,0]`
- Native
  - `crates/kid-core/src/boot.rs`
    - adds one extra fade window only when `demo_timeout_restart_uses_intro_return_slot_2_early_visibility` is set on timeout-restart intro sequence 5
  - `crates/kid-core/src/constants.rs`
    - `DEMO_AUTO_TIMEOUT_INTRO_FIFTH_SEGMENT_START_SKIP_LINEAGE_EXTRA_FRAMES`
  - `crates/kid-core/src/tests.rs`
    - `published_tas_intro_sequence_5_waits_for_the_frame_16648_start_press`
- Verification
  - `cargo test -q -p kid-core published_tas_intro_sequence_5_waits_for_the_frame_16648_start_press -- --nocapture`
  - `cargo test -q -p kid-core published_tas_mode_24_exposes_loc_1bcd0_slot_2_on_frame_16228 -- --nocapture`
  - `cargo test -q -p kid-core published_tas_second_timeout_restart_exposes_intro_sequence_5_slot_2_on_frame_10007 -- --nocapture`
  - `cargo test -q -p kid-core published_tas_second_timeout_restart_keeps_intro_sequence_5_alive_past_frame_10029 -- --nocapture`
  - `cargo test -q -p kid-core published_tas_intro_return_demo_1_uses_the_oracle_hidden_camera_wrap_on_frame_13188 -- --nocapture`
  - `cargo test -q -p kid-core demo_1_bootstrap_keeps_camera_hidden_through_frame_2537 -- --nocapture`
  - `cargo test -q -p kid-core published_tas_intro_sequence_1_start_skip_keeps_kid_hidden_on_frame_12430 -- --nocapture`
  - `cargo run -q -p kidcli -- tas-diff --scenario .cache/tas-imports/any-percent-published/scenario.json --frame-start 16629 --frame-end 16660 --include-advisory`
  - `cargo run -q -p kidcli -- verify-tas --scenario .cache/tas-imports/any-percent-published/scenario.json --frame-start 13189 --bisect`
    - first CRC divergence moved to frame `16798`
  - `mise run verify-slice-fast`

## Exact Next Moves

1. Run `cargo run -q -p kidcli -- tas-diff --scenario .cache/tas-imports/any-percent-published/scenario.json --frame-start 16794 --frame-end 16802 --include-advisory` to classify the new frame-16798 frontier.
2. Once the new divergent fields are known, inspect the owning ASM before changing code again. Do not assume it is still intro-sequence timing.
3. Add one focused regression for frame 16798 only after ASM + narrow diff agree on the real cause.

## Verification To Re-run

- `cargo test -q -p kid-core published_tas_intro_sequence_5_waits_for_the_frame_16648_start_press -- --nocapture`
- `cargo test -q -p kid-core published_tas_mode_24_exposes_loc_1bcd0_slot_2_on_frame_16228 -- --nocapture`
- `cargo test -q -p kid-core published_tas_second_timeout_restart_exposes_intro_sequence_5_slot_2_on_frame_10007 -- --nocapture`
- `cargo test -q -p kid-core published_tas_second_timeout_restart_keeps_intro_sequence_5_alive_past_frame_10029 -- --nocapture`
- `cargo test -q -p kid-core published_tas_intro_return_demo_1_uses_the_oracle_hidden_camera_wrap_on_frame_13188 -- --nocapture`
- `cargo test -q -p kid-core demo_1_bootstrap_keeps_camera_hidden_through_frame_2537 -- --nocapture`
- `cargo test -q -p kid-core published_tas_intro_sequence_1_start_skip_keeps_kid_hidden_on_frame_12430 -- --nocapture`
- `cargo run -q -p kidcli -- tas-diff --scenario .cache/tas-imports/any-percent-published/scenario.json --frame-start 16629 --frame-end 16660 --include-advisory`
- `cargo run -q -p kidcli -- verify-tas --scenario .cache/tas-imports/any-percent-published/scenario.json --frame-start 13189 --bisect`
- `mise run verify-slice-fast`

## Dead Ends / Do Not Retry

- Do not debug `finish_timeout_restart_intro_sequence_5_title_return()` for the old frame-16633 frontier. The published TAS had no Start edge before frame 16648, so that helper could not be the cause.
- Do not move the shared timeout-restart intro-sequence-5 timer globally. The older frame-10007/frame-10029 regressions still need the plain timeout path unchanged.
- Do not reopen frame 16228 as a generic mode-`$24` visibility issue. That slot-2 frontier remains fixed.
