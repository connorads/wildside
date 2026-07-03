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
  been reconfirmed again by live play. Treat the prior route-needed parking as
  the process failure: fixed-candidate probes and synthetic routes are not
  exercising the live paths the human keeps seeing.
- **#271 resolved 2026-07-02:** human retest reports RockTank head contact no
  longer hurts Kid/Red Stealth ("no hurty") after the delayed Tank/TankShooting
  side-contact timing fix. Keep #259 separate for the still-open floor/visual
  placement report.
- **#259 blocked 2026-07-02:** the current BLW2 #271 contact frame now has
  `rock_tank_contact_probe`, enemy-only, terrain-mask, native composite, and
  oracle composite evidence. It shows oracle-matched foreground occlusion, not
  a native-only Tank Y/priority fault. The unresolved part is the exact later
  BLW2/Highwater live frame the human still sees; do not retune Tank placement
  without that frame.
- **#324 resolved 2026-07-02:** human confirmed the prize/P block wobble is
  fixed after commit `00e7a6cb1`. The direct replay check remains verifier
  frame 840 from `playtest-2026-07-02-prize-wobble`: lifted P block, no
  diagonal wood leak underneath.
- **#369 blocked 2026-07-02:** current Highwater floor-contact evidence keeps
  `Flag_bottom.bin` and the waving flag visible through the ASM `loc_C0B6`
  cleanup window, with high-priority scene commands. The remaining live report
  needs the exact screenshot/recording/frame where the base is missing before
  retuning flag priority or cleanup timing.
- **#210/#254 remain suspect 2026-07-02:** menu-skip black squares/Red Stealth
  corruption was recently fixed-candidate, but recurrent live validation has
  failed enough times that the next playtest failure should reopen it
  immediately instead of asking for another proof route.
- **Visual route work:** do not retune Tank Y/priority, flagpole priority, or
  title/menu VDP from stale direct-camera or clean synthetic routes. If the
  route machinery cannot see the bug, improve the capture/probe surface first.
- **Proactive verification 2026-07-03:** no ready/open rows remain. Recent
  fallback passes covered compact ASM gaps across enemy contact/animation,
  projectiles, HeadyMetal render markers, frame-scene history, VDP decompression
  and intro-text invariants, physics camera/murder-wall branch hardening, and
  now scenario direct-title-card guards, `types.rs` ROM-facing contracts, and
  Hand, movement Intro Sequence 3/credits, plus weather storm/hail,
  lava-geyser render/script, hail worker `sub_61CA`, and weather runtime/init
  plus storm-init, Kid-hitbox, and lava-geyser hitbox-window/predicate/sprite
  helper boundaries, lava-geyser initial-delay/CRAM-rotation helpers, plus
  lava-fireball init/script/sprite-kind, vertical-window, reject/overlap/hitbox/live-hit edges,
  and storm `sub_6048` thunder/hail cadence plus live `$FFFFFB3D` free-slot
  shell routing, murder-wall/weather accessor cleanups, and compact ASM
  coverage promotions in Fireball/Drips/Mini Hopping Skull.
- **Current gates 2026-07-03:** clean mutation reruns now cover through storm
  `sub_6048` cadence, live `step_weather` free-slot routing, murder-wall
  warning dispatch, and weather ROM-facing label/path helpers.
  Latest focused clean reruns: types 211/204/7, Hand 50/45/5, movement 22/20/2
  + 21/20/1, weather 16/16/0 + 52/49/3 + 64/63/1 + 25/25/0 + 103/102/1 +
  65/51/14 + 92/92/0 + 16/15/1 + 14/14/0 + 7/6/1 + 26/25/1 + 22/22/0
  + 8/8/0 + 18/18/0 + 7/7/0 + 11/11/0 + 18/17/1 + 9/8/1 + 10/9/1
  + 11/11/0 + 12/11/1 + 73/71/2 + 35/34/1 + 20/20/0 + 47/45/2
  + 20/20/0 (mutants caught/unviable; all 0 missed). Latest compact ASM
  promotions now include hidden-block `loc_E3C0` and title-entry `loc_1ACC8`;
  test-only debt is 228; fast gate green.
  Latest full TAS (2026-07-03) exits 0 with CRC all 37759 frames matching and
  the known first state divergence at `transition_15_to_00_post`. ASM coverage
  remains `0` ordinary uncovered labels plus 21 emulated GEMS wrappers; do not
  cite those wrappers just to move the metric.

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
