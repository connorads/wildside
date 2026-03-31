---
slug: release-gate-boot-title-level-entry-md-runtime
cycle: 3
updated_at: 2026-03-31T01:39:11Z
---
# What Changed
- Re-read the title-menu ASM before touching native logic: `Obj_TitleMenu loc_1BAEA`, `loc_1BB02`, and `sub_1C246` in `.cache/upstream/kid-chameleon-disasm/kid.asm`.
- Re-ran native/oracle diagnostics with the supported commands instead of relying on the stale handoff model.
- Confirmed `MdRuntime` is already correct at `title@360`; the first blocking mismatch is later at `level_entry@520`.

# Current Model
- The frame-360 title checkpoint is no longer the frontier. `probe-native --scenario boot-title-start-level1 --frame-start 350 --frame-end 365` shows `frame 360: input Start mode TitleScreen ... lives/hp 3/2`, and the cached native state now records `title@360 -> game_mode=0x002C`.
- The real release-gate blocker is the transition after Start: `compare-native-oracle --scenario boot-title-start-level1` fails at `level_entry@520` with `native 0x000C` versus `oracle 0x0008`.
- ASM `sub_1C246` only handles the Start-edge return to `Game_Mode=$2C`; it does not explain the later `0x0008`/`0x000C` split. The next investigation has to move past the title loop and into the boot-to-level hand-off inside `MdRuntime`.

# Strongest Evidence
- Title loop ASM:
  - `.cache/upstream/kid-chameleon-disasm/kid.asm:30389` `loc_1BAEA` clears `Ctrl_Pressed`, hibernates one frame, and branches on the `Ctrl_1_Pressed` edge bit.
  - `.cache/upstream/kid-chameleon-disasm/kid.asm:30399` `loc_1BB02` sets `Game_Mode=$0010`.
  - `.cache/upstream/kid-chameleon-disasm/kid.asm:31119` `sub_1C246` sets `Game_Mode=$002C` on Start-edge paths.
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

# Exact Next Moves
- Read the ROM routines that own the post-title level-load hand-off for this path, then compare them with the `MdRuntime` frame stepping around the first bad checkpoint at `level_entry@520`.
- Add one focused verifier or `kid-md` regression that asserts `boot_title_start_level1` stays in the oracle-backed load state at `level_entry@520` before gameplay begins.
- Re-run `mise run release-gate` only after the `0x0008` versus `0x000C` hand-off is explained and fixed.

# Verification To Re-run
- cargo test -q -p kidcli capture_uses_current_frame_script_input -- --nocapture
- cargo test -q -p kidcli oracle_limitation -- --nocapture
- cargo run -q -p kidcli -- probe-native --scenario boot-title-start-level1 --frame-start 350 --frame-end 365
- cargo run -q -p kidcli -- compare-native-oracle --scenario boot-title-start-level1
- mise run release-gate

# Dead Ends / Do Not Retry
- Do not retry the old `title@360` hypothesis. The cached native state and `probe-native` both show `MdRuntime` already reaches `Game_Mode=$002C` with 3 lives at that checkpoint.
- Do not patch `Obj_TitleMenu`/`sub_1C246` behaviour again until the later level-entry split is understood; the title loop is not the current blocker.
- Do not rely on `oracle-probe --frame ...` or `inspect-scenario`; those CLI shapes do not exist here.
