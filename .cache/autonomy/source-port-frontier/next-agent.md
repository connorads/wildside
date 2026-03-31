---
slug: release-gate-md-runtime-ctrl-pressed-mode8
cycle: 20
updated_at: 2026-03-31T03:15:02Z
---
# What Changed
- Fixed the traced level-entry shell slot mapping to follow the ROM's creation order: native `loc_1AB26` workers now expose `gfx_slot_index` in allocation order instead of the old reverse-count guess.
- Updated the focused slot-2 worker test to match the ASM-backed `u` glyph and centred top-row destination.
- Re-ran the listed checks. All focused tests stayed green, and both boot compares still fail at frame 520 but now from a much closer pose.

# Current Model
- The frame-520 miss was not just vertical geometry. Native had been tracing the wrong title glyph because `gfx_slot_index == 2` was wired to the third-row `o` instead of the third allocated top-row letter.
- After the slot-order fix, frame 520 moved from `00954000,005F8000` to `008C8000,00268000` while the oracle remains `007F0000,00020000`.
- The remaining miss is now smaller and likely sits in `loc_1AC70` motion itself: either the 64-frame `loc_1AD2A` timeline, the start-position RNG consumption, or the Y anchor for the top-row `u`.

# Strongest Evidence
- ASM `.cache/upstream/kid-chameleon-disasm/kid.asm:28750-28824` `loc_1AB26` only allocates title objects in script order; it never assigns a reverse slot index.
- ASM `.cache/upstream/kid-chameleon-disasm/kid.asm:1845-1911` `Allocate_GfxObjectSlot` inserts equal-priority gfx objects after existing ones, so traced slot order follows creation order for the level-entry letters.
- Verification:
  - `cargo test -q -p kid-core boot_title_start_level_1_expands_loc_1ab26_letter_workers -- --nocapture`
  - `cargo test -q -p kid-core boot_title_start_level_1_adds_loc_1ad48_deferred_number_worker -- --nocapture`
  - `cargo test -q -p kid-core boot_title_start_stays_in_mode_8_without_second_start_press -- --nocapture`
  - `cargo test -q -p kid-core level_entry_has_no_damage_grace_window -- --nocapture`
  - `cargo test -q -p kidcli compare_native_states_reports_boot_level_entry_mode_8_release_gate -- --nocapture`
- Frontier compares now fail at:
  - `boot-title-start-level1`: `state level_entry@520 diverged in kid position (native 008C8000,00268000; oracle 007F0000,00020000)`
  - `boot-title-start-level1-move-right5s`: same frame-520 pose

# Exact Next Moves
- Stay on `loc_1AC70`: read the random-entry branches at `.cache/upstream/kid-chameleon-disasm/kid.asm:28896-28972` against the frame-520 oracle image/state and determine whether slot-2 `u` still differs because native consumes the wrong start-selector/RNG path or because the 64-frame movement timeline is off.
- Re-run `cargo run -q -p kidcli -- compare-native-oracle --scenario boot-title-start-level1` and `cargo run -q -p kidcli -- compare-native-oracle --scenario boot-title-start-level1-move-right5s` after the next focused hypothesis.
- Do not revisit the reverse slot mapping or the pre-centred raw layout targets.

# Verification To Re-run
- cargo test -q -p kid-core boot_title_start_stays_in_mode_8_without_second_start_press -- --nocapture
- cargo test -q -p kid-core level_entry_has_no_damage_grace_window -- --nocapture
- cargo test -q -p kidcli compare_native_states_reports_boot_level_entry_mode_8_release_gate -- --nocapture
- cargo run -q -p kidcli -- compare-native-oracle --scenario boot-title-start-level1
- cargo run -q -p kidcli -- compare-native-oracle --scenario boot-title-start-level1-move-right5s

# Dead Ends / Do Not Retry
- Do not restore the reverse `visible_glyph_count - index - 1` slot mapping; ASM `loc_1AB26` plus `Allocate_GfxObjectSlot` disproved it.
- Do not drop back to raw `TitleTextLayout_*` X coordinates; the ASM-backed `d7` centring still matters.
- Do not resurrect the 44-frame pre-roll theory.
- Do not touch RNG order outside `loc_1AC70` until the remaining frame-520 miss is narrowed against the ASM branch logic.
