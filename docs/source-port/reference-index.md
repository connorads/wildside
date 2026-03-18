# Source-Port Reference Index

## Local truth

- `.cache/upstream/kid-chameleon-disasm`
  - pinned upstream disassembly and bundled docs
- `.cache/generated/<rom-sha1>/asset-manifest.json`
  - typed extracted asset index used by the native runtime
- `.cache/native-oracle-runs/<scenario>/`
  - native checkpoint captures and oracle state for debugging parity
- `.cache/oracles/`
  - `jgenesis` helper and other oracle repos

## 68k semantics

- `.cache/references/m68k-instructions-documentation`
  - primary 68k instruction reference
- official manual fallback
  - https://www.nxp.com/docs/en/reference-manual/M68000PRM.pdf

## AS assembler

- `.agents/skills/as-assembler/SKILL.md`
  - entry point for AS syntax, directives, macros, and file tools
- `.agents/skills/as-assembler/references/`
  - command-line, pseudo-instruction, expression, and tool docs

## Kid bundled docs

- `.cache/upstream/kid-chameleon-disasm/doc/ram_layout.txt`
- `.cache/upstream/kid-chameleon-disasm/doc/formats.txt`
- `.cache/upstream/kid-chameleon-disasm/doc/sound.txt`
- `.cache/upstream/kid-chameleon-disasm/doc/vram_layout.txt`
- `.cache/upstream/kid-chameleon-disasm/variables.asm`
- `.cache/upstream/kid-chameleon-disasm/constants.asm`
- `.cache/upstream/kid-chameleon-disasm/kid.asm`

## GEMS and audio

- `.cache/references/gems`
  - GEMS tooling and driver research
- `.cache/references/gemsplay`
  - GEMS playback and inspection
- `.cache/upstream/kid-chameleon-disasm/doc/sound.txt`
  - Kid-specific sound notes

## Mega Drive hardware

- https://segaretro.org/Mega_Drive_official_documentation
- https://segaretro.org/Sega_Mega_Drive/VDP_general_usage
- https://segaretro.org/Sega_Mega_Drive/Palettes_and_CRAM
- https://segaretro.org/Enigma_compression

## Methodology

- `docs/source-port/methodology.md`
  - investigation playbook, testing taxonomy, regression discipline, cross-referencing protocol
  - metamorphic relations, mutation testing, divergence classification, fidelity metrics

## Source-port exemplars

- https://github.com/n64decomp/sm64
