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
- **#252 route-needed after fresh recheck:** current Level 01 RockTank capture and
  assembled tank/bullet sprites remain coherent; no current tank/helmet garble was
  reproduced without an exact human frame/route. #252 is back in awaiting-human.
- **#324 route-needed after fresh recheck:** known BLW1 row-1 and Highwater
  row-97 prize/P block routes are clean in current scene captures, and the other
  cached Highwater variants do not reproduce the weird-sprite wobble. #324 is
  back in awaiting-human pending an exact screenshot/recording/frame.
- **Human reconfirm 2026-07-02:** latest playtest still reports the same three
  longstanding visible symptoms: weird sprite behind prize/P blocks (#324),
  tank/helmet-looking garble (#252), and black squares where Red Stealth/title
  menu art should be after skipping the menu (#210/#254). These remain in the
  human-validation/blocker lane because current known routes were rechecked clean
  or fixed-candidate; next actionable step is an exact current screenshot,
  recording, route timing, or frame.
- **Proactive verification 2026-07-02:** no ready/open rows remain, so the latest
  loop mutation-audited `wildside-frame/src/hud.rs`. The HUD shard ended at 168
  mutants tested: 159 caught, 9 unviable, zero missed
  (`/tmp/mutants-frame-hud-20260702`). Continue with
  `mise run project-status`; if still empty, use another proactive-verification
  fallback slice and avoid re-running the HUD or compact enemy module shards
  unless their code changes.

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
