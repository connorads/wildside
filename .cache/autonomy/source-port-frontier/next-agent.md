---
slug: any-percent-published-frame-7969-knights-isle-native-stays-grounded-while-oracle-lifts-off
cycle: 29
updated_at: 2026-03-23T02:08:15Z
---

# Source-Port Frontier Handoff

## What Changed

- Ported the ROM wall-climb resume lookup from `loc_A30A`/`sub_B55C`: native now checks the single solid cell at `half_width + 1, y - 24` instead of the old airborne `probe_wall(..., 5, 31, ...)` gate when Iron Knight is already climbing.
- Added `terrain::probe_iron_knight_climb_resume_wall(...)` and wired the existing Iron Knight attach diagnostics to report that same resume-frame lookup whenever climb is already active.
- Added `iron_knight_resume_climb_uses_rom_wall_lookup`, then updated the frame-7969 frontier tests to the new truth: climb state and custom gravity survive frame 7969, but the frame still diverges on horizontal movement and a slightly-too-fast upward step.
- Re-ran `cargo test -p kid-core 7968 -- --nocapture`, `cargo test -p kid-core 7969 -- --nocapture`, `cargo test -p kid-core iron_knight_ -- --nocapture`, `cargo run -p kidcli -- tas-diff --scenario .cache/tas-imports/any-percent-published/scenario.json --frame-start 7967 --frame-end 7970 --include-physics-trace --include-advisory`, `cargo run -p kidcli -- verify-tas --scenario .cache/tas-imports/any-percent-published/scenario.json --frame-start 6975 --frame-end 9530 --bisect`, and `mise run verify-slice-fast`.

## Current Model

- The old frame-7969 wall-retention failure is fixed. Native now keeps `iron_knight_climbing=1`, keeps custom gravity live, and preserves upward motion on the resume frame.
- The first authoritative divergence is still frame 7969, but it is no longer "native drops climb and falls". It is now "native resumes climb, then still takes the generic horizontal run-speed path and climbs slightly too fast".
- On frame 7969 native now reaches `(x=0x03A82B00, y=0x01BEB800, x_vel=0x00002B00, y_vel=0xFFFFB800)` while oracle keeps Kid pinned at `x=0x03A80000` and climbs to `y=0x01BEC080` with gfx-slot velocity `0xFFFFC080`.
- The remaining bug therefore sits after the wall-retention gate. The likely live gap is the `loc_A28A` resume-path handling around held-right / `Get_RandomNumber_wordC` / `sub_A3BC`, not the wall lookup itself.

## Strongest Evidence

- `cargo run -p kidcli -- tas-diff --scenario .cache/tas-imports/any-percent-published/scenario.json --frame-start 7967 --frame-end 7970 --include-physics-trace --include-advisory` now shows frame 7969 native gameplay with `kid_iron_knight_attach_probe ... climbing=1=>1/wall=hit=1/x=944/cell=59,26`, `kid_jump_gravity_probe branch=airborne_custom_gravity ... custom_gravity=1`, and `kid_pre_physics_probe ... before_physics[...]y_vel=0xffffb800`.
- The same diff shows the remaining authoritative mismatch at frame 7969 is position only: native final `(0x03a82b00,0x01beb800)` vs oracle `0x30(kid_gfx)@0x03a80000,0x01bec080`.
- `cargo run -p kidcli -- verify-tas --scenario .cache/tas-imports/any-percent-published/scenario.json --frame-start 6975 --frame-end 9530 --bisect` still lands at frame 7969, but native now reports `vel=(Some(11008),Some(-18432))` instead of the earlier dropped-climb `vel=(Some(11008),Some(0))`.
- The focused unit test `iron_knight_resume_climb_uses_rom_wall_lookup` proves why the old gate was wrong: the ROM-shaped lookup still sees the wall at `half_width + 1`, while the earlier 5px airborne probe misses the same column and detaches one frame too early.

## Exact Next Moves

1. Re-read the remainder of the `loc_A28A` resume loop, especially the held-direction branch and `Get_RandomNumber_wordC` / `sub_A3BC`, to explain why oracle keeps `x_pos` pinned on frame 7969 while native still injects `x_vel=0x2B00`.
2. Trace the oracle frame-7969 resume step against that ASM and work out why native climbs to `0x01BEB800` while oracle lands at `0x01BEC080` with gfx-slot velocity `0xFFFFC080`. The remaining Y error is only `0x780`, so it likely comes from resume-path accel/order rather than the wall gate.
3. Fix the resume-frame horizontal/vertical update so frame 7969 stays at `x=0x03A80000` and climbs to `y=0x01BEC080`, then re-run the same TAS diff and bisect to push the first CRC divergence later than frame 7969.

## Verification To Re-run

- `cargo test -p kid-core 7968 -- --nocapture`
- `cargo test -p kid-core 7969 -- --nocapture`
- `cargo test -p kid-core iron_knight_ -- --nocapture`
- `cargo run -p kidcli -- tas-diff --scenario .cache/tas-imports/any-percent-published/scenario.json --frame-start 7967 --frame-end 7970 --include-physics-trace --include-advisory`
- `cargo run -p kidcli -- verify-tas --scenario .cache/tas-imports/any-percent-published/scenario.json --frame-start 6975 --frame-end 9530 --bisect`
- `mise run verify-slice-fast`

## Dead Ends / Do Not Retry

- Do not revert the new `sub_B55C`-shaped resume lookup. The old `probe_wall(..., 5, 31, ...)` gate was the direct cause of the dropped-climb bug on frame 7969.
- Do not broaden the initial airborne attach gate to the form half-width. That earlier dead end still moves the CRC frontier backward to frame 7906 during an unrelated airborne wall-cling window.
- Do not remove the grounded attach hibernate from frame 7968. The ROM still seeds `y_vel = 0xFFFFC000` at `loc_A276` and stops Kid for one frame before the resume loop runs.
