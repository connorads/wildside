---
slug: any-percent-published-frame-23248-intro-sequence-6-demo-return-stays-in-title-too-long
cycle: 69
updated_at: 2026-03-27T03:33:29Z
---

# Source-Port Frontier Handoff

## What Changed

- Fixed the frame-22700 later IntroSequence6 demo-return title-pose miss by teaching the generic demo-title rebuild window to end two frames earlier for that lineage in [`boot.rs`](/home/connor/git/kc/crates/kid-core/src/boot.rs), [`constants.rs`](/home/connor/git/kc/crates/kid-core/src/constants.rs), [`lib.rs`](/home/connor/git/kc/crates/kid-core/src/lib.rs), and [`trace.rs`](/home/connor/git/kc/crates/kid-core/src/trace.rs).
- Added [`published_tas_intro_sequence_6_timeout_restore_reuses_title_pose_on_frames_22700_and_22701`](/home/connor/git/kc/crates/kid-core/src/tests.rs), locking the last hidden title frame at 22699 plus the oracle-backed standard title pose on 22700..22701.
- Re-ran the focused IntroSequence6 regressions, `tas-diff --frame-start 22696 --frame-end 22704 --include-advisory`, `verify-tas --frame-start 21765 --bisect`, and `mise run verify-slice-fast`; the first authoritative divergence now moves to frame 23248.

## Current Model

- ASM `loc_1B5CE` restores the standard title pose before the later IntroSequence6 demo starts, and the published TAS hits Start on frame 22550, so the frame-22700 return is the generic demo-title path rather than a pure auto-timeout-only branch.
- Native now matches the earlier title-pose reappearance and stays authoritative-clean through frame 23247.
- The next blocker opens when that same later title return should re-enter demo play: on frames 23248..23249 native still reports `mode=0x002C` / `level=0x00`, while oracle has already flipped back into `mode=0x0010`; frame 23250 rejoins.

## Strongest Evidence

- ASM / script
  - `.cache/upstream/kid-chameleon-disasm/kid.asm`
    - `loc_1B5CE` writes `x_pos=$2E`, `y_pos=$8C` for the standard title pose.
    - `Obj_TitleMenu loc_1BAEA -> loc_1BB02` owns the later `$280` idle countdown and the eventual `move.w #$10,(Game_Mode).w` demo re-entry.
  - `.cache/tas-imports/any-percent-published/scenario.json`
    - Frame 22550 carries the published TAS Start press, so the frame-22700 return is on the generic demo-title path rather than a pure auto-timeout-only branch.
- Native
  - [`boot.rs`](/home/connor/git/kc/crates/kid-core/src/boot.rs)
    - Both demo-return entry points now preserve the later IntroSequence6 lineage bit before `enter_title_screen()` clears the demo bootstrap flags.
  - [`trace.rs`](/home/connor/git/kc/crates/kid-core/src/trace.rs)
    - `title_hides_demo_return_player()` now shortens the hidden rebuild window by two frames only for that later IntroSequence6 demo-return lineage.
  - [`tests.rs`](/home/connor/git/kc/crates/kid-core/src/tests.rs)
    - `published_tas_intro_sequence_6_timeout_restore_reuses_title_pose_on_frames_22700_and_22701` passes.
- Oracle / parity
  - `cargo run -q -p kidcli -- tas-diff --scenario .cache/tas-imports/any-percent-published/scenario.json --frame-start 22696 --frame-end 22704 --include-advisory`
    - Frames 22696..22703 now all match.
  - `cargo run -q -p kidcli -- verify-tas --scenario .cache/tas-imports/any-percent-published/scenario.json --frame-start 21765 --bisect`
    - First CRC divergence is now frame 23248.
  - `cargo run -q -p kidcli -- tas-diff --scenario .cache/tas-imports/any-percent-published/scenario.json --frame-start 23244 --frame-end 23252 --include-advisory`
    - Frames 23248..23249 diverge only on `game_mode` (`native=0x002C`, `oracle=0x0010`); frames 23244..23247 and 23250..23251 match.

## Exact Next Moves

1. Read `Obj_TitleMenu loc_1BAEA -> loc_1BB02` in `.cache/upstream/kid-chameleon-disasm/kid.asm` and prove why oracle flips back into `Game_Mode=$10` on frames 23248..23249 while native stays in `Game_Mode=$2C`.
2. Inspect the native title-idle countdown after the repaired frame-22700 title-pose restore and explain why the demo re-entry is two frames late.
3. Add one focused regression for the frame-23248/23249 `Game_Mode` re-entry only after the exact ASM countdown path is proven.

## Verification To Re-run

- `cargo test -q -p kid-core published_tas_intro_sequence_6_enters_demo_play_on_frame_21765 -- --nocapture`
- `cargo test -q -p kid-core published_tas_intro_sequence_6_uses_demo_1_hidden_camera_wrap_on_frame_21825 -- --nocapture`
- `cargo test -q -p kid-core published_tas_intro_sequence_6_keeps_demo_1_bootstrap_hidden_on_frame_21837 -- --nocapture`
- `cargo test -q -p kid-core published_tas_intro_sequence_6_timeout_restore_reuses_title_pose_on_frames_22700_and_22701 -- --nocapture`
- `cargo run -q -p kidcli -- tas-diff --scenario .cache/tas-imports/any-percent-published/scenario.json --frame-start 22696 --frame-end 22704 --include-advisory`
- `cargo run -q -p kidcli -- tas-diff --scenario .cache/tas-imports/any-percent-published/scenario.json --frame-start 23244 --frame-end 23252 --include-advisory`
- `cargo run -q -p kidcli -- verify-tas --scenario .cache/tas-imports/any-percent-published/scenario.json --frame-start 21765 --bisect`
- `mise run verify-slice-fast`

## Dead Ends / Do Not Retry

- Do not reopen the old cold-boot demo-1 wrap. `demo_1_bootstrap_keeps_camera_hidden_through_frame_2537` still expects `0xFF000000` on the cold-boot path.
- Do not reuse the Start-skip intro-return wrap. That path is already covered by `published_tas_intro_return_demo_1_uses_the_oracle_hidden_camera_wrap_on_frame_13188` and still expects `0xFF200000`.
- Do not keep treating frame 22700 as a pure auto-timeout bug. The published TAS script contains a real Start press on frame 22550, and the fixed behaviour belonged to the generic demo-title rebuild window.
