---
slug: any-percent-published-frame-28098-mode-28-camera-wrap-follow-on
cycle: 73
updated_at: 2026-03-27T06:10:00Z
---

# Source-Port Frontier Handoff

## What Changed

- Fixed the frame-28098 later mode-0x28 camera miss by splitting the later `IntroSequence6` stale-slot-2 lineage from the earlier timeout-restart hidden-camera-wrap lineage.
- Added [`published_tas_later_intro_sequence_6_revisit_keeps_camera_cleared_on_frame_28098`](/home/connor/git/kc/crates/kid-core/src/tests.rs), locking the oracle-backed window where the later revisit stays at `camera_y_pos=0` on local mode-`$28` frames 31-33.
- Re-ran `cargo test -q -p kid-core published_tas_later_intro_sequence_6_revisit_keeps_camera_cleared_on_frame_28098 -- --nocapture`, `cargo test -q -p kid-core published_tas_intro_sequence_6_wraps_hidden_camera_on_frame_20926 -- --nocapture`, `cargo test -q -p kid-core published_tas_later_intro_sequence_5_revisit_enters_intro_sequence_6_on_frame_28066 -- --nocapture`, and `mise run verify-slice-fast`. I also started `cargo run -q -p kidcli -- verify-tas --scenario .cache/tas-imports/any-percent-published/scenario.json --frame-start 23250 --bisect`, but it did not finish within this session window.

## Current Model

- ASM [`Load_IntroSequence6`](/home/connor/git/kc/.cache/upstream/kid-chameleon-disasm/kid.asm) always seeds `($FFFFA656).w` and calls `sub_1B222`, but the later title-restore revisit does not expose the earlier timeout-restart hidden wrap on frame 28098 even though it still carries `loc_1BCD0`'s stale slot-2 pose.
- Native had been using `carry_intro_sequence_5_slot_2_pose` for both behaviours. That was too coarse: frame 20926 really needs the hidden wrap, but frame 28098 only needs the stale pose carry.
- [`BootState::IntroSequence6`](/home/connor/git/kc/crates/kid-core/src/types.rs) now stores a separate `carry_timeout_restart_hidden_camera_wrap` bit, and [`trace.rs`](/home/connor/git/kc/crates/kid-core/src/trace.rs) only exposes the `$FFF0_0000` / ramped hidden camera when that narrower lineage bit is set.

## Strongest Evidence

- Native
  - [`boot.rs`](/home/connor/git/kc/crates/kid-core/src/boot.rs)
    - `enter_intro_sequence_6_from_timeout_restart_intro_sequence_5()` now snapshots `demo_timeout_restart_uses_intro_sequence_6_early_intro_sequence_5_handoff` before `enter_title_screen()` clears it, then uses that to decide whether the new `carry_timeout_restart_hidden_camera_wrap` bit should be set.
  - [`types.rs`](/home/connor/git/kc/crates/kid-core/src/types.rs)
    - `BootState::IntroSequence6` now distinguishes stale slot-2 pose carry from hidden camera wrap carry.
  - [`trace.rs`](/home/connor/git/kc/crates/kid-core/src/trace.rs)
    - `intro_sequence_6_timeout_restart_hidden_camera_y()` now requires `carry_timeout_restart_hidden_camera_wrap: true`.
  - [`tests.rs`](/home/connor/git/kc/crates/kid-core/src/tests.rs)
    - `published_tas_later_intro_sequence_6_revisit_keeps_camera_cleared_on_frame_28098` and `published_tas_intro_sequence_6_wraps_hidden_camera_on_frame_20926` both pass, proving the split.
- Oracle / parity
  - `mise run frontier-status`
    - Before the fix, the live blocker was frame 28098 with authoritative `camera_y_pos` mismatch `native=0xFFF00000`, `oracle=0x00000000`.
  - `cargo test -q -p kid-core published_tas_later_intro_sequence_6_revisit_keeps_camera_cleared_on_frame_28098 -- --nocapture`
    - Local mode-`$28` frames 31, 32, and 33 now all stay at `camera_y_pos=0`.
  - `mise run verify-slice-fast`
    - Green after the lineage split.

## Exact Next Moves

1. Re-run `cargo run -q -p kidcli -- verify-tas --scenario .cache/tas-imports/any-percent-published/scenario.json --frame-start 23250 --bisect` to capture the next first CRC divergence after frame 28098.
2. Once the new frontier frame is known, run a narrow `cargo run -q -p kidcli -- tas-diff --scenario .cache/tas-imports/any-percent-published/scenario.json --frame-start <frame-6> --frame-end <frame+6> --include-advisory`.
3. Read the owning ASM for that new frame before touching native code again.

## Verification To Re-run

- `cargo test -q -p kid-core published_tas_later_intro_sequence_5_revisit_enters_intro_sequence_6_on_frame_28066 -- --nocapture`
- `cargo test -q -p kid-core published_tas_later_intro_sequence_6_revisit_keeps_camera_cleared_on_frame_28098 -- --nocapture`
- `cargo test -q -p kid-core published_tas_intro_sequence_6_wraps_hidden_camera_on_frame_20926 -- --nocapture`
- `cargo run -q -p kidcli -- verify-tas --scenario .cache/tas-imports/any-percent-published/scenario.json --frame-start 23250 --bisect`
- `mise run verify-slice-fast`

## Dead Ends / Do Not Retry

- Do not key the hidden camera wrap off `carry_intro_sequence_5_slot_2_pose` alone again. Frame 28098 proves the later title-restore revisit carries the stale pose without carrying the wrap.
- Do not read `demo_timeout_restart_uses_intro_sequence_6_early_intro_sequence_5_handoff` after `enter_title_screen()`. That helper resets the lineage bits; snapshot what you need first.
- Do not revisit `loc_1BA5C` timing for this fixed frame. The bug was the shared IntroSequence6 lineage bit, not another mode-`$24` timeout length issue.
