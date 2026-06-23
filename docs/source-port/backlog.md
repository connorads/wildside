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

- **Select work with `mise run project-status`** (priority-ordered across the live
  lanes; file order is not priority). Take its top "ready"/open item — do not eyeball.
- **2026-06-22 HUMAN PLAYTEST TRIAGE (big batch).** 21 observations triaged against the
  lanes; full per-issue evidence is in the archive block "## 2026-06-22 playtest triage". NET:
  - **8 fix-candidates FAILED human validation and still need work:** #271 stomp-on-RockTank
    hurts (P0, now route-needed), #257 continue-screen wall-from-both-sides (P0), #252 tank
    sprite doubled + green bullets (P1), #259 RockTank floor-sink (P1, 5th reconfirm — close
    `[-]` or prove it on a natural L2 route, do not 6th-cycle `[?]`), #352 hidden jump-up block
    no anim (P1, RENDERER-only gap: scene.rs:5057 skips the HiddenReveal cell), #272 Cyclone
    transform voice (P1), #339 USM "washing" drone (P1), #265 EOL tally tick missing + rumble (P2).
    #335 Hills instant-death has a new fixed candidate and waits in `backlog-awaiting-human.md`.
  - **2 human-CONFIRMED WORKING → promoted `[?]`→`[x]`:** #343 Maniaxe axe now kills dragons,
    #346 spike→spike-wall chain. (Rows moved to the archive resolved index; final close is yours.)
  - **3 NEW rows:** #353 invisible metal-block spikes (P1 — kills you with no visible hazard),
    #355 Cyclone spin form no anim (P2 render), #356 Skydragon Castle bg piece dropout (P2).
  - **4 open rows refreshed:** #336, #324, #280 (blue/green seam-bar + late/half clouds), #345
    (Lion now ABSENT after death+respawn on 0x07).
- **O14 "Lion shoots red squares" stays under #289.** The current Lion-projectile local-palette
  and animation-clock fix removes the projectile residual in the refreshed recording. VERIFY next
  playtest; do NOT open a separate row.
- **#289 stays `[~]`.** USM3 0x06 human-confirmed "~back to normal" (working-confirm of the landed
  platform/blank-top-row/live-block/Dragon fixes). Latest pass fixed the Level 0x07 Island lower-shore
  `loc_12188` recording-settle phase; refreshed expected-red recording is trace/state-green with frame
  60 `45` / MAE `0.04169922` and frame 71 `83` / `0.12003813`. Remaining residual = top-right
  HUD/diamond strip plus frame-71 timer/top specks. Full do-not-retry list in archive #289.
- **NOT-A-BUG:** O7 USM "stacked hovering enemy" = EmoRock (0x1B) hover tracking Kid's X (ASM
  loc_3621A) — faithful; multiple rocks converge above the Kid by design. Only the #333 render
  approximation (unextracted hover frames) stays open.
