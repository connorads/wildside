---
slug: release-gate-md-runtime-ctrl-pressed-mode8
cycle: 19
updated_at: 2026-03-31T03:09:30Z
---
# What Changed
- Applied the first `Load_TitleCard` geometry fix in native: `starting_level_title_workers` now adds the ROM's horizontal title-card centring offset from `TitleCardSize_Index` when the level theme metadata is present.
- Updated the focused title-card worker tests to match the centred Level 1 destinations and fixed the stale worker-count assertion that still expected the pre-`loc_1AD48` shell length.
- Verified that the patch changes the live frontier: the frame-520 native X moved from `0042C000` to `00954000`, while Y stayed at `005F8000`.

# Current Model
- `d7` horizontal centring is real and was missing in native, but it only explains part of the frame-520 shell miss.
- The remaining divergence is now narrower: native has overshot past the oracle on X and is still far too low on Y, so either the slot/glyph mapping, vertical geometry, or the `loc_1AD2A` frame count is still off.
- Keep the centring patch. It improved parity materially and is ASM-backed.

# Strongest Evidence
- ASM `.cache/upstream/kid-chameleon-disasm/kid.asm:28654-28666` `Load_TitleCard` computes `d7=(#$28-width)/2` before the title shell appears; native now mirrors that horizontal offset.
- Focused verification is green:
  - `cargo test -q -p kid-core boot_title_start_level_1_expands_loc_1ab26_letter_workers -- --nocapture`
  - `cargo test -q -p kid-core boot_title_start_level_1_adds_loc_1ad48_deferred_number_worker -- --nocapture`
  - `cargo test -q -p kid-core boot_title_start_level_2_spaces_loc_1ad48_number_workers_by_16_pixels -- --nocapture`
  - `cargo test -q -p kid-core boot_title_start_isle_of_the_lion_lord_preserves_loc_1ab26_spacing_tokens -- --nocapture`
- Listed regression checks still pass except the frontier compares:
  - `cargo test -q -p kid-core boot_title_start_stays_in_mode_8_without_second_start_press -- --nocapture`
  - `cargo test -q -p kid-core level_entry_has_no_damage_grace_window -- --nocapture`
  - `cargo test -q -p kidcli compare_native_states_reports_boot_level_entry_mode_8_release_gate -- --nocapture`
- Frontier compare now fails at `state level_entry@520 diverged in kid position (native 00954000,005F8000; oracle 007F0000,00020000)` for both boot scenarios.

# Exact Next Moves
- Stay on `loc_1AC70`: determine whether the remaining miss is vertical geometry (`d6` / title-card Y anchoring), slot-to-glyph mapping, or the worker timeline by checking the centred destinations against the frame-520 oracle image before changing any more code.
- Re-run `cargo run -q -p kidcli -- compare-native-oracle --scenario boot-title-start-level1` and `...boot-title-start-level1-move-right5s` after the next focused hypothesis.
- Do not revert the horizontal centring patch, and do not jump back to the old raw-layout targets.

# Verification To Re-run
- cargo test -q -p kid-core boot_title_start_stays_in_mode_8_without_second_start_press -- --nocapture
- cargo test -q -p kid-core level_entry_has_no_damage_grace_window -- --nocapture
- cargo test -q -p kidcli compare_native_states_reports_boot_level_entry_mode_8_release_gate -- --nocapture
- cargo run -q -p kidcli -- compare-native-oracle --scenario boot-title-start-level1
- cargo run -q -p kidcli -- compare-native-oracle --scenario boot-title-start-level1-move-right5s

# Dead Ends / Do Not Retry
- Do not drop back to raw `TitleTextLayout_*` X coordinates; the ASM-backed `d7` offset clearly mattered.
- Do not resurrect the 44-frame pre-roll theory.
- Do not touch RNG order or elapsed-frame offsets until the remaining geometry/mapping ambiguity is narrowed.
