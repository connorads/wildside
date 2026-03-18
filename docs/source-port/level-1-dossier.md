# Level 1 Dossier

## Scope

This dossier covers the first vertical slice: boot to title, title to level load, early in-level settle, first scroll, first visible enemies, and first visible platforms.

## Local files to open first

- .cache/upstream/kid-chameleon-disasm/kid.asm
- .cache/upstream/kid-chameleon-disasm/doc/formats.txt
- .cache/upstream/kid-chameleon-disasm/doc/ram_layout.txt
- .cache/upstream/kid-chameleon-disasm/variables.asm
- .cache/upstream/kid-chameleon-disasm/constants.asm
- .cache/upstream/kid-chameleon-disasm/level/header/00.bin
- .cache/upstream/kid-chameleon-disasm/level/enemy/00.bin
- .cache/upstream/kid-chameleon-disasm/level/platform/00.asm
- .cache/upstream/kid-chameleon-disasm/code/enemy/ for the first active enemy modules

## Useful commands

```bash
cargo run -p kidcli -- inspect-level --level 00
cargo run --release -p kidcli -- play-native
cargo run -p kidcli -- verify-native-bootstrap
cargo run -p kidcli -- verify-native-gameplay --scenario pause-resume-level1
cargo run -p kidcli -- inspect-enemy --enemy 01
# optional spot-checks once the native path looks right
cargo run -p kidcli -- compare-native-oracle --scenario boot-title-start-level1
cargo run -p kidcli -- compare-native-oracle --scenario boot-title-start-level1-move-right5s
```

## Known level-1 facts

- level id: `0x00`
- display name: `Blue Lake Woods 1`
- header size: `6x4` screens (120x56 cells, 1920x896 px)
- player start: `1872,224`
- flag: `1872,768`
- bgscroll entries: 49 layers
- enemy layout object count: 22
- platform layout currently exposes one parsed placement in the split data

## Current oracle checkpoints in use

- `title@360`
- `level_entry@520`
- `level_settle@719`
- `move_mid@840`
- `move_end@1019`

## Rust-side targets

- `BootState` and title flow
- `LevelBundle` loading from extracted assets
- `PlayerState` with oracle-aligned fixed-point position
- `CameraState`
- active object roster with player, enemies, and platforms
- first enemy family behaviour modules

## Level-1 closure notes

- Player movement is now physics-driven via `kid-core::physics`: real horizontal acceleration/decel from the disassembly speed tables (`unk_9C0A`), jump initiation from `sub_A432`, variable gravity from `kid.asm:14060`, and camera follow from `sub_71E4`.
- The old scripted frame-count interpolation (`run_camera_x_for_hold_frames`, `OpeningScrollState`) has been removed -- the player moves from input, the camera follows.
- Speed tables cover all 3 terrain types (normal/ice/rubber) and 3 form variants (Kid/Micromax/Juggernaut). Level 1 uses `NORMAL_KID`.
- Jump height per form is transcribed from `unk_A49E`; RedStealth has a higher factor ($4A vs $42).
- The visible Level 1 enemy-family mapping is `Diamond` plus slot-decoded `MiniHoppingSkull`.
- Platform paths use preload plus activation-window logic over `level/platform/00.asm` placements.
- `play-native` now renders the decoded foreground layout as the base Level 1 scene, then layers live block overrides, sprites, and HUD on top so the opening area is readable without relying on oracle artefacts.
- This opening-slice dossier is complete; the next tracked frontier lives in `docs/source-port/level-1-full-traversal.md`.

## Next for Level 1: full traversal

The default native runtime still has a flat-floor fallback when no `BlockGrid` is loaded. Removing that fallback on the main path is the first blocker for the traversal milestone because it unlocks:
- Player walking on actual Level 1 terrain (slopes, gaps, platforms)
- Enemy terrain interaction (pacers reversing on walls, hoppers bouncing on ground)
- Flag reachability (flag is at y=768, below the flat floor at y=224)

Block format: 2 bytes per cell at `$FFA652`, documented in `doc/ram_layout.txt`.

### Remaining physics work

- Left movement, bidirectional turning (currently works but untested against oracle)
- Terrain type switching (ice, rubber) -- done at engine level, fidelity verification pending
- Form-specific speed table selection (Micromax, Juggernaut) -- done
- Y look-ahead camera offset (`$FFFFFB58`) -- done (+1/frame down+grounded, -8/frame decay, applied at >=30px)
