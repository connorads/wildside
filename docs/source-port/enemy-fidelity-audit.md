<!-- Generated 2026-05-29 by the `enemy-fidelity-audit` workflow (28 read-only Explore auditors, one per enemy family; every high/medium gap adversarially verified by an independent refuter before inclusion). This is a reviewable research artefact, not a gate. Each gap cites the owning ASM label and the native `file:line` searched. Re-run the workflow to refresh. -->

# Enemy Fidelity Audit

Adversarially-verified per-family audit of the Kid Chameleon source-port. The recurring failure mode across the codebase: ASM-faithful pure helpers exist and are unit-tested, but carry `#[cfg_attr(not(test), allow(dead_code))]` and are never wired into the live step/combat path. The runtime instead uses generic pacers, generic damage paths, or invented physics.

## Coverage summary

| Family | Coverage | Confirmed gaps |
|---|---|---|
| Archer (0x07) | partial | 2 |
| Armadillo (0x04) | partial | 4 |
| Big_Hopping_Skull (0x1D) | partial | 4 |
| Cloud (0x0E) | partial | 1 |
| Crab (0x09) | partial | 2 |
| Diamond (0x01) | stub | 5 |
| Dragon walking (0x0D) | partial | 1 |
| Dragon_Flying (0x0C) | partial | 1 |
| Driller (0x1A) | partial | 3 |
| Drips (0x16) | full | 0 |
| Emo_Rock (0x1B) | partial | 5 |
| Fireball (0x19) | partial | 1 |
| Fire_Demon (0x00) | partial | 3 |
| Goat (0x10) | partial | 4 |
| Hand (0x17) | partial | 6 |
| Lion (0x12) | partial | 4 |
| Mini_Hopping_Skull (0x1C) | partial | 5 |
| Ninja (0x11) | partial | 6 |
| Orca (0x08) | partial | 4 |
| Robot (0x03) | partial | 4 |
| Scorpion (0x13) | partial | 4 |
| Sphere (0x06) | full | 0 |
| Spinning_Twins (0x14) | partial | 5 |
| Tank (0x0A,0x0B) | partial | 3 |
| Tank_Shooting (0x0B) | full | 0 |
| Tar_Monster (0x05) | partial | 3 |
| Tornado (0x18) | partial | 3 |
| UFO (0x0F) | partial | 5 |

Totals: 28 families audited, 3 full (no gaps), 24 partial, 1 stub. 88 confirmed gaps.

## Confirmed gaps

### High severity

**Diamond — treated as a passive collectible, not a hostile enemy** (`Enemy01_Diamond_Init / loc_3AEC4 / loc_3AD7C`)
ASM Diamond is a hostile HP-bearing enemy (HP=enemy_level+2, min 2) running a per-frame move/collision state machine (pace ≤3 HP, targeting physics ball >3 HP). Native: `EnemyFamily::Diamond` is `Collectible`, removed on touch with diamond-count increment, and excluded from contact damage. `enemies.rs:8706`, `combat.rs:7928-7978`, `combat.rs:8183-8185`.

**Diamond — per-frame state machine never executed** (`loc_3ADA4 / loc_3AEC4 / loc_3AF14`)
`Collectible|Inert => {}` no-op in the step dispatch means no `step_diamond` ever runs. `enemies.rs:9150-9151`, `enemies.rs:8706`; the only live Diamond branch (`enemies.rs:13490`) is retention/culling, not movement.

**Diamond — high-HP movement/physics modeled but not wired** (`loc_3AD7C / loc_3AE3E / loc_3AE4A / loc_3AE68 / loc_3AE78`)
HP>3 target/fall/wall-bounce/slope physics is fully implemented as pure helpers but dead-code-gated; callers are only each other and tests. `enemies.rs:4413-4628`.

**Diamond — HP damage and hurt/death animations not applied** (`loc_3ADFA / loc_3AE28 / loc_3AEE6 / loc_3AEF8`)
ASM decrements HP on colid_hurt with hurt/death animations; native never decrements Diamond HP (instant pickup) and excludes it from sword/stealth/contact damage. `combat.rs:7969-7972`, `helmets.rs:5094-5095/5234/5922`, `combat.rs:8183-8185`.

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

**Fire_Demon — charging Berzerker side contact does not launch demon upward** (`loc_3A5E4 / loc_3A610`)
`process_berzerker_charge_hits` special-cases only Crab; FireDemon falls through to generic kill. `helmets.rs:5250/5266`; launch fns `enemies.rs:5254/5291` dead.

