<!-- Generated 2026-05-29 by the `enemy-fidelity-audit` workflow (28 read-only Explore auditors, one per enemy family; every high/medium gap adversarially verified by an independent refuter before inclusion). This is a reviewable research artefact, not a gate. Each gap cites the owning ASM label and the native `file:line` searched. Re-run the workflow to refresh. -->
<!-- Staleness-checked 2026-06-10: of 67 gaps, 33 fixed, 9 partially fixed, 25 still open — verdicts and evidence in fidelity-audit-staleness-2026-06-10.md. Still-open entries below remain authoritative; fixed entries are superseded. -->

# Enemy Fidelity Audit

Adversarially-verified per-family audit of the Kid Chameleon source-port. The recurring failure mode across the codebase: ASM-faithful pure helpers exist and are unit-tested, but carry `#[cfg_attr(not(test), allow(dead_code))]` and are never wired into the live step/combat path. The runtime instead uses generic pacers, generic damage paths, or invented physics.

## Coverage summary

| Family | Coverage | Confirmed gaps |
|---|---|---|
| Archer (0x07) | partial | 2 |
| Armadillo (0x04) | partial | 3 |
| Big_Hopping_Skull (0x1D) | partial | 3 |
| Cloud (0x0E) | partial | 1 |
| Crab (0x09) | partial | 2 |
| Diamond (0x01) | partial | 2 |
| Dragon walking (0x0D) | partial | 1 |
| Dragon_Flying (0x0C) | partial | 1 |
| Driller (0x1A) | partial | 3 |
| Drips (0x16) | full | 0 |
| Emo_Rock (0x1B) | partial | 4 |
| Fireball (0x19) | partial | 1 |
| Fire_Demon (0x00) | partial | 1 |
| Goat (0x10) | partial | 3 |
| Hand (0x17) | full | 0 |
| Lion (0x12) | partial | 1 |
| Mini_Hopping_Skull (0x1C) | partial | 2 |
| Ninja (0x11) | partial | 6 |
| Orca (0x08) | partial | 3 |
| Robot (0x03) | partial | 3 |
| Scorpion (0x13) | partial | 4 |
| Sphere (0x06) | full | 0 |
| Spinning_Twins (0x14) | partial | 5 |
| Tank (0x0A,0x0B) | partial | 3 |
| Tank_Shooting (0x0B) | full | 0 |
| Tar_Monster (0x05) | partial | 3 |
| Tornado (0x18) | partial | 3 |
| UFO (0x0F) | partial | 5 |

Totals: 28 families audited, 3 full (no gaps), 25 partial, 0 stub. 75 confirmed gaps.

## Confirmed gaps

### High severity

**Diamond — live HP damage and hurt/death collision words not plumbed** (`loc_3ADFA / loc_3AE28 / loc_3AEE6 / loc_3AEF8`)
ASM decrements HP on `colid_hurt`, hibernates through hurt/death animations, then either restores `stru_3ACA8` or deletes. Native now spawns Diamond as an HP-bearing Pacer, keeps prize-diamond collection separate, routes low-HP Diamonds through the shared walker, routes high-HP Diamonds through `loc_3AD7C`/`loc_3ADA4`, and waits for the ROM death animation on final damage. The live Diamond step still cannot receive the Kid/enemy `colid_hurt` word that selects the Diamond-specific hurt routes.

**Diamond — Berzerker side-launch not plumbed** (`loc_3AE84 / loc_3AF24`)
ASM routes Kid side contacts through Berzerker charge gates that disable level collision, keep Kid collision, freeze animation, and launch the Diamond upward at `-$48000`. Native has unit-tested high/low-HP gate helpers, but the live Diamond terrain loop does not yet receive `colid_kidright`/`colid_kidleft`, so side-launch cannot fire.

**Armadillo — damage recoil/invulnerability window not applied** (`loc_3C848`)
Survivors should set `$42=$FF88` (~120-frame recoil) gating re-hits. Native `take_damage_or_queue_death` has no recoil gate and `is_vulnerable` no Armadillo case, so multi-HP Armadillos can be re-hit every frame. `combat.rs:7810-7872`, `enemies.rs:8519-8540`; faithful `armadillo_damage_response_loc_3c848` (`enemies.rs:3648`) is test-only.

**Armadillo — Berzerker curl-and-launch reaction not simulated** (`loc_3C898 / loc_3C8CC`)
Charging-Berzerker side contact should curl the Armadillo and launch it airborne. `step_pacer` (`enemies.rs:15146`) has no collision/berzerker branch; unlike Crab (`enemies.rs:9211-9213`) there is no Armadillo runtime hook. Helpers `enemies.rs:3755/3790/3832` test-only.

**Armadillo — spawned-child fall-then-die death sequence not simulated** (`loc_3C93C / loc_3C954`)
Spawned children ($5C set) should fall to floor before death animation. Generic `remove_dead_enemy` zeroes velocity and plays death in place for all Armadillos. `combat.rs:7647-7661`, `enemies.rs:6609`; fall helpers `enemies.rs:3692` test-only.

