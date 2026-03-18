# Whole-Game Index

Lighter map for content beyond the current focus. Each area grows into its own dossier when implementation reaches it. See `research-map.md` for the active acceleration strategy.

## Enemy families (28 total, all done)

All 28 families implemented with ASM-verified constants. Per-family probe scenarios used for verification.

| Tier | Families | Status |
|------|----------|--------|
| 1 | Diamond, Goat, Armadillo, Tank, Crab, Scorpion, Driller | Done — ASM constants verified |
| 2 | Mini/BigHoppingSkull, Cloud, Archer, Sphere, EmoRock, Fireball | Done — ASM constants verified |
| 3 | Ninja, Dragon, Orca, Hand, Robot, SpinningTwins, TarMonster | Done — state machines from ASM |
| 4 | DragonFlying, Lion, UFO, Drips, FireDemon, Tornado | Done — complex AI from ASM |

Source: `.cache/upstream/kid-chameleon-disasm/code/enemy/*.asm`

## Player forms (all done)
- All 10 forms, speed tables for 3 form variants x 3 terrain types, jump height per form
- Form-specific abilities (Skycutter flight, Cyclone climb, RedStealth sword, etc.)
- Per-form Diamond Power effects (20 form/tier combos → 8 unique power types)
- Per-helmet transform duration from ASM Kid_Transform tables
- Source: `kid.asm`, `constants.asm`, helmet-related vars

## Bosses (wired, minor fidelity gaps)
- All 4 boss encounters wired
- Multi-head system done: 3 heads per boss with ASM position offsets and per-head HP (#060)
- HeadyMetal: phase-dependent fire rate + early X gate done; death wobble visual pending (#062)
- Source: boss enemy modules, kid.asm:40177-40700

## Level transitions (done)
- 105 levels loadable, full multi-level progression, ending sequence
- 9 oracle-warp-broken levels (jgenesis bug): 0x09, 0x32, 0x33, 0x50, 0x57, 0x5A, 0x5D, 0x5E, 0x63 (6 in BCD sweep)
- Source: level-order data, completion flow in `kid.asm`

## Menus and cutscenes (done)
- Boot/title, pause/resume, game over, continue screen, score board, sound test
- Intro sequence (5 segments from ASM), demo playback (2 demos from ROM data)
- Source: `scenes/`, `kid.asm` mode handlers

## Weather and special effects (done)
- Murder wall, weather effects (lava/storm/hail/snow), Y look-ahead camera
- Water shimmer (ASM ripple table), desert heat shimmer
- Source: header flags, bg theme logic, special-effect vars

## Audio and GEMS (done)
- GemsAudioEngine (Z80+YM2612+PSG from ROM), SDL2 audio, full FM+PSG
- Source: `sound/`, `doc/sound.txt`, `.cache/references/gems`, `.cache/references/gemsplay`

## Fidelity status
- Oracle sweep: **63/69 levels pass at 0px tolerance**, 6 errors (oracle warp bug)
- Gameplay sweep: **69/69 levels pass** (walk-right + jump + stomp scenarios)
- Advisory fields: `number_objects`, `number_hitpoints`, `number_lives`, `murder_wall_x` (oracle-pause artefacts)
- Authoritative fields (0 diffs): score, diamonds, position, camera, game_mode, level_id
- Determinism: verified — identical traces from repeated scenarios
- Code annotations: 3 TODO(fidelity), 6 DEVIATION, 1 ORIGINAL_QUIRK, 0 UNVERIFIED
- Perf: criterion benchmarks ~92ns/frame idle, ~3µs level load
