---
slug: release-gate-md-runtime-ctrl-pressed-mode8
cycle: 30
updated_at: 2026-03-31T04:15:02Z
---
# What Changed
- Added a typed native `rng_offset` trace field sourced from `RomRandomState`, and threaded it through `compare-native-oracle` output.
- This removed the last big ambiguity in the title-entry RNG model: native and oracle can now be compared directly on the ROM-style `RNG_Offset` word instead of only through slot-2 poses.
- Verified the new parity gap:
  - oracle frame 360 `rng_offset = 0xFFE5` (`-27`)
  - native frame 360 `rng_offset = 0xFFC8` (`-56`)
  - oracle frame 520 `rng_offset = 0xFFFB` (`-5`)
  - native frame 520 `rng_offset = 0xFFEC` (`-20`)
  - oracle frame 719 `rng_offset = 0xFFDE` (`-34`)
  - native frame 719 `rng_offset = 0xFFEC` (`-20`)
- That proves native still does two wrong things at once:
  - it starts title entry from the fresh seed instead of the boot-carried offset
  - once frame-460 eager seeding finishes, its `rng_offset` stops moving entirely through the rest of mode 8

# Current Model
- The frame-520 bug is now pinned to RNG lineage, not just pose maths.
- Native behaviour:
  - frame 360 title: `rng_offset=-56` (still seed position)
  - frame 460 eager title-worker rebuild consumes local bytes immediately
  - frame 520 and 719 both stay at `rng_offset=-20`, showing no further RNG consumption during level-entry mode
- Oracle behaviour:
  - title is already on a carried boot-global RNG offset by frame 360 (`-27`)
  - mode 8 continues consuming RNG between frame 520 and frame 719 (`-5 -> -34`)
- So the next fix must do more than seed workers later. Native needs both:
  - carried RNG state before mode 8 starts
  - continued RNG advancement during level-entry execution

# Strongest Evidence
- ASM:
  - `.cache/upstream/kid-chameleon-disasm/kid.asm:1106-1216` `Init_RNG`, `Get_RandomNumber_byte`, `Get_RandomNumber_word`
  - `.cache/upstream/kid-chameleon-disasm/kid.asm:1636-1661` `Execute_Objects`
  - `.cache/upstream/kid-chameleon-disasm/kid.asm:30399-30458` `Title_InputLoop`
  - `.cache/upstream/kid-chameleon-disasm/kid.asm:28588-29016` `Load_TitleCard`, `loc_1AB26`, `loc_1AC70`, `loc_1AD48`, `loc_1AE2E`
- Native code changes this cycle:
  - `crates/kid-core/src/types.rs`
  - `crates/kid-core/src/trace.rs`
  - `crates/kidcli/src/compare.rs`
  - constructor fallout updated in `crates/kidcli/src/native_player/mod.rs`, `crates/kidcli/src/recording.rs`, `crates/kidcli/src/sweep.rs`, `crates/kidcli/src/tas_diff.rs`, `crates/kidcli/src/main.rs`
- Verification artefacts:
  - `cargo run -q -p kidcli -- compare-native-oracle --scenario boot-title-start-level1`
  - `.cache/native-oracle-runs/boot_title_start_level1/native/state.json`
  - `.cache/native-oracle-runs/boot_title_start_level1/oracle/manifest.json`

# Exact Next Moves
- Use the new `rng_offset` parity target to implement the actual title-entry fix: carry native RNG consumption across the boot/title path so frame 360 no longer sits at `-56`, then add the missing mode-8 RNG advancement so frame 520/719 stop freezing at `-20`. Only once `rng_offset` tracks closer to oracle should you revisit slot-2 worker ordering/pose.

# Verification To Re-run
- cargo test -q -p kid-core boot_title_start_stays_in_mode_8_without_second_start_press -- --nocapture
- cargo test -q -p kid-core level_entry_has_no_damage_grace_window -- --nocapture
- cargo test -q -p kidcli compare_native_states_reports_boot_level_entry_mode_8_release_gate -- --nocapture
- cargo run -q -p kidcli -- compare-native-oracle --scenario boot-title-start-level1
- jq '.[] | select(.frame_index==360 or .frame_index==520 or .frame_index==719) | {frame:.frame_index,label:.label,rng_offset:.rng_offset}' .cache/native-oracle-runs/boot_title_start_level1/native/state.json
- jq '.states[] | select(.frame_index==360 or .frame_index==520 or .frame_index==719) | {frame:.frame_index,label:.label,rng_offset:.rng_offset}' .cache/native-oracle-runs/boot_title_start_level1/oracle/manifest.json

# Dead Ends / Do Not Retry
- Do not go back to pose-only reasoning now that native `rng_offset` is available. Use the direct parity signal.
- Do not patch slot ordering before native carries the boot/title RNG lineage. The new offsets show the stream is still wrong earlier than slot 2.
- Do not assume mode 8 is RNG-idle after the eager frame-460 seed. Oracle `rng_offset` keeps moving through frame 719.
