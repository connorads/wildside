---
slug: release-gate-md-runtime-ctrl-pressed-mode8
cycle: 6
updated_at: 2026-03-31T01:46:44Z
---
# What Changed
- Read the ROM’s input path around `ReadJoypad` and `Joypad_ReadFromHardware`, not just the mode handlers.
- Confirmed from ASM that the later mode-8 hand-off consumes the shared `Ctrl_Pressed` edge produced by `ReadJoypad`, not a title-specific latch.
- Tried lightweight `probe-cpu` snapshots, but they are too coarse to isolate the frame-520 mode-8/title-card loop.

# Current Model
- The frame-360 title checkpoint is no longer the frontier. `probe-native --scenario boot-title-start-level1 --frame-start 350 --frame-end 365` shows `frame 360: input Start mode TitleScreen ... lives/hp 3/2`, and the cached native state now records `title@360 -> game_mode=0x002C`.
- The real release-gate blocker is still the transition after Start: `compare-native-oracle --scenario boot-title-start-level1` fails at `level_entry@520` with `native 0x000C` versus `oracle 0x0008`.
- The ROM path distinguishes two separate Start-driven mode changes:
  - `Title_InputLoop` writes `Game_Mode=$0008` on the title Start edge.
  - `loc_1AE2E` later waits inside the level-entry/title-card loop and writes `Game_Mode=$000C` only after another `Ctrl_Pressed` Start edge.
- The VInt-gated frame counter was a real semantic gap in `kid-md`, but it is not sufficient to explain the release-gate failure by itself. After ungating frame counting, `compare-native-oracle --scenario boot-title-start-level1` still fails at `level_entry@520` with the same `native 0x000C` versus `oracle 0x0008`.
- The frontier is now narrower: the remaining mismatch likely sits on how `MdRuntime`/controller emulation generates the shared `Ctrl_Pressed` Start edge inside the mode-8 title-card loop rooted at ASM `loc_1AE2E`, or on another post-title timing edge after `ReadJoypad` updates `Ctrl_Pressed`.

# Strongest Evidence
- Post-title ROM path:
  - `.cache/upstream/kid-chameleon-disasm/kid.asm:30447` `Title_InputLoop` writes `Game_Mode=$0008` on the title Start edge.
  - `.cache/upstream/kid-chameleon-disasm/kid.asm:29049` `loc_1AE2E` hibernates until `Ctrl_Pressed` Start, then writes `Game_Mode=$000C`.
- Input-edge ROM path:
  - `.cache/upstream/kid-chameleon-disasm/kid.asm:1404-1428` `ReadJoypad loc_C4E -> loc_C9C` derives newly pressed keys and copies them into shared `Ctrl_Pressed`.
  - `.cache/upstream/kid-chameleon-disasm/kid.asm:1436-1451` `Joypad_ReadFromHardware` performs the TH-low/TH-high reads from `$A10003`.
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
  - `crates/kid-md/src/lib.rs:800-806` now counts every synthetic display frame while still gating only the IRQ request on `vint_enabled()`.
  - `crates/kid-md/src/lib.rs:2761-2772` adds `frame_render_boundary_is_not_gated_on_vint_enable`.
  - `crates/kidcli/src/oracle_helper_main.rs.txt:528-533` advances the oracle capture on every `TickEffect::FrameRendered`.
- Verification after the change:
  - `cargo test -q -p kid-md frame_render_boundary_is_not_gated_on_vint_enable -- --nocapture` passes.
  - `cargo test -q -p kid-md -- --nocapture` passes.
  - `cargo run -q -p kidcli -- compare-native-oracle --scenario boot-title-start-level1` still fails at `level_entry@520`.
- Lightweight CPU probes:
  - `cargo run -q -p kidcli -- probe-cpu --steps 250000 ... --summary-only` only reaches `game_mode=0x0000`.
  - `cargo run -q -p kidcli -- probe-cpu --steps 400000 ... --summary-only` reaches only `game_mode=0x0004`.
  - Those summaries are too coarse to diagnose the frame-520 mode-8 loop directly.

# Exact Next Moves
- Capture oracle CPU state at or around `level_entry@520` for `boot_title_start_level1`, so the exact ROM PC/routine during the mode-8 window is known before patching `kid-md`.
- Add one focused `kid-md` regression around the shared `Ctrl_Pressed`/Start-edge behaviour once that oracle frame confirms where the ROM is sitting.
- Re-run `cargo run -q -p kidcli -- compare-native-oracle --scenario boot-title-start-level1`, then `mise run release-gate` once the mode-8 hand-off is corrected.

# Verification To Re-run
- cargo test -q -p kidcli capture_uses_current_frame_script_input -- --nocapture
- cargo test -q -p kidcli oracle_limitation -- --nocapture
- cargo test -q -p kid-md frame_render_boundary_is_not_gated_on_vint_enable -- --nocapture
- cargo test -q -p kid-md -- --nocapture
- cargo run -q -p kidcli -- compare-native-oracle --scenario boot-title-start-level1
- mise run release-gate

# Dead Ends / Do Not Retry
- Do not retry the old `title@360` hypothesis. The cached native state and `probe-native` both show `MdRuntime` already reaches `Game_Mode=$002C` with 3 lives at that checkpoint.
- Do not patch `Obj_TitleMenu`/`sub_1C246` behaviour again until the later level-entry split is understood; the title loop is not the current blocker.
- Do not rely on `oracle-probe --frame ...` or `inspect-scenario`; those CLI shapes do not exist here.
- Do not treat VInt-gated frame counting as the primary frontier any more; the local semantic fix landed, but the authoritative boot-title compare did not move.
- Do not treat `Ctrl_1_Pressed` as the only relevant Start latch. ASM `loc_1AE2E` consumes shared `Ctrl_Pressed`, which is rebuilt by `ReadJoypad loc_C4E -> loc_C9C`.
