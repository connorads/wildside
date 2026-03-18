# Source-Port Methodology

Detailed reference for reverse-engineering discipline, testing strategy, and fidelity principles.
`CLAUDE.md` has the operational summary; this document explains the *why* and *how* in depth.

## Philosophy

### Accuracy is the goal; compatibility is the outcome

bsnes (byuu): "Accuracy is not 'does it play Zelda?' — playing games is a side effect of accuracy." Fix the engine, never the game. Accurate reimplementation naturally enables compatibility; the reverse is not true.

### Bugs are features

Original game bugs must be faithfully reproduced:
- **Julius** (Caesar III): 100% bidirectional savegame compatibility — any logic deviation breaks saves. Augustus (fork) proved this by breaking compatibility when it added gameplay changes.
- **OpenMW** (Morrowind): intentional bug preservation for mod compatibility. Attack angle uses flawed dot product math in the original; OpenMW reproduces the flaw.
- **Chocolate Doom**: demo sync requires frame-perfect reproduction of original behaviour including bugs. The Lost Soul bouncing bug fix in Ultimate Doom broke all earlier Doom II demos.
- **devilutionX** (Diablo): "Don't fix bugs found during RE — reproduce them exactly to validate correctness."

Mark preserved bugs with `// ORIGINAL_QUIRK: <description>` so they're visible and intentional.

### Progressive fidelity

Get it working first, get it accurate second — but track the gap. Approximate implementations are acceptable stepping stones when marked `// TODO(fidelity): <description>` and tracked in the backlog. Never ship a guess as if it were verified.

### Code as documentation

Implementation should be readable as a specification of original behaviour. bsnes: speed hacks and convenience features "destroy code's ability to serve as verifiable hardware documentation." Avoid clever abstractions that obscure what the original hardware/software actually did.

### Conservative design

bsnes: "Better to not allow valid behaviour than to allow invalid behaviour." Conservative design prevents broken hacks or ROM translations from depending on emulator bugs. When uncertain, preserve original behaviour verbatim rather than guessing at a "better" interpretation.

---

## Investigation Playbook

### Root cause taxonomy

| Category | Symptoms | Primary tools | ASM reference |
|----------|----------|--------------|---------------|
| Timing | Wrong frame for state change, desync drift | `compare-native-oracle`, frame-by-frame stepping | `kid.asm` main loop, interrupt handlers |
| Collision | Stuck in walls, falling through floors, wrong stomp | `capture-frames` + layer overlay, position trace | collision routines in `kid.asm` |
| State machine | Wrong lifecycle transition, stuck states | `probe-native` with state checkpoint logging | player/enemy state handlers |
| Rendering | Wrong tiles, palette, sprite position | `diagnose-frame --save-layers`, pixel diff | VDP register writes |
| Audio | Wrong BGM, missing SFX, timing | `verify-audio-driver`, `verify-audio-output` | `doc/sound.txt`, GEMS engine |

### Structured investigation workflow

This mirrors byuu's investigation loop (identify → capture → hypothesise → probe → verify → fix → sweep):

1. **Observe**: capture frames or run scenario showing the problem
2. **Classify**: which root cause category? (Use taxonomy table)
3. **Probe oracle**: run exact same scenario against jgenesis, capture state checkpoints at same frames
4. **Read disassembly**: follow source precedence — `kid.asm` → `variables.asm` → `doc/formats.txt`
5. **Cross-reference**: native logic vs ASM vs oracle data. All three must agree:
   - ASM + oracle agree, native differs → **native bug**
   - ASM + native agree, oracle differs → **oracle limitation** (rare but possible; 6 warp-broken levels are examples)
   - All three disagree → **deeper investigation needed** (hardware docs, web research)
6. **Write probe scenario**: targeted `DynamicScenario` exercising exactly this behaviour (like bsnes test ROMs — test ROMs must exercise specific undocumented behaviours with observable outcomes)
7. **Fix**: implement smallest correct change
8. **Verify**: probe scenario passes + full sweep passes + visual before/after comparison

### Formula reverse-engineering

When reimplementing a game formula (physics, damage, spawn rates), use the OpenMW empirical approach:

1. Set up controlled test conditions in oracle (known inputs)
2. Capture output values at checkpoints
3. Read disassembly for the formula's ASM implementation
4. Derive Rust equivalent from ASM
5. Compare Rust output against oracle across the full input range
6. Iterate until outputs match for all test cases

---

## Testing Taxonomy

### Architecture layers → test types

| Layer | What to test | How | Volume | Examples |
|-------|-------------|-----|--------|----------|
| Pure core | Physics, speed tables, slope formulas, FP16.16 arithmetic | Unit tests, property-based (proptest) | Most tests | `timing_regression_*`, slope formula tests |
| Enemy logic | AI state machines, activation, movement patterns | Unit tests + probe scenarios | Many tests | `test_enemy_*` family tests |
| Oracle comparison | Full engine state at checkpoints | `compare-native-oracle` scenarios | Per-feature | `DynamicScenario` with targeted inputs |
| Sweep | All levels idle state | `sweep --tier state` | 69 BCD levels | Pre-merge gate |
| Visual | Pixel-perfect frame output | `capture-frames` + byte comparison | Key frames | Before/after fix verification |
| Gameplay | Multi-frame player journeys | `verify-gameplay` scenarios | Key paths | pause/resume, death/respawn, transitions |

### Decision tree

