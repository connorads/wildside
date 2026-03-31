---
slug: release-gate-md-runtime-level-entry-slot2-rng-blocker
cycle: 50
updated_at: 2026-03-31T05:36:45Z
---
# What Changed
- Re-read the ROM title-card path around `loc_19C04 -> loc_1AB26 -> loc_1AC70 -> loc_1AD48 -> loc_1AE2E` and searched the native code for every `rom_random` consumer on the mode-$0008` path.
- Confirmed there are no extra `Get_RandomNumber_*` calls in the ROM title-card load/allocation path before `loc_1AC70`, and no extra native mode-$0008` RNG consumers beyond the per-frame `next_long` plus the moved `loc_1AC70` seeding.
- That means the remaining slot-2 miss is no longer “find another RNG consumer”; it is a frame-phase / scheduling ambiguity about how oracle checkpoint timing lines up with `Execute_Objects` and first worker execution.

# Current Model
- The visibility fix stands: frame 500..503 stay hidden, frame 504 is the first visible 13-slot shell, and slot 13 still waits until the next pass.
- The surviving slot-2 miss (`native 0x000E0000,0x00880000` vs oracle `0x007F0000,0x00020000` at `level_entry@520`) cannot be explained by another obvious RNG draw in the title-card code path.
- The live blocker is frame-phase semantics:
  - either oracle state checkpoints sample before/after a different point in the frame than native trace does, or
  - the mode-$0008` object loop / `Execute_Objects` scheduling is still modelled at the wrong point relative to the title-card workers.

# Strongest Evidence
- ASM:
  - `.cache/upstream/kid-chameleon-disasm/kid.asm:1636-1658` `Execute_Objects`
  - `.cache/upstream/kid-chameleon-disasm/kid.asm:28640-28728` `loc_19C04` / `Load_TitleCard`
  - `.cache/upstream/kid-chameleon-disasm/kid.asm:28704-28826` `loc_1AB26`
  - `.cache/upstream/kid-chameleon-disasm/kid.asm:28892-28950` `loc_1AC70`
  - `.cache/upstream/kid-chameleon-disasm/kid.asm:28952-28982` `loc_1AD48`
  - `.cache/upstream/kid-chameleon-disasm/kid.asm:28990-29041` `loc_1AE2E`
- Native search:
  - `rg -n "rom_random\\.next_|next_byte\\(|next_word\\(|next_long\\(" crates/kid-core/src -S`
  - only `crates/kid-core/src/lib.rs` and `crates/kid-core/src/movement.rs` participate on this frontier
- Oracle/native observations:
  - `.cache/oracle-probe/oracle/manifest.json` frame 500..503 hidden, frame 504 first visible shell
  - `.cache/native-oracle-runs/boot_title_start_level1/native/state.json` still first diverges at `level_entry@520`

# Exact Next Moves
- Do one investigation only: add the smallest possible targeted instrumentation to pin the frame phase of mode-$0008` RNG/object execution relative to the oracle checkpoints around frames 500..504.
- Specifically: capture native per-frame RNG offset plus whether `loc_1AC70` seeding happened before or after the trace snapshot for frames 500..505, then compare that against oracle checkpoint timing.
- Do not guess another RNG consumer; the ASM search already ruled that out for this path.

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
- Do not guess another hidden RNG consumer in the title-card path; the ROM/native search already ruled that out.
