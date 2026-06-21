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

- **Select work with `mise run project-status`** (priority-ordered across the live lanes;
  file order is not priority). Current focus is still #289 because it remains `[~]`.
- **#289 current evidence**: Level 0x07 Lion projectile launch, Kid collision,
  terrain snaps, and post-bounce timing now follow the ROM rows more closely. Native
  uses the projectile collision row (`Lion.asm loc_3E12A` -> `stru_3DF34` ->
  `CollisionSize_Index[667..669]`/`unk_31944`), keeps side/below damage alive
  (`loc_3E192` only deletes `colid_kidabove`/`colid_hurt`), moves before homing like
  the hibernating gfx object, runs `loc_3E260` bounces from the full `unk_31944`
  terrain box, and now preserves the `loc_3E2A8`/`loc_3E2C0` DBF bounce-loop state
  across frames so homing stays suspended after terrain hits. Fresh
  `issue289-level07-idle-current` still reports `trace_match=true`,
  `oracle_state_match=true`, `oracle_frame_match=false`. Native-player now also
  renders Kid's white damage flash for the sampled frames from `kid.asm loc_BC56` /
  `sub_80BC`, improving frame 60 from 7703 changed pixels / MAE 8.285068 to 7277 /
  7.258966 and frame 71 from 7928 / 8.547601 to 7502 / 7.521498.
- **Next #289 action**: inspect the remaining pixel-red Level 0x07 recording and
  broader hell-theme wrong-start/layout report. The concrete Lion projectile arc lead
  is now mostly exhausted: native reaches frame 60 `(427,648)` vs oracle `(427,649)`
  and frame 71 `(488,638)` vs oracle `(488,639)`, while strict frame diffs remain broad
  background/ocean/cloud colour/scroll mismatches. Do not retry simple chunked Plane-B
  row wrapping: a local experiment worsened the recording to frame 60 7752 / MAE
  7.8057895 and frame 71 7993 / 8.071968 by introducing top-row cloud errors. Also
  do not retry the stateless `$FFFF87B2` chunk-buffer plus 64-cell Plane-B wrap
  approximation; it worsened frame 60 to 7723 / MAE 7.803023 and frame 71 to 7929 /
  8.047498, again adding top-row cloud errors. Current candidate now applies the
  ROM `Time_Frames` basis to settled-warp `$00/$08` auto-scroll rows, improving
  frame 60 to 7070 / MAE 6.987421 and frame 71 to 7341 / 7.302334, but the
  recording remains state-clean/pixel-red. Continue from live `Horiz_Scroll_Data`
  phase/VDP row state or a fresh human-route repro before touching header/start layout
  code. Do not treat #345 Lion mid-level vanish as closed by this; that row still
  needs the scroll-off/scroll-back cull probe.
- **Gates this session**: `cargo fmt`, focused `cargo test -p wildside-cli background
  --no-default-features`, `cargo test -p wildside-cli time_frames_for_settled_warp
  --no-default-features`, fresh `verify-recording --slug issue289-level07-idle-current`
  (state-clean/pixel-red but improved), and `mise run verify-slice-fast` passed.
