---
slug: release-gate-md-runtime-level-entry-slot2-rng-blocker
cycle: 49
updated_at: 2026-03-31T05:34:59Z
---
# What Changed
- Moved `loc_1AC70` seeding out of level-entry shell construction and into first trace-visible execution, so the letter workers stay hidden through frame 503 and first appear on frame 504.
- Added a regression proving slot 2 is still hidden on frame 503 and only appears on frame 504, while keeping the earlier frame-504 slot-13 regression green.
- Re-ran `compare-native-oracle --scenario boot-title-start-level1`; the first miss is still `level_entry@520`, but the native slot-2 pose changed from the earlier long-glide shell to `0x000E0000,0x00880000`, confirming the old pre-504 visibility bug was real and now removed.

# Current Model
- Two separate issues were conflated:
  - visibility timing: fixed enough that frame 500..503 now have no visible title shell and frame 504 is the first visible shell;
  - RNG lineage before `loc_1AC70`: still wrong, so slot 2 seeds from the wrong random branch/bytes when frame 504 arrives.
- Native now matches the oracle’s “hidden until frame 504” structure, but not the oracle’s frame-504 slot-2 start pose (`0x00940000,0xFFE00000`).
- The remaining task is to identify which mode-$0008` path consumes RNG before the frame-504 worker execution and mirror that, without undoing the new visibility timing.

# Strongest Evidence
- ASM:
  - `.cache/upstream/kid-chameleon-disasm/kid.asm:28892-28950` `loc_1AC70`
  - `.cache/upstream/kid-chameleon-disasm/kid.asm:28952-28982` `loc_1AD48`
  - `.cache/upstream/kid-chameleon-disasm/kid.asm:28990-29041` `loc_1AE2E`
- Native regressions:
  - `crates/kid-core/src/tests.rs:453` `boot_title_start_level_1_keeps_slot_13_hidden_on_frame_504`
  - `crates/kid-core/src/tests.rs:502` `boot_title_start_level_1_keeps_slot_2_hidden_until_frame_504`
- Oracle / native comparison:
  - `.cache/oracle-probe/oracle/manifest.json` frame 500..503 has no visible shell; frame 504 is the first 13-slot shell
  - `.cache/native-oracle-runs/boot_title_start_level1/native/state.json` still first fails at `level_entry@520`, native kid/slot-2 `0x000E0000,0x00880000` vs oracle `0x007F0000,0x00020000`

# Exact Next Moves
- Do one investigation only: identify the remaining pre-frame-504 RNG consumers on the mode-$0008` path before `loc_1AC70` executes, then patch only that RNG lineage so slot 2 seeds from the oracle-backed branch/bytes.
- Preserve the new hidden-until-frame-504 regressions.
- Re-run `compare-native-oracle --scenario boot-title-start-level1` after the RNG-lineage patch; do not widen scope beyond the pre-`loc_1AC70` RNG path.

# Verification To Re-run
- cargo run -q -p kidcli -- oracle-probe --scenario .cache/oracle-probe/boot_title_start_level1_dense.json --frame-start 500 --frame-end 504
- cargo run -q -p kidcli -- probe-native --scenario boot-title-start-level1 --frame-start 500 --frame-end 504
- cargo test -q -p kid-core boot_title_start_stays_in_mode_8_without_second_start_press -- --nocapture
- cargo test -q -p kid-core boot_title_start_mode_8_keeps_advancing_rng_before_gameplay_entry -- --nocapture
- cargo test -q -p kid-core boot_title_start_level_1_keeps_slot_13_hidden_on_frame_504 -- --nocapture
- cargo test -q -p kid-core boot_title_start_level_1_keeps_slot_2_hidden_until_frame_504 -- --nocapture
- cargo run -q -p kidcli -- compare-native-oracle --scenario boot-title-start-level1

# Dead Ends / Do Not Retry
- Do not retry the pure “delay worker birth to frame 504” patch by itself.
- Do not retry the backed-out “delay worker birth + defer `loc_1AC70` RNG” patch unchanged; it still exposes the wrong frame-504 shell and later diverges at level-entry@520.
- Do not remove the mode-$0008` RNG burn entirely; that freezes the baseline RNG through gameplay entry and fails `boot_title_start_mode_8_keeps_advancing_rng_before_gameplay_entry`.
- Do not revisit shell-order fixes now that frame 500..503 are hidden and frame 504 shows only slots `0..12`; the live frontier is the remaining pre-`loc_1AC70` RNG lineage.
