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
- **2026-06-29 loop handoff:** #336 render/intro is fixed and moved to `[x]`.
  Native now keeps the BLW1 title-card -> live-gameplay handoff black through the ROM load window,
  then applies the oracle `sub_73D0`/`sub_14C0` palette fade while preserving the verified
  Kid/camera ramp.
- **Evidence:** route `.cache/debug/issue336-cold-boot-live/scenario-card-start.json` is
  CRC/checkpoint-clean for frames 880..1041. Captures match oracle black exactly at frames 960/970;
  fade brightness tracks oracle at frames 974/980/984, with residual AE from existing scene/HUD/tile
  differences rather than fade timing.
- **Verification:** focused core fade trace test, frame palette-fade tests, CLI scene-command test,
  `cargo check -q -p wildside-frame -p wildside-cli --tests`, route `verify-tas`, captures, and
  `mise run verify-slice-fast` exit 0 completed.
- **Next selector output:** no ready-to-implement rows; `project-status` now leads with #368 in
  playability/infra/docs. Top visual risks remain #210, #254, #324, #349, then #358.

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
| 210 | [ ] | P0 | gameflow/render | 2026-06-25 playtest (HEAD): skipping the intro still shows black/garbled graphics instead of the title/Red-Stealth pose; the 2026-06-23 fixed candidate FAILED human validation. | Carry the last composited VDP to `$2C` on ANY intro Start-skip frame; do not latch frame-specific constants (the rejected anti-pattern). Capturing the exact skip frame sharpens it. See archive #210 + 2026-06-25 triage. |
| 254 | [ ] | P1 | intro/render | 2026-06-25 playtest: same cluster as #210 — IntroSequence6 Start-skip still garbles (black-outline/blue shell); the 2026-06-23 candidate FAILED validation. | Engine-level VDP carry on any skip frame, no frame constants. Pair with #210. See archive #254 + 2026-06-25 triage. |
| 324 | [ ] | P1 | render/block | 2026-06-25 playtest: a prize block still shows a wrong/"witch" sprite (foreground bleeding through the wobble's transparent slots); the 2026-06-23 candidate FAILED validation. | Get an oracle-comparable BLW1 forest prize-wobble frame-diff before another change; do not blind-revert the scene.rs terrain-suppression guards. See archive #324 + 2026-06-25 triage. |
| 358 | [ ] | P2 | render/enemy | 2026-06-25 playtest: lightning clouds fire plain red dots; the ROM Cloud projectile (Cloud.asm loc_35DF0) renders a lightning-bolt sprite. Gameplay/lifetime is correct (#198); only the sprite is wrong. | Add `ActiveProjectileKind::CloudLightning` (types.rs), map it in trace.rs, draw the Cloud bolt art in scene.rs — same 3-file pattern as Tank/Tar/Lion bullets. capture-frames diff on a lightning-cloud level. |
| 349 | [ ] | P1 | render | 2026-06-26 playtest: post-death respawn card still shows the trees but NOT the level-NAME text; the 2026-06-21 fix (`b55244ad1`) IS in the build, so the candidate FAILED validation. | Re-investigate the respawn-card render path (InGame classification trace.rs:9177; glyph-worker filter; overlay window); add a native-player capture-frame assertion, not just the CRC-neutral render test. See archive #349. |
| 361 | [ ] | P2 | render/enemy | 2026-06-26 playtest: on the hand level the live Hand (0x17) animation free-runs/flickers ("spazzes") instead of holding body frames. Deferred #243/#333 follow-up, never given its own row. NOT the grab gating #185 (grab+slow works). | Add a `select_live_hand_frame` selector mirroring `select_live_tar_monster_frame`: hold Hand body frames off HandState, early-return None on hp==0 so the death table keeps death frames; capture-frames to confirm no flicker. Cross-ref #243/#333, #185. |
| 368 | [ ] | P3 | infra/docs | One-off tidy (not gated): fix stale citations loc_2510C (combat.rs:179) + loc_23629 (native_player/mod.rs:2086); gate-message → core/data (F6); refresh audio-triggers refs (F5); dedup CLAUDE.md asm-coverage block + document `--check-required` (F4). | Stale citations gone; CLAUDE.md block single-source. See plan #368. |