**Big_Hopping_Skull — patrol x-bounds and relaunch reset never run** (`loc_36770 / loc_36782 / loc_36798 / loc_366D0`)
Skull should be pinned to a fixed corridor ($3E/$40) with constant launch height. Live `step_big_hopping_skull` (`enemies.rs:12663-12732`) applies gravity + deflection only; no bound read, no $FFFE relaunch. Corridor helpers `enemies.rs:3342/3209` test-only.

**Fire_Demon — walking fire-trail emission never happens** (`loc_3A542 / loc_3A564 / loc_3A63C`)
Demon should leave up to 7 trail children while pacing. Only runtime trail push is a single death trail (`combat.rs:7676-7678`); walking-trail gate `enemies.rs:5067` is dead. `step_fire_demon_trails` (`projectiles.rs:1293-1307`) only ages trails.

**Goat — Berzerker-charge side contact does not launch airborne** (`loc_3CB76 / loc_3CBA8`)
Fixed candidate 2026-05-29: charging Berzerker side contact with no positive `$42(a3)` now launches Goat into the `loc_3CBA8` airborne loop instead of generic HP damage. Regressions: `berzerker_charge_launches_goat_instead_of_damaging_loc_3cb76`, `goat_berzerker_airborne_loop_matches_loc_3cba8`.

**Crab — directional vulnerability gate (front-immune) not enforced** (`loc_3B048 / loc_3B086 / loc_3B08C`)
Fixed candidate 2026-06-20: direct friendly-projectile hits already used the `loc_3B048` front-side gate; queued parent collision words now do too, so `colid_hurt` damages only when Kid is on the side the Crab faces and otherwise resumes the shared walker. Regressions: `maniaxe_axe_respects_crab_front_vulnerability_gate_loc_3b048`, `crab_pending_hurt_only_damages_from_facing_side_loc_3b048_3b08c`, `crab_pending_hurt_from_back_or_frozen_resumes_without_damage_loc_3b048`.

**Crab — frozen crab not immune to hurt collisions** (`loc_3B048` freeze-sprite check)
Fixed candidate 2026-06-20: direct Maniaxe/projectile hits and queued parent collision words now both route through `crab_collision_dispatch_loc_3b048`, where the `LnkTo_unk_C7C4E` frozen sprite branch resumes without HP damage. Regressions: `maniaxe_axe_respects_crab_frozen_immunity_loc_3b048`, `crab_pending_hurt_from_back_or_frozen_resumes_without_damage_loc_3b048`.

**Hand — grab never immobilizes/affects the Kid** (`loc_32906`)
Fixed candidate 2026-06-04: live Hand `HandState::Grabbing` now surfaces `KidGrabbedByHand` onto the player slice, weakening jump, suppressing crawl entry, and suppressing pull-up/climb escape consumers. Regressions: `refresh_active_roster_surfaces_hand_grab_onto_slice_loc_32906`, `loc_A466_kid_grabbed_by_hand_weakens_live_grounded_jump`, `loc_8c70_crawl_entry_suppressed_while_grabbed_by_hand`, `kid_flip_pullup_suppressed_while_grabbed_by_hand_loc_13411`.

**Hand — airborne collision dispatch table not wired** (`loc_329F8 / off_32A12`)
Fixed candidate 2026-06-20: queued parent collision words now route through `loc_329F8`/`off_32A12` while Hand is `Jumping`. Native clears X/Y velocity, resets to patrol, bounces through `loc_32A5A`/`loc_32A70`, enters `loc_32AA2` on signed-negative collision, and starts the `loc_32906` grab coroutine when the `loc_32A86` gate allows. Regressions: `hand_pending_airborne_collision_clears_velocity_words_loc_329f8`, `hand_pending_airborne_bounce_rolls_back_and_sets_loc_32a5a_velocity`.

**Hand — ground collision dispatch table not wired** (`loc_3299C / off_329B6`)
Fixed candidate 2026-06-20: queued parent collision words now route through `loc_3299C`/`off_329B6` while Hand is `Walking`. Native reverses through `loc_329E2`, writes Kid-below contact through `loc_329EC`, and enters `loc_32AA2` for signed-negative/hurt-class collisions. Regressions: `hand_pending_ground_reverse_collision_runs_loc_329e2`, `hand_pending_ground_destroy_collision_enters_loc_32aa2`, `hand_pending_ground_kid_contact_writes_kid_below_collision_loc_329ec`.

**Lion — hurt-recoil response absent** (`loc_3E318 / loc_3E32C`)
Fixed candidate 2026-06-20: queued parent collision words now route through `loc_3E318`; nonterminal hurt/Kid-above collisions decrement HP, hibernate through native damage flash as `stru_3DF42`, restore the saved charge branch through `loc_3E356`, and signed-negative/terminal hits enter `loc_3E378`/`stru_3DF48`. Regressions: `lion_pending_hurt_collision_hibernates_and_resumes_ground_charge_loc_3e318`, `lion_pending_kidabove_collision_preserves_airborne_resume_loc_3e356`, `lion_pending_terminal_collision_enters_loc_3e378_death_animation`.

