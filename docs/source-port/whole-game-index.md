# Whole-Game Index

A status map of what exists, what is verified, and what is still open. Not a completion certificate.

See `completion-campaign.md` for the active programme, `level-inventory.md` for
the ROM inventory and `backlog.md` for the canonical queue.

## Systems present in the native port

- Enemy roster: all 28 families wired into the runtime
- Player forms: all 10 forms exist with movement/combat support
- Bosses: all 4 boss encounters present
- Level flow: 106 non-sentinel logical entries, including 103 named playable
  candidates and three labelled leftovers; transitions and ending exist
- Menus and shell: boot/title, pause/resume, continue, score, sound test, demos
- Audio: native GEMS-backed audio path exists

Presence is not full verification.

## Systems with strong current evidence

- Native 1P playable gate: the project-health headline for real-player coverage, refreshed 2026-07-04 at report `1783122908`
- TAS parity: both imported recordings are clean to their recorded endpoints
- Oracle-backed 1P accurate gate: supporting parity evidence, not the primary health headline
- Expanded state sweep `1790032848` covers all 105 standard entries: 98
  passes, seven failures and zero fully masked entries. It checks settled and
  idle state, not natural traversal or enemy-position parity; see
  `level-inventory.md`.
- Determinism: repeated traces are stable on the covered scenarios

## Systems still carrying meaningful post-phase risk

These are active coverage obligations under the single-player completion campaign:

- Natural entry, traversal and completion beyond the current smoke gate and idle sweeps
- Block lifecycle visuals and interaction correctness beyond the current bundles
- Slopes, bounce surfaces, and other movement edge cases beyond current coverage
- Visual fidelity not covered by existing frame checks
- Enemy behaviour polish recorded in `phase4-enemy-findings.md`
- Known intentional approximations that remain post-phase fidelity debt

## Explicitly open areas

The live backlog and ROM inventory identify unresolved work:

- All confirmed playable levels and progression branches, with evidence that
  distinguishes ordinary play from attract demos and diagnostic warps
- Full-frame visual parity sweep and other broader verification hardening

Password-system investigation is no longer open: backlog #133 records the ASM audit and native options-menu alignment. The stock ROM title/options flow has no password entry path.

Recently closed items that used to appear here:

- HeadyMetal death wobble visual verification is covered by backlog #062 render regressions.
- 2-player alternating-mode verification is covered by backlog #132 native scenarios.
- The standard native playtest evidence pipeline is covered by backlog #130 recording bundles.

## Reading this file correctly

Use this file to answer:

- Does the subsystem exist at all?
- How strong is the current evidence?
- Is this still an open risk area?

Do not use this file alone to conclude the game is finished. The active finish-line definition is in `roadmap.md`, the live queue is in `backlog.md`, and the current phase completion decision is in `audits/completion-audit.md`.
