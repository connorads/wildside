<!-- Generated 2026-05-31 by the `platforms-helmets-weather-fidelity-audit` workflow (1 read-only Explore scout enumerated the helmet/form/Diamond-Power units; one read-only Explore auditor per unit compared the live native path to kid.asm; every high/medium gap adversarially verified by an independent refuter — defaulting to refuted — before inclusion; low gaps included without a refute pass). This is a reviewable research artefact, not a gate. Each gap cites the owning ASM label and the native `file:line` searched. Re-run the workflow to refresh. -->
<!-- Staleness-checked 2026-06-10: of 38 gaps, 10 fixed, 3 partially fixed, 25 still open — verdicts and evidence in fidelity-audit-staleness-2026-06-10.md. Still-open entries below remain authoritative; fixed entries are superseded. -->

# Helmet / Form / Diamond-Power Fidelity Audit

Adversarially-verified audit of the Kid Chameleon source-port's **helmet, form, and Diamond-Power** subsystem (`helmets.rs`, with `combat.rs` transform/HP, `projectiles.rs` weapons, `constants.rs`), comparing the live path to the `kid.asm` `Kid_Transform`, `DiamondPower_*`, and per-form weapon routines. Form *movement physics* is covered by `movement-fidelity-audit.md`; this audit covers transforms, abilities, and weapons.

The recurring failure mode is the same one the other audits found crate-wide: **ASM-faithful pure helpers exist and are unit-tested, but carry `#[cfg_attr(not(test), allow(dead_code))]` (or `#[cfg(test)]`) and are never called from the live path.** The Diamond-Power subsystem is the worst-hit area in the whole codebase by this measure — almost every per-power ROM routine is modelled and tested, yet the runtime ignores all of it in favour of flat timers, enum matches, and screen-wide approximations.

Findings that stand out:

- **Architectural — the entire Diamond-Power object model is dead.** The ROM allocates `$FFFFF2AC` slot objects threaded through free/active lists, advances each per-frame via the `DiamondPower_Main_Index` jump table, and tears them down on a status word (`kid.asm:44233-44441`). Native replaces all of it with a flat `diamond_power_frames` timer and a Rust `match diamond_power_type` (`helmets.rs:5918-5929`). Every slot/list/compile helper (`helmets.rs:1009-1155`) is test-only.

- **Circle of Doom and Wall of Death are an invented screen-wide kill-all.** Both route through `diamond_power_kill_enemies()` (`helmets.rs:6121`), which applies damage to *every* non-Diamond enemy each frame — no orbit, no sweeping wall, no per-slot collision geometry. Enemies off-screen and outside the ring/column die instantly, and the powers auto-expire on an invented 480-frame cap rather than when their slots shrink away. The faithful orbit/sweep models exist but are dead.

- **Snake Diamond Powers lose all homing.** The ROM's 6-segment chain with trail propagation, nearest-enemy acquisition, and per-form angle steering (`kid.asm:44621-44892`) is replaced by a single forward projectile every 8 frames (`helmets.rs:6013-6043`). Death Snake homing is entirely absent, and Micromax's distinct SwiftMiniSnake parameters are collapsed into DeathSnake.

- **The Eyeclops beam is modelled as a single point projectile** with invented speed/lifetime (`constants.rs:655-656`), instead of the ROM's growing multi-segment beam that damages every enemy along a full-screen horizontal line (`kid.asm:17999-18686`).

- **Maniaxe correctness issues are mostly closed:** the thrown axe now has no generic DBF lifetime, survives until enemy/solid/camera cull, uses the solid-only `$6000` terrain delete gate, reverses the live `ManiaxeAxe` projectile instead of `last_mut()`, and advances its `stru_8B86` animation clock for spinning native-player art. Remaining Maniaxe rows below are timing/architecture nits, not the former disappearing/static/green-dot failures.

