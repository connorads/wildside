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
- **2026-07-02 failed human validation:** #271, #324, #259, #252, #210, and
  #254 are reopened from the awaiting-human lane. The repeated failure
  pattern is now the bug: fixed-candidate probes and synthetic routes are not
  exercising the live play paths the human keeps seeing.
- **#369 flagpole base candidate 2026-07-02:** second root-cause pass found the
  base was present in scene commands but could still be masked because native
  dropped `Flagpole_Boss`'s `priority(a3)=1` on both flag sprites. Candidate is
  in `[?]`; human should retest natural EOL and verify the base survives over
  foreground.
- **#271 route-needed 2026-07-02:** moved to `[b]` after the current route set
  failed to reproduce a native-only Tank stomp bug. Use the new
  `probe-record-scenario-file --target-enemy-contact 0A/0B` scorer on the next
  live candidate route. Recording bundles with `--record-target-enemy 0A/0B`
  now write ready-made `route_target_score_commands` in
  `manifest.json`/`README.txt` and print the frontier/scorer commands on save;
  add `--record-stop-on-target-enemy-contact` for the next live capture so the
  bundle auto-saves just after the first target contact. Known current routes
  are not the live failure:
  `issue271-route-current-20260702` is a miss, and
  `issue271-sharp-current-20260702` is side/edge damage in both native and
  oracle. `issue271-sharp-sweep-jump184-20260702`, materialised from a
  `jump=184-206:4:1:right+b+c` batch, is a same-frame Tank stomp at frame 223
  and focused `tas-diff` frames 220..227 are MATCH, so nearby synthetic
  first-overlap stomps are also not the live failure. The next useful artefact
  is a live targeted `play-native --record` route for the reported failing
  jump. Then run the generated `verify-recording` /
  `tas_frontier_command` before changing combat. Do not retry generic left
  extra-step/left boundary patches from the older sharp-route phase note:
  current `tas-diff` reports frames 0..5 MATCH, frames 216..221 transiently
  divergent, then frames 222..228 MATCH including the damage contact at
  224/227. Also do not chase the old full-frame idle frame-300 RockTank
  assertion: current `idle_level_full_frames_01` oracle matches native at body/turret
  `$00C48000/$00C40000`; that cache-backed test expectation was stale.
- **Immediate work order:** take #210/#254 next. They share the rapid
  menu/title-skip black-square and Red Stealth/title corruption report; capture
  or synthesise exact Start timings, then compare raw VDP/scene frames before
  changing menu or form rendering.
- **Visual route work:** #324/#259/#252/#210 need current live recording or
  raw-VDP/scene paired captures before another renderer patch. Do not retune
  Tank Y/priority, block wobble sprites, or title/menu VDP from stale
  direct-camera or clean synthetic routes.
- **Proactive verification 2026-07-02:** Mini Hopping Skull air-damage /
  charging-Berzerker mutation shard has zero missed mutants
  (`/tmp/mutants-core-mini-hopping-skull-air-damage-20260702`: 12 tested, 7
  caught, 5 unviable).

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
| 210 | [ ] | P0 | gameflow/render | FAILED human validation 2026-07-02: rapid menu/intro skip can still produce black squares where Red Stealth/title/menu art should be. | Capture exact live Start timings or `play-native --record` bundle; compare raw VDP and scene frames around the first bad frame; do not rely on the covered `2400,2700,2720/2721` synthetic routes. |
| 254 | [ ] | P1 | intro/render | FAILED human validation 2026-07-02: Red Stealth/title-skip corruption persists as part of the #210 black-square route. | Pair with #210 route capture; isolate whether the bad pixels are carried VDP/title-menu state or a distinct transform/render path before changing form art. |
| 324 | [ ] | P1 | render/block | FAILED human validation 2026-07-02: prize/P block still shows wrong sprite/art behind the hit wobble, now visibly on top of the shake. | Capture current live screenshot/recording/frame; replay with `capture-scenario-frames --scene` and raw VDP; inspect block/effects/terrain layers before another `sub_1007A` renderer change. |
| 259 | [ ] | P1 | enemy/render | FAILED human validation 2026-07-02: RockTanks still appear halfway in the floor on levels 2 and 3. | Record natural route to named BLW2/Highwater tanks; tas-diff body/turret Y plus foreground occlusion depth; only close as faithful if oracle side-by-side shows the same composite. |
| 252 | [ ] | P1 | render/enemy | FAILED human validation 2026-07-02: tank/helmet-looking RockTank garble still appears in live play despite clean sheet/capture probes. | Capture exact live frame/recording; compare assembled body/turret/muzzle/bullet sprites and raw VDP tiles at that frame; route floor perception through #259 if geometry matches. |
