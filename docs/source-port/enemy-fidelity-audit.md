<!-- Generated 2026-05-29 by the `enemy-fidelity-audit` workflow (28 read-only Explore auditors, one per enemy family; every high/medium gap adversarially verified by an independent refuter before inclusion). This is a reviewable research artefact, not a gate. Each gap cites the owning ASM label and the native `file:line` searched. Re-run the workflow to refresh. -->
<!-- Staleness-checked 2026-06-10: of 67 gaps, 33 fixed, 9 partially fixed, 25 still open — verdicts and evidence in fidelity-audit-staleness-2026-06-10.md. Still-open entries below remain authoritative; fixed entries are superseded. -->

# Enemy Fidelity Audit

Adversarially-verified per-family audit of the Kid Chameleon source-port. The recurring failure mode across the codebase: ASM-faithful pure helpers exist and are unit-tested, but carry `#[cfg_attr(not(test), allow(dead_code))]` and are never wired into the live step/combat path. The runtime instead uses generic pacers, generic damage paths, or invented physics.

## Coverage summary

| Family | Coverage | Confirmed gaps |
|---|---|---|
| Archer (0x07) | partial | 2 |
| Armadillo (0x04) | partial | 3 |
| Big_Hopping_Skull (0x1D) | partial | 2 |
| Cloud (0x0E) | partial | 1 |
| Crab (0x09) | partial | 2 |
| Diamond (0x01) | partial | 2 |
| Dragon walking (0x0D) | partial | 1 |
| Dragon_Flying (0x0C) | partial | 1 |
| Driller (0x1A) | partial | 3 |
| Drips (0x16) | full | 0 |
| Emo_Rock (0x1B) | partial | 4 |
| Fireball (0x19) | partial | 1 |
| Fire_Demon (0x00) | full | 0 |
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
Fixed candidate 2026-06-20: queued live Diamond collision words now route through the Diamond-specific ASM damage tables. High-HP Diamonds consume `colid_hurt` through `loc_3ADA4` -> `loc_3ADFA`, decrement HP, and hibernate through native damage flash as `stru_3ACBE`; low-HP Diamonds consume `loc_3AEC4` hurt/death words, including the `loc_3AEE6` waited death animation. Regressions: `diamond_high_hp_pending_hurt_collision_enters_loc_3adfa`, `diamond_low_hp_pending_terminal_hurt_collision_enters_loc_3aee6`.

**Diamond — Berzerker side-launch not plumbed** (`loc_3AE84 / loc_3AF24`)
Fixed candidate 2026-06-20: queued Kid-side collision words now reach the high-HP `loc_3AE84` and low-HP `loc_3AF24` Berzerker gates. Charging side contact bypasses HP damage, seeds `y_vel=-$48000`, and enters the live `loc_3AEB2`/`loc_3AF50` airborne gravity loops. Regressions: `diamond_high_hp_pending_side_collision_enters_loc_3ae84_launch`, `diamond_low_hp_pending_side_collision_enters_loc_3af24_launch`.

**Armadillo — damage recoil/invulnerability window not applied** (`loc_3C848`)
Fixed candidate 2026-06-20: friendly-projectile `colid_hurt` and Kid-above stomp damage now both route through `armadillo_damage_response_loc_3c848`. Positive `$42(a3)` preserves HP, nonfatal damage seeds `$42=$FF88` and restores saved walk speed, and terminal damage enters the Armadillo death tail. Regressions: `maniaxe_axe_colid_hurt_respects_armadillo_recoil_window_loc_3c848`, `maniaxe_axe_colid_hurt_seeds_armadillo_recoil_loc_3c848`, `armadillo_stomp_respects_recoil_window_loc_3c848`, `armadillo_stomp_seeds_recoil_window_loc_3c848`.

**Armadillo — Berzerker curl-and-launch reaction not simulated** (`loc_3C898 / loc_3C8CC`)
Fixed candidate 2026-06-20: charging-Berzerker side contact with no positive recoil word now enters the live `loc_3C8CC` launch loop instead of HP damage. Regression: `berzerker_charge_launches_armadillo_instead_of_damaging_loc_3c898`.

