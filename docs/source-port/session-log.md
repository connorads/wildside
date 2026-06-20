# Source-port session log

Append-only history of per-session handoffs, extracted from the `backlog.md`
NOW block (which is the *current-state* handoff and is rewritten each session).
Newest entries first. This is narrative history; per-issue evidence keyed by
`#NNN` lives in [`backlog-archive.md`](./backlog-archive.md).

---

## 2026-06-20 handoff (#289 Lion projectile collision/lifecycle narrowed)

- **#289 stays `[~]`, with Level 0x07 Lion projectile collision/lifecycle partially fixed.** Native now uses the projectile's ROM sprite collision row (`Lion.asm loc_3E12A` -> `stru_3DF34` -> `CollisionSize_Index[667..669]`/`unk_31944`) instead of the generic hostile-projectile centre window, so frame 28's old `dx=11 dy=-17` candidate is correctly `overlap=0`. Side/below damage no longer deletes the shot; `Lion.asm loc_3E192` only deletes on `colid_kidabove`/`colid_hurt`. Fresh `issue289-level07-idle-current` still reports `trace_match=true`, `oracle_state_match=true`, `oracle_frame_match=false`; damage latches at frame 29 and native keeps the projectile alive.
- **NEXT**: continue #289 by tracing the Lion projectile terrain-bounce/trajectory path after damage. The remaining concrete mismatch is visual/trajectory: at frame 60 native projectile is around `(325,754)`, while oracle raw object slot `0xffff0f70` is `(427,649)`; frame diffs remain `7285/7515` changed pixels. Do not close #345 from this; that is still the separate Lion scroll-off/scroll-back vanish probe. Verification this session: `cargo test -p wildside-core projectile --lib`; regenerated + verified `issue289-level07-idle-current` (state-clean/pixel-red); `mise run verify-slice-fast`. `mise run verify-tas-full` exited 0 and CRC-matched all frames, but still reports existing authoritative object-count divergences from around frame 7850.

---

## 2026-06-20 handoff (#289 stale X rejected)

- **#289 stays `[~]`, but the Level 0x07 X lead is rejected.** Generated fresh local recording `issue289-level07-idle-current` and ran `verify-recording --slug issue289-level07-idle-current`: report has `trace_match=true`, `oracle_state_match=true`, `oracle_frame_match=false`. The earlier `native=$02118000` / `oracle=$02058000` Lion X note came from a stale/missing route and is not current state-oracle evidence. Temporary attempts to pin that X in `level07_recording_settle_lion_survives_frame0_offledge_fall_loc_3e7ca` and to alter Lion settle/movement were reverted.
- **NEXT**: continue with `mise run project-status`; if it returns #289 again, inspect `.cache/recordings/issue289-level07-idle-current/verify/` pixel diffs or capture the original live hell-route symptom before touching header/start/layout code. #345 Lion vanish remains separate.

---

## 2026-06-20 handoff (#289 partial Lion survival)

- **#289 stays `[~]`, with one Level 0x07 Lion subcase fixed.** Native no longer routes the lower Lion's settled frame-0 off-ledge charge fall through `Lion.asm loc_3E94E` when the body terrain scan only overlaps the floor row at the feet. The replacement cache-backed regression now asserts the oracle-backed survival state (`y=$02C0_0000`, HP 6, Kid HP 2, no damage) instead of the stale projectile/body-contact timing claims. Fresh `verify-tas` still reports first divergence at frame 0, but now only on the remaining X-position/grounding/export details (`native x=$02118000`, `oracle x=$02058000`); broader #289 hell spawn/layout work remains open. Verification: focused Level 0x07 Lion regression, Level 0x06 Flying Dragon guard, Lion terrain/floor-edge tests, `verify-slice-fast`, and `verify-tas-full` (exit 0; CRC all 37759 frames match, current baseline state-report divergences still begin at `transition_15_to_00_post`).
- **NEXT**: continue with `mise run project-status`; if it returns #289 again, derive the Level 0x07 settle X-position miss from `Lion.asm` plus oracle before touching header/start/layout code. #345 Lion mid-level vanish is not covered by this fix and still needs the scroll-off/scroll-back cull probe.

---

## 2026-06-20 handoff (#272 fixed candidate)

- **#272 DAC voice pitch moved `[~]` -> `[?]`.** Dynamic native-audio verification now writes `audio-event-oracle-windows.json` beside each real jgenesis WAV capture, so the old native-as-oracle audio audit is no longer the only pitch evidence. The reports first exposed the low/slow native DAC path, then verified the fix: standalone GEMS now advances the YM2612 timer/status shadow at the YM input clock instead of raw Z80 cycles, and clears the hidden init-frame audio event epoch before visible playback. Evidence: `issue272-red-stealth-air-attack-direct` reports `$13` at 641 Hz native vs 638 Hz jgenesis; the level route covers `$60`, `$13`, `$46`, `$66` in the same pitch range. Focused GEMS/verifier tests pass; human audio-device validation is still required.
- **NEXT**: continue with `mise run project-status`; #272 is no longer in the open queue. Current audio work is now the looping-SFX leak family (#342/#339), then block render (#324), entry presentation (#336/#348/#254/#210), and combat/render lifecycle (#343/#345/#346/#350).

---

## 2026-06-20 handoff (#264 fixed candidate)

- **#264 live audio slowdown moved `[~]` -> `[?]`.** Live `play-native` now keeps generated GEMS/fallback frames in a small software backlog whenever SDL's two-frame device queue is full, then drains those frames in order before/after later video frames. This removes the old path where GEMS advanced and an already-mixed block/diamond/BGM frame was discarded at the cap. Hard BGM resets still clear SDL plus the backlog. Diagnostics now expose backlog pressure (`backlog_frames`, `backlog_full_frames`, `max_backlog_frames`) and `skipped_full_frames` stays 0 for compatibility. Verification: focused audio regressions, `cargo fmt`, `mise run backlog-lint`, `mise run verify-slice-fast`, and `mise run verify-tas-full` exits 0 with CRC matching all 37759 frames; current baseline state-report divergences still start at `transition_15_to_00_post`.
- **NEXT**: continue with `mise run project-status`; #264 is no longer in the open queue and awaits real audio-device validation. Current thrust remains audio (#272/#342/#339 family), then block render (#324), entry presentation (#336/#348/#254/#210), then combat/render lifecycle (#343/#345/#346/#350).

---

## 2026-06-20 handoff (#335 fixed candidate)

- **#335 Hills 0x08 instant death moved `[~]` -> `[?]`.** The headless generated-assets debug entry path reproduced the human N/A+C skip failure: first visible InGame frame was already `DeathWindup` with a pre-aged Hills murder wall, then life dropped shortly after. Native debug title-card entry suppresses the loc_1AE2E carry frames, so the handoff now collapses the hidden Load_InGame shell back to a fresh settled target spawn and clears the hidden death windup before exposing Mode_Level. Verification: cache-backed core regression `debug_level_entry_hills_resets_murder_wall_before_live_handoff_loc_74e0`, ignored native-player generated-assets skip regression, `cargo fmt`, `mise run verify-slice-fast` green, and `mise run verify-tas-full` exits 0 with the current baseline state-report divergences unchanged (CRC all 37759 frames match).
- **NEXT**: continue with `mise run project-status`; #335 is no longer in the open queue and awaits human playtest validation. Current thrust remains audio (#264/#342/#272), block render (#324), entry presentation (#336/#348/#254/#210), then combat/render lifecycle (#343/#345/#346/#350). #334 remains blocked on a Caves-of-Ice live leak repro.

## 2026-06-20 handoff (#352 fixed candidate)

- **#352 hidden-block reveal/fold animation moved `[~]` -> `[?]`.** Live hidden-block headbutt reveals now queue `sfx_Reveal_Hidden_block` (`$12`) and spawn a `BlockBounceKind::HiddenReveal` worker with the ASM loc_E2E2/loc_E2F0 8-frame y-bob, so revealed cells have the fold/bob instead of instant pop-in. Trace exposes `BlockBounceTraceKind::HiddenReveal` for the renderer path. Verification: focused hidden-block/trace/block-bounce tests, `cargo fmt`, and `mise run verify-slice-fast` pass. `mise run verify-tas-full` keeps published TAS CRC parity across all 37759 frames; the state report still shows the known #290 object-count wording caveat. Human validation: hit hidden blocks on USM 0x04-0x06 or Highwater 0x02 and confirm the reveal animation is visible.
- **NEXT**: continue with `mise run project-status`; #352 is no longer in the open queue. The current thrust remains audio (#264/#342/#272), block render (#324), entry presentation (#336/#348/#254/#210), then combat/render lifecycle (#343/#345/#346/#350).

## 2026-06-20 handoff (compressed from NOW)

