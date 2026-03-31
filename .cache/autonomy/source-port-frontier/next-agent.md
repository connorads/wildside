---
slug: release-gate-boot-title-360-md-runtime
cycle: 2
updated_at: 2026-03-31T01:52:00Z
---
# What Changed
- Aligned `crates/kidcli/src/capture.rs` with the oracle helper’s input timing: `capture_scenario` now consumes `script[frame_index]` instead of `script[frame_index - 1]`.
- Added `capture_uses_current_frame_script_input` so the boot-title Start tap on frame 360 stays pinned to the current frame.
- Re-ran `mise run release-gate`; the `title@360` failure remained.

# Current Model
- The remaining blocker is not the closed TAS tail and not the one-frame script offset in `capture_scenario`.
- `boot_title_start_level1` still fails because the `MdRuntime` artefact never reaches title state at the title checkpoint: `/home/connor/git/kc/.cache/oracle-runs/boot_title_start_level1/native/state.json` reports `game_mode=0x000C`, `number_lives=0`, and `title_screen_ready=false` already at frame 360, while the oracle helper reports `game_mode=0x002C`, `number_lives=3`, and `main_menu_option=0`.
- Relevant ROM routines remain `loc_1BAEA`, `loc_1BB02`, and `sub_1C246`, but the present evidence points at the verifier/runtime path (`MdRuntime`) failing to reproduce that boot/title flow at all.

# Strongest Evidence
- Oracle helper timing: `crates/kidcli/src/oracle_helper_main.rs.txt:529` and `:638` already consume `resolved.script.get(frame_index)`, so the old `capture_scenario` indexing was genuinely stale.
- Native capture after the fix still shows the same bad state at frame 360:
  - `/home/connor/git/kc/.cache/oracle-runs/boot_title_start_level1/native/state.json`
  - `title`: `game_mode=12`, `number_lives=0`, `camera_y_pos=0x00730000`
- Oracle capture for the same checkpoint still shows the correct title menu state:
  - `/home/connor/git/kc/.cache/oracle-runs/boot_title_start_level1/oracle/manifest.json`
  - `title`: `game_mode=44`, `main_menu_option=0`, `number_lives=3`
- Fresh failing gate: `/home/connor/git/kc/.cache/release-gate/1p-accurate/1774920889/report.json`

# Exact Next Moves
- Diagnose the `MdRuntime` boot/title divergence directly: compare `crates/kid-md/src/lib.rs` frame stepping and controller application against the oracle helper’s per-frame loop to explain why `MdRuntime` is already in `0x000C` with zero lives by frame 360.
- Add one focused `kid-md` or verifier regression that locks the expected title-state checkpoint before attempting another fix.
- Re-run `mise run release-gate` only after that deeper `MdRuntime` / boot-title path is corrected.

# Verification To Re-run
- cargo test -q -p kidcli capture_uses_current_frame_script_input -- --nocapture
- cargo test -q -p kidcli oracle_limitation -- --nocapture
- mise run release-gate

# Dead Ends / Do Not Retry
- Do not retry the `script[frame_index - 1]` explanation; that mismatch is fixed and the gate still fails.
- Do not patch `kid-core` title-shell logic yet; the failing artefact comes from `MdRuntime` capture, not the native source-port runtime.
