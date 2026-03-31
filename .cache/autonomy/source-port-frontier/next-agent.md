---
slug: release-gate-md-runtime-ctrl-pressed-mode8
cycle: 28
updated_at: 2026-03-31T04:03:41Z
---
# What Changed
- Solved the frame-520 oracle pose search for the concrete `o@x=0x40` worker instead of treating it as an open brute-force problem.
- The oracle pose `0x007F0000,0x00020000` is only reachable from `loc_1AC70`'s **top-edge** branch at **elapsed frame 16** for the centred `o` target (`screen x=0xA0`, target y=`0x68`).
- That top-edge launch needs `Get_RandomNumber_word` low byte `0x36..0x39`, which yields `start_x=0x74`.
- Native's current `RomRandomState` stream for the second `o` worker does not provide that lineage:
  - visible glyph order still consumes `selectorByte=0x29` (`top`)
  - then `Get_RandomNumber_word=0x1D30`, so low byte `0x30` and `start_x=0x6C`
- That means the remaining miss is not “pick a different branch for the current worker”. The current RNG order and the oracle pose disagree before slot remapping even matters.

# Current Model
- The concrete oracle slot-2 worker is still the reverse-order third-line `o` whose title-card target `x` is `0x40` before the title-card centring offset.
- Its frame-520 pose implies two facts together:
  - the worker is on `loc_1AC70`'s top-edge path
  - the worker has only advanced 16 glide steps by frame 520
- Native is still modelling frame 520 as a much older worker on the current script-order RNG stream. The stream itself also disagrees: the second `o` gets low byte `0x30`, not the oracle-compatible `0x36..0x39`.
- Therefore the next causal question is timing/creation lineage: when does the ROM actually enter `Game_Mode=$0008` and seed the title workers relative to the frame-460 Start edge, and which worker consumes the `0x36..0x39` low byte on that real lineage?

# Strongest Evidence
- ASM:
  - `.cache/upstream/kid-chameleon-disasm/kid.asm:28695-28872` `loc_1AB26`
  - `.cache/upstream/kid-chameleon-disasm/kid.asm:28874-28960` `loc_1AC70`
  - `.cache/upstream/kid-chameleon-disasm/kid.asm:28962-29016` `loc_1AD48`
- Scenario timing:
  - `crates/kidcli/src/scenarios.rs:704-706` shows the boot-to-level script presses Start on global frame 460.
- Native title-worker rebuild:
  - `crates/kid-core/src/movement.rs:132-230`
  - current stream for visible glyphs is:
    - first `o` (`target_x=0x90`): `selectorByte=0x36`, branch `right`, random Y byte `0x56`
    - second `o` (`target_x=0xA0`): `selectorByte=0x29`, branch `top`, random word `0x1D30`, low byte `0x30`, `start_x=0x6C`
- Oracle pose search:
  - no branch/random combination reaches `0x007F0000,0x00020000` at elapsed `50..65`
  - exact solutions only appear at elapsed `16`, top-edge branch, target `(0xA0,0x68)`, low byte `0x36..0x39`
- Verification still fails unchanged:
  - `cargo run -q -p kidcli -- compare-native-oracle --scenario boot-title-start-level1`
  - `cargo run -q -p kidcli -- compare-native-oracle --scenario boot-title-start-level1-move-right5s`
  - both still report `state level_entry@520 diverged in kid position (native 0089A000,00262000; oracle 007F0000,00020000)`

# Exact Next Moves
- Trace the timing lineage, not the slot mapping: read the ASM around the Start-to-level-entry transition and determine on which global frame the ROM actually runs `loc_1AB26` / `loc_1AC70` for the frame-520 worker. Then compare that timing against native's current `BootState::StartingLevel` entry and elapsed-frame accounting before changing RNG or slot order.

# Verification To Re-run
- cargo test -q -p kid-core boot_title_start_stays_in_mode_8_without_second_start_press -- --nocapture
- cargo test -q -p kid-core level_entry_has_no_damage_grace_window -- --nocapture
- cargo test -q -p kidcli compare_native_states_reports_boot_level_entry_mode_8_release_gate -- --nocapture
- cargo run -q -p kidcli -- compare-native-oracle --scenario boot-title-start-level1
- cargo run -q -p kidcli -- compare-native-oracle --scenario boot-title-start-level1-move-right5s

# Dead Ends / Do Not Retry
- Do not retry the plain reverse-letter slot remap. The oracle-compatible pose now requires an elapsed-16 top-edge launch, so remapping the current native workers cannot solve it.
- Do not brute-force `loc_1AC70` branch/random combinations again without timing context. The search is already solved: only elapsed 16 with top-edge `start_x=0x74` fits the oracle slot-2 pose.
- Do not keep treating frame 520 as a ~60-frame-old worker just because the visible Start press is on frame 460. That assumption is now the leading suspect.