| Situation | Test type | Rationale |
|-----------|-----------|-----------|
| New engine behaviour | Probe scenario + oracle comparison | Like bsnes test ROMs: exercise specific behaviour, compare to ground truth |
| Fixed bug | Timing/gameplay regression test (`verify-slice-fast` gate) | Like Chocolate Doom statcheck: every fix becomes a permanent regression test |
| Pure function, large input space | Property-based test (proptest) | Speed tables, slope formulas, gravity curves — verify properties hold across range |
| Complex state hard to assert field-by-field | Snapshot test (insta) | NativeTraceFrame at checkpoint — detect unexpected state changes between commits |
| Visual correctness | Frame capture + pixel comparison | Like MAME acid tests: any pixel mismatch against oracle = regression |
| Determinism | Replay same scenario twice, diff output | Like Chocolate Doom: deterministic engine is prerequisite for all other testing |

### Future additions

- `insta` snapshot testing for NativeTraceFrame regression detection (native-vs-prior-native)
- `proptest` for speed table lookups, gravity curves, slope formulas
- CI/CD running `verify-slice-fast` on every push
- Input recording from live play sessions as new regression scenarios

---

## Regression Discipline

Every fix MUST become a permanent regression test. Chocolate Doom's entire Compet-N demo archive functions as a regression suite — thousands of real player demos that would desync if any gameplay logic changed. Our equivalent: probe scenarios + sweep + unit tests.

### Test type by fix category

| Fix category | Regression test type |
|-------------|---------------------|
| Physics fix | `timing_regression_*` unit test |
| Enemy behaviour fix | `test_enemy_*` + probe scenario |
| Visual fix | Frame capture golden master |
| State machine fix | Gameplay scenario with state checkpoints |
| Collision fix | Targeted scenario + sweep verification |

### Gates

- **Pre-commit**: `mise run verify-slice-fast` (timing + gameplay regression tests)
- **Pre-merge**: full sweep (`sweep --tier state` on all BCD levels)
- **Post-playtest**: triage new bugs into backlog, write regression tests for each

---

## Cross-Referencing Protocol

### Source resolution

| Situation | Primary source | Secondary source | If they disagree |
|-----------|---------------|-----------------|-----------------|
| Unknown behaviour | Oracle (jgenesis) | Disassembly (`kid.asm`) | Investigate both; oracle is runtime truth, ASM is specification |
| Oracle crash/error | Disassembly | RAM layout docs | Oracle is fallible; trust ASM if oracle fails (6 warp-broken levels) |
| Complex formula | ASM + `variables.asm` | Oracle checkpoint comparison | Derive formula from ASM, validate against oracle output |
| Hardware effect | Kid ASM first | Sega Retro Mega Drive docs | Kid ASM shows what the game actually does; hardware docs show what's possible |
| Undocumented register | M68k instruction docs | Test with oracle | Reference docs are specification; oracle is empirical validation |

### Resolving disagreements

When sources conflict:
1. **Oracle vs ASM**: oracle is runtime truth (what actually happens), ASM is specification (what should happen). If oracle shows behaviour X but ASM seems to say Y, re-read the ASM more carefully — subtle addressing modes, condition flags, or interrupt timing may explain the difference.
2. **Native vs both**: native is wrong. Fix native.
3. **All three disagree**: escalate to deeper investigation — Mega Drive hardware docs, M68k reference manual, or web research. Document the ambiguity.

---

## Common Pitfalls

Lessons learned from bsnes, Chocolate Doom, Julius, OpenMW, devilutionX, and project experience:

| Pitfall | What goes wrong | How to avoid |
|---------|----------------|--------------|
| **Game-specific fixes** | Fix one level, break another; structural issues go undiagnosed | Engine-level fixes only; full sweep after every change (bsnes) |
| **Guessing behaviour** | Subtle incorrectness compounds over time | Probe oracle first; never infer from "what seems right" (bsnes: 80hrs for one Speedy Gonzales bus conflict rather than guessing) |
| **Fixing original bugs** | Breaks compatibility with original data/replays | Preserve bugs with `ORIGINAL_QUIRK` marker (Julius, OpenMW, devilutionX) |
| **Skipping regression tests** | "Quick fix" introduces new bugs discovered later | Every fix → permanent test; no exceptions (Chocolate Doom statcheck) |
| **Optimising before understanding** | Speed hacks obscure correct behaviour | Get it right first, optimise later with benchmarks (bsnes) |
| **Non-deterministic engine** | Replay testing becomes impossible | Fixed RNG, deterministic frame stepping, no system-time dependencies (Chocolate Doom: M_Random seed bug broke cross-session determinism) |
| **Advisory field confusion** | Chasing divergences that are expected | Clear advisory vs authoritative distinction; mask advisory in sweep |
| **Code churn breaking replays** | Refactors silently change execution order | Snapshot tests detect state drift; replay scenarios catch desync (Chocolate Doom) |
| **Whack-a-mole emulation** | Each fix creates new regressions in a cycle | Understand the root cause at hardware/engine level before fixing symptoms (bsnes) |
| **Conflating compatibility with accuracy** | "It works" masks underlying incorrectness | Accuracy naturally enables compatibility; the reverse is not true (bsnes) |

---

## References

Projects studied for methodology:

| Project | Game | Key technique | URL |
|---------|------|--------------|-----|
| bsnes/higan | SNES (all) | Cycle-accurate emulation, test ROMs, code as documentation | bsnes.org |
| Chocolate Doom | Doom | Statcheck demo regression, deterministic replay, tiered compatibility | chocolate-doom.org |
| Julius | Caesar III | 100% savegame compatibility, bidirectional validation | github.com/bvschaik/julius |
| OpenMW | Morrowind | Bug compatibility tiers, empirical formula reverse-engineering | openmw.org |
| devilutionX | Diablo | Function-by-function binary matching, debug symbol recovery | github.com/diasurgical/devilution |
| SM64 decomp | Super Mario 64 | Matching decompilation (byte-for-byte binary identity) | github.com/n64decomp/sm64 |
| MAME | Arcade (all) | Test ROM acid tests, pixel-perfect frame regression | mamedev.org |
