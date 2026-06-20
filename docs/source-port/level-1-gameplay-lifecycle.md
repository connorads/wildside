# Level 1 Gameplay Lifecycle Dossier (ARCHIVED)

> All lifecycle states implemented: pause/resume, damage, death/respawn, game over, continue. See `research-map.md` for remaining work.

## Scope

Covers the Level 1 slice after the opening walk: pause/resume, the first gameplay damage transition, death, and respawn.

## Local files to open first

- .cache/upstream/kid-chameleon-disasm/kid.asm
- .cache/upstream/kid-chameleon-disasm/doc/ram_layout.txt
- .cache/upstream/kid-chameleon-disasm/variables.asm
- .cache/native-gameplay-oracle-runs/pause_resume_level1/
- .cache/native-gameplay-oracle-runs/death_respawn_level1/

## Useful commands

```bash
cargo run -p wildside-cli -- verify-native-gameplay --scenario pause-resume-level1
cargo run -p wildside-cli -- verify-native-gameplay --scenario death-respawn-level1
cargo run -p wildside-cli -- compare-native-gameplay --scenario pause-resume-level1
cargo run -p wildside-cli -- compare-native-gameplay --scenario death-respawn-level1
```

## Lifecycle checkpoints in use

- `pause_resume_level1`
  - `level_settle@719`
  - `pre_pause@839`
  - `paused_tail@919`
  - `resumed_tail@1119`
- `death_respawn_level1`
  - `level_settle@719`
  - `pre_death@1680`
  - `death_tail@1762`
  - `respawn_tail@1919`

## Rust-side targets

- explicit gameplay lifecycle state in `wildside-core`
- pause handling that freezes gameplay without mutating state
- Level 1 damage and death transition path
- respawn entry plus settle timing
- gameplay checkpoint comparison command in `wildside-cli`

## Level-1 lifecycle closure notes

- The gameplay lifecycle slice uses `compare-native-gameplay` for pause/resume and death/respawn checkpoints.
- The native runtime models visible death windup, hidden death frames, and respawn settle as explicit lifecycle phases.
- The smoke verifier runs past `respawn_tail@1919` and checks the final scripted state for the later visible gameplay return, instead of treating the hidden mode-`8` bootstrap as resumed gameplay.
- No Level 1 gameplay lifecycle research tasks remain open in this dossier for the pause/death/respawn slice.
- The next tracked frontier after this slice is `docs/source-port/level-1-full-traversal.md`.