**Mini_Hopping_Skull — runtime uses generic step_hopper; hop state machine not wired** (`loc_3EA88 / loc_3EAB2 … loc_3F146`)
Fixed candidate slices 2026-06-20: post-floor stopped wait now follows `loc_3F000`/`loc_3F090`/`loc_3F09E`/`loc_3F112`; a facing mismatch flips and restarts the 21-frame wait, and matched facing waits 11 frames before relaunch. Slope terrain contact now routes through `loc_3EAD8`/`loc_3ED6C`, including the `loc_3F22A` air-damage slope resume. Queued parent collision words now route through the grounded wait/pause handlers and airborne `loc_3F1B8`: side/below contacts allocate `sub_3F3F0`, grounded hurt clears motion through the hit animation path, airborne hurt enters `loc_3F1E6`, and charging-Berzerker side contact enters `loc_3F262`. Remaining runtime hop-state debt: the distinct slope/windup coroutine states are still compressed into native wait fields. Regressions: `mini_hopping_skull_restarts_wait_after_facing_flip_loc_3f090`, `mini_hopping_skull_does_not_edge_reverse_before_launch_loc_3f090`, `mini_hopping_skull_slopeup_nonpositive_x_enters_left_wait_loc_3ead8_3ec32`, `mini_hopping_skull_slopedown_negative_x_enters_left_wait_loc_3ed6c`, `mini_hopping_skull_air_damage_slope_landing_resumes_terrain_contact_loc_3f22a_3ead8`, `mini_hopping_skull_pending_ground_side_collision_queues_scream_worker_loc_3f03c`, `mini_hopping_skull_pending_ground_hurt_collision_uses_ground_hit_path_loc_3f05c`, `mini_hopping_skull_pending_air_hurt_collision_enters_loc_3f1e6_landing_loop`, `mini_hopping_skull_pending_air_side_collision_enters_berzerker_launch_loc_3f262`.

**Mini_Hopping_Skull — air-damage knock-up loop not driven** (`loc_3F1B8 / loc_3F1E6 / loc_3F200 / loc_3F22A / loc_3F238`)
Fixed candidate 2026-06-20: Kid-above stomp damage and queued airborne hurt now enter the `loc_3F1E6` setup, preserve live X in `target_x_velocity`, clear live X, seed `y_vel=$28800`, and loop through `loc_3F200` until floor/slope contact. Floor contact restarts stopped wait or death through `loc_3F158`; slope contact resumes terrain contact through `loc_3F22A` -> `loc_3EAD8`. Regressions: `mini_hopping_skull_stomp_enters_air_damage_loop`, `two_hp_enemy_dies_on_second_stomp`, `mini_hopping_skull_air_damage_slope_landing_resumes_terrain_contact_loc_3f22a_3ead8`, `mini_hopping_skull_pending_air_hurt_collision_enters_loc_3f1e6_landing_loop`.

**Ninja — attack trigger cadence wired** (`loc_3D92A / loc_3D942 / loc_3D958 / loc_3D960`)
Fixed candidate 2026-06-20: live Stealth pacing now initialises `$54(a5)` to `$12C`, rotates `$6E(a5)` between `$32/$12C` on the throw-frame gate, and enters `loc_3D9A4` jump attack only when `$4C(a5)` is zero. The overhead cone already produces the ROM's positive `$4C(a5)` pose instead of launching directly. Regressions: `ninja_cadence_timer_blocks_jump_until_zero_loc_3d942`, `ninja_cadence_wrong_walk_phase_resets_timer_one_loc_3d958`, `ninja_cadence_throw_frame_starts_jump_loc_3d960_3d9a4`.

**Ninja — blocking/guard-pose state machine + block counter not executed** (`loc_3D606 / loc_3D65A / loc_3D670 / loc_3D734 / loc_3D826 / loc_3D84E`)
Fixed candidate slices have wired negative `$4C(a5)` guard, positive `$4C(a5)` jump-threat pose, `$44` hit decrement, `sfx_Ninja_blocking`, `$54/$6E` cadence, and the `$5C(a5)` airborne knockdown branch. `enemies.rs:16613-16695`, `enemies.rs:6257`, `combat.rs:7647-7662`.

2026-06-19 partial: live `step_ninja` now enters and releases `NinjaState::Guard` for the Red Stealth `loc_3D734` -> `loc_3D826` side-threat branch. Later slices covered `$44` block counter, `sfx_Ninja_blocking`, hit/contact dispatch through `loc_3D65A`/`loc_3D670`, Maniaxe projectile guard, and the positive `$4C` jump-threat pose.