**Goat — Berzerker-charge side contact does not launch airborne** (`loc_3CB76 / loc_3CBA8`)
Same Crab-only special-case; Goat hits generic kill. `helmets.rs:5250-5266`; `enemies.rs:4162/4205` dead.

**Crab — directional vulnerability gate (front-immune) not enforced** (`loc_3B048 / loc_3B086 / loc_3B08C`)
Crab should take damage only from the side it does not face. Live damage gates on `is_vulnerable` (returns true for Crab) then unconditional `take_damage_or_queue_death`. `combat.rs:7789-7813`, `enemies.rs:8519-8540`; facing gate `enemies.rs:574-610` test-only.

**Crab — frozen crab not immune to hurt collisions** (`loc_3B048` freeze-sprite check)
A frozen crab should be invulnerable. `is_vulnerable` never reads `crab_frozen`; immunity lives only in the unused dispatch. `enemies.rs:8529-8540`, `enemies.rs:573-592`.

**Hand — grab never immobilizes/affects the Kid** (`loc_32906`)
`KidGrabbedByHand` is read by jump code but assigned nowhere; the Grabbing branch only snaps the Hand to the Kid. Grab is cosmetic. `enemies.rs:17611-17643`, `physics.rs:719`, `movement.rs` jump call sites.

**Hand — airborne collision dispatch table not wired** (`loc_329F8 / off_32A12`)
Directional bounce-off-terrain and negative-collision destroy-in-air are absent; Jumping arm does native probing only. `enemies.rs:17552-17609`; dispatcher `enemies.rs:16853` + bounce helpers test-only.

**Hand — ground collision dispatch table not wired** (`loc_3299C / off_329B6`)
Destroy-on-collision and the contact-effect are absent; Walking uses probe edge detection. ActiveEnemy has no `collision_type` field. `enemies.rs:17467-17509`, `enemies.rs:16798`, `enemies.rs:6350`.

**Lion — hurt-recoil response absent** (`loc_3E318 / loc_3E32C`)
Non-fatal hits should play the hit anim + stagger via $44 recoil counter. `step_lion` has no collision branch; LionState lacks a hurt variant. `enemies.rs:17699`, `enemies.rs:6290-6296`; `lion_collision_response_loc_3e318` (`enemies.rs:1597`) test-only.

**Mini_Hopping_Skull — runtime uses generic step_hopper; hop state machine not wired** (`loc_3EA88 / loc_3EAB2 … loc_3F146`)
ROM windup/airborne/launch co-routine collapsed into a single 21-frame counter + ballistic launch. `enemies.rs:9182`, `enemies.rs:14920-15018`; state helpers `enemies.rs:13734-14918` test-only.

**Mini_Hopping_Skull — air-damage knock-up loop not driven** (`loc_3F1B8 / loc_3F1E6 / loc_3F200 / loc_3F22A / loc_3F238`)
Non-fatal hurt should knock the skull up (y_vel=$28800 + $4000/frame). `step_hopper` has no hurt branch; combat only queues scream + generic knockback. `enemies.rs:14626-14708`, `enemies.rs:14920-15018`, `combat.rs:10097-10103`.

**Mini_Hopping_Skull — Berzerker-charge launch-up not driven** (`loc_3F262 / loc_3F288 / loc_3F244`)
Crab-only special-case again; skull hits generic kill. `helmets.rs:5266`, `enemies.rs:14920-15018`; launch helpers test-only.

**Ninja — attack trigger ignores $54/$6E cadence; divergent detection cone** (`loc_3D92A / loc_3D942 / loc_3D958 / loc_3D960`)
ROM fires on a periodic timer gated by the throw-frame; native fires off an overhead cone (which in ROM produces the guard pose). ActiveEnemy has no $54/$6E timer. `enemies.rs:16627`, cadence helpers `enemies.rs:16306-16415` test-only.

**Ninja — blocking/guard-pose state machine + block counter not executed** (`loc_3D606 / loc_3D65A / loc_3D670 / loc_3D734 / loc_3D826 / loc_3D84E`)
NinjaState has only Stealth/JumpAttack; no Block/Guard, no $44, no sfx_Ninja_blocking. Hits go through generic HP death. `enemies.rs:16613-16695`, `enemies.rs:6257`, `combat.rs:7647-7662`.

**Ninja — Berzerker side-contact knockback launch not wired** (`loc_3D698 / loc_3DB0A / loc_3D8A8 / loc_3D8BC`)
`step_ninja` never reads Berzerker_charging; helpers dead. `enemies.rs:16606`, `enemies.rs:16054/15842/16167`.

