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
- **No ready/open agent rows remain.** Proactive fallback: Archer, Big Hopping Skull, and Scorpion mutation shards are clean on current HEAD. Latest Scorpion pass pins `loc_3DD7E`, `loc_3DDA8`, `loc_3DDC8`, `loc_3DDEE`, and `stru_3DB8E` edges.
- **#280 moved to `[b]` route-needed.** Fresh BLW2/Hills/Bloody Swamp `$3E` probes go
  state-red; old `frame_index` diagnosis is stale. Do not patch without a clean route.
- **#317 moved to `[b]`:** Island shore, storm, lava geyser, and weather-4 Unknown concrete slices are fixed/covered; the only remaining weather debt is a shared typed VRAM backend, which is a dedicated renderer-cache architecture session rather than a narrow unattended bug slice.
- **#318 moved to `[x]`:** movement-fidelity audit now has no standalone open rows. `sub_DB22` is live-wired on airborne-to-grounded transitions; stale platform/climb rows were already covered, and `$14/$18` remains resolved under #224 as stuck-in-solid recovery rather than natural-play movement debt.
- **#316 `[x]`, #323/#347/#351 `[?]`:** DP raw slot storage is resolved; Maniaxe axe, Shifting block, and Drip child palette render paths are fixed on current HEAD and await latest-build human validation.
- **2026-06-22 HUMAN PLAYTEST TRIAGE (big batch).** 21 observations triaged against the
  lanes; full per-issue evidence is in the archive block "## 2026-06-22 playtest triage". NET:
  - **8 fix-candidates FAILED human validation.** #339/#335 are back to `[?]`, #257 is `[x]`,
    #259/#271/#272 are `[b]`; do not 6th-cycle #259.
  - **2 human-CONFIRMED WORKING → promoted `[?]`→`[x]`:** #343 Maniaxe axe now kills dragons,
    #346 spike→spike-wall chain. (Rows moved to the archive resolved index; final close is yours.)
  - **3 NEW rows opened:** #356/#353 moved to `[b]`; #355 moved to `[?]`.
  - **3 open rows refreshed:** #336, #324, #280 (blue/green seam-bar + late/half clouds).
    #280 and #345 have since moved to `[b]` route-blocked after fresh probes failed to
    isolate their reported live symptoms.
- **#289 moved to `[?]` fixed-candidate human validation.** `issue289-level07-idle-current`
  is trace/state/frame exact at frames 60 and 71; USM3 0x06 and O14 Lion projectiles need the
  same live retest. Full do-not-retry/final evidence is in archive #289.
- **NOT-A-BUG:** O7 USM "stacked hovering enemy" = EmoRock (0x1B) hover tracking Kid's X (ASM
  loc_3621A) — faithful; multiple rocks converge above the Kid by design. Only the #333 render
  approximation (unextracted hover frames) stays open.
- **Render-gap lesson:** headless CRC/state gates pass while a render-only gap ships (#352, #349);
  every missing-sprite/animation fix needs a native-player RENDER assertion or capture-frames diff.
- **#336 closed `[-]` NOT-A-BUG / ORIGINAL_QUIRK.** Cold-boot Start on the BLW1 card is
  CRC/checkpoint clean and oracle WRAM matches the camera/Kid-Y ramp; early-Start feel is #348.
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
- **#272 moved to `[b]` route/real-device blocked.** Native emits Cyclone `$32` on a
  level 0x14 pickup, but the route is oracle-red; 0x1B/0x25 searches found no bounce.
- **#210 moved to `[?]` fixed candidate.** Direct IntroSequence5 frame-2100 Start now has exact fade/title samples; human title/menu skip validation remains required.
- **#254 moved to `[?]` fixed candidate.** IntroSequence6 Start-skip no longer shows the blue hidden shell; original sample route AE is 22/0/0/0/0/0/8/0.
- **#256 moved to `[b]` route-needed.** Current Highwater route and best B+Right overlay still stop 75px left of the waterfall mask; priority-mask tests are green.
- **#324 moved to `[?]` fixed candidate.** Prize wobble now reuses copied block art when generated `$229` slots are neighbouring art; BLW1 native probes are clean, exact human route validation remains required.

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