2026-06-20 partial: live pending-collision handling now routes hurt/Kid-above IDs through `loc_3D606` -> `loc_3D65A`; ordinary hits decrement `$44(a3)` through `loc_3D670`, while non-zero `$4C(a5)` guard state preserves HP and queues `sfx_Ninja_blocking` ($52). Regressions: `ninja_pending_hurt_collision_decrements_block_counter_loc_3d606_3d670`, `ninja_guard_hurt_collision_queues_blocking_sfx_loc_3d65a`. Later slices covered Maniaxe projectile guard, positive `$4C` jump-threat pose, `$54/$6E` cadence, Kid-above pending collision dispatch, and airborne `loc_3D878`/`loc_3D8BC` Berzerker side-contact knockdown.

2026-06-20 partial: live active-enemy stepping now threads the active Maniaxe axe X delta into Ninja `loc_3D776`; a live `ManiaxeAxe` inside the half-open `[-$5A,+$5A)` band routes to `loc_3D826` guard pose. Regression: `ninja_maniaxe_projectile_threat_enters_guard_pose_loc_3d776_3d826`. Later slices covered positive `$4C` jump-threat pose, `$54/$6E` cadence, Kid-above pending collision dispatch, and airborne `loc_3D878`/`loc_3D8BC` Berzerker side-contact knockdown.

2026-06-20 partial: pending jump-attack collisions now route through `loc_3DA0C`; Kid-above enters `loc_3DA98`, decrements `$44(a3)`, bounces at `$38000`, and final hits enter a new `KidAboveKnockdown` state that runs the `loc_3DABC` floor-wait before `loc_3DB4E` death animation. Regressions: `ninja_jump_attack_kid_above_bounces_and_decrements_loc_3da0c_3da98`, `ninja_jump_attack_kid_above_final_hit_enters_landing_loop_loc_3dabc`, `ninja_kid_above_knockdown_starts_death_after_floor_loc_3dabc_3db4e`. Later slices covered `$54/$6E` cadence and the separate airborne `loc_3D878`/`loc_3D8BC` Berzerker side-contact knockdown branch.

2026-06-20 partial: live overhead proximity now routes through `loc_3D706` -> `loc_3D84E`, enters `NinjaState::JumpThreat` with positive `$4C(a5)`, and uses `loc_3D7FA` to hold/reset through `loc_3D7CA`. Regressions: `ninja_detects_player_in_cone_and_enters_jump_threat_loc_3d706_3d84e`, `ninja_overhead_right_inside_edge_enters_positive_jump_threat_loc_3d706_3d84e`, `ninja_positive_jump_threat_releases_when_kid_leaves_box_loc_3d7fa_3d7ca`. Later slices covered `$54/$6E` cadence and the separate airborne `loc_3D878`/`loc_3D8BC` Berzerker side-contact knockdown branch.

**Ninja — Berzerker side-contact knockback launch not wired** (`loc_3D698 / loc_3DB0A / loc_3D8A8 / loc_3D8BC`)
Fixed candidate 2026-06-20: live pending-collision handling now routes Ninja side contact through `loc_3D606` -> `loc_3D698`; charging Berzerker contact applies the existing `loc_3DB0A` launch state and enters the `loc_3DB3C` airborne loop. Follow-up fixed candidate 2026-06-20: `$5C(a5)` airborne pending collisions now route through `loc_3D878`; Berzerker side contact reaches `loc_3D8BC`, decrements `$44(a3)`, bounces at `$18000`, and terminal hits enter `loc_3D8D6` floor-wait knockdown. Regressions: `ninja_pending_berzerker_side_collision_enters_launch_loc_3d698_3db0a`, `ninja_airborne_berzerker_side_contact_enters_knockdown_loc_3d878_3d8bc`, `ninja_airborne_berzerker_side_contact_final_hit_enters_landing_loop_loc_3d8bc_3d8d6`.

**Ninja — HP-depleted knockdown fall (above-stomp) absent** (`loc_3D8BC / loc_3D8D6 / loc_3DA98`)
No kid-above contact handling and no knockdown state. `enemies.rs:6257-6263`, `enemies.rs:16680-16693`; knockdown helpers test-only.

Fixed candidate 2026-06-20 for jump-attack Kid-above: live pending collision dispatch now covers `loc_3DA0C -> loc_3DA98 -> loc_3DABC`, including the floor-wait before `loc_3DB4E`. Follow-up fixed candidate 2026-06-20 covers `loc_3D878`/`loc_3D8BC` airborne side-contact knockdown under `$5C(a5)`.

**Robot — live collision/hit routing into charge still absent** (`loc_3A96C / loc_3A890 / loc_3A9F0`)
ROM enters charge via routed damaging contact ($38(a2)<0 or $44 hits 0). Native final damage now enters `loc_3A96C` charge instead of generic death, but child-negative collision and shoot-time collision routes are still not fully live.

**Scorpion — sting attack never produced** (`loc_3DDEE / loc_3DE40`)
StingAnimation route only sets tracking_lock + walk_speed_override; the `..` discards tail hurtbox, object_meta $C000, stru_3DB8E, and recovery. Sting deals no damage. `enemies.rs:23838-23847`, `enemies.rs:5496`.

