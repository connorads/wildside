---
slug: release-gate-md-runtime-ctrl-pressed-mode8
cycle: 17
updated_at: 2026-03-31T03:04:29Z
---
# What Changed
- Traced the caller side of the level-entry shell through `LoadGameModeData -> Load_TitleCard -> loc_19C02 -> loc_1AB26 -> loc_1AE2E`.
- Killed the "44-frame pre-roll before worker allocation" theory: `Load_TitleCard` is the direct `Game_Mode=$0008` loader, and `loc_19C02` allocates both `loc_1AB26` and `loc_1AE2E` immediately before returning.
- Checked the oracle frame image `0520_level_entry.png`; it shows the live letters on the right-hand title-card columns, which does not match the current native target map used in the earlier 16-step inversion.

# Current Model
- The current native model is still wrong, but not because `Load_TitleCard` waits 44 frames before building the shell.
- The stronger miss is now the shell geometry/model itself: the parser/native target coordinates feeding `starting_level_title_workers` do not match the actual frame-520 oracle layout.
- Because that target map is wrong, the earlier oracle "16 steps vs native 60" inversion is not trustworthy enough to justify an elapsed-frame patch.

# Strongest Evidence
- ASM `.cache/upstream/kid-chameleon-disasm/kid.asm:23560-23578` `LoadGameModeData` dispatches `Game_Mode=$0008` straight to `Load_TitleCard`.
- ASM `.cache/upstream/kid-chameleon-disasm/kid.asm:28588-28724` `Load_TitleCard` does its VRAM/tilemap work and then `loc_19C02` immediately allocates `loc_1AB26` and `loc_1AE2E`; there is no hidden 44-frame wait in that loader path.
- Oracle image `.cache/native-oracle-runs/boot_title_start_level1/oracle/0520_level_entry.png` visibly places the active glyphs on the right side of the title card (`B`, `W`, `K`, etc.), while the current native target map assumes left-column destinations like `(0x20,0x28)` / `(0x40,0x68)`.
- `cargo run -q -p kidcli -- compare-native-oracle --scenario boot-title-start-level1` still fails first at `state level_entry@520 diverged in kid position (native 0042C000,005F8000; oracle 007F0000,00020000)`.

# Exact Next Moves
- Stay on `loc_1AC70`: trace the actual title-card destination data path from `level/levelnames.asm` / `AddrTbl_LevelNames` into the native `title_card` parser, and reconcile why native thinks slot-2 targets `(0x40,0x68)` while the oracle frame clearly shows a right-side title-card column.
- Re-run `cargo run -q -p kidcli -- compare-native-oracle --scenario boot-title-start-level1` after identifying the correct destination map; only revisit elapsed-frame maths once the target coordinates match the oracle image.
- Do not revive the raw `-44` offset theory, and do not add probes yet.

# Verification To Re-run
- cargo test -q -p kid-core boot_title_start_stays_in_mode_8_without_second_start_press -- --nocapture
- cargo test -q -p kid-core level_entry_has_no_damage_grace_window -- --nocapture
- cargo test -q -p kidcli compare_native_states_reports_boot_level_entry_mode_8_release_gate -- --nocapture
- cargo run -q -p kidcli -- compare-native-oracle --scenario boot-title-start-level1
- cargo run -q -p kidcli -- compare-native-oracle --scenario boot-title-start-level1-move-right5s

# Dead Ends / Do Not Retry
- Do not keep treating `Game_Mode=$0008` as if it delayed `Load_TitleCard` for 44 frames. The ASM loader path is direct.
- Do not trust the earlier 16-step oracle inversion until the slot/glyph target map is fixed.
- Do not patch elapsed frames or RNG order while the geometry feeding `loc_1AC70` is still suspect.
