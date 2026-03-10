# PRD: Level 1 Fidelity Deepening (Iteration 2)

## Goal

Deepen Blue Lake Woods 1 from "playable but crude" to "recognisably faithful". Add the missing combat mechanic (stomp-kill), visual feedback (HUD, invulnerability flash), slope physics, collectibles, knockback, and enemy HP.

## Current State (as of 2026-03-10)

| System | Status |
|--------|--------|
| Stomp-kill enemies | Missing |
| HUD (HP, lives, diamonds) | Missing |
| Slope floor physics | Missing (slopes treated as flat solids) |
| Diamond collectible pickup | Missing (diamonds deal damage instead) |
| Item spawning from prize blocks | Missing (blocks just clear) |
| Damage knockback | Missing (no physical response) |
| Per-enemy hit points | Missing (no HP tracking) |
| Invulnerability flash | Missing (300-frame window invisible) |

## Phases

### Phase 1: Slope Floor Physics (US-013)

**Why first**: Level 1 has slope terrain. Currently the player walks on the top edge of slope cells — visible gaps and wrong movement feel.

**Requirements**:
- Modify `probe_floor` in `terrain.rs` to compute diagonal floor Y within slope cells
- `SlopeUp`: floor Y rises (decreases) left-to-right. `SlopeDown`: opposite
- `on_ground = true` and `y_velocity = 0` on slopes, same as flat solid

**Acceptance criteria**:
- Player follows diagonal surface smoothly
- Walking on/off slopes has no sticking or fall-through
- Tests: `probe_floor` on slope cell returns interpolated Y

---

### Phase 2: Invulnerability Flash (US-018)

**Why**: The 300-frame invulnerability window is invisible. Player needs visual feedback.

**Requirements**:
- Blink the player rectangle during invulnerability (toggle every 4 frames)
- Expose `invulnerable_frames` in `NativeTraceFrame`
- Skip player draw command on blink-off frames

**Acceptance criteria**:
- Player visibly blinks after damage, stops after invulnerability expires
- Tests: scene output omits player rect on blink-off frames when invulnerable

---

### Phase 3: Stomp-Kill Enemies (US-011)

**Why**: Core combat mechanic — player has no way to defeat enemies.

**Requirements**:
- Stomp condition: player overlaps enemy AABB AND `y_velocity > 0` (falling) AND player feet above enemy vertical midpoint
- On stomp: remove enemy from `active_enemies`, bounce player upward
- Pure function: `classify_enemy_contact` returning `Stomp | Damage | None`
- Stomp check runs before contact damage

**Acceptance criteria**:
- Falling onto enemy from above kills it + bounces player
- Side/below contact still damages player
- Tests: stomp kills, side damages, bounce sets negative y_vel, enemy removed

---

### Phase 4: HUD Display (US-012)

**Why**: HP, lives, and diamond count are tracked but invisible.

**Requirements**:
- Persistent HUD bar: HP pips, lives count, diamond count
- Add `diamond_count: u16` to `NativeRuntime`
- HP as coloured rectangles (filled vs dim), lives/diamonds as text

**Acceptance criteria**:
- HP pips reflect current HP. Lives and diamond counter displayed
- Taking damage visibly reduces HP pips

---

### Phase 5: Diamond Collectible Pickup (US-014)

**Why**: Diamonds are the primary collectible. Currently they deal damage.

**Requirements**:
- Diamond family collected on touch (removed), not damaging
- Increment `diamond_count`. Every 50 diamonds → extra life
- Collision order: stomp → Diamond pickup → contact damage

**Acceptance criteria**:
- Walking into Diamond removes it + increments count. No damage
- 50th diamond grants extra life

---

### Phase 6: Item Spawning from Prize Blocks (US-015)

**Why**: Prize blocks clear but spawn nothing. Should reward the player.

**Requirements**:
- Headbutting prize block grants a diamond (increment `diamond_count`)
- Smash/metal blocks unchanged

**Acceptance criteria**:
- Headbutt prize block → diamond_count + 1

---

### Phase 7: Damage Knockback (US-016)

**Why**: Taking damage has no physical response.

**Requirements**:
- On contact damage, apply horizontal knockback away from enemy + small upward velocity
- Knockback direction based on enemy position relative to player
- No knockback on spike/pit death

**Acceptance criteria**:
- Contact damage pushes player away from enemy with upward arc
- No knockback on spike/pit death

---

### Phase 8: Per-Enemy Hit Points (US-017)

**Why**: Some enemies take multiple stomps. Layout data has `hit_points_plus_one` but it's unused.

**Requirements**:
- Add `hit_points: u8` to `ActiveEnemy`, init from layout data
- On stomp: decrement HP. If 0 → remove. If > 0 → survive, player bounces
- Damaged enemies flash white briefly

**Acceptance criteria**:
- 1-HP enemies die on first stomp. 2+ HP survive first
- Bounce on every stomp regardless

---

## Out of Scope

- Ice/rubber terrain physics (not in Level 1)
- Checkpoints (not in Level 1)
- Moving platforms
- Per-block item assignments from ROM (all prize blocks → diamond)
- Hopper/flyer enemy archetypes (not in Level 1)
- Player forms and abilities
- Audio/GEMS
- Score system beyond diamond counting
- Sprite/tile rendering

## Verification

Each phase verified by:
1. Manual playtest of Level 1
2. `cargo test` — all new + existing tests pass
3. Oracle comparison where applicable
4. No regressions to existing behaviour
