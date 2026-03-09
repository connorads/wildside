# Research Map

This map records what the source port still needs to learn, where to source that knowledge, and what the Rust target should look like.

## Development philosophy

**Progressive fidelity**: get it playable, then get it accurate. Every system starts approximate and is refined with oracle comparison. The oracle (jgenesis) is the safety net and primary testing tool.

**5 acceleration tracks** (see `acceleration-strategy.md` for full detail):
1. Block collision -- unlocks real terrain (highest priority)
2. Oracle-driven differential testing -- per-object state diffs for free TDD
3. Progressive fidelity enemies -- stub -> archetype -> full AI
4. Level traversal skeleton -- flag collision + level transitions for all 105 levels
5. LLM-assisted ASM translation -- workflow accelerator with mandatory oracle verification

## Done

### Boot and title
- Done: typed boot/title state machine (`BootState` enum)
- Done: mode flow, title timing, 6-press start handling
- Source: `kid.asm`, oracle checkpoints

### Level load
- Done: header parsing, asset pointers, `LevelBundle` loading from extracted assets
- Done: typed asset manifest with level headers, enemy layouts, platform layouts
- Source: `doc/formats.txt`, `level/header/00.bin`, generated asset manifest

### Player physics and camera
- Done: real input-driven physics from disassembly speed tables, jump formula, gravity, camera follow
- Done: `physics.rs` module with pure functions for horizontal velocity, jump, gravity, camera
- Done: speed tables for 3 terrain types x 3 form variants
- Source: `kid.asm` speed tables (`unk_9C0A`), `sub_A432` (jump), `kid.asm:14060` (gravity), `sub_71E4` (camera)

### Gameplay lifecycle
- Done: pause/resume, death windup/hiding/respawn settle
- Done: damage invulnerability (300 frames)
- Source: oracle gameplay checkpoints

### Object activation
- Done: enemy roster built from camera window, platform preload + activation window
- Source: `doc/ram_layout.txt`, enemy layouts

## Next: Block collision (Track 1)

### Terrain grid
- Need: parse level block data into `BlockGrid`, implement `probe_terrain` pure function
- Format: 2 bytes per cell at `$FFA652`, 3 bits collision type + 4 bits block type + 8 bits skin
- Source: `doc/ram_layout.txt`, `doc/formats.txt`, `level/block/*.bin`
- Rust target: `BlockGrid` type, `fn probe_terrain(x, y, grid) -> TerrainHit`
- Verify: oracle comparison of player positions on real terrain

### Player terrain integration
- Need: replace flat `floor_y_pos` with real terrain probing
- Source: collision routines in `kid.asm`
- Rust target: updated player update in `lib.rs` using `probe_terrain`

## Next: Level traversal (Track 4)

### Flag collision and level transitions
- Need: detect player reaching flag position, advance to next level
- Need: level ordering data, `player_start_x/y` from target level header
- Source: `kid.asm` completion flow, level-order data, `constants.asm`
- Rust target: flag detection + level transition state machine

## Next: Enemy system (Track 3)

### Enemy object model
- Need: `EnemyObject` with position, velocity, HP, facing, family-specific state
- Need: `fn update_enemy(enemy, world) -> enemy` pure function
- Rust target: `enemies.rs` module or `enemies/` tree

### Archetypes (cover ~60% of families)
- **Pacer**: walk back and forth, reverse on wall (Goat, Armadillo, Tank, Scorpion, Driller)
- **Hopper**: bounce with gravity, reverse on wall (Mini_Hopping_Skull, Big_Hopping_Skull, Crab)
- **Flyer**: sine-wave or linear path (Cloud, Dragon_Flying, Tornado)
- **Stationary attacker**: face player, project attacks (Archer, Fireball, Drips, Hand)
- Source: `code/enemy/*.asm`, oracle per-object traces

### Coroutine insight
- Enemy ASM uses cooperative multitasking (`j_Hibernate_Object_1Frame` yields, kernel resumes at saved PC)
- Translation to Rust requires converting implicit PC-as-state into explicit state enums
- Data-driven archetypes sidestep this for simple enemies

## Remaining player physics
- Air control (horizontal acceleration while airborne)
- Terrain type switching (ice, rubber) -- requires block collision first
- Y look-ahead camera offset (`$FFFFFB58`)
- Left movement oracle verification

## Deferred

### Platform collision
- Need: player-platform interaction (stand on, bump into)
- Need: platform movement scripts from `level/platform/*.asm`
- Blocked by: block collision (Track 1)

### Player forms
- Done: speed tables for all forms
- Remaining: form-specific abilities (Skycutter flight, Cyclone climb, etc.)
- Deferred until enemy system is working

### Audio and GEMS
- Deferred until gameplay is solid
- Source: `doc/sound.txt`, `.cache/references/gems`, `.cache/references/gemsplay`

### Bosses, menus, cutscenes, weather
- Deferred -- see `whole-game-index.md`

## Open questions

- What decompression is needed for block layouts? (Saxman/Enigma or already split?)
- How does the oracle helper need extending for per-object state capture?
- Which collision subroutines do enemies share? (`sub_36FF4`, `sub_36E84`, `sub_3C3CE`)
