---
slug: release-gate-md-runtime-level-entry-slot2-rng-blocker
cycle: 42
updated_at: 2026-03-31T05:27:00Z
---
# What Changed
- Read the ROM around the title-menu `Game_Mode=$0008` hand-off instead of guessing from native timing.
- Confirmed the title-menu start path is immediate:
  - gameplay `Teleport` / `loc_74E0` and the title menu both write `Game_Mode=$0008` and jump straight to `loc_6E2`
  - there is no extra hidden delay between the title-menu Start and `Load_TitleCard`
- That rules out the easy “native begins StartingLevel too early” theory.

# Current Model
- The authoritative `level_entry@520` mismatch is still the mode-`$0008` title-card shell’s slot-2 pose, not gameplay ownership.
- Two concrete causes remain alive:
  - native’s `StartingLevel` trace contract is wrong for some fields (`title_screen_ready`, camera, object counts)
  - the synthetic `loc_1AB26` slot-2 worker pose itself is wrong because the RNG-backed title-worker setup or motion model diverges before frame 520
- The ROM now makes one thing clear: the shell starts immediately on the title-menu Start, so the remaining ambiguity is not “when mode 8 starts?” but “why does native’s slot-2 worker land at `0089A000,00262000` while oracle’s WRAM-backed slot 2 is `007F0000,00020000`?”

# Strongest Evidence
- ASM:
  - `.cache/upstream/kid-chameleon-disasm/kid.asm:14778-14827` `Teleport`, `loc_B8DE`
  - `.cache/upstream/kid-chameleon-disasm/kid.asm:7643-7649` `loc_74E0`
  - `.cache/upstream/kid-chameleon-disasm/kid.asm:28718-28725` `Load_TitleCard`
  - `.cache/upstream/kid-chameleon-disasm/kid.asm:28750-29064` `loc_1AB26`, `loc_1AD48`, `loc_1AE2E`
- Compare artefacts:
  - `.cache/native-oracle-runs/boot_title_start_level1/native/state.json`
  - `.cache/native-oracle-runs/boot_title_start_level1/oracle/manifest.json`
  - `.cache/native-oracle-runs/boot_title_start_level1/oracle/0520_level_entry.png`
- Native sources implicated:
  - `crates/kid-core/src/trace.rs:1359-1361` `starting_level_slot_2_pose`
  - `crates/kid-core/src/trace.rs:1385-1387` zero `StartingLevel` camera
  - `crates/kid-core/src/movement.rs:124-249` title-worker rebuild + slot-2 pose model

# Exact Next Moves
- Real blocker: add one targeted oracle/native correlation for the title-card shell before changing code again.
- Specifically, capture or expose the mode-`$0008` slot-2 / RNG state around the title-menu Start and early `Load_TitleCard` lifetime so the next cycle can decide whether the first fix belongs in:
  - the RNG-backed `loc_1AB26` worker setup/motion model, or
  - the `StartingLevel` trace field mapping around that shell.
- Do not patch both areas blind. The ROM no longer leaves room for a timing guess; the next step needs one decisive measurement.

# Verification To Re-run
- cargo run -q -p kidcli -- oracle-probe --scenario .cache/oracle-probe/boot_title_start_level1_dense.json --frame-start 330 --frame-end 360 --wram-dump-frames 330,340,350,360
- cargo test -q -p kid-core boot_title_start_mode_8_keeps_advancing_rng_before_gameplay_entry -- --nocapture
- cargo test -q -p kid-core boot_title_start_stays_in_mode_8_without_second_start_press -- --nocapture
- cargo test -q -p kid-core level_entry_has_no_damage_grace_window -- --nocapture
- cargo test -q -p kidcli compare_native_states_reports_boot_level_entry_mode_8_release_gate -- --nocapture
- cargo run -q -p kidcli -- compare-native-oracle --scenario boot-title-start-level1

# Dead Ends / Do Not Retry
- Do not resurrect the “mode 8 starts late” theory; the ROM writes `Game_Mode=$0008` and jumps to `loc_6E2` immediately.
- Do not patch `title_screen_ready`, camera, and slot-2 pose together. That hides the causal field.
- Do not leave the blocker implicit. The next safe step is a single decisive probe, not another speculative trace tweak.
