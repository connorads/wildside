# Playtest Backlog

Canonical tracker for bugs and missing features. See `CLAUDE.md` for iteration protocol.

Status: `[ ]` open, `[~]` in progress, `[x]` done
Priority: P0 (game-breaking), P1 (core feel), P2 (missing system), P3 (polish)

| # | Status | Pri | System | Summary | Verify |
|-----|--------|-----|--------|---------|--------|
| 001 | [ ] | P0 | player | Player gets stuck after first damage | Take hit, confirm movement resumes |
| 002 | [ ] | P0 | enemy | Enemies don't die when stomped | Stomp enemy on level 0x00, confirm death |
| 003 | [~] | P1 | audio | No music or sound effects playing | Load level, confirm BGM and SFX |
| 004 | [ ] | P1 | render | Colour palette wrong (floor/tree colours reversed) | Capture frame, compare to oracle |
| 005 | [ ] | P1 | render | No parallax background | Scroll right, confirm background layers move |
| 006 | [ ] | P1 | enemy | Dragon moonwalking (sprite direction vs movement) | Watch dragon, confirm facing matches walk dir |
| 007 | [x] | P1 | camera | Screen shakes/moves weirdly when hit | Take damage, confirm camera stays stable |
| 008 | [x] | P1 | render | Unwanted dark overlay effect when hit | Take damage, confirm no full-screen tint |
| 009 | [x] | P2 | menu | No menu — spawns directly into game | Launch, confirm title screen appears |
| 010 | [~] | P2 | enemy | No enemy attack patterns (fire breath, arrows) | Reach archer/dragon, confirm projectiles |
| 011 | [ ] | P2 | block | Non-P bricks look half-broken (wrong sprite state) | Capture frame, compare brick tiles to oracle |
| 012 | [x] | P2 | block | No particle effects when blocks break | Hit block, confirm debris particles |
| 013 | [x] | P2 | block | P blocks don't convert to regular blocks after hit | Prize blocks vanish on hit (correct behaviour) |
| 014 | [x] | P2 | block | Diamonds/items don't visually emerge from blocks | Hit prize block, confirm item sprite appears |
| 015 | [ ] | P2 | player | Can't wall-grab (A+A to grab block edges) | Jump at wall edge, confirm grab |
| 016 | [x] | P2 | level | No level intro screen showing level name | Enter level, confirm name card |
| 017 | [~] | P2 | form | Masks/helmets not visible when they appear | Hit helmet block, confirm sprite |
| 018 | [ ] | P2 | form | No form-change animation when equipping helmet | Collect helmet, confirm transform anim |
| 019 | [~] | P2 | form | Player doesn't transform at all | Collect helmet, confirm form changes |
| 020 | [ ] | P3 | render | No water/shimmer background effect | Load water level, confirm shimmer |

## Investigation notes

### 001 — Player stuck after damage
Code review: `apply_damage()` sets `invulnerable_frames=66` and stays in `PlayerLifecycle::Alive`.
Movement continues normally during invulnerability. No mechanism found that blocks movement.
Needs live reproduction to identify root cause — may be a corner/wall physics edge case.

### 002 — Enemies don't die when stomped
Code review: `process_stomps()` correctly decrements HP and removes at 0. The stomp window
requires `pre_physics_y_velocity > 0` and `previous_bottom <= stomp_e_top`. This is strict
but correct per ASM. May feel harder than original due to FP16.16 position precision.

### 003 — Audio
GEMS engine is fully wired: BGM per theme, SFX for jump/stomp/damage/death/form/block.
Audio init prints status to stderr. If no sound, check SDL2 audio device availability.
Now triggers BGM on first InGame entry (was previously triggered at init).

### 006 — Dragon moonwalking
Both Dragon (flying) and DragonWalking (pacer) correctly set facing in their step functions.
Sprite system flips based on facing. Needs live reproduction to identify specific case.

### 010 — Enemy attacks
DragonWalking breath, Drips drops, Archer arrows, and boss retaliation projectiles are wired.
Mark as "investigate" — some families may still be missing projectile spawning.

### 013 — P-blocks
Prize blocks vanish when hit (cell cleared to empty). This is correct per original behaviour.
The block was already gone; the playtest may have expected a "used block" visual state.

### 017/019 — Helmets and form transform
`apply_prize_content` → `queue_form_change` → `resolve_pending_form_change` chain is wired.
Forms change player hitbox, speed table, and abilities. FloatingItem now spawns on prize hit.
The visual helmet sprite in the floating item is generic (coloured square) — not a helmet image.
Needs playtest to confirm form change actually triggers on helmet prize blocks.
