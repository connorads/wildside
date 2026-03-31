---
slug: release-gate-md-runtime-ctrl-pressed-mode8
cycle: 24
updated_at: 2026-03-31T03:44:21Z
---
# What Changed
- Re-read `loc_6E2`, `Mode_Standard`, `Execute_Objects`, `Allocate_ObjectSlot`, `Delete_CurrentObject`, `loc_1AB26`, and `loc_1AC70 -> loc_1AD2A` to pin down the first live mode-8 object passes after the title Start hand-off.
- Verified the frontier is still the same after restoring the tree to baseline:
  - `cargo test -q -p kid-core boot_title_start_stays_in_mode_8_without_second_start_press -- --nocapture`
  - `cargo test -q -p kid-core level_entry_has_no_damage_grace_window -- --nocapture`
  - `cargo test -q -p kidcli compare_native_states_reports_boot_level_entry_mode_8_release_gate -- --nocapture`
  - `cargo run -q -p kidcli -- compare-native-oracle --scenario boot-title-start-level1`
  - `cargo run -q -p kidcli -- compare-native-oracle --scenario boot-title-start-level1-move-right5s`
- Tried the obvious creation-order timing model locally and rejected it before landing: forcing a two-`Get_RandomNumber_long` pre-roll plus a 3-frame visible delay moved native to `00995000,00202000`, which is worse than baseline and still far from oracle.

# Current Model
- The remaining `level_entry@520` miss is not just the one-frame `loc_1AD2A` hibernate. The first visible slot-2 glyph still does not come from the native creation-order worker model at the correct RNG phase.
- ASM now narrows the problem to the early mode-8 object-list execution:
  - `loc_6E2` initialises slots and loads mode-8 data before the next `WaitForVint`.
  - The first `Mode_Standard -> Execute_Objects` pass burns one `Get_RandomNumber_long` and runs the initial mode-8 objects.
  - `loc_1AB26` allocates the `loc_1AC70` workers, but `Delete_CurrentObject` returns to `loc_E1E` from the predecessor link, so newly inserted equal-priority workers may not become trace-visible in the simple creation order that native currently assumes.
- The likely missing piece is therefore object-list order, not just a shared outer-frame count: which worker first receives a gfx slot on the second live pass, and whether trace slot 2 at frame 520 comes from a later-created glyph than the current `gfx_slot_index == visible_glyph_index` model.

# Strongest Evidence
- ASM:
  - `.cache/upstream/kid-chameleon-disasm/kid.asm:823-920` `loc_6E2` / `Mode_Standard` shows the `WaitForVint` boundary and that `Execute_Objects` runs once per outer frame.
  - `.cache/upstream/kid-chameleon-disasm/kid.asm:1559-1616` `Allocate_ObjectSlot` inserts equal-priority objects ahead of existing ones.
  - `.cache/upstream/kid-chameleon-disasm/kid.asm:1636-1694` `Execute_Objects` / `Hibernate_Object_1Frame` shows the per-pass `Get_RandomNumber_long` burn and the hibernation semantics.
  - `.cache/upstream/kid-chameleon-disasm/kid.asm:1718-1775` `Delete_CurrentObject` restores iteration from the predecessor link, which explains why `loc_1AB26`-spawned workers are not guaranteed to seed in plain creation order.
  - `.cache/upstream/kid-chameleon-disasm/kid.asm:28695-29064` `loc_1AB26`, `loc_1AC70`, `loc_1AD48`, `loc_1AE2E`.
- Baseline verification still fails at the same checkpoint after restoring the tree:
  - `boot-title-start-level1`: `state level_entry@520 diverged in kid position (native 0089A000,00262000; oracle 007F0000,00020000)`
  - `boot-title-start-level1-move-right5s`: same frame-520 pose
- Rejected local experiment:
  - A creation-order model with two `Get_RandomNumber_long` pre-roll calls and a 3-frame visible delay still produced slot-2 pose `00995000,00202000`, not the oracle `007F0000,00020000`.
  - A quick brute-force over small creation-order pre-roll/delay combinations also failed to reproduce the oracle slot-2 pose, which is another sign that the wrong worker/order is being traced.

# Exact Next Moves
- Stay on the mode-8 hand-off and read the object-list iteration consequences of `Allocate_ObjectSlot` + `Delete_CurrentObject` around `loc_1AB26`, then determine which `loc_1AC70` worker actually becomes trace-visible as gfx slot 2 on the second live `Execute_Objects` pass.
- After that object-order hypothesis, re-run `cargo run -q -p kidcli -- compare-native-oracle --scenario boot-title-start-level1` and `cargo run -q -p kidcli -- compare-native-oracle --scenario boot-title-start-level1-move-right5s`.
- Do not reopen broad title timing, centring, or the raw one-frame `loc_1AD2A` delay; this frontier is now about early mode-8 object execution order.

# Verification To Re-run
- cargo test -q -p kid-core boot_title_start_stays_in_mode_8_without_second_start_press -- --nocapture
- cargo test -q -p kid-core level_entry_has_no_damage_grace_window -- --nocapture
- cargo test -q -p kidcli compare_native_states_reports_boot_level_entry_mode_8_release_gate -- --nocapture
- cargo run -q -p kidcli -- compare-native-oracle --scenario boot-title-start-level1
- cargo run -q -p kidcli -- compare-native-oracle --scenario boot-title-start-level1-move-right5s

# Dead Ends / Do Not Retry
- Do not restore the reverse `visible_glyph_count - index - 1` slot mapping; the earlier ASM-backed slot work already ruled that out.
- Do not retry the “creation-order workers plus two `Get_RandomNumber_long` pre-rolls and a 3-frame visible delay” patch; it worsened frame 520 to native `00995000,00202000`.
- Do not keep brute-forcing outer-frame delays on top of the current creation-order model; the slot-2 mismatch now points to object-order/visibility, not another shared timer off-by-one.
- Do not revisit worker-slot mapping, title-card centring, or the reverted deferred-spawn experiment.
