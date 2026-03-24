---
slug: any-percent-published-frame-9312-native-enters-intro-sequence-4-before-oracle-leaves-intro-sequence-3
cycle: 41
updated_at: 2026-03-24T00:22:10Z
---

# Source-Port Frontier Handoff

## What Changed

- Re-read `sub_1B93E` / `loc_1B96A`, `sub_1C246` / `loc_1C250`, and the `Load_IntroSequence3` caller in `kid.asm`.
- Patched `step_boot_state()` so the timeout-restart intro-sequence-3 direct-title path is Start-driven only, matching `sub_1C246`; the segment-2 override no longer forces a natural title return.
- Added `published_tas_second_timeout_restart_intro_sequence_3_visit_does_not_jump_to_title_without_start`.
- Re-ran targeted tests, `tas-diff 9312..9350`, `verify-tas --frame-start 6975 --frame-end 9530 --bisect`, and `mise run verify-slice-fast`.

## Current Model

- The old frame-9312 bug is fixed in the narrow sense: native no longer jumps straight from timeout-restart intro sequence 3 to title without Start.
- The live frontier is still frame 9312, but it is now `game_mode=0x0020` (intro sequence 4) in native versus `game_mode=0x001C` (intro sequence 3) in oracle.
- Oracle still shows the second `Load_IntroSequence3` roster alive throughout frames 9312..9350, with Kid visible at `0x00700000,0x00CC0000` and object logic still sitting at `loc_1B96A`.
- The remaining bug is therefore the natural no-Start intro-sequence-3 lifetime on the second visit: native advances out of `loc_1B96A` too early, not to title but to intro sequence 4.

## Strongest Evidence

- `.cache/upstream/kid-chameleon-disasm/kid.asm`
  - `sub_1B93E` / `loc_1B96A` count down with `d0=$EF` then naturally write `Game_Mode=$20`.
  - `sub_1C246` / `loc_1C250` are the only direct-title path here, gated by `bclr #7,(Ctrl_1_Pressed).w`.
  - `Load_IntroSequence3` installs `sub_1B93E` as the object callback for this attract segment.
- `cargo run -p kidcli -- tas-diff --scenario .cache/tas-imports/any-percent-published/scenario.json --frame-start 9312 --frame-end 9350 --include-advisory`
  - native: `game_mode=0x0020`, no intro objects, Kid cleared
  - oracle: `game_mode=0x001C`, Kid still visible at `0x00700000,0x00CC0000`, object roster still rooted at `lp=0x01B96A`
- `cargo run -p kidcli -- verify-tas --scenario .cache/tas-imports/any-percent-published/scenario.json --frame-start 6975 --frame-end 9530 --bisect`
  - first CRC divergence remains frame 9312 after the patch, so the bogus title jump is gone and the next causal bug is the too-early intro-sequence-4 handoff

## Exact Next Moves

1. Read the second-visit intro-sequence-3 setup/reset path in ASM around `Load_IntroSequence3` and its callers to explain why oracle is still running `loc_1B96A` on frame 9312 instead of timing out into intro sequence 4.
2. Compare native `elapsed`/override state for the second visit against the oracle-backed timeline; patch the natural no-Start lifetime only once the re-entry/reset rule is clear from ASM.
3. After that fix, replace the broad “does not jump to title” regression with an oracle-backed frame-9312 check for the correct intro-sequence-3 state and rerun the same verification set.

## Verification To Re-run

- `cargo test -p kid-core published_tas_only_presses_start_on_the_first_timeout_restart_intro_sequence_3_visit -- --nocapture`
- `cargo test -p kid-core published_tas_demo_2_carries_intro_sequence_3_pose_into_the_first_title_frames -- --nocapture`
- `cargo test -p kid-core published_tas_demo_1_uses_the_second_intro_sequence_3_roster_pose_on_frame_9097 -- --nocapture`
- `cargo test -p kid-core published_tas_second_timeout_restart_intro_sequence_3_visit_does_not_jump_to_title_without_start -- --nocapture`
- `cargo run -p kidcli -- tas-diff --scenario .cache/tas-imports/any-percent-published/scenario.json --frame-start 9312 --frame-end 9350 --include-advisory`
- `cargo run -p kidcli -- verify-tas --scenario .cache/tas-imports/any-percent-published/scenario.json --frame-start 6975 --frame-end 9530 --bisect`
- `mise run verify-slice-fast`

## Dead Ends / Do Not Retry

- Do not reintroduce the old `segment == 2 && intro_sequence_3_frames_override.is_some()` natural-title shortcut; ASM proves that path was Start-driven only.
- Do not use the first timeout-restart visit at frame 6209 as evidence for the second visit's natural no-Start behaviour; frame 6207 is a real Start press.
- Do not treat the remaining frame-9312 bug as a title-screen issue. The live mismatch is now intro-sequence-4 versus intro-sequence-3.
