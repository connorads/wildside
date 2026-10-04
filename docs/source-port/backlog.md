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

- **Current work (2026-10-04):** #186 hurt rendering follows loc_3E32C: C7E58 pose and unchanged palette2. Saved121-route regression and fresh oracle state match; visible hurt region matches104..120. Wider airborne/invulnerable hurt and natural entry remain unproven. Neutral extension first drifts at148 Kid X velocity, a separate physics lead.
- **Intake:** Fixed fuzz 0x00-0x05 x8 seeds retains26 tracked clusters. Ledger has10 native exits and0/103 confirmed. Fresh07 Flag3201 is blocked by cb337ac6 at104 kid_x_pos; its104-input witness is registered under #522. Prior07 routes and #518's2c788cfe remain preserved.
- **Scope:** ADR0004 sets the fidelity floor: gameplay, enemy and RNG state, level exits, visible or audible bugs. Pixel baseline and injection ratchet stay.
- **After each fix:** Re-run fixed fuzz and affected ledger levels with `--write`; extend the ledger beyond0x10 once early levels confirm.
- **Jev allowance:** $5 additional across the campaign. #185 used48 requests in one attempt; lifetime charged upper bound $0.257086284.
- **Jev exploration review:** Review1791110086680455000 on16cb22342 plus this terrain candidate selects P1 #169 Drips. Settled01 state0..5 matches; EmoRock pixels differ349/350 on2/4, so strict preflight stops before collection. No navigation or paid request. #333 owns the renderer surface. One fix since review; review after two more.
- **Evidence boundary:** Cached Lion161, sprite78 and CLI Lion9 pass. Fast7511 plus35 CLI gameplay regressions and full37759 pass; full report is identical to parent. Sweep102/105 and fixed fuzz26 clusters retain baseline;26 generated fixture overwrites restored. Live SDL121 replays without drift. Unrelated projectile pixels, raw audio-state parity, natural entry, wider hurt and human play stability remain unproven. Recording export padding remains separate.

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
| 515 | [~] | P1 | kid-physics | Native derives MoveID; the ROM stores it at coroutine write sites. 10/26 fuzz clusters (20 routes) differ first: 7b0a8e1e, 3a36f8ac, 5a673417, a5798943, b5918261, 891e57ae, c9b3814a, c96a3c9d. | Fuzz 0x00-0x05 x 8 seeds has no `character_movement`-first cluster; `gameplay_regression_*` tests load the shrunk repros. |
| 517 | [ ] | P2 | enemy | Fuzz clusters 022889be (0x00 f309), d1446380 and 72129c1a (0x03), e9b6a11e (0x02): native keeps a different live object set or roster. | Each shrunk repro matches object counts and roster. |
| 518 | [ ] | P2 | kid-physics | Fuzz clusters 6c5f8e27 (0x04 f260, airborne `kid_x_vel`), 1f2cfa0d (0x04 f203, RNG offset/ring) and 70019eed (0x04 f127, RNG offset/ring); ledger08342e19 (0x09 f304) and2c788cfe (0x07 f45), airborne velocity. | Each repro matches the ROM through its divergence frame. |
| 519 | [ ] | P2 | enemy | Idle fuzz control diverges on `enemy_roster` at 0x01 frame 99, so fuzz skips that level (`start_state`). 0x05 is idle-clean after the Diamond turn fix. | Idle 600-frame control matches on 0x01. |
| 521 | [~] | P2 | level | Settled-warp start state differs at frame0 on 5a69ed83 (0x0a/0x0b enemy roster); Lion Init RNG on0x07 is verified. | Frame0 enemy roster matches the ROM on0x0a and0x0b. |
| 522 | [~] | P1 | kid-physics | Remaining crawl, jump, slope and zero-velocity stand-up physics. Clusters 0ed35d38/f92931b6/da81e8c7/417af3e3/ba78e9a9/0b645d5b/d38053c6/70d46852/ba3911e6/6a155c95; ledger 594f9474/6709165a/e1e5e429/b17bcd1f/7a06941b/a44fc576/cb337ac6. | Entry and nonzero stand-up match 9819d44c. Replay remaining shrunk frontiers. |
