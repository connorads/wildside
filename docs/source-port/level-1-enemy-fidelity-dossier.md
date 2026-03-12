# Level 1 Enemy Fidelity Dossier

## Scope

This dossier covers the current enemy-fidelity frontier for Blue Lake Woods 1:
- family-level native behaviour for `Diamond` and `MiniHoppingSkull`
- per-enemy native trace capture
- per-enemy native/oracle checkpoint comparison
- the remaining oracle mapping gap for visible enemy objects

## Read first

- `tasks/prd-level-1-enemy-fidelity-oracle.md`
- `.cache/upstream/kid-chameleon-disasm/code/enemy/Diamond.asm`
- `.cache/upstream/kid-chameleon-disasm/code/enemy/Mini_Hopping_Skull.asm`
- `.cache/upstream/kid-chameleon-disasm/doc/ram_layout.txt`
- `.cache/upstream/kid-chameleon-disasm/variables.asm`
- `.cache/upstream/kid-chameleon-disasm/level/enemy/00.bin`
- `.cache/generated/<rom-sha1>/asset-manifest.json`

## Useful commands

```bash
cargo run -p kidcli -- inspect-level --level 00
cargo run -p kidcli -- inspect-enemy --enemy 01
cargo run -p kidcli -- inspect-enemy --enemy 1c
cargo run -p kidcli -- trace-oracle-enemy-layout --scenario boot-title-start-level1
cargo run -p kidcli -- trace-oracle-enemy-layout --scenario level1-enemy-window
cargo test -p kid-core
cargo test -p kidcli
cargo run -p kidcli -- compare-native-oracle --scenario boot-title-start-level1
cargo run -p kidcli -- compare-native-oracle --scenario level1-enemy-window
```

## Native runtime status

- `Diamond` is modelled as a stationary collectible family
- `MiniHoppingSkull` now runs through a dedicated hopper update loop instead of pacer logic
- Native traces now expose:
  - stable layout identity
  - position
  - velocity
  - facing
  - grounded state
  - damage flash
  - hit points

## Level 1 family facts

- The split `level/enemy/00.bin` blob still decodes as 22 objects (`Diamond` + `MiniHoppingSkull`), but that data is not what the runtime is using in the local verified ROM
- Oracle layout tracing for `boot-title-start-level1` and `level1-enemy-window` shows the real Level 1 enemy batch as:
  - header `0x0792EE`
  - object batch `0x0792F8`
  - slots `[0xFFFF, 0x000C, 0xFFFF]`
  - 4 direct `0x0D` objects
- The first visible hostile in the real game is therefore `Dragon` (`0x0D`), not `Diamond` or `MiniHoppingSkull`

## Current oracle findings

- `kidcli trace-oracle-enemy-layout` now writes checkpointed oracle layout truth to `.cache/native-oracle-runs/<scenario>/enemy-layout-trace.json`
- The cache-local `jgenesis` helper now records:
  - `Addr_EnemyLayoutHeader`
  - `Addr_EnemyLayout`
  - current slot table
  - object batch header/count
  - decoded batch objects
  - visible enemy layout pointers from active gfx slots
- The traced visible Level 1 enemy uses layout pointer `0x0007930A` and family `0x0D`
- Native now uses the traced Level 1 roster via a `prepare_generated_assets` override, so:
  - `inspect-level --level 00` reports 4 `0x0D` objects
  - `probe-native --scenario boot-title-start-level1` activates `enemy_0D`
  - `compare-native-oracle --scenario boot-title-start-level1` passes
  - `compare-native-oracle --scenario level1-enemy-window` passes

## Working hypothesis

- The split enemy blob and the runtime ROM diverge for Level 1, so the immediate bottleneck was not just parser shape but source authority
- The oracle helper mapping step has ruled out the old “gfx object capture is blind” theory for the early Level 1 window
- The traced Level 1 override is the current authority for native/runtime work until the upstream split/source-data gap is decoded properly

## Next actions

- Decode why the upstream split `level/enemy/00.bin` does not match the traced ROM batch at `0x0792EA`
- Replace the current Level 1 trace override with a reproducible ROM/source decode path once that source gap is understood
- Keep the oracle layout trace as the regression check for any future parser replacement
- Re-run:
  - `boot-title-start-level1`
  - `boot-title-start-level1-move-right5s`
  - `level1-enemy-window`
- Once the Level 1 data-source gap is closed, extend the same seam to the next hostile family beyond the opening Dragon window
