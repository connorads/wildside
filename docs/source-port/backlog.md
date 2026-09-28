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

Status: `[ ]` open, `[~]` in progress, `[f]` frozen lane (standing decision; never selectable, do not extend), `[?]` proposed fix awaiting objective evidence or human validation, `[x]` done, `[-]` won't fix / not a bug, `[b]` blocked / needs human repro
Priority: P0 (game-breaking), P1 (core feel), P2 (missing system), P3 (polish)

`[?]` items carry an evidence block recording the investigation outcome: repro
scenario path, ASM labels cited, oracle-parity result, and the proposed-fix
commit hash. Agents may close objective bugs under
[`completion-campaign.md`](completion-campaign.md); subjective/device checks stay
for the final human playtest. If investigation stalls, record the rejected
hypotheses and next experiment under `#NNN` in the archive, park the case and
continue other useful work. Never commit repeated investigation churn.

Awaiting-human rows live in `backlog-awaiting-human.md` and add `Target` (`0xNN`,
`boot`, `demo`, `demo:0xNN`, `audio-device`, `recording:<slug>`, or `unknown`) plus
a short human-facing `Check` sentence so tools can show exactly what a playtester
must validate. The `Check` names the symptom and action directly; vague fallbacks like
"retest the reported route" belong in neither docs nor `project-status`.

## NOW

- **Current work (2026-09-28):** #185's saved ordinary-input route from a settled Highwater Pass 2 start now matches all 128 emulator gameplay/enemy frames through Hand jump, contact, grab and release. Live Hand collision uses the ROM's mapping rows, grab entry keeps its hibernate boundary, and follow snaps use Kid's resolved post-physics position.
- **Selection:** Finish #185 by attributing the remaining Kid-sprite pixels in the state-exact route. Do not close the visual claim from gameplay parity alone.
- **Scope:** Single-player across all playable levels, branches, forms and bosses. Preserve frozen #406/#409/#416 timing debt and its ratchets. Final package retains subjective/device checks; no human validation is claimed.
- **Jev allowance:** $5 additional across the campaign. #185 used 48 requests in one attempt. The ledger lifetime charged upper bound is $0.257086284. The checked-in route and later comparisons need no further paid navigation.
- **Jev exploration review:** Revision `cf5d84937`. #433 was the highest-priority eligible gap, but its 20-minute case window had expired. Offline ROM graph discovery replaced the false no-entry premise and found the natural Crypt chain. No request or spend occurred. Restart the three-fix cadence from this review.
- **#185 evidence:** `tests/data/dynamic-scenarios/issue-185-hand-natural-grab.json` covers frames 0..127. Strict enemy comparison reports 128 matches and zero divergences. Offline JEV replay matches trace and emulator state; residual pixels are Kid sprite animation and the documented HUD life icon, so the visual claim remains open.
- **Evidence boundary:** Published-route state/RNG gates cover one route. Frozen visual debt is 241 frames / 2,391 pixels. Neither that replay nor idle sweep checkpoints proves whole-game completion.

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
| 185 | [~] | P1 | enemy/render | In Highwater Pass 2, approach a Hand; it should crawl/jump with solid Hand frames and still grab Kid on contact. | Natural jump/contact/grab/release is state-exact; account for residual Kid-sprite pixels before closure. |
| 406 | [f] | P1 | rendering | FROZEN with #409 per ADR 0002: the lane stops accruing sessions; the march stays metered as ADR 0001 left it (frontier 995, injection ratchet). Residue authority moves to census-scoped reconstruction + raster-span-replay. | Do not extend the march; ratchets stay green; take #442-#450 instead. See ADR 0002. |
| 409 | [f] | P1 | rendering | FROZEN per ADR 0002: the recurrence is a downward-only witness (scheduled-frames ratchet 736 / boundary 1657; tas-pixel-baseline). ADR 0003 corrected the spans; #459 found the bounded entry phase non-invertible and closed NO-GO. | Do not extend or promote the lane; keep the 2,391-pixel and scheduled-frame ratchets green. Evidence: archive #459. |
| 416 | [f] | P2 | audio | Frontier FROZEN at frames 0..922 (decision in archive #416): `gems_frame_signature_v1` is a regression witness only; command parity (#435) supersedes it as audio authority. | Never advance `modelled_frames`; #434-#438 closed, remaining audio debt tracked via #439-#441; keep verify-tas-full green. |
