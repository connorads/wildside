# Kid Chameleon PC Port

This repo is a contained Rust workspace for getting the Mega Drive version of Kid Chameleon running natively on Linux, starting from a user-supplied ROM and a reproducible reference flow built around the public Kid Chameleon disassembly.

The current delivery frontier is 1P whole-game fidelity hardening on the native runtime path. `play-native` is the primary manual playtest surface, `1p-playable` is the primary project-health signal, and `1p-accurate` remains the parity-heavy supporting gate.

## Current status

Start with `kidcli project-status` or `mise run project-status`.

- Headline signal: `kidcli release-gate --target 1p-playable`
- Supporting signal: `kidcli release-gate --target 1p-accurate`
- Closed TAS frontier means `replay-matched` on those routes, not whole-game completion
- `kidcli project-status` keeps these signals separate so a green TAS/release artefact does not get read as "the game is nearly done"

- `kidcli play` starts the player-facing Linux alpha shell with ROM discovery, main menu, options, pause menu, persistent settings, and the current SDL host underneath.
- `kidcli play-native` starts the native runtime in a release build. By default it runs the native boot/title/game flow; use `--warp-level <hex> --settled` for a direct settled level playtest.
- `kidcli verify-rom` validates a local ROM and prints its header metadata and SHA-1.
- `kidcli probe-cpu` loads the ROM into a 68000 core, applies the real reset vector, and steps live instructions for bring-up/debugging.
- `kidcli verify-boot` asserts a known cold-boot trace against the current ROM and the current minimal Mega Drive bus stub.
- `kidcli verify-startup` asserts that the current runtime can progress through the early boot loops, reach `GameProgram`, and continue into the later startup VRAM clear path.
- `kidcli verify-level` asserts that the scripted front-end path can enter the first level and that holding `right` advances the in-level camera.
- `kidcli verify-gameplay` runs the longer gameplay-hardening scenarios for pause/resume, death/respawn, multi-level transition coverage, and a long transition soak.
- `kidcli verify-audio-driver` asserts that the integrated Z80 sound CPU reaches the point of issuing YM2612 and PSG writes by first-level entry.
- `kidcli oracle-setup` clones pinned external oracle repos under `.cache/oracles/`, builds a cache-local `jgenesis` helper, and clones `BlastEm` for optional manual debugger use.
- `kidcli extract-assets` runs the upstream split flow for the local ROM and exports a typed generated asset bundle under `.cache/generated/<rom-sha1>/`, including parsed level and enemy metadata in the manifest for native-runtime use.
- `kidcli probe-native --scenario ...` runs the native source-port bootstrap path against an existing scripted scenario and prints semantic checkpoints, including active roster state.
- `kidcli inspect-level --level ...` loads a typed level bundle from the generated cache and prints parsed level header metadata, bgscroll profile summary, platform layout summary, enemy layout summary, and the concrete resource files and byte sizes for that level.
- `kidcli inspect-enemy --enemy ...` resolves an enemy object ID or name from the extracted constants and points at the matching disassembly module when available.
- `kidcli verify-native-bootstrap` asserts that the native bootstrap skeleton reaches the title screen and then native in-game state for level 1.
- `kidcli compare-native-oracle --scenario ...` is an optional semantic checkpoint spot-check against the `jgenesis` oracle without using `kid-md`.
- `kidcli compare-native-gameplay --scenario ...` is an optional differential check against the gameplay oracle capture for pause/resume, death/respawn, and long traversal/transition scenarios.
- `kidcli play-native --record <slug>` records a live native repro to `.cache/recordings/<slug>/` as input script + native trace, and now writes ROM/build provenance plus record/verify commands into the bundle so the slug can travel as a bug artefact.
- `kidcli play-native --record <slug> --record-notes <path>` copies text notes into the bundle and now also saves `transcript.txt`, `final-frame.png`, plus `audio.wav` so copied repros carry immediate textual, visual, and audio context before verification.
- `kidcli verify-recording --slug <slug>` replays a recorded repro from `.cache/recordings/<slug>/`, compares the replayed native trace against the recorded baseline, and checks sampled checkpoint states/pixels against the oracle.
- `kidcli verify-recording --bundle <path>` does the same for a copied recording bundle outside `.cache/recordings/`, so a remote repro can be verified without manual ingest.
- `kidcli view-tas --scenario ...` launches an SDL viewer for a TAS scenario, running fresh native + `jgenesis` captures and showing native, oracle, and amplified diff panels frame-by-frame.
- `kidcli project-status` summarises repo health with `1p-playable` first, `1p-accurate` second, and TAS frontier state separate.
- `kidcli release-gate --target 1p-playable` is the primary native-playability gate. It composes native gameplay smoke coverage, the current required gameplay compares, and the core slope/bounce/block regressions.
- `kidcli release-gate --target 1p-accurate` remains the parity-heavy sign-off command. It composes native bootstrap, gameplay oracle compares, frame/audio oracle checks, and a full state sweep.
- `kidcli port-loop --scenario ... --iterations ...` repeatedly runs the native bootstrap verification plus native-oracle checkpoint comparison for the requested scenario.
- `kidcli autonomous-loop --goal level1-opening-slice` evaluates the Level 1 opening-slice completion contract, writes ledger/task artefacts under `.cache/autonomy/`, and selects the next unresolved capability.
- `kidcli autonomous-loop --goal level1-gameplay-lifecycle` evaluates the Level 1 gameplay lifecycle contract for pause/resume and death/respawn parity.
- `kidcli autonomous-loop --goal level1-full-traversal` evaluates the post-opening traversal contract for default-terrain authority and long transition parity.
- `kidcli autonomous-loop --goal source-port-frontier` is the operator entrypoint: while TAS frontier work is active it packages that frontier, and once TAS is closed it auto-delegates to the next repo-local lane such as `case-queue` or `sweep-parity`.
- `kidcli verify-audio-output --scenario ...` compares native audio against `jgenesis` for a deterministic scripted scenario and writes artefacts under `.cache/oracle-runs/<scenario>/`.
- `kidcli verify-frames --scenario ...` compares native rendered frames and checkpoint state against `jgenesis` for a deterministic scripted scenario and writes artefacts under `.cache/oracle-runs/<scenario>/`.
- `kidcli verify-oracle --scenario ...` runs both the audio and frame comparisons for the same scenario.
- `kidcli sweep` now appends structured run history to `.cache/metrics/sweep-history.jsonl`, including divergence categories, instead of leaving sweep truth only in one-off run directories.
- `kidcli render-frame` runs the game headlessly for a fixed number of instructions and writes a PNG using the current software renderer.
- `kidcli dump-audio` writes a deterministic stereo WAV from the current mixed FM + PSG audio path.
- `kidcli run-sdl` starts the lower-level SDL host used for bring-up and debugging.
- `kidcli bootstrap-upstream` clones the pinned public disassembly into `.cache/upstream/kid-chameleon-disasm`.
- `kidcli bootstrap-references` clones the pinned 68k and GEMS research repos into `.cache/references/`.
- `kidcli reference-build` replaces the upstream `kid.bin` with the local ROM, runs `split.py`, rebuilds the ROM with the bundled Linux assembler, and verifies whether the rebuilt ROM is bit-perfect.