- **ExtraHitPoint Diamond Power is live for the ROM reward orbit/cap path:** native now seeds the ten `DiamondPower_Main_ExtraHPLife` slots and awards HP only on `loc_3FC2A` convergence, incrementing `Extra_hitpoint_slots` while respecting the `< 4` cap.

The transform/HP core (`Kid_Transform`, `sub_7EB2`) is, by contrast, well-wired — its remaining gaps are mostly audio (transform SFX, music-speed) and benign hygiene where `helmet.reset()` is a superset of the ROM's per-flag clear.

## Coverage summary

| Unit | Coverage | Confirmed gaps |
|---|---|---|
| Helmet pickup gate + transform dispatch (`Check_Helmet_Change` / `Kid_Transform`) | partial | 2 |
| Transform hitpoints (`sub_7EB2`) + per-helmet base HP table (`unk_7EC2`) | partial | 1 |
| Post-transform cleanup, half-width (`sub_B41C`), transient-flag clear (`sub_DB22`) | partial | 4 |
| Diamond Power activation gate + two-tier cost (`DiamondPower_Run`/`_Check`) | partial | 4 |
| Diamond Power object list + sprite compile (`Addr_*DPObjectSlot`, `sub_3F5xx`) | stub | 3 |
| Circle of Doom / Wall of Death (geometry vs kill-all AoE) | stub | 3 |
| Snake Diamond Powers (Mini/Death/Swift/Death2) and tracking | stub | 4 |
| Slashing Rain / Tracking Rain Diamond Power | partial | 3 |
| Five-Way Shot (Juggernaut) Diamond Power | partial | 3 |
| Eyeclops light beam (normal C-press vs hard A+Start) | partial | 4 |
| Maniaxe thrown axe + Red Stealth sword | partial | 7 |

Totals: 11 units audited, 0 full, 8 partial, 3 stub. **38 confirmed gaps** (10 high, 9 medium, 19 low).

## Confirmed gaps

### High severity

**RESOLVED candidate 2026-06-19 — ExtraHitPoint Diamond Power awards through the `loc_3FC2A` slot convergence path and honours the four-slot cap** (`DiamondPower_Main_ExtraHPLife / loc_3FC2A…loc_3FC52`, `DiamondPower_Init_ExtraHitPoint`)
The ROM awards the IronKnight tier-2 +HP only when a converging slot lands, and only if `Extra_hitpoint_slots < 4`: `cmpi.w #4 … bge; addq.w #1,(Extra_hitpoint_slots); addq.w #1,(Number_Hitpoints)` (`kid.asm:44965-44968`). Native now seeds the ten orbiting reward slots from `DiamondPower_Init_ExtraHitPoint loc_3FB90/loc_3FB96`, renders them through `DiamondPower_Main_ExtraHPLife loc_3FBE4`, and only mutates `extra_hitpoint_slots`/`number_hitpoints` on convergence while respecting the ROM cap. Covered by `extra_hit_point_activation_waits_for_orbit_convergence_loc_3fb96_loc_3fc2a`, `extra_hit_point_activation_respects_four_slot_cap_loc_3fc4c`, and the pure `diamond_power_main_extra_hp_life_*` tests.

**Diamond Power — entire object linked-list (alloc/remove/reset/init/clear) is dead; live path uses a flat timer** (`sub_3F586/3F596/3F5BC/3F5E8/3F63C`)
The ROM threads each spawned DP object through the `$FFFFF2AC` slot pool with free/active/last lists, self-removal on status word `$38`, and `Diamond_power_active` maintenance that clears `FiveWayShotReady`/`SamuraiHazeActive`/`KidIsInvulnerable` when the list empties (`kid.asm:44233-44315`). Native `step_diamond_power` (`helmets.rs:5918`) models none of it: a flat `diamond_power_frames` timer (`6117`) + a `diamond_power_type` enum, with no slot pool, lists, or self-removal. The faithful helpers (`helmets.rs:1009,1026,1045,1077,1116`) are all `allow(dead_code)`, test-only (`combat_tests.rs:2041-2305`).

