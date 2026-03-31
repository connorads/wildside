---
slug: release-gate-md-runtime-ctrl-pressed-mode8
cycle: 27
updated_at: 2026-03-31T03:56:19Z
---
# What Changed
- Decoded the oracle frame-520 `layout_ptr` values for the title shell instead of treating them as opaque pointers.
- Confirmed the synthetic encoding:
  - high byte = glyph byte (`0x73='s'`, `0x64='d'`, `0x6F='o'`, ...)
  - low word = title-card target `x` (`0x0020`, `0x0030`, `0x0040`, `0x0050`, `0x0060`, `0x00DC`)
- That means the oracle frame-520 slot order is concretely `s d o o w e k a l e u l b {`, with slot 2 specifically the reverse-order third-line `o` at target `x=0x40`.
- This keeps the earlier rejection standing: reversing native slot indices alone is insufficient, because slot-2 glyph identity is now known and the remaining miss is the worker's RNG-seeded start pose.

# Current Model
- The creation-order model is wrong, but the naive reverse-letter model is also wrong.
- ASM still supports the structural part of the newer model:
  - `loc_1AB26` allocates visible letters in script order.
  - `Allocate_ObjectSlot` inserts equal-priority objects ahead of older peers.
  - `Delete_CurrentObject` means `loc_1AD48`'s spawned number worker does not execute until a later pass.
  - `Allocate_GfxObjectSlot` preserves execution order within a pass.
- Oracle frame 520 now proves something more specific than “reverse order”:
  - slot 2 is parent object `0xFFFF1310`
  - that worker's glyph is definitely `o`
  - that worker's target `x` is definitely `0x40`
- The remaining gap is therefore not glyph identity or shell order. It is the `loc_1AC70` seed lineage for that concrete `o@x=0x40` worker.

# Strongest Evidence
- ASM:
  - `.cache/upstream/kid-chameleon-disasm/kid.asm:1559-1616` `Allocate_ObjectSlot`
  - `.cache/upstream/kid-chameleon-disasm/kid.asm:1718-1775` `Delete_CurrentObject`
  - `.cache/upstream/kid-chameleon-disasm/kid.asm:1824-1911` `Load_GfxObjectSlot` / `Allocate_GfxObjectSlot`
  - `.cache/upstream/kid-chameleon-disasm/kid.asm:28695-29064` `loc_1AB26`, `loc_1AC70`, `loc_1AD48`, `loc_1AE2E`
- Oracle frame-520 slot table from `.cache/native-oracle-runs/boot_title_start_level1/oracle/manifest.json`:
  - slot `0`: `s@x=0x60`, `pp=0xFFFF13F8`, pose `0x00D80000,0x00020000`
  - slot `1`: `d@x=0x50`, `pp=0xFFFF1384`, pose `0x00620000,0x00DA0000`
  - slot `2`: `o@x=0x40`, `pp=0xFFFF1310`, pose `0x007F0000,0x00020000`
  - slot `3`: `o@x=0x30`, `pp=0xFFFF129C`, pose `0x00450000,0x00020000`
  - slot `4`: `w@x=0x20`, `pp=0xFFFF1228`, pose `0x01100000,0x008C0000`
  - slot `10`: `u@x=0x40`, `pp=0xFFFF0F70`, pose `0x01180000,0x00670000`
  - slot `12`: `b@x=0x20`, `pp=0xFFFF0E88`, pose `0x01100000,0x00220000`
  - slot `13`: `{@x=0xDC`, `pp=0xFFFF0DA0`, pose `0x00A1D000,0x00D6C000`
- Rejected experiment:
  - plain reverse-letter slot remap changed native frame 520 from `0x0089A000,0x00262000` to `0x00949000,0x005D6000`
  - same wrong pose on `boot-title-start-level1-move-right5s`

# Exact Next Moves
- Stay on the concrete `o@x=0x40` frame-520 worker and solve which `loc_1AC70` start branch (`left`, `top`, `right`, `bottom`) plus RNG byte/word outputs can land it at `0x007F0000,0x00020000`. Compare that selector stream against native's current `RomRandomState` consumption order before changing any slot mapping.

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
