---
slug: release-gate-md-runtime-ctrl-pressed-mode8
cycle: 7
updated_at: 2026-03-31T02:25:00Z
---
# What Changed
- Read the ROM again around `ReadJoypad`, `Joypad_ReadFromHardware`, `loc_1AE2E`, and `Title_InputLoop` before touching native code.
- Captured oracle CPU registers for `boot_title_start_level1` on frames 500..540 with `oracle-probe --capture-cpu-state`.
- Re-ran `compare-native-oracle --scenario boot-title-start-level1` to confirm the release gate is still `level_entry@520` (`native 000C`, `oracle 0008`).

# Current Model
- The frame-520 oracle checkpoint is not sitting in `loc_1AE2E` yet. By frames 503..540 the ROM PC is parked at `0x0000095C`, inside the `WaitForVint` loop immediately after `sub_8E0` (`WaitForVint` in `kid.asm`).
- That means the oracle is still in the mode-8/title-card presentation shell at `level_entry@520`; the native side is advancing to `Game_Mode=$000C` too early, before the ROM leaves the frame-boundary wait loop.
- ASM still says the later exit out of mode 8 must come from `loc_1AE2E` consuming shared `Ctrl_Pressed` (`bclr #Button_Start,(Ctrl_Pressed).w`), not directly from `Ctrl_1_Pressed`.
- The next gap is therefore native timing/order: either `MdRuntime` presents or consumes the frame-460 Start edge too early relative to `WaitForVint`/`ReadJoypad`, or mode-8 native code bypasses the ROM’s extra frame-boundary dwell before `loc_1AE2E` sees the shared edge.

# Strongest Evidence
- `.cache/upstream/kid-chameleon-disasm/kid.asm:1332-1430` `ReadJoypad` rebuilds `Ctrl_1_Pressed`, `Ctrl_2_Pressed`, then copies the active player's edge into shared `Ctrl_Pressed`.
- `.cache/upstream/kid-chameleon-disasm/kid.asm:1436-1451` `Joypad_ReadFromHardware` performs the TH-low/TH-high hardware reads from `$A10003`/`$A10005`.
- `.cache/upstream/kid-chameleon-disasm/kid.asm:29057-29063` `loc_1AE2E` hibernates one frame at a time, then consumes shared `Ctrl_Pressed` Start before writing `Game_Mode=$000C`.
- `.cache/upstream/kid-chameleon-disasm/kid.asm:1044-1054` `WaitForVint` spins on `V_Int_Done`; oracle PC `0x0000095C` falls inside this loop.
- Oracle CPU capture (`cargo run -q -p kidcli -- oracle-probe --scenario .cache/oracle-probe/boot_title_start_level1_dense.json --frame-start 500 --frame-end 540 --capture-cpu-state`):
  - frame 500: `PC=0x000138F2`
  - frame 501: `PC=0x00014316`
  - frame 502: `PC=0x00000DC4`
  - frames 503..540: `PC=0x0000095C`, `SR=0x2500`
- Authoritative compare still fails:
  - `cargo run -q -p kidcli -- compare-native-oracle --scenario boot-title-start-level1`
  - `state level_entry@520 diverged in game mode (native 000C, oracle 0008)`

# Exact Next Moves
- Read the native mode-8/frame-boundary path around `MdRuntime` and the source-port boot/title shell, then add one focused regression proving native must still be in mode 8 at frame 520 while the oracle is waiting in `WaitForVint`.
- Once that regression is red, patch the native Start-edge/frame-order logic so the post-title mode-8 shell cannot advance to `Game_Mode=$000C` before the ROM-backed `loc_1AE2E` timing would allow it.
- Re-run `cargo run -q -p kidcli -- compare-native-oracle --scenario boot-title-start-level1`, then `mise run release-gate`.

# Verification To Re-run
- cargo test -q -p kidcli capture_uses_current_frame_script_input -- --nocapture
- cargo test -q -p kidcli oracle_limitation -- --nocapture
- cargo test -q -p kid-md frame_render_boundary_is_not_gated_on_vint_enable -- --nocapture
- cargo test -q -p kid-md -- --nocapture
- cargo run -q -p kidcli -- compare-native-oracle --scenario boot-title-start-level1
- mise run release-gate

# Dead Ends / Do Not Retry
- Do not retry the old `title@360` hypothesis. Native already reaches `Game_Mode=$002C` there.
- Do not assume frame 520 is already inside `loc_1AE2E`; oracle CPU capture disproves that.
- Do not patch title-menu `Ctrl_1_Pressed` handling again. ASM says the later release gate uses shared `Ctrl_Pressed`.
- Do not treat the earlier VInt frame-counter fix as sufficient. It landed, and the authoritative compare did not move.
