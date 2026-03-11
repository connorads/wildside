# Superseded on 2026-03-11 by `tasks/prd-level-1-full-traversal.md`.

# PRD: Level 1 Enemy Fidelity + Oracle Object Capture

## Goal

Deepen the Blue Lake Woods 1 source-port slice from aggregate object-count parity to family-level enemy fidelity with oracle-backed checkpoint comparisons.

This cycle is scoped to the two enemy families already visible in the first Level 1 movement slice:
- `Diamond`
- `MiniHoppingSkull`

## Current state

| System | Status |
|--------|--------|
| Native enemy layout parsing | Done |
| Native enemy activation roster | Done |
| `Diamond` collectible semantics | Done |
| `MiniHoppingSkull` hopper archetype | Done in native runtime |
| Native per-enemy trace identity and motion fields | Done |
| `kidcli` per-enemy checkpoint serialisation | Done |
| `kidcli` per-enemy native/oracle compare path | Done |
| Oracle helper family-level enemy capture | Partial; real runtime still needs tighter mapping to visible oracle objects |

## Scope

### 1. Native enemy behaviour

- Replace the old bespoke family switch with a small archetype seam in `kid-core`
- Keep `Diamond` stationary and collectible
- Run `MiniHoppingSkull` through a proper hopper loop:
  - grounded delay
  - hop launch
  - airborne gravity
  - landing on terrain
  - reversal on wall/ledge
- Preserve existing stomp, contact damage, invulnerability, and pickup semantics unless clearly contradicted by oracle evidence

### 2. Native trace surface

- Expose stable enemy identity via level-layout index
- Expose per-enemy motion/state fields:
  - `object_type`
  - `layout_index`
  - `x_pos`
  - `y_pos`
  - `x_velocity`
  - `y_velocity`
  - `facing`
  - `on_ground`
  - `damage_flash`
  - `hit_points`

### 3. `kidcli` checkpoint comparison

- Extend native/oracle checkpoint capture to include per-enemy state
- Keep the top-level checkpoint compare already in place
- Add a second pass for Level 1 enemy snapshots comparing:
  - count
  - type ordering
  - position
  - velocity
  - facing
- Add a deterministic `level1_enemy_window` scenario covering:
  - level settle
  - first enemy window open
  - later enemy window tail

### 4. Docs and working notes

- Record the current frontier as enemy fidelity/oracle capture, not block collision or graphics
- Leave prior Level 1 PRDs in place but mark them as superseded
- Capture the current oracle findings and open mapping questions in a dedicated dossier

## Execution protocol

- Work autonomously until the whole cycle is complete
- Use TDD as the default rhythm: failing test, smallest fix, refactor
- After each coherent unit, run the relevant tests and make an MVP commit
- Do not stop at analysis or partial wiring
- If context is compacted, resume from:
  - the latest repo state
  - this PRD
  - the current dossier
  - touched tests
  - recent commits
- Persist through failing tests and broken compare runs until every acceptance item that is realistically actionable in-repo has either been completed or documented with precise blockers

## Acceptance criteria

- `kid-core` models `Diamond` and `MiniHoppingSkull` through the new archetype seam
- Native traces expose stable per-enemy identity and motion/state fields
- `kidcli` serialises and compares per-enemy checkpoint data
- `level1_enemy_window` exists and is covered by tests
- Docs point future work at Level 1 enemy fidelity and oracle capture
- The repo contains enough tests and notes that the next agent can continue from the next enemy family or the remaining oracle mapping gap without rediscovering the same ground

## Verification

Run and keep green:

```bash
cargo test -p kid-core
cargo test -p kidcli
cargo test -p kid-render
```

Run scenario checks:

```bash
cargo run -p kidcli -- compare-native-oracle --scenario boot-title-start-level1
cargo run -p kidcli -- compare-native-oracle --scenario level1-enemy-window
cargo run -p kidcli -- compare-native-gameplay --scenario pause-resume-level1
cargo run -p kidcli -- compare-native-gameplay --scenario death-respawn-level1
```

## Out of scope

- Whole-game enemy framework extraction
- New renderer work
- Bosses or non-Level-1 families
- HP comparison from oracle if the RAM path is not cheap enough this cycle