**Armadillo — spawned-child fall-then-die death sequence not simulated** (`loc_3C93C / loc_3C954`)
Fixed candidate 2026-06-20: `$5C(a5)` spawned children now enter the live `loc_3C93C` fall state on terminal damage, wait in `loc_3C954` until floor contact, then start `stru_3C69A` death animation. Regressions: `armadillo_spawned_child_terminal_damage_falls_before_death_loc_3c87e`, `armadillo_spawned_child_floor_contact_starts_death_tail_loc_3c954`.

**Big_Hopping_Skull — patrol x-bounds and relaunch reset never run** (`loc_36770 / loc_36782 / loc_36798 / loc_366D0`)
Fixed candidate 2026-06-20: live activation now packs the segment corridor from `loc_364A0` into `dragon_anchor_y`, `step_active_enemies` unpacks those `$3E/$40` words for Big Hopping Skull, and `loc_366D0` relaunch runs `loc_36770` before returning to the live loop. Regressions: `big_hopping_skull_live_floor_contact_timer_relaunches_like_loc_366d0`, `big_hopping_skull_live_relaunch_applies_patrol_bounds_like_loc_36770`, `big_hopping_skull_activation_carries_patrol_bounds_to_live_relaunch_loc_364a0_36770`.

**Fire_Demon — walking fire-trail emission never happens** (`loc_3A542 / loc_3A564 / loc_3A63C`)
Fixed candidate 2026-06-20: live `refresh_active_roster` now calls `spawn_fire_demon_walking_trails_loc_3a542` after the active enemy step, emits typed `FireDemonTrailChild` workers through `loc_3A63C`, respects the seven-child cap, rejects unsupported floor probes, and keeps the raw level-2 saved-walk cadence. Regressions: `fire_demon_refresh_active_roster_spawns_walking_trail_loc_3a542_3a63c`, `fire_demon_walking_trail_spawns_typed_child_loc_3a542_3a63c`, `fire_demon_walking_trail_respects_seven_child_cap_loc_3a564`, `fire_demon_walking_trail_skips_unsupported_floor_loc_3a63c`, `fire_demon_walking_trail_uses_raw_level_two_saved_walk_cadence_loc_3a542`.

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
Fixed candidate slices 2026-06-20: post-floor stopped wait now follows `loc_3F000`/`loc_3F090`/`loc_3F09E`/`loc_3F112`; a facing mismatch flips and restarts the 21-frame wait, and matched facing waits 11 frames before relaunch. Slope terrain contact now routes through `loc_3EAD8`/`loc_3ED6C`, including the `loc_3F22A` air-damage slope resume. Queued parent collision words now route through the grounded wait/pause handlers and airborne `loc_3F1B8`: side/below contacts allocate `sub_3F3F0`, grounded hurt clears motion through the hit animation path, airborne hurt enters `loc_3F1E6`, and charging-Berzerker side contact enters `loc_3F262`. Representation note 2026-06-20: native still compresses distinct slope/windup coroutine labels into `grounded_frames` plus a tagged `wait_timer`, but the observable behaviour is covered: exact 21/11-frame waits, signed Kid-X facing choices, boosted relaunch velocities, slope-up/down contact waits, air-damage slope resume, grounded/airborne collision routes, and delayed scream worker. Reopen only with a sprite/coroutine-label export requirement or a concrete oracle route showing behavioural drift. Regressions: `mini_hopping_skull_restarts_wait_after_facing_flip_loc_3f090`, `mini_hopping_skull_does_not_edge_reverse_before_launch_loc_3f090`, `mini_hopping_skull_slopeup_nonpositive_x_enters_left_wait_loc_3ead8_3ec32`, `mini_hopping_skull_slopedown_negative_x_enters_left_wait_loc_3ed6c`, `mini_hopping_skull_air_damage_slope_landing_resumes_terrain_contact_loc_3f22a_3ead8`, `mini_hopping_skull_pending_ground_side_collision_queues_scream_worker_loc_3f03c`, `mini_hopping_skull_pending_ground_hurt_collision_uses_ground_hit_path_loc_3f05c`, `mini_hopping_skull_pending_air_hurt_collision_enters_loc_3f1e6_landing_loop`, `mini_hopping_skull_pending_air_side_collision_enters_berzerker_launch_loc_3f262`. Focused evidence: `cargo test -p wildside-core mini_hopping_skull_ -- --nocapture` passes 91 tests.

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
Fixed candidate 2026-06-20: native final damage enters `loc_3A96C` charge instead of generic death, pending child-negative `$38(a2)<0` enters charge without HP damage, parent-negative collision enters `loc_3A9D2` destroy prep, parent/child `$800` continues charge, and shoot-time collision dispatch covers nonfatal/final projectile hits plus child `$2C` and Berzerker side-contact routes through `loc_3AAD4`/`loc_3AB04`/`loc_3AB56`/`loc_3AAFA`. Regressions: `robot_live_final_hit_enters_charge_instead_of_death_loc_3a8fc_3a96c`, `robot_child_negative_collision_enters_loc_3a96c_charge_without_hp_damage`, `robot_parent_negative_collision_enters_loc_3a9d2_destroy_not_charge`, `robot_mid_shoot_nonfatal_projectile_hit_resumes_walk_loc_3aad4_3aac4`, `robot_mid_shoot_final_projectile_hit_enters_charge_loc_3ab56_3a96c`, `robot_mid_shoot_child_kidabove_collision_hits_parent_loc_3aad4_3ab04`, `robot_mid_shoot_child_side_without_berzerker_restores_walk_loc_3aafa_3aa9e`, `berzerker_charge_robot_mid_shoot_head_side_destroys_parent_loc_3aafa_3a9d2`.

