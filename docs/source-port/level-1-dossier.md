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
cargo run -p kidcli -- inspect-enemy --enemy 01
cargo run -p kidcli -- compare-native-oracle --scenario boot-title-start-level1
cargo run -p kidcli -- compare-native-oracle --scenario boot-title-start-level1-move-right5s
```

## Known level-1 facts

- level id: `0x00`
- display name: `Blue Lake Woods 1`
- header size: `6x4` screens
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

- The opening walk now uses explicit native player, camera, terrain, and platform-loader state in `kid-core`; the Level 1 slice no longer depends on the old scripted movement helper.
- The opening terrain rule represented natively for this slice is a flat floor clamp at the recovered grounded Y used by the oracle-backed walk segment.
- The visible Level 1 enemy-family mapping represented natively for the opening slice is `Diamond` plus slot-decoded support for `MiniHoppingSkull`.
- The Level 1 platform path is represented natively as preload plus activation-window logic over the parsed `level/platform/00.asm` placements.
- There are no remaining open Level 1 research tasks in this dossier for the opening slice milestone.
