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
- **#339 just landed (`[ ]`→`[?]`, REOPEN resolved, awaiting human audio listen)**:
  the Rock Tank drive drone ($1F) no longer persists after a Kid death or stacks across
  level-skips. Mirrored ROM `Load_InGame`→`j_StopMusic` (loc_23629, the only death/skip
  off-switch) with `stop_level_looping_drones_on_load_ingame_loc_23629`, called at the
  respawn reset and the level-skip teardown — stops the live tank-drive AND clock-prize
  drones (also covers #342's death/level-load scope boundary). CRC-neutral; tests
  `loc_23629_*`; `verify-slice-fast` (6251+9) and `verify-tas-full` green (CRC all 37759
  match, 212 authoritative = #290 baseline). Full evidence: **archive #339**.
- **#350 also landed this session (`[?]`)**: Hills (theme 3) below-canopy foliage was
  blue/purple. Hills uses the LAYERED background path (not the chunked
  `background_palette_for_scene` the original note blamed); its lower foliage plane (ASM
  `loc_1295E` base `$07C0` = CRAM line 0) indexed the background sky palette. Extended the
  Mountain two-line branch to theme 3. Oracle-verified: warp idle frame-39 below-canopy
  diffs 1334→0. **archive #350**.
- **#289 stays `[~]` but is research-blocked**: 5+ consecutive instrumentation/docs
  commits, marginal MAE gains (Level 0x07 frame 60 `5080`→`4031`) and still
  `oracle_frame_match=false`. The remaining work is modelling incremental persisted
  Plane-B row state — a substantial renderer task, not a quick patch. Prefer
  project-status "ready" items; the next concrete lead and the full do-not-retry list
  are in **archive #289** (compare native/oracle B08/B17/B22/B23/B27/B29 row histories
  after the persisted preload replay).
- **#336 stays `[ ]`, regression bisected (no code change — oracle-gated)**: the
  cold-boot "Kid floats then drops" is the `LEVEL_ONE_LOAD_INGAME_CAMERA_RAMP_Y`
  camera pan (216→115px) introduced by c65df2500 (2026-06-12, recent — corrects the
  archive's stale "not a recent regression" verdict). It is applied only for level 0
  (other levels use the settled camera). The decisive oracle comparison is still
  unmet: `compare-native-oracle --scenario boot-title-start-level1` only reaches the
  title card (frame 719), not the live float. NEXT: oracle capture PAST the title card
  into live gameplay, per-frame camera/kid-Y diff; do NOT edit the ramp first (may be
  oracle-derived — never guess). Full evidence in **archive #336**.
- **Gates this session**: `cargo fmt` clean, both crates `cargo check` warning-free,
  new `loc_23629_*` (#339) and `hills_*_loc_1295e` (#350) regressions green,
  `mise run verify-slice-fast` green (6251 core + 9 CLI + verify-tas-frontier +
  backlog-lint), `mise run verify-tas-full` CRC all 37759 frames match with 212
  authoritative = #290 baseline. Both fixes are CRC-neutral (#339 audio commands; #350
  CLI render layer).
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
| 316 | [~] | P2 | form | Umbrella: audits/helmets-fidelity-audit.md — Five-Way/Snake live slot path/fades, Tracking Rain subpixel target velocity, destructive-DP invulnerability, Juggernaut ready-byte, stale Circle/Wall kill-all claims, Eyeclops growing beam strip/latch and short-beam tables fixed candidates; remaining rows still open after the staleness check | Per-gap rows in doc; Five-Way/Snake/Rain/Circle/Wall/Juggernaut/Eyeclops runtime tests; continue from next open helmets row |
| 317 | [ ] | P3 | weather | Umbrella: audits/weather-fidelity-audit.md — weather gaps still open after the staleness check | Per-gap rows in doc. (full history: archive #317) |
| 318 | [ ] | P3 | player | Umbrella: audits/movement-fidelity-audit.md — 8 of 25 still open after the staleness check | Per-gap rows in doc |
| 319 | [ ] | P3 | audio | Umbrella: audits/audio-triggers-fidelity-audit.md — remaining low/stub trigger gaps after Ankh/evanescent/ice-block/no-time and later enemy/BGM follow-ups; timeout `$51`/`$60` path is already live-tested | Per-gap rows in doc; tick off as wired |
| 320 | [ ] | P3 | camera | Umbrella: audits/camera-fidelity-audit.md — 6 gaps, none high | Per-gap rows in doc |
| 210 | [ ] | P0 | gameflow/render | 2026-05-30 playtest: intro/title/menu path still visually corrupt and can re-enter demo loop over Start/menu | Evidence source: human playtest after pushed commits. Symptoms: Wild Side kids… |
| 254 | [ ] | P1 | intro/render | 2026-06-07 playtest: intro ninja animation has black outline artefacts | Evidence source: human playtest. Symptom: ninja animation in the intro has… |
| 256 | [ ] | P1 | render/enemy | 2026-06-07 playtest: third-level Rhino/Berzerker renders in front of foreground tiles | Evidence source: human playtest. (full history: archive #256) |
| 280 | [ ] | P2 | render/background | 2026-06-09 playtest: parallax sky loads in late (level 3 top), stray dark-green… | Reconfirmed 2026-06-11 play-native: death-wall/hell route still shows green background artefacts. 2026-06-19 fixed candidate broadens Plane-B row wrapping to nonzero moving bgscroll rows across themes, covering the #289-linked Highwater/Bloody Swamp dark-green backdrop seam class; broader live-route background/layout validation remains open. |
| 289 | [~] | P1 | level/spawn | 2026-06-10 playtest: hell-theme levels start Kid in the wrong place and the map layout looks wrong | Follow-up 2026-06-21: partial chunked Plane-B row-window overlay improves Level 0x07 recording, but upper-row stale VDP history remains pixel-red. (full history: archive #289) |
| 323 | [ ] | P2 | render | Fixed candidate updated 2026-06-18: Maniaxe/Iron Knight axe now traces as `ActiveProjectileKind::ManiaxeAxe`, renders `stru_8B86`/`maniaxe_axe_1..4.bin` from the Maniaxe form visual bank instead of the generic green friendly-projectile rect, and advances the live `loc_8A52`/`stru_8B86` animation clock so thrown axes spin instead of staying on frame 1. Engine colid_hurt remains covered by #296 Maniaxe axe tests, so any remaining "doesn't hurt enemies" report needs a separate collision repro with the correct sprite visible. | Human validation: throw the axe in Maniaxe/Iron Knight form and confirm axe art spins, not a green dot/static frame. (full history: archive #323) |
| 324 | [ ] | P1 | render/block | Fixed candidate refined 2026-06-18: static live `$E7xx`/block cells follow ASM `sub_302F6`/`sub_3034A` and draw only Block_Mappings art, and active temporary `$6Fxx` prize-wobble cells now suppress the immutable foreground tile-layout layer while ASM `sub_1007A`'s copied `unk_E0E4E`/`unk_E0E5E` wobble object owns the cell. This prevents tree/wood foreground pixels showing through transparent prize-block wobble art. | Human validation: on a forest level, hit a prize block over tree decoration; the wobbling block should not show tree/wood pixels through it. (full history: archive #324) |
| 336 | [ ] | P1 | level/entry | 2026-06-20 playtest CONFIRMED real regression (human): on cold boot the first level shows Kid floating in the sky then snapping down to the floor. Visually obvious and clearly wrong; reported as a relatively recent regression, NOT the intended Load_InGame entry-settle. | 2026-06-21: bisected to c65df2500 (2026-06-12) — `LEVEL_ONE_LOAD_INGAME_CAMERA_RAMP_Y` (camera 216→115px, level-0 only). NEXT: oracle capture PAST the title card into live gameplay, diff per-frame camera/kid-Y; don't edit the ramp without it (may be oracle-derived). See archive #336. |
| 343 | [ ] | P1 | combat | 2026-06-20 playtest: axe/friendly projectiles pass through the walking dragon (0x0D) and other large enemies. Unit damage (take_dragon_walking_damage enemies.rs:8970, route loc_3CEBE) is ASM-faithful + wired, but the LIVE hit test `process_friendly_projectile_hits` (helmets.rs:6371) uses an invented fixed `ENEMY_CONTACT_HALF_WIDTH=4`/`HALF_HEIGHT=8` box (constants.rs:379/384) for ALL families instead of the real `CollisionSize_Index`, so the large dragon body sits outside the box. Concrete instance of the #308 umbrella; the 2026-06-19 "axe can't hit flying dragon" denial only covered the brief pre-flight loc_346C4 window | Source the contact box from CollisionSize_Index per family/frame (engine-level per #308); live walking-dragon axe-hit regression (none exists today). See archive #343 |
| 344 | [ ] | P2 | enemy | 2026-06-20 playtest: walking-dragon stomp reads as die-then-revive. Likely ASM-FAITHFUL (Dragon.asm loc_3CDFC HP=enemy_level+1; loc_3CEBE plays hurt/hibernate then resumes patrol on a 2HP+ dragon), reproduced correctly in native — so the defect is VISUAL/UX: confirm native plays stru_3CCE6 hurt anim (not a death-looking frame) during the 24-frame hibernate. Seed #200/#043/#199 are closed Flying-Dragon PROJECTILE rows (keyword-deflection trap) | Get the human to distinguish survive-flash vs true death+resurrect; verify hurt anim during hibernate; compare-native-oracle dragon-stomp. See archive #344 |
| 345 | [ ] | P1 | enemy | 2026-06-20 playtest: Lion vanishes mid-level on Isle/Land of the Lion Lord (0x07) — premature off-window cull vs ROM sub_3E8BA. Leading hypothesis: `large_pacer_should_remain_active` (enemies.rs:15921) keys on active_enemy_count (roster len) not ROM Number_Objects (all live objects), shifting the $A/$14 cull band (#289 proved native object accounting diverges here). Not covered by #186 (behaviour fidelity, parked [?]) or #289 (spawn umbrella, symptom buried) | Oracle scroll-off/scroll-back probe on 0x07; rule out the #239 faithful ledge-fall first; check respawn marker loc_3E97C. Engine-level fix only. See archive #345 |
| 346 | [ ] | P1 | gameplay | 2026-06-20 playtest: launched spike bullet (ShooterObject) force-deleted after ~120 frames (~480px) so it never reaches a distant wall to trigger the EvanescentBlock_Trigger return-spike chain. Native models it as a generic EnemyProjectile lifetime:120 (types.rs:3210); the generic step (projectiles.rs:2742/2711) deletes it and the exemption list (2700-2741) omits ShooterBlock. ASM Process_ShooterObjects (loc_EDFA) has NO timer — dies only on a collidable cell (loc_F00A) or level edge, keeps simulating off-screen | Exempt EnemyProjectileKind::ShooterBlock from the lifetime decrement; regression: a bullet launched >120f from a wall still reaches it and re-fires. See archive #346 |
| 347 | [ ] | P2 | render | 2026-06-20 playtest: shifting/push-up ("springy") block snaps to its new cell instead of showing the loc_113F4 8-frame/16px rise. Core is correct (SHIFTING_BLOCK_RISE_Y_DELTAS_LOC_113F4, reserved $6F cell, BlockBounceAnim kind=Shifting) but the renderer drops it: push_block_grid_overrides (scene.rs:3885) skips the cell (reserved $6F → has_block=false, not Prize); the bounce-sprite draw loop (scene.rs:3934) handles Prize only. Test `non_prize_cleared_block_bounce_keeps_foreground_terrain_visible` currently ENFORCES the no-anim behaviour. Distinct from #281 (trigger/SFX) | Draw the kind=Shifting bounce sprite (E0EBE art) over the 8 rise frames; flip the enforcing test. See archive #347 |
| 348 | [ ] | P1 | input | 2026-06-20 playtest: level/title card ignores Start for ~343 frames (122 for the Level-1 lineage) and needs a press-then-release; original exits the same frame Start fires. ASM loc_1AE2E exits on the FIRST Start edge from frame 2, no minimum-display gate. Native gates Start behind handoff_ready (>342; movement.rs:101) + a press-release latch (boot.rs:7388); the movement.rs:96-101 comment admits the 342 is an oracle-boot-trace replay artefact. Untracked (seed #016/#162/#327 all orthogonal + correctly closed) | Exit on the first Start edge for live/interactive play; gate the recorded delay behind a replay flag so boot traces (#305/#161/#248) stay green; DynamicScenario Start at card frame ~5 reaches Game_Mode $C within 1-2 frames. See archive #348 |
| 349 | [ ] | P1 | render | 2026-06-20 playtest: post-death level card renders the scene/trees but NOT the level-NAME text (A then loads fine). ASM lose_life→Game_Mode 8→Load_TitleCard allocates the same Title_Letter glyph workers as a normal entry, so the name MUST appear. Core model + tests pass (level_tests.rs:3579/3615 assert workers EXIST in the trace) but never assert the RENDERED native-player scene draws the name — the defect is in that render/integration gap. Distinct from #326 (WRONG name on ≥0x10 entry cards) and #257/#248 (black-screen recovery) | Add a native-player render assertion (not just core worker-presence); capture a post-death card frame; fix glyph ASCII fallback / title_elapsed offset / title_card None on the respawn path. See archive #349 |
| 351 | [ ] | P2 | render | 2026-06-20 playtest: Whispering Woods (levels $11/$12, theme 9 Forest, weather 0) falling "snow" is actually Drip enemies (0x16) rendering the wrong colour variant (orange not green). ROM ships 3 Drip palettes (Drips_1 green / Drips_2 orange / Drips_3 purple; kid.asm:48317-48319); native ENEMY_FAMILIES hardcodes one palette per family. No storm system runs here (Init_SpecialEffects returns on weather 0) — "should be snow" is a human misread but "orange particles" is a real colour-variant bug. Seed #317/#230 are the wrong (storm) system | Per-level slot/palette-line variant selection in bundle.rs load_enemy_visuals; level $12 capture at x~2264 vs oracle. See archive #351 |
