# Whole-Game Index

This is the lighter map after level 1. Each area should eventually grow into its own dossier when implementation reaches it.

## Player forms
- Source: `kid.asm`, `constants.asm`, helmet-related vars
- Rust target: form-specific move and attack behaviour

## Enemy families
- Source: `.cache/upstream/kid-chameleon-disasm/code/enemy/*.asm`
- Rust target: per-family modules and shared object hooks

## Bosses
- Source: boss enemy modules, VRAM/resource loading code, oracle scenarios to add later
- Rust target: scripted boss state machines

## Menus and cutscenes
- Source: `scenes/`, `kid.asm` mode handlers
- Rust target: native UI and scene controllers

## Level transitions
- Source: level-order data, completion flow in `kid.asm`, oracle long scenarios
- Rust target: transition and progression systems

## Weather and special effects
- Source: header flags, bg theme logic, special-effect vars
- Rust target: effect state plus render/audio hooks

## Audio and GEMS
- Source: `sound/`, `doc/sound.txt`, `.cache/references/gems`, `.cache/references/gemsplay`
- Rust target: event model first, native playback later
