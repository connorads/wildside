---
slug: release-gate-md-runtime-ctrl-pressed-mode8
cycle: 9
updated_at: 2026-03-31T03:35:00Z
---
# What Changed
- Patched the source-port boot shell so `StartingLevel` now models ASM `Title_InputLoop -> Game_Mode=$0008` followed by `loc_1AE2E`'s later shared-Start gate, instead of jumping straight into gameplay on the title-menu Start press.
- Updated the gameplay/trace path so `StartingLevel` reports `Game_Mode=$0008` and no longer runs gameplay immediately.
- Added a direct `kid-core` regression proving the boot script stays in mode 8 through frames 520 and 719 unless a second Start edge arrives.

# Current Model
- The mode bug is fixed. Native no longer reaches `Game_Mode=$000C` at `level_entry@520` from the frame-460 Start press alone.
- The new first authoritative miss is `kid position` at `level_entry@520`: native now exposes an empty mode-8 shell (`kid 0,0`), while oracle exposes a fully populated title-card/title-shell object roster with moving sprite positions.
- This is the real blocker behind the old mode mismatch: the native source-port does not implement the ROM’s mode-8 title-card presentation objects at all. Timing alone cannot clear the compare any further.

# Strongest Evidence
- `.cache/upstream/kid-chameleon-disasm/kid.asm:30450-30465` `Title_InputLoop` writes `Game_Mode=$0008` on the title-menu Start edge.
- `.cache/upstream/kid-chameleon-disasm/kid.asm:29057-29063` `loc_1AE2E` hibernates and waits for a later shared `Ctrl_Pressed` Start before writing `Game_Mode=$000C`.
- New native regression:
  - `cargo test -q -p kid-core boot_title_start_stays_in_mode_8_without_second_start_press -- --nocapture`
  - passes
- Existing gameplay helper still works with an explicit second Start:
  - `cargo test -q -p kid-core level_entry_has_no_damage_grace_window -- --nocapture`
  - passes
- Oracle probe dead-end:
  - `cargo run -q -p kidcli -- oracle-probe --scenario .cache/oracle-probe/boot_title_start_level1_confirm_start.json --frame-start 628 --frame-end 705 --watches 0xFC40:u16,0xF81C:u32,0xF820:u32,0x24FA:u32,0x24FE:u32`
  - keeps `number_hitpoints=2`, `camera_x_pos=0`, `camera_y_pos=0` through frame 705, so a frame-462 confirm-Start is not the missing oracle transition into gameplay
- Authoritative compare moved forward:
  - before: `state level_entry@520 diverged in game mode (native 000C, oracle 0008)`
  - now: `state level_entry@520 diverged in kid position (native 00000000,00000000; oracle 007F0000,00020000)`
- Oracle mode-8 shell data from `.cache/native-oracle-runs/boot_title_start_level1/oracle/manifest.json`:
  - `level_entry@520`: `game_mode=8`, `number_objects=15`, `number_gfx_objects=14`, `kid_x_pos=0x007F0000`, `kid_y_pos=0x00020000`
  - `level_settle@719`: `game_mode=8`, `number_objects=15`, `number_gfx_objects=14`, `kid_x_pos=0x00400000`, `kid_y_pos=0x00680000`
  - `active_enemies` in this window are actually the unmodelled title-card/title-shell sprites (`object_type=0`, 14 entries), not gameplay enemies.

# Exact Next Moves
- Read the ROM/title-card worker that owns the mode-8 shell sprite positions feeding the `level_entry@520` / `level_settle@719` oracle snapshots, then model that shell in native trace/runtime instead of exposing an empty `StartingLevel`.
- Once the owner routine is traced, patch `StartingLevel` trace/object output to emit the correct title-card roster, kid position, camera, and object counts for mode 8.
- Re-run `cargo test -q -p kid-core boot_title_start_stays_in_mode_8_without_second_start_press -- --nocapture`, `cargo run -q -p kidcli -- compare-native-oracle --scenario boot-title-start-level1`, then `cargo run -q -p kidcli -- compare-native-oracle --scenario boot-title-start-level1-move-right5s`.

# Verification To Re-run
- cargo test -q -p kid-core boot_title_start_stays_in_mode_8_without_second_start_press -- --nocapture
- cargo test -q -p kid-core level_entry_has_no_damage_grace_window -- --nocapture
- cargo test -q -p kidcli compare_native_states_reports_boot_level_entry_mode_8_release_gate -- --nocapture
- cargo run -q -p kidcli -- compare-native-oracle --scenario boot-title-start-level1
- cargo run -q -p kidcli -- compare-native-oracle --scenario boot-title-start-level1-move-right5s

# Dead Ends / Do Not Retry
- Do not revert to the old `StartingLevel -> InGame` countdown. The ROM requires a later shared Start edge after `Game_Mode=$0008`.
- Do not keep chasing frame-count constants. The next miss is missing mode-8 shell content, not another dwell off-by-one.
- Do not assume a frame-462 confirm-Start reproduces the oracle opening path. It only helps native gameplay harness tests reach `InGame`; oracle still stays in the mode-8 shell on that script.
- Do not assume the mode-8 oracle snapshots are gameplay objects. The `object_type=0` roster at frames 520 and 719 is the unmodelled title-card/title-shell presentation.
