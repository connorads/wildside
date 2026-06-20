# Project Taxonomy: What Kind of Project Is This?

When someone asks "is this really a source port?" or "why not do an SM64-style
matching decompilation?" they are asking the right questions. This document
answers them explicitly so future agents and contributors do not have to
rediscover the reasoning.

## The short version

This is a **semantic reimplementation** of a 68k assembly program into Rust,
verified by runtime oracle comparison rather than binary identity. We call it a
"source port" because the goal — running the game natively on modern hardware
with maximum fidelity — is the same as Chocolate Doom or OpenMW. But the
starting point and verification method are fundamentally different.

It is **not** an emulator (the emulator is our oracle, not our product). It is
**not** a matching decompilation (we cannot compile Rust back to a Mega Drive
ROM). It is **not** a traditional source port (we do not have C/C++ source to
refactor).

---

## Why not a matching decompilation?

The SM64 and Zelda OoT decomp projects write C that, when compiled with the
original toolchain, produces a **byte-for-byte identical ROM**. The binary is
the proof. We cannot do that for several reasons:

| Barrier | Why it blocks us |
|---------|-----------------|
| **No Rust→m68k backend** | LLVM (and therefore `rustc`) has no Motorola 68000 code generator. Even if one existed, the original game was assembled with `AS` (Macro Assembler AS), not compiled from a high-level language. |
| **No original toolchain for Rust** | SM64 uses the original Nintendo `IDO` compiler. We have no equivalent 68k compiler that targets the Mega Drive's specific memory map, VDP, and Z80 co-processor setup. |
| **Hardware is the API** | Original subroutines like `sub_E143A` spin on Z80 bus-request bits, wait for VDP DMA, and write to hardware registers at `$A11100`. These are not portable operations you can express in Rust and then compile to 68k. You would need to emit the exact same instructions, at which point you are writing assembly. |
| **No debug symbols** | devilutionX had debug symbols giving them function names and type boundaries. We have only a community disassembly with recovered labels. |
| **Rust assumes a modern OS** | `wildside-core` uses `Vec`, `HashSet`, `serde`, `fs::read`, and a heap allocator. The Mega Drive has 64KB of work RAM and no OS. A matching decompilation would need to use static pools and raw pointers, not Rust's standard library. |

The existing `.cache/upstream/kid-chameleon-disasm/` **is** already an editable
assembly source that can be modified and reassembled with `AS`. That is the
matching-decompilation equivalent for this game. Our project is orthogonal to
it: we translate assembly semantics into Rust, not into a rebuildable ROM.

---

## Why not share a core engine between emulator and native?

You might imagine a single gameplay core (physics, collision, AI) with two
front-ends:

1. An **emulator wrapper** that translates 68k register/memory state into the core's API
2. A **native wrapper** that feeds the same core from modern input/SDL

This is essentially the HLE (High-Level Emulation) idea used in Dolphin's DSP
and IOS reimplementations. We explicitly **rejected** it:

> | Hybrid native + interpreted | Boundary coupling at every interaction. |

The problem is that the original Kid Chameleon code does **not** separate
"engine" from "game." When Kid collides with an enemy, the 68k code does a
`jsr` into enemy logic that may:

- Hibernate via `j_Hibernate_Object_1Frame` (cooperative multitasking)
- Modify the shared object roster
- Trigger VDP DMA for sprite art
- Queue a sound command through the Z80 bus

Defining a clean API boundary at the `jsr` site means guessing at a separation
that the original developers never made. Any abstraction you introduce is a
place where native and emulator can diverge. We chose to keep the two runtimes
completely separate:

| Path | Crate | What it runs | Verification |
|------|-------|-------------|--------------|
| `play` | `wildside-md` | Original ROM byte-for-byte in a Mega Drive emulator | Is the emulator accurate? (jgenesis cross-check) |
| `play-native` | `wildside-core` | Hand-translated Rust reimplementation (audio via embedded emulator — see below) | Does native state match oracle state frame-by-frame? |

They share **assets** (`wildside-data`) and **input replay format** (`DynamicScenario`),
but not gameplay logic.

## The audio exception