**Scorpion — sting attack never produced** (`loc_3DDEE / loc_3DE40`)
Fixed candidate: live tracking now arms `scorpion_sting_frames` through `loc_3DDEE`, stops `$50(a5)`, exposes the linked tail with the `stru_3DB8E` contact phase/`object_meta` bit-15 semantics, damages Kid through the tail contact box, and recovers through `loc_3DE40` by clearing tail collision and restoring base speed/meta. Regressions: `step_scorpion_tracking_arms_sting_tail_loc_3ddee`, `scorpion_sting_tail_contact_damages_player_loc_3ddee`, `scorpion_sting_tail_contact_uses_rom_collision_size_rows_loc_1bce`, `berzerker_charge_loc_b692_reflects_player_x_velocity_once_for_scorpion_sting_bit_15`, `scorpion_sting_recovery_matches_loc_3de40_reset_side_effects`.

**Scorpion — Berzerker-charge upward launch not wired** (`loc_3DD0E`)
Fixed candidate: pending side collisions now route through `loc_3DD0E`; charging Berzerker contact before the sting sprite gate preserves HP, disables ground motion, seeds `y_vel=$FFFB8000`, and loops through `loc_3DD4A` gravity instead of generic damage. Regressions: `berzerker_charge_launches_scorpion_instead_of_damaging_loc_3dd0e`, `scorpion_pending_side_collision_enters_loc_3dd0e_berzerker_launch`, `scorpion_berzerker_airborne_loop_loc_3dd4a_applies_gravity_and_repeats`.

**Spinning_Twins — orbit form does not split into two walking children + falling visual** (`loc_3B93C`)
Fixed candidate: orbit contact now marks the `loc_3B93C` split sentinel, and the runtime materialisation pass replaces the original orbit object with the two `loc_3BA74` walking children plus the `loc_3B9B0` falling visual. The children start at +/-$20 px with `$FFFC8000`/`$38000` X velocities and `-$10000` Y velocity; the visual starts at parent X-$13/Y-$40 and uses the `dword_3BA60` drift cadence. Regressions: `spinning_twins_runtime_drop_materializes_two_children_and_visual_loc_3b93c`, `spinning_twins_drop_transition_loc_3b93c_splits_orbit_object`, `spinning_twins_first_walking_child_init_loc_3bb0c_launches_left`, `spinning_twins_second_walking_child_init_loc_3bb06_offsets_right`, `spinning_twins_falling_visual_init_loc_3b9b0_offsets_parent_position`, `spinning_twins_falling_visual_step_loc_3ba3c_moves_and_uses_table_cadence`.

**Spinning_Twins — walking-child launch/drag/settle physics not applied** (`loc_3BB2C`)
Fixed candidate: live walking children run the `loc_3BB2C` motion order, applying current X/Y motion before `$2800` Y acceleration, facing-dependent `$1200` X drag, the asymmetric `loc_3BB5A` settle gate, floor-contact Y clear, and `loc_3BB94` settle into the shared walker. Settled collision/death routes also cover `loc_3BBC4`/`loc_3BC34`/`loc_3BC78`/`loc_3BCA6`, including partner signalling. Regressions: `spinning_twins_live_walking_child_applies_x_motion_and_drag_loc_3bb2c_3bb52`, `spinning_twins_walking_velocity_gate_loc_3bb5a_settles_only_inside_asymmetric_band`, `spinning_twins_walking_collision_dispatch_loc_3bb6e_clears_and_routes_contacts`, `spinning_twins_floor_landing_loc_3bb8a_clears_collision_and_y_velocity`, `spinning_twins_settle_entry_loc_3bb94_clears_motion_and_resumes_walker`, `spinning_twins_nonfatal_hurt_hibernates_through_stru_3b836_loc_3bc34`, `spinning_twins_deleting_one_walking_child_marks_partner_loc_3bca6`.