- **Render-gap lesson:** headless CRC/state gates pass while a render-only gap ships (#352, #349);
  every missing-sprite/animation fix needs a native-player RENDER assertion or capture-frames diff.
- **#336 stays `[ ]`, regression bisected (no code change — oracle-gated)**: the
  cold-boot "Kid floats then drops" is the `LEVEL_ONE_LOAD_INGAME_CAMERA_RAMP_Y`
  camera pan (216→115px) introduced by c65df2500 (2026-06-12, recent — corrects the
  archive's stale "not a recent regression" verdict). It is applied only for level 0
  (other levels use the settled camera). The decisive oracle comparison is still
  unmet: `compare-native-oracle --scenario boot-title-start-level1` only reaches the
  title card (frame 719), not the live float. NEXT: oracle capture PAST the title card
  into live gameplay, per-frame camera/kid-Y diff; do NOT edit the ramp first (may be
  oracle-derived — never guess). Full evidence in **archive #336**.
- **#271 moved back to `[b]` route-needed after fresh Red Stealth audit.** BLW2 natural
  and two clean run+jump overlays are oracle-clean; the HP-loss cases are grounded/side
  contact before a clean first-frame stomp, not a native-only missed `$2C`. Highwater
  natural walk never activates a tank. Do NOT retune `loc_1CE4`/Tank HP without an exact
  live route or frame capture that shows a native-only clean jump-stomp failure.
- **#335 moved to `[?]` fixed candidate.** Natural 0x07→0x08 title-card handoff now refreshes
  murder-wall runtime at `loc_1AE2E -> loc_6E2 -> Load_InGame` while preserving the carried
  helmet. Cache-backed Red Stealth regression, `verify-slice-fast`, and `verify-tas-full`
  are green; human needs to re-test natural Hills entry.
- **Open P0 #257 continue-screen is the highest-priority actionable item.** #271 now waits in
  `backlog-awaiting-human.md` for an exact route.
  The rest of the `[?]` set in `backlog-awaiting-human.md` still
  waits on a playtest; for audio rows (#342/#264 and the reopened #272/#339) the recurring
  "needs a real audio device" block IS that listen. The promoted #343/#346 await your final close.

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
| 316 | [~] | P2 | form | Umbrella: audits/helmets-fidelity-audit.md — fixed candidates landed for several helmet gaps; remaining rows still open after staleness check. | Per-gap rows in doc; continue from next open helmets row. |
| 317 | [ ] | P3 | weather | Umbrella: audits/weather-fidelity-audit.md — weather gaps still open after the staleness check | Per-gap rows in doc. (full history: archive #317) |
| 318 | [ ] | P3 | player | Umbrella: audits/movement-fidelity-audit.md — 8 of 25 still open after the staleness check | Per-gap rows in doc |
| 319 | [ ] | P3 | audio | Umbrella: audits/audio-triggers-fidelity-audit.md — remaining low/stub trigger gaps after Ankh/evanescent/ice-block/no-time and later enemy/BGM follow-ups; timeout `$51`/`$60` path is already live-tested | Per-gap rows in doc; tick off as wired |
| 320 | [ ] | P3 | camera | Umbrella: audits/camera-fidelity-audit.md — 6 gaps, none high | Per-gap rows in doc |
| 210 | [ ] | P0 | gameflow/render | 2026-05-30 playtest: intro/title/menu path still visually corrupt and can re-enter demo loop over Start/menu | Evidence source: human playtest after pushed commits. Symptoms: Wild Side kids… |
| 254 | [ ] | P1 | intro/render | 2026-06-07 playtest: intro ninja animation has black outline artefacts | Evidence source: human playtest. Symptom: ninja animation in the intro has… |
| 256 | [ ] | P1 | render/enemy | 2026-06-07 playtest: third-level Rhino/Berzerker renders in front of foreground tiles | Evidence source: human playtest. (full history: archive #256) |
| 280 | [ ] | P2 | render/background | 2026-06-22 playtest reconfirm: background seam-bar near sky colour appears/disappears — BLUE on BLW2 0x01, GREEN on Hills 0x08 + 0x00/0x01 — clouds load in late and render half/quarter (Isle 0x07, island=Chunked bg). | Chunked-arm hard seam-clip, no Plane-B wrap (scene.rs:1823/1879) + invented frame_index auto-scroll (scene.rs:3404 vs camera-driven loc_3050A); broaden wrap to Chunked arm, add 0x07/0x3E coverage, do NOT retune CRAM colour. See archive #280. |
| 289 | [~] | P1 | level/spawn | 2026-06-10 playtest: hell-theme levels start Kid in the wrong place and the map layout looks wrong | 2026-06-23: Island lower-shore `loc_12188` settle fixed; expected-red trace/state green, frame 60 `45` / MAE `0.04169922`, frame 71 `83` / `0.12003813`; residual is HUD/timer only. See archive #289. |
| 323 | [ ] | P2 | render | Fixed candidate: Maniaxe axe traces as `ActiveProjectileKind::ManiaxeAxe`, renders/spins via ASM `stru_8B86`; any damage report needs separate collision repro. | Human validation: throw axe in Maniaxe/Iron Knight form and confirm spinning axe art. See archive #323. |
| 324 | [ ] | P1 | render/block | 2026-06-22 playtest reconfirm: BLW1 0x00 prize blocks STILL show tree/wood foreground through the wobble sprite's transparent slots — the 2026-06-18 terrain-suppression candidate (scene.rs:3795-3810) FAILED on the canonical forest route. | Oracle frame-diff a BLW1 0x00 prize block over tree decoration during the hit/wobble; no tree/wood pixels through it. Do NOT blind-revert the suppression `continue` guards — get an oracle forest frame first. See archive #324. |
| 336 | [ ] | P1 | level/entry | 2026-06-22 playtest RECONFIRM (persistent): BLW1 0x00 cold boot starts Kid high in/above the trees then floats down — level-0-only camera ramp (216→115px) outpaces the Kid world-Y settle (255→271px). Bisected to c65df2500. | NEXT: oracle capture PAST the title card (frame 719 / Game_Mode $C), per-frame camera/kid-Y diff; do NOT edit the ramp first (may be oracle-derived). See archive #336. |
| 344 | [ ] | P2 | enemy | 2026-06-20 playtest: walking-dragon stomp reads as die-then-revive; likely ASM-faithful survivor hurt/hibernate, needs visual/UX check. | Human distinguish survive-flash vs death+resurrect; verify hurt anim during hibernate. See archive #344. |
| 345 | [ ] | P1 | enemy | 2026-06-22 playtest: Lion now ABSENT after death+respawn on Isle of the Lion Lord 0x07 (extends the mid-level vanish) — same `large_pacer` off-window cull (roster len vs ROM Number_Objects) re-firing during the post-respawn camera resettle. | Probe 0x07 post-death respawn: confirm Lion respawns after resettle (enemy_status_words.fill(0)=SpawnFresh, not a stale $2168 marker); rule out #239 ledge-fall. See archive #345. |
| 347 | [ ] | P2 | render | 2026-06-20 playtest: shifting/push-up block snaps to new cell instead of showing ASM `loc_113F4` 8-frame/16px rise; renderer drops non-prize bounce sprite. | Draw kind=Shifting bounce sprite (E0EBE art) over 8 rise frames; flip enforcing test. See archive #347. |
| 348 | [ ] | P1 | input | 2026-06-20 playtest: level/title card ignores early Start; ASM `loc_1AE2E` exits on first Start edge from frame 2, native gates it. | Exit on first live Start edge; gate replay delay for boot traces. DynamicScenario card-frame ~5 should reach Game_Mode $C in 1-2 frames. See archive #348. |
| 351 | [ ] | P2 | render | 2026-06-20 playtest: Whispering Woods Drip enemies (0x16) render wrong palette variant; native hardcodes one Drip palette. | Add per-level slot/palette-line variant selection; compare level $12 x~2264 vs oracle. See archive #351. |
| 257 | [ ] | P0 | gameflow/death | 2026-06-22 playtest (fix failed validation): continue (CONTINUES) screen renders wrong — a wall-like graphic slides in from BOTH left and right instead of legible CONTINUES text + count. | Capture a live death→continue card frame; CONTINUES label + count must render as legible text/digits sliding to centre, not blocky wall art from either edge. Slide direction (loc_D31E/loc_D350) is faithful — don't remove it. See archive #257. |
| 252 | [ ] | P1 | render/enemy | 2026-06-22 playtest reconfirm (P2→P1): BLW2 0x01 / Highwater 0x02 RockTank (0x0A/0x0B) renders doubled/humanoid + low in floor; the shooting tank's bullets (TankBullet child) render as GREEN blobs. Render-only (enemy state oracle-faithful). | Walk BLW2/Highwater past a shooting RockTank: one solid grey tank (not doubled/humanoid, not half in floor) + bullets show ROM stru_3B526 art, not green rects (scene.rs:5923-5957; forest-borrow palette). Capture native vs oracle. See archive #252. |
| 259 | [ ] | P1 | enemy/render | 2026-06-22 reconfirm (5th human): specific RockTanks render half-sunk in floor — BLW2 0x01 (teleporter + flag tanks) and the first Highwater 0x02 tank; not all tanks. | ONE natural oracle BLW2 0x01 route at the named tanks + first Highwater tank; diff body/turret Y + occlusion vs oracle. Only-grass occludes → close [-]; else real residual. NOT direct-warp. See archive #259. |
| 352 | [ ] | P1 | render/block | 2026-06-22 reconfirm (fix failed): hidden jump-up blocks (USM 0x04-0x06 / Highwater 0x02) still pop in/out with no fold anim — core landed but the renderer skips the revealed cell. | Headbutt a hidden block (USM 0x04-0x06 / Highwater 0x02); the 8-frame y-bob (loc_E2E2/E2F0) must animate. Wire HiddenReveal into scene.rs:5057 block-bounce draw loop (mirror #347). See archive #352. |
| 272 | [ ] | P1 | audio | 2026-06-22 (transform-voice fix failed): picking up the Cyclone "spin form" helmet plays NO transform noise. Core mapping exists (id 0x32) but no Cyclone route was ever validated. Render half = #355. | Real-device/oracle: pick up a Cyclone helmet in normal play; transform SFX id 0x32 must be audible. Needs a state-aligned natural Cyclone-pickup route (none validated). See archive #272. |
| 339 | [ ] | P1 | audio | 2026-06-22 (fix failed): looping "washing"/rumble GEMS drone (Rock Tank drive $1F) still plays as soon as Under Skull Mountain 0x04 loads. j_StopMusic mirror only stops it at the OUTGOING teardown. | Real-audio-device: enter USM 0x04 — no looping drone on load. Confirm StopSequence($1F) HALTS not restarts (#341 audio-audit on the boundary); check skip/death vs clean load. See archive #339. |
| 265 | [ ] | P2 | audio/gameflow | 2026-06-22 playtest (fix failed validation, symptom inverted): EOL score-tally tick NOT heard + a "weird rumbling" at level end; 2026-06-15 StopMusic/suppress candidate likely over-suppressed the loc_D752 $0C tick. | Real-audio-device EOL run: counting tick (sfx_Socre_counter $0C, loc_D752) audible during the tally + stops at loc_D7F4; no rumble bleed. Cross-ref #339 drone / #264. Dummy SDL never repros. See archive #265. |
| 353 | [ ] | P1 | render/block | 2026-06-22 (NEW): USM spiked metal blocks render with NO screws/spikes — the spike art is missing so the kill hazard is INVISIBLE (player died on a visually-empty metal block). | capture-frames USM 0x04-0x06 at a spiked metal block vs oracle; native must draw the spikes. Collision still kills ($7000) while the foreground/Block_Mappings spike tile is absent. See archive #353. |
| 355 | [ ] | P2 | render/form | 2026-06-22 (NEW): the Cyclone "spin form" does not visibly SPIN in normal play. Cyclone art is extracted/wired but there is no continuous-spin path, so grounded Cyclone uses generic idle/walk. Audio half = #272. | Pick up a Cyclone helmet in normal play vs oracle: the spin/transform anim must play (CYCLONE_TRANSFORM_STEPS; flight spin sub_B270), then grounded Cyclone should animate. Distinct from #284 demo. See archive #355. |
| 356 | [ ] | P2 | render/background | 2026-06-22 (NEW): Skydragon Castle 0x1B sky-castle background pieces/sprites occasionally drop out in live play. Distinct from #280 (colour-seam) and #210 (demo/intro). | capture-frames --level 1B at scroll positions vs oracle; classify as Plane-A/B tile gaps vs Cloud/Tornado over-cull (#345/#289) vs faithful VDP 20-per-line sprite limit. See archive #356. |
