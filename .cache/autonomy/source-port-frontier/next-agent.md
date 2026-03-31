---
slug: release-gate-md-runtime-ctrl-pressed-mode8
cycle: 38
updated_at: 2026-03-31T04:39:14Z
---
# What Changed
- Read the loader/control-flow ASM around `loc_6E2`, `LoadGameModeData`, and `MainGameLoop`.
- Confirmed the exact reload mechanism:
  - when `$FFFFFBCE` is set, `loc_6E2` fades out, clears VRAM, reinitialises object and gfx-object slots, then calls `LoadGameModeData`
  - `LoadGameModeData` reloads the current mode from `GameLoadArray` without changing `Game_Mode`
  - `MainGameLoop` then continues executing the mode handler for that same `Game_Mode`
- Combined with the earlier WRAM dumps, this explains how the frame-70 empty object list can occur while `Game_Mode` stays `$0004`: some earlier path set `$FFFFFBCE`, `loc_6E2` cleared the lists, and `Load_IntroSequence1` was reloaded before `sub_1B8F4` reappeared by frame 78.

# Current Model
- The early cold-boot shell re-entry mechanism is no longer ambiguous:
  - mode `$0004` stays live
  - `$FFFFFBCE` triggers `loc_6E2`
  - `loc_6E2` wipes object state and reloads `Load_IntroSequence1`
- The remaining unknown is now narrower: which early mode-`$0004` path sets `$FFFFFBCE` before frame 70.
- Since the traced loader path itself is RNG-silent, the next useful investigation is to identify the specific object/routine in the early intro-sequence-1 lifetime that raises `$FFFFFBCE`.

# Strongest Evidence
- Supporting ASM:
  - `.cache/upstream/kid-chameleon-disasm/kid.asm:789-850` `loc_65C`, `loc_6E2`, `MainGameLoop`
  - `.cache/upstream/kid-chameleon-disasm/kid.asm:23560-23581` `LoadGameModeData`, `GameLoadArray`
  - `.cache/upstream/kid-chameleon-disasm/kid.asm:29067-29074` `Load_IntroSequence1`
- Previously pinned WRAM facts that this explains:
  - frames 40/70/78/79/99: `Game_Mode = $0004`
  - frame 70: object list empty
  - frame 78: `sub_1B8F4` has been allocated again

# Exact Next Moves
- Stay narrow: identify which early mode-`$0004` object/routine can set `$FFFFFBCE` before frame 70. Start with the live frame-40 `sub_1B8F4` path and any adjacent cold-boot intro objects, and read their ASM for `st ($FFFFFBCE).w` or a jump into `loc_6E2`.
- Use one targeted WRAM probe only if that ASM read still leaves more than one plausible setter.

# Verification To Re-run
- cargo run -q -p kidcli -- oracle-probe --scenario .cache/oracle-probe/boot_title_start_level1_dense.json --frame-start 330 --frame-end 360 --wram-dump-frames 330,340,350,360
- cargo test -q -p kid-core boot_title_start_mode_8_keeps_advancing_rng_before_gameplay_entry -- --nocapture
- cargo test -q -p kid-core boot_title_start_stays_in_mode_8_without_second_start_press -- --nocapture
- cargo test -q -p kid-core level_entry_has_no_damage_grace_window -- --nocapture
- cargo test -q -p kidcli compare_native_states_reports_boot_level_entry_mode_8_release_gate -- --nocapture
- cargo run -q -p kidcli -- compare-native-oracle --scenario boot-title-start-level1

# Dead Ends / Do Not Retry
- Do not keep treating the frame-70 gap as a hidden mode change; `loc_6E2` already explains how reload can happen with `Game_Mode` unchanged.
- Do not keep re-probing `Load_IntroSequence1` ownership; that is already settled.
- Do not widen to later title-screen (`$002C`) control flow yet; the unresolved setter is still in the early mode-`$0004` lifetime.
