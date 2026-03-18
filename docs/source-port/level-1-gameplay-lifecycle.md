# Level 1 Gameplay Lifecycle Dossier — ARCHIVED

> All lifecycle states implemented: pause/resume, damage, death/respawn, game over, continue. See `research-map.md` for remaining work.

## Scope

This dossier covers the next Level 1 slice after the opening walk: pause/resume, the first gameplay damage transition, death, and respawn.

## Local files to open first

- .cache/upstream/kid-chameleon-disasm/kid.asm
- .cache/upstream/kid-chameleon-disasm/doc/ram_layout.txt
- .cache/upstream/kid-chameleon-disasm/variables.asm
- .cache/native-gameplay-oracle-runs/pause_resume_level1/
- .cache/native-gameplay-oracle-runs/death_respawn_level1/

## Useful commands

```bash
cargo run -p kidcli -- verify-native-gameplay --scenario pause-resume-level1
cargo run -p kidcli -- verify-native-gameplay --scenario death-respawn-level1
cargo run -p kidcli -- compare-native-gameplay --scenario pause-resume-level1
cargo run -p kidcli -- compare-native-gameplay --scenario death-respawn-level1
```

## Lifecycle checkpoints in use

- `pause_resume_level1`
  - `level_settle@719`
  - `pre_pause@839`
  - `paused_tail@919`
  - `resumed_tail@1119`
- `death_respawn_level1`
  - `level_settle@719`
  - `pre_death@1279`
  - `death_tail@1439`
  - `respawn_tail@1919`

## Rust-side targets

- explicit gameplay lifecycle state in `kid-core`
- pause handling that freezes gameplay without mutating state
- Level 1 damage and death transition path
- respawn entry plus settle timing
- gameplay checkpoint comparison command in `kidcli`

## Level-1 lifecycle closure notes

- The gameplay lifecycle slice now uses `compare-native-gameplay` for pause/resume and death/respawn checkpoints.
- The native runtime now models visible death windup, hidden death frames, and respawn settle as explicit lifecycle phases.
- There are no remaining open Level 1 gameplay lifecycle research tasks in this dossier for the pause/death/respawn slice.
- The next tracked frontier after this slice is `docs/source-port/level-1-full-traversal.md`.
