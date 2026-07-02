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

## NOW

- **Select work with `mise run project-status`** (priority-ordered; take its top ready/open item, do not eyeball file order).
- **2026-07-02 failed human validation:** the recently reconfirmed rows have
  been split into fixed candidates or route-needed blockers. The repeated
  failure pattern is now the bug: fixed-candidate probes and synthetic routes
  are not exercising the live play paths the human keeps seeing.
- **#210/#254 fixed candidate 2026-07-02:** rapid title/menu row-select now
  carries the resident Red Stealth/title-menu VDP during the hidden
  `loc_1BBD6 -> loc_6E2 -> loc_1AE2E` title-card warm-up. Raw VDP
  native/oracle captures are exact for the formerly black-square scene-swap
  windows on `2400:start,2700:start,2720:start` and
  `2400:start,2698:start,2718:start`; rows moved to `[?]` for human replay.
- **#369 flagpole base candidate 2026-07-02:** second root-cause pass found the
  base was present in scene commands but could still be masked because native
  dropped `Flagpole_Boss`'s `priority(a3)=1` on both flag sprites. Candidate is
  in `[?]`; human should retest natural EOL and verify the base survives over
  foreground.
- **#271 route-needed 2026-07-02:** moved to `[b]` after the current route set
  failed to reproduce a native-only Tank stomp bug. Known current routes are not
  the live failure: `issue271-route-current-20260702` is a miss,
  `issue271-sharp-current-20260702` is oracle-parity side/edge damage, and
  `issue271-sharp-sweep-jump184-20260702` is oracle-parity same-frame stomp.
  Next useful artefact: live targeted `play-native --record` route with
  `--record-target-enemy 0A/0B --record-stop-on-target-enemy-contact`, then run
  the generated verifier/frontier/scorer commands before changing combat.
- **#252 route-needed 2026-07-02:** repeated current capture/test evidence
  after the #210/#254 fix still shows coherent RockTank body/turret/bullet art;
  move the tank/helmet garble report to `[b]` until an exact live frame or
  recording identifies the bad sprite.
- **#259 route-needed 2026-07-02:** repeated current target-route scoring still
  does not produce an oracle-valid side-by-side for the reported BLW2/Highwater
  floor-sink. Current BLW2 route reaches only divergent 0x0A contact; Highwater
  never activates a 0x0B tank. Move #259 to `[b]`; do not retune Tank
  Y/priority/foreground occlusion without an exact live screenshot/recording/frame
  and native/oracle side-by-side.
- **#324 route-needed 2026-07-02:** current BLW1 row-1 and Highwater row-97
  raw/scene/oracle captures still do not reproduce a native-only wrong
  prize/P-block sprite. Highwater frame 48's static source-cell P during a
  shifted bounce is also present in oracle, so do not patch that as a bug.
  Move #324 to `[b]`; next useful artefact is the exact live screenshot,
  recording, or frame where the wrong sprite is visible.
- **Immediate work order:** take the next `project-status` item; #210/#254,
  #252, #259, and #324 are back out of the open queue.
- **Visual route work:** do not retune Tank Y/priority, block wobble sprites,
  or title/menu VDP from stale direct-camera or clean synthetic routes.
- **Proactive verification 2026-07-02:** movement title-card activation
  constants now have an ASM-cited regression. The movement shard exposed two
  surviving mutants on `LEVEL_ENTRY_TITLE_DEFERRED_NUMBER_VISIBLE_FRAME`
  (`+1 -> -1/*1`); the new loc_1AB26/loc_1AD48 test pins letters at frame 43
  and the deferred level-number worker exactly one frame later. Targeted
  mutation rerun `/tmp/mutants-movement-title-frame-20260702`: 2 tested, 2
  caught.

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