Gameplay and graphics in `play-native` are genuine native reimplementation —
Rust translated from the disassembly, verified against the oracle. **Audio is the
one deliberate exception.** Rather than reimplement the GEMS sound engine, the
native runtime runs the *original* `GEMS_Sounddriver.bin` on an embedded Z80 +
`ymfm` YM2612 core: `wildside-md::GemsAudioEngine` drives the real driver binary
on an emulated Z80, with the FM chip emulated by `ymfm`. Only two parts are
native — the PSG, and the 68k-side "which sound to play" trigger logic that feeds
the driver. So an emulator genuinely *does* ship inside the product, but for sound
alone; everything observable in gameplay/graphics state is native Rust.

This is high-level emulation (HLE) scoped to one subsystem, chosen because the
GEMS driver is a self-contained binary with a narrow command interface — the
boundary the rest of the codebase lacks (see "Why not share a core engine"
above). See `docs/source-port/dark-model-wiring-triage.md` §9 (the live audio
path is the `wildside-md` real-driver chain, not the redundant `wildside-core`
`SoundDriver*` model) and `docs/source-port/audio-triggers-fidelity-audit.md`.
Note this is distinct from `play` (`mise run play`), which is fully
emulator-backed; `play-native` (`mise run play-native`) is native except for this
audio seam.

---

## Peer project comparison

| Project | Starting point | Language | Verification | Can compile back to original binary? | What we borrowed |
|---------|---------------|----------|-----------|--------------------------------------|------------------|
| **Chocolate Doom** | Released C source | C | 4000+ demo replays | Yes (same source, new compiler) | Regression discipline, deterministic replay |
| **OpenMW** | Released C++ source | C++ | Manual + community | Yes (same formulas, new engine) | Empirical formula reverse-engineering, bug preservation |
| **Julius** | Released C source | C | Bidirectional savegame compatibility | Yes | Savegame compatibility as correctness proof |
| **devilutionX** | x86 asm + debug symbols | C++ | Function-by-function binary matching | Yes (matching decompilation) | Function-level RE discipline, debug-symbol recovery |
| **SM64 decomp** | N64 ROM only | C | Byte-for-byte ROM identity | Yes (the whole point) | Incremental matching mindset, non-matching directory concept |
| **bsnes** | No source | C++ | Cycle-accurate hardware tests + test ROMs | N/A (emulator, not a port) | "Accuracy is the goal; compatibility is the outcome" |
| **This project** | 68k disassembly only | Rust | Runtime oracle comparison (jgenesis) | **No** (Rust does not target m68k; no toolchain) | Oracle-driven differential testing, progressive fidelity, code-as-documentation |

The crucial insight from this table: **verification method follows available
tooling.** SM64 verifies by compilation because they have a compiler that targets
the original hardware. We verify by runtime oracle because we do not.

---

## Why "source port" then?

We use the term because:

1. **Goal alignment**: The outcome we want is the same as Chocolate Doom or
   OpenMW — the game running natively on Linux/PC with modern tooling.
2. **Community recognition**: "Reverse-engineered native reimplementation" is
   more accurate but less recognisable. "Source port" signals the intent.
3. **Assembly as specification**: We treat the disassembly (`kid.asm`) as the
   canonical specification, analogous to how a traditional source port treats
   the released source as canonical. The difference is the *form* of the source,
   not the methodology.

If precision matters, the most accurate label is:

> **Oracle-verified semantic reimplementation from assembly**

But "source port" is the shorthand.

---

## What this means for correctness

Because we cannot verify by compilation, our correctness criteria are
**behavioral**, not **structural**:

- **SM64**: "Does the recompiled ROM have the same bytes as the original?"
- **devilutionX**: "Does the rebuilt function have the same x86 bytes as the original?"
- **This project**: "Does `NativeRuntime` produce the same player position, enemy
  states, and collision outcomes as `MdRuntime` when fed the same inputs?"

This is why the testing infrastructure is so heavy on oracle comparison, sweep
runs, TAS verification, and property-based testing. We are proving equivalence
of *semantics*, not *structure*.

---

## Related docs

- `docs/source-port/methodology.md` — the verification and testing taxonomy
  that implements the behavioural-equivalence approach described here.
- `docs/source-port/acceleration-strategy.md` — rejected approaches (interpreter,
  transpiler, hybrid) and why the current dual-runtime path was chosen.
- `docs/source-port/reference-index.md` — source materials and peer project links.
