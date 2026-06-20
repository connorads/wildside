# Command reference

The full `wildside` / `mise run` surface for Wildside. The [README](../README.md)
covers the everyday subset; this is the catalogue.

Verification tasks fail on Rust warnings on purpose. Use `mise run verify-*`,
`mise run test`, `mise run verify-slice-fast`, `mise run verify-slice-full`,
`mise run verify-1p-playable`, `mise run release-gate`, and the sweep tasks when
you want a clean warning-free gate; use `mise run play` or `mise run play-native`
for manual work without that strictness.

For first-time native setup, prefer `mise run bootstrap-native` over
`mise run extract-assets`.

## Contents

- [Playing](#playing)
- [Controls](#controls)
- [Verification gates](#verification-gates)
- [Recording & repro bundles](#recording--repro-bundles)
- [Oracle comparison](#oracle-comparison)
- [Assets & inspection](#assets--inspection)
- [Bring-up & debugging](#bring-up--debugging)
- [Bootstrap & references](#bootstrap--references)
- [Containment](#containment)
- [Cleanup](#cleanup)
- [Generated artefacts](#generated-artefacts)
- [Arm64 note](#arm64-note)

## Playing

- `wildside play` starts the player-facing Linux alpha shell with ROM discovery, main menu, options, pause menu, persistent settings, and the current SDL host underneath.
- `wildside play-native` starts the native runtime in a release build. By default it runs the native boot/title/game flow; use `--warp-level <hex> --settled` for a direct settled level playtest.
- `mise run bootstrap-native` is the canonical native-play setup command. It exports the generated asset bundle and dumps the early gameplay `level-layout` grids for levels `00-02`.

Pick the runtime first:

```bash
cargo run --release -p wildside-cli -- play                              # emulator-backed shell with menus + audio
cargo run --release -p wildside-cli -- play-native --rom ./kc.md         # native source-port runtime
cargo run --release -p wildside-cli -- play-native --warp-level 0 --settled   # fastest Level 1 loop
```

Useful launch overrides:

```bash
cargo run -p wildside-cli -- play --rom ./kc.md
cargo run -p wildside-cli -- play --fullscreen
cargo run -p wildside-cli -- play --scale 3
cargo run -p wildside-cli -- play --mute
cargo run --release -p wildside-cli -- play-native --fullscreen
cargo run --release -p wildside-cli -- play-native --scale 3
cargo run --release -p wildside-cli -- play-native --debug-overlay
```

The player shell searches for a ROM in this order: `./roms/kid.bin`, then
`./kid.bin`, then `./kc.md`. If none is found, `play` shows an in-app error
screen listing the paths it checked.

`play-native` auto-dumps missing early gameplay `level-layout` grids for levels
`00-02` before launch. For direct manual bug work on a specific level use
`--warp-level <hex> --settled`. Use `--debug-overlay` or `Tab` for the trace
overlay while testing. The default `wildside-core` runtime loads generated
`Level_Layout` terrain on construction, so the default gameplay path and the
gameplay/oracle harnesses share the same real-terrain authority.

To jump straight into the first level in the SDL host:

```bash
cargo run -p wildside-cli -- run-sdl --autostart-level
```

## Controls

### Player shell

- `Arrow keys`: D-pad / menu navigation
- `Z`: B / jump
- `X`: C / special
- `C`: A / run
- `Enter`: Start / confirm
- `Escape`: pause, back, or quit from the main menu
- `F11`: toggle fullscreen
- SDL gamepad: D-pad plus `A`, `B`, `X`, `Start`

Player shell features:

- Main menu: `Start Game`, `Options`, `Quit`
- Pause menu: `Resume`, `Restart Run`, `Return To Title`, `Options`, `Quit To Desktop`
- Options persist under `~/.config/kid-chameleon/config.toml` or `$XDG_CONFIG_HOME/kid-chameleon/config.toml`
- Persistent settings: display scale (`auto`, `1x`-`4x`), fullscreen, master/music/sfx volume, mute, keyboard bindings
- If SDL audio init fails, the shell continues silently and shows a warning toast instead of exiting

`Restart Run` restarts the game from boot. It does not yet restore the current
level in place; that would need runtime snapshots or save-state support, which is
intentionally out of scope for this alpha.

### Native playtest

- `Arrow keys`: move
- `Z`: jump
- `X`: special
- `C`: run
- `R`: restart the native Level 1 playtest
- `Tab`: toggle debug overlay
- `F11`: toggle fullscreen
- `Escape`: quit

## Verification gates

- `wildside verify-rom` validates a local ROM and prints its header metadata and SHA-1.
- `wildside verify-boot` asserts a known cold-boot trace against the current ROM and the current minimal Mega Drive bus stub.
- `wildside verify-startup` asserts the runtime can progress through the early boot loops, reach `GameProgram`, and continue into the later startup VRAM clear path.
- `wildside verify-level` asserts the scripted front-end path can enter the first level and that holding `right` advances the in-level camera.
- `wildside verify-gameplay` runs the longer gameplay-hardening scenarios for pause/resume, death/respawn, multi-level transition coverage, and a long transition soak.
- `wildside verify-audio-driver` asserts the integrated Z80 sound CPU reaches the point of issuing YM2612 and PSG writes by first-level entry.
- `wildside verify-native-bootstrap` asserts the native bootstrap skeleton reaches the title screen and then native in-game state for level 1.
- `wildside project-status` summarises repo health with `1p-playable` first, `1p-accurate` second, plus backlog risks and human blockers.
- `wildside release-gate --target 1p-playable` is the primary native-playability gate. It composes native gameplay smoke coverage, the current required gameplay compares, and the core slope/bounce/block regressions.
- `wildside release-gate --target 1p-accurate` is the parity-heavy sign-off command. It composes native bootstrap, gameplay oracle compares, frame/audio oracle checks, and a full state sweep.

Fast regression gate, before each commit touching `wildside-core` or
`wildside-cli` gameplay logic:

```bash
mise run verify-slice-fast
```

Full slice gate, before calling a traversal/runtime slice green:

```bash
mise run verify-slice-full
```

Canonical release sign-off, before declaring the native 1P path green:

```bash
mise run release-gate
```

For the fastest Level 1 feedback loop:

```bash
cargo run --release -p wildside-cli -- play-native --warp-level 0 --settled
cargo run -p wildside-cli -- verify-native-bootstrap
cargo run -p wildside-cli -- verify-native-gameplay --scenario pause-resume-level1
cargo run -p wildside-cli -- verify-native-gameplay --scenario death-respawn-level1
```

Tracked hooks live under `.hk-hooks/`. Run `git config --local core.hooksPath
.hk-hooks` once in the repo to enable the `hk` pre-commit gate.

## Recording & repro bundles

- `wildside play-native --record <slug>` records a live native repro to `.cache/recordings/<slug>/` as input script + native trace, and writes ROM/build provenance plus record/verify commands into the bundle so the slug can travel as a bug artefact. While recording, press `F10` to dump the current bundle and exit immediately.
- `wildside play-native --record <slug> --record-notes <path>` copies text notes into the bundle and also saves `transcript.txt`, `final-frame.png`, plus `audio.wav` so copied repros carry textual, visual, and audio context before verification.
- `wildside verify-recording --slug <slug>` replays a recorded repro from `.cache/recordings/<slug>/`, compares the replayed native trace against the recorded baseline, and checks sampled checkpoint states/pixels against the oracle.
- `wildside verify-recording --bundle <path>` does the same for a copied recording bundle outside `.cache/recordings/`, so a remote repro can be verified without manual ingest.

Turn a manual bug into a reproducible artefact:

```bash
cargo run --release -p wildside-cli -- play-native --warp-level 0 --settled --record level1-tree-break
cargo run --release -p wildside-cli -- play-native --warp-level 0 --settled --record level1-tree-break --record-notes /tmp/level1-tree-break.txt
cargo run -p wildside-cli -- verify-recording --slug level1-tree-break
cargo run -p wildside-cli -- verify-recording --bundle /tmp/level1-tree-break
```

Freeze repro on the default boot path. Reproduce the stall, then press `F10` to
save the bundle and exit:

```bash
cargo run --release -p wildside-cli -- play-native --record level1-load-freeze
```

## Oracle comparison

- `wildside oracle-setup` clones pinned external oracle repos under `.cache/oracles/`, builds a cache-local `jgenesis` helper, and clones `BlastEm` for optional manual debugger use.
- `wildside probe-native --scenario ...` runs the native source-port bootstrap path against an existing scripted scenario and prints semantic checkpoints, including active roster state.
- `wildside compare-native-oracle --scenario ...` is an optional semantic checkpoint spot-check against the `jgenesis` oracle without using `wildside-md`.
- `wildside compare-native-gameplay --scenario ...` is an optional differential check against the gameplay oracle capture for pause/resume, death/respawn, and long traversal/transition scenarios.
- `wildside view-tas --scenario ...` launches an SDL viewer for a TAS scenario, running fresh native + `jgenesis` captures and showing native, oracle, and amplified diff panels frame-by-frame.
- `wildside port-loop --scenario ... --iterations ...` repeatedly runs the native bootstrap verification plus native-oracle checkpoint comparison for the requested scenario.
- `wildside verify-audio-output --scenario ...` compares native audio against `jgenesis` for a deterministic scripted scenario and writes artefacts under `.cache/oracle-runs/<scenario>/`.
- `wildside verify-frames --scenario ...` compares native rendered frames and checkpoint state against `jgenesis` and writes artefacts under `.cache/oracle-runs/<scenario>/`.
- `wildside verify-oracle --scenario ...` runs both the audio and frame comparisons for the same scenario.
- `wildside sweep` appends structured run history to `.cache/metrics/sweep-history.jsonl`, including divergence categories, instead of leaving sweep truth only in one-off run directories.

If a gameplay compare reopens:

```bash
cargo run -p wildside-cli -- diagnose-native-gameplay-drift --scenario pause-resume-level1 --from-frame 719
cargo run -p wildside-cli -- compare-native-gameplay --scenario pause-resume-level1
cargo run -p wildside-cli -- compare-native-gameplay --scenario progress-two-transitions
```

Oracle spot-check after the native path looks right:

```bash
cargo run -p wildside-cli -- oracle-setup
cargo run -p wildside-cli -- verify-frames --scenario boot-title-start-level1-move-right5s
cargo run -p wildside-cli -- verify-audio-output --scenario boot-title-start-level1
```

Current oracle results on a known-good machine:

- `verify-frames --scenario boot-title-start-level1` passes.
- `verify-frames --scenario boot-title-start-level1-move-right5s` passes.
- `verify-audio-output --scenario boot-title-start-level1` passes.
- `verify-audio-output --scenario boot-title-idle10s` passes.

The audio oracle uses onset-aligned RMS-envelope similarity instead of raw
Pearson correlation, which better matches the positive-only envelope data emitted
by the native and `jgenesis` captures. Emitted artefacts under
`.cache/oracle-runs/` include checkpoint `state.json` alongside WAV/PNG output.

Native checkpoint comparison artefacts are written under
`.cache/native-oracle-runs/<scenario>/`, separate from the older `wildside-md`
oracle artefacts so the source-port loop can evolve independently while still
using `jgenesis` as the behavioural oracle.

## Assets & inspection

- `wildside extract-assets` runs the upstream split flow for the local ROM and exports a typed generated asset bundle under `.cache/generated/<rom-sha1>/`, including parsed level and enemy metadata in the manifest for native-runtime use.
- `wildside dump-level-layout --level <hex>` dumps the generated collision grid used by the native runtime into `.cache/generated/<rom-sha1>/level-layout/`.
- `wildside inspect-level --level ...` loads a typed level bundle from the generated cache and prints parsed level header metadata, bgscroll profile summary, platform layout summary, enemy layout summary, and the concrete resource files and byte sizes for that level.
- `wildside inspect-enemy --enemy ...` resolves an enemy object ID or name from the extracted constants and points at the matching disassembly module when available.

The generated source-port bundle is written under
`.cache/generated/<rom-sha1>/asset-manifest.json`, built from the public
disassembly split output and indexed into typed metadata for levels, constants,
RAM variables, enemy source files, and asset groups. The native runtime path is
still intentionally narrow, but now includes family-aware Level 1 enemy state,
richer trace surfaces, and per-enemy checkpoint plumbing for oracle comparison.

## Bring-up & debugging

- `wildside probe-cpu` loads the ROM into a 68000 core, applies the real reset vector, and steps live instructions for bring-up/debugging.
- `wildside render-frame` runs the game headlessly for a fixed number of instructions and writes a PNG using the current software renderer.
- `wildside dump-audio` writes a deterministic stereo WAV from the current mixed FM + PSG audio path.
- `wildside run-sdl` starts the lower-level SDL host used for bring-up and debugging.

For long CPU probes, prefer compact mode so the output stays readable:

```bash
cargo run -p wildside-cli -- probe-cpu --steps 120000 --summary-only
```

Script title-screen input while probing or rendering:

```bash
cargo run -p wildside-cli -- probe-cpu --steps 20000000 --press-start-frame 120 --summary-only
cargo run -p wildside-cli -- render-frame --steps 5000000 --press-start-frame 120 --out .cache/after-start.png
```

For more control, use `--input FRAME:buttons` or `--input START-END:buttons`:

```bash
cargo run -p wildside-cli -- probe-cpu --steps 8000000 \
  --press-start-frame 120 --press-start-frame 200 --press-start-frame 280 \
  --press-start-frame 360 --press-start-frame 440 \
  --input 460:start \
  --input 500-650:right \
  --summary-only
```

Button names are `up`, `down`, `left`, `right`, `a`, `b`, `c`, and `start`. You
can also pass a raw mask such as `0x88`.

Dump a longer audio capture for inspection:

```bash
cargo run -p wildside-cli -- dump-audio --scenario boot-title-start-level1 --out .cache/long-audio.wav
```

## Bootstrap & references

- `wildside bootstrap-upstream` clones the pinned public disassembly into `.cache/upstream/kid-chameleon-disasm`.
- `wildside bootstrap-references` clones the pinned 68k and GEMS research repos into `.cache/references/`.
- `wildside reference-build` replaces the upstream `kid.bin` with the local ROM, runs `split.py`, rebuilds the ROM with the bundled Linux assembler, and verifies whether the rebuilt ROM is bit-perfect.

For the source-port research map and local reference layout, start with
[`CLAUDE.md`](../CLAUDE.md),
[`docs/source-port/reference-index.md`](source-port/reference-index.md), and
[`docs/source-port/level-1-full-traversal.md`](source-port/level-1-full-traversal.md).

Loop sessions follow [`docs/source-port/loop.md`](source-port/loop.md): the
human-ordered backlog table is the queue and the NOW block at its top is the
session handoff. The old autonomous-loop orchestrator and RL prompt machinery
were removed (see tag `pre-autonomy-demolition`). Loop sessions are stateless
between runs. Historical goal contracts from the retired orchestrator remain at
`docs/source-port/level-1-goal.json`,
`docs/source-port/level-1-gameplay-lifecycle-goal.json`, and
`docs/source-port/level-1-full-traversal-goal.json`.

## Containment

- Runtime and build tooling live in this repo.
- Upstream research repos are cached under `.cache/` and ignored by git.
- Canonical imported TAS scenarios are tracked under `tests/data/tas/` so replay/viewer workflows work on a fresh clone.
- Source-port reference docs live in [`CLAUDE.md`](../CLAUDE.md) and [`docs/source-port/reference-index.md`](source-port/reference-index.md).
- `mise` pins `cmake 3.30.9` for bundled SDL2 builds (below the CMake 4.x compatibility break that SDL2 0.38.0 trips over).
- The only system package installed so far was `cmake`, before the repo switched to the `mise` pin.
- The local ROM is expected at `./roms/kid.bin`, `./kid.bin`, or `./kc.md`.

Everything under `.cache/` is treated as generated runtime state. If an artefact
needs to be durable, reviewable, or versioned, promote it into
`docs/source-port/`, code/tests, or explicit checked-in fixtures instead of
tracking it in `.cache/`.

The FM core is vendored under `third_party/ymfm` from `aaronsgiles/ymfm`
(BSD-3-Clause, Copyright (c) 2021 Aaron Giles); the `ymfm-sys` Rust wrapper is
MIT-licensed, so the crate is published as `MIT AND BSD-3-Clause`. The upstream
notice is preserved at `third_party/ymfm/LICENSE`.

The oracle loop builds `jgenesis` from source in `.cache/oracles/` without extra
system packages. `BlastEm` is cloned there as well, but building it locally needs
development packages that are not currently installed here:

```bash
sudo apt-get update
sudo apt-get install -y libsdl2-dev libglew-dev libgl1-mesa-dev
```

## Cleanup

Generated artefacts grow quickly. The repo exposes manual cleanup tasks:

- `mise run clean-build` removes `target/`
- `mise run clean-cache` prunes disposable cache artefacts and old run histories while preserving expensive bootstrap/toolchain caches such as `.cache/oracles/`, `.cache/oracle-cache/`, `.cache/generated/`, and `.cache/tooling/`
- `mise run clean-all` removes `target/` and performs aggressive cache cleanup, including those expensive caches

Inspect what would be deleted without removing anything:

```bash
python3 scripts/cleanup_generated.py --dry-run
python3 scripts/cleanup_generated.py --aggressive --dry-run
```

## Generated artefacts

Gameplay-hardening artefacts are written under `.cache/gameplay-runs/<scenario>/`
as JSON summaries and checkpoints. The long multi-level scenarios use the game's
built-in level-skip path once the internal cheat flag is enabled in the
verification runtime, so they stress deterministic level-load transitions without
depending on brittle blind movement scripts.

## Arm64 note

This machine is `aarch64`, while the upstream disassembly ships x86-64 Linux build
tools. `wildside reference-build` handles that automatically by building a native
copy of Macroassembler AS under `.cache/tooling/asl-releases` and using it as a
fallback. No system package install was needed for that path here.
