---
slug: any-percent-published-frame-7969-knights-isle-native-stays-grounded-while-oracle-lifts-off
cycle: 28
updated_at: 2026-03-23T01:58:08Z
---

# Source-Port Frontier Handoff

## What Changed

- Ported the ROM's one-frame coroutine boundary for grounded Iron Knight attach: `step_iron_knight_climb(...)` now reports grounded walking-path attaches, and the gameplay loop suppresses same-frame player physics for that case while preserving the seeded `y_vel = 0xFFFFC000`.
- Added `KidFrameMotionBranch::IronKnightAttachHibernate` plus a frozen physics-trace snapshot so frame 7968 now records the attach hibernate explicitly instead of falling through native normal physics.
- Rewrote the frame-7968/7969 Kid diagnostics to match the new frontier: frame 7968 now asserts preserved `x/y`, seeded climb velocity, and attach-hibernate motion; frame 7969 is now the first live next-frame physics break.
- Re-ran `cargo test -p kid-core 7968 -- --nocapture`, `cargo test -p kid-core 7969 -- --nocapture`, `cargo test -p kid-core iron_knight_ -- --nocapture`, `cargo run -p kidcli -- tas-diff --scenario .cache/tas-imports/any-percent-published/scenario.json --frame-start 7967 --frame-end 7970 --include-physics-trace --include-advisory`, `cargo run -p kidcli -- verify-tas --scenario .cache/tas-imports/any-percent-published/scenario.json --frame-start 6975 --frame-end 9530 --bisect`, and `mise run verify-slice-fast`.

## Current Model

- The frame-7968 blocker is fixed. Native now mirrors `loc_A276`: it consumes the grounded C-edge, enters Iron Knight climb, seeds `y_vel = 0xFFFFC000`, clears grounded state, and hibernates without advancing `x_pos` or `y_pos`.
- The first authoritative divergence is still frame 7969, but it has moved to the next frame's wall-climb resume path rather than the attach frame.
- On frame 7969 native starts from the preserved attach state at `x=0x03A80000`, `y=0x01BF0000`, `y_vel=0xFFFFC000`, but then drops out of Iron Knight climb before applying the ROM wall-climb update. That sends Kid into normal airborne physics, injects run speed (`x_vel=0x00002B00`), zeroes vertical velocity, and leaves `y_pos` unchanged at `0x01BF0000`.
- The strongest native clue is the mismatch between the attach/climb retention probes on frame 7969: `kid_flip_branch_probe` still sees the wall at `cell=59,27`, but `kid_iron_knight_attach_probe` reports the airborne wall probe as `wall=hit=0` and `climbing=1=>0`. The likely missing behaviour is the ROM wall-retention logic immediately after `loc_A28A` resumes, not the grounded attach gate.

## Strongest Evidence

- `cargo run -p kidcli -- tas-diff --scenario .cache/tas-imports/any-percent-published/scenario.json --frame-start 7967 --frame-end 7970 --include-physics-trace --include-advisory` now reports frame 7968 as advisory-only: native final `(0x03a80000,0x01bf0000)` with `vel=(0x00000000,0xffffc000)` and `motion=iron_knight_attach_hibernate`.
- The same diff still shows the first authoritative mismatch at frame 7969: native final `(0x03a82b00,0x01bf0000)` versus oracle `0x30(kid_gfx)@0x03a80000,0x01bec080 vel=0x00000000,0xffffc080`.
- Frame-7969 native diagnostics show the climb state being lost before movement: `kid_special_motion_probe ... iron_knight_climbing=0`, `kid_jump_gravity_probe branch=airborne_default_gravity ... custom_gravity=0`, and `kid_iron_knight_attach_probe ... climbing=1=>0/wall=hit=0`.
- `cargo run -p kidcli -- verify-tas --scenario .cache/tas-imports/any-percent-published/scenario.json --frame-start 6975 --frame-end 9530 --bisect` now lands at frame 7969 with native `kid=(0x03a82b00,0x01bf0000) vel=(Some(11008),Some(0))`, confirming the frame-7968 same-frame motion bug is gone.

## Exact Next Moves

1. Read the wall-climb resume path after `loc_A28A` in `kid.asm`, especially the `sub_7ACC` / `sub_B55C` / `sub_902A` path that runs after `Hibernate_Object_1Frame`, and work out how the ROM keeps Iron Knight in climb state on frame 7969.
2. Compare that ROM resume logic against native `step_iron_knight_climb(...)`'s "wall still exists" gate. The current airborne `probe_wall(..., 5, 31, ...)` looks too weak, because frame 7969 native loses climb even though the shared wall scan still hits `cell=59,27`.
3. Fix the resume-frame climb retention so frame 7969 stays at `x=0x03A80000`, advances upward to `y=0x01BEC080`, and keeps wall-climb motion instead of falling into normal airborne physics.
4. Re-run `cargo run -p kidcli -- tas-diff --scenario .cache/tas-imports/any-percent-published/scenario.json --frame-start 7967 --frame-end 7970 --include-physics-trace --include-advisory` and `cargo run -p kidcli -- verify-tas --scenario .cache/tas-imports/any-percent-published/scenario.json --frame-start 6975 --frame-end 9530 --bisect`. The target is moving the first CRC divergence later than frame 7969.

## Verification To Re-run

- `cargo test -p kid-core 7968 -- --nocapture`
- `cargo test -p kid-core 7969 -- --nocapture`
- `cargo test -p kid-core iron_knight_ -- --nocapture`
- `cargo run -p kidcli -- tas-diff --scenario .cache/tas-imports/any-percent-published/scenario.json --frame-start 7967 --frame-end 7970 --include-physics-trace --include-advisory`
- `cargo run -p kidcli -- verify-tas --scenario .cache/tas-imports/any-percent-published/scenario.json --frame-start 6975 --frame-end 9530 --bisect`
- `mise run verify-slice-fast`

## Dead Ends / Do Not Retry

- Do not remove the new grounded attach hibernate. The ROM really does stop executing Kid immediately after seeding climb state at `loc_A276`.
- Do not reapply `probe_wall_column(...)` to every Iron Knight attach path. That earlier dead end still moves the CRC frontier backward to frame 7906 during an airborne wall-cling window.
- Do not go back to the generic half-width-7 wall probe swap. The grounded attach gate is solved; the live bug is the resume-frame climb retention after the hibernated attach.