**Diamond Power — per-frame `DiamondPower_Main_Execute` / `Main_Index` dispatch omitted** (`loc_3F71E`, `off_3F748`, `DiamondPower_Run`)
Every frame the ROM indexes the 20-entry `DiamondPower_Main_Index` jump table by `Diamond_power_ID` and runs the per-power worker, which iterates the object list and advances each object (`kid.asm:44331-44416`). Native replaces this with a hardcoded `match self.progression.diamond_power_type` (`helmets.rs:5923`) over a separate enum and ad-hoc per-arm logic. The ID→worker dispatch and object-list iteration are lost; there is no `diamond_power_id` field.

**Circle of Doom — live path is an invented kill-all AoE; geometric orbit unwired** (`DiamondPower_Main_CircleOfDoom loc_3F818`, `DiamondPower_Init_CircleOfDoom loc_3F7D0`)
The ROM spawns 8 slots on a polar orbit (angle step `$200000`, radius `$00210000` shrinking `-$400`/frame, angle `-4`/frame), picks a sprite size from `byte_3F8B4`, and damages enemies via the orbiting **objects'** collision — no kill-all; the power ends when all 8 radii deplete (no timer) (`kid.asm:44469-44558`). Native routes CircleOfDoom (and WallOfDeath) through `diamond_power_kill_enemies()` (`helmets.rs:6121-6137`), applying `take_damage_or_queue_death()` to every non-Diamond enemy each frame — no geometry, orbit, radius, or per-slot collision — and caps duration at an invented 480 frames (`helmets.rs:961`). Off-ring and off-screen enemies are killed. Faithful `diamond_power_init_circle_of_doom_sub_3F7D0` (`helmets.rs:1218`) and `…_main_circle_of_doom_slot` (`1278`) are dead.

**Wall of Death — live path is an invented kill-all AoE; sweeping wall geometry unwired** (`DiamondPower_Main_WallOfDeath loc_4001E`, `DiamondPower_Init_WallOfDeath loc_3FFE4`)
The ROM spawns 10 slots forming a vertical wall that sweeps right (`+8`) to `x≥$139` then left (`-8`) to `x≤7` while `Diamond_power_timer=$4B0` counts down, then fades by decrementing size every 8th frame, damaging via the swept **objects** (`kid.asm:45368-45458`). Native routes WallOfDeath through the same `diamond_power_kill_enemies()` (`helmets.rs:6121`) — no wall, sweep, per-slot position, or fade. Enemies not yet swept (and off the column) die instantly. Faithful `diamond_power_init_wall_of_death` (`helmets.rs:1986`) and `…_main_wall_of_death_slot` (`2074`) are dead.

**Snake Diamond Power — entire 6-segment chain replaced by a single forward projectile (homing/trail omitted)** (`DiamondPower_Init_Snake loc_3F91C`, `DiamondPower_Main_Snake`)
The ROM spawns a 6-slot chain, each with a 3-entry trail buffer; each frame it propagates the trail, finds the nearest on-screen enemy by Manhattan distance (falling back to Kid), steers the head's angle toward the target octant with per-form turn-rate, moves by `sub_40236` sin/cos plus half of Kid's velocity, and compiles a sprite per segment (`kid.asm:44621-44892`). The live path (`helmets.rs:6013-6043`) pushes one `EnemyProjectile` straight in Kid's facing every 8 frames — no chain, trail, target acquisition, homing, Kid-velocity coupling, or per-segment sprites. Death Snake homing is entirely absent. Faithful helpers (`helmets.rs:1425,1485,1540,2460,2539,3583`) are all dead.

**Swift Mini-Snake (Micromax T2) distinct parameters collapsed into generic DeathSnake** (`DiamondPower_Init_SwiftMiniSnake loc_3F8D0`, `InitData_SwiftMiniSnake`)
The ROM gives Micromax tier-2 the SwiftMiniSnake init (wander `$10`, speed `$480`, size table `5,4,3,2,1,1`), distinct from DeathSnake (wander 6, speed `$400`, sizes `6,6,5,4,3,2`) (`kid.asm:44590-44652`). Native `diamond_power_for_form` maps Micromax(true) directly to `DiamondPowerType::DeathSnake` (`helmets.rs:953`), with no SwiftMiniSnake/DeathSnake2 variant, so Swift's higher speed, faster wander, and distinct taper are lost; the live speed select distinguishes only Mini vs Death (`6024-6029`).

