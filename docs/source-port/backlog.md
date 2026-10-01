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

- **Current work (2026-10-01):** #515 is next. Port the ROM's `Character_Movement` MoveID writes in place of `current_character_movement`; it owns 18 fuzz clusters and blocks the 0x00/0x01/0x04/0x05 ledger exits. Start from `tests/data/fuzz-repros/54fce7a1.json` (3 inputs: Down arrives while walking; native flips to crawl a frame before the ROM). #186 stays `[~]`.
- **Intake:** `fuzz-summary.yaml` (0x00-0x05 x 8 seeds) has 23 clusters, all tracked by #515-#519. `level-ledger.yaml` (0x00-0x10) has 10/17 native exits and 0/103 confirmed; every blocker is tracked by #515, #520 or #521.
- **Scope:** ADR 0004 sets the fidelity floor: gameplay, enemy and RNG state, level exits, visible or audible bugs. #406/#409/#416 closed under it; the pixel baseline and injection ratchet stay.
- **After each fix:** re-run `wildside fuzz run --levels 0x00-0x05 --seeds 8 --frames 600 --write` and `wildside ledger run --write`; extend the ledger past 0x10 once early levels confirm.
- **Jev allowance:** $5 additional across the campaign. #185 used 48 requests in one attempt. The ledger lifetime charged upper bound is $0.257086284.
- **Jev exploration review:** Revision `cf5d84937`. No request or spend occurred. Restart the three-fix cadence from this review.
- **Evidence boundary:** The published-route state/RNG gates cover one route; `verify-tas-full` passed at `047b09300`. The ledger and fuzz census measure the rest.

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
| 520 | [ ] | P1 | enemy | Ledger routes diverge on `enemy_roster` before their exits: 5c73a00e (0x03 f171), a8918bf2 (0x09 f80), d0eff6e6 (0x0e f133). | Each repro in `tests/data/fuzz-repros/` matches the roster; the levels re-run in the ledger. |
| 521 | [ ] | P2 | level | Settled-warp start state differs at frame 0 with no input: 2f5fb5c0 (0x07 RNG offset/ring) and 5a69ed83 (0x0a/0x0b enemy roster). | Frame 0 of the settled warp matches the ROM on 0x07, 0x0a and 0x0b. |