**Spinning_Twins — multi-hit HP damage/recovery and death state machine not reproduced** (`loc_3BC34`)
Fixed candidate: queued settled contact words now dispatch through `loc_3BBC4` before generic damage. `colid_hurt`/`colid_kidabove` decrement `$44` through `loc_3BC34`, nonterminal hits hibernate on `stru_3B836`, terminal hits enter the waited `loc_3BC56` death animation, and `loc_3BC78` status-table cleanup remains deferred until that animation finishes. Regressions: `spinning_twins_pending_hurt_collision_enters_loc_3bbc4_3bc34`, `spinning_twins_pending_terminal_hurt_waits_for_death_tail_loc_3bbc4_3bc56`, plus existing `spinning_twins_nonfatal_hurt_hibernates_through_stru_3b836_loc_3bc34`, `spinning_twins_finished_death_clears_positive_status_slot_loc_3bc78`, `spinning_twins_deleting_one_walking_child_marks_partner_loc_3bca6`.

**Orca — in-lunge Kid contact (hurt/Kid-above/side) not dispatched** (`loc_3D35A`)
Fixed candidate: queued lunge contact now dispatches through `loc_3D35A`. Hurt/Kid-above contacts enter the `loc_3D472` HP gate, nonterminal hits bounce downward through `loc_3D47A`, and side contacts go through the Berzerker gate instead of being ignored by terrain-only lunge probes. Regressions: `orca_pending_lunge_hurt_bounces_down_loc_3d35a_3d472`, `orca_pending_lunge_side_contact_enters_berzerker_launch_loc_3d4de_3d27e`.

**Orca — ceiling-hit HP decrement counter ($44) not modeled** (`loc_3D472`)
Fixed candidate: the row's "ceiling-hit" wording was stale (`colid_ceiling` routes to `loc_3D47A` without decrementing HP), but the real `loc_3D472` lunge hurt/Kid-above HP gate is now live. Terminal lunge damage starts `OrcaState::FatalFall` via `loc_3D48A`, falls through `loc_3D496`, and only starts waited `stru_3D11E` death on floor/slope contact at `loc_3D4BC`. Regression: `orca_pending_lunge_terminal_hurt_falls_until_floor_loc_3d48a_3d4bc`.

**Tank — body-vs-mouth/turret vulnerability gating not enforced** (`loc_3B464 / loc_3B6C0`)
Fixed candidate: both RockTank variants now expose the linked mouth/turret child as the stomp candidate instead of the parent body. Plain Tank already queued child hits through `loc_3B464`; Tank_Shooting now mirrors `loc_3B6C0`, so Kid-above contact writes `$2C` to the linked turret child and the parent consumes it on the later object pass. Regressions: existing `tank_child_stomp_hit_latches_contact_damage_through_loc_3b49a_hibernate`, new `tank_shooting_stomp_candidate_is_turret_child_loc_3b6c0`.

**Tank — negative parent-collision instant-death route not wired** (`loc_3B41C→loc_3B480 / loc_3B6C0→loc_3B6E6`)
Fixed candidate: queued parent negative collision words now bypass HP decrement for both RockTank variants and enter the waited child-hide death scripts. Plain Tank is covered by `tank_negative_parent_collision_enters_loc_3b480_death_before_hp_damage`; Tank_Shooting is covered by `tank_shooting_negative_parent_collision_enters_loc_3b6e6_before_hp_damage`.

**Tar_Monster — 5-projectile burst fires on every non-fatal hit instead of on death** (`loc_323E6 / loc_323C0`)
Fixed candidate: queued Tar Monster collision now follows `loc_3237A`/`loc_32386`/`loc_323C0`. Nonterminal hurt decrements HP and runs the hurt response without allocating `loc_32468`; terminal hurt/signed-negative/charging side routes enter `loc_323E6`, play the death-side `loc_32468` shoot cue, and spawn the `loc_324F8` five-child burst at the copied source position minus `$10` y before the waited death animation. Regressions: `tar_monster_nonfatal_hurt_does_not_spawn_burst_loc_323c0_323d6`, `tar_monster_terminal_hurt_spawns_death_burst_loc_323e6_324f8`.

