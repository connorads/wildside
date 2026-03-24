---
slug: any-percent-published-frame-20894-intro-sequence-5-mode-28-gap
cycle: 65
updated_at: 2026-03-24T06:57:19Z
---

# Source-Port Frontier Handoff

## What Changed

- Fixed the frame-20835 frontier by extending the shared timeout-restart intro-sequence-5 dwell in [`crates/kid-core/src/constants.rs`](/home/connor/git/kc/crates/kid-core/src/constants.rs).
- Added [`published_tas_intro_return_timeout_revisit_stays_in_intro_sequence_5_on_frame_20835`](/home/connor/git/kc/crates/kid-core/src/tests.rs), which locks the oracle-backed `Game_Mode=$24` + loc_1BCD0 slot-2 pose on frame 20835.
- Verified the earlier later-lineage guards still pass: frame 16648 Start deferral and frame 17404 demo-2 preload row were unaffected.

## Current Model

- ASM [`Load_IntroSequence5`](.cache/upstream/kid-chameleon-disasm/kid.asm) still seeds both [`loc_1BA28`](.cache/upstream/kid-chameleon-disasm/kid.asm) and [`loc_1BCD0`](.cache/upstream/kid-chameleon-disasm/kid.asm).
- [`loc_1BA28`](.cache/upstream/kid-chameleon-disasm/kid.asm) hands intro sequence 5 to the shared fade helper [`sub_1C204`](.cache/upstream/kid-chameleon-disasm/kid.asm), then [`loc_1BA5C`](.cache/upstream/kid-chameleon-disasm/kid.asm) owns the long `$24` dwell before the natural `move.w #$28,(Game_Mode).w` hand-off.
- The old native `17 + 0x1BD` approximation expired that shared timeout path too early; oracle keeps this later timeout-restart revisit in `Game_Mode=$24` through frame 20893.
- The next exposed bug is not another `$24` off-by-one. Oracle reaches the missing intermediate `Game_Mode=$28` on frame 20894 and stays there through at least frame 21250, while native still has no `$28` representation and remains in `TitleScreen` (`0x002C`).

## Strongest Evidence

- ASM
  - `.cache/upstream/kid-chameleon-disasm/kid.asm`
    - `loc_1BA28` writes `$44(a5)=loc_1BA5C` and calls `sub_1C204`.
    - `loc_1BA5C` is the natural intro-sequence-5 timeout owner and ends with `move.w #$28,(Game_Mode).w`.
    - `loc_1BCD0` keeps running during the same `$24` window and later spawns the `loc_1BE3C` / `loc_1BE88` title-piece workers.
- Native
  - [`crates/kid-core/src/constants.rs`](/home/connor/git/kc/crates/kid-core/src/constants.rs)
    - `DEMO_AUTO_TIMEOUT_INTRO_FIFTH_SEGMENT_FRAMES` now carries the shared timeout path 59 frames farther, matching the oracle-backed `$24` dwell through frame 20893.
  - [`crates/kid-core/src/tests.rs`](/home/connor/git/kc/crates/kid-core/src/tests.rs)
    - `published_tas_intro_return_timeout_revisit_stays_in_intro_sequence_5_on_frame_20835` passes.
- Oracle / parity
  - `cargo run -q -p kidcli -- oracle-probe --scenario .cache/tas-imports/any-percent-published/scenario.json --frame-start 20835 --frame-end 21020 --watches 0xFFFFFBCA:u16 | awk -F, '$1 ~ /^[0-9]+$/ && $1 >= 20835 && $2 != "0x24" { print; exit }'`
    - First non-`$24` row is `20894,0x28`.
  - `cargo run -q -p kidcli -- oracle-probe --scenario .cache/tas-imports/any-percent-published/scenario.json --frame-start 20894 --frame-end 21250 --watches 0xFFFFFBCA:u16 | awk -F, '$1 ~ /^[0-9]+$/ && $1 >= 20894 && $2 != "0x28" { print; exit } END { if (NR) print "done" }'`
    - No non-`$28` row through frame 21250.
  - `cargo run -q -p kidcli -- tas-diff --scenario .cache/tas-imports/any-percent-published/scenario.json --frame-start 20880 --frame-end 20940 --include-advisory`
    - Oracle stays `$24` through 20893 with the slot-2 pose at `kid_y_pos=$00E00000`, then flips to `$28` on 20894 while native is still `$2C`.

## Exact Next Moves

1. Implement an explicit native `$28` phase for the timeout-restart intro-sequence-5 natural return instead of collapsing straight from `$24` to `TitleScreen`.
2. Read [`loc_1BA5C`](.cache/upstream/kid-chameleon-disasm/kid.asm) and the following fade loop again, then map its `$28` behaviour to native trace fields around frames 20894, 20912, and 20926.
3. Add one focused regression for frame 20894 only after the native `$28` phase matches the oracle-backed mode/pose hand-off.

## Verification To Re-run

- `cargo test -q -p kid-core published_tas_intro_return_timeout_revisit_stays_in_intro_sequence_5_on_frame_20835 -- --nocapture`
- `cargo test -q -p kid-core published_tas_intro_sequence_5_waits_for_the_frame_16648_start_press -- --nocapture`
- `cargo test -q -p kid-core published_tas_intro_return_demo_2_uses_the_oracle_preload_row_on_frame_17404 -- --nocapture`
- `cargo run -q -p kidcli -- oracle-probe --scenario .cache/tas-imports/any-percent-published/scenario.json --frame-start 20835 --frame-end 21020 --watches 0xFFFFFBCA:u16 | awk -F, '$1 ~ /^[0-9]+$/ && $1 >= 20835 && $2 != "0x24" { print; exit }'`
- `cargo run -q -p kidcli -- oracle-probe --scenario .cache/tas-imports/any-percent-published/scenario.json --frame-start 20894 --frame-end 21250 --watches 0xFFFFFBCA:u16 | awk -F, '$1 ~ /^[0-9]+$/ && $1 >= 20894 && $2 != "0x28" { print; exit } END { if (NR) print "done" }'`
- `mise run verify-slice-fast`

## Dead Ends / Do Not Retry

- Do not keep pushing `DEMO_AUTO_TIMEOUT_INTRO_FIFTH_SEGMENT_START_SKIP_LINEAGE_EXTRA_FRAMES` for frame 20835. That lineage-specific knob is not what the 20835 natural-timeout revisit is using.
- Do not treat 20835 as another title-pose visibility bug. The real 20835 mismatch was the shared `$24` timeout expiring too early.
- Do not stop at “extend `$24` more” once 20835 is clean. Oracle explicitly enters `Game_Mode=$28` on 20894, and native currently has no such phase.
