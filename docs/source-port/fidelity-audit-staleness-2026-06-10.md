<!-- Generated 2026-06-10 by the fidelity-audit-staleness-check workflow (one read-only Explore extractor per audit doc parsed every confirmed gap; one read-only Explore re-checker per gap compared the gap claim against current code and git log since the doc's generation date; every fixed / partially-fixed / no-longer-applicable verdict was independently re-verified by a confirmer defaulting to still-open). This is a reviewable research artefact, not a gate. Re-run the workflow to refresh. -->

# Fidelity-Audit Staleness Report (2026-06-10)

Re-check of all 258 confirmed gaps recorded by the six late-May fidelity audits against the
code as of 2026-06-10. A gap counts as **fixed** only when the faithful behaviour is wired
into the live frame-step path (test-only wiring does not count). Still-open entries remain
authoritative in their original audit doc; this report carries the verdict and evidence for
everything that changed.

## Totals

| Audit doc | Gaps | Fixed | Partially fixed | Still open | No longer applicable | Fixed-claims overturned |
|---|---|---|---|---|---|---|
| enemy-fidelity-audit.md (2026-05-29) | 67 | 33 | 9 | 25 | 0 | 5 |
| movement-fidelity-audit.md (2026-05-30) | 25 | 15 | 2 | 8 | 0 | 3 |
| level-loading-fidelity-audit.md (2026-05-30) | 55 | 11 | 9 | 34 | 1 | 1 |
| platforms-fidelity-audit.md (2026-05-31) | 40 | 5 | 3 | 32 | 0 | 0 |
| helmets-fidelity-audit.md (2026-05-31) | 38 | 10 | 3 | 25 | 0 | 2 |
| weather-fidelity-audit.md (2026-05-31) | 33 | 10 | 7 | 16 | 0 | 0 |
| **Total** | **258** | **84** | **33** | **140** | **1** | **11** |

## enemy-fidelity-audit.md (2026-05-29)

Of 67 gaps, all 67 were re-checked: 33 are fixed, 9 partially-fixed, 25 still open, and 5 of the re-checker's verdicts were overturned by the confirmer.

### Fixed / partially-fixed gaps

| Gap | Sev | Verdict | Evidence |
| --- | --- | --- | --- |
| Armadillo — Berzerker curl-and-launch reaction not simulated | high | fixed | 2cfa4a24 wired the Launch branch (helmets.rs:6047-6061) into step_armadillo_berzerker_launch (enemies.rs:25901); walk-cap landed e174e16c |
| Goat — Berzerker side contact does not launch airborne | high | fixed | 484d72d8; goat_berzerker_side_gate_loc_3cb76 (enemies.rs:4236) wired in helmets.rs:6032-6046, airborne loop step_goat_berzerker_knockback (enemies.rs:25892) |
| Crab — directional vulnerability gate (front-immune) not enforced | high | fixed | 2961571d; take_colid_hurt_or_queue_death (enemies.rs:8641) calls crab_collision_dispatch_loc_3b048 (enemies.rs:614), live via helmets.rs:5851/5909/5940 |
| Crab — frozen crab not immune to hurt collisions | high | fixed | 2961571d; is_vulnerable reads crab_frozen (enemies.rs:8822-8824); freeze branch guards loc_3B048 dispatch (enemies.rs:632) |
| Hand — grab never immobilizes/affects the Kid | high | fixed | 8c3739d5; refresh_active_roster sets kid_grabbed_by_hand (enemies.rs:41358-41363); 4 consumers read it live (movement.rs:5178/5325, physics.rs:722, movement.rs:4973) |
| Robot — live collision/hit routing into charge still absent | high | fixed | 755a864d/e3b22784/a77daded; robot_collision_dispatch_loc_3a8ae (enemies.rs:19395) + shoot dispatch (enemies.rs:19790) live in apply_pending_enemy_collisions (combat.rs:7900) |
| Scorpion — sting attack never produced | high | fixed | 8e439b78; sting armed at enemies.rs:25717-25728, contact damage scanner returns tail hit (combat.rs:8914-8927) via resolve_frame_hazards (combat.rs:11132-11138) |
| Scorpion — Berzerker-charge upward launch not wired | high | fixed | c664c097/40f21d2d; scorpion_berzerker_side_gate_loc_3dd0e (enemies.rs:5832) live in combat.rs:7825 / helmets.rs:6141, airborne loop enemies.rs:25917 |
| Spinning_Twins — orbit form does not split into two children + falling visual | high | fixed | 0558eb4b; materialize_spinning_twins_drop_splits (enemies.rs:41249) inserts two walking children + falling visual on sentinel |
| Spinning_Twins — walking-child launch/drag/settle physics not applied | high | fixed | 3d200381; step_spinning_twins Walking branch (enemies.rs:24751-24789) applies $2800 gravity, $1200 drag, $3000 settle |
| Spinning_Twins — multi-hit HP damage/recovery and death state machine not reproduced | high | fixed | fe7584ed/26221020/bbfce555; loc_3BC34 contact (enemies.rs:8699), death-status route + partner signal in remove_dead_enemy (combat.rs:8017-8074) |
| Tornado — Cyclone-flying-above kill branch not reproduced | high | fixed | 3a0b0a2d; resolve_frame_hazards y-compare death/colid_kidbelow branch (combat.rs:11077-11091) live at lib.rs:3801 |
| UFO — fabricated 60-frame accel sign-flip vs random picker | high | fixed | e9ee41d5; ufo_motion_picker_loc_35326 + script table (enemies.rs:10902-10952) live at wait_timer==0 (enemies.rs:10977) |
| UFO — below-and-near-Kid chase steering not invoked | high | fixed | 52ea553e; ufo_tracking_command_loc_34e2a invoked live with real Kid X/Y (enemies.rs:11007) |
| UFO — two-phase hurt ($47) absent | high | fixed | bf14a91b; spawn_hit_points=2 (enemies.rs:40168), ufo_first_hurt_survive_loc_35228 in take_damage_or_queue_death (enemies.rs:8721-8728) |
| Archer — fire cadence uses arrow re-aim interval vs pose-table | medium | fixed | 833f1f85; ARCHER_SHOOTING_CYCLE_DELAY_FRAMES_LOC_341BE=121 used in step_archer reload (enemies.rs:25572) + first-shot (enemies.rs:40213) |
| Archer — shot-count gate counts shared pool, not arrows | medium | fixed | e7e9c88b; gate filters kind==ArcherArrow >= 2 (enemies.rs:25579-25589), pool-cap demoted to secondary guard (25590) |
| Cloud — lightning fixed 31-frame lifetime vs collision-bound | medium | fixed | b0fbb10b; CloudLightning lifetime:0 exempt from cull/countdown (projectiles.rs:2400-2433), persists until collision |
| Dragon (walking) — breath flame deleted on first Kid contact | medium | fixed | fc226e36; DragonWalkingBreath kept alive on overlap (projectiles.rs:3148-3156) via projectile_hits_player live at combat.rs:10948 |
| Dragon_Flying — descent fireball uses generic projectile vs lifecycle | medium | fixed | 8013f52c; step_dragon_flying_descent_fire_loc_34c26 coroutine (projectiles.rs:135) live via combat.rs:10945 -> projectiles.rs:2752 |
| Fireball — fire-trail child objects never spawned | medium | fixed | aa907ab3; step_bouncer_with_rng spawns FireballTrail on bounce/slope (enemies.rs:13890-13994), live sink passed at enemies.rs:41245 |
| Robot — projectile timing/rendering typed; shoot-time routes | medium | fixed | 020acc0f/e08faa15/5a4434ed/720dd7ba; muzzle/flight/impact helpers (enemies.rs:19886/19921/19950) live in step_projectiles + scene render |
| Scorpion — tail/stinger sub-object absent as live entity | medium | fixed | 8e439b78; scorpion_tail_sync_loc_3dc86 (enemies.rs:5715) drives child_attachment_snapshot (enemies.rs:9018), live contact at combat.rs:8914 |
| Spinning_Twins — Berzerker side-collision upward launch not reproduced | medium | fixed | 83e16aac; spinning_twins_berzerker_side_gate_loc_3bbf6 (enemies.rs:24246) live in helmets.rs:6064-6078, loc_3BC22 loop |
| Spinning_Twins — orbit oscillation timing + transition trigger diverge | medium | fixed | 58844d88; orbit index gated on animation_timer==1, parent x_pos anchored, drop gated on neg/$2C/$1C (enemies.rs:24007-24052) |
| Tank — non-fatal hit recoil + motion stop + hibernate not applied | medium | fixed | e93e233b; tank_update_loc_3b41c clears motion + hibernate consumed in step loop (enemies.rs:9709-9728); TankShooting via loc_3b700 |
| Tar_Monster — burst-child projectiles lack off_32656 bounce | medium | fixed | a2c90f2d; tar_monster_projectile_collision_loc_32682 (projectiles.rs:235) live via combat.rs:10945; budget seeded 2 (enemies.rs:21603) |
| Tar_Monster — straight attack glob does not burst on impact | medium | fixed | d33c67f8; glob terrain contact spawns 5 children (projectiles.rs:2855-2864) via spawn_tar_monster_retaliation, live at combat.rs:10945 |
| Tornado — terrain collision simplified; ceiling bounce missing | medium | fixed | 0fa4a85e; ceiling probe gated on y_velocity<0 -> apply_tornado_terrain_collision_loc_35948 BounceFloorCeil (enemies.rs:12848-12872) |
| Tornado — slope bounce (BounceUp/DownSlope) absent | medium | fixed | 0fa4a85e; slope detection + ±$10000 vectors in apply_tornado_terrain_collision_loc_35948 (enemies.rs:12856-12913), live |
| UFO — collision-type bounce table dispatch not used | medium | fixed | 1c30fe67; apply_ufo_terrain_collision_loc_35056 (enemies.rs:11074-11131) dispatches 4 categories + loc_350F2 rollback, live |
| UFO — random beam trigger uses non-ROM position hash | medium | fixed | f895cbe4; ufo_random_beam_should_start_loc_34f8e gates on rng.next_byte()<=4 (enemies.rs:11280-11288); position hash removed |
| Big_Hopping_Skull — patrol x-bounds and relaunch reset never run | high | partially-fixed | 7ec2d68b fixed the relaunch leg (enemies.rs:13785-13790); patrol x-bounds (loc_36770/36782/36798) STILL OPEN — big_hopping_skull_bounds_correction_loc_36770 (enemies.rs:3414) reachable only via test-only entry (enemies.rs:3282), live step never reads $3E/$40 bounds |
| Ninja — Berzerker side-contact knockback launch not wired | high | partially-fixed | e27dd54c wired loc_3D698/loc_3DB0A (enemies.rs:17427, live via helmets.rs:6107); loc_3D8A8/loc_3D8BC knockdown branch REMAINS — handlers (enemies.rs:17557/17606/17640) still dead_code, test-only callers |
| Tank — body-vs-mouth/turret vulnerability gating not enforced | high | partially-fixed | 00b34ef4/f654f3b4/00e59120 route Tank/TankShooting damage through child snapshot (combat.rs:7754-7795, helmets.rs:4332-4425); REMAINS — TankShooting stomp hitbox anchored on body, not turret child (stomp_contact_candidates special-cases only Tank at combat.rs:8197) |
| Driller — probe offset, wall-collision gate, Ice bullet dispatch | medium | partially-fixed | 38fcaeae/cb860578 fixed probe ±$18 (enemies.rs:16678) and 3 ice bullets (blocks.rs:2208); REMAINS — loc_3BE8E front-wall gate is a static Solid-cell probe, not the runtime collision-word gate $38(a4); regression at level_tests.rs:20513 passes via static check only |
| Goat — collision-type dispatch (death-on-negative, side-gate) | medium | partially-fixed | 484d72d8 (side-gate) + 8f582184 (HP/recoil loc_3CB38) fixed; core loc_3CB14 dispatch STILL OPEN — goat_collision_dispatch_loc_3cb14 (enemies.rs:4148) dead_code, no EnemyFamily::Goat branch in apply_pending_enemy_collisions (combat.rs:7730) |
| Lion — charge purely horizontal; ledge-launch leap absent | medium | partially-fixed | c6d851d6 added the no-floor airborne branch (enemies.rs:19312-19323) with gravity via step_dragon_walking_airborne; REMAINS — fall cap clamps at MAX_FALL_SPEED=0x8_0000 (enemies.rs:22172) vs ASM loc_3E77E $18_0000; faithful loc_3E77E helpers still test-only |
| Robot — body/head per-part collision and sprite swaps | medium | partially-fixed | 85d76046/8dca4122/5239f6c3/f87b0f42 wired four collision legs live (enemies.rs:19395, helmets.rs:4345/4379, combat.rs:8210); REMAINS — head hit-frame sprite swap (RobotChildSpriteUpdateLoc3A8Fc::UseHitFrame, enemies.rs:19494) computed but unconsumed by live caller (enemies.rs:8677) |
| Scorpion — collision/HP state machine not wired | medium | partially-fixed | 68250e30/4c9b2cea wired loc_3DCB0 routing + loc_3DCD4 hurt-recovery live (combat.rs:7798-7846, enemies.rs:9790-9799); REMAINS — death tail $3D flag: native HIDES the tail (enemies.rs:8938-8944) whereas ASM loc_3DCF4 ARMS it; faithful scorpion_death_route_loc_3dcf4 (enemies.rs:5801) still dead_code |

### Still-open gaps

- Diamond — live HP damage and hurt/death collision words not plumbed
- Diamond — Berzerker side-launch not plumbed
- Armadillo — damage recoil/invulnerability window not applied
- Armadillo — spawned-child fall-then-die death sequence not simulated
- Fire_Demon — walking fire-trail emission never happens
- Hand — airborne collision dispatch table not wired
- Hand — ground collision dispatch table not wired
- Lion — hurt-recoil response absent
- Mini_Hopping_Skull — runtime uses generic step_hopper; hop state machine not wired
- Mini_Hopping_Skull — air-damage knock-up loop not driven
- Ninja — attack trigger ignores $54/$6E cadence; divergent detection cone
- Ninja — blocking/guard-pose state machine + block counter not executed
- Ninja — HP-depleted knockdown fall (above-stomp) absent
- Orca — in-lunge Kid contact (hurt/Kid-above/side) not dispatched
- Orca — ceiling-hit HP decrement counter ($44) not modeled
- Big_Hopping_Skull — per-hit SFX cadence + kid-upward-bounce contact bug not driven
- Big_Hopping_Skull — death respawn marker (EnemyStatus_Table[d0]=$2168) not written
- Goat — Berzerker side contact during recoil ($42>0) does not shove+hurt-resume
- Hand — grab entry gate conditions not checked
- Hand — post-grab release re-jump skips the loc_3639E free-space check
- Lion — close-band tracker + $42 charge-windup state machine not used
- Mini_Hopping_Skull — post-landing facing-flip re-orientation pause not reproduced
- Ninja — jump-attack airborne loop omits ceiling/kid-above branches
- Ninja — far-Kid facing turn during pacing not applied
- Orca — side-collision Berzerker re-lunge (loc_3D4DE) absent

The fixes cluster heavily on Berzerker side-contact launches and the Spinning_Twins/Tank/UFO/Tornado subsystems, while the still-open gaps share one recurring signature: ASM-faithful helpers exist but remain `#[cfg_attr(not(test), allow(dead_code))]` with only test callers, never wired into the live frame step.

## movement-fidelity-audit.md (2026-05-30)

Re-checked 2026-06-10: of 25 gaps, all 25 re-checked — 15 fixed, 2 partially-fixed, 8 still-open, with the confirmer overturning 3 verdicts.

| Gap | Sev | Verdict | Evidence |
| --- | --- | --- | --- |
| Crawl stand-up wall escape (`sub_81B4`) implemented but never called | high | fixed | b9d1667a wired it; live path `crates/kid-core/src/lib.rs:3103-3133` -> `crates/kid-core/src/movement.rs:7998-8059`; the `#[allow(dead_code)]` at `movement.rs:7997` is now stale but cosmetic. |
| Airborne Skycutter air-jump does not delegate to `sub_9A20` | high | fixed | 8cd46fae added Skycutter branch ahead of `JUMP_AIR_PROFILES`; `crates/kid-core/src/movement.rs:5218-5225` and `5564-5571`, helper `crates/kid-core/src/physics.rs:561-615`, live at `movement.rs:3396`. |
| Slope walking — Up-held/Down-held do not drive uphill/downhill accel | high | fixed | 322be650 added `slope_horizontal_input_loc_9DAA` at `crates/kid-core/src/movement.rs:7304-7336`, fed to dispatch `movement.rs:7399`, live at `movement.rs:5109/5149/5493/5524/6985`. |
| Main object update — `$14/$18` compound-corner response absent | medium | fixed | 82eb26c0/d6323cb5/d53f7e61/d87c44c7/a4e1e4d0 ported it; `try_compound_slope_recontact_loc_a4ee` at `crates/kid-core/src/movement.rs:6824`, live call site `movement.rs:3729`; "not yet ported" line gone. |
| Crawl — `sub_7A10` entry `blocked` result discarded | medium | fixed | 4d0cd5de reads `.blocked` at `crates/kid-core/src/lib.rs:3168-3186`, clearing `movement_input.down`; walk-vs-crawl decision keys on it at `movement.rs:5252` (helper `movement.rs:7655`). |
| Jump — `KidGrabbedByHand` `+$20000` weakening modelled but unwired | medium | fixed | 72e681a7 plumbed flag into jump sites (`crates/kid-core/src/movement.rs:5322/5354/5739/5759/5811/5832`); 8c3739d5 sourced the grab (`crates/kid-core/src/enemies.rs:41358-41364`); arithmetic `physics.rs:757-761`. Integration-verified, not oracle-verified. |
| Damage — `apply_damage` clears wall-grab/cling the ROM never touches | medium | fixed | eb8d39f5 removed the wall-stick clears; current `apply_damage` at `crates/kid-core/src/combat.rs:9233-9310` writes no wall state; regression `level_tests.rs:17286`. |
| Crawl — `KidGrabbedByHand` not checked in entry gate | low | fixed | 8c3739d5 added the flag check at `crates/kid-core/src/movement.rs:5178` (and `5252`); sourced live at `crates/kid-core/src/enemies.rs:41358-41363`, set before player step (`lib.rs:2078` before `lib.rs:2183`). |
| Jump — `sfx_Jump` not emitted on launch | low | fixed | 0ecf4772 added `queue_jump_sound_from_probe_loc_A480` (`crates/kid-core/src/lib.rs:2616-2633`), live at `lib.rs:3261/3292`; ce5a3970 dedups the native fallback. |
| Airborne Skycutter wrongly gets the 1.5x run-accel multiplier | low | fixed | Root cause wired by 8cd46fae; Skycutter intercepted before the generic `accel+(accel>>1)` (`physics.rs:297-301`) at `crates/kid-core/src/movement.rs:5218-5225` and `5564-5571`. |
| Slope walking — steady frame does not mirror `y_vel = ±x_vel` | low | fixed | 33ef1b96 added post-gravity re-mirror at `crates/kid-core/src/movement.rs:3440-3453` (after the `sub_B168` grounded clear at `movement.rs:5401-5402`); in `AsmFaithful` branch `movement.rs:3065`. |
| Climb — wall-end release omits −4px/frame pop + jump-state handoff | low | fixed | 94003dfd writes `y_velocity = -0x4_0000` at `crates/kid-core/src/helmets.rs:5485-5492`; live at `lib.rs:3015`, custom_gravity drops next frame `lib.rs:3079`. |
| Climb — loop ignores per-frame `Addr_PlatformStandingOn` exit | low | fixed | 3b0d75e6 added `exit_iron_knight_climb_on_platform_landing_loc_A9AA` (`crates/kid-core/src/lib.rs:3815`), called live at `lib.rs:3680` (after platform collision at `lib.rs:3664`). |
| Climb — B-press jump-off (separate `sub_A3BC` hygiene) stale dead-code attr | low | fixed | 6744668e stripped `#[cfg_attr(not(test), allow(dead_code))]` from `iron_knight_climb_visual_sub_A3BC` (`crates/kid-core/src/helmets.rs:863-864`), live at `helmets.rs:5535`; sprite-mapping resolved by d75d0cfa/c7f407dd. |
| Climb — airborne wall-attach is a native extension with no ROM entry | low | fixed | 52ae15d6 added `&& slice.player.on_ground` to the gate and deleted the airborne branch; `crates/kid-core/src/helmets.rs:5393-5420`, live at `lib.rs:3015`. |
| Wall/ceiling — slope-compound corner-crush (`loc_B9A2`/`loc_BA5A`) unported | medium | partially-fixed | a4e1e4d0 corrected the no-kill premise and 82eb26c0/d87c44c7/d6323cb5/d53f7e61 wired the Juggernaut/Skycutter bounce (`crates/kid-core/src/movement.rs:6824-6974`, live `movement.rs:3729`). Remaining: the grid-aligned wall-hit recovery snap `loc_BA1C`/`loc_BAD2` is still unported, and the stale comment at `movement.rs:3574-3579` no longer holds since non-armoured `$14/$18` handoffs (Red Stealth #269, Iron Knight #258) now produce those collision types — #258 still open at P0 (`backlog.md:469`). |
| Braking — Left+Right conflict resolver (`sf Ctrl_Right_Held`) unwired | medium | partially-fixed | 031e69f8 wired `with_left_right_conflict_resolved` into `gameplay_input` (`crates/kid-core/src/lib.rs:2583-2606`, consumed `lib.rs:2117-2123`); a1e751f4 hardened `held_horizontal_direction` (`movement.rs:4254-4263`). Remaining: the demo-live branch (`lib.rs:2119-2120`, demo frame > 81) passes raw unresolved input into the velocity/brake math (`movement.rs:3219-3220`, `physics.rs:543` tests `input.right` first), so a both-held demo frame still resolves RIGHT contrary to `loc_7464`; that helper only governs the stationary wall gate. |

Still-open gaps (authoritative in the original doc):

- Ground accel — terrain index selection inverts the ROM's ice/rubber swap (high; symbolic mapping live at `types.rs:936-937` / `movement.rs:7771-7772`; swap fix attempted then rejected 2026-05-31 for regressing oracle death-respawn-level1; #218 still `[?]`).
- Landing — moving-platform carry bypasses the `sub_8F26`/`sub_902A` move-and-collide passes (medium; additive carry only at `platforms.rs:778-781`, no wall re-collide on carried X; #208/#227 still `[?]`).
- Berzerker — charge negates Kid `x_vel` unconditionally; ROM gates on `object_meta` bit 15 (medium; CONFIRMER OVERTURNED to still-open — gate keys on boss-family membership, conflating `bset #7` with bit 15, so it recoils off bosses the ROM passes through and misses the lunging Scorpion that actually sets bit 15).
- Ground accel — stale `not(test)` dead_code attr on wired `grounded_surface_type_sub_8600` (low; attr persists at `movement.rs:7721`; #209 sweep deliberately excluded movement.rs, folded into #217).
- Climb — B-press jump-off bypasses the `KidGrabbedByHand` weakening (low; still calls `physics::initiate_jump(0, ...)` at `helmets.rs:5447`; #219 explicitly scoped out IronKnight climb, but ASM `sub_A432` is helmet-agnostic so weakening should apply).
- Landing — grounded platform carry never re-seats `y_pos = platform_top − 1` (low; additive carry only at `platforms.rs:778-781`, deviation comment at `platforms.rs:763-765`; #208 still `[?]`).
- Damage — `clear_special_movement_state_sub_DB22` is test-only (low; still `#[cfg(test)]`-gated at `helmets.rs:4900-4929`; production uses live `refresh_grounded_helmet_state_sub_78E8`; #209 correctly left this genuinely-test-only fn gated).
- Skycutter — both-direction-held facing flip diverges from `sub_9A20`'s sequential branch (low; `next_facing` at `physics.rs:573-579` tests `input.right` first so both-held -> RIGHT, vs ROM's flip-toward-opposite; helper now live-wired so deviation reaches game state, untested for simultaneous L+R).

Pattern: the climb subsystem absorbed the largest share of fixes (five of the fifteen closed), with the bulk of the post-audit work landing in the 2026-05-31 sweep one day after generation; the three confirmer overturns were all "fixed -> downgraded" (one to still-open on Berzerker, two to partially-fixed), with the recurring failure mode being a real but narrow residual leg — demo-live input, the `loc_BA1C`/`loc_BAD2` recovery snap, or a gate keyed on the wrong condition — surviving an otherwise-resolved dominant path.

## level-loading-fidelity-audit.md (2026-05-30)

Re-checked all 55 gaps: 9 fixed, 11 partially-fixed, 34 still-open, 1 no-longer-applicable, and 1 verdict overturned by the confirmer (the Forced-Entry steel-block hardcode, downgraded from a re-checker "fixed" to partially-fixed below).

| Gap | Sev | Verdict | Evidence |
| --- | --- | --- | --- |
| Foreground terrain decode — non-oracle fallback misread raw BLOCK resource as final cells | high | fixed | `ba3e104d` / `9270d935` / `fedad2c9`; old `load_decoded_block_grid_for_level` gone (zero refs); `decode_level_layout` runs the faithful Saxman+theme-collision+run-list chain at crates/kid-core/src/level.rs:18502-18608, dispatching via `decode_block_layout` (crates/kid-data/src/lib.rs:4393), live from `warp_to_level` (level.rs:18133) |
| Theme art/tileset load — FG/BG art VRAM decompress + palette permutation entirely dead | high | fixed | `db7c351e`; reimplemented live in render layer — `load_level_visuals` decompresses FG/BG art and applies per-theme permutations at crates/kid-render/src/bundle.rs:203-294, reached via crates/kidcli/src/native_player/mod.rs:1132 (original kid-core helpers still dead) |
| Theme art/tileset load — Hill alt background art (second BG decompress) never executed | high | fixed | `db7c351e`; `background_followup_art_loc_11ea2_11ec4` returns Hills→hill_alt.bin at VRAM $07C0 at crates/kid-render/src/bundle.rs:794, appended live at bundle.rs:289 |
| Theme art/tileset load — Mountain cave/lightning weather BG variant never executed | high | fixed | `db7c351e`; theme 7→cave_alt ($7E5), theme 7 + weather 2\|3→mountain_lightning at crates/kid-render/src/bundle.rs:806-818, live via crates/kidcli/src/native_player/mod.rs:1132 |
| Block layout — `LoadBlockLayout` bit-stream decoder (`loc_132A6`) modelled but dead | medium | fixed | `22bbd57e` / `9270d935` / `ef5f1165`; wrong `decode_enigma` delegation replaced by faithful `decode_block_layout` (crates/kid-data/src/lib.rs:4393), wired into `decode_level_layout` (crates/kid-core/src/level.rs:18588), live from boot.rs:5746; residual: fixtured levels still prefer the RAM dump |
| Background scroll — `Load_BackgroundMappings` type1/type2 chunk install unwired | medium | fixed | `db7c351e` / `2a8920ef`; `load_background_visuals` dispatches type1/type2 at crates/kid-render/src/bundle.rs:823/903, live from mod.rs:1132 and rendered via crates/kidcli/src/native_player/scene.rs:1597 (kid-core helpers still dead) |
| Load_InGame — Forced Entry steel-block special-case (`loc_11C7E`) not modelled | medium | fixed | `9270d935`; steel-block insert at cell ($20,$B)=`0xE50B` in `decode_level_layout` (crates/kid-core/src/level.rs:18562-18571) plus fixture leg (21.bin byte 5784); both live legs produce the block |
| Level header decode — blank-top-rows never added to level height / `Camera_max_Y` | medium | fixed | `7bbf0fca`; `level_bounds` now folds `blank_top_rows` into height/camera at crates/kid-core/src/level.rs:19434-19456, live clamp at lib.rs:3143; regression `load_leveldata_height_flags_extend_level_bounds_loc_15a0` (448→544px) |
| Foreground terrain decode — no native theme-collision index table | low | fixed | `ba3e104d` / `fedad2c9`; `theme_collision_table_loc_11C36` builds the contiguous 256-entry table at crates/kid-core/src/level.rs:1810, packed by `level_layout_cell_loc_11C36` (level.rs:4189), wired live into `decode_level_layout` (level.rs:18535/18555) |
| Block-type dispatch — entire `AddrTbl_BlockTypes` dispatch unwired | high | partially-fixed | `9270d935`; all 17 types ported as `apply_block_overlay_loc_11C7E` (crates/kid-core/src/level.rs:4264) and wired into `decode_level_layout`, but `load_block_grid_for_level` still prefers the RAM dump (level.rs:18460) so full dispatch fires live only for fixture-less bonus levels; only the type-1 prize leg (`reconcile_prize_block_cells_loc_123DC`, level.rs:4480) runs per-cell for all levels |
| Block layout — `L_Forced_Entry` steel-block hardcode never implemented | high | partially-fixed | `9270d935`; faithful hardcode now exists at crates/kid-core/src/level.rs:18562-18571 and is reachable via the fixture-absent fallback, but for the real level 0x21 both 21.bin (16800 bytes) and 21.json exist, so the live path takes the ROM-dumped fixture branch (level.rs:18460) and the native insert is dead for the actual level — confirmer overturned the re-checker's "fixed" |
| Foreground terrain decode — native `Decompress_Chunk` + theme-collision mapping dead | medium | partially-fixed | `ba3e104d` / `9270d935` / `ef5f1165` / `fedad2c9`; theme-collision mapping now live via `decode_level_layout` (crates/kid-core/src/level.rs:18502), but it is only a fallback (fixtured levels still use the dump) and `decompress_chunk_attempts_loc_11C10` (level.rs:1923) stays test-only dead |
| Theme art — shared blocks/number/flag/beam art VRAM loads never executed | medium | partially-fixed | `a38535d4` / `fc4ae525`; 4 of 6 legs now live via `load_level_visuals` — block art, HUD numbers, flag-bottom (crates/kid-render/src/bundle.rs:4146), hitpoint art (bundle.rs:255, drawn hud.rs:124); still open: cyclops-beam (`Load_EyclopsBeamArt`, zero asset refs) and juggernaut skull/star (`sub_129CE`, crates/kid-data/src/lib.rs:1610, test-only) |
| Load_InGame — blank-top-rows grid shift (`sub_140F6`) unwired | medium | partially-fixed | `7bbf0fca`; `blank_top_rows` now folded into live bounds/camera (crates/kid-core/src/level.rs:19441, lib.rs:3143); still open: the grid SHIFT — `insert_blank_top_rows_*` (crates/kid-core/src/terrain.rs:415/454) and eight `adjust_*_for_blank_top_rows_loc_141EE..` (level.rs:7800-7972) stay test-only, so `decode_level_layout` never shifts rows down, blank-fills the top, or fixes block/hidden coords |
| Enemy spawn — `Manage_EnemyLoading` per-frame streaming spawn walk dead | high | partially-fixed | `a0ceb74e`; one leg wired — `enemy_spawn_inside_visible_inner_window_loc_36900` (crates/kid-core/src/enemies.rs:8574) live at enemies.rs:9386; the ROM status-word state machine (level.rs:8081-8196) stays dead and the live path still uses the invented generic activation window (enemies.rs:9381); commit body itself says "#213 remains open" |
| Post-load — lava/water + Island shore-anim objects never spawned | medium | partially-fixed | Fixed candidate refreshed 2026-06-16: live `begin_level_entry` spawns the Island `loc_1202A` cycle and lava/water `loc_1223C` rotator, frame stepping ticks both, and trace exports Island DMA + lava/water palette words. Covered by `begin_level_entry_spawns_island_shore_cycle_loc_11fb0`, `step_island_shore_cycle_ticks_loc_12098_12110_12188`, `begin_level_entry_spawns_lava_water_rotator_for_cave_loc_11f96_1223a`, and `trace_exposes_lava_water_palette_words_loc_1223c`. Remaining: renderer shore-art DMA consumption and hardcoded override palette word data, as tracked in `level-loading-fidelity-audit.md`. |
| Post-load — level-start music never emitted at level load | medium | fixed | Fixed 2026-06-16: `begin_level_entry` now queues the core `loc_11FD2` BGM id, including the murder-wall override before the foreground-theme lookup, and native-player prefers that queued id. Covered by `begin_level_entry_queues_level_bgm_from_loc_11fd2`, `begin_level_entry_queues_murderwall_bgm_before_theme_lookup_loc_11fd2`, and `play_native_level_bgm_prefers_core_queued_loc_11fd2_id`. |
| Post-load — `Init_SpecialEffects` only handles weather 1; storm/unknown branches not wired | medium | partially-fixed | `e0ecf788` / `95733ca5` / `0df73ce5` / `0ea02251` / `6f2eb4c3`; storm 2\|3 now wired in `init_weather`/`step_weather` (crates/kid-core/src/weather.rs:2461-2488/2344) with ASM `loc_14DAC` scalars and hail spawn, live from lib.rs:3758; still open: unknown(4) leg — no type-4 arm, `dispatch_level_special_effects_sub_6034` (weather.rs:174) returns None for index 4, `special_effect_unknown_*` (weather.rs:495-575) test-only |
| Block-debris — `sub_2A4C` compile debris sprites dead; live cull uses world-space `any()` | medium | fixed | Fixed candidate refreshed 2026-06-16: live `blocks.rs` now steps `sub_2A4C` debris, culls the lead shard relative to the camera, clears the whole slot when the tail shard reaches relative x `$1EF`, and exposes falling pieces for the renderer's `$8228` tile path. Covered by `highwater_block_debris_culls_against_camera_not_world_origin_sub_2a4c`, `live_block_debris_tail_fragment_culls_slot_sub_2a4c`, `compile_block_debris_sprites_sub_2a4c_tail_fragment_x_threshold_clears_after_four_sprites`, and `falling_block_debris_uses_sub_2a4c_rom_tile_after_destroy_handoff`. Separate `sub_28FC`/`sub_292E` scratch-init/spawn concerns remain listed below. |
| Post-load — murderwall palette/art VRAM setup omitted from `init_murder_wall` | low | partially-fixed | `78b9a4f3` / `5d80af0a`; cosmetic art+palette now loaded via render layer `MurderWallVisuals` (crates/kid-render/src/bundle.rs:120, player.rs:74, tile $5F60); still open: `copy_murder_wall_palette_loc_140EE` (crates/kid-core/src/weather.rs:8) stays test-only and `init_murder_wall` (weather.rs:2598) writes no VRAM/palette — fixed by a parallel asset path, not by wiring the helper |
| Block-mapping build — `block_type_base_cell_loc_122B8` `word_122C6` table dead | low | no-longer-applicable | `9270d935` / `22bbd57e`; helper now called live at crates/kid-core/src/level.rs:4315 for types 0\|2\|5\|7\|9 via `apply_block_overlay_loc_11C7E`, so the "dead/unwired" premise no longer holds (the original misplacement sub-claim also rested on a routing misread); residual is a stale docstring at level.rs:4861 only |

### Still-open gaps

- Block layout — hidden/prize scratch-list build (`loc_11C96/loc_11CA8`) and FG-queue overflow guards never wired
- Theme palette load — FG theme palette copy (`loc_11DB0`) modelled but never wired
- Theme palette load — BG theme palette copy (`loc_11E16`) modelled but never wired
- Theme palette load — per-map override palette DATA absent from native
- Block-mapping build — post-decode pointer cursor (`level_load_pointers_loc_11D0C`) dead; theme-mapping/enemy-layout pointer setup unwired
- Enemy spawn — `loc_11D0C` enemy-layout pointer install dead; live load uses a parallel re-decode
- Enemy spawn — `loc_31F8E` reset+seek (status-table seed, header-threshold walk, art/palette) entirely unwired
- Enemy spawn — multi-record layouts collapsed to a single segment; `loc_31FD4` count-skip seek unmodeled
- Background scroll — BG dimensions/format/theme load-time setup entirely unwired
- Background scroll — `off_7B1EC` scroll-speed list never wired (dead per-frame parallax)
- Background scroll — per-frame shift-data compute + VSRAM transfer dead; no scroll output
- Fixed 2026-06-16 — Load_InGame — level BGM selection (`loc_11FC8/loc_12004`) never invoked on the live load path; `begin_level_entry` now queues the core `loc_11FD2` level-start BGM including the murder-wall override, and native-player prefers that queued id, covered by `begin_level_entry_queues_level_bgm_from_loc_11fd2`, `begin_level_entry_queues_murderwall_bgm_before_theme_lookup_loc_11fd2`, and `play_native_level_bgm_prefers_core_queued_loc_11fd2_id`
- Fixed 2026-06-16 — Level header decode — `$FFFFFC29` continue-flag player-start skip not modelled; current level entry preserves caller-written starts through `pending_player_start_override_loc_11bd2`, covered by `preserved_player_start_entry_floor_snaps_loc_11bd2` and `loc_11bd2_preserves_existing_player_start_when_the_teleport_latch_is_set`
- Theme palette — Plethora FG override not representable in the classify enum
- Block-mapping build — downstream consumer (`loc_165E`) also dead, confirming OR-masked `Block_Mappings` never reach a tile word
- Platform install — `Initialize_Platforms` 17-entry slot free-list never built in the live path
- Platform install — no 17/`0x12` active-platform cap; free-list exhaustion path absent
- Block-debris — `sub_28FC` scratch init dead; live `Load_InGame` uses `Vec::clear`
- Block-debris — `sub_292E` debris spawn dead; live path is a parallel reimplementation in `blocks.rs`
- Load_InGame — FG/BG theme-flag decoders (`loc_11B92/loc_11B9A`) are test-only on the load path
- Level header decode — `Background_format`/layered-enigma width=`$40` not computed in runtime header decode
- Fixed 2026-06-16 — Level header decode — `Foreground_theme` sign-bit masking edge: bit6 stripped when bit7 clear; `kid-data::parse_level_header` now follows `loc_11B92`'s sign-bit-gated mask, covered by `parses_positive_foreground_theme_bit6_as_theme_data_loc_11b92`
- Theme palette — palette permutation tables (`loc_11E48/loc_11E8A`) past palette copy also unmodelled in live path
- Fixed 2026-06-16 — Enemy spawn — `loc_31F8E` axis-threshold seek uses unsigned compare in the live segment pick vs ROM signed `blt`; live `enemy_layout_view` now compares signed 16-bit threshold words, covered by `enemy_layout_view_loc_31fd4_uses_signed_threshold_words`
- Platform install — `Get_PlatformLayoutAddress` `MapPtfmLayout_Index`/`MapOrder_Index` indexing absent from runtime
- Platform install — duplicate-load and loader-offset advance helpers (`loc_286A / loc_286E`) modelled but unwired
- Camera/window init — faithful `loc_11F48` helper is dead; live path re-derives the snap inline
- Camera/window init — window/HUD plane VRAM clear (`$E000` fill `$0780`×`$800`) not executed
- Death respawn — enemy roster cleared but not re-primed for the settle window (`loc_368A8`)
- Death respawn — `Extra_hitpoint_slots` not cleared on `lose_life` (field now live, deviation active)
- Warp — teleporter door `FBCC=$EE` yellow warp flash and `d6=-5` Teleport SFX omitted
- Flagpole — hologram fade-out step modelled but unwired; no palette fade applied
- Fixed 2026-06-16 — Flagpole — EOL extra-life award routed through generic `add_score`/`arm_hundred_k_trip` rather than `loc_D79E`; current completion bonuses use `eol_score_counter_apply_loc_D77E`, covered by `loc_D77E_eol_score_counter_does_not_arm_hundred_k_trip_timer` and `loc_D77E_eol_score_counter_applies_cap_and_single_life_boundary`
- Level-name/title-card — faithful `load_title_card_*_loc_*` centering helpers are dead; logic re-implemented inline in `vdp.rs`

Pattern: nearly every fix landed in the theme-art/background subsystem and was achieved by reimplementing faithful behaviour in the kid-render asset/render layer rather than wiring the original kid-core helpers, which mostly remain test-only dead code.

## platforms-fidelity-audit.md (2026-05-31)

Re-checked all 40 gaps on 2026-06-10: 32 remain still-open, 5 fixed, 3 partially-fixed, and 0 overturned by the confirmer. Follow-up 2026-06-13: crawl-on-platform `sub_8ED0` width wiring, type-0 platform height decode, scripted-position wrapping, and lift ceiling crush are fixed, leaving 27 still-open from this audit.

| Gap | Sev | Verdict | Evidence |
| --- | --- | --- | --- |
| Platform loader — range check used camera window instead of Kid position + absolute (byte×32) edges | high | fixed | 3c43cb62 — `platform_window` built from `slice.player.x_pos/y_pos` as a one-point Kid window (enemies.rs:41307-41315); `buffer<<5` absolute edges in `platform_buffer_edge_px_loc_2762` (platforms.rs:522) consumed by `platform_is_active` (platforms.rs:635-647) |
| Platform loader — placement x/y decode omitted `asl #4` (×16), position 16× too small | high | fixed | 3c43cb62 — `platform_layout_coord_px_loc_2762` = `coord << 4` then `<< 16` fixed-point (platforms.rs:509-518); routed via `from_placement` (types.rs:2228,2242) and `active_platforms` (platforms.rs:612-613) |
| `sub_8ED0` — live carry/support check hardcoded Kid half-width (5), ignoring current form | high | fixed | ca11b4ad — caller now passes `current_form.half_width()`/`current_form` (platforms.rs:712-718); table at types.rs:874-887; live from lib.rs:3664-3669 |
| Lift platform — initial y-velocity invented as 0 (ROM is −65536 / −1px/frame up) | high | fixed | 6e7b58a1 — live spawn seed `y_velocity: -0x0001_0000` (blocks.rs:3646-3652), gated on `lift_block_hit` set at movement.rs:6440; stepper sub_FCA6/sub_FD00 (blocks.rs:1057,494) |
| Platform activation — camera-window basis instead of ROM Kid-centred per-platform buffered edges | high | fixed | 3c43cb62 — one-point Kid window (enemies.rs:41310-41315); `buffer_edge<<5` absolute world rectangle in `platform_buffer_edge_px_loc_2762` (platforms.rs:522-527) / `platform_is_active` (platforms.rs:635-648); live via lib.rs:2078 |
| `Platforms_CheckCollision` — escape-vector/flag algorithm modelled but unwired; live path a snap-tolerance approximation | high | partially-fixed | 3d6317a4 — escape-vector push wired live (platforms.rs:835-877, lib.rs:3664): horizontal/vertical magnitude compare + `apply_horizontal_platform_push_loc_2686`. Remaining: carry fields loc_26A2/26B6/26C0/26FE still `allow(dead_code)`, FAA contact-flag loop, `Addr_PlatformStandingOn` writeback, restart loop, and vertical-wins landing still uses `GROUND_SNAP_TOLERANCE_FP` (platforms.rs:883) |
| `Platforms_CheckCollision` — horizontal carry (loc_2686/26B6/26C0) omitted; Kid never pushed/carried sideways | high | partially-fixed | 3d6317a4 — sideways push wired (platforms.rs:835-876, sets on_ground=false). Remaining: velocity-carry + wall-flag legs still open — `positive/negative_platform_carry_loc_26A2/26B6/26C0` stay `#[cfg_attr(not(test),allow(dead_code))]` (callers only block_tests.rs); live push writes only x_pos, no `$26` carry/`$FAA2-3` flags; on-ground branch (platforms.rs:778) still additive with no sign-clamp |
| Platform preload — invented 64px X padding + camera basis replaced ROM per-record window | high | partially-fixed | 3c43cb62 fixed 3 of 5 legs (raw-word x, unscaled buffer_right, camera basis → Kid pos at enemies.rs:41308-41315; scaling via platforms.rs:585-595). 2026-06-13 follow-up removed the invented 64px `PLATFORM_PRELOAD_X_PADDING`. Remaining: the `advance_to_window`→`take(next_index).filter` decomposition (enemies.rs:41316-41344) still replaces the ROM single 12-byte-record cursor |

Still-open gaps (remain authoritative in the original doc):

- Slot pool — free-list replaced by monotonic `next_index`; no runtime allocation model
- Sprite emission/tiling dead; live renderer draws one fixed frame (no width/height tiling)
- `Platforms_CheckCollision` — `Addr_PlatformStandingOn` remains a native vector index, not a ROM slot address; `loc_26D4`/`loc_26FE` carry and same-frame restart/crush legs are now partially live
- `Platforms_CheckCollision` — FAA2–FAA5 contact flags, FA96 jump-platform ref, and `loc_271A` crush escalation never set
- TrapPlatformDown — entire coroutine unwired; 40 level instances become inert platforms
- TrapPlatformUp — entire coroutine unwired; 42 level instances become inert platforms
- Special-platform dispatch keys off `raw=="0"` instead of the `off_40D2` index (pp/4)
- Lift platform — ride/carry bypasses the `sub_8ED0` framework (bespoke ±[−2..4] foot snap)
- Slot pool — 17-slot capacity / `Number_Platforms` cap counter not modelled
- Platform loader — architecture mismatch: ROM one-record-per-frame cursor vs native full-rescan; dedicated helpers dead
- Field decode — art selection bypasses `PlatformArtAddr_Table`; render-flag vs type nibble conflated
- `Platforms_CheckCollision` — invented `GROUND_SNAP_TOLERANCE_FP` replaces exact AABB overlap test
- `Platforms_CheckCollision` — single-pass full-slot loop with epilogue decomposed into multiple recomputed passes
- Walk/crawl platform crush — additive carry instead of ROM snap-to-(platform_y-1)+copy-y_vel; d7==2 ceiling-up path approximated
- TrapPlatform — deactivation flag `$12(a3)` not modelled for special platforms
- Platform loader — offset/list helpers modelled but never wired into live roster build
- Layout indexing — platform index sign-extension (`ext.l`) not modelled; unsigned add only (no base-ROM effect)
- Scripted movement — global freeze gate `$FFFFFA64` not modelled in live caller
- SoftPlatform — launch velocity sampled post-physics instead of frame-start (one-tick timing nuance)
- SoftPlatform — `soft_platform_settled_step_loc_4160` is a dead/test-only helper (logic inlined elsewhere)
- Lift platform — `sfx_Elevator_block` sound never played
- Lift block — pre-spawn block bob animation (`sub_FB3E`) omitted; bob helpers dead
- Platform ride Y snap uses shared 6px `GROUND_SNAP_TOLERANCE_FP` rather than hitbox overlap

All movement in the past fortnight clustered in the platform loader/decode subsystem: commit 3c43cb62 alone closed three loader/activation gaps (ptfm scaling, Kid-centred range basis, ×16 decode) and partially cleared the preload gap, while the `Platforms_CheckCollision` collision model saw only the escape-vector push (3d6317a4) wired, leaving its carry, contact-flag and trap-coroutine legs still open.

## helmets-fidelity-audit.md (2026-05-31)

Of 38 gaps, all 38 were re-checked: 10 fixed, 3 partially-fixed, 25 still-open, with 2 verdicts overturned by the confirmer (Circle of Doom and Five-Way lifecycle, both demoted from "fixed" to partially-fixed / still-open respectively).

| Gap | Sev | Verdict | Evidence |
|---|---|---|---|
| ExtraHitPoint Diamond Power live path bypasses the 4-slot cap | high | fixed | 6c8d8c9f; faithful helper `diamond_power_main_extra_hp_life_slot` (helmets.rs:4037, drifted from cited 4775) enforces `extra_hitpoint_slots < 4` (helmets.rs:4102), caller increments both counters only on award (helmets.rs:4804-4808); dead-code attr removed; live via lib.rs:2097 -> helmets.rs:6853-6857 |
| Wall of Death live path is an invented kill-all AoE; sweeping wall geometry unwired | high | fixed | e2c05a7b, a899ed1a, b7b4ca9b; 10-slot sweep/fade in `diamond_power_main_wall_of_death_slot` (helmets.rs:2135), per-slot collision via helmets.rs:4460 -> 4392, live at lib.rs:2097 -> helmets.rs:6753 -> 4519; no blanket delete remains (grep empty) |
| Snake Diamond Power 6-segment chain replaced by single forward projectile | high | fixed | e340be44, 98291605; full chain wired (init helmets.rs:1444, trails 1510, find_target 1562, octant 2513, steer 2590, head move 2717) live via lib.rs:2097 -> helmets.rs:6839-6851; no `% 8` cadence substitute remains |
| Five-Way Shot faithful init/main helpers dead; live path uses divergent world-space projectile | high | fixed | e3965ac; slot model in `diamond_power_main_five_way_shot_slot` (helmets.rs:2387) with edge bounce/shrink/camera-follow, live via lib.rs:2097 -> helmets.rs:6808; old `EnemyProjectile`/`<<16`/`lifetime:80` removed (grep clean) |
| Circle/Wall diamond objects not spawned; sprites and per-slot collision omitted | medium | fixed | 2cf4677c, 398161f6, a899ed1a, e2c05a7b; slots are real player fields (lib.rs:120-121, seeded helmets.rs:6947/6952), sprites compiled per frame (helmets.rs:4498/4548), per-slot rect collision via helmets.rs:4392; live at lib.rs:2097 |
| Snake live-path firing cadence and lifetime are invented | medium | fixed | e340be44, 98291605, e6a5910a, 81ab57fb; persistent 6-slot chain (helmets.rs:4646), lifetime = `Diamond_power_timer` (Mini 0x1E0 / Death 0x4B0), pruning matches loc_3FB20 (helmets.rs:3770-3781); `% 8`/`lifetime=90` gone |
| Maniaxe axe invented 30-frame lifetime cap | medium | fixed | c39bcd13; spawns `lifetime: 0` (helmets.rs:6353-6366), `is_maniaxe_axe` exempted from lifetime delete/decrement (projectiles.rs:2406/2425), bounds-driven delete (projectiles.rs:2876-2897); live via lib.rs:3040 |
| Maniaxe axe terrain delete over-broad (deletes on slopes) | medium | fixed | b2a21e50; `projectile_terrain_contact_loc_3345c` now returns solid-only for ManiaxeAxe (projectiles.rs:858-862), slopes fall through; live via projectiles.rs:2930-2932 |
| Maniaxe axe turnaround reverses `last_mut()` with no ManiaxeAxe filter | medium | fixed | e8f5517d; turnaround now uses `iter_mut().find(kind == ManiaxeAxe)` (helmets.rs:6374-6385), no longer newest projectile; live via lib.rs:3040 |
| Red Stealth airborne down-slash invented fast-fall threshold (`0x10000` vs dynamic `$FFFFFAAA`) | medium | fixed | 44648484; hardcoded const now only fallback, dynamic `red_stealth_jump_threshold_y_velocity` read (helmets.rs:5765-5769) and written at jump-init (lib.rs:2628); live reader lib.rs:3038 |
| Diamond Power object linked-list (alloc/remove/reset/init/clear) dead; live path uses flat timer | high | partially-fixed | a899ed1a, 6c8d8c9f; per-type live slot Vecs now wired (#206) modelling per-slot removal + invuln maintenance (helmets.rs:6734, lib.rs:2097). REMAINS: the 5 generic `$FFFFF2AC` linked-list helpers (helmets.rs:1008-1128) still carry `#[cfg_attr(not(test),allow(dead_code))]`, test-only callers (combat_tests.rs:3277-3527); lifetime is still a flat `diamond_power_frames` timer, no generic `$38` self-removal / `Diamond_power_active` SNE teardown; #228 still open |
| Circle of Doom live path is an invented kill-all AoE; geometric orbit unwired | high | partially-fixed | 2cf4677c, 398161f6; kill-all gone, 8-slot polar orbit + per-slot collision wired (helmets.rs:6943-6947 / 4475 / 4392), live at lib.rs:2097. REMAINS (confirmer overturned "fixed"): the invented 480-frame cap is load-bearing, not advisory — `diamond_power_frames=480` (helmets.rs:6942) force-clears the orbit ~113 frames early (helmets.rs:6862-6874); ASM ends only on radius depletion |
| Snake invulnerability set ad hoc in live path; full snake state untracked | low | partially-fixed | e340be44, 98291605, e6a5910a, 81ab57fb; Leg B stale — `diamond_power_snake_slots` is a live field (lib.rs:123) running the full faithful chain (helmets.rs:6839-6852 -> 4646-4744). REMAINS: Leg A — snake branch still forces `invulnerable_frames = max(1)` ad hoc (helmets.rs:6844-6845), which the ASM `DiamondPower_Main_Snake` never does |

Still-open gaps (remain authoritative in the original doc):

- Diamond Power per-frame `DiamondPower_Main_Execute` / `Main_Index` indexed dispatch omitted; live path still uses a hardcoded Rust `match` on `diamond_power_type`, no `diamond_power_id` field (helmets.rs:6744)
- Swift Mini-Snake (Micromax T2) distinct parameters collapsed into generic DeathSnake; `SNAKE_INIT_DATA_SWIFT_MINI` dead/test-only (helmets.rs:1426)
- Eyeclops beam modelled as a single point projectile, not the growing multi-segment screen-wide beam; faithful helpers dead/test-only (helmets.rs:6225, lib.rs:3039)
- Eyeclops invented `BEAM_SPEED`/`LIFETIME` with no ASM basis still drive the live path (constants.rs:721-722, helmets.rs:6231/6236-6240)
- Juggernaut 5-way arming state-machine flag fused into one-frame activation; `FiveWayShotReady` never modelled as a persistent flag (helmets.rs:6899)
- Five-Way Shot projectile lifecycle (timer-gated shrink + removal) unreachable live (confirmer overturned "fixed"): shrink branch gates on `diamond_power_timer < 0` but the live caller passes unsigned `diamond_power_frames` which never goes negative (helmets.rs:2434 / 4584); live removal is still a flat-80-frame mass clear (helmets.rs:6875)
- Eyeclops hard-beam 2-diamond cost spent at arm-time, not ROM render-time; counter semantics not driven by `sub_E49A`; faithful gate dead/test-only (helmets.rs:6250, level.rs:16917)
- Form change slope-bypass `loc_7AD6` reconstructed from slope-cell probe rather than `Character_Movement==6` (lib.rs:2844)
- Form change invented +1 cleanup frame appended to transform hibernate count (combat.rs:9123)
- Transform HP `$FFFFFA64` skip is call-site-implicit, not a modelled guard (combat.rs:143-158)
- `sub_DB22` faithful clear helper is `#[cfg(test)]`-gated; live path uses broader `helmet.reset()` (helmets.rs:4900, combat.rs:9214)
- Transform SFX per-helmet sound (`unk_7ED8`), `sfx_Replenish_Health`, and `sfx_Voice_bummer` all omitted (combat.rs:9053-9216)
- Music-speed selection (`unk_7ECC` / `MusicSpeed_Murderwall`) omitted (combat.rs:9125-9129)
- `clr.w ($FFFFFB70)` walk/animation accumulator not modelled or cleared at transform (combat.rs:9131-9138)
- Tier-2 snake dispatch collapses SwiftMiniSnake into DeathSnake (Micromax T2) (helmets.rs:949 / 4623-4627)
- CircleOfDoom duration is an invented 480f cap (ROM power is indefinite) (helmets.rs:957, 6942)
- Diamond Power `DiamondPower_CompileSprites` (`sub_3F798`) sprite-table compile not wired; helper dead/test-only (helmets.rs:1153)
- Rain non-tracking Y movement uses 32-bit add instead of ROM word-only update (projectiles.rs:2297)
- Rain dead duplicate cull path for `EnemyProjectileKind::DiamondPowerRain` in `step_projectiles`; no live code constructs that kind (projectiles.rs:2390)
- Rain tracking enemy target subpixel truncated before velocity computation (projectiles.rs:2272-2273)
- Five-Way Shot cost gate routed through generic `DiamondPower_Check` (5-vs-20 branch) instead of Juggernaut's dedicated init; 20-cost branch dead (helmets.rs:6899-6907)
- Eyeclops beam-render direction latch `$FFFFFABE` modelled only as instantaneous velocity sign, not a persistent render latch (helmets.rs:6236-6240)
- Maniaxe axe velocity reversal negates `x_vel`; ROM flips a direction flag (architectural) (helmets.rs:6383)
- Red Stealth sword/airborne hit detection uses invented AABB extents instead of ROM `collision_type` (helmets.rs:5816-5876, constants.rs:695-696/703)
- Maniaxe axe turnaround decrement off-by-one on throw frame (decrements before first move) (helmets.rs:6341-6373)

Pattern: the Diamond Power subsystem absorbed almost every fix — the #206 per-type slot rewrite (landed 2026-05-31, before the audit doc commit) wired Circle, Wall, Snake and Five-Way slot machinery live, so most "faithful helper dead / live path is invented kill-all" gaps closed; the residue is now concentrated in the still-unwired generic linked-list/dispatch architecture (`Main_Index`, `sub_3F5E8`, `CompileSprites`) and the Eyeclops/transform-SFX peripherals that no commit touched.

## weather-fidelity-audit.md (2026-05-31)

Of 33 gaps, all 33 were re-checked: 11 fixed, 6 partially-fixed, 16 still-open, with 0 overturned by the confirmer.

| Gap | Sev | Verdict | Evidence |
| --- | --- | --- | --- |
| `init_weather` dispatches only weather 1 (Lava); Storm/Storm+Hail/Unknown init omitted | high | partially-fixed | e0ecf788 wired weather 2/3 init; f1f00717 stores the loc_14D82 saved palette pointer live; 2026-06-16 follow-ups record the loc_14D1E/loc_14D32 pattern upload shape, wire loc_14CCE/loc_14CD8 art plus loc_14CEC/loc_14CF2 live palette copy, apply live `sub_6048` storm fade palette ops/`sub_14C0` blend, and record weather-4 Unknown loc_14DF6 art + loc_14E42 foreground-plane init shapes live. REMAINS: renderer consumption of storm/Unknown plane uploads. |
| `step_weather` — `sub_6034` dispatch live; full `sub_61CA` worker remains open | high | partially-fixed | e0ecf788/95733ca5/608e528a wired dispatch; 2026-06-16 storm follow-ups feed generated storm palettes into live `step_level_special_effects_sub_6048`, apply its palette ops to `Palette_Buffer_2/3`, and consume the effective `loc_6058/loc_605E` `$8000007F` mask through `sub_14C0`. REMAINS: renderer consumption of recorded storm plane upload and the full 5-slot hail worker |
| `step_weather` — `sub_6048` palette/thunder live; snow/hail worker side remains partial | high | partially-fixed | e0ecf788/95733ca5/608e528a; faithful step_level_special_effects_sub_6048 now called live; state machine/timing/thunder SFX live; 2026-06-16 follow-ups apply the generated storm palettes, emitted `palette_ops`, and `sub_14C0` blend live. REMAINS: `sub_61CA` hail/snow worker is still generic-projectile based, and renderer consumption of the recorded storm plane upload remains open |
| `step_weather` — weather type 2 now reaches `sub_6048` | high | fixed | e0ecf788/95733ca5/608e528a; step_weather (weather.rs:2344-2405) no longer early-returns on weather_type!=3; gates via dispatch_level_special_effects_sub_6034 (weather.rs:174-187) mapping weather 2 and 3 to sub_6048; thunder fires for weather 2 (weather.rs:2397-2398); live from lib.rs:3758 |
| Storm hail — faithful 5-slot worker `step_weather_worker_sub_61CA` is dead/test-only | high | partially-fixed | 6f2eb4c3/0df73ce5/0ea02251 fixed spawn legs — 5-slot cap (weather.rs:2348-2357), cadence $25-(rand&$F) (weather.rs:2417-2428), jitter Camera_X+rand+$28 (weather.rs:2426/2435-2440); REMAINS: step_weather_worker_sub_61CA (weather.rs:726) `#[allow(dead_code)]`, test-only; live path pushes generic EnemyProjectile::Hail (weather.rs:2449-2456), so splash (weather.rs:881-932), impact engine (weather.rs:940-949), Kid $28 push-timer all unreachable |
| Storm hail — terrain collision-map (`$FFFF4A04`) spawn-gate and ice/lava splash emission omitted | high | still-open | All three legs live only in dead step_weather_worker_sub_61CA (weather.rs:726-958, `#[allow(dead_code)]` weather.rs:725, reachable only from tests.rs:288); live step_weather (weather.rs:2344) spawns unconditionally; live Hail handling only off-camera deletes (projectiles.rs:2786-2797) |
| Storm hail — Kid hitbox `$28` push timer (`move.w #$28,$38(a5)`) not applied by live path | high | still-open | $28 write exists only in dead worker (weather.rs:878, scan 858-879), imported solely by tests.rs:288; live step_weather (weather.rs:2344) spawns generic Hail (weather.rs:2449-2456); projectiles.rs handles only movement/culling (2786-2797), no $38(a5) write |
| Lava geyser — entire rise/hold/collapse driver is an invented 4-phase ramp | high | fixed-candidate | 2026-06-15 follow-up: live step_lava_geysers now uses ASM loc_1484E/loc_1496C/loc_1498C row cadence (16px rows, `$48=2` rise, `$50=$1E` 6-frame hold cycles, 1-row collapse) and init converts `height>>4` rows to pixel height; covered by `lava_geyser_erupts_and_collapses` and `step_lava_geysers_phase_3_collapse_decrements_one_sprite_row`. Remaining lava-geyser script/render/palette gaps stay listed separately. |
| Lava geyser — invented timing constants (`ERUPT_SPEED=4px`, `HOLD=30`, `DORMANT=60`) | high | fixed-candidate | 2026-06-15 follow-up: deleted live `ERUPT_SPEED=4`/raw `HOLD=30` usage in favour of named ASM domains (`ROW_HEIGHT=16`, rise `$48=2`, hold `$50=$1E` through `$48=6`, collapse `$48=1`) plus existing per-geyser loc_14772 hibernate. `$3C` disable remains under the script-walk sentinel gap. |
| RotatePalette_Lava_Water — per-frame lava/water palette cycle never runs | high | fixed | c20b70f6 added foreground_palette_for_scene (scene.rs:2049-2079) rotating 4 foreground words on ASM loc_1223A cadence (Mountain 5 / Cave 8); bd3dc877 phase-aligned; called per-frame via scene.rs:3223/3261 (invoked scene.rs:3651-3654); renderer-layer reimplementation (kid-core helpers level.rs:9264/9285 remain test-only) but observable deviation gone |
| `step_weather` — invented hail cadence/spawn physics diverges from `sub_6048`/`sub_61CA` | medium | fixed | 95733ca5/0df73ce5/6f2eb4c3/0ea02251 + others; Frame_Counter>$873 gate (weather.rs:2409/1198), $258/$4B0 toggle (weather.rs:1199-1206), Camera_X+rand+$28 spawn-X (weather.rs:2435-2440), cooldown $25-(rand&$F) (weather.rs:2427), 5-slot cap (weather.rs:2349-2356); no `level_timer.seconds`/`*7`/`%320` remain; live lib.rs:3758 |
| `step_weather` — wrong inactive hail period (600 vs ROM `0x4B0`=1200 frames) | medium | fixed | 95733ca5 deleted inline reseed, routed through step_level_special_effects_sub_6048 (weather.rs:2365); helper (weather.rs:1199-1206) reseeds `if hail_active {0x0258} else {0x04B0}` — correct asymmetric ROM values; no flat-600 reseed in live path |
| `step_weather` — timing uses `level_timer.seconds` instead of `Frame_Counter` gates (`0x834`/`0x873`) | medium | fixed | e0ecf788/95733ca5; live step_weather passes `frame_counter: self.frame_index` (weather.rs:2367) into sub_6048 gating on `frame_counter > 0x0873` (weather.rs:1198); live worker gate weather.rs:2409; $834 collapsed into dominating $873; `level_timer.seconds` gone |
| Storm hail — invented spawn cadence, period toggle, and PRNG with no ASM basis | medium | fixed | e0ecf788/0df73ce5/6f2eb4c3; 600-toggle replaced by loc_619C $258/$4B0 (weather.rs:1198-1221), fixed-30 by $FFFFFB3C $25-(rand&$F) (weather.rs:2420-2428), frame-hash X by Camera_X+rand+$28 (weather.rs:2435-2440); 5-slot cap (weather.rs:2432-2434); genuine ROM PRNG (types.rs:2062); no 600/`*7`/`%320`/`+13` left |
| Lava geyser — USM2 `$8000` small-fireball table entry dropped; fireball hardcoded | medium | fixed | bc87d764 added third table entry (0x346,0x312,0x8000) at weather.rs:2152-2156, decodes sign-bit `if h & 0x8000 != 0` (weather.rs:2502), routes through initialise_lava_fireball_state_loc_1499C (weather.rs:2503); live via init_weather (weather.rs:2475) <- boot.rs/level.rs:17972 |
| Storm init — entire `Init_SpecialEffect_Storm` omitted; faithful helpers dead/test-only | high | partially-fixed | e0ecf788/95733ca5/608e528a; init_weather now has weather 2/3 branch seeding storm state on WeatherState; f1f00717 stores loc_14D82 `$FFFFFADE`; 2026-06-16 follow-ups record loc_14D1E/loc_14D32 pattern table, `$0380_0380`, `$6000` command, DBF write count, wire loc_14CCE/loc_14CD8 art plus loc_14CEC/loc_14CF2 live palette copy, and add the weather-4 Unknown loc_14DF6/loc_14E42 live init records. REMAINS: renderer consumption of the storm name-table/palette and Unknown plane uploads |
| Storm init — `FB3C/FB3D/FAD6/FADA/FADC/FB52/FB54` seed words never written at runtime | high | fixed | e0ecf788; WeatherState carries five seed fields + FB52/FB54 (types.rs:684-692); live init_weather seeds for weather 2/3 (weather.rs:2461-2487); consumed every frame via sub_6048 (weather.rs:2365-2396) and inline cooldown/spawn (weather.rs:2420-2458); seeds match ASM kid.asm:28439-28447 |
| `step_weather` — invented hail spawn cadence and spawn-X (fixed 30 / frame-index hash) | low | fixed | 0df73ce5 replaced fixed-30 and `(frame_index*7+13)%320` inside live step_weather; cooldown $25-(rand&$F) (weather.rs:2427), spawn_x Camera_X+rand+$28 (weather.rs:2436-2440); old pattern gone; live lib.rs:3758 |
| Lava geyser — inter-cycle restart uses flat `DORMANT` instead of re-seeded per-geyser startup hibernate | low | fixed | 09211cf7 added LavaGeyser.startup_delay_frames (types.rs:1760), seeded from lava_geyser_initial_delay_frames_loc_14772 (weather.rs:2222), collapse restart now `geyser.timer = geyser.startup_delay_frames` (weather.rs:2572); flat LAVA_GEYSER_DORMANT_FRAMES=60 deleted (0 hits); live lib.rs:3759 |
| Lava fireball — faithful script-decode/init helpers dead; live path uses parallel static table | low | fixed | bc87d764 fixed SPAWN leg — hardcoded spawn replaced by table-driven initialise_lava_fireball_state_loc_1499C (weather.rs:2502-2511), now live; 2026-06-16 follow-up fixed SCRIPT leg — live runner now stores raw `unk_146CC` words, decodes triplets through decode_lava_fireball_script_step_loc_149E4, maps sprite script words to sub_14AB0 hitbox buckets, and advances Y through lava_fireball_loop_resume_loc_149FC |

### Still-open gaps

- Storm hail — terrain collision-map (`$FFFF4A04`) spawn-gate and ice/lava splash emission entirely omitted
- Storm hail — Kid hitbox `$28` push timer (`move.w #$28,$38(a5)`) not applied by live path
- Animation_Geyser — geyser palette cycle (`$28..$2E`) omitted from live path (helper unwired) — resolved candidate 2026-06-16 by `2cd804c5`, now ticked from live `step_lava_geysers`
- Post-load palette-object spawn (`loc_11F8C/loc_11F96`) never executed — objects are never instantiated
- Island shore — entire `loc_1202A` object routine never spawned or stepped at runtime
- Lava geyser — palette seed, `Animation_Geyser` rotator, native-player loc_14BE8 renderer consumption, and the `ArtComp_983D2_Lava` -> `$5F60` upload trace are live as of `4ad8a9a5`/`2cd804c5`/`8986d524` plus the 2026-06-16 upload-trace follow-up
- Lava geyser — script-walk intro animation + `$FFFF`/`$FFFE` sentinels omitted entirely
- Island shore — one-shot shore tile-art DMA upload at `loc_1202A` is not modelled by any helper
- Storm init — `$FFFFFADE` saved storm palette pointer now stored live as `Pal_7B85C`; broader storm palette copy/fade consumption still open
- Storm init — `$380`-tinted pattern-block VRAM upload (`loc_14D32`) absent from runtime
- Lava geyser — lava art `DecompressToVRAM` upload now modelled as a trace event; no general live VDP VRAM backend
- Lava geyser — top/middle/bottom sprite-row culling windows (`loc_14874/loc_148F8`) reconstructed in renderer, faithful core helpers dead
- Island shore — `off_1220E` frame-source link table not modelled (phase index only)
- Murderwall — init writes full 16.16 X but ROM stores only the integer high word
- Murderwall — palette copy (`loc_140EE`) helper is dead — never wired into runtime palette load

The storm/hail subsystem absorbed nearly all the progress — the 2026-06-01 trio e0ecf788/95733ca5/608e528a plus 0df73ce5/6f2eb4c3/0ea02251 drove almost every fixed and partially-fixed verdict by wiring the live `step_weather` dispatch, ROM hail cadence and seed words, while the lava-geyser palette/art-upload and Island-shore object-spawn families remain almost wholly dead and test-only.
