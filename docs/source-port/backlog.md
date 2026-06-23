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
- **#317 remains `[~]`:** Island shore DMA labels, storm `loc_14D8A` shared-art trace, lava geyser `loc_14772/loc_1478E` sound gates plus `$FFFE -> loc_1483A` disable/cooldown, and Unknown `loc_14E42` foreground-plane rendering are fixed; storm pattern/shared-art renderer, full lava script-walk/template-row output, and shared VRAM backend debt remain.
- **#316 remains `[~]`:** DP shared list + `sub_3F798` sprite compiler are
  gate-green; remaining helmets debt is exact raw shared DP payload storage.
- **2026-06-22 HUMAN PLAYTEST TRIAGE (big batch).** 21 observations triaged against the
  lanes; full per-issue evidence is in the archive block "## 2026-06-22 playtest triage". NET:
  - **8 fix-candidates FAILED human validation; #339 has since been re-fixed and moved
    back to `[?]`. #257 has since been fixed and moved to `[x]`. Remaining failed rows
    still open:** #271 stomp-on-RockTank hurts (P0, now route-needed), #259 RockTank floor-sink (P1, 5th reconfirm —
    close `[-]` or prove it on a natural L2 route, do not 6th-cycle `[?]`), #352 hidden
    jump-up block no anim (P1, RENDERER-only gap: scene.rs:5057 skips the HiddenReveal
    cell), #272 Cyclone transform voice (P1).
    #335 Hills instant-death has a new fixed candidate and waits in `backlog-awaiting-human.md`.
  - **2 human-CONFIRMED WORKING → promoted `[?]`→`[x]`:** #343 Maniaxe axe now kills dragons,
    #346 spike→spike-wall chain. (Rows moved to the archive resolved index; final close is yours.)
  - **3 NEW rows opened:** #353 and #356 remain open; #355 has since moved to `[?]`.
  - **3 open rows refreshed:** #336, #324, #280 (blue/green seam-bar + late/half clouds).
    #345 has since moved to `[b]` route-blocked after the settled probes did not reproduce
    the Lion absence.
- **#289 moved to `[?]` fixed-candidate human validation.** `issue289-level07-idle-current`
  is trace/state/frame exact at frames 60 and 71; USM3 0x06 and O14 Lion projectiles need the
  same live retest. Full do-not-retry/final evidence is in archive #289.
- **NOT-A-BUG:** O7 USM "stacked hovering enemy" = EmoRock (0x1B) hover tracking Kid's X (ASM
  loc_3621A) — faithful; multiple rocks converge above the Kid by design. Only the #333 render
  approximation (unextracted hover frames) stays open.
