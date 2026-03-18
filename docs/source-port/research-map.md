# Research Map

This map records what the source port still needs to learn, where to source that knowledge, and what the Rust target should look like.

## Development philosophy

**Progressive fidelity**: get it playable, then get it accurate. Every system starts approximate and is refined with oracle comparison. The oracle (`jgenesis`) is the safety net and primary testing tool.

## Current frontier

All 10 original milestones are done. The project is now in **playtest-driven iteration** — see `backlog.md` for the canonical bug/feature tracker and `CLAUDE.md` for the iteration protocol.

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

## Previously deferred (now done)

- Platform collision — done (static + moving, riding logic, movement scripts)
- Player forms — done (speed tables, form-specific abilities wired)
- Audio and GEMS — done (GemsAudioEngine: Z80+YM2612+PSG from ROM)
- Tier 3-4 enemies + bosses — done (all 28 families, 26 with real sprite art)

## Validation methodology

### Oracle comparison (correctness)
- jgenesis as ground truth, `DynamicScenario` per-frame button masks
- Sweep: 105 levels x idle scenario, state-level checkpoint comparison
- Position tolerance: 6px. Advisory fields masked.

### Pixel-perfect frame comparison (visual)
- 320x224 fixed-palette — exact pixel matching, `image` crate RGBA comparison
- `dify` CLI for ad-hoc diff

### Input replay + snapshot testing (regression)
- `DynamicScenario` for replay, `insta` for snapshot regression
- See `CLAUDE.md` for full details

## Open questions

- FloatingItem spawning: `prize_block_hit` doesn't record position — items never appear visually
- Multi-screen foreground: some levels reference block layout for level 0x00 (asset extraction gap)
- 6 oracle warp-broken levels (oracle crashes during gameplay: 0x09, 0x32, 0x33, 0x50, 0x57, 0x63)
- Demo playback: no source data found in disassembly