**Ninja — HP-depleted knockdown fall (above-stomp) absent** (`loc_3D8BC / loc_3D8D6 / loc_3DA98`)
No kid-above contact handling and no knockdown state. `enemies.rs:6257-6263`, `enemies.rs:16680-16693`; knockdown helpers test-only.

**Robot — charge entered unconditionally after every shot instead of as a hit-reaction** (`loc_3A96C / loc_3A890 / loc_3A9F0`)
ROM enters charge only via routed damaging contact ($38(a2)<0 or $44 hits 0); native sets `Charging` every shoot cycle. `enemies.rs:18583-18585`; routers `enemies.rs:17855/17973/18056` test-only.

**Robot — shooting not gated by sub_3C4F8 aiming logic** (`sub_3C4F8 / Enemy03_Robot_Shoot`)
Native fires whenever the timer expires, no random gate or turn/aim sequence. The gating helper is only invoked with the Tank kind at runtime. `enemies.rs:18548-18586`, helper `enemies.rs:38400` (Robot kind only from tests).

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

**Armadillo — walk-speed not capped at $28000 for HP≥3** (`loc_3C756 / loc_3C766`)
Generic linear formula gives HP4=$38000, HP5=$48000 vs ROM's $28000 cap; HP is unclamped (=level+1). `enemies.rs:3498/3535-3536`, used live at `enemies.rs:15154`.

**Big_Hopping_Skull — floor contact uses naive y_vel negation instead of loc_36580 stop+relaunch** (`loc_36580 / loc_366D0`)
ROM zeroes both velocities, plays landing anim, relaunches at constant y_vel=#-2; native negates accumulated velocity so hop height drifts. `enemies.rs:12700-12705` (comment admits divergence).

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

**Driller — drill probe uses parent half-width (12px) instead of child drill offset (24px)** (`loc_3BE78`)
ROM reads the child drill object x_pos at parent±$18; native probes at ±12 (the parent collision box). Misses blocks one cell out. `enemies.rs:15325/15456-15460`.

**Driller — drill fires every frame instead of only on the drill-bit animation frame** (`loc_3BE8E`)
ROM gates on $38(a4)==d1 (8/4) then clears it (once per swing); native probes from facing every step. `enemies.rs:15450-15476/15436`.

**Driller — drilling an Ice block does not spawn ice-shard bullets** (`loc_3BEF2`)
`driller_drilled_cells` discards the Ice/Smash discriminant and just clears the cell; no ice-bullet emitter exists. `enemies.rs:15468-15475`, `enemies.rs:38779-38781`.

**Fireball — fire-trail child objects never spawned** (`Enemy19_Fireball_BounceFloorCeiling / loc_32CE4 / loc_330D0 / loc_33128`)
ROM allocates trail children (≤5) on every bounce/slope; `step_bouncer_with_rng` only mutates velocity. `enemies.rs:12824-12845`; trail helpers `enemies.rs:12116/12564` dead.

**Fire_Demon — hurt collision dispatch + recovery anim use generic path** (`loc_3A586 / loc_3A5BE`)
Death works via generic frames, but per-hit stru_3A34E recovery and explicit collision-type routing are not driven; survivors show only the generic flash. `enemies.rs:8519-8526/8725/6619`, dispatch `enemies.rs:5306/5190` dead.

**Goat — Berzerker side contact during recoil ($42>0) does not shove+hurt-resume** (`loc_3CBBA`)
No $42/recoil branch or ±$14 positional shove in `process_berzerker_charge_hits`. `helmets.rs:5220-5272`, `enemies.rs:4236` dead.

**Goat — collision-type dispatch (death-on-negative, side-contact gate) not wired** (`loc_3CB14`)
Side-contact→berzerker and HP-decrement routing absent; generic combat branches only on 0x1C/0x2C/-1, never GOAT 0x20/0x24. `enemies.rs:4069-4085`, `enemies.rs:9227-9254`, `combat.rs:6643/6674/7336`.

**Goat — non-fatal hurt recoil ($42=$FF88, speed/anim restore) not driven** (`loc_3CB38`)
Generic damage path does only an 8-frame flash; no recoil hold/walk-speed/patrol-anim restore. `combat.rs:7813`, `enemies.rs:8519-8526`, model `enemies.rs:4101` dead.

**Hand — grab duration is a flat 10 frames instead of 10 Kid-$16-change ticks (+ missing Juggernaut early-exit)** (`loc_3292E`)
ROM decrements only on Kid facing-byte change and releases early on Juggernaut; `step_hand` lacks the Kid-direction/Current_Helmet inputs. `enemies.rs:17622-17623/17225/17458-17463`.

