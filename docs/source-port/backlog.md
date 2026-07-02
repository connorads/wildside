# Playtest Backlog

Canonical tracker for bugs, missing features, and verification gaps. See `CLAUDE.md`
for iteration protocol, `docs/source-port/methodology.md` for the investigation
playbook, and `docs/source-port/roadmap.md` for the programme-level workstreams.

Important: a green TAS frontier or a green `1p-accurate` release artefact does **not**
mean the game is nearly finished. Native-play breakage, wider verification gaps, and
documented deviations belong here too.

Progress language uses the evidence ladder from `methodology.md`:
`stubbed`, `boots`, `replay-matched`, `branch-matched`, `mechanic-swept`, `human-play-stable`.
A closed TAS frontier proves `replay-matched` on that route, not `human-play-stable`.

Status: `[ ]` open, `[~]` in progress, `[?]` agent proposes fix complete — **human must validate before closing**, `[x]` done, `[-]` won't fix / not a bug, `[b]` blocked / needs human repro
Priority: P0 (game-breaking), P1 (core feel), P2 (missing system), P3 (polish)

`[?]` items carry an evidence block recording the investigation outcome: repro
scenario path, ASM labels cited, oracle-parity result, and the proposed-fix
commit hash. Only a human may promote `[?]` → `[x]`; the agent lane never
auto-closes. If investigation stalls, write the dead end (rejected hypotheses
included) under the issue's `#NNN` in `backlog-archive.md` and leave status at
`[~]`. The stop rule comes from `loop.md`: no code commit ⇒ write the dead end
and stop — never commit churn.

## NOW

- **Select work with `mise run project-status`** (priority-ordered; take its top ready/open item, do not eyeball file order).
- **2026-07-02 failed human validation:** the recently reconfirmed rows have
  been reconfirmed again by live play. Treat the prior route-needed parking as
  the process failure: fixed-candidate probes and synthetic routes are not
  exercising the live paths the human keeps seeing.
- **#271 resolved 2026-07-02:** human retest reports RockTank head contact no
  longer hurts Kid/Red Stealth ("no hurty") after the delayed Tank/TankShooting
  side-contact timing fix. Keep #259 separate for the still-open floor/visual
  placement report.
- **#259 reopened 2026-07-02:** RockTanks still look halfway in the floor on
  levels 2/3. Next pass must pair enemy-only and final-composite views with
  `rock_tank_contact_probe`, because perceived floor sink may be causing
  players to aim at a damaging side box.
- **#324 fixed-candidate 2026-07-02:** human route
  `playtest-2026-07-02-prize-wobble` now has a direct current-code replay
  check: verifier frame 840 shows the lifted P block with no diagonal wood
  leak underneath. Oracle RAM on the focused hit shows the live cell is `$6F00`,
  while the copied wobble sprite still draws `$E102` source art. Keep awaiting
  human validation per tracker rules.
- **#369 reopened 2026-07-02:** flagpole base is still missing at natural EOL
  despite the priority candidate. Next pass should verify scene command
  presence, priority mask, and raw VDP/EOL sprite list on the exact natural
  finish frame.
- **#210/#254 remain suspect 2026-07-02:** menu-skip black squares/Red Stealth
  corruption was recently fixed-candidate, but recurrent live validation has
  failed enough times that the next playtest failure should reopen it
  immediately instead of asking for another proof route.
- **Visual route work:** do not retune Tank Y/priority, flagpole priority, or
  title/menu VDP from stale direct-camera or clean synthetic routes. If the
  route machinery cannot see the bug, improve the capture/probe surface first.
- **Proactive verification 2026-07-02:** movement title-card activation
  constants now have an ASM-cited regression. The movement shard exposed two
  surviving mutants on `LEVEL_ENTRY_TITLE_DEFERRED_NUMBER_VISIBLE_FRAME`
  (`+1 -> -1/*1`); the new loc_1AB26/loc_1AD48 test pins letters at frame 43
  and the deferred level-number worker exactly one frame later. Targeted
  mutation rerun `/tmp/mutants-movement-title-frame-20260702`: 2 tested, 2
  caught.

NOW is the current-state handoff: rewrite it at the end of every loop session, never
append. Old handoffs go to [`session-log.md`](./session-log.md).

For newly opened items, capture these facts in the investigation notes:

- evidence source: playtest, recording slug, oracle, or frame capture
- native repro status: reproduced / not yet reproduced
- oracle repro status: authoritative / advisory / unknown / oracle limitation
- minimum gate: `fast-slice`, `1p-playable`, `1p-accurate`, or `visual-parity`

> **This file is the open queue only** (`[ ]`/`[~]` rows); keep Verify cells short.
> `[?]`/`[b]` rows live in [`backlog-awaiting-human.md`](./backlog-awaiting-human.md);
> resolved rows and all per-issue evidence live in
> [`backlog-archive.md`](./backlog-archive.md) — grep it by `#NNN` (e.g.
> `grep -n '#238' backlog-archive.md`) for an issue's full history. Select work with
> `mise run project-status` (priority-ordered), not by file order. When a row leaves the
> open lane, move it to its lane and append closing evidence to the archive under `#NNN`.

| # | Status | Pri | System | Summary | Verify |
|-----|--------|-----|--------|---------|--------|
| 259 | [ ] | P1 | enemy/render | Reopened 2026-07-02: RockTanks still appear halfway in the floor on levels 2/3, likely interacting with #271 targeting. | Capture same frame as #271 where possible: body/turret Y, child collision box, enemy-only layer, composite foreground mask, oracle side-by-side. |
| 369 | [ ] | P1 | render/eol | Reopened 2026-07-02: flagpole base is still missing at end of level after the priority fixed-candidate. | Natural EOL capture: inspect scene commands, priority mask, raw VDP sprite list, and oracle frame for the base/bottom flag sprite. |
