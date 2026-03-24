---
slug: any-percent-published-frame-16633-intro-sequence-5-title-return-too-early
cycle: 60
updated_at: 2026-03-24T05:31:08Z
---

# Source-Port Frontier Handoff

## What Changed

- Fixed the frame-16228 `kid_y_pos` frontier without regressing the older frame-10007 intro-sequence-5 window.
- Read the owning ASM: `sub_1B988` flips to `Game_Mode=$24`, `Load_IntroSequence5` allocates `loc_1BA28` and `loc_1BCD0`, and `loc_1BCD0` writes the slot-2 pose with `move.w #0,$1A(a1)` / `move.w #$E0,$1E(a1)`.
- Added `published_tas_mode_24_exposes_loc_1bcd0_slot_2_on_frame_16228`.
- Preserved Start-skip intro-return lineage through the later demo auto-timeout path so only that lineage exposes `loc_1BCD0`'s slot-2 pose one frame earlier; the normal timeout-restart path still waits until frame 10007.

## Current Model

- The frame-16228 frontier is now authoritative-clean; `tas-diff --frame-start 16224 --frame-end 16232 --include-advisory` shows only advisory `number_objects` drift on frames 16227..16231.
- The blunt global fix was wrong: shifting the shared intro-sequence-5 slot-2 threshold to frame 57 made frame 10006 visible too early. The correct model is lineage-specific, not a universal timer change.
- The new first authoritative mismatch is frame `16633`.
- Current evidence says native exits intro sequence 5 into title too early on the same Start-skip intro-return lineage:
  - Native frames 16633..16636: `game_mode=0x002c`, `kid_y_pos=0x00000000`
  - Oracle frames 16633..16636: `game_mode=0x0024`, `kid_y_pos=0x00e00000`
- Frames 16629..16632 are authoritative-clean apart from advisory `number_objects=3`.

## Strongest Evidence

- ASM
  - `.cache/upstream/kid-chameleon-disasm/kid.asm`
    - `sub_1B988`: `move.w #$24,(Game_Mode).w` is the intro-sequence-4 -> intro-sequence-5 hand-off.
    - `Load_IntroSequence5`: allocates `loc_1BA28` and `loc_1BCD0`.
    - `loc_1BCD0`: writes the slot-2 pose with `move.w #0,$1A(a1)` / `move.w #$E0,$1E(a1)`.
    - `loc_1BA5C` polls `sub_1C246` during intro sequence 5, so the next frontier likely sits in that Start-driven title-return path.
- Native
  - `crates/kid-core/src/lib.rs`
    - carries a new Start-skip intro-return lineage bit through the later demo auto-timeout return.
  - `crates/kid-core/src/boot.rs`
    - preserves that lineage across `finish_demo_auto_timeout_return()`.
  - `crates/kid-core/src/trace.rs`
    - uses the lineage bit to expose `loc_1BCD0` slot 2 one frame earlier only on the intro-return-derived timeout-restart visit.
  - `crates/kid-core/src/tests.rs`
    - `published_tas_mode_24_exposes_loc_1bcd0_slot_2_on_frame_16228` locks the fixed frame down.
- Verification
  - `cargo test -q -p kid-core published_tas_mode_24_exposes_loc_1bcd0_slot_2_on_frame_16228 -- --nocapture`
  - `cargo test -q -p kid-core published_tas_second_timeout_restart_exposes_intro_sequence_5_slot_2_on_frame_10007 -- --nocapture`
  - `cargo test -q -p kid-core published_tas_second_timeout_restart_keeps_intro_sequence_5_alive_past_frame_10029 -- --nocapture`
  - `cargo test -q -p kid-core published_tas_intro_return_demo_1_uses_the_oracle_hidden_camera_wrap_on_frame_13188 -- --nocapture`
  - `cargo test -q -p kid-core demo_1_bootstrap_keeps_camera_hidden_through_frame_2537 -- --nocapture`
  - `cargo test -q -p kid-core published_tas_intro_sequence_1_start_skip_keeps_kid_hidden_on_frame_12430 -- --nocapture`
  - `cargo run -q -p kidcli -- tas-diff --scenario .cache/tas-imports/any-percent-published/scenario.json --frame-start 16224 --frame-end 16232 --include-advisory`
    - frame 16228 is now advisory-only
  - `cargo run -q -p kidcli -- verify-tas --scenario .cache/tas-imports/any-percent-published/scenario.json --frame-start 13189 --bisect`
    - first CRC divergence moved to frame `16633`
  - `cargo run -q -p kidcli -- tas-diff --scenario .cache/tas-imports/any-percent-published/scenario.json --frame-start 16629 --frame-end 16637 --include-advisory`
    - frames 16633..16636 diverge on `game_mode` (`0x002c` vs `0x0024`) and `kid_y_pos` (`0` vs `0x00e00000`)
  - `mise run verify-slice-fast`

## Exact Next Moves

1. Read `loc_1BA5C` and `sub_1C246` around the frame-16633 frontier. Prove why oracle stays in `Game_Mode=$24` with the `loc_1BCD0` slot-2 pose through frames 16633..16636.
2. Explain which native branch reaches `finish_timeout_restart_intro_sequence_5_title_return()` too early on the Start-skip intro-return lineage.
3. Add a focused regression for the frame-16633..16636 window, then fix the intro-sequence-5 -> title hand-off without reopening the cleared frame-16228 slot-visibility frontier.

## Verification To Re-run

- `cargo test -q -p kid-core published_tas_mode_24_exposes_loc_1bcd0_slot_2_on_frame_16228 -- --nocapture`
- `cargo test -q -p kid-core published_tas_second_timeout_restart_exposes_intro_sequence_5_slot_2_on_frame_10007 -- --nocapture`
- `cargo test -q -p kid-core published_tas_second_timeout_restart_keeps_intro_sequence_5_alive_past_frame_10029 -- --nocapture`
- `cargo test -q -p kid-core published_tas_intro_return_demo_1_uses_the_oracle_hidden_camera_wrap_on_frame_13188 -- --nocapture`
- `cargo test -q -p kid-core demo_1_bootstrap_keeps_camera_hidden_through_frame_2537 -- --nocapture`
- `cargo test -q -p kid-core published_tas_intro_sequence_1_start_skip_keeps_kid_hidden_on_frame_12430 -- --nocapture`
- `cargo run -q -p kidcli -- tas-diff --scenario .cache/tas-imports/any-percent-published/scenario.json --frame-start 16224 --frame-end 16232 --include-advisory`
- `cargo run -q -p kidcli -- verify-tas --scenario .cache/tas-imports/any-percent-published/scenario.json --frame-start 13189 --bisect`
- `cargo run -q -p kidcli -- tas-diff --scenario .cache/tas-imports/any-percent-published/scenario.json --frame-start 16629 --frame-end 16637 --include-advisory`
- `mise run verify-slice-fast`

## Dead Ends / Do Not Retry

- Do not change `TIMEOUT_RESTART_INTRO_SEQUENCE_5_SLOT_2_VISIBLE_ELAPSED_FRAMES` globally. Frame 10006 immediately regresses if the shared threshold moves from 58 to 57.
- Do not reopen frame 16228 as a generic mode-`$24` timer bug. The fixed behaviour is lineage-specific: only the Start-skip intro-return-derived timeout-restart visit exposes slot 2 one frame earlier.
- Do not treat frame 16633 as another slot-visibility threshold bug. The next frontier is a premature `Game_Mode=$2C` title return while oracle is still in intro sequence 5.
