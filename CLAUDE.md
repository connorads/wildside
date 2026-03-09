# Source-Port Reference Guide

This repo keeps source-port research material in cache-local checkouts and repo docs. Treat this file as the canonical guide for finding the right reference fast. `AGENTS.md` should point here as a symlink.

## Source precedence

Use sources in this order:

1. Local Kid truth
- `.cache/upstream/kid-chameleon-disasm`
- `.cache/generated/<rom-sha1>/asset-manifest.json`
- `.cache/native-oracle-runs/<scenario>/`
- `kidcli inspect-level`, `inspect-enemy`, `probe-native`, `compare-native-oracle`

2. Kid bundled docs
- `.cache/upstream/kid-chameleon-disasm/doc/ram_layout.txt`
- `.cache/upstream/kid-chameleon-disasm/doc/formats.txt`
- `.cache/upstream/kid-chameleon-disasm/doc/sound.txt`
- `.cache/upstream/kid-chameleon-disasm/variables.asm`
- `.cache/upstream/kid-chameleon-disasm/constants.asm`
- `.cache/upstream/kid-chameleon-disasm/kid.asm`

3. Cached research repos
- `.cache/references/m68k-instructions-documentation`
- `.cache/references/gems`
- `.cache/references/gemsplay`

4. Local skills and repo docs
- `.agents/skills/as-assembler/SKILL.md`
- `docs/source-port/reference-index.md`
- `docs/source-port/research-map.md`
- `docs/source-port/level-1-dossier.md`
- `docs/source-port/whole-game-index.md`

5. Web/manual fallback
- Sega Retro Mega Drive hardware docs
- official M68000 manual

## Cache layout

- `.cache/upstream/`
  - Kid-specific upstream source repos
- `.cache/generated/`
  - extracted typed asset bundles
- `.cache/native-oracle-runs/`
  - native vs oracle checkpoint artefacts
- `.cache/oracles/`
  - emulator/oracle repos and helpers
- `.cache/references/`
  - generic research repos used for source-port work
- `.cache/tooling/`
  - built tools such as ASL

## Which source to use

- Unknown 68k opcode or addressing mode
  - use `.cache/references/m68k-instructions-documentation` first
- Unknown AS directive, macro, or pseudo-op
  - use `.agents/skills/as-assembler/SKILL.md` and its `references/` docs
- Unknown Kid RAM variable or object-state field
  - use `variables.asm` and `doc/ram_layout.txt`
- Unknown level/header/enemy/platform format
  - use `doc/formats.txt`, `level/*`, and `kidcli inspect-level`
- Unknown enemy family logic
  - use `code/enemy/*.asm`, enemy layouts, and oracle checkpoints
- Unknown GEMS or audio bank behaviour
  - use `doc/sound.txt`, `.cache/references/gems`, and `.cache/references/gemsplay`
- Unknown Mega Drive hardware effect
  - use Kid asm first, then Sega Retro hardware docs

## First commands

```bash
mise run bootstrap-upstream
mise run bootstrap-references
mise run oracle-setup
mise run extract-assets
cargo run -p kidcli -- inspect-level --level 00
cargo run -p kidcli -- inspect-enemy --enemy 01
cargo run -p kidcli -- compare-native-oracle --scenario boot-title-start-level1
```

## Level 1 reading order

1. `docs/source-port/level-1-dossier.md`
2. `.cache/upstream/kid-chameleon-disasm/doc/formats.txt`
3. `.cache/upstream/kid-chameleon-disasm/doc/ram_layout.txt`
4. `.cache/upstream/kid-chameleon-disasm/kid.asm`
5. `.cache/upstream/kid-chameleon-disasm/code/enemy/*.asm` for the first visible enemy families
6. `.cache/native-oracle-runs/boot_title_start_level1/`
