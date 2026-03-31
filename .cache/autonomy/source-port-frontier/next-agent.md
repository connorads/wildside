---
slug: release-gate-md-runtime-ctrl-pressed-mode8
cycle: 23
updated_at: 2026-03-31T03:31:08Z
---
# What Changed
- Re-read `loc_1AC70 -> loc_1AD2A` and applied the visible one-frame hibernate that the ROM takes before the first title-glyph glide step.
- Updated native level-entry trace poses to treat the first `StartingLevel` frame as the untouched seeded start pose rather than step 1 of the glide.
- Re-ran the listed checks and both boot compares.

# Current Model
- Native `StartingLevel.elapsed_frames` was one frame too eager for the visible `loc_1AC70` shell. The first frame of the ROM object is the seeded start pose, not the first movement increment.
- That fix only moved the frontier by one shell step: frame 520 changed from native `008C8000,00268000` to `0089A000,00262000`, still well ahead of oracle `007F0000,00020000`.
- The remaining miss is therefore not just the `loc_1AD2A` initial hibernate. More of the mode-8 hand-off still starts too early in native before the visible slot-2 glide reaches the oracle timeline.

# Strongest Evidence
- ASM `.cache/upstream/kid-chameleon-disasm/kid.asm:28958-28978` `loc_1ACFE -> loc_1AD2A` writes `x_pos/y_pos`, computes `d2/d3`, then immediately calls `j_Hibernate_Object_1Frame` before the first `add.l d2,x_pos(a3)` / `add.l d3,y_pos(a3)` step.
- Verification:
  - `cargo test -q -p kid-core boot_title_start_stays_in_mode_8_without_second_start_press -- --nocapture`
  - `cargo test -q -p kid-core level_entry_has_no_damage_grace_window -- --nocapture`
  - `cargo test -q -p kidcli compare_native_states_reports_boot_level_entry_mode_8_release_gate -- --nocapture`
  - `cargo run -q -p kidcli -- compare-native-oracle --scenario boot-title-start-level1`
  - `cargo run -q -p kidcli -- compare-native-oracle --scenario boot-title-start-level1-move-right5s`
- The compares now fail at the same checkpoint but with the shifted native pose:
  - `boot-title-start-level1`: `state level_entry@520 diverged in kid position (native 0089A000,00262000; oracle 007F0000,00020000)`
  - `boot-title-start-level1-move-right5s`: same frame-520 pose

# Exact Next Moves
- Stay on the mode-8 hand-off and read the object-execution timing around `loc_6E2`, `Mode_Standard`, and the first live `Execute_Objects` pass to determine how many outer frames elapse before `loc_1AC70` should become trace-visible after the title Start hand-off.
- Re-run `cargo run -q -p kidcli -- compare-native-oracle --scenario boot-title-start-level1` and `cargo run -q -p kidcli -- compare-native-oracle --scenario boot-title-start-level1-move-right5s` after the next focused timing hypothesis.
- Do not revisit worker-slot mapping, title-card centring, or the reverted deferred-spawn experiment.

# Verification To Re-run
- cargo test -q -p kid-core boot_title_start_stays_in_mode_8_without_second_start_press -- --nocapture
- cargo test -q -p kid-core level_entry_has_no_damage_grace_window -- --nocapture
- cargo test -q -p kidcli compare_native_states_reports_boot_level_entry_mode_8_release_gate -- --nocapture
- cargo run -q -p kidcli -- compare-native-oracle --scenario boot-title-start-level1
- cargo run -q -p kidcli -- compare-native-oracle --scenario boot-title-start-level1-move-right5s

# Dead Ends / Do Not Retry
- Do not restore the reverse `visible_glyph_count - index - 1` slot mapping; ASM `loc_1AB26` plus `Allocate_GfxObjectSlot` disproved it.
- Do not spend another cycle guessing `loc_1AC70` start branches from native probes; the current oracle pose already points to a timeline or shell-state issue.
- Do not drop back to raw `TitleTextLayout_*` X coordinates; the ASM-backed `d7` centring still matters.
- Do not resurrect the 44-frame pre-roll theory.
- Do not retry the "defer native `loc_1AB26` worker spawn to `StartingLevel`" experiment; it left both frame-520 compares unchanged and broke the `boot_title_start_level_*` worker tests.
