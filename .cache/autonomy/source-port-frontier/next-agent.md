---
slug: release-gate-md-runtime-ctrl-pressed-mode8
cycle: 22
updated_at: 2026-03-31T03:27:30Z
---
# What Changed
- Re-read `Title_InputLoop`, `loc_6E2`, `LoadGameModeData`, `Load_TitleCard loc_19C04`, and `loc_1AE2E` together to trace the title-menu Start hand-off into mode 8.
- Tried deferring native `rebuild_starting_level_title_workers()` until the live `StartingLevel` shell instead of seeding it inside `begin_level_entry`.
- Reverted that experiment after verification: it did not change either frame-520 compare result and it broke the nearby `boot_title_start_level_*` worker tests.

# Current Model
- The remaining frame-520 miss is not fixed by simply delaying the native `loc_1AB26`-equivalent worker spawn to the first `StartingLevel` frame.
- ASM still says the title Start path is `Title_InputLoop -> move.w #8,(Game_Mode) -> st ($FFFFFBCE) -> jmp loc_6E2`, then the mode reload calls `LoadGameModeData`, whose mode-8 loader reaches `Load_TitleCard loc_19C04` and allocates both `loc_1AB26` and `loc_1AE2E`.
- Because the deferred-spawn experiment left the native/oracle pose unchanged, the live miss is more likely in other carried mode-8 shell state than in the raw `loc_1AB26` allocation moment itself.

# Strongest Evidence
- ASM `.cache/upstream/kid-chameleon-disasm/kid.asm:30450-30466` `Title_InputLoop` writes `Game_Mode=$0008`, sets `$FFFFFBCE`, and jumps to `loc_6E2`.
- ASM `.cache/upstream/kid-chameleon-disasm/kid.asm:823-850` `loc_6E2` clears/reloads mode data before entering the main loop; `LoadGameModeData` dispatches the mode-8 loader before any live mode-8 object execution.
- ASM `.cache/upstream/kid-chameleon-disasm/kid.asm:28600-28725` `Load_TitleCard loc_19C04` allocates both `loc_1AB26` and `loc_1AE2E`.
- Verification:
  - `cargo test -q -p kid-core boot_title_start_stays_in_mode_8_without_second_start_press -- --nocapture`
  - `cargo test -q -p kid-core level_entry_has_no_damage_grace_window -- --nocapture`
  - `cargo test -q -p kidcli compare_native_states_reports_boot_level_entry_mode_8_release_gate -- --nocapture`
  - `cargo run -q -p kidcli -- compare-native-oracle --scenario boot-title-start-level1`
  - `cargo run -q -p kidcli -- compare-native-oracle --scenario boot-title-start-level1-move-right5s`
- The compares still fail identically:
  - `boot-title-start-level1`: `state level_entry@520 diverged in kid position (native 008C8000,00268000; oracle 007F0000,00020000)`
  - `boot-title-start-level1-move-right5s`: same frame-520 pose

# Exact Next Moves
- Read the native mode-8/title-shell state against the ASM hand-off and check which title-only fields should be cleared or reset when `Title_InputLoop` jumps through `loc_6E2` into `Load_TitleCard`; focus on fields that still look title-derived at frame 520 (`title_screen_ready`, menu state, carried shell timing), not on `loc_1AB26` spawn timing.
- Re-run `cargo run -q -p kidcli -- compare-native-oracle --scenario boot-title-start-level1` and `cargo run -q -p kidcli -- compare-native-oracle --scenario boot-title-start-level1-move-right5s` after the next focused hypothesis.
- Keep the investigation on the title-menu/mode-8 shell. Do not revisit `loc_1AC70` branch choice or raw title-card centring.

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
- Do not touch RNG order outside `loc_1AC70` until the remaining frame-520 miss is narrowed against the mode-8 shell.
- Do not retry the "defer native `loc_1AB26` worker spawn to `StartingLevel`" experiment; it left both frame-520 compares unchanged and broke the `boot_title_start_level_*` worker tests.
