---
slug: release-gate-md-runtime-level-entry-slot2-rng-blocker
cycle: 46
updated_at: 2026-03-31T06:24:00Z
---
# What Changed
- Re-read `Allocate_ObjectSlot`, `Execute_Objects`, and the `loc_1AC70` / `loc_1AD48` title-card path instead of retrying the backed-out lazy-materialisation patch.
- Dumped oracle frame 504's visible `gfx_slot_index` list from `.cache/oracle-probe/oracle/manifest.json`.
- Confirmed a concrete scheduler fact from the ROM: newly allocated same-priority objects are inserted ahead of the current object in the linked list, so they are not reached again during the same `Execute_Objects` pass.

# Current Model
- Oracle frame 504 exposes exactly `gfx_slot_index` `0..12`. Slot 13 is absent on the first visible frame.
- That matches the ASM: `loc_1AB26` can allocate the deferred `loc_1AD48` helper in the same pass, but `loc_1AD48`'s own `loc_1AC70` suffix worker(s) cannot execute until the next frame because `Allocate_ObjectSlot` inserts same-priority objects ahead of the current node and `Execute_Objects` resumes from the current node's next pointer.
- So one part of the native 14-vs-13 mismatch is now explained: the level-number worker should not be visible on frame 504.
- The remaining open question is the letter-worker ordering / RNG seeding among slots `0..12`. Slot 2 still needs to land at oracle's `0x00940000,0xFFF00000`.

# Strongest Evidence
- ASM:
  - `.cache/upstream/kid-chameleon-disasm/kid.asm:1572-1606` `Allocate_ObjectSlot`
  - `.cache/upstream/kid-chameleon-disasm/kid.asm:1618-1658` `Execute_Objects`
  - `.cache/upstream/kid-chameleon-disasm/kid.asm:28828-28982` `loc_1ABF4`, `loc_1AC70`, `loc_1AD48`
- Oracle:
  - `.cache/oracle-probe/oracle/manifest.json`
  - Frame 504 visible slots:
    - `0: 0x01000000,0xFFE00000`
    - `1: 0x00680000,0x01000000`
    - `2: 0x00940000,0xFFE00000`
    - `3: 0x004C0000,0xFFE00000`
    - `4: 0x01600000,0x00980000`
    - `5: 0x01600000,0x00780000`
    - `6: 0x01600000,0x00CC0000`
    - `7: 0x00FC0000,0xFFE00000`
    - `8: 0x00640000,0xFFE00000`
    - `9: 0x00A40000,0xFFE00000`
    - `10: 0x01600000,0x007C0000`
    - `11: 0x01200000,0x01000000`
    - `12: 0x01600000,0x00200000`
  - No frame-504 slot 13 exists.
- Native baseline:
  - `cargo run -q -p kidcli -- probe-native --scenario boot-title-start-level1 --frame-start 458 --frame-end 466`
    still shows all 14 title workers immediately on frame 460.

# Exact Next Moves
- Do one investigation only: map oracle frame-504 slots `0..12` back to `loc_1AB26`'s script order and identify which letter each visible slot represents.
- Use the known target positions from the native title-card metadata plus oracle frame-504 start positions to infer the first-visible letter ordering before changing code.
- Once the slot-to-letter mapping is known, patch native so frame 504 exposes only the letter workers and keeps the deferred `loc_1AD48` number worker hidden until the next object pass.
- Do not reintroduce the lazy-materialisation patch until the slot-to-letter mapping is explicit.

# Verification To Re-run
- cargo run -q -p kidcli -- oracle-probe --scenario .cache/oracle-probe/boot_title_start_level1_dense.json --frame-start 499 --frame-end 504
- cargo run -q -p kidcli -- probe-native --scenario boot-title-start-level1 --frame-start 458 --frame-end 466
- cargo test -q -p kid-core boot_title_start_stays_in_mode_8_without_second_start_press -- --nocapture
- cargo test -q -p kid-core boot_title_start_mode_8_keeps_advancing_rng_before_gameplay_entry -- --nocapture
- cargo run -q -p kidcli -- compare-native-oracle --scenario boot-title-start-level1

# Dead Ends / Do Not Retry
- Do not retry the pure “delay worker birth to frame 504” patch by itself.
- Do not retry the backed-out “delay worker birth + defer `loc_1AC70` RNG” patch unchanged; it still exposes the wrong frame-504 shell and later diverges at level-entry@520.
- Do not assume slot 13 is part of the first visible shell. Oracle frame 504 proves it is not.
