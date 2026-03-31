---
slug: release-gate-md-runtime-ctrl-pressed-mode8
cycle: 18
updated_at: 2026-03-31T03:05:56Z
---
# What Changed
- Traced the native title-card destination data path from `level/levelnames.asm` through `parse_level_title_cards`.
- Confirmed native currently lifts the `TitleTextLayout_*` rows straight into `LevelTitleCard.line_destinations` as absolute `(x,y)` targets.
- Found the missing ROM-side geometry step in `Load_TitleCard`: before `loc_1AB26` ever spawns the worker objects, the title-card background is centred via `TitleCardSize_Index`, with the horizontal tile offset carried in `d7`.

# Current Model
- The likely geometry miss is no longer "bad parsed rows" but "missing centring offset".
- `TitleTextLayout_3Lines` really does store `$20/$28`, `$20/$48`, `$20/$68`, but `Load_TitleCard` also computes a theme-specific centre offset from `TitleCardSize_Index` before the level-entry shell appears.
- Native `starting_level_title_workers` currently targets the raw layout rows without that ROM centring context, which explains why the oracle image's active glyphs sit far to the right of the native model.

# Strongest Evidence
- `.cache/upstream/kid-chameleon-disasm/level/levelnames.asm:170-178` documents `TitleTextLayout_3Lines` as `$20,$28 / $20,$48 / $20,$68 / $DC,$50`; the parser in `crates/kid-data/src/lib.rs:1712-1782` copies those rows directly.
- ASM `.cache/upstream/kid-chameleon-disasm/kid.asm:28654-28718` `Load_TitleCard` computes `d6=(#$1C-height)/2` and `d7=(#$28-width)/2` from `TitleCardSize_Index` before it finishes laying out the centred title-card map and reaches `loc_19C02`.
- For Level 1's theme, `TitleCardSize_Index` entry 8 is `$12,$0C`, so `d7=(40-18)/2 = 11 tiles = 88 px` of horizontal centring. That is the same scale as the native/oracle X gap seen at frame 520.
- Oracle image `.cache/native-oracle-runs/boot_title_start_level1/oracle/0520_level_entry.png` still shows the live glyph stack on the right-hand title-card columns, which the raw `$20/$40/$60` native targets cannot explain.

# Exact Next Moves
- Stay on `loc_1AC70`: test the centring-offset hypothesis against ASM by reconciling `TitleCardSize_Index` / `d7` with the native title-card destinations, then patch native only if the adjusted targets line up with the oracle frame-520 shell.
- Re-run `cargo run -q -p kidcli -- compare-native-oracle --scenario boot-title-start-level1` after any destination-map change; if the right-side shell moves into place, then revisit worker timing/RNG.
- Do not go back to the raw pre-roll theory, and do not patch elapsed frames first.

# Verification To Re-run
- cargo test -q -p kid-core boot_title_start_stays_in_mode_8_without_second_start_press -- --nocapture
- cargo test -q -p kid-core level_entry_has_no_damage_grace_window -- --nocapture
- cargo test -q -p kidcli compare_native_states_reports_boot_level_entry_mode_8_release_gate -- --nocapture
- cargo run -q -p kidcli -- compare-native-oracle --scenario boot-title-start-level1
- cargo run -q -p kidcli -- compare-native-oracle --scenario boot-title-start-level1-move-right5s

# Dead Ends / Do Not Retry
- Do not keep assuming the `TitleTextLayout_*` rows are already final screen coordinates.
- Do not revive the 44-frame pre-roll theory; `Load_TitleCard` allocates the shell directly.
- Do not patch elapsed frames or RNG order before the destination geometry matches the oracle image.
