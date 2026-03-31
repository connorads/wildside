---
slug: release-gate-md-runtime-level-entry-position
cycle: 40
updated_at: 2026-03-31T05:06:00Z
---
# What Changed
- Read the early cold-boot mode-`$0004` ROM path directly instead of probing native first.
- Traced `Load_IntroSequence1 -> sub_1B8F4 -> loc_1B920 -> sub_1C246` and confirmed the first early intro return to `j_loc_6E2` is the shared Start-gated helper, not a separate hidden mode-`$0004` branch.
- Re-ran the handoff verification. The old release-gate note is now stale in one important way: `compare-native-oracle --scenario boot-title-start-level1` no longer fails on `game_mode` at `level_entry@520`; it now fails on Kid position.

# Current Model
- ASM answer:
  - `sub_1B8F4` reaches `loc_1B920`, then hibernates one frame at a time while calling `sub_1C246`.
  - `sub_1C246` does the shared Start return: `move.w #$2C,(Game_Mode).w; st ($FFFFFBCE).w; jmp (j_loc_6E2).w`.
  - So the early cold-boot intro return is not mysterious anymore; it is the same shared Start helper used by later intro sequences.
- Live blocker:
  - The authoritative verifier still reports `state level_entry@520 diverged in kid position (native 0089A000,00262000; oracle 007F0000,00020000)`.
  - That means the old “mode `$0008` vs `$000C`” release-gate note is obsolete as the active frontier. Native now reaches the later shell but lands Kid in the wrong place by level entry.

# Strongest Evidence
- ASM:
  - `.cache/upstream/kid-chameleon-disasm/kid.asm:29067-29074` `Load_IntroSequence1`
  - `.cache/upstream/kid-chameleon-disasm/kid.asm:30221-30240` `sub_1B8F4`, `loc_1B920`, `loc_1B924`
- `.cache/upstream/kid-chameleon-disasm/kid.asm:31146-31167` `sub_1C246`, `loc_1C250`, `loc_1C264`
  - `.cache/upstream/kid-chameleon-disasm/kid.asm:823-850` `loc_6E2`
- Oracle probe:
  - `.cache/oracle-probe/oracle/wram_frame_330.bin`
  - `.cache/oracle-probe/oracle/wram_frame_340.bin`
  - `.cache/oracle-probe/oracle/wram_frame_350.bin`
  - `.cache/oracle-probe/oracle/wram_frame_360.bin`
- Verification:
  - `cargo run -q -p kidcli -- compare-native-oracle --scenario boot-title-start-level1`
  - current failure: `level_entry@520` Kid position drift, artefacts in `.cache/native-oracle-runs/boot_title_start_level1`

# Exact Next Moves
- Stay narrow: inspect the `level_entry@520` artefacts in `.cache/native-oracle-runs/boot_title_start_level1` and identify which shell still owns Kid placement on the oracle side.
- Read the ROM around `loc_1AE2E` and the level-start hand-off it triggers before touching native code. The next question is no longer “who jumps to `loc_6E2`?”, it is “which post-title entry path leaves Kid at `(0x007F0000, 0x00020000)` instead of native’s later in-level pose?”
- Only add probes if the compare artefacts plus ASM leave two concrete placement owners alive.

# Verification To Re-run
- cargo run -q -p kidcli -- oracle-probe --scenario .cache/oracle-probe/boot_title_start_level1_dense.json --frame-start 330 --frame-end 360 --wram-dump-frames 330,340,350,360
- cargo test -q -p kid-core boot_title_start_mode_8_keeps_advancing_rng_before_gameplay_entry -- --nocapture
- cargo test -q -p kid-core boot_title_start_stays_in_mode_8_without_second_start_press -- --nocapture
- cargo test -q -p kid-core level_entry_has_no_damage_grace_window -- --nocapture
- cargo test -q -p kidcli compare_native_states_reports_boot_level_entry_mode_8_release_gate -- --nocapture
- cargo run -q -p kidcli -- compare-native-oracle --scenario boot-title-start-level1

# Dead Ends / Do Not Retry
- Do not spend another cycle searching for a second early mode-`$0004` jump into `loc_6E2`; ASM already shows `sub_1C246` is that path.
- Do not keep treating `level_entry@520` as a `game_mode` split until the verifier says so again; the current authoritative failure is Kid position.
- Do not widen back to later title-screen or timeout-restart frontiers from the stale task package until this local boot-title-start mismatch is understood.
