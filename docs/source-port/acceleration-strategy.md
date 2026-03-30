# Acceleration Strategy

## Philosophy

**Progressive fidelity**: get it playable, then get it accurate. Every system starts approximate and is refined with oracle comparison. The oracle (jgenesis) is the safety net.

**Exploit what we have**: full disassembly with symbols, typed asset manifest, oracle comparison, and the ROM. These assets should drive development, not slow it down.

## Tracks 1-5 (Done)

All five original infrastructure tracks are complete:

1. **Block Collision** — `BlockGrid` + `probe_terrain`, full cell encoding
2. **Oracle-Driven Differential Testing** — `DynamicScenario`, sweep harness, 63/69 levels passing
3. **Progressive Fidelity Enemies** — all 28 families wired via archetype system (Pacer/Flyer/Hopper/Bouncer/Inert)
4. **Level Traversal Skeleton** — flag collision + level transitions, 105 levels loadable
5. **LLM-Assisted ASM Translation** — workflow used throughout, oracle verification catches errors

## Current programme (Active)

Infrastructure completion is not the same thing as source-port completion.

The TAS frontier is currently closed, but the active programme is still broader:

- oracle sweep hardening and wider verification
- native 1P playability beyond the narrow release gate
- visual fidelity and rendering accuracy
- explicit open risk areas listed in `whole-game-index.md`

Use these as the current sources of truth:

- Programme status: `docs/source-port/roadmap.md`
- Remaining work: `docs/source-port/research-map.md`
- Risk/open areas: `docs/source-port/whole-game-index.md`
- Canonical queue when populated: `docs/source-port/backlog.md`
- Validation methodology: `docs/source-port/methodology.md`

Do not read this document as proof that the game is effectively done.

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

## Implementation Order (historical, all done)

1. Block collision — unlocks real gameplay
2. Flag collision + level transitions — unlocks traversal
3. Stationary enemy stubs — enemies visible and damageable
4. Oracle per-object capture — unlocks per-enemy testing
5. Enemy archetypes — enemies that move
6. Player-enemy collision + damage
7. Platform collision
8. Full enemy AI — oracle-verified per family

Current work is no longer defined by this historical track list. Use `roadmap.md`,
`research-map.md`, and `whole-game-index.md` for live status.
