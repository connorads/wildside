# Source-Port Roadmap: Make 1P Actually Work

## Current truth

The TAS frontier is in a good place:

- `kc_any_percent_2024` is authoritative-clean through frame `4916`
- `any-percent-published` is authoritative-clean through frame `37759`
- the current active frontier in `frontier-state.yaml` is `status: complete`

That does **not** mean the game is nearly finished.

The current `1p-accurate` release gate is useful, but it is narrower than "the game
works". Native play no longer has open agent-actionable P0/P1 playability rows in
the canonical backlog, but human validation/repro items and visual-fidelity polish
still block a completion claim. This roadmap treats those gaps as first-class work,
not as noise beneath a mostly-complete port.

## Finish line for this phase

Primary target: **single-player boot-to-credits that works in native play and converges on oracle-backed accuracy**.

Out of scope for this phase's finish line:

- 2-player alternation as a release blocker
- password flow as a release blocker
- built-in narrated playtest capture or microphone support

Those remain visible backlog epics, but 1P stability and fidelity come first.

## Workstreams

### 1. Truth reset and canonical tracking

Goal: stop future sessions from treating stale completion prose as source of truth.

Tasks:

- keep this roadmap as the top-level programme document
- treat `backlog.md` as the canonical queue for active source-port work
- use `completion-audit.md` before claiming the source-port objective is done
- keep `frontier-state.yaml` as the compact truth for TAS/frontier status only
- archive or rewrite any doc that still says "full playable game", "all done", or similar

Exit criteria:

- roadmap, research map, and whole-game index agree on current status
- completion audit maps finish criteria to current command/backlog evidence
- no active doc claims the project is effectively complete

### 2. 1P playability closure

Goal: fix the bugs that make native play unreliable even when narrow parity gates are green.

Original queue, now mostly closed by backlog #122-#130 and #136-#145:

- death/respawn camera drift and stuck-from-start failures
- Level 2 entry / spawn / floor-fall regressions
- enemy death timing or hurtbox bugs that still damage Kid after a kill
- block visual/state regressions
- slope traversal problems
- bounce-block behaviour
- form-specific block-breaking permission regressions

Rules:

- every bug gets a native repro artefact before or alongside the fix
- every fix lands with a focused regression test or scenario
- every fix is assigned to a gate: fast slice, 1P playable, 1P accurate, or visual parity

Exit criteria:

- native boot -> Level 1 -> Level 2 is stable
- death and respawn no longer leave the player or camera in broken states
- the recurring playtest failures above have permanent regression coverage
- remaining `[?]` rows have human validation, and `[b]` rows have human repro or a
  deliberate out-of-scope decision

### 3. Native playtest evidence pipeline

Goal: make real play sessions reproducible, reviewable, and useful to future agents.

Tasks:

- extend `play-native --record <slug>` from "input + trace" into a standard playtest bundle
- record enough metadata to replay and verify a run on another machine
- add ingestion and verification workflow for copied-in remote playtest bundles
- support optional sidecar artefacts such as frame captures, audio, and user notes
- keep narration/transcript support as an optional sidecar, not a phase-1 prerequisite

Bundle contract:

- manifest version
- ROM identity / build identity
- input script
- native trace
- checkpoint list
- optional frame/audio artefacts
- optional user-notes sidecar

Exit criteria:

- a playtest captured elsewhere can be copied into `.cache/recordings/` and replayed locally
- bug reports can cite a recording slug instead of free-text reproduction only

### 4. Verification hardening

Goal: widen "green" so it reflects player-visible reality, not just a narrow oracle slice.

Tracks:

- keep `1p-accurate` as the oracle-heavy release target
- add a separate `1p-playable` gate for native smoke coverage
- implement framewise checksumming and structured sweep history
- classify divergences explicitly instead of burying them in prose
- promote repeated debug fields into typed or tracked state when they become relied on

`1p-playable` must cover at minimum:

- boot to Level 1
- complete Level 1
- enter and survive Level 2
- die and respawn cleanly
- break common blocks and collect common prizes
- traverse slopes and bounce surfaces

Exit criteria:

- a green top-level status means the game is both parity-clean on known slices and playable through the core 1P path

### 5. Fidelity debt burn-down

Goal: remove "documented debt" as a resting state.

Tracks:

- movement and collision fidelity
- block and prize fidelity
- enemy lifecycle and damage fidelity
- rendering and visual fidelity
- shell/progression fidelity

Policy:

- documented DEVIATIONs are debt to remove or reclassify with evidence
- enemy gaps from `phase4-enemy-findings.md` move into the canonical active backlog
- visual TODOs stay visible until oracle-backed or frame-backed verification closes them

Exit criteria:

- no active deviation remains merely because it was once documented
- remaining fidelity gaps are either verified oracle limitations or explicitly deferred beyond 1P-perfect scope

### 6. Post-1P epics

These stay visible, but they do not block the 1P finish line:

- 2-player alternating-mode verification
- richer narrated-playtest tooling

Resolved and no longer open: password-system investigation. Backlog #133 records the ASM audit:
the stock ROM exposes no password entry path, and native options now mirror the ROM menu.

## Gates

Use four explicit gate buckets:

- `fast-slice`: focused regression tests used before most commits
- `1p-playable`: native smoke coverage for real player-facing failures
- `1p-accurate`: oracle-backed release gate and sweep evidence
- `visual-parity`: frame/pixel-backed checks for rendering fidelity

Every new backlog item should name its minimum required gate before it is considered done.
