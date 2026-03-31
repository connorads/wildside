---
slug: release-gate-md-runtime-ctrl-pressed-mode8
cycle: 39
updated_at: 2026-03-31T04:40:56Z
---
# What Changed
- Used the single targeted WRAM check the previous handoff allowed, reading `$FFFFFBCE` directly across the early mode-`$0004` window.
- Found that the reload flag is already latched throughout the whole observed early shell lifetime:
  - frames 40, 50, 60, 70, 78, 79, 80: `$FFFFFBCE = $FF`
- This changes the unresolved question:
  - it is no longer "who sets `$FFFFFBCE` before frame 70?"
  - it is now "which early mode-`$0004` path actually jumps into `loc_6E2` while `$FFFFFBCE` is already latched?"

# Current Model
- `$FFFFFBCE` being `FF` is not enough by itself to force an immediate reload; the actual reload only happens when execution jumps to `loc_6E2`.
- The frame-70 empty-list gap is therefore best explained as:
  - `$FFFFFBCE` was already set
  - some mode-`$0004` code path hit `j_loc_6E2`
  - `loc_6E2` wiped object state and reloaded `Load_IntroSequence1`
- Since `sub_1B8F4` and the already-traced title workers do not jump to `loc_6E2` that early, the remaining search target is the early mode-`$0004` control path that can transfer there while the flag is latched.

# Strongest Evidence
- WRAM flag check:
  - `.cache/oracle-probe/oracle/wram_frame_40.bin`
  - `.cache/oracle-probe/oracle/wram_frame_50.bin`
  - `.cache/oracle-probe/oracle/wram_frame_60.bin`
  - `.cache/oracle-probe/oracle/wram_frame_70.bin`
  - `.cache/oracle-probe/oracle/wram_frame_78.bin`
  - `.cache/oracle-probe/oracle/wram_frame_79.bin`
  - `.cache/oracle-probe/oracle/wram_frame_80.bin`
- Supporting ASM:
  - `.cache/upstream/kid-chameleon-disasm/kid.asm:789-850` `loc_65C`, `loc_6E2`, `MainGameLoop`
  - `.cache/upstream/kid-chameleon-disasm/kid.asm:29067-29074` `Load_IntroSequence1`
  - `.cache/upstream/kid-chameleon-disasm/kid.asm:30220-30238` `sub_1B8F4`, `loc_1B920`, `loc_1B924`
- Example flag bytes:
  - frames 40/50/60/70/78/79/80: `0xFBCE = FF`

# Exact Next Moves
- Stay narrow: search the early mode-`$0004` cold-boot control flow for the first path that can jump to `j_loc_6E2` before `sub_1B8F4` reaches its own end-of-sequence transition. Start from call sites and helpers reachable during the `Load_IntroSequence1` lifetime, not from later title-screen logic.
- Only probe again if the ASM leaves two concrete candidate jump paths alive after that read.

# Verification To Re-run
- cargo run -q -p kidcli -- oracle-probe --scenario .cache/oracle-probe/boot_title_start_level1_dense.json --frame-start 330 --frame-end 360 --wram-dump-frames 330,340,350,360
- cargo test -q -p kid-core boot_title_start_mode_8_keeps_advancing_rng_before_gameplay_entry -- --nocapture
- cargo test -q -p kid-core boot_title_start_stays_in_mode_8_without_second_start_press -- --nocapture
- cargo test -q -p kid-core level_entry_has_no_damage_grace_window -- --nocapture
- cargo test -q -p kidcli compare_native_states_reports_boot_level_entry_mode_8_release_gate -- --nocapture
- cargo run -q -p kidcli -- compare-native-oracle --scenario boot-title-start-level1

# Dead Ends / Do Not Retry
- Do not spend another cycle asking who first set `$FFFFFBCE`; WRAM now shows the flag was already latched throughout the observed early window.
- Do not re-check `Load_IntroSequence1` ownership or `loc_6E2` semantics; both are already established.
- Do not widen to later title-screen or mode-`$002C` code yet; the unresolved jump into `loc_6E2` is still inside the early mode-`$0004` lifetime.