- **2026-06-20 playtest triage — 28-symptom human playthrough cross-referenced vs backlog + ASM (29-agent workflow). Docs-only; the user's "issues keep getting missed or denied" complaint is CORRECT and falls into 4 anti-patterns.** (1) **DENIED-by-parking**: P1 audio rows sat at `[?]` across 5+ playtests, each pass tweaking `AUDIO_QUEUE_TARGET_FRAMES` and re-blocking on "needs a real audio device" — the recurring human repro IS that device, so `[?]` is not defensible. (2) **MISSED start-without-stop wiring**: loop-stop helpers (`loc_107E0` clock $0E, `loc_3C46E` tank-drive $1F) ported as unit-tested but `#[cfg(not(test))]` dead code never called at runtime. (3) **PARKED-with-gate-unmet**: #324's own verify gate ("human forest retest") was treated as satisfied; the fresh High Water Pass repro proves it still breaks. (4) **MISSED-by-keyword-deflection**: triagers grep "dragon"/"tank", land on closed projectile rows, declare it solved. Also: several "green" tests assert MODEL state or use a native-as-oracle audit, so they pass while the human-visible RENDER/PITCH defect goes undetected — durable fixes need native-player render assertions + a real jgenesis WAV comparison, not more model-level tests.
  - **REOPENED `[?]`/`[b]`→`[ ]`** (fresh human repro = the validation the row awaited, and it FAILED): #264, #272, #324, #280, #254, #210, #323; #339 (also P2→P1), #338 (also P2→P3); #336 + #335 flipped `[b]`→`[ ]` (the live repro the block waited for has arrived). **STATUS-CORRECTED** #271 `[x]`→`[?]` (closed on synthetic jugg/Dragon probes; its own note says "needs human re-test", and the live Level-2 RockTank stomp route was never captured).
  - **NEW rows #342–#352**: #342 clock-prize SFX leak (start-without-stop); #343 axe/friendly projectiles miss the walking dragon (live hit test uses an invented fixed 8×16 box, not `CollisionSize` — concrete #308 instance); #344 dragon stomp looks like die-then-revive (likely faithful, visual/UX); #345 Lion vanishes (premature off-window cull, `active_enemy_count` vs `Number_Objects`); #346 launched spike culled at 120f, never reaches the return-spike wall; #347 shifting/springy block has no rise animation (renderer drops it); #348 level card ignores Start ~343f vs `loc_1AE2E` first-edge; #349 post-death card missing level-name text; #350 Hills (theme 3) purple bg (no theme-3 palette branch); #351 Whispering Woods Drip wrong colour variant (orange not green); #352 hidden-block reveal/fold animation never spawned (split from the #309 [b] umbrella — #309 stays blocked on its type-8 slice).
  - **AUDIO splits into THREE distinct root causes — do NOT merge them**: (a) BGM stutter / "every other sound slows" = audio generated synchronously inside the video loop with a 2-frame queue cap (#264, native_player/mod.rs:801/39/844) — stop tweaking the cap, **decouple onto an audio thread / ring buffer**; triggers $12/$2D/$15/$56 are all ASM-correct so this is NOT a trigger gap. (b) DAC voices low/deep (#272) — the audio-audit "oracle" is the SAME native engine (`GemsAudioEngine::from_kid_rom`), so its delta-0 is blind to a systematic pitch error; a **real jgenesis WAV peak-freq comparison** is the only thing that resolves it (caveat: $6C/$6D are intentionally slowed in the ROM). (c) Looping-SFX leaks (#339, #342) — the dead-code stop helpers must be wired on death / level-skip / pickup-expiry.
  - **WORK ORDER (this batch)**: P0 first — **#335** (capture a live A+C skip into 0x08, dump frame-0..8 murder_wall_x vs player_x; suspect stale `MurderWall_X_pos`/fraction at weather.rs:3155 or low player_x during the entry-settle; group "other skip targets" with #266). Then AUDIO: **#264** (architecture) → **#339** + **#342** (stop-wiring) → **#272** (jgenesis audit). Then block render: **#324** (extend foreground-skin preservation to Metal/Shifting bounce cells — subsumes the metal-block background-erase symptom — and oracle-diff a forest AND High Water Pass prize hit before re-closing). Then entry-presentation: **#336** (cold-boot BLW1 frames 130..260 native vs oracle ramp diff), **#348** (level-card Start), **#254**/#210 (engine-level carry-last-VDP per `sub_1C246`, not probe-frame constants). Then combat lifecycle: **#343**, **#271** (live route), **#346**, **#345**, **#352**, **#350**. Probe-before-fix on every lifecycle/live-skip item per loop rules. Per-symptom evidence (ASM labels, file:line, why each status was/wasn't defensible) is in `backlog-archive.md` under each `#NNN` (grep `2026-06-20`).
- **Focus**: select with `mise run project-status` (priority-ordered; file order is not priority). **#274 is now status-corrected to fixed candidate**: current HEAD still rejects the cold-idle attract Diamond-prize hypothesis, with `verify-tas` CRC-clean through the first demo Cyclone prize/transform window; only an exact live SDL route/frame should reopen coding. **#289/#280 Plane-B dark-green seam has a fixed candidate**: nonzero moving `BackgroundScroll_ComputeShiftLayers loc_3050A` rows now wrap across themes, not just Hills `$08`; broader hell-route spawn/layout and live visual validation remain open. **#309 is blocked/exhausted**: actual generated Level_Layout grids still have 0 type-8 Shooter cells (type-9=562, type-C=864), and the only remaining block-audit deviations need a synthetic/oracle type-8 probe before changing the native `BlockType::Shooter` convention. **#310 is blocked/exhausted**: the terrain audit's concrete rows are fixed candidates, and the remaining trivia are representation-only, edge-only, or movement-fidelity/oracle-probe scope. **#312 is now status-corrected to fixed candidate**: the level-flow audit has no open rows left, only `[?]`/`[x]` fixed candidates with current regression coverage. **#313 is blocked/exhausted**: the only remaining shell-audit slice is visual pause VDP tile save/restore, but native-player is a stateless scene renderer with no persistent plane-A nametable to restore. **#315 enemy-fidelity audit is now status-corrected to fixed candidate**: every concrete row in `audits/enemy-fidelity-audit.md` has current fixed-candidate evidence/regressions; remaining caveats are representation-only or need fresh human/oracle routes. Continue via `mise run project-status`, not the enemy audit. **#272 is now a fixed candidate for source-port audio**: route `.cache/recordings/issue272-level01-redstealth-r0-25-b24-64/scenario.json` reaches Red Stealth transform `$2E`, and `verify-native-audio-output-file` passes; state verifier still splits around the floating item/object roster at the pickup, so human device validation remains required for the reported slow/low sound. **#338 is now a fixed candidate**: RockTank shot `$41` and creak `$3C/$42` all have native emit sites; the creak cadence follows Tank.asm `stru_3B27E`'s 80-frame linked-mouth loop and #341 audio-audit reports native-vs-ROM oracle duration delta 0 for `$3C`/`$42`. **#341 remains a fixed candidate** and is the standing tool for future SFX audits. **#249/#270 is blocked again**: the natural 0x04 held-Right/no-jump slope route is oracle-clean through the suspected loc_9FBC window, so the 2026-06-19 slope-into-wall claim is not an implementation route. **#334/#335/#336** still need human repro recordings before gameplay coding; #334's headless loader and `inspect-level` diagnostic now show the MapOrder-resolved Caves assets. Everything else from the 2026-06-19 + 2026-06-18 batches is `[?]`/`[-]`/`[b]` awaiting human validation or repro.
- **Current #316**: Five-Way Shot live slot path/negative-timer fade, Snake slot-chain negative fade/collision status/invulnerability stand-in, Tracking Rain target subpixel velocity, shared `Diamond_power_active` projection from live DP slots, `Diamond_power_ID`/`DiamondPower_Main_Index` dispatch, Juggernaut `FiveWayShotReady`, stale Circle/Wall kill-all audit claims, Eyeclops hard-beam render-time diamond spend, Eyeclops state-owned growing beam strip/direction latch/no invented speed-lifetime, Eyeclops `off_E560` short-beam table dispatch, Eyeclops hard-beam companion `loc_E948` object geometry, Maniaxe `loc_8A74` axe `x_vel` preservation, and the stale transform `+1` cleanup-frame audit claim are resolved/rejected based on current slot-vector/runtime/TAS tests. Continue #316 from the next open concrete row in `audits/helmets-fidelity-audit.md`; the broad shared DP-object linked-list row remains architectural debt.
- **NEXT (directed, 2026-06-19 triage — status after the 2026-06-19 implementation sprint)**:
  1. **#331 P1 [?] DONE** RockTank BLW2 self-kill — reverse-in-place loc_3C0D2 narrow-span wall turn. Awaiting human validation.
  2. **#276 P1 [?] DONE (already landed, e3c31ab3e)** — the reopen claim was STALE: the size-0 HUD pip fix was already on HEAD with green tests. No new work; awaiting human visual validation.
  3. **#249/#270 P1 [b] — DEAD END (do not retry the 4055 guard or the 0x04 held-right seed)**: the proposed wall_hit-gated guard at movement.rs:4055 is a NO-OP because `slope_sliding` suppresses the wall snap on slopes (movement.rs:6256, the BLW1 4082/6024 seam, #211), so `wall_hit` is always false on a slope-into-wall. The natural 0x04 held-Right/no-jump route with the periodic jump cleared (`150-153:none`, `150-220:right`) is CRC-clean against oracle frames 138..190, including the slope exit. Keep #249/#270 blocked until a sharper human route or natural BLW2 shaft route reproduces the launch.
  4. **#332 P1 [?] DONE (already landed, 7804966e7)** — same-form guard already at blocks.rs:4533. Stale reopen; awaiting human validation.
  5. **#243 + #333 P2 [?] DONE 2026-06-19** live-enemy render free-run — three `select_live_*` selectors hold Tar/Sphere/EmoRock on body frames (Tar/Sphere faithful; EmoRock a marked TODO(fidelity) approx). Hand 0x17 OUT of scope. Awaiting human capture-frames.
  6. **#338 P2 [?] DONE** RockTank SFX — SHOT `$41` and CREAK `$3C/$42` now ported. Awaiting human audio validation. **#339 [?] DONE** (drive-SFX stop).
  7. **#337 P2 [?] DONE 2026-06-19** Maniaxe stuck-axe — airborne loc_9454 latch tick (single-dispatch else-branch, no double-tick). Awaiting human playtest (no Maniaxe oracle).
  8. **#340 P3 [-] CLOSED** — `sfx_Voice_diamond` ($62) is an orphan ROM asset (zero trigger sites); native faithfully plays only $2D. NOT-A-BUG.
  9. **#334 P1 [b]** Caves-of-Ice level-skip mismap — `inspect-level` now reports MapOrder-resolved resource paths and Diamond/Hand data; still **needs human repro** to pin any live leak.
  10. **#341 P2 [?] DONE** sound-audit tool — native-vs-ROM-backed oracle duration/RMS/peak/frequency deltas now populate; use it for #338 creak cadence.
  11. **BLOCKED on human repro — do NOT auto-queue:** **#335 [b] P0** 0x08 instant-death-on-skip, **#336 [b] P1** first-level float. Need a `play-native` recording first.

- **Last session** (2026-06-19): **workflow-driven fix sprint — landed 4 fixes, 1 dead end, 1 not-a-bug close, corrected 2 stale reopens.** Ran a 7-item parallel prepare+adversarial-verify workflow over the directed queue, then drove the serial green-gated landing. (1) **#338 [~]** SHOT SFX `sfx_Tank_shoots` ($41) ported on the loc_3B730 spawn edge (`6901c1be5`); creak ($3C/$42) deferred (gun-anim phase counter). (2) **#243+#333 [?]** live-enemy render free-run fixed with three `select_live_*` selectors (`c72ba6641`) — Tar/Sphere ASM-faithful; EmoRock a marked TODO(fidelity) approx (the prepare spec's "loc_3621A has no Init_Animation" was a MISREAD — `loc_3621A` installs stru_3647E, drop animates stru_363FE; hover frames aren't extracted, so the helper pins the invariant live≠shatter-tail, not an exact frame). (3) **#340 [-]** closed NOT-A-BUG (`0091a1961`) — $62 is an orphan asset. (4) **#337 [?]** Maniaxe stuck-axe airborne gate clear (`eb6af3962`) — ported sub_942A loc_9454 as a NEW single-dispatch else-branch (NOT the prepare spec's pre-physics `!on_ground` tick, which the adversarial verify correctly flagged would double-tick on a landing frame); verify-tas-full 212 authoritative = baseline (CRC-neutral). (5) **#249/#270 DEAD END** (`0232f9588`, docs-only) — the 2026-06-19 "root cause identified (4055 wall_hit guard)" is provably a NO-OP: on a grounded slope `slope_sliding=true` so the wall snap is suppressed at movement.rs:6256 (BLW1 seam, #211) → `wall_hit` is always false → guard never fires; instrumentation confirmed `probe.hit cell=(4,4) Solid` with the snap suppressed and `wall_hit=false`. Real fix = centre-based loc_9FBC stop, needs a natural oracle route. Reverted the speculative guard rather than commit churn. (6) **#276 + #332**: confirmed both were ALREADY landed (e3c31ab3e / 7804966e7) — the NOW block's "implementation-ready" claims were stale; no new code. Process note: the prepare-the-fix agents were strong on ASM but twice wrong on native-side interactions (#249/#270 wall_hit suppression; #337 double-tick; #243 EmoRock Init_Animation) — every spec was re-derived against native before landing, which is why the adversarial-verify + serial-land split paid off. Each gameplay-affecting commit passed `verify-slice-fast` (6186+9) and #337 also `verify-tas-full` (baseline). **NEXT: #341** (sound-audit tool) then **#338 creak**; #249/#270 needs an oracle route.
- **Last session** (2026-06-19): **ported the #339 RockTank drive-SFX stop (`[ ]`→`[?]`).** The START (PlaySound `sfx_Tank_driving` $1F) was wired in `step_active_enemies_with_rng` via a pooled, recomputed refcount (count of `Tank|TankShooting` with the `shared_walker_patrol_short_next` latch); the STOP was unported, so the drone looped forever. ASM truth (verified): the per-object `$FFFFFA34` refcount decrements at loc_3C46E→loc_3C4A8 and replays `byte_3C46C[1]=sfx_Tank_driving` via `j_PlaySound2` on the 0 edge; **both** exits route there — scroll-off via the loc_3C436 bounds trampoline AND **death via Tank.asm loc_3B480 `bra.w loc_3C46E` after `Hibernate_UntilAnimFinished`** (Tank_Shooting.asm:139 likewise). Native already removes death corpses only when the death animation finishes (`remove_dead_enemy`), matching that post-hibernate timing. Fix mirrors the native pooled START: `refresh_active_roster` snapshots the pooled driving-tank count before the frame's activations/removals and emits `queue_play_sound2(TankDriving)` when it drops to 0 (`stop_rock_tank_drive_sound_on_last_exit_loc_3c46e`). The existing pure helper `deactivation_drive_sound_cleanup_loc_3C494` stays as the per-object ASM spec (its single-decrement can't express native's batch removal). Discovery via a 4-way Explore workflow; the START/STOP were initially mis-located to `activate_enemies_with_rng_inner` — they live in `step_active_enemies_with_rng`/`refresh_active_roster`. Regressions: `loc_3c46e_last_rock_tank_exit_emits_stopsequence_tank_driving_stop` + 3 (pooling 2→1 holds, silent when none drove, count filter). `verify-slice-fast` green (6183+9); full sweep 0-104 + `verify-tas-full` green (sound is advisory/CRC-neutral). Removed now-live `#[allow(dead_code)]` on `queue_play_sound2`/`QueuedSoundCommand::PlaySound2`. **UNVERIFIED**: whether `PlaySound2($1F)` audibly *halts* vs *restarts* the GEMS loop needs a real audio device (#264 headless caveat) — human to confirm the drone stops when the last tank leaves. Per-issue evidence under `#339` in archive. **NEXT: #338** (RockTank creak/shot SFX $3C/$42/$41 unported) or the #243+#333 render-flicker family.
- **Last session** (2026-06-19): **fixed #331 RockTank BLW2 wall-turn self-kill (`[ ]`→`[?]`).** Resolved the REFINED open question first: `collision_type=$FFFF` DOES kill the Tank (loc_3C464 → `jmp (a0)`=Tank.asm loc_3B41C `tst.w collision_type; bmi loc_3B480` death), so the native bit-2→terrain-kill is FAITHFUL — candidate (A) was wrong. Reproduced vs oracle (probe camera on BLW2 #03 at (1848,239)): native self-kills, oracle survives. The bug is positional: native over-advanced the tank to x=1878 on the turn frame (oracle rests at 1877) because the right-edge "extra half-step" (RightBoundaryAfterExtraAdvance, a #260 CLIFF behaviour) fired at a WALL, pulling the col-118 wall tile (a SlopeUp — `$4000` so the body scan kills, but not `Solid` so `probe_wall` ignores it) into the post-turn leftward body scan → bit-2. Fix: reverse in place (loc_3C0D2) when `wall_bit_set && moving_right && crossed_boundary && col_span*8<half_width`; guarded by the wall bit so #260's cliff path is untouched. BLW2 #03 probe now MATCHes 72/72; `verify-slice-fast` green (6179+9); full sweep 0-104 tier state 69 passed/0 failed; `verify-tas-full` CRC all 37759 frames match, 212 authoritative divergences = baseline (CRC-neutral). Regression `loc_3c0d2_rightward_wall_turn_reverses_in_place_not_extra_advance` (red without the fix). Per-issue evidence under `#331` in archive. NOTE: BLW2 start tank #01 gains a transient 2-frame facing phase at its own wall turn — a pre-existing ~1px spawn-walk lag the old fudge masked; turn position now matches oracle, sweep unaffected. **NEXT: #339** (RockTank drive-SFX stop-on-refcount-0).
- **Last session** (2026-06-19): **playtest triage — 11 new rows (#331-#341) + 4 reopens, no code yet.** Triaged a ~30-symptom human playthrough with a 15-cluster ASM+native workflow plus two focused agents (helmet-pip oracle verify; audio subsystem). Headline outcomes: (1) **#276 helmet HUD pips REOPENED — the human was right, it is a real oracle-verified bug**: the ROM draws a 5-HP Iron Knight as 5 dots and greys the trailing one on the first hit; native drops the trailing **size-0 (8x8, VISIBLE)** pip, so it shows 4→4. Prior NOT-A-BUG verdicts (and the `..._hidden_by_sub_bdc0` test) rest on a MD sprite-size-0=invisible misread — invert the test, emit the pip. (Practical gotcha: `kid.asm` has embedded NULs so plain `grep` silently misses; use `-a`. The disasm uses descriptive labels, not `loc_B700`/`sub_BDC0`.) (2) **RockTank de-confused into 3 separate things** so we stop the whack-a-mole: floor-sink = z-priority/foreground-occlusion (#259, NOT a Y bug, keep [?]); stomp-hurts = loc_1CE4 MoveID-jump gate (largely ORIGINAL, #271/#286); **BLW2 self-kill/stuck = NEW #331 regression** (57cf43bb8/410e713c0 body-overlap terrain-kill, only Robot-height-tested). (3) Slope-into-wall teleport root-caused (#249/#270, distinct from wall-cling). (4) Same-form helmet re-pickup freeze (#332). (5) Enemy flicker = render free-run (#243 reopen + #333). (6) The "USM2 tank guy" is the **Maniaxe form** (green dots = #323, not a current build?; "heavy helmet on slope" = a Juggernaut prize block) — NOT a Tank enemy; do NOT open a 0x05 Tank issue. (7) "Axe can't hit blob/dragon" is mostly not-a-bug (friendly-hit path has no invuln gate; pre-flight dragon has no collision = ROM-faithful). (8) Maniaxe can't climb slopes = ORIGINAL_QUIRK (#152, loc_9E6C). (9) Audio: voice triggers/ids ARE wired (ouch/die/bummer correct); "slow" = #264 queue-cap (resampler is rate-correct post-2f723ab8); **RockTank drive-SFX stuck loop (#339) + creak/shot unported (#338) are statically real**; diamond voice $62 unwired (#340); sound-audit tool requested (#341). Reconfirms: #259/#264/#280/#309/#323/#324/#266. Docs-only; per-issue evidence in `backlog-archive.md` (grep "2026-06-19").
- **Last session** (2026-06-18, latest+23): **fixed #330 Mode B2 blank-top-row decoder skip (`[ ]`→`[?]`).**
  `decode_level_layout` now mirrors ASM `loc_11C34` + `sub_140F6`: decode exactly the populated
  foreground height first, apply the block overlay at that height, then shift the finished grid down by
  `Number_blank_top_rows`. Logical `0x29`/resource `0x31` now decodes its 6720-byte foreground, gains two
  zero-filled top rows, and keeps solid terrain below. Regression:
  `decode_level_layout_shifts_blank_top_rows_after_loc_11c34_for_mode_b2`; focused cache-backed check and
  `mise run verify-slice-fast` green (6155). Note: the wider cache-backed
  `decode_level_layout_matches_fixture_collision_cells_loc_11c34` still exposes an unrelated level `0x42`
  high-skin/city collision-table residual; not part of #330.
- **Last session** (2026-06-18, latest+22): **fixed #329 leftover/aliased foreground render + collision (`[ ]`→`[?]`); split Mode B2 to #330.**
  Two mirrored asymmetries made leftover warp targets skip. (A render) `load_level_visuals` parsed the
  borrowed-logical foreground bytes against the mapped *resource* header (ASM `loc_11A50`) without truncating →
  now truncates to the resource grid (fixes `0x47`/`0x38`). (B1 collision) after the human warp-through (Mac, no
  `level-layout` fixtures) surfaced the remaining block-grid skips, `decode_level_layout` got the same ±5 tolerance:
  `resolve_terrain_to_expected_cells_loc_11C34` zero-pads a terrain buffer short by ≤5 to `expected_cells` instead
  of rejecting (fixes `0x2B`/`0x2D`/`0x32`/`0x39`). Smoke (real manifest): `0x47`/`0x38`/`0x32` now LOAD; `0x41`
  still skips (no foreground at all). **`0x29` still skips → #330** (blank-top-rows inflate `expected_cells` by 240,
  not the ≤5 case; needs the bigger `insert_blank_top_rows_sub_140F6` geometry change). Regressions:
  `load_level_visuals_truncates_borrowed_logical_foreground_to_resource_grid_loc_11a50`, `terrain_length_tolerance`
  unit group, cache-gated `decode_level_layout_tolerates_short_leftover_foreground_loc_11c34`;
  `mise run verify-slice-fast` green (6151).
- **Last session** (2026-06-18, latest+21): **fixed #322 render empty-frame hardening (`[b]`→`[?]`).**
  Added regressions for two renderer totality holes from the human crash lead: a present-but-empty flying-dragon
  family now falls back to the existing enemy rectangle instead of panicking on `% frames.len()`, and effect-only
  enemy sprites such as HeadyMetal split pieces skip empty loaded art instead of indexing `frames[0]`. This does
  not prove the original Under Skull Mountain 3 crash root cause; it closes the robust render path and keeps the
  issue in human validation. If the live repro still crashes, capture `RUST_BACKTRACE=1 mise run play-native` on
  Level 0x06 and file the file:line. Verification: focused `cargo test -q -p wildside-cli issue322 -- --nocapture`.
- **Last session** (2026-06-18, latest+20): **fixed #323 Maniaxe axe green-dot render (`[ ]`→`[?]`).**
  ASM `ManiaxeAxe_Init` installs `stru_8B86`, which cycles four form-art frames
  `LnkTo_unk_A589E/A5924/A59AA/A5A30`; native now traces `EnemyProjectileKind::ManiaxeAxe`
  as `ActiveProjectileKind::ManiaxeAxe`, loads `maniaxe_axe_1..4.bin` into the Maniaxe
  `PlayerFormVisuals::axe` bank with ROM `sprite_frame_unc` offsets, and renders typed axes
  as sprites instead of the generic friendly green rectangle. Engine hit delivery remains covered
  by the existing #296 Maniaxe axe tests. Verification: focused trace/render regressions,
  `cargo test -q -p wildside-core maniaxe_axe -- --nocapture`, `cargo test -q -p wildside-cli maniaxe -- --nocapture`,
  `cargo fmt --all`, and `mise run verify-slice-fast` green. Human validation: throw the axe in
  Maniaxe/Iron Knight form and confirm it is visible axe art; if enemy damage still appears wrong
  with the correct sprite, file a separate collision repro.
- **Latest playtest follow-up (2026-06-18): fixed #323 Maniaxe axe spin.**
  The previous render fix selected the right art bank but left the live axe's exported
  `animation_elapsed_frames` stuck at zero, so native stayed on `maniaxe_axe_1`.
  `step_projectiles` now advances the `stru_8B86` animation clock on each `loc_8A52`
  flight tick while preserving the zero-lifetime collision/cull semantics. Verification:
  `loc_8a52_axe_advances_stru_8b86_animation_clock` and
  `maniaxe_axe_projectile_spins_through_stru_8b86_frames`.
- **Last session** (2026-06-18, latest+19): **landed three playtest-triage fixes — #327, #326, #325 (`[ ]`→`[?]`).**
  Worked the directed triage order (skipping `[b]` #322). (1) **#327 P1 teleporter title-card**: ASM loc_BCF6 keeps
  Game_Mode=$C → Load_InGame, so teleporters reload with NO card. Extracted the shared Load_InGame loader
  (`load_level_entry_common`) out of `begin_level_entry` and added `begin_teleport_level_entry_load_ingame_loc_bcf6`
  landing in `BootState::InGame`; FBCC warp colour now stays visible to the InGame entry fade and clears at fade end.
  Cross-level already bypasses via `LevelSkipHold`. Flipped 3 StartingLevel-misread tests. (2) **#326 P3 title-card
  name**: `parse_level_name_comment_id` parsed the hex `AddrTbl_LevelNames` index comments as decimal — fixed to
  `u8::from_str_radix(_,16)`; new wildside-data test pins 0x1B/0x27. (3) **#325 P2 water palette**: theme palette path was
  numeric (`palette_fg/7.bin`) but assets are name-based — added `theme_palette_basename` (ROM-ordered name table)
  for fg+bg paths; replaced the masking numeric fixtures; real-asset Highwater regression confirms the loc_1223C band
  is non-zero. `verify-slice-fast` green (6135+); `verify-tas-full` green (advisory number_objects only). All three
  are `[?]` — human must validate visually (teleport: no card; Skydragon card name; Highwater waterfall cycles).
  Also INVESTIGATED (no code, deliberately not blind-fixed — both need work that would otherwise be a guess):
  **#323** axe sprite source pinned to `stru_8B86`→4 frames at ROM 0x0A589E + a 4-file wiring plan, blocked on
  decoding the mapping pfd offsets; **#324** confirmed the foreground/block two-plane DATA split but the RENDER
  behaviour needs jgenesis forest-level frames before reverting the scene.rs guards (blind revert risks re-opening
  the closed #289). See `#323`/`#324`/`#325`/`#326`/`#327` archive.
- **Last session** (2026-06-18, latest+18): **blocked #322 P0 crash (`[ ]`→`[b]`) — needs human repro. Docs-only.**
  First substantive investigation since the triage filed it. Disproved the archive's proposed "robust render-guard
  fix": it is **dead code**. A present-but-empty enemy family cannot occur in production — `load_enemy_visuals`
  (wildside-render/src/bundle.rs:3489) inserts a family only after `load_compressed_sprite_frames` returns `Ok` (any
  `Err`→`continue`, never inserts), that helper returns one frame per descriptor, and **no** `ENEMY_FAMILIES` spec
  has `frames: &[]` (grep over 34 specs = 0). So `families.get` is absent (→ existing `object_rect` fallback,
  sprites.rs:1512) or ≥1-frame; the `frames[0]`/`% len()` sites (sprites.rs:1494/1502; scene.rs:5602 is also
  boss-gated to 0x20/0x21) are unreachable on an empty Vec ⇒ guarding them = churn. Also re-audited the whole
  shoot→HP0→death→child chain for the 0x0C flying dragon (`EnemyFamily::Dragon`): all bounds-safe —
  `DRAGON_DIVE_SCRIPT[idx]` has a `>=len` check (enemies.rs:15356), friendly-projectile-hit loops index within
  `0..len` and remove in reverse post-loop (helmets.rs:5943/5990), death-child handling uses `retain_mut`+`get_mut`
  (combat.rs:8957/8995/8934), and the 0x0C death-frame arm is `len>=4`-guarded (sprites.rs:1372; its fallthrough is
  `% len()` = divisor-zero, not index-OOB). No safe blind fix exists and the exact `[0]`-on-empty-Vec site cannot be
  pinned statically (this audit + the original 15-agent triage both failed). Pinning needs `RUST_BACKTRACE=1
  mise run play-native` on Level 0x06 (interactive: Maniaxe form, jump, shoot a flying dragon) → needs a display the
  headless loop lacks. Blocked on that human run per the legend's "needs human repro"; once the backtrace gives the
  file:line, the fix is a targeted guard there + a regression — not the speculative render-guard sweep. See `#322` archive.
- **Last session** (2026-06-18, latest+17): **#309 shifting-block rise-table pinning committed**.
  Iteration had started on the #309 blocks umbrella (top open row before the latest+16 triage landed
  mid-session); completed it as proactive verification since #309's only open behavioural slice stays
  oracle-gated (type-8 Shooter). Discovery via `cargo mutants -f blocks.rs --list` (the full mutation
  run is blocked in this sandbox; `--list` is fast/allowed, hand-mutation is the documented reliable
  verifier) cross-referenced against the test files. Found the one block-bounce y-delta table with no
  exact-value guard: `SHIFTING_BLOCK_RISE_Y_DELTAS_LOC_113F4` (`[-2; 8]`, blocks.rs:360). Its metal
  siblings `…LOC_1117A/11196` and the prize-wobble table are all pinned
  (level_tests.rs:19807/19832/19837), but the shifting table feeds only the `BlockBounceKind::Shifting`
  arm of the `block_bounces` accumulation loop (blocks.rs:4342), and the only `shifting_block*` tests
  (runtime_tests.rs:7188/7212) assert `has_block` presence — never the rise `y_offset_px`. ASM-confirmed
  against sub_113C0/loc_113F4 (kid.asm:23332-23339): `move.w #7,d0; subq.w #2,y_pos(a3); dbf d0` = eight
  unconditional 2px-upward steps; the cell-above worker spawns at `y_offset_px = 16` (blocks.rs:3744) and
  the all-negative deltas pull it to exactly 0 as the row swap commits. New
  `shifting_block_rise_y_deltas_match_asm_loc_113f4` (level_tests.rs, beside the metal-table test) pins
  the table `[-2;8]`, the monotonic cumulative descent to -16, and the +16→0 landing. Genuine-gap proof
  by hand-mutation: flipping the first entry `-2 → 2` makes ONLY the new test fail — all 8 pre-existing
  `shifting_block*` tests still pass. `verify-slice-fast` green (6134+9); test-only ⇒ no `verify-tas-full`.
  #309 stays `[ ]` (umbrella — same residual: only the two oracle-gated type-8 Shooter deviations remain;
  the block-bounce y-delta table family prize/metal/shifting now has no mutation gap). See `#309` archive.
- **Last session** (2026-06-18, latest+16): **playtest triage — 7 new rows + 6 reconfirms, no code yet**.
  Triaged a human playtest with a 15-agent parallel workflow (per-symptom ASM+native+git+backlog dedup) plus an
  adversarial crash verify. **Actionable order for the next sessions (all `[ ]` unless noted):**
  1. **#322 P0 CRASH** — shooting a flying dragon on Under Skull Mountain 3 (Level 0x06) panics
     `index out of bounds: the len is 0 but the index is 0`. Root cause UNPINNED: the first agent's site
     (sprites.rs:1502) was REFUTED by adversarial verify (0x0C frames are statically len-9); dragon_script
     `[0]`/`[cmd]` (DRAGON_SCRIPT_* non-empty), boss-eye/HeadyMetal `[0]` (fixed arrays, boss-gated), and
     enemy-family render `frames[0]` (loader never inserts an empty family) are all ruled out. Needs
     `RUST_BACKTRACE=1 mise run play-native` on 0x06 to pin the line; the robust fix (guard the render
     `frames[0]`/`%len()` sites + skip empty-frame family inserts) holds regardless. See #322 archive.
  2. **#327 P1 teleporter shows the title-card loader** — fully ASM-traced + a clear fix: ROM keeps
     Game_Mode=$C (Load_InGame, no card) via FC29 at loc_BCF6; native routes through BootState::StartingLevel.
     BOTH same- and cross-level teleporters should skip the card. Flip the test that encodes the ASM misread
     (combat_tests.rs:2386). Most actionable of the batch.
  3. **#324 P1 block/terrain regression** — fixed candidate refined again: static live `$E7xx`/block cells follow
     ASM `sub_302F6`/`sub_3034A` and draw only Block_Mappings art, and active temporary `$6Fxx` prize-wobble
     cells are owned by ASM `sub_1007A`'s copied `unk_E0E4E`/`unk_E0E5E` object rather than leaking immutable
     foreground skin through transparent pixels. Needs human retest on a forest tree/block.
  4. **#289 P1 Skull Mountain 3 moving platforms** — fixed candidate: `PlatformScript_*-
     PlatformScript_BaseAddress` placement refs now normalise to the ASM `$A(a4)` relative script word, and stale
     absolute generated-cache offsets are repaired at runtime. Retest the Skull Mountain 3 climb.
  5. **#325 P2 water-palette regression** — fixed candidate: theme palette assets are name-based
     (`palette_fg/mountain.bin`), and live level entry now resolves the ROM-ordered theme basename before seeding
     `Palette_Buffer`. Human retest still needs to confirm the Highwater waterfall palette cycles in native play.
  6. **#323 P2 axe green dot** — fixed candidate updated: live Maniaxe/Iron Knight axes now trace as
     `ActiveProjectileKind::ManiaxeAxe`, use `stru_8B86`/`maniaxe_axe_1..4.bin` art, and advance the animation clock.
     Human retest still needs to confirm the thrown axe visibly spins.
  7. **#326 P3 title-card name** — fixed candidate: `parse_level_name_comment_id` now treats AddrTbl_LevelNames
     comments as hex, so rendered glyph cards are keyed correctly from 0x10 up. Human retest still needs to confirm
     Skydragon Castle 1 and later cards visually.
  8. **#328 `[-]`** — Cyclone "straight up / no spin / too powerful" is ORIGINAL_QUIRK (flight is y-only,
     spin is sprite-anim, no altitude cap). One cheap human check: held-C must not keep rising.
  **Reconfirms (no new rows):** #289 (appear-then-snap + hell wrong-start + dark-green sky + death-wall — the
  human evidence #283/#266/#280 were waiting on; murder-wall init verified faithful), #276 (HP "4->4->3"
  RE-CONFIRMED NOT-A-BUG: sub_BDC0 hides the odd 5th pip), #259 (RockTank floor-sink = z-priority not Y; head-stomp
  dispatch faithful per #193; tank "green dots" not reproduced), #264 (audio cluster: queue-cap frame drops on
  bursts; needs a real audio device — covers #265/#272/#287), #239 (walking dragon off-ledge = settled
  ORIGINAL_QUIRK), #309 (blocks "flip to empty" = evanescent, transient-passable by design; needs human to name
  the block). Full evidence per `#NNN` in `backlog-archive.md` (grep `2026-06-18 playtest triage`). Docs-only.
- **Last session** (2026-06-18, latest+15): **#309 evanescent-block helper pinning committed**.
  Recovered the prior (uncommitted) WIP — two exact-value tests for the type-9 evanescent-block
  restore/fire helpers — and verified both from scratch before committing. (1) `evanescent_collision_overlay_enabled_loc_ED52`
  (blocks.rs:2549): ASM-confirmed against loc_ED52 kid.asm:19240-19255 — the second `lsr.w #1,d0; bcs loc_ED7C`
  takes the theme-collision-overlay branch only when original action bit 1 (`$02`) is set; bit-1-clear
  falls through to plain `+$6000` solid. Native folds the hibernate cadence into the #214 Highwater timer
  threshold. The helper had **no direct test caller**, so its `&`/`&&`/`!=`/`>=` operators were unpinned;
  new `evanescent_collision_overlay_enabled_loc_ED52_matches_asm_action_bit1_and_timer` (combat_tests.rs)
  pins the truth table incl. real `$83`/`$80`/`$81` actions + below-threshold timer. (2) `fire_evanescent_block_projectiles_loc_EC76`
  (blocks.rs:2684): ASM-confirmed against loc_EC76 kid.asm:19124-19144 — `move.w #3,d4; btst d4,d3; dbf d4`
  fires bits 3,2,1,0 descending, d4=direction word; velocity is the round-trip inverse of `…loc_EDFA`
  (0=up,1=right,2=down,3=left). New `fire_evanescent_block_projectiles_loc_EC76_direction_table_matches_asm_bit_order`
  pins exact per-bit (xv,yv) tuples + emit order (left/down/right/up). Mutation over both helpers
  (`/tmp/mutants-evanescent-309`): **23/23 caught, 0 missed**; genuine-gap proof by hand-mutation (the
  documented `--re` caveat means the count alone isn't decisive) — `&0x02→&0x01` and the EC76 bit-3
  `-speed→speed` are caught by **only** the two new tests, no non-`demo_`/`snapshot` pre-existing test.
  `verify-slice-fast` green (6133+9); test-only ⇒ no `verify-tas-full`. #309 stays `[ ]` (umbrella — same
  residual: only the two oracle-gated type-8 Shooter deviations remain, no narrow behavioural slice; the
  evanescent-block restore/fire helper family now has no mutation gap). Next top open table row is still
  **#309** then **#310** (terrain). See `#309` archive.
- **Last session** (2026-06-18, latest+14): **#309 ice-block ABOVE arm + shooter viewport-boundary
  pinning committed**. Recovered the prior (uncommitted) WIP — two exact-value mutation pins for
  `blocks.rs` helpers — and verified both from scratch before committing. (1) ABOVE arm (`d6==2`,
  `loc_10FDC`) of `ice_block_destroy_effect_origin_sub_10F9C` (blocks.rs:2124): ASM-confirmed against
  sub_10F9C kid.asm:22881-22908 (`d6==2 → subq.w #4,y_pos(a3)`; 3→right/1→left/else→down). The three
  other arms had exact-value tests but ABOVE did not, so deleting that match arm fell through to the
  default down nudge and survived. New `ice_block_effect_sub_10f9c_uses_loc_10fdc_upward_offset`
  (runtime_tests.rs) drives spawn + one `step_visual_effects` for ABOVE (80,48→80,44); cargo-mutants
  over the helper now **12/12 caught, 0 missed** (incl. the previously-surviving ABOVE-arm delete).
  (2) Coarse viewport gate `shooter_horizontal_span_rejected_loc_F830` (blocks.rs:2840): ASM-confirmed
  the four-test reject chain against loc_F7E2..loc_F808 kid.asm:20468-20480 (d4=left col, d5=top row,
  d6=right col, d7=bottom row; rejects row>bottom, row<top, start_col>right, or start_col-left<=-8).
  The pre-existing loc_F830 test only exercised start_col>right, so the row vertical edges, the
  inclusive top/bottom rows, and the `<=-8` left-margin boundary were unpinned. New
  `shooter_horizontal_span_rejected_loc_f830_matches_asm_coarse_viewport_boundaries` (combat_tests.rs)
  pins them; `--re` is unreliable in this cargo-mutants setup (documented tooling note — it matched
  `ice_block_bullets` instead), so verified by hand-mutation: the test catches `&&`→`||` (row),
  `>`→`>=` (left margin), and `-`→`+` (subtraction). `verify-slice-fast` green (6131+9); test-only ⇒
  no `verify-tas-full`. #309 stays `[ ]` (umbrella — same residual: only the two oracle-gated type-8
  Shooter deviations remain open, no narrow behavioural slice; the ice-effect-origin and shooter-span
  helpers now have no mutation gap). Next top open table row is still **#309** then **#310** (terrain).
  See `#309` archive.
