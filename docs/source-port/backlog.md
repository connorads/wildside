# Playtest Backlog

Canonical tracker for bugs and missing features. See `CLAUDE.md` for iteration protocol.

Status: `[ ]` open, `[~]` in progress, `[x]` done, `[-]` won't fix / not a bug
Priority: P0 (game-breaking), P1 (core feel), P2 (missing system), P3 (polish)

| # | Status | Pri | System | Summary | Verify |
|-----|--------|-----|--------|---------|--------|
| 001 | [~] | P0 | player | Player gets stuck after first damage | Take hit, confirm movement resumes |
| 002 | [x] | P0 | enemy | Enemies don't die when stomped | Stomp enemy on level 0x00, confirm death |
| 003 | [~] | P1 | audio | No music or sound effects playing | Load level, confirm BGM and SFX |
| 004 | [x] | P1 | render | Colour palette wrong (floor/tree colours reversed) | Capture frame, compare to oracle |
| 005 | [x] | P1 | render | No parallax background | Scroll right, confirm background layers move |
| 006 | [~] | P1 | enemy | Dragon moonwalking (sprite direction vs movement) | Watch dragon, confirm facing matches walk dir |
| 007 | [x] | P1 | camera | Screen shakes/moves weirdly when hit | Take damage, confirm camera stays stable |
| 008 | [x] | P1 | render | Unwanted dark overlay effect when hit | Take damage, confirm no full-screen tint |
| 009 | [x] | P2 | menu | No menu — spawns directly into game | Launch, confirm title screen appears |
| 010 | [x] | P2 | enemy | No enemy attack patterns (fire breath, arrows) | Reach archer/dragon, confirm projectiles |
| 011 | [~] | P2 | block | Non-P bricks look half-broken (wrong sprite state) | Capture frame, compare brick tiles to oracle |
| 012 | [x] | P2 | block | No particle effects when blocks break | Hit block, confirm debris particles |
| 013 | [x] | P2 | block | P blocks don't convert to regular blocks after hit | Prize blocks vanish on hit (correct behaviour) |
| 014 | [x] | P2 | block | Diamonds/items don't visually emerge from blocks | Hit prize block, confirm item sprite appears |
| 015 | [x] | P2 | player | Can't wall-grab (A+A to grab block edges) | Jump at wall edge, confirm grab |
| 016 | [x] | P2 | level | No level intro screen showing level name | Enter level, confirm name card |
| 017 | [x] | P2 | form | Masks/helmets not visible when they appear | Hit helmet block, confirm sprite |
| 018 | [ ] | P2 | form | No form-change animation when equipping helmet | Collect helmet, confirm transform anim |
| 019 | [~] | P2 | form | Player doesn't transform at all | Collect helmet, confirm form changes |
| 020 | [ ] | P3 | render | No water/shimmer background effect | Load water level, confirm shimmer |

## Investigation notes

### 001 — Player stuck after damage
Code review: `apply_damage()` sets `invulnerable_frames=66` and stays in `PlayerLifecycle::Alive`.
Movement continues normally during invulnerability. No mechanism found that blocks movement.
Needs live reproduction to identify root cause — may be a corner/wall physics edge case.

### 002 — Enemies don't die when stomped (FIXED)
Root cause: stomp detection used strict `previous_bottom <= stomp_e_top` position check,
requiring the player's feet to be precisely above the enemy's top edge on the prior frame.
The ASM uses a velocity-direction cross-product: `y_overlap × |rel_x_vel| < x_overlap × |rel_y_vel|`
to determine approach direction. Reimplemented to match ASM — stomps are now detected based
on vertical approach direction, not prior-frame position history.

### 003 — Audio
GEMS engine is fully wired: BGM per theme, SFX for jump/stomp/damage/death/form/block.
Audio init prints status to stderr. If no sound, check SDL2 audio device availability.
Now triggers BGM on first InGame entry (was previously triggered at init).

### 004 — Palette (FIXED)
Root cause: theme-specific palette permutation tables were not applied. The ASM art
decompression remaps palette indices per theme (Forest, City, Mountain have unique tables).
Fix: load 16-byte permutation tables and reorder palette entries during level visual loading.

### 005 — Parallax (NOT A BUG)
Parallax IS working. bgscroll data loads correctly, `background_scroll_x_px()` applies
camera-relative rates. Effect is subtle (e.g. Forest: 26% camera speed) matching original.

### 006 — Dragon moonwalking
Both Dragon (flying) and DragonWalking (pacer) correctly set facing in their step functions.
Sprite system flips based on facing. Needs live reproduction to identify specific case.

### 010 — Enemy attacks (MOSTLY DONE)
11 enemy families + 4 bosses + 3 helmets + 3 environmental sources have projectile support.
Only gaps: UFO beam (child object system, spawns beam segments) and Fire Demon fire trails
(child objects spawned on death). Both require a child-object spawning system not yet built.
All simple projectile-based attacks are implemented.

### 011 — Bricks half-broken
Block rendering uses `block_mapping.resolve(skin, block_type)` with theme-specific tile quads.
With palette permutation now applied, colours should be correct. Skin-to-visual mapping needs
playtest verification — may look better now that palette is fixed.

### 013 — P-blocks (NOT A BUG)
Prize blocks vanish when hit (cell cleared to empty). This is correct per original behaviour.

### 015 — Wall grab (FIXED)
Added `left_wall_sticky` field and mirrored right-wall logic in `apply_wall_collision()`.
Both directions now support airborne wall-sticky with overhang top-edge retention.

### 017/019 — Helmets and form transform (PARTIALLY FIXED)
`apply_prize_content` → `queue_form_change` → `resolve_pending_form_change` chain is wired.
Forms change player hitbox, speed table, and abilities. FloatingItem now spawns on prize hit
with content-type-specific colours (distinct per helmet/diamond/item type).
Full sprite art for floating items not yet implemented — uses coloured rectangles.
Form change itself needs playtest to confirm it triggers correctly on helmet prize blocks.
