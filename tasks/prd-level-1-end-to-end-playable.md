# PRD: Level 1 End-to-End Playable

## Goal

Walk from spawn to flag on real Level 1 terrain, with enemies that can kill you, blocks that react, and level completion that triggers — making Blue Lake Woods 1 a complete, playable level.

## Current State (as of 2026-03-09)

| System | Status |
|--------|--------|
| Block grid collision (floor/wall/ceiling) | Done |
| Player movement (accel/decel/jump/gravity) | Done |
| Camera follow | Done |
| Death/respawn lifecycle | Done (damage, HP, lives, respawn at spawn point) |
| Enemy activation window | Done (positions loaded, roster built) |
| Block types parsed (smash, prize, ice, etc.) | Done (data only, no gameplay effects) |
| Flag position loaded | Done (data only, no collision) |
| Enemy AI / movement | Missing |
| Enemy-player collision | Missing |
| Pit/spike death triggers | Missing |
| Block interactions (smash, prize) | Missing |
| Flag collision / level completion | Missing |
| Air control | Missing |
| Block grid rendering | Missing (invisible terrain) |

## Phases

Phases are ordered by gameplay impact. Each phase produces a testable, committable increment. Phases are independent enough to be worked in any order, but the suggested order maximises playability earliest.

---

### Phase 1: Pit Death + Spike Death

**Why first**: The player can already walk on terrain and fall into gaps, but nothing happens. Without death triggers, you can't lose — and losing is what makes traversal meaningful.

**Requirements**:
- Detect when the player falls below the level floor boundary (y > level height in pixels) — trigger death
- Detect when the player contacts a `Spikes` collision cell — trigger death
- Wire both into the existing `apply_damage()` / death lifecycle
- Spikes should be instant kill regardless of HP (matches original)

**Acceptance criteria**:
- Falling off the bottom of the level triggers `DeathWindup` -> `Dying` -> `Respawning`
- Walking/falling onto a spike cell triggers the same death sequence
- Player respawns at level start position after death

---

### Phase 2: Flag Collision + Level Completion

**Why second**: With death working, the player needs a goal. Flag collision closes the gameplay loop: spawn -> traverse -> reach flag -> win.

**Requirements**:
- Detect player overlap with flag position (flag_x, flag_y from level header)
- Use a reasonable hit region (the flag is a 16x16 object; check overlap with player hitbox)
- On flag touch: transition to a completion state (can be minimal — freeze input, display "LEVEL COMPLETE" text, then advance)
- `advance_level()` already exists for debug; wire it as the post-completion action
- For now, advancing past the last level can loop back to Level 1 or show a placeholder

