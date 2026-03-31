---
slug: release-gate-md-runtime-ctrl-pressed-mode8
cycle: 15
updated_at: 2026-03-31T02:55:00Z
---
# What Changed
- Started the `loc_1AC70` parity pass by fixing native RNG consumption on the X-sampled shell edges: `Get_RandomNumber_word` now exists in `RomRandomState`, and title-card shell spawns use it where the ASM does.
- Added a direct RNG regression (`rom_random_state_matches_get_randomnumber_word_seeded_sequence`) so the native helper stays aligned with `kid.asm` `Get_RandomNumber_word`.
- Kept the iteration scoped to the RNG contract only; shell worker counts and deferred number workers were left untouched.

# Current Model
- The first authoritative divergence is still `level_entry@520`, but the native pose moved materially after the RNG fix: from `004EC000,00718000` down to `0042C000,005F8000` while oracle remains `007F0000,00020000`.
- That shift confirms `loc_1AC70` is the right surface and that native was previously consuming the wrong random source on the top/bottom-edge branches.
- The remaining gap is still inside `loc_1AC70`: exact spawn-edge coordinate selection is closer now, but the shell still does not match the oracle's top-edge slot-2 pose or its per-frame march to the target.

# Strongest Evidence
- `.cache/upstream/kid-chameleon-disasm/kid.asm:1210-1216` `Get_RandomNumber_word` is two successive `Get_RandomNumber_byte` calls packed into `(RNG_Buffer).w`; native now matches that helper exactly.
- `.cache/upstream/kid-chameleon-disasm/kid.asm:28924-28935` `loc_1AC70` uses `Get_RandomNumber_word` only on the branches that randomise X (`loc_1ACC8` / bottom-edge path), which was the missing native detail.
- `cargo test -q -p kid-core rom_random_state_matches_get_randomnumber_word_seeded_sequence -- --nocapture` passes with the seeded sequence `0x81C0, 0x9203, 0x97A3, 0x1C09`.
- `cargo run -q -p kidcli -- compare-native-oracle --scenario boot-title-start-level1`
  and
  `cargo run -q -p kidcli -- compare-native-oracle --scenario boot-title-start-level1-move-right5s`
  now fail first at `state level_entry@520 diverged in kid position (native 0042C000,005F8000; oracle 007F0000,00020000)`.

# Exact Next Moves
- Continue `loc_1AC70` only: verify the remaining branch math against the oracle frame-520 slot-2 pose, especially branch-to-edge mapping and the exact frame count used by the `loc_1AD2A` hibernate loop.
- Re-run `cargo test -q -p kid-core boot_title_start_stays_in_mode_8_without_second_start_press -- --nocapture`, `cargo run -q -p kidcli -- compare-native-oracle --scenario boot-title-start-level1`, then `cargo run -q -p kidcli -- compare-native-oracle --scenario boot-title-start-level1-move-right5s`.
- Do not touch worker allocation or spacing again unless the next `loc_1AC70` step exposes a concrete contradiction.

# Verification To Re-run
- cargo test -q -p kid-core boot_title_start_stays_in_mode_8_without_second_start_press -- --nocapture
- cargo test -q -p kid-core level_entry_has_no_damage_grace_window -- --nocapture
- cargo test -q -p kidcli compare_native_states_reports_boot_level_entry_mode_8_release_gate -- --nocapture
- cargo run -q -p kidcli -- compare-native-oracle --scenario boot-title-start-level1
- cargo run -q -p kidcli -- compare-native-oracle --scenario boot-title-start-level1-move-right5s

# Dead Ends / Do Not Retry
- Do not revert to `next_byte` for the X-sampled shell branches; ASM `Get_RandomNumber_word` is explicit and the compare delta proved it matters.
- Do not spend another cycle on allocation-level fixes. Worker count parity is already in place; the remaining miss is motion/RNG.
- Do not add new debug fields yet. The ASM still has enough structure left to follow directly.