**Hand — grab entry gate conditions not checked** (`loc_32A86`)
Grab fires on distance/height only; no crawl/Juggernaut/already-grabbed check. `enemies.rs:17600-17603`, gate `enemies.rs:16899` dead.

**Hand — post-grab release re-jump skips the loc_3639E free-space check** (`loc_32970`)
Unconditional re-jump; no clearance probe / destroy-if-blocked. `enemies.rs:17625-17643`; `emo_rock_clearance_probe_loc_3639e` only wired for Emo_Rock (`enemies.rs:23495`).

**Lion — roar windup + deferred projectile spawn not reproduced** (`loc_3E078 / loc_3E09E`)
Native fires same-frame on timer expiry; no roar animation gate. LionState lacks a roar state. `enemies.rs:17738-17766/6290-6296`; gate `enemies.rs:1498` test-only.

**Lion — charge is purely horizontal; ledge-launch leap with gravity not reproduced** (`loc_3E682 / loc_3E6AE / loc_3E6F6 / loc_3E732`)
Charging arm advances x only; no y_vel/gravity/$180000 cap or floor-landing snap. `enemies.rs:17771-17823`; charge-physics helpers `enemies.rs:1909/1960/2040/2071` test-only.

**Lion — close-band tracker + $42 charge-windup state machine not used** (`loc_3E054 / loc_3E434 / loc_3E450`)
Flat wait_timer + single >112px overshoot instead of ±$60/±$70 engagement windows and $42 count-up. `enemies.rs:17699-17822/6290-6296`; helpers `enemies.rs:205/1758` test-only.

**Mini_Hopping_Skull — native adds a platform-edge turnaround the ROM lacks** (`loc_3F1AA vs loc_3EAB2`)
ROM reverses only on wall or repicks from Kid-X; native adds `!probe_floor(look_ahead).on_ground` edge reversal. `enemies.rs:15038-15045`, reached via `enemies.rs:14935`.

**Mini_Hopping_Skull — post-landing facing-flip re-orientation pause not reproduced** (`loc_3F000 / loc_3F074 / loc_3F090 / loc_3F09E`)
ROM flips x_direction and restarts the grounded wait until facing matches Kid; native sets facing once + 21-frame countdown. `enemies.rs:14932-14956/9182`; helpers `enemies.rs:14464/14547/14848` test-only.

**Ninja — jump-attack airborne loop omits ceiling/kid-above branches + uses generic gravity cap** (`loc_3D9DA / loc_3DA0C / loc_3DAB0 / loc_3DA98`)
ROM gravity is uncapped; native clamps with `MAX_FALL_SPEED` and handles only wall/floor (no ceiling reflection, no kid-above). `enemies.rs:16645/16643-16693`; dispatcher `enemies.rs:15842/15955/16109` unused.

**Ninja — far-Kid facing turn during pacing not applied** (`loc_3D92A / loc_3D978 / loc_3D98A`)
ROM turns to face Kid when >$80 away; native sets facing only inside the attack trigger. `enemies.rs:16614-16641`, `enemies.rs:16306` test-only.

**Orca — lunge wind-up phase (6-frame loop with collision-abort) missing** (`loc_3D320`)
Native transitions Walking→Lunging and assigns launch velocities same-frame; no hibernate/collision-abort. windup_frames is computed then discarded. `enemies.rs:18676-18687/6268-6274/1245`.

**Orca — wall-pinned lunge X-drift handled, but side-collision Berzerker re-lunge absent** (`loc_3D4DE`)
Live path probes terrain only; never reads Kid side contact or Berzerker_charging during the lunge. (Wall-pin X-drift suppression is correctly implemented.) `enemies.rs:18657-18780/18694-18701/18739-18743`; gate `enemies.rs:1301/1096` test-only.

**Robot — projectile lifecycle: no muzzle offset, camera-window culling, or impact animation** (`Enemy03_Robot_Shot_Init / loc_3ABB6 / loc_3AC0C / loc_3AC48 / loc_3AC7E`)
Spawned at enemy x/y with fixed lifetime 60 as a generic projectile; no camera cull, no stru_3A762 impact. `enemies.rs:18568-18580`, `projectiles.rs:1432-1440`; lifecycle helpers `enemies.rs:18379/18417/18448/18467` test-only.

**Robot — two-object body+head structure collapsed to one** (`Enemy03_Robot_Init / loc_3A8AE`)
Single ActiveEnemy, no child-head object; per-part collision and head/leg sprite swaps absent. `enemies.rs:18494-18636`; dispatch `enemies.rs:17875-17876` and attachment helper `enemies.rs:38078` dead.

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