**Scorpion — Berzerker-charge upward launch not wired** (`loc_3DD0E`)
`step_scorpion_tracking` never reads collision_type/berzerker. `enemies.rs:23787`, `enemies.rs:5766/5807` test-only.

**Spinning_Twins — orbit form does not split into two walking children + falling visual** (`loc_3B93C`)
Transition result is discarded (`let _transition`); single enemy just flips to Walking. `step_spinning_twins` has no roster access to spawn siblings. `enemies.rs:22911`, `enemies.rs:22759`; child-init helpers `enemies.rs:22698/22726/22787` test-only.

**Spinning_Twins — walking-child launch/drag/settle physics not applied** (`loc_3BB2C`)
Walking branch applies gravity + floor snap + step_pacer only; no $1200 X-drag, no velocity gate, no settle. `enemies.rs:22914-22933`; drag/gate/settle helpers `enemies.rs:22272/22368/22674/22681` dead.

**Spinning_Twins — multi-hit HP damage/recovery and death state machine not reproduced** (`loc_3BC34`)
No $44 HP field, no Settled phase, no collision dispatch; delegates to step_pacer. `enemies.rs:22914-22933`, `enemies.rs:6329/6408-6409`; helpers `enemies.rs:22399/22535/22629/22609` dead.

**Orca — in-lunge Kid contact (hurt/Kid-above/side) not dispatched** (`loc_3D35A`)
Lunging arm probes terrain only; never reads collision_type for hurt/side. `enemies.rs:18689-18778`; dispatch `enemies.rs:1301-1320` test-only.

**Orca — ceiling-hit HP decrement counter ($44) not modeled** (`loc_3D472`)
No multi-hit-during-lunge death progression; OrcaState has no fatal-fall/dying state. `enemies.rs:18689-18778`; fatal-fall helpers `enemies.rs:1373/1440` test-only.

**Tank — body-vs-mouth/turret vulnerability gating not enforced** (`loc_3B464 / loc_3B6C0`)
Only the child mouth/turret collision word should be damageable; native damages the whole body via generic `is_vulnerable`. Child snapshot is render-only and skipped by the stomp loop. `combat.rs:7813`, `enemies.rs:8529`; gating `enemies.rs:24087/24226` dead.

**Tank — negative parent-collision instant-death route not wired** (`loc_3B41C→loc_3B480 / loc_3B6C0→loc_3B6E6`)
Negative collision should bypass HP and run the child-hide/Kid-collision-restore death. Runtime death is HP-only. `enemies.rs:24086`, `enemies.rs:8519-8526`, `combat.rs:7813`.

**Tar_Monster — 5-projectile burst fires on every non-fatal hit instead of on death** (`loc_323E6 / loc_323C0`)
`spawn_tar_monster_retaliation` is called only in the non-fatal branch; death path spawns no burst — inverse of ROM. `combat.rs:7865-7869`, `combat.rs:7813-7814`, `combat.rs:7647-7662`.

**Tornado — Cyclone-flying-above kill branch not reproduced** (`loc_35986`)
Missing the `y_pos` compare → death/respawn; live branch always bounces. Cyclone flight can never destroy a Tornado. `combat.rs:10072-10083`, `enemies.rs:10733-10734`.

**UFO — live motion uses fabricated 60-frame accel sign-flip instead of once-per-second random picker** (`Enemy0F_UFO_1SecLoop / loc_35326 / unk_35344 / loc_35444`)
No random byte / x_pos XOR / 16-entry table / acceleration ramp; just periodic sign-flip + probe_wall. `enemies.rs:10165-10204`, `enemies.rs:9322-9324`.

**UFO — below-and-near-Kid chase steering not invoked** (`loc_34E2A / loc_34E86 / loc_34E96`)
`step_ufo(enemy, block_grid)` has no player position; tracking command is test-only. UFO never steers toward the player. `enemies.rs:9323`, `enemies.rs:10165-10204`, `enemies.rs:10083`.

**UFO — two-phase hurt ($47) absent: first hit should spray debris and survive** (`Enemy0F_UFO_Hurt / loc_35150 / loc_351EE / loc_354DA`)
`remove_dead_enemy` treats UFO as single death; no $47 latch. First-hit debris helpers test-only. `combat.rs:7630-7645`, `enemies.rs:10978/10913`.

### Medium severity

**Archer — fire cadence uses arrow re-aim interval instead of pose-table delays** (`loc_341BE / loc_34248`)
ROM poses ~118-138 frames between shots (walking the $64/6/$C/$14 delay words); native uses arrow_accuracy (30/20/10) as the whole cadence and reuses it as reaim_interval — fires ~4-5x too fast. `enemies.rs:23707-23714/23766-23767`; pose helpers `enemies.rs:6004/6053` dead.

**Archer — shot-count gate counts the shared projectile pool, not arrows** (`loc_34248`)
ROM caps at 2 arrows via Number_of_Arrows; native gates on `projectiles.len() >= MAX_PROJECTILES (4)` over the shared pool. `enemies.rs:23722/23753`, `constants.rs:576-578`.

