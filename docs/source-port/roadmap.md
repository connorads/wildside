# Long-Tail Roadmap: Finish Kid Chameleon Source Port

## Current approach

All 10 milestones below are done or substantially complete. The project is now in **playtest-driven iteration** — bugs and missing features are tracked in `docs/source-port/backlog.md`. See `CLAUDE.md` for the iteration protocol and `docs/source-port/methodology.md` for the deep validation methodology.

## Context

All 10 milestones are complete. The port is a full playable game: all 10 forms, 28 enemy families (ASM-verified), 4 bosses, full audio (GemsAudioEngine), 105 levels loadable, menus (title, pause, game over, continue, score, sound test), ending sequence, demo playback, Diamond Power (20 form/tier combos), per-helmet transform timing, and multi-level progression. 63/69 BCD levels pass oracle sweep at 0px tolerance (6 errors are jgenesis oracle bugs). 69/69 gameplay scenarios pass. Perf baseline tracking done (#031). What remains: 3 visual fidelity items (#033 transform animation, fire trail sprites, floating item sprites), 2 gameplay items (#037 floor-snap settling, #062 HeadyMetal death wobble), and 6 documented deviations (RNG, structural differences). See `research-map.md` for details.

---

## Milestone 1: Tier 1 Enemy Families + Archetype System — Done

**Goal:** Every early level has moving enemies. Refactor enemy dispatch into reusable archetypes.

### Tier 1 families — Pacer archetype

| obj ID | Family | ASM lines | Speed (FP) by HP | Timers ($48/$4A) |
|--------|--------|-----------|-------------------|------------------|
| `$10` | Goat | 192 | 3HP:`$18000` 4HP:`$28000` 5HP:`$38000` | 9/16 |
| `$04` | Armadillo | 196 | 1HP:`$8000` 2HP:`$18000` 3HP:`$28000` | —/— |
| `$0A` | Tank | 163 | (shared with shooting) | —/— |
| `$09` | Crab | 262 | 2HP:`$8000` 3HP:`$18000` | 12/30 |
| `$13` | Scorpion | 260 | fixed `$18000` | 9/22 |
| `$1A` | Driller | 185 | 2HP:`$8000` 3HP:`$10000` 4HP:`$18000` | 12/16 |

All use: walk at speed, probe floor ahead for edge, probe wall ahead for reversal, reverse facing on either.

### Work items
1. Add `Pacer` archetype — `step_pacer(enemy, block_grid)`: walk at `walk_speed`, call `probe_wall` and floor-edge detection, reverse facing. Parameterised by per-family constants.
2. Wire 6 families — add `EnemyFamily` variants with `from_layout_object` mapping from IDs `$10`, `$04`, `$0A`, `$09`, `$13`, `$1A`.
3. Per-family constants — extract speed/HP from ASM init routines. HP formula: `current_hp = enemy_level + N`.
4. Tank shooting (`$0B`) — fires projectiles at intervals. Add lightweight `EnemyProjectile` system.
5. Oracle per-enemy regression — extend `trace-oracle-enemy-layout` for scenarios beyond Level 1.
6. Tier 1 enemy sprites — extend `EnemyVisuals` in `kid-render`.

---

## Milestone 2: Block Interactions + Prize System — Done

**Goal:** Blocks do things — prizes drop, blocks break, ice is slippery.

### Work items
1. Prize block contents — parse prize-type from block data. Spawn correct pickup: diamond, clock, ankh, helmet, 1-up.
2. Pickup entities — ankh (extra life + invulnerability), clock (time bonus), coin (continue), helmet pickups. Diamond Power at 20/50 diamonds.
3. Evanescent blocks — solid -> fading -> gone -> regenerating cycle.
4. Terrain type physics — wire `PlayerForm` to correct speed table. Ice/rubber selection based on block type under player.
5. Smash blocks — full clear-on-hit.
6. Ghost blocks — timer-driven solid/vanished cycle.

---

## Milestone 3: Platform Collision + Multi-Theme Visuals — Done

**Goal:** Player can stand on platforms. Levels beyond Cave theme render correctly.

### Work items
1. Platform collision — AABB overlap, riding logic, head-bump from below.
2. Platform movement scripts — parse and execute `ptfm_move` commands.
3. Generalise visual pipeline — `load_level_visuals(manifest, level_id)` replacing Level 1 hardcoding.
4. Decode all 11 themes — extend `extract-assets` for all themes.
5. Background rendering per theme — chunked (Sky/Ice/Island/Swamp/Cave/City) and layered (Hills/Desert/Mountain/Forest).

---

## Milestone 4: Tier 2 Enemy Families — Done

**Goal:** Enemies that fly, hop, and shoot arrows.

| obj ID | Family | Archetype | Key constants |
|--------|--------|-----------|---------------|
| `$1D` | Big Hopping Skull | Hopper | HP table: 0->2, 1->3, 2->4 |
| `$0E` | Cloud | Flyer | x-vel: `-$3000`/`-$5000`/`-$7000` |
| `$06` | Sphere | Bouncer | gravity `+$800`/frame, bounces all surfaces |
| `$07` | Archer | Stationary Attacker | fire timers: `$1E`/`$14`/`$0A` |
| `$1B` | Emo Rock | Stationary | hover step `$2000`, random timer |
| `$19` | Fireball | Launcher | gravity by HP: `$1000`/`$2000`/`$3000` |

---

## Milestone 5: Helmet Abilities (Movement) — Done

**Goal:** Transform and conquer — the game's signature mechanic.

### Work items
1. Wire form-specific speed tables — select by `(form, terrain_type)` instead of hardcoded `NORMAL_KID`.
2. Helmet pickup logic — prize blocks with helmet content trigger `queue_form_change`.
3. Skycutter flight — horizontal flight with gradual descent.
4. Cyclone wall climb — wall contact detection, climb state, vertical movement.
5. IronKnight climb — shares Cyclone infrastructure, different parameters.
6. Micromax wall/ceiling stick — gravity-direction state.
7. Juggernaut charge — high-momentum movement, breaks certain blocks.

---

## Milestone 6: Native Audio — Done

**Goal:** The game has a voice. Transformative quality-of-life.

Architecture: reuse Z80+GEMS binary from ROM via existing `z80` crate + `ymfm-sys` for perfect audio fidelity.

### Work items
1. Study GEMS command protocol — map game events to GEMS command IDs.
2. Audio subsystem in native player — SDL2 AudioQueue<f32>.
3. Z80+GEMS integration — instantiate Z80 with GEMS driver, YM2612, PSG.
4. Game event -> GEMS mapping — jump, land, damage, death, level complete, etc.
5. Music state machine — play on level load, pause, stop on death, crossfade on transition.
6. Audio mixing — FM + PSG -> SDL2 audio queue at 44100 Hz stereo.

**Note:** largely independent — can run in parallel with other milestones.

---

## Milestone 7: Tier 3-4 Enemies + Bosses — Done

**Goal:** The full bestiary. Boss fights are playable.

### Tier 3 (275-500 ASM lines each)
Ninja, Dragon (walking), Orca, Hand, Robot, Spinning Twins, Tar Monster

### Tier 4 (280-1300 ASM lines)
Dragon Flying, Lion, UFO (1294 lines), Drips (1308 lines), Fire Demon, Tornado

### Bosses
HeadyMetal (L$05), Shishkaboss (L$10), BoomerangBosses (L$1E), BagelBrothers (L$2A)

---

## Milestone 8: Helmet Abilities (Combat) + Remaining Blocks — Done

**Goal:** Full combat toolkit, full block vocabulary.

Combat: RedStealth sword, Eyeclops beam, Maniaxe throw, Berzerker push.
Blocks: Teleporter, Elevator/Lift, Shifting, Shooter, Hidden, Drill, Mushroom, Berzerker-pushable.

---

## Milestone 9: Weather, Murder Wall, Camera — Done

**Goal:** Environmental hazards complete the atmosphere.

Murder wall, weather effects (lava/storm/hail/snow), camera Y look-ahead, lava geyser blocks.

---

## Milestone 10: Menus, Progression, Polish — Done

**Goal:** A complete game from title to credits.

Done: game over/continue screen, score system, Diamond Power (20 form/tier combos), ending sequence, 105 levels loadable, full multi-level progression, demo playback (2 demos from ROM data), intro sequence (5 segments), per-helmet transform timing.
Remaining: transform animation visual sprites (#033 — needs ROM tile extraction).

---

## Dependency Graph

```
M1 (Tier 1 Enemies) ───┐
                        ├──► M4 (Tier 2 Enemies) ──► M7 (Tier 3-4 + Bosses)
M2 (Blocks + Prizes) ──┤                                       │
                        ├──► M5 (Helmets: Movement) ──► M8 (Helmets: Combat + Blocks)
M3 (Platforms + Themes)─┘                                       │
                                                                ▼
M6 (Audio) ─── independent, parallel ──────────────► M9 (Weather, Murder Wall)
                                                                │
                                                                ▼
                                                     M10 (Menus, Polish)
```
