# Research Map

This map records what the source port still needs to learn, where to source that knowledge, and what the Rust target should look like.

## Development philosophy

**Progressive fidelity**: get it playable, then get it accurate. Every system starts approximate and is refined with oracle comparison. The oracle (`jgenesis`) is the safety net and primary testing tool.

## Current frontier

1. Level 1 enemy fidelity
2. Oracle-driven per-enemy checkpoint comparison
3. Matching native state against the same observable runtime object surface the oracle exposes
4. Then broader Level 1 gameplay expansion

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

### Object activation and tracing
- Done: enemy roster built from camera window, platform preload + activation window
- Done: native trace surface now records stable enemy layout identity and motion/state fields
- Source: `doc/ram_layout.txt`, enemy layouts

## Current next: Level 1 enemy fidelity

### Native family behaviour
- Done: `Diamond` remains stationary and collectible
- Done: `MiniHoppingSkull` now uses a dedicated hopper loop in the native runtime
- Need: tighten behaviour against oracle-visible timing and contact windows

### Oracle object capture
- Done: shared oracle/native checkpoint schema now includes per-enemy snapshots
- Done: new `level1_enemy_window` scenario exists for the first Level 1 enemy slice
- Need: map oracle runtime gfx objects cleanly back to stable enemy identity/family
- Need: align native compare against the same observable enemy surface rather than the broader activated roster
- Source: `code/enemy/*.asm`, `doc/ram_layout.txt`, `variables.asm`, `.cache/native-oracle-runs/`

### Remaining enemy archetype work
- Next families after this cycle should reuse the same seam:
  - pacers
  - flyers
  - stationary attackers
- Source: `code/enemy/*.asm`, oracle per-object traces

## Remaining player/gameplay work

- Air control (horizontal acceleration while airborne)
- Terrain type switching (ice, rubber)
- Y look-ahead camera offset (`$FFFFFB58`)
- Flag collision and level transitions
- Block interactions beyond the current Level 1 slice

## Deferred

### Platform collision
- Need: player-platform interaction (stand on, bump into)
- Need: platform movement scripts from `level/platform/*.asm`
- Blocked by: tighter terrain/object parity work

### Player forms
- Done: speed tables for all forms
- Remaining: form-specific abilities (Skycutter flight, Cyclone climb, etc.)
- Deferred until the enemy/object surface is trustworthy

### Audio and GEMS
- Deferred until gameplay is solid
- Source: `doc/sound.txt`, `.cache/references/gems`, `.cache/references/gemsplay`

### Bosses, menus, cutscenes, weather
- Deferred; see `whole-game-index.md`

## Open questions

- Which linked object/layout fields in oracle runtime RAM cleanly identify visible enemy families?
- Which native enemies should be compared as activated roster members versus oracle-visible runtime objects?
- Which collision/behaviour subroutines do the next Level 1 enemy families share? (`sub_36FF4`, `sub_36E84`, `sub_3C3CE`)
