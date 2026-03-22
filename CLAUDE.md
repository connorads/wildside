# Source-Port Reference Guide

This repo keeps source-port research material in cache-local checkouts and repo docs. Treat this file as the canonical guide for finding the right reference fast. `AGENTS.md` should point here as a symlink.

## Ralph loop (agentic frontier sessions)

If your prompt mentions "Ralph loop", "frontier loop", or "source-port frontier", follow this path **exactly** — do not discover your own:

1. Read `docs/source-port/ralph-source-port-frontier-loop.md` (the loop spec)
2. Read `.cache/autonomy/source-port-frontier/next-agent.md` (the handoff note — this IS your context)
3. Read `.cache/autonomy/source-port-frontier/next-task.json` (the machine task package)
4. Pick one item from "Exact Next Moves" in the handoff note. Do that item only.
5. **Do NOT read** `frontier-state.yaml`, `backlog.md`, or `tas-roadmap.md` — the handoff note already distils them.
6. Commit your work. Update `next-agent.md` before exiting.

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
- `docs/source-port/frontier-state.yaml`
- `.agents/skills/as-assembler/SKILL.md`
- `docs/source-port/methodology.md`
- `docs/source-port/reference-index.md`
- `docs/source-port/research-map.md`
- `docs/source-port/level-1-dossier.md`
- `docs/source-port/whole-game-index.md`

5. Web/manual fallback
- Sega Retro Mega Drive hardware docs
- official M68000 manual

## Fidelity principles

- **Bugs are features**: original game bugs must be faithfully reproduced — collision edge cases, sprite glitches, off-by-ones. Mark with `// ORIGINAL_QUIRK: <description>`. (Julius, OpenMW, Chocolate Doom, devilutionX all preserve original bugs for compatibility)
- **Oracle is truth**: if native differs from oracle, native is wrong until proven otherwise. Exceptions: advisory fields only. ("Accuracy is not 'does it play Zelda?' — playing games is a side effect of accuracy" — byuu)
- **Never guess**: when behaviour is ambiguous, trace the disassembly or compare oracle frames. Write a probe scenario before writing any fix. (bsnes: 80 hours to reverse-engineer one Speedy Gonzales bus conflict rather than guessing)
- **Engine-level only**: no level-specific overrides or game-specific hacks. If a fix works for level 1 but breaks level 23, the architecture needs rethinking. (bsnes: "whack-a-mole emulation" anti-pattern)
- **Progressive fidelity**: approximate implementations acceptable as stepping stones when marked `// TODO(fidelity): <description>` and tracked. Never ship a guess as if it were verified.
- **Code as documentation**: implementation should read as a specification of original behaviour. Avoid speed hacks or clever abstractions that obscure what the original did. (bsnes: speed hacks "destroy code's ability to serve as verifiable hardware documentation")

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
- Frame-level divergence debugging (TAS or sweep)
  - follow `docs/source-port/debug-playbook.md` — probe before reading ASM

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
Compact live frontier state: `docs/source-port/frontier-state.yaml`

### Principles (bsnes methodology)
- Fix behaviours at engine level, never level-specific overrides
- Every fix must pass full sweep — if it fixes level 1 but breaks level 23, the fix is wrong
- Every bug fixed should become a permanent regression test (scenario or snapshot)
- Advisory fields (score, lives, diamonds, number_objects, murder_wall_x) are expected divergences, not bugs
- Probe before coding — write a scenario that exercises the behaviour, run against oracle, understand ground truth before touching native code
- Cross-reference sources — check disassembly AND oracle; if they agree, proceed. If they disagree, investigate further
- Preserve original quirks explicitly — mark with `// ORIGINAL_QUIRK:`. If behaviour looks wrong but oracle confirms it, the original game has that behaviour
- Advisory vs authoritative — advisory divergences are expected and masked in sweep. Authoritative divergences (position, velocity, state, lifecycle) are always bugs
- Reject dead ends once — record them in `frontier-state.yaml` so the next session does not retry them from memory or roadmap prose

