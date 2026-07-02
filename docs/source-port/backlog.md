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
- **2026-07-02 loop handoff:** #210/#254 have a fixed candidate for the
  early IntroSequence6 Start-skip menu timeout and moved back to human
  validation. #252 and #324 remain open from the same human reconfirm.
- **Current verification:** #210/#254 route `2400:start,2700:start` now keeps
  the Red Stealth title menu through frame 3299; native/oracle AE is `0` at
  2700/2720/3000/3299 and the known 8-pixel title-detail residue at 2800.
  Focused intro/menu tests are green; finish with `verify-slice-fast` and
  `verify-tas-full` before commit.
- **Next output:** run `mise run project-status` after commit; take the next
  listed open visual risk (#252 tank/helmet or #324 prize/P block, whichever
  the selector reports first).

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
| 324 | [ ] | P1 | render/block | 2026-07-02 human reconfirm: prize/P blocks can still show a weird sprite or terrain behind the wobble. | Re-test BLW1/Highwater prize wobble crop against oracle; inspect `sub_1007A` priority/art path. |
| 252 | [ ] | P1 | render/enemy | 2026-07-02 human reconfirm: RockTank/tank sprite can still look garbled/doubled; user described this as the tank/helmet sprite issue. | Re-test BLW2/Highwater tank body/turret render; keep state untouched unless oracle diverges. |