**Five-Way Shot — faithful init/main helpers exist but are dead; live path reimplements with a divergent world-space projectile model** (`DiamondPower_Init_FiveWayShot loc_400DE/400F4`, `DiamondPower_Main_FiveWayShot loc_4014C`)
The ROM spawns 5 slots with **screen-relative** positions and **integer screen-pixel** velocities (6/7/8 px/frame from `word_40124`), advancing and **bouncing them off the screen edges**, then shrinking size every 8 frames once the timer goes negative (`kid.asm:45461-45564`). Native models all of this faithfully (`helmets.rs:2233,2332`) but those are `allow(dead_code)`, test-only. The live runtime (`helmets.rs:5973-6011`) pushes 5 generic `EnemyProjectile` in **world** coordinates with velocity `i32::from(x_vel) << 16` — reinterpreting screen-pixel velocity as 16.16 world fixed-point — so shots fly straight off in world space with no edge bounce, no camera-follow, and no size-shrink/removal lifecycle (flat `lifetime:80`). The fan position and per-shot magnitudes happen to be correct.

**Eyeclops — beam modelled as a single point projectile instead of the ROM's growing multi-segment screen-wide beam** (`sub_E49A / loc_E4B2 / loc_E51A / loc_E650 / loc_E8A6`, `kid.asm:17999-18686`)
The ROM `sub_E49A` grows the beam each frame (indexing `off_E560` then procedural extension), extending it segment-by-segment from Kid's mouth to the screen edge and damaging every enemy along its full horizontal length via foreground sprite/tile collision (IDs `$127..$12C`). Native pushes one point `EnemyProjectile` with `PROJECTILE_HITBOX_HALF` extent (`helmets.rs:5522-5535`) flying at fixed velocity, hitting only one enemy near the travelling point and never growing/sweeping. The faithful render helpers (`helmets.rs:53,137`; `level.rs:15748,16203`) are dead (test-only).

**Eyeclops — invented `BEAM_SPEED`/`LIFETIME` with no ASM basis** (`sub_E49A loc_E7A4 / loc_E988`)
Native gives the beam `EYECLOPS_BEAM_SPEED = 0x8_0000` (8px/frame) and `EYECLOPS_BEAM_LIFETIME = 60` (`constants.rs:655-656`). The ROM beam has no travel velocity and no 60-frame lifetime: it grows a fixed segment width per frame and is torn down the frame it reaches the screen boundary (`loc_E7A4`/`loc_E988` clear `$FFFFFAB8`, `kid.asm:18476,18685`). Beam duration is a function of screen distance, not a hardcoded timer, and the 8px/frame speed is fabricated; these constants set the wrong expiry timing (`helmets.rs:5512-5532`).

### Medium severity

**Juggernaut 5-way arming — state-machine flag fused into one-frame activation (`FiveWayShotReady` never modelled as a persistent flag)** (`sub_7428`, `DiamondPower_Run Juggernaut branch`, `DiamondPower_Init_FiveWayShot`)
In the ROM the 5-way is a two-stage handshake: `sub_7428` sets the persistent `FiveWayShotReady` byte only from specific player-update states (and only Juggernaut+Start+A+≥5 diamonds); the cost is paid later in init, reached only when the flag is set, and a Juggernaut without the flag is entirely excluded from the Start+A/20-diamond path (`kid.asm:7569-7582,44333-44338,45461-45464`). Native (`helmets.rs:6068-6076`) re-evaluates the readiness inline from the current frame's input with no state gating or persistent flag, so the shot can fire in states where the ROM never calls `sub_7428`. Net diamond cost is preserved; the divergence is timing/state-gating.

