---
slug: any-percent-published-frame-12430-native-keeps-kid-after-oracle-zeroes-pose
cycle: 56
updated_at: 2026-03-24T04:18:34Z
---

# Source-Port Frontier Handoff

## What Changed

- Fixed the published-TAS frame-11549 Cyclone/prize-block frontier in native code.
- Added `DeferredPrizeBlockPhase::FreshSmash` so the freshly written `$E001` prize-block Smash cell stays non-blocking for one frame after `sub_1007A` finishes.
- Promoted `published_tas_second_timeout_restart_keeps_the_oracle_backed_cyclone_flight_arc_on_frame_11549` to a real regression test and rebased its stale frame-11548 camera expectation to `0x0073_0000`.
- Added focused unit coverage for the new one-frame non-blocking `$E001` window and removed the obsolete ignored diagnostic that only pinned the pre-fix re-hit.

## Current Model

- The old frame-11549 blocker is solved. `tas-diff --frame-start 11548 --frame-end 11551 --include-advisory --include-physics-trace` is now authoritative-clean across frames 11548..11550; only advisory drift remains.
- The fix is not “Cyclone gravity changed”. The ROM evidence is scheduler order:
  - `sub_7196` allocates Kid at object priority `#1`.
  - `sub_1007A` prize-block coroutine runs at `#$2001`.
  - `sub_1007A` spawns `Prize_Init` at `#$1FFF`.
  - `Allocate_ObjectSlot` keeps the object list sorted by priority, so Kid executes before both the prize-block coroutine and the spawned prize object.
- Native therefore needed a post-`sub_1007A` one-frame collision-visibility delay for the freshly written `$E001` cell, not more Cyclone Y-acceleration work.
- The next first authoritative mismatch after the fix is frame `12430`.
  - `verify-tas --scenario .cache/tas-imports/any-percent-published/scenario.json --frame-start 9531 --bisect` now reports `first CRC divergence at frame 12430`.
  - Native frame 12430: `mode=0x002c level=00 kid=(0x002e0000,0x008c0000) vel=(0,0)`.
  - Oracle frame 12430: `kid_x_pos=0`, `kid_y_pos=0`, with only one `0x30` gfx object left at `(0x00810000,0x00c80000)`.

## Strongest Evidence

- ASM
  - `.cache/upstream/kid-chameleon-disasm/kid.asm`
    - `sub_7196`: Kid allocated at priority `#1`.
    - `Allocate_ObjectSlot`: object list sorted by priority.
    - `sub_1007A`: prize-block coroutine priority `#$2001`, writes `$E001`, then spawns `Prize_Init` at `#$1FFF`.
- Regression + unit tests
  - `cargo test -q -p kid-core published_tas_second_timeout_restart_keeps_the_oracle_backed_cyclone_flight_arc_on_frame_11549 -- --nocapture`
    - passes
  - `cargo test -q -p kid-core fresh_smash_prize_ceiling_frame_does_not_block_motion -- --nocapture`
    - passes
  - `cargo test -q -p kid-core restored_prize_ceiling_frame_blocks_motion_without_restarting_dispatch -- --nocapture`
    - passes
  - `cargo test -q -p kid-core prize_block_lifecycle_spawns_item_only_after_bounce_completion -- --nocapture`
    - passes
- Oracle/native diff
  - `cargo run -q -p kidcli -- tas-diff --scenario .cache/tas-imports/any-percent-published/scenario.json --frame-start 11548 --frame-end 11551 --include-advisory --include-physics-trace`
    - frames 11548..11550 now show advisory-only drift
  - `cargo run -q -p kidcli -- verify-tas --scenario .cache/tas-imports/any-percent-published/scenario.json --frame-start 9531 --bisect`
    - first CRC divergence moved to frame `12430`
  - `cargo run -q -p kidcli -- tas-diff --scenario .cache/tas-imports/any-percent-published/scenario.json --frame-start 12426 --frame-end 12434 --include-advisory`
    - first authoritative diff is frame `12430`
    - native keeps Kid at `(0x002e0000,0x008c0000)` while oracle zeroes Kid pose

## Exact Next Moves

1. Re-read the ASM for the post-demo/title hand-off around native `mode=0x002c` and identify the oracle object routine at ROM address `0x01B920`. Start from the mode/state dispatch in `.cache/upstream/kid-chameleon-disasm/kid.asm`, not broad docs.
2. Explain why oracle zeroes `kid_x_pos`/`kid_y_pos` on frame 12430 while native keeps the level-00 grounded slice alive at cell `(2,9)`. Bias towards title/demo hand-off or hidden-Kid sequencing, not gameplay terrain/collision fixes.
3. Add a focused regression around frames 12429..12430 once the cause is proven, then fix the new frontier.

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
