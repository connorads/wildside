---
slug: release-gate-md-runtime-ctrl-pressed-mode8
cycle: 29
updated_at: 2026-03-31T04:08:24Z
---
# What Changed
- Traced the Start-to-level-entry path far enough to pin the missing timing model in ROM terms.
- Confirmed the title Start edge goes through:
  - `Title_InputLoop` (`kid.asm:30399-30458`) writes `Game_Mode=$0008`
  - `j_loc_6E2` reinitialises object/gfx RAM and calls `LoadGameModeData`
  - `GameLoadArray` maps mode 8 to `Load_TitleCard`
  - `Load_TitleCard` allocates `loc_1AB26` and `loc_1AE2E`
- Confirmed `Execute_Objects` (`kid.asm:1636-1661`) burns `Get_RandomNumber_long` once at the top of **every frame** before it walks objects.
- Confirmed native does not model that global per-frame RNG burn at all. `RomRandomState` is only consumed by local call sites such as `rebuild_starting_level_title_workers`.
- Probed native around frames 460..520:
  - frame 460 already enters mode 8 with all 14 title workers seeded at their start poses
  - frame 461 still shows the untouched start poses
  - frame 462 begins movement
  - frame 520 therefore shows a ~59-step-old slot-2 worker in native
- Boot-global RNG search now has a concrete oracle-compatible lineage:
  - if the reverse-order `o@x=0x40` worker first executes after 114 frame-level `Get_RandomNumber_long` burns, it gets selector byte `0x69` (`top`) and low byte `0x38`, which is inside the required oracle window `0x36..0x39`

# Current Model
- The bug is no longer best described as “wrong branch for the current worker”.
- Native currently collapses three ROM behaviours into one eager setup step:
  - global per-frame `Get_RandomNumber_long` consumption in `Execute_Objects`
  - object-list execution order after `Load_TitleCard`
  - each worker's own first-execution timing at `loc_1AC70`
- Because native seeds every title worker immediately in `begin_level_entry`, slot 2 uses a fresh local RNG stream and becomes ~59 steps old by frame 520.
- Oracle slot 2 instead behaves like a worker whose RNG draw comes from the boot-global stream after many prior frame-level burns; that is why the oracle-compatible `o@x=0x40` solution only appears once the pre-roll count reaches the low-byte `0x38` lineage.

# Strongest Evidence
- ASM:
  - `.cache/upstream/kid-chameleon-disasm/kid.asm:823-920` `j_loc_6E2`, `LoadGameModeData`, `GameLoadArray`
  - `.cache/upstream/kid-chameleon-disasm/kid.asm:1636-1661` `Execute_Objects`
  - `.cache/upstream/kid-chameleon-disasm/kid.asm:30399-30458` `Title_InputLoop`
  - `.cache/upstream/kid-chameleon-disasm/kid.asm:28588-28727` `Load_TitleCard`
  - `.cache/upstream/kid-chameleon-disasm/kid.asm:28750-29016` `loc_1AB26`, `loc_1AC70`, `loc_1AD48`, `loc_1AE2E`
- Native code:
  - `crates/kid-core/src/movement.rs:132-230` eagerly consumes `rom_random` inside `rebuild_starting_level_title_workers`
  - `crates/kid-core/src/types.rs:1384-1416` defines `RomRandomState`, but there is no native equivalent of the frame-level `Get_RandomNumber_long` burn in `Execute_Objects`
- Native probe:
  - `cargo run -q -p kidcli -- probe-native --scenario boot-title-start-level1 --frame-start 450 --frame-end 520`
  - frame 460 already shows 14 title workers at start poses
  - frame 462 is the first moving frame
  - frame 520 still reports native slot 2 at `0089A000,00262000`
- RNG search:
  - reverse-order worker stream plus boot-global pre-roll count `114` yields `o@x=0x40` selector byte `0x69` and low byte `0x38`
  - that matches the oracle-required top-edge / `0x36..0x39` word-low window

# Exact Next Moves
- Implement the ROM's boot-global RNG lineage for title entry instead of rebuilding all workers eagerly: add the missing per-frame `Get_RandomNumber_long` consumption on the title/level-transition object path, then move title-worker RNG seeding to first execution order rather than `begin_level_entry`. Verify whether slot 2 now migrates from the fresh local `0x29/0x30` stream toward the oracle-compatible `0x69/0x38` lineage.

# Verification To Re-run
- cargo test -q -p kid-core boot_title_start_stays_in_mode_8_without_second_start_press -- --nocapture
- cargo test -q -p kid-core level_entry_has_no_damage_grace_window -- --nocapture
- cargo test -q -p kidcli compare_native_states_reports_boot_level_entry_mode_8_release_gate -- --nocapture
- cargo run -q -p kidcli -- probe-native --scenario boot-title-start-level1 --frame-start 450 --frame-end 520
- cargo run -q -p kidcli -- compare-native-oracle --scenario boot-title-start-level1
- cargo run -q -p kidcli -- compare-native-oracle --scenario boot-title-start-level1-move-right5s

# Dead Ends / Do Not Retry
- Do not retry slot remaps or branch brute force without modelling the frame-level RNG burn. The missing `Get_RandomNumber_long` pre-roll invalidates those local-stream searches.
- Do not keep treating `begin_level_entry` as the moment every `loc_1AC70` worker draws its RNG. ROM timing is object-execution-driven, not eager setup.
- Do not assume the frame-460 Start edge implies a fresh RNG seed for level entry. The relevant lineage is boot-global.