- **Blocked on you (human repro/validation)**: the large `[?]` set in
  `backlog-awaiting-human.md` is waiting on a playtest; for audio rows the recurring
  "needs a real audio device" block IS that playtest.

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
| 289 | [~] | P1 | level/spawn | 2026-06-10 playtest: hell-theme levels start Kid in the wrong place and the map layout looks wrong | Follow-up 2026-06-21: settled-warp `$00/$08` auto-scroll now uses ROM `Time_Frames`; recording still pixel-red. Inspect live `Horiz_Scroll_Data`/VDP row state next. (full history: archive #289) |
| 323 | [ ] | P2 | render | Fixed candidate updated 2026-06-18: Maniaxe/Iron Knight axe now traces as `ActiveProjectileKind::ManiaxeAxe`, renders `stru_8B86`/`maniaxe_axe_1..4.bin` from the Maniaxe form visual bank instead of the generic green friendly-projectile rect, and advances the live `loc_8A52`/`stru_8B86` animation clock so thrown axes spin instead of staying on frame 1. Engine colid_hurt remains covered by #296 Maniaxe axe tests, so any remaining "doesn't hurt enemies" report needs a separate collision repro with the correct sprite visible. | Human validation: throw the axe in Maniaxe/Iron Knight form and confirm axe art spins, not a green dot/static frame. (full history: archive #323) |
| 324 | [ ] | P1 | render/block | Fixed candidate refined 2026-06-18: static live `$E7xx`/block cells follow ASM `sub_302F6`/`sub_3034A` and draw only Block_Mappings art, and active temporary `$6Fxx` prize-wobble cells now suppress the immutable foreground tile-layout layer while ASM `sub_1007A`'s copied `unk_E0E4E`/`unk_E0E5E` wobble object owns the cell. This prevents tree/wood foreground pixels showing through transparent prize-block wobble art. | Human validation: on a forest level, hit a prize block over tree decoration; the wobbling block should not show tree/wood pixels through it. (full history: archive #324) |
| 336 | [ ] | P1 | level/entry | 2026-06-20 playtest CONFIRMED real regression (human): on cold boot the first level shows Kid floating in the sky then snapping down to the floor. Visually obvious and clearly wrong; reported as a relatively recent regression, NOT the intended Load_InGame entry-settle. | Bisect the recent regression; capture cold-boot frames 130..260 vs an oracle Blue Lake Woods 1 cold-boot recording. Suspect EARLY_START_LEVEL_ONE_BOOTSTRAP_* / LEVEL_ONE_LOAD_INGAME_PLAYER_RAMP_Y (trace.rs:6089/6163). See archive #336. |
| 339 | [ ] | P1 | audio | 2026-06-19 playtest: RockTank drive SFX ($1F, a LOOPING sequence) plays forever / "bugs out and keeps playing" — fired once on first-visible init (enemies.rs:10193) but never stopped when the shared refcount hits 0 on death/offscreen (ASM loc_3C46E decrement+stop unported). Mirror `should_play_stop_sound: next_refcount==0` (blocks.rs:642). Also the #264 "tank-drive repeats at EOL" symptom | Fixed 2026-06-19: `refresh_active_roster` emits `StopSequence(sfx_Tank_driving)` ($12 cmdstopseq stop) when the pooled drive refcount drops to 0 as the last Rock Tank exits (scroll-off loc_3C436 + death Tank.asm loc_3B480, both → loc_3C46E→ (full history: archive #339) |
| 342 | [ ] | P1 | audio | 2026-06-20 playtest: clock-prize SFX leaks/keeps playing. Native emits `start_sequence(sfx_Clock_prize $0E)` (combat.rs:10666, native_player/mod.rs:2756) but NEVER the loc_107E0 `$12` cmdstopseq stop, so the looping clock sequence runs forever. ASM loc_10792/loc_107E0 (kid.asm:21996/22028) gate start/stop on the `$FFFFFB55` refcount. The stop helper `prize_clock_sound_cleanup_loc_107E0` (blocks.rs:630) exists + is unit-tested but is dead code never wired into the live clock teardown — same trap as #339 | Wire stop on the FB55 refcount >0→0 edge; regression `loc_107e0_last_clock_prize_exit_emits_stopsequence_clock_prize`; sound advisory/CRC-neutral so verify-tas stays green. See archive #342 |
| 343 | [ ] | P1 | combat | 2026-06-20 playtest: axe/friendly projectiles pass through the walking dragon (0x0D) and other large enemies. Unit damage (take_dragon_walking_damage enemies.rs:8970, route loc_3CEBE) is ASM-faithful + wired, but the LIVE hit test `process_friendly_projectile_hits` (helmets.rs:6371) uses an invented fixed `ENEMY_CONTACT_HALF_WIDTH=4`/`HALF_HEIGHT=8` box (constants.rs:379/384) for ALL families instead of the real `CollisionSize_Index`, so the large dragon body sits outside the box. Concrete instance of the #308 umbrella; the 2026-06-19 "axe can't hit flying dragon" denial only covered the brief pre-flight loc_346C4 window | Source the contact box from CollisionSize_Index per family/frame (engine-level per #308); live walking-dragon axe-hit regression (none exists today). See archive #343 |
| 344 | [ ] | P2 | enemy | 2026-06-20 playtest: walking-dragon stomp reads as die-then-revive. Likely ASM-FAITHFUL (Dragon.asm loc_3CDFC HP=enemy_level+1; loc_3CEBE plays hurt/hibernate then resumes patrol on a 2HP+ dragon), reproduced correctly in native — so the defect is VISUAL/UX: confirm native plays stru_3CCE6 hurt anim (not a death-looking frame) during the 24-frame hibernate. Seed #200/#043/#199 are closed Flying-Dragon PROJECTILE rows (keyword-deflection trap) | Get the human to distinguish survive-flash vs true death+resurrect; verify hurt anim during hibernate; compare-native-oracle dragon-stomp. See archive #344 |
| 345 | [ ] | P1 | enemy | 2026-06-20 playtest: Lion vanishes mid-level on Isle/Land of the Lion Lord (0x07) — premature off-window cull vs ROM sub_3E8BA. Leading hypothesis: `large_pacer_should_remain_active` (enemies.rs:15921) keys on active_enemy_count (roster len) not ROM Number_Objects (all live objects), shifting the $A/$14 cull band (#289 proved native object accounting diverges here). Not covered by #186 (behaviour fidelity, parked [?]) or #289 (spawn umbrella, symptom buried) | Oracle scroll-off/scroll-back probe on 0x07; rule out the #239 faithful ledge-fall first; check respawn marker loc_3E97C. Engine-level fix only. See archive #345 |
| 346 | [ ] | P1 | gameplay | 2026-06-20 playtest: launched spike bullet (ShooterObject) force-deleted after ~120 frames (~480px) so it never reaches a distant wall to trigger the EvanescentBlock_Trigger return-spike chain. Native models it as a generic EnemyProjectile lifetime:120 (types.rs:3210); the generic step (projectiles.rs:2742/2711) deletes it and the exemption list (2700-2741) omits ShooterBlock. ASM Process_ShooterObjects (loc_EDFA) has NO timer — dies only on a collidable cell (loc_F00A) or level edge, keeps simulating off-screen | Exempt EnemyProjectileKind::ShooterBlock from the lifetime decrement; regression: a bullet launched >120f from a wall still reaches it and re-fires. See archive #346 |
| 347 | [ ] | P2 | render | 2026-06-20 playtest: shifting/push-up ("springy") block snaps to its new cell instead of showing the loc_113F4 8-frame/16px rise. Core is correct (SHIFTING_BLOCK_RISE_Y_DELTAS_LOC_113F4, reserved $6F cell, BlockBounceAnim kind=Shifting) but the renderer drops it: push_block_grid_overrides (scene.rs:3885) skips the cell (reserved $6F → has_block=false, not Prize); the bounce-sprite draw loop (scene.rs:3934) handles Prize only. Test `non_prize_cleared_block_bounce_keeps_foreground_terrain_visible` currently ENFORCES the no-anim behaviour. Distinct from #281 (trigger/SFX) | Draw the kind=Shifting bounce sprite (E0EBE art) over the 8 rise frames; flip the enforcing test. See archive #347 |
| 348 | [ ] | P1 | input | 2026-06-20 playtest: level/title card ignores Start for ~343 frames (122 for the Level-1 lineage) and needs a press-then-release; original exits the same frame Start fires. ASM loc_1AE2E exits on the FIRST Start edge from frame 2, no minimum-display gate. Native gates Start behind handoff_ready (>342; movement.rs:101) + a press-release latch (boot.rs:7388); the movement.rs:96-101 comment admits the 342 is an oracle-boot-trace replay artefact. Untracked (seed #016/#162/#327 all orthogonal + correctly closed) | Exit on the first Start edge for live/interactive play; gate the recorded delay behind a replay flag so boot traces (#305/#161/#248) stay green; DynamicScenario Start at card frame ~5 reaches Game_Mode $C within 1-2 frames. See archive #348 |
| 349 | [ ] | P1 | render | 2026-06-20 playtest: post-death level card renders the scene/trees but NOT the level-NAME text (A then loads fine). ASM lose_life→Game_Mode 8→Load_TitleCard allocates the same Title_Letter glyph workers as a normal entry, so the name MUST appear. Core model + tests pass (level_tests.rs:3579/3615 assert workers EXIST in the trace) but never assert the RENDERED native-player scene draws the name — the defect is in that render/integration gap. Distinct from #326 (WRONG name on ≥0x10 entry cards) and #257/#248 (black-screen recovery) | Add a native-player render assertion (not just core worker-presence); capture a post-death card frame; fix glyph ASCII fallback / title_elapsed offset / title_card None on the respawn path. See archive #349 |
| 350 | [ ] | P1 | render | 2026-06-20 playtest: Hills of the Warrior (theme 3, level 8) renders purple below the tree canopy. `background_palette_for_scene` (scene.rs:2334) builds the line-1 background remap only for theme 7 (Mountain, :2339) and 9 (Forest, :2361) — there is NO theme==3 (Hills) branch, so Hills line-1 ($2780) tiles index a wrong palette line. Data/core already know Hills is two-line (loc_1295E, Palette_Permutation_BGHill, bundle hill_bg.bin). Distinct from the dark-green CRAM-backdrop seam (#280/#289 = the sky) | Add a background_theme_id==3 branch mirroring theme 7/9 (assemble line-1 from Palette_Permutation_BGHill/_alt per loc_1295E); capture Hills L8 below-canopy native vs oracle. See archive #350 |
| 351 | [ ] | P2 | render | 2026-06-20 playtest: Whispering Woods (levels $11/$12, theme 9 Forest, weather 0) falling "snow" is actually Drip enemies (0x16) rendering the wrong colour variant (orange not green). ROM ships 3 Drip palettes (Drips_1 green / Drips_2 orange / Drips_3 purple; kid.asm:48317-48319); native ENEMY_FAMILIES hardcodes one palette per family. No storm system runs here (Init_SpecialEffects returns on weather 0) — "should be snow" is a human misread but "orange particles" is a real colour-variant bug. Seed #317/#230 are the wrong (storm) system | Per-level slot/palette-line variant selection in bundle.rs load_enemy_visuals; level $12 capture at x~2264 vs oracle. See archive #351 |
