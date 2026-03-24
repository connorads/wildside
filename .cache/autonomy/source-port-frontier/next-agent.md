---
slug: any-percent-published-frame-11043-timeout-restart-demo-boot-camera-y-high
cycle: 52
updated_at: 2026-03-24T03:21:30Z
---

# Source-Port Frontier Handoff

## What Changed

- Re-read the timeout-restart title-shell path around `sub_1C246`, `Obj_TitleMenu`, `loc_1BAEA`, `Title_InputLoop`, and `loc_1BB02` in `.cache/upstream/kid-chameleon-disasm/kid.asm`.
- Fixed the timeout-restart intro-sequence-5 direct title return so native reuses the title shell's pre-menu attract wait with the ROM's 2-frame head start instead of restarting the full cold-boot lock.
- Added a focused regression proving the timeout-restart path now enters demo play on frame 10986 while keeping the preserved title pose visible.
- Re-ran the 10982..10994 oracle slice; it is now fully clean.
- Re-ran the CRC bisect from frame 9531; the first divergence moved from frame 10986 to frame 11043.

## Current Model

- The old frame-10986 frontier was genuine title-shell timing debt, but the handoff's "2-frame transient then back to title" model was too small. Oracle stays on `game_mode=0x0010` after frame 10986; clean native was simply entering demo play 2 frames late.
- `sub_1C246` returns straight to title after the shared 2-frame Start deferral, so the reused timeout-restart title shell is already 2 frames into `Obj_TitleMenu`'s pre-menu attract wait when title becomes visible.
- Shortening only `title_menu_arm_frame` by 2 in `finish_timeout_restart_intro_sequence_5_title_return()` makes native hit `loc_1BB02`'s demo-play write on the correct frame without disturbing the preserved title pose.
- The next live blocker is now a one-frame camera settle mismatch at frame 11043: native and oracle match on 11038..11042 and 11044..11047, but on 11043 native reports `camera_y_pos=0x02280000` while oracle reports `0x02200000`.

## Strongest Evidence

- `.cache/upstream/kid-chameleon-disasm/kid.asm`
  - `sub_1C246`: writes `move.w #$2C,(Game_Mode).w` on the direct intro-sequence-5 Start return without rebuilding the title shell.
  - `Obj_TitleMenu` / `loc_1BAEA`: keeps a separate pre-menu attract wait before demo playback.
  - `loc_1BB02`: writes `move.w #$10,(Game_Mode).w` when that attract wait expires.
- Native/oracle diagnostics
  - `cargo test -q -p kid-core published_tas_second_timeout_restart_starts_demo_play_on_frame_10986 -- --nocapture`
    - passes
  - `cargo test -q -p kid-core published_tas_second_timeout_restart_ -- --nocapture`
    - all 11 timeout-restart regressions pass
  - `cargo run -q -p kidcli -- tas-diff --scenario .cache/tas-imports/any-percent-published/scenario.json --frame-start 10982 --frame-end 10994 --include-advisory`
    - frames `10982..10993`: `MATCH`
  - `cargo run -q -p kidcli -- tas-diff --scenario .cache/tas-imports/any-percent-published/scenario.json --frame-start 11038 --frame-end 11048 --include-advisory`
    - frames `11038..11042`: `MATCH`
    - frame `11043`: only `camera_y_pos` diverges (`native=0x02280000`, `oracle=0x02200000`)
    - frames `11044..11047`: `MATCH`
  - `cargo run -q -p kidcli -- verify-tas --scenario .cache/tas-imports/any-percent-published/scenario.json --frame-start 9531 --bisect`
    - `RESULT: first CRC divergence at frame 11043`
  - `mise run verify-slice-fast`
    - passes

## Exact Next Moves

1. Read the demo-start / camera bootstrap ASM reached after `loc_1BB02`'s `Game_Mode=$10` write. Focus on the first hidden demo-start frames rather than reopening the title timer.
2. Add a focused regression for the new camera window: 11042 should match, 11043 should show the one-frame `camera_y_pos=0x02200000` oracle settle, and 11044 should return to the matched path.
3. Fix the timeout-restart demo bootstrap camera hand-off so frame 11043 matches, then re-run the bisect from frame 9531.

## Verification To Re-run

- `cargo test -q -p kid-core published_tas_second_timeout_restart_starts_demo_play_on_frame_10986 -- --nocapture`
- `cargo test -q -p kid-core published_tas_second_timeout_restart_ -- --nocapture`
- `cargo run -q -p kidcli -- tas-diff --scenario .cache/tas-imports/any-percent-published/scenario.json --frame-start 10982 --frame-end 10994 --include-advisory`
- `cargo run -q -p kidcli -- tas-diff --scenario .cache/tas-imports/any-percent-published/scenario.json --frame-start 11038 --frame-end 11048 --include-advisory`
- `cargo run -q -p kidcli -- verify-tas --scenario .cache/tas-imports/any-percent-published/scenario.json --frame-start 9531 --bisect`
- `mise run verify-slice-fast`

## Dead Ends / Do Not Retry

- Do not restart the timeout-restart intro-sequence-5 title shell from a fresh cold-boot attract wait. The ROM reuses the existing shell after `sub_1C246`, so native was simply 2 frames late entering demo play.
- Do not model frame 10986 as a trace-only `Game_Mode=$10` blip that settles back to title on 10988. The widened 10982..10994 slice shows oracle stays in demo play after 10986; the old "2-frame transient" theory was wrong.
- Do not run multiple heavyweight oracle-backed `tas-diff` / `verify-tas` jobs in parallel. They share mutable oracle/tooling state and can produce misleading diagnostics.
