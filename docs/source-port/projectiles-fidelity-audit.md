<!-- Generated 2026-06-10 by the interaction-fidelity-audit workflow (1 read-only Explore scout enumerated the units; one read-only Explore auditor per unit compared the live native path to the disassembly; every high/medium gap adversarially verified by an independent refuter - defaulting to refuted - before inclusion; low gaps included without a refute pass). This is a reviewable research artefact, not a gate. Each gap cites the owning ASM label and the native file:line searched. Re-run the workflow to refresh. -->

# Projectile System Fidelity Audit

Adversarially-verified audit of the player-form projectile and melee weapon paths in the Kid Chameleon source-port. At audit time, the dominant failure pattern was the same one the enemy audit found, sharpened here onto player weapons: an ASM-faithful collision predicate exists in `terrain.rs` (`probe_relative_collision_sub_B55C`, the exact `& $7000 == $6000` gate) and is wired into live movement, yet projectile terrain probes substituted the generic `solid_terrain_at` (`collision != Empty`). Several of those rows have since been resolved below. Remaining motifs originally included invented per-projectile timers, gates with no ASM basis, invented hitboxes/behaviours, and dropped helper workers; the live Red Stealth below-foot airborne stomp and EyeClops hidden-block foreground update worker have since been fixed.

## Coverage summary

| Unit | Coverage | Confirmed gaps |
|---|---|---|
| shared-projectile-step-and-cull | partial | 2 |
| projectile-vs-enemy-damage-delivery | partial | 1 |
| juggernaut-skull-shot-lifecycle | partial | 3 |
| skull-terrain-probe-fidelity | partial | 3 |
| maniaxe-axe-spawn-arc-spin-despawn | partial | 0 |
| red-stealth-sword-swing-melee | partial | 0 |
| red-stealth-airborne-downward-slash | partial | 1 |
| eyeclops-beam-fire-gate-and-damage | partial | 1 |
| juggernaut-five-way-shot-arming | full | 0 |
| projectile-vs-block-contact-and-prize-break | partial | 1 |
| offscreen-and-lifetime-culling-rules | partial | 3 |
| per-form-ammo-cooldown-gating | partial | 1 |

Original totals: 12 units audited, 1 full (no gaps), 11 partial, 0 stub. The audit originally found 17 confirmed gaps after refutation; 4 high/medium claims were adversarially refuted and dropped (skull probed-every-frame vs tile-boundary; Maniaxe axe deleted on throw-anim completion; Red Stealth ground re-swing invented-timer; EyeClops gate reopens on enemy contact). After the fixes recorded below, no confirmed projectile-system gaps remain active in this audit.

## Resolved since audit

