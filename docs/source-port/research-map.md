# Research Map

This map records what the source port still needs to learn, where to source that knowledge, and what the Rust target should look like.

## Development philosophy

**Progressive fidelity**: get it playable, then get it accurate. Every system starts approximate and is refined with oracle comparison. The oracle (`jgenesis`) is the safety net and primary testing tool.

## Current frontier

1. Level 1 full traversal
2. Real terrain authority on the default native runtime path
3. Flag completion and next-level transition parity via `progress-two-transitions`
4. Then deeper enemy-family fidelity work where traversal still exposes gaps

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

## Current next: Level 1 full traversal

### Traversal authority
- Need: remove the default flat-floor fallback from the main native runtime path
- Need: source collision authority from real terrain data instead of test-only grid injection
- Source: `kid-core`, generated Level 1 bundle, `doc/formats.txt`

### Traversal parity
- Need: make `compare-native-gameplay --scenario progress-two-transitions` pass
- Current failure: `progress_mid@1999` diverges in game mode (`native 000C`, `oracle 0008`)
- Source: `.cache/native-gameplay-oracle-runs/progress_two_transitions/`, `kid.asm`, gameplay comparison harness

### Enemy fidelity after traversal
- `Diamond` remains stationary and collectible
- `MiniHoppingSkull` uses a hopper loop in the native runtime
- Next deeper fidelity work should resume after traversal parity stops diverging
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
- Remaining: form-specific abilities (Skycutter flight, Cyclone climb, etc.)
- Deferred until the enemy/object surface is trustworthy

### Audio and GEMS
- Deferred until gameplay is solid
- Source: `doc/sound.txt`, `.cache/references/gems`, `.cache/references/gemsplay`

### Bosses, menus, cutscenes, weather
- Deferred; see `whole-game-index.md`

## Open questions

- Which generated Level 1 asset path should become the default source of runtime terrain authority?
- What is the first semantic cause of the `progress-two-transitions` game-mode divergence at `progress_mid@1999`?
- After traversal parity lands, which enemy-family gaps remain visible enough to justify the next oracle-fidelity cycle?
