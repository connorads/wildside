# Kid Chameleon PC Port

This repo is a contained Rust workspace for getting the Mega Drive version of Kid Chameleon running natively on Linux, starting from a user-supplied ROM and a reproducible reference flow built around the public Kid Chameleon disassembly.

## Current status

- `kidcli play` starts the player-facing Linux alpha shell with ROM discovery, main menu, options, pause menu, persistent settings, and the current SDL host underneath.
- `kidcli verify-rom` validates a local ROM and prints its header metadata and SHA-1.
- `kidcli probe-cpu` loads the ROM into a 68000 core, applies the real reset vector, and steps live instructions for bring-up/debugging.
- `kidcli verify-boot` asserts a known cold-boot trace against the current ROM and the current minimal Mega Drive bus stub.
- `kidcli verify-startup` asserts that the current runtime can progress through the early boot loops, reach `GameProgram`, and continue into the later startup VRAM clear path.
- `kidcli verify-level` asserts that the scripted front-end path can enter the first level and that holding `right` advances the in-level camera.
- `kidcli verify-gameplay` runs the longer gameplay-hardening scenarios for pause/resume, death/respawn, multi-level transition coverage, and a long transition soak.
- `kidcli verify-audio-driver` asserts that the integrated Z80 sound CPU reaches the point of issuing YM2612 and PSG writes by first-level entry.
- `kidcli oracle-setup` clones pinned external oracle repos under `.cache/oracles/`, builds a cache-local `jgenesis` helper, and clones `BlastEm` for optional manual debugger use.
- `kidcli verify-audio-output --scenario ...` compares native audio against `jgenesis` for a deterministic scripted scenario and writes artefacts under `.cache/oracle-runs/<scenario>/`.
- `kidcli verify-frames --scenario ...` compares native rendered frames and checkpoint state against `jgenesis` for a deterministic scripted scenario and writes artefacts under `.cache/oracle-runs/<scenario>/`.
- `kidcli verify-oracle --scenario ...` runs both the audio and frame comparisons for the same scenario.
- `kidcli render-frame` runs the game headlessly for a fixed number of instructions and writes a PNG using the current software renderer.
- `kidcli dump-audio` writes a deterministic stereo WAV from the current mixed FM + PSG audio path.
- `kidcli run-sdl` starts the lower-level SDL host used for bring-up and debugging.
- `kidcli bootstrap-upstream` clones the pinned public disassembly into `.cache/upstream/kid-chameleon-disasm`.
- `kidcli reference-build` replaces the upstream `kid.bin` with the local ROM, runs `split.py`, rebuilds the ROM with the bundled Linux assembler, and verifies whether the rebuilt ROM is bit-perfect.

## Containment

- Runtime and build tooling live in this repo.
- Upstream research repos are cached under `.cache/` and ignored by git.
- The only system package installed so far was `cmake` for bundled SDL2.
- The local ROM is expected at one of:
  - `./roms/kid.bin`
  - `./kid.bin`
  - `./kc.md`

## Quick start

```bash
mise trust .mise.toml
mise run play
mise run verify-rom
mise run probe-cpu
mise run verify-boot
mise run verify-startup
mise run verify-level
mise run verify-gameplay
mise run verify-audio-driver
mise run oracle-setup
mise run verify-frames
mise run verify-audio-output
mise run verify-oracle
mise run render-frame
mise run dump-audio
mise run run-sdl
mise run bootstrap-upstream
mise run reference-build
```

For a player-facing launch with the default ROM search path, use:

```bash
cargo run -p kidcli -- play
```

Useful launch overrides:

```bash
cargo run -p kidcli -- play --rom ./kc.md
cargo run -p kidcli -- play --fullscreen
cargo run -p kidcli -- play --scale 3
cargo run -p kidcli -- play --mute
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

To compare the native host against the `jgenesis` oracle, use:

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

Gameplay-hardening artefacts are written under `.cache/gameplay-runs/<scenario>/` as JSON summaries and checkpoints. The long multi-level scenarios use the game's built-in level-skip path once the internal cheat flag is enabled in the verification runtime, so they stress deterministic level-load transitions without depending on brittle blind movement scripts.

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

The current `Restart Run` option restarts the game from boot. It does not yet restore the current level in place; that would need runtime snapshots or save-state support, which is intentionally out of scope for this alpha.

Bundled SDL2 currently needs `cmake` on this machine. This was installed with:

```bash
sudo apt-get update
sudo apt-get install -y cmake
```

The FM core is vendored under `third_party/ymfm` from `aaronsgiles/ymfm` (BSD-3-Clause).

The oracle loop currently builds `jgenesis` from source in `.cache/oracles/` without extra system packages. `BlastEm` is cloned there as well, but building it locally needs development packages that are not currently installed here:

```bash
sudo apt-get update
sudo apt-get install -y libsdl2-dev libglew-dev libgl1-mesa-dev
```

## Arm64 note

This machine is `aarch64`, while the upstream disassembly ships x86-64 Linux build tools. `kidcli reference-build` now handles that automatically by building a native copy of Macroassembler AS under `.cache/tooling/asl-releases` and using it as a fallback. No system package install was needed for that path here.
