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
- **2026-06-25 HUMAN PLAYTEST TRIAGE (build = last pushed commit `253e98961`).** 17 observations
  triaged; full per-issue evidence in the archive block "## 2026-06-25 playtest triage". NET:
  - **4 fixed-candidates FAILED validation → reopened to the open queue with next-step
    hypotheses:** #210/#254 (intro skip still garbles — de-latch frame constants), #324 (prize
    block wrong/"witch" sprite), #335 (Hills 0x08 skip-entry instant death — freeze murder wall
    during the entry title card), #339 (USM load drone — extend StopMusic to death/respawn/skip).
  - **#336 REOPENED `[-]`→`[ ]` (P3).** Human cross-emulator evidence disputes the sky-float
    NOT-A-BUG; prior closure proved camera/Kid-Y WRAM only, never the palette fade-in timing.
  - **3 NEW rows:** #357 (Lion one-stomp death as base Kid), #358 (Cloud lightning renders as a
    red dot not the bolt sprite — 3-file render fix), #359 (Cyclone-level Sphere missing at spawn).
  - **Route-blocked reconfirmed LIVE on HEAD (stay `[b]`; sharper repro in the triage block):**
    #271 (stomp hurts at the L2 teleporter tank + L3 first tank), #259 (those same tanks sit in
    the floor), #249/#270 (slope-into-wall upward float now confirmed on BOTH left/right slopes —
    level named: **Under Skull Mountain 3, 0x06**), #345 (Lion vanish, base Kid).
- **#360 opened (`[ ]` P1 infra):** record-inputs capture flow to turn natural playtests into
  oracle-comparable routes — the unblock-key for the route-blocked tank/slope/Lion/Hills items.
- **CONFIRMED CORRECT (not bugs):** Storm-Mtn overhead "head guy" = EmoRock (0x1B) hover-track,
  no projectile (only #333 render approximation open). Lightning firing ~4 shots = correct count.
- **POSITIVES this playtest:** continues load; level 2 clean; Cyclone spin renders (#355); Tar/
  blob render improved (#243); Hand grab+slow works (#185 gameplay).
- **Render-gap lesson stands:** headless CRC/state gates pass while render-only gaps ship; every
  missing-sprite fix needs a native-player RENDER assertion or capture-frames diff.

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
| 335 | [ ] | P0 | gameflow/death | 2026-06-25 playtest: entering Hills of the Warrior 1 (0x08) via level-skip still instant-kills Kid unless Start is tapped immediately; the 2026-06-22 natural-route candidate FAILED. | Hypothesis: the murder wall must not age/advance while the entry title card is up (cross-ref #266 skip-tail pre-age). Build a headless title-card-then-live Hills repro. See archive #335. |
| 339 | [ ] | P1 | audio | 2026-06-25 playtest: the looping "washing" drone still plays on the Under-Skull-Mountain load; the 2026-06-23 StopMusic-on-load candidate did not cover the skip/death entry path. | Extend the `$1F` drive-drone stop to the death (DeathWindup), respawn bulk-clear, and debug-skip teardown paths; verify via the audio-events transcript. Real-device listen still required to close. See archive #339. |
| 336 | [ ] | P3 | render/intro | 2026-06-25: human disputes the NOT-A-BUG closure with cross-emulator evidence — the original fades from black and scrolls up from the ground, Kid not seen floating in the sky. | Prior closure proved camera/Kid-Y WRAM parity only, never palette fade-in timing. Probe oracle CRAM/palette fade vs the camera ramp: native may reveal Kid mid-sky during frames the ROM keeps black. See archive #336. |
| 357 | [ ] | P1 | enemy | 2026-06-25 playtest: the Lion died to a single stomp as plain base Kid (and/or vanished). ASM Lion HP = enemy_level+6 (min 6); a kidabove stomp should only decrement `$44` by 1 (Lion.asm loc_3E32C), needing ~6 hits. | Probe a base-Kid stomp on a full-HP Lion; confirm it needs ~6 hits, not 1, and does not vanish. Check the stomp reaches the live HP-decrement path (#315) vs a generic kill. Cross-ref #186/#345. |
| 358 | [ ] | P2 | render/enemy | 2026-06-25 playtest: lightning clouds fire plain red dots; the ROM Cloud projectile (Cloud.asm loc_35DF0) renders a lightning-bolt sprite. Gameplay/lifetime is correct (#198); only the sprite is wrong. | Add `ActiveProjectileKind::CloudLightning` (types.rs), map it in trace.rs, draw the Cloud bolt art in scene.rs — same 3-file pattern as Tank/Tar/Lion bullets. capture-frames diff on a lightning-cloud level. |
| 359 | [ ] | P2 | enemy/level | 2026-06-25 playtest: on the Cyclone-form sky level the bouncing ball (Sphere 0x06) usually at the spawn point is missing. Untracked before now. | Confirm the exact level id and entry path (level-skip vs natural). Check the enemy-loading activation window and the level-skip roster join (#334). See 2026-06-25 triage. |
| 360 | [ ] | P1 | infra | 2026-06-25 requested enabler: a record-inputs flow turning a natural live playtest into an oracle-comparable route. Existing `--settled --record` is warp-only and rejects debug skips, so live routes to named tanks/slopes can't be captured. | Record natural inputs from a settled start to a named coordinate/enemy; emit a bundle staying native/oracle CRC-comparable to the target window. Unblock-key for #271/#259/#249/#270/#345/#335; first target USM3 0x06 slope. See archive #360. |
