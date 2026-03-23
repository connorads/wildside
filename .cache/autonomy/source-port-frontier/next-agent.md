---
slug: any-percent-published-frame-7969-knights-isle-native-stays-grounded-while-oracle-lifts-off
cycle: 27
updated_at: 2026-03-23T01:46:34Z
---

# Source-Port Frontier Handoff

## What Changed

- Ported the ROM walking-path `sub_922C` gate into `terrain::probe_wall_column(...)`: it scans one edge column at `half_width + 1` through the full body height instead of using the generic three-point wall probe.
- Wired that helper into Iron Knight attach only for grounded entry. Leaving it on every attach path moved the first CRC divergence earlier to frame 7906, so the airborne path stays on the old probe for now.
- Updated the grounded Iron Knight unit test to require the ROM edge-column reach, and rewrote the published-TAS attach regression to assert the new frame-7968 attach firing instead of the old broken “stays grounded” state.
- Re-ran `cargo test -p kid-core iron_knight_ -- --nocapture`, `cargo run -p kidcli -- verify-tas --scenario .cache/tas-imports/any-percent-published/scenario.json --frame-start 6975 --frame-end 9530 --bisect`, `cargo run -p kidcli -- tas-diff --scenario .cache/tas-imports/any-percent-published/scenario.json --frame-start 7967 --frame-end 7970 --include-physics-trace --include-advisory`, and `mise run verify-slice-fast`.

## Current Model

- The old grounded “attach never fires” blocker is gone. On frame 7968, native now consumes the raw C edge, enters Iron Knight climb, and seeds `y_vel = 0xFFFFC000`, matching `loc_A276`.
- The live frontier moved one step earlier to the same frame: frame 7968 now diverges because native keeps running normal airborne physics immediately after the attach. That advances Kid to `x=0x03A82B00`, `y=0x01BEC000`, while oracle Kid gfx stays at `x=0x03A80000`, `y=0x01BF0000` on that frame with the lift-off velocity merely seeded.
- The remaining bug is therefore no longer the wall gate. It is the post-attach same-frame control flow: native seeds the climb correctly, but does not mirror ROM `loc_A276`'s one-frame hibernate / movement suppression before the next gameplay step.

## Strongest Evidence

- `cargo run -p kidcli -- verify-tas --scenario .cache/tas-imports/any-percent-published/scenario.json --frame-start 6975 --frame-end 9530 --bisect` now reports the first CRC divergence at frame 7968, not 7969 and not the earlier accidental 7906.
- `cargo run -p kidcli -- tas-diff --scenario .cache/tas-imports/any-percent-published/scenario.json --frame-start 7967 --frame-end 7970 --include-physics-trace --include-advisory` shows frame 7968 native `kid_iron_knight_attach_probe = c_press=1/ground=1/climbing=0=>1/.../y_vel=0x00000000->0xffffc000/.../candidate=1/attach=1`.
- The same frame-7968 diff shows native physics still advances immediately after attach: `post_h ... xv=0x00002b00 step=true`, `post_v ... y=0x01bec000`, final `(0x03a82b00,0x01bec000)`.
- Oracle frame 7968 keeps Kid gfx at `0x30(kid_gfx)@0x03a80000,0x01bf0000 vel=0x00000000,0xffffc000`, so the attach velocity is live there but the position has not advanced yet.
- First-attempt dead end: applying `probe_wall_column(...)` to airborne attach too made frame 7906 fire `kid_iron_knight_attach_probe = ... ground=0/climbing=0=>1/.../attach=1` during an existing wall-cling ascent, moving the CRC frontier earlier than the target window.

## Exact Next Moves

1. Read the `loc_A276` follow-on control flow in `kid.asm` again and trace exactly why the grounded attach frame hibernates instead of falling through native-style normal physics.
2. Adjust the grounded Iron Knight attach path so frame 7968 seeds climb state and `y_vel = 0xFFFFC000` without letting normal physics advance `x_pos` or `y_pos` on that same frame.
3. Re-run `cargo run -p kidcli -- tas-diff --scenario .cache/tas-imports/any-percent-published/scenario.json --frame-start 7967 --frame-end 7970 --include-physics-trace --include-advisory`. The target is frame 7968 native `attach=1` while staying at `x=0x03A80000`, `y=0x01BF0000`.
4. Re-run `cargo run -p kidcli -- verify-tas --scenario .cache/tas-imports/any-percent-published/scenario.json --frame-start 6975 --frame-end 9530 --bisect`. The target is pushing the first CRC divergence later than frame 7968.

## Verification To Re-run

- `cargo test -p kid-core iron_knight_ -- --nocapture`
- `cargo run -p kidcli -- verify-tas --scenario .cache/tas-imports/any-percent-published/scenario.json --frame-start 6975 --frame-end 9530 --bisect`
- `cargo run -p kidcli -- tas-diff --scenario .cache/tas-imports/any-percent-published/scenario.json --frame-start 7967 --frame-end 7970 --include-physics-trace --include-advisory`
- `mise run verify-slice-fast`

## Dead Ends / Do Not Retry

- Do not reapply `probe_wall_column(...)` to every Iron Knight attach path. That broad patch moved the first CRC divergence earlier to frame 7906 during an airborne wall-cling window.
- Do not go back to the generic “use form half-width 7” wall probe swap. That older dead end still over-attaches and is weaker than the grounded `sub_922C` port.
- Do not keep treating frame 7968 as a missed wall gate. The gate now fires; the remaining bug is the same-frame motion after `loc_A276` entry.
