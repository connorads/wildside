# Level 1 Dossier

## Scope

This dossier covers the first vertical slice: boot to title, title to level load, early in-level settle, first scroll, first visible enemies, and first visible platforms.

## Local files to open first

- `.cache/upstream/kid-chameleon-disasm/kid.asm`
- `.cache/upstream/kid-chameleon-disasm/doc/formats.txt`
- `.cache/upstream/kid-chameleon-disasm/doc/ram_layout.txt`
- `.cache/upstream/kid-chameleon-disasm/variables.asm`
- `.cache/upstream/kid-chameleon-disasm/constants.asm`
- `.cache/upstream/kid-chameleon-disasm/level/header/00.bin`
- `.cache/upstream/kid-chameleon-disasm/level/enemy/00.bin`
- `.cache/upstream/kid-chameleon-disasm/level/platform/00.asm`
- `.cache/upstream/kid-chameleon-disasm/code/enemy/` for the first active enemy modules

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

## Next level-1 research tasks

1. Identify the exact enemy families visible in the first movement slice and map them to `code/enemy/*.asm`.
2. Trace the platform manager routines that consume `level/platform/00.asm`.
3. Replace the remaining scripted camera/player heuristics with recovered movement logic.
4. Record the first collision and terrain rules needed for the opening walk segment.