**Circle/Wall — diamond objects not spawned; sprites and per-slot collision omitted** (`DiamondPower_CompileSprites loc_3F798`, `loc_3F880 / loc_4009C`)
Both powers compile their slots into the sprite buffer each frame and rely on those objects existing as collision entities (`kid.asm:44441,44538-44553,45446-45453`). The runtime never creates any circle/wall slot list (unlike rain), so the orbiting/sweeping diamonds are visually absent and the ROM damage vector (object collision) is replaced wholesale by the kill-all. The init helpers return `Vec<…Slot>` assigned to no player field (`helmets.rs:107`).

**Snake live-path firing cadence and lifetime are invented (no ASM basis)** (`DiamondPower_Main_Snake`, `loc_3FB20`)
The live approximation fires a projectile when `diamond_power_frames % 8 == 0` (`helmets.rs:6017`) with `lifetime=90` (`6039`). The ROM snake fires no discrete projectiles — it maintains a persistent 6-segment chain whose lifetime equals `Diamond_power_timer` (`$1E0` Mini / `$4B0` Death/Swift), pruned via `(Time_Frames)&$F==0` (`kid.asm:44850-44856`). The 8-frame interval and 90-frame lifetime are fabricated.

**Five-Way Shot — projectile lifecycle (timer-gated size shrink + removal) omitted from live path** (`DiamondPower_Main_FiveWayShot loc_4019A`)
Once `Diamond_power_timer` (init `$50`=80) passes zero, the ROM decrements each slot's size every 8 frames and removes it on underflow (`kid.asm:45556-45564`). The live path gives each projectile a flat `lifetime:80` (`helmets.rs:6006`) and never reproduces the staged shrink-then-remove. The faithful logic (`helmets.rs:2378-2389`) is dead.

**Eyeclops — hard-beam 2-diamond cost spent at arm-time in wrong place vs ROM render-time spend; counter semantics not driven by `sub_E49A`** (`loc_7606`, `sub_E49A loc_E7AC`)
In the ROM the standing/walking/jump handlers only **arm** the hard beam (`$8001 → $FFFFFAB8`) without spending diamonds; the 2-diamond deduction happens at render time inside `sub_E49A` once the beam reaches its width-0 point (`loc_E7AC`, `kid.asm:18495`). Native spends the 2 diamonds immediately at arm time (`helmets.rs:5551`) and sets `eyeclops_beam_counter = 0x8001` as a static mirror (`5553`); because `sub_E49A` never runs, the counter is bumped by a placeholder `wrapping_add(1)` (`5578`). The hard-beam destructive ball object (`$FFFFFABA`) is not modelled.

**RESOLVED — Maniaxe axe has no generic lifetime cap** (`ManiaxeAxe_Init / loc_8A52`, `kid.asm:9868-9911`)
The ROM flight loop is unbounded: the axe only dies on a solid block, leaving the camera box, or hitting an enemy, and the turnaround counter resets perpetually so the boomerang oscillates while on screen. Native `ManiaxeAxe` now spawns with `lifetime = 0`, is exempt from generic zero-lifetime deletion/decrement, and is culled by the Maniaxe-specific camera/solid/enemy paths. Covered by `maniaxe_axe_spawn_has_no_generic_lifetime_loc_8a52`, `maniaxe_axe_zero_lifetime_survives_inside_camera_bounds_loc_8a52`, and the focused `maniaxe_axe` test group.

**RESOLVED — Maniaxe axe terrain delete is solid-only** (`loc_8ACC`, collision test `kid.asm:9905-9911`)
The ROM tests `andi.w #$7000,d7; cmpi.w #$6000,d7; beq` — only collision nibble 6 (fully solid) deletes the axe; slope/other tiles let it pass. Native now special-cases `EnemyProjectileKind::ManiaxeAxe` in the terrain contact path so `$4000/$5000` slopes survive and `$6000` solid deletes. Covered by `maniaxe_axe_slope_contact_survives_loc_8a84` and `maniaxe_axe_solid_contact_deletes_loc_8acc`.

