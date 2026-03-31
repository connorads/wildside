---
slug: release-gate-md-runtime-frame-counter-vint-gated
cycle: 4
updated_at: 2026-03-31T01:41:38Z
---
# What Changed
- Re-read the post-title ROM path beyond the title loop, including `Title_InputLoop` and `loc_1AE2E`, before touching native runtime code.
- Compared the oracle helper’s frame loop against `MdRuntime`’s capture loop instead of re-running title-screen theories.
- Narrowed the likely cause to `MdRuntime` frame counting being gated on VInt enablement during the mode-8 level-start window.

# Current Model
- The frame-360 title checkpoint is no longer the frontier. `probe-native --scenario boot-title-start-level1 --frame-start 350 --frame-end 365` shows `frame 360: input Start mode TitleScreen ... lives/hp 3/2`, and the cached native state now records `title@360 -> game_mode=0x002C`.
- The real release-gate blocker is still the transition after Start: `compare-native-oracle --scenario boot-title-start-level1` fails at `level_entry@520` with `native 0x000C` versus `oracle 0x0008`.
- The ROM path distinguishes two separate Start-driven mode changes:
  - `Title_InputLoop` writes `Game_Mode=$0008` on the title Start edge.
  - `loc_1AE2E` later waits inside the level-entry/title-card loop and writes `Game_Mode=$000C` only after another `Ctrl_Pressed` Start edge.
- The strongest native-side explanation is structural: `MdRuntime::step` only increments `synthetic_vblanks` when `vint_enabled()` is true, and `capture_scenario` uses `run_until_frame_rendered` as its frame clock. The oracle helper advances on every `TickEffect::FrameRendered`, regardless of VInt. If the ROM masks VInt during the mode-8 load, native collapses rendered frames and reaches `0x000C` too early for checkpoint 520.

# Strongest Evidence
- Post-title ROM path:
  - `.cache/upstream/kid-chameleon-disasm/kid.asm:30447` `Title_InputLoop` writes `Game_Mode=$0008` on the title Start edge.
  - `.cache/upstream/kid-chameleon-disasm/kid.asm:29049` `loc_1AE2E` hibernates until `Ctrl_Pressed` Start, then writes `Game_Mode=$000C`.
- Native state cache:
  - `/home/connor/git/kc/.cache/native-oracle-runs/boot_title_start_level1/native/state.json`
  - `title@360`: `game_mode=44`, `number_lives=3`, `camera_y_pos=0`
  - `level_entry@520`: `game_mode=12`, `number_lives=3`, `camera_y_pos=0x00730000`
- Oracle state cache:
  - `/home/connor/git/kc/.cache/native-oracle-runs/boot_title_start_level1/oracle/manifest.json`
  - `title@360`: `game_mode=44`, `number_lives=3`, `camera_y_pos=0`
  - `level_entry@520`: `game_mode=8`, `number_lives=3`, `camera_y_pos=0`
- Repro command:
  - `cargo run -q -p kidcli -- compare-native-oracle --scenario boot-title-start-level1`
- Frame-loop mismatch:
  - `crates/kid-md/src/lib.rs:797-803` increments `synthetic_vblanks` only inside `if self.core.mem.vint_enabled()`.
  - `crates/kid-md/src/lib.rs:819-824` treats `synthetic_vblanks` as the capture-frame boundary.
  - `crates/kidcli/src/oracle_helper_main.rs.txt:528-533` advances the oracle capture on every `TickEffect::FrameRendered`.

# Exact Next Moves
- Add one focused `kid-md` regression that proves `run_until_frame_rendered` can miss rendered frames while VInt is masked, using the boot-title path around `level_entry@520`.
- Fix `MdRuntime` frame counting so capture/verification uses rendered-frame boundaries rather than `vint_enabled()`-gated synthetic interrupts.
- Re-run `compare-native-oracle --scenario boot-title-start-level1`, then `mise run release-gate`.

# Verification To Re-run
- cargo test -q -p kidcli capture_uses_current_frame_script_input -- --nocapture
- cargo test -q -p kidcli oracle_limitation -- --nocapture
- cargo run -q -p kidcli -- compare-native-oracle --scenario boot-title-start-level1
- mise run release-gate

# Dead Ends / Do Not Retry
- Do not retry the old `title@360` hypothesis. The cached native state and `probe-native` both show `MdRuntime` already reaches `Game_Mode=$002C` with 3 lives at that checkpoint.
- Do not patch `Obj_TitleMenu`/`sub_1C246` behaviour again until the later level-entry split is understood; the title loop is not the current blocker.
- Do not rely on `oracle-probe --frame ...` or `inspect-scenario`; those CLI shapes do not exist here.
- Do not assume `run_until_frame_rendered` means “one visible frame” today; in `kid-md` it currently means “next VInt-enabled synthetic VBlank”.
