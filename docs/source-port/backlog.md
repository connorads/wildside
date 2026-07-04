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
- **Proactive verification 2026-07-04:** no ready/open rows remain; latest addition is core trace opening-level object-count mutation hardening. The previous core trace Kid/object collision classifier, core boot end-sequence burst/falling/camera helper, core boot end-sequence text/page/palette helper, core boot options-menu text/draw helper, core boot end-sequence display/upload helper, core boot IO/fullscreen palette-fade helper, core boot continue-timeout title-return helper, core boot Sega-screen helper, core boot reset/startup helper, core boot pause-menu helper, core boot title/intro helper, CLI project-status selector, CLI asm-coverage classification, CLI scenarios-window, native-player route/contact, audio artefact, core demo, Fireball, Diamond, render tile, render palette, data title/menu, data background, data plane/mask, CLI view-tas, CLI hypothesis parser, CLI bootstrap layout, CLI capture helper, CLI TAS import/checkpoint, CLI sweep gameplay assertion, CLI sweep release-gate focus, CLI sweep frame-helper, CLI TAS-diff helper, CLI verify audio/window, CLI diff helper, CLI oracle helper, CLI compare gameplay-report, and CLI player-shell follow-ups are now closed separately with focused clean reruns.
- **Current gates 2026-07-04:** clean mutation reruns now cover through storm
  `sub_6048` cadence, live `step_weather` free-slot routing, murder-wall
  warning dispatch, weather label/path helpers, and the enemy module reruns
  listed below.
  Latest focused clean reruns: core trace opening-level object counts 32/31/1, core trace Kid/object collision classifier 20/19/1 (plus core boot level-layout/VDP spec helpers 12/0/12 unviable-only), core boot end-sequence burst/falling/camera helpers 44/36/8, core boot end-sequence text/page/palette helpers 21/10/11, core boot options-menu text/draw helpers 40/32/8, core boot end-sequence display/upload helpers 32/28/4, core boot IO/fullscreen palette-fade helpers 18/14/4, core boot continue-timeout title-return helpers 7/3/4 (plus continue-screen fade probe 2/0/2 unviable-only), core boot Sega-screen helpers 26/22/4, core boot reset/startup helpers 9/9/0, core boot pause-menu helpers 86/75/11, core boot title/intro helpers 127/91/36, CLI project-status selector/report 68/62/6, CLI asm-coverage classification helpers 21/19/2, CLI scenarios-window helpers 12/11/1, CLI player-shell helpers 77/73/4, CLI compare gameplay-report helpers 104/103/1, CLI oracle helpers 14/13/1, CLI diff helpers 91/91/0, CLI verify audio/window helpers 59/58/1 (helper target 0 missed; cargo-mutants also selected unrelated line-685 capture survivor), CLI TAS-diff helpers 127/126/1, CLI sweep frame helpers 8/5/3, CLI sweep release-gate focus 94/91/3, CLI sweep gameplay assertions 46/41/5, CLI TAS import/checkpoints 55/50/5, CLI capture helpers 25/25/0, CLI bootstrap layout 13/12/1, CLI hypothesis parser 19/19/0, CLI view-tas 54/49/5, data plane/mask 137/136/1, data background 68/63/5, data title/menu 48/39/9, render palette 35/33/2, render tiles 38/34/4, core demo 79/61/18, CLI native-player audio-artifacts 18/16/2, CLI case bootstrap/probe 46/46/0, CLI inspect target-contact 22/19/3, CLI main route-target 12/12/0, CLI recording route-target 13/13/0, constants 59/50/9, gfx-animation 18/16/2,
  scenario 98/80/18, types 211/204/7, Hand 50/45/5, movement 22/20/2 + 21/20/1,
  weather 16/16/0 + 52/49/3 + 64/63/1 + 25/25/0 + 103/102/1 +
  65/51/14 + 92/92/0 + 16/15/1 + 14/14/0 + 7/6/1 + 26/25/1 + 22/22/0
  + 8/8/0 + 18/18/0 + 7/7/0 + 11/11/0 + 18/17/1 + 9/8/1 + 10/9/1
  + 11/11/0 + 12/11/1 + 73/71/2 + 35/34/1 + 20/20/0 + 47/45/2
  + 20/20/0, Driller 24/19/5, Armadillo 46/34/12, Tornado 68/60/8, Tank 28/21/7 (mutants
  caught/unviable; all 0 missed). Latest compact ASM promotions now cover hidden-block, title-entry/VDP/KID-shell, gfx-animation, Lion-projectile slope, camera clamp, hidden-prize reveal-start, blank-top grid-shift, blank-top terrain-byte shift, leftward lead-cell collision, intro centre-particle, platform walking-carry, standing-platform crush, animated-solid coarse-cull layout-restore, shooter-projectile recycle, and Kid-projectile overlap-branch labels; test-only debt is 197; fast gate green.
  Friendly projectile overlap rerun is 85/79/6 equivalent after Sphere edge tests; remaining misses are empty-row guards and Maniaxe's default lifetime.
  Enemy module reruns are all 0 missed: Diamond 72 total (62/10), Fireball 60
  (47/13), Goat 23 (15/8), Fire Demon 48 (30/18), Scorpion 57 (46/11), Dragon 264 (222/42), Archer
  135 (124/11), Dragon Walking 136 (112/24), Big Hopping Skull 71 (56/15),
  Orca 91 (74/17), Robot 105 (93/12), Spinning Twins 126 (105/21), Cloud
  135 (119/16), Crab 87 (71/16), Emo Rock 92 (80/12), Mini Hopping Skull
  146 (90/56), Tar Monster 141 (107/34), Ninja 189 (156/33), UFO 232
  (194/38), Lion 251 (206/45), Drips 299 (234/65).
  Latest `1p-playable` gate passes: report 1783122908; gameplay sweep is 69 passed, 0 failed, 0 errors.
  Latest `1p-accurate` gate (2026-07-04) passes: report 1783122304; embedded
  sweep 1783122357 is 69/69 passed, 1037 oracle limitations, `unverified=69`.
  Latest full TAS (2026-07-03) exits 0 with CRC all 37759 frames matching and the
  known first state divergence at `transition_15_to_00_post`. ASM coverage
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
