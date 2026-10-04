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

- **Current work (2026-10-04):** #515 standing-entry provenance passes main fast/full and is committing separately after closeout. Isolated right-boundary continuation passes raw40/59/91, fast/full and sweep; affected05 ledger exposes later157 physics. #522 b492 slope continuation is source-proved next. #186 wider hurt/natural entry remains parked without actual-damage evidence.
- **Intake:** Fixed fuzz0x00-0x05 x8 seeds has24 clusters/31 divergent routes, with no earlier frontier against the stand-up parent. Five standing-entry signatures retire; new d31c1fb4 is shrunk300/15 nonzero under #308. All93 existing fixtures match backups. Ledger03 d471 belongs to #517;04 b492 to #522. Ledger remains10 native exits/0 of103 confirmed.
- **Scope:** ADR0004 sets the fidelity floor: gameplay, enemy and RNG state, level exits, visible or audible bugs. Pixel baseline and injection ratchet stay.
- **After each fix:** Re-run fixed fuzz and affected ledger levels with `--write`; extend the ledger beyond0x10 once early levels confirm.
- **Jev allowance:** $5 additional across the campaign. #185 used48 requests in one attempt; lifetime charged upper bound $0.257086284.
- **Jev exploration review:** Review1791123871374521000 on a5554be41 plus stand-up candidate selects P1 #169 Drips. Settled01 state0..5 matches; EmoRock pixels349/350 on2/4 block strict preflight before collection. No navigation/paid request. #333 owns that renderer surface. The stand-up unit is the third gameplay fix since the prior review; this review resets the cadence.
- **Evidence boundary:** Post-selector fast7511/38 and full37759 pass; full report equals parent. Sweep102/105 retains baseline. Actual SDL296 and native replay pass; scene292/293 retains84 HUD pixels each. Boundary52 residue improves758 to345 but actual mapping is proved skid88. Cached crawl141/1 has an identical parent Drips failure. Wider Lion hurt, raw audio-state parity, natural entry and human stability remain unproved.

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
| 515 | [~] | P1 | kid-physics | Standing-entry99485662, zero-speed stand-up and held/paused left-boundary controls match. Root cause identified for b5918261 right-boundary continuation; separate promotion pending. Other movement frontiers remain. | Fuzz 0x00-0x05 x 8 seeds has no `character_movement`-first cluster; `gameplay_regression_*` tests load the shrunk repros. |
| 308 | [ ] | P2 | combat | Combat audit contact/routing gaps remain; new d31c1fb4 (0x00) differs on HP/damage after the standing-entry frontier clears. | Shrunk d31c1fb4 matches HP/damage; remaining audit branches need their own comparable routes. |
| 517 | [ ] | P2 | enemy | Native counts/roster and diamonds differ. Root cause identified for ledger03 d4710e6c grounded Diamond Y snap;00c6aa8c/8ab6fa8c and prior roster/diamond frontiers remain. | Each shrunk repro matches object counts and roster. |
| 518 | [ ] | P2 | kid-physics | Fuzz clusters 6c5f8e27 (0x04 f260, airborne `kid_x_vel`), 1f2cfa0d (0x04 f203, RNG offset/ring) and 70019eed (0x04 f127, RNG offset/ring); ledger08342e19 (0x09 f304) and2c788cfe (0x07 f45), airborne velocity. | Each repro matches the ROM through its divergence frame. |
| 519 | [ ] | P2 | enemy | Idle fuzz control diverges on `enemy_roster` at 0x01 frame 99, so fuzz skips that level (`start_state`). 0x05 is idle-clean after the Diamond turn fix. | Idle 600-frame control matches on 0x01. |
| 521 | [~] | P2 | level | Settled-warp start state differs at frame0 on 5a69ed83 (0x0a/0x0b enemy roster); Lion Init RNG on0x07 is verified. | Frame0 enemy roster matches the ROM on0x0a and0x0b. |
| 522 | [~] | P1 | kid-physics | Zero-speed55fdeed4/6d09c11f stand-up matches. Root cause identified for b4924e7d slope continuation incorrectly entering crawl. Camera14c281f6/34acf586 and prior9cc019fb/physics blockers remain. | Entry and nonzero stand-up match 9819d44c. Replay remaining shrunk frontiers. |
