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

- **Current work (2026-09-22):** P0 #374 remains open. All five named starts match emulator state at frame 60. Hoverboard Beach and Lion's Den match pixels at two checkpoints; Stairway has an 84-pixel extra foreground shape. Scorpion Isle's lethal Archer Arrow matches state through frame 533, and fatal frame 440 is pixel-exact. Frames 441-444 retain 2,777 lower-scene pixel differences. The Island shore worker is one tick ahead at the settled-warp handoff; a whole-window rewind regressed 88 exact frames and was rejected. Trace the displayed `$F6C0` DMA residency at death without changing earlier cadence. The dense route next differs at frame 534 because native hides the old Kid pose eight frames before ROM's `loc_700/loc_720` clear completes. Bloody Swamp's idle route first differs at frame 144. #502's Jev attempts have no 20/21-Object witness. #300's second teleporter still loads ten frames early. The state sweep is 102/105.
- **Selection:** Run `mise run project-status` and `mise run jev-audit`. Objective awaiting-human cases are eligible for investigation and evidence-backed closure. Coverage prerequisites come first, then P0 #210, #370 and #374 according to dependencies and available evidence.
- **Scope:** Single-player across all playable levels, branches, forms and bosses. Preserve frozen #406/#409/#416 timing debt and its ratchets. Final package retains subjective/device checks; no human validation is claimed.
- **Jev allowance:** $5 additional across the campaign. BLW2 used 100 requests and $0.010448214. #502 used 269 charged requests across its three allowed attempts, adding $0.045005898; lifetime charged upper bound is $0.231980574. The best #502 route reached X607 and 15 Objects but no close-range Drips call. Its case window and attempt count are exhausted. No objective bug is closed from these routes.
- **#370 boundary:** Resident death fade and first Dragon breath duration are fixed. Natural reload remains comparison-inconclusive; title/load timing needs source-derived investigation. See `jev-pilot-results.md`; do not close from the symptom assertion alone.
- **Objective closure:** #350 Hills lower foliage is ROM-pixel-exact on the checked-in frame-39 route. A removed-fix control fails by 1,334 lower-scene pixels; 57 upper-sky pixels remain tracked separately.
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
| 374 | [~] | P0 | level/render | All five starts match state; Scorpion fatal frame 440 matches pixels; reload shell and later art differ. | Match full named routes in state and pixels; derive reload timing from ASM. |
| 502 | [ ] | P1 | enemies/scheduling | Drips range checks omit the live Object-pressure branch above 20 workers. | Acquire an interleaved allocation/deletion witness crossing 20/21 Objects. |
| 300 | [~] | P1 | level/teleporter | First pad route matches 25/25 gameplay frames with 31,226 changed pixels; second loads 10 frames early. | Model ASM reload work; resolve remaining pixels and natural entry. |
| 433 | [~] | P1 | enemies/RNG | Robot first-init motion, shots and grounded turns verified; hurt, slot reuse and natural entry remain open. | Probe interrupted turns and acquire a natural Crypt entry. |
| 509 | [ ] | P2 | enemies/scheduling | RockTank 0x1F and Lion 0x07 move on native target-live row 0 while oracle holds object positions. | Instrument one object scheduler pass independently of the player/RNG hand-off before changing live age. |
| 406 | [f] | P1 | rendering | FROZEN with #409 per ADR 0002: the lane stops accruing sessions; the march stays metered as ADR 0001 left it (frontier 995, injection ratchet). Residue authority moves to census-scoped reconstruction + raster-span-replay. | Do not extend the march; ratchets stay green; take #442-#450 instead. See ADR 0002. |
| 409 | [f] | P1 | rendering | FROZEN per ADR 0002: the recurrence is a downward-only witness (scheduled-frames ratchet 736 / boundary 1657; tas-pixel-baseline). ADR 0003 corrected the spans; #459 found the bounded entry phase non-invertible and closed NO-GO. | Do not extend or promote the lane; keep the 2,391-pixel and scheduled-frame ratchets green. Evidence: archive #459. |
| 416 | [f] | P2 | audio | Frontier FROZEN at frames 0..922 (decision in archive #416): `gems_frame_signature_v1` is a regression witness only; command parity (#435) supersedes it as audio authority. | Never advance `modelled_frames`; #434-#438 closed, remaining audio debt tracked via #439-#441; keep verify-tas-full green. |
