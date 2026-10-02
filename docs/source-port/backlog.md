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

- **Current work (2026-10-02):** #186 surviving stomp latches104, consumes105 and resumes120 on the saved121 route. Fast/full, sweep and fixed fuzz pass their baselines. Ledger07 retains79692a83 at118. Next fix is upper-Lion roar countdown on79692a83: ROM fire118/resume145; native fire119/resume146. #515/#522 remain open.
- **Intake:** Fixed fuzz 0x00-0x05 x 8 seeds retains26 tracked clusters. Ledger has10 native exits and0/103 confirmed; fresh ledger07 reaches Flag2756/transition3228 but blocks on79692a83 at118. Original464160b3 route advances past103 to its genuine stomp latch104.
- **Scope:** ADR0004 sets the fidelity floor: gameplay, enemy and RNG state, level exits, visible or audible bugs. Pixel baseline and injection ratchet stay.
- **After each fix:** Re-run fixed fuzz and affected ledger levels with `--write`; extend the ledger beyond0x10 once early levels confirm.
- **Jev allowance:** $5 additional across the campaign. #185 used48 requests in one attempt; lifetime charged upper bound $0.257086284.
- **Jev exploration review:** Last review1790965033462302000 fails pixels47; the post-fire fix makes47-69 exact. No paid collection; one fix since review. A genuine surviving-stomp witness is available at104/105, with recovery drift120.
- **Evidence boundary:** Lion loader67, roar120 and saved turn83 match gameplay/enemy/RNG. Surviving-hurt121 matches gameplay/enemy; fast/full retain pixel/GEMS command baselines. Sweep102/105 retains1B/4C/5C. Roar42 pixel frames differ; HP8 return31 faces differ. Natural entry, airborne hurt and human play stability remain unproven.

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
| 186 | [~] | P1 | enemy | Fight a Lion; hurt recovery and engagement must match the ROM. Saved surviving-stomp121 matches; upper Lion fires one callback late118 on79692a83. | Root cause identified: roar countdown returns after its zero tick. ROM fire118/resume145; saved hurt121 matches. |
| 515 | [~] | P1 | kid-physics | Native derives MoveID; the ROM stores it at coroutine write sites. 10/26 fuzz clusters (20 routes) differ first: 7b0a8e1e, 3a36f8ac, 5a673417, a5798943, b5918261, 891e57ae, c9b3814a, c96a3c9d. | Fuzz 0x00-0x05 x 8 seeds has no `character_movement`-first cluster; `gameplay_regression_*` tests load the shrunk repros. |
| 517 | [ ] | P2 | enemy | Fuzz clusters 022889be (0x00 f309), d1446380 and 72129c1a (0x03), e9b6a11e (0x02): native keeps a different live object set or roster. | Each shrunk repro matches object counts and roster. |
| 518 | [ ] | P2 | kid-physics | Fuzz clusters 6c5f8e27 (0x04 f260, airborne `kid_x_vel`), 1f2cfa0d (0x04 f203, RNG offset/ring) and 70019eed (0x04 f127, RNG offset/ring); ledger08342e19 (0x09 f304, airborne velocity). | Each repro matches the ROM through its divergence frame. |
| 519 | [ ] | P2 | enemy | Idle fuzz control diverges on `enemy_roster` at 0x01 frame 99, so fuzz skips that level (`start_state`). 0x05 is idle-clean after the Diamond turn fix. | Idle 600-frame control matches on 0x01. |
| 521 | [~] | P2 | level | Settled-warp start state differs at frame0 on 5a69ed83 (0x0a/0x0b enemy roster); Lion Init RNG on0x07 is verified. | Frame0 enemy roster matches the ROM on0x0a and0x0b. |
| 522 | [~] | P1 | kid-physics | Remaining crawl, jump, slope and zero-velocity stand-up physics. Clusters 0ed35d38/f92931b6/da81e8c7/417af3e3/ba78e9a9/0b645d5b/d38053c6/70d46852/ba3911e6/6a155c95; ledger 594f9474/6709165a/e1e5e429/b17bcd1f/7a06941b/a44fc576. | Entry and nonzero stand-up match 9819d44c. Replay remaining shrunk frontiers. |
