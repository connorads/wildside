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
- **Proactive verification 2026-07-02:** with no ready/open queue rows,
  mutation fallback covered `armadillo.rs` (34 caught, 12 unviable),
  `spinning_twins.rs` (105 caught, 21 unviable), and
  `big_hopping_skull.rs` (56 caught, 15 unviable). The `diamond.rs` mutation
  pass initially found one actionable zero-velocity ceiling boundary survivor;
  a regression now pins `kid.asm loc_3C2D0`/Diamond.asm `loc_3ADA4` and the
  rerun reports 62 caught, 10 unviable, 0 missed. ASM audits found nine live
  gameplay gaps. Armadillo.asm `loc_3C766`/`loc_3C7F8` now seeds the
  shared Scorpion.asm `loc_3DEA0` `$6E/$54` timer through
  `shared_walker_patrol_timer`, hibernating/reversing on expiry. Goat.asm
  `loc_3CC18` now runs before the live pacer step; turn routes through
  `loc_3CC36`/`loc_3CC46` clear velocity and hibernate through
  `loc_3E392`/`loc_3E546` instead of walking half a pixel in the old
  direction. Sphere.asm `loc_35F9E` now routes stored signed-negative,
  `colid_hurt`, and `colid_kidabove` collision words into
  Enemy06_Sphere_Kill `loc_36052` instead of ignoring them in the bouncer AI
  table. Diamond.asm `loc_3ADA4` now preserves high-HP `colid_ceiling`,
  `colid_slopeup`, and `colid_slopedown` terrain words so `loc_3AE3E`,
  `loc_3AE4A`, and `loc_3AE68` can run instead of collapsing to no-collision or
  generic floor. Driller.asm `loc_3BF4E` and Fire_Demon.asm `loc_3A5BE` hold
  hurt animations. Cloud.asm `loc_35DAE` waits before `loc_35DB8` lightning;
  Tornado.asm `loc_35908` reloads on post-decrement zero; Emo_Rock.asm
  `loc_3621A` waits 67 ticks of `stru_3647E` before `loc_3625E` gravity.
  `verify-slice-fast` passes; `verify-tas-full` exits 0 with CRC all 37759
  frames matching but still reports the current `1p-accurate` state gap (first
  authoritative diff: `transition_15_to_00_post` DragonWalking/object-count).
  ASM coverage remains `0` ordinary uncovered labels plus 21 emulated GEMS
  wrappers; do not cite those wrappers just to move the metric.

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