## Containment

- Runtime and build tooling live in this repo.
- Upstream research repos are cached under `.cache/` and ignored by git.
- Canonical imported TAS scenarios are tracked under `tests/data/tas/` so replay/viewer workflows work on a fresh clone.
- Source-port reference docs live in `CLAUDE.md` and `docs/source-port/reference-index.md`.
- `mise` now pins `cmake 3.30.9` for bundled SDL2 builds.
- The only system package installed so far was `cmake` before the repo switched to the `mise` pin.
- The local ROM is expected at one of:
  - `./roms/kid.bin`
  - `./kid.bin`
  - `./kc.md`

## Quick start

```bash
mise trust .mise.toml
mise run project-status
mise run play
mise run play-native
mise run verify-rom
mise run probe-cpu
mise run verify-boot
mise run verify-startup
mise run verify-level
mise run verify-gameplay
mise run verify-1p-playable
mise run verify-slice-fast
mise run verify-slice-full
mise run release-gate
cargo run --release -p kidcli -- play-native --warp-level 0 --settled --record level1-tree-break
cargo run --release -p kidcli -- play-native --warp-level 0 --settled --record level1-tree-break --record-notes /tmp/level1-tree-break.txt
cargo run -p kidcli -- verify-recording --slug level1-tree-break
cargo run -p kidcli -- verify-recording --bundle /tmp/level1-tree-break
mise run verify-audio-driver
mise run oracle-setup
mise run extract-assets
mise run probe-native
mise run inspect-level
mise run inspect-enemy
mise run verify-native-bootstrap
mise run compare-native-oracle
cargo run -p kidcli -- compare-native-gameplay --scenario pause-resume-level1
cargo run -p kidcli -- compare-native-gameplay --scenario progress-two-transitions
cargo run -p kidcli -- diagnose-native-gameplay-drift --scenario pause-resume-level1 --from-frame 719
mise run port-loop
cargo run -p kidcli -- autonomous-loop --goal level1-opening-slice
cargo run -p kidcli -- autonomous-loop --goal level1-full-traversal
mise run verify-frames
mise run verify-audio-output
mise run verify-oracle
mise run render-frame
mise run dump-audio
mise run run-sdl
mise run bootstrap-upstream
mise run bootstrap-references
mise run reference-build
```

