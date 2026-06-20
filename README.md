<p align="center">
  <img src="docs/assets/wildside-hero.webp" alt="Wildside key art: a neon synthwave hero in shades and a leather jacket carving on a hoverboard, with a row of characters arrayed behind." width="840">
</p>

<h1 align="center">Wildside</h1>

<p align="center"><em>A coding-agent experiment: SEGA's <strong>Kid Chameleon</strong> rebuilt in Rust and checked against a real emulator.</em></p>

---

Wildside is an informal experiment in how far you can get by running coding
agents in a loop against a hard target with an objective measure of correctness.
The target is *Kid Chameleon*, a Mega Drive platformer the author grew up on,
rebuilt natively in Rust from a user-supplied ROM. Every behaviour is traced
from the public disassembly and checked frame by frame against a real emulator
(`jgenesis`) used as a ground-truth oracle, so an agent cannot mark its own
homework. (The name is the in-fiction VR arcade game from *Kid Chameleon*'s own
story.)

The aim is a playable game. Bit-perfect parity is the standard the port is held
to, though it will not get all the way there, and that is fine. Fidelity is the
working method: read the 68k disassembly, diff against the oracle, reproduce the
original bugs on purpose, then trade accuracy toward something fun to play.

For the longer story, why it exists and how the agent loop actually worked, see
[`docs/why.md`](docs/why.md).

> **Disclaimer.** Kid Chameleon is a trademark of SEGA Corporation. This is an
> independent, non-commercial reimplementation of the Kid Chameleon engine in
> Rust. No game code, ROM data, or copyrighted assets are distributed with this
> repository. You must own an original copy of the Kid Chameleon ROM to use this
> software. SEGA is not affiliated with this project and has not endorsed it.

## Platforms

| Platform | Status |
| --- | --- |
| Linux (x86-64 / arm64) | ✅ Supported |
| macOS | ✅ Supported |
| Windows | Untested |
| Web (WASM) | 🚧 Planned |

Desktop builds run on SDL2. The `apt-get` and build-from-source steps in the
[command reference](docs/commands.md) are only needed for development and the
oracle, so you can skip them if you just want to play.

## Quick start

You supply the ROM at `./roms/kid.bin`, `./kid.bin`, or `./kc.md`. Then:

```bash
mise trust .mise.toml         # trust the task runner
mise run bootstrap-native     # one-time native setup (assets + level grids)
mise run play-native          # play the native source-port runtime
mise run play                 # or: the emulator-backed shell with menus + audio
mise run project-status       # repo health: 1p-playable / 1p-accurate signals
mise run verify-slice-fast    # fast regression gate (run before committing)
```

That's the everyday surface. The full catalogue lives in the
[command reference](docs/commands.md).

## Status

Current frontier: **1P whole-game fidelity hardening** on the native runtime.

- Headline signal: `wildside release-gate --target 1p-playable`
- Parity signal: `wildside release-gate --target 1p-accurate`

A green TAS or release artefact means `replay-matched` on those routes. That is
not the same as whole-game completion. `wildside project-status` keeps the two
signals separate so a green gate does not get read as "nearly done".

## Where to go next

| You want to… | Read |
| --- | --- |
| See every command, controls, oracle workflow | [`docs/commands.md`](docs/commands.md) |
| Understand the source-port research method | [`CLAUDE.md`](CLAUDE.md) · [`docs/source-port/reference-index.md`](docs/source-port/reference-index.md) |
| Take a loop session / pick the next task | [`docs/source-port/loop.md`](docs/source-port/loop.md) · [`docs/source-port/backlog.md`](docs/source-port/backlog.md) |
| Trace Level 1 in detail | [`docs/source-port/level-1-dossier.md`](docs/source-port/level-1-dossier.md) |

## Licence

See [`LICENSE`](LICENSE). The vendored FM core under `third_party/ymfm`
(`aaronsgiles/ymfm`) is BSD-3-Clause; the `ymfm-sys` wrapper is MIT, so that
crate ships as `MIT AND BSD-3-Clause`.