**UFO — two-phase hurt ($47) absent: first hit should spray debris and survive** (`Enemy0F_UFO_Hurt / loc_35150 / loc_351EE / loc_354DA`)
Fixed candidate: first hit now routes through `loc_351EE`/`loc_354DA`, spawning the ten debris children from `unk_351C4` before the saucer survives through `loc_35228`; final damage remains on the waited `loc_35174`/`loc_35180` parent death animation. Regressions: `ufo_first_hurt_projectile_hit_spawns_loc_351ee_debris_before_loc_35228`, `ufo_first_hurt_survives_and_primes_loc_35228`, `ufo_death_uses_loc_35174_parent_animation_before_delete`.

### Medium severity

**Archer — fire cadence uses arrow re-aim interval instead of pose-table delays** (`loc_341BE / loc_34248`)
Fixed candidate: live Archer firing reloads `ARCHER_SHOOTING_CYCLE_DELAY_FRAMES_LOC_341BE` from the shooting-table delays (`$64+1`, `$06+1`, `$0C+1` = `$79`) while fired arrows still use `arrow_accuracy` only for `loc_342D6` re-aim cadence. Regressions: `archer_shooting_cycle_delay_loc_341be_uses_pose_table_not_arrow_accuracy`, `archer_live_step_waits_pose_table_delay_between_arrows_loc_341be`.

**Archer — shot-count gate counts the shared projectile pool, not arrows** (`loc_34248`)
Fixed candidate: live Archer now represents `Number_of_Arrows` as the count of live `EnemyProjectileKind::ArcherArrow` projectiles and blocks only when that count is already 2; the shared pool cap remains a secondary allocation guard. Regression: `archer_loc_34248_caps_only_live_archer_arrows`.

**Big_Hopping_Skull — per-hit SFX cadence + kid-upward-bounce contact bug not driven** (`loc_365D8 / loc_365F2 / loc_36616 / loc_3664E`)
Fixed candidate: accepted Big Hopping Skull hurt/Kid-above contacts now apply the `loc_365D8` Kid y-velocity high-word write (`$FFFD`) before the existing nonfatal `stru_367CC` hibernate and delayed `loc_365F2`/`loc_36616` groan/voice cues; terminal damage still enters `loc_3664E`. Regressions: `big_hopping_skull_stomp_uses_loc_365d8_kid_bounce_velocity`, `big_hopping_skull_pending_kidabove_applies_loc_365d8_kid_bounce`, `big_hopping_skull_nonfatal_damage_enters_stru_367cc_hibernate`, `big_hopping_skull_hit_animation_queues_stru_367cc_sounds_like_loc_365f2`, `maniaxe_axe_colid_hurt_queues_big_hopping_skull_death_sfx_loc_3664e`.

**Big_Hopping_Skull — death respawn marker (EnemyStatus_Table[d0]=$2168) not written** (`loc_3664E / loc_36664 / loc_366A4 / loc_366B2`)
Fixed candidate: Big Hopping Skull death now waits through `stru_367FA`, then positive `$42(a5)` writes `$2168` to `EnemyStatus_Table` before `loc_366B2` deletion and the killed-enemy bit. Regression: `big_hopping_skull_death_waits_for_stru_367fa_animation_before_delete`.

**Cloud — lightning projectile has fixed 31-frame lifetime instead of collision-bound persistence** (`loc_35DF0 / loc_35E8A`)
Fixed candidate: Cloud lightning workers now spawn with `lifetime = 0`, are exempt from generic lifetime expiry/decrement, and keep moving until the terrain/collision route deletes them, matching `loc_35E8A`'s collision/$19-bound loop instead of the old 31-frame cap. Regressions: `cloud_attack_projectile_loop_loc_35e8a_waits_until_collision_or_animation_flag`, `cloud_runtime_loc_35cd8_latches_then_spawns_diagonal_lightning`, `cloud_lightning_loc_35e8a_persists_without_generic_lifetime_expiry`.

