---
slug: any-percent-published-frame-20835-title-vs-intro-sequence-5-return
cycle: 64
updated_at: 2026-03-24T06:36:08Z
---

# Source-Port Frontier Handoff

## What Changed

- Fixed the later Start-skip intro-return demo-2 preload row so frame 17404 now matches oracle.
- Added `published_tas_intro_return_demo_2_uses_the_oracle_preload_row_on_frame_17404`.
- Kept the plain timeout-restart demo-2 preload row intact: `published_tas_second_timeout_restart_uses_oracle_demo_camera_preload_on_frame_11043` still passes.

## Current Model

- ASM `Obj_TitleMenu loc_1BB02` still flips into `Load_DemoPlay -> Load_InGame`; the frame-17404 miss was trace parity, not a gameplay-state bug.
- Oracle WRAM keeps the later timeout-restart intro-return lineage on a one-frame `Camera_Y_pos=$0228_0000` preload row before `Load_InGame`'s `$02A0_0000` snap; native was wrongly reusing the plain timeout-restart `$0220_0000` row.
- Native now matches through frame 17404. The next first authoritative break is frame 20835.
- That new break is a mode/pose return bug: native stays in title (`Game_Mode=$2C`, `kid_y_pos=0`) while oracle is already back in intro sequence 5 (`Game_Mode=$24`, `kid_y_pos=$00E00000`).

## Strongest Evidence

- ASM
  - `.cache/upstream/kid-chameleon-disasm/kid.asm`
    - `Obj_TitleMenu loc_1BB02` is still the only title idle exit writing `move.w #$10,(Game_Mode).w`.
    - `Load_DemoPlay` immediately calls `Load_InGame`, whose `move.l d0,(Camera_Y_pos).w` snap explains the frame-17405 `$02A0_0000`.
- Native
  - `crates/kid-core/src/trace.rs`
    - The demo-2 preload branch now keys off `timeout_restart_title_intro_sequence_5_uses_intro_return_hidden_delay`, which is the surviving lineage bit for this later Start-skip intro-return path.
  - `crates/kid-core/src/tests.rs`
    - `published_tas_intro_return_demo_2_uses_the_oracle_preload_row_on_frame_17404` locks the one-frame `$0228_0000` preload row.
- Oracle / parity
  - `cargo run -q -p kidcli -- oracle-probe --scenario .cache/tas-imports/any-percent-published/scenario.json --frame-start 17403 --frame-end 17405 --watches 0xFFFFF820:u32,0xFFFFFBCA:u16,0xFFFFFC44:u16,0xFFFFFC2C:u16,0xFFFFFBC9:u8`
    - frame 17404 has `Camera_Y_pos=0x02280000`; frame 17405 has `0x02A00000`.
  - `cargo run -q -p kidcli -- tas-diff --scenario .cache/tas-imports/any-percent-published/scenario.json --frame-start 17400 --frame-end 17408 --include-advisory`
    - frames 17400..17407 all match after the fix.
  - `cargo run -q -p kidcli -- verify-tas --scenario .cache/tas-imports/any-percent-published/scenario.json --frame-start 16650 --bisect`
    - first CRC divergence moved to frame `20835`.
  - `cargo run -q -p kidcli -- tas-diff --scenario .cache/tas-imports/any-percent-published/scenario.json --frame-start 20831 --frame-end 20839 --include-advisory`
    - frames 20835..20838 diverge on `game_mode` (`native=0x002c`, `oracle=0x0024`) and `kid_y_pos` (`native=0x00000000`, `oracle=0x00e00000`).

## Exact Next Moves

1. Read `sub_1C246`, `loc_1BA5C`, `Load_IntroSequence5`, and `loc_1BCD0` in `.cache/upstream/kid-chameleon-disasm/kid.asm` before touching native code again.
2. Explain why oracle re-enters `Game_Mode=$24` with the `loc_1BCD0` slot-2 pose by frame 20835 while native stays in title mode for 4 extra frames.
3. Add one focused regression for frame 20835 only after ASM + the narrow diff agree on the exact return path.

## Verification To Re-run

- `cargo test -q -p kid-core published_tas_intro_return_demo_2_uses_the_oracle_preload_row_on_frame_17404 -- --nocapture`
- `cargo test -q -p kid-core published_tas_second_timeout_restart_uses_oracle_demo_camera_preload_on_frame_11043 -- --nocapture`
- `cargo test -q -p kid-core published_tas_intro_return_title_waits_until_frame_17347_to_start_demo_play -- --nocapture`
- `cargo run -q -p kidcli -- tas-diff --scenario .cache/tas-imports/any-percent-published/scenario.json --frame-start 17400 --frame-end 17408 --include-advisory`
- `cargo run -q -p kidcli -- verify-tas --scenario .cache/tas-imports/any-percent-published/scenario.json --frame-start 16650 --bisect`
- `cargo run -q -p kidcli -- tas-diff --scenario .cache/tas-imports/any-percent-published/scenario.json --frame-start 20831 --frame-end 20839 --include-advisory`
- `mise run verify-slice-fast`

## Dead Ends / Do Not Retry

- Do not change `DEMO_2_CAMERA_PRELOAD_Y_FP` globally. The plain timeout-restart path still needs `$02200000` on frame 11043.
- Do not thread the demo-2 preload row off `demo_uses_start_skip_intro_return_camera_wrap`; that flag does not cover the later timeout-restart intro-return lineage.
- Do not reopen frame 17404 as a gameplay-state bug. Oracle WRAM + ASM show this was a one-frame preload-row parity issue.
- Do not assume frame 20835 is another demo bootstrap miss. The narrow diff is back in title vs intro-sequence-5 mode/pose territory.