- **Last session** (2026-06-18, latest+13): **#309 shooter-block support direction-bit pinning committed**.
  Recovered the prior (uncommitted) WIP — an exact-value test for the per-direction neighbour-bit
  dispatch in `shooter_has_support_loc_f106` (blocks.rs:443) — and verified it from scratch before
  committing. ASM-traced all four directions against `sub_F096` loc_F1DE..loc_F248 (kid.asm:19775-19822):
  `d6=(word&$FF)-$2D`, then a fixed `lsr.w #1; bcc` chain assigns bit0=up (`a1-width`), bit1=right
  (`3(a1)`), bit2=down (`a1+width`), bit3=left (`-1(a1)`, `beq`); a set bit whose neighbour skin byte
  is non-zero marks the slot negative. The helper was private with no direct test — every prior
  `grid_with_shooter_blocks` test drives only the live scan with a single direction byte, so the
  bit→neighbour pairing arms had no exact-value pin. New regression
  `shooter_has_support_loc_f106_matches_asm_per_direction_neighbour_bits` pins each enabled direction
  vs its own skinned neighbour, the clear-neighbour negative, and the cross-direction negative (skinned
  neighbour whose bit is NOT set). Made the helper `pub(crate)` for the direct call. Mutation re-run
  (`/tmp/mutants-shooter-support-309`): **37 mutants, all 37 caught, 0 missed** (27 in the helper, all
  caught; the 10 incidentally-matched `ice_block_bullets_loc_11052` were already pinned).
  `verify-slice-fast` green (6129+9); test-only ⇒ no `verify-tas-full`. #309 stays `[ ]` (umbrella —
  only the two oracle-gated type-8 Shooter deviations remain open; the ice-bullet, metal-push,
  block-debris, and now shooter-support helper families have no mutation gap). Next top open table row
  is still **#309** (no narrow behavioural slice left) then **#310** (terrain). See `#309` archive.
- **Last session** (2026-06-18, latest+12): **#309 block-debris velocity-table pinning committed**.
  Recovered the prior (uncommitted) WIP — an exact-value test for the per-direction `sub_292E`
  shard velocity tables — and verified it from scratch before committing. ASM-verified all four
  tables byte-for-byte against the `dc.b` data in kid.asm (dispatch 4893-4918; tables 4949-5048):
  base `unk_29E8`/`unk_29F2`=`[0,0,8,8,4]`/`[0,8,8,0,4]`; ABOVE(2) x`[-1,-1,1,1,1]` y`[-4,-3,-3,-4,-5]`;
  BELOW(0) x`[-1,-2,1,1,-1]` y`[-2,0,0,-1,-2]`; LEFT(1) x`[-5,-4,-3,-2,-1]` y`[-3]×5`;
  RIGHT(else) x`[5,4,3,2,1]` y`[-3]×5` — all match the native tables. Only the ABOVE direction had
  an exact-shard test before, so deleting the BELOW/LEFT match arm (fall through to RIGHT default)
  survived mutation. New regression `block_debris_velocity_tables_match_asm_sub_292e_all_directions`
  pins all five shards per direction for BELOW/LEFT/RIGHT. Mutation re-run over the helper + swarm
  builder (`/tmp/mutants-debris-309`): **17 mutants, 16 caught, 0 missed, 1 unviable**.
  `verify-slice-fast` green (6128+9); test-only ⇒ no `verify-tas-full`. #309 stays `[ ]` (umbrella —
  only the two oracle-gated type-8 Shooter deviations remain open; the ice-bullet, metal-push, and
  now block-debris helper families have no mutation gap). Next top open table row is still **#309**
  (no narrow behavioural slice left) then **#310** (terrain). See `#309` archive.
- **Last session** (2026-06-18, latest+11): **#309 ice-bullet directional-table pinning committed**.
  Recovered the prior (uncommitted) WIP — an exact-value test for the per-direction ice-bullet
  offset + velocity tables — and verified it from scratch before committing. ASM-traced all four
  directions against `IceBlock_BulletPosOffset_Index` (kid.asm:22990), confirming the table holds
  *incremental* (x2,y2,x3,y3) deltas applied via `add.w (a1)+,d1/d2` at `loc_11052`, so bullet 3 =
  centre + (t0+t2, t1+t3): up `(4,4)`, right `(-4,4)`, down `(-4,-4)`, left `(4,-4)`. Velocity is the
  exact round-trip inverse of `shooter_object_direction_from_velocity_loc_EDFA`. The three prior
  ice-bullet tests only exercise remapped dirs 0/2 and never assert exact offsets/velocity, so the
  **left arm** (dir 3 third-bullet y-sign + left velocity x-sign) and the exact dir-2/3 tuples survived
  mutation. New regression `ice_block_bullet_offsets_and_velocity_match_asm_iceblock_bulletposoffset_index`
  pins all four directions. Mutation re-run over both helpers + wrapper (`/tmp/mutants-ice-dir-309`):
  **48 mutants, all 48 caught, 0 missed**. `verify-slice-fast` green (6127+9); test-only ⇒ no
  `verify-tas-full`. #309 stays `[ ]` (umbrella — only the two oracle-gated type-8 Shooter deviations
  remain open; the ice-bullet + metal-push helper families now have no mutation gap). Next top open
  table row is still **#309** (no narrow behavioural slice left) then **#310** (terrain). See `#309` archive.
