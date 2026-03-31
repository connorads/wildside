---
slug: release-gate-md-runtime-level-entry-slot2-rng-blocker
cycle: 48
updated_at: 2026-03-31T05:26:48Z
---
# What Changed
- Patched native level-entry trace exposure so the deferred `loc_1AD48` number worker stays out of the first visible shell only; frame 504 now exposes slots `0..12`, and slot 13 joins on frame 505.
- Added a regression proving the frame-504/frame-505 slot transition in `boot_title_start_level_1_keeps_slot_13_hidden_on_frame_504`.
- Re-ran the handoff verification. The frame-504 shell split is green, but `compare-native-oracle --scenario boot-title-start-level1` still first fails at `level_entry@520` on kid/slot-2 position.

# Current Model
- The shell-order question is settled:
  - `loc_1AB26` letters are the only visible workers on oracle frame 504.
  - The deferred `loc_1AD48` number worker is still absent on frame 504 and appears on the following pass.
- After matching that shell split, the remaining miss is now the slot-2 pose/RNG lineage itself, not the presence of slot 13.
- Native still lands slot 2 at `0x0089A000,0x00262000` by `level_entry@520`, while oracle expects `0x007F0000,0x00020000`.

# Strongest Evidence
- ASM:
  - `.cache/upstream/kid-chameleon-disasm/kid.asm:28704-28826` `loc_1AB26`
  - `.cache/upstream/kid-chameleon-disasm/kid.asm:28952-28982` `loc_1AD48`
  - `.cache/upstream/kid-chameleon-disasm/kid.asm:1618-1658` `Execute_Objects`
  - `.cache/upstream/kid-chameleon-disasm/kid.asm:28892-28950` `loc_1AC70`
- Native regression:
  - `crates/kid-core/src/tests.rs:453` `boot_title_start_level_1_keeps_slot_13_hidden_on_frame_504`
- Oracle / comparison:
  - `.cache/oracle-probe/oracle/manifest.json` frame 504 has gfx slots `0..12` only
  - `.cache/native-oracle-runs/boot_title_start_level1/native/state.json` still diverges first at `level_entry@520`

# Exact Next Moves
- Do one investigation only: read `loc_1AC70` again and line up its RNG branch/seed consumption with the surviving slot-2 miss, then patch only the slot-2 start-pose/RNG timing so `level_entry@520` moves towards oracle.
- Preserve the new frame-504 regression and existing baseline tests.
- Re-run `compare-native-oracle --scenario boot-title-start-level1` after the RNG/pose patch; do not widen scope beyond the slot-2 start-pose lineage.

# Verification To Re-run
- cargo run -q -p kidcli -- oracle-probe --scenario .cache/oracle-probe/boot_title_start_level1_dense.json --frame-start 499 --frame-end 504
- cargo run -q -p kidcli -- probe-native --scenario boot-title-start-level1 --frame-start 458 --frame-end 466
- cargo test -q -p kid-core boot_title_start_stays_in_mode_8_without_second_start_press -- --nocapture
- cargo test -q -p kid-core boot_title_start_mode_8_keeps_advancing_rng_before_gameplay_entry -- --nocapture
- cargo test -q -p kid-core boot_title_start_level_1_keeps_slot_13_hidden_on_frame_504 -- --nocapture
- cargo run -q -p kidcli -- compare-native-oracle --scenario boot-title-start-level1

# Dead Ends / Do Not Retry
- Do not retry the pure “delay worker birth to frame 504” patch by itself.
- Do not retry the backed-out “delay worker birth + defer `loc_1AC70` RNG” patch unchanged; it still exposes the wrong frame-504 shell and later diverges at level-entry@520.
- Do not assume the first visible frame includes the level-number worker. Oracle frame 504 proves it does not.
- Do not revisit shell-order fixes now that frame 504 matches; the live frontier is the surviving `loc_1AC70` slot-2 RNG/start-pose miss.
