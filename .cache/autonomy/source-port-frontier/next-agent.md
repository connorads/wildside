---
slug: any-percent-published-frame-12430-native-keeps-kid-after-oracle-zeroes-pose
cycle: 57
updated_at: 2026-03-24T04:26:00Z
---

# Source-Port Frontier Handoff

## What Changed

- Re-read the ROM labels behind the frame-12430 handoff note and corrected the first investigation target.
- `loc_1B920` is not a title/demo hand-off object. It is the `sub_1B8F4` intro-sequence-1 wait loop: hibernate for `#$12B+1` frames, then set `Game_Mode=$18`.
- The live `Game_Mode=$2C` hand-off labels are elsewhere: `loc_119D8` (demo Start-abort -> title), `sub_1C246` (intro-sequence-3/5 Start skip -> title), `Obj_TitleMenu loc_1BB02` (title -> demo/options/start), and the later `loc_1BCD0` / `Load_IntroSequence6` title rebuild path.
- No native code changed this cycle. This commit only removes the stale ROM lead from the handoff so the next agent starts on the right labels.

## Current Model

- The frame-11549 prize-block frontier remains solved; the first authoritative mismatch is still frame `12430`.
- The important correction is source identity: the handoff’s `0x01B920` target was wrong. That label belongs to intro sequence 1, not to the frame-12430 title/demo transition.
- Frame `12430` still looks like a title-side hidden-Kid sequencing problem:
  - Native frame 12430: `mode=0x002c level=00 kid=(0x002e0000,0x008c0000) vel=(0,0)`.
  - Oracle frame 12430: `kid_x_pos=0`, `kid_y_pos=0`, with only one `0x30` gfx object left at `(0x00810000,0x00c80000)`.
- Because `Game_Mode=$2C` is already active, the likely causes now sit in the title rebuild / visibility gates, not gameplay terrain:
  - ROM side: `loc_119D8`, `sub_1C246`, `Obj_TitleMenu loc_1BB02`, `loc_1BCD0`.
  - Native side: `BootState::TitleScreen`, `title_hides_demo_return_player()`, carried snapshot latches, and the `trace.rs` player-projection order.

## Strongest Evidence

- ASM
  - `.cache/upstream/kid-chameleon-disasm/kid.asm`
    - `sub_1B8F4` writes `loc_1B920` into `$44(a5)`.
    - `loc_1B920` loops for `#$12B+1` frames, then writes `Game_Mode=$18`.
    - `loc_119D8` writes `Game_Mode=$2C` when demo Start-abort wakes.
    - `sub_1C246` writes `Game_Mode=$2C` on intro-sequence Start skip.
    - `Obj_TitleMenu loc_1BB02` writes `Game_Mode=$10`.
    - `loc_1BCD0` is the title rebuild that allocates the carried slot-2 Kid/title gfx objects.
- Native model
  - `crates/kid-core/src/boot.rs`
    - `BootState::TitleScreen` owns the `0x002C` hand-off logic.
    - `finish_demo_title_return()`, `finish_demo_auto_timeout_return()`, `finish_timeout_restart_intro_sequence_3_title_return()`, and `finish_timeout_restart_intro_sequence_5_title_return()` set the title-side carry/hide latches.
  - `crates/kid-core/src/trace.rs`
    - `title_hides_demo_return_player()` decides whether Kid is still hidden while `Game_Mode=$2C` is already active.
    - `current_player_state()` prefers carried snapshot / carried intro poses before falling back to hidden or standard title pose.
- Existing parity evidence
  - `cargo run -q -p kidcli -- tas-diff --scenario .cache/tas-imports/any-percent-published/scenario.json --frame-start 11548 --frame-end 11551 --include-advisory --include-physics-trace`
    - frames 11548..11550 now show advisory-only drift
  - `cargo run -q -p kidcli -- verify-tas --scenario .cache/tas-imports/any-percent-published/scenario.json --frame-start 9531 --bisect`
    - first CRC divergence moved to frame `12430`
  - `cargo run -q -p kidcli -- tas-diff --scenario .cache/tas-imports/any-percent-published/scenario.json --frame-start 12426 --frame-end 12434 --include-advisory`
    - first authoritative diff is frame `12430`
    - native keeps Kid at `(0x002e0000,0x008c0000)` while oracle zeroes Kid pose

## Exact Next Moves

1. Do not read `loc_1B920` again for this frontier. Start from the real `Game_Mode=$2C` labels instead: `loc_119D8`, `sub_1C246`, `Obj_TitleMenu loc_1BB02`, and `loc_1BCD0`.
2. Explain which of the existing title-side native latches is wrong on frame 12430: preserved demo snapshot, hidden-Kid rebuild window, or carried intro/title pose. Compare the oracle frame against `title_hides_demo_return_player()` and the `current_player_state()` branch order in `crates/kid-core/src/trace.rs`.
3. Once the cause is proven, add a focused regression around frames 12429..12430 and fix the frontier.

## Verification To Re-run

- `cargo test -q -p kid-core published_tas_second_timeout_restart_keeps_the_oracle_backed_cyclone_flight_arc_on_frame_11549 -- --nocapture`
- `cargo run -q -p kidcli -- tas-diff --scenario .cache/tas-imports/any-percent-published/scenario.json --frame-start 11548 --frame-end 11551 --include-advisory --include-physics-trace`
- `cargo run -q -p kidcli -- verify-tas --scenario .cache/tas-imports/any-percent-published/scenario.json --frame-start 9531 --bisect`
- `cargo run -q -p kidcli -- tas-diff --scenario .cache/tas-imports/any-percent-published/scenario.json --frame-start 12426 --frame-end 12434 --include-advisory`
- `mise run verify-slice-fast`

## Dead Ends / Do Not Retry

- Do not reopen Cyclone Y-acceleration or `loc_B214..loc_B26C`; frame 11549 was a prize-block collision-visibility problem, not a vertical integrator bug.
- Do not revive the deleted “frame 11549 re-hits the restored prize smash block” diagnostic; it described the fixed native bug, not current truth.
- Do not trust `.cache/autonomy/source-port-frontier/next-task.json` for this frontier. It is still stale and points at the old frame-7969 Knights Isle work.
- Do not chase `loc_1B920` for frame 12430. That label is `sub_1B8F4`'s intro-sequence-1 wait loop, not the live title/demo hand-off.
