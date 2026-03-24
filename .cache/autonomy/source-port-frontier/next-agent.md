---
slug: any-percent-published-frame-13188-demo-bootstrap-hidden-camera-y-high
cycle: 58
updated_at: 2026-03-24T04:41:48Z
---

# Source-Port Frontier Handoff

## What Changed

- Fixed the frame-12430 hidden-Kid frontier.
- The bug was not a demo-return snapshot issue. A targeted regression proved frame 12429 is still `IntroSequence1`, so the live path is `sub_1B8F4` / `loc_1B920` polling `sub_1C246`, not `loc_119D8`.
- Native was exposing the standard title pose immediately after `sub_1C246` switched to `Game_Mode=$2C`. The ROM still needs the later `Load_IntroSequence6` / `loc_1BCD0` title rebuild settle window before Kid appears.
- Added `enter_title_screen_from_intro_return()` so intro-sequence returns reuse the local title-ready delay instead of showing the standard title pose on the first title frame.
- Added `published_tas_intro_sequence_1_start_skip_keeps_kid_hidden_on_frame_12430`.

## Current Model

- The frame-12430 title return is now authoritative-clean; `tas-diff --frame-start 12426 --frame-end 12434 --include-advisory` shows only advisory `number_objects` drift.
- The new first authoritative mismatch is frame `13188`.
- Current evidence says this is a one-frame hidden demo-bootstrap camera mismatch, not another visible title-pose bug:
  - Native frame 13188: `mode=0x0010 level=15 kid=(0x00000000,0x00000000) camera_y=0xff000000`
  - Oracle frame 13188: `mode=0x0010 level=15 kid=(0x00000000,0x00000000) camera_y=0xff200000`
- Frames 13189..13191 rejoin immediately on authoritative state; only advisory `number_objects` drift remains.

## Strongest Evidence

- ASM
  - `.cache/upstream/kid-chameleon-disasm/kid.asm`
    - `sub_1B8F4` allocates the intro-sequence-1 text object and stores `loc_1B920` in `$44(a5)`.
    - `loc_1B920` hibernates and calls `sub_1C246` every frame before naturally writing `Game_Mode=$18`.
    - `sub_1C246` writes `Game_Mode=$2C` immediately on Start, then returns through the shared title path.
    - `Load_IntroSequence6` / `loc_1BCD0` are the later title rebuild path, so the first `Game_Mode=$2C` frame should not expose the standard title pose yet.
- Native
  - `crates/kid-core/src/boot.rs`
    - `enter_title_screen_from_intro_return()` now forces the local title-ready delay on intro-sequence returns.
    - `step_boot_state()` uses that helper for both deferred Start skips and natural intro completion returns.
  - `crates/kid-core/src/tests.rs`
    - `published_tas_intro_sequence_1_start_skip_keeps_kid_hidden_on_frame_12430` locks the fixed path down.
- Verification
  - `cargo run -q -p kidcli -- tas-diff --scenario .cache/tas-imports/any-percent-published/scenario.json --frame-start 12426 --frame-end 12434 --include-advisory`
    - frames 12426..12433 are advisory-only (`number_objects`)
  - `cargo run -q -p kidcli -- verify-tas --scenario .cache/tas-imports/any-percent-published/scenario.json --frame-start 9531 --bisect`
    - first CRC divergence moved to frame `13188`
  - `cargo run -q -p kidcli -- tas-diff --scenario .cache/tas-imports/any-percent-published/scenario.json --frame-start 13184 --frame-end 13192 --include-advisory`
    - only authoritative diff is `camera_y_pos` on frame `13188` (`native=0xff000000`, `oracle=0xff200000`)

## Exact Next Moves

1. Read the cold-boot demo-start camera/bootstrap ASM that runs after the intro-sequence title shell returns to `Game_Mode=$10`. Do not reopen the fixed 12430 title-hide path.
2. Explain why oracle uses hidden `camera_y_pos=0xff200000` on frame 13188 while native keeps `0xff000000` for one frame. This looks like another hidden-camera bootstrap constant/write-site issue.
3. Add a focused regression for frame 13188 once the exact oracle write path is proven, then fix the camera settle.

## Verification To Re-run

- `cargo test -q -p kid-core published_tas_intro_sequence_1_start_skip_keeps_kid_hidden_on_frame_12430 -- --nocapture`
- `cargo run -q -p kidcli -- tas-diff --scenario .cache/tas-imports/any-percent-published/scenario.json --frame-start 12426 --frame-end 12434 --include-advisory`
- `cargo run -q -p kidcli -- tas-diff --scenario .cache/tas-imports/any-percent-published/scenario.json --frame-start 13184 --frame-end 13192 --include-advisory`
- `cargo run -q -p kidcli -- verify-tas --scenario .cache/tas-imports/any-percent-published/scenario.json --frame-start 9531 --bisect`
- `mise run verify-slice-fast`

## Dead Ends / Do Not Retry

- Do not treat frame 12430 as `loc_119D8` demo abort. The regression proves frame 12429 is still `IntroSequence1`.
- Do not chase preserved demo snapshots or `title_hides_demo_return_player()` for frame 12430. The bug was a too-eager standard title-ready gate on the intro-return path.
- Do not reopen `loc_1B920` as if it were unrelated noise. It was the right intro-sequence-1 loop for this fix; the mistake was assuming it was a demo-return path.
- Do not go back to the stale frame-11043 frontier. Current bisect evidence lands the first authoritative break at frame 13188.
