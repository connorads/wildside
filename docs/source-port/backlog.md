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
- **2026-07-02 #210/#254 fixed candidate:** exact old route
  `2400:start,2700:start` stayed green, but adding a rapid menu-select Start at
  `2720:start` reproduced native gameplay/black art while oracle stayed in the
  title/menu -> Blue Lake Woods title-card path. Native now honours
  `Obj_TitleMenu loc_1BB52` cursor-build and one `Title_InputLoop` hibernate
  before `loc_1BBD6`; row-select Start no longer carries into title-card
  `loc_1AE2E` on recorded cold-boot routes. #210/#254 are back in awaiting-human.
- **Current focus:** classify the prize-block and tank/helmet reports with fresh
  captures so #324/#252 do not absorb unrelated helmet-prize or route-only symptoms.

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
| 324 | [ ] | P1 | render/block | 2026-07-02 human reconfirm: prize/P blocks can still show a weird sprite or wrong art behind the wobble. Prior Highwater row-97 route is fixed, so this may be a different block/theme/route. | Capture exact current forest/Highwater prize-block hit and compare with oracle scene frames; do not patch blindly from the closed Highwater row-97 route. |
| 252 | [ ] | P1 | render/enemy | 2026-07-02 human reconfirm: tank/helmet-looking sprite can still appear garbled. Treat as RockTank body/turret render unless a fresh capture proves it is a helmet-prize sprite. | Get exact current capture/route/frame; classify enemy RockTank vs prize helmet first, then compare selected sprite frames/palette against ASM. |
