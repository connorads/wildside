# Research Map

This map records what the source port still needs to learn, where to source that knowledge, and what the Rust target should look like.

## Development philosophy

**Progressive fidelity**: get it playable, then get it accurate. Every system starts approximate and is refined with oracle comparison. The oracle (`jgenesis`) is the safety net and primary testing tool.

## Current frontier

1. Tier 1 enemy families (Pacer archetype: Goat, Armadillo, Tank, Crab, Scorpion, Driller) — **done**
2. Block interactions + prize system + terrain type physics — **terrain type done**, block interactions in progress
3. Platform collision + multi-theme visual generalisation — **static + moving platform collision done**
4. Tier 2 enemy families (Cloud, Sphere, BigHoppingSkull, EmoRock, Archer, Fireball) — **done**
5. Tier 3-4 enemy families (all 28 families wired) — **done**
6. See `roadmap.md` for the full 10-milestone plan

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

### Level 1 visuals
- Done: manifest-backed Level 1 visual loading and asset-backed native scene output
- Done: ROM-backed HUD, player, background, terrain, visible platforms, and visible Level 1 families on the default render path
- Source: generated bundle contract, `kid-render`, `kidcli::native_player`

### Object activation and tracing
- Done: enemy roster built from camera window, platform preload + activation window
- Done: native trace surface now records stable enemy layout identity and motion/state fields
- Source: `doc/ram_layout.txt`, enemy layouts

## Traversal closure

### Default runtime terrain authority
- Done: `NativeRuntime::new` now reloads generated `Level_Layout` terrain on construction
- Done: the default runtime path no longer depends on a separate explicit generated-terrain mode for Level 1 gameplay
- Source: `kid-core`, generated Level 1 bundle, `doc/formats.txt`

### Traversal parity
- Done: `compare-native-gameplay --scenario progress-two-transitions` passes on the default path
- Done: pause/resume and death/respawn stay green on the same default path
- Done: the long traversal script now takes the first opening Dragon hit on the oracle frame (`749`) under walk-only input as well as the existing run-held regression
- Done: `diagnose-native-gameplay-drift --scenario progress-two-transitions --from-frame 719` now reports no live drift after the skip choreography and Level 2 transition-arc fixes
- Source: `.cache/native-gameplay-oracle-runs/progress_two_transitions/`, gameplay comparison harness

### Enemy fidelity after traversal
- `Diamond` remains stationary and collectible
- `MiniHoppingSkull` uses a hopper loop in the native runtime
- Done: Pacer archetype for 6 Tier 1 families (Goat, Armadillo, Tank, Crab, Scorpion, Driller)
- Done: family-specific HP initialisation from ASM init routines
- Done: Cloud (horizontal flight + wall reversal), Sphere (gravity bounce), BigHoppingSkull (parameterised hopper)
- Done: Fireball (gravity bounce with level-dependent gravity), EmoRock/Archer (Inert stubs)
- Done: All 28 enemy families wired — Tier 3-4 mapped to closest archetypes (Pacer/Flyer/Hopper/Bouncer/Inert)
- Source: `code/enemy/*.asm`, oracle per-object traces

## Remaining player/gameplay work

- Terrain type switching (ice, rubber)
- Y look-ahead camera offset (`$FFFFFB58`)
- Block interactions beyond the current traversal slice
- Broader multi-level progression beyond the current transition scenario

## Deferred

### Platform collision
- Need: player-platform interaction (stand on, bump into)
- Need: platform movement scripts from `level/platform/*.asm`
- Blocked by: tighter terrain/object parity work

### Player forms
- Done: speed tables for all forms
- Done: form-specific ground speed table wiring (Micromax/Juggernaut use distinct tables)
- Remaining: form-specific abilities (Skycutter flight, Cyclone climb, etc.)
- Deferred until the enemy/object surface is trustworthy

### Audio and GEMS
- Deferred until gameplay is solid
- Source: `doc/sound.txt`, `.cache/references/gems`, `.cache/references/gemsplay`

### Bosses, menus, cutscenes, weather
- Deferred; see `whole-game-index.md`

## Open questions

- Which levels have the most block interactions to exercise first for oracle regression?
- Platform collision: what is the minimal set of levels to exercise riding + movement scripts?