- **Lion projectile homing Y-target offset sign** (`loc_3E20E / loc_3E240 / loc_3E192`) — fixed by `d1b3653c Fix Lion projectile homing target`. Native now computes `d7` as `projectile_y - kid_y - $10`, so the convergence point is `kid_y + 16px`, matching `loc_3E20E`; covered by `homing_y_target_offset_is_16px_below_player` and the adjacent `loc_3E192`/`loc_3E240` tests.
- **Lion projectile invented DBF lifetime** (`loc_3E192 / loc_3E2A8 / sub_3E2CE`) — fixed by `b9413dfd Stop Lion projectile lifetime countdown`. The live projectile pump now exempts `EnemyProjectileKind::LionProjectile` from generic lifetime decrement, matching the ASM loop that only exits through collision, owner/camera cull, or related delete paths; covered by `lion_projectile_loc_3e192_has_no_dbf_lifetime_counter`.
- **Maniaxe lifetime-zero axe enemy hit delivery** (`loc_8ABC / loc_8A52 / loc_8AF2`) — fixed by `f2a96990 Fix live Maniaxe axe enemy hits` and closed in backlog #296 by `ac3dbf51 Close stale Maniaxe axe hit row`. `process_friendly_projectile_hits` now allows live `EnemyProjectileKind::ManiaxeAxe` projectiles with `lifetime == 0` to damage enemies and delete on contact, matching `loc_8ABC`; covered by `live_maniaxe_axe_lifetime_zero_damages_and_deletes_loc_8abc`.
- **Juggernaut skull terrain collision gate** (`sub_87B0 / sub_B55C / loc_8852 / loc_8916`) — fixed by `dd72a5ed Fix skull terrain collision gate`. The live skull path now probes `terrain::probe_relative_collision_sub_B55C(..., 0, 0) == CollisionKind::Solid`, matching each `cmpi.w #$6000,d5` caller and ignoring hidden blocks, slopes and spikes; covered by `loc_8852_skull_wall_probe_ignores_non_6000_collision_classes` and `loc_8916_skull_floor_probe_ignores_non_6000_collision_classes`.
- **Maniaxe one-at-a-time gate duration** (`loc_76E4 / loc_8CB6 / loc_A7F8 / sub_78E8 loc_794C`) — fixed by modelling the Kid throw-animation latch separately from the axe object's lifetime. Native now clears `Maniaxe_throwing_axe` after the 17-frame `stru_8B4E`/`stru_8B60` throw pose while the first `ManiaxeAxe_Init` object keeps flying, allowing a second axe as in the ROM; covered by `grounded_helmet_state_sub_78e8_releases_maniaxe_gate_after_throw_animation`.
- **Juggernaut skull spawn-in-solid immediate delete** (`loc_87EC / sub_B55C / loc_899C`) — fixed by wiring the initial spawn-cell `sub_B55C` probe before the native projectile is pushed. Native now deletes the skull immediately when the post-offset spawn cell returns `$6000`, and still allows hidden blocks, slopes and spikes because `loc_87EC` only branches on `cmpi.w #$6000,d5`; covered by `loc_87EC_skull_spawn_inside_solid_deletes_immediately` and `loc_87EC_skull_spawn_probe_ignores_non_6000_collision_classes`.
- **Juggernaut skull invented DBF lifetime** (`sub_87B0 / loc_8810 / loc_899C`) — fixed by `78c956bc Stop skull projectile lifetime countdown`. The live projectile pump now recognises Juggernaut skull slots and does not decrement their native sentinel lifetime, matching the ASM loop that only exits through animation/object collision, terrain collision, or spent rebound budget; covered by `juggernaut_skull_loc_8810_has_no_dbf_lifetime_counter`.
- **Burst-child gravity order** (`loc_3263C / loc_325B6 / Hibernate_Object_1Frame`) — fixed by `ae206732 Fix burst projectile gravity order`. The shared projectile pump now lets Tar/Drips burst helpers move first, then applies `loc_3263C` post-move gravity, while leaving the explicit Juggernaut skull pre-move gravity path unchanged; covered by `loc_3263c_burst_helpers_apply_gravity_after_move`.
- **Lion projectile owner-token cull** (`sub_3E2CE / loc_3E314 / loc_3E30E`) — fixed by wiring the live owner-token check before the Lion camera-box cull. Native now deletes a Lion projectile whose `(segment, layout, token)` no longer matches an active Lion, and mirrors the ASM branch by skipping the `$5B` release decrement on token mismatch; covered by `lion_projectile_sub_3e2ce_deletes_owner_token_mismatch_without_release` plus the existing `lion_projectile_sub_3e2ce_*` camera-box tests.
- **Juggernaut skull concurrent startup workers** (`loc_A87A / loc_7718 / sub_86FA / sub_87B0`) — fixed by modelling pending `sub_86FA` workers as independent 3-frame counters instead of one scalar delay. Native now accepts consecutive C presses while `$FFFFFB70` is below 8, queues multiple startup workers, and spawns each skull when that worker's own counter reaches zero; covered by `juggernaut_skull_c_press_queues_concurrent_sub_86fa_workers`.
- **EyeClops beam hidden-block gameplay reveal** (`loc_E70A / loc_E756 / sub_E446`) — fixed by wiring the live beam render/collision column scan to `hidden_block_column_updates_sub_E446`. Native now mutates hidden cells touched by the beam into their revealed block type, including the adjacent-column scan once the beam counter reaches the ASM `loc_E790` threshold; covered by `eyeclops_beam_loc_E70A_reveals_hidden_blocks_in_scanned_column`. The `$12C` foreground upload worker returned by `sub_E446` still has no native queue and remains tracked below as visual fidelity work.
- **Red Stealth invented below-foot airborne hitbox** (`loc_B350 / loc_B380 / sub_868E / loc_76B0 / loc_8CF8`) — fixed by removing enemy damage from the airborne attack-down counter path. Native `process_sword_hits` now only checks the spawned `sub_868E` side sword worker hitbox, while `sub_B270 loc_B350/loc_B380` remains animation/sound/floor-impact state only; covered by `red_stealth_airborne_slash_does_not_damage_enemy_below_kid_loc_b350` and `red_stealth_airborne_slash_does_not_damage_shooting_rock_tank_below_kid`.
- **EyeClops beam hidden-block `$12C` foreground update worker** (`sub_E446 / loc_E70A / loc_E790`) — fixed by keeping each returned `HiddenBlockColumnUpdateSubE446` descriptor in live runtime state, ticking its `$12C` lifetime, clearing it on level resets, and counting it as a transient GFX worker. Covered by the extended `eyeclops_beam_loc_E70A_reveals_hidden_blocks_in_scanned_column` regression.

