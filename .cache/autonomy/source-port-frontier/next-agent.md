---
slug: release-gate-md-runtime-ctrl-pressed-mode8
cycle: 12
updated_at: 2026-03-31T02:23:15Z
---
# What Changed
- Extracted ROM title-card metadata from `level/levelnames.asm` into `kid-data`, so every `LevelRecord` now carries the exact `loc_1AB26` script bytes, line destinations, optional level-number destination, and `d0` glyph count.
- Bumped the generated bundle contract, regenerated `.cache/generated/28b904000b2863b6760531807760b571f1a5fc1d/asset-manifest.json`, and confirmed the manifest now serialises `title_card` data for level 00 and later entries.
- Kept the frontier stable: the mode-8 release gate tests still pass, and `compare-native-oracle` still fails first at `level_entry@520` on kid position because native still lacks the runtime `loc_1AB26` / `loc_1ABF4` shell.

# Current Model
- The asset-side blocker is cleared. Native now has the ROM data needed to build the level-entry title shell faithfully instead of reconstructing it from `display_name`.
- The first authoritative miss is unchanged: `boot-title-start-level1` and `boot-title-start-level1-move-right5s` still diverge at `level_entry@520` with native `kid 00000000,00000000` versus oracle `007F0000,00020000`.
- The next missing piece is runtime behaviour: `StartingLevel` still does not execute `loc_1AB26`'s letter fanout or `loc_1ABF4`'s deferred level-number worker, so native exposes an empty mode-8 shell.

# Strongest Evidence
- `.cache/upstream/kid-chameleon-disasm/kid.asm:28761-28852` `loc_1AB26 -> loc_1ABF4` reads `AddrTbl_LevelNames`, spawns one `loc_1AC70` worker per script byte sequence, then allocates `loc_1AD48` with the trailing `d0` count and remaining coordinate pointer.
- `.cache/upstream/kid-chameleon-disasm/kid.asm:28989-29009` `loc_1AD48` consumes exactly one more `(x,y)` pair and spawns repeated `#$7B` workers spaced `#$10` pixels apart for the level number.
- `.cache/generated/28b904000b2863b6760531807760b571f1a5fc1d/asset-manifest.json:122` now contains `title_card.script_bytes` and `level_number_glyph_count` for level 00; line 608 shows the same structure for level 01.
- Verification stayed stable:
  - `cargo test -q -p kid-core boot_title_start_stays_in_mode_8_without_second_start_press -- --nocapture`
  - `cargo test -q -p kid-core level_entry_has_no_damage_grace_window -- --nocapture`
  - `cargo test -q -p kidcli compare_native_states_reports_boot_level_entry_mode_8_release_gate -- --nocapture`
  - all pass
- Compare remains on the same frontier:
  - `cargo run -q -p kidcli -- compare-native-oracle --scenario boot-title-start-level1`
  - `cargo run -q -p kidcli -- compare-native-oracle --scenario boot-title-start-level1-move-right5s`
  - both fail at `state level_entry@520 diverged in kid position (native 00000000,00000000; oracle 007F0000,00020000)`

# Exact Next Moves
- Model `loc_1AB26` in `StartingLevel`: load the extracted `title_card` metadata, emit one native shell worker per script byte token, and preserve the ROM spacing rules for letters, apostrophes, spaces, and line breaks before touching later parity.
- After the letter fanout exists, model `loc_1ABF4 -> loc_1AD48`'s deferred level-number spawn using the extracted `level_number_destination` and `level_number_glyph_count`.
- Re-run `cargo test -q -p kid-core boot_title_start_stays_in_mode_8_without_second_start_press -- --nocapture`, `cargo run -q -p kidcli -- compare-native-oracle --scenario boot-title-start-level1`, then `cargo run -q -p kidcli -- compare-native-oracle --scenario boot-title-start-level1-move-right5s`.

# Verification To Re-run
- cargo test -q -p kid-core boot_title_start_stays_in_mode_8_without_second_start_press -- --nocapture
- cargo test -q -p kid-core level_entry_has_no_damage_grace_window -- --nocapture
- cargo test -q -p kidcli compare_native_states_reports_boot_level_entry_mode_8_release_gate -- --nocapture
- cargo run -q -p kidcli -- compare-native-oracle --scenario boot-title-start-level1
- cargo run -q -p kidcli -- compare-native-oracle --scenario boot-title-start-level1-move-right5s

# Dead Ends / Do Not Retry
- Do not go back to reconstructing the title shell from `display_name`; the ROM depends on script opcodes and layout-table state now available in `title_card`.
- Do not chase later mode-8 timings before the shell workers exist. The frontier is still missing content, not another countdown constant.
- Do not skip the deferred level-number path: `loc_1ABF4 -> loc_1AD48` owns the numeric suffix; it is not part of the main per-byte loop in `loc_1AB26`.