- **Last session** (2026-06-18, latest+10): **blocked #308 (`[ ]`→`[b]`)** — contact-box vein
  provably exhausted. New angle vs the prior flip/transcription passes: a complete systematic
  re-audit of all 27 `*_contact_bounds` helpers (combat.rs:11595-12287) against their cited ASM
  `CollisionSize_Index` rows, re-deriving the `SubBoxLoop` arithmetic (unflipped right=left+width,
  bottom=top+height, kid.asm:3342/3350; flipped mirror `(-(L+W),-L)`, kid.asm:3354-3370) for every
  single-row helper. **Zero actionable narrow slice**: every single-row box (Fireball/Sphere/Drips/
  TarMonster/SpinningTwins/UFO/Tornado/Robot-head/FireDemon/Armadillo/DragonWalking) is arithmetically
  correct, all multi-row families are documented union approximations with mirror-consistent flip arms,
  and coverage is complete — every family has an exact-value box-edge `*_contact_uses_rom_collision_size_row`
  test (all four edges), asymmetric families have exact `*_right_facing_flips_via_loc_1be4` tests, and
  zero/death rows are pinned. No bug, no test gap. Tooling note: `cargo mutants -f combat.rs --re 'contact_bounds'`
  does NOT scope this large file (`--re` ignored, runs whole file) — do not retry. Remaining #308 work is
  uniformly oracle-gated/architectural (per-sprite-frame `CollisionSize` for union families, the invented
  generic 8×16 default + MiniHoppingSkull arms, P3 projectile-CollisionSize CRC-gated wiring, P3 generic
  `colid` side/below delivery) — identical in kind to the basis on which #306/#307 were blocked. Now waits
  on a human to schedule the per-sprite-frame hitbox refactor or supply an oracle route. Docs-only (block).
  Next top open table row is **#309** (blocks umbrella — mutation-clean as of latest+7), then **#310** (terrain).
  See `#308` archive for the full per-helper arithmetic verification.
- **Last session** (2026-06-18, latest+9): **#308 RockTank contact-box flip-arm pin committed**.
  Found the gap the flip-arm pass (73b760cbc) missed: the **RockTank** (object_type 0x0A/0x0B)
  contact box is built **inline** in `enemy_contact_bounds` (combat.rs:11618-11637), not in a
  `*_contact_bounds` helper, so it was skipped — and even its unflipped arm had no exact-value
  test (the one Tank contact test asserts only a `Damage` result, never edges). Verified the row
  byte-for-byte in kid.bin: `unk_31A26`/`unk_31A54` both read `fff3 0019 fff0 0010` =
  `dc.w -$D,$19,-$10,$10`; SubBoxLoop (kid.asm:3335-3374) → unflipped `(-13,12,-16,0)`, flipped
  `(-12,13,-16,0)`. New regression `rock_tank_contact_box_facing_flips_via_loc_1be4` pins both
  arms. `verify-slice-fast` green (6126+9); test-only ⇒ no `verify-tas-full`. #308 stays `[ ]`
  (umbrella — remaining open work is sprite-link-aware per-frame hitboxes + the
  Drips/MiniHoppingSkull/HeadyMetal gaps, neither a narrow unattended slice without an oracle
  route). Next top open table row is **#309** (mutation-clean), then #310 (terrain). See `#308` archive.
- **Last session** (2026-06-18, latest+8): **#308 contact-box flip-arm pinning committed** (73b760cbc).
  Recovered the prior (uncommitted) WIP — 11 right-facing contact-box pins — and verified it from
  scratch before committing. ASM mirror confirmed against `GfxObjects_CollisionKid_SubBoxLoop_Flipped`
  (kid.asm:3354-3370): unflipped offsets `(L, L+W)` mirror to `(-(L+W), -L)` = `(-right, -left)`,
  top/bottom unchanged. Every prior contact-box test used `test_enemy` (defaults `facing=Left`), so the
  flip-asymmetric families' `Direction::Right` arm had **no** exact-value regression — a sign/value slip
  there would survive. New pins (Goat walk/damage, Crab, Armadillo, Lion, Scorpion walk + sting-tail
  child, Robot parent + linked-head child, Tornado live + death-active, DragonWalking damage) each assert
  the exact mirror of the already-ROM-verified (2026-06-13) unflipped row; all 11 pass. `verify-slice-fast`
  green (6125+9); test-only ⇒ no `verify-tas-full`. #308 stays `[ ]` (umbrella — remaining open work is
  sprite-link-aware per-frame hitboxes + the Drips/MiniHoppingSkull/HeadyMetal gaps, neither a narrow
  unattended slice without an oracle route). Next top open table row is **#309** (blocks umbrella, mutation-clean),
  then #310 (terrain). See `#308` archive for the full mirror derivation.
- **Last session** (2026-06-18, latest+7): **#309 metal-push survivor pinning committed**. Took the
  latest+6 handoff and ASM-verified the 4 flagged `metal_block_push` mutation survivors against
  kid.asm:23130-23284. Pinned the **3 genuinely real** ones with exact-value tests (runtime_tests.rs):
  `4067 < -> <=` (scan_right right-edge bound — metal run to the edge selects no block, `<=` panics),
  `4099 || -> &&` (scan_left abort on solid non-metal block — only column-zero was tested before),
  `4242 == -> !=` (step direction-select gates x_pos +4/-4 SIGN — the Berzerker integration test only
  asserts column hand-off, never x_pos). The 4th, `4258 < -> <=`, is an **equivalent mutant** (handoff
  guess was wrong): at `col+1==width` both branches yield the same `cell_at`-clamped empty `(0,0)`, and
  it's short-circuited by the gate's width check anyway. The `4113 | -> ^` pair is **proven equivalent**
  (bit-disjoint `0x80`/`collision<<4`/`0x0F` for all CollisionKind ≤ Spikes=7). Re-run over the 3
  functions (`/tmp/mutants-metal-push-309`): **34 caught, 1 missed** (only the 4258 equivalent).
  `verify-slice-fast` green (6114+9); test-only ⇒ no `verify-tas-full`. #309 stays `[ ]` (umbrella): only
  the two oracle-gated type-8 Shooter deviations remain open (0 shipped cells; prior removal broke 18
  tests — do not retry without a type-8 ceiling-headbutt oracle probe). The metal-push helper family now
  has no non-equivalent mutation gap. Next top open table row is **#308** (combat umbrella, still `[ ]`),
  then #309 (now mutation-clean) and #310 (terrain) — see `#309` archive for the full survivor breakdown.
- **Last session** (2026-06-18, latest+6): **#309 verification slice committed**. The blocks audit's
  confirmed behavioural gaps are all RESOLVED candidates; the only open audit rows are the two type-8
  Shooter deviations (oracle-gated, 0 shipped cells, prior removal broke 18 tests — do not retry). With
  no narrow behavioural slice, ran proactive mutation testing over `blocks.rs` ice-bullet + metal-push
  helpers (`.cache/mutants/metal-push`, 65 mutants, 10 missed). **Closed** the 4 ice-bullet survivors:
  `ice_block_bullets_loc_11052` sets four non-`Default` shooter-tracking fields (`shooter_tracks_layout=true`,
  layout col/row=source, `shooter_state_word=$FFFE`) mirroring the canonical `shooter_block_loc_EC76`
  bullet (ROM models ice bullets as ShooterObject bullets that track their source cell). New regression
  `ice_block_bullets_loc_11052_track_source_layout_cell_like_shooter_object_bullets` pins all three bullets;
  re-run reports 10 caught / 0 missed (`.cache/mutants/ice-bullets-fields`). `verify-slice-fast` green (6111+9);
  test-only. #309 stays `[ ]` (umbrella). **Handoff:** 6 metal-block-push survivors remain (`blocks.rs:4067/4099/4242/4258`
  real; `4113×2 | vs ^` likely equivalent) — next #309 iteration ASM-verifies and pins the 4 real ones; see `#309` archive.
- **Last session** (2026-06-18, latest+5): **#308 contact-box transcription fix committed**. Recovered the
  prior (uncommitted) WIP on the combat umbrella and *verified it from scratch* before committing. Two
  enemy contact boxes were transcribed wrong against the `GfxObjects_CollisionKid_SubBoxLoop` convention
  (right = left + width@`2(a3)`, bottom = top + height@`6(a3)`, confirmed at kid.asm:3342/3350): Sphere
  `unk_31AB0`/`unk_31ABA` (`fff9 000e fff9 000e` in kid.bin = `-$7,$E,-$7,$E`) had bottom clamped to 0,
  halving the 14×14 box centred on the ball's `$1E(a2)` anchor → corrected to +7; Drips parent
  `unk_3189A` (`fffc 000a fffe 0002` = `-$4,$A,-$2,$2`) had right=5/bottom=-1 → corrected to right=6/bottom=0.
  Both rows read byte-for-byte from `kid.bin`. Verification: `verify-slice-fast` green (6110+9); `verify-tas-full`
  reports **CRC all 37759 frames match, exit 0**, with an identical divergence profile to HEAD (212 pre-existing
  authoritative checkpoint divergences exist on baseline too — confirmed by a stash-and-rerun A/B) so the change
  is **CRC-neutral** on the published TAS. #308 stays `[ ]` (umbrella — many per-sprite-frame hitbox gaps remain).
  Next top open item is **#309** (blocks umbrella).
- **Last session** (2026-06-18, latest+4): **blocked #307 (`[ ]`→`[b]`)** per the loop anti-spin rule (2nd
  dead-end note, no genuinely new *outcome*-producing narrow angle). New angle vs the prior three runs:
  mutation-tested the freshly-wired (2026-06-17) **live `Init_SpecialEffects` storm machine** — the
  `weather.rs` helpers reached from live `init_weather` for weather types 2/3, untouched by the earlier
  level.rs island/lava/blank-top-rows runs. `cargo mutants -f weather.rs --re 'loc_14C..|loc_14D..|init_weather'`
  (`.cache/mutants/weather-storm`) → **39 mutants, 29 caught, 10 unviable, 0 survived**. Every viable mutant
  (the `fill_count = loop_count+1` slot count, the `4*8*8*4` written-longword product factors, the rain/hail
  bool→enum branches) is killed by the exact-value tests in `combat_tests.rs:22228..27396`. This is the **4th
  consecutive clean mutation result** on #307's post-load helper families (blank-top-rows-after, island,
  lava-rotate, storm — 0 survivors across all four), so the subsystem has no behavioural test gap left to
  close. The remainder is identical in shape to #306: only architectural gaps with real fidelity payoff remain
  (`EnemyStatus_Table`/`Manage_EnemyLoading` streaming, `off_7B1EC`/`loc_3050A` BG scroll, `loc_11E48..`
  theme-art VRAM) — none a narrow unattended slice, each needing a dedicated session **and an oracle route**.
  #307 now waits on a human to schedule that or supply an oracle route. Docs-only (block). Next top open item is **#308** (combat umbrella).
- **Last session** (2026-06-18, latest+3): **#307 verification-slice scan — dead end, no commit**. Followed the
  prior session's flagged angle (harden tests on the **live** post-load palette/object helpers). Mutation-tested
  both relevant live slices: `.cache/mutants/level-loading-live` (`classify_post_load_object_routine_loc_11F96`,
  `classify_island_background_object_loc_11FB0`, `select_level_start_music_loc_11FD2`/`select_foreground_theme_music_loc_12004`,
  the three Island palette-cycle bands `loc_12098/12110/12188`, live `step_island_shore_cycle_loc_1202a`) →
  **31 mutants, 25 caught, 6 unviable, 0 survived**; and `.cache/mutants/lava-rotate` (live `rotate_palette_lava_water_loc_1223C`
  4-word rotation + `step_rotate_palette_lava_water_loc_1223c` reload branch) → **15 mutants, all 15 caught, 0 survived**.
  No gap — every viable index/`==`/`&`-wrap mutant is killed by the existing exact-value tests. So the live post-load
  palette/object slice is already fully pinned; the only remaining #307 work is the architectural dead-islands
  (theme-art VRAM decompress, BG scroll, block-mapping tilemap, `EnemyStatus_Table`/`Manage_EnemyLoading` streaming),
  none a narrow unattended slice without an oracle route. First dead-end note on #307, left `[ ]` (not blocked — other
  live-helper verification slices remain viable). Docs-only (the `cargo mutants` unmutated baselines reconfirm the
  wildside-core lib suite green on HEAD). Next top open item is **#308** (combat umbrella).
- **Last session** (2026-06-17, latest+2): **#307 verification slice committed**. Recovered the
  prior (uncommitted) test-hardening on the live `sub_140F6` blank-top-rows shift: 4 exact-value
  unit tests in `terrain.rs` pinning `BlockGrid`/`BlockList::with_blank_top_rows_sub_140F6`. Re-verified
  the mutation gap is closed — `cargo mutants` over both methods now reports **8 mutants, 6 caught,
  2 unviable, 0 survived** (matches the archived before=0-caught/after=all-caught run). `verify-slice-fast`
  green (6110+9). Test-only ⇒ `verify-tas-full` not required. #307 stays `[ ]`: remaining gaps are
  rendering/VRAM dead-islands + the `EnemyStatus_Table`/`Manage_EnemyLoading` streaming machine
  (architectural, needs an oracle route) — no narrow unattended gameplay slice. Next top open item is **#308** (combat umbrella).
- **Last session** (2026-06-17, latest+1): **blocked #306 (`[ ]`→`[b]`)** per the loop anti-spin
  rule (2nd dead-end note, no new narrow angle). New angle vs the prior static walk: mutation-tested
  the pure collision-resolution helpers in `platforms.rs` (regex over `overlap_resolution|push_magnitude|
  platform_carry|apply_horizontal_platform_push|platform_landing|…`, output `.cache/mutants/platforms-pure2`)
  → **24 mutants, 11 caught, 13 unviable, 0 survived**. The 13 unviable are `Default::default()` subs on
  no-`Default` return structs; every viable sign/compare/branch mutant is killed by the exact-value tests
  in `block_tests.rs:16016-16216`. So the platform collision math has no test gap; the only remaining #306
  work is the architectural Vec→Platform_RAM linked/free-list refactor — non-behavioural, no oracle-observable
  payoff on swept levels, gate-risk only, a dedicated refactor session not an unattended slice. #306 now waits
  on a human to schedule that. Docs-only (block). Next top open item is **#307** (level-loading umbrella).