## Confirmed gaps

### High severity

None active after the Red Stealth below-foot hitbox fix.

### Medium severity

None active after the EyeClops hidden-block foreground update worker fix.

## Already tracked

- **RESOLVED 2026-06-12: Invented below-foot airborne hitbox damages enemies the ROM never hits** (`loc_B350 / loc_B380 / sub_868E`) — this audit corrected the older #153 tracking record, which had asserted the opposite. Native now treats `sub_B270 loc_B350/loc_B380` as animation/sound/floor-impact state only and leaves enemy damage to the `sub_868E` side sword worker.

## Backlog candidates

| # | Status | Pri | System | Summary | Verify |
|---|--------|-----|--------|---------|--------|
| FIXED | [x] | P1 | projectile | Live Maniaxe axe spawns with lifetime:0 and is skipped by process_friendly_projectile_hits (loc_8ABC) — fixed by `f2a96990` and closed in backlog #296 | `cargo test -q -p kid-core live_maniaxe_axe_lifetime_zero_damages_and_deletes_loc_8abc` |
| FIXED | [x] | P1 | projectile | Juggernaut skull terrain probe used generic solid_terrain_at (!=Empty) instead of sub_B55C ==$6000 gate — fixed by `dd72a5ed` | `cargo test -q -p kid-core probe_ignores_non_6000_collision_classes` |
| FIXED | [x] | P1 | projectile | Maniaxe one-at-a-time gate keyed to axe-object existence, not the ~17-frame throw animation (sub_78E8 loc_794C) — fixed by modelling the 17-frame throw latch separately from the flying axe object | `cargo test -q -p kid-core grounded_helmet_state_sub_78e8_releases_maniaxe_gate_after_throw_animation` |
| FIXED | [x] | P2 | projectile | EyeClops beam hidden-block reveal now mutates the live layout and keeps the `$12C` foreground update worker returned by sub_E446 live in runtime state | `cargo test -q -p kid-core eyeclops_beam_loc_E70A -- --nocapture` |
| FIXED | [x] | P2 | projectile | Lion homing projectile Y target sign inverted (loc_3E20E) — fixed by `d1b3653c`; native converges on kid_y+16px | `cargo test -q -p kid-core homing_y_target_offset_is_16px_below_player` |
| FIXED | [x] | P2 | projectile | Skull spawn-in-solid immediate delete (loc_87EC/loc_899C) not modelled — fixed by probing the post-offset spawn cell with sub_B55C and deleting immediately on `$6000` | `cargo test -q -p kid-core loc_87EC_skull_spawn` |
| FIXED | [x] | P2 | projectile | Skull startup serialized to one pending worker; ROM allows concurrent sub_86FA workers within the 3-frame window while FB70<8 (loc_A87A/loc_7718) — fixed by independent pending worker counters | `cargo test -q -p kid-core juggernaut_skull_c_press_queues_concurrent_sub_86fa_workers` |
| FIXED | [x] | P3 | projectile | Burst-child gravity applied pre-move; ROM generic-mover (is_moved) projectiles add gravity post-move (loc_3263C) — fixed by `ae206732` | `cargo test -q -p kid-core loc_3263c_burst_helpers_apply_gravity_after_move` |
| FIXED | [x] | P3 | projectile | Lion homing projectile gets invented 120-frame DBF lifetime; ASM loc_3E192 has none — fixed by `b9413dfd`; Lion shots no longer spend generic lifetime | `cargo test -q -p kid-core lion_projectile_loc_3e192_has_no_dbf_lifetime_counter` |
| FIXED | [x] | P3 | projectile | Juggernaut skull gets invented 120-frame DBF lifetime; ASM sub_87B0 has none — fixed by `78c956bc`; skulls no longer spend generic lifetime | `cargo test -q -p kid-core juggernaut_skull_loc_8810_has_no_dbf_lifetime_counter` |
| FIXED | [x] | P3 | projectile | sub_3E2CE owner-token gate not enforced on live cull (loc_3E314); projectile whose parent slot was reused not self-deleted — fixed by deleting on owner-token mismatch without releasing `$5B` | `cargo test -q -p kid-core lion_projectile_sub_3e2ce_deletes_owner_token_mismatch_without_release` |

