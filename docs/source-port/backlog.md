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

- **Current work (2026-10-04):** Shared-walker Y passes main fast/full, saved Diamond parity, census, affected03 ledger and sweep; commit separately. Next ready #518 revalidate the final RNG431 refinement at256 before applying the source-proved Init timing correction. Native missing music/effects, flagpost and tank-helmet reports need their exact launch/route. System audio stays muted.
- **Intake:** Fixed fuzz1791151797 retains23 clusters/31 divergent routes. Ledger031791152346 reaches Flag2824, blocked132 by existing b17bcd1f; old d471 route advances415->589. Ledger remains10 native exits/0 of103 confirmed. Restore23 census overwrites; all100 fixture bytes match. RNG431 finishes263 masks/129 nonzero/4853 replays; final refinement first diverges256 and stays uncommitted pending Init revalidation.
- **Scope:** ADR0004 sets the fidelity floor: gameplay, enemy and RNG state, level exits, visible or audible bugs. Pixel baseline and injection ratchet stay.
- **After each fix:** Re-run fixed fuzz and affected ledger levels with `--write`; extend the ledger beyond0x10 once early levels confirm.
- **Jev allowance:** $5 additional across the campaign. #185 used48 requests in one attempt; lifetime charged upper bound $0.257086284.
- **Jev exploration review:** Review1791140869823663000 on9d3f160d4 plus verified slope unit selects P1 #169 Drips. Settled01 state0..5 matches; existing #333 EmoRock pixels349/350 on2/4 block strict preflight. No paid navigation. Actual SDL321 and native replay321/321 pass. This review resets cadence before the next implementation.
- **Evidence boundary:** Slope fast7513 core/43 gameplay and full pass; full report equals right parent. Sweep102/105 retains baseline. Scene317/318 retains84 HUD pixels each. Scoped mutation catches16 viable substitutions; two do not compile. Cached crawl141/1 retains parent Drips failure. Wider Lion hurt, raw audio-state parity, natural entry and human stability remain unproved.

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
| 186 | [~] | P1 | enemy | Fight a Lion; hurt recovery and engagement must match the ROM. Saved hurt121/fire119/contact44/expiry576 match; original2617 matches0..992 before respawn bootstrap drift. | Hurt pose/palette and121-state replay match. Extend wider hurt and natural-entry witnesses. |
| 515 | [~] | P1 | kid-physics | Standing-entry99485662, zero-speed stand-up and held/paused left-boundary controls match. Right-boundary b5918261/891e57ae/c9b3814a saved routes match; main fast/full pass; later movement frontiers remain. Other movement frontiers remain. | Fuzz 0x00-0x05 x 8 seeds has no `character_movement`-first cluster; `gameplay_regression_*` tests load the shrunk repros. |
| 308 | [ ] | P2 | combat | Combat audit contact/routing gaps remain; new d31c1fb4 (0x00) differs on HP/damage after the standing-entry frontier clears. | Shrunk d31c1fb4 matches HP/damage; remaining audit branches need their own comparable routes. |
| 517 | [ ] | P2 | enemy | Grounded Diamond d4710e6c matches after shared-walker Y correction. Counts22f086c8/90c880cb and00c6aa8c/8ab6fa8c plus prior roster/diamond frontiers remain. | Each shrunk repro matches object counts and roster. |
| 518 | [ ] | P2 | kid-physics | Airborne velocity a4cec55f/6c5f8e27 (0x04/0x05),08342e19 (0x09) and2c788cfe (0x07); RNG offset/ring4317835b/1f2cfa0d/70019eed (0x04). | Each repro matches the ROM through its divergence frame. |
| 519 | [ ] | P2 | enemy | Idle fuzz control diverges on `enemy_roster` at 0x01 frame 99, so fuzz skips that level (`start_state`). 0x05 is idle-clean after the Diamond turn fix. | Idle 600-frame control matches on 0x01. |
| 521 | [~] | P2 | level | Settled-warp start state differs at frame0 on 5a69ed83 (0x0a/0x0b enemy roster); Lion Init RNG on0x07 is verified. | Frame0 enemy roster matches the ROM on0x0a and0x0b. |
| 522 | [~] | P1 | kid-physics | Slope-Down, flat crawl, held-through-cleanup2101fa23 and Juggernaut tile order match their witnesses. Camera14c281f6/34acf586 and40be89bd remain; RNG4317835b belongs to #518. | Entry and nonzero stand-up match 9819d44c. Replay remaining shrunk frontiers. |
