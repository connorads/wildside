---
slug: release-gate-md-runtime-ctrl-pressed-mode8
cycle: 21
updated_at: 2026-03-31T03:17:34Z
---
# What Changed
- Read `loc_1AC70` again with the frame-520 artefacts and narrowed the remaining miss without changing gameplay code.
- Re-ran the listed checks; they remain stable, and both boot compares still fail at the same frame-520 pose `008C8000,00268000`.
- The fresh conclusion is that the remaining error looks like level-entry timing, not a wrong `loc_1AC70` branch choice.

# Current Model
- The remaining slot-2 `u` miss is unlikely to be the random-entry branch. Oracle `y=0x00020000` is incompatible with a nearly completed 64-frame glide to target `y=0x00280000`, regardless of start branch, but it is consistent with the top-entry branch while only about half the native timeline has elapsed.
- Both boot scenarios still converge to the same wrong pose, which also argues against a scenario-specific input/RNG branch issue.
- The next likely fault is earlier than `loc_1AC70` movement math itself: native is probably entering `StartingLevel` or exposing the slot-2 trace too early relative to the ROM.

# Strongest Evidence
- ASM `.cache/upstream/kid-chameleon-disasm/kid.asm:28750-28824` `loc_1AB26` only allocates title objects in script order; it never assigns a reverse slot index.
- ASM `.cache/upstream/kid-chameleon-disasm/kid.asm:1845-1911` `Allocate_GfxObjectSlot` inserts equal-priority gfx objects after existing ones, so traced slot order follows creation order for the level-entry letters.
- ASM `.cache/upstream/kid-chameleon-disasm/kid.asm:28896-28972` `loc_1AC70` picks one of four start branches, but only the top-entry branch (`loc_1ACEA`, `d6=-$20`) can plausibly pair with the oracle's near-top frame-520 `u` pose; a 64-frame glide from any branch would already be much closer to the centred target than the oracle shows.
- Verification:
  - `cargo test -q -p kid-core boot_title_start_stays_in_mode_8_without_second_start_press -- --nocapture`
  - `cargo test -q -p kid-core level_entry_has_no_damage_grace_window -- --nocapture`
  - `cargo test -q -p kidcli compare_native_states_reports_boot_level_entry_mode_8_release_gate -- --nocapture`
- Frontier compares now fail at:
  - `boot-title-start-level1`: `state level_entry@520 diverged in kid position (native 008C8000,00268000; oracle 007F0000,00020000)`
  - `boot-title-start-level1-move-right5s`: same frame-520 pose

# Exact Next Moves
- Move one step earlier than `loc_1AC70`: read the title-menu to level-entry hand-off ASM (`loc_1AE2E` and the mode-8 entry path that reaches it) and determine why native's `StartingLevel.elapsed_frames` appears ahead of the oracle shell by roughly half the glide.
- Re-run `cargo run -q -p kidcli -- compare-native-oracle --scenario boot-title-start-level1` and `cargo run -q -p kidcli -- compare-native-oracle --scenario boot-title-start-level1-move-right5s` after the next focused hypothesis.
- Do not revisit the reverse slot mapping, the `loc_1AC70` branch-selection hypothesis, or the pre-centred raw layout targets.

# Verification To Re-run
- cargo test -q -p kid-core boot_title_start_stays_in_mode_8_without_second_start_press -- --nocapture
- cargo test -q -p kid-core level_entry_has_no_damage_grace_window -- --nocapture
- cargo test -q -p kidcli compare_native_states_reports_boot_level_entry_mode_8_release_gate -- --nocapture
- cargo run -q -p kidcli -- compare-native-oracle --scenario boot-title-start-level1
- cargo run -q -p kidcli -- compare-native-oracle --scenario boot-title-start-level1-move-right5s

# Dead Ends / Do Not Retry
- Do not restore the reverse `visible_glyph_count - index - 1` slot mapping; ASM `loc_1AB26` plus `Allocate_GfxObjectSlot` disproved it.
- Do not spend another cycle guessing `loc_1AC70` start branches from native probes; the current oracle pose already points to a timeline issue.
- Do not drop back to raw `TitleTextLayout_*` X coordinates; the ASM-backed `d7` centring still matters.
- Do not resurrect the 44-frame pre-roll theory.
- Do not touch RNG order outside `loc_1AC70` until the remaining frame-520 miss is narrowed against the ASM branch logic.
