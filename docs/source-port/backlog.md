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

- **Current work (2026-10-02):** #520 Dragon initial floor support is the next investigation. #522 entry and retained-velocity stand-up are verified; zero-velocity stand-up remains open.
- **Intake:** `fuzz-summary.yaml` (0x00-0x05 x 8 seeds) has 26 clusters, tracked by #515/#517-#519/#522. `level-ledger.yaml` (0x00-0x10) has 10/17 native exits and 0/103 confirmed; every blocker is tracked by #515, #520, #521, #522 or #524.
- **Scope:** ADR 0004 sets the fidelity floor: gameplay, enemy and RNG state, level exits, visible or audible bugs. #406/#409/#416 closed under it; the pixel baseline and injection ratchet stay.
- **After each fix:** re-run `wildside fuzz run --levels 0x00-0x05 --seeds 8 --frames 600 --write` and `wildside ledger run --write`; extend the ledger past 0x10 once early levels confirm.
- **Jev allowance:** $5 additional across the campaign. #185 used 48 requests in one attempt. The ledger lifetime charged upper bound is $0.257086284.
- **Jev exploration review:** Supported-edge candidate on `59eca7483`. Bounded #186 recoil discovery remains blocked by frame-0 RNG and 1,702 non-HUD pixels. Zero API requests or spend. Restart the three-fix cadence from this review.
- **Evidence boundary:** Fast and full gates verify projectile removal and crawl transitions. The published pixel/GEMS baselines are unchanged; the 105-level sweep retains three prior failed levels. Ledger remains 0/103 confirmed with ten native exits.

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
| 515 | [~] | P1 | kid-physics | Native derives MoveID; the ROM stores it at coroutine write sites. 10/26 fuzz clusters (20 routes) differ first: 7b0a8e1e, 3a36f8ac, 5a673417, a5798943, b5918261, 891e57ae, c9b3814a. | Fuzz 0x00-0x05 x 8 seeds has no `character_movement`-first cluster; `gameplay_regression_*` tests load the shrunk repros. |
| 517 | [ ] | P2 | enemy | Fuzz clusters 022889be (0x00 f309), d1446380 and 72129c1a (0x03), e9b6a11e (0x02): native keeps a different live object set or roster. | Each shrunk repro matches object counts and roster. |
| 518 | [ ] | P2 | kid-physics | Fuzz clusters 6c5f8e27 (0x04 f260, airborne `kid_x_vel`), 1f2cfa0d (0x04 f203, RNG offset/ring) and 70019eed (0x04 f127, RNG offset/ring). | Each repro matches the ROM through its divergence frame. |
| 519 | [ ] | P2 | enemy | Idle fuzz control diverges on `enemy_roster` at 0x01 frame 99, so fuzz skips that level (`start_state`). 0x05 is idle-clean after the Diamond turn fix. | Idle 600-frame control matches on 0x01. |
| 520 | [~] | P1 | enemy | Shared walker init/falling and Dragon world-edge turns match saved routes. Ledger 8d048d89 exposes a Dragon patrol turn one wake early at frame264. | Compare patrol timer, packed timer pair and state across init, wall turn and frames263-265. |
| 521 | [ ] | P2 | level | Settled-warp start state differs at frame 0 with no input: 2f5fb5c0 (0x07 RNG offset/ring) and 5a69ed83 (0x0a/0x0b enemy roster). | Frame 0 of the settled warp matches the ROM on 0x07, 0x0a and 0x0b. |
| 522 | [~] | P1 | kid-physics | Remaining crawl, jump, slope and zero-velocity stand-up physics. Clusters 0ed35d38/f92931b6/da81e8c7/417af3e3/ba78e9a9/0b645d5b/d38053c6/70d46852/ba3911e6; ledger 594f9474/6709165a/e1e5e429/b17bcd1f/7a06941b/a44fc576. | Entry and nonzero stand-up match 9819d44c. Replay remaining shrunk frontiers. |
| 524 | [ ] | P1 | player | Ledger 5aef09ec differs on damage, helmet transformation and Kid motion at 0x09 frame274. | Replay the shrunk route; compare contact, hitpoints and transformation start with the ROM. |