**Dragon (walking) — breath flame deleted on first Kid contact instead of persisting its animation** (`loc_3D02A / loc_3D082`)
Fixed candidate: walking Dragon breath now spawns as a typed stationary `DragonWalkingBreath` child at the `loc_3D070` facing-side offset, remains trace/object-count visible as the `loc_3D02A` child worker, and player overlap hurts Kid without consuming the child; deletion stays owned by the parent `$5B` / child `$18` `loc_3D082` poll. Regressions: `dragon_walking_fires_breath_in_facing_direction`, `dragon_walking_breath_hits_player_without_consuming_child_loc_3d082`, `dragon_walking_breath_child_poll_matches_loc_3d082`, `dragon_walking_breath_projectile_projects_to_trace_loc_3d02a`.

**Dragon_Flying — descent fireball uses a generic projectile instead of windup/travel/impact lifecycle** (`loc_34BAC / loc_34C26 / loc_34C3A / loc_34C4A`)
Fixed candidate: Dragon_Flying descent fire now spawns as typed `DragonFlyingDescentFire`, carries the `loc_34BE6` parent-word setup and mouth offset, ignores generic lifetime expiry, runs the `loc_34C26` startup DBF into `loc_34C3A` travel/off-screen/collision polling, and enters the two-step `loc_34C4A` cleanup before deletion. Regressions: `dragon_flyer_descent_fire_uses_live_horizontal_velocity_and_downward_arc`, `dragon_flying_loc_34be6_sets_descent_fire_from_parent_words`, `dragon_flying_descent_fire_loc_34c26_ignores_generic_lifetime`, `dragon_flying_descent_fire_loc_34c26_enters_travel_after_startup`, `dragon_flying_descent_fire_loc_34c4a_keeps_impact_cleanup_then_deletes`.

**Driller — probe offset, wall-collision gate, and Ice bullet dispatch closed** (`loc_3BE78` / `loc_3BE8E` / `loc_3BEF2`)
The earlier audit overstated the remaining gap. `$38(a4)` is `collision_type`, not an animation timer: ROM gates on `colid_leftwall`/`colid_rightwall`, then clears the child collision word before dispatching Smash/Prize or Ice block destruction. Native now probes the linked drill child at parent±$18, requires a Solid front-wall cell before drilling, preserves the Smash/Ice action discriminant, and spawns the three Ice shooter bullets. Regressions: `loc_3be78_driller_probes_right_child_x_not_parent_collision_width`, `loc_3bef2_driller_preserves_ice_block_dispatch_for_runtime_bullets`, `loc_3bef2_driller_ice_block_spawns_three_shooter_bullets_when_target_open`, `loc_3be8e_driller_requires_front_wall_collision_before_drilling`.

**Fireball — fire-trail child objects never spawned** (`Enemy19_Fireball_BounceFloorCeiling / loc_32CE4 / loc_330D0 / loc_33128`)
Fixed candidate: Fireball bounce/slope paths now spawn typed `FireballTrail` children from `loc_32CE4`/`loc_33128` and `loc_330D0`, enforce the ROM five-child cap, and keep trail children hostile without consuming them on Kid overlap while `loc_33110` owns deletion. Regressions: `fireball_floor_bounce_spawns_trail_child_loc_32ce4_33128`, `fireball_floor_bounce_respects_five_trail_cap_loc_32ce4`, `fireball_slope_up_spawns_trail_child_loc_330d0`, `fireball_trail_hits_player_without_consuming_child_loc_33110`.

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
Fixed candidate 2026-06-20: live walking now applies `loc_3E03C`/`loc_3E054`, holds `$4E(a5)` inside the wider band, clears it through `loc_3E066`, and sets it on charge exits through `loc_3E48E`/`loc_3E642`. Live charge now runs `$42(a3)` through `loc_3E434`, keeps `$42==$0A` on the direct movement path, increments negative wait states toward zero, branches state 0 through the ±$60 `loc_3E450`/`loc_3E604` alert gate before X commit, and routes positive alert states through the ±$70 `loc_3E476`/`loc_3E62A` release gate. Regressions: `lion_walking_tracks_far_kid_before_timer_tick_loc_3e03c_3e074`, `lion_walking_tracking_latch_holds_until_outer_band_exit_loc_3e054_3e066`, `lion_charge_window_gate_matches_loc_3e434`, `lion_charge_state_zero_enters_right_alert_before_x_commit_loc_3e450_3e4ce`.

