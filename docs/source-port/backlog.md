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
- **#289 stays `[~]`; Lion projectiles now render the ROM `stru_3DF34`
  sprite instead of the generic red dot.** Re-reading `Lion.asm loc_3E12A`
  shows the projectile child sets palette line 2, VRAM tile `$2FB`, and
  animation `stru_3DF34` (C7E20/C7E28/C7E30, six ticks each). Native-player now
  compiles those three mappings into object type `$12` and renders
  `ActiveProjectileKind::LionProjectile` through the Lion palette/art. Fresh
  expected-red `issue289-level07-idle-current` remains state-clean/pixel-red but
  improves from frame 60 `1297` / MAE `1.4228051` and frame 71 `277` /
  `0.2984375` to frame 60 `1289` / `1.4096261` and frame 71 `269` /
  `0.28263578`. **NEXT #289:** inspect the remaining frame-60 Lion
  sprite/origin/palette/body nuance and tiny top/background residual, not Lion
  projectile dot rendering, gameplay state, Plane-B rows
  B08/B17/B18/B19/B22/B23/B27/B29/B30, global `Time_Frames`, static cloud
  H-scroll, or simple `loc_12110` current-phase rendering. **archive #289**.
- **Recent landed rows now await human playtest**: #343 friendly projectiles vs large
  enemies and #346 launched spike-bullet lifetime are both `[?]`; full evidence and
  verification are in archive #343/#346.
- **#308 follow-up surfaced (still `[b]`, do not auto-take)**: #343 fixed only the
  projectile path; RedStealth sword (helmets.rs:6369) + berzerker charge (6532) still use
  the fixed `±4×±8` enemy box and per ROM `sub_1D84` should also source
  `enemy_contact_bounds` — same vein, separate CRC risk. Captured under archive #343.
- **#289 still should not be ground by guesswork**: the full do-not-retry list is in
  archive #289. Avoid header/start layout, simple scroll offsets, placement wrap,
  raw `$00/$08`, y=100 H-scroll, row-window count, render `Time_Frames`, stateless
  `$FFFF87B2`, broad raw `loc_30452` display-scroll substitution without the
  row-history/underlay fix, current-time all-row overlay, packed timed-row
  full-upload placement, alias-only rendering, alias-fed full row uploads,
  direct row-number-to-bgscroll preload mapping, source-row 8 live writes for
  B08, static cloud H-scroll after the previous-VDP-table latch, simple
  `loc_12110` middle-shore current-phase rendering, and reworking
  B08/B17/B18/B19/B22/B23/B27/B29/B30 row cells without fresh oracle row evidence.
- **#336 stays `[ ]`, regression bisected (no code change — oracle-gated)**: the
  cold-boot "Kid floats then drops" is the `LEVEL_ONE_LOAD_INGAME_CAMERA_RAMP_Y`
  camera pan (216→115px) introduced by c65df2500 (2026-06-12, recent — corrects the
  archive's stale "not a recent regression" verdict). It is applied only for level 0
  (other levels use the settled camera). The decisive oracle comparison is still
  unmet: `compare-native-oracle --scenario boot-title-start-level1` only reaches the
  title card (frame 719), not the live float. NEXT: oracle capture PAST the title card
  into live gameplay, per-frame camera/kid-Y diff; do NOT edit the ramp first (may be
  oracle-derived — never guess). Full evidence in **archive #336**.
- **Gates this session**: `cargo fmt --all --check`, `cargo check -q -p
  wildside-cli`, focused `wildside-cli` Lion live/death frame tests,
  expected-red #289 recording, enemy-state `tas-diff` frames 55..62,
  `backlog-lint`, `git diff --check`, and `mise run verify-slice-fast` all
  passed.
- **Blocked on you (human repro/validation)**: the large `[?]` set in
  `backlog-awaiting-human.md` is waiting on a playtest; for audio rows (#342/#339/#264/#272)
  the recurring "needs a real audio device" block IS that listen.

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
| 280 | [ ] | P2 | render/background | 2026-06-09 playtest: parallax sky loads in late (level 3 top), stray dark-green… | Reconfirmed 2026-06-11; Plane-B wrapping candidate landed, broader live-route validation remains. See archive #280. |
| 289 | [~] | P1 | level/spawn | 2026-06-10 playtest: hell-theme levels start Kid in the wrong place and the map layout looks wrong | 2026-06-22: Lion projectile sprite fix; expected-red frame 60 `1289` / MAE `1.4096261`, frame 71 `269` / `0.28263578`, state green. See archive #289. |
| 323 | [ ] | P2 | render | Fixed candidate: Maniaxe axe traces as `ActiveProjectileKind::ManiaxeAxe`, renders/spins via ASM `stru_8B86`; any damage report needs separate collision repro. | Human validation: throw axe in Maniaxe/Iron Knight form and confirm spinning axe art. See archive #323. |
| 324 | [ ] | P1 | render/block | Fixed candidate: live/wobbling prize blocks draw Block_Mappings art and suppress foreground tree/wood bleed-through per ASM `sub_302F6`/`sub_1007A`. | Human validation: hit a prize block over forest tree decoration; no tree/wood pixels should show through. See archive #324. |
| 336 | [ ] | P1 | level/entry | 2026-06-20 human-confirmed cold-boot regression: level 1 shows Kid floating in sky then snapping down. Bisected to camera ramp change. | NEXT: oracle capture past title card into live gameplay, diff camera/kid-Y; do not edit ramp without oracle. See archive #336. |
| 344 | [ ] | P2 | enemy | 2026-06-20 playtest: walking-dragon stomp reads as die-then-revive; likely ASM-faithful survivor hurt/hibernate, needs visual/UX check. | Human distinguish survive-flash vs death+resurrect; verify hurt anim during hibernate. See archive #344. |
| 345 | [ ] | P1 | enemy | 2026-06-20 playtest: Lion vanishes mid-level on Isle/Land of the Lion Lord (0x07), likely premature off-window cull vs ROM `sub_3E8BA`. | Oracle scroll-off/scroll-back probe on 0x07; check respawn marker `loc_3E97C`. See archive #345. |
| 347 | [ ] | P2 | render | 2026-06-20 playtest: shifting/push-up block snaps to new cell instead of showing ASM `loc_113F4` 8-frame/16px rise; renderer drops non-prize bounce sprite. | Draw kind=Shifting bounce sprite (E0EBE art) over 8 rise frames; flip enforcing test. See archive #347. |
| 348 | [ ] | P1 | input | 2026-06-20 playtest: level/title card ignores early Start; ASM `loc_1AE2E` exits on first Start edge from frame 2, native gates it. | Exit on first live Start edge; gate replay delay for boot traces. DynamicScenario card-frame ~5 should reach Game_Mode $C in 1-2 frames. See archive #348. |
| 351 | [ ] | P2 | render | 2026-06-20 playtest: Whispering Woods Drip enemies (0x16) render wrong palette variant; native hardcodes one Drip palette. | Add per-level slot/palette-line variant selection; compare level $12 x~2264 vs oracle. See archive #351. |
