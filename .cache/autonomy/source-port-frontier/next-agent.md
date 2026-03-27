---
slug: any-percent-published-frame-29402-mode-28-demo-handoff
cycle: 74
updated_at: 2026-03-27T07:05:00Z
---

# Source-Port Frontier Handoff

## What Changed

- Fixed the frame-28396 later mode-0x28 title-pose miss by splitting the later `IntroSequence6` revisit from the earlier timeout-restart visibility threshold: the non-wrap lineage now keeps Kid hidden through local mode-`$28` frame 332 and only exposes `loc_1B5CE`'s standard title pose on frame 333.
- Added [`published_tas_later_intro_sequence_6_revisit_keeps_title_pose_hidden_through_frame_28398`](/home/connor/git/kc/crates/kid-core/src/tests.rs), locking the oracle-backed window where the later revisit keeps `kid_x_pos` / `kid_y_pos` at zero through frames 28396..28398 and only restores `x=$2E`, `y=$8C` on frame 28399.
- Re-ran `cargo test -q -p kid-core published_tas_later_intro_sequence_6_revisit_keeps_title_pose_hidden_through_frame_28398 -- --nocapture`, `cargo test -q -p kid-core published_tas_later_intro_sequence_6_revisit_starts_delayed_camera_ramp_on_frame_28317 -- --nocapture`, `cargo test -q -p kid-core published_tas_intro_sequence_6_exposes_title_pose_on_frame_21224 -- --nocapture`, `cargo run -q -p kidcli -- tas-diff --scenario .cache/tas-imports/any-percent-published/scenario.json --frame-start 28390 --frame-end 28402 --include-advisory`, `cargo run -q -p kidcli -- verify-tas --scenario .cache/tas-imports/any-percent-published/scenario.json --frame-start 23250 --bisect`, `cargo run -q -p kidcli -- tas-diff --scenario .cache/tas-imports/any-percent-published/scenario.json --frame-start 29396 --frame-end 29408 --include-advisory`, and `mise run verify-slice-fast`.

## Current Model

- ASM [`loc_1B5CE`](/home/connor/git/kc/.cache/upstream/kid-chameleon-disasm/kid.asm) still owns the standard title-pose write on both lineages, but the later non-wrap revisit does not surface that pose until three local mode-`$28` frames after the earlier timeout-restart copy.
- [`trace.rs`](/home/connor/git/kc/crates/kid-core/src/trace.rs) now models the later `IntroSequence6` title-pose threshold separately from the wrapped timeout-restart path, so frames 28390..28401 are advisory-only.
- The next frontier is the later `Game_Mode` hand-off: native reaches `0x0010` on frame 29402 while oracle remains in mode `0x0028` through frame 29404, so the remaining bug is in the `loc_1BAEA` / `loc_1BB02` idle-to-demo transition rather than camera state or Kid visibility.

## Strongest Evidence

- Native
  - [`trace.rs`](/home/connor/git/kc/crates/kid-core/src/trace.rs)
    - `current_player_state()` now keeps the non-wrap later revisit hidden until local mode-`$28` frame 333 instead of reusing the earlier timeout-restart frame-330 title-pose threshold.
  - [`tests.rs`](/home/connor/git/kc/crates/kid-core/src/tests.rs)
    - `published_tas_later_intro_sequence_6_revisit_keeps_title_pose_hidden_through_frame_28398` passes and proves the new 3-frame hide on the later revisit.
- Oracle / parity
  - `cargo run -q -p kidcli -- tas-diff --scenario .cache/tas-imports/any-percent-published/scenario.json --frame-start 28390 --frame-end 28402 --include-advisory`
    - Frames 28390..28401 are now advisory-only; the old `kid_x_pos` / `kid_y_pos` miss is gone.
  - `cargo run -q -p kidcli -- verify-tas --scenario .cache/tas-imports/any-percent-published/scenario.json --frame-start 23250 --bisect`
    - The first CRC divergence moved to frame 29402.
  - `cargo run -q -p kidcli -- tas-diff --scenario .cache/tas-imports/any-percent-published/scenario.json --frame-start 29396 --frame-end 29408 --include-advisory`
    - Frames 29402..29404 now show the next authoritative miss: native `game_mode=0x0010`; oracle keeps `game_mode=0x0028`.

## Exact Next Moves

1. Read the later mode-`$28` `Obj_TitleMenu` hand-off around `loc_1BAEA` / `loc_1BB02` in [`kid.asm`](/home/connor/git/kc/.cache/upstream/kid-chameleon-disasm/kid.asm) before touching native code again.
2. Add a focused regression for frames 29402..29404 proving oracle keeps `Game_Mode=$28` while native currently reaches `0x0010` too early.
3. Re-run `cargo run -q -p kidcli -- tas-diff --scenario .cache/tas-imports/any-percent-published/scenario.json --frame-start 29396 --frame-end 29408 --include-advisory` after the next patch so the three-frame mode-`$28` settle is what moves.

## Verification To Re-run

- `cargo test -q -p kid-core published_tas_later_intro_sequence_6_revisit_keeps_title_pose_hidden_through_frame_28398 -- --nocapture`
- `cargo test -q -p kid-core published_tas_later_intro_sequence_6_revisit_starts_delayed_camera_ramp_on_frame_28317 -- --nocapture`
- `cargo test -q -p kid-core published_tas_intro_sequence_6_exposes_title_pose_on_frame_21224 -- --nocapture`
- `mise run verify-slice-fast`
- `cargo run -q -p kidcli -- verify-tas --scenario .cache/tas-imports/any-percent-published/scenario.json --frame-start 23250 --bisect`
- `cargo run -q -p kidcli -- tas-diff --scenario .cache/tas-imports/any-percent-published/scenario.json --frame-start 28390 --frame-end 28402 --include-advisory`
- `cargo run -q -p kidcli -- tas-diff --scenario .cache/tas-imports/any-percent-published/scenario.json --frame-start 29396 --frame-end 29408 --include-advisory`

## Dead Ends / Do Not Retry

- Do not fold the later title-pose threshold back into the earlier wrapped timeout-restart constant. Frame 28396 proves the later revisit needs its own 3-frame hide.
- Do not keep debugging camera state or Kid visibility for this frontier. `tas-diff` now shows the next blocker is the `game_mode` flip (`0x0028 -> 0x0010`), not `camera_y_pos` or `kid_x_pos`.
