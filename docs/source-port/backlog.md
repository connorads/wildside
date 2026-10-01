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

- **Current work (2026-09-28):** #186 is active. The checked-in Level 0x07 routes match 120 roar/contact frames and 67 live-loader gameplay frames. The live-loader enemy frontier retains four frame 62..65 airborne-schedule diagnostics. Resolve them, then capture a surviving stomp through recoil and recovery.
- **Selection:** #169 remains parked on non-comparable settled starts. #186 is the next P1 objective gap; loader and ledge gameplay parity do not prove the enemy frontier, recoil or recovery.
- **Scope:** Single-player across all playable levels, branches, forms and bosses. Invisible timing parity is out of scope per ADR 0004; the pixel baseline and injection ratchet stay. Final package retains subjective/device checks; no human validation is claimed.
- **Jev allowance:** $5 additional across the campaign. #185 used 48 requests in one attempt. The ledger lifetime charged upper bound is $0.257086284. The checked-in route and later comparisons need no further paid navigation.
- **Jev exploration review:** Revision `cf5d84937`. #433 was the highest-priority eligible gap, but its 20-minute case window had expired. Offline ROM graph discovery replaced the false no-entry premise and found the natural Crypt chain. No request or spend occurred. Restart the three-fix cadence from this review.
- **#185 evidence:** `tests/data/dynamic-scenarios/issue-185-hand-natural-grab.json` covers frames 0..127. Strict enemy comparison reports 128 matches and zero divergences. Offline JEV replay matches trace and emulator state. The remaining image bounds contain the documented HUD life icon and Kid slope frames owned by #211, with no standalone Hand residual.
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
| 186 | [~] | P1 | enemy | Fight a Lion; hurt recoil, ledge leap, roar windup, and engagement range should behave like the ROM. | Resolve live-loader pose pixels; capture surviving-stomp recoil and recovery. |
| 515 | [ ] | P1 | kid-physics | Native derives `character_movement` each frame (`current_character_movement`); the ROM stores MoveID at its coroutine write sites. 16 of 23 fuzz clusters (24 routes) diverge here first, e.g. 7b0a8e1e, 3a36f8ac, 54fce7a1, 87a37ef6. | Fuzz 0x00-0x05 x 8 seeds has no `character_movement`-first cluster; `gameplay_regression_*` tests load the shrunk repros. |
| 516 | [ ] | P1 | kid-physics | Fuzz cluster 60979027: on 0x04 Kid position, x/y velocity and camera diverge at frame 98 while moving right. | Shrunk repro `tests/data/fuzz-repros/60979027.json` matches the ROM; regression test. |
| 517 | [ ] | P2 | enemy | Fuzz clusters 022889be (0x00 f309), d1446380 and 72129c1a (0x03), e9b6a11e (0x02): native keeps a different live object set or roster. | Each shrunk repro matches object counts and roster. |
| 518 | [ ] | P2 | kid-physics | Fuzz clusters 6c5f8e27 (0x04 f260, airborne `kid_x_vel`) and 1f2cfa0d (0x04 f203, RNG offset/ring). | Both repros match the ROM through their divergence frames. |
| 519 | [ ] | P2 | enemy | Idle fuzz controls diverge on `enemy_roster` at 0x01 frame 99 and 0x05 frame 71, so fuzz skips both levels (`start_state`). | Idle 600-frame control matches on 0x01 and 0x05. |
