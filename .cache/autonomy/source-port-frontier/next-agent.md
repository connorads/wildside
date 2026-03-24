---
slug: any-percent-published-frame-17404-title-demo-preload-camera-row
cycle: 63
updated_at: 2026-03-24T06:19:14Z
---

# Source-Port Frontier Handoff

## What Changed

- Fixed the later Start-skip intro-return title fencepost so native now stays in `Game_Mode=$2C` through frame 17346 and only reaches demo play on 17347.
- Added `published_tas_intro_return_title_waits_until_frame_17347_to_start_demo_play`.
- Kept the earlier timeout-restart direct-return demo timing intact: `published_tas_second_timeout_restart_starts_demo_play_on_frame_10986` still passes.

## Current Model

- ASM `sub_1C246` still returns to title after the shared 2-frame Start deferral, but the later Start-skip intro-return lineage does not reach Obj_TitleMenu `loc_1BAEA` with the same amount of pre-wait credit as the plain timeout-restart path.
- Oracle-backed frame checks now match the later lineage keeping `Game_Mode=$2C` on frame 17346 and only letting Obj_TitleMenu `loc_1BB02` flip to `Game_Mode=$10` on 17347.
- The next first authoritative break is frame 17404.
- That new break is no longer a mode fencepost. Both sides are already in demo play, but native keeps `camera_y_pos=0x0220_0000` while oracle reports `0x0228_0000` for one frame on the hidden demo bootstrap preload row.

## Strongest Evidence

- ASM
  - `.cache/upstream/kid-chameleon-disasm/kid.asm`
    - `Obj_TitleMenu loc_1BAEA` reloads `move.w #$280,d0`, hibernates one frame per loop, and only `loc_1BB02` writes `move.w #$10,(Game_Mode).w`.
    - `sub_1C246` still performs the shared Start-driven title return to `Game_Mode=$2C`.
- Native
  - `crates/kid-core/src/boot.rs`
    - `finish_timeout_restart_intro_sequence_5_title_return()` now shortens `title_menu_arm_frame` by 1 frame for the later Start-skip intro-return lineage and by 2 frames for the plain timeout-restart path.
  - `crates/kid-core/src/tests.rs`
    - `published_tas_intro_return_title_waits_until_frame_17347_to_start_demo_play` locks the corrected title-idle/demo-start fencepost.
- Oracle / parity
  - `cargo run -q -p kidcli -- tas-diff --scenario .cache/tas-imports/any-percent-published/scenario.json --frame-start 17342 --frame-end 17350 --include-advisory`
    - frames 17342..17349 all match after the fix.
  - `cargo run -q -p kidcli -- verify-tas --scenario .cache/tas-imports/any-percent-published/scenario.json --frame-start 16650 --bisect`
    - first CRC divergence moved to frame `17404`.
  - `cargo run -q -p kidcli -- tas-diff --scenario .cache/tas-imports/any-percent-published/scenario.json --frame-start 17400 --frame-end 17408 --include-advisory`
    - frame 17404 diverges only on `camera_y_pos` (`native=0x02200000`, `oracle=0x02280000`).

## Exact Next Moves

1. Read `Obj_TitleMenu loc_1BB02`, `Load_DemoPlay`, and the `Load_InGame` camera preload path in `.cache/upstream/kid-chameleon-disasm/kid.asm` before touching native code again.
2. Explain why the later Start-skip intro-return lineage keeps the hidden demo bootstrap camera at `0x0228_0000` on frame 17404 while native reuses the plain timeout-restart preload row `0x0220_0000`.
3. Add one focused regression for frame 17404 only after ASM + the narrow diff agree on the exact preload-camera write path.

## Verification To Re-run

- `cargo test -q -p kid-core published_tas_intro_return_title_waits_until_frame_17347_to_start_demo_play -- --nocapture`
- `cargo test -q -p kid-core published_tas_intro_return_title_keeps_kid_hidden_through_frame_16798 -- --nocapture`
- `cargo test -q -p kid-core published_tas_second_timeout_restart_starts_demo_play_on_frame_10986 -- --nocapture`
- `cargo test -q -p kid-core published_tas_intro_sequence_5_waits_for_the_frame_16648_start_press -- --nocapture`
- `cargo run -q -p kidcli -- tas-diff --scenario .cache/tas-imports/any-percent-published/scenario.json --frame-start 17342 --frame-end 17350 --include-advisory`
- `cargo run -q -p kidcli -- tas-diff --scenario .cache/tas-imports/any-percent-published/scenario.json --frame-start 17400 --frame-end 17408 --include-advisory`
- `cargo run -q -p kidcli -- verify-tas --scenario .cache/tas-imports/any-percent-published/scenario.json --frame-start 16650 --bisect`
- `mise run verify-slice-fast`

## Dead Ends / Do Not Retry

- Do not shift `TIMEOUT_RESTART_TITLE_INTRO_SEQUENCE_5_HIDDEN_FRAMES` globally. The plain timeout-restart path already matches frame 10438.
- Do not reopen the frame-16798 hidden-pose gate. That frontier is fixed; the authoritative break moved beyond it.
- Do not change the plain timeout-restart direct-return title-arm shortening that already matches frame 10986.
- Do not assume frame 17404 is another title-idle fencepost. The narrow diff now shows only `camera_y_pos`.
