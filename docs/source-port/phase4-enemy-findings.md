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

### Cloud — Bounce path fixed, attack still missing
- HP: effectively 1-hit kill — functionally correct
- Velocity: $3000/$5000/$7000 by tier — correct values
- **Fixed: Post-bounce idle removal**: ASM `loc_35BDE -> loc_35C54` writes `move.w #$3C,d3` after wall reversal, but the loop never reads `d3`. Native now mirrors the ROM by reversing and resuming flight on the next frame, with `cloud_keeps_flying_immediately_after_wall_reversal` covering the path.
- **Missing: Lightning bolt attack**: ASM `loc_35CD8` spawns directional lightning bolts during 31-frame attack windows. Not in native at all.
- **Dormant: Velocity index off-by-one**: ASM indexes by raw `hit_points_plus_one`, native subtracts 1 first. All ROM Clouds use index 0, so no practical impact.

### MiniHoppingSkull — Fixed this session
- HP: `enemy_level + 1` — correct
- Hop velocity: base -$38000 Y, $18000 X — correct
- **HP boost**: +$4000 X, -$10000 Y at HP>1 — implemented this session
- Ground delay: 21 frames (dbf #$14) — correct (previously verified)

### BigHoppingSkull — Verified correct (previously fixed #056)
- Continuous bouncer with $FA0 gravity, wall/floor/ceiling/slope deflection

### Sphere — Missing slope deflection
- Gravity: $800/frame — correct
- Wall bounce: negate x_vel — correct
- Floor bounce: negate y_vel — correct
- Ceiling bounce: -(y_vel >> 1) — correct
- **Missing: Slope Up/Down deflection**: ASM swaps X/Y velocities (with negate on slope-up). `step_bouncer` has no slope handling. Exists in `step_big_hopping_skull` but not ported to Sphere.

### Archer — Fixed: arrow lifetime, re-aiming, spawn offset, dx==0 aiming
- HP: passthrough from layout — correct
- Fire timers: $1E/$14/$0A — correct
- Velocity table: all 28 entries — correct
- **Fixed: Aiming dx==0**: ASM `loc_34530` ORIGINAL_QUIRK — sets d7=0 then tests d7, not d4/dy. Always returns angle 21 (straight down). Native now matches.
- **Fixed: Arrow spawn offset**: ASM 3rd animation frame offsets (-8px X, -17/-24px Y). Native now applies per-facing and vertical/horizontal offsets.
- **Fixed: Arrow lifetime + re-aiming**: ASM arrows persist until terrain/object collision (loc_342D6). Re-aim ±1 angle every `arrow_accuracy` frames toward player (loc_34308/loc_34320 circular adjustment). Native now uses terrain_kill + reaim fields.
- **Fixed: Ratio overflow**: ASM loc_34546 returns 6/21 when ratio ≥ $3F. Native was clamping ratio instead. Now branches correctly.

### EmoRock — Verified correct
- Terminal velocity: $60000 threshold, $40000 cap — correct (previously verified)
- Hover step: $2000 — correct

### Fireball — Missing slopes
- Gravity and bounce cap now follow Fireball.asm `unk_32BA6`, so live HP 1/2/3 uses `$2000/$3000/$3000` gravity and `-$40000/-$50000/-$50000` as the capped rebound speed.
- **Missing: Slope deflection**: ASM has explicit SlopeUp/SlopeDown handlers (swap X/Y velocities). Not in `step_bouncer`.
- RNG perturbation on bounce: documented DEVIATION (±$4000 XOR'd with Camera_X_pos).

---

## Tier 3-4

### Dragon Walking — Fixed this session
- Breath interval: initial $12C (300f), reset $1F4 (500f) — dual constant implemented

### Dragon Flying — Fire velocity + collision gaps
- All script constants verified correct (drift speeds, accelerations, caps, dive script entries)
- All 3 variant scripts verified correct
- **Fixed: descent fire velocity**: ASM `loc_348A0` copies the dragon's live `x_vel` into the projectile and sets `y_vel=+3 px/frame`. Native now mirrors that path, with regression coverage for the descent fire spawn.
- **Fire spawn X offset missing**: ASM applies ±10px mouth offset. Native spawns at dragon centre.
- **Missing: Wall-reversal during Drift**: ASM reverses direction on terrain collision. Not in native.
- **Missing: Collision-recovery during Descent**: ASM has 6 recovery sub-paths. Not in native.
- **Missing: Death-on-stomp sequence**: ASM has specific death animation + explosion spawn.

### Tornado — Acceleration scaling fixed, Cyclone interaction still missing
- Cycle timers: 60/40/30 by level — correct
- Base acceleration: $C00 — correct
- Wall/floor bounce: correct
- **Fixed: Acceleration doubling for levels 1-2**: native now persists the ROM's `$4E(a5)` init flag and doubles the signed per-axis acceleration after `loc_35444` resolves the drift row.
- **Fixed: Target velocity doubling for level 2**: native now persists the ROM's `$4F(a5)` init flag, doubles the desired drift speeds, and reloads the 60/40/30 pattern timer from the raw layout level instead of live HP.
- **Missing: Cyclone-flying interaction**: ASM boosts player Y-acceleration ($28) and reflects Tornado x_velocity when player is in Cyclone form. Meaningful gameplay mechanic.
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
2. Tornado Cyclone-flying interaction
3. Crab player proximity freeze
4. Dragon Flying wall-reversal during Drift

### Medium impact (affects specific interactions)
5. Sphere/Fireball slope deflection
6. Driller block drilling
7. Probe half-width per-family customisation

### Low impact (edge cases or rare scenarios)
9. Crab/Driller speed cap at high HP (rare enemy levels)
10. Cloud lightning bolt attack
11. Dragon Flying fire spawn X offset
12. Dragon Flying collision-recovery during Descent
13. Dragon Flying death-on-stomp sequence
