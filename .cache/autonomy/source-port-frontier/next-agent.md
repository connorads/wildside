---
slug: release-gate-md-runtime-level-entry-slot2-rng-blocker
cycle: 43
updated_at: 2026-03-31T04:54:27Z
---
# What Changed
- Persisted `oracle-probe` manifests to `.cache/oracle-probe/oracle/manifest.json` so the title-card shell can be inspected without re-plumbing the helper each cycle.
- Re-ran the title-menu / mode-8 probes and compared them against native `probe-native` output for the same boot scenario.
- The new correlation rules out “fix trace fields first” as the next move: oracle’s mode-8 worker shell materialises later than native and with different birth timing.

# Current Model
- The first causal miss is in native’s synthetic `loc_1AB26` / `loc_1AC70` worker setup and glide timing, not in `StartingLevel` trace-field mapping.
- Oracle stays in `Game_Mode=$0008` with no slot-2 worker visible through frames 500..503; slot 2 first appears on frame 504 and the full 14-gfx shell lands on frame 505.
- Native already has the full 14-worker shell active by frame 500, so the bad frame-520 slot-2 pose is downstream of an early / wrong worker birth model.
- `title_screen_ready` is still wrong at frame 520, but that is not the first causal divergence any more.

# Strongest Evidence
- ASM:
  - `.cache/upstream/kid-chameleon-disasm/kid.asm:28750-29064` `loc_1AB26`, `loc_1AC70`, `loc_1AD48`, `loc_1AE2E`
- Oracle probe manifests:
  - `.cache/oracle-probe/oracle/manifest.json`
  - Frame 500: `game_mode=8`, `number_objects=2`, `number_gfx_objects=0`, `slot_2=null`, `rng_offset=65481`
  - Frame 504: `game_mode=8`, `number_objects=15`, `number_gfx_objects=13`, slot 2 at `0x00940000,0xFFF00000`
  - Frame 505: `game_mode=8`, `number_objects=15`, `number_gfx_objects=14`, slot 2 at `0x0092B000,0xFFF22000`
  - Frame 520: slot 2 at `0x007F0000,0x00020000`, `rng_offset=65531`
- Native probe:
  - `cargo run -q -p kidcli -- probe-native --scenario boot-title-start-level1 --frame-start 500 --frame-end 530`
  - Native already shows 14 active workers on frame 500, with the slot-2 trace pose effectively four frames ahead of oracle.
- Release-gate compare:
  - `.cache/native-oracle-runs/boot_title_start_level1/native/state.json`
  - `.cache/native-oracle-runs/boot_title_start_level1/oracle/manifest.json`
  - `compare-native-oracle` still fails at `level_entry@520` with native `0089A000,00262000` vs oracle `007F0000,00020000`

# Exact Next Moves
- Fix one thing only: align native’s early mode-8 worker birth timing with the ROM.
- Read `loc_1ABF4`, `loc_1AC70`, and the surrounding object-allocation path again, then patch the synthetic `starting_level_title_workers` lifecycle so slot-2 visibility starts on oracle frame 504 instead of native frame 500.
- Do not patch `title_screen_ready`, camera, or compare-field masking in the same cycle. Re-run the 500..530 probe window first and make sure worker counts / slot-2 onset match oracle before touching trace fields.

# Verification To Re-run
- cargo run -q -p kidcli -- oracle-probe --scenario .cache/oracle-probe/boot_title_start_level1_dense.json --frame-start 500 --frame-end 530 --wram-dump-frames 500,510,520,530
- cargo run -q -p kidcli -- probe-native --scenario boot-title-start-level1 --frame-start 500 --frame-end 530
- cargo test -q -p kid-core boot_title_start_mode_8_keeps_advancing_rng_before_gameplay_entry -- --nocapture
- cargo test -q -p kid-core boot_title_start_stays_in_mode_8_without_second_start_press -- --nocapture
- cargo test -q -p kid-core level_entry_has_no_damage_grace_window -- --nocapture
- cargo test -q -p kidcli compare_native_states_reports_boot_level_entry_mode_8_release_gate -- --nocapture
- cargo run -q -p kidcli -- compare-native-oracle --scenario boot-title-start-level1

# Dead Ends / Do Not Retry
- Do not go back to the “mode 8 starts late” theory; ASM already killed it.
- Do not spend the next cycle on `StartingLevel` trace-only field tweaks first. Oracle/native worker birth timing now shows the motion model is already wrong before frame 520.
- Do not probe 330..360 again to infer mode-8 behaviour; that window is still plain title shell in oracle.
- Do not patch worker timing and trace masking together. Keep the first gameplay change isolated to the `loc_1AB26` / `loc_1AC70` synthetic worker lifecycle.
