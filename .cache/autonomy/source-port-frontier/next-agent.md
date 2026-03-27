---
slug: any-percent-published-frame-28066-mode-24-follow-on
cycle: 72
updated_at: 2026-03-27T04:35:32Z
---

# Source-Port Frontier Handoff

## What Changed

- Fixed the frame-27603 mode-0x24 pose miss by keeping the later IntroSequence6 timeout-restore lineage bit alive when [`begin_demo_play()`](/home/connor/git/kc/crates/kid-core/src/boot.rs) starts the next demo from title.
- Added [`published_tas_later_intro_sequence_5_revisit_exposes_loc_1bcd0_slot_2_on_frame_27603`](/home/connor/git/kc/crates/kid-core/src/tests.rs), locking the oracle-backed revisit where `loc_1BCD0`'s slot-2 pose reappears at frame 27603.
- Re-ran `cargo test -q -p kid-core published_tas_later_intro_sequence_5_revisit_exposes_loc_1bcd0_slot_2_on_frame_27603 -- --nocapture`, `cargo test -q -p kid-core published_tas_intro_sequence_6_timeout_restore_reenters_demo_play_on_frame_23248 -- --nocapture`, `cargo run -q -p kidcli -- verify-tas --scenario .cache/tas-imports/any-percent-published/scenario.json --frame-start 27598 --frame-end 27608 --checkpoint-interval 1`, `cargo run -q -p kidcli -- verify-tas --scenario .cache/tas-imports/any-percent-published/scenario.json --frame-start 23250 --bisect`, `cargo run -q -p kidcli -- tas-diff --scenario .cache/tas-imports/any-percent-published/scenario.json --frame-start 27598 --frame-end 27608 --include-advisory`, `cargo run -q -p kidcli -- oracle-probe --scenario .cache/tas-imports/any-percent-published/scenario.json --frame-start 27598 --frame-end 27608 --watches 0xFF24FA:u16,0xFF24FE:u16`, and `mise run verify-slice-fast`; the first CRC divergence now moves to frame 28066.

## Current Model

- ASM [`Load_IntroSequence5`](/home/connor/git/kc/.cache/upstream/kid-chameleon-disasm/kid.asm) still reaches [`loc_1BCD0`](/home/connor/git/kc/.cache/upstream/kid-chameleon-disasm/kid.asm), whose third gfx slot writes `x=0`, `y=$E0`.
- The frame-27603 miss was not a new gameplay bug. Native had dropped the existing later IntroSequence6 timeout-restore lineage bit when the next demo started from title, so the following timeout-restart revisit hid `loc_1BCD0`'s slot-2 pose one frame too long.
- `begin_demo_play()` now preserves that lineage through `intro_sequence_6_early_title_restore`, so the frame-27603 revisit matches oracle and the frontier moves to frame 28066, still in `mode=0x0024`, `level=0x00`.

## Strongest Evidence

- ASM
  - [`kid.asm`](/home/connor/git/kc/.cache/upstream/kid-chameleon-disasm/kid.asm)
    - `Load_IntroSequence5` allocates `loc_1BCD0`, and `loc_1BCD0`'s third gfx slot still writes `move.w #0,$1A(a1)` / `move.w #$E0,$1E(a1)`.
- Native
  - [`boot.rs`](/home/connor/git/kc/crates/kid-core/src/boot.rs)
    - `begin_demo_play()` now keeps `demo_timeout_restart_uses_intro_return_slot_2_early_visibility` alive for the later `intro_sequence_6_early_title_restore` lineage instead of only the earlier immediate Start-skip return.
  - [`tests.rs`](/home/connor/git/kc/crates/kid-core/src/tests.rs)
    - `published_tas_later_intro_sequence_5_revisit_exposes_loc_1bcd0_slot_2_on_frame_27603` passes.
- Oracle / parity
  - `cargo run -q -p kidcli -- oracle-probe --scenario .cache/tas-imports/any-percent-published/scenario.json --frame-start 27598 --frame-end 27608 --watches 0xFF24FA:u16,0xFF24FE:u16`
    - Oracle WRAM slot 2 is `(0,0)` through frame 27602, then `(0,$E0)` on frames 27603..27608.
  - `cargo run -q -p kidcli -- verify-tas --scenario .cache/tas-imports/any-percent-published/scenario.json --frame-start 27598 --frame-end 27608 --checkpoint-interval 1`
    - CRC-clean across the repaired mode-0x24 window.
  - `cargo run -q -p kidcli -- verify-tas --scenario .cache/tas-imports/any-percent-published/scenario.json --frame-start 23250 --bisect`
    - First CRC divergence is now frame 28066, with native `mode=0x0024`, `level=0x00`, `kid=(0x00000000,0x00e00000)`, `vel=(0,0)`.

## Exact Next Moves

1. Run `cargo run -q -p kidcli -- tas-diff --scenario .cache/tas-imports/any-percent-published/scenario.json --frame-start 28060 --frame-end 28072 --include-advisory` and name the exact authoritative field that first diverges at frame 28066.
2. Re-read the `mode=0x0024` ASM path around `Load_IntroSequence5`, `loc_1BA28`, `loc_1BA5C`, and `loc_1BCD0` to identify which follow-on state transition owns frame 28066.
3. Add the next focused regression only after ASM plus the narrow diff window prove what frame 28066 should do.

## Verification To Re-run

- `cargo test -q -p kid-core published_tas_later_intro_sequence_5_revisit_exposes_loc_1bcd0_slot_2_on_frame_27603 -- --nocapture`
- `cargo test -q -p kid-core published_tas_intro_sequence_6_timeout_restore_reenters_demo_play_on_frame_23248 -- --nocapture`
- `cargo run -q -p kidcli -- verify-tas --scenario .cache/tas-imports/any-percent-published/scenario.json --frame-start 23250 --bisect`
- `cargo run -q -p kidcli -- verify-tas --scenario .cache/tas-imports/any-percent-published/scenario.json --frame-start 27598 --frame-end 27608 --checkpoint-interval 1`
- `cargo run -q -p kidcli -- tas-diff --scenario .cache/tas-imports/any-percent-published/scenario.json --frame-start 27598 --frame-end 27608 --include-advisory`
- `cargo run -q -p kidcli -- tas-diff --scenario .cache/tas-imports/any-percent-published/scenario.json --frame-start 28060 --frame-end 28072 --include-advisory`
- `mise run verify-slice-fast`

## Dead Ends / Do Not Retry

- Do not retune level-0x15 movement helpers again. The rebisect stays in `mode=0x0024`, `level=0x00`; the live gameplay blocker is gone.
- Do not narrow the lineage carry to `BootState::IntroSequence6` alone. The later failing demo starts from title, so the surviving bit is the broader `intro_sequence_6_early_title_restore` lineage.
- Do not revisit the removed generic top-edge wall-cling fallback. That was the previous frontier and is already verified clean.
