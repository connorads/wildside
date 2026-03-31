---
slug: release-gate-md-runtime-ctrl-pressed-mode8
cycle: 16
updated_at: 2026-03-31T03:02:23Z
---
# What Changed
- Re-ran the `loc_1AC70` investigation against the frame-520 oracle artefacts instead of patching blind.
- Proved the first visible mismatch is not `loc_1AD2A`'s per-axis maths: with the oracle frame-520 slots interpreted against the current target map, every letter worker is exactly 16 hibernation steps into the `loc_1AD2A` march and the deferred `loc_1AD48` number worker is 15 steps in.
- Proved the native synthetic shell is still far too early: the same inversion against native frame 520 lands every worker at 60 `loc_1AD2A` steps, which matches the current `BootState::StartingLevel.elapsed_frames` feed and explains the low/native-near-target pose.

# Current Model
- `loc_1AD2A`'s movement shell is structurally fine; the current trace bug is the timeline feeding it.
- The synthetic title workers are being sampled from the moment `begin_level_entry` flips into `BootState::StartingLevel`, but the oracle shows a 44-frame pre-roll before the letter workers have accumulated the frame-520 16-step march.
- Do not patch this with a raw `-44` offset yet. Oracle slot-2 also implies a later RNG state than the native precomputed worker list, so there is likely missing pre-`loc_1AC70` work between title Start and the first worker execution.

# Strongest Evidence
- ASM `.cache/upstream/kid-chameleon-disasm/kid.asm:28974-28986` `loc_1AD2A` is a simple one-frame hibernate loop with per-axis adds; nothing there justifies the native frame-520 overshoot.
- ASM `.cache/upstream/kid-chameleon-disasm/kid.asm:28750-28912` shows `loc_1AB26` only allocates the worker objects, while each `loc_1AC70` object consumes RNG when it first runs. Native still collapses that into one precomputed pass at `begin_level_entry`.
- `cargo run -q -p kidcli -- compare-native-oracle --scenario boot-title-start-level1` still fails first at `state level_entry@520 diverged in kid position (native 0042C000,005F8000; oracle 007F0000,00020000)`.
- Inverting the oracle frame-520 active-enemy slots against the current target map yields unique `loc_1AD2A` counts of 16 for slots 0-12 and 15 for slot 13; doing the same for native frame 520 yields 60 for every slot, matching the current synthetic `elapsed_frames` feed.

# Exact Next Moves
- Stay on `loc_1AC70`: find the missing pre-roll between `begin_level_entry` and the first worker execution by tracing the level-entry shell around `loc_19C02 -> loc_1AB26 -> loc_1AE2E`, and identify what should gate worker visibility/RNG consumption before `loc_1AD2A` starts counting.
- Re-run `cargo run -q -p kidcli -- compare-native-oracle --scenario boot-title-start-level1` after each hypothesis check; only touch native code once the missing pre-roll source is named from ASM.
- Do not land a magic elapsed-frame offset or new debug fields yet. The ASM still needs one more pass.

# Verification To Re-run
- cargo test -q -p kid-core boot_title_start_stays_in_mode_8_without_second_start_press -- --nocapture
- cargo test -q -p kid-core level_entry_has_no_damage_grace_window -- --nocapture
- cargo test -q -p kidcli compare_native_states_reports_boot_level_entry_mode_8_release_gate -- --nocapture
- cargo run -q -p kidcli -- compare-native-oracle --scenario boot-title-start-level1
- cargo run -q -p kidcli -- compare-native-oracle --scenario boot-title-start-level1-move-right5s

# Dead Ends / Do Not Retry
- Do not blame `loc_1AD2A`'s axis maths again. Oracle/native inversion already showed the loop count is the first mismatch.
- Do not patch with `elapsed_frames - 44` yet. That would hide the timing miss while leaving the RNG-order problem unexplained.
- Do not switch back to allocation-order assumptions for RNG consumption without proving when the first `loc_1AC70` object actually executes.
