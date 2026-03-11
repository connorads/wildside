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
- Done: speed tables for all 3 form variants x 3 terrain types, jump height per form
- Remaining: form-specific abilities (Skycutter flight, Cyclone climb, RedStealth sword, etc.)
- Deferred until enemy system and block collision are working
- Source: `kid.asm`, `constants.asm`, helmet-related vars

## Bosses (4 unique encounters)
- Deferred until enemy system is mature
- Source: boss enemy modules, VRAM/resource loading code

## Level transitions
- Active now: close the Level 1 traversal loop and stabilise the first long transition scenario
- Later: generalise flag collision + level ordering across all 105 levels
- Source: level-order data, completion flow in `kid.asm`

## Menus and cutscenes
- Boot/title done. Pause/resume done.
- Remaining: options, game over, continue, cutscenes
- Source: `scenes/`, `kid.asm` mode handlers

## Weather and special effects
- Deferred until core gameplay loop is solid
- Source: header flags, bg theme logic, special-effect vars

## Audio and GEMS
- Deferred until gameplay is solid
- Emulator has full FM+PSG via `kid-md` + `ymfm-sys`
- Source: `sound/`, `doc/sound.txt`, `.cache/references/gems`, `.cache/references/gemsplay`