**RESOLVED — Maniaxe axe turnaround targets the live axe object, not `projectiles.last_mut()`** (`loc_8A74`, `kid.asm:9879`)
The ROM turnaround flips the axe object itself (`a3` = the specific slot). Native now locates the active `EnemyProjectileKind::ManiaxeAxe` when the turnaround counter expires, so unrelated projectiles spawned later in the vector are not reversed. Covered by `loc_8a74_turnaround_reverses_live_axe_not_last_projectile`.

**RESOLVED — Red Stealth airborne down-slash uses the dynamic jump-launch velocity** (`loc_A466` / `loc_B334 / loc_B350 / loc_B380`, `kid.asm:13965-13973`, `kid.asm:14208-14248`)
The ROM stores the jump-launch y-velocity at `$FFFFFAAA` in `loc_A466`, then gates the downward-slash by comparing `(y_vel << 2)` against that stored threshold. Native now captures that launch velocity in `red_stealth_jump_threshold_y_velocity` and `step_red_stealth_airborne_slash` uses it instead of the old hardcoded `0x0001_0000` fallback. Covered by `red_stealth_airborne_slash_uses_loc_a466_dynamic_jump_threshold`.

### Low severity

**Form change — slope-bypass `loc_7AD6` reconstructed from slope-cell probe rather than `Character_Movement==6`** (`loc_7AD6`, `kid.asm:8186`)
The ROM skips the `sub_7B30` fit-check entirely when `Character_Movement == 6` (slope state). Native has no movement-state word; `slope_transform_bypass_loc_7ad6` (`lib.rs:2408`) reconstructs the condition via a floor-probe heuristic (`on_ground && x_velocity!=0 && slope cell`), which can diverge at slope edges where the ROM's state and the probe disagree.

