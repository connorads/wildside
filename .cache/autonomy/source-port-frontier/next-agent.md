---
slug: release-gate-md-runtime-mode8-start-edge
cycle: 5
updated_at: 2026-03-31T01:44:40Z
---
# What Changed
- Added a `kid-md` regression for `run_until_frame_rendered` while VInt is masked, then updated `MdRuntime::step` so synthetic frame counting is no longer gated on interrupt delivery.
- Re-ran the targeted `kid-md` test suite and the authoritative boot-title compare after that change.
- Confirmed the new frame-counter coverage is valid, but the boot-title frontier did not move.

# Current Model
- The frame-360 title checkpoint is no longer the frontier. `probe-native --scenario boot-title-start-level1 --frame-start 350 --frame-end 365` shows `frame 360: input Start mode TitleScreen ... lives/hp 3/2`, and the cached native state now records `title@360 -> game_mode=0x002C`.
- The real release-gate blocker is still the transition after Start: `compare-native-oracle --scenario boot-title-start-level1` fails at `level_entry@520` with `native 0x000C` versus `oracle 0x0008`.
- The ROM path distinguishes two separate Start-driven mode changes:
  - `Title_InputLoop` writes `Game_Mode=$0008` on the title Start edge.
  - `loc_1AE2E` later waits inside the level-entry/title-card loop and writes `Game_Mode=$000C` only after another `Ctrl_Pressed` Start edge.
- The VInt-gated frame counter was a real semantic gap in `kid-md`, but it is not sufficient to explain the release-gate failure by itself. After ungating frame counting, `compare-native-oracle --scenario boot-title-start-level1` still fails at `level_entry@520` with the same `native 0x000C` versus `oracle 0x0008`.
- The frontier is now narrower: the remaining mismatch likely sits on how `MdRuntime`/controller emulation generates the later `Ctrl_Pressed` Start edge inside the mode-8 title-card loop rooted at ASM `loc_1AE2E`, or on another post-title timing edge after the frame clock.

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
  - `crates/kid-md/src/lib.rs:800-806` now counts every synthetic display frame while still gating only the IRQ request on `vint_enabled()`.
  - `crates/kid-md/src/lib.rs:2761-2772` adds `frame_render_boundary_is_not_gated_on_vint_enable`.
  - `crates/kidcli/src/oracle_helper_main.rs.txt:528-533` advances the oracle capture on every `TickEffect::FrameRendered`.
- Verification after the change:
  - `cargo test -q -p kid-md frame_render_boundary_is_not_gated_on_vint_enable -- --nocapture` passes.
  - `cargo test -q -p kid-md -- --nocapture` passes.
  - `cargo run -q -p kidcli -- compare-native-oracle --scenario boot-title-start-level1` still fails at `level_entry@520`.

# Exact Next Moves
- Inspect the `kid-md` controller edge path during the mode-8 level-start/title-card window and compare it with the ROM’s `Ctrl_Pressed`-driven `loc_1AE2E` hand-off.
- Add one focused regression around that second Start-edge behaviour on the boot-title path, then fix it in `kid-md` if confirmed.
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
