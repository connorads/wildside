---
slug: release-gate-md-runtime-ctrl-pressed-mode8
cycle: 14
updated_at: 2026-03-31T02:47:30Z
---
# What Changed
- Modelled `loc_1ABF4 -> loc_1AD48` in `StartingLevel`: native now allocates the deferred `#$7B` level-number workers from `title_card.level_number_destination` and `level_number_glyph_count`, with `#$10` horizontal spacing between suffix glyphs.
- Kept the scope narrow: the new workers reuse the same shell-worker representation as the existing `loc_1AB26` letters, so slot 2 remains the same letter worker while the numeric suffix fills the next free gfx slots above the letters.
- Added regressions covering both the single-digit level-00 suffix (`boot_title_start_level_1_adds_loc_1ad48_deferred_number_worker`) and the two-digit level-01 spacing path (`boot_title_start_level_2_spaces_loc_1ad48_number_workers_by_16_pixels`).

# Current Model
- Content parity for the mode-8 shell is now complete at the allocation level: level 00 reaches 14 title-card workers in native, matching the oracle's 14 gfx objects by `level_entry` / `level_settle`.
- The first authoritative miss is unchanged at `level_entry@520` on kid position `004EC000,00718000` vs `007F0000,00020000`, so the remaining bug is inside `loc_1AC70`'s motion/RNG path rather than missing worker allocation.
- The native shell still uses direct interpolation and local RNG consumption; oracle uses `loc_1AC70`'s exact edge selection, secondary random coordinate, and per-frame object-loop updates.

# Strongest Evidence
- `.cache/upstream/kid-chameleon-disasm/kid.asm:28953-28972` `loc_1AD48` does exactly what native now mirrors: read one `(x,y)` pair from the saved layout pointer, allocate `#$7B` workers, and `addi.w #$10,d1` between each digit.
- `cargo test -q -p kid-core boot_title_start_level_1_adds_loc_1ad48_deferred_number_worker -- --nocapture` passes and confirms level 00 now allocates slot 13 at `(0xDC, 0x50)`.
- `cargo test -q -p kid-core boot_title_start_level_2_spaces_loc_1ad48_number_workers_by_16_pixels -- --nocapture` passes and confirms the level-01 two-digit suffix lands at `0xDC` and `0xEC`.
- `.cache/native-oracle-runs/boot_title_start_level1/oracle/manifest.json:287-487` shows oracle already has 14 gfx objects at `level_settle`, with slot 13 at `(0xDC, 0x50)` while slot 2 remains the third-row `o`.
- `cargo run -q -p kidcli -- compare-native-oracle --scenario boot-title-start-level1`
  and
  `cargo run -q -p kidcli -- compare-native-oracle --scenario boot-title-start-level1-move-right5s`
  still fail first at `state level_entry@520 diverged in kid position (native 004EC000,00718000; oracle 007F0000,00020000)`.

# Exact Next Moves
- Model `loc_1AC70` only: replace the current shell-worker interpolation with the ROM's exact spawn-edge selection, random axis sampling, apostrophe Y offset, and 64-frame per-axis march.
- Re-run `cargo test -q -p kid-core boot_title_start_stays_in_mode_8_without_second_start_press -- --nocapture`, `cargo run -q -p kidcli -- compare-native-oracle --scenario boot-title-start-level1`, then `cargo run -q -p kidcli -- compare-native-oracle --scenario boot-title-start-level1-move-right5s`.
- Do not revisit worker counts or number-worker spacing unless `loc_1AC70` work exposes a concrete allocation mismatch.

# Verification To Re-run
- cargo test -q -p kid-core boot_title_start_stays_in_mode_8_without_second_start_press -- --nocapture
- cargo test -q -p kid-core level_entry_has_no_damage_grace_window -- --nocapture
- cargo test -q -p kidcli compare_native_states_reports_boot_level_entry_mode_8_release_gate -- --nocapture
- cargo run -q -p kidcli -- compare-native-oracle --scenario boot-title-start-level1
- cargo run -q -p kidcli -- compare-native-oracle --scenario boot-title-start-level1-move-right5s

# Dead Ends / Do Not Retry
- Do not spend another cycle on `loc_1AD48`; the deferred number workers now exist and space correctly.
- Do not change slot ordering heuristically. Oracle still agrees that slot 2 is the third-row `o` and slot 13 is the deferred number worker.
- Do not keep the direct interpolation model once you touch motion. `loc_1AC70` updates through object state and RNG, so motion parity now requires following that routine directly.
