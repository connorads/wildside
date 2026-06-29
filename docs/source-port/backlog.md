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
- **2026-06-29 loop handoff:** #360 record-inputs capture flow is now a fixed candidate in the
  awaiting-human lane. `play-native --record` can persist a named route target
  (`--record-target-label/level/point/cell/cell-hitbox/enemy`) in `manifest.json`/README and emits
  a `verify-tas --bisect --frontier-kind ...` command for the saved `scenario.json`.
- **Verification:** focused CLI recording/target tests, `cargo fmt --check`, and
  `mise run verify-slice-fast` are green (clippy, hooks, backlog-lint, TAS frontier, 6470 core
  tests, 9 CLI slice tests). `verify-tas-full` was not rerun; this is recording metadata/CLI
  infrastructure, not gameplay-state logic.
- **Human validation:** record a real route to the first target, e.g. USM3 0x06 slope with
  `--record-target-label usm3-0x06-slope --record-target-level 06 --record-target-cell ...`, then
  run the bundle's `verify-recording` and `tas_frontier_command` to confirm where oracle parity
  first leaves the route.
- **Next selector output:** no ready-to-implement rows; `project-status` now leads with #359 in
  playability risks and #210 in visual risks.

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
| 336 | [ ] | P3 | render/intro | 2026-06-25: human disputes the NOT-A-BUG closure with cross-emulator evidence — the original fades from black and scrolls up from the ground, Kid not seen floating in the sky. | Prior closure proved camera/Kid-Y WRAM parity only, never palette fade-in timing. Probe oracle CRAM/palette fade vs the camera ramp: native may reveal Kid mid-sky during frames the ROM keeps black. See archive #336. |
| 358 | [ ] | P2 | render/enemy | 2026-06-25 playtest: lightning clouds fire plain red dots; the ROM Cloud projectile (Cloud.asm loc_35DF0) renders a lightning-bolt sprite. Gameplay/lifetime is correct (#198); only the sprite is wrong. | Add `ActiveProjectileKind::CloudLightning` (types.rs), map it in trace.rs, draw the Cloud bolt art in scene.rs — same 3-file pattern as Tank/Tar/Lion bullets. capture-frames diff on a lightning-cloud level. |
| 359 | [ ] | P2 | enemy/level | 2026-06-25 playtest: on the Cyclone-form sky level the bouncing ball (Sphere 0x06) usually at the spawn point is missing. Untracked before now. | 2026-06-26: ball APPEARS then drifts off the LEFT edge (not absent) — favour Sphere X-velocity vs camera-scroll/activation-window over the roster miss. Confirm exact level + entry path (#334). See 2026-06-26 + 2026-06-25 triage. |
| 349 | [ ] | P1 | render | 2026-06-26 playtest: post-death respawn card still shows the trees but NOT the level-NAME text; the 2026-06-21 fix (`b55244ad1`) IS in the build, so the candidate FAILED validation. | Re-investigate the respawn-card render path (InGame classification trace.rs:9177; glyph-worker filter; overlay window); add a native-player capture-frame assertion, not just the CRC-neutral render test. See archive #349. |
| 361 | [ ] | P2 | render/enemy | 2026-06-26 playtest: on the hand level the live Hand (0x17) animation free-runs/flickers ("spazzes") instead of holding body frames. Deferred #243/#333 follow-up, never given its own row. NOT the grab gating #185 (grab+slow works). | Add a `select_live_hand_frame` selector mirroring `select_live_tar_monster_frame`: hold Hand body frames off HandState, early-return None on hp==0 so the death table keeps death frames; capture-frames to confirm no flicker. Cross-ref #243/#333, #185. |
| 362 | [ ] | P2 | audio | 2026-06-26 playtest: RockTank drive SFX (`$1F` TankDriving) STARTS too early — audible from far off-screen before the tank is reached/visible. START side, distinct from #339 (STOP/persistence side). | The pooled drive refcount fires on first Tank activation, gated only by the off-screen enemy-activation window; compare native activation distance + `$1F` start-gate vs ROM `Tank.asm` Init on-screen gate. Needs #360. See archive #362. |
| 363 | [ ] | P2 | audio | 2026-06-26 playtest: Kid sometimes says "ouch" instead of "die" on a full death; human confirms it is a bug, not the deliberately-kept ROM exception. | Capture exact death cause, helmet, hit-count, audio transcript. Ouch is suppressed only on a clean fatal CONTACT and kept on helmet-loss/survivable-then-die; the bug is ouch on a clean single fatal contact, no helmet. See archive #363 (#308). |
| 364 | [ ] | P2 | infra/asm-coverage | Tokenizer-grade `#[cfg(test)]` detector for asm-coverage (fixes audit A5). A brace-counter mis-classifies (char-literal `'{'`, early gated helpers swallow production lines). Foundation for the honest reachable/test-only split. | Unit tests incl. the blocks.rs trap: 1692/4675 = production, 5700 = test. See audits/asm-coverage-remediation-plan.md #364. |
| 365 | [ ] | P2 | infra/asm-coverage | Reclassify coverage into reachable / test-only-debt / uncovered, plus a GEMS "emulated" bucket; surface in headline/`--json`/YAML. Also remove the dead `oracle_backed`/`OracleBacked` tier (0 populated, vestigial). Depends on #364. | Headline reads roughly 3788 reachable / 289 test-only / 40 uncovered (33 GEMS); no OracleBacked state. See plan #365. |
| 366 | [ ] | P2 | infra/asm-coverage | Triage the 5 orphan subroutines (sub_96E8 et al.) with `// TODO(fidelity)`/`// DEAD_ROM`; snapshot the 289 test-only labels as a thin ratchet baseline; add `asm-coverage --check-no-new-test-only`. Depends on #365. | A new untagged test-only port fails `--check-no-new-test-only`. See plan #366. |
| 367 | [ ] | P2 | infra/asm-coverage | Wire `verify-slice-fast` deps: #366 ratchet + `audit-debug-fields` (green) + `asm-coverage --check-required` (with real integration test); fold in E3 glob fix + registry hygiene (dead-entry warn + pending-hash backfill). Depends on #366. | verify-slice-fast goes red on a deleted required citation, a new test-only port, or an unregistered debug key. See plan #367. |
| 368 | [ ] | P3 | infra/docs | One-off tidy (not gated): fix stale citations loc_2510C (combat.rs:179) + loc_23629 (native_player/mod.rs:2086); gate-message → core/data (F6); refresh audio-triggers refs (F5); dedup CLAUDE.md asm-coverage block + document `--check-required` (F4). | Stale citations gone; CLAUDE.md block single-source. See plan #368. |
