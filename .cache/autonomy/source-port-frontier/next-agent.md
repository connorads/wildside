---
slug: any-percent-published-frame-11549-demo-2-cyclone-flight-y-high
cycle: 55
updated_at: 2026-03-24T03:59:36Z
---

# Source-Port Frontier Handoff

## What Changed

- Re-read `sub_B168` and `sub_1007A` in `.cache/upstream/kid-chameleon-disasm/kid.asm`. `sub_B168 loc_B214..loc_B26C` is still just the Cyclone Y-acceleration path; `sub_1007A` explicitly writes `$E001` before spawning the prize object.
- Added ignored diagnostic `published_tas_second_timeout_restart_frame_11549_rehits_the_restored_prize_smash_block` in [crates/kid-core/src/tests.rs](/home/connor/git/kc/crates/kid-core/src/tests.rs) to pin the live native behaviour on frame 11549.
- Corrected stale native comments in [crates/kid-core/src/types.rs](/home/connor/git/kc/crates/kid-core/src/types.rs) and [crates/kid-core/src/blocks.rs](/home/connor/git/kc/crates/kid-core/src/blocks.rs): the prize lifecycle does not clear straight to empty after the restore phase; `sub_1007A` writes collision-solid Smash cell `$E001` and then spawns the prize object.
- Verified the new ignored diagnostic and a nearby non-ignored published-TAS smoke test. The older ignored frame-11549 parity test still fails earlier at its frame-11548 camera expectation, so do not use that assertion as current truth until it is rebased.

## Current Model

- The frame-11549 blocker is no longer “Cyclone gravity/hand-off is wrong”. Native `step_cyclone_flight()` and the AsmFaithful vertical integrator both reach the oracle-backed post-vertical pose on frame 11549.
- The actual native break is an immediate re-hit on the restored prize-block cell at `(22,14)` after the post-vertical rise:
  - `physics.post_vertical = (0x010E9A00, 0xFFFF9A00)`
  - final native `kid_y_pos = 0x010F0000`, `kid_y_vel = 0`
  - `block_destroy_effects` contains `(352,224,8)` on the same frame
  - the floating prize item is already live at `level_col=22`, `level_row=14`, `remaining=499`
- By the time the diagnostic inspects the grid on frame 11549, both edge probes at the head row are empty. That means the in-frame snap is not a generic empty-space ceiling hit; native has already cleared the restored Smash block during the same frame.
- So the live question is now prize-cell collision visibility/timing around the restored `$E001` cell, not Cyclone Y-acceleration. Either native exposes that restored Smash block to Kid one frame too early on the timeout-restart path, or the player/block update ordering around that object coroutine is still wrong.

## Strongest Evidence

- ASM
  - `.cache/upstream/kid-chameleon-disasm/kid.asm`
    - `sub_B168` `loc_B214..loc_B26C`: Cyclone flying path only updates `Cyclone_YAcceleration` and `y_vel(a3)`.
    - `sub_1007A` around `loc_10120`: after the bounce coroutine, the ROM writes `$E001` via `sub_11530` and then spawns the prize object.
- Native diagnostic
  - `cargo test -q -p kid-core published_tas_second_timeout_restart_frame_11549_rehits_the_restored_prize_smash_block -- --ignored --nocapture`
    - passes
    - pins `physics.pre_vertical = (0x010F0000, 0xFFFF9A00)`
    - pins `physics.post_vertical = (0x010E9A00, 0xFFFF9A00)`
    - pins final native `kid_y_pos = 0x010F0000`, `kid_y_vel = 0`
    - proves `floating_items` already contains the prize at `(22,14)` with `remaining=499`
    - proves `block_destroy_effects` already contains `(352,224,8)` on that same frame
    - proves the grid is empty at both ceiling edge probes by frame end, so the block was re-hit and cleared during the frame
- Smoke test
  - `cargo test -q -p kid-core published_tas_demo_2_resumes_demo_play_on_frame_6906 -- --nocapture`
    - passes
- Known stale regression
  - `cargo test -q -p kid-core published_tas_second_timeout_restart_keeps_the_oracle_backed_cyclone_flight_arc_on_frame_11549 -- --ignored --nocapture`
    - currently fails at its frame-11548 camera expectation (`0x00740000` asserted vs native `0x00730000`)
    - this failure predates the new diagnostic and means the ignored parity test needs rebasing before it can serve as the fix gate

## Exact Next Moves

1. Re-read the surrounding object-order ASM for the prize-block restore/spawn path and the player update order on this timeout-restart demo path. Start from `sub_1007A` and the object scheduler it runs under, then compare that ordering to `step_deferred_ceiling_clears()` / `finish_deferred_ceiling_clears()` in native.
2. Fix the collision-visibility timing so frame 11549 keeps the oracle-backed `post_vertical` rise instead of re-hitting the restored Smash block. Bias towards lifecycle/order corrections in [crates/kid-core/src/blocks.rs](/home/connor/git/kc/crates/kid-core/src/blocks.rs) and [crates/kid-core/src/movement.rs](/home/connor/git/kc/crates/kid-core/src/movement.rs), not Cyclone gravity changes in [crates/kid-core/src/helmets.rs](/home/connor/git/kc/crates/kid-core/src/helmets.rs).
3. Once the fix lands, rebase the ignored frame-11549 parity test so its pre-frontier frame-11548 expectation matches current truth, then use it as the oracle gate for the final patch.

## Verification To Re-run

- `cargo test -q -p kid-core published_tas_second_timeout_restart_frame_11549_rehits_the_restored_prize_smash_block -- --ignored --nocapture`
- `cargo test -q -p kid-core published_tas_demo_2_resumes_demo_play_on_frame_6906 -- --nocapture`
- After the fix: `cargo run -q -p kidcli -- tas-diff --scenario .cache/tas-imports/any-percent-published/scenario.json --frame-start 11548 --frame-end 11551 --include-advisory --include-physics-trace`
- After the fix: `cargo run -q -p kidcli -- verify-tas --scenario .cache/tas-imports/any-percent-published/scenario.json --frame-start 9531 --bisect`
- After the fix: `mise run verify-slice-fast`

## Dead Ends / Do Not Retry

- Do not keep pushing on Cyclone Y-acceleration or post-physics `step_cyclone_activation()`. The frame-11549 diagnostic proves native already reaches the oracle-backed `post_vertical` rise before the snap.
- Do not treat the empty end-of-frame ceiling probes as proof that `apply_ceiling_collision()` never fired. The restored Smash block is already gone by the time the diagnostic inspects the grid; the same frame also spawns the matching `block_destroy_effect`.
- Do not rely on the existing ignored `published_tas_second_timeout_restart_keeps_the_oracle_backed_cyclone_flight_arc_on_frame_11549` expectations until its frame-11548 camera assertion is rebased.
- Do not follow `.cache/autonomy/source-port-frontier/next-task.json` for this frontier. It is still stale and points at the old frame-7969 Knights Isle work.
