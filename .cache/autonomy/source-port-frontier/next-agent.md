---
slug: any-percent-published-frame-28396-mode-28-title-pose-hidden
cycle: 74
updated_at: 2026-03-27T05:40:00Z
---

# Source-Port Frontier Handoff

## What Changed

- Fixed the frame-28317 later mode-0x28 camera miss by teaching the non-wrap `IntroSequence6` revisit to run the same 16-frame `GfxObjects_Collision_BottomBoundary4` worker from the delayed `$FFD00000` seed.
- Added [`published_tas_later_intro_sequence_6_revisit_starts_delayed_camera_ramp_on_frame_28317`](/home/connor/git/kc/crates/kid-core/src/tests.rs), locking the oracle-backed window where the later revisit stays flat at `$FFD00000` through local mode-`$28` frame 250 and then ramps by +3px/frame from frame 251.
- Re-ran `cargo test -q -p kid-core published_tas_later_intro_sequence_6_revisit_starts_delayed_camera_ramp_on_frame_28317 -- --nocapture`, `cargo test -q -p kid-core published_tas_later_intro_sequence_6_revisit_uses_delayed_seeded_camera_row_on_frame_28101 -- --nocapture`, `cargo test -q -p kid-core published_tas_intro_sequence_6_wraps_hidden_camera_on_frame_20926 -- --nocapture`, `cargo test -q -p kid-core published_tas_intro_sequence_6_starts_hidden_camera_ramp_on_frame_21142 -- --nocapture`, `cargo run -q -p kidcli -- tas-diff --scenario .cache/tas-imports/any-percent-published/scenario.json --frame-start 28312 --frame-end 28324 --include-advisory`, `cargo run -q -p kidcli -- verify-tas --scenario .cache/tas-imports/any-percent-published/scenario.json --frame-start 23250 --bisect`, and `mise run verify-slice-fast`.

## Current Model

- ASM [`Load_IntroSequence6`](/home/connor/git/kc/.cache/upstream/kid-chameleon-disasm/kid.asm) still seeds `($FFFFA656).w = $FFD00000` and mirrors it into `Camera_Y_pos` via `sub_1B222`, even on the later title-restore revisit.
- When [`GfxObjects_Collision_BottomBoundary4`](/home/connor/git/kc/.cache/upstream/kid-chameleon-disasm/kid.asm) later runs, it snapshots `d1 = ($FFFFA656).w >> 4` once and then applies `sub.l d1,($FFFFA656).w; bsr.w sub_1B222` for 16 one-frame iterations. With the later revisit's `$FFD00000` seed, `d1=$FFFD0000`, so the oracle-visible hidden camera rises by 3 pixels per frame instead of 1.
- [`trace.rs`](/home/connor/git/kc/crates/kid-core/src/trace.rs) now models that later non-wrap ramp separately from the earlier wrapped `$FFF0_0000` lineage. The next frontier is no longer camera state: native now exposes the standard title pose (`x=$2E`, `y=$8C`) three local mode-`$28` frames before oracle on frames 28396..28398.

## Strongest Evidence

- Native
  - [`trace.rs`](/home/connor/git/kc/crates/kid-core/src/trace.rs)
    - `intro_sequence_6_timeout_restart_hidden_camera_y()` now switches the later revisit from the delayed flat `$FFD00000` seed into a 16-frame `+$00030000` ramp starting on local mode-`$28` frame 251.
  - [`constants.rs`](/home/connor/git/kc/crates/kid-core/src/constants.rs)
    - Added a later-revisit-specific ramp start constant so the delayed seed and delayed worker timing stay separate from the earlier wrapped lineage.
  - [`tests.rs`](/home/connor/git/kc/crates/kid-core/src/tests.rs)
    - `published_tas_later_intro_sequence_6_revisit_starts_delayed_camera_ramp_on_frame_28317` passes and proves the new +3px/frame sequence.
- Oracle / parity
  - `cargo run -q -p kidcli -- tas-diff --scenario .cache/tas-imports/any-percent-published/scenario.json --frame-start 28312 --frame-end 28324 --include-advisory`
    - Frames 28312..28323 are now advisory-only; the old `camera_y_pos` miss is gone.
  - `cargo run -q -p kidcli -- verify-tas --scenario .cache/tas-imports/any-percent-published/scenario.json --frame-start 23250 --bisect`
    - The first CRC divergence moved to frame 28396.
  - `cargo run -q -p kidcli -- tas-diff --scenario .cache/tas-imports/any-percent-published/scenario.json --frame-start 28390 --frame-end 28402 --include-advisory`
    - Frames 28396..28398 now show the next authoritative miss: native `kid_x_pos=0x002E0000`, `kid_y_pos=0x008C0000`; oracle keeps both zero.

## Exact Next Moves

1. Read the later mode-`$28` title-pose write path around `loc_1B5CE` / the shared title-shell workers in [`kid.asm`](/home/connor/git/kc/.cache/upstream/kid-chameleon-disasm/kid.asm) before touching native code again.
2. Add a focused regression for frames 28396..28398 proving oracle keeps Kid hidden while native currently exposes the standard title pose.
3. Re-run `cargo run -q -p kidcli -- tas-diff --scenario .cache/tas-imports/any-percent-published/scenario.json --frame-start 28390 --frame-end 28402 --include-advisory` after the next patch so the three-frame title-pose settle is what moves.

## Verification To Re-run

- `cargo test -q -p kid-core published_tas_later_intro_sequence_6_revisit_starts_delayed_camera_ramp_on_frame_28317 -- --nocapture`
- `cargo test -q -p kid-core published_tas_later_intro_sequence_6_revisit_uses_delayed_seeded_camera_row_on_frame_28101 -- --nocapture`
- `cargo test -q -p kid-core published_tas_intro_sequence_6_wraps_hidden_camera_on_frame_20926 -- --nocapture`
- `cargo test -q -p kid-core published_tas_intro_sequence_6_starts_hidden_camera_ramp_on_frame_21142 -- --nocapture`
- `mise run verify-slice-fast`
- `cargo run -q -p kidcli -- verify-tas --scenario .cache/tas-imports/any-percent-published/scenario.json --frame-start 23250 --bisect`
- `cargo run -q -p kidcli -- tas-diff --scenario .cache/tas-imports/any-percent-published/scenario.json --frame-start 28312 --frame-end 28324 --include-advisory`
- `cargo run -q -p kidcli -- tas-diff --scenario .cache/tas-imports/any-percent-published/scenario.json --frame-start 28390 --frame-end 28402 --include-advisory`

## Dead Ends / Do Not Retry

- Do not flatten the later non-wrap camera path back to a permanent `$FFD00000`. Frame 28317 proves that revisit still runs the same `GfxObjects_Collision_BottomBoundary4` worker, just from a different seed.
- Do not reuse the earlier wrapped-lineage ramp timing for the later revisit. The later seed appears three local mode-`$28` frames later, and the ramp start shifts with it.
- Do not keep debugging camera state for this frontier. `tas-diff` now shows the next blocker is Kid visibility (`kid_x_pos` / `kid_y_pos`), not `camera_y_pos`.
