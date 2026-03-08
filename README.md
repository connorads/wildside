# Kid Chameleon PC Port

This repo is a contained Rust workspace for getting the Mega Drive version of Kid Chameleon running natively on Linux, starting from a user-supplied ROM and a reproducible reference flow built around the public Kid Chameleon disassembly.

## Current status

- `kidcli verify-rom` validates a local ROM and prints its header metadata and SHA-1.
- `kidcli probe-cpu` loads the ROM into a 68000 core, applies the real reset vector, and steps live instructions for bring-up/debugging.
- `kidcli verify-boot` asserts a known cold-boot trace against the current ROM and the current minimal Mega Drive bus stub.
- `kidcli verify-startup` asserts that the current runtime can progress through the early boot loops, reach `GameProgram`, and continue into the later startup VRAM clear path.
- `kidcli verify-level` asserts that the scripted front-end path can enter the first level and that holding `right` advances the in-level camera.
- `kidcli verify-audio-driver` asserts that the integrated Z80 sound CPU reaches the point of issuing YM2612 and PSG writes by first-level entry.
- `kidcli oracle-setup` clones pinned external oracle repos under `.cache/oracles/`, builds a cache-local `jgenesis` helper, and clones `BlastEm` for optional manual debugger use.
- `kidcli verify-audio-output --scenario ...` compares native audio against `jgenesis` for a deterministic scripted scenario and writes artefacts under `.cache/oracle-runs/<scenario>/`.
- `kidcli verify-frames --scenario ...` compares native rendered frames and checkpoint state against `jgenesis` for a deterministic scripted scenario and writes artefacts under `.cache/oracle-runs/<scenario>/`.
- `kidcli verify-oracle --scenario ...` runs both the audio and frame comparisons for the same scenario.
- `kidcli render-frame` runs the game headlessly for a fixed number of instructions and writes a PNG using the current software renderer.
- `kidcli dump-audio` writes a deterministic stereo WAV from the current mixed FM + PSG audio path.
- `kidcli run-sdl` starts a minimal interactive desktop host using bundled SDL2, the current software renderer, and the current mixed FM + PSG audio path.
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
mise run verify-rom
mise run probe-cpu
mise run verify-boot
mise run verify-startup
mise run verify-level
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

To dump a longer capture for inspection, use:

```bash
cargo run -p kidcli -- dump-audio --scenario boot-title-start-level1 --out .cache/long-audio.wav
```

To compare the native host against the `jgenesis` oracle, use:

```bash
cargo run -p kidcli -- oracle-setup
cargo run -p kidcli -- verify-frames --scenario boot-title-idle10s
cargo run -p kidcli -- verify-audio-output --scenario boot-title-start-level1
```

Current oracle results on this machine:

- `verify-frames --scenario boot-title-idle10s` passes.
- `verify-audio-output --scenario boot-title-start-level1` fails with about `5.224s` of audio onset drift versus `jgenesis`.
- `verify-frames --scenario boot-title-start-level1-move-right5s` fails at the `level_entry` checkpoint with an MAE around `58.46`.

Those failures are useful regression signals, not harness bugs. The emitted artefacts under `.cache/oracle-runs/` now include checkpoint `state.json` alongside WAV/PNG output, which is the intended debug loop for fixing the remaining audio and gameplay drift.

## Current controls

- `Arrow keys`: D-pad
- `Z`: B / jump
- `X`: C / special
- `C`: A / run
- `Enter`: Start
- `Escape`: quit the SDL host
- SDL gamepad: D-pad plus `A`, `B`, `X`, `Start`

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