- **Render-gap lesson:** headless CRC/state gates pass while a render-only gap ships (#352, #349);
  every missing-sprite/animation fix needs a native-player RENDER assertion or capture-frames diff.
- **#336 closed `[-]` NOT-A-BUG / ORIGINAL_QUIRK.** The missing live oracle gate is
  now met: a cold-boot route that presses Start on the BLW1 level card is CRC/checkpoint
  clean through the visible handoff, and direct oracle WRAM watches match native's
  `LEVEL_ONE_LOAD_INGAME_CAMERA_RAMP_Y` / Kid-Y tables byte-for-byte. Do not edit the
  ramp without a new native-vs-oracle divergence. The separate "level card ignores early
  Start" feel issue remains #348.
- **#339 moved back to `[?]` fixed candidate.** Clean USM 0x04 has no RockTank objects;
  the live report's load-time "washing" drone fits stale incoming GEMS state, not a
  freshly seeded local tank. Native now mirrors ASM `Load_InGame loc_23629` `StopMusic`
  even when the BGM cue tracker is `None`: the clean 0x04 audio transcript changed from
  `Start(6)` to `StopMusicThenStart(6)`. Human real-device listen still required.
- **#252 moved back to `[?]`:** live `stru_3B526` bullet pixels reject green/fallback colours; body/turret tests still cover doubled RockTanks; human BLW2/Highwater check remains.
- **#345 moved to `[b]` route-blocked.** Direct Level 0x07 probes show the Lion present;
  after #348, Start reaches live mode and the target Lion is still observed. Keep the
  `large_pacer` audit smell, but do not patch without a clean live absence route.
- **#348 moved to `[?]` fixed candidate.** Live/direct title cards now exit on the first
  `loc_1AE2E` Start edge; ScriptRunner/TAS keeps the issue305 replay delay.
- **#265 moved to `[b]` real-device-blocked.** Current HEAD scripted and dummy-SDL
  captures both emit `$0C` start/stop and go silent after transition; needs real-device repro before code.
- **#271 moved back to `[b]` route-needed after fresh Red Stealth audit.** BLW2 natural
  and two clean run+jump overlays are oracle-clean; the HP-loss cases are grounded/side
  contact before a clean first-frame stomp, not a native-only missed `$2C`. Highwater
  natural walk never activates a tank. Do NOT retune `loc_1CE4`/Tank HP without an exact
  live route or frame capture that shows a native-only clean jump-stomp failure.
- **#335 moved to `[?]` fixed candidate.** Natural 0x07→0x08 title-card handoff now refreshes
  murder-wall runtime at `loc_1AE2E -> loc_6E2 -> Load_InGame` while preserving the carried
  helmet. Cache-backed Red Stealth regression, `verify-slice-fast`, and `verify-tas-full`
  are green; human needs to re-test natural Hills entry.
- **#344 moved to `[?]` fixed candidate.** Walking Dragon nonlethal stomp render now holds
  ASM `stru_3CCE6` hurt frame during hibernate.
- **#355 moved to `[?]` fixed candidate.** Cyclone flight now renders ASM `sub_B270 loc_B2A0`
  `cyclone_spinning_1..3`; grounded Cyclone stays on the normal walk table. Audio remains #272.

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
| 316 | [~] | P2 | form | Umbrella: audits/helmets-fidelity-audit.md — DP shared list and sprite compiler rows are fixed candidates; exact raw shared DP payload storage remains open. | Continue from remaining helmets row; see archive #316. |
| 317 | [~] | P3 | weather | Umbrella: audits/weather-fidelity-audit.md — weather gaps still open after the staleness check | Per-gap rows in doc. (full history: archive #317) |
| 318 | [ ] | P3 | player | Umbrella: audits/movement-fidelity-audit.md — 8 of 25 still open after the staleness check | Per-gap rows in doc |
| 319 | [ ] | P3 | audio | Umbrella: audits/audio-triggers-fidelity-audit.md — remaining low/stub trigger gaps after Ankh/evanescent/ice-block/no-time and later enemy/BGM follow-ups; timeout `$51`/`$60` path is already live-tested | Per-gap rows in doc; tick off as wired |
| 320 | [ ] | P3 | camera | Umbrella: audits/camera-fidelity-audit.md — 6 gaps, none high | Per-gap rows in doc |
| 210 | [ ] | P0 | gameflow/render | 2026-05-30 playtest: intro/title/menu path still visually corrupt and can re-enter demo loop over Start/menu | Evidence source: human playtest after pushed commits. Symptoms: Wild Side kids… |
| 254 | [ ] | P1 | intro/render | 2026-06-07 playtest: intro ninja animation has black outline artefacts | Evidence source: human playtest. Symptom: ninja animation in the intro has… |
| 256 | [ ] | P1 | render/enemy | 2026-06-07 playtest: third-level Rhino/Berzerker renders in front of foreground tiles | Evidence source: human playtest. (full history: archive #256) |
| 280 | [ ] | P2 | render/background | 2026-06-22 playtest reconfirm: background seam-bar near sky colour appears/disappears — BLUE on BLW2 0x01, GREEN on Hills 0x08 + 0x00/0x01 — clouds load in late and render half/quarter (Isle 0x07, island=Chunked bg). | Chunked-arm hard seam-clip, no Plane-B wrap (scene.rs:1823/1879) + invented frame_index auto-scroll (scene.rs:3404 vs camera-driven loc_3050A); broaden wrap to Chunked arm, add 0x07/0x3E coverage, do NOT retune CRAM colour. See archive #280. |
| 323 | [ ] | P2 | render | Fixed candidate: Maniaxe axe traces as `ActiveProjectileKind::ManiaxeAxe`, renders/spins via ASM `stru_8B86`; any damage report needs separate collision repro. | Human validation: throw axe in Maniaxe/Iron Knight form and confirm spinning axe art. See archive #323. |
| 324 | [ ] | P1 | render/block | 2026-06-22 playtest reconfirm: BLW1 0x00 prize blocks STILL show tree/wood foreground through the wobble sprite's transparent slots — the 2026-06-18 terrain-suppression candidate (scene.rs:3795-3810) FAILED on the canonical forest route. | Oracle frame-diff a BLW1 0x00 prize block over tree decoration during the hit/wobble; no tree/wood pixels through it. Do NOT blind-revert the suppression `continue` guards — get an oracle forest frame first. See archive #324. |
| 347 | [ ] | P2 | render | 2026-06-20 playtest: shifting/push-up block snaps to new cell instead of showing ASM `loc_113F4` 8-frame/16px rise; renderer drops non-prize bounce sprite. | Draw kind=Shifting bounce sprite (E0EBE art) over 8 rise frames; flip enforcing test. See archive #347. |
| 351 | [ ] | P2 | render | 2026-06-20 playtest: Whispering Woods Drip enemies (0x16) render wrong palette variant; native hardcodes one Drip palette. | Add per-level slot/palette-line variant selection; compare level $12 x~2264 vs oracle. See archive #351. |
| 259 | [ ] | P1 | enemy/render | 2026-06-22 reconfirm (5th human): specific RockTanks render half-sunk in floor — BLW2 0x01 (teleporter + flag tanks) and the first Highwater 0x02 tank; not all tanks. | ONE natural oracle BLW2 0x01 route at the named tanks + first Highwater tank; diff body/turret Y + occlusion vs oracle. Only-grass occludes → close [-]; else real residual. NOT direct-warp. See archive #259. |
| 352 | [ ] | P1 | render/block | 2026-06-22 reconfirm (fix failed): hidden jump-up blocks (USM 0x04-0x06 / Highwater 0x02) still pop in/out with no fold anim — core landed but the renderer skips the revealed cell. | Headbutt a hidden block (USM 0x04-0x06 / Highwater 0x02); the 8-frame y-bob (loc_E2E2/E2F0) must animate. Wire HiddenReveal into scene.rs:5057 block-bounce draw loop (mirror #347). See archive #352. |
| 272 | [ ] | P1 | audio | 2026-06-22 (transform-voice fix failed): picking up the Cyclone "spin form" helmet plays NO transform noise. Core mapping exists (id 0x32) but no Cyclone route was ever validated. Render half = #355. | Real-device/oracle: pick up a Cyclone helmet in normal play; transform SFX id 0x32 must be audible. Needs a state-aligned natural Cyclone-pickup route (none validated). See archive #272. |
| 353 | [ ] | P1 | render/block | 2026-06-22 (NEW): USM spiked metal blocks render with NO screws/spikes — the spike art is missing so the kill hazard is INVISIBLE (player died on a visually-empty metal block). | capture-frames USM 0x04-0x06 at a spiked metal block vs oracle; native must draw the spikes. Collision still kills ($7000) while the foreground/Block_Mappings spike tile is absent. See archive #353. |
| 356 | [ ] | P2 | render/background | 2026-06-22 (NEW): Skydragon Castle 0x1B sky-castle background pieces/sprites occasionally drop out in live play. Distinct from #280 (colour-seam) and #210 (demo/intro). | capture-frames --level 1B at scroll positions vs oracle; classify as Plane-A/B tile gaps vs Cloud/Tornado over-cull (#345/#289) vs faithful VDP 20-per-line sprite limit. See archive #356. |
