---
slug: death_respawn_level1_case_00000_00060
cycle: 14
updated_at: 2026-03-31T22:58:15Z
---

## What Changed

- Re-ran `cargo run -p kidcli -- autonomous-loop --goal case-queue`; cycle 14 selected `death_respawn_level1_case_00000_00060` again.
- Re-verified the selected case before editing and confirmed the live first failure was still frame 33 `world.game_mode` (`native 0`, `oracle 4`).
- Read `.cache/upstream/kid-chameleon-disasm/kid.asm` around `Load_SegaScreen`, especially `loc_12E96 -> loc_12EDE`, before changing gameplay timing.
- Patched [`crates/kid-core/src/boot.rs`](/home/connor/git/kc/crates/kid-core/src/boot.rs) so the Sega-screen Start edge switches to intro sequence 1 on the same traced frame instead of deferring one extra frame.
- Added a regression in [`crates/kid-core/src/tests.rs`](/home/connor/git/kc/crates/kid-core/src/tests.rs) that locks the frame-33 Start edge to the ROM-backed same-frame `Game_Mode=$4` transition.
- Updated the broader boot expectation in [`crates/kid-core/src/tests.rs`](/home/connor/git/kc/crates/kid-core/src/tests.rs) to reflect the observed two-frame `LevelTransition` shell at frames 460..461 before `InGame` begins on frame 462.
- Re-ran the selected case; it now passes.

## Selected Case

- Case ID: `death_respawn_level1_case_00000_00060`
- Selector: `death-respawn-level1-case`
- Autonomy gate: `patch-allowed`
- Current verify result: `PASS`
- Current first failure: none

## Why This Case

- Cycle 14 still ranked this as the top autonomy-executable case.
- The remaining failure was a single authoritative `world.game_mode` edge inside the boot/start path, small enough to fix without widening the frontier.
- The ROM path was explicit, so this was a good candidate for a focused timing correction rather than more instrumentation.

## Strongest Evidence

- `.cache/upstream/kid-chameleon-disasm/kid.asm:25519` / `:25521` / `:25546` (`loc_12E96 -> loc_12EDE`) reads Start with `bclr #7,(Ctrl_1_Pressed).w` and branches straight to `move.w #4,(Game_Mode).w`.
- `cargo run -p kidcli -- case verify --case death-respawn-level1-case` now reports `case death_respawn_level1_case_00000_00060: PASS`.
- `.cache/cases/death-respawn-level1-case/verify.json` now has `"passed": true` and `"first_failure": null`.
- `cargo run -p kidcli -- probe-native --scenario boot-title-start-level1 --frame-start 455 --frame-end 465` shows frame 460 entering `LevelTransition` on the Start edge and frame 462 entering `InGame`, matching the updated broader boot regression.

## Exact Next Moves

1. Run `cargo run -p kidcli -- autonomous-loop --goal case-queue` again and take the next selected case, not this resolved one.
2. If this case regresses later, start from `.cache/upstream/kid-chameleon-disasm/kid.asm` `Load_SegaScreen` and re-check `loc_12E96 -> loc_12EDE` before touching any title/boot timing code.

## Verification To Re-run

- `cargo test -p kid-core sega_screen_start_press_switches_to_intro_sequence_on_same_frame -- --nocapture`
- `cargo test -p kid-core boot_with_start_presses_reaches_ingame_at_expected_frame -- --nocapture`
- `cargo run -p kidcli -- case verify --case death-respawn-level1-case`
- `cargo run -p kidcli -- case summarise --case death-respawn-level1-case`

## Dead Ends / Do Not Retry

- Do not restore the old Sega-screen deferral. The ROM does not keep Start hidden for an extra traced frame once `loc_12E96` sees the edge.
- Do not widen this session into other case IDs. This selected case is already green.
- Do not infer the broader boot timing from the old test expectation; use `probe-native --scenario boot-title-start-level1` when the frame boundary matters.
