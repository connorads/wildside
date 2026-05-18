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
- Human validation of current visual/playability evidence bundles
- Human repro for the remaining Level 2 death/loading report (#146)
- Block lifecycle visuals and interaction correctness beyond the current bundles
- Slopes, bounce surfaces, and other movement edge cases
- Visual fidelity not covered by existing frame checks
- Enemy behaviour gaps recorded in `phase4-enemy-findings.md`
- Known DEVIATIONs that are still parked rather than removed

## Explicitly open areas

- Fire Demon death-trail exact ROM tile extraction polish
- Remaining floating-item / block-state visual fidelity work
- Wider `1p-playable` gate beyond the current smoke contract

Password-system investigation is no longer open: backlog #133 records the ASM audit and native
options-menu alignment. The stock ROM title/options flow has no password entry path.

Recently closed items that used to appear here:

- HeadyMetal death wobble visual verification is covered by backlog #062 render regressions.
- 2-player alternating-mode verification is covered by backlog #132 native scenarios.
- The standard native playtest evidence pipeline is covered by backlog #130 recording bundles.

## Reading this file correctly

Use this file to answer:

- "Does the subsystem exist at all?"
- "How strong is the current evidence?"
- "Is this still an open risk area?"

Do **not** use this file to conclude the game is "basically finished". The active
finish-line definition is in `roadmap.md`, and the live queue is in `backlog.md`.
