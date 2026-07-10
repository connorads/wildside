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
- **Real source-port visual evidence added:** `1p-accurate` now renders four early Level 0 gameplay checkpoints through the source-port renderer and compares them with jgenesis. All four have exact state and no masking; worst drift is 2,293 pixels / MAE 1.677 at frame 30. The fail-closed policy kills all 38 focused mutants.
- **Visual confidence boundary:** the new probe is a settled-warp renderer ratchet, not authoritative cold-boot proof, and it is visibly non-pixel-exact. #375 tracks burning the ratchet towards zero. The older `frame_oracle:*` steps compare wildside-md ROM VDP output with jgenesis, not source-port pixels; reports now say so explicitly.
- **Full published-route evidence:** all 37,759 requested player/camera CRC frames match oracle, while 1,512 sampled checkpoints retain #290's baseline of 212 authoritative export diffs and 2 oracle limitations. This is not whole-state, enemy, render, or audio equality.
- **Whole-level sweep boundary:** the latest 69/69 state sweep has 0/69 levels with a real non-masked checkpoint; its 1,011 diffs are oracle limitations, so it proves no level fidelity.
- **Current awaiting-human P0s:** #210 rapid title/menu skip, #370 Dragon breath respawn, and #374 named later-level spawn parity.
- **Next open priority from `project-status`:** #375 early Level 0 visual parity.

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
| 375 | [ ] | P2 | Rendering | Early Level 0 source-port frames differ from oracle despite exact checkpoint state | `source_port_frame_oracle:level_00_early` ratchet towards zero |