**Mini_Hopping_Skull — post-landing facing-flip re-orientation pause not reproduced** (`loc_3F000 / loc_3F074 / loc_3F090 / loc_3F09E`)
Fixed candidate 2026-06-20: live `step_hopper` now preserves the stopped wait until `loc_3F074`, flips toward Kid through `loc_3F090` only after expiry, restarts `loc_3F000` on mismatch, and uses an 11-frame `loc_3F09E` matched-facing pause before `loc_3F112` relaunch. Regressions: `mini_hopping_skull_restarts_wait_after_facing_flip_loc_3f090`, `mini_hopping_skull_does_not_edge_reverse_before_launch_loc_3f090`.

**Ninja — jump-attack airborne loop omits ceiling/kid-above branches** (`loc_3DA0C / loc_3DAB0 / loc_3DA98`)
Fixed candidate 2026-06-20: the stale row is covered by live jump-attack collision dispatch through `loc_3DA0C`; ceiling contact enters `loc_3DAB0`, Kid-above contact enters `loc_3DA98`/`loc_3DABC`, and side contact reaches the `loc_3DB0A` launch path. Follow-up `$5C(a5)` airborne side-contact routing through `loc_3D878`/`loc_3D8BC` is also covered. Regressions: `ninja_jump_attack_ceiling_contact_negates_y_velocity_loc_3da0c_3dab0`, `ninja_jump_attack_kid_above_bounces_and_decrements_loc_3da0c_3da98`, `ninja_jump_attack_kid_above_final_hit_enters_landing_loop_loc_3dabc`, `ninja_pending_berzerker_side_collision_enters_launch_loc_3d698_3db0a`, `ninja_airborne_berzerker_side_contact_enters_knockdown_loc_3d878_3d8bc`.

**Ninja — far-Kid facing turn during pacing not applied** (`loc_3D92A / loc_3D978 / loc_3D98A`)
Fixed candidate 2026-06-20: live `step_ninja` now applies the `loc_3D92A` far-Kid facing decision before pacing, turning right through `loc_3D978` and left through `loc_3D98A` when Kid is outside the inclusive +/-$80 px band. Regressions: `ninja_far_kid_turns_right_during_pacing_loc_3d92a_3d978`, `ninja_far_kid_turns_left_during_pacing_loc_3d92a_3d98a`.

**Orca — wall-pinned lunge X-drift handled, but side-collision Berzerker re-lunge absent** (`loc_3D4DE`)
Fixed candidate: pending lunge side contact now follows `loc_3D4DE`; non-charging contact falls back to the lunge gravity path, while charging Berzerker contact routes through `loc_3D21E`/`loc_3D27E`, preserves HP, zeros X velocity, seeds `y_vel=$FFFB8000`, and enters the `loc_3D2AA` airborne loop. Regression: `orca_pending_lunge_side_contact_enters_berzerker_launch_loc_3d4de_3d27e`.

**Robot — projectile timing/rendering and shoot-time collision routes are typed** (`Enemy03_Robot_Shot_Init / loc_3ABB6 / loc_3AC7E / loc_3AC92`)
Fixed candidate: live shots now use the `loc_3AC0C` muzzle offset and `loc_3AC48` camera/collision gate, then enter `RobotShotImpact` with the `loc_3AC7E/loc_3AC92` velocity clear and X offset. Flying shots keep a separate animation clock for looping `stru_3A754` (`unk_C8278`, `unk_C8280`, `unk_C8288`) without turning the shot into a DBF-lifetime object. The impact object survives the exact `stru_3A762` 3+4+8 frame duration and renders the three ROM impact frames (`unk_C8260`, `unk_C8268`, `unk_C8270`). Shoot-time collision routes are covered by `loc_3AAD4`/`loc_3AB04`/`loc_3AB56`/`loc_3AAFA` regressions listed above.

**Robot — body/head per-part collision and sprite swaps still absent** (`Enemy03_Robot_Init / loc_3A8AE`)
Fixed collision candidate: live roster now exposes the linked head object from `loc_3A7EE/loc_3A816`, friendly projectile damage scans both the parent body and linked head before entering `loc_3A8FC`, charging Berzerker side contact now targets the linked head `$38(a2)` only before routing `$20/$24` through `loc_3A9CC -> loc_3A9D2`, and stomp candidates cover both parent and linked-head hit reactions without double-applying damage on an overlapping frame. Remaining caveat: shoot/hit head-leg sprite swaps are still approximate visual representation, not a known collision-route bug without a visual/coroutine-frame oracle route. Focused evidence: `cargo test -p wildside-core robot_ -- --nocapture` passes 59 tests.