**Form change — invented +1 cleanup frame appended to transform hibernate count** (`loc_7D0A / loc_7E14 / loc_7E90`)
Native sets `form_change_flash_frames = old_frames + new_frames + 1` (`combat.rs:8406`). The per-entry `mode` sums are faithful, but the forward loop hibernates exactly `sum(mode)` frames with no additional iteration; the extra `+1` (a parity tweak for `loc_7E90`'s final clear) lengthens the freeze by one frame vs the ROM.

**Transform HP — `$FFFFFA64` skip is call-site-implicit, not a modelled guard** (`sub_7EB2`, dying-flag gate `kid.asm:8382-8383`, `loc_B7EA`, `lose_life`)
The ROM gates `sub_7EB2` behind the `$FFFFFA64` dying flag, so the death helmet-strip transform skips the HP reset (HP set to 2 by `lose_life`). Native `transform_hitpoints_sub_7eb2` (`combat.rs:117-121`) is unconditional but only reached via the normal prize-pickup path; the death path uses `enter_death_hidden_state` (`combat.rs:9095-9104`) which sets HP directly. No divergence today, but the skip relies on call-site routing rather than the explicit gate.

**`sub_DB22` — faithful clear helper is `#[cfg(test)]`-gated; live path uses `helmet.reset()` (broader, acceptable)** (`sub_DB22`, `kid.asm:17170`)
`Kid_Transform` calls `sub_DB22` to clear a specific set of transient flags. The faithful `clear_special_movement_state_sub_DB22` (`helmets.rs:4220`) is `#[cfg(test)]` and never compiled into the runtime; the live path calls `self.helmet.reset()` (`combat.rs:8497`) = full `HelmetState::default()`, a superset. Benign, but the dead helper is the canonical failure-mode shape.

**Transform SFX — per-helmet transform sound (`unk_7ED8`), `sfx_Replenish_Health`, and `sfx_Voice_bummer` now core-queued** (`unk_7ED8`, `loc_7E14/loc_7E40`, `loc_7C76`)
The ROM transform plays three sounds (per-helmet transform, voice-bummer back to Kid, replenish-health on same-helmet pickup). Native now queues those via `resolve_pending_form_change` and `transform_sound_effect_loc_7cd0`, including the `Demo_Mode_flag` branch that skips the per-helmet cue before `loc_7CFE`.

**Music-speed selection (`unk_7ECC` / `MusicSpeed_Murderwall`) now queued for native audio** (`loc_7E14/loc_7E26`, `unk_7ECC`)
The ROM selects a post-transform music-speed byte by theme (or murderwall) and applies it via the GEMS driver (`kid.asm:8562-8567`). Native now records the byte in `played_music_speed_updates` and native-player replays it as the GEMS `FF 05 <speed>; FF 0D` sequence.

**`clr.w ($FFFFFB70)` walk/animation accumulator not modelled or cleared at transform** (`loc_7E48`, `kid.asm:8580`)
The ROM clears the `$FFFFFB70` walk/animation accumulator at transform. Native models no such per-frame accumulator (`combat.rs:8421` hand-waves it), so there is nothing to clear and no observable divergence.

**Tier-2 snake dispatch collapses SwiftMiniSnake into DeathSnake (Micromax T2)** (`DiamondPower_Init_Index +$A`, `DiamondPower_Init_SwiftMiniSnake`)
The `+$A` tier-2 index math is correct (`helmets.rs:934-955`), but Micromax T2 should index SwiftMiniSnake (sizes `5,4,3,2,1,1`; speed `$480`) not DeathSnake (`6,6,5,4,3,2`; `$400`). Native maps it to `DeathSnake` and the live spawner distinguishes only Mini(`0x300`) vs else(`0x400`) with `lifetime:90`. (Skycutter T2 DeathSnake2 is legitimately `bra DeathSnake`, so that collapse is faithful.)

**RESOLVED — CircleOfDoom lifetime is owned by the DP-object list, not a 480f timer** (`DiamondPower_Init_CircleOfDoom loc_3F7D0`)
`DiamondPower_Init_CircleOfDoom` never writes `Diamond_power_timer` — the orbiting power runs from the allocated DP-object list until cleared elsewhere. Native now leaves `diamond_power_frames` at zero for CircleOfDoom and treats the eight live `diamond_power_circle_slots` as the active lifetime, removing the invented 480-frame expiry. Covered by `circle_of_doom_activation_uses_slot_lifetime_not_timer_loc_3f7d0`.

**Diamond Power — `DiamondPower_CompileSprites` (`sub_3F798`) sprite-table compile not wired** (`loc_3F798`)
The ROM walks the DP sprite buffer emitting VDP entries, fixing each link byte by the running `Number_Sprites` count and masking pattern bit 15. Native does not model the DP sprite buffer (`lib.rs:1746-1750` states it is "not modelled"). Faithful `diamond_power_compile_sprites_sub_3F798` (`helmets.rs:1155`) is dead. Visual/VDP.

**Snake invulnerability set ad hoc in live path; full snake state untracked** (`DiamondPower_Main_Snake`)
The live path forces `invulnerable_frames = max(1)` each frame (`helmets.rs:6015-6016`) as a stand-in; the ROM snake routine does not itself set Kid invulnerability. There is no live snake-slot field, so the faithful chain-state model is unreachable.

**RESOLVED — Rain non-tracking Y movement uses the ROM's word-only update** (`loc_3FF70` vs `loc_3FE98`)
The ROM updates non-tracking rain Y as **word-only** (subpixel low word untouched) while tracking uses 32-bit (`kid.asm:45226,45317`). Live `step_diamond_power_rain_slots` now routes non-tracking slots through `diamond_power_rain_non_tracking_move`, preserving the low word while tracking slots keep the 32-bit velocity path. Covered by `diamond_power_rain_live_non_tracking_slot_uses_word_y_update_loc_3ff70`.

**RESOLVED — Rain has no duplicate generic projectile cull path** (`loc_3FF90/loc_3FFC0`)
Rain now lives only in the dedicated `diamond_power_rain_slots` DP object-list model; the unused `EnemyProjectileKind::DiamondPowerRain` carrier, its Rain-only generic projectile fields, and the duplicate `step_projectiles` rain movement/cull branches were removed. This leaves `step_diamond_power_rain_slots` as the single live implementation for `loc_3FF90/loc_3FFC0`.

**Rain tracking — enemy target subpixel truncated before velocity computation** (`loc_3FF4A`, `kid.asm:45300-45308`)
The ROM computes tracking velocity from the enemy's full 32-bit position. Native reconstructs the target from the word-only `find_nearest_enemy` result (`(target.x as i32 as u32) << 16`, `projectiles.rs:1517-1518`), zeroing the enemy's subpixel; the delta is then `asr` by ≥1, so the lost low word affects at most the sub-LSB. Negligible precision divergence.

**Five-Way Shot — cost gate routed through generic `DiamondPower_Check` (5-vs-20 branch) instead of Juggernaut's dedicated init; 20-cost branch is dead** (`DiamondPower_Run Juggernaut branch → DiamondPower_Init_FiveWayShot subq #5`, `sub_7428`)
In the ROM Juggernaut never enters the 20-diamond path — `DiamondPower_Run` diverts it straight to `DiamondPower_Init_FiveWayShot` (`subq #5`). Native funnels Juggernaut through the shared `DiamondPower_Check` decomposition (`helmets.rs:6056-6092`); because the outer gate already requires Start+A, the readiness helper always returns true, so cost is effectively always 5 and the `else 20` branch (`6074`) is unreachable for Juggernaut. Lands on the correct numeric result.

**Eyeclops — beam-render direction latch `$FFFFFABE` modelled only as instantaneous velocity sign, not a persistent render latch** (`loc_E4B2`, `kid.asm:18013`)
The ROM latches `x_direction` into `$FFFFFABE` once at fire and `sub_E49A` reads it every frame to mirror the whole beam. Native derives direction from `slice.player.facing` into the projectile's velocity sign at fire (`helmets.rs:5537-5541`), with no separate render latch and no mirrored-sprite path. Cosmetic for a point projectile.

**Maniaxe axe — velocity reversal negates `x_vel`; ROM flips a direction flag (architectural)** (`loc_8A74`, `kid.asm:9879-9882`)
The ROM turnaround flips `x_direction`/`$17` and lets the generic mover re-derive motion; it does not negate `x_vel`. Native directly negates the stored velocity (`helmets.rs:5676`), which also inverts the embedded Kid-velocity bias from the throw — may differ subtly from the ROM's direction-flag reconstruction. Lower confidence (mover not traced).

**PARTIAL 2026-06-12: Red Stealth grounded sword hit detection still uses fixed AABB extents; airborne enemy hitbox removed** (`sub_868E loc_86D8/loc_86EC`, `loc_B350`)
The ROM detects grounded sword hits via the engine's per-object `collision_type` from the spawned `sub_868E` hitbox object. Native `process_sword_hits` still approximates that grounded worker with a fixed `6×8` AABB at the faithful `±28/-10` anchor. The previously invented airborne `loc_B350` below-foot enemy hitbox is fixed: native now treats `sub_B270 loc_B350/loc_B380` as animation/sound/floor-impact state only and leaves enemy damage to the spawned side sword worker.

**RESOLVED — Maniaxe axe turnaround does not decrement on the throw frame** (`ManiaxeAxe_Init loc_8A42/8A52/8A74`, `kid.asm:9858-9882`)
The ROM seeds the turnaround counter with `moveq #$C,d0` in `loc_8A42`, enters `loc_8A52`, then `Hibernate_Object_1Frame` advances the axe before `loc_8A74` decrements it. Native now skips the turnaround decrement on the spawn tick, so the first observable tick preserves the seeded 12-frame counter. Covered by `loc_8a42_axe_velocity_uses_base_speed_and_sets_turnaround_counter`.
