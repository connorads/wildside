---
slug: release-gate-md-runtime-level-entry-title-card-ownership
cycle: 41
updated_at: 2026-03-31T05:17:00Z
---
# What Changed
- Inspected the `boot_title_start_level1` compare artefacts instead of widening to unrelated frontiers.
- Read the ROM around `Load_TitleCard`, `loc_1AB26`, `loc_1AD48`, and `loc_1AE2E`.
- Confirmed the `level_entry@520` oracle frame is still the mode-`$0008` title-card shell:
  - `Load_TitleCard` allocates `loc_1AB26` (letter workers) and `loc_1AE2E` (Start gate).
  - `loc_1AB26` expands the level-name glyph workers, then optionally allocates `loc_1AD48` for deferred level-number glyphs.
- Native is inconsistent on that shell: it traces slot-2 title-worker pose as “Kid”, but still reports zero camera, zero object counts, and `title_screen_ready=true`, unlike oracle.

# Current Model
- Oracle `level_entry@520` is not gameplay ownership yet. The captured PNG is the level title card, and oracle state still shows the title-card shell:
  - `game_mode=0x0008`
  - `number_objects=15`, `number_gfx_objects=14`
  - `camera_y_pos=0x00930000`
  - `title_screen_ready=false`
- Native `BootState::StartingLevel` currently mixes two models:
  - `current_player_state()` uses `starting_level_slot_2_pose(elapsed_frames)`, i.e. the `loc_1AB26` slot-2 glyph worker as “Kid”
  - `current_camera_state()` hard-returns zero for all `StartingLevel` frames
  - `title_screen_ready()` still uses the generic title-ready gate
- So the next fix is probably trace-facing, but the precise trace contract still needs one more ROM-oracle step: decide which starting-level fields should come from the title-card shell versus stale Kid WRAM.

# Strongest Evidence
- ASM:
  - `.cache/upstream/kid-chameleon-disasm/kid.asm:28718-28725` `Load_TitleCard` allocates `loc_1AB26` and `loc_1AE2E`
  - `.cache/upstream/kid-chameleon-disasm/kid.asm:28750-28895` `loc_1AB26`
  - `.cache/upstream/kid-chameleon-disasm/kid.asm:28989-29006` `loc_1AD48`
  - `.cache/upstream/kid-chameleon-disasm/kid.asm:29057-29064` `loc_1AE2E`
- Compare artefacts:
  - `.cache/native-oracle-runs/boot_title_start_level1/native/state.json`
  - `.cache/native-oracle-runs/boot_title_start_level1/oracle/manifest.json`
  - `.cache/native-oracle-runs/boot_title_start_level1/oracle/0520_level_entry.png`
- Native mismatch source:
  - `crates/kid-core/src/trace.rs:1359-1361` `BootState::StartingLevel` uses `starting_level_slot_2_pose`
  - `crates/kid-core/src/trace.rs:1385-1387` `BootState::StartingLevel` hard-returns zero camera
  - `crates/kid-core/src/movement.rs:240-249` `starting_level_slot_2_pose`

# Exact Next Moves
- Stay narrow: read the oracle helper field mapping for mode-`$0008` and decide which `StartingLevel` trace fields should mirror raw WRAM rather than the synthetic title-worker model.
- Then patch only the smallest coherent trace mismatch for the title-card shell, starting with the field that causes the first authoritative compare failure at `level_entry@520`.
- Re-run `compare-native-oracle --scenario boot-title-start-level1` immediately after the patch. If the first failure moves, capture that in the next handoff instead of guessing through multiple fields at once.

# Verification To Re-run
- cargo run -q -p kidcli -- oracle-probe --scenario .cache/oracle-probe/boot_title_start_level1_dense.json --frame-start 330 --frame-end 360 --wram-dump-frames 330,340,350,360
- cargo test -q -p kid-core boot_title_start_mode_8_keeps_advancing_rng_before_gameplay_entry -- --nocapture
- cargo test -q -p kid-core boot_title_start_stays_in_mode_8_without_second_start_press -- --nocapture
- cargo test -q -p kid-core level_entry_has_no_damage_grace_window -- --nocapture
- cargo test -q -p kidcli compare_native_states_reports_boot_level_entry_mode_8_release_gate -- --nocapture
- cargo run -q -p kidcli -- compare-native-oracle --scenario boot-title-start-level1

# Dead Ends / Do Not Retry
- Do not go back to searching for a hidden early mode-`$0004` jump; `sub_1C246` already answered that.
- Do not treat `level_entry@520` as gameplay ownership; the ROM and captured PNG both say it is still the title-card shell.
- Do not batch multiple starting-level trace fixes at once. Move the first authoritative compare failure, then stop and rewrite the handoff.
