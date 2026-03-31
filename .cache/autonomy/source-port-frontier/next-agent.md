---
slug: release-gate-md-runtime-ctrl-pressed-mode8
cycle: 36
updated_at: 2026-03-31T04:35:56Z
---
# What Changed
- Probed the cold-boot title path all the way back to frame 0, then tightened the two transition windows around frames 78..100 and 160..200.
- Found the first live frame for the pre-shell title lineage:
  - frames 0..30: no live object list
  - frame 40: one live object at `0xFFFF0DA0`, code `0x0001B8F4`
  - frame 70: object list empty again
  - frame 78: `sub_1B8F4` is live at `0xFFFF0DA0`
  - frame 79: `sub_1C204` fade-in (`0x0001C222`) plus the new text worker `loc_1C7A0`
  - frame 99: the main slot promotes from `sub_1C204` into `loc_1B924` (`0x0001B928`)
- Tightened the later delete/rebuild boundary:
  - frames 160..164: `loc_1B924` + `loc_1C7B2`
  - frame 165: the old two-object shell contents are cleared
  - frames 169..190: `Addr_FirstObjectSlot = 0`, list still empty
  - frame 195: rebuilt shell appears, but with a transient fifth head object at `0xFFFF0F70 -> 0x0001C7B2`
  - frame 199: shell settles to the known four-object roster
- Read the ASM for the new transition labels:
  - `sub_1B8F4`, `loc_1B920`, `loc_1B924`
  - `sub_1C204`, `loc_1C21E`, `loc_1C234`, `loc_1C240`
  - `loc_1C7A0`, `loc_1C7B6`

# Current Model
- The cold-boot title path is now split into two verified sub-transitions:
  - initial spawn: `sub_1B8F4` appears by frame 40, reappears at frame 78, then `sub_1C204` drives the fade-in until `loc_1B924` takes over at frame 99
  - later rebuild: the `loc_1B924` shell clears between frames 164 and 165, stays absent through frame 190, then the rebuilt title shell appears at frame 195 and stabilises by frame 199
- Everything traced so far on these title workers is still RNG-silent in the ASM.
- That leaves two realistic sources for the carried RNG offset:
  - some earlier cold-boot caller before `sub_1B8F4` first appears
  - or a non-title caller on the mode transition that re-enters the shell after frame 165

# Strongest Evidence
- Oracle probes:
  - `cargo run -q -p kidcli -- oracle-probe --scenario .cache/oracle-probe/boot_title_start_level1_dense.json --frame-start 0 --frame-end 80 --wram-dump-frames 0,10,20,30,40,50,60,70,79,80`
  - `cargo run -q -p kidcli -- oracle-probe --scenario .cache/oracle-probe/boot_title_start_level1_dense.json --frame-start 78 --frame-end 100 --wram-dump-frames 78,79,80,81,82,83,84,85,86,87,88,89,90,95,99,100`
  - `cargo run -q -p kidcli -- oracle-probe --scenario .cache/oracle-probe/boot_title_start_level1_dense.json --frame-start 160 --frame-end 200 --wram-dump-frames 160,161,162,163,164,165,166,167,168,169,170,171,172,173,174,175,176,177,178,179,180,185,190,195,199,200`
- Key WRAM artefacts:
  - `.cache/oracle-probe/oracle/wram_frame_78.bin`
  - `.cache/oracle-probe/oracle/wram_frame_79.bin`
  - `.cache/oracle-probe/oracle/wram_frame_99.bin`
  - `.cache/oracle-probe/oracle/wram_frame_165.bin`
  - `.cache/oracle-probe/oracle/wram_frame_195.bin`
  - `.cache/oracle-probe/oracle/wram_frame_199.bin`
- Supporting ASM:
  - `.cache/upstream/kid-chameleon-disasm/kid.asm:30220-30238` `sub_1B8F4`, `loc_1B920`, `loc_1B924`
  - `.cache/upstream/kid-chameleon-disasm/kid.asm:31083-31112` `sub_1C204`, `loc_1C21E`, `loc_1C234`, `loc_1C240`
  - `.cache/upstream/kid-chameleon-disasm/kid.asm:31404-31436` `loc_1C7A0`, `loc_1C7B6`

# Exact Next Moves
- Stay narrow: read the ASM immediately before `sub_1B8F4` on the cold-boot path and identify which mode/setup routine owns the frame-40 first spawn and the frame-70 clear/re-enter. Then use one more WRAM probe only if needed to pin that caller’s first live frame.
- If that owner routine is still RNG-silent, move one step earlier again. Do not skip ahead to unrelated title handlers.

# Verification To Re-run
- cargo run -q -p kidcli -- oracle-probe --scenario .cache/oracle-probe/boot_title_start_level1_dense.json --frame-start 330 --frame-end 360 --wram-dump-frames 330,340,350,360
- cargo test -q -p kid-core boot_title_start_mode_8_keeps_advancing_rng_before_gameplay_entry -- --nocapture
- cargo test -q -p kid-core boot_title_start_stays_in_mode_8_without_second_start_press -- --nocapture
- cargo test -q -p kid-core level_entry_has_no_damage_grace_window -- --nocapture
- cargo test -q -p kidcli compare_native_states_reports_boot_level_entry_mode_8_release_gate -- --nocapture
- cargo run -q -p kidcli -- compare-native-oracle --scenario boot-title-start-level1

# Dead Ends / Do Not Retry
- Do not re-probe 79..164 as if the transition were still unknown; that window is now bounded to `sub_1C204` then `loc_1B924`.
- Do not blame `sub_1B8F4`, `sub_1C204`, `loc_1B924`, or `loc_1C7A0` as direct RNG callers without new ASM evidence; the traced paths are RNG-silent.
- Do not treat frame 195's transient fifth head object as a new mystery frontier yet; first explain the owner routine that spawned and cleared the earlier `sub_1B8F4` lineage.
