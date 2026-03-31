---
slug: release-gate-md-runtime-ctrl-pressed-mode8
cycle: 31
updated_at: 2026-03-31T04:22:12Z
---
# What Changed
- Added `RomRandomState::next_long()` and used it to mirror ASM `Execute_Objects`' per-frame `Get_RandomNumber_long` burn while native stays in `BootState::StartingLevel`.
- Let the hidden gameplay slice keep stepping during mode 8 so `loc_1AE2E` only gates the later visible `Game_Mode=$000C` transition, not the underlying level shell.
- Added a regression test proving mode-8 RNG no longer freezes:
  - `boot_title_start_mode_8_keeps_advancing_rng_before_gameplay_entry`
- Verification now shows:
  - native frame 360 `rng_offset = 0xFFC8` (`-56`) unchanged
  - native frame 520 `rng_offset = 0xFFC8` (`-56`)
  - native frame 719 `rng_offset = 0xFFD4` (`-44`)
  - oracle remains `-27`, `-5`, `-34`

# Current Model
- The mode-8 freeze was real: native previously stopped advancing RNG after the eager `Load_TitleCard -> loc_1AB26/loc_1AD48` worker rebuild.
- Burning ASM `Execute_Objects` during `StartingLevel` fixes that freeze, but it is not the full ROM model:
  - it does nothing for the pre-mode-8 carried title offset, so frame 360 still sits on the fresh seed
  - it overshoots mode-8 parity shape, leaving frame 520 too early in the stream (`-56` instead of oracle `-5`)
- So the remaining bug is now narrower:
  - identify the real boot/title object lineage that advances RNG before frame 360
  - replace the coarse mode-8 long burn with the actual mode-8 object/activity mix

# Strongest Evidence
- ASM:
  - `.cache/upstream/kid-chameleon-disasm/kid.asm:1106-1229` `Init_RNG`, `Get_RandomNumber_byte`, `Get_RandomNumber_word`, `Get_RandomNumber_long`
  - `.cache/upstream/kid-chameleon-disasm/kid.asm:1636-1661` `Execute_Objects`
  - `.cache/upstream/kid-chameleon-disasm/kid.asm:28915-28958` `loc_1AC70` title-card worker RNG setup
  - `.cache/upstream/kid-chameleon-disasm/kid.asm:29072-29080` `loc_1AE2E`
- Native files touched this cycle:
  - `crates/kid-core/src/lib.rs`
  - `crates/kid-core/src/types.rs`
  - `crates/kid-core/src/tests.rs`
- Verification artefacts:
  - `cargo run -q -p kidcli -- compare-native-oracle --scenario boot-title-start-level1`
  - `.cache/native-oracle-runs/boot_title_start_level1/native/state.json`
  - `.cache/native-oracle-runs/boot_title_start_level1/oracle/manifest.json`

# Exact Next Moves
- Read the ASM that actually runs on the cold-boot title path before frame 360 and find which object/helpers consume RNG there; do not guess from the offset delta.
- Then read the real mode-8 object flow after `Load_TitleCard` and replace the coarse `Execute_Objects`-only burn with the specific object activity that explains oracle `rng_offset` moving `-5 -> -34` between frames 520 and 719.
- Keep the new regression test, but expect to tighten it toward oracle-backed checkpoint values once the real boot/title lineage is understood.

# Verification To Re-run
- cargo test -q -p kid-core boot_title_start_mode_8_keeps_advancing_rng_before_gameplay_entry -- --nocapture
- cargo test -q -p kid-core boot_title_start_stays_in_mode_8_without_second_start_press -- --nocapture
- cargo test -q -p kid-core level_entry_has_no_damage_grace_window -- --nocapture
- cargo test -q -p kidcli compare_native_states_reports_boot_level_entry_mode_8_release_gate -- --nocapture
- cargo run -q -p kidcli -- compare-native-oracle --scenario boot-title-start-level1
- jq '.[] | select(.frame_index==360 or .frame_index==520 or .frame_index==719) | {frame:.frame_index,label:.label,rng_offset:.rng_offset}' .cache/native-oracle-runs/boot_title_start_level1/native/state.json
- jq '.states[] | select(.frame_index==360 or .frame_index==520 or .frame_index==719) | {frame:.frame_index,label:.label,rng_offset:.rng_offset}' .cache/native-oracle-runs/boot_title_start_level1/oracle/manifest.json

# Dead Ends / Do Not Retry
- Do not treat the new `Execute_Objects` burn as the final fix. It only proves the freeze source.
- Do not go back to slot-2 pose-only reasoning while frame-360/title RNG lineage is still wrong.
- Do not assume every mode-8 frame should burn one random long forever; oracle `-5 -> -34` does not match a pure `+4 bytes/frame` model.
