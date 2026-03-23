---
slug: any-percent-published-frame-8000-demo-timeout-freeze-cuts-off-iron-knight-climb
cycle: 30
updated_at: 2026-03-23T02:21:45Z
---

# Source-Port Frontier Handoff

## What Changed

- Fixed the frame-7969 Iron Knight climb-resume bug by mirroring the ROM loop instead of falling back into generic jump/run physics.
- `step_iron_knight_climb()` now uses the immediate `previous_input` edge for `C`/`B`, matching `bclr #Button_C,(Ctrl_Pressed)` / `bclr #Button_B,(Ctrl_Pressed)` instead of the old 2-frame-late `prior_input` check.
- Added `IronKnightClimbStep` plus `KidFrameMotionBranch::IronKnightClimbResume`, and routed active climb frames through a dedicated vertical-only resume step that keeps `x_vel`/`x_pos` pinned while still running `sub_902A`-shaped y movement and collision.
- Added `published_tas_frame_7969_matches_oracle_iron_knight_climb_resume_pose` and updated the frame-7969 diagnostics to the new truth.
- Re-ran `cargo test -p kid-core 7968 -- --nocapture`, `cargo test -p kid-core 7969 -- --nocapture`, `cargo test -p kid-core iron_knight_ -- --nocapture`, `cargo run -p kidcli -- tas-diff --scenario .cache/tas-imports/any-percent-published/scenario.json --frame-start 7967 --frame-end 7970 --include-physics-trace --include-advisory`, `cargo run -p kidcli -- verify-tas --scenario .cache/tas-imports/any-percent-published/scenario.json --frame-start 6975 --frame-end 9530 --bisect`, `cargo run -p kidcli -- tas-diff --scenario .cache/tas-imports/any-percent-published/scenario.json --frame-start 7998 --frame-end 8002 --include-physics-trace --include-advisory`, and `mise run verify-slice-fast`.

## Current Model

- Frame 7969 is no longer authoritative. Native now matches oracle on the wall-pinned climb resume pose: `x=0x03A80000`, `y=0x01BEC080`, `x_vel=0`, and climb drift `y_vel=0xFFFFC080`.
- The first CRC divergence has moved to frame 8000.
- The new blocker is not inside the climb loop itself. Frames 7998 and 7999 still run `motion=iron_knight_climb_resume` and stay advisory-only, but frame 8000 abruptly switches native to `motion=demo_auto_timeout_frozen` while oracle continues live gameplay and climbs onward.
- Because native freezes on frame 8000, it reuses the frame-7999 pose (`y=0x01BABD00`, `y_vel=0xFFFFB600`) while oracle advances Kid gfx to `y=0x01BA6D00` with `y_velocity=0xFFFFB000`.

## Strongest Evidence

- `cargo run -p kidcli -- verify-tas --scenario .cache/tas-imports/any-percent-published/scenario.json --frame-start 6975 --frame-end 9530 --bisect` now reports `RESULT: first CRC divergence at frame 8000`.
- `cargo run -p kidcli -- tas-diff --scenario .cache/tas-imports/any-percent-published/scenario.json --frame-start 7967 --frame-end 7970 --include-physics-trace --include-advisory` shows frame 7969 is advisory-only: native final `(0x03a80000,0x01bec080)` with `motion=iron_knight_climb_resume`, matching oracle position.
- `cargo run -p kidcli -- tas-diff --scenario .cache/tas-imports/any-percent-published/scenario.json --frame-start 7998 --frame-end 8002 --include-physics-trace --include-advisory` shows:
  - frame 7999 native still in `motion=iron_knight_climb_resume`, final `y=0x01BABD00`
  - frame 8000 native flips to `motion=demo_auto_timeout_frozen` and keeps `y=0x01BABD00`
  - frame 8000 oracle keeps live gameplay and advances Kid gfx to `y=0x01BA6D00`, `y_velocity=0xFFFFB000`
- The frame-8000 native gameplay debug already points at the wrong branch directly: `kid_gameplay_path_probe: lifecycle=alive/motion=demo_auto_timeout_frozen/post=none/paused=0`.

## Exact Next Moves

1. Read the ROM demo/title-return path that corresponds to native `demo_auto_timeout_freeze_gameplay_frame` / `finish_demo_title_return()` and explain why native can still enter the freeze branch during live gameplay on frame 8000.
2. Trace the native writer of `demo_auto_timeout_freeze_gameplay_frame` across frames 7999-8000 and compare it to the oracle run; the climb loop is now fine, so the bug is the freeze snapshot arming one gameplay frame too early.
3. Fix only that freeze/handoff gate so frame 8000 stays on gameplay/climb instead of `DemoAutoTimeoutFrozen`, then re-run the frame-7998..8002 diff, the 6975..9530 bisect, and `mise run verify-slice-fast`.

## Verification To Re-run

- `cargo test -p kid-core 7968 -- --nocapture`
- `cargo test -p kid-core 7969 -- --nocapture`
- `cargo test -p kid-core iron_knight_ -- --nocapture`
- `cargo run -p kidcli -- tas-diff --scenario .cache/tas-imports/any-percent-published/scenario.json --frame-start 7967 --frame-end 7970 --include-physics-trace --include-advisory`
- `cargo run -p kidcli -- verify-tas --scenario .cache/tas-imports/any-percent-published/scenario.json --frame-start 6975 --frame-end 9530 --bisect`
- `cargo run -p kidcli -- tas-diff --scenario .cache/tas-imports/any-percent-published/scenario.json --frame-start 7998 --frame-end 8002 --include-physics-trace --include-advisory`
- `mise run verify-slice-fast`

## Dead Ends / Do Not Retry

- Do not reopen the frame-7969 `loc_A28A` wall-resume bug. The dedicated resume step plus `previous_input` edge detection fixed it and moved the frontier to frame 8000.
- Do not switch active Iron Knight climb frames back to generic `NormalPhysics`; that was the direct source of the old `x_vel=0x2B00` / `y=0x01BEB800` mismatch.
- Do not use `prior_input` for Iron Knight climb `C`/`B` edge checks. It reintroduces a phantom frame-7969 `C` press and the wrong `0xFFFFB800` climb boost.