**Acceptance criteria**:
- Touching the flag at (1872, 768) triggers level completion
- After a brief pause, the next level loads (or a placeholder if level data isn't extracted)
- The player can complete Level 1 from spawn to flag on real terrain

---

### Phase 3: Enemy Collision + Contact Damage

**Why third**: Enemies are visible (activated in the roster) but harmless. Contact damage makes traversal dangerous.

**Requirements**:
- Implement AABB overlap check between player hitbox and each active enemy's hitbox
- On overlap: call `apply_damage()` on the player (strips helmet first, then HP)
- Respect invulnerability frames (already implemented) — no double-hits
- Enemy hitbox size: use 16x16 default for all enemies initially
- No player-kills-enemy yet (that requires jump-on-head detection) — enemies are indestructible hazards for now

**Acceptance criteria**:
- Walking into a Diamond or MiniHoppingSkull deals damage
- Invulnerability frames prevent rapid re-hits
- Losing all HP triggers death sequence
- Enemies remain alive after contact (player can't kill them yet)

---

### Phase 4: Basic Enemy Movement (Archetypes)

**Why fourth**: Static enemies are avoidable by memorisation. Movement makes them real threats.

**Requirements**:
- **Pacer** archetype: walk horizontally, reverse direction on wall collision or platform edge. Apply to MiniHoppingSkull
- **Static** archetype: no movement, just occupy position. Apply to Diamond (diamonds don't move in the original)
- Enemy terrain collision: pacers use `probe_wall()` and `probe_floor()` from the existing terrain module
- Enemy gravity: pacers fall and land on terrain like the player
- Movement speed: approximate from oracle traces or use a reasonable default (1-2 px/frame)

**Acceptance criteria**:
- MiniHoppingSkull walks back and forth on terrain, reversing at walls and ledges
- Diamond stays at spawn position
- Enemies interact with real terrain (don't walk through walls or float)

---

### Phase 5: Block Interactions

**Why fifth**: Blocks are parsed with full type info but have no gameplay effect. This phase makes the world reactive.

**Requirements (prioritised by Level 1 relevance)**:
- **Prize blocks**: player headbutts from below -> block ejects item (stub: just mark block as "hit"/empty for now, no item spawning yet)
- **Smash blocks**: player headbutts from below -> block destroyed (set cell to empty, update collision grid)
- Block state changes must update the live `BlockGrid` so collision reflects the new state
- Visual feedback: changed blocks should render differently (empty space where block was)

**Acceptance criteria**:
- Jumping into a prize block from below changes it to an empty/hit state
- Jumping into a smash block from below destroys it
- Collision grid updates immediately — player can pass through destroyed blocks
- Block grid rendering (Phase 7) reflects the changes

---

### Phase 6: Air Control

**Why here**: Without air control, jumps are committed at launch angle. The original game has horizontal acceleration while airborne (reduced from ground accel). This is needed for precise platforming.

**Requirements**:
- Apply horizontal acceleration from input while `on_ground == false`
- Use reduced acceleration factor vs ground (the speed tables already have this data)
- Respect max air speed
- Turning in air should work but with slower response than ground

**Acceptance criteria**:
- Player can steer left/right while jumping
- Air acceleration is noticeably slower than ground acceleration
- Matches original feel (verify against oracle trace if possible)

---

### Phase 7: Block Grid Rendering

**Why last**: The game is playable without this — the player collides with invisible terrain. But seeing the terrain transforms the experience from "debugging tool" to "game".

**Requirements**:
- Render each non-empty cell in the block grid as a coloured rectangle (one colour per collision type: solid=grey, spikes=red, slope=brown, etc.)
- Render only cells visible within the camera viewport (don't draw all 200x28 cells)
- Block type can influence colour as a secondary indicator (prize=yellow, smash=orange, ice=cyan, rubber=pink)
- No actual tile graphics needed — coloured rectangles with type colouring is sufficient for "playable"

**Acceptance criteria**:
- Level 1 terrain is visible on screen
- Different collision types are visually distinguishable
- Camera scrolling works correctly with rendered terrain
- Performance is acceptable (only visible cells drawn per frame)

---

## Out of Scope (for this PRD)

- Actual sprite/tile graphics (ROM art extraction and rendering)
- Background parallax scrolling
- Sound / music
- Helmet pickups and form-specific abilities
- Moving platforms (lifts, shifting blocks)
- Player killing enemies (jump-on-head, projectiles)
- Multiple level support beyond basic `advance_level()`
- Score, HUD, UI beyond debug text
- Ice/rubber terrain physics (terrain type switching)
- Checkpoints (respawn always at level start)

## Verification

Each phase should be verified by:
1. Manual playtest — does it feel right?
2. Oracle comparison where applicable — `compare-native-oracle` for position/velocity alignment
3. No regressions — existing movement and collision behaviour unchanged

## Definition of Done

Level 1 is "playable to a decent level" when a player can:
- Spawn at (1872, 224)
- Walk right on real, visible terrain
- Jump over gaps (falling in = death)
- Avoid or take hits from moving enemies
- Break smash blocks and hit prize blocks
- Reach the flag at (1872, 768)
- See "level complete" and advance