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
- First visible families are:
  - `Diamond` (`0x01`)
  - `MiniHoppingSkull` (`0x1C`)
- Parsed Level 1 layout object ordering begins with diamonds, then skulls later in the file

## Current oracle findings

- The cache-local `jgenesis` helper now emits per-checkpoint enemy arrays in the shared schema
- Real oracle runs still show a mismatch between:
  - native activated enemy roster
  - oracle-visible/runtime gfx objects
- Direct helper instrumentation showed:
  - active gfx slots in Level 1 are present at the target checkpoints
  - one slot keeps a linked object pointer and a stable `0x0007930A` value at object offset `0x44`
  - the expected family ID is not yet trivially recoverable from the currently sampled gfx/object words

## Working hypothesis

- Native currently over-counts Level 1 enemies by comparing against the activated roster rather than the narrower oracle-visible/runtime object set
- The oracle helper likely needs one more mapping step from gfx slot -> linked object -> level enemy-layout identity before family/type comparison is trustworthy
- The first remaining fidelity task is to decode that link cleanly, then tighten native filtering to the same observable surface

## Next actions

- Decode the linked object/layout address path for Level 1 enemy gfx objects
- Convert oracle candidate slots into stable `(layout_index, object_type, motion)` snapshots
- Filter native checkpoint enemy snapshots to the same observable set before asserting count/type parity
- Re-run:
  - `boot-title-start-level1`
  - `level1-enemy-window`
- Once the first two families compare cleanly, extend the same seam to the next Level 1 hostile family
