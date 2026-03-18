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

## Iteration workflow

Canonical bug/feature backlog: `docs/source-port/backlog.md`

### Principles (bsnes methodology)
- Fix behaviours at engine level, never level-specific overrides
- Every fix must pass full sweep — if it fixes level 1 but breaks level 23, the fix is wrong
- Every bug fixed should become a permanent regression test (scenario or snapshot)
- Advisory fields (score, lives, diamonds, number_objects, murder_wall_x) are expected divergences, not bugs

### Session protocol
1. Read `docs/source-port/backlog.md`; pick the highest-priority open item
2. Mark `[~]` when starting work
3. Investigate: read relevant code, capture frames, view PNGs with Read tool
4. Fix: implement in focused commit(s)
5. Verify visually: capture frames, Read PNGs, compare before/after
6. Verify mechanically: `mise run verify-slice-fast` must pass
7. Mark `[x]` in backlog (same commit as the fix)
8. After 3-5 fixes: playtest with `mise run play-native`, triage new bugs

## Visual debugging

The Read tool supports viewing PNG/JPEG images directly. Use it to inspect captured frames,
diff composites, and per-layer diagnostics.

### Commands
- `cargo run -p kidcli -- capture-frames --level XX --frames N,N,N --output .cache/debug/`
- `cargo run -p kidcli -- diagnose-frame --level XX --frame N --output .cache/debug/ --save-layers`
- `cargo run -p kidcli -- sweep --levels 0-104 --tier state` (full oracle comparison)
- `cargo run -p kidcli -- diagnose-level XX` (deep single-level analysis)

### Verification loop
1. CAPTURE BEFORE: `capture-frames` to `.cache/debug/before/`
2. Read the PNGs to understand current state
3. Fix the code
4. CAPTURE AFTER: same command to `.cache/debug/after/`
5. Read the PNGs to confirm the fix
6. Run `mise run verify-slice-fast` for mechanical regression check

## Validation methodology

### Oracle comparison (correctness)
- jgenesis emulator as ground truth for game state
- `DynamicScenario`: per-frame u8 button masks, JSON-serialisable, identical execution paths in native and oracle
- Sweep: 105 levels x idle scenario, state-level checkpoint comparison
- Position tolerance: 6px. Advisory fields masked.

### Pixel-perfect frame comparison (visual correctness)
- 320x224 fixed-palette pixel art — exact pixel matching is appropriate (no perceptual metrics needed)
- Use `image` crate for byte-by-byte RGBA comparison
- `dify` CLI for ad-hoc investigation: `cargo install dify; dify native.png oracle.png -o diff.png`

### Input replay (regression)
- `DynamicScenario` already supports per-frame button recording + checkpoint assertions
- Both native and oracle use identical `from_mask(u8)` input paths
- Future: `record-inputs` command to capture live play sessions as replay scripts

### Snapshot testing (regression)
- `insta` crate: `assert_yaml_snapshot!` on NativeTraceFrame structs
- Redactions for advisory fields, `cargo insta review` TUI for accepting intentional changes
- Complements oracle (correctness) with native-vs-prior-native (regression)
