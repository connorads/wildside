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
- **2026-06-29 loop handoff:** #254 IntroSequence6 Start-skip is a fixed candidate and moved
  to `[?]` for human validation. Native now carries the natural `$28` VDP lineage only while
  `loc_1BAB0` is before `loc_1B446`'s first presented hidden-shell frame; Start 2431/2432 takes the
  normal `Load_TitleScreen` rebuild instead of the blue-shell carry.
- **Evidence:** Start=2431/2432 native-vs-oracle frames are exact at 2480, 2500, and 2520 after the
  fix (previously ~67k changed pixels at 2500/2520); Start=2420/2430 still preserves the intended
  hidden-shell carry, and Start=2400 keeps the previous tiny AE 22/8 title-detail residue only.
- **Verification:** focused cache-backed IntroSequence6 regressions, native/oracle frame captures,
  ImageMagick AE comparisons, `mise run verify-slice-fast`, and `mise run verify-tas-full` completed.
- **Next selector output:** take #324 first under visual risks, then #349, #358, and #361.

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
| 324 | [ ] | P1 | render/block | 2026-06-25 playtest: a prize block still shows a wrong/"witch" sprite (foreground bleeding through the wobble's transparent slots); the 2026-06-23 candidate FAILED validation. | Get an oracle-comparable BLW1 forest prize-wobble frame-diff before another change; do not blind-revert the scene.rs terrain-suppression guards. See archive #324 + 2026-06-25 triage. |
| 358 | [ ] | P2 | render/enemy | 2026-06-25 playtest: lightning clouds fire plain red dots; the ROM Cloud projectile (Cloud.asm loc_35DF0) renders a lightning-bolt sprite. Gameplay/lifetime is correct (#198); only the sprite is wrong. | Add `ActiveProjectileKind::CloudLightning` (types.rs), map it in trace.rs, draw the Cloud bolt art in scene.rs — same 3-file pattern as Tank/Tar/Lion bullets. capture-frames diff on a lightning-cloud level. |
| 349 | [ ] | P1 | render | 2026-06-26 playtest: post-death respawn card still shows the trees but NOT the level-NAME text; the 2026-06-21 fix (`b55244ad1`) IS in the build, so the candidate FAILED validation. | Re-investigate the respawn-card render path (InGame classification trace.rs:9177; glyph-worker filter; overlay window); add a native-player capture-frame assertion, not just the CRC-neutral render test. See archive #349. |
| 361 | [ ] | P2 | render/enemy | 2026-06-26 playtest: on the hand level the live Hand (0x17) animation free-runs/flickers ("spazzes") instead of holding body frames. Deferred #243/#333 follow-up, never given its own row. NOT the grab gating #185 (grab+slow works). | Add a `select_live_hand_frame` selector mirroring `select_live_tar_monster_frame`: hold Hand body frames off HandState, early-return None on hp==0 so the death table keeps death frames; capture-frames to confirm no flicker. Cross-ref #243/#333, #185. |
