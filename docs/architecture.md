# Architecture

Wildside is a Cargo workspace of six engine crates plus one vendored FFI shim.
Each crate owns one concern; the dependency graph is deliberately one-directional.

| Crate | Role |
| --- | --- |
| `wildside-rom` | Identify and verify a user-supplied ROM (SHA-1 / header). |
| `wildside-data` | Carve assets from the ROM and emit the typed asset manifest. |
| `wildside-core` | The engine: physics, combat, enemies, terrain — pure gameplay logic. |
| `wildside-render` | Visuals: tiles, palette, background, HUD, player → `DrawCommand`s. |
| `wildside-md` | Mega Drive emulator harness (r68k + Z80 + ymfm) used as the oracle. |
| `wildside-cli` | The shell: subcommands, SDL2, and all file I/O. |

**Boundary rule.** `core` depends only on `data`; `render` depends only on `data`.
Nothing device- or emulator-shaped (SDL2, the emulator, the filesystem) may enter
`core` or `render` — that keeps the engine deterministic and oracle-comparable.
Device and emulator code lives only in `md` and `cli`.

`ymfm-sys` is a seventh, vendored crate: a BSD-3-Clause FFI wrapper around the
[`aaronsgiles/ymfm`](https://github.com/aaronsgiles/ymfm) FM-audio core, consumed
by `md` and `cli`. It is excluded from the "six engine crates" count above.
