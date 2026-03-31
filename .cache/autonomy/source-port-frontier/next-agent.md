---
slug: release-gate-md-runtime-ctrl-pressed-mode8
cycle: 37
updated_at: 2026-03-31T04:37:58Z
---
# What Changed
- Followed `sub_1B8F4` upstream in the ASM instead of probing wider.
- Confirmed the cold-boot owner routine for the early shell:
  - `GameLoadArray` uses `Game_Mode` as a byte offset into the loader table
  - `Game_Mode = $0004` maps to `Load_IntroSequence1`
  - `Load_IntroSequence1` explicitly allocates `sub_1B8F4`
- Cross-checked the oracle WRAM for the mode values at the key shell frames:
  - frames 40, 70, 78, 79, 99: `Game_Mode = $0004`
  - frames 165 and 195: `Game_Mode = $002C`
- This means the frame-40 first spawn and the frame-70 clear/re-enter both happen under the same `Load_IntroSequence1` mode, while the later rebuilt shell is already under the title-screen loader mode.

# Current Model
- The early cold-boot shell is owned by `Load_IntroSequence1`, not some hidden side path.
- The object-list empty gap around frame 70 does not imply a mode change; `Game_Mode` stays at `$0004`, so the clear/re-enter must be explained by the intro-sequence-1 load/control flow itself.
- The later shell rebuild after frame 165 is on a different mode entirely: `Game_Mode = $002C`, which matches the title-screen loader slot.
- Since `Load_IntroSequence1` and its traced workers are still RNG-silent, the next useful target is the control flow that re-runs or re-arms that load while mode `$0004` remains live.

# Strongest Evidence
- Supporting ASM:
  - `.cache/upstream/kid-chameleon-disasm/kid.asm:23560-23581` `LoadGameModeData`, `GameLoadArray`
  - `.cache/upstream/kid-chameleon-disasm/kid.asm:29067-29074` `Load_IntroSequence1`
  - `.cache/upstream/kid-chameleon-disasm/kid.asm:30220-30238` `sub_1B8F4`, `loc_1B920`, `loc_1B924`
- WRAM mode checks:
  - `Game_Mode = $FFFFFBCA` from `.cache/upstream/kid-chameleon-disasm/variables.asm:127`
  - `.cache/oracle-probe/oracle/wram_frame_40.bin`
  - `.cache/oracle-probe/oracle/wram_frame_70.bin`
  - `.cache/oracle-probe/oracle/wram_frame_78.bin`
  - `.cache/oracle-probe/oracle/wram_frame_99.bin`
  - `.cache/oracle-probe/oracle/wram_frame_165.bin`
  - `.cache/oracle-probe/oracle/wram_frame_195.bin`
- Example mode dumps:
  - frames 40/70/78/79/99: `0xFBCA = 0004`
  - frames 165/195: `0xFBCA = 002C`

# Exact Next Moves
- Stay narrow: read the ASM control flow around `LoadGameModeData` and the main loader/flag path that can re-enter `Load_IntroSequence1` while `Game_Mode` stays `$0004`. Identify what clears the object list around frame 70 and causes `sub_1B8F4` to be allocated again by frame 78.
- Use one more WRAM probe only if the loader/control-flow ASM is still ambiguous after that read.

# Verification To Re-run
- cargo run -q -p kidcli -- oracle-probe --scenario .cache/oracle-probe/boot_title_start_level1_dense.json --frame-start 330 --frame-end 360 --wram-dump-frames 330,340,350,360
- cargo test -q -p kid-core boot_title_start_mode_8_keeps_advancing_rng_before_gameplay_entry -- --nocapture
- cargo test -q -p kid-core boot_title_start_stays_in_mode_8_without_second_start_press -- --nocapture
- cargo test -q -p kid-core level_entry_has_no_damage_grace_window -- --nocapture
- cargo test -q -p kidcli compare_native_states_reports_boot_level_entry_mode_8_release_gate -- --nocapture
- cargo run -q -p kidcli -- compare-native-oracle --scenario boot-title-start-level1

# Dead Ends / Do Not Retry
- Do not keep treating the frame-70 empty list as evidence of a hidden mode hop; WRAM shows `Game_Mode` is still `$0004`.
- Do not keep re-probing the already-bounded `sub_1C204 -> loc_1B924` window without first reading the loader/control-flow ASM that owns it.
- Do not jump ahead to the later `$002C` title-screen shell yet; the unresolved question is still why the `$0004` intro-sequence-1 load clears and re-enters before frame 79.
