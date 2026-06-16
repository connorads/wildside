# Backlog validation & planning report — 2026-06-14

**Method:** 12-cluster multi-agent workflow (54 agents) over all **93 live backlog
issues** (`[ ]` open, `[~]` in-progress, `[?]` proposed-fix). Each cluster validated
every member against current code/ASM/tests; every suspicious `[?]` verdict was then
re-checked by an independent adversarial skeptic; a planner produced one ASM-grounded
next step per cluster. Headline findings were spot-verified against HEAD by hand.

**Snapshot:** validated around `ca222a3b`..`80c1793f`. A concurrent loop session was
committing throughout (FireDemon #183, Cyclone #274 regression), so a few rows below
may have already moved — re-check the row before acting. The green gate
(`verify-slice-fast`: 5941 kid-core + 9 kidcli + TAS frontier) was passing.

**Scope of this report:** the *agent-doable* validation layer — does the claimed fix
actually exist in current code (cited ASM label present? cited regression test present?
evidence coherent / stale?). It does **not** replace the human-playtest layer that owns
`[?]` → `[x]`. 70 of 93 rows are `human_validatable_only` (need a playtest, not more
agent work).

---

## 1. Status-marker integrity — 7 `[?]` rows are mislabelled

`[?]` means "agent proposes fix complete, human validates." These rows carry `[?]` but
have **no fix landed against their headline symptom** (adversarially confirmed). They
make the queue look more complete than it is and should be re-triaged.

| Issue | P | Why mislabelled (file:line verified) | Recommend |
|---|---|---|---|
| **#207** | P1 | Generic collision helpers `sub_21F8`/`sub_1FA2`/`sub_1F52` still `#[cfg_attr(not(test), allow(dead_code))]`, **zero live callers** — only a comment (`combat.rs:11274`) + a `debug_assert_eq!` ref (`blocks.rs:1398`). Live `apply_shooter_barrel_support_sub_1f52` (`blocks.rs:1348`) is a *different* function. | `[ ]` open |
| **#214** | P1 | `#214` archive block is symptom + "Minimum gate" only — **no "Fixed" line**, "Native repro: not yet recorded." Cited tests belong to broader #065 shooter work. | `[ ]` open |
| **#213** | P1 | `[Verify]` investigation block, "Native repro: partial," no fix. Same live bug as P0 **#260**. | `[ ]` / fold into #260 |
| **#205** | P1 | The object-active oracle harness it exists to deliver **fails oracle parity at frame 0** (native `number_objects=5` vs oracle 9; `level_special_effects=null` vs 1). Mechanism present, prerequisite incomplete. | `[ ]` / `[b]` |
| **#217** | P1 | Structural epic ("wire move-state helpers"); first pass mis-attributed #227's platform-escape fix to it. #209's dead-code sweep names #217 as owner of **~2442 still-dead helpers** (movement.rs 85 dead). Not one fixable bug. | re-derive into children |
| **#257** | P1 | The `$1C`-hold continue-slide sub-fix is real but doesn't touch the core symptom (death→black, no-input auto-timeout). Archive: "never promote to `[x]`; residual remains open." | keep `[?]` as the human-validation residual of #248 |
| **#126** | P1 | Slope facing fix present + tested, but the walk symptom unresolved and all cited routes stale. Duplicate of #258/#211. | mark duplicate |

---

## 2. Full triage — 16 rows need a status change

Distribution across all 93: **44 ready-for-human-validation, 33 keep-as-is**, and the
following 16 needing action:

| Issue | P | Now | Action | Reason |
|---|---|---|---|---|
| #126 | P1 | `[?]` | mark-duplicate | of #258/#211 (slope); routes stale |
| #155 | P2 | `[?]` | mark-duplicate | of #260 Drips (archive: "continue under #213") |
| #169 | P1 | `[?]` | mark-duplicate | of #260 Drips |
| #227 | P1 | `[?]` | mark-duplicate | of #208 (platform layer); **evidence stale** — carry helpers now live in `platforms.rs:915/941/951` |
| #252 | P2 | `[?]` | mark-duplicate | = #145 + #167 (RockTank family) |
| #205 | P1 | `[?]` | downgrade-to-open | harness fails oracle parity (§1) |
| #207 | P1 | `[?]` | downgrade-to-open | helpers still dead_code (§1) |
| #213 | P1 | `[?]` | downgrade-to-open | no fix landed (§1) |
| #214 | P1 | `[?]` | downgrade-to-open | no fix landed (§1) |
| #277 | P1 | `[?]` | downgrade-to-open | own evidence (archive 9415/9599) **reconfirms still broken** + disproves cited `loc_6E2` theory |
| #320 | P3 | `[ ]` | **close** | row claims "6 gaps" but camera audit doc marks **all 6 `[x]` fixed 2026-06-13** — stale |
| #185 | P1 | `[?]` | needs-rederivation | claims grab cosmetic, but #219 wired `KidGrabbedByHand` live (`enemies.rs:42612`, consumed `enemies.rs:18746`+`physics.rs:722`) — contradicts HEAD |
| #217 | P1 | `[?]` | needs-rederivation | structural epic (§1) |
| #141 | P1 | `[?]` | consolidate | into intro cluster #161/#210 |
| #161 | P1 | `[?]` | consolidate | into intro cluster |
| #210 | P0 | `[?]` | split | multi-symptom; stray-Kid + TM-origin fixed, Wild-Side jumble / green-screen / menu-text open |

**Also flagged (status reads too low):** #272/#287/#265 are plain `[ ]` despite
substantial landed fixes — they read like `[?]` fixed-candidates. #248's evidence cites
a renamed constant (7210→24210) and two removed test names — refresh to current names.

---

## 3. Deduplication / consolidation map

The backlog is heavily redundant. 70/93 rows are `human_validatable_only`, so the win is
**one canonical row + one playtest per theme**:

| Theme | Rows | Canonical / action |
|---|---|---|
| Drips slime / metal-block | #155 #169 #213 → **#260** (P0) | archive already says "continue under #213" |
| Wall-hug / held-input launch | #147 #249 #270 #277 | merge #249+#270 (same bug); **downgrade #277** |
| Intro / title corruption | #141 #161 #210 #254 #288 | consolidate #141/#161/#210; #254/#288 = skip-path pair |
| Death → black screen | **#248** + #257 | #257 = human-validation residual of #248 |
| Enemy render (RockTank) | #144 #167 #252 #259 (+#145 closed) | fold #252/#259 into #145/#167 |
| Audio (live-SDL surface) | #003 #160 #216 #242 #264 #265 #272 #287 | one "live audio device validation pass" — all blocked on a real device, not code |
| Diamond-Power / forms | #206 + #228 | consolidate (one subsystem) |
| Platform physics | #208 + #227 | consolidate; **both stale** vs HEAD |
| Highwater foreground priority | #215 + #256 | same z-order bug |
| Level-flow fixed-candidates | #292 #298 #299 #300 | **highest-confidence** ready-for-validation — all wired + tested |
| Combat/enemy contact boxes | #308 + #315 | overlap on per-sprite contact-box modelling |

**Meta-cluster (architectural root):** #176 #183-189 #203 #205 #207 #208 #217 #227 #228
#230 all share the "wire dead ASM-faithful helpers into the live step" root and one
blocker — **#205's missing object-active oracle harness**. #205 is the keystone: deliver
an oracle-authoritative object/enemy-active case and ~8 enemy/physics ports become
agent-closable instead of playtest-only.

---

## 4. Plans — concrete next steps

6 actionable now, 1 blocked. (#274 is already being executed by the active loop.)

### #289 — hell-theme wrong spawn (level-flow) · actionable · ~2 commits
- **ASM:** `kid.asm:23777-23834` `loc_11BD2`/`loc_11BE4`/`loc_11BEC` (player-start + teleporter `$FFFFFC29` preserve latch, `loc_11C7E` decode tail); `Load_InGame`/`loc_1AE2E` title-card→`Game_Mode=$000C` hand-off.
- **Plan:** route the unsnapped `begin_level_entry` branches (teleporter-preserved-start `level.rs:18347`, `enter_level_skip_hold` `level.rs:18459`) through the shared `level_respawn_slice_with_floor_snap` helper — kills "Kid briefly appears in the air."
- **Oracle split:** native vs oracle `kid_y_pos` at first live frame; oracle = `0x6F0000` grounded, no-snap branches leave Kid at raw header y descending over ~12 frames.
- **Test:** `hell_level_entry_first_live_frame_settles_kid_to_floor_loc_11be4` + `teleporter_preserved_start_entry_floor_snaps_loc_11bd2` (`level_tests.rs`).
- **Risk:** teleporter destinations *may* be mid-air by design — verify vs oracle, mark `// ORIGINAL_QUIRK` if confirmed. Dragon_Flying render residual is out of scope (exhausted).

### #256 — Highwater Rhino/Berzerker in front of water (enemy-render) · actionable · ~2 commits
- **ASM:** `kid.asm:2224-2233` `loc_1234` (sprite priority bit), `kid.asm:33900-33940` `sub_302F6`/`sub_3034A` (foreground Scroll-A `$0100`→`$8000` tile write), `level.rs:5688-5747` (`loc_124BE` priority decode). Berzerker is `objid=$84` — a **Kid helmet form**, so this is player-vs-foreground priority, not enemy priority.
- **Plan:** route-free static audit (all input routes are archived dead-ends, capped short of the waterfall mask). `diagnose-frame --level 02 --frame <waterfall> --save-layers`; check `block_grid.raw_word_at(col,row) & 0x8100` on the waterfall-front columns.
- **Oracle split:** native `final_high_priority=false` on waterfall pixels (player low-priority write accepted) vs oracle compositing them high-priority and masking Kid.
- **Test:** `highwater_waterfall_front_foreground_cells_are_high_priority_sub_302f6` (`level_tests.rs`) + `kid_berzerker_form_draws_behind_high_priority_waterfall_foreground` (`scene.rs`).
- **Risk:** fix the parse/emit engine-wide; if oracle confirms the front *is* low-priority, this is `// ORIGINAL_QUIRK` → mark `[-]`.

### #319 — boss-eye SFX cue (audio) · resolved 2026-06-16
- **ASM:** `kid.asm:38894` `sub_37002` (`sfx_Boss_eye_pops` burst), `kid.asm:41947` `sub_393B2` (eye detach, plays `$48`), 20+ callers `loc_38962..loc_3937A`; `sub_37708 loc_3772A` is the already-wired `$48` template.
- **Plan:** wire `$48` into the live BoomerangBosses eye-piece-detach branch (`combat.rs:6894` step, currently spec-only `boomerang_boss_eye_piece_*_launch_loc_38962` helpers). Mirrors the HeadyMetal pattern at `combat.rs:8092`.
- **Oracle split:** core-trace vs ASM — native `played_sound_effect_ids` omits `0x48` on the detach frame. **No audio device needed** (cue-presence assertion).
- **Test:** `boomerang_boss_eye_detach_step_queues_boss_eye_pops_loc_38962` (`combat_tests.rs`).
- **Resolution:** live BoomerangBosses death entry now queues `QueuedSoundEffect::BossEyePops` through the `sub_393B2` helper path; generic `sub_37002` remains tracked in `audio-triggers-fidelity-audit.md`.
- **Risk:** engine-level, low; guard against double-emission with an exact-frame assertion.

### #301 — HeadyMetal composite worker lifecycle (bosses) · actionable · ~1 commit
- **ASM:** `Enemy20_HeadyMetal_Init`/`loc_39266` (12 body-piece seeds), `sub_37ABC`/`loc_37AD4` (`kid.asm:39996` — walks persistent `$FFFFFB7E` table, **skips null pointers**), `sub_37708 loc_37778` (nulls a slot after breakup).
- **Plan:** `heady_metal_composite_refresh_probe_sub_37abc` (`trace.rs:7085`) currently **re-derives all 12 slots every frame** from static offsets, so a piece nulled by breakup reappears. Make the table **persistent**: popped slots become `None` and are skipped (ASM dbf-with-null-skip).
- **Oracle split:** native-internal (no boss route exists) — popped piece reappears next frame vs should stay gone.
- **Test:** `live_heady_metal_composite_refresh_drops_popped_worker_slot_sub_37abc_loc_37778` red-first unit test (`combat_tests.rs`).
- **Risk:** HeadyMetal family only; no boss frames in TAS so CRC impact nil.
- **2026-06-16:** executed the load-time persistence subcase in `0062a30f`. Runtime now seeds the `$FFFFFB7A..$FFFFFBAA` table from `Enemy20_HeadyMetal_Init loc_39247`; `live_heady_metal_body_piece_workers_persist_after_init_loc_39247` guards against re-projecting `sub_37ABC` pieces from a moved tracking anchor. Full per-piece `sub_376EC` lifecycle remains open in `bosses-fidelity-audit.md`.

### #274 — Cyclone transform render (blocks-prizes) · actionable · ~2 commits · *loop already added the regression*
- **ASM:** `kid.asm:8425` `loc_7CD0` Kid_Transform (`off_7EE2` transform-step table), `loc_7C96`/`loc_7CA6` frame-table walk; native selector `sprites.rs:675-741` `select_player_transform_frame`.
- **Plan:** render-only. State/camera match (`verify-tas` CRC-green); the split is the transform sprite/frame/palette across frames 3491-3494 (exact-AE 419→143). Fix `select_player_transform_frame` post-entry hibernate ordering vs `off_7EE2`.
- **Test:** `cyclone_transform_steps_3492_3494_match_off_7ee2_forward_frames` (`mod.rs`, sibling to `loc_7cd0_*` tests).
- **Risk:** scope strictly to per-hibernate forward-frame selection — **two prior palette experiments over-reached and were rejected.** Verify against a second form's transform window (not a Cyclone-only hack).

### #305 — pause Give-Up last-life lane (shell/verify) · actionable · ~2 commits
- **ASM:** `Game_Paused_Loop`/`Game_Paused_ChkStart` (`kid.asm:6829-6878`), `loc_6B46`/`loc_6C0A` (give-up death-windup), the ROM joypad read copying `Ctrl_1_Pressed` ($FFFFF80E) into active `Ctrl_Pressed` ($FFFFF813).
- **Blocker (investigation):** in the oracle's paused loop, active `Ctrl_Pressed` reads `$00` while raw `Ctrl_1_Pressed` correctly shows the Start edge `$80`, so `Game_Paused_ChkStart` never fires (oracle stays paused, `number_lives=3`). Next step is an **oracle joypad-mirror routing** investigation, not another route retiming (rejected twice). Secondary: `scenario.rs:89` hardcodes boot `number_lives=3` — last-life route needs a 2-life-loss prefix or a lives-seed field.
- **Test:** new `issue-305-pause-give-up-last-life-probe.json` oracle-parity slice once routing is fixed.

### #260 — slime / metal-block contact (drips) · **BLOCKED** · 0 commits
- **Key correction — the search has targeted the wrong mechanism.** `Drips.asm loc_33AF6`
  turn-tables prove the crawler only **bounces/turns** on `$6000` metal cells; it never
  produces a block Destroy. Its leftward projectile (`loc_338F8` → `Tar_Monster loc_325B6`
  burst) fires on **support-loss/wall death**, not block destruction.
- The reported "block breaks + red projectile leftward" matches the **type-C `$EC2D`
  Driller/shooter** (`Driller.asm Enemy1A_Driller_Init loc_3BCF0`, `loc_3BE60`/`loc_3BE78`),
  **not the Drips crawler.** This explains why dozens of archived Drips-crawler-vs-row-32
  near-misses dead-ended.
- **Next:** build a probe that approaches the `$EC2D` shooter cell from the LEFT (so the
  leftward shot is between Kid and the cell), seeded from the cleanest oracle-comparable
  prefix; or get the human to confirm which on-screen object they saw. **Do not** patch
  the Drips crawler to Destroy a `$6000` cell — that contradicts `loc_33AF6` (fidelity
  regression).

---

## 5. Cross-cutting observations

- **Staleness pattern (slope/wall/Drips clusters):** nearly all concrete reproduction
  evidence lives in `.cache` recordings/cases that **no longer replay** (archive
  7854-7856, 8020-8022, 8230-8245 are walls of "rejected — stale, drift at frame
  0/169/192"). The committed unit/regression tests are the durable evidence; the cache
  routes are not. Direct-probe bootstrap mismatches block the live well/slope/wall
  routes, which is why these can't be agent-closed.
- **Evidence quality is high overall:** across all 12 clusters, **every cited ASM label
  and every cited test name was found in current code** (the only "missing test"
  was `iron_knight_climbing`, which is a state field, not a test fn). No fabricated
  tests or labels. The mislabelling is about `[?]` overstating progress, not invented
  evidence.
- **#205 is the highest-leverage unblock** in the whole backlog — it gates the entire
  enemy/physics "wire-the-helpers" meta-cluster.
- **Citation drift to refresh (cosmetic, non-blocking):** #189 line refs
  (`enemies.rs:19793` now lands in Lion code; Orca impl moved to ~20707), #243 quirk ref
  (`enemies.rs:8811`→8973), #248 constant/test names, #292 `sub_14244`/`Init_SpriteAttr_HUD`
  not grep-found as comments though the logic is present.

---

## Appendix — method & caveats

- **Workflow:** `backlog-validate-plan` — pipeline(12 clusters → validate → adversarial-verify-suspicious → plan). 54 agents, ~4M tokens, 23 min. Read-only (no cargo); test/label existence checked by grep; the green-gate run separately covers the dynamic layer.
- **Caveat:** snapshot validated `ca222a3b`..`80c1793f`; HEAD has since advanced. Re-check a row's current marker before acting. The concurrent loop owns `backlog.md`, so this report is standalone rather than inline edits.
- **What this does NOT do:** human playtest validation (`[?]` → `[x]` remains a human gate per `loop.md`). 70/93 rows are `human_validatable_only`.