### Session protocol
1. Run `mise run frontier-status` and read `docs/source-port/frontier-state.yaml` before scanning wider prose
2. If there is an active frontier, continue that work unless a higher-priority backlog item now blocks it
3. Read `docs/source-port/backlog.md`; pick the highest-priority open item only if no active frontier supersedes it
4. Mark `[~]` when starting work
5. Investigate (structured):
   a. Reproduce: capture frames or run scenario showing the bug
   b. Classify: timing, collision, state machine, rendering, or audio?
   c. Trace oracle: run same scenario against oracle, capture state checkpoints
   d. Read disassembly: find the relevant ASM routine (use source precedence)
   e. Cross-reference: compare native logic to both ASM and oracle data
   f. Write a probe scenario if one doesn't exist for this behaviour
6. After 2 consecutive probe/docs-only commits on the same frontier, the next session must add instrumentation or narrow the scenario before another gameplay patch
7. Fix: implement in focused commit(s)
8. Verify visually: capture frames, Read PNGs, compare before/after
9. Verify mechanically: `mise run verify-slice-fast` must pass
10. Update `docs/source-port/frontier-state.yaml` whenever the frontier, current hypothesis, rejected-hypothesis ledger, or `candidate_promotions` list changes
11. Mark `[x]` in backlog (same commit as the fix)
12. After 3-5 fixes: run `cargo mutants -p kid-core -- --lib` to check for test gaps
13. After 3-5 fixes: playtest with `mise run play-native`, triage new bugs

### Frontier diagnostics defaults
- For TAS frontier work, start with `cargo run -p kidcli -- verify-tas --scenario <scenario> --bisect --frontier-kind enemy`
- Use `--frontier-kind gameplay` only when Kid/camera/global behaviour is the known first causal divergence
- Follow with `cargo run -p kidcli -- tas-diff --scenario <scenario> --include-advisory --frontier-kind enemy` so hidden enemy drift is visible before the visible gameplay break

### Debug field promotion workflow
- Register every emitted `debug_fields` or `gameplay_debug_fields` key in `docs/source-port/debug-field-registry.yaml`
- New keys start as `exploratory` unless they already drive pass/fail parity or regression checks
- If a debug key is used to choose, reject, or verify a fix in repeated frontier work, mark it `candidate` and add it to `candidate_promotions` in `docs/source-port/frontier-state.yaml`
- Run `mise run audit-debug-fields` before landing instrumentation changes; unregistered keys are errors, stale `candidate` keys need promotion or an explicit exemption note
- Promote a key from debug bag to typed state when it is used in parity logic, checked-in regression assertions, or multiple frontiers; keep the debug bag only for raw or temporary correlation data

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

## Test strategy

| Situation | Test type | Rationale |
|-----------|-----------|-----------|
| New engine behaviour | Probe scenario + oracle comparison | Like bsnes test ROMs: exercise specific behaviour, compare to ground truth |
| Fixed bug | Timing/gameplay regression test (`verify-slice-fast` gate) | Every fix becomes a permanent regression test |
| Pure function, large input space | Property-based test (proptest) | Speed tables, slope formulas, gravity — verify properties hold across range |
| Complex state | Snapshot test (insta) | NativeTraceFrame at checkpoint — detect unexpected state changes between commits |
| Visual correctness | Frame capture + pixel comparison | Any pixel mismatch against oracle = regression |
| Determinism | Replay same scenario twice, diff output | Deterministic engine is prerequisite for all other testing |
| Engine-wide invariant | Metamorphic test | Determinism, bounds, symmetry — no per-case oracle needed |
| Stateful system | Stateful property test (proptest-state-machine) | Model lifecycle as states, auto-shrink failing transition sequences |
| Test suite confidence | Mutation testing (`cargo-mutants`) | Surviving mutants = test gaps; target ≥80% kill rate |

## Code conventions for fidelity

- `// ORIGINAL_QUIRK: <desc>` — faithfully reproduced original bug (verified against oracle/disassembly)
- `// DEVIATION: <desc>` — known intentional difference from original (e.g. modern input handling)
- `// TODO(fidelity): <desc>` — approximate implementation needing refinement
- `// UNVERIFIED: <desc>` — behaviour not yet compared to oracle/disassembly

See `docs/source-port/methodology.md` for full investigation playbook, testing taxonomy (metamorphic, mutation, fuzzing, Design by Contract), divergence classification, framewise checksumming, fidelity metrics, and cross-referencing protocol.
