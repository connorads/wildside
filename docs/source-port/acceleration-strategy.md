# Acceleration Strategy

## Philosophy

**Progressive fidelity**: get it playable, then get it accurate. Every system starts approximate and is refined with oracle comparison. The oracle (jgenesis) is the safety net.

**Exploit what we have**: full disassembly with symbols, typed asset manifest, oracle comparison, and the ROM. These assets should drive development, not slow it down.

## 5 Tracks

### Track 1: Block Collision (highest priority)

Without real terrain, the player walks on a flat floor and the game is unplayable. Parse level block data into a `BlockGrid`, implement `probe_terrain` as a pure function, replace the flat floor.

Format: 2 bytes per cell, 3 bits collision type + 4 bits block type + 8 bits skin.

### Track 2: Oracle-Driven Differential Testing

Extend the oracle to dump per-enemy state (position, velocity, HP) each frame. For any enemy: extract trace -> write test -> implement to pass. TDD powered by the actual game.

### Track 3: Progressive Fidelity Enemies

28 families x days each = months. Instead: stub -> archetype -> accurate.

Four archetypes cover ~60% of families:
- **Pacer**: walk back and forth, reverse on wall
- **Hopper**: bounce with gravity, reverse on wall
- **Flyer**: sine-wave or linear path
- **Stationary attacker**: face player, project attacks

### Track 4: Level Traversal Skeleton

Flag collision + level transitions = all 105 levels traversable. Reveals integration issues early.

### Track 5: LLM-Assisted ASM Translation

The disassembly has good symbols. LLM translation produces a first draft, oracle verification catches errors. Not a tool to build -- a workflow.

## Key Insight: The Coroutine Problem

Enemy ASM uses cooperative multitasking (`j_Hibernate_Object_1Frame` yields, kernel resumes at saved PC). This means:
- M68k interpreter approach would rebuild the emulator (rejected)
- Hand-translation requires explicit state enums from implicit PC-as-state
- Data-driven archetypes sidestep the problem for simple enemies

## Rejected Approaches

| Approach | Why |
|----------|-----|
| M68k interpreter in native runtime | Oracle already IS an emulator. No exit path. |
| Full ASM-to-Rust transpiler tool | Research project. LLM workflow is faster. |
| Hybrid native + interpreted | Boundary coupling at every interaction. |

## Implementation Order

1. Block collision -- unlocks real gameplay
2. Flag collision + level transitions -- unlocks traversal
3. Stationary enemy stubs -- enemies visible and damageable
4. Oracle per-object capture -- unlocks per-enemy testing
5. Enemy archetypes -- enemies that move
6. Player-enemy collision + damage
7. Platform collision
8. Full enemy AI -- oracle-verified per family
