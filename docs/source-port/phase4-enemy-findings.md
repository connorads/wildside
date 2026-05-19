# Phase 4: Per-Enemy ASM Fidelity — Verification Findings

Comprehensive ASM verification of all 28 enemy families against native implementation.
Each finding includes ASM line references and native code locations.

## Methodology

For each family: read full `code/enemy/<Family>.asm`, compare every constant, state transition, and behaviour against the native `crates/kid-core/src/enemies.rs` implementation. Findings categorised as:
- **Correct**: ASM and native match
- **Mismatch**: Values or logic differ
- **Missing**: Behaviour exists in ASM but not in native
- **Dormant**: Mismatch exists but never triggered by ROM data

---

## Tier 1 Pacers

### Goat — Verified correct
- HP: `enemy_level + 3` — correct
- Speed: base $8000, step $10000 — correct
- Tracking: >32px facing reversal (loc_3CC18) — correctly implemented
- Timers: $48=9, $4A=16 (sub-pixel alignment, not gameplay timers)

### Armadillo — Verified correct
- HP: `enemy_level + 1` — correct
- Speed: base $8000, step $10000 — correct
- Timers: $48=9, $4A=16

### Tank — Verified correct (previously fixed #036)
- HP: `enemy_level + 2` — correct
- Speed: base $8000, step $8000 — correct (was $10000, fixed)

### Crab — Multiple mismatches
- HP: `enemy_level + 2` — correct
- **Speed cap mismatch**: ASM is binary ($8000 at HP=2, $18000 at HP≥3). Native uses linear formula which overestimates at HP≥4: $28000 instead of $18000.
- **Probe half-width**: ASM uses $C (12px) via $48(a5). Native uses hardcoded 8px (`ENEMY_PROBE_HALF_WIDTH`). Affects wall/edge reversal timing by 4px.
- **Missing: Player proximity freeze**: ASM `loc_3B0BE` implements freeze within 64px when player faces Crab, unfreeze beyond 224px. Not in native.
- **Missing: Hit animation pause**: ASM pauses movement during hurt animation. Native continues pacing.
- **Missing: Berzerker-charge launch**: ASM launches Crab at 2x player velocity with deceleration physics.

### Scorpion — Implemented this session
- HP: `enemy_level + 3` — correct
- Speed: base $18000 — correct
- **Tracking callback**: implemented (walk_speed_override + scorpion_tracking_lock)
- Speed escalation: $38000 at HP≥5 within 170px — implemented

### Driller — Speed cap + missing mechanics
- HP: `enemy_level + 2` — correct
- Speed: base $8000, step $8000 — correct for HP 2-4
- **Speed cap mismatch**: ASM caps at $18000 for HP≥4. Native linear formula gives $28000+ at HP≥5.
- **Missing: Block drilling**: ASM drills through smashable/ice blocks during walk animation. Not in native.
- **Missing: Stuck-enemy death**: ASM kills enemy after ~64 direction reversals via $5A overflow counter.
- Timers: $48=12, $4A=16 (sub-pixel alignment thresholds, not timers)

---

## Tier 2

### Cloud — Bounce path and attack fixed
- HP: effectively 1-hit kill — functionally correct
- Velocity: $3000/$5000/$7000 by tier — correct values
- **Fixed: Post-bounce idle removal**: ASM `loc_35BDE -> loc_35C54` writes `move.w #$3C,d3` after wall reversal, but the loop never reads `d3`. Native now mirrors the ROM by reversing and resuming flight on the next frame, with `cloud_keeps_flying_immediately_after_wall_reversal` covering the path.
- **Fixed: Lightning bolt attack**: ASM `loc_35CD8 -> loc_35DB8 -> loc_35DF0` now drives native Cloud attack latching and hostile projectile spawn, with `cloud_runtime_loc_35cd8_latches_then_spawns_diagonal_lightning` covering the live flyer wiring.
- **Dormant: Velocity index off-by-one**: ASM indexes by raw `hit_points_plus_one`, native subtracts 1 first. All ROM Clouds use index 0, so no practical impact.

### Sphere — Slope deflection added
- Gravity: $800/frame — correct
- Wall bounce: negate x_vel — correct
- Floor bounce: negate y_vel — correct
- Ceiling bounce: -(y_vel >> 1) — correct
- **Fixed: Slope Up/Down deflection**: ASM `loc_36012`/`loc_36032` swaps velocities with appropriate negation. Native now mirrors `step_big_hopping_skull` behavior for both SlopeUp and SlopeDown.

### Fireball — Gravity/bounce cap fixed, slope deflection added
- Gravity and bounce cap now follow Fireball.asm `unk_32BA6`, so live HP 1/2/3 uses `$2000/$3000/$3000` gravity and `-$40000/-$50000/-$50000` as the capped rebound speed.
- **Fixed: Slope Up/Down deflection**: ASM `loc_32D06`/`loc_32D46` swaps velocities identically to Sphere. Native now handles both slope types in `step_bouncer_with_rng`.
- RNG perturbation on bounce: implemented with local RNG state.