**Scorpion — collision/HP state machine not wired; damage/death go through generic path** (`loc_3DCB0 / loc_3DCD4 / loc_3DCF4`)
Fixed candidate: pending collision dispatch now follows `loc_3DCB0`; signed-negative parent collision enters `loc_3DCF4` death before HP damage, hurt/Kid-above collision enters `loc_3DCD4` with the nonfatal `stru_3DB7A` hibernate/restore route, unknown positive/zero collisions resume through `loc_3DD5C`, and death marks the linked tail `$3D` before the `stru_3DB80` wait-delete path. Regressions: `scorpion_collision_dispatch_matches_loc_3dcb0_compare_order`, `scorpion_negative_parent_collision_enters_loc_3dcf4_death_before_hp_damage`, `scorpion_pending_hurt_collision_enters_loc_3dcd4_damage_route`, `scorpion_pending_zero_and_unknown_positive_collision_resume_loc_3dcb0_3dd5c`, `scorpion_death_waits_for_stru_3db80_animation_before_delete`.

**Scorpion — tail/stinger sub-object absent as a live entity** (`Enemy13_Scorpion_Init / loc_3DC86`)
Fixed candidate: snapshots expose the paired tail gfx object from `loc_3DC86`, including the direction-sensitive X/Y offsets, walking/sting contact phases, and hidden tail state during death. Regressions: `scorpion_active_snapshot_exposes_tail_child_from_loc_3dc86`, `scorpion_tail_sync_clear_direction_matches_loc_3dc86`, `scorpion_tail_sync_set_direction_matches_loc_3dcaa`, `scorpion_sting_tail_contact_uses_rom_collision_size_rows_loc_1bce`, `scorpion_death_waits_for_stru_3db80_animation_before_delete`.

**Spinning_Twins — Berzerker side-collision upward launch not reproduced** (`loc_3BBF6`)
Fixed candidate: direct charging-Berzerker side contact and queued settled side-contact collision both route through `loc_3BBF6`; native preserves HP, seeds `y_vel=$FFFB8000`, disables walking motion, and loops through `loc_3BC22` gravity instead of generic damage. Regressions: `berzerker_charge_launches_spinning_twins_instead_of_damaging_loc_3bbf6`, `spinning_twins_berzerker_side_gate_loc_3bbf6_requires_active_charge`, `spinning_twins_berzerker_launch_loop_loc_3bc22_adds_gravity_and_repeats`, plus `spinning_twins_settled_collision_dispatch_loc_3bbc4_routes_contact_words`.

**Spinning_Twins — orbit oscillation timing + transition trigger diverge** (`loc_3B904`)
Fixed candidate: orbiting Twins now keep the parent anchored, advance the orbit index only when `animation_timer(a3)==1`, and drop only on ROM contact words: signed-negative, `$2C` Kid-above, or `$1C` hurt. Runtime materialisation then performs the `loc_3B93C` split into two walking children plus the falling visual. Regressions: `spinning_twins_orbit_loc_3b904_advances_only_on_animation_timer_one`, `spinning_twins_orbit_collision_route_loc_3b904_drops_only_for_rom_contact_words`, `spinning_twins_runtime_drop_materializes_two_children_and_visual_loc_3b93c`.

**Tank — non-fatal hit recoil + motion stop + hibernate not applied** (`loc_3B49A / loc_3B700`)
Fixed candidate: plain Tank nonfatal mouth/turret hits and Tank_Shooting turret hits now clear parent motion, set the linked child recoil timer, hibernate through the five-frame recovery window, and resume through `loc_3B4BA`/`loc_3B720` without entering death. Regressions: `tank_update_loc_3b49a_hurt_recovery_stops_motion_and_recoils_child`, `tank_child_stomp_hit_latches_contact_damage_through_loc_3b49a_hibernate`, `stomp_nonfatal_plain_rock_tank_stops_motion_for_loc_3b49a_hibernate`, `tank_shooting_nonfatal_projectile_hit_hibernates_loc_3b700`, `tank_shooting_hurt_recovery_loc_3b700_stops_motion_then_resumes_walk`.

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