- **Last session** (2026-06-17, latest): **blocked #301 (`[~]`/`[ ]`→`[b]`)** per the loop anti-spin
  rule (2nd dead-end note, no new narrow angle). Confirmed the last plausible unpinned pure-function
  slice — the `sub_37AF0` 8-direction compass eye-attack sprite selector — is already faithful
  (`boss_eye_attack_sprite_sub_37332`, combat.rs:2956) AND already comprehensively pinned
  (`boss_eye_attack_sprite_sub_37332_matches_sub_37af0_direction_buckets_and_diamond_override`,
  weather_tests.rs:207). The three remaining audit rows (#136/#137/#142) are all architectural
  first-class object-slot scheduler lifecycle work with **no boss oracle (#205)** to verify — blind
  implementation would breach "never guess / oracle is truth". #301 now waits on #205. Docs-only
  (block). Then investigated **#306** (platforms umbrella) → **first dead-end note, left `[ ]`**: every
  behavioural row in `audits/platforms-fidelity-audit.md` is FIXED/partial; the live airborne collision pass
  already loops/accumulates/restarts across all platforms (`platforms.rs:1596-1635`); the only open
  work is the **non-behavioural** Vec→explicit-Platform_RAM-linked-list representation refactor (no
  oracle-observable payoff on swept levels, gate-risk only) plus the no-op `ext.l` index sign-extension
  (verified no-effect-on-base-ROM, offline asset build). No narrow unattended slice — needs a dedicated
  no-behaviour-change refactor session (sweep-green as the only check). Audit lines 82/111 cites are
  stale (line 82 is now the live `has_contact`). Next top open item is **#307** (level-loading umbrella).
- **Last session** (2026-06-17, later): #301 narrow-slice scan — docs-only, no gameplay patch.
  The two ASM-named narrow gaps in `audits/bosses-fidelity-audit.md` are both unactionable: the
  `sub_370D2` odd-frame `sfx_Plethora_x` timeout variant is **already faithful and tested**
  (corrected the stale audit prose), and the `sub_3A292` `palette_line` 2/3 flash is
  **render-gated, not engine-gated** — the live path already computes `next_palette_line` but
  the renderer loads only one CRAM row per enemy variant, so there is no row-3 boss-art data,
  and faithfully replacing the (deviating) white-flash needs the boss's 4-row palette + a boss
  oracle (#205). The `loc_3724A`/`sub_3728C` tail is object-context-ambiguous without macro
  defs. Recorded all three as the first explicit dead-end note under #301 in the archive.
  #301's narrow verifiable slices are exhausted; remaining work is the large first-class
  per-piece/eye object-slot scheduler lifecycle (architectural, no boss oracle to verify).
  **Next session on #301**: if no genuinely new narrow angle, set `[b]` per the anti-spin rule
  (this was the 1st dead-end note; a 2nd triggers the block).
- **Last session** (2026-06-17): blocked #260 (`[~]`→`[b]`) per the loop anti-spin rule —
  every synthetic route/bootstrap axis from 2026-06-07..17 is exhausted (Kid/block bounce
  near-misses only, Drips child always vertically separated, oracle often dead before the
  window); it now waits on a fresh human-equivalent live recording reaching the actual
  slime/metal-block contact. Then worked #301: audited the accessible HeadyMetal death
  routine `sub_37708` (kid.asm:39503) and found it faithfully implemented; the only real
  gap was test coverage of the `unk_3795E` piece-selection order (pinned at 3/37 phases),
  so added `heady_metal_breakup_step_sub_37708_selects_pieces_in_unk_3795e_order` pinning
  all 37 phases (verified byte-for-byte vs `unk_3795E` at kid.asm:39723; mutation-checked).
  `verify-slice-fast` green. Note for next session: the 0x37000 boss code lives in
  `kid.asm` keyed by `kid.asm:<line>` comments, NOT `loc_37708`-style labels — grep by the
  line number cited in code comments. #301's remaining gaps are the large architectural
  first-class per-piece/eye object-slot scheduler lifecycle, not narrow slices.
- **Next**: top open item is now #309 (P2 blocks umbrella, mutation-clean as of latest+7); #308 is `[b]` (contact-box vein provably exhausted by a full 27-helper arithmetic re-audit — remaining work is the oracle-gated/architectural per-sprite-frame `CollisionSize` hitbox refactor + projectile/colid oracle slices). #307 is `[b]` (behavioural verification slices exhausted — 4 consecutive clean mutation runs; architectural enemy-streaming/BG-scroll/theme-art VRAM remains, needs a dedicated session + oracle route). #306 is `[b]` (architectural linked-slot refactor only — needs a dedicated no-behaviour-change session, mutation-confirmed no collision-math test gap); #301 is `[b]` (waits on #205 boss oracle); #260 is `[b]` (human recording).
- **Earlier — last session** (2026-06-10): #290 TAS frame_7800 object-count divergence proposed fixed.
  Root cause was Dragon.asm loc_3CE90/loc_3CF90 walking-dragon breath child counting,
  not the suspected tank-pair collapse. Frames 7790..7810 match, the full
  published TAS now passes all 37759 frames by CRC, and `verify-slice-fast` is
  green.
- **Audit programme** (2026-06-10/11): ten new fidelity audits landed (blocks/combat/
  terrain/projectiles f307e9c4; shell/scoring/camera/level-flow/bosses/audio 12c81289)
  plus a staleness re-check of the six May audits (b2029fcd: 82 of 258 fixed, 140 still
  open; platforms 5/40 and level-loading 9/55 barely touched). 177 new adversarially
  verified gaps. Promoted #292-#300 as individual P1s, #301 bosses cluster, #302-#304
  P2s, #305 interactive-lane verification, #306-#320 per-doc umbrellas. Per-gap
  evidence lives in the audit docs (each ends with backlog-candidate rows), not the
  archive; the staleness report carries fixed/still-open verdicts with file:line.
- **Latest playtest, 2026-06-14**: latest pushed build still has several human-visible
  failures despite green focused gates. Most urgent: first-level stair/slope jump
  behaviour is still wrong in both directions (holding right into right stairs then
  jumping fires Kid up/right about 45 degrees; holding left into left stairs barely
  moves left after a delay). Treat this as fresh #258/#277 evidence and build a
  human-equivalent oracle route before another narrow helper patch. Other
  reconfirmed items: intro jacket appears before throw-off (#288/#161), level starts
  flicker / Kid briefly appears in the air (#122/#238/#289), block/diamond/helmet SFX
  still lag or deform music (#264/#272), Heavy Man top metal-block ding may be missing
  (#272/#297), helmet-guy hitting Dragon still appears to leave HP at 4 (#276/#271
  recheck), EOL countdown sound continues past zero until BLW2 intro (#265), BLW2
  RockTanks render half through floor near tunnel/teleporter (#259/#252), same-level
  teleporter visibly shows loader instead of only the teleporter effect (#298/#300),
  Samurai transform/attack audio is weak (#272/#287), Highwater Rhino/Berzerker still
  draws in front of water/foreground (#256/#215), Highwater Pass 2 hands flicker and
  diamond enemies are harmless (#185/#177), Level 0x08 death wall now appears but
  overlays the timer and continue screen is off-centre (#266/#303), death-wall level
  still has dark-green/cloud overlay artefacts (#280/#289), and later intro/Cyclone
  prize route still gives a Diamond and blocks progress (#284/#274/#273).
- **Next**: #258/#277 first-level stair/slope jump repro, then #284/#274 Cyclone prize
  if the movement route does not yield an actionable oracle split.
- **#258/#277 repro update (2026-06-14)**: added named probe
  `level00-slope-right-jump-probe` / recording
  `.cache/recordings/issue258-slope-right-jump-probe/scenario.json`. It starts
  at BLW1 `(760,231)`, holds Right after one neutral explicit-start frame, and
  pulses B on frame 13. Native/oracle diverge around the slope jump: at frame 15
  native keeps `kid_x_vel=0x6000` and frame 17 reaches `0xB600`, while oracle
  enters the `loc_AA22` jump/wall loop with `kid_x_vel=0` at frame 15 and only
  `0x2B00` by frame 17. Rejected hypothesis: this is not the slope-walk
  side-clamp guard suppressing `sub_8F26`; the native wall probe reports no hit
  on the split frame. Next investigation should trace the one-pixel earlier
  oracle slope phase / `loc_AA22` entry rather than broadening slope-as-wall
  collision.
- **#258/#277 follow-up (2026-06-14)**: the direct Level 00 probe is not clean
  movement evidence yet. `tas-diff` shows native/oracle already offset before
  the jump, and a 3x3 start-position nudge matrix over `(759..761,230..232)`
  still diverges at frame 0 with native Kid X consistently `$2000` subpixels
  behind the oracle sample. Natural routes are cleaner: Level 00 right-slope
  jump timings `428`, `432`, `437`, and the one-frame B pulse all pass
  `verify-tas` through the jump window; BLW2 stair-jump right-current and
  right late-B `76/84/90` all have no authoritative gameplay divergence; BLW2
  left early-tap also passes. BLW2 left-current diverges first in
  `number_objects` at frame 90, with later camera/object drift from the
  RockTank/ShooterBlock lifecycle, not Kid movement fields. Next movement work
  needs an exact live input recording or a fixed direct-probe bootstrap before
  patching slope/jump physics.
- **#258/#277 follow-up (2026-06-15)**: the direct Level 00 slope-jump bootstrap
  is now trace-clean for its checked window. Native now reports the same
  `Character_Movement=6` slope-walk export as oracle when `loc_9D22` has written
  MoveID 6 and same-frame `loc_9F18` clamps the slope velocity to zero. Focused
  checks on `.cache/recordings/issue258-slope-right-jump-probe/scenario.json`
  report `tas-diff --include-advisory --include-physics-trace` `30 match, 0
  diverge` and `verify-tas --frame-start 0 --frame-end 30 --checkpoint-interval
  1 --frontier-kind gameplay --bisect` passes. This unblocks using the direct
  probe as a bootstrap sanity check, but it still does not reproduce the latest
  human right/left stair launch complaint; next #258/#277 work needs a
  human-equivalent stair route or live input capture before changing jump
  physics.
- **#258/#277 follow-up (2026-06-15, later)**: the cached BLW2 right-stair
  fixture `.cache/recordings/issue277-blw2-stair-jump-right-current/scenario.json`
  is stale at frame 0 (`kid_y_pos` native `$01800000`, oracle `$017F0000`).
  A cache-only one-pixel-up start nudge
  `.cache/debug/issue277-start-nudge/right-y382.json` verifies cleanly through
  frame 180, so the scripted right-stair jump route is not current movement
  evidence. The paired left-current route also diverges at frame 0
  (`number_objects` native 4, oracle 5). The clean Level 00 slope-jump route
  matches frames 10..24 including physics trace, so the live 45-degree launch
  complaint still needs a new human-equivalent input capture.
- **#258/#277 follow-up (2026-06-15, held-jump routes)**: refreshed natural
  Level 00 route `.cache/recordings/issue-258-level00-natural-right-current/`
  and tested held-B+Right overlays around the first slope/stair window. Best
  native route `400..500 B+Right` reaches the slope and later target cleanly,
  but the materialised scenario `.cache/debug/issue258-natural-b400-500/scenario.json`
  passes `verify-tas --frame-start 380 --frame-end 560 --checkpoint-interval 1
  --frontier-kind gameplay --bisect` by CRC, so this held-right/held-jump
  natural route still does not reproduce the human 45-degree launch as an
  oracle divergence. The cached Level 00 left direct probe remains blocked as
  comparator input: original y=303 and nearby y=304/y=305 all first-diverge at
  frame 0 (`kid_y_pos` or `character_movement`), while y=302 jumps native to
  `$01210000` vs oracle `$012E0000`. Do not patch slope/jump physics from these
  direct left artefacts; the missing piece remains a human-equivalent live
  capture or a clean natural left route.
- **#258 fixed candidate (2026-06-15)**: direct right-slope jump route
  `.cache/recordings/issue258-slope-right-jump-probe/scenario.json` now passes
  `verify-tas --frontier-kind gameplay` across all 120 frames. Native had two
  ASM handoff gaps: frame 36 over-advanced X by `$A500` when `sub_902A ->
  loc_A99E -> loc_8BF0 -> loc_9D22` recontacted the first `$405C/$605D`
  SlopeUp/Solid edge at post-vertical fraction `$2A00`; frame 100 stayed in
  diagonal slope-walk instead of taking the `loc_9F18 -> loc_9FBC` flat-support
  exit and clearing `y_vel`. Regressions
  `level00_frame_36_direct_right_slope_jump_recontact_matches_oracle_loc_a99e`
  and `level00_frame_100_direct_right_slope_exit_clears_diagonal_y_vel_loc_9fbc`
  cover both exact oracle frames. This specifically addresses the held-Right
  BLW1 slope/stair launcher route; #277 left-stair/live BLW2 validation remains
  open until a clean human-equivalent route is captured.
- **#277 route repair (2026-06-15)**: repaired the stale BLW2 left-stair
  fixture with cache-local start nudge
  `.cache/debug/issue277-start-nudge/left-x1746-y237.json`; it passes
  `verify-tas --frame-start 0 --frame-end 180 --checkpoint-interval 1
  --frontier-kind gameplay --bisect`. Grounded `LEFT+B` overlay
  `.cache/debug/issue277-start-nudge/left-x1746-y237-b96.json` is also
  oracle-clean through the stair/jump window; native frame 98 takes
  `kid_grounded_release_probe held=left/jump=1` and
  `kid_jump_gravity_probe branch=jump_pressed` with negative x velocity. This
  rejects the repaired BLW2 scripted stair fixture as the current human
  left-input-drop repro. Earlier B88/B90-style pulses are contaminated by
  ShooterBlock damage before a useful grounded jump signal, so do not patch
  movement from those timings.
- **#277 recheck (2026-06-16)**: the repaired `left-x1746-y237-b96` fixture is
  gameplay-clean through the checked stair/jump window again after the shooter
  slot reread. `c80b9390`/`f5adb2090` first narrowed the native-only frame-165/166
  damage to a fresh visible `sub_F096` slot at source cell `(86,19)`, then the
  ROM reread showed `loc_F28C/loc_F410` only compile visible sprite entries;
  moving `ShooterObject_RAM` bullets are allocated separately by `loc_EC76`
  from evanescent/touch-triggered block workers. Native no longer spawns
  `EnemyProjectile::shooter_block_loc_EC76` from visible slots. Verification:
  `verify-tas --frame-start 0 --frame-end 180 --checkpoint-interval 1
  --frontier-kind gameplay --bisect` passes, and focused `tas-diff
  --frame-start 164 --frame-end 169 --frontier-kind gameplay --include-advisory
  --show-object-roster-on-match --include-physics-trace` reports `5 match, 0
  diverge`. This removes the fixture's shooter contamination but still rejects
  it as the current human dropped-left movement proof; keep #277 open for a
  fresh human-equivalent stair route if the visible complaint persists.
- **Then**: #260 live slime/metal-block route (the Drill-skin headbutt blocker is now
  gameplay-CRC clean on `.cache/debug/drips-route-candidates/orig_l480_530_b528.json`
  through frame 570, so do not re-open block type C/`sub_BB54` from that route; still
  need a route that reaches the reported live slime/metal-block contact. Current clean
  near-miss: `.cache/debug/issue260-tail-grid/l524_b530_540.json` passes
  gameplay CRC and enemy checkpoints through frame 560. A 2026-06-14 `480..540 B`
  lift overlay gets Kid's healthy hitbox to row-32 cells `32,32`/`33,32` at
  `dx=0 dy=-1 @490`, improving the old `dx=0 dy=-16` miss, but it is still not
  the slime/metal-block repro: the closest DripsSurfaceCrawler remains
  `dx=+102 dy=+1 @512`, and the active DripsDrop is still far right. Do not
  repeat this late B-lift window unchanged; it is a Kid/block-line near-miss
  only, not a live enemy/block contact. 2026-06-14 follow-up added durable
  route-search scenario `issue260-dlb-tail-b-row32-bounce`: it preserves the
  `dlb_tail_b` p11 overlay (`150..250 Left`, `251..350 B+Right`, `351..430 Right`,
  neutral `431..500`, `501..570 B+Left`) and verifies enemy-frontier clean through
  frames `450..530`; `tas-diff` frames `500..516` has `16 match, 0 diverge`.
  It produces a matched Bounce on Level 01 cell `37,32` at frame 511, with Kid's
  healthy hitbox `dx=0 dy=-1`; DripsDrop is still closest to Kid at `dx=0 dy=-1
  @519`, and the crawler appears later at `dx=+12 dy=-1 @577`. No Destroy occurs,
  so this is a better oracle-comparable row-32 bounce near-miss, not the reported
  slime/block destruction route. Next search should move the live Drips child onto
  the row-32 block line before/at the bounce instead of repeating Kid-only bounce
  tails. 2026-06-16 follow-up rechecked the durable route with the corrected
  baseline overlay (`0-0:right`) and confirmed the same outcome: enemy-frontier
  `verify-tas` passes frames `450..530`, `Bounce cell 37,32 @511` is present,
  and `Destroy` is still absent. A `lower_170_300` early-approach probe improves
  the DripsDrop/cell conjunction score to 40 but loses the row-32 bounce and ends
  with HP 0, so reject that route shape as a closer death path, not block proof.
  2026-06-14 follow-up route batches on the same committed scenario reject
  the obvious early-right acceleration axis: cleared-script `fast_*` routes overshoot
  into `LevelTransition` and lose DripsDrop/DripsSurfaceCrawler sightings entirely,
  while intermediate `mid_*` routes preserve Drips but leave Kid horizontally near
  row 32 while vertically separated (`dy=-33` or worse) and still report no
  Destroy/Bounce at cells `36,32`/`37,32`. Longer jump-hold `lift_*` variants can
  make an early Kid/block bounce (`lift_a`, `Bounce cell 37,32 @260`) but again have
  no live Drips projectile in the scored window. Do not repeat early-right/long-B
  acceleration from this base unchanged; the useful next axis needs a route that
  preserves the Drips child and raises Kid into row 32 without advancing into the
  early transition/bounce-only branch. Tooling now supports
  `probe-record-scenario-file-batch --target-projectile-cell-conjunction KIND:COL,ROW`
  to rank same-frame healthy Kid hitbox + projectile proximity to a cell. Re-scoring
  the intermediate branch shows `mid_g` is the real best same-frame lead
  (`drips-drop:36,32` score 36 at frame 308: Kid `dx=0 dy=-33`, drop
  `dx=0 dy=-3`), while `lift_a` is only a no-Drips early bounce. A tighter
  `mid_g` second-jump timing sweep (`mg_j220..mg_j270`) does not improve the
  vertical miss: best remains score 36 with `kid_dy=-33`, and no Destroy/Bounce
  proof appears. Do not repeat this `mid_g` second-jump timing axis unchanged.
  2026-06-14 `issue-260-early-jump-r360-conjunction` exposed the same
  falling-child contact timing on a shorter route: before the fix native kept
  HP at 2 while oracle took Drips `$28` damage at frame 372, then an over-broad
  first pass damaged frames 368-371. The committed fix gates DripsDrop
  object-collision damage until `loc_3345C` has reached the oracle-visible
  y-velocity window (`$8000+`); `verify-tas` frames `350..390` now passes all
  40 enemy-frontier checkpoints. This is contact-timing evidence only, not the
  row-32 block destruction route.
  2026-06-14 follow-up after materialising
  `.cache/recordings/issue260-dlb-tail-b-row32-bounce/`: targeted
  same-frame conjunction batches around the `431..500` neutral tail and late
  B+Left timing still fail to produce a live Drips/row-32 metal-block event.
  Best healthy candidate from the first pass was `rb471_500`
  (`471..500 Right+B,501..570 Left+B`), but it only scores 64 against
  `DripsDrop:36,32` at frame 640 (Kid `dx=+28 dy=-33`, drop `dx=0 dy=-3`)
  and has no Bounce/Destroy. The tighter follow-up found `rb451_500` with Kid
  and drop horizontally aligned to cell `36,32`, but Kid is still `dy=-33`,
  finishes at HP 0, and has no row-32 Bounce/Destroy. Do not repeat this
  `rb451..500`/late B+Left tail timing axis; it confirms the child can be near
  the cell horizontally, but not vertically or as a block-mutating contact.
  2026-06-15 follow-up: single `probe-record-scenario-file` now accepts
  `--target-projectile-cell` and `--target-projectile-cell-conjunction`, matching
  the batch scorer for deep inspection of one route candidate. On
  `issue260-dlb-tail-b-row32-bounce`, it confirms the base near-miss has Kid's
  healthy hitbox on cell `36,32` at frame 511, but the closest `DripsDrop`
  cell approach is frame 348 (`dx=0 dy=-3`) and the best same-frame conjunction
  is frame 640 with Kid still `dx=+127 dy=-33`; no Bounce/Destroy follows.
  The `pretap_*` pre-contact batch axis is rejected too: best `pretap_holdr`
  regresses to conjunction score 163 at frame 640 with no block event, while
  most variants stay at score 185 around frame 360. Do not repeat this
  pre-contact tap axis; the next route search needs an earlier/lower block
  approach or delayed Drips phase.
  Named route
  `issue260-drips-drop-contact-probe` promotes the `l528/b530/r536` tail and
  reaches exact native DripsDrop overlap at frame 535; oracle diff for frames
  520..545 has only the known non-authoritative `number_objects` limitation. It
  is route evidence, not the human metal-block contact repro. The right-side raw
  `dx=+13 dy=-1` candidates are HP-zero/LevelTransition routes; healthy closest
  remains `(584,543)` `dx=0 dy=-48`, so do not mine them as proof. Do not reuse
  `.cache/debug/issue260-route-probes/floor-explicit-left-drips-close-617.json` as live
  metal-block evidence. Fresh 2026-06-10 trace reconfirms that exact route is already
  Drips-phase shifted before contact because it uses `settle_oracle_warp=true`: native #08
  seeds at frame 13 with wait `$43` while oracle exposes `loc_33424 d0=$5d` at frame 15.
  The no-settle copy starts #07/#08 clean but is not a valid late metal-block proof either
  because the oracle side has lost the comparable Drips slots by frame 590. Next useful #260
  work is a genuinely refreshed route/bootstrap that starts clean and stays oracle-comparable,
  not Tank/Drips hitbox/block patches from these stale route files. Post-floor delayed-jump
  overlays from the current `l524_b530_540` tail (`release 548/556/560`, neutral/right/left
  jump variants) still rank the old frame-540 near-miss as the best healthy `(456,543)`
  approach, and right-biased variants spend HP without improving row-32 contact; do not
  retry that late-jump axis unchanged. Source-generated `issue260-drips-cleaner-prefix-audit`
  / `issue260-drips-b-jump-climb-seed` plus the `140..300:none,170..260:left` overlay no
  longer reproduces the archived `dx=-2 dy=0 @496` row-32 branch on current native; treat that
  archived branch as cache-specific until a current DynamicScenario file is recovered. Clearing
  `300..479` on the clean no-settle scratch route and restoring the late tail also leaves the
  same early healthy approaches (`(456,543)` `dx=-3 dy=-16 @68`, `(584,543)` `dx=0 dy=-48 @127`);
  do not repeat that idle-window axis unchanged. 2026-06-11 follow-up: a current
  `issue260-drips-drop-contact-probe` pre-380 prefix sweep (`160..330` left-window shifts,
  no-overlap left windows, split-left windows, and `260..360` coast/brake/right/B+right variants)
  also failed to improve the late row-32 target. Windowed probes over frames 300..620 still leave
  the best live late route as the already-documented `wait390`/`left400` family (`(584,543)`
  healthy `dx=+1 dy=-18 @444` or `dx=0 dy=-22 @470`, both later LevelTransition), while the
  drop-contact prefix variants either stay far left at the Drips Y row or keep the old
  `dx=0 dy=-48 @128` early pass. Do not repeat this pre-380 left-window axis unchanged.
  A later `450..469 B+Right` overlay on the named drop-contact probe improves healthy
  right target proximity to `(584,543)` `dx=-1 dy=-16 @539`, but it is not metal-block
  evidence: oracle damage at frame 529 comes from a right-side Drips crawler already
  active at `(588,592)` while native layout #08 is still in
  `pre_drop:0x8005` and has not spawned that cycle's child. Treat this as another
  phase-shifted route rejection, not a Drips hitbox or block-break lead. The cleaner
  `470..500 B+Right` variant verifies cleanly through frame 545 while staying alive,
  but still only reaches `(584,543)` at healthy `dx=+1 dy=-20`; do not repeat late
  right-window trimming as metal-block proof without a route-shape change. A fresh
  no-settle 720-frame route rebuild found native-live candidate `freshF`
  (`0..160 Right`, `55..125 B+Right`, `160..240 Left`, `240..320 B+Left`,
  `340..430 Right`, `478..528 Left`, `530..567 B+Right`) reaching `(456,543)`
  healthy `dx=0 dy=-16 @538`, but it is not oracle-comparable. After direct
  probe oracle warps were fixed to target Level 01 instead of stale Level 00,
  and the direct-probe title-card Start latch was drained/phase-aligned, Kid
  advances on the same input phase as native. The route still diverges
  immediately on bootstrap roster shape (`number_objects` native 4 vs oracle 5),
  and the late window first differs on HP at frame 480 (`native=2`,
  `oracle=1`) with oracle already exposing Drips `loc_33488` child/gfx slots.
  Treat `freshF` as no-settle bootstrap evidence only, not route proof or a
  Drips/block patch lead. 2026-06-11 follow-up after `513b36d1`: the
  focused right-side Drips contact route
  `.cache/debug/issue260-route-probes/drop-contact-br460-492-l493.json`
  now passes enemy-frontier oracle verification for frames 470..559. The fix
  aligned `Drips.asm loc_3340E` initial seed timing with the visible Kid gfx
  slot, preserved the two-frame loc_333F6→loc_3340E coroutine bridge, and made
  live `loc_33550` crawler contact consume the frame before Kid's horizontal
  movement. This clears the stale Drips initial-seed/contact hypothesis, but it
  still is not the reported live slime/metal-block repro; keep #260 `[~]` and
  continue from a refreshed route/bootstrap that reaches metal-block contact.
  2026-06-14 follow-up: `.cache/debug/issue260-brakehop-probe/oracle-frame-scenario.json`
  no longer diverges on the false Diamond pickup. Root cause was loader
  reconciliation rewriting BLW2 live cell `0xCE8A` from `$EC2D` (type-C no-op
  for `off_BB80`) to `$E102` purely from the durable prize Block List. Native
  now preserves non-prize live `Level_Layout` cells and the route verifies with
  no authoritative divergences through frames `0..700`. This is a fixed
  sub-case only: it removes the stale centre-`EC2D` false-prize branch, but it is
  still not the reported live slime/metal-block contact repro. Success criteria
  on these centre cells should not be "find `Destroy`": raw `$EC2D` is a type-C
  Drill/shooter cell, and ROM `off_BB80[$C]` is a headbutt no-op. Prefer probes
  that show the reported red projectile/type-C route or a live Drips child
  occupying the same row-32 cell window while Kid contacts the cell. A brakehop-seed
  tail branch, `.cache/recordings/issue-260-brakehop-b450-l485/scenario.json`
  (`300..430 Right`, `431..449 none`, `450..484 B`, `485..520 Left`), verifies
  enemy-frontier clean for frames `450..510` and `tas-diff` reports 57 matched
  frames. It bounces `cell 37,32 @460` and later gets a `DripsSurfaceCrawler`
  overlap with Kid (`dx=0 dy=+1 @505`), but these are separate events: the
  crawler is down at `cell 36,37`, `block_destroy_effects=[]`, and the row-32
  block only bounces. Treat this as clean separation evidence, not the reported
  slime/metal-block destruction route.
  2026-06-14 route follow-up: a fresh phase-delay batch over
  `.cache/recordings/issue-260-drips-b-jump-climb-seed-current/scenario.json`
  found one stronger raw `EC2D` hitbox pass, but not a usable repro. Candidate
  `400..620 none,430..455 Right,456..515 B+Right,516..550 Left,551..620 Right`
  reaches cell `36,32` at healthy `dx=0 dy=-1 @465`, then dies into
  `LevelTransition`; its closest DripsSurfaceCrawler is still `dx=-78 dy=+1 @563`.
  The best live survivor,
  `430..620 none,470..530 B+Right,531..560 Left,561..620 Right`, verifies
  enemy-frontier clean through frames `430..620` but only reaches
  `dx=0 dy=-2 @493`, with the closest crawler still `dx=-52 dy=+1 @539` and
  DripsDrop `dx=+48 dy=-11 @409`. Do not repeat this phase-delay/late-lift axis
  unchanged; it is another oracle-comparable near-miss, not the slime/metal-block
  contact. A 2026-06-14 refinement on the same seed, clearing `360..620` and
  trying staggered `390..575` right/B/right/left/right phases, was worse than
  the known 1px row-32 miss: best candidate `390..430 B+Right,431..500 Right,
  501..545 B+Right,546..575 Left,576..620 Right` reaches cell `36,32` at only
  `dx=+10 dy=-12 @409`, exits through `LevelTransition`, leaves the closest
  DripsSurfaceCrawler `dx=-148 dy=+1 @494`, and closest DripsDrop
  `dx=+22 dy=+10 @409`. Do not repeat this staggered right/B/right phase axis
  without changing the bootstrap or target Drips cycle. Tooling follow-up added
  `probe-record-scenario-file-batch --target-projectile-cell KIND:COL,ROW`; a
  smoke batch on the current seed ranks DripsDrop vs cell `36,32` and shows the
  drop already reaches `dx=0 dy=-3 @381` independent of the late overlays, so
  the next useful search axis is Kid timing around that early drop/cell window,
  not more post-390 drop positioning. A 2026-06-14 pre-climb route-shape batch
  on the same seed found a stronger live near-contact prefix: clearing
  `390..440`, then `441..480 Right`, `481..510 B+Right`, `511..565 Left`,
  `566..700 none` keeps final mode `InGame` with HP 1 while preserving
  Kid/DripsDrop `dx=-3 dy=-2 @483` and Kid/cell `36,32` hitbox
  `dx=-1 dy=-3 @488`; sibling exits (`pre390_left520`, `pre390_shortjump`)
  have the same closest frames. This is still not reported slime/metal-block
  proof: Kid spends HP at frame 465 before the row-32 pass and no live
  crawler/block mutation is demonstrated. The materialised survivor bundle
  `.cache/recordings/issue-260-pre390-survivor-current/scenario.json`
  verifies enemy-frontier clean for frames `450..520`, so use it as the current
  refreshed comparable prefix; do not treat the `pre390_*` exit trims as a
  Drips/block patch lead without a follow-on route that actually mutates the
  metal block. Follow-up block-event scoring (`--target-block-event`) confirms
  this family only bounces the row-32 metal block: `pre390_l510` and sibling
  exits report `Bounce cell 37,32 @489`, but all tested `p370..p410`
  pre-climb/exit variants report `Destroy cell 37,32 <none>`. Do not mine the
  pre390 exit trims for the reported block break unless a new route shape
  produces an actual destroy event. A broader re-score of the current documented
  route families (`seed_survivor`, `phase_delay`, `pre390`, `earlyB`, `kH`,
  `p11`, `wait390`, `left400`) likewise finds no `Destroy` at cells `36,32` or
  `37,32`; only `earlyB` (`Bounce cell 37,32 @380`) and `pre390` bounce. A first
  bounded Kid-timing batch around that window found
  `300..450 none,330..370 Right,371..420 B+Right,421..450 Left`
  gets Kid's healthy hitbox to `dx=-5 dy=-3 @379` against cell `36,32`, but it
  is still not contact proof: focused probing reports closest Kid/DripsDrop
  separation `dx=-23 dy=-30 @381`, no DripsSurfaceCrawler in frames 330..520,
  and the run exits through `LevelTransition`. Do not repeat this `earlyB`
  timing unchanged; use it only as evidence that Kid/cell and drop/cell are
  near at adjacent frames but not mutually aligned. 2026-06-15 follow-up:
  materialised
  `.cache/recordings/issue-260-pre390-b480-520/scenario.json` from the current
  pre390 survivor with `480..520 B`. It is oracle-comparable for the relevant
  window (`verify-tas --frame-start 470 --frame-end 505 --checkpoint-interval 1
  --frontier-kind enemy --bisect` passes), and it keeps the good DripsDrop
  near-overlap (`dx=-3 dy=-2 @483`) plus a row-32 metal-block bounce
  (`Bounce cell 37,32 @489`). It is still not the reported slime/metal-block
  mutation: `Destroy` is absent for cells `36,32`/`37,32`, the best same-frame
  Kid/drop/cell conjunction remains 36 px off vertically, and the route ends
  with HP 0. Do not repeat the direct `480..520 B`/`B+Right`/Right/Left tail on
  this prefix as block-destruction proof; the next useful axis must move the
  active drop/crawler onto row 32 or delay the block contact without losing HP.
  A follow-up delay sweep found a cleaner survivor lead,
  `.cache/recordings/issue-260-pre390-br508-538/scenario.json`
  (`470..590 none`, `508..538 B+Right`), verified enemy-frontier clean for
  frames `500..530`. It improves the same-frame Kid/drop/cell conjunction to
  Kid `dx=+5 dy=-1`, DripsDrop `dx=+8 dy=-3` at frame 520 and ends InGame with
  HP 1, but it still has no Bounce/Destroy at `37,32`. Post-538 nudges
  (`Right`, `B`, `B+Right`, `Left`, `B+Left`, and right-then-B continuations)
  preserve the same frame-520 lead or regress into `LevelTransition`; none
  creates a block event. Do not repeat post-538 tail nudges on this branch
  unchanged; the next useful axis is a pre-508 approach change that keeps the
  frame-520 alignment but pushes Kid into the block.
  Follow-up pre-508 approach search found
  `.cache/recordings/issue-260-pre390-r490-br508/scenario.json`
  (`470..590 none`, `490..507 Right`, `508..538 B+Right`), verified
  enemy-frontier clean for frames `510..525`. It produces a clean row-32 bounce
  (`Bounce cell 37,32 @518`) with the best same-frame geometry so far: Kid is
  at `dx=0 dy=-1` against the cell while the DripsDrop is at `dx=+8 dy=-3`.
  It still has no `Destroy`, and the projectile is close to the block rather
  than occupying/mutating it. Recovery overlays after frame 519 can keep HP 1
  with neutral input, but `Left`/`Right`/`B+Left`/`B+Right` recoveries either
  keep the same no-destroy bounce or enter `LevelTransition`. Do not repeat
  post-bounce recovery trims on `r490_br508`; the next useful axis must alter
  the Drips cycle or the pre-490 approach so the projectile/crawler itself
  reaches the row-32 block at contact time.
  2026-06-16 broader pre-330 approach sweep on the same route materialised
  `e220_l280_r330` (`220..279 Left`, `280..329 Right`) as
  `.cache/debug/issue260-e220-l280-r330.json`: it remains enemy-frontier clean
  for frames `500..525` and hits `cell 37,32` at `dx=0 dy=-1 @517`, but the
  closest DripsDrop conjunction is still `dx=+8 dy=-11` from that cell while
  Kid contacts it (`manhattan=20 @517`) and no `Destroy` occurs. Treat early
  left/right pre-330 route shaping on this seed as exhausted unless it changes
  the Drips child spawn/phase, not just Kid's climb timing.
  2026-06-16 post-330 jump/right refinement on the same seed used
  `Drips.asm loc_3340E`/`loc_3345C` timing: the right parent is in
  `pre_drop` at frame 330 and the active DripsDrop reaches `cell 36,32` at
  `dx=0 dy=-3 @361`. Best materialised lead
  `.cache/debug/issue260-r330-br354-l394.json`
  (`330..353 Right`, `354..393 B+Right`, `394..433 Left`) is enemy-frontier
  clean for frames `350..375` and improves same-frame Kid/drop/cell geometry to
  Kid `dx=+1 dy=-1`, DripsDrop `dx=0 dy=-3` at frame 364 (`manhattan=5`).
  It still produces no `Bounce`/`Destroy` at `36,32`/`37,32` and later exits
  through `LevelTransition`, so treat this as an oracle-comparable early-overlap
  dead end, not the reported slime/metal-block mutation repro.
  Follow-up pre-490 approach sweep over `430..489` windows did not improve the
  proof. The DripsDrop still reaches row-32 vicinity early (`dx=+8 dy=-3 @455`)
  while the DripsSurfaceCrawler remains far below the block line (`dy=-65`),
  and the tested `450..470`/`460..480`/`470..490` right/left/B/B+Right/B+Left
  variants either lose the bounce or reproduce only no-destroy bounces at
  frames `517`, `523`, or `542`. Do not repeat these short pre-490 window
  overlays on the pre390 survivor; the next useful axis needs a route/bootstrap
  change that alters the Drips child cycle itself, not another local Kid timing
  trim.
  Follow-up tooling added
  batch `--target-projectile` scoring for closest healthy Kid/projectile
  proximity. Re-scoring the same early timing family shows `kH`
  (`300..350 B,351..390 B+Right,391..430 Left`) as the best branch, but it is
  still only `dx=-9 dy=-11 @409` from DripsDrop while the drop/cell best remains
  `dx=0 dy=-3 @381`; this is closer than `earlyB` but still not a contact or
  block-mutation repro. A cleaner-seed early row-32 rebound batch on
  `.cache/recordings/issue260-drips-drop-contact-probe-local/scenario.json`
  found `re_none330_r360_bl400` (`300..450 none`, `330..359 Right`,
  `360..399 B+Right`, `400..430 B+Left`, `431..450 Left`) and materialised it as
  `.cache/recordings/issue-260-early-row32-bounce-re-none330/scenario.json`. It
  bounces `cell 37,32 @379` while a `DripsDrop` is active, but still has no
  `Destroy`; the native drop remains in `cell 36,33`, and oracle comparison is
  already invalid because `verify-tas --frame-start 350 --frame-end 390
  --frontier-kind enemy --bisect` first diverges on HP at frame 374. `tas-diff
  --drips-trace --frame-start 368 --frame-end 380` shows oracle's right Drips
  child in `Drips.asm loc_33460` writing `$28` contact from frame 372 while
  native is pursuing the later Kid/block bounce. Do not repeat this
  pre-330/360-430 cleaner-seed rebound axis as block-destruction proof. A
  tighter local search found native-live `p11`
  (`300..355 B,356..390 Right,391..425 B+Left`) with HP 1, closest Kid/DripsDrop
  `dx=+1 dy=-1 @402`, and Kid/cell hitbox `dx=0 dy=-1 @400`, but it is still not
  valid proof: a materialised `.cache/debug/issue260-p11/scenario.json`
  `verify-tas --frame-start 390 --frame-end 430 --frontier-kind enemy --bisect`
  first diverges at frame 396 on HP (`native=2`, `oracle=1`). Treat `p11` as a
  useful native route-shape lead only; fix or avoid the pre-396 HP divergence
  before using it for slime/metal-block contact evidence. Follow-up `tas-diff
  --drips-trace --frame-start 390 --frame-end 405` shows the split is a Drips
  fall-phase mismatch before contact: oracle writes `colid_kidbelow` (`$28`) on
  the right falling child by frame 394 (`xy=584,564`, `vel=$2E000`) and HP drops
  at frame 395, while native still has the same child at `y=541`, `vel=$1C000`.
  Do not patch from this stale p11 trace as a generic falling-drop damage toggle;
  first align the `loc_3340E`/`loc_3345C` fall timing for layout #08 or choose a
  route that avoids that pre-contact phase drift. A later 2026-06-14
  left-wall-bounce route on
  `.cache/debug/issue260-phase-delay-br470-20260614/oracle-frame-scenario.json`
  did provide clean proof: native spent HP on a visible `DripsDrop` overlap at
  frame 478 while oracle stayed at HP 2, and `Drips.asm loc_3345C` only waits
  for floor collision before `loc_33488`/`loc_335D8` creates splash/crawler
  state. Native now keeps falling drops out of generic player-damage/FA75
  contact, and the materialised route verifies with no authoritative
  divergences through frames `450..520`. 2026-06-14 follow-up isolated the first
  timing split earlier: at p11 frame 174 native layout #08 splash reseeds with
  RNG byte `$2C` and previous-frame Kid X word `$02A0`, producing wait `$00AB`;
  oracle is already in `Drips.asm loc_33424` after the one-frame hibernate with
  `d0=$00A2` at `a0=$33522`, current `(Addr_GfxObject_Kid).x_pos=$02A1FA00`,
  and the last generated RNG ring byte `$0A`. Native/oracle RNG offsets both
  read `$FFF5`, so the split is the RNG ring contents before this reseed, not
  table order, falling-drop damage, or a universal Kid gfx X offset. Rejected
  patch: using the existing initial-seed `drips_kid_gfx_x_pos_loc_3340e` (+16px)
  for splash reseeds worsens p11 (#08 wait `$003D` vs oracle `$0024` at frame
  300); the correct fix must find the pre-174 RNG-ring content drift before
  patching Drips timing. Follow-up typed `--rng-trace` byte columns show this
  ring-content drift already exists at p11 scenario frame 0 (`native_rng=$FFCF`
  byte `$31`, oracle byte `$09`, with matching cursor), so inspect the settled
  Level 1 warp/bootstrap RNG ring state before touching Drips again. Oracle
  settle tracing for the same p11 scenario ends `target_live` step 59 at
  `rng_offset=$FFCB`, `rng_last_byte=$32`, and scenario frame 0 advances to
  `$FFCF/$09`; native's settled-warp ring must be aligned to that hidden state
  before Drips timing can be trusted. 2026-06-14 native now injects that Level 1
  recording-settle ring: `tas-diff --rng-trace --frame-start 0 --frame-end 5`
  matches oracle bytes through frame 4 (`$FFCF/$09`, `$FFD3/$00`, `$FFD7/$7F`,
  `$FFDB/$E8`, `$FFDF/$92`). The p11 HP divergence remains at frame 396, and
  `--drips-trace --frame-start 173 --frame-end 175` still shows a frame-174
  Drips phase split (`native_rng=$FFF4`, `oracle_rng=$FFF5`), so continue from
  Drips.asm `loc_33424`/`loc_3345C` timing rather than the settled RNG ring.
  2026-06-14 follow-up: `GfxObject_Animate` (`kid.asm loc_1158/sub_1194`) makes
  Drips.asm `stru_33F90`'s raw `$A,$A,8,6,1` script effectively 31 frames, not
  35, because entry-load frames consume one tick and the duration-1 final entry
  finishes as soon as the terminator is visible. With that native timing,
  `tas-diff --drips-trace --frame-start 145 --frame-end 178` lines up the #08
  fall/splash reseed (`rng_last=$0A` at frame 174), but `verify-tas
  --frame-start 390 --frame-end 430 --frontier-kind enemy --bisect` still
  first diverges at frame 396 on HP. The remaining p11 split is now the
  projectile/player damage path: oracle writes `colid_kidbelow` (`$28`) on the
  falling Drips child at frames 394-396 while native keeps HP at 2. 2026-06-14
  follow-up fixed that split: Drips.asm `loc_3345C` leaves the falling child
  alive until floor `$0C`, but Kid's `Character_CheckCollision` `off_B640`
  still routes the same `$28` through `Check_for_recent_damge`; native now
  lets `DripsDrop` hurt Kid without consuming the child. `verify-tas` passes
  p11 frames `390..430`, and `tas-diff --drips-trace --frame-start 388
  --frame-end 402` confirms the oracle `$28` window on frames 394-399. A
  refreshed p11 route probe now verifies enemy-frontier clean through frames
  `360..430`; native p11 remains route-shape evidence only, not the human
  slime/metal-block proof. `probe-record-scenario-file --target-cell-hitbox
  36,32 --target-projectile drips-drop` reports Kid within `dx=0 dy=-1` of the
  cell at frame 400 and `dx=+5 dy=-1` from the DripsDrop at frame 395, but the
  run later enters death/`LevelTransition`. Post-contact tail variants can keep
  the route alive (`426..700 Left` ends `InGame` with HP 1), but that survivor
  retreats far left and leaves the nearest DripsSurfaceCrawler `dx=+208 dy=+1`.
  Early handoff batches around `356..425` also failed to improve the proof:
  the best direct drop branches either keep p11's `dx=+5 dy=-1` or trade it for
  `dx=+2 dy=-11`, and all still end in `LevelTransition`. Do not repeat this
  p11 early-handoff/post-contact tail axis unchanged; the next useful #260
  search needs a different route shape that aligns Kid, the live Drips child,
  and the row-32 metal block in the same frame window.
  2026-06-14 route follow-up: a pre-approach timing batch on `.cache/debug/issue260-p11/scenario.json`
  found `250..290 Left,291..330 B+Right` as a stronger native route shape:
  Kid reaches DripsDrop `dx=+1 dy=-1 @395` and cell `37,32` `dx=+1 dy=-1 @400`.
  Adding `426..700 Right` materialises `.cache/recordings/issue-260-slbr-right426/scenario.json`
  and produces `Bounce cell 37,32 @501`, but this is not oracle-comparable:
  `verify-tas --frame-start 390 --frame-end 520 --frontier-kind enemy --bisect`
  first diverges at frame 393 on HP (`native=2`, `oracle=1`). `tas-diff
  --drips-trace --frame-start 388 --frame-end 402` shows oracle's right falling
  Drips child writing `$28` contact at frames 392..399 before the native bounce
  route can prove anything. Treat this as a phase-shifted native near-miss, not
  metal-block proof; do not repeat this `slbr` pre-approach plus right-recovery
  family unchanged. Follow-up route proof: `mb_left_long` on the same p11 base
  (`150..230 Left`, `231..320 B+Right`, `321..390 Right`, `391..425 B+Left`)
  materialises as `.cache/recordings/issue-260-p11-mb-left-long/scenario.json`
  and is oracle-comparable for the block window:
  `verify-tas --frame-start 450 --frame-end 505 --frontier-kind enemy --bisect`
  passes. It produces native/oracle-matched `Bounce cell 37,32 @490`, but no
  `Destroy`, no active projectile in the bounce frame, and Drips layout #08 is
  still `pre_drop:0x800a` with no child; the nearest enemy support/contact probe
  is separated from Kid. Treat this as clean Kid/block-bounce evidence only,
  not the reported slime/block destruction route. Delaying the same shape with
  `426..500 none` materialises
  `.cache/recordings/issue-260-p11-mb-hold-none/scenario.json` and also verifies
  clean for frames `490..520`; it shifts the bounce to `cell 37,32 @511` while a
  falling `DripsDrop` is active, but that drop remains in cell `36,33`, no
  `Destroy` occurs, and the enemy contact/support probes are still separated.
  Do not repeat this delayed p11 bounce axis as a block-destruction lead unless
  a new route moves the active drop/crawler onto the row-32 block itself.
  A
  follow-up over the clean
  `.cache/debug/issue260-tail-grid/l524_b530_540.json` tail found `480..540 B`
  gets Kid's healthy hitbox within 1 px of row-32 cells `32,32`/`33,32` at
  frame 490, but still leaves the DripsSurfaceCrawler `dx=+102 dy=+1 @512`;
  keep searching for a route that brings the live child/projectile to the
  metal-block line instead of repeating Kid-only lift-window trims.
  2026-06-12 current-route approach-offset batch around frames `430..477`
  found no usable centre-`EC2D` contact. The most interesting right-side
  candidate, `approach_br448_465`, reaches `(584,543)` raw `dx=-10 dy=-1`
  by frame 559 but takes native DripsSurfaceCrawler damage at frame 549
  (`kid_projectile_damage_probe=hit:DripsSurfaceCrawler@0x024a2000,0x02500000`)
  and is not oracle-comparable: `verify-tas --frame-start 500 --frame-end 559`
  first diverges at frame 500 on HP (`native=1`, `oracle=0`). Treat this as
  another phase-shifted route rejection, not a Drips hitbox or block-break lead.
  2026-06-12 follow-up: pre-damage clears on the same
  `.cache/debug/issue260-approach-br448-465.json` candidate do not produce a
  usable route. Clearing `524..528`, `525..528`, `520..528`, or `512..528`
  keeps Kid healthy but collapses back to the old `dx=+1 dy=-20 @473` closest
  approach; clearing only `528` or `526..528` preserves the raw late approach
  but still enters death windup by frame 559. Do not repeat this pre-damage
  left-window-clear threshold axis without changing the earlier route shape.
  A post-`513b36d1` hybrid route batch over the named drop-contact probe also
  failed to find a live right-side contact: clearing `300..559`, applying an
  early `380..388/396 B+Right` pulse plus a later `424..504 B+Right` pulse,
  and trying left/right/no recovery ranked only `LevelTransition` candidates.
  The best right-target score was `(584,543)` healthy `dx=+15 dy=-26 @431`,
  worse than the existing `wait390`/`left400` near-misses; do not repeat this
  two-pulse hybrid axis unchanged. A later post-contact climb probe on
  `.cache/debug/issue260-route-probes/drop-contact-br460-492-l493.json`
  (`540..600 B`, with nearby `520..620` climb variants) improved native raw
  proximity to `(584,543)` at `dx=-2 dy=-1 @569`, but it is not usable evidence:
  closest healthy was only `dx=+1 dy=-20 @485`, and enemy-frontier oracle bisect
  on derived cache scenario
  `.cache/debug/issue260-route-probes/drop-contact-br460-492-l493-b540-600.json`
  first diverges at frame 560 on HP (`native=0`, `oracle=1`) after native-only
  DripsSurfaceCrawler damage. Do not repeat post-contact B-climb trimming as
  metal-block proof without a route-shape change. A follow-up avoid-then-climb
  family around the same damage window (`548..562`, `548..566`, `552..566`
  right/B+right/left avoidance, then `B`/`B+Left`/`B+Right` recovery through
  620) also ranked worse than the base route: best healthy right-target approach
  stayed `dx=-20 dy=-48 @549`. Do not repeat small avoid-window trims unless
  the pre-548 route shape changes. A pre-548 setup batch found `480..520 B+Left`
  can align native horizontally with `(584,543)` at `dx=0 dy=-48 @528`, but
  derived scenario
  `.cache/debug/issue260-route-probes/drop-contact-br460-492-l493-bl480-520.json`
  first diverges against oracle at frame 519 on HP (`native=1`, `oracle=0`);
  `tas-diff` shows oracle has already taken DripsSurfaceCrawler damage while
  native has not, so it is another phase-shifted setup rather than metal-block
  proof. The immediate `530..600` B/B+Left/B+Right climb continuations also do
  not improve the healthy score. Do not reuse this `480..520 B+Left` setup seed
  as proof. 2026-06-12 Iron Knight route-shape check: refreshed
  `issue260-drips-iron-knight-climb-probe`, extended it cache-locally to 700
  frames, and probed post-536 wall-climb overlays. Best new left-target
  approach was `500..535 Left` then `536..700 B+Left`, reaching `(456,543)`
  healthy `dx=-1 dy=-23 @552`; `verify-tas --scenario
  .cache/debug/issue260-iron-extended/ik-l500-bl536.json --frame-start 500
  --frame-end 560 --checkpoint-interval 1 --frontier-kind enemy --bisect`
  passed with no authoritative divergences. This is useful oracle-comparable
  route evidence, but still not row-32 slime/metal-block contact; do not repeat
  Iron Knight late B+Left wall-climb trimming unless the pre-500 route shape
  changes. 2026-06-12 follow-up: a small pre-500 replacement batch over
  `.cache/debug/issue260-iron-extended/ik-l500-bl536.json` changed the setup
  before the late Iron Knight wall-climb tail (`300..499` left-return, high-arc,
  brake-drop, and slingback variants). Best candidate was `pre300_high_arc`,
  still final `InGame` with `lives/hp 3/3`, but only reached `(456,543)` at
  healthy `dx=0 dy=-17 @546`; right target stayed at the old `dy=-48` class.
  This does not improve on the existing comparable near-miss and should not be
  mined as Drips/block proof. A fresh post-300 right-run/brake axis over
  `issue-260-drips-drop-contact-probe-current/scenario.json` (`300..400/430/450`
  clears, `400..520` right windows, `500..555 B+Right`, and early left brakes)
  also failed: the best raw right-target proximity was `dx=+3 dy=-8` around
  frame 456, but Kid was already unhealthy or fell into LevelTransition; every
  healthy closest stayed at the old early `dx=0 dy=-48 @128`. Do not retry this
  post-300 right-run/brake family without a different vertical setup. A 2026-06-11
  early reverse/jump sweep over `160..320` also only reproduced the known
  `(456,543)` `dx=0 dy=-16` near-miss and was not oracle-comparable at the late
  window (`number_hitpoints` native 2 vs oracle 1 at frame 470); do not retry
  that reverse-entry family unchanged. On the comparable left-tail seed
  `.cache/debug/issue260-tail-grid/l524_b530_540.json`, a 360..460 middle-setup
  batch (`B`, `B+Left`, `B+Right`, `Left`, `Right` over `360..420`,
  `380..440`, `400..460`) also failed: B-only variants remain the old clean
  `(456,543) dx=0 dy=-16 @540`, left/B+Left worsen vertical approach, and
  right/B+Right variants improve only invalid LevelTransition routes. Do not
  repeat this middle-setup axis unchanged. 2026-06-11 follow-up: three more
  route-shape batches failed to improve the current near-misses. On
  `.cache/debug/issue260-route-probes/drop-contact-br460-492-l493.json`, early
  420..520 brake/coast/B+Left/B+Right overlays over the right-side route still
  leave the best healthy `(584,543)` approach at `dx=+1 dy=-20`, and the best
  left-target approach only `dx=-1 dy=-20`. A follow-up
  `320..390 Left` / `390..460 B+Right` reverse-entry axis on the named
  drop-contact route still produced only the old left-side `dy=-16` near-miss
  and diverged against oracle at frame 470 on diamonds; do not repeat that
  axis unchanged. On
  `.cache/debug/issue260-tail-grid/l524_b530_540.json`, 430..520 B-pulses,
  left/right shaping, and B+direction variants preserve or worsen the existing
  `(456,543) dx=0 dy=-16 @540` clean seed. A final underside-target batch
  scored `(456,527)` directly with 500..550 B/coast/left/right variants; every
  healthy result was worse (`best dx=+2 dy=-32` to underside, `dx=+2 dy=-16` to
  row-32). Do not repeat these 420..550 shaping axes without a different
  bootstrap/route family. 2026-06-11 follow-up after #253 queue correction:
  a cleared 220..460 / 240..420 / 260..430 mid-route replacement batch
  (`pre240_clear_*`, `pre260_*`, `pre300_*`, `pre220_*`) also failed to improve
  the current seed. Every candidate either entered `LevelTransition` or stayed
  below the targets; best healthy right-target result was still `dy=-33..-48`,
  and best left-target result was worse than the existing clean `dy=-16` class.
  Do not retry this broad 220..460 mid-route replacement axis unless the
  bootstrap/route family changes. 2026-06-12 follow-up: early-vertical route
  batches over `.cache/debug/issue260-tail-grid/l524_b530_540.json` also failed
  to improve the seed. Varying the first jump (`55..145`, `70..135`, split
  holds), early left-air setup, and early right recovery (`190..360`) either
  reproduced `(456,543) dx=0 dy=-16 @540`, worsened to `dy=-48`, or made a
  different-but-useless left-air branch at `dx=+79 dy=-1 @548`; a targeted
  `548..620 B+Right` recovery still only found an early `dx=+2 dy=-17 @66`.
  Do not retry this early-jump/left-air/right-recovery family unchanged.
  Another 2026-06-12 alternate-bootstrap batch over the named contact probe
  cleared `0..239`/`0..299`, rebuilt early low/high arcs, left-air recovery,
  slowdown, double-jump, and right-sling shapes, then restored the known
  `478..528 Left`, `530..559 B`, `536..559 Right` tail. Scoring after frame
  300 still failed: best live result was `early_left_air_recover` at
  `(456,543) dx=-19 dy=-16 @539`, while row-aligned variants stayed far left
  (`dx=+78 dy=0 @546`) and right-biased variants transitioned. Do not repeat
  this alternate `0..299` bootstrap rebuild unless the base route changes.
  A narrower 2026-06-12 Iron Knight continuation check from the named
  `issue-260-drips-iron-knight-climb-probe-current` seed also failed: post-536
  B+Left/B+Right holds through 620/660/700/760 and directional recoveries to
  820 ranked no better than `(456,543) dx=-5 dy=-39` and `(584,543)
  dx=+55 dy=-16`, so the direct Iron Knight post-tail route moves away from
  centre `EC2D` contact. Do not repeat this post-536 Iron Knight continuation
  axis without changing the pre-536 route shape. A distinct pre-478 slingback
  family over the named contact probe (`0..160 Right+B`, `161..220 Left`,
  `221..420 Right+B`, `421..477 Left`) also failed after tail refinement:
  best live result was `sling_tail_right_early` at `(456,543) dx=-1 dy=-17`
  and `(584,543) dx=0 dy=-48`, with a focused probe staying `InGame`/HP 1 and
  showing only the earlier floor `DripsSurfaceCrawler` window, not centre-cell
  block contact. Do not repeat this pre-478 slingback family unchanged.
  2026-06-12 follow-up: two fresh route-shape batches also failed to improve
  the centre `EC2D` target. On oracle-comparable
  `.cache/debug/issue260-iron-extended/ik-l500-bl536.json`, first-jump,
  early-left, slingback, reverse/coast, and `260..430` up/down/B variants still
  ranked no better than the existing `dy=-23` left-target class or
  right-target `dx=-2 dy=-20`; every row-aligned right-side improvement stayed
  below the row-32 line. On the current
  `.cache/recordings/issue-260-drips-b-jump-climb-seed-current/scenario.json`,
  pre-360 lift/drop, double-rise, late-switch, and `430..580` lift variants
  left the best live/nonzero-HP candidate at `(456,543) dx=+6 dy=-19 @488`,
  worse than the baseline, while closer right-side passes were HP-zero or
  `LevelTransition`. Do not repeat these first-jump/early-sling/pre-360
  vertical setup probes unless the base route changes. 2026-06-13 follow-up on
  the same current b-jump climb seed: `probe-record-scenario-file-batch` found
  a small improvement by holding `170..260 B+Left`, reaching `(584,543)`
  healthy `dx=-4 dy=-17 @513` (manhattan 21, better than the old 23px miss).
  Focused `probe-record-scenario-file --input 170-260:b+left --target-projectile
  drips-surface-crawler` shows the crawler window is still at y `$0250` and the
  closest Kid/crawler separation is `dx=+13 dy=+21 @509`; Kid reaches HP 0 by
  the tail. A follow-up continuation sweep from that brake shape found
  `500..620 Left`/`B+Left` improves the healthy target score to `(584,543)`
  `dx=+1 dy=-18 @515` with HP 2, but focused probing still leaves crawler
  separation at `dx=-5 dy=+21 @521`; tighter `480..620`/`490..590`/`510..610`
  left/B+Left windows either keep the same manhattan-19 miss or collapse back
  to the old floor pass. Treat this as another route-shape near-miss, not
  metal-block proof, and do not repeat the `170..260 B+Left` plus `500..620
  Left/B+Left` brake axis unchanged. 2026-06-13 follow-up: `wildside-cli` now accepts
  `--target-cell col,row` for single and batch probe scoring, so #260 route
  search can rank actual 16x16 centre-`EC2D` block-cell contact instead of a
  guessed point. On `issue_260_drips_drop_contact_probe_01`, scoped to
  `--frame-start 520`, the current route remains 32px below left cell `28,32`
  and 157px away from right cell `36,32`; a small baseline/late-B/late-BR batch
  all tied at the same 32px left-cell miss. Do not re-run those unchanged late
  variants as metal-block proof. 2026-06-13 follow-up: `wildside-cli` also accepts
  `--target-cell-hitbox col,row`, which scores Kid's ASM `loc_902A`/`unk_B408`
  upper/body bounds against a 16x16 cell. On
  `issue-260-drips-b-jump-climb-seed-current`, the baseline stays alive and
  reaches right cell `36,32` at hitbox `dx=+1 dy=-2` (manhattan 3) at frame
  503; `early_br440_540`, `early_br450_550`, and `neutral430_jump470` improve
  only to manhattan 2 and all end HP 0. Treat the seed as a confirmed
  near-contact route, not metal-block proof, and do not reject it based on
  origin-only cell scoring. A follow-up late-climb reshape that clears
  `430..620`, applies `460..520 B+Right`, exits `521..560 Left`, then restores
  `561..620 Right` stays alive at HP 1, improves the right `EC2D` cell hitbox
  score to `dx=0 dy=-2 @483`, and verifies oracle-clean on the materialised
  cache scenario through frames 430..620. It is the best current comparable
  near-contact route, but focused probing still leaves the DripsSurfaceCrawler
  far away (`dx=-58 dy=+1 @575`), so it is not the reported slime/metal-block
  contact. A fresh `floor_sling` family on
  `issue-260-drips-drop-contact-probe-current` changed the bootstrap (`220..300
  B+Left`, `301..390 B+Right`, `391..462 Left`, `463..512 Right`) and found
  a stronger native-live right-side near-contact: `513..559 B+Right` reaches
  cell `36,32` at Kid-hitbox `dx=0 dy=-1 @522` while staying `InGame`/HP 1.
  This is still not slime/metal-block proof: the closest DripsDrop separation
  is `dx=-5 dy=-36 @529`, and the only DripsSurfaceCrawler frame is at 559
  with `dx=-67 dy=+1`. Do not repeat this `floor_sling` hand-off family
  unchanged unless a new bootstrap changes the Drips/Kid phase.
  2026-06-14 follow-up route rejection:
  `.cache/debug/issue260-prebl-r330-bl390-20260614/oracle/oracle-frame-scenario.json`
  (`240..470 none,250..320 B+Left,321..370 Right,371..430 B+Left,
  470..530 B+Right,531..560 Left,561..620 Right`) verifies enemy-frontier
  clean through frames `300..680`, and advisory `tas-diff` frames `640..680`
  reports 40/40 matches. It is a useful different route shape but still not
  the reported slime/metal-block proof: Kid reaches cell `28,32` at healthy
  `dx=-4 dy=-1 @380`, DripsDrop is only `dx=+8 dy=-11 @401`, and the later
  DripsSurfaceCrawler reaches Kid at `dx=+1 dy=+1 @665` on the lower floor
  while remaining `dy=-65` from row-32 cells. Do not repeat this
  pre-250/B+Left sling plus late B+Right/Left/Right tail as metal-block proof
  without changing the Drips cycle or vertical route.
  Follow-up survivor seed:
  `.cache/debug/issue260-prebl-longr-left-right-tail-20260614/oracle/oracle-frame-scenario.json`
  (`240..470 none,250..320 B+Left,321..390 Right,391..430 B+Left,
  470..530 B+Right,500..530 Left,531..700 Right`) verifies enemy-frontier clean
  through frames `450..700`, remains InGame with HP 1, and improves the live
  close-window evidence: DripsDrop separation is `dx=+1 dy=+2 @475` while Kid
  reaches cell `28,32` at healthy `dx=0 dy=-1 @480`. This is still not the
  reported slime/metal-block proof because the crawler only appears at frame
  700 on the lower floor, `dx=-55 dy=+1` from Kid and still vertically separated
  from row-32 cells. Use this as the current clean alive route seed; the next
  useful search needs to alter the Drips cycle or vertical route so the child
  reaches the row-32 block line, not just keep trimming the post-500 recovery.
  Drips-trace clarification: on this seed `tas-diff --drips-trace` frames
  `468..482` shows #07 falling through cells `28,34`/`28,35`/`28,36`, then
  splashing at frame 477 and reseeding at frame 478; the row-32 score is only
  a Kid/cell proximity metric, not a `$0C` splash on the reported block line.
  2026-06-14 follow-up on the same clean alive seed: a pre-splash/cycle batch
  over frames `360..470` (`B`, `B+Left`, `B+Right`, `Left`, `Right`, and
  split `B` then direction variants) still produced no row-32 block event.
  Baseline and right-biased survivors remain best for Kid/cell at
  `dx=0 dy=-1`, but DripsDrop still only reaches cell `28,32` at `dy=-3 @373`;
  the closest useful survivor crawler remains on the lower floor (`dy=-65`
  from cell `28,32`). Do not repeat this `prebl-longr` `360..470`
  pre-splash axis unchanged.
  A mid-bootstrap batch on the same clean alive seed over `120..260` also
  failed to create metal-block evidence. Candidate `mid_b140_240` was
  materialised as `.cache/recordings/issue-260-prebl-longr-mid-b140-240/scenario.json`
  and verifies clean for the block and crawler windows (`verify-tas` frames
  `450..500` and `620..670`, enemy frontier). It keeps the route alive at HP 1,
  reaches row-32 cells `28,32`/`29,32` at hitbox `dx=0 dy=-1 @480`, and bounces
  `cell 29,32 @480`, but `Destroy` is absent, the closest DripsDrop is still
  `dx=-17 dy=-11 @432`, and the crawler only reaches Kid later on the lower
  floor (`dx=+16 dy=+1 @649`, still vertically separated from row-32 cells).
  Do not repeat this `prebl-longr` `120..260` mid-bootstrap axis unchanged.
  2026-06-14 early-bootstrap check on the same seed (`0..120` / split
  `0..70,71..140` with `None`, `Right`, `B+Right`, `Left`, `B+Left`, `B`)
  also failed to create metal-block evidence. `early_none` and `early_r`
  reproduce a Kid/cell `dx=0 dy=-1` near-miss, but the Drips child is either
  shifted to the right-side column (`DripsDrop cell 28,32 dx=-121`) or remains
  the baseline lower-line splash (`dy=-3`). Split variants can bring a crawler
  close to Kid on the lower floor (`split_l_r` crawler `dx=0 dy=+1 @669`) but
  still leave it `dy=-65` from row-32 cells. Do not repeat this `prebl-longr`
  early-bootstrap cycle axis unchanged.
  A cache-local form-axis check on the same seed also failed to produce a
  metal-block proof. Red Stealth (`playtest_form=3`), Juggernaut (`5`), and
  Iron Knight (`6`) variants all stayed InGame, but none produced row-32
  `Destroy`/`Bounce` on cells `28,32`/`29,32` from a live Drips child.
  Juggernaut changed Kid's vertical route enough to keep healthy row-32 hitbox
  proximity at `dx=0 dy=-1 @405..407`, but the closest DripsDrop was still far
  from Kid/cell (`dx=-34 dy=-11 @178`; cell `28,32` `dx=-121 dy=-3 @332`)
  and the crawler still emerged on the lower floor (`dx=+16 dy=+1 @662` from
  Kid, `dy=-65` from row-32 cells). A bounded Juggernaut `300..430` timing
  batch preserved the same separation and reported no target block events, so
  do not repeat simple form swaps on this seed without a new Drips cycle or
  vertical route.
  2026-06-15 follow-up on the independent
  `issue_260_dlb_tail_b_row32_bounce_01` route: the new named
  `issue260-gr-pulse600` scenario overlays `431..500 Right` plus B pulses at
  `600..610`, `620..630`, and `640..650`. The materialised cache candidate
  verified enemy-frontier clean through frames `580..660`, and advisory
  `tas-diff` frames `636..652` reported 16/16 matches. It improves late
  DripsDrop/row-32 conjunction around frame 650 (`cell 36,32` Kid
  `dx=-7 dy=-1`, drop `dx=0 dy=-9`; `cell 37,32` Kid `dx=0 dy=-1`, drop
  `dx=+8 dy=-9`) and keeps the row-32 bounce at `cell 37,32 @610`, but it is
  still not the reported mutation: no destroy event occurs, and focused traces
  show the projectile separated from Kid/block contact
  (`kid_projectile_damage_probe dx=-19 dy=-23 overlap=0`). Use it as a durable
  oracle-clean near-miss seed; do not treat it as metal-block proof. Follow-up
  same-seed batches reject the obvious late-tail/setup variants: `520..680`
  left/right/B pulses can improve the drop-to-cell Y to `dy=-3`, but they pull
  Kid down/off line (`best pulse_early score=18`, Kid `dy=-14`) and still only
  bounce `cell 37,32 @590`; `431..500` setup/turn variants either reproduce
  the baseline (`late_right`, score 17, bounce `cell 37,32 @610`) or move Kid
  far away/enter `LevelTransition`. Do not retry this gr-pulse setup/tail axis
  without changing the Drips cycle or vertical route. 2026-06-15 follow-up:
  an earlier pre-431 route-shape batch over the same `issue260-gr-pulse600`
  seed also failed. Replacing `300..430` or `330..430` with `none`, `B`,
  `Right`, `B+Right`, `Left`, `B+Left`, and short split direction/jump windows
  either reproduced the baseline exactly (`pre300_b`/`pre300_br`/`pre300_r`:
  cell `37,32` Kid `dx=0 dy=-1 @610`, DripsDrop still `dy=-9 @650`, final
  HP 0) or kept Kid alive only by moving him far left of the target
  (`split_l_br` best alive score 107). This confirms the current near-miss is
  not sensitive to simple pre-431 input replacement; next useful work needs a
  different bootstrap/Drips cycle or new live recording, not more pre-300/pre-330
  trims on `issue260-gr-pulse600`.
  2026-06-16 engine-parity follow-up: after aligning Drips splash hibernate,
  `loc_3340E` Kid-gfx-x reseed timing, Drips crawler RNG order, RockTank
  mouth-child contact damage, and death-tumble movement trace, the materialised
  `issue-260-gr-pulse600-current` route verifies enemy-frontier clean from
  frames `1..700`. It is still only an oracle-clean near-miss: keep #260 open
  until a route proves the reported live slime/metal-block mutation.
  2026-06-15 follow-up on `.cache/recordings/issue-260-pre390-survivor-current/scenario.json`
  rejects the earlier B-window route-shape axis too. Broad `260..360` input
  replacements showed `pre260_b` can put Kid's healthy hitbox within 1 px of
  row-32 cell `36,32`, but it produces no row-32 Bounce/Destroy and leaves the
  closest DripsSurfaceCrawler 65 px below the target row. A tighter B-window
  sweep (`220..320` through `300..400`) repeats the same pattern: best
  `b220_320`/`b220_340`/`b220_360` put Kid at `dx=0 dy=-1 @489`, while the
  DripsDrop conjunction is still separated vertically (`projectile dy=-53`) and
  no row-32 block event occurs; later B windows only reproduce the old cell
  `37,32` bounce with the Drips child far below. Do not repeat simple
  pre-260/pre-300 B-window overlays on this seed; the next useful #260 route
  search still needs a different bootstrap/Drips cycle or a new live recording
  that brings the child onto the row-32 cell before Kid's block contact.
  2026-06-16 follow-up: a bounded mid-bootstrap batch on
  `.cache/recordings/issue-260-gr-pulse600-current/scenario.json` varied
  `120..430` windows and scored same-frame Kid hitbox plus DripsDrop/crawler
  proximity to cells `36,32`/`37,32`. Best `lowarc_b150` only reproduces the
  documented vertical split (Kid `dy=-33`, DripsDrop `dy=-3`, no row-32
  Bounce/Destroy), while bounce-preserving candidates still keep the projectile
  separated from the row-32 cells. Do not retry this `120..430` mid-bootstrap
  replacement axis on the `gr-pulse600` seed unchanged.
  #270 current wall-pulse probes split the evidence:
  Kid's natural BLW2 left-wall route with B held at `285..330` is gameplay CRC-clean through frame
  380, so Kid upward wall movement with B is ROM-faithful; direct Samurai `playtest_form=3` variants
  diverge before the B pulse and need a natural pickup/well route before any movement patch. #285 crawler art);
  2026-06-11 follow-up: cache-local natural Red Stealth route
  `.cache/debug/issue270-wall-launch/red_stealth_left_wall.json` is a better seed than the
  direct `playtest_form=3` probes. Before commit `1c06f5d8`, it blocked before proving the wall launch:
  `verify-tas --frame-start 0 --frame-end 520 --checkpoint-interval 1 --frontier-kind gameplay`
  first diverged at frame 266. `tas-diff` frames 250..272 showed oracle already in the
  Red Stealth wall/jump path (`kid.asm loc_A6F8 -> loc_A74E -> sub_A4EE -> loc_A75A`,
  trace code `0x00A71C`) with leftward x drift (`kid_x_pos 0x00660000`, then
  `kid_x_vel 0xffffc000`), while native stays pinned at `0x00670000`/`x_vel=0` with
  `kid_wall_cling_release_probe branch=sub_ADB8 sub_ADB8=column_shape_blocked`. Fixed candidate
  `1c06f5d8` allows the sampled `loc_AB0A` broad-probe release on either wall; the natural route now
  passes gameplay CRC through frame 520. Treat #270 as proposed fixed pending human validation of
  the original well-hug launch.
  #277 left-jump input drop reconfirmed after the loc_7464 candidate fix; the
  existing natural BLW2 route is not the repro because its frame-207 L+B jump
  stays CRC-clean and honours held-left through loc_7664 -> loc_A426 -> sub_B084;
  a later two-pulse variant (`205..245` and `270..300` Left+B) is also not the
  repro: frame 272 takes the jump edge, the following airborne frames keep
  negative held-left x velocity, and frames 260..310 pass gameplay CRC. A
  2026-06-12 later lower-left edge scan over the same clean fixture (`300..330`,
  `320..350`, `340..370`, `360..390`, `400..430`, `500..540`, and short
  `310`/`330`/`360` Left+B pulses) is also not the repro: frame 302 consumes
  Left+B as `jump_pressed`, then the level-edge clamp zeros x velocity only
  after several held-left airborne frames. Next pass needs a direct
  human-equivalent lower left-wall/stair jump repro. A 2026-06-12 `LEFT+C`
  variant of the original BLW2 route is invalid too: Kid is already HP 0/death
  windup at the wall-hold checkpoint and enters `LevelTransition`, so it never
  exercises the reported wall-jump input path. A 2026-06-11 lower-left-wall
  scratch route (`lower-left-wall-b-current`) is also not the repro: native
  keeps the held-left wall/jump path, while the
  first oracle split is a Drips controller/object-count mismatch at frame 103;
  a 2026-06-13 current-HEAD stair-route audit rejects
  `.cache/recordings/issue277-blw2-stair-jump-right-current/scenario.json`
  because it passes frames 0..180, and rejects the paired left route as an
  input signal because the first authoritative split is native
  ShooterBlock/stomp-bounce handling at frame 88/90, before it can prove a
  dropped-left jump. Fixed candidate `de869f33` closes the mirrored native
  floor-contact gap found during the latest playtest follow-up: ASM
  `sub_902A -> loc_A984 -> loc_A99E` also promotes leftward leading-edge
  SlopeDown recontact in the same frame, while native's row-boundary guard only
  allowed the rightward mirror. Regression
  `level00_airborne_leading_edge_slope_down_left_recontact_enters_loc_9d22_same_frame`
  now lands the leftward SlopeDown case without weakening the existing
  trailing-edge guard. 2026-06-14 rerun keeps that conclusion: the focused
  held-left input and leftward SlopeDown regressions pass; the paired direct
  stair routes split exactly as before. Right route
  `.cache/recordings/issue277-blw2-stair-jump-right-current/scenario.json`
  verifies frames `0..180` with no authoritative divergence, while left route
  `.cache/recordings/issue277-blw2-stair-jump-left-current/scenario.json`
  first diverges at frame 90 on `number_objects` plus camera after native
  ShooterBlock damage; `tas-diff` frames `86..94` shows frames 86..89 as
  non-authoritative object-count oracle limitations before the damage path.
  Keep #277 open for a fresh human-equivalent stair/wall route, not another
  patch from this blocked direct left-stair fixture.
  #291 stale #249 gate assertion re-derived (tail sample is airborne after walking off support).
- **Gates**: verify-slice-fast green. Full published TAS
  `verify-tas --scenario tests/data/tas/any-percent-published/scenario.json
  --frame-start 0 --checkpoint-interval 1 --bisect --frontier-kind gameplay`
  passes all 37759 frames by CRC.
