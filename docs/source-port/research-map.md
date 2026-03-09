# Research Map

This map records what the source port still needs to learn, where to source that knowledge, and what the Rust target should look like.

## Level 1 first

### Boot and title
- Need: mode flow, title timing, start handling
- Source: `kid.asm`, oracle checkpoints, current bootstrap trace
- Rust target: typed boot/title state machine

### Level load
- Need: header fields, asset pointers, initial map state
- Source: `doc/formats.txt`, `level/header/00.bin`, generated asset manifest
- Rust target: typed level-load pipeline and `LevelBundle`

### Player and camera
- Done: real input-driven physics from disassembly speed tables, jump formula, gravity, camera follow
- Done: `physics.rs` module with pure functions for horizontal velocity, jump, gravity, camera
- Remaining: air control, terrain-type switching, Y look-ahead offset, left movement oracle verification
- Source: `kid.asm` speed tables (`unk_9C0A`), `sub_A432` (jump), `kid.asm:14060` (gravity), `sub_71E4` (camera)
- Rust target: `PlayerState`, `CameraState`, `physics::*` pure functions

### Object activation
- Need: object slot lifecycle, active window, enemy status bookkeeping
- Source: object RAM notes in `doc/ram_layout.txt`, enemy layouts, object manager routines
- Rust target: object scheduler and active roster

### Platforms
- Need: layout semantics, loader state, movement scripts
- Source: `level/platform/*.asm`, platform manager RAM vars, relevant asm routines
- Rust target: platform placement parser plus platform behaviour system

### First enemy families
- Need: init/update state machines and activation rules for level 1 families
- Source: `code/enemy/*.asm`, enemy layouts, oracle traces
- Rust target: one Rust module per enemy family

## Whole-game lighter map

### Player forms
- Source: `kid.asm`, constants, helmet IDs, player routines
- Rust target: form-specific ability/state modules

### Enemy families and bosses
- Source: `code/enemy/*.asm`
- Rust target: per-family state machines with shared traits where useful

### Menus and cutscenes
- Source: `scenes/`, `kid.asm`
- Rust target: mode-specific UI/cutscene systems

### Weather and special effects
- Source: level headers, theme assets, special-effect vars and routines
- Rust target: render/effect commands driven by game state

### Audio
- Source: `doc/sound.txt`, `.cache/references/gems`, `.cache/references/gemsplay`
- Rust target: music/SFX event model first, native playback later

## Open questions to keep tracking

- Which platform scripts are needed for the first playable slice?
- Which RAM fields are semantic state vs transient hardware queues?
- Which rendering differences are gameplay-relevant vs cosmetic?
- How much of GEMS needs semantic parity before native playback becomes worthwhile?