## Not audited

- **TarMonsterStraightGlob gravity** — scout-excluded as Tar Monster family scope (deferred to enemy-fidelity-audit / backlog #046). Native gives `EnemyProjectileKind::TarMonsterStraightGlob` gravity `TAR_MONSTER_GLOB_GRAVITY` (0x1000) at enemies.rs:21563, but the ASM straight-glob loop `loc_324D0`/`loc_324E6` sets only x_vel and applies no gravity; gravity belongs to the burst children (`loc_3263C`). Not reported here as a projectile gap.
- **Maniaxe axe hit-extent dimensions** — could not be verified: the cited ROM `CollisionSize_Index unk_318B2` is absent from this disassembly version, so the native centred-±7 approximation vs the commented asymmetric (top=−14, height=14) box could not be confirmed against ASM lines. Needs an oracle probe.
- **Juggernaut skull C-press `is_animated` gate** — the `is_animated(a3)` C-press gate (kid.asm:13165, 7803) is omitted in native (helmets.rs:6605-6606); it could let the skull fire during a Kid animation the ASM would block, but reachability depends on the unmodelled animation state. Left as an ambiguity rather than a confirmed gap; warrants an oracle probe.
- **Maniaxe axe `$3E` facing field** — ASM `loc_8A66` writes `x_direction` to a per-object scratch byte `$3E(a3)` never read elsewhere; it tracks render facing for the spinning-axe sprite flip only, no hitbox/gameplay effect. Render-only, not modelled, not a gap.
- **Maniaxe terrain-vs-cull ordering** — ASM checks the solid-block terrain hit (`loc_8ACC`) before the out-of-bounds camera box (`loc_8AF2`); native checks the camera box (projectiles.rs:2854) before `terrain_kill` (projectiles.rs:2908). They disagree only on a frame where the axe is simultaneously on a solid block AND outside the camera box (ROM plays the hit-block anim/SFX, native silently deletes). Same delete outcome; the hit-block anim/SFX is a separate combat/animation concern, not the cull layer. Noted, not reported.
- **Skull per-frame x-probe vs tile-boundary gating** — the ASM only runs the wall probe after a tile-boundary-crossing test (`loc_8852` reached via `cmp.w d6,d7; bgt`) whereas native's per-frame x-probe is unconditional. The refuter found no observable divergence (the destination tile only differs when a boundary was crossed), so it was dropped; the boundary-gating concern is otherwise outside the solid-vs-slope scope and was not separately assessed.
- **Red Stealth airborne sprite mapping** — the dropped renderer sprite mapping (`step.visual` ignored in `step_red_stealth_airborne_slash`) is an acknowledged advisory follow-up under #153, a visual/non-damage concern, not re-reported as a damage gap.
- **EyeClops hard-beam ball object and 2-diamond spend timing** — owned by helmets-audit #92; the single-point hitbox and invented BEAM_SPEED/LIFETIME by #71/#74. Not re-reported.
- **Five-Way Shot arming/cost residuals** — the two existing divergences (state-machine flag fused into one-frame activation; cost routed through the generic `DiamondPower_Check` 5-vs-20 branch with the 20-cost branch dead) are already owned by helmets-fidelity-audit.md:79-80 and 150-151; the unit found no new gaps.

Closing trivia (folded, no backlog row): the Maniaxe axe also picked up a spurious `has_block` gate the ROM `loc_8A84` lacks, low-impact relative to the now-fixed slope breadth. Tooling caveat for re-runs: `grep` intermittently mis-detects kid.asm as binary and returns no matches (and silently mislabels labels as absent); use `grep -a`/`rg`/Read for all ASM citations.
