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

Awaiting-human rows live in `backlog-awaiting-human.md` and add `Target` (`0xNN`,
`boot`, `demo`, `demo:0xNN`, `audio-device`, `recording:<slug>`, or `unknown`) plus
a short human-facing `Check` sentence so tools can show exactly what a playtester
must validate. The `Check` names the symptom and action directly; vague fallbacks like
"retest the reported route" belong in neither docs nor `project-status`.

## NOW

- **Select work with `mise run project-status`** (priority-ordered; take its top ready/open item, do not eyeball file order).
- **2026-07-07 #210/#254 moved to awaiting-human:** rapid spam route (`2400:start` plus every even frame `2700..2730:start`) now stays in the title-card transition, consumes Start during the title-menu VDP carry, and fades the carried menu CRAM instead of releasing early gameplay. Human SDL validation still required.
- **2026-07-07 #230 moved to awaiting-human:** The Crypt report was SolidGhost block overlay timing/count parity, not weather (`weather=0`). Both 0x38 and 0x47 walk-right recordings now match oracle CRC for all 2460 frames; human visual validation still required.
- **2026-07-07 #270 moved to awaiting-human:** fresh USM1/USM3 slope-wall route searches were clean or stalled before the reported corner, so no gameplay patch is justified without a natural recorded route.
- **2026-07-07 #339 moved to awaiting-human:** USM2 loading-card audio now zero-fills commandless GEMS frames; live warp recording stayed 240 frames in `LevelTransition` with no audio commands and silent WAV output. Real speaker validation still required.
- **2026-07-07 #185 moved to awaiting-human:** Hand live render atlas now keeps ASM `stru_32B6A`/`stru_32B56`/`stru_32B50` frames before the `loc_32AA2` death tail; existing grab-dispatch regressions still pass, but the natural grab route needs human validation.
- **2026-07-07 #372 moved to awaiting-human:** sub_1007A prize-block wobble now renders after Scroll A terrain/block passes, so ordinary low-priority tree/background art cannot erase the copied wobble sprite. Human should re-run the forest prize-block route.
- **2026-07-07 #373 retired:** ASM and fresh native/oracle EOL captures show the flag/base holds static through the `loc_C070` hibernate and snap-removes at cleanup; there is no ROM sink animation to port. Exact-frame base visibility remains #369.
- **2026-07-07 #371 retired:** BLW1 walking Dragon off-screen fall/respawn is the same ROM-faithful no-ledge-clamp behaviour as #239; fresh enemy-frontier slices match oracle on the current branch.
- **Current awaiting-human P0s:** #210 rapid title/menu skip, #370 Dragon breath respawn, and #374 named later-level spawn parity.
- **Current verification:** proactive Goat enemy mutation refresh. `env FILE=crates/wildside-core/src/enemies/goat.rs OUTPUT=/tmp/mutants-goat-20260707-refresh TMPDIR=/tmp mise run mutants-file` reports `23 mutants tested in 3m: 15 caught, 8 unviable`, with no survivors and no source change warranted.
- **Next open priority from `project-status`:** none currently; use the proactive-verification fallback if no human row is promoted.
- **Still route-blocked but reconfirmed:** #252 Tank sprite garble, #280 Hills/green-cloud background seam, and #345 Lion vanish still need exact live frame/route evidence before implementation.
- **Closures from this playtest:** #003/#160 audio works; #126/#258/#268/#269 slope/climb rows work; #141/#144/#147/#167/#214/#238/#241/#246/#248/#262/#263/#264/#266/#272/#273/#274/#275/#276/#277/#279/#281/#283/#284/#286/#289/#335 validated fixed; #161/#176/#184/#187/#188/#189/#217/#260/#322 closed stale or superseded. See archive note `2026-07-06 playtest feedback triage`.

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
