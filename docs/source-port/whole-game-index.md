# Whole-Game Index

Snapshot of what exists, what is verified, and what is still open. This is a status map,
not a completion certificate.

See `roadmap.md` for the active programme and `backlog.md` for the canonical queue.

## Systems present in the native port

- Enemy roster: all 28 families are wired into the runtime
- Player forms: all 10 forms exist with movement/combat support
- Bosses: all 4 boss encounters are present
- Level flow: 105 levels load, transitions exist, ending exists
- Menus and shell: boot/title, pause/resume, continue, score, sound test, demos
- Audio: native GEMS-backed audio path exists

Presence is not the same as full verification.

## Systems with strong current evidence

- Native 1P playable gate: current project-health headline for real-player coverage
- TAS parity: both imported recordings are clean to their recorded endpoints
- Oracle-backed 1P accurate gate: supporting parity evidence, not the primary health headline
- State sweep: current `1p-accurate` artefact reports the 69 requested levels passing
- Determinism: repeated traces are stable on the covered scenarios

## Systems still carrying meaningful risk

- Native 1P playability beyond the current smoke gate
- Death/respawn and transition stability
- Level 2 entry and post-transition landing stability
- Block lifecycle visuals and interaction correctness
- Slopes, bounce surfaces, and other movement edge cases
- Visual fidelity not covered by existing frame checks
- Enemy behaviour gaps recorded in `phase4-enemy-findings.md`
- Known DEVIATIONs that are still parked rather than removed

## Explicitly open areas

- HeadyMetal death wobble visual verification
- Fire Demon death-trail visuals
- Remaining floating-item / block-state visual fidelity work
- 2-player alternating-mode verification
- Password-system investigation
- Full native playtest evidence pipeline
- Wider `1p-playable` gate beyond the current smoke contract

## Reading this file correctly

Use this file to answer:

- "Does the subsystem exist at all?"
- "How strong is the current evidence?"
- "Is this still an open risk area?"

Do **not** use this file to conclude the game is "basically finished". The active
finish-line definition is in `roadmap.md`, and the live queue is in `backlog.md`.
