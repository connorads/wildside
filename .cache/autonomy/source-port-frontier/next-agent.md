---
slug: release-gate-md-runtime-ctrl-pressed-mode8
cycle: 35
updated_at: 2026-03-31T04:32:55Z
---
# What Changed
- Moved the same cold-boot title WRAM/object-list probe earlier, first to frames 280..330, then 220..280, then 140..220.
- Confirmed the late four-object shell is already stable by frame 200, not just frame 330:
  - `0xFFFF0EFC -> code 0x0001BB50`
  - `0xFFFF0E88 -> code 0x0001B6A6`
  - `0xFFFF0E14 -> code 0x0001B650`
  - `0xFFFF0DA0 -> code 0x0001BAF6`
- Found the earlier pre-shell lineage:
  - frames 140..160: head pointer `0xFFFF0E14`, two live objects only
  - `0xFFFF0E14 -> code 0x0001C7B2`
  - `0xFFFF0DA0 -> code 0x0001B928`
- Found a gap before the stable shell appears:
  - frames 170..190: `Addr_FirstObjectSlot = 0`, no live object list
  - frame 200: the stable four-object shell has appeared
- Read the cited ASM for the disappearing lineage:
  - `0x1B928` is inside `loc_1B924` in `sub_1B8F4`
  - `0x1C7B2` is inside `loc_1C7AE/loc_1C7B6` in `loc_1C7A0`
  - neither path makes a direct `Get_RandomNumber_*` call

# Current Model
- The carried cold-boot RNG offset still does not come from a late-title hidden object near frame 330 or 360.
- The last pre-shell lineage visible in WRAM is also RNG-silent: `loc_1B924` only hibernates and calls `sub_1C246`, while `loc_1C7A0` is the text worker.
- The meaningful transition is now bounded much tighter:
  - pre-shell two-object lineage at 140..160
  - empty list at 170..190
  - stable four-object shell by 200
- The likely remaining source is therefore earlier cumulative object-loop RNG burn before frame 140, or some non-object caller on the cold-boot intro path before the shell reset.

# Strongest Evidence
- Earlier probes:
  - `cargo run -q -p kidcli -- oracle-probe --scenario .cache/oracle-probe/boot_title_start_level1_dense.json --frame-start 280 --frame-end 330 --wram-dump-frames 280,290,300,310,320,329,330`
  - `cargo run -q -p kidcli -- oracle-probe --scenario .cache/oracle-probe/boot_title_start_level1_dense.json --frame-start 220 --frame-end 280 --wram-dump-frames 220,230,240,250,260,270,279,280`
  - `cargo run -q -p kidcli -- oracle-probe --scenario .cache/oracle-probe/boot_title_start_level1_dense.json --frame-start 140 --frame-end 220 --wram-dump-frames 140,150,160,170,180,190,200,210,219,220`
- WRAM artefacts:
  - `.cache/oracle-probe/oracle/wram_frame_140.bin`
  - `.cache/oracle-probe/oracle/wram_frame_160.bin`
  - `.cache/oracle-probe/oracle/wram_frame_170.bin`
  - `.cache/oracle-probe/oracle/wram_frame_190.bin`
  - `.cache/oracle-probe/oracle/wram_frame_200.bin`
- Supporting ASM:
  - `.cache/upstream/kid-chameleon-disasm/variables.asm:52-53` `Addr_FirstObjectSlot`, `Addr_CurrentObject`
  - `.cache/upstream/kid-chameleon-disasm/kid.asm:30220-30238` `sub_1B8F4`, `loc_1B920`, `loc_1B924`
  - `.cache/upstream/kid-chameleon-disasm/kid.asm:31089-31137` `sub_1C204`, `sub_1C246`
  - `.cache/upstream/kid-chameleon-disasm/kid.asm:31404-31436` `loc_1C7A0`, `loc_1C7AE`, `loc_1C7B6`

# Exact Next Moves
- Stay narrow: probe just before frame 140 on the same cold-boot path until you find the first frame where the `loc_1B924` + `loc_1C7A0` two-object lineage first appears, then trace what object or game-mode transition deleted the list between frames 160 and 170 and rebuilt the four-object shell by frame 200.
- Read the ASM at that transition before changing native code. If the transition itself is still RNG-silent, keep walking earlier rather than guessing.

# Verification To Re-run
- cargo run -q -p kidcli -- oracle-probe --scenario .cache/oracle-probe/boot_title_start_level1_dense.json --frame-start 330 --frame-end 360 --wram-dump-frames 330,340,350,360
- cargo test -q -p kid-core boot_title_start_mode_8_keeps_advancing_rng_before_gameplay_entry -- --nocapture
- cargo test -q -p kid-core boot_title_start_stays_in_mode_8_without_second_start_press -- --nocapture
- cargo test -q -p kid-core level_entry_has_no_damage_grace_window -- --nocapture
- cargo test -q -p kidcli compare_native_states_reports_boot_level_entry_mode_8_release_gate -- --nocapture
- cargo run -q -p kidcli -- compare-native-oracle --scenario boot-title-start-level1

# Dead Ends / Do Not Retry
- Do not keep re-probing 200..360; the same four-object shell is already stable there.
- Do not blame `0x1B928` / `loc_1B924` or `0x1C7B2` / `loc_1C7A0` as direct RNG callers; the ASM for both is RNG-silent.
- Do not guess from the empty-list gap at 170..190 alone; the next useful evidence is the transition into the two-object lineage before frame 140, plus the concrete 160..200 rebuild path.
