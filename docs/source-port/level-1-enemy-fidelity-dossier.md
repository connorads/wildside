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

- Level 1 enemy layout contains 22 objects
- Current generated/native parse says the first visible families are:
  - `Diamond` (`0x01`)
  - `MiniHoppingSkull` (`0x1C`)
- Parsed Level 1 layout object ordering begins with diamonds, then skulls later in the file
- Oracle runtime evidence now contradicts that early-window assumption:
  - the first linked enemy object seen in the real game decodes to `0x0D`
  - that family is `Dragon`, not `Diamond` or `MiniHoppingSkull`

## Current oracle findings

- The cache-local `jgenesis` helper now follows `gfx slot -> parent object -> layout pointer` and can recover family IDs from cartridge ROM when they are needed
- Real oracle runs for:
  - `boot-title-start-level1`
  - `boot-title-start-level1-move-right5s`
  still capture no active `Diamond` or `MiniHoppingSkull` snapshots in the first 5 seconds
- Direct helper instrumentation showed:
  - active Level 1 gfx slots are present at the target checkpoints
  - a stable parent-object layout pointer at `0x0007930A`
  - the linked ROM object entry begins with `0x0D`, matching `Dragon`
- That means the current native/generated enemy-layout interpretation is still ahead of the source-port research, not the oracle helper capture

## Working hypothesis

- Native currently over-counts Level 1 enemies by comparing against a parsed roster that does not yet match the original segmented enemy layout
- The oracle helper mapping step is now good enough to rule out the original “gfx object capture is blind” theory for the early Level 1 window
- The first remaining fidelity task is to decode the real enemy layout/header segmentation used by `Addr_EnemyLayout` and `Addr_EnemyLayoutHeader`, then rebuild native activation against that source data

## Next actions

- Decode the real Level 1 enemy layout/header segmentation from `kid.asm` loader code
- Rebuild `kid-data` Level 1 enemy parsing so early runtime families match oracle evidence
- Keep the oracle helper ROM-backed family decode in place as the regression check for that parser fix
- Re-run:
  - `boot-title-start-level1`
  - `boot-title-start-level1-move-right5s`
  - `level1-enemy-window`
- Once the first two families compare cleanly, extend the same seam to the next Level 1 hostile family
