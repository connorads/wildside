---
slug: release-gate-md-runtime-level-entry-slot2-rng-blocker
cycle: 47
updated_at: 2026-03-31T06:32:00Z
---
# What Changed
- Mapped oracle frame-504 `gfx_slot_index` `0..12` back to `Blue Lake Woods` in `loc_1AB26` script order using the known target positions from the native title-card metadata.
- Confirmed that the first visible shell is exactly the 13 letter workers, not the deferred level-number worker from `loc_1AD48`.
- Narrowed the next patch to a concrete behavioural change: hide slot 13 until the next object pass, then line up the letter-worker reveal timing/RNG with oracle frame 504.

# Current Model
- The frame-504 shell is no longer ambiguous:
  - slot 0 = `B`
  - slot 1 = `l`
  - slot 2 = `u`
  - slot 3 = `e`
  - slot 4 = `L`
  - slot 5 = `a`
  - slot 6 = `k`
  - slot 7 = `e`
  - slot 8 = `W`
  - slot 9 = `o`
  - slot 10 = `o`
  - slot 11 = `d`
  - slot 12 = `s`
- Slot 13 is the level-number worker from `loc_1AD48`, and oracle proves it is not part of the first visible frame.
- That means the next native fix should split the level-entry shell into two phases:
  first reveal only the 13 `loc_1AB26` letters on frame 504, then let the deferred `loc_1AD48` suffix worker appear on the following object pass.
- The remaining unknown after that split is the exact RNG alignment for the letters, especially slot 2 (`u`) at `0x00940000,0xFFF00000`.

# Strongest Evidence
- ASM:
  - `.cache/upstream/kid-chameleon-disasm/kid.asm:28704-28826` `loc_1AB26`
  - `.cache/upstream/kid-chameleon-disasm/kid.asm:28952-28982` `loc_1AD48`
  - `.cache/upstream/kid-chameleon-disasm/kid.asm:1572-1606` `Allocate_ObjectSlot`
  - `.cache/upstream/kid-chameleon-disasm/kid.asm:1618-1658` `Execute_Objects`
- Native title-card metadata:
  - `crates/kid-core/src/tests.rs:392` confirms slot 2 is `u`
  - `crates/kid-core/src/tests.rs:422` confirms slot 13 is the deferred number worker at `0x01340000,0x00500000`
- Oracle:
  - `.cache/oracle-probe/oracle/manifest.json`
  - Frame 504 visible slots `0..12` only, with no slot 13
  - Frame 504 slot 2 at `0x00940000,0xFFE00000`

# Exact Next Moves
- Do one implementation only: patch native level-entry visibility so frame 504 exposes only the 13 `loc_1AB26` letter workers and keeps the `loc_1AD48` number worker hidden until the next object pass.
- Preserve the current baseline tests, then add a focused regression that slot 13 is absent on frame 504.
- After that split is in place, re-run oracle/native comparison to see whether the remaining slot-2 pose error is purely RNG timing or still includes ordering drift.
- Do not touch compare masking or later gameplay state in the same cycle.

# Verification To Re-run
- cargo run -q -p kidcli -- oracle-probe --scenario .cache/oracle-probe/boot_title_start_level1_dense.json --frame-start 499 --frame-end 504
- cargo run -q -p kidcli -- probe-native --scenario boot-title-start-level1 --frame-start 458 --frame-end 466
- cargo test -q -p kid-core boot_title_start_stays_in_mode_8_without_second_start_press -- --nocapture
- cargo test -q -p kid-core boot_title_start_mode_8_keeps_advancing_rng_before_gameplay_entry -- --nocapture
- cargo run -q -p kidcli -- compare-native-oracle --scenario boot-title-start-level1

# Dead Ends / Do Not Retry
- Do not retry the pure “delay worker birth to frame 504” patch by itself.
- Do not retry the backed-out “delay worker birth + defer `loc_1AC70` RNG” patch unchanged; it still exposes the wrong frame-504 shell and later diverges at level-entry@520.
- Do not assume the first visible frame includes the level-number worker. Oracle frame 504 proves it does not.
