# Level 1 Enemy Fidelity Dossier — ARCHIVED

> All 28 enemy families implemented with ASM-verified constants. Oracle sweep passes at 0px tolerance. See `research-map.md` for remaining work.

## Scope

This dossier covers the current enemy-fidelity frontier for Blue Lake Woods 1:
- family-level native behaviour for `Diamond` and `MiniHoppingSkull`
- per-enemy native trace capture
- per-enemy native/oracle checkpoint comparison
- Dragon as the first visible hostile family on the real Level 1 roster

## Read first

- `tasks/prd-level-1-enemy-fidelity-oracle.md`
- `.cache/upstream/kid-chameleon-disasm/code/enemy/Diamond.asm`
- `.cache/upstream/kid-chameleon-disasm/code/enemy/Mini_Hopping_Skull.asm`
- `.cache/upstream/kid-chameleon-disasm/code/enemy/Dragon.asm`
- `.cache/upstream/kid-chameleon-disasm/doc/ram_layout.txt`
- `.cache/upstream/kid-chameleon-disasm/variables.asm`
- `.cache/upstream/kid-chameleon-disasm/level/enemy/4A.bin`
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

- Logical Level 1 (`0x00`) resolves through resource level `0x4A`, so the authoritative split enemy blob is `level/enemy/4A.bin`
- The bundled `4A.bin` enemy layout decodes to:
  - slots `[0xFFFF, 0x000C, 0x0000]`
  - 4 direct `0x0D` objects
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
- `inspect-level --level 00` reports 4 `0x0D` objects through normal resource-level asset resolution
- `probe-native --scenario boot-title-start-level1` activates `enemy_0D`
- `compare-native-oracle --scenario boot-title-start-level1` passes
- `compare-native-oracle --scenario level1-enemy-window` passes

## Working hypothesis

- The old mismatch was mainly a logical-level vs resource-level resolution problem in the native asset path, not a missing split asset
- The oracle helper mapping step ruled out the old “gfx object capture is blind” theory for the early Level 1 window
- The remaining work is to tighten Dragon behaviour against oracle evidence while keeping the traced layout dump as a regression check

## Next actions

- Keep the oracle layout trace as the regression check for any future parser replacement
- Use `level/enemy/4A.bin` plus `maporder.asm` as the tracked Level 1 source authority
- Tighten Dragon behaviour and compare surfaces from this now-clean data seam
- Re-run:
  - `boot-title-start-level1`
  - `boot-title-start-level1-move-right5s`
  - `level1-enemy-window`
- Once Dragon is stable, extend the same seam to the next hostile family beyond the opening window