**Big_Hopping_Skull — per-hit SFX cadence + kid-upward-bounce contact bug not driven** (`loc_365D8 / loc_365F2 / loc_36616 / loc_3664E`)
ROM writes $2A(a2)=-3 to the Kid on every hurt/kidabove contact and plays groan/voice/dies SFX. Damage flows through generic process_stomps; the groan SFX at `combat.rs:6715` belongs to a boss routine. Helpers `enemies.rs:2947/2981/3004/3067` dead.

**Big_Hopping_Skull — death respawn marker (EnemyStatus_Table[d0]=$2168) not written** (`loc_3664E / loc_36664 / loc_366A4 / loc_366B2`)
Skull death uses generic killed_enemies set; only Dragon writes enemy_status_words on death. `combat.rs:7673-7674`, `combat.rs:7751-7754`, `enemies.rs:3150`.

**Cloud — lightning projectile has fixed 31-frame lifetime instead of collision-bound persistence** (`loc_35DF0 / loc_35E8A`)
ROM deletes only on collision_type≠0 or flag $19; native caps at 31 frames via the generic step. `enemies.rs:9869`, `projectiles.rs:1432-1439`; model fn `enemies.rs:9699` dead.

**Dragon (walking) — breath flame deleted on first Kid contact instead of persisting its animation** (`loc_3D02A / loc_3D082`)
Flame is a stationary multi-frame hazard in ROM; native pushes a generic projectile removed on first overlap (only FireDemonDeathTrail is exempt). `projectiles.rs:1934-1944`, `enemies.rs:20598-20610`.

**Dragon_Flying — descent fireball uses a generic projectile instead of windup/travel/impact lifecycle** (`loc_34BAC / loc_34C26 / loc_34C3A / loc_34C4A`)
No windup/travel sprite phase, no $19 off-screen kill (owner=None so never culled, persists to fixed 600-frame timer), no two-stage impact anim. `enemies.rs:13385-13400`, `projectiles.rs:1770`; lifecycle helpers `enemies.rs:8030/8078/8121` test-only.

**Driller — probe offset, wall-collision gate, and Ice bullet dispatch closed** (`loc_3BE78` / `loc_3BE8E` / `loc_3BEF2`)
The earlier audit overstated the remaining gap. `$38(a4)` is `collision_type`, not an animation timer: ROM gates on `colid_leftwall`/`colid_rightwall`, then clears the child collision word before dispatching Smash/Prize or Ice block destruction. Native now probes the linked drill child at parent±$18, requires a Solid front-wall cell before drilling, preserves the Smash/Ice action discriminant, and spawns the three Ice shooter bullets. Regressions: `loc_3be78_driller_probes_right_child_x_not_parent_collision_width`, `loc_3bef2_driller_preserves_ice_block_dispatch_for_runtime_bullets`, `loc_3bef2_driller_ice_block_spawns_three_shooter_bullets_when_target_open`, `loc_3be8e_driller_requires_front_wall_collision_before_drilling`.

**Fireball — fire-trail child objects never spawned** (`Enemy19_Fireball_BounceFloorCeiling / loc_32CE4 / loc_330D0 / loc_33128`)
ROM allocates trail children (≤5) on every bounce/slope; `step_bouncer_with_rng` only mutates velocity. `enemies.rs:12824-12845`; trail helpers `enemies.rs:12116/12564` dead.

**Goat — Berzerker side contact during recoil ($42>0) does not shove+hurt-resume** (`loc_3CBBA`)
Fixed candidate 2026-06-20: charging Berzerker side contact during positive `$42(a3)` now shoves Goat by ±$14 pixels through `loc_3CBBA`, restores the hurt-resume recoil state/saved walk speed, and skips generic HP damage. Regression: `berzerker_charge_recoil_goat_shoves_without_damage_loc_3cbba`.

**Goat — collision-type dispatch (death-on-negative, side-contact gate) not wired** (`loc_3CB14`)
Fixed candidate 2026-06-20: queued parent collision words now route through `goat_collision_dispatch_loc_3cb14`; signed-negative collisions enter the `loc_3CB64` death animation before any HP decrement, `colid_hurt`/`colid_kidabove` use the `loc_3CB38` recoil hibernate, and side contacts share the `loc_3CB76` Berzerker gate used by live player contact. Regressions: `goat_negative_parent_collision_enters_loc_3cb64_death_before_hp_damage`, `goat_pending_hurt_collision_enters_loc_3cb38_recoil_hibernate`.

**Hand — grab entry gate conditions not checked** (`loc_32A86`)
Fixed candidate 2026-06-20: the combat Hand grab path now routes through `hand_kid_contact_gate_loc_32a86`, rejecting crawl, Juggernaut, and already-grabbed states before starting `HandState::Grabbing`. Existing coverage: `hand_kid_contact_gate_loc_32a86_requires_grabbable_kid_state`.