Verification tasks intentionally fail on Rust warnings. Use `mise run verify-*`,
`mise run test`, `mise run verify-slice-fast`, `mise run verify-slice-full`,
`mise run verify-1p-playable`, `mise run release-gate`, and the sweep tasks when you want a clean warning-free gate;
use `mise run play` or `mise run play-native` for manual work without that strictness.

For the source-port research map and local reference layout, start with `CLAUDE.md`, `docs/source-port/reference-index.md`, and `docs/source-port/level-1-full-traversal.md`.

If you want to play or test the project, pick the runtime first:

- `cargo run --release -p kidcli -- play` for the emulator-backed shell with menus and audio.
- `cargo run --release -p kidcli -- play-native --rom ./kc.md` for the native source-port runtime.

For the fastest Level 1 feedback loop, use:

```bash
cargo run --release -p kidcli -- play-native --warp-level 0 --settled
cargo run -p kidcli -- verify-native-bootstrap
cargo run -p kidcli -- verify-native-gameplay --scenario pause-resume-level1
cargo run -p kidcli -- verify-native-gameplay --scenario death-respawn-level1
```

For a player-facing launch with the default ROM search path, use:

```bash
cargo run -p kidcli -- play
```

For the native runtime, use:

```bash
cargo run --release -p kidcli -- play-native
cargo run --release -p kidcli -- play-native --rom ./kc.md
cargo run --release -p kidcli -- play-native --warp-level 0 --settled
cargo run --release -p kidcli -- play-native --warp-level 0 --settled --record level1-tree-break
```

To turn a manual bug into a reproducible artefact, use:

```bash
cargo run --release -p kidcli -- play-native --warp-level 0 --settled --record level1-tree-break
cargo run -p kidcli -- verify-recording --slug level1-tree-break
cargo run -p kidcli -- verify-recording --bundle /tmp/level1-tree-break
```

Useful launch overrides:

```bash
cargo run -p kidcli -- play --rom ./kc.md
cargo run -p kidcli -- play --fullscreen
cargo run -p kidcli -- play --scale 3
cargo run -p kidcli -- play --mute
cargo run --release -p kidcli -- play-native --fullscreen
cargo run --release -p kidcli -- play-native --scale 3
cargo run --release -p kidcli -- play-native --debug-overlay
```

For long CPU probes, prefer the compact mode so the output stays readable:

```bash
cargo run -p kidcli -- probe-cpu --steps 120000 --summary-only
```

You can also script title-screen input while probing or rendering:

```bash
cargo run -p kidcli -- probe-cpu --steps 20000000 --press-start-frame 120 --summary-only
cargo run -p kidcli -- render-frame --steps 5000000 --press-start-frame 120 --out .cache/after-start.png
```

For more control, use `--input FRAME:buttons` or `--input START-END:buttons`:

```bash
cargo run -p kidcli -- probe-cpu --steps 8000000 \
  --press-start-frame 120 --press-start-frame 200 --press-start-frame 280 \
  --press-start-frame 360 --press-start-frame 440 \
  --input 460:start \
  --input 500-650:right \
  --summary-only
```

Button names are `up`, `down`, `left`, `right`, `a`, `b`, `c`, and `start`. You can also pass a raw mask such as `0x88`.

To jump straight into the first level in the SDL host, use:

```bash
cargo run -p kidcli -- run-sdl --autostart-level
```

The player shell searches for a ROM in this order:

- `./roms/kid.bin`
- `./kid.bin`
- `./kc.md`

If no supported ROM is found, `play` shows an in-app error screen with the paths it checked.

To dump a longer capture for inspection, use:

```bash
cargo run -p kidcli -- dump-audio --scenario boot-title-start-level1 --out .cache/long-audio.wav
```

If you want an oracle spot-check after the native path looks right, use:

```bash
cargo run -p kidcli -- oracle-setup
cargo run -p kidcli -- verify-frames --scenario boot-title-start-level1-move-right5s
cargo run -p kidcli -- verify-audio-output --scenario boot-title-start-level1
```

Current oracle results on this machine:

- `verify-frames --scenario boot-title-start-level1` passes.
- `verify-frames --scenario boot-title-start-level1-move-right5s` passes.
- `verify-audio-output --scenario boot-title-start-level1` passes.
- `verify-audio-output --scenario boot-title-idle10s` passes.

The audio oracle now uses onset-aligned RMS-envelope similarity instead of raw Pearson correlation, which better matches the positive-only envelope data emitted by the native and `jgenesis` captures. The emitted artefacts under `.cache/oracle-runs/` still include checkpoint `state.json` alongside WAV/PNG output for deeper debugging.

