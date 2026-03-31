---
slug: release-gate-md-runtime-ctrl-pressed-mode8
cycle: 32
updated_at: 2026-03-31T04:25:05Z
---
# What Changed
- Traced the cold-boot title path in ASM instead of guessing from the offset delta.
- Confirmed from `GameLoadArray` that the cold-boot title shell is still the `Load_IntroSequence6` lineage before it reaches `Obj_TitleMenu`; it is not the separate `Load_TitleScreen` loader.
- Read the title-path helpers on that lineage:
  - `Load_IntroSequence6`
  - `sub_1B222`
  - `loc_1BAB0`
  - `GfxObjects_Collision_BottomBoundary4`
  - `sub_1B43A`
  - `loc_1B6AC`
  - `loc_1B5CE`
  - `Obj_TitleMenu`, `loc_1BB52`, `Title_InputLoop`
- None of those labels make a direct `Get_RandomNumber_*` call.

# Current Model
- The missing pre-frame-360 RNG carry does not come from the obvious cold-boot title-build helpers themselves.
- Cold-boot title is still the `Load_IntroSequence6` object lineage, but that lineage's visible helpers are RNG-silent in the disassembly.
- That leaves two plausible ROM-side sources for the carried offset:
  - earlier still-live object code running under the same `Execute_Objects` loop before the title shell settles
  - some non-title helper on the cold-boot path outside the labels above
- The `sub_1D846/sub_1D86C/sub_1D8B4` random-camera/random-spawn cluster is not on this path; its callers sit in the `loc_1D1D6` cinematic sequence, not the cold-boot title lineage.

# Strongest Evidence
- ASM:
  - `.cache/upstream/kid-chameleon-disasm/kid.asm:23558-23581` `LoadGameModeData`, `GameLoadArray`
  - `.cache/upstream/kid-chameleon-disasm/kid.asm:29366-29443` `Load_IntroSequence6`, `sub_1B222`
  - `.cache/upstream/kid-chameleon-disasm/kid.asm:29762-29779` `GfxObjects_Collision_BottomBoundary4`
  - `.cache/upstream/kid-chameleon-disasm/kid.asm:29792-29918` `sub_1B43A`, `loc_1B6AC`, `loc_1B5CE`
  - `.cache/upstream/kid-chameleon-disasm/kid.asm:30390-30470` `Obj_TitleMenu`, `loc_1BB52`, `Title_InputLoop`
  - `.cache/upstream/kid-chameleon-disasm/kid.asm:32562-32638`, `.cache/upstream/kid-chameleon-disasm/kid.asm:33070-33159` `loc_1D1D6`, `sub_1D846`, `sub_1D86C`, `sub_1D8B4` as the rejected unrelated RNG cluster
- Verified direct-random call inventory still only shows:
  - `Execute_Objects` at `kid.asm:1637`
  - title-card worker init at `kid.asm:28915-28946`
  - unrelated weather/cinematic/enemy routines elsewhere

# Exact Next Moves
- Stay on the cold-boot title carry problem only: trace which objects are still alive when `Load_IntroSequence6` hands into `Obj_TitleMenu`, then identify whether any of those objects call helpers outside the RNG-silent labels above.
- In parallel with that reading, inspect the mode-8 object roster after `Load_TitleCard` to replace the coarse `Execute_Objects` burn with the actual object activity, but do not change native code until the ASM names the real callers.

# Verification To Re-run
- rg -n "Get_RandomNumber|Random" .cache/upstream/kid-chameleon-disasm/kid.asm
- cargo test -q -p kid-core boot_title_start_mode_8_keeps_advancing_rng_before_gameplay_entry -- --nocapture
- cargo test -q -p kid-core boot_title_start_stays_in_mode_8_without_second_start_press -- --nocapture
- cargo test -q -p kid-core level_entry_has_no_damage_grace_window -- --nocapture
- cargo test -q -p kidcli compare_native_states_reports_boot_level_entry_mode_8_release_gate -- --nocapture
- cargo run -q -p kidcli -- compare-native-oracle --scenario boot-title-start-level1
- jq '.[] | select(.frame_index==360 or .frame_index==520 or .frame_index==719) | {frame:.frame_index,label:.label,rng_offset:.rng_offset}' .cache/native-oracle-runs/boot_title_start_level1/native/state.json
- jq '.states[] | select(.frame_index==360 or .frame_index==520 or .frame_index==719) | {frame:.frame_index,label:.label,rng_offset:.rng_offset}' .cache/native-oracle-runs/boot_title_start_level1/oracle/manifest.json

# Dead Ends / Do Not Retry
- Do not chase `Load_TitleScreen` for the cold-boot frame-360 carry. `GameLoadArray` shows the cold-boot title uses `Load_IntroSequence6`.
- Do not blame `Obj_TitleMenu`, `loc_1BB52`, `Title_InputLoop`, `loc_1B5CE`, `loc_1B6AC`, or `GfxObjects_Collision_BottomBoundary4` for direct RNG calls; those labels are RNG-silent.
- Do not chase the `sub_1D846/sub_1D86C/sub_1D8B4` random-camera cluster for this frontier. Its callers are on a different cinematic path.