**Hand — post-grab release re-jump skips the loc_3639E free-space check** (`loc_32970`)
Fixed candidate 2026-06-20: `step_hand` now runs the shared `loc_3639E` clearance probe on grab release using Hand's `d6=9` height and `d7=$E` half-width, branches to `loc_32AA2` death animation when blocked, and only restarts `loc_328AA` jump setup when clear. Regressions: `hand_release_probe_uses_loc_32970_dimensions`, `hand_release_rejump_requires_free_space_loc_32970`, `hand_release_restarts_jump_when_space_is_free_loc_32970`.

**Lion — charge is purely horizontal; ledge-launch leap with gravity not reproduced** (`loc_3E682 / loc_3E6AE / loc_3E6F6 / loc_3E732`)
Fixed candidate 2026-06-20: live charge now uses the Lion/Dragon airborne charge tail, including ledge launch, lead-column snap, full-width ceiling/floor scans, gravity, landing resume, and terrain death. Regressions include `lion_airborne_charge_lands_on_full_width_floor_scan_loc_3e732`, `lion_airborne_charge_hits_full_width_ceiling_scan_loc_3e6f6`, `lion_airborne_charge_consumes_right_lead_column_boundary_loc_3e6ae`, and `lion_left_airborne_charge_body_terrain_enters_death_animation_loc_3e94e`.

**Lion — $42 charge-windup/window state machine is compressed** (`loc_3E434 / loc_3E450 / loc_3E604`)
Fixed candidate 2026-06-20 for the close-band tracker: live walking now applies `loc_3E03C`/`loc_3E054`, holds `$4E(a5)` inside the wider band, clears it through `loc_3E066`, and sets it on charge exits through `loc_3E48E`/`loc_3E642`. Regressions: `lion_walking_tracks_far_kid_before_timer_tick_loc_3e03c_3e074`, `lion_walking_tracking_latch_holds_until_outer_band_exit_loc_3e054_3e066`. Remaining debt: native still compresses `$42(a3)` charge-windup and the ±$60/±$70 engagement-window count-up into one overshoot check.

**Mini_Hopping_Skull — post-landing facing-flip re-orientation pause not reproduced** (`loc_3F000 / loc_3F074 / loc_3F090 / loc_3F09E`)
Fixed candidate 2026-06-20: live `step_hopper` now preserves the stopped wait until `loc_3F074`, flips toward Kid through `loc_3F090` only after expiry, restarts `loc_3F000` on mismatch, and uses an 11-frame `loc_3F09E` matched-facing pause before `loc_3F112` relaunch. Regressions: `mini_hopping_skull_restarts_wait_after_facing_flip_loc_3f090`, `mini_hopping_skull_does_not_edge_reverse_before_launch_loc_3f090`.

**Ninja — jump-attack airborne loop omits ceiling/kid-above branches** (`loc_3DA0C / loc_3DAB0 / loc_3DA98`)
Native handles only wall/floor contact during jump attack; ceiling reflection and Kid-above contact remain helper-only. `enemies.rs:17664-17710`; dispatcher `enemies.rs:16846/16960/17132` unused.

2026-06-19 partial: live jump attack now probes ceiling contact and applies `loc_3DAB0` y-velocity negation. Still open: Kid-above `loc_3DA98`, side-contact `loc_3DB0A`, and the broader `loc_3DA0C` collision-word dispatch.

**Ninja — far-Kid facing turn during pacing not applied** (`loc_3D92A / loc_3D978 / loc_3D98A`)
Fixed candidate 2026-06-20: live `step_ninja` now applies the `loc_3D92A` far-Kid facing decision before pacing, turning right through `loc_3D978` and left through `loc_3D98A` when Kid is outside the inclusive +/-$80 px band. Regressions: `ninja_far_kid_turns_right_during_pacing_loc_3d92a_3d978`, `ninja_far_kid_turns_left_during_pacing_loc_3d92a_3d98a`.

**Orca — wall-pinned lunge X-drift handled, but side-collision Berzerker re-lunge absent** (`loc_3D4DE`)
Live path probes terrain only; never reads Kid side contact or Berzerker_charging during the lunge. (Wall-pin X-drift suppression is correctly implemented.) `enemies.rs:18657-18780/18694-18701/18739-18743`; gate `enemies.rs:1301/1096` test-only.

**Robot — projectile timing/rendering is typed, but shoot-time collision routes remain collapsed** (`Enemy03_Robot_Shot_Init / loc_3ABB6 / loc_3AC7E / loc_3AC92`)
Live shots now use the `loc_3AC0C` muzzle offset and `loc_3AC48` camera/collision gate, then enter `RobotShotImpact` with the `loc_3AC7E/loc_3AC92` velocity clear and X offset. Flying shots keep a separate animation clock for looping `stru_3A754` (`unk_C8278`, `unk_C8280`, `unk_C8288`) without turning the shot into a DBF-lifetime object. The impact object survives the exact `stru_3A762` 3+4+8 frame duration and renders the three ROM impact frames (`unk_C8260`, `unk_C8268`, `unk_C8270`).

