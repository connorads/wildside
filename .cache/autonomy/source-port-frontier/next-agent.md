---
slug: release-gate-md-runtime-ctrl-pressed-mode8
cycle: 26
updated_at: 2026-03-31T03:54:14Z
---
# What Changed
- Re-read the same title-shell ASM as the prior cycle, then inspected the oracle frame-520 `active_enemies` table directly instead of reasoning from shell order alone.
- Confirmed the traced oracle `kid_gfx` really is `gfx_slot_index=2`, `parent_object_ptr=0xFFFF1310`, `layout_ptr=0x6F000040`, `x=0x007F0000`, `y=0x00020000`.
- Built the full slot/parent/layout table for frame 520. The live shell is:
  - slots `0..12`: parent objects `0xFFFF13F8` down to `0xFFFF0E88`
  - slot `13`: parent object `0xFFFF0DA0`, `layout_ptr=0x7B0000DC`
- Rejected a direct native patch that simply reversed the visible letter slot mapping. It worsened frame 520 to native `0x00949000,0x005D6000`, so slot identity is not explained by a plain reverse-letter remap either.

# Current Model
- The creation-order model is wrong, but the naive reverse-letter model is also wrong.
- ASM still supports the structural part of the newer model:
  - `loc_1AB26` allocates visible letters in script order.
  - `Allocate_ObjectSlot` inserts equal-priority objects ahead of older peers.
  - `Delete_CurrentObject` means `loc_1AD48`'s spawned number worker does not execute until a later pass.
  - `Allocate_GfxObjectSlot` preserves execution order within a pass.
- Oracle frame 520 therefore proves something more specific: the traced slot-2 worker is the object at `0xFFFF1310`, but native still does not know how that object's RNG-seeded start pose / glyph lineage maps to the traced oracle pose.
- The remaining gap is now about per-worker identity and seed lineage, not about broad title timing.

# Strongest Evidence
- ASM:
  - `.cache/upstream/kid-chameleon-disasm/kid.asm:1559-1616` `Allocate_ObjectSlot`
  - `.cache/upstream/kid-chameleon-disasm/kid.asm:1718-1775` `Delete_CurrentObject`
  - `.cache/upstream/kid-chameleon-disasm/kid.asm:1824-1911` `Load_GfxObjectSlot` / `Allocate_GfxObjectSlot`
  - `.cache/upstream/kid-chameleon-disasm/kid.asm:28695-29064` `loc_1AB26`, `loc_1AC70`, `loc_1AD48`, `loc_1AE2E`
- Oracle frame-520 slot table from `.cache/native-oracle-runs/boot_title_start_level1/oracle/manifest.json`:
  - slot `0`: `pp=0xFFFF13F8`, `lp=0x73000060`, `x=0x00D80000`, `y=0x00020000`
  - slot `1`: `pp=0xFFFF1384`, `lp=0x64000050`, `x=0x00620000`, `y=0x00DA0000`
  - slot `2`: `pp=0xFFFF1310`, `lp=0x6F000040`, `x=0x007F0000`, `y=0x00020000`
  - slot `3`: `pp=0xFFFF129C`, `lp=0x6F000030`, `x=0x00450000`, `y=0x00020000`
  - slot `13`: `pp=0xFFFF0DA0`, `lp=0x7B0000DC`, `x=0x00A1D000`, `y=0x00D6C000`
- Rejected experiment:
  - plain reverse-letter slot remap changed native frame 520 from `0x0089A000,0x00262000` to `0x00949000,0x005D6000`
  - same wrong pose on `boot-title-start-level1-move-right5s`

# Exact Next Moves
- Stay on the frame-520 worker-identity problem and map the oracle slot table's `layout_ptr` suffixes (`0x20/0x30/0x40/0x50/0x60`) plus `parent_object_ptr` order back to the concrete `loc_1AB26` glyph sequence and `loc_1AC70` RNG start selectors. Do not change timing. Do not retry the plain reverse-letter slot remap.

# Verification To Re-run
- cargo test -q -p kid-core boot_title_start_stays_in_mode_8_without_second_start_press -- --nocapture
- cargo test -q -p kid-core level_entry_has_no_damage_grace_window -- --nocapture
- cargo test -q -p kidcli compare_native_states_reports_boot_level_entry_mode_8_release_gate -- --nocapture
- cargo run -q -p kidcli -- compare-native-oracle --scenario boot-title-start-level1
- cargo run -q -p kidcli -- compare-native-oracle --scenario boot-title-start-level1-move-right5s

# Dead Ends / Do Not Retry
- Do not retry the “creation-order workers plus two `Get_RandomNumber_long` pre-rolls and a 3-frame visible delay” patch; it worsened frame 520 to native `00995000,00202000`.
- Do not retry the old raw reverse-letter mapping (`visible_glyph_count - index - 1`) by itself; oracle parent-pointer evidence made that hypothesis plausible, but the direct remap still worsened frame 520 to native `00949000,005D6000`.
- Do not keep brute-forcing outer-frame delays on top of either simple shell-order model; the remaining gap is worker identity / seed lineage.
- Do not revisit title-card centring or the reverted deferred-spawn experiment.
