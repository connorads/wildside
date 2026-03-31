---
slug: release-gate-md-runtime-ctrl-pressed-mode8
cycle: 33
updated_at: 2026-03-31T04:26:15Z
---
# What Changed
- Traced the live object roster that survives into the cold-boot `Load_IntroSequence6 -> Obj_TitleMenu` hand-off.
- Read the surviving objects' callees:
  - `loc_1B6AC`
  - `sub_1B71A`
  - `loc_1C7A0`
  - `sub_1C034`
  - `sub_1C0EE`
  - `sub_1C112`
  - `loc_1BB10`
- Confirmed those surviving helpers are also RNG-silent.

# Current Model
- The cold-boot frame-360 RNG carry is not coming from the obvious title builders, and it is not coming from the objects that remain alive once `loc_1BAB0` hands off into `Obj_TitleMenu`.
- By the time the title shell reaches the visible menu/text helpers, every traced callee in that roster is RNG-silent.
- That pushes the carry bug one step earlier:
  - either an earlier still-live object on the pre-menu `Load_IntroSequence6` timeline is consuming RNG before it dies
  - or the carry comes from a non-title object lineage that survives longer than expected before the hand-off settles

# Strongest Evidence
- ASM:
  - `.cache/upstream/kid-chameleon-disasm/kid.asm:29917-29948` `loc_1B6AC`
  - `.cache/upstream/kid-chameleon-disasm/kid.asm:29957-30029` `sub_1B71A`
  - `.cache/upstream/kid-chameleon-disasm/kid.asm:30370-30458` `loc_1BAB0`, `Obj_TitleMenu`
  - `.cache/upstream/kid-chameleon-disasm/kid.asm:30405-30418` `loc_1BB10`
  - `.cache/upstream/kid-chameleon-disasm/kid.asm:30849-30952` `sub_1C034`, `sub_1C0EE`, `sub_1C112`
  - `.cache/upstream/kid-chameleon-disasm/kid.asm:31404-31457` `loc_1C7A0`
- Roster/lifetime facts from the ASM:
  - `sub_1B43A` allocates `GfxObjects_Collision_BottomBoundary4`, `loc_1B6AC`, `loc_1B5CE`, then deletes itself.
  - `loc_1B6AC` later allocates `loc_1BB10`, then `loc_1C7A0`, and keeps looping.
  - `loc_1BAB0` later allocates another `loc_1BB10` and `loc_1C7A0` pair before `loc_1BAEA`.
  - None of those labels or their immediate callees touch `Get_RandomNumber_*`.

# Exact Next Moves
- The remaining ambiguity is now genuine. Add one targeted probe or oracle-side diagnostic that captures which object lineage is still running between the last pre-title `Load_IntroSequence6` frames and frame 360, so you can stop inferring from static call trees alone.
- Keep it narrow: the probe should answer “which live objects still execute before frame 360?” rather than dumping broad unrelated state.

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
- Do not re-read the already-cleared title helpers looking for direct RNG calls; `loc_1B6AC`, `sub_1B71A`, `loc_1C7A0`, `sub_1C034`, `sub_1C0EE`, `sub_1C112`, `loc_1BB10`, and `Obj_TitleMenu` are all RNG-silent.
- Do not switch back to `Load_TitleScreen`; cold boot is still on the `Load_IntroSequence6` lineage here.
- Do not widen the probe beyond the pre-frame-360 live object question. The missing carry source is now a narrow lifetime problem.
