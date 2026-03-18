# Whole-Game Index

Lighter map for content beyond the current focus. Each area grows into its own dossier when implementation reaches it. See `research-map.md` for the active acceleration strategy.

## Enemy families (28 total, 12,355 lines ASM)

Strategy: progressive fidelity (stub -> archetype -> full AI). See `research-map.md` Track 3.

| Priority | Families | ASM lines | Archetype |
|----------|----------|-----------|-----------|
| Tier 1 | Diamond, Goat, Armadillo, Tank, Crab, Scorpion, Driller | 160-260 | Pacer/Stationary |
| Tier 2 | Mini_Hopping_Skull, Big_Hopping_Skull, Cloud, Archer, Sphere, Emo_Rock, Fireball | 220-500 | Hopper/Flyer/Stationary |
| Tier 3 | Ninja, Dragon, Orca, Hand, Robot, Spinning_Twins, Tar_Monster | 275-500 | Complex |
| Tier 4 | Dragon_Flying, Lion, UFO, Drips, Fire_Demon | 280-1300 | Very complex |

Source: `.cache/upstream/kid-chameleon-disasm/code/enemy/*.asm`

## Player forms
- Done: all 10 forms, speed tables for all 3 form variants x 3 terrain types, jump height per form
- Done: form-specific abilities (Skycutter flight, Cyclone climb, RedStealth sword, Micromax, Juggernaut, etc.)
- Source: `kid.asm`, `constants.asm`, helmet-related vars

## Bosses (4 unique encounters)
- Done: all 4 boss encounters wired (HeadyMetal, Shishkaboss, BoomerangBosses, BagelBrothers)
- Source: boss enemy modules, VRAM/resource loading code

## Level transitions
- Done: 105 levels loadable, full multi-level progression, ending sequence
- 8 levels with broken oracle warp (game_mode=0x4E73): 0x09, 0x32, 0x33, 0x50, 0x57, 0x5A, 0x5D, 0x5E, 0x63
- Source: level-order data, completion flow in `kid.asm`

## Menus and cutscenes
- Done: boot/title, pause/resume, game over, continue screen, score board
- Remaining: cutscenes/intro art, demo playback (#026 open — no ROM source data found)
- Source: `scenes/`, `kid.asm` mode handlers

## Weather and special effects
- Done: murder wall, weather effects (lava/storm/hail/snow), Y look-ahead camera
- Remaining: pixel-perfect fidelity verification against oracle
- Source: header flags, bg theme logic, special-effect vars

## Audio and GEMS
- Done: GemsAudioEngine (Z80+YM2612+PSG from ROM), SDL2 audio, full FM+PSG
- Source: `sound/`, `doc/sound.txt`, `.cache/references/gems`, `.cache/references/gemsplay`
