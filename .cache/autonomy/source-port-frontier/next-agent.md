---
slug: release-gate-md-runtime-ctrl-pressed-mode8
cycle: 8
updated_at: 2026-03-31T02:32:00Z
---
# What Changed
- Read the native mode-8 shell again: `BootState::TitleScreen`, `BootState::IntroSequence6`, `begin_level_entry`, and `BootState::StartingLevel`.
- Added a focused `kidcli` regression that locks the current frontier shape: `level_entry@520` must report the `native 000C` versus `oracle 0008` game-mode split.
- Re-ran the real scenario to confirm the authoritative failure is unchanged after landing only the regression.

# Current Model
- Native still flips from the title shell into `BootState::StartingLevel { remaining_frames: TITLE_TO_LEVEL_FRAMES }` immediately when the title Start edge lands.
- That hard-coded 60-frame native countdown exposes `Game_Mode=$000C` by the `level_entry@520` checkpoint, but the oracle is still in `Game_Mode=$0008` and parked in `WaitForVint` there.
- The remaining fix is not in the comparator any more; it is in the source-port boot shell timing between the title/menu Start edge, `begin_level_entry`, and the eventual in-game hand-off that should happen only after the ROM-backed mode-8 dwell rooted at `loc_1AE2E`.

# Strongest Evidence
- `.cache/upstream/kid-chameleon-disasm/kid.asm:29057-29063` `loc_1AE2E` is still the ROM’s later exit from mode 8 via shared `Ctrl_Pressed`.
- `.cache/upstream/kid-chameleon-disasm/kid.asm:1044-1054` `WaitForVint` matches the oracle PC parked at `0x0000095C` on frames 503..540.
- `crates/kid-core/src/level.rs:166-193` `begin_level_entry` immediately sets `BootState::StartingLevel { remaining_frames: TITLE_TO_LEVEL_FRAMES }`.
- `crates/kid-core/src/boot.rs:1235-1241` `BootState::StartingLevel` decrements the countdown straight to `BootState::InGame`.
- New regression:
  - `cargo test -q -p kidcli compare_native_states_reports_boot_level_entry_mode_8_release_gate -- --nocapture`
  - passes
- Real repro still fails:
  - `cargo run -q -p kidcli -- compare-native-oracle --scenario boot-title-start-level1`
  - `state level_entry@520 diverged in game mode (native 000C, oracle 0008)`

# Exact Next Moves
- Patch the source-port Start-edge/frame-order timing so the post-title mode-8 shell cannot enter `BootState::InGame` before the ROM-backed `loc_1AE2E` dwell would allow it, then add a direct native regression around frame 520 if the fix needs extra protection.
- Re-run `cargo test -q -p kidcli compare_native_states_reports_boot_level_entry_mode_8_release_gate -- --nocapture`, `cargo run -q -p kidcli -- compare-native-oracle --scenario boot-title-start-level1`, then `mise run release-gate`.
- If frame 520 still misses after that patch, capture another oracle CPU window around the new first bad frame before changing any more native timing.

# Verification To Re-run
- cargo test -q -p kidcli compare_native_states_reports_boot_level_entry_mode_8_release_gate -- --nocapture
- cargo test -q -p kidcli capture_uses_current_frame_script_input -- --nocapture
- cargo test -q -p kidcli oracle_limitation -- --nocapture
- cargo test -q -p kid-md frame_render_boundary_is_not_gated_on_vint_enable -- --nocapture
- cargo test -q -p kid-md -- --nocapture
- cargo run -q -p kidcli -- compare-native-oracle --scenario boot-title-start-level1
- mise run release-gate

# Dead Ends / Do Not Retry
- Do not retry the old `title@360` hypothesis.
- Do not assume frame 520 is already executing `loc_1AE2E`; oracle CPU capture disproves that.
- Do not patch title-menu `Ctrl_1_Pressed` handling again; the later gate is shared `Ctrl_Pressed`.
- Do not treat the comparator regression as the fix. It only locks the failure shape.
