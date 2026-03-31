---
slug: release-gate-md-runtime-ctrl-pressed-mode8
cycle: 25
updated_at: 2026-03-31T03:46:53Z
---
# What Changed
- Re-read `Allocate_GfxObjectSlot` / `Load_GfxObjectSlot` (`sub_F6E`) in addition to the earlier object-list routines.
- Verified a tighter ASM model for the second live mode-8 pass:
  - equal-priority object slots are inserted before existing peers by `Allocate_ObjectSlot`
  - equal-priority gfx slots are appended after existing peers by `Allocate_GfxObjectSlot`
  - therefore the first `loc_1AC70` execution order would expose the title shell in reverse allocation order, with the deferred number worker ahead of the letters
- Re-ran the baseline verification and confirmed the frontier still sits at the same frame-520 pose:
  - `cargo test -q -p kid-core boot_title_start_stays_in_mode_8_without_second_start_press -- --nocapture`
  - `cargo test -q -p kid-core level_entry_has_no_damage_grace_window -- --nocapture`
  - `cargo test -q -p kidcli compare_native_states_reports_boot_level_entry_mode_8_release_gate -- --nocapture`
  - `cargo run -q -p kidcli -- compare-native-oracle --scenario boot-title-start-level1`
  - `cargo run -q -p kidcli -- compare-native-oracle --scenario boot-title-start-level1-move-right5s`
- Rejected a second local simplification: even the reverse-execution model with the deferred number worker occupying the earliest gfx slots still did not reproduce oracle slot 2 at frame 520.

# Current Model
- The remaining `level_entry@520` miss is not explained by either of the two obvious shell models:
  - plain creation-order workers
  - reverse execution order with the deferred number worker taking the earliest gfx slot(s)
- ASM now says the problem is narrower still: native is probably not tracing the same identity that the oracle surfaces as frame-520 `kid_gfx` slot 2.
- Concretely:
  - `loc_1AB26` allocates the workers in script order
  - `Allocate_ObjectSlot` reverses equal-priority execution order
  - `Allocate_GfxObjectSlot` preserves first-executed-first-visible order for equal-priority gfx objects
  - yet neither resulting shell identity model matches the oracle slot-2 pose
- That points to one of two remaining possibilities:
  - the oracle `gfx_slot_index == 2` at frame 520 is not the third visible title glyph in either simple order because another shell object consumes the early gfx slots on this path
  - or native should not derive the traced kid glyph purely from shell order at all, and needs a closer mapping to the oracle's frame-520 `parent_object_ptr` / gfx-slot lineage

# Strongest Evidence
- ASM:
  - `.cache/upstream/kid-chameleon-disasm/kid.asm:1559-1616` `Allocate_ObjectSlot`: equal-priority objects insert ahead of existing peers.
  - `.cache/upstream/kid-chameleon-disasm/kid.asm:1718-1775` `Delete_CurrentObject`: deleting `loc_1AB26` resumes iteration from the predecessor link, so the newly spawned workers wait for the next outer pass.
  - `.cache/upstream/kid-chameleon-disasm/kid.asm:1824-1911` `Load_GfxObjectSlot` / `Allocate_GfxObjectSlot`: equal-priority gfx objects append behind earlier executed peers.
  - `.cache/upstream/kid-chameleon-disasm/kid.asm:28695-29064` `loc_1AB26`, `loc_1AC70`, `loc_1AD48`, `loc_1AE2E`.
- Oracle frame-520 metadata still says slot 2 is the traced kid glyph:
  - `.cache/native-oracle-runs/boot_title_start_level1/oracle/manifest.json`
  - `kid_x_pos=007F0000`, `kid_y_pos=00020000`, `gfx_slot_index=2`, `parent_object_ptr=4294906640`
- Baseline native still fails at the same pose:
  - `boot-title-start-level1`: `native 0089A000,00262000; oracle 007F0000,00020000`
  - `boot-title-start-level1-move-right5s`: same frame-520 pose
- Rejected local experiments:
  - creation-order + two-`Get_RandomNumber_long` pre-roll + 3-frame visible delay -> `00995000,00202000`
  - reverse-execution-with-number-first shell order also failed to hit the oracle slot-2 pose

# Exact Next Moves
- Stay on the mode-8 hand-off and inspect which non-letter shell object(s) can occupy the earliest gfx slots on the second live pass before the traced slot-2 glyph appears; use the oracle frame-520 `parent_object_ptr` evidence rather than another abstract shell-order guess.
- After that identity hypothesis, re-run `cargo run -q -p kidcli -- compare-native-oracle --scenario boot-title-start-level1` and `cargo run -q -p kidcli -- compare-native-oracle --scenario boot-title-start-level1-move-right5s`.
- Do not reopen broad title timing or raw reverse-letter mapping; the remaining gap is about matching the oracle's traced gfx-object identity.

# Verification To Re-run
- cargo test -q -p kid-core boot_title_start_stays_in_mode_8_without_second_start_press -- --nocapture
- cargo test -q -p kid-core level_entry_has_no_damage_grace_window -- --nocapture
- cargo test -q -p kidcli compare_native_states_reports_boot_level_entry_mode_8_release_gate -- --nocapture
- cargo run -q -p kidcli -- compare-native-oracle --scenario boot-title-start-level1
- cargo run -q -p kidcli -- compare-native-oracle --scenario boot-title-start-level1-move-right5s

# Dead Ends / Do Not Retry
- Do not retry the “creation-order workers plus two `Get_RandomNumber_long` pre-rolls and a 3-frame visible delay” patch; it worsened frame 520 to native `00995000,00202000`.
- Do not restore the old raw reverse-letter mapping (`visible_glyph_count - index - 1`); even after confirming reverse execution order, the simplified reverse shell still does not match the oracle slot-2 pose.
- Do not keep brute-forcing outer-frame delays on top of either simple shell-order model; the remaining gap is now a traced-gfx identity issue.
- Do not revisit title-card centring or the reverted deferred-spawn experiment.
