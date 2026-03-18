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

## Bosses (all done)
- All 4 boss encounters wired (HeadyMetal, Shishkaboss, BoomerangBosses, BagelBrothers)
- Source: boss enemy modules, VRAM/resource loading code

## Level transitions (done)
- 105 levels loadable, full multi-level progression, ending sequence
- 6 oracle-warp-broken levels (jgenesis bug): 0x09, 0x32, 0x33, 0x50, 0x57, 0x5A, 0x5D, 0x5E, 0x63
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
