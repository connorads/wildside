---
slug: death_respawn_level1_case_00000_00100
cycle: 15
updated_at: 2026-03-31T23:59:00Z
---

## What Changed

- Re-ran `cargo run -p kidcli -- autonomous-loop --goal case-queue`; cycle 15 selected `death_respawn_level1_case_00000_00100`.
- Re-verified the selected case with the exact selector `death-respawn-level1-0-100` and refreshed the stale bundle state before editing.
- Read `.cache/upstream/kid-chameleon-disasm/kid.asm` around `Load_SegaScreen`, `sub_1B8F4`, and `sub_1C246` before touching boot/gameplay timing.
- Patched [`crates/kid-core/src/boot.rs`](/home/connor/git/kc/crates/kid-core/src/boot.rs) so intro Start-to-title returns use the ROM-backed one-frame outer deferral and let intro sequence 1 jump to title from any visible frame.
- Added/updated focused regressions in [`crates/kid-core/src/tests.rs`](/home/connor/git/kc/crates/kid-core/src/tests.rs) for the frame-78 title return and the repaired early-start boot path anchors.
- Removed the now-unused `INTRO_FIRST_SEGMENT_SKIP_ADVANCE_FRAMES` constant from [`crates/kid-core/src/constants.rs`](/home/connor/git/kc/crates/kid-core/src/constants.rs).
- Re-ran the selected case; it now passes.

## Selected Case

- Case ID: `death_respawn_level1_case_00000_00100`
- Selector: `death-respawn-level1-0-100`
- Autonomy gate: `patch-allowed`
- Current verify result: `PASS`
- Current first failure: none

## Why This Case

- Cycle 15 ranked this as the top autonomy-executable case.
- After refresh, the live first failure narrowed to boot timing at local frame 78: `world.game_mode` stayed at `$0004` natively while oracle had already moved to `$002C`.
- The ROM path was explicit and localised to the intro-return helper, so this was a focused engine-level fix rather than a case-specific hack.

## Strongest Evidence

- `.cache/upstream/kid-chameleon-disasm/kid.asm:29988` (`sub_1B8F4`) calls `sub_1C246` every intro-sequence-1 loop frame.
- `.cache/upstream/kid-chameleon-disasm/kid.asm:31119` / `:31125` (`sub_1C246 -> loc_1C250`) clears Start and writes `Game_Mode=$2C` through the shared title-return path.
- `cargo run -p kidcli -- case verify --case death-respawn-level1-0-100` now reports `case death_respawn_level1_case_00000_00100: PASS`.
- `cargo test -p kid-core intro_sequence_1_start_skip_reaches_title_on_same_frame -- --nocapture` passes and locks the repaired frame-78 return.

## Exact Next Moves

1. Run `cargo run -p kidcli -- autonomous-loop --goal case-queue` again and take the next selected case only.
2. If a later boot/timing case regresses, start from `sub_1B8F4` and `sub_1C246` in `.cache/upstream/kid-chameleon-disasm/kid.asm` before changing any intro/title timing.

## Verification To Re-run

- `cargo test -p kid-core intro_sequence_1_start_skip_reaches_title_on_same_frame -- --nocapture`
- `cargo test -p kid-core timing_regression_early_start_boot_path_reaches_level_transition_on_frame_178 -- --nocapture`
- `cargo test -p kid-core timing_regression_early_start_boot_path_stays_in_gameplay_through_frame_300 -- --nocapture`
- `cargo test -p kid-core boot_with_start_presses_stays_out_of_title_after_early_title_select -- --nocapture`
- `cargo run -p kidcli -- case verify --case death-respawn-level1-0-100`

## Dead Ends / Do Not Retry

- Do not trust the pre-refresh `verify.json`/`summary.md` for this case; they still pointed at the stale `player.x_pos` frame-0 mismatch until the exact selector was re-verified.
- Do not restore the old 2-frame `pending_title_skip` delay for intro returns; the ROM helper `sub_1C246` only needs a one-frame outer defer in native tracing.
- Do not keep the old intro-sequence-1 Start gating. The ROM allows the title-return request from the visible intro-1 frames instead of waiting for the old synthetic threshold.