**Robot — body/head per-part collision and sprite swaps still absent** (`Enemy03_Robot_Init / loc_3A8AE`)
Live roster now exposes the linked head object from `loc_3A7EE/loc_3A816`, friendly projectile damage scans both the parent body and linked head before entering `loc_3A8FC`, charging Berzerker side contact now targets the linked head `$38(a2)` only before routing `$20/$24` through `loc_3A9CC -> loc_3A9D2`, and stomp candidates cover both parent and linked-head hit reactions without double-applying damage on an overlapping frame. Shoot/hit head-leg sprite swaps are still approximate.

**Scorpion — collision/HP state machine not wired; damage/death go through generic path** (`loc_3DCB0 / loc_3DCD4 / loc_3DCF4`)
Multi-hit HP and death-anim delay preserved, but per-collision-type routing and hurt-anim-then-restore (stru_3DB7A→stru_3DB60) and tail $3D flag on death are not driven. `enemies.rs:5673-5743`, `combat.rs:7647`, `enemies.rs:6627`.

**Scorpion — tail/stinger sub-object absent as a live entity** (`Enemy13_Scorpion_Init / loc_3DC86`)
No paired tail gfx object ($4013/C7E1E) is ever allocated; the sting/death hurtbox flags have nothing to attach to. `enemies.rs:5642`, `enemies.rs:9261`, `enemies.rs:6417`.

**Spinning_Twins — Berzerker side-collision upward launch not reproduced** (`loc_3BBF6`)
Walking branch never reads Berzerker_charging. `enemies.rs:22914-22933`; helpers `enemies.rs:22466/22503` dead.

**Spinning_Twins — orbit oscillation timing + transition trigger diverge** (`loc_3B904`)
Native advances the orbit index every frame (no animation_timer==1 gate), applies offset to its own x_pos (not a child slot), and transitions on geometric overlap instead of collision_type (neg/$2C/$1C). `enemies.rs:22889-22894/22899-22908`.

**Tank — non-fatal hit recoil + motion stop + hibernate not applied** (`loc_3B49A / loc_3B700`)
ROM clears $50, sets recoil timer 6, hibernates 5 frames; generic damage only sets damage_flash_frames, so a hit tank keeps driving. `combat.rs:7813`, `enemies.rs:8519-8526`, helpers `enemies.rs:24178-24189/24332-24345` dead.

**Tar_Monster — burst-child projectiles lack the off_32656 velocity-reflection bounce** (`loc_325E6 / loc_32682 / off_32656`)
Children should reflect/halve x or y velocity for a 2-hit budget; native spawns wall_bounce:false plain projectiles that just fall and expire. `enemies.rs:20048-20069`, `enemies.rs:19685/19755`, `projectiles.rs:1816`.

**Tar_Monster — straight attack glob does not burst on impact** (`loc_324D0 / loc_324E6 / loc_324F8`)
Glob should spawn 5 children on terrain contact; native pushes it with no terrain_kill, so it expires silently. `enemies.rs:20003-20015`, `projectiles.rs:1786-1811/1432-1436`, `combat.rs:7868`.

**Tornado — terrain collision uses simplified reversal; ceiling bounce missing** (`loc_35948`)
Live step probes wall + floor (y_vel>0) only; no ceiling probe (loc_35146 path) so an upward-moving Tornado never reverses. `enemies.rs:11898-11902`, dispatch `enemies.rs:10645-10646` dead.

**Tornado — slope bounce (BounceUpSlope/BounceDownSlope) absent** (`loc_35948`)
No slope detection or diagonal ±$10000 velocity assignment; only plain axis flips. `enemies.rs:11880-11903`, enum/mapping `enemies.rs:10646-10669` dead.

**UFO — collision-type bounce table dispatch not used** (`Enemy0F_UFO_ExecCollisionBehavior / loc_35056`)
Live step does ad-hoc wall-only reversal; missing floor/ceiling bounce, slope reseed, and loc_350F2 rollback (3 of 4 bounce categories). `enemies.rs:10186-10202`, helpers `enemies.rs:10575/10576/10835/10853` dead.

**UFO — random beam trigger uses a non-ROM position hash** (`Enemy0F_UFO_ChkLoadBeam`)
Native uses `(x_pos+wait_timer)*2654435761>>24 < 4` (4/256) instead of `Get_RandomNumber_byte <= 4` (5/256), and never consumes the ROM RNG stream. `enemies.rs:10348-10356`.

## Reading

`full` families (Drips 0x16, Sphere 0x06) had zero confirmed gaps after adversarial verification. Every other family is `partial` or `stub`: the dominant pattern is ASM-faithful pure helpers that are correctly modelled and unit-tested but `#[cfg_attr(not(test), allow(dead_code))]` and never wired into the runtime step/combat path — so the live game uses generic pacers/hoppers/flyers, a generic HP damage path, and a Crab-only Berzerker special-case. Wiring those existing helpers into the live dispatch is the highest-leverage work.