The new generated source-port bundle is written under `.cache/generated/<rom-sha1>/asset-manifest.json`. It is built from the public disassembly split output and indexed into typed metadata for levels, constants, RAM variables, enemy source files, and asset groups. The current native runtime path is still intentionally narrow, but it now includes family-aware Level 1 enemy state, richer trace surfaces, and per-enemy checkpoint plumbing for oracle comparison.

The native checkpoint comparison artefacts are written under `.cache/native-oracle-runs/<scenario>/`. This path is separate from the older `kid-md` oracle artefacts so the new source-port loop can evolve independently while still using `jgenesis` as the behavioural oracle.

`play-native` is the native source-port runtime. By default it uses the native boot/title/game flow. For direct manual bug work on a specific level, use `--warp-level <hex> --settled`. Use `--debug-overlay` or `Tab` if you want the trace overlay while testing.

The default `kid-core` runtime now loads generated `Level_Layout` terrain on construction, so the default gameplay path and the gameplay/oracle harnesses share the same real-terrain authority by default.

The autonomous Level 1 loops write their ledger, per-cycle reports, and next-task package under `.cache/autonomy/<goal>/`. The machine-readable goal contracts live at `docs/source-port/level-1-goal.json`, `docs/source-port/level-1-gameplay-lifecycle-goal.json`, and `docs/source-port/level-1-full-traversal-goal.json`.

The live source-port frontier loop also writes to `.cache/autonomy/source-port-frontier/`. Its `next-task.json` is generated from `docs/source-port/frontier-state.yaml`, and `next-agent.md` is the cache-local handoff note the current agent leaves for the next loop cycle. When the TAS frontier is closed, that package now stays under `source-port-frontier` but points at the next concrete repo-local lane instead of vague broader-work prose.

Everything under `.cache/` is treated as generated runtime state. If an artefact needs to be durable, reviewable, or versioned, promote it into `docs/source-port/`, code/tests, or explicit checked-in fixtures instead of tracking it in `.cache/`.

Gameplay-hardening artefacts are written under `.cache/gameplay-runs/<scenario>/` as JSON summaries and checkpoints. The long multi-level scenarios use the game's built-in level-skip path once the internal cheat flag is enabled in the verification runtime, so they stress deterministic level-load transitions without depending on brittle blind movement scripts.

## Fast regression gate

Use the fast slice gate before each commit touching `kid-core` or `kidcli` gameplay logic:

```bash
mise run verify-slice-fast
```

Use the full slice gate before calling a traversal/runtime slice green:

```bash
mise run verify-slice-full
```

Use the canonical release sign-off before declaring the native 1P path green:

```bash
mise run release-gate
```

If a gameplay compare reopens, use:

```bash
cargo run -p kidcli -- diagnose-native-gameplay-drift --scenario pause-resume-level1 --from-frame 719
```

Tracked hooks now live under `.hk-hooks/`. Run `git config --local core.hooksPath .hk-hooks` once in the repo to enable the `hk` pre-commit gate.

## Current controls

Player shell defaults:

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
- Current persistent settings:
  - display scale: `auto`, `1x`, `2x`, `3x`, `4x`
  - fullscreen
  - master volume
  - music volume
  - sfx volume
  - mute
  - keyboard bindings
- If SDL audio initialisation fails, the shell continues silently and shows a warning toast instead of exiting

Native playtest defaults:

- `Arrow keys`: move
- `Z`: jump
- `X`: special
- `C`: run
- `R`: restart the native Level 1 playtest
- `Tab`: toggle debug overlay
- `F11`: toggle fullscreen
- `Escape`: quit

The current `Restart Run` option restarts the game from boot. It does not yet restore the current level in place; that would need runtime snapshots or save-state support, which is intentionally out of scope for this alpha.

Bundled SDL2 now uses the `cmake` version pinned in `.mise.toml` so it stays below the CMake 4.x compatibility break that SDL2 0.38.0 trips over.

The FM core is vendored under `third_party/ymfm` from `aaronsgiles/ymfm` (BSD-3-Clause).

The oracle loop currently builds `jgenesis` from source in `.cache/oracles/` without extra system packages. `BlastEm` is cloned there as well, but building it locally needs development packages that are not currently installed here:

```bash
sudo apt-get update
sudo apt-get install -y libsdl2-dev libglew-dev libgl1-mesa-dev
```

## Arm64 note

This machine is `aarch64`, while the upstream disassembly ships x86-64 Linux build tools. `kidcli reference-build` now handles that automatically by building a native copy of Macroassembler AS under `.cache/tooling/asl-releases` and using it as a fallback. No system package install was needed for that path here.
