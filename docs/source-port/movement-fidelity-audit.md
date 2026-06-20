<!-- Generated 2026-05-30 by the `player-movement-fidelity-audit` workflow (1 read-only Explore scout enumerated 15 player move-states; one read-only Explore auditor per move-state compared native to kid.asm; every gap adversarially verified by an independent refuter, defaulting to refuted, before inclusion). 44 agents total. This is a reviewable research artefact, not a gate. Each gap cites the owning ASM label and the native `file:line` searched. Re-run the workflow to refresh. -->
<!-- Staleness-checked 2026-06-10: verdicts and evidence in fidelity-audit-staleness-2026-06-10.md. Maintained 2026-06-13 as stale entries are fixed or reclassified below. -->

# Player Movement Fidelity Audit

Adversarially-verified audit of the Kid Chameleon source-port's player movement, comparing each native move-state to the `kid.asm` player object-update routines.

The recurring failure mode is the one the enemy-fidelity audit (`enemy-fidelity-audit.md`, #176) and the dead-helper census (`dead-helper-census.md`, #203) also found crate-wide: ASM-faithful pure helpers exist and are unit-tested, but are never called from the live step path, so the runtime falls back to a generic profile, a hardcoded `false`, or omits the behaviour. The original audit found 18 of 26 confirmed gaps in this modelled-but-unwired shape. The airborne Skycutter `sub_9A20`, `with_left_right_conflict_resolved`, `crawl_wall_escape_sub_81B4`, `sub_7A10.blocked`, and `initiate_jump_with_hand_grab` ground/climb instances are now fixed and test-locked.

Two findings are player-specific and do not fit that mould:

- Architectural (`$14/$18` compound corner): the native collision pipeline is decomposed per-axis (separate floor/ceiling and wall passes), so it never forms the ROM's compound `collision_type` `$14`/`$18`. A whole class of slope-into-wall corner behaviour (crush kill; Juggernaut/Skycutter diagonal bounce-out; grid-aligned recovery snap) therefore has no executable counterpart, only documentation comments. Relates to #207.
- Fixed 2026-06-13 (ice/rubber): the ROM deliberately cross-wires its terrain index (ice block to index 1 to the "rubber"/slippery table; rubber block to index 2 to the "ice"/grippy table). Commit "Cross-wire ice rubber ground speed tables" moved that quirk into `PlayerForm::ground_speed_table`, removed the old movement-local hand-off, and verified the selector plus the Level 1 frame-731 regression through `mise run verify-slice-fast`.

## Coverage summary

| Move-state | Coverage | Confirmed gaps |
|---|---|---|
| Kid main object update (integrate + collision dispatch) | partial | 1 |
| Standing / idle (`MoveID_Standingstill=0`) | full | 0 |
| Ground walk/run horizontal acceleration | full | 0 |
| Braking / skid / turn-around | full | 0 |
| Duck / crawl (`MoveID_Crawling=1`) | full | 0 |
| Jump initiation (ground launch velocity) | full | 0 |
| Variable jump height (B-held gravity + terminal velocity) | full | 0 |
| Airborne horizontal control | partial | 1 |
| Secondary airborne path (`sub_8506`) | full | 0 |
| Up-/down-slope walking (`MoveID_Up_Down_Hill=6`) | full | 0 |
| Wall-grab / climb (Iron Knight, `MoveID_Wall_Climbing=4`) | partial | 3 |
| Landing / floor-support & platform riding | partial | 2 |
| Wall-hit + ceiling-hit collision response | partial | 1 |
| Damage recoil / knockback & Berzerker charge | partial | 3 |
| Skycutter form-specific horizontal movement | full | 0 |

Original totals: 15 move-states audited, 3 full (no gaps), 12 partial. 26 confirmed gaps (5 high, 9 medium, 12 low; 23 wiring, 3 hygiene). The `$14/$18` compound corner is the remaining duplicated defect surfaced by different auditors (main-object-update + wall/ceiling); the former airborne Skycutter `sub_9A20` duplicate is fixed below.

## Confirmed gaps

### High severity

**FIXED 2026-06-13: Ground accel terrain index selection preserved the ROM's ice/rubber swap** (`loc_9AE0`/`loc_9AF2`/`loc_9AFC` select; `sub_8600`/`loc_8680`/`loc_8688` write)
`sub_8600` writes index 1 for ice blocks (`BlockType::Ice=2`) and 2 for rubber (`BlockType::Rubber=6`); the selector `loc_9AE0` reads them inverted (`#2` to `unk_9C7A` punchy table, `#1` to `unk_9CCE` slippery table). Net ROM behaviour: ice to slippery glide (`accel $200`, caps `$40000-$70000`), rubber to punchy (`accel $2000`/`decel $5000`). Commit "Cross-wire ice rubber ground speed tables" centralised that original quirk in `PlayerForm::ground_speed_table` and removed the previous `movement.rs` rubber-to-ice hand-off. Verified by selector regressions, `level00_frame_731_flat_rubber_span_uses_refreshed_code_two_table_loc_8e16`, and `mise run verify-slice-fast`.

**FIXED 2026-06-13: Crawl stand-up wall escape (`sub_81B4`) is wired live** (`sub_81B4` @kid.asm:8952, from crawl stand-up `loc_8270` @9033)
ROM stand-up runs `sub_DAA6` (overhead platform) to `sub_B43A` (floor) to Skycutter gate to `sub_81B4` (wall escape): probes 16px above the feet at `x±$E`; both sides solid means stay crawling (wedged); one side open means restore standing half-width and snap `x_pos` flush to the tile boundary. Native now calls `crawl_wall_escape_sub_81B4` from the live crawl-release gate (`lib.rs:3272-3301`) after the `sub_DAA6` and head-row checks; `loc_8270_sub_81b4_both_side_walls_keep_release_frame_crawling` and `loc_8270_sub_81b4_right_escape_snaps_live_standup` verify the blocked and snap-out branches.

**FIXED 2026-06-13: Airborne control delegates Skycutter air-jump to `sub_9A20`** (`sub_B084` @kid.asm:13921-13927; `sub_9A20` @11588)
`sub_B084`'s first action: `cmpi.w #Skycutter,(Current_Helmet); bne loc_B09E; move.l x_vel,d0; bsr sub_9A20`. Airborne Skycutter bypasses the `unk_A4B2`/`unk_A4C6` air tables and uses `sub_9A20` (caps `$20000` walk/`$40000` run, `$2000`/`$4000` accel, two-tier cruise-decel). Native now mirrors that in both `update_horizontal_velocity_only` (`movement.rs:5244-5251`) and the full horizontal motion helper (`movement.rs:5590-5597`). Regression `sub_b084_airborne_skycutter_uses_sub_9a20_velocity` (`movement_tests.rs:4728-4790`) proves both live paths apply `$2000` from `sub_9A20` instead of the generic Skycutter jump-air table's `$3400`.

**FIXED 2026-06-13: Slope walking honours Up-held / Down-held acceleration** (`loc_9DAA` @kid.asm:11986-11995, `loc_9DC0`/`loc_9DF2`)
`loc_9DAA` dispatches slope accel on four inputs: Right to uphill, Left to downhill, Up to uphill, Down to downhill; only with none held does it fall to neutral `loc_9E2E`. Native now maps vertical-only commands to the equivalent slope-relative horizontal command in `slope_horizontal_input_loc_9DAA` (`movement.rs:7378-7410`) before `update_level_one_slope_ground_velocity` runs. Regressions `slope_up_held_only_drives_uphill_accel_loc_9daa` and `slope_down_held_only_drives_downhill_accel_loc_9daa` verify the live `OpeningSliceState` path branches to `loc_9DC0` and `loc_9DF2`.

### Medium severity

**Main object update: `$14/$18` compound-corner response absent** (`sub_A4EE`/`loc_A53C`/`loc_A594`/`loc_A556`/`loc_A5AE`; IK tails `loc_B9A2`/`loc_BA5A`/`loc_BB08`/`loc_BB30`) *(architectural; same root as the wall/ceiling-response gap below)*
ROM integrates X+Y together then runs a single `j_GfxObjects_Collision`; `collision_type` `$14` (floor+right-wall corner)/`$18` (floor+left-wall) dispatch to corner handlers. Juggernaut/Skycutter get a diagonal escape (`x_vel`/`y_vel=$FFFE0000` for `$14`; `y_vel=$FFFE0000`, `x_vel=+$00020000` for `$18`) repositioned from the cached tile addr `$FFFFFB6C`; other helmets take `loc_B9A2` recovery. The native decomposed per-axis passes (`movement.rs:3374`/`3389` floor, `3563`/`3567` wall) never form the compound type; `movement.rs:3211` states plainly "The `$14/$18` crush paths are not yet ported". Only triggers on slope-into-wall corner geometry (hence medium).

**Wall/ceiling response: slope-compound corner-crush (`loc_B9A2`/`loc_BA5A`) unported** (reached from `sub_A4EE` `loc_A53C`/`$14`, `loc_A594`/`$18`; Juggernaut bounce `loc_BB08`/`loc_BB30`; recovery snap `loc_BA1C`/`loc_BAD2`)
Same compound-`collision_type` root as above, audited from the wall/ceiling angle: corner-crush kill (unless Juggernaut/Skycutter), the Juggernaut fixed-velocity bounce, and the grid-aligned wall-hit recovery snap have no executable counterpart, only the comment block at `movement.rs:3198-3243`. The genuine `$04`/`$08` wall and `$10` ceiling responses are faithfully wired (`apply_wall_collision` `movement.rs:5057`, `apply_ceiling_collision` `movement.rs:5253`); only the compound `$14/$18` corner is missing. Relates to #207.

**FIXED 2026-06-13: Braking resolves Left+Right before `sub_B084`** (`loc_7464` @kid.asm:7610-7614; affects facing gate `loc_9B14`)
ROM clears Right once per frame when both are held, so both-held resolves to LEFT. The live `update_horizontal_velocity_only` and `update_horizontal_motion` paths now call `with_left_right_conflict_resolved` before constructing `HorizontalInput`, matching `loc_7464` before `sub_B084` consumes the direction bytes. Regression `loc_7464_left_right_conflict_resolves_left_before_sub_b084` verifies both helpers face Left and take the `loc_9B90` strong-brake slot from a stopped, right-facing Kid.

**FIXED 2026-06-13: Crawl entry honours `sub_7A10` `blocked` result** (`loc_8C70` @kid.asm:10092; `sub_7A10` `loc_7A90` @8143)
ROM gates crawl entry: `bsr sub_7A10; bne loc_8CB6`. When blocked (both head-height side probes solid after the snap) Kid stays walking and uses the walk table. Native now calls `align_player_for_down_hold_sub_7A10` in the live walking-state crawl transition and clears `movement_input.down` when `DownHoldAlignmentSub7A10.blocked` is true (`lib.rs:3331-3352`). Regression `loc_8c70_sub_7a10_blocked_keeps_walking_velocity_path` verifies the blocked frame uses walking decel rather than the crawl air table.

**FIXED 2026-06-13: Grounded jump honours `KidGrabbedByHand` `+$20000` weakening** (`sub_A432`/`loc_A466` @kid.asm:12799-12801)
ROM adds `$20000` to the negated launch `y_vel` when `KidGrabbedByHand` is set, shortening a jump made while held by a Hand. Native live jump paths now call `initiate_jump_with_hand_grab` with the slice flag (`movement.rs:5346-5352`, `5796-5800`, `5816-5820`, `5868-5872`, `5889-5893`), and the Hand enemy step surfaces `HandState::Grabbing` onto `kid_grabbed_by_hand` before the player slice runs (`enemies.rs:42115-42130`). Regression `loc_A466_kid_grabbed_by_hand_weakens_live_grounded_jump` verifies the grounded player consumer, and `refresh_active_roster_surfaces_hand_grab_onto_slice_loc_32906` verifies the live Hand producer.

**Landing: moving-platform carry bypasses the `sub_8F26`/`sub_902A` move-and-collide passes** (`loc_8D72` @kid.asm:10150, `loc_82D8` @9063)
ROM inherits platform motion into the player's velocity then re-runs the collision passes, so the carried displacement is wall/floor/ceiling-clamped. Native (`platforms.rs:780-783`) adds `platform.x_velocity`/`y_velocity` straight onto `x_pos`/`y_pos` with no wall pass on the carried X delta, so a platform pushing a grounded Kid into a wall drives him through it. Falls under #208 (platform physics).

**FIXED 2026-06-13: Damage preserves wall-grab/cling state** (`Check_for_recent_damge`/`loc_B700`/`loc_B72C` @kid.asm:14632-14659)
The ROM hurt path only touches HP, damage flags, SFX, and helmet/death routing; it never writes the wall-stick/cling latches. Native `apply_damage_core` now leaves `right_wall_sticky`/`left_wall_sticky`/`wall_cling_direction` untouched while applying the `Just_received_damage` and hitpoint effects. Regression `damage_preserves_wall_sticky_loc_b700` covers the preservation against the ASM hurt path.

**FIXED 2026-06-13: Berzerker charge reflects Kid `x_vel` only for `object_meta` bit 15** (`loc_B678`/`loc_B692`/`return_B690` @kid.asm:14582-14598)
ROM negates `x_vel` on a charging side-contact only when bit 15 of the copied enemy `object_meta` is set (`bclr #$F,d6; bne loc_B692`). Native `process_berzerker_charge_hits` now routes that through `berzerker_charge_reflects_player_velocity_loc_b678`: ordinary enemy and boss-head contacts skip the reflection, while Scorpion's sting window reflects once. Regressions `berzerker_charge_loc_b692_skips_reflect_when_object_meta_bit_15_clear`, `berzerker_charge_loc_b692_skips_boss_heads_with_only_object_meta_bit_7`, and `berzerker_charge_loc_b692_reflects_player_x_velocity_once_for_scorpion_sting_bit_15` cover the gate.

### Low severity

**FIXED 2026-06-13: Removed stale `not(test)` dead_code attr from wired `grounded_surface_type_sub_8600`** (`sub_8600`)
`grounded_surface_type_sub_8600` is live-called from movement terrain refresh paths (`movement.rs:3142`, `5103`, `6837`, `6872`, `7026`, `7047`) and covered by the `sub_8600` rubber/ice regressions. The stale `#[cfg_attr(not(test), allow(dead_code))]` annotation has been removed; the function itself remains faithful, including the `loc_8688` rubber-wins-ties quirk.

**FIXED 2026-06-13: Crawl entry checks `KidGrabbedByHand` before taking Down-held crawl path** (`loc_8C70` @kid.asm:10084)
ROM skips crawl entry while held by a Hand via `tst.b (KidGrabbedByHand).w; bne.w loc_8CB6`. Native now suppresses the Down-held crawl/air-control path when `slice.kid_grabbed_by_hand` is set and also freezes the Down-look accumulator while grabbed (`lib.rs:3802-3810`). Regression `loc_8c70_crawl_entry_suppressed_while_grabbed_by_hand` verifies the velocity branch stays on the walking table instead of the crawl table, and `loc_8c70_grabbed_kid_down_held_freezes_look_offset` verifies the camera-look side effect.

**FIXED 2026-06-13: Jump launch queues `sfx_Jump` from `loc_A480`** (`sub_A432`/`loc_A480` @kid.asm:12821-12822)
ROM plays `sfx_Jump` after storing the launch velocity and applying the three-quarter `x_vel` reduction. Native now queues `QueuedSoundEffect::Jump` from `queue_jump_sound_from_probe_loc_A480` when the live jump probe reports `JumpPressed` or `JumpBuffered` (`lib.rs:2777-2792`), and both Iron Knight climb-resume and normal movement call sites invoke it after the slice step (`lib.rs:3430-3431`, `3462-3463`). Regression `grounded_jump_queues_sfx_jump_loc_a480` verifies the grounded launch sound against ASM `sub_A432`/`loc_A480`; title-card handoff regressions in `trace.rs` verify the sound is not buffered before live gameplay owns the shell.

**FIXED 2026-06-13: Airborne Skycutter bypasses the 1.5× run-accel multiplier** (`sub_B084` `loc_B09E` @kid.asm:13935-13939)
This was the low-severity consequence of the unwired `sub_9A20` air branch. Because the live airborne Skycutter paths now branch before `loc_B09E`, the generic `Ctrl_A` multiplier remains limited to the nine helmets that actually reach the ROM table path; Skycutter gets `sub_9A20`'s flat `+$4000` run step.

**FIXED 2026-06-13: Slope walking mirrors `y_vel = ±x_vel` on steady frames** (`loc_9D22` @kid.asm:11942-11949)
ROM stores the diagonal slope velocity every `loc_9D22` frame: `y_vel = -x_vel` for SlopeUp and `+x_vel` for SlopeDown. Native now mirrors that before and after the grounded slope gravity/position phases (`movement.rs:3394-3441`), preserves the diagonal value through slope floor handling (`movement.rs:6849-6858`), and still installs it on fresh recontact (`movement.rs:7171-7177`). Regressions `asm_faithful_loc_9d22_canonicalises_slope_y_velocity_from_x_velocity`, `asm_faithful_loc_9d22_preserves_slope_y_velocity_after_floor_snap`, and `asm_faithful_floor_collision_preserves_grounded_slope_y_velocity` cover the stored velocity against ASM `loc_9D22`.

**FIXED 2026-06-13: Climb wall-gone release seeds −4 px/frame pop and hibernates through `loc_A6F8`** (`loc_A2F6`/`loc_A30A` @kid.asm:12610-12625)
At the top of a wall ROM sets `y_vel=$FFFC0000` (−4.0 px upward), runs the airborne visual update, and branches to `loc_A6F8`, which sets `MoveID_Jump` and hibernates the release frame. Native now clears Iron Knight climb state, seeds `slice.player.y_velocity = -0x4_0000`, and returns `IronKnightClimbStep::ExitHibernate` from the wall-gone probe path (`helmets.rs:5577-5584`) so the runtime uses the existing `loc_A6F8` hibernate shell (`lib.rs:3385-3410`). Regression `loc_A30A_wall_gone_releases_with_upward_pop_velocity` verifies both the velocity seed and the hibernate handoff against ASM `loc_A30A`.

**Climb: loop ignores the per-frame `Addr_PlatformStandingOn` moving-platform exit** (`loc_A28A`/`loc_A9AA` @kid.asm:12579-12585,13266-13283)
ROM clears then re-checks `Addr_PlatformStandingOn` each climb iteration to transfer the climber onto a moving platform crossing the climb column. Neither `step_iron_knight_climb` nor `_resume` checks `active_platforms`. Edge geometry.

**FIXED 2026-06-13: Climb B-press jump-off honours `KidGrabbedByHand` weakening** (`loc_A426` to `sub_A432` `loc_A466` @kid.asm:12774-12805)
Climb B-press now calls `initiate_jump_with_hand_grab(0, jump_height, slice.kid_grabbed_by_hand)` (`helmets.rs:5533-5540`), matching the ROM's jump through `sub_A432` before `loc_A466` stores y velocity. Regression `loc_A426_iron_knight_climb_b_press_honours_kid_grabbed_by_hand_loc_A466` proves the grabbed launch is exactly `$20000` weaker while the existing `iron_knight_b_press_*` tests preserve normal detachment and held-left momentum.

**FIXED 2026-06-13: `iron_knight_climb_visual_sub_A3BC` no longer carries stale dead-code attr** (`sub_A3BC` @kid.asm:12694-12717)
The helper is live-called every climb frame from `step_iron_knight_climb` (`helmets.rs:5627`) and no longer carries `#[cfg_attr(not(test), allow(dead_code))]`. Existing regressions `iron_knight_climb_visual_sub_a3bc_wraps_negative_motion_to_top_pose` and `iron_knight_resume_climb_updates_sub_a3bc_pose_accumulator` cover the accumulator behaviour against ASM `sub_A3BC`.

**Climb: airborne wall-attach is a native extension with no ROM entry** (`loc_7696` @kid.asm:7762-7769 is the sole `bra loc_A276`)
The only ROM path into the wall-climb state is the grounded walk state; native additionally allows attach while airborne (`helmets.rs:4684-4698`), letting the Iron Knight grab a wall mid-jump where the ROM cannot. Over-permissive extension; the grounded (common) case is faithful.

**FIXED 2026-06-13: Grounded platform carry re-seats `y_pos = platform_top − 1`** *(to #208)* (`loc_8D72` @kid.asm:10164-10165, `loc_8306` @9077-9078)
ROM hard-reseats `y_pos` to `platform_y − 1` each riding frame and copies platform `y_vel` before the ceiling probe. Native now mirrors that in both the generic platform rider path and the lift-platform rider path instead of applying additive Y velocity. Regressions `loc_8da0_platform_y_position_reseats_grounded_player`, `platform_collision_reseats_rider_to_platform_y_loc_8da0`, and `lift_platform_carries_grounded_player` cover the generic and lift variants. The remaining platform-carry gap is the horizontal `sub_8F26`/wall re-collision bypass above.

**Damage: `clear_special_movement_state_sub_DB22` is test-only** *(hygiene to #209)* (`sub_DB22` @kid.asm:17170)
`#[cfg(test)]` helper (`helmets.rs:4180-4207`); not a behavioural gap (production clears flags via `refresh_grounded_helmet_state_sub_78E8`) but the ASM symbol resolves only to a test fixture, so an auditor searching this symbol finds no live wiring.

**FIXED 2026-06-13: Skycutter raw both-direction-held facing follows `sub_9A20` branch order** (`sub_9A20` `loc_9A84`/`loc_9A32` @kid.asm:11589-11594,11631-11635)
`update_skycutter_horizontal_velocity_sub_9A20` now derives `next_facing` from the same branch selector used for velocity (`physics.rs:569-579`): facing Right with raw Left+Right held flips left and takes `loc_9A90`, matching the ROM's left-first test. Regression `skycutter_sub_9a20_raw_both_held_uses_current_direction_branch_order` locks the edge case; live movement paths still resolve Left+Right earlier via `loc_7464`.

## Relationship to existing backlog

- Hygiene (1 remaining gap: `clear_special_movement_state_sub_DB22`): fold into #209 (strip stale `cfg_attr(not(test))` from wired helpers).
- Platform carry (remaining gap: X-axis re-collision): under #208 (all platform physics unwired).
- Compound `$14/$18` corner (the architectural pair): relates to #207 (generic collision primitives unwired).
- Wiring gaps that change behaviour against the ROM need the #205 object/enemy-active oracle harness to verify, except the ice/rubber inversion, which is a deterministic table-mapping bug fixable and verifiable now via a grounded-walk scenario.
