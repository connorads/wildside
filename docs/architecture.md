# Architecture

Wildside is a Cargo workspace of eight engine crates plus one vendored FFI shim.
Each crate owns one concern; the dependency graph is deliberately one-directional.

| Crate | Role |
| --- | --- |
| `wildside-rom` | Identify and verify a user-supplied ROM (SHA-1 / header). |
| `wildside-data` | Carve assets from the ROM and emit the typed asset manifest. |
| `wildside-core` | The engine: physics, combat, enemies, terrain — pure gameplay logic. |
| `wildside-render` | Visuals: tiles, palette, background, HUD, player → `DrawCommand`s. |
| `wildside-frame` | SDL-free CPU rasteriser + trace→GEMS audio director shared by the native player and the browser build; the `audio` feature adds the real `GemsAudioEngine` sink. |
| `wildside-md` | Mega Drive emulator harness (r68k + Z80 + ymfm) used as the oracle. |
| `wildside-web` | wasm (`cdylib`) browser session; drives the same rasteriser and audio director as native. |
| `wildside-cli` | The shell: subcommands, SDL2, and all file I/O. |

**Boundary rule.** `core` depends only on `data`; `render` depends only on `data`.
Nothing device- or emulator-shaped (SDL2, the emulator, the filesystem) may enter
`core` or `render` — that keeps the engine deterministic and oracle-comparable.
Device and emulator code lives in `md` and `cli`, and now reaches `frame` and
`web` for audio: the `audio` cargo feature is the seam. The trace→GEMS director
logic in `frame` is feature-free; enabling `audio` pulls in `wildside-md` for the
real `GemsAudioEngine` sink. `wildside-web` builds with that feature on, so the
browser hears the same GEMS output the parity gates verify.

`ymfm-sys` is a ninth, vendored crate: a BSD-3-Clause FFI wrapper around the
[`aaronsgiles/ymfm`](https://github.com/aaronsgiles/ymfm) FM-audio core, consumed
via `md`. It also cross-compiles to `wasm32-unknown-unknown` through zig (see
`crates/ymfm-sys/build.rs`), so the FM core runs in the browser. It is excluded
from the "eight engine crates" count above.

On wasm, `wildside-md` builds default-features-off (the pure-Rust VDP renderer);
the ymfm machine emulation arrives via `frame`'s `audio` feature.
