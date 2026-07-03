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
  fallback passes fixed compact ASM gaps for Armadillo, Goat, Sphere, Diamond,
  Driller, Fire Demon, Cloud, Tornado, Emo Rock, Robot, Berzerker charge
  contact, projectiles `stru_37BFC` timing, HeadyMetal `sub_37ABC` body-piece
  render markers, frame-scene Fire Demon/EOL/Plane-B row history, Cloud
  `loc_35D9E` attack-animation timing, VDP decompression wrapper invariants
  around `loc_14300/loc_14314`, and now VDP intro text cleanup stream-progress
  invariants around `loc_1C7B6/loc_1C7D2`.
- **Current gates 2026-07-03:** focused `decompress_to_vram` tests pass, and
  final VDP focused mutation rerun `.cache/mutants/vdp-decompress-20260703-after2`
  reports 32 mutants: 31 caught, 1 unviable, 0 missed, 0 timeouts. Focused
  `intro_text_` tests also pass, and final intro-text mutation rerun
  `.cache/mutants/vdp-intro-text-20260703-after` reports 15 mutants: 14 caught,
  1 unviable, 0 missed, 0 timeouts. The latest change is behaviour-neutral
  debug/test hardening; `cargo fmt --all --check`, `git diff --check`, `mise run
  backlog-lint`, and `mise run verify-slice-fast` pass. `verify-tas-full` was
  not rerun. Latest full TAS remains the 2026-07-02 pass with CRC all 37759
  frames matching and the known `1p-accurate` state gap at
  `transition_15_to_00_post` DragonWalking/object-count. ASM coverage remains
  `0` ordinary uncovered labels plus 21 emulated GEMS wrappers; do not cite
  those wrappers just to move the metric.

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
