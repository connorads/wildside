---
slug: release-gate-md-runtime-ctrl-pressed-mode8
cycle: 34
updated_at: 2026-03-31T04:29:04Z
---
# What Changed
- Used `kidcli oracle-probe` with WRAM dumps for frames 330, 340, 350, and 360 on the `boot_title_start_level1` scenario.
- Decoded the live linked object list from `Addr_FirstObjectSlot` (`$FFFFF842`) in the oracle WRAM dumps.
- Confirmed the late cold-boot title object roster is stable across those frames:
  - `0xFFFF0EFC -> code 0x0001BB50`
  - `0xFFFF0E88 -> code 0x0001B6A6`
  - `0xFFFF0E14 -> code 0x0001B650`
  - `0xFFFF0DA0 -> code 0x0001BAF6`
- Those code pointers map back to the already-cleared RNG-silent title handlers:
  - `0x1BB50` inside `loc_1BB52`
  - `0x1B6A6` inside `loc_1B6A2/loc_1B6AC`
  - `0x1B650` inside `loc_1B64C`
  - `0x1BAF6` inside `loc_1BAEE`

# Current Model
- The pre-frame-360 object lineup is not changing late in the title shell; the same four handlers stay live from at least frame 330 through frame 360.
- Every one of those live handlers is RNG-silent in the ASM.
- That means the missing cold-boot carry is not caused by some hidden late-title object swapping in near frame 360.
- The remaining explanations are now tighter:
  - cumulative `Execute_Objects -> Get_RandomNumber_long` burns over the earlier cold-boot/title lifetime
  - or an earlier, now-dead object lineage before frame 330 that made direct RNG calls and left the stream offset carried into the stable late-title shell

# Strongest Evidence
- Oracle probe command:
  - `cargo run -q -p kidcli -- oracle-probe --scenario .cache/oracle-probe/boot_title_start_level1_dense.json --frame-start 330 --frame-end 360 --wram-dump-frames 330,340,350,360`
- WRAM artefacts:
  - `.cache/oracle-probe/oracle/wram_frame_330.bin`
  - `.cache/oracle-probe/oracle/wram_frame_340.bin`
  - `.cache/oracle-probe/oracle/wram_frame_350.bin`
  - `.cache/oracle-probe/oracle/wram_frame_360.bin`
- Supporting ASM:
  - `.cache/upstream/kid-chameleon-disasm/variables.asm:52-53` `Addr_FirstObjectSlot`, `Addr_CurrentObject`
  - `.cache/upstream/kid-chameleon-disasm/kid.asm:1583-1615` `Allocate_ObjectSlot`
  - `.cache/upstream/kid-chameleon-disasm/kid.asm:1636-1661` `Execute_Objects`
  - `.cache/upstream/kid-chameleon-disasm/kid.asm:29917-29948`, `.cache/upstream/kid-chameleon-disasm/kid.asm:30390-30458`

# Exact Next Moves
- Stay narrow: move the same WRAM/object-list probe earlier on the cold-boot title path to the first frame where this four-object stable shell is not yet present, then identify which earlier object lineage disappears before frame 330.
- Once you find the earlier disappearing lineage, read that label’s ASM and check whether it is the direct RNG caller that explains the carried offset.

# Verification To Re-run
- cargo run -q -p kidcli -- oracle-probe --scenario .cache/oracle-probe/boot_title_start_level1_dense.json --frame-start 330 --frame-end 360 --wram-dump-frames 330,340,350,360
- cargo test -q -p kid-core boot_title_start_mode_8_keeps_advancing_rng_before_gameplay_entry -- --nocapture
- cargo test -q -p kid-core boot_title_start_stays_in_mode_8_without_second_start_press -- --nocapture
- cargo test -q -p kid-core level_entry_has_no_damage_grace_window -- --nocapture
- cargo test -q -p kidcli compare_native_states_reports_boot_level_entry_mode_8_release_gate -- --nocapture
- cargo run -q -p kidcli -- compare-native-oracle --scenario boot-title-start-level1

# Dead Ends / Do Not Retry
- Do not keep re-probing frames 330..360; the roster is stable there.
- Do not look for a late-title hidden RNG caller in `0x1BB50`, `0x1B6A6`, `0x1B650`, or `0x1BAF6`; those handlers are already mapped to RNG-silent ASM.
- Do not widen the dump range arbitrarily. The next useful probe is the earliest frame before this stable four-object shell appears.