---

## Tier 3-4

### Dragon Walking — Fixed this session
- Breath interval: initial $12C (300f), reset $1F4 (500f) — dual constant implemented

### Dragon Flying — Fire velocity/offset fixed, collision gaps remain
- All script constants verified correct (drift speeds, accelerations, caps, dive script entries)
- All 3 variant scripts verified correct
- **Fixed: descent fire velocity**: ASM `loc_348A0` copies the dragon's live `x_vel` into the projectile and sets `y_vel=+3 px/frame`. Native now mirrors that path, with regression coverage for the descent fire spawn.
- **Fixed: fire mouth X offset**: ASM `loc_34BAC/loc_34BE6` applies ±10px from the dragon mouth. Native applies the signed offset and `dragon_flyer_descent_fire_uses_live_horizontal_velocity_and_downward_arc` covers the live spawn path.
- **Missing: Wall-reversal during Drift**: ASM reverses direction on terrain collision. Not in native.
- **Missing: Collision-recovery during Descent**: ASM has 6 recovery sub-paths. Not in native.
- **Missing: Death-on-stomp sequence**: ASM has specific death animation + explosion spawn.

### Tornado — Acceleration scaling fixed, Cyclone interaction implemented
- Cycle timers: 60/40/30 by level — correct
- Base acceleration: $C00 — correct
- Wall/floor bounce: correct
- **Fixed: Acceleration doubling for levels 1-2**: native now persists the ROM's `$4E(a5)` init flag and doubles the signed per-axis acceleration after `loc_35444` resolves the drift row.
- **Fixed: Target velocity doubling for level 2**: native now persists the ROM's `$4F(a5)` init flag, doubles the desired drift speeds, and reloads the 60/40/30 pattern timer from the raw layout level instead of live HP.
- **Fixed: Cyclone-flying bounce interaction**: ASM `loc_35986` implements bounce when Cyclone-flying player contacts Tornado. Native now sets player x_velocity = ±$20000, Tornado x_velocity = opposite halved ∓$10000, and cyclone_y_accel = $28 (40). Other forms take normal contact damage.
- **Missing: Cyclone-flying interaction**: ASM boosts player Y-acceleration ($28) and reflects Tornado x_velocity when player is in Cyclone form. Meaningful gameplay mechanic — ~~IMPLEMENTED~~.
- RNG pattern selection: documented DEVIATION.

### Ninja, Hand, Lion, Robot, Orca, SpinningTwins, TarMonster, Drips
- Previously verified and fixed in earlier sessions (#040-#061)

### UFO — Previously verified
- Beam child-object system implemented (#024)

### Fire Demon — Previously verified
- Death trail visual is TODO(fidelity) (rendering only)

---

## Bosses

### HeadyMetal — Phase-dependent fire + X gate done; death wobble render path wired, ASM timer-boundary sprite regressions added, full oracle visual verification still pending (#062)
### Shishkaboss, BoomerangBosses, BagelBrothers — Multi-head system verified (#060)

---

## Systematic issues across families

### Probe half-width hardcoded at 8px
ASM per-enemy $48(a5) values: Crab=12, Driller=12, Armadillo=9, Scorpion=9, Goat=9. Native uses `ENEMY_PROBE_HALF_WIDTH = 8` for all pacers. This affects wall/edge reversal timing by up to 4px.

### Speed formula linear extrapolation beyond ROM data
Crab and Driller ASM cap speed at $18000 for high HP. Native `pacer_walk_speed` uses a linear formula that extrapolates beyond the 2-3 tiers the ASM actually handles. Only affects enemies with `enemy_level ≥ 2-3` which may be rare in ROM data.

### Slope deflection missing from step_bouncer
Both Sphere and Fireball ASM have explicit slope collision handlers. Only BigHoppingSkull has slope handling in native. Could be ported from `step_big_hopping_skull` to `step_bouncer`.

---

## Priority ranking for fixes

### High impact (affects gameplay noticeably)
1. ~~Archer arrow lifetime + re-aiming~~ — **FIXED**: terrain-kill, ±1 re-aim, spawn offset, dx==0 quirk
2. ~~Tornado Cyclone-flying interaction~~ — **FIXED**: bounce instead of damage when Cyclone-flying contacts Tornado
3. ~~Crab player proximity freeze~~ — **FIXED**: freeze within 64px when player faces crab, unfreeze beyond 224px
4. ~~Dragon Flying wall-reversal during Drift~~ — **FIXED**: wall collision flips direction and negates x_velocity in Drifting state

### Medium impact (affects specific interactions)
5. **FIXED**: Sphere/Fireball slope deflection
6. Driller block drilling
7. Probe half-width per-family customisation

### Low impact (edge cases or rare scenarios)
8. Crab/Driller speed cap at high HP (rare enemy levels)
9. Dragon Flying collision-recovery during Descent
10. Dragon Flying death-on-stomp sequence
