---
slug: release-gate-md-runtime-ctrl-pressed-mode8
cycle: 10
updated_at: 2026-03-31T03:48:00Z
---
# What Changed
- Traced the ROM worker that owns the mode-8 title-card shell roster instead of the earlier title-menu gate guess.
- Confirmed the shell objects at `level_entry@520` / `level_settle@719` come from the level-title builder `loc_1AB26`, which walks the level-name string and spawns title-card letters directly as `loc_1AC70` gfx objects.
- Confirmed any deferred multi-letter block is handed to `loc_1ABF4`, which allocates a `loc_1AD48` spawner that fans out extra `loc_1AC70` gfx objects from a coordinate table; the separate two-player gate object is `loc_1AD80 -> loc_1AE2E` and is not the 14-object shell seen in oracle.

# Current Model
- The mode bug is fixed. Native no longer reaches `Game_Mode=$000C` at `level_entry@520` from the frame-460 Start press alone.
- The new first authoritative miss is still `kid position` at `level_entry@520`: native exposes an empty mode-8 shell (`kid 0,0`), while oracle exposes the title-card letter shell driven by `loc_1AB26` / `loc_1ABF4` / `loc_1AD48` / `loc_1AC70`.
- The native source-port is missing the whole level-title presentation path: it does not decode the level-name script, does not emit the direct `loc_1AC70` letter objects, and does not model the deferred fanout blocks that `loc_1AD48` launches toward their target coordinates.

# Strongest Evidence
- `.cache/upstream/kid-chameleon-disasm/kid.asm:28746-28838` `loc_1AB26` loads title-card letter art/palette, walks `AddrTbl_LevelNames`, and spawns direct `loc_1AC70` gfx objects from the level-name string plus the `(x,y)` coordinate table in `a3`.
- `.cache/upstream/kid-chameleon-disasm/kid.asm:28839-28852` `loc_1ABF4` allocates `loc_1AD48` with `d0` and `a3` when the level-name script hits `$FF`, then optionally allocates the separate two-player gate object `loc_1AD80`.
- `.cache/upstream/kid-chameleon-disasm/kid.asm:28989-29009` `loc_1AD48` reads the target coordinate list and allocates repeated `loc_1AC70` workers.
- `.cache/upstream/kid-chameleon-disasm/kid.asm:28896-28988` `loc_1AC70` is the actual title-card letter worker: it picks a random off-screen start, then moves one gfx object toward the supplied destination over `#$3F` frames.
- `.cache/upstream/kid-chameleon-disasm/kid.asm:29010-29063` `loc_1AD80 -> loc_1AE2E` only animates the separate shell/gate pair and waits for Start; it does not explain oracle’s 14-object roster.
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
- `cargo run -q -p kidcli -- compare-native-oracle --scenario boot-title-start-level1-move-right5s`
  - fails on the same `level_entry@520` `kid position` miss, confirming the frontier is the shared mode-8 shell path rather than later gameplay input handling.

# Exact Next Moves
- Model `loc_1AB26`’s level-title presentation in native `StartingLevel`: decode the test/native level-name script, emit the direct `loc_1AC70` letter objects, and reproduce the deferred `loc_1AD48` fanout path instead of exposing an empty shell.
- Start with trace parity, not full rendering: emit the correct mode-8 kid position, object counts, and `active_enemies`/gfx-object roster for frames 520 and 719 so compare stops failing at the first shell checkpoint.
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
- Do not overfit the shell to `loc_1AD80 -> loc_1AE2E` alone. That worker only covers the extra two-player gate sprites; the main letter roster comes from `loc_1AB26` and deferred `loc_1AD48` fanout.
